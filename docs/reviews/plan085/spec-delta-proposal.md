# PLAN-085 Spec 增量提案（SD-01/02/03）

T-06 提出、T-08（merge）合入 canonical。**本轮不写 `docs/specs/`**——按四技能
合同，canonical Spec 由 merge 阶段落地。

---

## SD-01 → add `docs/specs/ui/tree-view.md`（节点/受控状态/展开选择/加载错误/键盘语义；覆盖 AC-01, AC-02, AC-03）

> 分类：feature ｜ 状态：active ｜ 来源计划：`docs/plans/085-official-tree-view-component.md`

### 1. 单源与包形态

- TreeView 单源为 `packages/tree-view/`（`TreeView` + `TreeIcon` +
  UI 投影模块 `tree_view_util.at`）。
- **包内模块文件不会被自动导入**：使用方必须显式
  `use tree_view_util: …` / `use tree_icon: TreeIcon`。漏写时
  `computed => fn(...)` 静默产出空列表且无诊断（实测）。
- 通用树工具（`flatten_tree`/`filter_tree`/`toggle_id`/`collect_ids`）的
  canonical owner 仍是 `bps.navigation.filetree.tree_util`；本包只持 UI 交互
  投影，不复制通用树算法。

### 2. 节点 schema 与身份

```
{ id: str, label: str, children: List, kind: str, icon: str, is_leaf: bool, badge: str }
```

- 字段**全键书写**：VM 的 Obj 字面量为形状锁定类型，缺键字段访问是硬错。
- `id` 在一实例内**唯一且稳定**，与显示文本、行序分开；同标签/重排/过滤不改变
  身份。
- 重复 `id` → 具名结构错误（`树结构非法:重复节点 ID <id>`），不静默挑第一个，
  也不发布歧义可操作行。
- 空树是**正常空态**；未知 `expanded` ID 被忽略且不产生新节点；`selected` 不
  存在时不显示选中，且不擅自回调改业务选择。

### 3. 受控数据与回调

| prop | 语义 |
|---|---|
| `nodes` | 节点树 |
| `expanded` | 受控展开 ID 列表 |
| `selected` | 受控选中 ID |
| `disabledIds` | 禁用节点 |
| `loadingIds` | 子节点加载中 |
| `errorIds` | 加载失败 |
| `on_toggle(id)` / `on_select(id)` / `on_retry(id)` | 上行请求，载荷为节点 ID |
| `directory` | 目录显示模式（纯展示，不判断 OS 文件类型） |

- 展开/选择由**应用**更新；组件不修改受控值。
- chevron 与标签是**兄弟命中区**：chevron 只请求展开、标签只请求选择，一次手势
  最多一条回调（嵌套可点击会在 Vue 冒泡下双触发）。
- 行点击不折叠目录。

### 4. 禁用、加载与错误

- 禁用节点：不产生选择/展开动作，**不进入键盘可达序**；仍可见（供读屏/MCP
  观察），其可用子树保持可达。
- 加载中 ≠ 失败：`loadingIds` 只显示「加载中」；`errorIds` 才显示「加载失败」+
  独立「重试」。不得用空 `children` 伪装失败。
- 空树显示弱化文本空态，是正常状态而非错误。

### 5. 键盘

| 键 | 行为 |
|---|---|
| ↑ / ↓ | 在可见且可用行间移动；到两端**不回绕** |
| Home / End | 首/末可用行 |
| → | 已展开父节点进入首个子节点；未展开则请求展开 |
| ← | 已展开则请求折叠；否则焦点落到可见父节点 |
| Enter | 请求选择当前焦点行，**不**改展开 |

> 绑定面精确为 `onkeydown.{up,down,left,right,home,end,enter}`。**Space 未绑定**
> ——组件注释里写作"Enter/Space 只请求选择"，但实际只有 `enter` 有按键绑定与
> 测试覆盖；Spec 不得声称 Space 可用，除非先补绑定与用例。

焦点与选择分离：焦点不写业务 `selected`；Tab 不被逐节点吞掉。

### 6. 渲染约束（VM 轨实测）

- **行视觉状态必须用字面量 class 分支**（disabled × selected|focused 四支）。
  `class:` 的 f-string **不插值 for 循环记录字段**（渲染为空），同记录的
  `text f"…"` 正常。
- 图标用**字面量**名分支（Vue 轨动态 icon name 落占位）。
- 空态由 `tv_derive_rows` 的**哨兵行**承载，而非独立 `is_empty` computed：
  后者对 `.nodes` 变更**保持陈旧**（实测 `rows` 正确重派生 2,0,2,0 而空态分支
  从不渲染；第一帧即空的树反而正常）。

### 7. MCP 可观察面

- 行展开/选择/重试为独立可命中节点；`autoui_action press` 按稳定 `vnode_` id
  派发并携带节点 ID。
- 行状态在 class → 快照 `style` 可见（`ring-2` / `bg-accent` / `opacity-50`），
  故「焦点 vs 选中」「禁用」「加载中/失败」对 MCP 可断。
- 节点 ID 绑定当前渲染修订；数据变化后旧 `vnode_` 必须被拒绝，不得跨版本复用。

### 8. 支持面与验收

- Windows native VM 与 Vue 双端同语义；Linux native 与统一回归按各计划
  closeout 门记录（缺环境 `not_run`，不记 pass）。
- 验收不接受"只编译"或"只有截图"：须有真实窗口/页面操作 + 状态读回。

---


## SD-02 → add `docs/specs/ui/tree-view.md`（验证面；覆盖 AC-03, AC-04, AC-06）

验收与可观测面作为同一 Spec 文档的独立增补条目：

- 双端验收：Windows native VM 与 Vue 必须同语义通过；Linux native 与统一回归按
  各计划 closeout 门记录，缺环境记 `not_run`，**不得记 pass**。
- MCP 稳定绑定：行/展开/重试动作按稳定 `vnode_` id 命中；`vnode_` 绑定当前渲染
  修订，**数据变化后旧 id 必须被拒绝**，不得按位置重定向（实测 native
  `stale_id_rejected: True`）。Vue 轨按 DOM 位置寻址，无修订绑定身份，该 oracle
  在 Vue 轨不适用并显式记为不适用，不记空过。
- 视觉证据：浅/深两套真实截图；空态、加载中、加载失败+重试、禁用、选中、焦点
  在同一张图中可区分。
- 代表规模与窄窗：1000 平铺 + 64 层嵌套的末行可达；窄窗下 label 不裁切或可滚动
  （**本计划未入库该证据，作为显式 pending 门，不记 pass**）。
- 验收不接受"只编译"或"只有截图"。
## SD-03 → modify `docs/specs/ui/examples-ownership.md`（正式包单源/示例接入/blueprint owner 不变；覆盖 AC-05）

在 §3 依赖定位之后追加一条：

- **TreeView 消费者（PLAN-085）**：`packages/tree-view` 为 TreeView 单源。
  `examples/ui/026-database` 与 `examples/ui/041-auto-edit` 改为 `use` 该包，
  各自 `components/treeview.at` 退役。
  - 示例**保留自身业务代码**：026 的 `components/tree_util.at`
    （`filter_tree`/`collect_ids`/`toggle_id`）与 041 的 `editor_store` 及
    `use bps.navigation.filetree.tree_util: toggle_id` 不变。
  - 语言 blueprint 仍是**通用树工具**的 canonical owner；包内
    `tree_view_util.at` 只持 UI 交互投影，不构成第二真源。
  - 包路径深度须与同文件已工作的 `pagination`/`file_dialog` 一致
    （`src/front` 起五级到仓根）。
  - `TreeView` 托管在 **App 根 widget**（两消费者与示例的形态）；路由页形态
    的回调差异由 `plan085_routed_callbacks` 在库级守护，不在本 Spec 承诺。

## 附记（非 SD，不改 canonical）

- `examples/ui/018-book-reader` 与 `examples/ui/027-file-manager` **没有**
  `TreeView (` 调用位，其 `components/treeview.at` 是从未实例化的副本。
  正式包的验收**不以它们为消费者证据**；清理归后续计划，避免在交付记录里
  虚构四消费者证据。
- 既有 `FileTree` 形态（018/026 的 `components/filetree.at`）本计划不动。
