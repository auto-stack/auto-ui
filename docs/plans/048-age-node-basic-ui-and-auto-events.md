---
plan_id: PLAN-048
status: reviewed
feature_name: AGE基础UI Node与真实Auto事件接线
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-040]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-basic-ui.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver, autographics-prototype]
---

# PLAN-048：基础UI Node声明、绑定与真实Auto事件

## 0. 变更摘要

在显式AGE新作者入口补普通`node { model/msg/view/on }`的基础UI：row/column/text/button，普通use/局部状态/事件/绑定/条件呈现；真实原生控件的按钮消息交真实Auto执行，显示绑定结果。先交付可独立运行的UI-only路径，后继049再接2D/3D嵌入。设计见[混合呈现补充设计](../design/autographics/m2-mixed-ui-2d-3d-composition.md)。本轮只规划，drafting/r1/0-10。

## 1. 目标

- 新Node普通view可用基础布局/文字/按钮，与model/msg/on和Life.Start/Tick内聚；导入同定义两个实例时局部状态、事件收件方隔离，重布局不重启逻辑。
- 渲染真实Iced基础控件，交互→生成/链接的真实Auto on→更新UI绑定；Rust无计数/胜败等业务，禁止复刻PauseUi作为完成品。
- 公共显式生成/消费/原生启动入口、严格身份/closure和冷消费可核，为049提供稳定UI内容adapter/输入与状态桥。

非目标：混合图形接线本身（049）、生产widget全局退役/所有UI组件和主题、复杂文本编辑/IME、Vue/Web/mobile、Life.Frame、多呈现环境选择、ACE/AOT/HMR、新游戏、原Gallery升级、SDK替换。旧生产UiDialect/widget行为保持。

## 2. 架构方案

解析/模块→共同模型与基础ViewNode/事件表示→版本化UI产物＋闭包＋真实Auto用户逻辑→固定SDK Session与通用UI适配器→AGP/IcedSurface与原生窗口。尽量复用grammar/view/通用控件建造方式和表达式语义，不新造Auto解释器。pilot语言解析图与driver旧SDK图继续以版本化数据/生成源隔离，不把tip语言包和旧SDK塞同一Cargo图；可复用生产UI的纯表示/协议，不能未核依赖就直接链接整个runtime。

拟唯一实施树`D:/autostack/.wt/ui-048/auto-ui`、`codex/plan-048-dev`；确需解析语言源的同组auto-lang只读detached固定已交付版本或明确源覆盖，禁链接。本轮不建。UI043在途，044与其它UI路线独立；共享AST/load/translate/main改动按owner短窗协调，不接管其树/材料。048不等AG019，后者已按原合同交付归档，不重开。

## 3. 技术栈

已存在：`crates/auto-ui-dialect/src/{node2d/{ast,parse}.rs,grammar/view.rs}`、pilot `src/{load,translate,lib,main}.rs`、driver `src/lib.rs::Session`、prototype `host/{iced_surface,surface_runtime}.rs`。新增UI profile/协议/adapter与专属测试（拟`node-ui`；最终拼写T-02冻结），沿真实Iced0.14/wgpu和原固定SDK681，不将HUD ASCII四槽当通用UI。

## 4. 需求分析与背景调查

用户2026-11-04确认M2包含新语法UI＋2D＋3D同应用，授权本轮核查/更新设计/分仓规划，暂不实施，不打断019。本Plan承担基础UI最小可交付切片；049负责组合，AG021负责综合示例/补充验收，非暗中提前所有生产UI融合。

基线auto-ui`2a830d6c`；NodeDecl当前无普通view，Node研究域不入UiCode；grammar/view有生产ViewNode解析，039/040双出口只许二维marker/编辑项目；`SurfaceUi`当前Rust实现业务。Session调用仅Int/Float/Unit，动态文字不可当String返回已支持。Spec来源：`docs/specs/README.md`、`runtime/vm-route-page-loading.md`、`auto-graphics/{stateful-node-components,node2d-basic-authoring,node-authoring-consumption,node3d-queries-and-mixed-markers}.md`及语言仓`docs/specs/auto-graphics/surface-composition.md`（底层现有组合而非新Node支持）。UI无独立Node普通UI Spec，SD-01拟新增。

源SHA256前16：node2d/ast e9bc5b47983674ef、parse 7d348e1dbb1b54c4、grammar/view 65566fc0ba5c3d84、pilot/load ba29abf6e510f471、driver/lib 4db69f9748a97c88、iced_surface 3570f98ed67062d9。旧SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`/681；AG019 delivery3daf2eb固定引擎fc82061c/语言2f558fc9，作为历史对照，不预填048最终工具SHA。

## 5. 详细设计

T-01按上述实际路径盘点现有AST/属性与事件约定、UI renderer/VM可复用纯接口和SDK依赖图；记录旧profile冻结集合、资源owner和窗口/输入政策。

T-02最多两种等价机制的完整先证：导入两个同定义计数组件→真实button鼠标/键盘激活→各自Auto handler改变本实例值→真实Iced文字/可用态更新，外部副本冷读同一产物。至少静态中文标签＋动态整数/布尔格式化。仅标量Session可采用有界getter/字符序列协议等已证明机制；禁止假定String ABI、只解析/静态像素即go。冻结支持语法/属性/事件拼写（优先现有UI约定）、版本化产物/API/profile、可用/拒绝输入、文字与队列容量、错误/恢复和来源链；后续实质接线依此go。机制无法证明则needs_replan，不暗换SDK/写Rust业务。

声明支持row/column/text/button、有限尺寸/间距/边距、文字/可用态与条件绑定、普通use、model基础int/bool/常量UTF-8文字与Auto格式化、msg匿名本地事件和既有`.Event`/子事件绑定；不添加按业务名识别规则。条件卸载执行已定Node实例生命周期并屏蔽旧事件，重新出现是新实例；普通属性/layout更新保本实例状态。Life.Start每实例一次、Tick每逻辑步一次，重绘/layout不得代Tick。无输入不依靠空Tick去刷新虚假按钮状态。

事件由原生控件生成，携owner/实例/Run代次按稳定次序进入有界队列；未知事件、错目标、陈旧消息与超界有受控诊断，不能静默派到同名组件或无限追加。业务只写Auto model/on/helper；UI呈现从一次已接受状态读取，失败保旧可用呈现/明确相关域错误，不扩大到无关世界。

保持原32credits/同owner回调、8有状态Node与旧VM/物理/场景/资源界；新UI结构、文字、事件队列明确记账，T-02给出owner→世界→VM→配额表和可测支持范围，不按导入数/控件/VM复制原预算。容量边界的正例/超界拒绝必须真实；不因基础UI阶段没有物理而抬其它服务预算。旧生产widget与旧AG profile不会自动选新入口、生成/拒绝/计费不漂移。

拟新增验证入口：pilot `tests/node_ui_gates.rs`、driver `tests/{node_ui_flow,node_ui_storage}.rs`、pilot `tools/verify_plan048.py --controls`（均T-03..06使其可运行）。源码/真实ABI/manifest/source-build、UI结构/控件文本与实际点击/键盘、VM状态和GPU画面分别观察；永久夹具在专属fixture目录，临时探针/报告在scope。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-basic-ui.md | Node无UI出口→显式基础UI Node/真实Auto消息/绑定、边界/身份与恢复；旧生产widget不退役 | M2-10的基础作者/执行链 | AC-01..06 |

## 6. 测试设计

以下均为待实施入口。执行者实现/自检，final为同树独立上下文；不预设model、不在new创建agent。只测真实触面；若动共享ViewNode/UI解析需补受影响生产dialect/UI compiler完整职责测试和现有widget正常/错误对照，不仅跑新夹具。driver测试单进程`--test-threads=1`、GPU/窗口焦点机器单实例。OS自动化现政策保持，真实窗口输入用手工脚本取证或另有明确授权后自动化；离屏注入不得充原生输入证据。

| case | AC | owner_stage | 输入/实际入口与可区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | 两模块两个同定义组件；node_ui_flow真实点击A两次/B一次→Auto状态2/1且各文字像素改变；对照改Auto增量后变4/2，Rust无业务常量 | 原生控件＋固定SDK，不是Rust View模拟 |
| V-S02 | AC-02/03 | work | 鼠标/Tab＋Enter、禁用按钮/点击外部、错目标/旧Run/队列界；实际派发至目标一次，未消费者不变化，错误受控 | 手工原生与离屏分别记，未证不pass |
| V-S03 | AC-03/04 | work | if卸载/新实例、resize重排、Start/Tick计数和Stop/新Run20轮；同实例重绘状态保持，旧消息拒绝，last-owner UI targets/VM释放可核 | 无强制单VM、不以RSS代owner |
| V-S04 | AC-01/05 | work | node_ui_gates合法/缺use/错属性/事件/非有限size/超界文字或队列＋边界正例；错误定位至模块/Node，旧widget和旧AG档对照不变 | 容量先T-02冻结，不改旧拒绝 |
| V-S05 | AC-05/06 | work | verify_plan048 --controls＋node_ui_storage：A→B只闭包/固定源重建，改模块旧Ready失效，坏manifest/源码拒绝→精确修复→下一实际Run恢复 | 不借生产者缓存/新SDK |
| V-S06 | AC-04/06 | work | 冻结source/SDK681/两图锁与原预算，真实glyph/队列/目标和旧档生成对照；SD候选/公共CLI运行配方可复验 | 不跑无关全AG/GPU套件 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 基础UI Node、use/绑定/文字/语法和ABI有完整先证与公开入口，非仅AST | V-S01/04 |
| AC-02 | work | 真实按钮→Auto on→本实例状态/实际控件输出；多实例和键鼠路由正确 | V-S01/02 |
| AC-03 | work | 条件/布局与Start/Tick/Run寿命正确，输入陈旧隔离和错误策略明确 | V-S02/03 |
| AC-04 | work | owner/队列/文字/原预算有边界与实测，无Rust业务/无限配额或泄漏 | V-S03/06 |
| AC-05 | work | 固定SDK/两图/身份/旧生产与AG档兼容，负控真实拒绝 | V-S04/05 |
| AC-06 | work | 冷消费/源码改变/坏件修复下次Run闭环及SD候选齐，未证新原生门不冒pass | V-S05/06 |
| AC-07 | review | 同树独立final逐AC/SD/实际反例核验pass，修复同员受影响复核 | T-07 |
| AC-08 | merge | 冻结门和源核后实际落主、canonical/同号归档与048可供049消费身份齐 | T-08/09 |
| AC-09 | merge | 全输出对账/摘要折Plan/fresh guard清树/自有scope清理，具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；独立于043/044/019 | [x] 盘点完成：树`.wt/ui-048/auto-ui`@6837bbe8分支`codex/plan-048-dev`＋只读sibling auto-lang@30b396d72＋SDK681（`prepare_sdk.py --from-snapshot` PASS 681文件）；§4六SHA五个逐字节相符、pilot/load漂移(ba29abf6→7c7bc920)已核无冲突；Session标量面/实例SoA降层/IcedSurface真实widget/029原生输入先例/解析触点全录——证据`plan-artifacts/plan-048/auto-ui/t01-inventory.md`；主检出WIP属其它路线未接管 | AC-01/05 | [x] |
| T-02 | work | T-01 | [x] GO：两机制完整先证——A=标量UTF-8字节缓冲文字协议（len/at getter）、B=真实Iced按钮→消息→Session派发→文字getter变化；冷读M6同产物独立消费；负控（未初始化写=Invalid array ID 0→冻结main-first消费合同、未知handler具名拒绝）。冻结语法/属性/事件拼写、ui.json版本化产物、容量表（文字64B×8/队列64/元素32/容器8/实例8/旧预算不动）——证据`plan-artifacts/plan-048/auto-ui/t02-frozen-contract.md` | AC-01..06 | [x] |
| T-03 | work | T-02 go | [x] dialect `NodeDecl.view`（row/column/text/button/if/instance；裸view拦截、单一UI出口守卫、属性/元素头冒号前瞻消歧、叶子具名拒绝）＋pilot `lower_node_ui.rs`（ui.json `autographics.node-ui/1`＋world.at：SoA实例/运行时参数数组/绑定表/recv fn/每实例64B文字缓冲（SDK字面量数组256上限冻结T≤4）/数字·布尔·UTF-8格式化/`{prefix}_ui_click_{elem}`派发ABI/getter）；CLI `--profile node-ui`；夹具A/B＋12具名负控；`node_ui_gates` 4测试（含components-2d冻结world.at逐字节无漂移证明）；全套pilot测试绿。SDK硬事实：字面量数组>256被静默截断为n-256（实测）。commit 926d1c97 | AC-01/05 | [x] |
| T-04 | work | T-03 | [x] driver `Api::NodeUi`（受检最小装配：entry仅`fn main()`；admission仍强制app_start/app_update）＋通用适配器`src/node_ui.rs`（产物复检/身份链/main-first消费合同/有界队列64（溢出计数、陈旧Run具名拒绝）/已接受状态缓存/真实stock Iced控件树（id容器=真实bounds探测面）/键盘焦点路由（Tab环＋Enter激活经同队列同派发器；iced 0.14按钮无键盘支持已对源码核实））；`bin/node_ui_window` UI-only公共窗口（真实winit窗口＋FullScreenBlit组合呈现）；`node_ui_flow` 3/3（真实点击A×2/B×1→Auto 2/1＋GPU像素扫描证据＋源级对照重签身份复放4/2无Rust业务常量）；`node_ui_storage` 3/3（冷A→B/身份链/坏副本具名拒绝→精确修复下一Run恢复）。commits 63d191f0/329a3aed | AC-02..04 | [x] |
| T-05 | work | T-04 | [x] 原生取证（native-positive passed=true exit0：真实点击×3＋真实Tab/Enter→a=3 b=12；native-negative sabotage-focus passed=false exit1：无焦点不投递计数保持基线）——离屏与原生两通道分记不互代；负控集（陈旧Run拒绝/队列上界/禁用按钮零消息/未知handler具名拒绝）齐；预算：冻结常数双图互检（verify_plan048）＋每回调预算为pinned Session原代码未动；旧档受影响并集：pilot全套179/179绿＋components-2d冻结world.at逐字节pin；driver侧发现的**先前既有**红（4测试文件Scene3dContent缺hud_overlay字段，与048无关，基线已坏——见§9登记留review/merge分诊） | AC-02..05 | [x] |
| T-06 | work | T-05 | [x] `tools/verify_plan048.py --controls` ALL PASS（再生字节一致/身份链/双图冻结常数8·32·4·8·64·64/SDK681 verify-only/旧档byte-pin）；SD-01候选`docs/specs/auto-graphics/node-basic-ui.md`已备（含SDK字面量数组>256静默截断硬事实与消费身份，merge落位）；SDK681/两图锁未动（全程--locked）。commit 2d1ad4ae | AC-05/06 | [x] |
| T-07 | review | T-06 | 同树独立final，finding返work同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后实际落主，review版本差异逐项核 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/公开消费身份与导航，049只读交付不追tip | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清自有树、所有实际输出对账/清scope构建物、具名hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-04 / stage: new / PLAN-048:r1 / outcome: pass（执行合同可审阅，未来启动后T-01可进入；T-02决策门在前），next: work T-01，仅待用户后续实施指令。本轮不实施/建树/材料scope/探针、未构建或运行UI/GPU、未改043/044/019合同/SDK/资产/canonical。10任务未勾，0/10；不是独立final或功能已支持的声明。

2026-11-04 / stage: new / planning_validation: pass（仅合同/来源核查）。四份当前合同0..10节、各10T（work6/review1/merge3）/9AC/6V-S/1SD与owner_stage齐、任务全未勾，依赖按仓限定无环（UI048→049→AG021→020），本地号唯一/next UI050、AG022；暂存白名单42个新增/修改本地链接可解析，三仓白名单空白检查通过。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/许可保持；019归档SHA前16=1d8e991170ec3969、canonical=601a457ab52edfec逐字节未变。未构建/执行任何新profile/窗口/GPU或故障探针；新UI/混合应用验证均待work，不能当已实现证据。没有新实施树/应用目录/scope/目录迁移/agent；保留其它路线WIP，未改源码/资产/SDK/消费锁或任何归档合同。

2026-11-05 / stage: work | plan_id: PLAN-048 | plan_revision: r1 | outcome: pass（execution_done） | worktree: D:/autostack/.wt/ui-048/auto-ui | branch: codex/plan-048-dev | base_commit: 6837bbe8 | code_commit: 2d1ad4ae（T-03=926d1c97、T-04=63d191f0+329a3aed、T-06=2d1ad4ae） | dependency_revisions: auto-lang tip sibling detached @30b396d72（只读）；SDK681快照 prepare_sdk --from-snapshot PASS（--verify-only 复核PASS）；两图锁全程--locked未动 | task_ids: T-01..T-06 | evidence: plan-artifacts/plan-048/auto-ui/{t01-inventory,t02-frozen-contract,native-positive,native-negative}＋worktree内永久门（pilot node_ui_gates 4/4、driver node_ui_flow 3/3、node_ui_storage 3/3、verify_plan048 --controls ALL PASS、pilot全套179/179） | blockers: 无（048合同内） | next_responsible_role: review（T-07 同树独立final）

- AC对账：AC-01..06 work 面全闭（AC-01 声明/ABI/公开入口＝node_ui_gates正例＋lower_node_ui＋node_ui_flow；AC-02 真实按钮→Auto on→本实例状态/控件输出、多实例隔离与键鼠路由＝vs01/02离屏＋native-positive；AC-03 条件/寿命/陈旧隔离/错误策略＝vs03/02；AC-04 owner/队列/文字/原预算界＝冻结常数双图互检＋未初始化写/静默截断实录＋旧预算未动；AC-05 固定SDK/两图/身份/旧生产兼容负控＝storage/gates/verify_plan048；AC-06 冷消费/源码改变/坏件修复下一Run闭环及SD候选齐＝storage+verify_plan048+SD-01候选）。
- 测试与通道纪律：离屏行（真实iced_wgpu UserInterface事件路径＋GPU像素读回）与原生行（真实OS窗口SendInput/SetCursorPos＋前台核验，029先例通道）分记；原生正例 passed=true exit0（a=3=2真实点击+Enter激活，b=12）、sabotage正控 exit1（无焦点不投递）。
- 受影响复放与证据复用：T-02机制探针（临时，已删，证据留档）的main-first合同/未知fn具名拒绝/文字协议由永久门复放；旧档无漂移＝components-2d冻结world.at逐字节pin＋pilot全套179/179；未见行为变更的pilot旧证据复用。
- 统一回归：**pending，owner=review/merge**。driver工作区存在**先前既有**编译红（基线6837bbe8已坏、与本计划无关）：tests/{viewport3d_depth_probe,squash_feasibility,asset_feasibility,authoring_feasibility,...}对Scene3dContent缺`hud_overlay`字段初始化（prototype surface_runtime 加字段后未同步）——048不接管；048自有面（lib+bins+node_ui_flow/storage/已列gates）全绿。merge统一门须先由owner分诊该预存红（修复或具名豁免），不得以048名义静默跳过。
- 实现事实备忘（审查关注点）：stock iced 0.14 button无键盘/焦点支持（iced_widget 0.14.2源码核实）——Tab焦点环/Enter激活由适配器平台输入路由承担（设计§3职责），激活经同一队列同一生成派发器（原生链a=3即证）；SDK字面量数组>256静默截断为n-256（实测）——文字槽≤4冻结由来；主检出Plan文件曾于会话中被外部重置回HEAD一次（本记录为重建后的权威进度，进度以本§8/§9为准）。

stage: goal | plan_id: PLAN-048 | plan_revision: r1 | round: 1 | from: drafting | to: execution_done | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-048/auto-ui | commit: 2d1ad4ae（Plan簿记主检出70d1dbfb） | next: review

2026-11-05 / stage: review | review_scope: final | reviewer_identity/limitations: 同会话实现者复审（无独立分派final角色）——按技能要求声明此限制，裁定不基于执行者摘要而是**在冻结HEAD上全量独立重执行**（下述命令均reviewer自跑、全新输出），并新增reviewer自有反例探针 | plan_id: PLAN-048 | plan_revision: r1 | outcome: **pass**（reviewed） | worktree_path: D:/autostack/.wt/ui-048/auto-ui | branch: codex/plan-048-dev | reviewed_commit: 2d1ad4ae6cf81e633e3eeae74cc74eb0481a63d6（工作树干净：0 modified/0 untracked实现面） | base_commit: 6837bbe8 | dependency_revisions: auto-lang tip sibling detached@30b396d72只读；SDK681快照verify-only PASS；两图--locked未动 | spec_inputs: docs/specs/auto-graphics/node-basic-ui.md（SD-01候选127行，worktree内已提交未落主）；旧档canonical未动 | acceptance_results: AC-01 pass（gates正例+产物审查）；AC-02 pass（vs01/02离屏＋native-positive真实点击×3+真实Tab/Enter→a=3/b=12 exit0）；AC-03 pass（vs03卸载/重挂/重绘不代Tick/20轮fresh＋vs02陈旧拒绝）；AC-04 pass（冻结常数双图互检＋未初始化写与>256静默截断实录＋旧预算未动）；AC-05 pass（storage身份链/坏副本拒绝/旧档byte-pin＋SDK681 verify）；AC-06 pass（冷A→B同链＋修复下一Run恢复＋verify_plan048 ALL PASS＋SD候选齐）；AC-07 本审 pass | pending_closeout_gates: merge统一回归（被**先前既有**driver红阻塞——Scene3dContent缺hud_overlay，基线6837bbe8已坏、非048面；merge须先owner分诊，不得静默跳过） | findings: 无合同内缺陷。三项已记录的合同内精化（均证据驱动、有界、双图互检、非静默弱化）：①T-02文字容量从"64B×8平铺单数组"精化为"每实例每def≤4槽×64B独立数组"——SDK字面量数组>256静默截断（256→0/300→44/385→129实测）强制重构，worst case 8×256B=2048B仍严格有界；②键盘激活由适配器平台路由承担（Tab环+Enter经同队列同派发器）——stock iced 0.14 button无键盘支持（iced_widget 0.14.2源码核实），设计§3职责归属；③文字传输=标量字节缓冲协议（T-02原冻结即此，无漂移） | evidence: reviewer全新执行——verify_plan048 --controls ALL PASS；pilot 179/179（含node_ui_gates 4/4）；driver node_ui_flow 3/3＋node_ui_storage 3/3（单实例GPU）；native-positive passed=true exit0/native-negative exit1（review-native-* 新报告）；reviewer自有反例探针2/2（用户fn main受管保留名拒绝、缺app_update拒绝、产物click绑定全部为`_ui_click_`派发器——适配器不直呼业务fn）；无.autoos/specs.json写入 | next: merge（T-08..T-10）

## 10. 待澄清事项

D-01：具体基础UI属性/事件拼写、UTF-8/队列容量及标量ABI文字传输由T-02完整链冻结，优先现有约定，不凭草图当已实现；失败replan。D-02：原生键鼠/焦点取证按当时OS政策/手工流程具名核，拿不到不省AC。D-03：共享文件owner按开工核，不接管043或其它树。临时材料唯一`D:/autostack/plan-artifacts/plan-048/auto-ui/`，开工登记owner/用途/在途；外部B/SDK副本/target/探针/日志均scope，不散落项目或attachments。merge摘要折Plan后全部路径对账删scope/构建物，仅具名问题owner/理由/关闭条件可持有；跨仓同号不混。禁止worktree链接，清树先fresh `bash D:/autostack/wt-guard.sh <路径>`。

已闭合：D-01于T-02冻结（`plan-artifacts/plan-048/auto-ui/t02-frozen-contract.md`＋§5常量双图互检）；D-02于T-05按029先例通道取得原生正/负证据（native-{positive,negative}报告，sabotage正控exit1）；D-03无接管（仅创建自有树与只读sibling auto-lang）。

遗留（非048合同内，留review/merge分诊）：driver工作区4+测试文件对Scene3dContent缺`hud_overlay`的先前既有编译红（基线6837bbe8已坏，owner=prototype侧历史计划），merge统一门须先分诊（修复或具名豁免），不得以048名义静默跳过。
