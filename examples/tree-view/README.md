# examples/tree-view — TreeView 受控示例

`packages/tree-view` 的自托管示例，同时是产品门
`tests/product/verify_tree_view.py` 的夹具（隔离目录运行，每次拷出仓外）。

视觉参考：[文件导航与轻量交互组件视觉参考 §2.2](../../../docs/design/autoui/file-navigation-and-interaction-visual-guide.md)。
包合同与 props/事件表见 [packages/tree-view/README.md](../../../packages/tree-view/README.md)。
计划：[PLAN-085](../../../docs/plans/085-official-tree-view-component.md)。

## 形态

树托管在 **App 根 widget**（`widget App`，无 routes），与两个真实消费者
（`examples/ui/026-database`、`examples/ui/041-auto-edit`）一致——包的回调在
"树挂根 widget"与"树挂路由页"之间有已知形态差异，本示例取已验证可用的前者。

窗口 `900x700`，`front_port: 4291`；`dep tree_view` 指向包根。

## 页面内容

| 区块 | 作用 |
|---|---|
| 顶栏 | 标题 + 「切换明暗」 |
| 控制条 | 「A 加载中」「A 加载失败」「A 清空」「B 加载中」「退出夹具」 |
| A 树（左） | 受控四态演示 + 禁用行 + 一行摘要 `A 树 sel=… exp=… <最近事件>` |
| B 树（右） | 同节点 ID、不同内容，验证两实例互不串扰 |

两棵树的 `aSummary` / `bSummary` 是**消费方内容摘要**（视觉指南要求用真实状态
读回证明），不是组件自带文本；组件本身不输出任何状态文案。

## 受控回路

```
chevron 点击 → on_toggle → 应用改 expanded  → 行重派生
行点击       → on_select → 应用改 selected  → 选中样式重派生
「重试」     → on_retry  → 应用清 errorIds  → 错误行恢复
A 清空       → aNodes = []                  → 空态文本出现
```

摘要里的 `exp=` 由 handler 在**修改列表之后**才写入（不是 f-string 里取
`.len()`），因为对列表参数取 `.len()` 在 VM 轨渲染为空。

## 种子数据

- A：`chapter-01`（展开，含子 `section-1.1`）、`images`（加载中 + 加载失败，
  故同时显示「加载中」和「重试」）、`chapter-02`（禁用）、`notes`。
- B：与 A **同 ID**（`chapter-01`、`notes`）但内容不同，用来证明两个受控实例
  的状态互不影响。

## 运行

```
auto run -r vm          # 真实 native 窗口
auto run                # Vue dev server + 浏览器
```

产品门（推荐，隔离目录 + 证据落盘）：

```
python tests/product/verify_tree_view.py \
  --auto-bin <auto-cli> --ui-tool-bin <auto-ui-tool> \
  --out-dir <fresh-dir> --backend native   # 或 vue
```

门覆盖：受控身份（选择/展开载荷与回读）、chevron 展开不改变选择、retry 恢复、
双实例隔离、空态、浅深截图、文档化退出与重开（native）。退出夹具用
「退出夹具」按钮，`Process.exit(0)`。
