# Gallery 全量迁入 Node 作者模型

2026-10-31，设计 v1；2026-11-03更新交付进度。作者模型沿[总体设计](declarative-authoring-model.md)，已迁移入口能力分别沿[017 Spec](../../specs/auto-graphics/node2d-g11-authoring.md)与[019 Spec](../../specs/auto-graphics/node2d-basic-authoring.md)。本文件归 auto-ui；Gallery 合同在各自仓管理。

2026-11-03最新核验：UI034已交付93eaa907、AG015 G10两入口已交付80e0b65，按实际metadata/src/generated新作者入口**19/22**，剩g04/2d、g04/3d、g05/3d。AG013及UI032/AG014均已交付，UI035原owner仍在实施3/10。034/015复审收据注明同实现会话复核；独立上下文证据/持名hold依原记录，不因归档自动推定已闭。旧日期计数保历史。

优先级见[M2总路线](m2-game-first-three-lane-roadmap.md)及[查询与完整G3设计](node3d-queries-and-complete-game.md)：新立UI039补三维查询/动态组/有限双维标记，AG016完整G3 T-01/02可准备、T-03等035实际交付及相关独立证据，AG017 G05/3d T-01可准备、T-02等039实际交付。三份暂不实施，不将全部Gallery迁完作游戏启动前置。

## 目标与现场

所有 Gallery 项目的作者源最终使用 Auto 的 node、model、view-2d/view-3d、on、Life、msg、绑定和普通 use。场景 JSON、句柄与单文件运行源可以继续作为生成产物，不要求它们消失，也不要求执行存储照搬声明树。资源、组件和纯函数应复用；不能为每个示例继续新增一个按示例名识别的专用编译器。

当前核验基线：ag-examples92dba6f7、auto-ui e6ef5381；12逻辑项/22入口，新作者19项为g01/main、g02/main、g03双维、g05/2d、g06双布局、g07双布局、g08双维、g09两入口、g10两入口、g11双维、g12/audio与hud。未迁三入口g04两维/g05-3d；纯场景无需角色/触发/Tick/用户VM。

auto-ui 017的g11-2d模式仍要求恰一个character2d，solid/camera限根，资源仅程序化二维图元，不能冒称所有二维/三维/资源服务已支持。旧016/017、AG007收据保持历史，后继不重开归档Plan。

## 逐入口迁移清单

| 入口 | 当前作者形式/能力 | 迁移波次与必须保留的观察 |
|---|---|---|
| g01/main | Node源码，AG008/019；四图元/alpha/显隐/数组序变体 | W1已交付；base+color/position/order/visible，保原alpha排序和等alpha数组序，不改成任意图层能力 |
| g02/main | Node源码，固定027纯静态 | W3 [AG011](../../../../ag-examples/docs/plans/archived/011-autographics-gallery-node3d-basics.md)已交付f35236dc；原GLB/红绿像素/许可/完整包保持，无假角色/用户VM |
| g03/2d | Node源码，AG008/019；父子group | W1已交付；父平移继承/祖先显隐/深层world坐标与Stop恢复 |
| g03/3d | Node源码，原Player/Mob与group | W3 AG011已交付；原几何/材质/父移/隐藏保持，固定027，完整G03包保019二维 |
| g04/2d | JSON，编辑器教程 | W6；Node为场景来源，原视口/选取/拖动/保存/重开仍是编辑器验收 |
| g04/3d | JSON，编辑器教程 | W6；原34步相关能力不改名为公共脚本API；OS自动化准入另受既有政策 |
| g05/2d | Node源码，固定021；segment/sweep/changed | W2a [AG009](../../../../ag-examples/docs/plans/archived/009-autographics-gallery-node2d-queries.md)已交付062081d；九旧义务/39oracle/完整混合维度包，OS阶段具名pending |
| g05/3d | 旧v3/v4 Auto、GLB/group＋二维标记 | W4a [AG017](../../../../ag-examples/docs/plans/017-autographics-gallery-node3d-queries.md)仅规划，完整迁移硬等[UI039](../../plans/039-autographics-node3d-queries-and-mixed-markers.md)实际交付；原三变体/九义务/完整混合包保持 |
| g06/layout-a | Node源码，AG008/019；台阶/跳跃 | W1已交付；输入沿、重力/支撑/碰撞、速度4→5对照、布局A原几何 |
| g06/layout-b | Node源码，AG008/019；坑道/跳跃 | W1已交付；同输入与A产生原几何差异，保无双跳/无缓冲跳与Stop恢复 |
| g07/layout-a | Node源码，固定027，AG012已交付 | W3 [AG012](../../../../ag-examples/docs/plans/archived/012-autographics-gallery-node3d-character-and-cameras.md)交付05f1b498；保main/main-fast/原六动作/支撑与完整包 |
| g07/layout-b | Node源码，固定027，AG012已交付 | W3 AG012；原坑道/几何/main-fast/速度对照及24项保持，实际独立模块参数边界见交付Spec |
| g08/2d | Node源码，AG008/019；跟随/夹界/letterbox | W1已交付；主alt相机配置、同tick角色与相机、实际条带和Pause |
| g08/3d | Node源码，固定027，AG012已交付 | W3 AG012；保二次n/主alt物理参数/同tick/投影，完整G08包保持固定019二维 |
| g09/atlas | Node源码，固定022 | W2b [AG010](../../../../ag-examples/docs/plans/archived/010-autographics-gallery-node2d-sprites-and-frame-animation.md)已交付8bfddca；原PNG/UV/alpha/九oracle与完整包保持 |
| g09/character | Node源码，固定022 | W2b AG010已交付；真实走跳/状态/帧像素/原九oracle与完整包，不跟升027/028 |
| g10/textures | Node源码，固定UI034 93eaa907 | W4b [AG015](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md)已交付80e0b65；原九项/六面UV/遮挡/图像变体/许可与完整包保持，独立复审证据限制见收据 |
| g10/animation | Node源码，固定UI034 93eaa907 | W4b AG015已交付；原四clip/主alt相位/朝向/颜色阴影/物理隔离与八项保持，不跟升035 |
| g11/2d | Node源码，AG007/017 | 已交付；保51原项+新增负控，后续共同端口修改选真实受影响回归 |
| g11/3d | Node源码，固定030，AG013已交付 | W3 [AG013](../../../../ag-examples/docs/plans/archived/013-autographics-gallery-g11-3d-node-migration.md)交付b48b304；具名引用/三同定义Coin局部状态、原25和完整双维包保持 |
| g12/audio | Node源码，固定023有限game档 | W5 AG006已交付e574b58；原两币/八oracle/ordinal/PCM/素材保持，025通用服务亦已交付，不自动重开006 |
| g12/hud | Node源码，新增入口、固定023有限game档 | W5 [AG006](../../../../ag-examples/docs/plans/archived/006-autographics-gallery-event-hud.md)已交付；三币/字形/PCM/共同接受/完整包，声学等具名gap保留，新增1而非旧项迁移 |

已完成原21中的18个、HUD新增1，当前19/22。待迁W4a一项（AG017硬等UI039）、W6两项（未取号），共三；AG017交付预计20/22、净增0，届时实际重算。旧归档计数保历史，不按Plan数估M2。

## 第一轮交付与后继边界

- [auto-ui/019](../../plans/archived/019-autographics-node2d-scenes-character-camera.md)：已交付c93adb7；纯视觉叶子/二维group、零或一个角色、跳跃/支撑、可选跟随相机与完整主alt源变体；公共准备/来源消费能力。018已被UI构建配置路线占用，不能覆盖。
- [ag-examples/008](../../../../ag-examples/docs/plans/archived/008-autographics-gallery-node2d-first-wave.md)：已完成并合并8a01a34（archived/r1，10/10，归档/清理见合同最终收据）；固定消费019实际交付，已迁G01、G03/2d、G06两布局、G08/2d。每个入口的旧行为/素材/变体/验收在计划中逐项承接；原3D和其它项不跟升。

先引擎交付、再内容；下一波引擎可与上一波固定消费的内容并行，读端不追开发tip。查询、二维图片、三维资源和声音的端口应复用共同领域表示，但共享文件要约定owner短窗，GPU/窗口/音频重型门单实例。当前018、语言901与UI014的真实写面由开工核对，不凭标题加全量串行前置。

2026-11-03下一轮：UI039三维查询/动态组/有限双维标记；AG016完整G3等UI035实交付；AG017 G05/3d等039实交付。准备可独立、全链各守硬门，不读开发tip。G04教程与最终M2统一版本/里程碑后继另立，不本轮预取编号。

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

本次按实际已交付源更新到19/22并规划UI039/AG016/AG017；035仍原owner实施；只改合同/设计/导航，不实施或建树/探针/素材，不构建/跑GPU。游戏已交付不改变Gallery入口分母。
