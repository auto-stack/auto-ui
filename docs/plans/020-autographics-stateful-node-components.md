---
plan_id: PLAN-020
status: drafting
feature_name: AutoGraphics有状态Node组件与局部事件组合
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 0
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
| T-01 | work | 后续启动、019实际交付 | 核Spec/SDK/依赖/018与008 owner，建唯一ui-020树，冻结基线与源码触面 | AC-01 | [ ] |
| T-02 | work | T-01 | 最多两接法完整Player/双状态组件/事件/GPU先证，Plan§9冻结IR/端口/CLI/时序/成本/门；不足回规划 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 共用组件IR、模块/实例/typed引用与诊断，V-01，不另造parser/VM | AC-02 | [ ] |
| T-04 | work | T-03 | 局部Auto状态/属性/Life/receiver/payload及视觉/角色接线，V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | 原额/输入/生命周期/20轮owner与旧消费者，V-04/05 | AC-04/05 | [ ] |
| T-06 | work | T-05 | 公共source-build消费A→B/坏件恢复/准确身份/格式与SD候选/移交折Plan，V-06 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD和遗漏，finding回work，pass标merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/依赖/输入，定向门或有据复用，实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 实际Spec/来源/归档与导航，证据摘要折Plan，不写派生JSON | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard后清树分支空组，输出路径对账/删自有scope/登记与完成hash | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-020 / plan_revision: 1 / outcome: pass（规划合同可审阅，T-02是实施内可行性门） / next: 后续用户启动后T-01/T-02。独占分配020并为两个已明确后继分别分配021/022，下一号023；本轮仅设计/合同/导航，全部新测试/源码/产物尚未生成或运行，无实施树/agent，不接管AG008。

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 共享组件IR、端口payload、参数/绑定与Life准确实现 | T-02在本合同语义/旧API许可内冻结；只剩根逻辑或改预算则needs_replan |
| D-02 | 新模式如何映射旧API及重入/守护、实际成本 | T-02有界真链，不能用parser绿替代；无无限宿主队列 |
| D-03 | 当前018共享依赖和AG008固定消费owner | T-01实际diff核，008不升级、不被新增前置阻塞 |
| D-04 | 独立终审上下文/GPU窗口 | T-06按届时授权安排，缺证据不标reviewed |

当前仅规划；021/022实施等待本计划实际交付，不提前假定接口已支持。
