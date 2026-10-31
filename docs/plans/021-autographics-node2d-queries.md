---
plan_id: PLAN-021
status: drafting
feature_name: AutoGraphics Node二维segment与sweep查询作者能力
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 0
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
| T-01 | work | 后续启动、020实际交付 | 核Spec/SDK/依赖/owner，建唯一ui-021树；固定G05源与原义务 | AC-01 | [ ] |
| T-02 | work | T-01 | 最多两接法三条完整链/费用/时序先证，Plan§9冻结typed服务/CLI/九类与V-S；不足回规划 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 复用020组件IR，新增query服务/引用/typed结果与诊断，V-01 | AC-02 | [ ] |
| T-04 | work | T-03 | 真实Auto segment/sweep、变换/绑定时机、marker/GPU与独立数值，V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | latest-slot/错kind/失败/旧代/owner20轮，九类义务与原额/守护负控，V-04/05 | AC-04/05 | [ ] |
| T-06 | work | T-05 | 公共消费/SaveAs/A→B/损坏恢复/旧消费者、SD与移交折Plan，V-06 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD/原分母与遗漏，finding回work，pass列merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结源/依赖/输入，定向门或有效复用后合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 实际Spec/来源/归档导航及证据折Plan，不写外仓Spec/派生JSON | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard/clean/祖先核后清树分支空组，输出对账与自有材料清理/登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-021 / plan_revision: 1 / outcome: blocked（实施硬前置020尚未实际交付，不阻本轮规划） / next: 020实际交付并获启动后T-01/T-02。仅合同/设计/导航/取号；未创建实现/夹具/树/agent，未构建/运行GPU或修改G05内容；所有拟建命令尚未执行。

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 020共用IR/事件/时钟与新消费实际接口 | 等020真实delivery，T-01重绑，不消费开发tip |
| D-02 | typed结果/空间输入/变换绑定可见时机及费用 | T-02按旧v3/v4语义整链冻结；不能每Node私有结果池或免费缓存 |
| D-03 | 原9类/14阶段中OS证据适用性与源级oracle落点 | T-02逐项映射，必需缺口保持pending，不静默恢复OS自动化 |
| D-04 | 022/018共享接线及独立review/GPU窗口 | T-01/T-06按真实owner与授权安排，无同树并发写 |

021不依赖022或AG008完成；共享接线不能隔离时顺序落地。
