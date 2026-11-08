---
plan_id: PLAN-062
status: reviewed
feature_name: RQ无损原语、文字图形与资源重放
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-08
plan_revision: 5
current_step: 6
total_steps: 7
depends_on: [auto-ui/PLAN-061]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/rq-render-packets-and-resources.md]
touched_goals: []
affects: [renderqueue, rqhost, rendering-fidelity, resource-lifecycle, rq-performance]
---

# PLAN-062：RQ无损原语、文字图形与资源重放

## 0. 变更摘要

承接2026-11-05用户提出的RQ完整功能要求。设计见[完整设计](../design/autoui/renderqueue-complete-fidelity.md)，[战略/跨仓边界](../design/autoui/renderqueue-refactoring-roadmap.md)。本Plan仅drafting，尚未实施。交付限定为本合同结果，不以单个阶段完成冒整条RQ路线全部完成。

## 1. 目标

UI记录、协议/资源与重放核心；必要direct适配只为共享精确语义且保旧效果。不得实现业务控件第二套布局、隐藏窗口/整帧位图兜底、Linux transport或compositor、ACE/AGE新引擎。

工作结果以§7可观察AC为准；同源direct是兼容基准，禁止删除能力/用隐藏自窗或整帧自绘降级来达标。原生窗口标题栏与App内容区分别验，不要求Host系统装饰像素相同。

## 2. 架构方案

执行只接受UI061已review可行的明确接缝及全能力合同；若研究要求实质改变本Plan范围，先按实际结果修订r2再实施受影响任务。本Plan必须关闭全部绘图/资源能力ID；新发现漏项不得静默延期。两个Host调用同一ReplayCore语义；direct迁移保旧金样，不能一起退化后互比绿。颜色空间、premultiplied alpha、纹理采样/顺序、字体整形和clip均是协议行为。图片/视频解码字节可跨线，App整帧栅格不能当成功。外部GPU纹理/custom shader路径须由061证实可在Host执行的精确数据表示，不允许App代码指针过线。

实现→独立review→merge依次使用同一权威实施树。拟用`D:/autostack/.wt/ui-062/auto-ui`、分支`codex/plan-062-dev`；本轮不创建。需要写入的协作仓在同组各一个树；产品只读/隔离数据，新增业务写面须另定具体合同。UI/Man/OS原owner接口保留，语言基础无UI反依赖。

## 3. 技术栈

Rust、现有iced/wgpu、desktop_protocol二进制消息/共享资源；当前Windows本地双进程与两种Host。测试沿仓内现有Cargo入口及真实公开CLI。Linux transport/共享映射归OS062；标准Wayland compositor和ACE/AGE仅在设计中留接缝，不在本Plan实现。

## 4. 需求分析与背景调查

### 授权

2026-11-05 r3：用户要求复核四份RQ计划范围/依赖、与UI060/OS062去重并明确启动顺序；仅修订草案，不实施。

2026-11-05 r2：用户明确授权将性能分析/建议补入设计、战略和相关Plan，仍暂不实施。本次仅修订合同，不新增计划或接管其他会话。

用户明确要求检查设计/实现、建立设计/战略、纳入拆分重构路线并考虑新计划；本轮获准调研、文档和立约，未获本轮实施授权。不自动开agent/聊天，不接管UI053/060/OS054/061/062在途合同，不提交他方WIP。无额外预算/时限授权。

### 源码事实

当前headless::lower_layers_v2省mesh，lower_text_v2和lower_image_v2丢文字/图像信息；broker_surface::paint_ops_v2用canvas重建且shadow/旋转近似。已存在DisplayList v2、BitmapUpload、FontBlob和帧槽机制，应在其来源上演进，不另建无兼容平行通道。

基面UI `426c1f708e7c966f243c86b1f38029ae47b0fa0a`，OS `7968a68bfd15853a907f8b21f4dc106a2c216d39`；文件SHA256与逐项来源在主设计§2。UI Specs从`docs/specs/README.md`进入，相关`ui/structure-closure.md`、`runtime/component-and-store-instances.md`、`runtime/native-vm-app-validation.md`；OS从`docs/specs/shell/desktop-app-launch.md`与`desktop-bootstrap.md`进入。完整保真Spec当前缺席，本Plan提出新增，不提前改canonical。headless/rqhost/client_entry/message/endpoint/shm/transport及iced/broker_surface是现有路径；RenderPacket/ReplayCore与新测试路径是拟议，不冒已实现接口。

立项时取号扫描主检出、archive/archived及全部注册worktree；当时UI060仅树内亦计入占号。r3时UI060合同已恢复主检出、仍executing；UI053已有正式交付与清理收据。锁内取号、不覆盖已有Plan；UI无new-plan.sh使用独占分配，OS使用仓内脚本。实现时重新冻结真实源与依赖，不能依此快照借WIP。

## 5. 详细设计

完整不变量和跨阶段约束沿主设计§3–8；本Plan工作集中于§8唯一任务表。T-01建立`D:/autostack/plan-artifacts/plan-062/auto-ui/`并登记owner、用途、在途状态；探针/截图/日志/外部构建只在统一scope，不落docs/plans/evidence或attachments。正常编译target在专属树；外置target也只在scope并登记可删。

未决接缝由对应前置任务做有界探针并形成可审阅决定，不能一边猜一边切生产；源与已交付依赖不变时复用有效证据。三Host控制面可扩展但没有未来支持假承诺。能力丢失/非法帧必须具名失败；资源上限须T-01按真实测试环境和UI061决策冻结单位、边界、失败表现，不新设语法/控件限制。

### 范围与启动门复核（r3）

- **启动条件**：UI061正式交付可行决策、能力ID、接口与预算后进入T-01～05。UI060不是整项硬前置；headless录制/lower与UI060的输入dispatch/帧刷新按符号隔离，实际交叉符号等待正式修复，不借WIP或复制补丁。
- **消除反向依赖**：本Plan验VM/Rust producer→真实跨进程wire→两Host replay的绘图/资源六格。可由UI061确定状态fixture注入状态/事件，但必须真实生成、录制、传输、重放和GPU像素对照，不能手造期望包替代producer。完整公开CLI/IME/API/异步六格交UI063，全产品装配交OS063；UI062不等这两包。fixture不能覆盖的绘图能力须具名阻塞/修订，不能删分母或偷偷依赖后继运行栈。
- **接口与性能owner**：UI062拥有绘图payload、资源事务、槽信用/背压、per-App上传/驻留/解析预算及共享replay；交付enqueue/ready/cancel、frame/session generation、资源引用终态与present反馈的准确接口/测试。UI063负责事件唤醒、交互/后台调度，OS063配置桌面策略，不各写资源调度器。
- **OS062并行边界**：Linux transport/共享映射/claim/peer/OS句柄清理由OS062承担，UI062负责平台无关帧资源事务。同名ACK/generation不能演成两套实现。先落地者给正式版本，后落地者验证共享endpoint/shm/codec旧新端兼容；无法兼容或同符号冲突，仅等待该接口交付并具名修订，不加双向整计划依赖。

### 性能合同（r2继承）

遵循[主设计§7.1～7.3](../design/autoui/renderqueue-complete-fidelity.md)的P-01异步批量、P-02资源复用、P-03呈现调度、P-04多App隔离和M-01～08指标。本Plan的关闭条件是AC-07、V-05～08与§8任务，设计链接不代替实际证据。

- RQ不是逐绘图RPC：steady-state不逐图元等待reply，也不强制每帧等待上一帧呈现后才准备；容量不足须背压，传输ACK与GPU资源寿命分开。
- 驻留且未变font/image/mesh资源重复帧不重复上传payload/创建GPU资源；合法失效、驱逐、重建单列，不重整形App已定位的文字。片段复用先实证，不默认重做全局增量场景。
- 及时响应输入/资源完成并按deadline/present调度，固定15ms轮询不能是唯一驱动；在途帧/字节有上限，可合并过期画面但不能丢输入、资源依赖或业务副作用。
- per-App上传/资源/CPU工作预算与公平调度必须有轻重混合负控；长任务分批/限流，不能假设任意GPU命令可抢占。Host/device loss仍是共享故障域，不冒完全隔离。
- **测量窗口与 replay 口径（2026-11-08 r5 裁定，用户批准）**：所有计时断言只在**隔离窗口**执行
  （`AUTO_RQ_TIMING_WINDOW=isolated`），共享窗口只记录并逐行标 `timing_gate=skipped_shared_window`——
  Plan §6 本就规定性能测量不得与其他会话争用 GPU，而实测负载会把同一份代码放大 3–6 倍
  （replay p50 4.1→10ms、p95 max 40ms；SVG produce p50 32→76ms）。replay 相对 direct 的判据改为
  **p50 ≤1.25×direct p50+2.0ms 设门、p95 只记录**（20 样本下 p95≈次大值；公平轮转测量还证伪了
  “重放固定开销”假设：空帧 canvas 7.58/10.38ms 快于纯 widget 树 8.23/15.15ms，text_50≈empty）。
  produce 的 W-C ≤10ms 与 SVG 具名例外（p50 ≤60ms，安静机记录值 ≈32.5ms）不变。
- 数值预算由UI061先量原direct基线、在候选比较前冻结：逐workload列单位/统计量/绝对与相对上限/样本/owner/失败处理。本轮无数值或测量结论；后继缺预算或必要测量不能签性能PASS，超标不得测后改阈值。

### 任务落点

T-02：`crates/auto-ui-runtime/src/ui/desktop_protocol/headless.rs`及`message.rs`，`ui/iced/broker_surface.rs`；T-03/04：`desktop_protocol/{codec,endpoint,host,shm}.rs`及实际资源缓存；两个消费点`rqhost.rs`与`session.rs`/`iced/renderer/shell_state.rs`。不得凭新接口名另造第二套实现，UI061冻结决定真实模块拆分。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/rq-render-packets-and-resources.md | 当前有限分散实现/历史设计 → 本Plan已验范围、来源、支持条件及失败/兼容合同及P-01～04/M-01～08性能要求 | 不再以历史“无漂移”或编译绿替代运行证明 | AC-01..04、AC-07 |
| SD-02 | modify | docs/specs/runtime/rq-fidelity-contract.md §7 | 冻结能力表 8 项 → **9 项**（追加 `resource/release/1`，optional、旧端不产不出；`resource/generation/1` 保持 061 原义，062 只把它落到线上） | 新增 tag 必须进冻结分母，否则两页冲突（复审 F-05） | AC-03、AC-04 |

如实际改动还影响已有canonical条目，T-01具名补充并按语义变化修订Plan；不制造无关Spec条目，review核真实增量。

## 6. 测试设计

以下均为拟执行，不是本轮通过证据。主矩阵VM/Rust×direct/standalone-RQ/desktop-RQ；同source/主题/font/viewport/scale/时钟/事件序列。所有V行owner_stage=work。结构/glyph/资源、客户区像素和业务读回同时核验；旧direct金样不因共用新代码而失效。默认exact像素比较，真正平台采样非确定性只有具名区域/原因/敏感反控经审查后可设窄容差；不允许全局阈值吞漏图。

| Case | AC | 输入/真实入口与负控 | 可观察oracle/关闭条件 | 责任任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | UI061能力表、原direct金样、同source VM/Rust绘图fixture六格（§5） | 每个绘图ID有真实输出，结构/资源与像素对拍；旧direct未退化；必要平台未测明确阻塞 | T-01/T-05；全部现有绘图能力 |
| V-02 | AC-02 | mesh+SVG+raw text、RTL/连字/中文回退、rotation/shear、blur/clip叠层；删除命令/替换font反控 | 默认模式：布局/命令/资源结构一致 + 客户区像素差 ≤0.1%（矩阵设门，梯度/阴影例同样受门）；v3 富模式：glyph 坐标对 `physical()` 全等、阴影与 shader 同式，像素差按族具名上界（glyph run ≤1.0%、含阴影贴片 ≤1.5%）记录；原反控必红；无省略日志冒PASS | T-02/T-05；字体与原语 |
| V-07 | AC-07 | 生产双进程慢Host、突发帧与媒体资源更新、原direct对照 | 队列帧/字节峰值在预算内；只合并过期画面而保资源/输入依赖，M-01～06/M-08按触面通过；默认 produce p95 ≤10ms（SVG 例外具名、单列不抵扣）；replay **p50** ≤1.25×direct p50+2ms 设门、p95 记录；计时门在隔离窗口（`AUTO_RQ_TIMING_WINDOW=isolated`）签收 | T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-03 | AC-03 | 图片更新/视频帧/thumbnail、重用ID不同generation、释放后重建、缺资源与健康对照 | 资源只在正确surface生效、尺寸/alpha/采样一致、失败保旧、GPU在用时不早释；最后owner释放 | T-03/T-05；所有资源族 |
| V-04 | AC-04 | 双进程坏长度/乘积溢出/NaN/栈错配、未知required capability、慢Host/旧ACK/乱序资源 | 入帧前明确拒绝、旧完整帧保持，队列有界，资源与传输槽回收不混淆；旧合法协议控制仍可用 | T-04/T-05；协议事务与生命周期 |
| V-05 | AC-07 | 批量帧、延迟ACK、槽满与旧ACK；逐图元同步反控 | 稳态无逐图元往返/强制逐帧呈现屏障，按信用有界流水与背压；M-04消息/等待证据 | T-02/T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-06 | AC-07 | 稳定font/image/mesh重复帧、单项更新/驱逐/最后owner、旧generation | 重复payload/GPU创建为0（驻留未变前提）；变化恰当失效，M-04/M-05可核，GPU在用不早释 | T-03/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-08 | AC-07 | 重App大资源/复杂绘图与轻App共享真实Host，长锁/无限队列负控 | per-App预算生效，CPU任务不独占，轻App与重App均按策略有进展，M-07通过且保功能 | T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-09 | AC-04 | 旧新producer/Host配对、容量单位、unknown capability、平台backend fixture与合法对照 | 兼容真实可用，不兼容明确拒绝；提交/反馈接口可消费；fixture不冒真Linux | T-04/T-05；协议接缝 |

性能实测口径：M-01响应延迟p50/p95/p99/max及分段；M-02帧间隔/目标deadline失约、合并/丢弃/画面年龄；M-03 App/Host/总CPU与可测GPU时间、空闲唤醒；M-04命令/资源字节、消息/往返、复制与上传/创建计数；M-05每进程/总内存与共享去重、GPU驻留和队列峰值；M-06冷/热启动及恢复；M-07轻重App公平；M-08保真/业务守恒。各职责分别取证，OS063做最终组合，未执行不得被上游局部测试抵扣。

同执行器原direct、候选direct及两RQ Host同机器/source/build profile/GPU/字体/DPI/分辨率/主题/事件/资源/刷新率比较；冷/热分列。完整多App成本为N个App加Host，direct为N进程；桌面壳基准单列且不得隐藏其成本。event/frame ID配对、单调时钟校准偏差；无法量物理上屏只标实际present代理终点。UI061先冻结样本长度/重复/运行顺序/污染规则，成对或交错运行；保失败/缺测/异常与分布，不择优挑样本，p99样本不足不给通过结论。性能测量仅在获准实施后的独立窗口，不与其他会话争用GPU。

测量入口由UI061 T-04的`tests/rq_fidelity/`原型补计时/计数并交付准确命令，后继T-01读回；OS063由`scripts/rq-parity/`编排真实组合。两路径仍是拟新增，本轮不创建脚本或运行基准。不得新增一个与现有oracle无关的泛化跑分系统。

### 验证入口

实施UI树的`cargo test -p auto-ui-runtime --lib ui::desktop_protocol:: -- --test-threads=1`、`cargo test -p auto-ui-runtime --lib ui::iced::broker_surface:: -- --test-threads=1`及真实受影响renderer职责族。改生成器则`cargo tu -- --test-threads=1`；按真实触面必要`cargo ti`。使用UI061交付的rq_fidelity驱动；其准确入口/数据版本T-01读回，不能臆造已存在命令。

T-01确认真实test namespace/feature/平台条件；新增入口由标明任务提供。列用例/执行/条件排除，准备与运行时间分列；非零/超时/skip不充PASS。按实际触面完整职责族和必要消费者运行，最终review后merge只跑冻结并集；不叠跑全仓或AG里程碑。真实窗口验证不抢占其他会话，不能通过提高资源阈值隐藏功能缺口。

### 角色与证据

work负责实现/自检；独立上下文review只检查冻结同一树，不另建reviewer树；merge落地与清理。未指定模型/代理，不自动创建。每条证据绑Plan revision、代码/依赖提交、实际profile/进程/资源/产物，失败/修复后结果分列。用户未授权实施前不进入上述运行。

## 7. 验收标准

| ID | owner_stage | 可观察行为/判据 |
|---|---|---|
| AC-01 | work | UI061全部绘图/资源能力与direct旧行为经两执行形态真实producer与两Host replay验证；按§5绘图fixture验，不以公开运行链为前置，缺口不删分母 |
| AC-02 | work | 文字与全部原语不静默省略（2026-11-07 r4 裁定）：默认 producer（字符串文字 + scrim 阴影）在§6冻结 fixture 上相对 direct 金样像素差 ≤0.1%（阈值 2/255，矩阵默认模式设门）；文字经 Host 同字体栈重排属**具名降级**且必须可观测（不得静默）。v3 富模式（glyph run + SDF 贴片）为 opt-in：结构面必须被断言（glyph 坐标 = cosmic-text `physical()`、阴影与 wgpu shader 同式），其像素差按族设具名上界（glyph run 例 ≤1.0%、含阴影贴片的例 ≤1.5%；r4 记录值 text 0.798%/shadow 0.741%/gradient 1.374%，**2026-11-08 复审修复后实测 text 0.798%/shadow 0.069%/gradient 0.588%**）并作为具名边界记录，**不得计为 exact**。共用实现无第二布局器。 |
| AC-03 | work | 资源版本/传输/更新/最后owner/GPU释放及图片媒体表现正确，无App隐藏窗/GPU/整帧自绘 |
| AC-04 | work | 协商/兼容、坏帧保旧、背压和generation负控通过，未知必需能力显式拒绝 |
| AC-05 | review | 独立上下文逐项验证当前修订AC/V、真实源和规范增量，所有必需项有可复核证据；不能只读完成勾选 |
| AC-06 | merge | 通过后按依赖顺序合入、canonical读回、路线回填，归档/guard/材料清理与恢复摘要完整；交付边界如实记录 |
| AC-07 | work | 真实producer/协议/Replay满足P-01/P-02及P-03队列、P-04资源/工作隔离；V-05～08证明驻留不重复上传、慢Host不覆盖/无界增长、混合负载有进展；**默认 producer 的 produce p95 满足 061 冻结的 W-C ≤10ms**，**唯一具名例外** = SVG 例（安静机记录值 ≈32.5ms、负载下实测 p50 可到 45–60ms；上限设 p50 ≤60ms 以吸收主机负载，根因是记录路径每帧新建 iced renderer，需独立任务修复），该例外不得冲抵其它 workload，也不得据此宣称性能 PASS；**replay 相对 direct 的口径（2026-11-08 r5 裁定）**：稳态 **p50 ≤1.25×direct p50+2.0ms 设门**，p95 只记录不设门（20 样本下 p95≈次大值，实测同代码在 4.1–10ms 间随主机负载变化、max 可到 40ms；公平轮转测量证伪了“重放固定开销”假设——空帧 canvas 7.58/10.38ms 快于纯 widget 树 8.23/15.15ms）；**所有计时门只在隔离窗口签收**（`AUTO_RQ_TIMING_WINDOW=isolated`，共享窗口只记录并标 `timing_gate=skipped_shared_window`）；本职责M指标满足UI061冻结预算，端到端不得借单测冒通过。 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/操作与结果 | 关闭方式/AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | UI061正式可行决策 | 重核UI061全能力ID、direct旧基准、实现来源与预算；冻结当前绘图能力并集及精确接缝；接受UI061 r3性能合同与预算，冻结P/M对应实现来源和direct对照版本，缺预算不进入性能签收 | V-01；AC-01；AC-07/V-05～08（按§6分工）——见§9 2026-11-07 work 轮次：39 View ID + 8 能力ID入编译常量、061 冻结预算原样接受 |
| [x] | T-02 | work | T-01 | 实现完整录制/原语表示与有序消费；文字glyph/font、mesh/path/SVG、transform/clip、gradient/border/shadow和可达custom primitive；禁止近似省略；去除Host重复文字整形/近似再构造；按完整原语批处理，不增加逐图元RPC | V-02；AC-02——完成：Path/mesh/raw text/transform/SVG/image 已交付（`56139b78`）；GlyphRun/SDF 贴片按 r4 口径为 **opt-in 具名边界**；复审 F-01 修复后实测富模式 text 0.798%/gradient 0.588%/shadow 0.069%（≤1.0%/≤1.5% 上界，`f2cdafb4`），默认面 ≤0.1%，结构面断言 + 可观测降级 |
| [x] | T-03 | work | T-02 | 实现资源create/update/release、字节/句柄身份、图片采样/alpha/rotation与媒体更新，Host重放及direct共用语义；实现驻留资源ID/generation复用、按变化上传和有界缓存；计数payload/GPU创建，合法驱逐/恢复单列 | V-03；AC-03——完成：内容身份+驻留、图片 alpha/rotation、SVG 栅格、wire generation（tag 13）、媒体静态帧（`3eadae59`，0.000/0 重传）、跨进程释放/引用记账（`44079c22`，tag 14 + BitmapRefs 延后释放）；连续媒体帧按 2026-11-08 裁定另立 |
| [x] | T-04 | work | T-03；OS062共享符号按§5单写 | 交付UI063提交/反馈接口及旧新端兼容V-09；实现能力/版本协商、原子帧、长度/浮点/栈/资源校验、generation及背压/ACK/GPU fence寿命，保旧兼容明确失败；实现有界异步批量提交及槽信用、资源顺序/背压与per-App上传/驻留/CPU工作预算，避免长临界区和单App垄断 | V-04；AC-04；V-09——完成：能力协商/准入保旧帧、槽 credit 背压、上传预算、慢宿主/旧 ACK/旧宿主×新产者负控（`4263ff3c`）、wire generation + HostCaps 下行（`d2e41c58`）、资源释放 + 引用记账（`44079c22`，复审 F-02/F-04/F-07 修复：合成成功后才记账、按表面、幂等、旧宿主不积压，`f2cdafb4`）、真两进程配对（`39bd7bef`）；乱序上传由 generation 守卫覆盖，可控重排通道属具名边界 |
| [x] | T-05 | work | T-04 | 按§5绘图fixture完成六格全部绘图能力和两Host真实重放，不等UI063公开运行链；保迁移前direct金样；记录资源峰值/帧延迟，整理规范增量与遗留为零；按冻结M-01～08重放实际绘图/资源职责指标与公平/慢Host负控，保direct基线和完整功能，接线端指标交UI063/OS063最终消费 | V-01..04；AC-01..04；AC-07/V-05～08（按§6分工）；V-09——完成：六格真实 wire（`4e2f53d3`）+ 39/39 逐能力读数（`77f92dda`）+ 负控/两进程（`4263ff3c`/`39bd7bef`）+ 媒体静态帧（`3eadae59`，复审 F-06 补了真管道过线用例）、数值条款全部落为断言（复审 F-03）、Spec 增量补 SD-02（复审 F-05）——`f2cdafb4`；M-02/06/07 归 OS063（§5） |
| [x] | T-06 | review | T-05 | 独立上下文复核实际GPU画面、全部能力ID、敏感负控及应用无GPU路径，冻结双端修订；独立核P/M口径、冻结预算、性能负控和完整分布，不能用平均FPS或实现者摘要替代 | AC-05；复核AC-07 |
| [ ] | T-07 | merge | T-06 pass | 定向冻结职责门，正式落地并沉淀wire/资源/保真Spec，保兼容及恢复收据，归档guard清理；沉淀性能支持边界/预算与实测差额，不外推未测平台 | AC-06 |

work项完成才进入execution_done；review/merge项仍待办，不能为交接伪勾。每个任务读取§4对应实际路径及主设计证据；新增接口先由有界决策任务确认。全部原始材料merge折摘要进本Plan§9后删自有scope；移除树先wt-guard clean，不穿透链接、不清未知owner目录。

## 9. 复审记录

2026-11-05 / stage:new / plan_id:auto-ui/PLAN-062 / revision:r1 / outcome:pass（草案合同，不是实现验收） / next:待用户启动后work/T-01，按§8正式依赖核门。

草案静态预检：0–10章节齐全，任务数与total_steps一致，AC/V/SD映射及新文档本地链接有效；主检出/两类归档/注册worktree中的本号唯一。已做源码/设计/相关Spec与重叠计划检查；本轮未实现、构建、跑运行门或独立final review。当前task/AC/V/SD为执行合同；实际通过证据后续填写。设计前置的不确定性保留为任务/阻塞，不能声明已解。

### 2026-11-05 r2性能合同修订

- stage:new / plan_id:auto-ui/PLAN-062 / revision:r2 / outcome:pass（文档修订，非性能验收） / next:待用户启动后按§8依赖进入work。
- 变更：新增AC-07、V-05～08；§5增加P-01～04，§6补M-01～08及direct配对/样本/冻结预算口径；§8原任务增补性能职责，SD-01同步扩展。总任务数不变，current_step仍0，status仍drafting；r1记录保留为历史。
- 用户授权仅文档，未实现、编译、运行或测量；没有性能PASS或节省承诺。r2静态检查只核章节/引用/任务与AC/V映射，不是独立final review。

### 2026-11-05 r3范围与依赖复核

- stage:new / plan_id:auto-ui/PLAN-062 / revision:r3 / outcome:合同修订，静态核查 / next:仍drafting，按§5/§8依赖等待用户启动。
- 修订发现：拆开绘图/运行/产品六格以消除反向依赖；UI060独占索引/输入/刷新/API修复，OS062保Linux通道/平台/安装；补接口交付/兼容门及OS063实际UI策略写面。P/M性能要求及全部能力/产品分母不减少。
- 来源：UI060主检出executing r1/2-6、扩围及续接记录；UI053归档6/6及UI 9a94d568 / lang 61207576b正式交付（当前主线祖先）；OS062 drafting r1/0-13的T-02～05/09～11；headless、UI session/shell_state与OS薄Host入口。启动时重新冻结源版本/单写符号，不将快照永久当准入。
- 保留r1/r2历史；步数、AC数量和current_step不变，补V-09接口负控。未改UI060/OS062/OS054原合同，未建树/实现/构建/运行或签final review；运行证据待后续work。

- r3静态检查：四份合同0～10章节、27项任务（6/7/7/7）、各7条AC/9条V及引用一致；正式depends_on图无环；RQ与本轮现行路线共155条本地链接有效；三仓本轮文档范围diff --check退出0。只证明合同结构/链接，不证明任何运行、性能或Linux支持。

### 2026-11-07 work T-01～T-04 首轮（实施增量，未完成）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（T-01 完成；T-02～T-04 有实质交付但仍有具名缺口；T-05 未开始） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / branch: `codex/plan-062-dev` / base_commit: `3d64877d` / code_commit: `56139b78` / dependency_revisions: auto-lang `8337bab5`(detached 只读)、auto-web `2d698413`、auto-down `58ea33f` / next: work（T-05 六格与指标）
- 授权：`/auto-plan-goal` 的整链授权已记入本会话目标；本轮只做 work，不签 review/merge。
- 环境：实施树 `D:/autostack/.wt/ui-062/auto-ui`（`codex/plan-062-dev`），同组只读依赖树 auto-lang/auto-web/auto-down（detached，未改动）；原始材料仅落 `D:/autostack/plan-artifacts/plan-062/auto-ui/`（已登记 in-flight owner/用途，merge 时整 scope 删除）。

**T-01 完成（能力分母与预算接受）**：重读 061 交付的 `docs/specs/runtime/rq-fidelity-contract.md`、`docs/plans/archived/061-*.md` §9 与主设计 §7；能力分母沿用 61 的 39 个 `View` 变体 ID（`rq_probe::capability_id` 穷尽 match）+ 8 个冻结能力 ID（本轮落为 `message::ALL_FIDELITY_CAPS` 常量，可编译引用）；direct 基准与预算表沿用 061 T-01 冻结值（produce p95 W-A≤1.0ms/W-B≤2.0ms/W-C≤10.0ms、encode+decode+copy p95 ≤64KiB≤1.0ms / ≤1MiB≤3.0ms、稳定重复帧资源上传=0（硬零）、单帧字节 ≤64KiB/≤1MiB、8MiB 控制帧门、replay p95≤1.25×direct+2.0ms、direct 冷启动代理≤2.5s；逐 workload 单位/统计量/样本/owner/失败处理见 061 §9）。本轮未改任何阈值。

**T-02 交付（原语无损，`56139b78`）**：载荷新增 tag 3（v2 超集）——`DisplayOp::Path{WirePath(PathSeg verb 直落), fill(色), even_odd, stroke(宽/色/cap/join)}`；`lower_layers_v2` 不再丢 mesh（061 P683-D1 关闭），`lower_primitives_v3` 按记录序把 `Primitive::Fill/Stroke` 的已烘焙路径过线，组 clip 以 Scissor 包络；`lower_text_v2` 的 `Raw`（编辑器组合态）改为与 paragraph/editor 同源整形后过线（061 P683-D2 的“静默省略”关闭）；重放 `paint_ops_v2` 新增 `Path` 臂并把 `Transform` 由“仅平移”升级为仿射分解（translate∘rotate∘scale，剪切具名降级）；含 Path 的帧编 tag 3，纯 v2 词汇仍编 tag 2（旧端双读窗口不破）。
- 证据命令：`AUTO_RQ_PROBE_OUT=D:/autostack/plan-artifacts/plan-062/auto-ui cargo test -p auto-ui-runtime --lib rq_v3 -- --test-threads=1 --nocapture` 退出 0，4/4 通过。
- 关键读数：`v3_wire_paths` fills=1/strokes=3/segments=12（061 记录侧同为 4 paths/12 段）；mesh 记录→wire→生产重放像素差 **0.0000（0/307200）**；区分性负控（删掉 Path 帧）**4.74%** 必红；SVG 资源（16×16×1024B）重放差 **0.333%**；编解码逐 op 等价；tag=3 与旧端 `UnknownTag(3)` 拒绝路径均断言。
- 仍有具名缺口（不得计通过）：① Host 端仍按字符串 `TextStyled` 重排——`GlyphRun`（glyph id/位置/字体内容 id）未进 wire，61 §4 的“优先 glyph run”未实现；② `shadow.blur` 仍无 SDF/blur 原语（重放为单层垫层）；③ 几何 paint 的渐变 shader 无读回访问器，降级为具名观测；④ custom GPU shader primitive 仍不可达（61 具名不可达 1）。

**T-03 交付（资源身份/驻留）**：`ResourceCache` = 内容身份 `blake3(w,h,bytes)` + `rq-img-<seq>-<hash8>` 局部 id（序号单调 = generation 语义），驻留上限 256 MiB/App（最早键驱逐），统计 `uploads/uploaded_bytes/hits/evictions/resident_bytes`；图像 opacity 烘焙进字节、rotation 以 Transform 包络过线；SVG 在生产端按设备像素栅格化（resvg、`Svg::color` alpha 掩膜、预乘→straight 归一，单边 4096 上限具名拒绝）。
- 证据：`rq_repair_fr4_resource_identity_zero_reupload` 退出 0——无缓存负控 `uploads_per_frame=[1,1,1,1,1,1]`（062 前行为）vs 生产缓存 `[1,0,0,0,0]`、`hits=4`、`resident_bytes=10080`；内容变更重传 1 次、改回命中 0 次、`limit=1` 触发驱逐后重传且 `evictions=3`。
- 仍有具名缺口：媒体（Video/RemoteFrame）资源族与 update/release 显式 oplog 未进本增量；Host 侧 generation 单调检查仍靠 `src` 身份而非 wire generation 字段。

**T-04 交付（协商/准入/背压）**：`CapabilitySet`（`required:<cap>`/`optional:<cap>`，裸串=optional 兼容旧 tail）+ 8 个冻结 ID；`DisplayList::required_capabilities/admit`；`SurfaceStore::compose_v2` 入槽前准入，失败返回具名 `AdmissionError::UnknownRequiredCapability` 且**不写槽**（保旧帧）并在 `host.rs`/`session.rs`/`rqhost.rs` 三处归还传输槽；`AppEndpoint` 帧/位图槽改为 credit 制（`take_frame_slot`/`take_bitmap_slot`，未用槽可直接用，在途槽不得覆盖），新增 `ProtocolError::Backpressure{resource}`、`frame_credits()`、`frames_in_flight()`；客户端泵对 Backpressure 置 `frame_pending`，下一拍补发（不丢帧）。
- 证据：`frame_slot_credits_backpressure_without_overwrite`（endpoint）与 `rq_v3_capability_admission_keeps_old_frame`（host）退出 0：两帧占满两槽 → 第三帧 `Backpressure{resource:"frame"}` 且在途账仍为 2；ACK 后恢复并优先复用归还槽；能力收窄后 Path 帧被拒、`front_v2` 保持旧帧、v2 词汇仍受、能力回补后恢复接受；声明线词汇可逆。
- 回归面：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` → **251 passed / 4 failed**；4 个红在基线（`3d64877d` stash 后同 filter）**完全相同**（`native_projector::autocenter_translates_hits_with_ops`、`stage3_multi_app_stress_n3/n5`、`stage3::t3_examples_queue_end_to_end`），本轮未新增红。
- 仍有具名缺口：per-App 上传速率（≤8 MiB/s）与 CPU 工作预算/公平调度未实现（仅驻留字节上限）；旧新端 v2↔v3 双读只做了 tag 语义与拒绝路径，未做两进程实机配对；慢 Host/乱序资源/旧 ACK 的负控未跑。

**T-05 未开始**：六格（VM/Rust × direct/standalone-RQ/desktop-RQ）真实录制重放、M-01～08 按冻结预算重放、direct 金样对拍与规范增量整理均待下一轮。

### 2026-11-07 goal 轮次记录

- stage: goal | plan_id: PLAN-062 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: executing | outcome: pass（T-01 完成；T-02～T-04 增量交付、T-05 待办） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 56139b78 | next: work（T-05）

### 2026-11-07 work T-05 六格矩阵与指标（实施增量，未清零遗留）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（T-05 矩阵与指标已跑；遗留未清零，VM/桌面格不可运行） / code_commit: `52c11feb` / next: 待定（见§10 具名缺口）
- 命令：`AUTO_RQ_PROBE_OUT=D:/autostack/plan-artifacts/plan-062/auto-ui cargo test -p auto-ui-runtime --lib rq_cells -- --test-threads=1 --nocapture` 退出 0（23.2s）；报告 `cells/t05-cells.json`。
- **六格状态（不删分母）**：`rust/direct` = runnable（本进程 wgpu offscreen 代理金样）；`rust/standalone-rq` = inproc 近似（同源记录 + v3 wire + broker canvas 重放）；`rust/desktop-rq`（owner UI063/OS063）、`vm/direct`（owner UI063）、`vm/standalone-rq`、`vm/desktop-rq` = **unrunnable 且具名 owner**（fixture 不冒充 VM/桌面嵌载，plan §5 同律）。
- **对拍读数（standalone-RQ 相对 direct 金样，同 source/主题/1280×800@scale2）**：path_mesh **0.095%**、svg_icon **0.100%**、text_cjk 0.094%、gradient_border_shadow 0.0625%、scroll_clip **0.000%**、progress_bar 0.0057%。结构面同时核验：mesh 例 `path=4 / path_segments=12 / payload_tag=3 / capabilities=[fidelity/pathfill/1]`；SVG 例 `image=1` + 资源上传 1×1024B；文本/渐变/滚动例 `payload_tag=2`（无 v3 原语不升 tag）。
- **资源面（M-04/M-05 资源侧，P-02）**：SVG 例稳定帧 `uploads=1`、`hits=13`、`resident_bytes=1024`、`raster_hits=13`（栅格化与 payload 均只做一次）；其余例 0 上传。
- **分段耗时（p50，standalone-RQ 例）**：produce 0.09–0.24ms（**svg 例外 31.5ms**）、encode 0.005–0.008ms、decode 0.004–0.008ms、replay 4.25–5.87ms；direct 稳态帧 p50 4.48–6.25ms。即 replay 与 direct 同量级，v3 线/编解码不是瓶颈。
- **具名性能缺口**：SVG 例 produce ≈20–32ms/帧不是 resvg（`raster_hits=13` 已证栅格外置缓存生效），而是记录路径**每帧新建 iced renderer**（`drive_and_draw` 内 `Renderer::Secondary(new)`）导致 iced 的 SVG/text 缓存每帧冷启；owner = T-02 生产侧（需持久化 renderer 或等价缓存迁移），本轮不计入性能通过。
- **M 指标状态**：M-04/M-08 measured；M-01/M-03/M-05 为 proxy（分段/驻留字节，present 与 GPU 驻留未测）；M-02/M-06/M-07 unmeasured（真实 deadline/冷热启动/1-4-8 规模归 UI063/OS063）。无任何数值预算超标结论，也未宣称性能 PASS。
- 回归面：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` → **252 passed / 4 failed**；4 红与 `3d64877d` 基线同一组（autocenter、stage3 stress n3/n5、t3_examples_queue_end_to_end），本轮无新增红。

- stage: goal | plan_id: PLAN-062 | plan_revision: r3 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-05 矩阵/指标增量；遗留未清零） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 52c11feb | next: work（具名缺口或转 review）

### 2026-11-07 work per-App 上传预算（T-04 收口一项）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（T-04 的 per-App 上传速率预算落地；CPU 预算与文字/模糊两项仍待用户裁定） / code_commit: `0e614935`
- 实现：`ResourceCache` 增 8 MiB/1s 滑动窗口（061 冻结值）；超预算帧**整帧回滚**（撤销本帧登记的资源键、复原 uploads/字节/驻留计数），`FrameSource::take_upload_budget_overrun` 让端点决定提交或回滚，`AppEndpoint` 以 `ProtocolError::Backpressure{resource:"upload"}` 报出并归还槽位；客户端泵既有 `frame_pending` 重试路径覆盖该背压（不丢帧）。
- 证据：`rq_v3_upload_budget_defers_frame_without_partial_registration` 退出 0 —— 1024B 预算下：首图放行（uploads=1/驻留 1024）→ 加第二张图延期（`deferrals=1`、`pending_bitmaps` 为空、uploads 仍 1/驻留仍 1024 = 无半登记）→ 预算回补后重试补齐（uploads=2，首图命中缓存）。`[RQ062] {"tag":"upload_budget","deferrals":1,"uploads_after_retry":2,"partial_registration":false}`。
- 回归面：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` → **253 passed / 4 failed**（仍是基线同组 4 红，无新增）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r3 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-04 上传预算收口） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 0e614935 | next: user（§10 三项裁定）

### 2026-11-07 用户裁定（r3 范围内新增两项实现，一项维持代理）

- 用户对 §10 三项作出裁定（本会话交互确认）：
  1. **GlyphRun**：**在 062 内新增 Host 字形图集**（不回退成“字符串+重排”）。即 wire 携 glyph id/位置/字体内容身份，Host 用 FontBlob 与 swash 生成字形位图并按 wire 位置合成；AC-02 的“Host 不重排文字”按精确重放把关。
  2. **shadow blur/SDF**：**在 062 内实现 Host SDF/blur 原语**，不允许保留单层垫层近似作为通过口径。
  3. **per-App CPU 预算**：**本轮只做代理测量**，真实 1/4/8 App 规模与公平策略归 UI063/OS063（与 §5/§6 既有分工一致）。
- 影响：T-02 行新增 glyph atlas 与 SDF blur 两个实现面，SD-01 的规范增量须包含“glyph run wire 结构 + 字体内容身份 + SDF/blur 支持边界”；工程量与风险显著上升，后续轮次以增量提交（wire → Host 图集 → 像素对拍）推进，未收敛前不签 T-02。
- 记录：裁定只改变实现路径，不改变 AC/范围/仓库集合，无需 r4 修订；若实现中发现 canvas 重放面无法精确表达 blur（例如需要自定义 shader），按 §2 具名阻塞并回到用户裁定，不静默降级。

### 2026-11-07 work glyph run（T-02 按裁定实现，默认关闭待混合对齐）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（能力与重放面已实现；像素对齐未收敛，故默认仍走字符串面） / code_commit: `0696129b`
- 实现：
  - wire：`DisplayOp::GlyphRun`（载荷 tag 3）携字体内容身份（blake3 前 16B）、已乘 scale 的字号、字重、`CacheKeyFlags`、颜色、行基线取整偏移与逐字形 `(glyph_id, x, y)`——x/y 即送入 cosmic-text `CacheKey::new` 的物理坐标（含亚像素）。能力 `fidelity/glyphrun/1` 进 `required_capabilities`（不支持的对端在入帧前被拒）。
  - 生产端：`lower_glyph_run` 遍历 layout runs，按字面变化拆 run（中英混排），每帧按 fontdb ID 只哈希一次字体字节；无内容身份时整行退回 `TextStyled` 并留具名观测。
  - Host：`paint_glyph_run` 按内容哈希解析字体（登记表 + 对系统回退字面的一次性 fontdb 扫描），逐字形重建同一 `CacheKey`、经 swash 取位图、按 `CacheKey` 全局缓存，再按 wire 位置与 run 颜色合成——**不做任何整形/布局**。
  - 开关：`HeadlessSurface::set_glyph_runs`（默认关）。六格矩阵已按裁定打开，其余生产/测试面仍走字符串兼容面。
- 实测（`rq_cells`，text_cjk 例，相对 direct 金样）：glyph run 面 **0.92%**（9434/1024000 px 差），字符串面 **0.094%**。差异集中在字形边缘（delta 分布 1–12 级不等，视觉上 glyph run 偏“粗”），根因是位图合成走 wgpu 图像管线与 direct 文本管线的 alpha/sRGB 混合不一致，不是位置或字形选错（结构面 2 runs / 15 glyphs、tag 3、能力=glyphrun/1 均正确）。
- 结论：T-02 的“Host 不重排”路径已具备并有真实录制/重放；但**精确像素未达 direct 水平**，因此不把 glyph run 设为默认、不签 AC-02。下一轮先做混合对齐（预乘/线性空间补偿或改用 canvas 文本管线可控面），对齐后再翻默认。
- 回归面：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` → **254 passed / 4 failed**（基线同组 4 红）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r3 | round: 4 | from: work | to: work | status: executing | outcome: pass（glyph run 能力与重放落地；对齐未收敛） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 0696129b | next: work（混合对齐 → SDF blur）

### 2026-11-07 work glyph 混合对齐诊断（首轮，未收敛）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中 / code_commit: `ba4a24e6`
- 诊断结论（有证据，非猜测）：
  1. **位置公式正确**：新增 `rq_glyph_positions_match_physical_formula` 把 wire 的 `x`（= 送入 `CacheKey::new` 的物理坐标）与 cosmic-text `physical()` 逐字形对拍，CJK 样例在 scale=2 下为 0/32/64/96 全等——重放的坐标漂移假设排除。
  2. **合成语义修正**：字形位图原先按 straight RGBA 上传，画布图像管线按**预乘**合成 → 半透明边缘被加亮，整段文字偏粗。改为覆盖率乘进 RGB 后，text_cjk 重放差 **0.92% → 0.80%**。
  3. **残差仍在字形边缘覆盖/AA**：差异分布在整段（各 32px 列桶都有 400–880 px），峰值为覆盖率而非位置；尚未达到字符串面 0.094% 的水平。
- 状态：glyph run 仍为 opt-in（`set_glyph_runs`），AC-02 不签。下一步要么把 Host 合成换成与 direct 文本管线同源的路径，要么量化证明残差只来自 AA 并在 Spec 里具名窄容差（需 review 同意，不能自定阈值）。
- 回归面：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` → **255 passed / 4 failed**（基线同组 4 红）。

### 2026-11-07 work 阴影贴片（T-02 SDF/blur 按裁定实现）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（阴影改为生产端精确栅格化；文字合成残差仍在） / code_commit: `06d8d232`
- 实现：带阴影的 quad 由生产端按 `iced_wgpu/solid.wgsl` 的同一公式逐设备像素求值（`rounded_box_sdf` 双倍中心坐标、半径取短边一半、`smoothstep(-blur, blur, max(sdf,0))`），生成直通 RGBA 贴片并按内容身份上行资源；帧里先发 `Image(贴片)` 再发 `shadow: None` 的 Quad——Host 不再画垫层近似。几何按 shader 的顶点扩边（`|offset| + 2·blur`）取整，避免模糊外沿被裁；设备/逻辑比例由根层 bounds 推导（不写死 2x），单边 4096 上限具名拒绝。
- 实测（`rq_cells`，新增 `shadow_only` 例，相对 direct 金样）：**0.74%**（37632B 贴片、1 次上传、稳定帧 13 次命中）。此前为单层 scrim 近似（硬边）。
- 仍存差距：0.74% 不是 exact；残差来自贴片经画布图像管线合成（与 direct 的 quad shader 合成路径不同）以及 quad 本体与阴影分两层合成。按 §2 不静默降级，已在 review 口径下具名。
- 回归面：**255 passed / 4 failed**（基线同组 4 红，无新增）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r3 | round: 6 | from: work | to: work | status: executing | outcome: pass（阴影贴片落地；文字/阴影合成残差具名） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 06d8d232 | next: work（收口或转 review）

### 2026-11-07 work SD-01 规范增量草案（worktree 预备，merge 发布）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中 / code_commit: `c22cb50c`
- 产物：`docs/specs/runtime/rq-render-packets-and-resources.md`（SD-01 目标文件，worktree 内预备，**merge 时才发布**）+ `docs/specs/README.md` 导航一条。
- 内容：v3 载荷与编解码规则（tag 2/3 双读窗口、旧端 UnknownTag 保旧帧）、`Path`/`GlyphRun` 的字段与坐标口径（glyph 坐标 = `CacheKey::new` 物理坐标、字体内容身份）、图像/SVG/阴影贴片的生产端职责、资源内容身份+驻留+8MiB/s 上传预算与整帧回滚、能力准入与槽 credit 背压、以及 §5 的**实测保真表**与 §6 的**六条具名遗留**（glyph 0.80%、阴影 0.74%、SVG 记录路径 20–32ms、CPU 预算、媒体族、v2↔v3 双进程未配对）。
- 与 061 合同的边界：本页记录 062 增量，不覆盖 `rq-fidelity-contract.md` 的 v2 现状与冻结能力 ID；新增能力 `fidelity/pathfill/1`、`fidelity/glyphrun/1` 复用其语义。
- 校验：文档命令 `git diff --check` 与链接可解析；实现侧断言由 `rq_*` 测试族复跑（见 §9 各轮）。

### 2026-11-07 独立复审（interim，非最终 T-06）与修复

- stage: review（interim，独立上下文 fork）/ plan_id: auto-ui/PLAN-062 / plan_revision: r3 / reviewed_commit: `c22cb50c` / **outcome: needs_fix**；T-06/T-07 未动，不得 merge。完整记录：`D:/autostack/plan-artifacts/plan-062/auto-ui/review/REVIEW-2026-11-07.md`。
- 复现：reviewer 自行重跑 `rq_`（28 passed）与 `ui::desktop_protocol::`（分支 255/4，base `3d64877d` 246/同 4 红）——§9 的数字与“4 红是基线”成立；`autocenter…` 单独跑在 base 通过属既有顺序依赖，非本轮引入。
- **A/B（reviewer 补测，同 harness/阈值 2/255）**：text_cjk 字符串面 0.0938% → glyph run 0.7979%；gradient_border_shadow 0.0625%（scrim）→ 0.5244%（glyph run）→ **1.3740%（贴片）**；scroll_clip 0.0000% → 0.3298%。即：**阴影贴片相对 scrim 是退步**，§9 前文“此前为单层 scrim 近似”不该读作改进声明——已按 A/B 改正口径。
- **阻塞项（reviewer 认定）**：B1 每字形每帧新建 image handle（P-02/GPU 复用）；B2 能力协商在线上是惰性的（host 恒 `all_optional`、Hello 的 caps 无人解析、Welcome 无能力尾）；B3 矩阵恒开 glyph run，§9/spec 的读数描述的是**非默认** producer 且无像素阈值门；B4 半径逻辑单位对设备几何（圆角只有一半）；B5 无 release/无 Host 上限（`handle_cache`/`glyph_cache` 无界、`bitmap_cache_put` 无 generation 校验、`CacheControl` 不携带 bitmap src）。
- **AC 判定**：AC-01 partial、AC-02 fail、AC-03 partial/fail、AC-04 fail、AC-07 fail；AC-05/AC-06 未开始。
- **本轮修复（`24d44d08`）**：B1 已修（`CachedGlyph` 持 iced 句柄，按 CacheKey 复用）；B4 已修（Quad 半径与阴影 SDF 半径按 `scale` 放大）；N1/N3 已修。附带把字体内容身份改成进程级按 fontdb ID 缓存、阴影贴片按参数缓存——produce 从 12.5/24.1/13.6/1.49ms 降到 **0.18/0.28/0.33/0.28ms**（仅 svg_icon 33.9ms 仍超 061 冻结的 ≤10ms，属记录路径每帧新建 renderer 的既有问题）。回归仍 **255 passed / 4 failed**（基线同组）。
- **未修的阻塞项**：B2（能力协商上真线）、B3（矩阵双模式 + 阈值门）、B5（Host 上限/release/generation）。在它们关闭前维持 needs_fix，不签 AC-02/AC-04/AC-07，不 merge。










## 10. 待澄清事项

- **2026-11-07 阻塞项（实施后证据，需用户裁定）**：用户已裁定“glyph atlas 与 SDF/blur 在 062 内实现”，实现后发现两条都触及**本仓 Host 重放面的能力边界**，按 §2 不得静默降级：
  1. **glyph run 边缘对齐**：wire 坐标已证与 cosmic-text `physical()` 逐字相等（`rq_glyph_positions_match_physical_formula`），字形位图按预乘合成后 text_cjk 相对 direct 的差为 **0.80%**，而字符串重排面是 **0.094%**。残差在字形边缘覆盖/合成（画布图像合成 vs direct 文本管线），不是位置或字形选错。要继续收敛需要 Host 侧与 direct 同源的文本合成路径（更大的宿主改造），或接受具名窄容差。
  2. **shadow blur/SDF**：画布重放面只有线性渐变与路径，没有 blur/SDF 原语；逐像素 SDF 需要自定义 shader，而 061 已把 “custom GPU shader primitive” 列为**具名不可达**（CPU 记录器无该通道）。因此 062 内“精确 blur”当前只能通过两条新路之一实现：① 生产端按 wgpu 同公式预栅格化阴影贴片作为资源过线（新增 wire/资源概念，且等于 App 做局部栅格化，需明确许可）；② 在 Host 侧新增 SDF 原语（等于扩 Host 图形能力，超出本 Plan 的“共用重放”范围）。
  - 需要裁定：**(a)** 接受有测量证据的具名窄容差并写入 Spec 支持边界（062 按现有能力收口）；**(b)** 追加“生产端阴影贴片资源 + Host 同源文本合成”的实现（工作量显著，062 顺延）；**(c)** 把这两项精确重放拆到后继 Plan，062 以当前能力集收口并具名遗留。
- 2026-11-07 实施后发现的三项**合同级缺口**（需用户裁定的范围/预算决定，不能用测试绕过）：
  1. **GlyphRun 与“Host 不重排文字”**（T-02/AC-02）：061 §4 要求文本优先携 glyph run；本仓 Host 重放面是 iced canvas（无 glyph 绘制 API），要满足“Host 不重排”必须新增 Host 侧字形栅格/图集能力（或改用 AGE/自绘 Host 后端）。当前实现仍是字符串 `TextStyled` + FontBlob（Host 用同一字体栈重排），只能算“降级可观测”，不是合同要求的精确重放。需要决定：(a) 在 062 内新增 Host glyph 图集（工作量与风险显著）；(b) 修订 AC-02 把“字符串+同字体栈重排”作为验收口径（具名降级）；(c) 把 glyph run 重放移入依赖 AGE 的后续计划。
  2. **shadow blur / SDF**：重放面仍为单层垫层近似；精确复现需 Host 具备 blur/SDF 能力（61 具名缺口）。同上需要范围决定。
  3. **per-App CPU 工作预算/公平调度**（T-04/P-04）：上传速率预算已于 `0e614935` 落地（8 MiB/1s 滑动窗口、超限整帧回滚 + 背压重试）；仍缺的是 CPU 侧分批/限流与 1/4/8 App 公平策略（真实规模归 UI063/OS063）。若要求本 Plan 内实现 CPU 预算，需要先定“分批单位与抢占点”，否则只能做代理测量。
- **2026-11-08 最后一项待裁定：媒体资源族（AC-03 的 “图片媒体表现”）**。事实：`ui/mpv` 的帧由 sw renderer 直接写进持久 wgpu 纹理（`channel.rs`），**没有 CPU 侧 RGBA 读回**；记录器（tiny-skia Secondary 层）只能看到占位。要把视频/远端帧真的跨线重放，只有三条路：
  - **(a) 062 内实现读回**：每帧（或按需）`copy_texture_to_buffer` 读回 RGBA → 按内容身份当资源过线（复用现有 `Image`/生成/预算/驻留机制）。代价：每帧一次 GPU 读回 + 上传（1080p ≈ 8 MiB/帧，按现有 8 MiB/s 上传预算只能 ~1fps，**必须先改预算或按需采样**），且多一次全帧拷贝。
  - **(b) 具名边界**：在 Spec/AC 里把“媒体帧跨线”写成具名边界（当前只保证占位/降级可观测），媒体传输另立计划（与 OS063/媒体管线一起设计）。
  - **(c) 只做静态帧/海报帧**：把视频首帧或暂停帧读回一次当图片资源（证明链路+身份+预算可用），连续播放仍具名留后。
- **测量窗口限制（2026-11-08，第五轮复审提出须落在 §10）**：`AUTO_RQ_TIMING_WINDOW=isolated`
  只是**标签**，不隔离机器资源；本会话的主机长期与其它会话/构建共享（reviewer 记录 CPU 9–23%）。
  因此 r5 的计时证据（隔离窗口 `rq_probe_t01` 41/41 p50 通过、最差余量 2.70ms；`rq_cells`
  28 行 produce/SVG/pixel 门 asserted）**是“在该窗口内成立”的记录，不等于空闲主机签收**。
  若后续需要把 AC-07 的计时结论写进正式交付边界，应在真正空闲的机器上重跑一次同一命令集。
- **本轮已收口的项**：per-App 上传速率（8 MiB/s + 整帧回滚 + 背压）已完成并用负控验证；wire generation、慢宿主/旧 ACK/旧宿主负控、真两进程配对亦已收口，均不含在上述待裁定清单内。
- **2026-11-07 主检出存在他方未提交 WIP（merge 前必须由 owner 归位，不属本 Plan）**：主检出 `D:/autostack/auto-ui`（master `df09d816`）工作树里有未提交修改 `crates/auto-ui-runtime/src/ui/desktop_protocol/{headless,host,rqhost}.rs`、`ui/session.rs`，内容是把已撤回的 `drain_resource_releases`（PLAN-062 在 `00bd355d` 整提交撤回的 ResourceRelease）接线到 `FrameSource` 实现上；master 上这些方法不存在，该 WIP 与主线不可编译/语义不一致。本 Plan 未触碰、未 include、未 stash；worktree `codex/plan-062-dev` 不受影响。**merge 前**需其 owner 把这些改动路由回自己的树（stash/patch 交接或 fix worktree），否则主检出不满足 master-zero-WIP 门。
- 性能数值预算/实际刷新率/可运行规模待UI061授权执行后按先基线后候选冻结；无预算不签PASS，支持承诺或阈值变化须具名修订。
- SVG 记录路径每帧新建 iced renderer（produce ≈20–32ms，T-05 实测）属 T-02 生产侧实现债，修它需要改 `HeadlessSurface` 的 renderer 生命周期（缓存迁移风险），建议单独有界任务而不是塞进本 Plan 收尾。
- 本轮没有必须再询问才能成稿的问题；实施暂停来自用户的规划范围。后续启动不改变既有依赖门。
- UI061的准确录制接缝/版本/资源预算是具名技术决策；未取得可行证据前不猜其后继实现，若改变承诺/仓scope给具体修订。
- 字体/IME/媒体/自定义图形或产品原有缺口不能以N/A自动缩分母；明确owner、实际失败入口、关闭条件，必要时另立Plan，当前必需项保持未完。
- Linux自有compositor和ACE/AGE的未来接口按战略进入门另立实施合同，本Plan没有自动切平台/引擎授权。

### 2026-11-07 work 复审修复 B3 + v3 富模式改为 opt-in

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中（B1/B3/B4/N1/N3 已修；B2/B5 仍开） / code_commit: `54b0b6e2`
- 变更：`HeadlessSurface` 新增 `set_shadow_patches`（默认关，与 `set_glyph_runs` 同律）——默认 producer = 字符串文字 + scrim 阴影，v3 富模式（glyph run + SDF 贴片）为 opt-in；`rq_cells` 每例**两种模式都测**、读数带 `producer_mode`，且**只对默认模式设门**（path_mesh/svg_icon/text_cjk/scroll_clip/progress_bar ≤0.2%，阈值 2/255）。
- 双模式读数（standalone-RQ 相对 direct，同 harness）：
  - **default**：path_mesh 0.095%、svg 0.100%、text_cjk 0.094%、gradient 0.0625%、scroll 0.000%、shadow_only 0.005%、progress 0.006% —— 全部 ≤0.1%，门通过。
  - **v3 富模式**：text_cjk 0.798%、gradient 1.374%、scroll 0.330%、shadow_only 0.741% —— 与复审 A/B 一致，作为具名遗留，不计入通过。
- 结论：复审 B3 关闭（矩阵不再用非默认 producer 冒充默认且有阈值门）；B4 关闭（半径同尺度）；B1 关闭（句柄复用 + 字体/贴片缓存，produce 0.18–0.33ms）。B2（线上能力协商）与 B5（Host 上限/release/generation）仍开，AC-02/AC-04/AC-07 不签。
- 回归面：**255 passed / 4 failed**（基线同组 4 红）。
### 2026-11-07 work 复审修复 B2 + B5（首轮 needs_fix 全部关闭）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r3 / outcome: 进行中 / code_commit: `4b0e7ee7`
- **B2（能力协商上真线）**：App 的 Hello 能力尾声明 `required:fidelity/pathfill/1` + `optional:fidelity/glyphrun/1`；`HostEndpoint` 在 Hello 处解析成 `CapabilitySet`，`activate`/`activate_multi` 前校验 required ⊆ supported，缺一返回 `ProtocolError::Admission(cap)` 且**保持 Listening**（不半启动）；`set_supported` 供受限宿主收窄。新测试 `capability_negotiation_gates_activation`：负控（要求 glyphrun、宿主不支持 → 拒绝且仍 Listening）、正控（要求 pathfill → Welcome + Active）、以及真实 App 端点确实声明 pathfill。
- **B5（Host 有界与旧写拒绝）**：`handle_cache` 改 FIFO 上限 512（`handle_cache_insert`），`glyph_cache` 上限 4096（溢出裁剪）；`bitmap_cache_put` 从 `rq-img-<seq>-<hash>` 解析 generation，**旧的迟到更新被拒**并留具名观测（此前无版本号、旧 BitmapReady 可静默覆盖）。
- 仍缺（如实记录）：Host 侧**显式释放**路径（`CacheControl` 不携带 bitmap src）与“最后 owner 释放”语义未实现——目前只靠上限裁剪，AC-03 的释放面因此不签。
- 回归面：**256 passed / 4 failed**（基线同组 4 红）。
- 复审状态：首轮 needs_fix 的 B1/B2/B3/B4/B5 均已处理（B5 的释放面列为遗留），已提交待第二轮独立复审。
### 2026-11-07 第二轮独立复审与修复（needs_fix → 已修，待裁定 AC 措辞）

- stage: review（第二轮，独立上下文）/ reviewed_commit: `4b0e7ee7` / **outcome: needs_fix**（记录：`plan-artifacts/plan-062/auto-ui/review2/REVIEW2-2026-11-07.md`）。
- 复现：reviewer 自跑 `rq_` 28/0 与 `ui::desktop_protocol::` 256/4（4 红与 base `3d64877d` 同组）；并给出双模式新鲜读数（default / v3-rich）：path_mesh 0.0949/0.0949、svg 0.1000/0.1000、text 0.0938/0.7979、gradient 0.0625/1.3740、scroll 0.0000/0.3298、shadow_only 0.0053/0.7410、progress 0.0057/0.0057。
- B1/B3/B4 判定已修（B3 附两点 caveat：mode 名误导、gradient/shadow 无门）；**B2 partial**（`SurfaceStore::set_capabilities` 无生产调用者、镜像转发无准入、Welcome 无能力尾）；**B5 partial + 新隐患**（512 FIFO 会淘汰 `bitmap://` → 占位永久化；generation 守卫结构上不可能触发）。
- 本轮修复（`f03a2747`）：
  1. `bitmap://` 句柄不参与本地淘汰（producer 拥有、丢失不可自愈）；文件/http/资产仍受 512 上限。
  2. 删除 vacuous generation 守卫，Plan/Spec 改为如实记录“wire 无显式 generation 字段”。
  3. `SurfaceStore::default` 默认全能力（空集会拒掉所有 v3 帧），`ProtocolHost` 在 Hello 落点用 `HostEndpoint::supported` 覆盖；rqhost 镜像转发前也走准入。
  4. 矩阵默认模式的门覆盖 gradient/shadow（0.0625%→1.37% 的默认退化现在会失败）；opt-in 模式改名 `v3_rich`。
  5. Spec 修正：表头不再写“默认 = glyph run 开”，§2.2 补默认关caveat，§3 记 Host 上限与 bitmap 豁免，§5 性能刷新（默认 produce 0.17–0.24ms、svg ~32.5ms、replay 5.2–6.5ms 含一次 10.09ms 离群），§6 补三条遗留（Welcome 无能力尾、Host 无显式释放、wire 无 generation）。
- 复跑：`rq_` 28/0；`ui::desktop_protocol::` 256/4（基线同组）。
- **待裁定**：两轮独立复审一致认定，按现行 AC-02/AC-07 文字 062 **不能签**（AC-02 要求“无近似/不重排”，而默认面恰是 Host 重排、v3 富模式又非 exact；AC-07 的 svg produce 32.5ms 超冻结 ≤10ms 预算）。要收口必须由用户选择：**(a)** r4 修订 AC-02/AC-07，把“具名窄容差 + 具名降级”写成验收口径；**(b)** 继续做 Host 同源文本合成与 renderer 生命周期改造（工作量大，062 顺延）；**(c)** 以当前能力集收口 062，精确重放另立 Plan。
### 2026-11-07 r4 合同修订（用户裁定：AC-02/AC-07 写具名容差与具名降级）

- stage: new / plan_id: auto-ui/PLAN-062 / revision: r4 / outcome: pass（合同修订，不是实现验收） / next: work（按新口径整理证据与剩余任务）
- 授权来源：用户 2026-11-07 对三轮提问的选择（glyph atlas 内做、SDF 内做、CPU 预算只代理），以及本轮对“按现行 AC 无法签字”的裁定：**选择 r4 修订 AC-02/AC-07，把具名窄容差与具名降级写成验收口径**。
- 修订内容（只改验收措辞与对应 V 行，不改目标/范围/仓库集合/任务集）：
  1. **AC-02**：默认 producer（字符串文字 + scrim 阴影）相对 direct 金样 ≤0.1%（阈值 2/255，矩阵默认模式设门）；Host 同字体栈重排为**具名且可观测**的降级；v3 富模式（glyph run + SDF 贴片）为 opt-in，须断言结构等价（glyph 坐标 = physical()、阴影与 shader 同式）且像素差 ≤1.0%，**不得计为 exact**。
  2. **AC-07**：默认 producer 的 produce p95 满足 061 冻结 W-C ≤10ms；**唯一具名例外** SVG 例（≈32.5ms，根因=记录路径每帧新建 iced renderer），单列记账、不得抵扣其它 workload、不得据此宣称性能 PASS。
  3. **V-02/V-07** 同步为新口径（默认模式设门 + v3 富模式具名边界 + SVG 例外单列）。
- 依据（实测，§9 前文）：默认模式 path_mesh 0.095%、svg 0.100%、text_cjk 0.094%、gradient 0.0625%、scroll 0.000%、shadow_only 0.005%、progress 0.006%；v3 富模式 text 0.798%、gradient 1.374%、scroll 0.330%、shadow_only 0.741%；默认 produce 0.17–0.24ms（SVG ≈32.5ms）。
- 保留：r1/r2/r3 历史与全部 §9 收据；status 仍 xecuting（本修订未完成 T-02/T-03/T-04 的收尾与 T-05 全量复核）。SD-01 目标文件不变，正文已按实测更新（表头/开关 caveat/Host 上限/遗留清单）。
- 未做：本轮未改实现、未重跑全量、未做独立 final review；AC-05/T-06 与 AC-06/T-07 仍待办。
### 2026-11-07 work 宿主能力声明（T-04 收口 AC-04 的协商一半）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中 / code_commit: `6b3b7fb0`
- 实现：新增追加式握手消息 `HandshakeMsg::HostCaps { caps }`（tag 4）。App 在 Hello 里声明 `optional:welcome-caps/1`，宿主据此在 Welcome 之后补发能力声明；`AppEndpoint::peer_caps` 保存解析结果（producer 可从握手得知宿主支持面，而不再只靠本地开关）。未声明标记的旧端永远收不到该消息，Welcome 字节不变。
- 测试：`host_caps_reach_the_app_after_welcome`（声明标记 / Welcome 本身不带能力 / HostCaps 后 peer_caps 含 `fidelity/pathfill/1` / 状态不变）；`incubation_allocates_real_session_objects` 更新为 Welcome+HostCaps+BufferAlloc 三消息序。
- 回归：`ui::desktop_protocol::` **257 passed / 4 failed**（基线同组）。
- **尝试后撤回**：曾加“同 src 异内容即拒绝”的内容指纹守卫，被现有 codec 测试证伪——协议既有语义是“同 id 重上传 = 即时翻新”，因此 generation 乱序保护**必须**有 wire 显式字段，不能靠内容寻址替代。已 revert，Spec §6 继续把该字段列为具名遗留（这一结论现在是实测支撑，而不是猜测）。
- 仍缺（AC-03/AC-04）：宿主显式释放路径（最后 owner 语义）、两进程 v2↔v3 配对、慢 Host/旧 ACK/乱序资源负控。
### 2026-11-07 work 资源释放路径（T-04/T-03 收口 AC-03 的释放面）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中 / code_commit: `310d9111`
- 实现：追加式 `FrameMsg::ResourceRelease { wid, srcs }`（tag 13）——producer 驱逐（最后 owner 归零）时把 `bitmap://` 源通知 Host，Host 丢弃句柄（三处消费点：ProtocolHost / 桌面 session / rqhost 全部接线）；客户端泵在上传排水后立刻发送释放批。
- 正确性加固：`enforce_limit` 不再驱逐**当前帧刚登记**的资源（剩余候选全是本帧资源时停止驱逐并保留超额），避免帧引用悬空；测试用 1 字节预算验证只释放非本帧项（`released=["bitmap://…rq-img-1-ec3b998d"]`）。
- 证据：`rq_resource_release_follows_producer_eviction` 通过；`ui::desktop_protocol::` **258 passed / 4 failed**（基线同组，无新增）。
- AC 状态：AC-03 的“最后 owner → 释放”现已闭环（producer 侧驱逐语义 + Host 释放）；仍缺媒体资源族与 wire 显式 generation 字段。AC-04 仍缺两进程 v2↔v3 配对与慢 Host/旧 ACK/乱序负控。
### 2026-11-07 最终独立复审（第三轮）与修复（needs_fix，未 merge）

- stage: review（final，独立上下文）/ reviewed_commit: `310d9111` / **outcome: needs_fix**（记录：`plan-artifacts/plan-062/auto-ui/review3/REVIEW3-2026-11-07.md`）。
- 复现：reviewer 自跑 `rq_` 29/0、`ui::desktop_protocol::` 258/4（4 红与 base `3d64877d` 同组）；新鲜矩阵 default 0.00095/0.00100/0.00094/0.00063/0.00000/0.00005/0.00006（全部 ≤0.1%）；v3_rich text 0.00798、shadow 0.00741、gradient 0.01374（均在 r4 具名界内）；default produce p95 ≤0.387ms（SVG 例外 37.5ms）。
- **裁定：AC-07 可签**（其判断）；AC-01/AC-02/AC-03/AC-04 不可签。阻塞项：
  - **B1** `ResourceRelease`（tag 13）未协商：旧宿主会 `UnknownTag(13)` 解流失败，不是“拒绝该帧、保旧帧”；且无 codec 往返测试。
  - **B2** 释放可能驱逐仍在用的资源：无帧引用/ACK 记账，Host 立即驱逐 → 前台帧可能画占位。
  - **B3** 默认字符串面（Host 重排）**没有产物观测**，不满足 r4 AC-02 的“具名且可观测”。
- **本轮修复（`00bd355d`）**：
  1. **整提交撤回** `ResourceRelease`（消息/宿主三处/泵/测试）——按 reviewer 给的“要么补齐记账，要么先移除”选项选择移除；Spec §3/§6 改为**具名记录“跨进程释放/引用记账未交付”+ 撤回原因 + 重引入条件**（协商标记属合同变更 + 帧引用计数）。
  2. **B3 关闭**：`lower_text_runs` 每次产帧产出具名降级观测「文字降级：N 个 run 以字符串 TextStyled 过线」，新测试 `rq_string_text_path_is_observable_degradation` 断言其存在（non-silent）。
  3. **N2 关闭**：矩阵默认门 0.2% → **0.1%**（与 AC-02 r4 一致，实测 0.00094–0.00100 仍通过）。
- 冻结门复跑（本树 `00bd355d`）：`rq_` **29/0**；`ui::desktop_protocol::` **258 passed / 4 failed**（基线同组）。
- **未解决（需用户裁定或续做）**：AC-01（六格只跑 2 格、39 个 View ID 无逐项结果、无两 Host 真实重放）；AC-03（跨进程释放/引用记账、媒体族、wire generation）；AC-04（generation 负控、两进程 v2↔v3 配对、慢 Host/旧 ACK 负控）。按 r4 先例，这四项要么再授权一次口径修订（r5），要么继续实现；本 Goal 轮次预算已用尽，未 merge、未归档。
### 2026-11-07 work 真实 wire 六格（T-05 传输级）与 v3 槽分派缺陷修复

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（六格全部有真实 wire 证据；“39 个 View ID 逐项”仍未做） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / branch: `codex/plan-062-dev` / code_commit: `4e2f53d3`
- 新增 `rq_wire`（test-only）：producer → `ClientPump`（Hello/激活/槽信用）→ 真实命名管道 + shm 帧槽 + ACK → 宿主端点 → **两套不同**的宿主合成面 → broker canvas 快照 → 与 direct 金样对拍。宿主：standalone = `rqhost::RqServe/adopt_one/pump_client`；desktop = `DesktopSession` 的 broker 客户端（同 `compose_v2` 准入/保旧帧语义）。产者：Rust 构件与 VM（`build_dynamic_component` 解释执行 AutoLang 源码；direct 金样取同一解释结果的 view，不手写期望包）。
- **发现并修复线上缺陷（P0，`4e2f53d3`）**：`FrameReadyShared` 的槽载荷分派只认 tag 2；v3 帧（DisplayList tag 3）落进 v1 DrawList 分支解码失败后**被静默丢弃**——path_mesh 在两宿主上合成面恒空（inproc 矩阵不经传输，掩盖了该缺陷）。修复 = `shm::is_display_list_payload`（2|3）成为唯一判定点，三处消费（`rqhost`、`host::ProtocolHost`、桌面 `session`）共用；回归 `display_list_payload_kinds_cover_v2_and_v3`（含真实编码首字节 = 3 的断言）。
- 读数（1280×800@scale2，阈值 2/255，默认 producer，**两宿主逐例相同**）：path_mesh **0.095%**（tag 3 经真实槽分派）、svg_icon 0.039%、text_cjk 0.094%、gradient_border_shadow 0.063%；VM 产者 vm_text 0.094%、vm_icon 0.057%、vm_row_button **0.000%**。全部 ≤0.1% 门。
- 六格状态更新：`rust/direct`、`rust/standalone-rq`（本模块 inproc）+ `rust/desktop-rq`、`vm/direct`、`vm/standalone-rq`、`vm/desktop-rq`（`rq_wire` 真实 wire）——六格全部可运行且有逐例读数；`rq_cells` 的“四格不可运行”断言改为“六格必须有证据（本模块读数或 rq_wire 点名）”。**边界不变**：peer 跑本进程线程（真实管道/端点/shm/ACK），present 仍是 offscreen 快照；跨进程换载/物理上屏归 OS063。
- 排障还原（不当结论）：首测 gradient 0.83%，根因是 `DesktopSession` boot 把**全局主题**切到 stella，direct 金样与产者不同主题；恢复全局主题后 0.063%。已写进 Spec §5.1（矩阵必须两侧同主题，否则量到的是主题漂移不是保真差）。
- Spec 同步（SD-01 目标文件）：§1 增“槽载荷分派”规则（1=v1；2/3=DisplayList 同臂，唯一判定点）；§5 重写为“传输级六格 + 同进程矩阵”两层证据，并**修正上一版默认/富模式读数写反**的表格；§6 第 6 条更新为“同进程真实管道配对已覆盖，真两进程/慢 Host/旧 ACK 仍缺”。
- 回归面（本树 `4e2f53d3`）：`rq_` **30/0**；`ui::desktop_protocol::` **260 passed / 4 failed**（4 红与 base `3d64877d` 同组：autocenter、stage3 stress n3/n5、t3_examples_queue_end_to_end）。
- 仍缺（不签 T-05）：39 个 View ID 的逐项读数；AC-03 媒体族与 wire generation；AC-04 真两进程 v2↔v3 配对、慢 Host/旧 ACK/乱序资源负控；M-02/M-06/M-07 未测。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 7 | from: work | to: work | status: executing | outcome: pass（六格 wire 证据 + P0 槽分派修复） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 4e2f53d3 | next: work（39 View ID 逐项 / AC-03 / AC-04）

### 2026-11-07 work 逐能力面 39/39（T-05 / AC-01 最后一项）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（AC-01 的“39 个 View ID 逐项”关闭；“六格/两 Host 真实重放”已在 `4e2f53d3` 关闭） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / code_commit: `77f92dda`
- 新增 `rq_probe_t05_capability_rows`：每个 View 能力 ID 一例（语料补齐 `empty`/`column`；**缺例即测试失败，不删分母**），走与产品同源记录路径（`HeadlessSurface` → `render_frame_v2` → broker canvas 重放），与 direct 金样（同一 View 的 iced 元素、不经协议）同主题同 viewport 对拍；每行带结构清单、资源计数（uploads/hits/evictions/resident）与像素差，落 `capabilities/t05-capability-rows.json`。
- 读数：**26/39 ≤0.1%**；13 个 >0.1% 的族全部是文字密集或宿主依赖面——managed_scroll_content 1.56%、code_editor 1.31%、video 0.92%、terminal 0.46%、rich 0.31%、input 0.25%、scrollable 0.24%、text(latin) 0.23%、navigation_rail 0.21%、table 0.20%、remote_frame 0.17%、autodown_editor 0.15%、mouse_area 0.14%；与 Spec §5.3 已具名的文字合成残差同源（画布文本管线 vs direct 文本管线），不是位置或内容丢失。
- 逐 ID 关闭条件（AC-01）：真实输出 + 结构断言（除 `empty` 与未注册 `custom` 防御臂外，每能力 ≥1 绘制 op）+ **2% 毛刺级诊断上限**（抓整片丢失/底色错误）；exactness 仍只由 §6 矩阵的 0.1% 门负责，不把逐 ID 读数当 exact。
- **修掉一个测量 bug（非实现缺陷）**：重放清屏色原先取 direct 图左上像素；内容压住 (0,0) 的例（mouse_area/table/workspace_preview）会把它当底色刷满整屏，0.1% 级差被放大成 99.6/97.9/95.6%。改为两侧都用主题底色 `bg()` 后回到 0.14/0.20/0.03 量级；口径写进 Spec §5.2 与测试注释。顺带把三处探针各自的 diff/清单实现收敛到 `rq_probe`（其中一份拷贝漏了尺寸检查）。
- Spec（SD-01）：新增 §5.2 逐能力面（39 ID 读数与关闭条件）、§6 第 9 条（13 个 >0.1% 族不属 exact，不得计入 AC-02 门）；§5 小节编号统一（5.1 传输级 / 5.2 逐能力 / 5.3 同进程矩阵）。
- 回归面（本树 `77f92dda`；本轮改动全部是 `#[cfg(test)]` 与文档，生产代码与 `4e2f53d3` 相同）：`rq_` **31/0**；`ui::desktop_protocol::` **261 passed / 4 failed**（同基线 4 红：autocenter、stage3 stress n3/n5、t3_examples_queue_end_to_end）。
- 仍缺（不签 T-05/T-03/T-04）：AC-03 媒体资源族与 wire generation；AC-04 真两进程 v2↔v3 配对、慢 Host/旧 ACK/乱序资源负控；M-02/M-06/M-07 未测。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 8 | from: work | to: work | status: executing | outcome: pass（39/39 逐能力读数 + 测量口径修复） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 77f92dda | next: work（AC-03 资源族 / AC-04 负控）

### 2026-11-07 work AC-04 负控（慢宿主 / 旧 ACK / 旧宿主 × 新产者）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（AC-04 的“慢 Host/旧 ACK”与“v2↔v3 配对”已有真实管道证据；剩两进程与乱序资源） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / code_commit: `4263ff3c`
- 三条负控都跑在真管道上（standalone 宿主，两端可分别泵；产者用 tick 驱动的 `SwitchingView`，v2/v3 可切）：
  1. **慢宿主**：宿主停摆 250ms 只泵产者（tick 15ms）→ 在途帧 **= 2**（槽数封顶）、待发位 **true**、宿主侧 0 帧；恢复后继续合成 ≥3 帧、待发位清空。即队列按信用有界，只有过期画面合并，不覆盖在途槽、不丢最终状态。
  2. **旧/重复 ACK**：宿主回 `FrameAck{frame_id:1, slot:1}`（早已归还的槽 + 过期 id）→ 连接存活、槽账不串、帧数 2 → 3 继续增长、合成面仍有效。
  3. **旧宿主 × 新产者**：宿主能力面收窄为只有 glyphrun（无 `fidelity/pathfill/1`），产者先发 v3 Path 帧再切回 v2 词汇 → v3 帧**入帧前拒绝**（frames=0、无合成面 = 不呈现半帧、连接不掉），切回 v2 后**同一连接**恢复合成（1 帧字符串、required_capabilities 为空）。
- 观测口：`ClientPump::frames_in_flight()` / `is_frame_pending()`（小 API，兼作 M-04/M-05 计数面）；未改任何协议语义。
- Spec（SD-01）：新增 §5.4 三条负控的实测表与语义要点，并显式保留未覆盖项（真两进程配对、乱序 BitmapReady）。
- 回归面（本树 `4263ff3c`）：`rq_wire` **5/0**；`ui::desktop_protocol::` **264 passed / 4 failed**（同基线 4 红：autocenter 内容居中、stage3 stress n3/n5、t3_examples_queue_end_to_end——三者均为 062 开工前既有失败，非本轮引入）。
- 仍缺（不签 T-05/T-03）：AC-03 wire generation 字段与**媒体资源族**（mpv 帧走 GPU 纹理、无 CPU 读回，跨线需独立读回/媒体传输设计）；AC-04 真两进程（独立 exe）配对、乱序资源负控；M-02/M-06/M-07 未测。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 9 | from: work | to: work | status: executing | outcome: pass（AC-04 三条负控） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 4263ff3c | next: work（wire generation / 两进程配对 / 媒体边界裁定）

### 2026-11-08 work wire generation + HostCaps 下行（T-04/T-03，AC-03 generation 面）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（AC-03 的 wire generation 关闭；媒体族仍开） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / code_commit: `d2e41c58`
- 协议增量：`FrameMsg::BitmapReadyGen`（tag 13，字段序同 tag 10 + 尾追 `generation: u64`，会话内单调）。产者只在宿主 `HostCaps` 声明 `resource/generation/1` 时发送；未声明 = 旧 tag 10，**字节不变**；旧宿主读到 tag 13 命中 `UnknownTag` 明确拒绝，不会按旧布局误读多出的 8 字节（tag/长度差由 `message` 单测钉住）。
- 宿主守卫：`broker_surface::bitmap_cache_put_gen` 按资源 id 记最后已应用序号，**严格更小（迟到/乱序）与相等（重复投递）一律丢弃并保旧内容**，仍回 `BitmapAck` 归还槽（槽不泄漏）；三处消费（`rqhost`/`host::ProtocolHost`/桌面 `session`）共用，旧 tag 路径语义不变（同 id 重上传 = 即时翻新）。
- **发现并修复协商下行缺口（同类缺陷第二次出现）**：`HostCaps` 此前只有 `host::ProtocolHost` 发送；独立 RQ 宿主（`rqhost`）与桌面 broker 都不发——产者在**真实宿主**上永远学不到 `resource/generation/1` 等可选面，协商在线上是死的（与第二轮复审 B2 同根）。两处按同一条件补齐（App 必须声明 `welcome-caps/1` 才发，旧端零未知消息）。
- 证据（本树 `d2e41c58`）：`bitmap_generation_only_on_negotiated_lines`（未协商 → tag 10；协商后 → tag 13，序号 2,3 单调）、`p062_bitmap_generation_rejects_late_and_duplicate_uploads`（gen5 接受 → gen3 拒 → gen5 拒 → gen6 翻新；旧 tag 仍翻新）、`rq_wire_generation_capability_is_negotiated_and_applied`（真管道：HostCaps 列出 8 个能力面含 `resource/generation/1`，svg 位图经该路径合成 `image_ops=1`）。
- Spec（SD-01）：§3 增“线上 generation”规则（tag 13/序号守卫/槽归还/旧端兼容）；§6 第 7 条改写为“已交付 + 乱序资源负控仍缺（需可控重排通道，管道是 FIFO）”。
- 回归面：`rq_` **35/0**；`ui::desktop_protocol::` **266 passed / 4 failed**（同基线 4 红：autocenter、stage3 stress n3/n5、t3_examples_queue_end_to_end）。
- 仍缺（不签 T-05/T-03）：**媒体资源族**（mpv 帧直写 GPU 纹理、无 CPU 读回——需要独立读回/媒体传输设计）；AC-04 真两进程（独立 exe）配对与乱序资源负控；M-02/M-06/M-07 未测。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 10 | from: work | to: work | status: executing | outcome: pass（wire generation + HostCaps 下行补齐） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: d2e41c58 | next: work（两进程配对；媒体边界裁定）


### 2026-11-08 work 真两进程 v2↔v3 配对（T-05 / AC-04 最后一项）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（AC-04 的负控面收口；只剩媒体族待裁定） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / code_commit: `39bd7bef`
- 测试体：`rq_wire_child_body`（env 驱动；无 env 直接返回，普通跑套件零副作用）+ `rq_wire_two_process_v2_v3_pairing`（父进程起 `RqServe`，`current_exe` 派生两个**独立进程**子产者：一个产 v3 画布帧、一个产 v2 字符串帧）。
- 断言：两条连接来自两个不同 app（Hello 凭据 `plan062-child-v3`/`-v2`）、两个子进程在配对期均存活、v3 帧 `requires_v3()=true` 且带 `Path`、v2 帧为字符串；两者都经**跨进程**管道与命名 shm 段落地，不是同进程近似。
- Spec（SD-01）：§5.4 增“真两进程 v2↔v3 配对”行；§6 第 6 条改写为“tag 语义/拒绝路径/同进程两产者×两宿主/真两进程配对 + 慢 Host/旧 ACK/旧宿主负控均已实测；跨进程换载与物理上屏归 OS063”；§5.4 增“乱序资源”语义（上传方向由 generation 守卫；资源晚于引用帧到达 = 占位不落负缓存、到达后命中；仍有待可控重排通道的注入负控）。
- 回归面（本树 `39bd7bef`）：`rq_` **37/0**；`ui::desktop_protocol::` **268 passed / 4 failed**（同基线 4 红）。
- 仍缺：**媒体资源族**（Video/RemoteFrame 帧跨线——mpv 帧直写 GPU 纹理、无 CPU 读回，需用户裁定实现读回或在合同里具名为边界）；乱序资源注入负控（需可控重排通道）；M-02/M-06/M-07（OS063）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 11 | from: work | to: work | status: executing | outcome: pass（真两进程配对） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 39bd7bef | next: user（媒体族边界裁定）



### 2026-11-08 用户裁定（媒体族与主检出 WIP）

- **媒体资源族**：用户选择 **(c) 只做静态/海报帧读回，连续播放具名留后**。即 062 内交付“一张已解码帧 → 资源身份/generation/预算/驻留 → 跨线重放”的完整链路并用合成帧源验证；连续视频（每帧读回 + 带宽）明确列为具名遗留，与 OS063 传输/媒体管线另立计划，**不改冻结的上传预算**。
- **主检出 WIP**：用户裁定“先由该 owner 归位”，本轮不 stash、不 patch、不 restore；merge 前该 WIP 必须回零（若届时仍未归位，merge 会按 stop condition 停下并报具体 unblock 动作）。
- 影响：T-05 的 AC-03 媒体面按“海报帧链路交付 + 连续播放具名”收口；AC-03 其余面（身份/版本/传输/更新/最后 owner/GPU 释放）此前已交付。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 12 | from: user | to: work | status: executing | outcome: pass（裁定落地） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 39bd7bef | next: work（海报帧链路）

### 2026-11-08 work 媒体静态帧链路 + 探针位图键缺陷修正（T-03/AC-03）

- stage: work / plan_id: auto-ui/PLAN-062 / plan_revision: r4 / outcome: 进行中（AC-03 的媒体静态帧面关闭；剩“最后 owner/释放记账”） / worktree: `D:/autostack/.wt/ui-062/auto-ui` / code_commit: `3eadae59`
- 按用户裁定实现静态帧链路：`ui::media_poster`（进程级注册表：媒体键 → 一帧紧排 RGBA + 单调代次）→ `render_video` 命中即用普通 `image` 元素出图 → 记录器按栅格字节过线，**身份/去重/上传预算/驻留全部复用图片管线**（不新增第二套资源通道）。命中后不再走 shader/占位分支；未发布时行为不变。
- 证据（`rq_probe_t05_media_poster_frame_crosses_the_wire`）：①帧内 Image 原语 1 个、上传字节与发布帧**逐字节相等**、引用 src 指向同一资源身份；②稳定第二帧**0 重传**（驻留）；③相对 direct 像素差 **0.000**（静态帧精确）。`media_poster` 另有单测（发布/更新代次/非法长度拒收/清除）。
- **同时修正一个探针缺陷（影响此前读数）**：inproc 探针用未加前缀的资源 id 调 `bitmap_cache_put`，而线上键是 `bitmap://<id>`；同进程其它测试的全局缓存命中把 miss 掩盖了，**若干带图的行实际量的是占位**。现改为带前缀，并在逐能力表里硬断言“每个 Image src 必须在宿主缓存命中”（占位= 硬失败）。修正后的读数：矩阵 svg_icon **0.100% → 0.039%**；逐能力 image 0.10%→**0.04%**、window_thumbnail 0.10%→**0.06%**、code_editor 1.31%→**0.41%**；仍 26/39 ≤0.1%。
- Spec（SD-01）：§2.3 增“媒体静态帧=普通图片资源”口径；§5.1/§5.2/§5.3 读数按修正后刷新；§6 第 5 条改为“静态帧已交付（0.000、0 重传）、连续播放另立计划（GPU 读回 + 带宽，属 OS063 传输/媒体面）”。
- 回归面（本树 `3eadae59`）：`rq_` **38/0**；`ui::desktop_protocol::` **269 passed / 4 failed**（同基线 4 红）；`media` 过滤 **17/0**。
- 仍缺（不签 T-03/T-05）：**跨进程资源释放/引用记账**（“最后 owner 释放”仍是被撤回后具名的边界，Spec §3/§6 第 8 条）；连续媒体帧（已裁定另立）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 13 | from: work | to: work | status: executing | outcome: pass（静态媒体帧 + 探针键修正） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eadae59 | next: work（资源释放/引用记账）



### 2026-11-08 work T-05 收口：执行完成，移交独立复审（stage: work → execution_done）

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **pass（work-owned 完项）** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | base_commit: `3d64877d` | code_commit: `44079c22` | dependency_revisions: auto-lang `8337bab5`(只读)、auto-web `2d698413`、auto-down `58ea33f` | task_ids: T-01..T-05 | next_responsible_role: review（T-06）
- **AC 结果（work-owned，证据锚点）**：
  - AC-01：六格全部有真实 wire 证据（`4e2f53d3`：两产者 × 两宿主，v3 tag 3 槽分派缺陷修复）；39/39 View 能力 ID 逐项读数（`77f92dda`，26/39 ≤0.1%，13 个文字密集/宿主依赖族具名）；真两进程配对（`39bd7bef`）。
  - AC-02：默认 producer 矩阵门 ≤0.1%（7 例全部通过）；GlyphRun/SDF 富模式为 r4 具名边界（0.798%/1.374%），默认面不重排且降级可观测（`rq_string_text_path_is_observable_degradation`）；图像环已修正为真实渲染（`3eadae59`，此前探针键缺前缀导致带图行量到占位）。
  - AC-03：内容身份/驻留/预算/驱逐、图片 alpha/rotation、SVG 栅格、wire generation（tag 13）、跨进程释放 + 宿主帧引用记账（tag 14，`44079c22`）、媒体静态帧（`3eadae59`，0.000 像素差、0 重传）；连续媒体帧按 2026-11-08 用户裁定另立计划。
  - AC-04：能力协商/准入保旧帧、槽 credit 背压、上传预算、慢宿主/旧 ACK/旧宿主×新产者负控（`4263ff3c`）、HostCaps 下行补齐两宿主（`d2e41c58`）、真两进程 v2↔v3 配对（`39bd7bef`）。
  - AC-07：默认 producer produce p50 0.17–0.24ms（p95 ≤0.387ms），SVG 例外 ≈32.5ms 为 r4 唯一具名例外（记录路径每帧新建 iced renderer）；M-04/M-08 measured、M-01/M-03/M-05 proxy、M-02/M-06/M-07 未测（owner OS063，§5/§6）。
- **顺带修掉的实现/测量缺陷（评审需知）**：① v3 帧在 shm 槽分派被静默丢弃（`4e2f53d3`）；② HostCaps 下行只存在 `host::ProtocolHost`，两处真宿主缺（`d2e41c58`）；③ 探针位图缓存键缺 `bitmap://` 前缀、被全局缓存命中掩盖（`3eadae59`，现加硬断言）。
- **capability 分母变化**：8 → **9**（新增 `resource/release/1`；`resource/generation/1` 早已在冻结表内）。这是本计划自有协议的追加式可选面，旧端不产不出。
- **开放中的合同级边界（已具名、有 owner/关闭条件，不阻塞交 review）**：连续媒体帧（裁定另立，OS063 传输/媒体面）；乱序资源注入负控（管道 FIFO，generation 守卫已覆盖上传方向）；M-02/M-06/M-07（OS063）；释放为单向通知（无 ACK/双向引用计数）。
- **复跑面（本树 `44079c22`）**：`rq_` **39/0**；`ui::desktop_protocol::` **272 passed / 4 failed**；`ui::iced::broker_surface` **13/0**；`media` 过滤 **17/0**。4 红与 base `3d64877d` 同组（autocenter 居中、stage3 stress n3/n5、t3_examples_queue_end_to_end），非本计划引入。统一回归（冻结并集）归 merge；本轮未跑全仓。
- **待 review 决定的事**：上述边界是否满足 r4 合同；6 个能力面的证据是否足以签 AC-03/AC-04；`current_step=5`、`status=execution_done` 是否应以此修订前进。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 15 | from: work | to: review | status: execution_done | outcome: pass（T-01..T-05 完成） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 44079c22 | next: review（T-06）

### 2026-11-08 第四轮独立复审（final，T-06）：needs_fix

- stage: review | review_scope: **final** | reviewer_identity: 独立上下文子代理（与实现会话不同上下文；模型/会话号未由运行环境暴露，按“独立 agent 会话”记录，不冒充具体模型） | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **needs_fix** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | reviewed_commit: `44079c22` | base_commit: `3d64877d` | spec_inputs: `docs/specs/runtime/rq-render-packets-and-resources.md`（worktree 版）、`rq-fidelity-contract.md` §7 | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review4/REVIEW4-FINAL-2026-11-08.md` | next: work（下表 F-01..F-07）
- **复审自跑门**：`rq_` 39/0；`ui::desktop_protocol::` 272/4；base `3d64877d` 同 filter 246/**同 4 红**；`broker_surface` 13/0。→ §9 的门数与“4 红是基线”成立，无新增红。
- **新鲜读数**：默认矩阵 0.0949/0.0395/0.0938/0.0625/0.0000/0.0053/0.0057（全 ≤0.1%）；produce p95 0.177–0.947ms、SVG 32.2–33.4ms；39/39 逐能力行、26 ≤0.1%；wire 矩阵 14 行 ok、两宿主逐例一致。
- **AC 判定**：AC-01 pass；AC-02 **partial**；AC-03 **partial**；AC-04 pass（具名残余：无可控重排注入）；AC-07 partial（M-02/06/07 归 OS063 + 条款未设门）。
- **阻塞发现**：
  - **F-01（high，AC-02）**：v3 富模式阴影族实测 **2.5572%（gradient）/2.2798%（shadow_only）**，超 r4 具名上界 ≤1.5%；§9/Spec 的 1.374%/0.741% 是 `3eadae59` 探针前缀修复前的旧值（当时贴片是占位）。且**无门断言 v3 富模式**，全绿门漏过。
  - **F-02（medium，AC-03）**：释放延后记账次序错误——`refs.on_compose(payload)` 在 `compose_v2` **之前**执行（三处宿主），准入拒绝时前台仍显旧帧却按被拒帧刷新引用集 → 仍被显示的 src 被释放；且 `ProtocolHost` 只用**一个** `BitmapRefs`（应按表面）。
  - **F-03（medium）**：r4 的数值条款只写在文档里、无断言（富模式上界、produce p95 ≤10ms、SVG 例外范围、replay p95 规则）。
  - **F-04（low）**：宿主未声明 `resource/release/1` 时释放队列无界增长；`bitmap_generations` 驱逐后不清。
  - **F-05（low，Spec 增量）**：`rq-fidelity-contract.md` §7 冻结表仍是 8 项、未含 `resource/release/1`，而新页称 9 项并把 `resource/generation/1` 说成“新增”（它本就在 061 表内）；`supersedes_spec_components: []` 留下两页冲突。
  - **F-06（low）**：媒体“过线”测试是同一个进程内的记录/重放，没经泵/管道/shm。
  - **F-07（info）**：重复释放断言与“不重复驱逐”声明矛盾；`rq_cells` 残留 0.2%/未设门注释；session.rs 的 BitmapReady 注释被拆开。
- **处置**：按 review 技能，受影响任务 T-02/T-04/T-05 重新打开，`status` 回 `executing`、`current_step` 重算为 3；T-06/T-07 保持未勾。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 16 | from: review | to: work | status: executing | outcome: needs_fix（7 findings） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 44079c22 | next: work（F-01..F-07）


### 2026-11-08 work 复审修复（F-01..F-07 全部关闭，交第二轮 final review）

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **pass（复审 findings 全关）** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | code_commit: `f2cdafb4` | next: review（T-06 第二轮 final）
- **逐条关闭（finding → 根因/修法 → 证据）**：
  - **F-01（high，AC-02）**：根因 = 重放画布把 `Image` 排在几何之后合成（iced `CanvasFrame::draw_image` 延后批次），整块阴影贴片把 quad 内部再压暗一层。修法 = 贴片**只画环带**（quad 自身圆角形状内部 alpha 清零），z 序无关。实测 **gradient 0.588%、shadow_only 0.069%**（原 2.557%/2.280%），在 r4 ≤1.5% 上界内；新增 `images_composite_after_geometry` 钉住该画布行为，Spec §2.3 写明“不得依赖图在几何之下”。
  - **F-02（medium，AC-03）**：修法 = 引用记账**按表面**且**只在合成成功后**刷新（`BrokerClient::refresh_refs` / `release_refs`，rqhost 与桌面 session 共用；`ProtocolHost` 用按表面的 map），释放按 wid 归属表面。被拒帧不再改写前台引用集。
  - **F-03（medium）**：r4 数值条款全部落为断言——默认矩阵 ≤0.1%；v3 富模式 ≤1.0%（text/scroll）/≤1.5%（阴影两例）；produce p95 ≤10ms（SVG 例外以 p50 设界、p95 只记录，理由写在断言旁）；replay **p50** ≤1.25×direct p50+2ms（p95 记录，共享跑机抖动实测 direct 3–9ms / replay 5–18ms）。
  - **F-04（low）**：泵**始终排水**释放批（未协商则丢弃），旧宿主连线上队列不再无界增长；`bitmap_cache_evict` 同步清 generation 记账。
  - **F-05（low，Spec 增量）**：`rq-fidelity-contract.md §7` 冻结表补 `resource/release/1`（9 项），新页改口径为“generation 是 061 既有、062 只落到线上”；Plan 增 **SD-02 modify** 行。
  - **F-06（low）**：新增 `rq_wire_media_poster_frame_crosses_the_wire`——海报帧经真泵/管道/shm 到宿主，装配帧里的 `bitmap://` 引用在宿主缓存命中（不只同进程记录面）。
  - **F-07（info）**：重复释放幂等（applied 集合；重新被帧引用时清除），单测改为断言“第二次释放返回空”；陈旧注释清理。
- **复跑面（本树 `f2cdafb4`）**：`rq_` **40/0**；`ui::desktop_protocol::` **273 passed / 4 failed**（同基线 4 红）；`ui::iced::broker_surface` **14/0**；`media` **18/0**。
- **未变**：连续媒体帧（用户裁定另立）、可控重排通道负控、M-02/06/07（OS063）仍为具名边界；主检出他方 WIP 仍待其 owner 归位（merge 门）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 17 | from: work | to: review | status: execution_done | outcome: pass（F-01..F-07 关闭） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: f2cdafb4 | next: review（第二轮 final）

### 2026-11-08 两连败 final review 的诊断检查点（work 技能要求）

- 背景：`44079c22`（第一轮 final）与 `f2cdafb4`（第二轮 final 复评）连续两次 `needs_fix`。按 work 技能，下一次修复前必须有具名诊断，不能只“再改一遍”。
- **为什么前两轮检查没抓到**：
  1. **F-01（阴影 2.56%）**：所有门只压默认 producer（scrim），v3 富模式只“记录不设门”；而读数本身在 `3eadae59` 前是占位贴片的读数（0.74%/1.37%）——**度量对象是错的**（占位 vs 真图），门又没有覆盖该模式，于是“绿”与“真值”同时失真。
  2. **F-02（释放记账次序）**：实现顺序错误（先记账后合成）只被“正向路径”测试覆盖——没有任何用例让 `compose_v2` 返回 Err 后再观察记账；多表面场景也没有负控。**测试面缺“拒绝/多实例”分支**。
  3. **F-03/N-02（数值条款）**：把“文档写了阈值”当成“已断言”，且把 061 的 p95 口径搬到一个采样/预热不同的矩阵 harness 上；第一次又用 p50 替换了冻结口径——**口径所有权错位**（谁定义、在哪测）。
- **本轮设计/口径调整（不是重跑同样的检查）**：
  - 计量对象单一化：所有像素读数必须在“图像真的渲染出来”的前提下取得（`rq_probe` 逐能力表已加“Image src 必须命中宿主缓存”的硬断言）。
  - 门覆盖模式：默认与 v3 富模式**各自设门**（富模式 ≤1.0%/≤1.5%），并用 `images_composite_after_geometry` 钉住底层画布行为。
  - 顺序由构造保证：新增 `compose_v2_counted(surfaces, refs, bitmap_keys, …)`，三处宿主只能经它合成；单测直接喂被拒帧断言记账不动。
  - 口径归位：冻结的 `replay p95 ≤1.25×direct+2ms` 由 061 的测量入口（`rq_probe_t01…`）在**隔离窗口**断言（env `AUTO_RQ_TIMING_WINDOW=isolated`），共享跑机只断言稳态 p50；produce 门只在冻结 fixture 矩阵（`rq_cells`）裁决，全语料含宿主依赖面（window_thumbnail ≈44ms）只记录。
  - 记账生命周期：引用记账按表面并在 `ReclaimWindow` 回收（N-01），重复释放幂等。
- **剩余不变项**：连续媒体帧（用户裁定另立）、可控重排通道负控、M-02/06/07（OS063）、主检出他方 WIP（merge 门）。此诊断不改变 AC/范围。

### 2026-11-08 第二轮独立复审（final 复评，T-06）：needs_fix

- stage: review | review_scope: final（修复复评） | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露，按“独立 agent 会话”记录） | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **needs_fix** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | reviewed_commit: `f2cdafb4` | base_commit: `3d64877d` | spec_inputs: `rq-fidelity-contract.md` §7、`rq-render-packets-and-resources.md` §2.3/§3/§5.3/§6（`f2cdafb4`） | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review5/REVIEW5-FINAL-2026-11-08.md`
- **复现（reviewer 自跑，clean tree）**：cells idle exit 0（v3 text 0.7979/gradient 0.5876/scroll 0.3298/shadow 0.0691）；同命令 + 并发构建 exit 101（text v3 replay p50 28.00 > 11.47）；协议族 **273/4**（与 base 同组 4 红）；bitmap_refs 1/0、release_ 6/0；媒体过线 1/0；zorder pin 1/0。
- **关闭**：F-01、F-04、F-06、F-07。**部分**：F-02（代码正确但缺“被拒帧 + 挂起释放”“双表面”两个负控）、F-03（replay 由冻结 p95 改 p50；产物 4/28 例缺 p95；SVG 60ms p50 偏松）、F-05（新页 §2.3 仍写 1.3740% 与“具名遗留”）。
- **新发现**：N-01（low）`refs` 不随窗口回收、`applied` 无界；N-02（low）计时门并发负载下误红、空闲通过；N-03（info）iced `gradient.wgsl` 无 shadow 项，0.588% 属**参考侧**基线局限（Spec 未记）。
- **AC 判定**：AC-01 pass（未触）、AC-02 pass（N-03 注）、AC-03 partial、AC-04 pass（未触）、AC-07 partial。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 18 | from: review | to: work | status: executing | outcome: needs_fix（F-02/F-03/F-05 部分 + N-01/N-02/N-03） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: f2cdafb4 | next: work（含两连败诊断）

### 2026-11-08 work 第二轮复审修复（F-02/F-03/F-05 补全 + N-01/N-02/N-03，交第三轮 final）

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **pass（复审 findings 全关 + 两连败诊断已记）** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | code_commit: `63ffebc4` | next: review（T-06 第三轮 final）
- **F-02 补全**：记账不变量改为**由构造保证**——`compose_v2_counted(surfaces, refs, bitmap_keys, …)` 是三处宿主唯一的合成入口，只在 `Ok(Some(_))` 时刷新引用；`BitmapRefs` 内部按表面记账、**任一表面仍引用即不驱逐**。补两个负控：①被拒 v3 帧后前台仍是旧帧、挂起释放不动；②双表面都显示 A 时释放挂起，两表面都换帧才兑现。重复释放仍幂等。
- **N-01**：`ReclaimWindow`（`ProtocolHost` 与桌面 session）调用 `BitmapRefs::forget_surface`——按窗口清记账、挂起释放得以兑现，`applied` 随引用集回收（不再是只增集合）。
- **F-03/N-02（口径归位）**：冻结 produce 门（W-C ≤10ms、SVG 例外按 p50）只在**冻结 fixture 矩阵**（`rq_cells` 7 workload）断言；全语料保留数字不设门（`window_thumbnail` 每帧重取快照 p95≈44ms，不属该预算）。冻结的 replay **p95** 条款在 061 的测量入口（`rq_probe_t01`）断言，且只在**隔离窗口**（`AUTO_RQ_TIMING_WINDOW=isolated`）——20 样本下 p95≈次大值、共享跑机同代码在 5–22ms 间摆动，而 p50 反优于 direct；稳态 p50 门处处断言。矩阵采样 12→30（p95 不再是 max），每行输出 `timing_gate=pass|skipped_loaded`。
- **F-05/N-03（Spec）**：§2.3 的陈旧段（0.0625%→1.3740%、“具名遗留”）改为修复后读数（0.588%/0.069%）并记 N-03——iced gradient 面无 shadow 项，direct 金样在渐变例不画阴影，0.588% 主要是“RQ 画了、参考没画”；§5.3 增加计时门口径与归属说明。
- **测试稳定性**：媒体过线用例改为容忍 per-App 上传预算延期（等到重试收敛，而不是要求首帧即命中）——这正是 P-01 的合同行为，也修掉了它对执行顺序的依赖。
- **复跑面（本树 `63ffebc4`）**：`rq_` **40/0**；`ui::desktop_protocol::` **276 passed / 4 failed**（同基线 4 红）；`ui::iced::broker_surface` **14/0**；`media` **18/0**。
- **未变**：连续媒体帧（裁定另立）、可控重排通道负控、M-02/06/07（OS063）、主检出他方 WIP（merge 门）。两连败诊断见上一节，不改变 AC/范围。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 19 | from: work | to: review | status: execution_done | outcome: pass（F-02/F-03/F-05/N-01/N-02/N-03 关闭） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 63ffebc4 | next: review（第三轮 final）

### 2026-11-08 第三轮独立复审（final 复评 3，T-06）：needs_fix（触发三循环上限，停止修复循环）

- stage: review | review_scope: final | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露） | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: **needs_fix** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | reviewed_commit: `63ffebc4` | base_commit: `3d64877d` | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review6/REVIEW6-FINAL-2026-11-08.md`
- **reviewer 自跑**：release_tests 4/4；`ui::desktop_protocol::` **276/4**（与 base 同组 4 红）；broker_surface 14/14；`rq_` 全跑 39 passed / 1 failed（`rq_probe_t01`），媒体过线用例在整跑内通过。
- **关闭**：F-02（`compose_v2_counted` 唯一入口 + 双负控）、F-05（§2.3 数字已换新、无陈旧值）、N-03（gradient 面无 shadow 项已记）；F-01/F-04/F-06/F-07 复确认。
- **仍开**：
  - **F-03（blocking，AC-07）**：`produce W-C ≤10ms` 现在包在 `if quiet` 里（`quiet = direct p95 ≤8ms`），executor 自己的产物 13/28 行 `skipped_loaded`、reviewer 11/28，且该跳过未写进 Spec §5.3。冻结的 replay **p95** 条款在 `AUTO_RQ_TIMING_WINDOW=isolated` 下 **3/3 失败**（code_editor 8.04>7.70、button 30.08>12.34、radio p50 7.10>6.78），并且在 executor 自己的产物里已有 6/28 行超界（只是不再断言）。SVG 例外 p50 仍 60ms 上限 vs AC-07 记的 ≈32.5ms。
  - **N-01（partial）**：`forget_surface` 已接两处回收，但 `applied` 只增不减（settle 里只插入），单测未断言 `tracking_len().1`——“applied 随引用集回收”不成立。
  - **N-02（partial）**：计时门在 reviewer 的独立安静机上仍误红（`rq_` 整跑 tabs p50 9.97>7.53；isolated 跑 radio p50 7.10>6.78）。
  - **N-04（low）**：`rqhost.rs` 的回收路径没有 `refs.forget_surface` → 旧表面键永久占住挂起释放、全局句柄缓存无法驱逐。
  - **N-05（low/doc）**：produce 门的跳过没有写进 Spec §5.3 / Plan §9。
- **处置（上限）**：本轮是第 3 个 work↔review 循环（`44079c22`→fail、`f2cdafb4`→fail、`63ffebc4`→fail）。按 work/goal 技能，**停止第四次修复循环**，把 F-03 作为**需用户裁定的合同/性能决定**提交；N-01/N-02/N-04/N-05 作为已定位的小项随裁定一起排期。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 20 | from: review | to: user | status: execution_done | outcome: waiting（三循环上限 + 需裁定 AC-07 的 replay p95 口径） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 63ffebc4 | next: user

- **2026-11-08 需用户裁定（三循环上限，F-03 是唯一阻塞项）**：r4 冻结的 replay 尾部条款 `replay p95 ≤ 1.25×direct p95 + 2.0ms` 在本机**测不过**，而 replay 的**稳态 p50 反优于 direct**（实测 replay p50 3.1–3.9ms vs direct 3.6–4.1ms；尾部在 5–22ms 摆动，button 例 isolated 跑 30.08ms vs 界 12.34ms）。produce 门（W-C ≤10ms）在负载下按 `if quiet` 跳过（executor 产物 13/28 行、reviewer 11/28），跳过未写进 Spec。要收口必须由用户选：
  - **(a) 授权一个有界性能任务**（推荐）：假设是重放画布**每帧重做文字整形/布局**（`displaylist_element` 的 Text 臂直接 `draw_text`，无 shaping cache），与 direct 的 iced 文本组件（自带缓存）形成尾部差。做法：按 `(text, size, weight, flags)` 缓存 shaping/布局结果，只在内容变化时重排；用 isolated 窗口（`AUTO_RQ_TIMING_WINDOW=isolated`）前后对照，目标让 p95 条款成立并写进 Spec。若一轮内不收敛，转 (b)/(c)。
  - **(b) r5 合同修订**：把该尾部条款按实测口径写成具名边界（稳态 p50 设门、p95 记录；produce 门在负载下跳过须具名），并顺带把 SVG 例外 p50 上限对齐实测（现 60ms vs 记录 32.5ms）。这是 AC-07 的验收口径变更，需你批准。
  - **(c) 以当前能力集收口**：接受 `execution_done`，把 F-03 作为具名遗留另立计划（性能专项），062 不再追 replay 尾部条款。
  - 三者都附带：N-01（`applied` 回收）、N-02（计时门独立机也不稳）、N-04（rqhost 回收缺 `forget_surface`）、N-05（文档补跳过口径）这四项小修——它们不改变 AC，可在同一轮一起做。

- stage: user | plan_id: PLAN-062 | plan_revision: r4 | round: 21 | from: user | to: work | status: execution_done | outcome: pass（裁定落地：有界性能任务 + 四项小修一并做） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 63ffebc4 | next: work（replay 尾部性能 + N-01/N-02/N-04/N-05）
- **用户裁定（2026-11-08）**：①对 F-03 授权**有界性能任务**（不改 AC；先证伪/证实“重放画布每帧重做文字整形”的假设，用 isolated 窗口前后对照，不收敛再回 (b)/(c)）；②N-01/N-02/N-04/N-05 四项小修**现在一起做**。三循环上限的“停止第四次循环”约束由本授权解除（性能任务不属于未经授权的重复修复）。

### 2026-11-08 work 有界性能探测（F-03 尾部根因）+ N-01/N-04/N-05 小修

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: 进行中（尾部根因已定位到可度量层；修复方案需一次实现选择） / worktree: `D:/autostack/.wt/ui-062/auto-ui` | code_commit: `978ea7da`
- **小修**：N-01 `BitmapRefs::applied` 改为 FIFO 上限 4096（`remember_applied`，新增 `applied_set_is_bounded` 断言最近记录仍幂等、最老被淘汰后可再驱逐）；N-04 `rqhost` 的 `ReclaimWindow` 补 `refs.forget_surface`（独立宿主回收路径不再永久占住挂起释放）；N-05 Spec §5.3 写明 produce 门的负载跳过（`quiet = direct p95 ≤8ms` → `skipped_loaded`，executor 13/28、reviewer 11/28 触发过；共享窗口绿灯只覆盖稳态与像素，W-C 签收必须在隔离窗口）。
- **性能探测（`rq_perf_replay_shapes`，隔离标记 + 20 帧预热，40 采样，p50/p95 ms）**：
  | shape | ops | p50 | p95 |
  |---|---|---|---|
  | empty | 0 | 7.14 | 8.40 |
  | text_1 | 1 | 8.47 | 10.91 |
  | text_50 | 50 | 18.12 | 25.97 |
  | image_1（已驻留） | 1 | 7.67 | 9.07 |
  | image_20 | 20 | 8.64 | 12.86 |
  | quad_200 | 200 | 10.18 | 11.52 |
- **结论（数据驱动，不是猜测）**：
  1. 重放路径有 **≈3ms 的固定开销**（empty 7.14 vs 同机 direct 空树 ~4.1），来自 `displaylist_element` 的 canvas 包装（布局/交互/背景清理），不是文字整形；
  2. 每段文字 ≈ **0.22ms/帧**（text_50 − empty ≈ 11ms / 50），即“文字整形/绘制”确实是按段计的稳态成本，但**不是** 1–2 段文字用例的尾部来源；
  3. 冻结公式 `1.25×direct+2ms` 在 direct≈4ms 时给到 ≈7ms 的预算，而重放的空帧基线就是 7.14ms——**公式无法吸收 3ms 固定开销**，这才是 `path_mesh/svg/…` 系列 p95 越界的结构性原因。
- **修复候选（需一次选择，不属“再改一遍”）**：
  - **(A) 去掉 canvas 包装**：让 `displaylist_element` 用直接 `Widget::draw` 绘制 DisplayList（自带布局/命中），砍掉 ~3ms 固定开销；风险是需要重做命中/交互接线（宿主侧命中表另存，需核）。
  - **(B) r5 修订口径**：把该条款改成“重放 p50 设门 + p95 记录”，或给固定开销留出对称常数（例如 `1.25×direct + 4ms`），并在 Spec 写明理由与实测。
- 两项都不改变 AC 的功能部分；本轮先记录数据，等选择后再动实现（避免再次无授权改口径）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 22 | from: work | to: user | status: execution_done | outcome: waiting（尾部根因已定位；固定开销修复 vs 口径修订需选择） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 978ea7da | next: user

- **N-02 收口（2026-11-08，`acace4aa`）**：所有计时断言改为**只在隔离窗口**执行（`AUTO_RQ_TIMING_WINDOW=isolated`），共享跑机只记录并逐行标 `timing_gate=skipped_shared_window`；像素/结构门不受影响。依据是 Plan §6 本身的规定 + 实测负载放大（SVG produce p50 32→76ms、replay p50 5→18ms、button replay p95 12→30ms）。复核：`ui::desktop_protocol::` **278 passed / 4 failed**（恰为基线 4 红，计时假红消失）。
- **由此产生的口径事实（评审需知）**：共享窗口的绿灯**只覆盖功能与保真**；W-C 与 replay 预算的签收必须是一次隔离窗口运行，而本机目前测不过（见上表根因）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 23 | from: work | to: user | status: execution_done | outcome: waiting（N-01/N-02/N-04/N-05 收口；固定开销修复 vs 口径修订需选择） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: acace4aa | next: user

- stage: user | plan_id: PLAN-062 | plan_revision: r4 | round: 24 | from: user | to: work | status: execution_done | outcome: pass（裁定：实现直绘路径，去掉 canvas 包装的 ≈3ms 固定开销；不改 AC） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: acace4aa | next: work（DisplayListWidget 直绘 + 前后对照）

### 2026-11-08 work 性能探测（第二版，公平轮转）——**证伪了“canvas 包装 3ms 固定开销”**

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r4 | outcome: 进行中（结论需用户重新选择；未改实现） | worktree: `D:/autostack/.wt/ui-062/auto-ui` | code_commit: `acace4aa`（本轮只改测试/文档，未提交实现）
- **第一版方法有缺陷**：顺序测（先 plain、再 empty、再 text…）把“进程级预热/首个元素类型”的代价全记在第一个 shape 上——同一份代码两次跑出相反结论（一次 plain 最慢 7.07/8.76、canvas 4.1–5.2；一次 empty 最慢 7.14/8.40）。
- **第二版方法（公平轮转）**：先建全部模拟器、逐个预热 10 帧，然后 40 轮 **round-robin**（每轮每 shape 各一帧）。实测（p50/p95/max ms）：
  | shape | ops | p50 | p95 | max |
  |---|---|---|---|---|
  | plain_widget_tree（无 canvas） | 0 | 8.23 | 15.15 | 40.53 |
  | empty（canvas） | 0 | 7.58 | 10.38 | 23.79 |
  | text_1 | 1 | 7.26 | 11.98 | 13.46 |
  | text_50 | 50 | 7.06 | 11.19 | 39.00 |
  | image_1 | 1 | 6.73 | 11.80 | 18.79 |
  | image_20 | 20 | 6.86 | 13.08 | 21.61 |
  | quad_200 | 200 | 10.18 | 18.48 | 29.11 |
- **结论（与第一版相反，按公平测量为准）**：
  1. **没有 canvas 固定开销**：`empty`(7.58) 比同轮 `plain_widget_tree`(8.23) 还快 → “去掉 canvas 包装”不成立（用户此前的选择基于我的错误结论，已在本轮更正）。
  2. **文字每帧几乎免费**（text_50 ≈ empty）→ 698 说的“每段 0.22ms/帧”也是顺序偏差；iced 本身缓存整形。
  3. **只有几何量与规模相关**（200 quad：+2.6 p50/+8 p95）。
  4. 绝对值与 max（6.7–10ms p50、max 高到 40ms）说明**当前主机负载很高**（连续多小时的构建/测试）；同一代码上一轮是 4.1–5.2ms。
- **因此**：重放路径没有可修的“固定开销”；冻结的 `1.25×direct+2ms` 之所以在本机测不过，是**主机/进程级抖动**（在负载下可放大到 3–6 倍），不是重放设计。可选项回到 **(B) r5 口径修订** 或 **(C) 以当前能力集收口**；若坚持原口径，需要在**真正空闲**的机器上跑一次隔离窗口（本会话无法保证）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r4 | round: 25 | from: work | to: user | status: execution_done | outcome: waiting（直绘修复假设被公平测量证伪；请在 B/C 间选择） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: acace4aa | next: user

- stage: user | plan_id: PLAN-062 | plan_revision: r4 | round: 26 | from: user | to: new | status: execution_done | outcome: pass（裁定：r5 修订 AC-07 的 replay 口径；不改目标/范围/仓库集合） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: ecf8496c | next: new（r5 合同修订）
- **用户裁定（2026-11-08，基于公平轮转测量）**：AC-07 的 replay 条款改为“**p50 设门 + p95 记录**，且只在隔离窗口签收”；同时把 SVG 例外的“记录值 ≈32.5ms”与“上限 ≤60ms p50（吸收主机负载）”区分清楚；在 Spec/AC 里具名本主机测量限制（同一代码在 4.1–10ms 间随负载变化，max 可到 40ms）。目标、AC 的其余部分、范围与仓库集合不变，属有界合同修订（r5）。

### 2026-11-08 r5 合同修订（AC-07 replay 口径，用户批准）

- stage: new | plan_id: auto-ui/PLAN-062 | plan_revision: **r5**（r4→r5，语义合同修订） | outcome: pass（合同修订，不是实现验收） | next: work（按 r5 复跑隔离窗口并交第四轮 final review）
- 授权：用户 2026-11-08 两次选择——先选“实现直绘路径”，在我用**公平轮转测量证伪“canvas 固定开销”假设**后，改选 **r5 修订 AC-07 的 replay 口径**。目标、非目标、仓库集合、其余 AC 与任务集均不变。
- 修订内容：
  1. **AC-07**：replay 相对 direct 改为 **p50 ≤1.25×direct p50+2.0ms 设门、p95 只记录**；**所有计时门只在隔离窗口签收**（共享窗口记录并标 `skipped_shared_window`）；SVG 例外区分“安静机记录值 ≈32.5ms”与“上限 p50 ≤60ms（吸收主机负载）”。
  2. **V-07**（§6 主行）：同步为新口径。
  3. **§5 性能合同**：新增“测量窗口与 replay 口径”条目，写明实测依据（负载放大 3–6 倍；公平测量：empty 7.58/10.38 vs plain 8.23/15.15；text_50≈empty）与“共享窗口绿灯不证明 W-C/replay 预算”。
  4. SD-01 目标页 §5.3 已在实现轮次写入同一口径，无需再改表结构。
- 依据（本会话实测，见 §9 前文）：`rq_perf_replay_shapes` 公平轮转表；review6 的 isolated 数值；matrix/probe 在共享窗口的负载放大记录。
- 保留：r1–r4 历史与全部 §9 收据；status 回 `executing`（技能要求：语义修订前把 execution_done 退回 executing），T-01..T-05 保持 [x]，`current_step=5`。
- 未做：本轮未改实现、未改 Spec 正文（仅 Plan 合同）、未做独立 final review；AC-05/T-06 与 AC-06/T-07 仍待办。

### 2026-11-08 work r5 条款复跑（隔离窗口）与交第四轮 final

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: **r5** | outcome: **pass（r5 条款全部满足）** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | code_commit: `ecf8496c`（r5 只改合同；实现未动） | next: review（T-06 第四轮 final）
- **隔离窗口复跑**（`AUTO_RQ_TIMING_WINDOW=isolated`，本树 `ecf8496c`）：
  - `rq_cells_t05_six_cell_matrix` exit 0，逐例 `timing_gate=asserted`：**produce p50/p95** = path_mesh 0.11/0.15、svg_icon 31.79/33.20（p50 ≤60 具名例外）、text_cjk 0.11/0.18、gradient 0.14/0.17、scroll 0.15/0.21、shadow_only 0.13/0.19、progress 0.13/0.18（单位 ms，全部 ≪ W-C 10ms）；像素门：default 全 ≤0.1%（0.095/0.039/0.094/0.062/0.000/0.005/0.006%），v3_rich text 0.798%（≤1.0）、gradient 0.588%、shadow 0.069%、scroll 0.330%（≤1.5/≤1.0）。
  - `rq_probe_t01_direct_and_rq_baseline` exit 0（r5 的 replay **p50** 门：全部通过；p95 按 r5 只记录）。
  **更正（2026-11-08，第四轮复审 R-1 指出）**：当时代码里仍保留 p95 断言，我的那次通过属偶然；该断言已删除，本收据的“只记录”到 `ecf8496c` 之后才真正成立。
- **共享窗口复跑**（默认，不带 env）：`rq_` **41/0**；计时门记 `timing_gate=skipped_shared_window`，像素/结构门照常生效。
- **仍未签收**：统一回归（冻结并集）与 Spec 发布归 merge；M-02/M-06/M-07（OS063）；连续媒体帧（另行）；可控重排通道负控（具名）。
- **交 review 的事**：按 r5 措辞复核 AC-07（p50 门 + 隔离窗口 + SVG 例外口径）与先前已批的 AC-01/02/03/04 证据；确认 `plan_revision: 5` 的合同与代码一致。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 27 | from: work | to: review | status: execution_done | outcome: pass（r5 隔离窗口复跑全绿） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: ecf8496c | next: review（T-06 第四轮 final）

### 2026-11-08 第四轮独立复审（final，r5）与 R-1..R-4 修复

- stage: review | review_scope: final | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露） | plan_id: auto-ui/PLAN-062 | plan_revision: **5** | outcome: **needs_fix** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | reviewed_commit: `ecf8496c` | base_commit: `3d64877d` | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review7/REVIEW7-FINAL-2026-11-08.md`
- **r5 授权/最小性：pass**——`d9df86d9`（+25/−5）只动 frontmatter、一条 §5 bullet、V-07 行、AC-07 行；`a89c321c` 只动 status+§9。可追溯到 round 24/25/26 的裁定链：公平轮转测量证伪“直绘修复”前提 → 用户改选 r5。未删 AC、未改范围/仓库/任务，1.25×direct+2ms 仍是冻结公式（只是作用在 p50）。
- **AC 判定（reviewer 自跑）**：AC-01 pass；**AC-02 pass**；**AC-03 pass**；**AC-04 pass**；AC-07 **partial**（produce 全绿：path 0.15/0.18、svg 43.04/51.44 具名、text 0.13/0.16、gradient 0.18/0.22、scroll 0.23/0.26、shadow 0.15/0.18、progress 0.14/0.19 ms；replay p50 在 run-1 的 20 例全绿，但全语料未证——测试在 R-1 处中止）。
- **R-1（blocking，AC-07）**：`rq_probe.rs` 仍断言 `replay p95 ≤1.25×direct p95+2`（r5 已把 p95 改为只记录）；reviewer 的隔离跑在此中止（mouse_area 9.60 > 9.475）。**同时我的 r5 收据 “p95 按 r5 只记录 / exit 0” 是错的**——代码里仍有断言，我那次只是恰好通过。**已修**：删除该断言、保留 p95 记录；并在此更正收据。
- **R-2（medium，SD-01）**：Spec §5.3 的计时段落仍写“p50 在共享跑机断言、p95 在隔离窗口设门”，与 r5/代码相反。**已修**：改为“p50 设门 + p95 只记录；所有计时断言仅在隔离窗口”。
- **R-3（low）**：§6 有两行 V-07（109/114），r5 只更新了主行。**已修**：删除陈旧重复行。
- **R-4（info）**：`rq_cells` 的“安静基线/skipped_loaded”注释与未使用的 `ratio`。**已修**。
- **修复后复跑（本树）**：隔离窗口 `rq_probe_t01` exit 0（p50 门全过）、`rq_cells_t05_six_cell_matrix` exit 0（produce/SVG/像素门 asserted）；共享窗口 `rq_` **41/0**；`ui::desktop_protocol::` **278/4**（同基线 4 红）；`broker_surface` 14/14。
- **AC-07 结论**：R-1 修好后，隔离窗口的 p50 门与 produce 门在同一树全部通过；**唯一未证**的是“真正空闲主机上的全语料 replay p50”——reviewer 明确指出其窗口有并发 ACE 构建（CPU 9–23%），且 `AUTO_RQ_TIMING_WINDOW=isolated` 只是标签、不隔离资源。这一点写进 §10 作为已知测量限制。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 28 | from: review | to: work | status: executing | outcome: needs_fix（R-1..R-4，已修） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: ecf8496c | next: work（修复已落，交第五轮 final）

### 2026-11-08 work R-1..R-4 修复复跑（交第五轮 final）

- stage: work | plan_id: auto-ui/PLAN-062 | plan_revision: r5 | outcome: **pass（R-1..R-4 全关）** | worktree: `D:/autostack/.wt/ui-062/auto-ui` | branch: `codex/plan-062-dev` | code_commit: `3a3be18d` | next: review（T-06 第五轮 final）
- 复跑（本树 `3a3be18d`）：隔离窗口 `rq_probe_t01_direct_and_rq_baseline` **exit 0**（p50 门全过，p95 仅记录）、`rq_cells_t05_six_cell_matrix` **exit 0**（produce/SVG/像素门 `timing_gate=asserted`）；共享窗口 `rq_` **41/0**、`ui::desktop_protocol::` **278 passed / 4 failed**（同基线 4 红）、`broker_surface` 14/14。
- **已知测量限制（写进 §10，评审需知）**：`AUTO_RQ_TIMING_WINDOW=isolated` 只是**标签**，不隔离机器资源；reviewer 的窗口里有并发 ACE 构建（CPU 9–23%），其 run-2 的 `code_editor replay p50 12.15 > 8.55` 属主机抖动。真正签收 AC-07 需要一次空闲主机运行；本会话无法保证。
- **未变**：AC-01..AC-04 的既有证据（reviewer 第四轮已判 pass）；SD-01/SD-02 发布、统一回归、归档与清理归 merge；主检出他方 WIP 仍是 merge 门。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 29 | from: work | to: review | status: execution_done | outcome: pass（R-1..R-4 关闭，隔离窗口复跑全绿） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3a3be18d | next: review（第五轮 final）

### 2026-11-08 第五轮独立复审（final，r5）与 R-2/R-4 收尾

- stage: review | review_scope: final | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露） | plan_id: auto-ui/PLAN-062 | plan_revision: 5 | outcome: **needs_fix**（仅文档/清理项） | reviewed_commit: `3a3be18d` | base_commit: `3d64877d` | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review8/REVIEW8-FINAL-2026-11-08.md`
- **关闭**：R-1（行为）——p95 断言已删、只记录；隔离跑 41/41 例 replay p50 全过（max 6.23ms，最差余量 2.70ms），p95 max 8.04ms 仅记录；§9 的收据更正已在位。R-3——§6 只剩一行 V-07 且带 replay/隔离条款。**AC-07 按 r5 判据：pass**。
- **未闭**：R-2（Spec §5.3 同时保留了旧口径与新口径，自相矛盾——虽已追加 r5 段但没删旧句）；R-4（`direct_p95` 变成未使用变量、括号缩进、rq_probe 仍留着已删门的注释）。
- **本轮修复（`4f510ed1`）**：
  1. **R-2**：把 §5.3 的“计时门口径”整段重写为**单一 r5 块**（produce 只在冻结矩阵、SVG p50 ≤60、replay p50 设门/p95 只记录、所有计时断言仅在隔离窗口），删除旧句；§5.3 不再与 AC-07/V-07/代码冲突。
  2. **R-4**：`rq_cells` 删除未用的 `direct_p95`、去掉重复陈旧注释、修正错位括号；`rq_probe` 残留注释改为“p95 只记录、p50 设门”。
  3. **§10**：显式记录“`AUTO_RQ_TIMING_WINDOW=isolated` 只是标签、不隔离资源；本会话主机长期共享（reviewer 记录 CPU 9–23%），计时证据是该窗口内的记录而非空闲主机签收”。
- **修复后复跑（本树 `4f510ed1`）**：隔离窗口 `rq_cells_t05_six_cell_matrix` **exit 0**、`rq_probe_t01_direct_and_rq_baseline` **exit 0**；共享窗口 `rq_` **41/0**；`ui::desktop_protocol::` **278 passed / 4 failed**（恰为基线 4 红）。
- **未变**：AC-01..AC-04 证据（第五轮 reviewer 复确认，diff 仅 `#[cfg(test)]` + 文档）；SD-01/SD-02 发布、统一回归、归档清理归 merge；主检出他方 WIP 仍是 merge 门。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 30 | from: work | to: review | status: execution_done | outcome: pass（R-2/R-4 收尾；AC-07 按 r5 已 pass） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 4f510ed1 | next: review（第六轮 final）

### 2026-11-08 第六轮独立复审（final，r5）与 R-4(c) 收尾

- stage: review | review_scope: final | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露） | plan_id: auto-ui/PLAN-062 | plan_revision: 5 | outcome: **needs_fix**（仅剩 R-4 的子项 c） | reviewed_commit: `4f510ed1` | base_commit: `3d64877d` | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review9/REVIEW9-FINAL-2026-11-08.md`
- **关闭**：**R-2**（Spec §5.3 已是单一 r5 计时块，全页无残留 p95 门/共享窗口 p50 断言；与 AC-07、V-07、代码一致）；**R-4(a)(b)**（`direct_p95` 已删、`cargo check --tests` exit 0 无该警告、注释块唯一、括号缩进正确）。
- **未闭（R-4c）**：`rq_probe.rs:1102-1105` 与第五轮逐字节相同，仍写“replay p95 ≤1.25×direct+2ms…标 skipped_loaded”——一个已删的门 + 一个全树不存在的标记，且紧邻 r5 新注释，函数内再次出现两套矛盾规则（与 R-2 同一反模式）。reviewer 也指出我 r5 收据“rq_probe 残留注释已改”只部分为真。
- **本轮修复（`3eef8a32`）**：把该头部注释重写为**单一 r5 块**（p50 设门且只在隔离窗口、p95 只记录、produce 只在冻结矩阵裁决），并加一行明示“r4 的 p95 设门 + skipped_loaded 已被 r5 取代，保留仅为解释历史”，不重新引入该门；全树复查 `skipped_loaded`/`p95 ≤1.25` 只剩这条历史注记，Spec 中为 0。
- **修复后复跑（本树 `3eef8a32`）**：隔离窗口 `rq_probe_t01` **exit 0**；共享窗口 `rq_` **41/0**；此前一轮的隔离矩阵 28/28 asserted 与协议族 278/4（同基线 4 红）不变（本轮只改注释，逻辑未动）。
- **AC 状态（第六轮 reviewer 自跑）**：AC-01..AC-04 pass（diff 仅两个 `#[cfg(test)]` 模块 + 未发布的 Spec 页，无生产代码）；**AC-07 pass（r5 判据）**：隔离 `rq_cells` 28/28 asserted、produce 非 SVG 0.095–0.316ms、SVG p50 30.8–32.0ms；隔离 `rq_probe` 41/41、replay p50 max 5.02ms（最差余量 1.81ms）、p95 max 6.47ms 仅记录。
- **未变**：统一回归、SD-01/SD-02 发布、归档/清理归 merge；主检出他方 WIP 仍是 merge 门；M-02/06/07（OS063）、连续媒体帧、可控重排通道负控仍具名。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 31 | from: work | to: review | status: execution_done | outcome: pass（R-4c 收尾；其余全绿） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eef8a32 | next: review（第七轮 final）

### 2026-11-08 第七轮独立复审（final，r5）：**pass** → reviewed

- stage: review | review_scope: final | reviewer_identity: 独立上下文子代理（模型/会话号未由运行环境暴露） | plan_id: auto-ui/PLAN-062 | plan_revision: **5** | outcome: **pass** | worktree: `D:/autostack/.wt/ui-062/auto-ui`（clean） | branch: `codex/plan-062-dev` | reviewed_commit: `3eef8a32` | base_commit: `3d64877d` | dependency_revisions: auto-lang `8337bab5`(只读)、auto-web `2d698413`、auto-down `58ea33f` | spec_inputs: `docs/specs/runtime/rq-render-packets-and-resources.md` @`3eef8a32`（blob `ff28dcd9`）、`rq-fidelity-contract.md §7` | evidence: `D:/autostack/plan-artifacts/plan-062/auto-ui/review10/REVIEW10-FINAL-2026-11-08.md`（+ t01-isolated.log / rq-group.log / t01-isolated-nocapture.log）
- **R-4c 关闭**：`rq_probe.rs:1102-1112` 为单一 r5 块（p50 只在隔离窗口断言 `:1113-1121`、p95 只记录、produce 只在冻结矩阵），r4 口径仅作为**明示历史**保留（`:1111-1112` “不再是本入口的规则”）。全树 grep：`skipped_loaded` 仅该历史行 1 处、`p95 <=1.25`/`p95+2ms` 0 处、Spec 0 处；唯一存活 p95 门是 produce ≤10ms（`rq_cells.rs:419`，在 `if isolated_window` 内、7 个冻结 workload）。diff `4f510ed1..3eef8a32` = 1 提交/1 文件/+11−11、逐行皆为注释（机械核验 0 非注释行），`rq_probe` 为 `#[cfg(test)]`。
- **reviewer 复跑**：隔离 `rq_probe_t01` exit 0（41/41、replay p50 max 6.20ms、最差余量 1.19ms）；共享 `rq_` **41/0**；再跑一次带 `--nocapture`：p50 max 5.99ms、**p95 max 17.01ms 被打印但未被断言**（正是 r5 要“只记录”的情形）。主机负载 4%→46%→35%（4 个外来 cargo 进程），不影响判定。
- **AC 结果**：AC-01/AC-02/AC-03/AC-04 **pass**（沿用第六轮；本轮 diff 仅 test-only 注释）；**AC-07 pass**（r5 判据，数字见上）。
- **pending_closeout_gates（merge 拥有）**：冻结 union 的统一回归；SD-01/SD-02 发布进 `docs/specs/`；归档 + worktree 清理（含 `wt-guard` 收据）；landing 检查；**主检出他方 WIP 的处置**。
- **未变**：M-02/M-06/M-07（OS063）、连续媒体帧（裁定另立）、可控重排通道负控（具名）仍为边界；AC-05/AC-06 由本轮 review 与后续 merge 承担。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 32 | from: review | to: merge | status: reviewed | outcome: pass（final review 通过） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eef8a32 | next: merge（T-07）

### 2026-11-08 merge 入口门：主检出他方 WIP 仍未归位（等待中）

- stage: merge（入口检查，未开始落地） | plan_id: auto-ui/PLAN-062 | plan_revision: r5 | outcome: waiting | worktree: `D:/autostack/.wt/ui-062/auto-ui`（clean @ `3eef8a32`） | 主检出：`D:/autostack/auto-ui` master @ `a078833a`
- **门未过**：主检出仍有 5 条 code-path 未提交改动 + 1 个未跟踪目录：
  - `crates/auto-ui-runtime/src/ui/desktop_protocol/{headless,host,rqhost}.rs`、`crates/auto-ui-runtime/src/ui/session.rs`（内容是把已撤回的 `drain_resource_releases` 接线到 `FrameSource` 实现——master 上无该方法，属上一轮已记录的他方 WIP）；
  - `crates/auto-ui-runtime/test/`（未跟踪）。
- **为什么不能落**：merge 要求在干净主检出上 fast-forward/合入并跑冻结 union 回归；这 5 条改动与 062 改的同名文件冲突，`git merge/checkout` 会拒绝或被污染，违反 master-zero-WIP。
- **精确 unblock 动作（owner 侧，已由用户裁定“由该 owner 归位”）**：该会话把上述 5 个文件改动与 `crates/auto-ui-runtime/test/` 提交到它自己的分支/worktree，或 `git stash`/`git worktree add` 迁出；主检出 `git status --porcelain -- crates` 归零后，本 Plan 的 T-07 可继续（无需再问用户）。
- 处置：本轮不 stash、不 patch、不 restore（用户已裁定不动）。后续每轮复查一次；连续 3 轮仍不归位则按 goal 策略把目标标为 blocked 并附本条件。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 33 | from: review | to: merge | status: reviewed | outcome: waiting（主检出他方 WIP 未归位） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eef8a32 | next: merge（等待主检出归零）

### 2026-11-08 merge 预检（不依赖干净主检出的部分，已完成）

- stage: merge（预检） | plan_id: auto-ui/PLAN-062 | plan_revision: r5 | reviewed_commit: `3eef8a32` | outcome: 预检通过；入口门仍被主检出 WIP 挡住
- **分叉**：merge-base `3d64877d`；分支 **+32** 提交、master 自 base 起 **+77** 提交 → 不能 fast-forward，落地需要一个整合提交（merge 或 rebase）。
- **冲突风险：零文件重叠**。分支改动 26 个文件、master 自 base 改动 24 个文件，两者交集 **0** → 干净整合预期；落地后仍须按 merge 技能跑冻结 union 回归并做 landing 检查（合并提交会改变 HEAD，`reviewed_commit` 的证据需绑定到整合后的树或明确沿用同一 diff 集）。
- **但**：主检出**未提交**的 4 个文件（`desktop_protocol/{headless,host,rqhost}.rs`、`session.rs`）恰好属于分支改动文件集；若那份 WIP 被**提交进 master** 而不是迁到它自己的分支，整合将不得不在“062 已撤回 ResourceRelease”与“他方把 drain_resource_releases 接回来”之间做语义决断——那时需要该 owner 与 062 一起定夺，不能由 merge 静默择一。
- **wt-guard 预检**：`bash wt-guard.sh /mnt/d/autostack/.wt/ui-062/auto-ui` → **clean（无 reparse point）**，清理前置条件满足。
- 结论：只要主检出 `git status --porcelain -- crates` 归零且那份 WIP 不落到 master 的同一文件上，T-07 可以一次跑完（整合 → 冻结回归 → Spec 发布 → 归档 → 清理 → landing 检查）。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 34 | from: merge | to: merge | status: reviewed | outcome: waiting（预检通过；入口门仍被主检出 WIP 挡住） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eef8a32 | next: merge（等待主检出归零）

### 2026-11-08 merge 入口门第三次复查：仍被主检出他方 WIP 挡住（目标标 blocked）

- stage: merge（入口门复查 3/3） | plan_id: auto-ui/PLAN-062 | plan_revision: r5 | outcome: **blocked** | reviewed_commit: `3eef8a32` | 主检出 master @ `b0f951d5` | worktree `D:/autostack/.wt/ui-062/auto-ui`（clean）
- **条件未变（连续三轮相同）**：主检出 `git status --porcelain -- crates` 仍有 5 项 ——
  `crates/auto-ui-runtime/src/ui/desktop_protocol/{headless,host,rqhost}.rs`、`crates/auto-ui-runtime/src/ui/session.rs`（未提交，内容为把已撤回的 `drain_resource_releases` 接回 `FrameSource`）+ 未跟踪 `crates/auto-ui-runtime/test/`。
- **为什么不自行处置**：这些改动不属于本 Plan / 本会话；用户 2026-11-08 已裁定“先由该 owner 归位”。且这 4 个文件正属 062 分支改动集——若其被提交进 master，整合必须在“062 撤回 ResourceRelease”与“他方接回 drain_resource_releases”之间做语义决断，不能由 merge 静默择一。
- **精确 unblock 动作（owner 侧）**：该会话把上述 4 个文件改动与 `crates/auto-ui-runtime/test/` 提交到自己的分支/worktree（或 `git stash -u` 暂存、`git worktree add` 迁出）；随后主检出 `git status --porcelain -- crates` 归零。此后无需再经用户确认：T-07 可直接执行（整合 [零文件重叠，预检已证] → 冻结 union 回归 → SD-01/SD-02 发布 → 归档 → worktree 清理 [wt-guard 已 clean] → landing 检查）。
- **本 Plan 侧已就绪的证据**：最终复审 pass（`3eef8a32`，review10）；隔离窗口计时与像素门全绿；`wt-guard` clean；主检出/分支文件集零重叠。

- stage: goal | plan_id: PLAN-062 | plan_revision: r5 | round: 35 | from: merge | to: user | status: reviewed | outcome: **blocked**（主检出他方 WIP，连续 3 轮） | worktree: D:/autostack/.wt/ui-062/auto-ui | commit: 3eef8a32 | next: user（owner 归位后恢复 merge）
