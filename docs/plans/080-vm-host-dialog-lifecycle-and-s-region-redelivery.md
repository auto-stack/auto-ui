---
plan_id: auto-ui/PLAN-080
status: drafting
feature_name: VM 宿主 IFileDialog 生命周期/生成可靠性与 S 区编辑链修复再交付（READER-009 消费解除）
author: [relay/009-session]
created_at: 2026-11-12T00:00:00Z
updated_at: 2026-11-12T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-080：VM 宿主 IFileDialog 生命周期/生成可靠性与 S 区编辑链修复再交付

## 0. 变更摘要

READER-009 消费窗实证的两个 auto-ui 域缺陷 + 一个再交付项（七臂二分 + run87-97b
插桩证据，全部在案）：

- **D1 宿主 IFileDialog 生命周期缺陷（mode A）**：对话框生成链完整到达 rfd
  （AUTO_DIALOG_DIAG：`host open_files enter` → `pick_files` → `exit n=0`），
  但对话框**打开后 ~1.1s 无输入自毁**（run97b 实证：t=158998ms enter →
  t=160144ms exit n=0，无用户交互）——或打开后**模态阻塞宿主**（run97b 后段
  harness 全部 MCP 调用挂起直至强杀）。候选=rfd/COM 套间（STA）与宿主线程、
  父窗 HWND 选择（多窗进程）、iced 事件循环与模态泵冲突。
- **D2 对话框生成 flake（mode B）**：同 cohort 部分运行中 park 后 **45-180s
  无 #32770**（run89/94/95 + cancel-diag.log 独立 Win32 扫描零命中）——
  bridge 请求未达宿主或生成路径静默失败。与 D1 可能同根（生命周期/线程域）。
- **D3 S 区编辑链修复再交付**：S02-S12 编辑链（"编辑"按钮派发断裂→W-07 五叶
  +S 族连锁）的修复内容只存在于 7d3a73aa 致死线捆绑的 auto-ui 中——需在
  owner-filter 修复后的基线上重做/移植（round49 bisect verdict 在案）。

消费方解除物：READER-009 picker 8 叶（D1/D2）、S 族与 W-07 编辑链（D3）、
theme 域不受本计划影响。

## 1. 目标

- D1：宿主 `pick_files` 的对话框生命周期确定性——生成后保持打开直至用户操作，
  关闭后结果正确回传（n=0 或 n=文件数）；连续 10 次生成零自毁。
- D2：生成可靠性——`pick_files` 调用后对话框在 10s 内生成的成功率 ≥ 连续
  10/10（009 消费矩阵语境）。
- D3：S 区编辑链在 owner-filter 基线上可用（"编辑"按钮派发到详情页编辑态），
  009 W-07 编辑链消费翻绿。
- 非目标：不重做 PLAN-079 的 owner-filter 修复（基线即含）；不动 949/995 性能
  语义；不处理非 Windows 平台对话框。

## 2. 架构方案

- D1 插桩调查（T-01）：AUTO_DIALOG_DIAG 扩点（对话框 HWND 创建/销毁、父窗
  解析结果、线程 COM 套间类型 CoGetApartmentType、WM_CLOSE 来源栈）+ 009
  harness 作复现台（其上下文 100% 复现 mode A）。
- D1 修复候选（按调查定谳排序）：① pick_files 调度到专用 STA 线程执行并泵
  消息（rfd 要求 STA 的标准解法）；② 父窗 HWND 选择修正（排除隐藏/错误窗）；
  ③ 模态泵与 iced 事件循环的交互隔离。
- D2：按 T-04 定谳——若与 D1 同根（线程/套间）则同修复覆盖；若为 bridge 分发
  丢失则补 bridge 请求的可靠送达。
- D3：从 7d3a73aa 线提取 S 区编辑链修复 diff → 在当前基线重放/重做（owner-
  filter 修复不受影响面），带 009 W-07 编辑链消费验证。

## 3. 技术栈

Rust（auto-ui-runtime vm_bridge / host_adapters::dialog / rfd / iced-winit）、
Win32 COM（CoGetApartmentType/SendInput/UIA 诊断）、现有 `cargo t` 组。

## 4. 需求分析与背景调查

- **授权**：READER-009 会话用户（round49"我在此起草（推荐）"先例 + round54
  "去auto-ui修复吗"方向确认）。执行由 auto-ui 会话接手。
- **证据链**（全部在案）：
  - mode A 插桩：plan-009/t03/{run97b.log,ws-ui-vm-97b-diag/app.log}
    （enter@158998ms → exit n=0 @160144ms，1.146s 自毁，无输入；后段 harness
    模态挂起直至强杀）。
  - mode B：plan-009/t03/{run89/94/95 dlg=NO DIALOG,cancel-diag.log 独立
    Win32 扫描 70s 零 #32770}。
  - 七臂二分：plan-009/t03/bisect-autolang/ + handoff-autolang/
    RESPONSE-bisect-verdict.md（D3 修复内容=7d3a73aa 线的 S 区修复，仅存在
    于致死组合捆绑的 auto-ui）。
  - PLAN-079 归档：owner-filter 修复+D2 决策工件（其 AC-02 验证窗 3/3 生成
    ——本计划的 mode A/B 为其后的残留/新形态）。
- **约束**：基线=master@8389a1e5（=9dd3fc27 内容）；五兄弟只读；不回退
  accesskit（已洗清）；949/995 性能语义不变。
- 观测 vs 假设：mode A 自毁=插桩实测（观测）；自毁机理=假设（T-01 定谳）；
  mode B 丢失点=假设（同前）。

## 5. 详细设计

### D1 生命周期确定性

- 调查：对话框线程的 COM 套间类型、父窗 HWND 的选择与状态、spawn→close 间
  的消息流（WM_CLOSE/WM_NCDESTROY 来源）。
- 修复方向（按定谳）：专用 STA 线程泵对话框（标准 rfd 解法）或修正父窗/套间；
  修复后对话框保持打开直至用户操作（无输入 10s 仍打开=确定性标准）。

### D2 生成可靠性

- 按 T-04 定谳：同根则 D1 修复覆盖并压测（10/10 生成）；异根则补 bridge 送达。

### D3 S 区编辑链再交付

- 7d3a73aa 线的 S 区修复内容在 owner-filter 基线上重放/重做；009 W-07 编辑链
  消费验证为准。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/runtime/native-vm-app-validation.md | 宿主 pick_files 对话框生命周期：生成后保持打开直至用户操作；模态期间宿主消息泵正确 | mode A 自毁+模态挂起实证（run97b） | AC-01/02 |
| SD-02 | add/modify | docs/specs/runtime/vm-route-page-loading.md（按定谳落点） | pick_files bridge 请求可靠送达宿主 | mode B 生成丢失 | AC-02 |
| SD-03 | add/modify | docs/specs/runtime/component-and-store-instances.md | S 区编辑链派发（按 7d3a73aa 修复内容重放口径） | D3 | AC-03 |

## 6. 测试设计

- D1 负例/压测：009 harness（ws-ui-vm 复现台）连续 10 次生成压测——无输入
  10s 内对话框保持打开；无自毁；宿主不挂起。
- D3 消费验证：009 picker_case cover 三腿 + W-07 编辑链。
- 组回归：plan069/plan079 相关组 + 本计划新组。

## 7. 验收标准

| AC | owner_stage | 内容 | 验证 |
|---|---|---|---|
| AC-01 | work | mode A 根因定谳+修复：对话框无输入 10s 保持打开；宿主不挂起 | 009 harness 压测 |
| AC-02 | work | mode B 根因定谳+修复：pick_files 后 10s 内生成 ≥10/10 | 同上 |
| AC-03 | work | D3 S 区编辑链在基线可用 | 009 W-07 消费 |
| AC-04 | review | 009 换 pin 消费：82 叶零死亡 + picker 8 叶 + W-07 编辑链 | READER-009 换 pin 回归 |

## 8. 执行步骤

| 任务 | owner_stage | 依赖 | 内容 | 验证 | AC |
|---|---|---|---|---|---|
| T-01 | work | — | D1 插桩调查（套间/父窗/消息流定谳） | 定谳记录 | AC-01 |
| T-02 | work | T-01 | D1 修复（按定谳：STA 线程泵/父窗修正/模态隔离） | 10 次压测 | AC-01 |
| T-03 | work | T-02 | D2 按定谳覆盖或补 bridge 送达 | 10/10 生成 | AC-02 |
| T-04 | work | — | D3 S 区修复重放/重做 | 009 W-07 消费 | AC-03 |
| T-05 | review | T-02..04 | 009 换 pin 消费回归（跨仓协作：READER-009 会话） | 82 叶零死亡+picker/W-07 翻绿 | AC-04 |

- 实现worktree：`D:/autostack/.wt/plan-080/auto-ui`，分支 `codex/plan-080-dev`，
  基线 master@8389a1e5。
- 消费方交接：交付 pin+驱动/工具哈希回执 → READER-009 换 pin 终收口。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-080 | plan_revision: r1 | outcome: pass
  （可交 work；T-01/T-04 为有界调查带决策工件） | 起草：READER-009 会话
  （round49 先例 + round54 方向确认） | next: work（auto-ui 会话接手，
  T-01 插桩定谳起步）。

## 10. 待澄清事项

- mode A 自毁与 7d3a73aa evening commits 的因果（若 evening 线引入了自毁，
  owner-filter 修复后是否残留）——T-01 插桩覆盖。
- D3 S 区修复 diff 的提取源（7d3a73aa 线的具体 commit 范围）——T-04 前由
  auto-ui 会话确认。
- 若 mode A/B 与 auto-lang 段记账交互（bisect 辅项），升级为跨仓协调。
