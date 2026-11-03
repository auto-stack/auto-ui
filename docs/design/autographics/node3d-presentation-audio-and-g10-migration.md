# 3D Node表现、声音/HUD与G10新作者迁移

> 2026-11-03后续核验：UI034交付93eaa907、AG015交付80e0b65，当前19/22；UI035仍原owner实施3/10。034/015收据为同实现会话复核，独立上下文证据与相关持名hold不据归档自动推定已闭。下一批UI039/AG016/AG017见[查询与完整G3设计](node3d-queries-and-complete-game.md)。下文为本批立约时快照，原合同与预算不改。

2026-11-03，设计v1。用户明确授权本批规划：auto-ui补3D表现与声音/HUD接线，ag-examples迁G10纹理/动画两个入口，原预算和验收不减；仅设计/合同/导航，不实施、建树/探针/素材、构建/GPU或创建agent。本设计承接[共同作者模型](declarative-authoring-model.md)和[M2游戏优先路线](m2-game-first-three-lane-roadmap.md)。

## 1. 实际基线与缺口

UI005纹理（351919c）、006节点动画（c681663）、012/013公共声音/HUD、027角色相机（f3dea0d）、028对象触发（ebac1cd）、030具名引用（804d7303，独立final后补齐）均已交付。032完整二维组合（a8c04d22）和AG014完整2D游戏（c141df8）已独立review/实际合入/归档清理。AG013 G11/3d也已交付b48b304，当前逐入口活源17/22；尚未迁G04两维、G05/3d、G10两入口。

当前Node三维只接平色GLB与角色/相机/对象/触发/具名参数；pilot ResourceKind::Model3D记录root哈希，无纹理/动画closure边和pose服务。audio3d/hud3d、ag_present3d早已存在，025只接二维Node，032有界组合只为components2d。底层支持存在不能当作新Node三维表达/完整G3已交付。

G10当前两入口固定006旧source/runner，共一个内容项/完整双入口包，纹理9与动画8共17命名oracle；两份旧合同/原源/阈值/负控必须承接。本文只扩大新作者接线，005/006/音频/HUD支持域及全部旧profile价格/字节保持。

## 2. 本批三合同与阶段依赖

| 合同/责任仓 | 范围 | 可并行准备 | 完整实施硬门 |
|---|---|---|---|
| [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md) / auto-ui | typed纹理/动画闭包、clip/目标、局部组件/原模型body、原G10十七项新生成链 | 005/006/030已实际交付；开工核owner/SDK/原件 | T-02最多两机制全链先证/冻结，不只parse |
| [UI035](../../plans/035-autographics-node3d-audio-hud-composition.md) / auto-ui | 具名cue/HUD通用服务、单视觉bundle、新显式三维有界组合及完整G3八项公共夹具 | T-01/02原资源/许可/oracle/接口/预算方案，不读034tip | T-03完整组合硬等034实际delivery/独立review/Spec/工具，再做最多两候选全链先证 |
| [AG015](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md) / ag-examples | G10纹理/动画两入口普通Node迁移，原17和全部内部控制、双入口包 | T-01原件/许可/原17/包准备 | T-02完整新链硬等034实际交付；**不依赖035**，不追加声音/HUD |

三份均drafting/r1、0/10。先034实现、035与AG015准备；034实际交付后035完整接线与AG015完整迁移可并行，各自固定实际版本，不追tip。035/AG015彼此无完成依赖。完整3D游戏未来另立ag-examples/games/platformer-3d合同，不本轮取号或创建目录；2D继续保固定032，不因3D能力自动升级。

## 3. 034：资源闭包、typed端口与通用表现

资源入口显式区分旧平色GLB、静态纹理GLB、有限节点动画GLB，名字为提案、准确形式034 T-02冻结；use/资源参数可复用，clip绑定到模型资源、视觉目标绑定到文档模型leaf或完整Run/代次对象。原GLB/URI/图片原字节、ordered image边/sha/尺寸/closure ID及原clip ordinal保持；具名引用在lowering变原ABI，不把字符串拼接/数字scene/slot/pair索引转嫁作者。

根/子Node局部model/msg/on，普通组件/资源/helper可拆模块，同定义实例独立phase/yaw/receiver/数值参数；无角色纹理场景不制造假Player/相机。原G10动画character模型直接作为body，保其实际初始世界AABB与非均匀scale，不换圆柱。clip/phase秒/loop/yaw弧度和动作选择由Auto提供；生成器只处理声明/公共服务，不自动Idle/计数/巡逻，不要求业务字段叫state/pose/facing。032保留字段仅属其已交付二维合同，不成为本片3D强制语义。

同一pose用于颜色/阴影；视觉不改physics/solid/support/trigger。无Host动画时间钟，Pause/Step/Stop/fresh及slot销毁重生仍按原语义，frame失败保既有profile完整last-good，不以此推出单模块/禁use/全世界故障策略。

资源机制最多两候选：公开import/Storage受检元数据供pilot绑定，或有限共享元数据层；须保持pilot语言图与SDK681 driver图隔离，不另造完整解码器/IO后门或强迫生成器初始化GPU。完整GLB/图像/轨道准入仍由driver原preflight独立执行，未使用声明也检查；actor-free纹理和原动画主alt都是真Auto/GPU先证路径。

## 4. 035：三维音画与完整G3公共整链

复用012/013 native/mixer/font/atlas及ag_present3d，一个视觉（kind=-1可无视觉）、四声音请求、四槽完整HUD；任意名字的局部Node用具名cue/clip/目标请求，gain/ordinal/文本/动作/胜败由Auto决定。actor-free音画和完整G3两条路径都必须证明。单独写/重复bundle/混写/坏ordinal/旧Run、资源/prepare/native故障沿原受控语义，已播PCM不回滚、不承诺硬件时刻一致。

完整G3夹具保原G3-01..08：相机相对四向/对角归一/朝向、地跳/落地、三维地墙顶/危险/越界、三币/目标、胜败fresh Enter、新局隔离、相机环绕缩放、原低多边形纹理/idle-walk-jump、jump-land-coin一次声音及n/3/胜败HUD；坐标/速度/相机参数见035。root+Player+三Coin+Hazard+Goal+Boundary恰8状态Node，纯模板/资源不计，普通use与局部性不缩水。

保原32池，T-03在034实际交付后最多A/B两候选完整先证：A复用原单条API；B新显式components3d/1（名称提案）复用032有限事件快照/批销毁语义。B价目公开：0..8完整上一Tick事件快照每callback一次1credit、纯Auto字段读取VM照计，第二次/混cursor/越界58；0..8完整不同live句柄全验证后批销毁1credit（空批亦收费），物理写仍按n，9项59/旧代33/第33物理写37。旧2D/3Dprofile bytes/价目不变，3D数据和Host独立准入须真实验证，不能直接拿2D成功代3D。

候选B收集分支input1+move26+grounded1+snapshot1+despawn-batch1+present3d1=31，只是待证和式；额外服务照计，所有可达最高分支/初始化/实际VMCPU分别测。不停移动、不跳事件、不漏第三件/必要声音或改原关卡凑预算。主地图不可达的多件/容量压力用公开边界夹具分列支持/拒绝，不能宣称实际路线到达。两候选均失败则功能未完成/needs_replan，不以no_go研究或抬额度结案。

## 5. G10内容保真与验收分工

[AG015 §5.2](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md)列原9+8名，保各名内部阈值/轨迹/原66状态控制、8坏证明及2发布回滚等适用义务。纹理保原三Kenney模型+UV六面、非对称图/alt、原KHR/sampler/材质/深度、main幅度.8/alt1.6和tick2隐藏/tick3恢复；动画保原character87624B、static/idle/walk/jump四ordinal、8nodes/44LINEAR/337samples，初始位置/非均匀scale/地面/原相机距6-pitch30、动作/重力/触发及主phase+=dt/alt+=2dt。

24UV ROI与独立CPU/f64 key/slerp/hierarchy/GPU silhouette-shadow等oracle保持；同输入240Tick visual开/关physics/camera/events除Run身份外相同。原normal7raw/150508B、含alt8raw/150787B和原六closure/poseCPU32592为内容保真基线；新生成层开销单独实测，不能为过预算删资源/轨道。

G10两个新活入口明确固定034 source-build/SDK/工具身份，公开main/alt、全部所需专项、完整双入口单项包。两入口A→B/SaveAs/关开/冷再生/坏件拒绝→精确恢复→同入口再Run都要新内容实际证据，不沿旧006绿截图。旧源/锁/收据保历史、旧exact门严格；新路由不升级其他Demo或游戏。净增0，实际交付后预计19/22，当前仍17/22。

引擎验证公共机制/价格/闭包/物理隔离和最小夹具；Gallery独立验证正式内容/原件/操作/发行包，未来完整游戏再独立验正式G3。035原完整G3夹具不是本轮游戏交付；M2统一版本/Gallery/两游戏原分母仍最后共同核验。

## 6. 原资源、预算与验证纪律

005/006纹理与节点动画支持域保持：opaque单/双面、baseColor×sRGB、UV0/KHR、nearest/linear LOD0，本地受检闭包；无PBR/透明/mipmap层/skin/morph/CUBIC/blend/rootmotion。原Kenney3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0及CC0/MIT/OFL来源固定，不下载当前上游替换历史源。

共用原raw8身份/32MiBunion/单件8MiB、CPU/GPU各64MiB、node128/clip16/channel128/samples8192/primitive16/mesh512；8状态Node与8slot/8pair是不同单位，120base+8reserved及deep/contact64base保持。credits32、lifecycle物理写32、managed96/user256/combined352KiB、1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s与8ms pump目标不变。音频4请求/8voice/8cue，四HUD槽每槽32ASCII/总128，固定LilitaOne/OFL-RFN、4atlas/1024边/4MiB和原服务域均保持；不给每组件/声音/HUD另开资源池。

每Plan只测真实影响面/必要消费者，不日常全AG/根crate；新验证工具未实现/未执行，不能把提案CLI记PASS。GPU/窗口/音频重型门同机单实例；真实人工、PCM/native路径和物理声学分开，缺必需证据具名；OS键鼠/焦点自动化暂停。final必须独立不同上下文，同权威树顺序，不以实现会话内重跑代替。

所有正式来源固定实际delivery/Spec字节/SDK681/两图依赖锁/toolchain/参数/真实工具hash和模块/资产/许可；current/history/pending/external owners合并计、20轮warm账/真实Weak和最终last-owner释放。新组合/内容的保存/再生/恢复实际测，不能只引用旧profile无关套件。临时材料各归plan-artifacts/plan-NNN/<repo>/，开工登记owner；merge摘要折Plan、全输出对账后删自有scope/构建物，具名问题持有；禁止链接/attachments/派生JSON，清树先fresh guard。

## 7. 后继与当前交付口径

本批只新增034、035和AG015三个合同；完整3D游戏、G05/3d查询、G04教程后继均不预占号。034可先实施，035/AG015先准备；034交付后后两者完整实现可并行，公共写面按owner短窗。035实际交付和AG015内容证据可为下一轮完整G3提供输入，不把整个Gallery迁完加作游戏启动前置。

截至本轮仅规划，三个新计划均0/10；当前Gallery17/22、AG014已交付和各历史边界分别记账，不静默把新合同当新能力。UI029/OS053和其他路线WIP保持原owner，本批不触碰其实现。
