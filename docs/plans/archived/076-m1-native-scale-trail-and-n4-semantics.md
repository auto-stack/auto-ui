---
plan_id: PLAN-076
status: drafting
feature_name: AGE M1原生取证scale轨迹修复与N4语义收口
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 0
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

授权：用户 2026-11-10 在 AG026 会话确认"N4 需要修复"并指示"用一个计划解决这两个问题"（F-N4-01 + F-SCALE-02）。范围=auto-ui 仓引擎/规范/测试；不含 ag-examples 侧改动与任何tag/发布。

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
| T-01 | work | - | m1_evidence observe_window 合并 + 单测（含编辑路径核对） | AC-01 | [ ] |
| T-02 | work | T-01 | mixed_regions_window last_scale 跟踪 + 开窗播种 + 集成测试 | AC-01 | [ ] |
| T-03 | work | T-02 | N4 检查表文本 + 冻结测试更新 + N4 语义回归测试 | AC-02 | [ ] |
| T-04 | work | T-03 | SD-01/SD-02 规范文档修订 | AC-03 | [ ] |
| T-05 | work | T-01..04 | scoped 回归（m1_native_evidence_support + node_mixed_regions 族） | AC-04 | [ ] |
| T-06 | review | T-05 | 同树独立 final（语义裁定对照 054） | AC-05 | [ ] |
| T-07 | merge | T-06 | 落主/canonical/归档清理 + AG026 消费提示 | AC-06 | [ ] |

执行工作树：`D:/autostack/.wt/ui-076/auto-ui`，分支 `codex/plan-076-dev`（起于执行时 auto-ui master）。

## 9. 复审记录

2026-11-10 / stage:new / auto-ui/PLAN-076:r1 / outcome:pass（规划合同可审阅；非实现/独立final）/ next:T-01 起按启动指令推进，0/7 未建树。依据=AG026 双档会话实证（F-N4-01/F-SCALE-02，证据包 plan-artifacts/plan-026/ag-examples/session-bundle）；N4 语义采用方案 b（保留 canonical 按下即派发），方案 a 否决理由见 §0。

## 10. 待澄清事项

D01：N4 语义采用方案 b（检查表收窄）为本合同承诺；若 owner 倾向方案 a（改派发时机为 release-inside），需升格 plan_revision 并重验区域路由证据——启动执行前可改。
D02：editor `--evidence-record` 路径的 scale 记录调用点以 T-01 有界核对为准；若编辑路径不经过 observe_window 则仅记录核对结论。
