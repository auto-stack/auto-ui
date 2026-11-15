---
plan_id: PLAN-085
status: executing
feature_name: TreeView 正式组件与示例复用
author: [Codex]
created_at: 2026-11-13T00:00:00Z
updated_at: 2026-11-15
plan_revision: 2
current_step: 1
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
| AC-05 | work | 四个示例使用官方单源且保留业务结果，第二绝对根解析，blueprint无第二 owner；V-05及源清单对照通过 |
| AC-06 | work | Win/Linux native 与 Vue 真实证据、代表样本、浅深/小窗可读性、退出重开齐全；受影响Q-01无未闭项；V-06/07及视觉指南通过 |
| AC-07 | review | 独立上下文逐项复核 AC-01…06、SD、真实依赖与原消费者行为，提交版本绑定结论，无必需门缺口 |
| AC-08 | merge | 新 Spec/示例归属增量、最终合并/归档/strategy互链及清理收据准确，未写派生索引或 legacy ledger |

## 8. 执行步骤

全部未完成，0/8。后续获实施启动后从 T-01 进入，无 084/086 前置依赖。

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证入口 / 预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | [x] 已完成。`docs/reviews/plan085/decisions.md` 冻结 props/禁用/懒加载/依赖解析/行状态渲染/键盘缺口/runner/规模；四例源摘要与 blueprint owner 复核入档 | 组内环境控制门 `verify_pagination.py --backend native` 复跑 pass；P-1…P-8 探针（包内导入、Init 构造、回调回送、行状态、子件键盘、1000 节点、MCP 行动作、双实例）结果见 §9 与 decisions §9 | 01…06 |
| T-02 | work / T-01 | 新 `packages/tree-view/`，复用既有派生工具、TreeIcon，落实结构/加载/事件/身份合同；不写语言仓 | 官方包生成/模型正反例；V-01/02/04结果准确，无复制工具owner | 01,02,04 |
| T-03 | work / T-02 | 实例局部焦点、键盘和 MCP 行绑定/语义；仅在 T-01 已定位的 compiler/runtime 通用接线点作最小扩展 | V-03/04 的两端逐键/旧ID/双实例反例通过，当前新action正例可用 | 03,04 |
| T-04 | work / T-03 | 新 `examples/tree-view/`，四例组件导入/实际 pac 依赖迁移，移除旧 TreeView shadow，保留业务包装和结果 | 四例 `auto run -r vm` 与实际 Vue运行；V-05精确消费结果，移根解析通过 | 05 |
| T-05 | work / T-04 | 新 tools测试和上述 product gate；落地 V-01…07，复用进程端口设施，运行 Win/Linux+Vue | §6命令及四消费者入口；每轨有真实窗口/事件/读回/关闭重开，不接受只编译；含r2 V-07的参考区域对照/真实状态截图 | 01…06 |
| T-06 | work / T-05 | 正式包 README、示例说明、`docs/reviews/plan085/`证据，提出最终 Spec delta，strategy 实施摘要；包README链接本Plan视觉指南和对应区域 | 逐项证据/文件摘要可读回，Q-01受影响项关闭，否则保持 executing | 06 |
| T-07 | review / T-06 | 同树独立复审源码和 SD，重放关键反例，不修实现 | 固定 reviewed commit、平台结果、AC verdict；未通过回同树work | 07 |
| T-08 | merge / T-07 pass | 建 `docs/specs/ui/tree-view.md`，修示例归属Spec，合并/归档/strategy链接/实际清理 | 源版本、最终Spec及树状态准确，不拿merge替代功能验收 | 08 |

## 9. 复审记录

- 2026-11-13：stage: new；PLAN-085 r1；outcome: pass（计划可交接，非实现通过）；next: work，在后续明确启动时执行 T-01。
- 新增 T-01…08、AC-01…08、SD-01…03；drafting，0/8。拟测试/示例/工作树尚未创建；canonical Specs 未改。

2026-11-14：stage: new；PLAN-085 r2；outcome: pass（视觉交接可供执行，非组件通过）；next: work，T-01。新增§5图/指南引用及V-07，AC-06/T-05/06纳入；现有受控API、四消费者、8任务与0/8不变。未实现控件或运行拟产品门。

2026-11-15：stage: work | plan_id: PLAN-085 | plan_revision: r2 | outcome: pass（T-01 完成，机制合同已冻结；非组件交付）| worktree: `D:/autostack/.wt/ui-085/auto-ui` | branch: `codex/ui-085-tree-view` | base_commit: `c78208de` | code_commit: 本 receipt 提交（`docs/reviews/plan085/decisions.md` 新增）| dependency_revisions: 组内 detached `auto-lang 86c7fd50b`、`auto-man 54fe407`、`auto-web 0adebc0`、`auto-down 5af11a3`（只读，零改动）| task_ids: T-01 | blockers: 无 | next_responsible_role: work（T-02）。

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
