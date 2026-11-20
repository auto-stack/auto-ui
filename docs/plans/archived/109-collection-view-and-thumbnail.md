---
plan_id: PLAN-109
status: archived
completion_kind: delivered
feature_name: CollectionView 与 Thumbnail 共用资源视图
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 2
current_step: 9
total_steps: 10
supersedes_spec_components: [docs/specs/ui/asset-browser.md, docs/specs/runtime/component-and-store-instances.md]
new_spec_components: [docs/specs/ui/collection-view.md]
touched_goals: []
---

# PLAN-109 CollectionView 与 Thumbnail 共用资源视图

## 0. 变更摘要

将 AssetBrowser 和 027 FileManager 各自的卡片、列表行、缩略图及条目交互收成同一个正式包 `packages/collection-view/`。公共组件名为 **CollectionView** 和 **Thumbnail**，两个组件可单独导入，CollectionView 默认组合 Thumbnail。首批消费者必须实际替换旧视图。路线对应 CL-27。见[组件库 strategy](../../design/autoui/component-library-extension-strategy.md)。

## 1. 目标

同一份当前结果可在网格与列表之间切换，保留有效选择并按稳定条目 ID 操作。文件管理器继续提供可排序明细列、目录进入、原有打开和菜单动作。AssetBrowser 继续提供分类、搜索、分页、预览、资源选择确认。缩略图统一显示图片、等待占位或类型图标，完整图片查看沿 089。

使用同一套组件实现覆盖原生 VM 和 Vue。本轮交付验证 Windows 两轨，Linux 两轨随统一 Linux 开发补验（DEBT-011）。普通键鼠与 MCP 均可完成核心动作，无人值守测试验证业务读回、失败和退出重开。首版范围为现有单选与当前结果集合。多选、拖放、虚拟化、任意格式解码和缩略图服务属于后续范围。

## 2. 架构方案

应用或 provider 提供已排序的结果及选择 → CollectionView 显示网格或列表 → 通过普通消息请求选择、打开、预览、菜单或排序 → 应用校验并处理。CollectionView 持局部键盘焦点。Thumbnail 接收上游提供的绘制来源和状态，复用既有图像渲染与类型图标。应用持文件系统、查询、分片、缩略图生产和缓存、资源权限及完整预览。

AssetBrowser 在自身层组合 TreeView、PaginationControl、ImageViewport / ImagePreviewDialog 和 CollectionView。FileManager 将当前列表与网格交给 CollectionView，保留快捷访问侧栏和现有滚动目录行为。目录树与分页仍可在应用层组合，本次 FileManager 迁移的验收范围是网格、列表和缩略图。

## 3. 技术栈

沿用 `.at` 正式包、现有主题、Grid/Row、Button、图像原语和普通事件。原生继续使用现有 ImageSurface 与缩略图 broker，Vue 使用既有浏览器图片渲染及原应用授权来源。T-01 对照两端实测选择共享渲染接缝和最小适配。包关系保持 CollectionView → Thumbnail，两个消费者 → 正式包。

## 4. 需求分析与背景调查

2026-11-20 用户要求为两个组件建立一份计划、迁移 AssetBrowser 和 FileManager、说明 091 衔接和跨平台测试、更新 strategy。当前授权范围为 auto-ui 规划文档。实施合同限本仓组件、示例、必要接缝及定向测试。用户没有指定模型、预算、并行执行或跨仓写入。

观察基线为 auto-ui main `448ed614f388c1ee16c68d2ac7cd0da46c73d4bd`。已核对[规范总览](../../specs/README.md)、[原生验收](../../specs/runtime/native-vm-app-validation.md)、[树](../../specs/ui/tree-view.md)、[分页](../../specs/ui/pagination.md)和[图片视口](../../specs/ui/image-viewport.md)。当前缺共用资源视图规范。

| 来源 | 观察与复用边界 |
|---|---|
| `examples/ui/027-file-manager/src/front/app.at` | SHA256 `6306d7d624aba15ded6f2b990eea11fbeadbf362a634185f570a8e637de1f7b8`。列表含名称、大小、类型、日期排序，网格与列表均接 SelectItem/OpenItem/ItemCtx。当前数字 ID 来自视图位置。迁移须由应用把稳定 ID 解析回当前文件。 |
| `examples/ui/027-file-manager/src/back/api.at`、`pac.at` | Vue 当前走 Rust 后端的 fs_list 等读盘接口，列表项分支中 thumb_src 为空。原生对前 120 个候选范围内的图片调用 image.thumb(pp, 256)，通过 image_surface 渐进显示。保留应用现有后端、缩略图预算及 Vue 文件操作权限。 |
| [091 计划归档](091-asset-browser-and-picker.md)与其实施树 `D:/autostack/.wt/ui-091/auto-ui` | 观察提交 `be13de09`。读取的局部工作文件 `packages/asset-browser/asset_browser.at` 含未提交变动，SHA256 `ccbcb8a9d25bff05ae59b0a0d692cb82342239fbb76122a118b055b48e78fd5f`，仅作设计证据。CardSelect/CardPreview、网格/列表、KindIcon、thumbnailUri 直接在 Browser 内，树/分页/视口已引用正式组件。 |
| `crates/auto-ui-runtime/src/ui/iced/broker_surface.rs`、`packages/image-viewer/` | broker 有现成的缩略图命中、缺图及图标回退路径。图片打开后的缩放/平移仍沿 089。T-01 验证普通包内的实际图像状态和回调，区分 URI 可用与像素已显示。 |
| `tests/product/verify_pagination.py`、`verify_image_preview.py`、`verify_file_dialog.py` | 复用本轮进程/端口、原生 MCP、实际浏览器、截图及重开设施。FileDialog 专用就绪和环境变量按该应用原样保留。迁移后也定向复跑 091 交付的产品门。 |

2026-11-20 `/goal 完成计划109` 授权在约定的 auto-ui 实施树执行、独立复审、合同内修复及落主归档。实施基线 `15fa7857`，已包含 091 交付 `c4588a88` 和 canonical Spec。091 收据中 Linux native 为 `not_run`。2026-11-20 用户授权在本会话处理原生嵌套回调运行时缺口，修复后继续 109。用户同时批准 Linux 验收随统一 Linux 开发补跑并登记为债务。Windows native 与 Windows Vue 的必验门继续执行。

### 与 091 的衔接

091 负责首版 AssetBrowser / AssetPicker 和 029 Gallery 迁移。其 §9 最新记录为 T-05 部分完成，原生 VM 投递缺口仍阻塞部分验收，Vue 尚待完整回放。091 维持自身计划和验收，**091 → 109 的 AssetBrowser 接入**为单向依赖。109 的公共包和 FileManager 可先做，T-05 及最后的双消费者验收须消费 091 已评审落主的提交、包与 Spec。T-01 记录 091 历史探针和交付 pin 的核验方式。依赖落主后从当前主线重核接口和现有回归，沿最终 091 源码迁移。若 107 同时修改 029 或共享运行路径，先核对实际写面与版本。

091 记录的 `sent>0, delivered=0` 和包子件事件转发问题进入 T-01 探针。以当前版本验证 CollectionView → 应用以及 CollectionView → AssetBrowser → AssetPicker → 应用的实际投递。T-01 复现原生回调到达中间件而不再上行；本计划的独立 T-10 修复当前运行时的这条链，保留 041 实例归属、挂卸和现有零/单参数合同。受影响的任务以正常链路恢复为完成条件。回放可定位故障，首次已受理但未完成业务动作的运行仍记失败。

## 5. 详细设计

### 公共接口

| 面 | 首版合同 |
|---|---|
| CollectionView props | `items:List=[]`、`viewMode:str="grid"`（grid/list）、`selectedId:str=""`、`scopeId:str=""`、`revision:int=0`、`disabled:bool=false`、`status:str="ready"`（loading/ready/error）、`emptyLabel/errorMessage:str`。应用控制模式与选择，控件提供具名网格/列表切换。 |
| 条目 | `{id,label,itemRevision,kind,icon,thumbnailUri,thumbnailState,cells,actions,defaultActionId,disabled}`。id 在实例当前结果中唯一且非空，itemRevision 随内容身份变动。cells 为 `{key,text}` 列表。actions 为 `{id,label,enabled}` 列表，条目可配置 open、preview、menu 等普通动作。defaultActionId 默认空，非空必须引用该条目已有的动作。未指定缩略图状态时，URI 有值视为 ready，空值视为 missing。 |
| 列表表头 | `columns:List=[]`，元素为 `{key,label,align,sortable}`。名称列始终存在，附加列从 cells 取可显示文本。受控 `sortKey/sortDirection` 展示当前排序，点击仅发请求。文件大小、日期及本地化由应用提供。 |
| CollectionView 消息 | `viewModeRequested(str)`、`selectionRequested(str)`、`itemActionRequested(str)`、`sortRequested(str)`。全部为 JSON 字符串，携 `seq,instanceToken,scopeId,baseRevision`。条目请求另带 `id,itemRevision`，动作另带 `actionId`。模式请求带 viewMode，排序请求带 sortKey，由应用更新顺序并回传方向。 |
| Thumbnail props | `src:str=""`、`alt:str=""`、`kind/icon:str`、`state:str="missing"`（missing/loading/ready/error）、`fit:str="cover"`（cover/contain）、`resourceId/resourceRevision:str`。上游 state=ready 表示提供了可尝试的来源。加载中、缺图或渲染失败显示对应占位，失败可观察，并保留可读替代文本。图像部分为非交互内容。 |
| 无效输入 | 当前集合重复或空 ID、重复列或动作 ID、非法视图模式显示具名配置错误，拒绝歧义动作。selectedId 不在当前结果时无行选中，外部选择不会被组件改写。缩略图数据错误只影响该缩略图，合法条目仍可选和打开。 |

```json
{"seq":3,"instanceToken":"app-1/mount-12","scopeId":"directory-2","baseRevision":6,"id":"file-8","itemRevision":"4","actionId":"preview"}
```

### 行为和所有权

- 网格和列表读取同一顺序与稳定条目身份。模式切换保留受控选中项及仍可见的焦点，焦点移除时落首个启用项或集合容器。列表用上下键，网格用左右键和按当前列数的上下键，Home/End 到首尾，边界保持，跳过禁用项。Space 请求选择。Enter 执行已配置的默认动作，未配置时请求选择。Tab 可进入条目附加动作并离开集合。
- 主行或卡片、打开、预览和菜单独立命中。单击选择。已配置且可用的默认动作由 Enter 或双击触发，FileManager 将它设为 open。Enter 在无默认动作时请求选择，显式禁用的动作保持禁用。右键和具名菜单按钮走同一 menu 请求。每次手势至多一次对应动作，预览或菜单点击零隐式选择。排序和视图切换各走对应请求。模式、焦点、选中、禁用及可见动作按实例和名称可发现，同名条目由当前条目身份区分。
- 动作发送与消费前核对真实挂载实例、scopeId、revision、条目 ID 和 itemRevision。换目录/查询/分页、列表重排、同 ID 换内容、卸载重建后旧绑定不得操作新对象。同 revision 内容变化也重验相关绑定，普通重绘保留可用性。FileManager 用当前目录及真实路径映射应用内稳定 ID，最终调用旧业务 handler 时重新解析当前目标，禁止直接传旧行号。
- Thumbnail 复用现有图像加载和类型图标，旧回调、旧图像或缓存回灌必须匹配当前实例、resourceId、resourceRevision 和 src。换资源先离开旧像素状态。可统一图片框和占位，缩略图生产、解码、缓存与请求数量由原服务持有。浏览器只使用应用已授权的可访问 URI，源不可用时保留类型占位。
- AssetBrowser 薄适配现有 records 和 query/result 身份，保留外部 props、事件载荷、跨页选择摘要、确认锁和权限。预览动作走 091 的既有处理链及 089 组件，沿用最终 091 提供的预览源语义。卡片缩略图显示状态与是否有授权预览源分别判断。029 Gallery 作为已有消费者做回归。
- FileManager 保留明细列及排序、目录导航、选中、现有菜单、操作入口和视图模式持久化。原生真实图片显示沿 image.thumb，Vue 真实目录无授权缩略图时显示类型图标。有可访问 URI 的标准 Thumbnail 样本由独立消费示例验证，保留与文件管理器真实磁盘能力的区别。
- 样式复用 091 左下区域参考、027 当前明细表布局和现有主题。网格随可用宽度减列，列表有清晰表头和可达的操作区。长中文、浅/深色和 420×560 小窗以真实截图和操作核验。见[样式指南](../../design/autoui/common-components-execution-and-visual-guide.md)和[既有参考图](../../design/autoui/assets/common-components-087-093-reference-v1.png)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/collection-view.md | 分散视图 → 正式 CollectionView/Thumbnail 的受控接口、身份、显示与动作 | 两个应用共用 | AC-01,AC-02,AC-03 |
| SD-02 | modify | docs/specs/ui/asset-browser.md | 091 交付后核对原规范 → AssetBrowser 组合共享条目视图，原公开合同继续生效 | 防重复与保兼容 | AC-04 |
| SD-03 | add | docs/specs/ui/collection-view.md | 两消费路径分别验收 → 单源、平台、MCP、无人值守和缩略图能力边界 | 可验证迁移 | AC-04,AC-05,AC-06,AC-07 |
| SD-04 | modify | docs/specs/runtime/component-and-store-instances.md | 原生直接调用父 handler 仅消费一跳 → 父 handler 明确重发的事件沿实际父组件实例继续派发 | 恢复包内多级受控组件挂载 | AC-04,AC-06 |

拟实施树 `D:/autostack/.wt/ui-109/auto-ui`，分支 `codex/ui-109-collection-view-thumbnail`。实现与独立复审使用同一树。T-01 用小夹具测含同名项的网格/列表、缺图→真图→坏图、普通动作到应用读回及嵌套事件接线。记录哪些路径当前可用、哪些需最小接缝或等待 091 pin。交接产物为新增 `docs/reviews/plan109/decisions.md`。

## 6. 测试设计

以下均为计划用例。T-06 新增 `crates/auto-ui-tools/tests/collection_view_component.rs` 和 `tests/product/verify_collection_view.py`。应用结果通过普通 UI 操作及实际页面、存储或后端读取交叉验证。

| Case / AC | 入口与用例 | 区分性断言及执行归属 |
|---|---|---|
| V-01 / AC-01 | 原生导入和 Vue 生成，移到第二绝对根并实际运行。另造缺包负控 | 仅一套公共实现可解析且实际收到事件。缺包具名失败。T-01/03/06 |
| V-02 / AC-02 | 普通示例，重名 A/B、禁用项、列表/网格切换及键盘、排序 | 选中仍为同一 ID。焦点顺序与列数符合合同，按钮不双触发。请求一次，应用确认前数据不变。T-03/06 |
| V-03 / AC-02,03,06 | 普通示例，双实例、刷新/重排、同 ID 换版本及卸载重开 | 旧动作零业务更改，当前动作正常。旧 A 图延迟返回时，B 的像素、占位和状态保持正确。用不同颜色源验证。T-02/03/06 |
| V-04 / AC-03,05 | 027 隔离目录，真实图片、目录、普通文件及损坏图片 | 原生有真实缩略图像素，空/坏图均可观察占位且仍能选择。Vue 读真实目录并尊重现有来源能力，标准示例的可用 URI 显示真实像素。元数据、排序、菜单目标、模式重开读回与原应用一致。T-04/06 |
| V-05 / AC-04,06 | 091 Picker 和 029 Gallery。分类、过滤、尾页、选中→预览→关闭→确认、取消/权限失效、Gallery 收藏读回 | 继续走正式树、分页面与视口。预览零选中变更。确认只写一次正确 ID/版本，拒绝或取消零写入。预览源和生命周期遵循 091/089。保留既有 091 用例及结果基线。T-05/06 |
| V-06 / AC-05,06 | FileManager 用户入口。选择后排序/过滤、同名项、进入目录、菜单和原打开动作 | 当前真实目标路径正确，旧节点拒绝，真实目录读回准确。原打开动作按实际宿主能力验证，不以系统关联程序作为跨平台成功依据。T-04/06 |
| V-07 / AC-06,07 | Windows 两轨逐门运行 core 和两个消费者，浅深色、420×560、长中文及错误态 | 操作后真实值正确，截图中缩略图/占位、焦点和文字可读。窗口、非空树/真实 DOM、动作、退出和重启收据完整。T-06/07 |
| V-08 / AC-06,07 | 多输入与嵌套层。事件漏投递、已退出端口、启动或动作失败的对照 | 动作 ACK 但业务值未变判 fail。真实 MCP 方法在关闭端口有界拒绝，配当前成功正例。失败保全最后证据并清理自有进程和端口。T-01/06/10 |
| V-09 / AC-04,06 | 原生四层包与一个直接绑定对照；同型双实例、未绑定及过期实例 | 当前接收者的中间态与应用最终值均读回一次。另一实例、未绑定父层及已卸载的旧绑定零写入。T-10。 |

拟议入口，由 T-06 实现和记录实际命令。每条跑独立证据目录。

```text
cargo test -p auto-ui-tools --test collection_view_component
python tests/product/verify_collection_view.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <dir> --backend native
python tests/product/verify_collection_view.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <dir> --backend vue --browser-mcp-config <config> --url <prepared-url>
```

- **本轮平台矩阵**为 Windows native VM 和 Windows Vue。每格分别记录组件与两个消费者。native 走 `auto run -r vm` 且保留实际声明后端。原生记录实际可见窗口归属与非空树，Vue 记录实际生成页、浏览器会话及其服务归属。Linux native VM 与 Linux Vue 在 DEBT-011 中登记为 `not_run`，沿自制发行版与 VMware 路线补跑。macOS 为后续独立验证项。
- **MCP** 以当前 snapshot/可访问树发现节点，用普通 action/keyboard 操作模式切换、选中、打开或预览、排序或菜单及实际适用的树/分页。native 使用现有 autoui 工具，Vue 用现有可操作真实页面的浏览器 MCP 提供方留证。Playwright 可跑普通浏览器回归，另留实际 MCP 调用和效果。键盘语义经真实焦点路径验证，不能只发送组件消息名。禁止直接改组件状态、调用应用 handler 或以硬编码 ID 和坐标替代当前节点发现。Windows 原生系统语义按适用的 [系统可访问性规范](../../specs/runtime/native-system-accessibility.md) 单列。
- **无人值守** 由脚本启动隔离的真实应用、临时目录/资源库、独立端口和浏览器配置。等待可观察就绪，有限期限执行，核对真实事件与业务读回。正常退出检查自有进程与端口释放，启动新会话完成一次新动作，再读回持久化模式或实际业务值。失败时保全日志、最后树与截图，在 finally 清理本轮进程和目录。缺平台、真实 MCP 提供方、必要来源或必需用例记 not_run 或 fail，相应验收保持开放。
- **证据** 每格保存 report.json，含计划版本、源码与依赖、CLI/工具摘要、具体命令、窗口或页面身份、case 预期与实测、图/树和读回摘要、清理重开、退出码。0 仅用于本轮必需门全部通过，失败与缺能力分别报告。Q-01/DEBT-006 以真实截图验证，文字和占位不能靠 snapshot 文本替代。双消费者迁移后的既有门按受影响面复跑。

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 正式包可单独导入两组件。第二绝对根生成、运行及缺依赖负控通过。V-01。 |
| AC-02 | work | 网格和列表共用 ID、受控选择与动作，键盘/鼠标行为正确，旧动作和重名负控通过。V-02/03。 |
| AC-03 | work | 真实缩略图及等待、空、坏图占位可观察，过期图像和回调不能污染当前项。V-03/04。 |
| AC-04 | work | 091 落主版本实际导入共用包并替换重复条目视图，Picker 和 029 原公开行为、实际确认及 Gallery 回归通过。V-05。 |
| AC-05 | work | 027 实际导入共用包替换两视图，现有目录、列、排序、菜单、权限与缩略图走真实路径。V-04/06。 |
| AC-06 | work | Windows 原生与 Vue 分别完成普通事件及实际 MCP 读回，关键正负控和原生无前台依赖验证通过。Linux 两轨记录 DEBT-011。V-03/05/06/07/08/09。 |
| AC-07 | work | 无人值守启动/失败清理/重开、移根、截图及用例证据齐。包 README 有真实用法和依赖闭包，Spec 提案可核对。V-01/07/08。 |
| AC-08 | review | 独立复审核对 AC-01…07、SD-01…04 和原 091/027 行为，绑定实际受评实现及依赖摘要，必需门都有结论。 |
| AC-09 | merge | 评审通过后核验实际交付、规范沉淀、路线与归档链接、版本和清理收据。 |

## 8. 执行步骤

若当前执行环境提供原生任务列表，实施前登记以下任务 ID。

- [x] T-01 验证包、图像和事件接缝
  - owner_stage: work。无组件迁移前置。复核 §4 版本、Grid 和图片原语、包注册及双生成器。通过 §5 小夹具和 V-01/08 核验读回，写 decisions.md 并标明 091 后续 pin 与 runner。AC-01/06。
- [x] T-10 修复原生多层回调转发
  - owner_stage: work。依赖 T-01 复现证据；`DynamicComponent::dispatch_parent_route` 已调用父 handler，父 handler 写入的 root emit 槽停留到后续输入。修复同一次交互内沿实际父实例消费明确转发，防止跨实例、卸载后转发和无绑定重放。定向复跑 V-09 与 041/085 受影响回调门。AC-04/06、SD-04。
- [x] T-02 实现 Thumbnail
  - owner_stage: work。依赖 T-01。新增 `packages/collection-view/thumbnail.at`，复用原图像管线和类型图标，明确过期图像和错误表现。V-03/04，AC-03。
- [x] T-03 实现 CollectionView
  - owner_stage: work。依赖 T-02。新增同包 `collection_view.at/package.at/pac.at` 和 `examples/collection-view/`，验证网格、列、选择、键盘、菜单与受控事件。V-01/02/03，AC-01/02/03。
- [x] T-04 迁移 FileManager
  - owner_stage: work。依赖 T-03。修改 027 的 app.at、pac.at 及必要薄适配。用目录、真实图片和排序/菜单回归验证原能力。V-04/06，AC-05。
- [x] T-05 迁移 AssetBrowser
  - owner_stage: work。依赖 T-03、T-10 和 091 已评审落主的实际版本。修改 `packages/asset-browser/` 与必要消费适配，替换重复视图。用 091 原入口和 029 回归核验。V-05，AC-04。
- [x] T-06 验证 Windows 两轨与两个消费者
  - owner_stage: work。依赖 T-04/T-05。实现 §6 新测试入口，按本轮两轨运行 V-01…08、必要旧门和失败清理，登记 Linux 债务，输出证据。AC-01…07。
- [x] T-07 准备规范与交付文档
  - owner_stage: work。依赖 T-06。新增包 README、实施报告及规范增量提案，核验真实命令、包闭包、状态矩阵和证据完整性。AC-07。
- [x] T-08 独立复审
  - owner_stage: review。依赖 T-01…07。在同一实施树核对提交、必需用例及规范增量，记录逐项结论。AC-08。
- [ ] T-09 合并与归档
  - owner_stage: merge。依赖 T-08 pass。按复审版本落主，沉淀规范，更新 strategy/交付 pin 和归档，核验实际清理。AC-09。

## 9. 复审记录

2026-11-20。stage: new，PLAN-109 r1，outcome: pass，next: work/T-01。新增 T-01…09、AC-01…09、SD-01…03。通过计划结构、源路径和参考图引用自检。初始进度 0/9，产品用例及实施待执行。

2026-11-20 执行启动。PLAN-109 r1，T-01 进行中。唯一实施树 `D:/autostack/.wt/ui-109/auto-ui`，分支 `codex/ui-109-collection-view-thumbnail`，基线 `15fa7857`。

2026-11-20。stage: new | plan_id: PLAN-109 | plan_revision: r2 | outcome: pass | next: work。用户批准 T-10 与 Linux 补验延期。修订 AC-06、V-07、T-05/06，增加 V-09、T-10、SD-04，保留 Windows 两轨及两个实际消费者的现有判据。原生复现以新构建工具验证，`Inner.changed → Middle.Forward` 执行后应用读回仍为 `waiting`。

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | checkpoint: T-01/T-10 | worktree: D:/autostack/.wt/ui-109/auto-ui | branch: codex/ui-109-collection-view-thumbnail | base_commit: 15fa7857 | code_commit: 5c9df17c | outcome: pass（仅 T-01/T-10） | next: T-02…T-07。T-01 的独立包两端生成、第二绝对根和缺包负控通过。T-10 正式原生窗从重名 B 条目按压，实际三跳后 App 文本 B，旧版本停在 Middle。

| 本轮范围 | 结果 | 耗时 |
|---|---|---|
| `collection_view_component` | 2/2。独立生成 0.901 s；移根生成、共享 Thumbnail 与缺包拒绝 1.402 s（初轮逐例）；更新后联合 2.54 s。 | 准备 11.319 s；后续 24.110 s |
| `plan109_nested_emit` | 1/1。原生真实实例绑定，另一路、未注册及释放后零写入；当前链与直接绑定各一次。 | 用例 0.068 s，进程墙时 0.355 s |
| `plan041_instance_ownership`、`plan085_routed_callbacks` | 4/4、2/2。 | 进程墙时 0.312 s、0.556 s；三个 Rust 目标统一预编译 96.853 s |
| 独立产品门 Windows native VM | 通过，两个不同可见窗口。MCP 选择 b、切列表保留；换版本后旧节点动作被拒绝；正常退出端口拒绝后重开再选择 a。 | 9.936 s，`core-native-final/report.json` |
| 独立产品门 Windows Vue | 通过，Playwright MCP 0.0.86，420×560。同名第二项 b → 列表保留 → ArrowUp + Space 选择 a，禁用项可发现。 | 3.172 s，`core-vue-final/report.json` |
| 027 Windows 两轨 | Native 在真实隔离目录经 MCP 选择 notes.txt 后切网格，读回仍为 notes.txt，截图已保存。Vue 用 MCP 按名称排序后样本顺序反转，切网格保留 notes.txt，双击子目录读回 inside.txt。 | Native 探针含启动 15.238 s；Vue MCP 9.293 s，文件 SHA256 FE727A2FA624F6FA… |
| 091/029 Windows 既有产品门 | Vue Picker 5/5；Native 15/18，分类阶段 kind 仍为 image，深色选中与 pending 回灌两项失败。确认接受、业务附件落账、029 真实扫描/查看器/收藏/目录 4 项通过。 | Vue 91.6 s；Native 296.2 s。SHA256 834D25BF94CE7AAA…、155CA6F79891ADD7… |

交付候选 `d9994d5d` 已提交到实施分支，下列产品证据使用该分支的源文件。Windows native Picker 复验以真实媒体票据预览 Lake.jpg，关闭预览保留选择和空附件日志，之后选择并确认得到 accepted。动作段 3.381 s，原始动作及树位于 `picker-109-preview-local/events.log`、`preview-tree.txt` 和 `accepted-tree.txt`。027 Native 的实际排序、菜单目标保留选中和进入子目录读回补充记录见 `t01/file-manager/native-menu-result.txt`。

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | checkpoint: T-04/T-05 | code_commit: 2fe2e595 | outcome: pass（迁移） | next: 完成 T-02/T-03 场景复验与 T-06/T-07。027 将启动专用计时器绑定到 `.booting`，复验后选择、排序、菜单目标及真实子目录均生效。091 产品门补齐了旧脚本漏判的确认重入、资源失效、取消与窄窗用例，30/30 全部完成；之前出现提前结束的报告保留供复核。

| 范围 | Windows 实测 | 时间与收据 |
|---|---|---|
| FileManager 原生 | 真实隔离目录，按名称排序后保留 notes.txt 选择，对另一行打开菜单后进入实际子目录并读到 inside.txt | 4.878 s，`t01/file-manager/prepared-native-startup-gated/report.json` SHA256 31A3E38299F0A96E88… |
| Picker 与 029 原生 | 30/30，通过真实扫描、预览、收藏、目录、尾页、权限失效拒绝、确认只写一次和取消零写入 | 140.6 s，`picker-native-final5/report.json` SHA256 5FE527844490F5F00C… |
| Picker Vue | 6/6，确认后真实附件日志增加；新缩略图用例核验 320px 实图解码、预览无业务变更与关闭后返回选择器 | 全程 50.5 s，新用例 1.94 s，`picker-vue-final6/report.json` SHA256 DB06B4CC70F149691F… |
| Gallery Vue | 真实浏览器 MCP，HTTP 目录扫描后选择 corner-marked，打开查看器，收藏夹仅包含目标，子目录读回 sub-photo | 动作段 5.899 s，`gallery-vue-final/inside.txt` SHA256 7B239A7848E1AA7B49… |
| Thumbnail Vue 迟到源 | 420×560，旧红色 HTTP 图在替换之后返回，当前条目中心仍为 RGB (5, 94, 244)，坏图占位可见 | 1.037 s，`t01/core/thumbnail-race/report.json` SHA256 6FBAD21B764D76467A… |

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | checkpoint: T-02/T-03 | outcome: pass。原生同 ID 换版本，坏图占位和新蓝图实绘对照通过。两份同型 CollectionView 使用相同条目 ID，右件操作仅更新右件，卸载旧节点拒绝，同位置重开可正常操作。原生前后台对照使用两个实际窗口，后台窗口完成选择，前台保持原状态。027 Vue 入口在等到应用首目录完成后再执行地址操作，重跑通过。

| 用例 | 时间 | 收据 |
|---|---|---|
| 原生换图和坏图 | 5.784 s | `t01/core/native-thumbnail-published/report.json` SHA256 CF5071769010E883CC…，新蓝图 120528 像素，旧红色 0 |
| 原生双实例、卸载、重挂载 | 2.269 s | `t01/core/dual-native/report.json` SHA256 0DAFDABB8CDA2DE03B… |
| 原生后台 MCP | 3.118 s | `t01/core/native-background/report.json`，记录首尾窗口句柄与两个应用读回 |
| 独立示例原生 / Vue | 6.442 s / 3.445 s | `core-native-media-published/report.json` SHA256 714CD871F7A333210A…；`core-vue-media-published/report.json` SHA256 9DC4D6CDA2C8983957… |
| 027 Vue | 8.89 s | `t01/file-manager/prepared-vue-wait-startup/report.json` SHA256 4A04C41C65D4DE8011…；真实后端目录请求和结果均已核对 |

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | checkpoint: T-03/T-06 | outcome: partial | next: 核验原生实际键盘焦点。原生用 MCP 选择 b 后切换列表，ArrowUp 与 Space 都返回 Key sent，应用读回仍为 b。原生现有全局按键表仅保存 widget 类型和 handler 名，集合的参数绑定未随事件送达当前实际焦点。Windows Vue 的实际按键已经通过。该项继续保留在 AC-02 范围内。

原生与 Vue 缩略图探针通过缺图、等待、坏图与换源恢复。独立 Vue 回放中错误图片已脱离 DOM。当前工作未获得 T-02…T-07 的整项验收，原生 Picker 红项与两消费路径的专项失败、重开、视觉证据仍需处理。证据在 `D:/autostack/plan-artifacts/plan-109/`。

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | outcome: execution_done | next: final review/T-08。实施树 `D:/autostack/.wt/ui-109/auto-ui`，分支 `codex/ui-109-collection-view-thumbnail`，`base_commit: 15fa7857a411707c44ceb6e1a37d1822808bef53`，`code_commit: 8daa99e46eda4b7f2e41d6e077877c4f00882731`。依赖 `auto-lang@94ac2d062254da202cc3d884051bbc78c2f6e94f`、`auto-web@26d085f7e54ea20c6997edf2dc23eef0cb57faf6`、`auto-down@72b097006f34db3a871449f10e213820d51abef6`、`auto-man@4a30483a590d81d5724cb453a0fd1fbc98d9aaf2`。CLI SHA-256 `3777610d73920312bcc2478d5fa38c52877ccf90442349de9489f294efee0b53`，工具 `d094e7ff73336f2ef3acda4bd4ee53d2a1c1a61b4ce724a281a93e6d179e382b`。原生工具按 `cargo build -p auto-ui-tools --bin auto-ui-tool --offline -j 5` 显式预编译，dev 增量、关闭调试符号，退出 0，21.46 s。运行时目标用 `cargo test -p auto-ui-runtime --lib --test image_surface_contract --test plan041_instance_ownership --test plan085_routed_callbacks --test plan085_keydown_ownership --test plan109_nested_emit --no-run --offline -j 5` 预编译，退出 0，42.05 s。生成用例用开发测试配置的已编译目标，读取冻结源码。原始报告相对 `D:/autostack/plan-artifacts/plan-109/`。

| AC / V | 实测路径与预期对照 | 结果与证据 |
|---|---|---|
| AC-01 / V-01 | Rust/Vue 生成；绝对移根及缺包具名拒绝。原生把包复制到第二应用根的 vendor 路径，以实际窗口和当前动作再次选中。 | `collection_view_component` 2/2，2.453 s 墙时；独立产品门原生 8.526 s，`core-native-delivery/report.json` SHA-256 19C757414CFB77788…；Vue 3.645 s，`core-vue-delivery/report.json` SHA-256 C53EE7A2C8F90453…。两轨退出 0。 |
| AC-02 / V-02/03 | 两轨均点击两个同名项中的 b，切列表保持 ID；原生实际焦点 ArrowRight 后状态不先变，Space 后才选 b；Vue Playwright MCP ArrowUp、Space 及 DOM 焦点验证。单独原生列表探针 End 跳过禁用项，Tab、Enter 激活附加按钮而保留选择。双实例右侧键只进右侧，重挂载后旧动作仍拒绝。 | 同上两轨；列表 6.882 s，`t01/core/native-key-direct/report.json` SHA-256 1561D63F7CAE471C…；双实例 5.889 s，`t01/core/dual-native-keys/report.json` SHA-256 CD0CDCFEA58757C6…。专项是实际原生控件焦点操作的 MCP 路径。 |
| AC-03 / V-03/04 | 原生坏字节显示占位，同 ID 换版本后像素从旧红换新蓝；Vue 将红图延迟到蓝图替换之后，实际截图中心仍是蓝色，错误 DOM 占位。 | 5.784 s，120528 蓝色像素、0 旧红像素，`t01/core/native-thumbnail-published/report.json` SHA-256 CF5071769010E883…；Vue 1.037 s，RGB (5,94,244)，`t01/core/thumbnail-race/report.json` SHA-256 6FBAD21B764D7646…。 |
| AC-04 / V-05 | 091 原生 30 项，含搜索 1 条、图像 5 条、文档 998 条、1003 项末页、过期结果与权限失效、确认只写一次、取消零写入及 029 真实扫描/查看器/收藏/目录。Vue 6 项，含实图解码、预览关回和确认落账；Vue Gallery 用实际浏览器 MCP 验证 HTTP 源与收藏、子目录。 | 原生 30/30，退出 0，共 253.8 s，业务预编译 107.5 s，`picker-native-delivery/report.json` SHA-256 1F59F8203CE154E0…。Vue 6/6，退出 0，共 136.8 s，预编译 114.5 s，`picker-vue-delivery/report.json` SHA-256 7104A900967C5190…。新 320px 缩略图/预览用例 1.910 s。Gallery Vue 动作 5.899 s，`gallery-vue-final/inside.txt`。 |
| AC-05 / V-04/06 | 027 读隔离真实目录，名称排序、保持 notes.txt、对另一项开菜单、进入子目录并读到 inside.txt。原生 420×560，正常关两次，重启后独立 PID 读回保存的网格模式。 | 原生 4.878 s、Vue 8.889 s，`t01/file-manager/prepared-native-startup-gated/report.json`、`t01/file-manager/prepared-vue-wait-startup/report.json`。模式动作 0.067 s、最终截图 0.855 s，须启动两次的手动诊断 22.165 s，`t01/file-manager/native-mode-reopen-fixture/report.json`。 |
| AC-06 / V-03/05/06/07/08/09 | 两个真实原生窗口中后台 MCP 修改第一窗，前台第二窗值无变化；Vue 使用浏览器和页面焦点。独立原生门遇到旧节点、旧端口各有有界拒绝，新会话可重复动作。四层的当前回调、同型另一实例、无绑定、释放和直接回调用例均有明确计数。 | 后台专项 3.118 s，`t01/core/native-background/report.json`。本轮原生回调 1/1、图像 1/1、041 ownership 4/4、085 routed callbacks 2/2、085 keydown 5/5，退出 0，目标墙时分别 0.101/0.563/0.145/0.407/0.192 s。原生 focus/modal 082 7/7、095 3/3、098 delivery 4/4、keyboard 4/4，共 3.04 s 用例时。Linux native VM/Vue 均 `not_run`，按用户裁定登记 DEBT-011。 |
| AC-07 / V-01/07/08 | 正式包清单、可复制入口、AssetBrowser 五包依赖闭包，三份规范草案已提交。独立门报告保留窗口、浏览器、旧节点、干净退出、重开信息。 | `asset_browser_component` 2/2，7.923 s 墙时，退出 0；README、规范提案及附录可逐项比对。 |

原生筛选初次复跑 `picker-native-key-final/report.json` 为 28/30，搜索条件仍是 Asset-050，造成本应主动清空后的两步错误。用原 provider 的 text/kind/folder/total 共同断言复位后，两项同轮 4.44 s 和 1.45 s 实测通过。最终报告中的旧文字仍写 21 页，实际断言一直校验当前 25 条/页对应的 41 页，冻结提交把收据说明同步为 41。此前原生组件组合门 10.89 s 保留，随后在已正常退出的旧连接上并行等待有界拒绝及独立再启动，冻结运行 8.526 s。既有 091 首次挂载计时包含外层预编译 107.5 s，深色选中旧例 10.35 s，Vue 入口旧例 15.10 s，以上记录保留。新增门的 10 秒上限及受影响旧门的原样本均已核对。

模态专项重放在同一源码重编的原生工具（SHA-256 AAC7C70C188D586D…）上通过。MCP 点击 Lake 后，ArrowDown 等待渲染帧，再按 Space 选中 Corner。Tab/Enter 打开其预览并保持选择和附件数，4.18 s，`picker-native-modal-keys-final/report.json` SHA-256 F014AA7AD248B785…。早先立即按下 Space 的探针未选中 Corner，重放证实需要等待原生控件完成聚焦，试验改动已撤回。

2026-11-20。stage: review | review_scope: final | plan_id: PLAN-109 | plan_revision: r2 | reviewer: /root/review109 | reviewed_commit: 8daa99e46eda4b7f2e41d6e077877c4f00882731 | base_commit: 15fa7857a411707c44ceb6e1a37d1822808bef53 | worktree_path: D:/autostack/.wt/ui-109/auto-ui | branch: codex/ui-109-collection-view-thumbnail | outcome: needs_fix | next: work。复审检查四个依赖 pin、工作树、091 交付祖先，`git diff --check` 通过。AC-01 pass；AC-02/03/07/08 fail；AC-04/05/06 partial；AC-09 待合并阶段。独立复测仅运行两个窄探针。Vue 1.026 s，原生 HTTP 3.753 s，均保留失败。复审绑定的 Spec blob 为 CollectionView `274f2c0bd3638e3bc32cbfe586547ce4acc37003`、AssetBrowser `01f6e4e4b6570a03fa11fdb6b505c4b494ec2786`、组件实例 `ca9fd87387c4610979d8ba8020a12d9e7f3f8de7`。SD-04 的实际父实例转发通过；SD-01/03 的焦点、坏图和交付证据随修复复审；SD-02 对读公开接口和查询反馈成立。

| Finding / AC | 反例与关闭条件 |
|---|---|
| F109-R1 / AC-02 | Windows Vue Core，420×560。聚焦 b 后按程序调用名称按钮触发实际排序，顺序 a/b/c 变 b/a/c，选中仍是 b，焦点变 BODY，Space 不能进集合。修复后验证同一 b 保持实际焦点，移除 b 时回退首个启用项或集合容器，随后 Space 有合法读回。复现 `review109/focus-retention.cjs`。 |
| F109-R2 / AC-03/04 | Windows 原生授权 HTTP 两源都返回 200 image/png，一个有效、一个损坏。正常蓝色像素 90396；损坏 b 可选但无错误占位。旧 `image` 的实际解码失败须驱动当前键控 Thumbnail，保留成功像素、换源和 Gallery。复现 `review109/native-http.py`。 |
| F109-R3 / AC-07 | 091 原生 `session-0.log` 和 Vue 启动日志显示测试入口内生成并 Cargo 编译后端。原样本保留为显式集成构建回归；新增日常 Vue 用例移至已准备的浏览器 URL，准备缺失有具名失败且不得构建。 |
| F109-R4 / AC-06/07 | 预备的 Vue Core、027 没有第二会话的动作，Picker 旧报告 cleanup/reopen 为空；原生成功关停已有证据，失败支路仍须保全最后样本并验证自有进程端口释放。补证区分外部服务与会话所拥有的资源，时限按新用例计算。 |

Windows Vue Picker 已补同版 420×560 深色、浅色和错误态。默认 CSS 为 dark/222.2 47% 7%，使用已有 pac theme 配置准备浅色后实际 DOM 为 0 0% 100%。验证两者的确认按钮在视口内、选择摘要正确、错误封锁确认，截图在 `picker-vue-delivery-visual` 与 `picker-vue-delivery-light`。旧演示的“切换明暗”未改变 CSS 的观测值，未作为浅色依据。原生显式浅色具有 805018 个亮像素和 12982 个暗像素，额外重放的截图尺寸是物理像素 917×1181，未将其算作 420×560 通过。

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | outcome: execution_done | code_commit: 8a0e381d696eedfd430ee1a11decb653f2025c67 | repairs: F109-R1…R4 | next: affected final review。基线、依赖 pin、工作树和 CLI 沿用上述收据。原生工具 SHA-256 `00448dd3f320dcc456515d8348ba7fe70493f9c65bd718c55a6815796d2517f4`，dev 增量、offline，显式构建退出 0。预编译等候已经持有的准备服务退出后重试，启动器产生的锁文件差异按原基线恢复。此轮产品门通过的进程均使用隔离样本与工具版本。

| Finding | 修复及实际复验 |
|---|---|
| F109-R1 | 同一逻辑条目用稳定焦点 ID，与校验当前版本的动作载荷分开。Vue 模式切换和排序保持实际 b 焦点，移除 b 后焦点落到实际第一项 a，Space 读回 a，普通打开保持选中。原生 Grid 在子树更新时按稳定键保留实际控件焦点，右键进出和排序后的显式操作分开。复审原始 Vue 回放修复后顺序 b/a/c、焦点仍 b。常规 Vue 门含第二会话 8.146 s，`core-vue-repair2/report.json`。原生门 5.733 s，`core-native-repair1/report.json`。原生额外排序后 Enter 实际打开 b，仍选中 b，2.612 s，`review109/native-focus-repair/report.json`。 |
| F109-R2 | 授权 HTTP 路径保留原读取链，并以成功缓存或实际读取/解码失败驱动 ImageSurface 的当前绑定。独立复审同两个 HTTP 200 原样重放，命中两个源，有效图 90396 个精确蓝像素，坏图明确占位且 b 仍可选，3.537 s，`review109/native-http-final/report.json` SHA-256 A51C2246748FB6D5…。既有本地票据换版本与过期源控制沿用同一格路径复验，`image_surface_contract` 1/1 和 029 Gallery 真实回归通过。Vue Gallery MCP 打开查看器、收藏过滤和实际目录 4.917 s，`gallery-vue-repair-warm/inside.txt`。 |
| F109-R3 | `verify_asset_picker.py` 明确为 091 既有端到端集成构建门，保留 1003 项与 30/5 分母。新增预览门迁入 `verify_collection_view.py --consumer asset-picker --backend vue --url <prepared-url> --browser-mcp-config <config>`，真实浏览器 MCP 断言 320px 下载解码、预览与关闭均零选择/落账，实测 5.786 s，`picker-vue-daily-r1/report.json`。对缺失准备服务、实际 connection refused 的反例返回具名 preparation failure，进程退出 1，1.141 s，`picker-vue-cold-negative/report.json`。文件级审阅显示日常入口没有后端构建或回退命令。 |
| F109-R4 | 实际原生坏 action 后保全树和截图，已退出、无拥有窗口，随后旧端点连接拒绝，5.54 s，`core-native-failure-r2/report.json`。Vue 真实无效节点败、保全非空树和截图，拥有的浏览器会话退出，1.358 s，`core-vue-failure-final/report.json`。Core Vue 两次独立会话，第二次真实选 a。Picker 两次独立浏览器会话读回实际 1003 项，5.300 s，`picker-vue-reopen-r1/report.json`。027 Vue 使用准备服务，关闭原页面，在不同浏览器上下文载入保持的浏览器存储，网格模式仍生效，再实际读目录 notes.txt，7.671 s，`t01/file-manager/prepared-vue-batched-v5/reopen/report.json` SHA-256 41631A969640FA3C…。测试拥有的浏览器会话独立记录，准备服务由启动器负责清理。 |

027 原生 4.285 s，`t01/file-manager/prepared-native-review-repair/report.json`。027 Vue 主场景用真实普通点击、右键、双击，检查排序后的顺序，另一行菜单不改 notes.txt 选择，进入真实子目录读到 inside.txt，6.121 s，`t01/file-manager/prepared-vue-batched-v5/report.json` SHA-256 8B771923F8B4B3F1…。早期组合用例 10.145、16.497、11.055 s，重开 10.092、13.325 s，原始失败均保存。最终把同一场景的浏览器动作批在实际 MCP 会话内执行，去除重复树读取，重开用真实新上下文及原会话存储，保留原正负控和整体计时。

091 原生整轮 `picker-native-repair-final/report.json` 29/30，第二次确认时业务计数未增加，保留为失败。确认必须从当前树等待实际可用，再以同一节点连续派发两次。定向修复后原矩阵在同一实现上 13/13，`picker-native-matrix-enabled/report.json` SHA-256 F6AB2AEE7DCB562C…，计数仅增 1 且 session 2 得到接受。未受该用例准备修订影响的其余 17 项沿用同一版有效结果，含原生 Gallery 4 项。

定向 Rust 目标本轮退出 0。图像 1/1 墙时 0.448 s，041 ownership 4/4 0.121 s，085 keydown 5/5 0.145 s，085 回调 2/2 0.344 s，109 nested emit 1/1 0.122 s，082 模态与系统输入 7/7 1.957 s。双轨生成、移根与负控 2/2 1.825 s，AssetBrowser 依赖闭包 2/2 7.424 s。修复后的日常环境为 Windows，Linux 维持按 DEBT-011 延期的原裁定。

2026-11-20。stage: review | review_scope: final | reviewer: /root/review109 | plan_id: PLAN-109 | plan_revision: r2 | reviewed_commit: 8a0e381d696eedfd430ee1a11decb653f2025c67 | base_commit: 15fa7857a411707c44ceb6e1a37d1822808bef53 | outcome: needs_fix | next: work/F109-R2b。F109-R1、R3、R4 已关闭。AC-01/02/05/06/07 pass，AC-03/08 fail，AC-04 partial。四依赖 pin、实施树、基线及 `git diff --check` 复核通过。SD-01/03 blob `a50a545eed25e4a63598e638c0eb38cc6a06695a`，SD-02 `01f6e4e4b6570a03fa11fdb6b505c4b494ec2786`，SD-04 `ca9fd87387c4610979d8ba8020a12d9e7f3f8de7`。独立原生失败保全和旧端口拒绝通过，5.391 s。其余已关闭证据保持上述收据。

F109-R2b 反例。同一授权 URI 和资源 ID 发布 revision 2 后，原生和 Vue 均继续显示 revision 1 的红图。原生原地址 `/unchanged.png` 返回 `no-store`，实际 MCP 读回 revision 2 和新节点，服务只收到最初红图请求，重绘为 90396 红、0 蓝，6.313 s，退出 1，`review109/native-http-stable-url/report.json`。在生成的 Vue Thumbnail 中，`no-store` 和 `max-age=3600` 都只有初次请求，DOM 虽重挂载但中心仍是 (230,20,30)，1.781 s，退出 1，`review109/vue-http-stable-url-report.json`。闭环需复验原地址新像素，并保留原坏图、迟到不同来源、正常 HTTP 与 Galllery 路径。

2026-11-20。stage: work | plan_id: PLAN-109 | plan_revision: r2 | outcome: execution_done | code_commit: 27441ee91171caeea810239e5cd90787ed0a4ec2 | finding: F109-R2b | next: affected final review。两条独立反例将原因定位为实际取图缓存，而非组件 key 或当前版本的 VM 读回。原生按已解析 URI、资源 ID 与 revision 保留有界的解码结果，新版本绕开旧的 URI 字节缓存并保留原授权 HTTP 请求地址和客户端。Vue 以新绑定读取原地址，重新验证 HTTP 结果和解码缓存。可读响应使用本次请求的对象 URL，跨域透明图片复用标准图片权限，先重新验证同一个 HTTP 地址。释放绑定时取消旧读并释放对象 URL。原生媒体票据路径仍使用原有注册表。

| R2b 复验 | 原样输入、读回和收据 |
|---|---|
| Windows 原生同 URI | 同一 HTTP 端点先红后蓝，MCP 实际替换到 revision 2，GET 地址保持 `/unchanged.png`。之前 90396 红、0 蓝，之后 0 红、90396 蓝。5.840 s，退出 0，`review109/native-http-stable-url-versioned-v2/report.json` SHA-256 946A175E62C198E3…。 |
| Windows Vue 同 URI | 重放 `no-store` 和 `max-age=3600` 两种缓存政策。相同 URL，实际旧色 (230,20,30) 变为 (5,94,244)。透明跨域场景读到相同路径下第二版本。两种政策组合 2.380 s，退出 0，`review109/vue-http-stable-url-versioned-report.json` SHA-256 7FA024F2E73F936F…。 |
| 坏图与迟到源 | 原生实际有效图 90396 蓝，HTTP 200 的损坏图具名占位且 b 仍可选，3.612 s，`review109/native-http-final-versioned-v2/report.json` SHA-256 2B8413C01D96B85…。Vue 同 ID 的旧红请求先到服务，切新后晚回。新图始终 (5,94,244)，损坏图占位，2.227 s，`review109/vue-versioned-race-final.json` SHA-256 1AA75164BBE386C8…。 |
| 091 Vue 实际消费 | 已准备的真实同源 Picker，下载解码 320px，预览和关闭均零隐式选中、零附件写，7.433 s，`picker-vue-daily-versioned-v2/report.json` SHA-256 10CF762A1A83CC5B…。一次并行编译期间的 8.293 s 浏览器连接超时作为失败保留，待主要编译结束后原样复验通过。 |
| 029 两轨消费 | 原生既有集成 4/4，包含真实扫描、身份→查看器、收藏、目录，逐例 1.25/2.74/4.47/1.05 s，含准备的整段 18.728 s，`gallery-native-versioned/report.json`。Vue 真实缩略图解码、查看器关闭回网格、收藏只剩目标、子目录 sub-photo，6.391 s 动作，`gallery-vue-versioned-batched/report.json` SHA-256 060AFC7984FD6D46…。早期单步 MCP 的控件等待超时保留，后续用相同实际浏览器操作和可观察读回的有界序列复验通过。 |

原生工具 SHA-256 `26f4e3de36e2c2bbe06fd330dfc45a3148bb4ba4c682947d863444ddc8bd2b95`，`cargo build -p auto-ui-tools --bin auto-ui-tool --offline -j 5`，dev 增量，38.86 s，退出 0。CLI 摘要和四依赖 pin 保持前轮值。§9 未受本次缩略图传输变化影响的焦点、回调、目录、冷服务、失败清理和退出重开的独立复审收据继续适用。只重放关联的来源、坏图及两个实际消费入口，未重跑无关产品集合。SD-01/03 修订后的 CollectionView blob `3496240ce93b22a25b4f5bbc4d5fc71d5fe1c6e7`。

2026-11-20。stage: review | review_scope: final | reviewer: /root/review109 | plan_id: PLAN-109 | plan_revision: r2 | reviewed_commit: 27441ee91171caeea810239e5cd90787ed0a4ec2 | base_commit: 15fa7857a411707c44ceb6e1a37d1822808bef53 | worktree_path: D:/autostack/.wt/ui-109/auto-ui | branch: codex/ui-109-collection-view-thumbnail | outcome: pass | next: merge。AC-01…08 全通过，四个依赖、CLI、工具、评审树及 `git diff --check` 已核对。F109-R2b 关闭。原生同授权路径换版 5.840 s，坏图控制 3.612 s。Vue 同地址两种缓存政策 2.380 s，迟到旧图和坏图 2.227 s。独立复审增加带现有 URL 片段的跨域变体，两种缓存政策均显示新版蓝图，1.513 s 用例、2.047 s 工具墙时，`review109/vue-http-query-fragment-report.json` SHA-256 3103480E9AD46BBF…。首次夹具的截图名称出错，修正后原样本通过。Vue Picker 7.433 s，Gallery Vue 6.391 s 和原生既有 4/4 通过。只重放实际受影响路径，上一轮其它验收继续适用。Linux 两轨按用户授权记录为 DEBT-011。

| delta | 规范路径 | 受评 Git blob |
|---|---|---|
| SD-01、SD-03 | `docs/specs/ui/collection-view.md` | `3496240ce93b22a25b4f5bbc4d5fc71d5fe1c6e7` |
| SD-02 | `docs/specs/ui/asset-browser.md` | `01f6e4e4b6570a03fa11fdb6b505c4b494ec2786` |
| SD-04 | `docs/specs/runtime/component-and-store-instances.md` | `ca9fd87387c4610979d8ba8020a12d9e7f3f8de7` |

2026-11-20。stage: merge | checkpoint: prepared | plan_id: PLAN-109 | plan_revision: r2 | original_reviewed_commit: 27441ee91171caeea810239e5cd90787ed0a4ec2 | rebased_commit: 799ea463bc1540efe5691016dddf697d48cf1d4f | delivery_commit: a6b166278dbdf59f5e7a3ac65955e20d579fa2f7。主分支先交付 125，当前基线 `68ff80f0cfd45571d4d2a8098a718221708a137b`。变基无冲突，`git range-diff 15fa7857..27441ee9 68ff80f0..799ea463` 共 11/11 标为 `=`。最终补丁仅给 125 引入的 `View::Button` 初始项提供 `keyboard: None`，ACE 类型化键盘和按钮动作仍由 125 的路由处理。独立复核者 /root/review109 绑定最终提交判定合并兼容通过，两套包目录及三个规范 blob 与原最终复审一致。guard 通过，计划路线对应 CL-27。

定向复验用 dev/debug 增量和独立预编译。原生工具重编 30.93 s，运行时默认测试目标准备 39.16 s，均退出 0。原生 `plan082` 7/7 墙时 1.489 s，109 嵌套回调 1/1 墙时 0.238 s。显式 ACE feature 在 109 原 AutoLang pin 编译时缺两个 125 后才提供的 `auto_lang_ace_host` 接口，退出 101，原始日志 `merge-ace-bridge-prep.log`。ACE 兼容门改用 125 已交付的 AutoLang `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28`，109 原核验 pin 保留于上文。

2026-11-20。stage: merge | checkpoint: landed | plan_id: PLAN-109 | plan_revision: r2 | delivery_commit: a6b166278dbdf59f5e7a3ac65955e20d579fa2f7 | outcome: pass。AutoUI master 从 `68ff80f0cfd45571d4d2a8098a718221708a137b` 经 `git merge --ff-only codex/ui-109-collection-view-thumbnail` 快进至交付提交，实际 HEAD、已复核分支和预期交付相同。两次 `wt-guard` 及 `git diff --check` 通过。显式 ACE feature 采用 125 的已交付 pin 后准备 82 s，退出 0。运行 `ace_node_desktop::widgets` 4/4，覆盖分页 UIA、值/关闭会话、disclosure 关闭与输入、滑块边界，墙时 0.123 s，执行 0.03 s。109 的依赖树恢复为已复审 pin。master 上用重编原生工具 SHA-256 `13CD5EB87387E81323C42FC742166F1410668FED66AE3E79A592F18454CF3D22` 和本 Plan 产品入口重放实际窗口、当前条目、同名 identity、旧节点拒绝与退出重开，9.809 s，退出 0，`landed-native-core/report.json` SHA-256 4E6280692DEFCB49…。其余已审 Windows 范围沿用等价 11 个补丁和同一实际包源的独立证据。未扩大统一回归范围。

2026-11-20。stage: merge | checkpoint: canonical_verified | plan_id: PLAN-109 | plan_revision: r2 | delivery_commit: a6b166278dbdf59f5e7a3ac65955e20d579fa2f7 | outcome: pass。实际 master 三份 Spec 与已评审的 canonical blob 一致。CollectionView `3496240ce93b22a25b4f5bbc4d5fc71d5fe1c6e7`，AssetBrowser `01f6e4e4b6570a03fa11fdb6b505c4b494ec2786`，组件实例 `ca9fd87387c4610979d8ba8020a12d9e7f3f8de7`。正式包及两个使用者入口可读，DEBT-011 在 master。引用的 091 归档、组件指南与参考图均存在。

2026-11-20。stage: merge | checkpoint: archived | plan_id: PLAN-109 | plan_revision: r2 | delivery_commit: a6b166278dbdf59f5e7a3ac65955e20d579fa2f7 | outcome: delivered。实施和复审已完成，T-09 的清理待验证。归档文件为 `docs/plans/archived/109-collection-view-and-thumbnail.md`。CL-27、计划入口和 Linux DEBT-011 已连接到正式交付。

## 10. 待澄清事项

用户已裁定由本会话执行 T-10，Linux 两轨验收随统一 Linux 开发补跑并登记 DEBT-011。T-01 的其它接缝和 Windows 两轨实际入口沿 r1 合同核验。
