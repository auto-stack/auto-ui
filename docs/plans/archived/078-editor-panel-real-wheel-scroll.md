---
plan_id: PLAN-078
status: archived
feature_name: AGE M1编辑器面板真实滚轮滚动修复（GAP-AG027-01）
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 6
total_steps: 6
depends_on: [auto-ui/PLAN-076, ag-examples/PLAN-027]
supersedes_spec_components: [docs/specs/auto-graphics/age-m1-editor-native-evidence.md]
new_spec_components: []
touched_goals: []
---

# PLAN-078：AGE M1 编辑器面板真实滚轮滚动修复（GAP-AG027-01）

## 0. 变更摘要

修复 G04 编辑器窗口 iced 面板 scrollable 不响应**真实 OS 滚轮**的缺陷（AG027
会话 finding F-02 / GAP-AG027-01）：grid/snap/focus/reset 按钮位于面板 fold
之下，真实输入不可达，直接阻塞 ag-examples PLAN-026 DEBT-002 的 G04 原生
20/34+e 族会话（posted 探针能滚到这些按钮，但 posted 通道按规范永不替代
native）。本计划修复真实滚轮→面板滚动的通路（或交付等价真实输入通路，如
键盘滚动，以 T-01 有界调查裁决为准），解除该阻塞；**不执行** G04 20/34
会话本身（属 AG026 消费侧，合并后由其按冻结 runbook 执行）。

## 1. 目标

- 编辑器窗口（`autographics-authoring --open <g04 产品>`）中，真实 OS 滚轮
  在面板区（x≥810）滚动 scrollable，fold 下按钮变为真实输入可达（点击后
  状态翻转可见）。
- 视口滚轮语义零回退：2D wheel 缩放、3D dolly、轨道/平移捕获行为不变。
- posted 路径不回退：posted 滚轮仍能滚动面板（现网对照锚保持绿）。
- AG027 冻结 runbook/checker 接口不变（阶段分母、字段、判定逻辑不动），
  仅其 F-02/GAP-AG027-01 具名阻塞解除。

非目标：不改区域派发时机、不改 N4 语义（PLAN-076 已闭）、不重开已归档
UI071/066/065、不执行 G04 原生 20/34 会话与签章（AG026 T-08 消费侧）、
不改 20/34 阶段分母或 checker 字段合同、不动 ag-examples 仓。

## 2. 架构方案

单仓 auto-ui；修复面 = `autographics-authoring` 编辑器窗口的滚轮路由链：

- **已观测事实**（候选 C 树 `4ebc51ce`）：bin `autographics-authoring.rs`
  已有 `WindowEvent::MouseWheel` 臂（行 244-249，`h.scroll(cursor, delta)`，
  PLAN-071 提交 `88771287` 引入）；`authoring/host/viewport.rs::scroll`
  （行 961-1067）在光标 x≥810 时向面板 iced 事件队列投递
  `Event::Mouse(WheelScrolled{delta})` 并记录面板本地光标 `[x-810, y]`；
  `authoring/host/panel.rs` 行 490-492 面板为 `scrollable(...)` +
  `observe_scroll(..., self.scroll_offset)`。
- **仍为假设**（T-01 裁决）：F-02 会话（2026-11-08）未滚动的原因——会话时
  该通路尚未合入（PLAN-071 后于会话）？事件未达 scrollable（drain/边界）？
  面板本地光标映射与 scrollable bounds 不符？posted 通路走的另一注入点？
  由 T-01 用交付的 `input_driver` 真实滚轮 + 6 点像素差探针在当前候选上
  复现/反证，产出根因决策工件后再实现（T-02）。
- 拟唯一树 `D:/autostack/.wt/ui-078/auto-ui`，分支 `codex/plan-078-dev`；
  材料唯一 scope `D:/autostack/plan-artifacts/plan-078/`。跨仓消费方
  ag-examples/PLAN-026 只在合并后消费正式交付。

## 3. 技术栈

已核：`experimental/autographics-auto-driver/src/bin/autographics-authoring.rs`
（MouseWheel 臂，候选 C `4ebc51ce` 行 244）、
`experimental/autographics-auto-driver/src/authoring/host/viewport.rs`
（`scroll`，行 961-1067；`events.clear()` drain 行 786）、
`experimental/autographics-auto-driver/src/authoring/host/panel.rs`
（iced scrollable + observe_scroll，行 52/490-492）、
ag-examples `checks/node_editor_tutorials/native-stages.md` §5 F-02 表行、
`checks/node_editor_tutorials/native-input/{input_driver,run_matrix_2d}.py`
（真实输入基建，G04 会话侧已交付）、AG027 `posted-probe-control` 对照锚。
Spec 现状：`docs/specs/auto-graphics/age-m1-editor-native-evidence.md`
（AG027 SD-02 交付）为编辑器原生证据合同。

## 4. 需求分析与背景调查

用户 2026-11-10 在 PLAN-026 收口目标续期指令中指明解锁路径："请用户立项
auto-ui 修复计划（同 UI-076 模式）修 GAP-AG027-01（iced 面板真实滚轮滚动/
等价真实输入通路）；修复交付后在本机跑 G04 原生 20/34+e 族会话使
verify_age_m1_editor_native PASS，再续 PLAN-026 T-08 → T-09 → T-10/T-11"。
**本合同=立项（drafting，仅文档）**：不建树、不改实现、不跑 GPU/窗口测试、
不写 specs.json/派生索引。执行须待用户对交付的明确启动（先例：UI-076 由
`/goal 完成计划076` 授予全链）。编号分配：主检出 active+archived 扫描，
max=077（`archived/077-windows-rq-product-coverage-closure.md`）→ 本计划
078，创建前复查唯一性。用户建议号"077"已被占用，以分配规则为准取 078。

## 5. 详细设计

T-01 有界调查（决策工件：`plan-artifacts/plan-078/investigation.md`）：
1) 用 `native-input/input_driver` 真实滚轮在面板区注入（当前候选构建），
   6 点像素差 + `scroll_offset` 观测；2) 对照 posted 注入同点位；3) 按
   结果分支——已修（候选 C 可滚）：记证据、跳过 T-02、跑 T-03 回归面后
   直接 SD/交接；仍坏：按二分定位根因（事件到达 scrollable？bounds 内？
   delta 形状？drain 时机？），根因记录后 T-02 实现修复。修复设计约束：
   不改 20/34 阶段分母与 checker 字段；不改变视口 wheel 分支优先级语义；
   若以键盘滚动为等价通路，必须同属真实 OS 输入栈（winit 事件→iced），并
   在 runbook 层注明可达操作。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| — | 无 | — | 引擎行为零变化（T-01 裁定：面板真实滚轮通路自 `4adf2d03`/Plan 879 起存在，实测可用，无缺陷可修）；GAP 记录所在仓为 ag-examples（`age-m1-editor-native-evidence.md` 与 runbook §5 F-02 表行在 AG027 落主侧），其更新由 AG026 会话准备时执行（AG026 仓 scope，非本计划） | 零代码交付无 spec 影响；跨仓记录归消费侧 | AC-04（转移 AG026） |

## 6. 测试设计

均未执行（本合同仅规划）。T-01 探针命令沿交付基建
`python checks/node_editor_tutorials/native-input/input_driver.py ...`（真实
SendInput 滚轮，明细以该文件 CLI 为准，T-01 物化准确命令）；T-03 引擎面：
`cargo test --locked --offline --test iced_seam_contract`（面板/iced 接缝族）
+ 触面相关既有族不回归；G04 包门 `verify_g04_packages` 保持绿（本修复不改
g04 产品/锁身份，预期不影响；如引擎 commit 变化则按 acceptance §3 由
AG026 消费侧统一安排并集——本计划交付物为引擎 master 上的新候选，AG026
T-06 已闭并集是否需重测按其消费决策记录，见 §10）。

| V | work任务 | 输入与区分性oracle | AC/边界 |
|---|---|---|---|
| V-01 | T-01/T-02 | 真实滚轮 vs posted 同点位对照；6 点像素差+scroll_offset；fold 下按钮真实点击状态翻转 | AC-01/02 |
| V-02 | T-03 | 视口 wheel 缩放/3D dolly 既有探针族零回退；iced_seam 族绿 | AC-02 |
| V-03 | T-03 | posted 滚轮滚动面板对照锚保持绿（不回退 posted 路径） | AC-03 |
| V-04 | T-04 | SD-01 spec 修订+runbook F-02 表行状态更新；AG026 消费提示 | AC-04 |

### 执行与证据规则

实现/构建/测试仅在唯一树 `D:/autostack/.wt/ui-078/auto-ui`（分支
`codex/plan-078-dev`），主检出只作 Plan 簿记；独立 review 同树独立上下文。
真实输入探针需本机非远程桌面会话（F-04 已解，AG026 T-05 同机先例）；posted
对照不需授权。GPU/窗口测试单实例错峰。零选择/子进程失败/timeout 不算 PASS；
首红、环境重放绿分别记录。merge 逐命令对账输出根并删除本计划 plan-artifacts
scope；未知归属 fail-closed；禁止 junction/symlink。

## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 真实 OS 滚轮使面板 scrollable 滚动，fold 下 grid/snap/focus/reset 按钮真实点击可达且状态翻转可见（或 T-01 裁定的等价真实通路同验证） | V-01 |
| AC-02 | work | 视口滚轮语义零回退（2D 缩放/3D dolly/捕获）既有探针族绿 | V-02 |
| AC-03 | work | posted 滚轮滚动面板对照锚绿 | V-03 |
| AC-04 | work | SD-01 spec 与 AG027 runbook F-02 表行状态更新，AG026 消费提示（DEBT-002 可执行）落地 | V-04 |
| AC-05 | review | 同树独立 final 逐 AC 复核 pass，绑定 revision 与 SHA | T-05 |
| AC-06 | merge | 冻结门后落主，canonical/导航/归档准确 | T-06 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 结果/写面 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 启动指令（用户目标续期 2026-11-10，§10 D01） | 有界调查+真实滚轮探针；决策工件 investigation.md（根因或"已修"裁定） | AC-01 | [x] 2026-11-10 裁定（经 T-05 复核锐化）=**GAP 解除不依赖历史解释**：面板真实滚轮通路自 `4adf2d03`（Plan 879）存在（候选 C 祖先已验），但该提交早于 F-02 会话时点，故"时序误差"仅是未决历史假说之一（复核意见：历史根因不完整，含未分解环境成分可能，如 F-04 ToDesk 输入钩子对滚轮注入的差异化吞没）；**解锁决定仅凭实测**——现候选系 fresh 构建上真实 SendInput 滚轮滚动面板直接测得（探针1 6点差622×6/band 132k panel_scrolled=true；探针2 双向下滚236k/上滚24k响应，复位不对称=保真度细节非阻塞）；工件=plan-artifacts/plan-078/investigation.md+probe/ |
| T-02 | work | T-01 | 按根因实现修复（engine 面板滚轮通路）；无缺陷则显式跳过并记录 | AC-01 | [x] 2026-11-10 按 §10 D02 **显式跳过**（无缺陷可修，零代码变化交付；无实现提交） |
| T-03 | work | T-02（或 T-01 已修裁定） | iced_seam 族+触面回归+posted 对照绿；视口探针零回退 | AC-02/03 | [x] 2026-11-10 零代码变化→无回退面：iced_seam_contract（prototype）**5/5 绿**（worktree 实测）；视口/posted 通路未触碰（源码行级观测+探针无损），更广族由 UI076 主检 38/38 smoke 与 AG026 T-06 候选 C 并集覆盖 |
| T-04 | work | T-03 | SD-01 spec 修订+runbook F-02 表行+AG026 消费提示 | AC-04 | [x→转移 AG026] 2026-11-10 裁定：spec/表行所在仓为 ag-examples（AG027 落主侧），更新归 AG026 会话准备（其目标已含"G04 会话使 checker PASS"，为会话前置必要项）；本计划无 spec 影响（见 §5 表） |
| T-05 | review | T-01..T-04 实质完成 | 同树独立 final，逐 AC 复核 | AC-05 | [x] 2026-11-10 独立子代理 fresh context PASS（5 项核证；意见已锐化 T-01 行——GAP 解除不依赖历史解释，仅凭候选系实测；详见 §9 复审行） | AC-05 |
| T-06 | merge | T-05 pass | 落主（零代码，归档+导航）/清理 | AC-06 | [x] 2026-11-10 归档 r1（零代码：无落主 diff；worktree/分支清理见 merge 记录；plan-artifacts scope 折叠删除，工件要点已折入本文件 T-01/§9 行） | AC-06 |

## 9. 复审记录

2026-11-10 / stage:review | review_scope: final | plan_id: PLAN-078 | plan_revision: r1 | outcome: pass（独立子代理 fresh context，只读；5 项逐条核证：源码行级事实 1a-1c、4adf2d03↔4ebc51ce 祖先、零提交零脏树、探针证据文件实存且 PNG 尺寸/md5 相异、计划行/授权记录齐） | reviewed_commit: b484fd57（计划文本）+ worktree 6b012f99（零 diff） | findings: 复核意见=历史"时序误差"解释不完整（4adf2d03 早于 F-02 会话时点，路径当时已在；历史根因未决，不排除 F-04 环境成分）；但解锁决定仅凭实测成立（消费树直接测量）。已按此锐化 T-01 行措辞：GAP 解除不依赖历史解释，仅凭候选系实测 | next: merge（T-06）

2026-11-10 / stage:work | plan_id: PLAN-078 | plan_revision: r1 | outcome: pass（execution_done：T-01..T-04 完成，零代码交付） | worktree: D:/autostack/.wt/ui-078/auto-ui（已建，无实现提交） | branch: codex/plan-078-dev | base_commit: 6b012f99 | task_ids: T-01,T-02(跳过),T-03,T-04(转移AG026) | evidence: 调查工件 plan-artifacts/plan-078/investigation.md + probe/（两探针脚本与 bundle 截图/trace）；裁定=GAP-AG027-01 时序性记录误差（通路自 4adf2d03/Plan 879 存在且为候选 C 祖先）；实测 panel_scrolled=true（6点差622×6/band132k；双向下滚236k上滚24k）；iced_seam_contract 5/5 | blockers: 无（work-owned 全闭；AC-04 转移 AG026 会话准备，属消费侧） | next_responsible_role: review（T-05 独立 final，零代码证据链复核）

2026-11-10 / stage:work | plan_id: PLAN-078 | plan_revision: r1 | outcome: pass(T-01 授权记录+开工) | worktree: D:/autostack/.wt/ui-078/auto-ui（新建，base=master 6b012f99） | branch: codex/plan-078-dev | task_ids: T-01 (in progress) | evidence: §10 D01 授权解释更新（用户目标续期指令=全链授权，UI-076 先例） | next: T-01 有界调查+真实滚轮探针

2026-11-10 / stage:new / auto-ui/PLAN-078:r1 / outcome:pass（规划合同可审阅；
纯文档，无实现/树/测试）/ next: 用户启动交付（先例口径：`/goal 完成计划078`）
→ T-01 起 0/6。依据=AG027 F-02/GAP-AG027-01 具名阻塞 + 候选 C 树观测事实
（bin MouseWheel 臂/viewport 面板分支/panel scrollable 已在，机制根因未裁）
+ AG026 DEBT-002 解锁路径（用户 2026-11-10 指令）。号源：active+archived
max=077 → 078，创建前唯一性复查通过。

## 10. 待澄清事项

D01：**执行授权（2026-11-10 用户目标续期已给）**：用户在 PLAN-026 收口目标的
连续两次续期指令中明确指名全链"启动 auto-ui/PLAN-078 端到端交付：修复
GAP-AG027-01 → 本机 G04 原生 20/34+e 族会话使 verify_age_m1_editor_native
PASS → 续 PLAN-026 T-08 签章摘要 → T-09 复审 → T-10/T-11 落主归档"；按
UI-076 先例（/goal 机制=全链授权：work/review/有界修复/merge）记为交付授权，
本条 2026-11-10 早前"执行授权未给"快照保留为历史。目标续期当时未获
AskUserQuestion 交互应答，授权解释基于指令文本本身，记录于此供 review
复核。D02：若 T-01 裁定
"候选 C 已可滚"（F-02 会话时点晚于修复合入的时序误差），则 T-02 显式跳过、
本计划以回归面+证据交付，AG026 直接解锁会话。D03：引擎 master 前进后
AG026 已闭的同版并集是否需按 acceptance §3 重测——属 AG026 消费决策：
修复若仅触及编辑器面板通路（不改 gallery/游戏运行时语义），由 AG026 记录
适用性论证；若身份失效规则触发，按其 §3 一次同版窗口重测。D04：等价真实
输入通路若选键盘滚动，不得引入全局快捷键副作用（text-focus 隔离原义务），
T-02 设计需复核 p16/p17 text-isolation 阶段兼容性。
