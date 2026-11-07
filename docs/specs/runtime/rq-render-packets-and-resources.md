# RenderQueue 无损原语、字形与资源重放（PLAN-062）

本页是 RQ **v3 载荷与原语、字形重放、资源身份/预算、能力准入与背压**的
canonical 规范。来源：PLAN-062（r3）的实施与实测。与
[保真合同](rq-fidelity-contract.md) 的关系：那页记录 v2 现状与冻结能力 ID，
本页记录 062 新增的 v3 增量、实测保真边界与具名遗留。**未在本页写明的行为
不视为已实现。**

## 1. 载荷版本与兼容窗口

| 载荷 | tag | 说明 |
|---|---|---|
| DrawList v1 | 1 | legacy 只读 |
| DisplayList v2 | 2 | 七原语（Quad/TextStyled/Scissor(/Pop)/Transform(/Pop)/Image） |
| DisplayList v3 | 3 | v2 超集：新增 `Path`、`GlyphRun` |

- **编码规则**：一帧只用一种 tag。含 `Path`/`GlyphRun` → tag 3；否则仍编
  tag 2（旧 Host 的双读窗口不被破坏）。旧 Host 对 tag 3 明确 `UnknownTag(3)`
  拒绝并**保旧帧**（不呈现半帧）。
- 信封 `PROTOCOL_VERSION` 不变（v3 只是载荷演进）。

## 2. v3 原语

### 2.1 `Path`（mesh/canvas 几何）

- 段序列 `PathSeg ∈ {MoveTo, LineTo, QuadTo, CubicTo, Close}`，坐标是**已烘焙
  变换的设备像素**（iced geometry 层在记录时已把变换乘进坐标）。
- 同 op 可带 `fill`（色/线性渐变 angle+stops）、`even_odd` 填充规则与
  `stroke{width,color,cap,join}`；`Primitive::Fill/Stroke` 各自独立记录。
- 记录端不得再丢弃 mesh（062 前 `lower_layers_v2` 只记观测）。
- 具名降级：`tiny_skia` 的非纯色 paint（渐变 shader 无可读回访问器）不承载，
  产出观测行，不静默丢图；`LineJoin::MiterClip` 按 Miter 承载。

### 2.2 `GlyphRun`（文字，Host 不重排）

- 字段：`font_hash`（字体字节 blake3 前 16 字节，**禁路径当身份**）、
  `font_size`（已乘 viewport scale）、`weight`、`flags`、`color`、
  `line_offset`（`round(line_y·scale)`，与 tiny-skia 绘制同律）、
  `glyphs: [{glyph_id, x, y}]`。
- `x`/`y` 是送入 cosmic-text `CacheKey::new` 的**物理坐标**（含亚像素）；
  Host 用同一 `(font_id, glyph_id, size, x, y, weight, flags)` 取回与生产端
  同一张字形位图，**不得**按字符串重新整形/布局。
- 字体解析：内容哈希 → fontdb ID（显式登记；未登记时对本地 fontdb 做一次性
  扫描以支持系统回退字面）。解析失败 → 具名告警且该 run 不呈现，不得静默换字体。
- 生产端按字面变化拆 run（中英混排）；无内容身份时整行退回 `TextStyled` 并
  留观测（可观测降级）。
- **当前开关**：`HeadlessSurface::set_glyph_runs`（默认关）。原因见 §5 实测：
  字形位图经画布图像管线合成，边缘覆盖率与 direct 文本管线不同。

### 2.3 图像与阴影

- 图像：`Image{rect, src, fit}`，`src` 指向内容寻址的 `bitmap://` 资源。
  `Rgba` 直取、`Bytes` 解码、SVG **生产端按设备像素栅格化**（resvg、
  `Svg::color` 以 alpha 掩膜着色、预乘→straight 归一，单边 4096 上限具名拒绝）。
- 图像 `opacity` 烘焙进资源字节；`rotation` 以 `Transform` 包络过线。
- **阴影**：带 shadow 的 Quad 由生产端按 `iced_wgpu/solid.wgsl` 同式求值
  （`rounded_box_sdf` 双倍中心坐标、半径取 `min(r, 短边/2)`、
  `1 - smoothstep(-blur, blur, max(sdf,0))`），生成贴片资源，帧内先发
  `Image(贴片)` 再发 `shadow: None` 的 Quad。Host 不再以垫层近似。

## 3. 资源身份、驻留与预算

- 身份 = 内容键 `blake3(w,h,bytes)`；局部 id = `rq-img-<seq>-<hash8>`，序号单调
  （等价 generation 语义）。内容未变的重复帧**零 payload 重传**，帧只引用同一
  `bitmap://` src；Host 侧按 src 复用已驻留纹理。
- 驻留上限 256 MiB/App（最早键驱逐）；驱逐后的重引用会重传一次，计数可解释。
- **上传预算**：8 MiB/1s 滑动窗口（per-App）。超预算帧**整帧回滚**（撤销本帧
  登记的资源键、复原 uploads/字节/驻留计数），端点返回
  `ProtocolError::Backpressure{resource:"upload"}` 并归还槽位；内容不丢、不得
  半登记、不得把已产出的上传半发出去。
- SVG 栅格按 `(内容, 目标尺寸)` 缓存（`raster_hits` 可观测），稳定帧不重复跑
  resvg。

## 4. 能力准入与背压

- 能力 ID 与 required/optional 语义见[保真合同 §7](rq-fidelity-contract.md)；
  本页新增的必需能力：`fidelity/pathfill/1`（Path）、`fidelity/glyphrun/1`
  （GlyphRun）。
- 线词汇：`required:<cap>` / `optional:<cap>`；裸串按 optional 解释（旧 tail
  字节兼容）。声明可逆（`CapabilitySet::to_wire` round-trip）。
- **准入**：Host 合成前校验帧的必需能力，缺一即具名拒绝
  （`AdmissionError::UnknownRequiredCapability`），**不写槽**（保旧帧）并归还
  传输槽。v2 词汇在收窄能力集下仍受。
- **槽 = 信用**：帧/位图槽按"空闲池优先 → 从未用过的槽"分配；在途（未 ACK）
  槽不得覆盖；无信用即 `Backpressure`，调用方保留待发状态重试。ACK 只归还传输
  槽并清在途标记，**不代表 GPU 完成**（GPU 寿命由 Host fence 管理）。

## 5. 实测保真边界（2026-11-07，offscreen 代理终点）

六格矩阵（`rq_cells`）：`rust/direct` 可跑、`rust/standalone-rq` 可跑（inproc
近似），`rust/desktop-rq`、`vm/direct`、`vm/standalone-rq`、`vm/desktop-rq`
为**不可运行格**（owner UI063/OS063，fixture 不冒充真实 producer）。同 source/
主题/1280×800@scale2，逐例像素差（阈值 2/255）：

| 例 | v3 原语 | 相对 direct 差（矩阵默认 = glyph run 开） | 字符串面（glyph run 关） |
|---|---|---|---|
| path_mesh | Path×4（12 段） | 0.095% | 0.095% |
| svg_icon | Image（16×16 资源） | 0.100% | 0.100% |
| text_cjk | GlyphRun×2（15 glyph）/ TextStyled | **0.80%**（具名遗留） | 0.094% |
| gradient_border_shadow | Quad + 阴影贴片 | 1.37% | 0.0625% |
| shadow_only | Image（阴影贴片） | **0.74%**（具名遗留） | — |
| scroll_clip | GlyphRun / TextStyled | 0.33% | 0.000% |
| progress_bar | Quad×2 | 0.006% | 0.006% |

- 结论清晰：**Path / SVG / Quad 类原语已经对齐（≤0.1%）**；文字与阴影在 glyph run /
  贴片路径下**尚未 exact**（0.33–1.37%），退回字符串面时文字重新变好（0.094%）。
- 两者的残差同源：贴片/字形位图经画布图像管线合成，与 direct 的文本/quad shader
  合成路径不同（位置与几何已证相等，例如 glyph `x` 与 cosmic-text `physical()`
  逐字相等；阴影贴片与 shader 同式求值）。
- 因此 v3 glyph run **默认关闭**；打开它需要先解决合成对齐（Host 侧同源管线或
  经 review 同意的具名窄容差）。字符串面保持 0.094% 且 Host 用同一字体栈重排
  （可观测降级）。
- 性能（p50，standalone-RQ 例）：produce 0.09–0.24ms（SVG 例外 20–32ms，根因是
  记录路径每帧新建 iced renderer 导致 iced 缓存冷启）、encode 0.005–0.008ms、
  decode 0.004–0.008ms、replay 3.3–5.9ms，direct 稳态帧 p50 3.3–6.9ms。
- M 指标：M-04/M-08 measured；M-01/M-03/M-05 为 proxy（present 与 GPU 驻留未测）；
  M-02/M-06/M-07 unmeasured（真实 deadline、冷热启动、1/4/8 App 归 UI063/OS063）。

## 6. 具名遗留（不得当作已通过）

1. glyph run 边缘对齐 0.80%（合成路径；见 §5）。
2. 阴影贴片合成残差 0.74%（同上）。
3. SVG 记录路径每帧新建 iced renderer → produce 20–32ms（T-02 实现债）。
4. per-App CPU 预算/公平调度未实现（真实规模归 UI063/OS063）。
5. 媒体资源族（Video/RemoteFrame）与 wire 上的显式 generation 字段未进本增量。
6. v2↔v3 双读只做了 tag 语义与拒绝路径，未做两进程实机配对；慢 Host/乱序资源/
   旧 ACK 负控未跑。
