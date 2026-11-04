# AGE M2游戏优先路线：内容闭环、编辑迁移与统一版本联验

2026-11-04，设计v8。AGE作者模型仍是AutoUI式Node/use、局部model/msg/on、view-2d/view-3d及具名资源/事件；桌面M2原两游戏/编辑/必要音画/资源离线/Gallery/统一最终引擎版本分母保持。ACE/AOT/ECS/CPC/跨端不新增前置。

> **命名与物理目录分开**：采用Auto Graphics Engine（AGE），详见[命名决议](age-naming-and-content-repository.md)。[内容仓020](../../../../ag-examples/docs/plans/020-age-examples-repository-rename.md)仅规划ag-examples→age-examples，默认019＋021及扩展M2必需门收口后维护窗口实施；保历史号与交付身份，不成为019前置。

> **2026-11-04 M2目标补齐**：用户明确基础UI、2D、3D必须在新语法下同应用组合，新增M2-10。核查显示底层合成已具备，Node普通UI view及到真实Auto事件/混合嵌入的桥尚缺。[补充设计与能力差距](m2-mixed-ui-2d-3d-composition.md)按[UI048基础UI](../../plans/048-age-node-basic-ui-and-auto-events.md)→[UI049混合引擎](../../plans/049-age-ui-2d-3d-composition.md)→[AG021综合应用](../../../../ag-examples/docs/plans/021-age-mixed-ui-2d-3d-application.md)规划，三份drafting/r1/0-10，暂不实施。019已完成旧合同，不改其归档/原候选/holds；020目录更名r2后置到扩展M2收口。

## 1. 当前真实状态

| 事项 | 已核事实及边界 |
|---|---|
| 完整2D | [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md)交付a8c04d22；[AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md)交付c141df8；其保存复用边界保原收据，不外推新G3豁免 |
| 3D表现/音画/查询 | [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md)93eaa907、[UI035](../../plans/archived/035-autographics-node3d-audio-hud-composition.md)5903839e、[UI039](../../plans/archived/039-autographics-node3d-queries-and-mixed-markers.md)8f577a34已实际落主/有canonical/旧树已清；同实现会话final限制如实记录，不称独立上下文pass |
| Gallery | [AG017](../../../../ag-examples/docs/plans/archived/017-autographics-gallery-node3d-queries.md)273b070、[AG018](../../../../ag-examples/docs/plans/archived/018-autographics-gallery-node-editor-tutorials.md)4cb297d已交付，当前活Node源**22/22**；12逻辑项/22入口不增，旧锁/来源边界保持 |
| 完整3D与原合同候选 | AG016交付55c39a2；[AG019](../../../../ag-examples/docs/plans/archived/019-autographics-m2-unified-release-validation.md)交付3daf2eb/独立R2/归档清理，原候选fc82061c；最终收据明确“不称M2完成”，原holds保持 |
| 编辑交付及新后继 | UI040交付fc82061c/canonical/独立final/清理齐，D-02 OS层15项仍pending；AG018/019原合同保持。UI043模型模板/UI044源码诊断本轮仅规划 |

权威分母仍为auto-lang [M2-01..09](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[G2-01..09/G3-01..08/原关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。Gallery22/22不是M2完成比例，游戏不计Gallery分母。

## 2. 当前M2内容线与依赖

[共同设计](m2-editor-and-release-convergence.md)承接上一批[三维查询/完整G3设计](node3d-queries-and-complete-game.md)，旧日期快照保历史。

| 线/责任仓 | 合同 | 可独立推进 | 完整后续硬门 |
|---|---|---|---|
| 引擎/auto-ui | [UI040](../../plans/archived/040-autographics-node-editor-projects.md)已交付 | editor-projects纯声明身份入口/项目化编辑7门/固定生成物/独立final齐 | AG018可固定消费；原OS层15项仍待真实证据，归档不等M2完成 |
| Demo/ag-examples | [AG018](../../../../ag-examples/docs/plans/archived/018-autographics-gallery-node-editor-tutorials.md)已交付 | G04双维Node源/原教程与包已落主 | 原OS层20/34分层及D-02具名边界按归档收据，不自动抹平 |
| 原合同里程碑集成/ag-examples | [AG019](../../../../ag-examples/docs/plans/archived/019-autographics-m2-unified-release-validation.md)已交付3daf2eb | 原九项/两游戏/22入口47变体在fc82061c候选对账，边界/holds按归档 | 不重开019；新增UI混用由048→049→AG021承担，不向旧合同塞任务 |

AG016/017/018/019均已合并，不接管/重开。019交付原合同候选不等扩展M2完成；共享gallery/packager/锁注册短窗，GPU/音频/焦点单实例及OS政策保持。本轮只核查/设计/规划048/049/AG021和020时机修订。

## 3. auto-ui可独立推进的可选后继

[模型对象组件与源码诊断设计](node-resource-components-and-source-diagnostics.md)新增两份仅规划合同：

| 合同 | 具体缺口与交付目标 | 与M2/并行关系 |
|---|---|---|
| [UI043](../../plans/043-autographics-node3d-model-object-templates.md) | driver已有Model运行实例，Node object3d仍只许builtin；补具名平色/opaque静态模型模板，真实出生运动销毁/多实例/非solid/资源owner原额 | 不等游戏或044；AG016保持035，未来内容显式消费另约 |
| [UI044](../../plans/044-autographics-node-source-diagnostics.md) | 多模块lower后回原module/Node/事件/helper；可选map/真实compile span/定位精度/接受源身份，无span runtime不造行 | 不等043、不换SDK/ACE；当前内容不升级debug模式 |

两者功能独立，共享load/translate/main/identity/Storage按owner短窗；T-02分别有界全链先证，旧profile字节/预算/原验收保持。不新增为当前M2前置，不把AG019候选自动改成新开发tip。041/042属其它UI组件store/重载路线，已占本地号，不重复编号或拿它们当AG整项前置。

## 4. G04不是简单的单维重写

原G04两个项目都含circle/quad与player/mob，2d另2个helper、3d另cylinder helper，都是behavior:null。039代码已有无state/Life时的无用户VM纯声明路径；尚须先证原对象/顺序与ID映射、项目自己的编辑/保存与消费身份。040优先复用，确有缺口才最小公共收口，不造重复渲染器/假角色或空Tick。

Node源/closure/生成基线是可再生初始来源；用户通过公共Open/SaveAs明确创建独立编辑副本，保存的是副本接受态，搬迁重开应保用户变更。拖动不暗改源/生成Ready，再生基线不静默覆盖用户副本；不在本期做反向AST写回。旧默认文档probe保作对照，原九require/20与34阶段及新Node项目自身导航/选择/实际Auto拖动/保存关开/SaveAs必须有真实证据，默认样例绿不代项目绿。未证必需OS项阻止相应AC关闭，不借旧pending豁免。

## 5. 内容计数、原预算与最后同版门

AG017贡献+1，AG018贡献+2均已实际交付，当前22/22，净增入口0。AG016完整游戏独立于迁移分母，保原G3八项/关卡/三币/相机/胜败重开/必要模型clip与jump-land-coin/HUD；UI035夹具不能代正式游戏。

原32credits/32物理写/8状态Node/8slot/8pair/128行、深度/mesh/原源VM/time、raw8身份32MiB单件8MiB/CPU-GPU64MiB及动画/声音/HUD上限都不抬。多模块同世界共池，运行组织可与声明树不同；原地图最高合法轨迹与容量夹具分开，不能移币/漏声/停移动或Rust业务凑预算。旧异版交付和原UI011 no_go历史保留，不混作同一当前证据。

AG019固定一个实际引擎commit与显式语言/兄弟仓源集合、SDK681/两Cargo图锁/toolchain/runner/lower/resource-meta，所有活入口与两游戏显式同版消费但各自profile不变。原M2-01..09/G2九项/G3八项/全部G项变体oracle在该候选真实Auto/GPU/PCM/native/编辑/owner重验；包A→B无producer/target/cache/网络/其它项目仍运行/修改源码真变化/保存搬迁/坏件修复后下次再读Run。

## 6. 里程碑与来源门

AG019是一次明确的里程碑档，原全AG/性能资源样本按原条件在用户启动后的单独时段单实例跑；不把每次修复/review/merge又变全套。OS/声学/设备证据分层，原必需门未证则候选未就绪。独立上下文final不能被同实现会话重跑替代；相关上游缺证由原owner/合法交接核验，不重开档造pass。034相关hold/旧runner缺失/D-05批门/M1标签映射差异按真实适用性记录，不移动历史标签或假定11911a4a…=679681b3…。内容原保持项可按原收据/保留件直接复验，不要求伪造历史等价。

扩展M2-ready必须原M2-01..09＋新增M2-10逐条实证、必要同版独立消费/独立final齐；019原九项证据按其真实身份保留，不能用旧hash冒充049新候选绿。本地候选不等远端发布，上传/tag另待指令。UI043/044仍可选，不扩ACE/跨端/完整生产UI为前置。

每Plan临时材料仅plan-artifacts/plan-NNN/<repo>/，merge摘要折Plan/全部输出对账后删自有scope/构建物，仅具名未解决问题持有；不创建attachments/链接/派生JSON，清树先fresh guard，不覆盖其它路线WIP。

## 7. 基础UI混用补齐与当前可执行边界

048补row/column/text/button的Node view、真实Auto事件与绑定，UI-only独立验证；049完整接线等048实交付，复用AGP单设备/合成/输入桥，补UI布局内view-2d/view-3d同时呈现、实例一次更新与焦点/裁剪；AG021正式接线等049实际交付，内容在新增showcases/ui-2d-3d，原Gallery12/22/47与两游戏分母不变。AG021需新混合应用双向交互/原生窗口/离线包与原九项新候选兼容矩阵；旧证据只有代码/输入/配置等价可证才具名复用。

未来启动后048可先推进；049 T-01与AG021 T-01/02可并行只读准备，后续分别等实际交付，不追tip。043在途/044可选与本补齐链无强依赖，但同文件写面先核owner短窗。原32credits/8状态Node/128行等预算不抬，原OS政策与GPU/焦点单实例保持，必需真输入未证不充pass。020是019/021后的维护性更名，不成为本链前置。
