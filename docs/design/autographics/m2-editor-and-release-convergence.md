# AG M2下一阶段：编辑教程与统一版本发行收口

> 2026-11-04现场更新：UI040已交付fc82061c/独立final/canonical/清理齐，原OS层15项仍D-02 pending；AG016已executing3/10固定035，AG017/018/019草案。UI043/044仅为可选引擎后继，不新增本批M2前置，见[资源组件/诊断设计](node-resource-components-and-source-diagnostics.md)。下文保2026-11-03立约快照，实际消费以各合同/交付证据为准。

2026-11-03 v1，target-state。用户要求提出并规划下一批；[UI040](../../plans/archived/040-autographics-node-editor-projects.md)、[AG018](../../../../ag-examples/docs/plans/018-autographics-gallery-node-editor-tutorials.md)、[AG019](../../../../ag-examples/docs/plans/019-autographics-m2-unified-release-validation.md)均drafting/r1/0-10，本轮不实施/建树/探针/素材/构建/GPU/OS自动化/agent/发布。保原预算/玩法/全部验收，不为凑三份任务另造功能。

## 1. 当前证据与剩余分母

UI035已实际交付5903839e/canonical node3d-audio-hud-composition.md（SHA d7272516…），UI039已交付8f577a34/canonical node3d-queries-and-mixed-markers.md（SHA117bb3da…）；旧实施树均已清，不重建。039归档metadata current_step6与merge收据delivered并存，按落主/源Spec/实际清理核实，不据计数单独推断未交付。UI034/035/039与AG015 final收据同实现会话复核，不能称独立上下文final；相关后继实际证据仍依合同，不据导航“实质解除”自动批准例外。

AG014完整2D已交付c141df8；AG016完整G3与AG017 G05/3d目前均drafting/r1/0-10、无实施树；本轮不改其合同/接管实现。Gallery按metadata/generated/Node活源仍**19/22**，未迁g04/2d、g04/3d、g05/3d。AG017贡献+1，AG018贡献+2，无新入口；两者均实际交付才可能22/22，游戏不计此分母。源码迁完也不等M2完成，原M2-01..09与两游戏/统一最终版本/独立发行尚须联验。

## 2. 本批三个合同与阶段依赖

| 责任线 | 合同 | 独立可做 | 后续硬门 |
|---|---|---|---|
| auto-ui公共编辑能力 | UI040 Node混合维度编辑工程/项目化验收 | 019/024/039实交付基础上，完整原G04纯声明双维对象、项目自身编辑/SaveAs和基线副本交接；不等AG016/017 | T-02最多A/B全链先证；能复用039静态路径就不新增profile |
| ag-examples Demo | AG018 G04两教程Node迁移 | T-01原九义务/20与34阶段/场景/旧锁/OS政策准备，不等016/017 | T-02完整迁移固定040实际交付/准确接口/独立证据 |
| ag-examples里程碑集成 | AG019 M2同版与独立发行联验 | T-01盘点全部原M2/入口变体/oracle/来源/发行环境 | T-02候选先证等040实际交付；T-03完整消费与联验等016/017/018实交付及必需独立证据 |

AG016游戏、AG017查询Demo继续各自原合同；UI040及AG018准备可独立推进，AG019主要为后置收口而非立即全量跑门。共享gallery/packager/锁注册与引擎公共文件owner短窗单写；每Plan每仓唯一树/target/材料scope，真实GPU/音频/窗口单实例。不得把未来040开发tip写入018/019消费锁。

## 3. G04真实缺口：已有静态双维机制，尚缺项目化交付证据

实际原G04/2d含6对象：circle/quad、player/mob、两个二维helper；G04/3d含5对象：circle/quad、player/mob、cylinder helper。两者都本来混合两维、无group/asset/behavior，不能按入口名删另一维，不能用假character/camera代player/mob视觉叶。

已核039 `lower_queries3d.rs`复用components3d，has_behavior只取stateful/Life，因此存在behavior:null静态路径。039根二维叶放在三维行后，默认dialect仍拒同Node双view；原G04的顺序/ID/像素/选取/默认校准并非一句“支持双出口”就证明保持。UI040先证A复用这一无用户脚本VM路线和019/024叶；只有确有具名缺口才选B新增有限editor-scene或等价公共适配。不为命名别扭造新渲染器，不全面放开所有旧profile。

旧verify_g04为二维实际项目20阶段；三维能力probe作用默认文档（旧注释17，但实际checker为34），再给项目smoke。原九require必须保持，同时补**新Node项目自己的**导航/选择/实际Auto拖动/保存关开/SaveAs证据。默认文档probe绿不代实际项目绿。原20/34阶段、grid遮挡/取消/Play还原/双按钮捕获等不减；缺阶段/部分report/错项目或runner必须拒绝。

## 4. 作者源、生成基线与用户编辑副本

本期不做反向AST编辑。Node源/closure/生成基线为固定可再生初始内容；公开Open/SaveAs或经先证的显式流程创建独立编辑副本，Editor保存该副本的接受态/资源闭包，关开和搬迁后保留用户操作。拖动/保存不暗改Auto源或生成身份，用户变更后的副本不伪装原基线hash；明确再生基线也不静默覆盖副本。准确支持域/来源关系/场景ID映射由UI040 T-02冻结并给AG018。

原导航夹界、2D/3D投影、snap/3px阈值/grid128线16KiB、4096quad/128场景节点/原资源和Auto交互预算保持。保存拒绝/取消/旧revision/焦点与DPI/late event/故障恢复、UndoRedo/PlayStop还原和真实资源owner不能略过。OS键鼠/抢焦点自动化暂停政策保持；实际Editor消息/Auto/离屏GPU与真实OS输入分别记，手工绑定新项目/runner取证可以补实际交互。必需阶段未证则AC未闭，不偷删分母或借其它旧Plan的pending作为豁免。

## 5. M2最后的同版消费与离线发行

AG019不是新引擎计划；它固定一个含040/035/039的实际已交付auto-ui源码commit，以及所有明确的语言/兄弟源码版本、SDK681内容锁、两个独立Cargo依赖图锁、toolchain/runner/lower/resource-meta身份。每程序保持自己的公开profile/API和多模块组织，“同版”不意味着都用同一profile/单文件/单VM。源码/资源/玩法与原G2九项/G3八项/原G01..G12全部oracle不变；显式再生与新活回执不可混用旧版本历史绿灯。

候选冻结与部分已交付代表项目先证后，完整内容改锁等AG016/017/018实际交付。G3原关卡/参数和音画以正式游戏合同核，035永久夹具适配不自动变更需求。T-01/02展开原各专项工具内部真实断言/变体，原12项22入口全清单与两游戏单列；同一候选真实Auto/Host/GPU/PCM/HUD/编辑/预算/20轮owner逐项验，不按banner或Plan数估M2。

每个游戏与每个Gallery单项包独立闭合：全部作者/资源URI/许可/固定工具/运行依赖/锁随包，B无producer checkout/target/cache/网络/其它项目仍公开运行/改Auto真变化/保存搬迁重开/坏件拒绝修复后下次再读并Run。T-02选定固定Windows运行二进制闭包或真正可离线冷建的源码闭包，明确B侧环境与全部构建依赖；Cargo.lock本身不等于带齐依赖。不建立通用安装器/下载平台。

AG019是一次明确里程碑档，可在用户启动后的单独时段/机器单实例跑原AG里程碑并集与完整内容验收，不把每次review/merge再变全套。原样本/规模/ignore/平台条件/CPU资源oracle保持，准备/编译/发现/运行/清理和排除/未证分列。OS自动化暂停不因“里程碑”自动解除，必需项缺证则候选未就绪。最终本地M2-ready结论须M2-01..09各有直接证据和独立final，发布/远端/标签另待明确用户指令。

## 6. 来源与收尾边界

034/035/039/AG015同会话final、历史旧runner不可达、GPU/声学/OS与D-05批门按真实适用性/owner/关闭oracle记录，不重开归档造独立pass。M1历史源11911a4a…与peel679681b3…差异仍未核等价，不移动标签/改SHA。原保持项可从原收据/保留件独立复验时据内容证明，不要求伪造历史映射；必需内容缺证仍未就绪。

新合同只提SD目标，不在规划写canonical Specs。每Plan同权威树不同上下文独立final后merge冻结门；不因本次无agent授权免复审。材料开工仅plan-artifacts/plan-NNN/<repo>/，候选ZIP/探针/外部消费者/外置target全归自有scope；merge必要摘要折Plan/全输出对账后清自有scope/构建物，仅具名未解决问题持有。禁止attachments/链接/派生JSON，清树先fresh guard，主题/Reader/ACE等WIP保持。
