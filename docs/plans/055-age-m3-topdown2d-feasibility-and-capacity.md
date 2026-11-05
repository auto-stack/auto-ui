---
plan_id: PLAN-055
status: drafting
feature_name: AGE M3俯视二维完整链与容量可行性研究
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-021, auto-ui/PLAN-022, auto-ui/PLAN-032]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/topdown2d-feasibility.md]
touched_goals: [GOAL-008]
affects: [autographics-research, topdown2d-capacity]
---

# PLAN-055：M3俯视二维最小链与容量研究

## 0. 变更摘要

在固定已交付AGE/旧SDK上，以最多两种等价机制验证正常Auto Node/use/model/on俯视移动→碰撞/交互→真实呈现的最小完整链，量化地图/对象/回调/资源瓶颈，给出go/bounded-go/no-go和下一份能力合同输入。是M3有界预研，**不交付完整种田游戏/新tilemap或新物理API**。见[首轮设计](../design/autographics/m2-closure-and-m3-first-wave.md)。本轮drafting/r1/0-10，不实施。

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
| T-01 | work | 后续另行启动；独立于054/AG021/020 | 冻结版本/端口/两候选/样本规模/预算单位/输入政策，唯一树/scope | AC-01/05 | [ ] |
| T-02 | work | T-01 | 最多两真实Auto→查询移动→GPU完整最小先证，全部成功/失败/缺口原样记录 | AC-01/02 | [ ] |
| T-03 | work | T-02已有机制结果（含no-go） | 独立轨迹/碰撞/交互反例；不能运行则定位不可达条件，禁止制造绿 | AC-02/03 | [ ] |
| T-04 | work | T-03 | ≤3规模容量/费用/VM/加载绘制/资源分项观测，合法/超界与数组隐患 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 有效最小链故障/20轮Run/修复下一次消费、旧档身份边界；不可达支路明示 | AC-04/05 | [ ] |
| T-06 | work | T-05 | 结论/SD/命令源摘要/容量支持表/后继建议和AG022消费输入 | AC-06 | [ ] |
| T-07 | review | T-06 | 同树独立研究复审，finding返work同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 核实落地文件仅已审知识/必要资产，不把实验补丁当功能合入 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/当前导航，注明go边界/失败与未证项 | AC-08 | [ ] |
| T-10 | merge | T-09 | guard/全路径对账/摘要折Plan/清自有scope和构建物，hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: new / PLAN-055:r1 / outcome: pass（研究合同可审阅，后续启动可进T-01；no-go也是研究结果）；next: work T-01。drafting0/10。当前只规划，未测容量/时长、未构建/探针/GPU/新功能/材料scope/树/SDK/canonical；不改变M2验收或将本Plan设为AG021前置。

2026-11-05 / stage: new / planning_validation: pass（合同预检，非实施/独立final）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD、drafting/r1/0-10与未勾任务齐；AG021 r2保同一分母，现场原owner已记录executing2/10及T-03等待054，未回退其进度。依赖DAG无环，M3研究不是M2前置；独占号源后重新扫描主/归档和注册树核新号054/055/AG022唯一，next UI056/AG023。三仓49处新增/修改本地链接可解析，白名单空白检查通过；现有`python tools/gallery.py validate --all` exit0，12逻辑项/22入口/许可保持。048/049归档与canonical、019归档/候选canonical逐字节内容未改（SHA前16分别dfbd9df1087d52a7/b7f5c867ee107c2a、7bda6f53faf16cae/8967cc6c8cf87f0a、1d8e991170ec3969/601a457ab52edfec）。未构建/运行新源码或GPU/原生输入/容量研究、未写新canonical/资产/锁、未创建树/scope/agent/发布或迁目录；ag-021已有树与材料属原owner，保留其WIP，本会话不代其提交执行进度或工作收据。第一次预检因AG021并发进入executing不再符合原drafting断言而停，按新证据更新导航后复核通过，不把这次状态变化归为实现测试红。

## 10. 待澄清事项

D-01：实际无重力/角色配置和query＋sprite组合以T-01/02代码/真链裁决，不假设接口存在。D-02：2DHD美术/摄像机路线未定，提供选项与依赖不擅自选整套效果；最终内容选择归AG022合同。D-03：容量数字/时间仅真实测量，未证/超界/no-go完整保留。scope唯一`D:/autostack/plan-artifacts/plan-055/auto-ui/`；先核owner避免OS055跨仓材料冲突，登记用途/在途；源码探针/SDK副本/target/外部消费/日志均scope。merge必要源码/命令/失败摘要折Plan并核全部实际输出，guard清自有树/删scope构建物；具名hold可留owner/理由/关闭条件，禁止链接/不触别仓同号。
