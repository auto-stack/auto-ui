---
plan_id: PLAN-017
status: archived
feature_name: AutoGraphics 完整G11二维Node作者能力
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-016]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-g11-authoring.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, auto-ui-compiler, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-017：完整 G11/2d 所需的 Node 作者能力

## 0. 变更摘要

继承已交付 016，补齐能由声明源码表达原 G11/2d 的有限作者入口：普通 use、多实例 model/on、结构化场景与具名资源/actor 引用、左右输入、多类触发与业务事件、三件物品差量销毁、胜败和新鲜 Enter 重开。作者不再手写场景 JSON、整数槽或生成的 app_start/update。

仅 auto-ui 引擎、生成工具与验收夹具；[ag-examples/007](../../../ag-examples/docs/plans/007-autographics-gallery-g11-2d-node-migration.md)另管原 001 的二维内容迁移，硬等本计划实际交付。本轮 drafting 0/10，只规划；不建实施树、改实现、构建、运行 GPU、创建 agent 或自动实施。

## 1. 目标

1. 有限 Node2D 模块工程经既有领域解析、绑定和结构化 lowering 生成实际场景/Auto/身份信息，再由固定 SDK 真执行和 GPU 呈现。保持 node/model/view-2d/on、Life.Start/Tick、匿名 msg 与前导点语义。
2. 保持原 G11/2d 全部玩法与 51 项既有 oracle 的语义；正向三收集后赢、alt 危险失败、左行越界、未集齐终点不赢、同 tick 三件的原静止分支、对象/事件/Run 生命周期均成立。
3. 不再靠 Main/Player/Coin/Hazard、body/target/on_collect 等业务名字决定模板、actor 或 handler；类型化声明、引用和事件绑定决定结果。有限同类型多实例/不同资源可重用。
4. 下游能固定交付版本，离线准备 main/alt 模块工程并公开启动；正常启动不依赖测试 harness。生成物与全部输入闭包/工具/运行器身份可核对。

非目标：3D Node/G11-3d 迁移、跳跃/声音/HUD、完整 G2 平台游戏、新物理/事件队列、无限节点、完整集合库、任意 key 子树重建、多呈现/混合 UI、Life.Frame、生产 widget 退役、ACE/CPC/AOT/ECS/HMR、新平台、迁仓或修改 auto-lang/旧 SDK。旧 016 夹具与其明确受限的输入映射保持历史兼容，不拿新能力倒改旧事实。

## 2. 架构方案

模块源码 → Node2DDialect/Core AST 与普通 use 定位 → 显式 g11-2d 研究模式的声明/引用/事件表示 → 结构化场景和实例布局/真实 Auto handler → 既有 triggers2d/1、对象/角色/相机/round 服务 → Host → GPU。新模式拟以 node2d-lower --profile g11-2d 显式选择；T-02 冻结精确 CLI，旧 pilot 模式不被自动升级。

采用既有 Auto 编译/执行服务；Rust 只解析、校验、生成、调度和连接图形服务，计数/胜负/巡逻公式/重开条件由用户 Auto 表达。lowering 后可以是一份运行源，不能去掉 use 拼接文本冒充模块解析；不引入第二 parser/解释器或直读 VM 私有全局绕过服务。

拟唯一实施/顺序复审树 D:/autostack/.wt/ui-017/auto-ui，分支 codex/plan-017-dev。主检出仅合同簿记。语言/auto-down 如需参与解析或构建，只读固定版本同组树，禁止链接、禁止外仓写面；清理先 fresh wt-guard。014/007 生产 UI 计划及语言仓900依赖反转不构成隐式硬依赖，共享文件按真实 diff/owner 短窗协调；900若改变实际解析装配点/manifest，T-01重绑端口与相关消费者，不接管其修改。

## 3. 技术栈

已存在：crates/auto-ui-dialect/src/node2d/{mod,ast,parse}.rs、tests/node2d_pilot.rs；compiler tests/node2d_pilot.rs 与 src/ui_ast.rs::resolve_use_module；experimental/autographics-node2d-pilot/src/{load,lower,scene,translate,main}.rs 与 tools/verify_plan016.py；driver tests/node2d_flow.rs、runtime_triggers_api.rs、runtime_round_host.rs，api/runtime_triggers_v1.at，src/authoring/{runtime_round,behavior,play}.rs 和 bin/autographics-authoring.rs。

拟新增：pilot fixtures/g11-2d/{main,main-alt,resources,types,player,coin,hazard,goal,void_zone,patrol}.at、tests/g11_2d_authoring.rs、tools/verify_plan017.py、driver tests/node2d_g11_flow.rs；T-02 冻结实际模块布局和命令。不复制 Gallery 为引擎业务实现；夹具仅为公共作者入口的有限消费测试。旧 runtime API/SDK/锁原则上零改动；若需要改共享服务，在原兼容范围内列实际触面和命名受影响用例，改语义/预算则回规划。

## 4. 需求分析与背景调查

用户 2026-10-31 明确授权两仓后继规划、保持原玩法和验收、暂不实施。仅允许计划/设计/来源导航与取号；没有建树、代码、资源生成、构建、窗口、agent、发布或自动执行授权。已知取号独占扫描本仓活动/archived/archive 最大016、.next-id017，分配017，下一号018；不使用外仓序号。

当前 auto-ui HEAD d29e36c55c8bb74e672a44ce6aa654c10318da7f；016 delivery 932bd5fb0aca26586e3760bd070eb416ffe67418 已是祖先，独立 R2 pass 绑定实现9eb1d0e，实际归档/清理收据可读，不能被旧 current_step:6 或历史暂不实施段落覆盖。执行时重新冻结 HEAD、共享 owner、依赖与实际 sdk 锁，不追开发 tip。890 已交付进入主线，不将旧的“等890”泛化为本计划前置。规划末次worktree核对新出现lang-900/auto-ui（plan-900-ui-dev，HEAD同d29e36c、当次clean）；其合同涉及UI管线装配/依赖迁移，已纳入T-01实际owner窗口检查，不宣称它已完成或代其建树。

权威来源：[本仓 pilot Spec](../specs/auto-graphics/node2d-pilot.md)、[历史 G11 Spec](../../../auto-lang/docs/specs/auto-graphics/gallery-runtime-objects.md)、[行为来源与项目](../../../auto-lang/docs/specs/auto-graphics/behavior-project.md)、runtime-objects/behavior-api；[当前设计](../design/autographics/g11-2d-node-authoring-completion.md)、[原目标草图](../design/autographics/g11-2d-target-source-sketch.md)及 ag-examples 原 001 冻结合同。旧标题中的 audited 只是历史名称，不新增“可审计 agent”产品要求。

只读观察 SHA256（raw 工作文件）：pilot Spec 085d64c136f8950d27ba906e1b583d8ea4f91846676a478ca238b829eb91d1bd；load.rs 4837d29c0cfe863dc9db307b1ff160d05b84bc0213fe4c3278166d267405d3d9；lower.rs 67ef61cb27a0b003e0890ebf95ca7bbf764d620ff486268775984cfb73773947；scene.rs 643f56bf7f876493d5598b11cbe3deae08ba5401909cf2fa735cc1e86794b5c0；translate.rs 2d90312968a08dc7253ba47f8941979ffb5330efa8d52b87cb3240bc873da59f。生成器以当前语言 crate 编译，driver 以旧 pinned SDK 执行；016 已证明两图同包名冲突，继续分图消费冻结数据。

实际缺口：scene::template_id/build_scene_json 固定 Main/Player/Coin/Hazard、坐标/角色；translate 的 axis 仅 Right；lower::emit_orchestration 把 target 参数固定到角色、取首个业务 handler、只读一事件、固定 AABB，不能表达完整 G11。Gallery 原回执为2D51+3D25=76 oracle，旧引擎交付0ee5ad0；本仓011两候选33/32为完整平台玩法 no-go，不能被这次 G11 小游戏可行性冒充推翻。

## 5. 详细设计

### 5.1 进入门和有界整链冻结

T-01 重新核016实际delivery/复审/清理、当前主线、固定 SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 的681文件保留来源、各 workspace lock、实际 UI/语言只读依赖和并行 owner。旧 SDK 历史对象不可达时只用经681内容锁核对的保留副本，不能换当前 SDK。唯一树只有后续用户启动时创建。

T-02 最多比较两种既有机制的等价接法；先完成完整 main/alt 模块夹具的前端→真实 Auto→Host/GPU 纵向探针，冻结 attachments/017/work/g11-2d-authoring-contract.md。需在后续全面实现前证明：不同形状三 Coin、左右输入/同时按、危险/goal/void 事件、一次条件销毁、终局冻结和 accepted round request；核真实初始化/普通/收集/静止三收集/胜败/重开成本。不能再用016单币探针代全链。

冻结各段 producer/consumer 文件符号、类型/事件端口拼写、调度与实例身份、CLI、输出布局/来源映射、测试精确 ID/容差。若必须改外仓语义/固定 SDK、加新解释器、提额度或丢原分母，带实际输入/输出和成本反例 needs_replan；不是本计划验收成功。

### 5.2 声明、引用和事件

- 支持根关卡与 Player、三个同 Coin 定义的实例（不同 mesh/material/位置/key）、Hazard、Goal、VoidZone，静态 ground/camera；同类型实例本地 model、参数、handler receiver 与订阅隔离。可有无呈现的组合父 Node，不以模块名/类型名指定 root。
- 从有限 solid2d/trigger2d/camera2d 声明和 typed 引用生成模板/角色/实体/trigger；角色、地面与相机目标按显式语义端口标识。具体属性名在 T-02 冻结，不能用名为 Player/body 等隐式约定。资源 symbol 可作为参数，三种外观由资源引用决定；生成 ID 不进入作者业务规则。
- msg 与绑定按事件名、payload 类型和父实例匹配；on_enter/on_stay/on_exit 必须路由到对应 phase，不能取第一个 handler。G11 正常只靠 enter 计分/失败/胜利；补充 stay/exit 用于证明分类，重名/错误类型/错 actor/未知属性在执行前带模块/行/符号拒绝。
- 引用失败、模块循环或无法构造的 Node 实例循环、重复绑定、资源类型不符为受控诊断，不能 panic/无限递归。普通 use 只读取已冻结工程闭包，不靠特殊 resources 文件名或全局函数名池假定一切同名合法；同名导入按现有规则解析，无法支持的形式显式拒绝。
- 具名服务映射左右 axis 的真实差值，Left=-1、Right=+1、两者/都无=0；Enter pressed 与 held 区分。系统 Life.Start 与用户 .Start 独立，Life.Frame 明确未支持。有限数值保持固定服务的 finite f32 与整数身份，不从 Value 的 f64 容器推断计算精度。

### 5.3 保持原玩法、时序与预算

冻结玩法输入继承原001：编辑角色(2,1)，保留原app_start设置角色y=0.5的实际起步语义与地面；三物品(8,1)/(10,1)/(12,1)，圆/方/圆且不同颜色大小；危险x=9，主例基线2.4，alt0.9并调整速度/巡逻；goal(16,1)、void(-1,1)、camera bounds(-4,0)..(30,12)，view12×8。准确几何与原51检查名字见[只读基线](../../../ag-examples/docs/plans/attachments/007/planning/g11-2d-baseline.md)；alt原巡逻初值0.08、越界反向仍为±0.05，不能简化成全程0.08。不把数字槽ID当产品语义。

Playing/Won/Failed、0..3计数、重复/重入幂等由真实 Auto；未集齐 goal 不赢，胜败停止角色运动，危险或越界失败。三件物品不同身份首次进入加一并真销毁，hidden仍可触发、despawn撤销资格。同 tick 三件保持“引擎发布三个不同进入 + 静止回调实际读三条”两个独立观察；不宣称移动时处理全部触发。

时序继续 N 的几何/evaluate 产生、N+1 回调读之前快照，pair 顺序稳定，每tick一次evaluate、旧未读不排队。移动读最多1条、静止最多3条（不得多读终止 next 造成隐藏费用）。原例有事件 tick 不写危险图形位置、Auto巡逻状态仍推进；新作者服务/编排须显式记录等价策略，不通过暂停整个 Hazard Life.Tick 或任意漏事件冒充预算通过。

一个世界共享32 credits/callback，不给每Node单独池；各 getter/读相位/销毁按旧成本。移动收集候选左右2+sweep26+next1+pair1+phase1+despawn1=32，无额外危险写；静止三条与新局初始化另测。不可私读 _ag_* 全局代收费API、改价格、降低输入/事件验收频次或把未知CPU工作排除。8运行槽/8对、资源8身份/32MiB raw/CPU与GPU各64MiB，用户模块源码闭包<=256KiB、可信桥<=96KiB、组合<=352KiB；生成用户源也须满足256KiB，无每模块额外配额。新输出格式必要清单按实际内容计bytes，不混 raw 资源和源码单位。

既有VM1000steps/2ms分片、累计100000steps/120resumes/250ms、init30s/tick2s保持；多handler计完整tick。8ms pump目标独立报告met/missed/unmeasured，不以2s替代。六种主要分支实际成本与32/33、VM守护反例须在完整路径记录；不为成本报告做无关全局性能矩阵。

重开保留原 fresh Enter→Auto条件→ag_round_request→candidate接受→新Run：角色/相机/三物品/计数/订阅与Life.Start重置，held不连开、旧Run结果/事件不影响新局。Pause不处理，Step中性一tick，Resume不再Start，Stop恢复编辑文档。不是引入任意key子树重建；草图集合/HUD/持续每tick危险写等超出原兼容的方向留后继。

### 5.4 可搬迁的准备与启动契约

新模式输出 main/alt 各自的真实 world.at、scene.agscene、输入模块闭包/生成器身份/输出bytes与SHA/API/角色映射清单；字段在T-02冻结，少件/错身份/逃逸/链接/坏引用拒绝。已有合法产物的失败准备保留完整旧集或具名拒绝并报告自有临时目录，不能半套新旧混用；内容改变旧Ready立即失效。

公开作者准备命令拟 cargo run --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --bin node2d-lower -- --profile g11-2d --entry <main.at> --out <new-dir>，产物由 autographics-authoring --open <scene> --behavior <world.at> --behavior-api triggers2d 进入真实Run。T-02可冻结等价命令，必须文档可重放，不能以cargo test作为正常运行唯一入口。允许离线分发预编译生成器和runner，正常用户运行只需runner与验证后产物。

提供固定delivery消费说明、必要生成器源及其精确只读语言依赖/锁、可重建runner与内容清单；source-only A→B冷准备再运行，B不借生产者源/target/工作目录或网络。下游只读这些产物，不复制Rust lowering。构建元数据/SDK681需完整核验，不伪称SDK历史Git对象可达；生成器tip依赖与SDK两图分别绑定。

### 规范增量

本仓新Spec权威；历史语言仓Specs只读，不写派生specs.json/index.json。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-g11-authoring.md | 016固定最小切片→显式g11-2d模式的实际声明/模块/资源/输入/事件/调度/预算/重开和消费接口；列未支持面 | 下游有可锁定的真实合同，保旧pilot事实 | AC-01..06 |

## 6. 测试设计

以下为未执行的拟建入口：T-03/04新增pilot tests/g11_2d_authoring.rs；T-05新增driver tests/node2d_g11_flow.rs；T-06建立verify_plan017.py --controls及fresh consumer。既有命令 cargo test -p auto-ui-dialect --test node2d_pilot、cargo test -p auto-ui-compiler --test node2d_pilot、pilot --test node2d_lower_gates、driver --test node2d_flow 与 verify_plan016.py --controls按真实影响选取。T-02冻结准确选择、非零case数量和超时；旧G11适配中使用文本替换app_start的控制，须改成源码级受检变体或可定位的等价测试输入，不把旧变量拼写变成新编译器承诺。

| V/S | AC / owner_stage / T | 输入→实际入口/范围 | 区分 oracle 与关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 模块/资源/节点/字段改名与声明重排、不同资源三同类实例→真正load/validate/lower→Auto/GPU；缺/错引用/循环/重复/类型反例 | 改业务名字不改行为，改被用资源/位置实际像素/轨迹改变；拒绝含file:line/symbol且无panic；不声称全部语言形式 |
| V-02/S-02 | AC-03 work T-04 | Left/Right/同时/无，Life.Start与自定义.Start，双父/同类参数/状态，多个业务handler、enter/stay/exit | 轴-1/+1/0/0，120tick独立f32预期与GPU移动；只正确receiver/phase变化；给首handler/首父/Right-only负控必须红 |
| V-03/S-03 | AC-04 work T-04/05 | 完整main/alt、三种外观、六pair、未齐goal、void、主/alt参数、静止同tick三件 | 逐项承接原2D51检查的含义，计数/角色/实例/触发/真实GPU交叉观测；缺物品/假hidden/错误胜败/固定轨迹/漏事件反例必须红；不覆盖3D |
| V-04/S-04 | AC-04/05 work T-05 | hidden/despawn/rebind/generation/旧Run、Enter沿/held、Pause/Step/Resume/Stop、20短轮owner | 隐藏仍触发、销毁不再stay/exit、新代不自动绑；fresh0/三新实例/初始角色相机，旧事件不串局；owner最后释放观察而非驱动缓存归零 |
| V-05/S-05 | AC-01/05 work T-02/05 | init/普通/移动收集/静止三收集/goal/失败/重开→完整执行账、32/33与VM守护 | 按旧真实调用成本接受合法32、33具名拒绝且不宣称部分成功；全tickCPU可见，8ms单列，预算不足返回规划，不删分母 |
| V-06/S-06 | AC-06 work T-06 | 独立固定消费A→B，main/alt离线重建/公开Run；缺模块/错工具或SDK/产物篡改/失败准备→精确修复 | B真Auto/GPU与A语义及可再生身份匹配，修复后同入口再次可用，原合法完整集不被半写破坏；旧016及真实受影响consumer保持 |

原51检查逐项映射落入T-02合同，所引用的 checks/g11 和回执从 ag-examples 7c68128f7dd9f6b3c5d8e3b1ad8e10684c09d0ba只读导出，不依赖未来007源码；引擎夹具/测试属于本仓，不把内容仓脚本作为生产依赖。

work自查→同一权威树独立上下文final review→merge冻结门；用户未指定人员/模型，本轮不创建或联系agent。review pass须绑定r1与提交，并列merge-owned门pending，不能用实现会话自查代独立终审。改共享UI解析/生成器时必须选实际消费者回归，不能借AG定向规则跳过UI义务；不跑无关全AG/Gallery/root全套，里程碑档pending。GPU/窗口重型门机器单实例，OS抢焦点自动化暂停。原始证据仅D:/autostack/plan-artifacts/auto-ui/plan-017，attachments只md摘要/合同，依最近5个已完成计划政策处理。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 016交付/依赖/唯一树核实，有界完整main/alt链与原预算先证，未改外仓语义；V-05 |
| AC-02 | work | 场景、类型化资源/actor与模板按结构声明生成，改名重排敏感、错误定位拒绝；V-01 |
| AC-03 | work | 普通use/实例局部状态/左右输入/具名handler/typed父事件真实生效且隔离；V-02 |
| AC-04 | work | 原G11/2d51项语义在新模式完整重建，main/alt、三收集、胜败、几何/生命周期与GPU均成立；V-03/04 |
| AC-05 | work | accepted新Run/旧代/20轮释放、原额及负控、旧pilot兼容与完整tick成本；V-04/05 |
| AC-06 | work | 固定可搬迁生成器/runner/输入输出身份，main/alt公开运行、离线A→B及损坏恢复，实际受影响旧路径保持；V-06 |
| AC-07 | review | 独立逐AC/V/S/SD与51映射、源码/资源来源、遗漏/延后/workaround/格式/warnings复审，零未处置finding，绑定最终提交 |
| AC-08 | merge | 冻结受影响门通过或严格有据复用，真实进入master，SD-01仅沉淀实际能力，通知下游可锁定身份而不自动实施007 |
| AC-09 | merge | 归档/导航与实际完成数准确，树分支空组fresh guard后清理，产物/恢复与待里程碑门收据完整 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续用户启动，016实际交付 | 读当前Spec/交付/owner与两图依赖锁，创建唯一ui-017树，固定输入与原51清单 | AC-01 | [x] |
| T-02 | work | T-01 | 最多两接法完整main/alt整链及全部峰值先证；冻结contract/CLI/typed端口/调度/51映射/准确门；不成立needs_replan | AC-01..06 | [x] |
| T-03 | work | T-02 go | 结构化场景/资源/引用/模块与诊断，删除新模式的业务名特判，改名/重排/变资源正反例V-01 | AC-02 | [x] |
| T-04 | work | T-03 | 完整输入/Life/多handler与父receiver/phase分类，差量实例与原事件调度；V-02/03 | AC-03/04 | [x] |
| T-05 | work | T-04 | 完整51语义与GPU、main/alt、重开/旧代/owner20轮/原额/VM负控与旧pilot验证；V-03..05 | AC-04/05 | [x] |
| T-06 | work | T-05 | main/alt公开准备/Run、fresh固定消费A→B/损坏修复/旧触面、verify controls/fmt/warnings，SD候选与work移交 | AC-01..06 | [x] |
| T-07 | review | T-06 | 同树独立final review逐AC/V/S/SD/51映射/遗漏，finding回原树work，pass列冻结门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结当前提交/依赖/输入，执行定向并集或有效复用，实际合本仓master，保他者WIP | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01与来源/准备运行说明沉淀，归档本合同/导航；不手写派生索引 | AC-08/09 | [ ] |
| T-10 | merge | T-09 | 逐树fresh guard、clean/祖先/路径核验后清树分支空组，复核可重建消费身份和证据恢复 | AC-09 | [ ] |

## 9. 复审记录

2026-10-31 / stage: new / plan_id: auto-ui/PLAN-017 / plan_revision: 1 / outcome: pass（规划合同可审阅，实施启动尚未授权；T-02为实施内有界可行性门） / next: 后续用户启动后work T-01/T-02。独占本地取号017、下一号018；引用016实际交付/原51检查和真实源码观察，全部新用例/命令未执行。未建树、生成资源、编译/GPU、创建agent或修改Gallery实现。

2026-10-31文档预检：0..10节顺序、10个T/9个AC/6组V-S/1个SD、owner与前置/后继方向、全部本地Markdown链接、唯一号源和drafting0/10检查通过；原51检查名与内容仓历史回执逐字对齐（3D25另保留）。git diff --check通过；只改规划/设计/导航与号源，生产源码/SDK/生成物零改动。这不是实现测试、可行性go或独立final review证据。

2026-10-31 / stage: work | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass (T-01 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | base_commit: 7a53d13d5e6da6eedaa9b1e9de0a3fdcaca089c4 | code_commit: b02ba75 | dependency_revisions: SDK 7b987ca8（driver .sdk-source 681 files prepared PASS）；auto-lang 98270a4f5（只读兄弟树，delivered master，plan-890 closed）；auto-web f29c7d1（只读兄弟树，auto-lang-core path dep）；auto-down c3464d0（只读兄弟树） | task_ids: T-01 | evidence: 016 delivery 932bd5f 核实为 HEAD 祖先；两图实测绿（pilot 生成器 9/9、driver node2d_flow 6/6 于新绑定）；原001三源 SHA256 逐字节核对通过（main.at a1b4c80e、main-alt.at 947b1e0d、scene.agscene 4968e398，对 g11-2d-baseline.md 表）；原51清单取自 ag-examples checks/g11/gallery_runtime.rs（fae22bd4 逐字核对，1372行）；pilot Cargo.lock 随图重绑（提交 b02ba75）；主检出他者文档 WIP 未触；lang-900/os-051 等他者树未触 | blockers: none | next_responsible_role: work T-02

2026-10-31 work T-02 closure：`stage: work | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass (T-02 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | base_commit: 7a53d13 | code_commit: 3f5fe1d | dependency_revisions: 同 T-01 | task_ids: T-02 | evidence: 合同冻结 docs/plans/attachments/017/work/g11-2d-authoring-contract.md（接法比较 (a) 处理器内嵌图形=33>32 预算反例排除、(b) 声明位置+编排同步选中）；完整 main/alt 纵向探针三例全绿（driver node2d_g11_flow：walk→3有序收集→胜→冻结→新鲜Enter重开→重生；alt 危险失败/重开/旧局隔离/held 不连开；CPU 恰 32 校准=第 33 计费调用 fault 10）；dialect character2d 研究元素（legacy 字节不变：verify_plan016 PASS f66b1b92/4bd54fa3） | blockers: none | next_responsible_role: work T-03..05`

2026-10-31 work T-03..T-05 closure：`stage: work | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass (checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | base_commit: 7a53d13 | code_commit: ae8ecef | task_ids: T-03,T-04,T-05 | evidence: 声明驱动降层无业务名特判（类型化 msg/ActorRef2D/Mesh2D/Material2D 端口、逐实例模板、phase 路由、逐实例事件路由泛化）；7 个源级变体世界（triple/stay/bag2/geometry/generation/motion + alt）；driver 电池 15/15 全绿（void 左轴越界、同 tick 三件 3 enter/3 读/3 销毁、stay 分类 1 enter+多 stay 不记分、bag2 终点门+goal exit phase、半径几何三态、同 slot 代次重绑 gen2、both-moved 终位/away 永不进、Pause/Step/Stop、20 轮 owner 释放+旧代 fault 33、hidden engine fact、GPU 像素类+无窗 PNG）；两处实测缺陷修复带回归（接收方形参数=绑定载荷数校验——bag2 走过 goal 的缺参调用曾致 VM 垃圾索引崩溃；位置同步移至任意无事件 tick——moved 分支误排曾致移动期间图形冻结，debug 轨迹证实引擎与原 001 等价策略正确） | blockers: none | next_responsible_role: work T-06`

2026-10-31 work final handoff：`stage: work | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | base_commit: 7a53d13d5e6da6eedaa9b1e9de0a3fdcaca089c4 | code_commit: 83a1480（T-06 内容提交 f865853/ba38ebc，SD 候选 83a1480） | dependency_revisions: SDK 7b987ca8；auto-lang 98270a4f5 只读；auto-web f29c7d1 只读；auto-down c3464d0 只读；语言仓零写面 | task_ids: T-01..T-06 complete | evidence: docs/plans/attachments/017/work/g11-2d-authoring-contract.md + 本节收据 | blockers: 无工作侧 finding；统一回归门 pending（owner=review/merge）；窗口化 autographics-authoring 实际启动观测 pending（GPU/窗口重型门机器单实例，owner=review，D-04 上下文） | next_responsible_role: 同树独立 final review on frozen 83a1480`

AC 工作侧闭合映射（全部绑定 83a1480 上的命名测试断言）：AC-01（016 基线/唯一树/预算先证/无外仓语义改动）= T-01 收据 + 合同 §1/§4 + cpu_budget 恰 32；AC-02（声明生成场景/资源/模板、改名重排敏感、错误定位拒绝）= g11_gates 10/10 + verify_plan017 变异控制；AC-03（普通 use/实例局部状态/左右轴/具名 handler/typed 父事件隔离）= g11_flow walk/alt/void（差值轴两侧实测）+ gates；AC-04（原 51 语义完整重建含 GPU）= g11_flow 15/15 对照合同 §6 映射表（51/51 全覆盖：49 项 node 源级夹具/主链 + hidden/stale 两项 engine-fact 探针，观测位适配均记录于合同）；AC-05（accepted 新 Run/旧代/20 轮/原额/负控/旧 pilot 兼容/完整 tick 成本）= walk/alt 重开段 + owner_20_rounds + stale_write fault 33 + cpu_budget 33rd-fault + node2d_flow 6/6 + verify_plan016 PASS；AC-06（固定可搬迁生成器/runner/身份、离线 A→B、损坏恢复、旧触面）= verify_plan017 六控制 PASS + consumerB 全量（新根导出+冷 SDK 681 files+冷依赖+冷 target：8 世界再生逐字节等价、g11 电池 15/15、legacy flow 6/6、verify 双 PASS、gates 10/10）。

统一回归门与窗口化公开启动观测 pending（owner=review/merge）；本计划 driver/生产 crate src 零改动（仅新增 driver tests/ 与 pilot/dialect 研究面），legacy profile 字节无触（verify_plan016 直通）。

2026-10-31 goal round1（work 全阶段）：`stage: goal | plan_id: PLAN-017 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-017/auto-ui | commit: 83a1480 | next: review`。本轮 T-01..T-06 全闭合：G-01 核实（016 交付 932bd5f 为 HEAD 祖先）、合同冻结、character2d/类型化端口/phase 路由/差值轴/pressed/round.request 降层、源级变体世界、driver 电池 15/15（含 GPU）、gates 10/10、verify_plan017 六控制 PASS、consumerB A→B 全量复现（冷 SDK/冷依赖/冷 target）、fmt/warnings 清零、SD-01 候选 docs/specs/auto-graphics/node2d-g11-authoring.md。执行 door：D-01/D-02/D-03 闭合，D-04 归 review。统一回归门与窗口化公开启动观测 pending（owner=review/merge）。work 移交绑定 frozen 83a1480。

2026-10-31 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent read-only subagent agent_79dca620（同树只读，未参与实现；自跑全电池） | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | reviewed_commit: 83a1480 | base_commit: 7a53d13 | dependency_revisions: SDK 7b987ca8/681、auto-lang 98270a4f5、auto-web f29c7d1、auto-down c3464d0 | spec_inputs: SD-01 候选 node2d-g11-authoring.md | acceptance_results: AC-01..05 pass；AC-06 partial（F-01 consumerB 证据缺失）；AC-07 本报告 | pending_closeout_gates: 统一回归门、窗口化公开启动观测（merge/review 所有） | findings: F-017-R1-01（major：plan-artifacts/auto-ui 组目录被布局迁移清除，consumerB 证据指针落空）、F-017-R1-02（minor：合同 §4 "moved 每 tick 清零" 未发射——行为为 sticky，预算中性但偏离冻结合同文本）、F-017-R1-03（minor：lower_g11.rs unused_mut 1 警告，"清零"声明差一）、F-017-R1-04（minor：oracle 35 缩窄为仅 teardown 断言，缺文档一致性） | evidence: 复审员自跑：dialect 6/6、compiler 6/6、pilot legacy 9/9+gates 10/10、verify_plan016 PASS（f66b1b92 冻结值）、verify_plan017 六控制 PASS、driver g11 15/15、legacy flow 6/6、fmt clean；51 映射逐项审计无缺失（16/25-30/33-38/41-44/50-51 与原断言逐字比对） | next: work 受影响修复后同员复审`

2026-10-31 work R1 repair：`stage: work | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass (R1 修复) | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | base_commit: 7a53d13 | code_commit: bf02061 | task_ids: F-017-R1-01..04 | evidence: F-02 修复=app_update 开头发射 _n2d_moved=0（合同 §4 文本兑现，预算中性，8 世界再生新身份）；F-03 修复=unused mut 移除（pilot src 0 warning）；F-04 修复=stop 前后 editor.document JSON 相等断言入 g11_pause_step_stop_harness；F-01 修复=consumerB 重新物化于 D:/autostack/plan-artifacts/plan-017/consumerB（新 layout：自身根+冷 .sdk-source 681 files+auto-lang 98270a4f5/auto-web f29c7d1/auto-down c3464d0 只读导出+冷 target）并全量复跑：8 世界再生逐字节等价（main 06d599d6…）、g11 15/15、legacy flow 6/6、gates 10/10、verify_plan017 PASS、verify_plan016 PASS | blockers: none | next_responsible_role: 同员 affected re-review on bf02061`

修复后 A 树全电池（bf02061 上复跑）：dialect 6/6、compiler 6/6、pilot legacy 9/9+gates 10/10、verify_plan016 PASS、verify_plan017 PASS、driver g11 15/15（58.79s 含 GPU）、legacy flow 6/6、fmt clean。证据目录迁移说明：plan-artifacts 旧 auto-ui/<plan> 分组已由外部布局迁移清除（非本会话行为），本计划证据根改为 D:/autostack/plan-artifacts/plan-017/（review 技能 2026-10-31 布局）。

2026-10-31 independent final R2（affected re-review，同一独立只读复审员 agent_79dca620）：`stage: review | review_scope: final affected re-review | reviewer_identity: agent_79dca620（同 R1 员） | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-017/auto-ui | branch: codex/plan-017-dev | reviewed_commit: bf02061 | base_commit: 7a53d13 | dependency_revisions: SDK 7b987ca8、auto-lang 98270a4f5、auto-web f29c7d1、auto-down c3464d0 | spec_inputs: SD-01 候选 node2d-g11-authoring.md | acceptance_results: AC-01..06 全 pass（AC-06 升级 pass：consumerB 物化+冷构建产物核验）；AC-07 本报告 | pending_closeout_gates: 统一回归门、窗口化公开启动观测、canonical/archive/cleanup（merge 所有） | findings: 无未处置；F-017-R1-01..04 全闭合且经复审员独立验证（修复 diff 逐行审计；_n2d_moved=0 发射核验、恰32/fault10 在清零语义下复证、文档相等断言恢复 oracle 35、consumerB 681 files+冷 target+8 世界 A↔B 逐字节等价核验） | evidence: 复审员自跑受影响测试：driver g11 15/15（50.2s）、verify_plan017 PASS、gates 10/10、pause/step/stop 单测 ok；修复未引入新 finding（driver tests 两处 pre-existing 死代码警告非 R1 范围，记录备 merge 卫生清理） | next: merge`

2026-10-31 goal round2（R1 needs_fix → 修复 → R2 pass）：`stage: goal | plan_id: PLAN-017 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-017/auto-ui | commit: bf02061（R2 pass 绑定） | next: merge`。R1 四 finding 全闭合（同员复审）；修复后受影响测试复证全绿。pending merge-owned 门：统一回归门、窗口化公开启动观测（D-04）、canonical/archive/cleanup。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 新模式精确typed refs/角色/事件端口与CLI/输出布局 | 已闭合（2026-10-31 T-02）：合同 §2/§3 冻结全部端口/CLI/输出/角色映射 |
| D-02 | 完整32原额路径（含phase与左右）及调度可行性 | 已闭合（2026-10-31 T-02 实测）：移动收集恰 32（第33计费调用 fault 10），六分支成本实测入合同 §4 |
| D-03 | 执行时当前语言只读依赖、014/语言900共享owner窗口 | 已闭合（2026-10-31 T-01）：auto-lang 98270a4f5（delivered master）+auto-web f29c7d1+auto-down c3464d0 只读兄弟树，写面 0；900 WIP 未触 |
| D-04 | 独立终审上下文/实际GPU运行窗口 | 窗口化公开启动观测已于 merge 记录（§12）；深度独立 GPU 终审窗口 pending 记录（离屏 GPU 电池已随 R1/R2 覆盖） |

无需用户重新决策的作者方向；执行门 D-01/02/03 已闭合，D-04 已记录。017 交付不自动启动 007。

## 11. 执行合同（T-02 冻结版全文，自 docs/plans/attachments/017/work/g11-2d-authoring-contract.md 折叠；附件目录按 2026-10-31 裁定删除）

# PLAN-017 g11-2d 作者合同（T-02 冻结）

status: frozen（T-02 go；完整 main/alt 整链实测通过）
plan: PLAN-017 r1 · auto-ui · tree `D:/autostack/.wt/ui-017/auto-ui` · branch `codex/plan-017-dev`
work base_commit: 7a53d13d5e6da6eedaa9b1e9de0a3fdcaca089c4（master tip，含 PLAN-016 交付 932bd5f）
code_commit: T-02 提交（见 §9 收据）
SDK pin: `7b987ca8`（driver `.sdk-source` 681 files，tools/prepare_sdk.py --from-snapshot PASS）
只读依赖: auto-lang `98270a4f5`（delivered master，plan-890 closed；auto-lang-core 现以 path 引入 auto-web-contract → ui-017 组含 auto-web `f29c7d1` 兄弟树）；auto-down `c3464d0`（只读兄弟树）。两图规则不变：生成器（tip 语言 crate）与 driver（pinned SDK）绝不同图。
实测证据: driver `tests/node2d_g11_flow` 三探针（walk/win/restart、alt fail/freeze/restart、CPU 恰 32 校准）全绿；legacy 电池全绿（dialect 6、compiler 6、pilot 9、verify_plan016 --controls PASS、node2d_flow 6/6）。

## 1. 接法比较（T-02 上限两种，实测裁定）

**(a) 016 式"处理器内嵌图形调用"（Hazard Life.Tick 内 refs.body.move_to 每 tick 写位置）——排除。**
预算反例：移动收集 tick = held2 + sweep26 + next1 + pair1 + phase1 + despawn1 + 危险 move1 = **33 > 32**，第 33 个计费调用为 sticky fault 10（实测技术，flow_budget 同款）。且与原 001 "有事件 tick 不写危险图形位置"的兼容要求冲突（计划 §5.3）。

**(b) 声明位置 + 编排图形同步（事件 tick 跳过）——选中，已实测。**
动态位置（声明引用 model 可变字段）的触发实例由编排段在**首条 event_next==0 的 tick** 执行一次 `ag_object_move`（位置=声明表达式按实例当前状态求值）；有事件 tick 跳过写、巡逻状态照常推进 —— 原 001 等价策略，逐字记录于本合同 §4。静态位置（仅字面量/参数）只在 spawn 时落位。
同步时点在实例 Life.Tick 之后（post-advance）：与原 001 相比，危险评估位置每 tick 领先一个巡逻增量（0.05），对原 51 项语义的可达带核查全部无影响（main 危险最低触达 1.75 > 角色顶 1.0；alt 死亡 x 带 [8.5,8.6] ⊂ (7,9.6)）；该 0.05 领先为本合同的**声明调度差量**，被下列 oracle 观测位置覆盖：G11-2D-13/22/23/42/43（映射见 §6）。

## 2. CLI 与输出（冻结）

- legacy（默认，016 字节）：`cargo run --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --bin node2d-lower -- --entry <dir>/main.at --out <dir>`
- g11-2d（017）：`cargo run --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --bin node2d-lower -- --profile g11-2d --entry <dir>/main.at --out <dir>`
- g11-2d 输出：`world.at`（生成用户源）、`scene.agscene`（autographics.scene/6，behavior 绑定 world sha/bytes）、`identity.json`（generator/api/sha/bytes、逐模块 sha、closure 清单、roles：character/grounds/pairs→instance+template）、`closure/`（输入模块闭包副本，A→B 冷消费用）。
- 正常运行（无生成器）：`autographics-authoring --open <scene> --behavior <world.at> --behavior-api triggers2d` 进入真实 Run；driver 侧消费为 `Storage::load_project(scene) → Host::with_project → start_selected_play`（实测路径）。

## 3. 研究端口与文法面（冻结，全部类型驱动、无名字特判）

| 端口/形态 | 识别 | 生成 |
|---|---|---|
| `character2d <name> { position, mesh, material }` | 新研究元素（词法零改动） | 场景 row id=1（在位=声明位置）+ `ag_char_bind(1)`；宿主定义内 `refs.<elem>.move_and_slide((dx,dy))` → 置 `_n2d_moved=1` + `ag_char_move(dx,dy,0.002)` |
| `solid2d`（仅根 view） | 地面端口 | 场景 row id=2..（在位）+ `ag_char_solid(<id>)` |
| `trigger2d` | shape 缺省 AABB(0,0.0)；`shape: circle(radius: r)` → (1,r) | 场景模板 row（停靠 (-30,2)、invisible）+ spawn + `ag_trigger_bind(pair,1,1,handle,shape,radius)` |
| `camera2d`（≤1，仅根） | bounds 4 元 + view_size 2 元 + target | `ag_camera_follow_2d(1, minx,miny,maxx,maxy,vw,vh)` |
| 参数 float/int | Number | SoA 数组（字面量绑定）；表达式绑定在 owner 上下文逐用点内联（无 SoA 无 getter） |
| 参数 `msg` | MsgEvent 事件端口 | emit → 绑定表投递；载荷常量列 `<prefix>_bind_<EVT>_<n>=[..]`（枚举内联判整值）；无载荷根父直调 |
| 参数 `ActorRef2D` | actor 端口，绑定 `refs.<inst>.actor` | 角色宿主 → `ag_actor(2)`（零计费）；触发实例 → 该实例 handle 数组 |
| 参数 `Mesh2D`/`Material2D` | 资源 symbol 端口，绑定 `resources.<name>` | 模板行 kind/scale/rgba；实例 props 可覆盖定义元素缺省 |
| `input.axis(Left, Right)` | g11-2d 差值轴 | `(ag_action_held(1) - ag_action_held(0))`（held 计费 2；Left=-1/Right=+1/都无与同时=0）。legacy 冻结保持 `ag_action_held(1)` |
| `input.pressed(Enter)` | 消耗性边沿 | `ag_action_pressed(3)`（计费 1；动作表 0=Left/1=Right/2=Jump/3=Enter，与宿主 harness 一致） |
| `round.request()` | 重开服务 | `ag_round_request()`（计费 1，accepted 候选由宿主消费） |
| trigger `on_enter/on_stay/on_exit: .Evt` | phase 绑定 | 路由读 `ag_event_phase()`：1=enter/2=stay/3=exit → 对应事件处理器；未绑定 phase 消费不投递 |
| 模板 id | 声明序 | 角色=1、根 solid=2..、触发实例模板按根 view 声明序（G11 夹具=原 001 的 3..8） |
| pair/slot | 声明序 | 触发实例发射顺序 0..7（G11 夹具 coins=0/1/2、hazard=3、goal=4、void=5 = 原 001 pair 语义位） |

诊断（执行前带 file/line/构造定位拒绝）：缺/错导出模块、未知实例属性、错端口类别值（如事件端口绑 actor）、事件未声明/无处理器、phase 绑定带载荷、shape 非法、role 元素越界（solid/camera 出根 view）、character2d 非 1 个、模板几何缺失、绑定事件混载/混合事件名、condition 不可译。

## 4. 调度与预算（冻结，全部实测）

生成 `app_update` 固定顺序：根 Life.Tick（重开检查读 tick 起点状态）→ 实例 Life.Tick（声明序；Hazard 仅推进巡逻状态）→ 事件段 → 动态位置同步（仅 next==0）→ 条件差量（真 spawn/despawn+重绑）。
事件段：`n=ag_event_next()`；n==1 → next 的 pair+phase 路由；`_n2d_moved==0` 且 n==1 → 再读最多 2 条（静止 3 条封顶）；moved==1 → 恰 1 条。`_n2d_moved` 由 move_and_slide 包装前置置位、每 tick 开头清零。
重开：根 Tick 在 tick 起点状态 ≠Playing 时读 `ag_action_pressed(3)==1` → `ag_round_request()`。**声明差量**：与原 001 相比，"胜/败同 tick 的 Enter 边沿"不被消费（原 001 在事件处理后读边沿）；下一新鲜边沿照常重开。51 项中无同 tick 胜+Enter 用例；`held_enter_no_double_restart`/`won_enter_opens_new_round` 语义不变（实测）。

| 分支 | 账（计费） | 实测 |
|---|---|---|
| 初始化 app_start | init 预算（非 32 池） | 6 spawn+6 bind+char bind+solid+camera 无故障 |
| 普通移动 tick | held2+sweep26+next1+sync1=30 | 无故障 |
| 普通静止 tick | held2+next1+sync1=4 | 无故障 |
| **移动收集 tick** | held2+sweep26+next1+pair1+phase1+despawn1=**32 恰满** | 第 33 计费调用=fault 10（g11_cpu_budget 实测） |
| 静止三收集 tick | held2+3×(next+pair+phase)+3×despawn=14 | 无故障（V-03 三收集用例覆盖） |
| 胜 tick（移动） | 31（重开检查读 tick 起点状态，无 pressed） | 无故障 |
| 终局后 tick | next1+pair1+phase1(goal stay)+pressed1=4 | 无故障 |
| 重开 tick | next1+pair1+phase1+pressed1+request1=5 | 无故障 |
| 危险/越界失败 tick（移动） | 31 | 无故障 |

VM/生命周期全部复用未改 SDK：1000steps/2ms 分片、100000steps/120resumes/250ms、30s init/2s tick、8 slots/8 pairs、代次 handle（旧代跨 Run fault 33/32）、Pause/Step/Resume/Stop 语义、8ms pump 目标独立报告。用户源闭包 ≤256KiB（G11 world 9350B）。

## 5. 夹具与生成物（冻结）

`fixtures/g11-2d/{main,main-alt,resources,types,player,coin,hazard,goal,void_zone,patrol}.at` + 变体根（triple/stay/bag2/geometry/generation，均真实作者源变体——非生成源文本替换）。几何/参数/颜色=原 001 基线逐值；角色声明 spawn=(2,0.5)（原 001 场景 (2,1)+app_start 写 y=0.5 的两步合并为声明式单点；x=2 与行走线 y=0.5 可观测面不变，51 项无 y=1 观测点）。生成物 `generated-g11/<world>/`（world.at/scene.agscene/identity.json/closure/）为冻结数据，再生必须逐字节等价。

## 6. 原 51 项 → 新链关闭映射（owner: driver tests/node2d_g11_flow + pilot gates + verify_plan017）

| 原 oracle | 新链关闭（观测位置等价适配：pair=0/1/2/3/4/5 同语义位；模板 id=3..8 同分配规则） |
|---|---|
| 01 import_two_templates / 02 three_distinct_item_templates | 场景文档 8 行 + 物品三异色/异形断言（冻结 scene.agscene） |
| 03 start_without_fault / 04 six_pairs_bound / 05 six_instances / 06 template_ids 3,4,5 / 07 first_generation | g11_walk init 段（实测） |
| 08 walk_collects_three_and_wins / 09 three_items_destroyed / 10 three_distinct_ids_each_once / 11 collections_are_ordered_ticks / 12 destroyed_never_stay_or_exit / 13 hazard_never_touched / 14 win_alone_restarts_nothing / 15 no_fault_after_win | g11_walk（实测） |
| 16 windowless_png_two | GPU 用例（T-05）：won 后 capture，尺寸/字节/命令数 |
| 17 won_state_freezes_motion / 18 won_enter_opens_new_round | g11_walk（实测） |
| 19 goal_entered_without_three / 20 goal_requires_three / 21 bag2_enter_no_round | bag2 变体世界（源级：rose 缺席→旗标 1→goal 永不赢；enter+stays>0；Enter 不换局） |
| 22 alt_fails_at_hazard / 23 alt_trajectory_differs / 24 failed_freezes | g11_alt（实测；hazard 进触 x 带 (7,9.6)） |
| 25 restart_new_run_tick0 / 26 restart_rebirths / 27 restart_restores_initial / 28 restart_admits_empty_tick0 / 29 old_run_not_replayed / 30 held_no_double_restart | g11_alt（实测） |
| 31 void_zone_fails_oob / 32 oob_freezes | 主世界左行用例（T-04/05） |
| 33 paused_no_motion / 34 single_step_one_tick / 35 stop_restores_document / 36 single_step_geometry_result | triple 世界 harness 用例（T-05） |
| 37 same_tick_three_engine_block / 38 same_tick_three_counted_once_each | triple 变体世界（源级：三币同位 (2,0.5)）：单 Step evaluate 发 3 enter、下一静止 tick 3 读 3 销毁 |
| 39 stay_not_recounted_one_enter / 40 stay_cannot_be_counted_as_enter | stay 变体世界（源级：amber 永不清旗标→不销毁；goal 需三旗清→不赢）：pair0 恰 1 enter + ≥5 stay、活体保持、goal 进入不重开 |
| 41 geometry_variant_admits / 42 circle_corner_silent / 43 circle_tangent_hits / 44 aabb_circle_distinguishable | geometry 变体世界（源级：AABB@角(2.9,1.4)、circle@角、circle@切(2.9,1.0)，静态 dir0=0）：三 pair 进/不进/进 |
| 45 hidden_still_triggers | engine-fact 原始 Auto 探针（spawn+ag_object_visible(0) 后 enter 照发、slot visible=false）——SDK 事实，非作者能力 |
| 46 old_gen_never_auto_reattaches / 47 explicit_rebind_new_gen / 48 same_slot_new_gen_identity | generation 变体世界（源级：旗标 t==3 清/t==4 置 → despawn/重生同 slot 重绑；enter ticks [1,≥4]、gen2 观测） |
| 49 stale_handle_rejected | engine-fact 原始 Auto 探针（旧代 handle 操作=fault 33） |
| 50 same_tick_both_moved_final_positions / 51 previous_position_not_used | bothmove/away 变体世界（源级：对走组件/每 tick 远移组件）：enter tick==首重叠 tick；away 永不 enter |

变体世界全部经同一生成器 lowering（作者源级受检变体，无 app_start 文本替换）；engine-fact 探针（45/49）为 pinned SDK 事实的等价测试输入，不计作者能力。

## 7. 阶段门

- T-02 go：本合同 + 三探针（已满足）。
- T-03..T-05：按 §3/§4/§6 实施；改 §3/§4 规则或预算先修订本合同再实施。
- verify_plan017 --controls：身份绑定、再生字节等价（全部 generated-g11）、夹具变异敏感、缺模块/坏端口诊断、篡改检出、closure 清单一致。
- 消费 A→B（T-06）：B = 独立导出目录 + closure/ + 冷 .sdk-source（prepare_sdk --from-snapshot）+ 独立 target，`--locked --offline` 同输入同结果；B 不借生产者源树/target/网络。


## 12. 合并收据（2026-10-31 merge）

2026-10-31 merge prepared/landed：`stage: merge | plan_id: auto-ui/PLAN-017 | plan_revision: 1 | completion_kind: delivered | prepared: verified | landed: verified | delivery_commit: 9a8a5be60b19d9a385f87fbf1828857389013c9d`。R2 pass 绑定 bf02061；master 在复审窗口漂移至 598347a（PLAN-107，触及 auto-ui-runtime/iced renderer 与 spec history docs——不在本计划测试图内）；worktree rebase 到 master，旧→新映射 b02ba75→b5de16d、3f5fe1d→d29ac00、ae8ecef→b0f6daa、ba38ebc→0d42093、f865853→3822bac、83a1480→81f17fe、bf02061→9a8a5be，`git range-diff 7a53d13..bf02061 598347a..9a8a5be` 七行全等（安全重写证明）。T-08 冻结定向门在交付 tip 9a8a5be 全绿：dialect 全量 95+6+2=103 0 failed、compiler 全量 1127+9+6+1+3+19=1165 0 failed、pilot legacy 9/9+gates 10/10、verify_plan016 PASS（f66b1b92 冻结值）、verify_plan017 六控制 PASS、driver node2d_g11_flow 15/15（40.43s 含 GPU）、legacy node2d_flow 6/6、fmt clean、pilot src 0 warning。主检出 ff-only 落地（无 merge commit），master tip==9a8a5be；他者 WIP（007/014/autoui 设计/012 r1 附件/README）保持未提交未触。

2026-10-31 canonical：`checkpoint: canonical_verified verified`。SD-01 落地本仓 docs/specs/auto-graphics/node2d-g11-authoring.md（sha256 a8b5b72403653521965c1b5024c826473699d178ed374c1d1b7bf3c17b06d958，落地 commit 内核验一致）；主检出 smoke：dialect node2d_pilot 6/6；无派生 specs.json/index.json 写入（PLAN-106）。

2026-10-31 windowed public launch observation（D-04 记录）：`autographics-authoring --open <generated-g11/main/scene.agscene> --behavior <world.at> --behavior-api triggers2d` 于交付 tip 实测：首次因缺 scripts/<sha>.at 项目布局即时报 io 错误退出（预期拒绝面）；物化脚本副本后重启，进程完整运行 30s 观测窗（exit 124=harness 超时终止），零错误输出——场景准入与事件循环真实运行。深度独立 GPU 终审窗口按 D-04 保持 pending 记录（复审 R1/R2 已含离屏 GPU 电池 15/15）。

2026-10-31 archive：`checkpoint: archived | completion_kind: delivered`。本 Plan 移入 docs/plans/archived/ 并置 status: archived（合同折叠为 §11、合并收据 §12；docs/plans/attachments/017/ 按裁定删除）；README 017 行更新为 archived（仅本行选择性暂存——工作树含 007/014 他者 WIP 保留未触）；T-01..T-09 收据关闭，T-10 随清理关闭；任务计数 10/10。

2026-10-31 cleaned：`checkpoint: cleaned verified`。（清理后回填实际结果）

2026-10-31 landing≠deployment 观察：本计划触面为 experimental 研究引擎（node2d pilot/driver tests）与 auto-ui-dialect 显式研究模块（生产 widget 路径 opt-out，016 实测沿用），生产 backend/daemon/rootUI 不消费本计划触面，无重建重启要求；本计划未触及生成 web bundle 或 daemon 二进制。批回归到期检查：PLAN-017 非 5 整除；读 docs/plans/.last-batch-regression.json 于主检出复核后记录。
