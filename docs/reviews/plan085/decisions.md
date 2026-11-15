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
- **已定位缺口（T-01 P-5，已收敛到可复现矩阵）**：`onkeydown.*` 的派发在
  存在**其它 widget 也声明同名键**时不可靠。`key_bindings` 是**按规范化键名的
  扁平 `HashMap<String, String>`**（`dynamic.rs:568-574`：先 clone 根
  `view.key_bindings`，再用 `collect_onkeydown_bindings_with_registry`
  `or_insert` 合并 registry 全部子件——**包括本帧未实例化的包组件**）。实测
  矩阵（同一 `ArrowDown`，同一个 `auto.exe`/`auto-ui-tool.exe`）：

  | 场景 | 结果 |
  |---|---|
  | 纯页 `demo`（无包导入），页声明 `down`+`up` | **pass**（`one=1 two=1`，两键独立） |
  | `App` 根声明 `down` | **pass**（`root_hits=1`） |
  | 页声明 `down`，同树渲染同包组件（该组件也声明 `down`） | **pass**（`page_hits=1`） |
  | 页声明 `down`，仅 `use` 导入包组件但**不实例化** | **fail**（`page_hits=0`，无错误） |
  | 页声明 `space`+`tab`，同树渲染同包组件（该组件声明 `down/up/…`） | **pass**（`one=1`，键名不冲突） |
  | 同包组件内声明 7 个键，页不声明任何键 | **partial**：单实例 `hits=1`；**两实例 `hits=0`** |

  可复现规则：**同名键在页与包组件之间、或包组件多实例之间冲突时，派发丢失且
  无诊断**；键名不冲突时正常。这解释了四示例「只有鼠标」的历史状态——树组件的
  `down/up/enter/left/right` 一旦进入包注册表就会与其它组件同名键互撞。
- **T-03 结论（2026-11-15，两次自我更正后定案）**：
  - **更正一（夹具）**：曾把「路由页 `onkeydown.*` 缺失」记为缺陷，实为
    **夹具布局错误**——路由页必须在 `src/front/pages/`；放进 `src/front/`
    的页被静默忽略。改正后路由页的键**正常进入**根组件收集面。
  - **更正二（oracle）**：曾把「外部包组件 keydown 不派发」记为仍开缺口，
    实为**探针 oracle 假象**。两条独立原因：
    ① 声明 `Init` 的子件在帧泵跑完 Init demand 之前渲染
    `Loading… (<Name>)` 占位，视图 dump 里既无行也无文本，看起来与派发失败
    一模一样；
    ② `View` 调试 dump 里 class 名不再以纯文本出现（解析为 `RingWidth(2.0)`
    等结构化类），拿 `ring-2` 子串去数焦点行恒得 0。
    用「渲染文本变化」作 oracle 后，同文件子件与外部包子件**都** `hits 0→1`
    ——包组件派发本来就是好的。
- **真正修复的缺陷（已提交 `815fec34`）**：`key_bindings` 的**键归属不确定**。
  子件半边来自 `registry.all()`，而它迭代 `HashMap` 且覆盖**整个模块**
  （含 import）。后果：未被渲染、仅 `use` 进来的组件可以抢走页面的键——
  此时按键什么都不发生，**无错误、无诊断**；且每轮赢家可能不同。GUI 实证
  `page_hits=0`。修复 = 只扫**本帧可挂载**的组件（`view_tree` 中解析为注册
  组件的 tag + 路由目标），并按首现序**传递**其可挂载子件；根仍先扫保住
  自己的键。三条永久反例（`plan085_keydown_ownership.rs`）。
- **T-05 卡点已收窄（2026-11-15，提交 `4053f5b8`）**：脚本化 native 全跑仍
  未走通，但缺口已定位到 **iced 事件回路**，不再是"不知道哪里不对"。
  今天逐条复现的对照矩阵（同一 `auto.exe` / `auto-ui-tool.exe`）：

  | 形态 | 行渲染 | 按下后写页面状态 |
  |---|---|---|
  | 树挂在 **App 根**（真实窗口） | 是（2 行） | **是**（`summary` 与 `selected` 都出现 `state_changes`） |
  | 树挂在 **路由页**（真实窗口） | 是（2 行） | **否**（无更新、**无任何错误输出**） |
  | 树挂在 **路由页**（库路径 `build_dynamic_component`） | 是（2 行） | **是**（`sel= exp=1` → `sel=p1`） |
  | `PaginationControl`（路由页 + 包，真实窗口） | 是 | **是**（既有产品门 pass） |
  | 最小包组件（路由页、str prop、list for-in） | 是 | **是** |

  → 结论：**页面/包装配本身没问题**；差异只在「路由页内的包子组件 +
  `on_toggle`/`on_select`/`on_retry` 三 msg prop + computed 派生」这一组合在
  iced 回路下，行的 `WantSelect` 能派发（handler 名与实例都正确）却**不落到
  页面**，且无 `[VM-HANDLER]`/`[VM-INSTANCE]` 诊断。
  - 已排除：下划线折叠（改 `ASelect` 无变化）、`.len()` 求值、双实例歧义
    （单实例同样复现）、包路径深度、路由页形态本身、条件守卫
    （`if !tv_member(...)` 去掉无变化）、嵌套包内子组件（`TreeIcon` 形态在
    Mini 里正常）。
  - **新增库级永久回归**：`plan085_routed_callbacks` **2/2 pass**——路由页包子
    组件确实收到 List prop 并派生 2 行、按下确实写页面状态。该测试是修复
    iced 路径时必须保持绿的参照面。
  - **下一步**：在 iced 回路里对照 `WantSelect` 派发前后的
    `dirty`/`child_instance_state_id` 解析，重点看路由页实例的
    `callback_parent_state` 是否在 iced 帧里拿到正确父态。
- 不得为过门而把夹具树搬回根 widget 掩盖缺陷，也不得删断言：回调必须
  在夹具既有形态下真实可用。

- **T-05 进行中（2026-11-15，提交 `ea4c825a`）**：
  - **tools 门已落并通过 2/2**：`crates/auto-ui-tools/tests/tree_view_component.rs`
    ——两个生成器都产出组件；Vue 面带引号 emit（`toggle`/`select`/`retry`，
    不带引号的变体不会被重写成 `emit`，回调会静默断链）、字面量 lucide
    chevron 分支、字面量行状态分支；第二绝对根 vendor 闭包**逐字节相同**，
    删依赖后在 strict 下是**具名失败**。
  - **示例与产品门已落**：`examples/tree-view/`（受控夹具：展开/选择/禁用/
    加载中/失败+重试、同页双实例、空树、浅深切换、文档化退出）与
    `tests/product/verify_tree_view.py`（`--auto-bin/--ui-tool-bin/--out-dir/
    --backend native|vue`，覆盖受控身份、重试、实例隔离、空态、截图、
    退出重开）。
  - **夹具构建中踩到并修好的两个坑**（记录避免重犯）：
    ① **handler 名不能带下划线**——`child_emit` 会折叠下划线，
    `A_Select` 与包内派发键冲突，回调到不了页面；改成 `ASelect` 等。
    ② **f-string 里对列表参数取 `.len()` 在 VM 轨渲染为空**——摘要行改为
    显式 model 字段维护。
  - **仍开的一条**：**脚本化 native 全跑尚未走通**。受控回调在**库路径**已被
    证明有效（对 `/TreeView#0` 派发 `WantSelect` 后 `aSummary` 变为
    `sel=chapter-01 ... select:chapter-01`），但真实窗口里用 `vnode` 行 id
    按下后页面摘要不更新，且**无任何错误输出**。已排除：下划线折叠、
    `.len()` 求值、双实例歧义（GUI 走 vnode 不受影响）、包路径。
    **下一步**：对比「根 widget 内树」（026，`DbSelect` 正常）与「路由页内树」
    （夹具）两种挂载的回调路由差异——这是目前唯一未消除的形态差异。
    T-05 **不宣布通过**。
- 不得为过门而把夹具树搬回根 widget 掩盖缺陷，也不得删断言：回调必须
  在夹具既有形态下真实可用。

- **T-04 迁移完成（2026-11-15，提交 `b89829fc` 026 / `fe411d45` 041）**：
  两例改用正式包，`components/treeview.at` 退役。**两条旧阻塞全部证伪**：
  - 「递归 computed 返回空」不成立：真实窗口 PROBE 读回 `dbNodes=1`、
    `dbNodes[0].id="db/app"`、`children.len()=3`；等价嵌套 fixture 喂正式包
    派生 root → group → leaf 三行（`plan085_derives_nested_filtered_tree_
    like_the_026_consumer`）。
  - 「包组件在 vm 宿主不生效」不成立：真实 `app.at` 走库路径构建时 041
    `TreeView` 挂载 `/TreeView#0`、13 个绑定行动作、25 个真 lucide 图标、
    0 个文本回落、7 个键盘绑定齐全。
  - **真正原因是我的两处操作错误**：026 的 `use` 路径少一级（解析到不存在的
    `examples/packages/tree-view`）；041 的调用位漏了 `($event)` 实参。
    两条都已修正。规则：包路径深度必须与同文件已工作的 `pagination`/
    `file_dialog` 一致。
- **消费者验证证据**：
  - 026 `tests/desktop_mcp.py`：**25/25 连续 3 次全绿**，其中
    「T1 对象树分类」「T2 定位 products 表行」直接驱动真实树行与切表。
  - 041 `tests/desktop_mcp.py`：**44 passed / 9 failed**，与**未迁移基线
    逐项相同**（menubar onclick、粘贴、关标签/空态、AUTO_OPEN_PATH、T10×3）
    ——这 9 项先于本计划存在，迁移未改变其行为。
  - 041 的**桌面 shell 路径**在这台机器上仍渲染树的文本回落（`[Image]`、
    无 `class`/`onclick`），而库路径与 026 均正常；该差异在基线同样存在，
    不是迁移引入，记为具名既存缺口，不阻塞本计划的功能验收。
- **T-04 关于「四消费者」的落地范围**：实际 grep 只有 026 与 041 存在
  `TreeView (` 调用位；018/027 只含从未被实例化的 `components/treeview.at`
  副本（`FileTree` 形态在 018/026 存在但也不在调用位）。因此本轮按**真实
  消费者**迁移两例；018/027 的未用副本留给 T-05 统一清理时一并删除，
  不在本轮虚构消费证据。
- 包仍持 UI 交互投影（`tree_view_util.at`），语言 blueprint 仍是通用树工具
  canonical owner；041 继续 `use bps...toggle_id`，未建第二真源。
- **MCP 通道的三个使用陷阱（写进测试模块头，避免后来者重犯）**：
  ① `on_with_input_for(widget, event)` 收的是 **msg 变体名**（`Activate` /
  `GoHome` / `Expand`），不是键名；派发 `"Enter"` 是静默 no-op。
  ② `autoui_state` 报的是**页面**字段；页面不镜像树的焦点时恒显示
  `focus_index: 0`，据此判断「没动」是错的。
  ③ 真实 GUI 里焦点行用 `ring-2 ring-primary`、选中行用
  `bg-accent text-accent-foreground`；两者同时成立时选中分支优先（设计如此）。
- 真实 native 窗口复核（最终二进制）：`focus P1 → Down → S1 → Home → P1 →
  Enter → selected=P1`；截图 `plan-artifacts/plan-085/kbd-native-proof.png`
  sha256 `4457C19C24A478ECD0761AB384EEFCC8E0E103601D721B9B1D177E99DFF26813`。
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
| P-5 子件键盘 | 包组件内 `onkeydown.*` 是否派发 | **两处更正后定案**：①**键归属不确定**（`registry.all()` 遍历 HashMap 且含未渲染导入，可抢走页面键，GUI 实证 `page_hits=0`）——已修复 `815fec34` + 三条永久反例；②曾记的「外部包组件不派发」是 **oracle 假象**（Init 占位 + class 名不进 dump）；用渲染文本作 oracle 后同文件与包组件均 `hits 0→1`。键盘语义由 `plan085_tree_keyboard.rs` 7/7 钉住 |
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

- **键盘语义已用组件自身回调钉住（`ec1c16a6`，7/7 pass）**：焦点从首个可达
  行开始；Down 逐行前进且两端饱和（不回绕）；Home/End 跳首末；已展开父节点上
  Expand 进入首个子节点且**不重发展开**；Enter 只请求选择、从不展开；禁用行
  可见但被导航跳过。