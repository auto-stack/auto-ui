# PLAN-086 T-01 决议 — PaginationControl API 冻结与真实运行探针

日期：2026-11-14。基线：master `93e6b924`（观察基线 4dd64b5f 的相关事实复核一致）。
工作树：`D:/autostack/.wt/ui-086/auto-ui`，分支 `codex/ui-086-pagination`。
探针工件：`D:/autostack/plan-artifacts/plan-086/probes/`（仓库外 scratch，按证据政策不入库）。

## 1. 组件命名与身份（AC-01）

- 高层入口定为 **`PaginationControl`**（fold key `paginationcontrol`）。内置族 fold 键
  `pagination`、`pagination_*` 全系及 `paginationlink`/`paginationprevious` 均为 builtin
  （`schema/aura.at:5103-5239`，tier `web_component`），按
  `component_registry.rs:353-384` 的 Builtin > Local > Package 硬编码优先级会压制同名
  包组件——本计划不改优先级、不占用内置名。
- 包目录：`packages/pagination/{package.at, pac.at, pagination_control.at, README.md}`。
  `pac.at` 必须存在：auto-man 的 dep 物化按 `<dep.path>/pac.at` 读取（探针实证）。

## 2. 底层复用决策（AC-01）

内置 pagination 族 **native 全系无转换**，双证据：

- 代码：`convert_element.rs:1662-1680` 无 pagination 臂 → 未知 tag 落 Column/Empty
  fallback；`element_coverage.rs:369-378` 全系 `NotConsumed("shadcn web 长尾")`；schema
  `iced: "none"`。
- 实跑（P1 探针，native `auto run -r vm` + MCP snapshot）：9 个族成员在渲染树中
  全部为空，无 fallback 内容、无崩溃。

Vue 轨内置族映射真实可用（P1 `auto build` 实证）：`pagination` → reka-ui
`<Pagination v-model:page :total :items-per-page>`，`pagination_list/item/ellipsis/next`
真实组件；`pagination_first/prev/last` 无 vue 映射（div fallback）。内置族保持原样不动。

**决议**：PaginationControl 用标准 `button`/`text` 控件组合实现单一受控算法，双端同构；
不宣称内置族 native 完整实现，也不改 builtin 解析优先级（与计划 §2 预案一致）。

## 3. 冻结 API（探针验证）

```at
widget PaginationControl (
    page: int = 1,
    pageSize: int = 10,
    total: int = 0,
    hideWhenSingle: bool = true,
    compact: bool = false,
    disabled: bool = false,
    showPageSize: bool = false,
    pageSizes: List = [10, 25, 50]
) {
    msg { Init, First, Prev, Next, Last, Size(int), "changed"(str), "error"(str) }
}
```

- 事件绑定：消费方 `onchanged: .Handler($event)`；载荷 JSON
  `{"page":N,"pageSize":M}`（str）。`."changed"(...)` 体内引号 emit 是唯一上行通道。
- **msg 变体必须带引号声明**（`"changed"(str)`）：vue 轨 emit 重写只识别引号变体——
  非引号 `changed(str)` 生成裸 `changed(...)` 调用导致 `vue-tsc TS2304`（探针实证，
  file-dialog `"accepted"(str)` 同形）。
- 派生（Init 内，逐帧重放、幂等；PLAN-437 子件 Init 补发）：
  - `page_count = total / pageSize + (total % pageSize > 0 ? 1 : 0)`，下限 1；
    不用 `total+pageSize-1` 形态（防 i32 溢出）。
  - `current = clamp(page, 1, page_count)`，越界只投影不发 changed。
  - `pageSize <= 0` / `total < 0`：导航隐藏 + `."error"(具名错误)`，不除零、
    不刷回调（错误只在配置变化帧发一次）。
  - 单页隐藏：`hideWhenSingle=true 且 page_count<=1` 时**整个组**（含容量按钮）
    条件化不入树（file-dialog `if .pageCount > 1` 同形）；错误配置不被隐藏掩盖。
- 页容量切换：`pageSizes` 去重保序渲染为标准按钮（026 现形一致，MCP press 可驱动；
  内置 `select` 控件无 .at 用例先例，留作后续选项）。选新值发一次
  `{page:1,pageSize:N}`，同值不发。

## 4. emit 与实例路由机制（探针验证）

- vue 轨：体内 `."changed"(x)` → `function changed(arg){ emit('changed', arg) }` +
  emit 尾挂（探针生成物实证）。
- native 轨：PLAN-576 G3 `__emit_msg/__emit_payload` 桥 →
  `vm_bridge.instance_callback` **实例键**路由（child≠root 时优先实例表）；
  PLAN-051 C2① 声明式回送为辅。
- **双实例独立性实证**：同页两个 PaginationControl 实例 + 一个嵌套实例
  （App→FdMini→PaginationControl），分别 press 后各自 page 独立更新
  （P1 1→2 而 P2 不动；P2 last→101/101），MCP handler 路径含实例索引
  （`.PaginationControl demo@//PaginationControl#1.Next`、
  `//FdMini#0/PaginationControl#0.Next`）。child_emit v1"同名全局单路由"限度
  被实例键表覆盖，V-03 双实例语义可行。
- 载荷实证：嵌套链 `PaginationControl."changed" → FdMini.MiniTick` 收到
  `{"page":2,"pageSize":10}`。

## 5. 包依赖闭包与语法约束（探针验证，AC-01/06）

- widget 级 `use { package: X from "相对目录" }` 相对**声明文件父目录**解析；
  **传递依赖（包→包→消费方）在双轨均可用**：native 走 `dynamic_build.rs:296-395`
  visited 扫描（含包文件自身 use 链）；vue 走 `load_package` + 依赖物化。
- vue 生成要求消费方 `pac.at` 声明 `dep X { path: "..." }`（auto-man 物化到
  `deps/<name>/`，Phase-1c 扫 dep 根）；生成的 `pages/*.vue` 正确 import
  `@/components/{Widget}.vue` 并以 `:prop`/`@changed` 接线（探针生成物实证）。
- **widget prop 签名语法为 `name: type = default`**（冒号+默认值必需）；
  裸 `name type` 形态报 "unexpected token" 且**消费方实例静默不渲染**
  （探针踩坑实证）。
- **V-06 负例**：删必要依赖 → vue/build 轨具名失败（`Failed to read config file
  deps/<name>\pac.at`，不静默捡旧组件）；**native 轨缺失依赖 = 静默缺席**
  （`load_package` 失败仅 `log::warn!`，无 logger 时不可见；消费方实例不渲染）。
  门设计：tools 层负例用 build 轨具名失败断言；native 静默缺席作为已记录风险
  （无害但不诊断），不在本计划内改 VM 行为。

## 6. 整数与边界（AC-02）

- 运行时状态数值 = `auto_val::Value::Int(i32)`（`state_migration.rs:285-290` 对
  I64/U64 静默截断）；vue 字面量提取仅 `Int(i32)`/数值串。**契约**：total 语义
  为条目数（i32 域）；派生公式防溢出；文件窗 pageSize=100 × i32 上限页数足够。
- V-01 断言用真实包派生路径（脚本不得只重写 ceil 公式）。

## 7. Runner 与 Q-01（AC-06/07）

- native Windows runner 实证：`auto run -r vm` + `AUTOUI_MCP_PORT`；MCP
  snapshot/action/press、窗口归属、退出回收按 NVM-01..06（`docs/specs/runtime/
  native-vm-app-validation.md`）。026 读回路径 = `desktop_mcp.py`（state 读回）。
- vue runner 实证：`auto run`（vue）→ vite dev server（探针 127.0.0.1:3000）+
  VM 后端；**Python playwright 驱动真实 chromium** 完成渲染断言与点击链
  （实例隔离、末页跳转、父状态回写全部实证）。`verify_pagination.py --backend vue`
  采用此路径。
- Linux：WSL Ubuntu-22.04 在位（未验工具链/GUI，T-05 落实；若不可用按计划
  缺 runner 不记通过并如实报告）。
- 文字完整性：沿 file-dialog 先例（checkbox 语义名 + 图标按钮 tooltip 元数据
  `\uee03/\uee02` 匹配），PaginationControl 按钮带可访问名称/tooltip。

## 8. 消费者接线决议（AC-05）

- **FileDialog**（包内适配，公共 1-based）：`PaginationControl` 用
  `page: .displayPage`（= model.page+1，已有投影变量）、`pageSize: 100`、
  `total: .visibleFiles`（过滤后计数，对齐现 pageCount 口径）、`hideWhenSingle: true`
  （组件隐藏与现 `if .pageCount > 1` 语义一致）、无容量切换。`onchanged` →
  `fd_command = {op:"page", seq, index: event.page-1}`（复用现 fd_command 分页接线，
  不只改显示页码）。model 继续持有 0-based page、切片、过滤重置、选择清空。
- **026 Database**：`page/pageSize/totalRows` 直接绑定（1-based 现状）；
  `showPageSize: true`、`pageSizes: [10,25,50]`、`hideWhenSingle: false`；
  `onchanged` → 现有页码/容量 handler 泛化为接收 `{page,pageSize}`：更新 page、
  重切片 `activeRows`、维护 `showingText`。搜索/排序/总量缩减仍由应用协调。
- 首个交付顺序：先迁移两消费者（T-04），`examples/pagination/` 作为最小官方示例
  同批交付；TreeView/084 无交付依赖。

## 9. 探针清单（全部本地 scratch，含命令/输出）

| 探针 | 结论 |
|---|---|
| P1 builtin native（p1-builtin, run -r vm, MCP snapshot） | 全系成员渲染为空，无 native 支持 |
| P1 builtin vue build（p1-builtin, auto build） | reka-ui 映射可用；first/prev/last div fallback |
| P2 包闭包 native（p2-pkg-import, run -r vm） | 传递依赖 + 双实例 + 嵌套实例 + JSON 载荷全部实证 |
| P2 包闭包 vue build+browser（p2-pkg-import） | SFC 生成、emit 重写、vite build、playwright 点击链全实证 |
| P2-neg 缺依赖（p2-neg, build + run -r vm） | build 轨具名失败；native 轨静默缺席（已记录） |

复现命令与原始输出摘录见 `D:/autostack/plan-artifacts/plan-086/probes/*.log`。

## 10. T-03 追加决议（2026-11-14，r2 实施中）

- **派生架构定案**：Init 收敛挂载语义（PLAN-536 T3），受控投影改 **computed** 平铺派生
  （无相互引用；computed 体求值无 computed 兜底，链式引用静默 None——P3 全组消失实录）。
- **最小通用适配（eval.rs）**：视图/computed 快速路径 Bina 只有 Cmp/Logic/Add，
  Sub/Mul/Div/Mod 缺臂 → ceil 派生静默 None。补臂（worktree `54712797`），与 Add 同构，
  除零 → None。受影响面 = eval.rs 视图求值；P3 矩阵为定向回归；统一回归归 review/merge。
- **双端整除一致性**：JS float 除使 `total/ps+1` 出 11.03（vue 实录）；
  定案公式 `(total - total%ps)/ps + (total%ps > 0 ? 1 : 0)` 双轨整数精确。
- **错误文本静态化**：vue 轨 computed 内 f-string 插值生成 undefined（生成器缺口），
  错误文本改常量串（具名语义不变，值插值放弃）。
- **Init 内 emit 不派发**（native 实证 probe:no-err）→ 错误走可见文本，error 事件不设。
- **prop 字面量负数**：VM 轨 `total: -5` 静默失效（默认值），状态绑定 `.neg` 正常；
  负例测试须状态绑定（真实消费形态）。
- **键盘**：MCP keyboard（Tab/Enter/Space）在后台窗口探针中不触发激活；file-dialog 门的
  键盘路径在同一基础设施下通过（模态焦点），产品门（T-05）按同条件复验；组件不注册
  全局快捷键（§5 语义）。
- **修订绑定/身份**：MCP 元素身份=handler 绑定哈希（同 handler 持久合法）；卸载/隐藏后
  旧 id 拒绝实证（VNode not found）；双实例/嵌套实例路由独立（P2/P3）。
- **依赖工作树组**：`.wt/ui-086/{auto-lang,auto-web,auto-down}`（master 检出）+
  `.wt/ui-086/auto-man`（分支 `codex/ui-086-automan`，Cargo.toml 路径指向本组，仅构建
  管道不入 auto-man 主线）；补丁版 auto.exe/auto-ui-tool.exe 由该组产出（T-05 产品门
  使用，产物修订随本 Plan 记录）。
