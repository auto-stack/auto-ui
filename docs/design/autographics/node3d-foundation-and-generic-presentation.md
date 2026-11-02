# 3D Node基础与通用声音/HUD接线

2026-11-01，设计v1；保留024/025首次规划边界，当前状态以本次补记与各合同为准。作者目标沿[声明与组件模型](declarative-authoring-model.md)，波次沿[Gallery迁移路线](gallery-node-authoring-migration.md)。本文件是正式架构/支持边界，不是临时探针或实现验收。

## 当前交付与后继补记

截至2026-11-02，024/025/027/028均已交付清理，AG011交付f35236dc。当前Gallery13/22，剩9；012用户告知在途，不接管。下方旧计数/树状态保首次规划快照。[新后继设计](node3d-symbolic-bindings-and-g11-migration.md)规划030（具名引用/跨Node参数与局部组件）和013（G11/3d内容，硬等030实际交付），原预算/玩法/验收保持、暂不实施。

## 现场与两份合同（首次规划快照）

本轮只读基线auto-ui0b6e74d；020组件16ddfa0、021查询1bec52f、023有限game32d7a81已经实际交付。022由其独立树完成复审/合入，进度以owner记录为准；AG006固定023、AG009固定021均已在途；不从旧导航drafting0/10推断未实施，也不接管它们。Gallery仍12逻辑项/21入口、已迁6/21，新合同不计完成。

| 引擎计划 | 本期交付边界 | 硬前置 | 内容边界 |
|---|---|---|---|
| [024 3D声明/静态资源/组件基础](../../plans/archived/024-autographics-node3d-foundation.md) | view-3d、有限三维图元/平色GLB、同维translation/visible层级、局部model/msg/on/use、纯场景和有状态模式 | 已交付020；复用现有3D渲染/资产/基础行为 | 为G02/G03/3d提供能力，不改项目/不分配内容号 |
| [025 声音/HUD通用Node接线](../../plans/archived/025-autographics-node-presentation-services.md) | 非游戏/游戏通用typed资源/槽/局部事件、四请求一次收口；业务全在作者Auto，无角色可用、连续移动/提前终点原额闭合 | 已交付020/023，底层012/013 | 不暗加AG006前置，不改其固定023；需要切版本由内容owner显式修订 |

两份均drafting r1、0/10，可独立规划/实施准备；实际实施另待用户启动。没有彼此或022功能依赖。共享Node AST、load/translate、CLI/assembly的写入按owner短窗顺序接线，不复制子系统、不在同树并发写；重型GPU/音频/窗口门机器单实例，OS键鼠/焦点自动化继续暂停。各自target/consumer/报告独占。

## 024：把三维作为Node呈现，保留既有领域边界

一套Node组件/模块/receiver机制支持view-3d，复用Core Expr/Body、020的局部存储/typed事件/helper作用域。固定树的每个根/子Node从首版可有model/msg/on，局部状态驱动其自身三轴位置/显隐；不重演子组件仅纯view后才补状态。Node组合不隐式等于场景transform parent，显式同维group才有逐层f32平移和祖先visibility AND。

本片的单一view-3d出口不是多呈现或UI/2D/3D混合实现；已有view/view-2d/view-3d命名方向保留。primitive种类/静态属性只映射当前Document能力，GLB只有既有平色静态域；不添加组TRS、运行旋转/缩放、PBR/纹理、动画/骨骼、角色/相机/触发。纯场景不强制model/Tick/用户VM，有状态视觉不强制假Player或运行相机。优先使用已有behavior=null和基础v1行为接线，实际可行性由024 T-02三条真链证明。

GLB由具名资源symbol/typed参数引用，原完整SHA/length/format定义身份，资源名字不是ID。同SHA多Node共享几何/GPU owner，不每实例复制；当前accepted/history/candidate/外部pin按原账与最后owner处理。保GLB raw8MiB/JSON1MiB、asset128nodes/16expanded primitives、每primitive65536vertices/每asset262144vertices与786432indices、资源8身份/32MiB raw/CPU-GPU各64MiB、512潜在mesh含隐藏项。场景128对象+组和8有状态组件为另外两个包络，不能混算。新文件/typedGLB准入仍复用现有受限decoder，不相信自报manifest或重写导入器。

必须承接的旧观察来自真实G02/G03：G02两部件1528B GLB SHA42c9da67…及MIT、原红绿像素阈值/重开/坏GLB拒绝；G03/3d world=(1.8,1,0)、父x+2→3.8、祖先隐藏/实际capture变化/Run不改文档。它们共有十一义务，不因旧banner只有一个而减少。再补双局部组件/两个父receiver/三轴绑定/生命周期/owner反例。API v1自身不具备空间32池，不虚构收费/放开查询；各选中API原价格、VM/source/时限、8ms目标与原资产边界全保留。

下一阶段再按实际024接口规划3D角色/运行相机→实例/触发，之后三维查询/纹理/节点动画及编辑器教程；目前不取后继号，不把基础GLB通过当完整3D游戏完成。

## 025：资源/呈现是服务，计分和胜败是作者程序

023实际交付是有价值的有限game例，但lower_components_game含Collect/Goal/Fail、固定count/state、自动Idle、两pending声音槽。现有真实证据是该例通关与音画，不是任意Node程序的通用性证明；canonical 023保持其版本事实，新模式另立Spec/显式入口，不能重开归档Plan或改旧世界hash/期望遮盖差异。

025的新服务表示只能有资源、typed引用/槽、事件和有限呈现请求，不应知道“几个币算赢”或何时despawn。作者on/helper决定局部状态、一次收集/胜败/文字/cue/gain，任意合法Node/root/字段名均可，两个同定义组件各有receiver/model。生成Auto可以维护Run ordinal/有界四请求/一次收口，因为这是传输协调；不得隐式count++、代写r_state、自动冻结输入或丢第三/第四声。

无游戏角色的通知/状态面板也是必要正控，不为音效/HUD暗建Player/camera。现底层audio/hud组合依附character链，需要T-02有界比较新显式组合的复用方式；新版本/Frame/marker编号按注册表冻结，旧012/013/023装配与准入字节不改，不用删除旧角色门或另一套VM解决。此片落地二维服务，3D接线另随三维运行基础推进，不因“通用”承诺任意硬件/维度。

所有Node请求按真实Auto顺序收集，Run精确int ordinal统一递增；四声音请求合法，第五个必须真实具名拒绝，不能封顶2静默丢弃。四槽完整状态/固定ASCII/Font/原mixer规则保持，多个slot writer绑定歧义在新typed层拒绝。回调末一次bundle1计费，standalone写仍原价且不能混写；原getter/触发/snapshot实际收费，只有明确物化的snapshot值可在Auto内按位读取，不自动缓存/省略旧API调用。

原32池的先证必须包含连续移动、收集、未齐提前终点/站立/离开重入、合法绕币/Jump-land重合，不只默认通关轨迹。收集候选1snapshot+26move+1grounded+1next+1pair+1despawn+1bundle=32；无despawn的未齐分支可读phase仍≤32。此只是约束和式，必须含全部生成handler/绑定/请求/字符串的真实成本；不成立回规划，不默提限、自动Idle、删验收或改布局排除合法路径。

旧023的pad-stay派生变体33/32与旧Idle行为继续作为旧档事实/负观察。新模式按作者定义的运动/业务比较，不要求作者照抄旧预算门。保G12a八旧oracle、原两币运动/声音/许可与G12b H01..12全部适用义务；引擎测试不能代替AG006内容接受。声音/HUD底层4请求/8voice、四槽每32 ASCII/总128、固定Lilita One/OFL、资源8/32MiB/64MiB与atlas/warm/owner/故障/native边界均保留。ToDesk设备观察不等于物理声学输出，该gap不消失。

## 进入、验证与消费

每Plan T-01重绑当前主线、已交付Spec、固定SDK7b987ca8…的681内容锁/两图依赖与真实owner；T-02最多两接法做完整可行性先证，再冻结准确语法/CLI/profile/预算/独立oracle。此处列出的typed种类/模块路径是目标，不能写成当前已实现事实。两份合同都具有AC/V-S/SD/任务归属，独立review在同一权威树的不同上下文完成；本轮不创建/联系agent或chat。

新source-build准备必须绑定真实generator和runner/SDK/锁/toolchain/模块及资源许可闭包。冷A→B仅持闭包与固定工具再生并真实运行、SaveAs关开重开、坏件拒绝→精确还原→同入口再可用；不是hash-only或旧截图。旧exact锁保持严格，原验收按真实受影响面复用/重跑，不为日常合入跑全AG。只记录实测数量，partial/empty/旧报告拒绝；新增产品规范在merge后才沉淀。

UI024拟唯一树ui-024/auto-ui、UI025拟ui-025/auto-ui（均codex/plan-NNN-dev）；暂未建立。临时材料唯一plan-artifacts/plan-NNN/auto-ui，开工登记owner/用途/在途，merge摘要折Plan并对账全部实际输出后删除自有scope/构建物，仅具名未解决问题按owner/理由/关闭条件登记持有。禁止链接、attachments含md、跨仓同号覆盖与派生JSON写入；清树先fresh guard。

本轮仅分配这两份实质合同，下一本地号026以后重新核；不立Gallery迁移号、不修订AG006/009/010、不改变当前6/21完成计数，024/025不会悄悄成为任何在途内容Plan前置。预算抽象/完整数值语义/ACE等独立路线继续等待相应成熟阶段。
