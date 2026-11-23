# ImageViewport / ImagePreviewDialog — 通用受控图片查看（docs/specs/ui/image-viewport.md）

来源：PLAN-089 r2（SD-01/SD-02）。组件源 `packages/image-viewer/`（`package.at`/
`pac.at`/`image_viewport.at`/`image_preview_dialog.at`/`README.md`）；消费演示
`examples/image-preview/`；产品门 `tests/product/verify_image_preview.py`
（native/vue）与 tools 门 `crates/auto-ui-tools/tests/image_viewer_component.rs`。

## 身份与共存

- 高层入口是包组件 **`ImageViewport`**（受控视口）与 **`ImagePreviewDialog`**
  （模态预览 = dialog chrome + 内嵌视口的事件纯转发组合）。两件包组件不占任何
  内置 schema 名，Builtin > Local > Package 优先级不变。
- 组件不持字节、不解码、不发业务请求：像素/解码/ticket 生命周期继续归既有
  media 管线（`/api/__auto/media/{id}/{revision}`）与消费方的媒体服务；组件只持
  元数据与视图变换请求。v1 仅受控视图：业务 view 由调用方持有，组件只持局部
  手势投影（拖拽跟手显示、松手发一次请求并回弹等待接受）。

## Props 与事件（冻结契约）

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `src` | `str = ""` | media ticket URI（opaque；组件不做文件 IO） |
| `alt` | `str = ""` | 替代文本（占位/无障碍） |
| `resourceId` / `resourceRevision` | `str = ""` | 有效资源身份 `(resourceId 或 src, resourceRevision)`；任一变化即换代 |
| `revision` | `int = 0` | 受控 view 目标版本；请求回显为 `baseRevision` |
| `fit` | `str = "contain"` | `contain\|width\|one-to-one\|free`；非法值 → `invalid_view` 错误事件（同配置只报一次） |
| `zoom` | `float = 1.0` | 界限 0.05…64（沿 `image_surface_geometry` clamp）；越界夹取、非有限数 → `invalid_view` |
| `offsetX` / `offsetY` | `float = 0.0` | 平移（视口中心锚，逻辑 px） |
| `rotation` | `int = 0` | 显示旋转，规范 0…359；不修改原图 |
| `showToolbar` | `bool = true` | 工具栏显隐（全部动作有普通命名控件，MCP/键盘可操作） |
| `canPrevious` / `canNext` | `bool = false` | 前后张可用性；无相邻图按钮禁用 |
| `open`（Dialog） | `bool = false` | 受控开合；关闭派发 `cancelled`（每次开启至多一次） |
| `title`（Dialog） | `str = "图片预览"` | 对话框标题 |

事件（载荷 str JSON）：`viewRequested`、`loaded`、`error`、`navigateRequested`、
`cancelled`（Dialog）。`viewRequested` 一次携带完整 tuple
`{seq, baseRevision, resourceId, resourceRevision, view:{fit,zoom,offsetX,offsetY,rotation}, source}`，
`source ∈ zoom|pan|fit|rotate|reset|gesture`。**平面别名键**：载荷顶层另附
`fit/zoom/offsetX/offsetY/rotation`（与 `view` 内同值）——`.at` 消费方在成员位读
`view` 键受 DotView 词元解析缺口影响（PLAN-089 decisions.md §7 P4），平面键是该
语言缺口的适配通道；两处数值恒等，冻结 `view` 对象保留给非 `.at` 消费方。
`loaded` 最小体 `{resourceId, resourceRevision}`（不捏造未回传的元数据）；`error`
含 `{code, message, ...}`，`code ∈ load_failed|invalid_view`。

## 行为语义（双端一致）

- **受控回路**：请求 → 调用方按 `baseRevision` 校验 → 一次性赋值并增 `revision`
  → 组件 Tick 重投影。拒绝 = 保持原视图（请求不作成功论）；同值零事件。
- **换代**：`(src, resourceId, resourceRevision)` 任一变化即换代——手势与
  loaded/error 门清零，初始基线 = 调用方随新图提供的 view（缺省
  contain/1/0/0/0）；旧图迟到事件不落新图（底层 src revision 门
  `ImageSurfaceInputState`）。
- **同资源重绘/主题变化**：零请求、零重置（Tick 快照语义）。
- **窄窗工具栏（420×560）**：一行内保留前后图/缩放/适应/宽度/1:1 与具名
  **“更多”**；旋转（左转90°/右转90°）、重置、平移收入“更多”菜单。12 个动作
  均为普通命名控件，420 宽不裁切（工具行 + 菜单逐控件 rect 断言），Dialog
  关闭控件始终可达。
- **变换一致性**：按钮/滚轮/拖拽共用 `image_surface_geometry` 同一算法（视口
  中心锚）；缩放步进 ×1.25/÷1.25（二进制精确）。
- **Dialog**：加载/错误不阻塞关闭；关闭不写原图/数据库；共享资源引用归媒体
  管线。

## 平台适配（PLAN-089 落地的运行时面）

- **native 输入适配**：`ui/iced/image_surface_input.rs`——Plan 547 Task 25 缺口
  的落地。`View::ImageSurface` 输入三臂（wheel/pan/dblclick）改为
  `ImageSurfaceCallback`（绑定位声明参数 → 事件期追加固定载荷；空参绑定零载荷，
  与 Vue「空声明 → 无参调用」同语义）；`ImageSurfaceInput` 包装 widget 复用
  `normalize_image_surface_event` + `ImageSurfaceInputState`。载荷形状（跨后端
  冻结）：wheel `(delta_y)`、pan `(dx, dy, phase:"start"|"move"|"end")`、
  dblclick 无载荷。Vue 侧 wheel 参数由生成器 `$event.deltaY` 替换承担。
- **媒体通知 sweep 适配**：包组件实例的 handler 基座名归约到类型名
  （实例 path/`@seq`/`#n` 装饰）；根 bridge 声明表之外的子件 handler 按零参
  回退派发（onload/onerror 常规绑定形态），不再整事件丢弃。
- **语言缺口适配**：`view` 为 `.at` 保留字——载荷键以带引号 `"view"` 输出
  （解析探针 d1/d2），顶层另附平面别名键；消费方用平面键或 `req["view"]`
  索引（成员位 DotView 词元静默吞键）。

## 受检包的 Iced 视口（PLAN-117）

受检包的 Iced 图像节点将加载、失败、滚轮和平移事件交给当前挂载且源版本匹配的组件。渲染器按视口与裁剪区绘制已接受的变换。媒体 URI 由受检宿主和发送请求的文件授权提供。查看器关闭或授权撤销时，宿主回收对应的媒体运行时和票据。

## 消费方式与依赖闭包

1. 消费方 `pac.at`：`dep image_viewer { path: "…" }`（vue 轨物化扫描）。
2. widget 内：`use { package: image_viewer from "…" }`。
3. 受控回路：`viewRequested` → 校验 baseRevision → 原子赋值 + revision++ →
   组件 Tick 重投影；navigateRequested 只发方向，集合/顺序归应用。
4. vendored：整目录复制即脱离主源根可用（`image_viewer_component.rs` 第二根
   用例 + 具名缺依赖失败）。

## 验证入口

- tools 门：`cargo test -p auto-ui-tools --test image_viewer_component`
  （双生成器 + 载荷形状 + vendored 闭包 + 具名缺依赖失败）。
- 产品门：`python tests/product/verify_image_preview.py --auto-bin <auto>
  --ui-tool-bin <tool> --out-dir <isolated> --backend native|vue`
  （V-01…V-06 实矩阵；gate 期夹具与包路径重写为绝对、finally 字节恢复）。
- 既有回归：`cargo test -p auto-ui-runtime --test image_surface_contract
  --test image_pipeline_parity`。
