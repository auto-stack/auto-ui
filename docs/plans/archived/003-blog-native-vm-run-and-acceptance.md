---
plan_id: PLAN-003
status: archived
feature_name: blog-native-vm-run-and-acceptance
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 2
current_step: 7
total_steps: 7
completion_kind: delivered
depends_on: []
supersedes_spec_components: []
new_spec_components: ["docs/specs/runtime/native-vm-app-validation.md"]
touched_goals: []
---

# PLAN-003：Blog原生VM窗口运行与可交互验收前置

## 0. 变更摘要

由[BLOG-004](D:/autostack/auto-blog/docs/plans/004-dual-runtime-acceptance.md)登记的原生UI前置：诊断真实Blog作者窗口的启动/驱动缺口，证明框架的真实原生窗口能接收输入并可重复退出重开；完整Blog业务任务由004闭合。与本仓001/002的AutoGraphics生命周期/触发工作无依赖，不重开它们。不是通用桌面自动化大改，也不替代Blog最终Rust组合门。

## 1. 目标

区分交互桌面环境、CLI/工具交接、MCP驱动、UI runtime与应用.at接缝；仅修实际owner内的最小阻断，提供可由Blog复用的原生窗口启动/交互/清理合同。缺可交互桌面时保留blocked条件，不能以HTTP可达或窗口截图当作可操作通过。

## 2. 架构方案

主改面为auto-ui `crates/auto-ui-tools/src/{run_vm,proc_host}.rs`、bin/auto_ui_tool.rs及auto-ui-runtime动态UI/真实MCP路径（后者具体符号先调查）；不在UI复制应用草稿存储或修改Rust后端合同。Man编排根因由其真实owner另立最小前置，不在UI假接线。

主检出master只持Plan簿记；唯一实现/顺序复审树拟为 `D:/autostack/.wt/ui-003/auto-ui`，分支 `codex/plan-003-dev`。框架同组只读依赖按Cargo真实兄弟布局固定，需依赖worktree时按owner规范建立，禁止junction/symlink。Blog开发/验收仍在 `D:/autostack/auto-blog` 单写者clone，UI计划不写其业务代码。

## 3. 技术栈

现有AutoUI原生VM/iced、auto-ui-tool、proc_host与可用MCP。已读run_vm.rs说明托管时VM主线程hook、未托管时CLI主线程；不得未实测就归因为“环境问题”。实测原生前端命令为 `auto run -r vm`，配独立数据根/端口；`--server vm` 选择后端，不选择VM窗口。控制夹具声明 `api: "rust"`，以生成的真实Rust HTTP API验证事件到达；实际Blog作诊断消费者，必要时用T-02约简的非空原生控件/API夹具验证框架正控（不是mock窗口）。Blog最终仍须完整实际应用＋Rust后端门，夹具不能替代。

不硬编码猜测的MCP方法名。先读取实际工具协议/驱动入口，固定探活、窗口身份、动作与状态观察、超时/清理的可重放序列；原生窗口生命周期要有有界等待，失败保留日志。

## 4. 需求分析与背景调查

用户2026-10-29明确授权“建立Blog后续计划，专门补齐Rust后端和原生VM界面的运行与验收，关联必要框架前置计划，并更新总跟踪表”。本轮只编制合同、必要反链与组合pin，不实施代码、不运行全局Cargo门、不宣布运行缺口已关闭。后续执行须按四技能进入work；未指定模型、独立会话、自动续跑或并行agent。

Goal授权（2026-10-29）：用户请求“`/goal 完成计划UI-003`”，沿用现有Goal并授权在本计划既定范围内依次work、review、merge、规范沉淀、归档与安全清理。主检出中的无关docs WIP保持原样，实现在已登记的唯一worktree；不新建会话/agent，不改Blog或AutoLang owner代码。

UI基线 `78ee3b70c38697e51181e296eb91e88ea1e761b4`；现有attachments/002/在途材料保持。知识权威沿语言仓 `docs/specs/auto-ui/project.md`（生产owner迁出）、`docs/specs/auto-ui-compiler/project.md`；本仓当前无docs/specs，拟新增的是小范围原生应用验收合同，不搬迁所有历史知识。

- `crates/auto-ui-tools/src/run_vm.rs` — SHA256 `075b17fdf4dc12d883f8a1e78c50d23cfb1d6b286043afda8faf5c72abd6235c`。
- `crates/auto-ui-tools/src/proc_host.rs` — SHA256 `bc3dc50ecdcd54e434a04e698909b0b63b9b90d5184b8e5cd918358ff7cb0d76`。
- `crates/auto-ui-tools/src/bin/auto_ui_tool.rs` — SHA256 `198838b2931d2228ccfb216b28e501d6d6fb23fe10cecd186d0a715d8967fbe3`。
- `crates/auto-ui-runtime/src/ui/dynamic.rs` — SHA256 `62e560deae710a2e878d5c7bab8b15476f3b7930447c1c25a861b47b82d02ac2`。

001历史原生VM四轮不可驱动为调查输入；本次只核对源与CLI --help，没有新跑原生窗口。不能从Notes/Todo成功推导Blog路由/作者窗口必然可用，也不能把Blog使用VM后端的HTTP记录当原生UI证据。

## 5. 详细设计

### 首任务与关闭合同

T-01无Rust交付前置：固定应用/CLI/UI/lang版本、可交互会话条件和驱动协议，分段检查CLI→tool进程→VM主线程→窗口/root→实际输入事件→store/API。拟新增 `docs/plans/attachments/003/native-path-decision.md` 记录实际路径/owner、日志和可重复操作。将“进程启动即退”“驱动发现不到”“事件不进widget”“应用路由不兼容”等分别归因，不用无界重启掩盖。

若当前无框架缺陷：以有效真实窗口正控＋退出重开和故障传播证据形成不需代码修复回执；若有缺陷，T-02先约简，T-03只修UI owner。问题落到Blog作者widget/路由时由BLOG-004 T-03接入，前置回执必须明确哪个是待应用闭合路径，以真实非空原生控件/API正控关闭框架项，不要求BLOG-004先完成。完整Blog任务的失败继续由004阻断，不能随框架pass消失。

### 规范增量

| delta_id | action | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/native-vm-app-validation.md（拟新增，本仓） | 无明确可复验原生应用合同 → 实际启动/驱动/身份/事件到达/失败/退出与环境边界 | HTTP或空窗口不能冒充原生用户任务通过 | AC-01..04 |

已有语言侧UI知识保留权威；若实际改动要求修改其现有语义，先明确增量与owner，不复制一份冲突规范。本轮不发布canonical。

## 6. 测试设计

证据政策：原始日志/编译诊断/截图/探针输出放本地 `docs/plans/attachments/`（不提交非md证据）；Plan内保留命令、退出码、版本、关键结果及本地路径+内容hash摘要。不得以日志里的pass字样代替断言。所有测试使用独立数据目录与明确端口；先检查占用，只关闭本次创建的PID，不批量杀同仓或其他用户进程。

| case_id | owner / AC | 输入/实际路径 | 关闭oracle |
|---|---|---|---|
| U-01 | T-01/02/04，AC-01/02 | 真Blog源码诊断＋T-02最小原生正控，`auto run -r vm`，控制应用以 `pac.at api: "rust"` 选择真实Rust API；可交互桌面/固定工具版本 | 窗口与驱动关联到本次PID/应用会话，正控native root非空、真实控件可操作；Blog接缝缺项单列004，不仅后端健康 |
| U-02 | T-02/04，AC-02 | 经实际协议在原生正控中选择/点击/输入中文、换行，触发真实API并重开 | 事件到达真实widget/store，真实后端/文件与输入对应；仅关闭框架输入链，不能直接POST后截图冒充，也不作为Blog作者任务已过 |
| U-03 | T-02/04，AC-03 | 不可用独立MCP端口/目标已退出/后端不可达，附合法同输入控制 | 有界非零失败、错误归属准确；不能得到0场景pass，合法控制可重复成功 |
| U-04 | T-04/05，AC-04 | 正常关闭本次拥有进程，重启同应用与隔离配置 | 会话与驱动释放、不误连前实例或残留PID；重开可再次交互。若应用声明数据持久化，数据留存由BLOG-004以Blog自身存储合同验证，AutoUI控制夹具不冒充持久化测试 |

永久驱动/fixture由T-02根据真实协议建立，优先复用现有AutoUI验证入口，不先编造方法/硬编码vnode编号。实现阶段只跑受影响定向检查；冻结代码与最终review后由merge持本仓适用t/tu/ti/tn门，具体触面和原生人工门结果必须记录，不套用语言仓档位。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 失败位置/真实owner、桌面条件、CLI/UI依赖与驱动协议明确，历史环境结论有新鲜调查；必需条件欠缺时明确阻塞 |
| AC-02 | work | 实际Blog诊断的归属明确，非空原生正控窗口可识别并经实际输入触发真实API；U-01/02事件与后端证据相连，所有UI owner阻断闭合；完整Blog任务仍由004验证 |
| AC-03 | work | U-03故障控制有界非零且正控通过；不能以HTTP健康/单截图/mock/跳过替代原生交互 |
| AC-04 | work | U-04退出/重启与进程/驱动身份闭环，形成Blog可复用的精确版本/命令/操作回执；仅解除UI前置，不声称Rust已通过 |
| AC-05 | review | 冻结实现/依赖与Spec增量，复核真实事件链和负控，记录上下文限制，无未关闭必需环境门 |
| AC-06 | merge | 适用最终门通过后交付UI、Spec/ledger归属与归档/guard清理完成，BLOG-004可消费精确版本；不等待Blog最终Rust双端通过 |

## 8. 执行步骤

- [x] T-01（owner_stage=work；依赖无；AC-01）：在唯一框架树调查实际启动/驱动路径，产出native-path-decision.md；明确环境缺项与owner。[✅ 已完成] 固定AutoUI CLI/tool `auto 0.1.0+v0.0.20-118-ge6cc06f` / `auto-ui-tool 0.1.0`、AutoUI `a226962eae3caf319ebe3e1a2b91ed380eab40cb`、AutoLang `09bb414875d4e15050ecbbaf3959b1d72976d857`，读取实际MCP驱动并将Blog失败归到后端API生成/构建边界；详见本地附件 `../attachments/003/native-path-decision.md`。
- [x] T-02（owner_stage=work；依赖T-01；AC-01..03）：约简原生输入/生命周期问题，按真实协议建立可重复驱动/正反控，记录永久入口命令。[✅ 已完成] 增加被跟踪的 `crates/auto-ui-tools/tests/fixtures/plan003-native-vm-control/`，复用AutoLang `AutoUiMcpClient` 真实MCP工具；运行命令为 `auto run -r vm -B <api-port> -F <front-port>`，`pac.at api: "rust"` 生成真实Rust API。
- [x] T-03（owner_stage=work；依赖T-02；AC-02/03）：只修UI owner确认缺陷；不重现时以有效正控/版本解释关闭，无需空补丁；Man/app问题分别登记消费者待办，禁止越owner。[✅ 已完成] 控制夹具的窗口、输入、后端调用和生命周期都通过；未复现AutoUI owner缺陷，没有UI运行时代码补丁。Blog生成Rust后端仍有37项编译错误，在窗口初始化前退出，留给BLOG-004消费闭合。
- [x] T-04（owner_stage=work；依赖T-03；AC-02..04）：U-01..04实际窗口、事件、失败与退出重开回放；只清理本次进程与隔离数据。[✅ 已完成] 控制夹具连续关闭/重开、两次本机窗口与真实API读回通过；负控MCP拒绝在2.052秒内返回，API/MCP端口关闭后可立即重绑；Blog的持久化仍由BLOG-004验证。
- [x] T-05（owner_stage=work；依赖T-04；AC-04）：形成Blog可消费回执和拟SD-01，提交实现/测试/摘要，转execution_done；Rust组合不在本前置的通过声明。[✅ 已完成] 提交本地回执、SD-01规范草案/索引及可复用夹具说明；BLOG-004未通过Rust后端/持久化验收，不在此计划宣称通过。
- [x] T-06（owner_stage=review；依赖T-05；AC-05）：同一框架树暂停实现，最终复核真实原生输入/退出/负控与Spec，不通过则回work。[✅ final repair re-review pass；同一执行上下文的限制已记录]
- [ ] T-07（owner_stage=merge；依赖T-06；AC-06）：完成适用本仓门、交付/规范/索引/归档/guard清理，把精确UI版本交BLOG-004接入。[AutoUI交付/ledger/归档/cleanup均已完成；由于AutoLang批回归门到期且AutoLang主检出存在既有WIP，完整merge closeout等待安全执行回归。]

## 9. 复审记录

stage: new | plan_id: PLAN-003 | plan_revision: 1 | outcome: pass（有界原生路径调查就绪；环境/运行能力未验证） | next: work/T-01（另行执行）。7项未勾选，本轮未启动原生窗口或修改UI源码。

## 10. 待澄清事项

无须再次确认原生VM是否必须验收。可交互桌面、实际MCP协议/可用工具、进程退出原因由T-01核实；若当前环境无法提供实际交互，给出具体缺失条件并保持任务未完成，不因是环境问题就放行最终pass。

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r1 | round: 1 | from: preflight | to: work | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-003/auto-ui (not created; main-checkout WIP ownership unresolved) | commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | next: user

当前阻塞：主检出存在未提交的 `docs/design/README.md`、`docs/design/autoui/**`、`docs/plans/README.md`、`.next-id` 和新建的 PLAN-007；本轮无法确认其所有者及与本次落地的关系。请告知这些变更是否由你或其他会话持有，以及是否可以原样保留并在PLAN-003落地后再协调合并。所有文件均已保留，未开始PLAN-003实现。

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r1 | round: 2 | from: user | to: user | status: drafting | outcome: waiting | worktree: D:/autostack/.wt/ui-003/auto-ui (not created; main-checkout WIP ownership unresolved) | commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | next: user

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r1 | round: 3 | from: user | to: user | status: drafting | outcome: blocked | worktree: D:/autostack/.wt/ui-003/auto-ui (not created; main-checkout WIP ownership unresolved) | commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | next: user

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-003/auto-ui (branch codex/plan-003-dev; main-checkout docs WIP preserved) | commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | next: review

Work receipt (2026-10-29): stage: work | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | baseline: a226962eae3caf319ebe3e1a2b91ed380eab40cb | AutoLang: 09bb414875d4e15050ecbbaf3959b1d72976d857 | worktree: D:/autostack/.wt/ui-003/auto-ui | scope: native AutoUI prerequisite | evidence: `../attachments/003/execution-receipt.md`, `../attachments/003/native-path-decision.md`; positive replay SHA-256 7569161DF6CFD0232923B52ECAE6F173303A5A9BA006A279D54E8D66FCE01427; negative MCP SHA-256 FE24A7EECB00DDF11AB5FD54CF38EA4FA5916C12FB0326163A2B678C6422C88C; Blog pre-window Rust build failure 37 errors/31 warnings, CLI exit 1, remains BLOG-004-owned. Command/consumer boundary correction is recorded in r2 without changing the goal or repository ownership.

PLAN revision 2 explanation: execution proved `-r vm` selects the native frontend while `--server vm` selects a backend; the control app uses its declared Rust API. U-04's durable-data oracle is application-owned, so BLOG-004 verifies Blog draft survival; PLAN-003 keeps the required AutoUI session close/reopen plus fresh real API event/readback. This aligns the already-approved UI/consumer boundary and does not remove the Blog persistence gate.

Review receipt (2026-10-29): stage: review | review_scope: final | reviewer_identity/limitations: same active Codex context as work; verdict reconstructed from committed source, pinned dependency Specs, raw acceptance logs, screenshots and Plan; no separate reviewer context available, so not represented as independent | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-003/auto-ui | branch: codex/plan-003-dev | reviewed_commit: 83ebb2ad2c14912dd25b73739a69ae3e857cb57f | base_commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | dependency_revisions: AutoLang 09bb414875d4e15050ecbbaf3959b1d72976d857; main default branch advanced independently to 0261ae9dacf786beb6dceb630be6af267f242625 and will be rebased/range-checked in merge | spec_inputs: `docs/specs/runtime/native-vm-app-validation.md` SHA-256 F6EA4068A011A806DF86DD784213CDD81E5BBA3AA98BC35927AAD0049B31FF0C; `docs/specs/README.md` SHA-256 C08E05354CD336E92FEA9911C6144C35FC988F6440567D73503D91E77BBA5952; AutoLang owner Spec at the pinned dependency SHA-256 E728CEB555436529F6A8C6288B100FBD73B827958F28DE61F369264C0334A12A | acceptance_results: AC-01 pass (native path, tool/dependency pins and first-failed-boundary owner); AC-02 pass (visible HWND/current tree, live text/button events and exact generated-Rust API readback; Blog failure is pre-window); AC-03 pass (live MCP snapshot positive plus unused-port `autoui_snapshot` refusal in 2.052s); AC-04 pass (two exit-0 sessions, all six owned PIDs exited each time, API/MCP port rebind and fresh-session event/API readback); AC-05 pass (this final review; same-context limitation stated); AC-06 pending merge | pending_closeout_gates: post-rebase `cargo t`, `cargo tu`, `cargo ti`, `cargo tn`; Spec/ledger publication; fast-forward landing; Plan archive verification; `wt-guard.sh` and owned worktree cleanup; production-artifact freshness observation; batch-regression due check | findings: none against the r2 work-owned criteria; the required Blog Rust backend/persistent-draft acceptance remains open under BLOG-004 and is not waived | evidence: primary durable-path replay `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-acceptance-fixture.log` SHA-256 7569161DF6CFD0232923B52ECAE6F173303A5A9BA006A279D54E8D66FCE01427; U-03 `.../native-control-negative.log` SHA-256 FE24A7EECB00DDF11AB5FD54CF38EA4FA5916C12FB0326163B2A678C6422C88C; Blog run summary `.../blog-native-acceptance-isolated.log` SHA-256 0DE7F9625BCA4833D450C370621A4E5A2604703901EFD16B01314D46431BD70D and compiler log `.../blog-native-20261029-201821-first.log` SHA-256 0CF38E0CB057ADEAD4A88457994E5438C699C3DB71FF7E476DBF20115C0C18D1; frozen Spec delta is in reviewed commit | next: merge.

Receipt correction (2026-10-29): the earlier round-4 goal handoff was written before the work commit and names baseline `a226962eae3caf319ebe3e1a2b91ed380eab40cb`; the actual committed work reviewed above is `83ebb2ad2c14912dd25b73739a69ae3e857cb57f`. Its earlier inline negative-log digest also has a transposed character; authoritative digest is `FE24A7EECB00DDF11AB5FD54CF38EA4FA5916C12FB0326163B2A678C6422C88C`, matching the committed Markdown execution receipt. Prior receipts are preserved append-only.

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 5 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-003/auto-ui (branch codex/plan-003-dev) | commit: 83ebb2ad2c14912dd25b73739a69ae3e857cb57f | next: merge

Merge gate diagnosis (2026-10-29): post-review rebase mapped `83ebb2ad2c14912dd25b73739a69ae3e857cb57f` to `bc3993fa537833bce588bcd61402c23d763b4d6c` on default `0261ae9dacf786beb6dceb630be6af267f242625`; `git range-diff` showed the implementation patch equal. A docs-only follow-up `9b14c7a332b2e2b5be836d19cc4484f284137dca` exports the fixture from `git archive` into a unique target-tree copy; it prevents ignored replay outputs from contaminating repository-copying delivery tests. First `cargo t` had 6 delivery failures because the earlier in-place acceptance replay left ignored `rust-workspace/Cargo.toml` under the source fixture; after removing those generated text files, the same gate passed: 2,443 passed, 886 filtered/skipped, exit 0. Evidence: first log SHA-256 `25351E56A5124A6CE9EF05C9FB829BE43BDB4C9975E3E62BC29E16EF0E017630`; clean rerun `D0EB4F59AF0BF3E40BCA3D32A857FAC73B26573E1DC29AE0E1C60976CC47A7A7`. `cargo tu`: 861 passed, 2 ignored, exit 0, log SHA-256 `C978E236D0C009B2AC4B9BF8C4697DC2BE3EE167A55CA817C51551A89EAC4343`. `cargo ti`: 28 passed, 15 skipped, exit 0, log SHA-256 `CB39E9AC65F849787BB9CBCC8EB1FE9AE3EF75C67E31CD92975A08F604C10E50`. `cargo tn` exited 101 before creating either drag window because `native_dnd_e2e.rs` looks for `scripts/tools/native-fixture/Cargo.toml` under the AutoUI checkout; the file is not present there but is tracked under the pinned AutoLang sibling `09bb414875d4e15050ecbbaf3959b1d72976d857`. Both selected native tests failed at fixture build, so no native-drag pass is claimed. Gate log SHA-256 `CCA5DD464D85F7259FBA239AD86479A7BEC55AC103EB5C6CCAFB340105251683`. This is an AutoUI-owned test harness path mismatch exposed by the required gate; work must route fixture-source lookup to the declared AutoLang sibling and keep its build output in the AutoUI target tree, then run affected `tn`, review, and closeout. The Blog consumer blocker remains separate.

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 6 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-003/auto-ui (branch codex/plan-003-dev; rebased candidate 9b14c7a332b2e2b5be836d19cc4484f284137dca) | commit: 9b14c7a332b2e2b5be836d19cc4484f284137dca | next: work.

Work repair receipt (2026-10-29): stage: work | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | base_commit: 9b14c7a332b2e2b5be836d19cc4484f284137dca | code_commit: a2ea08c45c50cb925e394d32621e97a442257fa0 | dependency_revisions: AutoLang 09bb414875d4e15050ecbbaf3959b1d72976d857 | worktree_path: D:/autostack/.wt/ui-003/auto-ui | branch: codex/plan-003-dev | repair: `native_dnd_e2e::fixture_bin` now resolves the tracked fixture from the local root, `AUTO_LANG_ROOT`, or declared sibling; copies its manifest/README/main source under AutoUI `target/native-fixture/source`; builds under AutoUI `target/native-fixture/build`; no source/lockfile/target was written to AutoLang. `rustfmt --edition 2021 --check` and `git diff --check` passed. The affected `cargo tn` rerun on this commit passed both real cursor E2E cases (`t3_drag_in_fixture_to_desktop_drop_target`, `t4_drag_out_desktop_to_fixture`), exit 0; log `D:/autostack/auto-ui/docs/plans/attachments/003/work/merge-cargo-tn-repair.log` SHA-256 `ECC0E624717BDE1C975E9E4289BDEC180135BFC777BC74448EBBAB093EAB002A`. Prior `t/tu/ti` passes remain bound to `9b14c7a`; this repair changes only the `native_dnd_e2e` integration target, which those aliases do not select. Their logs and selection counts are recorded in the merge-gate diagnosis above; final review must verify this unaffected-input claim. `AUTO_LANG_ROOT` used for the rerun was the pinned sibling; its worktree remained clean. | blockers: none in work-owned criteria; review must inspect this repair | next_responsible_role: final review.

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-003/auto-ui (branch codex/plan-003-dev) | commit: a2ea08c45c50cb925e394d32621e97a442257fa0 | next: review

Review re-receipt (2026-10-29): stage: review | review_scope: final repair re-review | reviewer_identity/limitations: same active Codex context as work; reconstructed from the exact repair diff, committed source, pinned AutoLang fixture, current gate selection, and fresh sibling-fallback gate output; not represented as a separate-context review | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-003/auto-ui | branch: codex/plan-003-dev | reviewed_commit: a2ea08c45c50cb925e394d32621e97a442257fa0 | base_commit: 0261ae9dacf786beb6dceb630be6af267f242625 | dependency_revisions: AutoLang 09bb414875d4e15050ecbbaf3959b1d72976d857 (clean) | spec_inputs: unchanged frozen delta `docs/specs/runtime/native-vm-app-validation.md` SHA-256 F6EA4068A011A806DF86DD784213CDD81E5BBA3AA98BC35927AAD0049B31FF0C; `docs/specs/README.md` SHA-256 C08E05354CD336E92FEA9911C6144C35FC988F6440567D73503D91E77BBA5952; AutoLang owner Spec SHA-256 E728CEB555436529F6A8C6288B100FBD73B827958F28DE61F369264C0334A12A | acceptance_results: AC-01 pass (prior pinned path/owner evidence unchanged); AC-02 pass (prior native-window event/API evidence unchanged); AC-03 pass (prior paired live/unused-MCP evidence unchanged); AC-04 pass (prior close/reopen and cleanup evidence unchanged); AC-05 pass (repair now locates the tracked AutoLang fixture under the sibling even when `AUTO_LANG_ROOT` is invalid, copies only its manifest/README/main source to AutoUI target, builds there, and runs both drag directions); AC-06 pending merge bookkeeping/archive/cleanup | pending_closeout_gates: ledger publication and readback; fast-forward landing; archive/link/count verification and docs commit; fresh `wt-guard.sh` and owned worktree cleanup; production-artifact freshness record; batch-regression due check | findings: none | evidence: current sibling-fallback `cargo tn` log `D:/autostack/auto-ui/docs/plans/attachments/003/work/merge-cargo-tn-sibling-fallback.log` SHA-256 D2E3AECF6E9D1FE35DE0AEB9B3E38E2E35E904D46AF177F24104450E0B6229A5, exit 0 (2 passed); pinned-env rerun `.../merge-cargo-tn-repair.log` SHA-256 ECC0E624717BDE1C975E9E4289BDEC180135BFC777BC74448EBBAB093EAB002A; `t/tu/ti` selection definitions and successful logs remain bound to 9b14c7a and exclude this integration test; reviewed delta hashes unchanged | next: merge.

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 8 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-003/auto-ui (branch codex/plan-003-dev) | commit: a2ea08c45c50cb925e394d32621e97a442257fa0 | next: merge

Merge receipt (2026-10-29): stage: merge | checkpoint: prepared | plan_id: PLAN-003 | plan_revision: r2 | outcome: pending | reviewed_commit/delivery_candidate: a2ea08c45c50cb925e394d32621e97a442257fa0; review pass is bound to this commit and pinned AutoLang 09bb414875d4e15050ecbbaf3959b1d72976d857 | default: auto-ui master 0261ae9dacf786beb6dceb630be6af267f242625; candidate is a clean descendant; no additional rebase required | canonical delta: `docs/specs/runtime/native-vm-app-validation.md` SHA-256 F6EA4068A011A806DF86DD784213CDD81E5BBA3AA98BC35927AAD0049B31FF0C plus `docs/specs/README.md` SHA-256 C08E05354CD336E92FEA9911C6144C35FC988F6440567D73503D91E77BBA5952 | ledger projection targets: auto-lang workspace `.autoos/specs.json`, project `auto-lang`, version 137 / snapshot SHA-256 B2EF2E4F669D6505429E20F75B34B3E53FE7CDDE7EF8E0863E153A68CA224FC7; designs `P003-NVM-APP`, tests `P003-NVM-T`, reviews `P003-R2`, reports `P003-M` (all IDs absent at preparation); no AutoUI-local `.autoos` exists, and prior AutoUI plans record their ledger deposits in the AutoLang workspace. No in-session Spec writer is available; the apparent localhost responder is an echo server returning `{}` and the Spec store endpoint is not live, so ledger publication will use guarded offline projection after landing | next: land, then validate the guarded projection.

Merge receipt (2026-10-29): stage: merge | checkpoint: landed | plan_id: PLAN-003 | plan_revision: r2 | outcome: pending | default branch: `auto-ui/master` fast-forwarded 0261ae9dacf786beb6dceb630be6af267f242625 → a2ea08c45c50cb925e394d32621e97a442257fa0 via `git merge --ff-only codex/plan-003-dev`; resulting tip equals the reviewed delivery candidate, no merge commit, and all 12 expected code/fixture/Spec paths are tracked. The reviewed rebase patch-equivalence and merge-owned `cargo t`/`tu`/`ti`/`tn` evidence remain bound to the same final commit as detailed above; no source changes occurred during landing. Existing main-checkout docs WIP is preserved.

Merge receipt (2026-10-29): stage: merge | checkpoint: ledger_refreshed | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | target: `D:/autostack/auto-lang/.autoos/specs.json` (the existing AutoUI-owner workspace ledger; AutoUI itself has no `.autoos`, and no store writer is reachable; channel: guarded offline projection) | version 137→138; six section IDs validated, all existing ledger fields/items preserved, four unique item IDs read back with complete `SpecItem::new` fields and nonempty canonical `file`/`tags`: designs `P003-NVM-APP`→`auto-ui/docs/specs/runtime/native-vm-app-validation.md`; tests `P003-NVM-T`→`auto-ui/crates/auto-ui-tools/tests/fixtures/plan003-native-vm-control/README.md`; reviews `P003-R2` and reports `P003-M`→`auto-ui/docs/plans/archived/003-blog-native-vm-run-and-acceptance.md`. Counts: designs 373→374, tests 216→217, reviews 363→364, reports 266→267. Source hashes: canonical Spec F6EA4068A011A806DF86DD784213CDD81E5BBA3AA98BC35927AAD0049B31FF0C; Spec index C08E05354CD336E92FEA9911C6144C35FC988F6440567D73503D91E77BBA5952; test fixture README 54A3A3D70BEBEC3A5F308F73E934B9B33298DAB03912A2B599EBF5B6DC9735A6. Ledger readback SHA-256 17D286F1779A9598A207DC2DA348DC3AD975597A06B18BEC184A424333E7822F; byte-exact pre-projection backup `D:/autostack/plan-artifacts/plan-003/ledger-specs-backup-v137.json` SHA-256 B2EF2E4F669D6505429E20F75B34B3E53FE7CDDE7EF8E0863E153A68CA224FC7; projection script `D:/autostack/plan-artifacts/plan-003/project_ledger.py` SHA-256 344E1A1332D6A0E588A6426CA1024D9B5C54E3A9222EA6C0842391CA1D3D1448.

Merge receipt correction (2026-10-29): the preceding ledger row records a valid readback from a provisional projection, but the file is tracked by AutoLang and that main-checkout write was reverted byte-for-byte to v137 before landing. The provisional hash is not the final ledger state. The earlier prepared/landed rows were initially recorded as pending; their operations are now verified pass by the current main tip and archive commit below. The final tracked-ledger worktree commit and readback are recorded in the next ledger receipt.

Merge receipt (2026-10-29): stage: merge | checkpoint: ledger_refreshed | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | authoritative channel: guarded offline projection in dedicated AutoLang worktree `D:/autostack/.wt/lang-003/auto-lang`, followed by `git merge --ff-only` into `auto-lang/master`; pinned AutoLang dependency tree `D:/autostack/.wt/ui-003/auto-lang` stayed clean and read-only. Ledger v137→v138; designs 373→374, tests 216→217, reviews 363→364, reports 266→267; the six required section IDs, unique IDs, four item records, `SpecItem::new` fields, `file`/`tags`, and unchanged pre-existing parsed data were verified. AutoLang ledger commit `2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95`; `auto-lang/master` now equals this commit; final main-workspace ledger SHA-256 C520695D9E090458F8D80B2407B5A431DDA95EF446A5FBD4CB79ADBFBF009D80 (Git blob d995bf894d9af82249098d246a0963511d21243d). Items: designs `P003-NVM-APP`→`auto-ui/docs/specs/runtime/native-vm-app-validation.md`; tests `P003-NVM-T`→`auto-ui/crates/auto-ui-tools/tests/fixtures/plan003-native-vm-control/README.md`; reviews `P003-R2` and reports `P003-M`→`auto-ui/docs/plans/archived/003-blog-native-vm-run-and-acceptance.md`. Canonical Spec, index, and fixture README hashes remain F6EA4068A011A806DF86DD784213CDD81E5BBA3AA98BC35927AAD0049B31FF0C, C08E05354CD336E92FEA9911C6144C35FC988F6440567D73503D91E77BBA5952, and 54A3A3D70BEBEC3A5F308F73E934B9B33298DAB03912A2B599EBF5B6DC9735A6. Worktree-source byte backup `D:/autostack/plan-artifacts/plan-003/ledger-specs-backup-v137-autolang-worktree.json` SHA-256 3445BD068672F4FB101824C99B0DB44EBBC7CCFD6903F030F6B932D63CA21212; the different root-checkout backup hash B2EF2E4F... is only its CRLF-normalized equivalent and was restored exactly after the provisional write. Projection script SHA-256 5CC735EF308BF686D1E0CDD920222E7D6435DC6471ED95F1B319E54EC35EA44E. AutoLang main's 105 pre-existing worktree-status entries matched byte-for-byte before and after its one-file fast-forward.

Merge receipt (2026-10-29): stage: merge | checkpoint: archived | plan_id: PLAN-003 | plan_revision: r2 | outcome: pass | completion_kind: delivered | archive: `docs/plans/archived/003-blog-native-vm-run-and-acceptance.md`; archive commit `43d9b3a9ed3d60124d3b57fc9b4babbc7526a0a5` committed the archived Plan and its README navigation row to `auto-ui/master`. The archived frontmatter is `status: archived`, and links to the two tracked attachment summaries resolve under `docs/plans/attachments/003/`; the Plan README archive link resolves; the active Plan path is absent; T-01..T-06 are checked and T-07 remains pending only for cleanup.

Merge receipt (2026-10-29): stage: merge | checkpoint: cleaned | checkpoint_outcome: pass | plan_id: PLAN-003 | plan_revision: r2 | UI worktree `D:/autostack/.wt/ui-003/auto-ui` was clean, fully landed, and `wt-guard.sh` reported clean (exit 0) immediately before removal; branch `codex/plan-003-dev` was deleted after verifying it is an ancestor of `auto-ui/master`. Its Cargo `target` cache (`CACHEDIR.TAG` present) was removed with the worktree; 427 `.pdb` files totaling 11,604,127,744 bytes were reclaimed. All 331 linked `.exe` outputs (1,360,540,160 bytes) were copied with per-file SHA-256 verification to `D:/autostack/plan-artifacts/plan-003/worktree-exe/`; manifest `D:/autostack/plan-artifacts/plan-003/worktree-exe-manifest.tsv` SHA-256 DF1DEE80186B380A585BA880FD26BFB650ED527F91BAFD6ED53009BF9571B843. The artifact folder contains no `.pdb` and no Cargo cache (`CACHEDIR.TAG` absent); its `target` subfolder only preserves the original relative layout for the `.exe` files. The separate AutoLang ledger worktree `D:/autostack/.wt/lang-003/auto-lang` was clean and guarded (exit 0), then removed after commit `2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95` landed; branch `codex/plan-003-ledger` was deleted and its empty group removed. `D:/autostack/.wt/ui-003` is retained because the read-only sibling worktrees `auto-lang` and `auto-down` remain there. No other worktree or sibling was removed.

Production-artifact observation (2026-10-29): the reviewed AutoUI diff changes only an integration-test helper, a tracked acceptance fixture, and documentation; AutoLang source was not changed by the ledger commit. AutoUI `target/release` and `gen/front/vue/dist` were absent on the main checkout at inspection. No release binary, dependency executable, web bundle, or running process consumes the test/fixture/Spec-only changes, so no production rebuild or restart was needed.

Batch-regression due check (2026-10-29): AutoLang's `docs/plans/.last-batch-regression.json` is readable and covers through plan 880 (`covered_commit` 03f851505, timestamp 2026-10-28T18:55:00Z). AutoLang master contains PLAN-885 after that commit; 885 is divisible by five, so the AutoLang `tf`/`tt`/`tb` batch gate is due. It was not started: the required AutoLang main checkout has 105 pre-existing modified/untracked docs/design, plans, attachment, and report entries (about 2.13 MB), and one live Blog backend `cargo run` parent/child pair (PIDs 59648/9868) is active. The git status of those 105 AutoLang entries was captured before/after the ledger fast-forward and is byte-for-byte identical; no user WIP or process was changed. The `auto-plan-regress` precondition requires a clean main checkout and one gate instance. T-07 remains open until the due gate can run safely.

Goal handoff (2026-10-29): stage: goal | plan_id: PLAN-003 | plan_revision: r2 | round: 9 | from: merge | to: user | status: archived | outcome: waiting | delivery: AutoUI master `43d9b3a9ed3d60124d3b57fc9b4babbc7526a0a5`; AutoLang ledger master `2fdee4b0c817aa3e2effb331f0f8b57b3ef56d95`; archive and cleanup verified | pending: decide whether the AutoLang docs WIP may be temporarily stashed/restored to satisfy the due batch-regression clean-main precondition; T-07 remains open pending `tf`/`tt`/`tb`.
