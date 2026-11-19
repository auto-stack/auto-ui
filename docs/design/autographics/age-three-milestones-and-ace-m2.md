# AGE两阶段路线与ACE M2接洽

> **2026-11-17 当前路线已统一**：[三路线strategy](../../../../auto-lang/docs/design/strategy/age-m2-three-route-strategy.md)、[共同架构](../../../../auto-lang/docs/design/autographics/age-m2-three-route-architecture.md)、[首批后引擎扩展合同](age-m2-engine-extension-contract.md)、[新版AutoForge产品架构](../../../../auto-forge/docs/design/architecture.md)为当前target-state入口。本页下方原M2/M3/M4排期、旧Forge定位及计划状态为对应日期历史快照；已交范围/标签/预算/研究保留，旧快照不作为当前阻塞或完成依据。

R1复用980/981/1068/OS064/985–987：ACE M2-A真实基础→完整UI/Widgets AV→AC→A2R→真实AutoOS全量迁移/安装退役。R2复用UI058→AG023→AG029与**已交UI080**→AG030；新增UI099地图、100场景资源、101 PBR/多灯、102通用TRS动画，AV包可完整归档，981/1068后续接AC/A2R，AG031保原门并接四扩展三端。R3新版AutoForge按工程/人工编辑→Agent提案/真实预览→App/Game AV创作→AC/A2R输出，独立产品验收；不把纯静态场景编辑或旧Agent冒新产品。

新cap/Forge不增加985/A前置；原UI各波不等新cap/Forge全交，AGE与Forge最终签章不互等。UI055/059/AG022研究不重做；621项是研究目录，不是承诺本轮全部实现。此次仅规划，原执行状态/已完成任务保持，不启动/恢复实施。


> 本轮v23：A最小真实交付、B完整UI/Widgets的AV→AC→A2R三波及OS后续以§18为准。旧v22等日期快照不覆盖新阶段合同；全部原最终AGE义务保留。

> 本轮v22 Node桌面合同见§17：A交AV＋Rust FFI可复用Iced入口，B补AC原生桌面；980 r13／981 r15／1068 r2／985 r21为本轮任务合同。旧v21等状态仅作历史，不覆盖原Plan/正式收据。

> 2026-11-16 路线v21全计划A/B状态与接续见§16：ACE M2移出Godot/GDScript/TSCN目标，977仍4/10，剩六项全B不能当前归档；982仍0/37，分母缩减非新增完成；976保Vue/Ark/Web完整生产TS/JS闭包。AGE自有Node、2D/3D/光影及G2/G3原门保持。旧§15状态快照不覆盖本次目标边界，当前依赖看统一总览§17；仍只规划。

> 2026-11-16 路线v16早时状态快照见本文§15与[统一总览§11](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#11-建议的实际排程)：970/978已归档，971/A与977/PY已正式交；973/974/981和985/A在途，M2A-08未通过。979仍6/12等具名输入，980仍0/35但HELLO最小正式前置已齐；SOURCE不再卡旧的类型/推断缺口，当前重点是Node构造调用的record借用/所有权。只规划，不启动/恢复实施。

> 2026-11-15 v15历史快照（当前以§15为准）：状态/接续见本文§14及[统一总览](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md)：968/969/972/1062已归档；970为1/8、973为executing 0/10、979为6/12、981为executing 0/34且SOURCE有未交付实现；980仍0/35。979–981具名交接合同仍见§13，原源Schema/CST路线A与模块let推断沿已定方向继续；本轮只更新规划，不恢复目标或启动实施。

> 同日v14历史快照（状态已由§14更新，交接合同保留）：交接见本文§13及[统一总览§14](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#14-2026-11-15状态核对与979收口风险)：981/SOURCE承接原960多模块领域组合，980/DIALECT交完整UI字段证明，979收到正式具名接口后完整回验；不得互等整包归档。979仍executing 6/12、980/981未启动；968/972各实施中1步。只修规划，不恢复目标或AG029暂停。

> 早前同日状态快照（当前交接以§13为准）：状态与接续顺序以[统一总览§11/§14](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#11-建议的实际排程)为当前导航：967/1059已归档；972为1/10、979为6/12且有多模块等收口缺口；新增整包首选968。UI058/AG023/AG030/AG031未启动，UI080为3/10，AG029为2/10且用户暂停；G2/G3暂无可无阻整包推进的新计划。本文旧日期快照保留为历史，不覆盖原Plan最新收据。本次仅更新规划。

2026-11-13，路线v13历史快照。Node首切片/兼容规则见本文§12；2026-11-15交接与顺序由§13更新，其余状态与阶段门仍按原合同；原阶段名/归档文件/验收ID保持历史身份，不批量更名。仅规划，不授权源码、探针、素材、构建、GPU/原生输入、实施树、agent、打标或发布。

## 0. 当前基线与授权

AGE M1已于2026-11-10完成独立复审、落主、归档、清理并由用户授权签章。AG026 r1、11/11，交付9264fac、归档a008be1、清理70d6e36、签章ea0fc59；DEBT-001/002均closed，严格delivery_ready=true、milestone_ready=true、pending=[]。DPI bundle168检查、编辑350检查、基线50进程结果为该候选的既有正式证据，本轮只核来源，不重跑GUI。[当前验收规范](../../../../age-examples/docs/specs/auto-graphics/age-m1-acceptance.md)与[AG026](../../../../age-examples/docs/plans/archived/026-age-m1-dpi-native-debt-and-final-closure.md)为权威。

三个本地注释标签v0.6-AGE-M1已核对象/目标及对当前HEAD的祖先关系：auto-lang `30b396d720f5b3e12730bbd8b0a26bdb3e7f89f4`、auto-ui `4ebc51cefa2b0e97e24997407bc6a19b45caf630`、ag-examples `ea0fc594b2c85620c75b570cda370c31e18b10f0`。标签固定实际验收版本，不自动追最新HEAD；原生N4具名覆盖边界保留，旧waiver不贡献里程碑。历史ag-v0.6-m1仍是另一身份，不移动/重打/倒签。

2026-11-11用户要求：先修过时状态与981批次计数，再将国风首切片、正式3D光影、M2联合验收缺口细化为可执行合同，复用研究、不重复立项；仍只规划。已交UI055/059、AG022及M1全部归档保持不动。新合同只承接无人覆盖的产品/公共能力/独立内容/联合门，不创建另一语言/桌面迁移计划。

历史M1状态快照（当前签章状态见§12）：ACE M1的1000保历史归档；[1005](../../../../auto-lang/docs/plans/archive/1005-ace-m1-architecture-evidence-and-independent-audit-admission.md)现已11/11归档，交付05b7c9d7e为语言当前主线1169f13594祖先。签前分域facts来源/虚构commit准入缺口由[1013](../../../../auto-lang/docs/plans/archive/1013-ace-m1-phased-facts-provenance-and-tag-admission.md)承接，文档drafting r1、0/10；本地正式ACE M1 tag仍未发现，1005历史tag_ready不替当前准入，AGE M1标签也不能替代。旧1005 round15/T-01..06快照已过时，本轮不重开它或接管1013。

## 1. 阶段身份与历史映射

| 路线 | 范围与完成谓词 |
|---|---|
| AGE M1（原AG/AGE M2） | 两游戏、Gallery、编辑、混合UI+2D+3D公共基线；已按AG026签章 |
| AGE M2（原增强M3+融合M4，后合并的M2/M3） | ACE正常作者前门、AV开发、新A2R发行、完整传统UI与AGE语法、二维/三维增强、生产融合、真实OS及增强联合签章 |
| ACE M2-A | 完整语言/新库/FFI/宿主/构建与RG-A真实验证；979、980及981的SOURCE/SESSION/RUST/LANGUAGE/PROOF在A |
| ACE M2-B | M2A-08后全消费者迁移、发行、默认与旧实现退役；981的CONTENT/RUST2D/RUST3D/FUSION/AUDIT在B |

AGE不再有独立M3，ACE的A/B不为AGE增加阶段。AGE不等ACE总标签才接线，ACE完整语言不反等AGE新增图形全完；各自完成谓词不能互充。原M2-01..10/G-M01..09/AG-A/AGE-L、历史文件名与协议继续有效；AG020更名只改同仓定位，不是图形前置。

## 2. 已交研究与产品输入

| 输入 | 实际结论与使用边界 |
|---|---|
| UI055，交付255350a2 | 无地面27credits bounded-go；角色移动+查询35>32、满铺地面t=0为no-go。供UI058公共组合/非阻挡地面使用，不是国风游戏go |
| UI059 | 研究桥bounded-go；固定LightDef/MeshPipeline与CPU Lambert/遮挡/GPU选点/20轮研究Run已证，正式Light API未交。UI080复用方法/源码，不重新做可行性项目 |
| AG022，交付9a2f2c59 | r187 commit7e89746a687bb88271253698fa108b1dd59825d0、621唯一项研究表；完整同语义覆盖0。国风与光影正式范围现在由AG029/UI080/AG030接收 |
| UI067，交付21fa29e0 | 当时Text前门拒绝、标量正控，两候选no-go；979/981负责后续正式关闭，研究交付不冒产品可用 |
| UI072 / AG028 | 独立player和两Windows可玩包已交，分别5fd25be8/48b0d00c；981复用壳/包验证，旧VM玩法包不等新A2R包 |
| UI061–063 | 已归档交付3efe4538/cae1632e/96236725，均为当前主线祖先；RQ载荷/资源/执行输入合同复用，不能把旧VM/Rust接口当ACE迁移已完 |
| OS068及其它在途 | OS068仍executing，产品/RQ修复未来消费正式来源；不把分支tip、自报构建或进程存在充最终OS证据 |

研究Spec及固定字节在各新Plan §4列明；不重抓目录/模型/素材，不改归档为新能力。当前driver Session仍持AutoVM/AutoTask且使用旧Parser/create_vm_from_source，Node pilot仍旧前门；因此不能从M1可玩包推断新作者链已完成。

## 3. 当前合同与唯一职责

| owner / Plan | 当前阶段/修订/进度 | 唯一交付和下一入口 |
|---|---|---|
| [语言979](../../../../auto-lang/docs/plans/979-ace-m2-schema.md) | drafting r10，0/12 | H0最小AGE Node/text Schema前门可按现有基础先行；通用Schema完整双后端/泛型CT仍依原963/966 |
| [语言980](../../../../auto-lang/docs/plans/980-ace-m2-autoui.md) | drafting r9，0/35 | HELLO先消费F0/H0/E0/B0交AV首窗；原传统UI完整语法/组件/各模式与A2R继续，不等981终验 |
| [语言981](../../../../auto-lang/docs/plans/981-ace-m2-age.md) | drafting r12，0/34 | 十批：SOURCE/SESSION/RUST/LANGUAGE/PROOF与CONTENT/RUST2D/RUST3D/FUSION/AUDIT；AUDIT先核其余九批，十批齐交全包final |
| [语言984](../../../../auto-lang/docs/plans/984-ace-m2-os-apps.md) | drafting r7，0/25 | 生态接口/收据；OS覆盖切片供AG031，不复制OS064或等待其它非桌面产品才交AGE切片 |
| [OS064](../../../../auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md) | drafting r4，0/97 | 独占HOST/真实桌面.at、普通Node桌面根、28App及发现的新增consumer/ASSEMBLY；A仅HOST+PROOF |
| [UI058](../../plans/058-age-topdown2d-queries-and-sprites.md) | drafting r3，0/10 | 原五对象/同Tick/query/sprite＋非阻挡可见地面；T-02中立先证可先行，作者T-04等981所需A正式切片 |
| [AG023](../../../../age-examples/docs/plans/023-age-m3-topdown2d-example.md) | drafting r3，0/10 | 最低正常作者教学小例/双端源码冷包，不含完整国风游戏 |
| [AG029](../../../../age-examples/docs/plans/029-age-farming-first-playable-slice.md) | executing r1，2/10；用户暂停 | 国风纯2D小院、两田种植/生长/收获、最小存档、AV/A2R与最终包；T-01/02准备，正式实现等UI058/AG023/981/973等及M2A-08 |
| [UI080](../../plans/080-age-node3d-directional-light-and-shadow.md) | executing r1，3/10 | 中立T-01..03已有实施记录，T-05/06有中立部分；正常Node作者整链仍等979/981所需A来源，未独立复验其GPU记录 |
| [AG030](../../../../age-examples/docs/plans/030-age-directional-light-content-and-oracles.md) | drafting r1，0/10 | 原创平面/立方体光影内容、六项独立CPU/GPU oracle/同源双端/冷包；不复制UI080 |
| [AG031](../../../../age-examples/docs/plans/031-age-m2-joint-acceptance-and-signoff.md) | drafting r1，0/10 | 四线/真实OS/增强混合/来源/独立复审及正式签章；不替各owner实施，不成为上游开工门 |
| [AG020](../../../../age-examples/docs/plans/020-age-examples-repository-rename.md) | drafting r2，0/10 | 准备按原T-01/02；实际切换仍核原M2冻结版本与维护窗口，不因AGE M1签章自动更名；物理根仍ag-examples |

语言979/980/981/984及总路线等仍有本地未跟踪草案；文件存在不是已提交/交付。表中UI080已3/10、AG029已2/10且暂停；其余按原Plan当前来源核。它们已有他会话实施树/材料，不把本轮只规划误写成从未实施；本轮不操作这些树，其他会话WIP保留。

## 4. 按最小接口执行，避免循环

2026-11-11首Hello补充：按[语言路线§16](../../../../auto-lang/docs/design/strategy/ace-m2-delivery-roadmap.md#16-首个ace-ui-helloffi与最小宿主路线2026-11-11当前)，967/F0与979/H0可独立推进，F0→969/E0→978/B0，四个最小正式接口齐→980/HELLO AV真实单text窗口；原967/T-12、978/T-03/09及980/RUST同源A2R并行，AV不反等A2R。五个原Plan新增20任务，无新号；早期Hello不等全语言/新库/async/完整SDK，但也不冒完整RG-A03、传统UI、AGE语法或M2A-08。Iced是编译宿主，AV经通用受检FFI/shim交换文本与结果，不解释整套Rust库；后续完整Node/SESSION/RUST仍按981原最小接口与完整义务。


能力链：已交959/960/961→962及963/966/967等各自职责→979共同前门→981/SOURCE→SESSION与RUST分别推进；980沿自己的DIALECT/STATE/各模式推进，LANGUAGE/PROOF消费已交接口。完整依赖见各Plan，不能将这一简写当成单一串行总链。

图形链：UI055/AG022→UI058中立服务与非阻挡地面；UI059/AG022→UI080中立Light服务，两者不等981整包。最终Node接线只消费979与981/SOURCE/SESSION/RUST及所用LANGUAGE的A正式来源；AG023先教学闭环，AG029再验完整国风切片；UI080→AG030正式光影内容。AG029/030产品实施按各合同M2A-08门，准备/服务可先行，不等待ACE整个M2标签。

981的基础迁移/两原游戏/融合不依UI058/AG023/UI080/AG029/030/031。979不等980/981产品终验；980/PARITY不反等LANGUAGE；SESSION不等RUST，RUST不等AV热更新。两个游戏各自A2R可分别推进，最终V/R各需同源AV和原独立oracle。

OS064/HOST只取公共宿主与980所验模式，A/PROOF只验原桌面+Notes/Clock具名真实业务；B核普通Node桌面根/全部App/双Host/direct-RQ原格。984单向收OS064，OS覆盖切片交AG031，不把984其它非桌面产品终验塞给AGE。AG031只汇合，不被任何producer依赖；985/A不等AGE新G2/G3或AG031，B实现消费M2A-08阶段收据，不把985整Plan写成环。

阶段fold正式来源须repo/SHA、接口/grammar/ABI/SDK、源/资产与实际结果，分支reviewed/冻结tip只供准备，不替落主。公共grammar/core/host ABI/打包工具短窗单写；GPU/焦点/音频机器单实例，未来按当次授权排窗。

## 5. AGE M2四线与联合完成门

| 线 | 必需结果 |
|---|---|
| E | 正常Node直接共同前门；AV开发/编辑更新，新A2R真实生成Rust/编译，两原游戏各自最终exe+资源ZIP、全Gallery/编辑/开发包迁移；无旧Parser/VM/Trans隐式fallback |
| G2 | UI058原五对象/同Tick/32credits及可见非阻挡地面；AG023小例；AG029纯2D国风两田种收/最小存档/资源许可与最终同源双端包 |
| G3 | UI080/AG030六项G3-C01..06：正式声明、有限方向/能量、硬阴影、opaque几何法线、同源AV/A2R、资源/恢复。固定621目录只做partial/none机制映射，完整同语义0不因六能力改绿 |
| F | 980完整传统UI、981完整AGE语法/五view与生产组件/主题/Store/布局/事件/Reload/direct/RQ/输入IME捕获/设备资源；原模式承诺不减 |

不可替代的硬门：M2-UI-FULL、M2-AGE-SYNTAX、M2-OS-REAL和增强后的M2-JOINT；具体行级门在AG031 §5，不在路线再造第二验收清单。原widget/store源通过同一AGE Schema显式兼容配置保留，规范Node副本另验，纯UI必须真实AV/A2R；完整语法按AG-A01..09/AGE-L01..05与实际grammar分母，不以两个游戏通过替代。Frame主时钟/无可见出口/#if检查由981/LANGUAGE决定门关闭，不能删必需构造。

两原游戏A2R同源同资源→Checked语义→新provider→Rust→实际编译→宿主/GPU/输入/音频→最终ZIP，各自原玩法/资源/许可/中文说明/hash/仓外试玩/坏件修复/退出齐；Rust宿主内嵌AV/旧VM解释玩法不合格。复用UI072/AG028外壳/包装，不重复立同名发行。

真实OS必须原桌面业务及普通Node桌面根、各App原功能/数据/模式/窗口/launcher/dock/通知/工作区/主题配置/daemon/资源恢复齐；28是快照，不是排除其他consumer的上限。空桌面、小例、进程存在不能替原产品。AGE签章消费实际OS收据，不扩大为984所有非桌面产品前置。

AG031区分work候选就绪、独立review与实际落地/清理后的milestone_ready；先独立重建required/来源/oracle后给审计凭据，禁止自引PASS。增强混合样例复用原showcase，AV/A2R及direct/RQ适用格都验UI+国风机制+正式光影、时钟与输入/恢复/冷包。任一required未执行/unknown/waiver保持pending；计划归档或981完成不自动等于AGE M2完成。

## 6. 不减范围的预算、资产与验证纪律

UI058保原最低Player+2障碍+2同定义Target、原费用和实例/scene包络，地面不可参与阻挡；国风T-02先核最高合法Tick与存档公共服务，不能删田/停移动/隔帧query或提额。32credits、语言fuel、CPU/GPU/内存各计各账。UI080沿单2048²硬阴影/有限参数域，名义16MiB不冒实测驻留；不以旧研究计时承诺FPS或物理显存归零。

AG029原创小图集/字体与AG030原创网格必须真实hash/许可/来源，当前全未制作；不把three.js MIT推定为外部资源许可，不批量取模型/HDR。原M1锁/分母保持，候选变更遵现行Spec失效规则，旧研究可复用结论而非替新候选运行。

普通计划只跑受影响命名case/必要消费门，完整AGE必要并集在AG031唯一里程碑窗口统筹；不为每个merge重复全量GPU。原生输入与fixture、首次红与隔离重放、排除与PASS分列，缺环境不改NA。临时材料只在plan-artifacts/plan-NNN/<repo>/，无实施树的本轮文档辅助材料在系统临时目录；merge折摘要/全输出对账后清自有scope，具名hold与旧候选依赖不可擅清。

## 7. 本轮修订与后续入口

本轮修正981 r10十批34任务及AUDIT原8/9批残留；984 r7明确OS正式覆盖切片；UI058 r3新增AC-11/V-S08非阻挡可见地面并明确A接口；AG023 r3更新M1/最小接口且保小例；AG020只修正M1时机状态。新UI080/AG029/030/031各r1、0/10，分别承公共光影/首切片/独立光影内容/联合门。无重复研究或已有player/包/桌面迁移立项，归档和canonical Specs不改为未来能力。

未来明确启动时：语言962的960/961依赖已交；UI058/080 T-01→T-02中立有界决定可先行；979/980/981/AG023/029/030/031按各合同准备，真实作者接线与产品门须各最小接口/M2A-08齐。945–948仍等ACE M1-TAG，不借AGE标签放行。没有从当前规划自动转work的授权。

上轮规划静态预检（Hello追加前，保历史）：8份新建/修订合同共119任务全部0完成，四仓可达221 Plan无缺失引用/环，19份规划导航393个本地文件链接可解析；研究输入/621表保持原字节。仅文档检查，不代产品/独立实现验收。

## 8. 历史决策保留与当前优先级

2026-11-05 UI055/059与AG022研究、UI058/AG023小例立项；11-06至11-08 UI065/066/067与AG024/025/026/027/028分别处理M1修复/来源/原生债务/player与包，均按其归档实际收据保留，不重开。11-09 AGE原M2/M3合并成当前M2，ACE另拆A/B；11-10 AG026实证闭双债并签M1。旧“债务OPEN/AG026草案/M1未签/UI062在途/UI063草案”等当日观察已由本页当前收据替代，不能用旧快照重判M1未完。

原M1历史标签来源限制、N4 coverage_boundary、旧exact runner/其他具名未证边界继续沿现行Spec；新标签未抹历史。历史细节在原同号归档Plan、[M1验收规范](../../../../age-examples/docs/specs/auto-graphics/age-m1-acceptance.md)和[ACE Design55](../../../../auto-lang/docs/design/55-ace-m2-complete-language-and-product-replacement.md)保留；本轮不改任何旧验收结论为新产品通过。


## 9. 统一跟踪与分线执行（2026-11-11）

ACE M2、首UI Hello与AGE M2采用[语言总路线§17](../../../../auto-lang/docs/design/strategy/ace-m2-delivery-roadmap.md#17-ace与age统一调度分里程碑验收2026-11-11当前)统一跨仓依赖和交付前沿；本文保AGE的E/G2/G3/F与AG031联合完成视图。保留ACE M2、AGE M2两个里程碑；Hello归ACE接入线的早期消费者验证，不另立第三套同级里程碑/总控Plan。原Plan仍为唯一任务、验收、进度和证据来源。

执行分ACE基础能力、ACE接入UI/AGE、AGE图形/内容三线。967/F0、969/E0、978/B0只在基础owner实施一次；979/H0/980/HELLO消费，后续完整979/980与981按原接口扩大；UI058/UI080中立服务可按既有前置并行，作者接线等所需A正式来源，AG029/030产品实施保M2A-08门。UI/内容视图引用原任务，不重复统计Hello五包20项任务。

当前完整AGE解阻排程以§10为准：整包根962只依赖已交960/961；967/F0和979/H0为并行Hello切片，不能代完整通用Node语义。先前“先M1收口再967”是11-11排程建议，不扩大为所有M2工作树开发的硬禁令；945–948明确M1-TAG、各产品正式来源门及用户对UI080/AG029已有决定保持。M2集成分支只是待确认方案，须先修订受影响执行合同，未创建/放行；本会话仍只规划。F0→E0→B0且H0齐后验AV首窗，A2R同源腿沿原任务并行；随后全UI/Node/语言新库按真实依赖交付，不把Hello变成所有图形/语言任务的新增统一门。M2A-08后推进各原B职责/正式内容，AG031最后汇合，仍不成为上游或985/A开工前置。

总调度只维护原Plan/任务引用、合同revision、正式源/接口、已验范围/缺项、具体阻塞和下一就绪任务；状态以读回各原Plan为准，不维持第二份完成勾选。公共文件单写、每Plan每仓唯一树、GPU/输入单实例及独立review门沿原规则。ACE的985/986/987和AGE的AG031分别验收；任何一边/首窗通过均不自动关闭另一边。

本次只补跨路线排程与跟踪说明，未改实施合同/计数器、未创建新Plan或聊天/agent/监控，仍只规划。


## 10. 从共同根推进、隔离开发与正式验收（2026-11-12，当前）

完整分析见[语言路线§18](../../../../auto-lang/docs/design/strategy/ace-m2-delivery-roadmap.md#18-age阻塞的共同上游与m2隔离集成建议2026-11-12当前)。现有完整M2计划中先取962：它仅依960/961，两者已正式交付，不需再造一份“零依赖”Plan。963继承962；963后完整967可与964→965→966推进；966/963交通用979，运行/会话线交967/968/969/970，标准库/构建线交972/973/978，按981原最小批次汇合。

UI080执行回执中的979通用能力并非串行反等967/F0→969/E0→978/B0整条Hello链：979自身前置为960/963/966，CPU/投影与真实宿主/SDK消费要分清。实际双后端验证仍核所用host/build接口，缺口不能假称已交；H0仅widget/text，不足以关闭UI080 T-04正常Node/GPU整链。UI080现在3/10，中立部分已做，不再安排重复研究；它不额外等待985/A整包出口，但必须满足自己现行979/981正式来源及用户等待决定。

AG029现在2/10且用户暂停，未授权恢复；其T-03现合同还明确要求UI058/AG023与973/978/980/981实际来源、M2A-08，故只有962或Hello通过并不能直接复工。985/A出口消费完整A能力与真实RG，不是最上游可先生成的许可；AG029先于该出口完整交付须正式修订阶段/验收，不能靠取消依赖或旧VM通路绕过。

可提议本地M2集成分支接收各Plan唯一树的已审固定提交，供后继工作树基于确切SHA开发，先不进master；但当前“正式落主才消费”的合同不会因新分支自动解除。必须先在相应原Plan明确开发集成候选的允许范围、基底/依赖身份、review与失效规则，最终M1/主线/发布门仍保；945–948的tag前禁令不豁免。该方案本轮尚未采用，不创建分支/树，不改变UI080等待或AG029暂停，不接管他会话。

现场M1状态已刷新为1005归档、1013新签前修复、正式tag未见，不再沿用1005 round15为当前。当前问题同时含能力未交和现行来源/时序门，分别处理；tag出现不会自动实现979/981，master是否合入也不等于代码能否在获准固定候选中开发。


## 11. ASE M0与AGE的消费边界（2026-11-12，新增）

用户将浏览器前后台整体框架命名为 **Auto Scape Engine（ASE）**，AutoScape为浏览器产品名。新增[ASE M0路线](../../../../auto-lang/docs/design/strategy/ase-m0-delivery-roadmap.md)及语言仓1020–1023草案：基线/接口→无窗口AV→两页文字→双Counter会话。首批属于跨运行时研究，UI薄适配仍在本仓原owner实现，未创建独立ASE仓或本仓重复能力Plan。

ASE 1020/1021不需要AGE；1022消费ACE F0/H0/E0/B0/980 HELLO正式接口后验证公共frame，1023另等基本状态/按钮事件与保留会话正式接口。基础纯UI不反等981完整Node、UI080光影、AG029游戏或AG031联合签章；同样不能从AGE M1已签推断新ACE作者链、公共嵌入和会话能力已交。

M0不包含2D/3D或完整图形融合。M0后的混合视图切片再按981及公共组合/设备/输入/资源接口的实际交付进入，由原owner维护能力合同；不为ASE复制renderer/旧VM行为桥。不改980/981原完整义务、AGE M2联合门、UI080等待、AG029暂停或M2A-08要求；ASE M0既不是AGE M2签章也不成为其新增全局前置。

本轮只补消费排程导航，未改实施任务/AC、未建树或运行原生/GPU测试，未把任何接口或M0标为已交付。


## 12. AGE Node首Hello与位置敏感名称（2026-11-13，当前）

用户确认AGE直接扩展AutoUI声明/组件，首Hello采用`node Hello { view { text "Hello" {} } }`，无需先实施传统widget接入线。979/H0由AGE Schema驱动parser/结构校验及typed投影，980/HELLO复用AutoUI组件/Iced宿主接真实ACE/FFI；981复用此入口按能力扩Node、实例、状态、2D/3D和完整语言。H0是最小Node切片，不等完整981；981按已交接口进入，不等完整980模式终验。全量传统UI、AGE-L/AG-A、各模式/AV/A2R及M2A-08/B门照旧。

`widget/model/view/on`等是位置与领域上下文共同决定的node名称：只在AGE已启用、当前Schema声明的NodeName槽及父规则内识别；变量/形参/函数/字段/字符串和无AGE模块仍是普通Auto名字。Schema同时参与解析与结构校验，行为仍经共享verify；不升级成全局关键词或独立AGE parser。规则冲突、回退上下文/缓存隔离、仅改Schema即可改变实际解析的反控由979核，auto-ui提供领域规则/组件语义，不另做一份语法核心。

旧widget原源保留，通过同一AGE Schema显式legacy-widget声明头别名进入Node候选，保原字节/span/body/expected；作者只改声明头的Node迁移副本另记为迁移兼容。两种证据同一旧义务只计一次，不以Node副本冒原源直接支持，不引入源码正则改写/旧引擎fallback；原组件与业务不能为迁移而删减。兼容别名本轮不退役；987移除旧引擎不自动删除原源接受义务，改承诺须显式裁定。

963已执行；可并行979/H0-I/V/R/D（限现有函数/text、不等完整泛型/CT）与967/F0→969/E0→978/B0。共享grammar/CST/AST/resolve/query及测试注册按owner短窗串行/正式SHA交接，不改963树或消耗未审WIP。H0/F0/E0/B0正式D齐→980/HELLO，随后980/981按接口扩能力；通用Schema/完整语言仍等所需963/966。详细任务/AC/反例和来源见语言979/980/981当前§5–8及语言总路线§24。本轮只规划，不启动GPU/窗口/探针/素材/构建/新树或恢复已暂停工作。

M1状态刷新：1025已8/8归档，本地正式v0.6-ACE-M1目标b923362f47bc8861407e9a470d63ddfdb505982b；此前“1013未收口/tag未发现”是历史来源，不再作为当前等待。945–948仍先核原M1-TAG和本包实施授权；AGE M1历史与各独立产品门不变。


2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。

## 13. 979完整验收与980/981具名交付（2026-11-15，当前）

本轮按用户要求修订原[979 r11](../../../../auto-lang/docs/plans/979-ace-m2-schema.md)、[980 r10](../../../../auto-lang/docs/plans/980-ace-m2-autoui.md)、[981 r13](../../../../auto-lang/docs/plans/981-ace-m2-age.md)及Design55 §7.2，复用已有模块/Schema研究，不新增Plan、任务或实现。960已归档，待做的是模块图与AGE领域的组合，实际归981/SOURCE原B-SOURCE-I/V/R；语言owner维护公共grammar/CST/packages/verify与双后端，UI owner维护领域/节点/实例和typed描述，Core不依UI crate。

正式交付为：981/SOURCE的IF-981-MODULE（至少两文件helper/Node、实例2:1/N Tick、AV+真实rustc A2R与原span）、IF-981-REF（声明/可见性与同型正反例）、IF-981-FIELDS（完整AGE构造清单/映射/删项反控）、IF-981-ORIGINAL（g01原四图形/资源/隐藏项的原源CPU证明）；980/DIALECT交IF-980-FIELDS（完整WidgetDecl字段来源/投影/错误/覆盖检查）。清单必须可被真实机制检查，不能只交研究报告或手写HIR。每项原R复核子面后按原phase fold交正式SHA/版本/源闭包/API/Schema/compat/命令与结果；不提前勾整个批次。

顺序：`960及979基础已交 → 981/SOURCE四IF`，旁线`979基础已交 → 980/DIALECT字段IF`；输入正式交付后`979 T-03/T-08/T-04补全部原V → T-05 → T-06/T-07`。979保留V-09④/V-10/V-11/V-DETAIL-01和全部原AC，不允许以A表归档；980/981消费其已交接口，不等待其最终归档。981/LANGUAGE继承引用/完整构造，继续完整五view/生命周期/UI互操作；980全模式及实际GPU/窗口/游戏/包的原验证继续保留，CPU接面不充产品PASS。

当前968/969/972已归档，970在途；978/973等按已交实际前置接续。首ACE+AGE Hello仍在980/HELLO，只等F0/H0/E0/B0正式接口，不增加多模块、981整包或979归档门。完整A能力/真实PROOF齐后985/A→产品B；UI058/UI080仍核981适用正式A来源，AG029暂停及M2A-08门、AG030产品门保持，AG031最后联合。此次没有新增可从开工无阻完整归档的G2/G3计划，不重复已交中立研究，也不启动窗口/GPU/素材/探针/编译/实施树或恢复979目标。

## 14. 最新状态与后续顺序（2026-11-15，v15）

> 历史快照；970/978及SOURCE的类型/推断进展已由2026-11-16的§15更新。

语言主线读取基面3279f2f728f5699250a88eb94d76352ece132cb7。968/969/972/1062均已归档，正式交付在主线，对应语言工作树已不在登记中；970执行中1/8，973在核对期间转执行中0/10。979仍6/12等具名正式输入，980草案0/35，981执行中0/34（已有SOURCE代码，不是无进展）。UI058/AG023/AG030/AG031各0/10、UI080 3/10、AG029 2/10且暂停保持。

981的g01必须保持原源，经共同Schema/CST、共享绑定/检查和typed描述；已有auto-ui语义/用例用于复用和oracle，不直接桥接旧AST/旧Parser。实施树1c2fcdbd5已补age-package语法、目前仍具名拒绝模块级let推断；按A路线/选项1在原SOURCE任务补齐，不要求作者加注解/改const或降低979 V-DETAIL-01。原SOURCE四IF仍待完整独立复核与正式合入，内部绿不能冒979已收件；本次不恢复该目标。

近期闭环是981/SOURCE交四IF、980/DIALECT交IF-980-FIELDS，979收到哪项先做对应T-03/T-08回验，全部原V齐后T-04→T-05→T-06/T-07。979用已交基础先供生产，不等980/981整包；两边也不等979归档。现有970/973原树继续，尚未开工的整包候选976/978的直接前置已齐，磁盘优先倾向在途973→976→978逐包完成清理。若首窗优先，978/B0后即可980/HELLO；F0/H0/E0已交，不新增多模块、完整SOURCE或979归档门。

随后980在979与所需库/目标/SDK接口齐后完成全模式/PARITY与整包A；981的SESSION/LANGUAGE/PROOF等消费970/980/978正式能力，全部required A汇合985/A后才进入B内容/两游戏发行/融合/审计。新G2/G3作者接线仍取981正式A接口，AG029与AG030产品进入门/M2A-08保持，AG031最后联合；没有新增可无阻完整推进的G2/G3计划。Linux延期/1061/1063与独立ACE/AGE签章原口径保持。详[语言路线§32](../../../../auto-lang/docs/design/strategy/ace-m2-delivery-roadmap.md#32-最新计划状态与后续执行顺序2026-11-15r26)。

## 15. 早时状态与交接顺序（2026-11-16，v16历史；当前见§16）

语言读取基面5360b8833822b66e951c113e1de88d79ffb9afec。970会话与978构建/SDK均已归档清树；971/A及977/PY有正式阶段来源，B/其他目标仍未完。当前971 4/11、973 0/10（有核心实现）、974 1/8、977 4/16、979 6/12、981 0/34、985 4/14均在途；980 0/35、976 0/8等未开工。0计数不抹掉已有源码，但不能用源码提交或内部绿冒正式交接。

981已提交b7f5a913d对资源端口/签名、受支持let推断、场景属性与AgePackage类型面做了接入；原g01已过解析/绑定/类型检查，当前记录的未闭点为Node实例构造/隐藏值调用的record借用/所有权。node普通use不要求pub按最新用户裁定，不能将此扩大为普通Auto声明的普遍可见性变更。原树未提交探针/修改保在途；仍须完整原源CPU描述、独立正反例/恢复、四IF独立复核并正式落主，才能供979验收。旧parser/既证旧产物不替新前门，不改g01原源。

近期顺序：973继续完整系统库，974核心网络腿继续、文件腿等973；981完成SOURCE调用/所有权和原源闭合；980/DIALECT交完整字段证明，979随正式输入逐项T-03/T-08回验，再完整T-04/T-05/T-06/T-07。若新增完整核心计划优先976（前置已齐）；若首窗优先，980/HELLO的F0/H0/E0/B0已齐，可进入原I/V/R/D，不再等978或979整包/完整SOURCE。窗口实际验收仍未由本次状态刷新执行。

随后973→975/974文件腿，所需库/目标/Schema接口齐后980全模式/PARITY可整包归档；981 SESSION/RUST/LANGUAGE/PROOF消费正式970/978/980等来源，全部required A/RG齐→985/A→M2A-08后B内容/两游戏发行/融合/审计。971/B与977/REVERSE也等该出口；不能把985已经开工当它已签出口。

UI058/AG023/AG030/AG031各0/10，UI080 3/10、AG029 2/10且暂停未变，G2/G3暂无新增无阻完整候选，不重复已交中立研究。新作者接线、M2A-08产品门和独立AG031/ACE/ROS/AAC签章保持；本轮只更新路线，未推进原Plan勾选、未运行测试/GPU/探针、未创建/恢复实施或清理工作树。完整语言排程见[语言路线§33](../../../../auto-lang/docs/design/strategy/ace-m2-delivery-roadmap.md#33-最新状态与阶段执行顺序2026-11-16r27)。

## 16. 全计划A/B状态、出口与AGE接续（2026-11-16，v21当前）

当前完整ACE/AGE主线汇总见[统一总览§17](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#17-2026-11-16-aceage-m2全计划状态与ab顺序本轮当前)。本轮读40bec290a：979 9/12待final/收尾；975 8/11且B1正式已交，1064执行0/13/MODULE在做；1065已建树且用户确认工作中（metadata原drafting0/6待同步）；980 9/35/Hello已批审D待正式，981元数据0/34但SOURCE已正式落主。

A可整包收口：1065、1064、975、976（0/8）、979、980。跨A/B必须先交A：971-PROOF/977-PY已交；981 SOURCE/SESSION/RUST/LANGUAGE/PROOF、982 CONFIG/PROOF、983 PORT/PROOF、984 EMBED、OS064 HOST/PROOF先齐。985当前4/14，汇合全部required语言/新库/FFI/真实RG/组合→T-A03→A04→A05交M2A-08，随后才原B产品；不等985整包归档，也不让B反卡A。

推荐先清979，1065和1064 MODULE并行，模块正式交后补975；完整库/AC/原义务真齐后逐包收1065/1064/975，补976完成980全模式。981就绪SESSION/RUST/LANGUAGE并行，最终PROOF取完整适用正式能力。982–984/OS064的A接面靠近出口集中交，不提前铺开全B；A后按现合同保唯一树到B结束，不能为清树先冒归档。

出口后971/B、977 REVERSE、981内容/两游戏/FUSION/AUDIT按真实前置推进，982完整工具→983完整消费者、OS064全App/装配及984生态最后汇合；985/B→986→987最终安装/退役。981完整B仍保同源AV/A2R/游戏/融合原义务，SOURCE/Hello不代整包。

AGE当前：UI058/AG023各0/10、UI080 3/10最终接线需要所用981 A正式接口（SOURCE单项不充分）；AG029 2/10用户暂停不自动恢复，AG030 0/10，正式国风/光影产品保M2A-08；AG031 0/10等待四线成果与真实OS等原required来源后独立联合签章。Godot后置不减AGE自有Node/2D/3D/光影，TS仍保完整生产消费闭包，Linux按1061/1063既定NOT_RUN移交。

本轮只更新路线状态/顺序，不实施、测试、建树、取号、合并、清理或恢复目标。

## 17. Node桌面入口与AC原生补齐（2026-11-16，v22当前）

本轮用户只授权规划，沿[980/NATIVE](../../../../auto-lang/docs/plans/980-ace-m2-autoui.md)、[981](../../../../auto-lang/docs/plans/981-ace-m2-age.md)、[1068/T-04](../../../../auto-lang/docs/plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md)及[985](../../../../auto-lang/docs/plans/985-ace-m2-certification.md)原任务交付，不新立项、不写实现或canonical Specs。已交HELLO是真实AV文本/FFI字节接收/Rust宿主窗；A2R AXR0有界输出窗另算，均不代可复用UI-FFI入口或AC桌面。

A先980 B-NATIVE-I/V/R：普通AGE Node项目经Schema/共享verify→AV业务→967自动binding/shim受检UI效果/事件队列→同一Iced投影/主线程宿主。文字/函数文本、原002 Counter和两实例、原001主题/选择复制、关闭/坏更新/修复及完整NATIVE原义务都有独立oracle；移除实际UI提交、仅留byte_len验证必须红。交IF-980-NODE-DESKTOP-A后981 SESSION/LANGUAGE复用，不复制UI前端/控件库；原A2R、完整语法/五view/PROOF门继续，首AV入口不等AC、完整981或976/WEB。全部A-required齐后985/A交M2A-08，AC桌面按行有B承接而非记PASS。

B由1068/T-04接真AC机器码业务与typed事件回调到同一Rust/Iced宿主，宿主可以Rust预编译，guest不回AV或用A2R/rustc代编。981 FUSION/AUDIT消费其正式AC切片核UI/2D/3D、direct/RQ/device epoch/输入焦点/资源/包，1068不等981整包、981不等1068全AAC实证，最终985/B收真实三端/产品闭合→986/987。980完整A交付可归档，后续AC实现用1068/981各自唯一组与公共文件单写协作。

AGE独立G2/G3/AG031与两游戏A2R包不被AC样例替代，AG029暂停不自动恢复；Godot移出M2、JET/ARK归M3、WEB/TAURI与既定Linux移交保持。详细顺序见[总览§18](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#18-2026-11-16-node桌面入口当前顺序)。本轮仅静态文档修订，没有新运行/测试或M2A/B签章。

## 18. 最小A、完整UI三波与OS后续（2026-11-16，v23当前）

A由980 MIN-A-UI（普通Node Hello/函数中文/最小Counter、实际UI FFI/回AV事件与owner）、981 MIN-A-AGE（正常多模块与最小真实UI/2D/3D AV场景）、OS064 MIN-A-OS（真实启动/激活/窗口链承载隔离同源Node见证）交代表性真实接口，连同既定语言/标准库/非图形FFI/SDK与其他RG给985/A。完整旧示例/Widgets、AGE-L五view/复杂Frame/全语言产品覆盖、图形A2R和完整OS业务逐项交B，不再反等完整980 PARITY/981 RUST才签A。Notes/Clock完整旅程归OS064 A15/A12，真实桌面/全部登记App/双Host/旧数据归三波后的HOST B/ASSEMBLY；没有删旧oracle或冒迁移已完成。

M2A-08正式后：980 B-AV＋981域AV部分按U01..06完成全部UI/Widgets，交UI-W1-AV；再1068/T-04＋981 AC域同源AC机器业务/事件经同宿主，交UI-W2-AC；再980/981 RUST W3生成/编译/真实运行，交UI-W3-A2R。每波只当前后端的完整独立oracle，不同一计划同时实现三端，多个简单例可合组；A2R旧证据复用，只有第三波/最终门签图形三端。

Widgets源码在auto-os/widgets-gallery但组件/页面语义由UI波消费原源验证，完整gallery容器/OS注册不前置W1。三波后OS064全桌面/App→984 CLOSURE/985 B候选→986→987。1068最终T-06/08可后收W3，不能要求其整包先完才让W3启动；OS不等985 B最终、A不等任一完整UI波。AGE内容/两游戏包/G2/G3/AG031独立门及AG029暂停、direct/RQ/设备/焦点/音画/资源原义务保留，不用UI示例绿冒完整图形/游戏完成。

完整计划/执行顺序见[总览§19](../../../../auto-lang/docs/design/strategy/ace-route-plan-map.md#19-2026-11-16-a最小出口与完整ui三波os迁移本轮当前)；仅规划/静态核验，无新Plan、实现、窗口运行、合入或目标恢复。Godot移出M2、JET/ARK归M3、WEB/TAURI现役与Linux延期条件保持。
