---
plan_id: auto-ui/PLAN-052
status: archived
feature_name: rust-ui 原生 exe 生成器对应用 back 源的编译修复
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 5
total_steps: 5
base_branch: master
base_commit: 8ed8a2e1f2dc93bf31d6f730d380f55bdbc9baa1
depends_on: []
affects: [rust-ui, a2r-codegen, plan-910-publish, auto-os/PLAN-054]
---

# auto-ui/PLAN-052：rust-ui 原生 exe 生成器对应用 back 源的编译修复

## 0. 变更摘要

`auto build -r rust`（PLAN-910 发布链）对 auto-os 统一组合 8 个基础应用
（notes/todo/calendar/paint/database/chat/blog/charts）中 6 个 Rust back
应用生成 68 个 Rust 编译错误；同一锁定工具链组合下 002-counter 对照
**PASS**（exe 发布成功、0 错误）。本计划修复 rust-ui/a2r 生成器对这些
app back 源 constructs 的失配，使 6 app 原生 exe 可构建——auto-os/PLAN-054
iced 轨 Rust 六雄 Outproc 嵌载的具名最小前置。

## 1. 事实与证据（2026-11-05 T-04 实机）

- 复现：候选组合 `D:/autostack/.wt/os-054/auto-os/apps/auto-notes` 执行
  `auto build -r rust` → cargo exit 101，生成 member `notes-45ee43a7` 68 个
  错误；同批 todo/calendar/database/chat/blog 全部同型失败。
- 错误签名（notes，去重后）：E0071/E0117/E0277/E0308/E0425/E0428/E0433/
  E0599/E0609；典型例：`expected Option<serde_json::Value>, found Value`
  （生成码返回类型失配，`serde_json::Value::Null` 未包 `Some`）。
- **合法对照**：`examples/ui/002-counter` 同锁定组合（auto-lang 70b4a1de7 /
  auto-ui b3414750 / auto-man 03bb65e / auto-web 37d32c22）`auto build -r rust`
  PASS，`rust-workspace/002-counter-6e81e48a/bin/debug/counter.exe` 发布成功。
  → 组合无恙，失配为 app back 源特化（这些 app 使用较新 back constructs：
  中文标识符/复合类型/Option 返回等，对照 counter 的极简 back）。
- 历史锚：NOTES-004 时代 rust-ui 重写（plan-910）后 notes 原生曾 19/19
  （tool=auto-ui 2f43b4f），当前 b3414750 对该源失败——疑 2f43b4f→b3414750
  间 rust-ui 生成器回归或对新增 constructs 未覆盖（T-00 定位现有端口后结论）。
- 生成器位置：`crates/auto-ui-tools/src/rust_ui.rs` + PLAN-910 发布链
  （`auto-generation-core::generated_build`，auto-lang 侧消费接口）。

## 2. 目标

- G1：6 app back 源在 master 当前工具链下 `auto build -r rust` 全部 PASS，
  exe 按 PLAN-910 约定路径发布。
- G2：002-counter 及既有 rust-ui 门零回归（对照组与受影响旧门全绿）。
- G3：修复为生成器侧通用能力（constructs 级），不改任何 app 仓源码
  （app 源在各自 002/004/005 交付基线上冻结）。

## 3. 边界

- 不评 8 app 业务行为；不动 auto-os pin；不碰 desktop-host（另案
  auto-ui/PLAN-053）；不在 auto-lang 跑全量 `cargo t`（Category A 纪律，
  只跑受影响固定组）。

## 4. 验收标准

- AC-01：6 app（notes/todo/calendar/database/chat/blog，交付 pin）exe
  构建 PASS，0 编译错误。
- AC-02：002-counter + 既有 rust-ui/910 单测与冻结门零回归。
- AC-03：notes exe 冷启动冒烟（发布路径发现 + 启动退出码 0）。
- AC-04：修复 diff 只落生成器（auto-ui 本仓），逐错误家族具名对账
  （68 错误分族→根因→修复→重测）。

## 5. 执行步骤（草案，待 work 细化）

- [x] T-00：以 notes 为最小复现，50 错误（非 68——重测全量日志去重后）分五族，逐族定位生成器根因；产出 affected-constructs 表与修复策略。[✅ 2026-11-05] 证据：docs/plans/evidence/052/codegen-families.md——F1 实体字段点访问×24（E0609，user type 降链 Value 但 store 发点访问）/F2 用户型双重定义（E0428+E0117，alias 通道与 api struct 通道未仲裁）/F3 子组件 Msg 未入编译集×7（E0425 NavTreeMsg/EditorPanelMsg）/F4 api 桩 Option 缺省×3（E0308，default_expr `_` 臂恒 Null）/F5 杂项待复测定档。对照 counter PASS 与族表自洽。
- [x] T-01：生成器修复 + notes 单 app 构建绿。[✅ notes 50→0，notes.exe 发布；族谱见证据文档 F1-F5]
- [x] T-02：6 app 全量构建 + counter/旧门回归 + notes 冒烟。[✅ 2026-11-05 19:16 sweep-r8：notes/todo/calendar/database/chat/blog 全部 exit 0 + 0 错误 + exe 发布（notes 84MB 19:13:20 / todo 83MB 19:13:46 / calendar 83MB 19:14:34 / database 84MB 19:14:59 / blog realworld 85MB 19:15:55 / chat 84MB 19:16:16）；002-counter exit 0。blog 191→0 含多路由静态 router（路由表/__Navigate/outlet/link/router.push/param/页 store 同步）。AC-03 冒烟：notes.exe 启动存活 8s + CloseMainWindow → exit 0。单元门：nextest compiler+tools 565 run / 560 pass / 5 红均为 vue/gallery 与 auto-lang 受管语料（diff 无交集，语料期望 149 vs auto-lang master 实有 123）；shell-pack freshness 门已 regen 入库并绿。证据：evidence/052/codegen-families.md「T-02 终态」；提交 886a22de + 021582ef。]
- [x] T-03：漂移核对/文档/收据，提交 execution_done。[✅ base_commit 8ed8a2e1 在祖先链；diff 仅 auto-ui 本仓生成器（rust.rs/rust_ui.rs）+ shell-pack 生成物 + docs；auto-lang/auto-web/auto-down 兄弟树 dirty=0；证据/收据已更新。]
- [x] T-04：复审。[✅ 2026-11-05 自审（同一实现会话，limitation 已记录）：AC-01/02/03/04 全部 pass；5 个 nextest 红在主检出 master 上同样复现（预存在，非回归）；规范增量=空（附说明）。]
- [x] T-05：merge 按本仓流程。[✅ 2026-11-05 landed 64aef780（ff-only，无 merge commit）；canonical 无 spec 变更（no-impact）；archived → docs/plans/archived/；worktree/branch 清理 + plan-artifacts 删除见 merge 收据。]

## 6. 规范增量

无（空影响说明）：本计划是 rust-ui/a2r 生成器能力修复——多路由 `routes{}` /
`outlet` / `link(to:)` / `router.push` / `router.param` 的静态发射，以及 store
归属、typed 槽位收口、api 桩签名等；均为生成器内部输出行为，`docs/specs/`
无对应 rust 代码生成契约条款（此前多路由在 Outlet 位以 `compile_error!` 响亮
拒绝，无 spec 承诺可被修改或退役）。

- `supersedes_spec_components`: []
- `new_spec_components`: []
- `touched_goals`: []

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-052 | outcome: drafting（事实证据随案；
  由 auto-os/PLAN-054 T-04 定界产出，消费方=054 恢复后 iced Rust 六雄） | next: work/T-00

## 10. 待澄清事项

编号 052 为立项时占用（.next-id 滞后记 050，以本会话冲突检查 052/053
无在飞分支/树为准）；如与他案撞号，执行前取新号并重命名。
- stage: work | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | outcome: pass（T-00 完成，T-01 F4 已修实测在飞） | worktree_path: D:/autostack/.wt/ui-052/auto-ui | branch: codex/plan-052-dev | base_commit: 8ed8a2e1 | code_commit: 待提交（F4 编辑在工作树，随验证结果提交） | task_ids: T-00 | evidence: 五族表 docs/plans/evidence/052/codegen-families.md（50 错误全量分族+生成器锚点+修复策略；F2 附加设计输入：api struct 直编与 stub API_DATA Value 桶的边界冲突记录）；组内兄弟 auto-lang/auto-web/auto-down 按 master 补齐（metadata 绿）；tool 已构建 | blockers: 无 | next: work/T-01（F4 实测 → F2/F1/F3 逐族修）
- stage: work | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | outcome: pass（T-01 完成：notes `auto build -r rust` 0 错误，notes.exe 发布于 rust-workspace/auto-notes-45ee43a7/bin/debug/notes.exe） | worktree_path: D:/autostack/.wt/ui-052/auto-ui | branch: codex/plan-052-dev | code_commit: 本提交 | task_ids: T-01 | evidence: 50→0 全程族谱（F1-F5 逐族修复：F4 Option 缺省/F2 alias 仲裁/F3 配方预注册+模块定位+文本兜底/E0608 struct 投影/ref 循环变量追踪/sidebar 族 a2r 映射/比较面 typed 归一/DELETE bool 返回）；002-counter 对照仍 PASS（T-02 复验） | next: work/T-02（6 app 全量 + counter 回归 + notes 冒烟）
- stage: goal | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-052/auto-ui | commit: c7f7da57 | next: work/T-02（blog 多路由 router 静态发射）
  轮内容（T-02 进行中，非阶段完成）：并行 PLAN-054 会话按用户裁决让出共享树，其补丁 ea9272cb 已并入基线继续。本轮实机 sweep（生成器全量再生成）：notes 0 / todo 0 / calendar 0 / database 0 / chat 95→0（exe 18:13:57 发布）/ blog 191→17 / 002-counter 0。修复 10 族（try 体扫描、无 api imports 工程实体型、typed prop↔Value 桶桥、多级 Value 点链、逐文件多 store 注册与 store 归属、nil 哨兵、typed 记录字段 clone、Option/Value 槽收口、无 handler 按钮 View<()>、raw ident/无载荷形参补绑定），逐项证据见 docs/plans/evidence/052/codegen-families.md「T-02 执行场」。剩余：blog 15×router + 多路由拒绝门（静态路由功能缺口，下一轮实现）+ follow() 零参 stub（auto-lang parse_api_module 中段路径参数面，非本仓）。回归：多 store/无条件点击移除等共享改动后 notes/todo/calendar/database/counter 复测全绿（sweep-r4）；chat 于最终 HEAD 复测绿。
- stage: work | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | outcome: pass（T-02/T-03 完成，execution_done） | worktree_path: D:/autostack/.wt/ui-052/auto-ui | branch: codex/plan-052-dev | base_commit: 8ed8a2e1 | code_commit: 021582ef | task_ids: T-02,T-03 | evidence: sweep-r8 全绿（6 app 0 错 + exe 发布；counter 0）；notes 冷启动 exit 0；nextest 560/565（5 红为 vue/gallery + auto-lang 语料漂移，diff 无交集）；shell-pack freshness 绿；diff 范围仅本仓生成器+生成物+docs | blockers: 无 | next: review（auto-plan-review）
- stage: goal | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | round: 2 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-052/auto-ui | commit: 021582ef | next: review
  轮内容：blog 191→0（静态多路由 router 全链：__ROUTES/__route_match/__Navigate/__apply_route/outlet match/link 点击/router.push+param/页面 canonical store 双向同步）、follow() 零参 stub → POST 桩按契约全参、shell-pack 入库物 regen。全量 sweep-r8 六 app 全绿 + counter 绿；notes 冒烟 exit 0；nextest 560/565（5 红预先存在：vue/gallery 3 + vue theme 1 + auto-lang 受管语料 149 vs 123，均为 diff 无交集面）。T-03 漂移核对完成：diff 仅 auto-ui 本仓生成器/生成物/docs，兄弟树 dirty=0。计划状态 execution_done，进入 review。
- stage: review | review_scope: final | reviewer_identity: 本实现会话自审（limitation：与实现同一 context，无独立 reviewer 会话；verdict 由 artifacts/命令重放重建，未采信执行者摘要） | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-052/auto-ui | branch: codex/plan-052-dev | reviewed_commit: 021582ef（review 记录提交见后随 docs commit） | base_commit: 8ed8a2e1 | dependency_revisions: auto-lang 122725e9a / auto-web 37d32c2 / auto-down 7c4d5cb / auto-man 03bb65e（`auto` = 03bb65e 构建） | spec_inputs: docs/specs/ui、docs/specs/runtime、docs/specs/modules 已读；无 rust 代码生成/router 契约条款（规范增量空，见 §6 说明；live specs 未改） | acceptance_results: AC-01 PASS（sweep-r8 六 app exit 0/0 错 + 六 exe 发布于各自 rust-workspace/<member>/bin/debug/，笔记 84MB–blog 85MB/19:13–19:16；counter exit 0）；AC-02 PASS（counter 0 错；nextest 565 run/560 pass；5 红= vue theme 1 + vue gallery 3 + 受管语料 1，均在主检出 master（25d4ea5d）同测复现，属预存在基线红，非本 diff 回归；shell-pack freshness 门 regen 后绿）；AC-03 PASS（notes.exe 发布路径发现→启动存活 8s→CloseMainWindow→exit_code=0）；AC-04 PASS（diff 8ed8a2e1..HEAD 仅本仓 crates/auto-ui-compiler/src/ui_gen/rust.rs、crates/auto-ui-tools/src/rust_ui.rs、crates/shell-pack/src/lib.rs（生成物）+ docs/plans；兄弟树 dirty=0；逐族对账见 evidence/052/codegen-families.md） | pending_closeout_gates: merge 阶段：计划文件在分支（主检出无该文件，历史交接即如此——merge 时随分支落地）；plan-artifacts/plan-052 清理；无 code gate 欠账 | findings: 无阻塞项。观察 2 条（不降级、非本计划 AC）：① 删除 `*-back` 后首次 backend refresh 偶发 exit -1（`-back` crate 手工 cargo build exit 0，保留 `-back` 后 sweep 全绿；另有 orphan cargo 占锁致瞬时 -1，清理后复测绿）——属 auto 发布链/环境，另案跟踪；② 路由页 `__apply_route` 每次切页重放页 Init（含首帧），页面 API 可能被调两次（业务行为不在本计划边界 G3/§3）。 | evidence: D:/autostack/plan-artifacts/plan-052/{sweep-r8-summary.log,sweep-r1-auto-*.log,nextest-gate-final.log,review-main-checkout-reds.log,notes-build-final.log}；durable 摘录=evidence/052/codegen-families.md「T-02 终态」+ 本记录 | next: merge（auto-plan-merge）
- stage: merge | plan_id: auto-ui/PLAN-052 | plan_revision: r1 | outcome: pass | delivery_commit: 64aef780（ff-only：master 25d4ea5d → 64aef780，无 merge commit） | base_commit: 8ed8a2e1 | reviewed_commit: 021582ef（rebase 前）→ rebase 后等价链 9c7fb0d8/a02932bc/64aef780；`git range-diff 8ed8a2e1..5cbf6455 master..64aef780` 全 35 项 `=`（零 `!`），patch 等价 | dependency_revisions: auto-lang 122725e9a / auto-web 37d32c2 / auto-down 7c4d5cb / auto-man 03bb65e | canonical_specs: 无（approved no-impact delta，见 §6；docs/specs 未改） | archive_path: docs/plans/archived/052-rust-exe-codegen-fix-for-app-backs.md | completion_kind: delivered
  检查点：prepared=PASS（工作树 clean、reviewed HEAD 021582ef、delta 空、targets 无 spec）；landed=PASS（master tip=64aef780=delivery commit，线性）；canonical_verified=PASS（no-impact delta 经复核；main 集成冒烟：cargo build -p auto-ui-tools 40.7s 成功、examples/ui/002-counter `auto build -r rust` exit 0 且 counter.exe 87MB 19:30:29 发布——注意主检出须用主树 AUTO_* env，混用 ui-052 auto-web 会撞 auto-web-contract 双链接）；archived=PASS（git mv 入 docs/plans/archived/ + status: archived，docs/ plans 已随 64aef780 入库）；cleaned=见下条追加。
  检查点续：cleaned=PASS（wt-guard /mnt/d/autostack/.wt/ui-052 clean x2；4 个 worktree（auto-ui/auto-lang/auto-web/auto-down）经各自仓 `git worktree remove` 移除，组目录 .wt/ui-052 已空并删除；分支 codex/plan-052-dev 删除（原 64aef780，已 landed）；plan-artifacts/plan-052 全 scope 56 文件删除（证据已折叠于本文件 T-02 终态/T-04/§6）；.retention/policy.json 已登记 merge_closeouts）。交付物检查：本次为生成器源码修复，主检出 `cargo build -p auto-ui-tools` 已于 19:30 重建 debug tool 并 counter 冒烟绿；无 backend release/daemon/web bundle 产物需重建（vue bundle 不受 rust 轨影响）；批量回归：auto-ui 无 docs/plans/.last-batch-regression.json 门定义（039/040 先例 D-05），交 regress owner 按需执行，不阻断本次交付。
  交付判定：AC-01..04 全 pass（见 T-04 记录），merge pass，completion_kind=delivered。
