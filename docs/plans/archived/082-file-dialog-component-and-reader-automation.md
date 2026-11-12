---
plan_id: PLAN-082
status: archived
feature_name: FileDialog 通用文件选择组件与 Reader 无人值守选择链
author: [Codex]
created_at: 2026-11-12T00:00:00Z
updated_at: 2026-11-12T00:00:00Z
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/reader/library-workspace.md]
new_spec_components: [docs/specs/ui/file-dialog.md]
touched_goals: []
---

# PLAN-082：FileDialog 通用文件选择组件

## 0. 变更摘要

新增 AutoUI 官方复合组件 **`FileDialog`**（中文：文件选择对话框），以应用内模态 Dialog 呈现路径输入、目录列表、文件类型过滤、名称过滤、单选/多选与确认/取消。首个消费者为 auto-reader：书籍多选导入和封面单选。所有操作经过普通 AutoUI 控件和现有 MCP 事件通道，消除该链对系统 IFileDialog 焦点和人工操作的依赖。

本计划已获 `/goal 完成计划082` 全链授权并进入 executing，当前完成 T-01..T-07（7/8），独立最终复审已通过，实现保持冻结，待交付收口。早期仅写 auto-ui；Reader 写入交接延至 T-05。不改 PLAN-081 的 D3 范围，不宣称 READER-009 已通过。

## 1. 目标

1. 提供 Reader 之外也能使用的 AutoUI 组件；不把目录浏览与选择状态写死在 Reader。
2. Windows/Linux 原生 VM 应用使用相同布局、交互和事件协议；差异收敛到路径/文件系统适配。
3. 真实用户和 MCP 使用同一套控件完成打开、导航、过滤、选择、确认和取消。无人值守测试不需要抢系统焦点、模拟桌面鼠标或手动选文件。
4. 将 Reader-009 当前八个 picker 相关叶子的选择阶段改为自动操作，保持其独立 catalog/API/受管字节 oracle、原门槛与分母。

首版非目标：保存/另存为、选择目录作为结果、新建/删除/重命名/复制/移动、文件预览、缩略图、递归搜索、收藏/最近使用持久化、完整文件管理器、网络文件协议和云盘。目录可浏览，但结果只包含文件。file-manager 仅作为复用设计参考，本期不迁移其业务。

平台边界：必交 Windows 和 Linux 的原生 VM 文件选择；Linux 无真实执行环境时保持该 AC 未通过，不能以 Windows 结果宣称跨平台交付。普通浏览器没有任意本机目录访问权，Reader Vue 继续使用现有浏览器授权 picker/字节句柄链，不伪造绝对路径；Vue 本地目录 provider、macOS 实机及 a2r 自制选择器接线为后续范围。既有 a2r/浏览器构建兼容须保持。

## 2. 架构方案

### 2.1 组件、文件系统和业务的边界

建议以官方 `.at` 组件包 `packages/file-dialog/`（新）发布 `FileDialog`，内部复用现有 Dialog/Popover Modal、输入框、按钮、列表、复选框与官方主题；内部目录面板为私有组合单元，首版不另承诺公共 `FilePicker`。不以仅 Reader 可用的临时页面或自画 OS 子窗口替代组件。

数据流：应用设置打开状态和配置 → FileDialog 实例 → 异步目录 provider（本地 Rust 适配）→ 当前请求/目录行 → 用户或 MCP 选择 → 确认前校验 → accepted/cancelled/error → Reader 选择适配 → 既有导入/封面 API → 受管文件与结果 UI。组件不读出版物正文、不做 EPUB/图片解析、不承担导入事务。

现有 `host_adapters/dialog.rs` 是 rfd 单一 OS 真源，`ui/dialog.rs` 是 a2r 薄门面；不直接在其中用阻塞等待替换系统弹窗，也不全局改写 `dialog_open*`。Reader 改用显式组件事件回调，避免原 `.PickFiles -> pickLocalFiles().await` 在等待结果时阻挡组件自己的交互派发。旧原生 API 暂保兼容，Reader 的新原生产品链不允许静默回退系统选择器。

### 2.2 有界可行性任务

T-01 在唯一实施树内验证“外部官方组件包 → 打开普通 Modal → 异步列目录 → MCP 输入/勾选 → emit 到 routed Reader 页”的整链，冻结实际可编译语法与 provider 接入方式。优先复用当前公开 FS/宿主能力；若只能在 UI runtime 添加 App 级服务，以不违反 compiler→runtime 禁止反依赖的方式接入。**不得先假定 `.at` 可持 Rust trait 或编造新语言 builtin。** 需要语言新能力时提交具名最小前置与复现，依赖步骤等待其合同就绪；本计划不直接改 auto-lang。

## 3. 技术栈

AutoUI `.at` 官方组件包、Rust 文件系统/provider 与异步任务、现有 VM/Iced 模态渲染和 MCP snapshot/action 通道、现行主题 scope。组件逻辑通过普通 props/model/msg/emit 组合；provider 负责文件系统 IO，异步结果回到所属 App/组件实例。单测与集成沿本仓 Cargo 命令；Reader 沿现有 Python 产品门。

## 4. 需求分析与背景调查

### 授权与基线

用户 2026-11-12 明确要求：结合其他 UI 框架命名/设计基本功能，以 Reader（尤其 009 测试）为首版依据，建立实施计划。已授权本轮只读调查与新增计划文件；未将“建立计划”解释为本轮立即实施、合并或更改消费者现行计划。本轮不改其他未提交文档、`.next-id`、派生 index 或 legacy ledger，不创建新 chat/代理。

起草观察：auto-ui 主检出 `69f52f24`；Reader 主检出 `aa58fef763e440394484662e102329b7ee7f0471`；009 权威树 `D:/autostack/.wt/reader-009/auto-reader` 为 `2370eca1545cdd3999ce84f42c230861e0dfd0a7`。主检出有他人 WIP；这些是观察基线，实施 T-01 必须重新冻结兼容的 source/tool/dependency cohort，不能直接复用安装工具冒充新组件。

| 来源（已读） | 起草事实 / 定位 |
|---|---|
| `docs/specs/README.md`、`docs/specs/ui/structure-closure.md` | 规范总览、四面 owner 与禁止反依赖；尚无 FileDialog 规范 |
| `docs/specs/runtime/component-and-store-instances.md` | 实例/事件/迟到回调约束；SHA256 `5BA3A663F74DF551633925EE3F7588C913B500C65A5547FEB8C0C68C1EC28B3D` |
| `docs/specs/ui/theme-scope-and-publication.md` | App/实例主题隔离；SHA256 `703DA9A3F06AF73DE5D4DD7B0B99C9468F31BEAC554DF7124DEFF8DC511132A5` |
| `docs/specs/runtime/native-vm-app-validation.md` | NVM-01..06：当前树发现→真实 MCP 事件→数据读回→close/reopen；SHA256 `A20E599F2D2494BADC3EB88486EC86BB8912B88E7BF9E25E97747B52BE5953E2` |
| `docs/plans/081-vm-host-dialog-lifecycle-and-s-region-redelivery.md` r2 | 干净环境 16/16 可打开不等于可自动操作；DirectUI 操作困难。081 已收窄 D3，本组件另立合同 |
| `../auto-reader/docs/plans/009-workspace-delivery-and-import-seams-closure.md` r3 §6/§9 | 真 UI 选择后独立读回；不允许 API 预写、预置已在库书籍冒充成功 |
| `../auto-reader/docs/specs/reader/library-workspace.md` | 主检出导入描述仍含旧 GAP；不能覆盖 009 当前实施证据，T-05 对齐真实交付后再校正文案 |
| 009 树 `src/front/ports/file_host.vm.at` | 书籍 `dialog_open_files("txt,md,markdown,epub")`；封面 `dialog_open("png,jpg,jpeg,webp")`；name/path/local_id envelope；SHA256 `1701F647D4B7AECDB582EB66175E91A6B97EBFDF5D3489A95EE89DA49F1A28D0` |
| 009 树 `src/front/pages/bookshelf.at` / `book_detail.at` | `.PickFiles`/`.PickCover` 现在 await 文件选择；首版需分别接 accepted/cancelled/error，恢复 importing 状态 |
| 009 树 `tests/product/verify_readest_workspace.py` picker-gate | `picker_session.py` 是八叶唯一生产者；原每腿 600 秒轮询人工操作，不应另新增第二生产者导致重复计数 |
| `crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | 官方组件包加载/实例接入已有基础 |
| `crates/auto-ui-compiler/src/ui_gen/widget/registry.rs`；`crates/auto-ui-runtime/src/ui/view.rs` | 已有 Dialog 注册及 PopoverPlacement::Modal，应复用 |
| `examples/ui/027-file-manager/src/back/api.at`、`src/front/app.at` | 已有列目录、根入口及前端名称/排序处理；可参考路径经验，不依赖或复制 file-manager 业务 API |

### 外部组件参考与取舍

- [Qt Quick FileDialog](https://doc.qt.io/qt-6/qml-qtquick-dialogs-filedialog.html)：借鉴 `FileDialog` 命名、currentFolder、fileMode（单/多文件）、nameFilters、selectedFiles 与 accepted/rejected 分离。首版不实现 SaveFile，也不采用原生窗口优先策略。
- [GTK FileDialog](https://docs.gtk.org/gtk4/class.FileDialog.html) 及 [open_multiple](https://docs.gtk.org/gtk4/method.FileDialog.open_multiple.html)：借鉴 initial-folder、filters、accept-label 和异步完成；保持取消与运行错误区分。

以上为 2026-11-12 在线查阅的设计参考；项目契约由本计划定义，不追随上游 API 版本。

## 5. 详细设计

### 5.1 首版界面与交互

```text
┌ 选择书籍                                            × ┐
│ 根位置 [D:\ ▾]  [上一级]  [路径输入……] [转到] [刷新]    │
│ 名称过滤 [________________]  类型 [支持的书籍 ▾]       │
│ [ ] 名称（目录在前、文件名稳定排序）      类型 / 大小    │
│     📁 子目录                      [进入]              │
│ [✓] 示例.epub                                         │
│ [ ] 笔记.md                         （可滚动列表）     │
│ 载入中 / 空目录 / 无匹配 / 具名错误与重试               │
│ 已选 2 项   [清空选择]                    [取消] [打开] │
└────────────────────────────────────────────────────────┘
```

最小布局可随窄窗口换行，列表滚动，底部操作始终可达；沿 App family/mode 主题，不硬编码 Windows 外观。类型/大小作为简单辅助信息，不提供多列排序系统。

- `initialDirectory` 默认使用应用给出的目录，否则用户 home；无效初始路径显示原因并允许回 home，不静默取消。Windows 可从盘符/根位置切换，Linux 提供 `/` 与 home；不做网络位置发现。
- 路径框接受目录绝对路径、相对当前目录路径及文件完整路径；Enter/转到目录=导航，文件=导航父目录并预选，**仍须确认**；单个输入路径不解析 shell、环境变量或引号列表。空路径报校验错误。显示名与完整系统路径分离，不用拼接斜杠方式推断平台根。
- 目录显示独立“进入”动作，普通点选文件切换选中；多选使用每行复选框、不依赖 Ctrl 修饰键；单选最多一个。返回上级在根禁用，刷新重读当前目录。首版不做跨目录选中篮子：成功换目录、名称过滤或类型过滤变化均清空选择，UI 和说明明确这一规则；导航失败保留上一有效目录/选择并显示输入错误。
- 类型过滤使用命名扩展名组（不含点、大小写不敏感；提供配置内的过滤组切换），目录不被扩展名过滤隐藏。Reader 仅提供支持格式组，不能用“所有文件”绕过类型约束。名称过滤为当前目录文件名的大小写不敏感子串，目录仍可导航，不是递归搜索或正则。
- 默认隐藏系统/点隐藏项，可显式“显示隐藏文件”；默认目录在前、名称稳定排序，相同排序键按原名/完整身份决胜。Windows 隐藏属性和 Unix 点文件规则归 provider。空目录和过滤无匹配区别于错误。
- 无有效选择时禁用打开；确认前重校验存在、普通文件/可读、符合类型组、去重。某选中文件已消失或不可读则保持对话框、列出失败项、不发布部分 accepted；用户调整后重试。文件过滤不是内容校验，坏 EPUB/伪图片仍由 Reader 报业务错误。
- Esc、取消、关闭按钮都只触发一次 cancelled；外点遮罩不关闭、也不把点击传到后层。打开时焦点进入路径框，Tab/Shift+Tab 留在弹层，Enter 执行当前控件语义、空格切换勾选；关闭后回原触发控件（已卸载则回可用 App 焦点）。底层 MCP 动作也要受模态边界约束。

### 5.2 对外语义接口（提案；T-01 冻结 `.at` 写法）

| 面 | 首版合同 |
|---|---|
| 配置 | `open`、`title`、`initialDirectory`、`selectionMode = singleFile / multipleFiles`、`filters = [{id,label,extensions}]`、`activeFilterId`、`acceptLabel`、`showHidden`；默认 singleFile、打开。没有 filter 时为所有普通文件 |
| 状态 | currentDirectory、pathInput、nameQuery、activeFilterId、loading、entries、selectedPaths、error；状态按实例隔离，父组件不维护第二份选择算法 |
| accepted | 一次性 `files: [{name,path}]`，非空、去重、绝对系统路径、按当前稳定列表次序；组件关闭。保留 OS 路径表示，JSON 安全转义 |
| cancelled | 一次终态，不携带文件、不被解释为错误；组件关闭 |
| error | `{code,message,retryable}`，如 not_found / permission_denied / invalid_path / io_error / unsupported_path；可恢复错误不关闭、不伪装空列表或用户取消 |

IO provider 的概念能力：roots/home、listDirectory、resolve/stat/validate；不提供任意文件写操作。路径含中文、空格、引号、反斜杠必须往返一致；首版 `.at` 字符串不能无损表示的 Unix 非 UTF-8 名称具名 unsupported_path，不替换字符后返回另一条路径。符号链接显式导航一次解析，不递归扫描；坏链接具名错误。UNC/网络目录可按 OS 路径尝试，但不承诺离线网络及时返回或网络位置搜索；挂起请求不得冻结主事件循环，可取消/改目录并忽略迟到结果。

### 5.3 生命周期、规模与 MCP 合同

- 打开建立 session token；每次列目录建立 request generation。旧目录请求晚于新目录完成、取消后回调、owner 路由卸载均不可修改当前实例或新打开会话。accepted/cancelled 互斥且至多一次；关 App 释放自有任务引用。一个 owner 同时一个活动对话框，重复打开聚焦现有弹层，不叠栈。
- 目录扫描不在渲染线程同步执行；仅当前层，不递归。首版验收夹具含 1000 条目录项、中文长文件名，列表分页或虚拟滚动，所有条目均可达。T-01 冻结实际分页/批大小和响应预算，不能把产品 1000 本书指标当目录性能证据；不静默截断超额目录，也不承诺任意规模瞬时完成。
- MCP 使用已有 tree/snapshot、输入、press/选择、滚动和键盘通道；不新增“替用户直接返回指定路径”的测试专用捷径。控件暴露角色、名称、enabled、selected、loading、错误和当前目录；每行携带稳定实例内身份和可区分的原始名称/路径。按当前树发现 element ID，不硬编码帧内编号。
- 各动作都有普通控件：转到、上一级、刷新、名称过滤、类型切换、显示隐藏、文件勾选、进入目录、清空、取消、确认。真实 UI 与 MCP 共用 handler/选择校验。snapshot 不只显示文字而必须注册可操作 action；过期行 ID 不能选中新目录同一序号的另一文件。
- 不宣称只靠逻辑单测就完成无人值守：必须在真实 native VM 窗口中，用该 session MCP 完成选择，并观察业务消费。Windows/Linux 分别有可追溯执行证据。

### 5.4 Reader 集成及责任

书架 `.PickFiles` 打开 `multipleFiles`，扩展组 txt/md/markdown/epub；详情 `.PickCover` 打开 `singleFile`，扩展组 png/jpg/jpeg/webp。把原 await 之后的业务提取到 accepted 消费 handler；取消/错误分别处理并恢复 busy 状态，不让打开选择器就预先锁死所有交互。结果映射回 `{name,path,local_id:""}`，保留既有导入路径与逐书结果。

组件只确认文件可读及扩展名，不修改 Reader 的 30MiB 出版物、8MiB 封面限额、hash 去重、坏文件诊断、取消保持 catalog/封面不变等规则。重复书仍交 Reader 去重；多选结果不能重复同一路径。Vue 的 File/字节 token 不转换为本地绝对路径。

UI082 负责组件、通用 fixture、接入补丁与这八叶的自动选择适配；READER-009 原 owner 负责同一权威 Reader 树写入交接、冻结 pin 和最终整体验收，原八叶的断言不删改。081 的 D3、995 后写入族和 W-07 等其他问题不挪入本计划，也不因八叶通过就宣布 009 全量成功。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/file-dialog.md（新） | 无公共组件 → props/事件、路径/过滤/多选、错误与平台边界 | 可复用公共契约 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/ui/file-dialog.md（同文生命周期/MCP节） | 无目录会话规则 → 异步请求隔离、模态边界、可操作节点与真实无人值守证据 | 防错选、死锁和假自动化 | AC-03/04/05 |
| SD-03 | modify | auto-reader 仓 docs/specs/reader/library-workspace.md（跨仓） | 原生系统选择描述/遗留GAP → 自制组件选择与实际 delivered/pending 事实；业务阈值不变 | 首个消费者真实契约；落入 Reader 的009沉淀项 | AC-06 |

已存在实例、主题和 NVM 规范直接遵守，不预设改其合同。只有 T-01 证明公共规则必须改变才提出对应 revision；不为实现方便削弱既有约束。

## 6. 测试设计

以下为原定检查设计；实际执行结果和精确版本见§9。T-02 扩展 `crates/auto-ui-runtime/tests/file_dialog.rs` 并建立 `crates/auto-ui-runtime/tests/file_dialog_model.rs`（模型/provider，不启动 OS 文件窗），T-04 建立 `crates/auto-ui-tools/tests/file_dialog_component.rs`（新，包加载和生成兼容）及 `tests/product/verify_file_dialog.py`（新，真实 VM/MCP 产品夹具）。具体 fixture app 放 `examples/file-dialog/`（新），T-01 冻结其 pac/启动依赖。

| Case | owner_stage / AC | 输入、实际入口与不变量 | 区分性 oracle / 边界 |
|---|---|---|---|
| C-01 | work / AC-01/02 | 包编译与真实 fixture 窗：MCP 路径输入→进入子目录→上级→单选/复选→确认 | 文件全集由隔离临时目录独立生成，确认结果精确等于所选绝对路径，不仅核 count；同名不同父目录、中文/空格/引号路径不串 |
| C-02 | work / AC-02/04 | 类型切换、名称过滤、隐藏项、目录仍可进入、大小写扩展名、空目录/无匹配、刷新 | 精确可见集、选择清空规则、selected/enabled 与路径结果一致；非法类型直接输入仍不能确认；坏 EPUB 由 Reader 而非 selector 内容判断 |
| C-03 | work / AC-03 | 慢 A→快 B、旧行 action、关闭再打开、两个同型实例/不同 App、owner 卸载、重复确认 | A 不覆写 B；旧行不选中 B 同序号；旧 session 不给新 owner 发布结果；accepted/cancelled 互斥；有合法当前请求对照。故障注入限模型/provider，真实 fixture 另验取消重开 |
| C-04 | work / AC-02/03 | 不存在/权限拒绝/被删文件/坏链接/unsupported_path，读取中取消和重试 | 错误具名且 UI 可恢复，无假 empty/cancel/部分 accepted；更正合法目录后确能选择。权限夹具须先证实 OS 真实拒绝，不靠管理员环境误判 |
| C-05 | work / AC-04/05 | 真实 Windows/Linux VM：模态打开后 MCP 试按底层动作、路径键盘输入、Tab/空格/Esc、滚动到1000项尾部、明暗/窄屏、连续打开取消10轮 | 底层业务无副作用；焦点恢复；尾条确可选；截图中底钮可达；十轮全有单次取消且再开可用。MCP不依赖窗口前台；不运行侵入式桌面焦点实验 |
| C-06 | work / AC-06 | 009 picker-gate 八叶，真实组件控件→Reader handler→API/catalog/受管文件冷读 | 见下表；不存在预置已导入书使 oracle 假绿；书籍/封面字节 hash 与源一致；数据失败不能被选择 UI 的 PASS 掩盖 |
| C-07 | work / AC-01/06 | 旧 dialog API 定向测试、组件包生成、Reader Vue 既有文件选择和编译兼容 | 不全局改旧 API 语义；Vue 仍为授权文件/字节链，不暴露虚假路径；旧门选测数>0 |

009 八叶的**既有名称必须保留**（以下均由新原生组件触发，不靠系统窗手测）：

| 叶子 | 自动动作与独立断言 |
|---|---|
| W-02.picker-mixed-batch | 列表勾选 TXT/MD/EPUB → 确认 → 三格式正确受管字节/书卡与逐项结果 |
| W-02.small-batch-budget | 按现有夹具和预算真实选批导入；记录选择时间/导入时间，不放宽原阈值 |
| W-02.picker-duplicate-bad | 选含重复与坏文件批次 → 逐项成功/重复/失败，失败无假书 |
| W-02.picker-cancel | 打开再取消 → 库数量/原件 hash 不变，busy恢复且可再开 |
| W-03.picker-png | 详情中选 PNG → 受管封面字节身份和实际显示成立 |
| W-03.picker-jpeg（producer保留picker-jpg） | 同上 JPEG，不只看扩展名 |
| W-03.picker-webp | 同上 WebP |
| W-03.picker-cancel | 已有封面时打开再取消 → 原封面身份和字节不变 |

计划命令（带“新”的入口由相应任务创建后才可运行）：

```powershell
# UI 实施树，T-02/T-04 后；输出必须选中实际测试，零选择/skip不算通过
cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model
cargo test -p auto-ui-tools --test file_dialog_component
cargo test -p auto-ui-runtime --lib plan804_dialog_dnd_provider
python tests/product/verify_file_dialog.py --auto-bin <pinned-auto> --out-dir <isolated-output>
# Reader 009 权威树；入口/参数已存在，T-05修其picker生产者
python tests/product/verify_readest_workspace.py --profile vm --auto-bin <pinned-auto> --out-dir <isolated-output>
python tests/product/verify_readest_workspace.py --profile vue --auto-bin <pinned-auto> --out-dir <isolated-output>
```

T-01 枚举旧 provider 测试实际全名并冻结定向回归清单、fixture 命令和性能预算；新组件入口不得将缺测试、缺平台、连接失败、GAP/not_run 计 PASS。Reader 聚合入口可能因既有独立问题失败：保留真实退出码，八叶结果单列；将首个失败边界交009/081原 owner，不为让UI082通过隐去失败。

内部检查由实施上下文完成；最终复审由后续独立上下文在同一实现树顺序执行，当前不派代理、不指定模型。复审重演 C-01/C-03/C-05/C-06 的实际路径与反例，绑定 plan_revision、两仓 commit、工具及依赖 hash；同会话起草检查不当独立 final pass。

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 官方可导入 FileDialog 包与公开配置/事件可用；独立 fixture 不依赖 Reader；C-01/C-07，旧 API 兼容与包加载均非零选测通过 |
| AC-02 | work | 路径导航、根位置、名称/类型/隐藏过滤、单/多选、确认校验及错误恢复完全符合§5；C-01/02/04精确集合与路径断言通过 |
| AC-03 | work | 列目录不阻塞 UI，过期请求/行/关闭owner不污染新状态，终态至多一次；C-03/04与真实取消重开通过 |
| AC-04 | work | 普通 MCP tree/action 可独立操作全部关键控件；模态输入隔离、键盘和焦点恢复、1000条尾部可达、明暗窄屏成立；C-01/02/05具直接证据 |
| AC-05 | work | Windows/Linux native VM 分别完成真实组件选取、取消重开，UI/MCP行为同合同；C-05及NVM记录，不用跨编译或mock代替Linux实机 |
| AC-06 | work | Reader 原生书籍多选/封面单选消费组件，八叶全自动且原oracle/阈值不变，Vue授权链受影响回归成立；C-06/C-07，精确pin与SD-03交接可查 |
| AC-07 | review | 独立上下文核AC-01..06及SD-01..03、反例、平台/八叶实证与规范文本，绑定实际reviewed_commit，全部通过才 reviewed |
| AC-08 | merge | 精确交付UI及Reader相应提交/依赖pin，规范核验、耐久回执、归档与自有树清理；不动009其他未完成项/他人WIP |

## 8. 执行步骤

T-01..T-07已完成，AC-01..07由§9最终证据及独立最终复审覆盖；两仓实现冻结。下一步T-08精确提交交付、规范核验与清理；AC-08仍待实际收口。

| ID | owner_stage | 依赖 | 改面/结果 | 验证与预期 | AC |
|---|---|---|---|---|---|
| T-01 [x] | work | 实施启动；与009原owner交接 | 在新UI树核实际cohort/包机制/异步FS与emit路径；用最小完整fixture证明§2.2，记录 `D:/autostack/plan-artifacts/plan-082/feasibility.md`（本地临时，耐久摘要见§9）；冻结API、分页/响应预算、命令/测试全名、Linux runner及Reader写入时段 | 真实MCP选择回调成功，未阻塞；无语言改动或具名前置已交付后再放行依赖步骤 | AC-01/03/05 |
| T-02 [x] | work | T-01 | 新 `crates/auto-ui-runtime/src/ui/file_dialog/` 模型/服务和 `tests/file_dialog.rs` / `tests/file_dialog_model.rs`；按T-01落实provider挂载（可能触及dynamic/vm_bridge/session，确有必要才改） | C-02/03/04；异步请求、路径往返、错误分类、取消、稳定选择全集oracle通过 | AC-02/03 |
| T-03 [x] | work | T-02 | 新 `packages/file-dialog/package.at` 与组件源；复用Dialog/Modal及主题，补必要snapshot/action/焦点/模态接线；不复制一套专用MCP调度 | C-01/02/05相关模型与UI腿；所有控件可发现可操作、1000项尾部可达 | AC-01/02/04 |
| T-04 [x] | work | T-03 | 新fixture、UI产品门、包/生成兼容测试；完成Windows/Linux实机与10轮取消重开，补SD-01/02规范 | 第6节UI命令全绿、非零选测、C-01..05/07，平台分列证据 | AC-01..05 |
| T-05 [x] | work | T-04；009写入交接 | Reader `bookshelf.at`/`book_detail.at` 与 `ports/file_host.vm.at` 事件接入、包依赖；`tests/product/picker_session.py`/聚合picker生产者改MCP自动操作（先核实际路径）；SD-03由009沉淀 | C-06八叶全PASS；Vue受影响回归；完整009矩阵实际结果/外域问题据实交接，无人工picker等待 | AC-06 |
| T-06 [x] | work | T-05 | 汇总cohort/路径/命令/hash/每AC与Spec delta证据；内检反例/旧门、两仓pin候选；保持实现树供复审 | 第6节适用门及AC-01..06全部闭合，无缺平台/缺八叶；仅review/merge项未勾时可execution_done | AC-01..06 |
| T-07 [x] | review | T-06 | 独立上下文同树复演并审规范，暂停实施写入；失败回原树修复 | AC-07 pass，精确版本绑定；不改源代码替实施者修复 | AC-07 |
| T-08 [x] | merge | T-07 pass | UI组件先交付、Reader消费精确pin按009原owner流程交付；canonical核验/耐久摘要/本计划归档与自有scope清理 | 实际交付哈希、规范路径/回执和清理齐全；009仍由原合同判全局完成 | AC-08 |

实施树：UI `D:/autostack/.wt/ui-082/auto-ui`，分支 `codex/plan-082-dev`，已创建；T-01提交 `f8fbb1e1`。Reader **复用** `D:/autostack/.wt/reader-009/auto-reader` / `codex/reader-009-dev`，不另建副本并行写同一app；T-01记录实际resolved路径/branch/commit，T-05暂停009其他写入并交接。计划簿记留主检出。其他兄弟仓只读；需要跨仓能力前置另立合同，不直接扩范围。

## 9. 复审记录

- 2026-11-12：stage: new | plan_id: PLAN-082 | plan_revision: 1 | outcome: pass（实施合同可交接，非实现/独立复审通过） | next: work T-01，待后续启动实施。新增T-01..08、AC-01..08、C-01..07、SD-01..03；本轮仅完成源码/规范/009/081与官方框架资料调查及草案创建。所有产品检查未执行，0/8不预勾。
- 设计自检：不把 native 系统选择 API 全局替换；复合组件+非阻塞provider+事件接入形成闭环；目录列表的1000条与Reader书库1000本分开；Linux实机为交付门而非推测；008/009原业务验收与其他owner缺陷不被该组件代结。

- 2026-11-12：授权补充：用户 `/goal 完成计划082` 授权 work → independent review → bounded repair → merge 全链；不改变 r1 的范围、验收和跨仓归属。预检 UI master=69f52f24ad8d767a78f32623798debc5b9ee3370；无 ui-082 树。Reader 009 权威树工作区干净，branch=codex/reader-009-dev。未开始实现/构建/产品测试。
- 2026-11-12：stage: work | plan_id: PLAN-082 | plan_revision: 1 | outcome: blocked | worktree_path: not_created | branch: codex/plan-082-dev (intended) | task_ids: T-01 | evidence: git status --short / git worktree list --porcelain；Reader 主检出存在未跟踪 tests/.reader_smoke.mjs（7246 bytes，读取确认为 Playwright 测试源码）及两份 assembly.json；UI 主检出存在 examples 下未跟踪 deps/dist/build 和既有 .wt | blockers: 主检出零 WIP 入口要求，源码归属尚未确认；未改动/纳入他人文件 | next_responsible_role: user/原 owner 路由 WIP 后 work T-01。
- stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 1 | from: work | to: user | status: drafting | outcome: waiting | worktree: not_created | commit: 69f52f24ad8d767a78f32623798debc5b9ee3370 | next: user

- 2026-11-12：stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 2 | from: user | to: user | status: drafting | outcome: waiting | worktree: not_created | commit: 69f52f24ad8d767a78f32623798debc5b9ee3370 | next: user | 用户答复：由原任务负责人将 tests/.reader_smoke.mjs 移入对应工作树，完成后继续；本轮 git status / Test-Path 核验文件仍在主检出，因此入口尚未解除，未执行实现/构建/产品检查。

- 2026-11-12：stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 3 | from: user | to: user | status: drafting | outcome: blocked | worktree: not_created | commit: 69f52f24ad8d767a78f32623798debc5b9ee3370 | next: user | 第三轮同一入口阻塞：git status 仍列出 Reader tests/.reader_smoke.mjs，Test-Path=True。用户选择等待原 owner 路由，不授权本会话搬移；未执行阶段工作/检查。目标标记 blocked，解除方式为原负责人移入所属工作树后用户恢复目标。

- 2026-11-12：用户明确覆盖此前脚本入口阻塞：忽略 Reader tests/.reader_smoke.mjs，直接创建 auto-ui worktree；早期仅 UI 树，Reader 写入交接延后 T-05。已执行 git worktree add -b codex/plan-082-dev D:/autostack/.wt/ui-082/auto-ui master，exit=0；基线 69f52f24ad8d767a78f32623798debc5b9ee3370，树/分支核验正确且工作区干净。不创建 Reader 树，不改该脚本。旧 waiting/blocked 回执保留为历史，以本授权解除其条件。宿主 goal 当前仍为 blocked，需用户 /goal resume 激活自动阶段链；本次仅完成用户明确要求的 worktree 建立，T-01 尚未闭合。

- 2026-11-12：恢复 work T-01。上一轮建立干净 UI 树为 progress，用户已恢复 active，旧脚本阻塞已解除。执行入口：UI=69f52f24ad8d767a78f32623798debc5b9ee3370；只读依赖 detached sibling：auto-lang=f9fb3854beb3886338156cfa61d16f7f84893cc9、auto-web=8b1742a0f84c24173768265cb999a648039d1e2b、auto-down=58ea33f699efa5cb1194473a2d45dcec51006842（仅满足现有 Cargo 相对依赖，不修改兄弟仓）。Reader 不建树/不写入。Linux Ubuntu-22.04 WSL 可执行，DISPLAY=:0、WAYLAND_DISPLAY=wayland-0；此项只证明 runner 存在，非 AC-05 通过。
- T-01 当前生产者/消费者探针：packages/file-dialog/FileDialog 的普通模型 fd_active/fd_request/fd_path → DynamicComponent 拥有的 ComponentIo → 非渲染线程 DirectoryRequest → 相同实例 state_id/代际的 fd_reply JSON → 组件 Tick → accepted/cancelled → fixture App；取消/卸载丢弃接收通道，后台不持有 VM。既有 dialog_open* 未改；compiler 不依赖 runtime，无 auto-lang 修改。该接入尚待实际包加载/模态/事件/MCP证明，不能放行 T-02。初始 cargo check -p auto-ui-runtime --locked exit=0（2m34s）；原始日志 D:/autostack/plan-artifacts/plan-082/check.log。当前 cargo test --test file_dialog 编译在途，非通过。

- 2026-11-12：T-01 checkpoint pass（仅可行性任务，非 work 阶段最终 pass）。code_commit=f8fbb1e1，UI 权威树 D:/autostack/.wt/ui-082/auto-ui / codex/plan-082-dev。`cargo check -p auto-ui-runtime --locked` exit=0；`cargo test -p auto-ui-runtime --test file_dialog --locked` exit=0，4 passed/0 ignored/0 filtered：directory_identity_sorting_unicode_and_errors、superseded_request_and_cancel_cannot_publish_old_results、package_fixture_builds_and_opens_a_real_component_instance、nested_callbacks_are_bound_to_their_parent_instance_and_app。
- T-01 Windows真实链：`cargo rustc -p auto-ui-runtime --example file_dialog_fixture --locked -- -C link-arg=/STACK:16777216` exit=0，二进制 SHA256=008A6001BE613B9C647F3624B3CEBF8722665E05B217F524A647C1A6CE7482D8。独立夹具原生窗 PID18408 / HWND23724682 / title Auto - App，MCP19282；24次当前树发现/动作，路径框→转到→进入中文子目录→checkbox toggle→打开→routed selector 结果文本与新建持久回调文件（先删除旧输出防假绿）→精确 os.path.samefile 书籍身份成立；取消两次计数1→2、既有 accepted 文件字节不变、重开可操作。截图 tmp/autoui-screenshot-1794468821527.png 已人工视觉检查：真实居中遮罩/面板、路径框、列表和底部按钮；这只证 T-01 窄链，未证 C-05 全项。
- T-01 区分性反例：首轮 native-probe.log 中 accepted 落盘而 routed 页结果仍空、取消计数0；原因是 dispatch_parent_route 固定写 App 根态、type级进程全局 route。修复：VmBridge 按实际 child state保存回调，按实例 parent_state_id派发、实例释放清理回调，跨App自然隔离；新增 keyed siblings + 不同App不同callback绑定反例及 routed parent 精确结果/零参取消/重复取消检查。旧 dialog_open* 未变；无 auto-lang 改动。原型首次缺少 accepted/cancelled msg 声明亦已补齐。
- T-01冻结：包语法 `use { package: file_dialog from "..." }`，widget普通 props/msg/model 足够；最终配置名保持§5.2，selectionMode为singleFile/multipleFiles字符串、filters为List记录；accepted(str)最终携JSON `{files:[{name,path}]}`，cancelled零参，error(str)携JSON `{code,message,retryable}`。**当前原型 accepted 仍是单一路径，只作为可行性片，T-02/03必须补成最终合同**。目录后台IO→App-owned ComponentIo→对应实例/代际fd_reply→Tick消费，fd_*为私有协议；最终UI分页100项/页，不截断全集；本地1000目录项结果预算2秒、普通MCP动作确认1秒（网络不可达路径不作及时性承诺），待T-04实测，未预记通过。Ubuntu-22.04 WSL GUI runner存在，Linux真实窗口验收仍待做。用户指示早期只UI，Reader写入时段在T-05交接，不提前锁009树。
- T-01旧门冻结（静态枚举，不是执行）：ui_tests::plan804_dialog_dnd_provider::plan804_dialog_dnd 下 core_shims_have_no_dialog_dnd_os_reference / shim_absence_contract_dialog_error_dnd_false / shim_delegation_and_slot_independence / shim_cancel_returns_empty_string / ni_slot_lock_released_before_provider_call / assembly_identity_and_merge_preserve_providers；T-04执行 `cargo test -p auto-ui-runtime --lib plan804_dialog_dnd_provider`。新组件及Vue门仍待T-04；Reader八叶待T-05。
- T-01本地证据：D:/autostack/plan-artifacts/plan-082/feasibility.md SHA256=2a3bd5cb61ab6e537171172422fbbf48764d49f37bb9795367e565b58a940b43；feasibility-final.log=d8623d70efc6f761e6dd29974875e481f08b30226ab2af757d2f29ff06cd0680；native-probe2.log=b40d75143f11fe1895241fbfc5f1cb1fc3c0776ed012364ffdf5377adaa2f2d4；native-probe-trace.json=77abc6b97fc416119d63bf26d2eee63aa74a7e6a6ef2a8b42421aba7b573736c。原生探针CloseMainWindow未退出的一次被自有PID强制清理，导致过一次LNK1104后重建通过；不得据此宣称NVM-05干净关闭已过，T-04另验。目录隐藏过滤/完整选择校验/焦点/模态MCP边界/1000项/两平台/Reader/独立review/merge皆未闭合。
- stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass（T-01 checkpoint only） | worktree: D:/autostack/.wt/ui-082/auto-ui | commit: f8fbb1e1 | next: work T-02

- 2026-11-12：T-02进行中（基线f8fbb1e1，UI树/分支不变）。实际入口：DialogModel 单独持有 filters/selection/current listing/session/revision；FileSystem trait 只做 navigate/validate，LocalFileSystem 在后台线程解析路径、列当前目录/根、确认前 stat+File::open；worker只持owned请求和取消flag，channel替换使过期结果不可发布。成功导航/过滤/刷新清选择；失败导航保上一有效目录/选择；row id含session/revision/完整path；单选/多选与稳定列表序accepted均由同模型决定。T-03将替换T-01原型状态算法，不保留双份选择真源。测试分两目标（既有file_dialog + 新file_dialog_model），对应第6节命令同步修正，验收规则未变。
- T-02初始 Windows 模型6/6通过（model-tests.log）；1003项全集分页/尾项、精确可见集、坏路径恢复、消失文件原子拒绝/恢复后成功、慢A→快B/取消重开等已有真实执行。补加迟到validate/filter/cancel/owner-drop、跨模型row token、JSON路径与Windows实际hidden/system属性检查，待最终统一运行；不能用初始6项代替新增覆盖。
- Linux prerequisite实证：首次因WSL本机代理失效无法拉registry，当前命令仅清本次proxy环境并设CARGO_HTTP_PROXY为空，未改用户全局配置，下载/编译已能进行。随后出现同仓renderer非Windows stub可见性7个E0425（4个函数）：sync_native_geometry/native_slot_dpi_scale/handle_native_slot_event/restore_all_native_slots，在shell_state.rs定义为私有，host.rs兄弟模块经glob无法调用。已仅补pub(super)以对齐Windows分支，不改行为；model-linux3.log保留原红，model-linux4.log重跑在途。Linux实际权限/非UTF8/坏链接模型腿尚未记PASS，Linux原生组件窗更未验。

- 2026-11-12：T-02 checkpoint pass（仅任务，整体仍executing 2/8）。Linux构建入口修复提交2b5e98cc；核心模型/组件接线提交ba97656d，UI权威树干净。DialogModel按实例拥有所有选择状态；ComponentIo按state_id/实例拥有模型，释放实例的同一构建帧清理自有任务，reload清理服务；worker无VM引用。fd_command/fd_view/fd_event是包私有投影/命令/事件邮箱；事件排队逐个送给普通emit，不新增MCP接口。每次普通handler结束即排空命令，防两次勾选之间无Tick时覆盖前一操作；CPU完成路径在安全段边界同步排空。组件当前薄视图已消费新模型；T-01单路径原型回调已正式替换为accepted(str) JSON `{files:[{name,path}]}`，fixture由路由handler解析消费。
- T-02精确结果：Windows `cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model --locked` exit=0，4+10=14 passed，0 ignored/filtered；Linux同一命令 exit=0，4+10=14 passed，0 ignored/filtered。Linux经Ubuntu-22.04非root(uid1000)实际执行，测试先要求OS File::open真实PermissionDenied再判provider同码，另验Unix非UTF8名unsupported_path/坏链接not_found；Windows实际SetFileAttributesW并读回hidden/system位后，验证默认隐藏/显式显示。平台特有用例各自执行，未用Windows代替Unix腿。
- T-02区分性oracle：真实目录精确可见集（大小写类型、名称子串、目录保留、隐藏）、失败导航保上一有效目录与选择/合法恢复、完整文件路径仅预选且不能绕类型组、成功导航/过滤/刷新清选择；单选最多1项与清空；删去已选b.md后confirm仅发布Error而无部分accepted，恢复同文件后重试按稳定列表序[a.txt,b.md]一次性accepted；1003个中文长名称文件全集跨11页精确集合相等、每页≤100且尾项1002可确认，模型请求结果≤2秒断言通过（不代表真实窗口1000项门）。受控慢A→快B、取消后重开、迟到validate在filter/cancel/owner-drop后不发布accepted、旧session/旧revision行拒绝、当前行阳性对照、跨模型隔离与JSON路径往返均通过。
- T-02组件接线回归：Reader/auto-lang均未修改（仅写UI，依赖cohort只读），包fixture在Windows/Linux均经真实Widget/VM构建、View::Popover Modal打开、普通路径/两个实际checkbox绑定事件（中间无Tick）→已选2项→异步确认→精确routed父页结果；取消计数只增1，重复取消不增；同型keyed父/另一个App不同callback不串。此处为组件/VM测试，**不是Linux原生窗口/NVM pass**。真实MCP模态边界、旧行vnode身份、键盘/焦点、主题窄屏、1000项UI可达、10轮取消及Reader八叶仍待T-03..05；完整AC未预勾。
- T-02证据目录 D:/autostack/plan-artifacts/plan-082/：model-windows-final.log SHA256=E610E00B8989016750F3E5D24A043FA8AC6BE549684830ED902ECCC2CF26880E；model-linux-final.log SHA256=28a490189a2daf35e49bca3fba757d0a26b2e7901e909e955965eef65844e835。Linux命令仅对本次构建取消HTTP(S)/ALL proxy环境并设置CARGO_HTTP_PROXY为空，CARGO_TARGET_DIR指UI树target-linux；未改全局proxy。该目录为明确自有、已本地排除的构建中间物，最终随UI树清理。旧系统dialog API定向6测、包/Vue生成兼容仍由T-04执行；不扩大为全仓测试。
- stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass（T-02 checkpoint only） | worktree: D:/autostack/.wt/ui-082/auto-ui | commit: ba97656d | next: work T-03


- 2026-11-12：T-03 checkpoint pass（局部任务，整体executing 3/8），code_commit=edfa4695a11cda998b06fd229b0addc6865a3409。官方组件完成home/roots/up/refresh、路径Enter/转到、名称/类型/隐藏过滤、选择/清空/分页/重试、关闭/取消/确认；公共配置与JSON事件沿普通实例handler。根位置用横向滚动避免native不支持flex-wrap导致裁切；滚动主体与底部操作分开。
- T-03输入边界：普通MCP action/type/sequence及keyboard共用当前模态范围，背景负控零业务事件；关闭内容不发布为可操作树。Button/Checkbox发布ID绑定实例+实际消息（含row session/revision/path），列表位置复用不使旧ID命中新行；发布/实际View异步交错亦拒绝旧identity，几何alias保留。snapshot checked/disabled/enabled及实际handler由同一View发布，修复源Aura展开路径错配的邻接事件元数据。按钮disabled表达式补typed bool/比较/否定，空选择确认、首尾分页、活动类型与清空的禁用状态有直接组件断言。
- T-03焦点：跨平台原生Button/Checkbox焦点包装器保留原鼠标handler，Enter/Space发同一消息；ModalExplicit外点/失焦不关闭、Esc普通cancel；Tab/ShiftTab限活动弹层并循环。App按模态进入/退出保存恢复焦点，代际guard阻止旧异步焦点偷取新会话。Windows系统provider共享实际控件焦点，避免双焦点owner/双次Space；新增headless生产树断言system focus与native focus一致且只发一次事件。Tick按真实child identity订阅，两个同型选择框（先开后关）均收到自己的时钟，不再落最后实例。
- T-03验证：Windows controls-tests3.log为5组件+10模型通过；disabled-controls-tests.log新增禁用/启用断言5/5通过。Linux controls-linux.log同样5+10=15通过（含Unix权限/非UTF8真实腿）。focus-modal-tests5.log 5 passed（4本计划+1既有同前缀表格测），旧p491焦点7/7、vnode_converter 23/23通过。Windows accessibility整组初次并行12/14通过，两红分别RefCell借用/unchanged wake；两红单测各通过，新增共享焦点用例后明确 --test-threads=1 整组15/15通过（accessibility-regression-serial2.log）。不把并行红日志删除或伪记为全绿；后续原生headless门顺序执行。
- T-03 Windows真实宿主（由当前树example生产VM/iced/MCP路径启动，非NVM最终CLI腿）：controls-probe2.log 41次真实调用通过，Tab/ShiftTab/Space选择计数0→1→0→1→2、背景动作/CtrlS拒绝、旧目录row拒绝、中文文件accepted JSON新输出exact samefile、取消/关闭两次且输出字节不变；navigation-probe4.log 67调用通过path Enter/隐藏真实属性/清空/两类型/名称/刷新/坏路径具名错误和重试/up/home/D盘。large-probe.log 1003中文长名跨11页精确集合，无重复/截断，末项1002真实确认samefile，目录就绪0.7557秒<2秒；旧第一页ID在尾页拒绝。截图1794474313687显示关闭后焦点环回到选择文件按钮；1794473996888显示真实模态/路径框焦点/底部按钮。
- T-03证据界限：上述真实二进制在最后metadata/provider共享焦点小修前构建，最后小修有unit生产树证据，T-04必须重新构建精确CLI并重演；不得以旧binary代最终交付。Linux真实窗口、明暗窄屏、10轮取消、工具包/生成兼容、旧系统API六测、规范SD-01/02仍待T-04。Windows自有进程WM_CLOSE后主窗消失但进程/MCP仍驻留（无NVM05 pass）；本轮明确清理自有PID35100，T-04需要真实退出输入/干净重开证明，不复用旧端口的陈旧snapshot。
- T-03本地scope D:/autostack/plan-artifacts/plan-082/，SHA256：controls-linux.log=67F580FF01B1C3F3CE3892F6B0E284CFAE39325B71078F4DC59284D7849FBE6B；controls-probe-trace.json=9BE1E9A6FBFB0ED999DDA48B42C8B0BA73ABEB6C4B5513676DACDAEA0E3FFAD5；large-probe-trace.json=57FAC18BE7436B1ACE69218C15E46011A5323CA8AC968402481D5EEB235A9637；navigation-probe-trace.json=D898264DF7EC6D8A01809D0957D074E340E21403DE2BC64A01B375898A5DDED1；accessibility-regression-serial2.log=F08A8FF7EE6BA177E98AA5ECB1D9FEC1D75DDE81C5F543DF60F5EA85581E83FA。Reader/其他依赖无写入；UI主线已由其他任务前进68cb676d，待merge按实际主线集成，不覆盖他人提交。
- stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass（T-03 checkpoint only） | worktree: D:/autostack/.wt/ui-082/auto-ui | commit: edfa4695 | next: work T-04


- 2026-11-12：T-04进行中，T-03基线edfa4695。已新增永久 tests/product/verify_file_dialog.py、tools file_dialog_component 两测、公开规范SD-01/02草案及单选/错误/确认计数/主题/正常退出夹具。UI树保持唯一writer；新增只读CLI依赖tree D:/autostack/.wt/ui-082/auto-man detached54fe4070bdbd06015ea4521074479ae2e2f8b14f，以构建Linux core CLI；不改该仓源码，不写Reader。Windows工具构建和Linux工具/core CLI构建均通过。Vue真实request生成测试揭示外部包只注册名字而未写SFC；采用已有受支持的src/front/components原样vendoring路线，两测Windows/Linux各2/2通过（package-generation-tests3.log / package-generation-linux.log），不把Vue生成当浏览器本地目录能力。
- T-04当前实机证据（尚非最终源码全闭合）：product-windows-08与product-linux-05均full_native_vm pass，真实auto run -r vm +自有标题/OS窗口句柄；精确同名不同父目录、JSON路径、返回多文件顺序[a.txt,b.MD]、singleFile后选b、类型/名称过滤/目录保留/not_found公共error/合法retry且原accepted字节不变、10轮取消重开、1003项11页精确集合/尾项确认、主题、正常退出CLI=0/全部自有PID消失/旧MCP真实connection-refused与可重绑/新会话新动作及新落盘全部通过。Windows目录就绪0.3112秒，Linux0.8605秒；动作ack均<1秒（不把startup snapshot计动作）。Windows CLI SHA=e762cec894a52f94ebb16da3499b7889ed8861ff8df4633418c31034d1022385，UI工具=30d309ef14200b88a95b6a5e5920430bebc3caca42f3d0e3eb0181b31b9525e6；Linux CLI=bf8560b8831b2601d2dc388ea4887c346319204c6c6a858e4249d7c83e2e93a3，UI工具=bd431670f63323c092dd040b7377403046fa420a414823b6c5f7d5377806846e。Linux runner Ubuntu-22.04 X11/WSLg、ICED_BACKEND=tiny-skia真实软件绘制窗口；默认GPU曾Invalid surface，原红保留。初始Linux索引联网未就绪，随后只在隔离app/am.at选已有本地registry镜像URL，CLI正常缓存克隆，不改用户全局配置。
- Windows/Linux 420x560 scoped narrow_visual门均通过，三张明暗/滚后footer截图已视觉检查：路径框聚焦可见、滚动后a.txt勾选、页码/选择数量和底部按钮可见可操作。修复ModalExplicit关闭→重开时内容Tree重建，避免上次滚动位置遮住初始路径焦点；后续dark重开截图已证回顶部。Windows实际UIAutomation系统树记录Edit/CheckBox角色、路径值、focus=true与可见bounds，并通过系统Toggle→MCP已选1→普通toggle恢复0。旧系统API六测、Windows runtime5+10、plan082头less六测通过。
- T-04追加反例未闭合：系统树中背景按钮仍enabled，实际UIA Invoke把底层次数0改为1；system-modal-counterexample.log/trace-1.json保留，trace SHA256=1D95ED1C7BCE4D6D036EC9B382D46457A967CEEFEE647704738BDFF66FA52920。因此上述full门仅证明MCP/控件/生命周期，不充当系统模态边界最终pass，T-04仍未勾。已实现发布时禁用/移除背景系统动作 + 消费时当前View范围/关闭内容复核；新增打开前排队动作反例、关闭后合法背景对照。首轮因新增fixture model分号解析与另一App初始化后的堆ID相同暴露身份缺口；fixture改合法多行，VmBridge补进程唯一host_owner_id，系统Owner/Key含App ID，两个已初始化App隔离。system-modal-tests2.log16/16通过；当前最终源围栏测试与重建在途，必须用新工具重放系统反例后才可T-04闭合。


- 2026-11-12：T-04 checkpoint pass（仅任务，整体executing 4/8），code_commit=949b7c0152aa4f64ab3a50422b6c7c48c4722781，UI权威树clean。SD-01/02草案已在该树 docs/specs/ui/file-dialog.md；新永久入口为tests/product/verify_file_dialog.py、crates/auto-ui-tools/tests/file_dialog_component.rs与examples/file-dialog/README.md。新fixture公开正常“退出夹具”输入，Process.exit(0)走宿主正常返回/工具Completed，而非以强杀充当NVM关闭；旧WM_CLOSE驻留记录保留为历史，最终门用文档化退出输入完成。
- T-04最终系统边界闭合：system-modal-tests-final.log16/16顺序执行通过，覆盖原provider14项、共享实际焦点/一次Space、新的当前modal消费范围与打开前排队反例/关闭内容反例/关闭后合法背景阳性。VmBridge host_owner_id按构造独立于局部堆ID生成；Owner与系统Key含该App ID，另一个已初始化同构VM仍不可消费动作，reload不复用旧节点身份。最终Windows真实UIAutomation中底层计数enabled=false、InvokePattern不可用、底层次数保持0；合法a.txt CheckBox系统Toggle后真实MCP读回已选1，再普通toggle恢复0。目录路径Edit读到非空值、focus=true、560x68可见bounds；每个角色/值/焦点/bounds原始记录在system-tree.json。最终MCP背景与旧行负控同样通过，关闭后底层动作正常计1。
- T-04最终生产cohort：两平台都从独立app副本真实执行 `auto run -r vm`，FILE_DIALOG_INITIAL_DIRECTORY/RESULT_PATH只供业务夹具，不注入选择或回调。Windows已安装CLI版本 `auto 0.1.0+plan-890-verified-r7-t19-20-g347c29f-dirty`，以实际二进制hash绑定而不推断它等同某源码HEAD；Linux core CLI来自只读54fe4070树、版本auto0.1.0+unknown。UI源949b7c01；auto-lang=f9fb3854beb3886338156cfa61d16f7f84893cc9 / auto-web=8b1742a0f84c24173768265cb999a648039d1e2b / auto-down=58ea33f699efa5cb1194473a2d45dcec51006842保持只读。Windows UI tool SHA256=90e409d8cca657a98d4ff0bd8035db7a9c5afdeeb7bb6cac93c20b28f778c50f；Linux UI tool=5e8b7d95b81442ad3f5cce7dd346677d2e40f84579dafe369049fb139c7f001c。CLI hashes延用本节前述精确值。Windows最终首窗PID46924/HWND213584012/title FileDialog fixture；Linux首窗PID439/XID0x600002/title相同。Windows两会话MCP端口13064/7732、native PID46924/26752，第二HWND81726966；Linux两会话MCP58397/35923、native PID439/519、XID各0x600002。CLI/工具父子归属在启动时捕获；两次退出均核所有捕获PID消失与旧端口可重绑。
- 最终 `product-windows-final` / `product-linux-final` 均full_native_vm pass、两会话成功退出CLI=0，所有捕获自有PID退出、旧真实MCP请求得到有界connection-refused并可重绑，同配置fresh app/session再真实选择并新写JSON。原有accepted JSON与新回调分开校验，无预置导入结果；samefile精确身份、稳定有序多选[a.txt,b.MD]、单选最后b、同名不同父目录/中文空格单引号路径、过滤/错误/原文件不变、十轮每次仅cancel+1、1003项11页全集相等/尾项1002选取均通过。目录ready Windows0.40425秒、Linux0.86311秒，低于2秒；普通MCP动作ack<1秒。Windows系统反例修复后重新构建/重放；Linux亦在最终common owner字段构建后完整重放，不沿用旧二进制。
- T-04视图/兼容：最终Windows tail截图实际显示1000/1001/勾选1002、11/11页、已选1及底部按钮；Linux最终dark重开截图路径框回顶部可见且聚焦。Windows/Linux各自420x560 narrow_visual通过，初始/滚后选中文件+底部操作/切换dark三截图已视觉核验；该scope不单独冒充full门。root Input/Checkbox/Button全走既有renderer/action。工具包生成Windows2/2、Linux2/2，runtime Windows5+10通过，旧dialog API定向6/6与modal/focus头less6/6通过；不以这些逻辑测代两平台真实窗口。Reader源码与Vue文件/字节授权链仍未改，八叶/SD-03明确待T-05，不宣称READER-009全量通过。
- 最终本地证据scope D:/autostack/plan-artifacts/plan-082/，SHA256：product-windows-final/report.json=21E3D4698FBBF599FEDFCAABD117AFA91ED03EEF73A08695F4FAA4A7F38BBD74；product-linux-final/report.json=45A1D36949BEB6D0CA223E968BCA490782195515769DBD39C025D365C05C08FC；system-modal-tests-final.log=4C63E374906AD23564DF1F98511548C28C5D05E8A1D30B3320983310436293C5；product-windows-final/system-tree.json=916C888BA72610CD788B9332A1B3C669CBD094527A69E564FDCA1796E92DB45D；package-generation-tests3.log=60DE334B9DBF35F27FA55E13794315CDDB9489B7B6F9230541AAB8C55EFD1F83；package-generation-linux.log=F04AEC0B977563E17E2AF9F5D61E0472E920855BAC2333A823819864B3D9003D。narrow两份report hash分别FA8B310D7A21D845A947A5E26EC246E25F07A0965AAF6E0628775343CE063566（Windows）、D0EB9A753405C62206B913437D7E14B64B97A02DDE651FAD36E2760744E4C625（Linux）。最终工作区clean；无自有原生验证进程留存。以上是work内检/任务checkpoint，不是独立final review。
- stage: goal | plan_id: PLAN-082 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass（T-04 checkpoint only） | worktree: D:/autostack/.wt/ui-082/auto-ui | commit: 949b7c01 | next: work T-05，先核Reader 009权威树/原owner写入交接


- 2026-11-12 T-05写入窗口：Reader权威树 D:/autostack/.wt/reader-009/auto-reader / codex/reader-009-dev，当前2370eca，tracked/untracked工作区干净；主检出旧tests/.reader_smoke.mjs与两份assembly仍按用户裁定不处理。应用列示无active Reader写入会话；原“完成计划009”和“继续完成计划008”会话已归档/notLoaded，已只读核其交接史。当前goal既有T-05全链授权下，由本会话在同一权威树顺序承担限定选择器接入/八叶生产者/SD-03写入；不另建Reader副本，不改009其他任务状态，不发未经用户明确授权的跨会话消息。Reader基线与原oracle先读后改；Vue保留File/字节token，不把原生路径暴露成浏览器授权。UI实现949b7c01冻结，当前无竞争writer/build。

- T-05中间检查：官方包新增 pac.at，支持现行 declared path dependency；工具包永久测试扩为3/3通过（package-dependency-tests.log），提交113c3765。Reader独立vendor路径避免加载无关Tree旧包，不改Tree解析器。组件源码SHA与上游逐字一致。八叶仍由唯一picker_session生产者输出，原混合/重复/取消/封面字节与预算oracle保留，并增加MD腿。workspace_contract实际必需名为W-03.picker-jpeg，旧聚合误映射picker-jpg；仅把既有producer key picker-jpg映射到该canonical叶，不改分母/语义/门槛。
- T-05真实反例reader-picker-04：mixed-batch与small-batch-budget通过（3格式精确受管字节/坏书无假记录，end-to-end 5.94秒<240秒）；第二批打开失败，四次重新发现仍Stale action identity。原始snapshot显示“导入书籍”却onclick=.CancelImport。调查定位projection同步先set_styled_vtree后update，新增action identity因此绑定上一帧View；守卫拒绝正确，非导入错误。修复为同一锁内先update当前View再发布同帧snapshot，保持旧row/action拒绝。plan082定向7/7通过，正在重建真实工具后重放；旧工具T04证据不冒充该修复验证。此前reader-picker-02 catalog元数据读错已改/books+catalog独立联合，原生original_path按samefile而非web file:token判定。三图片夹具Pillow实际解码有效。T-05尚未完成。

- T-05追加证据：reader-picker-07八叶自动数据门8/8通过（真实组件选取→业务API，约5.94秒首批预算），但截图复核不能当最终视觉pass：4px PNG只画细线，JPEG/WebP仍复用相同URL的PNG缓存。修正Reader详情显式w-44/h-64与libview三条cover_uri生产路径使用catalog独一受管filename版本query；新门加入Pillow实际解码颜色≥屏幕2%像素门及30秒有界重试，防[Image]占位假绿。
- Vue实际build揭露只生成SFC不足：依赖组件scaffold未齐、quoted-public-msg无local-handler时调用缺转发函数、内联List props自动key误推id、模板page算术误成page1。保留原生provider边界，按现有Vue语义补最终依赖组件闭包与缺本地handler的quoted msg emit转发；Reader过滤配置移model，组件displayPage移投影状态；新独立picker_mode双端port将nativebool/envelope与浏览器File端口分离，不更改文件/字节transport。一次尝试在.at补空handler吞掉VM父回调，reader-picker-08反例保留并已撤回空handler；须原生实际重放和Vue真实build后方可闭合。Windows运行中的工具锁文件导致一次构建os error5，后续工具按scratch副本运行，不能记作产品回归。

- T-05消费者checkpoint（整体仍4/8、未final review）：Reader提交a42b49bbdf416c90e950aab4ed99c063cdce0497，UI机制提交4e025c30528be67775824184922e6744e6d60476（另c9de726b仅补失败现场）。reader-picker-12真实原生8/8 PASS，三格式精准新书3本、重复后精准4本/alphaID不变、坏书无假记录、两种取消库/原件/受管hash不变；三图片HTTP/受管字节相等且实际屏幕匹配色各占4.4%>2%。JPEG/WebP改用新增完整不透明8x8夹具，旧不完整JPEG（双解码器颜色不一致）及全透明WebP仍保留并核原件未变，不把透明图背景当解码证据。三截图已目视核PNG蓝/JPEG红/WebP橙为完整封面面积，不再是细线或旧缓存。快照[Image]不带src故改用实际像素，不降低显示断言。当前两仓各唯一writer、Reader主检出脚本仍未碰。
- Vue：真实CLI generate3=0，正式npm build3=0（类型检查+Vite），reader-browser-final=0，真实Playwright FileChooser多选TXT/MD/EPUB→File/原始字节upload→独立managed bytes与file:name、取消前后冷hash、PNG naturalWidth4+HTTP源字节相同三项PASS；无MCP数据注入/直接import API预写。browser01首次代理仍默认17825造成读回超时，设置既有AUTO_HTTP_PORT到自有backend后02/final通过；不改用户全局代理或源阈值。新独立双端port只负责native模式/事件envelope，旧TS File/transport源码最终未改。依赖包源码SHA=0b1b2375c684c2a2f0ee7b4a6a0ef7ab21eda7f7dc1c67262cf4be2e82059729；vendor来源精确UI4e025c30。两页各命名book/cover_picker_filters，避免相同新字段在路由页消费中混淆；实际单选图片已验证。
- 本地hash（同scope）：reader-picker-12/results.json=11f2ec5045a55f586ac6b004c57745def8873e8fe4a1376b60fbffad4c8801a6；reader-browser-final/results.json=faee08da087fd8b1963dc5bc1489d9e9b8a6fae9fcba668207daf6048519dba1；package-final-tests.log=b6823c0f5ea5b960cf2a4aea318703ba09f9e326ccc23cfe6c0ade810c296719（3/3含真实existing-project增量刷新分支与quoted事件转发/scaffold）；reader-vue-build3.log=1a4edff9620a400dfa71c3823259954148bcb664acc787422fbc3b726427c4d8；t05-runtime-tests.log=a34efc35963205c4fc47c5685f037ad023b736bb14f3facfaef95807cc73998a（5+10）。这些是消费者scope，完整009两profile聚合仍待跑并据实列外域红，不宣称009完成。
- T-05通用组件重放发现未闭合：product-windows-t05第一次第5页连接reset；serial一次UIA选1后普通toggle未读到0；新增失败现场保留alive进程/当前snapshot。依赖source发现入口也补固定worktree roots，避免隔离app退回移动主检出。pinned重放在十轮取消后下一次Open无模态，但CLI/窗口/MCP仍活，原cancel计11。定位Tick仅每50ms采样open/seen_open，快速parent false→true被漏；既有unit刻意多发一次Tick掩盖。移去该等待后fast-reopen-red真实4/5（fixture用例红），保留反例。修复在provider每个宿主边界读当前open=false并即时复位seen_open/取消模型，拒绝过期open command；正在green5+10与重建，尚未把最终通用门标pass。Windows/Linux最终runtime新pin及完整Reader矩阵都待续。

- T-05快速重开闭合checkpoint：UI提交5bde71a9（当前权威树clean），fast-reopen-red去掉人为补Tick后4/5、green修复后5+10全绿，旧行/迟到验证/取消/owner丢弃断言仍保留。Windows product-windows-fast-reopen full_native_vm=pass，真实两个CLI/native会话、新选择/落盘及正常退出CLI0/所有自有PID消失/旧MCP拒绝与可重绑成立；首native PID27864/HWND2166990，fresh PID46472/HWND24907820，目录ready0.30174秒；十次快速取消重开、全部1003条11页与尾项、UIA正负控/背景MCP拒绝/键盘/筛选错误恢复全过。当前Windows UI tool SHA=adca3c7adf79326a903d0c21b417e21887104513fa21038a7bbb271830e0671c，精确源码5bde71a9；report hash=0f34a9081a81bb87e9fadb35710ccf518c37fa5ad88e9c294aac0d6bd5e70282；red hash=10821f828acadd465e65d35f6877c0cd7eb7c3287fb1fbfd0fb63550af9e712f，green hash=33d3c93910cc00cca8ae977b146541bf96994952040e430e794b3ba77b112f48。Linux新runtime工具已重建，真实full门在途。Reader完整矩阵vm→vue顺序在途（15913），含唯一八叶生产者，未预标结果。

- Linux最终生产pin（source runtime5bde71a9，后f67523d9只完善测试/规范说明）：product-linux-final-pin full_native_vm pass，真实X11/WSLg/tiny-skia两会话窗口PID634/727、XID各0x600002（新进程身份区分重用句柄），目录ready0.91926秒<2秒；JSON精确选择/单多选/旧row拒绝/1003全集尾项/十轮取消/主题/正常退出CLI0+所有自有PID消失+旧MCPconnection-refused可重绑+fresh会话新结果成立。Linux UI tool SHA=9e79aa4190e797c3e74b13c0a476c320dc8def080a12d83c74a42758573f4249，CLI延用bf8560b...，report SHA=82a615e41584d6c7749fba57257ba37d1f3529f43f30b5d9544a60c4090b7bc5。前一次full所有选择与两CLI0已过，但退出后一瞬仍见PID532导致红；失败清理检测CLI已退出，未强杀该子进程，随后ps证已自行消失。永久门改正常quit后最多5秒轮询“全自有PID实际消失”，记录起始剩余/耗时/最终空集；新门仍要求真实端口拒绝与可重绑/新会话，不以僵尸或超时冒充pass。
- Reader矩阵vm早期具名红（继续执行完整分母）：cached-view39 PASS后execution因HTTP ConnectionResetError10054、bulk-contract8 PASS后同类连接reset；各source_unchanged=true，尚未据此宣称选择器失败或009完成。read-contract26/26、transport30/30、view-pipeline15/15已产生真实结果。后续UI/旧门/picker/主题及vue档仍在途，待最终退出码/完整逐叶交接。

- T-06准备（矩阵仍在途，尚不execution_done）：已将主线93362c05同步到UI权威树，当前d18a4ef1；69f52f24..master实际差异仅PLAN-081文档190行，所有产品代码/Spec目标无新差异，因此已过代码证据不因该文档同步重跑。merge阶段按技能需线性rebase并记录feature旧→新patch等价映射，最终review会绑定规范化后的两仓pin。Reader已有84个未落主线的009提交，不把整条009分支无选择并入主线；082只交其消费提交/精确UI pin/SD-03到009既定沉淀流程，保留009全局未完成项。
- T-06内检待复演的边界（非已确认产品结论）：父组件显式open=false后在cancel回调前立刻open=true时，旧fd_event能否污染新会话；封面accepted业务handler在HTTP parked期间再次按替换按钮时，是否继承了旧PickCover的重入互斥。完成矩阵后用实际受影响路径检查，有反例则在原树有界修复并只重放受影响门；不拿八叶既过替这些生命周期判断，不预判为跨仓前置。


- T-05/T-06追加事实（2026-11-12 20:58，尚未execution_done）：完整Reader矩阵已终止，原始VM exit=3，147记录=55 PASS/82 FAIL/10 not_run，results SHA256=0672e855d1950194f4b9d1debb09779d8d3bf7fd7caa22206a945b0195e521a7；Vue exit=3，149记录=12 PASS/32 FAIL/105 not_run，SHA256=5fc6eda42161d8d596d6bad06e86f8b5010cfd6d89bbd826c5a46063f8763ef2。两聚合的八个canonical picker叶均PASS；producer picker-jpg映射既有W-03.picker-jpeg，修正文案不改required-set或门槛。VM首失败边界包括UI冷启动18.75秒超15秒及后续ConnectionReset10054（cached-view39绿、bulk8绿后断连）；Vue Rust backend生成首边界catalog.at类型错误，后续未执行如实保留。source_unchanged=true；全009未通过，结果交009原owner后续处理。本轮只复演082受影响路径，不因聚合红改阅读/EPUB/theme业务。
- T-06生命周期真反例与修复：父件Close→实际view(false)→Open而无中间Tick，5bde版本仍保留已选1；9d6f7c7e把render观察到的false边界保留到provider处理，并在SDK新会话清旧fd_event。永久root_parent_close_reopen测试要求新会话0项、旧行拒绝、当前行可选、cancel恰一次；component6+model10 PASS（root-parent-edge-tests2）。9041efec对row_id中路径做JSON编码；旧共享消息codec将含U+001F文件名拆成另一普通文件token，red真实失败；新message_identity_tests经真实encode/decode roundtrip且不等另一token，1 PASS，component6+model10 PASS，不改全局codec/语言ABI。
- T-06封面重入：cover-reentry-red在accepted后HTTP未完成时再次按原替换封面，真实第二个modal出现。Reader补cover_updating guard并禁用替换/清除，业务完成/失败才恢复。reader-picker-candidate2全部8 PASS，PNG/JPG/WebP三次busy_reentry rejected=true，disabled控件普通press确实回No 'press' handler且无modal，随后合法换封面仍过。可见蓝/红/橙封面像素各占截图4.4%，独立HTTP/受管字节一致；JSON结果DTO改标准stringify以保留控制字符路径，浏览器File/字节传输源码不变。
- T-06通用复用反例：两个不同父件同名picker_filters各默认PICTURES/png与BOOKS/txt，真实App+package+实例timer构建显示右件被左件PICTURES覆盖（parent-config-control.log）。根因legacy root包含子件模型副本，prepare_child_render_state step3将副本当父频道。正在按实际父对象读取，root只继承根声明/无view store频道；不再从任意root同名副本播种。永久不同父类型测试要求左右各精确文件集合，parent-config-fix-tests2 component7+model10 PASS；相关child/Init回归及最终两平台工具重建在途。以上修复仍属AC-01/03/05，未变r1合同；当前未知阻塞为空，但受影响最终产品重放未完成故不交final review。

- T-06 parent配置修复的兼容对照：`cargo test -p auto-ui-runtime --lib child --locked -- --test-threads=1` 首次79 PASS/2 FAIL/1 ignored；严格恢复修复前HEAD源于同树重跑两失败，p536_t8_child_widget_root_alert_dialog_resolves_open baseline1 PASS，f5_embedded_fullstack_grandchild_viewport baseline1 FAIL（n=0）。因此前者为本次回归，后者为既有红，不混淆。前者在首render前通过旧type-only入口写root，新增显式legacy root receiver登记，只允许该目标类型保留既有root接收语义，普通实例仍只继承实际父频道；正在重放相关child集及17项组件/模型。全部baseline源已finally恢复，未提交或遗失修复。Linux初版parent修复full产品门PASS，report SHA=e32e4b02c7e94db4b18ae6cf0e066df748df23f26895d2773088c585c278e9a6，tool=21547c685eaf98ac00f98dba65d471f19e4fe068430bb23ee104e8f466ea7d92；因刚补legacy兼容仍须最终工具及受影响产品复演，不把该旧候选当最终pin。

- T-06最终构建及规范化：UI旧tip 0ebbc5d7fdb81195758714b66a3e72dc2c4ea4e8 → face42c86028bc517a3ef84aa078f0c3b2f2a4c3，rebase master=b926fab4816e9a58a65a05926d28667455746e73，13个feature补丁range-diff全部=（SHA256 fe38ac2f4c53de996ac7209a6ec2ac02611a2182eab5f88004cae218c0da2827），d18 merge节点移除；old/new完整树差仅后续PLAN-081文档32行，产品代码/规范无差，构建和已过门patch-equivalent。映射：
  - f8fbb1e12d4ad4e29b2d70d7084137deb73490ab → e8ea4fbdacb5eeebb235ffa3927f2a84dc9732a5
  - 2b5e98ccbce2fbc221c0caebeb921c4716ab1642 → 4e74f845a6113ef5f9930cd80955645403c2081d
  - ba97656de5753629730168b9b5c0faf75689d01e → caa893bcf091d080417f222b442c809d2be86edc
  - edfa4695a11cda998b06fd229b0addc6865a3409 → 0c0d74143a09ff955b2c755d8645b55966db7d6e
  - 949b7c0152aa4f64ab3a50422b6c7c48c4722781 → f5d58a426df522097a9f479dbcb6e2b2f8ea1fa2
  - 113c37659e054cb7e42c9605e527b0f321f262af → be05f70ec7ccbdca682b8e194bf044e28c2e88a5
  - 4e025c30528be67775824184922e6744e6d60476 → 0246b30434b96e7e8e4a47fe8cf7d35115aaffcf
  - c9de726b5a7b85fdfff0462a556cda8eda7d50cf → fc20a3f432ccf4572d38da53911c833625ed92a6
  - 5bde71a968036286384948bc31de630f742606b9 → 6fa38a4adcd069641dea025fb2bb862477ea0b79
  - f67523d9ed32be8ee36a7040ff24493c98135fc9 → 94397b331f531a0d20456266d430610648995fdf
  - d18a4ef1a21daf5b25fcbe01b50ed4ae6c19f794：旧 merge 节点，规范化时移除，无 feature patch 对应
  - 9d6f7c7e7e85795e6fbd5de94623097ea4f114b7 → ea32c0cd3a116de1727edff44f54134a22ed984d
  - 9041efec47087a9e7d73580f9e6bfacd486e24ca → 9be6069812dd9e14e745a1a07d4445f1725fb634
  - 0ebbc5d7fdb81195758714b66a3e72dc2c4ea4e8 → face42c86028bc517a3ef84aa078f0c3b2f2a4c3
- 最终源：UI face42c86028bc517a3ef84aa078f0c3b2f2a4c3，Reader e6090b1345f063129283b8fb55c423f50f875f56（a42消费+e609修复）；两树clean。SDK包ea32c0cd3a116de1727edff44f54134a22ed984d，vendored字节hash=71b1092136ef3379891a520ceeaaFe3b91aA4ebd36e3a9c111ebb9999b0b6fc4（以实际README小写为准）；README精确host/cohort/CLI及009交付方式齐全，未推进009旧兄弟依赖树。child最终80 PASS/1既有FAIL/1 ignored，新增回归已闭合；组件7+模型10 PASS，log SHA=2728daa3331530b27648ec30df5ad5348f6449c85c1bcf9e576d7a6ef5a7f1b9。
- 最终Windows/Linux full_native_vm均PASS：Windows product-windows-final-082b/report.json SHA=d345be0cda81e667fa23cd5ea602fca2c7534a9fa1570b620540beb19be73403，工具55813b8da540d896f1985199068f4821e2eb7c218b85e3630deb05c426c9afba；Linux product-linux-final2/report.json SHA=c4427708c81ab6417287881a8851941b5531b6476965a1bd1b67d6bd576404d6，工具ffef5bc0f131b2bf33ad46ed03bff631c0b3c0fc935189ffb0686d44632c2806。两平台原生真实进程/窗口、精确目录全集1003条11页和尾项、10取消重开、过滤/错误恢复/单多选、同名父目录身份、正常退出CLI0/所有自有PID消失/旧MCP拒绝且可重绑/新进程重新接受全成立。Windows本轮首run在最后Quit回复timeout，选择阶段均已完成但整轮FAIL，不抹去；相同冻结源码/工具且无并行原生操作的082b整轮PASS，未延长超时或改oracle。Linux使用真实WSLg/tiny-skia，不是跨编译替证。
- Reader Vue真实CLI incremental generate2 exit0、vue-tsc/vite build3 exit0；reader-browser-frozen2三项File字节导入/取消/封面解码PASS，SHA=7a47d30e762d8b02ac5a41e0877094e1bbc232f4de0a3398ada3ac437ae9a2df。前一个browser-frozen未显式设五仓roots，错误cohort启动服务后断连，保持FAIL记录；恢复约定roots后7秒全PASS，代码/阈值未变，新增结果meta记录五仓roots/子进程退出前状态，防再次遗漏。最终Reader原生8叶重放在途，待完成后才execution_done。


- T-05最终八叶：`python tests/product/picker_session.py --auto-bin D:/autostack/auto-man/target/debug/auto.exe --ui-tool D:/autostack/plan-artifacts/plan-082/ui-tool-final.exe --out-dir D:/autostack/plan-artifacts/plan-082/reader-picker-final-082` exit0，所有五仓roots显式固定，8/8 PASS；mixed/budget/duplicate-bad/book cancel/PNG/JPEG/WebP/cover cancel全部保留原oracle、180/240秒门槛。三种封面均另验busy按钮不可重开，正常后续选择恢复；目录/API/受管字节/原路径与截图真实像素齐全。结果SHA256=805f3b4e9e255674f2ebaed2bdce8fd0ab1b07292325f7879b7e2ee50a869e2b。Reader测试主体e6090b1345f063129283b8fb55c423f50f875f56；最终1999275447a852aaa7ccfcab8fd2a4552f14f0d3仅README hash说明，业务/test/spec Git blob不变。同步vendor为上游canonical LF，原CRLF与LF逐字节归一比对相等，无DSL变更；三份vendor与UI package完全相同。规范化SDK canonical SHA256=26bd598e3242ecd0aedbe7aba535d50395b8a5f9f91704a541dc47b1cad05276；前述71b109...是原CRLF工作文件hash，不是canonical blob，不再作精确vendor源hash。

- stage: work | goal_round: 8 | plan_id: PLAN-082 | plan_revision: 1 | outcome: pass | task_ids: T-05,T-06 | worktree_path: D:/autostack/.wt/ui-082/auto-ui / D:/autostack/.wt/reader-009/auto-reader | branches: codex/plan-082-dev / codex/reader-009-dev | UI code_commit: face42c86028bc517a3ef84aa078f0c3b2f2a4c3 | Reader code_commit: 1999275447a852aaa7ccfcab8fd2a4552f14f0d3 | Reader scoped base: 2370eca1545（完整精确base可由a42b49bb父提交核验） | dependency_revisions: lang=f9fb3854beb3886338156cfa61d16f7f84893cc9, web=8b1742a0f84c24173768265cb999a648039d1e2b, down=58ea33f699efa5cb1194473a2d45dcec51006842, man-source=54fe4070bdbd06015ea4521074479ae2e2f8b14f；CLI/tool binary身份见上文，不混同source pin | blockers: none | next: review / 独立上下文 T-07，实施写入暂停。

- Readiness / AC evidence map（work-owned）：
  - AC-01 pass：官方package+实际独立fixture、包加载/VM与Vue生成/旧provider6旧门（T04）、当前两平台full产品门；公开事件通过routed实例回调，Reader不是必需依赖。
  - AC-02 pass：provider/model10项精确集合/过滤/隐藏/导航失败保留/确认原子/结果路径，当前两平台普通控件真实单多选、同名不同父路径与千条尾项；未降低门槛。
  - AC-03 pass：慢请求/取消/owner丢弃/迟到验证原反例、root无中间Tick close→reopen、U+001F真实codec不串文件、不同父件同名配置隔离；当前7+10 PASS，相关child80 PASS、既有f5红单独对照。终态不重复由真实cancel计数和模型覆盖。
  - AC-04 pass：当前Windows系统TogglePattern正控、背景Invoke/MCP负控、目录Edit值/实际bounds、普通MCP字段与焦点/键盘；两平台1003项全集11页及末项确认。T04窄屏420x560明暗图/UIA断言复用：布局/主题/系统modal实现未被后续生命周期/token/父默认播种改动替换，当前两平台全门同时覆盖现布局。
  - AC-05 pass：最终Windows082b/Linuxfinal2均真实本机native，取消重开10轮、两CLI/native窗口会话、正常退出和PID/端口负控全部过；Windows一次Quit回复timeout整轮红保留，未修改oracle，后续同pin整轮绿。
  - AC-06 pass：Reader最终8/8+三封面busy负控；Vue generate/build和浏览器File字节导入/取消/真实解码3/3；完整VM/Vue矩阵真实exit3及外域红已列明，不宣称009全局过。Reader README将消费commit/精确UI host+包+cohort/旧兄弟工具未升级及SD-03交给009既定owner流程，不合并其84个其他提交。
  - Open work-owned findings / coverage gaps / contract decisions: empty。仅AC-07/T-07 review与AC-08/T-08 merge未完成；本上下文内检不当独立pass。统一所需门当前两平台、八叶、Vue/browser已有同语义冻结版本证据，复审无需按角色切换再跑全009或语言全仓。
  - Frozen canonical Spec delta：UI docs/specs/ui/file-dialog.md Git blob SHA256=8fa25b4b61da7ddb25159830ebbdaca078a5b3f84f2bd16a32727d0febb899d4; Reader docs/specs/reader/library-workspace.md Git blob SHA256=86a086a6fddd5f583e1f741d1dfe013f6af0340444934b62d314d4e0046bb0ba。SD-01/02新增UI一文，SD-03修改Reader文；supersedes/new metadata已按实目标登记，touched_goals空因无既有goal源条目需替换。无legacy ledger/index写入。


- stage: review | review_scope: final | plan_id: PLAN-082 | plan_revision: 1 | outcome: pass | reviewer_identity: 独立子上下文 `/root/final_review_082`，受 T-07 同树顺序复审委派；本上下文未参与实现/修复，未派其他代理，未写实现/Spec；工具未暴露模型/会话编号，不作推定。limitations: Windows代表路径由本轮新建夹具真实重演；Linux整门/窄屏主题/旧API/完整八叶/Vue沿用指纹与配置未变的原始证据，经读取源码断言、trace、报告和截图独立核验，不因角色切换重跑完整009矩阵或全仓门。review skill SHA256=3d9263331895dd09cff5015783d9fc21b8be8441505d4101d52f8197d6d2a1f7。
  - worktree_path/branch/reviewed_commit/base_commit：UI `D:/autostack/.wt/ui-082/auto-ui` / `codex/plan-082-dev` / `face42c86028bc517a3ef84aa078f0c3b2f2a4c3` / `b926fab4816e9a58a65a05926d28667455746e73`；Reader `D:/autostack/.wt/reader-009/auto-reader` / `codex/reader-009-dev` / `1999275447a852aaa7ccfcab8fd2a4552f14f0d3` / `2370eca1545cdd3999ce84f42c230861e0dfd0a7`（a42b49bb父提交）。本轮前后两树clean、HEAD不变，worktree list验证归属；Reader只审a42/e609/199三提交，未把009其余84提交算作082。
  - dependency_revisions：lang=f9fb3854beb3886338156cfa61d16f7f84893cc9，web=8b1742a0f84c24173768265cb999a648039d1e2b，down=58ea33f699efa5cb1194473a2d45dcec51006842，man-source=54fe4070bdbd06015ea4521074479ae2e2f8b14f，兄弟树均clean。独立产品回放显式固定ui-082五仓roots；Windows tool SHA256=55813b8da540d896f1985199068f4821e2eb7c218b85e3630deb05c426c9afba、CLI=e762cec894a52f94ebb16da3499b7889ed8861ff8df4633418c31034d1022385；Linux工具ffef5bc0f131b2bf33ad46ed03bff631c0b3c0fc935189ffb0686d44632c2806，均实际重算一致。13/13 range-diff为等价；订正上文手工映射末三项：旧rev-list含merge节点导致zip错位，raw映射始终正确，无源码变化。
  - acceptance_results（AC → tasks → 实际链/证据）：

    | AC / delta | 结果 | 独立检查与覆盖边界 |
    |---|---|---|
    | AC-01 / T01,T03,T04 | pass | official包→实际VM实例→routed callback；三份vendor与上游Git blob逐字节一致。读包加载/双生成器3条及旧provider6条实际断言/非零结果；compiler无runtime反依赖改动、旧OS API未替换。 |
    | AC-02 / T02,T03,T04 | pass | 新C01路径/同名不同父目录/Unicode空格引号精确accepted；新C04选中文件删除→not_found、原持久accepted字节不变且选择保留、恢复后整批精确accepted。模型10条本轮全过，核过滤/隐藏/非法类型/失败保留/列表排序/原子确认断言；Linux非UTF8/坏链接/真实拒绝读原断言及PASS。 |
    | AC-03 / T01,T02,T06 | pass | 本轮C03旧目录ID、取消重开旧session ID均拒绝，新行正常；7条组件测试+10条模型测试exit0，无skip，含慢A/快B合法对照、迟到验证、owner drop、同名父配置、不同App回调及无中间Tick关闭重开；检查receiver替换、实例销毁/回调清理与token真实codec。 |
    | AC-04 / T03,T04 | pass | 新C05背景业务动作拒绝、Esc取消/重开、Tab/ShiftTab、关闭后背景可用；沿用两平台1003条全集11页精确set与尾项及Windows系统Toggle正控/背景负控。亲检窄屏footer图、Linux尾页图：底钮可见可达；保留主题/焦点证据。 |
    | AC-05 / T04 | pass | 复核Windows082b与Linuxfinal2的真实句柄/PID、工具身份、MCP调用原始trace（两session分别231/25与661/20条）、退出reap remaining=[]和新会话正控。目录ready 0.263/0.863秒，取消重开10轮；Linux为WSLg真实执行。Windows早先Quit超时红未删或算绿。 |
    | AC-06 / T05,T06 | pass | 本轮C06新隔离空库，普通控件选择自建TXT+坏EPUB，精确仅1本、原路径/受管字节相同且坏文件具名失败；PNG普通选择→HTTP字节/冷文件/可见解码（4.4%像素），busy重入无handler且无modal；书籍/已有封面取消均冷hash不变。原8/8结果/源断言及三封面busy反例齐全；Vue真实build和File字节导入/取消/解码3项保留，picker-jpg仅映射canonical jpeg不改分母。 |
    | SD-01/02 | pass | 新UI canonical文与公开包/模型/异步实例/MCP模态/平台边界一致，未把浏览器生成误写为本机目录能力；生命周期/主题/NVM既有源未改。 |
    | SD-03 | pass | Reader规范描述原生事件消费、浏览器授权、busy/版本化封面与八叶边界，保留30MiB/8MiB与业务责任；不宣称009整体完成。 |
    | AC-07 / T07 | pass | 独立上下文已完成上述代码/Spec/原始oracle核查及代表性真实路径/反例重演。AC-08/T08仍pending。 |

  - spec_inputs（Git blob SHA256）：实例=5ba3a663f74df551633925ee3f7588c913b500c65a5547feb8c0c68c1ec28b3d；主题=703da9a3f06af73de5d4dd7b0b99c9468f31beac554df7124deff8dc511132a5；NVM=a20e599f2d2494badc3eb88486ec86bb8912b88e7bf9e25e97747b52be5953e2；structure=a5dba7bc48d44742c8a55348b2ef96c0077848c48226711dd1d759db13b0ae25；SD-01/02 UI=8fa25b4b61da7ddb25159830ebbdaca078a5b3f84f2bd16a32727d0febb899d4；SD-03 Reader=86a086a6fddd5f583e1f741d1dfe013f6af0340444934b62d314d4e0046bb0ba。冻结before/after副本 `D:/autostack/plan-artifacts/plan-082/review-spec-delta.txt` SHA256=6e2454bd66750b1d93e34f09ce85d04cc1bacd9a40b4441887e8e5384680601e。metadata supersedes/new目标正确，touched_goals空有既述理由；两仓scope无legacy ledger/index写入。
  - evidence：本轮 `python D:/autostack/plan-artifacts/plan-082/review-ui-replay.py` exit0、`python .../review-reader-replay.py` exit0；`cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model --locked` exit0（7+10 PASS，0 ignored/filtered）。UI result SHA256=a20cfae2ed25912652bd602a71068a8272e466dbfca3c74017c0fea2be97c65b；Reader result=a85df00038ab7f2da5ecace79ede3b24456a97665c6438ea49cfb952e35cf02f；raw traces分别3451f60becea96dcf4a4099a652ee93ad70dac504e52a969cf1592738b14be57 / cdf6de4e51067cf16fc4b745e3350297659b3a3eb107fbfaff9fe5ef96c2583a；model log=ae1220f042f796ad60fc687ef3b5b11652fd7625f1732322fb18a0ab9590deff。路径均在同scope下 `review-ui-replay/`、`review-reader-replay/`、`review-model.log`；完整指纹/复用raw核验 `review-audit.json` SHA256=2a4e1662f01ab6563cf63c4896529fcd37fed118378be35515bea33bdaac60ef。
  - findings: 无未关闭082合同内finding；current blocking question=none，§10旧入口待处理为已解除历史，不要求移动Reader脚本。related child集80 PASS/1既有f5 FAIL/1 ignored及严格修复前f5红对照已核，不伪绿；完整Reader VM/Vue聚合exit3继续开放给009，分别55/82/10与12/32/105，不以082八叶替全局结论。
  - pending_closeout_gates / next: merge T-08。当前同语义冻结版本的双平台full_native_vm、Reader八叶、Vue build/browser证据可复用；若实现/依赖/配置漂移须重审受影响项。主线后续变化截至314ce78b9fc564bd31a01a9d76e8c4f293660470仅081计划文档，冻结review HEAD未动；merge若rebase至新master须记录range-diff/产品树等价，并将Reader README host/package pin仅作doc-only delivery descendant校准，不涵盖未来语义改动。合并须先精确交付UI、按009流程交付仅三条Reader消费提交及相同host/cohort pin，完成canonical规范核验、耐久回执/归档/自有scope清理；不并入009整分支，不触碰其主目录忽略脚本和其他WIP，不宣称旧部署已更新。本次只将Plan置reviewed、T07勾选、7/8，不进行合并/部署/清理。


- Goal round 9: stage review | outcome pass | plan_revision 1 | reviewed_ui face42c86028bc517a3ef84aa078f0c3b2f2a4c3 / reviewed_reader 1999275447a852aaa7ccfcab8fd2a4552f14f0d3 | 独立/root/final_review_082完整回执见上，T-07已核7/8；next merge。
- stage: merge | goal_round: 10 | plan_id: PLAN-082 | plan_revision: 1 | outcome: in_progress (已交付/规范核验，待归档提交和清理) | prepared: final pass绑定版本、两树clean、scope与delta授权均核验；无实现新改动/无需重跑无关门。
- landed UI：`git rebase master` base=314ce78b9fc564bd31a01a9d76e8c4f293660470，reviewed face42 → delivery dad0514fcba24452d6f19217b12e29189698f772；13 feature补丁range-diff全部=，old/new树仅081计划文档差异、产品/Spec等价；`git merge --ff-only codex/plan-082-dev` exit0，主master当场精确等于delivery，无merge commit。第二次落地映射（替代任何旧pin指向，补丁语义未变）：
  - 1:  e8ea4fbd =  1:  d8b1b7e8 feat(file-dialog): prove async package selection and routed instance callbacks
  - 2:  4e74f845 =  2:  e111003d fix(runtime): expose non-Windows renderer shim helpers to host
  - 3:  caa893bc =  3:  93370394 feat(file-dialog): own filtered selection and async validation per session
  - 4:  0c0d7414 =  4:  028b2347 feat(file-dialog): complete native controls and modal input boundary
  - 5:  f5d58a42 =  5:  46080515 test(file-dialog): verify native lifecycle on Windows and Linux
  - 6:  be05f70e =  6:  8937ca21 feat(file-dialog): support declared vendored package dependencies
  - 7:  0246b304 =  7:  703f9518 fix(file-dialog): publish current actions and refresh Vue dependencies
  - 8:  fc20a3f4 =  8:  5732fd29 test(file-dialog): retain live process state on product gate failures
  - 9:  6fa38a4a =  9:  bef7ebde fix(file-dialog): retain fast parent close and reopen transitions
  - 10:  94397b33 = 10:  2f9e46da test(file-dialog): bound normal child reaping before restart
  - 11:  ea32c0cd = 11:  a1bb6917 fix(file-dialog): preserve rendered parent close boundaries
  - 12:  9be60698 = 12:  a32e038c fix(file-dialog): keep filename controls out of message framing
  - 13:  face42c8 = 13:  dad0514f fix(runtime): seed child models only from their owning parent channels

- Reader scope交付：消费base=2370eca1545cdd3999ce84f42c230861e0dfd0a7，实施a42b49bbdf416c90e950aab4ed99c063cdce0497 / 修复e6090b1345f063129283b8fb55c423f50f875f56 / canonical-LF说明1999275447a852aaa7ccfcab8fd2a4552f14f0d3；doc-only delivery descendant f6ab049ab65e16de0f84a900a84a38400ee3e05f仅将README host置UI dad0514fcba24452d6f19217b12e29189698f772、package置a1bb69178b864a0b24afdbfa320977ee20b47545。逐文件diff证明业务/tests/Spec不变，vendor三个文件与落地主UI package字节完全相等，SDK SHA26bd598e3242ecd0aedbe7aba535d50395b8a5f9f91704a541dc47b1cad05276。交付到既有009分支/权威树，按r1 T08原owner流程消费，不将其此前84提交无选择并入main，不动009主检出脚本、其他未完成项及原owner树；此处“交付”指本合同的009消费交接，不宣称Reader主线已合并/009全局完成。
- canonical_verified：UI主master docs/specs/ui/file-dialog.md Git blob SHA8fa25b4b61da7ddb25159830ebbdaca078a5b3f84f2bd16a32727d0febb899d4，与独立review冻结一致。Reader009 docs/specs/reader/library-workspace.md Git blob SHA86a086a6fddd5f583e1f741d1dfe013f6af0340444934b62d314d4e0046bb0ba，与review冻结一致，作为SD03已提交交009沉淀；未冒称Reader主线canonical已更新。无index/ledger写入，缓存状态不作源交付门。
- Landing smoke：`python D:/autostack/plan-artifacts/plan-082/landing-smoke.py` exit0，实际AUTO_UI_ROOT=D:/autostack/auto-ui主线，其他四roots保持受验cohort，原生窗口MCP选主线官方包→精确a.txt持久路径→重开取消→正常退出CLI0/自有PID尽/旧MCP拒绝/端口可重绑。落地runtime/tool与受验实现patch-equivalent，结果SHAbfec514c907c9b3a3202679b5b6ed18b228124b044d4953020126dcf5a5f38e8；源码/Spec与pin校准后状态clean。归档后只清理ui-082自有五仓工作树/临时branch、注册plan-082 scope；Reader009树和其他任务树保留。
- Evidence ownership/retention：本goal所有已知scope文件已对§9命令/日期/产物核归属，补登记D:/autostack/plan-artifacts/.retention/policy.json为auto-ui/PLAN-082 in-flight；未知scope不删除。原matrix snapshot逐PID+创建时间核无残留；复审和landing窗口已各正常关闭。所有需保留的正反结果/矩阵红/哈希/回放方法均在本Plan耐久摘要，无需raw hold；清理时删除全部plan-082暂存，不重建附件/index。
- Deployment observation（source landing非部署，保留当前运行服务）：D:/autostack/auto-ui/target/debug/auto-ui-tool.exe 最后构建2026-11-11 04:43:22，为旧产物，未覆盖/重启；主UI target/release/auto-ui-tool.exe不存在。D:/autostack/auto-reader/gen/front/vue/dist/index.html旧bundle日期2026-11-06 04:47:30，Reader主release/debug可执行文件不存在；Reader009旧UI兄弟源码9dd3fc27f5d8fbd4714a5c8aff1a5d9a50379608且无target/debug工具，未升级。Reader009本次gen/front/vue/dist/index.html日期2026-11-12 21:05:30，经浏览器3项实证，为开发验证构建；正式部署仍由009 owner按README精确源cohort重建/显式roots执行。其他仓无本计划产品源码改动，无依赖daemon重启。auto-lang未落源提交，非该仓batch触发，不另跑语言全仓回归。
- archived checkpoint准备：completion_kind delivered；T08仍未勾、current_step7，因自有scope/树清理未完成。归档文件在默认分支提交后才记archived已验证，最终清理后回填8/8并再提交；不会将旧WIP或未提交docs视为归档完成。


- stage: merge | goal_round: 10 | plan_id: PLAN-082 | plan_revision: 1 | outcome: pass | completion_kind: delivered | delivery_ui: dad0514fcba24452d6f19217b12e29189698f772 (master ff-only) | reader_scoped_handoff: f6ab049ab65e16de0f84a900a84a38400ee3e05f on codex/reader-009-dev | canonical_verified: SD01/02主UI Git blob8fa25b4b...899d4、SD03既定009交接Git blob86a086a6...6bb0ba均等于独立review冻结，最终复核未变 | archived: docs/plans/archived/082-file-dialog-component-and-reader-automation.md，归档提交d75e56f41be3bfcdbd874697b235d573bdfc3853已在默认分支，非untracked假归档 | cleaned: verified。
- 清理实际结果：立即删除前逐树绝对路径限定D:/autostack/.wt/ui-082、git status clean、wt-guard clean；auto-ui/auto-lang/auto-web/auto-down/auto-man五树均由对应repo `git worktree remove`成功，UI已合并branch `codex/plan-082-dev`已删除，空ui-082组目录已删除。所有target/pdb/exe中间产物随树删除，不复制回存。Reader009权威树/其旧兄弟和其他任务树未移除，HEAD f6ab049 clean。
- 暂存清理实际结果：scope wt-guard clean/注册归属/holds=[]均核验。最初变量路径的批量强制删除命令被自动审批拒绝（仅返回blocked by policy），未绕过审批；改用已核明确字面路径、不强制的Remove-Item获准执行。仅剩本任务为隐藏文件测试创建的input/.hidden-book.txt，恢复Normal属性后同一不强制删除完成。最终Test-Path plan-082=false、ui-082=false、开发branch查询不存在。全部raw证据已按本Plan摘要折叠，registry改deleted-at-merge/no holds；无未解决raw hold。
- 最终实际状态检查：T01..T08全勾、current_step=total_steps=8，status archived；Plan已跟踪、三个本地Markdown链接解析有效，UI delivery为master祖先且其产品补丁线性，Reader业务/tests/Spec与review版本相同，仅README指向等价落地pin。无后续实现、测试或发布前置由082欠付；已记旧部署观察和Reader009外域红交接。主检出其他设计/计划/.next-id WIP全部保留，仅提交本Plan归档与本次receipt；不以整工作区clean为由丢弃他人改动。next: done。

## 10. 待澄清事项

无必须让用户补充的首版产品问题；按本合同的简版打开文件窗口执行即可。实施技术待定由T-01负责：实际 `.at` 回调/provider能力、性能预算/分页与Linux执行环境。它们有明确产物和依赖门，不以重复询问用户“是否同意已要求的功能”替代调查。若Linux资源缺失则AC-05保持待验；若发现必须扩展语言或浏览器任意目录授权，则提交具体前置/范围变化，不能默默宣称已支持。


### 2026-11-12 执行入口历史（已解除）

用户全链授权已收到。当前需由原 owner 将 Reader 主检出的 `tests/.reader_smoke.mjs` 路由到所属工作树，或由用户授权备份到仓库外后继续。其他未提交文档、assembly 产物及 UI deps/dist/build 保留原状，不静默删除、暂存或合入。等待期间不执行实现、构建或产品检查；T-01..08 均未完成。解决入口后再建 ui-082 权威实施树，继续 T-01 的技术可行性、Linux runner 与 Reader 009 写入交接调查。

用户已选择由原任务负责人处理上述测试脚本；本会话不移动该文件。处理完成后核验主检出状态并继续 T-01。

2026-11-12 最新裁定：上述脚本无需处理，不再阻塞 UI 实施。UI 权威树已创建于 D:/autostack/.wt/ui-082/auto-ui；Reader 写入前在 T-05 再核交接。
