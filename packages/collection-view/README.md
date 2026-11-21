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

`items` 和 `columns` 分别接收 `List<CollectionItem>` 和 `List<CollectionColumn>`，类型由 `collection_view.collection_schema` 导出。没有单元格或动作时传空列表。没有缩略图 URI 或默认动作时传空字符串。空 `thumbnailState` 按是否提供 URI 决定显示图片或占位图。

事件载荷是 JSON 字符串。消费端使用当前挂载实例、scope、结果 revision 与条目身份验证后再处理。公开接口与可观察行为见 [CollectionView 规范](../../docs/specs/ui/collection-view.md)。可运行消费代码见 [独立示例](../../examples/collection-view/src/front/app.at)、[FileManager](../../examples/ui/027-file-manager/src/front/app.at) 与 [AssetBrowser](../asset-browser/asset_browser.at)。

在 `examples/collection-view` 运行 `auto run -r vm` 或 `auto run -r vue`。示例提供同名条目、禁用条目、列表排序、移除条目与更新版本的动作。准备工具和应用后，使用 `tests/product/verify_collection_view.py` 运行日常产品门。`--consumer file-manager` 连接已启动的 027 和隔离目录，`--consumer asset-picker` 连接已启动的 Vue Picker。Vue 门需要 `--url` 和 `--browser-mcp-config`。日常入口只驱动已准备的工具和服务。

091 原有的 AssetPicker 和 Gallery 端到端回归由 `tests/product/verify_asset_picker.py` 执行。该集成入口生成并构建隔离后端，再执行原有产品矩阵。可通过 `--scenario reopen` 检查已准备 Vue 消费者的退出重开，通过 `--scenario failure` 检查独立示例的失败清理。
