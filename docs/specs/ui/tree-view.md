# TreeView — 正式受控树组件（docs/specs/ui/tree-view.md）

来源：PLAN-085 r2（SD-01/SD-02）。组件源 `packages/tree-view/`；自托管演示
`examples/tree-view/`；产品门 `tests/product/verify_tree_view.py`（native/vue）与
`tests/product/verify_tree_keyboard.py`（真实键盘）；tools 门
`crates/auto-ui-tools/tests/tree_view_component.rs`。

## 1. 单源与包形态

- 高层入口是包组件 **`TreeView`**（含 `TreeIcon` 与 UI 投影模块
  `tree_view_util.at`）。全仓只有一份 `treeview.at` 实现；旧示例内嵌副本已退役。
- **包内模块文件不会被自动导入**：使用方必须显式
  `use tree_view_util: …` / `use tree_icon: TreeIcon`。漏写时
  `computed => fn(...)` **静默产出空列表且无诊断**（实测）。包内亦以同式自用。
- 通用树算法（`flatten_tree`/`filter_tree`/`toggle_id`/`collect_ids`）的 canonical
  owner 仍是语言 blueprint `bps.navigation.filetree.tree_util`；本包只持 **UI 交互
  投影**（可达序、焦点序号、行状态旗标、图标派生），不构成第二真源。

## 2. 节点 schema 与身份

```
{ id: str, label: str, children: List, kind: str, icon: str, is_leaf: bool, badge: str }
```

- 字段**全键书写**：VM 的 Obj 字面量为形状锁定类型，缺键的字段访问是硬错。
- `id` 在一实例内**唯一且稳定**，与显示文本、行序分开；同标签/重排/过滤不改变身份。
- **空 `id` 是非法输入**，具名上报（`树结构非法:存在空节点 ID`）。
- **重复 `id` 是非法结构**，具名上报（`树结构非法:重复节点 ID <id>`），且**不发布
  歧义可操作行**——所有行转为惰性，避免同一 id 被选中两次。
- 空树是**正常空态**（弱化文本），不是错误。未知 `expanded` ID 被忽略；`selected`
  不存在时不显示选中，且不擅自回调改业务选择。

## 3. Props 与事件（冻结契约）

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `nodes` | `List = []` | 嵌套节点树 |
| `expanded` | `List = []` | 受控展开 ID 列表 |
| `selected` | `str = ""` | 受控选中 ID；空串 = 无选中 |
| `disabledIds` | `List = []` | 禁用节点：不产生选择/展开，且**不进入键盘可达序**；仍可见 |
| `loadingIds` | `List = []` | 子节点加载中：显示「加载中」 |
| `errorIds` | `List = []` | 加载失败：显示「加载失败」+ 独立「重试」 |
| `on_toggle` / `on_select` / `on_retry` | `msg` | 上行请求，载荷为该节点 ID |
| `directory` | `bool = false` | 目录显示模式（**纯展示**，不判断 OS 文件类型） |

- 展开/选择由**应用**更新；组件不修改受控值。
- chevron 与标签是**兄弟命中区**：chevron 只请求展开、标签只请求选择；一次手势最多
  一条回调（嵌套可点击会在 Vue 冒泡下双触发）。行点击**不**折叠目录。
- 加载中 ≠ 失败：`loadingIds` 只显示「加载中」；`errorIds` 才显示「加载失败」+ 重试。
  不得用空 `children` 伪装失败。
- 未加载的目录（`kind: "dir"`、`children` 空、`is_leaf: false`）**仍可请求展开**，
  不被当叶节点。

## 4. 键盘语义

绑定面精确为 `onkeydown.{up,down,left,right,home,end,enter}`。

| 键 | 行为 |
|---|---|
| ↑ / ↓ | 在可见且可用行间移动；到两端**不回绕** |
| Home / End | 首/末可用行 |
| → | 已展开父节点进入其首个可用子节点；未展开则请求展开 |
| ← | 已展开则请求折叠；否则焦点落到可见父节点 |
| Enter | 请求选择当前焦点行，**不**改展开 |

- **焦点与选择分离**：焦点不写业务 `selected`。
- **焦点修复**：数据更新使当前焦点不可见时（删除/过滤/所属子树折叠），焦点回落到
  最近**可见祖先**，再否则首个可用行。实现上用**结构父节点**（无条件下钻整棵树）而
  非可见父节点——折叠后被聚焦子节点已不在 `expanded` 中，可见祖先查找会找不到它并
  返回空串、使修复静默失效。
- `Space` **未绑定**按键通道（仅注释提及）；不得在补绑定与用例前声称可用。

## 5. 渲染约束（VM 轨实测）

- **行视觉状态必须用字面量 class 分支**（disabled × selected/focused 四支，另加
  selected+focused 组合支）。`class:` 的 f-string **不插值 for 循环记录字段**
  （渲染为空）；同记录的 `text f"…"` 正常。
- 图标用**字面量**名分支（Vue 轨动态 icon name 落占位）。
- 缩进按层渲字面量宽度，**超过 4 层封顶**（深嵌套不越出可视区）。
- **空态由派生哨兵行承载**，不用独立 `is_empty` computed：后者对 `.nodes` 变更
  **保持陈旧**（实测 `rows` 正确重派生 2,0,2,0 而空态分支从不渲染）。回归
  `plan085_empty_state`。

## 6. 重复 ID 检测（性能与正确性契约）

`tv_duplicate_id` 在**每次派生**执行，**对任意节点规模**，无阈值。

实现为**拼接字符串 + 子串包含**：`seen` 累积 `|esc(id)|` token，每结点一次
`contains`。成员检测走**原生**字符串搜索而非 VM 字节码迭代，1000 节点整套用例
约 2.1s（旧 `tv_member` 逐元素扫描 ≥14s，耗尽指令预算并渲染 0 行）。

三处不可回退的细节（每处都由实测缺陷换来）：

1. **`seen` 起始必须是 `""` 而非 `"|"`**：以 `"|"` 起步时首个 token 后 `seen` 为
   `||a|`，已含空 id 的 token `||`，空 id 会命中并返回 `""`——与"无重复"同一哨兵，
   扫描提前结束、后续真重复漏检。
2. **空 id 单独具名上报**，不混入 token 流：空 id 本身非法，且没有任何框架字符能把
   它与相邻 token 的边界区分开。
3. **转义必须单射**：`!` → `!0` 先、`|` → `!1` 后。只做 `|` → `!!` 不是单射——
   字面含 `!!` 的 id 会与含 `|` 的 id 撞同一 token（`a|b` 与 `a!!b` 均被误判重复）。

回归：`plan085_scale::n02_large_tree_duplicates_are_detected_at_any_size`、
`plan085_review_fixes::n02_{empty_id,pipe_in_id,escaping_is_injective,…}`。

## 7. MCP 可观察面

- 行/展开/重试为**可独立命中**的动作；`autoui_action press` 按稳定 `vnode_` id 派发
  并携带节点 ID。
- **动作标识绑定当前渲染修订**：数据变化后旧 `vnode_` 必须被**拒绝**，不得按位置
  重定向（native 实测 `stale_id_rejected: true`；Vue 轨按 DOM 位置寻址、无修订绑定
  身份，该 oracle 不适用并显式记为不适用）。
- 节点语义写在快照可读的 **raw class** 上（与 selected/focused/disabled 同一通道）：
  `tv-tree` 根标记、`tv-lvl-N` 行层级（1–4，>4 为 `tv-lvl-deep`）、`tv-dis` 惰性行、
  `tv-sel` 选中行。名称取行 label 文本；展开取 chevron 方向。
- **本组件不声称 ARIA 合规树**：runtime 无 role/aria/level 通道，DSL 写 `role:`
  会被静默接受后丢弃。该缺口具名保留，不得据此宣称无障碍达标。
- oracle 提示：上述标记会被**已解析的 debug View 丢弃**（样式解析只留已知 token），
  但存在于 `raw_class`——即快照打印的内容。对 debug View 断言会得到**假阴性**。

## 8. 支持面与验收

- Windows native VM 与 Vue 双端同语义；产品门两轨均须 pass。
- Linux native 腿与统一回归按各计划 closeout 门记录；缺环境记 `not_run`，**不得记
  pass**。
- 验收不接受"只编译"或"只有截图"：须有真实窗口/页面操作 + 状态读回。
- 代表规模：1000 平铺、**单根 1000 子节点**（后者是平铺夹具无法覆盖的形态）、64 层
  嵌套，末行均须可达。
