# PLAN-830 T-02 — 反向离屏 UI whole-path 可行性结论（2026-10-22）

**结论：可行，已实证。** 真实 iced 0.14 UI（stock `button` + `text_input`）
可以在不运行 `iced::application` 的情况下，完全经公开 API 渲染进离屏纹理、
被 3D 透视平面采样、呈现到同一窗口，并把窗口输入映射回真实控件。窗口模式
canonical 探针全门禁通过（`surface-offscreen` case，`scratch/830/surface-offscreen`
report.json，presses=1、input="ok8"、6/6 gates）。T-03 起的依赖实现按本文合同推进。

## 1. 公开接法（唯一确定路线，未 fork、未 patch）

```
GpuDevice（宿主唯一 Device/Queue，AGP 既有）
  └ iced_wgpu::Engine::new(adapter, device.clone, queue.clone,
          Rgba8UnormSrgb, None /*antialias*/, Shell::headless())   // 公开
      └ iced_wgpu::Renderer::new(engine, Font::DEFAULT, 16px)      // 公开
          └ 每帧：UserInterface::build(view, bounds, cache, &mut r) // iced_runtime
                   UserInterface::update(events, cursor, r, &mut Null剪贴板, &mut msgs)
                   应用 msgs → 重建 view → UserInterface::draw(r, theme, style, cursor)
                   Renderer::present(bg, format, &离屏TextureView, &Viewport)  // 公开
```

- `iced::Element` 的默认 Renderer 是 fallback 类型（wgpu+tiny_skia）；元素必须
  钉在 `iced_wgpu::Renderer` 上（`SurfaceElement` 别名）。`iced::core` 路径不
  公开，统一走 `iced_wgpu::core` / `iced_wgpu::graphics` 再导出。
- AGP 为此新增直接依赖 `iced_runtime = "0.14"`（stock，同版本线；计划 §3 允许，
  这正是 T-02 点名的"公开 Iced runtime UserInterface 边界"）。
- 控件布局**精确探查**用公开 `widget::Operation`（`text_input`/`text` 钩子）；
  iced 0.14 `button` 不走 `focusable` 钩子——按钮 bounds 以其标签文本
  （"count: …"）为主锚，focusable 命中只作后备。
- 一次 frame = build→update→(应用消息)→build→draw→present；消息不滞后一帧。

## 2. GPU/纹理/色彩合同（T-04 沿用）

- 离屏内容目标：`OffscreenTarget::new_sampled`（usage=RENDER_ATTACHMENT|
  COPY_SRC|**TEXTURE_BINDING**）。旧 `new`（不可采样）语义保留，两种合同
  显式区分；`resize` 保持原 usage（本 T 修复的实错：resize 曾把采样目标
  降级回不可采样，窗口首帧即触发 wgpu validation error）。
- 格式链：内容纹理 `Rgba8UnormSrgb`（sRGB 编码、**预乘 alpha**——iced 输出）；
  采样得线性预乘；写回 sRGB 目标由 wgpu 转码；合成 blend=预乘 over
  （src One，dst OneMinusSrcAlpha）。V-02 平面区域当前为不透明 UI 底色；
  半透明彩色基线按计划在 T-04 冻结后验收（本 T 未冒充已通过）。
- 提交顺序=DAG 序：先内容（iced present 自提交），再 3D 场景（MeshPipeline
  自带深度 pass），再平面 pass（load-op，画家序：平面是本场景声明的最前
  可交互物，其后无遮挡物）；同一 pass 内不采样正在写的纹理（结构保证）。
- 稳态零 GPU→CPU readback；readback 仅出现在验收 capture（4 次/全程，门禁
  上限 4）。单一 DeviceToken（`dev{ordinal}:{adapter_digest}`）贯穿。

## 3. 输入返送事实（T-05 泛化、T-06 IME 的基础）

- 坐标链实测往返误差 **0.000px**（UI local → 平面 world → 正投影 window px →
  逆投影 → ray/平面求交 → UV → local）。
- OS 注入的三个实证坑（全部已修，写入代码）：
  1. **DPI 虚拟化**：注入进程 DPI-unaware 时，PostMessage 坐标被 OS 按 2×
     缩放（实测 posted (300,400) → winit (601,800)@scale2.0）；注入脚本必须先
     `SetProcessDPIAware()`（之后 GetClientRect 返回物理 1800×1280）。
  2. **winit 不处理 WM_CHAR**：文本在 WM_KEYDOWN 内经 ToUnicode 派生；键盘
     注入须投递带 VK+扫描码的 WM_KEYDOWN/WM_KEYUP 对（WM_CHAR 无效）。
  3. 按下/释放对不得与"移开光标"同帧批量（button 要求 release 时光标仍在
     bounds 内）——headless 事件序列同样适用。
- 点击/键盘优先走真实前台输入（mouse_event/SendKeys），会话拒绝前台时回退
  PostMessage 注入（仍是 winit 真实消息管线，报告如实标注 `os_window_events`；
  与 829 spike 的 posted_queue 证据同一诚实等级）。

## 4. 资源/预算/诊断（供 V-07/V-09）

- 每 UI 实例 = 1 Engine + 1 Renderer + 1 采样纹理（480×360 起步）；纹理随
  resize 重建，Engine/Renderer 存续（局部变更，无全局重建）。
- 帧 CPU 成本（release，本机）：35 帧 headless 全绿路径无卡顿；计量归 V-09。
- 诊断开关：`AGP_INPUT_DEBUG=1` 打印 CursorMoved/MouseInput/Key/目标坐标
  （env 门控，默认静默，不影响正常路径）。

## 5. §5.6 Shader 资源/布局/绑定事实（当前实现可核，供 V-10）

| 事实 | 值（`renderer/composition/mod.rs`，T-02 冻结源） |
|---|---|
| SurfacePlanePipeline WGSL stage IO | vs: vertex_index→(clip pos, uv@loc0)；fs: 采样返回 vec4 |
| 绑定 | @group(0)：u0 uniform(128B, VERTEX)、t0 texture_2d<f32> filterable(FRAGMENT)、s0 Filtering sampler |
| uniform 布局 | [0..64)=model mat4x4 col-major、[64..128)=view_proj mat4x4；无 push constant；对齐=vec4 天然满足 |
| 顶点数据 | 无 vertex buffer（6 顶点硬编码三角带，本地 X→U、-Y→V） |
| blend | 预乘 over（color/alpha 均 One/OneMinusSrcAlpha）；depth 无（画家序合同） |
| FullScreenBlit | 全屏三角（opaque alpha=1），同纹理/采样器布局，无 blend |
| features/limits | 空额外 features；`Limits::downlevel_defaults().using_resolution(adapter)`（GpuDevice 既有） |
| 错误分类 | 管线创建失败→`RendererError::BadShader{stage}`；目标尺寸 0→`ZeroSizeTarget`（既有 taxonomy 未扩） |

未来 AutoShader/CAP：以上为**当前手写 WGSL 的布局事实**，device 端编译产
物须经同一绑定/布局接缝进入；本 Plan 不实现生成/热重载（计划 §5.6 不变）。

## 6. 边界与遗留（不冒充已完成）

- 3D 平面遮挡/深度策略：T-02 场景平面为声明最前物（画家序）；任意深度
  组合与遮挡合同是 T-04/T-05 的正式范围。
- 捕获/焦点/IME/多实例：T-06/T-07；本 T 只有单 UI 单平面。
- 负控（sabotage）门禁：V-08/T-08；T-02 报告中 negative_fired=false 是诚实
  标注（仅正控）。
