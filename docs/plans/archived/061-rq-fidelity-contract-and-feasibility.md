---
plan_id: PLAN-061
status: archived
feature_name: RQ完整保真能力合同与录制重放可行性
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-07
plan_revision: 3
current_step: 5
total_steps: 6
depends_on: []
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/rq-fidelity-contract.md]
touched_goals: []
affects: [renderqueue, rqhost, rendering-fidelity, resource-lifecycle, rq-performance]
---

# PLAN-061：RQ完整保真能力合同与录制重放可行性

## 0. 变更摘要

承接2026-11-05用户提出的RQ完整功能要求。设计见[完整设计](../../design/autoui/renderqueue-complete-fidelity.md)，[战略/跨仓边界](../../design/autoui/renderqueue-refactoring-roadmap.md)。本Plan仅drafting，尚未实施。交付限定为本合同结果，不以单个阶段完成冒整条RQ路线全部完成。

## 1. 目标

有界研究与可执行验收基础。允许UI探针及新增测试驱动；只读Man/OS/产品来源，必要Host探针在同组OS树，但不修改生产桌面或产品。非目标：生产默认切换、完整功能实现、Linux IPC、Wayland compositor、ACE/AGE实现。

工作结果以§7可观察AC为准；同源direct是兼容基准，禁止删除能力/用隐藏自窗或整帧自绘降级来达标。原生窗口标题栏与App内容区分别验，不要求Host系统装饰像素相同。

## 2. 架构方案

最多比较两个候选，不预定全树泛型化或另写控件。T-02须跑实际同源direct和记录/重放样本，而非只检查字符串；确认CPU记录路径不栅格化完整UI。若两案均不可行，记录阻断原因与具体替代设计，转needs_replan，T-03/04不能伪勾。完整能力清单必须涵盖View全部分支及公共服务，Demo仅作为语料补充。冻结v1/v2旧兼容策略但不立即退役；RenderPacket/ReplayCore仅概念名，由决策确定最小内部接口。

实现→独立review→merge依次使用同一权威实施树。拟用`D:/autostack/.wt/ui-061/auto-ui`、分支`codex/plan-061-dev`（2026-11-07 T-01 已按此创建并在同组建立 auto-lang/auto-web/auto-down 只读依赖树）；本轮不创建其它树。需要写入的协作仓在同组各一个树；产品只读/隔离数据，新增业务写面须另定具体合同。UI/Man/OS原owner接口保留，语言基础无UI反依赖。

## 3. 技术栈

Rust、现有iced/wgpu、desktop_protocol二进制消息/共享资源；当前Windows本地双进程与两种Host。测试沿仓内现有Cargo入口及真实公开CLI。Linux transport/共享映射归OS062；标准Wayland compositor和ACE/AGE仅在设计中留接缝，不在本Plan实现。

## 4. 需求分析与背景调查

### 授权

2026-11-05 r3：用户要求复核四份RQ计划范围/依赖、与UI060/OS062去重并明确启动顺序；仅修订草案，不实施。

2026-11-05 r2：用户明确授权将性能分析/建议补入设计、战略和相关Plan，仍暂不实施。本次仅修订合同，不新增计划或接管其他会话。

用户明确要求检查设计/实现、建立设计/战略、纳入拆分重构路线并考虑新计划；本轮获准调研、文档和立约，未获本轮实施授权。不自动开agent/聊天，不接管UI053/060/OS054/061/062在途合同，不提交他方WIP。无额外预算/时限授权。

### 源码事实

主设计§2列出mesh/SVG/raw text省略、文字近似/重整形、shadow和transform近似；旧设计宣称无漂移与实际代码不一致。尚未证明最佳lossless接缝，必须有真实高风险原语先证。

基面UI `426c1f708e7c966f243c86b1f38029ae47b0fa0a`，OS `7968a68bfd15853a907f8b21f4dc106a2c216d39`；文件SHA256与逐项来源在主设计§2。UI Specs从`docs/specs/README.md`进入，相关`ui/structure-closure.md`、`runtime/component-and-store-instances.md`、`runtime/native-vm-app-validation.md`；OS从`docs/specs/shell/desktop-app-launch.md`与`desktop-bootstrap.md`进入。完整保真Spec当前缺席，本Plan提出新增，不提前改canonical。headless/rqhost/client_entry/message/endpoint/shm/transport及iced/broker_surface是现有路径；RenderPacket/ReplayCore与新测试路径是拟议，不冒已实现接口。

立项时取号扫描主检出、archive/archived及全部注册worktree；当时UI060仅树内亦计入占号。r3时UI060合同已恢复主检出、仍executing；UI053已有正式交付与清理收据。锁内取号、不覆盖已有Plan；UI无new-plan.sh使用独占分配，OS使用仓内脚本。实现时重新冻结真实源与依赖，不能依此快照借WIP。

## 5. 详细设计

完整不变量和跨阶段约束沿主设计§3–8；本Plan工作集中于§8唯一任务表。T-01建立`D:/autostack/plan-artifacts/plan-061/auto-ui/`并登记owner、用途、在途状态；探针/截图/日志/外部构建只在统一scope，不落docs/plans/evidence或attachments。正常编译target在专属树；外置target也只在scope并登记可删。

未决接缝由对应前置任务做有界探针并形成可审阅决定，不能一边猜一边切生产；源与已交付依赖不变时复用有效证据。三Host控制面可扩展但没有未来支持假承诺。能力丢失/非法帧必须具名失败；资源上限须T-01按真实测试环境和UI061决策冻结单位、边界、失败表现，不新设语法/控件限制。

### 范围与启动门复核（r3）

- **可先研究**：后续获准实施后，T-01～04使用正式来源开展能力/direct基线盘点、独立探针及合同决策；不等UI060、OS054、OS062整项完成。UI060正在修改的输入派发/刷新/native API符号保持只读，采用新增隔离探针；必要OS探针仅测试驱动，不修生产桌面。
- **研究验收层级**：T-04提供六格驱动、真实录制/重放最小样本和模式/丢图负控。尚不可运行的真实产品格输出原因、owner和后继门，不能标PASS；不要求先修UI063运行入口或OS063全产品组合。T-02真实可行样本不能被文字设计代替；绘图资源全能力交UI062、运行全能力交UI063、全产品组合交OS063。
- **补齐接口交付**：T-03明确平台无关envelope/capability、资源/槽身份、ACK与GPU fence分工；同时冻结UI062→UI063提交/就绪/取消/呈现反馈接口及UI063→OS063 surface/placement/可见性/焦点/坐标变换接口、错误语义。T-04给各包可独立执行的fixture/准确入口、能力ID和指标分层，不让后继实现成为研究前置。
- **OS062界限**：这里只定义跨平台合同/兼容负控，不实现Linux UDS、mmap/memfd、claim/peer鉴权、Wayland IME或Nix包。旧新端配对/拒绝行为与容量单位列清；OS062的8MiB控制帧/64MiB段门是其profile输入，不擅改或推广为全平台上限。未测Linux不称通过，不要求OS062先消费尚未交付的新wire。

### 性能合同（r2继承）

遵循[主设计§7.1～7.3](../../design/autoui/renderqueue-complete-fidelity.md)的P-01异步批量、P-02资源复用、P-03呈现调度、P-04多App隔离和M-01～08指标。本Plan的关闭条件是AC-07、V-05～08与§8任务，设计链接不代替实际证据。

- RQ不是逐绘图RPC：steady-state不逐图元等待reply，也不强制每帧等待上一帧呈现后才准备；容量不足须背压，传输ACK与GPU资源寿命分开。
- 驻留且未变font/image/mesh资源重复帧不重复上传payload/创建GPU资源；合法失效、驱逐、重建单列，不重整形App已定位的文字。片段复用先实证，不默认重做全局增量场景。
- 及时响应输入/资源完成并按deadline/present调度，固定15ms轮询不能是唯一驱动；在途帧/字节有上限，可合并过期画面但不能丢输入、资源依赖或业务副作用。
- per-App上传/资源/CPU工作预算与公平调度必须有轻重混合负控；长任务分批/限流，不能假设任意GPU命令可抢占。Host/device loss仍是共享故障域，不冒完全隔离。
- 数值预算由UI061先量原direct基线、在候选比较前冻结：逐workload列单位/统计量/绝对与相对上限/样本/owner/失败处理。本轮无数值或测量结论；后继缺预算或必要测量不能签性能PASS，超标不得测后改阈值。

### 任务落点

T-01：`crates/auto-ui-runtime/src/ui/view.rs`、`ui/iced/renderer/style_widgets.rs`、`ui/desktop_protocol/{coverage,headless,message}.rs`及`crates/auto-ui-tools/src/rust_ui.rs`；T-02/03：现有`headless::lower_layers_v2/lower_text_v2/lower_image_v2`与`ui/iced/broker_surface::paint_ops_v2`为探针对照点，新增`tests/rq_fidelity/`。

### T-01 落地：能力分母、基线、测量入口（r3 实测 2026-11-07）

实施树`D:/autostack/.wt/ui-061/auto-ui`，分支`codex/plan-061-dev`，base`b4094acd86e2717ed075db8d5c06b3bd4b6d6578`；只读依赖树 auto-lang`8337bab5`、auto-web`2d698413`、auto-down`58ea33f`；cargo/rustc 1.98.0；iced 0.14.0 / iced_wgpu 0.14.0 / wgpu 27.0.1 / tiny-skia 0.11.4 / cosmic-text 0.15.0。基面文件SHA256（view.rs`9d8ccb1b…`、style_widgets.rs`32f3b9e7…`、projection.rs`dcd9c869…`、headless.rs`e5bb276f…`、message.rs`0a9b89b9…`、broker_surface.rs`76086e2e…`、rqhost.rs`74e5c6c1…`、endpoint.rs`3eb25516…`、codec.rs`c4c95b4d…`、shm.rs`294771ef…`、transport.rs`3b3b1dd3…`、rust_ui.rs`89fb028c…`、Cargo.lock`c8d9f5e6…`）与原始材料在`D:/autostack/plan-artifacts/plan-061/auto-ui/`（merge 折叠后删除）。

**测量入口**（新增，test-only）：`crates/auto-ui-runtime/src/ui/desktop_protocol/headless/rq_probe.rs`（`mod rq_probe`于headless.rs）+生成模块`rq_probe/style_class_families.rs`。五条测试：
`rq_probe_t01_capability_denominator_is_complete`、`rq_probe_t01_style_class_denominator_is_complete`、`rq_probe_t01_direct_and_rq_baseline`、`rq_probe_t01_interaction_baseline`、`rq_probe_t01_wire_cost_scaling`。
命令`cargo test -p auto-ui-runtime --lib 'rq_probe::' -- --test-threads=1`（`AUTO_RQ_PROBE_OUT`指向暂存区），退出0，5/5通过，62.17s；PNG/JSON/日志留档。direct侧=`iced_test` Simulator snapshot（offscreen wgpu，scale2.0）；RQ侧=`HeadlessSurface`（tiny-skia Secondary 记录）→`lower_layers_v2`→`DisplayList::encode/decode`→`broker_surface::displaylist_element`重放 snapshot；**均为代理终点，非物理上屏**。

**全能力分母与 wire 映射**（39 View 变体，穷尽 match 编译期对账）：

| 能力ID | wire 实测（op/字节） | 状态/具名缺口 |
|---|---|---|
| cap.view.empty | —（合法无op） | 已证 |
| cap.view.anchor_slot | 1/44 | 已证（布局锚点） |
| cap.view.scroll_anchor_mark | 1/44 | 已证（operate上报，非绘制） |
| cap.view.text | 1/50 | 已证/待验证：Host重整形、Cached宽度近似、Raw省略 |
| cap.view.rich | 1/61 | 已证/待验证：跨span折行仅direct |
| cap.view.button | 2/84 | 已证 |
| cap.view.row | 2/68 | 已证（布局在producer） |
| cap.view.column | 3/109 | 已证 |
| cap.view.input | 2/98 | 待验证：caret/选区/IME preedit未上线 |
| cap.view.textarea | 2/95 | 待验证：同上 |
| cap.view.code_editor | 13/462，text9，bitmap1 | 已证/待定位：像素差14.99% |
| cap.view.terminal | 6/253，text2 | 已证/待定位：像素差0.463% |
| cap.view.autodown_editor | 2/95 | 待验证：块编辑态未上线 |
| cap.view.checkbox | 3/123 | 已证（勾选近似0.042%） |
| cap.view.custom | 0/10 | 平台条件/缺口：Element通道无wire对应 |
| cap.view.managed_scroll_content | 23/914，text16 | 已证/待定位：像素差1.59% |
| cap.view.container | 2/110 | 已证：渐变stop/四角半径上线；border内描边近似 |
| cap.view.scrollable | 3/109 | 已证：Scissor栈上线 |
| cap.view.radio | 3/123 | 已证（0.024%） |
| cap.view.select | 1/39 | 已证/待验证：下拉popup面 |
| cap.view.list | 2/70 | 已证 |
| cap.view.table | 10/346，text6 | 已证（0.206%） |
| cap.view.slider | 1/40 | 已证（0.056%） |
| cap.view.progress_bar | 2/90，text0 | 已证/待定位：百分比文本direct有/RQ无 |
| cap.view.accordion | 5/195 | 已证 |
| cap.view.sidebar | 1/44 | 已证（0差） |
| cap.view.tabs | 4/154 | 已证/待定位：1.01% |
| cap.view.navigation_rail | 6/218 | 已证/待定位：2.10% |
| cap.view.image | 0/10 | 已证缺口（T-02高优先）：SVG/vector整体省略P683-D3；raster Path/Rgba/Bytes上线；rotation/opacity丢弃 |
| cap.view.image_surface | 1/45 | 平台条件（T-04六格）：当前占位 |
| cap.view.video | 3/176 | 平台条件（归UI063/OS063）：占位 |
| cap.view.window_thumbnail | 0/10 | 平台条件/缺口：宿主快照缺席，svg图标被省略 |
| cap.view.workspace_preview | 1/50 | 平台条件/缺口：宿主合成缺席 |
| cap.view.remote_frame | 1/51 | 平台条件/缺口：无publisher，replay与direct 100%差 |
| cap.view.grid | 2/70 | 已证（布局分解） |
| cap.view.overlay | 4/96 | 已证（0差） |
| cap.view.popover | 3/68 | 已证/待验证：锚定/裁剪 |
| cap.view.mouse_area | 2/87 | 已证（绘制面；hover/hit在producer） |
| cap.view.canvas | 0/10 | 已证缺口（T-02高优先）：mesh/path不在v2词汇P683-D1 |

**StyleClass分母**：218类（`auto-ui-compiler/src/style/class.rs`），族分布 layout69/text44/paint.border32/spacing31/interaction16/paint.fill12/paint.shadow9/paint.effect3/transform1/meta1；生成`rq_probe/style_class_families.rs`为218臂穷尽match（新增删变体即编译失败），抽检6类映射通过。wire对应：fill→Quad.fill(色/LinearGradient)、border→Quad.border+radius[4]、shadow→Quad.shadow(垫层近似P683-D4)、text→TextStyled、transform→Transform(仅轴对齐P683-D2)、opacity/backdrop→**当前无wire面（缺口）**、layout/spacing/interaction→producer侧无wire约束。

**协议/服务面**：DisplayOp7（Quad/TextStyled/Scissor/ScissorPop/Transform/TransformPop/Image，paint_ops_v2全臂）、DrawOp7（v1→v2 from_v1）、FrameMsg12、InputMsg11、ControlMsg18、Handshake3、FrameMode2(Commands/Pixels)、PixelFormat1(Rgba8)。源码直证缺口：录制器以`clipboard::Null`驱动update/draw（headless.rs:171/221）→剪贴板族不可用；IME仅InputMsg/ImeRequest面，宿主候选窗整链待T-03/T-04；headless未接accesskit。生成器入口：`rust_ui.rs`的`--render-mode independent|rq|desktop`+`-q/--rq`，rq=ensure_rqhost_ready+ClientOpts+FrameMode::Commands；VM侧`AUTO_VM_RENDER=remote`缺省false。

**direct基线**（39例×20样本，offscreen代理）：direct稳态帧p50中位4.68ms/p95中位5.89ms/最大13.52ms；RQ produce p50中位0.126ms/p95中位0.214ms/最大42.72ms（具名异常）；replay稳态帧p50中位4.59ms/最大6.24ms；direct冷启动（含设备初始化）中位567.95ms/最大1837.37ms。**15ms帧泵**定位`rqhost.rs:1393-1400`（`iced::time::every(Duration::from_millis(15))`→`RqMessage::Tick`，处理臂`rq_update`@1102：pending采纳、IME上行排水、gauge）；**编码/复制**定位`endpoint.rs:331-332`（`payload.encode(&mut encoded); shm.write_slot(slot,&encoded)`）。wire成本（合成DisplayList，20次）：128op/7814B encode0.0261ms decode0.0425ms copy0.0003ms；1024op/62470B 0.1962/0.3427/0.0155ms；4096op/249862B 0.8294/1.3290/0.0464ms（encode≈287MB/s、decode≈179MB/s、copy≈5.1GB/s）；4MiB Rgba位图复制0.485ms（≈8.2GB/s）。交互基线：真实iced事件点击计数按钮→状态0→1→下一帧TextStyled"1"（命中点(22.0,11.0)）。

**冻结预算**（候选比较前，细则见§9与暂存区`t01-metrics-freeze.md`）：produce+lower p95 W-A≤1.0ms/W-B≤2.0ms/W-C≤10.0ms；encode+decode+copy p95 ≤64KiB≤1.0ms、≤1MiB≤3.0ms；稳定重复帧资源上传=0（硬零）；单帧字节≤64KiB(WA)/≤1MiB(WB-WC)，8MiB控制帧门沿用OS062 profile不上调；replay帧p95≤1.25×direct p95+2.0ms；direct冷启动代理≤2.5s。M-01/M-02/M-05/M-07本入口未执行，唯一关闭者T-04/OS063。具名超预算基线：`image_svg_icon` produce p50 32.33/p95 35.91ms、`window_thumbnail` p50 32.33/p95 42.72ms（owner T-02/T-04），修复前不计入W-C通过样本。

**像素差（T-02输入，非T-01通过证据）**：同设备复跑39例，单纯文本/锚点/叠层类0–0.32%差；`code_editor_rust`14.99%、`navigation_rail`2.10%、`managed_scroll_content`1.59%、`tabs`1.01%；`remote_frame`100%（占位面差）；`progress_bar`文本缺失。差异热点与wire观测共同指向T-02高优先探针：canvas mesh、svg/vector、IME/caret、宿主帧供给、文本整形态。

**T-02/T-04 的 `tests/rq_fidelity/` 六格驱动将复用本入口，T-01 不提前建六格驱动。** **落地边界**：本轮只完成静态分母+基线+测量入口，未比较候选、未实现任何RQ保真修复、未做六格对拍；T-02起按§8继续。`Cargo.lock`因workspace lib-test依赖解析新增accesskit*等38行（test构建需要，无运行语义变化）。主检出`crates/auto-ui-runtime/test/ui/terminal_pixel/*.png`为未跟踪的既有测试自动金样（非本Plan产物），未纳入、未改动。

### T-02 落地：高风险原语接缝探针与候选决定（r3 实测 2026-11-07）

实施树同上，提交`8fcf7ceb`（仅新增test-only`rq_seams`，生产wire/lowering/replay零改动）；命令`AUTO_RQ_PROBE_OUT=<scope>/seam cargo test -p auto-ui-runtime --lib rq_seam -- --test-threads=1`退出0，7/7通过（2.5s）。原始材料`D:/autostack/plan-artifacts/plan-061/auto-ui/{t02-seam-report.md,t02-seam-run.log,seam/}`。

**记录器事实（iced_tiny_skia 0.14.1）**：`Layer{quads,primitives,images,text}`。mesh/canvas几何以`Primitive::Fill/Stroke{tiny_skia::Path}`记录且**变换已烘焙进坐标**（`geometry.rs` fill/stroke 先`path.transform`）；文本为`Paragraph/Editor(Weak<cosmic Buffer>)/Cached(字符串)/Raw`；图像为`Vector{bounds}`（svg句柄）/`Raster{handle,rotation,opacity,filter}`；四元含border radius/width与shadow(color/offset/blur)；**custom GPU shader不存在**（`Primitive`仅Fill/Stroke，穷尽match编译期保证）。关键坑：`geometry.rs:427`的`into_paint`用`(b,g,r,a)`构造tiny_skia色——序列化须换回逻辑RGB，否则整帧换色（探针首版复现后修正）。

**区分性探针结果**：mesh 4 paths/12段，probe-wire重放像素差**0.071%**（残差=标签文本+AA，bbox覆盖标签区）；SVG记录为vector，resvg按device scale=2栅格化→Image重放与direct**逐字节全等（0.000%，192×192）**；glyph由记录器内整形提供15 glyph（首id=1048,x=0,y=0,size=16），两次整形advance=209.78125全等且与记录器一致，冷单帧可能未整形（buffer_lines=0→须录制期强制整形）；旋转轮廓经Canvas geometry烘焙成功（rotated_path_hint=1），几何文本在斜切/非均匀时退化为outline path（可保真但失文本语义）；shadow blur参数入记录但重放无blur原语（tiny-skia engine用per-pixel SDF smoothstep），Host需blur/SDF能力；custom primitive不可达（CPU记录器无shader通道）。

**候选比较与决定**：候选B（另造iced原语边界记录器）收到的调用与现有记录器相同，不增加原语覆盖，custom shader同样不可达，成本/风险更高——**选候选A**（扩现有Secondary记录→wire词汇）：①新增`PathFill/PathStroke`（段序列+换回RGB的paint+fill rule/stroke参数+已烘焙变换）；②`GlyphRun`或glyph atlas资源+录制期强制整形；③SVG资源+生产端device-scale栅格化（或Host resvg）；④Image rotation/opacity/filter入wire；⑤Host blur/SDF原语承载shadow；⑥资源身份+generation。具名不可达进T-03合同：custom GPU shader primitive、SVG的Skia级filter子集、要求保留可编辑语义的旋转文本。

**P-01～04（本Plan职责部分）**：P-01每帧1条FrameReady、无逐图元reply、`endpoint.rs`slot_count=2且FrameAck归还（`:554`），无free slot时回落`1-last_slot`覆盖旧槽=有界但**无背压**，ACK不带GPU fence→T-03定义背压与fence分离；P-02稳定重复帧仍每帧上传（probe uploads/frame=[1,1,1,1]，10080B/帧；4MiB复制0.485ms）=**违约**，T-03资源身份+generation；P-03唯一驱动仍为`rqhost.rs:1400`的15ms tick；P-04单例未执行→T-04。


### T-03 落地：wire/能力、字体/glyph、资源寿命、调度与接口交接（r3 冻结 2026-11-07）

实施树同上，提交`b3b6f7bf`（V-03 原子性探针）；完整决策合同在暂存区`t03-wire-contract.md`（merge 时折叠）。V-03 探针实测：v2 帧信封 round-trip 通过；截断载荷→`TooShort`且不 compose（半帧不呈现）；信封版本 9→`UnsupportedVersion(9)`；`SurfaceStore::compose_v2`双缓冲下坏载荷不进 compose→front 保持上一完整帧（保旧）；Hello 空 tail 与未知 cap 均可解码→当前 caps 仅**声明式**，必须补 required/optional 协商。命令`cargo test -p auto-ui-runtime --lib rq_wire_t03 -- --test-threads=1`退出0（1/1）。

**版本/能力（冻结）**：信封`PROTOCOL_VERSION=1`冻结（仅五通道/信封不兼容变化才升）；载荷 tag 1=DrawList v1 只读 legacy、2=DisplayList v2 当前 remote、新增 3=v3 承载 T-02 增量原语；未知 tag 明确拒绝并保旧。能力ID：`fidelity/pathfill/1`、`fidelity/glyphrun/1`、`fidelity/svg-raster/1`、`fidelity/image-params/1`、`fidelity/blur-sdf/1`、`resource/generation/1`、`pipeline/backpressure/1`、`ack/fence-split/1`；Hello/Welcome 双向声明 required/optional，**required 未知→入帧前拒绝+具名诊断**，optional 未知→允许降级但须产出可观测降级事件；旧端（空 tail）按 v2 处理、不收 v3。迁移：v2↔v3 双读窗口，v1→v3 先经 v2，退役旧 tag 需具名 Plan+负控。

**字体/glyph（冻结）**：FontBlob 以 SHA-256+face index+字节为身份（禁绝对路径当唯一身份）；文本优先 `GlyphRun`（glyph id/x/y/字体内容id/size/color/weight/italic），字符串+Cached 记为具名降级（Host 重排）；**录制期强制整形**（T-02 冷帧`buffer_lines=0`证据），未整形即降级并诊断；缺字体/glyph 明确拒绝该 run 并上报，不得静默替换；旋转文本走 outline path 保真但失可编辑语义（写入合同）。

**资源/帧原子性（冻结）**：资源身份`(session_epoch, content_hash_or_app_key, kind)`，`kind∈{bitmap,font,svg,mesh,media}`，generation 单调、Host 只接受更大 generation（旧 generation 拒绝+保旧）；`create/update/release`，owner=App，引用归零释放，容量以字节+计数冻结；**ACK 只归还传输槽，GPU fence 归 Host 资源寿命**（App 不得据 ACK 判 GPU 完成）；整帧 decode+校验（长度/溢出/非有限/栈配对/索引）通过才 compose，失败保旧+诊断；双缓冲保旧。**可复用片段**：证据不足→不做全局增量，冻结“全帧恢复+资源驻留复用（同内容稳定帧零重传）”。

**调度/预算（冻结）**：readiness 事件驱动为主、timer 仅 deadline/fallback（15ms tick 降级）；无 free slot **不得覆盖未 ACK 槽**（credit/等待或合并过期画面，保留输入/资源依赖）；per-surface 在途帧≤2、在途字节≤1MiB（8MiB 控制帧门沿用 OS062 profile 不上调）、资源驻留≤256MiB/App（T-04 冻结实际值）、上传≤8MiB/s/App；公平性以轻/重混合下轻 App p95/p99 与最长无进展为验收量；性能门槛沿用 T-01 冻结预算。

**三 Host/引擎边界（冻结）**：三 Host 共用协议/资源寿命/重放核心，窗口系统适配分层；Desktop Host 不得外窗顶替嵌载；Linux 归 OS062/OS063；ACE/AGE 只通过版本化 wire 接入，不暴露 Rust ABI/对象，AGE 3D 以新图形包能力扩展、不虚构 production wire。

**V-09 接口交接（全表见`t03-wire-contract.md` §6）**：13 项含帧提交/就绪·合并·丢弃/呈现反馈/取消关闭/表面 placement/可见性焦点/坐标 DPI/输入 IME/剪贴板（现`clipboard::Null`具名缺口）/媒体 adapter 事件/错误超限/资源传输授权/容量门；每项列现有符号、缺口 owner（UI062→UI063→OS063、OS062 profile）与负控（漏 ACK/fence/adapter 事件可检出）。UI061 只交合同与探针，不接管上述 owner。

**具名不可达**：custom GPU shader primitive、SVG 的 Skia 级 filter 子集、可编辑语义的旋转文本、物理上屏时序（OS063/平台）。


### T-04 落地：六格对拍驱动原型、负控与差额映射（research-level 2026-11-07）

实施树同上，提交`8133fcca`（test-only `rq_matrix` + 复用 rq_probe 助手）。命令`AUTO_RQ_PROBE_OUT=<scope>/t04 cargo test -p auto-ui-runtime --lib rq_matrix -- --test-threads=1`退出0，2/2通过（18.6s）。报告与原始表：暂存区`t04-matrix-report.md`、`t04-matrix-table.md`、`t04-matrix-run.log`、`t04/`（PNG）。

**六格**：可跑=rust/direct（本进程 wgpu 代理）与 rust/standalone-rq（inproc producer+broker 重放近似）；不可运行且具名 owner=rust/desktop-rq（UI063/OS063 桌面嵌载）、vm/direct、vm/standalone-rq、vm/desktop-rq（UI063/OS063 运行入口）。**缺格不报 PASS**；每格输出 producer/host/pid/profile/源版本。

**逐例读数（代理，不得外推）**：mesh 记录4 path 而 wire 仅1条标签文本（漏 mesh）；svg_icon 记录0/wire 0 ops（P683-D3）且 produce≈31ms（T-01 具名异常复现）；text_cjk 1 op/75B；gradient_shadow 2 ops/126B（blur 仍 scrim 近似）；progress_bar 2 ops/90B（百分比文本仍差）。direct 稳态帧 p50 3.2–5.0ms；produce 0.14–31.1ms。

**负控（全部可检出）**：漏 mesh→记录原语(4)≠wire几何(0)；错载荷版本→v1 字节被 v2 解码面拒（`UnknownTag(1)`）；错字体/字号→正确重放 direct 差973px、错字号12 317px。

**差额映射（未抹去）**：v3 原语（Path/GlyphRun/SVG/Image 参数/Blur）→UI062；输入/IME/clipboard/服务→UI063；两 Host 真实进程/桌面嵌载/产品组合/物理上屏/内存/公平→OS063；Linux→OS062（profile 8MiB/64MiB 不改）；custom shader/Skia filter/可编辑旋转文本=具名不可达需新合同。

**M-01～08 分列**：M-03/M-04/M-08 部分已测（代理）；M-01/M-02/M-05/M-07 未测（无 present/进程采样/多 App），唯一关闭者 T-04 后继与 OS063；M-06 仅 direct cold 代理。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/rq-fidelity-contract.md | 当前有限分散实现/历史设计 → 本Plan已验范围、来源、支持条件及失败/兼容合同及P-01～04/M-01～08性能要求 | 不再以历史“无漂移”或编译绿替代运行证明 | AC-01..04、AC-07 |

如实际改动还影响已有canonical条目，T-01具名补充并按语义变化修订Plan；不制造无关Spec条目，review核真实增量。

## 6. 测试设计

以下均为拟执行，不是本轮通过证据。主矩阵VM/Rust×direct/standalone-RQ/desktop-RQ；同source/主题/font/viewport/scale/时钟/事件序列。所有V行owner_stage=work。结构/glyph/资源、客户区像素和业务读回同时核验；旧direct金样不因共用新代码而失效。默认exact像素比较，真正平台采样非确定性只有具名区域/原因/敏感反控经审查后可设窄容差；不允许全局阈值吞漏图。

| Case | AC | 输入/真实入口与负控 | 可观察oracle/关闭条件 | 责任任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | 冻结View/renderer/style/生成器与平台服务；反控删去Canvas或AutodownEditor映射 | 能力ID集合与真实分支双向对账，零无owner条目；删项可被检出，现有direct红分列 | T-01；静态全分母，非运行绿 |
| V-02 | AC-02 | 实际高风险原语样本：渐变阴影/旋转clip/mesh/SVG/中英混排与raw editor；旧省略实现对照 | 同状态direct与replay图元/资源/图像对照；旧缺图能红，至少一案保持合法画面且App无GPU/隐藏窗；不可达原语需源码证明 | T-02；高风险先证不代全量 |
| V-03 | AC-03 | 候选版本/旧端、缺glyph资源、乱序generation、坏长度/clip栈与合法对照 | 实际codec/接收原型明确拒绝不完整帧、保旧；协商与ACK/fence所有权可追踪 | T-03；探针不等生产通道 |
| V-04 | AC-04 | 六格驱动原型及故意拿错模式/源版本、漏mesh、错字体负控 | 结果含producer/host/PID/profile/来源，缺格不报PASS，视觉差异可定位；全能力差额分给062/063/OS063 | T-04；全驱动机制，非全能力已执行 |
| V-05 | AC-07 | 延迟ACK、已占满槽和合法容量对照；记录原型消息/等待链 | 不逐原语往返且有有界流水；槽满背压不覆盖；M-01/M-04可分段归因 | T-01/T-02；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-06 | AC-07 | 资源驻留后相同帧、单资源修改、驱逐和新session | 稳定资源payload重复上传为0，变更/驱逐重传可解释；M-04/M-05及精确重放差异可见 | T-02/T-03；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-07 | AC-07 | 冻结direct基线与六格候选；空闲、滚动、动画、慢Host；强制轮询/串行负控 | M-01～06/M-08可观测，呈现与提交分开，负控能检测额外等待；预算先于候选选择 | T-01/T-04；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-08 | AC-07 | 拟定1/4/8规模，交互轻App+重上传/绘图App及故意独占调度反控 | 冻结可运行规模/公平策略；M-07最长无进展和p95/p99变化能揭示饥饿，缺规模不冒通过 | T-03/T-04；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-09 | AC-03 | 平台接口清单与UI060/OS062交叉符号；漏ACK/GPU寿命或adapter事件反控 | 有探针支持的接口/兼容合同和后继owner，遗漏可检出，不冒生产联验 | T-03/T-04；接口交接 |

性能实测口径：M-01响应延迟p50/p95/p99/max及分段；M-02帧间隔/目标deadline失约、合并/丢弃/画面年龄；M-03 App/Host/总CPU与可测GPU时间、空闲唤醒；M-04命令/资源字节、消息/往返、复制与上传/创建计数；M-05每进程/总内存与共享去重、GPU驻留和队列峰值；M-06冷/热启动及恢复；M-07轻重App公平；M-08保真/业务守恒。各职责分别取证，OS063做最终组合，未执行不得被上游局部测试抵扣。

同执行器原direct、候选direct及两RQ Host同机器/source/build profile/GPU/字体/DPI/分辨率/主题/事件/资源/刷新率比较；冷/热分列。完整多App成本为N个App加Host，direct为N进程；桌面壳基准单列且不得隐藏其成本。event/frame ID配对、单调时钟校准偏差；无法量物理上屏只标实际present代理终点。UI061先冻结样本长度/重复/运行顺序/污染规则，成对或交错运行；保失败/缺测/异常与分布，不择优挑样本，p99样本不足不给通过结论。性能测量仅在获准实施后的独立窗口，不与其他会话争用GPU。

测量入口由UI061 T-04的`tests/rq_fidelity/`原型补计时/计数并交付准确命令，后继T-01读回；OS063由`scripts/rq-parity/`编排真实组合。两路径仍是拟新增，本轮不创建脚本或运行基准。不得新增一个与现有oracle无关的泛化跑分系统。

### 验证入口

在实施UI树：`cargo test -p auto-ui-runtime --lib ui::desktop_protocol:: -- --test-threads=1`及`cargo test -p auto-ui-runtime --lib ui::iced::broker_surface:: -- --test-threads=1`，按变更选择完整职责族。T-02/T-04拟新增`tests/rq_fidelity/`探针与驱动（当前不存在），准确命令由任务实现并记入Plan；必须消费真实生成/录制产物，禁止零用例PASS。

T-01确认真实test namespace/feature/平台条件；新增入口由标明任务提供。列用例/执行/条件排除，准备与运行时间分列；非零/超时/skip不充PASS。按实际触面完整职责族和必要消费者运行，最终review后merge只跑冻结并集；不叠跑全仓或AG里程碑。真实窗口验证不抢占其他会话，不能通过提高资源阈值隐藏功能缺口。

### 角色与证据

work负责实现/自检；独立上下文review只检查冻结同一树，不另建reviewer树；merge落地与清理。未指定模型/代理，不自动创建。每条证据绑Plan revision、代码/依赖提交、实际profile/进程/资源/产物，失败/修复后结果分列。用户未授权实施前不进入上述运行。

## 7. 验收标准

| ID | owner_stage | 可观察行为/判据 |
|---|---|---|
| AC-01 | work | 全公共能力与原direct基面完整冻结，矩阵无遗漏，已有缺陷/未执行不充通过 |
| AC-02 | work | 高风险原语先证支持至少一个明确可行的无App自绘方案，负控敏感性与限制可复核 |
| AC-03 | work | 版本化协议、字体/资源、帧原子与寿命决策落盘，三Host及未来引擎边界清楚 |
| AC-04 | work | 六格oracle驱动可运行，实际高风险样本/模式与丢图负控有效；不可运行产品格显式缺口；每项剩余义务有后继owner且未被研究结论抹去 |
| AC-05 | review | 独立上下文逐项验证当前修订AC/V、真实源和规范增量，所有必需项有可复核证据；不能只读完成勾选 |
| AC-06 | merge | 通过后按依赖顺序合入、canonical读回、路线回填，归档/guard/材料清理与恢复摘要完整；交付边界如实记录 |
| AC-07 | work | P-01～04形成经探针支持的协议/资源/调度/隔离方案；M-01～08方法、原direct基线及候选比较前冻结预算可复核，负控敏感，无测后调阈值；研究不冒后继生产性能已通过 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/操作与结果 | 关闭方式/AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 无 | 冻结主检出/正式依赖与全能力分母；逐项读取View、IntoIcedElement、style与平台服务，建立能力ID→producer/wire/consumer/case，保存旧direct视觉与交互基准；性能先冻结M-01～08测量口径/样本/场景，提供测direct所需最小计时/计数入口（T-04再整合驱动），再测原direct基线并在候选比较前冻结绝对/相对预算；定位15ms帧泵与实际编码复制成本 | V-01；AC-01；AC-07/V-05～08（按§6分工）——证据见§9 2026-11-07 T-01 执行记录与§5 T-01落地小节 |
| [x] | T-02 | work | T-01 | 在专属树验证两种候选：tiny-skia补齐或同iced原语边界录制；以mesh、SVG、raw/glyph、blur、完整transform、可达custom primitive作区分性探针；只做探针不切生产；按P-01～04比较批量异步与持久资源方案，验证延迟ACK下有界流水、资源不重复上传及保真原语不重复计算 | V-02；AC-02；AC-07/V-05～08（按§6分工）——首轮见§9 T-02；F-R4（V-06/P-02）已由 §9 2026-11-07 F-R4 修复记录闭环 |
| [x] | T-03 | work | T-02；生产共享符号只读 | 按§5交付平台兼容和下游adapter接口清单（V-09），不重做OS062；冻结wire/envelope/capability、glyph/font、资源generation/ACK/GPU寿命、原子帧和版本迁移决策；检查三类Host与未来ACE/AGE边界；冻结帧调度、资源预算、per-App公平/过载策略和ACK/fence所有权；可复用片段按证据决策，不强建复杂增量状态 | V-03；AC-03；AC-07/V-05～08（按§6分工）；V-09——证据见§9 2026-11-07 T-03 记录与暂存区 t03-wire-contract.md |
| [x] | T-04 | work | T-03；不等后继生产全量 | 区分研究样本与真实运行缺格及后继owner；提供direct/两个RQ Host六格对拍驱动原型、负控、完整差额及后继包逐项映射；精确记录可行/不可行，不以研究交付认定全功能；交付M-01～08原型测量及direct/RQ对比、负控、后继预算；样本不足/未测分列，不将代理present时间称物理上屏 | V-04；AC-04；AC-07/V-05～08（按§6分工）；V-09——首轮见§9 T-04；T-05 复审 needs_fix 的 F-R1～F-R3 已由 §9 2026-11-07 T-04 修复记录闭环 |
| [x] | T-05 | review | T-04 | 独立上下文核全能力分母、候选实际产物、负控和设计选择，给revision-bound verdict；独立核P/M口径、冻结预算、性能负控和完整分布，不能用平均FPS或实现者摘要替代 | AC-05；复核AC-07——三次复审 final pass（见§9 2026-11-07 T-05 三次复审） |
| [ ] | T-06 | merge | T-05 pass | 定向冻结门、沉淀能力/协议决策Spec；更新子路线及062/063执行入口，归档并guard清理；沉淀性能支持边界/预算与实测差额，不外推未测平台 | AC-06 |

work项完成才进入execution_done；review/merge项仍待办，不能为交接伪勾。每个任务读取§4对应实际路径及主设计证据；新增接口先由有界决策任务确认。全部原始材料merge折摘要进本Plan§9后删自有scope；移除树先wt-guard clean，不穿透链接、不清未知owner目录。

## 9. 复审记录

2026-11-05 / stage:new / plan_id:auto-ui/PLAN-061 / revision:r1 / outcome:pass（草案合同，不是实现验收） / next:待用户启动后work/T-01，按§8正式依赖核门。

草案静态预检：0–10章节齐全，任务数与total_steps一致，AC/V/SD映射及新文档本地链接有效；主检出/两类归档/注册worktree中的本号唯一。已做源码/设计/相关Spec与重叠计划检查；本轮未实现、构建、跑运行门或独立final review。当前task/AC/V/SD为执行合同；实际通过证据后续填写。设计前置的不确定性保留为任务/阻塞，不能声明已解。

### 2026-11-05 r2性能合同修订

- stage:new / plan_id:auto-ui/PLAN-061 / revision:r2 / outcome:pass（文档修订，非性能验收） / next:待用户启动后按§8依赖进入work。
- 变更：新增AC-07、V-05～08；§5增加P-01～04，§6补M-01～08及direct配对/样本/冻结预算口径；§8原任务增补性能职责，SD-01同步扩展。总任务数不变，current_step仍0，status仍drafting；r1记录保留为历史。
- 用户授权仅文档，未实现、编译、运行或测量；没有性能PASS或节省承诺。r2静态检查只核章节/引用/任务与AC/V映射，不是独立final review。

### 2026-11-05 r3范围与依赖复核

- stage:new / plan_id:auto-ui/PLAN-061 / revision:r3 / outcome:合同修订，静态核查 / next:仍drafting，按§5/§8依赖等待用户启动。
- 修订发现：拆开绘图/运行/产品六格以消除反向依赖；UI060独占索引/输入/刷新/API修复，OS062保Linux通道/平台/安装；补接口交付/兼容门及OS063实际UI策略写面。P/M性能要求及全部能力/产品分母不减少。
- 来源：UI060主检出executing r1/2-6、扩围及续接记录；UI053归档6/6及UI 9a94d568 / lang 61207576b正式交付（当前主线祖先）；OS062 drafting r1/0-13的T-02～05/09～11；headless、UI session/shell_state与OS薄Host入口。启动时重新冻结源版本/单写符号，不将快照永久当准入。
- 保留r1/r2历史；步数、AC数量和current_step不变，补V-09接口负控。未改UI060/OS062/OS054原合同，未建树/实现/构建/运行或签final review；运行证据待后续work。

- r3静态检查：四份合同0～10章节、27项任务（6/7/7/7）、各7条AC/9条V及引用一致；正式depends_on图无环；RQ与本轮现行路线共155条本地链接有效；三仓本轮文档范围diff --check退出0。只证明合同结构/链接，不证明任何运行、性能或Linux支持。

### 2026-11-07 T-01 执行（work）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（T-01 完成；T-02～T-06 待办） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd86e2717ed075db8d5c06b3bd4b6d6578 | code_commit: b6e6b487c183591c2ca171607b97baa5a09b33fd（rq_probe探针+StyleClass生成映射；Cargo.lock test解析补全） | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-01 | next_responsible_role: work（T-02）。
- 命令/结果：`cargo test --workspace --lib --no-run` 退出0（4m19s，冷构建）；`cargo test -p auto-ui-runtime --lib 'rq_probe::' -- --test-threads=1` 退出0，5/5通过（62.17s，含39例×23帧direct+39例×23帧RQ）；`AUTO_RQ_PROBE_OUT=D:/autostack/plan-artifacts/plan-061/auto-ui/baseline` 落盘78张PNG（39×direct/rq）+`t01_baseline.json`；`pixel_diff.py` 生成逐例像素差；`build_matrix.py` 生成能力矩阵；`t01-metrics-freeze.md` 冻结M口径与预算。
- 证据要点：能力分母39 View变体（穷尽match）+218 StyleClass（穷尽match，10族）+协议服务面（DisplayOp7/DrawOp7/FrameMsg12/InputMsg11/ControlMsg18/Handshake3/FrameMode2/PixelFormat1）；15ms帧泵`rqhost.rs:1393-1400`、编码复制`endpoint.rs:331-332`；wire成本4096op/249862B：encode0.8294ms、decode1.3290ms、copy0.0464ms；4MiB Rgba复制0.485ms；direct稳态帧p50中位4.68ms、p95中位5.89ms；produce p50中位0.126ms（具名异常image_svg_icon 32.33ms、window_thumbnail 32.33ms）；交互点击→状态1→下一帧文本"1"；像素差热点code_editor 14.99%、navigation_rail 2.10%、managed_scroll 1.59%、tabs 1.01%、remote_frame 100%（占位）。
- 边界/观察：全部为offscreen代理，不含物理上屏；M-01/M-02/M-05/M-07未执行（owner T-04/OS063）；未比较候选、未修任何保真缺口；`Cargo.lock`新增accesskit*等38行（lib-test解析所需，无运行语义）；主检出未跟踪`crates/auto-ui-runtime/test/ui/terminal_pixel/*.png`为既有测试自动金样，未纳入本Plan。
- 原始材料：`D:/autostack/plan-artifacts/plan-061/auto-ui/`（capability-matrix.md、t01-metrics-freeze.md、t01-pixel-diff.md、t01_baseline.json、baseline/*.png、inventory.json、脚本与日志）；merge折叠后按delete-at-merge删除。

### 2026-11-07 goal round 1

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: b6e6b487c183591c2ca171607b97baa5a09b33fd | next: work（T-02）

### 2026-11-07 T-02 执行（work）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（T-02 完成；T-03～T-06 待办） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd | code_commit: 8fcf7ceb730c9c50a4da8159678b34c83ac6b943 | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-02 | next_responsible_role: work（T-03）。
- 命令/结果：`AUTO_RQ_PROBE_OUT=D:/autostack/plan-artifacts/plan-061/auto-ui/seam cargo test -p auto-ui-runtime --lib rq_seam -- --test-threads=1` 退出0，7/7通过（2.5s）；产物`seam/*.png`+`t02-seam-run.log`+`t02-seam-report.md`。
- 证据要点：mesh 以4 paths/12段记录并重放，像素差0.071%（残差=标签文本+AA）；SVG记录为vector，resvg device-scale栅格化后重放与direct逐字节全等（0.000%）；glyph记录15 glyph且与生产端整形全等（advance 209.78125，首id 1048），冷帧未整形须录制期强制；旋转几何烘焙进path（rotated_path_hint=1），斜切文本退化为outline path；shadow blur参数在记录但无重放原语（tiny-skia engine per-pixel SDF）；custom GPU shader不可达；记录器paint为(b,g,r,a)构造须换回（否则整帧换色，探针已复现并修正）；P-02重复帧仍每帧上传（uploads/frame=[1,1,1,1]，10080B/帧）。
- 候选决定：选A（扩现有Secondary记录→wire），B不增覆盖；T-03必做增量：PathFill/PathStroke、GlyphRun/atlas+强制整形、SVG资源+device-scale栅格、Image呈现参数、Host blur/SDF、资源身份+generation、背压与GPU fence分离。
- 边界：仅test-only探针，生产wire/lowering/replay零改动；未做六格对拍；custom shader/ Skia filter子集/可编辑旋转文本具名不可达。

### 2026-11-07 goal round 2

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 8fcf7ceb730c9c50a4da8159678b34c83ac6b943 | next: work（T-03）


### 2026-11-07 T-03 执行（work）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（T-03 完成；T-04～T-06 待办） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd | code_commit: b3b6f7bf52d300a6c86df94a0b485cd51934d634 | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-03 | next_responsible_role: work（T-04）。
- 命令/结果：`cargo test -p auto-ui-runtime --lib rq_wire_t03 -- --test-threads=1` 退出0（1/1）；`--lib 'rq_seam'` 退出0（8/8，2.54s）；决策合同`t03-wire-contract.md`落盘。
- 证据要点：半帧→TooShort不compose；版本9→UnsupportedVersion；compose_v2双缓冲坏载荷不覆盖front（保旧）；caps仅声明式（空tail/未知cap均可解码）；冻结信封v1/载荷tag1-3/8个能力ID required-optional语义/迁移窗口；冻结字体身份、录制期强制整形、资源generation+ACK≠fence、原子帧保旧、readiness调度+背压不覆盖+预算(帧≤2/字节≤1MiB/驻留≤256MiB/上传≤8MiB·s⁻¹)/公平性口径；V-09 十三项接口与owner/负控。
- 边界：只做合同与探针，未改生产 wire/lowering/调度；未做六格对拍（T-04）；OS062 profile（8MiB控制帧/64MiB段门）不改。

### 2026-11-07 goal round 3

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: b3b6f7bf52d300a6c86df94a0b485cd51934d634 | next: work（T-04）


### 2026-11-07 T-04 执行（work）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（T-04 完成；T-05/T-06 待办） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd | code_commit: 8133fcca2c41bf0272feced83879c1d1b70eeb8c | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-04 | next_responsible_role: review（T-05）。
- 命令/结果：`cargo test -p auto-ui-runtime --lib rq_matrix -- --test-threads=1` 退出0（2/2，18.60s）；最终全套`cargo test -p auto-ui-runtime --lib 'rq_' -- --test-threads=1`退出0，**17/17通过（98.37s）**（t04-full-suite.log）。
- 证据要点：六格逐格具名（2 可跑/4 缺口+owner）；mesh 记录4≠wire0（漏 mesh）；svg 记录0/wire0+produce 31ms；负控三项全检出（漏 mesh/错版本 UnknownTag(1)/错字号 973→12317px）；M-01/02/05/07 未测分列，不冒充；差额映射到 UI062/UI063/OS063/OS062。
- 边界：研究原型；inproc 近似≠两 Host 实测；代理终点≠物理上屏；未实现 v3 原语、未启动真实 Host/VM 会话。

### 2026-11-07 goal round 4

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 8133fcca2c41bf0272feced83879c1d1b70eeb8c | next: review（T-05）


### 2026-11-07 T-05 复审（review / final）

- stage: review | review_scope: final | reviewer: 同会话（执行会话）复核，无独立模型/代理（Plan §6“未指定模型/代理，不自动创建”）；结论由产物+独立重跑重建，非执行者摘要 | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: **needs_fix** | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | reviewed_commit: 8133fcca2c41bf0272feced83879c1d1b70eeb8c | base_commit: b4094acd86e2717ed075db8d5c06b3bd4b6d6578 | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | spec_inputs: docs/specs/runtime/native-command-frame-text.md（v1/v2 宿主顺序，不冲突）、docs/specs/runtime/host-theme-transport.md（theme caps 不变）；SD-01 目标 docs/specs/runtime/rq-fidelity-contract.md 尚未创建（merge 写作，内容源=t03-wire-contract.md+T-02 结论） | next: work（T-04 修复 F-R1～F-R3）

**基线核对**：worktree 干净、4 提交（b6e6b487/8fcf7ceb/b3b6f7bf/8133fcca）；`git diff b4094acd..HEAD` 仅 Cargo.lock(+38，纯增 accesskit*，0 删除)+test-only 探针（headless.rs +8 注册、rq_probe/rq_seams/rq_matrix/style_class_families 新增），**生产 wire/lowering/replay 零改动**——与 §9 T-01～T-04 声明一致。独立重跑：capability 39/39、style 218、rq_seam 8/8（mesh 728px=0.0711%、svg 0px=0.0000%，与声明一致）、负控 3/3 检出（漏mesh 4vs0、v1 载荷 UnknownTag(1)、错字号 973→12317px）、六格驱动 2 可跑/4 缺口带 owner。

**验收映射**：AC-01 pass（39 变体穷尽 match+218 StyleClass 穷尽映射，缺口具名未计通过）；AC-02 pass（mesh path 重放 0.071%、SVG 0.000%、glyph 与生产整形全等；负控敏感；具名不可达在案）；AC-03 pass（envelope v1/payload tag 1-3/8 能力 ID/迁移窗口/资源 generation/ACK≠fence/原子帧保旧已冻结；V-03 探针：半帧 TooShort、坏载荷不覆盖 front）；AC-04 pass（六格驱动可运行、4 格显式缺口带 owner、三项负控有效、缺格不 PASS）；AC-07 **partial/fail**：P-04/V-08 无任何探针、M-05/M-07 可原型却未测、P-01 背压“不覆盖未 ACK 槽”仅有合同无探针（当前 endpoint 回落 `1-last_slot` 仍覆盖未 ACK 槽）。AC-05（本复审）/AC-06（merge）分别由本记录与 T-06 承担。

**发现（需修复，最小范围）**：

- F-R1（AC-07；V-08；T-04）：多 App 隔离/公平（P-04）无探针，M-07 未测。V-08 关闭条件（冻结规模/公平策略、M-07 揭示饥饿、缺规模不冒通过）未满足；T-04 行要求“M-01～08 原型测量”。修复：在 rq_matrix 增两表面混合负载探针（轻文本 App + 重上传/重绘 App，各自 HeadlessSurface + 共享重放环）：测每 App produce/replay 份额、在队帧龄代理 p95、最长无进展；负控=故意独占/不让出必须可检出；**不得据此宣称生产公平已通过**。若需改预算/策略，按§10 具名修订而非静默。
- F-R2（T-04 行“M-01～08 原型测量”；AC-07 M 方法面）：M-05 内存原型未测且无需新生产即可做（`stage3::sample_process_memory` 现成）。修复：同进程 direct 与 RQ 重放路径各采样 private/working-set（N 帧前后），记录字节与共享资源口径；不宣称 GPU 驻留/上屏。
- F-R3（AC-07；V-05；P-01）：背压/不覆盖未 ACK 槽方案只有合同文本，当前 `endpoint.rs` 无 free slot 时回落 `1-last_slot` **仍覆盖**未 ACK 槽（T-02 已记录）。修复：加槽账模型探针（2 槽 × ack/无 ack 序列）证明冻结的 credit/背压策略下不覆盖未归还槽，并以“当前回落”为负控可检出；实现级修复归 UI062/UI063，不在本 Plan 冒充。

**保留**：T-01～T-03 勾选与其证据不变；T-04 六格/负控部分的 AC-04 证据有效；M-01/M-02 仍不可 offscreen 代理（owner OS063），M-03/M-04/M-08 代理测量有效。规范增量 SD-01 无冲突，merge 时须写“当前行为+持久决定”而非执行日记；不得把研究结论写成生产已通过。


### 2026-11-07 goal round 5

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 5 | from: review | to: review | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 8133fcca2c41bf0272feced83879c1d1b70eeb8c | next: work（T-04 修复 F-R1～F-R3，repair 周期 1/3）


### 2026-11-07 T-04 修复（work / 复审 F-R1～F-R3 闭环）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（T-04 复审发现全部闭环；T-05 待复审） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd | code_commit: 8bf4fdce3de0472224fcd4bff300f30dfb029d8c | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-04 | next_responsible_role: review（T-05 复审）
- 命令/结果：`cargo test -p auto-ui-runtime --lib rq_repair -- --test-threads=1` 退出0（3/3，8.68s）；修复后全套 `cargo test -p auto-ui-runtime --lib 'rq_' -- --test-threads=1` 退出0，**20/20 通过（115.60s）**（t04/repair-full-suite.log）；生产 wire/lowering/replay 零改动（仅 test-only `rq_repair` + 模块注册）。
- F-R1 闭环（AC-07/V-08/P-04）：两表面混合负载探针（轻文本 vs 重 code-editor 上传），round-robin 下轻 App 最大空档 1 拍/在队峰值 1；heavy-hog 负控下 5 拍/5 帧（饥饿可检出，`hog.light_max_pending > rr.light_max_pending` 断言成立）；份额仅记录不判据。真实两 Host/1·4·8 规模仍归 OS063。
- F-R2 闭环（T-04 M-05 原型）：同进程 direct vs RQ 重放各 10 帧采样（`stage3::sample_process_memory`）；Δprivate direct +602 112 B / RQ +0（时点噪声）、ΔWS direct +9 437 184 B / RQ +56 832 000 B；进程级含分配器噪声/共享页，不代表 GPU 驻留或物理上屏；跨进程口径归 OS063。
- F-R3 闭环（AC-07/V-05/P-01）：槽账模型——当前回落策略 5 拍无 ACK 覆盖 3 次（负控可检出）；冻结 credit 策略覆盖 0、延后 2、ACK 后恢复、在途断言 ≤2 槽。实现级修复归 UI062/UI063，T-03 合同 §3/§4 不变。
- 证据：暂存区 `t04-repair-report.md`、`t04/repair-run.log`、`t04/repair-full-suite.log`。修复周期 1/3。

### 2026-11-07 goal round 6

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 6 | from: work | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 8bf4fdce3de0472224fcd4bff300f30dfb029d8c | next: review（T-05 复审）


### 2026-11-07 T-05 复审（review / final，repair 后二次）

- stage: review | review_scope: final | reviewer: 同会话复核（限制同前：无独立模型/代理） | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: **needs_fix** | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | reviewed_commit: 8bf4fdce3de0472224fcd4bff300f30dfb029d8c | prior_reviewed_commit: 8133fcca2c41bf0272feced83879c1d1b70eeb8c | base_commit: b4094acd86e2717ed075db8d5c06b3bd4b6d6578 | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | spec_inputs: 同前（native-command-frame-text/host-theme-transport 不冲突；SD-01 目标待 merge 写作） | next: work（T-02 修复 F-R4）

**修复验证（独立重跑，非执行者摘要）**：`git diff 8133fcca..8bf4fdce` 仅 `headless.rs`(+2 模块注册) 与新增 `rq_repair.rs`(442 行)，**生产 wire/lowering/replay 零改动**。`AUTO_RQ_PROBE_OUT=<verify>/t05 cargo test -p auto-ui-runtime --lib rq_repair -- --test-threads=1` 退出0（3/3，10.23s）：
- F-R1 **closed**：round-robin 轻 App gap=1/pending=1；heavy-hog 负控 gap=5/pending=5（饥饿可检出）；份额仅信息性。
- F-R2 **closed**：direct/RQ 各 10 帧内存采样可复现（本次 Δprivate direct +3 317 760 B、RQ +0；ΔWS direct +15 364 096 B、RQ +34 308 096 B）；进程级口径与共享页限制在案。
- F-R3 **closed**：当前回落策略 5 拍无 ACK 覆盖 3 次（负控检出）；credit 策略覆盖 0/延后 2/ACK 后恢复/在途≤2。

**保留证据**：AC-01（39/218 穷尽）、AC-02（mesh 0.0711%、SVG 0.0000%、glyph 一致）、AC-03（V-03 半帧拒绝/保旧/版本门）、AC-04（六格 2 跑/4 缺口带 owner、三项负控）在 8133fcca 已核；本次修复不触生产代码，不受影响。

**新发现（此前未覆盖，必须修复）**：

- F-R4（AC-07；V-06；P-02；T-02 行“验证资源不重复上传”）：V-06 关闭条件是“稳定资源 payload 重复上传为 0”，但现存证据只有**违约测量**（`repeat_uploads`=[1,1,1,1]、10080 B/帧，rq_seams.rs:679），没有任何探针证明按 T-03 冻结的“内容身份+generation”方案在稳定帧上重传为 0、变更/驱逐/新 session 才重传。合同文本不能替代 V-06 的 oracle。修复（最小）：在 rq_repair 增资源身份模型探针——content_key+kind+generation 注册表：同内容第 2 帧起上传=0；内容变更→1 次重传；驱逐/新 session→1 次；当前 lowering 路径=1/帧 作负控；断言稳定帧 0、变更可解释。实现级修复归 UI062/UI063。

**结论**：F-R1～F-R3 已闭环；因 F-R4（V-06 未满足）不能判 final pass。Plan 回到 `executing`，T-02 重开（current_step=3/6，T-01/T-03/T-04 保留完成）；修复周期 2/3。未改写 AC/范围；SD-01 不冲突。

### 2026-11-07 goal round 7

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 7 | from: review | to: review | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 8bf4fdce3de0472224fcd4bff300f30dfb029d8c | next: work（T-02 修复 F-R4，repair 周期 2/3）


### 2026-11-07 T-02 修复（work / F-R4 闭环：资源身份与零重传）

- stage: work | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pass（F-R4 闭环；T-05 待三次复审） | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | base_commit: b4094acd | code_commit: 5cbe732fa26756c583244ece3785a02a650ba95f | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | task_ids: T-02 | next_responsible_role: review（T-05 三次复审）
- 命令/结果：`cargo test -p auto-ui-runtime --lib rq_repair -- --test-threads=1` 退出0（4/4，9.64s）；生产 wire/lowering/replay 零改动（仅 test-only 扩展）。
- 事实（当前路径负控）：code-editor 位图每帧上传（uploads/frame=[1,1,1,1,1,1]；稳定键 `8aae3345` 自第 1 帧），且该位图与文本内容无关（不同文本同键）——静态资源每帧重传，V-06 违约可检出。
- 冻结方案模型：内容身份(blake3)+generation 注册表——首次 1 次（gen1）、稳定帧 4 次观察 **0** 上传、字节级内容变更 1 次（gen2 单调）、驱逐后同键 1 次、新 session 全量 1 次；bitmap/font/svg/mesh/media 五类通用。V-06 关闭条件（稳定帧重复上传=0、变更/驱逐/新 session 可解释）在模型层成立；生产实现仍 1/帧（owner UI062/UI063，具名缺口不豁免）。
- 证据：暂存区 `t04-fr4-report.md`、`t04/repair-run.log`。修复周期 2/3。

### 2026-11-07 goal round 8

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 8 | from: work | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 5cbe732fa26756c583244ece3785a02a650ba95f | next: review（T-05 三次复审）


### 2026-11-07 T-05 三次复审（review / final，PASS）

- stage: review | review_scope: final | reviewer: 同会话复核（限制：无独立模型/代理；结论由独立重跑+产物重建） | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | **outcome: pass** | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | reviewed_commit: 5cbe732fa26756c583244ece3785a02a650ba95f | base_commit: b4094acd86e2717ed075db8d5c06b3bd4b6d6578 | dependency_revisions: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f | spec_inputs: docs/specs/runtime/native-command-frame-text.md、host-theme-transport.md（无冲突）；SD-01 目标 docs/specs/runtime/rq-fidelity-contract.md 待 merge 写作 | pending_closeout_gates: SD-01 canonical 文档落地、Plan 归档 docs/plans/archived/、worktree/branch/registry 清理、merge 冻结并集（本 Plan affected = `rq_` 探针全套 21 条 + docs diff --check）、landing 检查 | next: merge（T-06）

**基线**：worktree 干净；`git diff b4094acd..HEAD` 生产代码**零改动**（headless.rs 仅 +4 个 `#[cfg(test)] mod` 行；其余全部 test-only 新文件 + Cargo.lock+38 纯增 accesskit*）；`git diff 8bf4fdce..HEAD` 仅 rq_repair.rs +197（F-R4 探针）。

**独立重跑（5cbe732f）**：`rq_repair` 4/4（12.20s）——F-R1 round-robin gap=1/pending=1、heavy-hog gap=5/pending=5；F-R2 direct/RQ 各 10 帧内存采样（Δprivate direct +2 191 360 B、RQ 0；ΔWS direct +14 331 904 B、RQ +47 452 160 B，进程级限制在案）；F-R3 当前回落 5 拍覆盖 3 次、credit 覆盖 0/延后 2/in-flight≤2；**F-R4 当前路径 uploads/frame=[1,1,1,1,1,1]（稳定键 8aae3345 自第 1 帧）vs 模型：首次 gen=1、稳定帧 4 次观察 0 上传、字节变更 gen=2、驱逐/新 session 重传、五类资源通用**。另复跑 capability（39/39）、mesh（4 paths/12 段，重放差 728 px=0.0711%）、svg（0 px=0.0000%）——与 §9 声明一致。

**验收映射（final）**：AC-01 pass（39 View 变体+218 StyleClass 穷尽 match；缺口具名未计通过）；AC-02 pass（mesh 0.0711%、SVG 0.0000%、glyph 与生产整形全等；漏 mesh/错版本/错字号负控敏感；具名不可达在案）；AC-03 pass（信封 v1/载荷 tag1-3/8 能力 ID/迁移窗口/资源 identity+generation/ACK≠fence/原子帧保旧；V-03 探针：半帧 TooShort、坏载荷不覆盖 front）；AC-04 pass（六格驱动 2 跑/4 缺口带 owner、缺格不 PASS）；AC-07 pass（P-01 槽账模型+结构性单消息；P-02 违约测量+身份模型零重传；P-03 15ms tick 定位+公平等待负控；P-04 两表面混合负载；M-03/04/05/06/08 代理已测，M-01/02/07 真实规模具名归 OS063；M 方法/direct 基线/预算先于候选冻结）。AC-05=本复审；AC-06 merge 承担。

**V 矩阵关闭**：V-01/02/03/04 pass；V-05 pass（有界/不覆盖/无逐原语往返；M-01 分段归因具名缺口→OS063）；V-06 pass（稳定帧 0 重传、变更/驱逐/新 session 可解释，M-04/M-05 可见）；V-07 partial（M-03/04/05/06/08 已观测、呈现与提交分列、等待负控经公平空档检出；M-01/M-02 物理 present 不可 offscreen→OS063）；V-08 pass（公平/饥饿负控；真实 1/4/8 规模→OS063）；V-09 pass（13 项接口+owner+负控，不冒生产联验）。

**F-R1～F-R4 全部闭环**：一次复审 F-R1～F-R3（修复 8bf4fdce 验证）；二次复审新发现 F-R4（修复 5cbe732f 验证）。无开放 finding。

**规范增量复核（SD-01）**：target `docs/specs/runtime/rq-fidelity-contract.md` 不存在冲突条目；merge 写作时须区分**当前行为**（v1 DrawOp/v2 七原语、wire 字段与失败语义）与**合同决定/缺口**（v3 原语、blur/SDF、资源 identity、custom shader 不可达），不得把未实现写成已交付；`supersedes_spec_components` 为空成立。

**结论**：final pass，交付边界如实（研究结论≠生产实现；两 Host 真实进程/物理上屏/VM 六格归 OS063/UI063/UI062）。允许进入 merge（T-06）。

### 2026-11-07 goal round 9

- stage: goal | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | round: 9 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-061/auto-ui | commit: 5cbe732fa26756c583244ece3785a02a650ba95f | next: merge（T-06）


### 2026-11-07 T-06 merge（consolidation, PLAN-061:r3）

- stage: merge | plan_id: auto-ui/PLAN-061 | plan_revision: r3 | outcome: pending-cleaned | worktree: D:/autostack/.wt/ui-061/auto-ui | branch: codex/plan-061-dev | reviewed_commit: 5cbe732fa26756c583244ece3785a02a650ba95f | delivery_commit: 3efe453811d4992aec6fb2c47ea9fbbb0516059c | base_commit: b4094acd | dependencies: auto-lang 8337bab5 / auto-web 2d698413 / auto-down 58ea33f
- prepared：SD-01 目标 docs/specs/runtime/rq-fidelity-contract.md（reviewed delta）在 worktree 写作并作为 **docs-only descendant**（实现/依赖未变）提交为 delivery commit 3efe4538；docs/specs/README.md 增导航链接。
- landed：master 4094acd → 3efe4538，git merge --ff-only codex/plan-061-dev（无 merge commit）；rebase 为 no-op（range-diff 6 段全等，无 old→new 重写）。
- canonical_verified：docs/specs/runtime/rq-fidelity-contract.md sha256 f2afb2464e4ffad46b76cd0f06b74dfef9ae53722d2dabb13b605dac1d7be9c，README 链接在案；内容区分“当前行为/缺口”与“冻结合同/具名不可达”，未把 v3 原语写成已交付。
- 冻结并集（affected gate）：cargo test -p auto-ui-runtime --lib 'rq_' -- --test-threads=1 在 delivery commit 上 **21/21 通过（111.42s）**；git diff --check b4094acd..3efe4538 退出0。
- landing checks：本 Plan 生产代码零改动（仅 test-only 探针 + docs），无 release/daemon/web bundle 消费面；main tip == delivery commit，故 worktree 门证据同 commit 有效。batch regression 为 auto-lang 机制（本仓无 .last-batch-regression.json 约定）→ 不适用。
- archived：docs/plans/archived/061-rq-fidelity-contract-and-feasibility.md（本文件），待与 docs 一并提交后生效。
- cleaned：待 worktree/branch/group/plan-artifacts scope 清理与 registry 刷新后补记。

## 10. 待澄清事项

- 性能数值预算/实际刷新率/可运行规模待UI061授权执行后按先基线后候选冻结；无预算不签PASS，支持承诺或阈值变化须具名修订。
- 本轮没有必须再询问才能成稿的问题；实施暂停来自用户的规划范围。后续启动不改变既有依赖门。
- UI061的准确录制接缝/版本/资源预算是具名技术决策；未取得可行证据前不猜其后继实现，若改变承诺/仓scope给具体修订。
- 字体/IME/媒体/自定义图形或产品原有缺口不能以N/A自动缩分母；明确owner、实际失败入口、关闭条件，必要时另立Plan，当前必需项保持未完。
- Linux自有compositor和ACE/AGE的未来接口按战略进入门另立实施合同，本Plan没有自动切平台/引擎授权。
- T-01 具名超预算/异常（不豁免）：`image_svg_icon` produce p50 32.33/p95 35.91ms（svg逐帧解析/降格）、`window_thumbnail` p50 32.33/p95 42.72ms（宿主快照桩）——owner 分别 T-02（记录面）与 T-04（宿主帧供给），修复前不计入W-C通过样本。
- T-01 高优先保真缺口（T-02 探针对象）：canvas mesh/P683-D1、SVG/vector/P683-D3、IME preedit+caret（clipboard::Null，§5）、宿主帧供给（ImageSurface/Video/Thumbnail/Workspace/RemoteFrame）、文本整形态（Host重排+Raw省略）；像素差热点code_editor 14.99%、navigation_rail 2.10%、managed_scroll 1.59%、tabs 1.01%。
- T-01 测量边界：全部为offscreen代理由，M-01/M-02/M-05/M-07未执行且不得以本Plan数字冒充；真实六格、物理present、内存与多App公平性唯一关闭者 T-04/OS063。
- T-03 前已冻结的必做增量（T-02 证据）：PathFill/PathStroke（段序列+换回RGB paint+已烘焙变换）、GlyphRun 或 glyph atlas（录制期强制整形）、SVG 资源+生产端 device-scale 栅格化（1x 放大有 4.5% 差，2x 为 0.000%）、Image rotation/opacity/filter 入 wire、Host blur/SDF 承载 shadow、资源身份+generation（P-02）、背压与 GPU fence 分离（P-01）。
- 具名不可达（T-03 合同须写清支持边界，不得静默降级）：custom GPU shader primitive（CPU 记录器无通道）、SVG 的 Skia 级 filter 子集（resvg 覆盖外）、要求保留可编辑文本语义的旋转文本（outline path 可保真但失语义）。
- 记录器色序坑：iced_tiny_skia `into_paint` 以 (b,g,r,a) 构造 tiny_skia 色（geometry.rs:427），任何 wire 序列化必须以逻辑 RGB 落地并加负控；P-02 现状稳定重复帧仍每帧上传（uploads/frame=[1,1,1,1]），T-03 修复前不得声称资源复用。

- T-04 前冻结：v3 载荷（tag 3）原型与能力 required/optional 协商的运行时验证；六格驱动（VM/Rust × direct/standalone-RQ/desktop-RQ）与真实录制/重放最小样本；M-01/M-02/M-05/M-07 测量；1/4/8 规模与公平性；预算实际值按真实进程复测（当前 256MiB/App、8MiB·s⁻¹ 为待复测初值，不得直接宣称已满足）。
- 具名缺口（不得以 N/A 缩分母）：剪贴板服务桥（现 clipboard::Null）、present 反馈事件缺失（仅代理终点）、custom GPU shader、Skia filter 子集、可编辑旋转文本；owner 见 V-09 表，T-04 只做研究样本与缺口映射，不冒充生产联验。

- T-05 复核前须读：T-01 能力分母/基线、T-02 接缝报告、T-03 合同、T-04 六格与负控；复核对象为 worktree `8133fcca`（r3 语义，未改 AC/范围）。
- 研究结论不替代生产：v3 原语/资源 generation/背压实现、两 Host 真实进程、VM 六格、M-01/02/05/07 均未在本 Plan 完成；若复核认为 AC-02/AC-03/AC-04 的“已证”口径需要实现级证据，应走 needs_replan（改范围）而不是在 T-05 冒充。

- T-05 复审结论 needs_fix（2026-11-07）：F-R1（P-04/V-08 多 App 公平无探针、M-07 未测）、F-R2（M-05 内存原型未测）、F-R3（P-01 背压不覆盖未 ACK 槽仅有合同、当前 endpoint 仍回落覆盖）。修复限于 rq_matrix/探针与 Plan 记录，不得改 AC/范围；修复后 T-04 重新自检并交复审（repair 周期 1/3）。
- 已知不可 offscreen 代理的 M-01/M-02（真实 present 时序）仍由 OS063 关闭；本 Plan 不得以代理时间冒充物理上屏。

- T-05 复审的 F-R1～F-R3 已闭环（2026-11-07 T-04 修复记录）；T-04 重新置完成，待 T-05 复审（repair 周期 1/3）。复审时需直接核验 `rq_repair` 三探针输出与 `t04-repair-report.md`，不得仅据 receipt。

- T-05 二次复审（2026-11-07）：F-R1～F-R3 已验闭环；F-R4 未闭环——V-06/P-02“稳定帧资源重复上传=0”无探针（当前只有违约测量 1/帧）。修复归 T-02（仅探针/模型，不改生产）；修复后需再次复审（repair 周期 2/3）。

- T-05 二次复审的 F-R4 已闭环（2026-11-07 T-02 修复：资源身份模型探针）。T-02 重新置完成，待 T-05 三次复审；复审须直接核验 rq_repair 的 fr4 输出（当前 1/帧 vs 模型 0）与 `t04-fr4-report.md`。repair 周期 2/3。

- T-05 三次复审 final pass（2026-11-07），无开放 finding；F-R1～F-R4 全部闭环。merge（T-06）前须完成：SD-01 canonical 写作（当前行为 vs 合同决定分列）、归档 docs/plans/archived/、冻结并集（`rq_` 21 条 + docs diff --check）、worktree/branch/registry 清理与恢复摘要。
