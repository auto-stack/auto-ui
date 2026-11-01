---
plan_id: PLAN-020
status: archived
feature_name: AutoGraphics有状态Node组件与局部事件组合
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 10

2026-11-01 / stage: work / plan_id: auto-ui/PLAN-020 / plan_revision: r1 / outcome: pass(T-01..T-06全work任务完成，execution_ready) / worktree: D:/autostack/.wt/ui-020/auto-ui / branch: codex/plan-020-dev / base_commit: 6ab7f2d / code_commit: e2e32ff(lock rebind)+e660798(T-03/04)+bbfa358(T-05)+c264d99(T-06，amend剔除误收瞬态探针) / dependency_revisions: auto-lang b1a4871aa、auto-web ec9c4ba（detached只读clean）、SDK .sdk-source 681（prepare PASS） / task_ids: T-01..T-06 / evidence: §8各任务行+t02探针scratch D:/autostack/plan-artifacts/plan-020/auto-ui/（README/README进度/sd-01-candidate.md/t02/）；门禁亲跑计数=pilot 9+10+9+5、driver node_components_flow 9/9单线程（真GPU）、旧消费者13/13+6/6+15/15、verify_plan020 --controls 16组8产品、瞬态t02_probe两测绿 / blockers: 无 / next_responsible_role: review（T-07同树独立final）。AC-01..06全关：AC-01基线/先证（T-01/T-02）、AC-02 V-01（T-03）、AC-03 V-02/03（T-04）、AC-04/05 V-04/05（T-05）、AC-06 V-06（T-06）。评审待办：AC-07独立复审；merge门（冻结定向门或严格有效复用、Spec沉淀、归档/清理）pending未声称。授权沿/goal整链（work/review/有界修复/merge），D-01..03已闭合（T-02冻结IR/端口/成本；owner核clean），D-04=本review轮同树独立上下文（fresh subagent非人工，同019 R1模式）。

2026-11-01 / stage: review | review_scope: final | reviewer_identity: fresh独立子代理上下文（同宿主会话家族派生，非人工；全部门禁亲跑） | plan_id: auto-ui/PLAN-020 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-020/auto-ui | branch: codex/plan-020-dev | reviewed_commit: 845d83f（c264d99的amend终态，仅差=剔除误收瞬态t02_probe.rs；§8收据中c264d99字样由此勘误为845d83f） | base_commit: 6ab7f2d | dependency_revisions: auto-lang b1a4871aa/auto-web ec9c4ba只读clean | spec_inputs: SD-01候选（scratch sd-01-candidate.md） | acceptance_results: AC-01..06全部pass（独立复核：33 pilot门禁/9 flow/16 verifier组/旧13+15/verify016-019全PASS亲跑；AC证据映射逐条见评审报告）；AC-07=本轮4 findings未清 | pending_closeout_gates: merge冻结定向门、Spec沉淀、归档/清理 | findings: F-R1-01(P2,AC-07,新增代码7条编译告警：lower_components.rs 5+node_components_flow.rs 2)、F-R1-02(P2,AC-02/06+SD-01,参数只读未强制且无负控——`.speed=3.0`降层绿并运行时可变；需具名拒绝+负夹具或SD措辞降级)、F-R1-03(P2,AC-06/SD-01,SD §0 GPU覆盖过宽——8产品仅probe/lifecycle/init-fault有GPU证据)、F-R1-04(P2,AC-05/V-05,8ms独立报告无交付物——需§9口径记录或具名merge持有项) | evidence: D:/autostack/plan-artifacts/plan-020/auto-ui/review-probe/ | next: work修复F-R1-01..04（有界，无需replan）

2026-11-01 / stage: work / plan_id: auto-ui/PLAN-020 / plan_revision: r1 / outcome: pass(R1..R3修复) / worktree: D:/autostack/.wt/ui-020/auto-ui / branch: codex/plan-020-dev / base_commit: 6ab7f2d / code_commit: a3bb092(R1修复F-R1-01..04)+85f4ea9(R2修复F-R2-01)+910d91d(R3修复F-R3-01) / dependency_revisions: 不变 / task_ids: T-03,T-04,T-05,T-06受影响面 / evidence: R1=7告警清零+参数只读具名拒绝+neg-param-write夹具+verifier 17组+SD-01 GPU/CPU事实化+8ms实测(avg 1761us/max 3129us vs 8000us cap)；R2=赋值目标检查移入walk_expr Bina臂（表达式嵌套`.param`写同拒绝），nested独立负夹具，verifier 18组；R3=守卫限Asn左值（RHS参数读合法+.positive pin gate 6/6）；每轮受影响门禁亲跑全绿，8产品world sha字节不变（probe 9241d013） / blockers: 无 / next_responsible_role: review（受影响复核）

2026-11-01 / stage: review | review_scope: final(受影响复核×3) | reviewer_identity: 同一fresh独立子代理上下文（非人工） | plan_id: auto-ui/PLAN-020 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-020/auto-ui | branch: codex/plan-020-dev | reviewed_commit: 910d91d | base_commit: 6ab7f2d | dependency_revisions: 不变 | spec_inputs: SD-01候选（经R1..R3措辞收敛：GPU/CPU证据split、参数只读=赋值目标具名拒绝/RHS读合法、8ms口径实测） | acceptance_results: AC-01..06 pass（沿R1评审证据+受影响复核）、AC-07 pass（F-R1-01..04/F-R2-01/F-R3-01全部独立验证关闭；评审者自证：bypass探针红/读探针绿/世界sha不变/门禁亲跑gates 6/6+verifier 18组+flow 9/9）；AC-08/09 merge pending | pending_closeout_gates: merge冻结定向门或严格有效复用、SD-01落docs/specs/、归档导航、树清理与plan-artifacts删除 | findings: 无（零未处置） | evidence: D:/autostack/plan-artifacts/plan-020/auto-ui/review-probe/（含r2-probe/持久探针） | next: merge。备注：交付tip=910d91d（§8内c264d99字样=amend前T-06 hash，amend仅剔瞬态探针，后续hash均为其干净后裔）

stage: goal | plan_id: auto-ui/PLAN-020 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-020/auto-ui | commit: 910d91d（e2e32ff+e660798+bbfa358+845d83f+a3bb092+85f4ea9+910d91d七提交） | next: merge
total_steps: 10
depends_on: [auto-ui/PLAN-019]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/stateful-node-components.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, auto-ui-compiler, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-020：有状态Node组件、局部绑定与父子事件

## 0. 变更摘要

补齐019基础二维中子Node只允许纯view的限制：普通use导入的组件真正持有model/msg/on，局部状态驱动其呈现，Life/业务事件绑定到正确实例；角色控制逻辑可以放回Player内部。形成后续[021查询](021-autographics-node2d-queries.md)、[022精灵](022-autographics-node2d-sprites-and-frame-animation.md)共享的有限组件合同。

本轮只规划drafting0/10，不建树/写实现/构建/GPU/agent。AG008固定消费019，不等待本计划，不改其在途合同；设计见[共同设计](../design/autographics/node-components-queries-and-sprites.md)。

## 1. 目标

1. 固定组件拓扑中，根与子Node均可内聚model/view-2d/on，匿名msg、普通use、前导点和具名引用真实执行；不用根全局字段或Rust玩法代替组件状态。
2. 同类型/同字段/同子实例名在不同父路径下状态、handler receiver、事件端口和资源owner互不混淆；共享定义不共享可变model。
3. 输入、Start/Tick、参数初值/持续绑定、普通视觉与角色运行位置的所有权清晰，符合原执行/预算与Pause/Step/Stop/fresh语义。
4. 交付可复用组件表示/接线与公开source-build消费，不再为查询/精灵复制另一套实例运行时；保016/017/019旧入口兼容。

非目标：任意动态Node拓扑/key/HMR、动态solid/多角色、查询/PNG/精灵/3D/声音/HUD、生产widget退役、view混合/多呈现/Life.Frame、完整UI框架迁移、ACE/AOT/CPC/ECS、修改语言仓/固定SDK或提高预算。

## 2. 架构方案

普通模块图→既有Node2DDialect/Core AST→共用组件/实例/引用/receiver表示→实例局部Auto存储与真实handler→既有行为/角色/相机与GPU服务。优先复用017实例表/父事件和019基础场景生成，剥离业务名/profile特判，不复制parser/解释器或一Node一VM。Rust只编译/调度/连接服务，公式和状态转移在Auto。

新组件模式显式选择；T-02冻结准确CLI与原runtime API映射，旧scene-2d纯场景仍无VM，旧character/camera/g11-2d不静默换语义/产物。拟唯一实施/顺序复审树D:/autostack/.wt/ui-020/auto-ui、codex/plan-020-dev；主检出只合同簿记，只读依赖按已核固定SHA同组定位，禁链接，清理先fresh guard。

## 3. 技术栈

已存在：crates/auto-ui-dialect/src/node2d/{ast,parse,mod}.rs；compiler src/ui_ast.rs::resolve_use_module；pilot src/{load,lower,lower_g11,lower_basic2d,translate,main,lib}.rs；driver authoring/{play,game_input,runtime_objects,runtime_round,host}.rs与tests/node2d_basic_flow.rs。lower_basic2d::collect_basic明确拒绝子model/msg/on，gen_user_source目前只执行根；lower_g11已有ParamKind/Instance/member_hook等有限接线。

拟新增共享组件IR/lowering模块（候选src/component_ir.rs、src/lower_components.rs）、持久fixtures/components2d/、tests/node_components_gates.rs、driver tests/node_components_flow.rs及tools/verify_plan020.py。精确路径和公共函数T-02冻结；必要回归夹具是产品源码，临时探针不得入仓。不写其他仓或Gallery实现。

## 4. 需求分析与背景调查

用户2026-11-01明确要求把下一轮二维查询/精灵能力及有状态组件问题规划成计划。本轮授权设计/合同/编号/导航，未授权实施、接管AG008、创建agent或改预算。两个主题拆三份适中合同，020是共同实际交付前置；021/022无相互功能依赖。

基线auto-ui 10fdf79（019交付c93adb77c25ec2278eb4aaad41c6e4fe25549c22，R2 pass基线afb4ef5经range-diff等价变为8209713，规范/归档/清理收据齐）；ag-examples 912bd70，008用户报告在途。本轮初核无树，末核已出现唯一.wt/ag-008/ag-examples、codex/plan-008-dev、HEAD912bd70；主检出执行记录尚待owner同步，本轮不接管。UI018有自身树codex/plan-018-dev；共享manifest/解析面开工核owner，不加018全完成前置。

权威：[019 Spec](../specs/auto-graphics/node2d-basic-authoring.md)、[消费身份](../specs/auto-graphics/node-authoring-consumption.md)、[017实例合同](../specs/auto-graphics/node2d-g11-authoring.md)、[作者目标](../design/autographics/declarative-authoring-model.md)。raw SHA256：lower_basic2d.rs 6e8652f2d717f110016d6d1efce9ab214239b3344c95d4fd928f41bfe5b52b46；lower_g11.rs 4f5106865d3b601a2e8d1d7b9e1fa3379dbb1ce9d9ce7d7c3864aa38b2f62ad4；load.rs 4837d29c0cfe863dc9db307b1ff160d05b84bc0213fe4c3278166d267405d3d9；translate.rs 894dd1473916177e9c07f23d7a95831352a07eee710cf025a001a239138ceddb；node2d/ast.rs 452fc66987a63a52481d48b9e56faa07f317db24add6585efe95de3e394e5cca。

观察不是验收：019仅根model/on且组件参数+纯view；017已局部验证实例/事件，不等于两套路径已共用完整语义。父事件payload和响应式端口的准确落点需要T-02整链证明。独占020取号后另分配021/022，每份均有实质范围，不为后续波次预占编号。

## 5. 详细设计

### 5.1 先证与交付面

T-01重绑当前HEAD/019真实交付、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467的681内容锁、生成器与pinned执行图各依赖/锁、018/008 owner。SDK历史对象不可达只读受检冻结副本，不换当前语言tip。唯一树在后续用户启动后建立。

T-02最多两种既有机制接法，先跑一Player局部vy/grounded控制+两个同类型有状态组件+不同父receiver事件+两块GPU变化的真实完整链，冻结Plan§9：组件IR/端口/调度/CLI/API/准确V-S命令、成本与故障出口。若必须复制执行器、改外仓语义、把业务放Rust、提预算或只剩根逻辑，needs_replan，不能以parser绿或有限no-go判本Plan完成。

### 5.2 有限支持、状态与绑定

- ≤8个有状态Node实例含根（沿016有限实例包络）；第9个在执行前按资源包络具名拒绝，不吞节点或静默改为无状态；无状态组件/展开场景原128节点义务不缩减，不承诺八个物理角色。固定实例拓扑可含无view的逻辑父/子；普通use、嵌套父子、同定义多实例，各局部model/事件端口独立。
- `.field/.Event`按当前实例，`refs.<local>`按该Node的view/显式输出端口解析，不按全局短名。根同名字段是合法独立状态，不准flatten为一份字段；程序/资源不可变部分可共享。普通helper模块作用域和同名不同模块函数正确解析/降层，不能靠全局名字池假装支持use。
- 原有限数值/int/enum/资源/actor端口与必要bool沿Auto规则；参数只读、模型初值在新实例建立时求值一次，动态输入属性更新不能重置model。模型/handler均与view局部引用，不要求把整份model逐字段映射回根。
- 视觉position/visible等已支持值可绑定局部model，运行结果进入真实GPU；visible=false保对象身份。character.position为新Run初值，运行位置由角色服务拥有，不能每Tick被初值覆盖。Render parent只有显式group语义，Node组合不隐式叠加位移。
- 新增模式不承诺任意if创建/销毁Node子树、key重建或动态solid。017原条件实体销毁及旧实例/slot语义保持；Stop/fresh/初始化失败/Close的组件释放与旧代隔离本期必须验证。

### 5.3 Life、事件、输入

准备所需资源/图形引用/静态绑定→初始化参数与model→声明先序每Node一次Life.Start（仅旧API允许的Start服务）→固定世界Tick。每Tick输入仅按世界原路由采样，声明先序在子Tick前求其当前参数→Life.Tick→同步typed业务事件直达绑定receiver→Tick末允许的呈现绑定同步。精确相依细节T-02在这个可观察顺序内冻结，不能每Node独立推进时钟。

消息使用匿名msg，`.Local`与事件端口payload按类型/形参数绑定；同名子节点在两父实例下必须分别投递，支持转发到正确祖先。系统Life.Start与业务.Start互不混淆。重入/循环不形成无界宿主队列，继承VM/回调守护并准确报告；不新增通用异步消息/跨tick队列承诺。

input.pressed沿旧消耗性语义，不为每Node复制免费边沿或重新采样；有多个读取者时执行顺序与费用可见。Pause无Tick，Step一个中性世界Tick，Resume不重复Start，render次数不推进model。新Run重建模型/绑定/接收方，旧Run completion/消息不得修改或发布到新根。

### 5.4 反例、预算与消费

必要夹具：Player拥有自己的vy/grounded/输入规则；根拥有同名vy哨兵。两个同类型Gauge拥有相同phase/ticks字段、不同初值/速度，局部model驱动各自shape位置/显隐；父a/meter和父b/meter的带int消息只改对应父，错误receiver控制必须红。通过源码/参数改变实际轨迹，不能Rust补公式或getter自报success代GPU。

共享32 credits/世界callback，不能每Node reset。原128场景节点、各runtime API原源码/managed/组合/资源包络与finite f32不变；8个状态实例不获得8倍预算。VM1000steps/2ms、累计100000steps/120resumes/250ms、init30s/tick2s保持，完整Tick计入handler/事件/绑定，8ms目标独立报告。120固定Tick的状态/图像对照、20短轮owner以及32/33/执行守护反例均需真实测量。

沿019 source-build/closure/identity基础扩展实例路径/公开端口映射与实际generator/runner记录；019现规范只实现generator部分，runner来源与真实报告需本计划闭合，不能假定已支持。新源/锁/端口/产物变化使旧记录失效；固定source-only A→B冷重生/真Run、损坏拒绝→精确恢复再可用。旧019及AG008固定消费不跟升；兼容与消费者门按真实触面选。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/stateful-node-components.md | 基础二维根逻辑/子纯view→显式有状态组件模式、局部实例/属性/事件/Life/绑定及消费边界；旧入口不变 | 给021/022与后续作者提供稳定组件合同 | AC-01..06 |

## 6. 测试设计

拟建：pilot --test node_components_gates，driver --test node_components_flow -- --test-threads=1，verify_plan020.py --controls；T-02冻结准确包/完整case集合、非零计数和超时。旧node2d_basic_gates/basic_flow、g11/legacy与dialect/compiler消费者按真实改变选完整触面，当前新增命令未运行。

| V/S | AC / owner_stage / T | 输入→实际入口/范围 | 区分oracle/关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 同定义/字段/子名、两父、同名helper不同模块、缺use/错类型/错端口→真实解析/绑定/执行 | 每路径状态与符号正确；诊断含模块/行/符号，不靠rename才能运行；非法图/引用不panic |
| V-02/S-02 | AC-03 work T-04 | Player局部规则+双Gauge、根哨兵、120Tick参数变化→真实Auto/资源/GPU | 独立f32计数/轨迹，H1变化不改H2，model不因prop更新重置；两ROI按各自model改变，根未代执行业务 |
| V-03/S-03 | AC-03 work T-04 | 带payload父/祖先事件、同名.Start、两个父接收方、错误receiver控制 | 参数类型和接收实例精确，系统/业务事件独立；错误绑定/共享字段/丢payload负控红 |
| V-04/S-04 | AC-04 work T-05 | Pause/Step/Resume/render-only、失败初始化、20短轮Stop/fresh/Close、旧Run结果与合法资源pin | Start次数/Tick计数准确、旧代不串局；最后owner释放，合法共享资源不提前释放，不要求GPU缓存归零 |
| V-05/S-05 | AC-01/05 work T-02/05 | 8实例/32与33、重复pressed读取、完整Tick事件/绑定/VM守护 | 无每Node配额/时钟；正确消耗次序、完整成本和原具名故障，8ms口径明确，不降样本或默提额 |
| V-06/S-06 | AC-06 work T-06 | 固定消费A→B、源/锁/端口/runner/closure坏件、旧019/017真实消费者 | 实际身份+Auto/GPU+恢复后真Run闭合，旧固定入口/生成字节保持；不得用旧截图/空选择/自填记录充绿 |

work自查→同树独立上下文final review→merge冻结门，未指定人员/模型，本轮不创建/联系agent。只测新增/受影响完整定向集，必要UI共享消费者不能省略；全AG留里程碑。GPU/重型门单实例，OS抢焦点自动化仍暂停。

临时材料唯一D:/autostack/plan-artifacts/plan-020/auto-ui/；开工登记owner/用途/路径，外部consumer/SDK/target/log显式此处；禁止attachments含md。合同/SD/handoff/命令exit/关键数字摘要折入本Plan§9。merge对账全部实际输出路径后删自有scope与缓存；仅具名未解决问题按owner/理由/关闭条件登记持有，不覆盖同号他仓材料。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 019实际基线/依赖与有界整链/原预算先证，不复制执行器或根代业务 |
| AC-02 | work | 普通模块、同类型多路径组件、局部状态/typed引用/诊断真实成立，V-01 |
| AC-03 | work | Player局部走跳、双实例model→GPU、typed父子/祖先消息与参数绑定，V-02/03 |
| AC-04 | work | Life/输入/Pause/Step/Stop/fresh、初始化失败/20轮owner/旧Run隔离，V-04 |
| AC-05 | work | 原共享配额/CPU/资源/数值包络与负控、完整Tick报告，V-05 |
| AC-06 | work | 可供后继复用的组件/公开消费合同、A→B/坏件恢复及旧消费者兼容，V-06 |
| AC-07 | review | 同树独立逐AC/V/S/SD/遗漏/格式/warnings复审、绑定提交、零未处置finding |
| AC-08 | merge | 冻结定向门或严格有效复用、实际合master、新Spec/来源/归档导航准确 |
| AC-09 | merge | fresh guard/clean/祖先核后清树分支空组、全部材料路径清理/登记与10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续启动、019实际交付 | 核Spec/SDK/依赖/018与008 owner，建唯一ui-020树，冻结基线与源码触面 | AC-01 | [x] 2026-11-01：019交付c93adb7在基HEAD 6ab7f2d祖先链实证；五触面hash逐一相符（§4原值不变）。ui-020组3树建成：auto-ui@6ab7f2d分支codex/plan-020-dev + 只读依赖auto-lang@b1a4871aa/auto-web@ec9c4ba（detached；组内相对路径解析实测必要，901/902 landed后lang tip新增auto-test-case dev-dep闭包，pilot Cargo.lock重绑+12行=首提交e2e32ff）；driver .sdk-source由tracked sdk/frozen快照prepare=681文件PASS。owner核：018 archived+cleaned（master 6ab7f2d）、008 archived+cleaned（ag-examples c9c672c），无在飞共享owner；主检出WIP仅docs/**簿记（018收据明确保留的他owner记录），零代码WIP。基线绿：pilot cargo check PASS（1m09s）+ gates 9/9 basic、10/10 g11、9/9 lower（与019记录相符）。观察（非020门）：driver verify_source.py→verify_plan013因016/017/019新增三node2d测试文件而source set drift，master基线继承，019同期也未跑它；020不修013 identity。scratch登记D:/autostack/plan-artifacts/plan-020/auto-ui/README.md | AC-01 |
| T-02 | work | T-01 | 最多两接法完整Player/双状态组件/事件/GPU先证，Plan§9冻结IR/端口/CLI/时序/成本/门；不足回规划 | AC-01..06 | [x] 2026-11-01：真机GO——零外仓/零driver/api/runtime改动，业务全在生成Auto。两接法=（1）017 SoA每定义数组+按实例索引handler（model→`<prefix>_<field>[i]`、params只读数组app_start求值一次、`<inst>_lifestart(i)`一次/`<inst>_lifetick(i,dt)`声明先序父先子后）；（2）017 emit_hook+下降期绑定表同步直呼（`on .Ev(p)`→`fn <prefix>_<ev>(i,p)`、emit→`<child>_<ev>(<child>_bind_<EV>[i],v)`，无宿主队列）。真链实测：Player局部vy/grounded走跳（离散oracle步进/apex/无双跳全对）、双同定义Gauge异初值/速度（phase0 0/5，speed 1/6）各驱GPU shape位移与显隐（hide_at 61后blue类归零、green仍在、slot generation/identity不变）、根vy哨兵999全程不被组件写、带int事件只达对应父meter（120tick count 12/8精确；crossed-binding负控实测2/3互换=红）、双GPU变化（12→8tick窗口 da=5.6px/db=32px，20px/world实测校准）。成本冻结：walk tick=30 credits（ag_char_move=26=1+24+1、pressed/held各1、grounded读1）、ag_object_spawn/move/visible各1、共享池32/callback（_ag_spatial_begin每app_update重置，不可每Node reset）、第33计费=具名fault 10（tick60实测触发→探针改even-tick移动后max=32）；VM max_call_steps=5992、max_slices=6（callback限100000步/120resumes内）。关键发现：Objects2V1/Triggers2V1 entry注入_ag_cam2_freeze，未声明camera即fault 27→components模式必须声明camera2d（本探针follow 2d rect[0,-4,40,12] view[40,12]静态居中，1world=20px）；character服务仍one-per-Run。调度（既有entry冻结序）：_ag_char_tick_begin→_ag_obj_tick_begin→app_update(根→实例声明先序，emit同tick同步直达receiver)→triggers档才有_ag_trg_evaluate；trigger事件读的是上一tick末快照（既有语义），业务msg事件零延迟。CLI=新components模式复用objects2d/1装配链（T-03定名，暂components-2d）；V-S命令=driver `cargo test --locked --offline --test node_components_flow -- --test-threads=1`（真门T-04建）+CPU会话SourceSnapshot Api::Objects2V1路径（g11样式）+pilot gates。探针材料：D:/autostack/plan-artifacts/plan-020/auto-ui/t02/（world.at sha 5889adb46e515eb4e82230c167cda5d435e38657c599291ab616622774c6b1c4 3461B+scene.agscene）；瞬态tests/t02_probe.rs两测全绿（未提交，T-03+由产品tests替代）。无needs_replan触发条件 | AC-01..06 |
| T-03 | work | T-02 go | 共用组件IR、模块/实例/typed引用与诊断，V-01，不另造parser/VM | AC-02 | [x] 2026-11-01：零parser/VM改动（node2d dialect原生解析model/msg/on/emit；translate.rs仅增可选call_scope/child_emit钩子，旧档传None字节不变——g06-a/g11 main再生diff干净）。组织选择：§3候选路径取单模块src/lower_components.rs（IR+验证+generation，3118行），component_ir.rs未建。IR=根+声明先序实例树（嵌套路径a/meter、同定义多实例独立SoA槽、def路径集环诊断）；诊断全部具名"module/line/symbol"：缺use、参数缺失/未知prop、未知事件端口、emit arity不匹配、未知模块内helper、环、重复路径、第9有状态实例超8包络、无camera、character/solid/camera放置规则、>128场景节点；非法图不panic（全部Result）。同模块作用域helper：池化`<module>_<fname>`前缀+按定义模块解析（helpers正夹具：helpera/helperb同名bump各自解析）。实现经授权general-purpose子代理执行、协调者独立复验。提交e660798 | AC-02 |
| T-04 | work | T-03 | 局部Auto状态/属性/Life/receiver/payload及视觉/角色接线，V-02/03 | AC-03 | [x] 2026-11-01：夹具fixtures/components2d（main/player/panel/gauge/resources+helpers正夹具+6负夹具）；生成产物generated-components2d/{probe,helpers}冻结（world.at sha 9241d0132f469b605f5d488650ebb40e86af34e79798feb615fc47599d76060b 4654B，再生成diff等值）。Player局部走跳（driver真GPU：落地0.502/走行oracle/apex界限/无双跳全对，角色逻辑在Player组件自身model/handler非根非Rust）；双同定义Gauge异初值/速度局部model驱动各shape位移+显隐（slot精确even-tick公式、hide后generation/identity保持、GPU blue/green类独立运动）；根vy哨兵999（CPU getter，120tick不被组件写）；typed父子事件精确投递（120tick count 12/8；crossed-binding源变异负控红=2/3互换实测）；门=driver node_components_flow 2/2单线程（协调者亲跑exit 0）。作者语法偏离说明：子投递用选择器调用`meter.Add(.payload)`（parser拒绝`emit child.Ev`，零parser改动约束下的等价形态，Call{Dot(child,Ev)}恰为child_emit钩子捕获形）；character2d允许root视图或恰一实例定义（角色服务one-per-Run保持）。提交e660798 | AC-03 |
| T-04 | work | T-03 | 局部Auto状态/属性/Life/receiver/payload及视觉/角色接线，V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | 原额/输入/生命周期/20轮owner与旧消费者，V-04/05 | AC-04/05 | [x] 2026-11-01：driver node_components_flow扩至8/8单线程（协调者亲跑exit 0，17.2s）；pilot components gates 5/5；旧消费者未改动全绿亲核：node2d_basic_flow 13/13、node2d_flow 6/6、node2d_g11_flow 15/15（协调者抽核g11）。V-05实测：budget夹具每tick恰32 credits（walk 30+parity错峰2写）120tick绿、calls=514/max_call_steps=7389/max_slices=8；budget-plus唯一delta=第5个Gauge→tick2恰33计费=具名fault 10冻结在tick2（world ticks==2）；doublepress消耗性边沿（同tick p1==1/p2==0、单跳vy=8-20dt）；envelope=8有状态实例（root+player+6 ticker）120tick绿、9th仍是T-03 lowering负控。V-04实测：Start计数每Run恰1（Resume不重复Start，count保持5）、Tick计数恰=调度数；pause暂停期probe-dispatch处理恰一中性tick（019冻结模式）、render-only帧零推进、single_step恰一tick；20轮Stop/fresh每轮重建（gauge x回初值公式、player回出生点、模板identity (3,4)跨轮稳定、blue/green类仍渲染=合法共享资源未早释放的可观察面）；init-fault=lowering绿9 spawn→运行时_ag_start故障31（slot 8越界，runtime_objects.rs SLOTS=8冻结合同）、host报APIError code=31且不发布good帧、stop可用、随后good项目正常跑。生成器随T-05修一处：多元素def模板名去重（名字=identity），单元素def冻结probe形态字节不变（probe sha 9241d013复验）。实现经授权子代理、协调者独立复验。提交bbfa358 | AC-04/05 |
| T-06 | work | T-05 | 公共source-build消费A→B/坏件恢复/准确身份/格式与SD候选/移交折Plan，V-06 | AC-01..06 | [x] 2026-11-01：tools/verify_plan020.py --controls=16控制组/8产品全PASS（exit 0亲跑）：每产品identity链（world sha==scene behavior.source.sha256==identity、byte_length、closure manifest==files==live fixtures、provenance绑world sha+真实consumer文件include_str!交叉核）；A→B冷再生字节等值（仅closure副本）；mutation敏感性（base_x 4.0→5.0真实变化非静默同字节）；6负控具名拒绝（missing-use为load级，观测形态已记录）；坏件消费=冻结world副本 corrupt一字节→真实load拒绝（观测错误'behavior source closure identity: <scripts/9241d013….at>'）→精确恢复→load+绿Run再可用（driver v06测试9/9真实演示）。runner provenance闭合：identity.json记录实际消费测试文件/消费路径/装载校验机制（019只闭合generator部分的缺口补齐）。fmt clean；pilot全绿9+10+9+5；flow 9/9。SD-01候选文本落scratch D:/autostack/plan-artifacts/plan-020/auto-ui/sd-01-candidate.md（159行，019规范体例，AC-01..06映射，不含aspirational声称）。8产品world sha：probe 9241d013/helpers 8eee3d52/budget e80e4e31/budget-plus b2e16b65/lifecycle d42d84de/lifecycle-doublepress 86645931/envelope a9cc07d6/init-fault a4af6335。勘误：中途commit曾误收瞬态t02_probe.rs，已amend剔除（该文件保持未跟踪瞬态）。实现经授权子代理、协调者独立复验。提交c264d99(amend后) | AC-01..06 |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD和遗漏，finding回work，pass标merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/依赖/输入，定向门或有据复用，实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 实际Spec/来源/归档与导航，证据摘要折Plan，不写派生JSON | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard后清树分支空组，输出路径对账/删自有scope/登记与完成hash | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-020 / plan_revision: 1 / outcome: pass（规划合同可审阅，T-02是实施内可行性门） / next: 后续用户启动后T-01/T-02。独占分配020并为两个已明确后继分别分配021/022，下一号023；本轮仅设计/合同/导航，全部新测试/源码/产物尚未生成或运行，无实施树/agent，不接管AG008。

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-020:r1 / outcome: pass / prepared: reviewed基线910d91d+SD-01 spec于worktree提交6ca5164（docs-only后裔，实现/依赖零变化）；wt-guard clean / rebase: master前进（2f3155b并发plan-014 docs，不在依赖图）；rebase后旧→新映射 e2e32ff→5f1d360、e660798→eb03870、bbfa358→3daf62d、845d83f→3a49e81、a3bb092→cb03b71、85f4ea9→d15f0f7、910d91d→229be47、6ca5164→16ddfa0，git range-diff 7/7全等0不等（安全重写证明）；rebased tip抽核 components gates 6/6 / landed: 主检出 git merge --ff-only codex/plan-020-dev → master tip=16ddfa0（无merge commit） / canonical_verified: docs/specs/auto-graphics/stateful-node-components.md sha256 5dd9e372c452684969c2bd65004aebd918a3bcbf96e70e25883800eb6857be54 @ 16ddfa0 / archived: docs/plans/archived/020-autographics-stateful-node-components.md status:archived 提交7f42d5f（勘误：该提交同时收编他会话两个README导航hunk docs/design/README.md+6、docs/plans/README.md +7-2——docs-only并发WIP误并入，内容保留、归属记录于此；master已landed不重写；本merge块系归档后补记，替换一次未命中的静默no-op插入） / cleaned: 三树fresh guard全clean后git worktree remove×3（auto-ui含--force：唯一未跟踪=授权瞬态t02_probe.rs随树消亡；auto-lang/auto-web各自repo注销+prune）+branch -d codex/plan-020-dev(was 16ddfa0已landed)+空组rmdir+worktree list 0项；plan-artifacts/plan-020全删（SD-01候选已landed为canonical spec，其余摘要折入本文件§8/§9，delete-at-merge 2026-10-31）；主检出遗留非本Plan WIP=根Cargo.lock 1行(a2r-std getrandom,他会话)+attachments/012+docs/design/autoui+plans 007/014(他会话)，保留不接管 / 交付后观察: 本仓为库/研究仓无驻留生产二进制交付面（017/019同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang），020不适用 / 10任务10/10完成，AC-01..09全关，交付commit 16ddfa0

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 共享组件IR、端口payload、参数/绑定与Life准确实现 | T-02在本合同语义/旧API许可内冻结；只剩根逻辑或改预算则needs_replan |
| D-02 | 新模式如何映射旧API及重入/守护、实际成本 | T-02有界真链，不能用parser绿替代；无无限宿主队列 |
| D-03 | 当前018共享依赖和AG008固定消费owner | T-01实际diff核，008不升级、不被新增前置阻塞 |
| D-04 | 独立终审上下文/GPU窗口 | T-06按届时授权安排，缺证据不标reviewed |

当前仅规划；021/022实施等待本计划实际交付，不提前假定接口已支持。
