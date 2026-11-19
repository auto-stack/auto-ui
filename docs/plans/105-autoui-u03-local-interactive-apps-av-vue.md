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
- [ ] **T-04｜work** 从真实入口逐例验 AV/Iced、权限、错误恢复、关闭与资源。（**部分**：015/018/022 已真开 Iced 窗并产证据 JSON——frames 2/2/3、errors 0、sessions 1，收据见 §9 W25；013/016 待 S1 store、012 待局部列表；脚本化点击受 handler 体限制仍拒。）
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

2026-11-19 stage: work（T-02 W1：组件面 entry 值边界）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev`（基 master `670c720a8`）| code_commit: auto-lang `2a186c66a` | task_ids: T-02（W1）| evidence: `D:/autostack/plan-artifacts/plan-105/t03-*.log` + `probe/plan105_u03_probe.rs`

**恢复与基面**：用户告知 104 已交付并合并；按裁定从 **master 重建** 105 树（不共享 104 分支）：auto-ui `481a3cb5`、auto-lang `670c720a8`、auto-web detached `26d085f7`、auto-down detached `72b0970`。

**master 缺口复测（T-02 矩阵）**：S7 `outlet` 已由 104 交付（proj/pkg **OK**）；015、022 投影 OK；其余 S1 store、S2 列表初值、S3 f-string、S4 int→text、S5 msg 载荷逐行、S6 timer、S8 `&&` 绑定仍在。012/016/013/018 的真实首阻断已重新定位（分别 L85 列表初值 / L12 msg 块 / L54 `}` / L81 `}`）。

**本刀交付（S8 + `to:` 键 + 包面事件绑定）**：component 面 `age-event-binding` 实参括号**必填并前置**——`checked: .a == 0 && .c > 0` 不再被裸 `.a` 贪吃成事件绑定（此前整类 U03 元素绑定回滚到 item 级）；裸 `.Msg` 仍走 pratt Field（投影按事件键 + msg 名单识别）；事件绑定面开 **age-package**（012/016 的 `onclick: .Msg(args)` 在包面此前被当普通调用 verify 拒）；`to:` 是全局保留 token，条目键位按 `r-to` 直收（018 路由按钮 / Widgets sidebar、breadcrumb 族）。永久回归 `plan105_component_entries` **4/4**；grammar census 具名重基线（age-component `2df608b3…`→`fe09a364…`、age-package `baa977a4…`→`4b6a9719…`，13 邻居逐字节不变）。affected 组全绿：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24（单组 0.03–1.58s）。

**判读（下一刀）**：012 先解 S2 列表初值（typed/untyped、空/非空）；016 解 S5 msg 载荷逐行；013/018 的首阻断已不是 entry 绑定（推进到 L54/L81 的元素体/子组件深处，下一轮二分）；store 面 S1（`store` 声明 + `use back.api` + `.store.*` + 方法调用）是最大单刀，独立切。`next_responsible_role: work（T-02 W2）`

2026-11-19 stage: work（T-02 W2：msg / 视图数据列表 / 组件面 f-string）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `45d2448a1` | task_ids: T-02（W2）| evidence: `plan-artifacts/plan-105/t04-*.log` + `probe/`

**交付**：①`msg` 末项尾随逗号容错——016 逐行一成员、末项带逗号的书写形此前让 msg 块整体回滚到 model 槽位；980 坏形状用例的「悬空逗号」改为「双逗号」并具名说明（尾逗号已成合法形）。②`age-view-data-lit` 补**单行逗号分隔**形（016 `var weekday_labels = ["Su", …]`；元素=记录或标量表达式，原始 span 文本，1071 只读视图数据边界不变）。③`fstr::desugar_widget_source`（反引号 + `f"` 两遍）接入 `age_component` 门面——传统 AutoUI 源 f-string 在组件面可解析（此前只脱反引号形）。永久回归 `plan105_component_entries` **7/7**；census 重基线（age-component `fe09a364…`→`d79b24f8…`、age-package `4b6a9719…`→`6e9ae5ae…`）；affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**真实六例复测**：**013 投影 OK**（entry 绑定 + f-string 后整例可投影）；016 推进到 L73（`button ">"`/`variant` 已排除，待按前缀二分）；018 仍 L81；012 仍 L85（typed `[]str` 初值）；015/022 OK。

**下一刀**：S2 typed 列表初值（012 `[]str/[]int`；`w_rows` 有写面，需先定列表 state vs 只读数据边界）；S3 包面/包片段 f-string 脱糖接入（auto-ui `ace_package_lane` / `ace_package_sfc` 的 units）；S4 int→text；016/018 深层二分。`next_responsible_role: work（T-02 W3）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 45d2448a1 | next: work

2026-11-19 stage: work（T-02 W3：包面 f-string 脱糖）| PLAN-105 r1 | outcome: pass（W3 完成；S4 试做后回退并记账）| worktree: `D:/autostack/.wt/ui-105/auto-ui` + `auto-lang` @ `plan-105-dev` | code_commit: auto-ui `0af20bf9`（auto-lang 本刀未改，停 `45d2448a1`）| task_ids: T-02（W3）| evidence: `plan-artifacts/plan-105/t05-ace-bridge-check.log`

**交付**：新增 `ui::ace_package_units::desugar_units`（单元集合 → 脱糖后单元集合）并接入三条包面车道的**唯一**入口——`PackageStateEngine::new`、`generate_package_sfc`、`PackageFragmentLane::compile`。此前只脱 entry 单元，其余单元里的 `f"…"` 在 verify 期具名拒；现在包闭包全单元统一脱糖，对无 f-string 源逐字节 no-op（980/981/104/1072 用例不受影响）。`cargo check -p auto-ui-runtime --features ace-bridge` 通过（运行期随 T-05 A2Vue 浏览器档复验）。

**S4 int→text 试做（已回退，未提交）**：把文本叶隐藏函数由 `expr.clone()` 改为 `__fstr(expr)`（组件面 `age_component::hidden_fn` + 包面 `age_package::hidden_fn`）后，组件面通过、包面报 `source.module-symbol: unknown function __fstr`——包 binder 符号面不认内建 `__fstr`（只认源里脱糖出的调用形）。两处已 `git checkout` 回退。结论：S4 需独立方案（直接构造 `ExprKind::TextDisplay`，或给包 binder 增内建识别 + 正反例），下一刀单列。

**其余**：S2 typed 列表初值未动（012 仍卡 `var cities []str`；`w_rows` 有写面，需先定列表 state vs 只读数据）；016 卡 L73、018 卡 L81（深层二分未做）。
`next_responsible_role: work（T-02 W4：S4 或 S2 typed 列表 / 016、018 二分）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 0af20bf9 | next: work

2026-11-19 stage: work（T-02 W4：int 文本叶 TextDisplay）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `c8ca978d0` | task_ids: T-02（W4）| evidence: `plan-artifacts/plan-105/probe/` + affected 组运行记录

**交付**：`text .n`（int 字段）此前按 str 叶发 `.clone()`，verify 报 `numeric receiver has no method clone`。改为**静态类型分流**：`self.<field>` 且字段 int/i32 → 发 `ExprKind::TextDisplay`（PLAN-964 的 <int>→str 显示节点），其余仍 clone。组件面 `age_component::hidden_fn` 与包面 `age_package::hidden_fn` 各加 display 位（四个调用点按 `text_leaf_is_int_field` 判定）；`domain::conclusion_of_def` 的文本结论补认 `TextDisplay`（否则 int 叶仍被判「非文本」）。永久回归 `p105_int_field_text_leaf_displays`（int+str 混合叶，累计 **8 例**）。affected 全绿：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**真实六例复测**：013/015/022 投影 OK；012 仍卡 L85（typed `[]str` 初值）；016 卡 L73、018 卡 L81（深层二分未做）；包面 6/6 仍 `widget` 回滚（包面 body/多单元，后续刀）。

**下一刀**：S2 typed 列表（012；`w_rows` 写面需先定列表 state vs 只读数据边界）、016/018 前缀二分、S1 store。`next_responsible_role: work（T-02 W5）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: c8ca978d0 | next: work

2026-11-19 stage: work（T-02 W5：link 元素 + 018 整例投影）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `3ff086991` | task_ids: T-02（W5）| evidence: `plan-artifacts/plan-105/probe/`（bisect 脚本）+ affected 组运行记录

**交付**：`link (to: "/") { … }`（018/021/023 真语料路由元素）——`link` 是全局保留 token（`r-link`），元素标签位只收 ident，整块回滚到 bare-element 兜底。新增 `age-element-link-block`（`r-link(field:"tag")` + 括号属性 + 可选体块）并挂进 `age-view-element`。永久回归 `p105_link_element_tag_projects`（累计 **9 例**）。grammar census 具名重基线：age-component `d79b24f8…`→`90be7af3…`（age-package 不受影响）。affected 全绿：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**真实六例进展**：**018 组件面整例投影通过**（前缀 bisect 全部 cut OK）；016 卡在 `text f"Selected: ${.store.selected_date}" { style: caption_text }`——`caption_text` 是 stylekit 导入样式，单源组件投影没有 import 上下文；012 仍卡 typed 列表；013/015/022 OK。

**下一刀**：016 的导入样式解析（组件面前置或车道侧解析，需与 1072 的 style 解析合同对账）、S2 typed 列表（012）、S1 store。`next_responsible_role: work（T-02 W6）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 3ff086991 | next: work

2026-11-19 stage: work（T-02 W6：组件面源准备）| PLAN-105 r1 | outcome: pass（本刀完成；接线留 T-04）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `c03eb6d0` | task_ids: T-02（W6）| evidence: `plan-artifacts/plan-105/t06-source-prep-test.log` + `probe/`（016 内联样式复测）

**交付**：`ui::ace_source_prep`——`pac_dep_roots`（行扫描 `pac.at` 的 `dep <name> { path }`，归一化 `..`）+ `prepare_component_source`（`use <dep>.<module>: item, …` 时到依赖源取单行 `pub style <item> = "…"` 字面量并**内联**；所有 `use` 行**等长空白化**保 span/行号）。只内联单行样式字面量，参数化/多行样式具名不猜。单元测试 2/2（内联 + 未知项不内联），`cargo test -p auto-ui-runtime --features ace-bridge --lib ui::ace_source_prep` 通过。

**016 实测**：组件面墙确认为导入样式 `caption_text`（stylekit）；探针里手工内联后 016 的前缀 bisect 从 L180 全视图翻到「全部 cut OK 之外的最后一段」，说明内联是必要条件之一，但 016 仍有更深构造（下一刀接 `prepare_component_source` 到 AV/SFC 投影前，再复测定位）。

**下一刀**：把 `prepare_component_source` 接到 AV `--source` 装载点与 SFC 投影前（T-04 rig），复测 016；S2 typed 列表（012）；S1 store。`next_responsible_role: work（T-02 W7）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: c03eb6d0 | next: work

2026-11-19 stage: work（T-02 W7：类型化列表数据字段改写）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `8e6e29c7` | task_ids: T-02（W7）| evidence: `plan-artifacts/plan-105/t07-source-prep-test.log` + `probe/`

**交付**：`ace_source_prep` 补**类型化列表数据字段**改写——012 的 `var cities []str = [...]` 在组件面没有「列表 state」切片（类型位 `[]T` 可解析，但列表字面量初值走 pratt 硬失败 `expected expression, found [`）；ACE 两面已支持无类型列表字面量（1071 只读视图数据），源准备层把类型段等长空白化 → 投影按 view_data 收下（span 不变）。只改「类型含 `[]` 且初值以 `[` 开头」的字段；标量/非列表初值/普通语句原样（单测覆盖）。写面边界：被 handler 写入的列表字段（012 `w_rows`）改写后成为视图数据，由具名拒绝兜底（列表 state 另刀，不静默丢写）。单测 3/3。

**实测推进**：012 组件面由 L85（model 列表）→ **L182**，下一墙为 SVG 属性名 `stroke-width: "5"`（连字符键，语法层不支持，下一刀）；016 仍卡导入样式后的更深构造；013/015/018/022 投影 OK。

**下一刀**：连字符 entry key（`age-event-key` 分段）、012 的 `w_rows` 写面边界（列表 state 单列）、016 复测、S1 store。`next_responsible_role: work（T-02 W8）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 8e6e29c7 | next: work

2026-11-19 stage: work（T-02 W8：连字符键 + 括号属性元素开包面）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `b8837722b` | task_ids: T-02（W8）| evidence: `plan-artifacts/plan-105/t08-real-six-dash.log` + `probe/`

**交付**：①**连字符 entry key**（012 SVG 表盘 `stroke-width:` / `stroke-linecap:`）——`age-event-key` 增 `ident ('-' ident)*` 段链；键名改由键节点**源文本切片**直接取，点修饰符与连字符段同源，不再手工拼 `.`。②**括号属性元素开包面**（`age-element-props-block` / `-props-body` / `-link-block` 及 `age-view-element` 对应 alt）——016/018/012 的属性元素在包面此前整块回滚（package 真跑必需）。永久回归 2 例（累计 **11 例**）。census 重基线：age-component `90be7af3…`→`5095d16a…`、age-package `6e9ae5ae…`→`c6e96f4d…`。affected 全绿：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**012 新墙**：模型列表经源准备改写后，012 组件面推进到 L182 附近的 SVG 区块；但 `svg (viewBox: …) { circle (…) {} }` 单独探针**两面都 OK**，说明真正的墙在其更早的 tab/导航构造（下一轮前缀细二分）。016 仍需接线后复测；013/015/018/022 投影 OK。

**下一刀**：012 早段（L120–180）细二分；`w_rows` 写面（列表 state 单列）；016 接线复测；S1 store。`next_responsible_role: work（T-02 W9）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: b8837722b | next: work

2026-11-19 stage: work（T-02 W9：view 级 for 单变量形）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `6ac8edc92` | task_ids: T-02（W9）| evidence: `plan-artifacts/plan-105/probe/`（深前缀 bisect）

**交付**：012 world-clock 用 `for r in .w_rows { … }`（**单变量** view 级 for），此前 view-for 只收双变量形，整块回滚。语法拆成 `age-view-for-indexed`（双变量）与 `age-view-for-plain`（单变量）两条子规则（alt 不与步骤混排，两面同开）；投影把 for 字段移到解包后变体节点、`index` 缺失时置空串（不再 expect 崩）。永久回归 `p105_view_for_single_and_indexed_variables`（累计 **12 例**）；census 重基线：age-component `5095d16a…`→`9052d5cf…`、age-package `c6e96f4d…`→`ea361d1d…`。affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**012 推进**：模型列表（源准备改写）→ 连字符键 → 单变量 for 后，012 的组件面推进到 **L445 段**：handler 体内 `var`（let）+ `if` 语句 + f-string 组合（`var cc_str = f"${cc}"` / `if cc < 10 { cc_str = … }`）——下一刀。

**下一刀**：handler 体 let/if/f-string 组合（012 秒表/计时器段）、`w_rows` 写面（列表 state 单列）、016 接线复测、S1 store。`next_responsible_role: work（T-02 W10）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 6ac8edc92 | next: work

2026-11-19 stage: work（T-02 W10：handler 局部量 var + 包面 f-string 脱糖）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `be5d31b2f` | task_ids: T-02（W10）| evidence: `plan-artifacts/plan-105/probe/`（pkg/handler 探针）

**交付**：①**handler 局部量 `var` 拼写**（012 秒表 `var ss = total_s % 60`；AGE 作者面是 `let`）——`age-on-let` 拆成 `-let`/`-var` + `-tail` 三规则（两面同开），投影沿两层子节点找持有 `name`/`value` 的 tail 节点（旧形兼容）。②**包面统一 f-string 脱糖**：`compile_age_package_with_scenario` 入口对每个单元源做 `desugar_widget_source`（与组件面同律，含 handler 体）；③**`__fstr` 放行**：模块 binder 与包 binder 均把脱糖生成物透传给 resolver 的 builtin 展平器。永久回归 `p105_handler_var_locals_and_fstring`（两面成功；局部量**再赋值** `b = "0" + b` 仍具名拒绝 `source.age.component-event`——hook 局部再赋值是后续刀，不静默丢写），累计 **13 例**。census 重基线：age-component `9052d5cf…`→`01d03716…`、age-package `ea361d1d…`→`7ac658fe…`。affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**012 推进**：handler 体 `var`/f-string/`if` 两面均可解析；剩余具名拒绝=**hook 局部再赋值**（`if cc < 10 { cc_str = … }`，binder 按模型字段判）。013/015/018/022 投影 OK；包面真跑仍卡 **S1 store** 单元。

**下一刀**：hook 局部再赋值（`age-on-branch-stmt` 裸 ident 赋值 + 绑定层区分局部/模型字段）、S1 store、016 接线复测、012 `w_rows` 写面。`next_responsible_role: work（T-02 W11）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: be5d31b2f | next: work

2026-11-19 stage: work（T-02 W11：hook 局部量再赋值）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `215e14e59` | task_ids: T-02（W11）| evidence: `plan-artifacts/plan-105/probe/`

**交付**：012 秒表段 `var cc_str = f"${cc}"; if cc < 10 { cc_str = "0" + cc_str }`——裸名再赋值此前被 binder 判成「未声明 model 字段」。①`ComponentAssign` 增 **`bare`** 位（`.field =` 点语法恒指实例字段；裸名形是 `age-on-assign`）；②铸语句时裸名**先查执行路径绑定表**（形参 + 先前 hook-local 遮蔽同拼写 model 字段——981 既有语义），命中即局部写，未命中才落字段；③**可变绑定**：语句面铸完后按赋值目标名把对应 `let` 翻成 `var`（verify 的 `immutable-write` 要求）；④未派发 handler 的字段核对跳过 `let` 声明名。回归升级为两面必须通过再赋值，981 shadowing 负控保持绿（点语法不被遮蔽规则改写）。affected：plan105 13/13、104 3/3、1072 19/19、980 39/39、981 **248/248**、979 24/24、census 4/4（本刀无语法改动）。

**012 推进**：整体投影推进到 **L268**——新墙是**单行多 entry 体**：`button "时−" { onclick: .ADnH, style: stepper_btn }`（同行逗号分隔的多个 entry，当前 entry 面只收 `list-nl`）。

**下一刀**：单行逗号分隔 entry 体、S1 store、016 接线复测、012 `w_rows` 写面。`next_responsible_role: work（T-02 W12）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 215e14e59 | next: work

2026-11-19 stage: work（T-02 W12：012 形状回归锁定 + 定位）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `4538aba9c` | task_ids: T-02（W12）| evidence: `plan-artifacts/plan-105/t09-012-deep.log`、`t10-012-branch.log`、`t11-012-trim.log` + `probe/`

**交付**：把 012 已验证可用的真语料形状锁成永久回归——①元素体**单行逗号分隔**多 entry（`button "时−" { onclick: .ADnH, style: stepper_btn }`，两键两面对过）；②handler 内**无 else 的 if** + 局部量再赋值（`if .n < 10 { a = "0" + a }`，投影保留 `If { els: None }`）。plan105 回归 **15 例**。

**排查记录**：012 的 `view mini`（命名视图）、`design { mode: if … }`、括号属性元素、单行逗号体在单独探针里两面都 OK；012 组件面整体投影仍卡主视图 **alarm 段（L244–316 区域）**，但按 tab 分支整块删除、按行尾裁剪都不改变失败——说明是段内某一构造与上下文组合（非单分支可分离），下一轮用「逐项提取成最小 widget」定位。affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**下一刀**：012 alarm 段逐项提取定位、S1 store、016 接线复测、012 `w_rows` 写面。`next_responsible_role: work（T-02 W13）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 14 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 4538aba9c | next: work

2026-11-19 stage: work（T-02 W13：AGE 索引后缀）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `5eb4a4633` | task_ids: T-02（W13）| evidence: `plan-artifacts/plan-105/t12..t17-*.log` + `probe/`

**定位结论（方法论收获）**：012 组件面整体投影的墙**不在视图**——诊断里指到 L268（视图行）是**回退位置**；删掉 `on { … }` 块整例即通过，逐段增长把墙定位到 `.Tick` 体的 `var ls = secs + .offsets[.local_city] * 3600`。

**交付**：AGE 两面此前**没有索引后缀步**（`[key]` 只开在 aggregates/auto/closures/scheduler/config-document），见到 `[` 整段回滚成 `source.age.structure`。把 `age-component`/`age-package` 加进该 postfix 步 → 解析/投影通过。语义边界（诚实）：列表索引在包面具名 `source.age.component-unsupported`（列表 state/索引求值另刀，不静默当 0）；`Time.now_sec()` 这类模块调用仍是具名 `source.undefined`（stdlib 接线另刀）。永久回归 `p105_list_index_expr_projects`（累计 **16 例**）；census 重基线（age-component `d6c48e1d…` / age-package `0aa320f4…`）。affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**下一刀**：列表 state + 索引求值（`.offsets[i]`，012 世界时钟必需的语义）、handler 内 stdlib 模块调用（`Time.now_sec()`）、S1 store、016 接线复测。`next_responsible_role: work（T-02 W14）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 15 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 5eb4a4633 | next: work

2026-11-19 stage: work（T-02 W14：连字符元素标签）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `aaa7763c1` | task_ids: T-02（W14）| evidence: `plan-artifacts/plan-105/t18..t20-*.log` + `probe/`

**交付**：016 组件面整体投影的墙（诊断行 L91 是视图行）定位到 `grid-item { text d { … } }`——元素标签位只收 `ident`，连字符自定义元素名整块回滚成 bare-element 兜底。新增 `age-element-tag`（`ident ('-' ident)*`）+ `age-element-tag-dash`，四个元素体规则（labeled-block/block/props-block/labeled）标签位改用该规则；投影取 tag 源文本切片，保留完整连字符名。永久回归 `p105_hyphenated_element_tags_project`（累计 **17 例**）；census 重基线（age-component `f3e48f7b…` / age-package `6850dae6…`）。affected：104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4。

**016 推进**：`grid-item` 修好后 016 整体投影从 L91 推进到 **L175**（footer 的 `text f"Selected: ${.store.selected_date}" { style: caption_text }`）；剩余报错是 style 引用 `caption_text` 在「完整 prepared 源」里不解析（同形最小样例两面对过、style 置于 widget 前后都过）——下一轮用最小化 diff 复现（疑与 `on`/`store` 引用组合或 prep 拼接有关）。

**下一刀**：016 的 `caption_text` 解析最小复现、列表 state/索引语义、S1 store、012 `w_rows` 写面。`next_responsible_role: work（T-02 W15）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 16 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: aaa7763c1 | next: work

2026-11-19 stage: work（T-02 W15：依赖样式内联修复）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `60b20fcc` | task_ids: T-02（W15）| evidence: `plan-artifacts/plan-105/t21..t25-*.log` + 端到端单测

**交付（两个真 bug）**：①`style_literal` 扫描用 `?` 提前返回——styles.at 首行非 `pub style` 就整函数放弃，**依赖样式从未真正内联**（016 的 `caption_text` 因此永远解析不到）；改逐行 `continue`。②内联样式追加到文件尾时模块样式表解析不到（016 实测：追加→不解析；**插到组件头之前**→整例通过）；现按首个 `widget`/`node`/`component` 行前插入。

**证据**：新增端到端单测 `real_u03_apps_project_after_prep`——**013/015/016/018/022 经真实 prep（真 pac.at 依赖 + 真 stylekit 源）后组件面整体投影通过**（016 从此解除组件面阻断）；012-clock 的 `on` 块仍有未接语义（列表索引求值等），不在断言内、按缺口表跟踪。单测 4/4。

**下一刀**：012 的 `on` 块剩余语义（列表索引/模块调用）、列表 state、S1 store、AV/SFC 接线（T-04）。`next_responsible_role: work（T-02 W16）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 17 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 60b20fcc | next: work

2026-11-19 stage: work（T-02 W16：handler 裸名复合赋值）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` + `auto-ui` @ `plan-105-dev` | code_commit: auto-lang `054f3b1d4`、auto-ui `d12ff251` | task_ids: T-02（W16）| evidence: `plan-artifacts/plan-105/t26..t29-*.log` + `probe/`

**交付**：①**handler 裸名复合赋值**（012 日历换算段 `if mp >= 10 { civil_m = mp - 9 }` / `civil_y += 1`）——`age-on-assign` 改 `ident age-assign-op pratt`，投影把 `x += e` 脱糖为 `Assign { rhs: BinOp(Var(x), Add, e) }`（裸名目标在铸语句期按执行路径绑定表解析，与 981 遮蔽同源）；回归 `p105_handler_compound_assign`（累计 **18 例**）；census 重基线（component `0c1a1938…` / package `5f7a3a8a8…`）；affected 全绿（104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4）。②清掉 012 定位用的两个临时 debug 测试（auto-ui `d12ff251`）。

**012 定位推进（逐 handler → 逐语句二分）**：31 个 handler 里唯一失败的是 `.Tick`（L438–593）；语句二分把墙从 `civil_y += 1` 推进到 `for c in .cities { … }`——**handler 级 `for` 循环**在语句面尚未开面（handler 只有 assign/let/if）。另记：裸名赋值在 `if` 分支内可用、在 handler 体顶层仍不解析（`bare` 形只在 branch 语句面命中），下一刀一并核。

**下一刀**：handler 级 `for`（列表迭代 + 具名边界）、顶层裸名赋值核对、S1 store、T-04 接线（AV/SFC 真跑）。`next_responsible_role: work（T-02 W17）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 18 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 054f3b1d4 | next: work

2026-11-19 stage: work（T-02 W17：handler 级 for 结构）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `5c7be439d` | task_ids: T-02（W17）| evidence: `plan-artifacts/plan-105/t30..t32-*.log` + `probe/`

**交付**：**handler 级 `for` 结构入候选**——`age-on-for`（`for <value> in <expr> { … }`）挂进 `age-on-widget-stmt`/`age-on-branch-stmt`；AST `ComponentStmt::For` + 六处匹配点收口（字段核对/局部名/表达式走查/内联 lambda/span/hook 铸语句；`flat_handler_assigns` 递归循环体）。语义边界（诚实）：**列表迭代未接**——`hook_stmts_lower` 与 view 内联 lambda 都返回具名 `source.age.component-unsupported`，不静默丢循环体。回归 `p105_handler_for_structure_and_named_reject`（累计 **19 例**）；census 重基线（component `fa87753e…` / package `b7f0aeac…`）；affected 全绿。

**012 定位推进**：`.Tick` 的 for 结构现在可入候选；前缀二分把剩余墙推进到 for 体内的 `var rows []str = []` / `rows.push(…)`——**handler 局部列表字面量 + push**（列表/集合承载），下一刀。另记：`on` 体顶层裸名赋值仍走旧路径（`age-view-stmt`，无 Let），需与 widget 形语句面统一。

**下一刀**：handler 局部列表字面量 + push、顶层裸名赋值统一、S1 store、T-04 接线。`next_responsible_role: work（T-02 W18）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 19 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 5c7be439d | next: work

2026-11-19 stage: work（T-02 W18：handler 顶层裸名赋值）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `c8b087aab` | task_ids: T-02（W18）| evidence: `plan-artifacts/plan-105/t33..t34-*.log` + `probe/`

**交付**：widget 形 handler 体顶层的**裸名赋值**（012 `civil_y += 1` / `y = 2`）此前只认 `.field =`。`age-view-assign-stmt` 的点前缀改为**可选**（`opt { dot(field:"dot") }`）；投影按 `dot` 字段区分——无点=裸名（`bare=true`，执行期按绑定表解析为局部量/字段遮蔽），有点=实例字段；**复合赋值读侧**同步分流（裸名 `Var(name)` / 点前缀 `self.<field>`，修掉此前 `self.y` 的 "no local named `self`"）。回归 `p105_toplevel_bare_assign`（累计 **20 例**）；census 重基线（component `0e2f0fbd…` / package `f81b7642…`）；affected 全绿（104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4）。

**012 推进**：顶层裸名赋值两形（局部/字段）现在两面通过；前缀二分把墙确定在 `.w_tick >= 4` 分支里的 `var rows []str = []` + `rows.push(…)`——**handler 局部列表字面量 + push**（集合承载面），下一刀。

**下一刀**：handler 局部列表字面量 + push、列表迭代语义、S1 store、T-04 接线。`next_responsible_role: work（T-02 W19）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 20 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: c8b087aab | next: work

2026-11-19 stage: work（T-02 W19：AV 源准备接线 + 桥实测）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `5c8610f5` | task_ids: T-02/T-04 前置 | evidence: `plan-artifacts/plan-105/`（桥装载实测输出）

**交付**：①`ace_node_desktop_window --source` 装载前按最近 `pac.at` 祖先做 `prepare_component_source`（use 空白化 + 依赖样式内联 + 类型化列表数据字段改写）；无 `pac.at` 的源原样装载（104 简单源行为不变）。②`ace_package_sfc::control_flow_kind` 收 `ComponentStmt::For` 为具名拒（"for"），与 let/if 同一条「只复放平铺赋值」纪律。

**T-04 桥装载实测（五个可投影应用）**：`015/022` → `text leaf mismatch: view tree has 2/3, AV observed 4/5`（文本叶计数口径）；`016/018` → `style reference settings_label/sub_text does not resolve`（harness `blank_style_surface` 未覆盖多行条件样式内的引用）；`013` → 文本叶求值链 `no local named self` / `field access requires a record-typed value` / `numeric receiver has no method clone`（文本叶表达式面）。三条已入缺口表，是下一阶段 T-04 的具体墙。

**下一刀**：修 015/022 文本叶计数口径 或 016/018 条件样式空白化（二选一先通一个），013 文本叶表达式面、012 局部列表/push、S1 store。`next_responsible_role: work（T-02 W20）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 21 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 5c8610f5 | next: work

2026-11-19 stage: work（T-02 W20：样式空白化覆盖行内/多行条件）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `5efe82b5` | task_ids: T-02/T-04 前置 | evidence: 单测 + 桥装载复测

**交付**：016/018 桥装载墙——`blank_style_surface` 只空白化「行首 `style `」，漏掉①行内元素体 `text "Theme" { style: settings_label }`、②多行条件样式 `style: if .a { … } else { … }`；漏掉的引用在样式声明被空白后解析不到。重写后：逐行找字符串外 `style:`，按值区间空白化（同层 `,` 截断；否则到本行最后一个 `}` 前——元素收尾括号保留；值未收口=多行条件样式，整行尾+续行按深度整行空白化，语法块整体消失）。字节长度不变。单测 +2（行内条目/多行条件），既有 5 例（含 006/007/010 真源投影+编译）保持绿。

**桥装载复测**：016/018 的 `style reference … does not resolve` 消失，墙前移到 `on` 块内结构性解析（下一刀定位）；015 同；022 仍 `text leaf mismatch: view tree has 3, AV observed 5`（文本叶计数口径）。

**下一刀**：016/018/015 的 `on` 块结构墙定位（逐 handler 二分）、022 文本叶计数口径、013 文本叶表达式面、012 局部列表/push、S1 store。`next_responsible_role: work（T-02 W21）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 22 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 5efe82b5 | next: work

2026-11-19 stage: work（T-02 W21：AV 装载原源优先）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `50bcaf04` | task_ids: T-02/T-04 前置 | evidence: 单测 7 例 + 桥装载复测

**交付**：桥装载结构性失败的根因是「先空白化样式再投影」——真实工程源的样式引用本可由本地/内联声明解析，空白化反而打碎合法结构（嵌套单行元素的条件样式）。①`NodeDesktopBridge::load` **先用原源投影**，失败才退回空白化（104「样式面不消费」语义不变）；②`blank_style_surface` 边界修正：普通条目到**第一个**同层 `}` 前（嵌套单行元素只带走自己的收尾括号）、单行条件值到本行最后一个 `}` 前、多行条件样式整行尾+续行按深度整行空白化。单测 7 例（新增 2）。

**桥装载复测（raw-first 后）**：**015/018/022 的结构错误全部消失**，只剩 `text leaf mismatch: view tree has N, AV observed N+2`（循环体文本叶展开口径：shape 只数模块作用域叶，编译侧展开循环体叶）；016 剩 `leaf-expr … no local named self`（文本叶里 `.store.*` 引用链）。两条都已定位到函数级。

**下一刀**：文本叶计数口径（循环体叶展开，015/018/022 一把通）、016 文本叶 store 引用链、013 文本叶表达式面、012 局部列表/push、S1 store。`next_responsible_role: work（T-02 W22）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 23 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: 50bcaf04 | next: work

2026-11-19 stage: work（T-02 W22：循环体文本叶计数 → 三应用桥装载）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: auto-ui `b4ef3848` | task_ids: T-02/T-04 | evidence: 回归单测 + 桥装载复测

**交付**：`text leaf mismatch: view tree has N, AV observed N+2` 的成因=编译侧按视图数据**展开循环体模板叶**，harness 的 `shape` 只数模块作用域叶。`NodeDesktopBridge::load` 在不一致时先用 `expand_view_for_leaves` 求可展开叶数：`observed == 模块叶 + 展开叶` 则以编译值为真值接受（不支持的展开形状仍具名拒）。新增永久回归 `real_u03_apps_load_through_desktop_bridge`——**015-notes / 018-book-reader / 022-kanban 经真实源准备后成功进入 AV 桥**（T-04 首批三应用）。

**剩余**：013/016 卡文本叶 `.store.*` 引用链（`leaf-expr: no local named self`，已定位到 store 成员链进入文本叶隐藏函数）；012 卡 handler 局部列表/push；S1 store 未动。

**下一刀**：文本叶 `.store.*` 链（013/016 一把通，需 store 成员在文本叶隐藏函数里的绑定）、012 局部列表/push、S1 store、AV 真开窗（T-04 收口）。`next_responsible_role: work（T-02 W23）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 24 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: b4ef3848 | next: work

2026-11-19 stage: work（T-02 W23：`.store.*` 具名拒）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `7bf36c400` | task_ids: T-02（W23）| evidence: 回归单测

**交付**：013/016 桥装载失败长期显示 `leaf-expr: no local named self`，真实原因是 `.store.*`（store 是独立单元/声明，单源组件面没有 store 绑定，S1 未接线）落进文本叶隐藏函数的间接错误。`compile_age_component` 入口扫描文本叶表达式与 handler 语句面，命中 `.store.*` 链即 `source.age.component-unsupported` 具名拒（"references need the package face … (S1)"）。回归 `p105_store_ref_is_named_reject_in_component_face`（累计 **21 例**）；无语法改动，census 4/4。

**下一阶段**：013/016 = **S1 package store 接线**（store 声明解析 + 包面 store 实例 + `.store.*` 绑定 + store 方法调用），已是唯一挡住这两个应用的大件；012 = handler 局部列表/push；随后 AV 真开窗 / A2Vue。

**下一刀**：S1 store（语法→投影→包面绑定，分刀推进）、012 局部列表、AV 真开窗。`next_responsible_role: work（T-02 W24）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 25 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: 7bf36c400 | next: work

2026-11-19 stage: work（T-02 W24：store 声明入语法）| PLAN-105 r1 | outcome: pass（本刀完成；T-02 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-lang` @ `plan-105-dev` | code_commit: auto-lang `d8c75a7d8` | task_ids: T-02（W24，S1 首刀）| evidence: 回归单测 + census

**交付（S1 首刀）**：`store <Name> { model/msg/on/design/computed/watch/timer }` 声明入语法（`age-store-item` + 上下文名 `age-store-head`，`store` 是 ident token 不新造保留字），挂进 age-component/age-package 的 item 分派；模块投影入口识别 store 声明即 `source.age.component-unsupported` 具名拒（"parsed; semantics not wired yet (S1)"），绝不静默降级成同名记录类型或跳过行为。回归 `p105_store_decl_is_named_pending`（累计 **22 例**）；census 重基线（component `147373b7…` / package `e50580b5…`）；affected 全绿（104 3/3、1072 19/19、980 39/39、981 248/248、979 24/24、census 4/4）。

**下一刀**：store 语义（共享实例 + `.store.*` 绑定 + store 方法调用 + 包面多单元 store 装载）——013/016 由此解锁；随后 012 局部列表/push、AV 真开窗。`next_responsible_role: work（T-02 W25）`

stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 26 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-lang | commit: d8c75a7d8 | next: work


stage: goal | plan_id: PLAN-105 | plan_revision: r1 | round: 27 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-105/auto-ui | commit: b4ef3848 | next: work

2026-11-19 stage: work（T-02 W25 / T-04 部分验收：三应用真开 Iced 窗）| PLAN-105 r1 | outcome: pass（本刀为验收证据；T-02/T-04 未整体完成）| worktree: `D:/autostack/.wt/ui-105/auto-ui` @ `plan-105-dev` | code_commit: 无（证据轮，代码同上）| task_ids: T-04（部分）| evidence: `plan-artifacts/plan-105/av-015.json`、`av-018.json`、`av-022.json`

**证据（真实 AV/Iced 窗 + 证据 JSON）**：用 `ace_node_desktop_window --source <app.at>`（`--source` 装载前已接 `prepare_component_source`，见 W19）逐例真开窗：
- `015-notes`：frames_rendered=2、errors=0、sessions=1、buttons=3、field_names=3；sha256 前 16 `2F46EB008526E99F`。
- `018-book-reader`：frames_rendered=2、errors=0、sessions=1、buttons=1、field_names=2；sha256 前 16 `7CC5ED29069260B3`。
- `022-kanban`：frames_rendered=3、errors=0、sessions=1、buttons=1、field_names=3；sha256 前 16 `CA0452C42593006E`。
命令（可复跑）：`ace_node_desktop_window.exe --source <repo>/examples/ui/<app>/src/front/app.at --evidence <out.json> --instances 1 --hold-ms 1500 --min-frames 2`。

**边界（诚实）**：脚本化点击（`--script "3:click:0:0"`）在 015 上被具名拒——`ToggleDarkMode`/`NewNote` 的 handler 体含调用/let/控制流（"a directed event must not run a partial body"），需 S1 store（`.store.*`）与 handler 语义接线；静态装载与渲染已通。

**下一刀**：S1 store 语义（013/016 + 015/018 的可点击事件）、012 局部列表/push、再跑三应用点击脚本与 013/016 装载。`next_responsible_role: work（T-02 W25）`
## 10. 下游交接

本计划按 U03 独立完成。AV 收据与 1071 已交范围及其他 U 组结果汇合后供 W1 签发、1068/T-04 和 985/B 接收。A2Vue 收据供 1072 与产品 CLI 消费。A2R/AC 后续计划沿同一源码、Widgets 和独立预期补验。跨仓 golden 基线维护按已有 PARITY 路线归档。

2026-11-19 停放与恢复：按用户裁定暂停过本计划开发；104 已交付并合入 master（ui `5a862d46` / lang `2fb094483`）后，已按裁定从 master 重建工作树（不共享 104 分支）并完成 T-02 W1。恢复/续做时先读 §9 的 W1 收据与 `plan-artifacts/plan-105/t02-gap-map.md`（S1–S10 缺口与顺序在案）。
