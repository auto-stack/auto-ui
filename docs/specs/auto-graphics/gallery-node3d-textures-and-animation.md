# gallery-node3d-textures-and-animation

SD-01 candidate（PLAN-034 交付；canonical 落地由 merge 核验）。
G10 纹理/动画双入口从旧直呼脚本与 005/006 锁面迁移到普通
components-3d-presentation 作者链。原件/许可/预算/旧档历史保持不变；
本 Spec 只新增作者面规则，不改变任何运行时受信单元。

## 1. 具名资源构造（textured3d.glb / animated3d.glb）

- `let platform = textured3d.glb("assets/platform.glb")` 声明静态纹理 GLB；
  `let hero = animated3d.glb("assets/character.glb")` 声明原节点动画 GLB。
  单字符串实参、`.glb` 后缀、glTF magic、原始字节 SHA+长度身份；作者不写
  数字 scene/slot/pair/clip 索引。
- 旧 `model3d.glb` 面不变；平色构造器声明的文件若被既有公开导入揭示为
  携图像/动画内容，pass B 具名拒绝（旧构造器不静默升档）。
- 闭包元数据（closure id/图像边/原 clip 序数）由既有公开导入
  （driver `resource-meta` 走 `assets::import`/`import_animated` 同一独立
  准入面）产出；生成器保持 GLB 盲（不解析结构、无第二解码器）；几何与
  闭包准入最终由 driver 原preflight/`load_declared` 独立复验。

## 2. 生成管线（CLI 冻结）

1. `node2d-lower --profile components-3d-presentation --entry <dir>/main.at --out <out>`
   （声明面 → `<out>/resources-request.json`）
2. `resource-meta --entry <dir> --request <out>/resources-request.json --out <out>/resources.json`
3. 同一 lower 命令 + `--resources <out>/resources.json`（产品面：
   `world.at` + `scene.agscene` + `assets/<sha>.glb` + `assets/<sha>.png` +
   `identity.json` + `closure/`）。

装配身份：动画资源在案 = `autographics.node_animation3d/1`；角色/相机/
池任一在案 = `autographics.triggers3d/1`；纯静态纹理场景 =
`autographics.behavior/1`。场景对象以 `closure:<id>` 引用纹理/动画资源，
平色仍为 `sha256:<sha>`。

## 3. 具名 clip 与视觉姿态

- clip 具名 `clips.<资源>.<名|base|序数>`：`base` = -1（base pose）；数字形
  段 = 显式文件序（关键字名如 "static" 的拼写通道；数字形段永不作名解
  析）；未知名/重名歧义/跨模型错配具名拒绝。文件原 ordinal/名称保持。
- 姿态服务 `refs.<leaf>.pose(clip, phase, loop, yaw)` →
  `ag_visual_pose(0, <1基文档id>, …)`；leaf 限本定义绑定 animated3d 资源
  的模型叶（参数绑定叶不授 pose）。行表 0 基运行寻址 → 文档 id +1 换算。
- 动画档输入面：输入 Tick handler 前置一次 `ag_actions_snapshot` 解码
  （1 计费），保持原 32 分支预算（snapshot1+move26+grounded1+event3+visual1，
  fault 10 上界不变）。

## 4. 角色 model body

character3d body 可为绑定 animated3d 资源的单模型叶（原模型即角色）；
原世界 AABB 与绑定参数保持，0.8×1.0×0.8 门由 driver `_ag_char3_shape`
独立执行；生成器不做 AABB 预检、不以圆柱代原模型。恰一角色约束不变。

## 5. 原 G10 十七项与验收

原九项（textures）与原八项（animation）在新链逐项成立：独立 CPU/f64
oracle（UV 24 探针容差 2、KHR/wrap/nearest/linear、GLB 直读 slerp/层级
21 态逐顶点 <2e-5）、GPU 轮廓 ≤2、删轨/错 yaw/冻结相位负控真失败、
240Tick 同输入 pose/physics/camera/events 除 Run 身份外相同、两工程并集
8 身份 150787B、20 轮暖账、A→B 冷再生逐字节、保存/搬移/重开/坏依赖
拒绝恢复。永久门：
- `tools/verify_plan034.py --controls`（7 控制）
- driver：`node3d_presentation_flow`、`node3d_presentation_gates`、
  `node3d_presentation_oracles`、`node3d_presentation_oracles_anim`
  （GPU 门 `--test-threads=1` 单实例）。

## 6. 消费合同（AG015 / 035）

AG015（G10 双入口迁移）与 035（音画后继）以本计划实际交付为硬门：
本 Spec + 生成的十个子入口产品 + resource-meta 工具面即其消费接口；
不读开发 tip。原件字节/URI/许可闭包不可重写；Kenney 上游
3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0 与 8 份按 SHA 许可文件保持。
