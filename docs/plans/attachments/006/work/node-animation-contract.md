# PLAN-006 T-02 执行合同（r2输入修订待实施）

2026-10-30 当前入口：new已完成r2有界修订，按`input-budget-revision-r2.md`回work实现/验证S-06。原T-02固定向量probe没有证明完整输入预算，故仍重开；原GPU/存储/采样证据按未变输入保留，“完整组合预算已完成”不作为当前结论。目标/32credits/旧价/CPU/范围不变；下述聚合input API已成为当前设计，但尚未实施/实测。

原实施基线 `80d9bbe`，当前冻结代码基线`ccfe6d9`；输入及004/005/SDK/两锁身份见 `source-preflight.md`。r2只修订输入读取设计，不扩大r1范围、AC、资源或CPU/时间门。原始纹理角色已过Auto→Host→同帧真实GPU与输出校验（`t02-integration.md`），完整键控组合的预算待S-06补证。T-03已完成，T-04四clip数值/画面部分已有证据；其余物理/owner/fresh consumer及独立review/merge不预称完成。

## 公共profile与视觉状态

`autographics.node_animation3d/1`，CLI `--behavior-api node-animation3d`，version=110，profile=21；在冻结triggers3d/1的实际装配上追加视觉单元与freeze/tick hook，旧14个profile的装配字节必须不变。2D精灵仍由旧sprites2d/1承接，新3D profile保留对象/触发/相机组合，非跨维度混用Run。

单setter `ag_visual_pose(kind int, entity int, clip int, phase float, looping int, yaw float) int`：kind0为静态文档stable id，kind1为001完整Run/slot/generation token；显式kind避免文档id和token别名。返回1成功/0粘性错误。clip=-1为base、0..N-1为原文件ordinal；原character固定0/static、1/idle、2/walk、3/jump。名称仅元数据，不重排/合并clip。clip/phase/loop/yaw一次受检写，成本为共享32credits中的1credit，无零成本视觉旁路。每callback最多32次调用；错误后整个候选拒绝，不发布部分状态。

phase唯一单位f32 seconds，必须有限且非负；由Auto在fixed dt推进，Host不自行计时。looping仅0/1；loop精确duration回0、one-shot夹末端，duration0取单pose。新clip立即用本次phase采样，无隐式重置时钟。yaw单位radians，有限且abs<=1024*TAU（f32上界6433.98175455），Host归一[-PI,PI)。静止时Auto保留上次朝向，Host不读速度/grounded来代选clip或yaw。

文档默认视觉为base/-1、phase0、loop0、yaw0；新槽generation从模板初始视觉重建而非继承模板上次运行态或槽旧代。销毁撤销视觉，旧Run/旧generation/无合法模型绑定受检拒绝。静态绑定数至多128，与原Run节点上限相同；槽仍8。Stop与fresh Run重建app_start状态，Pause不改phase，neutral Step只推进一fixed dt。

## r2 聚合输入（设计已冻结，待work实现）

同一未发布profile追加`ag_actions_snapshot() int`，一次共享credit读取原Host送入的6held+6未消费pressed，返回`held + 64*pressed`（0..4095），bit0..5为Left/Right/Forward/Back/Jump/Restart。成功读取消费当前tick所有未读pressed，held保持；重复读取的pressed为0，和旧pressed getter混读不能重放已消费bit。app_start无输入为0，Pause/neutral Step/fresh Run沿原clear/路由；无跨tick或跨Run缓存、无新增Host wire、无Rust玩法。

该helper仅属于node_animation3d/1受管单元，旧14profile与旧逐键1credit价格/字节不改。Auto自行解码、算相机相对方向/跳跃/clip/phase/yaw。完整活动回调要求真实实测 `1 snapshot +26 move +1 grounded +1 event_next +1 event_pair +1 event_phase +1 visual =32`；旧逐键反例仍error10，新组合必须在真实事件读取分支fault0、输入改变实际运动/clip/yaw，额外1credit必须error10。S-06全部关闭规则/命令/代表性bit与mixed状态见`input-budget-revision-r2.md`，当前不标绿。

## wire（已接入并验证）

固定header四int：status=1、bound count、Run、tick。随后Run接受的稳定model target行（按Run顺序），最后8槽行。每行6值：四int+两float。static ints=`node ordinal,asset ordinal,clip,looping`，floats=`phase,yaw`；slot ints=`generation,asset ordinal,clip,looping`，floats同static。free/unbound槽六值均canonical0。Host独立检查实际model内容身份、类型/长度、clip域、finite phase/yaw、bool域、Run/tick、实例实际模板与generation、每个live bound模板有且仅有一行。用户可任意写视觉请求，Host只用此受检表产生候选pose。

## 节点/矩阵/资源路径

源原件的GLB JSON/BIN（外图closure不变）→有界结构、类型/跨度/时间/数值检查→不可变Animation→Auto视觉表→Host验证→Animation.sample→共同Prepared实例矩阵→颜色与shadow共用矩阵。

原层级base world以parent*local TRS组成；每个mesh仍有005 source node与invertible base world关系。视觉最终矩阵为`document/instance world * visual yaw at model origin * sampled node world * inverse(mesh baked base world)`，从而复用原geometry/UV/texture缓存且准确撤销005顶点的base world；无需每tick重写geometry或texture，不将动画结果写回物理节点。父层级继承规则与raw node对应必须由两层数学fixture及真实GPU断言同时关闭。颜色与shadow消费同一MeshInstance数组。

保留16clips/128channels/8192累计channel key samples、nodes128。每个channel仅STEP/LINEAR；rotation为正规化最短弧slerp，禁止分量lerp冒充。所有channel在分配keys/outputs之前完成metadata计数/类型/span检查，再查时间有限非负严格递增、输出形状/有限、正scale、单位quat、唯一node/path与无环唯一父层级。skin/morph/weights/cubic显式unsupported；数据损坏invalid，额度resource limit，不能静默丢轨道。

动画payload、轨道容器/名字/拓扑/keys/outputs和每实例有界pose scratch按实际类型大小纳入原64MiB CPU总账；GPU既有64MiB总账与512潜在mesh实例继续有效，raw8身份32MiB/单raw8MiB/JSON1MiB等不变。Animation核心`cpu_bytes`只提供payload+一实例scratch的保守账，调用方必须按潜在实例数另外预留scratch，不能把共享payload误当全部实例免费。

## 区分性检查与当前关闭状态

| 规则 | 实际路径/区分oracle | 当前状态 |
|---|---|---|
| V-01 原件与模式 | 原character全部4clip/44channel/337samples、层级；坏span/count/time/重复node/path、skin/morph/cubic与额度分类 | T-03生产准入29反例/精确额度/原件正控通过；资源union门前零key分配，见t03-admission-and-storage.md |
| V-02 数学 | 120度旋转在1/4时间必须30度；独立point坐标区分slerp与nlerp；反号quat同pose、STEP边界、loop/one-shot/零duration | core5/5与原四clip21状态、43596独立源顶点/2074207稳定GPU probe通过；实际阴影及完整T-04仍待 |
| V-03 朝向/物理 | Auto camera-relative movement+yaw；颜色/shadow相同矩阵；视觉开关同输入碰撞/支撑/触发结果相同 | 未关闭 |
| V-04 事务/身份/owner | Host坏输出/prepare failure保留完整last-good；Pause/Step/新局/Stop；20轮Weak/上传/最终drop | producer/19wire负控与T-03资源/保存恢复通过；完整事务/owner仍T-05 |
| V-05/S-06 预算/消费 | 真实完整键控Auto同callback输入/move/grounded/相机/event_next/pair/phase/visual；固定原clip经GPU；fresh A→B离线保存重开 | 原逐键输入反例error10，r2聚合读取与S-06待work；consumer/identity仍T-06 |

最小整链当前结果：公开`import-animated`/“Import Animated GLB”保留原GLB+外图；schema6 closure新增`node_animation=true`并以新profile域计入closure id，旧false省略且旧identity保持。保存搬迁冷重开重建4clip，旧import拒动画，旧Play profile拒新动画payload，不能静默丢轨道。新Host验证wire并在共同Frame内sample；19项坏wire负控及最终矩阵非有限/奇异拒绝通过，颜色/shadow共用同一矩阵。固定位置/相机/yaw/mesh/texture只擦除node轨道的GPU负控使像素变化。单tick全链3.836ms，默认100000-step/120-resume/250ms callback与2s整tick均未触发；此为最小探针，非T-06完整性能/消费验收。
