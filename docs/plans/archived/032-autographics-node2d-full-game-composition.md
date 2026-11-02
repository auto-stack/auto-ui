---
plan_id: PLAN-032
status: archived
feature_name: AutoGraphics完整G2精灵对象音画组合与预算收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 2
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-022, auto-ui/PLAN-025]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-full-game-composition.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-032：完整G2精灵、对象、声音/HUD组合与预算收口

## 0. 变更摘要

把020局部组件、022精灵、025通用声音/HUD和对象/触发/新局接入同一普通Node作者链，并证明冻结完整G2/G2-01..09在原32池、VM/资源包络内成立。引擎只做公共服务/作者接线和永久最小夹具，正式游戏在ag-examples/games/platformer-2d；不再只交一份隔离no_go研究。

见[三线设计](../design/autographics/m2-game-first-three-lane-roadmap.md)。drafting r2、0/10，暂不实施。本批与[AG014（原G2-001）](../../../ag-examples/docs/plans/014-autographics-complete-2d-platformer.md)、[AG013 r2](../../../ag-examples/docs/plans/013-autographics-gallery-g11-3d-node-migration.md)并行准备；游戏最终接线等本计划实际交付，不追开发tip。本轮不建实施树/探针/源码/资产、不构建/跑GPU/OS自动化、不创建agent。

## 1. 目标

1. 普通use/node/model/view-2d/msg/on/Life.Tick组合Player、Enemy、三同定义Coin、Goal、Boundary与root局部状态；真实Auto持运动/巡逻/计数/胜败/动作帧/cue/HUD，编译器无业务名特判。
2. 角色运动/相机、精灵帧与flip、动态敌人/触发/销毁、jump/coin音效、四槽HUD及accepted新局共同运行，支持固定完整G2正常路线/失败/重开和合法峰值。
3. 原Callback32、32物理lifecycle写、八状态Node/八slot/八pair及原资源/VM/source/时限保持；新组合操作如需新增，必须显式新版本/有限域/公开价目，旧API价目/字节不变。
4. 真Auto/Host/GPU/PCM/owner与独立consumer/闭包证明可交给游戏仓固定消费；引擎绿不代游戏/整个M2交付。

非目标：正式游戏源码/资产制作/发行、Gallery修改、3D表现音画/完整3D游戏、通用无限批处理/新物理/动态Node拓扑、多呈现/Life.Frame、任意字体/骨骼/PBR、ACE/AOT/ECS/CPC/跨端、抬总额或降低冻结游戏分母。030三维引用不是功能前置，不借此接管030/031或Runtime主题WIP。

## 2. 架构方案

共享Node组件/资源表示→新显式完整二维作者档（暂称components-2d-runtime，准确名称T-02冻结）→现有sprite/对象/trigger/audio/HUD与必要的新有界公共操作→真实生成Auto/固定SDK Session→Play独立准入/Host/GPU/mixer/native。优先复用现有hud2d/1的组合基础；如果新增操作，使用新profile/API/assembly身份，不覆写旧hud2d/1或025 presentation-2d。

新服务只处理数据快照/请求/引用/有限提交，不能知道Coin/Goal/Playing/bag或自动停输入。游戏通过正式接口使用，不写专用Rust Host。两依赖图独立：生成器读固定语言依赖，driver读7b987ca8/681 SDK；不拍平、不换当前SDK。

拟唯一实施/同树顺序独立复审树D:/autostack/.wt/ui-032/auto-ui，codex/plan-032-dev，本轮未建。与主题/Reader/030共享AST/load/translate/CLI/assembly接线按owner短窗，独立模块可并行；GPU/窗口/音频重型门同机单实例，target/report独占。

## 3. 技术栈

已核pilot src/{lower_components,lower_presentation,lower_components_game,load,translate,main,lib}.rs，dialect src/node2d/；driver api/{runtime_objects_v1,runtime_triggers_v1,sprite2d_v1,public_audio2d_v1,public_hud2d_v1}.at及authoring/{behavior,play,host,runtime_round,public_audio,public_hud}等现有模块，既有node_sprites/node_presentation门与Host/native/mixer。

拟新增pilot src/lower_components2d_runtime.rs、fixtures/components2d-runtime/、tests/node2d_full_game_gates.rs、tools/verify_plan032.py；driver tests/node2d_full_game_flow.rs、必要public_2d_composition_api.rs及新版本化API单元（新路径，T-02冻结准确布局）。复用原PNG/PCM/font/几何解码和现有Frame准入，不能复制另一运行时；永久最小夹具归引擎，临时先证仅在scope。

## 4. 需求分析与背景调查

用户2026-11-02批准游戏优先路线及引擎/完整游戏/Demo三线，每批可并行三计划；本轮授权更新设计/路线和规划，不是实施/发布授权。2D引擎、独立游戏与Gallery各持合同，正式游戏放ag-examples/games/platformer-2d，不放引擎；3D表现音画在共同设计明确后继支持域，本批不预占号。用户随后批准游戏和Demo同仓分区，G2-001迁为ag-examples/014；本次r2仅同步归属/引用，引擎范围/预算/验收与任务保持。

观测基线auto-ui ab3910de，规划期间030刚落主804d7303（归档/清理在案）；与2D无功能依赖，其同会话review记录另由AG013核独立门，不自动称独立final。031主题/Runtime有其它owner未提交实现，本轮不触碰。实际开工T-01重绑最新已交付源码/锁与真实owner，不按标题或status推就绪。

权威：[020局部组件](../specs/auto-graphics/stateful-node-components.md)、[022精灵](../specs/auto-graphics/node2d-sprites-and-frame-animation.md)、[025通用音画](../specs/auto-graphics/node-presentation-services.md)、旧[G2研究](../../../auto-lang/docs/specs/auto-graphics/g2-full-loop-budget-feasibility.md)、[冻结玩法](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[HUD](../../../auto-lang/docs/specs/auto-graphics/public-hud.md)/[触发](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)。

实读SHA256：022 Spec=f997bae66305eef07d5eb902dcc5ba195350c2ffff38360fcd456dda9877f1b2；025 Spec=62090294bcbdb83affc2194a0e93e0d284290578e28d162f1ce6f7748528d610；lower_presentation.rs=b6cdbf3fd68cd068eb38880d19f454b78df539871053cde418e9da315047d344；public_audio2d_v1.at=dab843e7ec9bc3bf2f72e7044dce8267e8508d409afdbcbd84d1f108d23456bf；public_hud2d_v1.at=54f297a400f0d254bc8699f62413917515a486cee3f3691d8be72eaa0a505982；runtime_objects_v1.at=fe9a8948270d6a8756661830aeb1c6ce7f35c18eb2f3e65d5befaf8595408dc5；runtime_triggers_v1.at=b2b75b168570d8cf86a470b65511d3923a04d0ae0848caa694a2b619fc0b8ce1。

已知缺口：022与025是不同作者档；025固定kind=-1没有sprite写。旧研究正常分支input3+move26+grounded1+sprite1+patrol1+next1=33，三件与目标的完整事件峰值亦未证实；025有限G12通过不能替代完整G2，特别不能用作者halt停走/跳过巡逻或结构性不可达目标凑预算。

## 5. 详细设计

### 5.1 准备与最多两候选完整先证

T-01核020/022/025实际交付/Spec/独立review/冻结、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467与681文件内容锁、生成器语言/依赖锁/toolchain/owner，开工才建唯一树/scope。历史对象缺失只用受检固定副本，不换SDK。重读冻结G2参数/原失败case与合法控制，不能只凭旧研究的failure banner。

T-02仅比较A现有组合、B本节公开的版本化有界操作。均走源/闭包→真实Auto/Host→完整路线与峰值→GPU/PCM/HUD，冻结准确作者语法/引用/调度/API编号/价目/容量/诊断/Frame/资源账/游戏对接文档。允许据等价事实选A免新增操作；若选B严格受已写支持域约束。两者都不成立则功能未完成，停止依赖实现并needs_replan，不以no_go研究结案/无限第三方案/涨32或删玩法完成。

### 5.2 组合作者与状态边界

一个物理2D角色/静态solid、跟随夹界/letterbox、可辨识敌人持续巡逻、三个独立Coin/Goal/Boundary/root逻辑，根＋Player＋Enemy＋三Coin＋Goal＋Boundary在八状态Node内；纯平台/不可变资源不占状态槽。Player自己的vy/grounded/pose/facing，Coin各持handle/collected，Enemy持巡逻，root仅关卡状态与显式顺序；普通use/匿名msg/父子receiver沿020，不硬编码业务字段/Node名。

具名资源/帧/cue/事件目标，引用在下降层映射，不要求作者猜索引；必要的二维名义绑定复用共享机制但不依赖030三维profile。支持同定义Coin多实例局部状态和receiver，模型参数一次初始化、只读参数与局部赋值分开；source/roles/资源闭包完整公开。业务逻辑与纯素材制作归游戏；引擎可用最小同语义夹具，不复制正式游戏项目。

保持固定dt1/60、原几何/速度4/重力20/jump8/clearance.002，连续采输入/移动/敌人更新，N几何事件在N+1读、一次收集真销毁、未齐目标不赢、危险/敌人/越界失败、终局fresh Enter accepted新Run。不能移动金币/删最高分支/收集时停走、停敌人、隔帧消费或Rust替代玩法。

### 5.3 新有界公共操作提案与兼容（非当前已实现）

A复用已有ag_actions_snapshot和ag_present2d（帧/flip＋四声音＋四HUD）加单条事件/销毁；需重放完整峰值，不因平均值低就go。B在新显式profile增加以下通用价目；旧next/getter/despawn价格/语义/字节原样，不能把旧单调用改成免费：

| 提案操作 | 支持与拒绝域 | 新profile拟价目 |
|---|---|---|
| events.snapshot | 每callback至多一次，物化上Tick完整0..8条事件、原pair序/phase/source/绑定代/geometry及Run/tick，全身份int，定长容量+count；空合法，第二次/混单条cursor具名拒绝，未读不跨tick排队 | 快照整体1 credit，本地物化值读取属Auto计算但VM/CPU全计 |
| objects.despawn_batch | 显式调用，0..8长度域T-02冻结空批语义；最多8个不同live完整代句柄，同Run，先全验证再修改；重复/错Run/旧代/第9项具名拒绝，不改普通despawn的即时语义 | 一次批调用1 credit，物理写计数仍加实际N，第33物理写先错37；空批也不免费 |

快照是独立不可变值，不是私读VM字段或跨Tick免费游标。快照保留的旧source句柄用于后续操作仍走原alive/generation验证；after next snapshot的旧值只是历史事实，不授权操作新代。批销毁不合成exit、不自动重绑，批内失败不能部分修改可观察对象表；各对象死亡/身份/Host admission按原公共语义独立校验，loop/价格不能在Rust中按game字段推导。

原32池内候选峰值：inputSnapshot1+charMove26+grounded1+enemyMove1+eventsSnapshot1+despawnBatch1+presentationBundle1=32；无despawn分支31。此是待实测账，不等同可行性。三币/coin+enemy+goal竞争必须读全部所需事件并按作者规则处理，batch不是偷偷把事件挪后帧；编译器只负责通用快照/类型与有限提交。

新API命名/版本/marker/profile/wire编号由T-02查注册表唯一，新增准入/身份/故障域必须明确。可复用旧Frame已含对象/事件行则不额外复制世界；若需扩wire记精确长度/类型/零padding和独立Host验证。旧profile不自动获得新保留函数，不改旧v1..HUD/022/025/028/030世界字节或用户合法helper命名。

### 5.4 表现/声音/HUD与资源

把022 typed PNG/帧/flip和025资源/cue/HUD接到同一公共呈现，不再kind=-1只音画；原单bundle≤1动态视觉+4声音+完整四HUD，显式禁止混写/双bundle/错代/错frame。最小G2需要玩家idle/run/rise/fall/flip和敌人可辨识sprite/持续位移；若冻结验收要求超出一次动态视觉域，T-02必须明确等价公共扩展/价目或needs_replan，不能删必要表现或绕过旧混写门。

jump与每Coin一次声，四请求全部传递，五请求仍具名拒绝不截断；真实合法峰值所需声音不得被抑制。原8voice、原gain/精确ordinal/oldRun拒绝，Pause/Step/Stop/native失败规则保持；GPU/HUD/音效共同逻辑接受不承诺物理扬声器同时到达。HUD四槽各≤32 ASCII/总128、固定Lilita/OFL，不用系统字体或把DOM overlay当完整生产UI。

保八资源身份/32MiB raw/CPU-GPU各64MiB、原PNG域与GLB无关域、font atlas/warm/last-owner和全部旧守护；128scene与objects的120base+8reserved、八slot/八pair、八状态Node分别计数。原1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s、用户256KiB/managed96KiB/combined352KiB、8ms pump目标不变。批快照/数组/metadata/缓存纳原账，不能用“逻辑字段少”冒充完整内存计量。

### 5.5 正式游戏对接与交付

T-02冻结引擎接口文档并折Plan，供AG014独立游戏作者草稿准备；不得传开发tip/未审二进制当固定consumer。T-08..09实际落主/Spec/来源闭合后，游戏T-05才绑定实际交付源/SDK/锁/toolchain/参数/generator/runner hash及模块/资产许可闭包。接口变更仅有新revision和影响清单，不覆盖其他仓实现。

新profile A→B仅闭包冷再生/真实Run、SaveAs关开、坏件拒绝→精确恢复→再Run、20轮newRun/Stop/churn/owner、同Tick相机/帧/PCM/HUD实测。旧档按真实受影响case验证，不能以保存旧exe冒充永久来源；新canonical只在review/merge后落位。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-full-game-composition.md | 精灵/通用音画作者分档、完整G2未证→新显式组合/具名局部组件、如需新增有限操作则公开域/价目/身份及原32内完整证据；旧档不变 | 正式游戏固定消费，避免把G12/旧no_go当完整G2结论 | AC-01..06/08 |

## 6. 测试设计

新入口待T-03..06建立执行，不是现成证据。无用户指定实现/内部/final人选；同权威树顺序、独立final不同上下文，不能用实现同会话自检冒充；本轮不创建agent。AG专项只测真实触面，非每次全AG。

| ID/AC/owner_stage | 不变量/合法与非法输入/状态 | 实际入口/区分oracle/边界 |
|---|---|---|
| V-S01 / AC-01/02 / work | 八逻辑组件/同定义Coin隔离、任意合法命名/参数/use/精灵/cue/HUD，错引用/只读参数写/第9状态非法 | 新node2d_full_game_gates→实际generator/scene/world/identity/closure，局部值/receiver/目标正确，位置诊断无半产物；非只有语法绿 |
| V-S02 / AC-02/03 / work | 完整G2-01..09原路线/死亡/重开、三币/Jump-land/未齐目标/敌人目标竞争、普通移动 | 新node2d_full_game_flow真实Auto/Host/GPU/PCM＋独立f32/几何/事件/PNG采样/字形ROI/PCM oracle；冻结原坐标/阈值/步数，坏轨迹/错帧/漏第三事件真失败 |
| V-S03 / AC-03/04 / work | 选B时0/1/8事件批、空批计费/8销毁/写帽；9项/重复/旧代/混读/第二快照/帧块篡改 | public_2d_composition_api＋真实Host准入，精确int身份/原N→N+1/顺序/拒绝无部分修改、32pass33fault10/第33物理写37；选A必须等价峰值原额实证，不静默跳此义务 |
| V-S04 / AC-03/04 / work | 一视觉/四声/四HUD、五声/双bundle/混写/旧Run/坏资源、actor/slot销毁重生 | 真新profile/GPU/mixer/native：玩家帧/敌人位移/字形/PCM来自同一Auto状态，ordinal一次/不丢声/共同接受与原last-good；物理声学缺证具名，不把虚拟设备当扬声器 |
| V-S05 / AC-03/05 / work | held/blur/Pause/中性Step/Stop/fresh/round失败/20轮、所有旧档/旧合法helper | 实际Host事件与新源/owner，按原输入边沿/文档Undo/旧代/声音清理，新资源最后owner与warm账平坦；旧022/025/受影响profile字节/负控不变，OS自动化暂停不冒称实跑 |
| V-S06 / AC-05/06 / work | source-build/闭包A→B/SaveAs/损坏模块资源锁/partial报告拒绝→恢复 | 新verify_plan032/真generator与runner、真实来源/哈希/再Run，覆盖固定新档，不是全AG/游戏内容交付；无生产者路径/网络回退 |

原G2-01..09逐项覆盖：输入释放/地跳空中拒二跳/地墙顶边坑/三币一次真销毁/巡逻触敌失败/未齐与已齐目标胜败新局/跟随夹界相机/idle-run-jump-fall方向/事件一次声音与正确HUD。原UI011完整失败轨迹/合法隔离控制保留作为新旧区分输入，不降低原几何或把skip计PASS。

拟树内命令：`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node2d_full_game_gates -- --test-threads=1`；driver改manifest为experimental/autographics-auto-driver/Cargo.toml并选node2d_full_game_flow及必要public_2d_composition_api；`python experimental/autographics-node2d-pilot/tools/verify_plan032.py --controls`。新路径T-03..05建立，T-02冻结准确CLI/输出；期望完整原路线与全部具名负控闭合、非partial/零测试/失败前缀计量。

定向兼容：node_sprites_gates/flow、node_presentation_gates/flow、公共对象/触发/HUD/audio的实际触面、020共享组件/receiver及旧verifier；T-02列受影响命名case并集，冻结提交/输入后有据复用有效证据，不跑根crate或全AG作日常merge门。准备/冷编译/发现/执行/清理/整命令时间与实际运行/条件排除数分别记录，硬守护与8ms目标分开，无exact meter不伪造余量。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 同一普通Node/use局部组件链表达角色/敌人/Coin/Goal/Boundary、帧/cue/HUD，八状态内、无业务Rust/保留字段特判 | V-S01 |
| AC-02 | work | 同一个完整G2程序逐项通过G2-01..09、正常/失败/重开及原几何/合法峰值，无停走/停敌人/少读事件/改关卡 | V-S02 |
| AC-03 | work | input/运动/巡逻/事件/销毁/帧/PCM/HUD共同运行，独立数值/GPU/音频/字形、生命周期/last-good与负控真触发 | V-S02..05 |
| AC-04 | work | 每分支原32/32物理写/八状态/八slot/八pair/120+8/资源VM源码额内实测；如选B则新域/价目/诊断显式、旧价目不变 | V-S03/04 |
| AC-05 | work | 旧profile/022/025字节与合法/非法输入保持、owner无泄漏，新版本不自动升级旧源/保留名 | V-S05/06 |
| AC-06 | work | actual source-build/SDK681/工具/锁/许可闭包、A→B/SaveAs/冷再生/恢复/20轮齐，SD候选及游戏可消费对接文档移交 | V-S05/06 |
| AC-07 | review | 同树独立final逐AC/实际diff/SD/告警/遗漏延期workaround审查，finding回work复核pass，绑定提交/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际合主，SD-01 canonical核字节/来源、归档导航，游戏T-05才锁实际交付 | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组，全输出对账/摘要折Plan/删scope构建物与具名hold/最终hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；020/022/025已交付 | 核§4/SDK/原G2/owner/写面，唯一树和scope登记 | AC-01/05/06 | [x] 2026-11-02 base816b8801：§4七项SHA256全部字节相符（022/025 Spec、lower_presentation、4个.at）；sdk/frozen 681/681对source-lock.json零失配；G2矩阵与预算研究原文重读（32池/move26校准/原33拒绝/isolated账）；树D:/autostack/.wt/ui-032/auto-ui分支codex/plan-032-dev无链接；scope=D:/autostack/plan-artifacts/plan-032/auto-ui/SCOPE.md。观测：verify_plan879红为主上既有（879 identity 10-29冻结早于023在11-01改.gitattributes，两checkout字节一致且git-clean）；本计划门为verify_plan032，不改.gitattributes。主checkout WIP=031 owner在途（§4预declared），零接触 |
| T-02 | work | T-01 | A/B最多两候选完整先证，冻结公共域/价目/CLI/profile/协议/原G2与峰值、游戏接口文档；不足needs_replan不结案 | AC-01..06 | [x] 2026-11-02 选B（A经真实路线证伪）。冻结：profile=autographics.components2d/1（number 116/profile 27/cli components2d，全hud2d栈+组合单元派生装配，旧profile字节不变）；新单元api/public_composition2d_v1.at：ag_events_snapshot()=1 credit物化0..8事件（pair/phase/绑定代/source身份/geometry+run/tick，第二次/混cursor具名58，字段读0 credit），ag_object_despawn_batch(n,h0..h7)=1 credit、0..8域、先全验证后修改、重复/第9项具名59、旧代/错Run沿用33、第33物理写37、空批合法不免费。B世界tests/fixtures/plan032/world-g2-components2d.at（sha dbb65232…）+scene-g2-components2d.agscene。实证：VM级node2d_full_game_flow——完整G2路线win t=887每tick绿，walk tick=31（+1恰32池绿/+2具名10红），collect tick恰=32（+1恰在首次收集108红），敌触/坑败负控绿，A同脚本走绿至首次收集即具名10红；Host级node2d_full_game_media——真Host帧块idle[0,0]/run[32]/rise[64]/fall[96]+flip、HUD 0..3/Playing/Won字形、敌实例巡逻渲染、相机夹位揭示、PCM经真mixer 8行/7包逐位相符、终局Enter新Run（tick 884→30、币复原） |
| T-03 | work | T-02 go | 公共有限操作/版本/独立准入及API正负测试（选A无需新增则记实证）；旧API字节/价目不变 | AC-03/04/05 | [x] 2026-11-02 driver public_2d_composition_api 7/7绿：域负控具名（快照二次/混cursor双向/越界=58；批n=9/负/重复无部分修改=59；旧代/错Run沿objects单元33）；8句柄一批1credit全销毁、空批合法；快照字段/身份/绑定代/run与事件形状相符、记录句柄仍走原alive/generation验证（二次销毁33）；旧profile（triggers/sprites/audio/hud/presentation）装配字节pin：public cursor各恰1处、无组合单元/_ag_cmp重命名，components2d恰1包装+1内部+entry钩子。play.rs空间构造白名单补components2d；测试专用assembled_units_for只读视图 |
| T-04 | work | T-03 | generic Node组合降层/资源/receiver/present与gates/真实flow，原G2最小夹具，无正式游戏写面 | AC-01/02/03 | [x] 2026-11-02 dialect components2d块（audio/cue/hud/trigger沿025逐字+patrol/atlas行；span→range避关键字；parse continue修复）+pilot lower_components2d_runtime（025管线复用：prepare_pres/场景JSON；PresGen.mode分支4注入点；冻结根state/角色pose/facing具名校验；obj.despawn→批调用与input.snapshot→r_snap缓存translate服务；region rule沿F-6）。fixture 9模块+4资产→generated-components2d-runtime/g2（world sha d0c6c196…，scene api components2d/1，atlas sprite绑定，4 cue/HUD/14行）。pilot gates 3/3（再生成字节一致+骨架契约pin：无cursor/held/pressed/halt、5行缓冲、kind=0收口；具名拒绝5项）；driver node2d_full_game_generated 2/2：生成世界完整G2路线绿（win）、walk+1恰32池/+2具名10、collect+1恰首次收集t=108（与手工世界同tick）、敌触/坑败负控+终局Enter绿 |
| T-05 | work | T-04 | verify_plan032与完整G2/所有峰值、预算/协议/资源/音画/生命周期负控 | AC-01..05 | [x] 2026-11-02 tools/verify_plan032.py --controls 全绿：A→B冷再生三产物字节一致（world sha d0c6c196…）、identity/闭包/provenance绑定（9模块+4资产、consumer=driver三flow）、骨架契约pin（14 pin+5禁）、缺失state具名负控；--controls委托driver四套件全绿（public_2d_composition_api 7、node2d_full_game_flow 2、node2d_full_game_generated 2、node2d_full_game_media 4含真Host GPU/PCM/新局/损坏拒绝→精确恢复→再Run）。覆盖边界如实记录：生成世界VM路线/预算=generated套件，音画GPU/PCM=media套件（手工同账世界，同一profile/链路），20轮Run/Stop churn沿023/025既有profile无关play套件 |
| T-06 | work | T-05 | source-build/A→B/SaveAs/恢复/20轮、受影响兼容并集，SD候选与真实工具/接口移交 | AC-05/06 | [x] 2026-11-02 SDK A→B实证：本worktree .sdk-source由sdk/frozen 681文件冷物化（prepare_sdk --from-snapshot PASS）后全部构建/测试跑通=源闭包→真实Run；损坏消费恢复+fresh Run（media套件）；20轮churn/SaveAs沿profile无关play既有套件（023/025）不重复。受影响兼容并集全绿：pilot 9套件84测试（presentation/components_game/sprites/components/queries/g11/lower/3d×3），driver 6套件56测试（presentation/sprites/components flow、triggers_api、character2d_api、t05_game_budget）——共享translate/lower_components改动面全数回归。SD候选=docs/specs/auto-graphics/node2d-full-game-composition.md（worktree内，merge落位）；接口移交=该文档+identity.json provenance（consumer绑定driver三flow） |
| T-07 | review | T-06 | 同树独立上下文final/精确revision，finding回work复核，pass后merge门pending | AC-07 | [x] 2026-11-02 独立子代理上下文final pass（见§9 review receipt，绑定c98aeec7/r2；4自选反例全过、AC-01..06自证、SD-01批准；OBS-1/2非阻断） |
| T-08 | merge | T-07 pass | 冻结提交/输入/定向门或有据复用，来源核后实际合master | AC-08 | [x] 2026-11-02 master已前移（816b8801→771dd10d，029文档提交、零文件重叠）；rebase后range-diff 5/5全等（patch-identical，c98aeec7→a8c04d22安全重写证明）；ff-only合master（tip=a8c04d221f9e…）；落主smoke：verify_plan032 --controls --skip-driver绿（再生成字节一致/identity/骨架pin/负控；driver四套件沿patch-identical基线复用） |
| T-09 | merge | T-08 | SD-01 canonical/来源核、归档/导航和可锁交付身份，不写派生JSON | AC-08 | [x] 2026-11-02 canonical落位docs/specs/auto-graphics/node2d-full-game-composition.md（sha ffbb5752…，状态行激活为canonical交付=a8c04d22）；场景绑定d0c6c196…在主；归档docs/plans/archived/同名文件status:archived；未写specs.json/派生JSON |
| T-10 | merge | T-09 | fresh guard清树/分支/空组，全输出对账/删scope构建物/登记最终收据 | AC-09 | [x] 2026-11-02 wt-guard clean→删worktree D:/autostack/.wt/ui-032/auto-ui+分支codex/plan-032-dev+空组目录；plan-artifacts/plan-032 scope折叠后删除（scope含SCOPE.md与临时脚本，全部折叠进本Plan）；registry刷新；本仓库无生产部署面（test-only消费者，落主products已提交）——deployment观测如实记录 |

## 9. 复审记录

2026-11-02 / stage: new / plan_id: auto-ui/PLAN-032 / plan_revision: 1 / outcome: pass（合同可审阅移交；A/B可行性、尤其新批操作价目是公开提案，不是已批准实现/独立final） / next: 用户启动后work T-01/T-02。0/10。三线授权仅规划；已有底层组合和022/025边界已核，旧no_go不外推，新G2未通过。本轮无实施树/源码/资产/探针/提案Spec/构建/GPU或agent。

2026-11-02三线规划静态预检：三合同均0..10节、10T/9AC/6V-S/1SD，work6/review1/merge3、drafting0/10与阶段门一致；AG013原25名及顺序逐字保持，G2原九项/原几何与预算不减。241个当前管理文档本地链接经核（含历史归档定位修正）；本轮新增引用无断链。活动/归档编号唯一，UI next033（031为其他owner已保留号）、AG next014、游戏next002。metadata实际生成路径16/22，`python tools/gallery.py validate --all`退出0、12逻辑项/22入口/许可一致；022/025/030 canonical SHA与引用一致。未创建三计划实施树、提案canonical Spec、游戏源/素材/探针/构建/GPU/agent。以上仅规划/目录验证，不是实际新链可行性go、游戏运行验收或独立final。

2026-11-02 / stage: new / plan_id: auto-ui/PLAN-032 / plan_revision: 2 / outcome: pass（归属/引用修订；暂不实施） / next: 用户启动后work T-01/T-02。正式游戏现为ag-examples/PLAN-014 r2、games/platformer-2d独立工程；原G2-001退为指引。032原10T/9AC/6V-S/1SD、原预算/新操作提案、引擎与游戏交付门不变，仅同步作者合同归属；旧r1预检号源/两仓口径保历史。本轮未实施或构建，不接管AG013/主题WIP。

2026-11-02 / stage: work | plan_id: auto-ui/PLAN-032 | plan_revision: 2 | outcome: pass（execution_done） | worktree: D:/autostack/.wt/ui-032/auto-ui | branch: codex/plan-032-dev | base_commit: 816b8801 | code_commit: c98aeec7 | dependency_revisions: auto-lang 835816fa3（.wt/ui-032/auto-lang detached 只读；driver .sdk-source=sdk/frozen 681 冷物化 PASS） | task_ids: T-01..T-06 全部[x]（current_step 6/10；T-07..T-10=review/merge 所有方未动） | evidence: 选B（A由真实路线证伪：同脚本走绿至首次收集即具名10）；冻结 profile=autographics.components2d/1（116/27/components2d）+组合单元（events_snapshot=1 credit、despawn_batch=1 credit 0..8域、具名58/59/沿33/37）；手工世界 sha dbb65232…、生成世界 sha d0c6c196…；峰值实测 walk=31（+1恰池/+2 fault10）、collect=32（+1恰首次收集t=108，手工与生成同tick）、app_start=22；完整G2-01..09两世界全绿+敌触/坑败/终局Enter新局；真Host GPU帧/flip/HUD字形/敌巡逻/相机夹位/PCM真mixer逐位（8行/7包）；损坏副本拒绝→精确恢复→再Run。verify_plan032 --controls 全绿（再生成字节一致/identity/骨架pin/负控+driver四套件）。兼容并集：pilot 84+driver 56 全绿。SD候选=worktree docs/specs/auto-graphics/node2d-full-game-composition.md。scope=D:/autostack/plan-artifacts/plan-032/auto-ui/。 | blockers: 无 | next_responsible_role: review（T-07 同树独立上下文 final，绑定 c98aeec7/plan_revision 2；merge 门 pending）

2026-11-02 / stage: goal | plan_id: PLAN-032 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-032/auto-ui | commit: c98aeec7 | next: review

2026-11-02 / stage: goal | plan_id: PLAN-032 | plan_revision: r2 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-032/auto-ui | commit: c98aeec7 | next: merge

2026-11-02 merge consolidation receipt (PLAN-032:r2)：
- prepared：reviewed基线c98aeec7（base 816b8801，auto-lang 835816fa3，SD候选已随T-05提交）；master前移至771dd10d（029文档、零文件重叠）
- landed：rebase到master后range-diff 5/5全等（c98aeec7→a8c04d22映射：fdf013b0→a208d215、7b9816f5→d4494027、c98aeec7→a8c04d22）；git merge --ff-only落主，master tip=a8c04d221f9e3b25a38cc7885a747a67c0960e09（无merge commit）
- canonical_verified：docs/specs/auto-graphics/node2d-full-game-composition.md在主（sha ffbb5752a7d17e8dd59941457172cfa8419440957701a9a88ed464b989ce2ae9，状态行激活为canonical交付）；场景绑定d0c6c196…在主；落主smoke verify_plan032 --controls --skip-driver绿（driver套件沿patch-identical基线复用）
- archived：docs/plans/archived/032-autographics-node2d-full-game-composition.md，status: archived，T-01..T-10对账（6 work+1 review+3 merge全[x]），本收据与docs提交同入默认分支（PLAN-104 UAT-K13）
- deployment观测：本仓库（auto-ui）交付面=测试/研究套件与committed products，无生产进程/release二进制消费——Landing≠deployment观测如实记录为不适用

2026-11-02 / stage: review | review_scope: final | reviewer_identity/limitations: 独立子代理上下文（与实现会话分离；未伪造模型/会话ID）；限制如实记录——复用verifier子进程的media套件未三次重跑、兼容并集抽查+装配字节pin未全量重跑（84+56为实现会话数字，审查者抽查代表性旧档套件+CE-3边界）、无声学/OS自动化观测（沿§10 D-04缺证具名） | plan_id: auto-ui/PLAN-032 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-032/auto-ui | branch: codex/plan-032-dev | reviewed_commit: c98aeec7e5c9a00c5f255d2507f17b04dde9b0ce | base_commit: 816b8801 | dependency_revisions: auto-lang 835816fa3（detached只读）、.sdk-source=sdk/frozen 681（审查者自数） | spec_inputs: worktree docs/specs/auto-graphics/node2d-full-game-composition.md（SD-01候选；裁定：批准T-09落位canonical；候选与§5规范增量行逐项相符、profile身份/价目/峰值账/兼容声明与实现一致、state/pose/facing判定为通用统一冻结作者契约非业务特判——预算分母未降） | acceptance_results: AC-01..06全部pass（审查者逐项自证：diff全文审读、生成世界grep无ag_action_held/pressed/halt/ag_event_next、walk=31/collect=32为真实断言pin、nodes_gates/flow/generated/media四套件自跑、CE-1双快照同tick=58、CE-2批9/负数=59且对象表无部分修改（正控n=3=3销毁）、CE-3旧档装配边界（无新符号/cursor仍public）、CE-4场景sha名下世界副本1比特翻转=加载拒绝→恢复绿）；AC-07=本审查（pass） | pending_closeout_gates: T-08实合master+定向门/证据复用核、T-09 SD-01 canonical落位+来源核+归档导航、T-10 fresh guard清树/对账/最终hash登记；AG014 T-05绑定交付身份待T-08/09后 | findings: 无阻断项；OBS-1（环境）：审查者首次media套件3/4因其scratch crate 792包冷构建并发饱和违反单实例前置，单实例重跑4/4（45.55s）——无代码含义，merge门保持单实例纪律；OBS-2（文档精度）：app_start 22 credit为实测数无测试pin——不阻断，可选merge时补pin | evidence: 审查者scratch crate（temp p032-independent-review，只读依赖被审树）；verify_plan032 --controls exit 0（审查者自跑）；diff 53文件+6795/−29；无specs.json/.autoos写入、.gitattributes零改动（879既有红未触碰）、无junction、scratch未入仓 | next: merge（T-08..T-10）
2026-11-02迁移静态预检：ag-examples独占核活动/archived/archive编号后分配014，next015；原G2-001移出后仅留指引/历史，无第二套活动frontmatter。AG014 r2与UI032 r2均0..10节、10T/9AC/6V-S/1SD、drafting0/10；AG014§5.2原关卡/九项规则与§5.3原资源/预算对照源提交逐字一致。239个当前管理文档本地链接可解析；`python tools/gallery.py validate --all`退出0，12逻辑项/22入口/许可一致；本轮各仓白名单diff空白检查通过。AG013在途合同/实现未编辑；未创建游戏实现目录/新实施树/提案canonical Spec/源码/素材/探针，无构建/GPU/agent。以上是迁移规划与目录验证，不是新游戏运行或独立final证据。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | A复用是否足够或B必要、新API/marker/profile/诊断/Frame准确形态 | work T-02最多两候选完整证明并冻结；不得扩大批容量/改价目/抬总额或研究no_go代功能交付 |
| D-02 | 三币/coin+enemy+goal/Jump-land及声音峰值的完整原额 | work T-02/05真实最高分支；必要表现超单视觉域须明确公共扩展或needs_replan，不删验收 |
| D-03 | 游戏与引擎接口冻结/实际交付双阶段和owner接线 | work T-01/02记录接口revision/受影响方，游戏不读tip，最终仅锁T-08/09实际身份 |
| D-04 | 独立final上下文/重型门与OS/声学缺证 | review/work分别负责，缺证具名，不把实现自审/虚拟声学冒充独立/物理证据 |

材料唯一D:/autostack/plan-artifacts/plan-032/auto-ui/，开工登记owner/用途/在途；探针/日志/截图/包/外部消费者/外置target只归scope，永久产品/必要回归归仓，禁止attachments/第二暂存区。merge折摘要后全输出对账并删自有scope/构建物，仅具名未解决问题持有；禁止worktree链接，清树先fresh D:/autostack/wt-guard.sh，旧specs.json只读。
