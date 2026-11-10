---
plan_id: PLAN-078
status: drafting
feature_name: AGE M1编辑器面板真实滚轮滚动修复（GAP-AG027-01）
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 0
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
| SD-01 | modify | docs/specs/auto-graphics/age-m1-editor-native-evidence.md | "面板 fold 下按钮真实输入不可达（GAP-AG027-01，F-02）"→真实滚轮（或裁定的等价真实通路）可达且已验证；G04 会话 grid/snap/focus/reset 阶段可执行 | 解除 DEBT-002 执行阻塞 | AC-01..04 |

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
| T-01 | work | 启动指令（/goal 完成计划078 或同义） | 有界调查+真实滚轮探针；决策工件 investigation.md（根因或"已修"裁定） | AC-01 | [ ] |
| T-02 | work | T-01 | 按根因实现修复（engine 面板滚轮通路）；无缺陷则显式跳过并记录 | AC-01 | [ ] |
| T-03 | work | T-02（或 T-01 已修裁定） | iced_seam 族+触面回归+posted 对照绿；视口探针零回退 | AC-02/03 | [ ] |
| T-04 | work | T-03 | SD-01 spec 修订+runbook F-02 表行+AG026 消费提示 | AC-04 | [ ] |
| T-05 | review | T-01..T-04 实质完成 | 同树独立 final，逐 AC 复核 | AC-05 | [ ] |
| T-06 | merge | T-05 pass | 落主/canonical/导航/归档/清理 | AC-06 | [ ] |

## 9. 复审记录

2026-11-10 / stage:new / auto-ui/PLAN-078:r1 / outcome:pass（规划合同可审阅；
纯文档，无实现/树/测试）/ next: 用户启动交付（先例口径：`/goal 完成计划078`）
→ T-01 起 0/6。依据=AG027 F-02/GAP-AG027-01 具名阻塞 + 候选 C 树观测事实
（bin MouseWheel 臂/viewport 面板分支/panel scrollable 已在，机制根因未裁）
+ AG026 DEBT-002 解锁路径（用户 2026-11-10 指令）。号源：active+archived
max=077 → 078，创建前唯一性复查通过。

## 10. 待澄清事项

D01：**执行授权未给**——本计划 drafting 0/6，未建树/未改实现/未跑测试；
交付启动沿用户明确指令（先例：`/goal 完成计划078`）。D02：若 T-01 裁定
"候选 C 已可滚"（F-02 会话时点晚于修复合入的时序误差），则 T-02 显式跳过、
本计划以回归面+证据交付，AG026 直接解锁会话。D03：引擎 master 前进后
AG026 已闭的同版并集是否需按 acceptance §3 重测——属 AG026 消费决策：
修复若仅触及编辑器面板通路（不改 gallery/游戏运行时语义），由 AG026 记录
适用性论证；若身份失效规则触发，按其 §3 一次同版窗口重测。D04：等价真实
输入通路若选键盘滚动，不得引入全局快捷键副作用（text-focus 隔离原义务），
T-02 设计需复核 p16/p17 text-isolation 阶段兼容性。
