---
plan_id: PLAN-019
status: archived
feature_name: AutoGraphics Node2D场景层级与走跳相机
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 10
total_steps: 10
depends_on: [auto-ui/PLAN-017]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-basic-authoring.md, docs/specs/auto-graphics/node-authoring-consumption.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, auto-ui-compiler, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-019：Node2D纯场景、层级、走跳与跟随相机

## 0. 变更摘要

把已交付017的有限角色/触发作者入口扩展到首批普通二维项目：G01图元、G03二维层级、G06两角色布局、G08二维跟随相机所需能力。支持零角色纯场景、视觉叶子/group与组件组合、单角色跳跃/支撑、可选相机；普通use、node/model/view-2d/on、Life.Start/Tick、匿名msg/具名引用继续复用。

内容迁移归[ag-examples/008](../../../../ag-examples/docs/plans/008-autographics-gallery-node2d-first-wave.md)，硬等本计划实际交付。总体剩余20入口的波次见[迁移设计](../../design/autographics/gallery-node-authoring-migration.md)。本轮仅规划drafting0/10，不实施、不建树/agent。

## 1. 目标

1. 纯绘图/层级工程不再要求伪造一个Player/trigger，不为没有逻辑的场景制造强制Tick；视觉可见性、父子变换和组件状态各有准确语义。
2. 真实Auto可表达原G06左右/Jump边沿、重力/速度/支撑与原G08相机配置；不得用Rust内置Controller替代作者逻辑。
3. 原样承接首波五入口的几何/像素/输入/布局/变体/生命周期要求，保旧profile价格/数学/资源/守护。新能力不是完整平台游戏，也不推翻011原额no-go。
4. 提供可真实取得/重建并启动的交付消费契约，区分原已认证二进制与固定源码重建，适配merge即删除计划材料政策；不伪装旧hash或依赖历史exe永久在盘。

非目标：3D/view-3d、多呈现/混合UI、Life.Frame、任意runtime拓扑/动态solid、多角色、新物理、查询/PNG/GLB/精灵/声音/HUD、G04编辑器反向源码同步、ACE/AOT/CPC/ECS/HMR、语言仓/固定SDK语义修改、预算提升、Gallery内容/全量迁移一次交付。

## 2. 架构方案

普通模块→Node2DDialect/Core共同语义→有限二维声明图/实例与类型化端口→既有场景层级与行为服务的结构化生成→固定SDK真实Auto/Host→GPU。新作者profile拟scene-2d、character-2d、camera-2d（T-02冻结准确CLI和原runtime profile映射），共用解析/绑定/IR/资源处理；不是每例复制一个lower_gXX。016默认和017 g11-2d输入/产物保持兼容，不静默升级。

拟唯一实施/同树顺序复审树D:/autostack/.wt/ui-019/auto-ui、codex/plan-019-dev。主检出只计划簿记；语言/auto-web/auto-down必要依赖只读固定SHA同组树，禁止链接/外仓写面。018构建配置/语言901/UI014如有共享manifest或解析端口，按实际owner协调短窗，不把其全完成加成隐式前置。

## 3. 技术栈

观察已存在：crates/auto-ui-dialect/src/node2d/{ast,parse,mod}.rs；compiler src/ui_ast.rs；pilot src/{load,lower,lower_g11,scene,translate,main,lib}.rs；driver authoring/{document,hierarchy,play,host}.rs及host/子目录、api/{character2d_v1,runtime_camera2d_v1}.at、tests/{node2d_g11_flow,node2d_flow}.rs。角色grounded和输入成本来源为语言仓game-actions-and-character2d.md，相机来源runtime-game-cameras.md，层级来源scene-hierarchy.md。

拟新增：pilot src/lower_basic2d.rs（或既有共同lowering中等价组织）、fixtures/basic2d/持久产品回归夹具、tests/node2d_basic_gates.rs、tools/verify_plan019.py；driver tests/node2d_basic_flow.rs；共同来源消费工具/manifest（具体路径T-02冻结）。不新增lexer/parser/VM，不把全部新代码堆到G11专用lowerer或向Core AST塞图形字段。真实已有文件/符号在T-01重核，以上新路径均尚未创建。

## 4. 需求分析与背景调查

用户2026-10-31要求其余旧AG示例都迁新代码形式，并按需补新形式支持。依设计+Plan流程，本轮将此目标拆分并提交首波合同供审阅，不将其解释为已批准未见合同的全部实现/合入/发布。允许规划/设计/取号/导航；没有本轮实施/构建/GPU/新agent授权。

本仓观察HEAD4e9acd52596c69f380447ef0608ef256439802e4，017实际delivery9a8a5be60b19d9a385f87fbf1828857389013c9d已入主线；内容仓dbb4b75（007交付939f20c），12项21入口仅g11/2d已迁新。当前018由UI构建配置路线先取号，本计划独占重核最大018/.next-id019后分配019，下一号020，不覆盖018；AG008独立编号。其他owner未提交文档保留。

权威：[017实际Spec](../../specs/auto-graphics/node2d-g11-authoring.md)、[语言仓AG overview](../../../../auto-lang/docs/specs/auto-graphics/overview.md)、scene-hierarchy/game-actions-and-character2d/runtime-game-cameras/gallery-basic-six；本仓[作者目标](../../design/autographics/declarative-authoring-model.md)及新迁移设计。历史Specs含过时仓归属段，不据其标题宣称旧单文件限制为最终要求，也不擅改已有数学/验收。

raw SHA256：017 Spec a8b5b72403653521965c1b5024c826473699d178ed374c1d1b7bf3c17b06d958；lower_g11.rs 4f5106865d3b601a2e8d1d7b9e1fa3379dbb1ce9d9ce7d7c3864aa38b2f62ad4；load.rs 4837d29c0cfe863dc9db307b1ff160d05b84bc0213fe4c3278166d267405d3d9；node2d/ast.rs 5cf76769fa1ff4f4433191e522021f0854c3e4462d72be6a8d94004768a2a42a；parse.rs 9681e2be0eed5bdd20e9974fa01958462f9f32dd90fea82612e8c8c11aab561f。当前017要求恰一character、根solid/camera和程序化资源，角色移动服务未向作者暴露grounded/Jump完整语义，因此不能直接批量改Gallery。

旧首波来源：内容仓tools/verify_examples.py（G01 12个require点、G03每维6点、G06 7个require点但布局循环实际10次断言），G08/2d回执8项。G01/G03元数据无behavior，G03 README所称observer并非当前二维文件/入口事实，不能将不存在脚本当已交付；原检查banner数量不等于断言分母。

试玩观察（非本轮测试）：此前固定017源码本机重建39a8e27b…（不同于原锁6ceeb384…）在tick112自动暂停backlog>4。未知原因须区分捕获/背景负载与执行路径，不能据此直接改守护，亦不能当桌面运行已证明稳定。新代码触及的回归须实际对照关闭。

## 5. 详细设计

### 5.1 进入与有界闭环

T-01重核017delivery/Spec/独立review、当前写面/owner、生成器只读依赖、旧SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467的681内容锁、各Cargo锁；历史对象不可达只用受检冻结副本。五个入口及所有变体从ag-examples dbb4b75只读固定，未来008 tip不是引擎测试依赖。

T-02最多两种既有端口接法先完成纯图元/深层group+显隐、走跳落地与坑道对照、主alt相机三条真实整链；冻结公共CLI/IR/typed端口/消费身份/精确V-S/预算与输出协议。初始化/普通走/起跳/落地/中性/相机/纯绘图完整成本先测，不能先写完所有代码再检查是否超过32。方案和反例摘要在Plan§9，临时探针仅指定scope。必须改外仓语义、复制解释器、提高预算、删原行为或不能提供可闭合消费身份时needs_replan，不把no-go作为本Plan通过。

### 5.2 声明和运行边界

- scene-2d允许0角色/0触发/无camera；有限纯视觉叶子（候选shape2d）与group2d的具体写法T-02冻结，仍在view-2d中组合。纯视觉不自动成为solid，visible=false保对象身份，不能被lower成if销毁；颜色/alpha/尺寸/初值来自具名资源和属性。
- Node组合树、render parent图、模块图和运行状态分别处理。group仅同维平移与可见性继承，旧上限128总节点、同维单父/无环、按层f32求和、有限坐标；保旧排序/隐藏节点规则，不额外引入深度截断。组件可组合复用，作者不写数字parent/id；同定义实例model/receiver不展平共享。
- character-2d恰1静态绑定角色、有限静态solid；默认原G06呈现，不自动启用相机。camera-2d再声明1跟随camera和typed target，原bounds/view_size/clamp/letterbox规则不变。原每种runtime API显式选择，不能因存在某字符串或文件名切模式。
- 新服务覆盖左右轴、Jump pressed沿、move_and_slide结果的grounded/必要support与refs，常规精确拼写T-02冻结。gravity、vy、speed、jump条件在Auto model/on/helper，不移到Rust。纯input的重复读取/消费顺序保持原约定，不因多个Node重复采样或凭缓存豁免费用。系统Life与业务.Start区分，Frame越界拒绝。
- 参数/资源/父节点/角色/camera引用按类型和owner解析，错维/循环/重复局部绑定/错资源/错service/profile准入前定位拒绝；普通use复用共享模块语义，不复制AST或文本替换导入。不承诺本期全部Auto语法/无限规模。

### 5.3 行为与原额

原G01按alpha升序、不透明先绘、等alpha按原数组序；四变体须按原图像规则变化，不升级为任意绘制顺序。G03二维深层叶子world(35,5)，父移动后x+20、隐藏祖先后叶子不可见，Run/Stop不改编辑文档；多层相加按原f32次序。

G06原speed4/fast5、jump8、gravity20、dt1/60，台阶/坑道几何不改；按原脚本时序消费一次Jump、保持无双跳/无缓冲跳、地/墙/顶/最终支撑与中性Step。必要getter不能读错上一move结果。G08主rect x[0,30] y[-4,12]/view12×8，alt rect x[4,26] y[-2,10]/view8×6；同tick角色/相机共同发布，800×380下主570×380/两侧115，alt约506.67×380/两侧146.67，独立公式核对。

旧公共服务32 credits/callback、角色正常move26/early25、grounded1、左右2、pressed1不变；完整走跳候选30、必要其它调用另计，不能把G11满池经验套作所有profile证明。每Node不重置配额；源码闭包/生成用户源<=原256KiB，managed与合计按实际选择旧API原envelope核验（不得统一抬成更大值）；内存/资源/节点/碰撞与VM1000steps/2ms、累计100000steps/120resumes/250ms、init30s/tick2s等沿原服务；8ms pump目标另列met/missed/unmeasured。0触发静态场景不占用或模拟8个runtime槽。

场景层级与角色/相机的支持条件分开：纯层级保原128节点内无语义深度截断；角色/相机保其原wide128/dense-depth64条件、冻结solid集合及初始化CPU限制（127 solids的既有超限负例仍拒绝）。不能把各profile支持矩阵拼成“全部组合都可用”，也不因源码Node少就少计展开后的实际节点。

20短轮Run/Stop/Close及失败准备按最后owner检查，合法共享mesh/payload可由Editor保留至最终关闭；不要求驱动缓存归零。暂停/焦点/文本输入清理、Step中性、Stop恢复均保原规则；本轮不恢复OS抢焦点自动化。试玩backlog观察须有原/新同背景对照及完整Tick报告，不能静默调大阈值；与新写面无关的问题记录具名债务，不以假PASS关闭。

### 5.4 产物和可用的消费身份

公共准备输出scene、可选真实用户Auto（纯场景可不需要用户脚本）、closure与identity；包含每模块bytes/SHA、声明→对象/资源/父/角色来源映射和具体runtime profile。变体源变化使旧Ready失效；半写/坏件不得混为有效产物。可在既有g11-2d和新模式共用数据结构，必须保旧有效输入/输出兼容证据。

共同消费规范SD-02在T-02冻结，仅对明确选择的新迁移入口启用版本化source-build模式：固定交付commit/完整源闭包/SDK681/依赖锁/平台/工具链与构建参数；受控离线准备和构建产生记录，绑定实际generator/runner二进制SHA，消费者核该记录、实际文件与运行报告。实际build/执行必须闭合，不能信任任意自填receipt或与被测输出同源的success旗标；错source/依赖/参数/runner与过期receipt负控须拒绝。

这是本机重建的来源消费结果，不冒称历史锁二进制身份或复用其运行证明。旧exact-binary锁保持严格比对，新模式不能成为失败后自动fallback。T-02给出明确生产/验证责任与必要fresh smoke；如无法闭合则回规划，不预先将下游换锁判为通过。不引入PKI/通用安装器/远端发布平台，生成器与pinned执行SDK继续分图，不为原exe缺失保永久计划证据。

固定源消费A→B不借主检出/.wt/生产者target或网络；生成器与runner可独立构建，生成产物、profile、源映射、报告对身份与业务结果交叉核。公开Run不以test harness为唯一入口。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-basic-authoring.md | G11专用角色/触发要求→明确纯视觉/层级/角色/相机三类有限作者入口、typed服务/原额/兼容 | 首波5入口能力，不冒称所有AG已支持 | AC-01..05 |
| SD-02 | add | docs/specs/auto-graphics/node-authoring-consumption.md | exact-binary旧锁仍严格；新增显式固定source-build的实际产物/记录/运行验证与迁移边界 | 不再靠历史exe永久保留才能运行新迁移内容 | AC-01/06 |

不修改历史语言仓Spec/账本；应用派生索引不构成交付门。

## 6. 测试设计

拟入口由T-03..06创建并在T-02冻结精确非零case集：pilot --test node2d_basic_gates，driver --test node2d_basic_flow -- --test-threads=1，verify_plan019.py --controls。旧node2d_pilot/node2d_lower_gates/node2d_g11_gates、node2d_flow/node2d_g11_flow和verify_plan016/017按真实变更选择；shared UI dialect/compiler须核其真实消费者，不凭实验目录豁免。当前所有新增用例未执行。

| V/S | AC / owner_stage / T | 实际输入→消费者与边界 | 独立oracle/负控关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 无角色纯场景、同组件多实例、父子group、改模块/资源/节点名→parser/binder/lower/Host/GPU | 无隐藏Player/额外VM；alpha与原数组序/祖先AND显隐、world(35,5)/(55,5)实际成立；错父/循环/错type/隐式solid反例红 |
| V-02/S-02 | AC-03 work T-04 | 原G06两布局、speed4/5、左右同时/无、Jump held/新沿/空中/中性→真实Auto与几何 | 独立离散f32 vy/位移/顶点/最终支撑；台阶vs坑道、无双跳/缓冲；Right-only/复用旧grounded/Rust替业务负控可区分 |
| V-03/S-03 | AC-03 work T-04/05 | G08主alt、clamp边界、surface resize/DPI、Pause/Step/Stop→角色+camera+GPU | same-tick身份、独立clamp/letterbox数值与实际条带；一tick落后/固定投影/假相机负控红；不含3D |
| V-04/S-04 | AC-04 work T-05 | Run/Pause/Step/Stop/Close20短轮、异常准备/非法ref与合法外部pin | 原model/handler/资源owner终态、Stop编辑文档不变，初始化后Start一次/新Run；未执行OS路由明确标gap，不重放绿覆盖首次红 |
| V-05/S-05 | AC-01/05 work T-02/05 | 初始化/走/跳/落/中性/相机与32/33、VM守护、试玩backlog原新对照 | 全Tick计费/step/resume/time可见，保原样本/守护与8ms口径；无静默提额，性能观察未定因不能称回归已解决 |
| V-06/S-06 | AC-06 work T-02/06 | 新source-build、实际工具/runner、冷A→B、源/锁/工具链参数/文件篡改/旧receipt→修复再Run | 固定实际来源+二进制hash+真实报告闭合；错身份/空case/旧snapshot/原锁fallback拒绝，精确恢复后同入口可用 |

实现自查→同一树独立上下文final review→merge冻结门；未指定人员/模型，本轮不新建agent/chat。review逐V/S/SD与五入口源基线核，不以parser绿替整链或自查替独立终审。仅本Plan新增/受影响定向集合、GPU重型门单实例，全AG留里程碑；不运行无关语言全套。

临时材料唯一D:/autostack/plan-artifacts/plan-019/auto-ui/，开工登记owner/用途/在途状态；外部consumer/SDK/target/报告显式此处。禁止attachments含md，不将探针/执行handoff/SD拷贝放仓内；必要永久回归测试/夹具归上述产品路径。merge前将命令/exit/数字/关键节选/identity折入本Plan§9，再删自有scope及缓存；具名未解决问题写owner/原因/关闭条件和登记表，跨仓同号不覆盖，未知目录不盲删。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 017真基线、只读两图依赖/原额及消费身份经有界三链先证；失败不降低分母 |
| AC-02 | work | 零角色视觉/层级组件与资源符号真实生效，改名/顺序/显隐/错误诊断V-01 |
| AC-03 | work | G06走跳/布局/源变体与G08同tick跟随/主alt/letterbox在真实Auto/GPU成立，V-02/03 |
| AC-04 | work | 原生命周期/owner20轮/初始化故障、旧016/017和真实UI消费者回归，V-04 |
| AC-05 | work | 原预算/VM/数值/节点边界及32/33负控、完整tick性能口径，V-05 |
| AC-06 | work | 公共准备/真实启动、固定source-build身份与A→B/坏件恢复闭合，V-06 |
| AC-07 | review | 独立逐AC/V/S/SD/原5入口要求/遗漏/格式/warning复审，绑定提交、零未处置finding |
| AC-08 | merge | 冻结定向门/有据复用、master实际delivery、两个新Spec只沉淀实际能力，归档导航准确 |
| AC-09 | merge | 10任务计数/guard清树分支空组/材料清理路径对账与登记、具名持有和恢复事实准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续用户启动，017实际交付 | 核当前Spec/源/SDK/依赖与018/901/014 owner，建唯一ui-019树；冻结五入口全部原变体/断言与数据 | AC-01 | [x] 2026-10-31：017 Spec a8b5b724/lower_g11 4f510686/load 4837d29c/ast 5cf76769/parse 9681e2be 与§4五hash逐一相符（主检出d85edbb）；018 drafting0/8、014 drafting0/11、语言901 executing（其auto-ui树lang-901有WIP crates/auto-ui-tools/src/rust_ui.rs，不接管）；SDK .sdk-source=681文件。ui-019组4树建成：auto-ui@d85edbb分支codex/plan-019-dev + 只读依赖auto-lang@98270a4f5/auto-web@f29c7d1/auto-down@c3464d0（detached）。五入口基线自ag-examples dbb4b75冻结至D:/autostack/plan-artifacts/plan-019/auto-ui/baseline/（21文件含4场景5变体2脚本对，G01 ff20607c/G03 b5ba7d62/G06main c2155ae3/G08main cf08e7d7与008§4指纹相符；layout-b与layout-a共用同一main.at c2155ae3，差异全在scene；verify_examples.py 00b29818相符）；B01×12/B03×6/B06×10义务源=verify_examples.py实读，B08×8义务源=008§5.3表+runtime_camera_host.rs/verify_plan875.py | AC-01 |
| T-02 | work | T-01 | 最多两接法三条完整链先证与成本/消费身份闭合，Plan§9冻结CLI/IR/typed服务/V-S精确集；不成立回规划 | AC-01..06 | [x] 2026-10-31：两既有端口接法=（1）scene-embedded project play（character/1、runtime_camera2d/1、behavior=null三形态同路径）+（2）纯场景scene play（probe源，无VM）。三链全真机通过（commit bbfee0e）：G01五入口GPU像素类+变体敏感性、G03 derive world (35,5)/(55,5)+祖先AND、G06走/落地/跳沿/apex/无双跳/坑道对照/speed4→5、G08 clamp/letterbox115px/同tick/alt506.67×380、成本恰30（第33计费调用fault 10）、相机app_start零额外计费。CLI=node2d-lower --profile scene-2d|character-2d|camera-2d --entry --out（scene.agscene+identity.json+closure/，脚本profile另有world.at）；IR=既有共享translate（Root标量r_<field>）；typed服务拼写=input.axis(Left,Right)/input.pressed(Jump|Enter)/refs.<char>.move_and_slide((dx,dy))/refs.<char>.grounded(q)/refs.<char>.support(q)；消费身份=场景behavior.source绑world sha/bytes+closure/逐模块sha+roles具名映射 |
| T-03 | work | T-02 go | 纯视觉/层级/0角色模式、结构化lowering/引用诊断/多实例，V-01 | AC-02 | [x] 2026-10-31：dialect+shape2d/group2d（混合块文法、circle/quad作用域符号）；lower_basic2d.rs组件参数含float/int/Material2D/Mesh2D四类typed端口；scene-2d拒绝character/solid/camera/trigger/model/msg/on/if，同view重名拒绝，circle非等尺寸拒绝，未知prop/资源定位拒绝；组件多实例+参数敏感性+改名不改几何（gates 9例全绿，bbfee0e+9096bd8） |
| T-04 | work | T-03 | Auto走跳/grounded/输入、可选camera主alt、同tickGPU，V-02/03 | AC-03 | [x] 2026-10-31：driver node2d_basic_flow 13例全绿（9096bd8）：walk/land逐tick f32独立oracle（rest=face+半宽+clearance 0.002）、jump沿/apex离散界/空中沿弃用不缓冲、layout-b坑道真掉落vs layout-a台阶阻挡、fast位移差恰30·dt、G08主/alt投影+letterbox+同tick发布+Pause/Step/Stop、G01五变体GPU类、G03世界和/显隐/文档不变 |
| T-05 | work | T-04 | 生命周期/20轮owner/原额负控/旧兼容及真实消费者，记录backlog对照与性能边界，V-04/05 | AC-04/05 | [x] 2026-10-31：20轮Run/Stop全fresh起点+第21轮正常（owners逐轮释放）；成本恰30负控（31/32过、33=fault 10）；旧链回归全绿：driver node2d_flow 6/6+node2d_g11_flow 15/15、verify_plan016 OK controls（016字节冻结保持）、verify_plan017 PASS、dialect/compiler node2d_pilot 6+6；backlog观察=旧017根因项（D-04），本plan生成源为同型triggers/character调用且未触及pump路径，无新写面回归证据；性能边界=离屏GPU用例内实测（13例17s内完成），桌面持续流畅不在本轮口径 |
| T-06 | work | T-05 | source-build实际身份/公共消费/冷A→B/坏件恢复、必要格式/警告/SD候选与移交折入Plan，V-06 | AC-01..06 | [x] 2026-10-31：verify_plan019 --controls 19组PASS=13产品再生字节等价+身份绑定（场景sha/closure逐模块sha/fixture零漂移）+变异敏感性+character-2d拒相机诊断+cold A→B四组（B仅持closure/再生字节等价、篡改模块被manifest sha捕获、精确还原同入口复用）；--build-record/--verify-record绑定实际generator二进制sha（2c6041c415f1）+Cargo锁+rustc+13产品hash，漂移record负控拒绝；driver侧scene.source绑定篡改拒绝；cargo fmt全触面+全部gate复绿（9096bd8）；SD-01/SD-02候选定稿见规范增量表（与本§9冻结一致） |
| T-07 | review | T-06 | 同树独立final逐AC/V/S/SD及旧断言审查，finding回work，pass列merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结源/依赖/输入、定向并集或严格有效复用后合master | AC-08 | [x] 2026-11-01：基线=final pass@afb4ef5（旧hash）→rebase映射f765213/d5f0bc0/8209713（range-diff 4/4全等）；901三提交仅触auto-ui-tools非本Plan依赖图，冻结有效；git merge --ff-only → master tip c93adb7c25ec2278eb4aaad41c6e4fe25549c22 无merge commit | AC-08 |
| T-09 | merge | T-08 | 两Spec/来源/归档/导航，折叠证据至Plan，不写派生JSON或外仓Spec | AC-08/09 | [x] 2026-11-01：SD-01/SD-02两Spec落地landed commit（sha 0dc10c47…/f6689bf1…，§9 merge收据）；Plan归档docs/plans/archived/019-…md status: archived（提交898b0f1，内链按归档父目录修复5/5解析）；证据已全部折入本文件（§8行+§9收据，命令/exit/计数/identity）后删plan-019 scope | AC-08/09 |
| T-10 | merge | T-09 | fresh guard/clean/祖先核后清树分支空组，逐全部实际输出路径对账，删自有plan-artifacts并刷新登记/完成hash | AC-09 | [x] 2026-11-01：fresh guard全4树clean（ui-019/auto-ui+三只读依赖）；git worktree remove×4+branch -d codex/plan-019-dev(was c93adb7已landed)+空组rmdir；plan-artifacts/plan-019/全删(67文件231K)并刷新.retention/policy.json（auto-ui/PLAN-019 cleaned，无具名持有，document指向归档文件存在）；主检出non-docs clean，其余docs WIP为其他owner保留。交付后观察：本仓为库/研究仓，无驻留生产二进制/daemon/web bundle交付面（017同例）；pilot/driver targets随worktree消亡即政策；gallery内容侧(008)消费本交付走其自身source-build记录。批量回归due核：auto-ui无.last-batch-regression.json（该due程序属auto-lang），019非5倍数，不适用。 |

## 9. 复审记录

2026-10-31 / stage: new / plan_id: auto-ui/PLAN-019 / plan_revision: 1 / outcome: pass（首波合同待审，T-02为后续实施内有界门） / next: 后续用户启动后T-01/T-02。首次尝试分配018时独占重核发现另一owner已取号，未创建冲突Plan；重核后分配019/下一020。仅设计/计划/导航/号源；无实现树、代码/资产、构建/GPU或agent。所有新命令/样本均未执行，不把历史回执和试玩当新能力证明。

2026-10-31文档预检：0..10节、10任务/9AC/6组V-S/2SD、全部本地Markdown链接、跨仓依赖和owner/终审/合入分工、取号唯一与drafting0/10通过。总设计逐项覆盖当前21入口（1已迁+20待迁；HUD另列），首波检查义务12+6+10+8=36，计数与源工具控制流/回执交叉核；新profile和消费机制仍属T-02待实证假设，不写成通过结果。仅文档验证，不是实现或独立final review。

stage: goal | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-019/auto-ui | commit: d85edbb（树基，尚无代码提交） | next: work

2026-10-31 / stage: work / plan_id: auto-ui/PLAN-019 / plan_revision: r1 / outcome: pass(T-01) / worktree: D:/autostack/.wt/ui-019/auto-ui / branch: codex/plan-019-dev / base_commit: d85edbb0988f1cb91e3ec6a7e21ba4d872bdfb02 / code_commit: 无（T-01为核验/建树/冻结，无代码改动） / dependency_revisions: auto-lang 98270a4f5、auto-web f29c7d1、auto-down c3464d0（均detached只读）、SDK .sdk-source 681文件 / task_ids: T-01 / evidence: D:/autostack/plan-artifacts/plan-019/auto-ui/baseline/（21文件+tools_verify_examples.py，hash见T-01行）；owner核=018 drafting0/8、014 drafting0/11、901 executing(lang-901树WIP rust_ui.rs不接管)；主检出仅docs/**簿记改动无代码WIP / blockers: 无 / next_responsible_role: work（T-02三链先证）。授权沿/goal整链授权（work/review/有界修复/merge）。

2026-10-31 / stage: work / plan_id: auto-ui/PLAN-019 / plan_revision: r1 / outcome: pass(T-02..T-06 全work任务完成，execution_ready) / worktree: D:/autostack/.wt/ui-019/auto-ui / branch: codex/plan-019-dev / base_commit: d85edbb / code_commit: bbfee0e64194856aaa06a39cd09ec76f4f805b19 + 9096bd82425025821f4e8cbb6c3cbdde474a47bf / dependency_revisions: auto-lang 98270a4f5/auto-web f29c7d1/auto-down c3464d0只读、SDK .sdk-source 681（prepare_sdk --from-snapshot PASS）/ task_ids: T-02,T-03,T-04,T-05,T-06 / evidence: 见§8各任务行；测试计数=pilot gates 9+g11 gates 10+legacy 9、dialect 6、compiler 6、driver node2d_basic_flow 13（单线程，含真GPU）、driver node2d_flow 6+node2d_g11_flow 15（旧链回归）、verify_plan016 OK、verify_plan017 PASS、verify_plan019 --controls 19组、build-record roundtrip+漂移负控。

T-02冻结合同（后续波次按此消费，修改须回本Plan）：
- CLI：node2d-lower --profile legacy|g11-2d|scene-2d|character-2d|camera-2d --entry <dir>/main.at --out <dir>。输出：scene.agscene（schema/6，scene-2d behavior=null，character/camera分别绑autographics.character/1与autographics.runtime_camera2d/1及world sha/bytes）+identity.json（generator/api/module shas/closure清单/roles具名映射）+closure/<module>.at；脚本profile另有world.at。
- 作者面：shape2d name{kind:circle|quad, position:(x,y), size:(w,h), material:resources.<color>, visible:bool?}（circle要求size.x==size.y）；group2d name{position?, visible?}{混合块children}（仅scene-2d）；character2d name{position, material}（恰一，盒0.8×1.0，root view或组件view）；solid2d name{position, mesh, material}（仅脚本profile root view，声明序绑定）；camera2d name{target:refs.<char>.actor, bounds:(min_x,min_y,max_x,max_y), view_size:(w,h)}（camera-2d恰一，root view）；组件节点=参数(float/int/Material2D/Mesh2D)+纯view；scene-2d拒绝model/msg/on/if/角色/solid/相机/trigger。
- 服务拼写（共享translate映射）：input.axis(Left,Right)→(ag_action_held(1)-ag_action_held(0))（held计费2）；input.pressed(Jump)→ag_action_pressed(2)/Enter→3（计费1）；refs.<char>.move_and_slide((dx,dy))→ag_char_move(dx,dy,0.002)（26）；refs.<char>.grounded(q)→ag_char_grounded(q)/support→ag_char_support(q)（各1）。生成源根model为r_<field>标量；作者fn原文pooling（纯函数，不含服务调用）；Life.Start并人app_start（生成绑定先于作者体）、Life.Tick即app_update体。G06型完整tick成本=1+2+26+1=30（实测恰30，第33计费调用fault 10）；相机配置仅在app_start（零tick成本）；纯场景无VM。
- 场景序：id 1起对象/组共享命名空间、声明序深度优先、对象数组=2D绘制序、渲染端alpha升序/等alpha原数组序/不透明先绘为既有渲染器行为（G01链实测保持）；group仅同维平移+可见性继承（derive逐层f32求和+祖先AND）。
- 消费身份（SD-02 engine侧机制）：场景behavior.source绑生成源sha/bytes；closure/为输入闭包唯一消费物（B仅持closure再生字节等价，实测四组）；篡改由manifest sha捕获、精确还原即恢复；--build-record绑定实际generator二进制sha+Cargo锁+rustc+13产品hash，漂移拒绝。旧exact-binary锁不弱化，新模式不构成fallback。

边界与移交：GPU像素类oracle为类级（相对通道规则+计数/差分），默认2D呈现约1px/unit未作逐像素位置断言（G03位置oracle=derive CPU准入、G08=host clamp/letterbox精确数学）；生成源块注记、identity/场景JSON形状为冻结面（再生等价控制）。未执行OS抢焦点路由（原边界保持）；试玩backlog=D-04既有债务不因本轮关闭。next_responsible_role: review（T-07同树独立final）。

stage: goal | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-019/auto-ui | commit: 9096bd82425025821f4e8cbb6c3cbdde474a47bf（bbfee0e+9096bd8两提交） | next: review

stage: review | review_scope: final | reviewer_identity: 独立子代理上下文（同树 afb4ef5 前为 9096bd8；限制=同宿主会话派生的fresh上下文，非人工） | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-019/auto-ui | branch: codex/plan-019-dev | reviewed_commit: 9096bd8 | base_commit: d85edbb | dependency_revisions: auto-lang 98270a4f5/auto-web f29c7d1/auto-down c3464d0只读均clean、SDK 681 | spec_inputs: SD-01/SD-02 delta文本（目标spec尚未发布，merge所有） | acceptance_results: AC-01..06 pass（独立复核：基线指纹重算、104点基线对比100/104、全部门禁亲自重跑全绿：pilot 9+10+9、driver 13、controls 19组、016 OK、017 PASS、dialect/compiler 6+6、driver旧链6/6+15/15）；AC-07=本轮；AC-08/09 merge pending未声称 | pending_closeout_gates: merge冻结定向门、Spec沉淀、归档/清理 | findings: F-R1-01(P1,v-position语义偏离原B01-11规则：原为蓝移(40,0)消除重叠而非红覆盖蓝)、F-R1-02(P2,v-color重着色值(0.1,0.9,0.2,0.6)≠原(0.1,0.8,0.2,0.5))、F-R1-03(P2,128上限将组重复计数)、F-R1-04(P2,walk oracle未读initial绑定告警)、F-R1-05(P2,--verify-record未核Cargo锁) | evidence: D:/autostack/plan-artifacts/plan-019/auto-ui/review-probe/（cmp_baseline.py等） | next: work修复F-R1-01..05

stage: goal | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-019/auto-ui | commit: 9096bd8 | next: work

2026-11-01 / stage: work / plan_id: auto-ui/PLAN-019 / plan_revision: r1 / outcome: pass(R1修复) / worktree: D:/autostack/.wt/ui-019/auto-ui / branch: codex/plan-019-dev / base_commit: d85edbb / code_commit: afb4ef5318846095ce74a1af197df352e726ec37（R1修复，含暂存目录勘误amend） / dependency_revisions: 不变 / task_ids: T-03,T-04（受影响面） / evidence: F-R1-01 v-position蓝移(40,0)按原B01-11规则重做+driver改为原oracle断言（blue保持容差内+red严格增加）；F-R1-02 deep_green=(0.1,0.8,0.2,0.5)原值；F-R1-03 节点上限改objects+groups单计；F-R1-04 未用绑定删除（告警消）；F-R1-05 --verify-record先核workspace_lock；全部五组g01产品再生（resources模块sha随资源值更新），g01-v-*误名目录已清理并amend。修复后门禁：pilot gates 9/9、driver node2d_basic_flow 13/13、verify_plan019 --controls 19组PASS、build-record roundtrip PASS（新generator sha 8fcf84d990b7绑定）、树clean@afb4ef5。受影响再复核已交原独立reviewer（范围=五finding+共享不变量），其余证据沿用。 / blockers: 无 / next_responsible_role: review（受影响再复核→final pass）

stage: review | review_scope: final | reviewer_identity: 独立子代理上下文（同上轮限制） | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-019/auto-ui | branch: codex/plan-019-dev | reviewed_commit: afb4ef5318846095ce74a1af197df352e726ec37 | base_commit: d85edbb | dependency_revisions: 不变（只读三仓clean） | spec_inputs: SD-01/SD-02 delta（文本与实际行为一致、无越界声称；目标spec merge所有） | acceptance_results: AC-01..06 pass（沿用R1证据+受影响复核）、AC-07 pass（本审：触及集18文件与修复声明一致、101点基线对比0失败、节点上限公式=runtime公式、锁漂移负探针拒绝、五组g01 identity绑定新resources sha e0caad8d冻结==再生、受影响套件亲跑pilot 9+10+9/driver 13/controls 19/fmt clean零新告警）、AC-08/09 merge pending | pending_closeout_gates: merge冻结定向门/两Spec沉淀/归档导航/树清理与plan-artifacts删除 | findings: 无（F-R1-01..05全部关闭，无新问题） | evidence: D:/autostack/plan-artifacts/plan-019/auto-ui/review-probe/（复审探针） | next: merge

stage: goal | plan_id: auto-ui/PLAN-019 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-019/auto-ui | commit: afb4ef5318846095ce74a1af197df352e726ec37 | next: merge

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-019:r1 / outcome: pass(进行中→archived 前各检查点如下) / worktree: D:/autostack/.wt/ui-019/auto-ui / branch: codex/plan-019-dev / prepared: reviewed基线afb4ef5+SD-01/SD-02两Spec于worktree提交e37d1ec（docs-only descendant，实现/依赖零变化）；wt-guard clean / landed: master前进（901三提交b565675/bde25c0/e1fe4ca，仅触crates/auto-ui-tools，不在本Plan依赖图）；rebase后旧→新映射 bbfee0e→f765213、9096bd8→d5f0bc0、afb4ef5→8209713、e37d1ec→c93adb7，git range-diff 4/4全等（安全重写证明）；主检出 git merge --ff-only codex/plan-019-dev → master tip=c93adb77c25ec2278eb4aaad41c6e4fe25549c22（无merge commit） / canonical_verified: docs/specs/auto-graphics/node2d-basic-authoring.md sha256 0dc10c476f8739fabaf0b7f1b5e88679a775752733d53608944961f3e90a8498、docs/specs/auto-graphics/node-authoring-consumption.md sha256 f6689bf162eb992830dfa3a43f16168f2d1ca6b7470128ebbb51579df75667e3，均在landed commit c93adb7；main smoke：cargo check -p auto-ui-dialect PASS（对当前auto-lang master解析） / next: archived→cleaned

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 纯场景/层级/角色/相机准确typed端口与CLI/profile映射 | T-02用共同语义冻结；不超出五入口能力，不另造脚本语言 |
| D-02 | 固定源码重建如何闭合实际消费身份而不绕旧锁 | T-02冻结显式新版合同/验证反例；不能成立则needs_replan，不自填绿receipt |
| D-03 | 当前018/901/014依赖/解析端口与共享owner | T-01实际SHA和diff核，不要求其全完成，不接管WIP |
| D-04 | 试玩backlog观察根因/可比原新测量 | T-02/05确认环境与真正触面；不得提前声称已修或直接加阈值 |
| D-05 | 独立终审上下文与GPU窗口 | T-06按届时授权交接，未取得证据不标reviewed |

本轮仅规划，008须等019实际交付；后面波次不预取编号。

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-019:r1 / outcome: pass / consolidated: PLAN-019:r1 各检查点全部实证——prepared=afb4ef5基线+SD两Spec@e37d1ec（guard clean）；landed=ff-only master tip c93adb7（range-diff 4/4全等安全重写，901仅触auto-ui-tools不涉本面）；canonical_verified=两Spec sha 0dc10c476f8739fabaf0b7f1b5e88679a775752733d53608944961f3e90a8498/f6689bf162eb992830dfa3a43f16168f2d1ca6b7470128ebbb51579df75667e3@c93adb7+main smoke cargo check -p auto-ui-dialect PASS；archived=docs/plans/archived/019-autographics-node2d-scenes-character-camera.md status:archived 提交898b0f1（10任务/9AC计数一致、内链5/5解析、导航指向本Plan）；cleaned=4树worktree remove+branch删除（was c93adb7）+空组rmdir+guard全clean+plan-artifacts/plan-019（67文件231K）删除+registry .retention/policy.json刷新（cleaned/无持有/document路径实证存在）+收尾提交3b65d3a；交付后观察=库/研究仓无驻留生产二进制交付面（017同例），content侧008按其source-build合同消费本交付；批量回归due=auto-ui无该程序（属auto-lang），019非5倍数不适用。10/10任务，AC-01..09全部关闭。