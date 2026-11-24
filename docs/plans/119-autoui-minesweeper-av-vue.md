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
