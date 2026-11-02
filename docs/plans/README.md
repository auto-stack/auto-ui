# auto-ui 计划管理

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
| 007 | [树状主题、官方系列与示例归位/首批迁移](007-autoui-tree-theme-and-examples.md) | auto-lang未执行草案转交，原号释放 | drafting/r4，0/10；接受014/900/901/018交付，381文件/37目录示例归位仍全量待做；启动核近期Cargo有限出口与906正式组合，当前仅修订、暂不实施 |
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
| 029 | [Reader原生输入链与必要接缝收口](029-reader-native-input-and-surface-closure.md) | 026原生GAP与coverage v4后继 | drafting/r1、0/12；真实AutoUI原生VM的选区→.at→保存/冷恢复，IME/几何/系统无障碍、双实例/清理、严格非零汇总与旧门；全部必需门通过后独立上下文review，仅起草、未实施 |
| 030 | [三维具名引用与跨组件接线收口](030-autographics-node3d-symbolic-component-bindings.md) | 028作者收口/G11-3d前置 | drafting/r1，0/10；slot/template/pair符号与跨Node Actor只读参数、三同定义Coin局部状态；保原25/八状态/32，暂不实施 |

024/025/027/028已交付，026/029为Reader路线；本轮AG新增030；并行新计划的当前号源以`.next-id`为准，下一份以独占取号时本地活动/归档最大号重核（018分配后已有并行019，勿回退号源）；不能因外仓号882/883跳号。003是Blog路线，004..006是AG表现阶段，007是主题/示例工作包，014是拆分路线结构收口包，018是UI编译配置收口，026是Reader阅读表面跨仓探针；这些合同分别管理。状态以各自合同为准。设计与并行/后继入口见[本仓设计索引](../design/README.md)。

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
