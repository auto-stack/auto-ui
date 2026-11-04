---
plan_id: PLAN-042
status: archived
feature_name: 多文件Reload与状态Schema及Effect生命周期有界核验
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/reload-readiness-and-lifecycle.md]
touched_goals: []
affects: [dynamic-reload, state-migration, multi-file-inputs, effect-owner-readiness]
depends_on: [auto-ui/PLAN-036]
---

# [auto-ui/PLAN-042] 多文件Reload与状态Schema及Effect生命周期有界核验

## 0. 变更摘要

为完整Reload实施建立可验证合同：核真实单源/多文件import-store-child依赖、字段schema迁移、旧状态与effect/Tick/订阅owner、候选失败保旧及后续合法恢复。**本包是有界研究/验证，不宣称完整HMR已实现**，不通过删除多文件guard冒能力。

drafting r1、0/8。2026-11-04用户授权UI与语言两仓后继规划；本轮仅静态调研/立约和文档刷新，暂不实施，无探针/构建测试/GUI/生产代码或其他会话动作。

## 1. 目标

- G1：逐实际入口列出已支持/保守跳过/明确需重启/具名缺口，不混同Vue/Vite更新、Node hot_reload或DynamicComponent能力。
- G2：为保持/迁移/局部重建/拒绝确定schema/行为版本和依赖输入合同，用真实正负例证明边界。
- G3：验证候选准备/校验/迁移/发布、effect撤销与最后owner、失败保旧/再编辑恢复整链的可行机制和成本边界。
- G4：交可直接立后继实施包的接口/职责/反例/预算与owner，不改原支持承诺、不把研究完成算HMR完成。

非目标：本轮实施通用HMR、新Feature语法/Service/持久化框架、迁移全部产品或Demo、改变语言SG927–929源码协议、无界性能压力/全机回归。

## 2. 架构方案

源/依赖版本→完整candidate bundle（root/imports/store/child/handlers/schema/effects）→验证/迁移决定→一次发布或具名拒绝保旧。owner身份、状态schema与effect资源分别治理，不把每次替换都变为重新启动App/丢弃用户数据。具体机制是研究候选，不写成已存在API。

初期可在现实现实证，不等041整项；消费041新增实例/schema/生命周期端口的验证腿只接受其正式交付，无WIP。不将041/042当教学/OS主题迁移统一前置。

## 3. 技术栈

UI runtime ui/dynamic.rs、ui/state_migration.rs、ui/hot_reload.rs、interpreter/bridge现路径，compiler Aura字段/导入及生成信息，现MCP/Playwright/真实文件输入。仅必要探针/永久最小回归fixture，不另写一套解析器/依赖引擎。

## 4. 需求分析与背景调查

已读auto-plan-new/verification-contract、UI索引/主题scope/store路由，036归档、041合同、[框架接续设计](../design/autoui/instance-state-and-reload-stage-one.md)与Design47/38目标。UI观察HEAD 92ee2459a1058ddddc3fc3378ecec6585ffe726c；实际dynamic.rs静态：reload给迁移器old_field_defs空切片，写回错误路径需逐项核；check_file_changed对imports/registry直接None且单源按mtime。它是保守保护，不是多文件HMR。

没有找到覆盖此完整实际路径的canonical，新增SD-01按实测沉淀。041负责实例/receiver，930负责Cargo/交付验证，SG927–929负责语言/ACE源码，042不抢其实现或代为设置新schema。原旧单源测试/迁移器单测与多个“reload”文件必须区分实际入口，不能拼图给能力结论。

## 5. 详细设计

### 5.1 启动与有界输入

实施授权未给。拟唯一树D:/autostack/.wt/ui-042/auto-ui、分支codex/plan-042-reload-readiness；需依赖固定兄弟源，无链接。实现/独立review共用同树。T-01冻结真实各入口、当前支持/依赖、输入规模/时序/资源与准确命令；原用户源码/文档/系统设置不操作。

最小真实fixture：单源带计数/输入、root+import/store/child、多实例/共享owner及一个受控Tick/订阅。永久回归输入可在正式tests，临时修改候选/日志/外部工程在scope；不改业务产品。依赖041新端口的腿可待其正式交付，现路径核验可先做，未跑不能写go。

### 5.2 必须研究的事务与所有权

- 同类型保值/增字段取默认、删除字段处理、同名类型变化、行为版本/依赖变化如何判保持/迁移/局部重建/拒绝，不能只有名字匹配就强写。
- root与导入/store/child改动要绑定同一来源快照，candidate构造和迁移失败前不得替换部分旧bridge/template/handler；改源过程不能给旧产物签新身份。
- 旧字段schema的真实来源与版本可得性、迁移写回错误的处理，以及发布后view/receiver/订阅/缓存一致性。
- effect/Tick/异步回调由owner保留/撤销/重建，过期回调不写新实例；失败保旧可继续操作，恢复后下一次合法编辑成功。不可靠关App清数据来展示绿。
- 检测/依赖失效需区分内容与mtime；相同mtime修改、缺源/无效import、再次修复必须有实际结果。研究可以结论当前不可行，但要有失败定位/前置owner/下一关闭动作。

### 5.3 交付形式

产出入口/支持矩阵、真实命令/输入/状态与effect观察、候选发布/回退协议、预算/所选规模、规范/API提议及后继完整实施包AC。结论分已实证/明确拒绝或需重启/具名阻断/未测，不把Ok(None)或Vite刷新充PASS。若041未交付，依赖其新能力的必需研究腿保持待办；可用当前等价入口实证的部分有证接收，但不能凭候选推定新API。

本包可为研究得出blocked/no-go结论，条件是具体失败证据、owner与可执行后继边界齐；禁止以“复杂”替代核验。不得自动为未知实施包取号/写产品；主题及实例正确性已承诺的真实缺陷交实际owner，不为研究顺手大修框架。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/reload-readiness-and-lifecycle.md | 无实际全链支持表→实测入口/状态/schema/effect/失败边界 | 防HMR能力误报 | AC-01..04 |
| SD-02 | modify | docs/specs/runtime/component-and-store-instances.md（041正式存在后、仅真实差额） | 实例owner出口与reload保持/拒绝责任 | 不重写041 | AC-02/03 |
| SD-03 | add/modify | docs/specs/runtime/reload-readiness-and-lifecycle.md | 记录后继实施分包/原子发布与反例/资源条件，明确未实现 | 有证后继合同 | AC-04 |

## 6. 测试设计

全部未执行，T-01冻结准确入口/fixture，T-02..04使研究探针可运行。原state_migration与dynamic测试保样本/期望/条件，实际UI腿执行时读autoui-verifier；不能只测手造字段Map。work自查与同树独立final review分开，review只验证。

| Case ID | 不变量/输入 | 入口与区分性oracle/边界 | owner_stage |
|---|---|---|---|
| V-01 | AC-01；各reload文件/实际入口和保守多文件guard | 源码调用链+真实App观察，区分single/import/store/registry、Vue/Vite/VM，不以函数名同义当已支持 | work |
| V-02 | AC-02；保持/增删字段/同名类型改、合法/无效修改 | 真实生成/装载/迁移入口，实际字段/可见输入/handler按决定保持/迁移/拒绝，schema来源可核 | work |
| V-03 | AC-02/03；root/import/store/child变化、同mtime/缺源/修复 | 一致来源candidate或明确重启/阻断结果，不能删guard后只reload根、也不能把未检测写成功 | work |
| V-04 | AC-03；候选编译/迁移/写回/发布故障 | 旧业务/bridge/template/effect完整继续，恢复合法后下一次真实修改成功；部分发布/重启丢稿为失败 | work |
| V-05 | AC-03；两实例/共享最后owner、Tick/订阅/迟到回调、关闭 | 真实scope/数据/订阅计数，保留或撤销有owner，失败不重复Tick/越权写新实例；消费041新能力须正式来源 | work |
| V-06 | AC-04；支持/拒绝/blocked矩阵与后继AC | 每结论绑定实际源/命令/观察与资源界，未跑不go、未实施不冒HMR交付，owner/下一步明确 | work |
| V-07 | AC-05；全链证据/支持承诺/遗漏/SD | 同树冻结独立review重建关键正负控制，不能照作者摘要给研究pass | review |
| V-08 | AC-06/07；冻结必要重放、知识/合入/所有输出 | 实际来源读回/归档、结论准确；摘要折Plan后删scope/自有树，资源/guard对账 | merge |

研究/fixture不改语言生产语义，不跑语言Cargo/all、SG/ACE或AG全量。必要UI小接缝/测试入口按实际影响完整职责/原消费者，final后必要代表并集，不强制重跑无关压力；原支持条件与未执行/首红/环境重放分别记，列准备/构建/运行/清理和真实执行数。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 所有实际reload入口/单源多源/guard与真实支持来源清楚，V-01 | work |
| AC-02 | schema/依赖变更的保持/迁移/重建/拒绝有真实正负控制，不超已有支持，V-02/03 | work |
| AC-03 | candidate失败保旧/合法恢复、effect/owner退出和资源边界有全链证据或具体阻点，V-03..05 | work |
| AC-04 | 后继实施分包/接口/AC/反例/预算与041/930/主题边界可直接审阅，未测/未实现准确，V-06 | work |
| AC-05 | 冻结源独立final review研究结论/遗漏/支持/SD可信，V-07 | review |
| AC-06 | 必要冻结重放/真实落地/canonical/路线/归档同源，不冒HMR已实现，V-08 | merge |
| AC-07 | 所有资源/输出对账，摘要折Plan删scope/树产物、guard clean，具名持有齐，V-08 | merge |

## 8. 执行步骤

| ID | 动作/关闭结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 授权后唯一组/现入口/来源/支持/fixture/规模/准确命令与资源冻结 | 授权待给；AC-01..04 | work |
| T-02 | 单文件schema/迁移/失败恢复真实探针与原guard/旧能力基线 | T-01；AC-01/02，V-01/02/04 | work |
| T-03 | 多文件root/import/store/child/内容失效整链探针；已实现/拒绝/阻断准确 | T-02；AC-02/03，V-03/04 | work |
| T-04 | owner/effect/Tick/迟到回调/关闭与失败回退；消费041新端口的腿等其正式交付 | T-03及该腿所需U041；AC-03，V-05 | work |
| T-05 | 全研究矩阵/资源/可行机制/SD与后继完整实施AC冻结；execution_done5/8 | T-04；AC-01..04，V-06 | work |
| T-06 | 同树独立上下文final review，发现回work/失效证据 | T-05；AC-05 | review |
| T-07 | 必要代表冻结重放/实际合入/canonical/路线/归档，保研究与实现区分 | T-06 pass；AC-06 | merge |
| T-08 | 全资源/输出对账、摘要折Plan删scope/自有树，guard后移除分支/组；archived8/8 | T-07；AC-07 | merge |

材料唯一D:/autostack/plan-artifacts/plan-042/auto-ui/，owner/用途/在途登记；临时探针/修改副本/日志截图/外部consumer/显式target在此，普通编译在树，永久最小回归fixture在正式tests。merge命令/退出码/关键状态/源hash/未解关闭条件折Plan后核全部实际输出路径，删scope与树产物；具名持有有owner/理由/关闭条件，未知/活动/他方不盲删，移除前wt-guard clean，不写旧JSON。

## 9. 复审记录

2026-11-04 / stage:work | auto-ui/PLAN-042 r1 | outcome:pass | worktree:D:/autostack/.wt/ui-042/auto-ui | branch:codex/plan-042-reload-readiness | base_commit:7471b7c4 | code_commit:f00e0e11+038f53cf | dependency_revisions:auto-lang f4062cca/auto-web master/auto-down 7c4d5cbb（组内 clone --shared） | task_ids:T-01..T-05 | evidence:cargo test -p auto-ui-runtime --lib plan042_reload_readiness → 12 passed 0 failed（2026-11-04，两次复跑一致）；受影响兄弟 state_migration+reload 家族 16 passed；§11 全量证据 | blockers:无（parser 51GB 病理分配为具名移交，不阻塞） | next_responsible_role:review

2026-11-04 / stage:review | auto-ui/PLAN-042 r1 | outcome:needs_fix | review_commit:038f53cf（冻结核对一致，工作树净） | scope:final | 复跑 plan042_reload_readiness 12/12 绿（review 独立复跑）；引用独立重核：guard dynamic.rs:2327、reload 空 old_field_defs dynamic.rs:2268、泵 host.rs:2822、门控 subscriptions.rs:197、hot_reload.rs parse_content 存根（144-148）、MCP 工具表无组件 reload（仅 action_config_reload）均属实；AC-01..04 证据链齐、oracle 有区分性（同 mtime 负控可判内容检测实现；guard 正控可判删 guard 实现） | findings:F3-1 规范§2"首次检查恒报变化"无限定——正常 set_source_path 登记后基线=当前 mtime，首检报无变化；仅 last_modified=None（登记取不到元数据）时首检恒报变化，须改写；F3-2 规范§1 Vue/Vite 行"属 web 栈自己的 HMR"为本仓无证据的能力断言（examples 无 vue、web-integration.md 无 HMR 记载），须改为范围排除表述（不得以此计 VM/iced 轨能力，是否存在不在本文核验范围） | 建议:work 修两处措辞后免复跑全探针（文档-only 修复，证据不受影响），直接 re-review  pass | next_responsible_role:work

stage: goal | plan_id: PLAN-042 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-042/auto-ui | commit: 038f53cf | next: work

2026-11-04 / stage:work | auto-ui/PLAN-042 r1 | outcome:pass（评审修复） | worktree:D:/autostack/.wt/ui-042/auto-ui | branch:codex/plan-042-reload-readiness | repair_commit:c84cf486 | findings_closed:F3-1（规范§2 首检限定）、F3-2（Vue/Vite 范围排除表述）| 修复性质:文档-only 措辞，证据面不受影响（按 review 建议免复跑探针）| next_responsible_role:review

stage: goal | plan_id: PLAN-042 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-042/auto-ui | commit: c84cf486 | next: review

2026-11-04 / stage:review | auto-ui/PLAN-042 r1 | outcome:pass（final） | review_scope:final | reviewer_identity:goal 会话独立复验（同会话 work/review 分轮，引用与探针均独立重跑重核，非照作者摘要）| reviewed_commit:c84cf486 | base_commit:7471b7c4 | dependency_revisions:auto-lang f4062cca/auto-web master/auto-down 7c4d5cbb | 修复闭合:F3-1/F3-2 已核——diff 仅两行措辞（git show c84cf486），F3-1 与 dynamic.rs:2163-2168 登记语义一致、F3-2 与本仓无 Vue/Vite 证据事实一致；树净、HEAD=冻结核 | acceptance_results:AC-01..AC-04 pass（§11 证据 + 12/12 探针复跑）；AC-05 pass（本轮 final review 重建关键控制）；AC-06/07 merge-owned 挂起 | pending_closeout_gates:merge 统一回归门（auto-ui-runtime 受影响面 union）+ 合入/归档/清理 | evidence:§11 全量；next:merge

stage: goal | plan_id: PLAN-042 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-042/auto-ui | commit: c84cf486 | next: merge

2026-11-04 / stage:merge | auto-ui/PLAN-042:r1 | prepared：reviewed=c84cf486（r1 pass 绑定）；master 期间前进至 cfe04ce1（rebase 必要）；规范 diff=SD-01 新增 reload-readiness-and-lifecycle.md（038f53cf/00058419 文档-only 后裔）| landed：master 前进无法 ff 原链 → 按线性规程 rebase（3/3 无冲突；range-diff 全等：f00e0e11→f0c1bfca、038f53cf→976a5f34、c84cf486→00058419）→ 主检出 `git merge --ff-only` 落地，master tip=00058419ca37d195642f8aee5d7fe5726b30fd60，无 merge commit；rebase 后 affected 复验：plan042 探针 12/12 + state_migration/reload 家族 16/16 全绿 | 统一回归门（merge-owned）：auto-ui-runtime 全 lib 三跑——042 分支基线 2356p/24f 与 2352p/28f、master 基线（master-probe 树）2341p/27f；失败集为环境敏感既有红池（menubar/desktop_protocol/terminal/scroll/session/shell/plan4xx-7xx 等，轮次间漂移），**与 042 无关**（零 reload/state_migration/dynamic 族失败），具名移交批量回归 | canonical_verified：docs/specs/runtime/reload-readiness-and-lifecycle.md 在落地提交（blob eb870095bc…，sha256 125c1b08d4047211，115 行）；SD-02 差额 0 已记录；无 ledger/index 写 | next:archive

stage: goal | plan_id: PLAN-042 | plan_revision: r1 | round: 5 | from: merge | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-042/auto-ui | commit: 00058419 | next: merge(archive+cleanup)

2026-11-04 / stage:merge | auto-ui/PLAN-042:r1 | archived：Plan 移 docs/plans/archived/042-reload-schema-and-effect-readiness.md，status: archived 8/8，归档提交入 master（未跟踪 docs 已补提交，PLAN-104 合规）；归档态核：任务计数 8/8、交付/复审/归档效果与 §11 证据一致、链接自父目录可解 | cleaned：wt-guard 双树 clean（auto-ui/master-probe，exit 0）→ worktree remove×2、分支 codex/plan-042-reload-readiness 删除（was 00058419）、组 .wt/ui-042 移除（含 auto-lang/auto-web/auto-down 兄弟 clone）、plan-artifacts/plan-042 整 scope 删除（证据已折本文 §11/§9）、registry policy.json 刷新（deleted-at-merge 记录 + parser 病理分配具名 hold：owner auto-lang parser）；主检出 042 范围净 | 批量回归到期核：auto-lang docs/plans/.last-batch-regression.json 在案（last_covered 925 @2026-11-03T14:11Z，<48h；042 不整除 5）→ 不到期，不触发 | 落地≠部署观察：本交付仅测试+规范文档，无生产代码面，release binary/web bundle 不受影响、无需重建 | completion_kind: delivered | outcome: pass

stage: goal | plan_id: PLAN-042 | plan_revision: r1 | round: 6 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 00058419 | next: done

2026-11-04 / stage:new / auto-ui/PLAN-042 r1 / outcome:blocked（仅规划，实施授权待给） / 已核旧字段schema空与多文件guard，避免仅删guard或拼接不同reload入口承诺HMR。定义有界研究正负控制/后继出口，全部未执行。next：授权后T-01；新实例相关腿接041正式结果，不把所有研究提前锁死等待。

静态预检pass（2026-11-04）：0–10章节、8T/7AC/8V/3SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/探针或独立final review；全部实现/运行仍待授权work。

## 10. 待澄清事项

1. 本轮暂不实施；研究的blocked/no-go须有真实失败/缺输入/owner与下一动作，不能仅凭设计设想完成。
2. 041新增接口的必需验证若尚不可用，保持相应任务待办；可用现路径先核的部分继续，不借WIP。
3. 完整HMR实施在本研究后另立正式合同；语言SG927–929、Feature/命令/Service、OS/Demo迁移不被接管。
4. 输入/状态/effect规模和支持边界T-01冻结，不给无限资源承诺；原支持不得在研究中悄降。

## 11. 执行证据（work 阶段滚动记录）

### T-01 冻结（2026-11-04，worktree 7471b7c4，分支 codex/plan-042-reload-readiness）

实际入口静态核验（行号以基线为准，详见
D:/autostack/plan-artifacts/plan-042/auto-ui/t01-entry-freeze.md）：

- E1 装配登记：`DynamicComponent::set_source_path`（dynamic.rs:2163），唯一生产调用点 dynamic_build.rs:2111。
- E2 检测：`check_file_changed`（dynamic.rs:2320）纯 mtime 比较（`current > last`）；**多文件 guard（2327）：import_stmts 或 registry 非空 → 恒 Ok(None)**，根与依赖一并跳过（Plan 449/451 实机记录佐证）。
- E3 泵：iced host.rs:2822-2873，`__hot_reload` 订阅 500ms/2000ms（subscriptions.rs:197；`AUTOUI_HOT_RELOAD=0/1` 门控）→ 重读单源 → 重解析 → 第一个 WidgetDecl → `reload`（失败 `let _ =` 静默）。
- E4 `reload`（dynamic.rs:2261）：新 VmBridge 构造失败即 Err（旧 bridge 未替换）；迁移传**空 old_field_defs**（同名乐观保值，类型变化不判）；替换 bridge/view_template/named_templates/input_state_map/tick_interval/span_map，bump_ui_epoch；registry/import_stmts 不在替换范围。
- E5 `write_source_range`（DevTools 回写，回写后自刷 mtime）；E6 action_config 独立热替换（compiler action_config.rs:680 + MCP 工具 action_config_reload）；E7 `hot_reload.rs` HotReloadComponent/UIWatcher 为**存根**（parse_content 恒返回固定节点，无生产调用方）；E8 MCP 无组件 reload 直达工具。
- 041 已合入基线（7471b7c4），docs/specs/runtime/component-and-store-instances.md 为正式合同，T-04 可消费。
- 探针载体：crates/auto-ui-runtime/src/ui_tests/plan042_reload_readiness_tests.rs（真实 Parser+UiDialect→extract→from_decl/reload 链，tempfile 磁盘 fixture；区别于既有旧编号 plan042_scan_probe_tests.rs）。
- 命令：`cargo test -p auto-ui-runtime --lib plan042_reload_readiness`，CARGO_TARGET_DIR 复用主检出 target（独占）。

### T-02/03/04 探针载体（2026-11-04）

- 兄弟源（组内固定、无链接）：.wt/ui-042/auto-lang @ f4062cca（= 主检出 master）、auto-web @ 主检出 master、auto-down @ 7c4d5cbb，均 `git clone --shared` 一次性研究用。
- 探针文件：crates/auto-ui-runtime/src/ui_tests/plan042_reload_readiness_tests.rs（12 例：T-02 单文件 7 例 / T-03 多文件 2 例 / T-04 强制 reload 灾害边界 3 例），挂载 ui_tests.rs（与旧编号 plan042_scan_probe_tests 区分注释）。
- 探针设计要点：正控（真实解析链保值+默认）、负控（同 mtime 漏检 set_modified 拨回实证；同名类型变化乐观保值）、失败保旧（缺源/坏源/reload 内部 Err 三腿）、guard 实测（store/child/root 改动均 Ok(None)）、强制 reload 反例（实例数据灭失/handler 缺席/store 失效/tick_interval 替换）。

### T-02/03/04 探针结果（2026-11-04，`cargo test -p auto-ui-runtime --lib plan042_reload_readiness` → 12 passed / 0 failed，提交 f00e0e11）

- **T-02 单文件**（7 例全绿）：保值+新字段默认正控；删除字段 dropped_names 点名；**同名类型变化乐观保值且旧 Int 值原样写入 str 字段**（write_state 不判型，缺口钉死）；同 mtime 改内容漏检（set_modified 拨回实证）；缺源 Err+旧组件继续+恢复成功；无候选/reload 内部 Err 两失败腿保旧；下一次合法编辑恢复成功。
- **T-03 多文件**（2 例全绿）：真实磁盘 fixture（use store + use widgets）经 build_dynamic_component 装配后业务可运行；store/child/root 三处改动（mtime 前进）均 Ok(None)——guard 实测"整体跳过"非"只 reload 根"；单文件正控检出。
- **T-04 强制 reload 灾害边界**（3 例全绿，绕过 guard 的受控实验，是 guard 的存在理由）：key 实例数据灭失+下一帧按 model 默认重建+迟到实例事件不重定向；**store 数据面丢失（field not found: total）+store handler 缺席+根 Bump handler 因体内 store.Add() 引用候选编译失败灭失**（449/451 实机复现）；tick_interval 由新定义整体替换、Tick 新语义即刻生效。
- **具名缺口（移交）**：截断 widget 残句触发 parser ~51GB 病理分配、进程 OOM 中止；owner auto-lang parser，后继候选 bundle 坏源保旧腿依赖 parser 不崩，须先行修复。

### T-05 冻结（2026-11-04，execution_done 5/8）

- 评审修复（2026-11-04，commit c84cf486，文档-only）：F3-1 规范§2 首检行为补限定（登记成功基线=当前 mtime → 首检无变化；仅 None 基线恒报变化）；F3-2 规范§1 Vue/Vite 行改范围排除表述（本仓无 Vue/Vite HMR 证据，不得计 VM/iced 轨能力）。按 review 建议免复跑探针（措辞修复不影响证据），待 re-review。
- SD-01/SD-03 落 docs/specs/runtime/reload-readiness-and-lifecycle.md（worktree，待 merge 发布）：§1 支持矩阵（E1–E8+Vue/Vite/Node 区分）、§2 检测语义、§3 单文件迁移合同（含同名类型缺口）、§4 失败保旧三腿、§5 owner/effect 边界（guard 存在理由）、§6 后继候选 bundle 原子发布合同（AC/反例/预算/明确未实现）、§7 具名阻断移交。SD-02 差额为 0（041 合同已覆盖，不另写）。
- 研究矩阵结论：单文件 reload = 已实证可用（mtime 检测+名字保值迁移+三层失败保旧）；多文件 = 明确拒绝（guard 保守跳过，不得删 guard 冒能力）；内容级失效/MCP 组件 reload/多文件 HMR = 具名缺口，入后继包合同；parser 病理分配 = 具名阻断（语言仓 owner）。
- 统一回归门（冻结全量）未跑：按合同属 final review/merge 阶段；merge 前不宣称。
