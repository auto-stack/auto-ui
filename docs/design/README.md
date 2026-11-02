# auto-ui设计索引

AG实验代码与实现计划已归本仓；旧canonical Specs与历史设计继续按明确跨仓链接引用，以下只登记新增本仓设计。

| 专题 | 范围/状态 |
|---|---|
| [M2游戏优先三线并行](autographics/m2-game-first-three-lane-roadmap.md) | 2026-11-02 v2；UI032＋ag-examples/AG014完整游戏＋AG013 r2 Demo每批三线，游戏/Demo同仓分区；同步规划3D表现/音画，2D稳定闭环后交付完整3D；仅规划 |
| [AG声明与组件作者模型](autographics/declarative-authoring-model.md) | 2026-10-30总体目标回归；AutoUI式声明/组件/绑定/事件，AG-A01..09；六项细节为讨论建议，未实施 |
| [G11/2d Node完整作者能力](autographics/g11-2d-node-authoring-completion.md) | 017/AG007已实际交付；完整目标草图未全部实现，旧三维及runner裁决边界保留 |
| [Gallery全量Node作者迁移](autographics/gallery-node-authoring-migration.md) | 当前16/22，剩6；AG012已交付05f1b498，AG013 r2并行准备/完整迁移核030独立review门 |
| [Node局部组件、二维查询与精灵](autographics/node-components-queries-and-sprites.md) | 2026-11-01规划v1；020补子model/msg/on→021查询/022精灵；AG008已合并并固定019，与下一波能力合同独立 |
| [UI/Man结构收口](autoui/ui-man-structure-closure.md) | target-state r1；[本仓014](../plans/archived/014-ui-man-structure-closure.md) drafting0/11；U890后收CLI/Vue/Aura/renderer，U014后按007原门推进主题/示例；仅规划 |
| [AG M2表现与资源](autographics/m2-presentation-and-resources.md) | 规划v1；本仓004/005/006串行，Gallery内容另仓，均未实施 |
| [树状主题与官方系列](autoui/theme-conventions-and-pilot.md) | 主题约定、真实demo首批与全部应用后继接入；auto-ui/007 r3 drafting |
| [示例归属与搬迁](autoui/examples-ownership-and-relocation.md) | examples/ui 拟从语言仓归位本仓；同007先搬来源、后主题改写，不新建 ui-examples |
| [Button树状主题](autoui/button-theme-atom.md) | 普通use、token引用、模式、widget局部设计；未实施 |
| [主题迁移逐项清单](autoui/theme-migration-inventory.md) | 当前源目录与旧样式静态盘点，不是运行通过证明 |
| [数据优先模型](autoui/design-data-model.md) | 当前方向；[抽象研究](autoui/design-system-abstractions.md)与[旧语言方案](autoui/button-design-syntax.md)为讨论依据 |
| [AG公共声音与HUD](autographics/m2-public-audio-and-hud.md) | 012/013/023有限game与AG006、025通用服务均已交付；006固定023不自动重开，声学等具名gap保持 |
| [Node声音/HUD与下一轮Gallery](autographics/node-audio-hud-and-gallery-followups.md) | 历史拆分快照；当前16/22与三线优先级见新路线，025已交付，AG006固定023 |
| [3D Node基础与通用声音/HUD接线](autographics/node3d-foundation-and-generic-presentation.md) | 024/025/027/028及AG011/012已交付；030落主/独立review证据待补，AG013保原门 |
| [3D角色/运行相机与Gallery三维迁移](autographics/node3d-runtime-and-gallery-migration.md) | 027已交付f3dea0d，原模型/角色/相机能力供011/012固定消费；原拆分快照保留 |
| [三维实例/触发/新局与角色相机内容](autographics/node3d-instances-triggers-and-gallery-motion.md) | 028与AG012已实际交付，012固定027；原拆分快照保留，当前三线见新设计 |
| [三维具名引用与G11迁移](autographics/node3d-symbolic-bindings-and-g11-migration.md) | UI030交付804d7303但独立review证据待补；AG013 r2准备与UI032/AG014并行，保原25与完整双维包 |
