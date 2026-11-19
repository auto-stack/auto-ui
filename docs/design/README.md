# auto-ui设计索引

## AGE M2三路线当前入口（2026-11-17，仅规划）

| 设计 | 范围 |
|---|---|
| [统一strategy](../../../auto-lang/docs/design/strategy/age-m2-three-route-strategy.md) | R1生产UI替代、R2首批后2D/3D增强、R3新版AutoForge；A/B和三波有向交接 |
| [共同架构](../../../auto-lang/docs/design/autographics/age-m2-three-route-architecture.md) | 单一Schema/Node、typed资源/事件/FFI、阶段来源与真实验收边界 |
| [引擎详细合同](autographics/age-m2-engine-extension-contract.md) | UI099–102四有界AV合同；AC/A2R由981/1068既有owner承接，AG031增量required |
| [AutoForge产品架构](../../../auto-forge/docs/design/architecture.md) | 工程/源码往返/隔离预览/恢复/Agent不可信提案；新仓独立产品 |
| [AutoForge首批执行路线](../../../auto-forge/docs/strategy/roadmap.md) | 001→002→003先AV整包，004先AC阶段再A2R，不反卡985/A或AG031 |

UI080已归档10/10，保有限单方向光/硬阴影的实际已交范围；旧M3/M4文件名和下方日期状态为历史映射，当前调度以上述入口及Plan实际正式来源为准。

AGE（Auto Graphics Engine；旧称AG/AutoGraphics）实验代码与实现计划已归本仓；旧canonical Specs与历史设计继续按明确跨仓链接引用，以下只登记新增本仓设计。

| 专题 | 范围/状态 |
|---|---|
| [AutoUI组件库扩展路线 strategy](autoui/component-library-extension-strategy.md) | r6/2026-11-14：084/086代码交付归档，保留待补门；下一项建议085 TreeView。094–097经[再次审核](../reviews/component-plans-094-097-2026-11-14/README.md)为drafting r2、0/8，复用既有[四组件图](autoui/file-navigation-and-interaction-visual-guide.md)；087–093合同保持 |
| [AGE两阶段当前路线与ACE M2接洽](autographics/age-three-milestones-and-ace-m2.md) | 2026-11-09：原M2增强与M3融合合并M2，执行/二维/三维/生产融合四线并行规划；979 r5、980 r4、981 r7、UI058/AG023 r2；传统UI全支持与完整AGE语法为独立必需门，仍未实施。下方旧M3/M4文件名/日期快照为历史映射，当前口径以本路线为准 |
| [两游戏Windows独立运行与二进制发行](autographics/games-windows-player-and-release.md) | UI072已交5fd25be8、10/10；AG028已归档11/11，原两游戏exe＋资源便携包已交。ACE/A2R新发行归981，不重做原游戏 |
| [RQ完整功能与直接渲染一致性](autoui/renderqueue-complete-fidelity.md) / [子路线](autoui/renderqueue-refactoring-roadmap.md) | UI061–063及OS063草案r3；VM/Rust六格、三类Host、ACE/AGE；异步批量/复用/调度/隔离与direct性能对照，仅规划 |
| [下一批M3组合/研究与执行顺序](autographics/m3-next-wave-execution-order.md) | UI058俯视组合/059方向光研究/AG023二维小例；先行准备与真实交付门分明，仅规划 |
| [全AGE计划状态快照](autographics/plan-status-and-execution-order-2026-11-05.md) | 迁仓后逐项合同/实际交付与当前8项未完、并行和顺序；不重复计历史映射 |
| [原子Reload与浏览器实例第二批](autoui/atomic-reload-and-live-isolation-stage-two.md) | 语言940坏源恢复→UI056原子多文件Reload；UI057真实Vue隔离独立并行，全部仅规划 |
| [M2最后缺口与M3首轮预研](autographics/m2-closure-and-m3-first-wave.md) | UI054补公共混合接线；UI055俯视二维研究/AG022种田切片与three.js盘点可并行准备；仅规划 |
| [M3二维/2DHD与复杂三维增强](autographics/m3-2d-3d-capability-roadmap.md) | 国风种田切片＋three.js固定目录；首轮UI055/AG022研究合同已规划，未实施产品能力 |
| [M4与现有AutoUI融合](autographics/m4-autoui-integration-roadmap.md) | 原M3融合顺延；等ACE Schema成熟及旧编译器替换实际启动，再从001逐步跑通UI Demo；无新Plan或实施 |
| [M2新语法UI/2D/3D同应用](autographics/m2-mixed-ui-2d-3d-composition.md) | M2-10；048/049已交付有限切片，UI054→AG021 r2补公共区域/寿命与最终验收，原分母保持 |
| [AGE命名与内容仓目录](autographics/age-naming-and-content-repository.md) | 正式Auto Graphics Engine（AGE）；ag-examples/020 r2仅规划更名，默认019＋021/扩展M2收口后，历史号/SHA/schema不变 |
| [模型对象组件与Node源码诊断](autographics/node-resource-components-and-source-diagnostics.md) | UI043/044均已交付，原合同/来源与预算边界保留，不自动升级旧游戏 |
| [M2编辑教程与同版发行收口](autographics/m2-editor-and-release-convergence.md) | 040/AG018/019均已交付，OS等具名边界保持；新M2-10另接补齐设计，不重开旧合同 |
| [三维查询、完整G3与G05后继](autographics/node3d-queries-and-complete-game.md) | 历史批次；035/039/AG016/017已交付，来源限制/原验收保持，当前见M2路线 |
| [3D表现/音画与G10新作者迁移](autographics/node3d-presentation-audio-and-g10-migration.md) | 历史批次；034/035/AG015已交付，相关独立证据/hold据实保留 |
| [M2游戏优先三线并行](autographics/m2-game-first-three-lane-roadmap.md) | v10；Gallery22/22、两游戏/019旧候选已交；054→AG021 r2及必需门收M2，055/AG022预研M3 |
| [AGE声明与组件作者模型](autographics/declarative-authoring-model.md) | AutoUI式声明/组件/绑定/事件，AG-A01..09与历史收据身份保持 |
| [G11/2d Node完整作者能力](autographics/g11-2d-node-authoring-completion.md) | 017/AG007已实际交付；完整目标草图未全部实现，旧三维及runner裁决边界保留 |
| [Gallery全量Node作者迁移](autographics/gallery-node-authoring-migration.md) | 当前22/22；AG017+1/AG018+2已落主，源码迁完仍不等M2 |
| [Node局部组件、二维查询与精灵](autographics/node-components-queries-and-sprites.md) | 2026-11-01规划v1；020补子model/msg/on→021查询/022精灵；AG008已合并并固定019，与下一波能力合同独立 |
| [UI/Man结构收口](autoui/ui-man-structure-closure.md) | target-state r1；[本仓014](../plans/archived/014-ui-man-structure-closure.md) drafting0/11；U890后收CLI/Vue/Aura/renderer，U014后按007原门推进主题/示例；仅规划 |
| [AG M2表现与资源](autographics/m2-presentation-and-resources.md) | 004/005/006底层表现已交付；历史拆分保留，当前新作者接线见034/035/G10设计 |
| [树状主题与官方系列](autoui/theme-conventions-and-pilot.md) | 主题约定、真实demo首批与全部应用后继接入；auto-ui/007 r3 drafting |
| [示例归属与搬迁](autoui/examples-ownership-and-relocation.md) | examples/ui 拟从语言仓归位本仓；同007先搬来源、后主题改写，不新建 ui-examples |
| [Button树状主题](autoui/button-theme-atom.md) | 普通use、token引用、模式、widget局部设计；未实施 |
| [主题迁移逐项清单](autoui/theme-migration-inventory.md) | 当前源目录与旧样式静态盘点，不是运行通过证明 |
| [数据优先模型](autoui/design-data-model.md) | 当前方向；[抽象研究](autoui/design-system-abstractions.md)与[旧语言方案](autoui/button-design-syntax.md)为讨论依据 |
| [AG公共声音与HUD](autographics/m2-public-audio-and-hud.md) | 012/013/023有限game与AG006、025通用服务均已交付；006固定023不自动重开，声学等具名gap保持 |
| [Node声音/HUD与下一轮Gallery](autographics/node-audio-hud-and-gallery-followups.md) | 历史拆分；025/AG006/035已交付，当前19/22，正式G3归AG016合同 |
| [3D Node基础与通用声音/HUD接线](autographics/node3d-foundation-and-generic-presentation.md) | 024/025/027/028/030及AG011/012/013均已交付；后继034表现→035三维音画，原预算/验收保持 |
| [3D角色/运行相机与Gallery三维迁移](autographics/node3d-runtime-and-gallery-migration.md) | 027已交付f3dea0d，原模型/角色/相机能力供011/012固定消费；原拆分快照保留 |
| [三维实例/触发/新局与角色相机内容](autographics/node3d-instances-triggers-and-gallery-motion.md) | 028与AG012已实际交付，012固定027；原拆分快照保留，当前三线见新设计 |
| [三维具名引用与G11迁移](autographics/node3d-symbolic-bindings-and-g11-migration.md) | 030已交付且独立final补齐；AG013已交付b48b304，原25/完整双维包保持；新批见034/035/AG015 |
