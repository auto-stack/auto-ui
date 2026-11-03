# M2游戏优先路线：内容闭环、编辑迁移与统一版本联验

2026-11-03，设计v5。AG作者模型仍是AutoUI式Node/use、局部model/msg/on、view-2d/view-3d及具名资源/事件；桌面M2原两游戏/编辑/必要音画/资源离线/Gallery/统一最终引擎版本分母保持。ACE/AOT/ECS/CPC/跨端不新增前置。

## 1. 当前真实状态

| 事项 | 已核事实及边界 |
|---|---|
| 完整2D | [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md)交付a8c04d22；[AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md)交付c141df8；其保存复用边界保原收据，不外推新G3豁免 |
| 3D表现/音画/查询 | [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md)93eaa907、[UI035](../../plans/archived/035-autographics-node3d-audio-hud-composition.md)5903839e、[UI039](../../plans/archived/039-autographics-node3d-queries-and-mixed-markers.md)8f577a34已实际落主/有canonical/旧树已清；同实现会话final限制如实记录，不称独立上下文pass |
| Gallery | [AG015](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md)80e0b65后当前活源**19/22**，余g04/2d、g04/3d、g05/3d；AG013 G11/3d b48b304及其它已迁入口保持固定消费 |
| 待实施内容 | [AG016完整G3](../../../../ag-examples/docs/plans/016-autographics-complete-3d-platformer.md)、[AG017 G05/3d](../../../../ag-examples/docs/plans/017-autographics-gallery-node3d-queries.md)均drafting/r1/0-10、无实施树；引擎源码/工具已有实交付，完整开工仍核各合同独立证据及相关hold，不改owner合同 |
| 本批新合同 | UI040编辑工程、AG018 G04、AG019 M2同版联验，均drafting/r1/0-10，只规划 |

权威分母仍为auto-lang [M2-01..09](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[G2-01..09/G3-01..08/原关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。Gallery19/22不是M2完成比例，游戏不计Gallery分母。

## 2. 当前下一批与依赖

[共同设计](m2-editor-and-release-convergence.md)承接上一批[三维查询/完整G3设计](node3d-queries-and-complete-game.md)，旧日期快照保历史。

| 线/责任仓 | 合同 | 可独立推进 | 完整后续硬门 |
|---|---|---|---|
| 引擎/auto-ui | [UI040](../../plans/040-autographics-node-editor-projects.md) | 原G04双维纯声明对象、项目感知编辑/保存副本交接；不等016/017 | T-02最多A/B全链先证；现有039静态路径足够就不新加profile |
| Demo/ag-examples | [AG018](../../../../ag-examples/docs/plans/018-autographics-gallery-node-editor-tutorials.md) | T-01原G04九义务/2D20与3D34/全部对象/旧锁/政策准备 | T-02完整迁移等040实交付/准确接口/独立证据；不等游戏或017 |
| 里程碑集成/ag-examples | [AG019](../../../../ag-examples/docs/plans/019-autographics-m2-unified-release-validation.md) | T-01原M2/全部入口变体/oracle/来源债/发行环境盘点 | T-02候选先证等040交付，T-03完整同版消费等016/017/018交付及必需独立证据 |

AG016/017可按原合同另行启动，UI040可独立推进，018准备可并行；019完整联验是后置门，不能提前运行后以缺产品删分母。共享gallery/packager/锁注册/引擎公共面owner短窗，各Plan一仓一树、独立target/材料scope；GPU/音频/窗口机器单实例，OS抢焦点自动化暂停不因新计划解除。本轮不自动实施。

## 3. G04不是简单的单维重写

原G04两个项目都含circle/quad与player/mob，2d另2个helper、3d另cylinder helper，都是behavior:null。039代码已有无state/Life时的无用户VM纯声明路径；尚须先证原对象/顺序与ID映射、项目自己的编辑/保存与消费身份。040优先复用，确有缺口才最小公共收口，不造重复渲染器/假角色或空Tick。

Node源/closure/生成基线是可再生初始来源；用户通过公共Open/SaveAs明确创建独立编辑副本，保存的是副本接受态，搬迁重开应保用户变更。拖动不暗改源/生成Ready，再生基线不静默覆盖用户副本；不在本期做反向AST写回。旧默认文档probe保作对照，原九require/20与34阶段及新Node项目自身导航/选择/实际Auto拖动/保存关开/SaveAs必须有真实证据，默认样例绿不代项目绿。未证必需OS项阻止相应AC关闭，不借旧pending豁免。

## 4. 内容计数、原预算与最后同版门

AG017贡献+1，AG018贡献+2，净增入口均0：按先后可为19→20→22或19→21→22，只有真实src/generated/metadata与验收完成才重数。AG016完整游戏独立于这些迁移，保原G3八项/关卡/三币/相机/胜败重开/必要模型clip与jump-land-coin/HUD；UI035夹具不能代正式游戏。

原32credits/32物理写/8状态Node/8slot/8pair/128行、深度/mesh/原源VM/time、raw8身份32MiB单件8MiB/CPU-GPU64MiB及动画/声音/HUD上限都不抬。多模块同世界共池，运行组织可与声明树不同；原地图最高合法轨迹与容量夹具分开，不能移币/漏声/停移动或Rust业务凑预算。旧异版交付和原UI011 no_go历史保留，不混作同一当前证据。

AG019固定一个实际引擎commit与显式语言/兄弟仓源集合、SDK681/两Cargo图锁/toolchain/runner/lower/resource-meta，所有活入口与两游戏显式同版消费但各自profile不变。原M2-01..09/G2九项/G3八项/全部G项变体oracle在该候选真实Auto/GPU/PCM/native/编辑/owner重验；包A→B无producer/target/cache/网络/其它项目仍运行/修改源码真变化/保存搬迁/坏件修复后下次再读Run。

## 5. 里程碑与来源门

AG019是一次明确的里程碑档，原全AG/性能资源样本按原条件在用户启动后的单独时段单实例跑；不把每次修复/review/merge又变全套。OS/声学/设备证据分层，原必需门未证则候选未就绪。独立上下文final不能被同实现会话重跑替代；相关上游缺证由原owner/合法交接核验，不重开档造pass。034相关hold/旧runner缺失/D-05批门/M1标签映射差异按真实适用性记录，不移动历史标签或假定11911a4a…=679681b3…。内容原保持项可按原收据/保留件直接复验，不要求伪造历史等价。

M2-ready必须原九项逐条实证、同版独立消费和独立final齐；本地候选/配方不等远端发布，推送/上传/标签另待明确指令。当前只新取040/018/019，不预取更多编号或承诺完成日期；不扩ACE/跨端/生产UI为前置。

每Plan临时材料仅plan-artifacts/plan-NNN/<repo>/，merge摘要折Plan/全部输出对账后删自有scope/构建物，仅具名未解决问题持有；不创建attachments/链接/派生JSON，清树先fresh guard，不覆盖其它路线WIP。
