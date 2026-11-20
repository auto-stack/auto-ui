---
plan_id: PLAN-124
status: archived
completion_kind: delivered
feature_name: AutoUI U03 路由与页面挂载 AV 和 A2Vue
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 1
current_step: 6
total_steps: 6
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/ace-route-page-loading.md]
touched_goals: []
---

# PLAN-124 AutoUI U03 路由与页面挂载

## 0. 变更摘要

2026-11-20 从 [105 r2](../105-autoui-u03-local-interactive-apps-av-vue.md) 移交 018 Book Reader 和 022 Kanban 公用的 routes/outlet 实现、独立回归与双端能力收据。105 继续交两个原应用的完整结果，包括生产页面数据、动态列表、后端 API、原交互和整例生命周期。

## 1. 目标

使用原路由形状在 AV/Iced 和 A2Vue/浏览器实际挂载页面。验模块别名、首屏、链接和带参导航、返回、目标页状态及关闭重开。同时交一条可单独消费的正式能力收据，供 105 回接原应用。

## 2. 架构方案

已有 AGE 投影器把路由声明提取到 route 成员槽位，AutoUI 已有 legacy VM 页面装载与参数/历史行为可作对照。沿受检声明、页面模块解析、目标页实例及各端渲染链完成 ACE 消费。使用同一份路由语义和原模块名；将路由专属代码和回归放在独立模块，复用既有实例和事件身份。

124 持路由声明及页面挂载的实现写面。105 持动态列表、Store/后端、时钟和应用整合。预期两线在宿主和生成器的接点以小范围接线汇合。若发现必须改同一投影/初始化函数或源准备分支，T-01 逐函数核对并定一个 owner，独立模块与原型验证可并行；接点按已审提交接续。121 等其他实际消费者复用同一能力。

## 3. 技术栈

观察入口为 AutoLang 的 experimental/ace/grammars/auto-source.at 中 age-routes-member、age-route-def，及 experimental/ace/crates/core/src/source/cst/project.rs 的 route 槽位投影。AutoUI 从 NodeDesktopBridge::load_package 和 generate_package_sfc 接实际 AV 宿主与 Vue 产品线。原消费者见 [018](../../../examples/ui/018-book-reader/src/front/app.at)、[022](../../../examples/ui/022-kanban/src/front/app.at) 及各自 pages/。

两仓实施树拟使用 D:/autostack/.wt/ui-124/{auto-ui,auto-lang}，每仓分支 plan-124-dev，从各自 master 起步。依赖仓只读并钉版本。

## 4. 需求分析与背景调查

用户授权拆分 105 中可并行的剩余范围。2026-11-20 的 /goal 完成计划124 授权本计划按 r1 实施、独立复审、有界修复、合并和归档。核对基线为 AutoUI master 448ed614、AutoLang master 2c6120d8b、105 实施树收据 W66–W68（AutoUI 3627f71c、AutoLang 9be4d0042）。105 的 018 和 022 已打开外壳并有部分点击证据，收据仍缺 routes/outlet 的页面挂载。路由声明与 105 在途的列表值、循环叶及 Store 改写是不同能力。

参考 [1071 U03 原矩阵](../../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[1072 Vue 正式交付](../../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)、[现有 VM 页面语义](../../specs/runtime/vm-route-page-loading.md) 和 crates/auto-ui-runtime/src/ui_tests/plan069_route_dispatch_tests.rs。VM 的已有证据用于核预期。ACE 两端按 §6 各自实跑。

## 5. 详细设计

T-01 从主线冻结一个两页、独立标量状态的原路由形状回归。沿用 018 的模块/路径参数和返回形状，以及 022 的有外壳路由页。固定页面内容仅用于验证路由，原应用仍按 105 的真实页面和数据验收。读回当前路径、目标页标记和页内状态，区分根壳渲染、挂载错误页、错误事件归属和迟到回调。

### 105 回接清单

| 原应用 | 原路径与页面文件 | 交接 |
|---|---|---|
| [018 Book Reader](../../../examples/ui/018-book-reader/src/front/app.at) | `/` → `pages/bookshelf.at`，`/book/:id` → `pages/book_detail.at`，`/book/:id/chapter/:ch` → `pages/reading.at`，`/settings` → `pages/settings.at` | 用同一模块名和路由表接入。夹具覆盖首屏、详情参数、页面字段组装路径、返回和设置页。另用原章节路径验证包内导入和双参数，原章节页由 105 带生产数据复跑。 |
| [022 Kanban](../../../examples/ui/022-kanban/src/front/app.at) | `/` → `pages/board.at` | 从嵌套 outlet 挂载真实看板页。124 验同形壳与独立标量状态，105 校验列表、拖放、Store 和后端。 |

105 接续时，AV 使用路由感知的 `NodeDesktopBridge::load_package` 和原页面单元。Vue 用 `AUTO_UI_ACE_VUE=1` 的项目路径发布 App、页面与 router。124 持 routes、路由专用标签分支和宿主挂载；105 持通用括号属性、style/f-string 源准备及 Store/列表。对两条分支执行只读合流模拟，确认常规 SFC 入口、语法 alt 与语法指纹需在 105 同步主线后接续。105 已有 `3ff086991` 的普通 link 解析；合流保留路由专用分支和必要的其他 link 形，按合流语法重新生成两 AGE 指纹。当前两树互不写入。

### 规范增量

| ID | 动作与目标 | 规则 | 验收 |
|---|---|---|---|
| SD-01 | 新增 docs/specs/runtime/ace-route-page-loading.md | 沉淀实测的 ACE 模块挂载、参数、历史、页面状态和错误边界；既有 VM 合同作为语义对照。 | AC-01、AC-02 |

## 6. 测试设计

下列用例由 T-01 固定输入、T-02 至 T-04 接通并执行。使用真实 Iced 窗口和 1072 的 Vue 构建/浏览器入口，永久测试覆盖源投影、宿主接线与失败分支。

| 用例 | 观察结果 |
|---|---|
| V-01 | 首屏及带参导航挂载正确模块，页面事件读取相同参数和值，返回恢复正确历史条目。独立静态两页可从主线开发和复验。 |
| V-02 | 快速切换或关闭重开后，原页面事件和迟到结果保持原实例归属；新页面从自己的状态运行。 |
| V-03 | 缺页、坏模块或路径不匹配给出可定位的各自诊断；有效相邻路由及无路由单页继续工作。 |
| V-04 | 将路由能力接回 018/022 的原路径，提供可复用的子项收据和后续外部依赖清单。整例回放由 105 在合入后按原矩阵完成。 |

执行基线回归包括 AutoUI 的 ace_node_desktop、plan980_ace_vue_tool_wiring。改 AutoLang 后，冻结提交跑相关固定组，候选为 cargo t ace-source ace-av ace-a2x，触及语法或语义再并入 ace-syntax 或 ace-semantics。实际组按写面确认。新增用例的实际命令、计时和结果见 §9。冻结后的 AutoLang 受影响固定组由 merge 阶段复验。

## 7. 验收标准

| ID | owner_stage | 完成条件 |
|---|---|---|
| AC-01 | work | V-01～V-03 在两个生产入口通过，读回路径、目标页、业务状态和关闭终态。 |
| AC-02 | review | 核页面导入与发布、有效相邻路由、错误页、实例隔离，以及与 105 的单写边界；核 V-04 原源回接范围。 |
| AC-03 | merge | 落主接线和规范，发布绑定提交的 AV/A2Vue 收据、原路径映射及 105 消费方式。 |

## 8. 执行步骤

如当前工具支持原生 TODO，开工前登记下列任务 ID。

- [x] T-01 冻结路由夹具和接线归属
  - owner_stage: work。AC-01、AC-02。从 master 固定实际源形、既有路由入口、两仓路径和可重放的触面。
- [x] T-02 交付受检路由描述与页面解析
  - owner_stage: work。AC-01。复用已有 route 投影，登记明确的源模块与错误诊断。
- [x] T-03 验证 AV 页面实例和历史
  - owner_stage: work。AC-01。执行 V-01～V-03，记录真实窗口、导航、参数、错误及关闭。
- [x] T-04 验证 A2Vue 路由并准备原应用交接
  - owner_stage: work。AC-01、AC-02。实际构建并运行浏览器，记录同源结果和 V-04 可回接边界。
- [x] T-05 独立复审与冻结
  - owner_stage: review。AC-02。复验两端、单页兼容、失败用例及实现归属。
- [x] T-06 落地并交 105 消费
  - owner_stage: merge。AC-03。版本固定后按仓规沉淀、归档及清理。

## 9. 复审记录

2026-11-20 stage: new | PLAN-124 r1 | outcome: pass（计划及写面核对） | task_ids: T-01..T-06 | acceptance_ids: AC-01..03 | next: work/T-01。实测从实施阶段开始。

2026-11-20 stage: work | PLAN-124 r1 | outcome: in_progress | task_ids: T-01、T-02 | worktree: D:/autostack/.wt/ui-124/auto-ui @ plan-124-dev | base_commit: 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd | dependency_revisions: auto-lang 2c6120d8b33ead58e087553a3a50391f3d853a21、auto-down 72b097006f34db3a871449f10e213820d51abef6、auto-web 26d085f7e54ea20c6997edf2dc23eef0cb57faf6。按 /goal 授权全链路。初始基线 cargo check -p auto-ui-runtime --features ace-bridge --offline 退出 0（93 s）。

从 AGE ComponentNode.members 的 route 行提取模式和原模块别名。包闭包按 entry 所属包的模块 ID 查询。路径命中坏模块和路径未命中各有单独证据，需经实际宿主校验。原 018 的 /、/book/:id、/settings 与 022 嵌套 outlet 已写入独立夹具；预期 17/42 分别注入原页面实例，返回用历史条目，关闭后按新实例观测。105 的源准备、store、列表和同名 SFC 前门预处理各由 105 持有；124 以独立路由模块承接，接点在工作树提交前另核。

2026-11-20 stage: work | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | task_ids: T-01、T-02 | code_commit: AutoUI e72a8b2b6015a4a9f273dcfc40b62a418342b062、AutoLang 44673cc102402566d3f4a1c20233e9ec1d219548。初次 Reader 源投影在保留字 link 处失败，Kanban 与无路由单页对照通过。新语法分支接入后，冻结版内置投影回归 2/2（执行 0.04 s），AGE 语法指纹四例 4/4（0.36 s）。仅 AgeComponent 和 AgePackage 改动，原 13 邻居保持原指纹。目标页业务状态用 AV 状态行或 Vue defineExpose 读回，避免把 105 持有的包面整数文本叶缺口带进路由夹具。原路径清单与合流接点见 §5。第一次冻结版并行链接触发 Windows 1455，现串行复验五个受影响测试目标。

2026-11-20 stage: work | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass | task_ids: T-01～T-04 | worktree_path: D:/autostack/.wt/ui-124/{auto-ui,auto-lang} | branch: plan-124-dev | base_commit: AutoUI 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd、AutoLang 2c6120d8b33ead58e087553a3a50391f3d853a21 | code_commit: AutoUI e72a8b2b6015a4a9f273dcfc40b62a418342b062、AutoLang 44673cc102402566d3f4a1c20233e9ec1d219548 | dependency_revisions: auto-down 72b097006f34db3a871449f10e213820d51abef6、auto-web 26d085f7e54ea20c6997edf2dc23eef0cb57faf6 | blockers: 无 work-owned blocker | next_responsible_role: review

AC-01：Reader 实测从 / 经页面整数字段组装地址到 /book/17、AV/浏览器 Add 令 total=17，再经壳链接到 /book/42，书号 42 且 total=0。返回 /book/17 分配新实例。AV 定向旧 session 点击被丢弃，浏览器旧 Back 元素点击未改变新路由。三种失败是缺模块 /missing、无主件的 /broken、路径不匹配 /unknown，随后 Settings 仍挂载。关窗清掉当前页，重开首屏分配新实例，最终所有 native sessions 关闭。Kanban 在嵌套 outlet 中挂载 Board，关闭前后新实例各经一次 Move 从 0 到 1。Iced reader 54 帧、kanban 5 帧。Vue 运行时零 pageerror。AC-02 所需 018/022 原路径映射、依赖清单及与 105 的接点见 §5，T-05 待复审。AC-03、T-06 与固定组回归待 merge，尚未落地。

| 会话测试范围 | 结果 | 准备 / 执行 |
|---|---|---|
| AutoUI cargo test -j1 --offline，ace-bridge/ace-vue，ace_node_desktop、plan124_routes、plan980_package_sfc、plan124_ace_vue_routes、plan980_ace_vue_tool_wiring | 目标分别 11/11、2/2、8/8、1/1、1/1。两个既有 Iced 排窗与两个既有浏览器用例各以未排窗分支返回，其余断言实跑。 | 串行准备含锁等待 169 s，执行分别 1.06 s、0.47 s、0.11 s、0.77 s、2.78 s。发现与清理未单独计时。初次并行链接返回 E0786/Windows 1455。 |
| AutoUI ui::ace_route::tests | 2/2，另有 2615 个未选 | 准备 102 s，执行 0.04 s。 |
| AutoLang plan981_grammar_scope（已编译 ace_core 上同源码 rustc --test） | 4/4，两个 AGE 指纹重冻，13 个相邻指纹平齐 | 准备与运行合计 1.23 s，测试执行 0.36 s。 |
| Iced reader、kanban | 2/2，准备 ui980 后使用真实窗口 | Windows debug 入口最终构建 77 s。默认栈的初次 Reader 进程溢出，复跑构建入口用 cargo rustc --example ace_node_desktop_window -- -C link-arg=/STACK:33554432。执行与退出 3.684 s、1.365 s。 |
| Vue Reader、Kanban | auto-ui-tool generate_ui → SFC/esbuild debug 准备 → Playwright Chromium(msedge)，2/2 | 生成 0.941/0.565 s，构建 0.466/0.159 s，执行与关闭 4.599/1.243 s。初次 Playwright 默认 Chromium 二进制缺席，改用本机 Edge。 |

新增用例逐例计时（debug/test，含进程启动和清理）：`project_reader_aliases_parameters_and_independent_errors` 0.188 s，核别名、参数、缺/坏模块和相邻路由；`project_kanban_shell_and_legacy_single_page` 0.107 s，核嵌套 outlet 与旧单页；`av_package_routes_preserve_owner_across_navigation_and_reopen` 0.443 s，核旧回调、状态、历史和重新挂载；`route_sfc_publishes_pages_and_local_diagnostics` 0.117 s，核浏览器发射和局部错误；`ace_route_project_emits_and_refreshes_the_full_page_set` 0.596 s，核全量发布、补齐缺页的增量更新与幂等；新增 Iced/浏览器用例各例耗时见上表，全部低于 10 s。测试使用已准备且带 Rust 编译器指纹的 ui980 和已编译浏览器包。

收据（本计划临时证据目录 `D:/autostack/plan-artifacts/plan-124/`，SHA256 前 16 位）：`serial-ui-targets.log` D30AEA5EEDAEE31F，`final-native-reader.json` 5780BA64A541357D，`final-native-kanban.json` 2C5A8D9568484BD7，`final-web-verdict.log` 06FCEE5A0098220C，`route-census.log` A433E2EB3F74F5B5，`final-case-timings.log` 20D19D242B2C5C21。自有 Cargo 解析锁缓存漂移已撤回，两实施树 clean。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: e72a8b2b6015a4a9f273dcfc40b62a418342b062 / 44673cc102402566d3f4a1c20233e9ec1d219548 | next: review | authorization: 用户 /goal 完成计划124 授权 r1 实施、复审、有界修复、合并与归档。

2026-11-20 stage: review | review_scope: final | reviewer_identity: 当前会话按冻结工作树复核 | limitations: 同会话复审，另建输入而无独立会话 | plan_id: PLAN-124 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-124/{auto-ui,auto-lang} | branch: plan-124-dev | reviewed_commit: AutoUI e72a8b2b6015a4a9f273dcfc40b62a418342b062 / AutoLang 44673cc102402566d3f4a1c20233e9ec1d219548 | base_commit: 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd / 2c6120d8b33ead58e087553a3a50391f3d853a21 | acceptance_results: AC-01 已验原夹具、导入组合待证，AC-02 fail，AC-03 pending | pending_closeout_gates: AutoLang 冻结固定组、合并、归档、清理 | next: work

F-124-01（P1，T-02～T-05）：合法的目标页模块可导入独立包内帮助函数。复审输入在临时证据 `review-holdout/src/front` 下，路径 `/book/:id/chapter/:ch` → `reading.at`，同包 `helpers.at` 导出 triple，目标 handler 读两个参数。经真实 auto-ui-tool generate_ui（生成 21 文件，report 0.719 s）却发射错误组件，诊断为 `source.unsupported: use is not part of this source profile`（error-1 SHA256 3BD743013AE794F84）。对照坏页得到独立的 `source.age.structure`（error-2 SHA256 4D7F3BAC8E635575）。生产入口调用的组件单文件投影在到达包绑定器前误拒 use；AV 路由表走同一准入点。修复需先让路由壳、目标页与 emitter 走包面投影，保留 import 的实际绑定，并覆盖 helper 实参内的 route 参数。关闭判据是在两端 /book/23/chapter/8 得到 triple(23)+8=77，坏页隔离，平坦无导入用例继续通过。在找到阻断后未启动统一回归。复审未添加永久例，复用冻结原夹具证据。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: e72a8b2b6015a4a9f273dcfc40b62a418342b062 / 44673cc102402566d3f4a1c20233e9ec1d219548 | next: work

2026-11-20 stage: work | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass | task_ids: T-02～T-04 | worktree: D:/autostack/.wt/ui-124/{auto-ui,auto-lang} | branch: plan-124-dev | code_commit: AutoUI 860f44458cab6c131aa11a82acb7c23f3c0d8082 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756 | dependency_revisions: auto-down 72b097006f34db3a871449f10e213820d51abef6、auto-web 26d085f7e54ea20c6997edf2dc23eef0cb57faf6 | next_responsible_role: review

F-124-01：包面提供候选及导入声明，重写只针对路由宿主命令和参数读取，实际 helper 解析仍通过包绑定。复合表达式先前在 Vue 将 t0（69）误写入，改为使用 976 fragment 的 expr（t1=77）。原留出输入还发现 page 非扁平包首个模块时局部 span 无法命中 HIR，按编译产物记录的 entry offset 查找。永久 Reader 夹具令 `chapter_math` 排在 `reading` 前；`/book/23/chapter/8` 的真实 AV 和 Vue App 均读回 77。旁路无路由、缺页、坏页、历史、迟到和关闭控制仍通过。无新增测试函数，改变了现有用例的组合输入。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: 860f44458cab6c131aa11a82acb7c23f3c0d8082 / 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: review

2026-11-20 stage: review | review_scope: final | reviewer_identity: 当前会话重建冻结证据并复放原留出输入 | limitations: 同会话复审 | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-124/{auto-ui,auto-lang} | branch: plan-124-dev | reviewed_commit: AutoUI 860f44458cab6c131aa11a82acb7c23f3c0d8082 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756 | base_commit: 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd / 2c6120d8b33ead58e087553a3a50391f3d853a21 | spec_inputs: docs/specs/runtime/ace-route-page-loading.md SHA256 8A0ECE48363C93E9B907FF3BCBC0DD3308F0AEB1FDD5825D4E8A4AF8AC15C5F0 | acceptance_results: AC-01 pass、AC-02 pass、AC-03 merge pending | next: merge

原留出输入经真实 `generate_ui` 再次发布，阅读路由现指向 `reading.vue`，导入的 `helpers.triple` 已出现在受检发射，表达到 t1，坏源仍只生成 `ace.route.page` 诊断组件。对照原应用的五个路径与目标文件，105 写自己的源准备和业务层，124 写路由准入、实例和发布。拟规范与实际路径、历史重建和失败隔离一致。T-05 完成。merge 尚需在冻结版本跑 AutoLang `ace-source ace-syntax ace-av`，再按交付检查落地、归档、清理。A2Vue 消费通过 AutoUI SFC 与工具及真实浏览器覆盖，未改 AutoLang A2X 发射器。

| 回归范围 | 结果及执行时间 | 准备与边界 |
|---|---|---|
| plan124_routes、plan980_package_sfc、plan124_ace_vue_routes | 2/2（0.64 s）、8/8（0.09 s）、1/1（0.80 s） | 串行 debug/test 准备 124 s，旧 SFC 两个排窗用例未选。 |
| 已绑定同源的 Iced Reader/Kanban | 2/2，56/4 帧，3.818/1.219 s；新章节 score=77 | Iced 入口准备 134 s，最后的 span 改动仅影响 Vue，复用了窗口二进制，用最新夹具重跑。 |
| 真实 Chromium Reader/Kanban | 2/2，执行至退出 2.204/0.956 s，零 pageerror | CLI 生成 1.261/0.551 s，SFC debug 构建 1.028/0.157 s。 |
| AutoLang grammar census | 4/4，0.35 s，只有本轮 AgePackage 指纹更新为 65d176e3… | 复用同源码编译的 ace_core，独立准备与执行共 2.51 s。 |

改动用例按精确过滤复跑：AV 路由和新章节 0.552 s，SFC 页面发射 0.149 s，Vue 工具全量与增量更新 1.277 s；其余不受影响的既有 AutoUI 门证据复用前轮。真实窗与 Chromium 用例的执行时间见上表，均低于 10 s。证据 SHA256 前 16 位：`repair-import-offset-r2.log` DBDD5D59B23C7309、`repair-offset-web.log` 6920017E8C844371、`repair-offset-native.log` C07CB4EC41B9FEBB、`re-review-holdout-final.log` AB5C40AF9B03A5D9。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: 860f44458cab6c131aa11a82acb7c23f3c0d8082 / 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: merge

2026-11-20 stage: merge | checkpoint: prepared | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | reviewed_commit: AutoUI 860f44458cab6c131aa11a82acb7c23f3c0d8082 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756 | delivery_candidate: AutoUI be62b4d952e00f1d29d7bb0676985de5e22801b7 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756

AutoUI master 新增的 d4d9ee4e 仅改 PLAN-091 文档。计划树 rebase 后 range-diff 3/3 等价，e72a8b2b→633cba87、df75ccd5→7ddddf43、860f4445→be62b4d9，测试输入与 SD-01 哈希不变。AutoLang master 仍为 2c6120d8b。四树预检 wt-guard clean。临时证据根已在 retention 登记。两主检出含其他计划和设计文档的未提交状态，AutoUI 另有预存 tmp/checklinks.py，后续仅处理 PLAN-124 所属文件。固定组 `ace-source ace-syntax ace-av` 等待单机资源，观察到 `D:/autostack/.wt/lang-1046/auto-lang` 的 conformance 进程 PID 15072，命令 `cargo test -p ace-aac-syntax-conformance --test conformance aac1046_il_phases -- --nocapture` 测试执行中。尚未开始固定组准备、运行或合并。

2026-11-20 stage: merge | checkpoint: prepared | plan_id: PLAN-124 | plan_revision: r1 | outcome: blocked | delivery_candidate: AutoUI be62b4d952e00f1d29d7bb0676985de5e22801b7 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: merge when test slot is available

统一门仅完成只读选组，`ace-source`（ace-core）、`ace-syntax`（语法 runtime/grammar/core/artifacts）、`ace-av`（AV/AV-core）的三条 nextest invocation 都在 `experimental/ace`，组配置 SHA256 `8bb5c62196c720dbd4d78c3f15246b7add4ff1ed834d4ae69b4912dcf39f295a`。初次带 checkout 外 CARGO_TARGET_DIR 的选组被脚本拒，因此正式准备使用原 checkout 默认 target。随后数次占用复查确认 PLAN-1046 重启 `aac1046_il_phases`，最新 conformance PID 81348。PLAN-1083 正构建 `ace-semantics-store --bin sema-worker`，runner PID 64940。两次连续占用观察为 52.136 s / 53.863 s，均无可用窗口。按单机测试规则尚未开始准备、执行、快进合并、归档或清理。AutoLang 批回归 receipt 为 2026-11-20T00:00Z，PLAN-124 不触发 %5，当前此门不到期。无新增用例或执行结果，前轮 AC-01/02 证据继续有效。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 5 | from: merge | to: merge | status: reviewed | outcome: waiting | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: be62b4d952e00f1d29d7bb0676985de5e22801b7 / 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: user

2026-11-20 stage: merge | checkpoint: prepared | plan_id: PLAN-124 | plan_revision: r1 | outcome: waiting | delivery_candidate: AutoUI be62b4d952e00f1d29d7bb0676985de5e22801b7 / AutoLang 8741c81c350f4ad80de3fbd5414c690b4cd94756

占用进程结束且复查空闲后，已启动三组 `--prepare-only`。查到受选工具清单 `scripts/test-governance/toolchains.json` 要求 abi、a2x、store-gc worker 使用 release，text-runtime、data-runtime 等使用 runtime-release，准备器无 debug 覆盖入口。现行测试纪律不允许日常合并门准备隐式触发 release。在第一条 ace-core nextest archive 编译阶段终止本计划启动的准备进程（53.294 s，exit 1）。未触发 release 工具构建，未取得完整 archive 或准备清单。请求用户确认本次是否允许固定组按仓库清单准备测试依赖。独立 ace-syntax 组无工具准备项，在机器空闲时仍可运行。

独立 ace-syntax 等待机器窗口未能开始，末次核实 PLAN-1046 的 conformance PID 26712 仍在运行。本轮未执行固定组测试，无新增用例。交付提交和既有复审证据仍有效。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 6 | from: merge | to: merge | status: reviewed | outcome: waiting | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: be62b4d952e00f1d29d7bb0676985de5e22801b7 / 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: user

2026-11-20 stage: merge | checkpoint: ace-syntax-prepared | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | frozen_lang: 8741c81c350f4ad80de3fbd5414c690b4cd94756

机器空闲时运行 `cargo t ace-syntax --prepare-only`，exit 0，41.503 s。只准备语法组 debug/test 归档，无额外工具。工作树内准备清单 `target/test-groups/prepared/9357318b88fab6fe/prepared.json` SHA256 64C229E57C204FD478B27B23D3C6F7ED71B3106CC85DDF2C74C26B7297CEE611，archive-0.tar.zst SHA256 3BC2DE05017021B254F6D14D036D0565EDF7E045B1E1C2393C77C66B6B577ED9（11,069,647 bytes）。后续直接 `cargo t ace-syntax` 使用此清单。其他两组的 release 测试依赖仍待用户确认。

2026-11-20 stage: merge | checkpoint: ace-syntax-passed | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass for ace-syntax | frozen_lang: 8741c81c350f4ad80de3fbd5414c690b4cd94756

PLAN-1046 与 PLAN-1083 空出窗口后，`cargo t ace-syntax` 复用本轮 debug/test 归档，exit 0，5.088 s，选中 157、执行 157、未运行 0、错误 0。结果挂于 `D:/autostack/plan-artifacts/plan-124/merge-ace-syntax-results.json`，SHA256 FF3D3F48F94ED0E31F8FBDA2A1AE6B5EF1389942B80FAD45BAD2EED1FCC1CC52，prepared source state 9357318b88fab6fe4805c26cfa240c4ea76a5182a70ca17175c5d85da21e801e。无新增用例。固定组只剩 ace-source、ace-av，随后是两仓合并、归档及清理。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 7 | from: merge | to: merge | status: reviewed | outcome: waiting | worktree: D:/autostack/.wt/ui-124/auto-ui | commit: be62b4d952e00f1d29d7bb0676985de5e22801b7 / 8741c81c350f4ad80de3fbd5414c690b4cd94756 | next: user

2026-11-20 stage: merge | checkpoint: release-test-tools-authorized | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | frozen_lang: 8741c81c350f4ad80de3fbd5414c690b4cd94756

用户明确授权本计划按仓库清单构建 ace-source、ace-av 的 release/runtime-release 测试依赖。测试归档仍使用 test profile。现有 ace-syntax 157/157 的执行凭证继续有效。剩余两组将在机器空闲时按原固定组入口准备并执行。

2026-11-20 stage: merge | checkpoint: spec-digest-corrected | plan_id: PLAN-124 | plan_revision: r1 | outcome: verified

以 Git 对象重新核验复审提交 860f44458 和交付提交 be62b4d9 中的规范。两者的 docs/specs/runtime/ace-route-page-loading.md 字节完全一致，实际 SHA256 为 54C98E0C78E176940DDFB42578C352208CEB7E1807491ADF7A94AF7D12E9A6CD。此值更正上方复审收据误记的摘要。正文、交付补丁和既有验收不变。

2026-11-20 stage: merge | checkpoint: lang-1083-rebase | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | frozen_ui: be62b4d952e00f1d29d7bb0676985de5e22801b7 | rebased_lang: 21a0f70f9292082c4eef578d0df0f374c337738b

等待机器窗口 583.91 s 后开始旧基线 ace-source/ace-av 准备。两个 test profile 归档成功、首批 debug 工具完成、ABI release 准备进行中时，发现 PLAN-1083 已快进 AutoLang master 至 27087938db793bfa3dcd732b97c774efbd2766f2，涉及 124 同一 CST 投影、语法及指纹用例。终止本计划准备器 PID 4968 及其子进程，准备耗时 639.66 s，exit 1，未启动测试。旧基线的 157/157 保留为历史结果，三组都将按新基线重新验证。

AutoLang 计划树 rebase master 无文本冲突。`git range-diff` 2/2 等价（44673cc10→b6edb7ffa、8741c81c3→21a0f70f9），`git diff --check` 通过。PLAN-1083 冻结的 Collections、Aggregates、Scheduler 三项与本计划的两个 AGE 项分离，需验证合成语法及实际双端行为。AutoUI 提交与规范摘要不变。

2026-11-20 stage: merge | checkpoint: lang-1083-grammar-verified | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | rebased_lang: dcd4fdafd412be911dc44ba1fee911a916d9c198

合成的 21a0f70f9 上 `cargo test --locked --offline -p ace-core --test conformance --no-run` 准备 25.630 s，独立执行 `plan981_grammar_scope` 4/4，841 筛出，测试 0.35 s，调用 0.488 s，exit 0，机器等待 211.458 s。`rebase-grammar-test.log` SHA256 BA6CE13643D8FA54ADE2DE11D389DEE697B70514F0931D50C8E07DCF725E5A36。

主线接着追加 1083 归档收据至 92b3f1842。再 rebase 至 dcd4fdafd，`range-diff` 2/2 等价，整树仅 1083 归档文档改变，复用 4/4 指纹结果。当前针对新版依赖重建 AV 与 A2Vue 入口，复验后按合成版本跑三组固定回归。

2026-11-20 stage: review | checkpoint: lang-1083-dual-host-replay | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass for AC-01/02 | ui: be62b4d952e00f1d29d7bb0676985de5e22801b7 | lang: dcd4fdafd412be911dc44ba1fee911a916d9c198

复核 1083 合成后的入口与同会话实证：AV example debug 重建 131.878 s；auto-ui-tool ace-vue debug 重建 116.984 s；均 exit 0。Iced 真窗 Reader 51 帧、3.805 s；Kanban 5 帧、1.263 s。实测 Reader 路由及返回、错误隔离、章节双参和 helper 后 score=77、关闭后无活动会话。Kanban 关闭重开，两次页面独立状态均为 moved=1。两用例调用合计 5.540 s，exit 0。

Edge/Playwright Reader 1.907 s、Kanban 0.994 s，均 `PLAN124_BROWSER_OK`。生成 1.575/0.608 s、打包 1.941/0.167 s、调用合计 7.613 s，exit 0。机器等待：AV 构建 73.610 s，工具构建 21.100 s，真窗与浏览器回放 154.660 s。没有新增用例、没有改 Code/Spec。固定组仍独立待验。

证据 SHA256 rebase-native-verdict.log 537A4863EDAB47067063BA0A654ACD317386DE6B8EEAD96DC342B9C672DACAB1；rebase-native-reader.json 35452CDB7CF769E8B492C36E48521FA3FDEF94ABEEB3830E74F7B8DB34221A78；rebase-native-kanban.json 2C5A8D9568484BD74CB159F1CEAE13D76CAB6FE94065CF84901F5483F7166E93；rebase-web-verdict.log 794455691AF96511EEC00FB9695EA39B4277464594C00B1E023BCD363DC6045D；rebase-browser-reader.log 90E252763F715C5D4FF98E0DEE89960FAD7366A3274B88C9A86A7F47FB882C2B；rebase-browser-kanban.log AA025738B015C077B30CF10A5CD0116FAF64E06C30C78749978C43BF46032AF4。

2026-11-20 stage: merge | checkpoint: post1083-fixed-groups-prepared | plan_id: PLAN-124 | plan_revision: r1 | outcome: in_progress | frozen_lang: dcd4fdafd412be911dc44ba1fee911a916d9c198

机器等待 241.13 s 后准备三组固定回归，exit 0，准备调用 805.295 s，准备器记录 804.70 s。三组 nextest archive 均用 test profile；用户授权的原配置工具准备完成。manifest SHA256 AC451CC1977239E4E5778BC79609B8D36855141C0A644BF503401E99E579857F，source_state ef73b16648d4821fe806c9b00f7a8775021eb0b17037a6682a15aed88a1e0bbb，固定组选组配置 8bb5c62196c720dbd4d78c3f15246b7add4ff1ed834d4ae69b4912dcf39f295a。归档 SHA256 依次为 d105b7909d146d836554f2a214a31830a32f38d74c094065985bbb9680603009、5e2f20128a5b9c20408c1d9655e2583c8097fe7a5548346a0fc47162cb268965、9cb323fc9eb7080f16b1e428b226d2dd367b3c8388ce2dc1f17f27303e590ee9。

2026-11-20 stage: merge | checkpoint: post1083-fixed-groups-executed | plan_id: PLAN-124 | plan_revision: r1 | outcome: triage | frozen_lang: dcd4fdafd412be911dc44ba1fee911a916d9c198

机器等待 156.75 s，使用冻结归档执行，调用 90.737 s，exit 1。`ace-source` 930/931，另有 1 跳过，13.030 s；scalar_source::deep_not_chain_reaches_checked_bundle 发生 Windows 0xc00000fd 线程栈溢出。`ace-syntax` 157/157，通过。`ace-av` 1481/1484，另有 23 跳过，65.884 s；两个已登记 D-1098-01 的 host_console 断言失败，以及 sys_d01_d06_1064::v02_d06_spawn_with_output_av 的执行墙钟超时。`rebase-ace-results.json` SHA256 7D3400911131A311F3D1642AF253AA154E198E6430B7E9BE599F2AB127A54956。

相同归档具名复跑，显式继承仓库的 RUST_MIN_STACK=16777216。AV 进程用例 1/1 通过，调用 8.371 s，exit 0，结果摘要 FFABC5C6DF3BEBF3827C21FFB633D3D0581357DA722BED709A5DC60F6816DCCB。深链栈溢出重现，测试 6.114 s、调用 9.090 s、exit 1，结果摘要 3D0F78AF4CA645437B19781EB4819AF8F93E2C7BDD23CBBD53F86E498A1EE8D5。正在用相同 debug/test 准备入口、栈和单线程执行对照当前父基线与候选。

2026-11-20 stage: merge | checkpoint: post1083-stack-baseline | plan_id: PLAN-124 | plan_revision: r1 | outcome: inherited failure confirmed

用同一 worktree/target、Cargo test profile、RUST_MIN_STACK=16777216、相同具名用例及单测试线程，候选 dcd4fdafd 与准确父基线 92b3f1842 都以 Windows 0xc00000fd 线程栈溢出失败。分别准备 4.395/5.125 s、执行 6.107/6.607 s，机器等待 113.78 s。候选、基线日志 SHA256 分别为 9E888E67121188DC7984AE053789570CC0B2F3C5588C8B012CB3DA7A9F981F07、6C59EDAE834AD261EF071A04BD30934042316C87BAD019A2F0D2D07E89E38610。已恢复原分支及干净候选。该用例保留原 20,000 深度、栈设置及失败记录。

D-1098-01 的两个 host_console 名称与断言与 1083 已记录的父基线签名匹配。子进程用例单独复跑通过，按原固定组归档并仅调整 nextest 调度至 4 线程再全组确认一次。

2026-11-20 stage: merge | checkpoint: gate-triage-reviewed | plan_id: PLAN-124 | plan_revision: r1 | outcome: no new regression | frozen_lang: dcd4fdafd412be911dc44ba1fee911a916d9c198

同归档执行 `cargo t ace-av`，仅调 nextest 调度线程为 4。等待 10.47 s、调用 98.434 s、nextest 93.101 s，1482/1484、23 跳过，exit 1。子进程用例通过，全组只剩父基线 D-1098-01 两个具名 host_console 断言失败。结果摘要 A7DBD1D8DD30B2E359BF23C484EE1E51597CB97BBE0AE9CEC30171CA38488FE7。固定组结果保留原始失败，确认三项存量失败、无新增回归。未新增、忽略或修改任何测试。

2026-11-20 stage: review | checkpoint: final-ui-lock-receipt | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass | reviewed_ui: a69eb8b2cc64bda380248f1507db8e3c83e053fd | reviewed_lang: dcd4fdafd412be911dc44ba1fee911a916d9c198 | limitations: 同会话复审

双端重建后的唯一未提交项是 AutoUI Cargo.lock 的 ace-a2x-store 条目新增一条既有 ace-foreign-contract 路径依赖。依赖自 AutoLang 29726d171 已存在，提交 a69eb8b2 固定实际重放所用的锁文件，无包版本、源文件或规范变化。锁文件 SHA256 2411B8A65FB03CDD33898B8D97A05E4A6E0B7D27CA941548FDAAC781137341BE。两个入口的 `cargo tree --locked --offline` 分别以 ace-bridge、ace-vue 特性通过。真实双端收据与 AutoLang 固定组证据继续适用。前轮 AC-01/02 复验通过，AC-03 进入落地。

2026-11-20 stage: merge | checkpoint: landed, canonical_verified | plan_id: PLAN-124 | plan_revision: r1 | outcome: delivered, cleanup pending | AutoUI: a69eb8b2cc64bda380248f1507db8e3c83e053fd | AutoLang: dcd4fdafd412be911dc44ba1fee911a916d9c198

两仓 master 依次 git merge --ff-only plan-124-dev，均为单父线性快进，AutoLang 基线 92b3f1842、AutoUI 基线 d4d9ee4e。AutoDown、AutoWeb 主检出提交与实证钉住提交一致。主检出的 AV 和 A2Vue 依赖解析均以 cargo tree --locked --offline 通过，日志 SHA256 分别为 5030FDD9114C3A5BCB1860255D80F1D0FCE309D9FD184826212701D66E404CCE / 5030FDD9114C3A5BCB1860255D80F1D0FCE309D9FD184826212701D66E404CCE，Cargo.lock 与实证相同。主线 [ACE 路由页面规范](../../specs/runtime/ace-route-page-loading.md) SHA256 54C98E0C78E176940DDFB42578C352208CEB7E1807491ADF7A94AF7D12E9A6CD，SD-01 与交付提交一致。AC-03 已落地，§5 保留 105 对 018/022 的回接范围和入口。

本档案中的计划材料路径和摘要记录的是执行时的临时证据。完成 §8 T-06 后按保留策略删除临时目录。

2026-11-20 stage: merge | checkpoint: archived | plan_id: PLAN-124 | plan_revision: r1 | outcome: checked on disk, cleanup pending

实际归档位置、状态、任务计数 5/6、归档与规范的所有计划本地链接、计划导航与 105 回接引用已核对。T-06 待清理后勾选。

2026-11-20 stage: merge | checkpoint: cleaned | plan_id: PLAN-124 | plan_revision: r1 | outcome: pass | archive_commit: 46f3bd686423f42b381e0385d8d8992b3b0a0723

依路径、Git 注册及不可跟随链接的 wt-guard 分别复核后，已删除 ui-124 组的四个工作树、两条已合入的 plan-124-dev 和空组目录。临时材料摘要已归入本档案。登记的 plan-artifacts/plan-124 已删除，两处保留登记表均更新为 deleted-at-merge，无持有项。归档文件、链接、代码祖先关系、规范摘要和 T-01 至 T-06 六项均核对完成。

产物核对：主检出标准 release 位置未发现 auto-ui-tool、桌面示例和 av 可执行文件。018/022 的现有 gen/front/vue 目录时间分别为 2026-11-10 14:44/14:46，早于本次交付。双端实证使用本计划的独立调试产物。批量回归收据 2026-11-20T00:00:00Z，last_covered_plan_id=1098，本次合入计划号不触发批门且距收据未满 48 小时。

stage: goal | plan_id: PLAN-124 | plan_revision: r1 | round: 8 | from: merge | to: done | status: archived | outcome: pass | commit: AutoUI a69eb8b2cc64bda380248f1507db8e3c83e053fd / AutoLang dcd4fdafd412be911dc44ba1fee911a916d9c198 | next: done

## 10. 待澄清事项

无。固定回归的三项父基线失败已保留在上述收据中，PLAN-105 按 §5 使用本计划覆盖的原路由和双端入口。
