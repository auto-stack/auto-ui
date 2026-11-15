# PLAN-085 决策记录（T-01）

2026-11-15，执行树 `D:/autostack/.wt/ui-085/auto-ui`，分支
`codex/ui-085-tree-view`，基线 `c78208de`（master）。本文冻结实施语义与
T-01 机制结论；所有决策保持 Plan §5/§7 不变。探针均在本仓外 scratch
`D:/autostack/plan-artifacts/plan-085/` 执行，命令/结果摘录于 §9。

## 1. 来源复核（§4 逐项，实际 HEAD）

| 来源 | 当前事实（本树复核） |
|---|---|
| `examples/ui/018-book-reader/src/front/components/treeview.at` | SHA256 `D1571EF0…B2FBD0047`；`use tree_util: flatten_tree`，`style tree_chevron` 字面量 |
| `examples/ui/026-database/src/front/components/treeview.at` | SHA256 `31BB0990…22934335`（Plan §4 记录值复现一致）；chevron style 内联字面量 |
| `examples/ui/027-file-manager/src/front/components/treeview.at` | SHA256 `83A275DF…ADDE5C49F`；chevron 命中区多 `rounded hover:bg-accent/60` |
| `examples/ui/041-auto-edit/src/front/components/treeview.at` | SHA256 `F2E35890…AD6DC6C5`；`use bps.navigation.filetree.tree_util: flatten_tree`（四例中唯一直用语言 blueprint） |
| 四例 tree_util.at | 字节相同，SHA256 `4F4BA602…A268BA7A7`（216 行，含 `flatten_tree`/`filter_tree`/`toggle_id`/`collect_ids`/`has_id`/`ext_icon`） |
| 语言 blueprint `tree_util.at` | `D:/autostack/auto-lang/packages/blueprints/navigation/filetree/tree_util.at` SHA256 `F1E3CC80…541C0158`；与本仓副本仅首行注释不同（`PLAN-070 收编`），函数体逐行相同 |
| 语言 blueprint `tree_icon.at` | SHA256 `B0ED22D7…9275D8A`；与本仓 026 副本仅首行注释不同 |
| `tree_icon.at` 三份差异 | 018/027 相同（`5C364541…444E258`，style 字面量）；026 用 `stylekit.styles: icon_base`（`28F42744…53FEFE391`），调色板 14 项（多 `table`/`zap`） |
| `crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | Builtin > Local > Package 优先级不变；抽取后须删除四例局部 widget 才能不遮蔽正式包 |
| `docs/specs/runtime/native-vm-app-validation.md` | 真实窗口/MCP 事件/读回/负控/退出重开/证据记录合同（NVM-01…06）在位 |
| `docs/specs/ui/examples-ownership.md` | 示例单源在本仓 `examples/ui/`；bps 为语言仓只读资产，L1 零副本 |
| `tests/product/verify_pagination.py` | 可复用的进程归属/窗口/MCP/退出重开/截图设施；`--backend native|vue` 双轨已在位 |
| `docs/specs/ui/tree-view.md` | **不存在**（SD-01/02 目标正确，T-08 建立） |

四例 `TreeView` props 与 view 结构一致：`(nodes, expanded, selected, on_toggle,
on_select, directory)`；`computed rows => flatten_tree(...)`；chevron 与 label
为兄弟命中区（避免 vue 冒泡双触发）；`directory` 只影响图标派生，不含文件系统
业务。差异仅 3 处（chevron style 变量化、hover 类、blueprint vs 本地 util）。

## 2. 包与依赖解析（冻结）

- 新包目录 `packages/tree-view/`：`package.at`、`pac.at`、`tree_view.at`（主
  widget）、`tree_icon.at`（`TreeIcon` 调色板，14 项取 026 超集 + 三个正式包
  新增名）、`tree_util.at`（**UI 适配投影**，见 §4）、`README.md`。
- **包内模块文件不会被自动导入**（T-01 实证，见 §9 P-1）：每个 widget 文件
  必须显式 `use tree_util: <fn…>`；漏写时 `computed => fn(...)` 静默产出空
  列表、无诊断。此条写入包 README 与 Spec。
- `pac.at` 声明 `render: "vue"`（与 file-dialog/pagination 同形）。消费者用
  `use { package: tree_view from "<解析后包目录>" }`；示例另在 pac 增
  `dep tree_view { path: "../../../packages/tree-view" }` 供 vue 轨 Phase-1c
  扫描与 native 轨 VM sweep。
- 语言 blueprint 仍是 `flatten_tree` 通用工具的 canonical owner，本包**不复制**
  一份通用树工具作为第二真源；`packages/tree-view/tree_util.at` 只承载 UI 交互
  投影（禁用可达、focus ordinal、行状态旗标），通用形状/遍历语义由 blueprint
  提供。041 保留其 `bps` 依赖用于 `toggle_id`，其 `TreeView` 改用正式包。

## 3. 公开接口（冻结，兼容原四例）

```
widget TreeView (
    nodes: List = [],            // 嵌套 record: {id, label, children, kind, icon, is_leaf, badge}
    expanded: List = [],         // 受控展开 ID 列表
    selected: str = "",          // 受控选中 ID
    disabledIds: List = [],      // 新增：禁用节点 ID
    loadingIds: List = [],       // 新增：子节点加载中
    errorIds: List = [],         // 新增：加载失败
    on_toggle: msg,              // 兼容：仅请求展开/折叠
    on_select: msg,              // 兼容：仅请求选择
    on_retry: msg,               // 新增：请求重试加载（载荷为节点 ID）
    directory: bool = false      // 兼容：目录显示模式（仅图标派生意图）
)
```

组件 `msg` 变体：`"toggle"(str)`、`"select"(str)`、`"retry"(str)`（引号形式
必需——vue 轨 emit 重写只识别引号变体，PLAN-086 同款结论），加上键盘动作
`Up/Down/GoHome/GoEnd/Expand/Collapse/Activate`。回调一律走
`."<emit>"(id)` 体内调用，由 `child_emit` 路由到父 `on_<name>:` prop；
`on_toggle`/`onToggle`/`ontoggle` 三形经大小写+下划线折叠归一（PLAN-614 R1
机制，T-01 复跑通过）。

组件只持 `focus_index int` 与 `invalid str` 两个实例局部字段；不复制
`selected`/`expanded`。受控 props 每帧重播种，焦点不写业务选择。

## 4. 派生与行状态（冻结，含 T-01 实测约束）

- `tree_util.at` 提供：`derive_rows(nodes, expanded, selected, disabled,
  loading, error, focus)` → 行记录
  `{id, label, has_kids, open, disabled, selected, focused, loading, error}`；
  `count_rows(nodes, expanded, disabled)` → 键盘可达行数；
  `member(list, id)` / `duplicate_id(nodes)` / `focus_id(rows, index)`。
- **`class:` 的 f-string 不插值循环记录字段**（T-01 实证 P-4：`class: f"base
  ${r.flag} tail"` 渲染为 `base  tail`；同一记录的 `text f"…${r.flag}"` 正常）。
  因此行视觉状态一律用**字面量 class 分支**（未选中未聚焦 / focused ring /
  selected / selected+disabled / disabled），不做 f-string 拼接。`style:`
  的 f-string 另有既有 Tailwind 失效问题（原组件注释已记录），同样回避。
- 焦点序 = 可见且可用的行序（`count_rows` / `focused` 旗标同源），禁用行
  仍显示但 `focused=false`、不占键盘序号。焦点修复（数据更新后）由消费方
  或组件内 `focus_index` clamp 完成，绝不改 `selected`。

## 5. 键盘（冻结接法 + 已定位缺口）

- 上/下在可见可用行移动；Home/End 首末；右展开；左折叠；Enter 请求选择。
  焦点与选择分离；一次按键最多一个回调。
- **已定位缺口（T-01 P-5）**：`onkeydown.*` 写在**页/根 widget** 上可派发
  （实测 `page_hits=1`）；写在**子 widget（包组件）** 上不派发（同一
  `ArrowDown`，子件 `hits=0`，无错误）。`collect_onkeydown_bindings_with_registry`
  已遍历 registry 子件并按 `or_insert` 合并，疑似值被写成
  `"<Widget>.<Handler>"` 但派发面在子件实例上无法解析（`instance=None` 且
  无存活实例解析路径）。T-03 的最小修复点 = **子件 keydown 派发解析**，不新增
  测试专用入口、不给每个示例各留一份键盘算法；若修复不可行则按 Plan §5 具名
  记录未闭项，不静默降级为鼠标专用树。
- Tab 离开树：树容器不加入逐节点 Tab 序列（组件不产生 focusable 输入），
  沿用既有 `__focus_next_input` 焦点环；空树/全禁用树同样可离开。

## 6. MCP 可观察面（T-01 实测）

- 树容器携带 `on_toggle`/`on_select`/`on_retry` 与 `onkeydown.*` 属性；行
  label 为 `text` 节点，展开/选择/重试为独立可命中的 `container`/`row`/
  `button` 节点，`autoui_action press` 以 `vnode_<id>` 精确派发
  （实测 `handler: .Probe3demo@//Probe3#0.WantSelectsp1`，载荷 ID 正确）。
- 行状态在 `class`→快照 `style` 属性可见（`ring-2 ring-primary` /
  `bg-accent text-accent-foreground` / `opacity-50`），因此「焦点 vs 选中」
  「禁用」「loading/error」对 MCP 可断。节点 ID 绑定当前渲染修订；数据变化后
  旧 `vnode_` 必须被拒绝（T-05 反例）。
- 禁止跨数据版本缓存节点 ID；同 ID 被替换不得复用旧处理器（`vnode_` 每帧
  由视图路径派生）。

## 7. 首版支持规模（T-01 实测）

- 1000 平铺节点：真实窗口完整渲染 1000 行，快照 458 KB，无静默截断
  （P-6）。64 层嵌套由同一迭代栈派生（无递归），随 T-05 样本复验。
- 该数字是测试样本，不是语法上限或性能承诺；不做任意截断、不暗加上限。

## 8. 四消费者迁移（冻结计划）

| 示例 | 迁移内容 | 保留 |
|---|---|---|
| 018 book-reader | 删 `components/treeview.at` + `tree_util.at`？**保留** `tree_util`（`filter_tree`/`collect_ids` 仍被章节侧栏使用），仅删 `treeview.at` 与 `tree_icon.at`，改导入正式包 | 章节数据/选中读回 |
| 026 database | 删 `treeview.at`，`tree_icon.at` 改由包提供；保留本地 `tree_util` 供 SQL/对象状态使用 | 对象选择→表数据读回 |
| 027 file-manager | 删除未使用的 `treeview.at`/`tree_util.at`/`tree_icon.at`（该例当前无 TreeView 实例，仅 `FileIcon`/`fs_util` 在用） | 目录进入/列表 |
| 041 auto-edit | 删 `treeview.at`，改导入正式包；保留 `bps` 依赖（`editor_store` 用 `toggle_id`） | 工作区文件选择→编辑器打开 |

迁移前逐个复核实际调用点（T-04 落实），不得因删除未用文件破坏 `package.at`
的 official 包声明。

## 9. 探针清单（scratch `D:/autostack/plan-artifacts/plan-085/`）

| 探针 | 问题 | 结果 |
|---|---|---|
| P-0 环境控制 | 组内 `auto-man` detached + 本树 `auto-ui` 能否跑通既有真实门 | **pass**：`verify_pagination.py --backend native` 复跑绿（历史页/尾页/容量/空态/禁用/stale/浅深截图/退出重开/端口回收），证明环境可用、非新组件行为 |
| P-1 包内模块导入 | 包目录里 `<name>_util.at` 是否自动可用 | **fail→机制定案**：不自动；漏 `use tree_util: …` 时 `computed => fn(...)` 静默空列表、无诊断。补 `use` 后同表达式产出 4 行正确结果 |
| P-2 Init 内构造节点树 | 受控 `nodes` 由 `Init` 里 `List.push(record)` 构造是否成立 | **pass**：`{id,label,children,is_leaf,badge}` 记录可构造、可嵌套、`len()`/索引/`for` 迭代均正常；`children` 内联字面量与先建子列表两形均可 |
| P-3 回调回送 | 包组件 `on_toggle: msg` 是否可回送父 handler | **pass**：`handler: .TreeViewdemo@//TreeView#0.Togglesch1`，父状态 `expanded` 更新、行视图重派生 |
| P-4 行状态渲染 | `class: f"…${r.state}"` 能否按行变化 | **fail→定案**：循环记录字段在 `class:` f-string 中渲染为空（`base  tail`），同记录 `text f"…"` 正常。改用字面量分支后 focused/selected/disabled/loading/error 全部正确可见 |
| P-5 子件键盘 | 包组件内 `onkeydown.down` 是否派发 | **gap**：子件 `hits=0`；同键写页根 `page_hits=1`。T-03 最小修复点已定位（子件 keydown 派发解析），不新增测试入口 |
| P-6 代表规模 | 1000 节点是否完整渲染 | **pass**：1000/1000 行，快照 458 273 字节，末行 `Node 1000` 在树内 |
| P-7 MCP 行动作 | `autoui_action press` 是否命中行/chevron/重试并带正确 ID | **pass**：chevron→`toggle:ch1`、行→`select:p1`、重试→`retry:p2`（且 `errorIds` 清除）；真实 026 组件同形复跑通过 |
| P-8 双实例 | 同 ID 两实例是否互不串扰 | **pass（可选面）**：两实例各自持 `focus_index`，父状态单源更新后两实例同步投影；不存在共享实例局部态 |

## 10. Runner 与平台门

- Windows native：本机；组内 `auto-man` detached 构建 `auto.exe`（rela
  解析到本树 auto-ui 构成"补丁版"），`auto-ui-tool` 在本树构建。
- Vue 轨：`auto run` + vite + Playwright（复用 `verify_pagination.py` 的
  `Browser` 形态），真实浏览器操作生成应用控件。
- Linux：WSL Ubuntu-22.04 在位；与 PLAN-084 同一边界（wgpu 表面在 WSLg/
  Xvfb 下 `Invalid surface`）——不可用则记 `not_run` 并保持 pending closeout
  门，不记 pass。
- 统一回归：work 冻结后由 final review/merge 执行。

## 11. Spec 增量提案（T-08 落 canonical）

SD-01/SD-02 → 新 `docs/specs/ui/tree-view.md`：节点与受控状态、展开/选择/
加载/错误/重试、禁用与键盘语义、MCP 可观察面与稳定寻址、包内模块显式导入、
行状态以字面量分支渲染（f-string 循环字段限制）、代表规模与双端证据。
SD-03 → `docs/specs/ui/examples-ownership.md`：四示例改由正式包单源，
blueprint owner 不变，示例保留业务状态与读回。
