# Vue Plain 模式原生元素投影（表格族与 alert-dialog）

> PLAN-112 T-02 规范落点（D-2 选项 ③，用户 2026-11-23 授权）。只写已验行为与接口。
> 分类：ui ｜ 状态：active ｜ 来源计划：docs/plans/112-autoui-sys-monitor-av-vue.md

## 1. 适用范围

`auto-ui-compiler/src/ui_gen/vue/`（`VueGenerator`）有两种发射模式与本次合同相关：

- **shadcn 模式**（产品默认；pac.at 未声明 `shadcn: off`）：组件族标签映射为 shadcn-vue
  组件（`<TableRow>`/`<TableCell>`/`<AlertDialog>`…），**本合同不改变它**。
- **Plain 模式**（`VueGenerator::new()`；A2Vue lane 产品道 `generate_sfc_with_lane_product*`
  与 `pac.at shadcn: off`）：此前 `table-header/row/head/cell`、`alert-dialog*` 没有映射臂，
  恒坍缩为 `div`/`span`——表格行不成行、单元格竖排；`:open` 落在常显的 div 上，对话框
  永驻文档流且不成模态。

## 2. 合同（Plain 模式）

| 源标签 | 发射 | 说明 |
|---|---|---|
| `table-header` / `table-body` | `<thead>` / `<tbody>` | 只收 kebab 组件名；PascalCase 仍是用户组件引用 |
| `table-row` | `<tr>` | `v-for`、`:key`、`@click`、`@contextmenu.prevent`、`:class` 照通用属性通道 |
| `table-head` / `table-cell` | `<th>` / `<td>` | `text:` 内容与 `style:` 类照通用通道 |
| `alert-dialog (open: X)` | `<div v-if="X" role="alertdialog" aria-modal="true" class="fixed inset-0 z-50 … bg-black/80" @click.self="X = false">` | **仅当带 `open`**；无 `open` 的嵌套形态走通用臂 |
| `alert-dialog-content` | `<div>` + 面板默认类 | 见 §3 |
| `alert-dialog-header` / `-footer` | `<div>` + 布局默认类 | |
| `alert-dialog-title` / `-description` | `<h2>` / `<p>` + 默认类 | |
| `alert-dialog-cancel` / `-action` | `<button class="…" @click="($event) => { X = false; return H(…) }">` | 关闭 + 显式调用 handler |

`alert-dialog-cancel`/`-action` 的 `@click` 必须是**箭头函数表达式**：

- 内联语句里裸函数名不会被调用，`return` 又不被 Vue 的表达式解析接受；
- 箭头函数按函数处理器对待，返回值（async handler 的 promise）交还 Vue——拒绝经
  Vue 的错误处理落 `console.error`，而不是未处理拒绝。与 shadcn 模式下纯方法处理器
  的错误路由一致。

按钮的文字来自位置参数（props 的 `text` 键），与通用臂同一条消费语义。

## 3. 默认类（`extract_classes`，Plain 模式）

键是 schema 规范名（snake_case）：面板 `bg-background text-foreground rounded-lg border
shadow-lg p-6 w-full max-w-lg`；标题 `text-lg font-semibold`；描述 `text-sm
text-muted-foreground`；取消 `inline-flex … border bg-background px-4 py-2 text-sm
font-medium`；确认同形而底色 `bg-primary text-primary-foreground`。shadcn 模式下这些标签
命中 shadcn 组件，`skip_defaults` 跳过，不发射。

## 4. 永久用例

- `crates/auto-ui-runtime/tests/plan112_a2vue_lane_store.rs`
  - `p112_lane_widgets_bind_the_store_surface`：025 的 lane 产物——原生 `<table>/<thead>/
    <tbody>/<tr>/<th>/<td>`、`v-if` 门控的模态根、按钮标签、关闭 + 显式调用。
  - `p112_generator_plain_native_table_and_modal_leave_shadcn_mode_untouched`：同一份源
    Plain ↔ shadcn 对照——Plain 落原生，shadcn 仍是 `TableRow`/`AlertDialog*`。
- `crates/auto-ui-runtime/tests/plan112_a2vue_sites.rs` 浏览器档：原 `025/tests/smoke.spec.ts`
  的结构选择器（`table tbody tr`、真实 `<button>`、模态）在真 Chrome 成立。
- 生成器统一回归：`cargo tu`（`auto-ui-compiler` lib `ui_gen::`）。

## 5. 边界（显名）

- `dialog`/`modal` 的通用形态不在本合同内（`modal` 仍是常显遮罩 + `:open`）。
- `badge` 仍是 `span` + 默认类；`variant` 属性照通用属性通道透传。
- 无 ESC 关闭；遮罩点击关闭与取消/确认自动关闭对齐 shadcn-vue AlertDialog 的既有行为。
- 波及面：`examples/ui` 无一声明 `shadcn: off`，Plain 只经 lane 产品道到达。仓内使用
  表格族标签的只有 025 与 026、使用 `alert-dialog` 的只有 025 / 027 / 041；其中只有
  025 与 026 有 lane 钉点——026（PLAN-113）的 lane 浏览器证据在新映射下复验通过，
  PLAN-1072 / PLAN-980 的 6 个浏览器挂载用例同样复验通过。
