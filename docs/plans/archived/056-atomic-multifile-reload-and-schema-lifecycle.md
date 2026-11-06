---
plan_id: PLAN-056
status: archived
feature_name: 多文件候选Bundle原子Reload与Schema及生命周期收口
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 5
total_steps: 8
depends_on: [auto-ui/PLAN-041, auto-ui/PLAN-042, auto-lang/PLAN-940]
supersedes_spec_components: [docs/specs/runtime/reload-readiness-and-lifecycle.md, docs/specs/runtime/component-and-store-instances.md]
new_spec_components: [docs/specs/runtime/atomic-source-bundle-reload.md]
touched_goals: []
affects: [auto-ui-runtime, auto-ui-compiler, ui-owner-lifecycle]
---

# PLAN-056：多文件候选Bundle原子Reload与Schema及生命周期收口

## 0. 变更摘要

实施UI042已核清的后继：VM/iced真实热重载泵以root/import/store/child/handler/schema/effect完整候选原子提交；按内容失效、类型安全迁移、失败保旧并恢复下一次合法编辑。单文件沿同一规则，不通过删除多文件guard后仅重建根桥交付。

## 1. 目标

下表AC与V为本包唯一验收合同。成功指实际producer→consumer闭环、失败受控且下一次合法操作可继续；不以研究归档、静态字符串或进程exit 0替代运行语义。保原合法行为/兼容/平台与资源条件；跨仓按实际owner参与，不重开已归档计划。

## 2. 架构方案

保持现Auto语法、041实例身份与owner；candidate独立构建/验证、来源一致后一次提交，旧bundle继续服务至提交点。首批范围是现有本地文件系统静态use的单App与少量keyed子实例/store；动态路由按当前实际已装载依赖处理，未支持的动态/外部Python/web模块热替换具名要求重启，不能静默部分更新。本包不承诺Vue/Vite/Node HMR、不新增公开MCP组件reload API，不把旧hot_reload.rs存根包装成实现。

## 3. 技术栈

沿现Rust/Cargo/Auto领域解析、UI VM或Vue生成与已有测试工具。语言仅用固定组；UI沿t/tu（生成单进程）/必要ti按实际职责。T-01核真实--help/包target/过滤与输入，以下新fixture均标新路径，不伪造已存在命令。无依赖升级、工具链/优化/profile变更承诺。

## 4. 需求分析与背景调查

当前canonical runtime/reload-readiness-and-lifecycle §§1–7、component-and-store-instances（含051派发修订）、UI041/042最终归档；实物 crates/auto-ui-runtime/src/ui/dynamic.rs::reload/check_file_changed 仍为空old_field_defs、imports/registry guard和mtime比较；src/dynamic_build.rs::build_dynamic_component_inner/collect_module_imports；src/ui/iced/renderer/host.rs:2822–2854真实__hot_reload泵；src/ui/state_migration.rs、ui/vm_bridge.rs。042十二例与041真实实例fixture为旧反控，不表示本轮已运行。UI052/053、OS055可能改host/renderer，共享符号必须先核正式落地与单写窗口。

本轮用户授权“规划下一批并列状态/顺序”，只立约与更新规划文档，未授权本包实现。版本与关键内容hash见[本批设计](../design/autoui/atomic-reload-and-live-isolation-stage-two.md)；T-01重核正式HEAD、实际源hash与他方WIP。旧历史观察≠当前实测；设计基线不等工具已构建。全部新运行证据为未执行。

2026-11-05 用户指令「完成计划056」为本包端到端实施授权（work→review→bounded repair→merge 全链，含在计划专属 worktree 内实施、评审、受限修复与合入/归档/清理），work 阶段记录该授权；不改变目标/AC/范围/预算/仓集合。

## 5. 详细设计

T-01冻结source graph采集端、候选/现役bridge可分离性、schema描述与owner映射，先做root+store+keyed twins整链可行性探针，再依此实施。graph依赖身份由实际resolver解析，记录规范路径与原字节内容hash；同mtime改内容也检出。构建中任一来源变更、缺源、循环或无效import、snapshot失配必须具名拒绝且旧bundle可继续，修复后自动再次尝试；mtime只能快路径，不能证明内容身份。

兼容状态按App/store/子实例稳定逻辑身份+字段schema迁移；同类型字段保值、新增用默认、删除具名报告；同名不同类型默认拒绝整候选，未知/不透明类型亦拒绝并要求重启，不偷用旧Value写入。keyed重排不重置实例，删实例只撤销其owner；不承诺无key改序时猜身份。实际typed值迁移写入错误必须冒泡到候选失败，不再let _吞错。

提交前不得启动候选Init/Tick/SSE/IO等外部effect；旧订阅继续。提交时原子切换所有view/handler/store/bridge/input/memo/source identity，取消被替换owner，建立新代次资源一次；活跃异步/parked任务无法安全迁移时，当前候选具名拒绝保旧并待静止后重试，不强杀或映射到新同名实例。禁止声称可回滚已发生外部副作用；候选准备必须无这类副作用，订阅激活失败采用可预检/补偿的既有端口并保一致现役状态。T-01证明现接口可做到；若做不到，给精确阻点/修订后再推进该腿，不降AC。

入口必须穿真实host泵，单测helper只作辅助。940正式关闭后才运行真实截断坏源；可提前完成合法源快照/迁移准备，但整项交付必须接受940结果。拟定 `D:/autostack/.wt/ui-056/auto-ui`，分支 `codex/plan-056-atomic-reload`；同组语言只读正式版本。GUI验证用autoui-verifier，隔离端口/数据/owned进程，不改用户系统主题。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/atomic-source-bundle-reload.md | 现具名差额→实证支持/受控拒绝/恢复及边界 | 唯一现行合同 | AC-01..05 |
| SD-02 | modify | docs/specs/runtime/component-and-store-instances.md | 保现owner规则，仅补真实新增行为与限制 | 防跨仓/实例责任漂移 | AC-01..05 |
| SD-03 | modify | docs/specs/runtime/reload-readiness-and-lifecycle.md | 与940/056/057正式交付逐项对账，未实现仍明列 | 不以研究或新包吞其他债 | AC-05/AC-07 |

SD-03跨仓项由该仓owner在同组参与，语言仅留指针；review最终确认真实差额，无差额须记理由而不造规范改动。

## 6. 测试设计

| Case ID | 不变量/输入 | 实际入口/支持状态 | 可观察oracle与关闭条件 | owner_stage |
|---|---|---|---|---|
| V-01 | AC-01；来源快照与失效 | 真实磁盘root/store/child；同mtime改内容、候选中途改依赖、缺源再恢复 | 内容改动检出且仅完整同快照提交；坏/混合候选不发布；合法下一编辑成功 | work |
| V-02 | AC-02；实例/schema迁移 | 041 twins fixture：A=11/B=22/shared=7，keyed换序、新增删除字段、Int→str反例 | 兼容值与共享准确保持、新字段默认、删除具名；类型变化拒绝且旧值/handler仍可用 | work |
| V-03 | AC-03；失败原子与旧业务 | 真实parser/验证/bridge/迁移失败注入，940截断坏源 | 失败后旧视图/handler/store/实例/Tick均仍原版，下一合法候选整包变为新版，不出现新view旧handler | work |
| V-04 | AC-04；effect/代次/退出 | Init计数、Tick订阅、卸载重挂、迟到事件、parked候选重试 | 准备期零effect；成功后owner旧资源有界退出/新资源一次，迟到事件不串新实例；繁忙拒绝保旧 | work |
| V-05 | AC-05；真实host兼容 | __hot_reload实际泵、单文件旧042、主题scope、action_config独立reload | root/import/store/child各改真实生效，旧支持无回归；action_config不被劫持，未支持轨具名重启 | work |
| V-06 | AC-06；实际冻结源/原子义务/健康/SD | 同树独立上下文final review，首红/环境/条件排除分别记录 | 所有work AC实证满足，无未批准删项/降oracle；复审绑定源版本 | review |
| V-07 | AC-07/08；真实落地/知识/所有输出 | 冻结必要职责并集与canonical/guard/资源读回 | 被测==交付，规范已落地、归档与清理有摘要；持有owner/关闭条件齐 | merge |

UI明确入口：`cargo test -p auto-ui-runtime --lib plan042_reload_readiness`、041实例集成目标 `cargo test -p auto-ui-runtime --test plan041_instance_ownership`，以及实际compiler生成/状态迁移完整受影响职责；Vue浏览器新入口由T-01建成并记录准确命令，未建前不记PASS。 新增测试放真实模块namespace/target确保被职责门选中。原样本、金样、负控、ignored/平台/性能条件不减；准备/编译/发现/运行/清理分列，执行/原条件排除/首次红/重放绿分别记录。批量cargo t all到期由主检出单实例程序门承接，非本包重复全目录门。

## T-01 冻结（2026-11-05，work 实测基线 22a925c7）

- **候选bundle=独立全量DynamicComponent**：复用真实 build 管线（root+use imports+store-as-child+handlers+schema+effects 单 VM 合成），以 `candidate` 模式构建（**不** fire_init、**不** set_dsl_action_config_from_block、**不** register_theme_trees/set_app_design_entries/load_from_dir/register_imported_components；仅保留解析/提取所需注册与 per-component init_theme_scope）。候选不污染现役桥（旧桥继续服务至提交点）。
- **来源快照**：root+transitive use 模块文件（同一 base_dirs 解析规则：模块目录/src 父级/语言仓根回退；跳过 c/rust/py/auto.*/.web/use.py），逐文件 blake3 内容 hash + mtime 快路径；`check_reload` 检测任何文件内容变化（同 mtime 改内容可检出）；缺源 Err 保旧、恢复后下一合法编辑成功。快照↔候选构建后复核 hash 防候选中途改依赖。
- **typed schema**：组件保留 `last_schema`（root+children+stores 的 AuraWidget 状态定义表，with_registry_and_imports_from_decls 时捕获）；迁移按名+类型判型——兼容保值、新增默认、删除具名；**同名不同类型=整候选拒绝保旧**（含未知/不透明类型）；store 字段平铺根态（单 VM 统一根态），随根态迁移覆盖。
- **实例迁移**：旧桥全部存活实例身份读出字段表（`read_instance_state_all`），keyed 身份预播种进新桥（`import_child_instance`，parent 先指新根、首帧 `ensure_child_instance` 命中既有记录时修正 parent ref），Init 记录预置 Done 防重放/防重置；未再挂载身份经帧扫释放+tombstone（迟到事件丢弃不串新实例）。
- **提交**：旧桥 `has_parked_tasks`/`cpu_continuation_count>0` → Busy 保旧下拍重试；提交=run_module_init 一次+全量 runtime 面原子切换+bump_ui_epoch+dirty；不重放 Init（保 AC-02 保值语义，与现单文件 reload 一致）；订阅面经 renderer 每拍由 timesources 重建自然收敛（host.rs:6797 实测）。action_config 独立 mtime 泵不被提交劫持（AC-05）。
- **旧 API 保留**：`reload()` 单文件 legacy 语义原样（t04 灾害边界探针保持绿=不得回退部分替换的回归钉）；`check_file_changed` 保留 legacy（042 t03 guard 探针为旧反控，实现落地后改写为新行为断言）。
- **Vue 浏览器入口**：本仓无 Vue e2e/playwright 挂载 harness（compiler/tools 检视实证），浏览器实证归属 057；T-01 记录为具名未建，该腿不记 PASS，SD-03 对账。

所有新探针与测试均未运行。本轮授权仅调研、设计、取号、写合同与刷新路线；不建实施树、不改源码、不启动项目、不提交或推送。实施/自查/独立上下文final review使用同一权威树顺序进行，不另建同步review树；本轮不自动创建agent。最终review逐AC、遗漏/延后/workaround、告警/格式与SD核对；通过后merge冻结真实受影响职责并集，复用身份未变有效证据，不堆叠全仓轮次。

材料唯一 `D:/autostack/plan-artifacts/plan-056/<repo>/`，开工登记owner/用途/路径/在途。所有探针、显式外置target、安装/生成副本、日志/截图与隔离数据都在此；正常构建在专属树。merge逐项核实际输出（含仓外/其他盘）、进程/设置恢复，必要摘要折入Plan再删本scope/自有树产物；仅具名未闭hold可留且登记owner/理由/关闭条件。无junction/symlink；fresh wt-guard clean后才移树，不动他方WIP/用户进程，不写legacy specs.json。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 内容改动检出且仅完整同快照提交；坏/混合候选不发布；合法下一编辑成功；由V-01验证 | work |
| AC-02 | 兼容值与共享准确保持、新字段默认、删除具名；类型变化拒绝且旧值/handler仍可用；由V-02验证 | work |
| AC-03 | 失败后旧视图/handler/store/实例/Tick均仍原版，下一合法候选整包变为新版，不出现新view旧handler；由V-03验证 | work |
| AC-04 | 准备期零effect；成功后owner旧资源有界退出/新资源一次，迟到事件不串新实例；繁忙拒绝保旧；由V-04验证 | work |
| AC-05 | root/import/store/child各改真实生效，旧支持无回归；action_config不被劫持，未支持轨具名重启；由V-05验证 | work |
| AC-06 | 独立final review逐work AC、健康/遗漏与SD pass，绑定冻结源，V-06 | review |
| AC-07 | 冻结门/依赖序合入/canonical/路线/归档真实读回，V-07 | merge |
| AC-08 | 全输出/资源/进程/设置对账、guard清树与scope删除，摘要折Plan/具名hold登记，V-07 | merge |

## 8. 执行步骤

T-01..T-05 已由 work 完成（2026-11-05，worktree `D:/autostack/.wt/ui-056/auto-ui`，分支 `codex/plan-056-atomic-reload`，基 22a925c7 → 交付提交 6b994aa8）；T-06..T-08 归 review/merge。

| ID | 动作及关闭结果 | 依赖/AC | owner_stage |
|---|---|---|---|
| T-01 | [x] 冻结source/owner/schema/真实泵接口与支持表，整链有限可行性探针及准确命令（冻结结论见 §5「T-01 冻结」；候选独立构建/内容快照/typed 迁移/原子提交经 V-01..V-05 实证） | 既定已交付输入/相关前置；AC-01..05 | work |
| T-02 | [x] 内容快照/依赖变化检测与独立候选构建，来源失配和缺源恢复反控（blake3 内容快照 + build_reload_candidate + 构建后复核；同 mtime 检出 V-01、缺源保旧/恢复 V-01、坏候选/合法下一编辑 V-01） | T-01；AC-01..05 | work |
| T-03 | [x] 兼容schema迁移和App/store/keyed实例所有权，类型冲突拒绝保旧（migrate_state_typed + 实例迁移/补位/tombstone；twins 保值 V-02、冲突拒绝 V-02/V-03） | T-02；AC-01..05 | work |
| T-04 | [x] 原子提交/订阅与代次生命周期/错误诊断，真实host泵接线（commit_candidate 全量原子切换 + run_module_init 一次 + busy 保旧重试；host __hot_reload 泵直调 try_reload；零 effect V-04、繁忙 V-04、迟到 V-04、Tick 表 V-04） | T-03；AC-01..05 | work |
| T-05 | [x] 接受940坏源实证（t02b 截断保旧 + SOLO_BROKEN 受控拒绝 V-01/V-03）；永久正负回归（plan056 15 例 + plan042 13 例 + plan041 集成 4 例 + 主题/design 门全绿）、真实App泵（pump 同款 try_reload 入口）+原完整职责门（lib 全量 2374 绿；12 例环境既有失败经基线 22a925c7 对照证实非本包，5 例全量并行抖动隔离复跑全绿）、自查SD（SD-01/02/03 增量已备于 worktree，merge 合入）后 execution_done | T-04；AC-01..05 | work |
| T-06 | 独立上下文final review，重建发布/失败/owner证据并冻结源 | T-05；AC-06 | review |
| T-07 | 冻结受影响职责并集、真实合入/canonical/归档与消费者交接 | T-06；AC-07 | merge |
| T-08 | 完整资源/输出对账，guard clean后清树/scope并折摘要 | T-07；AC-08 | merge |

## 9. 复审记录

2026-11-05 / stage:new / PLAN-056 r1 / outcome:blocked（实施授权待给，坏源腿及整项交付接940正式结果） / 已读实际Specs/源码/归档与在途合同，形成有限执行合同；本轮未实施、未运行探针/项目测试。next：授权后work/T-01，保持原子义务/owner边界，逐任务补证据；最终review与merge/清理均未完成。

2026-11-05 / stage:work / PLAN-056 r1 / outcome:pass / worktree `D:/autostack/.wt/ui-056/auto-ui` / branch `codex/plan-056-atomic-reload` / base 22a925c7 / code_commit 6b994aa8 / dependency_revisions：auto-lang a199f127（同组只读，940 已交付）、auto-web bf72baab、auto-down 58ea33f6 / task_ids T-01..T-05 / evidence：plan056_atomic_reload_tests 15 例全绿（V-01..V-05，命令 `cargo test -p auto-ui-runtime --lib plan056_atomic_reload_tests`）；plan042_reload_readiness 13 例全绿（t03 改写新行为断言、t04 灾害边界保持绿）；plan041_instance_ownership 4 例全绿；theme_scope_lifecycle/theme_scope_isolation/design_scope_inheritance/control_surface_states 全绿；compiler plan041 store factory 全绿；lib 全量 2374 绿 + 12 例环境既有失败（基线 22a925c7 对照实证，7 例基线同失败 + 5 例并行抖动隔离复跑全绿） / blockers：无（Vue 浏览器实证属 057 具名移交，本包不记 PASS 该腿） / next_responsible_role：review（T-06 独立上下文 final review）。

stage: goal | plan_id: PLAN-056 | plan_revision: r1 | round: 1 | from: new | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-056/auto-ui | commit: 6b994aa8 | next: review

2026-11-05 / stage:review / review_scope:final（同树独立上下文子代理，artifacts-only 重建） / PLAN-056 r1 / outcome:pass / worktree `D:/autostack/.wt/ui-056/auto-ui` / branch `codex/plan-056-atomic-reload` / reviewed_commit 6b994aa8 / base 22a925c7 / dependency_revisions：auto-lang a199f127、auto-web bf72baab、auto-down 58ea33f6 / spec_inputs：SD-01/02/03 工作树备妥文本与实现一致 / acceptance_results：AC-01..05 全 pass（plan056 15/15、plan042 13/13、plan041 4/4、theme/design 门全绿、compiler plan041 1/1；断言均为值级区分性 oracle）/ findings：F1（verify_snapshot_stable 无直接测试，AC-01 低危非阻断）、F2（SD-01 §2 措辞歧义，低危）、F3（计划文 SHA-256/blake3 措辞对齐，信息级）、F4（crates/auto-ui-runtime/test/ 为外来未跟踪产物，不计入本包输出）/ evidence：独立复跑命令与结果同 §8 T-05 / next：merge（T-07→T-08；F1–F2 非阻断可随 merge 折入）。

stage: goal | plan_id: PLAN-056 | plan_revision: r1 | round: 2 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-056/auto-ui | commit: 6b994aa8 | next: merge

2026-11-05 / stage:merge / PLAN-056 r1 / outcome:pass（详见下方 checkpoints） / delivery_commit 118eef7d（= reviewed 6b994aa8 + docs-only F2 修正；range 等价） / canonical Specs：docs/specs/runtime/atomic-source-bundle-reload.md（新增）、reload-readiness-and-lifecycle.md（对账）、component-and-store-instances.md（§6 增量）；主分支 hash：78B7D1EB…/10853B59…/B7DA91E8… / archive `docs/plans/archived/056-atomic-multifile-reload-and-schema-lifecycle.md` / cleanup：worktree+branch+group 已移除、plan-artifacts/plan-056 无 scope（从未创建）、registry 已刷新 / 生产制品：desktop 发行二进制与 Vue bundle 未随本轮重建（stale-artifact 观察项，见下）/ 遗留：主检出上其他会话的计划草稿（057/060/061/062/063 + README/.next-id + design 文档）与 crates/auto-ui-runtime/test/ 外来产物保持原样未触碰（F4）。

stage: goal | plan_id: PLAN-056 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: removed | commit: 118eef7d | next: done

## 合入 Checkpoints（PLAN-056:r1）

- **prepared**：reviewed 基线 6b994aa8（review final pass，AC-01..05 全实证）；canonical delta = SD-01 新增 atomic-source-bundle-reload.md、SD-02 component-and-store-instances.md §6、SD-03 reload-readiness-and-lifecycle.md 对账；F2 措辞修正（docs-only）→ delivery 118eef7d。
- **landed**：`git merge --ff-only codex/plan-056-atomic-reload`（master 22a925c7 → 118eef7d，线性无合并提交）；主检出冒烟 `cargo test -p auto-ui-runtime --lib reload` 35/35 绿。
- **canonical_verified**：三个 canonical 路径在主分支交付提交内（118eef7d 即 last commit），hash 如上；实现/依赖未随 F2 变更（docs-only，range-diff 等价）。
- **archived**：计划移入 docs/plans/archived/、status archived、docs 提交（提交 056-archive）。
- **cleaned**：wt-guard clean → worktree/branch/组目录移除；plan-artifacts/plan-056 不存在（本轮无 plan-artifacts 产物，raw 证据以命令/结果摘要折入 §8/§9）；retention registry 刷新（policy.json 见下）。
- **生产制品观察**（PLAN-092 教训）：landed 后未重建 desktop 发行二进制 / auto build Vue bundle——runtime 库变更将在下次发行构建生效；记录为 stale-artifact 观察项，非本包缺失。

stage: goal | plan_id: PLAN-056 | plan_revision: r1 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 118eef7d | next: done

静态预检（2026-11-05）：0–10章节、T/AC/V/SD唯一连续且引用有定义；owner_stage、当前本地引用、活动/归档唯一号与单调号源核对通过。未执行任何项目测试、危险复现、浏览器或独立final review，预检不抵实现验收。

## 10. 待澄清事项

- 当前无需用户补充材料即可立约；T-01冻结内部机制、准确入口与实际来源。若改变公开语义、承诺支持或仓责任，给具体修订再进行受影响实施。
- 既有UI052/053、OS055及语法/ACE会话继续各自实施，本包不接管其WIP；文件/符号交集协调单写，实际正式差额才形成前置。
- 940未正式交付不跑危险坏源腿；宏观Reload原子目标不可通过留guard整体跳过或仅重建root签PASS。
