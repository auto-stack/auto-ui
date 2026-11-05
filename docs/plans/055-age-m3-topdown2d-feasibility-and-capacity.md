---
plan_id: PLAN-055
status: execution_done
feature_name: AGE M3俯视二维完整链与容量可行性研究
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-021, auto-ui/PLAN-022, auto-ui/PLAN-032]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/topdown2d-feasibility.md]
touched_goals: [GOAL-008]
affects: [autographics-research, topdown2d-capacity]
---

# PLAN-055：M3俯视二维最小链与容量研究

## 0. 变更摘要

在固定已交付AGE/旧SDK上，以最多两种等价机制验证正常Auto Node/use/model/on俯视移动→碰撞/交互→真实呈现的最小完整链，量化地图/对象/回调/资源瓶颈，给出go/bounded-go/no-go和下一份能力合同输入。是M3有界预研，**不交付完整种田游戏/新tilemap或新物理API**。见[首轮设计](../design/autographics/m2-closure-and-m3-first-wave.md)。本轮已完成有界研究，原额/生产API不变；实际结论和失败见§9。

## 1. 目标

- 核现有角色/通用对象与二维query服务能否表达无重力俯视移动、斜向/墙角/交互目标，业务由真实Auto处理。
- 固定一个最小地图/对象和有限规模对照，区分新语法接线缺口、旧VM/配额/资产/绘制瓶颈，报告单位明确的实际观测/拒绝。
- 提供可复查源码/输入/独立oracle、容量支持表、M3最低引擎缺口与成本/版本建议，供AG022冻结首个游戏切片；no-go同样可作为合格研究结论。

非目标：正式俯视引擎/图块批绘/寻路/存档系统、完整2DHD画面/光照、游戏业务、生产UI/Schema/ACE/AOT替换、静默抬原预算、two games/Gallery升级或M2验收。新功能实施另立合同，不能凭研究门绿认领已支持产品能力。

## 2. 架构方案

只读固定已交付的Node/query/sprite/公共服务与固定SDK681，在唯一研究worktree完成最小作者源/脚本调用/真实GPU消费；原生输入受政策限制时报告实际层，不把数据注入称OS键盘。候选仅选择：A现有角色配置是否能实现无重力俯视；B通用对象＋二维sweep/segment能否组成作者侧移动/交互。任何不存在接口记缺口，不自造函数名假通过。

拟唯一树`D:/autostack/.wt/ui-055/auto-ui`，`codex/plan-055-dev`；依赖auto-lang/auto-down按固定交付sibling只读。与054无功能依赖，可在其实施/AG021内容准备时做独立研究；actual GPU/焦点/重门错峰，target/scope不得共用。所有探针/候选补丁为计划材料，不把临时研究解释器/绕预算实现合入产品。研究必要的永久Spec/设计和复用到正式功能的测试资产由review决定；默认本Plan只落研究知识。

## 3. 技术栈

已读节点与执行链：pilot `src/{load,translate,lower_components,lower_components2d_runtime}.rs`，driver `src/authoring/`、固定API单元/渲染服务；Specs `node2d-queries.md`、`node2d-sprites-and-frame-animation.md`、`node2d-full-game-composition.md`。固定e02ca9f8可作为研究来源，T-01核其引用依赖与实际工具source身份；不跟054tip、不把049新UI ABI自动当旧game ABI。

## 4. 需求分析与背景调查

用户2026-11-05授权规划下一步；M3方向为俯视角2D/2DHD国风种田与复杂3D，M4才做正式AutoUI/ACE融合。既有M2平台游戏不是俯视游戏现成支持证明。021 query档有segment/sweep但明确不承诺新physics/自动slide；022 sprite档与query并非现成任意组合；032要求character/camera及固定部分根字段。必须实际核合适档与公共组合，不能改专用业务字段后声称通用俯视完成。

输入基线auto-ui e02ca9f8，SDK7b987ca8…/681；full-game Spec SHA前16=e693338b0a88a7e6。上下游Spec/Plan在T-01按内容哈希冻结。counter已核主/归档及注册树052/053，不占外仓OS055号；本Plan055/next056，只规划，未构建/实际测量。

## 5. 详细设计

T-01冻结研究支持表/原预算/候选版本、具体端口拼写/语义、编译器/VM/工具与输入、运行环境和测量方法；先列可证明结果与“不支持”的诊断。定义最小普通Auto模块样本（地面、障碍、一个控制对象和两交互物），8×8单层自制图元地图是候选起点；实际场景对象/状态Node/资源账按真实建模核，不按图块数自动给额外预算。2DHD只整理画面构成选项和必要依赖，不在本Plan渲染整套光照/景深。

T-02最多两种全链候选，先验证输入轴/坐标单位、水平/垂直/斜向与墙角、无重力需求、真实query结果→Auto状态→GPU像素/实例变换。语法层目前表达不了而底层能做的，明确列作者接线缺口；仅原始API链成立不能记正常Node作者链go。可做最小接缝探针验证假设，但候选是研究，不新增生产profile/语法或换SDK来把结论涂绿。

T-03以独立f32轨迹/碰撞oracle重放起停、斜向、碰墙/沿边、同时按相反方向、最近交互目标/错目标、Run切换；记录输入来自真实窗口/平台事件/受控fixture的层级。可行性所需最小渲染不靠截图预录，修改Auto规则后结果应真实改变。不可用接口或不合法组合是no-go证据，不删失败样本。

T-04按T-01冻结的有限规模做8×8、12×8、16×16候选地图/对象对照；这不是承诺三规模都接受，超过旧128行/状态8等约束时应受控拒绝并记录最先门。至多三规模×两机制，不做全仓压力或无限放大；同配置真实CPU Tick/编译/加载/首次呈现/稳态绘制、资源/target owner与执行费用分别记录。测量重复/环境/冷暖与退出码明确，旧game32credits、VM步数、时间和GPU资源不混算。字面量数组256/257等触面须具名校验，不利用旧SDK截断或分池/隔帧丢逻辑过关。

T-05对可接受最小链做坏输入/过期句柄/query token、资源缺失/坏件、旧Ready/源变化、Stop/newRun与恢复下一Run；必要原生键盘按既有OS政策手工或明确授权窗口，未实跑只记该层未证。容量与语义反例不是断言所有未来地图不可行。T-06形成：可行机制/支持与拒绝表、字段/单位与源版本、完整源/命令/实际oracle摘要、瓶颈和最小M3实现建议、2DHD选项待定项；给AG022精确可消费结果。

研究go仅代表明确版本/样本/规模/条件下机制成立，bounded-go列明缺口，no-go说明反例与下一步。后续真正增加地图批绘/更大容量/公共俯视控制端口须另立Plan/资源预算，不把新能力塞研究merge。原M2 API/资产/源码/锁不变，研究结果不回填AG021为验收前置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/topdown2d-feasibility.md | 俯视支持/容量未知→固定版本go/bounded-go/no-go及实际边界/缺口/后继输入；不是功能宣称 | 支持M3先证后立能力计划 | AC-01..06 |

## 6. 测试设计

以下是待执行研究case，不是已测证据；T-01先冻结工具调用与可复现probe命令，新增source/报告统一scope，不给不存在的生产CLI造承诺。实际调用现有pilot `node2d-lower --profile <已核档> --entry <scope>/... --out <scope>/...`及固定driver入口；每候选有明确命令/真实exit/输出。实现与自检同树，研究final为独立上下文在同树复验关键结论，同会话复跑不充独立。GPU/原生门单实例，未改根语言代码不跑根cargo全量。

| case | AC | owner_stage | 输入/实际入口与可区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | ≤2候选普通Node源、真实端口/固定SDK，输入→Auto→query/物件→GPU；改Auto速度结果改变，语法不能表达则具名缺口 | 无专用Rust玩法或假接口 |
| V-S02 | AC-02/03 | work | 四向/斜向/墙角/相反输入/最近与错交互目标，独立f32与位移/碰撞/像素对照；与零输入正控区分 | 不把平台跳跃默认重力当俯视成功 |
| V-S03 | AC-03/04 | work | 三规模以内合法与超界、数组/句柄/token边界，费用/steps/ms/对象/资源分单位，超原额受控拒绝 | 不关门限/多开池/丢帧省业务 |
| V-S04 | AC-04/05 | work | 坏源/资产/旧Ready/Run代次拒绝→精确修复→下次真实消费恢复，20轮小样本Run/Stop实际owner释放 | 非只删日志或看进程exit0 |
| V-S05 | AC-01/05 | work | 精确源码/SDK681/工具/图锁/输入/测量环境与旧profile对照，研究候选与生产支持明示分开 | 不改原M2锁/语义/全AG无关门 |
| V-S06 | AC-06 | work | 报告结论与原失败样本/边界/图形实测对应，AG022所需规模/后继接口缺口和源/命令可重建 | no-go可合格，缺证不能涂go |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 输入/支持/固定源/实际端口与≤2候选全链研究有真实结果，未实现状态如实 | V-S01/05 |
| AC-02 | work | 俯视移动/碰撞/交互的语义结论有真实Auto/GPU与独立oracle | V-S01/02 |
| AC-03 | work | 规模/费用/数值和原门限有合法/拒绝样本及单位，无静默放宽 | V-S02/03 |
| AC-04 | work | 资源/Run/错误恢复与容量风险实证，失败保留不充绿 | V-S03/04 |
| AC-05 | work | 研究来源/命令可复核，旧生产档保持、未证OS层准确记录 | V-S04/05 |
| AC-06 | work | go/bounded-go/no-go、SD候选与M3最小能力/AG022输入完整可审 | V-S06 |
| AC-07 | review | 独立上下文重建关键结论/反例与SD，给出研究pass或needs_fix，不以实现者自审代 | T-07 |
| AC-08 | merge | 实际落主研究知识/canonical/归档，消费者明确研究边界非产品能力交付 | T-08/09 |
| AC-09 | merge | 全输出对账/源码命令摘要折Plan/fresh guard清自有树和scope构建物、hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续另行启动；独立于054/AG021/020 | 冻结版本/端口/两候选/样本规模/预算单位/输入政策，唯一树/scope | AC-01/05 | [x] |
| T-02 | work | T-01 | 最多两真实Auto→查询移动→GPU完整最小先证，全部成功/失败/缺口原样记录 | AC-01/02 | [x] |
| T-03 | work | T-02已有机制结果（含no-go） | 独立轨迹/碰撞/交互反例；不能运行则定位不可达条件，禁止制造绿 | AC-02/03 | [x] |
| T-04 | work | T-03 | ≤3规模容量/费用/VM/加载绘制/资源分项观测，合法/超界与数组隐患 | AC-03/04 | [x] |
| T-05 | work | T-04 | 有效最小链故障/20轮Run/修复下一次消费、旧档身份边界；不可达支路明示 | AC-04/05 | [x] |
| T-06 | work | T-05 | 结论/SD/命令源摘要/容量支持表/后继建议和AG022消费输入 | AC-06 | [x] |
| T-07 | review | T-06 | 同树独立研究复审，finding返work同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 核实落地文件仅已审知识/必要资产，不把实验补丁当功能合入 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/当前导航，注明go边界/失败与未证项 | AC-08 | [ ] |
| T-10 | merge | T-09 | guard/全路径对账/摘要折Plan/清自有scope和构建物，hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: new / PLAN-055:r1 / outcome: pass（研究合同可审阅，后续启动可进T-01；no-go也是研究结果）；next: work T-01。drafting0/10。当前只规划，未测容量/时长、未构建/探针/GPU/新功能/材料scope/树/SDK/canonical；不改变M2验收或将本Plan设为AG021前置。

2026-11-05 / stage: new / planning_validation: pass（合同预检，非实施/独立final）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD、drafting/r1/0-10与未勾任务齐；AG021 r2保同一分母，现场原owner已记录executing2/10及T-03等待054，未回退其进度。依赖DAG无环，M3研究不是M2前置；独占号源后重新扫描主/归档和注册树核新号054/055/AG022唯一，next UI056/AG023。三仓49处新增/修改本地链接可解析，白名单空白检查通过；现有`python tools/gallery.py validate --all` exit0，12逻辑项/22入口/许可保持。048/049归档与canonical、019归档/候选canonical逐字节内容未改（SHA前16分别dfbd9df1087d52a7/b7f5c867ee107c2a、7bda6f53faf16cae/8967cc6c8cf87f0a、1d8e991170ec3969/601a457ab52edfec）。未构建/运行新源码或GPU/原生输入/容量研究、未写新canonical/资产/锁、未创建树/scope/agent/发布或迁目录；ag-021已有树与材料属原owner，保留其WIP，本会话不代其提交执行进度或工作收据。第一次预检因AG021并发进入executing不再符合原drafting断言而停，按新证据更新导航后复核通过，不把这次状态变化归为实现测试红。


2026-11-05 / stage: work / entry: T-01 started. 用户 `/goal 完成计划055` 授权 work→独立review→有界repair→merge/归档/清理全链。唯一树 D:/autostack/.wt/ui-055/auto-ui，branch codex/plan-055-dev，base 220115df62040f113bc689f0500af80fc11569a3；只读同组 auto-lang a0750086c57c809a586884ac8b0519e8c162979b / auto-down 7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c / auto-web 37d32c22d1a4e7744710bf8f02f6c74d5a96c897。main 无源码WIP，仅旧terminal_pixel四张PNG与并行计划文档，保留；OS055树/材料不属于本Plan。唯一材料scope D:/autostack/plan-artifacts/plan-055/auto-ui/，owner本goal，包含源码探针/两套独占target/日志，禁止删除auto-os sibling。prepare_sdk.py --from-snapshot exit0，681文件/17包哈希通过。实际编译/CPU/GPU尚待，未记go。

### T-01..06 研究结果与复建合同（2026-11-05）

- work-owned AC-01..06 / V-S01..06：pass，研究结论为 B 无地面最小链 bounded-go、完整铺地图元建模 no-go；A 同回调角色移动+最近查询 no-go。no-go 是合同允许的研究交付，不是产品支持宣称。T-07..10 仍归 review/merge，未关闭。
- 固定基线/依赖同入口记录；关键 lower_components.rs、translate.rs、driver api/、source-lock.json 对 e02ca9f8 的 `git diff --stat e02ca9f8 -- <paths>` 无输出。新UI/regions代码虽然已在主干，不用于本研究；旧game API/锁/source未改。落地提交 e85f730952714cab7e34b646611575f26efe672a 仅新增 SD-01 canonical 候选；最终知识为 docs/specs/auto-graphics/topdown2d-feasibility.md。
- 环境 Windows，i5-13600KF（14核20逻辑），系统枚举 RTX4060Ti（wgpu实际adapter ID未记录），rustc/cargo1.98.0；继承仓库 RUST_MIN_STACK=16777216（测试线程宿主栈，不改VM/credit预算）。pilot和driver各自独占target，无根cargo全量；`cargo build --locked --offline --bin node2d-lower` / driver `cargo build --locked --offline --lib` exit0，冷构建6m11s/3m52s。driver SDK prepare默认生成681文件的 ignored .sdk-source 于本树；全量哈希已验，测后已移到材料scope/sdk-source；它不落库、不作为产品修改，完整清理对账包含该路径。
- T-01 冻结仅两机制 A character2d/move_and_slide，B shape2d/sweep+segment；地图仅8×8、12×8、16×16，正控无地面；障碍(5,2)、控制对象(2,2)、近目标(2,6)、远目标(2,7)，相机跟随占位角色(-100,-100)，单位图元地面；不得将地面行排除或不可见后仍记完整地图go。速度world units/s、dt秒、归一化f32=0.70710677、t裕量0.001无量纲，世界半宽0.5；CPU transport/Host Action fixture，OS键盘/焦点/同时按键未证。总行/状态实例与credits、VM steps/slices、μs和GPU资源bytes分别计账。旧128行/状态8（含根）/spawn槽8/callback32/VM1000 steps per slice及100000累计与120 slices保持。
- T-02/03 A components-2d 的 input.axis(Down,Up) exit2 UndefinedVariable Down；改走已存在 queries-2d x/y后普通作者源正常降层，CPU运行35 credits/status10。生成源接缝对照去掉交互（仅证明原因，不充普通作者全链go）26 credits/status0，x=y=2.0666666。B普通作者源降层exit0；control/changed/map8/map12各68 ticks 独立signed f32 slab/Minkowski oracle核两轴/最近目标、hit控制、对角归一化、零轴抵消、负向与墙停；credits恰27。control final x3.999998、changed3.9998667；map8/map12恒(2,2)。额外77tick墙角/沿边oracle：斜向墙停(3.9999528,3.9999528)，改纯沿边后(3.9999528,4.799952)。本候选无自动slide，不以停两个轴冒认slide。27+第二sweep16+2getter=45的算式是本候选设计成本，不是所有算法不可能的证明。
- T-04 `run_lower.py` exit0为记录器退出码，逐产品实际退出码单列：A标准拼写exit2；A queries/无地图B/变速B/8×8/12×8均exit0；16×16 exit3 `scene exceeds 128 nodes (261 rows)`，无后续VM/GPU数据。对应行数5/69/101/261。最近一次降层JSON含每个真实命令、ms和worldSHA。状态边界同8×8：根+7状态图块=8接受、根+8=9具名拒绝tile7_0，均未改门限。数组255/256/257真实 `.len()`=255/0/1，断言冻结错误结果并记不安全，256/257不是受控拒绝/可用容量；不利用截断让地图过关。
- 四个CPU样本（各68 tick，独立Session，非性能SLA）：compile μs=195402/168600/148935/158697；max完整tick μs=1873/1763/13992/19629；maxsteps=4337/4337/35142/50182；maxslices=5/5/36/51，均credit27/status0。8×8/12×8完整CPU Tick超8ms参考线但VM门内，不伪称运行时已拒绝；Host pump可跨帧。没有把CPU回调、Host帧提交、GPUreadback和真实GPU执行时长混算。
- T-05 GPU 20轮同一真实Host fresh Run/Stop完整通过（gpu-reused-host.log，exit0，71.50s），每轮变换符合4*dt*0.70710677，橙色像素中心x304→308，暂停帧不推进；每轮Stop后Run消失、VM owner probe=false、spatial Weak升级None。target_count4/7584000 bytes、mesh_count7/136952 bytes、depth18057216 bytes、two_buffers196672 bytes、外部资产0，各轮无驻留计数增长；上传累计计数增长属写入量。速度8变体实际GPU transform加倍，8×8真实Host坐标仍2,2。仅显式owned计数，GPU timestamp/驱动内部总内存未测。
- T-04 GPU容量补测各一次，8×8/12×8完整load+compile+首呈现=1809434/1546319μs；同配置输入到呈现=32444/45301μs；各10暂停帧提交分别1266–1855/1595–2870μs；资源计数同上，保持不能遍历地面的反例。无地图交互两变体各10暂停帧提交1592–3905μs；不是活动业务帧率或GPU timestamp。
- T-05 最近目标→Auto model→整数flag显隐→实际GPU像素：近目标3显示far，错目标4不显示far，节点visible与GPU像素对照均通过，23.36s。初版直接show(bool)虽然降层接受，实际Run报runtime/_ag_error_argument return type mismatch fff3000080000000；失败源码/产物通过interaction_bool/wrong_target_bool保留。修作者写法为int0/1不改引擎，正确/错目标链绿，不称布尔ABI已修复。
- T-05 世界源改4.0→8.0保旧sha、缺文件均Storage具名拒绝；精确还原后同入口新Host Run移动，exit0/2.38s。B第二query后再读第一sweep token具名13；合法下一Run修复恢复，wrong space9轴不移动、space2恢复。静态row无spawned句柄，因此旧动态handle代次支路不可达；0外部资产且typed query+sprite组合未交付，坏PNG/音频支路未运行，不能由坏世界源冒认。二者为明确研究缺口，M3后继合同输入，不改原M2支持。
- 失败/工具纠正完整区分：初版紧写同一行的多个属性/语句导致作者parse拒绝；探针格式化误拆`var/let dy`、未用Tile导入造成多root、根负字面量初始化均已修作者输入，不算引擎容量失败。Rust harness首编有Windows路径转义/E0502，修后通过。直接调用exe未带仓库线程栈曾c00000fd；加回既有RUST_MIN_STACK后原样oracle通过，无VM预算变更。首次GPU每轮另建整个Host，完成0..12共13轮后初始化停滞；终止本scope PID38320（进程终止/文件锁释放稍有延迟），LNK1104期间改research target名v2后继续；该失败不作20轮成功。正式重用Host的完整20轮独立计数，无截样本、无隐藏初次失败。
- T-06 SD-01及AG022输入：选渲染/碰撞/交互层与地图表示，预算明确；保角色35与通用27及铺地反例；数组编码安全门；公共上下输入/斜向/墙角/同tick交互/sprite/相机组合另立能力合同。2DHD分层图元或atlas/离线背景/镜头路线只列选项与光照、排序、景深、资源依赖，不交付画面、不替AG022定美术。

#### 重建与验证入口

在同组固定源树、相同锁/SDK下，将下列脚本逐段恢复到绝对scope（均以各段文件名保存）。`prepare_probes.py` 被run_lower.py调用，继而运行 `interaction_inputs.py`、`state_inputs.py`；`prepare_harness.py`将既有node_queries_flow.rs确定版本的辅助函数与本次probe_tests.rs拼接（include改绝对路径、所有宿主临时project改scope），只临时注册显式test target，**完成后git restore driver Cargo.toml**。禁止合入这个临时manifest或probe。SDK准备使用官方工具 `python tools/prepare_sdk.py --from-snapshot`，681hash通过；工具默认输出临时在ignored tree，工作结束移scope/sdk-source，对其哈希保持不变。

```powershell
# cwd <tree>/experimental/autographics-node2d-pilot
$env:CARGO_TARGET_DIR='D:/autostack/plan-artifacts/plan-055/auto-ui/target-pilot'
cargo build --locked --offline --bin node2d-lower
# cwd driver（先prepare SDK）；第二套独占target
$env:CARGO_TARGET_DIR='D:/autostack/plan-artifacts/plan-055/auto-ui/target-driver'
cargo build --locked --offline --lib
python <scope>/run_lower.py
python <scope>/interaction_inputs.py
python <scope>/state_inputs.py
python <scope>/prepare_harness.py
cargo test --locked --offline --test plan055_research --no-run
# 同树本轮target重命名为plan055_research_v2，源逻辑不变；实际exe见下述hash表
$env:RUST_MIN_STACK='16777216'
# 逐项调用scope/target-driver/debug/deps/plan055_research_v2-*.exe
# --exact <test> --nocapture --test-threads=1；GPU项严格串行
# plan055_cpu_signed_diagonal_wall_and_floor
# plan055_character_move_interact_budget
# plan055_cpu_corner_slide_and_stale_token
# plan055_array_encoding_boundary
# plan055_gpu_changed_and_fresh_twenty
# plan055_interaction_gpu_and_wrong_target_control
# plan055_bad_source_identity_restored_run
# plan055_scale_gpu_observations
```

8个具名研究测试在相应最终输入上分别exit0；无全仓压力/全量语言门。gpu20的源/输入/SDK在后续只增加其他测试时未改，复用其有效证据而不重新跑20轮。

#### 冻结输入和本地材料哈希

| repo文件 | SHA256 |
|---|---|
| docs/specs/auto-graphics/node2d-queries.md | 0d930490f45d9a1297443eb867718976a8d940d0812bace278a8701e383cf20b |
| docs/specs/auto-graphics/node2d-full-game-composition.md | e693338b0a88a7e6e7313ba1dc7572f354d99b19cf786ef8a9a8cebc01fb76fb |
| docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md | f997bae66305eef07d5eb902dcc5ba195350c2ffff38360fcd456dda9877f1b2 |
| experimental/autographics-auto-driver/research/source-lock.json | f5d7f1f8e3e2bfedf70775ee4b9741080d712b809640ee334ae075a335a54959 |
| experimental/autographics-auto-driver/Cargo.lock | c0c4f725ca0245d4b39cc4d14299f76362da3e2553cca1e653fec1ca4c9827eb |
| experimental/autographics-node2d-pilot/Cargo.lock | d37d41852b9712dbe832274e67819825331b872e861801a7e9fa80063c9eea62 |
| docs/specs/auto-graphics/topdown2d-feasibility.md | 1812780466bb2c21d8cb23cc577df28e98837a56c6a15823ffa0703a6e2afb69 |

| scope文件（均在D:/autostack/plan-artifacts/plan-055/auto-ui/） | SHA256 |
|---|---|
| pilot-build.log | d9af91f23e6335b80d469b99e9ac71ba33800b31bd338de8ddd9977dd7bd13d3 |
| driver-build.log | d9c425e7f56f5b3ea3f05efa64b18aac17ad63702b612c30037c1eae560e65a9 |
| cpu.log | 361def8b49ed8ba4f6ddf49aff72c1b35bda040ed6976494834506aee5a3fb4a |
| character-runtime-v2.log | b59b1409a981ff514e73b6be57610af63e986d13849a29efb5e86db854af2cc9 |
| corner-stale-fixed-env.log | bd142d8d6ad482dc7c2f4e256b73a1d728fcb3531e4143cc8795293bc344f961 |
| arrays-final.log | 5d2af694c2a10c8543384e2568852e45383f73cc7f1bd1eecd1d5cf6c9eb8e08 |
| gpu.log | a7e452766241043642441946e04af1c28e12d01a964e00f8fe981b5e6c0a45d5 |
| gpu-reused-host.log | ec1aa9961e69bcab2bef955b81c5edb57062aefebf1a23f74416d0e60e44cb24 |
| interaction-boolean-failure.log | fb1673c84821e2964108a7df57bdbecbd305be6bd273515e80e674ee959027af |
| interaction-integer-gpu.log | e07996d6b781569a19e4a6542b866404b0804ccd4444214d421cf0e06b916c6b |
| identity-recovery.log | 5624ddabb3b8a4bf963d014e5ebaaab6c8e5b88c20eaa07681bc41763b122e0e |
| scale-gpu.log | 6b866a4f82c37f820c910b39549a41d82fa27a42d7d54e68005be7153bcec160 |
| states.log | 9ea5b267cad3baad8ae0b3aef316f3c0de25a2b0d4512db8646acd83a9c0533b |
| lower-results.json | 50b9885160b4d2575eb588863245dd18ea421f2acd8b7fb9b66d3c418c037376 |
| test-build-final.log | 266ca7669b6714ef1cfa580136d296398468a0d90372cb014c624f5aa6f6b624 |
| prepare_probes.py | 90e79c49ae85602ff8eb3cacb6bf4232c5c5dcb89445a31d0109db86a2a7ab16 |
| run_lower.py | 4488fab32dfb7827cc59b1ceeea207bd7360d44fc50fbd29b64d98c59a674939 |
| interaction_inputs.py | e7576f37f45d903ed418bd49dd07f38475a1e65cacc8ab81b6f0fa0556b4d97d |
| state_inputs.py | 5797c41dee086d8fb9c032b7efe6fefbf30c3db2700c7ddf9775cc82302297a2 |
| prepare_harness.py | 30d69965d681c9f179d496615149ad2ebaa0ce04f2cb3a79a8b76632591f9302 |
| probe_tests.rs | 623d78c9d34a98c61251db8a3e5ce3ec7951cbcaf9ab8c6ee25dbf51b598e006 |
| combined_tests.rs | cdad9b3e88ff4693a3b02cc3ce9aaca3ec68900dd8295359458b6f465492e3fd |
| target-driver/debug/deps/plan055_research_v2-139e7c7fdb31031c.exe | 184d862f08d3e081cb9624c5229589e64d2bffa017f58113cd500d42fa20ed03 |

#### 完整研究源（仅计划材料，非新增产品入口）

##### prepare_probes.py

```python
import pathlib, shutil, subprocess, json, hashlib, time
S=pathlib.Path(__file__).resolve().parent
R=pathlib.Path('D:/autostack/.wt/ui-055/auto-ui')
P=R/'experimental/autographics-node2d-pilot'
base=S/'inputs'; base.mkdir(exist_ok=True)
for f in (P/'fixtures/queries2d').glob('*.at'): shutil.copyfile(f,base/f.name)
def write(p,t):
 import re
 if p.suffix=='.at':
  t=re.sub(r' +(position|material|mesh|bounds|view_size|tx|ty):',r'\n\1:',t)
  t=re.sub(r' +var ',r'\nvar ',t)
  t=re.sub(r' +dy =',r'\ndy =',t)
  t=re.sub(r' +if ',r'\nif ',t)
  t=t.replace('var\ndy','var dy')
  t=t.replace('let\ndy','let dy')
 p.write_text(t,encoding='utf-8')
# Candidate A: normal character input service; no gravity is authored.
write(base/'character.at','''use resources: lab_char
node Main {
 view-2d {
  character2d body { position: (2.0, 2.0) material: resources.lab_char }
  camera2d cam { target: refs.body bounds: (0.0, 0.0, 8.0, 8.0) view_size: (8.0, 8.0) }
 }
 on { Life.Tick(dt) -> {
  let dx = input.axis(Left, Right) * dt
  let dy = input.axis(Down, Up) * dt
  let q = refs.body.move_and_slide((dx, dy))
 } }
}
''')
# Candidate B: general query mover, no gravity or dedicated Rust gameplay.
write(base/'runner.at','''use resources: lab_runner, lab_quad1
node Runner { view-2d { shape2d body {
 mesh: resources.lab_quad1 position: (2.0, 2.0) material: resources.lab_runner
} } }
''')
write(base/'wall.at','''use resources: lab_wall, lab_wall_mesh
node Wall { view-2d { shape2d body {
 mesh: resources.lab_wall_mesh position: (5.0, 2.0) material: resources.lab_wall
} } }
''')
write(base/'near.at','''use resources: lab_point, lab_quad1
node Near { view-2d { shape2d body {
 mesh: resources.lab_quad1 position: (2.0, 6.0) material: resources.lab_point
} } }
''')
write(base/'far.at', (base/'near.at').read_text().replace('Near','Far').replace('(2.0, 6.0)','(2.0, 7.0)'))
write(base/'tile.at','''use resources: lab_wall, lab_quad1
node Tile(tx: float, ty: float) { view-2d { shape2d body {
 mesh: resources.lab_quad1 position: (tx, ty) material: resources.lab_wall
} } }
''')
def source(w,h,ground=True,speed=4.0):
 tiles='\n'.join(f'Tile tile{x}_{y} {{ tx: {x}.0 ty: {y}.0 }}' for y in range(h) for x in range(w)) if ground else ''
 return '''use resources: lab_runner
use spectator: Spectator
use runner: Runner
use wall: Wall
use near: Near
use far: Far
use tile: Tile
node Main {
 model { var speed = SPEED var target = -1 }
 view-2d {
  Spectator spectator {} Runner runner {} Wall wall {} Near near {} Far far {}
  TILES
  camera2d cam { target: refs.spectator.actor bounds: (0.0, 0.0, 8.0, 8.0) view_size: (8.0, 8.0) }
 }
 on { Life.Tick(dt) -> {
  var dx = 0.0
  var dy = 0.0
  if input.space() == 2 { dx = input.x() * .speed * dt dy = input.y() * .speed * dt }
  if dx != 0.0 { if dy != 0.0 { dx = dx * 0.70710677 dy = dy * 0.70710677 } }
  let q = query2d.sweep(refs.runner, 1, dx, dy)
  let t = q.t()
  let hit = q.hit()
  var fraction = 1.0
  if hit == 1 { fraction = t - 0.001 if fraction < 0.0 { fraction = 0.0 } }
  refs.runner.shift(dx * fraction, dy * fraction)
  let px = refs.runner.world(0)
  let py = refs.runner.world(1)
  let a = query2d.segment(refs.runner, 1, px, py, px, py + 8.0)
  .target = a.node()
 } }
}
'''.replace('SPEED',str(speed)).replace('TILES',tiles).replace('var target = -1','var target = 0').replace('use tile: Tile\n','use tile: Tile\n' if ground else '')
for name,w,h,g,sp in [('control',8,8,False,4.0),('changed',8,8,False,8.0),('map8',8,8,True,4.0),('map12',12,8,True,4.0),('map16',16,16,True,4.0)]:
 write(base/(name+'.at'),source(w,h,g,sp))
print('prepared inputs',base)
write(base/'char_player.at','''use resources: lab_runner
node CharPlayer {
 model { var target = -1 }
 view-2d { character2d body { position: (2.0, 2.0) material: resources.lab_runner } }
 on { Life.Tick(dt) -> {
  let dx = input.x() * 4.0 * dt
  let dy = input.y() * 4.0 * dt
  let q = refs.body.move_and_slide((dx, dy))
  let a = query2d.segment(self.row(), 1, self.world(0), self.world(1), self.world(0), self.world(1) + 8.0)
  .target = a.node()
 } }
}
''')
write(base/'character_query.at','''use char_player: CharPlayer
use near: Near
use far: Far
node Main {
 view-2d {
  CharPlayer player {} Near near {} Far far {}
  camera2d cam { target: refs.player.actor bounds: (0.0, 0.0, 8.0, 8.0) view_size: (8.0, 8.0) }
 }
}
''')
```

##### run_lower.py

```python
exec((__import__('pathlib').Path(__file__).parent/'prepare_probes.py').read_text())
exe=S/'target-pilot/debug/node2d-lower.exe'
results=[]
for name,profile in [('character','components-2d'),('character_query','queries-2d'),('control','queries-2d'),('changed','queries-2d'),('map8','queries-2d'),('map12','queries-2d'),('map16','queries-2d')]:
 out=S/'products'/name
 cmd=[str(exe),'--profile',profile,'--entry',str(base/(name+'.at')),'--out',str(out)]
 t=time.perf_counter(); r=subprocess.run(cmd,capture_output=True,text=True); ms=(time.perf_counter()-t)*1000
 (S/(name+'-lower.log')).write_text(r.stdout+r.stderr)
 row=dict(name=name,profile=profile,exit=r.returncode,ms=ms,command=cmd,output=r.stdout+r.stderr)
 results.append(row); print(json.dumps(row))
write(S/'lower-results.json',json.dumps(results,indent=2))
```

##### interaction_inputs.py

```python
import pathlib,shutil,subprocess,json
s=pathlib.Path(__file__).parent
for name,target in [('interaction_bool',3),('wrong_target_bool',4),('interaction',3),('wrong_target',4)]:
 d=s/name;shutil.copytree(s/'inputs',d,dirs_exist_ok=True)
 far=(d/'far.at').read_text().replace('material: resources.lab_point','material: resources.lab_point\nvisible: false')
 (d/'far.at').write_text(far)
 show=f'refs.far.show(.target == {target})' if name.endswith('_bool') else f'var shown = 0\nif .target == {target} {{ shown = 1 }}\nrefs.far.show(shown)'
 src=(d/'control.at').read_text().replace('.target = a.node()',f'.target = a.node()\n{show}')
 (d/'main.at').write_text(src)
 cmd=[str(s/'target-pilot/debug/node2d-lower.exe'),'--profile','queries-2d','--entry',str(d/'main.at'),'--out',str(s/'products'/name)]
 r=subprocess.run(cmd,capture_output=True,text=True)
 print(name,r.returncode,r.stdout,r.stderr)
 assert r.returncode==0
```

##### state_inputs.py

```python
import pathlib,shutil,subprocess
s=pathlib.Path(__file__).parent
for n in [7,8]:
 d=s/f'states{n+1}';shutil.copytree(s/'inputs',d,dirs_exist_ok=True)
 (d/'state_tile.at').write_text((d/'tile.at').read_text().replace('node Tile','node StateTile').replace('view-2d {','model { var n = 0 }\nview-2d {'))
 t=(d/'map8.at').read_text().replace('use tile: Tile','use tile: Tile\nuse state_tile: StateTile')
 for x in range(n): t=t.replace(f'Tile tile{x}_0',f'StateTile tile{x}_0')
 (d/'main.at').write_text(t)
 cmd=[str(s/'target-pilot/debug/node2d-lower.exe'),'--profile','queries-2d','--entry',str(d/'main.at'),'--out',str(s/'products'/f'states{n+1}')]
 r=subprocess.run(cmd,capture_output=True,text=True);print(n+1,r.returncode,r.stdout,r.stderr)
```

##### prepare_harness.py

```python
from pathlib import Path
s=Path(__file__).parent
r=Path('D:/autostack/.wt/ui-055/auto-ui')
d=r/'experimental/autographics-auto-driver'
text=(d/'tests/node_queries_flow.rs').read_text()
text=text.replace('../../autographics-node2d-pilot/', (r/'experimental/autographics-node2d-pilot').as_posix()+'/')
text=text.replace('PathBuf::from(env!("CARGO_MANIFEST_DIR"))\n        .join("target/plan021-queries")','PathBuf::from(S055).join("host-projects")')
text=text.replace('combined.push_str("\\nfn probe_credit() int {return _ag_spatial_calls}\\n");','combined.push_str("\\nfn probe_credit() int {return _ag_spatial_calls}\\n");\n    if world_src.contains("var r_target") { combined.push_str("\\nfn probe_target055() int {return r_target}\\n"); }')
text+='\n'+(s/'probe_tests.rs').read_text()
(s/'combined_tests.rs').write_text(text)
m=d/'Cargo.toml'
t=m.read_text()
if 'plan055_research' not in t:
 m.write_text(t+'\n[[test]]\nname = "plan055_research"\npath = "D:/autostack/plan-artifacts/plan-055/auto-ui/combined_tests.rs"\n')
print('research-only test manifest registered; restore before delivery')
```

##### probe_tests.rs

```rust
const S055: &str = "D:/autostack/plan-artifacts/plan-055/auto-ui";
fn product055(name: &str) -> (String,String) {
    let p=PathBuf::from(S055).join("products").join(name);
    (std::fs::read_to_string(p.join("scene.agscene")).unwrap(),std::fs::read_to_string(p.join("world.at")).unwrap())
}
// Independent signed-direction slab oracle. All quantities f32, inclusive contact.
fn slab055(o:[f32;2], d:[f32;2], boxes:&[(usize,[f32;2],[f32;2])], half:f32)->(f32,i32) {
    let mut best=1.0f32; let mut node=-1;
    for &(n,lo,hi) in boxes { if n==1 {continue} let mut a=0.0f32; let mut b=1.0f32;
        for k in 0..2 { let l=lo[k]-half; let h=hi[k]+half;
            if d[k]==0.0 {if o[k]<l || o[k]>h {a=2.0;break;}}
            else {let x=(l-o[k])/d[k]; let y=(h-o[k])/d[k]; a=a.max(x.min(y)); b=b.min(x.max(y));}
        }
        if a<=b && a<=1.0 && (node<0 || a<best) {best=a;node=n as i32;}
    } (best,node)
}
#[test]
fn plan055_cpu_signed_diagonal_wall_and_floor() {
 for name in ["control","changed","map8","map12"] {
    let (scene,world)=product055(name); let boxes=seg_boxes(&scene);
    let mut s=probe_session(&scene,&world,name,1);
    let speed=if name=="changed" {8.0} else {4.0};
    let mut pos=[2.0f32,2.0]; let mut max_us=0u128; let start=std::time::Instant::now();
    // Start/stop, all directions, opposite-axis cancellation (0), diagonal, wall/corner.
    for (i,(x,y)) in [(0,0),(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,-1),(0,0)].into_iter().chain(std::iter::repeat((1,0)).take(60)).enumerate() {
      let mut d=[x as f32*speed*DT,y as f32*speed*DT]; if x!=0 && y!=0 {d[0]*=0.70710677;d[1]*=0.70710677;}
      let (t,n)=slab055(pos,d,&boxes,0.5); let f=if n<0 {1.0} else {(t-0.001).max(0.0)};
      pos[0]+=d[0]*f;pos[1]+=d[1]*f;
      set_action(&mut s,2,x,y); let now=std::time::Instant::now(); tick_cpu(&mut s,i as i32+1); max_us=max_us.max(now.elapsed().as_micros());
      assert_eq!(status(&mut s),0,"{name} tick {i}");
      let actual=[fcall(&mut s,"ag_world",&[Int(1),Int(0)]),fcall(&mut s,"ag_world",&[Int(1),Int(1)])];
      assert!((actual[0]-pos[0]).abs()<1e-3 && (actual[1]-pos[1]).abs()<1e-3,"{name} {i}: {actual:?} expected {pos:?}");
      assert_eq!(credit(&mut s),27);
      let (_,target)=slab055(pos,[0.0,8.0],&boxes,0.0);
      assert_eq!(icall(&mut s,"probe_target055",&[]),target,"nearest target, not far/wrong row");
    }
    if name.starts_with("map") {assert_eq!(pos,[2.0,2.0],"visible ground overlaps sweep at t=0");}
    else {assert!(pos[0]>3.9 && pos[0]<4.01,"wall stops at x4: {pos:?}");}
    let c=credit(&mut s);
    println!("PLAN055 cpu name={name} ticks=68 final={pos:?} compile_us={} max_tick_us={max_us} elapsed_us={} credits={c} max_steps={} max_slices={}",s.compile_us,start.elapsed().as_micros(),s.max_call_steps,s.max_call_slices);
 }
}
#[test]
fn plan055_gpu_changed_and_fresh_twenty() {
 let (scene,world)=product055("control");
 let (changed_scene,changed_world)=product055("changed");
 let cold=std::time::Instant::now();let mut h=start_with(&scene,&world,"055-reused-host");let cold_us=cold.elapsed().as_micros();
 for round in 0..20 {
   let t=std::time::Instant::now();
   if round>0 {h.start_selected_play().unwrap();h.pause_play("");settle(&mut h);}
   let startup_us=if round==0 {cold_us} else {t.elapsed().as_micros()};
   let before=pixel_classes(&mut h); let x0=center(&class_hits(&before,"orange"),0);
   step_action(&mut h,2,1,1); assert!(play_error(&h).is_empty(),"{}",play_error(&h));
   let p=actor(&h,2); let want=2.0+4.0*DT*0.70710677;
   assert!((p[0]-want).abs()<1e-3 && (p[1]-want).abs()<1e-3,"{p:?}");
   let render=std::time::Instant::now(); let after=pixel_classes(&mut h); let render_us=render.elapsed().as_micros();
   assert_ne!(before,after,"actual GPU pixels respond");
   let x1=center(&class_hits(&after,"orange"),0); assert!(x1>x0,"screen moves right {x0}->{x1}");
   for _ in 0..3 {h.frame(None).unwrap();} assert!((actor(&h,2)[0]-want).abs()<1e-3);
   let owner=h.play.as_ref().unwrap().owner_probe().unwrap();
   let spatial=h.play.as_ref().unwrap().spatial_owner_probe().unwrap();
   println!("PLAN055 resources round={round} {}",h.resource_snapshot());
   h.stop_play(); assert!(h.play.is_none(),"Stop removes Run owner");
   assert!(!owner(),"VM owner released");assert!(spatial.upgrade().is_none(),"query descriptor released");
   println!("PLAN055 gpu round={round} startup_us={startup_us} render_probe_us={render_us} x={x0}->{x1} owner_stopped=true");
 }
 let mut h=start_with(&changed_scene,&changed_world,"055-changed"); step_action(&mut h,2,1,1);
 let p=actor(&h,2);assert!((p[0]-(2.0+8.0*DT*0.70710677)).abs()<1e-3);h.stop_play();
 let (map_scene,map_world)=product055("map8");let mut h=start_with(&map_scene,&map_world,"055-floor");step_action(&mut h,2,1,1);
 assert_eq!(actor(&h,2)[0],2.0);assert_eq!(actor(&h,2)[1],2.0);h.stop_play();
 println!("PLAN055 changed Auto speed doubles actual transform; visible floor blocks true Host Run");
}
#[test]
fn plan055_bad_source_identity_restored_run() {
 let (scene,world)=product055("control");
 let dir=write_project(&scene,&world,"055-damage");
 let v:serde_json::Value=serde_json::from_str(&scene).unwrap();let sha=v["behavior"]["source"]["sha256"].as_str().unwrap();
 let file=dir.join("scripts").join(format!("{sha}.at"));
 std::fs::write(&file,world.replace("4.0","8.0")).unwrap();
 let bad=Storage::load_project(&dir.join("scene.agscene"),&Registry::new());assert!(bad.is_err());
 println!("PLAN055 old source identity rejected {:?}",bad.err());
 std::fs::remove_file(&file).unwrap();assert!(Storage::load_project(&dir.join("scene.agscene"),&Registry::new()).is_err());
 std::fs::write(&file,&world).unwrap();let mut h=start_with(&scene,&world,"055-damage");
 step_action(&mut h,2,1,0);assert!(play_error(&h).is_empty());assert!(actor(&h,2)[0]>2.06);h.stop_play();
 println!("PLAN055 exact restore -> fresh true Host Run moved");
}
#[test]
fn plan055_array_encoding_boundary() {
 for n in [255usize,256,257] {
   let values=(0..n).map(|i|i.to_string()).collect::<Vec<_>>().join(",");
   let src=format!("var a=[{values}]\nfn main() {{}}\nfn size() int {{return a.len()}}\n");
   let mut s=Session::new(&src).unwrap();s.call("main",&[],ResultKind::Unit).unwrap();let actual=s.int("size",&[]).unwrap();
   assert_eq!(actual,(n%256) as i32,"frozen SDK encoding hazard, not valid map capacity");
   println!("PLAN055 array literal n={n} actual_length={actual}; n>=256 unsafe, no capacity credit");
 }
}
#[test]
fn plan055_character_move_interact_budget() {
 let (scene,world)=product055("character_query");
 let mut s=probe_session(&scene,&world,"055-char",1);set_action(&mut s,2,1,1);tick_cpu(&mut s,1);
 let code=status(&mut s);let c=credit(&mut s);println!("PLAN055 character+segment status={code} credits={c}");assert_eq!(code,10,"26 move +8 cast +1 getter exceeds32");
 // Legitimate same-author control removes interaction, without adding gravity.
 let world=world.replace(" let a = ag_cast_segment(2, charplayer_qrow[i], 1, ag_world(charplayer_qrow[i], 0), ag_world(charplayer_qrow[i], 1), 0.0, ag_world(charplayer_qrow[i], 0), (ag_world(charplayer_qrow[i], 1) + 8.0), 0.0)\n charplayer_target[i] = ag_cast_node(a)","");
 let mut s=probe_session(&scene,&world,"055-char-move-only",2);set_action(&mut s,2,1,1);tick_cpu(&mut s,1);
 assert_eq!(status(&mut s),0);let x=fcall(&mut s,"ag_world",&[Int(0),Int(0)]);let y=fcall(&mut s,"ag_world",&[Int(0),Int(1)]);
 println!("PLAN055 character no-gravity movement-only x={x} y={y} credits={}",credit(&mut s));
 assert!((x-(2.0+4.0*DT)).abs()<1e-3 && (y-(2.0+4.0*DT)).abs()<1e-3);
}
#[test]
fn plan055_cpu_corner_slide_and_stale_token() {
 let (scene,world)=product055("control");let boxes=seg_boxes(&scene);
 let mut s=probe_session(&scene,&world,"055-corner",1);let mut pos=[2.0f32,2.0];
 for (i,(x,y)) in std::iter::repeat((1,1)).take(65).chain(std::iter::repeat((0,1)).take(12)).enumerate() {
  let mut d=[x as f32*4.0*DT,y as f32*4.0*DT];if x!=0 && y!=0 {d[0]*=0.70710677;d[1]*=0.70710677;}
  let (t,n)=slab055(pos,d,&boxes,0.5);let f=if n<0 {1.0} else {(t-0.001).max(0.0)};
  pos[0]+=d[0]*f;pos[1]+=d[1]*f;set_action(&mut s,2,x,y);tick_cpu(&mut s,i as i32+1);assert_eq!(status(&mut s),0);
  for k in 0..2 {assert!((fcall(&mut s,"ag_world",&[Int(1),Int(k as i32)])-pos[k]).abs()<1e-3);}
  if i==64 {assert!((pos[0]-4.0).abs()<1e-3 && (pos[1]-4.0).abs()<1e-3);println!("PLAN055 diagonal into wall stops both axes {pos:?}; no automatic slide");}
 }
 assert!(pos[1]>4.7);println!("PLAN055 deliberate along-edge input works {pos:?}");
 let changed=world.replace(" r_target = ag_cast_node(a)"," let stale = ag_sweep_t(q)\n r_target = ag_cast_node(a)");assert_ne!(world,changed);
 let mut s=probe_session(&scene,&changed,"055-stale",2);set_action(&mut s,2,0,0);tick_cpu(&mut s,1);assert_eq!(status(&mut s),13);
 let mut fresh=probe_session(&scene,&world,"055-next-run",3);set_action(&mut fresh,9,1,1);tick_cpu(&mut fresh,1);assert_eq!(status(&mut fresh),0);assert_eq!(fcall(&mut fresh,"ag_world",&[Int(1),Int(0)]),2.0);
 set_action(&mut fresh,2,1,0);tick_cpu(&mut fresh,2);assert_eq!(status(&mut fresh),0);assert!(fcall(&mut fresh,"ag_world",&[Int(1),Int(0)])>2.06);
 println!("PLAN055 stale13 -> newRun repaired; wrong-space9 neutral, validspace2 moves");
}
#[test]
fn plan055_interaction_gpu_and_wrong_target_control() {
 let mut pixels=Vec::new();
 for name in ["interaction","wrong_target"] {
  let (scene,world)=product055(name);let mut h=start_with(&scene,&world,name);
  assert!(!h.play.as_ref().unwrap().last.as_ref().unwrap().nodes.get(&5).unwrap().visible);
  step_action(&mut h,2,0,0);assert!(play_error(&h).is_empty(),"{name}: {}",play_error(&h));
  let visible=h.play.as_ref().unwrap().last.as_ref().unwrap().nodes.get(&5).unwrap().visible;
  assert_eq!(visible,name=="interaction","nearest result -> Auto state -> target presentation");
  pixels.push(pixel_classes(&mut h));
  let mut frame_us=Vec::new();for _ in 0..10 {let t=std::time::Instant::now();h.frame(None).unwrap();frame_us.push(t.elapsed().as_micros());}
  println!("PLAN055 interaction name={name} far_visible={visible} paused_render_submit_us={frame_us:?}");h.stop_play();
 }
 assert_ne!(pixels[0],pixels[1],"wrong target changes actual GPU pixels");
}
#[test]
fn plan055_scale_gpu_observations() {
 for name in ["map8","map12"] {
  let (scene,world)=product055(name);let t=std::time::Instant::now();let mut h=start_with(&scene,&world,name);let first_us=t.elapsed().as_micros();
  let t=std::time::Instant::now();step_action(&mut h,2,1,1);let tick_to_present_us=t.elapsed().as_micros();
  let error=play_error(&h);assert!(error.is_empty(),"{name}: {error}");assert_eq!(actor(&h,2)[0],2.0);
  let mut render_us=Vec::new();for _ in 0..10 {let t=std::time::Instant::now();h.frame(None).unwrap();render_us.push(t.elapsed().as_micros());}
  println!("PLAN055 scale {name} first_present_including_load_compile_us={first_us} tick_to_present_us={tick_to_present_us} steady_paused_submit_us={render_us:?} resource={}",h.resource_snapshot());h.stop_play();
 }
}
```

stage: work | plan_id: PLAN-055 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-055/auto-ui | branch: codex/plan-055-dev | base_commit: 220115df62040f113bc689f0500af80fc11569a3 | code_commit: e85f730952714cab7e34b646611575f26efe672a | dependency_revisions: auto-lang a0750086c / SDK 7b987ca8 (681hash) / auto-down7c4d5cb / auto-web37d32c2 | task_ids: T-01..06 | evidence: above commands/hashes/oracle and all preserved failures | blockers: none; 0 required work AC remaining | next_responsible_role: independent final review T-07; docs-only content/link/format gate pending merge.

stage: goal | plan_id: PLAN-055 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-055/auto-ui | commit: e85f730952714cab7e34b646611575f26efe672a | next: review

### 独立 final review R1（2026-11-05）

stage: review | review_scope: final | reviewer_identity: 独立子上下文 /root/review055（与实施上下文分离；不虚构模型或会话ID） | plan_id: PLAN-055 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-055/auto-ui | branch: codex/plan-055-dev | reviewed_commit: e85f730952714cab7e34b646611575f26efe672a | base_commit: 220115df62040f113bc689f0500af80fc11569a3 | dependency_revisions: auto-lang a0750086c57c809a586884ac8b0519e8c162979b / auto-down 7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c / auto-web 37d32c22d1a4e7744710bf8f02f6c74d5a96c897 / runtime SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467、scope/sdk-source 全681文件与source-lock逐一hash相等。

- 当前主Plan:r1为合同；审前/审后树clean，HEAD与分支不漂移。唯一差异为新增137行SD，无生产源码/锁/预算或ledger变更。review skill SHA256 `3d9263331895dd09cff5015783d9fc21b8be8441505d4101d52f8197d6d2a1f7`。spec_inputs沿上表三份现行Spec/source-lock/Cargo.lock；冻结SD-01 SHA256 `1812780466bb2c21d8cb23cc577df28e98837a56c6a15823ffa0703a6e2afb69`。spec impact paths/new_spec_components/touched_goals与SD一致，未改规范。
- 复建审查：Plan所折6份完整源码与scope文件逐字一致；combined/probe/exe和gpu20/lower-results的SHA均吻合上表。生成器→真实world/source SHA→旧SDK→Host→像素链可追踪；oracle为宿主signed f32 slab/Minkowski，非调用Auto实现取期望值；正负方向、重叠、最近/错目标等有可区分断言。无地面合法链不冒认完整地图；sprite/外部资产/动态handle/OS键盘/自动slide未证且准确排除；13Host停滞/stack环境错误/show(bool)ABI/数组截断原失败均保留。

| acceptance_results / AC→任务 | result | 独立方法与实际结果（绑定上述commit/SDK/源） |
|---|---|---|
| AC-01→T01/02 | pass | 源码/端口/≤2机制审查；独立lower character exit2 UndefinedVariable Down；B普通作者链真实执行 |
| AC-02→T02/03 | pass | `plan055_cpu_signed_diagonal_wall_and_floor` 4×68tick、两轴/最近目标/credit27通过；control终点[3.999998,1.9999999]，changed[3.9998667,2]，map8/map12恒[2,2]；`plan055_cpu_corner_slide_and_stale_token` 墙角[3.9999528,3.9999528]、纯沿边[3.9999528,4.799952]通过；GPU交互正确visible=true、错目标false且真实像素不同 |
| AC-03→T03/04 | pass | 独立`plan055_character_move_interact_budget` status10/credit35、移除交互正控status0/credit26/x=y2.0666666；`plan055_array_encoding_boundary` 255/256/257→255/0/1；lower states8接受/9拒绝tile7_0、map16 exit3/261rows；全为原额 |
| AC-04→T04/05 | pass | 独立token13→freshRun恢复、wrong-space9中性/space2移动；审坏源恢复和20轮freshRun/Stop断言、0..19原子输出及owned resource计数；复用源/输入/SDK未变的71.50s证据，不重跑GPU20，不把13Host拼绿 |
| AC-05→T01/05 | pass | 固定依赖与681hash独立核准；所有复验均显式RUST_MIN_STACK=16777216与--exact/--nocapture/--test-threads=1；所有新输出仅自有scope；生产档保持、OS层未证准确 |
| AC-06→T06 | partial | 边界/后继M3/AG022输入完整，唯一F-055-R1-01数字与冻结日志不一致 |
| AC-07→T07 | partial | 独立重建完成；SD需修复后才能final pass |
| AC-08/09→T08..10 | pending | merge-owned，不认领归档/导航/清理已完成 |

独立运行命令：cwd上述唯一树；`$env:RUST_MIN_STACK='16777216'; <scope>/target-driver/debug/deps/plan055_research_v2-139e7c7fdb31031c.exe --exact <test_name> --nocapture --test-threads=1`。上述4个CPU研究测试分别exit0（2.68s/1.48s/0.94s/0.15s）；单实例GPU `plan055_interaction_gpu_and_wrong_target_control` exit0/11.96s。复验CPU最大tick为1895/1808/14177/20693μs、steps4337/4337/35142/50182、slices5/5/36/51，仍为样本非SLA。GPU暂停提交样本另见1055–23835μs，不替换原观测或包装为活动帧率。lower四项使用相同输入、独占`<scope>/review-products/<name>`输出，退出码2/3/0/3及诊断已核。

| finding | severity / affected | 违反既有规则、实际/期望、原因/边界与最小关闭范围 |
|---|---|---|
| F-055-R1-01 | P2 / AC-06, T-06, dependent T-07 | SD-01测量口径称“降层端到端命令约7–26ms”，但其绑定`lower-results.json`（SHA256 50b9885160b4d2575eb588863245dd18ea421f2acd8b7fb9b66d3c418c037376）与lower-final.log逐case为23.3139/38.5543/51.2631/66.7361/52.4086/60.2815/69.7427ms；七条总范围23.3139–69.7427ms、五条接受38.5543–66.7361ms。已确认是文档数字与冻结输入不一致，不是运行回归；未找到7–26的可绑定来源。按AC-06真实测量对应要求，在work同树修正SD到所选明确样本范围，并将这组实际数字/退出码摘要折Plan（scope后续会删），提交更新SD hash。关闭oracle仅核修复diff、JSON七条数值/成功与拒绝分组和新SD hash；无须重跑CPU/GPU/20轮/构建。其他结论及AC证据保留。 |

新复验日志均在`D:/autostack/plan-artifacts/plan-055/auto-ui/`：
- `review-interaction-gpu.log` SHA256 `a7652df33e8f5c9a958c434ae379b7584c021f1ad87405f0f61d326f2395acc0`。
- `review-lower-character.log` SHA256 `5b8974a71dda0371cb763dc2e4c0fb6ecc0c0f626eafb3ef4bff7a6fd6e20595`。
- `review-lower-map16.log` SHA256 `6a9223b0a8df5719eb325fd5adcecddd24ecbd8c47a28a93a8a3f3bc1614ad9a`。
- `review-lower-states8.log` SHA256 `1fd4370d34817abfaf7520e248552d810f3af388fe3e3edaec3e138948c4fe2d`。
- `review-lower-states9.log` SHA256 `911217ed3e98a6246a73569b08a516064091e5ed3a9cac82d5821ff06cdde725`。
- `review-plan055_array_encoding_boundary.log` SHA256 `04631d3b1f346cb87894163430ec4a37bf71eb1bc22afc49d38b7bec0be9bc15`。
- `review-plan055_character_move_interact_budget.log` SHA256 `e59634d92f147bbd49390de556802ca5c6a0191029eb3f55c7f1e844d52587a7`。
- `review-plan055_cpu_corner_slide_and_stale_token.log` SHA256 `ed7a34986294db0bb06704015ba13606f2b9e315a5e1b6972e6935b002f8bfea`。
- `review-plan055_cpu_signed_diagonal_wall_and_floor.log` SHA256 `f5da706ea0caa63ed32f0f41e9e0ff2b9b72e6b9d32782aeda5e1e8b4316daa2`。

pending_closeout_gates: docs-only content/link/format；Spec指向未来同号archive链接须merge实际迁移后关闭；全scope输出对账/guard/清理T08..10归merge。next: work仅修F-055-R1-01→提交→同独立reviewer仅复核受影响SD/Plan数字及hash。T06/T07重开，current_step=5/10；不改plan_revision（无语义合同修改）。

stage: goal | plan_id: PLAN-055 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-055/auto-ui | commit: e85f730952714cab7e34b646611575f26efe672a | next: work（F-055-R1-01，仅SD数字）

### F-055-R1-01 有界文档修复（2026-11-05）

冻结 `lower-results.json` 的7条原始样本持久摘要如下；记录器本身exit0不等于各产品均接受。

| 样本 | profile | 逐命令exit | 端到端ms | 实际结果 |
|---|---|---:|---:|---|
| character | components-2d | 2 | 23.3139 | UndefinedVariable Down |
| character_query | queries-2d | 0 | 38.5543 | 角色移动+交互源生成成功，VM后续35credits拒绝另计 |
| control | queries-2d | 0 | 51.2631 | 无地面B生成成功 |
| changed | queries-2d | 0 | 66.7361 | speed8变体生成成功 |
| map8 | queries-2d | 0 | 52.4086 | 69行接受，不能遍历地面 |
| map12 | queries-2d | 0 | 60.2815 | 101行接受，不能遍历地面 |
| map16 | queries-2d | 3 | 69.7427 | 261行超过128拒绝 |

修复不变量：报告必须对应其冻结样本及退出码分组。仅修canonical的范围为七条23.3139–69.7427ms、五条接受38.5543–66.7361ms、两条拒绝23.3139/69.7427ms；其余源/配置/SDK/测试输入未变。`git diff --check` exit0；逐项与JSON核对，无运行门重跑、无阈值/合同变更。SD新SHA256 `c185f8f2df252951bb57332c6f8803aa687c02a6acf51e6faa967359d0cbaac8`，旧hash181278…仅保留历史审计。F-055-R1-01→完整采样对应不变量→SD量测段/Plan持久摘要→仅4增2删文档修复→同独立reviewer定向复核pending，其他AC01..05已通过证据复用，0其他work缺口。

stage: work | plan_id: PLAN-055 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-055/auto-ui | branch: codex/plan-055-dev | base_commit: e85f730952714cab7e34b646611575f26efe672a | code_commit: 6277e50f88a09be3850403356f2e76809f3081d4 | task_ids: T-06 / F-055-R1-01 | evidence: frozen JSON + seven-case table + doc diff/check; runtime untouched | blockers: none | next_responsible_role: same independent final reviewer, affected SD/hash only; merge docs gate remains pending

stage: goal | plan_id: PLAN-055 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-055/auto-ui | commit: 6277e50f88a09be3850403356f2e76809f3081d4 | next: review

## 10. 待澄清事项

D-01：实际无重力/角色配置和query＋sprite组合以T-01/02代码/真链裁决，不假设接口存在。D-02：2DHD美术/摄像机路线未定，提供选项与依赖不擅自选整套效果；最终内容选择归AG022合同。D-03：容量数字/时间仅真实测量，未证/超界/no-go完整保留。scope唯一`D:/autostack/plan-artifacts/plan-055/auto-ui/`；先核owner避免OS055跨仓材料冲突，登记用途/在途；源码探针/SDK副本/target/外部消费/日志均scope。merge必要源码/命令/失败摘要折Plan并核全部实际输出，guard清自有树/删scope构建物；具名hold可留owner/理由/关闭条件，禁止链接/不触别仓同号。
