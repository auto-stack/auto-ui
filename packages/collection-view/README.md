# CollectionView 与 Thumbnail

`CollectionView` 在网格和列表之间切换，并把选择、条目动作、排序和模式请求交给应用。应用持有排序后的当前结果、受控选择和业务动作。 `Thumbnail` 可以独立使用，也由 CollectionView 默认组合。

```at
dep collection_view {
    path: "../../packages/collection-view"
}
```

```at
use { package: collection_view from "../../../../packages/collection-view" }

CollectionView (
    items: .items, columns: .columns,
    viewMode: .mode, selectedId: .selected,
    scopeId: .scope, revision: .revision,
    onselectionRequested: .Select($event),
    onviewModeRequested: .ChangeView($event),
    onitemActionRequested: .Act($event),
    onsortRequested: .Sort($event)
) {}

Thumbnail (src: .authorized_uri, alt: "缩略图", state: "ready",
           resourceId: .id, resourceRevision: .resource_revision) {}
```

事件载荷是 JSON 字符串。消费端使用当前挂载实例、scope、结果 revision 与条目身份验证后再处理。公开接口与可观察行为见 [CollectionView 规范](../../docs/specs/ui/collection-view.md)。可运行消费代码见 [独立示例](../../examples/collection-view/src/front/app.at)、[FileManager](../../examples/ui/027-file-manager/src/front/app.at) 与 [AssetBrowser](../asset-browser/asset_browser.at)。

在 `examples/collection-view` 运行 `auto run -r vm` 或 `auto run -r vue`。示例提供同名条目、禁用条目、列表表头与更新版本的动作。产品验收入口为 `tests/product/verify_collection_view.py`，可用 `--consumer file-manager` 复验预启动的 027 和隔离目录。AssetPicker 与 Gallery 使用 `tests/product/verify_asset_picker.py`。
