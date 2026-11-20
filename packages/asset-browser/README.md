# packages/asset-browser — AssetBrowser & AssetPicker（PLAN-091）

通用应用内授权资源浏览与选择组件（`AssetBrowser` / `AssetPicker`）。用于附件、图片素材、封面等**应用内既有授权资源库**的浏览与选择；组合 `packages/tree-view`（PLAN-085 分类树）、`packages/pagination`（PLAN-086 分页）与 `packages/image-viewer`（PLAN-089 同面板预览）。

本包**不**承担本地磁盘枚举、文件上传/删除或替代 `packages/file-dialog`（本地路径选择仍归 `FileDialog`）。

视觉参考：[通用组件执行与视觉指南 §3](../../docs/design/autoui/common-components-execution-and-visual-guide.md)；合同与执行记录：[PLAN-091](../../docs/plans/091-asset-browser-and-picker.md)。

## 组件与文件

| 文件 | 作用 |
|---|---|
| `asset_browser.at` | `widget AssetBrowser` — 浏览面（分类树、搜索、类型过滤、grid/list 视图、分页、单选、同面板图片预览） |
| `asset_picker.at` | `widget AssetPicker` — 组合 `AssetBrowser` 与 `dialog` 的受控模态选择器（会话/防重锁/确认反馈/取消） |
| `asset_kind_icon.at` | `widget KindIcon` — 缩略图缺失/非图片类型的图标兜底 |
| `asset_browser_util.at` | 记录校验、分类树节点适配、索引查找与派生辅助 |
| `package.at` | 包清单 |
| `pac.at` | dep 物化描述 |

## Props（r2 冻结合同）

### AssetBrowser

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `records` | `List = []` | 当前页记录列表：`{id, revision, label, kind, thumbnailUri, resourceRef}`；重复 `id` 或非法结构报 `invalid_records` |
| `folders` | `List = []` | 嵌套分类树 `{id, label, children}`（薄适配 `TreeView`） |
| `query` | `str = ""` | JSON 查询状态：`{"id":"q2","folderId":"nature","text":"lake","kind":"image","page":1,"pageSize":25}`（`page` 为 1-based） |
| `querySeq` | `int = 0` | 应用最近接受的请求 `seq`（与组件发出序列对齐，判定最后请求是否已反馈） |
| `resultQueryId` | `str = ""` | 当前 `records`/`total` 对应的 `query.id`；与 `query.id` 不一致时结果标过期、不可确认 |
| `total` | `int = 0` | 当前过滤域总条数（供 `PaginationControl` 投影） |
| `selection` | `str = ""` | 当前受控选中项 JSON（`{id, revision, label}`）或空串 |
| `revision` | `int = 0` | 应用资源/查询修订号，随上行请求 `baseRevision` 回传 |
| `loading` | `bool = false` | 查询进行中；期间禁用过时交互并显示加载态 |
| `errorMessage` | `str = ""` | 请求/provider 错误文案；非空时保留上一有效结果并显式标过期、封禁确认 |
| `viewMode` | `str = "grid"` | `"grid"` / `"list"` 视图形态 |
| `locked` | `bool = false` | 确认 `pending` 期锁定查询与选择变更（`AssetPicker` 自动传入 `confirm_lock || pending`） |

### AssetPicker

包含 `AssetBrowser` 全部 props（除内部 `locked`），并追加：

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `open` | `bool = false` | 受控开关；`false → true` 递增会话 `session` |
| `title` | `str = "选择资源"` | 对话框标题 |
| `pending` | `bool = false` | 父级正在执行确认验证；期间锁定查询/选择/重复确认，取消仍可用 |
| `confirmation` | `str = ""` | JSON 确认反馈：`{"session":2,"seq":15,"status":"accepted"\|"rejected","code":…,"message":…}` |

## 事件（带引号 msg 变体）

| 事件 | 触发方 | 载荷（JSON 字符串） | 语义 |
|---|---|---|---|
| `"queryRequested"(str)` | Browser / Picker | `{"seq":N,"baseRevision":R,"query":{folderId,text,kind,page,pageSize}}` | 搜索（Enter/按钮）、分类/类型切换或分页变化时发出 |
| `"selectionRequested"(str)` | Browser / Picker | `{"queryId":…,"id":…,"revision":…,"label":…}` 或 `{"queryId":…,"clear":true}` | 单击卡片/列表行或键盘 `Enter` 发出选择意图 |
| `"previewRequested"(str)` | Browser / Picker | `{"id":…,"revision":…,"label":…,"kind":…,"src":…}` | 点击预览按钮触发，同时在 Browser 内打开同面板 `ImageViewport` 预览 |
| `"confirmRequested"(str)` | Picker | `{"seq":S,"session":K,"baseRevision":R,"id":…,"revision":…,"queryId":…}` | 确认点击时同步锁定本地请求（一轮最多一次），由应用校验身份/权限 |
| `"cancelled"` | Picker | （无载荷） | 关闭按钮或取消点击触发，作废本轮选择意图 |

## 消费方式（依赖闭包与可运行样例）

1. 在消费方 `pac.at` 声明四包依赖（或整组 vendor 到第二根后指向同一绝对/相对目录）：

```at
dep asset_browser { path: "../../packages/asset-browser" }
dep tree_view     { path: "../../packages/tree-view" }
dep pagination    { path: "../../packages/pagination" }
dep image_viewer  { path: "../../packages/image-viewer" }
```

2. 在消费方 `widget` **内部**导入（勿放在文件顶层）并绑定受控 props 与事件：

```at
widget App {
    use { package: asset_browser from "../../packages/asset-browser" }
    // ...
    view {
        AssetPicker (
            open: .open,
            title: "选择附件资源",
            pending: .pending,
            confirmation: .confirm_fb,
            records: .records,
            folders: .folders,
            query: .query,
            querySeq: .query_seq,
            resultQueryId: .result_qid,
            total: .total,
            selection: .selection,
            revision: .revision,
            loading: .loading,
            errorMessage: .error_msg,
            onqueryRequested: .OnQuery($event),
            onselectionRequested: .OnSelect($event),
            onpreviewRequested: .OnPreview($event),
            onconfirmRequested: .OnConfirm($event),
            oncancelled: .OnCancel
        ) {}
    }
}
```

完整可运行样例：
- 模态选择器 + 授权资源 provider（1003 条规模、代次门、确认校验、附件日志）：[`examples/asset-picker/`](../../examples/asset-picker/)
- 嵌入页面库面消费（真实照片扫描 + 查看器 + 收藏 + 目录导航）：[`examples/ui/029-photo-gallery/`](../../examples/ui/029-photo-gallery/)

## 已知边界（v1）

- 依赖闭包需 `asset-browser` + `tree-view` + `pagination` + `image-viewer` 四包齐备；缺依赖在 Vue/build 轨具名失败，native 轨静默缺席（由 `crates/auto-ui-tools/tests/asset_browser_component.rs` 兜底）。
- 缩略图与预览 `src` 必须为应用提供的授权媒体 URI（opaque），组件不持有原始字节或拼接本地磁盘路径。
- Linux native 轨未在本机环境运行（门脚本已保留 `native` 入口，无 Linux runner 时单列 `not_run`）。
