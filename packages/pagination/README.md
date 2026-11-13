# packages/pagination — PaginationControl（PLAN-086）

统一受控分页组件：公共 1-based 页码；同一份数据在 Windows/Linux native VM 与 Vue
中呈现相同页数、边界与回调。组件不持有第二份 page、不切片数据、不发业务请求；
宿主数据变化后由消费方更新受控值，组件只投影。

视觉参考：[文件导航与轻量交互组件视觉参考 §2.3](../../docs/design/autoui/file-navigation-and-interaction-visual-guide.md)
（多页/首面/单页/空结果四状态）；合同与执行记录：[PLAN-086](../../docs/plans/086-shared-pagination-control.md)。

## 组件

| 文件 | 作用 |
|---|---|
| `pagination_control.at` | `widget PaginationControl` — 唯一高层入口 |
| `package.at` | 包清单（name/version/namespace/description） |
| `pac.at` | dep 物化读取的 pac 描述（缺它 dep 根不解析） |

> 命名约束：`pagination` 及 `pagination_*`/`paginationlink`/`paginationprevious`
> 是内置 schema tag（Builtin > Local > Package 硬编码优先级），包组件不可占用；
> 高层入口定名 `PaginationControl`（fold 键无冲突）。

## Props

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `page` | `int = 1` | 受控页码（1-based）。越界只投影 clamp，不发事件 |
| `pageSize` | `int = 10` | 每页条数；`<=0` 为具名配置错误 |
| `total` | `int = 0` | 条目数（不是页数）；`0` 是正常空结果；`<0` 具名配置错误 |
| `hideWhenSingle` | `bool = true` | `pageCount<=1` 时整组（含容量按钮）不入树；需要空/单页仍改容量的消费者传 `false` |
| `compact` | `bool = false` | 紧凑布局：仅上/下 + 当前/总页 |
| `disabled` | `bool = false` | 全部按钮禁用 |
| `showPageSize` | `bool = false` | 容量切换按钮组可见性 |
| `pageSizes` | `List = [10, 25, 50]` | 容量候选（去重保序；非正数具名错误；当前 pageSize 不在候选时挂载期补尾） |

## 事件

| 事件 | 载荷 | 语义 |
|---|---|---|
| `"changed"(str)` | JSON `{"page":N,"pageSize":M}` | 唯一上行通道。一个用户操作最多一条；边界/重复/disabled 零事件 |
| （错误） | 可见具名文本 | 非法配置渲染 `pagination: …` 错误文本（MCP 可读），不刷回调、导航整组隐藏 |

> 绑定形态：`onchanged: .Handler($event)`；体内 `."changed"(json.stringify({...}))`
> 是唯一 emit 通道（msg 变体必须带引号声明）。非法配置走可见文本而非 `error`
> 事件——native 轨子件 Init 内 emit 不派发（PLAN-086 T-01 探针实证）。

## 消费方式（依赖闭包三件套）

```at
// 1) pac.at 声明 dep 根（vue 轨 Phase-1c 扫描发出 SFC；路径相对本 pac）
dep pagination { path: "../../packages/pagination" }

// 2) widget 内包导入（native 轨 VM sweep 传递装载；路径相对声明文件）
use { package: pagination from "../../packages/pagination" }

// 3) 受控回路：changed → 应用更新 page/pageSize → 切片真实数据 → 传回投影
PaginationControl (page: .page, pageSize: .pageSize, total: .filteredCount,
                   showPageSize: true, pageSizes: [10, 25, 50],
                   onchanged: .PageChanged($event)) {}
```

脱离源根的 vendor/移根：把 `packages/pagination` 整目录（三件套）复制到第二根，
两条路径都指过去；缺依赖在 build 轨具名失败（native 轨静默缺席，已记录）。
026-database 与 packages/file-dialog 是官方消费样例；`examples/pagination` 是
四状态最小演示（产品门 `tests/product/verify_pagination.py` 以其为夹具）。

## 已知边界（v1）

- 容量候选 `pageSizes` 是挂载期静态配置；动态换出候选外的 `pageSize` 需重挂载（`key` 变更）。
- 页容量切换按钮为标准按钮形态（026 现形一致）；下拉（select）视觉留作后续。
- checkbox 的 `text:` 在 vue 轨是选中指示器 slot 内容（shadcn 惯例，全仓既有行为）——
  分页组件不使用 checkbox，不受影响。
- Linux native 轨门未运行（本仓库工作环境无 Linux 工具链；门脚本已带
  `WINIT_UNIX_BACKEND=x11` 适配，runner 就绪即可执行）。
