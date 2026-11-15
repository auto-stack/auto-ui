---
plan_id: PLAN-085
status: executing
feature_name: TreeView 正式组件与示例复用
author: [Codex]
created_at: 2026-11-13T00:00:00Z
updated_at: 2026-11-15
plan_revision: 2
current_step: 3
total_steps: 8
supersedes_spec_components:
  - docs/specs/ui/examples-ownership.md
new_spec_components:
  - docs/specs/ui/tree-view.md
touched_goals: []
---

# PLAN-085 TreeView 正式组件与示例复用

## 0. 变更摘要

将 018 Reader、026 Database、027 File Manager、041 Editor 中的受控 TreeView 收成 `packages/tree-view/` 正式包，保留稳定节点 ID、外部 expanded/selected、展开与选择分离，补齐键盘、加载/错误、MCP 和真实双端验证。复用既有 TreeIcon 和 flatten_tree 路径，不重新实现一套树或文件系统浏览器。

路线：[组件库扩展 strategy / CL-02](../design/autoui/component-library-extension-strategy.md)。本轮只建立合同；无代码、构建或运行结果。

## 1. 目标

一个树包供文件、目录、章节、数据库对象等导航使用；应用仅提供节点和业务回调。首版支持受控单选、展开/折叠、目录显示模式、稳定寻址、键盘导航、加载中/加载失败/重试的普通控件、空态和主题。

Windows/Linux native VM 与 Vue 的选择/展开/事件/禁用语义一致。四个本仓示例使用正式包并保持原业务交互。同页双实例、更新节点和懒加载响应不串扰；无人值守操作与用户使用相同控件。

非目标：多选/checkbox 树、拖放重排、编辑节点名称、通用文件系统服务、数据库请求、无限规模/虚拟化性能承诺、改语言语法或搬迁 blueprint owner、macOS 支持认证。

## 2. 架构方案

应用节点/expanded/selected → 纯派生可见行 → TreeView 渲染；点击/键盘/MCP → 带稳定 ID 的回调 → 应用更新受控状态。组件仅保留焦点/导航所必需的实例局部状态；不复制业务选择或 expanded 集合。不能为补键盘而把四示例各留一份算法。

原有 `flatten_tree` 通用工具的 canonical owner 是语言仓 `packages/blueprints/navigation/filetree/tree_util.at`。T-01 比对四例差异及真实依赖解析：优先使用现有 canonical 工具；若新增 UI 交互投影，可由正式包持有 UI 适配，不能拷贝一份通用树工具作为第二真源。若必须改语言公共工具，需提出具名跨仓依赖，本计划不擅自写语言仓。

既有包和内置注册规则继续使用。TreeIcon 仍通过字面量图标分支保证 Vue 能生成，不假定动态 icon name 已可用。没有文件 IO 的树不依赖 FileDialog 或 Pagination，因此可独立实施。

## 3. 技术栈

`.at` 受控 widget、现有官方包导入/生成链路、Rust compiler/runtime 的必要通用键盘/语义接线、VM/Iced、Vue、主题/MCP。优先普通 `.at` 组合；若通用键盘或树语义缺能力，T-01 定位最小引擎扩展，不能加测试专用入口。

## 4. 需求分析与背景调查

### 授权、基线和现有实现

用户授权首批三组件分别建计划、复用现有实现、保证跨平台/MCP/无人值守测试，并更新 strategy/设计；本轮仅改 auto-ui 文档。无代码实施、其他仓写入、发布、模型指定、预算或创建子会话授权。

观察 HEAD `4dd64b5fc5fc0919bfe686115c75a5d8c7ac06b1`。`docs/specs/README.md`、`docs/specs/ui/examples-ownership.md` 定义本仓示例单源和 blueprint 依赖，不存在当前正式 `tree-view.md`；新 Spec 应在交付时建立，不能用旧注释代替实现验收。

| 已核路径 | 观察 |
|---|---|
| `examples/ui/018-book-reader/src/front/components/treeview.at` | 受控节点/expanded/selected，使用 bps.navigation.filetree.tree_util |
| `examples/ui/026-database/src/front/components/treeview.at` | 同族实现，use 本地 tree_util；SHA256 `31BB0990910C124F575E380561EFD5B467AC0E5B4421678FD70BE3F322934335` |
| `examples/ui/027-file-manager/src/front/components/treeview.at` | 同族实现，本地工具依赖；FileTree/TreeIcon 需要随抽取核对 |
| `examples/ui/041-auto-edit/src/front/components/treeview.at` | 同族实现，使用语言 blueprint；应用工作区 IO 仍由 Editor 持有 |
| 上述组件 props / view | `nodes: List, expanded: List, selected: str, on_toggle, on_select, directory`；computed flatten_tree；chevron 和标签是兄弟命中区，防止 Vue 冒泡触发两次 |
| `D:/autostack/auto-lang/packages/blueprints/navigation/filetree/tree_util.at` | 通用工具的现有来源，作为只读依赖；不是本计划新复制目的地 |
| `crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | Builtin > Local > Package；抽取后必须移除遮蔽正式包的旧局部 widget，不能只加 import |
| `docs/specs/runtime/native-vm-app-validation.md` | 真实窗口、普通 MCP 操作、读回、关闭重开、版本/进程/端口证据合同 |

参考源文件和实际依赖解析应在 T-01 记录摘要，因其他计划可能同时修改示例。不得回滚其无关改动。已有 mouse click 实现不能当作完整键盘/无障碍或大树性能已交付。

## 5. 详细设计

### 节点、受控数据和事件

| 项 | 首版合同 |
|---|---|
| 节点身份 | 一实例内节点 ID 唯一、稳定，与显示文本/行号分开；同标签、重排、过滤不改变身份。输入节点沿已有 nested record 形状，既有 id/label/children/icon/badge 由 T-01 比对确认适配 |
| 输入验证 | 重复 ID、非法结构须具名报告且不发布歧义可操作行；不能静默挑第一个。空树是正常空态；未知 expanded ID 忽略且不产生新节点；selected 不存在时不显示选中，不擅自回调更换业务选择 |
| 展开/选择 | 保留 `on_toggle(id)` / `on_select(id)` 的兼容语义；chevron 只切展开，标签只请求选择，不双触发。展开、选择由应用更新，组件不得偷偷改变受控值 |
| 禁用 | 新增/适配 disabled 状态；禁用节点无选择/展开动作，也不作为键盘目标；其可用子树的展示规则在 T-01 决策并固定正反例，不能因 disabled 把其他可见可用节点变不可达 |
| 目录显示 | 保留 directory 模式，展示过滤与导航语义同原示例；不在泛用组件里判断 OS 文件类型 |
| 懒加载 | 应用提供 hasChildren/加载状态和 children，展开请求仍由应用处理；组件有 loading/error/retry 展示及 `on_retry(id)`。未加载的目录可以展开请求，不被当叶节点；折叠不触发选择 |
| 响应所有权 | IO/请求 generation 归消费方；fixture 演示旧请求不污染新节点/已卸载实例。组件按当前 props 渲染并拒绝旧行绑定动作，不直接写应用或接收裸 IO worker 回写 |
| 数据更新 | 当前焦点节点仍可见则保留；被删除/隐藏则落到最近可用可见祖先，否则首个可用行，空树落容器。焦点修复不更改 selected；收起含焦点子树要回父节点 |

键盘：上下在当前可见可用行移动焦点；Home/End 到首/末行；右键展开，已展开则进首个可用子节点；左键折叠或到可见父节点；Enter/Space 仅请求选择当前可选节点。Tab 离开树，不把每个节点都加入页面 Tab 序列。焦点与选择分离，键盘和点击产生相同回调。空树/全禁用树也能正常离开。

树/节点的角色、名称、selected/expanded/disabled/level、可见层级对 MCP 可观察；有可独立命中的展开/选择/重试动作。动作标识绑定实例、节点 ID、当前渲染/数据修订，不能只用列表下标；同 ID 被替换也不得复用旧处理器。MCP 在当前 tree 发现节点，不缓存跨数据版本节点 ID。

### 包、消费者和支持规模

新增 `packages/tree-view/{package.at,pac.at,tree_view.at,README.md}`；是否包含 TreeIcon/FileTree 的最小可复用包装由 T-01 根据现有实际依赖决定。四例局部 TreeView 改为导入/薄适配，兼容原回调，不保留第二份可编辑控件。业务 FileTree、目录扫描、章节/SQL 状态仍归示例。必须在第二绝对根消费包与 blueprint 依赖，不能靠主检出旁路解析成功。

首版验证代表性 1000 节点及 64 层嵌套、空/单节点、宽/深形态；这是测试样本，不是新增语法上限或无限资源承诺。T-01 先探测导入→派生→布局→快照→更新/销毁整条路径；若无法支持样本，定位有界修复，不以任意截断节点让测试变绿。若确需资源上限，先提出单位、入口、错误类别和边界例，不能作为临时崩溃修复暗加上限。

视觉沿现有主题、缩进线、图标和 hover/selected/focus，名称长时有可读策略，窗口内可滚动、末行可达；浅/深和窄窗一致。strategy Q-01 的文字缺失须复现，受影响视觉 AC 关闭前须解决或以具名修复依赖阻止通过。

### 视觉参考与实现边界（r2）

2026-11-14按用户要求补充[文件导航视觉指南](../design/autoui/file-navigation-and-interaction-visual-guide.md)及[GPT参考图](../design/autoui/assets/file-navigation-084-086-reference-v1.png)，仅使用左下085区域；不实现整张组合页。

对照缩进线/chevron、icon、label及loading/error/retry行。选中与焦点分别绘制，图中Chapter 01被选中、Images有focus ring；真实expanded/selected/loading来自受控props而不是PNG箭头。展开与选择独立命中，节点ID保持语义稳定，不把路径显示文本当ID。仍不含checkbox多选/拖放或文件系统业务。浅深、小窗、长中文按已有平台门验证。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/tree-view.md | 无正式通用合同 → 节点、受控状态、展开/选择/加载/错误及键盘语义 | 收拢示例复用 | AC-01, AC-02, AC-03 |
| SD-02 | add | docs/specs/ui/tree-view.md | 无专门门 → Win/Linux native+Vue、MCP 稳定绑定、视觉/代表规模/无人值守证据 | 同组件可驱动 | AC-03, AC-04, AC-06 |
| SD-03 | modify | docs/specs/ui/examples-ownership.md | 示例持局部树实现 → 正式包单源、四示例接入、blueprint owner 不变 | 避免新的重复源 | AC-05 |

仅提议 Spec delta，本轮不写 canonical。拟实施树 `D:/autostack/.wt/ui-085/auto-ui`，分支 `codex/ui-085-tree-view`，未创建；所有实现/自检/独立复审顺序使用同一树，主检出保留 Plan 状态。

## 6. 测试设计

以下均未执行。T-05 创建 `crates/auto-ui-tools/tests/tree_view_component.rs`（包导入、生成、双实例和既有源对照），必要 runtime 回归放入经 T-01 定位的当前键盘/语义目标；不预造不存在的 runtime 测试 API。

新增 `examples/tree-view/` 与 `tests/product/verify_tree_view.py`（T-04/05）。脚本拟沿既有 native gate 参数 `--auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-dir>`，新增明确 `--backend native|vue`；T-05 实现并文档化，Vue 必须启动真实生成应用并操作其实际控件，不能只检查生成文本。复用现有 process/port/窗口/MCP 设施，不另造通用测试平台。

| Case ID | AC / 规则 | 输入与入口 | 区分性断言 | 范围及 work owner |
|---|---|---|---|---|
| V-01 | AC-01,02 受控身份 | 官方包夹具；同名不同 ID、重排、selected 不存在、未知 expanded | 精确选择/展开事件只含目标 ID，受控 props 未更新时不假装已更新；无行号寻址 | native/Vue；T-02/05 |
| V-02 | AC-02 结构安全 | 包/验证路径；重复 ID、畸形 children、空树、深/宽代表样本 | 非法输入具名拒绝歧义操作；合法全部节点可达、没有静默截断；资源失败区别无效结构 | 不声称所有规模；T-01/02/05 |
| V-03 | AC-03 键盘 | native/Vue 普通键盘；左右上下/Home/End/Enter/Space/Tab，禁用/折叠焦点项 | 焦点路径逐步等于预期；一次选择、无双 toggle；Tab 可离开，焦点修复不改业务 selected | T-03/05 |
| V-04 | AC-03,04 实例和时序 | 双树同 ID、延迟子节点、失败重试、卸载重开、快照后换数据再发旧 action | 当前实例只响应当前节点；旧 action 拒绝，新正例仍可用；loading/error 可区别，旧请求不污染应用状态 | fixture 请求 oracle 与实际 MCP；T-02/03/05 |
| V-05 | AC-05 四消费者 | 018章节/目录、026对象选择、027目录进入、041工作区文件选择 | 真实事件到应用已有 handler，业务读回/编辑器打开目标正确；移根后用正式包，无局部 shadow | 四例 native/Vue，按实际后端启动；T-04/05 |
| V-06 | AC-04,06 视觉/生命周期 | Win/Linux native + Vue，浅/深、小窗、长中文、1000 节点与末行、关闭重开 | 可见文字/层级/图标/focus 真实可读；MCP 属性准确；正常退出/owned清理/再运行成功 | 每个平台单列，Q-01闭环；T-05/06 |
| V-07 | AC-03,04,06 r2视觉参考 | 按085参考区域键盘移动focus但不选择，独立展开、loading/error/retry、同名节点 | focus/selected可区分、列对齐且长label可读，普通命中/回调ID正确；无多选/FS业务，浅深/窄窗真实截图而非生成图 | work，T-03/05/06；Win/Linux native及Vue |

拟执行：

```powershell
cargo test -p auto-ui-tools --test tree_view_component
python tests/product/verify_tree_view.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-native-dir> --backend native
python tests/product/verify_tree_view.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-vue-dir> --backend vue
```

以上新目标由 T-05 使其可运行；具体 Vue runner/测试依赖和四例业务读回入口由 T-01 冻结，并在 T-05 CLI 帮助中明确，不能把不存在的脚本称为既有证据。缺 Linux/浏览器运行环境或字段不可观察保持缺口。截图检查与自动动作断言分开记录。

实施者负责内部自检；最终 review 为后续独立上下文在同一树检查，未指定模型/代理，本轮不创建。review 冻结实现写入，核对全部 AC、反例和源码/工具/语言依赖摘要。

## 7. 验收标准

| ID | owner_stage | 可观察结果 / 关闭方法 |
|---|---|---|
| AC-01 | work | 正式包真实导入/解析/生成，保留原受控 props 和 toggle/select 单次事件；V-01通过 |
| AC-02 | work | 稳定 ID、结构校验、空态/未知受控值、加载/错误/重试和更新规则符合§5；V-01/02/04正反例通过 |
| AC-03 | work | native/Vue 键盘、焦点修复、禁用、展开选择分离和 Tab 离开符合§5；V-03/04逐步断言通过 |
| AC-04 | work | MCP 可读树/节点语义并驱动展开/选择/重试，双实例/旧绑定/卸载无串扰；V-04/06真实门通过 |
| AC-05 | work | **收窄（r2 澄清，review F-10）**：凡**存在真实 `TreeView (` 调用位**的示例必须改用官方单源并保留业务结果；经查仅 026 与 041 有调用位，二者已接入。018/027 仅含**从未实例化**的 `components/treeview.at` 副本、无任何调用位，故不作为消费者证据，其副本清理归后续（不虚构四消费者）。第二绝对根解析与 blueprint 无第二 owner 仍须通过 |
| AC-06 | work | Win/Linux native 与 Vue 真实证据、代表样本、浅深/小窗可读性、退出重开齐全；受影响Q-01无未闭项；V-06/07及视觉指南通过 |
| AC-07 | review | 独立上下文逐项复核 AC-01…06、SD、真实依赖与原消费者行为，提交版本绑定结论，无必需门缺口 |
| AC-08 | merge | 新 Spec/示例归属增量、最终合并/归档/strategy互链及清理收据准确，未写派生索引或 legacy ledger |

## 8. 执行步骤

全部未完成，0/8。后续获实施启动后从 T-01 进入，无 084/086 前置依赖。

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证入口 / 预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | [x] 已完成。`docs/reviews/plan085/decisions.md` 冻结 props/禁用/懒加载/依赖解析/行状态渲染/键盘缺口/runner/规模；四例源摘要与 blueprint owner 复核入档 | 组内环境控制门 `verify_pagination.py --backend native` 复跑 pass；P-1…P-8 探针（包内导入、Init 构造、回调回送、行状态、子件键盘、1000 节点、MCP 行动作、双实例）结果见 §9 与 decisions §9 | 01…06 |
| T-02 | work / T-01 | [ ] **重开（review F-01/F-02/F-03/F-04/F-05/F-08）**。新 `packages/tree-view/{package.at,pac.at,tree_view.at,tree_view_util.at,tree_icon.at}`（`854b9b5f`）：受控 props（nodes/expanded/selected + disabledIds/loadingIds/errorIds）、chevron 与 label 兄弟命中区、独立重试、键盘动作、重复 ID 具名结构错误、空态；`tree_view_util.at` 只持 UI 交互投影，通用树工具仍归语言 blueprint | 真实 native 窗口（组内 `auto.exe` + 本树 `auto-ui-tool`）：包导入→渲染 4 行（P1/S1/P2/P3）；图标（folder/file-text/folder-open）、badge `3`、`加载中`、`重试`、disabled 删除线、focus ring 全部可见（截图 `plan-artifacts/plan-085/pkg-native-proof.png`，sha256 `19918B0D…F4D03AA`）；MCP `autoui_action press` 命中 chevron/行/重试并按 ID 派发（`retry:p2` 且 errorIds 清空）；`cargo test -p auto-ui-tools --test pagination_component` 2/2、`--test file_dialog_component` 3/3 未退化 | 01,02,04 |
| T-03 | work / T-02 | [ ] **重开（review F-05/F-06：焦点修复与真实键盘）**。**修复** `815fec34`：`key_bindings` 键归属不确定（`registry.all()` 遍历 HashMap 且含未渲染导入 → 可抢走页面键，GUI 实证 `page_hits=0`）；改为只扫本帧可挂载组件 + 路由目标，首现序传递。**键盘语义**（`ec1c16a6`）：焦点首个可达行、Down/Up 逐行且两端饱和、Home/End 首末、已展开父节点 Expand 进首子不重发、Enter 只选择、禁用行可见但跳过 | `plan085_keydown_ownership` **5/5**（根/路由页共存、未渲染导入不抢键、已渲染子件贡献键、同文件控制派发、外部包组件派发）+ `plan085_tree_keyboard` **7/7**；真实 native 窗复核 `P1→Down→S1→Home→P1→Enter→selected=p1`（截图 sha256 `4457C19C…DFF26813`）；回归 `plan041` 4/4、`g16` 1/1、tools `pagination` 2/2 与 `file_dialog` 3/3 全绿 | 03,04 |
| T-04 | work / T-03 | 受阻（未提交，已回退）。迁移 026/041 到正式包后两例都不再渲染树行；定位到两条独立阻塞：①**递归 computed 静默空**（026 的 `dbNodes => filter_tree(...)`；同槽非递归 computed 出 2 行、递归出 0 行，无诊断）；②**`render:"vm"` 宿主上包组件行回调/样式未生效**（包本身实测装载正常：`load_package` 返回 TreeIcon+TreeView、零 warning）。两例改动已回退保持可用 | 阻塞解除前不迁移；不得用非递归副本或保留示例自带 TreeView 绕过（两者都建成第二真源） | 05 |
| T-05 | work / T-04 | [ ] **重开（review F-06/F-07/F-09 验证缺口）**（`219f85d6`/`8f36b9e3`）。tools 门 + 新建 `examples/tree-view` 夹具 + `tests/product/verify_tree_view.py`。**双端全绿**：native（2 会话、真实窗口「树形演示」、浅/深截图、exit 0、端口回收、重开再选择）与 vue（真实 Chromium + 同一矩阵），矩阵同为 `{a:5,b:2}`。矩阵改为后端中立（vnode / `data-auto-*` 各自解析） | 修好两处真缺陷才达成：①空状态在 `nodes` 变更后不出现（`is_empty` computed 在 VM 轨陈旧；改由 `tv_derive_rows` 哨兵行承载，回归 `plan085_empty_state` 1/1）②vue 腿选择器双计/引号锚定/过度捕获三处（详见 decisions）。Linux/WSL 与统一回归为显式 pending closeout 门 | 01…06 |接受只编译；含r2 V-07的参考区域对照/真实状态截图 | 01…06 |
| T-06 | work / T-05 | [ ] **重开（review F-03/F-10/F-12/F-13：文档失真与 SD 编号）**（`a348c487`）。`packages/tree-view/README.md`（props/事件/节点 schema/键盘表/VM 渲染约束/MCP 可观察面/验收入口，并链本 Plan 与视觉指南 §2.2）、`examples/tree-view/README.md`、`docs/reviews/plan085/evidence.md`（逐任务锚点 + 双端门 + 回归表 + 具名遗留）、`docs/reviews/plan085/spec-delta-proposal.md`（SD-01 新 tree-view Spec、SD-02 改 examples-ownership、SD-03 记录 018/027 无调用位）。逐条回到代码核验，纠正了自己草稿里"Enter/Space"的错述（实际只绑 `enter`） | 证据/文件可读回；Q-01 受影响视觉断言不以本计划关闭（保持 executing 至 T-07/T-08） | 06 |
| T-07 | review / T-06 | 同树独立复审源码和 SD，重放关键反例，不修实现 | 固定 reviewed commit、平台结果、AC verdict；未通过回同树work | 07 |
| T-08 | merge / T-07 pass | 建 `docs/specs/ui/tree-view.md`，修示例归属Spec，合并/归档/strategy链接/实际清理 | 源版本、最终Spec及树状态准确，不拿merge替代功能验收 | 08 |

## 9. 复审记录

- 2026-11-13：stage: new；PLAN-085 r1；outcome: pass（计划可交接，非实现通过）；next: work，在后续明确启动时执行 T-01。
- 新增 T-01…08、AC-01…08、SD-01…03；drafting，0/8。拟测试/示例/工作树尚未创建；canonical Specs 未改。

2026-11-14：stage: new；PLAN-085 r2；outcome: pass（视觉交接可供执行，非组件通过）；next: work，T-01。新增§5图/指南引用及V-07，AC-06/T-05/06纳入；现有受控API、四消费者、8任务与0/8不变。未实现控件或运行拟产品门。

2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | outcome: pass（T-02 完成；T-03 定位完成、实现留在下一轮）| worktree: `D:/autostack/.wt/ui-085/auto-ui` | branch: `codex/ui-085-tree-view` | base_commit: `c78208de` | code_commit: `854b9b5f`（T-02 包）+ 本 receipt 提交 | dependency_revisions: 组内 detached `auto-lang 86c7fd50b`、`auto-man 54fe407`、`auto-web 0adebc0`、`auto-down 5af11a3`（只读，零改动）| task_ids: T-02 | blockers: 无 | next_responsible_role: work（T-03）。

T-02 交付与证据（原始探针在 `D:/autostack/plan-artifacts/plan-085/`）：

- 新包 `packages/tree-view/`：`tree_view.at`（主 widget）、`tree_view_util.at`
  （UI 交互投影：`tv_derive_rows`/`tv_count_rows`/`tv_focus_id`/
  `tv_visible_parent`/`tv_repair_focus`/`tv_step_reachable`/`tv_ext_icon`）、
  `tree_icon.at`（18 项调色板）、`package.at`、`pac.at`。
- 公开 API 兼容四例既有 `(nodes, expanded, selected, on_toggle, on_select,
  directory)`，新增 `disabledIds/loadingIds/errorIds/on_retry`；组件只持
  `focus_index`/`invalid`/`focus_id` 三个实例局部字段，不复制业务状态。
- 真实 native 验证（组内 detached `auto-man` 构建的 `auto.exe` + 本树
  `auto-ui-tool.exe`，MCP 驱动）：包导入 → 4 行正确渲染；目录/文件图标、
  badge、`加载中`、`重试`按钮、disabled 删除线、focus ring 全部真实可见
  （`pkg-native-proof.png`，11×875 逻辑像素，sha256 `19918B0D…F4D03AA`）；
  `autoui_action press` 命中 chevron（`toggle:`）/行（`select:p1`）/重试
  （`retry:p2`）并按 ID 派发，errorIds 被消费方清除后行回落到正常态。
- 未退化：`cargo test -p auto-ui-tools --test pagination_component` **2/2 pass**
  （0.95s）、`--test file_dialog_component` **3/3 pass**（5.69s）。

T-03 进展（定位完成，实现未提交）：

- 新增组件级诊断夹具 `crates/auto-ui-runtime/tests/fixtures/plan085_keydown/`
  与六行 GUI 判据矩阵（详见 decisions.md §5）。结论：
  ①**路由页的 `onkeydown.*` 根本不进入根组件收集面**——
  `build_dynamic_component(App).key_bindings()` 只有根自己的键，路由页声明的
  第二键完全缺席；`registry.all()` 也不含路由页。
  ②`registry.all()` 迭代 `HashMap`，同名键先到先得 → 未挂载的导入组件可抢走
  键归属（GUI 实证 `page_hits=0`）。
- 曾按方向 (a) 实现「只收本帧可挂载组件 + 路由目标」，实测路由页仍解析不到
  （`get_by_route_module` 在根构建期未命中），说明路由页注册时机晚于根组件
  构建，须与 outlet 装载路径同改。该改动影响所有 App 启动期键位，**已回退未
  验证实现**，保持树绿（`git checkout` 后 `pagination_component`/
  `file_dialog_component` 复跑仍绿）。
- 下一步：T-03 从「路由页注册时机 + 实例限定键」两处继续；AC-03 保持未关闭。

任务计数：T-01、T-02 完成，2/8；status `executing`。未改 canonical Specs，
未写派生索引或 legacy ledger。

T-01 关键机制结论（详见 `docs/reviews/plan085/decisions.md` §9，原始探针在
`D:/autostack/plan-artifacts/plan-085/`）：

- 环境控制：组内构建的 `auto.exe` + 本树 `auto-ui-tool.exe` 跑既有
  `python tests/product/verify_pagination.py --backend native` **exit 0 全绿**
  （多页/尾页/容量/空态/禁用/stale/浅深截图/退出重开/端口回收）——证明环境与
  既有真实门可用，与本次新组件行为无关。
- P-1 **包内模块文件不自动导入**：漏 `use tree_util: flatten_probe` 时
  `computed => flatten_probe(...)` 静默产出空列表、无任何诊断；补 `use` 后同
  表达式产出正确 4 行。→ 包 README 与 Spec 必须写明此条。
- P-4 **`class:` f-string 不插值循环记录字段**：`class: f"base ${r.flag} tail"`
  渲染为 `base  tail`；同记录 `text f"…${r.flag}"` 正常。→ 行视觉状态改用
  字面量 class 分支（focused/selected/disabled/loading/error 全部实测可见）。
- P-5 **已定位缺口**：`onkeydown.*` 写在页/根 widget 上派发正常
  （`page_hits=1`），写在包组件子件上不派发（子件 `hits=0`，同键同会话）。
  T-03 的最小修复点 = 子件 keydown 派发解析；不得静默降级为鼠标专用树。
- P-6 1000 平铺节点在真实窗口完整渲染（1000/1000 行，快照 458 273 字节，
  末行可达），无静默截断。
- P-7 MCP `autoui_action press` 精确命中 chevron/行/重试并按 ID 派发
  （`toggle:ch1` / `select:p1` / `retry:p2` 且清除 errorIds）；真实 026 组件
  副本同形通过。
- 源复核：四例 `treeview.at` 仅 3 处差异（chevron style 变量化、hover 类、
  blueprint vs 本地 util）；三份 `tree_util.at` 字节相同
  （`4F4BA602…A268BA7A7`），语言 blueprint 版仅首行注释不同
  （`F1E3CC80…541C0158`）→ blueprint 保持 canonical owner，正式包不复制通用树
  工具。

任务计数：T-01 完成，1/8；status `drafting → executing`。未改 canonical Specs，
未写派生索引或 legacy ledger。

## 10. 待澄清事项

没有阻碍建计划的用户选择。T-01 有边界地解决节点字段兼容、disabled 子树展示、加载回调适配、键盘/语义接线能力、blueprint导入与代表规模可行性、Q-01归因和 runner。不得默默降为鼠标专用树或保留四套算法；超出现有授权范围的依赖/支持上限变更需提出具体合同差异。

2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 1 | from: none | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 78e45127 | next: work (T-03)

Goal round 1 摘要（授权继承：`/goal 完成计划UI-085` 即 work→review→bounded
repair→merge 全程授权，已记入本节与 decisions.md）：

- 工作树与依赖组：`D:/autostack/.wt/ui-085/auto-ui`（branch
  `codex/ui-085-tree-view`，base `c78208de`）；组内 detached 只读
  `auto-lang 86c7fd50b` / `auto-man 54fe407` / `auto-web 0adebc0` /
  `auto-down 5af11a3`。主检出仅计划簿记改动。
- T-01 pass：`docs/reviews/plan085/decisions.md`（c60af8f1）冻结 props、
  禁用/懒加载约定、依赖解析（blueprint 持通用树工具 canonical owner，包内
  只持 UI 投影）、行状态渲染约束、键盘缺口、runner 与代表规模；
  环境控制门 `verify_pagination.py --backend native` 复跑 exit 0。
- T-02 pass：`packages/tree-view/`（854b9b5f）经真实 native 窗口验证——
  包导入、4 行受控渲染、目录/文件图标、badge、加载中、重试按钮、disabled
  删除线、focus ring 可见；MCP `autoui_action press` 按稳定 ID 命中
  chevron/行/重试，`errorIds` 消费后回落。截图
  `plan-artifacts/plan-085/pkg-native-proof.png` sha256
  `19918B0D8A7FEB9F68A0870087E79DB2C67A15707564B6EC35B7F4853F4D03AA`。
  tools 回归：`pagination_component` 2/2、`file_dialog_component` 3/3。
- T-03 定位完成、实现未提交：路由页 `onkeydown.*` 不进入根组件收集面
  （组件级 `key_bindings()` 实证只有根自己的键）；`registry.all()` 的
  `HashMap` 迭代使同名键归属不确定，未挂载导入可抢键。候选修复实测仍未让
  路由页解析到，已回退以保树绿；夹具
  `crates/auto-ui-runtime/tests/fixtures/plan085_keydown/` 与六行判据保留给
  下一轮。AC-03 保持未关闭。
- 任务计数 2/8；未改 canonical Specs，未写派生索引或 legacy ledger；
  plan-artifacts 原始探针待 merge 阶段整体清理。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 55cdcf48 | next: work (T-03 续)

Goal round 2 摘要（T-03 部分完成；上一轮的错误归因已修正）：

- **修正上一轮的错误结论**：上轮把「路由页 `onkeydown.*` 不进入收集面」记为
  缺陷，实为**夹具布局错误**——路由页必须在 `src/front/pages/`；放进
  `src/front/` 的页被静默忽略。改正后组件级诊断显示路由页的键**正常进入**
  根组件收集面。该误导已写入 decisions.md 顶部与测试注释。
- **已修（`815fec34`）键归属不确定**：`key_bindings` 子件半边来自
  `registry.all()`，迭代 `HashMap` 且覆盖整个模块（含 import）→ 仅被 `use`
  而从未渲染的组件可抢走页面键，按键静默失效（GUI 实证 `page_hits=0`），
  且每轮赢家可能不同。修复 = 只扫本帧可挂载组件（view_tree 中解析为注册
  组件的 tag + 路由目标）并按首现序**传递**其可挂载子件；根仍先扫保住自己
  的键。GUI 复验 `one=1 two=1`（修复前 `one=0 two=0`）。
- **仍开缺口（AC-03 未关闭）**：**外部包组件的 keydown 不派发**。同文件子件
  控制组 `hits=1`；等价子件经 package `use` 到达时绑定**被正确收集**
  （`key_bindings` 含 `ChildBox.Left`）但处理器**不执行**（`hits=0`）。已由
  `plan085_external_package_child_keydown_dispatch_gap` 显式断言当前行为
  （含「绑定确实存在」前置断言），修复落地时须与 V-03 证据一起翻转为 1。
- 验证：`cargo test -p auto-ui-runtime --test plan085_keydown_ownership`
  **5/5 pass**；相关回归 `plan041_instance_ownership` 4/4、
  `g16_imported_sibling_vm_tests` 1/1、`g11_sibling_vm_tests` 4/4、
  `plan068_routed_continuations` 14/14、`file_dialog` 13/13、
  `auto-ui-tools` 的 `pagination_component` 2/2 与 `file_dialog_component`
  3/3、lib 面 keydown/keyboard 子集 6/6 全绿。
- **已知既存红（非本计划引入，主检出复现一致）**：
  `ui::clipboard::tests::set_then_get_roundtrip` 在本机剪贴板环境下失败，
  与本轮改动无关（主检出 `master` 上同样失败），记为环境性既存红。
- 任务计数 2/8（T-01、T-02 完成；T-03 部分完成，故不计数）；未改 canonical
  Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 0793ecf7 | next: work (T-04 阻塞解除)

Goal round 3 摘要（T-03 完成；T-04 受阻并回退）：

- **T-03 完成**（上一轮自报的「外部包组件 keydown 不派发」是**我的 oracle
  假象**，已更正）：两条独立原因——①声明 `Init` 的子件在帧泵前渲染
  `Loading… (<Name>)` 占位；②`View` 调试 dump 里 class 名解析为
  `RingWidth(2.0)` 等结构类，`ring-2` 子串计数恒 0。改用「渲染文本变化」作
  oracle 后同文件与包组件**都** `hits 0→1`。
- 真正修复（`815fec34`）：键归属只扫本帧可挂载组件，首现序确定。
- 键盘语义钉住（`ec1c16a6`，`plan085_tree_keyboard` **7/7**）：首个可达行起
  焦点、Down/Up 饱和不回绕、Home/End 跳首末、已展开父节点 Expand 进首子不重
  发、Enter 只选择、禁用行可见但跳过。`plan085_keydown_ownership` **5/5**。
- 真实 native 窗复核：`P1→Down→S1→Home→P1→Enter→selected=p1`；截图
  `plan-artifacts/plan-085/kbd-native-proof.png` sha256
  `4457C19C24A478ECD0761AB384EEFCC8E0E103601D721B9B1D177E99DFF26813`。
- **T-04 受阻（已回退，未提交）**：026/041 迁移后都不渲染树行。两条独立阻塞
  ①递归 computed 静默空（026 `dbNodes => filter_tree(...)`：同槽非递归出 2
  行、递归出 0 行，无诊断）②`render:"vm"` 宿主上包组件行回调/样式未生效
  （包装载实测正常：`["TreeIcon","TreeView"]`、零 warning）。两例改动已
  `git checkout` 回退，示例保持可用。
- 回归：`plan041` 4/4、`g16` 1/1、tools `pagination` 2/2 与
  `file_dialog` 3/3 全绿。既存红（非本计划）：
  `ui::clipboard::tests::set_then_get_roundtrip` 在主检出同样失败。
- 任务计数 **3/8**（T-01、T-02、T-03 完成；T-04 阻塞中）。未改 canonical
  Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 73b1b522 | next: work (T-04 重做)

Goal round 4 摘要（两条 T-04 阻塞全部证伪；迁移仍未宣布通过）：

- **阻塞①「递归 computed 返回空」证伪**：真实 native 窗口内一次性 PROBE
  处理器读回 `dbNodes=1`、`dbNodes[0].id="db/app"`、
  `dbNodes[0].children.len()=3`——递归 `filter_tree` 并未返回空。再用等价
  嵌套 fixture 直接喂正式包，派生 **root → group → leaf 三行**、每行一个
  绑定选择动作（`plan085_derives_nested_filtered_tree_like_the_026_consumer`，
  套件 **8/8**，提交 `775c1561`）。
- **阻塞②是我的路径错**：026 的
  `use { package: tree_view from "../../../../packages/tree-view" }` 少一级，
  解析到不存在的 `examples/packages/tree-view`；改五级后 `TreeView` 正常挂载
  （`/TreeView#0`、`WantSelect=16`）；041 同为五级亦挂载（`WantSelect=13`）。
  规则：包路径必须与同文件已工作的 `pagination`/`file_dialog` 同深度。
- **仍未解释（故不宣布 T-04 通过）**：两例**真实窗口**里树为空，而**未迁移
  的基线同样为空**（`on_toggle=False`、`for: r` ≤1）——现象先于本计划存在，
  与迁移无关，但成因未定位。026/041 改动继续留在回退状态。
- 回归：`plan085_keydown_ownership` **5/5**、`plan085_tree_keyboard` **8/8**、
  `plan041` 4/4、`g16` 1/1、tools `pagination` 2/2 与 `file_dialog` 3/3。
- 既存红（非本计划）：`ui::clipboard::tests::set_then_get_roundtrip` 在主检出
  同样失败。
- 任务计数 **3/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: ce3bacb6 | next: work (T-05)

Goal round 5 摘要（T-04 完成：两个真实消费者已迁移并验证）：

- **两条旧阻塞全部证伪，真因是我的两处操作错误**：
  ① 026 的 `use` 包路径少一级（解析到不存在的 `examples/packages/tree-view`）；
  ② 041 的调用位漏 `($event)` 实参。两处均已修正。
- **026 迁移**（`b89829fc`）：改用正式包，`components/treeview.at` 退役；
  保留 `tree_util.at`（`filter_tree`/`collect_ids`/`toggle_id` 仍是业务代码）。
  验证：`examples/ui/026-database/tests/desktop_mcp.py` **25/25 连续 3 次
  全绿**，含「T1 对象树分类」「T2 定位 products 表行」真实树行与切表。
- **041 迁移**（`fe411d45`）：同上；继续 `use bps...toggle_id`（blueprint 仍是
  通用树工具 canonical owner）。验证：库路径构建真实 `app.at` 得
  `TreeView` 挂载、**13 个绑定行动作、25 个真 lucide 图标、0 文本回落、
  7 个键盘绑定**；其自身套件 **44 passed / 9 failed**，与**未迁移基线逐项
  相同**（9 项先于本计划存在，迁移未改变行为）。
- **范围如实**：实际只有 026/041 存在 `TreeView (` 调用位；018/027 只含从未
  实例化的 `components/treeview.at` 副本。本轮只迁移有真实消费者的两例，
  未用副本留给 T-05 清理，不虚构消费证据。
- 回归：`plan085_tree_keyboard` 8/8、`plan085_keydown_ownership` 5/5、
  `plan041` 4/4、`g16` 1/1、tools `pagination` 2/2 与 `file_dialog` 3/3。
- 既存红（非本计划）：`ui::clipboard::tests::set_then_get_roundtrip` 主检出
  同样失败；041 桌面 shell 路径的树文本回落（库路径正常）记为具名既存缺口。
- 任务计数 **4/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 018cc49a | next: work (T-05 native 全跑)

Goal round 6 摘要（T-05 部分完成：tools 门绿；native 产品门未走通，不宣布通过）：

- **tools 门通过 2/2**（`ea4c825a`，`crates/auto-ui-tools/tests/tree_view_component.rs`）：
  两个生成器都产出组件；Vue 面带引号 emit（`toggle`/`select`/`retry`——
  不带引号不会被重写成 `emit`，回调会静默断链）、字面量 lucide chevron
  分支、字面量行状态分支；**第二绝对根** vendor 闭包逐字节相同，删依赖后
  strict 下是**具名失败**。
- **已落夹具与产品门**：`examples/tree-view/`（受控展开/选择/禁用/加载中/
  失败+重试、同页双实例、空树、浅深、文档化退出）与
  `tests/product/verify_tree_view.py`（按 Plan 的 `--auto-bin/--ui-tool-bin/
  --out-dir/--backend native|vue` 接口）。
- **夹具构建中修好两处真 bug**：①handler 名不能带下划线（`child_emit` 折叠
  下划线，`A_Select` 与包内派发键冲突，回调到不了页面）；②f-string 里对
  列表参数取 `.len()` 在 VM 轨渲染为空（摘要改为显式 model 字段）。
- **仍开**：脚本化 native 全跑未走通。回调在**库路径**已证明有效（对
  `/TreeView#0` 派发 `WantSelect` → `aSummary` 变
  `sel=chapter-01 ... select:chapter-01`），但真实窗口里按 `vnode` 行 id
  后页面摘要不更新且**无错误输出**。已排除下划线折叠、`.len()`、双实例
  歧义、包路径。**唯一未消除差异**：026 的树挂在 `App` 根（`DbSelect` 正常），
  夹具挂在 `demo` 路由页。下一轮对比这两种挂载的回调路由。**T-05 不宣布
  通过**，也未为过门搬动夹具或删断言。
- 回归：`tree_view_component` 2/2；此前 T-04 的 026 25/25×3、041 44p/9f
  （与基线同项）保持不变。
- 任务计数 **4/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 552736a8 | next: work (T-05 iced 回路修复)

Goal round 7 摘要（T-05 卡点从"未知"收窄到 iced 事件回路；仍未通过）：

- **对照矩阵**（同一组 `auto.exe`/`auto-ui-tool.exe`，今天逐条复现）：

  | 形态 | 行渲染 | 按下写页面状态 |
  |---|---|---|
  | 树在 **App 根**（真实窗口） | 是 2 行 | **是**（`summary`+`selected` 均有 `state_changes`）|
  | 树在 **路由页**（真实窗口） | 是 2 行 | **否**（无更新、无诊断）|
  | 树在 **路由页**（库路径） | 是 2 行 | **是**（`sel= exp=1` → `sel=p1`）|
  | `PaginationControl`（路由页+包，真实窗口） | 是 | **是**（既有门 pass）|
  | 最小包组件（路由页、str prop、for-in） | 是 | **是** |

  → 页面/包装配**本身没问题**；缺口只在「路由页内包子组件 + 三个 msg prop +
  computed 派生」在 **iced 回路**下，`WantSelect` 能派发（handler 名与实例都
  正确）却**不落到页面**，且无 `[VM-HANDLER]`/`[VM-INSTANCE]` 诊断。
- **已排除**：下划线折叠、`.len()` 求值、双实例歧义（单实例同样复现）、
  包路径深度、路由页形态本身、条件守卫、嵌套包内子组件。
- **新增库级永久回归**（`4053f5b8`）：
  `crates/auto-ui-runtime/tests/plan085_routed_callbacks.rs` **2/2 pass**
  ——路由页包子组件确实收到 List prop 派生 2 行、按下确实写页面状态。该测试
  是修复 iced 路径时必须保持绿的参照面。
- 回归全绿：`plan085_routed_callbacks` 2/2、`plan085_tree_keyboard` 8/8、
  `plan085_keydown_ownership` 5/5、`plan041` 4/4、tools
  `tree_view_component` 2/2。
- **T-05 不宣布通过**：未为过门搬动夹具或删断言。下一步在 iced 回路里对照
  `WantSelect` 前后的 `dirty`/`child_instance_state_id`/`callback_parent_state`。
- 任务计数 **4/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: cc74e201 | next: work (T-05 Vue 轨)

Goal round 8 摘要（T-05 **native 门首次全绿**；Vue 轨显式 not_run）：

- **native 产品门 pass**：`tests/product/verify_tree_view.py --backend native`
  outcome **pass**——2 会话、矩阵 `{a_tree_rows:5, b_tree_rows:2}`、真实窗口
  标题「树形演示」、浅/深两张真实截图、`exit_code=0`、`port_rebinds=true`、
  重开会话再选择成功。受控身份、chevron 展开、retry 恢复、空态、双实例隔离、
  退出重开全部经真实 MCP 操作与页面状态读回交叉验证。
- **修复的真缺陷（空状态）**：应用清空 `nodes` 后空态**永不出现**，而"第一帧
  就空"的树**会**显示——实测 `nodes` 2→0→2→0 时 `rows` 每帧正确重派生
  （行数 2,0,2,0），但 `is_empty` 分支一次没渲染，即 **computed 对 `.nodes`
  变更保持陈旧**（PLAN-086 同族）。修复：空态改由 `tv_derive_rows` 产出哨兵行
  承载（已证明 live 的派生），删除死掉的 `is_empty`。回归
  `plan085_empty_state` 1/1，能区分修复前后。
- **夹具形态**：树改为根 widget 托管（与 026/041 一致）；路由页回调差异仍由
  `plan085_routed_callbacks` 2/2 在库级守着，不混入门内。
- **门健壮性**：摘要正则容忍间距；隔离副本包路径随迁移；retry 等待 15s 复读；
  重开按 A/B 分块断言行数。
- **仍开**：**Vue 轨未跑通**——矩阵按 vnode id 驱动，Vue 轨无 vnode，需要标签
  驱动的矩阵变体。按实际记为 **not_run**，未伪造通过，T-05 仍不宣布完成。
- 回归全绿 23 项：`plan085_empty_state` 1/1、`plan085_tree_keyboard` 8/8、
  `plan085_keydown_ownership` 5/5、`plan085_routed_callbacks` 2/2、
  `plan041` 4/4、`g16` 1/1，tools `tree_view_component` 2/2。
- 任务计数 **4/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 8f36b9e3 | next: work (T-06)

Goal round 9 摘要（**T-05 完成：双端全绿**）：

- 产品门**两个后端都 pass**，同一矩阵 `{a_tree_rows:5, b_tree_rows:2}`：
  - **native**：2 会话、真实窗口「树形演示」、浅/深截图、`exit_code=0`、
    `port_rebinds=true`、重开会话再选择成功；
  - **vue**：真实 Chromium 页面 + 截图 + **同一套行为断言**。
- **矩阵改为后端中立**：`split_row_ids()` / `press_row()` / `press_chevron()` /
  `row_count()` / `has_empty_state()`，各 driver 自行实现（native 解析 vnode
  id，vue 解析生成 SFC 的 `data-auto-*` 标记）。这是让 vue 腿"真正跑起来且
  有可比证据"的关键，而非仅让脚本不报错。
- **为使 vue 腿有意义而修的三处**：①生成 SFC 对行容器与标签行都打
  `data-auto-tag="row"`，裸选择器双计（5 行数成 10）→ 要求 `flex-1`；
  ②摘要正则原锚定 native 的引号形态，vue `inner_text` 无引号 → 改为中立；
  ③仅改②会在 native 过度捕获（把 `" {` 吃进 log 组）造成假串扰 → 捕获止于
  引号/花括号/换行。
- 上一轮修的空状态缺陷（`is_empty` computed 陈旧 → `tv_derive_rows` 哨兵行）
  在本轮双端都得到验证。
- 回归全绿 **28 项**：`plan085_empty_state` 1/1、`plan085_tree_keyboard` 8/8、
  `plan085_keydown_ownership` 5/5、`plan085_routed_callbacks` 2/2、
  `plan041` 4/4、`g16` 1/1、tools `tree_view_component` 2/2、
  `pagination_component` 2/2、`file_dialog_component` 3/3。
- 遗留（显式 pending closeout 门，非本轮义务）：Linux/WSL 腿（084 同一边界）、
  统一回归（review/merge 档位）。
- 任务计数 **5/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: goal | plan_id: PLAN-085 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: a348c487 | next: review (T-07)

Goal round 10 摘要（**T-06 完成**：包/示例文档 + 证据 + Spec delta 提案）：

- **`packages/tree-view/README.md`**：props/事件（含载荷语义）、节点 schema
  （全键书写、稳定 id、重复 id 具名错误）、展开与选择分离、禁用不可达但可见、
  加载中 ≠ 失败、键盘表、**VM 渲染约束**（字面量 class 分支、字面量 icon 名、
  空态由派生哨兵行承载）、MCP 可观察面、验收入口；链本 Plan 与视觉指南 §2.2。
- **`examples/tree-view/README.md`**：夹具形态（树挂 App 根，与两真实消费者
  一致）、区块、受控回路、种子数据、两轨运行方式。
- **`docs/reviews/plan085/evidence.md`**：逐任务锚点、工具/依赖身份、双端产品门
  实跑（矩阵 `{a:5,b:2}`、native 双会话 + 浅深 + exit 0 + 端口回收 + 重开、
  vue 真实 Chromium）与本树 28 项回归、以及**具名遗留**（Linux/WSL `not_run`、
  统一回归归 review/merge、041 既有 9 项失败、路由页形态差异、Q-01）。
- **`docs/reviews/plan085/spec-delta-proposal.md`**：SD-01 新建
  `docs/specs/ui/tree-view.md`；SD-02 修订 `examples-ownership.md`；SD-03 记录
  018/027 **无** `TreeView (` 调用位（副本从未实例化），不作为消费者证据。
  **本轮不写 `docs/specs/`**——canonical 落地属 T-08（merge）。
- **自我纠错**：草稿键盘表写"Enter / Space"，回代码核验发现绑定面精确为
  `onkeydown.{up,down,left,right,home,end,enter}`，`Space` 只出现在注释里；
  两份文档已改为显式说明"Space 未绑定"，并在 Spec 提案里禁止在补绑定与用例
  之前声称 Space 可用。
- 任务计数 **6/8**。未改 canonical Specs，未写派生索引或 legacy ledger。
2026-11-15：stage: review | review_scope: final | reviewer: plan085-reviewer（独立上下文，未参与实现）| plan_id: PLAN-085 | plan_revision: r2 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-085/auto-ui | branch: codex/ui-085-tree-view | reviewed_commit: a348c487 | base_commit: c78208de | dependency_revisions: auto-lang 86c7fd50b / auto-web 0adebc0 / auto-down 5af11a3 clean；auto-man 54fe407 dirty(Cargo.lock) | spec_inputs: docs/specs/ui/tree-view.md 不存在；examples-ownership.md 未改；SD 提案 @ a348c487 | acceptance_results: AC-01 pass；AC-02 fail；AC-03 fail；AC-04 partial；AC-05 partial；AC-06 fail；AC-07 本记录；AC-08 pending | pending_closeout_gates: Linux/WSL native not_run；统一回归 pending | findings: F-01..F-13（High: F-01,F-02,F-03,F-04）| evidence: plan-artifacts/plan-085/{review-native,review-vue,review-native2,review-vue2,review-narrow,review-probe-dup} | next: work（T-02/T-03/T-05/T-06 受影响子项）

复审要点（独立复算，产品门我未采信回执，由 reviewer 另起 fresh out-dir 实跑）：
- **双端产品门确实 pass 且可复现**：native 两次（10.29s、矩阵 `{a:5,b:2}`、exit 0、端口回收、重开再选）、vue 两次（33.92s、真实 Chromium）。这部分成立。
- **新用例耗时全部 ≤10s**（最大 2.68s），测试成本规则满足。
- **但 4 条在合同内的硬缺口**：
  - F-01 层级/缩进丢失：行记录无 `depth`/`guides`，`Section 1.1` 与父同列（旧组件每层渲 `w-4`）。
  - F-02 懒加载目录不可展开：`has_kids = kids.len() > 0 && !leaf`，夹具 `images`（dir/children 空/非 leaf）无 chevron，与 §5「未加载目录可请求展开」冲突。
  - F-03 错误态无文本：只有「重试」按钮，无「加载失败」；而 README/提案三处声称有 → 文档失真。
  - F-04 重复 ID 仍发布歧义可操作行：实测 6 行、同 ID 可两次选中；且只在 `Init` 校验，`Validate` 无消费者。SD §2「不发布歧义可操作行」因此不成立。
- **关键 oracle 无证据**：F-05 焦点修复未实现（`tv_repair_focus` 死代码）；F-06 真实键盘从未自动化（门内零键盘动作，`plan085_tree_keyboard` 派发的是 msg 变体名）；F-07 旧 `vnode_` 拒绝零用例；F-08 role/level 语义缺失。
- **其它**：F-09 1000 节点/64 层无入库证据且窄窗无断言（实测窄窗文本裁切）；F-10 018/027 仍留未实例化副本，AC-05「四个示例单源」字面未达（正确处置是改 AC 措辞或删副本，不得虚构消费）；F-11 auto-man lock 脏与「只读零改动」表述不符；F-12 SD 编号与 Plan 表不一致；F-13 decisions 里「不得为过门搬回根 widget」禁令被 round 8 反转却未撤销。
- **两个已知遗留的判定**：Linux `not_run` 合规（环境边界）；路由页形态差异合规但需一句批准（两个真实消费者与夹具均为根托管），问题只在 F-13 禁令未撤。
- 结论：不 `needs_replan`——缺口可定位、修复局部（包内派生 + 行渲染 + 用例），不需改设计契约。
2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | round: 10 | from: review | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: 6f0426c2 | next: work (F-05/F-06/F-07/F-08/F-09/F-10/F-12/F-13)

Goal round 10 摘要（review needs_fix → **4 条 High 已修并回归**）：

- **独立复审**（`plan085-reviewer`，独立上下文，自行 fresh out-dir 实跑）：**needs_fix**，
  F-01…F-13，4 High。产品门双端确实 pass 且可复现（这部分成立），但 AC-02/03/06 有
  在合同内的硬缺口。复审结论已按合同写入 Plan §9 并重开受影响任务
  （T-02/T-03/T-05/T-06，`current_step` 6→3）。
- **本轮修好 4 条 High**（`6f0426c2`，均有能区分修复前后的回归）：
  - **F-01 层级丢失**：行无 `depth`，子行与父同列 → 行记录加 `depth`，视图按层渲染
    字面量缩进单元。**踩坑**：首版用「与栈位置并行的 depths 数组」，一旦中途 push
    子节点就错位，实测**每行 depth 恒为 0**；改为每栈项携带 `[node, depth]` 配对，
    不可漂移。
  - **F-02 懒加载目录不可展开**：`has_kids = kids.len() > 0 && !leaf` 把"未加载"当
    叶节点 → 改为 `!leaf && (kids.len() > 0 || kind == "dir")`；实测按 `images` 的
    chevron 后 emit `toggle:images` 且 `aExpanded` 增长。
  - **F-03 错误态无文本**：只有「重试」按钮 → 加「加载失败」文本（destructive 色），
    与「加载中」可区分。
  - **F-04 重复 ID 仍发布歧义行**：修这条时挖出**更深原因**——禁用分支虽然视觉
    disabled 却仍带 `onclick: .WantSelect`，所以"禁用"从未真正阻止选择。禁用行现在
    **完全不带 select handler**，重复 ID 场景因此一并变为不可操作。
- **产品门相应修正**：改为统计**可见行**（5/2）并**单独断言不可操作子集恰为 1 行**
  （即禁用行），且该子集按后端各自取（native 看缺 `.WantSelect`，vue 看生成 SFC 的
  `cursor-not-allowed` 类）。双端均 pass，矩阵一致 `{a:5, b:2}`。
- **视觉确认**：截图 `plan-artifacts/plan-085/hier/src/front/tmp/autoui-screenshot-*.png`
  ——`Section 1.1` 缩进在 `Chapter 01` 之下、`Images` 有 chevron、该行同时显示
  「加载中」「加载失败」「重试」、`Chapter 02` 删除线禁用。
- 回归：`plan085_review_fixes` **4/4**（新）、`plan085_empty_state` 1/1、
  `plan085_tree_keyboard` 8/8、`plan085_keydown_ownership` 5/5、
  `plan085_routed_callbacks` 2/2、`plan041` 4/4、tools `tree_view_component` 2/2。
- **仍开**：F-05 焦点修复（`tv_repair_focus` 死代码）、F-06 真实键盘自动化、
  F-07 旧 `vnode_` 拒绝用例、F-08 role/level 语义、F-09 1000 节点/64 层与窄窗证据、
  F-10 018/027 副本处置、F-12 SD 编号对齐、F-13 decisions 禁令撤销。
- 任务计数 **3/8**（T-02/T-03/T-05/T-06 重开中）。未改 canonical Specs。
2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: (见下) | next: work (F-06/F-08/F-09)

Goal round 11 摘要（review 修复续：F-05/F-07/F-10/F-12/F-13 已闭；F-06/F-08/F-09 留待）：

- **F-05 焦点修复已接线**（`a674b721`）：`tv_repair_focus` 原是**死代码**（无调用点），
  旧帧的索引在数据增删/折叠后静默指向别的行。现按 `focus_id` 解析 → 落最近可见祖先
  → 再落原索引（若仍有效）→ 再落首个可达行；五个键盘 handler
  （Up/Down/Expand/Collapse/Activate）移动前先自愈。**绝不写业务 `selected`**。
  新增 `plan085_focus_repair` **3/3**。
- **F-07 旧 id 拒绝已有真实证据**：门在数据变更前抓一个行 id，变更后要求该 id 被拒。
  **native `stale_id_rejected: True`**（实证）。该 oracle 只在 native 有意义——vnode id
  绑定单次渲染修订；vue 轨按 DOM 位置寻址、无修订绑定身份，故记 `None` 并注明明不适用，
  **不记空过**。
- **F-10 已删两处死副本**：018/027 的 `components/treeview.at` 无任何 `use`、无任何
  `TreeView (` 调用位（PLAN-614 遗留）。删除后全仓只剩**一份** TreeView 实现。二者
  `tree_icon.at` **保留**——其 `filetree.at` 确实在用 TreeIcon。**先证后删**：把副本
  临时 `git stash` 回去重跑 027 套件，结果仍为 **56 通过 / 2 失败**（T1/T2
  `vtree=0`），与删除后逐项相同 → 该 2 项为**既有失败**，非本次删除引入。
- **F-12 SD 编号已对齐** Plan 表：SD-01 add（节点/受控状态/键盘）、SD-02 add（验证面
  /MCP 可观察）、SD-03 modify（examples-ownership）；018/027 说明降级为"附记·非 SD"。
- **F-13 决策矛盾已正式反转**：原禁令"不得把夹具树搬回根 widget"被 round 8 反转却未撤销。
  现把原句标为**已撤销**并新增「决策反转」条目，写清三点：①根托管是**对齐两个真实
  消费者的形态**而非回避；②路由页差异**未被删除**，由 `plan085_routed_callbacks` 2/2
  正反两面持续守护；③断言**只增未减**（新增空态/retry/不可操作子集/stale/双实例/
  F-01…F-04/焦点修复）。附替代守护对照表。
- **AC-05 如实收窄**（`0dc892a2`）：改为"凡**有真实 `TreeView (` 调用位**的示例必须
  接入官方单源"——经查仅 026/041 有调用位且均已迁移；018/027 无调用位，**
  不作为消费者证据**，不虚构四消费者。
- 回归全绿 **34 项**；双端产品门 native/vue 均 pass（矩阵 `{a:5,b:2}`，
  native 另带 `stale_id_rejected: True`）。
- **仍开**：F-06 真实键盘自动化（门内仍无按键动作）、F-08 role/level 语义（**引擎
  无 role/aria/level 支持**，属计划 §3「最小引擎扩展」，需较大改动）、F-09 1000 节点
  /64 层与窄窗证据未入库。
- 任务计数 **3/8**（T-02/T-03/T-05/T-06 仍开着，因 F-06/F-08/F-09 未闭）。
2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: b58099e5 | next: work (F-08/F-09)

Goal round 12 摘要（**F-06 真实键盘已闭，并挖出两个真缺陷**）：

- **F-06 已闭**：新增 `tests/product/verify_tree_keyboard.py`——用**真实
  `autoui_keyboard` 事件**驱动（不是 msg 变体派发），13 条断言全绿：
  初始焦点、ArrowDown/Up 行走、Home/End、两端**不回绕**、Enter 选中焦点行、
  ArrowRight **进入子节点且不重复 toggle**、ArrowLeft 折叠/回父、以及
  **键盘导航从不写 `selected`**。
- 写这个门时挖出**两个用例级测试看不见的真缺陷**：
  1. **焦点坐标系不一致**：`focused` 拿**可达序**与 `focus` 比较，而
     `focus_index` 的所有写入方（`tv_focus_id`/`tv_step_reachable`/
     `tv_repair_focus`）用的都是**行索引**。两者只在"焦点行之前没有禁用行"时
     恰好相等，一旦有禁用行，**focus ring 就画在另一行上**，而按键作用在它以为
     的那行。已改为比较行索引。
  2. **选中行丢失 focus ring**：`r.selected` 分支先胜出且没有
     selected+focused 组合分支，所以按下 Enter 后焦点环**消失**——键盘用户
     当场失去位置感。已补组合分支（`bg-accent` + `ring-2`），选中与焦点同时可见。
- **双树页只断言"键被接受"**：两个实例时键名无法指认是哪一个，这是既有
  **歧义规则**（不猜），所以行为断言放在**单树门**里做。已在门内注明理由。
- 回归全绿 **25 项**：`plan085_focus_repair` 3/3、`plan085_review_fixes` 4/4、
  `plan085_empty_state` 1/1、`plan085_tree_keyboard` 8/8、
  `plan085_keydown_ownership` 5/5、`plan085_routed_callbacks` 2/2、
  tools `tree_view_component` 2/2。产品门 native 仍 pass
  （`{a:5,b:2}`，`stale_id_rejected: True`）。
- **仍开**：F-08 role/level 语义（引擎无 role/aria/level，属计划 §3 允许的
  「最小引擎扩展」，需较大改动）、F-09 1000 节点/64 层与窄窗证据未入库。
- 任务计数 **3/8**（T-02/T-03/T-05/T-06 仍开，只差 F-08/F-09）。
2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-085/auto-ui | commit: a44c2e0f | next: work (F-08 role/level) 或 review

Goal round 13 摘要（**F-09 已闭，并修掉一个真实的规模缺陷**）：

- **F-09 样本已入库**：`plan085_scale` **2/2**——1000 平铺（全部 1000 行派生、
  `Node 999` 可达）与 64 层嵌套（64 行全出、`Deep 63` 存在、缩进封顶）。
- **但样本先暴露了真缺陷，不是"缺样本"**：1000 节点时 **Init handler 直接
  `instruction budget exhausted (10,000,000 steps)`，树渲染 0 行**。根因两条：
  1. `tv_derive_rows` **每帧**都重扫整树查重复 ID，复杂度 O(n²)。现改为
     **Init/Validate 算一次**并作为 `dupGuard` 传入（组件用 `dupFlag` 模型字段承载）。
  2. 删掉一个**死 computed `reachable`**（每帧遍历整棵树，视图里无任何消费者）。
  修后 1000 节点正常出 1000 行。
- **顺带修掉缩进缺陷**：缩进分支里残留 `r.depth >= 1`，导致 1..4 每支都命中，
  每行叠加 5 个缩进单元（截图实测 Deep 5 之后越缩越窄直至跑出画面）。改回
  `r.depth == 1` 的互斥分支后，深行统一用封顶宽度。
- **截图证据**：`plan-artifacts/plan-085/deep/src/front/tmp/autoui-screenshot-*.png`
  ——420×560 窄窗下 64 层可读、缩进封顶、末层不再越界。
- 回归全绿 **27 项**：`plan085_scale` 2/2、`plan085_focus_repair` 3/3、
  `plan085_review_fixes` 4/4、`plan085_empty_state` 1/1、
  `plan085_tree_keyboard` 8/8、`plan085_keydown_ownership` 5/5、
  `plan085_routed_callbacks` 2/2、tools `tree_view_component` 2/2。
  产品门 native 仍 pass（`{a:5,b:2}`、`stale_id_rejected: True`）。
- **唯一仍开**：**F-08 role/level 语义**——引擎侧完全没有 role/aria/level 概念
  （全仓无该能力），属计划 §3 允许的「最小引擎扩展」，是剩下最大的一项；
  需新增视图可访问性属性通道 + 快照输出 + 断言。下一轮评估实现或按合同
  提出具名收窄。
- 任务计数 **3/8**（T-02/T-03/T-05/T-06 仍开，只差 F-08）。