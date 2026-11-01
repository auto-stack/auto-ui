# AutoGraphics 三维声明/静态资源/组件基础作者入口（auto-ui PLAN-024 SD-01 候选）

Status: candidate（auto-ui PLAN-024 T-03..T-06 实测交付；本稿为
docs/specs/auto-graphics/node3d-foundation.md 的 SD-01 候选文本，merge
阶段按收据落位。只描述实际交付并验证的能力，不声称任何未实现的
语言/运行时特性。）

## 0. 范围

`node2d-lower --profile components-3d` 接受普通 Auto 模块工程，把
**显式 view-3d 三维作者入口**落在 020 的组件/模块/receiver 机制上：
普通 `use` 导入的组件定义与根 node 均可携带 `view-3d { ... }`，元素为
`cylinder3d`（现有 Cylinder 图元）、`model3d`（静态平色 GLB，typed
Model3D 资源/参数引用）与显式 `group3d`（同维父组）；每个根/子 Node
从首版即可内聚 `model`/`msg`/`on`（Life.Start/Life.Tick + 匿名
`.Ev`），局部状态驱动其自身叶的三轴 translation 与 visible。纯场景
（全工程无 model/msg/on）生成 `behavior: null` 的静态场景，不强制
用户 Auto/VM 角色或相机。生成 `autographics.scene/6` 工程
（`behavior.api = autographics.behavior/1`，actors null/null）与真实
用户 Auto 源，经既有 v1 机制执行：文档节点直写（`ag_set_local`/
`ag_set_visible`，行句柄 = driver `hierarchy::ordered()` 先序 0-based
下标）+ 按变更计费的逐 tick 呈现同步。三条产品链的执行证据（只述
实测）：**真 GPU 链**（driver node3d_basic_flow 6/6，单线程）覆盖
g02（原 B02-01..04：运行成功、红/绿部件像素 >20 原阈值、inspect
document == 运行 document）、g03 三变体（原 B03-01..06：world
(1.8,1,0)、父 x+2→3.8、祖先隐藏 AND、两次真运行、capture 不同、Run
不改文档/Stop 恢复）与 twin（双同定义组件独立 phase/位置/显隐、两个
typed 父接收载荷各自正确、GPU world 跟随、20 轮 fresh 零 GLB 重上
传）；**CPU 会话**（受信 VM）覆盖 SoA 独立性、Start 恰一次、tick 一
一对应、fresh 重建；**载入层**覆盖 129 行拒绝与坏 world/GLB 单字节
拒绝→精确还原→再运行。V-01 降层诊断（14 条具名负控）与 V-06
verify_plan024（29 控制组：身份链/冷 A→B 字节等值/变异敏感/注释不
敏感/MIT 许可/消费面）全部成立。

非范围（不冒称支持）：3D 角色/运行相机/实例生成销毁/触发/查询、纹
理/材质系统扩面/骨骼/节点动画、动态资源/拓扑/key/HMR、运行期
scale/yaw/pitch/TRS、任意 3D 模型格式、view 与 view-2d/view-3d 混合
（单 view 出口具名拒绝）、组平移/可见的 model 绑定（组静态）、
objects3d 复制（不强迫无角色场景声明假 Player）。**旧入口不变**：
016 legacy、017 g11-2d、019 basic2d、020 components-2d、021
queries-2d、022 sprites-2d、023 game 字节冻结（verify_plan016/
017/019/020/021/022 全部 PASS；pilot 61 测 + 驱动 node2d_basic_flow
13/node_components_flow 9/node_queries_flow 11/node_sprites_flow 4
全绿复跑），不被自动升级。

## 1. 声明面（T-02/T-03 冻结拼写）

- 域标记：`view-3d { ... }`（View+Sub+3）；同一 node 携带 view-2d 与
  view-3d 具名拒绝（single view exit per node）。view 块可省略
  （仅逻辑组件）。
- `cylinder3d name { props }`：`position: (x, y, z)`（三分量，必需）；
  `scale: (sx, sy, sz)`（缺省 [1,1,1]，静态）；`yaw`/`pitch`（缺省
  0，静态）；`visible`（缺省 true）。`rgba`/材质不暴露（3D 材质固定
  [0.9,0.3,0.2,1.0]，未知 prop 具名拒绝）。
- `model3d name { props }`：在 cylinder3d 面之上增加必需 `model:`
  绑定 = `resources.<name>` 或 Model3D 参数（符号引用，名字不是整
  数 ID；错资源端口具名拒绝）。
- `group3d name { position; visible; children 混合块 }`：仅平移与可
  见性；翻译/可见绑 model 具名拒绝（叶才绑 model）。场景 parent 只
  由显式 group3d 决定，Node 组合本身不隐式叠加 transform。
- 实例化：view-3d 内 `NodeName instance_name { params }`，可嵌套；
  其实例的 view-3d 并入同维场景，行名 `{path}/{elem}`、组名
  `{path}/{group}`（场景行名唯一，重复实例路径具名拒绝）。
- 参数端口：int/float/Model3D（`ptype_of_3d`）；root 不得声明参数；
  参数只读（handler 赋值具名拒绝）；msg 载荷数值（Model3D 载具名拒
  绝）；数值参数绑资源具名拒绝。
- 动态面仅叶 `position` 三轴与 `visible`（任一 `.field` 绑定即动
  态）；scale/yaw/pitch/model 绑 model 具名拒绝（Run 内不可变）。
- 池化纯函数、handlers、选择器/emit 投递沿 020 全套合同。

## 2. 装配与句柄（T-02 冻结，实测修正一处）

- 行句柄 = driver `hierarchy::ordered()` 先序 **0-based** 下标
  （`ag_set_local(fields[node*12+axis])` 寻址、`ag_valid` 允许 0）；
  文档行 id 独立为 1-based 标签（scene JSON `id` = 下标+1，`parent`
  指向 1-based id）。实施期间以真 GPU 帧读数修正过一版 1-based 句
  柄（旧期望由 v04 CPU 会话证明不成立）。
- 每定义 SoA：model 字段、运行期参数数组（handler 引用面）、动态叶
  行表 `<prefix>_<elem>_rows=[每实例句柄]` 与缓存
  `<prefix>_<elem>_px/py/pz/vis`（初值 = 静态初值）；root 动态叶为
  标量 `r_<elem>_*`。呈现 fn `<prefix>_present(i)` 在 tick 末同步：
  位置按平方差门、可见按 int != 门，写
  `ag_set_local(row,0/1/2,…)`/`ag_set_visible(row,…)`。
- handler 空间服务（v1 面）：`refs.<elem>.place(x,y,z)`（语句，三
  条 ag_set_local）、`refs.<elem>.shift(dx,dy,dz)`、
  `refs.<elem>.show(b)`；值位置 `refs.<elem>.local(axis)/world(axis)/
  visible()/row()` 与 `self.*` 同型。place/shift/show 值位置具名拒
  绝。021 二维 place/shift/show 行为不变（space3 环境缺席走旧路）。
- app_start：Life.Start 们（根先、声明先序）；app_update 冻结序：
  根 main_lifetick → （root present）→ 每实例 lifetick → present。
  无 char/solid/camera/spawn 面。
- emit 修正：实例 → root 直呼生成 `main_<ev>(args)`（旧模板缺左括
  号，实例→实例投递字节不变）；root_ctx 直呼同样修复。

## 3. 资源与身份（V-05/V-06）

- `model3d.glb("path.glb")`：身份 = 完整原始字节 SHA256 + byte_length
  + glb 容器魔数（生成器只读原始字节与哈希，不解析 GLB 结构）；缺
  文件/非 glb 容器具名拒绝。几何/域准入全在 driver 既有受限 decoder
  preflight（分配前：budgets raw≤8MiB/JSON≤1MiB/nodes≤128/expanded
  primitives≤16/每 primitive≤65536 vertices/每 asset≤262144/indices
  ≤786432、平色材质、无纹理/动画/皮肤/morph）——本期零改动，按未
  触碰代码复用原证据。
- 产品四件：`scene.agscene`（behavior.source 绑 world sha256 与
  byte_length；assets 清单）+ `world.at`（有状态链）+ `identity.json`
  （generator `autographics-node2d-pilot plan024 components-3d`、api
  `autographics.behavior/1`、user_source sha/bytes、逐模块 sha、
  closure 清单、roles 行表、assets sha/declared、provenance
  product/consumer/consumption/load_verification/world_sha256）+
  `closure/<module>.at`（冷再生唯一输入）+ `assets/<sha256>.glb`
  字节等值复制。
- 资源许可：原 G02 GLB（sha 42c9da67…，1528B，MIT，PLAN-838 自研）
  以只读副本进 fixtures/node3d/{g02,assets}/，许可文本
  fixtures/node3d/licenses/two-parts.LICENSE 进闭包。

## 4. 包络与计量（V-05 实测）

- **8 有状态实例**（root 含 model/msg/on 面才计入；纯场景为 0）：第
  9 个具名拒绝；envelope8（root+7）降层 Ok。
- **128 场景行**（组+对象）：128 Ok / 129 具名拒绝（降层）+ driver
  载入层同拒绝（node resource limit 128）。
- v1 无 32 计费池：空间写按 VM 原额（1000 steps/slice、2ms 整 tick、
  callback 100000 steps/120 resumes/250ms、init 30s/tick 2s、
  1-inflight/四 pump credit）+ 完整 tick 记账；呈现写按变更计费门。
- tick 墙钟观察（twin CPU 会话 120 tick）：**avg 657µs / max 990µs**
  （对宿主 8ms = 8000µs 帧上限 ≥8× 余量；CPU 会话口径，独立报告，
  不冒充宿主帧时延保证）。
- **512 潜在 mesh 实例**（含隐藏项）：原实现未被本期触碰
  （host/viewport.rs prepare_frame + play_adapter）；plain-GLB 域每件
  ≤2 primitives、文档 128 行包络下 potential 上界 128×2=256 < 512，
  上限沿原证据复用（观察：plain 链 128 包络先于 512 上限生效；
  tests/asset_feasibility 目标在 master 上因 PLAN-013 hud_overlay 字
  段早已失编译，属先在状态，非本期破坏，零 diff）。

## 5. 生命周期（V-04 实测）

- `Life.Start` 每 Run 恰一次（twin 双载荷 lo=1/hi=1；fresh 重建后重
  放恰一次；tick 不重放 Start）；`Life.Tick` 与派发严格一一对应；
  暂停下单次 Step 恰一个中性 tick；render-only 帧不推进任何 model。
- visible=false 保叶身份、不等于销毁：隐藏实例的 model/Tick 继续
  （twin right 隐藏后 phase 继续推进，77 tick 实测 4.2667=3.0+76/60）。
- Stop 恢复文档默认；20 轮 stop→fresh 全部重建（计数复位、无旧 Run
  泄漏、文档不变）；暖资源 owner：首帧后 mesh_uploads 基线 20 轮不
  变（GLB 不重解析/重上传）。
- 初始化失败面沿 020/023 宿主合同（last-good 保留、同入口还原绿）。

## 6. 诊断与消费合同（V-01/V-06）

- 执行前具名拒绝（`module <m>, line <n>, symbol '<s>'` / 载入层
  `file:line:`）：缺 use（undeclared node）、实例环
  （`instance cycle: A -> B -> A`）、重复实例路径、第 9 有状态、129
  行、未知 prop、组平移/可见绑定、scale 绑定、双 view 出口、root 参
  数、参数写（params are read-only）、错资源端口（not a
  model3d.glb resource）、非数值 msg 载荷、未知 view 域标记、缺 GLB
  文件、坏 typed 绑定。负面集 tools/verify_plan024（14 条）与 pilot
  node3d_basic_gates 双重复放，不 panic。
- 冷 A→B：closure + GLB 字节即可在 B 侧字节等值再生 scene（+world）
  （verify_plan024 五产品全过）；变异敏感（p0 3.0→4.0 改变 world
  sha 与场景行）、注释不敏感（byte-equal world）；消费面
  node3d_basic_flow.rs include_str! 冻结五产品，driver 载入路径哈希
  校验 behavior 源与每个 assets/<sha>.glb，坏件载入拒绝（v06 实测：
  world 半字节翻转与 GLB 单字节翻转均拒，精确还原同入口再绿）。
- provenance：identity 记录 product/consumer/consumption/
  load_verification/world_sha256；consumer 文件存在且 include_str! 消
  费（verifier 核）。

## 7. 验收映射

- AC-01（基线/先证）：T-01/T-02（本 spec §1/§2；三链真 GPU/CPU 双
  面）。
- AC-02（view-3d/typed 资源/组件符号/错误定位）：§1/§6（gates 7 +
  verifier 负控复放）。
- AC-03（G02 五 + G03 六 + 局部状态/事件→真 GPU）：§0（flow v02/v03）。
- AC-04（实例/父 receiver 隔离、Life/旧 Run/20 轮 owner/warm）：§2/§5
  （flow v03/v04 + v05 step/warm）。
- AC-05（原 API/VM/source/128/8/GLB/512 包络及反例）：§3/§4（gates +
  flow v05 + 未触碰代码复用）。
- AC-06（公共准备/实际来源、离线 A→B/坏件恢复、旧 2D 兼容）：§3/§6
  （verifier 29 组 + 旧入口六 verifier + 旧 flow 四套全绿复跑）。

## 8. provenance

本候选由 auto-ui PLAN-024 T-06 建立：事实行为沿 T-02 冻结合同
（Plan §9）与 T-03..T-06 实测（pilot node3d_basic_gates 7/7、driver
node3d_basic_flow 6/6 单线程、verify_plan024 --controls 29 控制组、
冻结六 verifier 全 PASS、旧 flow 四套全绿；twin world sha
b7bedd5a9287956f4fefc85646202f5d339e5b378a1ca95e50fb0ec8e23ed52e）。
016..023 既有 spec 与锁保持原状，不在本文件重述。
