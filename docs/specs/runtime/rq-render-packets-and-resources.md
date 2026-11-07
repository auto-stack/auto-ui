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
- **槽载荷分派**（`FrameReadyShared` + shm 槽）：槽内首字节 = 载荷种类 tag。
  `1` = DrawList v1；`2`/`3` = **DisplayList**（v2 词汇 / v3 超集），二者必须
  **同臂**进入 v2 合成面（`compose_v2` 负责能力准入与保旧帧）。把 tag 3 当 v1
  解码会**静默丢整帧**——`shm::is_display_list_payload` 是唯一判定点，三处宿主
  消费（`rqhost`、`host::ProtocolHost`、桌面 `session`）共用。
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
- **媒体静态帧**（2026-11-08 裁定）：视频/远端媒体的**一帧**（海报/首帧/暂停帧）
  由 `ui::media_poster` 供给，`render_video` 命中即用普通 `image` 元素出图——
  于是它就是普通图片资源（内容身份、去重、上传预算、驻留全部同律）。连续播放
  帧不在本页能力内（见 §6）。
- **阴影**：带 shadow 的 Quad 由生产端按 `iced_wgpu/solid.wgsl` 同式求值
  （`rounded_box_sdf` 双倍中心坐标、半径取 `min(r, 短边/2)`、
  `1 - smoothstep(-blur, blur, max(sdf,0))`），生成贴片资源，帧内先发
  `Image(贴片)` 再发 `shadow: None` 的 Quad。Host 不再以垫层近似。
  **实测边界（不得写成改进）**：贴片路径相对原单层 scrim 在
  `gradient_border_shadow` 上是退步（0.0625% → 1.3740%，同 harness/阈值），
  因为贴片经画布图像管线合成、且阴影与 quad 分成两层而非 shader 内一次混合。
  贴片的价值是几何正确（同式 SDF）而非像素更近；在合成对齐解决前，该能力按
  **具名遗留**处理，不得作为保真通过证据。

## 3. 资源身份、驻留与预算

- 身份 = 内容键 `blake3(w,h,bytes)`；局部 id = `rq-img-<seq>-<hash8>`，序号单调
  （等价 generation 语义）。内容未变的重复帧**零 payload 重传**，帧只引用同一
  `bitmap://` src；Host 侧按 src 复用已驻留纹理。
- **线上 generation（AC-03，已交付）**：宿主在 `HostCaps` 里声明
  `resource/generation/1` 时，producer 把位图上传换成 `BitmapReadyGen`
  （tag 13，字段序同 tag 10 + 尾追 `generation: u64`，会话内单调）。宿主按
  `id` 记最后已应用序号：**严格更小的序号（迟到/乱序）与相等序号（重复投递）
  一律丢弃并保旧内容**，但仍回 `BitmapAck` 归还槽（槽不得泄漏）。未声明该能力
  的旧宿主永远收不到 tag 13（字节流不变），保持“同 id 重上传 = 即时翻新”语义。
  三处宿主消费（`rqhost`、`host::ProtocolHost`、桌面 `session`）共用同一守卫；
  `HostCaps` 下行在三个宿主都已接线（此前只有 `host::ProtocolHost` 有，独立
  RQ 宿主与桌面 broker 的协商在线上是死的）。
- 驻留上限 256 MiB/App（最早键驱逐）；驱逐后的重引用会重传一次，计数可解释。
- **释放（AC-03，已交付）**：producer 侧资源的**最后 owner 归零**（驻留驱逐）时，
  驱逐记录进入释放队列，泵在排水点经 `ResourceRelease`（tag 14）批量上报
  `bitmap://<id>`。仅在宿主声明 `resource/release/1` 时发送——旧宿主永远收不到该
  tag（字节流不变，不会解流失败）。
  宿主侧做**帧引用记账**（`host::BitmapRefs`）：每次合成 v2 帧刷新“前台帧引用的
  src 集合”，收到释放时**前台仍引用就挂起**，等下一帧不再引用才真正驱逐并清缓存
  键——所以“立即释放”不会把仍在屏上的图丢掉。未被引用的释放立即兑现；producer
  重发同一 src 不重复驱逐。三处宿主消费（`rqhost`、`host::ProtocolHost`、桌面
  `session`）共用同一记账件。
  仍有具名边界：释放是**单向通知**（无 ACK/无引用计数交换），Host 自身上限仍
  兜底；Host 侧句柄缓存上限 512 条，但 **`bitmap://` 条目不参与本地淘汰**
  （producer 拥有、丢失不可自愈）；超限的其它来源（文件/http/资产）按插入序逐出。
- **上传预算**：8 MiB/1s 滑动窗口（per-App）。超预算帧**整帧回滚**（撤销本帧
  登记的资源键、复原 uploads/字节/驻留计数），端点返回
  `ProtocolError::Backpressure{resource:"upload"}` 并归还槽位；内容不丢、不得
  半登记、不得把已产出的上传半发出去。
- SVG 栅格按 `(内容, 目标尺寸)` 缓存（`raster_hits` 可观测），稳定帧不重复跑
  resvg。

## 4. 能力准入与背压

- 能力 ID 与 required/optional 语义见[保真合同 §7](rq-fidelity-contract.md)；
  本页新增的必需能力：`fidelity/pathfill/1`（Path）、`fidelity/glyphrun/1`
  （GlyphRun）；新增的**可选面**（协商后启用、旧端不产不出）：`resource/generation/1`
  （位图显式 generation，tag 13）、`resource/release/1`（资源释放，tag 14）。
  冻结能力分母由 8 项扩为 **9 项**（`ALL_FIDELITY_CAPS`）。
- 线词汇：`required:<cap>` / `optional:<cap>`；裸串按 optional 解释（旧 tail
  字节兼容）。声明可逆（`CapabilitySet::to_wire` round-trip）。
- **握手准入**：App 在 Hello 尾声明 `required:`/`optional:` 能力；Host 解析后若有
  不支持的必需能力 → `activate()` 返回 `ProtocolError::Admission(cap)` 且**保持
  Listening**（不半启动）。帧合成前再校验一次：缺必需能力即具名拒绝
  （`AdmissionError::UnknownRequiredCapability`），**不写槽**（保旧帧）并归还
  传输槽。v2 词汇在收窄能力集下仍受。
- **宿主能力下行**：App 在 Hello 声明 `optional:welcome-caps/1` 时，Host 会在
  Welcome 之后追加 `HandshakeMsg::HostCaps { caps }`（追加式 tag 4，未声明的
  旧端永远收不到）；App 存入 `peer_caps`，producer 因此可从握手而非本地开关
  决定是否用 v3 原语。
- **槽 = 信用**：帧/位图槽按"空闲池优先 → 从未用过的槽"分配；在途（未 ACK）
  槽不得覆盖；无信用即 `Backpressure`，调用方保留待发状态重试。ACK 只归还传输
  槽并清在途标记，**不代表 GPU 完成**（GPU 寿命由 Host fence 管理）。

## 5. 实测保真边界（2026-11-07，offscreen 代理终点）

两层证据，缺一不可：传输级（真实 wire + 两宿主）证明**链路不丢帧/不改帧**；
同进程矩阵证明**记录/降格本身的保真**。

### 5.1 传输级六格（`rq_wire`，真实管道 + 两宿主合成面）

链路：producer → `ClientPump`（Hello/激活/槽信用）→ 真实命名管道（编解码）+
shm 帧槽 + ACK → 宿主端点 → 两套**不同**的宿主合成实现 → broker canvas 快照：

- standalone：`rqhost::RqServe` / `adopt_one` / `pump_client`（独立 RQ 宿主同代码）；
- desktop：`DesktopSession` 的 broker 客户端（桌面嵌载宿主同代码）。

同 source/主题/1280×800@scale2，阈值 2/255（默认 producer = 字符串 + scrim）：

| 格 | 产者 | 例 | 像素差 |
|---|---|---|---|
| rust/standalone-rq、rust/desktop-rq | Rust 构件 | path_mesh（Path×4 / 12 段，tag 3） | 0.095% |
| 同上 | Rust | svg_icon（Image 资源经 shm 位图段） | 0.039% |
| 同上 | Rust | text_cjk（字符串面） | 0.094% |
| 同上 | Rust | gradient_border_shadow | 0.063% |
| vm/standalone-rq、vm/desktop-rq | VM（AutoLang 解释执行） | vm_text | 0.094% |
| 同上 | VM | vm_icon（`icon` → lucide svg 资源） | 0.057% |
| 同上 | VM | vm_row_button | 0.000% |

- 两宿主逐例读数**相同**：同一载荷 + 同一 `front_v2` 合成面代码，差异只可能来自
  传递层（当前为零）。
- **v3 tag 3 帧必须经槽分派到达宿主**（见 §1）。真实 wire 首次实测时该分派只认
  tag 2，v3 帧被当 v1 DrawList 解码失败后**静默丢弃**（path_mesh 合成面恒空）；
  修复后 path_mesh 在两种宿主上都是 0.095%。
- 边界：peer 跑本进程线程（真实管道/端点/shm/ACK），**不是独立进程**；present
  是 offscreen 快照。跨进程换载、物理上屏与真实刷新 deadline 归 OS063。

### 5.2 逐能力面（39 个 `View` 变体 ID，`rq_probe_t05_capability_rows`）

每个能力 ID 至少一例，走与产品同源的记录路径（`HeadlessSurface` →
`render_frame_v2` → broker canvas 重放），与 direct 金样（同一 `View` 的 iced
元素，不经协议）同主题/同 viewport 对拍；结构清单与资源计数一并落
`t05-capability-rows.json`。**覆盖分母不删**：缺例即测试失败。

- **26/39 ≤0.1%**（本页 §5.3 的 exactness 口径）。
- **>0.1% 的 13 个族（具名，诊断读数，不是 exactness 声明）**：managed_scroll_content
  1.562%、video 0.922%、terminal 0.456%、code_editor 0.407%、rich 0.314%、input 0.253%、
  scrollable 0.245%、text（latin 例）0.234%、navigation_rail 0.209%、table 0.203%、
  remote_frame 0.169%、autodown_editor 0.148%、mouse_area 0.136%。
  这 13 个族都是**文字密集或宿主依赖面**（编辑器/终端/富文本/输入/表格/导航栏/
  媒体占位），残差与 §5.3 的文字残差同源（画布文本管线 vs direct 文本管线的
  字形覆盖率/AA），不是位置或内容丢失。
- **逐 ID 的关闭条件**（AC-01）：每个 ID 有真实输出，且结构面断言——除
  `cap.view.empty`（0 op）与未注册的 `cap.view.custom`（防御臂 0 op）外，每个
  能力至少产出一个绘制 op；像素差设 **2% 毛刺级诊断上限**（抓整片丢失/底色
  错误），exactness 仍由 §5.3 的 0.1% 门负责。
- 底色口径（踩过的坑）：两侧都用主题底色 `bg()` 清屏；**不得**用 direct 图的
  左上像素当重放底色——内容压住 (0,0) 的例（mouse_area/table 等）会把它当底色
  刷满整屏，把 0.1% 级的差放大成 99%（已在本轮修掉并写进测试注释）。

### 5.3 同进程矩阵（`rq_cells`，记录/降格保真）

同 harness、同阈值；默认 producer 与 v3 富模式（glyph run + SDF 贴片，**opt-in**）
分别度量：

| 例 | 默认 producer（字符串 + scrim） | v3 富模式（opt-in） |
|---|---|---|
| path_mesh | 0.095% | 0.095% |
| svg_icon | 0.039% | 0.039% |
| text_cjk | 0.094% | **0.798%**（具名遗留） |
| gradient_border_shadow | 0.063% | **1.374%**（具名遗留） |
| scroll_clip | 0.000% | **0.330%**（具名遗留） |
| shadow_only | 0.005% | **0.741%**（具名遗留） |
| progress_bar | 0.006% | 0.006% |

- 默认 producer 全部 ≤0.1%（AC-02 r4 门）；v3 富模式是**具名边界**，不得计为
  exact，也不得用其读数替代默认面。
- 富模式残差同源：字形位图/阴影贴片经画布图像管线合成，与 direct 的文本/quad
  shader 合成路径不同（位置与几何已证相等：glyph `x` 与 cosmic-text `physical()`
  逐字相等；阴影贴片与 shader 同式求值）。
- 性能（p50，默认 producer）：produce 0.17–0.24ms（SVG 例外 ~32.5ms，根因是
  记录路径每帧新建 iced renderer 导致 iced 缓存冷启）、encode 0.005–0.008ms、
  decode 0.004–0.008ms、replay 5.2–6.5ms（首帧有一次 10.09ms 离群），direct 稳态帧
  p50 3.3–6.9ms。
- M 指标：M-04/M-08 measured；M-01/M-03/M-05 为 proxy（present 与 GPU 驻留未测）；
  M-02/M-06/M-07 unmeasured（真实 deadline、冷热启动、1/4/8 App 归 UI063/OS063）。

### 5.4 负控（AC-04 / V-04，真实管道）

三条负控都跑在真管道上（`rq_wire`，standalone 宿主可分别泵两端）：

| 负控 | 做法 | 实测 |
|---|---|---|
| 慢宿主 | 宿主停摆 250ms 只泵产者（tick 15ms） | 在途帧 **= 2**（槽数封顶）、待发位 **true**、宿主侧 0 帧；恢复后继续合成 ≥3 帧、待发位清空 |
| 旧 ACK | 宿主回 `FrameAck{frame_id:1, slot:1}`（早已归还的槽 + 过期 id） | 连接存活，帧数 2 → 3 继续增长，合成面仍有效 |
| 旧宿主 × 新产者 | 宿主能力面收窄为只有 glyphrun（无 `fidelity/pathfill/1`），产者先发 v3 Path 帧、再切回 v2 词汇 | v3 帧**入帧前拒绝**：`frames=0`、无合成面（不呈现半帧）、连接不掉；切回 v2 后同一连接恢复合成（1 帧字符串） |
| **真两进程** v2↔v3 配对 | spawn 两个**独立 exe**（`current_exe` 自派生，env 驱动）子产者，一个产 v3 Path 帧、一个产 v2 字符串帧，同一宿主实例收两路 | 两条连接来自两个不同 app（Hello 凭据），各自跨进程 shm/管道落地：v3 帧 `requires_v3()=true` 带 Path、v2 帧为字符串；两个子进程在配对期均存活 |

- 语义要点：慢宿主下**只有过期画面被合并**，帧槽按信用封顶，输入/资源依赖不丢；
  拒绝路径**保旧帧 + 归还槽**，不把“宿主已读走载荷”误当“已合成”。
- **乱序资源**：上传方向由显式 generation 守卫（更小/相等序号丢弃，见 §3）；
  资源**晚于**引用帧到达时帧照常呈现（`bitmap://` miss = 占位、不落负缓存），
  位图到达后同一 src 的下一帧/重绘命中。仍缺的是可控**重排通道**下的注入负控
  （管道 FIFO，天然保序）。
- 未覆盖：跨进程换载与物理上屏（OS063）、媒体资源族（见 §6）。

## 6. 具名遗留（不得当作已通过）

1. glyph run 边缘对齐：v3 富模式 0.80%（默认 producer 仍走字符串面，Host 用同一
   字体栈重排——可观测但**不是** AC-02 要求的“不重排+精确”）。
2. 阴影贴片合成残差：v3 富模式 0.74%（默认面仍是 scrim 近似 0.0625%）。
3. SVG 记录路径每帧新建 iced renderer → produce ≈32.5ms，超 061 冻结的 ≤10ms。
4. per-App CPU 预算/公平调度未实现（真实规模归 UI063/OS063）；上传预算记账目前
   只有端点产帧路径会调用，探针/新 FrameSource 需自行调用。
5. **媒体族**：**静态帧（海报/首帧/暂停帧）链路已交付**（`ui::media_poster` 供给面；
   `render_video` 命中即用普通 `image` 元素出图，记录器按栅格字节过线，资源身份/
   去重/预算/驻留全部复用图片管线）。实测：过线字节与发布帧逐字节相等、稳定帧
   0 重传、相对 direct 像素差 **0.000**。**连续播放帧未交付**——mpv 帧直写持久
   wgpu 纹理、无 CPU 读回，且 1080p RGBA ≈8 MiB/帧远高于冻结的 8 MiB/s 上传预算；
   按 2026-11-08 用户裁定另立计划（与 OS063 传输/媒体管线一起设计）。RemoteFrame
   的远端帧同属该后续面。
6. v2↔v3 双读已覆盖 tag 语义、拒绝路径、**同进程真实管道**（两种产者 × 两宿主）
   与**真两进程**（独立 exe 子产者）配对；慢 Host/旧 ACK/旧宿主负控均有实测
   （§5.4）。跨进程换载与物理上屏仍归 OS063。
7. ~~wire 无显式 generation 字段~~ **已交付（AC-03）**：`BitmapReadyGen`（tag 13）
   + `resource/generation/1` 协商，宿主按 `id` 拒迟到/重复上传（见 §3）。未覆盖
   的相邻项：**乱序资源负控**（BitmapReady 早于引用帧 / 帧引用尚未到达的 src）
   仍缺——需要可控的消息重排通道，当前管道是 FIFO。
8. ~~Host 侧没有跨进程释放/引用记账~~ **已交付（AC-03）**：`ResourceRelease`
   （tag 14）+ `resource/release/1` 协商 + 宿主 `BitmapRefs` 帧引用记账（见 §3）。
   仍属**单向通知**：无释放 ACK、无双向引用计数交换；Host 自身上限仍兜底。
9. 逐能力面 39 个 ID 中 **13 个族 >0.1%**（§5.2）：managed_scroll_content 1.562%、
   video 0.922%、terminal 0.456%、code_editor 0.407% 等，都是文字密集或宿主依赖面，
   残差与文字面同源（画布文本管线 vs direct 文本管线的字形覆盖率/AA）。**不属
   exact**，不得计入 AC-02 的 0.1% 门；逐 ID 的关闭条件是“有真实输出 + 结构断言
   + 2% 毛刺级上限”，不是逐 ID 都 ≤0.1%。
