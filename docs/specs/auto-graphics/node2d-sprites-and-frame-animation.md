# AutoGraphics Node 二维精灵与局部帧动画作者入口（auto-ui PLAN-022 SD-01 候选）

Status: candidate（auto-ui PLAN-022 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md 的 SD-01
候选文本，merge 阶段按收据落位。只描述实际交付并验证的能力，不声称
任何未实现的语言/运行时特性。）

## 0. 范围

`node2d-lower --profile sprites-2d` 在 020 有状态组件之上补齐**类型化
PNG 图像/具名帧表/组件局部 frame-flip 绑定与 typed 渲染实例写端口**：
普通模块工程的 `resources` 模块声明 `image2d.png("assets/x.png")`（身份 =
完整原始字节 SHA256 + byte_length + IHDR 尺寸）与 `frame2d.at(resources.x,
x, y, w, h)` 具名帧表项；组件 view 内 shape2d/character2d 携带
`image:`（直连或 Image2D 参数）+ `frame:`/`flip:`（字面量、参数或 `.field`
model 引用）即精灵叶子——静态绑定行直接落场景文档 sprite 字段（旧
autographics.scene/6 schema 原形），Field 绑定行走模板+spawn+逐 tick
`<prefix>_present(i)` 帧写。生成 `behavior.api = autographics.sprites2d/1`
工程经冻结 host 链进真实 GPU；原 G09 的 18 项验收 oracle 逐字保留名并
在新链全绿（driver node_sprites_flow 实测）。`render2d.spawn/despawn/
frame` 提供有限 spawn/despawn 的 typed 端口（token 语义沿既有对象服务：
重生代 +1、旧代写具名 fault 33）。

非范围（不冒称支持）：动态非 image shape2d、每定义多个图像元素、多实例
异图帧写、3D 纹理/骨骼/GLB、通用动画 timeline/插值/渲染时钟、
`Life.Frame`、声音/HUD、动态 solid/多角色、通用动态 Node 子树/key 重建、
HMR/ECS/CPC/ACE/AOT、新平台、预算提升。**旧入口不变**：legacy/017/019/
components-2d/queries-2d 字节冻结（verify_plan016/017/019/020/021 再生
控制在 022 生成器改动后复跑全 PASS）；旧消费者 node_components_flow 9/9
复跑绿。

## 1. 声明面（T-02 冻结拼写）

- 图像资源：`let atlas_img = image2d.png("assets/atlas.png")`（resources
  模块；文件相对工程根）。缺文件/坏签名在载入层具名拒绝
  （`image resource 'x': cannot read '<path>'` /
  `is not a readable PNG (signature/IHDR)`）；完整 PNG 准入仍属 driver
  资源服务（生成器只读 IHDR 元数据，不写第二套 decoder）。
- 帧表：`let f_amber = frame2d.at(resources.atlas_img, 32, 0, 32, 32)`，
  名字即符号、声明序即 ordinal；rect 越图像（w,h≥1、x+w≤宽、y+h≤高）
  降层具名拒绝（`frame rect [90, 0, 32, 32] exceeds image 'atlas_img'
  (96x32)`）。
- 精灵叶子：shape2d/character2d 携带 `image: resources.<img>`（或
  Image2D 参数逐实例传图，但每定义多实例必须解析到同一图像）+ `size`
  （显示 scale）+ `frame:`/`flip:`（`Lit`/参数/`.field`）。字面量
  ordinal 越帧表、字面量 flip ∉ {0,1}、image 绑非 image 资源均具名拒绝。
  每定义至多一个图像元素、至多一个动态元素。
- typed 渲染实例：`render2d.spawn("<root-view 叶子名>", x, y) -> token`
  （目标必须是根视图带 image 的静态叶子，字面 frame/flip 为文档默认）；
  `render2d.despawn(token)`；`render2d.frame(token, "<img>", ordinal,
  flip)` 语句形态展开为帧表 if-chain（每分支恰一次 `ag_sprite_slot` 写；
  越界 ordinal 走退化矩形写、由受信单元自身矩形域检查具名 fault 42）。
  同元素多 spawn 站点复用同一槽（respawn 语义）。
- 帧写合并：动态图像元素的 `<prefix>_present(i)` 每 tick 恰一次
  `ag_sprite_frame(<字符行 id>, …)`（字符宿主）或
  `ag_sprite_slot(<prefix>_handle[i], …)`（实例模板），pose→rect 由生成
  if-chain 查全局帧表数组 `{img}_fx/fy/fw/fh`；声明绑定读的 `.pose` 在
  handler 内按序计算，present 不重算（空中姿态不因 handler 后的 vy
  更新而错）。

## 2. 装配与运行时（T-02 冻结）

- 生成工程 `autographics.scene/6` + `behavior.api =
  autographics.sprites2d/1`；objects 声明序 = 文档序 = 绘制序（作者用
  声明顺序显式层序：atlas 黑色 underlay 声明在精灵叶之前，透明边界露出
  underlay——A02 实测）。文档 sprite 字段 = `{asset: "sha256:<hash>",
  frame: [x,y,w,h], flip}`，默认 frame/flip 来自声明初值（Lit/参数/model
  初值经帧表解析）。
- host 链零改动消费：`Storage::load_project`（assets/<sha>.png 逐字节
  身份核验）→ `Host::with_project` → `_ag_spr_map/_ag_spr_img/_ag_spr_def/
  _ag_spr_defh` 冻结配置 → sprites2d/1 受信单元 → 逐 tick sprite 块
  admit（新位置不与旧帧同发布）。
- 生命周期：Pause 不推进视觉、Step 恰一中性 tick、Stop 恢复文档默认
  sprite、重开新 Run 从初始化帧起（A08/C06 实测，save/reopen 同入口）。
  纹理按内容身份缓存（20 轮 fresh 后共享资源仍渲染）。

## 3. 预算与包络（V-05 实测）

- 32 credits/世界 callback 池不变：候选 atlas tick =
  held2 + move26 + grounded1 + frame1 = 30；character tick =
  pressed1 + held2 + move26 + grounded1 + frame1 = 31（实测：+1 pad 读
  32 恰过、+2 第 33 计费在预测 tick 具名 fault 10）。
- 8 运行时对象槽（objects2d SLOTS=8）：app_start 声明先序 spawn（每实例
  每动态/图像模板元素一槽）+ render2d.spawn 站点（按元素去重）合计 ≤ 8，
  降层具名拒绝（第 9 槽 fault 31 不可达）。
- 8 有状态实例包络不变（第 9 具名拒绝）；sprites-2d 场景 ≤128 行。
- 用户源不可直呼保留传输 `_ag_fail`（admission 实测拒绝）——越界帧序数
  由受信单元边界拒绝（fault 42），生成代码只写服务。

## 4. 诊断合同（V-01，module/line/symbol 具名）

缺图像文件、坏 PNG 签名/IHDR、帧 rect 越图像、字面量 ordinal 越帧表、
字面量 flip 域、image 绑非 image 资源、未知 render2d.spawn 目标、
render2d arity、每定义多图像元素/多动态元素、第 9 槽、第 9 有状态实例
——全部执行前具名拒绝（pilot node_sprites_gates 10 例 + verify_plan022
负控复放，不 panic、无静默回退）。

## 5. 原 G09 18 项映射（逐字保留，node_sprites_flow 实测全绿）

A01 atlas_main_frame_flip_left_right（新字符行 id 由场景读出）；
A02 atlas_transparent_underlay_layer_order（声明序层序）；A03
atlas_half_alpha_cpu_gpu_composite（独立 PNG 解码 + linear-light 合成）；
A04 atlas_alt_cadence_same_tick（alt 夹具组件）；A05
atlas_flip_negative_control（flip: 0 夹具）；A06
atlas_destroy_respawn_default_sprite / A07 atlas_stale_handle_rejected
（render2d spawn/despawn/frame，代次 1→2、默认帧恢复、旧代写 fault 33 +
last-good）；A08 atlas_pause_step_stop_fresh_run（save/reopen）；A09
atlas_alpha_negative_control（harness 重编码不透明 PNG、场景资产身份
同步改写、同 oracle 可区分）；C01 character_idle_run_left_right；C02
character_jump_fall_land；C03 character_pose_gpu_pixels（五姿态具名点
×独立解码×GPU 容差 3）；C04 character_reused_capture_negative_control；
C05 character_collision_and_camera（原碰撞/相机参数不动）；C06
character_pause_step_stop_fresh_run；C07 character_main_alt_same_motion
（运动同、cadence 异）；C08 character_wrong_air_run_negative_control；
C09 character_budget_32_pass_33_refused。另加 plan022/pair
_component_state_isolation（同定义双实例独立 phase/pose/flip 共享 PNG）
与 plan022/v06_damaged_copies_rejected_then_recovery_green（world/资产
一字节损坏 → 载入具名拒绝 → 精确还原 → 绿 Run）。

## 6. 消费与身份（V-06）

每产品目录：scene.agscene（behavior.source 绑 world sha/byte_length）+
world.at + identity.json（generator `plan022 sprites-2d`/api/closure
逐模块 sha/assets[sha,file,declared]/provenance 消费方）+ closure/（冷
再生唯一输入）+ assets/<sha>.png。verify_plan022 --controls：10 产品身份
链（world sha == 场景绑定 == identity；closure == 夹具；资产 sha == 固定
origin 字节）、闭包独占冷 A→B 字节等值、夹具语义变异敏感性（注释变异
被正确忽略、字面量变异改变 world sha）、6 负控具名复放、CC0 许可/来源
清单/两 PNG origin hash 固定。固定输入：fixtures/sprites2d/{assets,
licenses,manifests} 为 ag-examples G09 只读固定复制（atlas 95a08525… 258B
96×32、character 8fde96f6… 735B 160×32）。

## 7. 验收映射

- AC-01（020 基线/两接法/原额先证）：§1/§2/§3（T-02 冻结 + 18 项全绿）。
- AC-02（typed 图像/帧/端口/闭包与严格准入）：§1/§4（V-01 gates +
  verifier 负控）。
- AC-03（局部状态驱动 frame/flip/五pose/同 tick）：§1/§5（A01..A05、
  C01..C05 实测）。
- AC-04（slot/旧代/生命周期/默认帧/故障）：§2/§5（A06..A08、C06）。
- AC-05（18 项逐项可追溯/原 32/33/VM/资源/数值包络）：§3/§5（C09 +
  flow 实测记录）。
- AC-06（公共固定消费/许可闭包/A→B/坏件恢复/旧路径兼容）：§6 + v06 控
  （verify_plan022 + 旧入口字节冻结复跑）。

## 8. provenance

本候选由 auto-ui PLAN-022 T-06 建立：接口/预算/18 项映射沿 T-02 实测
冻结（Plan §9），实测面 = pilot node_sprites_gates 10 例、driver
node_sprites_flow（18 项 + pair + v06 全绿）、verify_plan022 --controls
（10 产品身份链 + 冷 A→B + 6 负控 + 许可/来源 origin hash）。016/017/
019/020/021 既有 spec 与锁保持原状，不在本文件重述。
