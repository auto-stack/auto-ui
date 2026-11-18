# PLAN-089 T-01 决策记录

按[执行与视觉指南 §5](../../../../docs/design/autoui/common-components-execution-and-visual-guide.md) 结构记录。机械决策随证据落在本页；语义仍以 Plan §5 r2 表为准。

## 1. baseline

| 项 | 值 |
|---|---|
| 仓库 / 树 | `auto-ui` @ `D:/autostack/.wt/ui-089/auto-ui`（组 `D:/autostack/.wt/ui-089/`） |
| 分支 / base | `codex/ui-089-image-viewer`，base = master `ba92217bfc1607e713feaf803e04e99655684d37` |
| 依赖兄弟树 | `auto-lang` @ `0b6d3198d`、`auto-web` @ `fd115ce0`、`auto-down` @ `2724389b`（均 master HEAD detached，与主检出同基线；auto-ui 工作区经 `../../../auto-lang|auto-web|auto-down` 相对 path 解析） |
| 观察源码 | `crates/auto-ui-compiler/schema/aura.at` imagesurface 块（:748-771）；`crates/auto-ui-runtime/src/ui/iced/image_surface.rs`（geometry/normalize）；`crates/auto-ui-runtime/src/ui/view.rs`（:1082 ImageSurface 节点、:2110 image_surface_events）；`crates/auto-ui-runtime/src/ui/iced/renderer/{mod.rs:1940-1978, projection.rs:3460-3486, media.rs:49-183}`；`crates/auto-ui-runtime/src/ui/session.rs`（PLAN-095 loaded/error 门表）；`crates/auto-ui-compiler/src/ui_gen/vue/{template_media.rs:2014-2044, script.rs:655-684, shadcn_attrs.rs:5128-5166}` |
| 局部 wrapper 摘要 | `examples/ui/031-image-viewer/src/front/image_viewport.at` SHA256 `84d748abdfae2a7b5bfca37592b7c8757b3af5250e14011e6d4b7e914b552525`（与 Plan §4 记录一致） |
| 新增路径 | `packages/image-viewer/`、`examples/image-preview/`、`crates/auto-ui-tools/tests/image_viewer_component.rs`、`tests/product/verify_image_preview.py`、`docs/specs/ui/image-viewport.md`（worktree 草拟，T-08 沉淀）、本目录 |
| 主检出状态 | 文档类未提交改动为其他任务既有 WIP；`crates/auto-ui-runtime/test/` 下未跟踪文件仅为 terminal_pixel 快照 PNG（测试瞬态产物），非源码 WIP，本计划不触碰 |

## 2. public interface（对 Plan §5 r2 表逐项核对）

- **事件承载机制**：包组件用带参 `msg` 通道（FileDialog/NumericField 先例）：`"viewRequested"(str)`、`"loaded"(str)`、`"error"(str)`、`"navigateRequested"(str)`、`"cancelled"`；消费方 `onviewRequested: .X($event)`；组件内 `."viewRequested"(json.stringify({...}))` 派发。跨后端一致（`.at`/SFC emit 层），不依赖 builtin 事件系统。
- **props**：按 r2 表 camelCase + 默认值（`fit:str="contain"`、`zoom:float=1`、`showToolbar:bool=true` 等）；底层 imagesurface 的 `src/alt/fit/zoom/offset_x/offset_y/rotation` 原名映射，不改名。
- **受控投影**：沿 NumericField 模式——Tick(interval) 对比 `revision`/受控 props 快照，变更才重投影；组件不发"自动重置"业务请求。
- **fixture / 生成**：探针 consumer + 生成结果见 §3；tools 门 T-05 落 `image_viewer_component.rs`（沿 `numeric_fields_component.rs` 双生成器 + vendored 闭包 + 具名缺依赖失败模板）。
- **loaded 最小体**：`{resourceId, resourceRevision}`（仅组件 props 已知字段，不捏造 w/h/字节数）；error `{code,message}` + 资源身份。

## 3. mechanism probe

### 3.1 已沿源码核实的链路事实

1. **规范变换**：`image_surface_geometry`（iced/image_surface.rs:58）——contain/width/one-to-one/free × zoom(clamp 0.05…64) × offset × rotation(rem_euclid 360)；命中/外接盒 `rotated_bounds`。这是唯一变换算法，按钮/滚轮/拖拽共用。
2. **规范事件负载**：`normalize_image_surface_event`（:242）——`Wheel{delta_y,x,y}`、`Pan{dx,dy,phase(Start/Move/End)}`、`DoubleClick{x,y}`、`Loaded{width,height,revision}`、`Error{code,message,revision}`；`ImageSurfaceInputState` 按 src revision 失效 pan/单次门——**迟到跨图事件在底层已丢弃**（r2 "旧 loaded/viewRequested 不落新图"的低层支撑）。
3. **Vue 侧已带参**：vue 生成器按绑定处声明参数生成全参调用（template_media.rs:2014-2044——pan 三相 `(dx,dy,phase)`；`handler_to_function_call_with_params` shadcn_attrs.rs:5160）；`@wheel/@dblclick/@load/@error` 齐全（image_surface_contract.rs:107-121 断言）。
4. **native 侧缺口（具名）**：Plan 547 Task 25 的 input adapter 不在树内——`View::ImageSurface.on_wheel/on_pan/on_double_click` 是无参 `Option<M>`，renderer 转换 `IcedMessage::from_dynamic` 丢参（renderer/mod.rs:1972），paint pass 显式忽略（projection.rs:3480 注释 "consumed by the input adapter (Task 25)"）；`render_image_surface`（media.rs）只渲染不含任何输入包装。**native wheel/pan/dblclick 事件今天不会到达 `.at` handler**；loaded/error 经 session.rs PLAN-095 per-App 门表另行派发。
5. **031 wrapper 缺陷证实**：`image_viewport.at` 把 `Wheel/Pan/DoubleClick` 以 `on_wheel()` 等无参转发（二次丢参）；且 031 自己的 `app.at` 并不消费该 wrapper（直接用 image_surface）——纯 shadow，T-04 移除。
6. **包消费机制**：`use { package: x from "<相对路径>" }`（component_registry.rs load_package，Builtin > Local > Package）；`package.at` 清单可选；`pac.at` 为应用描述符。包脱离主源根可解析（numeric_fields_component.rs vendored 用例先例）。

### 3.2 适配决议（r2 "修正适配"的落点）

- **native 输入适配（本计划内补）**：在 native ImageSurface 渲染路径补输入包装（PointerArea/Plan 499 先例：bounds 现场 + `shell.publish` 闭包），复用 `normalize_image_surface_event`+`ImageSurfaceInputState` 纯函数，把 Wheel/Pan/DoubleClick 规范负载以 `DynamicMessage::Typed{args: Vec<Value>}` 派发——`.at` handler 声明参数（`.OnWheel(delta float, x float, y float)`）即收坐标，与 Vue 全参形态对齐。不改 `image_surface_geometry`/`normalize_image_surface_event` 语义，不改底层接口名。
- **包组件不依赖 builtin 负载也能工作**：工具栏全部为普通 `button onclick`（MCP/键盘可操作），v1 按钮路径优先；手势路径（wheel/pan/dblclick）为增强通道，两后端行为对齐后 V-01 才判 pass。
- **变换一致性**：按钮缩放/平移与手势共用同一投影公式（viewport 中心锚 + fit_scale×zoom），在包内以整数有理化实现按钮步进、以事件坐标实现手势，均产出同一 viewRequested tuple；不得出现两套算法。
- **失败控制**：非法 fit/非有限 zoom → `invalid_view` error 事件，不派发 viewRequested；zoom 夹取 0.05…64 与 `image_surface_geometry` clamp 一致（夹取后同值零事件）。

### 3.3 整链探针（已回填，2026-11-17）

命令与结果（全部 dev/debug profile，本机 Windows，worktree `D:/autostack/.wt/ui-089/auto-ui`）：

- [x] `cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity` → **2/2 pass**（0.35s 执行；contract 测试已升级为断言输入三臂回调的负载语义：无参绑定零载荷、declared 形态追加 `(dx,dy,phase)` 字符串相位）
- [x] 最小 consumer（vendored 包 + App 绑定 viewRequested/loaded/error/navigateRequested）经双生成器 → `cargo test -p auto-ui-tools --test image_viewer_component` **2/2 pass**（5.63s）：rust/vue 生成均含 ImageViewport/ImagePreviewDialog；vue 面断言 `JSON.stringify({ seq:`/`baseRevision:`/六个 source 标记/`direction: previous|next`/`deltaY`/三相 pointer 包装/`@dblclick`；严格模式缺依赖 = 具名失败
- [x] 解析探针（bisect 证据，d1/d2 变体）：`.at` 对象字面量中 **`view` 是保留字**——r2 载荷键 `view` 必须以带引号 `"view": {...}` 输出（`json.stringify({"view": {...}})` 解析通过且输出 JSON 键名不变）；裸 `view:` 键报 `Expected key, got View`。已落地到包源码（16 处）
- [x] `cargo test -p auto-ui-runtime --lib -- image_surface mouse_area onseek progress` → 17 pass（1.75s，改动面相邻回归）；`image_surface_vm_builder` 单测已升级含 declared-pan 载荷断言
- 基线观察：改前同命令无法在新 worktree 运行（依赖兄弟树缺失）；改前语义基线以 `93e6b92..ba92217b` 主检出既有测试为准，本提交前 contract/parity 在改动后代码上全绿（无前置红灯）
- 提交：`072b934e`（runtime 适配 + 包 + 示例 + tools 门 + 本页）

## 4. dependencies

| 依赖 | 交付状态 | 本计划用法 |
|---|---|---|
| ImageSurface / media ticket（547 系） | 已交付（本仓） | 唯一资源/解码通道；不改 media 服务 |
| NumericField 受控投影模式（087） | 已交付（packages/numeric-fields） | Tick 快照 + seq/baseRevision 载荷模板 |
| FileDialog/PaginationControl 事件与 modal（084/086） | 已交付（packages/file-dialog、pagination） | 带参 msg、dialog 组合、`onaccepted: .X($event)` 接法 |
| 087 数值组件（Plan §3 提及"087为可复用优化而非硬前置"） | 已交付 | 不作硬依赖；缩放数值输入首版用普通 button/input |
| 媒体 fixture（031 tests/fixtures） | 已交付 | rgb-1x1 三格式、orientation-6、alpha-2x1、corrupt.bin、scenic-320x240 + manifest.json；T-04 另补非对称角点图 |
| 跨仓媒体服务变更 | **无** | 发现需要时具名返回，不在本计划内写 |

## 5. runners

| 轨道 | 入口 | 状态 |
|---|---|---|
| 原生回归（单测） | `cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity`；tools 门 `cargo test -p auto-ui-tools --test image_viewer_component` | 可用（本机 Windows；构建窗口受他组编译占用约束，见 §3.3） |
| native 产品门 | `python tests/product/verify_image_preview.py --auto-bin <auto> --ui-tool-bin <tool> --out-dir <isolated> --backend native`；会话 = `auto run -r vm` + `AUTOUI_MCP_PORT` + MCP HTTP（autoui_snapshot/action/keyboard），就绪文本/退出控件按本例适配，不复用 FileDialog Session 构造器（沿 verify_numeric_fields.py:60-130 模式） | Windows 可用；Linux runner 本机无 → 单列 not_run |
| Vue 产品门 | 031 先例 `tests/vue-actions.json` 经标准 Playwright runner；030 `tests/`（playwright.config.ts + smoke.spec.ts）为启动参考。**旧 030 媒体盘/autoplay 配置不照抄** | 本机 chromium 可用性 T-05 实测后回填；不可用则该轨 not_run，不并入总绿 |
| 031 vm_matrix.py | 含旧二进制默认路径——runner 显式 pin `--auto-bin/--ui-tool-bin`，不拿旧默认路径作证据 | 已确认风险，T-05 规避 |

## 6. next

- 允许进入：T-02（`packages/image-viewer/` 抽取 + native 输入适配实现）。
- 前置缺口：无阻塞性缺口；§3.3 探针回填是 T-02 关闭条件之一。
- 未决机制：无混入 T-02 的自由设计——native 适配按 §3.2 决议执行；若实现中证明 `DynamicMessage::Typed` args 无法从渲染层派发，回本页增补决议（不 silent 改合同）。

## 7. T-05 运行期发现（2026-11-17/18，产品门联调实录）

| # | 发现 | 证据 | 决议/状态 |
|---|---|---|---|
| P1 | split 模式下 back 进程票据不入前进程注册表——前进程 image_surface 永远 `media ticket missing` | demo 早轮 `VM-MEDIA ... OnError` + 前进程空注册表；probe 单测 `open_session→uri` <250ms 证明运行时本身自洽 | demo 改走前端进程内建 `auto.image` 会话（单进程渲染）；back 保留为 031 同构消费面 |
| P2 | 前端 VM 进程对相对夹具路径不解析——`tests/...`、`../tests/...`、`examples/...` 五种相对形态全部空索引；绝对路径可用 | probe 多候选探测（`|tests/...:empty|...`）；demo18 绝对路径 loaded=2 | gate 期原地重写三个夹具字面量为绝对路径，finally 字节恢复（restore 校验）；提交源保持相对形态 |
| P3 | App 级 Tick 需显式 `timer { Tick (every_ms) }` + msg 变体；组件 `interval` 约定只覆盖子件 Tick | demo7 `timer: event 'Tick' has no matching msg variant`；probe ticks=79/14s | demo 落显式 timer；包组件沿用 interval 约定（子件 Tick 实证走通） |
| P4 | `view` 是 `.at` 保留字：对象字面量裸 `view:` 键报 `Expected key, got View`；表达式成员位 `.view` 被 DotView 词元吞掉（静默取空） | 解析探针 d1/d2；`req.view.fit`→空 vs `req.source`→正常（probe3 A/B/C） | 包载荷以带引号 `"view"` 键输出（键名不变）；消费方 `req["view"]` 索引读取 |
| P5 | `auto-ui-tool.exe` 是 split 会话的 VM UI 宿主（auto.exe 的 [tool] 分支 spawn）——runtime 修复必须重建 auto-ui-tool，否则旧行为继续 | 旧串 `skipping dispatch` 在新 auto.exe 会话仍出现；字节搜索两 exe：tool 缺 marker、auto.exe 有 | 重建双二进制；gate 显式 `--ui-tool-bin` 指向同源构建 |
| P6 | 【已定位并关闭】三层叠加根因：(a) 探针与演示文件在长调试中被多次实验性改写互相污染（OpenALiteral/ProbeOpen/open3 残留、pac 被覆盖为 timer-probe），(b) 前端 VM 进程 cwd 非项目目录——包 use 与夹具的**相对路径全部不解析**（组件不挂载/索引为空；A/B 实证：绝对包路径 mounted、相对不 mounted），(c) MCP press 与首帧 vnode 登记竞态（首 press 静默 no-op） | PLAN089SWEEP 树转储（组件子树 0 节点 vs probe16 修复后 4 节点/loaded=1）；demo18/30/31/32 对照 | (a) 实验文件全部还原并提交固化；(b) gate 期原地重写包 use + 夹具为绝对路径（finally 字节恢复；提交源保持可移植相对形态），031 V-04 腿同法；(c) settle + 重试（press_until）。修复后 native 产品门 **RESULT: pass，16 pass/0 fail/3 not_run**（V-03 对话框内 MCP 腿与 V-04 轨道缺口单列，见 §7 P8/P9） |
| P7 | 机器占用：lang-980（ace_ts_provider）与 plan-982（p982_* 测试）编译期间按纪律等待并通知；未并行构建 | 进程 CommandLine 记录（23:23-23:24 起跑） | 构建窗口错峰执行 |

### 3.3 补充：已回填探针结果（更新）

- `cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity` → 2/2 pass（contract 升级为断言输入三臂回调负载语义）
- `cargo test -p auto-ui-tools --test image_viewer_component` → 2/2 pass（5.63s）
- `cargo test -p auto-ui-runtime --lib -- image_surface mouse_area onseek progress` → 17 pass（1.75s）
- native 链路里程碑：demo18（split 会话）全链实证 loaded=2（双实例单次）+ viewRequested tuple 接受（rev 推进）——组件/运行时合同面成立；产品门剩余阻塞见 P6

| P8 | 【r3 更正：非 PLAN-084 收窄行为】dialog 开启时 `press('放大')` 被 modal scope 拒绝——**根因＝同名双实例按钮的标签歧义**：快照首匹配落在树序前部的内嵌实例（活动 dialog 外），modal scope 正确拒绝该目标（背景隔离如期工作）；同 modal 内 `关闭`（dialog 头）press 成功证明 modal scope/包子树识别无缺口 | V-03 对话框内 放大 press → `Modal dialog owns input; target is outside the active dialog`；同腿 `关闭` press pass（cancelled=1） | r3 修复：gate `press(occurrence='last')` 消歧取 dialog 实例（正腿＝双实例变换独立 oracle）+ 首匹配显式负控（背景拒绝 oracle）；V-03 两腿转正 |
| P9 | 031/029 back 首次生成 + 编译耗时（约 2-4 分钟/个）——Session ready 截止已按会话序号放宽（V-04 腿 600s）；029 的 V-04 腿在其 back 编译缓存冷启动时仍可能超时 | V-04 029 legs 首轮 `app never rendered` | 已放宽至 600s；若仍超时记 not_run（单平台缺口，不并入总绿） |

## 8. T-05/T-06 证据汇总（2026-11-18）

### 8.1 native 产品门（demo, 前端 auto.image 会话拓扑）

`python tests/product/verify_image_preview.py --auto-bin <auto-man/target/debug/auto.exe> --out-dir plan-artifacts/plan-089/gate-native --backend native`
→ **RESULT: pass | 16 pass / 0 fail / 3 not_run | 709s**（report.json 含逐 case 计时）。

- pass：V-01 open→loaded(5.5s)/放大 zoom=1.25(2.6s)/宽度/↻90°/**角点像素在帧内**(1.7s)/方向平移+40(2.2s)；V-06 tuple/换图原子重置/主题零重置；V-02 损坏具名 error/重试/恢复(共 3.1s)；V-03 modal 开关+cancelled(2.7s)；V-05 退出重开(8.9s)；夹具 sha256 不变(1.3s)。
- not_run：V-03 对话框内 MCP 腿（modal scope 收窄，P8）；V-04 031/029 腿（见 8.3）。
- 证据物：`plan-artifacts/plan-089/gate-native/report.json` + `v01-corners-native.png` 等 4 张截图。

### 8.2 vue 轨

首访页面渲染成功（图片预览演示 + 组件工具栏全量 + 平面键受控回路 UI 全部在 DOM）；
媒体 open 后 loaded 未达——**根因 P10**：`auto.image.*` 是 `__vmOnly` VM 专属内建
（vue.rs natives 层，浏览器运行期具名抛错），浏览器无法进行本地文件会话。
031 的 vue 轨媒体走 back.api HTTP（back 进程建立会话、vite 代理 /api 拉流）——
而该形态在 native split 下前进程注册表缺票（P1）。**单源双轨的媒体会话需要
split 拓扑下的前端注册表喂给机制（media proxy/feed），超出本计划范围——具名
后续依赖（P10）**。vue 轨产品门记 not_run（单平台缺口，`gate-vue/report.json`）。

### 8.3 V-04 031/029 腿（r3 更正：按原始证据归因）

- 031：**根因＝P1，非"driver 时序差"**。session-3.log 实证组件已挂载、URI 正常
  （`/api/__auto/media/2858e0…/1`），但常驻 `status=Failed("media ticket missing")`
  ——back 进程票据不入前进程注册表（split/merged 拓扑下 back 持票）。早期手记
  "手动诊断两次到达 ready" 无法复现且与本日志矛盾，以日志为准。修复依赖具名
  后继机制（split 前端注册表 feed/媒体代理），r3 已裁定收口轨改为 031 的
  back-HTTP vue 轨（V-04R）。
- 029：back 首次生成+编译发生在**计时 case 内**（608.62s，超 600s deadline）——
  除冷启动耗时外，本身即 test-discipline 违例（日测不自建前置）。r3 已改为
  显式 pre-gate 准备段（prewarm），计时腿只消费已物化产物。

### 8.4 首轮（2026-11-18 r2）测试模块/耗时表——**r3 更正版**

> 更正说明：本表初版记"最大单 case 8.9s、全部 ≤10s 门内"，与其自引
> report.json 原始值矛盾（T-07 复审 F-05 实证）。以下为按
> `gate-native/report.json`（sha256 前缀 83c88d5241c51695）逐 case 原值
> 的更正版；r3 重跑后的新表见 §8.5。

| 模块 | 结果 | 准备 | 发现 | 测试 | 清理 | 总计 |
|---|---|---|---|---|---|---|
| auto-ui-tools image_viewer_component（2 case） | 2/2 pass | 增量编译 | ~1s | 9.0s | - | ~10s |
| auto-ui-runtime image_surface_contract + image_pipeline_parity | 2/2 pass | 增量 | ~0.2s | 0.35s | - | ~0.6s |
| auto-ui-runtime lib（image_surface/mouse_area/onseek/progress 过滤 17 case） | 17 pass | - | - | 1.75s | - | ~2s |
| 产品门 native（19 case：16 pass/0 fail/3 not_run） | pass | back 生成+编译 ~40-90s | ~30s | ~350s | ~10s | 709.4s |

首轮新增 case 计时**原值**（report.json）：最大 **16.49s（V-05 退出重开）**、
**10.13s（V-01 open A，含会话拉起）**——两例超 10s 新 case 门＝F-01；not_run
尝试耗时 54.99s（031）/608.62s（029，含在测 back 编译）。首轮计时缺陷的结构
原因：`Case` 计时基线早于会话拉起（case 时长吸收 bring-up）+ V-05 终止/重开
未拆分。r3 修复：`Case.pause_timer()`（拉起入准备段）+ V-05 拆分为
"退出终止"与"重开就绪"两案 + prewarm 准备段。

### 8.5 r3 重跑（2026-11-18，F-01/F-02/F-04/T-09 修复后——最终证据）

**修复落地面**（r3 复审合同 F-01…F-05 + T-09）：

- **F-01 计时合规**：`Case.pause_timer()`（会话拉起＝准备段，入 `report['prep']`）；
  V-05 拆分为"退出终止"+"重开就绪"两案；vue 重开链改两段构造
  （`defer_render`/`render_ready`——拉起 12.5-14.3s 入 prep，渲染就绪 case
  0.37s）；029 照片集/AUTO_PHOTO_ROOT 与 031 夹具重写入 pre-gate 准备段
  （`prewarm_vue`，冷物化 15-110s 不进计时 case）；031/029 pac 端口 gate 期
  改绑（3031 落 Windows 保留段 → vite `listen EACCES`）；`taskkill /T` 整树回收。
- **F-02 对话框内 oracle**：根因＝**同名双实例按钮的标签歧义**（快照首匹配
  落在内嵌实例，modal scope 正确拒绝）——非 PLAN-084 行为、非包子树识别缺口
  （同 modal 内 `关闭` press 一直可达）。gate 改 `press(occurrence='last')`
  取对话框实例＋首匹配显式负控（背景拒绝 oracle）＋挂载稳定门（快照 ticks
  掩码后连续两帧一致再压，vnode 登记竞态 flake 消除）。
- **F-04 vue 轨**：渲染诊断面（pageerror/console/html 全落入失败消息）后，
  vue demo 每轮正常渲染（首访空体未复现；09:46 失败归因孤证不立）。
- **F-05 记录更正**：§8.3/§8.4 按原始证据重写（见上）。
- **T-09 V-04R**：vue 收口轨全绿（见下表）；native split 腿记 P1 具名
  not_run（不空耗 doomed 试跑，证据冻结于 session-3.log）。

**新增发现 P11（vue codegen，已修复＋回归测试）**：lucide 图标 import 与
.at msg handler 同名冲突（`ZoomIn`/`ZoomOut`）——模板图标元素解析到 handler
函数被当作函数式组件每次 render 调用，发出 source=zoom 请求风暴（031 实测
203 条、视图缩到 0.05 下限、画面成 4×3 像点）。修复：`lucide_component_name`
统一 `Lucide` 前缀别名（import `ZoomIn as LucideZoomIn` 对应）；编译器 vue
测试 342/342 绿＋新增回归 `plan089_lucide_icon_handler_name_collision_aliased`；
金样 012_icon_state/desktop_surface_asset 同步。此为 vue 收口轨的阻断级发现，
属本计划 runtime/codegen 修复面（P6 同族），已随 r3 落地。

**最终门（同一冻结基线：compiler 修复后重建 auto.exe@auto-man cab347b + 最终
gate 脚本）**：

| 门 | 结果 | case 构成 | 总耗时 | 最大 pass case |
|---|---|---|---|---|
| 产品门 native（`gate-native-r3/report.json`，sha256 前缀 765de8e52cfda9d2） | **pass** | 19 pass / 0 fail / 2 not_run（V-04R native split＝P1 具名边界） | 46.8s | 7.34s（全部 ≤10s 门内） |
| 产品门 vue（`gate-vue-r3/report.json`，sha256 前缀 d0cc5cd9b3f17b1b） | **pass** | 16 pass / 0 fail / 18 not_run（demo 媒体腿＝P10 具名边界；功能矩阵由 V-04R 承担） | 176.5s | 全部 ≤10s 门内 |
| tools 门 image_viewer_component + tree_view_component | 4/4 pass | — | ~9s | 7.54s |
| runtime image_surface_contract + image_pipeline_parity | 2/2 pass | — | ~0.4s | — |
| compiler --lib（vue 模块 342/342；rust 模块 2 红为**基线既有**，stash 对拍实证与本改无关） | pass | — | ~1.5s | — |

**V-04R 关键证据（vue/back-HTTP 真实媒体，AC-05 收口）**：
- 031 打开媒体→受控回路就绪：`Ready`/`视图已接受` + `Images 1 corner-marked.png`（0.95s）
- 角点像素框内象限分布：`red:TL green:BL blue:TR`（2.11s）
- 旋转达像素：DOM bounding box 64×48→48×64 纵横互换 + 框内角点重排（3.17s）
- 平移达像素：img box x 228→268（+40px @zoom1，3.43s）
- 前后图导航：媒体 URI 切换 + 状态 Next（2-3s）
- 029：网格两照片（back photo_service 扫描 AUTO_PHOTO_ROOT）→ 查看区组件挂载
  （工具栏+胶囊）→ 变换截图 → 关闭回网格（合计 ~5s）

**本会话测试模块/耗时表（r3 会话合计，test-discipline）**：

| 模块 | 结果 | 准备 | 测试 | 清理 | 备注 |
|---|---|---|---|---|---|
| compiler --lib（vue 342 + 全量） | 342/342 绿；2 rust 红＝基线既有（stash 对拍） | 增量 | ~1.5s/轮 | - | 修复迭代期共 3 轮 |
| tools 两门 | 4/4 pass | 增量 | ~9s | - | 1 轮（曾被泄漏进程锁 exe，/T 修复后通过） |
| runtime contract+parity | 2/2 pass | 增量 | ~0.4s | - | 1 轮 |
| 产品门 native | 19/0/2 | 会话拉起×2（prep） | 46.8s（终版） | taskkill /T | r3 共 8 轮（3-7 轮计时受邻组 cargo 构建窗口与 flake 修复迭代影响，原始值见各 report.json） |
| 产品门 vue | 16/0/18 | prewarm×2+重开拉起（prep 15-110s/12-14s） | 176.5s（终版） | 同上 | r3 共 11 轮（含 F-04 诊断迭代） |
| 031 vue 探针（probe_031_vue.py） | 诊断用 | - | ~1-2min/轮 | 自动还原 | 定位 P11 的关键工具，留档 plan-artifacts |

新增 case 计时：终版两门全部 ≤10s（native 最大 7.34s；vue 最大 <10s）。
r3 迭代期的超时 case（10.13/16.49/23.16/21.72/25.04/12.25/11.53 等）均在
修复收敛前，原始值保留于对应轮次日志（plan-artifacts/plan-089/*run*.log）
与 §8.4 更正版对照。
