# AGE M2游戏优先路线：内容闭环、编辑迁移与统一版本联验

> **2026-11-05 再核现场**：054已有唯一实现树和1f3be333代码提交，头部状态/进度尚未同步执行记录，按实施中/未交付报告；AG021原owner2/10等054。055/AG022仍草案。下一批[058/059/AG023](m3-next-wave-execution-order.md)仅规划M3组合/研究/小例，不新增M2前置；[全路线状态与顺序](plan-status-and-execution-order-2026-11-05.md)逐项登记。

2026-11-05，设计v10。AGE作者模型仍是AutoUI式Node/use、局部model/msg/on、view-2d/view-3d及具名资源/事件；桌面M2原两游戏/编辑/必要音画/资源离线/Gallery/统一最终引擎版本分母保持。ACE/AOT/ECS/CPC/跨端不新增前置。

> **命名与物理目录分开**：采用Auto Graphics Engine（AGE），详见[命名决议](age-naming-and-content-repository.md)。[内容仓020](../../../../ag-examples/docs/plans/020-age-examples-repository-rename.md)仅规划ag-examples→age-examples，默认019＋021及扩展M2必需门收口后维护窗口实施；保历史号与交付身份，不成为019前置。

> **2026-11-05 实际交付后的收口路线**：048基础UI（cd1e161a）/049有限root混合（b7f01267）已交付/归档，尚缺AG021原要求的普通use区域组件与条件挂卸/代次，且两份final均记同实现会话。详见[当前缺口与下一批](m2-closure-and-m3-first-wave.md)。本轮规划UI054补公共混合接线并独立核验新候选，AG021修订r2固定消费054实际交付，原G-M01..09/验收分母保持；019历史不改，020仅M2必需门闭合后更名维护。UI055/AG022可并行准备M3研究，不成为M2前置；均暂不实施。

## 1. 当前真实状态

| 事项 | 已核事实及边界 |
|---|---|
| 完整2D | [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md)交付a8c04d22；[AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md)交付c141df8；其保存复用边界保原收据，不外推新G3豁免 |
| 3D表现/音画/查询 | [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md)93eaa907、[UI035](../../plans/archived/035-autographics-node3d-audio-hud-composition.md)5903839e、[UI039](../../plans/archived/039-autographics-node3d-queries-and-mixed-markers.md)8f577a34已实际落主/有canonical/旧树已清；同实现会话final限制如实记录，不称独立上下文pass |
| Gallery | [AG017](../../../../ag-examples/docs/plans/archived/017-autographics-gallery-node3d-queries.md)273b070、[AG018](../../../../ag-examples/docs/plans/archived/018-autographics-gallery-node-editor-tutorials.md)4cb297d已交付，当前活Node源**22/22**；12逻辑项/22入口不增，旧锁/来源边界保持 |
| 完整3D与原合同候选 | AG016交付55c39a2；[AG019](../../../../ag-examples/docs/plans/archived/019-autographics-m2-unified-release-validation.md)交付3daf2eb/独立R2/归档清理，原候选fc82061c；最终收据明确“不称M2完成”，原holds保持 |
| 编辑交付及新后继 | UI040交付fc82061c/canonical/独立final/清理齐，D-02 OS层15项仍pending；AG018/019原合同保持。UI043模型模板/UI044源码诊断均已交付归档/清理 |

权威分母仍为auto-lang [M2-01..09](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[G2-01..09/G3-01..08/原关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。Gallery22/22不是M2完成比例，游戏不计Gallery分母。

## 2. 当前M2内容线与依赖

[共同设计](m2-editor-and-release-convergence.md)承接上一批[三维查询/完整G3设计](node3d-queries-and-complete-game.md)，旧日期快照保历史。

| 线/责任仓 | 合同 | 可独立推进 | 完整后续硬门 |
|---|---|---|---|
| 引擎/auto-ui | [UI040](../../plans/archived/040-autographics-node-editor-projects.md)已交付 | editor-projects纯声明身份入口/项目化编辑7门/固定生成物/独立final齐 | AG018可固定消费；原OS层15项仍待真实证据，归档不等M2完成 |
| Demo/ag-examples | [AG018](../../../../ag-examples/docs/plans/archived/018-autographics-gallery-node-editor-tutorials.md)已交付 | G04双维Node源/原教程与包已落主 | 原OS层20/34分层及D-02具名边界按归档收据，不自动抹平 |
| 原合同里程碑集成/ag-examples | [AG019](../../../../ag-examples/docs/plans/archived/019-autographics-m2-unified-release-validation.md)已交付3daf2eb | 原九项/两游戏/22入口47变体在fc82061c候选对账，边界/holds按归档 | 不重开019；新增UI混用由048/049已交付切片→054→AG021 r2承担，不向旧合同塞任务 |

AG016/017/018/019均已合并，不接管/重开。019交付原合同候选不等扩展M2完成；共享gallery/packager/锁注册短窗，GPU/音频/焦点单实例及OS政策保持。本轮只规划054/055/AG022及AG021 r2前置衔接。

## 3. 已交付可选后继与本轮并行研究

UI043模型模板/UI044源码诊断已交付（044 delivery 9bf9874a），实际支持/来源/预算依各归档和canonical；旧AG016/019不自动升级锁。这两份不再计未完成，041/042与在途052/053属于其它UI路线，不接管或复用其号。

新[UI055](../../plans/archived/055-age-m3-topdown2d-feasibility-and-capacity.md)固定已交付源研究俯视移动/交互/场景链与容量；[AG022](../../../../ag-examples/docs/plans/022-age-m3-farming-slice-and-threejs-inventory.md)整理国风首切片/素材及固定three.js全目录差距。研究准备可与054/AG021并行，022 T-06等055实际结果；go/bounded-go/no-go均须真实证据，不把研究归档当产品能力。共享源只读，材料/target独占，GPU/原生焦点错峰。

## 4. G04不是简单的单维重写

原G04两个项目都含circle/quad与player/mob，2d另2个helper、3d另cylinder helper，都是behavior:null。039代码已有无state/Life时的无用户VM纯声明路径；尚须先证原对象/顺序与ID映射、项目自己的编辑/保存与消费身份。040优先复用，确有缺口才最小公共收口，不造重复渲染器/假角色或空Tick。

Node源/closure/生成基线是可再生初始来源；用户通过公共Open/SaveAs明确创建独立编辑副本，保存的是副本接受态，搬迁重开应保用户变更。拖动不暗改源/生成Ready，再生基线不静默覆盖用户副本；不在本期做反向AST写回。旧默认文档probe保作对照，原九require/20与34阶段及新Node项目自身导航/选择/实际Auto拖动/保存关开/SaveAs必须有真实证据，默认样例绿不代项目绿。未证必需OS项阻止相应AC关闭，不借旧pending豁免。

## 5. 内容计数、原预算与最后同版门

AG017贡献+1，AG018贡献+2均已实际交付，当前22/22，净增入口0。AG016完整游戏独立于迁移分母，保原G3八项/关卡/三币/相机/胜败重开/必要模型clip与jump-land-coin/HUD；UI035夹具不能代正式游戏。

原32credits/32物理写/8状态Node/8slot/8pair/128行、深度/mesh/原源VM/time、raw8身份32MiB单件8MiB/CPU-GPU64MiB及动画/声音/HUD上限都不抬。多模块同世界共池，运行组织可与声明树不同；原地图最高合法轨迹与容量夹具分开，不能移币/漏声/停移动或Rust业务凑预算。旧异版交付和原UI011 no_go历史保留，不混作同一当前证据。

AG019固定一个实际引擎commit与显式语言/兄弟仓源集合、SDK681/两Cargo图锁/toolchain/runner/lower/resource-meta，所有活入口与两游戏显式同版消费但各自profile不变。原M2-01..09/G2九项/G3八项/全部G项变体oracle在该候选真实Auto/GPU/PCM/native/编辑/owner重验；包A→B无producer/target/cache/网络/其它项目仍运行/修改源码真变化/保存搬迁/坏件修复后下次再读Run。

## 6. 里程碑与来源门

AG019是一次明确的里程碑档，原全AG/性能资源样本按原条件在用户启动后的单独时段单实例跑；不把每次修复/review/merge又变全套。OS/声学/设备证据分层，原必需门未证则候选未就绪。独立上下文final不能被同实现会话重跑替代；相关上游缺证由原owner/合法交接核验，不重开档造pass。034相关hold/旧runner缺失/D-05批门/M1标签映射差异按真实适用性记录，不移动历史标签或假定11911a4a…=679681b3…。内容原保持项可按原收据/保留件直接复验，不要求伪造历史等价。

扩展M2-ready必须原M2-01..09＋新增M2-10逐条实证、必要同版独立消费/独立final齐；019原九项证据按其真实身份保留，不能用旧hash冒充054新候选绿。本地候选不等远端发布，上传/tag另待指令。UI043/044已交付且保持可选，不扩ACE/跨端/完整生产UI为前置。

每Plan临时材料仅plan-artifacts/plan-NNN/<repo>/，merge摘要折Plan/全部输出对账后删自有scope/构建物，仅具名未解决问题持有；不创建attachments/链接/派生JSON，清树先fresh guard，不覆盖其它路线WIP。

## 7. 基础UI混用补齐与当前可执行边界

048基础UI与049root有限混合已交付；049 canonical明确非root区域/条件区域挂卸不支持，Host还有固定名称/实例0路径。AG021原G-M07/AC-04仍需寿命和另一owner存活，不能删条件操作或改成根展开来绕过。[054](../../plans/054-age-mixed-regions-components-and-lifecycle.md)补metadata驱动区域/组件实例/挂卸代次/原生输入，在独立上下文核验新候选和048/049必需复用面；不倒签旧归档。

AG021原owner已记T-01/02准备完成、executing/r2/2-10，T-03完整接线等054实际交付/独立final/canonical与工具来源。不追tip，showcases/ui-2d-3d独立工程/包不增Gallery分母；最终旧九项、新M2-10、两游戏与22入口47变体逐项对账，真实受影响门重验、可证等价具名复用；任何必需原生/来源/批门未证都不称M2完成。

预算分profile：旧game32credits/物理32/8状态Node/128行保持；048/049 UI为1000 VM steps/回调、累计100k、8实例/32元素每def/4文字槽×64B/队列64；054拟活区域全应用合计≤4。不同单位不互换，不给每模块复制额度。OS政策/机器GPU焦点单实例保持。AG020只是收口后的更名维护，不是功能前置。

## 8. M2之后：M3增强2D/3D，M4正式融合AutoUI

2026-11-04用户调整里程碑：**M3优先增强AGE的2D/2DHD与复杂3D能力，原AutoUI/ACE正式融合任务顺延M4**。

- [M3能力路线](m3-2d-3d-capability-roadmap.md)：以俯视角2D/2DHD国风种田承载游戏可玩切片，和three.js固定版本逐项能力矩阵驱动引擎/游戏/示例并行准备；首轮由055/AG022研究冻结规则候选、画面选项、规模和固定目录差距，后续产品批次另立合同，不将完整商业游戏或全部上游目录默认为已交付。
- [M4融合路线](m4-autoui-integration-roadmap.md)：保留Schema方言成熟且旧Auto编译器替换实际启动的前置，届时统一接入AutoUI/AGE并从examples/ui/001按能力依赖推进全部UI Demo原行为兼容。

M3不因ACE正式集成等待而停在原地；055/AG022研究准备可先行，正式产品能力依阶段和具体合同核M2收口/真实前置。M2本轮只新增054和修订AG021 r2，不回改048/049或019归档/原分母。020仍在M2收口后维护窗口，不等M4。当前仅规划三份新合同，未建树/运行/实现/发布。

2026-11-05 UI055交付更新：本页055链接已指向同号研究归档；[当前可行性与容量结论](../../specs/auto-graphics/topdown2d-feasibility.md)给出通用无地面原型 bounded-go、完整铺地图元/角色同回调交互 no-go，以及数组编码与原预算边界。055的规划状态已由实际研究收据替代；AG022/058及M2依赖仍按各自合同和实交付判断，不将研究当完整产品能力。
