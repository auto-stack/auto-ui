---
plan_id: PLAN-011
status: drafting
feature_name: AutoGraphics二维完整玩法整链预算验证
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-004, auto-ui/PLAN-008, ag-examples/PLAN-002]
affects: [auto-ui-research]
supersedes_spec_components:
  - docs/specs/auto-graphics/game-actions-and-character2d.md
new_spec_components:
  - docs/specs/auto-graphics/g2-full-loop-budget-feasibility.md
touched_goals: [GOAL-008]
---

# PLAN-011：2D完整玩法整链预算验证

## 0. 变更摘要

用已交付004＋008的公共sprites2d profile验证同一真实Auto程序能否同时承担跑跳/碰撞、巡逻敌人、三物品一次收集、胜负/新局、跟随相机和精灵；逐callback计量并找出最坏分支。只交付可重放研究夹具、数值/像素/预算反例与结论，不交付正式游戏或修改公共引擎。声音/HUD的G2-09缺公共服务，必须单列缺口，不能以意图计数冒充真实播放/UI通过。

## 1. 目标

1. 将原G2-01..09完整分母逐项映射，检验G2-01..08在同一Run/同一callback链的可组合性；不是把G09与G11两份独立绿报告拼起来。
2. 明确32credits、源/步数/时限、节点/槽/触发对、资源与CPU的实际最坏值；正常route、同时收集/敌人/跳跃/换帧等尖峰须分别报告。
3. 提供`go_core`、`no_go_current_profile`或`requires_capability`的带证据决策，明确G2-09/8ms目标/最终美术/发行是否仍未证实。负结论是合法研究交付，不能为了go删分母、改价格或静默抬限。

非目标：正式游戏仓/关卡产品、美术制作、声音/HUD实现、SDK/VM/AG公共API/可信单元/帧协议/预算修改、Gallery目录/旧项锁升级、3D、跨端/生产UI/HIR/ACE。研究原型留本仓研究目录，完整游戏以后独立仓立Plan。

## 2. 架构方案

唯一实现/顺序复审树拟`D:/autostack/.wt/ui-011/auto-ui`，分支`codex/plan-011-dev`；本轮不创建。实现只新增`research/autographics-g2-budget/`独立非成员workspace及必要文档；冻结引擎consumer与SDK放本Plan独立准备目录，两个现有experimental工作区、src/api、根Cargo与005/006实现只读。新harness有自己的Cargo.toml内`[workspace]`/Cargo.lock，不加入根或AG工作区，target/SDK准备/报告不与005/006/Gallery共用。

真实输入→既有public loader/Session→同一Auto app_start/app_update→既有Host prepare/accept/publish→状态、预算和GPU观察。Rust只加载、投递固定输入、观察/断言与收据，不在Rust里巡逻、算收集、触发胜负、代选帧或把Host反算结果喂回Auto。固定consumer源不得打补丁；需要引擎能力则报告后继owner，不在本计划顺手修。

## 3. 技术栈

已核输入：`experimental/autographics-auto-driver/api/{character2d_v1,runtime_objects_v1,runtime_triggers_v1,sprite2d_v1}.at`、`src/authoring/{play,host,behavior}.rs`、`host/viewport.rs`、SDK与两AG锁；参考只读固定ag-examples交付d1b4f9baf6f83575af4c669efa9b17f507d9003d（G09代码/素材/许可，不等待其清理记录） `examples/g09-sprites-and-frames/character/main.at`、`examples/g11-runtime-objects/2d/main.at`、`checks/g09/gallery_runtime.rs`。不能从当前005开发树取新能力。

拟新增研究路径（当前均不存在）：`research/autographics-g2-budget/{Cargo.toml,Cargo.lock,fixtures/,tests/full_loop.rs,tools/verify.py,README.md}`。fixtures含真实.at/scene/既有CC0图及来源清单；reports/decision原始输出写本地artifact目录，摘要才入库。永久命令/适配签名由T-02对实际公开入口冻结，不能把本节待建路径当已运行证据。

## 4. 需求分析与背景调查

用户2026-10-29明确要求规划“2D完整玩法整链预算验证”，同时另规划Gallery G10a，暂不实施。本轮仅取号/计划/规划附件/相关引用，不建树、写harness或Auto、生成资源、编译/GPU/窗口/agent/发布。未指定实现者或模型，不创建独立chat。

auto-ui调查HEAD `a448d5716bf3a108def48033383d1c1a1f852894`；004 delivery `e0fce13810f9e1af28182950d62cfce8722aa30a`，008身份修复 `0261ae9dacf786beb6dceb630be6af267f242625`均已合入；只固定消费0261ae9这一已交付闭包，含004公共功能，不追master开发tip。G09锁记录SDK旧pin/681文件/17包、479依赖/29路径边及identity1431；其runner/identity是可核消费线索，T-01仍核真实字节/格式与replay，不以字符串代替验证。

知识：[现状](../../../auto-lang/docs/specs/auto-graphics/overview.md)、[G2冻结规则与分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[860早期可行性](../../../auto-lang/docs/specs/auto-graphics/game-runtime-feasibility.md)、[公共字符](../../../auto-lang/docs/specs/auto-graphics/game-actions-and-character2d.md)、[触发/新局](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[精灵](../../../auto-lang/docs/specs/auto-graphics/sprites-and-image-resources.md)、[G09](../../../auto-lang/docs/specs/auto-graphics/gallery-sprites-and-frame-animation.md)。Spec权威仍为auto-lang，SD路径以该知识仓为根，本轮不改canonical正文。

观察：G09示例正常完整回调约31/32（26 move+2 held+1 jump edge+1 grounded+1 sprite）；object_move/despawn各1、event_next及每个getter各1，事件next即使无事件也计费，未读事件不跨tick排队。旧861 Spec表写move25，而当前交付源码有前置1＋v5 24＋最终支撑1=正常26，提前返回可能较少；T-01必须先用源码与32/33边界实验核准，不能挑便宜的旧表让整链go。精灵tick_begin只清写次数、帧状态跨tick保留，这是候选减少重复setter的依据，不是已验证整链优化。详细指纹/边界见[规划合同](attachments/011/planning/g2-probe-contract.md)。

取号时.next-id=009，但已有本仓archived/010，max本地号=010；独占扫描活动/archived/archive后本次取011、计数推进012。009不自行回填，007/008及其它路线WIP原样保留，不把其文档混入本提交。

## 5. 详细设计

### 5.1 首任务与固定预算

T-01核唯一owner/基线/只读consumer与G09固定代码/素材/许可、源identity1431/SDK681/两锁/栈、既有角色/对象/事件/精灵实际计价与可观察统计入口；先核现有32/33正反回执与源码价格，记录旧Spec差异；本研究新鲜边界实验在T-04执行，不先于T-02冻结契约。若所需来源不可证则blocked，不切换当前SDK或005 tip。

T-02写`attachments/011/work/probe-contract.md`，冻结同一G2程序的输入序列、路径坐标、状态/事件延迟、loop候选、尖峰/错误场景、计量方法、像素ROI/容差与决策schema。保持32credits/callback、1000steps/slice、100000steps/callback、120resumes/250ms、用户源256KiB/受管96KiB/组合352KiB、30s初始化/2s tick、8ms pump目标及004资源上限；dt=1/60、速度4、重力20、jump8、clearance0.002。统计须区分callback与整个tick、VM/Host准备/绘制/读回、debug与release、硬限制与8ms目标，不能拿片段时间代替整tick。

### 5.2 全分母与路线

基础几何沿原G2：边界x=[0,30],y=[-4,12]，出生(2,1)、半宽0.4/半高0.5；三段地面[0,8]/[10,18]/[20,30]×[-1,0]，顶平台[12,15]×[1,1.25]，墙x0/30与顶y10，坑下界y=-3；敌人x11..17,y0.5以1单位/秒巡逻，接触失败；金币(4,1)/(13,2.25)/(24,1)、目标(28,1)、半径0.5；相机视区12×8跟随夹界。探针先证路线可达，不因原坐标困难改平台/金币或移走敌人。坏境/压力另建受控fixture，不能改基础route后仍引用原分母。

| 分母 | 本计划可观察内容 | 尚不等同于 |
|---|---|---|
| G2-01..03 | 真输入左右/松手、单地跳/空中拒二跳、地墙顶/边缘/坑失败 | OS键鼠自动化，本期仍暂停 |
| G2-04..06 | 同一Auto更新巡逻、三不同实例首次收集/真销毁、敌人/越界失败、未集齐终点不赢、集齐赢/Enter新Run | 分时演示、跨tick人为暂停运动或免费Rust业务 |
| G2-07/08 | 同tick运动/相机/正确idle/run/rise/fall与方向像素；Stop/新Run初值 | 最终关卡美术/完整游戏产品 |
| G2-09 | 记录同一真实Auto状态产生的有限jump/coin/HUD变更意图及其分支额外需求，明确公共服务缺失 | 真音效/HUD或该条验收pass；意图计数不是播放/显示证据 |

事件严格保留既有N步几何→N+1回调读取的单tick延迟；不要求不存在的零延迟，也不额外分摊到后续回调/丢弃未读事件。正常route与代表峰值分别覆盖移动+跳/落地+换帧+敌人移动+收集、三币同接触、敌人/终点与收集邻近，记录发生的实际callback，不用“平均≤32”掩盖某步越限。

### 5.3 有界候选与决策

最多比较两种同语义候选：A按现有G09+G11公共接口直接组合；B只允许在用户Auto中利用已确认的状态保持/有限缓存/消除重复读取与无变化setter，或其它等价公共调用安排。两者同几何/输入/事件语义/采样状态，全部Auto玩法；不能改可信单元/价格、跳过每tick角色/敌人更新、降低采样率、免费扫描/新批处理接口、把事件计分挪到Rust或借“优化”删掉必需呈现。T-02冻结B的差异与等价判据，发现预算不足时不可无限尝试第三种。

负结论仍须跑完可执行的分支压力/合法对照与当前限制定位：不能只构造一个故意超额脚本就声称所有合法实现不可能。结论限定为“当前profile＋这两种已检候选/冻结分母下未满足”，不提供全语言数学不可能性证明。若A先失败，后续可用分支隔离、错误注入和等价B补齐原因；不能以probe Fault阻止必要对照。

决策必须分栏：`core_semantics`、`hard_budgets`、`target_8ms`、`g2_09_public_services`、`full_m2_proven`。go_core仅在G2-01..08完整同程序/所有必需峰值及硬额通过时成立；超额/掉事件/删语义为no_go_current_profile；缺公共能力为requires_capability，可与核心go/no_go同时报告。研究AC/V通过指验证与结论诚实完备，不要求两个候选游戏均成功：预算Fault导致后续状态不可达时，逐条记not_established_budget_fault、最早失败输入/trace和有界分支隔离对照；不得把未到达状态标语义pass或删case。所有必需分母/对照仍须有执行记录，核心未完整通过绝不go_core。G2-09当前missing_public_capability、full_m2_proven恒不得由本研究置true；8ms不达必须单列missed及建议，不用2s硬额通过冒充性能目标达标。

### 5.4 计量、发布与重复性

优先用原公开SDK/Host可观察统计；需要从既有VM读取诊断全局也只用于只读oracle，不作为免费玩法输入。无法直接读meter时用固定增量调用的32/33阈值与完整价格表交叉校准；不得为计量补丁引擎。每callback记录阶段/分支/调用数/credits/步数/resumes/耗时/Run/tick/结果，初始化单列；超额QueryBudget与超时/步数错分开。图像读回和断言成本单列，不计入正常运行CPU假装开销，也不能把实际Host帧准备剔除。

基础route+无输入对照/尖峰/最坏tick必须真实Auto→Host→GPU；关键检查点比较坐标/相机/实例/计数与实际ROI/精灵轮廓，不只metadata。坏输出/超额/故障保持整个last-good；Pause/neutral Step/Stop/新Run/旧句柄隔离闭合，20次短Run/Stop查实际VM/资源owner并区分Editor/外部pin合法保留。正常暖帧不重解码/传静态图，可绑定004未变底层证据并加本研究整链抽样。

T-02冻结可复现固定输入与种子；每候选每条基础route最多2400逻辑tick，性能固定3次release完整成功route/代表失败分支及1次debug对照，20轮寿命仅短循环，不跑全AG。若候选早Fault，按同一有限列表取失败前数据/隔离对照，不能无限重启。记录机器/构建/负载/数据窗口/raw摘要，CPU与GPU计时不可混用。失败后精确恢复源重放合法正控，证据绑定固定引擎/研究commit/资源/配置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang知识仓） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/g2-full-loop-budget-feasibility.md | 独立角色/示例可行→冻结完整G2核心与尖峰的实测/不通过原因/能力缺口，完整M2仍未证实 | 决定后继公共服务或游戏Plan | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/game-actions-and-character2d.md | 旧表move25→按T-01/04确证的正常完整支撑路径实际价格与提前返回边界更正文档，若原表被证实正确则记录无需修改 | 不靠过期价格得go，不改运行实现/限额 | AC-01/04 |

## 6. 测试设计

待T-02..05建立新入口：`cargo test --manifest-path research/autographics-g2-budget/Cargo.toml --locked --offline --test full_loop -- --test-threads=1`；`python research/autographics-g2-budget/tools/verify.py --controls`。实际子case/consumer-root参数与schema由T-02先冻结，本轮均未运行；0selected/缺分母/旧runner/错结论/缺峰值/超时无报告不算绿。负研究结论用expected outcome与证据链验证，不要求候选游戏本身伪造成功。

| V/必需S | AC/owner_stage/启用任务 | 输入→实际入口 | 区分性oracle/关闭条件 | 边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01 work T-01/02 | 固定0261consumer、G09图/许可、32/33调用阈值；旧Spec25与源码正常26 | 两锁/identity/SDK/actual code相符；计价与边界一致，错误pin/漏case拒绝；探针与005代码零交叠 | 只读来源不等于本研究已跑 |
| V-02/S-02 | AC-02 work T-03/04 | 同一Auto完整G2正常成功/失败route、两候选同输入、未集齐终点/巡逻/松手/空中重跳 | 原G2-01..08逐条真实状态与独立geometry/像素oracle；冻结敌人/漏币/免费Rust替代、空中run等错误被分辨 | G2-09缺口单列，不假pass |
| V-03/S-03 | AC-02/03 work T-04 | 移动/敌人/换帧/单币与三币/危险邻近等最坏callback；same-tick多事件与次步读 | 列实际各分支用量、队列读取/三ID各一次与发布时序；33credit/丢事件变异非零且整last-good，恢复正控绿 | 当前离散触发，不新增一般物理 |
| V-04/S-04 | AC-03/04 work T-04/05 | 3次release/1debug、硬额/8ms、Pause/Step/20短Run/Stop/新局/旧代 | CPU窗口/最大步数/credit/源和资源账独立可重算，目标miss不隐去；weak owner/新Run初值及失败恢复 | 无真实OS输入或GPU timestamp承诺 |
| V-05/S-05 | AC-05 work T-05/06 | 独立consumer重放、decision输入/输出、缺G2条目/未测标go/partial/错误报告、源码或控制变异 | 相同固定输入可重放，结论与每条证据一致；“core通过但G2-09缺”不能变full_m2=true，负研究结论有反例与合法控制 | 不证明完整游戏发行或未检实现均不可能 |

实现者/模型未指定；work自查后同一唯一树独立上下文final review，未授权新agent/chat，不新建第二review树。review逐AC/V/S/SD核丢分母/静默放宽/平均掩峰值/计量归因/owner与scope；pass标merge冻结门pending。测试只本研究受影响面与必要正反例，OS抢焦点/置顶自动化暂停，GPU机器单实例，全AG留里程碑pending。根/AG公共代码零改，不跑根t/tf/tv/tu/docs_gen。

原始探针输出/trace/PNG/日志/consumer放`D:/autostack/plan-artifacts/auto-ui/plan-011/`，attachments仅md契约/摘要；研究fixture产品源可入research，raw证据不换扩展名入库。命令/exit/关键数字/路径+hash写摘要，内存/输出有界；新consumer准备不能共用005的target/SDK目录。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 固定004＋008消费/身份/SDK/计价差异与32/33边界、唯一树/零共享引擎改动，V-01 |
| AC-02 | work | G2-01..09完整映射、同程序G2-01..08真实整链/两候选/原几何/输入与像素敏感；G2-09不冒充通过，V-02/03 |
| AC-03 | work | 最坏callback与多事件/时序/超额last-good/新局/寿命、原限额下的通过或有据不通过，V-03/04 |
| AC-04 | work | 可重算分支预算/源码步数/资源/CPU release与debug、硬限与8ms目标分开，SD-02有实证处理，V-04 |
| AC-05 | work | 有界独立重放、decision schema与缺口/后继建议/负控自洽，研究负结论合法且非全局不可能性，V-05 |
| AC-06 | review | 同树独立上下文核完整AC/V/S/SD/误导结论/私有业务/warnings/fmt，无未处置finding，merge门pending |
| AC-07 | merge | 冻结研究commit＋固定consumer受影响门/负控及有效复用通过后合auto-ui/master，不改005/006共享实现 |
| AC-08 | merge | 研究Spec/ledger/索引、本仓Plan归档与准确结论、guard clean清本树/分支/空组，恢复材料可追溯 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、004/008已交付 | 核基线/owner/固定consumer/来源与计价差异，建唯一ui-011树/独立准备目录 | AC-01 | [ ] |
| T-02 | work | T-01 | probe-contract：全G2映射、两候选边界、input/peak/统计/ROI/决策schema与新入口先冻结 | AC-01..05 | [ ] |
| T-03 | work | T-02 | research独立harness/Auto/原几何scene/既有CC0图来源/独立oracles，A完整直组合先建立 | AC-02 | [ ] |
| T-04 | work | T-03 | A/B正常与尖峰/32-33/事件/故障/像素、价格归因，记录反例不顺手改引擎 | AC-02..04 | [ ] |
| T-05 | work | T-04 | release/debug/20短循环/资源预算、fresh重放与decision完整性负控 | AC-03..05 | [ ] |
| T-06 | work | T-05 | 结论/SD/后继接口缺口摘要、fmt/研究限定测试、提交，转独立review；负结论不删AC | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立final review，核结论边界及全部V/S；merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结研究门/有效复用后仅合本仓研究，不带入005/006 WIP | AC-07 | [ ] |
| T-09 | merge | T-08 | 按SD沉淀auto-lang知识/ledger/索引，本仓归档与真实decision/delivery | AC-08 | [ ] |
| T-10 | merge | T-09 | Git Bash wt-guard clean后清唯一树/分支/空组，保原始证据和恢复收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-011 | plan_revision: 1 | outcome: pass | next: 用户启动work T-01/T-02`。0/10，本轮仅规划；未建树/写probe/编译/运行预算或GPU/agent。004＋008固定输入已交付；不等005/006，且只能与其共享GPU门串行、source/target/准备目录互不写入。G10a另由ag-examples/003认领。

2026-10-29 文档预检：两份计划各0..10节、10个T/8个AC/5个V与S、work/review/merge职责、依赖/SD/来源边界完整；本批48个本地链接可解析，本仓编号唯一/取号计数一致。G10a六个固定来源文件长度/SHA与Git源commit只读复核一致。未执行probe/内容/CPU/GPU/实现测试，此结果不是独立final review或功能通过。

## 10. 待澄清事项

无新增产品决策。计价/全组合能否成立是本研究待实测问题，不是既成go；最多两候选，失败以有据no_go/requires_capability交付，不顺势扩大成引擎修复。G2-09公共声音/HUD尚缺，不能预先承诺完整M2可行；需要新接口/预算或正式游戏时据本结论另立Plan。正式游戏/资源发行仍独立仓，未来ID另核，不在本轮创建。
