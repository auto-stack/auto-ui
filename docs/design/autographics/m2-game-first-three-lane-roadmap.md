# M2游戏优先路线：引擎、完整游戏、Gallery三线并行

2026-11-02，设计v1。用户明确批准路线调整：现在推进“2D引擎组合＋完整2D游戏”，同时规划3D表现/音画接线；2D闭环稳定后再交付完整3D游戏。三条线分别持计划、独立推进，每批可并行三份合同；这是工作组织，不授权本轮实施/创建agent。

## 1. 当前事实与调整理由

028交付ebac1cd，030刚交付804d7303；030 canonical node3d-symbolic-component-bindings.md SHA6ed0bf070de48c3fae4ff8c1687dca9d67c8645010aef5783e49ca14b3352e18，落主/归档/清理有收据，但其final记录明确“实现会话内重建审查，非独立会话”。不能将同会话自审称独立复审；AG013完整迁移前须补独立上下文、绑定实际交付的review证据，本轮不重开030归档或擅自复审/创建agent。2D组合没有030功能依赖。

AG012已交付05f1b498，原24项/六变体/完整G07/G08包闭合；当前活作者源按逐入口src/generated和消费Spec核为16/22。旧归档采用11+3=14/22的历史基线，差额为G12 audio/hud两入口；两者实已在AG006转Node，故当前覆盖16，未迁六入口：G04两维、G05/3d、G10纹理/动画、G11/3d。旧收据保留，不按计划数推里程碑百分比。

020/022/025已提供局部组件、精灵与通用二维音画；底层hud2d已经组合sprites/triggers/audio，但025作者档只使用kind=-1声音/HUD，不提交精灵视觉。旧UI011只证明固定0261ae9的两候选完整G2失败（典型33/32），不证明现版本不可能，也不被025有限G12“恰32”自动关闭。完整G2/两款独立游戏仍未交付。

原M2-01..09/G2-01..09/G3-01..08分母不变，详见[原M2合同](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[冻结关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[旧预算研究](../../../../auto-lang/docs/specs/auto-graphics/g2-full-loop-budget-feasibility.md)。Gallery覆盖和原通用能力不等于完整M2。

## 2. 本批三份合同及真实依赖

| 并行线/权威仓 | 本批合同 | 可独立推进 | 硬门 |
|---|---|---|---|
| 引擎/auto-ui | [UI032：2D完整游戏组合](../../plans/032-autographics-node2d-full-game-composition.md) | 020/022/025实际交付，完整G2夹具、公共组合/有限操作、生成器与来源 | T-02完整路线/峰值原额先证，旧接口不变；不是以no_go冒充功能交付 |
| 完整游戏/ag-platformer-2d | [G2-001：独立2D完整游戏](../../../../ag-platformer-2d/docs/plans/001-autographics-complete-2d-platformer.md) | T-01..04冻结玩法/来源/许可、制作资源/关卡、普通Auto逻辑和作者源；不读032开发tip | T-05完整接线/正式运行硬等032实际交付及冻结Spec；不能把部分准备计游戏交付 |
| Demo/ag-examples | [AG013 r2：G11/3d](../../../../ag-examples/docs/plans/013-autographics-gallery-g11-3d-node-migration.md) | 复用既有号；先原件/25映射/包工具准备；不依赖032/G2 | T-02完整迁移固定030实际交付，先补其独立review门，不消费开发tip |

本轮只新增032和游戏本仓001；013修订为r2、保全部原AC/25项，不重复新建Demo计划。ag-platformer-2d为本地仅管理文档的新独立仓；无远端/源码/资产/构建/实施树。未来3D游戏另独立仓，尚不初始化。

三线互不写对方实现。游戏发现缺口以可重放案例交引擎合同处理，不在游戏/内容仓补Rust玩法。引擎放永久最小验证夹具，正式游戏/素材归游戏仓，Gallery示例仍归ag-examples；不要把全游戏复制进Gallery或引擎。引擎签名/语义在032 T-02冻结并记计划，游戏可据该文档写草稿；正式依赖锁/可运行产品只在实际交付后绑定。禁止以共享开发checkout/SDK副本/target偷越门。

共享AST/load/translate/assembly写入按owner短窗；每Plan每仓唯一权威树、同树不同上下文独立review。GPU/窗口/音频重型门同机单实例，其他源码/资源/只读检查可并行；OS键鼠/焦点自动化继续暂停，必需实机证据另列真实人工走查或专项授权，不能计作自动通过。

## 3. 032组合与预算提案（不是已实现接口）

目标是普通Node/model/view-2d/msg/on/use，角色/敌人/三Coin/Goal/Boundary/root局部持状态，固定八有状态Node；纯平台/模板不占状态数。根组织显式顺序，玩法全由真实Auto决定。结合022图像/帧/flip、025声音/HUD与完整对象/触发/Restart，不能沿用023固定count/state或自动Idle游戏专用编译器。

保原60Hz、全Callback32空间credits、32 lifecycle物理写、八对象槽/八pair、120base+8reserved、512潜在mesh、8资源身份/32MiB raw、CPU/GPU各64MiB、4声音请求/8voice、4HUD槽/每槽32ASCII、原VM/source/CPU/时限与8ms pump目标。不能通过收集时停走、停敌人、隔帧处理、挪币/改关卡、漏第三事件或降低验收兑现预算。

T-02最多两候选：A只复用已交付input snapshot/呈现bundle与单条event/despawn；B在**新显式profile/API版本**加入两种公共有界操作。B的拟议价目与支持域本轮公开如下，旧单条API原价/字节保持，不是旧getter免费化；准确签名/编号/诊断T-02在注册表核唯一后冻结，实施须以该明确合同为准。

| 拟新操作 | 有限支持域/语义 | 新版本拟计价 |
|---|---|---|
| 完整事件快照 | 每callback至多一次，把上Tick至多8条已冻结事件按pair序物化为定长、有count、全int身份/phase/source/generation/geometry的值；空集合法；未读不跨Tick排队，仍N→N+1，不允许同回调再用单条cursor混读 | 整个有界快照1 credit；物化后的本地值读取是Auto计算，VM/CPU照计 |
| 对象批量销毁 | 每次至多8个不同、同Run/完整代次的live句柄；先验证全部再修改，重复/旧代/错Run/第9项具名拒绝；调用点显式，不悄悄延迟普通despawn；不会合成exit | 一次批调用1 credit；lifecycle写计数仍按实际N个对象计，第33物理写仍错37 |

用明确的新组合操作压缩往返，保每Tick角色move26、grounded1、敌人move1与实际视觉/声音/HUD，不给每个Node独立额度。候选峰值账：input snapshot1+move26+grounded1+enemy move1+event batch1+despawn batch1+presentation bundle1=32。此为待验证和式，不是go；必须实际跑正常路线及移动/Jump-land/三币同Tick/敌人-目标竞争/未齐目标/重开等组合。单个业务演示、初始重叠便宜分支或平均值不代完整最高分支。

原bundle一次视觉＋四声音＋四HUD已交付；最小G2采用玩家idle/run/rise/fall/flip的一个动态视觉，敌人位置每Tick真实更新、独立可辨识sprite。若冻结游戏验收确需同Tick多项视觉变化而原域不够，T-02必须明确其有界公共协议/价目或needs_replan，不能删必要表现/绕过混写门。合法2D跳跃/收集音效峰值不得靠静默丢声过四请求门，五请求负控仍拒绝；四声音在现有bundle中全传递。

新增Unit/Frame/assembly若确有必要须显式版本化、完整独立准入，旧profile的历史错误/价目/字节和SDK681固定源保持。若两候选都不成立，032仍未完成，回规划提出具体阻塞；不能以一份no_go研究替代引擎功能交付，也不要求在未获批准时涨32额度。

## 4. 游戏交付与先证分工

G2-001承接858固定G2几何/速度/重力/输入、三币/敌人/终点/失败重开、idle/run/jump/fall、一次jump/coin音效、n/3与胜败HUD、资源许可与离线闭包。初期分阶段演示可作为开发进度，完整验收必须同一程序/同一引擎版本证明所有G2-01..09，不以Gallery凑拼图。游戏计划T-03素材及T-04纯逻辑/作者源准备不等引擎，T-05..06锁实际交付后真实整链、编辑/保存/搬迁/错误恢复。

032与游戏各有独立oracle：引擎验证公共组合/价格/资源/准入/负控，游戏验证正式内容与路线。必要测试只按真实修改影响面，不为日常merge跑全AG。正式依赖固定实际source/SDK/锁/toolchain/参数与generator/runner身份；不因Windows同源PE hash有差异而豁免来源，不把陈旧exact二进制当永久交付。

## 5. 同时规划的3D表现/音画接线（本批不预占编号）

两个后继能力切片已定支持边界，尚不创建正式实施Plan；在2D引擎接口冻结/共享写面稳定后据实际代码取号，不阻塞本批三线准备。

| 切片 | 复用已交付 | 必须证明/排除 |
|---|---|---|
| 3D Node表现 | UI005纹理/UV、UI006有限节点动画/朝向、027/028/030组件 | typed纹理/GLB资源闭包、clip/phase/yaw作者Auto驱动、角色/visual绑定、颜色阴影同pose、碰撞/触发不被视觉动画改写、旧代/重生/Stop；仅既有无skin LINEAR/STEP域，不增骨骼/PBR全材质 |
| 3D Node音画组合 | audio3d/hud3d、现有ag_present3d、对象/触发/相机与新有限操作可行结论 | clip/音效/HUD与完整G3在同Tick共同消费、jump/land/coin一次性、4请求/8voice/4槽、native/资源owner/失败恢复、全G3峰值预算；不能把底层fixture当新Node已接线 |

进入标准：G3-P01完整资源/许可/再生；P02 idle/walk/jump与phase/yaw；P03视觉-物理隔离；P04原相机相对运动与事件；P05声音/HUD/共同呈现；P06全G3最高分支原额；P07生命周期/坏件/旧代；P08仓外冷消费。两片均须独立review/冻结/实际交付；不提前认领通过，不将031主题/生产UI、ACE/AOT/ECS/CPC/跨端加为前置。

完整3D游戏在2D完成可玩→失败重开/获胜重开、必要表现/音画、保存搬迁、独立review/落主之后接棒；其资源/玩法设计可早做，但最终产品交付按此顺序。G05/G10/G04等Demo迁移与3D游戏能力互相提供用例，不要求整个Gallery先完成才启动游戏；M2最终仍要求原Gallery/两游戏/统一版本与发行分母全部闭合。

## 6. 路线维护规则

当前正式下一批恰三份合同：UI032、G2-001、AG013 r2；“三份”不是同一时刻必须都无依赖，阶段门必须如实记录。下一轮每条线按实际完成/缺口滚动，不预取一串编号或用Plan数量承诺百分比/日期。长期梦想仍是同一Auto程序覆盖不同屏幕，当前M2桌面目标不变。

本轮管理文档可在主检出维护；未来所有实现/构建在各自唯一树，临时材料只放plan-artifacts/plan-NNN/<repo>/，同号按仓区分、开工登记owner/用途/在途，merge折摘要后删除本Plan自有scope/构建物/全部实际外部输出；仅具名问题持有。禁止链接/attachments/派生JSON，清树先fresh guard。未创建远端、运行probe、游戏素材或实施代码。
