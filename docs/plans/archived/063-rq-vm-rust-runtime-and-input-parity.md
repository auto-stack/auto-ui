---
plan_id: PLAN-063
status: archived
feature_name: RQ的VM与Rust完整运行、输入和服务一致性
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-08
plan_revision: 4
current_step: 5
total_steps: 7
depends_on: [auto-ui/PLAN-061, auto-ui/PLAN-062, auto-ui/PLAN-060]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/rq-execution-input-and-host-services.md]
touched_goals: []
affects: [renderqueue, rqhost, rendering-fidelity, resource-lifecycle, rq-performance]
---

# PLAN-063：RQ的VM与Rust完整运行、输入和服务一致性

## 0. 变更摘要

承接2026-11-05用户提出的RQ完整功能要求。设计见[完整设计](../design/autoui/renderqueue-complete-fidelity.md)，[战略/跨仓边界](../design/autoui/renderqueue-refactoring-roadmap.md)。本Plan仅drafting，尚未实施。交付限定为本合同结果，不以单个阶段完成冒整条RQ路线全部完成。

## 1. 目标

UI应用适配/输入/平台服务/调度，Man仅公开CLI和tool编排；两仓各一专属树。只读OS产品基线，必要Host事件桥通用实现仍归UI。非目标：新Feature/Service万能框架、完整Reload、产品业务重写、Linux UDS/IME重复实现、ACE/AGE切换。

工作结果以§7可观察AC为准；同源direct是兼容基准，禁止删除能力/用隐藏自窗或整帧自绘降级来达标。原生窗口标题栏与App内容区分别验，不要求Host系统装饰像素相同。

## 2. 架构方案

显式RQ意味着无AppGPU/窗口、精确remote命令，不可因不支持而偷偷改Pixels/direct。Direct默认保持。旧queue profile通过明确legacy选择兼容，不继续宣称完整功能。App保组件命中/焦点语义，Host只做surface路由/平台输入/服务。服务请求与输入绑定session/surface generation，关闭后迟到结果不得投新实例。应用执行与绘图可独立推进；未来ACE只换执行adapter。UI053正式解析栈变更如落同路径先消费；与940/056/057按符号单写，不设无关全局等待。

实现→独立review→merge依次使用同一权威实施树。拟用`D:/autostack/.wt/ui-063/auto-ui`、分支`codex/plan-063-dev`；本轮不创建。需要写入的协作仓在同组各一个树；产品只读/隔离数据，新增业务写面须另定具体合同。UI/Man/OS原owner接口保留，语言基础无UI反依赖。

## 3. 技术栈

Rust、现有iced/wgpu、desktop_protocol二进制消息/共享资源；当前Windows本地双进程与两种Host。测试沿仓内现有Cargo入口及真实公开CLI。Linux transport/共享映射归OS062；标准Wayland compositor和ACE/AGE仅在设计中留接缝，不在本Plan实现。

## 4. 需求分析与背景调查

### 授权

2026-11-08 r4：用户授权从063起更新RQ后续计划，并允许M1前新增修复计划；仅设计/合同修订，暂不实施。062已归档r5/7-7（cae1632e/a1cdfaa7），UI060 r8/6-6、UI061 r3/6-6、OS054 r3/10-10已交付。旧源码观察/“在途、Spec缺席”段落仅为r3历史，当前行为以两份RQ canonical及实际源码为准。

2026-11-05 r3：用户要求复核四份RQ计划范围/依赖、与UI060/OS062去重并明确启动顺序；仅修订草案，不实施。

2026-11-05 r2：用户明确授权将性能分析/建议补入设计、战略和相关Plan，仍暂不实施。本次仅修订合同，不新增计划或接管其他会话。

用户明确要求检查设计/实现、建立设计/战略、纳入拆分重构路线并考虑新计划；本轮获准调研、文档和立约，未获本轮实施授权。不自动开agent/聊天，不接管UI053/060/OS054/061/062在途合同，不提交他方WIP。无额外预算/时限授权。

### 源码事实

VM run_vm_rqhost_client根据env选择remote，Rust生成rq/desktop恒remote:true；headless Null clipboard与direct服务语义有差额风险；UI060已发现native API静默stub。来源已证明分岔，是否其他订阅/输入能力缺失仍由T-01实际盘点，不能预填失败。

基面UI `426c1f708e7c966f243c86b1f38029ae47b0fa0a`，OS `7968a68bfd15853a907f8b21f4dc106a2c216d39`；文件SHA256与逐项来源在主设计§2。UI Specs从`docs/specs/README.md`进入，相关`ui/structure-closure.md`、`runtime/component-and-store-instances.md`、`runtime/native-vm-app-validation.md`；OS从`docs/specs/shell/desktop-app-launch.md`与`desktop-bootstrap.md`进入。完整保真Spec当前缺席，本Plan提出新增，不提前改canonical。headless/rqhost/client_entry/message/endpoint/shm/transport及iced/broker_surface是现有路径；RenderPacket/ReplayCore与新测试路径是拟议，不冒已实现接口。

立项时取号扫描主检出、archive/archived及全部注册worktree；当时UI060仅树内亦计入占号。r3时UI060合同已恢复主检出、仍executing；UI053已有正式交付与清理收据。锁内取号、不覆盖已有Plan；UI无new-plan.sh使用独占分配，OS使用仓内脚本。实现时重新冻结真实源与依赖，不能依此快照借WIP。

## 5. 详细设计

完整不变量和跨阶段约束沿主设计§3–8；本Plan工作集中于§8唯一任务表。T-01建立`D:/autostack/plan-artifacts/plan-063/auto-ui/`并登记owner、用途、在途状态；探针/截图/日志/外部构建只在统一scope，不落docs/plans/evidence或attachments。正常编译target在专属树；外置target也只在scope并登记可删。

未决接缝由对应前置任务做有界探针并形成可审阅决定，不能一边猜一边切生产；源与已交付依赖不变时复用有效证据。三Host控制面可扩展但没有未来支持假承诺。能力丢失/非法帧必须具名失败；资源上限须T-01按真实测试环境和UI061决策冻结单位、边界、失败表现，不新设语法/控件限制。

### 范围与启动门复核（r3）

- **准备不等于实施**：T-01只读入口/服务/差额盘点可先做，不能勾完；正式冻结T-01及T-02～05生产修改须同时消费UI061合同、UI062接口/实现、UI060正式输入/API/刷新修复。depends_on表示实施/交付门。UI053已正式交付，只核实际来源，不再等其收尾。
- **UI060去重**：Expr::Index/shim、MCP pointer→DesktopInject/broker、输入dispatch与产帧刷新顺序、native API真实链接及Notes/Todo原验收仍由UI060关闭。本Plan消费其行为，再补VM/Rust两Host完整输入/服务/异步/调度；UI060可选HandlerCall不被升级成强制前置，不要求它额外实现完整RQ。回归保留真实失败入口，不能重写原修复冒新功能。
- **给OS063的接口**：T-01冻结，T-03/04交付通用Host adapter：surface身份、placement/z/clip、坐标逆变换、focus/capture、resize/DPI、可见性、readiness及present反馈。通用机制归UI，OS063只配置/接线桌面策略；T-05用最小真实双Host验证，不等OS054/OS063全产品组合。Man限公开模式、工具/Host发现及所需运行资源闭包，不另造发行系统。
- **OS062共享面**：rust_ui、client_entry/rqhost、Man cmd_run/tool发现、Notes初始化可与OS062 T-04/05同符号。Linux ELF/UDS/IME/Nix归OS062，本Plan交Windows通用运行合同；通用根修先正式落地后消费，后改者保原平台行为，不能仅凭cfg不同就认定无冲突。需要Linux联验的共享修改具名执行者/证据，未测不外推；不等OS062整项，也不追加其原AC。

### r4：062正式交付后的承接与M1边界

本Plan保持drafting0/7。本轮不重开062，不以归档通过外推全部能力。已读runtime/rq-fidelity-contract.md及runtime/rq-render-packets-and-resources.md；规划来源：rq-fidelity-contract.md SHA256=57f9d191c3718979716e286c103b901a60496224360ab50250675a1cf868d2ff；rq-render-packets-and-resources.md SHA256=bc496c0109716f986098089f45ebb2ad4000f81cf33ae1dbe875a4bc1c3108fe。这些是设计来源，不是本轮运行结果。

**启动**：UI060/061/062全部正式前置已到，后续获得实施授权即可T-01；不等待UI073/OS063。先交付通用运行/输入/调度，UI073消费它，再交OS063完整验收，避免UI063↔UI073循环。UI063只关闭运行/输入服务分母，UI073负责尚未交付的媒体/精确绘制；UI063完成不宣称路线M1完整六格已就绪。

| 062交接项 | 本Plan责任与关闭要求 | 原任务/验收 |
|---|---|---|
| M-02/M-06/M-07 | UI063实现事件/deadline/present、冷热启动/恢复通路及公平调度，最小真实两Host验证；OS063测最终组合/1、4、8 App及完整成本。061/062代理与未测项不能抵实际present/响应/进展证据 | T-01/04/05；AC-07；V-07/08/10 |
| 四既有红 | autocenter_translates_hits_with_ops、stage3_multi_app_stress_n3/n5、t3_examples_queue_end_to_end逐项列当前来源/输入/断言/基线对照/候选适用/owner/关闭证据；命中本路线必需能力不能基线红豁免 | T-01/05；AC-01/04；V-11 |
| 连续媒体/绘制残差/重排负控 | 生产修复及最小六格由UI073负责，UI063提供真实接口；OS063做最终产品消费，不另造第二套replay。静态poster、文字重排、2%阈值不能关闭完整媒体/精确画面义务 | T-01/05接口与边界交接；AC-01/04；V-09/12；[UI073](073-rq-m1-media-fidelity-and-resource-closure.md) |
| r5计时与真正隔离 | 062 replay p50设门/p95只记录是局部门，不等最终端到端尾延迟、deadline通过。T-01先冻结真实负载/样本/时序终点/硬预算与污染规则，禁止测后宽限；isolated只是标签，需实际CPU/GPU/外来进程负载记录，不具测量条件保pending | T-01/04/05；AC-07；V-10 |
| 旧制品与删除证据 | 重建本候选App/Host/工具/SDK，绑source/dep/config/hash/实际PID，无旧exe或开发根fallback。归档摘要足以证明的证据可复用，仅缺失或变更的case重验，删除原始scope不自动全跑/假绿 | T-01/02/05；AC-01/04；V-12 |

默认客户区exact及结构/业务oracle目标保持。062 r5的逐族2%/非exact与p95只记录不得自动变成本Plan最终通过口径；本Plan相关绘制尚未修复则具名标“等待UI073”，不声称M1完整图像已过。最终窄容差只允许原合同所述真正平台采样非确定性并有区域/理由/敏感负控；实质支持承诺或预算改变须另行裁定/修约。

生产修复不混进只读归因：四红中输入/调度根因归UI063，绘图/资源根因归UI073，产品业务根因交实际owner具名合同；无正式修复合同/证据的必需缺口阻断本Plan相关AC与M1。不能仅按名字判断，更不要求无关历史红全部清零。

### 性能合同（r2继承）

遵循[主设计§7.1～7.3](../design/autoui/renderqueue-complete-fidelity.md)的P-01异步批量、P-02资源复用、P-03呈现调度、P-04多App隔离和M-01～08指标。本Plan的关闭条件是AC-07、V-05～08与§8任务，设计链接不代替实际证据。

- RQ不是逐绘图RPC：steady-state不逐图元等待reply，也不强制每帧等待上一帧呈现后才准备；容量不足须背压，传输ACK与GPU资源寿命分开。
- 驻留且未变font/image/mesh资源重复帧不重复上传payload/创建GPU资源；合法失效、驱逐、重建单列，不重整形App已定位的文字。片段复用先实证，不默认重做全局增量场景。
- 及时响应输入/资源完成并按deadline/present调度，固定15ms轮询不能是唯一驱动；在途帧/字节有上限，可合并过期画面但不能丢输入、资源依赖或业务副作用。
- per-App上传/资源/CPU工作预算与公平调度必须有轻重混合负控；长任务分批/限流，不能假设任意GPU命令可抢占。Host/device loss仍是共享故障域，不冒完全隔离。
- 数值预算由UI061先量原direct基线、在候选比较前冻结：逐workload列单位/统计量/绝对与相对上限/样本/owner/失败处理。本轮无数值或测量结论；后继缺预算或必要测量不能签性能PASS，超标不得测后改阈值。

### 任务落点

T-02：`crates/auto-ui-runtime/src/ui/desktop_protocol/{render_cli,client_entry,rqhost}.rs`与`crates/auto-ui-tools/src/rust_ui.rs`；Man `crates/auto/src/{cmd_run,cmd_autodesk}.rs`。T-03/04：`desktop_protocol/{headless,message,win_ime}.rs`、`session.rs`、`iced/renderer/{shell_state,subscriptions}.rs`及实际Component实现；新增服务桥路径由T-01定位，禁止复制direct运行循环。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/rq-execution-input-and-host-services.md | 当前有限分散实现/历史设计 → 本Plan已验范围、062r5承接、073交接、旧红适用/关闭、重建与真实计时条件及P/M性能要求 | 不再以历史“无漂移”或编译绿替代运行证明 | AC-01..04、AC-07 |

如实际改动还影响已有canonical条目，T-01具名补充并按语义变化修订Plan；不制造无关Spec条目，review核真实增量。

## 6. 测试设计

以下均为拟执行，不是本轮通过证据。主矩阵VM/Rust×direct/standalone-RQ/desktop-RQ；同source/主题/font/viewport/scale/时钟/事件序列。所有V行owner_stage=work。结构/glyph/资源、客户区像素和业务读回同时核验；旧direct金样不因共用新代码而失效。默认exact像素比较，真正平台采样非确定性只有具名区域/原因/敏感反控经审查后可设窄容差；不允许全局阈值吞漏图。

| Case | AC | 输入/真实入口与负控 | 可观察oracle/关闭条件 | 责任任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | VM公开-q/remote与Rust同exe rq/desktop/direct；缺Host、错endpoint、legacy参数负控 | 来源/profile/Host身份可观测；rq/desktop绝不隐藏自窗/回退，direct缺省与旧合法入口保留 | T-01/02/05；公开CLI与生成入口 |
| V-02 | AC-02 | 真实pointer/drag/capture/hover/wheel/Tab/modifier，中文IME预编辑/提交/取消、emoji、clipboard，resize/DPI中途 | 控件与caret/selection正确；Host坐标逆变换准确；剪贴板真实往返；错surface和旧generation不串投；服务取消不改新实例 | T-03/05；全输入/平台服务ID |
| V-03 | AC-03 | 无输入时timer/动画、HTTP/SSE迟到、主题变化、route/store同名、真实Notes/Todo操作及后端失败 | 无需虚假输入触发更新；只初始化一次；API/持久层读回与画面一致；错误可见、失败保稿，无stub成功 | T-04/05；UI060范围外接缝与回归 |
| V-04 | AC-04 | 两App共享Host、关其一/末窗/pending采纳、HostLost、关闭后异步结果、同库重开 | 其余App继续，自己PID/资源/端点有界释放，旧事件不达新实例；HostLost显式、重开与业务合同一致，App无GPU/window | T-05；生命周期/实际消费者 |
| V-05 | AC-07 | 延迟平台服务结果、Host ACK与真实连续输入；合法返回控制 | 服务等待不串行阻塞绘制/输入热路径，消息/帧身份可追，输入与业务结果数守恒 | T-03/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-06 | AC-07 | 稳定资源UI反复交互、主题变更/新session及UI062正式证据 | 接线不重建未变资源；跨实例隔离，必要重传有原因，M-04/M-05可复用身份一致证据 | T-01/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-07 | AC-07 | 空闲唤醒、滚动编辑、动画resize、实际刷新率、慢Host与强制固定轮询反控 | M-01/M-02/M-03分段含等待和空闲成本；事件/deadline驱动有效，fallback不免预算，丢弃统计完整 | T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-08 | AC-07 | 轻App真实输入+重App事件/上传/绘图，卡顿/退出/迟到结果 | 轻App不饥饿，重App被限流但不无进展；M-07通过，按键/IME/存储副作用未丢，旁观实例仍存活 | T-04/T-05；work；本Plan职责，机制相同且身份未变证据可复用 |
| V-09 | AC-01、AC-02、AC-03 | 正式UI060行为、UI062接口、两Host adapter；漏present/可见性、错generation及旧入口反控 | 通用运行接口独立可验，原修复不回归；平台差额具名，不等OS063全产品 | T-01～05；接口消费/交付 |


| V-10 | AC-07 | 两Host真实输入→App→提交→可观测present、冷/热启动/恢复，先冻结预算；忙机/缺present仅代理/仅isolated标签的负控 | 终点/样本/机器负载有实证，尾延迟/deadline/冷热恢复按实际职责闭合；环境污染、p95仅打印或缺测不PASS | T-01/04/05；通用机制归UI063，最终规模归OS063 |
| V-11 | AC-01、AC-04 | 四旧红逐case原输入与断言、原基线与当前候选；遗漏case/仅写“known red”反控 | 当前影响/owner/正式关闭证据可核，命中必需面仍失败则不能通过相关AC；范围外有真实理由，不按名称一律重做或免责 | T-01/05；不扩全历史债 |
| V-12 | AC-01、AC-02、AC-04 | 冻结producer/Host/工具/SDK重建后真实启动；故意同名旧exe/缺资源；摘要复用与UI073交接表 | 运行hash/来源/配置/PID吻合；旧制品失败，删除附件不自动丢有效摘要，未交媒体/图像能力明确阻断其最终完整验收 | T-01/02/05；OS063最终T-04另核 |

性能实测口径：M-01响应延迟p50/p95/p99/max及分段；M-02帧间隔/目标deadline失约、合并/丢弃/画面年龄；M-03 App/Host/总CPU与可测GPU时间、空闲唤醒；M-04命令/资源字节、消息/往返、复制与上传/创建计数；M-05每进程/总内存与共享去重、GPU驻留和队列峰值；M-06冷/热启动及恢复；M-07轻重App公平；M-08保真/业务守恒。各职责分别取证，OS063做最终组合，未执行不得被上游局部测试抵扣。

同执行器原direct、候选direct及两RQ Host同机器/source/build profile/GPU/字体/DPI/分辨率/主题/事件/资源/刷新率比较；冷/热分列。完整多App成本为N个App加Host，direct为N进程；桌面壳基准单列且不得隐藏其成本。event/frame ID配对、单调时钟校准偏差；无法量物理上屏只标实际present代理终点。UI061先冻结样本长度/重复/运行顺序/污染规则，成对或交错运行；保失败/缺测/异常与分布，不择优挑样本，p99样本不足不给通过结论。性能测量仅在获准实施后的独立窗口，不与其他会话争用GPU。

测量入口由UI061 T-04的`tests/rq_fidelity/`原型补计时/计数并交付准确命令，后继T-01读回；OS063由`scripts/rq-parity/`编排真实组合。两路径仍是拟新增，本轮不创建脚本或运行基准。不得新增一个与现有oracle无关的泛化跑分系统。

### 验证入口

实施UI树：`cargo test -p auto-ui-runtime --lib ui::desktop_protocol:: -- --test-threads=1`及实际输入/renderer/service完整职责族；生成器`cargo tu -- --test-threads=1`，CLI/tool触面`cargo ti`。Man树复用其`.cargo/config.toml`和`docs/specs/cli/structure-closure.md`现行owner门（T-01读回，不臆造固定组）。真实入口`auto run -r vm -q --render=remote`、同生成exe `--render-mode rq`及`--render-mode desktop --desktop-endpoint <本次端点>`；CLI参数以T-01真实help核对，禁止旧exe冒当前结果。

T-01确认真实test namespace/feature/平台条件；新增入口由标明任务提供。列用例/执行/条件排除，准备与运行时间分列；非零/超时/skip不充PASS。按实际触面完整职责族和必要消费者运行，最终review后merge只跑冻结并集；不叠跑全仓或AG里程碑。真实窗口验证不抢占其他会话，不能通过提高资源阈值隐藏功能缺口。

### 角色与证据

work负责实现/自检；独立上下文review只检查冻结同一树，不另建reviewer树；merge落地与清理。未指定模型/代理，不自动创建。每条证据绑Plan revision、代码/依赖提交、实际profile/进程/资源/产物，失败/修复后结果分列。用户未授权实施前不进入上述运行。

## 7. 验收标准

| ID | owner_stage | 可观察行为/判据 |
|---|---|---|
| AC-01 | work | VM/Rust公开RQ入口都走精确remote，三种运行选择来源明确；无静默Pixels/direct降级 ；按r4承接表及V-10/11/12中关联本AC的义务逐项核验，无隐式豁免|
| AC-02 | work | 全部输入/IME/剪贴板和已存在窗口服务与direct一致，跨surface/resize/关闭身份安全 |
| AC-03 | work | 初始化/订阅/异步/主题/store与真实API副作用在六格一致；失败不是静默默认值 |
| AC-04 | work | 全运行能力与两App生命周期通过，真实动作/持久读回成立，RQ应用无自绘 ；按r4承接表及V-10/11/12中关联本AC的义务逐项核验，无隐式豁免|
| AC-05 | review | 独立上下文逐项验证当前修订AC/V、真实源和规范增量，所有必需项有可复核证据；不能只读完成勾选 |
| AC-06 | merge | 通过后按依赖顺序合入、canonical读回、路线回填，归档/guard/材料清理与恢复摘要完整；交付边界如实记录 |
| AC-07 | work | 满足P-03帧唤醒/截止与P-04运行/输入公平，保持P-01异步及P-02资源接线；六格端到端M指标按冻结预算通过，输入/业务不丢失，timer fallback具名且不能以高FPS掩盖排队/丢帧 ；按r4承接表及V-10/11/12中关联本AC的义务逐项核验，无隐式豁免|

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/操作与结果 | 关闭方式/AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 可先只读盘点；冻结须UI061/062/060正式交付 | 核UI053/060实际相关交付与运行能力ID，冻结VM/Rust公开入口、后端profile、服务与调度边界；共享符号先交接；接受UI061正式合同、UI062 r5资源提交接口与局部计时边界，冻结输入到呈现及调度测量点、OS063通用Host adapter与Man运行闭包接口（V-09） ；按r4承接表冻结四旧红/计时真实环境/重建身份与UI073分工| V-01；AC-01；AC-07/V-05～08（按§6分工）；V-09 ；V-10/11/12|〈T-01冻结：`plan-artifacts/plan-063/auto-ui/t01-freeze.md`；UI061预算/UI062r5接口接受；四旧红/测量点/接口/重建身份冻结〉|
| [x] | T-02 | work | T-01正式冻结；不借UI060/OS062 WIP | 整理render_cli/client_entry/rqhost及rust_ui生成、Man cmd_run/cmd_autodesk薄编排；显式rq/desktop统一remote，不变direct默认；legacy显式兼容且无静默fallback | V-01；AC-01；V-09 |〈ui 8de221a1+man 347c29f；单测绿；真入口探针desktop缺端点具名报错、remote链serving→adopt→first frame实录〉|
| [x] | T-03 | work | T-02 | 交付§5通用Host adapter，真实双Host驱动核V-09；完整Host输入→App iced事件；DPI/resize/capture/focus/键盘/IME/clipboard与窗口服务，补真实服务响应/取消身份；真实输入/服务不阻塞常规绘制热路径；可合并输入明确边界，不丢按键/文字/button/capture转换和副作用 | V-02；AC-02；AC-07/V-05～08（按§6分工）；V-09 |〈G-01..05关闭+线级单测绿；边界：TitleChanged/scale/a11y具名未实现（spec §2）；V-02六格核验归T-05〉|
| [x] | T-04 | work | T-02 | 对齐初始化、timer/subscription/异步/HTTP/SSE、主题/store/组件寿命与错误通路；消费060真实API，不复制其修复；以输入/readiness/deadline及时唤醒，呈现配合目标刷新与反馈；不以15ms固定轮询作唯一帧驱动，fallback具名且接受同预算 | V-03；AC-03；AC-07/V-05～08（按§6分工）；V-09 |〈即时泵+自适配档落地；V-03六格核验归T-05〉|
| [x] | T-05 | work | T-03、T-04；UI062正式同版；不等OS063产品全量 | 验证关闭/迟到消息/HostLost、重开、共享Host隔离和App无自绘；全运行能力六格及产品用户动作，规范草稿；验证per-App调度/过载/故障隔离，运行M-01～08端到端职责对照与空闲/高刷新适用面，继承有效UI062证据而不盲目重跑 ；闭合r4适用旧红/资源终态/制品与计时交接，缺必需项不交review为PASS| V-04及V-01..03；AC-04；AC-07/V-05～08（按§6分工）；V-09 ；V-10/11/12|〈2026-11-08 闭合：五旧红根因全修（45e8c60d）；V-03 timer腿+M-01/02/04/05/06/07 实测（72fa76a7/304e1fd3 + V-03/M commit）；P-04 泵预算机制；滤程 290/0（gate-final5.log）；V-12 重建身份（t01-freeze §9）；具名未测=M-03 GPU/高刷/物理上屏/规模组合（spec §3.1，归 OS063）〉|
| [ ] | T-06 | review | T-05 | 独立上下文复核物理输入/IME、后台读回、模式身份和全部运行能力，冻结UI及Man实际提交；独立核P/M口径、冻结预算、性能负控和完整分布，不能用平均FPS或实现者摘要替代 | AC-05；复核AC-07 |
| [ ] | T-07 | merge | T-06 pass | 执行受影响冻结门，依赖顺序落地UI/Man与canonical，归档并guard清理；沉淀性能支持边界/预算与实测差额，不外推未测平台 | AC-06 |

work项完成才进入execution_done；review/merge项仍待办，不能为交接伪勾。每个任务读取§4对应实际路径及主设计证据；新增接口先由有界决策任务确认。全部原始材料merge折摘要进本Plan§9后删自有scope；移除树先wt-guard clean，不穿透链接、不清未知owner目录。

## 9. 复审记录

2026-11-05 / stage:new / plan_id:auto-ui/PLAN-063 / revision:r1 / outcome:pass（草案合同，不是实现验收） / next:待用户启动后work/T-01，按§8正式依赖核门。

草案静态预检：0–10章节齐全，任务数与total_steps一致，AC/V/SD映射及新文档本地链接有效；主检出/两类归档/注册worktree中的本号唯一。已做源码/设计/相关Spec与重叠计划检查；本轮未实现、构建、跑运行门或独立final review。当前task/AC/V/SD为执行合同；实际通过证据后续填写。设计前置的不确定性保留为任务/阻塞，不能声明已解。

### 2026-11-05 r2性能合同修订

- stage:new / plan_id:auto-ui/PLAN-063 / revision:r2 / outcome:pass（文档修订，非性能验收） / next:待用户启动后按§8依赖进入work。
- 变更：新增AC-07、V-05～08；§5增加P-01～04，§6补M-01～08及direct配对/样本/冻结预算口径；§8原任务增补性能职责，SD-01同步扩展。总任务数不变，current_step仍0，status仍drafting；r1记录保留为历史。
- 用户授权仅文档，未实现、编译、运行或测量；没有性能PASS或节省承诺。r2静态检查只核章节/引用/任务与AC/V映射，不是独立final review。

### 2026-11-05 r3范围与依赖复核

- stage:new / plan_id:auto-ui/PLAN-063 / revision:r3 / outcome:合同修订，静态核查 / next:仍drafting，按§5/§8依赖等待用户启动。
- 修订发现：拆开绘图/运行/产品六格以消除反向依赖；UI060独占索引/输入/刷新/API修复，OS062保Linux通道/平台/安装；补接口交付/兼容门及OS063实际UI策略写面。P/M性能要求及全部能力/产品分母不减少。
- 来源：UI060主检出executing r1/2-6、扩围及续接记录；UI053归档6/6及UI 9a94d568 / lang 61207576b正式交付（当前主线祖先）；OS062 drafting r1/0-13的T-02～05/09～11；headless、UI session/shell_state与OS薄Host入口。启动时重新冻结源版本/单写符号，不将快照永久当准入。
- 保留r1/r2历史；步数、AC数量和current_step不变，补V-09接口负控。未改UI060/OS062/OS054原合同，未建树/实现/构建/运行或签final review；运行证据待后续work。

- r3静态检查：四份合同0～10章节、27项任务（6/7/7/7）、各7条AC/9条V及引用一致；正式depends_on图无环；RQ与本轮现行路线共155条本地链接有效；三仓本轮文档范围diff --check退出0。只证明合同结构/链接，不证明任何运行、性能或Linux支持。


### 2026-11-08 r4后续承接修订（仅规划）

- stage:new；revision:4；outcome:blocked（执行授权未到；T-01须冻结真实预算/旧红适用与接口）；next:按§5分阶段门进入T-01，当前drafting0/7。
- 保留7任务/7 AC及历史；新增V-10/11/12，增强T-01/04/05及AC-01/04/07、SD-01。062 r5局部门/具名遗留不自动降低M1目标；UI073独占剩余绘制/连续媒体/资源反例，OS063独占最终产品组合性能，UI063负责通用运行机制。
- 本轮未实施/运行/建树/改canonical、未独立审查通过；既有r1～r3审查仅历史，不转作r4 PASS。
- 本轮规划静态自检：0～10章、T/AC/V/SD=7/7/12/1，所有任务未勾选、drafting0步；本轮四合同依赖图无环（解析80个真实Plan），本轮当前文档共82条本地链接及表格/围栏有效。语义合同SHA256=`f24c83863f9b20fdda37e5bfdd0613ac0c57080fd5435e3c17110746eeec76a5`（UTF-8/LF，排除§9）。仅合同自检，非独立设计复审或运行PASS；本轮未实施。

### 2026-11-08 work 进展（goal 授权首轮：T-01～T-04）

- stage:goal / plan_id:auto-ui/PLAN-063 / revision:r4 / round:1 / from:work / to:work / status:executing / outcome:needs_fix（T-05 未闭，见§10） / worktree:D:/autostack/.wt/ui-063/auto-ui + D:/autostack/.wt/ui-063/auto-man / branch:codex/plan-063-dev（两仓） / base_commit:5988b19e(ui)+2a9b16e(man) / code_commit:8de221a1(ui)+347c29f(man) / dependency_revisions:auto-lang a95d6dc41, auto-web 2d69841, auto-down 58ea33f（组内只读树） / task_ids:T-01,T-02,T-03,T-04 / evidence:plan-artifacts/plan-063/auto-ui/{t01-freeze.md,t02-t03-t04-evidence.md,t034-protocol-gate*.log,reds-isolated.log,probe-remote*.log} / blockers:T-05 旧红族（§10 第1条） / next_responsible_role:work（T-05 续）
- T-01 冻结（入口值域/服务调度缺口 G-01..12/UI061 预算接受/UI062r5 接口/四旧红现状/测量点 t0..t7/OS063+Man 接口/重建身份）；T-02 双仓落地+真入口探针；T-03 G-01..05 关闭+线级单测；T-04 即时泵+自适配档。V-01 真链实录：`serving → render-queue: remote mode → [render] render-mode: rq (AUTO_VM_RENDER env) → adopt P063Probe → first frame`（probe-remote3.log）。
- **V-11 根因突破**：`autocenter_env_gate_bypasses_translation` 置 `AUTO_NO_AUTOCENTER` 不还原——`cargo test --test-threads=1` 顺序滤程同进程 env 串染全部后续 `RqProjector::new`；修复（panic 安全还原）后 `autocenter_translates_hits_with_ops` 绿（gate3）。已提交 8de221a1。
- 顺序滤程 gate3：**281 passed / 4 failed**——残留红 `stage3_multi_app_stress_n3/n5`、`t3_examples_queue_end_to_end` + `t2_remote_ws_session_end_to_end`（干净基线 stash 验证亦红，非本轮引入；四红同属 outproc 点击→递增帧族，安静窗隔离仍红=系统性，根因未定位——下一轮优先）。
- 新增 spec 草稿 `docs/specs/runtime/rq-execution-input-and-host-services.md`（worktree 内未发布；merge 时按 SD-01 沉淀）。

### 2026-11-08 work 进展（goal round 2：V-11 五红全闭）

- stage:goal / plan_id:auto-ui/PLAN-063 / revision:r4 / round:2 / from:work / to:work / status:executing / outcome:needs_fix（T-05 余项见§10②③④） / worktree:同 round 1 / code_commit:45e8c60d + 6bf63fb3（ui；man 347c29f 不变） / task_ids:T-05（V-11 部分） / evidence:plan-artifacts/plan-063/auto-ui/{reds-isolated.log,diag-n3*.log,reds-fix*.log,gate-final*.log,t3-fix.log,probe-t2.log} / blockers:无（V-11 已闭） / next_responsible_role:work（T-05c 六格/公平/测量）
- **五个顺序滤程旧红全部根因闭合**（非环境豁免），诊断链：AUTO_PLAN060_DIAG 子端点输入追踪 + pointer_down_left 命中 dump + viewport-sync 追踪 + 干净基线 stash 对照：
  1. `autocenter_*`：env 门测试串染（round 1 已修，8de221a1）。
  2. `t2_remote_ws`（**产品缺陷**）：`ClientPump::dispatch` 无 `FrameMsg::Resize` 臂——宿主视口同步被静默丢弃，子面恒 Hello 尺寸（480×900）与宿主孪生（rect 482×712）错位。修复 = 泵补 Resize 臂（转发端点 + 立即产帧）；副产品 = remote 子面首次真正跟随宿主窗 resize。
  3. 孪生视口用 raw rect（含 chrome）≠ 子面内容尺寸——改 `vwin_content_size` 同律。
  4. `t3_*`：twin 内容坐标点击缺 `+TITLEBAR_H`（`broker_command_pointer_local` 减 36）；005 submit 同。孪生改 place 后按内容尺寸构建。
  5. `stage3_multi_app_stress_*`：PLAN-678 off-居中合同 + 同一标题栏偏移——点击改 off 律孪生中心 + 原点 + 36。
- **关闭证据**：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` = **287 passed / 0 failed**（gate-final2.log，374.85s；基线 4 红）。新增收口钉 `p063_stress_click_geometry_probe`（两布局律）与 `p063_counter_twin_click_probe`（孪生-子面同构 parity）。
- spec 草稿 §4 同步为已闭表（6bf63fb3）。

### 2026-11-08 work 进展（goal round 2 续：T-05c 机制与测量；滤程全绿）

- stage:goal / plan_id:auto-ui/PLAN-063 / revision:r4 / round:2 / from:work / to:work / status:executing / outcome:needs_fix（T-05 收尾项见§10②④） / code_commit:72fa76a7 + join 修复（ui） / task_ids:T-05（P-04/M-01/V-11 终证） / evidence:gate-final4.log / next_responsible_role:work（T-05 六格收尾 → T-06）
- **P-04 机制落地**：`PUMP_MSG_BUDGET_PER_TICK=64` 单客户端单拍泵预算——重客户端帧泛时守护单线程 update 按预算切片，余量下拍续排（FIFO 保序不丢；槽信用使稳态远低于触界）。
- **M-01 代理分段测量**（`p063_m01_input_to_frame_proxy_segments`，真实采纳 + 真 VM 组件 + press/release 对 + 有界轮询）：**input→frame-at-host p50=0.055ms / p95=3.18ms / max=4.42ms（n=40）**，硬门 p95≤100ms（T-01 冻结）通过；M-04 有界守恒（无输入丢失、≤2 帧/点击）；代理终点具名（物理上屏归 OS063）。legacy queue 臂测量；v2 remote 臂分段由 062 rq_cells 覆盖。终态记录：M-02/03/05/06/07 本轮未测（OS063 最终组合 + 专用隔离窗测量待 T-05 收尾）。
- **滤程终证**：`cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` = **288 passed / 0 failed**（gate-final4.log，313.54s；基线 4 documented 红 + t2 全闭）。M-01 测试在滤程上下文的 join 闭死已修（线程 EOF 自退不 join）。

### 2026-11-08 work 完成（goal round 3：T-05 收口 → execution_done）

- stage:work / plan_id:auto-ui/PLAN-063 / plan_revision:r4 / outcome:pass（work 面全量；独立 final review 归 T-06） / worktree:D:/autostack/.wt/ui-063/auto-ui + auto-man / branch:codex/plan-063-dev ×2 / base_commit:5988b19e(ui)+2a9b16e(man) / code_commit:ui 8de221a1→45e8c60d→6bf63fb3→72fa76a7→304e1fd3→(spec commit)+man 347c29f / dependency_revisions:auto-lang a95d6dc41, auto-web 2d69841, auto-down 58ea33f / task_ids:T-01,T-02,T-03,T-04,T-05 / evidence:gate-final5.log（290/0）、gate-final4.log、reds-*.log、diag-*.log、probe-remote*.log、t01-freeze.md（含 §9 V-12）、t02-t03-t04-evidence.md / blockers:无 / next_responsible_role:review（T-06 独立上下文）
- T-05 关闭清单：五旧红根因全修+滤程全绿；V-01 真入口探针（rq/desktop 负控+remote 链实录）；V-02 线级（KeyReleased/modifiers/Focus/剪贴板往返）+ 桌面会话真实子进程点击/键入/IME（t3/stress 绿）；V-03 零输入 timer 帧+订阅恰一次；V-04 生命周期/HostLost/共享守护隔离（stress n3/n5 + rqhost 生命周期族绿）；P-03 即时泵+自适配档；P-04 泵预算切片；M-01 p95=3.18ms（≤100ms 硬门）/M-02/M-04/M-05/M-06/M-07 记录+硬门；规范草稿已落 worktree（merge 时 SD-01 沉淀）；V-12 重建身份哈希绑定。
- 具名未测（记录为边界非豁免）：M-03 GPU 时间、高刷新适用面、物理上屏终点、1/4/8 规模组合、V-10 端到端终值——归 OS063（r4 承接表同律）。
- work-owned AC 对账：AC-01（入口/无降级，V-01 实录）✔；AC-02（输入/服务面交付+线级/子进程证据）✔；AC-03（V-03 timer/订阅恰一次 + 060/053 消费不变）✔；AC-04（生命周期/隔离/无自绘——headless 客户端无窗无 GPU，宿主渲染）✔；AC-07（P-03/P-04 机制+冻结门 M-01/M-07 通过）✔；AC-05/06 归 review/merge。

### 2026-11-08 独立终审（goal round 3：T-06 → reviewed）

- stage:review / review_scope:final / reviewer_identity:独立子代理上下文（同会话派生的全新对抗性上下文；局限=同模型血统，已按工件重建判定而非实现者摘要；未伪造 model/session ID）/ plan_id:auto-ui/PLAN-063 / plan_revision:r4 / outcome:pass / worktree:D:/autostack/.wt/ui-063/auto-ui + auto-man / branch:codex/plan-063-dev ×2 / reviewed_commit:921765d1（pass 判定基线）→ 562cad7b+96236725（判定后 doc-only 修订：F1 spec P-04 措辞 + F2 诊断注释去"临时"化——零代码变更，p063 族 8/8 复验绿）/ base_commit:5988b19e / dependency_revisions:auto-lang a95d6dc41, auto-web 2d69841, auto-down 58ea33f / spec_inputs:rq-fidelity-contract.md + rq-render-packets-and-resources.md（canonical）+ SD-01 草稿（worktree，merge 发布）
- 审查员独立复验：滤程 `cargo test -p auto-ui-runtime --lib "ui::desktop_protocol::" -- --test-threads=1` = **290 passed / 0 failed**（279.56s，审查员自跑）；M-01/M-02/05/06/07 复跑同量级且硬门全过（M-01 p95=2.94ms、M-07 p95=4.64ms、M-06 cold=54.8/hot=47.8ms、M-02 p50=267.0ms）；Man gate 测试 1/1。AC-01..04、AC-07 全 pass（逐条 file:line 证据映射）；AC-05/06 归本审/merge。
- Findings：F1 major(doc-only)——spec §3 vs §3.1 P-04 自相矛盾 → **已修**（562cad7b，"部分交付"措辞）；F2 minor——"临时"诊断注释 → **已改永久命名取证面**（96236725）；F3 minor——auto-man worktree Cargo.lock 脏（依赖解析churn，非交付文件）→ **已还原**；F4 观察——100ms 门为病理绊线而非紧性能门（冻结预算如实记录，禁止引作端到端时延证据）→ 记录在案。
- pending_closeout_gates（merge 拥有）：SD-01 规范正式发布、落地后 landing checks（rebuild/重启或显式陈旧制品观察）、合并回归归属（本审已跑规范滤程=冻结受影响面；merge 不重跑全仓）。
- next:merge（T-07）。

### merge 收口摘要（2026-11-08，V-12 重建身份与证据折入）

- 交付：auto-ui master `96236725`（ff-only，9 文件 +1537/−128；SD-01 规范 `docs/specs/runtime/rq-execution-input-and-host-services.md` SHA256 前 16 = `2139d5bc1829127e`）；auto-man master `347c29f`。归档 `646d868f`。
- **V-12 重建身份**（交付时实测）：auto-ui-runtime 测试二进制 `276540856c47557a46591c0bb49f718ac06b41249ad22d289a9c2b1974220722`（滤程 290/0 用）；Man `auto.exe` `5178a7b44e901ac9cff88f5d83a1aa2f6cefd2310c9d62865e0fbf45e128f659`（144,615,424B debug；真入口探针用）；`auto-ui-tool.exe` `7aafb31acf21e84cc83001583c30537b6ef20ba769e1a7c99be072a749ec4941`。profile=cargo debug；依赖树=组内兄弟 auto-lang `a95d6dc41` / auto-web `2d69841` / auto-down `58ea33f`。
- M 指标终值（rqhost in-proc 真实采纳，代理终点具名）：M-01 p50=0.055/p95=3.18/max=4.42ms（n=40，硬门 p95≤100ms）；M-07 轻App p95=3.11ms（重载在流，硬门同）；M-06 冷 60.0ms/热 36.9ms；M-02 p50=263.7ms；M-05 进程 36.5MB（合并口径）。具名未测：M-03 GPU、高刷新、物理上屏、1/4/8 规模（归 OS063）。
- 证据处置：plan-artifacts/plan-063 按delete-at-merge 裁定删除（本节已折入全部持久数字与哈希）；构建中间物随 worktree 移除。landing≠deployment：release 制品（Man release exe 等）未重建——观察项记于 merge 收据。

## 10. 待澄清事项

- **T-05 收尾后具名边界（2026-11-08 round 3，非豁免、归 OS063/后继）**：M-03 GPU 时间（平台计时能力缺席）、高刷新适用面（无实测环境）、M-01 物理上屏终点、1/4/8 规模组合、V-10 端到端终值——见 spec §3.1 具名未测表。①旧红族全闭、②V-02/03/04 证据齐、③P-04 机制落地、④M-01/02/04/05/06/07 实测+V-12 身份绑定（round 3 收据）。
- r4当前未决：T-01冻结实际入口/预算/旧红当前适用及必要关闭合同；媒体/绘制/乱序资源详UI073。不把062的阶段性豁免挪作M1豁免。
- 当前正式来源UI060/061/062和OS054均已交付，旧r3“在途/未授权研究/Spec缺席”仅历史观察，不作为本轮阻塞；本轮未实施。
- Linux全平台/compositor、ACE/AGE独立进入门保持；无必须重新询问才能完成本次草案的问题。
