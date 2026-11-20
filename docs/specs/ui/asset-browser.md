# AssetBrowser & AssetPicker — 通用资源浏览与选择（docs/specs/ui/asset-browser.md）

来源：PLAN-091 r2（SD-01 / SD-02）。组件源 `packages/asset-browser/`；示例 `examples/asset-picker/` 与 `examples/ui/029-photo-gallery/`；产品门 `tests/product/verify_asset_picker.py`（native / vue）与 tools 门 `crates/auto-ui-tools/tests/asset_browser_component.rs`。

## 身份与边界（SD-01）

- 高层入口为 **`AssetBrowser`**（页面内嵌浏览面）与 **`AssetPicker`**（组合 `AssetBrowser` 与 `dialog` 的受控模态选择器）。
- 用于应用内既有授权资源库的分类、搜索、类型过滤、分页、单选、同面板图片预览与确认选择；**不**替代 `packages/file-dialog`（本地磁盘路径选择仍归 `FileDialog`），不枚举原始磁盘、不持有原始字节。
- 组件依赖使用各自的正式包源码。
  - `packages/tree-view/`（PLAN-085 `TreeView`）— 分类侧栏 `{id, label, children}` 薄适配；
  - `packages/pagination/`（PLAN-086 `PaginationControl`）— 1-based 结果分页；
  - `packages/image-viewer/`（PLAN-089 `ImageViewport`，pin=`5dbc3661`）— 同面板/同 Dialog 临时预览视口。
  - `packages/collection-view/`（PLAN-109 `CollectionView`、`Thumbnail`）提供条目网格、列表、缩略图和普通条目交互。AssetBrowser 在当前查询、结果版本和授权资源身份校验后继续发出已有公开事件。

## Props 与事件契约（r2 冻结）

### AssetBrowser

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `records` | `List = []` | 当前页记录 `{id, revision, label, kind, thumbnailUri, resourceRef}`；重复 `id` 或空 `id`/`revision` 报具名错误 `invalid_records` |
| `folders` | `List = []` | 嵌套分类节点 `{id, label, children}` |
| `query` | `str = ""` | JSON 查询状态 `{"id":…,"folderId":…,"text":…,"kind":…,"page":1,"pageSize":25}` |
| `querySeq` | `int = 0` | 应用最近接受请求的 `seq` |
| `resultQueryId` | `str = ""` | 当前 `records`/`total` 对应的 `query.id`；不匹配时标过期、不可确认 |
| `total` | `int = 0` | 过滤域总条数 |
| `selection` | `str = ""` | 当前受控选中项 JSON（`{id, revision, label}`）或空串 |
| `revision` | `int = 0` | 应用修订号，随 `baseRevision` 回传 |
| `loading` | `bool = false` | 查询加载态 |
| `errorMessage` | `str = ""` | provider 错误文案；非空时保留上一有效结果并显式标过期、封禁确认 |
| `viewMode` | `str = "grid"` | `"grid"` / `"list"` |
| `locked` | `bool = false` | 确认 `pending` 期锁定查询与选择变更 |

### AssetPicker

包含 `AssetBrowser` 全部 props（除内部 `locked`），并追加：`open: bool = false`、`title: str = "选择资源"`、`pending: bool = false`、`confirmation: str = ""`（`{"session":N,"seq":S,"status":"accepted"|"rejected","code":…,"message":…}`）。

### 事件

- `"queryRequested"(str)`：`{"seq":N,"baseRevision":R,"query":{folderId,text,kind,page,pageSize}}`。搜索由 Enter/按钮触发，分类/类型/分页即时触发。
- `"selectionRequested"(str)`：`{"queryId":…,"id":…,"revision":…,"label":…}` 或 `{"queryId":…,"clear":true}`。
- `"previewRequested"(str)`：`{"id":…,"revision":…,"label":…,"kind":…,"src":…}`；打开同面板 `ImageViewport`，不改变 `selection`。
- `"confirmRequested"(str)`：`{"seq":S,"session":K,"baseRevision":R,"id":…,"revision":…,"queryId":…}`。点击确认同步锁定本地请求，一轮最多一次；应用在唯一接受 handler 重新校验身份与权限后回灌 `confirmation`；`rejected` 解锁并显示错误，`accepted` 由父级关闭 `open=false`。
- `"cancelled"`：取消/关闭触发，作废本轮选择意图；旧确认反馈不得再写入业务引用。

## 消费与依赖闭包（SD-02）

- 消费方 `pac.at` 声明 `asset_browser`、`tree_view`、`pagination`、`image_viewer` 四包 `dep`；在 `widget` **内部** `use { package: asset_browser from "…" }`（不可置于文件顶层）。
- 移根/vendor 时四包整目录字节一致复制到第二根；缺依赖在 build/Vue 轨具名失败（`asset_browser_component.rs` 负例门验证）。

## 交付门（SD-02）

- tools 门：`cargo test -p auto-ui-tools --test asset_browser_component`（双生成器 + 移根闭包 + 缺依赖具名失败）。
- Win native VM 产品门：`tests/product/verify_asset_picker.py --backend native`（1003 规模/搜索/过滤/分类树/尾页/空错/慢 A 后 B/跨页选择/确认接受与拒绝/取消重开/双实例/深浅与窄窗截图 + `029-photo-gallery` 真实扫描/查看器/收藏/目录导航）。
- Vue 产品门：`tests/product/verify_asset_picker.py --backend vue`（Playwright 真实浏览器挂载/选择/确认落账/截图）。
- Linux native：无本机 runner，单列 `not_run`，不并入总绿。
