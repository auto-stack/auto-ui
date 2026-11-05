# AGE全计划状态与执行顺序（2026-11-05）

本表范围为迁仓后auto-ui的AGE合同与ag-examples全部内容合同，不把其它AutoUI路线或auto-lang历史号重复计数。历史861/874/875/877/878/879/880及882/883映射仍按原档；本表不重写历史。截止本轮观察，两仓共57份当前身份合同：**49份已交付/归档、8份未完成**；其中研究交付不等于产品能力。没有将源码迁移22/22或Plan总数当M2完成比例。

证据口径：本轮读当前合同/归档最终收据和相关canonical，核已列交付提交均为各仓HEAD祖先，并核当前注册树/实现提交。没有重跑49份历史GPU或逐AC重审；独立复审/原生OS/历史来源的已知限制照原收据保留。旧部分frontmatter仍停5/10、6/10或7/10，按最终交付/落地主张与实仓核对，不能据勾选alone推断；也不因此重开归档。UI039主目录旧draft与归档同身份重复，只计已交付一次，不代owner删除旧影子。

## 1. 当前未完成合同（8份）

| 合同 | 实际状态/证据 | 下一步与硬前置 |
|---|---|---|
| [UI054混合区域组件/寿命](../../plans/054-age-mixed-regions-components-and-lifecycle.md) | 实施中、未交付；唯一ui-054树/branch，已有1f3be333实现提交；主头部drafting/current_step与执行记录不一致，T-01/02有实录；不代owner改状态 | 原owner继续后续接线/原生与预算门、独立final/冻结/merge；其交付前AG021不能进入T-03 |
| [UI055俯视二维容量研究](../../plans/archived/055-age-m3-topdown2d-feasibility-and-capacity.md) | r1研究交付，独立final pass；归档/清理见同号收据 | [canonical结论](../../specs/auto-graphics/topdown2d-feasibility.md)：无地面原型bounded-go；铺地/角色组合no-go，供AG022消费 |
| [UI058俯视控制/查询/精灵组合](../../plans/058-age-topdown2d-queries-and-sprites.md) | 本批drafting/r1/0-10，无树/实施 | T-01只读准备可先；T-02等055/AG022实际输入及M2必需门真实收口；先证不过或需提预算须replan |
| [UI059三维方向光/阴影作者链研究](../../plans/archived/059-age-node3d-lighting-and-shadow-feasibility.md) | 已交付研究/r1/10-10；bounded-go，独立final pass | Auto动态标量→受检研究桥→GPU已证；正式Light能力另立约，非生产Host/原生输入 |
| [AG020内容仓更名](../../../../ag-examples/docs/plans/020-age-examples-repository-rename.md) | drafting/r2/0-10，无实现树 | M2所有必需门实际收口后的维护窗口；不是功能门、不是058/059前置 |
| [AG021混合UI/2D/3D综合应用](../../../../ag-examples/docs/plans/021-age-mixed-ui-2d-3d-application.md) | executing/r2/2-10，原owner已有ag-021唯一树，T-01/02完成记录；工作会话曾标等待/blocked | T-03等054实际交付/独立final/canonical及048/049复用必需面独立核验；保原G-M及M2-01..10/两游戏/22入口47变体 |
| [AG022国风首切片/three.js盘点](../../../../ag-examples/docs/plans/022-age-m3-farming-slice-and-threejs-inventory.md) | drafting/r1/0-10，无ag-022树或研究结论 | T-01..05可独立准备，T-06等055实际研究交付（含no-go） |
| [AG023俯视组合小例](../../../../ag-examples/docs/plans/023-age-m3-topdown2d-example.md) | 本批drafting/r1/0-10，无树/实施 | T-01/02准备可先；T-03等058实际交付/独立final/准确来源、AG022实际输入及M2收口 |

## 2. auto-ui已交付AGE合同（30份）

| 合同 | 范围 | 交付提交/当前边界 |
|---|---|---|
| [UI001](../../plans/archived/001-autographics-runtime-object-lifecycle.md) | AutoGraphics有限运行对象身份与生命周期 | 已交付/归档 `537122b`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI002](../../plans/archived/002-autographics-trigger-events-and-round-restart.md) | AutoGraphics有限触发事件与新局隔离（引擎） | 已交付/归档 `0ee5ad0`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI004](../../plans/archived/004-autographics-public-images-and-sprite-frames.md) | AutoGraphics公共图片资源与2D精灵帧 | 已交付/归档 `e0fce13`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI005](../../plans/archived/005-autographics-textured-models-and-uv.md) | AutoGraphics有限纹理模型与UV资源闭包 | 已交付/归档 `351919c`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI006](../../plans/archived/006-autographics-node-animation-and-visual-facing.md) | AutoGraphics无蒙皮节点动画与视觉朝向 | 已交付/归档 `c681663`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI008](../../plans/archived/008-plan004-identity-closure.md) | PLAN004交付身份闭包修复 | 已交付/归档 `0261ae9`；固定消费身份修复属于AGE，不因文件名不含autographics而漏计 |
| [UI011](../../plans/archived/011-autographics-g2-full-loop-budget-feasibility.md) | AutoGraphics二维完整玩法整链预算验证 | 已交付/归档 `c4143af`；研究no_go_current_profile保留，不抵产品可行性 |
| [UI012](../../plans/archived/012-autographics-public-audio-and-accepted-cues.md) | AutoGraphics公共音频资源与接受后一次播放 | 已交付/归档 `78632af`；32等原包络/native声学边界按收据 |
| [UI013](../../plans/archived/013-autographics-public-hud-and-presentation-state.md) | AutoGraphics公共HUD与共同呈现状态 | 已交付/归档 `8b1e799`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI016](../../plans/archived/016-autographics-node2d-minimal-validation.md) | AutoGraphics Node2D最小真实执行验证 | 已交付/归档 `932bd5f`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI017](../../plans/archived/017-autographics-g11-2d-node-authoring.md) | AutoGraphics 完整G11二维Node作者能力 | 已交付/归档 `9a8a5be`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI019](../../plans/archived/019-autographics-node2d-scenes-character-camera.md) | AutoGraphics Node2D场景层级与走跳相机 | 已交付/归档 `c93adb7`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI020](../../plans/archived/020-autographics-stateful-node-components.md) | AutoGraphics有状态Node组件与局部事件组合 | 已交付/归档 `16ddfa0`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI021](../../plans/archived/021-autographics-node2d-queries.md) | AutoGraphics Node二维segment与sweep查询作者能力 | 已交付/归档 `1bec52f`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI022](../../plans/archived/022-autographics-node2d-sprites-and-frame-animation.md) | AutoGraphics Node二维精灵资源与局部帧动画 | 已交付/归档 `c6aa63a`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI023](../../plans/archived/023-autographics-node2d-audio-and-hud.md) | AutoGraphics二维Node声音与公共HUD作者能力 | 已交付/归档 `32d7a81`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI024](../../plans/archived/024-autographics-node3d-foundation.md) | AutoGraphics三维声明静态资源与有状态组件基础 | 已交付/归档 `64166db`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI025](../../plans/archived/025-autographics-node-presentation-services.md) | AutoGraphics声音与HUD通用Node接线收口 | 已交付/归档 `e9cc929`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI027](../../plans/archived/027-autographics-node3d-character-and-runtime-camera.md) | AutoGraphics三维角色与运行相机Node接线 | 已交付/归档 `f3dea0d`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI028](../../plans/archived/028-autographics-node3d-instances-triggers-and-rounds.md) | AutoGraphics三维实例触发与新局Node接线 | 已交付/归档 `ebac1cd`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI030](../../plans/archived/030-autographics-node3d-symbolic-component-bindings.md) | AutoGraphics三维具名引用与跨组件接线收口 | 已交付/归档 `804d7303`；独立上下文final已补录，历史同会话记录保留 |
| [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md) | AutoGraphics完整G2精灵对象音画组合与预算收口 | 已交付/归档 `a8c04d22`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md) | AutoGraphics3D纹理资源与节点动画作者接线 | 已交付/归档 `93eaa907`；原final为同实现会话，必需复用面独立核验/hold未自动关闭 |
| [UI035](../../plans/archived/035-autographics-node3d-audio-hud-composition.md) | AutoGraphics3D声音HUD与完整G3组合接线 | 已交付/归档 `5903839e`；同实现会话final及声学/批门hold按原记录 |
| [UI039](../../plans/archived/039-autographics-node3d-queries-and-mixed-markers.md) | AutoGraphics三维查询与有限二维三维组合作者接线 | 已交付/归档 `8f577a34`；同实现会话final/旧OS边界保留；主目录旧draft不是第二计划 |
| [UI040](../../plans/archived/040-autographics-node-editor-projects.md) | AutoGraphicsNode混合维度编辑工程与项目化验收 | 已交付/归档 `fc82061c`；原生编辑OS层具名pending不抵当前必需门 |
| [UI043](../../plans/archived/043-autographics-node3d-model-object-templates.md) | AutoGraphicsNode三维模型资源对象模板接线 | 已交付/归档 `ba080e27`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI044](../../plans/archived/044-autographics-node-source-diagnostics.md) | AutoGraphicsNode多模块源码定位与分层诊断 | 已交付/归档 `9bf9874a`；原支持/预算与清理边界依归档/canonical，非全M2结论 |
| [UI048](../../plans/archived/048-age-node-basic-ui-and-auto-events.md) | AGE基础UI Node与真实Auto事件接线 | 已交付/归档 `cd1e161a`；有限基础UI已交；同会话final限制，当前复用核验归054/AG021 |
| [UI049](../../plans/archived/049-age-ui-2d-3d-composition.md) | AGE基础UI与二维三维Node同应用组合 | 已交付/归档 `b7f01267`；root有限混合已交；区域组件/挂卸缺口归054，同会话final限制保留 |

## 3. ag-examples已交付合同（001..019，共19份）

| 合同 | 范围 | 交付提交/当前边界 |
|---|---|---|
| [AG001](../../../../ag-examples/docs/plans/archived/001-autographics-gallery-runtime-objects.md) | AutoGraphics Gallery G11对象触发与新局示例 | 已交付/归档 `cb4e23c`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG002](../../../../ag-examples/docs/plans/archived/002-autographics-gallery-sprites-and-frame-animation.md) | AutoGraphics Gallery G09精灵与帧动画 | 已交付/归档 `d1b4f9b`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG003](../../../../ag-examples/docs/plans/archived/003-autographics-gallery-textured-models.md) | AutoGraphics Gallery G10a纹理模型与UV | 已交付/归档 `4816807`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG004](../../../../ag-examples/docs/plans/archived/004-autographics-gallery-node-animation-and-facing.md) | AutoGraphics Gallery G10b节点动画与视觉朝向 | 已交付/归档 `a279797`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG005](../../../../ag-examples/docs/plans/archived/005-autographics-gallery-audio-actions.md) | AutoGraphics Gallery G12a动作与一次音效 | 已交付/归档 `5996d8f`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG006](../../../../ag-examples/docs/plans/archived/006-autographics-gallery-event-hud.md) | AutoGraphics Gallery G12b事件HUD与G12新Node作者路线 | 已交付/归档 `e574b58`；声学具名边界保留 |
| [AG007](../../../../ag-examples/docs/plans/archived/007-autographics-gallery-g11-2d-node-migration.md) | AutoGraphics Gallery G11二维Node源码迁移 | 已交付/归档 `939f20c`；旧runner负报告具名待判，不默改来源 |
| [AG008](../../../../ag-examples/docs/plans/archived/008-autographics-gallery-node2d-first-wave.md) | AutoGraphics Gallery二维新作者形式首波迁移 | 已交付/归档 `8a01a34`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG009](../../../../ag-examples/docs/plans/archived/009-autographics-gallery-node2d-queries.md) | AutoGraphics Gallery G05二维查询新Node作者迁移 | 已交付/归档 `062081d`；OS阶段pending按原记录 |
| [AG010](../../../../ag-examples/docs/plans/archived/010-autographics-gallery-node2d-sprites-and-frame-animation.md) | AutoGraphics Gallery G09精灵与角色帧动画新Node作者迁移 | 已交付/归档 `8bfddca`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG011](../../../../ag-examples/docs/plans/archived/011-autographics-gallery-node3d-basics.md) | AutoGraphics Gallery G02与G03三维内容Node作者迁移 | 已交付/归档 `f35236dc`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG012](../../../../ag-examples/docs/plans/archived/012-autographics-gallery-node3d-character-and-cameras.md) | AutoGraphics Gallery G07与G08三维角色相机Node作者迁移 | 已交付/归档 `05f1b498`；原玩法/oracle/锁与许可、包边界按归档保持 |
| [AG013](../../../../ag-examples/docs/plans/archived/013-autographics-gallery-g11-3d-node-migration.md) | AutoGraphics Gallery G11三维具名组件作者迁移 | 已交付/归档 `b48b304`；二维/三维原锁及旧runner持有保持 |
| [AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md) | AutoGraphics完整2D平台游戏 | 已交付/归档 `c141df8`；完整2D游戏已交，不计Gallery分母 |
| [AG015](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md) | AutoGraphicsGalleryG10纹理动画双入口Node作者迁移 | 已交付/归档 `80e0b65`；内容/上游原final同会话限制与hold保留 |
| [AG016](../../../../ag-examples/docs/plans/archived/016-autographics-complete-3d-platformer.md) | AutoGraphics完整三维平台探索收集游戏 | 已交付/归档 `55c39a2`；完整3D游戏已交，不计Gallery分母 |
| [AG017](../../../../ag-examples/docs/plans/archived/017-autographics-gallery-node3d-queries.md) | AutoGraphicsGalleryG05三维查询Node迁移 | 已交付/归档 `273b070`；原二维锁/旧runner与OS边界保持 |
| [AG018](../../../../ag-examples/docs/plans/archived/018-autographics-gallery-node-editor-tutorials.md) | AutoGraphicsGalleryG04双维编辑教程Node迁移 | 已交付/归档 `4cb297d`；双维编辑教程交付，OS20/34分层pending保留 |
| [AG019](../../../../ag-examples/docs/plans/archived/019-autographics-m2-unified-release-validation.md) | AutoGraphicsM2统一引擎版本与独立发行联验 | 已交付/归档 `3daf2eb`；统一旧候选fc82061c已交，收据明确不称完整M2；原holds保留 |

## 4. 实际执行顺序

1. M2主线：054原owner完成实现/必要原生证据/独立final/冻结落主 → AG021从T-03消费准确交付，完成新混合应用/旧项目同版兼容/离线包及M2-01..10所有必需门 → 给出M2真实结论。不能用同实现会话重跑补独立身份，未证OS/历史来源/批门按适用性具名对账，不私批豁免。
2. 并行研究：055与059可独立研究，AG022 T-01..05可并行准备。AG022 T-06实际消费055结果；go/bounded-go/no-go如实，不能把未测容量写成可用。
3. 下一批二维：058 T-01、AG023 T-01/02准备可先行。正式顺序为055＋AG022研究/需求输入，以及M2真实收口 → 058 T-02最小全链先证/实现/独立final/交付 → AG023 T-03正常作者接线/真实运行/冷包/独立交付。
4. AG020在M2必需门收口后另择维护窗口更名；它不阻M3研究/能力，也不需等待M4。若与内容后继碰窗，先冻结旧/新根引用和owner，保历史ID/SHA/交付身份。
5. 059结论和AG022完整固定目录差距形成后，再确定正式Light/材质/阴影能力及对应内容合同；本轮不预取未知后继号、不认领three.js覆盖或完整种田游戏。M4仍等ACE Schema成熟与旧编译器替换实际启动。

并行不等于共享一棵树或同时抢GPU/焦点：每Plan每仓唯一树、target/报告/plan-artifacts子scope独占；共享AST/load/translate/资源/Host按owner短窗，真实GPU/窗口焦点/音频重门机器单实例。新计划只是规划；UI054与AG021已有实现不由本会话接管。

## 5. 号源、排除项与局限

UI056/057为同时取号的其它AutoUI路线（多文件reload/schema与Vue Store隔离），052/053等也不计AGE；本批锁号核主/归档/所有注册树后取058/059，next060。AG取023，next024。跨仓同号不是共享身份或材料目录owner。

048/049和旧部分计划已交付不等所有独立证据债自动消除；完整M2仍未认领。现有静态核查/归档读取不替代新Plan work/review的真实运行。状态可能随其它会话继续变化；本表为本轮证据快照，最新合同与实际树/提交高于日期旧文字。未修改任何在途合同/归档/canonical/SDK/生成物/实现树，不创建agent。

统计校正：本轮最终逐项检查补入UI008（交付0261ae9为主HEAD祖先），故总数为34个UI AGE身份＋23个内容身份＝57，49交付/8未完。不能只按文件名含autographics/age过滤整条路线；UI008归档/提交/历史身份不改。

2026-11-05 UI055交付更新：本页055链接已指向同号研究归档；[当前可行性与容量结论](../../specs/auto-graphics/topdown2d-feasibility.md)给出通用无地面原型 bounded-go、完整铺地图元/角色同回调交互 no-go，以及数组编码与原预算边界。055的规划状态已由实际研究收据替代；AG022/058及M2依赖仍按各自合同和实交付判断，不将研究当完整产品能力。
