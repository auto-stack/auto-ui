# AutoUI 组件库扩展路线 strategy

> 版本：r6，2026-11-14。主责：auto-ui。084/086代码已交付归档；085及087–093仍drafting r2、0/8，094–097经再次复核为drafting r2、0/8。下一项优先085 TreeView；轻量交互批可独立从094开始。084的Linux验证/统一回归/DEBT-006仍显式待补，本轮只修计划/路线与引用，不启动实施。

以已有组件、示例和应用中的重复实现为起点，将常见交互收成正式组件。优先解决文件选择、树导航和分页的复用，让普通用户、MCP 和无人值守测试使用同一套可见控件。组件负责交互和状态合同，应用负责数据、文件写入、导入事务和业务权限。

## 1. 当前基线和文档分工

2026-11-13静态盘点中，schema 有 437 个 element 声明，包含组件族的子元素，不能解释成 437 个完整可用组件。`packages/` 当前正式资产有 `themes`、`file-dialog` 和086交付的 `pagination`。已有 Button、Input、Dialog、AlertDialog、Progress、Table/DataTable、Command、Calendar、ImageSurface、示例内ImageViewport、CodeEditor 等基础；登记、Vue 映射、原生转换、键盘交互和实际消费者必须分别核验。

- **当前已交合同**：以 [Specs](../../specs/README.md) 及相关源码为准；[FileDialog Spec](../../specs/ui/file-dialog.md) 已含打开文件、目录选择与保存目标模式，084交付7ea71f0c；Linux原生验证仍按归档收据待补，不据Spec存在声称四平台全门已通过。
- **未来方向和追踪**：本 strategy 持有候选 ID、优先级、计划关联及最新交付入口。它不替代 Spec，也不把“已计划”写成“已支持”。
- **执行合同**：各 Plan 持有唯一的任务、验收、进度和复审记录。本页只摘要阶段；冲突时先核对 Plan 和交付收据，再修正摘要，不另建一份执行任务表。
- **视觉依据**：[FileDialog 改进设计及实拍](file-dialog-ui-polish.md)。参考图只指导布局；交付依据为真实窗口、操作结果和对应版本截图。
- **原批次视觉入口**：[文件导航与轻量交互视觉指南](file-navigation-and-interaction-visual-guide.md)覆盖CL-01…09：084–086引用文件导航图，094–097复用已生成的四组件图，088/089复用既有图。四份新Plan的§5直接引用PNG及对应区域，§6/7绑定真实视觉/键盘/MCP与消费验收；参考图不是已实现证据。
- **通用扩展批交接**：[执行与视觉指南](common-components-execution-and-visual-guide.md)包含七组件GPT参考图、统一尺寸/状态及runner约定；[本轮计划复核](../../reviews/component-plans-087-093-2026-11-14/README.md)记录十项缺口与r2处理。r2从T-01启动，技术探针和依赖pin仍是后续实施前置。

首轮源码观察基线为 auto-ui `4dd64b5fc5fc0919bfe686115c75a5d8c7ac06b1`；2026-11-14新批复核HEAD为`93e6b924749a45c5c6c390725a4bb2d8a10bfc07`，具体输入与摘要在各Plan §4。工作目录还存在其他设计/计划/测试WIP，未将其算作本路线交付。FileDialog UI 改进已在祖先提交 `3219e7b`，Reader009 消费同步为 `ef00316`，不是 Reader 主检出已集成的证明。

本次094–097立项观察HEAD为 `a6f776c24a4ba98611849955edceb5a04edac969`；复用源及未提交文件摘要分别写入各Plan §4。Reader当前主检出仅作只读来源；新计划不跨仓修改Reader或同步vendor。

## 2. 路线和计划登记

“待立项”不承诺排期；“已立计划”不代表实施已启动。每个候选只设一个主要交互 owner；组合式对话框复用内部控件，避免新增同义实现。

| ID | 批次及组件 | 复用来源 | 首版边界与用途 | 当前阶段 / 合同 |
|---|---|---|---|---|
| CL-01 | 第一批 FileDialog 目录选择 / 保存模式 | `packages/file-dialog`、runtime `ui/file_dialog`、现有 Dialog 生命周期 | `singleDirectory` / `saveFile`；初始目录、默认文件名、扩展名、覆盖确认；实际写入仍归应用 | [PLAN-084归档](../../plans/archived/084-file-dialog-folder-and-save-modes.md)，r2、8/8，交付7ea71f0c；Linux（VMware）/统一回归/DEBT-006待补 |
| CL-02 | 第一批 TreeView | 018 Reader、026 Database、027 File Manager、041 Editor 的受控树及 `tree_util` | 正式树包，稳定节点 ID、展开/选择、键盘、受控懒加载状态；不带文件系统业务 | 已立 [PLAN-085](../../plans/085-official-tree-view-component.md)，drafting r2，0/8 |
| CL-03 | 第一批 Pagination | 现有 `pagination` 声明族、FileDialog 分页、026 手工分页 | `PaginationControl` 统一受控入口；页码/总量/每页数量、边界、紧凑布局、单页隐藏；保留现有内置组件族 | [PLAN-086归档](../../plans/archived/086-shared-pagination-control.md) r2，代码交付765251b9；[Spec](../../specs/ui/pagination.md)已落主；Linux原生及统一回归待补，非跨平台全门通过 |
| CL-04 | 第二批 PromptDialog | Dialog/Input/AlertDialog、027重命名/新建表单 | 受控单文本输入、本地与业务校验、确认/取消/提交锁。027 原生文件操作保留在应用内，Vue 维持只读 | [PLAN-094](../../plans/archived/094-prompt-dialog.md) r3 已交付，Windows Native VM 与 Vue 通过独立复审。[规范](../../specs/ui/prompt-dialog.md) · [正式包](../../../packages/prompt-dialog/README.md) |
| CL-05 | 第二批 TaskProgressDialog | Progress/Dialog、Reader主检出导入和批处理状态（只读） | 确定/不确定进度、结果明细、请求取消/取消中/终态；首个消费者为应用持有的实际记录校验任务，Reader迁移另立产品计划 | [PLAN-095](../../plans/095-task-progress-dialog.md)，drafting r2，0/8；复用既有参考图 |
| CL-06 | 第二批 CommandPalette | 现有Command族、028 Launcher搜索/选中/键盘 | local/provided 单一过滤来源、稳定 ID、分组、禁用和键盘提示。028 保留原评分、recent、环绕和宿主启动 | [PLAN-096](../../plans/archived/096-command-palette.md) r3 已交付 `a227015e`，Windows Native VM 与 Vue。真实 AutoOS Launcher 搜索后单次启动 Calculator。[规范](../../specs/ui/command-palette.md) · [正式包](../../../packages/command-palette/README.md) |
| CL-07 | 第二批 TagInput | 015 Notes标签控件/store、Reader主检出book_detail（只读） | 标签增删、精确去重、静态建议、键盘/上限和修订确认；015真实存储消费，Reader后续独立迁移 | 已归档 [PLAN-097](../../plans/archived/097-tag-input.md)（delivered r2 @`4950ddd5`：包 `packages/tag-input/` + 规范 [docs/specs/ui/tag-input.md](../specs/ui/tag-input.md)；015-notes 迁移；Windows native/vue 独立复审通过，Linux 两轨按 084 §10 记 not_run 未宣称支持） |
| CL-08 | 通用扩展批 ColorPicker / ColorDialog | 031 Paint 色板、Dialog、087数值输入 | 内嵌选色与对话框；sRGB/HEX/RGB/HSV、透明度、预览/提交/取消；实际Paint消费 | 已立 [PLAN-088](../../plans/088-color-picker-and-dialog.md)，drafting r2，0/8 |
| CL-09 | 通用扩展批 ImageViewport / ImagePreviewDialog | 031局部ImageViewport、ImageSurface、029 Gallery | 缩放/平移/90°旋转/适应窗口、错误与modal；图片查看不扩成绘图或3D引擎 | 已归档 [PLAN-089](../../plans/archived/089-image-viewport-and-preview-dialog.md)（delivered r3 @`5dbc3661`，9/9：包 `packages/image-viewer/` + 规范 [docs/specs/ui/image-viewport.md](../specs/ui/image-viewport.md)；031/029 迁移 + 窄窗“更多”菜单；091 消费 pin=`5dbc3661`） |
| CL-10 | 能力补齐研究 DataTable | 已有注册、Vue 映射、026 表格及 CRUD blueprint | 先核查排序/选择/原生交互，再补正式合同；不作为“从零新增 DataTable”统计 | 待调查 |
| CL-11 | 能力补齐研究 Date/Time Picker | Calendar 声明及原生日历转换 | 核查日期、时间、区间、格式、键盘和双端语义，再决定组合组件 | 待调查 |
| CL-12 | 储备研究 FontPicker / FontDialog | Reader 字号预设、字体资源及主题接口 | 先调查字体枚举、可用字体与回退；现有字号选择不等于完整字体选择器 | 待调查 |
| CL-13 | 通用扩展批 NumericField / UnitField | Input/Button、number_field声明、示例数值设置 | 十进制草稿/验证/步进/范围、明确单位换算；原builtin兼容 | 已交付 [PLAN-087归档](../../plans/archived/087-numeric-and-unit-fields.md)，r3、6/8（T-07/08归复审/合并），交付45a80058；[Spec](../../specs/ui/numeric-fields.md)；[包](../../../packages/numeric-fields/README.md)；Win native+vue双轨门全绿，Linux轨not_run如实待补 |
| CL-14 | 通用扩展批 PropertyInspector | 标准表单控件、087/088 | 数据描述的分组设置/属性面板；类型验证、默认值恢复、目标修订；无任意反射/eval | 已立 [PLAN-092](../../plans/092-property-inspector.md)，drafting r2，0/8 |
| CL-15 | 通用扩展批 AssetBrowser / AssetPicker | 029 Gallery、085树、086分页、089图片预览 | 授权资源库分类/搜索/过滤/单选/预览；不复制本地FileDialog或文件服务 | 已归档 [PLAN-091](../../plans/archived/091-asset-browser-and-picker.md)（delivered r2，8/8：包 `packages/asset-browser/` + 规范 [docs/specs/ui/asset-browser.md](../specs/ui/asset-browser.md)；`examples/asset-picker/` + 029 Gallery 迁移 + Win native 18/18 & Vue 5/5 & tools 2/2 全绿；Linux native 无 runner 单列 `not_run`） |
| CL-16 | 通用扩展批 DockLayout / DockPanel | resizable/tabs/slots、041布局 | 同窗口分割、标签分组、移靠/resize/隐藏恢复、布局数据保存；非OS窗口dock | 已立 [PLAN-093](../../plans/093-dock-layout-and-panels.md)，drafting r2，0/8 |
| CL-17 | 通用扩展批 MediaPlayer / TransportControls | 020/030媒体视口和控制条、现有video/mpv | 音视频普通播放/定位/音量/倍速、真实回灌/失败；不造解码器或媒体库 | 已立 [PLAN-090](../../plans/090-media-player-and-transport-controls.md)，drafting r2，0/8 |
| CL-18 | 专业储备 VectorField / TransformEditor | 后续复用087、AGE变换数据 | XY/XYZ及位置/旋转/缩放；等首次游戏/CAD对象编辑明确坐标/单位语义后设计 | 仅候选，未立计划 |
| CL-19 | 专业储备 GradientEditor | 后续复用088、现有渐变绘制能力核验 | 色标/透明度/插值；等图像编辑或粒子/材质应用出现时讨论 | 仅候选，未立计划 |
| CL-20 | 专业储备 LayerPanel / SceneTree | 后续复用085 | 图层/场景层级、显隐/锁定/排序；等分层文档/游戏场景模型明确后设计 | 仅候选，未立计划 |
| CL-21 | 专业储备 Ruler / Guide / GridOverlay | 图片/画布/场景坐标机制 | 标尺、参考线、网格/吸附反馈；等排版/绘图/CAD精度和坐标需求明确后设计 | 仅候选，未立计划 |
| CL-22 | 专业储备 TransformGizmo | AGE相机/命中与后续变换编辑 | 移动/旋转/缩放手柄、轴约束/吸附；等2D/3D对象操控应用再设计 | 仅候选，未立计划 |
| CL-23 | 专业储备 Timeline / TrackEditor | 现有媒体进度/未来动画模型 | 播放头/轨道/片段/关键帧；等动画/视频剪辑应用明确时间基准后设计 | 仅候选，未立计划 |
| CL-24 | 专业储备 CurveEditor | 后续图形绘制/数值输入 | 控制点/切线/插值；动画缓动和图像色调曲线可能需不同领域适配 | 仅候选，未立计划 |
| CL-25 | 专业储备 NodeGraphEditor | 后续通用图形交互和应用图模型 | 节点/端口/连线/分组；等材质/流程/脚本应用明确类型/连线规则后设计 | 仅候选，未立计划 |
| CL-26 | 专业储备 CanvasViewport / SceneViewport | 已有Canvas/AGE混合区域；图片部分先由CL-09收拢 | 通用绘图/场景视口、坐标/选择覆盖层；等图像创作或游戏/CAD应用再设计，不重复图片查看器 | 仅候选，未立计划 |
| CL-27 | CollectionView / Thumbnail | 091 的资源卡片和列表、027 的明细与网格、现有缩略图渲染 | 共用网格、列表、缩略图、焦点与条目动作。AssetBrowser 和 FileManager 均已迁移 | [PLAN-109（归档）](../../plans/archived/109-collection-view-and-thumbnail.md)，已交付 r2，`a6b16627` |

未标外仓的复用定位均为本仓相对路径：示例在 `examples/ui/`，schema 在 `crates/auto-ui-compiler/schema/aura.at`。Reader外仓只读来源在095/097 §4列出完整位置和版本。Blueprint 单源仍是语言仓 `packages/blueprints`，不得因抽取 UI 包而产生第二份可编辑通用工具实现。

### 2.1 通用扩展批的选择理由

按用户“先做更常见、可能用于其他类型App的一半左右”的要求，从上一轮约15组创作组件中选出7组。CL-08/09沿用原候选ID，不重复立项；复杂视口先收拢图片查看部分。下表解释用途，不维护第二份执行进度。

| 组件 | 日常理解 | 专业领域之外的用途 |
|---|---|---|
| 数值/单位输入 | 能加减、校验、带单位的数字框 | 阅读字号、缩放、超时、尺寸、普通设置 |
| 选色器 | 像文字颜色设置一样选色和透明度 | 笔记标注、主题预览、标签/封面配色 |
| 图片视口/预览 | 能放大、平移、旋转的图片查看器 | 阅读器插图、聊天附件、商品图、文件管理器 |
| 媒体播放/控制条 | 可嵌进页面的音视频播放器 | 课程、语音/视频附件、素材预览 |
| 资源浏览/选择 | 从应用已有素材/附件库中选一个资源 | 封面、附件、模板、图片素材，不要求本机文件路径 |
| 属性面板 | 根据字段描述自动组合一组设置项 | 偏好设置、表单、对象详情编辑 |
| 停靠布局 | 能调大小、换位置、收起恢复的工作区面板 | 编辑器、文件管理器、调试台、数据工作台 |

CL-18…26当前只保留目的、来源和重新讨论的触发条件，不冻结API、不创建实施Plan。等实际应用出现，再与当时的领域模型一起设计；无需先学习全部专业编辑器概念才能使用本批通用组件。

## 3. 第一批执行顺序和所有权

原先086→084的代码交付链已完成：086交付 `765251b9`，084交付 `7ea71f0c`且归档8/8。**下一项建议085 TreeView**：四个现有消费者均有可复用局部实现，无尚未交付组件依赖，并为091 AssetBrowser提供前置；T-01先在当前084/086基线上核对027/041，保留其文件/保存业务。不因新组件计划跳过第一批剩余项，也不重开084/086。Linux/统一回归及084的DEBT-006依归档收据待补，代码交付不等于这些门已通过。

| 范围 | 唯一 owner | 交接规则 |
|---|---|---|
| 受控分页 API、页数/边界计算及两个首批消费者迁移 | PLAN-086 | 消费者负责数据切片；FileDialog 的内部 0-based 页码在接入层转换成公共 1-based 页码；数据库原有 1-based 合同保持 |
| FileDialog 新模式、路径验证、覆盖确认、模式示例 | PLAN-084 | 已交付7ea71f0c；后续消费者沿正式Spec/包，不重写已统一分页，保留打开模式及Reader行为 |
| 树交互、节点合同、官方包和四示例接入 | PLAN-085 | 不接管文件系统、数据库请求或 Editor 工作区业务；加载结果由消费方持有 |
| 消费应用的依赖 pin、导入数据和发布 | 各应用 owner | auto-ui 交付包和工具的版本/摘要；需要同步 vendor 时列完整依赖闭包，不能只拷主文件。应用在自己的计划中验证实际数据结果 |

`pagination` 已是内置名字，组件注册表优先级为 Builtin > Local > Package。因此新组合包使用 `PaginationControl`，保留内置声明族，禁止靠同名遮蔽接管解析。T-01 必须证明真实导入/生成/运行链路，不能只通过字符串检查。

### 3.1 通用扩展批的依赖和建议顺序

既有086→084代码已交，当前先085；085不等待新批组件。新批优先基础087和图片089，088消费087；090可独立推进。下面依赖指评审交付的确切版本，T-01调查可以提前，但不能用复制实现绕过依赖。这里不授权自动启动实施会话。

| Plan | 后续实施接续 | 同文件/状态owner协调 |
|---|---|---|
| 087数值输入 | 无新组件硬依赖，为088/092提供基础 | 独立普通设置夹具；不抢媒体/图片业务owner |
| 088选色器 | T-03等待087 | 负责Paint选色和内容颜色兼容，不改主题全局状态 |
| 089图片查看 | 可独立；087为可选复用 | 先迁031视口与029查看区，随后091重基线网格 |
| 090媒体播放 | 可独立 | 负责020/030控制条和实例媒体身份；业务列表/服务不搬迁 |
| 091资源选择 | T-03等待085/086/089（089 pin=`5dbc3661`，消费 `packages/image-viewer/` + `docs/specs/ui/image-viewport.md`） | 消费029网格，复用图片预览；不另建文件系统服务 |
| 092属性面板 | T-03等待087/088 | 先交普通设置消费，资源字段/反射/多对象编辑不纳本版 |
| 093停靠布局 | 核心探针可独立，041迁移T-04等084/085 | 保留Editor文档/撤销/焦点和actions；与OS native_dock功能分开 |

新批首版控制范围：选色仅sRGB普通颜色；数值单位仅明确可换算组；资源库先单选；停靠只在单窗口；图片只查看变换；媒体不新造解码器；属性面板只支持列明的普通字段类型。专业领域计算仍由应用/AGE/媒体服务或未来专业内核持有。

### 3.2 轻量交互批094–097的接续

四项可分别进入各自T-01，无彼此或087的硬依赖。在轻量交互批内建议先094（单输入和明确消费路径），随后097；096和095按消费者需求接续，不要求等待多媒体组件。全路线下一项仍优先085，以上是四项内部顺序。各自复用现有基础机制，不借“统一交互”另造全局提交服务/任务引擎/命令总线/标签数据库。

| Plan | 首批消费者及复用owner | 关键交付边界 |
|---|---|---|
| 094 PromptDialog | 027应用持fs操作；新增普通名称管理示例跨端读回 | pending不重复提交，失败留窗；027 Vue不增加磁盘写权限 |
| 095 TaskProgressDialog | 新增普通记录校验任务；Reader仅作状态/取消协议参考 | 发cancelRequested不等于任务已停止，任务与结果由应用持有 |
| 096 CommandPalette | 028持原排序/recent/宿主；普通设置示例跨端执行 | local/provided不双重过滤，查询序号防迟到结果；原有网格/环绕保留 |
| 097 TagInput | 015 Notes持update_tags/vocab；新增普通设置示例 | 稳定noteId+revision防错写，业务拒绝保留草稿，计数仅看已接受tags |

已有四组件图存仓库assets，四个Plan直接链接同一PNG/指南和各自区域；不重复生成图片。实现后应逐Plan登记源码/包/工具身份和真实窗口证据，交付后才把本页阶段改为已支持。

### 3.3 共用资源视图接续

CollectionView 与 Thumbnail 由正式包提供。AssetBrowser 在 091 交付基线上复用集合视图，027 FileManager 保持真实目录、排序、权限与缩略图路径。Windows 原生 VM 与 Vue 验收记录见 [PLAN-109 归档](../../plans/archived/109-collection-view-and-thumbnail.md)。Linux 补验见 [DEBT-011](../../plans/debts.md#debt-011-collectionview-的-linux-平台验收)。

## 4. 统一组件合同

### 4.1 平台与后端

原生运行基线沿现有 FileDialog Spec：**Windows、Linux 的 native VM**，两端使用同一 `.at` 组件和交互语义，不调用系统文件窗口。TreeView、PaginationControl及087…097同时要求Vue浏览器交互验收；094–097将Win native、Linux native、Win Vue、Linux Vue逐项列为必需门，缺门不汇总成跨平台通过。视觉布局和行为保持一致，路径根、隐藏属性、大小写和权限按各OS的真实语义展示，不伪造统一文件系统。

FileDialog 的目录 IO 是原生宿主能力；生成 Vue 不授予浏览器任意磁盘访问权。浏览器沿既有用户授权文件/字节令牌路径；没有显式宿主 provider 时新目录/保存模式须给出具名不可用结果，不能弹系统窗口冒充普通 MCP 控件。Rust 源码生成通过不等于交付 Rust 宿主文件服务。

macOS 采用相同组件和 API 方向，但当前没有本路线的真实窗口证据。宣称支持前补实际构建、原生窗口、路径/字体、MCP、关闭重开门；不得把 Windows 通过或交叉编译记成 macOS 通过。首批不将 macOS 标成已支持。

### 4.2 视觉、键盘及状态

使用现有主题语义色、尺寸和 focus ring，不另造固定调色板或等待新样式语言。共享部件包括工具栏、紧凑图标按钮、状态行、空/加载/错误态。图标按钮必须保留可访问名称和 hover 提示；checkbox 有独立列；长中文/路径可读，滚动区与固定操作区分开。浅色、深色、小窗口、缩放后可操作。

受控数据和组件局部焦点分别持有；两个同页实例不能串扰。按稳定业务 ID 寻址，重排、过滤、重开后旧 MCP 节点不得命中新对象。空态、加载、失败、取消、禁用都必须可观察，不能以空列表掩盖 IO 失败。

### 4.3 MCP 与无人值守

使用实际控件树的 snapshot/find/action/type/键盘/scroll/screenshot。测试不得注入选中路径、直接调用 accepted 回调、用 `AUTO_OPEN_PATH` / `AUTO_SAVE_PATH` 代替操作，也不依赖操作系统前台窗口焦点。树的展开/选择、页码切换、保存覆盖确认都必须有普通可寻址控件。

无人值守门采用隔离临时目录、合成数据、独立端口、带版本摘要的工具，记录原生窗口身份、当前节点发现、实际回调/持久读回、错误对照、退出、进程/端口回收及重开。只清理由本轮创建且身份验证过的进程和目录。Linux UI沿084归档§10中用户指定的自制发行版+VMware路线，须产生真实native窗口/非空控件树；不以WSL/WSLg替代或继续追WSLg。缺runner、窗口、动作或断言记not_run，相应验收保持未关闭。

### 4.4 编辑组件的共同行为

颜色、数值、图片视图、属性和布局均明确区分外部受控值与实例草稿/手势；一次提交最多一个业务事件，取消不擅自写文档。需要撤销的应用根据begin/update/commit/cancel组织历史，组件不另造全局撤销栈。

滑块/色面/图片/面板拖动必须有同一事件链上的精确输入、按钮或命名菜单入口，供键盘和MCP使用；不是测试后门。资源、字段、panel、媒体会话都按稳定ID和revision/owner绑定，旧动作不得误命中新内容。图片和视频验收看实际像素/解码，属性和布局验收看真实消费值/内容/bounds，不只看显示标签或测试总数。

## 5. 质量门和现存问题

| ID | 内容 | 关闭依据及 owner |
|---|---|---|
| Q-01 | Reader009首报，084双实例复现并归因公共渲染器文本shaping/布局（input/text）；精确修复未交付 | [DEBT-006](../../plans/debts.md#debt-006-native-渲染器文字字形丢失strategy-q-01plan-084-复现)持修复依赖和现有证据；各Plan T-01检查影响，真实截图关闭受影响AC；不在组件包做逐字形workaround，不以MCP文本完整/环境归因结案 |
| Q-02 | 声明存在不等于原生交互可用 | 每个候选立项前跟踪声明→生成/转换→真实事件→消费结果；首批各自 T-01 形成决策记录 |
| Q-03 | 组件主文件升级但依赖/工具未同步 | 交付时列源提交、包文件集合和摘要、运行工具摘要；消费方读回实际所用版本，独立完成产品验收 |
| Q-04 | 平台或无人值守门缺失被总体 green 隐藏 | 每个平台/后端分别列 pass / fail / not_run 及证据；必需门未运行则相应能力未验收 |

Q-01 原始消费证据现可在Reader主检出的[实测报告及同目录截图](../../../../auto-reader/docs/reviews/file-dialog-ui-sync-2026-11-13/README.md)读取；旧`.wt/reader-009/`已不是可依赖入口。报告可读不等于字体问题已修复；各受影响Plan仍须记录当前版本截图和归因。UI 的[旧改进报告](file-dialog-ui-polish.md)是当时的视觉/功能记录，不覆盖后续Reader的问题。

## 6. 后续立项和更新规则

1. 从 CL-ID 选择候选，核验当前 Spec、源码和至少一个真实消费者；合并重复需求，避免重开已归档计划。先明确复用边界，再新建或修订实施 Plan。
2. Plan 写明平台矩阵、受控状态、MCP 节点/动作、错误/取消语义、真实消费者、测试入口和 Spec 增量；缺机制时由有边界的 T-01 调查先产决策，不用假想 API 掩盖阻点。
3. 在本页更新候选的唯一主 Plan、revision、阶段和依赖。具体任务勾选仅留在 Plan；本页状态过时时标日期并以 Plan 为准。
4. work 完成记录源码/工具/依赖身份和各门结果；独立 review 在同一实施树核验，通过后 merge 沉淀 canonical Specs、交付和归档证据。再把本页从“已立计划”改成“已交付”，附最终提交、Spec 和消费门链接。
5. 新增组件或更改支持范围时，先修订本页的候选/边界，再将完整合同写进对应 Plan。没有自动执行、定时追踪或跨应用发布授权由本页产生。

本次[094–097执行准备度审核](../../reviews/component-plans-094-097-2026-11-14/README.md)记录身份/确认/排序/现有机制与平台证据缺口及r2闭环；结论只允许进入T-01，不替代组件产品review。

## 7. 本次接续记录

| 日期 | 变化 | 验证状态 |
|---|---|---|
| 2026-11-13 | 建立 CL-01…12；首批关联 084/085/086，确定 086→084 的联调依赖；补统一平台/MCP/视觉门和 Q-01 | 仅规划；三个 Plan r1、0/8；代码、Spec、应用副本均未变更 |
| 2026-11-14 | r2：从创作工具候选中选7组通用组件，CL-08/09升为088/089，新增CL-13…17及087/090…093；专业CL-18…26仅登记，补日常用途、首版边界和依赖 | 新增7个Plan均drafting r1、0/8；仅计划/设计，无代码或产品测试 |
| 2026-11-14 | r3：用户要求再次审核执行细节与生成界面图；087…093修为r2，补接口/消息样例/状态/反例/视觉规格和现有runner边界 | 七项仍drafting、0/8；文档与图像资产完成，实施/产品门未运行 |
| 2026-11-14 | r4：补文件导航084–086图、CL-04…07轻量交互候选图和提示；084–086修r2并挂入视觉case；Color/Image复用既有参考 | 084/085及087–093仍drafting r2、0/8；086已开始实施，保留已完成T-01和owner记录；CL-04…07待立项。本次只补图/文档 |
| 2026-11-14 | r5：CL-04…07分别补齐094–097，来源/接口/异步身份/真实消费/反例/平台MCP与无人值守门；引用既有四组件PNG，更新设计导航；对账086交付归档状态 | 四项drafting r1、0/8，计划静态自检，无组件实施/产品测试；086代码交付765251b9，Linux原生与统一回归仍待补 |

| 2026-11-14 | r6：再次审查094–097并修r2，补挂载身份/显式确认、现有Init/焦点限制、保留跨组评分、GUI/MCP/IME证据分层与失败清理；对账084交付并推荐085 | 四项drafting r2、0/8；静态计划审核。084交付7ea71f0c/归档8/8，Linux（VMware）/统一回归/DEBT-006仍待补；无组件实施 |
| 2026-11-16 | r7：CL-13/PLAN-087 交付——numeric-fields 官方包（NumericField/UnitField 十进制定点引擎）+examples/numeric-fields 消费示例+双轨产品门；编译器两处最小适配（a2ts range 助手内联、camelCase 事件名保形） | 087 delivered：r3，T-01…06 完成 6/8，交付45a80058归档；Win native（20+1not_run）/vue（21）门全绿；Linux not_run 待补；088/092 可消费 087 基础（版本/摘要见归档证据） |
| 2026-11-20 | CL-27 / PLAN-109 CollectionView 与 Thumbnail 交付 | r2，`a6b16627`。Windows 原生与 Vue 通过，Linux 两轨按 DEBT-011 补验 |
