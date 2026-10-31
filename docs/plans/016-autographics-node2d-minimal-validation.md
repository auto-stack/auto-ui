---
plan_id: PLAN-016
status: executing
feature_name: AutoGraphics Node2D最小真实执行验证
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-31
plan_revision: 3
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-013]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-pilot.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, auto-ui-compiler, autographics-auto-driver, autographics-node2d-pilot]
---

# PLAN-016：Node2D最小真实执行验证

## 0. 变更摘要

在auto-ui验证一个Player、两个同定义但状态独立的Hazard、一个可销毁Coin。真实跨文件use、node/model/view-2d/on、Life系统事件、匿名msg/前导点、具名资源/节点引用，经过共同领域语义与真实Auto执行进入现有AG图形路径。成功必须包括实际运动、事件到正确实例、Coin真实销毁及GPU变化，不是parser或日志演示。

本轮用户只授权规划，保持drafting/0 of 10，不建树、改实现、创建agent或启动构建。当前013正在唯一树实施；016实施硬等013实际交付，避免共享AG资产/Host/assembly写面竞争。这是本Plan的串行基线选择，不是HUD成为Node语言的架构前提。

## 1. 目标

1. 一个公共研究入口读取普通模块工程，而非固定示例名或拼接成单文件绕过模块语义；model/view-2d/on保持本实例局部引用。
2. Life.Start一次/新运行实例，Life.Tick(dt)按世界固定时钟；系统事件不要求msg声明，业务事件仍msg { Collect(Key) }和.Collect(key)。
3. 两个Hazard同字段名、不同参数且同一世界运行，状态、handler receiver和订阅互不串线；可共享不可变资源。
4. Player真实输入/碰撞移动；Coin触发发送有类型事件给父，父model改变条件声明，实体/trigger真实销毁、重复消息幂等；Stop/fresh Run恢复。
5. 新语法仅显式研究入口启用，旧widget/旧AG profile不自动改写或升级；提供真实图形与实例状态的可区分证据，形成局部研究Spec。

非目标：完整G11胜负/三Coin/危险伤害/终点/Enter新局，view-3d实现、UI/2D/3D混合嵌套、多呈现选择器与view-portrait/view-widget、独立view文件导入、完整资源导入/编辑器往返、公共Node SDK/默认替换、全量widget退役、Vue/Iced组件迁移、ECS/并行多VM、完整CPC、ACE/AOT/HMR/新平台、预算/数值语义改造、Gallery内容或引擎锁升级。Hazard在本切片只验证独立运动和状态，不增加失败逻辑。

## 2. 架构方案

普通Auto模块工程 → 既有UiDialect/Core模块端口的显式Node2D研究模式 → 校验领域模型/引用/事件类别与实例receiver → 既有Auto编译/执行服务 → Node实例状态、绑定差量与逻辑时钟 → AG公开服务/资源/运动/trigger/renderer → 真GPU离屏画面。

节点条件为false只销毁对应实例/触发器；渲染数据同步不意味着全世界一笔玩法事务。公共语义校验和简单实例表足够，初期不做ECS、不要求一Node一VM。Rust可做调度、类型化桥和图形服务，不能实现计分/收集规则/巡逻公式或用source关键词识别结果。

唯一实施/顺序复审树拟D:/autostack/.wt/ui-016/auto-ui、codex/plan-016-dev。只改本仓；必要只读依赖按env→同组兄弟→主检出既有规则定位并固定来源，禁止链接。若工具必须有参与仓checkout，先记录只读来源/唯一树约定，不擅自改其他仓代码。013的实现与owner不接管；014如进入共享UI文件，按实际diff约定短窗，不把其全量完成加成隐式前置。清树须Git Bash wt-guard clean，保恢复收据。

## 3. 技术栈

已存在：auto-ui-dialect/src/{grammar/widget.rs,declarations.rs,ast.rs,codec.rs,validation.rs}，auto-ui-compiler/src/ui_ast.rs的parse_ui/lower_ui/resolve_use_module，auto-ui-runtime/src/ui/{vm_bridge.rs,handler_codegen.rs}作为机制参考；AG driver的authoring/{behavior.rs,play.rs}、behavior/assembly.rs及src/lib.rs的Session，现有对象/触发/角色与GPU基础。

拟新增（均未创建）：crates/auto-ui-dialect/tests/node2d_pilot.rs、crates/auto-ui-compiler/tests/node2d_pilot.rs；experimental/autographics-node2d-pilot/独立workspace（入口/实例adapter/fixtures）、experimental/autographics-auto-driver/tests/node2d_pilot.rs如实际consumer适合，或pilot自有tests/node2d_flow.rs；tools/verify_plan016.py及fixtures/minimal/{main,resources,types,player,hazard,coin,patrol}.at。T-02冻结精确包名/测试位置/命令，不制造两个相同运行入口。

优先复用共同领域AST/模块解析而非新lexer/parser/解释器；node可映射共同组件表示，view-2d增加明确域，不复制一套UI AST或把图形字段塞回Core AST。当前AG固定SDK和生产UI编译链不同，具体接法必须T-02整链证明；不能直接把SDK换成当前语言tip。

## 4. 需求分析与背景调查

用户连续确认：node统一定义、view/view-2d/view-3d直接呈现、Life系统事件、匿名msg、.Collect与本实例成员；默认内聚model/呈现/on。最后直接授权“规划最小验证计划，一个Player、两个独立Hazard和可销毁Coin，放auto-ui，暂不实施”。没有实施、建树、agent/模型、性能预算提高或自动续跑授权。

已读[共同作者要求](../design/autographics/declarative-authoring-model.md)、[G11目标草图v4](../design/autographics/g11-2d-target-source-sketch.md)、语言仓[AG overview](../../../auto-lang/docs/specs/auto-graphics/overview.md)、[行为profile](../../../auto-lang/docs/specs/auto-graphics/behavior-api.md)、[运行对象](../../../auto-lang/docs/specs/auto-graphics/runtime-objects.md)、[绑定工程](../../../auto-lang/docs/specs/auto-graphics/behavior-project.md)和[Design47](../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md)。旧Spec描述实验现状，不声称本Plan能力已交付。新局key/三维/全事件生命周期只作为未来目标，不偷入本切片。

只读基面auto-ui 3e61d03（本轮文档前HEAD）；代码观察SHA256：

| 来源 | SHA256 / 事实 |
|---|---|
| dialect grammar/widget.rs | a08fa031f43455b367f72b20afbcdc920856ccef26941153ae918bc41e8ac2c6；widget入口/点前缀handler已有 |
| dialect declarations.rs | e816dfb370bb5768673b388f0dbb1bc4017e662598d70385591132eb79b9ad67；匿名msg已支持，不是新语法发明 |
| dialect ast.rs | 1e75439a5699057ac37e91f346b71c8080c9cbe1220b048f465e9fb0b145aed6；单view/named views不能冒充view-2d |
| compiler ui_ast.rs | 03f7dfd7f5888ef1e1724b06d86e0bd397242f62915f7255b38d698ceb7a510e；已有域解析与use定位 |
| runtime vm_bridge.rs | 6c4c320e14cd8ae235d1d637594daa238b07564e986eedc6f20ad9826bfb8775；存在child状态与merged根字段兼容路径 |
| AG authoring/behavior.rs | 96e3771e293c019c51e50e970ec4182e097c7bbbd46757f40a829875ea6a3306；当前import拒绝 |
| AG src/lib.rs | 4db69f9748a97c88cc02e4230146dc026fb730a0175701799b7628bc96c56b6c；root签名与组合源码Session |

执行时核013最终代码/依赖/评审与冻结门并重绑实际身份，不固定上表旧HEAD作为未来delivery。当前无ui-016树；主检出有其他owner的013/014/主题文档与号源WIP，不能覆盖或混提交。取号本轮用本地独占锁，兼扫活动、archived、archive，最大015而.next-id仍015，分配016并纠正下一号017；不影响语言/Gallery号源。

## 5. 详细设计

### 5.1 进入门与有界整链接法

G-01：013独立final review、冻结定向门、实际master交付和准确恢复/清理记录已核，再建唯一树；本轮不等它来完成规划。独立研究adapter是后续工作范围，不声称生产ACE/AutoUI融合获放行；旧ACE成熟度门保持。T-01固定本仓/依赖SHA、公共端口/锁/SDK身份/实际写面和其他owner窗口。

T-02最多比较两种既有端口接法，必须均复用共同语法/Core解析和真实Auto执行：(a)生产UI模块/编译服务执行有限Node实例；(b)共同领域模型经局部研究adapter连接已固定AG运行服务。不是两套新实现。写新attachments/016/work/node2d-contract.md：输入工程→模块图→领域表示→实际代码/handler→实例状态→AG对象→GPU的逐段文件/符号/owner、CLI/精确测试、费用、时序与故障出口。原型Start+一次Tick+两个实例+一次真实事件必须先跑通，才能继续T-03..06。

若必须改其他仓语言/SDK语义、复制parser/VM、做通用ACE融合、把Auto规则放Rust或无法在本合同预算内执行，T-02记录原输入和阻点，needs_replan；不能把no-go研究当本Plan全部验收通过。源码拼接只可为编译器正常lowering的内部产物，不能用文本替换/去掉use模拟模块解析。

### 5.2 具体语义与最小输入

- 根Node的model/条件view-2d/on同作用域。普通use导入Player、Hazard、Coin、纯patrol函数及resources；缺模块、错导出/类型、重复局部绑定应有源码位置诊断。冻结闭包每文件字节与实际依赖关系，运行不偷读变化文件；不新增自动联网/插件加载。
- 一个Player，静态ground，两个Hazard同类型不同props，一个Coin、一个camera。Player从(2,1)向右2单位/秒；Coin在行走线上约(4,1)。Hazard置于不碰Player的不同y层。Hazard字段同名phase/ticks/starts，初始相位和速度不同；另有根同名phase作为哨兵，不能字段展平共享。每实例结果须可从真实Auto状态读回并与渲染交叉核。
- Life.Start/Tick是系统事件，msg { Collect(Key) }是本Node业务事件，.Collect绑定具体父实例。不按handler名字尾部混淆Life.Start和自定义.Start；一个补充夹具证明两者独立。Life.Frame定义为呈现帧节奏事件，仅留后继；Frame与view-3d均不纳入本Plan实现、夹具或验收，不用空处理器冒充支持。
- Life.Tick采用1/60固定dt；每世界只采样一次输入；每存活实例每逻辑tick更新一次，与render次数/多视图无关。Start在必要引用/资源就绪后一次/新Run；Pause不更新，Step一个中性tick，Resume不重复Start；fresh Run回初态。Mounted等旧生命周期维持原入口，不在本Plan新做完整UI挂载体系。
- 根coin_present是Coin存在的唯一玩法状态；Collect中由Auto检查/改false，绑定差量销毁真实对象/trigger并释放订阅。不能用visible=false、移动场外、Rust删除规则或只改日志代替。重复Collect对该局幂等；旧代事件在fresh Run不能修改新根，不能发给碰巧同名的新实例。
- 有类型资源symbol和actor引用，不要求作者写数字对象/槽/资源ID。T-02冻结有限公开actor端口与refs表示，错误引用在进入执行前拒绝。资源用程序化小图元/单色材质，不加PNG/GLB导入和新数据格式；共享合法资源不等于共享model。
- 候选tick顺序固定为输入→Tick→变换/运动结果→触发→有序业务事件→条件差量→呈现；T-02给出当前实现可达的准确顺序。此新研究入口的顺序不能偷偷改旧N→N+1 profile。若需换顺序影响AC，先修订合同。

本轮命名修订：本切片统一写view-2d；view、view-3d、view-portrait、view-widget及内部混合嵌入的总体设计见[多呈现机制](../design/autographics/multi-presentation-selection.md)，不纳入本Plan实现或验收。Node直属呈现与内容树内嵌入按位置区分；本Plan只做前者。T-02须冻结view-2d领域标记的解析/lowering，不改变普通表达式减号语义，不复制一套词法/表达式解析器。

### 5.3 资源、预算、故障与所有权

有限Windows x64/真实离屏GPU研究入口，只覆盖上述小工程；工程源码闭包合计<=256KiB、可信桥<=96KiB、合计<=352KiB，沿现有节点/8实例与8资源身份/32MiB raw、CPU/GPU各64MiB上界，不能按模块各获一份额度。没有任意模块数/无限容器的承诺，也不临时加语法深度上限掩盖问题。

保留旧API计价和旧profile字节/行为，不给导入helper免费旁路。新pilot在一个世界tick共享图形服务32credits；不得每Node reset池获取32倍额度。既有VM分片1000steps/2ms、累计100000steps/120resumes/250ms的执行守护继续适用；多handler须报告完整tick累计工作和宿主总时限2s，不能分拆绕过。8ms pump测量目标met/missed/unmeasured另列，不以2s代性能目标。T-02校准真实运动/输入/两Hazard/收集峰值；候选1snapshot+26move+2Hazard移动+1next+1pair+1despawn=32仅算式，非可行性证据。事件/绑定/Auto操作的真实CPU和桥调用一并实测；不足回规划，不降采样/丢事件/静默提额。新资源/binding初始化具体成本也须证明。

不改变语言数值规则，finite f32边界沿固定执行服务，身份/计数保持整数。测试比较逻辑计数精确值、浮点位置按独立预期与冻结容差；不承诺未来AOT/跨GPU逐位相同。不得用容差掩盖实例错接、错误分支或没有运动。

生命周期域是pilot世界/Run→Node实例→订阅和运行对象；代码/不可变资源可由上级共享。正常销毁、初始化失败、运行异常、Stop/关闭与旧代迟到路径做必要定向证据。保留合法外部资源pin的寿命，最终释放后看weak owner/实例和订阅，不要求显卡驱动缓存归零。单组件错误的局部隔离不扩成全世界分布式事务；本Plan只冻结pilot错误/终止策略，不改旧profile故障语义。

### 规范增量

新Spec归本仓，历史auto-lang Specs只读引用，不批量迁移或写派生specs.json/index.json。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-pilot.md | 目标草图→有限显式研究入口的实际语法/模块/实例/事件/时钟/资源/费用与未支持面 | 将真实验证结果与正式产品融合区分 | AC-01..06 |

## 6. 测试设计

以下全部是拟建入口，未运行：T-03建立dialect/compiler `--test node2d_pilot`；T-04..06建立pilot flow测试和tools/verify_plan016.py。建议命令形状为cargo test -p auto-ui-dialect --test node2d_pilot、cargo test -p auto-ui-compiler --test node2d_pilot及cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --test node2d_flow -- --test-threads=1；python experimental/autographics-node2d-pilot/tools/verify_plan016.py --controls。实际包/路径/选择数T-02冻结，0selected、漏case、旧报告或错runner必须拒绝。

| V/S ID | AC / work task | 输入→实际入口 | 区分oracle与关闭条件 | 边界 |
|---|---|---|---|---|
| V-01 / S-01 | AC-01/02 T-02/03 | 多文件node/view-2d/Life/msg与导入helper；缺模块/错导出/资源错型/坏actor ref/重复绑定→真实解析/绑定/编译 | 合法真实运行；坏例执行前定位到模块/行/符号；改变patrol函数实际轨迹改变，改未用资源不改玩法；不能只测AST | 只所列Node2D有限子集，不承诺全部语言/模块循环形式 |
| V-02 / S-02 | AC-03 T-04 | H1/H2同字段名、不同初值/速度、根phase哨兵；一固定输入跑120ticks | 每个starts=1、ticks=120，根哨兵不变；各phase/位置按独立f32计算匹配；中途只改H1合法参数/事件，H2不变，GPU两个ROI各按预期移动；共享状态/错receiver控制必须红 | 不以两个不同类型掩盖同类型实例问题 |
| V-03 / S-03 | AC-04 T-04/05 | Player穿过Coin、重复Collect，事件投递到两个父实例的独立补充夹具 | 指定父coin_present=false，其他父保持true；Coin实例/trigger/订阅不存在，后续通过旧位置不再新触发；同位置放负控只有hidden仍判失败；GPU Coin ROI消失且地面/其他对象保持 | 一Coin主例；补充两父只验证receiver，不扩多场景产品 |
| V-04 / S-04 | AC-03/05 T-04/05 | Start、Pause/Step/Resume、反复绘制、Stop/fresh Run、旧事件、Life.Start对自定义.Start | Start一次，暂停计数不变、Step恰+1中性输入，绘制无tick无状态推进；fresh初态与旧代隔离，系统/用户同名事件不混；20轮后实例/订阅/VM owner按约释放 | 实测窗口/设备限制如实记录，不做全HMR/多窗口 |
| V-05 / S-05 | AC-06 T-02/06 | 真输入→Auto→服务→GPU、32/33与CPU守护负控、冻结工程A→B独立根/locked消费 | 原额/完整tick统计、33拒绝且无半执行宣称；B实际运行同输入/结果且不借生产者源码；错误源/缺模块/错身份→拒绝→恢复后真运行；旧受影响样本保持 | 仅pilot消费包，不产Gallery ZIP/锁 |

每V均含正常和区分性负控；原始Auto状态/运行对象/订阅观察与GPU像素形成不同层的证据。预期位置/计数/输入由独立小型oracle计算，不复制被测handler或仅读其自报success；漏绘/静止/假despawn/错实例/旧报告要能检出。T-02冻结合法范围、像素ROI/容差、实际选测ID与超时。每运行路线<=120ticks，20轮生命周期短例；不以手工“看起来正确”或日志代替真实执行。

用户未指定实现者/独立审核者或模型。本Plan规定work→同一权威树独立上下文final review→merge冻结门顺序；本轮不新建或联系agent/chat。review pass必须记录merge-owned门pending；不得用同会话自查冒称独立终审。

按既有AG测试裁定，只跑本Plan新增/受影响命名case与必要负控；不为合入跑全AG、全Gallery或根无关t/tf/tv/tu/docs_gen。若T-02发现必须改变通用UI既有行为/生成器而非显式pilot隔离路径，需先修订触面和门禁，不能借AG例外跳过原消费者义务。OS抢焦点自动化暂停；实际GPU/重型门单实例，headless无法覆盖的必需项记gap。raw源码快照/runner/log/PNG本地D:/autostack/plan-artifacts/auto-ui/plan-016；attachments只提交md摘要/合同。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 013真实基线、唯一研究入口/模块→执行→GPU链及资源预算先以T-02冻结；无第二解释器/未批准语言仓写面；V-01/05 |
| AC-02 | work | node/view-2d/Life与匿名msg/前导点、use、资源/actor引用真实生效；正常/诊断和改helper敏感性，V-01 |
| AC-03 | work | H1/H2同定义不同实例状态/receiver/订阅隔离；Start/Tick实际调度与暂停绘制语义，V-02/04 |
| AC-04 | work | Player真实运动/接触→正确父事件→条件差量真实销毁Coin；重复/错receiver/假隐藏负控，V-03 |
| AC-05 | work | Stop/fresh/旧代、正常及失败owner终态20轮；合法共享资源寿命准确，V-04 |
| AC-06 | work | 120tick真实GPU、完整预算/守护负控、独立消费A→B与损坏恢复，旧profile/受影响UI路径保持；V-05 |
| AC-07 | review | 独立重建AC/V/S/SD和模块/实例数据流、遗漏/延后/workaround/格式/warning检查，无未处置finding；merge门pending |
| AC-08 | merge | 冻结提交定向门与有效复用审核通过、真实合入本仓，SD新Spec准确沉淀且不发布未实现能力；全AG里程碑pending |
| AC-09 | merge | 本Plan/来源导航归档，唯一树/分支/空组fresh guard clean后安全清理，EXE/原证据恢复映射与实际回读完整 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、G-01 | 核013实际SHA/SDK/来源、规约与写面/owner；建唯一ui-016树，记录本仓及只读依赖身份 | AC-01 | [x] |
| T-02 | work | T-01 | 最多两接法的真实whole-path探针、实例/事件/预算先证；node2d-contract冻结机制/精确入口和oracle，失败needs_replan | AC-01..06 | [x] |
| T-03 | work | T-02 go | 显式pilot域解析/共同模型/普通模块绑定与诊断；新增dialect/compiler命名tests，V-01 | AC-02 | [x] |
| T-04 | work | T-03 | Auto实例状态/receiver/Life固定时钟与input→Player/H1/H2→AG/GPU，独立状态与事件分类V-02/04 | AC-03 | [x] |
| T-05 | work | T-04 | Coin触发→父model→条件差量/真实despawn，Pause/Step/旧代/owner20轮及负控V-03/04 | AC-04/05 | [x] |
| T-06 | work | T-05 | 新fresh消费A→B/120tickGPU/预算/旧触面，工具正反控、fmt/warning/提交、SD候选及完整移交V-05 | AC-01..06 | [x] |
| T-07 | review | T-06 | 同树独立final review逐AC/V/S/SD/遗漏，绑定提交；finding回原树work，pass标冻结门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结代码/依赖/输入，执行受影响门或严格有据复用后合本仓master | AC-08 | [ ] |
| T-09 | merge | T-08 | 仅沉淀实际pilot Spec/来源导航，归档本合同；不手写派生账本/索引 | AC-08/09 | [ ] |
| T-10 | merge | T-09 | 每树guard clean、HEAD祖先/clean/path核验后清树/分支/空组；保EXE/证据及恢复收据，回读完成 | AC-09 | [ ] |

当前可交接的是draft合同；只有用户后续启动且G-01成立，T-01才可执行。T-06完成即可交review，不要求work提前合入/清树。没有任意agent/模型或自动继续授权。

## 9. 复审记录

2026-10-30 / stage: new / plan_id: auto-ui/PLAN-016 / plan_revision: 1 / outcome: blocked（仅实施前置013实际delivery及后续启动尚未满足；非文档起草阻塞） / next: 用户审阅；后续启动并核G-01后work T-01/T-02。drafting0/10；本轮只读调查/独占取号与合同/导航，未建树/改代码/生成夹具/构建/测试/GPU/agent。T-02机制仍为有界假设，命令和样例是未来产物，不写成通过证据。

2026-10-30 文档预检：0..10节顺序、10个T/9个AC/5组V-S、任务/验收/SD承接和前置方向核对通过；15个当前本地链接可解析，016活动/归档唯一、下一号017，目标实施树不存在。另核匿名msg和点前缀已有支持，未将其冒称本期新增。仅静态文档/来源检查，不是实现验收或独立final review。

2026-10-30 / stage: new / plan_id: auto-ui/PLAN-016 / plan_revision: 2 / outcome: blocked（原实施依赖/后续启动门保持） / next: 仍暂不实施，未来满足G-01并获启动后work T-01/T-02。依据用户明确命名选择，§1/5/7与草图中的固定更新统一为Life.Tick；Life.Frame仅为后继呈现帧事件，不纳入016实现/夹具/验收。T/AC/V/S/SD数量、预算、依赖与支持范围保持，匿名msg/前导点等方向不变；r1记录及命名历史保留于本节/Git，不将文档修改写成执行证据。

2026-10-30 / stage: new / plan_id: auto-ui/PLAN-016 / plan_revision: 3 / outcome: blocked（原实施进入门保持） / next: 仍暂不实施。按用户确认，统一view-2d名称，草图v4；同名嵌入区域/多呈现变体只在总体设计说明，016只验证Node直属二维块。T/AC/V/S/SD数量、原预算/依赖保持，不纳入混合嵌套、view-3d/portrait/widget选择或Life.Frame。此前命名历史与r1/r2记录保留，尚无实现验证。


2026-10-31 work T-01/T-02 closure：`stage: work | plan_id: auto-ui/PLAN-016 | plan_revision: 3 | outcome: pass (T-02 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-016/auto-ui | branch: codex/plan-016-dev | base_commit: 69690dfb8e0b691a9af8d9e1eacc240511d2e83e | code_commit: 758f909 | dependency_revisions: SDK 7b987ca8（.sdk-source 681 files prepared）、auto-lang 3fa75dd9 只读兄弟树、auto-down c3464d0 只读兄弟树、语言仓零写面 | task_ids: T-01,T-02 | evidence: attachments/016/work/node2d-contract.md；原型 t02-prototype.log SHA 1e54b98f5cdf9a1dfbd9e172e30b51de4bc39246e329500efcaa20e628e72a19（2 passed/0 failed） | blockers: none | next_responsible_role: work T-03`。G-01 核实（013 归档 60e6d48、交付 8b1e799、canonical 23ed93e6/台账 d910b869 已入 auto-lang/master）。接法比较： (a) 生产 UI 服务结构性排除（merged-root 状态按名合并/无实例 receiver，vm_bridge.rs:894-916/1791/972/3517 代码证据），不改生产面； (b) 选中——原型 Start+一次Tick+两实例+一次真实事件+真 despawn+幂等+预算满池实测通过。主检出他者文档 WIP 保留未触。

2026-10-31 work T-03 closure：`stage: work | plan_id: auto-ui/PLAN-016 | plan_revision: 3 | outcome: pass (T-03 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-016/auto-ui | branch: codex/plan-016-dev | base_commit: 69690dfb | code_commit: 475140a | dependency_revisions: 同上 | task_ids: T-03 | evidence: auto-ui-dialect tests/node2d_pilot 6 passed；auto-ui-compiler tests/node2d_pilot 6 passed；两 crate 全量套件 0 failed（dialect 95+6+2、compiler 1123+9+6+1+3） | blockers: none | next_responsible_role: work T-04`。新增 dialect node2d 研究 dialect（UiDialect 生产路径零变化）与 14 文件夹具；诊断带文件/行号；Core 形态实测发现（Life.* 模式 Core 不支持→研究域自有 on 解析；`view-2d` 词法=View+Sub+2；`msg` 参数端口非词法关键字）均冻结入合同 §3a。

2026-10-31 work T-04 closure：`stage: work | plan_id: auto-ui/PLAN-016 | plan_revision: 3 | outcome: pass (T-04 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-016/auto-ui | branch: codex/plan-016-dev | base_commit: 69690dfb | code_commit: 50852bc | dependency_revisions: 同上 | task_ids: T-04 | evidence: 生成器（load/validate/lower/scene/translate）+ 冻结生成物 generated/{world.at 4858B sha f66b1b92…, scene.agscene, identity.json}；driver tests/node2d_flow CPU 电池 2/2 passed（合同 S6：探针 plan016_probe.rs 删除，superseded） | blockers: none | next_responsible_role: work T-04b/T-05`。依赖树实测约束：tip（生成器）与 pinned-SDK（driver）两树不得同图（同包名冲突），冻结为"生成器不依赖 driver、生成物为冻结数据、再生字节等价控制"。生成缺陷四项实测修正：Op::NotEq 为绑定臂误配（真变体 Neq/Ge/Le）、赋值括号、emit 目标 snake、int 字段初值形态。

2026-10-31 work T-04b/T-05 closure：`stage: work | plan_id: auto-ui/PLAN-016 | plan_revision: 3 | outcome: pass (checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-016/auto-ui | branch: codex/plan-016-dev | base_commit: 69690dfb | code_commit: d8fa903 | dependency_revisions: 同上 | task_ids: T-04b,T-05 | evidence: driver tests/node2d_flow 4/4 passed（GPU 像素类三层证据、Pause/Step 中性、20 轮 Stop/fresh、20 轮 CPU owner 释放、双父 receiver 隔离补充夹具） | blockers: none | next_responsible_role: work T-06`。GPU 校准：光照混合下相对通道类规则（红 237,124,124/橙 243,188,137/金 249,231,124/地面 179,179,188）；场景模板停靠防 born-overlap；scene/6（无 HUD/audio 负载）；嵌套实例（Collector→Coin）经绑定表投递实测隔离。

2026-10-31 work final handoff：`stage: work | plan_id: auto-ui/PLAN-016 | plan_revision: 3 | outcome: pass | worktree_path: D:/autostack/.wt/ui-016/auto-ui | branch: codex/plan-016-dev | base_commit: 69690dfb8e0b691a9af8d9e1eacc240511d2e83e | code_commit: 59ff619 | dependency_revisions: SDK 7b987ca8/681；auto-lang 3fa75dd9（只读）；auto-down c3464d0（只读）；driver/生产 crate 源零改动（本计划 driver 侧仅新增 tests/） | task_ids: T-01..T-06 complete | evidence: attachments/016/work/t06-readiness.md | blockers: 无工作侧 finding；统一回归门 pending（owner=review/merge）；D-04 独立终审上下文归 review | next_responsible_role: 同树独立 final review on frozen 59ff619`。AC-01..06 工作侧全闭合映射见 readiness；verify_plan016 --controls PASS exit0（身份/再生等价/变异敏感/缺模块诊断/篡改检出）；consumerB A→B：git archive 新根+auto-lang@3fa75dd9+冷 SDK → 再生 sha f66b1b92/4bd54fa3 与 A 逐字节等价，driver node2d_flow 在 B 全量通过。fmt 已过；warning 面仅 sdk 既有 146 项（非本计划引入）。

2026-10-31 goal round1（work 全阶段）：`stage: goal | plan_id: PLAN-016 | plan_revision: r3 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-016/auto-ui | commit: 803026d（T-06 内容提交 59ff619 为冻结代码提交） | next: review`。本轮 T-01..T-06 全闭合：G-01 核实、合同冻结、研究文法+模块绑定 12 例、降层生成器+冻结生成物、driver 6 例（CPU/GPU/预算/旧代/双父/生命周期/owner）、verify_plan016 --controls PASS、consumerB A→B 全量 6 passed（冷 SDK 681 files，t06-consumerB.log SHA 见 plan-artifacts）。执行 door：D-01/D-02/D-03 闭合，D-04 归 review。统一回归门 pending（owner=review/merge）。

## 10. 待澄清事项

| ID | 事项 | 关闭owner/下一步 |
|---|---|---|
| D-01 | 已闭合（2026-10-31）：013 已实际归档（60e6d48/8b1e799），T-01 核实 | 关闭于 T-01 |
| D-02 | 共同领域/模块与固定AG SDK如何在本仓接通，是否无需外仓语义变更 | T-02最多两接法先证；不能成立则带反例修订，不先全面实施 |
| D-03 | refs/类型化事件端口的精确语法、资源准备、tick/触发交接与原额可行性 | T-02在已定node/view-2d/Life/msg/前导点内冻结常规细节；改变AC/额度/领域范围另修订 |
| D-04 | 独立终审上下文及必需实际GPU环境 | T-06交接按后续用户安排或已授权资源核；没有证据保持pending，不伪称独立pass |

无须本轮重新决定已确认作者方向；以上是未来执行的依赖/机制关闭项，用户本轮“暂不实施”持续有效。
