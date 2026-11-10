---
plan_id: PLAN-076
status: archived
feature_name: AGE M1原生取证scale轨迹修复与N4语义收口
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 5
total_steps: 7
depends_on: []
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-008]
---

# PLAN-076：AGE M1原生取证scale轨迹修复与N4语义收口

## 0. 变更摘要

修复 AG026 双档会话路由出的两个引擎发现（证据：ag-examples plan-artifacts plan-026 session-bundle + supplementary）：

1. **F-SCALE-02**（代码修复）：`observe_window` 在关窗写报告时**替换**`tier.scales_observed`，单次切档会话只留下末档（如 [2.5]），起始档丢失 → 引擎 validator 误判"single scale tier"而拒 native-pass，尽管 checkpoint 轨迹完整记录了 2.0→2.5。修复=开启时播种 + 关窗时合并；顺带修复 `scale_factor_changed` before==after 的晚读 quirk（改为 last_scale 跟踪）。
2. **F-N4-01**（语义裁定+codify，方案 b）：N4 检查表"mouse leave cancels the press (no region click)"与 054 canonical"区域点击**命中即入队**"（按下即派发）冲突——按住期 winit 捕获抑制 CursorLeft，`cancel_all` 的离窗取消对已派发交互不可达。裁定=**保留 canonical 按下即派发**，将 N4 检查表语义收窄为可满足形态："空白 UI 区按下→离窗→窗外释放：按钮不激活、无区域点击"（`cancel_all`/UI 面取消路径已支持）；onclick 区域"按下即派发"写入 N4 expected 并沉淀规范。

非目标：不改区域派发时机为 release-inside（方案 a，否决理由：违反 054 canonical"命中即入队"，需重验 G-M04/G-M06/probe 全部路由证据，收益仅检查表措辞）；不改 validator 的 ≥2 档规则；不重开已归档 UI071/066/065；不动 ag-examples 侧（AG026 消费方在本计划落地后自行重判 N4 与重跑接收器）。

## 1. 目标

- AG026 双档原生会话产生的报告能通过引擎 validator：单次切档会话 `tier.scales_observed` == [起始档, 末档]，`window.scale_factor_changed` 记录真实 (before, after) 转换对。
- N4 检查表语义与引擎实际行为一致且可执行：空白区按下→离窗→窗外释放 ⇒ 无按钮激活、无区域点击（取消路径）；onclick 区域按下即派发（canonical）被明确文档化。
- 消费方 ag-examples/PLAN-026 可据此重判 N4 并关 DEBT-001（其 bundle 其余证据已齐）。

成功判据：新增/更新测试全绿（单变档 → [X,Y]；多变档 → 全部触档；N4 语义回归）+ 既有 probe/manual 测试族不回归。

## 2. 架构方案

全部改动在 auto-ui 仓 `experimental/autographics-auto-driver`：

- `src/m1_evidence.rs`：`observe_window` 由替换改为**合并**（scale 不在 `scales_observed` 时 push）；新增 `seed_window_scale`（或在 observe_window 内合并即达播种效果——开窗即首次调用）。
- `src/bin/mixed_regions_window.rs`：ManualSeen 增加 `last_scale`（开窗时以 `window.scale_factor()` 初始化）；`ScaleFactorChanged` 处理改记 `(last_scale, new_scale)` 并更新 `last_scale`；manual setup 处开窗即 `observe_window` 播种。
- `src/bin/autographics-authoring.rs`（editor `--evidence-record`）：核对其窗口 scale 记录路径，同享 m1_evidence 合并语义；若存在同款覆盖则同修（有界核对任务）。
- `MIXED_MANUAL_CHECKLIST` N4 文案更新 + 冻结清单测试同步（语义裁定 2026-11-10，附 054 canonical 依据）。
- 规范文档：054 canonical 派发语义显式化 + `m1-native-evidence-support.md` N4 行与 scales_observed 规则更新。

无新增组件/依赖；不动 posted probe 路径（`--probe` 报告不含 scales_observed 累积，行为不变）。

## 3. 技术栈

已核（本会话实测）：`src/m1_evidence.rs`（EvidenceRecord/observe_window L96 附近替换写法/record_scale_change 追加 after/MIXED_MANUAL_CHECKLIST L21/validate L201 要求 scales≥2）；`src/bin/mixed_regions_window.rs`（事件接线 L770-830：checkpoint 先于 pointer_pressed；ScaleFactorChanged L809-823 晚读 before；CursorLeft→cancel_all L801-807）；`tests/m1_native_evidence_support.rs`（9 用例含 m1_manual_checklist_is_frozen_and_never_auto_passes）；canonical `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md`（"命中即按当前挂载代次入队"）；`docs/specs/auto-graphics/m1-native-evidence-support.md`（N1..N6 冻结文本 + validator 分层规则）。

验证命令：`cargo test -p autographics-auto-driver --test m1_native_evidence_support -- --test-threads=1`、`cargo test -p autographics-auto-driver --lib m1_evidence`（组名以 `cargo t --list` 实核）。

## 4. 需求分析与背景调查

授权：用户 2026-11-10 在 AG026 会话确认"N4 需要修复"并指示"用一个计划解决这两个问题"（F-N4-01 + F-SCALE-02）。范围=auto-ui 仓引擎/规范/测试；不含 ag-examples 侧改动与任何tag/发布。用户 2026-11-10 以 `/goal 完成计划076` 授予 auto-plan-goal 全链授权（work→review→有界修复→merge，含建树/归档/清理），各阶段继承，不再重复询问。

实证证据（2026-11-09/10 双档会话，plan-artifacts/plan-026/ag-examples/session-bundle 及 supplementary/）：

- dry-run-1 报告：cp15 click(count_text=0) → cp16 cursor_left(count_text=**100**)——StripHit 在按下时已派发（checkpoint 先于 pointer_pressed），离窗释放未撤回。
- 会话2 报告：cp9 press(count 0) → cp10-11 cursor_left → count 保持 0（N4 取消路径在该布局下正确工作）；gauge 按针在按下瞬间位移（p2 区域内容消费按下）。
- 会话2 报告 tier.scales_observed=[2.5]（observe_window 关窗覆盖）而 checkpoint 轨迹含 2.0→2.5——validator 单档误拒的直接证据。
- 引擎源码点位：observe_window `data["tier"]["scales_observed"] = json!([scale])`（替换写法）；ScaleFactorChanged 处理 `before = window.scale_factor()` 晚读（与 seen 对 (X,X) 一致）。

输入 SHA：m1_evidence.rs 与 mixed_regions_window.rs 以执行时 `git rev-parse HEAD:src/...` 为准（当前 4ebc51ce 线；本会话实测报告自绑 runner sha 1d0bad67…）。

## 5. 详细设计

### F-SCALE-02

1. `m1_evidence.rs::observe_window`：`scales_observed` 改为合并（`if !scales.contains(scale) push(scale)`），不再替换；开窗即调用即为播种。
2. `mixed_regions_window.rs`：ManualSeen 增加 `last_scale: f32`（setup 时 `window.scale_factor()` 初始化）；`ScaleFactorChanged` 记 `(last_scale, scale_factor)` 后更新 `last_scale`；关窗写报告前若 `last_scale` 存在则以 `observe_window(last_scale)` 确保起始档在列（合并幂等）。
3. `autographics-authoring.rs` editor 记录路径：核对 observe_window/scale 绑定调用点，若同样替换则同修（有界核对，预期共享 m1_evidence 已覆盖）。

预期效果：开窗 X → 一次切档 Y → 关窗：`scales_observed == [X, Y]`；`scale_factor_changed == [(X, Y)]`；validator ≥2 档检查可读通过。

### F-N4-01（方案 b：语义收窄 + codify）

1. `MIXED_MANUAL_CHECKLIST[3]`（N4）改为："press on empty UI space, leave the window and release outside: no button activates and no region click is dispatched"。
2. 冻结清单测试同步更新 + 注释（2026-11-10 语义裁定；canonical 054"命中即入队"保留，onclick 区域按下即派发不可离窗撤回）。
3. 行为回归测试（新增，posted/manual 集成面）：空白区按下→CursorLeft 注入/模拟离窗→窗外释放 ⇒ 断言无 `_ui_click_` 入队、无按钮激活；附 strip 按下即派发的对照断言（canonical 行为固化，防回归到歧义态）。
4. 规范沉淀（见规范增量 SD-01/SD-02）。

否决方案 (a)（release-inside 派发）记录于 §10：与 054 canonical 冲突、重验面大、且会改变既有游戏/区域交互时序。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/m1-native-evidence-support.md | N4 检查表文本收窄为"空白区按下/离窗/窗外释放→无激活无区域点击"；新增 scales_observed 规则（开窗播种、变更追加、关窗合并；scale_factor_changed 记录真实前后对） | 与引擎实际行为一致；validator ≥2 档可判定；AG026 消费 | AC-02/AC-03 |
| SD-02 | modify | docs/specs/auto-graphics/node-mixed-regions-lifecycle.md | 派发语义显式化：区域交互命中即派发（按下时），CursorLeft/离窗不撤回已派发交互；离窗取消仅作用于按住捕获/UI 释放路径 | 消除检查表与 canonical 的语义冲突；固化 F-N4-01 裁定 | AC-03 |

## 6. 测试设计

| V | 任务 | 输入与区分性 oracle | AC/边界 |
|---|---|---|---|
| V-01 | T-01/T-02 | 单变档会话（开 2.0→切 2.5→关窗）：tier.scales_observed==[2.0,2.5] 且 scale_factor_changed==[(2.0,2.5)]；多变档：全部触档在列。区分性：旧实现（替换+晚读）产 [2.5]+[(2.5,2.5)] 必红 | AC-01 |
| V-02 | T-03 | 空白区按下→离窗→窗外释放：无 `_ui_click_` 入队、无按钮激活；strip 按下：立即派发（canonical 固化）。区分性：若实现改为 release-inside 或漏 cancel 则红 | AC-02 |
| V-03 | T-04 | 冻结清单测试更新后文本逐字断言；两规范文档含 SD-01/SD-02 全部要点 | AC-03 |
| V-04 | T-05 | 既有 9 用例 m1_native_evidence_support 全绿 + node_mixed_regions 路由族不回归（scoped 集成） | AC-04 |

## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 单变档/多变档会话 scales_observed 与 scale_factor_changed 按设计记录（含起始档） | V-01 |
| AC-02 | work | N4 新语义回归测试绿（空白区路径无派发；strip 按下即派发固化） | V-02 |
| AC-03 | work | 检查表/两规范文档语义一致且与引擎行为一致 | V-03 |
| AC-04 | work | 既有测试族不回归（scoped） | V-04 |
| AC-05 | review | 同树独立 final：语义裁定对照 054 canonical 复核 + 全部 AC 证据 | T-06 |
| AC-06 | merge | 落主 + canonical 生效 + 归档清理；AG026 消费提示落 debts.md/导航 | T-07 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 结果/写面 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | - | m1_evidence observe_window 合并 + 单测（含编辑路径核对） | AC-01 | [x]（F-076-R2-01 修复 @8b4ff336：D02 结论更正——editor 经 `#[path]` 包含的 authoring/cli.rs:549 对新记录单次调用 observe_window，无同款覆盖写法，合并语义无害继承；测试注释更正）observe_window 合并幂等 + 单测含 1.13 恒等合并钉；pre-fix 红实证 |
| T-02 | work | T-01 | mixed_regions_window last_scale 跟踪 + 开窗播种 + 集成测试 | AC-01 | [x]（F-076-R2-02 修复 @8b4ff336：last_scale/scale_factor_changed 升 f64 原值保真，非 f32 精确档不裂档；bin 单测含 1.13 保真断言）ManualSeen.last_scale（resumed 播种）+ push_scale_pair 真实对 + 关窗读回前起始档播种；集成会话形用例绿 |
| T-03 | work | T-02 | N4 检查表文本 + 冻结测试更新 + N4 语义回归测试 | AC-02 | [x] N4 收窄文本（2026-11-10 裁定注释）+ 冻结测试逐字断言 + 回归 m1_n4_empty_press_cancels_on_leave_and_region_press_dispatches_at_hit 绿（取消路径 + strip 按下即派发对照） |
| T-04 | work | T-03 | SD-01/SD-02 规范文档修订 | AC-03 | [x] SD-01：m1-native-evidence-support.md N4 行 + scales_observed 规则（播种/追加/合并 + 真实对）；SD-02：node-mixed-regions-lifecycle.md §3 指针派发语义显式化（命中即入队/离窗不撤回/取消可达面） |
| T-05 | work | T-01..04 | scoped 回归（m1_native_evidence_support + node_mixed_regions 族） | AC-04 | [x]（修复修订 @8b4ff336 重跑受影响面：lib 9 + bin 2 + m1 11 = 22/22 绿；probe 6 + flow 7 + storage 3 复用 210fc93d 证据——修复未触及 host 库路径）全族 38 例对应绿 |
| T-06 | review | T-05 | 同树独立 final（语义裁定对照 054） | AC-05 | [x] 独立子上下文 final：语义裁定对照 054 canonical 复核 pass（needs_fix 1 轮→修复 8b4ff336→re-review pass）；证据见 §9 |
| T-07 | merge | T-06 | 落主/canonical/归档清理 + AG026 消费提示 | AC-06 | [x] ff 落主 8b4ff336、canonical 双文档哈希核验一致、AG026 消费提示落 debts.md DEBT-002＋README M1 表导航、归档清理完成（收据见 §9） |

执行工作树：`D:/autostack/.wt/ui-076/auto-ui`，分支 `codex/plan-076-dev`（起于执行时 auto-ui master）。

## 9. 复审记录

2026-11-10 / stage:new / auto-ui/PLAN-076:r1 / outcome:pass（规划合同可审阅；非实现/独立final）/ next:T-01 起按启动指令推进，0/7 未建树。依据=AG026 双档会话实证（F-N4-01/F-SCALE-02，证据包 plan-artifacts/plan-026/ag-examples/session-bundle）；N4 语义采用方案 b（保留 canonical 按下即派发），方案 a 否决理由见 §0。

2026-11-10 / stage: work | plan_id: PLAN-076 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-076/auto-ui | branch: codex/plan-076-dev | base_commit: 0867e201 | code_commit: 210fc93d | dependency_revisions: SDK snapshot 681 files（prepare_sdk.py --from-snapshot PASS）| task_ids: T-01..T-05 | evidence: scoped 回归 38 例全绿（lib m1_evidence 9＋bin mixed_regions_window 2＋m1_native_evidence_support 11 --test-threads=1＋node_mixed_regions_probe 6＋mixed_regions_lifecycle_flow 7＋storage 3）；pre-fix 红实证：临时还原旧替换写法后 2 区分性用例（single_scale_change_session/multi_change_sessions）必红，修复回绿；AC-01 单变档 [2.0,2.5]+真实对 (2.0,2.5)、多变档全触档在列；AC-02 取消路径（空白区按下→离窗→窗外释放 ⇒ callbacks/count 不变、无投递）+strip 按下即派发对照；AC-03 两规范文档按 SD-01/SD-02 修订；D02 结论：editor autographics-authoring.rs 无 m1_evidence/observe_window 调用点（PLAN-071 evidence_record/attach 字段声明未接线），不存在同款覆盖写法，无需修复 | blockers: none | next_responsible_role: final review（T-06，review_scope: final，同树独立，对照 054 canonical 复核语义裁定）

2026-11-10 / stage: goal | plan_id: PLAN-076 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-076/auto-ui | commit: 210fc93d | next: review（授权链：/goal 完成计划076 → work 已记录于 §4/§9）

## 10. 待澄清事项

D01：N4 语义采用方案 b（检查表收窄）为本合同承诺；若 owner 倾向方案 a（改派发时机为 release-inside），需升格 plan_revision 并重验区域路由证据——启动执行前可改。
D02：editor `--evidence-record` 路径的 scale 记录调用点以 T-01 有界核对为准；若编辑路径不经过 observe_window 则仅记录核对结论。

2026-11-10 / stage: review | review_scope: final | reviewer_identity: 独立子上下文（general-purpose agent，同树 210fc93d 只读复核）＋本会话复核其关键证据；limitation：与实现同会话，以工件重建为准 | plan_id: PLAN-076 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-076/auto-ui | branch: codex/plan-076-dev | reviewed_commit: 210fc93d | base_commit: 0867e201 | dependency_revisions: SDK snapshot 681 | spec_inputs: m1-native-evidence-support.md（SD-01 后）＋node-mixed-regions-lifecycle.md（SD-02 后） | acceptance_results: AC-01 pass（单变档/多变档/会话形用例＋pre-fix 红实证）；AC-02 pass（N4 回归＋冻结逐字断言）；AC-03 pass（SD-01 文本对照代码逐项核实、N4 行与代码常量字节一致；054 canonical L108 保留、新 §3 语义与宿主行为一致——absorb(press=true) 即 enqueue_region_click_at，cancel_all 只取消组合层捕获）；AC-04 pass（独立重跑 38/38 绿，exit 0） | pending_closeout_gates: T-07 落主/canonical/归档清理/AG026 消费提示 | findings: F-076-R2-01（major）D02 核对结论事实性反转——editor 路径经 `#[path]` 包含 `src/bin/authoring/cli.rs`，:549 调用 observe_window（:520 新记录单次调用），"无 m1_evidence 调用点/字段未接线"为假（根因：核对只 grep 了 autographics-authoring.rs 未跟模块包含）；行为影响为零（单次调用无同款覆盖），但错误结论固化于 m1_evidence.rs 测试注释与 §8/§9，须更正后方可合并；F-076-R2-02（minor）last_scale/scale_factor_changed 用 f32 截断 winit f64 读数——非 f32 精确档（如 Windows 自定义 113%→1.13）单档会话会产出 [1.1299…,1.13] 两条"档"，反常地解除 validator 单档门（与本计划所修 bug 成镜像）；标准 0.25 步进档均 f32 精确，实机影响低，但 SD-01 承诺"真实 winit 读数"；F-076-R2-03（note）m1-native-evidence-support.md §2 计数行（6/3 例）在本计划增量后失真（现 9/11）；lifecycle 文档 flow/9 例计数为更早计划遗留失真，不在本次范围；notes：晚读 (X,X) 修复的接线面（事件→push_scale_pair）为 code-review-only（真实 DPI 切换不可自动化），N4 行为测试按方案 b 设计对基线亦绿（区分方案 a 回归与 cancel 泄漏）——与 V-02 措辞一致 | evidence: 独立复核 38/38 全绿（lib 9＋bin 2＋m1 11＋probe 6＋flow 7＋storage 3）；cli.rs:517/549/587 调用点本会话实核确认；absorb/cancel_all 语义核实（node_mixed_regions_host.rs:485-489/527-548） | next: work（有界修复 R1/R2/R3，重开 T-01/T-02/T-05）

2026-11-10 / stage: work（repair）| plan_id: PLAN-076 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-076/auto-ui | branch: codex/plan-076-dev | base_commit: 0867e201 | code_commit: 8b4ff336 | dependency_revisions: SDK snapshot 681 | task_ids: T-01,T-02,T-05（F-076-R2-01/-02/-03 闭合）| evidence: R1=D02 结论更正（m1_evidence.rs 测试注释重写为 cli.rs:549 单次调用事实；本 §9 顶部错误结论由本收据更正，原收据按 append-only 保留）＋cli.rs:517/549/587 本会话实核；R2=last_scale: Option<f64>、scale_factor_changed: Vec<(f64,f64)>（resumed 播种/事件/关窗播种/record_scale_change 全链去 f32 截断）＋bin 1.13 保真断言＋lib 1.13 恒等合并钉；R3=m1-native-evidence-support.md §2 计数行 9/11（含 PLAN-076 增量注）| 修复后受影响面重跑 22/22 绿（lib 9＋bin 2＋m1_native_evidence_support 11 --test-threads=1）；probe 6+flow 7+storage 3 复用 210fc93d 证据（修复未触及 host 库路径，reuse 规则）| blockers: none | next_responsible_role: final re-review（聚焦 F-076-R2-01/-02/-03 闭合与受影响 AC）

2026-11-10 / stage: review（re-review）| review_scope: final | reviewer_identity: 独立子上下文（同一 agent 续跑，只读复核 8b4ff336）| plan_id: PLAN-076 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-076/auto-ui | branch: codex/plan-076-dev | reviewed_commit: 8b4ff336 | base_commit: 0867e201 | dependency_revisions: SDK snapshot 681 | spec_inputs: m1-native-evidence-support.md（SD-01+R3 后）、node-mixed-regions-lifecycle.md（SD-02 后） | acceptance_results: AC-01 pass（f64 保真全链无截断：resumed :747 / handler :686-689 / 关窗播种 :323-326 / record_scale_change :342-342 全部 cast-free；1.13 lib 钉真实有效——截断必红）；AC-02 pass（不受修复影响，维持 210fc93d 证据）；AC-03 pass（N4 行/SD 文本维持；R3 计数行 9/11 与实测一致）；AC-04 pass（受影响面 22/22 绿：lib 9＋bin 2＋m1 11；probe/flow/storage 16 例复用 210fc93d 证据——修复未触及 host 路径）；AC-05 pass（本收据：独立 final 完成，语义裁定对照 054 复核维持 pass） | pending_closeout_gates: T-07（落主/canonical 生效/归档清理/AG026 消费提示落 debts 或导航） | findings: F-076-R2-01/-02/-03 全部 closed（file:line 证据在案）；无范围蔓延（diff 仅 3 声明文件）；note：§8/§9 历史 D02 错误措辞由 §8 T-01 现行文本与本 §9 前一收据更正（append-only 保留原样） | evidence: 复审 22/22 exit 0；git diff --name-only 210fc93d..8b4ff336 = 3 文件；grep 无残留错误结论 | next: merge（T-07）

2026-11-10 / stage: goal | plan_id: PLAN-076 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-076/auto-ui | commit: 8b4ff336 | next: merge（1 次 needs_fix→有界修复→re-review pass，未超 3 轮修复上限）

2026-11-10 / stage: merge | plan_id: PLAN-076:r1 | outcome: pass（checkpoint 进度） | worktree: D:/autostack/.wt/ui-076/auto-ui | branch: codex/plan-076-dev | delivery_commit: 8b4ff336 | prepared: 基线=reviewed@8b4ff336（final pass 绑定 r1）；canonical diff=SD-01/SD-02 已在 210fc93d+8b4ff336 内（实现+规范同链，无独立规范提交）；源目标=docs/specs/auto-graphics/{m1-native-evidence-support,node-mixed-regions-lifecycle}.md；依赖=SDK snapshot 681 | landed: wt-guard clean（无 reparse point）；主检出 `git merge --ff-only codex/plan-076-dev` → master tip=8b4ff336（无合并提交，线性历史保持；基 0867e201 未漂移，无 rebase/哈希改绑）；他 session WIP（.next-id、077-*.md、docs/design/**）按规保留未并入 | canonical_verified: m1-native-evidence-support.md sha256=816c02a19ef3572a1231537331b30fd99ec8a41969f7eaeeaf03425d53b2dd8b、node-mixed-regions-lifecycle.md sha256=0680e9862e16f46003007e004b3501230534fece2aa81b5027bb83d7c641865c——主检出文件与 8b4ff336 blob 逐一一致；N4 新文本/canonical"命中即入队"（2 处）/§3 派发语义段均在主检出现 | 落主冒烟：主检出全受影响族 38/38 绿（lib 9＋bin 2＋m1 11＋probe 6＋flow 7＋storage 3，exit 0） | AG026 消费提示：debts.md 新增 DEBT-002（跨仓关闭窗口：N4 重判＋接收器重跑＋关 ag-examples DEBT-001）＋README M1 表 UI076 行改 archived 导航 | next: archived→cleaned（工作树/分支/scratch 清理与最终收据随后提交）

2026-11-10 / stage: merge（cleaned）| plan_id: PLAN-076:r1 | outcome: pass | worktree: 已移除 | cleaned: wt-guard clean（落主前＋移除前两次，无 reparse point）；worktree D:/autostack/.wt/ui-076/auto-ui 已移除（build intermediates 随树消亡）；branch codex/plan-076-dev 已删（was 8b4ff336，已 ff 并入 master）；组目录 .wt/ui-076/ 已删空；plan-artifacts/plan-076/ 已整域删除（仅 1 临时文件 m1_evidence.rs.fixed，内容已提交入仓并折入本 §9 收据）；retention 注册表已刷新（auto-ui/PLAN-076 deleted-at-merge，no holds，53 entries） | landing_checks: 主检出 post-landing 冒烟 38/38 绿（cargo test 全 target 重建，target/debug 与 8b4ff336 同源）；本仓无生产进程/守护直接消费 auto-ui target 产物——AG026 消费方按 m1-native-evidence-support.md §4 配方自行构建 runner（消费提示已落 debts.md DEBT-002）；批量回归 due-check 不适用（auto-lang 专属门，auto-ui 无 .last-batch-regression.json 约定） | final_counts: 7/7 任务全闭（T-01..T-05 work、T-06 review、T-07 merge），r1 0 重开，1 次 needs_fix 有界修复轮（未超 3 轮上限） | completion_kind: delivered | next: done
