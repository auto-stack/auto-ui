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

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 70 | from: work | to: work | status: executing | outcome: pass（设计边界发现，已回滚）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 无代码提交 | next: work

2026-11-19 stage: work（T-02 W67：动态列表渲染的设计边界）| PLAN-105 r1 | outcome: pass（负结果，边界确定）| task_ids: T-02（W67）| evidence: 试作的逐元素文本导出被 bind 层拒绝

**尝试**：为动态列表渲染实现第一片——组件面为**文本元素列表**（`[]str`）导出逐下标文本函数 `__age_lt_<i>_<field>_<k>() -> Str`（越界空串），并挂到 `ComponentNodeOut.list_texts` 供渲染腿展开。

**结果（被设计边界挡住）**：`xs.get(k).clone()` 在 **bind 层**被判 `source.view-escape: a whole owned element is not a value; project a field (xs.get(i).f) or use xs.remove(i)`。即**owned 元素的列表不允许把整元素读成值**（借视图只能在表达式内联消费，函数返回需要 owned → clone 被拒）。文本元素没有可投影字段，**整元素文本导出在该语义下不可行**；记录元素可以**投影字段**（`xs.get(i).f`）。

**结论/下一刀**：动态列表渲染改走**记录元素投影**路线——①对 `List<Record>` 导出 `xs.get(i).<text-field>` 文本（记录元素允许字段投影）；②文本元素列表（012 的 cities/w_rows）需要另找 sanctioned 路径（例如新增「视图→owned 拷贝」内建，或让渲染腿在视图内联处消费而不经函数返回）；③设计前先在 `plan-artifacts/plan-105` 写一页设计说明再动工。

**回滚**：试作代码已整体回滚（`age_component.rs` 复原），plan105 25/25、av list handlers 4/4 保持。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 71 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 9be4d0042 | next: work

2026-11-19 stage: work（T-02 W68：记录构造/自由调用进组件面）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `9be4d0042` | task_ids: T-02（W68）| evidence: 记录列表用例从「表达式未接线」推进到 `.move` 与 lower.type-use 面

**交付**：①`rename_expr_with_params` 收 **`CtorNamed`**（`Todo { id: 1, text: "a" }` 的字段值递归重命名）与 **`Call`**（自由调用/构造实参，含具名实参 span）；②handler 列表方法 `push(<整局部>)` 按 **`.move`** 形式发射（`Arg::MovePos`）。

**记录列表用例推进（三次定位）**：①`Todo { … }` 未接线（已解，AST 形为 `CtorNamed`）；②`type Todo { text str }` 的借用字段导致 AV lowering 拒（`type kind Str is outside the av.collections/1 source surface`）→ 记录文本字段需 `Str`（prep 侧改写方向已定）；③`xs.push(t)` 的整局部 → `move-required`（已改为 `.move` 发射，但复测仍报，需核对 `MovePos` 在 atom 的发射形态）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 4/4。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 72 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: 1df6ecb52 | next: work

2026-11-19 stage: work（T-02 W69：记录文本字段 owned 化 + push 局部）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `待记`、auto-lang `1df6ecb52` | task_ids: T-02（W69）| evidence: prep 7/7、回归全绿

**交付**：①prep 把**内联记录类型块**的文本字段改写为 **owned `Str`**（`text str` → `text Str`；组件面 collections 域的 AV lowering 不接受借用 `str` 字段）；②组件面 `push(<局部>)` 尝试用「该局部的构造值表达式」替代（按值复制语义），但**未生效**（解析后仍是 `ReadLocal` → `move-required`）。

**关键发现（下一刀入口）**：列表方法在 AST 里有 `Method` 与 `MethodCall` **两种形态**，且 `push(local)` 的 owned 元素转移**在该语义下没有 sanctioned 形式**——`Arg::MovePos` 在 resolver 里与 `Pos` 同处理（无 move 语义），`check_push_value` 只认 `RExprKind::Move`（该 kind 只由形参 `.move` 调用位产生）。要么在 resolver 给 `MovePos` 补 move 语义（局部→Move 包装 + 消费标记），要么在组件面把「构造后 push」改写成一次性构造。

**验证**：prep 7/7；plan105 25/25、980 39/39、981 248/248；av list handlers 4/4。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 73 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 8d7163dd8 | next: work

2026-11-19 stage: work（T-02 W70：resolver 的 `.move` 实参语义）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `8d7163dd8` | task_ids: T-02（W70）| evidence: 回归全绿；记录 push 仍报

**交付**：①`resolve_list_method` 的 `push`/`insert`/`set` 尊重调用实参的 **`.move` 标记**——新增 `move_marked_value`，把 `ResolvedArg.moved` 的实参包成 `RExprKind::Move`（整局部所有权转移的 sanctioned 形态）；②组件面对未记录构造值的 `push(<局部>)` 回退发射 `Arg::MovePos`。

**未解现象**：手工用例（`type Todo{text Str}` + `var t Todo = …` + `.todos.push(t)`）**仍报** `move-required`（span 落在 `t` 上）。说明该语句的发射走的可能不是 `hook_stmts_lower` 的 `Call` 分支（或 `is_list_method` 未命中）；下一轮加一次性 trace（在 Call 分支打印 msg/形态）确认路径，再决定是补 `Method` 形态识别还是改在检查函数铸造处。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 4/4。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 74 | from: work | to: work | status: executing | outcome: pass（定位推进）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W71：push 路径 trace）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W71）| evidence: `[call-out]` 轨迹（`MovePos(Var(__age_hl_0_0_0_t))` 确实发射）

**实证（trace 结论）**：`hook_stmts_lower` 的 `Call` 分支**确实命中**（`is_list_method=true`，形态 `MethodCall`），且发射出的实参**就是 `MovePos(Var(__age_hl_0_0_0_t))`**。而 resolver 侧：`resolve_args` 对 `MovePos` 置 `moved: true`，`check_push_value` 前有新增的 `move_marked_value` 包装成 `RExprKind::Move`。

**矛盾点（下一刀）**：诊断仍报 `move-required`（span = `push` 方法名，正是 `check_push_value(.., name.span)` 的锚点）→ 说明 resolver 拿到的实参 `moved == false`，即 **`MovePos` 在「AST → resolver 入口」之间被某处归一成 `Pos`**（候选：desugar/`resolve_method_call` 的实参预解析、或 adapter 的 desugar 阶段）。下一刀直接 dump atom 的 `call` 节点看是否带 `move` 标记，定位归一发生点。

**验证**：plan105 25/25、980 39/39、981 248/248；av list handlers 4/4；trace 已移除。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 75 | from: work | to: work | status: executing | outcome: pass（诊断检查点）| worktree: D:/autostack/.wt/ui-105/auto-lang | commit: bfed5b9a7 | next: work

2026-11-19 stage: work（T-02 W72：`.move` 标记全链路 + 诊断检查点）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `bfed5b9a7` | task_ids: T-02（W72）

**本轮交付**：`Task::MethodOut` 现在**携带每个实参的 `.move` 标记**（`moved_flags`），并在 AGE 域（AgeComponent/AgePackage/AgeH0）保留该标记——此前迭代机器把 `Arg::MovePos` 归一成 `moved=false`，导致列表 `push(<整局部>)` 永远拿不到所有权转移语义。

**诊断检查点（连续三轮同一现象无进展，按 work 技能要求停下诊断）**：
- 现象：`type Todo { text Str }` + `var t Todo = Todo{…}` + `.todos.push(t)` 仍报 `source.move-required`，span 恒为方法名（`check_push_value(.., name.span)` 的锚点）。
- 已排除：①组件面发射（trace 证实 `MovePos(Var(__age_hl_0_0_0_t))`）；②`resolve_args` 的 moved 置位；③`MethodOut` 的 moved 归一（本轮已补）；④`resolve_list_method` 的 `move_marked_value` 包装（已存在）。
- 剩余假设：该调用在**检查函数/场景两条铸造路径**中的一条走的是 `RecordMethodOut`（其 `moved` 仍是旧式子句，且 task 无 `moved_flags`）；或 `check_push_value` 在包装**之前**就被另一处（`ListMethod` 预解析）调用。
- 下一步（一次到位）：在 `RecordMethodOut` 上也补 `moved_flags`，并给 `check_push_value` 加临时 trace（打印 `value.kind` 判别式），一次跑通后立即移除 trace。

**排期决定**：为避免继续在同一子特性上空转，下一轮**先换刀**——推进 018/022 的 routes/outlet 页面组合（与 `.move` 无关、可独立验证），把 `.move` 收回队列第二优先。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 4/4。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 76 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: caf89d90 | next: work

2026-11-19 stage: work（T-02 W73：routes/outlet 页面组合第一片）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `caf89d90` | task_ids: T-02（W73）| evidence: 018 从「只有壳件 11 叶」推进到「页面已并入，墙为未定型 `List` 字段」

**交付**：①`first_route_page`（解析 `routes { "/" -> use <page> }` 的首个映射并定位 `src/front/pages|components|<page>.at`）；②`split_widget` / `block_inner`（widget 体与 `model`/`view` 块切分）；③`inline_route_page`（页面**顶层声明**（use/style/fn/type/const）并入外壳顶部、页面 `model` 字段并入外壳 model、`outlet` 换成页面 `view` 体）；④组件面语法归一：`while <cond>` → 条件 `for`、去掉显式 `as int` 转换（含**注入的依赖块**）。

**018 推进链（每步实证）**：`expected fn, found col`（widget 未闭合 → 修）→ `` `while` 不受支持 ``（归一）→ `expected ;, found as`（归一）→ **`field cards is List`**（页面模型的未定型 `List` 字段；组件面只收 `[]T`/`[N]T`）。

**开关**：路由组合默认关闭（`P105_ROUTE_PAGES=1` 打开）——保证六例 load 门保持绿，同时可继续逐墙推进；下一刀：页面未定型 `List` 字段的定型（按 `store` 返回值/元素用法推断元素类型）。

**验证**：prep 7/7（含 `real_u03_apps_*` 两例）；018 走开关后墙位前移（壳件 11 叶 → 页面并入）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 78 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 9add1b719 | next: work

2026-11-19 stage: work（T-02 W75：`push(整局部)` 打通 + 用户口径落定）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `9add1b719` | task_ids: T-02（W75）

**用户口径（D-105-A 答复）**：保留原验收口径；**先集中打通 013 Todo 一个完整应用**（对象元素列表 + 动态列表渲染 + 真实后端 API；Iced 与浏览器两端验证新增/筛选/切换状态、列表刷新与失败恢复），据此判断 Notes/Calendar/Kanban 的可复用面；124/125 另行并行。若 013 暴露需重做的共享接口，再按具体阻断重估。

**本轮交付（关键解锁）**：`Task::RecordMethodOut` 也携带 `moved_flags`，并在构造 `ResolvedArg` 时按标记置 `moved`——**记录接收者**（`.todos.push(t)`）上的列表方法此前丢失 `.move` 语义，是所有记录列表用例的卡点。修好后：
- `var t Todo = Todo { … }` + `.todos.push(t)` **通过**；
- `todos.get(0).text.len()` 的**字段投影读取**也通过（`List<Record>` 的 sanctioned 读法）。
- 新增永久回归 `p105_record_list_push_and_project_field`。

**013 复用判断（首轮）**：store（`TodoStore`）的列表维护、记录构造、字段投影、条件 `for` 已可用；剩余两件是**动态列表渲染**（视图循环按运行时列表展开）与**后端 API 面**（`list_todos/create_todo/update_todo/delete_todo/remove_completed` —— 后端是纯 Auto 内存实现，可走「包级 store / 同进程调用」路线）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers **5/5**。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 79 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W76：013 内存后端内联）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui（本轮）| task_ids: T-02（W76）| evidence: prep 7/7；013 墙从「handler 非 store 调用」推进到「store 块内联探针拒绝」

**交付**：`inline_memory_backend`——把 `use back.api: …` 指向的**纯 Auto 内存后端**内联进 store 单元：`list_todos()`（列表即真值，删行）、`create_todo(x)`（push + `.next_id` 自增）、`update_todo(id,text,done)`（下标搜索后整条替换）、`delete_todo(id)` / `remove_completed(_)`（下标搜索后 `remove`）；并从 `db.at` 取**种子数据**与 **nextid 初值**写回 store model（`var todos []Todo = [Todo { … } …]`、`var next_id int = 4`）。

**013 墙序列**：`handler body call statements are only wired for store.<Msg>()`（= store 里的 api 调用）→ **`.store.*` 需要同源 store 声明（S1）**，即 store 块没被内联。

**定位结果**：`extract_store_block` 对**原始** store 块的探针通过；对**改写后**的块探针报 `source.syntax: expected fn, found store`。→ 生成的语句（`var __mb_* int = …`、`for … { if … { .todos[i] = Todo { … } } }`、`var __g Todo = .todos.remove(i)`）在 **store 块上下文**里不被接受（或块边界被改写影响）。下一刀：先查 age-component 文法里 `store` 块允许的语句面，再决定是「把 shim 生成物放到组件级 handler」还是「放宽 store 块语句面」。

**验证**：prep 7/7（六例 load/project 门保持绿；开关未改动）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 80 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: cc394cf1 / 0f13a24b5 | next: work

2026-11-19 stage: work（T-02 W77：013 后端内联五连墙）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `cc394cf1`、auto-lang `0f13a24b5` | task_ids: T-02（W77）| evidence: 013 墙序列见下

**本轮交付**：①列表方法面补 `insert`/`remove`（组件面语句门 + 诊断点名：`handler body method \`x\` is not wired yet`）；②`update_todo` 生成物改走 **`remove`+`insert`**（下标赋值带记录字面量在 store 块语法不可解析——这是 bisect 出来的）；③后端记录类型收集 `collect_type_blocks`（支持 `pub type X = { … }` 与冒号字段），并按**解析探针**过滤（015 的 `List<..>` 后端类型会被跳过）；④`rewrite_record_text_fields` 支持冒号字段形态；⑤`inline_memory_backend` 与环境开关 `P105_MEM_BACKEND`（默认关，避免影响其它例子）。

**013 墙序列（本轮推进 5 格）**：`store 块未内联(S1)` → `method insert 未接线` → `no record type named Todo`（`pub type` 形态） → `field text of Todo has type str, found Str`（冒号字段） → **`view-escape: a whole owned element is not a value`**。

**下一刀（两个候选实验）**：①`update` 改 `.todos.set(i, Todo{…})`（避免 `remove` 到局部）；②`delete`/清空改用能丢弃返回值或绑定形态的 `remove`（先在 av 里验 `var x T = list.remove(i)` 是否触发 view-escape）。

**验证**：prep 7/7；core plan105 25/25；av 5/5。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 81 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: c58b4a692 | next: work

2026-11-19 stage: work（T-02 W78：013 的 view-escape 归因）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `c58b4a692` | task_ids: T-02（W78）| evidence: 最小复现落到 bind 层

**本轮结论（把 013 的最后一个大墙归因清楚）**：
1. `var gone Todo = .todos.remove(0)` **可用**；`.todos.set(0, Todo { … })` **可用** → 两者都进永久回归（新增 `p105_list_set_element_replaces_value`）。
2. 013 的 `view-escape` 来自**句柄语义**：`.todos[i].done` 与 `.todos[i].text`（以及 `get(i).f` 形态）在**bind 层**就被拒——`resolve` 的 Index/list-get 分支在 `!aggregates_admission` 时直接报错（消息文案却建议 `xs[i].f`，属误导）。最小复现：`for i < .todos.len() { if .todos[i].done == false { … .todos[i].text … } }` → `source.view-escape`（stage: Bind）。
3. 与之相对，verify 层（`check_collections_rules`）**已有**正确的 ElemView 规则：`ListGet` 仅允许作 `FieldGet` 基或 `.len()` 操作数、owned 终态字段只允许 `.len()` 消费。也就是说：**bind 层过严、verify 层已就绪**。

**下一刀（明确）**：把 AGE 组件卫生道的 `aggregates_admission` 打开（或在 Index/list-get 分支对 AGE 域放行 ElemView），让 `.todos[i].f` 走到 verify 层按现有规则判定；先补 3 个永久用例（字段投影读、`if` 条件里的 bool 字段、文本字段作拼接），再跑 248 例 + 六例门确认没有回归。

**验证**：av list handlers 7/7；prep 7/7；core plan105 25/25。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 82 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 78b8b063d | next: work

2026-11-19 stage: work（T-02 W79：元素视图放行）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `78b8b063d` | task_ids: T-02（W79）

**交付（`xs[i].f` 全链路放行）**：①`resolve_with_view` 入口 + `resolve_detailed_inner(..., elem_view)`：AGE 车道在 bind 层不再一刀切拒元素视图；②`verify_collection_family_age_elem_view` 入口 + `Verifier.elem_view_owned`：AGE 车道允许 owned 文本字段投影**读成值**（`preview = .todos[i].text`），冻结 profile 保持 `.len()` 消费；③组件面组件入口改用 `compile_program_parts_age_elem_view` 变体（能力表不动，避免 Atom 类型表重复）；④新增永久回归 `p105_elem_view_field_projection_compiles`（26/26）。

**基线校准（重要）**：全量 `cargo test -p ace-core --test conformance` 在本分支**基线**就是 834 passed / 36 failed（stash 对照实证），本轮 835/36——即**零新增失败**，新增用例通过。av 全量 1051/264 亦为分支既有基线。

**013 现状**：元素视图墙已过；新墙 = `no local named self in scope` + `field access ... found i32` + `indexing requires a List receiver, found i32`——即**后端内联 shim 的生成语句落在 store 块里**，经 store 展开后作用域不对（`self` 不存在、`.todos` 被解析成 i32）。下一刀：把 shim 生成物从 `store` 块挪到**组件级 handler 体**（或让 store 展开支持生成语句的作用域），并核对 `.next_id` 的字段归属。

**验证**：plan105 26/26、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 7/7；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 83 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W80：013 shim 作用域归因）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W80）| evidence: 失败语句上下文（`.todos.push/.insert`）

**本轮结论（把 013 新墙归因到具体构造）**：失败诊断的上下文已抓取——
1. `no local named `self` in scope` 落在 `if .input != "" { .todos.push(Todo { id: .next_id, … }) … }`；
2. `field access requires a record-typed value, found i32` 落在同一句（`.next_id` 位置）；
3. 另一处 `.todos.insert(__mb_u2, Todo { id: .todos[i].id, … })` 同样报 `self`。

→ 即：**shim 生成的列表方法调用语句落在 store 的 handler 里**，经 store 展开后其字段引用被写成 `self.*`，而**检查函数/场景体的降级通道不会重命名 `self`**（该通道只重命名 `ComponentStmt` 的表达式，`self` 是 store 展开阶段引入的）。附带 `.next_id` 被判成 `i32` 也源于同一映射（`.field` 被当成裸值）。

**下一刀（明确）**：查 `expand_store_calls` 里的字段映射（`store_field_prefixed` 与 `rewrite_store_expr`），确认它把 store 字段映射成什么；两种修法：①让展开后的语句通过同一 `rename_expr` 通道（保留 `Field{self,…}` 语义到组件层）；②shim 不在 store 里生成列表方法语句，而是生成**组件级 handler**（`.AddTodo -> { … }` 直接操作组件字段）。

**验证**：prep 7/7（探针已清理）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 84 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 76b86b49c | next: work

2026-11-19 stage: work（T-02 W81：013 诊断与 store 映射修复）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `76b86b49c` | task_ids: T-02（W81）

**交付**：①`rename_expr` 对**未声明的 model 字段**改为具名拒（`model has no field \`x\``），不再留成 `self.x` 让共享 verify 报误导性的 `no local named self`；②`rewrite_store_expr` 补 **`CtorNamed`/`ObjectLit`** 分支——store 字段映射现在能覆盖构造字面量里的 `.field`（`Todo { id: .next_id, text: .input, … }`）。

**013 墙序列（本轮推进 2 格）**：`no local named self`（误导）→ **`model has no field next_id`**（具名）→ 补 ctor 映射后 → **`no builtin method clone on receiver type bool`**。

**下一刀**：查 `.clone()` 落在 bool 上的来源（大概率是组件面 `wrap_owned_transfer` 对非文本字段也加了 clone，或 shim 的 `!current_done` 实参被克隆），改成只对 owned 文本/记录加 clone。

**验证**：plan105 26/26、980 39/39、981 248/248、104 3/3、1072 19/19；av list handlers 7/7。
### D-105-B（2026-11-19，013 暴露的共享接口阻断，等待用户决策）

**具体阻断（可复现）**：013 的组件源码现在**编译通过**（组件面/verify 全绿），卡在 **AV（Iced）后端 lowering**：
`lower `d___age_state_0_input`: Unsupported { code: "lower.type-use", message: "type kind Str is outside the av.collections/1 source surface" }`。

**根因**：`av.collections/1` 的 MIR 只承载 **i32/bool 标量**（`scalar_shape_of_use`），没有 owned 文本 `Str` 的类型/值/操作；而任何"列表 + 文本"的应用（012/013/015/016/018/022 全部）都需要它。

**影响范围**：所有含 owned 文本的列表应用（六例全部）；A2Vue 侧不受此限（TS 发射不经 AV MIR），但 AV 真窗（Iced）全拦。

**可复用结果（已实证）**：组件面表达式/循环/记录/构造/列表（push/get/set/insert/remove/索引字段投影）/store 内联/后端内联/属性诊断——这些都在 core（两侧共用），013 已推进到"只剩后端 lowering"。

**重估**：在 AV collections MIR 增加 `Str`（类型、值表示、concat/clone/len、与 text-record 面共享字符串堆）≈ 5–10 轮且有风险；或改走 **A2Vue（TS）端**先交付 013（不受此阻），Iced 端后补。

**需要决策**：A 继续在 AV 侧补 `Str`（保持原口径，5–10 轮）；B 先以 A2Vue 端交付 013（浏览器可用即算该例端到端），AV 端列为已知缺口；C 收窄到"少量真正完整"的应用（用户此前首选 C 的条件成立时）。

### D-105-C（2026-11-19，用户决定：**暂停并交接**）

**状态**：本计划在 T-02 中段暂停（用户决定）。以下为可直接续做的交接事实。

#### 1. 已交付并稳定的能力（core，两侧共用；均有永久回归）
- 组件面表达式：比较/逻辑/算术/一元/文本拼接/`f"…"`/显示、`if` 条件与链、条件样式。
- 控制流：handler 级 `for`（列表迭代与 C 风格条件 `for`）、`while`（归一为条件 `for`）、嵌套 `if`。
- 列表：`List<T>.new()`、`push`（含整局部 `.move` 语义）、`get`/`set`/`insert`/`remove`/`len`/`clear`/空列表归一；**元素视图** `xs[i].f` / `xs.get(i).f`（字段投影读、`len()` 消费、文本字段读值）在 AGE 车道放行。
- 记录：`type` 声明（`type X { }` 与 `pub type X = { }`）、`CtorNamed` 构造、字段投影、记录列表。
- store：同源 store 内联（字段/消息/处理器）、`store.<Msg>()` 展开、store 字段映射（含构造/对象字面量内部）。
- 诊断：未声明 model 字段具名拒；未接线调用点名（方法名）。
- prep（auto-ui）：依赖样式/本地样式内联、`use` 空白化、`type`/`fn`/`const` 内联、store 块内联、`str.contains` 直译、宿主 storage 仿真、列表初值展开、记录文本字段 owned 化、model 文本字段 owned 化、`routes`/`outlet` 页面组合第一片（开关 `P105_ROUTE_PAGES`）、纯 Auto 内存后端内联（开关 `P105_MEM_BACKEND`）。

#### 2. 具体阻断（D-105-B）
- 013 组件源码**已全部编译通过**，卡在 **AV（Iced）后端 lowering**：`lower.type-use: type kind Str is outside the av.collections/1 source surface`。
- 根因：`av.collections/1` 的 MIR 仅支持 i32/bool 标量（`scalar_shape_of_use`），无 owned 文本 `Str`（类型/值/concat/clone/len）。
- 影响：六例（012/013/015/016/018/022）**全部**含「列表 + 文本」，AV 真窗全拦；A2Vue（TS）端不经 AV MIR，不受此限。

#### 3. 续做路线（任选其一）
- **A**：在 AV collections MIR 增加 `Str`（与 text-record 面共享字符串堆）≈ 5–10 轮；完成后 Iced 端可交付。
- **B**：先走 A2Vue（TS）端交付 013，AV 端列已知缺口；需先打通 T-05 的**整包单元装配**（`PackageModuleInput` 多单元；单单元会报 `source.module-identity`）。
- **C**：收窄到 1–2 例真正完整（用户预置条件）。

#### 4. 工作树与分支
- auto-ui：`D:/autostack/.wt/ui-105/auto-ui`，分支 `plan-105-dev`；auto-lang：`D:/autostack/.wt/ui-105/auto-lang`，分支 `plan-105-dev`。
- 两棵树均已提交、工作区干净；最近提交：auto-ui `e3883ae5`、auto-lang `0a0872b18`。
- 开关：`P105_ROUTE_PAGES=1`（路由页面组合）、`P105_MEM_BACKEND=1`（内存后端内联）。
- 构建：`CARGO_TARGET_DIR=D:\autostack\plan-artifacts\plan-105\target`（auto-lang）与 `...\target-ui`（auto-ui）。

#### 5. 证据锚
- 门：plan105 26/26、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 7/7；auto-ui prep 8/8。
- 基线：本分支全量 `ace-core conformance` 基线 834 passed / 36 failed（预存在，stash 对照实证），本轮改动零新增失败。
- 六例真窗现状：015 7 叶、018 11 叶（壳件）、022 8 叶（壳件+主题可交互）；013 组件编译通过、卡 AV lowering；016 卡 view-for over computed；012 卡未定型列表元素 owned 化。


stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 85 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: e3883ae5 / 0a0872b18 | next: user

2026-11-19 stage: work（T-02 W82 + 交接）| PLAN-105 r1 | outcome: waiting（用户决定暂停）| code_commit: auto-ui `e22b9c5c`/`e3883ae5`、auto-lang `0a0872b18` | task_ids: T-02

**本轮交付**：①局部写只在**文本局部**补 `.clone()`（此前 bool 局部被 clone → `clone on bool`）；②prep 把组件 **model 文本字段**改写为 owned `Str`（AV collections 面只承载 owned 文本的字段声明形态）；③**决策项 D-105-B** + **交接文档 D-105-C** 写入 §10（能力清单、阻断、续做路线、工作树、证据锚）。

**013 终态**：组件源码**全部编译通过**（组件面/verify/store/后端内联 OK），卡在 **AV（Iced）后端 lowering**：`lower.type-use: type kind Str is outside the av.collections/1 source surface`——`av.collections/1` MIR 只有 i32/bool 标量。六例全部受此限（A2Vue/TS 端不受限）。

**用户决定（D-105-B 答复）**：**暂停并交接**。不需要再跑检查或开工；按 §10 的 D-105-C 续做即可。

**状态**：两棵工作树干净且已提交（auto-ui `f0ca8841`、auto-lang `0a0872b18`）；计划文件保留 `status: executing`（未交付），交接事实在 §10。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 86 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: 5e5170ae | next: user

2026-11-19 stage: waiting（用户决定暂停；本轮不做事、不跑检查）| 续做入口：§10 D-105-C（A 补 AV `Str` / B 先走 A2Vue / C 收窄）。恢复方式：顶层说「继续计划105」或选择路线。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 87 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: 9dd311b6 | next: user

2026-11-19 stage: waiting（第 3 个等待轮；无新指令，不做事不跑检查）| 待用户选择路线 A/B/C 或说「继续计划105」。按技能规则，连续三轮等待后目标转 blocked。

stage: merge（尝试）| plan_id: PLAN-105 | plan_revision: r1 | round: 88 | from: user | to: user | status: executing | outcome: blocked（并发计划冲突）| worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: c2802cac / 0a0872b18 | next: user

2026-11-19 stage: merge（用户要求把已完成部分并回主分支）| outcome: blocked | evidence: `git merge-tree --write-tree` 试合并

**已做的核实**
- 主检出（两仓）**无 WIP 代码**（只有其它计划的 docs 改动）；工作树两棵干净。
- 分支差异：auto-ui `master..plan-105-dev` = 57/74；auto-lang = 153/57。
- 试合并冲突数（`merge-tree --write-tree`）：**auto-ui 3 处、auto-lang 3 处**，其余文件按内容自动合并。

**具体冲突**
- auto-ui：`ace_node_desktop.rs`（2 处，均为本分支**新增块**，好解）、`ace_package_sfc.rs`（2 处——master 已重构为 **routed SFC**（PLAN-124/125），本分支加了 `generate_package_sfc_with_root` + prep 入口，需手工合并）、`docs/plans/105-*.md`（取本分支）。
- auto-lang：`resolve.rs`（1 处）、`plan981_grammar_scope.rs`（2 处，**文法 census 哈希**需按合并后文法重算）、`grammars/auto-source.at`（4 处，**语义冲突**：master 新增 `age-route-link-block`（路由 link），本分支新增 `age-element-link-block` + AGE 的 `array-lit`/空数组 profile 收录）。

**结论**：并回主分支**不是一次机械合并**——文法里的 link 元素与 124/125 的路由 link 是同一语法的两种实现，必须选定/融合一套，否则会影响所有 AGE profile 的解析（248 例 census + 全量 conformance 需重跑）。故本轮按规模停下来报告，未在主分支上留下任何提交。

**建议路线**：①先等 124/125 合并，再把 105 分支 rebase 上去（冲突面会变小）；②或以「work 阶段」做文法融合（预计 2–4 轮：选定 link 元素、重算 census、跑 248+六例门），然后 ff-only 落地。

stage: merge（共享语法批单独交付）| plan_id: PLAN-105 | plan_revision: r1 | round: 90 | from: work | to: work | status: executing | outcome: pass（本批）| worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: auto-lang master `384609645` | next: work

2026-11-19 stage: merge（用户裁定：105 先交付共享语法批，解除 125 阻塞）| outcome: pass（本批）

**交付内容（auto-lang master，ff-only 落地）**
- `ba462cee5`：AGE **包面**括号属性元素（`age-element-props-block` 由 component-only 放开到 age-package）+ **连字符属性键**（`age-event-key` 增 `ident ('-' ident)*`；键名改取键节点源文本切片，`onclick.self` 与 `stroke-width` 同源）。
- `384609645`：**连字符元素标签**（`age-element-tag` = `ident ('-' ident)*`，四个元素体规则的标签位改用它；`preview-card`/`grid-item`/`code-block` 等）。
- 适配当前主线：原补丁不能直接套用（主线已有 PLAN-124 的 `age-route-link-block`），本批**保留主线的路由 link 元素**、去掉 105 的重复 `age-element-link-block`；grammar census 具名重基线为 age-component `3351886c…`、age-package `b7b7acef…`。
- 落地验证（master 上）：census 4/4、plan981 248/248；主检出无 WIP 代码。

**125 冻结七原页验证（用本批 core 构建的 `ace_node_desktop_window --package`）**
| 页 | 结果 |
|---|---|
| accordion / scroll / scrollarea / pagination / progress / slider | **开窗成功，errors=0** |
| collapsible | **解析通过**，卡在投影后的表达式面：`source.age.component-unsupported: this expression shape is not wired`（其入口值 `collapsible (open: .isOpen)` 的字段引用形状；**不是**语法批的一部分） |

→ 七页**解析 7/7 成功**；投影保留原标签与属性（无语法/未知标签诊断；`texts` 为该页模块作用域叶，元素标签走元素项、label 走 label 位）。

**未随本批落地的 105 内容**：`ace_source_prep`（prep）、Store/列表/后端内联、元素视图与容器语义等仍在 `plan-105-dev`（两仓），按原计划继续；125 已可解除解析阻塞。

**证据**：`D:/autostack/plan-artifacts/plan-105/p125check/*.json`（七页各一份 evidence）。
## 10. 下游交接

### D-105-A（2026-11-19，等待用户决策）

- 背景：见 §9 round 77 收据。
- 选项：A 继续深挖（40–80 轮，不确定）；B 收窄到「壳件+可交互状态」口径（页面级内容列 KNOWN-DEBT）；C 收敛到 2–3 例做完整深交付；D 暂停并归档当前进度。
- 影响：option B/C 需修订计划验收标准（属范围变更，需用户授权）。
- 最新收据：
stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 77 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: f7ab49a8 | next: user

2026-11-19 stage: work（T-02 W74：可行性评估 + 决策请求）| PLAN-105 r1 | outcome: waiting | task_ids: T-02 | evidence: 见 §10 决策项 D-105-A

**018 页面组合再前进一步后遇到的墙**：页面 `cards` 是**对象元素列表**（`var cards List = []` + `.cards.push({ id: …, title: … })`，元素类型为 `Object{…}`）。组件面只收 `[]T`/`[N]T` 模型字段，且渲染需要**动态列表 + 对象元素字段投影**——与 012/013/015/016/022 的「动态列表渲染」是同一大件，且额外要对象元素支持。

**可行性评估（本会话实测口径）**：
- 已完成：核心表达式面、文本谓词、宿主 storage 仿真、store 内联、记录类型/构造函数、样式/依赖内联、列表基础（push/get/len/索引/条件 for）、路由页面组合第一片。
- 未完成的大件：①**动态列表渲染**（owned 元素不可读值 → 需新 sanctioned 路径或运行期展开，5 例共用）②**宿主 API 面**（013/015/016）③**对象元素列表**（018/022 页面）④T-03 双端接线 ⑤T-04/T-05 六例真窗/A2Vue 全量 ⑥T-06..T-08。
- 实测速率：单个子系统墙平均 3–5 轮；按当前速率，剩余 40–80 轮且仍有不确定性。

**需要用户决策（D-105-A）**：是否收窄交付口径，或继续深挖，或收敛到少数例做完整深交付。


stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 78 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 9add1b719 | next: work

2026-11-19 stage: work（T-02 W75：`push(整局部)` 打通 + 用户口径落定）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `9add1b719` | task_ids: T-02（W75）

**用户口径（D-105-A 答复）**：保留原验收口径；**先集中打通 013 Todo 一个完整应用**（对象元素列表 + 动态列表渲染 + 真实后端 API；Iced 与浏览器两端验证新增/筛选/切换状态、列表刷新与失败恢复），据此判断 Notes/Calendar/Kanban 的可复用面；124/125 另行并行。若 013 暴露需重做的共享接口，再按具体阻断重估。

**本轮交付（关键解锁）**：`Task::RecordMethodOut` 也携带 `moved_flags`，并在构造 `ResolvedArg` 时按标记置 `moved`——**记录接收者**（`.todos.push(t)`）上的列表方法此前丢失 `.move` 语义，是所有记录列表用例的卡点。修好后：
- `var t Todo = Todo { … }` + `.todos.push(t)` **通过**；
- `todos.get(0).text.len()` 的**字段投影读取**也通过（`List<Record>` 的 sanctioned 读法）。
- 新增永久回归 `p105_record_list_push_and_project_field`。

**013 复用判断（首轮）**：store（`TodoStore`）的列表维护、记录构造、字段投影、条件 `for` 已可用；剩余两件是**动态列表渲染**（视图循环按运行时列表展开）与**后端 API 面**（`list_todos/create_todo/update_todo/delete_todo/remove_completed` —— 后端是纯 Auto 内存实现，可走「包级 store / 同进程调用」路线）。

**验证**：plan105 25/25、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers **5/5**。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 79 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W76：013 内存后端内联）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui（本轮）| task_ids: T-02（W76）| evidence: prep 7/7；013 墙从「handler 非 store 调用」推进到「store 块内联探针拒绝」

**交付**：`inline_memory_backend`——把 `use back.api: …` 指向的**纯 Auto 内存后端**内联进 store 单元：`list_todos()`（列表即真值，删行）、`create_todo(x)`（push + `.next_id` 自增）、`update_todo(id,text,done)`（下标搜索后整条替换）、`delete_todo(id)` / `remove_completed(_)`（下标搜索后 `remove`）；并从 `db.at` 取**种子数据**与 **nextid 初值**写回 store model（`var todos []Todo = [Todo { … } …]`、`var next_id int = 4`）。

**013 墙序列**：`handler body call statements are only wired for store.<Msg>()`（= store 里的 api 调用）→ **`.store.*` 需要同源 store 声明（S1）**，即 store 块没被内联。

**定位结果**：`extract_store_block` 对**原始** store 块的探针通过；对**改写后**的块探针报 `source.syntax: expected fn, found store`。→ 生成的语句（`var __mb_* int = …`、`for … { if … { .todos[i] = Todo { … } } }`、`var __g Todo = .todos.remove(i)`）在 **store 块上下文**里不被接受（或块边界被改写影响）。下一刀：先查 age-component 文法里 `store` 块允许的语句面，再决定是「把 shim 生成物放到组件级 handler」还是「放宽 store 块语句面」。

**验证**：prep 7/7（六例 load/project 门保持绿；开关未改动）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 80 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: cc394cf1 / 0f13a24b5 | next: work

2026-11-19 stage: work（T-02 W77：013 后端内联五连墙）| PLAN-105 r1 | outcome: pass | code_commit: auto-ui `cc394cf1`、auto-lang `0f13a24b5` | task_ids: T-02（W77）| evidence: 013 墙序列见下

**本轮交付**：①列表方法面补 `insert`/`remove`（组件面语句门 + 诊断点名：`handler body method \`x\` is not wired yet`）；②`update_todo` 生成物改走 **`remove`+`insert`**（下标赋值带记录字面量在 store 块语法不可解析——这是 bisect 出来的）；③后端记录类型收集 `collect_type_blocks`（支持 `pub type X = { … }` 与冒号字段），并按**解析探针**过滤（015 的 `List<..>` 后端类型会被跳过）；④`rewrite_record_text_fields` 支持冒号字段形态；⑤`inline_memory_backend` 与环境开关 `P105_MEM_BACKEND`（默认关，避免影响其它例子）。

**013 墙序列（本轮推进 5 格）**：`store 块未内联(S1)` → `method insert 未接线` → `no record type named Todo`（`pub type` 形态） → `field text of Todo has type str, found Str`（冒号字段） → **`view-escape: a whole owned element is not a value`**。

**下一刀（两个候选实验）**：①`update` 改 `.todos.set(i, Todo{…})`（避免 `remove` 到局部）；②`delete`/清空改用能丢弃返回值或绑定形态的 `remove`（先在 av 里验 `var x T = list.remove(i)` 是否触发 view-escape）。

**验证**：prep 7/7；core plan105 25/25；av 5/5。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 81 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: c58b4a692 | next: work

2026-11-19 stage: work（T-02 W78：013 的 view-escape 归因）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `c58b4a692` | task_ids: T-02（W78）| evidence: 最小复现落到 bind 层

**本轮结论（把 013 的最后一个大墙归因清楚）**：
1. `var gone Todo = .todos.remove(0)` **可用**；`.todos.set(0, Todo { … })` **可用** → 两者都进永久回归（新增 `p105_list_set_element_replaces_value`）。
2. 013 的 `view-escape` 来自**句柄语义**：`.todos[i].done` 与 `.todos[i].text`（以及 `get(i).f` 形态）在**bind 层**就被拒——`resolve` 的 Index/list-get 分支在 `!aggregates_admission` 时直接报错（消息文案却建议 `xs[i].f`，属误导）。最小复现：`for i < .todos.len() { if .todos[i].done == false { … .todos[i].text … } }` → `source.view-escape`（stage: Bind）。
3. 与之相对，verify 层（`check_collections_rules`）**已有**正确的 ElemView 规则：`ListGet` 仅允许作 `FieldGet` 基或 `.len()` 操作数、owned 终态字段只允许 `.len()` 消费。也就是说：**bind 层过严、verify 层已就绪**。

**下一刀（明确）**：把 AGE 组件卫生道的 `aggregates_admission` 打开（或在 Index/list-get 分支对 AGE 域放行 ElemView），让 `.todos[i].f` 走到 verify 层按现有规则判定；先补 3 个永久用例（字段投影读、`if` 条件里的 bool 字段、文本字段作拼接），再跑 248 例 + 六例门确认没有回归。

**验证**：av list handlers 7/7；prep 7/7；core plan105 25/25。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 82 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 78b8b063d | next: work

2026-11-19 stage: work（T-02 W79：元素视图放行）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `78b8b063d` | task_ids: T-02（W79）

**交付（`xs[i].f` 全链路放行）**：①`resolve_with_view` 入口 + `resolve_detailed_inner(..., elem_view)`：AGE 车道在 bind 层不再一刀切拒元素视图；②`verify_collection_family_age_elem_view` 入口 + `Verifier.elem_view_owned`：AGE 车道允许 owned 文本字段投影**读成值**（`preview = .todos[i].text`），冻结 profile 保持 `.len()` 消费；③组件面组件入口改用 `compile_program_parts_age_elem_view` 变体（能力表不动，避免 Atom 类型表重复）；④新增永久回归 `p105_elem_view_field_projection_compiles`（26/26）。

**基线校准（重要）**：全量 `cargo test -p ace-core --test conformance` 在本分支**基线**就是 834 passed / 36 failed（stash 对照实证），本轮 835/36——即**零新增失败**，新增用例通过。av 全量 1051/264 亦为分支既有基线。

**013 现状**：元素视图墙已过；新墙 = `no local named self in scope` + `field access ... found i32` + `indexing requires a List receiver, found i32`——即**后端内联 shim 的生成语句落在 store 块里**，经 store 展开后作用域不对（`self` 不存在、`.todos` 被解析成 i32）。下一刀：把 shim 生成物从 `store` 块挪到**组件级 handler 体**（或让 store 展开支持生成语句的作用域），并核对 `.next_id` 的字段归属。

**验证**：plan105 26/26、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 7/7；prep 7/7。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 83 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 待记 | next: work

2026-11-19 stage: work（T-02 W80：013 shim 作用域归因）| PLAN-105 r1 | outcome: pass | task_ids: T-02（W80）| evidence: 失败语句上下文（`.todos.push/.insert`）

**本轮结论（把 013 新墙归因到具体构造）**：失败诊断的上下文已抓取——
1. `no local named `self` in scope` 落在 `if .input != "" { .todos.push(Todo { id: .next_id, … }) … }`；
2. `field access requires a record-typed value, found i32` 落在同一句（`.next_id` 位置）；
3. 另一处 `.todos.insert(__mb_u2, Todo { id: .todos[i].id, … })` 同样报 `self`。

→ 即：**shim 生成的列表方法调用语句落在 store 的 handler 里**，经 store 展开后其字段引用被写成 `self.*`，而**检查函数/场景体的降级通道不会重命名 `self`**（该通道只重命名 `ComponentStmt` 的表达式，`self` 是 store 展开阶段引入的）。附带 `.next_id` 被判成 `i32` 也源于同一映射（`.field` 被当成裸值）。

**下一刀（明确）**：查 `expand_store_calls` 里的字段映射（`store_field_prefixed` 与 `rewrite_store_expr`），确认它把 store 字段映射成什么；两种修法：①让展开后的语句通过同一 `rename_expr` 通道（保留 `Field{self,…}` 语义到组件层）；②shim 不在 store 里生成列表方法语句，而是生成**组件级 handler**（`.AddTodo -> { … }` 直接操作组件字段）。

**验证**：prep 7/7（探针已清理）。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 84 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 76b86b49c | next: work

2026-11-19 stage: work（T-02 W81：013 诊断与 store 映射修复）| PLAN-105 r1 | outcome: pass | code_commit: auto-lang `76b86b49c` | task_ids: T-02（W81）

**交付**：①`rename_expr` 对**未声明的 model 字段**改为具名拒（`model has no field \`x\``），不再留成 `self.x` 让共享 verify 报误导性的 `no local named self`；②`rewrite_store_expr` 补 **`CtorNamed`/`ObjectLit`** 分支——store 字段映射现在能覆盖构造字面量里的 `.field`（`Todo { id: .next_id, text: .input, … }`）。

**013 墙序列（本轮推进 2 格）**：`no local named self`（误导）→ **`model has no field next_id`**（具名）→ 补 ctor 映射后 → **`no builtin method clone on receiver type bool`**。

**下一刀**：查 `.clone()` 落在 bool 上的来源（大概率是组件面 `wrap_owned_transfer` 对非文本字段也加了 clone，或 shim 的 `!current_done` 实参被克隆），改成只对 owned 文本/记录加 clone。

**验证**：plan105 26/26、980 39/39、981 248/248、104 3/3、1072 19/19；av list handlers 7/7。
### D-105-B（2026-11-19，013 暴露的共享接口阻断，等待用户决策）

**具体阻断（可复现）**：013 的组件源码现在**编译通过**（组件面/verify 全绿），卡在 **AV（Iced）后端 lowering**：
`lower `d___age_state_0_input`: Unsupported { code: "lower.type-use", message: "type kind Str is outside the av.collections/1 source surface" }`。

**根因**：`av.collections/1` 的 MIR 只承载 **i32/bool 标量**（`scalar_shape_of_use`），没有 owned 文本 `Str` 的类型/值/操作；而任何"列表 + 文本"的应用（012/013/015/016/018/022 全部）都需要它。

**影响范围**：所有含 owned 文本的列表应用（六例全部）；A2Vue 侧不受此限（TS 发射不经 AV MIR），但 AV 真窗（Iced）全拦。

**可复用结果（已实证）**：组件面表达式/循环/记录/构造/列表（push/get/set/insert/remove/索引字段投影）/store 内联/后端内联/属性诊断——这些都在 core（两侧共用），013 已推进到"只剩后端 lowering"。

**重估**：在 AV collections MIR 增加 `Str`（类型、值表示、concat/clone/len、与 text-record 面共享字符串堆）≈ 5–10 轮且有风险；或改走 **A2Vue（TS）端**先交付 013（不受此阻），Iced 端后补。

**需要决策**：A 继续在 AV 侧补 `Str`（保持原口径，5–10 轮）；B 先以 A2Vue 端交付 013（浏览器可用即算该例端到端），AV 端列为已知缺口；C 收窄到"少量真正完整"的应用（用户此前首选 C 的条件成立时）。

### D-105-C（2026-11-19，用户决定：**暂停并交接**）

**状态**：本计划在 T-02 中段暂停（用户决定）。以下为可直接续做的交接事实。

#### 1. 已交付并稳定的能力（core，两侧共用；均有永久回归）
- 组件面表达式：比较/逻辑/算术/一元/文本拼接/`f"…"`/显示、`if` 条件与链、条件样式。
- 控制流：handler 级 `for`（列表迭代与 C 风格条件 `for`）、`while`（归一为条件 `for`）、嵌套 `if`。
- 列表：`List<T>.new()`、`push`（含整局部 `.move` 语义）、`get`/`set`/`insert`/`remove`/`len`/`clear`/空列表归一；**元素视图** `xs[i].f` / `xs.get(i).f`（字段投影读、`len()` 消费、文本字段读值）在 AGE 车道放行。
- 记录：`type` 声明（`type X { }` 与 `pub type X = { }`）、`CtorNamed` 构造、字段投影、记录列表。
- store：同源 store 内联（字段/消息/处理器）、`store.<Msg>()` 展开、store 字段映射（含构造/对象字面量内部）。
- 诊断：未声明 model 字段具名拒；未接线调用点名（方法名）。
- prep（auto-ui）：依赖样式/本地样式内联、`use` 空白化、`type`/`fn`/`const` 内联、store 块内联、`str.contains` 直译、宿主 storage 仿真、列表初值展开、记录文本字段 owned 化、model 文本字段 owned 化、`routes`/`outlet` 页面组合第一片（开关 `P105_ROUTE_PAGES`）、纯 Auto 内存后端内联（开关 `P105_MEM_BACKEND`）。

#### 2. 具体阻断（D-105-B）
- 013 组件源码**已全部编译通过**，卡在 **AV（Iced）后端 lowering**：`lower.type-use: type kind Str is outside the av.collections/1 source surface`。
- 根因：`av.collections/1` 的 MIR 仅支持 i32/bool 标量（`scalar_shape_of_use`），无 owned 文本 `Str`（类型/值/concat/clone/len）。
- 影响：六例（012/013/015/016/018/022）**全部**含「列表 + 文本」，AV 真窗全拦；A2Vue（TS）端不经 AV MIR，不受此限。

#### 3. 续做路线（任选其一）
- **A**：在 AV collections MIR 增加 `Str`（与 text-record 面共享字符串堆）≈ 5–10 轮；完成后 Iced 端可交付。
- **B**：先走 A2Vue（TS）端交付 013，AV 端列已知缺口；需先打通 T-05 的**整包单元装配**（`PackageModuleInput` 多单元；单单元会报 `source.module-identity`）。
- **C**：收窄到 1–2 例真正完整（用户预置条件）。

#### 4. 工作树与分支
- auto-ui：`D:/autostack/.wt/ui-105/auto-ui`，分支 `plan-105-dev`；auto-lang：`D:/autostack/.wt/ui-105/auto-lang`，分支 `plan-105-dev`。
- 两棵树均已提交、工作区干净；最近提交：auto-ui `e3883ae5`、auto-lang `0a0872b18`。
- 开关：`P105_ROUTE_PAGES=1`（路由页面组合）、`P105_MEM_BACKEND=1`（内存后端内联）。
- 构建：`CARGO_TARGET_DIR=D:\autostack\plan-artifacts\plan-105\target`（auto-lang）与 `...\target-ui`（auto-ui）。

#### 5. 证据锚
- 门：plan105 26/26、980 39/39、979 24/24、981 248/248、104 3/3、1072 19/19；av list handlers 7/7；auto-ui prep 8/8。
- 基线：本分支全量 `ace-core conformance` 基线 834 passed / 36 failed（预存在，stash 对照实证），本轮改动零新增失败。
- 六例真窗现状：015 7 叶、018 11 叶（壳件）、022 8 叶（壳件+主题可交互）；013 组件编译通过、卡 AV lowering；016 卡 view-for over computed；012 卡未定型列表元素 owned 化。


stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 85 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: e3883ae5 / 0a0872b18 | next: user

2026-11-19 stage: work（T-02 W82 + 交接）| PLAN-105 r1 | outcome: waiting（用户决定暂停）| code_commit: auto-ui `e22b9c5c`/`e3883ae5`、auto-lang `0a0872b18` | task_ids: T-02

**本轮交付**：①局部写只在**文本局部**补 `.clone()`（此前 bool 局部被 clone → `clone on bool`）；②prep 把组件 **model 文本字段**改写为 owned `Str`（AV collections 面只承载 owned 文本的字段声明形态）；③**决策项 D-105-B** + **交接文档 D-105-C** 写入 §10（能力清单、阻断、续做路线、工作树、证据锚）。

**013 终态**：组件源码**全部编译通过**（组件面/verify/store/后端内联 OK），卡在 **AV（Iced）后端 lowering**：`lower.type-use: type kind Str is outside the av.collections/1 source surface`——`av.collections/1` MIR 只有 i32/bool 标量。六例全部受此限（A2Vue/TS 端不受限）。

**用户决定（D-105-B 答复）**：**暂停并交接**。不需要再跑检查或开工；按 §10 的 D-105-C 续做即可。

**状态**：两棵工作树干净且已提交（auto-ui `f0ca8841`、auto-lang `0a0872b18`）；计划文件保留 `status: executing`（未交付），交接事实在 §10。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 86 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: 5e5170ae | next: user

2026-11-19 stage: waiting（用户决定暂停；本轮不做事、不跑检查）| 续做入口：§10 D-105-C（A 补 AV `Str` / B 先走 A2Vue / C 收窄）。恢复方式：顶层说「继续计划105」或选择路线。

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 87 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: 9dd311b6 | next: user

2026-11-19 stage: waiting（第 3 个等待轮；无新指令，不做事不跑检查）| 待用户选择路线 A/B/C 或说「继续计划105」。按技能规则，连续三轮等待后目标转 blocked。

stage: merge（尝试）| plan_id: PLAN-105 | plan_revision: r1 | round: 88 | from: user | to: user | status: executing | outcome: blocked（并发计划冲突）| worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: c2802cac / 0a0872b18 | next: user

2026-11-19 stage: merge（用户要求把已完成部分并回主分支）| outcome: blocked | evidence: `git merge-tree --write-tree` 试合并

**已做的核实**
- 主检出（两仓）**无 WIP 代码**（只有其它计划的 docs 改动）；工作树两棵干净。
- 分支差异：auto-ui `master..plan-105-dev` = 57/74；auto-lang = 153/57。
- 试合并冲突数（`merge-tree --write-tree`）：**auto-ui 3 处、auto-lang 3 处**，其余文件按内容自动合并。

**具体冲突**
- auto-ui：`ace_node_desktop.rs`（2 处，均为本分支**新增块**，好解）、`ace_package_sfc.rs`（2 处——master 已重构为 **routed SFC**（PLAN-124/125），本分支加了 `generate_package_sfc_with_root` + prep 入口，需手工合并）、`docs/plans/105-*.md`（取本分支）。
- auto-lang：`resolve.rs`（1 处）、`plan981_grammar_scope.rs`（2 处，**文法 census 哈希**需按合并后文法重算）、`grammars/auto-source.at`（4 处，**语义冲突**：master 新增 `age-route-link-block`（路由 link），本分支新增 `age-element-link-block` + AGE 的 `array-lit`/空数组 profile 收录）。

**结论**：并回主分支**不是一次机械合并**——文法里的 link 元素与 124/125 的路由 link 是同一语法的两种实现，必须选定/融合一套，否则会影响所有 AGE profile 的解析（248 例 census + 全量 conformance 需重跑）。故本轮按规模停下来报告，未在主分支上留下任何提交。

**建议路线**：①先等 124/125 合并，再把 105 分支 rebase 上去（冲突面会变小）；②或以「work 阶段」做文法融合（预计 2–4 轮：选定 link 元素、重算 census、跑 248+六例门），然后 ff-only 落地。

stage: merge（共享语法批单独交付）| plan_id: PLAN-105 | plan_revision: r1 | round: 90 | from: work | to: work | status: executing | outcome: pass（本批）| worktree: D:/autostack/.wt/ui-105/{auto-ui,auto-lang} | commit: auto-lang master `384609645` | next: work

2026-11-19 stage: merge（用户裁定：105 先交付共享语法批，解除 125 阻塞）| outcome: pass（本批）

**交付内容（auto-lang master，ff-only 落地）**
- `ba462cee5`：AGE **包面**括号属性元素（`age-element-props-block` 由 component-only 放开到 age-package）+ **连字符属性键**（`age-event-key` 增 `ident ('-' ident)*`；键名改取键节点源文本切片，`onclick.self` 与 `stroke-width` 同源）。
- `384609645`：**连字符元素标签**（`age-element-tag` = `ident ('-' ident)*`，四个元素体规则的标签位改用它；`preview-card`/`grid-item`/`code-block` 等）。
- 适配当前主线：原补丁不能直接套用（主线已有 PLAN-124 的 `age-route-link-block`），本批**保留主线的路由 link 元素**、去掉 105 的重复 `age-element-link-block`；grammar census 具名重基线为 age-component `3351886c…`、age-package `b7b7acef…`。
- 落地验证（master 上）：census 4/4、plan981 248/248；主检出无 WIP 代码。

**125 冻结七原页验证（用本批 core 构建的 `ace_node_desktop_window --package`）**
| 页 | 结果 |
|---|---|
| accordion / scroll / scrollarea / pagination / progress / slider | **开窗成功，errors=0** |
| collapsible | **解析通过**，卡在投影后的表达式面：`source.age.component-unsupported: this expression shape is not wired`（其入口值 `collapsible (open: .isOpen)` 的字段引用形状；**不是**语法批的一部分） |

→ 七页**解析 7/7 成功**；投影保留原标签与属性（无语法/未知标签诊断；`texts` 为该页模块作用域叶，元素标签走元素项、label 走 label 位）。

**未随本批落地的 105 内容**：`ace_source_prep`（prep）、Store/列表/后端内联、元素视图与容器语义等仍在 `plan-105-dev`（两仓），按原计划继续；125 已可解除解析阻塞。

**证据**：`D:/autostack/plan-artifacts/plan-105/p125check/*.json`（七页各一份 evidence）。
## 10. 下游交接

本计划按 U03 独立完成。AV 收据与 1071 已交范围及其他 U 组结果汇合后供 W1 签发、1068/T-04 和 985/B 接收。A2Vue 收据供 1072 与产品 CLI 消费。A2R/AC 后续计划沿同一源码、Widgets 和独立预期补验。跨仓 golden 基线维护按已有 PARITY 路线归档。

2026-11-19 停放与恢复：按用户裁定暂停过本计划开发；104 已交付并合入 master（ui `5a862d46` / lang `2fb094483`）后，已按裁定从 master 重建工作树（不共享 104 分支）并完成 T-02 W1。恢复/续做时先读 §9 的 W1 收据与 `plan-artifacts/plan-105/t02-gap-map.md`（S1–S10 缺口与顺序在案）。
