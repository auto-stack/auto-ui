---
plan_id: auto-ui/PLAN-074
status: executing
feature_name: RQ桌面broker外部CLI客户端attach闭合（OS063 DG-1修复）
author: [Codex]
created_at: 2026-11-09
updated_at: 2026-11-09
plan_revision: 1
current_step: 1
total_steps: 5
depends_on: [auto-ui/PLAN-063, auto-ui/PLAN-073]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [desktop-broker-attach, client-entry, rq-execution-input-and-host-services]
---

# PLAN-074：RQ桌面broker外部CLI客户端attach闭合（OS063 DG-1修复）

## 0. 变更摘要

闭合 auto-os/PLAN-063 §10 **DG-1**：外部 CLI 客户端（`auto run --render-mode=desktop
--desktop-endpoint=<broker> --autodesk-incubate --app386=<注册表id>`）向桌面会话 broker
请求孵化被收纳（incubation app-N 排队）后，宿主 `attach_pending_incubations(5000)` 预算内
**零收到客户端消息**，未达 Active → 弃连 → 客户端 `exit=HostLost`。in-test 程序化 attach
路径全绿——缺口限定在外部 CLI 客户端形态的 attach 机械。本 Plan 定位根因（红线测试钉住）、
修复、并以真实桌面端到端闭合 DG-1，交还 OS063 产品嵌入战役。

## 1. 目标

修复后：外部 CLI 客户端按上述方言在桌面 broker 上 5s 预算内达 Active、虚拟窗落地、
双向帧/输入通——OS063 的逐产品桌面嵌入战役（N=1/4/8、M-02/06/07 终值）得以恢复。
不改动：broker 收纳协议（incubate DesktopBus 记录/专用管道分配）、attach 预算值、
rqhost adopt 臂、shell outproc 臂、产品业务、OS 仓。

## 2. 架构方案

单写面 = `crates/auto-ui-runtime`（client_entry/attach 机械；若根因落在 Man CLI 参数
面则加 `auto-man` crates/auto 为第二写面并在 §4 记录）。红线测试钉根因：client_entry
层级的 VM 组件 + `ClientTarget::Broker` 对真实 in-proc broker+DesktopSession harness
的 attach Active 断言（今日必红）。修复实现→独立review→merge 依次同一权威实施树；
拟用 `D:/autostack/.wt/os-063/auto-ui`（branch `auto-os-dev`，延用 os-063 组树——
PLAN-063 T-03/F-1 已在该树交付，谱系连续）；本 Plan 不新建组。

## 3. 技术栈

Rust、现有 desktop_protocol 传输/端点状态机；测试沿 nextest sanctioned 上下文
（.config/nextest.toml）+ 仓内既有 cargo 入口。无新依赖。

## 4. 需求分析与背景调查

### 授权

2026-11-09 用户裁定（OS063 goal 会话转达）：开修复计划修复 DG-1。实施与合并随本
Plan 全链授权（work→review→merge）。前置：UI063（96236725）、UI073（82b61084）均已
交付归档；OS063 组 UI 树同步至 bcceb419+（auto-os-dev 17880ccb）。

### 诊断事实（OS063 2026-11-09 实测，三形态全参数复现）

- 宿主侧：`[autodesk-broker] incubation ... failed to reach Active (budget)`——无
  `attach on_message err` / `attach codec err` 行 = `broker_attach_one`
  （session.rs:4683）在 5s 预算内 `end.try_recv()` **零消息**（非解码/协议错）。
- 客户端侧：`request_incubation_render`（broker.rs:214）成功——连主管道 → 发
  `incubate` DesktopBus 记录 → 收专用管道名 → `transport::connect(app-N)` 返回
  专用连接。此后客户端静默（无装载/编译/错误日志），最终 `exit=HostLost
  revision=0/1`（revision=1 次 = 泵至少建过一帧）。
- 对照组：t3/stress harness（stage3.rs:3549 spawn_children）为测试二进制 re-exec +
  进程内程序化连接——**不经 CLI client_entry**，故全绿不覆盖本缺口；生产注册表路径
  （桌面自 spawn）日常可用，未被本诊断触碰。
- 疑点（T-01 定案）：`run_client_entry`（auto-man cmd_autodesk.rs:50）只解析
  `--autodesk-render`（非 `--render`）/`--app386`/`--autodesk-broker`；VM 孵化形态
  的 target/frame_mode/remote 三态裁决、组件装载时序与 app_end 交接臂为首要嫌疑
  （如泵启动前等 Welcome、或 app_end 未交入该形态的会话循环）。
- 环境：OS063 隔离基座（probe-root 裁剪 manifest）boot 绿、release 产物
  auto `114eb679…`；复现日志 plan-artifacts/plan-063/auto-os/t04-calc*.log。

## 5. 详细设计

T-01 红线测试先行（允许红提交），修复以最小 diff 落在根因臂；不动协议词汇与
attach 预算。三态裁决矩阵（spawn 参数/pac/auto 探测 × rq/desktop/independent）在
修复后逐一过线级断言。若根因涉及 `--render-mode`/`--render` 参数归一，Man CLI 侧
修复须保持 693 前 exe 的孵化臂兼容（dual dialect 不回退）。

### 任务落点

T-01/T-02 修改 `crates/auto-ui-runtime/src/ui/desktop_protocol/{client_entry,client_runtime,broker}.rs`
与（如需）auto-man `crates/auto/src/cmd_autodesk.rs`；T-03 为真实桌面端到端验证
（复用 OS063 probe-root 基座与启动环境，见其 runbook）。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/runtime/rq-execution-input-and-host-services.md §1 | 外部 CLI 客户端孵化臂"收纳后 attach Active"合同从隐式转显式（含线级判据） | 未在本页写明的行为不视为已实现 | AC-01..03 |

## 6. 测试设计

| Case | AC | 输入/真实入口与负控 | 可观察oracle/关闭条件 | 责任任务 |
|---|---|---|---|---|
| V-01 | AC-01 | client_entry 层 VM 组件 + Broker target 对 in-proc broker+session harness（t3 同款真实管道对） | attach 预算内 Active + wid 落地；修复前必红（红线提交留档） | T-01/T-02 |
| V-02 | AC-02 | 真实桌面 boot（probe-root）+ 外部 CLI 客户端全方言 | Active、虚拟窗落地、双向帧/输入通；双侧日志审计锚 | T-03 |
| V-03 | AC-03 | sibling 臂：rqhost adopt、shell outproc、ClientTarget::Desktop adopt、dual dialect 旧 exe 兼容 | 既有族门全绿（desktop_protocol nextest sanctioned + ui::session），零行为回归 | T-02 |
| V-04 | AC-01 | 负控：端点缺席/错名/预算耗尽仍具名失败（不吞错） | 具名错误行与现行为一致 | T-02 |

## 7. 验收标准

| ID | owner_stage | 可观察行为/判据 |
|---|---|---|
| AC-01 | work | 红线测试修复前红/修复后绿，绑定根因臂 file:line；负控 V-04 具名 |
| AC-02 | work | 真实桌面 + 外部 CLI 客户端 attach Active、虚拟窗落地、帧/输入双向通（DG-1 关闭） |
| AC-03 | work | sibling 族门零回归（desktop_protocol nextest sanctioned + ui::session 全绿） |
| AC-04 | work | dual dialect 旧 exe 兼容不回退（spawn_exe_child_args 双方言测试仍绿） |
| AC-05 | review | 独立上下文复核 AC-01..04 与真实源；OS063 DG-1 关闭条件满足确认 |
| AC-06 | merge | 落地/归档/清理 + OS063 消费回执（其战役恢复） |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/操作与结果 | 关闭方式/AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 可立即（诊断事实充分） | 红线测试 V-01（必红留档）+ 根因定位记录（臂/行） | V-01；AC-01 |
| [ ] | T-02 | work | T-01 | 最小 diff 修复；V-01 绿；sibling V-03/V-04 族门 | AC-01/03/04 |
| [ ] | T-03 | work | T-02 | 真实桌面端到端 V-02（OS063 基座）+ SD-01 草案 | V-02；AC-02 |
| [ ] | T-04 | review | T-03 | 独立上下文复核（AC-05） | AC-05 |
| [ ] | T-05 | merge | T-04 pass | 落地/归档/清理 + OS063 回执 | AC-06 |

## 9. 复审记录

2026-11-09 / stage:new / plan_id:auto-ui/PLAN-074 / revision:r1 / outcome:pass（草案合同）
/ next:work T-01（全链授权在案）。静态自检：0-10 章齐、T=5、AC=6、V=4；诊断事实全部
绑定 OS063 实测日志；红线先行纪律在案。

## 10. 待澄清事项

- 无阻塞性待决。根因落点（client_entry vs Man CLI）由 T-01 定案；若落在 Man CLI，
  双写面纪律按 §2（同组树 + auto-man 组树）。

### 2026-11-09 work 进展（T-01 红线闭合）

- stage:work / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:pass（T-01） / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / base_commit:bcceb419 / code_commit:c2254800 / task_ids:T-01 / evidence:plan-artifacts/plan-063/auto-os/t04-campaign-runbook.md（PLAN-074 节） / blockers:无 / next_responsible_role:work（T-02 transport 层修复）
- 红线测试 `p074_broker_incubate_vm_client_attach_active` 落地并按预期红（5s 确定性：
  pending 出现 → attach 预算耗尽 → 0 落地）。逐层插桩（临时，已撤）定位：客户端 Hello
  已发出（send 成功返回），宿主 pending 端 5s 零读——缺陷在传输配对层。T-02 断点：
  transport.rs 最小二线程 loopback 复现 → serve_once wait_connect 端差异对照。
