---
plan_id: PLAN-021
status: archived
feature_name: AutoGraphics Node二维segment与sweep查询作者能力
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 10
total_steps: 10
depends_on: [auto-ui/PLAN-020]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-queries.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-021：G05/2d所需的Node查询作者能力

## 0. 变更摘要

在020局部组件合同上提供有限二维segment和sweep服务、typed节点/结果引用、空间输入与标记呈现，让G05/2d的查询/运动规则可写在自己的Node model/on中，由真实Auto执行。保留旧v3/v4数值、结果寿命、费用和原例完整验收义务。

本轮仅规划drafting0/10；不迁移Gallery内容、不实施/建树/GPU/agent。依赖020实际交付，与022无功能依赖；AG008继续固定019。[共同设计](../design/autographics/node-components-queries-and-sprites.md)规定共用组件IR、短窗接线与原预算。

## 1. 目标

1. 普通use可导入查询组件/helper/资源，`.field`和事件归本实例；具名NodeRef/ActorRef代替作者手写document id/ordinal/slot。
2. segment最近命中与sweep有限体积移动在新作者链真实生效，query结果映射到正确标记、point/normal和显隐；主/changed源产生真实差异。
3. typed结果保旧世界共享latest-slot及跨callback/错kind/失败attempt失效语义，多组件不获得私有免费结果槽；原f32和32费用不变。
4. 交付可固定消费/冷重生/公开Run与source-build身份，后续Gallery迁移独立立合同，不追本计划开发tip。

非目标：3D查询/GLB/triangle或OBB精确碰撞、无限ray、新physics/自动slide/相对动态目标、角色玩法扩面、精灵/音画、长期query快照容器、通用借用检查器、CPC/ACE/AOT/ECS、预算提升/固定SDK语义修改或G05内容仓改造。

## 2. 架构方案

普通Node模块→020组件IR/receiver/具名节点图→typed空间服务/引用lowering→原autographics.behavior/3或/4受信assembly→pinned SDK真实Auto→Host空间帧/GPU。新作者模式拟query2d-segment/query2d-sweep，T-02冻结准确CLI/端口；不按G05文件名分支、不复制另一套实例运行时或Rust几何算法作为业务。

拟唯一实施/同树顺序复审树D:/autostack/.wt/ui-021/auto-ui、codex/plan-021-dev。主检出只Plan簿记；020真实delivery作为基面与SDK/生成器依赖固定。与022主要新增独立lower/service模块，共享load/translate/main/identity接线按owner短窗；无法隔离则顺序，不在同树并发写。

## 3. 技术栈

已存在pilot src/{load,translate,lower_basic2d,lower_g11}.rs、dialect node2d AST；driver api/{behavior_segment_v3,behavior_sweep_v4}.at、authoring/{behavior,spatial,document,hierarchy,play}.rs、样例空间查询及相应用例。020将交付的组件IR/接口路径尚待实际冻结，不引用其开发tip。

拟新增pilot查询service/lower模块、fixtures/queries2d/{segment,sweep,changed,...}.at、tests/node_queries_gates.rs、driver tests/node_queries_flow.rs、tools/verify_plan021.py。永久夹具/测试归本仓；原G05源只读，临时探针单独指定scope，不创建Gallery生产文件。

## 4. 需求分析与背景调查

用户2026-11-01要求规划下一轮二维查询/精灵与有状态组件问题。本计划只认领G05/2d能力和原义务，未授权实施/agent/放宽验收或修改外仓。020已起草但未交付，021实施硬等它；不等待AG008完成，也不把021加成008前置。

当前观察auto-ui10fdf79（019交付c93adb7），ag-examples912bd70。权威：[segment](../../../auto-lang/docs/specs/auto-graphics/segment-query-profile.md)、[sweep](../../../auto-lang/docs/specs/auto-graphics/swept-aabb-profile.md)、[层级](../../../auto-lang/docs/specs/auto-graphics/scene-hierarchy.md)、[019消费](../specs/auto-graphics/node-authoring-consumption.md)及[020合同](020-autographics-stateful-node-components.md)。

原G05/2d raw SHA：segment.at 02fd3678115c053dcc866c7ab8bd1a3feac897296acb903514de5ccb74aedfd0；sweep.at ed2d9fbbb51f5aa6c5a4f68f722029ca2405743eba6dd7751730035267f14e45；variants/segment-changed.at 87610d6aebbdd233a3809112d142dbc5a484c2e328f22f7fdf479c34604126de。当前metadata只默认segment(v3)，README称的sweep变体没有实际variants字段；本计划需能力全链/明确公共模式，不把旧文案当已通启动证明。

原verify_examples::verify_g05按维度循环有9类require：segment成功、14phase、非空trace、saveas导出、sweep成功、14phase、changed合法、changed source真变、partial拒绝。原probe的底层数值/窄相位与图形观察不因banner只有1项而删；OS输入自动化暂停，不能伪称新链已实跑硬件。

## 5. 详细设计

### 5.1 进入门和先证

T-01核020独立pass/冻结门/master实际delivery、组件Spec与清理；重绑SDK7b987ca8/681固定副本和全部锁/生成器依赖，不能升级SDK。记录AG008/022/018实际owner和共享端口，仅读固定内容基线。

T-02最多两接法先完成segment→typed结果→标记、sweep→Auto显式clearance移动→GPU、reach20→10敏感性三条闭环；校准全部callback费用/时序。将精确公共语法、typed引用/结果、变换与声明绑定可见时机、CLI/API/九类义务映射、oracle/超时冻结到Plan§9。不需新增parser/VM或改外仓；不成立带反例needs_replan，不默提额或由Rust补运动。

### 5.2 服务、引用和数值

- 节点/target/ignore通过具名typed引用按展开图解析；ignore可合法缺省，底层handle0仍合法，不把0当absence。query不可命中group，错维/坏引用/非法visible_only在执行前或旧受检边界具名拒绝。G05标记可用具名target→展示位置映射，不强迫作者手写ordinal；诊断记录保真实运行映射以对照原node字段。
- 空间输入沿原ag_input_space/x/y契约，不能把space误作Jump键或强迫启用角色GameInput；准确typed拼写与键位/viewport条件T-02核。保segment移动60×dt、sweep请求位移speed100（原码不再乘dt），不为统一代码形式改变单位。
- 原finite f32、2D z=0、闭区间、signed-zero平行、零长、退化箱、inside/boundary起点、t1切触、最近真实t与精确同t按Run ordinal、X/Y法线择优保持。二维圆形mesh此处按真实AABB查询，不借G11圆触发替换。
- sweep仍固定朝向world AABB、目标在查询瞬间静止；起始overlap/contact→t0/normal0（含朝外请求），未命中→O+D，显式clearance由Auto规则处理，不加隐形skin/slide。源已完成的显式set/move在同callback后续query可见；声明绑定何时同步与命令写入分开说明，不重排有副作用操作。
- 所有matching geometry在合法nearest裁剪前按旧规则finite-check；GeometryUnavailable9与ArithmeticNonFinite12不能降为miss。观察容差仅t=max(1e-6,8EPS)，position=max(1e-3,8EPS×max(1,abs(expected),abs(actual)))，hit/miss、ordering、normal符号/轴、overlap不以容差放宽。

### 5.3 typed结果与局部组件

返回typed Cast/Sweep result句柄，getter仍进入原API；同一世界共享一个latest typed slot，不为各Node复制结果快照或配额。下一query/失败attempt、callback begin/end、Stop使旧token失效；setter不改当前结果快照；错kind/stale/missing按原13、序号耗尽14不回绕。不得把typed包装宣称永久有效；长期状态保存须读取为值。

两独立query组件同世界顺序执行：A读完自己的结果再B查询合法，A在B后再读旧token必须拒绝；这不是状态串线。Node自身model/marker/事件仍隔离，错误receiver不能改另一实例。普通helper内的合法结果读取保类型/费用；不新增通用编译期生命周期分析承诺。

保marker/door/point-marker实际展示hit/node/t/point/normal，sweep写actor后再次读origin快照应不变。主/changed改变reach20→10必须改变实际query或像素，只有source hash不同不足。两Node getter及转换只按原费用，不私读VM全局或免费memo来绕过API。

### 5.4 包络、消费与兼容

cast8、sweep16、result getter1、继承AABB1，共享32/callback。原segment夹具候选13（cast8+5getters），sweep候选23（16+6getters+一次重读）只是计费草算，真实完整路径T-02校准；32合法/33拒绝和token失败先失效的反例保留。每Node不reset预算，原VM/时限/源码/managed/128节点/查询标量状态包络沿各旧profile，8ms目标另列，不创建无限结果列表。

沿020公开消费与source-build合同扩展模式/角色/typed结果映射、全部输入闭包和实际generator/runner/锁/报告身份。只用旧已支持持久格式/绑定机制，不为新作者入口擅改旧API版本。source-only A→B冷再生+真Run、保存/SaveAs/新根重开、坏件拒绝→精确恢复可用；不依赖生产者源码/target/网络。旧020/019/017真实受影响路径保持，未迁移G05/3d和Gallery锁不变。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-queries.md | 原v3/v4底层能力→新Node服务/typed引用与结果寿命/输入/数值费用/消费和未支持面 | 后续G05内容迁移有实际固定版本 | AC-01..06 |

## 6. 测试设计

T-03..06拟建pilot --test node_queries_gates、driver --test node_queries_flow -- --test-threads=1、verify_plan021.py --controls；准确选择/数量/命令T-02冻结，目前未运行。独立f32 slab/Minkowski oracle不得调用被测SDK查询函数；原9类/14阶段要落为实际新源状态与trace检查，不能把旧probe旗标空退出当通过。确需真实OS且无法有效复用时保持gap交专项授权，不擅自恢复抢焦点。

| V/S | AC / owner_stage / T | 输入→真实入口/范围 | 区分oracle/关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | imported组件/具名target/ignore、handle0、group/错维/错服务/缺资源 | binder和真实执行对象映射正确，错引用带模块/行；改名不改几何，改目标位置真改结果 |
| V-02/S-02 | AC-03 work T-04 | segment hit/miss/inside/零长/signed-zero/tie/切线/parent更新→Auto/Host/GPU | 原数值顺序/法线/nearest独立算式，marker与真实结果一致；AABB错序/旧位置/几何异常当miss负控红 |
| V-03/S-03 | AC-03 work T-04 | sweep薄墙/endpoint离散miss/overlap/away/zero、明确clearance与origin重读 | 有限体积早于origin cast、t0/overlap精确，移动后原token快照不变；自动slide/epsilon放宽负控可区分 |
| V-04/S-04 | AC-04 work T-05 | 两组件latest-slot、第二次/失败query、错kind/跨callback/Stop/fresh、owner20短轮 | 旧token被正确拒绝而节点状态不串；合法值快照保留、序号不回绕/无结果列表增长 |
| V-05/S-05 | AC-01/05 work T-02/05 | G05 segment/sweep/changed、九类义务、32/33/CPU守护 | 实际新源结果/trace/保存导出/partial拒绝与GPU；full Tick费用实测、不降14阶段义务，OS证据边界真实 |
| V-06/S-06 | AC-06 work T-06 | 固定消费A→B、closure/结果类型/锁/runner坏件、SaveAs重开/精确恢复、旧组件消费者 | identity+真Auto/GPU与重开后的同入口闭合，空/partial/旧报告拒绝；不修改Gallery内容 |

work自查→同树独立上下文review→merge冻结门，无指定模型，本轮不创建agent。选新增/受影响完整定向集和必要共享UI消费者，非全AG；GPU/重型门单实例，OS自动化仍暂停。

临时材料唯一D:/autostack/plan-artifacts/plan-021/auto-ui/，开工登记owner/用途/在途；外部消费/SDK/target/report显式此处，禁止attachments含md。契约/SD/证据/移交摘要折Plan§9；merge路径对账后删自有scope，具名未解决问题按owner/理由/关闭条件持有，未知/他仓材料不盲删。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 020实际基线/稳定组件合同、三条完整链与原费用先证，未复制执行器/外仓语义 |
| AC-02 | work | 普通use与typed节点/结果/输入准入和精确诊断，V-01 |
| AC-03 | work | 原segment/sweep几何数值/显式变换/标记与GPU、changed实际敏感，V-02/03 |
| AC-04 | work | 跨组件共享latest-slot/结果寿命、局部model与owner/旧Run隔离，V-04 |
| AC-05 | work | G05九类与14阶段义务可追溯、原额/CPU/失败恢复与OS边界准确，V-05 |
| AC-06 | work | 固定source-build公开运行/A→B/持久重开/坏件恢复与旧消费者兼容，V-06 |
| AC-07 | review | 独立逐AC/V/S/SD/原分母/遗漏/格式/warnings核查，绑定提交，无未处置finding |
| AC-08 | merge | 冻结定向门/有据复用后实际合master，准确Spec/归档/来源导航 |
| AC-09 | merge | guard核树清理、全部输出路径对账/材料删除或具名持有、登记与10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续启动、020实际交付 | 核Spec/SDK/依赖/owner，建唯一ui-021树；固定G05源与原义务 | AC-01 | [x] 2026-11-01：020门核绿——归档件docs/plans/archived/020-…status:archived（提交7f42d5f/6ec7524），交付16ddfa0在master祖先链（ff无merge commit），canonical spec stateful-node-components.md@16ddfa0在案，三树/分支/plan-artifacts-020清零（其goal收尾3dd050f/88c96ba实证）。ui-021组3树建成：auto-ui@3dd050f分支codex/plan-021-dev+只读依赖auto-lang@b1a4871aa/auto-web@ec9c4ba（detached clean；pilot `../../../auto-lang`与auto-ui-compiler/dialect的`../../../auto-web`组内相对解析实测必要）。SDK重绑：driver tools/prepare_sdk.py --from-snapshot=**PASS pinned SDK: 681 files**（与020同一681内容锁，未升级）。pilot `cargo check --locked` PASS 51.3s（**锁字节零漂移**——020 landed锁已匹配本pin，无rebind提交）；基线门9/9 basic+10/10 g11+9/9 lower+6/6 components全绿；driver node_components_flow预编译exit 0。G05源SHA256三件逐一相符Plan§4原值（ag-examples@72de198 examples/g05-queries/g05-2d/{segment,sweep,variants/segment-changed}.at）。owner核：AG008 archived+cleaned（ag-examples 8a01a34交付）、UI022无树未启动、UI023在飞ui-023树（独立树无并发写；共享接线靠merge顺序，不设同树窗口）、AG009/AG010=硬等本计划的消费者；主检出WIP仅他会话docs簿记+根Cargo.lock 1行a2r-std（020收据同记，零代码WIP）。scratch登记D:/autostack/plan-artifacts/plan-021/auto-ui/README.md。T-02前置事实（三Explore代理+亲核）：objects2d/1装配已含v3+v4+spatial+motion+character+camera+objects全单元（assembly.rs query_api/motion_api），entry每callback `_ag_cast_token=0`+`_ag_spatial_begin()`；slot对象渲染专用不入spatial世界；行序char=1→solids→根静态→逐实例（collect_scene next_id从1）；character/solid行被char服务快照守护（非char移动fault 20/21）；现网无任何components产物调用query API | AC-01 |
| T-02 | work | T-01 | 最多两接法三条完整链/费用/时序先证，Plan§9冻结typed服务/CLI/九类与V-S；不足回规划 | AC-01..06 | [x] 2026-11-01：真机GO（9/9探针全绿，零外仓/零driver api/runtime/assembly改动，业务全在将生成的Auto）。两接法=（1）根作用域：root on Life.Tick内query2d调用+typed结果+具名行refs映射标记；（2）实例作用域：同def两Querier实例各自self.row()/self.world()发起segment并读各自model（声明序A先B后）。三链实测：segment链=runner(6,2)→+reach20 ray、CPU独立f32 slab oracle逐checkpoint吻合（起点t=0.5近命中target、行走进inside t0/n0、过target后far t按reach、miss后door隐藏）、GPU真实像素marker位移(+100,+16px)与door出现；sweep链=薄墙(20,2)1x8、speed100请求位移、显式clearance 0.02、CPU+GPU收敛停于17.48=wall_lo−half−clearance（后续tick保持零穿入）、away全量位移−100、起始overlap含dx=0→t0/start_overlap1/n0阻挡、origin快照写后重读不变；changed敏感性=reach 20→10源变（scene不变）→起点t 0.5→1.0（闭区间切触）+中途far t 0.3→0.6、CPU marker行与GPU像素均真实差异（非仅source hash）。费用冻结：segment链13/tick（cast8+5getter）、sweep链23/tick（16+6getter+origin重读）、cross两实例22/tick（2×(8+3getter)）、共享32池第33计费=具名sticky fault 10在超额调用处实测；cross-callback失效由既有entry保证（_ag_spatial_begin+_{_ag_cast_token=0}每callback）。时序：segment链CPU tick墙钟avg 1981us/max 2809us（120tick含getter读回，8000us宿主帽独立报告）。运行时控制钉死：第二次query使旧token失效→重读具名13；cast token上sweep getter具名13（kind guard）；零solid app_start（ag_char_bind+camera）绿。解析面：lab作者源在现行dialect/loader全parse+closure成功（零parser改动），lowering当前对query2d.*以module/line/symbol具名拒绝（T-03前的正确形态）。T-02关键发现：auto词法含连字符标识符（`t-r_clearance`并一词）→生成源二元减号必须带空格（原G05即如此）；spatial descriptor行=局部半 extents[1,−hx,−hy,0,+hx,+hy,0]（worlds+cb），020 flow harness灌的绝对lo/hi仅因当时无query路径而未暴露；camera view=bounds内clamp半视口居中（本labs视口x[0,40]y[−4,8]，20px/world，sRGB画布）；character行恒id1/ord0，其后实例静态行按声明先序；spawned slot对象不入spatial世界（查询面=场景文档行）；非char移动character/solid行=运行时fault 21/20（lowering将对具名行拒绝这些写）。探针材料：D:/autostack/plan-artifacts/plan-021/auto-ui/t02/（labs作者源+notes）；瞬态tests/t02_probe.rs（driver）+tests/t02_parse_probe.rs（pilot）未提交，T-03产品化后删除。命令：pilot `cargo test --locked --test t02_parse_probe`、driver `cargo test --locked --offline --test t02_probe -- --test-threads=1`均exit 0亲跑 | AC-01..06 |
| T-03 | work | T-02 go | 复用020组件IR，新增query服务/引用/typed结果与诊断，V-01 | AC-02 | [x] 2026-11-01：提交642cb2d。translate.rs增Ctx.query=Option<QueryEnv>（旧档None字节不变）：query2d.segment(6)/sweep(4)→原ag_cast_segment(2,…)/ag_sweep_aabb(…,0.0)、typed结果let-only绑定（RefCell栈，translate_handler在handler边界清理）、cast5/sweep6 getter机械映射（错kind具名拒绝）、refs.place/shift/show/world/local与self.row()族（place=语句级双ag_set_local）、input.space/x/y→transport ag_input_*、裸refs.<inst>值=行ordinal。lower_components.rs：plan_dispatch(+queries)模拟绑定序并具名拒绝（未知引用/错kind/未绑定getter/重绑/var绑定/元数/visible_only非字面量/sweep mover -1/self.row()在root/character行位置写——全部module/line/symbol）；plan期refs_names表（顶层实例∧(静态行∨char)）与collect_scene行序同规则；SceneBuild增refs+self_rows；per-def {prefix}_qrow数组（self行SoA寻址，如querier_qrow=[3,4]）；gen_recv_fn线程query env。main.rs/lib.rs：--profile queries-2d、identity generator="…plan021 queries-2d"、provenance consumer注册。词法发现：-1源码=Unary(Sub,Int(1))（handle识别覆盖）。诊断实测（CLI亲跑）：未知引用/错kind/未绑定/character行写/sweep mover -1/var绑定/components档query2d全部module/line/symbol具名。夹具fixtures/queries2d（segment/sweep/crosslab/changed+9件共享模块）与冻结产物generated-queries2d/{q-segment 52a37119,q-sweep f7dc9f19,q-cross eda9e765,q-changed dac39f5b}（两轮再生字节等值）。门=tests/node_queries_gates 6/6（形状/字节稳定/qrow隔离/changed仅reach差/8负控具名/components档拒绝/sweep形状）；legacy门9+10+9+6全绿字节冻结 | AC-02 |
| T-04 | work | T-03 | 真实Auto segment/sweep、变换/绑定时机、marker/GPU与独立数值，V-02/03 | AC-03 | [x] 2026-11-01：提交8a73906。tests/node_queries_flow 6/6单线程（冻结产物q-*为唯一输入，include_str+sha链抽查）：V-02 segment链CPU独立f32 slab oracle 120tick逐checkpoint吻合+恰13 credits/tick（墙钟avg 1853us/max 2882us<8000us独立报告）+GPU marker位移(+100,+16px)/door出现+changed(q-changed)真实差异（起点t 0.5→1.0闭区间切触、中途0.3→0.6、CPU行+GPU像素均异）；V-03 sweep链CPU零位移miss/逼近收敛17.48=wall_lo−half−clearance/away全量/起始overlap含dx=0→t0+start_overlap1+n0/origin快照写后重读不变+恰23 credits/tick+GPU可见clearance停（runner[17.48]wall[20.0]）；声明绑定时机=每tick末lifetick内同步写行（源序不重排，T-02冻结合同）；V-04早期证据q-cross两同def实例声明序各自query+model隔离+22 credits/tick。旧消费者node_components_flow 9/9绿；瞬态t02探针删除（被冻结产物门取代） | AC-03 |
| T-05 | work | T-04 | latest-slot/错kind/失败/旧代/owner20轮，九类义务与原额/守护负控，V-04/05 | AC-04/05 | [x] 2026-11-01：提交6d3ca3f。q-cross-plus产物（第三Querier qc@x=2→3 cast+9 getter=33计费）→恰第33计费具名sticky fault 10（v05_budget_plus门）；runtime控制（冻结场景行+最小变体world，probe_session_rows喂入）：同callback第二query使旧token失效→重读具名13、cast token上sweep getter具名13；GPU生命周期门：render-only settle零推进、每Step恰一tick、暂停帧零移动、20轮Stop/fresh每轮重建（runner回(6,2)/门重隐/链重跑绿）；changed身份断言（world sha异+场景几何同，Q8+AG009 Q10一并）；VM守护原额：max_call_steps 3418<100000、max_call_slices 4≤120（120tick独立报告）。九类义务映射逐项落§9冻结核（①⑦⑧⑨直接门证据，②⑥运行时等价观测，③checkpoint记录，④⑥消费合同，OS阶段具名pending按D-03） | AC-04/05 |
| T-06 | work | T-05 | 公共消费/SaveAs/A→B/损坏恢复/旧消费者、SD与移交折Plan，V-06 | AC-01..06 | [x] 2026-11-01：提交d455652。tools/verify_plan021.py --controls=**17控制组/5产品全PASS**：每产品身份链（generator=…plan021 queries-2d/api/closure清单==夹具/provenance consumer注册）+仅凭closure副本冷A→B字节等值+changed-variant身份控制（同几何不同源绑定）+runner位置变异敏感性（scene/identity漂移+模块sha捕获、world.at稳定=scene-only变异的预期形态）+8条负控CLI具名复放+components档边界+坏件消费（python重放+driver实演）；driver v06_damaged_queries_world_copy_rejected_then_recovery_green：完好载入跑绿→单字节损坏载入即拒（'behavior source closure identity: <scripts/<sha>.at>'具名绑sha）→精确还原→真Run输入推进；旧消费者：verify_plan020 --controls 18组8产品PASS（字节冻结）、node_components_flow 9/9；cargo fmt clean；五产品sha复验不变（q-segment 52a37119）。SD-01候选文本落scratch sd-01-candidate.md（docs/specs/auto-graphics/node2d-queries.md于merge落位）。node_queries_flow终态11/11单线程 | AC-01..06 |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD/原分母与遗漏，finding回work，pass列merge门pending | AC-07 | [x] 2026-11-01：同树独立final两轮——R1 needs_fix（F-R1-01..04：P2多静态shape refs静默接受+3×P3）全门禁亲跑（pilot 9+10+9+6+6/flow 11/11/verify021/verify020/components_flow 9/9/6条CLI负控）；R1修复02e9c86后同评审者受影响复核**pass**（反例复现已具名拒绝、components档不受累、五产品sha复验）；F-R2-01..03文本项由R2闭环（9ecebae gates q04 10案+q04b=7/7、SD候选计数20组/11负控） |
| T-08 | merge | T-07 pass | 冻结源/依赖/输入，定向门或有效复用后合master | AC-08 | [x] 2026-11-01：冻结源/依赖/输入核验+range-diff 7/7全等安全重写（master并发plan-023 docs），rebased tip抽核gates 7/7+sha不变，git merge --ff-only合master tip=1bec52f无merge commit；SD-01于1bec52f逐字落位（020先例） |
| T-09 | merge | T-08 | 实际Spec/来源/归档导航及证据折Plan，不写外仓Spec/派生JSON | AC-08/09 | [x] 2026-11-01：canonical spec docs/specs/auto-graphics/node2d-queries.md sha 0d930490…@1bec52f核验；来源导航=SD §6/§8五产品sha+provenance consumer注册；证据折Plan§8/§9；未写外仓Spec/派生JSON |
| T-10 | merge | T-09 | fresh guard/clean/祖先核后清树分支空组，输出对账与自有材料清理/登记 | AC-09 | [x] 2026-11-01：三树fresh guard clean×3→worktree remove×3+prune、branch -d（was 1bec52f已landed）、空组ui-021删除、worktree list余2项；plan-artifacts/plan-021整删（delete-at-merge，labs/SD已入仓）；registry policy.json增PLAN-021 cleaned行（无holds）；输出对账=五冻结产品在仓generated-queries2d/（sha见SD §6）、canonical spec在docs/specs/、无外仓/无生产二进制交付面；主检出遗留WIP均他会话簿记 | AC-09 |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-021 / plan_revision: 1 / outcome: blocked（实施硬前置020尚未实际交付，不阻本轮规划） / next: 020实际交付并获启动后T-01/T-02。仅合同/设计/导航/取号；未创建实现/夹具/树/agent，未构建/运行GPU或修改G05内容；所有拟建命令尚未执行。

2026-11-01 / stage: goal / plan_id: auto-ui/PLAN-021 / plan_revision: r1 / round: 1 / from: blocked(new) | to: work | status: executing | outcome: pass（T-01/T-02完成） / worktree: D:/autostack/.wt/ui-021/auto-ui (+只读siblings auto-lang@b1a4871aa/auto-web@ec9c4ba) | commit: 无（基3dd050f；T-01/T-02为核验+探针，零产品代码提交） / next: work(T-03)。

**T-02冻结合同（queries-2d，2026-11-01，探针实证）：**

- CLI/身份：pilot新增显式profile `queries-2d`（`node2d-lower --profile queries-2d --entry <dir>/main.at --out <dir>`）；`components-2d` 对query2d构造具名拒绝；identity `generator="autographics-node2d-pilot plan021 queries-2d"`、`api="autographics.objects2d/1"`不变（零runtime/assembly改动）。
- 查询调用（仅queries-2d；handler体内/视图prop外）：`query2d.segment(ignore, visible_only, ax, ay, bx, by)`（2D固定，z=0隐含；ignore=具名行引用|self.row()|-1字面量）与 `query2d.sweep(mover, visible_only, dx, dy)`（mover=具名行引用|self.row()，无-1）。lowering→原`ag_cast_segment(2,…)`/`ag_sweep_aabb(…,dx,dy,0.0)`。
- typed结果：仅`let q = <query调用>`绑定；q按调用种类typed为Cast/Sweep结果，getter仅限结果变量方法调用：cast=`.hit()/.node()/.t()/.point(axis)/.normal(axis)`，sweep=`.hit()/.node()/.t()/.origin(axis)/.normal(axis)/.start_overlap()`；错kind/未绑定/再赋值/算术使用=lowering具名拒绝（运行时13仍为backstop）。q即token（运行时int，同一世界共享latest slot，第二query/失败attempt/callback起止失效——原语义，零新运行时）。
- 具名typed引用：`refs.<顶层实例名>`值=该实例primary静态行的行ordinal（int）；实例须恰一静态shape，动态only/零/多=具名诊断；`self.row()`=本实例primary静态行。行op（queries-2d新增）：`shift(dx,dy)`→ag_move_local(h,dx,dy,0.0)、`place(x,y)`→ag_set_local×2、`show(flag)`→ag_set_visible、`world(axis)`/`local(axis)`→ag_world/ag_local；refs与self两形同权。解析到character行/solid行的写op（shift/place）=lowering具名拒绝（运行时fault 21/20 backstop）；show/world/local对character行只读合法。嵌套子实例refs、refs整树遍历=本切片不承诺（具名不支持）。019既有refs.<x>.move_to/move_and_slide/grounded/support语义不变。
- 输入：`input.space()/input.x()/input.y()`→ag_input_space()/ag_input_x()/ag_input_y()（transport `_ag_action(space,x,y,toggle)`喂入；space==2=2D游戏面）。不用character action位（不误作Jump键、不强启GameInput）。
- 生成源形态（T-03/T-04按此emit）：root模型`r_<field>`标量+`main_lifetick(dt)`；实例SoA沿020；引用行ordinal为字面量（lowering确定性解析）；**二元减号一律带空格**（词法连字符标识符）；`*1.0`语句值约定沿020；静态行无present()；app_start=ag_char_bind(1)*1.0+ag_camera_follow_2d(1,rect,view)*1.0（零solid合法）。
- 场景行序（引用解析依据）：character行id1/ord0恒首→(无solid时)逐实例静态行按声明先序id递增；query几何=场景文档行；slot对象不入spatial世界；>128行具名拒绝沿020。
- 费用/守护：cast8/sweep16/getter1、共享32/callback（第33=sticky fault 10在超额调用处）；cross-callback由entry `_ag_spatial_begin`+`_ag_cast_token=0`失效；VM/时限包络沿旧profile；实测时序见§8 T-02行（avg 1981us/max 2809us<8000us）。
- G05九类义务映射（V-05/T-05落产品门）：①segment成功=新链全绿+oracle吻合（CPU+GPU）；②/⑥14phase=OS探针阶段结构不属本链，按D-03映射为运行时等价观测（run/pause无事件/一中性step/推进/stop还原/新Run重建——020 V-04 harness在query lab上重放），OS键盘/焦点/OS画布还原阶段具名pending（OS自动化暂停，不伪称）；③非空trace=query结果hit/node/t/point/normal经getter+独立oracle逐checkpoint记录（产品gate写trace记录）；④saveas导出=019/020消费合同SaveAs/重开（T-06）；⑤sweep成功同①；⑦changed合法=reach10变体全链绿；⑧changed source真变=identity/source hash不同**且**实际结果/像素不同（Q8+AG009新增Q10一并覆盖）；⑨partial拒绝=sticky具名负控（fault 10/13、坏闭包拒绝T-06），不把旧旗标空退出当通过。
- V-S命令冻结：pilot `cargo test --locked --test node_queries_gates`；driver `cargo test --locked --offline --test node_queries_flow -- --test-threads=1`；verifier `python tools/verify_plan021.py --controls`（T-03..06建成；现探针为瞬态t02_*未提交）。

2026-11-01 / stage: work | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass（T-01..T-06全work任务完成，execution_ready） | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | base_commit: 3dd050f | code_commit: 642cb2d(T-03)+8a73906(T-04)+6d3ca3f(T-05)+d455652(T-06，含fmt与q-cross-plus provenance注册) | dependency_revisions: auto-lang b1a4871aa、auto-web ec9c4ba（detached只读clean）、SDK .sdk-source 681（prepare PASS，未升级） | task_ids: T-01..T-06 | evidence: §8各任务行+scratch D:/autostack/plan-artifacts/plan-021/auto-ui/（README登记/labs作者源/sd-01-candidate.md/t02补丁脚本）；门禁亲跑计数=pilot gates 9+10+9+6+6、driver node_queries_flow 11/11单线程（真GPU）、node_components_flow 9/9、verify_plan021 --controls 17组5产品、verify_plan020 --controls 18组8产品、verify_plan016/017/019字节冻结产物未动 | blockers: 无 | next_responsible_role: review（T-07同树独立final）。AC-01..06全关：AC-01基线/三链/原费用先证（T-01/T-02）、AC-02 V-01（T-03）、AC-03 V-02/03（T-04）、AC-04/05 V-04/05（T-05）、AC-06 V-06（T-06）。评审待办：AC-07独立复审；merge门（冻结定向门、Spec沉淀、归档/清理）pending未声称。授权沿/goal整链（work/review/有界修复/merge），D-01..04已闭合（D-01=020交付基面已核、D-02=T-02冻结合同、D-03=OS阶段具名pending映射、D-04=本review轮同树独立上下文fresh子代理非人工，019/020同例）。瞬态t02探针已全部删除（冻结产物门取代）。

2026-11-01 / stage: review | review_scope: final | reviewer_identity: fresh独立子代理上下文（同宿主会话家族派生，非人工；全部门禁亲跑） | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | reviewed_commit: d455652 | base_commit: 3dd050f | dependency_revisions: auto-lang b1a4871aa/auto-web ec9c4ba只读clean；SDK verify-only PASS 681 | spec_inputs: SD-01候选（scratch sd-01-candidate.md） | acceptance_results: AC-01..06全部pass（独立复核：pilot 9+10+9+6+6、flow 11/11单线程真GPU、verify021 17组5产品、verify020 18组8产品、components_flow 9/9、6条CLI负控亲跑；字节冻结/差异范围/无ledger写入全核）；AC-07=本轮4 findings未清 | pending_closeout_gates: merge冻结定向门、Spec沉淀、归档/清理、批量回归 | findings: F-R1-01(P2,§9冻结合同违规,多静态shape实例refs静默接受而合同承诺具名诊断——附加shape无声入世界；修复=plan期具名拒绝+gates/verifier负控各+1；零冻结产物受累)、F-R1-02(P3,driver node_queries_flow.rs未fmt-clean 31 hunks)、F-R1-03(P3,再赋值诊断line 0无module/symbol+rebind/再赋值未入双负控集)、F-R1-04(P3,SD-01 §0 V-01覆盖表述过宽+§5再赋值措辞+§3点值墙钟入spec) | 另记（非本Plan回归，merge门注意）：driver authoring_feasibility测试目标在--tests下编译失败（E0063 hud_overlay，Plan-879时代遗留，021未触碰；021全部定向--test命令不受影响） | evidence: 评审者自跑命令/数字见其报告（AC映射含6条CLI负控原文） | next: work修复F-R1-01..04（有界，无需replan）

2026-11-01 / stage: work | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass（R1修复F-R1-01..04） | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | base_commit: 3dd050f | code_commit: 02e9c86(R1修复) | dependency_revisions: 不变 | task_ids: T-03/T-04/T-06受影响面 | evidence: F-R1-01=plan期refs表构建具名拒绝（instance X has N static shapes; a named row ref resolves exactly one…，queries档门控components不受累）+gates q04(10负控)/verifier(multi-static-shape)双复放；F-R1-03=再赋值升plan层QEvent::ResultAssign（module/line/symbol具名）+rebind/reassign双负控集（gates q04 + verifier result-rebind/result-reassign）；F-R1-02=driver node_queries_flow.rs cargo fmt clean（仅测试字节）；F-R1-04=SD-01候选§0覆盖表述收敛/§5再赋值入module/line/symbol清单+多静态shape补录/§3墙钟点值移收据 | 每轮受影响门禁亲跑全绿：verify_plan021 --controls **20控制组/5产品**PASS（17+3新负控）、pilot gates 9+10+9+6+6、node_queries_flow 11/11单线程、两crate fmt --check clean、五产品sha不变（q-segment 52a37119复验） | blockers: 无 | next_responsible_role: review（同评审者受影响复核R1）

2026-11-01 / stage: review | review_scope: final(受影响复核R1) | reviewer_identity: 同一fresh独立子代理上下文（非人工） | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | reviewed_commit: 02e9c86（R1修复；后继9ecebae=R2文本/复放补全，见下） | base_commit: 3dd050f | dependency_revisions: 不变 | spec_inputs: SD-01候选（R2文本修正后终稿） | acceptance_results: AC-01..06 pass（沿R1评审证据+受影响复核：F-R1-01反例独立复现已具名拒绝且components档exit 0、verify_plan020 18组复跑PASS、pilot gates全绿、五产品sha逐一复验不变）、AC-07 pass（F-R1-01..04关闭独立验证；F-R2-01..03为P3文本项——R2已在9ecebae+sandbox收据内闭环：gates q04扩至10案+新q04b多静态shape具名拒绝（7/7绿）、SD候选§6/§8计数20组/11负控、§3墙钟点值移收据、§0 
残迹移除）；AC-08/09 merge pending | pending_closeout_gates: merge冻结定向门或严格有效复用、SD-01落docs/specs/、归档导航、树清理与plan-artifacts删除、批量回归due（非本Plan门；评审者记录driver authoring_feasibility在--tests下编译失败为Plan-879时代遗留，非021回归） | findings: 无未处置 | evidence: 评审者自跑命令/数字见其两轮报告 | next: merge。交付tip=9ecebae

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-021:r1 / outcome: pass / prepared: reviewed基线9ecebae+SD-01 spec于worktree提交31a49fb（docs-only后裔，实现/依赖零变化）；wt-guard clean / rebase: master前进（aa47313+b8e51c5+f220d72并发plan-023 docs，不在依赖图）；rebase后旧→新映射 642cb2d→68e81c5、8a73906→78ad138、6d3ca3f→557758a、d455652→a5eeea1、02e9c86→fecda32、9ecebae→6b11543、31a49fb→1bec52f，git range-diff 7/7全等0不等（安全重写证明）；rebased tip抽核 queries gates 7/7+q-segment sha 52a37119不变 / landed: 主检出 git merge --ff-only codex/plan-021-dev → master tip=1bec52f（无merge commit） / canonical_verified: docs/specs/auto-graphics/node2d-queries.md sha256 0d930490f45d9a1297443eb867718976a8d940d0812bace278a8701e383cf20b（10136字节，T-07评审绑定候选逐字落位，020同例）@ 1bec52f / archived: 本文件移docs/plans/archived/ status:archived（提交见git log；registry 021行同步archived） / cleaned: 三树fresh guard全clean（wt-guard clean×3）后git worktree remove×3（auto-ui自仓注销；auto-lang@b1a4871aa/auto-web@ec9c4ba各自repo注销+prune）+branch -d codex/plan-021-dev(was 1bec52f已landed)+空组ui-021 rmdir+worktree list余2项（主检出+ui-023他会话）；plan-artifacts/plan-021全删（labs作者源已入仓fixtures/queries2d、SD候选已landed为canonical spec、t02补丁脚本瞬态随删；sd-01-candidate/README摘要折入本文件§8/§9，delete-at-merge 2026-10-31）；registry .retention/policy.json 增auto-ui/PLAN-021行（cleaned，无holds）；主检出遗留非本Plan WIP=根Cargo.lock 1行(a2r-std getrandom,他会话)+attachments/012+docs/design/autoui+plans 007(他会话)，保留不接管 / 交付后观察: 本仓为库/研究仓无驻留生产二进制交付面（017/019/020同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang），021不适用；评审者记录driver authoring_feasibility在--tests下编译失败（Plan-879时代遗留E0063 hud_overlay，非021触碰）留作批量回归观察项 / 10任务10/10完成，AC-01..09全关，交付commit 1bec52f

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: 无（主检出；ui-021三树/分支codex/plan-021-dev/组/plan-artifacts-021已清零，worktree list仅主检出+ui-023他会话） | commit: c65a018（goal收尾；交付链=1bec52f交付+spec → a3e83c8归档+registry → c65a018清理回执） | next: done / Done五项核对：①交付1bec52f在master祖先链、--ff-only无merge commit（merge-base实证）②canonical spec docs/specs/auto-graphics/node2d-queries.md sha 0d930490…@1bec52f在案（T-07评审绑定候选逐字落位）③归档文件status:archived已提交（a3e83c8/c65a018），registry 021行archived④树/分支/组/plan-artifacts-021清零实证（worktree list 0项ui-021、branch 0、组目录/scope不存在）⑤主检出遗留WIP（Cargo.lock 1行a2r-std/attachments/012/007与autoui设计稿）均他会话簿记，本Plan零代码WIP；landing check=本仓库/研究仓无生产二进制交付面（017/019/020同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang）不适用；遗留观察（非本Plan门）：driver authoring_feasibility --tests编译失败（Plan-879时代E0063）留批量回归观察，AG009按其合同硬等本交付并自行消费

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

2026-11-01 / stage: work | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass（T-01..T-06全work任务完成，execution_ready） | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | base_commit: 3dd050f | code_commit: 642cb2d(T-03)+8a73906(T-04)+6d3ca3f(T-05)+d455652(T-06，含fmt与q-cross-plus provenance注册) | dependency_revisions: auto-lang b1a4871aa、auto-web ec9c4ba（detached只读clean）、SDK .sdk-source 681（prepare PASS，未升级） | task_ids: T-01..T-06 | evidence: §8各任务行+scratch D:/autostack/plan-artifacts/plan-021/auto-ui/（README登记/labs作者源/sd-01-candidate.md/t02补丁脚本）；门禁亲跑计数=pilot gates 9+10+9+6+6、driver node_queries_flow 11/11单线程（真GPU）、node_components_flow 9/9、verify_plan021 --controls 17组5产品、verify_plan020 --controls 18组8产品、verify_plan016/017/019字节冻结产物未动 | blockers: 无 | next_responsible_role: review（T-07同树独立final）。AC-01..06全关：AC-01基线/三链/原费用先证（T-01/T-02）、AC-02 V-01（T-03）、AC-03 V-02/03（T-04）、AC-04/05 V-04/05（T-05）、AC-06 V-06（T-06）。评审待办：AC-07独立复审；merge门（冻结定向门、Spec沉淀、归档/清理）pending未声称。授权沿/goal整链（work/review/有界修复/merge），D-01..04已闭合（D-01=020交付基面已核、D-02=T-02冻结合同、D-03=OS阶段具名pending映射、D-04=本review轮同树独立上下文fresh子代理非人工，019/020同例）。瞬态t02探针已全部删除（冻结产物门取代）。

2026-11-01 / stage: review | review_scope: final | reviewer_identity: fresh独立子代理上下文（同宿主会话家族派生，非人工；全部门禁亲跑） | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | reviewed_commit: d455652 | base_commit: 3dd050f | dependency_revisions: auto-lang b1a4871aa/auto-web ec9c4ba只读clean；SDK verify-only PASS 681 | spec_inputs: SD-01候选（scratch sd-01-candidate.md） | acceptance_results: AC-01..06全部pass（独立复核：pilot 9+10+9+6+6、flow 11/11单线程真GPU、verify021 17组5产品、verify020 18组8产品、components_flow 9/9、6条CLI负控亲跑；字节冻结/差异范围/无ledger写入全核）；AC-07=本轮4 findings未清 | pending_closeout_gates: merge冻结定向门、Spec沉淀、归档/清理、批量回归 | findings: F-R1-01(P2,§9冻结合同违规,多静态shape实例refs静默接受而合同承诺具名诊断——附加shape无声入世界；修复=plan期具名拒绝+gates/verifier负控各+1；零冻结产物受累)、F-R1-02(P3,driver node_queries_flow.rs未fmt-clean 31 hunks)、F-R1-03(P3,再赋值诊断line 0无module/symbol+rebind/再赋值未入双负控集)、F-R1-04(P3,SD-01 §0 V-01覆盖表述过宽+§5再赋值措辞+§3点值墙钟入spec) | 另记（非本Plan回归，merge门注意）：driver authoring_feasibility测试目标在--tests下编译失败（E0063 hud_overlay，Plan-879时代遗留，021未触碰；021全部定向--test命令不受影响） | evidence: 评审者自跑命令/数字见其报告（AC映射含6条CLI负控原文） | next: work修复F-R1-01..04（有界，无需replan）

2026-11-01 / stage: work | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass（R1修复F-R1-01..04） | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | base_commit: 3dd050f | code_commit: 02e9c86(R1修复) | dependency_revisions: 不变 | task_ids: T-03/T-04/T-06受影响面 | evidence: F-R1-01=plan期refs表构建具名拒绝（instance X has N static shapes; a named row ref resolves exactly one…，queries档门控components不受累）+gates q04(10负控)/verifier(multi-static-shape)双复放；F-R1-03=再赋值升plan层QEvent::ResultAssign（module/line/symbol具名）+rebind/reassign双负控集（gates q04 + verifier result-rebind/result-reassign）；F-R1-02=driver node_queries_flow.rs cargo fmt clean（仅测试字节）；F-R1-04=SD-01候选§0覆盖表述收敛/§5再赋值入module/line/symbol清单+多静态shape补录/§3墙钟点值移收据 | 每轮受影响门禁亲跑全绿：verify_plan021 --controls **20控制组/5产品**PASS（17+3新负控）、pilot gates 9+10+9+6+6、node_queries_flow 11/11单线程、两crate fmt --check clean、五产品sha不变（q-segment 52a37119复验） | blockers: 无 | next_responsible_role: review（同评审者受影响复核R1）

2026-11-01 / stage: review | review_scope: final(受影响复核R1) | reviewer_identity: 同一fresh独立子代理上下文（非人工） | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-021/auto-ui | branch: codex/plan-021-dev | reviewed_commit: 02e9c86（R1修复；后继9ecebae=R2文本/复放补全，见下） | base_commit: 3dd050f | dependency_revisions: 不变 | spec_inputs: SD-01候选（R2文本修正后终稿） | acceptance_results: AC-01..06 pass（沿R1评审证据+受影响复核：F-R1-01反例独立复现已具名拒绝且components档exit 0、verify_plan020 18组复跑PASS、pilot gates全绿、五产品sha逐一复验不变）、AC-07 pass（F-R1-01..04关闭独立验证；F-R2-01..03为P3文本项——R2已在9ecebae+sandbox收据内闭环：gates q04扩至10案+新q04b多静态shape具名拒绝（7/7绿）、SD候选§6/§8计数20组/11负控、§3墙钟点值移收据、§0 
残迹移除）；AC-08/09 merge pending | pending_closeout_gates: merge冻结定向门或严格有效复用、SD-01落docs/specs/、归档导航、树清理与plan-artifacts删除、批量回归due（非本Plan门；评审者记录driver authoring_feasibility在--tests下编译失败为Plan-879时代遗留，非021回归） | findings: 无未处置 | evidence: 评审者自跑命令/数字见其两轮报告 | next: merge。交付tip=9ecebae

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-021:r1 / outcome: pass / prepared: reviewed基线9ecebae+SD-01 spec于worktree提交31a49fb（docs-only后裔，实现/依赖零变化）；wt-guard clean / rebase: master前进（aa47313+b8e51c5+f220d72并发plan-023 docs，不在依赖图）；rebase后旧→新映射 642cb2d→68e81c5、8a73906→78ad138、6d3ca3f→557758a、d455652→a5eeea1、02e9c86→fecda32、9ecebae→6b11543、31a49fb→1bec52f，git range-diff 7/7全等0不等（安全重写证明）；rebased tip抽核 queries gates 7/7+q-segment sha 52a37119不变 / landed: 主检出 git merge --ff-only codex/plan-021-dev → master tip=1bec52f（无merge commit） / canonical_verified: docs/specs/auto-graphics/node2d-queries.md sha256 0d930490f45d9a1297443eb867718976a8d940d0812bace278a8701e383cf20b（10136字节，T-07评审绑定候选逐字落位，020同例）@ 1bec52f / archived: 本文件移docs/plans/archived/ status:archived（提交见git log；registry 021行同步archived） / cleaned: 三树fresh guard全clean（wt-guard clean×3）后git worktree remove×3（auto-ui自仓注销；auto-lang@b1a4871aa/auto-web@ec9c4ba各自repo注销+prune）+branch -d codex/plan-021-dev(was 1bec52f已landed)+空组ui-021 rmdir+worktree list余2项（主检出+ui-023他会话）；plan-artifacts/plan-021全删（labs作者源已入仓fixtures/queries2d、SD候选已landed为canonical spec、t02补丁脚本瞬态随删；sd-01-candidate/README摘要折入本文件§8/§9，delete-at-merge 2026-10-31）；registry .retention/policy.json 增auto-ui/PLAN-021行（cleaned，无holds）；主检出遗留非本Plan WIP=根Cargo.lock 1行(a2r-std getrandom,他会话)+attachments/012+docs/design/autoui+plans 007(他会话)，保留不接管 / 交付后观察: 本仓为库/研究仓无驻留生产二进制交付面（017/019/020同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang），021不适用；评审者记录driver authoring_feasibility在--tests下编译失败（Plan-879时代遗留E0063 hud_overlay，非021触碰）留作批量回归观察项 / 10任务10/10完成，AC-01..09全关，交付commit 1bec52f

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-021 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: 无（主检出；ui-021三树/分支codex/plan-021-dev/组/plan-artifacts-021已清零，worktree list仅主检出+ui-023他会话） | commit: c65a018（goal收尾；交付链=1bec52f交付+spec → a3e83c8归档+registry → c65a018清理回执） | next: done / Done五项核对：①交付1bec52f在master祖先链、--ff-only无merge commit（merge-base实证）②canonical spec docs/specs/auto-graphics/node2d-queries.md sha 0d930490…@1bec52f在案（T-07评审绑定候选逐字落位）③归档文件status:archived已提交（a3e83c8/c65a018），registry 021行archived④树/分支/组/plan-artifacts-021清零实证（worktree list 0项ui-021、branch 0、组目录/scope不存在）⑤主检出遗留WIP（Cargo.lock 1行a2r-std/attachments/012/007与autoui设计稿）均他会话簿记，本Plan零代码WIP；landing check=本仓库/研究仓无生产二进制交付面（017/019/020同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang）不适用；遗留观察（非本Plan门）：driver authoring_feasibility --tests编译失败（Plan-879时代E0063）留批量回归观察，AG009按其合同硬等本交付并自行消费

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 020共用IR/事件/时钟与新消费实际接口 | 等020真实delivery，T-01重绑，不消费开发tip |
| D-02 | typed结果/空间输入/变换绑定可见时机及费用 | T-02按旧v3/v4语义整链冻结；不能每Node私有结果池或免费缓存 |
| D-03 | 原9类/14阶段中OS证据适用性与源级oracle落点 | T-02逐项映射，必需缺口保持pending，不静默恢复OS自动化 |
| D-04 | 022/018共享接线及独立review/GPU窗口 | T-01/T-06按真实owner与授权安排，无同树并发写 |

021不依赖022或AG008完成；共享接线不能隔离时顺序落地。
