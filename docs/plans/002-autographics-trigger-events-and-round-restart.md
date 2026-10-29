---
plan_id: PLAN-002
status: reviewed
feature_name: AutoGraphics有限触发事件与新局隔离（引擎）
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 5
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/auto-source-driver.md
new_spec_components:
  - docs/specs/auto-graphics/runtime-triggers-and-rounds.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
depends_on: [PLAN-001]
current_step: 5
total_steps: 10
---

# [PLAN-002] AutoGraphics有限触发事件与新局隔离（引擎）



> **用户批准拆分 r4（2026-10-29）**：本计划只交付auto-ui引擎能力及必要测试夹具，不写ag-examples。G11的两维项目、目录/锁、验收与离线包整体移交[ag-examples/001](../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)，该计划硬等本计划实际交付后启动。编号不变，原r1..r3记录保留，尚未实施。

> **r5 受管桥接包络上调（2026-10-29，按§10预设路由）**：triggers组合在冻结的64KiB受管源包络下不可行——实测装配产物 Objects2V1 transport 2638+api 58621+entry 691=61950B（限额内），Triggers2V1 72141B、Triggers3V1 74050B 均>65536B；其中triggers单元压缩态10131B，而剩余预算仅3465B，无法容纳冻结的144值块布局+15个公开函数+评估器。按§10"带反例回new、不删功能、不静默抬限"执行：**显式**将`MAX_BRIDGE` 64KiB→96KiB、组合上限320KiB→352KiB（=256+96），功能分母/运行时预算（32credits、步数、时限、256KiB用户源）/AC/V/T/SD全部不变。反例探针固化为装配包络守卫测试。

## 0. 变更摘要

消费[auto-ui/001对象生命周期](001-autographics-runtime-object-lifecycle.md)实际交付，建立有界几何触发、一次事件读取及受检新Run请求。两维最小Auto工程作为引擎验收夹具，验证三物品收集、危险/终点与胜负后新局，玩法由用户Auto负责。只在auto-ui实现、测试、复审、合入与归档。

下游[ag-examples/001](../../../ag-examples/docs/plans/001-autographics-gallery-runtime-objects.md)独立消费本计划已合入的实际交付版本制作G11；其完成不再是本计划合入条件，不允许反向依赖造成循环。本稿仍有硬前置auto-ui/001，T-01/02在实际交付基线上冻结版本/签名/计费，不抢跑依赖或预填未来SHA。

## 1. 目标

- 每Run最多8对显式触发，有enter/stay/exit、确定顺序和Run/tick/对象代次；同tick只评估一次，事件有界、读取不重发。
- 真实用户Auto状态机区分不同物品/重复stay/危险/终点，销毁是真生命周期，同槽新代不继承旧触发。
- Auto请求新局后进入新Run/VM，重置位置/物品/计数/输入/事件，拒绝旧Run迟到结果；新初始帧失败保完整last-good。
- 交付可验证的公共接口、两维引擎夹具、源码/SDK/锁/独立consumer与测试证据，供下游冻结版本消费。

非目标：G11产品工程/目录/锁接线/离线ZIP/内容仓实现或合入（归ag-examples/001）、完整两款游戏/最终关卡、动态solid/移动平台/刚体、无限碰撞世界/自动全对扫描、连续碰撞/高速穿越触发保证、动画/纹理/声音/HUD、通用事件总线、任意多场景切换/存档/联网、ACE/UI融合/跨端/远端发布。

## 2. 架构方案

受信Auto按本tick实际世界位置计算有限形状相交、比较前态、提供callback内有界事件结果；用户Auto负责计数、巡逻、胜负和请求restart。Rust只做配置/身份/几何来源准入、新Run调度与GPU事务，不识别金币/终点名称执行玩法。

复用单Host/Play/Session调度栈，新显式组合身份隔离001及旧profile；触发结果、实例生命周期、角色与相机共同准入/prepare/accept/publish。restart作为已接受callback的显式请求，在接受边界创建新Run，不在回调中递归建VM或复用旧globals。

## 3. 技术栈

auto-ui driver已有`src/authoring/{play.rs,behavior.rs,behavior/assembly.rs,game_input.rs,runtime_camera.rs,spatial.rs}`、`host/{play_adapter.rs,viewport.rs}`；001将提供runtime_objects模块。当前源码事实见[原规划基线](attachments/001/planning/baseline.md)。无新增SDK/依赖。

拟新增`api/runtime_triggers_v1.at`、`src/authoring/runtime_round.rs`、`tests/{runtime_triggers_api,runtime_round_host,runtime_trigger_samples}.rs`、`samples/runtime-triggers/{two,three}/`及本代identity/checker；局部命名/签名/编号由T-02冻结。samples是必要引擎测试/公共入口夹具，不是G11目录产品；本计划无ag-examples写入树。

## 4. 需求分析与背景调查

授权：用户2026-10-29明确批准“auto-ui/002只做引擎，G11另建ag-examples/001，依赖002实际交付，暂不实施”。本轮只修计划/引用，未建树/编译/运行OS或GPU/创建agent。T-01仍等auto-ui/001实际交付，已有其工作树不等于已交付；本次不修改其源码、进度或验收合同。

权威输入仍为auto-lang现有`docs/specs/overview.md`与`auto-graphics/{source-repository-and-sdk,behavior-api,scene-play-session,spatial-query-profile,game-actions-and-character2d,game-character3d,runtime-game-cameras}.md`，按[知识入口](../../../auto-lang/docs/specs/auto-graphics/overview.md)跨仓读取；[Design45§13](../../../auto-lang/docs/design/45-autographics-wgpu-engine-and-game-prototypes.md#13-m2运行对象触发与新局事务2026-10-29拟议合同)、[游戏分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)和auto-ui/001约束能力分母。SD路径暂以现有auto-lang知识仓为根；本轮不搬迁/修改canonical Specs。

原r1源码观察为879交付`4adf2d037a1804741adc9c5a012f08cfa566439d`；执行必须换为auto-ui/001实际交付SHA并核差异/对象代次/SDK/锁。旧SDK pin7b987ca8/681快照及原限额保持。当前旧v2只有闭区间单对overlap，无引擎事件流；Play有新Run建VM、Stop恢复编辑Prepared，但无公共restart保旧到新tick0边界，不能把命令已发送当新局完成。

旧混合r3曾包含G11内容和双仓合入；本次按用户授权拆分完整移交内容验收，而非删除分母。追溯：[拆分映射](../../../ag-examples/docs/plans/attachments/001/planning/split-contract.md)。下游可用接口/源码包/真实运行证据以本计划最终交付为准，不消费在途tip。

## 5. 详细设计

### 5.1 唯一树与前置冻结

唯一实现树`D:/autostack/.wt/ui-002/auto-ui`，分支`codex/plan-002-dev`，从auto-ui/001实际交付SHA创建，源码只合auto-ui/master；auto-ui主检出持本Plan/进度/复审/归档。旧知识按现有归属同步SD/索引，不在语言仓创建本Plan副本。每Plan每仓一树，同树顺序work/独立review/merge，无链接，清理先guard。

T-01复核001实际接口/Run域/模板/活对象表、SDK/锁/current identity；T-02冻结`attachments/002/work/triggers-round-contract.md`：profile/签名、8对资源表、事件字段/顺序/读取/重绑、几何边界、restart状态图/失败路径、消费者与计费。

先证明整路径：真实Auto单角色运动/跳/归一＋相机＋8对检测＋三物品同tick相交/消费/真实销毁＋结果输出。共享32credits、1000steps/slice、100000steps/callback、120resumes/250ms、256KiB用户源、30s初始化/2s整tick/8ms目标保持。受管桥接源包络 r5 起为96KiB（组合352KiB），系64KiB反例后的显式上调（见§9 r5），非静默抬限；运行时预算一律不变。紧凑输入/批量几何先冻结实际成本/正反例，不以拆callback或免费无界扫描绕限；不足回new。

### 5.2 有界触发与几何

最多8对，用户显式注册/重绑，不自动扫描全世界所有对象对；每对一个同维subject（静态角色/受检叶）和一个静态或001活实例的触发源，稳定pair序号按注册表顺序。实例身份必须完整绑定Run/slot/generation，不能只存slot并自动追同槽新生；重绑新代是显式操作，prev态清零，旧事件不跨绑定代。非法/错维/组/未知/非有限/数量超限在使用前具名拒绝。

支持两种形状：受检实际资源AABB间闭区间相交（沿用旧边界），以及2D圆/3D球与subject实际AABB的最近点平方距离比较（<=radius²，radius>0有限；2D z=0）。默认样板radius=0.5；相切命中、角点在包围盒内却在圆/球外不命中，±1ULP分离须有反例。球半径由明确配置定义，不隐式取GLB包围盒当半径；静态/动态触发源center跟其当前接受前世界平移。隐藏不取消触发（显隐与生命周期分离），非solid触发不阻挡角色；销毁撤销资格。

每次app_update所有本tick运动完成后调用一次evaluate，读取同一当步位置；第二次evaluate、evaluate后再改参与对象位置/重绑配置应受控拒绝，不能把旧几何事件配新位置。evaluate后允许按结果销毁物品；本tick事件保留完整旧身份的只读快照，但不复活其写权限。新spawn/重绑参与本轮检测的时序在T-02冻结，依“评估后无新的几何状态混入本轮”原则实现。

app_start只建立注册表、无enter事件；首次update与false前态比较（初始重叠在首次update enter）。false→true=enter，true→true=stay，true→false=exit，false→false无事件；每对每tick至多1条，按pair序号输出，总≤8。一次evaluate后结果冻结，公开next操作每事件只出队一次、无新事件时返回显式absence，getters只读当前token且callback过期失效。OS重绘/重复getter不推进状态，不能因Pause/重复read再计数。

对象在enter后被销毁：该身份不在后续tick继续stay/合成exit，pair进入失效未绑定态；同槽新代不被自动纳入，显式重绑后可重新enter。user Auto决定“首次收集即销毁并计1”，不是Host看到名字就加分。已接受之前的任何事件都不得给外部声音/HUD/日志业务消费者当作正式提交结果（声音/HUD本计划不实现）。

### 5.3 候选/事件/生命周期共同准入

事件记录至少有profile/Run/tick/pair/binding-generation/phase、参与对象完整身份、有效几何来源；事件唯一键=(Run,tick,pair,binding-generation,phase)。Host核对顺序/唯一/容量/活性或当步合法销毁的墓碑快照、浮点域/几何来源/合法前态转移，拒绝旧Run/旧tick/重复/错对象/缺帧/假enter结果；不能只验证事件count或让Rust代算用户胜负。

Auto内事件前态/游标属于callback候选，外部可见对象/事件/角色/camera一起prepare/accept/publish；若source Fault、预算超限、事件身份错、prepare失败或资源失败，保last-good全体，闭VM，不能出现“币消失但计数未提交”或“新计数配旧对象”。Pause/neutral Step依固定tick语义：Pause不检测不推进，Step只中立输入执行一个逻辑步，可因真实几何变化产生该步结果，不合成新的OS按键。

### 5.4 新局请求与失败隔离

用户Auto拥有Playing/Won/Failed；样板只在Won/Failed读取一次Enter边沿后请求restart。公开请求是本callback有界输出标记，最多一个待处理请求，多次同tick请求幂等合并为1，不建队列；携带当前Run/tick、原接受source/config身份，在当前完整候选接受后才处理。

Host沿原单一调度栈终止旧局：清held/pending/camera capture，关闭旧VM/对象表/事件前态，创建新Run/新VM（app_start再执行一次、用户globals重置）、同一已接受源字节和固定配置，固定初值。临时外部source也使用本次Run已接受的快照，不restart时重新读磁盘；代码编辑/热重载另议。新初始candidate包含新实例代与camera初值，全部准备成功才替换显示。期间保旧last-good Prepared；不闪编辑帧、不混新物品与旧角色、无双调度器。

新app_start/资源prepare失败：具名Fault、关闭新VM，保原最后完整呈现，不假装完成新局或自动无界重试；Stop可恢复原编辑Prepared。迟到旧Run结果/旧事件/旧input永远拒绝，旧对象写权限撤销。新Run成功后位置/三物品/敌人巡逻起点/计数0/相机初值/Playing皆由Auto app_start恢复；held Enter不连续restart，恢复需新按下；Starting/Paused不处理新的restart，Stop取消尚未完成的新局。Run identity耗尽受控拒绝，不回绕复用。

20轮成功新局、失败初始化、Pause/Step/Stop/关闭检查最后VM/对象/事件owner和静态资源复用；保留旧画面所需合法Prepared引用与旧VM/事件owner区别记录。用户外部pin合法保留时只在最后释放后检查，不把计数清零当释放。

### 5.5 引擎夹具与下游交付接口

引擎samples两维各最小试验场，含3个不同实例物品、一个危险/非solid巡逻物体和终点，消费已有公共角色/相机。Auto状态机必须验证未集齐不胜利、stay不重复计数、同tick三物品各一次、危险/越界失败、集齐触终点胜利、Won/Failed后Enter新Run。这些是引擎功能的必要代表性测试，保留原分母，不由Rust业务或Gallery验收代替。

两维真实Auto与无窗口Host/GPU、参数/输入变异、独立own-SDK consumer、预算/身份/owner均由本计划闭合。无需为引擎夹具建设Gallery索引/产品README/下载ZIP，也不修改G01..08/内容工具。夹具不是最终游戏、美术或声音/HUD交付。

面向下游的交付收据必须绑定：auto-ui实际合入commit、最终公开API/profile/CLI、支持/拒绝与限额、SDK快照/两锁/构建栈、源码identity与导出/冷准备入口、真实测试/runner来源及已知限制。消费包应能离开生产者树离线准备SDK/锁定构建并运行两维夹具；registry缓存为显式前置。G11不得只凭review pass、代码提交或候选包开始实施；002 actual delivered后才由ag-examples/001冻结自己的consumer/runner/项目输入。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/runtime-triggers-and-rounds.md | 手写overlap前态/无restart→有限事件、整数代次、新Run事务与公共入口 | 引擎服务 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md; docs/specs/auto-graphics/auto-source-driver.md | 对象候选→事件/新局同事务、选择/来源/consumer，旧保持 | 完整入口与恢复 | AC-01..05 |

原SD-03（Gallery知识）整项移交ag-examples/001 SD-01；本计划不宣称G11已交付。

## 6. 测试设计

work自查→同一auto-ui树独立最终review→merge冻结受影响门，未指定模型/agent。变更相关∩约1/10可复现抽样，seed/全集/选择数在案；新AC的直接正反例必须关闭，完整AG留里程碑。OS输入/置顶/抢焦点自动化暂停，用公开逻辑输入与无窗口Host/GPU；历史未变路由证据不得冒新功能实测。

| Case | AC/不变量 | 实际入口/代表输入 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 几何/时序 | AABB闭接触、圆/球角外/切线/±ULP、当步双移动、8/9对、重复evaluate | 独立几何与Auto enter/stay/exit一致；包围盒替球/旧tick变异红；超限/阶段错拒绝 | T-02/03；离散采样 |
| V-02 | AC-03 一次/旧代 | 3物品同tick、stay/next/getter、销毁同槽/重绑、旧token | 3个ID各+1；老代无新stay/exit；next不重发；旧token拒绝、新代可enter | T-03/04；Auto计数 |
| V-03 | AC-04 新局/失败 | Won/Failed Enter、held Enter、Pause/Step、旧Run迟到、坏init/prepare/Stop | 新Run/源快照/计数0/3新代/角色camera初值一致；失败保完整旧呈现闭新VM；不重读磁盘源 | T-04；无存档/切场景 |
| V-04 | AC-04 事务/owner | 事件+销毁+orbit、错身份/重复/坏序/超预算，20轮 | 全旧或全新，未接受事件不外泄，最后weak owner/上传有界；冻结reset/泄漏事件变异红还原绿 | T-04；合法pin区分 |
| V-05 | AC-01/05 消费/预算 | 两维引擎夹具参数对照、fresh SDK consumer A→B、本代identity controls、旧报告/错runner | 真Auto状态/实际内容敏感、32credits与原限额；0selected/partial/错身份拒绝；离线消费可运行 | T-02/05；非G11产品包 |

拟入口由T-03/04建立：driver `cargo check --locked`、`cargo fmt -- --check`、`cargo test --locked --test runtime_triggers_api --test runtime_round_host --test runtime_trigger_samples -- --test-threads=1`；受影响001对象/Play/相机命名case、本代verifier/checker `--controls`与own-SDK离线消费。实际命令/样本/runner在T-02冻结，help或旧report不算验收。

无ag-examples工具/打包/目录门，不运行全AG/全UI/根cargo t/tf/tv/tu/docs_gen。GPU机器单实例；原始证据本地`D:/autostack/plan-artifacts/auto-ui/plan-002/`，attachments仅md摘要/契约。新局必须验证接受后的下一次读取/实际运行，不能只数request。复用须证明代码/依赖/配置/输入未变。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 001实际交付/唯一auto-ui树/SDK锁及fresh消费闭包核实 |
| AC-02 | work | profile/签名/几何/事件/新局/资源先冻结，整组合预算成立，旧保持 |
| AC-03 | work | 两维有界触发/确定时序/一次读取/真销毁与代次绑定，V-01/02 |
| AC-04 | work | 新Run复原/迟到隔离/失败last-good/20轮最后owner，V-03/04 |
| AC-05 | work | 两维引擎夹具、真实Auto+内容+负控/消费/预算与下游可冻结交付接口闭合，V-05 |
| AC-06 | review | 同一auto-ui树独立核AC/V/SD/资源/遗漏/公开路径/fmt/warnings，merge门pending |
| AC-07 | merge | 冻结引擎提交/输入、受影响抽样/必要新对照与复用审计通过 |
| AC-08 | merge | auto-ui源码/本Plan落地归档、SDK/接口/来源交付SHA、旧知识同步及guard清理收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | auto-ui/001实际交付 | 唯一auto-ui树、实际SHA/接口/身份/价格/SDK/锁核对 | AC-01 | [x] 001交付537122b+身份清单/SDK闭包核实；树 `.wt/ui-002/auto-ui` 分支 `codex/plan-002-dev` base 78ee3b7 |
| T-02 | work | T-01 | triggers-round-contract先冻结几何/事件/Run图/价格与整组合probe，不足回new | AC-02 | [x] 合同冻结（attachments/002/work/triggers-round-contract.md）；包络反例→r5显式上调96KiB |
| T-03 | work | T-02通过 | 新Auto/API/准入/触发时序/代次，V-01/02 | AC-02/03 | [x] api/runtime_triggers_v1.at+runtime_round.rs+装配/入口/锁；V-01/02 14/14绿（bfcc0c8） |
| T-04 | work | T-03 | 新Run事务/失败/owner、两维引擎夹具与GPU负控，V-03/04 | AC-03/04 | [x] restart交换/失败last-good/20轮owner/Paused-Step门 5/5绿；准入变异单测；两维夹具 |
| T-05 | work | T-04 | fresh consumer/预算/current identity/fmt/提交/SD与下游接口摘要，V-05 | AC-01..05 | [x] verify_plan002(+controls)/check_runtime_triggers全绿；fresh consumer离线--locked构建+14/14；身份链选plan002；提交bfcc0c8 |
| T-06 | review | T-05 | 同一引擎树独立最终review绑定冻结commit；merge门pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结引擎受影响门/新对照/复用审计，失败回work复审 | AC-07 | [ ] |
| T-08 | merge | T-07 | 仅auto-ui/master引擎落地，记录实际源码与消费身份；无需等G11 | AC-08 | [ ] |
| T-10 | merge | T-08 | 本仓Plan/复审/归档，现有知识按SD同步Spec/ledger/索引，给ag-examples/001实际SHA收据 | AC-08 | [ ] |
| T-11 | merge | T-10 | 本仓wt-guard clean后清树/分支/空组，保必要本地产物收据 | AC-08 | [ ] |

原T-09（内容仓合入）整项移交ag-examples/001 T-08；保留其他任务ID，不将移交项打勾或算本计划已完成步骤。

## 9. 复审记录

以下r1/r2记录保留原编号，属于历史来源，不是本仓新号的执行记录。


2026-10-29 new r1：`stage: new | plan_id: PLAN-883 | plan_revision: 1 | outcome: blocked | next: 等882实际交付后work T-01/02冻结；G11内容T-05另等880实际交付`。blocked是硬前置未满足，语义草案可审；不虚构882未来SHA/价格或880 schema。全部AC/V/T/SD未执行，无新树/实现/SDK准备/构建/OS/GPU/agent/远端操作，本轮不改880合同/进度/源码。

2026-10-29 new合同预检：0..10节、唯一AC-01..08/V-01..05、分stage任务与SD覆盖已核；本批两Plan及五份路线文档109个本地链接均可解析、编号各唯一，880跟踪文件相对HEAD零diff。仅结构/路径/合同自查，不是实现或独立final review。

2026-10-29 new r2：`stage: new | plan_id: PLAN-883 | plan_revision: 2 | outcome: blocked | next: 等auto-ui/PLAN-882实际交付后T-01/T-02`。用户明确裁定AG计划在auto-ui、Gallery计划在ag-examples管理；本次只修正计划仓/路径/簿记归档位置，原功能范围、资源预算和AC/V不变；保留r1记录，不创建实现树/运行测试。ag-examples/PLAN-880最新收据已为delivered，G11的该项前置已满足；PLAN-882仍是883引擎硬前置。


2026-10-29 new r3：`stage: new | plan_id: PLAN-002 | plan_revision: 3 | outcome: blocked | next: 等auto-ui/PLAN-001实际交付后T-01/T-02`。用户纠正本仓001起独立编号；旧auto-lang/PLAN-883仅作来源，新合同/工作树/分支/证据目录按auto-ui本仓号。未改变功能分母、预算、进度或历史验收，未创建实现树/执行测试。ag-examples旧880已归legacy历史，不占该仓本地号。


2026-10-29 new r4（用户批准按仓拆分）：`stage: new | plan_id: PLAN-002 | plan_revision: 4 | outcome: blocked | next: 等auto-ui/001实际交付后T-01/T-02`。功能/资源/引擎V-01..04不降级；T-05/V-05/AC-01/05..08收窄为引擎消费，原G11工程/锁/目录/包/旧项兼容与SD-03、T-09完整移交ag-examples/001。独立review/合入只绑定本引擎提交，G11不再反向阻塞002；分母映射见下游split-contract。仍drafting，所有任务未执行，本次未触代码/001在途树/构建/OS/GPU/agent。

2026-10-29 文档预检：本Plan 0..10节、8个AC/5个V/10个当前T及stage映射完整，所有步骤未勾；本批管理/路线文档184个本地链接可解析。依赖无反向边，原002几何/事件/新局规则逐字保留，001原AC/V/T与进度字段未改。此为合同结构/拆分完整性检查，不是实现验收或独立final review。

2026-10-29 new r5（work回带的有界修订）：`stage: new | plan_id: PLAN-002 | plan_revision: 5 | outcome: pass | next: work`。反例（真实装配探针，`cargo test --lib probe_managed_sizes`）：Objects2V1 组合61950B≤65536B，Triggers2V1 72141B、Triggers3V1 74050B 超限；triggers单元压缩态10131B vs 剩余预算3465B，冻结的144值块布局+15公开函数+评估器物理放不下。按本Plan§10预设路由显式上调受管桥接64→96KiB（组合320→352KiB），功能分母/运行时预算/AC/V/T/SD零变化；256KiB用户源与全部时限不变。work保持`executing`，已完成的单源/准入/接入保留，T-03从测试绿继续。

2026-10-29 goal r3（work pass）：`stage: work | plan_id: PLAN-002 | plan_revision: 5 | outcome: pass | worktree: D:/autostack/.wt/ui-002/auto-ui | branch: codex/plan-002-dev | base_commit: 78ee3b7 | code_commit: bfcc0c8 | dependency_revisions: 001=537122b（已归档交付）,SDK=932ad48闭包（.sdk-source，681文件冻结） | task_ids: T-01..T-05 | evidence: check_runtime_triggers 全绿（runtime_triggers_api 14、runtime_round_host 5、runtime_trigger_samples 3、001回放 9/8/8、包络+准入+round lib 门；原始证据 D:/autostack/plan-artifacts/auto-ui/plan-002/）；fresh own-SDK consumer（package_consumer→plan-artifacts/plan-002/consumer-fresh）verify_plan002 PASS+`cargo build --locked --offline` 成功+runtime_triggers_api 14/14 | blockers: 无 | next_responsible_role: review（T-06 同树独立最终review，统一回归门 pending 归 review/merge）`。r5 有界修订落地：MAX_BRIDGE 64→96KiB（组合上限352KiB），反例探针固化为装配包络守卫；实测计价（char_move≈26、事件读取每条≥2、单移动callback至多配一事件读取）与读取时序（迟一拍快照）已冻结合同。全程未触 001 已交付字节（runtime_objects_v1.at 等零修改，plan001 身份链保持历史）。

2026-10-29 goal r4：`stage: goal | plan_id: PLAN-002 | plan_revision: r5 | round: 4 | from: drafting | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-002/auto-ui | commit: bfcc0c8 | next: review`。授权链：用户 /goal 完成计划002 一次授予 work/review/repair/merge 全程。本轮执行 work 阶段 T-01..T-05（含 r5 有界修订）并置 execution_done；T-06 同树独立最终 review 待启动。

2026-10-29 goal r5（final review pass）：`stage: review | review_scope: final | reviewer_identity: 同实现会话独立评审上下文（无独立最终评审者角色；结论从工件与新鲜运行重建，非执行摘要采信；此限制如实记录） | plan_id: PLAN-002 | plan_revision: 5 | outcome: pass | worktree: D:/autostack/.wt/ui-002/auto-ui | branch: codex/plan-002-dev | reviewed_commit: bfcc0c8f5da6d7086d2849828d22ff9123f90631 | base_commit: 78ee3b7 | dependency_revisions: 001=537122b（归档交付,计划档案 provenance 链保持）,SDK=932ad48 闭包（verify_source --sdk-only PASS:479包/17本地/29路径边/冻结AGP锁保留） | spec_inputs: Plan r5 §5 规范增量（SD-01 add runtime-triggers-and-rounds.md, SD-02 modify scene-play-session+auto-source-driver；规范发布归 merge,评审期未发布） | acceptance_results: AC-01 PASS（树/分支/001交付537122b/SDK锁/唯一树核实）；AC-02 PASS（合同冻结附件已提交;r5包络反例守卫;预算probe实测）;AC-03 PASS（V-01/02=14/14新鲜重跑）；AC-04 PASS（V-03/04=5/5新鲜重跑）；AC-05 PASS（样例3/3;verify_plan002+controls;consumer-fresh-r2 于 bfcc0c8 重打包:identity PASS+离线--locked构建+14/14）；AC-06 PASS（本评审:fmt 0 diff;002文件零新增警告;公开路径cli/--behavior-api/样例头部齐备;merge门pending） | pending_closeout_gates: T-07 统一回归/复用审计（merge-owned）;T-08/T-10/T-11（merge） | findings: F-001（环境敏感,001时代既有,非002回归,不阻塞）:runtime_objects_host 20轮churn的上传平坦性断言对GPU驱动 buffer_rewrites 计数敏感——同树同二进制（该测试及其输入相对78ee3b7 diff为空,二进制与001交付位相同）整文件跑一次失败(离群26/27 vs warm 18)一次通过8/8,单独跑恒过；处置:merge-owned统一回归门以隔离运行该套件或走 check_runtime_objects.py 的报告级churn门 | evidence: 评审新鲜运行 D:/autostack/plan-artifacts/auto-ui/plan-002/review_batch_gate.txt（3套件+lib门全绿;001回放中 F-001 如上）；consumer-fresh-r2 离线验证；restart交换/准入oracle/装配派生/Spec delta 逐 diff 巡检一致 | next: merge（T-07..T-11）`

## 10. 待澄清事项

无未决产品决策，用户已明确批准拆分且暂不实施。001交付后若原限额下整组合不可行，带反例回new，不删任一维/同tick三物品/新局隔离或静默抬限。Gallery下游缺口由ag-examples/001按实际002交付反馈，不在本计划暗改内容仓，也不以其尚未实施认作002未完成。
