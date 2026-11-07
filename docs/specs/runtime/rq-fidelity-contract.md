# RenderQueue 保真合同（wire、字体、资源与呈现）

本页是 AutoUI RenderQueue（RQ）**当前行为**与**已冻结合同**的 canonical 规范。
来源：PLAN-061（r3）的探针与决策记录；实现级缺口仍按本页“当前缺口”与
“具名不可达”列明，不因本页存在而视为已实现。

## 1. 适用范围与所有权

- App（Rust 编译或 VM 执行）拥有业务状态、组件、布局、命中与事件处理，
  并产出 CPU 侧记录；RQ Host 拥有 GPU/device、目标表面与最终栅格化/合成/呈现。
- App **不得**使用隐藏窗口、自有 GPU 或整帧 CPU 栅格化后传截图冒充 RQ；
  源图片/视频解码帧属资源，不等于 App 自绘整帧。
- 三种 Host（Standalone RQHost、Desktop RQHost、Linux compositor Host）共用
  本页的协议、资源寿命与重放核心；窗口系统适配分别实现。

## 2. 线协议与版本（当前）

| 层 | 当前值 | 兼容规则 |
|---|---|---|
| 信封 | `APDL` magic + `u16 PROTOCOL_VERSION=1` + 5 通道（Handshake/Frame/Input/Control/Observe）+ 长度 | 版本不符 → `UnsupportedVersion`；未知通道/消息 tag/尾部多余字节 → 明确错误（`UnknownChannel`/`UnknownTag`/`TrailingBytes`） |
| 帧载荷 | tag 1=`DrawList` v1（legacy 只读）；tag 2=`DisplayList` v2（当前 remote 产线）；tag 3=保留（见 §7） | 未知 tag → 拒绝且**保旧帧**（不呈现半帧） |
| 帧模式 | `Commands` / `Pixels` | 模式选择必须显式；不静默降级 |
| 能力声明 | Hello/Welcome 尾部 `theme_caps` 等字符串**声明式** | 空 tail 与旧端字节等价；未知 cap 当前不阻塞解码 |

`DisplayList` v2 的原语词汇（当前仅七种）：
`Quad`（fill=纯色/线性渐变、radius[4]、border、shadow）、`TextStyled`、
`Scissor`/`ScissorPop`、`Transform`/`TransformPop`（2×3 矩阵）、`Image`。

握手：`Hello`（app→host：身份/尺寸/字体/表面/能力）→ `Welcome`（host→app：
app id/wid/surface/rect/frame_mode）→ `Ready`。

## 3. 当前行为与已知缺口（不得当作通过）

当前 v2 产线由 iced 的 tiny-skia Secondary 记录层降格而来，已实测的缺口：

- **mesh / canvas 几何**：记录层含 `Primitive::Fill/Stroke` 路径（变换已烘焙进
  坐标），但 v2 wire 无路径原语 → 当前被省略（仅记观测）。
- **SVG / vector 图像**：记录层以 `Image::Vector` 出现，v2 省略；需生产端按
  设备 scale 栅格化为位图资源（同设备 scale 下可逐字节一致）。
- **文本**：记录层对 live paragraph/editor 含 glyph 几何（字体整形后），但
  v2 `TextStyled` 只携字符串/字号/字重/斜体/颜色，Host 必须用自身字体栈重排；
  冷帧可能尚未整形（`buffer_lines=0`）→ 首帧缺字风险。`Raw`（编辑器组合态）
  当前直接省略。
- **完整 2D 变换**：canvas 几何的旋转/缩放会烘焙进路径（可达）；但 iced
  `Transformation` 仅 scale+translate，文本旋转/斜切不可经该类型表达（几何
  文本会退化为 outline path，可保真但失去文本语义）。
- **阴影 blur**：`Quad.shadow(color/offset/blur)` 参数在 wire 中，但重放端
  无 blur 原语（现为单层 scrim 近似）；精确复现需 Host 具备 blur/SDF 能力。
- **图像呈现参数**：`Image` 不携 rotation/opacity/filter，当前丢弃。
- **资源复用**：当前稳定重复帧仍每帧上传位图（无内容身份/缓存）——违反 P-02。
- **剪贴板**：headless/记录路径以 `clipboard::Null` 驱动，剪贴板族不可用。
- **呈现反馈**：无独立 present 事件；评测终点为代理（draw/读回），不是物理上屏。

## 4. 字体与文本（合同）

- **字体身份** = SHA-256 内容哈希 + face index + 字节（FontBlob 通道）；
  绝对路径不得作为唯一身份。内容变必须新 id。
- 文本承载优先级：`GlyphRun`（glyph id、x/y、字体内容 id、size、color、
  weight/italic）> 字符串 `TextStyled`（降级：Host 重排，必须可观测）。
- 录制期必须强制整形（对 Paragraph/Editor/Raw），未整形即降级并诊断，
  不得静默出空文本。
- 缺字体/glyph：Host 必须明确失败/上报并保留旧内容，不得静默替换字体。
- 旋转文本若需保真，走 outline path（失去可编辑语义），该限制必须写入
  支持边界。

## 5. 资源与帧事务（合同）

- 资源身份：`(session_epoch, content_hash_or_app_key, kind)`，
  `kind ∈ {bitmap, font, svg, mesh, media}`；**generation 单调递增**。
- Host 只接受更大 generation 的更新；旧 generation 明确拒绝并保旧资源。
- `create/update/release`，owner=App，引用归零释放；容量以字节+计数计。
- **ACK 与 GPU fence 分离**：`FrameAck` 只归还传输槽（Host 已读走载荷）；
  GPU fence 由 Host 在资源不再被 GPU 使用时释放。App 不得据 ACK 判 GPU 完成。
- **原子帧**：整帧 decode + 结构校验（长度/乘积溢出/非有限数/栈配对/索引/ID）
  通过后才 compose；失败**保旧帧**并诊断，不呈现半帧。
- 合成采用双缓冲（写新槽 → front 指向新槽 → 释放旧槽）；崩溃/断连保留最后
  完整帧。
- 可复用片段：当前合同为“全帧恢复 + 资源驻留复用”；不做未经实测的全局增量。

## 6. 呈现与调度（P-01～P-04 合同要求）

- **P-01 异步批量**：每帧一条帧消息（允许协议内分块），禁止逐图元请求/应答；
  ACK 管传输槽，fence 管 GPU 资源寿命；容量耗尽须有界背压，
  **不得覆盖未 ACK 的槽**；显式查询/握手可有 reply，但不得串入绘制热路径。
- **P-02 资源复用**：按内容身份注册/更新；内容未变的稳定帧不得重复上传
  payload 或重建 GPU 资源；失效/驱逐/新 session 的重传必须可解释；
  App 已完成的文字布局不得在 Host 重排。
- **P-03 帧调度**：readiness（输入/资源完成/present 反馈）驱动为主；
  定时器仅用于 deadline 或具名平台 fallback；每 surface 在途帧/字节有上限；
  可合并过期画面，但不得丢输入、资源依赖或业务副作用。
- **P-04 多 App 隔离**：按 session/surface 计量并限制排队帧、上传字节、驻留
  资源与 CPU 批处理；Host 调度不得让重负载独占全局锁/事件循环/GPU 提交队列；
  重任务须在能力/预算内拆批、限流或显式拒绝；不得承诺任意 GPU 命令可抢占。

预算与测量口径（方法、单位、样本规则、失败处理）以 PLAN-061 冻结的
M-01～M-08 定义为准；无预算或必需测量缺席不得签性能 PASS。

## 7. 冻结能力与迁移（tag 3 / v3 预留）

能力 ID（Hello/Welcome 双向声明 required/optional）：
`fidelity/pathfill/1`、`fidelity/glyphrun/1`、`fidelity/svg-raster/1`、
`fidelity/image-params/1`、`fidelity/blur-sdf/1`、`resource/generation/1`、
`pipeline/backpressure/1`、`ack/fence-split/1`。

- required 未知 → **入帧前拒绝**并具名诊断；optional 未知 → 允许降级但必须
  产出可观测降级事件。
- 旧端（空 tail）按 v2 处理，不收 v3 帧；迁移为 v2↔v3 双读窗口，
  v1→v3 先经 v2；退役旧 tag 需具名计划与负控。
- 信封版本仅在五通道/信封布局不兼容变化时升级。

## 8. 三 Host 与引擎边界

- Desktop RQHost 不得以桌面外独立窗顶替嵌载；surface/placement/可见性/焦点/
  坐标变换由桌面侧策略负责。
- Linux compositor Host（UDS/共享映射/claim/Wayland IME）归 OS062/OS063；
  OS062 profile 的 8 MiB 控制帧 / 64 MiB 段门是输入，不在本页推广为全平台上限。
- ACE（执行引擎）与 AGE（图形引擎）只通过版本化 wire 接入，不暴露 Rust
  ABI/对象；AGE 3D 以新图形包/资源能力扩展，当前不虚构 production wire。

## 9. 具名不可达（合同边界，禁止静默降级）

1. custom GPU shader primitive（CPU 记录器无通道；需 App GPU，违反所有权）。
2. SVG 的 Skia 级 filter 子集（resvg 覆盖外）。
3. 要求保留可编辑文本语义的旋转文本（outline path 可保真但失语义）。
4. 物理上屏/真实 present 时序（归平台/OS063；评测终点须标代理）。
