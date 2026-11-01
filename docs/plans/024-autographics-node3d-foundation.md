---
plan_id: PLAN-024
status: drafting
feature_name: AutoGraphics三维声明静态资源与有状态组件基础
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-020]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-foundation.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-024：3D声明、静态资源与组件基础

## 0. 变更摘要

在020已交付的组件/模块/receiver机制上增加显式view-3d作者入口：有限三维图元、静态平色GLB、同维translation/visibility层级，根与子Node从首版均支持model/msg/on/Life.Start/Life.Tick和普通use。作者写声明/具名资源/局部逻辑，scene JSON、整数句柄和运行单源是生成产物。

本计划提供后续G02与G03/3d迁移所需引擎能力，不修改Gallery项目或新建内容计划。与声音/HUD通用化无功能依赖；共用AST/loader/组件接线按owner短窗协调。设计见[下一阶段共同设计](../design/autographics/node3d-foundation-and-generic-presentation.md)。drafting r1、0/10，暂不实施。

## 1. 目标

1. 纯静态三维场景可无用户model/Tick/VM、无角色与运行相机；有逻辑场景的Node可局部持状态、处理typed事件并驱动实际三维位置/显隐。
2. 普通use拆分组件/资源/helper，同定义多实例/同字段/不同父路径互相隔离；树状声明不等于独立VM或强制碰撞主体。
3. GLB由symbol/typed参数引用，复用既有准入/资源owner；同SHA共享原始/几何/GPU资源，离线项目来源和坏件恢复闭合。
4. 在新源真实链上承接G02四项＋原坏资产拒绝、G03/3d六项原义务，不把单banner当全部验收；旧2D各模式/既有3D运行器规则不变。

非目标：3D角色/运行相机/实例生成销毁/触发/查询、纹理/材质系统扩面/骨骼/节点动画、动态资源/拓扑/key/HMR、运行期scale/yaw/pitch/TRS、任意3D模型格式、view与view-2d/view-3d混合或多呈现、生产widget替换、编辑器反向写Auto、ACE/AOT/ECS/CPC/跨端与发行。后继能力另立合同，不预取编号。

## 2. 架构方案

普通模块闭包→现有Core Expr与Node领域AST扩展view-3d→复用020的组件实例/符号/handler绑定→三维场景/资产/具名引用表示→原静态场景play，或显式基础行为v1执行Auto局部逻辑→现有层级/GLB renderer/GPU。

优先纯场景behavior=null与脚本autographics.behavior/1：v1已提供3轴translation/visibility/group派生，不为了复用objects3d而强迫无角色场景声明假Player/相机。准确profile/schema/可空actor规则由T-02真链冻结；若现行装配不能提供此能力，带反例回规划，不暗换成角色专用档。只新显式入口，旧assembly/字节/接口/费用不改；不复制parser/词法/表达式/VM或为G02/G03各写专用生成器。

拟唯一实施/同树顺序复审树D:/autostack/.wt/ui-024/auto-ui、codex/plan-024-dev；只读固定语言/SDK依赖，主检出只簿记，禁止链接，清理先fresh guard。

## 3. 技术栈

已核：crates/auto-ui-dialect/src/node2d/{ast,parse,mod}.rs；pilot src/{load,translate,lower_components,lower_basic2d,scene,main,lib}.rs；driver authoring/{document,hierarchy,assets,behavior,behavior_project,play,storage}.rs、api/behavior_v1.at。现NodeDecl只有view2d，ResourceKind仅Quad/Circle/Color（022在途扩图像）；Document实际3D种类Cylinder/Player/Mob/Model，group仅translation/visible，不发明cube或完整TRS支持。

拟新增共享三维scene/resource适配模块（如src/lower_scene3d.rs，保共用组件IR）、fixtures/node3d/、pilot tests/node3d_basic_gates.rs、driver tests/node3d_basic_flow.rs与tools/verify_plan024.py。名字/准确路径T-02冻结；永久回归夹具及许可属于产品，临时探针不得入仓。外仓和Gallery只读。

## 4. 需求分析与背景调查

用户2026-11-01明确授权在auto-ui规划本计划与声音/HUD通用接线收口，保原预算/验收、同步路线，暂不实施。授权只含设计/合同/编号/导航，无实施/agent/迁仓/发布；不接管022或AG006/009，不修改其合同/依赖。

基线auto-ui0b6e74d，020交付16ddfa0、0211bec52f、02332d7a81在master；初核时022树处于独立复审修复，后续以其owner记录为准，main存在他owner的Cargo.lock/计划/导航WIP，均保留。AG008已交付、Gallery仍12项21入口/迁移6/21；本轮观察ag-examples c5d3876，AG006/009已有在途树，不能当旧“均drafting无树”继续写。

权威：[020组件](../specs/auto-graphics/stateful-node-components.md)、[作者目标](../design/autographics/declarative-authoring-model.md)、旧[层级](../../../auto-lang/docs/specs/auto-graphics/scene-hierarchy.md)/[资产](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)/[行为](../../../auto-lang/docs/specs/auto-graphics/behavior-api.md)与[Gallery基础六项](../../../auto-lang/docs/specs/auto-graphics/gallery-basic-six.md)。本Plan只提新Spec，不改canonical现状或派生JSON。

raw SHA256：node2d/ast.rs 96f336f9d89155033199cc01b62df198d94a407bd129f20f15755277b830ee0b；load.rs 4837d29c0cfe863dc9db307b1ff160d05b84bc0213fe4c3278166d267405d3d9；lower_components.rs f0652c75d3c43e28a470b3096b3abd13af0b4947d24e7e627692b6faf7923384；driver assets.rs 9a01c06f73bdd16b04c2fc019d97574c76031ae4e631bb6e86dc2d97dd070e6b；behavior_v1.at 343f34a27c62f58980ef9df34a009341b20ba47d5a1b38cbbde7a1c3d499939f。后续022合入可合理改变共享源，T-01绑定实际基线，不将旧hash当永远锁定整个仓。

只读G02原GLB SHA42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671、1528B、两部件红/绿三角，完整MIT许可；原scene SHAe118a0c935f259aa08b27a0286c0697a6d7b74334ad8c8ace929a7c34276494b。G03/3d scene SHAb20e9723fc4a94a77a803ff3793ba6f1de0e214a4bedd2e7711c78948bded185；root(0.5,0,0)+child(0.5,1,0)+deep(0.8,0,0)逐层f32→(1.8,1,0)。这些是规划输入，不是新链通过证据。

## 5. 详细设计

### 5.1 进入与有界先证

T-01核020真实交付/Spec、当前master与022/025及UI其它路线owner、两图锁/依赖源、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467的681内容锁。历史对象缺失只用受检冻结副本，不切SDK tip。未获后续实施启动前不建树或消费工程。

T-02最多两种既有装配接法，先跑三条闭环：无角色G02平色多部件GLB纯场景；无角色G03/3d层级/祖先隐藏/父平移；含两个同定义局部状态组件与typed父事件、动态三轴位置/显隐的真实Auto/GPU。冻结§9准确AST/profile/schema/CLI、NodeRef3D/资源端口、声明绑定/显式写的顺序、费用/限额、独立图像/层级oracle与全部V/S命令。不能靠静态截图/parser绿/隐藏角色占位或复制执行器过门；不成立needs_replan，不能删有状态组件或纯场景能力。

### 5.2 声明、作用域与生命周期

- 本片支持普通node中的单一view-3d出口，组件可无view而仅持逻辑；多个出口选择与嵌入混合不在本片，不因此回退统一命名。复用Core三元坐标表达式，不新增一套语言/解释器。
- 候选呈现元素为现有3D图元、同维group和静态model；准确拼写T-02冻结。图元种类/初始scale/yaw/pitch/rgba只映射现有Document能力：不增加任意mesh/PBR、且保既有3D图元占位色规则；GLB原部件颜色由原资源管线呈现。
- 参数至少int/float及必要Mesh3D/Model3D/具名NodeRef3D端口，typed资源与引用不成为作者整数ID。参数沿只读、初始化语义；本片要求的响应是本Node model→本Node view位置/显隐，不顺手承诺完整跨组件响应系统。readonly赋值拒绝而RHS读取合法；同定义多实例、不同父路径同名子节点、两模块同名helper均正确。
- 根/子model与匿名msg/on从首版即真实执行，`.field/.Event`按实例；Life.Start每Run一次、Life.Tick固定世界dt=1/60、同步消息按020接收方规则，无一Node一VM/独立Tick/配额。不能把子状态搬根、按Player/Coin/G02等业务名称生成规则。
- 场景parent只由显式同维group决定，Node组合本身不隐式叠加transform；继承逐层f32平移与祖先visibility AND，无组旋转/scale。固定拓扑，无运行spawn/despawn。visible=false保叶身份/资源，不等于销毁。
- 动态translation/visible绑定和必要显式同维引用写经原公共v1完成；T-02冻结同callback读写可见时机，不重排带副作用调用。primitive的scale/angles/material、model资源绑定在Run内保持既有不可变规则；3D叶可在不改资产的前提下动态平移/隐藏。
- Pause无Tick，Step一次中性Tick，Resume不重Start，render/capture不推进状态。Stop恢复文档默认，fresh Run重建局部model/handler/Run引用；旧Run结果不得落新实例。不同维度引用/parent、环/未知use/重名/错端口具名定位拒绝，无panic/深度截断。

### 5.3 GLB、身份与原资源包络

仅旧静态平色GLB2域：单JSON+BIN/嵌入buffer，POSITION/NORMAL/有限TEXCOORD与合法indices按旧语义；无外部URI/image/texture/animation/skin/morph/压缩/扩展要求。缺可选默认scene声明的单scene合法；保重复node/mesh名和重复实例，不靠display name绑定。所有声明含未使用项在decoder分配前校验类型/span/环/变换/几何，复用既有assets准入，不复制GLB解析器，也不信manifest自报bytes/尺寸。

上限保持：raw≤8MiB、JSON≤1MiB、asset nodes≤128、expanded primitives≤16、每primitive vertices≤65536/每asset vertices≤262144/indices≤786432。原8身份/32MiB raw、64MiB CPU/GPU/8文件32MiB含候选、history、预留和残留；原renderer≤512潜在mesh实例含隐藏项。模型内部节点、场景对象+group≤128、组件有状态实例≤8是不同计量单位，不能混算/每组件重置。

原完整raw SHA/length/format绑定资源；Model3D符号可跨模块传参，同SHA共用不可变CPU/GPU owner而不是每Node复制mesh。资源许可/原始来源进可搬迁闭包；warm更新位置/文字无关不能重解析/上传GLB。Editor/Run/history/外部pin按最后owner释放，合法pin不早释；20短轮与最终owner观察区分缓存/泄漏，不声称驱动内部驻留归零。

### 5.4 原验收与计量

G02原义务B02-01运行成功、B02-02红部件像素>20、B02-03绿部件>20（原800宽图每2像素采样和色差阈值不降）、B02-04inspect重开document==运行document；B02-05原tamper_negative实际单字节坏GLB被engine拒绝。G03/3d原B03-01 world=(1.8,1,0)、B03-02父x+2→deep x3.8、B03-03祖先隐藏有效、B03-04 base/hidden两次真运行成功、B03-05实际capture不同、B03-06 Run不改文档/Stop恢复。十一义务都要经新作者链重建，旧banner或源hash不能代替。

附加必测：两个同定义组件独立phase/位置/显隐、两个父接收载荷各自正确、重命名root/model/资源不改行为；128场景行边界/129拒绝和8有状态/第9拒绝；无角色/无actor纯场景不强制VM。层级原abs≤100000、finite/f32/隐藏节点也校验，几何与像素由独立公式观察，不调用被测派生函数造expected。

基本v1自身没有query32credits池，不假称其每个getter都收费或免费开放查询；严格保持所选API原计价/managed/source域，生成Auto与业务/helper一起计入原VM1000steps/2ms分片、100000steps/120resumes/250ms累计、init30s/tick2s、1-inflight/四pump credit。若后续明确选带空间服务的新档，32/callback原价必须保，不在本片提供该服务。原v1用户源256KiB、API/bridge64KiB、combined320KiB按实际准入核；完整Tick/准备/派生/GPU各自记账，8ms pump目标不变且不冒充硬时延保证。不得扩quota或省层级/资源校验换绿。

### 5.5 固定消费、恢复与兼容

新显式prepare产scene＋可选用户Auto＋identity/roles＋模块/原GLB/许可closure；同时绑定实际generator与runner、SDK/两图Cargo锁/toolchain/参数，不把源SHA字面量当二进制证明。沿019/020实际source-build模式扩展，不改任何旧exact锁或自动fallback；静态档可无world，消费者准确区分而不加假Tick。

A→B仅持完整闭包与固定工具离线冷再生、纯场景和有状态真Run/GPU；SaveAs/关闭/新根重开保持资源/文档默认。GLB/许可/被用模块/identity/锁/record实际变异→拒绝→精确还原→同入口重开再Run；失败发布保最后有效产品或准确报告自身残留，不能产出半新可用Ready。共享语法/资源接线对016/017/019/020/021/023及022若已落地选真实受影响门，旧生成字节/物理/锁不动。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-foundation.md | 3D底层与2D作者入口分离→view-3d/静态GLB/有限层级/局部组件/来源与支持域；旧入口不变 | 为G02/G03三维迁移与后继角色提供稳定基础 | AC-01..06 |

## 6. 测试设计

拟pilot --test node3d_basic_gates、driver --test node3d_basic_flow -- --test-threads=1、tools/verify_plan024.py --controls；准确命令/非零case与超时由T-02冻结，目前未创建/运行。只跑本Plan新增/受影响定向测试和必要共享UI消费者，全AG留里程碑；不跑语言根Cargo或干扰性OS输入探针。

| V/S | AC / owner_stage / T | 输入→真实路径/范围 | 区分oracle/关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 普通use/view-3d/typed Model与引用/多路径同名→AST/IR/准入 | 合法符号与改名稳定；缺模块/错维/环/只读写/错资源具名定位拒绝；无另一parser/专例分派 |
| V-02/S-02 | AC-03 work T-04 | 原GLB/原hierarchy主父移隐藏→公共Host/GPU/独立世界坐标 | B02五＋B03六义务逐项新验；红绿部件与隐藏/移动像素，原阈值与纯场景无角色保持 |
| V-03/S-03 | AC-03/04 work T-04/05 | 两同定义局部model/三轴绑定/显隐/两个父payload→真实Auto/GPU | 独立f32轨迹/两ROI/根哨兵与receiver正确，改一个实例不动另一，显隐不丢身份；错误绑定负控红 |
| V-04/S-04 | AC-04 work T-05 | Pause/Step/Stop/fresh/旧Run/20短轮/合法外部GLB pin | Start/Tick次数正确、文档默认恢复、旧Run不串；warm上传0、最后owner释放不误伤合法共享 |
| V-05/S-05 | AC-01/05 work T-02/05 | 128/129行、8/9状态、GLB预算/坏typed/span/隐藏512上限、完整Tick | 原资源单位/分配前准入/VM/source域保留，边界具名拒绝；原asset/authoring反例与真实负控有效，8ms口径另报 |
| V-06/S-06 | AC-06 work T-06 | source-build A→B冷闭包/SaveAs重开/坏件修复/旧2D消费者 | 实际工具/runner身份＋新Auto/GPU和恢复再Run，静态null world准确；旧模式字节/行为保持，无Gallery写入 |

work自查→同一权威树独立上下文final review→merge冻结门；未指定人员/模型，不创建agent/chat。GPU/窗口/重型门机器单实例，OS抢焦点继续暂停。

所有临时consumer/SDK/target/log/PNG仅D:/autostack/plan-artifacts/plan-024/auto-ui/，开工登记owner/用途/在途；禁止attachments含md。契约/SD/命令exit/关键数字/移交摘要折Plan§9；merge对账全部实际输出后删自有scope/构建物，仅具名未解决问题按owner/理由/关闭条件登记持有，不能覆盖同号他仓材料或手写派生JSON。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 固定020/SDK/依赖与三条真链有界先证，纯场景不隐藏角色/相机或复制执行器 |
| AC-02 | work | view-3d/use/typed资源引用/组件符号/错误定位完整，V-01 |
| AC-03 | work | 原G02五＋G03/3d六义务和局部状态/事件→真实GPU，V-02/03 |
| AC-04 | work | 实例/父receiver隔离、Life/旧Run/20轮资源owner与warm，V-03/04 |
| AC-05 | work | 原API/VM/source/128节点/8组件/GLB与512容量包络及反例，V-05 |
| AC-06 | work | 公共准备/实际来源、离线A→B/SaveAs/坏件恢复、旧2D受影响兼容，V-06 |
| AC-07 | review | 独立逐AC/V/S/SD/十一旧义务/遗漏/格式/warnings核验，绑定提交，零未处置finding |
| AC-08 | merge | 冻结定向门或有据复用后实际合master、本仓Spec/来源/归档导航准确 |
| AC-09 | merge | fresh guard/clean/祖先核后清树分支空组、全材料路径对账删除/登记、10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施启动 | 核020/SDK/两图锁/022与025及跨路线owner，建唯一ui-024树/冻结源 | AC-01 | [ ] |
| T-02 | work | T-01 | 最多两接法纯GLB/层级/双状态组件三链先证，§9冻结AST/模式/CLI/计量/原十一义务 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 共用Node AST/实例表示扩view-3d/typed模型引用与资源准入，V-01 | AC-02 | [ ] |
| T-04 | work | T-03 | 原GLB/层级及局部Auto状态/事件/三轴绑定真实接线，V-02/03 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 原预算/128-129/8-9/资源反例、生命周期/20轮owner与warm，V-04/05 | AC-04/05 | [ ] |
| T-06 | work | T-05 | source-build/冷A→B/SaveAs/修复/旧消费者、SD与移交折Plan，V-06 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD/原分母复审，finding回work，pass标merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/依赖/资源/输入，定向门或有据复用后实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 沉淀Spec/来源/归档导航与证据摘要，不写派生JSON或改变Gallery完成数 | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard清树分支空组、所有输出全对账删scope/登记与完成计数/hash | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-024 / plan_revision: 1 / outcome: pass（规划合同可审阅，已有功能前置020已交付，T-02是未来实施内先证门） / next: 后续用户明确启动后T-01/T-02。本轮暂不实施，drafting0/10；未创建新源码/夹具/资源/树、运行构建/GPU或agent。此记录不是可行性go或独立终审。

2026-11-01规划预检：两合同各0..10节、10任务（work6/review1/merge3）、9AC/6组V-S/1SD与drafting0/10一致；新增合同/共同设计19个本地链接可解析。独占取号后复扫活动/archived/archive编号唯一、下一026；依赖无环，两计划无相互功能依赖。024十一旧义务按verify_g02四require＋tamper_negative一项＋verify_g03六require静态枚举；025保G12a原八名与G12b H01..12，原预算/旧023字节不变。均为文档静态检查，不是新产品验证/可行性go或独立终审；未创建实施树/提案Spec/源码/素材、构建/GPU/音频/agent。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 共用Node AST/三维资源表示与无角色状态组件的准确装配 | T-02在旧v1/层级/资源合同内最多两接法冻结，不强迫角色或删局部状态 |
| D-02 | model绑定/显式写/parent派生的准确时机和所选API计量 | T-02真链与独立f32 oracle，不能把v1与空间32池混称 |
| D-03 | 022/025共享AST/load/translate接线与消费基线 | T-01核owner/实际已合入版本，功能上不等022/025，无法隔离共享写则顺序短窗 |
| D-04 | 独立review/GPU时段与历史GLB/许可来源 | T-01/06核固定字节/实际授权；新证据未跑不计完成，旧SDK来源不替换 |
