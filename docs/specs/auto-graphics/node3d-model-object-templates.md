# AutoGraphics 三维模型资源对象模板接线（auto-ui PLAN-043 SD-01 候选）

Status: candidate（auto-ui PLAN-043 T-01..T-06 实测交付；本稿为
docs/specs/auto-graphics/node3d-model-object-templates.md 的 SD-01
候选文本，merge 阶段按收据落位。只描述实际交付并验证的能力，不声称
任何未实现的语言/运行时特性。）

## 0. 范围

`node2d-lower --profile model-objects-3d`（PLAN-043 T-02 冻结的显式
组合入口）把 028 的 events 对象/触发/模板域与 034 的受检资源元数据
（`resource-meta` 公开导入产出）组合起来：具名静态 GLB 资源
（平色 `model3d.glb` 与 opaque 纹理 `textured3d.glb`）成为真正
spawn/move/visible/despawn 的 `object3d` 模板外观。底层 Model 实例
能力（030/035 已交付）不变，driver 运行时代码零改（实例适配路径本就
按资源键解析模型）；唯一的 driver 语义收口是 character3d 滑扫跳过
不可见行（见 §6 SD-02）。

作者面（相对 028 的唯一增量）：

- `object3d { }` 模板叶 kinds 扩展为
  `cylinder3d/player3d/mob3d/model3d`。model3d 叶必须携带具名资源
  绑定 `model: resources.<name>`；参数化绑定（`self.x`/裸参数）在
  模板无参域下具名拒绝。
- 模板行资源键按资源 ctor 决定：平色 `model3d.glb` → `sha256:<glb
  sha>`（024 公开准入面，字节身份）；opaque `textured3d.glb` →
  `closure:<id>`（034 闭包身份，外部纹理 PNG 的 sha/尺寸随闭包准入）。
  同一工程可混用两类模板；资源共享、实例身份独立。
- CLI 两段式（034 同一公开面）：pass A
  `--profile model-objects-3d --entry main.at --out DIR` 产
  `resources-request.json`；`resource_meta --entry <dir> --request …
  --out resources.json` 跑公开导入；pass B 加 `--resources
  resources.json` 产最终产品（`document_id
  plan043-model-objects-3d-<root>`）。

拒绝域（执行前具名拒绝，module/line/symbol 定位）：

- `animated3d.glb` 声明在 model-objects-3d **整域**拒绝（v1 模板是
  静态外观；不删动画轨道伪装静态）；受检元数据揭示动画内容时模板叶
  二次拒绝。
- 模板形状约束沿 028：恰一叶/禁 params/model/msg/on/character3d/
  camera3d/solid/禁在任何视图实例化/叶位置可见性静态。
- 未知资源名（`resources.nope`）、textured 元数据缺 closure（陈旧
  resources.json）具名拒绝。
- 旧档面不变：`components-3d-events` 档仍拒绝 `textured3d.glb`/
  `animated3d.glb` 声明（028 拒绝域不拓宽）；034 presentation 档
  "元数据在案必须有 closure"语义不变（model-objects 入口对受检元数据
  按 kind 过滤：model3d 条目滤出回落 sha 键，仅本入口内生效）。

## 1. 执行证据（只述实测）

**降层门**：pilot `tests/node3d_model_objects_gates.rs` 4/4——g12
actor-free 链两次再生长字节一致、模板行 kind/resource/parked/typed
双槽 spawn 端口、负控：未知资源/solid/参数绑定/model 状态块/textured
档域/animated 域/模板形状。

**消费验证器**：pilot `tools/verify_plan043.py --controls` 全绿——
identity（generator/api/document_id/sha/closure/资源键/资产清单）、
冷再生 A→resource-meta→B 字节一致（平色与 opaque 两产品）、突变敏感
（场景行语义编辑移动 scene 字节、行为源稳定）、旧档冻结（028
g11-main 从闭包再生成字节一致）。

**driver 全链（`tests/node3d_model_objects_flow.rs` 8/8，GPU 单实例
--test-threads=1）**：

- g12-model（平色 two-parts，MIT 自撰测试几何）：Storage 准入 →
  Host/Triggers3V1 → Start 双实例（同模板 id、gen1、各自句柄）→
  GPU 双 part 像素 oracle（绿三角判别式；静态圆柱默认红不可改色）
  → move/hide/显式触发 pair0（事件消费滞后作者一 tick）→ despawn
  后绿像素=0 且红色回落圆柱基线（真销毁/真隐藏变异 oracle）→ 同槽
  gen2 资源共享再暖 → Stop/重开新 Run 再渲染。
- g13-model-opaque（Kenney coin CC0，g10-textures 在库许可）：闭包
  键模板 + 纹理 PNG 闭包准入 → 亮金像素 oracle（r224-255/g192-224/
  b64-96；圆柱橙/红不在类）→ 全生命周期同 g12。
- 准入负控：剥模板资源键 → "model resource reference"；GLB 单字节
  篡改 → 具名拒绝。
- 预算/身份负控：33rd credit→fault 10（角色 op 计费不占 object 写
  上限）、33rd lifecycle write→fault 37（写上限先于计费）、slot 8→
  fault 31、despawn 后旧句柄→fault 33。
- 非 solid 轨迹对照（g14-model-actor）：隐藏模板 parked 在行走车道 +
  活实例 + 销毁后三阶段，逐 tick 角色身体位置与"模板停远处"对照
  **逐位相等**且走过无阻挡（043 语义收口后）。
- 资源并集/暖账（V-S04）：20 轮 start/tick/stop 后 mesh_count 与
  decode_attempts 平坦（注册表共享 GLB 身份，暖 decode/upload=0）；
  并集=单件 1528B，raw8 身份/单件 8MiB/总 32MiB/CPU/GPU 各 64MiB
  原额内；20 轮后文档逐字不变。
- 512 mesh denominator：63/65-mesh 合成模板 × 8 槽边界两侧——521
  在 Host 首帧 prepare 具名拒绝（"resource limit: potential mesh
  instances 512"），505 正常启动；invisible parked 模板行计入
  denominator（隐藏不减额）。
- SaveAs/搬迁/坏件恢复：saveas 新根冷打开真 Run 绿；新根 GLB 单字节
  损坏具名拒绝→修复同入口绿；g13 闭包 PNG 损坏→修复→绿。

## 2. 不变量

- 实例身份/生命周期/预算沿 001/028 冻结域：handle=
  run*524288+slot*65536+generation；8 槽/32 写/共享 32 credit；
  物理写 32 先于计费；隐藏仍活；销毁回收 slot、重用必须新代；拒绝保
  last-good、不污染 generation 高水；实例永不进 Document/空间世界/
  solid/角色相机状态（adapter id 0 非持久）。
- 模板与活实例均非 solid、非 character/camera 目标；触发仍由显式
  shape/radius/句柄身份驱动，视觉模型不决定碰撞。
- 资源域沿 024/034：原始字节 sha 身份、公开独立准入、闭包跨
  034/035 完整消费；生成器不解析 GLB 结构（动画/纹理身份只来自
  resource-meta 受检元数据）。
- 旧档字节/费用不跟升：028/030/034/035 在库产品与 gates 全绿
  （028 gates 7/7、旧档冻结再生成字节一致、runtime/events/basic/
  bindings/audio-hud flows 与 character3d api/host/samples 回归绿）。

## 3. 非目标（本期不做）

运行实例动画 pose/骨骼、动态 mesh/material/TRS/纹理替换、动态
Node 树/跨场景、完整资源参数多态、新物理/触发算法、游戏内容迁移。
模板首版仍是一个静态模型叶、无参数/model/msg/on；业务属独立有状态
owner。
