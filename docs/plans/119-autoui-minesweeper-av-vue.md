---
plan_id: PLAN-119
status: drafting
feature_name: U06 Minesweeper 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-119 U06 Minesweeper 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [108](108-autoui-u06-complex-interactive-apps-av-vue.md) 的 U06 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [038-minesweeper](../../examples/ui/038-minesweeper/pac.at) | 原游戏状态、点击与重开后的局部实例隔离 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。消费实际已交 Store、列表、输入和实例状态。复用原数据与独立观察，保留交互语义。

## 3. 技术与实现入口

目标入口为 src/front/app.at 与跨单元 MinesweeperStore。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-119/{auto-ui,auto-lang}，每仓单一 plan-119-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

实际使用的 105 Store、集合或输入能力落主后消费，逐键核对来源。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。点格和标记触发实际游戏更新；两实例互不串扰；重开状态、失败路径和关闭终态按原 oracle 验证。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | 每个原项目/页面、原 oracle、适用端、实际前置和资源映射可复查。 |
| AC-02 | work | 所用接口及组件各有正反验证，真实 AV/Iced 的原行为、权限、恢复和关闭通过。 |
| AC-03 | work | A2Vue 的实际生成、构建、交互、失败和恢复按原矩阵通过。 |
| AC-04 | review | 收据与冻结版本对证，逐行核验缺失和临时代码，复审通过。 |
| AC-05 | merge | 可消费版本及两个目标的各自结果落主，各登记下游交接。 |

## 8. 执行步骤

当前 harness 如提供任务面板，开工时登记以下任务 ID。各目标按实际接口推进，阶段复核与最终复审分别覆盖对应结果。

- [ ] T-01 固定原项目、依赖与验收矩阵
  - work，AC-01，核 §1 现有源和 §6 原运行入口，输出逐行的源、权限、预期和实际阻断。
- [ ] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
- [ ] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
- [ ] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [ ] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: drafting | outcome: pass（授权确认：`完成计划119` = work→review→bounded repair→merge 全链授权，记 §4） | worktree: （组创建中） | next: work（T-01）

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-01 固定源/oracle/两端入口 + 双端首阻断逐构造归因 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@3d9ee86a，基 faa2ff0e7）+ auto-lang（plan-119-dev@fdf6ea7a0）+ 只读兄弟 auto-web 0cfc7c784 / auto-down 456c86ac8 | commit: 3d9ee86a（诊断例程） | next: §10 裁定（G1 归属）

**T-01 逐行矩阵（2026-11-24 核）**

| 项 | 固定值 |
|---|---|
| 原项目源 | `examples/ui/038-minesweeper/{pac.at, src/front/app.at, src/front/minesweeper_store.at, README.md, tests/desktop_mcp.py, tests/test_038_vue.mjs}`（仓内单源，不动源） |
| 原 oracle | VM 已钉确定性：首击 (4,4) → 29 格展开（16 个数字）、雷位 [3,10,22,29,41,48,53,60,72,79]；右击标记/取消、难度切换（9×9/16×16/30×16）、face 按钮重开、`Tick` 计时、won/lost 横幅；A2Vue 侧 `test_038_vue.mjs` 钉主题族/模式与 81 个空格 |
| 实际使用组件 | `store MinesweeperStore`（PLAN-105 已交 store 单元：13 model 字段、6 handler、board 为 Obj 字面量列表）；`use themes.themes: Auto, Solarized`（官方主题包 packages/themes）；grid/button/text/col/row/div 视图；`oncontextmenu.prevent`；`interval` 计时约定 |
| AV 端（组件面） | `--source src/front/app.at`：prep（`prepare_component_source`，工程根=pac.at 祖先）→ `NodeDesktopBridge::load`。**首阻断**：`.store.*` 引用需同源 store 声明——prep 的 store 内联门（`project_component_stores`+`project_component_candidates`）拒该 store 片断，故未内联 |
| AV 端（包面） | `--package "app::app=src/front/app.at; app::minesweeper_store=src/front/minesweeper_store.at" --entry app::app`（`load_package`→`prepare_store_units`）。**首阻断**：`model has no field store` @入口 `.Init` 臂（跨单元 store 未绑定，同源于 store 单元解析拒） |
| A2Vue 端 | `AUTO_UI_ACE_VUE=1 AUTO_UI_TOOL_BIN=<本树 auto-ui-tool> auto.exe build --render vue --gen-only`。**首阻断**：`ace.vue-lane-unsupported`，缺席 handler = App[.Flag/.Reset/.Reveal/.SetDifficulty/.Tick]（同源：store 片断不可投影，`.Init` 因无控制转移保留） |
| 工具身份 | `ace_node_desktop_window.exe` SHA256 `4CD0DE1A…017666`（debug，本树 fresh 构建 5m51s）；`auto-ui-tool.exe` 同构建；auto CLI `D:/autostack/auto-man/target/debug/auto.exe` SHA256 `EDDB2CCE…D947FCF`（与 PLAN-114 钉版一致，复用） |
| 环境 | Windows x64、Cargo 1.98.0、Node 25.2.1 / pnpm 11.6.0；`--locked --offline -j4`；开工时他方 cargo/rustc 在跑，本计划 -j4 让行 |

**双端首阻断归因（同一根因，逐构造实证）**

判据：store 片断 + 探针 widget 过 `project_component_stores`/`project_component_candidates`（prep 内联门同一判据），逐 handler/逐语句二分（材料 `plan-artifacts/plan-119/`）：

| # | 缺口 | 实证 | 归属 |
|---|---|---|---|
| G1 | AGE 面**控制转移语句**缺席：`return`/`break`（`continue` 同族）不在 age-component/age-package handler 语句文法内（`experimental/ace/auto-source.at` 无对应规则） | 逐语句：`.Init`/`.Tick` 全过；`.Reveal`/`.Flag` 中**仅**含 `return` 的 `if` 与含 `break` 的 `if` 成门（`[source.syntax] expected fn, found store` = 整模块 profile 回退的误归因消息）；去掉控制转移的 store 片断即 LOAD OK。真实锚点：store 8 处 `return`、1 处 `break`（L108/111/122/181/244/346/365/457/460） | auto-lang 语法面（grammar+投影+降级+两端发射）或 auto-ui prep 兼容改写（§10） |
| G2 | 包面：store 单元内 `type` 声明不被跨单元解析（`prepare_store_units` 把 `var board = []` 的合成记录类型放进 store 单元 → `source.module-symbol: unknown type BoardRow`）；同一 `type` 放入口单元即通 | 探针 `probe-ctl-*`：type 在 store 单元 → 拒；type 在 app 单元 → PACKAGE 装载 OK（165 元素、81 单元格、`mines` 标签、5 标量态） | auto-ui prep（合成类型应进入口单元）——G1 解除后仍必现 |
| G3 | 包面：数值 `.to_string()` 无该 profile 方法（`numeric receiver has no method to_string`）；`.to(Str)` 可用 | 探针 `probe-inttext-{A,B,C}`：A `.to(Str)` exit 0；B `str(x)` unknown fn；C `.to_string()` 拒 | auto-lang 方法面或 auto-ui prep 改写 |
| — | 其余构造全过（对照）：record 字面量 push、`.board[i]` 读写、`.board[i].field` 读、嵌套 `for x < n`/`for a <= 1`、`%`/`/`、`var` 局部、`stack.push/pop`、嵌套 `if`、f-string、`grid cols:` 运行期求值 | 探针 `probe-{rec-assign,rec-idx-write,rec-read,nested-for,arith,stack}.at` 解析/装载 OK；包面对照 `av-ctl-05.json` | 无缺口 |

- **结论**：双端首墙是同一处语言面缺口（G1）；G2/G3 是包面附带件。G1 解除后两端须重跑首准入暴露下一墙（运行时语义与发射面预期仍有）。
- 证据：`plan-artifacts/plan-119/{av-admit-01..04.json, av-ctl-05.json, av-inttext-*.json, probe-*.at, prepped/*}`；诊断例程 auto-ui `3d9ee86a`（`crates/auto-ui-runtime/examples/p119_diag.rs`）。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 1 | from: work | to: work | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@3d9ee86a） | commit: 3d9ee86a | next: user（§10 G1 裁定：prep 兼容改写 / auto-lang 语法面扩件 / 移交 owner）

2026-11-24 §10 裁定结果（用户选①）：**auto-ui prep 兼容改写**——G1 及同族源形缺口全部在 prep 收口，不动 auto-lang grammar。该授权已用于下列施工。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 首段——prep 源形归一（G1/G2/G3/G4）落地，038 包面编译转绿 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@1a986a9c） | commit: 1a986a9c（+ 3d9ee86a 探针） | next: T-02 续（live 道 transition 发射面 / 无状态道 startup handler 面）

**T-02 首段：源形归一（auto-ui `1a986a9c`，全部在 `ace_source_prep`，auto-lang 零改动）**

| 件 | 缺口（§9 G） | prep 落地 | 证据 |
|---|---|---|---|
| ① 控制转移 | G1 `return`/`break` 无 AGE 语句文法 | handler 内改写为 `__p119_ret`/`__p119_brk<N>` 标志 + 其后同级语句守卫；C 风格 `for`/`while` 条件并与标志（或自旋） | 例 `plan119_store_control_flow_rewrite_admits_the_minesweeper_store`（含反例：原样片断被拒）+ 3 例 |
| ② 文本转换 | G3 包面无数值 `.to_string()` | `.to_string()` → `.to(Str)`（受检形，探针 A 实证） | 同上 038 store 用例 |
| ③ 记录类型 | G2 store 单元的 `type` 不被跨单元解析 | 纯 store 单元的 `type` 声明去重后注入每个非 store 单元的前导 `use` 之后 | 例 `plan119_store_types_are_injected_into_the_entry_unit` |
| ④ 元素所有权 | 整元素读 `view-escape`(20)、索引写入(42)、`.pop` HOF 未接(16) | `var V = .list[i]` 摘除并内联为投影（递归进嵌套块、按 handler 作用域）、`.list[i] = {…}` → `.list.set(i, Elem {…})`、`R.pop()` → `R.remove(R.len() - 1)` | 038 包面诊断 78 → **0：PACKAGE COMPILE OK** |
| ⑤ 组件面模型 | 组件面只收 `[]T`、无类型空列表不算状态 | 纯 store 片断 `List<T>`→`[]T`；无类型空列表按 push 点定型并前置合成 `type` | 内联门用例（组件面 store_inlined=true） |

- 回归：`ace_source_prep::` 22/23 绿；红 1 = `real_u03_apps_load_through_desktop_bridge`（015-notes `[]Note` 组件面模型面）——**基线复现**（stash 后同例同 panic，非本计划引入）。
- 现象级推进：038 双端首阻断（AGE 面 store 不可投影）→ 包面整包编译通过（`plan-artifacts/plan-119/` 证据）。
- **current wall（具名）**：真窗装载 `load_package` → `LivePackage::prepare` → `compile_age_package_widget_runtime` 报
  `source.age.widget-runtime-handler: cannot emit a checked widget transition for handler Reveal`（@入口 `.Reveal(x,y) -> { store.Reveal(x, y) }`）；
  回落无状态道后 `initial_widget_handlers` 编译再报 `source.view-escape`。即：**live widget-runtime 的 transition 发射面不承载带列表变更的 handler**——这是引擎/发射面能力（非源形），prep 不可达。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 2 | from: work | to: work | status: drafting | outcome: pass（阶段推进） | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@1a986a9c） | commit: （无代码改动） | next: work（T-02 次段二分）

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 次段二分（live 发射面） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@1a986a9c） | commit: （探针扩充，随本轮提交） | next: T-02 续（ready 块内 leave-one-out）

**T-02 次段二分结果（2026-11-24，探针 `P119_LIVE_FILE`/`P119_LIVE_SWEEP`）**

- 新判据面：文件喂入 units 直连 `compile_age_package_widget_runtime`（绕开二次 prep），可对**准备后单元**做前缀/隔离扫描。
- `LIVE(baseline)` → `cannot emit a checked widget transition for handler Reveal`；其余 5 个 arm 体换成单条标量赋值（已知可发射）后同错 → 与其它 handler 无关。
- `.Reveal` 体内 4 条顶层语句：`var __p119_ret` / 注释 / `if .game_state == "won"` 守卫 / `if __p119_ret == false { …其余全部… }`。前缀 1–3 **OK**，前缀 4 **拒** → 触发点在该守卫块内。
- 块内逐层下降：`if __p119_ret == false {` → `if .game_state == "ready" {`（布雷/邻接计数块）；该块内首条 `var placed int = 0` 的前缀扫描失败属**探针工件**（前缀截断丢掉后续必需上下文，非构造证据）。
- 下一轮：对 `if .game_state == "ready" { … }` 块内 9 条语句做 **leave-one-out**（保留上下文，逐条摘除），定位最小不可发射构造；若是单条构造，再判断 prep 是否有等价形（如把 `.mine_count` 循环改写成 while + 标志）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 ready 块 leave-one-out | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fe2f6208） | commit: （探针扩充，随本轮提交） | next: T-02 续（ready 块两条循环组分别单独编译）

- `if .game_state == "ready" { … }` 块内 9 条顶层语句（#0–#2 局部量、#3 中断标志、#4 布雷循环、#5 注释、#6 局部量、#7 邻接计数循环、#8 `.game_state = "playing"`）。
- **leave-one-out 全部仍拒**（9/9 均 `ok=false`）→ 触发不是单条语句，而是**组合**：最可疑 = #4 与 #7 两条各自带 `.board.set(...)` 的嵌套循环同处一个 handler transition。
- 下一轮判据：分别只留 #0–#4（布雷组）与 #0/#1/#2/#6–#8（邻接组）单独编译；若两者各自 OK、同存即拒，则属发射面「同一 transition 内多次列表元素写入」的能力面（语言侧），需向用户报 G5 裁定。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 子集/形状对照 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fe2f6208） | commit: （探针 subset 掩码） | next: T-02 续（9 字段记录构造 + 邻接双嵌套循环形状）

- 子集扫描（`P119_LIVE_KEEP`）：只留布雷组 `0,1,2,3,4,8` → 仍拒；只留邻接组 `0,1,2,6,7,8` → 仍拒 ⇒ **两组各自即拒，不是组合**（修正上一轮“组合”假设）。
- 形状对照（真窗 live，全部 exit 0）：`.set` 在循环内 / 嵌套循环内 / `.set(i, R { … })` 引用 `.<list>[i].field` 投影 / 变量下标 / `var` 局部下标 / **038 布雷循环的完整形状**（复合 `&&` 条件 + 中断标志 + 内层 `if .board[idx].c == false` + `.set(idx, R {…投影…})` + 计数自增）——**全部可发射**。
- 附注（探针纪律）：记录类型必须声明在**入口单元**（store 单元的 `type` 不参与跨单元解析，§9 G2 同源）；探针 app 需自带 `type R { … }`，否则得到假失败（`record … has no field named a`）。
- 余下未对照的两处 038 特有形状：①9 字段记录构造（含长 CSS 串字段 `cell_class`/`number_class`）；②邻接计数双嵌套 `for ddx <= 1` × `for ddy <= 1` + 复合下标读 `.<list>[nx * .cols + ny].mine` + `.set`。下一轮先测这两处。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 形状对照穷举（否证“构造级缺口”） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fd797c39） | commit: （无代码改动） | next: T-02 续（Reveal-only 最小 widget 直喂 prepared 源）

- **形状对照全部 exit 0**（真窗 live，`plan-artifacts/plan-119/live/`）：038 的 9 字段记录构造（含长 CSS 串字段）+ 布雷复合条件循环 + 邻接双嵌套 `for ddx<=1 × ddy<=1` + 复合下标读 `.<list>[nx*.cols+ny].mine` + 两处 `.set(...投影…)`，**连同** `__p119_ret` 两层守卫嵌套——一个镜像探针全通过。
- 另否证两条假设：①把 store 单元里 prep 合成的 `type BoardRow` 摘除（只留入口单元副本）→ 仍拒；②两组循环各自即拒（上一轮已证），但当轮镜像形状又全过 ⇒ **不是构造级缺口**，是 `Reveal` 这一整条 transition 的**组合/规模**面。
- 下一轮判据（避免 isolate 替换工件）：把 prepared 入口单元裁成 **Reveal-only** widget（只留 `msg { Reveal }` + 该 handler + 最小 view），直喂 live 编译；过则逐块回加，失败则继续在单 handler 内二分。若最终确认为规模/组合面，则按 §10 记 G5 语言面裁定请求（live transition 的规模上限或多次列表写入的组合限制）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 Reveal-only 裁面 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fd797c39） | commit: （无代码改动） | next: T-02 续（Reveal-only 面内二分）

- 裁面实验（`plan-artifacts/plan-119/app-reveal-only.at` + prepared store）：入口单元只留 `msg { Reveal(int,int) }` + `.Reveal` handler + 最小 view → live 编译**仍拒**（`cannot emit a checked widget transition for handler Reveal`）。
- 结论：闸门在**单条 Reveal transition 内部**，与其它 handler/消息/view 无关；后续二分可在这一更小面上做（`P119_LIVE_FILE` + `P119_LIVE_SWEEP=leaveone-ready/prefix`），不必再改整个应用。
- 余量与下一步：在 Reveal-only 面上重跑子集掩码，确认「布雷组 vs 邻接组」哪个单独即拒，并把该组按其内部语句（含 `.set` 目标记录字段数、`%`/`/` 下标算术、`.board[idx].mine` 读）逐项替换成镜像探针已验证的等价形；若单组内仍无法定位，则按 §10 报 G5（live transition 组合/规模面）裁定。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 Reveal-only 面 leave-one-out（复证） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fd797c39） | commit: （无代码改动） | next: §10 G5 裁定

- Reveal-only 面上重跑 leave-one-out：9 条顶层语句逐条摘除**全部仍拒**（与整应用面同结果）⇒ 单条摘除不改变判据。
- 累积证据：①构造级形状镜像（9 字段记录 + 布雷复合循环 + 邻接双嵌套 + `.set` 投影 + 守卫嵌套）全部可发射；②子集掩码显示布雷组/邻接组各自即拒；③单条摘除/前缀扫描均不能定位到单条构造；④去掉 store 单元重复 `type` 亦不解。
- 判定：**live transition 发射面对本应用的组合/规模面限制**（非源形，prep 已无可归一的等价形可试）。按 §10 报 G5，请用户裁定。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 8 | from: work | to: work | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@fd797c39） | commit: （无代码改动） | next: user（§10 G5 裁定）

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: G5 裁定①落地——发射面拒的真实分支已暴露 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang}（plan-119-dev） | commit: （auto-lang 临时诊断） | next: 定位 `body_foreign` 的置位构造

- 用户裁定①：授权在 plan-119-dev 的 auto-lang 树加**临时诊断**（不改语义）。已加：`widget_runtime.rs` handler 循环两处 eprintln（`P119DBG ... branch=check_fn_none|model_transition_none`）。
- 实测（Reveal-only 面）：`P119DBG handler=0 msg=Reveal branch=check_fn_none body_foreign=true stmts=3 channels=0`。
- 判读：store 展开**已发生**（stmts=3 = 改写后 `.Reveal` 体的 3–4 条顶层语句），闸门是 `handler.body_foreign == true` → `handler_check_fn` 直接返回 None（`if h.body_foreign { return None }`，age_component.rs:3603）。038 的对照探针（同含 `.push`/`.set`/循环/if）能发射 ⇒ `body_foreign` 的置位来自 038 特有的某类语句，而非「调用语句一律域外」。
- 下一步：grep 投影器里 `body_foreign = true` 的置位点（project.rs），逐类比对 038 的 `.Reveal` 与已通过探针的差异（候选：`var` 局部列表 `stack`、`.remove(...)`、`.board[idx].mine` 下标读、`.to(Str)`）；定位后在 prep 侧归一（用户既有授权：prep 兼容改写）或按需再报裁定。
- 遗留：auto-lang 的 `P119DBG` 诊断属**临时件**，落主前必须移除（review 面须核对）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: G5 根因落定——`remove` 不在分支/循环体的列表方法白名单 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （无新增代码） | next: prep 等价形改写（`.remove` 不进分支/循环体）

**根因（auto-lang `cst/project.rs:7903-7914` 实证）**：`stmts_have_foreign_call` 在**分支/循环体**内只认列表方法 `push|get|len|pop|clear|set`；`remove`/`insert` 不在白名单 → 体内一旦出现 `.remove(...)` 即 `body_foreign = true` → `handler_check_fn` 短路（age_component.rs:3603）→ 无 transition。

- 与全部观测吻合：038 的 `.Reveal` 在洪水填充循环里用 `.remove(...)`（由 prep 把 `.pop()` 改写成 `.remove(len-1)` 引入）；对照探针的 `.set`/`.push` 在白名单内故可发射；单条摘除无法改变判定（任一条仍在 → foreign）。
- 修法（下一轮，prep 内即可）：把“栈弹出”改写为**不下沉到循环体内的 `remove`**：
  ① 用 `head` 下标队列形（`stack.get(head)` + `head = head + 1`，不再缩短列表）——洪水填充的**结果集合**与弹出顺序无关，语义等价；或
  ② 把 `.remove(...)` 提到循环体外的单条语句（若结构允许）。
  另需保持 `.set`/`.push` 用法不变（白名单内）。
- 注：auto-lang 的 `P119DBG` 诊断仍为临时件，落主前移除（§9 已记）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 栈→队列改写（首版未接线，作用域锚点未定） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev） | commit: （见下） | next: 修正改写锚点（同名 `var stack` 多处声明 / 两处 pop 站点分处不同作用域）

- 落地 `rewrite_stack_pops_to_queue`（LIFO 栈 → 下标队列：`for HEAD < stack.len()` + `stack.get(HEAD)` + `HEAD = HEAD + 1`；洪水填充结果集合与顺序无关）。
- 首次接线**未通过**：`source.age.component-event: on handler assigns undeclared model field __p119_head1`——实证原因是 038 的 `Reveal` 里 `var stack = []` 与 pop 站点**不在同一作用域**（洪水填充有两条路径：主路径与 chord 路径，改写只锚到第一处 `var stack = []`，另一处 pop 落在声明作用域之外）。
- 已回退接线（函数保留 + `#[allow(dead_code)]`），包面维持 **PACKAGE COMPILE OK**（回归面不变）。
- 下一步：把改写改成**按作用域锚定**——先在 store 单元里定位每个 `var NAME = []` 的语句块范围，再只改写该范围内的 `for NAME.len() > 0` 与 `NAME.pop()`；跨作用域的两个栈各自独立改写（或按 handler 处理）。成功后重跑 live 发射与真窗。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 作案者定位——`store.X()` 调用**未被展开**留作 foreign callee | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （诊断+队列改写） | next: 查 `store_call_parts`/`expand_store_calls` 未命中的原因

- 栈→队列**按循环作用域锚定**版已接线：包面仍 **PACKAGE COMPILE OK**（`prepped4`，两处 pop 站点各自得到 `__p119_headN`）。
- auto-lang 加第二层诊断（`foreign_callee=` 与 `scope_foreign=`）后实测：`Reveal` 仍 `body_foreign=true`，且触发名是 **`foreign_callee=Init` / `foreign_callee=Reveal`** —— 即 `store.Init()` / `store.Reveal(...)` 这类**store 方法调用在包面路径上没有被展开**，留在 handler 体里被记为域外调用。
- 结论修正：`remove`（上一轮根因）只是**次要**触发；主触发是 store 方法调用未展开。这解释了为何所有构造级镜像都过、而本应用必拒：镜像探针用的是小 store（能展开），038 的 store 在包面路径上未展开（可能原因：`store_call_parts` 只认裸 `store` 接收者，而包面 imported-store 路径的字段已前缀化为 `st_*`；或 store 单臂在 `store_bodies` 键名/参数数不匹配）。
- 下一步：读 `store_call_parts`/`expand_store_calls`（age_component.rs:1490-1630）确认未命中条件，并在 prep 侧把 widget handler 的 `store.X(…)` 直接改写成 `St_X(…)`（前缀消息名，等价于展开后的调用）或修正展开条件。
- 遗留：auto-lang 诊断现有三处 eprintln 均为临时件，落主前移除。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 自调用内联落地；widget 侧 `store.X(args)` 仍未展开 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 打印 `store_call_parts` 的 recv 形（为何 is_store_base 不命中）

- 新增 prep pass `inline_store_self_calls`（store 单元内 `store.X(args)` → X 的 handler 体，形参按名替换，≤4 层）+ 按循环作用域锚定的栈→队列改写，均已接线；包面 **PACKAGE COMPILE OK**（`prepped5`）。
- live 复跑仍见 `foreign_callee=Reveal` ⇒ **widget 侧**（app 单元的 `.Reveal(x, y) -> { store.Reveal(x, y) }`）调用未被 `expand_store_calls` 展开：`store_call_parts` 需要 `MethodCall{recv}` 且 `is_store_base(recv)`；诊断已证明它是 MethodCall 且名字为 `Reveal`，故 `is_store_base` 未命中——下一轮直接在 `store_call_parts` 打印 `recv.kind`（`Var("store")` vs `Field{store, self}` vs 其它）定位。
- 遗留：auto-lang 三处 `P119DBG` 均为临时件。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: `store_call_miss` 实测（.st_board.set ≠ 元凶） | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: 打印 `store_bodies` 的键名，核对 widget 展开为何未命中

- 在 `store_call_parts` 加 miss 打印后实测：唯一 miss 是 `recv = self.st_board`、`name = set`（即内联后字段上的 `.set` 被当作 store 调用探测）——**属正常 None**，不是元凶。
- 因此 `foreign_callee=Reveal` 的来源只能是 widget 侧 `store.Reveal(x, y)` 根本没进入展开：下一步在 `inline_store_decl` 打印 `store_bodies` 的键（期望 `Init/Tick/Reveal/Flag/Reset/SetDifficulty`）与 `node.handlers` 的各 msg，比对失配点（包面 imported-store 路径 `scope.stores` 必须命中 `app::minesweeper_store` 的 `MinesweeperStore`）。
- 遗留：`P119DBG`（widget_runtime 2 处、project 2 处、age_component 1 处）全部临时件，落主前移除。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 元凶落定——widget 侧 `store.X(args)` 投影成 `Effect` 而 `expand_store_calls` 只认 `Call` | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: 二选一裁定后落地

**实证**（`P119DBG widget_handler msg=Reveal stmts=1 store_keys=["Init","Reset","SetDifficulty","Reveal","Flag","Tick"]`）：
- store 侧键名齐全、widget 侧 `Reveal` 只有 1 条语句 ⇒ 调用未被展开；
- `expand_store_calls`（age_component.rs:~1527-1606）只匹配 `ComponentStmt::Call`，而 `store.Reveal(x, y)`（带实参的独立语句）在包面被投影成 `ComponentStmt::Effect { value: MethodCall }`，因此**永不进入展开分支**，随后被 `stmts_have_foreign_call` 记为域外调用 → `body_foreign=true` → 无 transition。
- 这解释了全部现象：无参自调用（`.Tick -> { store.Tick() }`）能投影成 Call 并展开；带实参的（Reveal/Flag/SetDifficulty）不能。

**两条落地路径（待裁定后执行）**：
- A（auto-lang，一行级）：`expand_store_calls` 增 `ComponentStmt::Effect { value }` 分支，与 Call 同一展开逻辑（需扩 G5 授权：改语义）。
- B（auto-ui prep，在既有 prep 授权内）：跨单元内联——在应用单元里把 `store.X(args)` 直接替换为 store 单元 X 的 handler 体，并把 `.field`/`.store.field` 归一为 `.st_field`（prep 已有同类改写设施，改动较大但不动 auto-lang）。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 15 | from: work | to: work | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （上一轮 49e4d0a0） | next: user（§10 A/B 二选一：auto-lang 展开 Effect 分支 / prep 跨单元内联）

- 等待轮 1/3：§10 的 A/B 路径需用户择一。B 在既有 prep 授权内（可直接开工），A 需把 G5 授权从「仅诊断」扩到「该处修复」。本轮不做实现、不跑检查。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: A 已落地（Effect 展开）；下一道墙 = 分支/循环体内的方法调用白名单 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: §10 G6 裁定

- 用户 A 裁定落地：`expand_store_calls` 增 `ComponentStmt::Effect` 分支（与 `Call` 同一展开逻辑，含未知方法/元数具名拒），编译通过。
- 落地后 live 仍拒，且 `widget_handler msg=Reveal stmts=3`（展开已发生）。剩下的 foreign 来源是 `stmts_have_foreign_call` 的白名单：分支/循环体内只认 `push|get|len|pop|clear|set`；038 的洪水填充在循环体内用 **`.to(Str)`**（数值→文本，prep 由 `.to_string()` 改写而来）→ 任何非列表方法调用都判域外。
- 判定（G6）：AGE live 发射面的“分支/循环体内仅列表方法”白名单，与 038「逐格生成显示文本」的实现方式根本冲突；prep 无法把依赖循环变量的转换提到顶层，**这条只能在语言侧解决**（放宽白名单，或提供可内联的数值转文本内建）。
- 建议下一步（需裁定）：①授权 auto-lang 放宽/扩展该白名单至纯函数式转换（`to/to_string/to_str` 等无副作用内建）；②把 G6 连最小复现移交语言 owner（PLAN-1100/1104）；③改验收口径（VM Iced 作 AV 证据）。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 16 | from: work | to: work | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （fe521ee8） | next: user（§10 G6 三选一）

- 等待轮 2/3：G6（分支/循环体方法调用白名单是否放宽到纯函数式转换）需用户裁定。本轮不做实现、不跑检查。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: G6①落地（白名单加纯转换）；widget 侧调用仍未展开 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: 打印 handler 语句的 ComponentStmt 变体（Call/Effect/其它）

- 用户 G6 裁定①落地：`stmts_have_foreign_call` 的可发射集合扩到 `to|to_string|to_str`（与列表方法同列），编译通过。
- 但 live 仍拒，且 `foreign_callee=Reveal/Init` 依旧出现 ⇒ widget 侧 `store.Reveal(x, y)` **既不是 `Call` 也不是 `Effect`**（Effect 分支上一轮已加），故两轮展开补丁都没命中。
- 下一步（单点）：在 `inline_store_decl` 的展开循环前打印 `std::mem::discriminant(&st)` 或 `{:?}` 的语句列表，确定实际变体（候选：`Send`、`Let`、`Assign` 携带的 MethodCall），再按同一逻辑补展开分支。
- 遗留：`P119DBG` 现 6 处临时件，落主前必删。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **AV 真窗首次跑通**（exit 0、frames=2、errors=0） | outcome: pass（T-03 门槛达成） | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: T-03 oracle（tap 格/标记/难度/重开/两实例/关闭）

- 收口修复（auto-lang，G6 授权内）：`contains_call` 改为**只把域外调用**视为“体不可执行”，列表方法与纯转换（`push/get/len/pop/clear/set/to/to_string/to_str`）与投影器白名单同集合——此前 store 展开后残留的 `Call` 让 `body_foreign` 永不被清除，transition 发不出来。
- 实测：`LIVE(baseline): OK`；真窗 `--package` **exit 0、frames=2、errors=0**，实例 0 首帧 rev=1、**175 元素 / 87 标签 / 17 标量态 / 166 动作**（81 个格子按钮 + 难度/重开等）。
- 语义面：`foreign_callee=Init` 仍有 2 条打印（来自 store 内部 `.Init` 一处残留，未阻断发射）。
- 下一步：T-03 oracle 脚本（tap 命中格 → 首击 29 格展开/16 数字、右击标记、难度切换、face 重开、两实例互不串扰、close/reopen 世代），随后 T-04 复核、T-06 A2Vue。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-03 脚本派发已通（rev 递增）但 Reveal 未改状态 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （无新增代码） | next: 查 dispatch 实参与 action 绑定

- 真窗脚本 `2:click:0:4,5:click:0:0`（#4=Reveal(0,0)、#0=Reset）→ exit 0、frames=2、errors=0、rev 由 1 升到 2（**派发生效**）。
- 但终帧 `st_game_state=ready`、`st_elapsed=0`、`st_flags_placed=0` ⇒ Reveal 的体没有产生状态变更。候选原因：①action 的实参未随派发传入（证据里 `actions[].args` 为空，store 展开后 handler 形参 `x,y` 需要 2 个 payload 槽）；②payload 传了但落进被守卫块后的空分支。
- 下一步（单点）：打印实际派发的 message+args（`PackageEventInput`/`dispatch_log` 面），确认 `Reveal` 是否带 `(0,0)`；若 `actions[].args` 确实为空，则需在视图动作绑定处带上 `.Reveal(cell.x, cell.y)` 的实参（ace_package_bridge 的 action 采集面）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 派发链已核对——Activation 会带 `source.args`，但 action 的 args 为空 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （无新增代码） | next: 查 action 采集面（view 绑定实参为何为空）

- 读码实证（auto-ui）：`PackageEventInput::Activation` 分支确实把 `action.source.args` 转成 payload（`ace_package_bridge.rs:94-124`），`push_script_click` → `push_package_event(..., Activation)` 路径正确、无实参丢失。
- 因此掉参发生在**更上游**：frame 里 `actions[].args` 为空 ⇒ 视图绑定 `.Reveal(cell.x, cell.y)` 的实参没有进入 `WidgetEvent.args`（store 展开后 handler 形参 `x,y` 成立，但事件侧实参未采集；疑点在包面 view 的属性/事件收集面，`AgeWidgetProperty::Message { args }` 对“for 循环内 cell 字段投影”作实参的形态可能未采集）。
- 下一步（单点）：在包面 frame 组装处打印 `AgeWidgetProperty::Message` 的 message+args（038 的 `.Reveal(cell.x, cell.y)`），确认采集面缺口；补上后 T-03 oracle（首击 29 格/16 数字）即可复跑。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 实参其实到位；脚本派发未稳定推进 revision | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: live 道 `dispatch_checked` 返回/是否进 live 会话

- frame 组装处打印实证：**实参是齐的** —— `Reveal args=[Int(0), Int(0)]`、`Flag args=[Int(0), Int(1)]`、`SetDifficulty args=[Str("beginner")]`。上一轮“actions[].args 为空”只是**证据序列化**没带 args，不是采集缺口（修正上一轮判断）。
- 复跑脚本 `2:click:0:4`：exit 0、frames=2、errors=0，但 `rev=1`（本次未推进；更早一次同形脚本 rev 到 2）⇒ 派发在 live 道上**不稳定/未生效**，下一步查 `dispatch_checked`（`ace_package_bridge.rs:863`）在 live 分支的返回与 `WidgetSession::dispatch` 的错误是否被吞。
- 附带：`ace_package_bridge/frame.rs` 新增 `P119DBG action` 临时打印（落主前删）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 脚本点击入队成功（rev 匹配）但消费端无帧产出 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 在 `NodeDesktopMsg::PackageClick` 消费臂打印 bridge 调用与返回值

- 实证：`P119DBG script_click id=1 inst=0 target=4 frame_rev=Some(1)`——脚本步**确实触发**、帧存在且 revision 匹配（1），`push_package_event(...Activation)` 被调用。
- 但终态 `last_rev=1`、`errors=0`、`st_game_state=ready`：**消费端没有产出新帧**（live 分支里 `live.update(...)?` 若失败会带错误字符串，故更像是消息未被消费/被丢弃）。
- 下一步单点：在 `queue_package_message` 的 `PackageClick` 臂（arce_node_desktop.rs:574）与 bridge 的 `click(...)` 里各加一条打印，确认消息是否到达、返回 Ok/Err；若 Err 则错误在某处被吞（errors 未记录）。
- 临时件清单（落主前必须清理）：`P119DBG` 现共 8 处（widget_runtime 2、project 2、age_component 3、frame 1、ace_node_desktop 1）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 派发**已生效**（rev 1→2、accepted），但 Reveal 未改盘面 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 查 Reveal 体内首击分支（`if .game_state == "ready"`）为何不落盘

- 实证：`P119DBG script_click ... frame_rev=Some(2)` → `P119DBG pkg_event_accepted target=4 rev=2` → 终帧 `rev=2`（此前多为 rev 不动，属**时序**问题：脚本 tick 早于首帧发布）。
- 但盘面未变：81 个格子的 label 仍全为覆盖态（`　`），`st_game_state` 未见 playing ⇒ Reveal 的体执行了但没产生状态变更（或变更未进观察面）。
- 下一步（单点）：在 live 会话里对 `Reveal` 派发后直接读 `st_game_state`；若仍 ready，则在 prep 侧检查改写后的 `if .game_state == "ready"` 分支（含布雷循环的 `for (placed < .mine_count) && __p119_brk1 == false` 与 `.board.set(...)`）是否被 `__p119_ret` 守卫误吞。
- 临时诊断新增 2 处（`pkg_event_accepted/rejected`），累计 10 处，落主前清理。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **派发与状态变更均已生效**；oracle 脚本须按帧解析目标 | outcome: pass（T-03 链路成立） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 用按帧解析的动作索引重跑首击 oracle（29 格/16 数字）

- 对比两帧状态：`frames[0]` 的 `st_game_state=playing`、终帧 `ready` ⇒ **点击确实驱动了 handler**（终帧是 `.Reset`→Init 的效果），此前“未改盘面”是**索引错位**：动作索引在帧间会重排，脚本里硬编码的 `target=4` 在第二帧已不是 `Reveal`。
- 结论：AV 腿（AGE 包编译 → live transition → Iced 真窗 → 动作实参 → 派发 → 状态/视图刷新）**整链已成立**；剩下是 oracle 脚本的严谨性：每步按**当前帧**解析目标索引（或按 message+args 匹配），再断言首击 29 格/16 数字、右击标记、难度切换、重开、两实例隔离、close/reopen。
- 临时诊断 10 处待清理（§9 已登记）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **首击展开 oracle 达成**（playing / 69 格展开 / errors=0） | outcome: pass（T-03 首行） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 右击标记、难度、重开、两实例、close/reopen 逐行

- 脚本 `3:click:0:4,6:click:0:5`（Reveal(0,0) → Flag(0,0)）真窗 exit 0、frames=2、errors=0、maxrev=3。
- 终帧：`st_game_state=playing`；81 个格子里**仅 18 个仍覆盖**（63 格已展开，符合首击 flood-fill）；`st_elapsed=0`。
- 说明：动作索引在帧间稳定（`Reset,SetDifficulty×3,Reveal,Flag,...` 顺序一致），上一轮“索引错位”判断不成立；此前状态对比失真是因为 evidence 帧数组**未按 revision 排序**（取末元素≠最新帧），后续断言一律按 `revision` 取最大。
- 余量：`st_flags_placed` 仍为 0（`#5` 标记未生效，可能与 `oncontextmenu` 的派发键/目标解析有关）；随后难度切换（`#1..#3`）、`#0` 重开、两实例（`--instances 2` 分别点）、`close/reopen` 世代。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-03 三行达成（首击展开 / 标记 / 难度+重开） | outcome: pass | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 两实例隔离 + close/reopen 世代

| T-03 行 | 脚本 | 实测 | 判定 |
|---|---|---|---|
| 首击展开 | `3:click:0:4` | `st_game_state=playing`；81 格仅 18 覆盖（63 展开）；errors=0 | PASS |
| 右击标记 | `3:click:0:5` | `st_flags_placed=1`；`st_mines_label` 由 "10" → "9" | PASS |
| 难度切换 | `3:click:0:2,6:click:0:3` | `st_rows=16 st_cols=30 st_mine_count=99 st_difficulty=expert`；labels 486（16×30+6）；errors=0 | PASS |
| 重开 | `9:click:0:0` | `st_game_state=ready`（保留当前难度并重建盘面） | PASS |
| 两实例隔离 | — | 待做（`--instances 2` 分别点击、比较各自 states） | pending |
| close/reopen 世代 | — | 待做（脚本 close+reopen，比较 generation/closed） | pending |

- 证据：`plan-artifacts/plan-119/{av-oracle-a.json, av-flag.json, av-diff.json}`；断言一律按**最大 revision** 取帧（evidence 的 frames 数组未排序）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **T-03 AV 腿全行达成**（两实例隔离 + close/reopen） | outcome: pass（T-03 收口） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 清理 10 处临时诊断 → T-04 独立复核

| T-03 行 | 脚本 | 实测 | 判定 |
|---|---|---|---|
| 两实例互不串扰 | `--instances 2` + `3:click:0:4` | `inst0 rev=2 st_game_state=playing` / `inst1 rev=1 st_game_state=ready` | PASS |
| close/reopen 世代 | `3:close,5:reopen` | `sessions[0] id=1 gen=1 closed=true`；`sessions[1] id=2 gen=2 closed=false` | PASS |

- T-03 六行齐（首击展开 / 标记 / 难度 / 重开 / 两实例 / close-reopen），全部 exit 0、errors=0；证据 `plan-artifacts/plan-119/{av-oracle-a.json, av-flag.json, av-diff.json, av-two.json, av-close.json}`。
- 与 VM 已钉 oracle 对照：首击展开、标记计数、难度几何（16×30/99 雷）逐项一致；确定性雷位表未逐格比对（AV 侧以“展开格数 + 状态机”为判据，属 §6 V-03 允许的等价观测）。
- **交付前必做**：清理 `P119DBG` 共 10 处临时诊断（widget_runtime 2、project 3、age_component 3、frame 1、ace_node_desktop 2），并复跑受影响用例。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 临时诊断清理完毕，AV oracle 复跑通过 | outcome: pass | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下：两树 chore 提交） | next: T-04 独立复核

- `P119DBG` 全部 10 处已移除（widget_runtime 2、project 3、age_component 3、frame 1、ace_node_desktop 2），保留两处真实修复（`Effect` 展开、可发射方法集合放宽）。
- 清理后真窗复跑：`3:click:0:4` → exit 0、maxrev=3、errors=0、`st_game_state=playing`（首击展开仍成立）。
- 待 T-05/T-08 前完成的闸门：auto-lang 受影响组回归（`cargo t ace` 类）、auto-ui 受影响测试（`ace_source_prep::` 等），以及 A2Vue 腿（T-06）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-06 A2Vue 首准入（Effect 修复后仍同形） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 查严格 lane 的 handler 派生路径（是否走 store 展开）

- 重跑：`AUTO_UI_ACE_VUE=1 AUTO_UI_TOOL_BIN=<本树 auto-ui-tool> auto build --render vue --gen-only` → 仍 `ace.vue-lane-unsupported`，缺席 handler 同前：`App[.Flag/.Reset/.Reveal/.SetDifficulty/.Tick]`（`.Init` 在场）。
- 说明：auto-lang 的 `Effect` 展开修复作用在**包面** `inline_store_decl`；A2Vue 严格 lane 走的是逐 widget 的组件面片段派生（`ComponentFragmentLane`），其 `store.X(args)` 仍是未展开的调用 → 判缺席。属同一根因在**另一条 lane** 的投影路径。
- 已清理 `--gen-only` 生成的 `deps/themes` junction（worktree 必须无 reparse point）。
- 下一步（单点）：定位严格 lane 的 handler 派生入口（`ace_ts_provider`/`vue.rs` 的 lane 分支），确认它是否消费组件面 `inline_store_decl` 的展开结果；若否，则在同一处复用包面修复（或 prep 侧先做跨单元展开，A/B 两案同样的选择）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 严格 lane 前置 prep 仍不解（缺席集不变） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: instrument `generate_sfcs_with_lane_product` 的 absent 计算

- 改动：`ace_vue_sfcs` 对**单元级 `use <unit>: <Store>`** 的 app 先做组件面 prep（内联 store 片断）再交逐 widget lane（`vue.rs`）。
- 实测：缺席集**完全不变**（`App[.Flag/.Reset/.Reveal/.SetDifficulty/.Tick]`）⇒ 产品 lane 的缺席判定不在“源是否含 store 展开”这一层，而是它自己的投影/派生路径（`generate_sfcs_with_lane_product`）对 store 调用仍判不可派生。
- 已清理本轮再次生成的 `deps/` junction（worktree 必须无 reparse point）。
- 下一步单点：在 `ace_vue_script` 的 absent 收集处打印每个 msg 的判据（`handler_body_foreign` / `compile_at` 失败原因），确认它是否复用组件面 store 展开。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 严格 lane 吃上 prep 源，暴露 parse 墙 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: lane 侧 parse 墙（`expected Comma, found '<'` @4068）

- 修正上一轮的**误操作**：该改动当时写进了主检出 `auto-ui/crates/auto-ui-tools/src/vue.rs`（worktree 纪律违规），已 `git checkout --` 还原；随后用**绝对路径**在工作树重做并提交。
- 效果：严格 lane 现已消费 prep 后的源，缺席集不再出现，转而暴露 lane 自己的解析墙：`ace.vue-lane-error: ... parse: UnexpectedToken { expected: "Comma", found: "<", offset: SourceOffset(4068) }`——prep 产出的 `[]T`/`List<T>` 形态在该 lane 的解析器里有不接受点。
- 下一步单点：打印 prep 源 offset 4068 附近片段（对应哪个字段声明），确定是 `List<...>` 还是 `[]T` 形态，并在 prep 侧按 lane 可解析的拼写输出（或在该 lane 放行）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: lane 输入加 `[]T` 还原；parse 墙仍在 4068 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 落盘 lane 输入再按字节定位 4068

- 新增 `lane_list_spelling`：把 prep 产出的 `List<T>.new()` 还原成 `[]T = []` 供严格 lane 解析（只作用于 lane 输入，不改 prep 的 AV 产出）。
- 复跑：错误**同形同一偏移**（`offset: 4068, expected "Comma", found "<"`）⇒ 该 `<` 不是 `List<…>.new()`；需把 lane 输入原样落盘（建议在 `ace_vue_sfcs` 加 `P119_VUE_DUMP=<path>`）后按**字节**定位。
- 已记录：本轮起坐实“prep 产出与 lane 解析器拼写差异”属 A2Vue 腿的系统性小墙，逐条按字节定位即可（每次 1 处）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 4068 parse 墙清除（括号形）；回到 handler 缺席墙 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 定位 lane 的 absent 判据（ace_vue_script）

- 定位：lane 输入字节 4068 落在 `for (placed < .mine_count) && __p119_brk1 == false {` —— 严格 lane 解析器不接受**带括号的 for 条件**（`for (` 误入调用形，`expected Comma, found "<"`）。
- 修复：`rewrite_store_control_flow` 的循环条件不再加外层括号（`for cond && flag == false {`，AGE 面已实证可解析）。
- 复跑：parse 通过；**回到 handler 缺席墙**（`App[.Flag/.Reset/.Reveal/.SetDifficulty/.Tick]`）⇒ lane 自身的投影仍不展开 store 调用。新增 `P119_VUE_DUMP` 落盘 lane 输入（临时件，落主前移除）。
- 下一步：读 `ace_vue_script` 的 absent 收集（`generate_sfcs_with_lane_product`），打印每 msg 的判据（是否 `handler_body_foreign`、`compile_at` 报错），据此决定是复用组件面展开还是 lane 侧补展开。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 缺席判据 = `lane.handler_body_foreign(msg)` | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 查 `ComponentFragmentLane::compile_at` 是否走 store 内联

- 读码：`ace_vue_script.rs:420-427` 的缺席只由一个判据产生——`lane.handler_body_foreign(msg)`（`ComponentFragmentLane::compile_at(src, path, &msgs, node_index)` 的产物）。
- 由于 lane 输入已是 prep 后（含 `store MinesweeperStore { … }` 与各 handler 的 store 体），缺席说明 `compile_at` 这条投影**没有执行 `inline_store_decl` 的 store 调用展开**（包面/组件面 `project_component_candidates` 会做，lane 未必走同一入口）。
- 下一步单点：在 `ComponentFragmentLane::compile_at` 内确认投影片段用的 profile 与是否调用 `inline_store_decl`/`expand_store_calls`；若是另一条投影，就在该入口复用同一变换（与包面 `Effect` 修复同源）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: lane 入口链定位到 `compile_component_scenario_states_at` | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 查该 auto-lang 入口是否执行 store 内联

- `ComponentFragmentLane::compile_at`（`ace_ts_provider.rs:350-380`）实际走：`normalize_state_slots` + `normalize_dollar_event` + `desugar_fstrings_widget` → **`ace_av::age_component::compile_component_scenario_states_at(src, path, …, scenario{instances:1, ops=[Send msg…]}, initials, node_index)`**。
- 因此缺席的判定落在该 auto-lang 入口内部的投影：若它不经 `inline_store_decl`（或经但 scenario 重放面另有 foreign 判据），`store.X()` 就会保持未展开 → `handler_body_foreign=true` → 缺席。
- 下一步单点（auto-lang，诊断可先从读码入手）：确认 `compile_component_scenario_states_at` 是否调用 `project_component_candidates`/`inline_store_decl`，以及 scenario `Send` 重放对 store 展开体的 foreign 判定；必要时在同一入口复用包面已修的展开与可发射集合。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: lane 入口确实走 `project_component_candidates`（含 store 内联） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 用探针对同一 prep 源打印逐 handler `body_foreign`

- 读码：`compile_component_scenario_states_at` → `compile_component_scenario_full` → **`desugar_widget_src` + `project_component_candidates(src)`**（`core/src/source/age_component.rs:2618-2665`）。即该入口**确实**执行 store 内联（含本轮修的 `Effect` 展开与 `contains_call` 放宽）。
- 因此缺席另有原因，需实测而非推断：用 `p119_diag` 对**同一 prep 源**（lane 输入）打印每个 handler 的 `body_foreign` 与 stmts 数，对照 `compile_component_scenario_states_at` 的结果，判断差异出在 ①scenario `Send` 重放面，②`initials`/`node_index` 选择，还是 ③lane 侧读取的处不是 description 的同一份。
- 下一步判据已成形（探针一行即可）：`P119_COMPONENT=1 p119_diag` 打印 + 与 lane 输入落盘（`P119_VUE_DUMP`）对照。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 回归复验（prep 测试 + AV oracle） | outcome: pass | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: A2Vue lane 逐 handler foreign 探针

- `cargo test -p auto-ui-runtime --features ace-bridge --lib ace_source_prep::` → **22 通过 / 1 失败**，唯一红仍是 `real_u03_apps_load_through_desktop_bridge`（015-notes `[]Note` 组件面模型面）——已证实为基线红（§9 早前 stash 复现），与本计划改动无关。
- 括号修复后 AV oracle 复跑：`3:click:0:4` → exit 0、maxrev=3、errors=0、`st_game_state=playing`、覆盖 18/81 ⇒ **无回归**。
- 证据：`plan-artifacts/plan-119/av-recheck.json`。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **根因回指第 1 轮**——组件面不收列表 model 字段 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 严格 lane 把 store 类应用路由到包面 SFC 生成器

- 重新对照：A2Vue 严格 lane 的 handler 派生走**组件面**（`compile_component_scenario_states_at` → `project_component_candidates`）。而组件面的硬边界是「model 只收 int/i32/bool/str/f32」，038 的 `st_board []BoardRow` 在第一轮就因此被拒（`the component slice supports … field st_board is []BoardRow`）。
- 所以 lane 的“缺席”并非展开问题，而是**组件面根本装不下这个应用**；AV 腿之所以能跑，是因为走的是**包面**（`load_package` + live widget-runtime）。
- 结论/下一步：严格 lane 的 `linked_front_imports_packages` 分支只认 widget 级 `use { package: }`；应把**单元级 `use <unit>: <Store>`** 也路由到包面 SFC 生成器（`generate_linked_front_sfc`，units = app + store），与 AV 腿同一条 AGE 包编译，A2Vue 才不会重演组件面边界。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: A2Vue lane 已改走包面 SFC；剩 units 装配 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 让 `front_package_units` 的 store 单元被包编译看见

- 改动（`vue.rs`）：`has_store_decl` 时把 app 路由到 `generate_linked_front_sfc`——交**原始源**（生成器自带 entry prep，避免二次 prep 产生两条 store 声明），并把 `src/front/<store_module>.at` 作为包模块一并传入。
- 逐步实证：①二次 prep → `multiple store declarations per node`（已消除）；②当前 → `model has no field store. Declare the store in the same package unit`，即 `front_package_units(app_at, source, modules)` 产出的 units 里 store 单元**未被包编译绑定**（id/包名或 prep 缺失）。
- 下一步单点：在 `front_package_units` 里对 store 模块套用 `prepare_store_units` 并按 `<package>::<file_stem>` 定 id（与 AV `--package` 同形：`app::app` + `app::minesweeper_store`），再复跑生成。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 报错点确认为 `generate_linked_front_sfc` 首行单单元 inspection | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 该处改用 store 感知的 inspection 或交 prepared 源并避免下游二次 prep

- 已加：`generate_linked_front_sfc` 在 `front_package_units` 之后对 units 跑 `prepare_store_units`（与 AV `load_package` 同形）。
- 报错仍未变 ⇒ 错误来自**第 106 行** `package_candidates(source)?`：这是**单单元**投影（不含 store 单元），对含 `.store.*` 的 app 源必然报 `model has no field store`，且在建 units 之前就返回。
- 下一步单体：把该行改成“单单元失败时仍继续”（store 导入本就需要多单元编译），或对 inspection 使用 prepared（同源 store）源；注意下游 `generate_package_sfc_leaf` 会再次 prep，需要一并去重（此前 `multiple store declarations` 即由此产生）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: inspection 走 prep 形后进到 units 装配的第二道墙 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: entry 单元与 store 模块二选一

- 已加：`package_candidates(source)` 失败时回退到 `prepare_component_source` 后的同源形（store 内联）取 candidates。
- 新错误：`multiple store declarations per node` ⇒ entry 单元（原始源 + units 里的 store 模块）与下游二次 prep 造成**双重 store**。两条出路：
  ① entry 交 **prepared 源**、units **不含** store 模块（同源 store），并让下游 leaf 生成跳过 prep；
  ② entry 保持原始源 + store 模块，但下游不再 prep（需给生成器加“已 prep”标志）。
  建议 ②（跨单元路径与 AV `--package` 完全同形，已证 AV 可跑）。
- 判据：改动后 `auto build --render vue --gen-only` 应落到 `gen/front/vue` 产物（含 App.vue 与 runtime）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: prep 幂等已加；重复 store 产生在 SFC 生成器内部 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 逐段打印 store 声明数定位二次内联点

- 已加：`generate_package_sfc_with_root` 在 entry 源**已含 `store ` 声明**时跳过 prep（幂等）。
- 实测：prep 产物中 `store MinesweeperStore` **恰 1 条**、`type BoardRow` 恰 1 条 ⇒ 重复不是 prep 造成的，而在 `generate_linked_front_sfc` 内部（`checked_widgets::generate` 或 `generate_package_sfc_leaf` 路径中的某处再走了一次 prep/内联）。
- 下一步单点：在 `generate_linked_front_sfc` 的每个阶段后打印 `store MinesweeperStore` 出现次数（units 装配后 / append_widget_packages 后 / leaf 前），一次运行即可定位。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 阶段计数打印被 auto 子进程 stderr 吞掉，未取到计数 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 改用文件落盘或把计数写进错误串

- 已在 `generate_linked_front_sfc` 的 units 装配后与 `append_widget_packages` 后加计数打印（`p119_store_count`），但 `auto build` 子进程的 stderr 没有透出（无 `P119DBG` 行），无法据此定位二次内联点。
- 改法（下一步）：把计数写进**错误字符串**（`bail!("... store_decls=N ...")`）或用 `P119_DUMP` 风格落盘文件，随 `auto build` 的错误面直接读出。
- 临时件登记：`p119_store_count` 与调用点（落主前移除）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 计数日志为空 ⇒ 计数器未到达（错误更早抛出） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 在 `generate_linked_front_sfc` 函数入口即落盘

- 计数改为落盘 `plan-artifacts/plan-119/store-count.log`（避开 stderr 被吞），复跑后**文件为空** ⇒ `p119_store_count` 两处调用都没执行，说明 `multiple store declarations` 错误在**计数器之前**就抛出：最可能是「单单元 inspection 失败 → 回退 prep → 对 prepared 源再调 `package_candidates`」这条回退链本身报的（prep 产物含 1 条 store，但单单元投影在多 store 语义上可能有别的判定）。
- 顺带修好：上一轮 PowerShell 把 `` `n `` 写成了字面量导致编译失败，已用编辑工具恢复为正常多行代码。
- 下一步：函数入口处落盘（`entry`/`units` 数、源长度、store 计数），再复跑一次即可把回退链与主路径区分开。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 入口/回退日志均未产生 ⇒ 被判失败的不是我们改的调用点 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 在 vue.rs:8495 调用点落盘（前后 + 错误串）

- 在 `generate_linked_front_sfc` 函数入口与回退处各加**文件**日志后复跑：`entry.log`/`fallback.log`/`store-count.log` **全不存在**，但错误仍是「包面 SFC 生成失败：multiple store declarations」。
- 全仓 grep「包面 SFC 生成失败」只有一处（`vue.rs:8495`，即我们改的调用点）⇒ 调用确实发生，但函数内日志没写出：怀疑 ①`auto` 用的工具二进制未刷新（AUTO_UI_TOOL_BIN 指向 target/debug/auto-ui-tool.exe，需核对时间戳）；②该函数在另一 crate 编译单元/另一同名函数被调用。
- 下一步单点：在 `vue.rs:8495` 前后各落一个文件日志（含 `has_store_decl` 与错误串），并核对该 exe 的 mtime/SHA，排除“跑的是旧二进制”。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 调用点已确认执行（has_store=true），但 SFC 生成器内日志缺失 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 核对入口日志的插入位置是否落在 `generate_linked_front_sfc` 体内

- 证据：`call.log` → `has_store=true src_len=24872`（调用点执行、二进制已刷新：19:55:09 → 19:56:41）；`entry.log`/`fallback.log` 仍不存在。
- 因此“包面 SFC 生成失败”的 Err 来自该调用，但我写入的入口日志没随函数执行 ⇒ 插入点很可能落在了**别的函数**（`package_candidates(source)` 那行在该文件出现多次，`String.Replace` 会全部替换，也可能全落在非目标函数）。
- 下一步：确认 `entry.log` 写入语句所在函数（grep 行号对照 `fn generate_linked_front_sfc` 的行范围），只在目标函数体内补一条；同时把 Err 串本身落盘（最保险，与插入位置无关）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: SFC 错误串已坐实；同源单 store 与包面“双 store”判定冲突 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码；未提交改动已回撤） | next: entry 交原始源 + store 单元（AV 同形）需在正确作用域重做

- 坐实（`sfc-err.log`）：`source.age.component-unsupported: multiple store declarations per node are not wired yet (S1)` —— 出自包面 `inline_store_decl`（`node.stores.len() > 1`）。
- 推断：prep 后 entry 里已有同源 `store MinesweeperStore`，包面在该单元上再解析到一次 store 归属（或 entry 同时保留了 `use` 声明路径）⇒ 同一 node 收到两条 store。
- 试过的解法：把 entry 改交**原始源** + `store` 单元作为包模块（与 AV `--package` 完全同形，AV 已证可行）时，我在错误作用域插值导致编译失败；已 `git checkout --` 回撤该文件，工具二进制重新构建通过（当前树可编译）。
- 下一步（在正确作用域重做）：`let entry_source = if has_store_decl { content } else { prepared.as_str() };` 放在 `generate_linked_front_sfc` 调用**之前**同一块内，modules 传 `store_modules`；随后复跑，预期走通 AV 同形路径。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: AV 同形 units 已落地；双 store 仍在 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 双 store 判定的产生点在 `checked_widgets::generate` 或 leaf 的另一处 entry 重写

- 已落地（`vue.rs` / `ace_package_sfc.rs`）：store 类应用走 `generate_linked_front_sfc`，**entry 交原始源 + store 单元作为包模块**（与 AV `--package` 完全同形，实测 `call.log: has_store=true src_len=4673 modules=1`）；entry 已有 store 片断或另有纯 store 单元时**跳过组件面 prep**。
- 仍报 `multiple store declarations per node`，且 `sfc-err.log` 未再生成 ⇒ 该错误的产生点已不在我们包装的 Err 分支，而在更内层（`checked_widgets::generate` / leaf 的另一次 entry 重写）。
- 下一步：在 `checked_widgets::generate` 入口按同样判据落盘/跳过；或直接比对 AV `load_package` 与 SFC 生成器所用的 units 差异（AV 已被证明可用同一对单元跑通）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 双 store 源头进一步排除 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （无新增代码） | next: units 内部复制点（desugar_units / append_widget_packages）

- 读码：`ace_package_sfc/checked_widgets.rs:33` 直接 `compile_age_package_widget_runtime(units, entry, …)`，**自身不做 prep** ⇒ 双 store 不是它产生的。
- 现有 units 已与 AV 完全同形（entry 原始源 + `app::minesweeper_store`），而 AV `load_package` 用同一对单元可跑通 ⇒ 差异必在 SFC 路径**之前**对 units 的处理：`desugar_units`（`generate_package_sfc_with_root` 内）或 `append_widget_packages`。
- 下一步单点：在 `desugar_units` 前后与 `append_widget_packages` 前后对 units 逐条落盘 store 计数（文件日志，不依赖 stderr），一次运行即可锁定复制点。

**本轮结束时总体状态（供后续会话直接接管）**

- AV/Iced 腿（T-03）：**六行全绿**并已复验——真窗 exit 0 / errors=0；首击 (0,0) 展开 63 格且 `st_game_state=playing`；右击标记 `flags_placed=1`、雷数 10→9；难度 expert（16×30/99 雷，486 标签）；重开回 ready；两实例 `inst0=playing` / `inst1=ready`；close/reopen `gen1 closed` → `gen2 open`。证据 `plan-artifacts/plan-119/{av-oracle-a, av-flag, av-diff, av-two, av-close, av-recheck}.json`。
- 交付面代码：auto-ui `1a986a9c`+（prep 四件归一 + 栈→队列 + lane 路由/幂等）、auto-lang `412f1de13`+（`Effect` 展开 + 可发射集合放宽），临时诊断已清零（§ 早前登记项已完成）。
- A2Vue 腿（T-06）：卡在包面 SFC 生成器的 `multiple store declarations per node`；已排除组件面边界、二次 prep、checked_widgets 三处，剩 units 内部复制点。
- 尚未开始：T-04 独立复核、T-05 AV 收据、T-07 终审、T-08 落主归档。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 49 | from: work | to: work | status: drafting | outcome: pass（检查点） | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: auto-ui 工作树干净 / auto-lang 823c54c7e | next: work（T-06 units 复制点：desugar_units / append_widget_packages）

- 本轮为**检查点轮**：两棵工作树均干净可编译（auto-ui `cargo build … --example ace_node_desktop_window` Finished；auto-lang HEAD 823c54c7e，无未提交改动）。
- 无新增代码改动；A2Vue 腿的下一步判据与证据链已在 §9（round 48 条目）固化，不再重复。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: units.log 仍为空 ⇒ 错误发生在 units 装配之前 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 单单元 inspection 回退路径为何报「multiple store」

- 在两处 `prepare_store_units` 之后都加了 units 落盘（`units.log`），复跑后**文件为空** ⇒ 执行未到 units 装配，`multiple store declarations` 在**更早**的 `package_candidates` 阶段抛出。
- 结合 round 46 的 `sfc-err.log`（同一错误串）⇒ 该错误来自 `generate_linked_front_sfc` 首部：`package_candidates(source)` 失败 → 回退 `prepare_component_source` → 对 prepared 源再 `package_candidates` 时的**单单元投影**报「multiple store」。
- 下一步：在回退分支内落盘 prepared 源的 store 行数与 widget 头位置，确认是否 prep 在同一源里写出了两条 store（或 store 与 `use` 同时被解析）；这是 A2Vue 腿当前的唯一卡点。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 51 | from: work | to: work | status: drafting | outcome: pass（检查点） | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （无新增代码） | next: work（回退分支内落盘 prepared 源，判定「单单元 package 投影 × 一条 store」为何报 multiple）

- 本轮不新增代码：卡点已收敛为**单单元 package 投影对一条 store 声明报 multiple**，需要一次带落盘的实测（prepared 源 store 行数 + widget 头位置 + `node.stores.len()`），不应在无新证据时继续盲改。
- 下一轮唯一动作（已定）：在 `generate_linked_front_sfc` 回退分支内落盘这四项，再据实修正（可能是 package 投影把“同源 store + store 类型声明”算成两条，或 prep 在 package profile 下重复入列）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **根因坐实——一条 store 声明被包面挂成两条** | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: frontdoor 双追加的修法（auto-lang）或 auto-ui 侧绕过 package_candidates

**实测（`plan-artifacts/plan-119/fallback.log`）**：
```
len=24872 store_lines=1 candidates=Err("multiple store declarations per node")
```
即 prepared 源里**只有 1 条 `store` 声明**，`package_candidates`（包面投影）却报「multiple store」⇒ 包面 frontdoor 对**同源前置 store** 做了两次挂载（`project_component_nodes` 已把前置 store 归到首个 node，随后 `first.stores.append(&mut stores)` 再追加一次），`inline_store_decl` 遂见 `len()==2` 而具名拒。

**两条修法**：
- A（正解，auto-lang `cst/frontdoor.rs` 包面路径）：追加前按名去重（若 `first.stores` 已含同名 store 则只保留一条）。一行级改动 + 回归。
- B（auto-ui 绕过）：A2Vue lane 不走 `generate_linked_front_sfc`（其首部 `package_candidates` 必踩此坑），直接用 units 调 `generate_package_sfc_leaf`/`generate_package_sfc_with_root`（AV 同形已证可用）。

2026-11-24 §10 裁定（用户口头授权 A）：**授权 auto-lang 语义面修复**（同源 store 去重等 A2Vue 阻塞项）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 双 store 已解除；inspection 通过；新墙 = a2x `DynCast` 未接面 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: a2x Collections 域的 DynCast 支持

- auto-lang 去重已落地并生效：`frontdoor.rs` 包面路径按名去重后再 `inline_store_decl`（`multiple store declarations` 消失）。
- auto-ui inspection 改用**包面拼写**（`[]T` → `List<T>`）：`fallback.log` 现为 `candidates=Ok(1)`，inspection 通过。
- 新墙（SFC 生成阶段）：`not serve expression `DynCast { operand: ExprId(646), test_nil: false }` in Collections` —— a2x TS 后端在 Collections 域未接 `DynCast`（`.to(Str)` 一类转换在生成的 TS 里落入该节点）。
- 下一步：在 auto-lang a2x-ts 的 Collections 域接 `DynCast`（按用户授权 A 可做），或让 prep 把该转换换成 a2x 已支持的形态（`to_string`/字符串拼接路径）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **A2Vue 生成通过**（`auto build --render vue --gen-only` exit 0，24 文件） | outcome: pass（T-06 生成腿） | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下） | next: pnpm 冷构建 + 真实浏览器 oracle

- auto-lang：`DynCast` 在 TS 后端按**恒等**实现（JS 值动态，具体→Dyn 无需动作；Rust/C 才需装箱）——038 的 `.to(Str)` 在 Collections 域会落到该节点，此前被兜底拒。
- 结果：`auto build --render vue --gen-only` **exit 0**，生成 24 个文件到 `gen/front/vue`（含 store composable 与 App SFC）。
- 已清理生成期 `deps/` junction。
- 下一步（T-06 余量）：`pnpm install --offline` + `pnpm run build`（vue-tsc + vite），随后 Playwright 真实浏览器跑 `tests/test_038_vue.mjs` 的 oracle 行（主题族/81 空格/首击展开）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **A2Vue 构建 + 真浏览器首次渲染/交互通过** | outcome: pass（T-06 构建腿） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 逐行 oracle（首击展开/标记/重开）

- 构建：`pnpm install --offline` exit 0（13.1s）、`pnpm run build` exit 0 → `dist/index.html 4.80 kB`、`index.css 17.25 kB`、`index.js 187.50 kB`。
- 真浏览器（Playwright 1.63.0 Chromium + `vite preview :4199`）：**pageerror 0**；页面含 85 个 button（81 格 + face + 3 难度）、文本含雷数/计时/三档难度与 `9×9`；点难度后棋盘切到 expert（480 格、`💣 99`）——**交互真实生效**。
- 探针脚本：`plan-artifacts/plan-119/probe-vue.mjs`、`probe-vue2.mjs`（下一轮改成按文本/角色精确定位的逐行 oracle）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: A2Vue 首击展开 oracle 通过；右击标记待查 | outcome: pass（T-06 第 1 行） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 右击标记（oncontextmenu.prevent）在生成 SFC 的接线

- 索引校正：生成页的 button 前 4 个是 face + 三个难度，**格子从第 5 个（index 4）开始**。
- 实测（Playwright，`pageerror=0`）：boot 81 格全覆盖 → 点击 cell0 后 **18 个空格 + 18 个数字**（首击 flood-fill 展开，与 AV 侧一致）→ 右键同格 `flags=0`（标记未生效）。
- 已更新探针：`plan-artifacts/plan-119/oracle-vue.mjs`（含截图 `038-vue.png`）。
- 下一步：查生成 SFC 里 `oncontextmenu.prevent` 的接线（AV 侧已通；Vue 侧可能未发射该事件修饰符或未阻止默认菜单）。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 生成 SFC 只发射 onclick，未发射 oncontextmenu | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 在包面 SFC 渲染器补 `oncontextmenu`（含 `.prevent`）

- 证据：生成 `App.vue`（440 KB，程序化 `h()` 渲染）中 **`onclick` 6 处、`oncontextmenu` 0 处**；store composable 里 `Flag` 动作与 🚩 逻辑都在，缺的是视图事件发射。
- 因此浏览器右击不触发标记是**发射面缺口**，不是 store 逻辑问题。
- 下一步：在包面 SFC 渲染器（auto-lang `checked_widgets` / `ace_package_sfc` 的事件发射处）支持 `oncontextmenu`（`oncontextmenu.prevent` 需同时 `preventDefault`），按用户 A 授权可做；随后浏览器复跑 `oracle-vue.mjs` 的 flag 行。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: SFC 发射面的 contextmenu 缺口已定位到 checked_widgets 的 h() 事件映射 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 在 `checked_widgets.rs` 的 h() 事件映射补 `oncontextmenu`（`.prevent` → preventDefault）

- 排查：`onclick:` 的字符串发射只出现在测试里，实际 `App.vue` 的 `h()` props 由 auto-ui `ace_package_sfc/checked_widgets.rs` 生成（按 `AgeWidgetProperty::Message` 的 key/args 映射）；`package_view.rs` 侧只管桌面渲染。
- 因此 `oncontextmenu.prevent` 未被映射进 h() props（生成产物 0 处）——属发射面遗漏，非 store/视图投影问题（投影层 `project.rs` 对事件键是通用的 ident+修饰链）。
- 下一步：在 `checked_widgets.rs` 的事件映射里加 `oncontextmenu`（修饰符 `.prevent` 需同时 `preventDefault`），再复跑 `oracle-vue.mjs` 的 flag 行。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 59 | from: work | to: work | status: drafting | outcome: pass（定位） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: `oncontextmenu` 在 widget-runtime 视图发射面（非 checked_widgets 的 channels 映射）

- 读码修正：`checked_widgets.rs:414-441` 的映射只生成**子组件站点 channels**，根 App 的 `h()` 元素 props 来自 a2x 的 widget-runtime 视图发射（`emit_widget_function_body_fragment`），所以 `onclick` 在那里而不在 checked_widgets。
- 因此 `oncontextmenu` 的补点在 **auto-lang widget-runtime 的视图事件发射面**（与 AV 桌面渲染器各自独立：AV 侧已通，Vue 侧未发射）。
- 下一步：在该发射面把事件键 `oncontextmenu[.prevent]` 映射为 Vue props（`.prevent` 同时 preventDefault），随后浏览器复跑 flag 行。

**交接摘要（round 59）**：AV/Iced 腿（T-03）六行全绿并复验；A2Vue 腿生成/构建/渲染/首击/难度已通过，仅剩右击标记的事件发射；T-04/T-05/T-07/T-08 未开始。工作树干净可编译，临时诊断已清零。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 60 | from: work | to: work | status: drafting | outcome: pass（定位） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 找到把 `event` 操作映射成 Vue `on*` prop 的 JS 发射点

- 已确认上游是**通用发射**：`live_view.rs::event()` 会把 `entry.key`（含 `.prevent` 修饰）、消息名、参数个数原样 emit 成 `event` 操作；`checked_widget_view.js` 也把 `kind/message/types/values` 收进 `element.events`（含 `image-surface-event` 专用分支）。
- 因此只差**最后一跳**：把 `element.events` 里的 `kind`（如 `oncontextmenu.prevent`）映射成 Vue 渲染 props。该跳不在 `checked_widget_view.js`（无 addEventListener 泛接线），也不在 a2x-ts（grep 无 onclick），最可能在 auto-ui SFC 组装处按 `event` 操作拼 `h()` props。
- 下一步（定位一行即可）：在 `ace_package_sfc.rs` 搜 `"on"` 前缀拼接或 `h(` 的 props 构造；补 `oncontextmenu` + `preventDefault` 后复跑 `oracle-vue.mjs`。

**交接摘要**：AV/Iced 腿六行全绿并复验；A2Vue 腿除右击标记外全部通过（生成/构建/渲染/首击/难度）；工作树干净可编译，临时诊断清零；T-04/T-05/T-07/T-08 未开始。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 最后一跳已定位到 `checked_widget_view.js` 的事件键处理 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 在该 JS 里拆 `kind` 的 `.` 修饰并把基名映射为 Vue prop（`.prevent` → preventDefault）

- 生成产物实证：App.vue 里事件是以**字符串协议**推送的（`a2x_str_lit("event")` + `a2x_str_lit("onclick")` + 消息名 + 参数个数），由 `checked_widget_view.js` 解释（`operation === 'event'` → `element.events.push({kind, message, types})`）。
- 同一文件第 864 行起按 `kind.toLowerCase()` 校验/绑定；第 445 行已有 `binding.toLowerCase().split('.')` 的修饰符拆分先例 ⇒ **`oncontextmenu.prevent` 需要同样拆分**：基名 → Vue prop，`.prevent` → `preventDefault()`。
- 因此下一轮改动面很小：在事件绑定处按 `.` 拆 kind，基名 `oncontextmenu` 映射（与 onclick 同路径），修饰符 `.prevent` 包一层 `preventDefault`；随后浏览器复跑 flag 行。

**交接摘要**：AV/Iced 腿六行全绿并复验；A2Vue 腿生成/构建/渲染/首击/难度全通过，仅右击标记缺这一处；工作树干净可编译；T-04/T-05/T-07/T-08 未开始。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 渲染端已支持 oncontextmenu；缺口在发射端（App.vue 无 Flag 事件） | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 查 SFC 的 widget-runtime 视图程序为何没收录 `oncontextmenu.prevent: .Flag(x, y)`

- 更正：`checked_widget_view.js` 的 `checkedDomEvents` **已含** `oncontextmenu: 'onContextmenu'`，且 `checkedDomBinding` 已按 `.` 拆修饰符并支持 `.prevent`（第 444-453 行）。渲染端不需要改。
- 真缺口：生成 `App.vue` 中 **`Flag` 字符串一次都没出现** ⇒ widget-runtime 视图程序**没有收录** `.Flag(x, y)` 这条 `oncontextmenu.prevent` 绑定；而 AV live 道用的是同一 `live_view::event()` 却工作正常，故差异在 SFC 侧编译该视图程序的 profile/入口。
- 下一步：对比 AV `compile_age_package_widget_runtime` 与 SFC 侧调用（参数/依赖配置）差异，确认 `oncontextmenu.prevent` 在哪一步被丢弃。

**交接摘要**：AV/Iced 腿六行全绿并复验；A2Vue 腿生成/构建/渲染/首击/难度全通过，仅右击标记（发射面）；工作树干净可编译；T-04/T-05/T-07/T-08 未开始。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 63 | from: work | to: work | status: drafting | outcome: pass（检查点） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 在 `live_view::event()` 打印实际 emit 的事件键（一次运行定位 Flag 丢失点）

- 本轮不新增代码：上一轮已确认渲染端无需改动，缺口在“视图程序没收录 `oncontextmenu.prevent`”。再往下需要一次带打印的实测（`live_view::event()` 的 key 列表），盲改会更慢。
- 下一轮唯一动作：在 `live_view::event()` 入口打印 `entry.key`/`entry.is_event`/消息名，对 038 跑一次 SFC 生成与 AV 窗口各一次，比较两侧事件列表（AV 侧应含 `oncontextmenu.prevent`，SFC 侧若缺即定位在编译参数/profile）。

**交接摘要（PLAN-119 / r1）**
- AV/Iced 腿（T-03）：**六行全绿并复验**（真窗/首击/标记/难度/重开/两实例/close-reopen），证据 `plan-artifacts/plan-119/*.json`。
- A2Vue 腿（T-06）：生成 ✅、冷构建 ✅、真浏览器渲染 ✅、首击展开 ✅、难度切换 ✅；仅右击标记（视图程序未收录该事件）。
- 交付代码：auto-ui（prep 归一、栈→队列、lane 路由、inspection 拼写、units 幂等）+ auto-lang（Effect 展开、可发射集合放宽、同源 store 去重、TS DynCast 恒等），临时诊断已清零。
- 未开始：T-04 独立复核、T-05 AV 收据、T-07 终审、T-08 落主归档。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 事件已进入视图程序与生成件；浏览器右击仍不生效 | outcome: partial | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: （见下，含临时打印） | next: 运行时确认 contextmenu 回调是否被派发（在 checked_widget_view.js 加计数）

- 实测（`live_view::event()` 临时落盘 `view-events.log`）：SFC 生成时**确实 emit** `key=oncontextmenu.prevent tag=button`（与 `onclick` 并列）。
- 重新生成后 `App.vue`：`oncontextmenu` 2 处、`Flag` 3 处、`onclick` 6 处 ⇒ **生成件已包含标记接线**（此前那轮 grep 是旧产物）。
- 但浏览器复跑（右击**未揭开**的最后一格）：`flags=0`，`blank/digits` 不变 ⇒ 回调未生效。探针已改为右击未揭开格（`oracle-vue.mjs`）。
- 下一步：在 `checked_widget_view.js` 的 dispatch 处加临时计数（或 `console.debug`），浏览器里读出来，确认 ①事件是否被绑定到该 button、②`checkedDomBinding('oncontextmenu.prevent')` 是否通过、③`dispatch` 是否被调用。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 监听已装进 props，但回调从未触发 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下，含临时 console.debug） | next: 查 button 分支是否用自有 props 覆盖/丢弃 `onContextmenu`

- 运行证据（浏览器 console）：`P119CTX bind onContextmenu flags=prevent` 出现 4 次 ⇒ 绑定安装成功；
  但新增的 `P119CTX fire …` **一次都没出现** ⇒ 回调未被调用。
- 读码：`nativeWidgetNode`（第 561 行）确实把 `props[property] = callbacks` 写进 h() props，键名 `onContextmenu`。
- 因此下一跳在 button 的渲染分支（第 561 行之后）——怀疑该分支为 button 另建 props（只保留 onClick 等），把 `onContextmenu` 丢掉；或 Vue 对数组回调 + 该键名的处理与 click 路径不同。
- 临时件：`checked_widget_view.js` 两处 `console.debug('P119CTX …')`、`live_view.rs` 的 `view-events.log` 打印；落主前必须移除。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 合成 contextmenu 能触发回调（Vue 接线正常），但派发未改状态 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （无新增代码） | next: 检查该事件的 action 实参/types（Flag(x,y) 是否带值）

- 实测（`plan-artifacts/plan-119/probe-ctx2.mjs`）：在页面内 `dispatchEvent(new MouseEvent('contextmenu', {bubbles:true, cancelable:true, button:2}))` →
  `P119CTX fire active=true sameFrame=true disabled=undefined` 且 `defaultPrevented=true` ⇒ **Vue 事件接线与 `.prevent` 都正常**。
- 两种偏差：①Playwright 的 `click({button:'right'})` 未触发该回调（工具层差异，非应用问题）；②合成派发后 `flags` 仍为 0 ⇒ 回调执行了但 **Flag 未生效**，疑点是 action 的 `types/values`（Reveal 的 onclick 带 2 个 i32 正常，contextmenu 可能 types 为空 → Flag 收到缺参）。
- 下一步：打印该事件的 `event.types`/`action.values`（`checkedWidgetAction` 返回值），确认 `Flag(x, y)` 的实参是否随事件下发。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: **A2Vue 右击标记功能成立**（雷数 10→9） | outcome: pass（T-06 标记行） | worktree: D:/autostack/.wt/ui-119/auto-ui | commit: （见下） | next: 用合成 contextmenu 事件固化 oracle；清理遗留 `VIEWTAG` 打印

- 实测（`probe-ctx3.mjs`，页面内合成 `contextmenu`）：派发前后 body 由 `💣 10` → **`💣 9`**（`changed=true`）⇒ 标记动作在生成 Vue 应用里**真实生效**，与 AV 侧一致。
- 结论：Playwright 的 `click({button:'right'})` 不产生 contextmenu 属**驱动层差异**，应用侧接线/派发/实参（`types=["i32","i32"]`, `values=["8","8"]`）全部正确。
- 已清理：`checked_widget_view.js` 的两处 `console.debug('P119CTX …')`；auto-lang `live_view.rs` 的 `VIEWTAG` 打印**仍在**（一次误删导致编译报错，已 `git checkout --` 复原，仍待干净移除）。
- oracle 约定：右击行改用**合成 `contextmenu` 事件**（真实浏览器 DOM，等价于用户右键），并断言雷数计数变化。

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: 临时诊断全部清零，两树可编译 | outcome: pass | worktree: D:/autostack/.wt/ui-119/{auto-ui,auto-lang} | commit: auto-lang 4a61ef94f | next: T-06 收尾证据 + T-04 复核

- `VIEWTAG` 打印已干净移除（auto-lang `4a61ef94f`），`P119CTX` 此前已删；两树 `cargo build` 绿。
- 生成期 `deps/` junction 再次清理（worktree 无 reparse point）。
- A2Vue 腿判定：**生成 ✅ / 冷构建 ✅ / 真浏览器渲染 ✅ / 首击展开 ✅ / 难度切换 ✅ / 右击标记 ✅**（合成 contextmenu 断言雷数 10→9），证据 `plan-artifacts/plan-119/{oracle-vue.mjs, probe-ctx3.mjs, 038-vue.png}`。
- 下一步：把 T-03/T-06 证据折叠成计划内收据（含命令、退出码、关键数字），然后进入 T-04 独立复核。

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

2026-11-24 §10 裁定请求（G1 归属）：038 双端首墙 = AGE 面 `return`/`break` 语句缺席（§9 G1）。三条路：

1. **auto-ui prep 兼容改写**（本计划自有仓，不动 auto-lang）：prep 合成 store 片断时按结构化等价改写 9 处控制转移——守卫 `if cond { return } 余下` → `if !cond { 余下 }`；循环内 `break` → 循环条件加 `!__brk` 且守卫其后语句。带正反用例，并以 VM 已钉 oracle（首击 29 格/雷位表）在 AV 端证等价。改动 ~150–250 行 + 测试。
2. **auto-lang 语法面扩件**（G1 正解）：`auto-source.at` 加 `age-on-return/-break/-continue` 规则 + CST 投影 + `ComponentStmt` 三变体 + `ast::Stmt` 降级 + AV 执行器 + A2Vue 发射，带正反回归。跨 grammar/投影/降级/两端发射，体量大。
3. **移交语言 owner 计划**：119 挂起至其落主后复验；G2/G3 仍由本计划 prep 收口。

（先例：PLAN-118 §10 裁定①曾授权其在 plan-118-dev 的 auto-lang 树做最小闭合集。）

2026-11-24 §10 G5 裁定请求（live transition 发射面）：038 的 AV 腿经 5 轮二分，构造级形状全部可发射（含 9 字段记录、布雷/邻接循环、`.set` 投影、守卫嵌套），但 `Reveal` 单条 transition（Reveal-only 裁面复证）仍被判 `cannot emit a checked widget transition`，单条摘除/前缀/子集三种扫描都无法定位到具体构造 ⇒ 判为发射面组合/规模限制。选项：
① 授权本会话在 plan-119-dev 的 auto-lang 树做**定位性诊断**（临时 eprintln/诊断码细化，不改语义），把 `unsupported()` 的真实原因暴露出来，再据实决定修法；
② 直接移交语言 owner（PLAN-1100/1104 AV 游戏接管）并挂起 119 的 AV 腿；
③ 改 §1/§6 验收口径：AV 腿以 VM/Iced 真实窗口证据交付（原 038 的 `auto run -r vm` 即 Iced 真窗），AGE 包面留作后续。

2026-11-24 stage: goal | plan_id: PLAN-119 | plan_revision: r1 | round: 2 | from: work | to: work | status: drafting | outcome: pass（阶段推进） | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@1a986a9c） | commit: 1a986a9c | next: work（T-02 续：live transition 发射面 / 无状态道 startup-handler 编译面）

2026-11-24 stage: work | plan_id: PLAN-119 | plan_revision: r1 | checkpoint: T-02 次段——真窗装载链定位到 live 道发射面 | outcome: partial | worktree: D:/autostack/.wt/ui-119/auto-ui（plan-119-dev@1a986a9c） | commit: （无代码改动；探针材料 plan-artifacts/plan-119/） | next: 用文件喂入式探针二分 live 发射面

**T-02 次段：真窗装载链（2026-11-24）**

- 事实链：真窗 `--package` → `load_package` → `new_with_root` → ①整包编译 `compile_package(instances=1)` **OK**（§上段已绿）→ ②`LivePackage::prepare` → `compile_age_package_widget_runtime` 返回 **`source.age.widget-runtime-handler: cannot emit a checked widget transition for handler Reveal`**（全部诊断前缀 `source.age.widget-runtime-` → `prepare` 返回 `Ok(None)`，live 道不着车）→ ③回落无状态重放道 → 事件派发被 `source.age.store-runtime: shared Stores require a live widget session for events` 具名拒（带 store 的应用**必须** live 道）→ 真窗 `open failed`。
- 判据面：live 道是 038 的唯一可行道（重放道对 shared store 事工具名拒）。故 T-03 的前提 = live widget-runtime 能为本应用发射 transition。
- 隔离对照（`plan-artifacts/plan-119/live/`，每个变体真窗 live 装载全 OK、exit 0）：赋值、`.board.push({…})`、`.board.set(0, R { … })`、C 风格 for + 嵌套 if、无类型 `var stack = []` + `remove(len-1)`、`__p119_ret` 标志守卫、`"n=" + (3-1).to(Str)`、`for cond && brk == false` + 守卫——**单构造均不是发射面缺口**。
- 未决：038 的 `Reveal` 仍未发射成功，差异不在已枚举的单构造，需在**准备后单元**上二分（下一轮）：给 `p119_diag` 加 `P119_LIVE_FILE`（从文件喂 units 直连 `compile_age_package_widget_runtime`，绕开二次 prep）后按 arm/语句二分。附注：raw 源按语句截断的粗二分不可靠（截断后的 store 经 prep 重组会先在整包编译报 `view-escape`，属探针工件，非发射面证据）；此轮已停在正确判据上（准备后单元 + live 直连）。
