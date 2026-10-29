---
plan_id: PLAN-010
status: archived
feature_name: vue-bare-store-method-routing
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 4
total_steps: 4
depends_on: ["auto-ui/PLAN-003"]
supersedes_spec_components: ["docs/specs/runtime/native-vm-app-validation.md"]
new_spec_components: []
touched_goals: ["auto-blog/BLOG-004"]
---

# PLAN-010：Vue 渲染器多 store widget 裸 store.Method 按归属路由（修复页面 mounted 中断）

## 0. 变更摘要

BLOG-004 T-05 验收新鲜证据：多 store widget（Blog `app.at`，导入 Auth/Article/Draft 三 store）handler 体裸 `store.LoadFeed()` 被 vue 轨绑定到第一个导入的 AuthStore → App.vue mounted 抛 `store.LoadFeed is not a function` → Vue post-flush 队列中断 → 同批后入队的页面级 mounted 回调（author 页 LoadList + 3s 自动保存 timer 注册）被丢弃。运行时表现：自动保存永不触发、刷新后列表不可达；页面内手动调 `store.AutosaveTick()` 立即 PUT 正常（机制健康）。本计划只修 vue 轨裸 store 调用的跨 store 归属路由；不改产品语义、不动 VM 轨、不改单 store widget 行为。是 BLOG-004 §10 登记的前置（其 T-05 关闭条件）。

## 1. 目标

vue 轨对多 store widget 的裸 `store.Method()` 按方法归属 emit 对应 facade 变量（如 `articleStore.LoadFeed()`）；归属歧义/未知保持现状（首 store）；单 store widget 发射逐字节不变。消费关闭条件：重建 auto-ui-tool 后重生成 Blog，`python tests/dual/verify_dual.py --profile vue-rust` EXIT=0（自动保存 + 刷新重开转绿）。

非目标：App.vue mounted 的 `token` null 异常（属 auth init 时序，若修复路由后仍复现再评估）；VM 轨；多 store widget 的 TS 类型强化；ark/jet 轨。

## 2. 架构方案

语言合同（article_store.at 注释）：`LoadFeed` 特意取专名使裸 `store.LoadFeed()` 无歧义解析到 ArticleStore——即跨 store 按方法名解析是既定语义，vue 轨未实现。VM 轨运行时合并命名空间故未暴露。

数据流：项目级驱动聚合各 store 的暴露面（`AuraStore.handlers` 键集）→ 经 `ComponentGenOptions` 传入 widget 生成 → `VueGenerator.handler_ts_ctx` 构建方法→facade 路由表（仅当 store_deps>1；归属唯一且非首 store 才路由，歧义/未知回落首 store）→ `ts_adapter` 裸 store 调用头按路由表改发 facade 变量。

触点：
1. `auto-ui-compiler/ui_gen/api.rs`：`ComponentGenOptions.store_surface` + 传入 generator。
2. `auto-ui-compiler/ui_gen/vue.rs`：`with_store_surface` + handler_ts_ctx 路由表。
3. `auto-ui-compiler/ui_gen/ts_adapter.rs`：裸 store 调用头路由。
4. `auto-ui-tools/shadcn_build.rs`：ui_build_* 透传 surface。
5. `auto-ui-tools/vue.rs`：from_workspace_reads 预扫聚合各 .at 的 StoreDecl handlers 面，实扫时传 opts（含 regenerate_source_files 路径）。

## 3. 技术栈

现有 auto-ui-compiler（vue.rs/ts_adapter.rs/api.rs）+ auto-ui-tools 驱动。测试沿既有 vue.rs 内嵌测试模式（parse_ui → generate → assert SFC 内容）。

## 4. 需求分析与背景调查

证据链（2026-10-29，BLOG-004 会话）：
- 生成的 Blog `gen/front/vue/src/App.vue`：`const store = reactive(useAuthStore())`；mounted 调 `store.LoadFeed()` → PAGEERROR `store.LoadFeed is not a function`；另有 `token` null PAGEERROR（auth 时序，本计划只观测不修）。
- author 实例探针：`isMounted:true, m:2`（钩子已注册未触发）→ post-flush 队列中断结论。
- NET 捕获：页面加载无初始 `GET /api/drafts`（LoadList 未跑）；15s 零 tick PUT；手动 `store.AutosaveTick()` 立即 PUT。
- article_store.at:45-47 注释明示 LoadFeed 专名设计意图。
- vue.rs:31895 既有注："多 store 场景留待命名 facade 上游项"——即本计划。

## 5. 详细设计

### 解析规则

`routing(method)`：对 widget 的 store_deps = [D0, D1..Dn]，surface(S) = handlers ∪ msg 集：
- method ∈ surface(Di) 唯一归属且 Di ≠ D0 → emit `facade_var(Di).method`
- 其余（歧义、仅 D0、未知）→ emit `store.method`（现状）
surface 缺省（None，如旧调用方）→ 全部现状。单 store widget 不建路由（零变化）。

### 规范增量

| delta_id | action | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/vue-bare-store-routing.md（拟新增） | 无约定 → vue 轨裸 store.Method 解析规则与边界 | BLOG-004 类消费者可复验 | AC-01..03 |

## 6. 测试设计

| case_id | owner | 输入与入口 | 判据 |
|---|---|---|---|
| V-01 | T-01 | 三 store fixture widget（Auth/Article，handler 裸调 LoadFeed/Init/Logout）经 with_store_surface 生成 | SFC 含 `articleStore.LoadFeed()`；`store.Init()` 不变（歧义/唯一归属非首才路由）；单 store 无路由时逐字节现状 |
| V-02 | T-01 | surface=None 调用方 | 发射与现状一致（回归守卫） |
| V-03 | T-02 | 重建 auto-ui-tool → 重生成 Blog → BLOG-004 `verify_dual.py --profile vue-rust` | EXIT=0：App.vue 无 LoadFeed 抛错、author mounted 生效（初始 GET 出现、tick PUT 出现）、自动保存+刷新重开转绿 |

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 裸 store.Method 跨 store 路由实现 + V-01/V-02 通过（编译级证据） |
| AC-02 | work | 消费者闭环 V-03：Blog vue-rust profile EXIT=0（BLOG-004 T-05 全绿） |
| AC-03 | work | 单 store/无 surface 路径零变化（回归守卫通过） |
| AC-04 | review | 固定实现复验 V-01..03 与 SD-01，无越界改动 |
| AC-05 | merge | 本仓门 + Spec/索引/归档 + 重建交付 pin 交 BLOG-004 登记 |

## 8. 执行步骤

- [x] T-01（owner_stage=work；依赖无；AC-01/03）：实现触点 1..5 + V-01/V-02 单元测试，提交。证据：提交 `86a1006`（worktree plan-010-dev，rebase 到 master 后 ff 合并）；`cargo test -p auto-ui-compiler --lib test_plan010` 1 passed；store 族 31 passed 回归守卫（含既有 A1 qualified-call 测试不变）。实施：ComponentGenOptions.store_surface → VueGenerator.with_store_surface → handler_ts_ctx 路由表（唯一归属 deps[1..] 且首 store 未声明才路由）→ ts_adapter 裸 `store` 头改发 facade；tools 驱动聚合各 .at 的 StoreDecl handlers 面（解析失败静默跳过=现状）。
- [x] T-02（owner_stage=work；依赖T-01；AC-02）：重建 auto-ui-tool，Blog 重生成，BLOG-004 vue-rust profile EXIT=0；提交消费证据回执。证据：合并后重建双二进制（见 §9 merge 回执）；Blog `verify_dual.py --profile vue-rust` EXIT=0（BLOG-004 T-05 全绿，报告 attachments/004/dual/）。
- [x] T-03（owner_stage=review；依赖T-02；AC-04）：同树最终复审。证据：§9 review 回执。
- [x] T-04（owner_stage=merge；依赖T-03；AC-05）：门/Spec/归档/交付登记。证据：§9 merge 回执。

## 9. 复审记录

stage: work | plan_id: PLAN-010 | plan_revision: 1 | outcome: pass（T-01 完成） | worktree_path: D:/autostack/.wt/ui-010/auto-ui | branch: plan-010-dev | base_commit: 6e9716b（master，rebase 前 43d9b3a——rebase 期间另一会话推进 master 3 个 docs 提交，无重叠） | code_commit: 86a1006 | dependency_revisions: auto-lang 09bb4148（组内兄弟树 detached）、auto-down c3464d0（组内兄弟树 detached） | task_ids: T-01 | evidence: 见 T-01 行；先修发射缺 `.method` 的笔误（单测红→绿） | blockers: 无 | next_responsible_role: review

stage: review | review_scope: final | reviewer_identity: 同会话执行上下文（非独立角色，从工件重建判定） | plan_id: PLAN-010 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-010/auto-ui | branch: plan-010-dev | reviewed_commit: 86a1006 | base_commit: 6e9716b | acceptance_results: AC-01 pass（路由规则=唯一归属 deps[1..] 且首 store 未声明；单测断言路由/保持/回归三面）；AC-03 pass（surface 缺省回归守卫 + 单 store 不建路由 + 既有 31 store 族测试全绿）；AC-04 复核：裸头才路由（限定头显式意图不动）、调用方全部更新（3 处）、编译 0 error | findings: 边界记录——(1) 驱动侧 store 文件解析失败静默跳过路由（回落现状，SD-01 已登记）；(2) auth `token` null PAGEERROR 不在范围（§10）；(3) 消费者闭环（AC-02）需合并后重建 CLI+tool 双二进制执行，见 merge 回执 | spec_inputs: docs/specs/runtime/vue-bare-store-routing.md（SD-01 新增） | next: merge（T-04）

stage: merge | plan_id: PLAN-010 | plan_revision: 1 | outcome: pass | prepared: 86a1006@6e9716b、SD-01 spec | landed: master 尖峰=86a1006（`git merge --ff-only plan-010-dev`，rebase 后线性） | spec_refreshed: docs/specs/runtime/vue-bare-store-routing.md 新增 | archived: docs/plans/archived/010-vue-bare-store-method-routing.md（status: archived 4/4 tasks） | cleaned: 组内 worktree ui-010 三树 guard 后移除 | completion_kind: delivered | evidence: 消费者闭环（AC-02）——合并后重建 auto-ui-tool（master）与 auto-man CLI（链接 master auto-ui-compiler 的 shell-pack 闭包），Blog 重生成 App.vue 含 `articleStore.LoadFeed()`，`verify_dual.py --profile vue-rust` EXIT=0（15 场景全绿：自动保存/刷新重开/重启重开转绿）；原始报告 auto-blog docs/plans/attachments/004/dual/ | next: 归档提交后清理；BLOG-004 §11 登记消费

## 10. 待澄清事项

无——用户 2026-10-29 已明确要求交付本修复（BLOG-004 续跑授权）。若 V-03 仍红于 auth `token` null 时序，按边界升级 BLOG-004 评估，不在本计划扩scope。
