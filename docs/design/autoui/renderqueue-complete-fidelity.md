# RenderQueue完整功能与直接渲染一致性

2026-11-11 target-state r6/当前验收边界：1003与OS068已正式交付，989 r7按OS068明示修约接收≤60Hz/DWM呈现、M-04机制、P-03异层配对、M-01协议面及Video机制/后端owner移交。严格光子/>60Hz/字节计数/replay p95/原生100ms和完整Video产品后端不冒已交；未被明示修约的原required证据仍核适用性/缺证。当前调度见[子路线v7](renderqueue-refactoring-roadmap.md)与[989 r7](../../../../auto-lang/docs/plans/989-split-refactor-joint-delivery-and-tag-readiness.md)，原更强技术目标保留为历史/后继输入，不再据旧字面自动阻已修约子项。P2与有限升级恢复定义尚阻989 T-02；本轮只规划，不改canonical/实施。

2026-11-10 target-state r5当前：OS066归档仍有AC-02 partial及Demo/产品/最终性能余项，新的[OS068](../../../../auto-os/docs/plans/068-windows-rq-remaining-fixes-and-full-acceptance.md)负责有限修复与完整终验；[语言1003](../../../../auto-lang/docs/plans/1003-generated-debug-policy-c-production.md)负责生产C默认/迁移回退。OS068准备/机制修复可先，最终C同源重建和六格/性能等1003正式交付，之后供989 r6消费；见[子路线v6](renderqueue-refactoring-roadmap.md)。全能力/全Demo/active产品、异步批量/资源复用/帧调度/隔离、M-01～08与真实物理/GPU/高刷原门保持，缺设备保持未完成；不以owner名字或归档计数签完整PASS。本轮只设计/立约，不改canonical或实施。

2026-11-10 target-state r4规划：OS063有限交付后的Windows完整目标由UI077→OS066承接，989 r4仅在治理与完整RQ汇合后作联合总收尾；见[子路线v5](renderqueue-refactoring-roadmap.md)。原目标不降，缺测/缺产品不以归档豁免，本轮未实施。

2026-11-08 target-state r3补充：当前行为以062 r5的canonical为准，其完成不等本设计完整目标已达。后续分工/进入门以[子路线v4](renderqueue-refactoring-roadmap.md)为唯一当前调度：UI063 r4通用运行→UI073 r1连续媒体/精确绘制/资源反例→OS063 r4最终全产品→语言989 r3联合M1。原“完全一致”及P/M目标保持，062局部2%/默认重排/p95只记录不自动下放为最终验收。下面2026-11-05基面属于历史观察，不冒当前源码现状。

2026-11-05，target-state r2。用户要求的AutoUI重构子路线；本轮补充性能合同，未运行验收、未实施。战略与计划见[子路线](renderqueue-refactoring-roadmap.md)。本设计继承remote Renderer方向，纠正旧设计“构造上不可能漂移”的过度结论，不重开历史有限交付。

## 1. 目标与三种Host

同一AutoUI应用无论由VM执行还是编译成Rust，均能以RQ运行：App拥有业务状态、组件、布局、命中与事件处理，输出渲染命令；RQHost拥有GPU/device、目标表面与最终栅格化/合成/呈现。App可做CPU布局、文字整形、图片解码；不得用隐藏窗口、自有GPU或整帧CPU栅格化后传截图冒充RQ。源图片/视频解码帧是资源，不等于App自己画完整UI。

当前后端为iced/wgpu。RQ与direct必须在同一应用状态、视口、字体、主题、DPI、时钟与资源版本下产生相同客户区画面及交互结果。原生标题栏、桌面边框等由Host策略决定，独立验收，不用裁去App内容来隐藏差异。

| Host形态 | 呈现目标与责任 | 当前/未来边界 |
|---|---|---|
| Standalone RQHost | 独立进程、多App原生窗口；由首个App发现/孵化，同用户实例共享 | 当前Windows完善；Linux作为普通Wayland客户端的准入归OS062 |
| Desktop RQHost | AutoOS虚拟桌面的子表面；桌面负责窗口管理、堆叠、裁剪、缩放、焦点和关闭 | 不得用桌面外独立窗顶替嵌载；不得仅用inproc VM顶替VM RQ |
| Linux compositor Host | 同时接受Auto RQ命令和标准Wayland客户端表面，承担seat/output及系统合成器职责 | 未来独立实施；不是在COSMIC窗口中启动RQHost就完成 |

三种Host共用协议、资源寿命和重放核心，窗口系统适配分别实现。不要求把整个桌面、应用业务或VM搬进Host。

## 2. 核查基面与真实缺口

本轮观察：UI master `426c1f708e7c966f243c86b1f38029ae47b0fa0a`；OS main `7968a68bfd15853a907f8b21f4dc106a2c216d39`。活动UI053/060仍推进，以下是主检出源码事实，不覆盖其未交付WIP。`docs/specs/README.md`存在，UI无overview.md；当前结构/实例/主题/原生验证Specs有效，但完整RQ保真规范缺席，旧总体设计仍在语言仓。

路径以下以UI仓为根：

| 发现 | 实际路径/符号 | 结论与后继 |
|---|---|---|
| 两种生产者已能共享remote路径 | `desktop_protocol/client_entry.rs`：`run_dynamic_client`、`run_native_client`；`headless.rs`：`HeadlessSurface`、`HeadlessFrameSource`、`McpHeadlessSource` | 无需重写两套控件；需要证明两种实际公开入口都走此路径 |
| legacy仍与remote并存 | `native_projector.rs::RqProjector`；`rqhost.rs::run_vm_rqhost_client`由`AUTO_VM_RENDER=remote`选remote，缺省false；`rust_ui.rs`生成的rq/desktop明确`remote:true` | VM的`-q`和Rust的rq不是仅凭名字就同路线；Man `cmd_run.rs`只在显式remote配置时透传。要整理模式选择与迁移，保原direct默认，不静默降级 |
| 组件树相同不保证原语保真 | `headless.rs::drive_and_draw/lower_layers_v2`使用tiny-skia Secondary记录再降格 | 与direct wgpu两种后端的行为、原语覆盖和顺序仍须逐项证明 |
| mesh漏画 | `lower_layers_v2`对`layer.primitives`仅追加P683-D1观察 | canvas/vector/custom可能受影响；不是已经做了位图fallback |
| 文本信息损失 | `lower_text_v2`：Raw省略，Cached对齐用`text_width_approx`，段落取字符串由Host重整形；协议TextStyled不携完整字体/glyph信息 | 中英混排、RTL/连字、回退字体、富文本、编辑preedit须由真实glyph/布局证据核验 |
| SVG/图片变换缺失 | `lower_image_v2`：Vector直接返回，rotation/opacity只记观察，Image仅Stretch | 修复必须覆盖图像资源与呈现参数，不能声明“iced全集自动支持” |
| Host仍近似重放 | `iced/broker_surface.rs::paint_ops_v2/paint_shadow_scrim`：canvas路径重建，shadow为稀释垫层，非轴对齐仅平移，border近似 | 需共用精确原语重放；不能靠放宽截图阈值消债 |
| 两类Host已有落点 | `rqhost.rs::run_daemon/rq_view`（iced daemon）、`session.rs`/`iced/renderer/shell_state.rs`（桌面采纳），OS `host/desktop/src/main.rs`薄装配 | 保留现有交付；抽共享能力，不另造两份渲染器 |
| 跨平台通道尚非完整 | `transport.rs`非Windows为pipe_stub拒绝；`shm.rs`非Windows为进程内Registry | Linux真实UDS/共享映射归OS062，不能用loopback或编译绿认定跨进程可用 |
| 运行功能不由绘图自动获得 | headless用Null clipboard；tick/poll_due_timers与direct订阅面需对账；UI060发现native API stub | 输入、clipboard、异步、服务、init、退出与真实后端都要验；060归原owner，本路线接受正式交付 |

证据锚：headless.rs SHA256 `B0BFEFBDE67439229BB832FE6A2BA2617AD2FD031DDBCF33926AD035D527183D`；broker_surface.rs `B142C010D042BBA3DB7A8C250F5620F75FAE0B426B79F42D4650F03A95A4757E`。这些是静态证据，未做本轮截图/性能/运行测量。

## 3. 目标架构与单源规则

```text
Auto源 → VM / Rust（未来ACE）→ App model/store/生命周期
    → 同一组件、布局、文字与命中实现（当前iced；未来AGE）
    → 已解析、无业务回调的RenderPacket + ResourceOps
       ├─ direct：本进程使用同一ReplayCore → iced/wgpu → 自有窗口
       └─ RQ：编码/传输 → Host校验 → 同一ReplayCore → Host表面
Host平台输入/服务 → session/surface/event身份 → App原组件处理 → 下一帧
```

这是目标依赖结构，RenderPacket/ReplayCore是拟议概念，当前并无这些公开类型。UI061先验证最小可行接缝，不预先把3.5万行适配层泛型化，也不新拆crate。优先保留`into_iced`组件实现，把lossless recorder与重放放在同一iced原语边界。比较两案：修补tiny-skia记录缺口；或在现有iced Renderer适配/受控依赖补丁中录制完整原语。依据实际颜色、文字、mesh、custom shader可达性定案。不能继续手写第二套布局/命中器来“补齐”RQ。

direct路径最终与RQ共享原语消费语义，但迁移前direct画面是兼容oracle。禁止同时简化direct和RQ后互比绿。固定旧direct金样与交互状态；合法原有能力必须保留。受硬件/平台影响的非确定性分开记录，不把缺失图元列为抗锯齿误差。

RQHost只处理协议内的渲染/表面/资源/服务；不运行App业务handler，不反序列化Rust闭包、VM Value或iced指针。测试/MCP可带语义观测与动作，但不得代替物理输入通道的验收。

## 4. 完整覆盖的分母

UI061在冻结源码上列出`View`所有变体、`IntoIcedElement`各分支、style属性、记录原语、平台服务、生成器入口与当前Demo/真实产品需求，给稳定能力ID及producer→wire→consumer→case映射。至少覆盖下列族，实际发现的额外公开能力必须入表：

- 布局/装饰：row/column/grid、约束与viewport单位、滚动/嵌套clip、overlay/popover、z序、transform、透明度、四角半径、border、gradient、shadow、主题与DPI。
- 文本/编辑：字体身份/回退、glyph位置与cluster、富文本、选区/caret、换行/裁剪、CodeEditor/Terminal/AutodownEditor、中文IME与组合串、复制粘贴。
- 资源/图形：Image/ImageSurface/SVG/动画图、Canvas/mesh/custom widget与可达custom GPU primitive、Video/RemoteFrame、thumbnail/workspace preview；纹理更新、尺寸与采样、颜色/alpha、动态失效和释放。
- 交互/运行：键盘/modifier/repeat/Tab、pointer capture/拖拽/wheel/hover、focus/blur、IME候选窗定位、clipboard、文件选择/外部打开等实际存在的窗口服务、可访问性；init、timer、异步/HTTP/SSE、路由/store、theme/effect、关闭与迟到回调。

每项状态为已证/待实现/待验证/平台条件/具名上游阻塞；“未覆盖”不能计入完成。已有direct或Rust生成缺陷须保留直接轨红及owner，不能据其失败自动缩小RQ分母。用户要求完整功能，若某能力必须改变支持承诺，先给具体方案修订，不能自行退成试点交付。UI061研究交付与整条RQ路线完成是两种结论。

## 5. 线协议、资源与帧事务

1. 分离envelope版本、display payload版本、capability/profile。现`PROTOCOL_VERSION=1`、DrawList v1与DisplayList v2的编号不同，不能把其中一个加一当全栈协商完成。建立双端可证明的版本/required capability握手，未知必需命令在入帧前明确拒绝，旧端行为有负控。保留legacy明确入口，迁移后退役另验。
2. 表面身份至少为session epoch + surface id/generation，frame sequence与resource generation关联；帧携logical/physical viewport、scale、damage及资源依赖。输入亦绑定身份/坐标epoch，resize前后不把旧坐标错投新surface。
3. 有序命令保留clip、完整二维transform、blend/opacity和层次顺序；可达mesh/path与纹理采样有精确表示。文字优先携稳定字体内容身份与已整形glyph/cluster/位置，避免Host再次按不同字体/算法排版。精确wire结构和字体实现可行性由UI061决定，不能承诺一个尚未存在的API。
4. ResourceOps明确create/update/release、字节/句柄类型、owner、容量和引用。不得把App文件绝对路径当Host总能读取；包资源、动态位图、字体及媒体须有传输/授权解析合同。图片源数据与整帧截图严格区分。
5. 完整验证资源/长度/乘积溢出/非有限数/栈配对/索引/ID后才提交帧；失败保留旧完整帧并诊断，不呈现半帧。ACK归还传输槽不等于GPU fence完成；资源不得在GPU在用时重用。慢Host有有界背压，可合并过期画面但不能跳过资源创建、释放或输入副作用。
6. 断连/HostLost先停止错误派发并保留业务owner。初版支持可诊断退出和用户重启恢复；透明重连只有实现epoch重建、资源全量重发、焦点/输入去重后才宣称支持。旧文“下一帧即可恢复”不作为证据。生命周期测试不杀旁观进程。
7. IPC用户身份/端点归属及容量校验纳入实际同用户本地profile；不新增网络远程安全承诺。限制由可测资源合同冻结，超限明确失败，不能以预算偷偷禁止既有语法/控件。

## 6. 输入、服务与应用执行

Host采集本地事件，按正确表面/焦点/capture及DPI逆变换回App逻辑坐标；App iced树保有命中、selection与滚动态。跨窗口拖动、按下后移出、遮挡、resize中的旧事件、关闭后迟到事件均须区分性反例。IME不仅传最终文字，还包括preedit/selection/commit/cancel及候选窗位置；Host OS位置与App caret坐标一致。

clipboard、窗口尺寸/标题/光标、文件对话框等平台服务以有owner的请求/结果桥接，保direct语义；不能把Null实现当无差。Rust和VM使用同一服务边界、应用初始化及调度合同；异步结果/timer/动画不能依赖某条路径才有的重绘事件才推进。theme/Store/组件实例沿041/055现行合同，未来056/057只按实际接口消费，不把完整Reload作为RQ前置。

UI060的真实API、输入修复与UI053的解析栈/生成修复按正式交付消费。UI063处理其范围外的完整运行接缝；不重新做同一修复、不把已授权在途计划加上本路线所有验收。

## 7. 一致性与关闭oracle

主矩阵为 **VM/Rust × direct/standalone-RQ/desktop-RQ = 六格**。同一编译产物或同源VM输入、真实入口、版本、内容区尺寸、字体、主题、scale、事件序列和资源固定。Linux同矩阵按平台准入逐步建立，Windows绿不外推；第三类compositor需另建会话级矩阵。

- 结构：逐帧规范化RenderPacket/glyph/资源顺序和布局/命中区域；原语丢失必须失败，不能只数帧。
- 视觉：同机器固定GPU/驱动/字体与时钟，对相同状态的客户区做pixel diff、差异热图和语义区域检查。目标相同输出；先以exact比较，只有经独立确认的采样/抗锯齿非确定性才可设具名窄容差，记录区域/理由/负控，不能用全局SSIM阈值吞掉图元。迁移前direct金样保持。
- 行为：真实键鼠/IME/剪贴板和API/持久层读回；绘图相同但输入、保存或错误处理不同同样失败。注入须走Host事件转发，不直接改model冒成功。
- 所有权：App无GPU设备/渲染表面/隐藏窗/整帧栅格器，Host实际消费帧并呈现；代码路径检查与运行device/window/进程观测同时成立，链接了wgpu库本身不算App在绘制。
- 失败：丢mesh/SVG/字体替换、旧generation、坏资源、慢Host、HostLost、关闭末窗与仍有pending采纳等负控分别可被检测；缺测试工具/运行环境记未执行。

全能力矩阵须闭合，不能以001/003/004或Counter/Notes代表全部；测试语料和真实产品分开计，OS054八应用、OS062两应用原合同不缩减。基准记录帧编码/传输/重放/呈现耗时、队列峰值和App/Host内存，先冻结可用预算，不捏造本轮加速或内存收益。

### 7.1 性能判断与当前证据边界

RQ定位为**异步批量显示列表＋持久资源＋集中GPU执行**，不是逐绘图函数同步RPC。绘制原语本身不等于X11的性能问题；同步往返与重复工作才是需要避免的机制。Xlib的同步调用会引入往返延迟，XCB用异步请求改善这一点，参见[X.Org说明](https://www.x.org/guide/xlib-and-xcb/)。现代X也有直接渲染，不得把全部X11路径描述成逐图元RPC或据架构名宣称更快。

静态已见`endpoint.rs`将整帧编码后写共享槽、管道发送通知，并有FrameAck/BitmapAck；这避免逐原语协议请求，但**共享内存不等于零拷贝**，编码/复制/解码仍要计量。`rqhost.rs::rq_subscription`目前以15ms定时Tick驱动帧泵，属于待验证的等待/节拍风险，不是本轮测得的延迟或对实际FPS上限的结论。文字重整形、canvas再构建和资源反复上传也须逐段计量后归因。

Wayland通常让客户端提交已渲染buffer，而Auto RQ把最终绘制放到Host，不能直接继承其性能结论，见[Wayland官方架构](https://wayland.freedesktop.org/architecture.html)。本路线目标是可核验的开销/响应与资源治理，不预设“比direct快”“比X11快”或固定节省比例；没有同环境X11实测就不作X11速度比较。

### 7.2 性能机制硬要求（P-01～P-04）

| ID | 必须满足的行为 | 区分性验证与责任 |
|---|---|---|
| P-01 异步批量 | 按frame/transaction批量编码提交，允许协议允许的有界分块，禁止每条图元请求/应答；steady-state不要求每帧先等待上一帧呈现ACK才开始准备。ACK管理传输槽/流量，GPU fence管理在用资源；容量耗尽时允许有界背压，不能覆盖未归还槽或无限堆积。显式服务查询与握手可有reply，但不得把其同步等待串进普通绘制热路径 | UI061原型用延迟ACK与合法容量控制识别隐式串行；UI062真实双进程闭合。报告每帧消息数/字节与等待原因，不能只凭API名async判定 |
| P-02 资源复用 | 字体、图片、mesh等按内容身份及generation注册/更新，帧引用资源；在缓存驻留且资源内容未变时，重复帧不得重复上传资源payload或重复GPU创建。失效/驱逐/新session允许可解释重传；不能跨owner泄漏资源。App已完成的文字布局/整形不在Host再次重排。稳定显示片段复用/局部更新先研究收益，保全帧恢复；不强制首版构造复杂全局增量场景 | UI061冻结缓存状态/变更轨迹；UI062对稳态重复帧与单资源修改核payload/创建计数，缓存驱逐、generation变更和最后owner释放为对照；优化不得牺牲保真 |
| P-03 帧调度 | readiness/输入/资源完成及时唤醒，呈现节奏跟随实际目标与可用present反馈；不用固定15ms轮询作为唯一帧驱动。必要timer用于动画deadline或具名平台fallback，不能忙等。每surface在途帧/字节/资源都有上限；过期画面可合并，但必须保留资源依赖和输入/业务副作用，不能将丢帧隐藏在高FPS里 | UI062负责队列/传输背压，UI063负责输入到帧的唤醒与deadline，OS063核两Host实测；注入慢Host/ACK延迟、空闲/恢复与突发输入，观察队列、画面年龄、missed deadline和空闲唤醒 |
| P-04 多App隔离 | 按session/surface计量并限制排队帧、上传字节、驻留资源和CPU批处理工作；Host调度不得让大上传/重场景独占全局锁、事件循环或GPU提交队列。昂贵CPU准备需可分批/调度，轻应用输入与呈现有独立进展。不能承诺任意GPU命令可抢占：大任务须在支持能力/资源预算内拆批、限流或显式拒绝，异常不能静默拖死全部窗口 | UI062资源/工作预算，UI063输入优先与调度隔离，OS063真实混合负载关闭。对重App注入过载/卡顿/断连，观察轻App的p95/p99响应、最长无进展间隔和关闭结果；同App业务状态与输入顺序仍保持 |

输入合并只允许语义明确可合并的事件，例如不跨button/capture边界的连续位置更新；按键、文字提交、按钮转换、服务结果及持久化副作用不得为吞吐任意丢弃。共享Host带来潜在资源收益，也带来集中瓶颈和故障域；不声称进程内各App能完全隔离device loss或Host崩溃。

### 7.3 direct对比方法与指标（M-01～M-08）

对每个执行器分别比较：原direct基面、候选direct、standalone-RQ、desktop-RQ；不拿VM-RQ与Rust-direct混比。仍用六格功能语料，性能选具名工作负载，不取代全功能分母。固定source/build profile、优化级别、GPU/驱动、OS、字体、DPI、客户区分辨率、主题、输入/资源轨迹、显示刷新率、电源模式和后台负载；编译/构建不计作交互时间。保迁移前direct视觉和业务oracle，不能降分辨率/画质/动画/采样来换速度。

| ID | 指标与单位 | 计量/解释要求 |
|---|---|---|
| M-01 响应延迟 | 输入到对应帧呈现ms，p50/p95/p99/max；应用更新、编码、传输/排队、解码、GPU提交/完成、present分段 | 关联event/frame/session ID；定义起点为OS/Host事件接收等真实观测点，跨进程用可比单调时钟并校验偏差。GPU提交或present API返回不是物理上屏；无呈现时间源时标明代理终点，不叫input-to-photon，缺项不能填0 |
| M-02 帧节奏 | 实际呈现帧间隔ms分布、按目标刷新周期的missed deadline比例、有效FPS、合并/丢弃/重复帧数、最旧待呈现帧年龄 | 输入序列和目标相同；不能只报告平均FPS或只保留下成功帧，静态界面不强迫持续出帧 |
| M-03 CPU/GPU成本 | App/Host各自及总CPU时间、GPU时间（平台可取时）、编码/解码/重放时间、空闲唤醒次数/秒 | 分析文字重整形、canvas重复构造/细分、分配和等待；无GPU计时能力明确未测，不以CPU提交耗时替代 |
| M-04 传输/重复工作 | 命令/资源字节每帧及每秒、消息数、同步往返数、复制次数/字节（可观测范围）、资源payload上传/GPU创建次数与缓存命中/失效 | “共享内存”“async”“cache”均不是性能证据；稳定驻留资源与单项变更有明确期望 |
| M-05 内存/容量 | 每App、Host、总private/working-set；共享段单列去重；GPU驻留可测值；队列帧/字节峰值与稳态 | 不只报告App减少而隐藏Host成本，不重复计算共享内存后宣称节省；direct N进程与RQ N App+Host总成本同口径 |
| M-06 启动/恢复 | 冷/热启动到首个可交互帧ms，Host冷孵化/已运行分列，资源重建/恢复到可交互时间及回收终态 | 与运行态成本分开；崩溃重启不冒透明恢复，启动后必须真实输入有效 |
| M-07 多App公平性 | 轻App单独与混合重载下M-01/M-02变化、最长无进展间隔、每App处理份额、队列/资源峰值 | 同时记录重App吞吐/限流/拒绝，不能通过饿死重App制造轻App绿；不要求所有App平均分GPU，采用冻结的交互/后台服务策略 |
| M-08 保真与业务守恒 | 对应状态的视觉/命令/资源断言、处理输入/业务结果数量、错误与资源终态 | 性能样本伴随功能正确性；输入丢失、漏图、未完成任务的“快速样本”无效但失败仍保留 |

工作负载至少含：空闲静态、文字滚动/编辑与IME、动画/resize、图片mesh资源首次及重复使用、局部资源修改、媒体持续更新，以及交互轻App与上传/绘制重App混合。以1/4/8 App作为拟定规模点，UI061先核资源可承受性并冻结实际支持规模；不可运行的规模记缺口，不外推已支持。目标刷新率按真实显示环境冻结，高刷新率能力没有实测不宣称。冷/热、单App/多App、两Host分别报告，不将教学样本当全部产品性能结论。

UI061 T-01先冻结测量定义、输入轨迹、样本长度/重复次数、运行顺序与污染判定；随后测原direct基线，在候选优化/选择结果出来前冻结各workload的绝对与相对预算表（适用M-ID、单位、统计量、样本要求、允许差额、owner及失败处理）。具体数值当前待测，不伪造统一百分比/SLA；这不是已满足性能门。候选前后成对/交错测量，保样本数与原分布，p99样本不足不得下结论。异常样本保留原因，不能择优挑轮次。

后继实现必须满足P-01～04及冻结预算；没有预算或必需测量缺席则不签性能PASS。预算超标先定位/修复，若要改变支持规模、保真承诺或已冻结阈值须具名修订，不能测后放宽。对比表同时报绝对值、相对direct差额及收益/代价；允许如实得到RQ在部分指标更慢的结论，但超验收预算不能归档冒已完善。当前没有任何性能结果。

## 8. ACE与AGE的未来替换

两轴分别演进：执行引擎VM/a2r→ACE；组件/图形引擎当前AutoUI/iced→AGE。wire只含可版本化的渲染/资源/输入/服务数据，不暴露任一引擎对象或Rust ABI。ACE与AGE可分别接入，禁止绑成一次全栈默认切换。

AGE可以提供新的producer和Host replay后端；共用surface/session/resource/输入控制平面，不强迫AGE全部3D效果降成旧2D DisplayList。未来3D以版本化图形包/资源/材质能力扩展，App仍不持有GPU；Host只执行受支持的图形数据，不能接受任意App原生代码/闭包。shader/device capability、资源隔离和同步另立可验合同，当前不虚构AGE production wire。

迁移顺序：同一能力语料→ACE旧UI producer对照→AGE producer/replayer对照→两者组合→真实产品迁移→默认切换/旧路径退役。可并存版本与回退必须有实际闭包。沿[AGE M4](../autographics/m4-autoui-integration-roadmap.md)的Schema成熟及旧编译器替换实际启动门；本轮不提前实施M4，也不让当前RQ完善等待ACE/AGE全部完成。

## 9. 来源与文档职责

- 当前实现：上述UI runtime/tools源码、Man `crates/auto/src/cmd_run.rs`、OS `host/desktop`；结构/组件/主题/原生验证以UI `docs/specs/`为准。
- 历史方向：[RQ remote设计](../../../../auto-lang/docs/design/autoui/rq-remote-renderer.md)、[桌面协议](../../../../auto-lang/docs/design/autoui/desktop-protocol-v1.md)。其已实施/全集/无漂移措辞只能按真实有限证据理解。
- 未来Linux：[AutoOS Host设计](../../../../auto-os/docs/design/04-rqhost-platform-and-compositor.md)。标准Wayland客户端提交buffer而非Auto RQ命令，依据[Wayland官方架构](https://wayland.freedesktop.org/architecture.html)；两种入口须在compositor侧汇合，不能称RQ协议本身即Wayland。
- 本页是目标设计，不覆盖canonical current-state；各计划review后按真实交付新增Specs，不写legacy specs.json。

范围与接口交付的r3复核见[子路线§7](renderqueue-refactoring-roadmap.md)：UI062绘图fixture六格、UI063完整运行六格、OS063产品组合分层，避免反向依赖；UI060原修复和OS062 Linux职责保持。设计目标/能力分母不变，四份执行草案现r3。
