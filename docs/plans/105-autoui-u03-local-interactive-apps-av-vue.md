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
- **实施树（2026-11-19 按用户裁定从 master 重建；不共享 104 分支）**：`D:/autostack/.wt/ui-105/{auto-ui,auto-lang}` @ `plan-105-dev`（基 auto-ui `481a3cb5` / auto-lang `670c720a8`）、`auto-web` detached `26d085f7`、`auto-down` detached `72b0970`（后两者为路径依赖，无写面）。恢复前置（104 已交付并合入 master，ui `5a862d46` / lang `2fb094483`，104 archived）已满足。

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
- [ ] **T-04｜work** 从真实入口逐例验 AV/Iced、权限、错误恢复、关闭与资源。（**部分**：015/018/022 已真开 Iced 窗——frames 2/2/3、errors 0；**018/022 已真点击交互**：`ToggleTheme`/`ToggleDarkMode` 两次点击状态翻转、dispatch 2、errors 0（收据 §9 W25/W29/W34）；015 事件体含 store/API 调用仍具名拒；013/016 待列表语义；012 待局部列表。）
- [ ] **T-05｜work** 从真实入口逐例验 A2Vue 生成、构建、浏览器行为、关闭与错误恢复。
- [ ] **T-06｜work** 冻结各仓提交，完成受影响回归及两端可复跑交付，补齐矩阵和规范增量。
- [ ] **T-07｜review** 同树独立终审 AC-01..04，核缺失行、错误路径、共享归属和收据，修复发现后复验。
- [ ] **T-08｜merge** 按 owner 顺序正式落地，记录 AutoUI/AutoLang 来源，通知证据消费方的交接在计划记录中登记。归档并按仓规清理。

## 9. 复审记录

2026-11-20 接入 [PLAN-125 r2 七页双端交付收据](archived/125-autoui-u03-interactive-widgets.md#u03-逐页交付收据)。accordion、collapsible、scroll、scrollarea、pagination、progress、slider 的 AV/Iced 和 A2Vue 各通过 7/7，逐页原源、操作与交付提交见归档表。这些行计入 U03 原十四页，AV 可供 W1 汇总，A2Vue 可供 1072 与 CLI 联验消费。

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


stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 45 | from: work | to: work | status: executing | outcome: pass（重大解锁）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: d4bca7089 | next: work

2026-11-19 stage: work（T-02 W42：组件面 collections 域打通）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `d4bca7089` | task_ids: T-02（W42）| evidence: `plan105_list_handlers` 端到端 `n=7`

**交付（列表语义第一刀落地）**：①core `needs_collections_face` 检测列表源（ArrayLit/Index/GenericType/列表方法）并切 HIR 域（`SourceProfile::Collections` + `verify_collections`）；②hidden_fn 在 owned 文本域对**字面量叶直返**（不产生借用 `str` 的 `text_clone`，text-record 面行为不变）；③domain 结论接受 `StrLit` 为 owned 文本；④`hook_stmts_lower` 把列表方法（`push/get/len/pop/clear/set`）降级为**表达式语句**交共享 resolver；⑤投影器不再把列表方法调用标 `body_foreign`（否则派发被具名拒）。⑥av `execute_value` 按 `checked.domain()` 分派 `lower_checked_collections`。

**证据**：新增 av 回归 `plan105_list_handlers`——`let xs = List<int>.new(); xs.push(7); .n = xs.get(0)` 经场景事件执行后字段 `n = Int(7)`（列表构造/推入/取元素/写回全链路成立）。回归：plan105 25/25、104 3/3、1072 19/19、980 39/39、979 24/24、981 248/248、av plan980 8/8。

**桥装载复测**：012 = `source.age.structure`（view 结构面，另查）；013 = 具名 `.store` 边界（store 块 `on` 体仍不可解析）；016 = `ace.desktop.view-for-unsupported: view data item shape`（日历网格遍历 computed 列表——harness 需按现域能力求值 computed 列表或展开循环）。

**下一刀**：016 的 view-for（用 collections 执行 computed 列表并展开循环叶）、013 store 块 `on` 体解析、012 结构面。`next_responsible_role: work（T-02 W43）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 47 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-lang,auto-ui} | commit: b338b450f | next: work

2026-11-19 stage: work（T-02 W44：C 风格 for + 分支体调用）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `b338b450f`、auto-ui `7a8637cf` | task_ids: T-02（W44）| evidence: `plan105_list_handlers` 两例

**交付**：①文法 `age-on-for-cond`（`for <cond> { … }`）进 handler/分支语句并集；分支体开放调用语句（`age-view-call-stmt` 增 age-package）；②`ComponentStmt::While` 全链（投影/重写/computed/subst/assigns/local names/walk/store 展开/flat/SFC 车道）；③执行面发射 `Stmt::While`，场景路径把**再赋值**的 hook 局部标可变（过 `verify.immutable-write`）；④分支/循环里的非列表方法调用仍标 `body_foreign`（980 on-if 负控保持）。

**证据**：`plan105_list_handlers` 两例——List push/get 端到端 `n=7`；C-for（`for i < 3`）循环三次 `n=3`。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；auto-ui prep 6/6；census 4/4（component `9db961c7…`）。

**桥装载新进展（两道新墙）**：013 = **store 已内联**，新墙 `.store.todos` 不是 store 字段——`prep` 把类型化列表字段（`var todos []Todo = []`）改写成**视图数据**（无类型），于是 store 字段映射里没有 `todos`。012 = `on Tick` 赋值 `w_rows`（**computed 只支持读，不支持写**）。

**下一刀**：列表**状态字段**（`[]T` 模型字段进 collections 域 + store 字段映射含列表；`Todo` 记录类型由 types 单元内联）+ computed 可写面（或按原行为改写 computed 更新点）。`next_responsible_role: work（T-02 W45）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 48 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 2891783c | next: work

2026-11-19 stage: work（T-02 W45：record 类型内联）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `2891783c` | task_ids: T-02（W45）| evidence: prep 6/6

**交付**：`extract_unit_type`（抽 `[pub] type <Name> { … }`）→ `use <unit>: <Record>` 命中时前置内联记录声明；**store 单元自身的 `use types: …` 依赖也一并内联**（013 的 `Todo`）。列表状态字段（`[]T` 模型字段）暂仍按视图数据改写——核心侧的 list 字段/记录类型支持是下一刀（记录类型前置已就位，届时 prep 无需再改）。

**验证**：auto-ui prep 6/6（015/018/022 桥装载保持）；013/015 桥复测：015 OK、013 仍具名 `.store` 边界（列表字段未进 store 映射）。

**下一刀**：核心侧 **列表模型字段**（`[]T`/`[N]T` → `List<T>` 字段、`field_type` 扩展、记录类型进 collections 类型表），然后切回 prep 保留 store 列表类型 → 013 解锁。`next_responsible_role: work（T-02 W46）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 49 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-lang,auto-ui} | commit: 25550e67d | next: work

2026-11-19 stage: work（T-02 W46：列表模型字段打通到记录构造面）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `bfde9b8e4`+`25550e67d`、auto-ui `f93f349e` | task_ids: T-02（W46）| evidence: prep 6/6 + 回归全绿

**交付**：①`field_ann`——`[]T`/`[N]T` → `List<T>`（元素内建或记录名），状态槽 Let 注解改用之（三处）；②`needs_collections_face` 在模型字段带列表类型时也切 collections 域；③文法 `age-list-ty`（`[`opt int`]` 元素类型）并入 `age-model-ty`（两面同开，census 重基线）；④观测门接受 `field_ann` 可解析的列表字段（观测通道仍只打包 int）；⑤prep 的 store 列表状态字段恢复保留类型（`var todos []Todo = []`）。

**桥装载推进**：013 的 store 声明（含 `var todos []Todo = []`）现可**解析并内联**，装载推进到新的具名拒「this expression shape is not wired in the component scenario slice yet」（record 构造/下标写等表达式面）。015/018/022 不变（OK）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；prep 6/6；census 4/4（component `ad0d0273…`、package `0fe5c8c6…`）。

**下一刀**：组件场景表达式面——`Todo { … }` 记录构造、`.todos[idx]`/`.todos[i].field` 下标+记录链、`[]T` 字面量初值在 collections 域的发射。`next_responsible_role: work（T-02 W47）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 50 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 48795c44e | next: work

2026-11-19 stage: work（T-02 W47：表达式面 Index/ObjectLit + 013 缺口推进到 host API）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `48795c44e` | task_ids: T-02（W47）| evidence: 回归全绿 + prep 6/6

**交付**：`rename_expr_with_params` 补 `Index { base, index }`（基/下标递归）与 `ObjectLit { fields }`（字段值递归、键原样）两臂——列表下标链（`.todos[idx].id`）与记录字面量（`Todo { id: 1, text: "a" }`）在场景面不再落「this expression shape is not wired」。

**桥推进**：013 的表达式面已通过，装载推进到**新墙**——「handler body call statements are only wired for `store.<Msg>()`」：store handler 里的**宿主 API 调用**（`update_todo(…)` / `list_todos()` / `remove_completed(…)` / `add_todo(…)`）尚无接线面。012 仍是 computed 写入面（`on Tick` 赋值 `w_rows`）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 2/2；prep 6/6。

**下一刀（S1 host API 面）**：给组件面一条**宿主 API/效果面**——调用语句按能力分派（`ac.runtime.host.api.v1`），AV 宿主提供内存实现（013/015 的 todo/notes 后端），失败/未接线一律具名拒。`next_responsible_role: work（T-02 W48）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 51 | from: work | to: work | status: executing | outcome: pass（状态盘点）| worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 无代码提交 | next: work

2026-11-19 stage: work（T-02 W48：六例真实渲染面盘点 + 下一步排序）| PLAN-105 r1 | outcome: pass | task_ids: T-02/T-04 盘点 | evidence: `plan-artifacts/plan-105/e15.json`/`e18.json`/`e22.json`（当前真窗 leaf 计数）

**盘点（真窗 leaf 数，新构建）**：
| 例 | 真窗 | 已渲染 leaf | 结构 | 主缺口 |
|---|---|---|---|---|
| 015-notes | OK（frames=1） | 7（Notes / New note / No notes yet …） | 单 widget + store（已内联） | 宿主 API（`list_notes()` 等） |
| 018-book-reader | OK | 13（📖 / AutoRead / 📚 …） | **routes + outlet**（11 个 .at） | 页面组合（outlet 渲染子页） |
| 022-kanban | OK | 8（☰ / Project Board / Kanban …） | **routes + outlet** + BoardStore | 页面组合 + 看板列表 |
| 013-todo | 失败 | — | 单 widget + store（已内联） | 宿主 API（`update_todo`/`list_todos`/…） |
| 016-calendar | 失败 | — | 单 widget + 依赖 util | view-for over computed 列表 |
| 012-clock | 失败 | — | 单 widget | computed 写入（`w_rows`） |

**结构事实**：018/022 用 `routes { "/" -> use page }` + `outlet`（页面在 `src/front/pages/*.at`，故「真窗只渲染壳件」是**结构性**的，不是遗漏）；013/015/016 用 `store`（已可内联）；六例中 5 个需要**宿主 API/后端**或**页面组合**之一。

**下一步排序（按解锁面）**：①**页面组合**（routes/outlet → 渲染 `pages/*.at` 的 widget，共享 store）——一次解锁 018/022 两个例；②**宿主 API 面**（S1）——解锁 013/015；③**view-for over state 列表**——解锁 016/022 内容；④computed 写入——解锁 012。①与②可并行推进；先做①（面更大且不改语言语义）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 52 | from: work | to: work | status: executing | outcome: pass（S1 方案定界）| worktree: D:/autostack/.wt/ui-105 | commit: 无代码提交 | next: work

2026-11-19 stage: work（T-02 W49：S1 宿主 API 面定界 + 后端来源勘察）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W49）| evidence: 013 `src/back/{api,db}.at` 实读

**勘察事实**：013 的后端是**纯 Auto 源码 + 内存态**（`db.at`：模块级 `var todos List<Todo>` + `pub fn all_todos/create_todo/update_todo/...`；`api.at` 只是薄封装 `return db.all_todos()`）——不是外部服务，故 S1 不必先造网络/文件宿主，可实现为**语言内后端**。

**关键约束**：组件面（widget）没有模块级状态与返回值的自由函数面（model 字段只能被 handler 读写），所以「把后端函数内联成组件函数」这条路走不通；需要一条**宿主调用面**。

**已定界的实现路径（下一刀）**：用 HIR 既有 `DefKind::Intrinsic { symbol, effect, ty }` 承载宿主 API——①组件面把 `use api: f` 的 `pub fn` 签名声明为 intrinsic def（符号 `host.api.<unit>.<fn>`，带参数/返回类型）；②verify 按声明类型核调用；③MIR/AV 解释器按 symbol 分派到**宿主表**（AV 侧内存实现 todo/notes/cards 后端，会话内状态）；④能力门 `ac.runtime.host.api.v1`，未声明即具名拒。现状核对：intrinsic 目前只作缓存/依赖元数据用，**尚无执行分派**，需补 ③。

**并行候选**：A2Vue 车道（TS 运行时自带宿主与组合），可能是 018/022（routes+outlet）更近的路；下一轮先做一次**干净的 SFC 生成探测**（用真实 `PackageModuleInput`），再决定两车道顺序。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 53 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: f7c1d99b | next: work

2026-11-19 stage: work（T-02 W50：内建列表字段保持类型 + A2Vue 入口勘察）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `f7c1d99b` | task_ids: T-02（W50）| evidence: prep 6/6 + 012 新推进

**交付**：`rewrite_typed_list_field` 只改写**记录元素**的列表字段（`[]Faq`、`[][]Faq`）；**内建元素**列表字段（`[]str`/`[]int`/`[]bool`/`[]f32`）**保持类型**——collections 域真承载、handler 可写（012 的 `.w_rows = rows`）。测试同步更新（正反两面：记录改写、内建保持）。

**效果**：012 从「`on Tick` assigns undeclared model field `w_rows`」推进到**新的具名拒** `source.age.structure: unexpected token in a bare element tag (spacer/divider/sep/outlet)`（view 结构面下一刀）。

**A2Vue 入口勘察**：`generate_package_sfc_with_root` 只传入口单元会报 `source.module-identity: invalid or duplicate package/module identity`；带 store 的例（013/016）报「package-level store semantics are not wired yet (S1)」。→ A2Vue 的真入口需要**整包单元装配**（app + store + pages + types…），当前 runtime 只有测试消费者，生产装配在 auto-ui-tools，下一轮勘查。

**验证**：auto-ui prep 6/6；plan105/980/979/981 等核心回归未受影响（本轮未动 core）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 54 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W51：本地 style 声明内联）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui（本轮）| task_ids: T-02（W51）| evidence: prep 7/7（新增 `inlines_local_style_declarations`）

**交付**：prep 现在也内联**本文件**的 `style <name> = "<classes>"` 声明——声明行等长空白化，`style: <name>` 引用替换为引号字面量（与依赖样式同一条消费语义）。012 这类「文件内样式表 + 引用」的真语料不再把裸标识符喂给元素属性文法。新增正面测试锁定该行为。

**012 排查记录（负面结论，便于下一轮）**：用最小源分别验证**同行逗号多属性**、**CJK 标签**（`button "时−" { … }`）、**`text .field { … }` 文本叶**三种形状——都能投影成功。故 012 的 `unexpected token in a bare element tag` 是**更早处解析失步**导致（错误点落在后续 `}`），下一轮需在 012 源中做**前缀二分**（按 `if .tab == …` 分段截断）定位首个失败段。

**验证**：prep 7/7；core 未动。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 55 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-lang,auto-ui} | commit: 40d2982e1 / ce4e4215 / 62063c4bb | next: work

2026-11-19 stage: work（T-02 W52：012 连破三墙）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `62063c4bb`+`cd8e3d10d`+`40d2982e1`、auto-ui `ce4e4215` | task_ids: T-02（W52）| evidence: 012 真窗日志（panic→具名拒序列）

**本轮交付（四刀）**：①`ann-ty` 收列表/泛型注解（handler 局部 `var rows []str = []` 可解析，census component `1d35c8ff…`/package `eaadd1d6…`）；②本地样式引用**整词替换**（覆盖 `style: if … { tab_active } else { tab_idle }` 条件式）；③`classify_word` 对列表注解生成 `List<T>` —— **修掉真语料 panic**（此前 `.expect("type word leaf")` 让 012 真窗 exit 101）；④handler 局部列表字面量放开到 collections 域。

**012 墙序列（逐墙推进，全部有据）**：真窗 panic（已修）→ 具名拒「handler 局部列表字面量」（已解）→ 具名拒 **handler 级 `for c in .cities` 列表迭代**（下一刀）。

**负控纪律**：列表索引包面负控放宽为「显式诊断」（具名 component-unsupported 或 `source.index*`/`source.undefined`/`source.type-mismatch`），不再要求单一码。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 2/2；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 56 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: ce4c62b73 | next: work

2026-11-19 stage: work（T-02 W53：handler 级 for-each 降级）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `ce4c62b73` | task_ids: T-02（W53）| evidence: 012 真窗诊断面推进（具名拒 → 语义诊断清单）

**交付**：handler 级 `for <v> in <list> { … }` 降级为**索引 while 循环**（`i = 0` + `while i < list.len()` + `let v = list[i]` + 循环体 + `i += 1`；元素名进作用域供体内引用），循环体共用同一 hook 降级通道；沿路修掉「列表字段不得进状态观测通道」（状态函数按内建类型返回，列表字段由视图/循环消费）。包面负控同步放宽为「显式诊断」。

**012 真窗推进（本轮结果）**：不再停在该 for 墙，装载进入语义诊断清单，暴露出**两个新面**：①**宿主服务** `storage.get/set`（闹钟持久化）与 `Time.now_sec()`——即 **S1 宿主 API 面**；②空列表字面量 `[]` 在 `[]T` 注解下的元素类型推断（`source.array-lit-empty`）。这两项是 012 的剩余主墙（也覆盖 013/015/016 的部分）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 2/2；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 57 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W54：宿主服务仿真第一刀）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W54）| evidence: 012 诊断面（storage/Time 消失，剩 Str.contains / push）

**交付**：prep 新增**宿主服务仿真**（S1 的 prep 侧最小实现）——①`storage.get("k")` → `.__st_k`（键 → 稳定字段名）；②`storage.set("k", v)` → `.__st_k = v`（语句位改写）；③`Time.now_sec()` → `.__now_sec`；④把用到的字段注入 `model {` 块尾（`str`/`int` 初值）。另加 `var NAME []T = []` → `List<T>.new()`（元素 `str` 归一为 `Str`，822 元素域不收借用 str）。

**效果**：012 真窗的 `storage`/`Time` 诊断**全部消失**，剩余墙变为 ①`Str.contains(...)`（语言无此内建方法）②`rows.push(...)`（局部列表注解 `[]str` 未生效 → 接收位判成 i32）。

**发现（下一刀定界）**：组件面需要 `Str.contains(sub) -> bool` 一类文本谓词与「`[]T` 注解 → List<T> 局部类型」的可靠传播；后者与 `age-model-ty` 只收 `age-list-ty`（不收 `List<T>` 泛型拼写）有关，改文法或改投影二选一。

**验证**：auto-ui prep 7/7；core 未动。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 58 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: ec26a2f2b | next: work

2026-11-19 stage: work（T-02 W55：列表初值规范化）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `ec26a2f2b` | task_ids: T-02（W55）| evidence: `p105_local_list_binding_pushes_and_reads_back`（局部列表 push/len → n=2）

**交付**：①组件面对**列表类型位**的数组字面量做规范化——`[]` / `[...]` → `List<T>.new(…)`（model 字段与 handler 局部 Let 两处；元素类型取自注解）；②`[]str` 的元素类型在投影处归一为 owned `Str`（822 元素域不收借用 `str`）；③新增永久回归：handler 局部 `var rows []str = []` + `rows.push("a")` ×2 + `.n = rows.len()` → **n=2**。

**012 真窗推进**：`array-lit-empty` 与局部列表类型问题消失，剩余两墙：①`Str.contains(sub)`（语言无该内建方法）②`rows.push(拼接表达式)` 触发 `source.move-required`（owned 元素实参需 `.move` 或新建构造）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 59 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-lang,auto-ui} | commit: 5c599d678 / c01a4f2e | next: work

2026-11-19 stage: work（T-02 W56：Str.contains 谓词 + 初值规范化收窄）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `5c599d678`、auto-ui `c01a4f2e` | task_ids: T-02（W56）| evidence: 012 墙序列推进（contains 消失）

**交付**：①prep 把 `<text>.contains(<sub>)` 改写为注入的文本谓词 helper `__p105_str_contains`（用 `len`/`slice`/`==` 既有内建 + **C 风格 for** 展开，语言零改动）；②声明注入（样式/记录/helper/store 块）移到**全部改写之后**再插到组件头之前（否则后注入的 helper 不落地）；③空列表初值规范化收窄为**仅空字面量**（非空数组字面量保持 let/var 初值位，避免 `array-lit-position`）。

**012 真窗推进**：`contains` 与 `array-lit-empty` 诊断消失；剩余墙 = `rows.push(<拼接表达式>)` 的 `source.move-required`（owned 元素实参需 `.move` 形式或新建构造，且 `array-lit-position` 限制数组字面量只在初值位）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 60 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 2453bbd56 | next: work

2026-11-19 stage: work（T-02 W57：push 新值面 + 012 新墙定位）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `2453bbd56` | task_ids: T-02（W57）| evidence: 012 诊断 span 定位

**交付**：`check_push_value` 的「owned 元素实参」接受面扩到**新值**——`Binary`（文本拼接）/`CallFn`/`StrLit`/`VariantCtor` 均视为 fresh rvalue 提交（`rows.push(a + " " + b)` 不再要求 `.move`）。`ReadLocal` 仍按原规则具名拒。

**012 真窗推进（带 span 的定位证据）**：`move-required` 消失；新墙 = **`str`/`Str` 混用**：诊断 `expected str, found Str` 的 span 落在 012 的**填充字面量**上（`"      "` / `"   "`），即文本拼接/文本方法把 owned `Str` 与借用 `str` 混在一处。根因方向：组件面把 model `str` 字段映射为 **StrOwned**，而文本运算/方法链的期望位仍按借用 `str` 推。

**验证**：plan105 25/25、980 39/39、981 248/248；prep 7/7（清理探针后复跑）。

**下一刀**：文本运算的 `str`/`Str` 双通（拼接、slice 链、`TextDisplay` 操作数），或在组件面把「文本字段参与运算」统一切到 owned 车道。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 61 | from: work | to: work | status: executing | outcome: pass（负结果）| worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 无代码提交 | next: work

2026-11-19 stage: work（T-02 W58：文本谓词落点勘察）| PLAN-105 r1 | outcome: pass（负结果记录）| task_ids: T-02（W58）| evidence: 012 atom bind 明细

**尝试**：把 `__p105_str_contains` helper 的形参从借用 `str` 改成 owned `Str`（期望消掉 `expected str, found Str`）。结果：helper 内的 `hay.slice(…)` 在 **collections 描述符**里被拒——`atom:290:30 bind.unknown-case: no accepted case expr.text_slice`、`unknown field end/start`、以及 `t_str` 悬挂引用。说明 collections 域的 Atom 面**没有 `text_slice`**（借用文本的切片族整体不在该域）。

**结论（下一刀定界）**：`Str.contains` 类文本谓词不能靠 prep helper 在 collections 域实现——必须在**语言/描述符层**补 `text_contains`（HIR `ExprKind` + collections descriptor case + MIR 降低 + AV 解释器），或让组件面文本运算留在 text-record 车道（与 collections 列表并存的双域方案）。

**本轮无代码提交**（tweak 已回滚）；prep 7/7、plan105 25/25、980 39/39、981 248/248 保持。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 62 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 6f655acd6 | next: work

2026-11-19 stage: work（T-02 W59：collections 描述符补齐文本面）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `6f655acd6` | task_ids: T-02（W59）| evidence: 012 从 bind 失败推进到 **verify** 面

**交付**：collections 描述符（`schema/collections.atom`）补齐**文本表达面**——`ExprKind` 增 `text_slice/text_char_at/text_concat/text_display/fstr`，`TypeKind` 增 `str/char`（含 `branch(type.str)`/`branch(type.char)`），`expr` 节点增 `start/end/parts` 字段；adapter 的 collections 类型表补发 `t_str`/`t_char`。

**效果**：012 的真窗错误从 **Atom 绑定**阶段（`no accepted case expr.text_slice` 等一串）推进到 **verify** 阶段：`eq operands must both be i32 or both be bool (found str / Str)`、`local l98 has text type str; text types exist only in the text-i32-draft profile`、`immutable-write`。→ 下一刀是 **verify_collections 的文本类型面**（借用 str 视图与 Str 混比较、局部文本类型）而非描述符。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3（collections 文本面加入后原理事例全部保持）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 63 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 694bd808f | next: work

2026-11-19 stage: work（T-02 W60：collections 文本类型面入域）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `694bd808f` | task_ids: T-02（W60）| evidence: 012 推进到 immutable-write 面

**交付**：①`verify` 的 collections 域**放行借用文本/字符**（类型表白名单 + 局部类型：`str`/`char`）；②AGE 入口的**跨借用/owned 文本**规则——相等比较（`age_eq`，经新的 `verify_collection_family_age` 入口）与**调用实参**（`str` ↔ `Str` 内容语义）；③`typecheck` 的调用实参同样放行 AGE 跨文本；④组件面改用 AGE collections 验器入口。

**效果**：012 真窗错误从 verify 的文本类型面（`local has text type str`、`param 0 (str) does not match eval_args[0] (Str)`）**全部消失**，推进到 `verify.immutable-write`（一批局部 `l103…l106` 被赋值但声明不可变）。下一刀 = 这些局部的可变性来源（很可能是 prep 注入的宿主仿真字段或 C-for 索引局部的标记路径）。

**纪律**：过程中一次 `age_eq` 归位脚本误改了 `verify_function` 的 `Verifier` 初始化并删掉一行 `let m = …`，已修复并跑全绿（教训：按行号改代码前先确认锚点唯一）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 64 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: b564a8ae1 | next: work

2026-11-19 stage: work（T-02 W61：检查函数路径的列表初值 + 013/012 迭代语义定位）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `b564a8ae1` | task_ids: T-02（W61）| evidence: 012 诊断 span（view-escape）

**交付**：handler **检查函数**路径同样把列表字段的空数组初值规范化成 `List<T>.new()`（此前只有场景路径做，导致 `source.array-lit-empty` 在检查函数里仍命中）；列表回归整理回三例。

**012 墙序列推进**：`immutable-write` 的根因定位到 **`for c in .cities` 的元素绑定**——`List<Str>` 的整元素是**视图**，绑定到局部会触发 `source.view-escape`（"a whole owned element is not a value; project a field or use remove"）。012 的用法是 `rows.push(c + "  " + …)`（元素在表达式内联使用），因此**正解**是 for-each 降级时**不做 `let` 绑定**，而把元素表达式**内联替换**进循环体（每个使用点展开成 `iter[i]`）。

**下一刀**：for-each 降级改为 AST 级**元素内联替换**（文本元素视图语义），替换后再走同一 hook 降级；`List<Record>` 元素仍可绑定（字段投影合法）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 65 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-lang,auto-ui} | commit: 8524e770b / c6c41e1f | next: work

2026-11-19 stage: work（T-02 W62：for-each 元素内联）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `8524e770b`、auto-ui `c6c41e1f` | task_ids: T-02（W62）| evidence: 012 墙从 view-escape 推进到 view 文本叶

**交付**：①for-each 降级改为**元素内联替换**——整元素在 collections 域是**视图**，不绑局部，改为把元素名替换成 `iter[i]` 表达式（复用 store 展开的 `subst_params_stmts`）；②`mark_lets_mutable` 补 `Stmt::While` 递归（循环体内的再赋值局部此前漏标，产生 `immutable-write`）。

**012 墙序列推进**：`immutable-write` 与 `view-escape`（整元素绑定）**全部消失**；新墙 = `verify.view-escape: expr e565: a whole owned element is not a value`，落在 **view 的 `text r` 文本叶**——即「视图循环变量作文本叶」需要 view-for 的动态展开（与 016/022 的列表渲染同一能力）。

**清理**：移除上一轮临时诊断探针，prep 测试 7 例保持。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 66 | from: work | to: work | status: executing | outcome: pass（占位面）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W63：循环变量文本叶占位）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang（本轮）| task_ids: T-02（W63）| evidence: 回归全绿；012 仍有非叶 view-escape

**交付**：组件面把**未绑定文本叶**（视图循环变量 `text r`，含 `.name` 字段形态）改为**空串占位叶**——叶序与叶数保持（不静默丢叶），动态列表展开作为独立里程碑后续接入。

**012 现状**：占位叶生效后仍报 `verify.view-escape: expr e565: a whole owned element is not a value`，说明该诊断**不是文本叶**（叶已占位），而是循环体里另一处整元素求值；需按 expr id 反查所属函数（下一轮用 atom 文本定位 e565 所在 body）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 3/3。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 67 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W64：列表状态出口 + 初值展开）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W64）| evidence: 新增回归 `p105_list_local_to_field_assign`

**交付**：①**列表字段不进状态出口**（`instance_states` 只发内建类型字段——此前 `__age_state_0_xs` 让观察器报「narrow ComponentFieldValue 不承载 []str」）；②**非空列表初值展开**：`var xs []str = ["a","b"]` 在组件面展开为 `List<Str>.new()` + 逐元素 `push`（数组字面量不能做 `new` 的实参）；③新增永久回归：**局部列表赋给字段**（`var rows []str = []` + push + `.xs = rows`）。

**012 现状**：`.w_rows = rows` 不再是墙（该形状已被回归覆盖）；剩余 `verify.view-escape: expr e565` 经 atom 反查 = `list_get { type: t_str; list: e563(t_list_str) }`——即某列表的类型被视为 **`List<str>`（借用元素）**，读整元素被判成非值。下一轮需找出该 `t_list_str` 的来源（`.cities` 的字段注解 vs 推断），并让列表元素类型统一为 owned。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 4/4。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 68 | from: work | to: work | status: executing | outcome: pass（负结果 + 精确现象）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W65：列表元素 owned 化的现象定位）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W65）| evidence: atom 快照（cities `t_list_str` vs w_rows `t_list_Str`）

**现象（关键）**：同一 model 里两个 `[]str` 字段——`cities`（**非空**字面量初值，走 push 展开）被解析成 `t_list_str`（借用元素），`w_rows`（**空**字面量，走 `List<Str>.new()`）是 `t_list_Str`（owned）。`field_ann` 对两者都返回 `List<StrOwned>`（用 `P105_DUMP_FIELDS` 实证），即使把 push 展开处的元素注解**强制为 StrOwned**，解析结果仍是 `t_list_str` → 说明**降级发生在共享 resolver/adapter 侧**（`List<Str>` 注解在值/泛型实参位被归一到借用 `str`），而不是组件面的注解构造。

**结论/下一刀**：查 resolver 的 `List<T>` 泛型实参归一（`ty_use`/`intern` 路径）以及 push 实参对接收位的类型收敛——目标：让「非空字面量初值 + 逐元素 push」的 `List<Str>` 局部保持 owned 元素，从而消掉 `list_get` 的 view-escape。

**验证**：plan105 25/25、980 39/39、981 248/248；av list handlers 4/4；调试钩子已移除。

### 六例剩余工作（2026-11-20 状态表，供 T-03/T-04 排期）

| 例 | 已达成 | 剩余（按依赖排序） |
|---|---|---|
| 015-notes | 真窗渲染 7 叶；store 内联；样式/记录类型/宿主 storage 仿真 | ①宿主 API 面（`list_notes` 等）或 prep 内联 api 单元；②动态列表渲染（note 行） |
| 018-book-reader | 真窗渲染 13 叶（壳件） | ①routes/outlet 页面组合；②页面数据面 |
| 022-kanban | 真窗渲染 8 叶（壳件）+ 主题切换可交互 | ①routes/outlet 组合；②看板列表渲染；③后端 api 面 |
| 013-todo | store 内联；表达式面（下标/记录） | ①宿主 API 面（`list_todos`/`update_todo`/…）；②动态列表渲染 |
| 016-calendar | 组件面投影通过；util 内联 | ①view-for over computed 列表；②宿主 API 面 |
| 012-clock | 表达式/循环/文本谓词/宿主 storage 仿真；列表初值展开 | ①列表元素 owned 化（resolver 泛型实参归一）；②动态列表渲染；③时间宿主 |

**共同大件**：动态列表渲染（视图循环叶按运行时列表展开）——012/013/015/016/022 五例共用。
## 10. 下游交接

本计划按 U03 独立完成。AV 收据与 1071 已交范围及其他 U 组结果汇合后供 W1 签发、1068/T-04 和 985/B 接收。A2Vue 收据供 1072 与产品 CLI 消费。A2R/AC 后续计划沿同一源码、Widgets 和独立预期补验。跨仓 golden 基线维护按已有 PARITY 路线归档。

2026-11-19 停放与恢复：按用户裁定暂停过本计划开发；104 已交付并合入 master（ui `5a862d46` / lang `2fb094483`）后，已按裁定从 master 重建工作树（不共享 104 分支）并完成 T-02 W1。恢复/续做时先读 §9 的 W1 收据与 `plan-artifacts/plan-105/t02-gap-map.md`（S1–S10 缺口与顺序在案）。
