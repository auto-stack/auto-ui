---
plan_id: PLAN-062
status: executing
feature_name: RQ无损原语、文字图形与资源重放
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-07
plan_revision: 3
current_step: 1
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
- 数值预算由UI061先量原direct基线、在候选比较前冻结：逐workload列单位/统计量/绝对与相对上限/样本/owner/失败处理。本轮无数值或测量结论；后继缺预算或必要测量不能签性能PASS，超标不得测后改阈值。

### 任务落点

T-02：`crates/auto-ui-runtime/src/ui/desktop_protocol/headless.rs`及`message.rs`，`ui/iced/broker_surface.rs`；T-03/04：`desktop_protocol/{codec,endpoint,host,shm}.rs`及实际资源缓存；两个消费点`rqhost.rs`与`session.rs`/`iced/renderer/shell_state.rs`。不得凭新接口名另造第二套实现，UI061冻结决定真实模块拆分。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/rq-render-packets-and-resources.md | 当前有限分散实现/历史设计 → 本Plan已验范围、来源、支持条件及失败/兼容合同及P-01～04/M-01～08性能要求 | 不再以历史“无漂移”或编译绿替代运行证明 | AC-01..04、AC-07 |

如实际改动还影响已有canonical条目，T-01具名补充并按语义变化修订Plan；不制造无关Spec条目，review核真实增量。

## 6. 测试设计

以下均为拟执行，不是本轮通过证据。主矩阵VM/Rust×direct/standalone-RQ/desktop-RQ；同source/主题/font/viewport/scale/时钟/事件序列。所有V行owner_stage=work。结构/glyph/资源、客户区像素和业务读回同时核验；旧direct金样不因共用新代码而失效。默认exact像素比较，真正平台采样非确定性只有具名区域/原因/敏感反控经审查后可设窄容差；不允许全局阈值吞漏图。

| Case | AC | 输入/真实入口与负控 | 可观察oracle/关闭条件 | 责任任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | UI061能力表、原direct金样、同source VM/Rust绘图fixture六格（§5） | 每个绘图ID有真实输出，结构/资源与像素对拍；旧direct未退化；必要平台未测明确阻塞 | T-01/T-05；全部现有绘图能力 |
| V-02 | AC-02 | mesh+SVG+raw text、RTL/连字/中文回退、rotation/shear、blur/clip叠层；删除命令/替换font反控 | 布局/glyph/命令及固定GPU客户区一致；无省略日志冒PASS，原反控必红 | T-02/T-05；字体与原语 |
| V-03 | AC-03 | 图片更新/视频帧/thumbnail、重用ID不同generation、释放后重建、缺资源与健康对照 | 资源只在正确surface生效、尺寸/alpha/采样一致、失败保旧、GPU在用时不早释；最后owner释放 | T-03/T-05；所有资源族 |
| V-04 | AC-04 | 双进程坏长度/乘积溢出/NaN/栈错配、未知required capability、慢Host/旧ACK/乱序资源 | 入帧前明确拒绝、旧完整帧保持，队列有界，资源与传输槽回收不混淆；旧合法协议控制仍可用 | T-04/T-05；协议事务与生命周期 |
| V-05 | AC-07 | 批量帧、延迟ACK、槽满与旧ACK；逐图元同步反控 | 稳态无逐图元往返/强制逐帧呈现屏障，按信用有界流水与背压；M-04消息/等待证据 | T-02/T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-06 | AC-07 | 稳定font/image/mesh重复帧、单项更新/驱逐/最后owner、旧generation | 重复payload/GPU创建为0（驻留未变前提）；变化恰当失效，M-04/M-05可核，GPU在用不早释 | T-03/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-07 | AC-07 | 生产双进程慢Host、突发帧与媒体资源更新、原direct对照 | 队列帧/字节峰值在预算内；只合并过期画面而保资源/输入依赖，M-01～06/M-08按触面通过 | T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
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
| AC-02 | work | 文字与全部原语无静默省略或近似，producer→wire→Replay精确；共用实现无第二布局器 |
| AC-03 | work | 资源版本/传输/更新/最后owner/GPU释放及图片媒体表现正确，无App隐藏窗/GPU/整帧自绘 |
| AC-04 | work | 协商/兼容、坏帧保旧、背压和generation负控通过，未知必需能力显式拒绝 |
| AC-05 | review | 独立上下文逐项验证当前修订AC/V、真实源和规范增量，所有必需项有可复核证据；不能只读完成勾选 |
| AC-06 | merge | 通过后按依赖顺序合入、canonical读回、路线回填，归档/guard/材料清理与恢复摘要完整；交付边界如实记录 |
| AC-07 | work | 真实producer/协议/Replay满足P-01/P-02及P-03队列、P-04资源/工作隔离；V-05～08证明驻留不重复上传、慢Host不覆盖/无界增长、混合负载有进展；本职责M指标满足UI061冻结预算，端到端不得借单测冒通过 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/操作与结果 | 关闭方式/AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | UI061正式可行决策 | 重核UI061全能力ID、direct旧基准、实现来源与预算；冻结当前绘图能力并集及精确接缝；接受UI061 r3性能合同与预算，冻结P/M对应实现来源和direct对照版本，缺预算不进入性能签收 | V-01；AC-01；AC-07/V-05～08（按§6分工）——见§9 2026-11-07 work 轮次：39 View ID + 8 能力ID入编译常量、061 冻结预算原样接受 |
| [ ] | T-02 | work | T-01 | 实现完整录制/原语表示与有序消费；文字glyph/font、mesh/path/SVG、transform/clip、gradient/border/shadow和可达custom primitive；禁止近似省略；去除Host重复文字整形/近似再构造；按完整原语批处理，不增加逐图元RPC | V-02；AC-02；AC-07/V-05～08（按§6分工）——进行中：Path/mesh/raw text/transform/SVG/image 已交付（`56139b78`，mesh 重放 0.0%、SVG 0.333%）；GlyphRun wire、blur/SDF、几何渐变仍具名缺口（§9） |
| [ ] | T-03 | work | T-02 | 实现资源create/update/release、字节/句柄身份、图片采样/alpha/rotation与媒体更新，Host重放及direct共用语义；实现驻留资源ID/generation复用、按变化上传和有界缓存；计数payload/GPU创建，合法驱逐/恢复单列 | V-03；AC-03；AC-07/V-05～08（按§6分工）——进行中：内容身份+驻留缓存（稳定帧 0 重传、驱逐可解释）、图片 alpha/rotation、SVG 栅格资源已交付；媒体族与 wire generation 字段待补（§9） |
| [ ] | T-04 | work | T-03；OS062共享符号按§5单写 | 交付UI063提交/反馈接口及旧新端兼容V-09；实现能力/版本协商、原子帧、长度/浮点/栈/资源校验、generation及背压/ACK/GPU fence寿命，保旧兼容明确失败；实现有界异步批量提交及槽信用、资源顺序/背压与per-App上传/驻留/CPU工作预算，避免长临界区和单App垄断 | V-04；AC-04；AC-07/V-05～08（按§6分工）；V-09——进行中：能力协商/准入保旧帧、槽 credit 背压、无覆盖在途槽已交付；per-App 上传速率/CPU 预算与慢 Host 负控待补（§9） |
| [ ] | T-05 | work | T-04 | 按§5绘图fixture完成六格全部绘图能力和两Host真实重放，不等UI063公开运行链；保迁移前direct金样；记录资源峰值/帧延迟，整理规范增量与遗留为零；按冻结M-01～08重放实际绘图/资源职责指标与公平/慢Host负控，保direct基线和完整功能，接线端指标交UI063/OS063最终消费 | V-01..04；AC-01..04；AC-07/V-05～08（按§6分工）；V-09 |
| [ ] | T-06 | review | T-05 | 独立上下文复核实际GPU画面、全部能力ID、敏感负控及应用无GPU路径，冻结双端修订；独立核P/M口径、冻结预算、性能负控和完整分布，不能用平均FPS或实现者摘要替代 | AC-05；复核AC-07 |
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







## 10. 待澄清事项

- 2026-11-07 实施后发现的三项**合同级缺口**（需用户裁定的范围/预算决定，不能用测试绕过）：
  1. **GlyphRun 与“Host 不重排文字”**（T-02/AC-02）：061 §4 要求文本优先携 glyph run；本仓 Host 重放面是 iced canvas（无 glyph 绘制 API），要满足“Host 不重排”必须新增 Host 侧字形栅格/图集能力（或改用 AGE/自绘 Host 后端）。当前实现仍是字符串 `TextStyled` + FontBlob（Host 用同一字体栈重排），只能算“降级可观测”，不是合同要求的精确重放。需要决定：(a) 在 062 内新增 Host glyph 图集（工作量与风险显著）；(b) 修订 AC-02 把“字符串+同字体栈重排”作为验收口径（具名降级）；(c) 把 glyph run 重放移入依赖 AGE 的后续计划。
  2. **shadow blur / SDF**：重放面仍为单层垫层近似；精确复现需 Host 具备 blur/SDF 能力（61 具名缺口）。同上需要范围决定。
  3. **per-App CPU 工作预算/公平调度**（T-04/P-04）：上传速率预算已于 `0e614935` 落地（8 MiB/1s 滑动窗口、超限整帧回滚 + 背压重试）；仍缺的是 CPU 侧分批/限流与 1/4/8 App 公平策略（真实规模归 UI063/OS063）。若要求本 Plan 内实现 CPU 预算，需要先定“分批单位与抢占点”，否则只能做代理测量。
- **本轮已收口的项**：per-App 上传速率（8 MiB/s + 整帧回滚 + 背压）已完成并用负控验证，不含在上述待裁定清单内。
- 性能数值预算/实际刷新率/可运行规模待UI061授权执行后按先基线后候选冻结；无预算不签PASS，支持承诺或阈值变化须具名修订。
- SVG 记录路径每帧新建 iced renderer（produce ≈20–32ms，T-05 实测）属 T-02 生产侧实现债，修它需要改 `HeadlessSurface` 的 renderer 生命周期（缓存迁移风险），建议单独有界任务而不是塞进本 Plan 收尾。
- 本轮没有必须再询问才能成稿的问题；实施暂停来自用户的规划范围。后续启动不改变既有依赖门。
- UI061的准确录制接缝/版本/资源预算是具名技术决策；未取得可行证据前不猜其后继实现，若改变承诺/仓scope给具体修订。
- 字体/IME/媒体/自定义图形或产品原有缺口不能以N/A自动缩分母；明确owner、实际失败入口、关闭条件，必要时另立Plan，当前必需项保持未完。
- Linux自有compositor和ACE/AGE的未来接口按战略进入门另立实施合同，本Plan没有自动切平台/引擎授权。
