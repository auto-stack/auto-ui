---
plan_id: PLAN-105
status: executing
feature_name: AutoUI U03 本地应用与编辑 AV 与 A2Vue 交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 1
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-105 AutoUI U03 本地应用与编辑 AV 与 A2Vue 交付

## 0. 变更摘要

2026-11-19 按用户裁定建立独立交付计划。承接语言 980 B-AV、1071 冻结清单中 U03 的原示例和关联 Widgets，验收 ACE AV 桌面和 A2Vue 浏览器。由本计划负责 AutoUI 和所需 AutoLang 改动，并补齐这些应用实际用到的缺失组件。A2R、AC 由后续独立计划接同源验收。本次为规划，进度 0/8。

## 1. 目标

以下 6 个原项目及关联 Widgets 各有两端可复跑结果。复用原 `pac.at`、作者源、依赖及独立预期。交付可从干净目录构建和运行的项目、固定工具与 SDK 版本、每端操作记录及独立复审结论。

| AutoUI 原例 | 两端项目级检查重点 |
|---|---|
| [012-clock](../../examples/ui/012-clock/pac.at) | 开关窗、暂停恢复与定时节奏 |
| [013-todo](../../examples/ui/013-todo/pac.at) | 新增、筛选、切换状态与列表更新 |
| [015-notes](../../examples/ui/015-notes/pac.at) | 编辑、保存或取消、关闭恢复与原数据契约 |
| [016-calendar](../../examples/ui/016-calendar/pac.at) | 日期选择、月份切换与日程原交互 |
| [018-book-reader](../../examples/ui/018-book-reader/pac.at) | 翻页、滚动及进度或焦点的原行为 |
| [022-kanban](../../examples/ui/022-kanban/pac.at) | 卡片状态与分栏交互、取消或切换后的隔离 |

关联 Widgets 使用 [AutoOS 规范来源](../../../auto-os/widgets-gallery/src/front/) 的 pages 与实际 components 闭包。calendar、scroll、scrollarea、datatable、table、accordion、collapsible、carousel、pagination、sidebar、toast、sonner、progress、slider。T-01 把 1071 标识逐项对应到实际页、组件与原测试，给复用页记录版本和各组消费结果。

## 2. 架构与跨仓职责

- **AutoUI 主责。** 负责示例、组件契约、Iced 宿主消费、Vue 生成产物及真实交互。先闭 Todo 的列表与事件，再跑 Clock、Notes 及剩余项目。用现有 Store、编辑会话和浏览器运行时绑定分别验两个后端。定时器同一活动实例只执行一次，关闭、页面切换和重开后旧回调无效果。表格/滚动/焦点回归后再把组件接回原应用。
- **AutoLang 为同一计划的实施仓。** 检查跨模块 helper、集合绑定与列表身份、Store 的事件/载荷、定时回调和异步取消。AutoLang 在已有受检语言与 AV/A2TS 能力上补实际缺口，校验回调世代、类型与失败路径。复用已交日期、文本、文件或调度接口并固定消费版本。 相关改动在组内 AutoLang 树完成，沿原模块 owner 审查与合入。
- **真实组件。** 对本组使用的每个缺失组件或缺失行为，优先补已有组件，再补所需新组件。交付适用属性、事件、绑定、生命周期和无障碍行为，两端实现、永久回归及原项目调用都计入本计划的完成条件。已有其他计划在写同一组件时，先对账实际界面与写面，固定单一实施负责人，当前计划持有本消费者验证及关闭条件。

## 3. 技术与实现入口

AV 沿 `crates/auto-ui-runtime/src/ui/ace_node_desktop.rs` 的公开宿主路径、受检 AGE Schema 和 typed 事件。A2Vue 沿 `crates/auto-ui-runtime/src/ui/ace_vue_script.rs`、`ace_ts_provider.rs`、`ace_package_sfc.rs` 和正式 `auto-ui-tools` 入口，验生成的 Vue/TS、运行时与资源。AutoLang 优先复用已有 AGE/ACE、AV 和 A2TS 管线。Windows AV 及所选浏览器配置各有实测记录，其他原适用平台按冻结的运行条件记账。

## 4. 来源与前置

- [980 原 W1 和 ④-A 设计](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 和 [1071 示例/Widgets 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)。已有运行收据按实际输入、范围和版本复用。
- [AV 宿主及 W1 收据](../../../auto-lang/docs/specs/ace-migration/m2-autoui.md)、[1072 Vue 生产计划](../../../auto-lang/docs/plans/1072-autoui-vue-web.md)、[976 TS/JS 交付](../../../auto-lang/docs/plans/archive/976-ace-m2-typescript.md)、[978 构建交付](../../../auto-lang/docs/plans/archive/978-ace-m2-build.md)。
- U02–U06 按用例所需接口推进。遇到共享的 ④-A、事件、集合、外部服务接面或组件缺口，记录具体接口、实现负责人、交付版本与消费测试，已具备前置的样例继续验收。1072 的 982 联合 CLI 验收可消费本计划的 A2Vue 证据。
- **授权（2026-11-19，首条 goal 收据同记）**：用户目标「完成计划105」按 `/auto-plan-goal` 解释为全链授权——work（含建树）→ review → 合同内有限修复 → merge（canonical Specs、索引诊断非阻断、归档、工作树清理、落地核对）。不授权改目标/验收/阈值/兼容承诺/仓集/预算，也不授权跳过工作树与证据纪律。
- **共享实现线（用户裁定，2026-11-19）**：105 与 104 共用一套 auto-ui/auto-lang 实现，相同组件或 UI 能力有缺陷时不各自开发一套。105 实施树以 `plan-104-dev` tip 为基（auto-ui `d2e7d7a4` / auto-lang `2b829c5d1`），每轮先同步 104/1072 已提交的共享提交；G1–G5（props 形、str 比较、动态 style/view-if、列表初值/view-for、主题数据）由 104 与 1072 在途实现，105 只 rebase/cherry-pick 消费、不重写；U03 新增的 store/后端/子组件面（G6–G11）由 105 在共享 ACE 模块一次实现，供 104/106–108 复用。不建组件副本、不复制生成器、不在 `examples/ui` 另立一套。
- **实施树（2026-11-19 T-01 建立；2026-11-19 用户裁定后停放）**：原 `.wt/ui-105/{auto-ui,auto-lang,auto-web,auto-down}` 与分支 `plan-105-dev` 已删除并 `wt-guard` 收口。**恢复时按 master 重建**（不再基于 104 分支、不共享工作树）：`D:/autostack/.wt/ui-105/{auto-ui,auto-lang}` 分支 `plan-105-dev` 从当时 master 起，auto-web/auto-down 按路径依赖同级检出。恢复前置 = 104 已提交并合入 master。

## 5. 实施与交付边界

先冻结 U03 矩阵，逐行绑定项目原源、Widget 页和组件、预期、失败反例、权限、资源、版本和 AV/A2Vue 状态。从原项目的第一个失败处建立可复跑入口，修所用能力或组件，回同一项目完成两端验收。公共语义走原 owner 的统一接口，源级修复覆盖 AV 和 A2TS 的实际消费者。

实施使用 `D:/autostack/.wt/ui-105/{auto-ui,auto-lang}` 的关联工作树。用到其他仓版本时明确固定提交，按组内路径或既有显式覆盖解析。合同、进度和归档在 AutoUI 主检出管理，实施和构建在工作树。跨仓落地顺序以实际生产者和消费者验证为准。

## 6. 测试设计

| 项 | 独立证据 |
|---|---|
| V-01 来源与覆盖 | 原项目、Widget 页、具体组件与两端矩阵对账。逐例固定原入口、工具/SDK/Schema、期望值和资源。 |
| V-02 语言与宿主 | 受影响的 AutoLang 测试组、实际 AV/A2TS 消费用例、有效用法与相邻坏源、诊断 span 和错误恢复。回归测试保留对共享语义的判别力。 |
| V-03 组件 | 新补的组件及行为具名入表。两端测试属性/事件、键盘或指针、可访问性、关闭或卸载及所用项目正反例。 |
| V-04 AV 原项目 | 真实 Iced 窗口运行全部本组项目及相关 Widgets，核实际输入、状态、输出、生命周期与资源。按触面复用 `ace_node_desktop` 等入口并记录窗口准备。 |
| V-05 A2Vue 原项目 | 从干净目录经公开工具生成并构建 Vue/TS 和运行时，在真实浏览器中跑相同原源与独立行为断言，核真实 ACE 生成与构建派发、关闭和错误恢复。 |
| V-06 交付复跑 | 冻结两个仓的提交与各端依赖，复跑对应矩阵，核发布产物、可执行命令和子进程退出。原适用行均有真实结果。 |

## 7. 验收标准

| 编号 | 完成条件 |
|---|---|
| AC-01 | 示例、Widgets、资源和各端原预期完整追溯，版本固定。 |
| AC-02 | 本组所需的 AutoLang 改动已交，对应组件缺口在实际应用中关闭。V-02/V-03 正反两面通过。 |
| AC-03 | V-04/V-05 中本组每条必验项均有各自的通过证据，生成、运行、错误恢复及资源终态可复查。 |
| AC-04 | 独立复审通过，按对应仓流程合入修复并发布 V-06 收据。向 W1 汇总提供 AV 格，向 1072 提供 A2Vue 格，归档和清理本计划工作树。 |

## 8. 执行步骤

- [x] **T-01｜work** 冻结原项目、Widgets 与两端覆盖矩阵，确认首批运行顺序及共享接口和组件的唯一写面。收据见 §9（2026-11-19 T-01）：六例原源事实、组件面/包面首阻断、缺口台账 G1–G11、共享实现线与唯一写面已冻结；矩阵与原始日志在 `D:/autostack/plan-artifacts/plan-105/t01-matrix.md` + `t01-probe.log`。
- [ ] **T-02｜work** 补本组所需 AutoLang 源码、AV/A2TS 语义与宿主接口，交正反例及来源。
- [ ] **T-03｜work** 完成缺失组件及行为、双端接线和实际消费者回归。
- [ ] **T-04｜work** 从真实入口逐例验 AV/Iced、权限、错误恢复、关闭与资源。
- [ ] **T-05｜work** 从真实入口逐例验 A2Vue 生成、构建、浏览器行为、关闭与错误恢复。
- [ ] **T-06｜work** 冻结各仓提交，完成受影响回归及两端可复跑交付，补齐矩阵和规范增量。
- [ ] **T-07｜review** 同树独立终审 AC-01..04，核缺失行、错误路径、共享归属和收据，修复发现后复验。
- [ ] **T-08｜merge** 按 owner 顺序正式落地，记录 AutoUI/AutoLang 来源，通知证据消费方的交接在计划记录中登记。归档并按仓规清理。

## 9. 复审记录

2026-11-19 stage: new，drafting 0/8。U03 的原样示例、Widgets、AutoLang 支持和组件补齐均处于待实施状态。

2026-11-19 stage: work（T-01 冻结矩阵与首阻断）| PLAN-105 r1 | outcome: pass | worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev`（基 `plan-104-dev` = `d2e7d7a4`）/ `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev`（基 `plan-104-dev` = `2b829c5d1`）/ 路径依赖 auto-web detached `3d328a3`、auto-down detached `72b0970` | task_ids: T-01 | evidence: `D:/autostack/plan-artifacts/plan-105/t01-matrix.md` + `t01-probe.log`（SHA256 `0C9D0E11C1CC0EFE36C5BFC7CF601524C7C9C25FF692EF8C638DAEBEFCFBE8A3`，4344B）

**原源冻结（6 例）**：012/700 行/20 style/21 msg/无 store；013/132/0/3/store + `TodoList()` 子组件 + `.store.*`；015/125/2/6/store + `NavTree`/`EditorPanel`；016/225/4/6/store + `computed` + `for`×3 + 6 处动态 style；018/112/1/2/store + props 子节点；022/72/0/2/store + view-if。全部 `dep themes`（015/016/018/022 另 stylekit）。

**两端入口实测（本次真实运行，非静态推断）**：`cargo test -p ace-core --test plan105_u03_probe`（临时探针用后删净，980/104 先例；0.09s 用例，构建 34.0s）。包面 6/6 同点失败：`source.syntax: expected fn, found widget` @`widget App {`（item 级回滚位泛错，同 104 T-01/T-02 判读，真因在例内）。组件面投影（绕回滚位）真因：012 `var cities []str = […]` 列表初值 @model 字段；013 `checked: .store.active_count == 0 && .store.todos.len() > 0`；015 **投影 OK**（墙在 store/子组件/on 体）；016 `msg {` 多行块；018 `sidebar_menu_button (to: "/", exact: true)` props 子节点；022 按钮内 view-if 的 `}` 回滚位。`store S { model { … } }` 最小复现=组件面 `expected node at the declared node slot`（AutoUI `store` 不在 AGE item 面；AGE `StoreItem` 是模块级可变 var，勿混）。

**判读**：U03 = 104 共享面（G1 props 形 / G2 str 比较 / G3 动态 style+view-if / G4 列表初值 / G5 主题数据接面）∪ U03 新增面（G6 store+后端 `use back.api`、G7 `.store.*`/`&&`/`.len()`、G8 子组件实例化与组合、G9 msg 多行/computed、G10 f-string/svg/`view mini`、G11 Tick 定时）。105 不另立实现：G1–G5 等 104 在途提交落后 rebase 消费；G6–G11 由 105 在共享 ACE 模块（`age_package.rs`/`age_component.rs`/`grammars/auto-source.at`）一次实现，保留可被 106–108 复用的唯一实现。

**Widgets**：14 页（calendar/scroll/scrollarea/datatable/table/accordion/collapsible/carousel/pagination/sidebar/toast/sonner/progress/slider）实读均无 `use`/components 闭包（自包含 widget），普遍依赖 props 形；只读 auto-os 规范源，不写。

**主检出观察**：`packages/themes/*`、`examples/ui/stylekit/*` 未提交删除仍在（104 T-01 已记，owner 待收口）；merge 的 master-zero-WIP 门对 104/105 同时生效。105 工作树从 HEAD 建、文件在位，本计划不受影响。
`next_responsible_role: work（T-02，入口门=104 共享提交落 plan-104-dev 后 rebase）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: d2e7d7a4 | next: work

2026-11-19 stage: work（T-02 共享面合流——只消费，不重写）| PLAN-105 r1 | outcome: in_progress（T-02 未完成，等待 104 包面主体投影后续提交）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | commit: auto-lang `c784de094` / auto-ui `683e0e2a` | task_ids: T-02（partial）| evidence: `D:/autostack/plan-artifacts/plan-105/t02-probe-union.log`、`t02-grammar-census.log`

**并入共享面（只消费）**：①PLAN-1072 T-02 刀A/刀B（组件面 style 开口、view-for + 仅视图数据、条件样式值）→ auto-lang `d27a7450c` / auto-ui `683e0e2a`（auto-ui 为 fast-forward，auto-lang clean merge）；②PLAN-104 T-02 W1-U02 接面（括号属性形、`type:` 属性名、str 比较）→ auto-lang `c784de094`。冲突两处：`age-event-key` 采用 104 的 `age-event-key-base`（ident | `age-entry-type-key`）并保留 1072 注释；`plan981_grammar_scope` 两行按并集树重冻结——age-component `42afe2cb…`→`922807b2…`、age-package `91c53cde…`→`dc6ffe3c…`，13 邻居逐字节不变（census 7/7 绿）。

**合流验证（ace-core conformance，串行）**：plan104_u02_surfaces 4/4、plan1072_component_style 15/15、plan981_age_package 12/12、plan980_lambda_if 5/5、plan979_component 24/24、grammar 身份 7/7——全绿（0 失败）；临时探针用后删净，工作树 clean。

**U03 复测（合流线）**：包面 6/6 仍 `expected fn, found widget`——104 的包面主体投影仍在途（W1-U02 三处接面是前置），等其后续提交再 rebase。组件面：012 仍 `[]str` model 初值；016 仍 `msg` 块；013/018/022 回滚位由 `expected text` 变为 `expected type at declared entry-key`（104 新增 name 规则后诊断落点变化，非法源结论不变，真因待逐例二分）；015 投影 OK；`store S {…}` 仍 `expected node`。

**判读**：单一实现线现已同时含 104+1072 的共享面，105 未写任何重复实现；T-02 剩余门 = 104 包面主体投影后续提交（包面 widget 体）+ U03 store/子组件面（105 单写）。`next_responsible_role: work（T-02 续）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: c784de094 | next: work

2026-11-19 stage: work（T-02 暂停）| PLAN-105 r1 | outcome: waiting | task_ids: T-02（未完成）| evidence: `D:/autostack/plan-artifacts/plan-105/t02-gap-map.md` + `t02-synthetic*.log` + `t02-022-bisect.log`

**暂停原因（用户 2026-11-19 指示）**：105 与 104 同时开发、却共享依赖，不合适。先停止 105 的开发；等 104 提交并合并到 master 后，再按 master 重开 worktree（**不再共享**）。

**已收口**：中止进行中的「104 T-02 第二批」合并（`git reset --hard c784de094`，丢弃未提交的现场收敛），auto-ui/auto-lang 两树 clean；`wt-guard` 四树 clean；worktree×4 与分支 `plan-105-dev`×2 已删除；组目录 `.wt/ui-105` 已删；构建缓存 `plan-artifacts/plan-105/target`（471.9 MB）删除，实测日志与矩阵保留。

**本轮新增证据（停放前完成）**：U03 面 40 例最小合成源矩阵（§9 汇总；全表见 `t02-gap-map.md`）——已通面：view-if/else、str 比较、design 条件 mode、子组件（含 props）、style 引用/条件 style、简单 checkbox 绑定、多行 msg（无载荷）；待补面：model 列表字面量初值、组件面投影 entry 值内 `&&`、包面 int→text（`clone` 拒）、msg 载荷变体逐行、f-string 文本、store 声明/`back.api`/`.store.*`、`outlet`（022 墙，替换对照确认）、timer 块。缺口归属 S1–S9 与实施顺序见 `t02-gap-map.md`。

**合流经验（留给 104/1072 合流参考，未落代码）**：105 曾将 1072 刀A/B 与 104 的 G1/G2 合到一树（`c784de094`），实测 **1072 的 `age-if-expr` / `ComponentEntryValue::If` 与 104 的 `age-entry-value-cond` / `ComponentEntryValue::Cond` 是同一语义的两套实现**（条件条目值/动态 style）。105 现场按 `Cond` 单一语义收敛（project.rs 两支同投影、frontdoor.rs 两处、plan1072 测试），未提交。建议 104 与 1072 后落 master 者按先落者收敛，避免双实现长期并存；105 恢复后只消费 master 上的收敛结果。

**恢复条件（精确）**：`D:/autostack/auto-ui` 与 `D:/autostack/auto-lang` 的 master 已含 104 的交付提交（`git log master` 可查其 Plan-104 trailer）后，按 §4 从 master 重建 105 工作树（不再基于 104 分支），再续 T-02 的 S1–S9。`next_responsible_role: user（裁定恢复时机）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: waiting | worktree: （已停放，无） | commit: — | next: user

## 10. 下游交接

本计划按 U03 独立完成。AV 收据与 1071 已交范围及其他 U 组结果汇合后供 W1 签发、1068/T-04 和 985/B 接收。A2Vue 收据供 1072 与产品 CLI 消费。A2R/AC 后续计划沿同一源码、Widgets 和独立预期补验。跨仓 golden 基线维护按已有 PARITY 路线归档。

2026-11-19 停放：按用户裁定暂停本计划开发，等 104 合入 master 后从 master 重开工作树（不再共享 104 分支）。恢复时先读 §9 的 waiting 收据与 `plan-artifacts/plan-105/t02-gap-map.md`（S1–S9 缺口与顺序在案）。
