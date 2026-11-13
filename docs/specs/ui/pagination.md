# PaginationControl — 统一受控分页（docs/specs/ui/pagination.md）

来源：PLAN-086 r2（SD-01/SD-02）。组件源 `packages/pagination/`；最小演示
`examples/pagination/`；产品门 `tests/product/verify_pagination.py`（native/vue）与
tools 门 `crates/auto-ui-tools/tests/pagination_component.rs`。

## 身份与共存

- 高层入口是包组件 **`PaginationControl`**（`packages/pagination/`）。`pagination`
  与 `pagination_*` 全系、`paginationlink`、`paginationprevious` 是内置 schema tag
  （tier `web_component`），在 Builtin > Local > Package 硬编码优先级下压制同名
  包组件——本合同不改变该优先级，也不占用内置名。内置族行为保持原样：native 无
  转换臂（渲染为空），vue 经 shadcn/reka-ui 映射。
- 组件不持有第二份 page、不切片数据、不发业务请求；宿主数据变化后由消费方更新
  受控值，组件只投影。页容量切换/搜索/排序/总量缩减的协调归消费方。

## Props 与事件（冻结契约）

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `page` | `int = 1` | 受控页码，公共 1-based。越界只投影 clamp，不自发 changed、不改宿主状态 |
| `pageSize` | `int = 10` | 每页条数；`<=0` 具名配置错误 |
| `total` | `int = 0` | 条目数（不是页数）；`0` 是正常空结果（pageCount=1）；`<0` 具名配置错误 |
| `hideWhenSingle` | `bool = true` | `pageCount<=1` 时整个分页组（含容量按钮）不入可操作树；需要空/单页仍改容量的消费者显式传 `false` |
| `compact` | `bool = false` | 紧凑布局：仅上/下 + 当前/总页；常规模式加首/末 |
| `disabled` | `bool = false` | 全部按钮禁用 |
| `showPageSize` | `bool = false` | 页容量切换按钮组可见性 |
| `pageSizes` | `List = [10, 25, 50]` | 容量候选：去重保序；含非正数具名配置错误；当前 pageSize 不在候选时挂载期补尾显示 |

事件：`"changed"(str)` 携带 JSON `{"page":N,"pageSize":M}`，是唯一上行通道；
消费方以 `onchanged: .Handler($event)` 绑定。一个用户操作最多一条 changed；
边界/重复/disabled/同页零事件。换容量只发一次 `{page:1,pageSize:N}`，相同值不发。

非法配置（`pageSize<=0`、`total<0`、`pageSizes` 含非正数）是具名配置错误：错误以
可见文本 `pagination: …` 呈现（MCP snapshot/state 可读），导航整组隐藏、错误不被
单页隐藏掩盖、不因反复渲染刷回调。错误不走事件——native 轨子件 Init 内 emit 不派发
（PLAN-086 T-01 实证），可见文本是双端一致的具名可观察面。

## 派生语义（双端一致）

- `pageCount = (total - total % pageSize) / pageSize + (total % pageSize > 0 ? 1 : 0)`，
  下限 1。该形态在 native 整数除与 JS float 除下同得整数（防溢出：不用
  `total+pageSize-1`；运行时状态数值为 i32 域）。
- `current = clamp(page, 1, pageCount)`；`canPrev = page > 1 且 pageCount > 1`；
  `canNext = page < pageCount`；`groupVisible = pageCount > 1 || !hideWhenSingle`。
- 响应式派生为无相互引用的平铺 computed（Init 收敛为挂载语义，PLAN-536）；props
  每帧重播种，computed 随视图重建重估——受控 page/total 变化即时反映到页码/边界/隐藏。

## 消费方式与依赖闭包

三件套缺一不可：

1. 消费方 `pac.at`：`dep pagination { path: "…" }`（vue 轨 dep 物化 + Phase-1c 扫描
   发出 SFC；物化读取 `<path>/pac.at`）。
2. widget 内：`use { package: pagination from "…" }`（相对声明文件解析；native 轨
   VM sweep 传递装载，包→包嵌套可用）。
3. 受控回路：`changed` → 应用更新 page/pageSize → 重切片真实数据 → 传回投影；
   显示页必须与真实行/查询一致（跨层 oracle）。

移根/vendor：整目录三件套（`package.at`/`pac.at`/`pagination_control.at`）复制到第二
根并保持字节一致；缺依赖在 build 轨具名失败（native 轨缺失=静默缺席，已记录——
native 消费者必须依赖 build 轨负例门）。包组件不得命名 `pagination`/`pagination_*`
折叠冲突名。

## 交付门（SD-02）

- Win native VM：真实窗口（NVM-01…06）、MCP 快照/操作、四状态、跨层 oracle、
  卸载后旧 id 拒绝、退出/端口回收/重开（`tests/product/verify_pagination.py`）。
- Vue：vite + 真实浏览器（playwright）同一矩阵。
- 视觉：浅/深主题、窄窗（420×560）分页单行、按钮名称/tooltip 与禁用清楚
  （视觉参考 §2.3）。V-08 截图带 sha256 入 report.json。
- Linux native 轨：门脚本已适配（`WINIT_UNIX_BACKEND=x11`）；无 runner 环境不记
  通过（PLAN-086 交付时显式记录未运行）。
- tools 门：双生成器、vue 面 float-safe ceil 形态断言、vendor 闭包、strict 具名
  失败负例（`pagination_component.rs`）。

## 已知边界（v1）

- `pageSizes` 为挂载期静态配置；动态换出候选外的 `pageSize` 需重挂载（`key` 变更）。
- 容量切换按钮为标准按钮形态；下拉视觉留作后续。
- 键盘：按钮沿宿主焦点链（Tab/Enter/Space 为运行时焦点管理语义，组件不注册全局
  快捷键）；后台窗探针不触发激活为已知基础设施行为。
