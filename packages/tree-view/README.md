# packages/tree-view — TreeView（PLAN-085）

统一受控树：文件、目录、章节、数据库对象等导航共用一套控件。节点、展开、选中、
禁用、加载中、加载失败全部由应用持有并传入，组件只做纯派生渲染 + 上行请求回调；
它不复制业务选择、不读文件系统、不发请求。

视觉参考：[文件导航与轻量交互组件视觉参考 §2.2](../../docs/design/autoui/file-navigation-and-interaction-visual-guide.md)
（缩进线/chevron、图标、label 在稳定列中；焦点与选择分离；loading 与 error/retry
独立可见）。合同与执行记录：[PLAN-085](../../docs/plans/085-official-tree-view-component.md)，
实施决策见 [docs/reviews/plan085/decisions.md](../../docs/reviews/plan085/decisions.md)。

## 组件

| 文件 | 作用 |
|---|---|
| `tree_view.at` | `widget TreeView` — 唯一高层入口 |
| `tree_view_util.at` | UI 交互投影：可见行派生、键盘可达序、焦点序号、图标派生 |
| `tree_icon.at` | `widget TreeIcon` — 有界 lucide 图标调色板 |
| `package.at` | 包清单（name/version/namespace/description） |
| `pac.at` | dep 物化读取的 pac 描述（缺它 dep 根不解析） |

> **包内模块文件不会被自动导入**。使用本包的 widget **必须**显式 `use`：
> `use tree_view_util: …`、`use tree_icon: TreeIcon`（包内以同式自用）。漏写时
> `computed => fn(...)` 会**静默产出空列表且无任何诊断**——这是实测过的坑，
> 不是理论风险。

## Props

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `nodes` | `List = []` | 嵌套节点树（schema 见下） |
| `expanded` | `List = []` | 受控展开 ID 列表 |
| `selected` | `str = ""` | 受控选中 ID；空串 = 无选中；不存在的 ID 不显示选中、不擅自回调改选择 |
| `disabledIds` | `List = []` | 禁用节点：不产生选择/展开动作，也不进入键盘可达序；仍可见（供读屏/MCP 观察） |
| `loadingIds` | `List = []` | 子节点加载中：该行显示「加载中」 |
| `errorIds` | `List = []` | 加载失败：该行显示「加载失败」+ 独立「重试」按钮 |
| `on_toggle` | `msg` | 请求展开/折叠（载荷：节点 ID） |
| `on_select` | `msg` | 请求选择（载荷：节点 ID） |
| `on_retry` | `msg` | 请求重试加载（载荷：节点 ID） |
| `directory` | `bool = false` | 目录显示模式：按目录/文件派生图标（**纯展示**，不判断 OS 文件类型） |

### 节点 schema

```
{ id: str, label: str, children: List, kind: str, icon: str, is_leaf: bool, badge: str }
```

字段必须**全键书写**——VM 的 Obj 字面量是形状锁定类型，缺键的字段访问是硬错。
`id` 在一实例内唯一且稳定，与显示文本/行号分开；重复 `id` 产出具名结构错误
（`树结构非法:重复节点 ID …`），不静默挑第一个。`icon` 非空时优先于自动派生。

## 事件与语义

| 行为 | 结果 |
|---|---|
| 点 chevron | 只请求展开/折叠；**不**请求选择 |
| 点标签行 | 只请求选择；**不**折叠目录 |
| 点「重试」 | 只请求重试该节点的加载 |
| Enter | 请求选择当前焦点行，**不**改展开（`Space` 未绑定按键通道；语义上同属
"激活"，但当前只有 Enter 有 `onkeydown.enter` 绑定与测试覆盖） |
| ↑ / ↓ | 在**可见且可用**行之间移动；到两端不动（不回绕） |
| Home / End | 跳首个/末个可用行 |
| → | 已展开的父节点：进入其首个可用子节点；未展开：请求展开 |
| ← | 已展开：请求折叠；否则焦点落到可见父节点 |

cheveron 与标签是**兄弟命中区**（嵌套可点击会在 Vue 冒泡下双触发），因此一次
手势最多产生一条回调。焦点与选择分离：焦点不写业务 `selected`。

## 空状态、加载与错误

- 空树渲染一条弱化文本 `空树无可操作行`，是**正常空态**，不是错误。
- **实现注记（重要）**：空态由 `tv_derive_rows` 产出的一条**哨兵行**承载
  （`empty: true`、空 id、disabled），而不是独立的 `is_empty` computed。原因是
  实测 `is_empty => tv_count_rows(.nodes,…) == 0` 在 VM 轨上对 `.nodes` 的变更
  **保持陈旧**：`nodes` 2→0→2→0 时 `rows` 每帧正确重派生（2,0,2,0）而空态分支
  一次都不渲染；"从第一帧就空"的树反而正常。空态因此挂在已证明 live 的派生上。
  回归：`crates/auto-ui-runtime/tests/plan085_empty_state.rs`。
- 加载中 ≠ 失败：`loadingIds` 只显示「加载中」；`errorIds` 才显示「加载失败」+
  重试。不得用空 `children` 伪装失败。

## 键盘与宿主的已知边界

- `onkeydown.*` 写在**页/根 widget** 上派发正常；本组件自身也声明了
  `onkeydown.*`。同页**多个**本组件实例时，键盘焦点归属按实例身份解析
  （`TypedInst`）；按键名合成的派发在 ≥2 存活实例时按既有合同不静默裁决。
- 已知并已隔离的形态差异：树托管在 **App 根 widget** 时（两个真实消费者 026/
  041 的形态）回调正常；托管在**路由页**时，真实窗口里行可渲染但按下不写页面
  状态（库路径正常，无诊断）。该差异由
  `crates/auto-ui-runtime/tests/plan085_routed_callbacks.rs` 在库级守护，不混入
  本包的产品门。

## 依赖

- 通用树工具（`flatten_tree`/`filter_tree`/`toggle_id`/`collect_ids`）的 canonical
  owner 仍是语言仓 blueprint `bps.navigation.filetree.tree_util`；本包**不复制**
  一份通用树工具，`tree_view_util.at` 只持有 UI 交互投影（可达序、焦点序号、
  行状态旗标、图标派生）。
- 图标为 lucide 名字；Vue 轨只发射**字面量**名（动态名落占位），故
  `tree_icon.at` 用有界分支。

## 用法

```auto
widget Page {
    msg { Init, TvToggle(str), TvSelect(str), TvRetry(str) }
    model {
        var nodes List = []
        var expanded List = []
        var sel str = ""
    }
    use { package: tree_view from "../../packages/tree-view" }
    view {
        TreeView (nodes: .nodes, expanded: .expanded, selected: .sel,
                  on_toggle: .TvToggle($event), on_select: .TvSelect($event),
                  on_retry: .TvRetry($event), directory: true) {}
    }
}
```

pac 里需要 dep 根（vue 轨扫描发出 SFC；native 轨由 widget 级 `use` 的 VM sweep
传递装载）：

```
dep tree_view { path: "../../packages/tree-view" }
```

## 验收入口

| 轨道 | 入口 | 结果 |
|---|---|---|
| 生成（双生成器 + 第二绝对根 vendor 闭包 + 具名缺依赖失败） | `cargo test -p auto-ui-tools --test tree_view_component` | 2/2 |
| 行为（键盘语义、实例隔离、空态、派发） | `cargo test -p auto-ui-runtime --test plan085_tree_keyboard --test plan085_empty_state --test plan085_keydown_ownership --test plan085_routed_callbacks` | 8+1+5+2 |
| 产品门（真实 native 窗口 / 真实 Vue 页面） | `python tests/product/verify_tree_view.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native\|vue` | 双端 pass，矩阵 `{a:5,b:2}` |

自托管示例：[`examples/tree-view/`](../../examples/tree-view/)（受控四态 + 同页
双实例 + 空树 + 浅深切换 + 文档化退出）。

## 节点语义（MCP 可观察面）

**本包不声称 ARIA 合规树**：runtime 没有 role/aria/level 通道（view/vnode/snapshot
三层均无），DSL 里写 `role: "tree"` 会被静默接受后丢弃。因此本包把两个此前只能
"推断"的事实，写在 **MCP 真正读取的那一面**（快照里的 `raw_class`），复用
selected/focused/disabled 早已使用的同一通道：

| 标记 | 位置 | 含义 |
|---|---|---|
| `tv-tree` | 树根容器 | 该子树是树（区别于普通列表） |
| `tv-lvl-1` … `tv-lvl-4` | 每行缩进单元 | 该行层级 1–4 |
| `tv-lvl-deep` | 每行缩进单元 | 层级 > 4（缩进封顶，不再累加） |
| `tv-dis` | 行 | 禁用/惰性行（不可选择） |
| `tv-sel` | 行 | 已选中（禁用与正常分支都带） |

其余语义沿用既有表示：**名称**取行 label 文本；**展开**取 chevron 图标方向；
**焦点**为 `ring-2`；**加载中/失败**为对应文本。

> oracle 提示：这些标记会被**已解析的 debug View 丢弃**（样式解析只保留认识的
> token），但存在于 `raw_class`——即快照打印的内容。拿 debug View 断言会得到
> **假阴性**。另外 vue 轨读 `inner_text`，看不到 class，故标记断言仅 native 执行。
## 大树的结构校验

重复 ID 检测在**每次派生**都执行，**对任意规模**——不再有节点数阈值。

早期版本用 `tv_member(seen, id)` 逐元素线性扫描，复杂度 O(n²)：1000 节点每帧约 50 万
次 VM 迭代、耗时约 14s，会耗尽指令预算并**渲染 0 行**；当时按节点数设了阈值来回避，
代价是超阈值后受控更新引入的重复 ID 检测不到（review N-02）。

现在 `tv_duplicate_id` 改为**拼接字符串 + 子串包含**：

```
seen = ""                          // 不能用 "|" 起步：那样首个 token 后 seen 就含 "||"，
                                   // 空 id 会命中并被当成「无重复」，导致扫描提前中止
每结点：
  if id == ""   -> 返回 "<empty-id>"（具名错误，与「无重复」的空串哨兵区分）
  esc  = id.replace("!", "!0").replace("|", "!1")   // 单射转义，顺序必须是 ! 先
  token = "|" + esc + "|"
  if seen.contains(token) -> 返回 id（真重复）
  seen = seen + token
```

三处细节都是踩坑换来的，改动前请先读：

- **起点是 `""` 而不是 `"|"`**：以 `"|"` 起步时首个 token 之后 `seen` 为 `||a|`，已含空
  id 的 token `||`，空 id 会命中并返回 `""`——与"无重复"同一个值，扫描提前结束、后续真
  重复漏检（review N-02a）。
- **空 id 单独具名上报**（`树结构非法:存在空节点 ID`），不混进 token 流——空 id 本身
  就是非法输入，且没有任何框架字符能把它与相邻 token 的边界区分开。
- **转义必须单射**：只把 `|` 换成 `!!` 不是单射——字面含 `!!` 的 id 会与含 `|` 的 id
  撞成同一个 token（`a|b` 与 `a!!b` 都被误判重复）。先 `!`→`!0` 再 `|`→`!1` 才是
  一一对应（review N-02b 后续）。

成员检测走**原生**字符串搜索而非 VM 字节码循环，实测 1000 节点整套用例 **2.13s**
（旧实现 ≥14s 且渲染 0 行）。id 两端包 `|` 防止 `ab` 误命中 `b` 的子串误报。
**所有 id 都走这条原生路径**——含 `|` 的 id 不再回退到逐元素比较。

因此：

| 树的总节点数 | 受控更新后的重复 ID 检测 |
|---|---|
| 任意 | **自动**——每帧重校验，重复立即发现，歧义行不再发布 |

**回归**：`plan085_scale::n02_large_tree_duplicates_are_detected_at_any_size` 断言
601 节点树在受控更新引入重复后**可操作行归零**；`f09_thousand_nodes_derive_every_row`
与 `n01_single_root_with_thousand_children_still_renders` 守住 1000 节点规模。