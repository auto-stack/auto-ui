# M2游戏优先路线：引擎、完整游戏、Gallery三线并行

2026-11-03，设计v3。AG以AutoUI式Node声明、局部model/msg/on、view-2d/view-3d、普通use及具名资源/事件表达图形应用。主线按完整2D闭环→3D表现与声音/HUD公共接线→完整3D游戏推进，Demo持续并行。当前M2目标为桌面，两游戏/编辑/资源与离线发行/Gallery/同一最终版本分母保持；ACE/AOT/ECS/CPC/跨端不新增为M2前置。

## 1. 当前实际交付

- 005纹理351919c、006节点动画c681663、012/013声音/HUD、025通用二维服务e9cc929/54899e9、027角色相机f3dea0d、028对象触发ebac1cd、030具名引用804d7303均已交付；030独立final证据已补齐，旧“待补”快照不再作当前门。
- [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md)完整二维组合已交付a8c04d22；[AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md)正式完整2D游戏已交付c141df8，分别独立复审/实际落主/Spec沉淀/归档清理。游戏保存关开/SaveAs沿引擎profile无关play套件的具名复用边界见AG014收据，不把单游戏交付等同M2全部通过。
- [AG013](../../../../ag-examples/docs/plans/archived/013-autographics-gallery-g11-3d-node-migration.md)G11/3d已交付b48b304；[AG012](../../../../ag-examples/docs/plans/archived/012-autographics-gallery-node3d-character-and-cameras.md)此前已交付05f1b498。当前逐入口新作者活源**17/22**，剩G04两维/G05三维/G10纹理动画两入口；历史13/22/14/22/16/22回执保留，不按Plan数估M2百分比。
- 正式3D游戏尚未立项，现有3D Node没有005/006完整纹理/动画及公共声音/HUD作者组合。底层能跑不等于作者接线/完整G3已交付。

权威分母：[M2-01..09](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[G2-01..09/G3-01..08与原关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。旧UI011固定0261ae9两候选33/32的no_go是历史，已由032新显式有界组合取得新的完整二维证据，不能外推成永久语言限制或抹掉旧失败。

## 2. 当前下一批：034、035、AG015

正式设计见[3D表现/音画与G10迁移](node3d-presentation-audio-and-g10-migration.md)。本轮三份均drafting/r1、0/10，仅规划、暂不实施。

| 线/责任仓 | 当前合同 | 可独立推进 | 依赖硬门 |
|---|---|---|---|
| 引擎表现/auto-ui | [UI034](../../plans/034-autographics-node3d-materials-and-animation.md) | 005/006/030实际交付基础上先证typed纹理/动画/clip/目标和局部组件 | T-02最多两机制全链先证/原32与G10十七项保持 |
| 引擎音画/auto-ui | [UI035](../../plans/035-autographics-node3d-audio-hud-composition.md) | T-01/02原G3/素材/许可/oracle/接口和预算准备 | T-03完整组合等034实际交付/独立review/Spec/工具；不读tip |
| Demo/ag-examples | [AG015](../../../../ag-examples/docs/plans/015-autographics-gallery-g10-node-migration.md) | T-01原G10原件/17/许可/包准备 | T-02完整迁移等034实际交付；不等035，不加声音/HUD |

034交付后035完整接线和AG015完整内容迁移可并行。三份是阶段并行，不意味着可越过实际交付门；引擎同仓共享AST/load/translate/assembly短窗单写，内容工具/registry/packager按owner协调，各Plan各树/target/材料scope，GPU/窗口/音频重型门机器单实例。

## 3. 能力与预算边界

034接原opaque纹理/UV/KHR/LOD0和有限节点LINEAR/STEP TRS，原GLB/闭包/clip保字节/ordinal；Auto决定clip/phase/yaw，颜色阴影共pose、物理不变，无假Player的actor-free场景与原模型角色都要先证。新3D档由声明绑定/普通handler决定业务，不将032的state/pose/facing保留字段推广为3D语义，不用Rust推断Idle/计分/巡逻。

035复用audio3d/hud3d/一个视觉＋四声＋四HUD和032有界机制的可复用部分，actor-free服务与原完整G3八项均需实际证明。必要三维快照/批销毁使用新显式身份/公开域和价目，旧档字节/价目保持，不直接借2D成功推定3D。完整G3/合法峰值原32内实测；两候选失败则needs_replan，不以no_go研究代功能交付、不停移动/漏事件/静音或改地图凑池。

共享原credits32、物理lifecycle写32、8状态Node/8slot/8pair、120base+8reserved128及deep/contact64base、mesh512、raw8身份/32MiB/单件8MiB、CPU/GPU各64MiB和原VM/source/时间限额。纹理/动画、声音/HUD具体域和全部负控/owner/资源许可按新设计及各合同冻结；不给每组件/服务另开预算池。

## 4. 内容仓、游戏与独立交付

用户已批准ag-examples统一管理内容：examples/保G01..G12能力Demo，games/platformer-2d持已交付完整G2，未来games/platformer-3d另立完整G3合同；共用本仓Plan编号，各游戏独立源码/关卡/依赖锁/资源manifest/启动/验收/离线包。旧ag-platformer-2d仅留迁移指引，无第二活动合同，不再新建游戏Git仓。

AG015只迁G10两活入口，保原9+8与内部原义务/原件/四clip/主alt/几何和完整双入口包；净增0，交付后预计19/22，届时按实际metadata重算，当前不提前记完成。AG014/其他Demo保固定消费，新的3D能力不强制全体跟升。M2最终统一引擎/SDK版本与发行门仍独立核验。

游戏和Demo的独立性以离线闭包验证：共享资源/工具须在包内收齐，A→B无生产者checkout/target/网络及其他Demo隐含依赖；实际Run/保存关开/SaveAs/冷再生/坏件恢复。引擎夹具绿不代内容接受，内容绿不代完整游戏或整个M2。新三维组合/内容的保存义务要真实新链证据，不能沿旧profile无关报告一概冲销。

## 5. 后继与维护

当前只给用户明确批准的034/035/AG015取号；完整3D游戏、G05/3d查询、G04教程和M2最终统一版本/里程碑验证按实际缺口滚动，不预取一串编号或承诺日期。3D游戏在034/035实际交付和2D闭环证据基础上另立；无需等待所有Demo迁完。长期同一程序覆盖不同屏幕的目标保留，当前桌面范围不变。

本轮只改合同、设计、路线和导航；不建实施树/源码/素材/探针/提案canonical Spec，不构建/运行GPU或OS自动化，不创建agent/远端/标签。future实现和review一Plan每仓一权威树、不同上下文独立final、merge冻结门；材料plan-artifacts/plan-NNN/<repo>/，开工登记owner，merge折摘要/全输出对账后删自有scope/构建物，具名问题持有。禁止worktree链接/attachments/派生JSON，清树先fresh guard，不覆盖其他路线WIP。
