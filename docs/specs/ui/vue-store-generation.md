# Vue 店生成发射（vue store generation）

> PLAN-051 新增合同。店模型（`store { model { ... } }`）与 main.ts 脚手架
> 的 TS 发射面。分类：ui ｜ 状态：active ｜ 来源计划：
> docs/plans/051-store-factory-and-main-ts-regressions.md

## 1. 店模型初始化器（PLAN-051 SD-01）

- 标量/数组/map/`*.new` 容器与结构字面量初始化器一律按值发射：
  - 标量：字面量（str 用单引号、int/bool 直书）；
  - 数组 `[]` → `[]`；map `{}`/`{ k: v }` → 对象字面量（plan-012 gap 15）；
  - `*.new([])` 容器构造 → `[]`（plan-043 cat）；
  - **结构字面量 `Type{ field: value, ... }`（Expr::Node）→ 对象字面量
    `{ field: value, ... }`**（PLAN-051：字段递归同函数；`loop` 节点保持
    保守 null——该形态实际到不了店初始化器，纯防御）。不再回落 `null`：
    null 默认曾使 `ref<any>(null)` 的店在首读字段即
    "Cannot read properties of null"（消费方 BLOG-005 作者页实录）。
- 生成形态：`const <name> = ref<<ts-type>>(<init>)`（User 等用户型 ts-type
  回退 `any`，plan-041 E(e) 后续 typed 化不在本合同）。

## 2. main.ts 相邻 import 分隔（PLAN-051 SD-01）

- `generate_main_ts` 的相邻 import 片段（base/i18n/router/店上下文）之间
  恒有换行；有/无路由 × 有/无店上下文四组合输出均须为合法 TS
  （回归：`from './router'import { … }` 冷 scaffold 语法错）。
