---
plan_id: PLAN-017
status: drafting
feature_name: AutoGraphics 完整G11二维Node作者能力
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 0
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
| T-01 | work | 后续用户启动，016实际交付 | 读当前Spec/交付/owner与两图依赖锁，创建唯一ui-017树，固定输入与原51清单 | AC-01 | [ ] |
| T-02 | work | T-01 | 最多两接法完整main/alt整链及全部峰值先证；冻结contract/CLI/typed端口/调度/51映射/准确门；不成立needs_replan | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 结构化场景/资源/引用/模块与诊断，删除新模式的业务名特判，改名/重排/变资源正反例V-01 | AC-02 | [ ] |
| T-04 | work | T-03 | 完整输入/Life/多handler与父receiver/phase分类，差量实例与原事件调度；V-02/03 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 完整51语义与GPU、main/alt、重开/旧代/owner20轮/原额/VM负控与旧pilot验证；V-03..05 | AC-04/05 | [ ] |
| T-06 | work | T-05 | main/alt公开准备/Run、fresh固定消费A→B/损坏修复/旧触面、verify controls/fmt/warnings，SD候选与work移交 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 同树独立final review逐AC/V/S/SD/51映射/遗漏，finding回原树work，pass列冻结门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结当前提交/依赖/输入，执行定向并集或有效复用，实际合本仓master，保他者WIP | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01与来源/准备运行说明沉淀，归档本合同/导航；不手写派生索引 | AC-08/09 | [ ] |
| T-10 | merge | T-09 | 逐树fresh guard、clean/祖先/路径核验后清树分支空组，复核可重建消费身份和证据恢复 | AC-09 | [ ] |

## 9. 复审记录

2026-10-31 / stage: new / plan_id: auto-ui/PLAN-017 / plan_revision: 1 / outcome: pass（规划合同可审阅，实施启动尚未授权；T-02为实施内有界可行性门） / next: 后续用户启动后work T-01/T-02。独占本地取号017、下一号018；引用016实际交付/原51检查和真实源码观察，全部新用例/命令未执行。未建树、生成资源、编译/GPU、创建agent或修改Gallery实现。

2026-10-31文档预检：0..10节顺序、10个T/9个AC/6组V-S/1个SD、owner与前置/后继方向、全部本地Markdown链接、唯一号源和drafting0/10检查通过；原51检查名与内容仓历史回执逐字对齐（3D25另保留）。git diff --check通过；只改规划/设计/导航与号源，生产源码/SDK/生成物零改动。这不是实现测试、可行性go或独立final review证据。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 新模式精确typed refs/角色/事件端口与CLI/输出布局 | T-02在既定Node/view-2d/Life/msg/use内冻结常规细节；改变语义/范围/额度须用户决策 |
| D-02 | 完整32原额路径（含phase与左右）及调度可行性 | T-02真实分支校准；若不足带反例needs_replan，不静默提高/降验收 |
| D-03 | 执行时当前语言只读依赖、014/语言900共享owner窗口 | T-01重绑实际SHA/锁/解析装配点与写面；不要求其全量结束，也不覆盖其WIP |
| D-04 | 独立终审上下文/实际GPU运行窗口 | T-06按届时授权资源交接；未取得独立证据保持pending，不冒称pass |

没有需要用户现在重选的作者方向。当前唯一允许动作仍是规划；017交付不自动启动007。
