# AutoGraphics 正式 Node 方向光、硬阴影与公共宿主端口（auto-ui PLAN-080 SD-01 候选）

Status: candidate（auto-ui PLAN-080 T-02/T-03 中立实测交付；AV/A2R 作者整链
T-04 等 979/981 A 切片落主后并入。本稿为 docs/specs/auto-graphics/
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

## 5. Pending（T-04 并入）

同源 Node 声明 → 979 共同前门 → 981 SOURCE/SESSION/RUST/LANGUAGE A 切片 →
AV 开发/新 A2R 真实 rustc 编译/真实 GPU 的作者整链；V-01（声明/span/实例
事件 2:1、第二灯 GPU 前拒绝）、V-04（同源 AV/A2R 双端结果随作者修改变化）。
本稿对应部分待 979/981 A 切片落主后合并定稿，未交项保持 pending。
