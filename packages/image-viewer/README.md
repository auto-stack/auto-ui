# image-viewer（PLAN-089 官方图片查看包）

通用受控图片视口 `ImageViewport` 与模态预览 `ImagePreviewDialog`。资源/解码
继续走既有 media ticket 通道（`/api/__auto/media/{id}/{revision}`）与
ImageSurface 管线；本包只持元数据与视图变换请求，不持字节、不建第二套
图像资源系统。

## 消费

```auto
widget Gallery {
    use { package: image_viewer from "../../../packages/image-viewer" }
    msg {
        Init, Tick,
        ApplyView(str), MarkLoaded(str), ShowError(str), Navigate(str),
        OpenPreview, ClosePreview
    }
    model {
        var interval int = 80
        var src str = "/api/__auto/media/lake/7"
        var rid str = "lake"
        var rrev str = "7"
        var rev int = 4
        var fit str = "contain"
        var zoom float = 1.0
        var ox float = 0.0
        var oy float = 0.0
        var rot int = 0
        var open bool = false
        var status str = ""
    }
    view {
        col (style: "gap-2 p-4") {
            button "预览" { onclick: .OpenPreview }
            ImagePreviewDialog (
                src: .src, resourceId: .rid, resourceRevision: .rrev,
                revision: .rev, fit: .fit, zoom: .zoom,
                offsetX: .ox, offsetY: .oy, rotation: .rot,
                open: .open, title: "图片预览",
                canPrevious: true, canNext: false,
                onviewRequested: .ApplyView($event),
                onloaded: .MarkLoaded($event),
                onerror: .ShowError($event),
                onnavigateRequested: .Navigate($event),
                oncancelled: .ClosePreview
            ) {}
        }
    }
    on {
        .OpenPreview -> { .open = true }
        .ClosePreview -> { .open = false }
        // 唯一业务 view 写点：按 baseRevision 校验后一次性赋值并增 revision。
        // 平面别名键读取（req.fit 等）——见事件表 `viewRequested` 行说明。
        .ApplyView(ev str) -> {
            let req = json.parse(ev)
            if req.baseRevision == .rev {
                .fit = req.fit
                .zoom = req.zoom
                .ox = req.offsetX
                .oy = req.offsetY
                .rot = req.rotation
                .rev = .rev + 1
            }
        }
        .MarkLoaded(ev str) -> { .status = f"loaded ${json.parse(ev).resourceId}" }
        .ShowError(ev str) -> { .status = f"error ${json.parse(ev).code}" }
        // 前后张：只收方向请求，资源集合/顺序归应用（查询/换图由应用做）。
        .Navigate(ev str) -> {
            let req = json.parse(ev)
            if req.direction == "next" { /* 应用换图：更新 src/revision */ }
        }
    }
}
```

## ImageViewport

受控图片视口。v1 仅受控视图：业务 view 由调用方持有，组件只持局部手势
投影（拖拽跟手显示、松手发一次请求并回弹等待接受）。

### props

| 名 | 型 | 默认 | 说明 |
|---|---|---|---|
| `src` | str | `""` | media ticket URI（`/api/__auto/media/...`） |
| `alt` | str | `""` | 替代文本（占位/无障碍） |
| `resourceId` | str | `""` | 资源身份（与 `resourceRevision` 组成有效身份） |
| `resourceRevision` | str | `""` | 资源修订（换图判定 + 事件回带） |
| `revision` | int | `0` | 受控 view 目标版本（回显在请求 `baseRevision`） |
| `fit` | str | `"contain"` | `contain\|width\|one-to-one\|free` |
| `zoom` | float | `1.0` | 缩放（界限 0.05…64，沿 ImageSurface clamp） |
| `offsetX` / `offsetY` | float | `0.0` | 平移（视口中心锚，逻辑 px） |
| `rotation` | int | `0` | 显示旋转（规范到 0…359；不修改原图） |
| `showToolbar` | bool | `true` | 工具栏显隐 |
| `canPrevious` / `canNext` | bool | `false` | 前后张可用性（无相邻图按钮禁用） |

### events（载荷均为 str JSON）

| 事件 | 载荷 | 说明 |
|---|---|---|
| `viewRequested` | `{seq, baseRevision, resourceId, resourceRevision, view:{fit,zoom,offsetX,offsetY,rotation}, fit, zoom, offsetX, offsetY, rotation, source}` | `source ∈ zoom\|pan\|fit\|rotate\|reset\|gesture`；一次一条完整 tuple。冻结的 `view` 对象之外另附**平面别名键**（`fit/zoom/offsetX/offsetY/rotation` 与 view 内同值）——`.at` 消费方在成员位读 `view` 键受 DotView 词元解析缺口影响（decisions.md §7 P4），平面键为语言缺口的适配通道；非 `.at` 消费方用 `view` 对象。**两处数值恒等，接受回路的唯一写点语义不变** |
| `loaded` | `{resourceId, resourceRevision}` | 当前资源加载完成（最小体，不捏造元数据） |
| `error` | `{code, message, resourceId, resourceRevision}` | `code ∈ load_failed\|invalid_view`；损坏/失效 URI 是 error |
| `navigateRequested` | `{seq, baseRevision, resourceId, resourceRevision, direction}` | `direction ∈ previous\|next`；只请求，集合/顺序归应用 |

### 行为边界

- 换图（`src`/`resourceId`/`resourceRevision` 任一变化）：手势与 loaded/
  error 门清零，初始基线 = 调用方随新图提供的 view（缺省
  `contain/1/0/0/0`）；旧图迟到事件不落新图（底层 src revision 门）。
- 同资源重绘/主题变化零请求、零重置（Tick 快照语义）。
- 同图双实例变换独立（实例态，无全局）。
- 拖拽：move 只更新局部投影，end 发一次 `source:"gesture"` 请求并回弹；
  调用方接受（增 `revision`）后落位，拒绝则维持原视图。
- 缩放步进 ×1.25/÷1.25；按钮/滚轮/拖拽同一变换算法（视口中心锚）。
- 非法 `fit` / 非有限 `zoom` → `invalid_view` 错误事件（同配置只报一次）。
- 键盘/MCP：全部动作有普通命名按钮；默认无全局快捷键（不抢占其他输入框）。

## ImagePreviewDialog

模态预览 = `dialog` chrome + 内嵌 `ImageViewport`，全部 viewport props +
`open: bool = false`、`title: str = "图片预览"`。事件同上转发，另有
`cancelled`（Escape/关闭按钮/程序性关闭，每次开启至多一次）。加载/错误
不阻塞关闭；关闭不写原图/数据库。

## 依赖闭包

- ImageViewport 使用 `std.json: json_str` 转义字符串字段，在组件内组装固定结构的事件 JSON。
- 内建：`col/row/text/button/icon/image_surface/dialog/dialog-content`
- 无第三方包依赖；包目录整体拷贝即脱离主源根可用（vendored 闭包）。

## 支持矩阵

| 轨道 | 状态 |
|---|---|
| VM native（iced，Windows/Linux） | wheel/pan/dblclick 经 PLAN-089 输入适配；按钮全量可用 |
| Vue（chromium/Playwright） | wheel `$event.deltaY`、pan 三相、`@dblclick` 同形 |
| MCP | snapshot + 普通按钮动作可完成全部操作（含平移键/工具栏） |

## 已知边界（v1）

- `error.code = "load_failed"` 为组件级分类（native 媒体门失败原因短文不
  透传占位）；`loaded` 不含宽高元数据。
- dblclick 无载荷（toggle 1:1/contain 语义）；wheel 锚点为视口中心（与
  ImageSurface 变换算法一致，不做光标锚第二套算法）。
- 文本字形渲染受 Q-01（DEBT-006，native input/text shaping）影响面与
  既有组件相同；受影响验收以真实截图对照关闭（见 docs/reviews/plan089/）。
