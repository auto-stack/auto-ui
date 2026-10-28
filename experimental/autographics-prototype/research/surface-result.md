# PLAN-830 — 可组合 Surface/Portal 与嵌套输入：实验结果（2026-10-22 r6 work）

**一句话**：在独立 AGP 上，真实 iced 控件、2D 游戏、3D 场景可以互相嵌套消费
（UI→3D→UI、3D→UI→2D），输入沿窗口→挂载链→子内容逐级映射并正确处理捕获/
焦点/IME，共享内容每帧只推进一次，结构变更/失败资源/卸载不会留下残影、半图或
悬空状态。
**真实 OS 中文 IME 已验证**：`surface-ime-os` 窗口用例在真实窗口中让系统中文 IME
组合并提交，内层 `text_input` 收到 `text="abc你好你好"`（11/11 门禁，含
`os_ime_commit`、锚点随挂载、失焦清焦点、无焦点 IME 丢弃）。

## 1. 证据总览（全部本树可复现）

| 证据 | 入口 | 结果 |
|---|---|---|
| 契约测试 | `cargo test --test surface_contract` | **29/29**（身份/共享/DAG/环/深度预算/原子批次/结构脏传播/跨帧 alpha/坏资源/多相机/嵌套遮挡/焦点取消/锚点） |
| R2/R3/R4 永久回归 | `cargo test --test surface_review_r2 --test surface_review_r3 --test surface_review_r4` | **16/16**：R2 五项 + R3 六项（合批唯一焦点/动作不丢/拒绝无副作用）+ R4 五项（取消先于 pending 渲染、revalidate 卸载、取消后新点击/公开 focus 仍生效） |
| AGP 全量回归 | `cargo test` | **162 passed, 0 failed**（157 + R4 永久回归 5） |
| V-02 离屏 UI whole-path | `--case surface-offscreen --mode window` | **6/6**：真实 button 点击 + 键入 ok8（OS 事件路径），平面显示 UI，稳态 0 readback |
| V-02/03/04 合成 + S-01 | `--case surface-compose` | **17/17**：3 层 DAG、alpha 精确匹配、clip 切边、共享挂载单次穿透、S-01 外层 UI→3D→内层 UI 真实点击/焦点/DPI2/resize/像素 |
| V-03/04/05 输入/生命周期 | `--case surface-input --mode window` | **17/17**：真实 OS 点击穿过 3D 平面命中真实 button；嵌套 2D 独立输入；焦点唯一；IME 语义；锚点；捕获保持；销毁/失焦同时清 capture+焦点并 unfocus iced |
| V-03/04/06 场景 | `--case surface-scenarios` | **14/14**：S-02(全链路暂停/恢复/reset/文本-游戏焦点切换/单tick+double tick 负控)、S-03(共享 **Squash** 两挂载同像素+实例隔离+每挂载一次输入) |
| V-07/08/09 生命周期/负控/计量 | `--case surface-lifecycle` | **14/14**：20 轮 close/reopen view + hide/restore + 拒绝变更；真实 `build_seq/target_creations` 计数；7 项真实故障负控全部被捕获 |
| V-05 真实 OS 中文 IME | `--case surface-ime-os --mode window` | **11/11**：窗口聚焦、IME 启用、public focus 探针、真实键盘落字、preedit、**CJK commit**、锚点随挂载、锚点在投影区、失焦清焦点、无焦点 IME 丢弃 |
| V-09 独立消费/重定位 | `--case independent-consumer` | **pass**：closure/locked build/A→B relocation（29 字段无差异）/root collateral（相对 `ab56c32ec` 零生产改动；SD 草稿单列） |

负控实测（`surface-lifecycle` 报告 `metrics.negative_controls`，全部走生产路径）：
- clip：关闭 scissor 后，**quad 内、clip 外**的像素变洋红（`[255,0,255]`）；恢复后回父色。
- 单 tick/重复渲染：正常 1 次/面/帧；`force_double_render` 真实再调一次内容路径 → 观测到 2。
- alpha：正常 `[188,138,138]`；切到 opaque-replace 管线 → `[137,0,0,127]`；恢复后回正确值。
- 重复事件：一次 KeyPressed → `"a"`；运行时复制事件列表 → `"aa"`。
- 每帧 readback：5 次 `probe_pixel` → 计数 +5（稳态门禁仍要求 0）。
- 第二设备：真实 `GpuDevice::new()` 内容 → `DeviceMismatch`；**2D/iced/3D 三种
  adapter 逐一被拒**，raw-device 内容 → `DeviceUnverifiable`，图不变、不建 target；
  同设备三种内容正控成功。
- 路由 oracle 敏感性：把 router 返回的 local 改成 `[9999,9999]`（同一真实入口），
  场景门禁 8 项失败、整体 exit=1；不篡改时 15/15 exit=0。
- 依赖顺序：composite-before-render → `DependencyOrder` 分类；恢复正常帧成功。
- 加倍模拟（S-02）：正常 tick delta=1；`tick_n(2)` 真实第二步 → delta=2。

## 2. 架构落点（代码地图）

| 层 | 文件 | 职责 |
|---|---|---|
| GPU 无关组合 | `core/composition/mod.rs` | Surface/View/Portal 身份（复用 `core::id` 代际）、整批校验→整状态提交、环/深度预算具名拒绝、**结构变更脏传播**、`render_order/tick_order` |
| GPU 无关输入 | `core/composition/input.rs` | 命中链、clip/**递归遮挡**、capture 路径保持、**cancel 清 capture+焦点**、非有限输入拒绝、**按 host Surface 的 3D 投影** |
| GPU 合成 | `renderer/composition/mod.rs` | `SurfacePlanePipeline`(3D 平面)、`PortalQuadPipeline`(2D+可选 scissor/opaque 负控)、`FullScreenBlit` |
| 运行时 | `host/surface_runtime.rs` | DAG 驱动帧、内容适配器、**只对新鲜父面合成**、事务式 target add/resize、真实 target 计数、故障注入 hooks |
| 宿主输入 | `host/composition_input.rs` | window↔view、2D 反算、**按 host 相机的 3D 射线/UV**、活跃焦点路径锚点、取消/失焦生命周期、OS 注入辅助 |
| 离屏 iced | `host/iced_surface.rs` | 公开 API 驱动的离屏 iced（Engine/Renderer/UserInterface/present）、focus/unfocus 探针 |

## 3. 与 829 的兼容

- 829 的两个游戏（Breakout / Squash）规则、golden、探针命令均未修改；
  S-02 复用冻结 `BreakoutDriver`，S-03 复用冻结 `SquashDriver` + `build_scene`，
  未另写玩法。
- `OffscreenTarget::new` 保留 829 的非采样 usage；新增 `new_sampled` 才有
  `TEXTURE_BINDING`。`resize` 保持原 usage。
- 829 的 `ag2d-breakout` / `ag3d-squash` / `iced-shared-device` 复跑均 exit=0。

## 4. 计量（V-09，本机 CPU 侧，GPU 计时未接线）

`surface-lifecycle` 报告的 `metrics.composition` 给出 60 帧的 CPU 帧时
p50/p95/max、内容穿透/合成次数、脏跳过次数、readback 增量、**真实
target_creations/target_drops 与每面 build_seq**。20 轮只允许 20 次显式 resize
创建目标（10 轮 × 2 面），canvas（未 resize）的 build_seq 全程不变。

**上传**（`metrics.composition.uploads`，采样点=60 帧循环前后快照，单位=次/字节）：
- `plane` / `portal_quad` / `content` 三个**独立**分项（各自来自真实
  `queue.write_buffer` 记账），另给 `composition_path`（前两项之和）与
  `path_total`（三项之和）。本机实测：quad 66 次/10560B、path_total 84 次/
  11616B（content=1056B）。
- 门禁是**分管线精确**的：`quad_upload_counter_matches_composites`（改写次数
  与字节分别 == quad composites 与 ×160）、
  `plane_upload_counter_matches_composites`（平面专用控制块：== plane
  composites ×128）、`upload_total_is_the_sum_of_its_sources` 与
  `plane_control_total_is_the_sum_of_its_sources`（总量**严格等于**三项之和）、
  `content_upload_counter_is_exact`（新建空 canvas 一次内容穿透恰为 1 次/64B）。
- **省略计数的负控**（V-08 hook，真实 GPU 写入保留、只跳过记账）：
  `neg_plane_upload_omission_caught` 与 `neg_content_upload_omission_caught`
  证明对应遗漏会让同一 predicate 变红；恢复后控制块重新为绿。
- 明确排除：iced 内部上传（在 `iced_wgpu::Renderer::present` 内）、纹理/bind
  group 创建、驱动侧拷贝——报告字段 `excluded` 具名，不冒充已计量。

**内存**（`metrics.composition.memory`，采样点同上）：
- `process_working_set_mb_{start,end}`：本进程 OS 驻留（含 wgpu 驱动分配、iced、
  资产），**不是**归属内存，也不等于驱动自报。
- `owned_target_bytes`：运行时真正拥有的离屏目标字节 = Σ`w*h*4`
  （`Rgba8UnormSrgb`，按真实尺寸；驱动对齐/额外开销不含），与
  `owned_target_count/slots` 同时给出（本机 4 个目标 = 1,511,424B）。
- `gpu_memory_available=false` + `gpu_memory_unavailable_reason`：wgpu 27 无
  可移植的进程级 GPU 内存查询，DXGI budget/usage 属 adapter 专属且未实现；
  `driver_residency` 单独标注“未测量”，不与 RSS 混用。
- 门禁 `memory_facts_reported_and_separated` 要求三类事实齐备且相互分离。

GPU timestamp 明确为 `gpu_timing_available=false`，不伪造。

锁定构建与重定位：AGP `cargo check --locked` 通过、依赖闭包无路径依赖违规、
A→B 重定位比对 **29 字段 0 差异**；`collateral` 相对本 Plan 基面
`ab56c32ec4cb28e01fbcca74e9861164ffad43f0` 的根仓生产改动为 **0**
（两份 SD 草稿单列为 `provisional_spec_drafts`）。

## 5. 已知偏差与记录

1. **注入路径**：OS 输入优先真实前台点击/SendKeys；会话拒绝前台时回退
   PostMessage（仍走 winit 真实消息管线），报告以 `os_window_events` /
   `posted_queue` 如实标注。
2. **DPI**：注入进程必须 `SetProcessDPIAware()`；S-01 另有模拟 DPI2 的变换门禁。
3. **winit 不处理 WM_CHAR**：键盘注入需投递带 VK+扫描码的 WM_KEYDOWN/UP。
4. **3D 遮挡**：平面按声明序画家式合成；任意深度组合留给后续扩展（profile 明示）。
5. **Squash 共享场景的画面**：S-03 用冻结 `SquashDriver`/场景资产，但相机取景与
   829 的 golden 帧不同（且共享场景只渲染 20 tick、无 mob），因此**不与
   ag3d-squash golden 做逐像素对拍**；门禁是“两挂载同像素 + 与共享源同像素 +
   单 tick”。829 自身 golden 仍由 `ag3d-squash` 独立守护。
6. **解引用顺序边界**：卸载后**不** revalidate 直接 release 会返回旧 target；
   显式 revalidate 对照通过。宿主生命周期已接 revalidate/cancel，但 API 不禁止
   显式调用方保持旧顺序——这是调用约定，不是已修产品缺陷。

## 6. 实施中发现的真实缺陷（已修，记录在案）

1. **一个 surface 两块纹理**：`SurfaceRuntime` 的 slot 目标与 `IcedSurface`
   自有目标是两个不同纹理 → 组合窗口里 iced 内容恒为空。修为
   `frame_into(&slot.target)` + `new_borrowed`。
2. **`text_input` 没挂 `on_input`**：控件能画、能聚焦，但输入被丢弃。已补。
3. 测试自身 u8 溢出导致 clip 门禁假阴性，已改 i32。
4. **结构变更不置脏**：SetMount/RemovePortal/RemoveView/SetSurfaceSize 后父面
   仍 clean → 卸载残影、resize 空白。已修（submit 合并 affected + 消费者链）。
5. **clean 父面逐帧重复叠半透明 Portal**：合成无条件执行 → 每帧变色。已修为
   只对“本帧重新渲染过内容”的父面合成。
6. **GPU 资源失败先提交图**：AddSurface/resize 先改图再建 target，超限触发
   wgpu panic 并留下半提交。已修为 target 先行 + 前置限额检查 + 具名错误。
7. **3D 命中遍历所有父相机**：共享子面在双父面下 15/24 错映射。已改为按
   host Surface 的相机解析，未注册即 miss。
8. **递归遮挡被父面吞掉**：内层非交互遮挡返回 None 被当成“无更深命中”，
   回落到父内容。已改为沿整条链传播 Occluded。
9. **取消不清焦点 / 无焦点 IME fallback**：失焦后仍保留键盘目标，IME 固定投给
   `ui_surface`。已修为 cancel 清 capture+焦点、宿主 unfocus iced、无焦点丢弃
   并计数。
10. **IME 锚点取 BFS 第一挂载**：共享内容两挂载时锚点落在第一个 Portal；根框
    返回 None。已改为活跃焦点/捕获路径 + 根 Surface 空路径。
11. **生命周期/负控自证**：target 重建靠循环自增、alpha 期望是常量、第二设备
    只匹配字符串。已改为真实 build_seq/计数、真实 opaque 管线、真实第二设备。
12. **独立消费基线过期**：`independent-consumer` 仍钉 829 起始提交
    `fdb1fad2ce`，在 830 分支必然红。已改为默认 `ab56c32ec`（可 env 覆盖）并
    显式区分 SD 草稿。
13. **活体内容不刷新消费者（r2 F01）**：iced 真实 message 更新了子面输出，但
    clean Canvas 消费者仍显示旧纹理（旧逻辑只在事件前冻结 dirty）。已修为
    adapter 报告“本帧输出变化”（message/捕获事件/重绘请求/布局/真实 caret、
    3D 内容指纹），运行时在同帧把消费者链置为本帧需重采样。
14. **焦点迁移只改 router（r2 F02）**：A→B 点击后两个真实 text_input 都保持
    focused，`window_focus_lost` 只清最后一个。已修为 router 记录
    `FocusTransfer{from,to}`，运行时在同帧渲染前释放其他 iced 面的真实焦点并
    关闭其组合态；公开 focus 调用同样强制唯一目标。
15. **IME 锚点仍是固定点（r2 F03）**：adapter 丢弃 `UserInterface::update` 的
    `State`，宿主用 `[40,26]` 当 caret，正式门禁拿整个平面当 caret 区。已修为
    每帧注入 `RedrawRequested`、取 `InputMethod::Enabled.cursor`、按真实 caret
    矩形投影；OS 锚点每帧跟随输入/选择/滚动。
16. **3D 内容绕过设备归属（r2 F04）**：`Scene3d` 的 `device_ordinal` 恒为
    `None`，外设备 MeshPipeline 被 Ok 接纳。已修为 MeshPipeline 记录 ordinal、
    三种 adapter 逐项校验、无法证明归属即 `DeviceUnverifiable`（fail closed）。
17. **resize 替换目标不计 drop（r2 F05）**：`slot.target = new` 释放旧纹理却只
    `target_creations += 1`。已修为创建即计、替换/丢弃准备/slot 删除均计 drop，
    并加 `live_targets()==live_target_slots()==live_surfaces()` 守恒门禁。
18. **场景门禁绕过 router 输出（r2 F06）**：点击用了预设 local，按键目的地写死，
    `text_after_game_focus` 根本没发键。已修为控件事件消费 router 返回的
    `(surface, local)`、按键按 router 目标派发（宿主映射游戏动作）、边界/往返/
    目的地断言 + 故障注入自检（9999 变异即红）。
19. **合批输入留下双焦点（r3 F-830-R3-01）**：runtime 从 HashMap 里挑一个
    ButtonPressed 当目标，但同帧每个事件列表都会跑、各自重新聚焦；A→B 两次点击
    攒进一帧后两个真实文本框都 focused，blur 后旧面仍残留。已修为由 router 的
    终态（`CompositionGraph::note_routed_focus`）决定本帧唯一编辑目标，非目标面
    在**绘制前**清除真实焦点与组合态（事件本身仍投递，先前点击的动作不丢）。
20. **拒绝请求有副作用（r3 F-830-R3-02）**：公开 focus 先释放其他全部 iced 焦点、
    再查目标 slot；跨 session/stale/非 iced Surface 被拒时旧合法焦点已被清掉。
    已修为“先校验目标、后迁移”；拒绝不动当前焦点与 IME 状态。
21. **V-09 上传/内存证据缺失（r3 F-830-R3-03）**：lifecycle 报告只有 12 个字段，
    没有组合路径上传量与内存事实。已修为两条组合管线 + 内容适配器真实
    `write_buffer` 记账、进程工作集与 owned 目标字节分开报告、GPU/驱动驻留具名
    不可用，并加三项门禁（上传=合成数×固定字节、路径总量覆盖组合量、平面计数器
    为活的控制块）。
22. **计量门禁可被其他管线顶替（r4 F-830-R4-02）**：平面活控只看“合计 ≥1 次/
    128B”，quad 的 160B 即可冒充；内容汇总量只用 `>=` 比较，完全漏汇总仍过。
    已修为分管线精确断言（`plane`/`portal_quad` 各自 == composite 数 × 固定字节）、
    总量严格等于三项之和、内容适配器 64B 精确活控，并加两组 V-08 省略计数负控；
    外部注入“跳过平面 record / 跳过内容汇总”均使门禁红、exit=1。
23. **取消先于 pending 渲染时焦点复活（r4 F-830-R4-01）**：宿主在 about_to_wait
    才 flush 事件，失焦/revalidate 若发生在路由之后、渲染之前，runtime 把
    “无目标”当成“允许所有面聚焦”，两个真实文本框重新 focused。已修为 router 目标
    三态 `Unset/Target/Cancelled`：取消后（含路径销毁的 revalidate 与
    `release_iced_focus`）任何 pending 批次都不得产生编辑态，直到新的合法点击或
    公开 focus；新点击/公开 focus 仍正常生效。

## 7. 仍未取证/未实现（不冒记完成）

- 生产 AutoUI/RQHost/Forge 接线、AutoShader/CAP 实现：按计划保持 pending。
- 任意 3D 深度遮挡、GPU timestamp 计时、候选窗逐像素模板比对。
- 组合管线创建未安装 error scope（见交接 §2/§3）。
