# AutoGraphics 三维角色/运行相机/内置视觉声明接线（auto-ui PLAN-027 SD-01 候选）

Status: candidate（auto-ui PLAN-027 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node3d-character-and-runtime-camera.md 的 SD-01
候选文本，merge 阶段按收据落位。只描述实际交付并验证的能力，不声称
任何未实现的语言/运行时特性。）

## 0. 范围

`node2d-lower --profile components-3d-runtime` 在 024 的共享三维组件
管线上新增三类声明接线，把**公共 3D 角色与运行相机**接到普通
`use`、`node`/`model`/`view-3d`/`on`（Life.Start/Life.Tick + 匿名
`.Ev`）和 typed 引用上：

- `character3d { }`（node 级）：本定义绑定为本 Run 恰一物理角色；
  生成 `app_start` 注入 `ok=ok+ag_char3_bind(<文档 id>)`。角色体 =
  恰一 `cylinder3d` 叶、静态 position、冻结 scale 0.4×0.5×0.4
  （受信 `_ag_char3_shape` 0.8×1.0×0.8 box 的编译面早具名版本）。
- `solid: true`（cylinder3d 叶 prop）：角色 solid（全静态 AABB；
  按 root 先/实例声明序注入 `ok=ok+ag_char3_solid(<文档 id>)`）。
- `camera3d { distance/pitch/projection/ortho_size }`（仅 root）：运行
  相机一次性配置，target 恒为本 Run 恰一角色；值域沿
  runtime_camera3d/1 冻结域（distance [4,12]、pitch [15,75]、
  projection perspective|ortho、ortho_size [1,64]）编译面具名预检，
  生成 `ok=ok+ag_camera_orbit_3d(<角色 id>, …)`。
- `player3d`/`mob3d`（view-3d 叶）：具名内置视觉，映射原
  Kind::Player/Mob 与原 player.glb（05d8588e…，52392B）/mob.glb
  （8a82adb5…，63828B）mesh/材质；字节沿 driver 原 owner 暖上传
  （圆柱+player 3 网格+mob 3 网格常驻），声明不携带资源字节，
  Kind 不等于角色身份、不可 solid、不可 scale/yaw/pitch。

行为/装配身份由声明选择：`camera3d` →
`autographics.runtime_camera3d/1`（marker 104/profile 15）；`character3d`
→ `autographics.character3d/1`（marker 102/profile 13）；仅有为行为
的场景 → `autographics.behavior/1`；纯静态场景 `behavior: null`（无
角色/相机/用户 VM 可消费）。驱动 `start_selected_play` 按该串装配。
输入沿六动作冻结表 Left/Right/Forward/Back/Jump/Restart=0..5；
控制公式（重力/跳跃/方向组合/对角归一）留在作者 Auto；编译器只映射
`refs.<elem>.move3d(dx,dy,dz)`→`ag_char3_move(…,0.002)`、
`.grounded(q)`/`.support(q)`、`camera3d.forward_x/forward_z/right_x/
right_z/yaw/pitch/distance()`（零空间计费读取）。同一 profile 保持
024 全部静态能力（cylinder3d/model3d/group3d/020 局部事件）。
PLAN-043（SD-02）收口 + PLAN-064（SD-01 修订）：`ag_char3_move` 的滑扫
跳过不可见文档行（`visible_only=1` 语义）——parked spawn 模板与隐藏表现
行永不阻挡角色行走；但 `ag_char3_solid` 注册的 solid 行在滑扫期间被临时
视为可见（`effective` 置 1、调用返回前恢复），因此 `solid:true` +
`visible:false` 的行（如 platformer-3d 的隐形平台板）仍阻挡。043 的“全部
已交付内容的阻挡行均可见”前提被反例证伪；可见性不再代理 solidity，
solidity 以 `ag_char3_solid` 注册为准（g14 轨迹对照仍逐 tick 逐位相等）。

## 1. 执行证据（只述实测）

**真 GPU 链**（driver `node3d_runtime_flow` 7/7，单线程 64MiB 栈）：
原 G07 layout-a 九项（import 3 圆柱、60 forward |z|>3.5、y≈0.5、
800×400 windowless PNG >10000B、60 中性 tick 位置逐位不动、speed
4→5 ≥1.2×、对角=轴向 1e-3、x 墙 5.498±0.02、单跳沿 apex>1.5 落回
0.5±0.01）、layout-b 两项（4 对象、同源几何 pit 落差 y<−3）与 G08/3d
十三项（yaw0/45/8/perspective 源配置、tick0 VP==glam oracle 精确、
10 tick VP 随新位置且非旧、drag 90° VP==oracle、orbit 后 forward
=+x（|dz|<1e-3）、yaw0/yaw90 像素不同、Paused zoom 丢弃、Running
zoom 8×1.1^8 夹 12、Stop 文档逐字节一致、fresh Run 原点重启、alt
6/30/ortho16 配置、alt VP==正交 oracle、冻结世界轴假源反向对照）全
部经生成链逐名复刻成立。**typed/预算拒绝**（V-S03）：同一 tick 第二
move → fault 10；旧 token 读 → fault 13；相机 target 非角色 → 30；
重复配置 → 29；域外 distance → 27；127 注册 solid 全重叠足迹 → INIT
即被原累计步门整帧拒绝（"127solid 初始化拒绝"负控保持；128 行文档
边界在 2 注册 solid + 125 非固体柱下通过并走步）。**输入/相机事件**
（V-S04）：blur 清 held、Step 中性零漂移。**内置视觉**（V-S05）：
两次捕获逐位稳定、与 ground-only 文档捕获不同（真实渲染）、暖存储
恰 7 网格。**生命周期**（V-S06）：B 闭包目录真 Run、SaveAs 副本关开
重开、坏 world 单字节载入拒绝→精确还原→再 Run、20 轮 start/stop
暖网格账平坦。**编译面**（V-S01）：`node3d_runtime_gates` 9 门（字节
稳定/注入序/六动作/相机读取/SoA/内置 kind/behavior null/接收器隔
离/双反例降层）+ 9 条具名拒绝；`verify_plan027 --controls` PASS
（11 产品身份链/冷 A→B 字节等值/speed 变异敏感/注释不敏感/内置
GLB 来源与许可闭包/消费面对账）。**旧档冻结**：024
verify_plan024 --controls PASS、020 verify_plan020 --controls PASS、
pilot 全部既有 gates 目标绿（byte-stable 复用）。

非范围（不冒称支持）：Gallery 业务修改（另仓）、多角色/动态 Node
子树/销毁/触发服务、三维查询、纹理/骨骼/模型动画、运行 group TRS、
多呈现/Life.Frame、025 声音/HUD、生产 UI、ACE/AOT/ECS/跨端、提高
预算。**旧入口不变**：024 components-3d 及更早 profile 字节冻结
（verify_plan016/017/019/020/021/022/024 PASS；pilot 既有 gates 全绿
复跑），不被自动升级。实测边界记录：127 注册 solid（含角色）经完整
宿主链在 INIT 触发原累计步门拒绝——"128 稀疏"原批准包络按 plan-874
的 128 行扫描口径保持（行 ≠ 注册 solid），不构成预算提升。

## 2. 冻结拼写与所有权（T-02/T-03/T-05）

- 生成 `app_start` 注入序：作者 Life.Start 们（root 先、声明先序）
  之后：bind → solids（root 先/实例声明序 = 文档 id 升序）→ camera。
  主机在 app_start 之后织入 `_ag_char3_freeze`/`_ag_cam3_freeze`。
- 文档 id（1-based）= 场景对象表序；bind/solid/camera 的 stable id
  即该 id；typed MoveToken 语义沿受信单元（token0 = freeze 初始支
  撑态可读、latest 可读、旧 token/错 Run/错 kind 拒绝且无误写）。
- 初始 position 与运行 position 分所有者：角色体 position 静态（绑
  定具名拒绝），运行 position 只由 move3d 产生；solid 全静态。
- 32 空间计费沿受信单元：4 held + 1 pressed + 26 move + 1 grounded
  = 32；support 附加读仍 1；相机读取零空间费。VM 原额（1000
  steps/slice、2ms slice、100000 steps/120 resumes/250ms 回调、
  init30s/tick2s、8ms pump）全保。

## 3. 来源与寿命闭包

内置模型字节身份 = 原型仓冻结资产
（experimental/autographics-prototype/assets/squash_the_creeps/art/
player.glb、mob.glb；godotengine/godot-demo-projects@b761b4cd，
模型 MIT，见原型 README 资产来源行）；driver `Host::with_project`
按原 owner 暖上传一次，实例声明不重传、不复制、不重编码。作者夹具
的永久源 = pilot `fixtures/components3d-runtime/`；生成产品 =
`generated-components3d-runtime/`（字节冻结，verify_plan027 再生
门守护）；消费 = driver `tests/node3d_runtime_flow.rs`（include_str!
冻结产品字节，provenance 逐产品对账）。
