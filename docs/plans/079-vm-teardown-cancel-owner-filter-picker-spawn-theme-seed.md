---
plan_id: auto-ui/PLAN-079
status: execution_done
feature_name: vm_bridge teardown-cancel owner-filter、picker 对话框生成稳定化与阅读主题种子链闭合
author: [relay/009-session]
created_at: 2026-11-11T00:00:00Z
updated_at: 2026-11-11T01:30:00Z
plan_revision: 1
current_step: 6
total_steps: 7
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-079：vm_bridge teardown-cancel owner-filter、picker 对话框生成稳定化与阅读主题种子链闭合

## 0. 变更摘要

三缺陷打包（消费方 READER-009 七臂二分定谳，2026-11-10/11）：

- **D1 teardown-cancel 错杀跨实例 parked 段**：`vm_bridge.rs:3575 cancel_parked_by_fn`
  按 fn_name 取第一个匹配取消——跨实例同名 handler（每张书卡同 fn_name）常态下
  会错杀活实例的 parked 段。受害者为最后活段→VM 判"全部结束"→会话无 host 退出
  请求而优雅自返；与 `rc_release_task_stack` 栈份额清账竞争→账务损坏→无 panic
  静默消失。引入提交=7d3a73aa（teardown-cancel 语义），该提交与 auto-lang
  938/995-era 组合致死（七臂 B/C 臂实证），故修复=**在 d0e1a688 基线上按
  `owner_state_id` 过滤重做 teardown-cancel 语义**（字段为 7d3a73aa 自有，数据已在
  ParkedTask 上）。
- **D2 picker 对话框生成 flake**：d0e1a688 bridge 上 `bookshelf_PickFiles` park 后
  对话框 45s 内不生成（009 run87/88 2/2；e0b6339a 上 3/3 秒弹）——有界插桩调查
  park→host bridge→dialog spawn 路径后修复。
- **D3 阅读主题种子链导航后未达**：宿主策略 dark_mode 在书架已种子（壳像素过），
  页面导航后 reading_page 的保留字段未播种（theme=light 且 theme_explicit=false，
  派生逻辑本身在位 reading_page.at:1439-1443）——runtime 种子链修复。

消费方解除物：READER-009 的 S 族/batch 区（D1）、picker 8 叶（D2）、theme
F-derive+roundtrip（D3）。

## 1. 目标

- D1：teardown-cancel 按 owner_state_id 精确取消；跨实例同名 handler 卸载负例
  （错杀红证）转绿；会话无退出请求自返/静默消失两死亡形态根除。
- D2：PickFiles park 后对话框在消费矩阵语境可靠生成（生成率修复后 ≥ 连续 3 次
  全量矩阵无 NO DIALOG）；根因留档。
- D3：宿主策略 dark_mode 在页面导航后种子到 reading_page 保留字段；无显式选择时
  阅读默认随宿主派生（059 合同 tier-2 follow）。
- 非目标：不重做 949/995 性能面；不动 7d3a73aa 中与本三缺陷无关的有效改动；
  不在致命组合（7d3a73aa × 938/995-era）上调参。
- 影响仓：auto-ui（本仓，唯一实现位）。约束：五个兄弟仓只读。
- 成功样态：009 消费矩阵（82 叶）在新 pin 上零死亡 + 本计划三 AC 全绿。

## 2. 架构方案

- D1：`vm_bridge.rs` ParkedTask 注册/取消路径增加 owner_state_id 维度——
  `cancel_parked_by_fn`（:3575）改为 `(fn_name, owner_state_id)` 二元匹配；
  实例卸载路径（:2555+ teardown-cancel）仅取消属主为本实例的 parked 段；
  补跨实例并发卸载压测（两实例同名 handler，卸载 A 断言 B 的 parked 段存活且
  迟到事件仍执行）。
- D2：有界插桩调查（T-04 决策工件）：在 park→bridge 请求→host pick_files
  能力→IFileDialog spawn 链逐点留痕，定位 45s 无窗口的丢失点；按定位结果修复
  （候选：bridge 请求被 teardown-cancel 类逻辑吞掉 / host 侧生成条件依赖前台态）。
- D3：runtime 在页面创建/导航时按宿主策略（AUTOOS_DESKTOP_CONFIG 派生 dark_mode）
  重播种新页面 widget 的保留字段（与壳根同字段同源）；059 的显式优先语义不变。

## 3. 技术栈

Rust（auto-ui-runtime vm_bridge / iced-winit 宿主）、现有 `cargo t` 测试组、
UIA 诊断链（消费方 009 已建 uia_act.ps1 + picker harness，验收消费复用）。

## 4. 需求分析与背景调查

- **授权**：READER-009 会话用户 2026-11-11 选择"我在此起草（推荐）"——本计划
  由 009 会话按 round27/28 先例代为立项；执行由 auto-ui 会话接手。
- **证据链**（全部在案）：
  - 七臂二分矩阵与死亡签名判据：
    `plan-009/handoff-autolang/RESPONSE-bisect-verdict.md`；
    驱动/锁/完整运行：`plan-009/t03/bisect-autolang/`（7 驱动 6 锁 6 运行）。
  - D1 代码锚点：vm_bridge.rs:3575（cancel_parked_by_fn 首匹配）、:2555+
    （teardown-cancel）、rc.rs:578-585 × generic_registry.rs:703-709（栈份额清账）。
  - D1 消费侧死亡证据：run81（batch 区 restore 503 后静默消失）、run86（S01 后
    拒绝连接）、七臂 B/C（自返形态）——t03/{ws-ui-vm-81,85,86,bisect-autolang}。
  - D2 证据：run87/88 picker 段 `dlg=NO DIALOG`（2/2）vs run80/82/84（e0b6339a
    3/3 生成）；probe-13 DIAG（IFileDialog 文件名框=DirectUI 无 HWND Edit——
    生成后的输入通道已由消费方纯 UIA ValuePattern 解决，本缺陷仅"生成"一步）。
  - D3 证据：009 theme-gate state-F.txt（书架 dark_mode=true 已种子、reading_page
    theme=light 且 theme_explicit=false）+ reading_page.at:1439-1443（派生逻辑
    在位）+ PLAN-059 交付合同（follow tier）。
- **约束**：基线=d0e1a688（069 终版，含 accesskit——七臂已洗清其致死嫌疑）；
  五兄弟仓只读；修复不得 reintroduce 7d3a73aa 的跨实例错杀。
- **相邻计划协调**：PLAN-068（executing，原生异步路由/结果投影/系统无障碍）——
  域相邻但任务不重叠（其遗留=自身消费关闭项）；本计划不动其任务。
- 观测 vs 假设：D1 的 fn_name 首匹配机制=源码实读（观测）；D2 丢失点=假设
  （T-04 有界调查定谳）；D3 种子时机=假设（状态转储实证"未达"，具体丢失环节
  待插桩）。

## 5. 详细设计

### D1 teardown-cancel owner-filter

- ParkedTask 已含 owner_state_id（7d3a73aa 自有）。取消路径签名改为
  `cancel_parked_by_fn(fn_name, owner_state_id)`；全部调用点（实例卸载/作用域
  终止/显式取消）传属主 id；仅 `task.owner_state_id == owner_state_id` 的段被取。
- 实例卸载路径（:2555+）同步改造；`rc_release_task_stack` 清账顺序在取消之后
  保持不变（竞争窗口由"只取消本实例段"收敛消除）。
- 负例测试（T-01 红证先行）：两实例同名 handler（同 fn_name 不同
  owner_state_id），A 卸载→断言 B 的 parked 段存活、B 的迟到事件仍执行且计数
  正确；A 卸载后无"全部结束"误判。

### D2 picker 对话框生成插桩与修复（有界调查→修复）

- T-04 插桩点：park 注册→bridge pick_files 请求发出→host 收到→host 调
  IFileDialog→HWND 创建。逐点时间戳留痕到工具宿主日志。
- 决策工件：定位丢失点后按点修复；若丢失点在 host 生成条件（前台态依赖），
  修复=生成不依赖前台或 park 前预检。

### D3 阅读主题种子链

- 页面创建/导航路径上，将宿主策略派生的 dark_mode 播种到新页面 widget 的
  保留字段（与壳根同字段同源）；显式选择（theme_explicit）语义不变、始终优先。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/runtime/component-and-store-instances.md | parked 段取消：按 fn_name 全局首匹配 → 按 (fn_name, owner_state_id) 属主匹配；实例卸载仅取消本实例段 | 跨实例同名 handler 错杀=会话死亡根因（七臂 B/C+run81/86） | AC-01 |
| SD-02 | modify | docs/specs/runtime/native-vm-app-validation.md | 宿主策略种子：页面导航后新页面 widget 保留字段按宿主策略重播种 | 无显式选择时阅读默认随宿主派生（059 follow tier）未达成 | AC-03 |
| SD-03 | add/modify | docs/specs/runtime/vm-route-page-loading.md（按 T-04 定谳落点） | pick_files 能力的对话框生成保证 | D2 消费方 picker 腿阻塞 | AC-02 |

## 6. 测试设计

- D1 负例：`cargo t`（plan069 相关组先例——round48 复审曾跑 plan069 13 组全绿）
  新增 `f2_teardown_cancels_only_owner_parked_segments`：两实例同名 handler、
  卸载 A→B 段存活+迟到事件执行+无"全部结束"误判。红证先行（现行为必红）。
- D1 压测：多实例（≥8）同名 handler 随机卸载循环 N 轮，断言无静默消失、
  账务守恒（既有守恒断言沿用）。
- D2：插桩日志证据 + 消费方 009 picker 段 3 连全量矩阵无 NO DIALOG。
- D3：vm 轨宿主策略→导航→reading state 断言 `theme: "dark"`（009 theme-gate
  F-derive 哨兵复用）。
- 组回归：受影响组并集（plan069 组 + 本计划新组）全绿；不跑全仓。
- 消费验收：READER-009 换 pin 后全矩阵（82 叶）零死亡 + picker 8 叶 +
  theme F-derive 翻绿（消费方 own）。

## 7. 验收标准

| AC | owner_stage | 内容 | 验证 |
|---|---|---|---|
| AC-01 | work | teardown-cancel 仅取消属主段：错杀负例红→绿；受影响组并集全绿 | cargo t 负例+组并集 |
| AC-02 | work | PickFiles park 后对话框生成可靠（3 连全量无 NO DIALOG）；根因留档 | 009 消费矩阵 picker 段 dlg 输出 |
| AC-03 | work | 宿主策略→阅读页派生哨兵绿（theme:"dark"，theme_explicit=false） | 009 theme-gate F-derive |
| AC-04 | review | 009 消费矩阵（82 叶）零死亡复证 + 三 AC 消费证据核验 | READER-009 换 pin 回归 |

## 8. 执行步骤

| 任务 | owner_stage | 依赖 | 内容 | 验证 | AC |
|---|---|---|---|---|---|
| T-01 | work | — | 错杀负例红证（现行为测试化：两实例同名 handler 卸载 A→B 段被误杀断言） | 测试红 | AC-01 |
| T-02 | work | T-01 | owner_state_id 过滤修复（cancel_parked_by_fn+卸载路径全部调用点） | 负例绿 | AC-01 |
| T-03 | work | T-02 | 多实例压测+受影响组并集回归 | 压测守恒+组全绿 | AC-01 |
| T-04 | work | — | D2 插桩调查（决策工件：丢失点定谳） | 插桩日志+定谳记录 | AC-02 |
| T-05 | work | T-04 | D2 按定位修复 | 009 picker 段 3 连生成 | AC-02 |
| T-06 | work | — | D3 种子链修复（导航播种） | 009 F-derive 哨兵绿 | AC-03 |
| T-07 | review | T-03/T-05/T-06 | 009 换 pin 消费回归（跨仓协作：READER-009 会话执行，本计划验收其证据） | 82 叶零死亡+三 AC 消费证据 | AC-04 |

- T-01..T-06 [x]（work 已完成，见 §9 r1 work 收据）；T-07 [ ] review-owned。

- 实现worktree：`D:/autostack/.wt/plan-079/auto-ui`，分支 `codex/plan-079-dev`，
  基线 d0e1a688。
- 消费方交接：交付 pin+驱动哈希回执 → READER-009 会话换 pin 终收口。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-079 | plan_revision: r1 | outcome: pass（可交 work；T-04 为有界调查带决策工件） | 起草：READER-009 会话（用户授权"我在此起草"） | next: work（auto-ui 会话接手，T-01 红证起步）。

- stage: work | plan_id: auto-ui/PLAN-079 | plan_revision: r1 | outcome: pass（execution_done；T-01..T-06 完成，T-07 review-owned 保持开放） | worktree: D:/autostack/.wt/plan-079/auto-ui | branch: codex/plan-079-dev | base_commit: d0e1a688 | code_commits: c241f796（D1 owner-filter + 负例/压测）、89c5b718（D2 插桩链）、08a5f320（D3 种子链 + 测试）、9dd3fc27（SD-01/02/03） | dependency_revisions: auto-lang@0b3fc9063（H 配方基线，只读检出 .wt/plan-079/auto-lang）、auto-web@bf72baab、auto-down@58ea33f | task_ids: T-01..T-06 | evidence:
  - D1 红证：`cargo nextest run -p auto-ui-runtime --lib f2_teardown_cancels_only_owner_parked_segments` 修复前 FAIL（幸存段 4000001≠B 的 4000002，错杀形态）；修复后 PASS。
  - D1 压测：`f2_multi_instance_random_teardown_keeps_live_segments` PASS（8 实例×8 轮随机卸载：活段守恒/受害者挂载 stake 恰释放一次/根态 RC 不动）。
  - D1 组回归：130 tests PASS（vm_bridge 模块 + plan069/702/711/841/056/041/370 并集）。
  - D2 决策工件：plan-artifacts/plan-079/D2-decision-artifact.md（丢失点=teardown-cancel 跨实例错杀，与 D1 同根；7d3a73aa diff + 版本相关性 + 009 round49 收据）；插桩 AUTO_DIALOG_DIAG=1 四点就绪（park/cancel/release/host）。
  - D3：`f2_navigation_seeds_host_policy_dark_mode` PASS（light→dark→light 三导航播种 + 未声明字段负控）；组回归 194 tests PASS（vm_bridge + theme/plan370/056/069/711/osconfig 并集）。
  - SD-01/02/03 canonical 增量在 worktree 提交 9dd3fc27（未发布，merge 落地）。
  | blockers: 无（AC-02 的 009 picker 3 连矩阵与 AC-03 的 F-derive 哨兵、AC-04 全矩阵归 T-07/review 消费验证） | next_responsible_role: review（独立 review 对 9dd3fc27 复审；T-07 跨仓消费回归由 READER-009 会话执行，本计划验收其证据）。

- stage: goal | plan_id: auto-ui/PLAN-079 | plan_revision: r1 | round: 1 | from: new | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/plan-079/auto-ui | commit: 9dd3fc27 | next: review

- stage: review | review_scope: internal（计划 §8 指定 T-07=READER-009 会话为终审验收执行者；本收据为 work-owned 判定的独立工件复核，同实现会话执行、判定从 Plan/代码/diff/测试在盘推导） | plan_id: auto-ui/PLAN-079 | plan_revision: r1 | outcome: pass（work-owned 判定；终态 `reviewed` 待 T-07 消费证据验收） | worktree: D:/autostack/.wt/plan-079/auto-ui | branch: codex/plan-079-dev | reviewed_commit: 9dd3fc27 | base_commit: d0e1a688 | dependency_revisions: auto-lang@0b3fc9063（只读检出 .wt/plan-079/auto-lang，未改动）、auto-web@bf72baab、auto-down@58ea33f | spec_inputs: docs/specs/runtime/component-and-store-instances.md（sha 1731317a）、native-vm-app-validation.md（sha 8f7ab2ca）、vm-route-page-loading.md（sha 3d532167），均未发布、merge 落地 | acceptance_results:
  - AC-01 **pass**：`f2_teardown_cancels_only_owner_parked_segments` 修复前 FAIL（幸存 4000001≠B 4000002，错杀形态实证）→ 修复后 PASS；`f2_multi_instance_random_teardown_keeps_live_segments` PASS（8×8 随机卸载：活段守恒/受害者挂载 stake 恰释放一次/根态 RC 不动）；冻结并集 211 tests PASS（vm_bridge + plan069/702/711/841/056/041/370/theme/409/osconfig/dialog/plan804）。
  - AC-02 work 侧 **pass**（丢失点定谳 + 插桩 AUTO_DIALOG_DIAG ①-④ + 段存活不变量的永久反例）；009 picker 3 连全量矩阵 **pending**（T-07）。
  - AC-03 work 侧 **pass**（`f2_navigation_seeds_host_policy_dark_mode`：light→dark→light 三导航播种 + 未声明字段负控）；009 theme-gate F-derive 哨兵 **pending**（T-07）。
  - AC-04 **pending**（009 换 pin 全矩阵 82 叶零死亡 + 三 AC 消费证据，T-07）。
  | findings: 无 needs_fix。注记：cancel_parked_by_fn 的 PLAN-711 原文档首句仍为旧签名措辞（下文已补 D1 段落），非语义问题。 | pending_closeout_gates: T-07（READER-009 换 pin 消费回归：82 叶零死亡 + picker 3 连无 NO DIALOG + theme F-derive 翻绿；驱动/代码 pin = 本 branch @ 9dd3fc27 + auto-lang 0b3fc9063） | evidence: 见 r1 work 收据（红证/压测/组回归命令与结果在盘）；本收据复核了 4 提交 diff（c241f796/89c5b718/08a5f320/9dd3fc27）、worktree 洁净度、依赖检出未改动、无 ledger/specs.json 写入 | next: review（T-07 消费方 READER-009 执行；其 round49 已登记"等 auto-ui owner-filter 修复交付后消费修复版+picker 门接回"）。

- stage: goal | plan_id: auto-ui/PLAN-079 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass（内部 review：work-owned 判定 pass；T-07 消费验证 pending） | worktree: D:/autostack/.wt/plan-079/auto-ui | commit: 9dd3fc27 | next: review（T-07/READER-009）

## 10. 待澄清事项

- 7d3a73aa 中与本三缺陷无关的有效改动是否随其回退一并丢失→本计划仅重做
  teardown-cancel 语义，其余改动不在本计划范围（如发现依赖请在 T-02 前提出）。
- auto-lang 辅助复核（0b3fc9063..36b5fbd4a 段记账交互）归 auto-lang 会话，
  不在本计划任务内。
- PLAN-068 仍 executing（域相邻）：若其后续任务触碰 vm_bridge 同区，需与本计划
  协调写入顺序。
