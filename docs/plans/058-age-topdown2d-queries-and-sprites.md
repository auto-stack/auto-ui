---
plan_id: PLAN-058
status: executing
feature_name: AGE M2俯视二维公共组合与ACE作者接线
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-19
plan_revision: 8
current_step: 3
total_steps: 10
depends_on: [auto-ui/PLAN-055, ag-examples/PLAN-022, ag-examples/PLAN-021, ag-examples/PLAN-026, auto-lang/PLAN-981]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-topdown-composition.md]
touched_goals: [GOAL-008]
affects: [node2d-authoring, query-sprite-composition, autographics-auto-driver]
---

# PLAN-058：俯视二维控制、查询与精灵公共组合

## 0. 变更摘要

2026-11-19 **r8（用户裁定，范围有界修订）**：AC/A2R 格与双后端义务**原封转移**给新建后继 **PLAN-103**（`103-topdown2d-ac-a2r-wave-consumption.md`，drafting，等 1068/T-04＋981 AC/A2R 域端口＋UI-W2-AC/W3-A2R 波落地后开工）；本计划收口为 **AV 面**——T-04/T-05 重定义 AV 格（已交内容＋剩 OS 真实输入、V-S04 显式闭环、V-S05 冷 A→B AV 腿），T-06 保留 AV 配方/冷包、双后端配方转移，SD-01 措辞对齐实际支持域，AC-10 与 V-PHASE-GFX 多端条款转 PLAN-103 承接；原 V 行 ID/输入/expected/独立 oracle/包络**不减**；触碰路由缺口（17-op 无 dispatch）随 V-S03 完整形归 PLAN-103（scenario 侧 2:1 已证，作者面路由等 IF）。任务 ID/已完成证据全保留。

2026-11-18 r7当前：T-03完成（3/10）。**981 动态世代接口已消费（`12cb146b`，pin 刷 `0a167f611`）**：`AgeGfxCall`/`dispatch_with_ctx` 会话事实接进 `NeutralGfxHost`，判罚点落 UI058 车道——fault-13 判定（同 callback 二查询杀旧 token/纯 callback 边界/跨 Tick，载荷双面命名 tick/callback/epoch）与 fault-33 判定（死句柄写具名拒携会话事实，真实车道 `frame_after_despawn` 驱动）4/4 绿，连续链与双实例用例骑乘验证。**T-05 AV 阶段收尾已交（`350af5b8`）**：GPU **同宿主连续帧链**（单渲染宿主 6 连帧，AV 行位经 `Play::call_script`→`ag_biz_apply`/`ag_set_local` 逐帧应用，渲染路径零业务；逐帧质心=当帧 AV 行投影≤1px、逐帧位移匹配行差 6.00/6.50/6.50/3.00/0.00px）＋**use 实例目标**（fixture 升级 Target 定义 node+两实例行，ForNode 定向 2:1 Touch→实例状态 2/1 独立，探针+主 fixture 整合）。**AC/A2R 阶段门精确阻塞**（1068 drafting 0/10、981 AC/A2R 域图形端口未交、W2-AC/W3-A2R 未落地——见 D-07），按合同不以 AV/A2R 代 AC、A2R 不先于本包 AC。T-04/T-05 均未勾（原全量义务不减）。

历史：2026-11-18 r7：T-04 G1/G2 正式作者形已接（`0e67b1cd`，AV phase 内）：981 G1+G2 pin `6e456794d` 落地后，fixture 改真实 `on Life.Tick`（hook-local `let q Sweep2D` + `if/else`）＋源侧 `layer: render-only`（typed `AgeSceneRow.layer` 替代宿主行名表），AV runner 改经 scenario 状态函数驱动；40 Tick 同宿主连续帧逐 Tick oracle 复跑同数（第35首触停位8.6/每 Tick 32 credits），GPU 同 Tick 预证复跑像素字节同前（3.00px/0.00px）。两条语言纪律实测入档：021 token 走查拒"后读"、每函数静态准入拒计费链复制。

历史：2026-11-18 r7：T-01/T-02/T-03完成。新增**公共render-only行层**：`autographics.topdown2d/1`（profile 30/version 121，首个同assembly含query+sprite的档）＋`scene/9`可选`layer`＋`behavior_renderonly_v1.at`（0 credit行表）＋render-only感知segment/sweep单元（冻结v3/v4复制＋候选跳过；render-only作ignore/mover→fault 60）；旧22 profile/10 unit/价目/号逐字节不变。E1“65536代理”被对抗反例否决并删除。旧021/022消费者11/11、4/4；controls 6/6。981 `D-981-AGE-F32-SURFACE` 已关闭（AV `be8480cae`+A2R `a4950f0b5`，auto-lang master），跨仓状态观察另登记`D-981-F32-CONSUMER-WINDOW`。T-04 AV 阶段（`b071634c`预备/`f6418120`三臂/`7a1503b7`整链/`ba85f33f`GPU同Tick预证）已交。

历史修订：2026-11-18 r6：T-01/T-02完成（2/10）。T-02机制冻结=**碰撞代理与外观分离**：统一静态场景行的数值语义成立，但旧queries-2d/sprites-2d互斥、冻结v3/v4无per-row碰撞位（可见地面入候选即fault 9），故按合同第二机制分离；四要件实测=一个逻辑owner/同Tick共同接受/无重复Tick或错位/无隐藏阻挡。预算实测：32 credits/回调合法、第33次与40>32具名fault 10零效果sticky、attempt计数无回退且fault终止Run、额度按Tick重置。GPU离屏证明查询世界位与精灵像素同Tick一致（4.0→210px、查询停位8.58→400px）。T-03接公共render-only行/层与生成器排除地面；原AC/V/预算不减。

历史修订：2026-11-18 r5：981侧T-02接口联冻（IF-981-GFX-AV-SEAM）由本包回签；32 credits/世界回调（Tick）计量统一为「下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝」；T-02先证接口冻结为已有静态f32行值＋宿主行写，真实Auto移动积分等981 f32 AV切片（不以int近似）；10任务、0完成，仍仅规划。

历史修订：2026-11-11 r3：AG026已11/11签章，M1前置已闭；最终作者链消费981/SOURCE/SESSION/RUST及所用LANGUAGE的A切片正式来源，不等981整包归档。保原最低对象/预算，新增可见非阻挡地面组合AC-11/V-S08，为AG029国风首切片提供公共图形接面；不承担种植/存档/素材或重做055研究。10任务、0完成，仍仅规划。

历史修订：2026-11-09 r2：归入[合并AGE M2](../design/autographics/age-three-milestones-and-ace-m2.md)二维线。中立图形服务与ACE接入并行，最终Node作者链走981的同源AV/A2R，不扩旧SDK源码生成器。原最低样本/图形预算/兼容义务及10任务保留，新增AC-10/V-S07；drafting、0/10，仅规划。

承接055真实研究与AG022需求输入，交付有限、显式版本的正常Auto俯视二维组合作者链：无重力四/八向控制、单次sweep停墙、最近交互查询、具名精灵帧与普通use组件局部事件。不是完整种田游戏。本轮只规划；055已交研究结论不等于本组合go，正常作者终验仍须原M1必需门关闭。见[本批路线](../design/autographics/m3-next-wave-execution-order.md)。

## 1. 目标

普通Node/use/model/msg/on/Life.Tick承载业务；一Player、两静态障碍、两个同定义交互物构成最小完整链。非零输入轴归一，斜走不快于直走；一个Tick最多一次sweep并停于首次接触，不要求自动滑墙。查询/世界位置与当Tick精灵位置一致，两个交互物的状态/事件各自独立；最近目标、同距顺序、超距不触发由公共查询结果＋Auto逻辑表达。

不包含tilemap批绘、任意大图、多角色/动态碰撞、完整背包种植存档、深度排序/2DHD光照、混合UI、寻路/自动滑墙、跨端/预算提升。通用ACE/AV/A2R归979/981，本包只接图形端口和领域声明，不重复实现语言后端。国风首切片由AG029新drafting合同承接，本包不冒完整游戏，不改M1平台游戏/Gallery分母。

2026-11-19 r8：本计划交付域=**AV 面**（同一 Node 源经 981 AV 真跑的全部行为/预算/呈现/故障/恢复验收）；AC 机器执行格与 A2R 真编格由 **PLAN-103** 承接（转移清单见 §5 r8 块），原目标/最低样本/预算/独立 oracle 不减。

## 2. 架构方案

显式版本化能力/profile（T-02冻结）复用020局部组件语义、021 segment/sweep和022 PNG/帧表的中立实现，打通同一owner的查询/移动/精灵。旧SDK681仅作历史兼容基线；新Node经979/981共同前门受检，AV和新A2R通过同合同typed端口消费服务。Auto持业务，Rust只做通用算法；不新增旧world.at生成路径，不把互斥旧profile拼贴当组合支持。

唯一树`D:/autostack/.wt/ui-058/auto-ui`（2026-11-18已建），分支`codex/plan-058-dev`，base `b77998da`、当前code `7165dbad`（T-03）。同组依赖源按实际交付只读，无链接；共享AST/load/translate/API/host写面先核054与其它owner短窗，报告/target/scope独占。T-01/T-02在该树执行；主检出只留Plan簿记。

## 3. 技术栈

旧链调查/兼容输入已核pilot `src/{load,translate,lower_components2d_runtime}.rs`，driver `src/authoring/runtime_sprites.rs`和既有查询/精灵服务；现有CLI `node2d-lower --profile queries-2d|sprites-2d|components-2d-runtime --entry … --out …`分别有不同支持域，不能当现成组合入口。中立服务沿现有driver/renderer owner复用，Node适配沿981接口，不将旧两个Cargo图强并、不复制解释器；新SDK/ABI/构建入口消费981/978正式交付。

## 4. 需求分析与背景调查

2026-11-18执行授权：用户要求UI058启动T-01→T-02先证（统一场景行优先，旧slot实际阻断才试碰撞代理与外观分离），并让981开工f32 AV切片；T-02预算用例钉住两次sweep=32合法、sweep+segment=24后另一Node sweep超额零效果、随后1-credit继续拒、下一Tick重置。证据落`plan-artifacts/plan-058/auto-ui/`，代码只在唯一worktree提交，不改Plan外文件。

2026-11-18当前授权：用户要求立即回签981的T-02接口联冻；预算口径统一为「下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝」；T-02先证用已有静态f32行值＋宿主行写，真实Auto移动积分等981的f32 AV切片。本轮只修改规划/导航，不建树、不跑Cargo/GPU、不改Spec/价目/文法、不勾任务。

2026-11-11当前授权：用户要求修正过时规划状态/981计数，并将国风首切片、正式光影与联合验收细化为可执行合同，复用已交研究、不重复立项；本轮只修改规划/导航，不启动实施、建树、制作素材、构建、运行或发布。既有其他会话WIP与归档/当前Spec保留。

2026-11-05原立约观察（以下状态为历史）：用户授权下一批计划及状态/顺序；该轮只设计/计划/导航，不执行。当时auto-ui主观察64aef780，055/AG022未交付、054有树及实现1f3be333但未交付，AG021 T-03等054。没有本组合canonical，新知识由SD-01在review/merge沉淀，本次不改已交付Specs。

已读Specs `node2d-queries.md`（SHA前16 0d930490f45d9a12）、`node2d-sprites-and-frame-animation.md`（f997bae66305eef0）、`node2d-full-game-composition.md`（e693338b0a88a7e6）。021明确不含sprite/自动slide，query世界不纳spawned slot；022帧写依有限render槽；032要求一角色/相机及固定state/pose字段，不能换重力/业务字段就称通用俯视。源码lower_components2d_runtime SHA6eed9a6c6ac447ae、translate9350491665faf8a0为静态观察。本Plan不假设这些约束已消除。

2026-11-08的M2规划暂停保留为历史；2026-11-09用户提出合并原M2/M3并行ACE接入和2D/3D增强，本轮恢复规划修订，仍未授权实施/先证/素材/构建。055、AG022、AG021已交不重做研究，M1最终原生门由AG026及债务收据判定。补读979/981合同及auto-ui的ace-migration-feasibility Spec。下文原M2必需门均指当前AGE M1，不是等待合并M2最终签章。

## 5. 详细设计

### 2026-11-19 r8 范围有界修订（用户裁定；优先于 2026-11-17 阶段合同的 AC/A2R 行）

用户裁定（2026-11-19）：AC/A2R 内容整体转移给新建后继计划 **PLAN-103**，本计划收口 AV 面并允许在 AV 格闭合后完成 review/merge；PLAN-103 等 1068/W2/W3 落地后再开工。**转移清单（原封，不减）**：

| 项 | 原位置 | 去向 |
|---|---|---|
| V-S01..S08 各行的 **AC 格/A2R 格**（输入/expected/独立 oracle/包络逐行原封） | §6 V 表 | PLAN-103 §5 继承表 |
| **AC-10**（同一 Node 源经 981 在 AV 和 A2R 真实编译运行；AV 面已交） | §7 | PLAN-103 AC-10 |
| V-PHASE-GFX 的 AC/A2R 格与多端"整包拒绝"条款 | §6 | PLAN-103（AC-PHASE-GFX 承接行） |
| T-06 的**双后端** AG023 配方（两入口准确版本/命令/冷包） | §8 T-06 | PLAN-103 T-05 |
| 触碰路由语言缺口（作者面业务→实例事件派发；17-op 无 dispatch 端口） | D-06 具名缺口 | PLAN-103 T-02（IF-TOPDOWN-TOUCH-ROUTE） |

**本计划保留（不减）**：最低样本（1Player+2障碍+2同定义目标＋可见非阻挡地面）、32 credits/物理写 32、独立 f32 轨迹/segment/GPU oracle、旧档零漂移、V 各行 AV 格、T-04/T-05/T-06 的 AV 剩余格（OS 真实输入、V-S04 显式闭环、V-S05 冷 A→B AV 腿、AV 配方）。2026-11-17 阶段合同的"AV先交"行由本块收口；其 AC/A2R 行由 PLAN-103 对应合同接管——原表格文本保留在 git 历史（`b18196fd` 及此前）与本节下方，不再作为本计划现行合同。后继计划未开工不构成本计划的未决 hold（PLAN-103 已立项登记 owner）。

### 2026-11-17 当前阶段合同（优先于下文旧双端排期，原验收不减）

统一执行[三路线strategy](../../../auto-lang/docs/design/strategy/age-m2-three-route-strategy.md)及[引擎扩展设计](../../../auto-ui/docs/design/autographics/age-m2-engine-extension-contract.md)。本轮只修订计划，未启动/恢复实施，不新增完成项；原任务ID、已完成证据、最低样本、独立oracle、容量/许可/资源/冷包和最终产品义务全部保留。

| 阶段 | 最小正式前置/本包结果 | 勾选与后续责任 |
|---|---|---|
| AV先交 | M2A-08、本包所用981源/Schema/typed服务/SESSION端口及上游**AV正式子面**；中立算法/只读准备仍按原先行条款。先以同一普通Node源运行原各V行的AV格，真实GPU/交互/错误/资源/冷包及独立oracle齐 | 同树停写独立phase复核→按owner正式fold交IF-AGE-GFX-AV/G2或G3（含本包号/源/资产/接口/SDK/实际运行身份）；AV格齐不提前勾含其它端的原完整任务。保唯一树/resync，不清树、不冒整包归档 |
| AC随后 | UI-W2-AC、本包AV正式来源、1068/T-04与981 LANGUAGE/FUSION所用AC域端口；沿既有公共owner接同源AC真实机器业务/同host，复用AV输入/原独立oracle | 1068/981生产机器接面，本包消费并补原各V行AC格、真实包/失败恢复，独立phase复核/fold交IF-AGE-GFX-AC/G2或G3；不得用AV或A2R代AC |
| A2R最后 | UI-W3-A2R、本包AC正式来源及981 RUST/FUSION所用A2R域端口 | 保原AV/A2R全部承诺，实际生成/rustc/最终包，补原各V行A2R格后交IF-AGE-GFX-A2R/G2或G3。原整项work、final review/merge/归档只能在所有适用格闭合后勾 |

上游“正式交付”在本包**当前阶段**指对应AV/AC/A2R子面，不反等上游整包归档；不得用未审WIP顶替。UI-W1/2/3签原UI/Widgets分母，不等待本包或新引擎扩展；本包也不等AG031或AutoForge最终签章。原已有A2R能力可复用同身份证据，不要求每次AV开发重编；新增A2R完整证明留第三波。原始G2/G3输入不因后端切换改写为Rust业务或缩减功能。

frontmatter为最终依赖并集，不是所有任务开工栅栏。T-01核已交研究/需求与oracle；T-02可先做中立服务/GPU最低负载先证，T-03推进公共实现/typed端口/领域声明准备，与981语言工作并行。T-04..06的正常Node整链必须消费981/SOURCE、SESSION、RUST及所用LANGUAGE A切片已独立复核/落主的接口来源，和AG026已闭M1收据（9264fac、签章ea0fc59）；raw/原生服务go不替作者链go。981的既有M1迁移/融合不依赖UI058/AG023终验，禁止互等。AG020更名不是前置。

T-02至多两种机制：优先统一静态场景行查询/位置与sprite表现；若旧slot限制必须分离碰撞代理与外观，必须证明一个逻辑owner、同Tick共同接受、没有重复Tick或错位/隐藏阻挡。冻结普通具名资源/组件事件、输入轴/pressed语义、世界单位/dt、无输入/同时反向输入、归一/起始重叠/最近命中/同距顺序、相机和资源身份。具体API不由本合同凭空命名；先证失败需needs_replan，不把raw API成功充正常Node链go。

最低验收就是1Player＋2障碍＋2同定义交互物及一个可见非阻挡背景（保原五对象，不删目标），根及局部状态总数≤8、场景≤128行、原每世界callback32credits/物理写32及原source与资产包络保持；旧1000步每slice属历史VM成本，不等于ACE fuel，T-02联981冻结计量/资源映射与超界反例（预算口径见下「2026-11-18 T-02 联冻回签与预算口径」）。地图仅小范围，不把055三规模都当必须接受；scale以055/AG022实际结果冻结且不得缩掉以上最低对象/行为。完整合法峰值同Tick包含移动、必要query/事件和精灵收口；steps/credits/ms/GPU字节分开计，不以隔帧查目标/停帧写图或复制池省预算。若最小链需提高限额或新的计费策略，T-02列原账/候选账并needs_replan，不越权放宽。

新profile必须版本隔离：旧021/022/032的schema/产物与价格保持，旧M2项目固定锁不自动升级。独立f32轨迹/segment最近命中oracle与GPU位置/帧/像素共同验证；按键四向/斜向/停走/反向同时按、墙/墙角/起始overlap、两目标同距/超距、目标事件只更新自身、改Auto速度/事件后实际变化。暂停/Step/freshRun与旧句柄/坏源/资产恢复下一Run闭环。

T-03..06拟新增driver `tests/node_topdown2d_{flow,storage}.rs`与`tools/verify_plan058.py --controls`（新路径，由T-03建立并冻结命令）。Node声明/能力注册沿981同一接口，不在pilot追加旧生成后端。T-04/05使同一作者源/资产经AV和A2R→Rust真实编译运行，对照独立数值/事件/GPU oracle；T-06交AG023准确源、host ABI/provider、AV运行/A2R构建及冷包配方。通用provider缺口回981/语言owner，本包不复制。真实窗口输入与受控fixture分层。

### 2026-11-18 T-02 联冻回签与预算口径（T-02 冻结项）

**回签绑定**：对象为981 handoff（2026-11-18，`auto-lang/docs/plans/981-ace-m2-age.md` §9；IF-981-GFX-AV-SEAM `ddbda0228`+`ed8b16e01`，981记录auto-ui master `f43ef59b`）。本回签在auto-ui `91d549ea`完成；`f43ef59b..91d549ea`对`experimental/autographics-auto-driver`、`experimental/autographics-node2d-pilot`、`crates/auto-ui-tools`零diff，服务面表（17 op/6族/7端口）快照未漂移。逐项答复与边界见§9。

**T-02 先证接口（本包冻结）**：用**已有静态 f32 行值**（场景行/`AgeSceneValueFn`观察真值，及fixture侧`scene_rows()`的f32 lo/hi/pos）与**宿主行写**（宿主侧写入行身份/可见性/字段/Descriptor；旧链先例`tests/node_queries_flow.rs:101-131`的`_ag_identity`/`_ag_visibility`/`_ag_append_field`/`_ag_spatial_config`）构成验证接口；独立f32轨迹/segment最近命中oracle在测试侧计算。T-02不以f32业务式为前提。真实Auto移动积分（`x = x + vx * dt`、位置/速度f32字段）等981 `D-981-AGE-F32-SURFACE`的f32 AV切片，UI058不以int近似或变通绕过。

**预算口径（与981联冻，UI058回签）**：32 credits/**每世界回调（Tick）**，全Node共享一池，不按帧、不跨Tick累计、不给每模块复制额度。计量为累计扣费，统一写成：**「下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝」**。即：恰好32合法；任何会使累计>32的**收费**调用在扣费前具名拒绝（语言侧静态准入=每函数调用点合计≤32，超界`lower.age-gfx-credits`具名拒零效果；宿主动态计费按同表具名拒），不扣费、不移动、不发布；本Tick内其后的收费调用（含更小价目）一律继续拒绝（sticky）；981记录的「第33」按累计credits解释，不按调用序号。例：两次sweep（16+16）已耗完32 credits，本Tick下一次收费调用即拒。0 credit的端口字段读不产生扣费、不改变计量；cast/sweep/getter价目不动（segment 8/sweep 16/其余服务1/端口字段读0）；物理写32与credits分开计，steps/credits/物理写/ms/GPU字节不互换。若最小链需提高限额或改计费策略，T-02列原账/候选账并needs_replan，不越权放宽。

**T-02 冻结结果（2026-11-18实测，机制/预算/呈现）**：
- **统一场景行数值语义成立**：同一f32行快照分别驱动查询（宿主行写`_ag_identity/_ag_visibility/_ag_append_field/_ag_spatial_config`）与呈现（文档行值→PNG精灵），查询`Tick`与渲染`frame.tick`同为1；sprite像素与查询世界位按相机投影逐点一致（行值4.0→px210,190；查询停位8.58→px400,190）；地面174716/164171px真实可见。
- **分离原因不是slot容量**：queries-2d（`objects2d/1`）与sprites-2d（`sprites2d/1`）互斥，无单assembly组合；且冻结v3/v4候选集=全部2D行、**无per-row碰撞位**，`_ag_spatial_config` box flag=0得具名fault 9（整体失败，不跳过），"可见但不可碰"不能用同一行表表达。
- **冻结机制=碰撞代理与外观分离**：地面=render-only行（可见、不进查询候选表），查询表仅solid行；一个逻辑owner（同一Row/f32快照）、同Tick共同接受、每Run恰1次Tick无重复/错位、无隐藏阻挡。055的t=0/start_overlap/no-move作为可判别变异反例（地面误入候选即复现）；021/022旧产物字节与价目不变。T-03据此建公共render-only行/层归属、生成器排除地面及永久用例/V-S08。
- **预算实现口径（与统一句一致）**：`_ag_cast_charge`先累加后判界，被拒调用使attempt计数不回退（实测40→41）；fault 10具名且Run终止；额度按Tick重置（连续3 Tick各32合法），故障不因Tick清除；零效果=不移动、不发布、token 0。

**T-03 冻结结果（2026-11-18实测，公共render-only行层）**：
- **新profile `autographics.topdown2d/1`**（profile 30/version 121、CLI `topdown2d`；注册唯一性由controls C5守）：显式组合2D栈（render-only感知的segment/sweep/motion＋character/camera2d/objects/triggers/sprite＋render-only行表），是首个同assembly含query+sprite的档；旧22 profile/10个冻结unit/价目/号逐字节不变（C2/C5；`objects2d`仍16/105）。
- **render-only行表**（`behavior_renderonly_v1.at`，0 credit）：宿主按文档`layer`逐行`_ag_ro_reset/_set`（0/1二值、≤128行）；render-only感知的segment/sweep单元在候选循环跳过（冻结v3/v4复制＋跳过；render-only作ignore/mover→具名fault 60）；行仍持真实AABB，`Descriptor::query_candidates()/is_query_candidate()`为公共候选视图。
- **格式隔离**：`autographics.scene/9`＝/6形状＋可选`layer`（kebab；仅2D叶；缺省=legacy collision）；≤/8含`layer`或旧profile载入含layer文档→具名拒；旧5..8文档encode/hash逐字节不变（无layer键）、104个`generated-*`场景0 layer。
- **预算**：32 credits/回调不变（segment 8/sweep 16/其余1/字段读0）；跳过0 credit；地面仍占1行/1 descriptor（≤128），不新开池。
- **V-S08**：正例地面174116px可见、非候选（sweep→ord3 t=0.046000004/停位8.58；segment→ord5 t=0.25；credits=32）；变异去layer→t=0/start_overlap=1/位置4.0且segment被地面t=0抢选；**对抗反例**（地面置(65540,65540)、射线穿旧E1代理点/横跨真实box）→render-only hit=(0,0)，同文档去layer→均命中（t=0/t=0.2）。E1代理已删、非承载。
- 旧021/022消费者：`node_queries_flow` 11/11、`node_sprites_flow` 4/4；`public_audio_compat` 3D六项为T-03前已漂移的预存红（同源stash复现；3D unit `9c13cca4` vs 基线`7bfbac0d`），非本任务回归。

**T-04 AV 阶段结果（2026-11-18实测，AV phase pass；T-04 未勾，AC/A2R/GPU待后续波）**：
- **正式来源/pin**：981 record ABI `5ee4d23c0`（receipt `381d7a364`，`D-981-GFX-PORT-ABI` closed）；`.sdk-source` overlay 刷新到 auto-lang master `64e8380f4`（681 base+1432 overlay；`prepare_sdk --verify-only` 与 `verify_overlay` 双 PASS），提交 `sdk/overlay/overlay.json`；driver 直依赖 `ace-av-core`（Value/StrView 映射）。预存红：`viewport3d_depth_probe`/`squash_feasibility` 缺 `Scene3dContent.hud_overlay`（PLAN-013 字段、测试未更新，与本 pin 无关，未修）。
- **前门**：`main.at` 经 `compile_age_package` 真编译（1 node/6 scene 行/符号 `p7_plan058m4_main_tick_step`，37ms）；`on Life.Tick` 保留原形（assignment-only，G2待）；AV 驱动走无参 `pub fn tick_step()` **宿主状态形态**（`row.world` 读位、`input.*`/`tick.dt`、**恰一次** sweep 停墙、`row.place`、`segment` 最近、`render2d.spawn/frame` 同 Tick；每 Tick 恰 32 credits）。
- **适配**：`impl AgeGfxHost for NeutralGfxHost`——`Value↔GfxArg`（含 `StrView`）、3 个 record op 复用三臂、13 个标量/行/精灵 op、fault 去 `age-gfx.` 前缀；收费/sticky 仍归 981 `AgeGfxSession`，provider 不计费。
- **runner**（`tests/plan058_t04_av_chain.rs` 3/3，74/33/45ms）：40 Tick 逐 Tick 对独立 oracle 比位置/sweep(node,t,start_overlap)/segment(hit,node,t)；首次接触**第35 Tick**、停位 **8.6**、t=**0.5000353**，此后不穿透；同 Tick spawn 位置=行位、1 token 复用；输入 (1,0)/(1,1)/(0,0)/(−1,0) 与改源速度 8→4（单 Tick 0.0666667）随动；32 合法/第 33 credit sticky 拒/48 credit 拒且零效果/`begin_tick` 重置/`stop` 具名拒。
- **AV/GPU 同 Tick 预证**（`ba85f33f`）：AV 链跑 36 Tick 后取 34/35/36 行值注入同一 6 行 sprites2d 离屏场景（相机绑停放的 spectator、no-op probe 源），每 Tick 恰一步即 capture 800×380，atlas alpha=255 四色掩码质心 vs `camera.screen_2d` 投影：tick34 row 8.533329→proj(425.33)/质心(425.00)、tick35 row 8.6→proj(428.50)/质心(428.00)，Δ≤1px；34→35 质心位移 **3.00px**（期望 3.17，>1px）排除晚一帧；35→36 位移 **0.00px**（停 8.6 不穿）；ground 174752px、wall 质心≤2px、`frame.tick`=1、每 Tick 32 credits。渲染源为 no-op probe（不在渲染路径重跑业务）；G1/G2 与 AC/A2R 不在本子面。
- **边界**：G1（AGE 源侧 render-only 声明）与 G2（hook-local let/memoized）仍待 981；本阶段 render-only 地面由宿主按 scene 行名承载；GPU 像素/AC/A2R 与 `use` 实例归 T-05/后续波；`render2d.frame` 的 str 仅字面透传（G4 资源名语义待资源身份）。
- **G5 pin 刷新（2026-11-18，981 G1+G2 落主后）**：`.sdk-source` overlay 刷新到 auto-lang master `6e456794d`（981 G2 = `on` 体 hook-local `let`+`if/else` 作者面 + G1 = `shape2d.layer: render-only` typed 行事实，叠加 980 P3 style 面；681 base+**1434 overlay**，`+2` = 981 两个新永久用例文件）；`refresh_overlay.py`→`apply_overlay.py` 双 PASS、`verify_overlay`（base 681 + overlay 1434）与 `prepare_sdk --verify-only` 双 PASS；提交 `sdk/overlay/overlay.json`（`65a6b2cf`）。上边界的 981 侧前置（G1/G2）**已解除**——T-04 前门可改真实 `on Life.Tick { let q Sweep2D = query2d.sweep(…) if q.start_overlap {…} else {…} }` 作者形与 `layer: render-only` 场景行（981 永久用例已证 provider 只收一次 sweep、分支复用同一结果）。
- **动态世代接口交接（2026-11-18，981 第三批 `82aecc9ad`，T-04 消费合同）**：981 已把**宿主侧 token/世代事实**交到会话层——`AgeGfxHost::dispatch_with_ctx(symbol, args, ctx: &AgeGfxCall)`（**默认转发**旧 `dispatch`，既有 `NeutralGfxHost` 不改即兼容；要消费世代事实时实现本方法）。`AgeGfxCall { tick, callback, query_epoch, credits_used, symbol }` 语义：`run_def`=一次 callback 世代（`callback` 单调、`query_epoch` 归零）；`query2d.segment/sweep` 派发各推进 epoch（1 基单调、非查询 op 不动）——**同 callback 第二次查询 ⇒ epoch 前进 ⇒ 先前 token 失效**（021 fault 13 的宿主侧动态事实，与语言侧静态走查 `source.age.query-token-stale` 同口径）；`credits_used` 为本次记账后的累计（预算对账同一份）。**判罚分工（用户裁定）**：fault 13/33 的判定与载荷由 **UI058 宿主**实现（用 `query_epoch` 给自家 token/句柄记账、打真实 fault 载荷）；981 只提供会话事实、不重复判。981 侧已实证（`plan981_gfx_hooks` AV 真跑）：同 callback 两查询 epoch 1→2、标量 op 不动、跨 callback epoch 归零 + callback +1、跨 Tick（`begin_tick`）账本/世代边界、每 Tick 32 credits 记账值逐派发到达。UI058 接法：把 T-03/T-04 的 host 切到 `dispatch_with_ctx`，用 `AgeSession`/连续帧 runner 的**同一批双实例用例**接 ctx 断言：①同帧第二次 sweep 后旧 token 读被判 13（宿主载荷）；②跨帧（新 run_def）token 全部失效=世代 2；③跨 Tick 重置。会话访问器 `query_epoch()/callback_seq()` 可作断言锚。

**T-04 G1/G2 正式作者形结果（2026-11-18实测，AV phase 内；T-04 未勾）**：
- **pin/resync**：分支并 master（merge `cbe350b4`，含 G5 overlay pin `65a6b2cf`@auto-lang `6e456794d`）；`apply_overlay.py` 自 pin 树精确物化（auto-lang 主检出已前移 982 合并 `600d31527`，哈希不符按设计拒）→`verify_overlay` PASS（681 base+1434 overlay）→`prepare_sdk --verify-only` PASS。
- **正式 `on Life.Tick`（G2）**：fixture 删 `tick_step` 宿主态形；Tick 体=hook-local `let`（`q Sweep2D` 等 14 个）+归一化/记录 `if/else`；runner 经 `compile_age_package_with_scenario`（1 实例+Tick op）驱动 `__age_state_0_last_node`（字段 init+handler 重放，同一次重放执行全 Tick 体恰一次）。
- **源侧 `layer: render-only`（G1）**：`rows_from_scene` 改读 `AgeSceneRow.layer` typed 事实（`Some(RenderOnly)→RenderOnly`），宿主行名表删除；cpu 面 `Symbol("render-only")` 孪生同测；去 layer 变异→`layer=None`→ground 重入候选（sweep node0/t=0/overlap，055 反例经源事实复现）。
- **同宿主连续帧验收**：同一 `AgeGfxSession`+`NeutralGfxHost` 连跑 40 Tick 逐 Tick 对独立 oracle（位/sweep node,t,start_overlap/segment/恰1 spawn+1 frame），第35首触停位8.6/t=0.5000353，36..40 t=0 且 `start_overlap=true`→`last_node=-2`（分支记录事实），每 Tick 恰 32 credits；GPU 同 Tick 预证复跑 34/35/36 质心vs投影≤1px、34→35 3.00px、35→36 0.00px，证据与前一收据**逐字节同**（`d191558e`/`23af8c82`/`e7bf2a64`——真实作者形与宿主态形渲染输出全同）。
- **两条语言纪律实测（作者形约束，入档）**：①021 token 走查——回调内第二次 `query2d.*` 使此前绑定的名字（含经字段写传播的 `__age_s_*` 状态槽）`source.age.query-token-stale` 具名拒；`q.t` 立即读合法，查询前字段写须在查询后以达成位移重写刷新（每字段状态函数尾读自身槽）。②每函数静态准入——隐藏函数内 gfx 调用点合计≤32（三分支复制计费链=93>32 `lower.age-gfx-credits` 具名拒零效果），故 Tick 保单份计费链、轴归一化经 `vx`/`vy` 模型字段承载。作者面不动 32/8/16/1。
- **门禁**：`plan058_t04_av_chain` **5/5**（含新增 `g2_branch_and_g1_layer_facts_are_source_bound`：typed layer+cpu 孪生/overlap 起步 then 分支 -2/wall_b 非地面/变异重入/预算 32；逐例 61/3622/52/141/62ms）；`plan058_t04_{arms,oracle,provider,pin}` 6/4/5/2；`node_topdown2d_flow` 5/5；`s2_age_routing` 21/21；`verify_plan058.py --controls` 6/6。旧021/022 消费者（43s/50s）输入面与驱动旧档路径在本轮零改动，沿用 T-04 AV 收据同 pin 前身证据（测试纪律：无关行为复用已绿证据）。
- **边界**：AC/A2R/真实 GPU 连续帧链/`use` 实例仍待 W2/T-05（渲染注入形保持预证边界）；`render2d.frame` str 仍字面透传（G4 资源名语义待资源身份）；TS/WEB 复放车道对 `handler_stmts` Let/If 的具名拒归 980 owner。

**T-05 AV 阶段收尾结果（2026-11-18实测；AC/A2R 阶段门未开，见 D-07）**：
- **GPU 同宿主连续帧链**：`Play::call_script`（窄访问器，VM 面按名调用脚本函数——`vm` 私有字段的唯一新增公开口；play.rs 最近触碰为本包 T-03，无在途其他 owner）＋渲染脚本 `ag_biz_apply(x,y)`（裸语句 `ag_set_local(2,0/1,…)` 直写玩家对象局部变换，**只应用给定数、零业务**）。单渲染宿主连续呈现 AV 帧 31..=36：每帧"AV Tick→apply→恰一步→capture"，逐帧精灵质心=当帧 AV 行相机投影（Δ≤1px），逐帧位移匹配行差（6.00/6.50/6.50/3.00/0.00px；接触步 3.00px>1px 可判别、停步 0.00px≤0.5px 不穿墙），`frame.tick` 累计恰一步/帧，每 AV Tick 恰 32 credits，ground 174752/174716px。证据 `t05-gpu-continuous.json`（`e65f23a4…`）+6 PNG（f31 `206869ec…`）。
- **use 实例目标（V-S03 结构形）**：fixture 两目标升级 `Target` 定义 node（`model hits int`/`msg { Touch }`/`on { .Touch -> { .hits += 1 } }`）＋Player 场景两实例行 `Target target_a/b { position, size }`（ordinals 4/5、几何与旧平面形全同——40 Tick oracle/GPU 全链零漂移）。实测语言事实：①场景 node 序号**每模块局部**（跨模块时各模块 node 0 撞号、ForNode 定向失效→双 node 须同模块；Target=0/Player=1）；②实例行作者属性合法，`cpu` 带逐实例解码 position/size（宿主行直读）；③状态函数拼写=模块局部 node 0 用兼容形 `__age_state_<inst>_<field>`、node>0 用 `__age_state_n<node>_<inst>_<field>`；④`ForNode` 定向 2:1 Touch→`__age_state_0_hits`=2/`__age_state_1_hits`=1 独立（探针+主 fixture 整合两用例）；⑤作者面"业务→实例事件派发"（玩家触碰路由到命中目标）无 gfx 端口（17 op 无 dispatch）——具名缺口，归 981 IF（同 G1..G5 模式），本包不建私有旁路。
- **AC/A2R 阶段门核对（D-07 证据）**：PLAN-1068（AC 机器执行主责）`drafting` 0/10；981 的 `IF-AGE-GFX-AC`（LANGUAGE/FUSION AC 部分＋1068/T-04）与 B-RUST*/A2R 域图形端口均未交付（981 §8 各行未勾，"图形新增A2R实现/验证在UI-W2-AC正式后"）；UI-W2-AC/UI-W3-A2R 波次无落地收据。按 §5 阶段合同：AC 格不得以 AV/A2R 代，A2R 须本包 AC 正式来源在前——两项保持 pending，T-05 原全量义务不减。
- **门禁（最终修订 `350af5b8` 全绿）**：`plan058_t04_av_chain` 5/5（5.13s；实例形 fixture 下 40 Tick 逐 Tick oracle/预算/分支/GPU 同 Tick 全保持）；`plan058_t05_gpu_continuous` 1/1（1.72s）；`plan058_t05_use_instances` 3/3（0.07s；逐例 4/6/52ms）；`plan058_t04_{arms,oracle,provider,pin}` 6/4/5/2；`node_topdown2d_flow` 5/5（4.47s）；`s2_age_routing` 21/21；`verify_plan058.py --controls` 6/6。新用例全部远低于 10s 门。

**T-05 动态世代接口消费结果（2026-11-18实测，pin `0a167f611`）**：
- **pin/事实面**：overlay 刷到 auto-lang master `0a167f611`（981 宿主动态 token/世代批 merge `2cb5df262`；681 base+**1435** overlay，+1 文件）；`AgeGfxCall { tick, callback, query_epoch, credits_used, symbol }` 经 `dispatch_with_ctx` 逐派发到达（默认转发不破既有 provider——`SharedHost` 包装显式转发）。
- **判罚点落 UI058（981 只交事实）**：`NeutralGfxHost` 新增 `CallFact` 日志＋查询 token 簿记＋`last_ctx`；**fault-13 判定** `resolve_query_token`——世代边界（tick/callback）取**最近任意派发**的 ctx、epoch 取簿记（同 callback 第二次查询把簿记推进 E+1 即杀旧 token）；载荷双面命名（held {tick,callback,epoch,sweep} vs "moved to tick/callback N" / "epoch to N"）。**fault-33 判定**：dead 句柄写（frame/despawn）具名拒 `age-gfx.stale-render-handle` 携会话事实，never-issued 保持 `age-gfx.bad-token`（拆分）。
- **验证（`plan058_t05_gfx_epochs` 4/4，逐例 32/36/59/26ms）**：①连续链骑乘——每 Tick 首 op epoch0、sweep→1（credits 21）、segment→2（30）、标量 op 原样带当前值、frame 收 32；`frame.tick`/callback/epoch 与会话访问器全同；②fault-13 载荷——同 callback 二查询杀旧 token（"epoch to 2"）、**纯 callback 边界**（sweep_only 16+peek 1=17 预算合法，"moved to callback 3"）、跨 Tick（"moved to tick 3"）；③双实例骑乘——Touch 回调零 gfx 派发（epoch 0）、Tick 回调 1→2、2:1 状态保持（last_node=4）；④fault-33 真实车道——`frame_after_despawn`（spawn→despawn→frame 死句柄）具名拒携事实、零发布。
- **门禁（`12cb146b`）**：av_chain 5/arms 6/oracle 4/provider 5/pin 2/gpu_continuous 1/use_instances 3/topdown2d_flow 5/s2_age_routing 21/controls 6 全绿（fixture 增 `frame_after_despawn` 不触 40 Tick/预算路径）。
- **边界**：判罚面为本宿主车道语义（PLAN-938 host authority）；981 侧 epoch=纯事实不动 op 语义；TS/WEB 车道不消费 ctx。

**T-05 缺口⑤答复：跨模块实例身份已修 + typed 队列可直接复用（2026-11-18，981 `3047b3481`）**：
- **跨模块撞号已关闭（981 侧三处统一为包级组件声明序）**：①场景 `Send/SendArgs` 的声明级核对从"逐模块用本模块 nodes"移到**绑定器包级聚合点**（越界消息报包内组件数）；②发射的 `__age_state_n<k>_…` 由 `node_base` 线程化为**包级序号**（此前是模块局部序号，与读回/AV 的包级期望错位——这正是"两 node 必须同模块"的根因）；③AV 观察改用 `symbol_of_in(module, name)`（原按源名表在多模块下同名互覆盖）。**永久反例**（`plan981_cross_module_nodes` 4 例）：两真实模块 `ForNode{0}×2+{1}×1` ⇒ 2:1；模块输入序调换 ⇒ 包级序随动；越界 `ForNode{2}` ⇒ 报"package declares 2 component(s)"；未包裹 `Send` 仍重放全部组件。**UI058 可把 `Target`/`Player` 拆回两个真模块**（不再需要塞同模块）。
- **typed 队列复用：不需要新接口**。会话层已有 `send_to_component(node, instance, msg)` / `send_args_to_component(node, instance, msg, args)`——载荷经唯一 verify 按位绑定、`schedule` **原子重编**（被拒请求不污染既有 schedule）、stopped/越界/arity/type 具名拒；组件参数**现即包级序号**。"命中目标 → 向该实例派发"在**宿主侧**用这两个方法即可（判罚/路由=宿主 authority）。
- **唯一确实缺失项（本批不新增，明确取舍）**：**作者代码内自派发**（17 个 gfx op 无 dispatch 端口）。做成作者可见端口会绕过 host authority（判罚与算法在宿主），且 W1 不需它——故保持具名缺口，**留待真实消费者提出**时按新服务面走版本化流程。
- **消费提示**：本答复绑定 auto-lang `3047b3481`；下次刷 pin 时一并生效（本批无 ABI 变更，仅场景身份与符号解析，overlay 内容会随文件哈希变化）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-topdown-composition.md | 有限query/sprite各档→显式俯视组合支持/一次sweep/同Tick表现和原预算边界；旧档保持 | 正常Auto作者与分仓可验证交付 | AC-01..06/10/11 |

为AG029追加的公共接面仅是可见地面/阻挡查询分离：复用单张背景sprite或等价非碰撞渲染层，角色可遍历地面且仍受原两障碍阻挡，查询不把地面误选成最近目标。T-02比较至多原两种策略并冻结layer/mask/owner与同Tick发布；T-03实现及建V-S08，T-04/05验双端/费用。不得扩成tilemap批绘、种田业务或新文件系统；AG029存档归973，库存/时间归内容。AG023可沿用不含背景的原最低教学场景，其原验收不减。

## 6. 测试设计

### V-PHASE-GFX（新增阶段控制，原V行全部保留）

T-03..T-06及同号阶段复核消费原输入/expected：①原AV必需格齐、AC/A2R尚pending时只能交AV phase；②同源AC禁AV guest仍得到独立预期，假机器执行必须红；③A2R保真实生成/rustc/无VM运行/冷包；④任一原required格缺失、错源/ABI/资源/候选或只贴截图，整包拒绝；⑤删阶段剩余端或将pending充NA，AG031接收必须红。对应AC-PHASE-GFX；不在日常AV门运行全套A2R或全量AG。

以下均为未来work验证合同，本轮未执行。实施者先自检；同一权威树由独立上下文final复审，不能用同实现会话重跑充独立。只跑真实受影响case/必要消费门，GPU/窗口焦点/音频重门按机器单实例；不为merge重复全量AGE，不改根语言时不跑根全仓。新路径/入口由任务建立后才可执行。

2026-11-19 r8：下列 V 行的 **AC 格/A2R 格**由 PLAN-103 逐行承接（行 ID/输入/expected/独立 oracle/包络不变，含 V-S07 的 A2R 腿与 V-S03 的作者面触碰路由完整形）；本计划关闭各行的 **AV 格**（V-S07 的 AV 运行腿已交 `350af5b8`/`12cb146b`，V-S03 的 scenario 侧 2:1 已交）。

| case | AC | owner_stage | 输入/入口与可区分预期 | 覆盖边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | T-04/05正常use模块全链：轴(1,0)/(1,1)同速、零/反向输入、改Auto速度真实变，比较f32轨迹/GPU；T-02格先用静态f32行值＋宿主行写（不依赖f32业务式） | 不能由平台重力或raw接口代作者支持；f32业务式待981 AV切片 |
| V-S02 | AC-02/03 | work | 一次sweep首次停墙/墙角/起始overlap；scene位置与精灵像素同Tick，最近和同距target顺序独立oracle | 不要求自动slide，不改旧query容差 |
| V-S03 | AC-03/04 | work | 两同定义目标A两次/B一次各状态2/1、超距不触发；两次sweep(16+16)=32 credits合法，下一次收费调用（累计>32）即具名拒绝、零效果、本Tick后续收费调用保持拒绝 | 四/八向与交互不隔帧；各单位独立；预算口径见§5 |
| V-S04 | AC-04/05 | work | Pause/Step/20轮Run，旧句柄/坏模块/资产拒绝→精确修复→相同公开入口下次真Run | owner/上传/残留对象有收支证据 |
| V-S05 | AC-05/06 | work | verify_plan058/storage冷A→B无producer根/target/网络，改源旧Ready失效，旧021/022/032身份不变 | 不改M2消费者锁 |
| V-S06 | AC-01/06 | work | 055已交结论/AG022需求、原M1必需门与981正式来源→支持/限制/SD/AG023配方逐项对照 | 未证不冒go，不等待合并M2最终签章；研究完成不替本组合先证 |
| V-S07 | AC-10 | work | T-03建永久用例，T-04/05经981公开入口使同一Node源/资源AV运行与A2R真实编译运行；四八向/碰撞/最近目标/2:1事件/GPU位置各对独立oracle，改源新结果、错签名/坏资源拒绝修复后真Run | 不手造HIR/手写Rust业务、不用旧Parser/AutoVM或旧生成器；T-04前冻结实际命令/ABI/provider |
| V-S08 | AC-11 | work | T-03建含可见背景+原五对象正例；T-04/05同源AV/A2R核遍历地面、两障碍停墙、只选Target、GPU位置同Tick，显式把背景误纳碰撞的实际变异须复现t=0并使独立轨迹oracle红，恢复正确分层后绿；原32credits/模型/scene包络不抬 | 复用055满铺反例，不把移除全部地面或障碍当修复；非tilemap/存档 |

## 7. 验收标准

- [ ] **AC-PHASE-GFX / work**：原每一V行的 **AV 格**按 §5 r8 适用性闭合，独立 oracle/包/资源与失败义务在 AV 面守恒；**AC/A2R 格与多端"整包拒绝"条款由 PLAN-103 原封承接**（§5 r8 转移清单）；由 V-PHASE-GFX 的 AV 格、T-03..06 与 T-07 final 核验。

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 研究/需求、原M1必需门与981正式接口真实闭合，完整先证和版本化支持表可审 | V-S01/06 |
| AC-02 | work | 归一无重力控制/一次sweep停墙/数值与普通Node真执行成立 | V-S01/02 |
| AC-03 | work | query位置/精灵同Tick一致，最近目标和双实例事件隔离正确 | V-S02/03 |
| AC-04 | work | 原预算峰值/owner/Run/超界和错误恢复真实成立；超界=下一次扣费后累计>32 credits即具名拒绝、零效果、本Tick后续收费调用保持拒绝 | V-S03/04 |
| AC-05 | work | 冷消费/坏源修复下一Run及旧profile/锁兼容齐 | V-S04/05 |
| AC-06 | work | SD/独立oracle与精确AG023 **AV 入口**齐（双后端配方归 PLAN-103），无完整种田能力虚称 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/实际来源与代表性反例pass，finding返work同员有界复核 | T-07 |
| AC-08 | merge | 冻结触面门/来源核后实际落主、canonical及同号归档/导航，结论和实际支持域一致 | T-08/09 |
| AC-09 | merge | 全实际输出对账/必要源命令结果摘要折Plan/fresh guard清自有树与scope构建物，具名hold登记 | T-10 |
| AC-10 | **PLAN-103** | 同一Node源码经981在AV和A2R真实编译运行、行为/绑定/资源及独立oracle正确，AG023两入口可消费；AV 面已交（`350af5b8`/`12cb146b`），A2R 与触碰路由 IF 归后继 | V-S07；PLAN-103 继承表 |
| AC-11 | work | 可见地面与阻挡/目标查询分离，原五对象/同Tick/费用和双端成立，供AG029公共消费 | V-S08、T-02..06 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续另行启动；T-01不等研究 | 只读差距/源版本/最低样本和oracle准备，核owner/唯一树scope | AC-01/AC-06 | [x] 2026-11-18：055 spec c185f8f2/归档10-10、AG022归档10-10、981 seam ddbda0228+ed8b16e01 32/8/16/1、f32 在途；样本6行/32credits（t01-entry.md） |
| T-02 | work | T-01；055/AG022已交结论；981 T-02联冻已回签 | ≤2机制中立服务/GPU最低负载先证，冻结profile/typed端口/预算映射/数值/共同发布；先证接口=已有静态f32行值＋宿主行写（不依赖f32业务式），真实Auto移动积分等981 f32 AV切片；不冒Node整链go，失败replan | AC-01/AC-02/AC-03/AC-04/AC-10/AC-11 | [x] 2026-11-18：probe c5086f50 5/5（queries-2d/sprites-2d；宿主行写/f32行值）；机制=碰撞代理与外观分离四要件；GPU 210/400px同Tick；32合法/33 fault10/40>32零效果sticky/40→41/按Tick重置；单assembly与use实例留T-03 |
| T-03 | work | T-02机制可行；981稳定合同供准备 | 中立图形服务/能力注册/可见非阻挡地面/Node领域声明与永久用例，不扩旧生成器；与981不相交写面并行 ；执行以§5当前端阶段合同为准，原全量义务不减 | AC-01/AC-02/AC-03/AC-05/AC-10/AC-11 | [x] 2026-11-18：新profile `autographics.topdown2d/1`(30/121)+scene/9 layer+render-only行表/感知segment/sweep(fault 60)；旧22 profile/10 unit/价目零漂移；`node_topdown2d_flow` 5/5、controls 6/6、旧消费者11/11+4/4；E1代理被65536对抗反例否决并删；commit 7165dbad |
| T-04 | work | T-03；981所需A切片正式来源及AG026已闭M1收据；AV 整链/G1/G2 正式作者形/epoch 消费已交 `b071634c`/`f6418120`/`7a1503b7`/`ba85f33f`/`0e67b1cd`/`12cb146b`（pin `0a167f611`）；r8 后本任务=**AV 格**：剩 OS/真实窗口输入通道与 V-S04 显式闭环用例；AC/A2R 格归 PLAN-103 | 正常Node同源 AV 整链（四八向/查询/呈现同步/实例/owner/故障及输入门的 AV 可证面）；执行以§5 r8 块为准，AV 原验收不减 | AC-02/AC-03/AC-04/AC-05/AC-11 | [ ] |
| T-05 | work | T-04 AV 格；AV 收尾已交 `350af5b8`（GPU 同宿主连续帧链+use 实例+2:1 隔离）＋`12cb146b`（epoch 消费）；r8 后本任务=**AV 格**：剩 V-S05 冷 A→B AV 腿与 V-S01..08 的 AV 格成套收口；AC/A2R 格归 PLAN-103 | V-S01..08 的 AV 格（预算合法/超界、独立数值/GPU oracle、旧档触面回归）；执行以§5 r8 块为准，AV 原验收不减 | AC-02/AC-03/AC-04/AC-05/AC-06/AC-11 | [ ] |
| T-06 | work | T-05 AV 格 | 冷A→B/修复下一Run、SD 与 AG023 **AV 入口**准确版本/命令/配方；**双后端配方归 PLAN-103**；执行以§5 r8 块为准，AV 原验收不减 | AC-05/AC-06/AC-11 | [ ] |
| T-07 | review | T-06 | 同树独立final，真实逐AC/SD/源和反例核验；finding同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结受影响门/准确身份后落主，不扩大无关全套门 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/导航与真实支持/未证边界，非产品研究不冒功能交付 | AC-08 | [ ] |
| T-10 | merge | T-09 | 全路径对账/摘要折Plan/fresh guard清自有树与scope、hold登记 | AC-09 | [ ] |

## 9. 复审记录

### 2026-11-19 stage: new（**PLAN-058 r8 范围有界修订**：AC/A2R 转移 PLAN-103，收口 AV 面）| PLAN-058 r8 | outcome: pass（修订合同可审阅；本计划保持 executing，T-04..06 按 r8 AV 格继续）

**授权**：用户 2026-11-19 裁定——把 AC/A2R 内容挪到新计划、UI058 完成并合并；新计划等 1068 完成后再开始。
**变更**：①§5 新增 r8 块（转移清单五项：V 行 AC/A2R 格、AC-10、V-PHASE-GFX 多端条款、T-06 双后端配方、触碰路由缺口；本计划保留义务清单）；②T-04/T-05/T-06 重定义 AV 格并保留任务 ID 与全部已完成证据；③§6 V 表加承接注（行内容不改写）；④§7 AC-PHASE-GFX/AC-06/AC-10 改写（AC-10 owner=PLAN-103，不标 NA）；⑤§1/§0/§10 对应行；⑥新建 **PLAN-103** `103-topdown2d-ac-a2r-wave-consumption.md`（drafting）承接全部转移项。
**不减承诺**：原 V 行输入/expected/独立 oracle/包络逐行原封；最低样本/预算/旧档零漂移/SD-01 的 AV 面事实不变。
**依赖变化**：本计划 frontmatter depends_on 不变（055/AG022/AG021/AG026/981 均已交付，为 AV 面基础）；1068/W2/W3 波次依赖移入 PLAN-103。
**next**：work（T-04/T-05/T-06 的 AV 剩余格：OS 真实输入、V-S04 显式闭环、V-S05 冷 A→B AV 腿、AV 配方），随后 T-07 独立 final（r8 域）→ T-08..10 合并；PLAN-103 等 1068/W2/W3。

### 2026-11-18 stage: work（**981 动态世代接口消费**：AgeGfxCall 接入＋fault-13/33 判罚载荷由 UI058 验证）| PLAN-058 r7 | outcome: pass（消费面完成；T-04/T-05 未勾）

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | code commit `12cb146b`（6 文件 +665/−28，含 overlay.json）| 主检出只改Plan簿记。
**pin**：overlay 刷到 auto-lang master `0a167f611`（981 宿主动态 token/世代+A2C 宿主面批 `82aecc9ad`/merge `2cb5df262`；`refresh_overlay`→`apply_overlay` 1435→`verify_overlay` PASS 681+1435→`prepare_sdk --verify-only` PASS）。
**任务**：消费 981 `IF-981-GFX-HOST-PORT` 顺位第三批的会话事实面（981 §9「宿主动态 token/世代」收据；判罚点归 UI058/PLAN-938）。
**交付**：①`NeutralGfxHost::dispatch_with_ctx`（CallFact 日志/查询簿记/last_ctx；op 语义零改动）；②fault-13 判定 `resolve_query_token`＋fault-33 判定 `stale-render-handle`（dead 句柄写拒，与 never-issued `bad-token` 拆分）；③fixture `frame_after_despawn`（真实车道 fault-33 驱动）；④`SharedHost` 包装显式转发（默认转发会绕过 ctx）；⑤`plan058_t05_gfx_epochs` 4/4。
**验证数字**：连续链骑乘——每 Tick 首派发 epoch 0、sweep epoch1/credits21、segment epoch2/credits30、标量 op 原样带值、frame 收 32；`query_epoch()/callback_seq()` 与 ctx 全同。fault-13——同 callback "epoch to 2"、纯 callback 边界（sweep_only16+peek1=17，"moved to callback 3"）、跨 Tick（"moved to tick 3"）。双实例——Touch 回调零派发/epoch0、Tick 回调 1→2、`last_node=4`、2:1 保持。fault-33——`age-gfx.stale-render-handle` 携 "old generation...tick N callback M"、frames=0 零发布。
**命令/结果**：`cargo test --test plan058_t05_gfx_epochs` 4/4（0.16s）；全门禁 av_chain5/arms6/oracle4/provider5/pin2/gpu_continuous1/use_instances3/topdown2d_flow5/s2_routing21/controls6 全绿（fixture 增 fn 不触既有路径）。
**check→finding 映射**：①簿记只在查询时更新→世代边界改取最近任意派发 ctx（peek 后仍能判 callback 前移）；②peek 前 begin_tick 误加→跨 Tick 提前（预算也禁 32+1）→账重排：sweep_only16+peek1=17 演示纯 callback 边界；③会话世代顺延（前段耗 tick1/c1）→期望值顺延；④`run_def` 无参→fault-33 驱动并为单 unit fn。
**边界**：判罚语义=本宿主车道（PLAN-938）；epoch=纯事实（981 不判）；A2C 宿主面（同批 981 交付）归 AC 车道 1068/T-04 消费，本包不涉。
**next**：AV 侧对 981 事实面消费完成；AC/A2R 仍按 D-07 等波次门。

### 2026-11-18 stage: work（**T-05 AV 阶段收尾**：GPU 同宿主连续帧链 + use 实例目标；AC/A2R 阶段门精确阻塞）| PLAN-058 r7 | outcome: **pass（AV 阶段子面；T-04/T-05 未勾，AC/A2R blocked per D-07）**

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | code commit `350af5b8`（6 文件：fixture/play.rs/4 测试）| 主检出只改Plan簿记。
**任务**：T-05（AV 阶段收尾——§5 AV 行"真实GPU…及独立oracle齐"的连续帧部分＋V-S03 use 实例切片）。
**前置核对（D-07）**：AC 随后/A2R 最后的阶段门未开——PLAN-1068 `drafting` 0/10（AC 机器执行主责）、981 `IF-AGE-GFX-AC`/B-RUST* A2R 域图形端口未交付、UI-W2-AC/UI-W3-A2R 无落地收据；按合同不以 AV/A2R 代 AC、A2R 后于本包 AC。继续不受阻工作。
**交付**：①`Play::call_script`（窄访问器，VM 面按名调用；呈现数据通道，play.rs 最近触碰为本包 T-03、无在途他 owner）；②`tests/plan058_t05_gpu_continuous.rs`：单 AV 宿主 36 Tick＋**单渲染宿主**连续帧 31..=36，`ag_biz_apply`（`ag_set_local` 裸语句）逐帧应用 AV 行位——渲染路径零业务；③fixture 两目标升级 `Target` 定义 node+两实例行（V-S03 结构形，几何/序数不变）；④`tests/plan058_t05_use_instances.rs`：实例事实+ForNode 定向 2:1（探针）+主 fixture 整合。
**命令/结果（最终修订全绿）**：av_chain 5/5（5.13s）；t05_gpu_continuous 1/1（1.72s）；t05_use_instances 3/3（0.07s）；arms 6/oracle 4/provider 5/pin 2；node_topdown2d_flow 5/5；s2_age_routing 21/21；controls 6/6。
**数字**：连续帧逐帧质心 vs 当帧 AV 行投影 Δ≤1px；逐帧位移匹配行差 6.00/6.50/6.50/3.00/0.00px（预期 6.33/6.33/6.33/3.17/0.00，像素栅格量化）；`frame.tick` 恰一步/帧；每 Tick 32 credits；ground 174752px。2:1：`__age_state_0_hits`=2、`__age_state_1_hits`=1。新用例计时：GPU 连续帧 1720ms/52ms/6ms/4ms——全部 ≤10s 门。
**check→finding 映射**：①`ag_set_local` 返回 unit→`*1.0` TypeError→裸语句；②连续宿主 `frame.tick` 累计→按帧序断言恰一步；③`source.age.scene-element`（跨模块实例行）→同包模块须显式 `use <module>: <Node>`；④`ForNode{node:0}` 双模块撞号（场景 node 序号每模块局部）→双 node 同模块（Target=0/Player=1）；⑤`source.type-mismatch`（探针业务 f32→i32）→探针业务改 i32 字面量。均以最小改法收敛，语言事实入 §5。
**证据**：`t05-gpu-continuous.json`（`e65f23a4…`）+f31..f36 PNG（f31 `206869ec…`）；`Play::call_script` 实现与门禁见提交 `350af5b8`。
**边界**：AC/A2R 格 pending（D-07 精确阻塞与解除动作）；作者面"业务→实例事件派发"无 gfx 端口=具名缺口归 981 IF；渲染桥仍为 AV→呈现单向数据应用（OS 输入/交互反转归后续）。
**next**：AC/A2R 等 D-07 解除（1068/T-04＋981 AC 域端口→UI-W2-AC→A2R）；AV 侧 T-04/T-05 子面已齐，整任务原全量义务不减。

### 2026-11-18 stage: work（**T-04 G1/G2 正式作者形**：真实 `on Life.Tick` + 源侧 `layer: render-only`，同宿主连续帧复验）| PLAN-058 r7 | outcome: **pass（AV phase 内；T-04 未勾，AC/A2R/GPU 全量/`use` 待 W2/T-05）**

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | merge `cbe350b4`（并 master：G5 pin `65a6b2cf`@auto-lang `6e456794d` + 980/089 落主）| code commit `0e67b1cd`（3 文件 +279/−86）| 主检出只改Plan簿记。
**依赖**：SDK overlay 自 pin 树 `D:/autostack/plan-artifacts/plan-058/auto-ui/pin-6e456794d/`（git archive；auto-lang 主检出已前移 982 合并 `600d31527`，活树哈希不符按 fail-closed 拒，属预期）→ `apply_overlay` 1434 文件 PASS → `verify_overlay` PASS（681+1434）→ `prepare_sdk --verify-only` PASS。
**任务**：T-04（G1/G2 接线；981 侧前置已解除——G5 收据 `f9ade844`）。
**交付**：①fixture 删 `tick_step` 宿主态形，`on Life.Tick` 改真实 hook-local `let`（14 个，含 `let q Sweep2D`）+ 归一化/记录 `if/else`；②ground 行 `layer: render-only`（G1），`rows_from_scene` 改读 `AgeSceneRow.layer` typed 事实；③runner 经 `compile_age_package_with_scenario`（1 实例+Tick）驱动 `__age_state_0_last_node`；④新增 `g2_branch_and_g1_layer_facts_are_source_bound`；⑤pin 测试改真实形事实（layer/let-if/tick_step 退役）。
**命令/结果**：`cargo test --locked --offline --test plan058_t04_av_chain -- --test-threads=1` **5/5**（suite 5.22s）；`plan058_t04_{arms,oracle,provider,pin}` 6/4/5/2；`node_topdown2d_flow` 5/5（3.74s）；`s2_age_routing` 21/21（0.14s）；`verify_plan058.py --controls` 6/6。
**数字**：40 Tick 每 Tick 恰 32 credits、无 fault；逐 Tick 位/sweep(node,t,start_overlap)/segment 与 oracle 一致；第35首触停位 8.6/t=0.5000353；36..40 t=0 且 start_overlap=true → `last_node=-2`；GPU 同 Tick：tick34 8.533329→质心 425.00/Δ≤1px、tick35 8.6→428.00、34→35 3.00px、35→36 0.00px、ground 174752px——证据 `t04-gpu-tick34.png`（`d191558e…`）/`tick35+36.png`（`23af8c82…`）/`t04-gpu-same-tick.json`（`e7bf2a64…`）与前一收据**逐字节同**。新测试逐例计时：budget 61ms / GPU 3622ms（复用例）/ input 52ms / chain 141ms / g1g2 62ms（≤10s 门全过）。
**check→finding 映射（本轮三轮红→绿的判别点）**：①`source.age.query-token-stale`（`__age_hl_*_q`）→ 021 纪律：回调内第二次 `query2d.*` 使此前绑定名失效，`if q.start_overlap` 后置非法 → 改 `q.t` 立即读；②同名（`__age_s_0_blocked`）→ token 经字段写传播进状态槽，查询前写后读仍拒；③同名（`__age_s_0_vx`）→ **每字段状态函数尾读自身槽**：任何查询前字段写都使其字段读取过时 → 查询后以达成位移重写 `vx`/`vy` 刷新；④`lower.age-gfx-credits`（调用点合计 33/93>32）→ 每函数静态准入拒计费链复制 → 单份计费链+`vx`/`vy` 承载轴归一化。最终形为同时满足 021 走查、静态准入、G2 语句面与 32-credit 峰值的唯一作者形。
**边界**：AC/A2R/真实 GPU 连续帧链/`use` 实例待 W2/T-05；渲染仍注入形（预证边界）；`render2d.frame` str 字面透传；TS/WEB 对 `handler_stmts` Let/If 具名拒归 980。
**next**：work T-05（W2-AC/981 AC 域、A2R 真编、GPU 连续帧与 `use` 实例），按 §5 阶段合同；本包 AV 侧对 981 的缺口清零。

### 2026-11-18 stage: work（T-04 AV/GPU 同 Tick 子面预证）| PLAN-058 r7 | outcome: pass（子面；T-04 未勾）

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | code commit `ba85f33f`（1 文件+504）| 主检出只改Plan簿记。
**任务**：T-04（AV/GPU 子面）。
**证据**：`t04-gpu-tick34.png`（`d191558e…`）、`t04-gpu-tick35.png`/`tick36.png`（`23af8c82…`）、`t04-gpu-same-tick.json`（`e7bf2a64…`）、`t04-chain.md` §4.5（`88b085d6…`）。
**命令/结果**：`cargo test --test plan058_t04_av_chain` **4/4**（GPU 例 4420ms，`shift34_35=3.00px`、`shift35_36=0.00px`）；`node_topdown2d_flow` 5/5；`plan058_t04_pin` 2/2；`verify_plan058.py --controls` 6/6。
**数字**：tick34 row 8.533329→投影425.33/质心425.00/Δ(-0.33,-0.56)；tick35 row 8.600000→428.50/428.00/Δ(-0.50,-0.56)；tick36 同35；每 Tick 32 credits；ground 174752px；wall 质心≤2px；mask 1014px。
**同 Tick 判别**：逐 Tick 恰一步后立即 capture（`frame.tick`=1）；34→35 位移 3.00px（若晚一帧为0，>1px 排除）；35→36 0.00px（停墙不穿）。
**边界**：AC/A2R 双端与真编、G1/G2、GPU 之外输入通道不在本子面；渲染源为 no-op probe；G4 资源名仅字面透传。
**next**：G1/G2 落地后以源侧声明+`on Life.Tick` 替换 `tick_step`/行名地面并复跑同一用例；AC 等 W2-AC/981 AC 域。

### 2026-11-18 stage: work（T-04 AV 整链：正常 Node 源经 981 host port 真跑）| PLAN-058 r7 | outcome: pass（AV phase；T-04 未勾，AC/A2R/GPU 待）

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | code commit `7a1503b7`（8 文件，含 `sdk/overlay/overlay.json`）| 主检出只改Plan簿记。
**pin**：overlay 刷新到 auto-lang master `64e8380f4`（含 record ABI `5ee4d23c0`、f32、seam）；`prepare_sdk.py --verify-only` PASS（681 base+1432 overlay/17 local packages）、`verify_overlay.py` PASS two-layer。预存红：`viewport3d_depth_probe.rs`/`squash_feasibility.rs` 缺 `Scene3dContent.hud_overlay`（PLAN-013 字段，测试未更新，与本 pin 无关，未修）。
**任务**：T-04（AV 阶段）。
**证据/命令**：`t04-chain.md`（`11c6e365…`）；`plan058_t04_av_chain` **3/3**（74/33/45ms）；`plan058_t04_{arms,oracle,provider,pin}` 6/4/5/2；`node_topdown2d_flow` 5/5；`s2_age_routing` 21/21；旧021 `node_queries_flow` 11/11；旧022 `node_sprites_flow` 4/4；`verify_plan058.py --controls` 6/6。
**数字**：40 Tick 每 Tick **32 credits**、无 fault；逐 Tick 位置/sweep(node=2,t≈0.5,start_overlap)/segment(node=4,t=0.25) 与 oracle 一致；首次接触**第35 Tick 停位8.6/t=0.5000353**；输入四变体与改源速度8→4（0.0666667）随动；第33 credit sticky、48 credit 拒、零效果、`begin_tick` 重置、`stop` 具名拒。
**边界**：G1（AGE `shape2d` 无 render-only 声明）与 G2（hook-local let/memoized）仍待981；AV 驱动用 `pub fn tick_step()` 宿主状态形态，`on Life.Tick` 原形保留；GPU 像素/AC/A2R/`use` 实例/OS 输入待 T-05/后续波；`render2d.frame` str 仅字面透传。
**next**：W2-AC/981 AC 域与 G1/G2 后补 AC/A2R 与 GPU（T-05），T-04 全量义务不减。

### 2026-11-18 stage: work（T-04 三臂：segment/sweep/image2d 对齐 record ABI）| PLAN-058 r7 | outcome: pass（三臂完成；T-04 仍未勾，待 G1/G2+pin）

**绑定**：worktree 同上 | code commit `f6418120b86b48e0149ed3d72e8bf31ee4252ad3`（5 文件+886/−7）| 主检出只改Plan簿记。
**ABI 来源**：981 `D-981-GFX-PORT-ABI` closed（auto-lang `5ee4d23c0`，receipt `381d7a364`，master `f59592174`）；一次宿主调用返回完整 `AgeGfxReturn::Record`，AV 按调用槽 record 复核；收费由 981 `AgeGfxSession` 负责（segment 8/sweep 16/image 1、`age-gfx.credits` sticky）。
**交付**：`src/authoring/gfx_host_arms.rs` 三臂＋`dispatch`（symbols `age.query2d.{segment,sweep}.v1`/`age.image2d.png.v1`）；`Image2D` 改为 ABI 四字段 `[sha256,len,width,height]`；测试 `tests/plan058_t04_arms.rs` 6 例。
**记录顺序（逐元素断言）**：Cast2D `[hit,node,t,point_x,point_y,normal_x,normal_y]`；Sweep2D `[hit,node,t,origin_x,origin_y,normal_x,normal_y,start_overlap]`；Image2D `[sha256 Str,byte_length i32,width i32,height i32]`。
**实测（root 复跑）**：arms 6/6（0.00s）、oracle 4/4、provider 5/5、controls 6/6。几何：segment mover1 (4,0)→(4,6)=`true,4,0.25,(4,1.5),(0,−1)`（裸 box）；sweep mover1@(8.533333,0) dx=0.13333334=`true,2,t≈0.5,origin(8.533333,0),(−1,0),false`（mover-half Minkowski，停位 8.6）；image2d 独立 sha2/png 复算一致（`95a08525…`，258B，96×32）。render-only 不入候选，改 Collision 即 055 node0/t0/overlap；render-only mover/越界/参数/路径/缺图/坏 PNG 均具名 fault。
**边界**：三臂不计费（归 981 session）；AV/A2R 实际执行、PNG 进工程/GPU 像素、`use` 实例未覆盖；G1（AGE 源侧 render-only 声明）仍未交。
**next**：pin 刷新后写 `impl AgeGfxHost` 薄适配（Value↔GfxValue、StrHandle、fault 去前缀），用 `AgeGfxSession` 跑 `on Life.Tick`，逐 Tick 对 oracle。

### 2026-11-18 stage: work（T-04 预备：正常 Node 源/独立 oracle/candidate provider）| PLAN-058 r7 | outcome: pass（备料完成；T-04 仍未勾，等 981 端口+pin）

**绑定**：worktree 同上 | code commit `b071634cfc7d74b2c641de817fc599c8826983ea`（6 文件+1234；T-04 未勾）| 主检出只改Plan簿记。
**交付**：正常 AGE Node 源 `fixtures/plan058/topdown-node/main.at`（1Player+2障碍+2同定义目标+可见非阻挡地面；f32 model/一次 sweep 停墙/segment 最近/同 Tick `render2d.spawn/frame`；无 `_ag_*`；预算 31/32）；`src/authoring/gfx_host_port.rs`（UI 侧 **candidate** provider 契约：17 op typed/Cast2D/Sweep2D/身份/32 credits/40>32 fault10 零效果 sticky/字段读0）；`tests/plan058_t04_{oracle,provider,pin}.rs`。
**实测（root 复跑）**：oracle 4/4（<0.01s：|v|·dt=0.133333340 对角同速；34 Tick 停位8.6、首次t=0.5000353；segment 最近ord4 t=0.25、同距小序；地面排除+055变异 node0/t0 可判别）；provider 5/5（<0.01s：合法 Tick 9调用31/32；sweep16+segment8+16=40>32 fault10、attempt40→41、下一Tick重置；render-only 行永不作目标）；pin canary 2/2（旧 pin 无 seam/f32，报 `source.module-let-inference`；pin 刷新即红强制 re-base）。
**AGE 侧缺口（已并入 981 `IF-981-GFX-HOST-PORT` 补充）**：G1 `shape2d` 无 render-only/碰撞层声明位；G2 Tick 体 assignment-only；G3 无 typed 行句柄；G4 `render2d.frame` 的 str 拼写未证；G5 pin 仍旧。
**待办（T-04 第1步）**：刷新 `.sdk-source` pin 到含 seam+f32 的 commit 并记录；端口到后以前门替换 G1/G2 临时形态，写薄 host-port 适配，逐 Tick 对 oracle（停位8.6/查询node/t/同Tick位置），不抬 32/8/16/1。

### 2026-11-18 stage: work（T-04 入口核对：接口阻塞，路由 981）| PLAN-058 r7 | outcome: **blocked（外部接口）** | 不勾 T-04

**核对通过**：981 f32 AV `be8480cae`+A2R `a4950f0b5`、receipt `77dc0fc26`（`D-981-AGE-F32-SURFACE` closed，auto-lang master）；`render2d.spawn` `0e6c03c3d`；IF-981-GFX-AV-SEAM `ddbda0228`+`ed8b16e01`（17 op/6族/7端口/32 credits）；AG026 M1 `9264fac`/签章`ea0fc59`。
**阻塞证据**：`auto-lang/experimental/ace/crates/lowering/src/hir_adapter.rs:5082` 对 15 个 HostAlgorithm + `HostIdentity` 在 AV 车道具名拒 `lower.age-gfx-lane`；981 `age_session_core` 与 UI AGE session 均无 gfx 宿主派发；T-03 中立 profile 是 driver 旧 API，不能直接充 AGE typed lane。故“正常 Node 源 AV 整链”缺执行口。
**决定（用户 2026-11-18）**：路线 A=回 981 owner 补 SESSION/typed 宿主端口；请求 `IF-981-GFX-HOST-PORT` 已并入 981 §9（含派发/端口/预算/身份/永久用例/交接要求）。UI058 不先建私有 runner/旧生成器旁路。
**状态**：T-04 保持`[ ]`（入口阻塞）；T-03 交付不变。next：等 981 `IF-981-GFX-HOST-PORT` 正式来源后开 T-04。

### 2026-11-18 stage: work（T-03 公共 render-only 行层）| PLAN-058 r7 | outcome: pass（T-03 完成，3/10；T-04 可开；981 f32 AV/A2R 已落主）

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | base `b77998da` | code commit `7165dbad667972fc3fc27cbaaa65f81d4d4bd63b` | 主检出只改Plan簿记。
**任务**：T-03。
**交付**：见§5「T-03 冻结结果」；新增`tests/node_topdown2d_flow.rs`、`tools/verify_plan058.py --controls`、手冻夹具`fixtures/plan058/topdown-ground/`（scene `56bdcc71…`/2823B、world `5ac49d91…`/1304B）；`public_audio_compat`仅把新profile从历史基线集合具名排除。
**命令/结果**：`cargo test --locked --offline --test node_topdown2d_flow -- --test-threads=1`→5/5（逐例0.503/1.480/1.495/0.945/0.001s，suite 4.47–5.36s；root复跑绿）；`python tools/verify_plan058.py --controls`→6/6 PASS（C1 identity/C2 9旧产品sha零漂移/C3 104旧场景0 layer/C4 RO非候选/C5 10旧unit+30/121/C6子进程3.68–4.49s）；旧消费者`node_queries_flow` 11/11（43.1s）、`node_sprites_flow` 4/4（49.6s）；`t03-vs08-before.png` `25b52c6d…`/`after` `0eebc963…`。预存红：`public_audio_compat` 3D六项（同源stash同红；3D unit `9c13cca4` vs 基线`7bfbac0d`），非本任务。
**依赖更新（981 f32）**：`D-981-AGE-F32-SURFACE`关闭——AV `be8480cae`、A2R `a4950f0b5`、receipt `77dc0fc26`，均已在auto-lang master；`render2d.spawn`成员位切片`0e6c03c3d`同交。状态观察/呈现另登记`D-981-F32-CONSUMER-WINDOW`（待回签；不阻T-04以`execute_component_def`取`Value::F32`先证）。
**边界**：981 Node作者链/AV-A2R整链/OS输入、`use`两实例包络、AC/A2R留T-04/05；新profile不含audio/HUD单元（/9不带其字段）。
**next**：work T-04（正常Node同源链，先AV），按§5阶段合同。

### 2026-11-18 stage: work（T-01→T-02 先证）| PLAN-058 r6 | outcome: pass（T-01/T-02 完成，2/10；T-03 可开）

**绑定**：worktree `D:/autostack/.wt/ui-058/auto-ui` | branch `codex/plan-058-dev` | base `b77998da` | code commit `c5086f50144f355346fd2ae2736d529d76e0e8ff`（只加 `tests/plan058_t02_probe.rs`）| 主检出只改Plan簿记。
**依赖版本**：055 spec `docs/specs/auto-graphics/topdown2d-feasibility.md`（sha256-16 `c185f8f2df252951`）＋归档10/10；AG022 archived 10/10（`966716d7b`）；981 IF-981-GFX-AV-SEAM `ddbda0228`+`ed8b16e01`，`GFX_CALLBACK_CREDIT_BUDGET=32`、segment8/sweep16/其余1/字段读0；`D-981-AGE-F32-SURFACE` 在途（f32注解/model字段未交，T-04前置）。
**任务**：T-01、T-02。
**证据**：`plan-artifacts/plan-058/auto-ui/t01-entry.md`（`80ae49ca…`）；`t02a-render-a.png` `33c5d019…`、`t02a-render-b.png` `ae41b5ad…`、`t02a-row-presentation.json`、`t02a-ground-separation.json`、`t02b-over-budget.json`；probe `tests/plan058_t02_probe.rs`。
**命令与结果**：`python tools/prepare_sdk.py --verify-only` → exit 0（681 base+1414 overlay/17 local packages；`--from-snapshot`/`apply_overlay`因主检出前移拒绝，按`overlay.json`逐文件物化回退，记录在t01-entry）；`cargo test --locked --offline --test plan058_t02_probe -- --test-threads=1 --nocapture` → **5 passed/0 failed**；逐例 0.33/0.54/3.05（GPU空闲复跑；并发构建时曾11.56s，属机器负载非用例回归）/0.40/1.07s；rustfmt与dead-code清理后复跑同绿。
**实测摘要**：sweep→obstacle_a t=0.046000004、停位8.58；segment→target_a t=0.25（target_b t=0.4583为真实第二候选）；地面正控不停走，变异（地面入候选）→t=0/start_overlap=1/位置4.0（055反例可判别）；GPU两Run行值4.0→px(210,190)、8.58→px(400,190)；两次sweep=32合法且连续3 Tick各32（按Tick重置）、第33次fault10、40>32零效果sticky（attempt 40→41）。
**边界（未覆盖）**：单assembly query+sprite组合（T-03）；两个同定义`use`实例未占8实例包络（T-03/T-04）；真实Auto移动积分`x=x+vx*dt`等981 f32 AV切片（T-04）；OS输入/AC/A2R。未改Spec/旧价目；主检出他人WIP未触碰。
**next**：work T-03（公共render-only行/层、能力注册、可见非阻挡地面、永久用例与V-S08），仍以§5当前端阶段合同为准。

### 2026-11-18 stage: handoff（T-02 接口联冻：UI058 侧回签）| PLAN-058 r5 | outcome: pass（接口回签；T-02 实施未跑、f32 业务式仍为981前置）

**绑定身份**：981 handoff（auto-lang master `a13a6e6a1`；IF-981-GFX-AV-SEAM `ddbda0228`+`ed8b16e01`；其记录auto-ui快照`f43ef59b`）↔本包r5（auto-ui `91d549ea`）。`f43ef59b..91d549ea`对`experimental/autographics-auto-driver`、`experimental/autographics-node2d-pilot`、`crates/auto-ui-tools`零diff，981服务面表仍适用。本轮只改规划：未建`ui-058`树、未跑Cargo/GPU/产品/窗口、未勾T-02及以后任务、未改Spec/价目/文法。

**对981四项联冻的答复**（对应981 §9「四项联冻」「3个回签请求」）：
1. 预算域=**每世界回调（每Tick）**，与981一致；统一口径见§5「预算口径」及§6 V-S03：下一次扣费后累计超过32 credits即具名拒绝、零效果、本Tick后续调用保持拒绝（两次sweep 16+16后下一次收费调用即拒）。
2. `render2d.spawn` **保021拼写**，接受981在AGE两面调用成员位接收`spawn`的文法重基线；T-02冻结期以符号`age.render2d.spawn.v1`为准；不新增别名、不改021/022价目。
3. getter轴参数按981冻分量字段（`point_x/point_y`、`normal_x/normal_y`、`origin_x/origin_y`、`start_overlap`），字段读0 credit；不做动态轴`point(i)`。
4. f32面（`D-981-AGE-F32-SURFACE`）：拼写`f32`与同型四则/比较由981交付；UI058 T-02先证接口改用已有静态f32行值＋宿主行写（§5），不以f32业务式为前提；真实Auto移动积分等该f32 AV切片，不用int近似；若T-02判定必须依赖f32业务式，回needs_replan并把f32列为981前置（不降原验收）。

**给981的回签文本（可直接并入981 §9该handoff行）**：

> UI058回签（2026-11-18，PLAN-058 r5，auto-ui `91d549ea`）：四项联冻接受——①预算域=每世界回调（Tick），统一口径「下一次扣费后累计超过 32 credits 即具名拒绝、零效果，且本 Tick 后续调用保持拒绝」（两次sweep 16+16后下一次收费调用即拒）；②`render2d.spawn`保拼写、待981文法接收，T-02冻结期用符号`age.render2d.spawn.v1`；③getter用分量字段、字段读0 credit；④f32拼写/运算面仍为981前置，T-02先证用已有静态f32行值＋宿主行写、不依赖f32业务式，真实Auto移动积分等`D-981-AGE-F32-SURFACE`的AV切片，不用int近似。UI058未建树/未跑Cargo/GPU，T-02及以后未勾。

**边界**：T-02的中立服务实现/v3/v4查询/行状态/帧写/PNG载入/输入喂值仍归UI058（PLAN-938 host authority），981未代跑或代签；T-04起正常Node整链仍须981 SOURCE/SESSION/RUST及所用LANGUAGE的A切片正式来源。981交付f32面与`spawn`文法前，相关调用点不得写入业务源（具名拒优先，不以变通绕过）。

next: work（T-01只读准备→T-02先证）；本轮未开树。

### 2026-11-17 三路线修订的规划预检（不是实施验收）

本轮仅文档：统一strategy/架构、四个引擎AV新合同与四个新版AutoForge合同；保本包原任务ID/进度/全部旧AC/V/SD，新增阶段义务见§5–7。静态核对8新Plan共74任务、9既有Plan共140原任务ID、阶段DAG 44节点/90依赖无环、新增本地链接均可达，16份取材源码/规范/已归档合同身份未改变；四仓文档diff格式检查通过。981并发实施会话的T-01勾选保留，不计本轮交付。下方历史记录原样保留。

只说明规划合同可审阅；未运行Cargo/GUI/模型/构建/产品测试，未建实施树、未resume暂停工作、未合入/归档/签章。所需正式接口/最低负载/真实oracle仍须未来work→独立review→merge实际验证；原已存在历史失效链接不作为当前入口，新增链接已单独核对。

2026-11-11 / stage:new / PLAN-058:r3 / outcome:pass（规划修订）；blocked（未授权实施，981所需A接口未交） / next:明确启动后T-01/02，中立服务先行，T-04等A正式来源而非981全包。AG026已11/11签章，旧M1未闭描述仅属历史。新增AC-11/V-S08与地面公共接面，原T/AC/V最低样本/预算保留，10任务/11AC/8验证行、0完成。不制作资源或实施。


2026-11-09 / stage:new / r2：合并M2二维线，中立服务先并行、T-04起作者整链等981及原M1门，新增AC-10/V-S07；10任务和原9AC/6V、最低样本/图形预算/兼容义务保留。981不反向等本包/AG023，无新增Plan。drafting、0/10；仅规划，实施handoff blocked（未授权实施、接口/原生门未齐），非产品/独立final PASS。

规划静态核验：979/981/UI058/AG023可达199份跨仓Plan，无缺失引用/依赖环；本包10任务、10AC、7验证行且原ID全保留、0完成，新增链接可解析。T-02中立机制可行不充作者go，T-04/05仍负完整Node/AV/A2R与原最小负载验收；无产品运行证据。

2026-11-05 / stage: new / auto-ui/PLAN-058:r1 / outcome: pass（合同可审阅，dependency-ready任务见T-01）；后续依赖和真实先证门未满足时不得推进。当前drafting0/10；仅规划、没有新树/材料scope/源码/探针/构建/GPU/资产/canonical/SDK变更或agent。具体可执行边界见§5/§8，未把上游草案或WIP当交付。

2026-11-05 / stage: new / handoff: pass（T-01只读准备可进）；blocked（T-02等055/AG022实际交付与M2必需门真实收口）。drafting/r1/0-10，全部任务未勾；本批仅规划，未授权或启动本批实施。

2026-11-05 / stage: new / planning_validation: pass（合同预检，不是独立final/实现认证）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD，依赖实际解析DAG无环、M3未成为M2前置；主/归档及注册树独占取号后核UI058/059、AG023唯一，next UI060/AG024。三仓当前白名单新增/修改本地链接均可解析（含UI008补录），空白检查通过；其中共享导航包含他路线WIP，仅选择本轮AGE行/章节暂存，不代其提交。状态表逐项34 UI＋23 AG＝57（49交付/8未完），49个所列delivery均为各自主HEAD祖先，UI039旧draft影子不重复计。初次表计数模式只匹配“UI编号]”而漏带中文标题的8行，修正为编号后允许标题后复核通过；不是实现红。固定e02ca9f8的pipeline/lower_components2d_runtime与本轮所记源码内容一致；历史canonical/归档及当前054/AG021合同未由本会话修改。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/六旧锁与许可一致。未构建/运行本批新probe/GPU/原生输入/容量/代码，未创建本批树/材料scope/新canonical/资产/SDK或agent；研究未执行不充go，计划未实施不充功能交付。

2026-11-05 / planning_count_correction：末轮逐项归属核验补入UI008（004身份修复，0261ae9为主祖先），正确统计49已交付/8未完；仅修正路线簿记数字，不改任何目标/AC/依赖/预算或执行状态。


2026-11-11 / planning_validation: pass（仅文档静态预检）。本轮8份新建/修订合同§0–10唯一、119任务/frontmatter/当前AC到任务覆盖一致、全部drafting/0完成；原任务/AC/V保留（UI058仅新增AC-11/V-S08），4份新合同各10任务/9AC/6验证。当前四仓可达221个Plan节点，无缺失引用/依赖环；UI039原活动影子/归档双文件按既有同号身份处理，不新计交付。十九份触达规划/导航的393个本地文件链接可解析，16处旧归档链接已纠正，定向diff空白检查通过；研究Spec、r187的621原表和pipeline研究输入hash保持不变。批次级检查确认979/980/981不反等新增增强/AG031，985/A不反等G2/G3或AG031，984只交OS覆盖切片而不要求AGE等全生态。未构建/运行用例、制作素材、创建实施树/新证书/新canonical、提交/push或打tag；此记录不代实现独立final。

## 10. 待澄清事项

D-01：055/AG022研究已交，最低组合仍待本包先证，不能把研究当产品完成；需变更预算则replan。D-02：轴/姿态/单sweep及所有权由T-02冻结，不默加滑墙。D-03：AGE M1原生门已由AG026签章，981所需A正式接面仍未齐，前期并行不豁免终验，也不等待本AGE M2总签。D-04：T-02先证不以f32业务式为前提（已按静态行值完成）；真实Auto移动积分（位置/速度f32字段与f32业务式）的前置已满足——981 `D-981-AGE-F32-SURFACE`关闭（AV `be8480cae`+A2R `a4950f0b5`，auto-lang master；receipt `77dc0fc26`），UI058仍不以int近似；跨仓状态观察/呈现按981 `D-981-F32-CONSUMER-WINDOW`另行回签，T-04 AV先证可用`execute_component_def`取`Value::F32`。材料唯一`D:/autostack/plan-artifacts/plan-058/auto-ui/`，开工核跨仓同号owner并登记用途/路径/在途，外部B/SDK副本/target/原日志探针全收自有子scope；merge摘要折Plan、全输出对账/guard清树/删除自有scope构建物，仅具名owner/理由/关闭条件hold例外；不穿透链接、不碰外仓同号、不创建第二权威树。D-05（已解）：T-02冻结分离；T-03采用新profile+per-row render-only表（非代理），E1(65536代理)被对抗反例否决并删除；SD-01须在`node2d-topdown-composition.md`沉淀profile 30/121、scene/9 layer、fault 60与旧档零漂移。D-06（T-04入口，2026-11-18）：981 AV 车道具名拒15个HostAlgorithm+HostIdentity（`hir_adapter.rs:5082 lower.age-gfx-lane`），且无gfx宿主派发，故"正常Node源AV整链"缺执行口；用户裁定路线A=回981 owner补SESSION/typed宿主端口，`IF-981-GFX-HOST-PORT`请求已并入981 §9；UI058不建私有runner/旧生成器旁路，T-04待该正式来源。T-04预备（`b071634c`）另实测 AGE 侧缺口G1..G5（`shape2d`无render-only声明位/Tick体assignment-only/无typed行句柄/frame str拼写未证/pin旧），已并入981请求补充；pin刷新为T-04第1步。三臂（`f6418120`）已按 `D-981-GFX-PORT-ABI`（`5ee4d23c0`/`381d7a364`）实现 segment/sweep/image2d 的精确 record 与 dispatch；收费仍归981 session；G1 源侧声明与 G2 语法仍待981。T-04 AV 阶段已真跑（`7a1503b7`）：overlay pin `64e8380f4`、`impl AgeGfxHost` 全 16 op、`tick_step` 宿主形态 40 Tick 每 Tick 32 credits、停位8.6/逐Tick oracle、预算 sticky/重置/stop；AV/GPU 同 Tick 预证 `ba85f33f`（34/35/36 质心 vs 投影≤1px、34→35 3.00px 排除晚帧、35→36 0.00px 不穿墙）；T-04 未勾，AC/A2R/GPU 全量/`use` 实例待 W2/T-05。**G1/G2 解除并已接（`0e67b1cd`，2026-11-18）**：981 G1+G2 落主 `6e456794d`（G5 pin `65a6b2cf`/收据 `f9ade844`）；fixture 改真实 `on Life.Tick`+源侧 `layer: render-only`，AV 侧对 981 的缺口清零。作者面两条纪律实测入档（021 token 走查含状态槽传播/每字段状态函数尾读；每函数静态准入≤32 拒计费链复制）——详见 §5「T-04 G1/G2 正式作者形结果」与 §9 该轮 check→finding 映射；后续作者源按该形书写。D-07（AC/A2R 阶段门，2026-11-18）：T-05 的 AC/A2R 格 blocked——AC 随后需「UI-W2-AC＋1068/T-04＋981 LANGUAGE/FUSION 所用 AC 域端口」，A2R 最后需「UI-W3-A2R＋本包 AC 正式来源＋981 RUST/FUSION 所用 A2R 域端口」；实测 PLAN-1068 `drafting` 0/10、981 `IF-AGE-GFX-AC`/B-RUST* 图形端口未交（"图形新增A2R实现/验证在UI-W2-AC正式后"）、W2/W3 无落地收据。不以 AV/A2R 代 AC；解除动作=1068 T-04 与 981 AC 域端口交付→UI-W2-AC 收据→本包补 AC 格→A2R 波。AV 阶段不受阻部分（GPU 连续帧链、use 实例、2:1 隔离）已交 `350af5b8`；作者面"业务→实例事件派发"（触碰路由）无 gfx 端口=具名缺口，并入 981 IF 清单（同 G1..G5 模式）。**r8（2026-11-19 用户裁定）**：AC/A2R 等待整体转移给 **PLAN-103**（`103-topdown2d-ac-a2r-wave-consumption.md`，drafting；转移清单见 §5 r8 块）——该外部等待不再阻塞 PLAN-058 的 AV 面收口与合并；触碰路由缺口随 V-S03 完整形归 PLAN-103 T-02（IF-TOPDOWN-TOUCH-ROUTE）。
