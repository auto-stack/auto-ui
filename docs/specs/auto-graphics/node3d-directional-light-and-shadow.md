# AutoGraphics 正式 Node 方向光、硬阴影与公共宿主端口（auto-ui PLAN-080 SD-01 候选）

Status: candidate（auto-ui PLAN-080 T-02/T-03 中立实测 + T-04 作者整链
实测交付；979/981 A 切片已落主。本稿为 docs/specs/auto-graphics/
node3d-directional-light-and-shadow.md 的 SD-01 候选文本，merge 阶段按收据
落位。只描述实际交付并验证的能力，不声称任何未实现的 979/981 接口或
完整 three.js 覆盖。）

## 0. 范围

059 研究已证底层 MeshPipeline 单方向光/硬阴影可用、作者链为 bounded-go
（研究静态描述＋Auto 动态标量绑定）。PLAN-080 将研究桥提升为**正式公共
能力**：冻结中立 Light 端口（typed 数值准入）、数值域、GPU 策略与资源
计量，并在 AV 开发/新 A2R 双端通过 979 共同前门及 981 领域接口消费同一
中立 MeshPipeline。本 spec 记录已冻结的中立端口与实测证据；作者整链
（Node 声明 → 979 → 981 AV/A2R → 真实 GPU）待 A 切片落主后并入。

## 1. 中立 Light 端口（T-02 冻结，已交）

`agp::render3d::LightParams` 是唯一合法的 typed 入口：从作者参数构造
`LightDef`，全部数值在 GPU 上传前受检。字段与支持域：

| 字段 | 冻结支持域 | 拒绝分类 |
|---|---|---|
| `dir_to_light` | 有限、非零（长度平方有限且 ≥ε²），归一化后有限 | Invalid |
| `color` | 每通道 ∈[0,1] | Invalid（非有限）/ Unsupported（越界） |
| `energy` | ∈[0,1] | Invalid（非有限）/ Unsupported（越界） |
| `center` | 有限 | Invalid |
| `radius` | ∈(0,32] world units | Unsupported |
| `near`/`far` | 0≤near<far≤160 | Unsupported |
| `bias` | ∈[0,.01] | Unsupported |
| 生成 `shadow_vp` | 矩阵全部元素有限（look_at 正交，方向平行常用 up 时用确定替代 up，不产生 NaN） | Invalid |

错误分类独立于语言语法错误：NaN/坏类型 → `LightError::Invalid`；
第二灯/超域 → `LightError::Unsupported`。拒绝时零 GPU 写入，调用方保持
last-good。

## 2. GPU 策略（已冻结）

单方向光、单张 2048² Depth32Float 硬阴影图、opaque 平色网格、环境项固定
0.25、现行 culling/bias 与逆转置法线语义保留；独立网格非均匀缩放不误用
model 矩阵变换法线。shadow 开关与遮挡物位置由 Auto 状态改变，不隐式使用
`sample_light` 常量覆盖作者输入；`sample_light` 与 M1 源/锁/价格保持原样，
不隐式升级。

## 3. 资源与寿命（已冻结）

阴影名义 16MiB（2048²×4B），不等于实测驻留或已证释放。scene/generation
持已发布描述；运行调用 pin 旧代；device owner 持 shadow texture/view 与
uniform；替换候选失败不动当前代，Stop 最后 owner 释放；新 epoch 不得重用
旧句柄。实例/uniform 分账沿用 059 微场景：双实例每帧 352B/3 写
（64 camera + 160 light + 128 instance）；mesh 上传按 identity 去重。
旧 32credits 不等同 ACE fuel，不放大旧 128 行/8 实例/32 费用，未证 FPS
不承诺。

## 4. 实测证据（T-02/T-03）

- prototype `render3d::light` 单元 5/5：域边界正控、非法近邻、up 回退、
  斜投影候选仅 x/z 平面。
- prototype `tests/node3d_lighting` 2/2 真 GPU：CPU Lambert/阴影 oracle
  （sRGB 每通道容差 3/255）、拒绝保 last-good。
- driver `tests/node3d_lighting_flow` 4/4 真 GPU：V-02 数值/法线、
  V-03 阴影 on/off/移动正反控、V-05 准入拒绝、V-06 20 轮/resize/零
  尺寸/device epoch。
- 旧触面：全部 `sample_light()` 调用点仍在 prototype 内部 host 用例，
  `LightDef` 结构未变；prototype lib 35/35、textured_models 4/4 绿。

## 5. 受影响命名 case 并集（T-06 冻结，中立部分）

| Case / 入口 | 范围 | 状态 |
|---|---|---|
| `experimental/autographics-prototype/tests/node3d_lighting.rs`（2 cases） | 中立端口→GPU 像素 oracle、拒绝保 last-good | 绿（T-02） |
| `experimental/autographics-auto-driver/tests/node3d_lighting_flow.rs`（4 cases） | V-02/V-03/V-05/V-06 中立验证 | 绿（T-03） |
| prototype `src/render3d/light.rs` 单测（5） | 域边界/非法近邻/up 回退 | 绿（T-02） |
| driver `tests/node3d_runtime_flow.rs` vs06 | 坏 world 拒绝→修复→恢复、20 轮 Run/Stop | 既有绿，T-05 复用 |
| prototype `tests/textured_models.rs`、lib 35 项 | 旧触面回归 | 绿 |
| 主 checkout `tests/surface_review_r2.rs` | 预存编译缺陷（缺 `hud_overlay` 字段，master HEAD 同败，非本计划引入） | 不修，merge 不扩无关门 |

冷消费：AG030 只消费本 spec 冻结的公共 typed 端口（`LightParams`/`LightDef`）
与数值域，不复制引擎实现；其 T-03 配方入口为
`examples/m2/directional-light/` + `tools/verify_directional_light.py`（AG030
创建），须 UI080 落主与 979/981 A 接口齐后实施。

## 6. G3-C01..06 身份与限制（逐项，T-06 登记；作者整链 T-04 已并入）

| ID | 中立已验证（T-02/T-03） | 作者整链（T-04，979/981 A 切片落主后） |
|---|---|---|
| G3-C01 | 单灯 typed 端口、第二灯 GPU 前拒绝、last-good | AGE 前门消费普通 Node（model 灯参数）；Node 声明/实例事件 2:1 |
| G3-C02 | 方向有限非零/归一化/up 回退、energy∈[0,1]、color∈[0,1]、非均匀缩放法线 | 作者值经 AV/A2R 双端生效（int 缩放 0..10→0.0..1.0，F32 归 981 B 批次） |
| G3-C03 | 单张 2048² Depth32Float 硬阴影、开关/移动正反控 | 作者源改 shadow 开关/遮挡位置 |
| G3-C04 | opaque 参与投影/受影、逆转置法线、culling/bias 保留 | 同源资产双端一致 |
| G3-C05 | —（需 979/981） | 同源 AV/A2R 真实 rustc/GPU、改 Auto 常量双端变化（V-04 已验） |
| G3-C06 | 准入/失败保旧、resize/零尺寸、device epoch、20 轮 owner | 作者 Run/Stop 与最后 owner 释放 |

## 7. 作者整链（T-04 已交付）

同源 Node 声明 → 979 共同前门 → 981 SOURCE/SESSION/RUST/LANGUAGE A 切片 →
AV 开发/新 A2R 真实 rustc 编译/真实 GPU 的作者整链已验（V-04）：
driver `node3d_lighting_flow::v04_age_author_chain_drives_gpu_light` —— 普通
Node 源（model energy/shadow）经 AGE 前门编译 → AV 观察 `states=[[5,0]]` →
冻结 LightParams admit → 真实 GPU 像素 lit=0.75；`age_tick(1)` 同源修改 →
`states=[[6,0]]` → 像素变 0.85。A2R 腿由 981 `plan981_age_package_a2r` 2/2
（同一 AGE 前门源 → CheckedModule → A2R provider → 真实 rustc）。能力边界：
AGE av.owned/1 当前不持有 F32（model 字段仅 int/i32/bool/str），灯参数用
int 缩放表达，float model 字段归 981 B 批次。
