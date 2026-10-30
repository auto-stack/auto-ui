---
plan_id: PLAN-008
status: archived
feature_name: PLAN-004交付身份清单修复
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30
plan_revision: 1
current_step: 2
total_steps: 2
depends_on: [auto-ui/PLAN-004]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-008：PLAN-004交付身份清单修复

## 0. 变更摘要

修复 PLAN-004 身份生成器误收窗口探针按 PID 写入的临时 `.agscene` 文件，重生成清单，并验证真实源文件改动、缺失和未列文件仍被拒绝。文件和目录遍历一致排除易失产物/构建目录。仅修复身份闭包；不改产品功能、预算、必需文件或预期产品源哈希。

## 1. 目标

让 PLAN-004 的身份清单只绑定可复现的交付源和稳定证据，fresh consumer 的 `python tools/verify_source.py` 通过，身份负控继续有效。

非目标：修改 PLAN-004 的功能验收或公开API；删除身份门；重定向预期源哈希；改 ag-examples/PLAN-002 实现。

## 2. 架构方案

在 `D:/autostack/.wt/ui-008/auto-ui`、分支 `codex/plan-008-dev` 修复 auto-ui 的交付身份生成/验证。主检出只保存此计划和进度；下游 ag-examples consumer 只用于最终复核，不修改其文件。

## 3. 技术栈

Python身份生成/验证脚本、SHA-256清单；沿用现有PLAN-004检查，不新增运行时依赖。

## 4. 需求分析与背景调查

2026-10-29 用户授权为解除 auto-ui/PLAN-004 与 ag-examples/PLAN-002 的阻塞，做一次有界交付身份修复并复核。失败证据：`D:/autostack/ag-examples/docs/plans/attachments/002/work/t01-consumer-blocker.md`；消费者错误为缺失 `research/authoring-native-7624.agscene`，期望哈希 `18ef9a0e…`。auto-ui 源码 `experimental/autographics-auto-driver/src/bin/autographics-authoring.rs` 在探针启动时写 `research/authoring-native-<PID>.agscene`；`gen_plan004_identity.py` 全包递归且没有排除此类动态文件。归档PLAN-004记载交付清单预期1431项，现清单1432项。此修复不影响规格，故Spec delta为空。

## 5. 详细设计

排除规则仅匹配 `autographics-auto-driver/research/authoring-native-<十进制PID>.agscene`；不得排除同目录任意 `.agscene`。生成器与验证器必须采用同一策略，验证器拒绝清单显式 pin 此路径。目录列表按完整相对路径过滤 `target` 与 `.sdk-source` 等已忽略目录的全部后代。fresh consumer须证明清单不含目标动态产物，且改动/删除已列源、加入任意非例外文件均失败。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| — | — | 无 | 无行为或接口规格变化 | 仅修复验证闭包中的易失探针产物分类 | — |

## 6. 测试设计

| V/case | AC/owner_stage | 输入与入口 | 可观察判据 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/work | 生成身份清单；两种脚本 | PID动态探针文件及构建目录后代不入清单，正常/稳定`.agscene`仍受哈希与闭包校验 | 不验证窗口探针运行本身 |
| V-02 | AC-02/work | verify_plan004 `--controls` | 修改/删除/任意未列文件/错误pin均非0；恢复后通过 | 不替代fresh consumer |
| V-03 | AC-03/work | ag-examples PLAN-002 consumer的`python tools/verify_source.py` | exit 0且报告PLAN-004身份通过 | 不运行PLAN-002实现/全量回归 |

实施与复核在当前会话顺序执行；复核会从提交/diff和原始入口重建结果，不声称有独立会话。通过后由 PLAN-002 owner 继续消费。

## 7. 验收标准

| ID | owner_stage | 结果与证据 |
|---|---|---|
| AC-01 | work | 生成器/验证器精确排除 PID 临时场景与被忽略目录后代；任意其它 `.agscene`仍受闭包校验。 |
| AC-02 | work | 身份清单稳定为1431项；`verify_plan004.py --controls`通过，源码/删除/任意未列文件/错误pin拒绝与探针正控保留。 |
| AC-03 | work | fresh consumer执行`python tools/verify_source.py` exit 0；PLAN-004身份、681个SDK文件、479个依赖包/29条本地路径边通过。 |
| AC-04 | review | 检查提交diff、例外匹配窄度、AC-01..03证据；无finding并诚实记录同会话复核限制，再交还PLAN-002 owner。 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户授权 | 精确排除临时探针场景及构建目录后代，重生成清单，执行身份与负控、fresh consumer验证 | AC-01..03 | [x] |
| T-02 | review | T-01 | 顺序复核变更与当前复核证据；记录限制和交接 | AC-04 | [x] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-008 | plan_revision: 1 | outcome: pass | next: work (T-01)`。此计划由用户授权的PLAN-004有界交付身份修复派生；保持在executing直至复核交还consumer owner。

2026-10-29 work T-01：`stage: work | plan_id: auto-ui/PLAN-008 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-008/auto-ui | branch: codex/plan-008-dev | base_commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | code_commit: 0261ae9dacf786beb6dceb630be6af267f242625 | dependency_revisions: PLAN-004 delivery e0fce138; PLAN-002 pin 0ee5ad07; SDK pin 7b987ca8 | task_ids: T-01 | blockers: none | next_responsible_role: review (T-02)`。`python tools/gen_plan004_identity.py`生成1431项；`python tools/verify_plan004.py --controls` exit 0（修改/删除/任意未列/错误pin拒绝，PID探针正控通过）；`python tools/verify_source.py` exit 0（681 SDK files、479 packages、29 local path edges、forbidden=0）。fresh consumer：`D:/autostack/plan-artifacts/ag-examples/plan-002/consumer-plan008-r2`，`verify_source.py` exit 0；身份文件SHA-256 `f59502b49d40338e3b2c8b24f332afb449bca4ef06dcc0b60afab6be7d9cfdeb`，与工作树副本一致。

2026-10-29 final review：`stage: review | review_scope: final | reviewer_identity/limitations: 当前Codex会话顺序复核；未使用独立会话，按提交、清单、负控和fresh consumer原始结果重建判定 | plan_id: auto-ui/PLAN-008 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-008/auto-ui | branch: codex/plan-008-dev | reviewed_commit: 0261ae9dacf786beb6dceb630be6af267f242625 | base_commit: a226962eae3caf319ebe3e1a2b91ed380eab40cb | dependency_revisions: PLAN-004=e0fce138; PLAN-002 pin=0ee5ad07; SDK=7b987ca8 | spec_inputs: none; no runtime/API behavior change | acceptance_results: AC-01 pass（exact PID path regex与非例外未列文件负控）；AC-02 pass（1431项、verify_plan004 --controls）；AC-03 pass（fresh consumer verify_source exit 0，479 packages/29 local edges）；AC-04 pass（差异无finding） | pending_closeout_gates: reviewed commit快进至master，并在原auto-ui source路径重跑verify_source.py | findings: none | evidence: T-01收据及同一commit上的复跑；consumer identity SHA-256 f59502b49d40338e3b2c8b24f332afb449bca4ef06dcc0b60afab6be7d9cfdeb | next: merge reviewed fix into master and rerun the original consumer gate`。

2026-10-29 delivery：审阅提交 `0261ae9dacf786beb6dceb630be6af267f242625` 已快进至 auto-ui/master；原交付路径 `D:/autostack/auto-ui/experimental/autographics-auto-driver` 的 `python tools/verify_source.py` exit 0（身份1431、SDK681、479 packages、29 path edges、forbidden=0）。本地consumer副本 `D:/autostack/plan-artifacts/ag-examples/plan-002/consumer-plan008-r2` 同样exit 0且身份文件SHA-256相同。PLAN-002的身份前置已满足，可由其owner继续T-01/T-02；本Plan未更改ag-examples文件。

2026-10-30 merge r1（PLAN-008:r1，delivered）：`stage: merge | plan_id: auto-ui/PLAN-008 | plan_revision: 1 | completion_kind: delivered | outcome: pass | checked: prepared, landed, ledger_refreshed, archived | delivery_commit: 0261ae9dacf786beb6dceb630be6af267f242625 | knowledge_commit: auto-lang 130caf7ff | pending: cleaned`。

- **prepared**：reviewed 基线 `0261ae9d`（final pass；其 pending_closeout_gates 两项——快进 master 与原路径 verify_source 复跑——已由 2026-10-29 delivery 收据完成并核实）。冻结 Spec delta 为空（§5 规范增量表：无行为/接口规格变化，review AC-04 绑定），无 canonical 文档目标；ff-only 落地使交付提交=复核提交，无文档后继需要重审。
- **landed**：`git merge-base --is-ancestor 0261ae9d master` 通过（2026-10-29 delivery 收据记录的 ff-only 落地、无 merge commit 与实际仓库一致）；2026-10-30 在主检出当前尖峰复跑 `python tools/verify_source.py`，REAL_EXIT=0（身份1431、SDK681、479包、29本地路径边、forbidden=0），main 已知良好。
- **ledger_refreshed**：workspace auto-lang，`.autoos/specs.json` v148→v149，通道 **guarded offline projection（无可达 store writer：8080 无监听，19999 `/api/specs` 返回空 `{}` 非 store 进程，同 PLAN-015 先例）**。字节级备份 `D:/autostack/plan-artifacts/auto-ui/plan-008/ledger-specs-backup-v148.json` SHA256 `be62d88190e5bf38615bd5aa1179f93b4db890f791fed05aecf3128140a79ddf`（与 015 收据 v148 尾哈希一致，链连续）。因该文件非均匀 pretty-print（历史投影为文本级插入），采用同款**文本级限定插入**：新增 `UI008-R1`（reviews 375→376）与 `UI008-MERGE`（reports 277→278），两条均带完整 `file`/`tags`；守卫全过：未触及 item 逐条保持、外科范围可逆证明（新文本-插入-版本号还原==备份）、六段 schema、1477 唯一 id、版本恰好+1、读回一致。投影脚本保全于 `D:/autostack/plan-artifacts/auto-ui/plan-008/project_ledger.py`。知识分支 `plan-008-ledger` 提交 `69147a6b6`，rebase 到前进后的 master（`397d2bf37`→`32a790df2`，期间 ledger 零改动）为 `130caf7ff`（旧→新映射，单文件 52+/3- 补丁等价），主检出 `--ff-only` 落地后读回 SHA256 `f55880886aad6d895aac03a2f66bee643ad95bacdd048eeaeae47e68f99eaa6b` 与写出一致。
- **archived**：本文件由 `docs/plans/` 移入 `docs/plans/archived/` 且 `status: archived`；全文无相对链接，无需改写；`docs/plans/README.md` 008 行改指归档路径与 delivered 状态（该 README 同时有他会话未提交 WIP：007/014 行与页脚改写——本次仅以 HEAD 版本+本行归档态提交，随后把其 WIP（含归档后的 008 行）还原到工作区，未吞并）。
- **pending**：`cleaned`（wt-guard 后移除 ui-008 树/分支/空组、组内与 plan-artifacts 中间产物回收）、landing 产物观察、批回归 due 检查（均在清理收据记录）。

2026-10-30 merge cleaned（PLAN-008:r1，最终收口）：`stage: merge | plan_id: auto-ui/PLAN-008 | plan_revision: 1 | completion_kind: delivered | outcome: pass | checked: prepared, landed, ledger_refreshed, archived, cleaned | delivery_commit: 0261ae9dacf786beb6dceb630be6af267f242625 | knowledge_commit: auto-lang 130caf7ff | archive_commit: auto-ui 1e3c983 | pending_merge_owned_gates: none`。

- **cleaned**：移除前核实 ui-008 树 clean（`git status` 空）且 HEAD `0261ae9` 为 master 祖先；`bash wt-guard.sh D:/autostack/.wt/ui-008/auto-ui` 报 clean（exit 0，无 junction/symlink）；`git worktree remove` 后 `git branch -d codex/plan-008-dev`（was 0261ae9，已合并）；`git worktree list` 与分支表复核 ui-008 零残留。会话中临时建立的知识工作树 `…/ui-008/auto-lang`（分支 plan-008-ledger，was 69147a6b6→rebase 130caf7ff）已在 ff 落地验证后先行移除并删分支。空组目录 `D:/autostack/.wt/ui-008/` 已删（rmdir 仅空目录成功）。组内与 `D:/autostack/plan-artifacts/plan-008/`（目录不存在）经 find 复核均无 cargo target 树/.pdb 需回收（0 命中）。保留恢复材料：`D:/autostack/plan-artifacts/auto-ui/plan-008/`（v148 ledger 字节备份、project_ledger.py 投影脚本、inspect_boundaries.py、readme-wip-backup.md）。
- **landing/产物观察**：交付提交仅 3 个文件且全部位于 `experimental/autographics-auto-driver/`（`research/plan004-identity.json` 重生成 + `tools/gen_plan004_identity.py` + `tools/verify_plan004.py`，`git show --stat 0261ae9` 核实）；根 crate、后端/daemon 与生成 web bundle 均不消费该实验包工具，无生产产物需重建/重启，无 stale-artifact 项。
- **批回归 due 检查**：auto-lang `docs/plans/.last-batch-regression.json` 覆盖 887/2026-10-29T15:53:56Z，本收据时点约 11h（<48h）；auto-lang id 空间 887 之后无 id%5==0 落地（下一个 890 仍未落地，仅 WIP 树）。该步骤按 merge 技能定义是 auto-lang 计划的门，auto-ui/PLAN-008（id 8）不触发。如实记录观察项：ui/PLAN-015（id%5==0）于 2026-10-30T02:23:39Z 落地在收据之后，按 004 先例该触发的处置归属 auto-lang 后续循环（890 落地时）与 015 自身收尾（其合并时已在冻结基线跑完统一门：cargo t 2450/tu 862/ti 28/tn 2 全 exit 0，native 22/22）。

## 10. 待澄清事项

无；不修改下游仓库，不更改必需源文件和预期产品哈希。

