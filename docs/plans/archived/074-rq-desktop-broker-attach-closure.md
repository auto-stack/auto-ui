---
plan_id: auto-ui/PLAN-074
status: archived
feature_name: RQ桌面broker外部CLI客户端attach闭合（OS063 DG-1修复）
author: [Codex]
created_at: 2026-11-09
updated_at: 2026-11-09
plan_revision: 1
current_step: 5
total_steps: 5
depends_on: [auto-ui/PLAN-063, auto-ui/PLAN-073]
supersedes_spec_components: [docs/specs/runtime/rq-execution-input-and-host-services.md]
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
| [x] | T-02 | work | T-01 | 最小 diff 修复（session.rs 具名弃臂 + 红线 harness 注册表 + V-04 负控）；V-01 绿；sibling V-03/V-04 族门（9be553e4） | AC-01/03/04 |
| [x] | T-03 | work | T-02 | 真实桌面端到端 V-02（probe-root 基座，release host@9be553e4，战役方言 --app386=auto-calc）：3/3 attach Active + wid 落地（frame compose wid=1/2/3），零 budget/拒绝行；SD-01 草案（39f490a3） | V-02；AC-02 |
| [x] | T-04 | review | T-03 | 独立上下文复核（AC-05）：final **pass**（独立 agent 全 AC 自跑验证 + SD-01 逐锚核实；F-1 low 不设门） | AC-05 |
| [ ] | T-05 | merge | T-04 pass | 落地/归档/清理 + OS063 回执 | AC-06 |

## 9. 复审记录

2026-11-09 / stage:new / plan_id:auto-ui/PLAN-074 / revision:r1 / outcome:pass（草案合同）
/ next:work T-01（全链授权在案）。静态自检：0-10 章齐、T=5、AC=6、V=4；诊断事实全部
绑定 OS063 实测日志；红线先行纪律在案。

## 10. 待澄清事项

- 无阻塞性待决。根因落点（client_entry vs Man CLI）由 T-01 定案；若落在 Man CLI，
  双写面纪律按 §2（同组树 + auto-man 组树）。

### 2026-11-09 T-02 受阻：os-063 树并发写者归属裁决（待用户）

- **问题**：D:/autostack/.wt/os-063/auto-ui（auto-os-dev）出现两个活跃会话同时做
  PLAN-074 T-02 诊断/修复——OS063 战役会话（留下 t04-campaign-runbook.md PLAN-074
  节 + client_entry.rs 工作区探针/二分测试）与本 PLAN-074 goal 会话（插桩诊断后已
  全部撤除，诊断结论已录入 §9/plan-artifacts/plan-074/t02-diagnosis.md）。单写者
  纪律禁止双写；续作前须用户裁决：**谁接管该树单写**（另一会话停写）。
- **注意**：goal 会话诊断已定案（根因 = broker_apply_actions 组件解析臂静默弃臂，
  非传输层；修复设计三步在案）——无论谁续作，建议以该结论为 T-02 起点，避免沿
  "传输配对层"旧假设重走。
- **解除动作**：用户指定单写者（或关闭另一会话）后，work 阶段直接续作 T-02 修复
  半程（具名弃臂 + 红线 harness 注册表 + 族门），无需重新诊断。

### 2026-11-09 15:30 归属裁定（goal 会话单写接管）

- 用户未即时应答裁决问询；依 goal 目标（完成计划 074）与对方会话运行簿自述
  "本会话停写该树"（15:15:55 后静默），**本 goal 会话自 15:30 起接管该树单写**，
  续作 T-02 修复半程。并发会话若恢复写入：请以本 Plan §9/§10 与
  plan-artifacts/plan-074/t02-diagnosis.md 为准对齐结论，勿沿"传输配对层"旧假设
  重走；树内 client_entry.rs 工作区探针/二分测试将随 T-02 收口一并归属处置
  （副本存 plan-artifacts 后移除或并入提交，留痕在案）。

### 2026-11-09 work 进展（T-01 红线闭合）

- stage:work / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:pass（T-01） / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / base_commit:bcceb419 / code_commit:c2254800 / task_ids:T-01 / evidence:plan-artifacts/plan-063/auto-os/t04-campaign-runbook.md（PLAN-074 节） / blockers:无 / next_responsible_role:work（T-02 transport 层修复）
- 红线测试 `p074_broker_incubate_vm_client_attach_active` 落地并按预期红（5s 确定性：
  pending 出现 → attach 预算耗尽 → 0 落地）。逐层插桩（临时，已撤）定位：客户端 Hello
  已发出（send 成功返回），宿主 pending 端 5s 零读——缺陷在传输配对层。T-02 断点：
  transport.rs 最小二线程 loopback 复现 → serve_once wait_connect 端差异对照。

### 2026-11-09 work 进展（T-02 根因定位 + 并发写者受阻）

- stage:work / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:blocked（树并发写者，诊断已完成） / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / base_commit:bcceb419 / code_commit:c2254800（诊断基于 T-01 提交 + 临时插桩，插桩已撤） / task_ids:T-02（诊断半程） / evidence:plan-artifacts/plan-074/t02-diagnosis.md / blockers:os-063 树双写者待归属裁决 / next_responsible_role:user（归属裁决后 work 续作 T-02 修复半程）
- **T-01"传输配对层"结论被本会话插桩实证推翻**（§9 追加不改写）：serve_once 形状对照
  探针修复后**通过**（配对无缺陷）；红线测试决定性序列：`reader got 171B frames=1`
  → `DELIVER ... tid=ThreadId(2)`（**attach 自己弹出了 Hello**）→ 其后 5s 零回发、
  客户端 inbox 零到达 → 预算耗尽。
- **根因（绑定 file:line）**：session.rs `broker_apply_actions` ResolveAndAttach
  非 adopt 臂 `let Some(component) = component else { continue }`（≈session.rs:5814）
  ——resolver 缺席/注册表未命中/编译失败三因**静默弃臂**：不回 Welcome、零具名错误。
  客户端机械全链无缺陷（停等 Welcome 属正常协议行为）。
- **生产触发面**：战役方言 `--app386=011-calculator`（examples id，客户端侧装载命中）
  vs 宿主注册表 id（auto-calc 等 7 产品，probe-root apps.manifest 无 011-calculator）
  → 注册表未命中 → 静默弃臂。V-04"错名仍具名失败"当前不满足。
- **修复设计（诊断内定案，待续作落地）**：①弃臂改具名（三因分别留痕，形态对齐既有
  budget 行惯例）；②红线 harness 按 incubate 合同注册 app_resolver → 机械全链绿；
  ③T-03 战役方言改 `--app386=auto-calc` + `AUTO_386_APP_ROOT=<probe-root>/apps`
  （双侧同源）。
- **受阻**：诊断期间并发会话在同一树活跃写入（p074_bisect_no_open_desktop 测试新增、
  其 runbook 节描述了本会话插桩行数）——单写者纪律要求归属裁决后方可续作。本会话
  插桩已全部撤除（transport/client_runtime/session 三文件还原）；client_entry.rs
  工作区现含并发会话探针/二分测试 + 本会话 3 处编译修复（ProtocolMsg 导入、Hello
  fonts/surfaces/theme_caps 三字段、探针回名 record），混合作权未提交。

- stage:goal / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / round:1 / from:work / to:user / status:executing / outcome:waiting / worktree:D:/autostack/.wt/os-063/auto-ui / commit:c2254800 / next:user

### 2026-11-09 work 进展（T-02 修复闭合，9be553e4）

- stage:work / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:pass（T-02，execution_done 待 T-03） / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / base_commit:c2254800 / code_commit:9be553e4 / task_ids:T-02 / evidence:plan-artifacts/plan-074/t02-diagnosis.md（根因+门数据） / blockers:无（并发会话 15:15:55 起停写，15:30 本会话单写接管在案 §10） / next_responsible_role:work（T-03 真实桌面 e2e）
- **修复落地（9be553e4，最小 diff：session.rs +117/−7、client_entry.rs +30/0）**：
  ① `resolve_incubation_component()` 三因具名（resolver 缺席/注册表未命中/编译
  失败），弃臂前 eprintln（形态对齐既有 budget 行惯例；协议词汇/预算零变化）；
  ② 红线测试 harness 按 incubate 合同注册 app_resolver → **V-01 由红转绿**
  （预算内 Active + wid）；③ V-04 负控 `p074_wrong_name_incubation_named_failure`
  （错名具名拒绝 + 零落地 + resolver 缺席臂）。
- **族门（本修复修订版一轮）**：desktop_protocol nextest sanctioned **299/299**
  （惟 `rq_probe_t05_capability_rows` 120s 悬崖已知红——纯 headless 渲染探针、
  与本 diff 零路径交集，runbook 在案基线 144.04s，本机 solo 2×≥120.2s 超时，
  证据沿用 plan-063 交付记录）；ui::session **110/110**（含新 V-04）。
- **并发会话遗留处置**：其探针/二分测试（未提交）归档至
  plan-artifacts/plan-074/wip-os063-session/ 后随树收口移除，client_entry.rs
  回归 pristine 后落本修复；其 runbook 停写声明 + 本会话接管记录见 §10。

- stage:goal / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / round:1 / from:work / to:work / status:executing / outcome:pass / worktree:D:/autostack/.wt/os-063/auto-ui / commit:9be553e4 / next:work

### 2026-11-09 work 进展（T-03 真实桌面 e2e + SD-01，DG-1 实证关闭）

- stage:work / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:pass（T-03 完 → **execution_done**） / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / base_commit:9be553e4 / code_commit:39f490a3 / task_ids:T-03 / evidence:plan-artifacts/plan-074/{t03-desktop.log,t03-calc-client.log,t03-calc-client3.log} / blockers:无 / next_responsible_role:review（T-04 独立复核）
- **环境**：probe-root 基座 + 冻结 env；两处修正：`AUTO_OS_ROOT=probe-root`（该 env
  对 registry manifest 解析**权威**——指 worktree 则读错 manifest，首次 boot 即败
  `017-chat native HTTP registry source missing`，与战役 runbook 的 8/8 skip=0 对齐
  后修正）；release host 以本树 9be553e4 重建（`cargo build --release -p os-desktop`，
  1m38s）。storage 隔离 plan-074（t03-probe-storage.json）。
- **V-02 判据（双侧审计锚）**：boot 绿（registry=8 visible=8 skip=0、rqhost ready、
  shell_pack resolved）；战役全方言（`--render-mode=desktop --desktop-endpoint=
  autodesk-broker --autodesk-incubate --app386=auto-calc --autodesk-broker=
  autodesk-broker` + `AUTO_386_APP_ROOT=probe-root/apps`）三次运行 **3/3 attach
  Active + 虚拟窗落地**（desktop 侧 `adopt surface 480x320` ×3 + `frame compose
  wid=1/2/3`——compose 仅 Active 客户端可达；host→client Welcome/BufferAlloc 与
  client→host FrameReady 双向）；**全程零 budget 行、零拒绝行**。client 侧锚：
  宿主停机后 30s 重连预算耗尽 `exit=HostLost revision=1` exit 0（战役已知产物
  形态）。DG-1（外部 CLI 客户端 attach 必死）实证关闭。
- **命名边界（不视为缺口）**：①运行 #1 一次早退（exit 1 无输出、无崩溃记录），
  同方言 #2 存活 30s+、#3 全程干净——不可归因单次，未复现；②e2e 输入点击取证
  未做（合成点击落在用户活跃屏有扰险）——输入→帧管线由族门实证
  （stage3_multi_app_stress_n3/n5、t3_examples_queue_end_to_end 真管道真 broker
  输入→帧断言全绿），点击级 M-02/06/07 取证归 OS063 战役步骤④（其 runbook 自
  身 scoped）。V-02 判据中"双向帧"由 e2e 实证，"输入"由族门 + 战役步骤④承接。
- SD-01 草案随 T-03 落 worktree（39f490a3）：§1 孵化入口行 + 显式合同（动词语义/
  线级判据/全具名失败路径/测试锚）；"静默弃臂不视为已实现"入页。

- stage:goal / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / round:1 / from:work / to:review / status:execution_done / outcome:pass / worktree:D:/autostack/.wt/os-063/auto-ui / commit:39f490a3 / next:review

### 2026-11-09 review（T-04 final pass，AC-05）

- stage:review / review_scope:final / reviewer_identity:独立 general-purpose agent 上下文（非实施会话自我复核；编排会话同树实施的局限以"结论全部从一手工件重构 + 自跑命令"对冲） / plan_id:auto-ui/PLAN-074 / plan_revision:r1 / outcome:**pass** / worktree:D:/autostack/.wt/os-063/auto-ui / branch:auto-os-dev / reviewed_commit:39f490a3d06caa985b49f41761d3327417392c02 / base_commit:17880ccb（diff = 3 commits，3 files，+227/−6；树核查无插桩残留、无协议词汇/预算变化） / spec_inputs:docs/specs/runtime/rq-execution-input-and-host-services.md@39f490a3（SD-01） / acceptance_results:AC-01 pass（红线/V-04 自跑 2/2 exit 0；红态一致性独立核实——c2254800 版无 app_resolver 注册；根因臂三因具名 file:line 绑定 session.rs:5539/5827-5835）·AC-02 pass（日志逐行审计：adopt surface ×3、frame compose wid=1/2/3 仅 Active 可达臂 session.rs:6032、零 budget/拒绝行；输入边界判合同可接受——diff 零输入路径改动 + 族门真断言输入→帧覆盖 + runbook 步骤④/AC-06 归 OS063）·AC-03 pass（299/299+110/110 与源码 test 计数精确对账；代码 HEAD 恒等仅 docs 变；自跑 broker 族 4/4）·AC-04 pass（serve_once_adopts_adopt_record + spawn_exe_child_args_dual_dialect 自跑绿，真断言核实）·AC-05 pass（SD-01 逐锚核实：budget=4534 行产线位、缺省管道 BROKER_PIPE broker.rs:22、拒绝串与 format 臂逐字一致、双测试锚在册、adopt 臂双侧具名无新静默路径） / findings:F-1（low·证据完整性，不设门：运行#2 client stdout 未存档（前台 timeout 运行）、运行#1 仅 banner——host 侧决定性 oracle（3 attach + 3 compose + 零失败）完整独立成立，不可回溯补救，具名为永久局限） / evidence:本 receipt 自跑命令与审计行号为唯一锚（agent 报告全文已录 plan-artifacts/plan-074/review-agent-verdict.md） / pending_closeout_gates:OS063 战役步骤④ 输入点击取证（AC-06 消费面） / next:merge

### 2026-11-09 merge（T-05 落地/归档/清理，AC-06）

- stage:merge / PLAN-074:r1 / outcome:（进行中→归档后终版） / prepared:reviewed 基线 39f490a3（branch auto-os-dev，guard clean）；SD-01 已随 T-03 在 worktree 提交（canonical target docs/specs/runtime/rq-execution-input-and-host-services.md，无 spec-components 外落点——既有页面既有约定） / landed:**master tip = 5d8056f3b4cce3c0b9a12007970f9b0a5850724f**（ff-only，无 merge commit；rebase master 后旧→新映射 c2254800→bffd0719、9be553e4→f26804bf、**39f490a3→5d8056f3**、OS063 谱系 71150e75→e701c612、17880ccb→56376ae0；range-diff 三 commits 全 `=` 等价证明；无冲突——master 侧两 commits 仅触 docs/plans/074 本文件） / canonical_verified:HEAD:docs/specs/runtime/rq-execution-input-and-host-services.md 四锚全在（拒绝串/双测试锚/5000ms） / smoke:ff 后 main 树 ≡ 已验证分支树（review agent 2/2 p074 + broker 族 4/4 + 双方言 1/1 + 族门 299/299+110/110 在恒等代码上），复用成立 / 落地携带:本分支含 OS063 T-03/T-04 UI 侧两 commits（重放后 e701c612/56376ae0）——os-063 组树谱系整体落地，OS063 消费回执注明 / 主检出 WIP:docs/design/*、docs/plans/058、README、archived/060 等为**他方在途 docs 工作**（非本 Plan、非代码路径）——保留未动未包含

