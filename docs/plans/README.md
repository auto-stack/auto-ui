# auto-ui 计划管理

## AGE两阶段当前规划（2026-11-09）

用户新方向将AGE原M2图形增强与M3融合并为M2：ACE/AV/A2R、二维、三维、生产融合四线并行规划，仍不实施/先证/制作素材或运行构建。979 r5（0/8）、980 r4（0/31）、981 r7（0/31）、UI058/AG023 r2（各0/10）复用原号；先中立图形服务，最终Node双后端接线消费正式ACE接口。完整范围与联合门见[当前两阶段路线§6/§13](../design/autographics/age-three-milestones-and-ace-m2.md)。AG020更名、RQ063/073仍守独立owner，不充M1或新增融合重复立项。

M2另有两项独立硬门：980完整传统AutoUI（原widget/store源、纯UI AV/A2R、全部既有组件/模式）与981完整AGE语法（五view/选择/条件/Start-Tick-Frame/新旧UI互操作），不能以两个游戏/少量基础控件代替；状态均草案，仍仅规划。

| M1相关计划 | 当前状态 | 交付/剩余 |
|---|---|---|
| [UI076 scale轨迹与N4语义](archived/076-m1-native-scale-trail-and-n4-semantics.md) | archived r1，7/7 | 8b4ff336已交：F-SCALE-02 observe_window 合并语义＋f64 真实读数、F-N4-01 方案b收窄（SD-01/02 沉淀）；AG026 重判 N4 关 DEBT-001 提示见 debts.md DEBT-002 |
| [UI071 本机clip/取证支撑](archived/071-age-m1-local-clip-and-native-evidence-support.md) | archived r1，9/9 | a20fa04d已交，026/027该前置已满足 |
| [UI072 独立游戏宿主](archived/072-age-games-standalone-windows-player.md) | archived r1，10/10 | 5fd25be8已交，已供028消费 |
| [AG028 两游戏exe＋资源发行](../../../ag-examples/docs/plans/archived/028-age-games-windows-binary-and-resource-release.md) | archived r1，11/11 | 两Windows便携ZIP及清单已交，REL-DELIVERY-001仅持有最终文件待用户接管；不重开 |
| [AG027 OS编辑20/34](../../../ag-examples/docs/plans/027-age-m1-editor-native-stage-debt-closure.md) | archived r2，10/10 | 已交4bfbb15；真实OS执行/负控仍为DEBT-002残余，不据归档签M1-ready |
| [AG026 双DPI/最终M1收口](../../../ag-examples/docs/plans/026-age-m1-dpi-native-debt-and-final-closure.md) | drafting r1，0/11 | 071已交，准备可按启动指令推进；真实双scale外部门、027正式收据仍待 |

因此M1当前剩余活动收口合同是AG026（AG027已归档、残余债务保留）；DEBT-001/002仍OPEN，M1-ready未签。两游戏发行是追加交付，不改变原M1签章判据。旧065/066/067/024/025已交付、039影子不重复计。更多来源见[路线当前M1清单](../design/autographics/age-three-milestones-and-ace-m2.md)。本次只修订阶段/合同与排程，未运行实现/测试/输入或新增计划。

本仓独立编号从001开始；本地下一号见`.next-id`。跨仓身份写作`auto-ui/PLAN-NNN`。

## 当前合同

| 本仓号 | 计划 | 旧仓来源 | 状态 |
|---|---|---|---|
| 001 | [运行对象生命周期](archived/001-autographics-runtime-object-lifecycle.md) | auto-lang/PLAN-882 | archived r4；代码交付537122b，独立收据见归档 |
| 002 | [触发与新局（引擎）](archived/002-autographics-trigger-events-and-round-restart.md) | auto-lang/PLAN-883引擎部分 | archived r5；交付 0ee5ad0，SD/ledger 同步（auto-lang v130） |
| 003 | [Blog原生VM运行与可交互验收前置](archived/003-blog-native-vm-run-and-acceptance.md) | BLOG-004消费前置，非AG工作 | archived/r2; AutoUI prerequisite delivered a2ea08c; BLOG-004 owns full Blog acceptance |
| 004 | [公共图片与2D精灵帧](archived/004-autographics-public-images-and-sprite-frames.md) | AG M2表现 | archived；交付 e0fce13（reviewed 0e6e361），SD/ledger 沉淀 auto-lang v134 |
| 005 | [3D纹理与UV](archived/005-autographics-textured-models-and-uv.md) | AG M2表现 | archived r3；交付351919c，独立R3通过，SD/ledger v141已同步 |
| 006 | [节点动画与视觉朝向](archived/006-autographics-node-animation-and-visual-facing.md) | AG M2表现 | archived/r2；R2 pass；交付c681663，ledger143 |
| 007 | [树状主题、官方系列与示例归位/首批迁移](archived/007-autoui-tree-theme-and-examples.md) | auto-lang未执行草案转交 | archived/r4，10/10；示例来源归UI、Theme Lab/002/005交付；后继031/033已交付，OS真实事件沿053，正式包渠道/生成输入沿036 |
| 008 | [PLAN-004交付身份清单修复](archived/008-plan004-identity-closure.md) | auto-ui/PLAN-004身份闭包 | archived/r1；delivered 0261ae9，原路径verify_source通过；ledger auto-lang v149；清理回执见计划 |
| 011 | [2D完整玩法整链预算验证](archived/011-autographics-g2-full-loop-budget-feasibility.md) | AG独立研究 | archived/r2；两候选no_go_current_profile（33/32），交付c4143af；知识/ledger v145，完整M2未证实 |
| 012 | [公共音频与接受后一次播放](archived/012-autographics-public-audio-and-accepted-cues.md) | AG公共服务 | archived/r1；45case与完整fresh通过，公共音频交付；G12a内容另仓 |
| 013 | [公共HUD与共同呈现](archived/013-autographics-public-hud-and-presentation-state.md) | AG公共服务 | archived/r1；R2 pass；冻结定向门通过，交付8b1e799；SD/ledger auto-lang v152；清理回执见计划 |
| 014 | [UI/Man内部结构收口](archived/014-ui-man-structure-closure.md) | 语言拆分路线第二结构包，主改面UI、Man参与 | archived/r2，11/11；R1 pass（reviewed；三独立上下文源码重建，仅doc精度级发现当场修复）；交付d66627a（UI，rebase重写range-diff全等）/ auto-man 1eeac78 / auto-lang d5cfc79c1（SD-05/06），统一最终门全绿（UI 2960/2960、tu 866/0、ti 28/28；Man t/tcore/tc/ti全绿）；canonical specs三仓核验在案；014交付，007按原门推进；清理回执见计划 |
| 015 | [VM路由页动态加载与原生Blog闭环](archived/015-vm-route-page-dynamic-load.md) | BLOG-004消费前置 | archived/r2；交付df39593；native22/22，规范/ledger148；清理回执见计划 |
| 016 | [Node2D最小真实执行验证](archived/016-autographics-node2d-minimal-validation.md) | AG共同作者模型最小切片 | archived/r3；R1 needs_fix 11项→修复→同员R2 pass；交付932bd5f（实现9eb1d0e），gate全绿；SD node2d-pilot.md 沉淀本仓；清理回执见计划 |
| 017 | [完整G11/2d Node作者能力](archived/017-autographics-g11-2d-node-authoring.md) | 016后继/原G11二维兼容 | archived/r1；R1 needs_fix 4项→修复→同员R2 pass；交付9a8a5be（bf02061 rebase，range-diff全等），T-08门全绿；SD node2d-g11-authoring.md 沉淀本仓；清理回执见计划 |
| 018 | [Design52 D：UI编译配置收口](archived/018-ui-build-configuration-closure.md) | Design52 D；接受auto-lang/900与901交付 | archived/r1，8/8；R1 pass；交付968ac83（两轮rebase，range-diff 8处全等，ff-only零merge commit），canonical specs docs/specs/build/{ui-compilation-configuration,ui-test-selection}.md 在master；runtime语义等价+测试边界重构，无产品行为变更；014/007职责未并入；清理回执见计划 |
| 019 | [Node2D场景层级与走跳相机](archived/019-autographics-node2d-scenes-character-camera.md) | Gallery新作者形式首波引擎 | archived/r1，10/10；交付c93adb7，独立R2通过/冻结门/两Spec与清理收据齐；AG008已合并8a01a34并固定消费019，Gallery交付时迁移6/21；当前总进度见设计路线 |
| 020 | [有状态Node组件与局部事件](archived/020-autographics-stateful-node-components.md) | AG局部性共同基础 | archived/r1，10/10；R1 needs_fix 4项→修复（含R2/R3各1项）→受影响复核pass；交付16ddfa0（7提交rebase重写，range-diff 7/7全等），verifier 18组/flow 9/9/旧消费者门全绿；SD stateful-node-components.md 沉淀本仓；021/022/023硬等解除；清理回执见计划 |
| 021 | [Node二维segment/sweep查询](archived/021-autographics-node2d-queries.md) | W2a/G05-2d引擎 | archived/r1，10/10；R1 needs_fix 4项→修复→同评审者受影响复核pass（P3文本项R2闭环）；交付1bec52f（8提交rebase重写，range-diff 7/7全等），verify021 20组5产品/flow 11/11/verify020 18组全绿；SD node2d-queries.md 沉淀本仓；AG009可消费；清理回执见计划 |
| 022 | [Node精灵资源与帧动画](archived/022-autographics-node2d-sprites-and-frame-animation.md) | W2b/G09双入口引擎 | archived/r1，10/10；R1 needs_fix 5项→修复→同评审者re-review pass；交付2f45baa+c6aa63a（rebase over 023交付/024-025规划，受影响验证rebased态全绿：七verifier/flow 22行receipt/pilot 54/旧consumer 9/9），SD node2d-sprites-and-frame-animation.md sha f997bae6 沉淀本仓；原G09 18项oracle逐字全绿于sprites-2d链；清理回执见计划 |
| 023 | [Node二维声音与公共HUD](archived/023-autographics-node2d-audio-and-hud.md) | AG006 r2作者能力前置 | archived/r1，10/10；R1 needs_fix 3项P2（6新告警+SD计数笔误）→修复d125b6d→同评审者复核pass；交付32d7a81（13提交rebase重写，range-diff 13/13全等），统一门全绿（pilot 44/driver 45/旧flow 35/verifier 10组）；SD node2d-audio-and-hud.md 沉淀本仓；AG006 T-01硬门解除；清理回执见计划 |
| 024 | [3D声明/静态资源/组件基础](archived/024-autographics-node3d-foundation.md) | W3基础/G02与G03-3d引擎 | archived/r1，10/10；R1 needs_fix 4项（LICENSE常量/同名helper覆盖/投递负控/v-parent像素）→修复→同评审者re-review pass；交付64166db（rebase后range-diff全等，ff-only零merge commit），门禁= pilot 62/gates 8/driver flow 6/旧consumer 13+9+11+4/verify_plan024 29控制组/六冻结verifier全绿；SD node3d-foundation.md sha 9451cf0f 沉淀本仓；原G02五+G03/3d六义务逐项复验于新链；清理回执见计划 |
| 025 | [声音/HUD通用Node接线收口](archived/025-autographics-node-presentation-services.md) | 023有限game档的通用服务后继 | archived/r1，10/10；交付e9cc929、canonical/锁收口54899e9，独立final/落主smoke/归档清理齐；通用服务已交付，不自动重开AG006固定023内容 |
| 026 | [阅读引擎嵌入与定位回传探针](archived/026-reader-engine-embedding-and-locator-probe.md) | auto-reader/READER-006、coverage v3 后继 | archived/r1、流程10/10；Vue指定样本30/30，原生嵌入/装载/IPC已证，真实输入/IME/无障碍等未闭合；V-08/V-10未单列。研究归档不等于双端全门go，下一阶段先收口原生接缝 |
| 027 | [3D角色与运行相机Node接线](archived/027-autographics-node3d-character-and-runtime-camera.md) | W3运行能力/G07与G08-3d引擎 | archived/r1，10/10；交付f3dea0d，独立final/冻结/归档清理齐；SD node3d-character-and-runtime-camera.md sha95a86660；原24oracle保持，AG011/012可固定消费，不把引擎绿当内容交付 |
| 028 | [三维实例/触发/新局Node接线](archived/028-autographics-node3d-instances-triggers-and-rounds.md) | W3/G11-3d有限服务作者接线 | archived/r1，10/10；交付ebac1cd，独立final/冻结/归档清理齐；SD sha be4d9c15；25旧oracle保持，字符串/数值pair/本定义subject仍为有限边界，030补具名参数/跨组件接线 |
| 029 | [Reader原生输入链与必要接缝收口](archived/029-reader-native-input-and-surface-closure.md) | 026原生GAP与coverage v4后继 | archived/r1、12/12；双profile冻结exit0（vm 92记录/vue 34记录零非绿，N-01..12全绿+两端旧门28/28、27/27+R-05 22/22）；N-04配对协议、DPI窗口比例oracle、SD-01/02沉淀；独立review pass（AC-11，e7b09ea8），landing 65bb455f；026两项具名GAP闭合，残留边界见归档与native-input spec |
| 030 | [三维具名引用与跨组件接线收口](archived/030-autographics-node3d-symbolic-component-bindings.md) | 028作者收口/G11-3d前置 | archived/r1，10/10；交付804d7303、Spec SHA6ed0bf07/冻结/清理有记录；review原为同实现会话重建（非独立会话，历史记录保留）；2026-11-02经用户授权补录独立上下文final pass（独立reviewer绑定804d7303：range-diff两行全等/canonical 6ed0bf07三点等/verify --controls 32 checks复跑PASS/gates 5/5/flow 8/8/AC-01..06全pass/自选反例5条具名拒绝，收据在归档§9），AG013 T-02门已开启 |
| 031 | [主题作用域、原子发布、正式主题包与基础控件收口](archived/031-theme-scope-publication-and-package-closure.md) | 拆分后架构工作包A；接受007/910 | archived/r1，11/11；已交付（评审 pass @ b6c26186，ff-only 落地）；SD-01..05 + packages/themes 生效，003/010 双端迁移完成，后继033已交付，B1累计10例，余24应用；发行/生成输入差额接036，宿主接OS053 |
| 032 | [完整2D游戏组合与预算收口](archived/032-autographics-node2d-full-game-composition.md) | M2游戏优先/引擎线 | archived/r2；交付a8c04d22，独立final/落主/Spec/清理齐；原32内完整G2公共组合，AG014已固定消费并交付 |
| 033 | [剩余六个基础示例正式主题迁移](archived/033-basic-examples-theme-migration.md) | UI031后继B1六例 | archived/r1，9/9；实际交付d734f1fe、Spec/归档清理齐；复审记录同实现会话重构的来源限制见归档，非AG新计划前置 |
| 034 | [3D纹理资源与节点动画作者接线](archived/034-autographics-node3d-materials-and-animation.md) | 005/006/030新作者收口 | archived/r1，交付93eaa907、Spec gallery-node3d-textures-and-animation.md；AG015已消费。复审收据明确同实现会话复核，相关独立证据/持名hold不据归档视为已闭 |
| 035 | [3D声音/HUD与完整G3组合接线](archived/035-autographics-node3d-audio-hud-composition.md) | 034后继/公共三维音画 | archived/r1，交付5903839e（ff-only落主、7补丁range-diff等价、主smoke 26/26）；SD-01 canonical node3d-audio-hud-composition.md字节核d7272516。复审收据如实记录同实现会话final（判定从工件重建、五门独立重跑），同034先例不称独立上下文pass；D-04声学/走查hold随归档保留，D-05批量回归被阻具名移交（他会话主checkout WIP+缺批量门定义），不阻塞下游消费。AG016完整接线引擎前置实质解除，034相关证据核验仍归AG016计划内 |
| 036 | [正式主题包发行与生成输入收口](archived/036-official-theme-distribution-and-input-closure.md) | 031/033后继 | archived/r1，10/10；三仓落地/清理齐；原V04×2/V05a归语言930，Tauri R-6与vue-render渠道另归owner，不冒全部发行已绿 |
| 037 | [B2十二个业务示例主题迁移](archived/037-business-examples-theme-migration.md) | 031/033后继B2 | archived/r1；最终收据11/11、delivery0f90ec1b，头部仍7/11待owner对账；B1+B2累计22，原首红/复审来源及vue-render渠道待办按归档保留 |
| 038 | [B3专用示例主题支持面核验](archived/038-specialized-examples-theme-readiness.md) | B3有界研究 | archived/r1，8/8；十二例实测/具名前置已落Spec，研究不抵迁移；正式后继045/046/047 |
| 039 | [3D查询、动态组与二维标记Node接线](archived/039-autographics-node3d-queries-and-mixed-markers.md) | G05/3d新作者前置；021/024/030 | archived/r1，交付8f577a34（含复审finding R-1修复ff5d267d）；SD-01 canonical node3d-queries-and-mixed-markers.md已落。final收据如实记录同实现会话（判定从工件重建、pilot152/dialect103/gates7/flow8/verifier独立重跑全绿）；D-03 OS phase-14探针沿AG009持有、G05逐字OS迁移归AG017。AG017固定消费其实际交付，前置实质解除 |
| 040 | [Node双维编辑工程与项目化验收](archived/040-autographics-node-editor-projects.md) | G04纯场景/编辑副本与公共验收收口 | archived/r1，10/10；已交付fc82061c（ff-only落主；独立subagent上下文final pass绑定fc82061c）；SD-01 canonical node-editor-projects.md sha 1daa7111；editor-projects身份孪生入口（queries-3d同机制）+G04忠实双维夹具/冻结产品+driver项目化7门+verify_plan040全PASS+旧档字节冻结复核；AC-03 OS层15行D-02具名pending保持开放；清理回执见计划；AG018可固定消费实际交付 |
| 043 | [Node3D模型资源对象模板](archived/043-autographics-node3d-model-object-templates.md) | Node模板builtin限制后继 | 已交付并归档/清理；静态opaque模型模板与非solid/owner边界见实际收据，不自动升级旧游戏锁 |
| 044 | [Node源码定位与分层诊断](archived/044-autographics-node-source-diagnostics.md) | 多模块作者体验后继 | 已交付9bf9874a，canonical/归档清理齐；固定SDK无span时仍如实fallback |
| 041 | [组件与Store实例所有权收口](archived/041-component-and-store-instance-ownership.md) | Design47/S后继 | archived/r1，10/10；delivered da4485b4（ff-only 落主）；独立 final pass @cf087715；SD-01..04 canonical 落地（component-and-store-instances.md）；VM 实例分裂+store 每App context；13 base-red/边界如实记录 |
| 042 | [Reload/schema/effect有界核验](archived/042-reload-schema-and-effect-readiness.md) | Design47/R后继 | archived/r1，8/8；研究交付；实际原子多文件Reload后继056，坏源由语言940，浏览器隔离由057 |
| 045 | [媒体三Demo主题迁移](archived/045-media-demo-theme-migration.md) | 038实测后继 | archived/r1，9/9；019/020/030交付，TS1005既有红保留owner |
| 046 | [图像文件三Demo主题迁移](archived/046-image-and-file-demo-theme-migration.md) | 038实测后继 | archived/r1，9/9；三例及Web/Man接缝已落地清理 |
| 047 | [阅读编辑数据绘制六Demo主题迁移](archived/047-specialized-workbench-demo-theme-migration.md) | 038实测后继 | archived/r1；最终收据9/9、头6/9未同步；B3十二已迁、教学累计34；025 a2r既有红具名保留 |
| 048 | [基础UI Node与Auto事件](archived/048-age-node-basic-ui-and-auto-events.md) | AGE M2-10基础UI | 已交付cd1e161a，归档/清理；四类控件/Auto事件已支持；final为同实现会话，独立上下文复用核验归054/AG021 |
| 049 | [UI/2D/3D同应用组合](archived/049-age-ui-2d-3d-composition.md) | AGE M2-10混合切片 | 已交付b7f01267，归档/清理；root有限混合档，不支持组件区域/条件挂卸；同会话final限制保留，补齐归054 |
| 054 | [混合区域组件与生命周期](archived/054-age-mixed-regions-components-and-lifecycle.md) | M2/AG021公共接线 | 已交付2571556e（ff-only落主），归档/清理；v2组件区域/条件挂卸/活区域≤4/bool高槽修正在案；独立final pass（subagent上下文）；AG021按SD-01候选§2配方消费 |
| 055 | [M3俯视二维与容量可行性](archived/055-age-m3-topdown2d-feasibility-and-capacity.md) | M3首轮研究 | archived/r1；独立final pass；无地面原型bounded-go、铺地/角色35-credit组合no-go；canonical与清理见同号收据 |

当前AGE见[最后缺口与首轮预研](../design/autographics/m2-closure-and-m3-first-wave.md)：048/049已交付，但049的root有限支持不能关闭AG021原组件化/条件挂卸义务。M2主链为**054实际交付/独立核验→AG021 r2最终同版消费与M2-01..10必需门→M2结论**；019旧候选/holds保历史，Gallery22/22不是M2完成比例。020只是M2收口后更名维护，不是功能门。AG021原owner已记T-01/02准备完成、executing/r2/2-10；T-03等054真实交付，不追tip。

043/044均已交付；055与ag-examples/022可并行准备M3输入，AG022最终容量建议等055实际研究结果，正式M3产品能力不倒灌M2。本轮只规划，不建树/运行/接管实现。取号独占核主检出、归档及所有注册worktree的本仓合同：原counter050落后于已存在050..053，上批取054/055时推进至056；本批另路线已取056/057，本批取058/059推进060。不覆盖其它合同，未来新号仍重新核现场。

## 拆分路线下一批（2026-11-05）

| 主合同 | 状态与边界 |
|---|---|
| [052原生exe生成器](archived/052-rust-exe-codegen-fix-for-app-backs.md) | archived5/5、64aef780交付；OS054接正式修复继续组合 |
| [053宿主多API与栈修复](archived/053-desktop-host-multi-api-surface-and-stack-fix.md) | archived/r2，6/6；UI 9a94d568、lang 61207576b正式交付与清理完成 |
| [056原子多文件Reload](056-atomic-multifile-reload-and-schema-lifecycle.md) | drafting/r1、0/8；坏源腿及最终交付接语言940，暂不实施 |
| [057 Vue真实双App/Store隔离](057-vue-app-store-isolation-and-owner-lifecycle.md) | drafting/r1、0/7；接041/050/051，独立于完整Reload；暂不实施 |

设计见[坏源/原子Reload/浏览器实例第二批](../design/autoui/atomic-reload-and-live-isolation-stage-two.md)，总调度见[拆分路线v27](../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)。AGE的054/055另线，OS055另仓同号不混用；三新合同未运行测试或实施。

## 下一批及真实执行顺序（2026-11-05）

[全路线逐项状态](../design/autographics/plan-status-and-execution-order-2026-11-05.md)列全部本地AGE合同与交付来源，头部状态不能代真实完成。054已有唯一树/实现提交，头部drafting与执行记录不同步，按实施中未交付；不由本规划会话修改或提交其进度。055/AG022仍草案。

| 本仓号 | 计划 | 状态与硬门 |
|---|---|---|
| 058 | [俯视二维控制/查询/精灵组合](058-age-topdown2d-queries-and-sprites.md) | drafting/r2/0-10；M2二维线，中立服务可与ACE并行，T-04起同源AV/A2R等981正式来源及原M1门；原最小样本/预算保留，未实施 |
| 059 | [方向光/阴影作者链研究](archived/059-age-node3d-lighting-and-shadow-feasibility.md) | archived/r1/10-10；研究bounded-go：Auto标量绑定→真GPU；20轮研究Run/Stop及三路恢复；非正式Light API |

主链054→AG021→M2结论；研究055与AG022准备、059可并行；M3二维正式链055→AG022＋M2收口→058→ag-examples/023。020仅M2后维护更名。056/057已由其它UI路线占用，本批独占重核取058/059，counter060；后续重新核现场，不覆盖其合同。

## 拆分路线状态更新（2026-11-05，v27）

UI052已交付；UI053已archived/r2/6-6并完成清理。UI060（索引、iced桌面outproc输入/刷新及真实native API）仍executing/r1/2-6，扩围与续接记录为准，当前合同已恢复[主检出](060-a2r-index-vm-semantics-and-desktop-outproc-input.md)；原owner继续执行，尚未正式交付。UI053/060分别为OS054的Vue/原生轨前置，不与OS060主题批次混用。UI056/057、语言940/942仍沿各自草案。

OS055现已归档11/11，真实系统事件及载体闭环完成，OS061已executing2/9。仅更新状态，不改任何实施合同或代行验收；AGE054/055/058/059与本线分别管理。

## 债务/候选

未立项的候选功能与债见 [debts.md](debts.md)（当前：DEBT-001 autoui MCP 批量动作）。

## 管理规则

- 2026-10-31新规：新计划不写attachments（含md）；临时材料统一plan-artifacts/plan-NNN/<repo>/，merge前将必要摘要折入Plan并清理自有scope/构建中间物；仅具名未解决问题持有，旧specs.json只读。
- 本仓主检出`docs/plans/`持唯一活动合同、进度、复审和归档；归档目录以已交付001/002的实际约定为`archived/`（取号兼扫历史`archive/`，避免遗漏）。
- 取号只扫描本仓活动/归档的本地Plan；外仓历史来源、附件、重定向不计高水位。独占核`.next-id`和本地最大号后创建，禁止覆盖。
- 跨仓计划由主责任仓持一份合同，参与仓只留定位指引；旧auto-lang编号保留为来源，不再作为本地编号。
- 四阶段new/work/独立review/merge沿用；源码修改和构建在合同指定唯一worktree，Plan簿记在本仓主检出；禁止worktree链接，清理先guard。
- 001/002 已归档（002 交付 0ee5ad0）；变更相关测试/全量AG里程碑/OS自动化暂停等原政策保持。
- 旧Specs/设计的现有知识权威仍按Plan跨仓引用，本次没有批量复制历史知识。

r2文档原貌保留于本仓提交`6c7bcdccb08b6320e5920e704990ab6eaa515e41`；r1保留于auto-lang提交`a5d8a4fa8`。编号更正不重写这些历史。

用户批准按仓拆分（2026-10-29）：002仅auto-ui引擎/夹具/消费证据；[ag-examples/001](../../../ag-examples/docs/plans/archived/001-autographics-gallery-runtime-objects.md)独立承接G11，硬等002实际交付。两份分别review/merge，无反向依赖。


2026-11-05 UI055研究已交付：当前结论见[canonical可行性与容量](../specs/auto-graphics/topdown2d-feasibility.md)及[同号归档](archived/055-age-m3-topdown2d-feasibility-and-capacity.md)；055的旧规划状态由实际研究收据替代。AG022/058继续按其自身合同和已交付前置推进。


## RQ完整功能子路线（2026-11-05新立，暂不实施）

以[子路线](../design/autoui/renderqueue-refactoring-roadmap.md)为准；不重开683/690历史交付，不把未来ACE/AGE设为当前前置。

- [061保真合同与可行性](061-rq-fidelity-contract-and-feasibility.md)：drafting r3，0/6。
- [062无损绘制与资源](062-rq-lossless-rendering-and-resource-parity.md)：drafting r3，0/7，接061。
- [063运行与输入](063-rq-vm-rust-runtime-and-input-parity.md)：drafting r3，0/7，接060正式修复与062。
- [OS063双Host装配](../../../auto-os/docs/plans/063-rq-dual-host-assembly-and-parity.md)：drafting r3，0/7；OS062 Linux平台准入保原合同。

2026-11-05 r2：四份合同补P-01～04性能机制、M-01～08 direct对比指标及AC-07/V-05～08；先基线后冻结预算，本轮不实施、不测量，任务数不变。

2026-11-05 r3：四RQ草案范围/依赖复核，增加V-09接口兼容负控；绘图、运行、产品六格分层，明确UI060/OS062去重和OS063的UI策略接线。启动条件见[子路线§7](../design/autoui/renderqueue-refactoring-roadmap.md)；不实施、不更改在途合同。
