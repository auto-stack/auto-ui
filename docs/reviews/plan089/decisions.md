# PLAN-089 T-01 决策记录

按[执行与视觉指南 §5](../../../../docs/design/autoui/common-components-execution-and-visual-guide.md) 结构记录。机械决策随证据落在本页；语义仍以 Plan §5 r2 表为准。

## 1. baseline

| 项 | 值 |
|---|---|
| 仓库 / 树 | `auto-ui` @ `D:/autostack/.wt/ui-089/auto-ui`（组 `D:/autostack/.wt/ui-089/`） |
| 分支 / base | `codex/ui-089-image-viewer`，base = master `ba92217bfc1607e713feaf803e04e99655684d37` |
| 依赖兄弟树 | `auto-lang` @ `0b6d3198d`、`auto-web` @ `fd115ce0`、`auto-down` @ `2724389b`（均 master HEAD detached，与主检出同基线；auto-ui 工作区经 `../../../auto-lang|auto-web|auto-down` 相对 path 解析） |
| 观察源码 | `crates/auto-ui-compiler/schema/aura.at` imagesurface 块（:748-771）；`crates/auto-ui-runtime/src/ui/iced/image_surface.rs`（geometry/normalize）；`crates/auto-ui-runtime/src/ui/view.rs`（:1082 ImageSurface 节点、:2110 image_surface_events）；`crates/auto-ui-runtime/src/ui/iced/renderer/{mod.rs:1940-1978, projection.rs:3460-3486, media.rs:49-183}`；`crates/auto-ui-runtime/src/ui/session.rs`（PLAN-095 loaded/error 门表）；`crates/auto-ui-compiler/src/ui_gen/vue/{template_media.rs:2014-2044, script.rs:655-684, shadcn_attrs.rs:5128-5166}` |
| 局部 wrapper 摘要 | `examples/ui/031-image-viewer/src/front/image_viewport.at` SHA256 `84d748abdfae2a7b5bfca37592b7c8757b3af5250e14011e6d4b7e914b552525`（与 Plan §4 记录一致） |
| 新增路径 | `packages/image-viewer/`、`examples/image-preview/`、`crates/auto-ui-tools/tests/image_viewer_component.rs`、`tests/product/verify_image_preview.py`、`docs/specs/ui/image-viewport.md`（worktree 草拟，T-08 沉淀）、本目录 |
| 主检出状态 | 文档类未提交改动为其他任务既有 WIP；`crates/auto-ui-runtime/test/` 下未跟踪文件仅为 terminal_pixel 快照 PNG（测试瞬态产物），非源码 WIP，本计划不触碰 |

## 2. public interface（对 Plan §5 r2 表逐项核对）

- **事件承载机制**：包组件用带参 `msg` 通道（FileDialog/NumericField 先例）：`"viewRequested"(str)`、`"loaded"(str)`、`"error"(str)`、`"navigateRequested"(str)`、`"cancelled"`；消费方 `onviewRequested: .X($event)`；组件内 `."viewRequested"(json.stringify({...}))` 派发。跨后端一致（`.at`/SFC emit 层），不依赖 builtin 事件系统。
- **props**：按 r2 表 camelCase + 默认值（`fit:str="contain"`、`zoom:float=1`、`showToolbar:bool=true` 等）；底层 imagesurface 的 `src/alt/fit/zoom/offset_x/offset_y/rotation` 原名映射，不改名。
- **受控投影**：沿 NumericField 模式——Tick(interval) 对比 `revision`/受控 props 快照，变更才重投影；组件不发"自动重置"业务请求。
- **fixture / 生成**：探针 consumer + 生成结果见 §3；tools 门 T-05 落 `image_viewer_component.rs`（沿 `numeric_fields_component.rs` 双生成器 + vendored 闭包 + 具名缺依赖失败模板）。
- **loaded 最小体**：`{resourceId, resourceRevision}`（仅组件 props 已知字段，不捏造 w/h/字节数）；error `{code,message}` + 资源身份。

## 3. mechanism probe

### 3.1 已沿源码核实的链路事实

1. **规范变换**：`image_surface_geometry`（iced/image_surface.rs:58）——contain/width/one-to-one/free × zoom(clamp 0.05…64) × offset × rotation(rem_euclid 360)；命中/外接盒 `rotated_bounds`。这是唯一变换算法，按钮/滚轮/拖拽共用。
2. **规范事件负载**：`normalize_image_surface_event`（:242）——`Wheel{delta_y,x,y}`、`Pan{dx,dy,phase(Start/Move/End)}`、`DoubleClick{x,y}`、`Loaded{width,height,revision}`、`Error{code,message,revision}`；`ImageSurfaceInputState` 按 src revision 失效 pan/单次门——**迟到跨图事件在底层已丢弃**（r2 "旧 loaded/viewRequested 不落新图"的低层支撑）。
3. **Vue 侧已带参**：vue 生成器按绑定处声明参数生成全参调用（template_media.rs:2014-2044——pan 三相 `(dx,dy,phase)`；`handler_to_function_call_with_params` shadcn_attrs.rs:5160）；`@wheel/@dblclick/@load/@error` 齐全（image_surface_contract.rs:107-121 断言）。
4. **native 侧缺口（具名）**：Plan 547 Task 25 的 input adapter 不在树内——`View::ImageSurface.on_wheel/on_pan/on_double_click` 是无参 `Option<M>`，renderer 转换 `IcedMessage::from_dynamic` 丢参（renderer/mod.rs:1972），paint pass 显式忽略（projection.rs:3480 注释 "consumed by the input adapter (Task 25)"）；`render_image_surface`（media.rs）只渲染不含任何输入包装。**native wheel/pan/dblclick 事件今天不会到达 `.at` handler**；loaded/error 经 session.rs PLAN-095 per-App 门表另行派发。
5. **031 wrapper 缺陷证实**：`image_viewport.at` 把 `Wheel/Pan/DoubleClick` 以 `on_wheel()` 等无参转发（二次丢参）；且 031 自己的 `app.at` 并不消费该 wrapper（直接用 image_surface）——纯 shadow，T-04 移除。
6. **包消费机制**：`use { package: x from "<相对路径>" }`（component_registry.rs load_package，Builtin > Local > Package）；`package.at` 清单可选；`pac.at` 为应用描述符。包脱离主源根可解析（numeric_fields_component.rs vendored 用例先例）。

### 3.2 适配决议（r2 "修正适配"的落点）

- **native 输入适配（本计划内补）**：在 native ImageSurface 渲染路径补输入包装（PointerArea/Plan 499 先例：bounds 现场 + `shell.publish` 闭包），复用 `normalize_image_surface_event`+`ImageSurfaceInputState` 纯函数，把 Wheel/Pan/DoubleClick 规范负载以 `DynamicMessage::Typed{args: Vec<Value>}` 派发——`.at` handler 声明参数（`.OnWheel(delta float, x float, y float)`）即收坐标，与 Vue 全参形态对齐。不改 `image_surface_geometry`/`normalize_image_surface_event` 语义，不改底层接口名。
- **包组件不依赖 builtin 负载也能工作**：工具栏全部为普通 `button onclick`（MCP/键盘可操作），v1 按钮路径优先；手势路径（wheel/pan/dblclick）为增强通道，两后端行为对齐后 V-01 才判 pass。
- **变换一致性**：按钮缩放/平移与手势共用同一投影公式（viewport 中心锚 + fit_scale×zoom），在包内以整数有理化实现按钮步进、以事件坐标实现手势，均产出同一 viewRequested tuple；不得出现两套算法。
- **失败控制**：非法 fit/非有限 zoom → `invalid_view` error 事件，不派发 viewRequested；zoom 夹取 0.05…64 与 `image_surface_geometry` clamp 一致（夹取后同值零事件）。

### 3.3 整链探针（待构建窗口）

命令与结果（baseline image 回归 + 最小 consumer 双生成器）在构建窗口开放后回填：

- [ ] `cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity`（改动前基线）
- [ ] 最小 consumer（包 + App 绑定 viewRequested/loaded）经 `ui_build(rust)` + `ui_build(vue)` 生成成功
- 结果：待回填

## 4. dependencies

| 依赖 | 交付状态 | 本计划用法 |
|---|---|---|
| ImageSurface / media ticket（547 系） | 已交付（本仓） | 唯一资源/解码通道；不改 media 服务 |
| NumericField 受控投影模式（087） | 已交付（packages/numeric-fields） | Tick 快照 + seq/baseRevision 载荷模板 |
| FileDialog/PaginationControl 事件与 modal（084/086） | 已交付（packages/file-dialog、pagination） | 带参 msg、dialog 组合、`onaccepted: .X($event)` 接法 |
| 087 数值组件（Plan §3 提及"087为可复用优化而非硬前置"） | 已交付 | 不作硬依赖；缩放数值输入首版用普通 button/input |
| 媒体 fixture（031 tests/fixtures） | 已交付 | rgb-1x1 三格式、orientation-6、alpha-2x1、corrupt.bin、scenic-320x240 + manifest.json；T-04 另补非对称角点图 |
| 跨仓媒体服务变更 | **无** | 发现需要时具名返回，不在本计划内写 |

## 5. runners

| 轨道 | 入口 | 状态 |
|---|---|---|
| 原生回归（单测） | `cargo test -p auto-ui-runtime --test image_surface_contract --test image_pipeline_parity`；tools 门 `cargo test -p auto-ui-tools --test image_viewer_component` | 可用（本机 Windows；构建窗口受他组编译占用约束，见 §3.3） |
| native 产品门 | `python tests/product/verify_image_preview.py --auto-bin <auto> --ui-tool-bin <tool> --out-dir <isolated> --backend native`；会话 = `auto run -r vm` + `AUTOUI_MCP_PORT` + MCP HTTP（autoui_snapshot/action/keyboard），就绪文本/退出控件按本例适配，不复用 FileDialog Session 构造器（沿 verify_numeric_fields.py:60-130 模式） | Windows 可用；Linux runner 本机无 → 单列 not_run |
| Vue 产品门 | 031 先例 `tests/vue-actions.json` 经标准 Playwright runner；030 `tests/`（playwright.config.ts + smoke.spec.ts）为启动参考。**旧 030 媒体盘/autoplay 配置不照抄** | 本机 chromium 可用性 T-05 实测后回填；不可用则该轨 not_run，不并入总绿 |
| 031 vm_matrix.py | 含旧二进制默认路径——runner 显式 pin `--auto-bin/--ui-tool-bin`，不拿旧默认路径作证据 | 已确认风险，T-05 规避 |

## 6. next

- 允许进入：T-02（`packages/image-viewer/` 抽取 + native 输入适配实现）。
- 前置缺口：无阻塞性缺口；§3.3 探针回填是 T-02 关闭条件之一。
- 未决机制：无混入 T-02 的自由设计——native 适配按 §3.2 决议执行；若实现中证明 `DynamicMessage::Typed` args 无法从渲染层派发，回本页增补决议（不 silent 改合同）。
