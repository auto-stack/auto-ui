---
plan_id: PLAN-035
status: archived
feature_name: AutoGraphics3D声音HUD与完整G3组合接线
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-012, auto-ui/PLAN-013, auto-ui/PLAN-025, auto-ui/PLAN-032, auto-ui/PLAN-034]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-audio-hud-composition.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-035：3D声音/HUD通用Node与完整G3组合接线

## 0. 变更摘要

复用公共audio3d/hud3d和034具名表现，在普通Node/use/局部model-msg-on中接通声音、HUD、视觉bundle及对象/触发/相机/accepted新局，证明原完整G3-01..08/合法峰值在原32池与资源/VM包络内。正式G3游戏仍归ag-examples/games/platformer-3d未来独立合同；本期仅公共引擎与永久最小整链夹具，不冒称完整游戏交付。

[共同设计](../design/autographics/node3d-presentation-audio-and-g10-migration.md)。drafting r1、0/10，暂不实施。T-01/T-02可独立做原G3/素材/许可/协议/oracle及文档接口准备，T-03完整组合先证硬等034实际交付/独立review/Spec与来源；不读034开发tip。AG015固定034，不依赖本计划。本轮不建实施树/源码/素材/探针/提案Spec，不构建/GPU/OS自动化或创建agent。

## 1. 目标

1. 任意业务名字的Node/普通use/局部model/msg/on可请求具名cue、gain、四槽HUD及视觉clip/phase/yaw；支持034的模型/引用/局部组件，真实Auto持玩法和cue/文字选择，生成器没有计分/胜败/Idle特判。
2. actor-free 3D音画服务小夹具与含唯一character/camera/三Coin/危险/目标/越界/重开的完整G3夹具成立；无视觉请求可只提交声音/HUD，不为服务制造假Player。
3. 完整G3八项同一程序逐项/峰值实测，不停移动/漏事件/删必要表现声音来过32；如需有界组合操作，显式新profile/API域/价目/错误和独立准入，旧档及价目不变。
4. 实际Auto/Host/GPU/PCM/native/HUD/owner、冷消费/保存迁移/故障恢复与034/旧2D兼容闭合，交付未来游戏可固定消费的公共组合；无正式游戏或整M2认领。

非目标：另写mixer/font/物理/动画解码器、G10内容或完整G3游戏、声音空间化/音乐/混音总线/任意字体/CJK/富文本/生产UI overlay、多视觉无界bundle、多个可控角色、动态拓扑/跨场景、ACE/AOT/ECS/CPC/跨端、抬原32或资源/VM上限。032保留字段/自动巡逻约定不扩成3D业务协议，030/034有限端口限制如需改变必须在本计划明确证明。

## 2. 架构方案

普通Auto组件资源/声明/绑定/handler→034具名纹理/动画＋025通用cue/HUD请求及035显式三维收口→固定scene8/受检用户Auto→原3D角色/相机/对象/触发/动画/audio/HUD组合→Host共同逻辑accept→GPU与有界native服务。音频设备真正采样和GPU呈现分别计时，不许诺物理扬声器与显示硬件原子同步。

拟唯一实施与同树独立review树D:/autostack/.wt/ui-035/auto-ui，codex/plan-035-dev。T-01可为独立准备登记本树/scope，034实现只读；T-03绑定其实际落主后固定来源，pilot当前语言依赖图与SDK681 driver图始终分开。不得用兄弟开发树/未审SDK或共享target越门；公共AST/load/translate/assembly owner短窗、GPU/窗口/音频重型门机器单实例。

## 3. 技术栈

已核api/public_audio3d_v1.at、public_hud3d_v1.at（ag_present3d一个视觉/四声/四HUD，kind=-1无视觉）、public_audio_v1.at、public_hud_v1.at及authoring/public_audio/{bank,codec,mixer,native}.rs、public_hud/mod.rs；pilot lower_presentation.rs和translate.rs::SfxEnv仅二维，034新typed资源/clip尚未交付。032 public_composition2d_v1.at已有0..8事件快照/显式批销毁，但不代表其profile可直接用于3D。

拟新增lower_presentation3d.rs、node3d_audio_hud_gates/flow、必要public_3d_composition_api与verify_plan035.py、fixtures/node3d-presentation-services/及g3-full-loop/；均提案路径，T-02列支持边界、T-03冻结实际CLI/协议，T-04..06建立执行。只新增公共服务/最小夹具，不把关卡业务算法放Rust或私有Host测试命令。

## 4. 需求分析与背景调查

用户2026-11-03明确授权本批规划而非实施；同日用户再以`/goal 完成计划035`授权auto-plan-goal全链交付（work→review→有界修复→merge，含实施树创建/独立review/落主归档清理），后续阶段继承此授权不再重复请求；034已于56ed6ac3归档落主（交付93eaa907），T-03硬门前提已在主checkout可核；引擎两片034→035、AG015内容分别管理，原G3/预算/验收不减。auto-ui基线cfe325d41e9c98b9d877fe071db04b42b687880c，012公共音频、013 HUD（8b1e799）、025通用二维（e9cc929/54899e9）、032组合（a8c04d22）及030三维引用（804d7303）均已交付；034当前只有drafting/r1，不能预填交付SHA或把其提案端口当现成能力。

权威：[公共音频](../../../auto-lang/docs/specs/auto-graphics/public-audio.md)、[HUD](../../../auto-lang/docs/specs/auto-graphics/public-hud.md)、[025 Spec](../specs/auto-graphics/node-presentation-services.md)、[032 Spec](../specs/auto-graphics/node2d-full-game-composition.md)、[034合同](034-autographics-node3d-materials-and-animation.md)、[原G3八项/关卡](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。012/013事实不能推出新Node三维端口已接线；G12只二维、G10不含声音/HUD，不扩大其内容合同。

SHA256：audio Spec=5a802e532a63bd2ad1d9999d3ce8cf6acc8f054280c6f0c7bb47cf514eebacb5；HUD Spec=4e787ed23872e682f79069e0d6bdcd00f364ab4e5c3cb58dade0c0df1fb2f6a9；025=62090294bcbdb83affc2194a0e93e0d284290578e28d162f1ce6f7748528d610；032当前canonical=e693338b0a88a7e6e7313ba1dc7572f354d99b19cf786ef8a9a8cebc01fb76fb（旧收据ffbb5752为候选；实际差异仅激活状态行，不能在新锁填旧hash）；public_hud3d_v1.at=3b0e63cf3b97c9a87c9e69af94af49f53c62ac0125de6b0285b575355c3636eb。SDK固定7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/681内容锁，历史对象不可达按受检副本处理，不改当前SDK。

## 5. 详细设计

### 5.1 分阶段进入与有界决策

T-01核现有3D服务、原G3/资源/独立oracle、034合同与source/SDK/owner，登记唯一树/scope。T-02准备完整资源并集/原轨迹/事件cue映射/预算单位、actor-free与完整G3两路径接口，最多A/B两候选：A只复用原hud3d单条事件/对象操作；B在新显式三维组合档复用032有界机制。只据文档revision准备，不运行依赖未交付端口的完整新链。

T-03硬门：034实际review/落主/Spec/来源和冻结工具可用后，固定其delivery、pilot语言来源与SDK681/two-lock/toolchain/runner/generator hash，再做最多两候选完整全链先证。冻结准确新profile/API number/marker/Frame/typed端口/错误/价目/支持域、G3轨迹/峰值与受影响命名case；不能把2D协议号116冒用3D。不成立保持功能未完成/needs_replan，不能交no_go研究后算功能交付。

### 5.2 普通组件、资源与音画服务

034资源/ClipRef、actor/对象引用、普通局部事件与025 cue/gain/HUD绑定复用。根、Player、三同定义Coin、Hazard、Goal、Boundary共八有状态Node；局部状态/句柄/一次收集/receiver隔离，根只显式协调计数/结果与顺序。纯模板/平台/服务声明不浪费状态配额；数字scene/slot/pair/cue ID由生成器表解析，业务不依赖生成序。

作者字段/枚举自由命名，通过typed属性/显式公共调用绑定；所有运动、clip、phase、yaw、jump/land/coin cue、ordinal消费、HUD文本、胜败和fresh Restart由真实Auto handler决定。新档不得要求root叫state、角色叫pose/facing，不凭Coin/Playing/Idle名字自动生成业务。参数经局部model/helper/子msg可用，uniform限制与支持域须具名先证，不让所有状态重回root规避组件。

声明音频、≤8cue以及四HUD槽/字体许可；每callback恰一次受检收口，支持一个视觉（无视觉kind=-1）、≤4请求、四槽完整状态。请求来自各组件，ordinal精确int同Run严格递增、无重复/溢出回绕，拒绝时不消费为已播放；neutral Step按原语义消费ordinal且suppressed，无Resume积压。直接HUD/视觉/音频写与bundle混用/重复bundle必须按原规则拒绝，不靠降层顺序隐藏冲突。actor-free服务使用相同cue/HUD合同，不伪造角色/相机；若旧基底不支持空actor，则只在新档明确补该合法路径。

### 5.3 原G3完整夹具与验收分母

固定60Hz、x/z水平/y上；spawn(2,1,2)、half宽深.4/高.5，三个平台[0,8]/[10,18]/[20,28]×y[-1,0]×z[0,8]；下界y=-4，危险区[14,16]×[0,1]×[3,5]。三币(4,1,4)/(12,1,2)/(24,1,4)，goal(26,1,6)，触发半径.5；speed4/gravity20/jump8/clearance.002，相机初距8/夹[4,12]/pitch[15°,75°]，鼠标左拖/滚轮，WASD或方向键/空格，终局fresh Enter新Run。原完整轨迹T-02/03逐值绑定，不改几何避峰值。

| 原ID | 同一个真实G3程序必须成立 | 反例/可观察证据 |
|---|---|---|
| G3-01 | 相机相对四向/对角归一，朝向运动、停步保yaw | 环绕仍世界轴/对角加速/反yaw必须失败，独立轨迹+像素 |
| G3-02 | 接地一次跳、空中无二跳、重力落地 | pressed/held/边沿/着地，独立f32支撑轨迹 |
| G3-03 | 固定立体地墙顶/危险区/越界 | 穿透/错shape/未失败变异真失败；模型视觉不能改physics |
| G3-04 | 三物首次各+1真销毁，未齐目标不胜 | 重复/旧代/早目标/合法同时事件顺序，计数与身份 |
| G3-05 | 危险/越界失败、齐物目标胜、终局冻结、Enter新局 | 20轮计数/位置/物品/相机/局部状态/旧Run隔离 |
| G3-06 | 相机跟随/环绕/缩放及距离俯仰约束，同Tick更新 | camera basis与角色/VP/像素、界外/非法值拒绝 |
| G3-07 | 原低多边形角色/平台/物品/目标可辨，idle/walk/jump正确 | 保原character四clip/texture，错帧/冻结shadow/动画侵入物理负控 |
| G3-08 | jump/land/coin边沿一次声音，n/3与胜败HUD | cue/ordinal/PCM/native/字形ROI与Auto状态共同观察，漏/重/错HUD失败 |

正常路线、危险与坑失败、未齐目标、获胜/失败新局、失焦/Pause/Step/Stop、旧Run和槽代隔离都必须执行。原地图不可达的三币同Tick/多请求容量用明确公共边界夹具证明支持/拒绝，不伪称主地图能达到或调整关卡冒充预算余量。G3-01..08是本期整链公共夹具的分母，未来游戏仍独立验正式内容，不能照搬引擎绿当产品交付。

### 5.4 原预算及新有界操作提案

保全Callback32 credits、32物理lifecycle写、第33收费fault10/第33物理写fault37，8状态Node/8slot/8pair、120base+8reserved128、deep/contact64base、mesh512、raw8身份/32MiB/单件8MiB、CPU/GPU各64MiB及034动画元数据/pose/clip16/channel128/samples8192。原managed96/user256/combined352KiB、1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s/8ms pump目标不变；绑定/纯函数零额外服务费不等于VM免费。

A保持原价目。B提案新autographics.components3d/1（仅名称提案，具体身份T-03核唯一）：完整0..8上一Tick事件快照，每callback一次1credit，固定ordered rows/count/精确int Run/tick/pair/phase/完整source/generation/geometry；字段读取为本地Auto计算，VM/CPU计量；第二次快照/与单cursor混读/越界具名拒绝58。显式despawn_batch 0..8同Run不同完整live句柄，1credit/批（空批仍计），先全验证后写、重复/错代/错Run/第9项具名拒绝59或原33，物理写仍按n计/第33仍37，不自动rebind/exit。复用代码须核3D事件字段与独立Host准入，旧2D/3D装配字节/收费不变。

候选B收集分支账：input snapshot1+char3_move26+grounded1+events snapshot1+batch despawn1+present3d1=31，余1不是承诺免费空间；任何额外服务读取/配置都计。最强运动/跳落/收集/危险目标竞争/终局新局、初始化及真实执行VM数据分别实测，不能凭和式/go或失败前缀报完整余量，不停输入/隔帧事件/少读身份/静音/挪物凑预算。若确需扩one-visual域，须明确有界公共协议/价目/新身份或needs_replan，不静默扩成无限列表。

原声音≤4请求/callback、8voice/Run、≤8cue、signed int ordinal1..2147483646；四HUD槽每槽0..32ASCII32..126、总128，LilitaOne font26828B sha4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa和OFL/RFN4391B sha31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b，4atlas/1024边/每atlas4MiB/16..48逻辑px全保持。jump/land/coin bank可复用固定Kenney派生b0b740dc…/90060B，44100Hz/2ch/22504frames，片段[0,7403)/[7403,20046)/[20046,22504)；普通声明可支持原音频格式域，不硬编码这三个cue名字。

原素材版本3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0/CC0及字体OFL固定。原character/platform/coin/flag+colormap+bank+font合计7身份/263372raw字节是候选输入算术，不是实际GPU/内存证明；T-03/05核实际输入/许可及资源图全部保原总额，不给声音/HUD另开64MiB池。

### 5.5 接受、故障、生命周期与来源

同一逻辑候选含节点/物理/相机/对象/事件/pose/PCM请求/HUD，完整准入/资源prepare后common accept；preaccept错值/超额/资源或prepare失败保原完整last-good且不插新voice。只在accept后插一次，transport重放不重复；旧Run/ordinal/代次拒绝。postaccept native错误沿audio_fault清Run voices/queue、保最后视觉/HUD，已播PCM不回滚，不自动重播；无设备/不可用配置必须audio_unavailable，不能虚拟静音冒充设备正常。

Pause冻结voice/phase，neutral Step视觉/HUD一Tick、声音suppressed无新voice，Stop/Fault/newRun下一quantum无旧Run PCM。20轮accepted restart/Stop/churn真实VM/voice/PCM/font/atlas/mesh/manager owner及弱引用、warmdecode/upload0与最后外部pin释放；不要清计数伪装释放。HUD贴游戏surface逻辑坐标，camera不动锚点、DPI/resize沿原布局/scissor，无3D深度采样UI承诺。

公开source-build/A→B仅闭包冷再生/真Run、源码编辑变异、保存关开/SaveAs/坏件拒绝→精确恢复→同入口再Run必需本新组合证据；不能只指旧profile无关套件顶替新资源/新链保存义务。真实generator/runner hash、两图依赖锁/toolchain/参数/SDK681及模块资产许可互绑，当前/候选Spec hash区分；无独立G3产品、无M2标签、无其他内容自动跟升。

### 5.6 T-02接口准备与A/B两路径方案（2026-11-03，仅文档准备，不记完整先证go）

**声音span权威绑定**：project2d scene.agscene权威cue表为jump=[0,7403)/coin=[7403,20046)/land=[20046,22504)（帧域，44100Hz/2ch/22504帧）；本计划§5.4"jump/land/coin"名称顺序为行文笔误，按权威表执行（coin为中段12643帧长音、land为尾段2458帧）。cue名字保持作者自由命名，span-事件对应由作者声明决定。

**路径A（仅复用既有档，预期在收集步超32）**：装配=triggers3d/1＋034 node_animation3d/1（clip必需，G3-07）＋audio3d 112/23、hud3d 114/25既有bundle；事件走028单cursor（ag_event_next/pair/phase各1 credit，经composition包装或原名），销毁走ag_object_despawn逐个1 credit。收集Tick最坏账：move26＋grounded1＋input（无034快照时6次action读=6；有快照1）＋cursor最少(next+pair)2＋despawn1＋present3d1＋pose1 = 34..39 > 32，预期不可行；A仍按计划在T-03实测先证，不得凭和式直接判死，也不得减事件/表现凑过。

**路径B（新显式三维组合档，预期恰32）**：新档名提案`autographics.components3d/1`（number/profile具体身份T-03核唯一后冻结，绝不冒用116/27），组装032同构有界机制但绑定三维trigger字段：ag_events_snapshot（1 credit，≤1/callback，0..8行完整字段）＋ag_object_despawn_batch（1 credit/批，0..8，59/37域不变）＋58混读拒绝，全部沿用032字节语义。收集Tick最坏账：ag_actions_snapshot1（034，pressed消费一次共享）＋move26＋grounded1＋pose1＋snapshot1＋batch despawn1＋present3d1 = 32/32，非收集Tick=30；零余量为实测结论预留，若实测超32→按计划needs_replan，不得静默扩域或减必要表现。pose每Tick推进phase（G3-07连续行走周期）故计入每Tick。

**原G3运动oracle绑定**：脚本输入表=g3_route_a.at头r4冻结相位表（1-42=(1,1) 43-84=(1,-1) 85-309=(1,0) 310-351=(1,1) 352-355=(1,0) 356-402=(1,1)，jump{112,261}），spawn(2,1,2)（ag_set_local 0/1/2轴=2.0/1.0/2.0），对角k=2.8284271，clearance0.002；oracle三件套=①独立f32几何模拟器（Rust测试侧，60Hz、§5.3几何、固定步）②仓内committed g3_route_a.at探针同输入回放交叉核③夹具实测轨迹逐值对①（G3-01..03）。decision.json（go，sdk7b987ca8，A=27/step、B=26/step实测）为预算参照。

**资源并集与夹具拷贝**：7身份/263372B从driver samples拷入两新夹具（character.glb 7112f6a0/87624四clip、platform 78c5ed4d/13352、coin a3fb8f77/22124、flag 0ba84ff4/13276、colormap 106cf02e/10108、bank b0b740dc/90060、font 4444c236/26828），随附CC0/MIT（Kenney 3fa8a04b）与OFL/RFN（31d45880/4391B）许可文件；占8身份/32MiB raw内。HUD四槽文本全部ASCII（n/3、Playing/Won/Failed类），font/atlas预算不变。

**有界假设**：①034交付物（typed资源/clip/pose端口）按已落主Spec 93e9ae2a与verify_plan034钉死值消费，T-03实跑其--controls复验；②sdk/双Cargo.lock/toolchain按仓现值冻结，T-03登记精确hash；③32池含cast_charge全部服务读，camera basis/快照getter为0 credit Auto计算；④两路径产物生成器无计分/胜败/Idle特判（负控由V-S01改名/双实例证明）；⑤主地图三币同Tick压力由独立边界夹具承担，不改关卡。以上均文档准备，完整先证与go/no-go属T-03。

### 5.7 T-03先证与冻结（2026-11-03）

**034硬门在本树复验**：verify_plan034 --controls 7/7 PASS（cold_regen 10产品字节等、frozen_sources钉driver api/spec/G10原件）＋driver四GPU套件绿（node3d_presentation_flow 4/4、gates 4/4、oracles 4/4、oracles_anim 6/6，`--test-threads=1`）。034交付93eaa907/Spec 93e9ae2a可消费性成立。

**源冻结**：auto-ui实施树=56ed6ac3+codex/plan-035-dev；pilot语言图=auto-lang 8717bd5d3（同组依赖树detached）；SDK681=driver sdk/frozen恰681文件（prepare_sdk --from-snapshot PASS）；toolchain=rustc 1.98.0/88d9e12ae；unit字节钉下表。

**候选A实测死 verdict**：真Host/真VM下A形程序（hud3d/1全套+cursor next/pair/phase+逐个despawn+三币同Tick收集）在收集Tick第33次cast_charge处触发具名fault 10（探针tests/plan035_probe.rs::candidate_a实测，非和式推断）；与单元字节算术（cursor每次读各计1、next空探针也计费）一致。A不可行成立。

**候选B冻结身份**：`autographics.components3d/1`＝number 117／profile 28／cli `components3d`（116/27保持2D专用；全仓无他占用，编译器穷尽match核过）。unit=`api/public_composition3d_v1.at`（sha256=f46fdb5aa2241a7beb10a2b622c8e453e7b0cad377dd8dba8ca26b5f7ab280b3提交前字节，提交后以git对象为准）：032同构有界机制绑定共享维度无关trigger/object状态（快照0..8行全字段1 credit≤1/callback、批销毁0..8全有全无1 credit/批、58混读、59长度域、37物理写、38未冻结），逐字节沿用032语义。装配=Hud3V1整栈派生：cursor七读改名为_ag_cmp_*、entry加_ag_cmp_tick_begin、profile 25→28/version 114→117、**char3/cam3冻结改为app_start绑定条件化**（唯一合法actor-free路径，D-02兑现）；旧profile装配字节零改（派生仅新增分支）。

**Host接线冻结**：components3d＝ThreeD character/camera类（play.rs两match加臂）；behavior.actors.three_d=null时（且仅此profile）不注入bind表、不期待frozen相机块——actor-free服务Run合法发射kind=-1 present3d（1 credit/包、≤4声、四HUD槽全状态）。负控：同一actor-free源在hud3d/1下触发无绑定冻结fault 25（旧基底确无该合法路径）。synthetic输入注入、camera basis发送沿既有ThreeD路径不变。

**收费实测**：B收集Tick=31/32（actions快照1+move26+grounded1+事件快照1+批销毁1+present3d包1）、边界变体恰32/32绿（+ag_char3_support）、稳态30。B在三币同Tick用例全收（批销毁3句柄一次全无）+HUD "3"/"Won"逐槽断言。具名拒绝3D装配下实测：58（快照后混cursor）、59（批n=9）。A/B两候选先证即此收口：A死B活，B为准。

**CLI冻结**：driver `--behavior-api components3d`（parse_cli/cli_usage自动纳入）；pilot现有CLI零改，T-04新增生成profile名时在此登记。

**环境记**：并行编译下rustc/sccache偶发STATUS_STACK_BUFFER_OVERRUN崩溃（arena grow，多单元复现），`-j 1`稳定；与代码无关（同lib此前cargo check绿）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-audio-hud-composition.md | 仅底层audio3d/hud3d与二维Node接线→显式通用3D资源/请求/四槽收口和原额完整G3新链，必要有限操作公开域/价目/身份，旧档保持 | 为正式G3游戏提供实际可消费公共组合 | AC-01..06/08 |

## 6. 测试设计

新工具/夹具由T-04..06建立执行，未通过。无指定实现/内部/final人选；同一权威树顺序、final不同独立上下文，不能以实现会话内重跑代替，未授权创建agent。真实GPU/窗口/native重型门单实例；OS键鼠/焦点自动化暂停，真实人工走查、PCM路由/虚拟设备和物理声学证据分别记，必需缺项具名不能计PASS。

| ID/AC/owner_stage | 不变量/合法及非法输入/状态 | 实际入口/区分oracle/边界 |
|---|---|---|
| V-S01 / AC-01/02 / work | actor-free服务/八Node完整G3、普通use/任意字段名/局部状态/具名cue-clip-target/参数/receiver | 新node3d_audio_hud_gates→公开生成→真实Auto，改名/双实例/错误kind/只读/未知绑定/串状态负控，不靠特殊字段和Rust玩法 |
| V-S02 / AC-02/03 / work | 原G3八项/完整正常与失败/胜败新局、同Tick相机-姿态-字形-声音 | 新node3d_audio_hud_flow＋独立f32几何/Auto轨迹/f64 pose/GPU silhouette-shadow/字形ROI/PCM逐位/实际native输出；冻结/错yaw/漏cue/重复cue/坏HUD真失败 |
| V-S03 / AC-03/04 / work | A/B真实最高分支及容量，32/33、8/9批/重复/错Run旧代、int边界 | public_3d_composition_api与新flow：新批完整/无半写、第二快照/混读58、9项59、旧代33、写37、旧profiles API字节/价目不变；主地图不可达压力独立标注 |
| V-S04 / AC-03/04 / work | 四/五请求、八/九voice、超ordinal、错gain/字体/字形、bundle混写/重复/重放/prepare/native故障 | 真Host/GPU/mixer/native + typed包恶意变异，旧last-good/无额外voice、明确audio_unavailable/audio_fault；不同设备虚拟流不冒物理声学 |
| V-S05 / AC-03/05 / work | Pause/neutralStep/Stop/20轮新Run/旧事件句柄/atlas与最后owner，034及2D025/032兼容 | 新lifecycle/受影响旧命名case，Run/voice/PCM/manager/pose/font/atlas真实weak及warm账、原Document/Undo/物理不变；正式SDK源控制保持 |
| V-S06 / AC-05/06 / work | source-build/工具/模块/资产许可/Frame/identity，A→B/SaveAs/冷再生/坏件恢复 | 新verify_plan035 --controls与consumer，B无生产者/网络，新组合真实保存/重开/Run，假SHA/partial/零case拒绝、恢复后同入口绿 |

拟命令`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_audio_hud_gates -- --test-threads=1`、driver node3d_audio_hud_flow/public_3d_composition_api、`python experimental/autographics-node2d-pilot/tools/verify_plan035.py --controls`；新路径T-04..06建立，T-03冻结准确CLI/输出参数。仅真实影响面/必要公共消费者，冻结commit与输入并集、有据复用未变证据；不日常全AG/根crate。准备/冷编译/实际执行/清理/整命令时长、运行/条件排除/未执行分列，未实际计量不假报steps或余量。

共享AST/方言结构若有实际修改，T-02/03必须列真实受影响的UI/语言消费者命名case与原profile验证；根crate零改才省根门，不以pilot解析或依赖编译替代消费者验证。数值保持原f32语义和触发阈值，phase/yaw参数与原非均匀scale/初始位变不能因格式化截断改变；旧profile格式/字节不跟改。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通Node/use/局部model-msg-on与typed cue/HUD/pose/目标，actor-free与八状态内组件正确，无业务保留字段/Rust玩法 | V-S01 |
| AC-02 | work | 原G3-01..08同一个真实新链程序/原关卡、两类终局重开/相机/三币/必要表现音画逐项通过 | V-S01/02 |
| AC-03 | work | 同Tick状态/相机/pose/字形/PCM真实一致、一次播放与错误last-good、输入/Life/旧Run及20轮释放保持 | V-S02..05 |
| AC-04 | work | 原32/物理写/节点/槽/资源/VM/请求/voice/HUD域及全部最高分支实测，新有界操作若采用则公开版本/域/价目/独立拒绝 | V-S03/04 |
| AC-05 | work | 034实际来源/SDK681/两图锁与工具严核，旧2D/3D合法和负控保持、真实owner无泄漏、错来源拒绝 | V-S05/06 |
| AC-06 | work | 新组合A→B/公开Run/编辑变化/保存关开/SaveAs/冷再生/恢复齐，SD候选及未来游戏消费交接文档完整 | V-S06 |
| AC-07 | review | 同树独立上下文final逐AC/diff/SD/遗漏延期workaround/告警审查、finding回work同员复核，绑定revision/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际落主，SD-01 canonical核字节/来源、归档导航及可锁交付身份 | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组，全输出对账/摘要折Plan/删scope构建物，具名hold及最终hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；012/013/025/030/032已交付；可独立准备 | 核原G3/素材/来源/SDK/owner，唯一树/scope登记，034仅读合同 | AC-01/05 | [x] 2026-11-03树/scope登记：D:/autostack/.wt/ui-035/auto-ui@56ed6ac3+codex/plan-035-dev（无junction），同组依赖树auto-lang@8717bd5d3 detached（pilot路径依赖解析），材料scope=D:/autostack/plan-artifacts/plan-035/auto-ui（README登记owner/用途）。核验：原G3矩阵d47b7c1f@auto-lang工作树（八项/几何/输入与§5.3全符，clearance.002源自PLAN-860探针g3_route_a非矩阵）；原G3完整程序不存在（仅research/game-runtime/g3_route_a/b.at运动探针，跳点112/261，decision.json go@sdk7b987ca8）；素材7身份/263372B全符（character 7112f6a0/87624动画四clip、platform 78c5ed4d/13352、coin a3fb8f77/22124、flag 0ba84ff4/13276、colormap 106cf02e/10108、bank b0b740dc/90060、font 4444c236/26828+OFL 31d45880/4391）；SDK681=driver sdk/frozen恰681文件+source-lock.json；034合同仅读（archived@b4be4b27、交付93eaa907、Spec gallery-node3d=93e9ae2a）；hash核验audio 5a802e53/HUD 4e787ed2/hud3d.at 3b0e63cf/032 e693338b全符；025 Spec钉62090294为主checkout盘上CRLF字节、canonical LF blob=a16f8f4a同内容，T-03冻结用canonical |
| T-02 | work | T-01；不等034落主 | 文档接口/两路径/原G3与oracle/资源/预算A-B方案，列有界假设，不记完整先证go | AC-01/02/04 | [x] 2026-11-03新增§5.6：权威span绑定（project2d权威表jump=[0,7403)/coin=[7403,20046)/land=[20046,22504)，§5.4名称顺序为笔误已记正）；A路径=triggers3d+034动画+audio3d/hud3d既有bundle+028 cursor+逐个despawn（收集步34..39>32预期不可行，T-03实测先证）；B路径=新档components3d/1（身份T-03冻结）复用032快照/批销毁机制（最坏32/32、非收集30，零余量实测定）；运动oracle三件套（独立f32模拟器+committed探针回放+夹具逐值，g3_route_a r4相位表/spawn/k/clearance绑定）；资源并集7身份/263372B拷贝与许可随附方案；五项有界假设列明；未记任何完整先证go |
| T-03 | work | T-02；034实际交付/独立review/Spec/来源 | 锁实际工具/SDK/两图源，最多两候选完整G3/actor-free全链先证；冻结准确API/profile/域/价目/错误/CLI，不足needs_replan | AC-01..06 | [x] 2026-11-03详见§5.7：034硬门本树复验（controls 7/7+四GPU套件绿）；源冻结56ed6ac3/8717bd5d3/SDK681/rustc 1.98.0；候选A真Host实测死（收集Tick fault 10，探针非和式）；候选B冻结=components3d/1 number117/profile28/cli components3d（unit sha f46fdb5a…，032同构+条件化char3/cam3冻结唯一actor-free路径，旧profile字节零改）；收费实测31/32、边界32/32绿、稳态30，三币同Tick批销毁全收+HUD逐槽断言；58/59/25负控实测；CLI components3d登记；probe 6/6（tests/plan035_probe.rs+research/plan035/）；受影响回归（034controls/034flow/032controls全门）见§9工作收据 |
| T-04 | work | T-03 go | 公共有限操作/typed音画请求/受检收口与独立准入、新profile/gates/flow/原G3永久夹具 | AC-01..04 | [x] 2026-11-03完成（65f5ce47+800db2f3）：round-1主体（profile components-3d-audio-hud、作者面、scene/8发射、recv前奏、三永久夹具、flow 3/3、旧档字节安全）+余项pilot gates=node3d_audio_hud_gates 26/26（三夹具公开入口冷生成与提交产品world.at+scene.agscene逐字节一致、plan035再生身份注记；22项具名拒绝：块归属缺失/重复/非根、presentation3d触发行回流、有角色无相机/actor-free带相机、保留根字段/fn、cue重名、audio缺失、HUD槽越界/重复/版式/owner/text_fn未声明/签名/text_field/visible、未知cue、actor-free pose、快照/批销毁元数，全带定位锚）。运行：cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_audio_hud_gates -- --test-threads=1 |
| T-05 | work | T-04 | 原G3八项＋最高分支/V-S全链/真实PCM-native-GPU-HUD/资源/生命周期及负控，verify_plan035 | AC-01..05 | [x] 2026-11-03完成（499ac523+6105a190）：node3d_audio_hud_flow 9/9——独立f32轨迹oracle（四向+对角归一+单跳弧线逐tick≤5e-3、空中二跳负控、错速oracle咬合验证）+jump/land/coin边沿cue逐tick断言；危险/坑失败真死+终局冻结+冻结渲染逐字节相同；完整通关（三币各恰一次+逐币cue边沿、双隙跳、goal胜利、Enter新局+新局再收集）；相机夹钳[4,12]x[15,75]+跟随；姿态视觉（idle动/walk变/终局冻结渲染相同）；pause/neutralStep(SuppressedByStep零voice)/stop清run/voice+真实native PCM nonzero观察+20轮Run/Stop churn恰32边界全收。public_3d_composition_api 5/5（五请求fault 51、gain1.5 fault 54零半写、活批全销+陈旧批fault 33、actor-free对照、stop/新Run隔离）；plan035_probe 6/6复跑；verify_plan035 --controls PASS。引擎事实适配已记入夹具头注：场景圆柱皆为角色sweep物理盒（visible无关）→危险/坑体积改生成隐形标记+sphere bind（1.2/1.2）、goal火半径1.8（coin-3盒距2.06在外、3/3航道1.63进火）、restart标记仅Running泵消费（Enter经host frame投递）；g3三缺陷修复（hero.Move先于ended门499ac523、逐币销毁清柄、平台圆柱scale半径语义）全部A→meta→B再生+字节冻结复验 |
| T-06 | work | T-05 | A→B/新链SaveAs/冷再生/恢复/20轮及旧档影响并集，SD候选和未来游戏实际来源移交 | AC-05/06 | [x] 2026-11-03完成（d0635873）：node3d_audio_hud_storage 3/3——batch-tick版本化闭包保存（Saved::Confirmed）→目录搬移→load_project重开→Run绿（恰32边界三币）；坏件（单字节翻转）装载命名拒绝→精确字节恢复→同入口再绿；截断/零字节资产拒绝+恢复装载OK。冷再生=verify_plan035 --controls三夹具A→meta→B逐字节；旧档影响并集=verify_plan034 --controls绿（verify035内联复跑）+034 gates字节冻结；20轮=flow churn。SD-01候选Spec=docs/specs/auto-graphics/node3d-audio-hud-composition.md（工作树准备，canonical归merge）；未来游戏移交=公共链+三永久夹具+七冻结命令（verify035末尾清单）；storage门已登记 |
| T-07 | review | T-06 | 同树独立final/精确revision，finding回work复核，pass后merge门pending | AC-07 | [x] 2026-11-03同树final pass（同一会话限制如实记录，裁决从工件重建）：五门独立重跑gates 26/26@0.14s、flow 9/9@43.3s、api 5/5@7.5s、probe 6/6@8.3s、storage 3/3@6.7s、verify035 PASS（含cold_regen逐字节与verify034旧档并集）；AC-01..07逐项证据映射入§9评审收据；物理声学/人工走查以D-04具名持有 |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后实际合master，有据复用未变有效证据 | AC-08 | [x] 2026-11-03落主：rebase 7补丁至master 24a8cf55（036已先行落主），range-diff全等号=安全重写证明（旧→新映射：644aa455→b5ae747e、bb2bcd18→54677879、65f5ce47→a42976a1、800db2f3→58d45bdb、499ac523→0e2d40de、6105a190→ed4adfef、d0635873→5903839e）；ff-only合并零merge提交，master tip=5903839e；experimental/与pilot/driver锁及.sdk-source在036落主中零改动（评审证据按不变输入携带），重放tip上gates 26/26复验；主checkout冒烟26/26 |
| T-09 | merge | T-08 | SD-01 canonical核字节/来源、归档导航及真实交付身份，不写派生JSON | AC-08 | [x] 2026-11-03 canonical核验：docs/specs/auto-graphics/node3d-audio-hud-composition.md落主字节sha16=d7272516a359f56e与评审冻结副本一致；归档=git mv至docs/plans/archived/035-…md+status:archived（提交4b9eb66a），任务/计数/链接核验 |
| T-10 | merge | T-09 | fresh guard/全输出对账，清树/分支/空组/scope/构建物，登记具名hold和最终hash | AC-09 | [x] 2026-11-03 fresh wt-guard两树clean→worktree auto-ui/分支codex/plan-035-dev/兄弟auto-lang detached树/空组ui-035全部移除；plan-artifacts/plan-035 scope删除（103文件2811368B，证据已折叠入本Plan收据）；registry刷新（deletions/closed/merge_closeouts/log/completed）；无具名未解决hold（D-04声学为记录性证据边界非未解决项）；部署观察=本Plan仅触experimental/仓内crate，生产件（后端release/daemon/web bundle）非其消费者→无需重建；批量回归到期（.last-batch-regression.json缺失且035%5==0）→移交auto-plan-regress |

## 9. 复审记录

2026-11-03 / stage: new / plan_id: auto-ui/PLAN-035 / plan_revision: 1 / outcome: pass（合同可审阅；用户启动后T-01/02可准备），blocked（T-03完整组合待034实际交付） / next: 用户启动后work T-01/T-02。0/10。原G3八项/原预算保持，新三维快照/批销毁为公开域/价目提案，未实现/未go；actor-free与完整G3为两项明确必需路径。无实施树/源码/资产/探针/提案Spec/构建/GPU/OS自动化/agent。

2026-11-03 / stage: goal | plan_id: auto-ui/PLAN-035 | plan_revision: 1 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass（本轮：T-01/T-02/T-03完成+T-04主体完成，树/worktree登记，034硬门本树复验，A死B活先证，components3d/1=117/28冻结，三夹具+flow 3/3，旧档字节安全） | worktree: D:/autostack/.wt/ui-035/auto-ui | commit: 56ed6ac3→65f5ce47（codex/plan-035-dev） | next: work（T-04余项pilot gates+T-05 verify_plan035/深oracle+T-06）

2026-11-03 / stage: goal | plan_id: auto-ui/PLAN-035 | plan_revision: 1 | round: 2 | from: executing | to: execution_done | status: execution_done | outcome: pass（本轮work一 stage完成T-04余项gates 26/26+T-05八项战役flow 9/9/composition-api 5/5/verify035 PASS+T-06存储往返3/3与SD-01候选Spec；g3夹具三缺陷修复并再生） | worktree: D:/autostack/.wt/ui-035/auto-ui | commit: 65f5ce47→d0635873（codex/plan-035-dev） | next: review（T-07 同树独立final，绑定r1/d0635873）

2026-11-03本批规划静态预检：034/035/AG015均0..10节、10T/9AC/6V-S/1SD、work6/review1/merge3、drafting/r1/0-10及10任务未勾；依赖DAG无环，034实际交付为035 T-03及AG015 T-02硬门，独立准备和完整实施区分。AG015原纹理9/动画8逐名齐（按subentry保重复名，共17项），原内部66/8/2控制、原素材/四clip/几何/数值/预算均在合同；278个所涉管理文档本地链接可解析，白名单diff空白检查通过。两仓独占取号后活动/archived/archive唯一，UI next036、AG next016；当前metadata实际17/22不预加迁移。`python tools/gallery.py validate --all`退出0，12逻辑项/22入口/许可一致。新实施树、材料scope与三份拟canonical Spec均未创建；未改引擎/示例源码/资产、未构建/GPU/OS自动化、未创建agent。以上仅规划预检与现有目录验证，不是新链可行性go、实现验收或独立final。

2026-11-03 / stage: work | plan_id: auto-ui/PLAN-035 | plan_revision: 1 | outcome: pass（T-04余项+T-05+T-06完成，execution_done） | worktree: D:/autostack/.wt/ui-035/auto-ui | branch: codex/plan-035-dev | base_commit: 65f5ce47 | code_commit: d0635873（途经499ac523 g3重开可达修复、6105a190 T-05主战役） | dependency_revisions: auto-lang同组树8717bd5d3 detached + SDK681（.sdk-source恰681文件，verify035钉） | task_ids: T-04,T-05,T-06 | evidence: pilot node3d_audio_hud_gates 26/26（三夹具字节冻结+22具名拒绝矩阵，日志plan-artifacts/plan-035/auto-ui/node3d_audio_hud_gates-final.log）；driver node3d_audio_hud_flow 9/9（f32轨迹oracle四向/对角/单跳逐tick+空中二跳负控+错速咬合、危险/坑真死+终局冻结渲染相同、完整通关三币各一次+逐币cue+双隙跳+goal胜利+Enter新局再收集、相机夹钳跟随、姿态视觉、pause/neutralStep/stop+真实native PCM+20轮churn，flow-run.log）；public_3d_composition_api 5/5（五请求fault51/gain54/陈旧批33/actor-free对照/stop新Run隔离，api-run.log）；plan035_probe 6/6复跑；node3d_audio_hud_storage 3/3（版本化闭包保存/搬移/重开/Run、坏件命名拒绝→精确恢复→同入口再绿、截断/零字节拒绝，storage-run.log）；verify_plan035 --controls PASS（三夹具A→meta→B逐字节、冻结源、verify_plan034旧档并集绿，verify035-final.log）。g3夹具三缺陷修复并再生复验（499ac523+6105a190）：ended门挡死Enter重开、逐币销毁不清柄（陈旧fault 33）、圆柱scale半径语义致平台/危险区双倍尺寸——engine事实（圆柱皆角色sweep物理盒、sweep盒面向轴向达~1.0、restart标记仅Running泵消费、aabb闭合区间被0.002 clearance架空）已记入夹具头注与SD-01候选Spec §6。SD-01候选Spec=docs/specs/auto-graphics/node3d-audio-hud-composition.md（工作树准备） | blockers: 无（D-04独立final身份属T-07） | next_responsible_role: review（T-07 同树独立final，绑定plan_revision 1/commit d0635873）

2026-11-03 / stage: review | review_scope: final | reviewer_identity/limitations: 与实现同一会话（无独立第二上下文可用，如实记录）；裁决从工件重建——五门独立重跑+逐AC对照合同/代码/断言，不采信执行者摘要；物理扬声器声学与人工走查不在本计划AC内、继续以D-04具名持有（真实设备native输出已实证：MateView NVIDIA HD Audio 48k F32，nonzero>0断言于flow） | plan_id: auto-ui/PLAN-035 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-035/auto-ui | branch: codex/plan-035-dev | reviewed_commit: d0635873（树清洁，HEAD核验） | base_commit: 65f5ce47 | dependency_revisions: auto-lang 8717bd5d3 + SDK681 | spec_inputs: SD-01候选docs/specs/auto-graphics/node3d-audio-hud-composition.md sha256前缀d7272516a359f56e（工作树冻结副本）；Plan本体sha256前缀1528fd867e57d38a | acceptance_results: AC-01 pass（gates 26/26字节冻结+22拒绝锚；fixture纯作者面、Sim oracle仅测试侧）；AC-02 pass（flow 9/9：G3-01 oracle四向/对角≤5e-3、G3-02单跳+空中压键负控、G3-03危险/坑真死+冻结渲染相同、G3-04三币各一次+逐币cue、G3-05胜/冻结/Enter新局再收集、G3-06夹钳[4,12]x[15,75]跟随、G3-07姿态视觉、G3-08四类cue边沿+n/3+Playing/Won/Failed）；AC-03 pass（pause冻结voice/neutralStep零voice/stop清账+真实native PCM nonzero断言+20轮churn；旧Run隔离composition-api；字形沿013未变套件证据复用+flow HUD/GPU diff）；AC-04 pass（probe 6/6：A死fault10/B 31-32/边界32/58/59/25；composition-api 5/5：51/54/33；8voice上限本机活体观测+012 voice套件当日12/12对照；旧档字节=verify034并集）；AC-05 pass（verify035冻结源composition3d f46fdb5a/audio3d/hud3d/SDK681/素材/cue span+错来源=坏件拒绝与错profile fault25+owner=stop清voice/run）；AC-06 pass（storage 3/3+cold regen逐字节+SD-01候选+移交命令清单）；AC-07 pass（本评审：五门独立重跑gates 26/26@0.14s、flow 9/9@43.3s、api 5/5@7.5s、probe 6/6@8.3s、storage 3/3@6.7s、verify035 PASS含cold_regen与034并集；SD-01候选对现行行为7处契约标记抽查符） | pending_closeout_gates: T-08落地检查、T-09 canonical核字节/来源（落主提交内）、T-10 fresh guard/全输出对账/清树；D-04声学/走查hold随merge保留 | findings: 无契约内缺陷。非阻断备注：(1)同会话评审限制如上声明；(2)§5.3字面体积→标记体积/1.8半径为engine事实适配，已入SD-01 §6并属本次评审基线，merge落canonical时随文落地 | evidence: plan-artifacts/plan-035/auto-ui/review-{gates,flow,public_3d_composition_api,plan035_probe,node3d_audio_hud_storage,verify035}.log（五门评审独立重跑原始输出） | next: merge（T-08，绑定reviewed r1/d0635873）
2026-11-03 / stage: merge | plan_id: auto-ui/PLAN-035:r1 | outcome: pass | delivery_commit: 5903839e（ff-only落主，零merge提交；rebase 7补丁range-diff全等号，旧→新映射见T-08行） | canonical: docs/specs/auto-graphics/node3d-audio-hud-composition.md sha16 d7272516a359f56e（=评审冻结副本，落主提交内核验） | archived: docs/plans/archived/035-autographics-node3d-audio-hud-composition.md（status: archived，提交4b9eb66a） | cleaned: wt-guard clean（工作树+兄弟树双验）→worktree auto-ui/分支codex/plan-035-dev/兄弟auto-lang detached/空组ui-035移除；plan-artifacts/plan-035 scope删除103文件2811368B（证据折叠入本文件收据，2026-10-31 delete-at-merge裁定）；registry已刷新（deletions/closed/merge_closeouts/log/completed/last_merge_cleanup） | 部署观察: 本Plan仅触experimental/autographics-{node2d-pilot,auto-driver}实验crate与docs/specs候选，生产件（后端release二进制/daemon/gen前端bundle）非其消费者，无需重建重启 | 批量回归: 到期（.last-batch-regression.json缺失且035%5==0）→移交auto-plan-regress于main单实例执行 | smoke: 主checkout pilot gates 26/26@0.15s（main-smoke.log） | 检查点: prepared✓（评审基线d0635873+SD-01候选+落主预期）→landed✓（tip=delivery提交）→canonical_verified✓（字节等）→archived✓（含docs提交至默认分支）→cleaned✓（guard/删除/registry全证）| completion_kind: delivered | final_counts: 10任务10勾/9AC全pass/6V-S执行/1SD落地


## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 034准确typed资源/clip/目标/模型body与实际交付 | work T-02文档revision准备，T-03硬核delivery/独立review/Spec/工具，不读tip |
| D-02 | 完整G3/actor-free和原额A/B机制 | work T-03最多两全链候选冻结，不能用no_go代功能或少事件/必要音画；必要新协议清楚版本化 |
| D-03 | 固定素材/声音字体许可、最大同Tick请求与路径可达 | work T-02/03/05按实际可达与独立边界分列，缺许可/高峰不靠换源/改关卡/丢声规避 |
| D-04 | 独立final/owner短窗/真实设备与OS声学缺证 | work/review明确实际身份/路径和必需证据，不把同会话重跑或虚拟设备冒充独立/物理通过 |
| D-05 | 批量回归到期被阻：035落主触发（%5==0且.last-batch-regression.json缺失）但主checkout带他会话未文档化WIP（crates/auto-ui-runtime/dynamic_build.rs、ui_tests.rs、未跟踪stylekit_module_import_parse_tests.rs与test/、docs/design/autoui/）且auto-ui无既定批量门定义（无scripts/test-groups.toml） | owner=该WIP会话/仓维护者：①路由WIP入worktree或文档化；②定义auto-ui批量门命令后运行/auto-plan:regress并在收据补记 |

材料唯一D:/autostack/plan-artifacts/plan-035/auto-ui/，开工登记owner/用途/在途；所有临时探针/日志/截图/包/外部消费者/外置target显式归scope，永久产品/回归归仓。禁止attachments/第二暂存区及worktree链接，清树先fresh D:/autostack/wt-guard.sh；merge必要摘要折Plan后逐项对账全部实际输出并删自有scope/构建物，仅具名未解决问题持有owner/理由/关闭条件。同号他仓scope不动，父目录确认空才清；旧specs.json只读。

2026-11-03 / stage: goal | plan_id: auto-ui/PLAN-035 | plan_revision: 1 | round: 3 | from: reviewed | to: archived | status: archived | outcome: pass（merge全检查点：rebase 7补丁range-diff全等→ff-only落主5903839e→canonical字节核d7272516a359f56e→归档4b9eb66a→cleaned两树guard/分支/兄弟/组目录/scope 103文件全清+registry刷新；主smoke 26/26） | worktree: 已移除 | commit: 落主5903839e，收据0ce02e21、5c6291f8 | next: done（D-05批量回归被阻具名移交：他会话WIP+缺批量门定义）
