# CollectionView 与 Thumbnail

来源为 PLAN-109 r2。两个组件由 `packages/collection-view` 提供，共享同一份 AutoUI 源码。

## 受控集合

应用提供已排序的 `items`、`columns`、`viewMode`、`selectedId`、`scopeId` 和 `revision`。可通过 `status`、`emptyLabel`、`errorMessage` 与 `disabled` 表达当前可用性。列表表头展示 `sortKey` 和 `sortDirection`。集合内部保留键盘焦点，每个条目使用当前结果中唯一、非空的 `id`，由 `itemRevision` 区分内容版本。

条目携带 `label`、`kind`、`icon`、`thumbnailUri`、`thumbnailState`、`cells`、`actions`、`defaultActionId` 与 `disabled`。列由 `key`、`label`、`align`、`sortable` 描述，其中名称列总是显示。条目的 cells 按列 key 取显示文字。重复或空条目 ID、重复列或动作 ID、未知默认动作及非法模式显示配置错误。

`viewModeRequested`、`selectionRequested`、`itemActionRequested`、`sortRequested` 发送 JSON 字符串。所有请求包含 `seq`、`instanceToken`、`scopeId` 与 `baseRevision`，条目请求另带 `id` 与 `itemRevision`。普通选中、预览、打开、菜单和排序仍由应用执行业务校验。当前结果身份、位置或条目内容变化后的旧绑定不能发有效请求。显示与键盘操作共用相同结果、顺序和控件实例。

方向键与 Home/End 移动实际键盘焦点并跳过禁用条目。重排和模式切换保留仍可用的焦点条目。该条目退出当前结果时，焦点移到首个启用条目或集合容器。Space 请求选中有焦点的条目。Enter 执行可用的默认动作，未配置默认动作时请求选择。Tab 依次到达主条目和可用附加按钮。原生与 Vue 都通过有焦点的按钮派发这些键。同屏多个集合和分类树分别保留自己的事件接收方。

## 缩略图

`Thumbnail` 接收 `src`、`alt`、`kind`、`icon`、`state`、`fit`、`resourceId` 与 `resourceRevision`。支持 `missing`、`loading`、`ready` 和 `error`。应用发布 ready 时，图片仍须经渲染器加载。缺图显示类型图标，加载失败有可观察的占位。更换资源或来源时替换图像绑定。原生媒体票据由 ImageSurface 消费，原生 Gallery 的授权 HTTP 缩略图复用现有 HTTP 读取链，读取或解码失败投递给当前缩略图实例。Vue 消费应用已授权的 URI。原尺寸查看、缩放和平移沿用 ImageViewport。

## 消费与验收

FileManager 持有目录、排序、真实路径、文件权限和缩略图生产。AssetBrowser 保留树、查询、分页、选择与 ImageViewport，沿用 AssetPicker 的确认和权限合同。Windows native VM 与 Windows Vue 分别验证业务读回，Linux 验收计划见 [DEBT-011](../../plans/debts.md#debt-011-collectionview-的-linux-平台验收)。
