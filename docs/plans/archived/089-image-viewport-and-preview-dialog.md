---
plan_id: PLAN-089
status: archived
feature_name: ImageViewport 与 ImagePreviewDialog 通用图片查看
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-18
plan_revision: 3
current_step: 9
total_steps: 9
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/image-viewport.md]
touched_goals: []
---

# PLAN-089 通用图片视口与预览对话框

## 0. 变更摘要

将CL-09提前立项，抽取031 Image Viewer已有ImageViewport与工具栏为正式图片查看组件，并组合ImagePreviewDialog。上一轮Canvas/SceneViewport候选在本批只落普通图片查看部分，绘图画布/三维视图另留候选。路线见[组件库strategy](../../design/autoui/component-library-extension-strategy.md)。本轮仅规划，0/8。

r3（2026-11-18）：T-07 复审 needs_fix（F-01…F-05）后，用户裁定 F-03/AC-05 走"修订本计划新增 phase"路径：AC-05 收口轨改为 031/029 既有 back-HTTP 媒体的 vue 轨（V-04R，新任务 T-09）；native split 轨维持 P1 具名 not_run，split 注册表 feed 记具名后继依赖（另立）。T-05/T-06 复开承载 F-01/F-02/F-04/F-05 修复。0/9。

r4（2026-11-18）：T-07 再复审 needs_fix（F-06/F-07/F-08；F-01/F-02/F-05 关闭确认，AC-05 经 V-04R 收口并含评审独立探针）。F-04 仅部分关闭：窄窗腿未交付；AC-06 的 420×560 窄窗与加载/错误/禁用导航截图无证据；V-04R 两条 ok 恒真断言需收紧；decisions §8.5 三处秒数与 report 原值不符。复开 T-05/T-06/T-09。4/9，current_step 5。

r4 work（2026-11-18）：F-06/F-07/F-08/F-09 全部关闭——组件“更多”菜单（420×560 不裁切）+ V-05 窄窗/V-06 状态腿 + V-04R 效果断言；native 25/0/2（max 8.46s）、vue 17/0/18（max 4.18s）。T-01…T-06+T-09 完成（7/9），execution_done，next: T-07 再复审。

r4 复审（2026-11-18）：T-07 再复审 **pass**，status `reviewed`；F-06/F-07/F-08/F-09 关闭核验通过，AC-01…AC-06 全部 pass；8/9，current_step 8；next: T-08 merge（Spec 发布/归档/091 pin）。

merge（2026-11-18）：delivered @main **`5dbc3661`**（ff-only，17 commits；master 前置 c0035034），Spec/strategy 互链/归档/清理完成；用户裁定 merge 后不复测。9/9。

## 1. 目标

图片加载/错误、缩放、平移、适应窗口、原始比例、90°旋转、重置和可选前后张；可嵌入阅读器/笔记/文件管理器或作为modal预览。Windows/Linux native VM及Vue同义，MCP可用普通按钮/数值输入操作，不只能拖图片。

非目标：新增解码器、图像写回/裁剪/滤镜、矢量绘图、通用Canvas、三维相机/AGE场景、浏览器任意磁盘IO、Reader跨仓同步。显示旋转不修改原图。

## 2. 架构方案

调用方提供现有media ticket URI/资源身份 → 原ImageSurface管线 → 正式ImageViewport和工具栏 → 变换请求 → 当前实例视图状态。ImagePreviewDialog仅组合modal和视口。像素/解码/后端会话继续归既有媒体服务，前端只持元数据与视图变换。

新增`packages/image-viewer/`，收拢031局部widget，避免局部ImageViewport遮蔽正式包。保持原ImageSurface支持范围和回调负载；现有wrapper会丢失部分回调参数，T-01沿真实链核对并修正适配，不用无参数事件代替坐标数据。

## 3. 技术栈

现有`.at`、ImageSurface、media ticket、Rust image pipeline、VM/Iced及Vue、Dialog/主题/MCP；不新建一套图像渲染资源系统。数字缩放使用标准输入或087已交组件，087为可复用优化而非本Plan硬前置；首版按钮/预设缩放必须可独立完成全部操作。

## 4. 需求分析与背景调查

用户要求选通用程度高的一半先建实施计划，本合同对应图片查看。当前为建计划/更新路线，后续依Plan推进；无指定模型、预算、并行代理和跨仓发布。观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读Specs索引、示例归属/主题/结构边界、native验证合同。暂无正式通用图片视口Spec，现有源码/测试是本次观察依据；新Spec交付时沉淀。

| 已核来源 | 复用边界 |
|---|---|
| `examples/ui/031-image-viewer/src/front/image_viewport.at` | 已有消息通道wrapper，SHA256 `84D748ABDFAE2A7B5BFCA37592B7C8757B3AF5250E14011E6D4B7E914B552525` |
| 同例`app.at`、`SPEC.md` | 已有缩放/旋转/平移/前后图片和media会话消费，抽控件不搬业务服务 |
| `crates/auto-ui-compiler/schema/aura.at` imagesurface | fit/zoom/offset/rotation及媒体ticket接口，zoom描述范围0.05…64 |
| `crates/auto-ui-runtime/src/ui/{image_pipeline.rs,iced/image_surface.rs}` | 既有资源和变换/命中路径，应复用 |
| `crates/auto-ui-runtime/tests/{image_surface_contract,image_pipeline_parity}.rs` | 实际文件解析/VM/Vue/Rust生成与管线的既有回归入口 |
| `examples/ui/029-photo-gallery/src/front/app.at` | 当前图片网格/查看消费点，本计划先与091协调写入：089负责查看区，091后续只迁网格 |

## 5. 详细设计

- 正式ImageViewport沿既有src/alt/fit/zoom/offset/rotation通道，v1仅提供受控视图；业务view由调用方持有，组件只持局部手势，不同时维护另一份默认model。接口与原子更新规则见r2表。
- 缩放保留现有0.05…64边界；适应、原始比例、放大/缩小、方向平移和重置都有普通命名控件。坐标系和缩放锚点沿ImageSurface，不能让按钮与滚轮采用不同变换算法。
- 旋转按钮每次±90°，重置恢复当前图片初始视图；不重编码/写文件。已有低层其他rotation输入不受新工具栏限制。
- 新图片资源身份/revision改变时，调用方原子提供该图初始view，组件开始loading并失效旧回调；同资源仅重绘或主题变化不重置。损坏/失效URI是error，不伪装空图或无限loading。
- 前后张只发当前资源ID及方向请求，资源集合/顺序归应用；无上一/下一张禁用。标题/alt/尺寸等是可观察元数据，不用临时绝对路径冒充浏览器资源URI。
- Dialog沿既有modal焦点/背景隔离、Escape关闭、关闭恢复触发器；图片加载/错误不阻塞关闭。关闭/卸载须撤销本组件订阅或引用，不能释放其他视口仍在使用的共享资源。
- 同图双实例变换独立，旧action/延迟loaded/error不改变新图。图片预览与文档编辑状态分开；关闭不写原图片/数据库。
- 031消费正式包并保留现有media服务；029查看区使用通用Dialog/Viewport，原收藏/目录/鉴权仍归Gallery。包脱离主源根仍能解析，所有必要依赖列入快照。

### r2 执行接口与样例

按用户2026-11-14再次审核与样式参考要求细化；先读[执行与视觉指南](../../design/autoui/common-components-execution-and-visual-guide.md)的089区域/工具栏/窄窗规则。

| 接口 | r2合同 |
|---|---|
| ImageViewport props | `src/alt/resourceId/resourceRevision:str`, `revision:int=0`, `fit:str="contain"`, `zoom:float=1`, `offsetX/offsetY:float=0`, `rotation:int=0`, `showToolbar:bool=true`, `canPrevious/canNext:bool=false` |
| ImagePreviewDialog props | 同上，加`open:bool=false`, `title:str="图片预览"`；Dialog只组合视口，不另持业务view |
| 视图请求 | `viewRequested(str)`，一次消息携完整view，如`{"seq":3,"baseRevision":4,"resourceId":"lake","resourceRevision":"7","view":{"fit":"free","zoom":1.25,"offsetX":12.5,"offsetY":-4,"rotation":90},"source":"zoom"}` |
| 资源/导航事件 | `loaded(str)`, `error(str)`, `navigateRequested(str)`，均带当前resourceId/resourceRevision；navigate方向为previous/next；Dialog有`cancelled` |
| 映射 | 正式camelCase props在适配层映射原ImageSurface的offset_x/offset_y等；底层接口不改名；loaded不得捏造未回传的图片元数据 |

有效资源身份为`(resourceId或src,resourceRevision)`；view的revision单独用于外部受控更新。src/身份/view必须由消费handler一次性发布；新图默认`contain,1,0,0,0`，显式恢复视图则由调用方作为该图初始基线提供。组件不在render过程中发“自动重置”业务请求或反复调用setter；换图的旧loaded/viewRequested不能落到新图。

普通变换请求携全tuple，父应用按baseRevision接受后一次赋值并增revision，避免先zoom后offset的中间错误帧。fit取现有contain/width/one-to-one/free；rotation输出规范到0…359，按钮只±90。zoom界限0.05…64，非有限数/非法fit给`invalid_view`；显示操作不写图像。连续拖动可更新局部手势投影，但最终业务view只有调用方一份。

loaded/error只针对当前资源，错误含code/message；标题/尺寸等元数据来自消费方或真实加载回调。前后按钮只请求导航，调用方实际查询/换图。默认快捷键仅在该视口有焦点时生效，不能抢占其他输入框。

loaded最小体为`{"resourceId":"lake","resourceRevision":"7"}`；navigateRequested最小体为`{"seq":4,"baseRevision":5,"resourceId":"lake","resourceRevision":"7","direction":"next"}`。viewRequested.source取zoom/pan/fit/rotate/reset/gesture，不能由消费者根据按钮显示文字猜命令。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/image-viewport.md | 局部wrapper→正式图片视口/工具栏/Dialog、变换及资源revision合同 | 跨应用图片查看 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/image-viewport.md | 无正式消费门→MCP/modal/资源释放、两示例/双端视觉及生命周期 | 自动化且无副作用 | AC-04,AC-05,AC-06 |

拟树`D:/autostack/.wt/ui-089/auto-ui`，分支`codex/ui-089-image-viewer`，未创建。实现/独立最终review顺序同树，主检出维护计划；本轮不写canonical。

## 6. 测试设计

T-04新`examples/image-preview/`，实际消费媒体URI并读回当前资源/视图；T-05新`crates/auto-ui-tools/tests/image_viewer_component.rs`和`tests/product/verify_image_preview.py`。CLI采用`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`，T-01确定实际浏览器/媒体服务入口；全部为拟执行。

| Case ID | AC / 实际入口与输入 | 区分性断言 | work owner / 边界 |
|---|---|---|---|
| V-01 | AC-01,02 官方包/031：非对称带透明角图片，按钮/滚轮/平移/旋转/重置 | 已知角点像素和视图数值相符；旋转90°方向正确，缩放/坐标不是只更新label；原图哈希不变 | T-02/05；既有支持格式 |
| V-02 | AC-03 延迟A换B、损坏URI、重试，同图主题切换 | A回调不能覆盖B；错误可观察、能恢复；主题重绘不重置视图/发新业务选择 | T-02/03/05 |
| V-03 | AC-04 双实例/旧节点/modal背景、关闭其中一图/关闭重开 | 视图不串扰、旧action拒绝；共享图另一实例仍有效；背景不可操作、焦点恢复 | T-03/05；资源引用实际链 |
| V-04 | AC-05 031与029普通选择/预览/前后图→实际媒体结果 | 目标资源ID/图像像素正确；收藏/目录业务不回退；关闭无原件/数据库写入 | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，浅深/窄窗/错误/移根，退出重开 | 工具栏文字可读、图像裁切正确、动作可达；版本绑定native窗口及非空树，owned资源/端口回收并重开 | T-05/06；Q-01受影响项闭环 |

拟命令：`cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity`；`cargo test -p auto-ui-tools --test image_viewer_component`；`python tests/product/verify_image_preview.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue单列。

不能仅比较变换字段或截图存在：V-01实际像素角点与V-04资源身份构成跨层oracle。普通MCP/键盘操作，不注入view或accepted。NVM退出重开门、依赖摘要、失败对照、共享资源最后owner检查必需；无runner记缺口。独立最终复审在同树独立上下文，不创建代理。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/06，work，T-02/05）：单条viewRequested包含完整tuple；换图和view原子重置，迟到A事件不改B；在当前图的同值重绘/主题变化中零重置请求；按指南§3.2实际拍摄加载/错误/禁用导航及窄窗工具栏。
- T-01从已存在的`ui/iced/image_surface.rs::image_surface_geometry`、`normalize_image_surface_event`和原contract目标追链；原wrapper的无参数事件需与底层负载对照，不能误认坐标已自动传上来。
- fixtures优先使用031的`tests/fixtures/manifest.json`及小型RGBA/EXIF/损坏样本，T-04补独立的非对称角点图并记录生成法。既有031 `vm_matrix.py`含旧二进制默认路径，runner须显式pin实际工具，不拿旧默认路径作本次证据。

### r3 修订（2026-11-18，用户裁定 F-03 路径）：AC-05 收口轨与 gate 修复

- **AC-05 收口轨＝031/029 既有 back-HTTP 媒体服务的 vue 轨**。可行性已核实：031/029 pac.at 均已含 `dep image_viewer`（T-04 迁移，:36/:45）；front 经 `back.api current_uri` 取媒体 URI，浏览器 image_surface 经 vite `/api` 代理向 back 取字节——pre-plan 547 既有可用架构，不触 P1（前端注册表）与 P10（`__vmOnly`）。native split 轨维持 P1 具名 not_run、**不并入总绿**；split 前端注册表 feed/媒体代理＝具名后继依赖（另立计划，沿 §10 跨仓/平台边界，非本计划范围）。
- 新增 **V-04R**（AC-05 收口 oracle，`--backend vue`）：031 腿＝vite+back 会话，浏览器打开图片→组件 loaded→工具栏 ↻90/平移→截图像素 oracle（真实资源身份）→前后图；029 腿＝查看区打开→loaded→变换→关闭→网格/收藏/目录业务读回不回退。驱动必须走 Browser 通道——现 `run_v04` 在 vue 下误用 native Session 机制（r3 修复项，不得以 native 冒充 vue 轨）。
- **gate 修复随 T-05 复开**：显式 pre-gate 准备段（031/029 back 生成+编译移出计时门，腿内只等会话 ready——消除 608.62s 在测编译）；新增 case 计时合规（会话拉起移出 case 或拆分，≤10s 门，report 原值入档）；cleanup 退出码注明或修复。
- **对话框内 oracle（F-02）随 T-05 复开**：修 runtime MCP/modal-scope 对包组件子树的目标识别（P6 sweep 同族），**或**交付键盘驱动+`autoui_state` 读回的对话框内变换独立性断言——二者其一，不降低 AC-04 断言强度（双实例变换独立必须有通过证据）。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 正式包真实导入/运行，原ImageSurface与生成/管线回归、移根兼容；V-01/05 |
| AC-02 | work | 缩放/平移/旋转/fit/重置沿同一变换，实际像素与数值正确、不写原件；V-01 |
| AC-03 | work | 资源换代/加载/错误/重试/主题重绘语义正确；V-02 |
| AC-04 | work | 普通MCP/键盘、modal、双实例、过期事件、共享资源引用及关闭恢复正确；V-03/05 |
| AC-05 | work | **r3 修订**：031和029经既有back-HTTP媒体服务的**vue轨**真实目标/像素/前后图及原业务控制通过（V-04R）；native split轨＝P1平台边界具名not_run（feed具名后继依赖），不并入总绿 |
| AC-06 | work | Win/Linux native+Vue、浅深/窄窗、移根/退出重开有真实证据，受影响Q-01闭环；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立review复核AC-01…06/SD及像素/资源oracle，固定提交，无必需门缺失 |
| AC-08 | merge | Spec、合并/归档、strategy交付互链/实际清理准确；为091提供确切可消费pin |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan089/decisions.md`核对wrapper负载、变换/资源owner、031/029回归入口、Vue/runner/Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 最小图像/回调/共享释放探针有实际像素及源摘要，无第二资源系统 | 01…06 |
| [x] T-01 ✅ 已完成 (2026-11-17, worktree `D:/autostack/.wt/ui-089/auto-ui` @ `codex/ui-089-image-viewer`, commit `072b934e`) | decisions.md §1-6 全表落档；wrapper负载实链核对（builtin Wheel/Pan/DoubleClick 原生不派发→Task-25缺口，`ImageSurfaceCallback`+`ImageSurfaceInput`补齐，contract测试断言负载形状）；§3.3探针：`image_surface_contract`+`image_pipeline_parity` pass（0.35s），tools门`image_viewer_component` 2/2 pass（5.63s，双生成器+vendored闭包+具名缺依赖失败）；发现并落地`view`保留字→r2载荷键以带引号`"view"`输出（解析探针d1/d2证据）；依赖兄弟树auto-lang@0b6d3198d/auto-web@fd115ce0/auto-down@2724389b | 01…06 |
| [x] T-02 ✅ delivered @5dbc3661 (2026-11-18) | work / T-01 | 新`packages/image-viewer/`（`image_viewport.at`/`image_preview_dialog.at`/`pac.at`/`package.at`/README）；抽现有 viewport/变换投影，保留低层通道 | V-01/02 与既有 image 目标通过（复审+终版门） | 01…03 |
| [x] T-03 ✅ delivered @5dbc3661 (2026-11-18) | work / T-02 | 标准工具栏/ImagePreviewDialog、焦点/MCP/实例修订及资源取消；r4 窄窗“更多”菜单 | V-03 旧ID/背景/共享资源正反例通过（native 终版门） | 04 |
| [x] T-04 ✅ delivered @5dbc3661 (2026-11-18) | work / T-03 | 新`examples/image-preview/`；迁 031 局部 wrapper 和 029 查看区，移除 shadow | V-04 资源/像素/原业务读回（V-04R 031/029）；029 网格保留给 091 后续重基线 | 05 |
| [x] T-05 ✅ r4复开完成 (2026-11-18, commits `d120b30a`+`ab0df194`) | F-06：窄窗腿（native 窗口 420×560/client 407×524、更多菜单、重置/放大/右转/平移→生效、Dialog 关闭；vue 12 控件 rect 全在 420 内）+ V-06 加载/禁用导航（disabled:true 同帧）与错误态截图；F-01/F-02 证据沿用 r3（门重跑仍绿）；F-09 auto-lang probe 已清；r3 已落 pause_timer/拆案/prewarm/端口改绑/taskkill /T | 产品门 native 25/0/2（max 8.46s，65.4s）；vue 17/0/18（max 4.18s，126.7s）；新增 7 case 全 ≤10s | 01…06 |
| [x] T-06 ✅ r4复开完成 (2026-11-18, commit `ab0df194`) | F-08：decisions §8.5 三处秒数按 report 原值更正（1.66/2.79/3.36 → 2.11/3.17/3.43）；新增 §8.6 终版证据（两 report sha256、prep/cleanup、7 个新增 case 计时、V-04R 关键 oracle、测试模块表）；r3 已落 §8.3/§8.4 归因更正与 §8.5 | 版本/资源/工具身份与报告逐 case 原值可复核 | 06 |
| [x] T-09 ✅ r4复开完成 (2026-11-18, commit `ab0df194`) | F-07：`029 组件变换被接受` 改 img rect/transform 效果断言（64×48 `matrix(1)`→80×60 `matrix(1.25)`，稳定后截图）；`031 原业务面保留` 改目录索引断言（Images 面板 ≥2 `.png` + 打开/目录控件）；旋转/平移与 V-01 统一走“更多”；r3 已落 V-04R 双轨与 P11 修复 | V-04R 终版全绿（vue 17/0/18，max 4.18s）；AC-05 收口证据齐 | 05 |
| [x] T-07 ✅ T-07 再复审 pass (2026-11-18, reviewed `ab0df194`) | F-06/F-07/F-08/F-09 关闭核验：窄窗 420×560（native`窗口 420×560/client 407×524`+动作生效+Dialog 关闭；vue 12 控件 rect 全在视口内）、V-06 加载/禁用导航与错误态截图、029/031 效果断言、§8.5/§8.6 记录一致、auto-lang 树净；AC-01…AC-06 全部 pass | reviewed commit ab0df194；两门 25/0/2、17/0/18，新增 case 全 ≤10s（max 4.03s 新增；native 8.46s 为既有重开冷启动）；merge-owned 统一回归留 T-08 | 07 |
| [x] T-08 ✅ merge pass (2026-11-18, delivery `5dbc3661`) | ff-only 落主（master 前置 c0035034；reviewed ab0df194 经 rebase+range-diff 等价）；Spec 发布 `docs/specs/ui/image-viewport.md` @5dbc3661 sha a78467a7908f5282；strategy 089 归档互链 + 091 pin=`5dbc3661`；归档 `docs/plans/archived/089-…`；清理 worktree（先摘 1080 junction）/branch/plan-artifacts（318 files）/registry；用户裁定 merge 后不复测 | 最终文件/提交/资源关闭证据准确，给091交付版本 | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-089 r1；outcome: pass（计划可交接）；next: work，从T-01开始。新增T-01…08、AC-01…08、SD-01/02，0/8；无产品测试/新组件已完成声明。

2026-11-14修订交接：stage: new；PLAN-089 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-04/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

2026-11-17：stage: work | plan_id: PLAN-089 | plan_revision: r2 | outcome: pass（T-01关闭，T-02核心+T-04示例骨架随首个提交落地，T-03/T-04迁移/T-05产品门未完）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: ba92217b | code_commit: 072b934e | dependency_revisions: auto-lang@0b6d3198d, auto-web@fd115ce0, auto-down@2724389b（同组兄弟树，master HEAD detached）| task_ids: T-01（完成）,T-02/T-04（进行中）| evidence: decisions.md（§3.3已回填探针命令与结果：image_surface_contract+image_pipeline_parity pass、image_viewer_component 2/2 pass、"view"保留字引号键决议）；`view`保留字发现为本计划r2合同的机械适配（载荷键带引号输出，不改合同语义）| blockers: 无 | next_responsible_role: work（T-04的031/029迁移 + T-05产品门）
2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r2 | outcome: blocked（T-05 产品门 native 轨阻塞于 P6，其余工作推进；未到 execution_done）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: ba92217b | code_commit: eb8a35a8（含 072b934e/8d6167e6/5f292732 系列）| dependency_revisions: auto-lang@0b6d3198d, auto-web@fd115ce0, auto-down@2724389b, auto-man@cab347b（同组兄弟树）| task_ids: T-04（示例+031/029 迁移完成）,T-05（tools 门 2/2 pass；产品门脚本就绪；native 轨阻塞 P6）| evidence: decisions.md §7 P1-P7（split 票据边界/相对路径/App timer/view 保留字/tool 宿主二进制/未决 P6/占用记录）；demo18 曾全链实证 loaded=2 + viewRequested 接受 | blockers: P6——包组件 media notify 收集：sweep 未到达包组件 image_surface（无 VM-MEDIA、loaded 不派发），demo18 同二进制曾走通后回归，根因未定位 | next_responsible_role: work（以 probe12 最小复现定位 P6 后跑通 verify_image_preview.py native 全轨）

2026-11-18：stage: goal | plan_id: PLAN-089 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-089/auto-ui | commit: eb8a35a8 | next: work（P6 解除后续跑 native 产品门；tools/Vue 轨与 T-06/T-07/T-08 依 stage map 续行）

2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: needs_fix→resolved→pass（P6 已定位关闭，native 产品门 GREEN）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: ba92217b | code_commit: 至 6d8..（P6 关闭 + 证据提交，见 git log）| task_ids: T-05（tools 门 2/2 + native 产品门 GREEN 16/0/3 not_run + vue 轨 not_run P10）、T-06（spec/strategy/README/evidence 落档）| evidence: decisions.md §7 P1-P9 + §8 证据汇总；gate-native/report.json（16 pass 逐 case 计时，最大 8.9s ≤10s 门）；P6 三层根因关闭（文件污染 + cwd 相对路径 + vnode 竞态）；vue 轨 not_run = P10 __vmOnly 平台边界（具名后续依赖：split 前端注册表 feed）| blockers: 无阻塞性 blocker——3 项 not_run 均为具名平台边界（P8 modal scope / P1 票据隔离 / P10 __vmOnly），作为承认缺口交 review 裁定 | next_responsible_role: review（T-07 独立复审，裁定 not_run 缺口接受度）

2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r2 | outcome: pass（execution_done）| worktree: D:/autostack/.wt/ui-089/auto-ui | code_commit: 4f2ac101+（T-05/T-06 证据与文档提交）| task_ids: T-01…T-06 全完成（6/8）；T-07 review、T-08 merge 待 stage map 续行 | next: review

2026-11-18：stage: goal | plan_id: PLAN-089 | plan_revision: r2 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-089/auto-ui | commit: c8a95cd7 | next: review（T-07 独立复审：裁定的 not_run 缺口 P8/P1/P10 接受度 + 全 AC 复核）

2026-11-18：stage: review | review_scope: final（T-07）| reviewer_identity/limitations: 同会话族独立轮次（非独立模型实例）；结论全部从原始工件独立重建（report.json 原始逐 case 结果、session-3/4 日志、包/运行时源码、截图、gate 脚本），不采信执行摘要——已实证执行摘要与原始证据存在矛盾（见 F-01）| plan_id: PLAN-089 | plan_revision: r2 | outcome: **needs_fix**（AC-04/05/06 证据缺口 + 测试纪律违例；AC-01/02/03 pass）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | reviewed_commit: c8a95cd7（树净，无未提交实现）| base_commit: ba92217b | dependency_revisions: auto-lang@0b6d3198d, auto-web@fd115ce0, auto-down@2724389b | spec_inputs: docs/specs/ui/image-viewport.md（worktree 草拟 delta，SD-01/02 落点核对通过：`ui/iced/image_surface_input.rs` 实存、载荷 `"view"` 引号键+顶层平面别名键实存于包源 :267/:300/:313、媒体 sweep 适配描述与 P6 修复一致；最终冻结留 T-08）| evidence_identity: gate-native/report.json mtime 2026-11-18 10:11:02 在末次内容提交（10:18:51）前落盘＝证据覆盖 HEAD 内容；crates/ 自 21788bea 后零改动（docs/示例/gate 脚本除外，最终晨跑即对最终内容）；gate-native/report.json sha256 前缀 83c88d5241c51695，gate-vue 9681e71fa6bf0539，v01-corners-native.png d2b0e681f3d41d8c（与 report 内一致）

**AC 复核结论**：AC-01 **pass**（包真实导入/双生成器/vendored 移根闭包/既有回归——本轮复审在新树重跑：image_surface_contract 1/1 (0.38s)、image_pipeline_parity 1/1、image_viewer_component 2/2 (7.63s)）；AC-02 **pass**（V-01 矩阵 zoom1.25/宽度/↻90/平移+40 + 角点像素截图实证：旋转 90° 后红绿蓝角点位置正确、工具栏全具名控件、`loaded=2` 双实例、viewRequested tuple 读回含平面别名键；夹具 sha 不变）；AC-03 **pass**（V-02 损坏具名 error/重试/恢复 + V-06 换图原子重置/同图主题零重置）；AC-04 **partial**（内嵌实例 MCP 回路、modal 开关、cancelled 恰一次、背景隔离、共享资源 loaded=2 均 pass；但对话框内视口 MCP 不可达（F-02）且该 not_run 腿是"双实例变换独立"oracle 的唯一载体——AC-04 的双实例断言无通过证据）；AC-05 **未满足**（V-04 双腿 not_run：031 session-3.log 实证组件已挂载但 `status=Failed("media ticket missing")`=P1 真边界；029 冷编译 608.62s 超时=在测编译违纪（F-01）——031/029 真实媒体消费在任何后端均无证据）；AC-06 **partial**（Win native 浅深/窄窗/退出重开截图 pass；Linux 无 runner 具名 not_run **接受**；vue 整轨因 "vue demo never rendered; body: (empty)" (96.32s) 会话启动失败未跑（F-04））；AC-07 本轮=needs_fix 不发 pass；AC-08 merge 待。

**三项 not_run 缺口裁定**：
- **P8（modal scope）＝裁定不接受执行侧分类**。执行侧记"PLAN-084 收窄行为；接受回路共用已覆盖"。原始证据反驳：modal 开启时 MCP press 对话框头内 `关闭`（dialog-content 内，image_preview_dialog.at:36）**成功**（V-03 关闭腿 pass, cancelled=1）——modal scope 并不一揽子拒绝 dialog 内目标；被拒的 `放大` 位于**包组件子树**（ImageViewport 工具栏）内。故这是 modal-scope/MCP 目标解析对包组件子树的识别缺口（与 P6 媒体 sweep 同表示族，且该族修复已在本计划 runtime 内落地过），非 PLAN-084 平台行为。且此腿是 AC-04 "双实例变换独立" 的唯一正式 oracle，不能以"回路共用"豁免。→ **返工**（F-02）。
- **P1（split 票据隔离）＝裁定接受为平台边界，但 AC-05 关闭依据随之不满足，需合同层决策**。session-3.log 实证 front 注册表缺 back 进程票据（组件挂载正常、URI 正常、仅 ticket missing）；demo 侧改前端 `auto.image` 会话属合同内自由（demo 自持媒体），但 031/029 按 §5 合同"保留现有 media 服务"（back 持票）——native 轨在现运行时下不可达，修复需跨进程注册表 feed/媒体代理＝§10 预示的具名跨仓依赖，不在本计划擅写。031 pre-plan 的 `vm_matrix.py` 入口被 T-05 明示规避未跑，须 pin 二进制补跑或正式退役其证据地位。→ 具名依赖 + AC-05 收口路径决策（补合同修订或立项 feed 机制）（F-03）。
- **P10（`__vmOnly`）＝裁定接受，但仅限 demo 的 vue 媒体腿**。`auto.image.*` 为 VM 专属内建、浏览器具名抛错属实；demo vue 媒体腿依赖 split 前端注册表 feed（同 P1 具名依赖）。但 vue 轨其余可行腿（demo UI/明暗/窄窗——不依赖媒体；031/029 vue 轨经 back HTTP 媒体——pre-plan 既有可用架构，不受 P1/P10 影响）本次未交付：gate 死于会话启动（F-04），§8.2 手动首访渲染成功与 gate 空体矛盾未定位。→ 部分接受 + 返工（F-04）。
- Linux 无 runner 具名 not_run＝**接受**（本机平台可用性边界，与 §5 runners 表一致）。

**Findings**（均含最小重放域；计划级回归按纪律不入各 finding）：
- **F-01（测试纪律，needs_fix）**：新增 gate case 超 10s 门——"V-01 open A→loaded" **10.13s**、"V-05 退出重开" **16.49s**（report.json 原值）；not_run 尝试本身耗 54.99s/608.62s，其中 029 腿在计时测试内等待 back 首次生成+编译（600s deadline）＝"日测不自建前置"违纪，须改显式 pre-gate 构建阶段（031/029 back 先行物化，腿内只等会话 ready）。**且 decisions.md §8.4 实证失真**：记"最大单 case 8.9s、全部 ≤10s 门内"，与其自引 report.json（16.49s/10.13s）矛盾——须按原始值更正；cleanup exit_code 1 未解释须注明或修复。最小重放：前置构建后重计时首 case/V-05（会话拉起移入准备段）；如仍超限拆分 case。
- **F-02（P8 误分类+AC-04 双实例 oracle 缺失，needs_fix）**：违反而既有规则＝AC-04"普通MCP/键盘、modal…正确"与 V-03 双实例断言。已确立事实：`关闭`（包内 dialog 头）MCP 可达、`放大`（包组件子树）被拒"target is outside the active dialog"。调查方向（非确诊处方）：runtime MCP/modal-scope 目标解析对包组件实例子树的可见性（P6 sweep 同族，devtools/解析面）。修复或等价证据二选一：修识别缺口（P6 式）；或交付键盘驱动的对话框内变换 + `autoui_state` 读回的独立性断言。最小重放：开 modal→press 放大→断言可达/或键盘路径→断言 dialog 实例请求与内嵌实例互不串扰。
- **F-03（AC-05/P1，决策依赖）**：AC-05 关闭依据（V-04 通过）当前任何后端均未满足；修复路径＝具名跨仓依赖（split 前端注册表 feed/媒体代理）立项或 AC-05 合同修订（如改以 back-HTTP vue 轨为 031/029 收口轨）；029 重试前先落 F-01 前置构建。归 work 推进 + 上浮用户决策。
- **F-04（vue 轨可行腿未交付，needs_fix）**："vue demo never rendered; body: (empty)"（96.32s，含 90s 轮询）未定位，而 §8.2 声称手动首访渲染成功——矛盾未关闭。须修 vue 会话拉起（vite 冷启/物化诊断）后跑：demo UI/明暗/窄窗腿（无媒体依赖）+ 031/029 vue V-04 腿（back HTTP 媒体）。demo 媒体腿维持 P10 not_run。
- **F-05（记录失真，needs_fix，随 F-01）**：decisions.md §8.4 计时表/结论与 report.json 原始值不符（8.9s vs 16.49s；"全部≤10s"不成立）；§8.3 将 031 腿归因 P1 时序差"未定位"与 session-3 实证（ticket missing 常驻、非时序）不一致——按原始证据更正归因。

**pending_closeout_gates**：merge-owned 统一回归/收口未跑（正常留 T-08）；F-01/F-02/F-04 修复后受影响门重跑（V-03 对话框腿、V-04 腿、vue 轨、受影响 case 重计时）。

**本会话复审测试表**（test-discipline）：

| 模块 | 结果 | 准备 | 测试 | 总计 |
|---|---|---|---|---|
| auto-ui-runtime image_surface_contract + image_pipeline_parity（复审重跑，冻结树 c8a95cd7） | 2/2 pass | 增量检查 ~50s | 0.38s | ~50s |
| auto-ui-tools image_viewer_component（复审重跑） | 2/2 pass | 增量（已热） | 7.63s | ~9s |
| native 产品门 19 case | **复用执行侧 10:11 跑**（内容同一性已核，未重跑，709.4s/16 pass/0 fail/3 not_run） | — | — | 复用 |
| vue 产品门 | 复用执行侧 09:46 跑（not_run） | — | — | 复用 |

新增 case 计时复核：report.json 原值最大 **16.49s（V-05）**、10.13s（V-01 open）超 10s 门→F-01；工具门最大 case 7.63s 门内。机器占用：开跑前查无 cargo/rustc/auto 测试编译进程（仅 4 个 node 常驻）。

next: **work**——T-05/T-06 复开（受影响证据随 F-01/F-02/F-04/F-05 关闭重跑）；F-03 的 AC-05 收口路径（跨仓 feed 立项 vs 合同修订）上浮用户决策后随行。当前完成计数 T-01…T-04（4/8），current_step 回 5。

2026-11-18：stage: revise（r3，用户裁定）| plan_id: PLAN-089 | plan_revision: r2→r3 | outcome: pass（修订合同可执行）| 决策内容：F-03/AC-05 采"**修订本计划新增 phase**"路径（用户 2026-11-18 指定），否决以跨仓 feed 立项为本计划收口前提——AC-05 收口轨改为 031/029 既有 back-HTTP 媒体的 **vue 轨**（V-04R，新任务 T-09）；native split 轨维持 P1 具名 not_run 不入总绿，split 前端注册表 feed/媒体代理记**具名后继依赖**（另立计划交接物）| 变更明细：frontmatter total_steps 8→9；§0 r3 摘要；§6 新增"r3 修订"段（V-04R 合同：Browser 驱动禁止 native Session 冒充、pre-gate 准备段、F-02 双路径或其一）；§7 AC-05 关闭依据修订；§8 T-05/T-06 复开注记（F-01/F-02/F-04 → T-05；F-05 → T-06）+ 新增 T-09 行（位于 T-07 前）| 可行性依据（本轮核实）：031 pac.at:36、029 pac.at:45 均已含 `dep image_viewer`（T-04 迁移物）；031 front 经 `use back.api: … current_uri` 取 URI（app.at:5/:206）——vue 轨不触 P1/P10 | 执行顺序：T-05（复开修复）→ T-06（更正）→ T-09（V-04R）→ T-07（再复审）→ T-08（merge）| 完成计数 T-01…T-04（4/9）| next: work

2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r3 | outcome: **pass（execution_done，T-05/T-06/T-09 复开/新增任务全关闭）** | worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: c8a95cd7（r3 修订时 HEAD）| code_commit: **2432c25f**（95787e0e compiler P11 修复 + 47e9cdbe ignore + 2432c25f gate/证据；树净）| dependency_revisions: auto-lang@0b6d3198d, auto-web@fd115ce0, auto-down@2724389b, auto-man@cab347b（本地重建 auto.exe，未改 auto-man 源）| task_ids: T-05（复开完成：F-01 计时合规/F-02 对话框 oracle/F-04 vue 轨）, T-06（复开完成：F-05 记录更正）, T-09（完成：V-04R 全绿 + P1 具名交接）| evidence: decisions.md §8.5 终版表；gate-native-r3/report.json（19 pass/0 fail/2 not_run=P1 具名，sha256 前缀 765de8e52cfda9d2，max case 7.34s，46.8s）；gate-vue-r3/report.json（16/0/18=P10 demo 媒体腿，sha256 前缀 d0cc5cd9b3f17b1b，全 ≤10s，176.5s）；V-04R 关键 oracle：031 角点框内象限（red TL/green BL/blue TR）、旋转 bounding box 64×48→48×64 互换、平移 x+40、媒体 URI 导航切换、029 网格→查看器→关闭全绿；tools 4/4；runtime contract+parity 2/2；compiler vue 342/342（rust 模块 2 红＝基线既有，stash 对拍实证与本轮无关）| **新发现 P11（本轮修复）**：vue codegen lucide 图标与 .at handler 同名冲突——图标元素解析到 handler 函数按 render 调用＝请求风暴（031 实测 203 条 source=zoom、视图 0.05 下限、4×3 像点）；修复＝Lucide 前缀别名（`95787e0e`）+回归测试+金样同步 | blockers: 无 | next_responsible_role: review（T-07 再复审：F-01…F-05 关闭核验 + P11 修复核验 + AC-04/05/06 证据重裁定）| 完成计数 T-01…T-06+T-09（7/9），current_step 7

2026-11-18：stage: review | review_scope: final（T-07 再复审，r3 修复后）| reviewer_identity/limitations: 同会话族独立轮次（非独立模型实例）；结论从原始工件重建——gate-native-r3/gate-vue-r3 report.json 逐 case 原值、browser-*.log、截图、包/生成物源码、编译器与 tools 测试；执行侧摘要仅当索引（r2 已实证摘要有失真，本轮继续以原值核对）| plan_id: PLAN-089 | plan_revision: r3 | outcome: **needs_fix**（AC-06 证据缺口（F-06）+ F-04 未完全关闭 + V-04R 两条恒真断言（F-07）+ 记录失真复发（F-08）；AC-01…AC-05 pass）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | reviewed_commit: `2432c25f`（树净；实现基线 ba92217b，r3 修复区间 c8a95cd7..2432c25f）| base_commit: ba92217b | dependency_revisions: auto-lang@0b6d3198d（工作树有 1 处未提交 `#[cfg(test)]` throwaway probe，见 F-09）、auto-web@fd115ce、auto-down@2724389、auto-man@cab347b | spec_inputs: `docs/specs/ui/image-viewport.md` @2432c25f sha256 前缀 48eeba6c82185721（r3 未改该 delta；SD-01/02 落点与当前实现一致，最终冻结留 T-08）| evidence_identity: native report sha256 765de8e52cfda9d2（19 pass/0 fail/2 not_run，max 7.34s，sum 32.9s，总 46.8s）；vue report sha256 d0cc5cd9b3f17b1b（16/0/18，max 3.43s，sum 23.2s，总 176.5s，prep 031 21.9s/029 15.5s/reopen 19.11s）；两门 prep 之外无 >10s case；cleanup exit_code=1 已注记（terminate-based）+owned_reaped=true

**证据同一性**：两 report mtime 13:35:48/13:38:58 早于 final commit 13:39:51（运行先冻结内容、提交随后打包）；report 的 case 构成/字段（prep、V-05 拆分、cleanup note）与提交脚本一致；031 gen 产物（13:36 生成）已含 `LucideZoomIn` 别名，证明运行用 auto-ui-tool（sha ff172df6）已含 P11 修复；reviewer 另以提交源重跑编译器/tools 两组。未提交的 auto-lang probe 为 `#[cfg(test)]`，不进 gate 产物。

**关闭核验**：
- F-01 **closed**：终版两门逐 case 原值全 ≤10s（native max 7.34s=V-05 重开；vue max 3.43s）；`pause_timer` 仅移出会话拉起且 prep 单独入表；V-05 已拆两案；`taskkill /T` 整树回收；cleanup 码注记在 report。未发现以 pause_timer 隐藏 case 内工作的用法。
- F-02 **closed**：native V-03 负控=`Error: Modal dialog owns input; target is outside the active dialog or no longer exists`（首匹配=内嵌实例被具名拒绝）；正腿=`当前视图 … zoom=1.25 … rev=10 请求数=5`（`occurrence='last'` 对话框实例请求被接受）；挂载前 ticks 掩码双帧稳定门。r3 区间 runtime/包源零改动，与"根因=同名双实例标签歧义、modal scope 无缺陷"的更正一致；原 P8"包子树识别缺口"诊断据此证伪并关闭。
- F-04 **partial（未关闭）**：vue 会话 4 轮全部渲染（browser-1..4.log 均 VITE ready；report 16 pass），demo UI/明暗腿与 031/029 V-04R 腿已交付；但 F-04 明列的"窄窗腿"未交付（详见 F-06）。
- F-05 **closed（更正本身）**：§8.3 按 session-3 实证（ticket missing 常驻、非时序差）重写；§8.4 按 r2 report 原值 16.49/10.13 重写；本轮对照 gate-native/report.json 核验一致。**但 §8.5 出现同类记录失真 → F-08**。
- P11 **verified**：编译器 `cargo test -p auto-ui-compiler --lib --features ui` = 1171 pass / 2 fail（`ui_gen::rust` 两例；`ui_gen/rust.rs` 在 c8a95cd7..2432c25f 零改动＝基线既有）/ 2 ignored；`vue` 过滤 343/343；P11 回归 `plan089_lucide_icon_handler_name_collision_aliased` 独立重跑 pass（0.21s）；031 生成物实存 `import { ZoomIn as LucideZoomIn, … }` 与 `<LucideZoomIn>`；V-04R 031 旋转/平移达像素，旧 203 条 source=zoom 风暴无复现。tools `image_viewer_component` 在 P11 后复跑 2/2（7.64s）。

**AC 复核结论**：AC-01 **pass**（复用 r2 双生成器/vendored 证据 + 本轮 tools 2/2 复跑 + 编译器 vue 343/343；rust 2 红为基线既有、与本改无关；P11 别名在真实生成物核验）| AC-02 **pass**（native V-01 r3 原值：open 3.55s/zoom 1.25/宽度/rot90/角点/平移 x=40/夹具哈希不变；包/运行时自 c8a95cd7 未变）| AC-03 **pass**（native V-02 损坏具名 error/重试计数器/恢复 + V-06 换图原子重置/同图零重置；实现未变）| AC-04 **pass**（F-02 正负腿；共享资源 loaded=2 沿用 r2 已复核；vue V-03 腿=P10 demo 媒体边界 not_run，双实例 modal oracle 已由 native 面载明）| AC-05 **pass**（V-04R：031 角点 `red:TL green:BL blue:TR`、旋转 rect `{220,100,64,48}`→`{228,92,48,64}`+象限重排 `red TR/green TL/blue BR`、平移 x 228→268、媒体 URI `e0e21703…`→`cb6c8687…`；029 网格→查看区→关闭；**029 受控变换由本轮独立探针复核**：img rect 64×48 `matrix(1,0,0,1,0,0)` → 80×60 `matrix(1.25,0,0,1.25,0,0)` → ↻ 后 60×80 `matrix(0,1.25,-1.25,0,0,0)`；native split＝P1 具名 not_run，按 r3 合同不入总绿）| AC-06 **partial**（Win native+vue 明暗/退出重开 pass、Linux 无 runner 具名 not_run 接受、移根由 tools vendored 用例覆盖；**420×560 窄窗无用例/截图，加载/错误/禁用导航截图缺失**）| AC-07 本轮 needs_fix | AC-08 merge 待

**Findings**（最小重放域；计划级回归不并入）：
- **F-06（AC-06/V-05/V-06 证据缺口，needs_fix）**：违反既有规则＝AC-06"浅深/**窄窗**、移根/退出重开有真实证据"、V-05 行"浅深/窄窗/错误/移根"、V-06"按指南§3.2实际拍摄加载/错误/禁用导航及窄窗工具栏"、指南 §3.5"两套主题模式+至少一个 **420×560** 窄窗"。实测：两门 report 的 visual 仅 dark/light（native 2560×1600、vue 1100×800），cases 列表无窄窗/错误/禁用导航项，plan-artifacts 全量检索无 420/560/窄窗工件。**同时更正 r2 复审"Win native 窄窗截图 pass"的记述不成立**（无任何工件支撑）。最小重放：vue `page.set_viewport_size({width:420,height:560})`、native 窗口 resize 至 420×560，各截图并断言工具栏文字可读、常用动作可达（如 press 放大/平移）、不溢出遮挡；补 damaged 错误态与 `canPrevious/canNext=false` 禁用态截图；单案 ≤10s。
- **F-07（V-04R 恒真断言，needs_fix）**：`V-04R 029 组件变换被接受`（gate :1169-1171）与 `V-04R 031 原业务面保留`（:1109-1111）`ok` 为常量 True、observed 空——AC-05 收口腿存在假绿风险（产品行为本轮已由评审探针证实，但 gate 不能自证）。且 `v04r-029-viewer.png` 捕到 zoom 过渡帧（角点几何≈1.125 缩放），非稳定态。最小重放：029 断言 img rect/transform 前后（64×48→80×60，↻→60×80）后再截图；031 实际打开 Stats/目录并断言内容，或删去未验证的"统计"表述。
- **F-08（记录失真复发，needs_fix，随 T-06）**：decisions.md §8.5 引用 031 角点/旋转/平移 per-case 秒数 1.66/2.79/3.36，与其自引 gate-vue-r3/report.json（sha d0cc5cd9…，原值 2.11/3.17/3.43）不符（疑取自第 9 轮）；须按 report 原值更正。其余引用（native 7.34/46.8、vue 176.5、031 打开 0.95s、029 合计 ~5s）核对一致。
- **F-09（依赖树清理，低，随 T-05 收口）**：auto-lang@0b6d3198d 工作树有 1 处未提交 `crates/auto-media-core/src/image_pipeline.rs` 的 `#[cfg(test)] plan089_probe_tests`（自注"throwaway probe (removed before handoff)"）；不进构建产物、不影响 reviewed_commit，但不得带入 handoff——移除后再收口。

**pending_closeout_gates**：F-06/F-07/F-08（+F-09 清理）修复后重跑受影响 native/vue 门并更新 §8；T-07 再复审；merge-owned 统一回归留 T-08。**这是第二次 final review needs_fix**——下一轮 handoff 前须先核对 F-06 的覆盖设计（窄窗/错误/禁用导航腿如何进入 V-05/V-06 而非"补一张图"）与 F-07 的断言形态，再交复审。

**本会话复审测试表**（test-discipline；未重跑产品门——执行侧报告按 sha/逐 case 原值复核 + 独立探针补证；机器同期有另一组 ace release 构建，未做计时门）：

| 模块/检查 | 结果 | 准备 | 测试 | 总计 |
|---|---|---|---|---|
| compiler --lib --features ui（全量） | 1171 pass / 2 fail（ui_gen::rust 两例，基线既有）/ 2 ignored | 增量重建 32.35s | 1.90s | ~34s |
| compiler vue 过滤（含 P11 回归） | 343/343 pass | 热 | 1.37s | 1.4s |
| P11 回归单测（plan089_lucide_…） | 1/1 pass | 热 | 0.21s | 0.2s |
| auto-ui-tools image_viewer_component（P11 后复跑） | 2/2 pass | 热 | 7.64s | 7.6s |
| 029 变换独立探针（Browser/`auto run`，AUTO_PHOTO_ROOT 隔离） | 变换实际达 img（rect/transform 见 AC-05） | 产物已物化 | 未计时（探针，含会话启停） | 会话外 |
| native 产品门 19+2 / vue 产品门 34 | **复用**执行侧 13:35/13:38 报告（sha 765de8e5…/d0cc5cd9…；逐 case 原值已核，未重跑） | — | — | 复用 |

新增 case 计时复核：终版两门无 >10s case；本轮未新增产品门计时 case（029 探针不计入产品门）。证据留档：`plan-artifacts/plan-089/probe-029-transform.py` 与 `probe-029-transform.result.json`（sha256 前缀 a02c2548f28bceb3）。

next: **work**——复开 T-05（F-06 窄窗/错误/禁用导航腿，F-09 清理）、T-06（F-08 秒数更正）、T-09（F-07 断言收紧）；完成计数 T-01…T-04（4/9），current_step 5；随后 T-07 再复审。

2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r3 | outcome: **progress（修复已提交 `d120b30a`；最终门待机器空闲）** | worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: 2432c25f | code_commit: `d120b30a` | task_ids: T-05（r4：窄窗/状态腿已落，门待跑）, T-06（r4：§8.5 三处秒数已更正，§8.6 待门后补）, T-09（r4：V-04R 断言路径已收紧为“更多→…”效果断言，门待跑）| 已落：组件工具栏一行保留 前后图/缩放/适应/宽度/1:1 + 具名“更多”，旋转（左转90°/右转90°）/重置/平移入“更多”菜单；`run_narrow`（native resize 420×560、vue viewport）与 V-06 加载/错误/禁用导航腿；031 vm_matrix 同步 | 独立探针（非最终门）：vue 12 控件 rect 全在 420 视口内（`plan-artifacts/plan-089/probe-narrow.py`）；native 窗口 420×560/client 407×524、更多菜单可达、窄窗动作 zoom=1.25/rot=90/x=40、Dialog 关闭（`probe-run-narrow.py`）；tools `image_viewer_component` 2/2（8.77s） | 观察：native 全门冒烟在并发构建下 V-03 对话框腿一次 5s 等待超时（同逻辑独立探针 `probe-dialog.py` 立即 pass），按纪律未把受扰计时当证据 | blockers: **机器占用**——另一组 ace 测试/构建（lang-980/981，cargo/rustc 持续 10min+）按 test-discipline 未启动最终 native/vue 门 | next_responsible_role: work（等窗后跑两门 + §8.6 + 完成计数 7/9）| 完成计数 T-01…T-04（4/9），current_step 5

2026-11-18：stage: work | plan_id: PLAN-089 | plan_revision: r3 | outcome: **pass（execution_done，F-06/F-07/F-08/F-09 关闭）** | worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | base_commit: 2432c25f | code_commit: **`ab0df194`**（组件/门 `d120b30a` + 证据/断言 `ab0df194`；树净）| dependency_revisions: auto-lang@0b6d3198d（probe 已清）、auto-web@fd115ce、auto-down@2724389、auto-man@cab347b | task_ids: T-05（r4 完成：窄窗与 V-06 状态腿）、T-06（r4 完成：§8.5 更正 + §8.6）、T-09（r4 完成：V-04R 断言收紧）| evidence: gate-native-r4e（25 pass/0 fail/2 not_run=P1，max 8.46s，65.4s，sha256 d4101aed04db3e2d）；gate-vue-r4c（17/0/18=P10 demo 媒体，max 4.18s，126.7s，sha256 83d7581752891827）；新增 7 case 全 ≤10s（0.31–4.03s）；V-05 窄窗 native 窗口 420×560/client 407×524 + 更多菜单 + 动作生效（zoom1.25/rot90/x40）+ Dialog 关闭；vue 12 控件 rect 全在 420 内；V-06 加载/禁用导航 `disabled:true` 与错误态截图；V-04R 029 rect 64×48→80×60 matrix1.25、031 目录 ≥2 `.png`；demo bool 归一（can_prev/last_is_b）| blockers: 无（13:39–15:39 邻组 ace 构建占用，按 test-discipline 等窗后跑终版门；开跑前 active=0）| next_responsible_role: review（T-07 再复审 F-06/F-07/F-08 关闭 + AC-06 重裁定）| 完成计数 T-01…T-06+T-09（7/9），current_step 7

2026-11-18：stage: review | review_scope: final（T-07 再复审，r4 修复后）| reviewer_identity/limitations: 同会话族独立轮次（非独立模型实例）；结论从原始工件重建——gate-native-r4e/gate-vue-r4c report.json 逐 case 原值、截图、提交的 gate/组件源码、spec delta；执行摘要仅作索引 | plan_id: PLAN-089 | plan_revision: r3 | outcome: **pass**（F-06/F-07/F-08/F-09 关闭；AC-01…AC-06 pass）| worktree_path: D:/autostack/.wt/ui-089/auto-ui | branch: codex/ui-089-image-viewer | reviewed_commit: **`ab0df194`**（树净；r4 修复区间 2432c25f..ab0df194，实现基线 ba92217b）| base_commit: ba92217b | dependency_revisions: auto-lang@0b6d3198d（树净）、auto-web@fd115ce、auto-down@2724389、auto-man@cab347b | spec_inputs: `docs/specs/ui/image-viewport.md` @ab0df194 sha256 前缀 **a78467a7908f5282**（新增窄窗工具栏规则=行1 8 + 更多菜单 4，与实现一致；SD-01/02 落点复核通过，最终冻结留 T-08）| evidence_identity: native report sha256 **d4101aed04db3e2d**（mtime 15:56:35，25 pass/0 fail/2 not_run，max 8.46s，65.4s）；vue report sha256 **83d7581752891827**（mtime 15:55:15，17/0/18，max 4.18s，126.7s）；两 report 均早于 final commit 15:57:33（内容冻结后打包），提交的 gate/组件/demo 与运行内容一致；auto sha 8507fc81…、tool sha 7682f13c…（r3 后由 tools test 重建，同源；report 绑定实际 sha）；fixtures af62192b/2391b725/2d3d3e31 未变；cleanup exit 1 注记 + owned_reaped=true；reopen ready

**F 关闭核验**：
- **F-06 closed**：组件“更多”菜单（行1 8 + 菜单 4 = 12 命名控件）；native 窄窗（window 420×560 / client 407×524）动作真实生效（zoom=1.25 rot=90 x=40）、Dialog 关闭 cancelled+1、工具栏可达截图；vue 窄窗 12 控件 rect 全在 420 视口内（更多 right=370）、禁用态可见、组件区截图；V-06 加载/禁用导航（A：上一张 `disabled:true`/下一张 enabled；B 反向）与 damaged 错误态（`load_failed` + 截图 2560×1600）。窄窗尺寸=窗口外框 420×560（客户区 407×524 为标题栏/边框差），满足指南 §3.5 的 420×560 窄窗；加载证据=加载完成态（组件无独立 loading 视觉；空态“没有图片”/错误态 V-02 分别承担，未伪造占位）。
- **F-07 closed**：029 `组件变换被接受` 现断言 img rect 64×48 `matrix(1)` → 80×60 `matrix(1.25)`（稳定后截图）；031 `原业务面保留` 改为目录索引断言（Images 面板 ≥2 `.png` + 打开/目录控件；实测 `rgb-1x1.png`/`scenic-320x240.png` 等），脚本扫描无 `, True)` 恒真行。
- **F-08 closed**：decisions §8.5 三处秒数 2.11/3.17/3.43 与 r3 report 一致；§8.6 与 r4 report 逐项一致（新增 case 0.79/0.90/1.95/0.31/4.03/2.23/0.95；max 8.46/4.18；report sha 前缀）。
- **F-09 closed**：auto-lang 树净（未提交 probe 已移除）。
- 非 finding 记录：demo bool 归一（`can_prev`/`last_is_b` int→bool）修掉 V-06 反向禁用断言暴露的 app 缺陷；组件公开合同未变。

**AC 复核结论**：AC-01 **pass**（tools 2/2 在组件菜单后复跑；compiler vue 343/343、全量 1171/2 基线红/2 ignored 复用；P11 别名在生成物）；AC-02 **pass**（V-01 全矩阵新菜单路径 + 角点像素 + 夹具哈希不变）；AC-03 **pass**（V-02 损坏 error/重试/恢复、V-06 换图原子重置/主题零重置）；AC-04 **pass**（V-03 正负腿/关闭恢复、双实例模态 oracle、nav disabled 正反）；AC-05 **pass**（V-04R 031 角点象限/旋转 bbox 互换/平移 +40/URI 导航/目录索引；029 变换 rect/transform；native split P1 具名 not_run）；AC-06 **pass**（Win native+vue 明暗、420×560 窄窗、加载/错误/禁用导航、退出重开；Linux 无 runner 具名 not_run 接受；移根 tools vendored）；AC-07 本轮=pass；AC-08 merge 待

**pending_closeout_gates**：merge-owned 统一回归、Spec 发布、归档与 strategy/091 pin 交付（T-08）；plan-artifacts 收口删除。

**本会话复审测试表**（本轮未新增运行——复用 r4 门，报告 sha/逐 case 原值已核；tools/compiler 证据沿用）：

| 模块/范围 | 结果 | 准备 | 测试 | 总计 |
|---|---|---|---|---|
| native 产品门 27 case（gate-native-r4e） | 25 pass / 0 fail / 2 not_run(P1)，max 8.46s | 会话拉起×2 | 65.4s | 复用 |
| vue 产品门 35 case（gate-vue-r4c） | 17 pass / 0 fail / 18 not_run(P10)，max 4.18s | prewarm 031 17.4 / 029 13.2 / reopen 20.04 | 126.7s | 复用 |
| tools `image_viewer_component` | 2/2 pass | 热 | 8.77s | 组件菜单后，复用 |
| compiler（vue 343/343；全量 1171/2 基线/2 ignored） | pass | — | — | 无 compiler 改动，复用 |

新增 r4 case 计时：0.79（加载/禁用导航）、0.90（错误态）、1.95（窄窗 resize）、0.31（窄窗可达）、4.03（窄窗动作）、2.23（窄窗 Dialog）、0.95（vue 窄窗）——全部 ≤10s；native 最大 8.46s 为既有 V-05 重开冷启动（非新增）。

next: **T-08 merge**——发布 `docs/specs/ui/image-viewport.md`、合并/归档、strategy/091 pin 与 plan-artifacts 清理；merge 期统一回归为 merge-owned。

2026-11-18：stage: merge | plan_id: PLAN-089 | plan_revision: r3 | outcome: **pass（delivered）** | delivery_commit: **`5dbc3661`**（main，ff-only，17 commits；master 前置 c0035034）| reviewed_commit: `ab0df194`（T-07 pass）| canonical: `docs/specs/ui/image-viewport.md` @5dbc3661 sha256 前缀 **a78467a7908f5282**（SD-01/02 + 窄窗规则；reviewed→landed 文件逐字节相同）；`docs/design/autoui/component-library-extension-strategy.md` 089 归档互链 + 091 pin=`5dbc3661`；`file-navigation-and-interaction-visual-guide.md`/component review README 的 089 链接改指归档路径（两者为既存未跟踪设计/复核文档，仅本地改指；tracked 交付面为 strategy） | archive: `docs/plans/archived/089-image-viewport-and-preview-dialog.md`（随本提交入库）| cleanup: worktree `D:/autostack/.wt/ui-089/auto-ui` 已移除（wt-guard clean；先摘 1080 个 pnpm junction/symlink，无穿透），branch `codex/ui-089-image-viewer` 已删，`plan-artifacts/plan-089`（318 files）已删，registry 已刷新（`auto-ui/PLAN-089`，deleted-at-merge/no holds）| **checkpoints**：prepared ✓（rebase c0035034→5dbc3661；`git range-diff` 16/17 提交逐条相等，唯一差异=commit1 的 Cargo.lock hunk 按 master 更新锁丢弃；package/gate/spec/demo 与 reviewed ab0df194 逐字节相同）｜landed ✓（`git merge --ff-only`，无 merge commit；main tip=5dbc3661）｜canonical_verified ✓（spec blob 840a77a7…、sha256 a78467a7…；strategy/091 互链）｜archived ✓（archive 路径与链接已核）｜cleaned ✓（见上）| tests: **用户裁定不复测**（"这一波merge后不需要再次重新测试了。直接合并即可"）——复审证据 @ab0df194 沿用：native 25/0/2（max 8.46s，65.4s）、vue 17/0/18（max 4.18s，126.7s）、r4 新增 case 0.31–4.03s；本轮无新增 test/build | batch regression: `docs/plans/.last-batch-regression.json` 缺失=按政策 due；按用户不复测裁定未启动，留作 user handover | environment: main 检出存在与本计划无关的既存 dirty WIP（`Cargo.lock`、docs design 等），未触碰；本计划交付文件全部入库 | completion_kind: **delivered**

## 10. 待澄清事项

r2已选定v1仅受控视图及完整tuple消息，T-01验证wrapper负载、原子发布、共享资源撤销、fixtures和runner；没有阻碍立项的用户决策。不得缩成纯图片img或以新增解码器/3D引擎替代既有资源通道。发现需跨仓媒体服务变更则具名依赖，不在本计划擅自写入。

2026-11-18（work 阻塞待续）：T-05 native 产品门阻塞于 **P6**——包组件 media notify 收集：sweep 未到达包组件 image_surface（无 VM-MEDIA 行、loaded 不派发），demo18 同二进制曾全链走通（loaded=2 + viewRequested 接受）后回归，根因未定位。已落两修复（实例名归一、零参回退派发，devtools.rs）与最小复现 probe12（timer-probe + image-viewer 包 + loaded 中继）。**解除动作**：以 probe12 为最小复现定位 sweep 对包组件子树的收集缺口（方向：包组件子树在动态视图中的表示与 `collect_media_surfaces` 遍历臂覆盖），修复后重跑 `python tests/product/verify_image_preview.py --auto-bin <auto> --ui-tool-bin <tool> --out-dir <isolated> --backend native`。其余轨道（tools 门、031/029 生成、Vue 轨）不阻塞；P1-P5 平台边界已按 decisions.md §7 落地。（P6 已于 r2 定位并关闭，见 §9 r2 记录；本条留作历史。）
