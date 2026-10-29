---
plan_id: PLAN-011
status: archived
feature_name: AutoGraphics二维完整玩法整链预算验证
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30
plan_revision: 2
completion_kind: delivered
delivery_commit: c4143af68697f65813cbea9fe941face3fa945bd
knowledge_commit: b5556e1696d029d0b4deca8748f985018be5740c
current_step: 9
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

知识：[现状](../../../../auto-lang/docs/specs/auto-graphics/overview.md)、[G2冻结规则与分母](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[860早期可行性](../../../../auto-lang/docs/specs/auto-graphics/game-runtime-feasibility.md)、[公共字符](../../../../auto-lang/docs/specs/auto-graphics/game-actions-and-character2d.md)、[触发/新局](../../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[精灵](../../../../auto-lang/docs/specs/auto-graphics/sprites-and-image-resources.md)、[G09](../../../../auto-lang/docs/specs/auto-graphics/gallery-sprites-and-frame-animation.md)。Spec权威仍为auto-lang，SD路径以该知识仓为根，本轮不改canonical正文。

观察：G09示例正常完整回调约31/32（26 move+2 held+1 jump edge+1 grounded+1 sprite）；object_move/despawn各1、event_next及每个getter各1，事件next即使无事件也计费，未读事件不跨tick排队。旧861 Spec表写move25，而当前交付源码有前置1＋v5 24＋最终支撑1=正常26，提前返回可能较少；T-01必须先用源码与32/33边界实验核准，不能挑便宜的旧表让整链go。精灵tick_begin只清写次数、帧状态跨tick保留，这是候选减少重复setter的依据，不是已验证整链优化。详细指纹/边界见[规划合同](../attachments/011/planning/g2-probe-contract.md)。

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

优先用原公开SDK/Host可观察统计；需要从既有VM读取诊断全局也只用于只读oracle，不作为免费玩法输入。无法直接读meter时用固定增量调用的32/33阈值与完整价格表交叉校准；不得为计量补丁引擎。每callback记录阶段/分支/调用数/校准credits/耗时/Run/tick/结果，初始化单列。r2用户已批准：固定公开Play不能取得精确steps/resumes时明确记unmeasured，保留1000steps/slice、100000steps/callback、120resumes/250ms的原SDK硬限及源码证据，不把硬限冒充实际消耗最大值；超额QueryBudget与超时/步数错分开。图像读回和断言成本单列，不计入正常运行CPU假装开销，也不能把实际Host帧准备剔除。

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
| V-04/S-04 | AC-03/04 work T-04/05 | 3次release/1debug、硬额/8ms、Pause/Step/20短Run/Stop/新局/旧代 | CPU窗口/校准credit/源和资源账独立可重算；精确steps/resumes按r2批准的不可观测标注及SDK原硬限独立说明，目标miss不隐去；weak owner/新Run初值及失败恢复 | 无真实OS输入或GPU timestamp承诺 |
| V-05/S-05 | AC-05 work T-05/06 | 独立consumer重放、decision输入/输出、缺G2条目/未测标go/partial/错误报告、源码或控制变异 | 相同固定输入可重放，结论与每条证据一致；“core通过但G2-09缺”不能变full_m2=true，负研究结论有反例与合法控制 | 不证明完整游戏发行或未检实现均不可能 |

实现者/模型未指定；work自查后同一唯一树独立上下文final review，未授权新agent/chat，不新建第二review树。review逐AC/V/S/SD核丢分母/静默放宽/平均掩峰值/计量归因/owner与scope；pass标merge冻结门pending。测试只本研究受影响面与必要正反例，OS抢焦点/置顶自动化暂停，GPU机器单实例，全AG留里程碑pending。根/AG公共代码零改，不跑根t/tf/tv/tu/docs_gen。

原始探针输出/trace/PNG/日志/consumer放`D:/autostack/plan-artifacts/auto-ui/plan-011/`，attachments仅md契约/摘要；研究fixture产品源可入research，raw证据不换扩展名入库。命令/exit/关键数字/路径+hash写摘要，内存/输出有界；新consumer准备不能共用005的target/SDK目录。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 固定004＋008消费/身份/SDK/计价差异与32/33边界、唯一树/零共享引擎改动，V-01 |
| AC-02 | work | G2-01..09完整映射、同程序G2-01..08真实整链/两候选/原几何/输入与像素敏感；G2-09不冒充通过，V-02/03 |
| AC-03 | work | 最坏callback与多事件/时序/超额last-good/新局/寿命、原限额下的通过或有据不通过，V-03/04 |
| AC-04 | work | 可重算分支预算/源码与SDK步数及resumes硬限/资源/CPU release与debug；精确steps/resumes按用户r2批准标unmeasured，硬限与8ms目标分开，SD-02有实证处理，V-04 |
| AC-05 | work | 有界独立重放、decision schema与缺口/后继建议/负控自洽，研究负结论合法且非全局不可能性，V-05 |
| AC-06 | review | 同树独立上下文核完整AC/V/S/SD/误导结论/私有业务/warnings/fmt，无未处置finding，merge门pending |
| AC-07 | merge | 冻结研究commit＋固定consumer受影响门/负控及有效复用通过后合auto-ui/master，不改005/006共享实现 |
| AC-08 | merge | 研究Spec/ledger/索引、本仓Plan归档与准确结论、guard clean清本树/分支/空组，恢复材料可追溯 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、004/008已交付 | 核基线/owner/固定consumer/来源与计价差异，建唯一ui-011树/独立准备目录 | AC-01 | [x] |
| T-02 | work | T-01 | probe-contract：全G2映射、两候选边界、input/peak/统计/ROI/决策schema与新入口先冻结 | AC-01..05 | [x] |
| T-03 | work | T-02 | research独立harness/Auto/原几何scene/既有CC0图来源/独立oracles，A完整直组合先建立 | AC-02 | [x] |
| T-04 | work | T-03 | A/B正常与尖峰/32-33/事件/故障/像素、价格归因，记录反例不顺手改引擎 | AC-02..04 | [x] |
| T-05 | work | T-04 | release/debug/20短循环/资源预算、fresh重放与decision完整性负控 | AC-03..05 | [x] |
| T-06 | work | T-05 | 结论/SD/后继接口缺口摘要、fmt/研究限定测试、提交，转独立review；负结论不删AC | AC-01..05 | [x] |
| T-07 | review | T-06 | 同树独立final review，核结论边界及全部V/S；merge门pending | AC-06 | [x] |
| T-08 | merge | T-07 pass | 冻结研究门/有效复用后仅合本仓研究，不带入005/006 WIP | AC-07 | [x] |
| T-09 | merge | T-08 | 按SD沉淀auto-lang知识/ledger/索引，本仓归档与真实decision/delivery | AC-08 | [x] |
| T-10 | merge | T-09 | Git Bash wt-guard clean后清唯一树/分支/空组，保原始证据和恢复收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-011 | plan_revision: 1 | outcome: pass | next: 用户启动work T-01/T-02`。0/10，本轮仅规划；未建树/写probe/编译/运行预算或GPU/agent。004＋008固定输入已交付；不等005/006，且只能与其共享GPU门串行、source/target/准备目录互不写入。G10a另由ag-examples/003认领。

2026-10-29 文档预检：两份计划各0..10节、10个T/8个AC/5个V与S、work/review/merge职责、依赖/SD/来源边界完整；本批48个本地链接可解析，本仓编号唯一/取号计数一致。G10a六个固定来源文件长度/SHA与Git源commit只读复核一致。未执行probe/内容/CPU/GPU/实现测试，此结果不是独立final review或功能通过。

2026-10-30 goal授权：用户 `/goal 完成计划UI-011` 授权 work→独立review→有界修复→merge/知识沉淀/清理全链；保持 r1 原范围和分母。主树代码无WIP，其他路线文档不归本Plan。唯一树 `D:/autostack/.wt/ui-011/auto-ui`，分支 `codex/plan-011-dev`，base `7aa33edbad072ae166063868423ceda9bc35296d`；固定consumer仍为0261闭包，独立准备目录 `D:/autostack/plan-artifacts/auto-ui/plan-011/consumer`。T-01进行中。

2026-10-30 T-01/T-02：固定consumer独立复制；`python tools/verify_plan004.py` exit0（1431项），`python tools/prepare_sdk.py --from-snapshot` exit0（681文件/17包），`python tools/verify_source.py` exit0（479包/29路径边/forbidden0）。首次未传准备参数失败后按工具要求补 `--from-snapshot`，没有改SDK。契约先于新实验冻结于 [probe-contract](../attachments/011/work/probe-contract.md)。唯一树/分支已实查，T-01/02完成；T-03研究夹具已建立，首次独立编译进行中，尚无整链预算/GPU实测通过。

2026-10-30 T-03：独立研究workspace/lock、原几何scene、A/B真实Auto、固定G09 CC0图/许可、首轮Host/GPU探针建立。独立锁保留固定consumer全部registry identities，仅研究PNG读回新增png0.17.16；首次从空锁生成曾漂移，已在编译前改为继承consumer锁并核对无丢失。core首轮2/2候选均tick1 QueryBudget且Frame/GPU完全保留；仍继续T-04/05，不把早Fault或isolated分支当完整核心通过。fresh-consumer从固定Git身份清单独立导出后verify_source exit0。

2026-10-30 work checkpoint：研究代码 `64edf5852bb0657902e0e326253fc893585ce041`，仅新增research独立workspace。原A/B均首回调QueryBudget；debug3/3、release各定向面、fresh consumer3/3通过，50个branch rows；32-pass/33-refuse、N+1三ID真实销毁、GPU/整帧last-good、20短Run/Stop与真实owner全部完成。3次release窗口max2587/2795/2711us，仅已测失败前缀/合法隔离分支；不声明原路线成功。`python research/autographics-g2-budget/tools/verify.py --controls` exit0，10坏报告拒绝。详见 [work results](../attachments/011/work/results.md)，local decision `D:/autostack/plan-artifacts/auto-ui/plan-011/decision.json` SHA256 `39fb222b82247acd86fc7c1392e13e5382e8e606c5838d502c239cbd879095fc`。

`stage: work | plan_id: PLAN-011 | plan_revision: 1 | outcome: blocked | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | base_commit: 7aa33edbad072ae166063868423ceda9bc35296d | code_commit: 64edf5852bb0657902e0e326253fc893585ce041 | dependency_revisions: fixed consumer0261ae9/SDK7b987ca | task_ids: T-01,T-02,T-03,T-05 complete; T-04/AC-04 exact meter open; T-06 readiness pending | evidence: attachments/011/work/results.md | blockers: user decision on exact per-callback steps/resumes observability | next_responsible_role: user then work/review`

`stage: goal | plan_id: PLAN-011 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-011/auto-ui | commit: 64edf5852bb0657902e0e326253fc893585ce041 | next: user`

2026-10-30 goal round2：重读§9/§10，原计量验收问题尚无用户答复。上一轮有实际代码/证据进展；本轮属于同一阻塞条件下无可执行后续阶段，不是进程等待。按waiting规则不执行实现/测试/复审/合并，不把自动继续消息视为验收口径授权。

`stage: goal | plan_id: PLAN-011 | plan_revision: r1 | round: 2 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-011/auto-ui | commit: 64edf5852bb0657902e0e326253fc893585ce041 | next: user`

2026-10-30 goal round3 blocked audit：上一轮为同一阻塞下无进展，本轮重读§9/§10仍无用户决定；不存在需要轮询的活动进程，也无不改变原验收要求的剩余阶段可执行。连续三轮为同一exact steps/resumes观察缺口及验收决定阻塞；保留研究commit/唯一树/原始证据，目标标blocked，Plan仍executing4/10。解阻动作：用户明确接受将该两项标unmeasured并保留SDK硬限继续交付，或提供/另行授权满足原要求的计量能力。自动goal继续不构成该决定。

`stage: goal | plan_id: PLAN-011 | plan_revision: r1 | round: 3 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-011/auto-ui | commit: 64edf5852bb0657902e0e326253fc893585ce041 | next: user`

2026-10-30 用户决定（异步问题原文回复）：**“接受不可观测标注，继续复审合并”**。用户已明确批准将固定consumer不能公开取得的逐callback精确steps/resumes记为unmeasured，保留原SDK硬限和其余真实测量。32credits、输入/玩法/原几何、两个候选、G2完整分母、G2-09缺公共能力、禁止引擎改动与full_m2_proven=false均不变。该答复解除上一轮验收口径阻塞，后续语义合同更新到r2，并复核受影响AC-04/V-04/T-04/T-06，不重跑无变化的运行实验。当前host get_goal仍返回blocked；尚未开始新work/review/merge阶段，等待用户以 `/goal resume` 恢复目标调度。

2026-10-30 r2合同修订：用户答复“接受不可观测标注，继续复审合并”并明确发送 `/goal resume`。仅更新§5.4、V-04/AC-04的精确steps/resumes观察口径为unmeasured+原SDK硬限，不改32credits/玩法/分母/作用仓或候选范围。沿用已验证的原始GPU、预算、性能、生命周期证据：研究Rust/Auto/scene/PNG/两锁/固定consumer/SDK输入不变；仅同步说明与报告来源身份。历史blocked记录保留，当前阻塞已解决。

`stage: new | plan_id: PLAN-011 | plan_revision: 2 | outcome: pass | changed: §5.4,V-04,AC-04 | authorization: explicit user acceptance and resume | next: work`

2026-10-30 r2 work readiness：T-04/T-06完成，6/10，work-owned AC-01..05均有直接源码/执行断言/原始证据，已批准的steps/resumes不可观测边界明确标注，未处理的工作内finding/必需覆盖缺口/用户决定均为空。GPU/32-33/像素/事件/寿命/CPU/fresh数据复用 [results](../attachments/011/work/results.md) 所绑定运行；未修改Auto/Rust/scene/PNG/Cargo锁/固定引擎与SDK。仅文档r2同步及线性rebase，range-diff四项全部`=`：7db7a49→ed317ab、31fa291→84ebaa6、64edf58→6039257、08cabcb→5ef611b。`verify.py --controls`在当前树exit0，10坏报告拒绝，依赖identity1431/SDK681/479包/29路径边仍通过。

`stage: work | plan_id: PLAN-011 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | base_commit: 7ec8f102917587ecd13851c3cd43172800fbf77e | code_commit: 5ef611b51699e26a79a282962f86ecba5f3a88f9 | dependency_revisions: fixed0261ae9/SDK7b987ca | task_ids: T-01..06 | evidence: r2-decision-gate.log; decision.json SHA256 a1ab99b42491a74397ecb306900ab7ef7aa5c49da9851d596c06da6f4f49abba | blockers: none | next_responsible_role: independent final review`

当前冻结Spec delta SHA256 `522d91bc43f10337e6c491806119a25e91b17db8e0fac76a80cc74d994f08c7c`。T-07 final review及T-08..10冻结门/知识沉淀/ledger/归档/guard清理仍pending；工作树干净，实施写入暂停。

`stage: goal | plan_id: PLAN-011 | plan_revision: r2 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-011/auto-ui | commit: 5ef611b51699e26a79a282962f86ecba5f3a88f9 | next: review`

2026-10-30 independent final review r2：`/root/ui011_final_review`在同一冻结树5ef611b完整核AC/V/S/SD，outcome needs_fix。UI011-R1：报告准入未拒绝空source_inputs/不存在commit/虚假first_fault及与raw矛盾的CPU统计与8ms分类；UI011-R2：缺逐callback源码派生/校准调用账及真实accepted意图观察。见[独立复审](../attachments/011/work/review-r2.md)。不是已批准的steps/resumes不可观测问题，也不否定有据no_go。T-04/05/06重开，3/10，其余原始运行证据仍可复用。

`stage: review | review_scope: final | reviewer_identity: /root/ui011_final_review | plan_id: PLAN-011 | plan_revision: 2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | reviewed_commit: 5ef611b51699e26a79a282962f86ecba5f3a88f9 | base_commit: 7ec8f102917587ecd13851c3cd43172800fbf77e | findings: UI011-R1,UI011-R2 | evidence: attachments/011/work/review-r2.md | pending_closeout_gates: research freeze gate/SD/ledger/archive/guard | next: work`

修复选择：R1把所有派生字段与完整manifest/可验证Git源/raw统计一致性作为同一准入不变量，并保留坏报告反例。R2在原A/B源码零改前提下新增隔离Auto诊断夹具，公开setter只输送意图计数；故障回调明确不得发布新诊断，预算采用原价/32-33实测校准的源码账，区分所需需求、已到达32/拒绝33和不可达后缀。仅运行新意图case/fresh定向重放、账目与报告负控；原Rust函数/Auto/几何/锁/PNG/性能输入未变部分复用，统一冻结门仍merge-owned pending。

2026-10-30 r2 repair1：UI011-R1/R2按[closure](../attachments/011/work/repair-r2.md)整体修复。报告与raw/source/Git一致性18负控通过；新增13意图rows的原/fresh定向case各1/1；852条源码派生/校准callback账显式区分normal/early/拒绝/未达后缀/无callback。原三测试函数/Auto/几何/性能/生命周期输入未变，明确复用其原始数据，不重复全局或3次性能。当前HEAD c53941a102e62013d4bc3fa9e5b14e9a846c430a、树干净，T-01..06完成6/10，未处置work-owned findings/必需gap/用户决定为空；新独立复审待完成。

`stage: work | plan_id: PLAN-011 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | code_commit: c53941a102e62013d4bc3fa9e5b14e9a846c430a | task_ids: T-01..06 | evidence: attachments/011/work/repair-r2.md | blockers: none | next_responsible_role: independent final re-review`

2026-10-30 final re-review2（c53941a）needs_fix：R1/accepted意图已关闭，R2账目出现阶段/身份回归。6个accepted峰值N被算成47credits/未来4事件3销毁；air相对t0被标绝对tick1/run空，实应jump-rise(run9,tick5)后的tick6。见[复审2](../attachments/011/work/review-r2-repair1.md)。连续两次final失败，执行前已完成[诊断检查点](../attachments/011/work/diagnosis-r2.md)，明确case标签/同builder自证为何漏检、整个阶段族影响及独立hard-cap/首队列/轨迹身份oracle调整。T-04/05/06重开3/10；下一修复只工具账目，不重跑无影响运行族。

`stage: review | review_scope: final | reviewer_identity: /root/ui011_final_review | plan_id: PLAN-011 | plan_revision: 2 | outcome: needs_fix | reviewed_commit: c53941a102e62013d4bc3fa9e5b14e9a846c430a | findings: UI011-R2 phase/identity; UI011-R1 closed | evidence: attachments/011/work/review-r2-repair1.md | next: work`

2026-10-30 repair2 readiness：先完成两次失败[诊断与设计/oracle调整](../attachments/011/work/diagnosis-r2.md)，再提交 cb02a86ea272bb167858876ccfb2a2d7cc058cd6。首update按实际attempted tick推空队列而非标签；6处accepted峰值N由错47修为32，N+1保留refused33与假设需求；轨迹以已绑定jump-rise Run/tick锚定，debug首air恢复run9/tick6。独立语义约束+真实坏ledger变异通过，`verify.py --controls` exit0/20拒绝；852账目条目、accepted最小需求>32为0，原/fresh正控均保留。无运行源码/SDK/输入/CPU/owner变化，沿用原始验证；R1及accepted意图复审已关闭，本轮只修R2阶段身份。当前work-owned open findings/必需gap/决定为空，6/10，冻结树干净，第三次final review pending。

`stage: work | plan_id: PLAN-011 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | code_commit: cb02a86ea272bb167858876ccfb2a2d7cc058cd6 | task_ids: T-01..06 | evidence: attachments/011/work/diagnosis-r2.md; repair2-gate.log | blockers: none | next_responsible_role: independent final re-review`

2026-10-30 第三次独立final review PASS：冻结 cb02a86ea272bb167858876ccfb2a2d7cc058cd6，独立上下文/root/ui011_final_review；[收据](../attachments/011/work/review-r2-repair2.md) SHA256742216d579bc1d563c85b08f9961abcca6e4e87fb040ba73c908a106890bc924。R1/R2全部关闭，AC-01..06/V/S/SD通过，无在范围内finding；20坏报告负控及独立语义B47/首getter/错airRun/结束态巡逻变异被拒，非同builder自证。T-07完成7/10，状态reviewed。SD SHA256522d91bc43f10337e6c491806119a25e91b17db8e0fac76a80cc74d994f08c7c。

`stage: review | review_scope: final | reviewer_identity: /root/ui011_final_review | plan_id: PLAN-011 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-011/auto-ui | branch: codex/plan-011-dev | reviewed_commit: cb02a86ea272bb167858876ccfb2a2d7cc058cd6 | base_commit: 7ec8f102917587ecd13851c3cd43172800fbf77e | dependency_revisions: fixed0261ae9/SDK7b987ca | spec_delta_sha256: 522d91bc43f10337e6c491806119a25e91b17db8e0fac76a80cc74d994f08c7c | acceptance_results: AC-01..06 pass | findings: none | pending_closeout_gates: frozen research/merge/SD/ledger/archive/guard | next: merge`

2026-10-30 merge checkpoint PLAN-011:r2：prepared=final pass cb02a86 + frozenSD522d91bc...8c7c；landed=auto-ui/master快进至c4143af68697f65813cbea9fe941face3fa945bd，旧428fac→c4143af，6提交range-diff全部`=`（merge-range-diff.txt），research树与review冻结逐字节相同。冻结研究门4 selected/4 passed（43.48s），20报告负控通过；GPU无窗口、无OS自动化、未跑根cargo或全AG。main smoke为同研究报告准入，固定consumer1431/SDK681/479包/29边仍通过。

SD-01/02已于auto-lang/master b5556e1696d029d0b4deca8748f985018be5740c快进交付：新增g2-full-loop-budget-feasibility.md，game-actions-and-character2d.md正常move26更正，module overview/plans索引回写；spec-index.py执行后索引无语义变化（62projects，原有ungrouped警告未扩域处理）。知识commit由cb859e78→b5556e169 range-diff `=`跟进主仓簿记变化，未改Rust/API。

ledger_refreshed=auto-lang/.autoos/specs.json（tracked）经知识worktree受保护离线投影v144→145（服务8080 GET超时且无listener，无可达store writer）。byte-exact backup `D:/autostack/plan-artifacts/auto-ui/plan-011/ledger-backup-v144.json` SHA25664a3ba942f8af749bf6c72be52dc8032f1aac06ee984581f3ff95a923e8d15fd；结果SHA256cf77747b3d5af7eb69e68460628bc8baa87c433424667ede6150db219e7694a7。仅新增UI011-G2-BUDGET/designs、UI011-TESTS/tests、UI011-R2/reviews、UI011-MERGE/reports并复用P861-CHARACTER-ACTIONS，所有file/tags/整数时间戳/六段schema/readback通过；未触及部分按原始span逐字节保留，version只+1。已在main核源与metadata/hash一致。

研究结论no_go_current_profile只限已检两候选；release最大2.795ms仅失败前缀/合法隔离分支，debug最大18.119ms；exact steps/resumes按用户r2批准unmeasured。G2-09仍missing_public_capability/full_m2_proven=false。T-08/09完成9/10，实际归档/文档提交门随本checkpoint完成；T-10 guard移除/缓存回收尚pending，不提前标清理成功。主树其他计划文档WIP均为进入时既存并保留，未混入本Plan提交。

`stage: merge | plan_id: PLAN-011 | plan_revision: 2 | checkpoint: landed,ledger_refreshed | delivery_commit: c4143af68697f65813cbea9fe941face3fa945bd | knowledge_commit: b5556e1696d029d0b4deca8748f985018be5740c | outcome: cleanup_pending | next: archive commit then guarded cleanup`

## 10. 待澄清事项

无新增产品决策。计价/全组合能否成立是本研究待实测问题，不是既成go；最多两候选，失败以有据no_go/requires_capability交付，不顺势扩大成引擎修复。G2-09公共声音/HUD尚缺，不能预先承诺完整M2可行；需要新接口/预算或正式游戏时据本结论另立Plan。正式游戏/资源发行仍独立仓，未来ID另核，不在本轮创建。


2026-10-30 历史待确认（已由上述用户答复解决）：§5.4/AC-04要求逐callback精确steps/resumes，固定0261的Play/Session公开接口不提供这些计数，且本计划禁止改引擎。当前证据可证明SDK原硬限执行，但不能提供真实消耗最大值。是否接受本次有据no_go研究明确记为unmeasured、保留硬限与其余实测后继续独立复审/合并？或保留精确计数要求、暂不合并，待另行授权的计量能力补齐。未经答复不视为降低验收要求；4/10，保持executing，未复审/合并/改canonical Spec或ledger/清树。


当前状态：用户已确认计量口径并明确发送 `/goal resume`；验收阻塞已解除，无待澄清事项。应用get_goal仍返回旧blocked，不能由工具改active；依用户明确继续指令按无自动续跑模式在本轮顺序完成，不虚构active状态、不再要求重复resume。实施与预算约束保持原范围。
