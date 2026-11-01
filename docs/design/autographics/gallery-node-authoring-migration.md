# Gallery 全量迁入 Node 作者模型

2026-10-31，设计 v1；2026-11-02更新交付进度。作者模型沿[总体设计](declarative-authoring-model.md)，已迁移入口能力分别沿[017 Spec](../../specs/auto-graphics/node2d-g11-authoring.md)与[019 Spec](../../specs/auto-graphics/node2d-basic-authoring.md)。本文件归 auto-ui；Gallery 合同在各自仓管理。

2026-11-02本轮核验：UI025/027已交付并清理（e9cc929＋54899e9、f3dea0d）；AG010交付8bfddca。当前代码/metadata为11/22新作者入口（原21迁10＋HUD新增1），剩11；旧010部分收据9/22不覆盖当前真实源码清单，保原历史不据其减分母。AG011用户告知开发中，已核见唯一ag-011树，合同任务进度待owner同步，本轮不接管/翻其状态。[UI028与AG012](node3d-instances-triggers-and-gallery-motion.md)各0/10、暂不实施；新草案和在途内容不计迁移完成。

## 目标与现场

所有 Gallery 项目的作者源最终使用 Auto 的 node、model、view-2d/view-3d、on、Life、msg、绑定和普通 use。场景 JSON、句柄与单文件运行源可以继续作为生成产物，不要求它们消失，也不要求执行存储照搬声明树。资源、组件和纯函数应复用；不能为每个示例继续新增一个按示例名识别的专用编译器。

当前核验基线：ag-examples2fcbaa72、auto-ui867819bf，12逻辑项/22入口；新作者入口g01/main、g03/2d、g05/2d、g06/layout-a、g06/layout-b、g08/2d、g09/atlas、g09/character、g11/2d、g12/audio、g12/hud，共11个（原21迁10＋新增1），剩11。011待交付不提前加2；012未来实际迁3也不预加。G01/G02/G03原为JSON场景，纯场景无需角色/触发/Tick/用户VM。

auto-ui 017的g11-2d模式仍要求恰一个character2d，solid/camera限根，资源仅程序化二维图元，不能冒称所有二维/三维/资源服务已支持。旧016/017、AG007收据保持历史，后继不重开归档Plan。

## 逐入口迁移清单

| 入口 | 当前作者形式/能力 | 迁移波次与必须保留的观察 |
|---|---|---|
| g01/main | Node源码，AG008/019；四图元/alpha/显隐/数组序变体 | W1已交付；base+color/position/order/visible，保原alpha排序和等alpha数组序，不改成任意图层能力 |
| g02/main | JSON+静态GLB，迁移在途（用户告知） | W3 [AG011](../../../../ag-examples/docs/plans/011-autographics-gallery-node3d-basics.md)固定实际交付024/027；合同/树进度登记待owner同步，保原GLB/像素/许可/完整包，不接管 |
| g03/2d | Node源码，AG008/019；父子group | W1已交付；父平移继承/祖先显隐/深层world坐标与Stop恢复 |
| g03/3d | JSON，原Player/Mob与group，迁移在途（用户告知） | W3 AG011；027已交付原内置模型透传，保几何/材质与父移/隐藏，不换圆柱，不再宣称027未交付 |
| g04/2d | JSON，编辑器教程 | W6；Node为场景来源，原视口/选取/拖动/保存/重开仍是编辑器验收 |
| g04/3d | JSON，编辑器教程 | W6；原34步相关能力不改名为公共脚本API；OS自动化准入另受既有政策 |
| g05/2d | Node源码，固定021；segment/sweep/changed | W2a [AG009](../../../../ag-examples/docs/plans/archived/009-autographics-gallery-node2d-queries.md)已交付062081d；九旧义务/39oracle/完整混合维度包，OS阶段具名pending |
| g05/3d | 旧v3 Auto+场景，segment/sweep | W4a；同上三维与parent空间，需先有3D基础 |
| g06/layout-a | Node源码，AG008/019；台阶/跳跃 | W1已交付；输入沿、重力/支撑/碰撞、速度4→5对照、布局A原几何 |
| g06/layout-b | Node源码，AG008/019；坑道/跳跃 | W1已交付；同输入与A产生原几何差异，保无双跳/无缓冲跳与Stop恢复 |
| g07/layout-a | character3d旧Auto，固定方向基 | W3 [AG012](../../../../ag-examples/docs/plans/012-autographics-gallery-node3d-character-and-cameras.md)草案，固定027；保main/main-fast/六动作/归一/接触支撑，不强制相机 |
| g07/layout-b | character3d旧Auto，另一布局 | W3 AG012草案；原坑道/几何与main/main-fast完整保留，独立速度变体对照 |
| g08/2d | Node源码，AG008/019；跟随/夹界/letterbox | W1已交付；主alt相机配置、同tick角色与相机、实际条带和Pause |
| g08/3d | camera3d旧Auto，环绕/缩放/相对运动 | W3 AG012草案，固定027；保实际二次n/主alt物理参数/同tick/输入清理/投影，完整G08包保固定019二维 |
| g09/atlas | Node源码，固定022 | W2b [AG010](../../../../ag-examples/docs/plans/archived/010-autographics-gallery-node2d-sprites-and-frame-animation.md)已交付8bfddca；原PNG/UV/alpha/九oracle与完整包保持 |
| g09/character | Node源码，固定022 | W2b AG010已交付；真实走跳/状态/帧像素/原九oracle与完整包，不跟升027/028 |
| g10/textures | v1 Auto+GLB/PNG/UV | W4b；原件与UV夹具、遮挡/显隐/许可与完整资产闭包 |
| g10/animation | animation3d Auto+clip/visual pose | W4b；原四clip、yaw/phase、颜色/阴影/物理隔离，不把动画公式放Rust |
| g11/2d | Node源码，AG007/017 | 已交付；保51原项+新增负控，后续共同端口修改选真实受影响回归 |
| g11/3d | triggers3d旧Auto | W3 [UI028](../../plans/028-autographics-node3d-instances-triggers-and-rounds.md)草案补实例/触发/新局Node接线与三种Mob模板，保原25项；内容后继等实际交付再立，不预取号 |
| g12/audio | Node源码，固定023有限game档 | W5 AG006已交付e574b58；原两币/八oracle/ordinal/PCM/素材保持，025通用服务亦已交付，不自动重开006 |
| g12/hud | Node源码，新增入口、固定023有限game档 | W5 [AG006](../../../../ag-examples/docs/plans/archived/006-autographics-gallery-event-hud.md)已交付；三币/字形/PCM/共同接受/完整包，声学等具名gap保留，新增1而非旧项迁移 |

已完成W1五入口、AG007一入口、W2a一入口、W2b两入口、W5原audio一入口，共原21中的10个；HUD新增1，故当前11/22。待迁W3六个（011两入口在途、012三入口草案、G11-3d等028后继）、W4a/W4b三个、W6两个，合计11；迁移净增0，新增入口独立记账。

## 第一轮交付与后继边界

- [auto-ui/019](../../plans/archived/019-autographics-node2d-scenes-character-camera.md)：已交付c93adb7；纯视觉叶子/二维group、零或一个角色、跳跃/支撑、可选跟随相机与完整主alt源变体；公共准备/来源消费能力。018已被UI构建配置路线占用，不能覆盖。
- [ag-examples/008](../../../../ag-examples/docs/plans/archived/008-autographics-gallery-node2d-first-wave.md)：已完成并合并8a01a34（archived/r1，10/10，归档/清理见合同最终收据）；固定消费019实际交付，已迁G01、G03/2d、G06两布局、G08/2d。每个入口的旧行为/素材/变体/验收在计划中逐项承接；原3D和其它项不跟升。

先引擎交付、再内容；下一波引擎可与上一波固定消费的内容并行，读端不追开发tip。查询、二维图片、三维资源和声音的端口应复用共同领域表示，但共享文件要约定owner短窗，GPU/窗口/音频重型门单实例。当前018、语言901与UI014的真实写面由开工核对，不凭标题加全量串行前置。

2026-11-02下一轮：027实际交付f3dea0d，025已交付清理，AG010已交付、011用户告知在途。[本轮设计](node3d-instances-triggers-and-gallery-motion.md)立AG012（G07两布局/G08-3d三入口）与UI028（三维实例/触发/新局Node），各0/10。012固定027不等011/028；028只写引擎，为G11-3d后继做前置，内容计划等交付再立。随后三维查询、纹理/动画、编辑器教程按证据细分；完整M2两游戏/整链预算/发布仍pending，不预领一串编号。

## 不变的交付要求

1. 作者写模型/声明/事件与普通模块。逻辑仍由真实Auto执行；声明与运行组织独立，不为可审计性禁use，不引入新解释器或专用Rust玩法。
2. 每波记录已交付项/入口/实际检查集合。旧回执只有verify_examples一个banner的，必须展开工具中的实际断言，不能用“原来一项、现在也一项”删除验收。未迁移入口、旧锁与素材保持；迁移入口以新的精确source/资源闭包、负控和真运行替代旧认证，不沿用旧绿色截图。
3. 原profile价格/数值/维度/资源和故障边界不变。不得为统一Node形式强迫纯场景启用角色/触发profile；G06不自动加相机。预算改造、ACE/AOT、ECS、完整M2游戏及多呈现选择不是迁移的附带承诺。
4. G04只改变场景的作者来源。生成文档仍可编辑/保存/重开；不承诺编辑器反向重写Auto或双向同步。若需此能力必须另立明确合同。
5. 新增示例优先进入新作者路线；既有未实施草案（特别AG006）要在真实依赖明晰后显式修订，不能一边记全量迁移一边无说明累积旧脚本。

## 运行供给与现场风险

当前清理政策删除计划临时二进制；PE受路径/构建环境影响，固定源码重建哈希可能不同。AG007新2d锁仍只接受历史6ceeb384…，用户试玩使用9a8a5be固定源码重建39a8e27b…，两者不能混称。后续迁移必须提供实际可用的消费/重建方案，不能靠把旧exe当证据永久保留、改写旧锁或静默跳过hash门。

019/008已交付并冻结**仅适用于其新迁移入口**的版本化来源消费合同，内容消费规范见[AG008 Spec](../../../../ag-examples/docs/specs/auto-graphics/gallery-node2d-basics.md)：固定source/SDK681/依赖锁/工具链与构建参数；本地重建记录绑定实际二进制hash并验证真实消费者。它必须区别原已认证runner与本机重建，原锁模式继续严格拒绝不匹配。不得把任意自填JSON、当前tip或source字面SHA当已证明来源；若同源重建的闭合验证做不到，须带实际反例回规划，不让内容仓自行豁免。

另有一次真实试玩观察：G11/2d在本机重建版到tick112出现`auto-Pause: tick backlog >4`。此为观察而非根因结论（窗口捕获/背景负载与执行路径尚未区分），不反推Node语法不行，也不能以51个离屏检查绿宣称桌面持续流畅。首轮整链验证需记录同机背景、完整Tick/pump与原码/新码对照，保暂停守护，改新路径引入的回归由其owner修复；无证据不调额度或删除暂停。

## 证据与当前范围

本设计是长期路线和接口要求，不是临时探针/验收报告。临时材料只在plan-artifacts/plan-NNN/<repo>/，禁止新建attachments（包括md）；merge前摘要折叠进Plan，清理自有scope及构建中间物，具名未解决问题按新政策持有。永久产品源码、必要回归测试/夹具和canonical Spec按所属仓维护。

本次仅按027/025与AG010实际交付证据更新导航/路线至11/22，并规划UI028/AG012；不实施任何计划，不改示例/引擎源码、旧锁/验收，不建树、不构建/跑GPU。
