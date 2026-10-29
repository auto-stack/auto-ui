# PLAN-006 T-02 当前执行合同（探针建设中，未通过整链门）

基线 `80d9bbe`，输入及004/005/SDK/两锁身份见 `source-preflight.md`。本合同固定本期普通实现选择，不扩大r1的范围、AC、资源或CPU/时间门。当前采样/Auto producer骨架在唯一ui-006树建设；直到原始纹理角色经过Auto→Host→同帧真实GPU且整组合预算实测通过，T-02保持未完成，不启动T-03广泛准入/持久化矩阵。

## 公共profile与视觉状态

`autographics.node_animation3d/1`，CLI `--behavior-api node-animation3d`，version=110，profile=21；在冻结triggers3d/1的实际装配上追加视觉单元与freeze/tick hook，旧14个profile的装配字节必须不变。2D精灵仍由旧sprites2d/1承接，新3D profile保留对象/触发/相机组合，非跨维度混用Run。

单setter `ag_visual_pose(kind int, entity int, clip int, phase float, looping int, yaw float) int`：kind0为静态文档stable id，kind1为001完整Run/slot/generation token；显式kind避免文档id和token别名。返回1成功/0粘性错误。clip=-1为base、0..N-1为原文件ordinal；原character固定0/static、1/idle、2/walk、3/jump。名称仅元数据，不重排/合并clip。clip/phase/loop/yaw一次受检写，成本为共享32credits中的1credit，无零成本视觉旁路。每callback最多32次调用；错误后整个候选拒绝，不发布部分状态。

phase唯一单位f32 seconds，必须有限且非负；由Auto在fixed dt推进，Host不自行计时。looping仅0/1；loop精确duration回0、one-shot夹末端，duration0取单pose。新clip立即用本次phase采样，无隐式重置时钟。yaw单位radians，有限且abs<=1024*TAU（f32上界6433.98175455），Host归一[-PI,PI)。静止时Auto保留上次朝向，Host不读速度/grounded来代选clip或yaw。

文档默认视觉为base/-1、phase0、loop0、yaw0；新槽generation从模板初始视觉重建而非继承模板上次运行态或槽旧代。销毁撤销视觉，旧Run/旧generation/无合法模型绑定受检拒绝。静态绑定数至多128，与原Run节点上限相同；槽仍8。Stop与fresh Run重建app_start状态，Pause不改phase，neutral Step只推进一fixed dt。

## wire（待Host接入验证）

固定header四int：status=1、bound count、Run、tick。随后Run接受的稳定model target行（按Run顺序），最后8槽行。每行6值：四int+两float。static ints=`node ordinal,asset ordinal,clip,looping`，floats=`phase,yaw`；slot ints=`generation,asset ordinal,clip,looping`，floats同static。free/unbound槽六值均canonical0。Host独立检查实际model内容身份、类型/长度、clip域、finite phase/yaw、bool域、Run/tick、实例实际模板与generation、每个live bound模板有且仅有一行。用户可任意写视觉请求，Host只用此受检表产生候选pose。

## 节点/矩阵/资源路径

源原件的GLB JSON/BIN（外图closure不变）→有界结构、类型/跨度/时间/数值检查→不可变Animation→Auto视觉表→Host验证→Animation.sample→共同Prepared实例矩阵→颜色与shadow共用矩阵。

原层级base world以parent*local TRS组成；每个mesh仍有005 source node与invertible base world关系。视觉最终矩阵为`document/instance world * visual yaw at model origin * sampled node world * inverse(mesh baked base world)`，从而复用原geometry/UV/texture缓存且准确撤销005顶点的base world；无需每tick重写geometry或texture，不将动画结果写回物理节点。父层级继承规则与raw node对应必须由两层数学fixture及真实GPU断言同时关闭。颜色与shadow消费同一MeshInstance数组。

保留16clips/128channels/8192累计channel key samples、nodes128。每个channel仅STEP/LINEAR；rotation为正规化最短弧slerp，禁止分量lerp冒充。所有channel在分配keys/outputs之前完成metadata计数/类型/span检查，再查时间有限非负严格递增、输出形状/有限、正scale、单位quat、唯一node/path与无环唯一父层级。skin/morph/weights/cubic显式unsupported；数据损坏invalid，额度resource limit，不能静默丢轨道。

动画payload、轨道容器/名字/拓扑/keys/outputs和每实例有界pose scratch按实际类型大小纳入原64MiB CPU总账；GPU既有64MiB总账与512潜在mesh实例继续有效，raw8身份32MiB/单raw8MiB/JSON1MiB等不变。Animation核心`cpu_bytes`只提供payload+一实例scratch的保守账，调用方必须按潜在实例数另外预留scratch，不能把共享payload误当全部实例免费。

## 区分性检查与当前关闭状态

| 规则 | 实际路径/区分oracle | 当前状态 |
|---|---|---|
| V-01 原件与模式 | 原character全部4clip/44channel/337samples、层级；坏span/count/time/重复node/path、skin/morph/cubic与额度分类 | 核心test已建立，非生产assets入口 |
| V-02 数学 | 120度旋转在1/4时间必须30度；独立point坐标区分slerp与nlerp；反号quat同pose、STEP边界、loop/one-shot/零duration | 核心test已建立；GPU画面待接入 |
| V-03 朝向/物理 | Auto camera-relative movement+yaw；颜色/shadow相同矩阵；视觉开关同输入碰撞/支撑/触发结果相同 | 未关闭 |
| V-04 事务/身份/owner | Host坏输出/prepare failure保留完整last-good；Pause/Step/新局/Stop；20轮Weak/上传/最终drop | producer坏clip/phase/loop/yaw/旧token入口已建立；Host及owner未关闭 |
| V-05 预算/消费 | 真实Auto同callback运动（含垂直跳）、grounded、相机basis读取、触发event一次读取、视觉setter；固定原clip经GPU；fresh A→B离线保存重开 | producer组合test已建立，尚未把其结果当整链预算；consumer/identity待T-06 |

T-02通过仍须完整实际生产路径/原纹理角色、矩阵与画面、预算/CPU/步数及所有wire校验真实执行；本附件不预称AC-01..05完成。
