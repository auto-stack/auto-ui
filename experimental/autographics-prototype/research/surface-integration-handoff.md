# PLAN-830 — Surface/Portal 组合与嵌套输入交接（V-10，2026-10-22 r6 work 更新）

本文件是 830 实验面到未来 **生产 AutoUI / RQHost / AutoShader(CAP)** 的交接清单。
它只记录**当前代码可核的事实**与**明确缺口**；不把实验 API 冒充生产 ABI，也不把
AutoShader/CAP 或 I1–I3 记作已交付。

## 1. 生产者 → 消费者（producer / consumer）

| 数据 | 生产者（当前代码） | 消费者 | 生命期与代际 |
|---|---|---|---|
| Surface 内容纹理 (`Rgba8UnormSrgb`, 预乘 alpha) | `host/surface_runtime.rs::render_content`（iced `Renderer::present` / `MeshPipeline::render_with` / `QuadPipeline::render_with`） | `SurfacePlanePipeline`（3D 平面）、`PortalQuadPipeline`（2D 四边形，scissor clip）、`FullScreenBlit`（窗口呈现） | 每 Surface 一个 `OffscreenTarget`（`new_sampled`），随 `resize_surface` 重建；`OffscreenTarget::build_seq` 是真实创建序号；graph 身份 = `core::id` 代际 |
| 组合结构（Surface/View/Portal + 挂载） | `core/composition::{CompositionGraph, CompositionChange}` | `render_order()/tick_order()`（调度）、`core::composition::input`（命中/捕获/焦点）、`host/composition_input.rs`（窗口映射/射线/锚点） | 一次 `submit` 整批校验 + 整状态替换；失败零副作用、不耗 id |
| 挂载变换（2D 仿射 / 3D 平面） | `MountKind::Flat2d{offset,scale,clip}` / `Plane3d{center,half}` | renderer（`PortalQuadPipeline`/`SurfacePlanePipeline`）、input（反算/射线，按 host Surface 取相机） | 图内不可变快照；`SetMount/AddPortal/RemovePortal/AddView/RemoveView/SetSurfaceSize` 显式置脏 owner + 消费者链 |
| 内容变更 | `mark_content_dirty`、结构变更提交、**内容适配器本帧输出变化**（`IcedSurface::last_change`：message/捕获事件/重绘请求/布局/真实 caret；3D 内容指纹） | `SurfaceRuntime::frame`（只跳过干净面；只对**本帧重渲染过内容**的父面做 portal 合成；本帧发生输出变化的 Surface 在同一帧把消费者链置为需重采样） | 消费者链级联；`clear_dirty` 每帧末；静态帧不重复叠层 |
| 输入事件/焦点权威 | `host/composition_input::{pointer_pressed,moved,released,cancel_all,window_focus_lost,revalidate,take_focus_transfer}`；router 目标三态写入 `CompositionGraph::{note_routed_focus,routed_focus,routed_focus_state,cancel_routed_focus,clear_routed_focus}`（`Unset/Target/Cancelled`） | `SurfaceRuntime::frame_with_cursors`（按面投递 iced 事件+光标；`(surface,local)` 由 router 返回并被真实控件消费；每面 `IcedSurface::focus_allowed` 由 router 终态决定，绘制前清除非目标面焦点/组合态） | capture 绑挂载路径 + generation；销毁/失焦/离窗 → 清 capture + 焦点并返回待 unfocus 的 Surface；press 迁移焦点 → `FocusTransfer{from,to}`。合批（A→B/B→A 同帧）以 router 终态为唯一真实编辑目标；公开 focus 先校验后迁移，非法引用拒绝无副作用 |
| IME 锚点 | **真实 caret** `IcedSurface`（iced `InputMethod::Enabled.cursor`，每帧 `RedrawRequested` 驱动）→ `anchor_to_window` / `ime_anchor_window` / `ime_caret_window_rect`（活跃焦点/捕获路径 → 挂载链 → 窗口 px）→ `set_ime_anchor_screen`/`client_to_screen` | OS IME（`ImmSetCompositionWindow`，CFS_POINT） | 焦点面 + 挂载路径 + 控件 caret；输入/选择/滚动/挂载变化即重算；无焦点不投递、不 fallback |
| 生命周期/失败计量 | `SurfaceRuntime::{stats.target_creations,target_drops,live_targets,owned_target_bytes}`、`OffscreenTarget::build_seq` | `surface-lifecycle` 门禁（真实计数 + `live_targets()==live_target_slots()==live_surfaces()` 守恒） | 创建成功即计；resize 替换、被丢弃的准备、slot 删除都计 drop；同尺寸 resize 为 no-op；失败 resize/add 保持 last-good |
| 上传计量 | `SurfaceRuntime::{plane_uploads,portal_quad_uploads,content_uploads,upload_stats}`；`SurfacePlanePipeline.uploads`（每 composite 128B）、`PortalQuadPipeline.uploads`（160B）、内容适配器 `MeshPipeline/QuadPipeline.uploads` | `surface-lifecycle` `metrics.composition.uploads`（采样点/单位/公式/排除项）+ 分管线精确门禁 + 总量=分项之和 + 省略计数负控 | 真实 `queue.write_buffer` 记账；iced 内部上传、纹理/bind group 创建、驱动侧拷贝明确未计；V-08 钩子可关闭平面/内容记账以证明门禁能识别遗漏 |
| 内存计量 | `SurfaceRuntime::owned_target_bytes()`（Σ`w*h*4`）+ 进程工作集采样 `metrics::sample_rss_mb` | `metrics.composition.memory`（采样点/单位/边界） | owned 目标字节与进程 RSS 分开；GPU/驱动驻留具名不可用（wgpu 27 无可移植查询） |

## 2. 当前 GPU 布局/绑定/错误事实（§5.6，可核）

| 管线 | stage IO | 绑定布局 | 混合/深度 | 错误出口（真实） |
|---|---|---|---|---|
| `SurfacePlanePipeline`（3D 平面） | vs: 6 顶点硬编码 → clip pos + uv@loc0；fs: `textureSample` | group0: u0 uniform(**128B** = model 64B + view_proj 64B；WGSL struct 128B，无外部 padding)、t0 texture_2d<f32> filterable、s0 Filtering | 预乘 over；无 depth（画家序） | 构造函数返回 `Result` 但当前未安装 wgpu error scope；管线/着色器创建错误**未捕获分类**，见 §3 |
| `PortalQuadPipeline`（2D 四边形） | vs: rect+parent_size → NDC；fs: 采样 | group0: u0 uniform buffer **分配 160B**，WGSL struct 实际 **32B**（rect 16 + parent_size 8 + _pad 8），每帧只写前 24B；t0、s0 | 预乘 over（`new_with_alpha(false)` 为负控 opaque replace）；`set_scissor_rect` 裁剪（`clip=None` 仅负控） | 同左；未捕获分类 |
| `FullScreenBlit`（呈现） | 全屏三角 | group0: t0+s0 | 不透明（alpha=1） | `SurfaceLost`（窗口目标既有路径） |
| iced 内容（离屏） | 公开 `iced_wgpu::Renderer::present(bg, fmt, &TextureView, &Viewport)` | iced 内部（Engine 的 quad/text 管线） | iced 自身预乘 alpha | `RendererError` 包装 + `IcedSurface` 目标错误 |
| `QuadPipeline`（2D 内容） | 2D quad/圆 | group0 camera uniform 64B + storage quads | 内容自绘 | `QuadPipeline::new` 用 error scope → `BadShader{stage,message}`（已分类） |
| `MeshPipeline`（3D 内容） | mesh VS/FS + shadow pass | camera/light/instance + mesh material bind group | depth32 + shadow | `MeshPipeline::new` 的着色器错误路径按其实现记录；未捕获的 wgpu 校验错误不冒记为已分类 |

资源/设备事实：单一 `GpuDevice`（`instance→adapter→device/queue`）；**Iced/2D/3D 内容适配器都记录创建时 device ordinal**，raw-device 构造无法证明归属时由运行时以 `DeviceUnverifiable` 拒绝；
`Limits::downlevel_defaults().using_resolution(adapter)`，无额外 features；
内容纹理 usage = `RENDER_ATTACHMENT|COPY_SRC|TEXTURE_BINDING`（旧 `OffscreenTarget::new`
保持 829 的非采样 usage，两合同显式区分）；`resize` 保持 usage。
`OffscreenTarget` 在 `create_texture` **之前**检查 `max_texture_dimension_2d`，
超限返回 `RendererError::TextureTooLarge`（不会 panic）；零尺寸返回
`ZeroSizeTarget` 并保留最后有效目标。
组合层具名错误：`InvalidSurface{detail}`（含目标创建失败）、
`DeviceMismatch{expected,actual}`（内容 device ordinal 与运行时不一致）、
`DeviceUnverifiable{detail}`（无法证明同设备归属，fail closed）、
`DependencyOrder{detail}`（脏子内容未在本帧渲染就被父采样）。
**未捕获分类**：`SurfacePlanePipeline`/`PortalQuadPipeline`/`FullScreenBlit`
的管线创建没有 error scope，wgpu 校验错误会走默认错误处理；不要把这条路径
写成“已全部分类”。GPU timestamp 仍未接线（`gpu_timing_available=false`）。

## 3. 明确缺口（不冒记已交付）

| 缺口 | 现状 | 归属 |
|---|---|---|
| OS 级中文 IME 组合 | **已取证**：`surface-ime-os` 在真实窗口中启用系统 IME，真实键盘产生 preedit 并提交 CJK（`abc你好你好`）；锚点取真实控件 caret（`InputMethod::Enabled.cursor`）沿活跃焦点挂载链投影 + `Window::set_ime_cursor_area` 每帧刷新，门禁断言锚点在投影 caret 矩形内且该矩形远小于平面；无焦点 IME 被丢弃、失焦清焦点 | 完成（候选窗逐像素截图比对可后续补） |
| 管线创建错误分类 | 组合管线未安装 error scope（见 §2） | 后续 renderer 专项 |
| 任意 3D 深度遮挡 | 平面按声明序画家式合成，未接 depth buffer | 后续扩展；当前 profile 明确平面为最前交互面 |
| 曲面/任意网格 Portal、跨进程 GPU 共享、多窗口焦点 | 非目标（计划 §1 profile） | 未来 Plan |
| AutoShader/CAP 源码入口、device IR、生成/热重载 | 未实现（计划 §5.6 明确不做） | 未来 AutoShader Plan |
| 生产 AutoUI / RQHost / Forge 接线 | 未接线（I1/I2/I3 pending 不变） | 迁移完成后的生产集成 Plan |
| GPU timestamp 计时 | 未接线（报告 `gpu_timing_available=false`，不伪造） | V-09 后续 |
| iced 剪贴板 | `Null` 剪贴板（复制/粘贴不可用） | 生产 adapter 专项 |
| 每帧重绘 | iced 内容与 3D 场景每帧无条件重绘；dirty 跳过仅对 2D canvas；本帧输出变化由 adapter 事实判定并同帧传播给消费者 | 后续优化 |
| GPU/驱动驻留内存 | 未测量：wgpu 27 无可移植的进程级 GPU 内存查询，DXGI budget 属 adapter 专属未实现；报告只给进程工作集与 owned 目标字节 | 后续 metrics 专项（不与 V-09 混淆） |

## 4. 复现命令（本树）

```bash
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --test surface_review_r2 --test surface_review_r3 --test surface_review_r4  # R2/R3/R4 永久回归 16 例
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked            # 162 例（含 surface_contract 29、surface_review_r2 5、surface_review_r3 6、surface_review_r4 5）
cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --test surface_contract
PROBE=experimental/autographics-prototype/target/release/graphics-probe.exe
$PROBE --case surface-offscreen --replay canonical --report scratch/830/surface-offscreen --mode window
$PROBE --case surface-compose   --replay canonical --report scratch/830/surface-compose
$PROBE --case surface-input     --replay canonical --report scratch/830/surface-input  --mode window
$PROBE --case surface-scenarios --replay canonical --report scratch/830/surface-scenarios
$PROBE --case surface-lifecycle --replay canonical --report scratch/830/surface-lifecycle
$PROBE --case surface-ime-os    --replay canonical --report scratch/830/surface-ime-os --mode window
$PROBE --case independent-consumer --replay canonical --report scratch/830/independent-consumer
```

`independent-consumer` 的 root-collateral 基线默认钉本 Plan 基面
`ab56c32ec4cb28e01fbcca74e9861164ffad43f0`（可用 `AGP_BASE_COMMIT` 覆盖）；
`docs/specs/auto-graphics/` 下的 provisional SD 草稿单独记录为
`provisional_spec_drafts`，其余根仓改动仍算 collateral。

829 的两游戏与旧探针命令保持其冻结 README 不变（本 Plan 未改其 golden/规则）。
