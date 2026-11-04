---
plan_id: PLAN-051
status: drafting
feature_name: 店工厂结构默认值/main.ts 拼接/VM 新建草稿三回归收口（plan-041 窗口）
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 7
total_steps: 4
depends_on: []
supersedes_spec_components: []
new_spec_components: ["docs/specs/ui/vue-store-generation.md"]
touched_goals: []
---

# PLAN-051：店工厂结构默认值 / main.ts 拼接 / VM 新建草稿三回归收口（plan-041 窗口）

## 0. 变更摘要

修 plan-041 窗口引入/暴露的三处回归（消费方 auto-blog BLOG-005 实录，用户 2026-11-04 深夜授权路线①同构介入）：
①**店模型结构字面量默认值丢失**——`store_init_to_js` 无 `Expr::Node` 臂，`var me User = User{...}` 落 `_ => "null"` → 生成 `ref<any>(null)`，Init 首读字段即 `null.token` pageerror（vue 臂实录）；
②**main.ts import 拼接缺换行**——`{router_import}{store_ctx_import}` 相邻且后者无前导 `\n` → `from './router'import {...}` 语法错，冷 scaffold（write_project_files 全量写 main.ts）必坏（B-01 阻塞实录）；
③**VM 场景「新建草稿」不跳转**——原生臂按下后视图不切换（auto-blog native 回放三轮 deterministic 实录，快照仍列表页），根因待有界调查（③a）后修复或具名拆分。

## 1. 目标

- ②店模型结构字面量默认值按 ts_adapter 既有语义（`Type{f: v}` → 对象字面量，ts_adapter.rs:2247 先例）发射，`ref<T>({...})`；loop 节点保持 delegate。既有 scalar/map/array/`*.new` 行为零变化。
- ①main.ts 两 import 间恒有换行；有/无路由、有/无店四组合输出均合法 TS。
- ③(a) 有界调查产出根因决策稿；(b) 修复使 VM 场景新建草稿进入编辑视图，或具名拆分（须给消费方可见的验收路径）。
- 消费指针：auto-blog BLOG-005（冷 scaffold B-01 + native 回放 22/22 + vue-dev-rust 16/16 复跑）在其簿记验收。

非目标：不改 VM 字节语义/消息分发合同；不改 handler 内 Expr::Node 语义（ts_adapter 既有行为不动）；不动 PLAN-050 的 optimizeDeps/strictPort 发射；主题迁移线（plan-046）不在内。

## 2. 架构方案

- ②`auto-ui-compiler/src/ui_gen/vue/store.rs::store_init_to_js` 增 `Expr::Node` 臂：`node.name == "loop"` → 现有 delegate 行为（返回 null 并注释，保持保守）；否则按 `node.args.args`（Pair/Pos/Name 三形态）递归 `store_init_to_js` 发射 `{ k: v, ... }`——与 ts_adapter.rs:2254-2280 的 handler 侧先例逐字同构。
- ①`auto-ui-tools/src/vue.rs:1562` `store_ctx_import` 前导补 `\n`（或 1566 format 串内加分隔）——单字符级。
- ③先查：VM 场景（run_vm 场景生成/消息分发）在 press→msg dispatch→view state 翻转链上的回归点；候选=plan-041 店工厂 VM 侧同族（VM 店模型初始化）、plan-044 诊断注入改变了 press 分发路径。有界调查（T-03）决策稿定位后最小修复。

## 3. 技术栈

Rust（auto-ui-compiler / auto-ui-tools）；cargo t 固定组（`tu`=ui_gen 单进程档、lib 面）；VM 调查用既有 native 驱动（消费方 harness 复用）。

## 4. 需求分析与背景调查

- 观察事实（2026-11-04）：
  - `store_init_to_js`（store.rs:1181-1254）臂面：Int/Unary/Float/Str/Bool/Array/Object/`*.new`/Nil + `_ => null`；无 `Expr::Node`。plan-043 注释自认「Struct-literal `Type{...}` (Expr::Node) … fall back to null」。
  - ts_adapter.rs:2254 已有 `Expr::Node` → 对象字面量的 handler 侧实现（loop 例外），可同构复用。
  - vue.rs:1566 format：`{base}{i18n_setup}{router_import}{store_ctx_import}\nconst app...`；`router_import`="\\nimport router from './router'"（1547，无尾换行）、`store_ctx_import`="import { installAutoStoreContext } …\\n"（1562，无前导换行）→ 拼接即语法错。实证：auto-blog 冷 scaffold main.ts:26（`from './router'import`），esbuild transform 失败页面 500。
  - VM：auto-blog native 回放 22/22（11-01 工具 `4d2198b8`=d43425c 窗口）→ 现 master 工具 `81116bab` 三轮 deterministic「新建草稿后视图不切换」（快照恒列表页）；消费方 A/B 已排除 app 侧（checkout 掉全部 app 改动同败）。
- 授权：用户 2026-11-04 深夜于 auto-blog BLOG-005 goal round-6 拍板路线①（同构）：本仓新建最小计划修三处并交付，消费方随后收口。
- 冻结观察基线：

| 源文件 | SHA256（完整值） |
|---|---|
| `crates/auto-ui-compiler/src/ui_gen/vue/store.rs` | work 首日实测记录 |
| `crates/auto-ui-tools/src/vue.rs` | work 首日实测记录 |

## 5. 详细设计

1. **T-01（②）**：`store_init_to_js` 增 `Expr::Node` 臂（见 §2）；单测=含结构字面量默认的店模型生成 `ref<any>({ id: 0, ... })`（字段序保持）；既有 scalar/map/new 用例不回归。
2. **T-02（①）**：`store_ctx_import` 前导 `\n`；单测=四组合（有/无路由 × 有/无店）main.ts 均过 esbuild 解析（或字符串级断言相邻 import 间有 `\n`）。
3. **T-03（③a 有界调查）**：决策稿（scratch `D:/autostack/plan-artifacts/plan-051/`）——用 auto-blog native 驱动最小复现（press→snapshot），定位 press→msg→view 链断点（候选：VM 店模型初始化/plan-044 分发路径）；产出=根因+最小修复设计或具名拆分决定。
4. **T-04（③b）**：按 T-03 结论修复 + VM 面验证（消费方 native 回放作为跨仓 oracle 指针）。
5. **T-05**：受影响组回归（`tu` + lib 面）+ 全部新测试绿。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/vue-store-generation.md | 店模型初始化器按值发射（结构字面量→对象字面量，不再 null 回退）；main.ts 相邻 import 恒分隔 | plan-041 窗口两回归的合同化（复审 F-01/SD 确认新建） | AC-01/02 |
| SD-02 | modify | docs/specs/runtime/component-and-store-instances.md | C-02 迟到事件语义修订：MCP/派发面 identity miss 先件名存活解析（恰一存活重定向；0 丢/≥2 丢）；进程内面保持严格 | 视图携页名 vs 键控身份格式脱 → 合法 press 被误丢（复审 F-01） | AC-03 |

## 6. 测试设计

最终 review 由独立上下文在同一 worktree 核对。

| Case ID | AC / invariant | 输入/正式入口 | 区分性 oracle / 预期 | 边界 |
|---|---|---|---|---|
| B-01 | AC-01 | 店模型 `var u User = User{...}` | `ref<...>({ ... })` 含全部字段，字段序稳定 | loop 节点保守不变 |
| B-02 | AC-01 | scalar/map/`*.new`/数组初值 | 输出与改前逐字一致 | 零回归 |
| B-03 | AC-02 | 有路由+有店（auto-blog 形） | `'/router'`与`'import {`之间有 `\n` | 四组合 |
| B-04 | AC-03 | VM press 新建草稿（调查后） | 编辑视图出现（或拆分具名+消费方路径） | — |
| B-05 | AC-04 | `tu` + lib 组 | 零新增失败 | 对照冻结基线 |

## 7. 验收标准

- AC-01（work）：B-01/B-02 过——店模型结构默认值保真，既有形态零回归。
- AC-02（work）：B-03 过——main.ts 四组合合法。
- AC-03（work）：③a 决策稿有具名结论；③b 修复过 B-04（或拆分决定获消费方认可路径）。
- AC-04（review）：独立上下文核对 + `tu`/lib 零新增失败。
- AC-05（merge）：SD-01 落地；消费指针（BLOG-005 B-01 冷 + native 回放）在其簿记。

## 8. 执行步骤

- [x] T-01（owner_stage=work；AC-01）：store_init_to_js Node 臂 + 单测。[✅ 2026-11-04] Expr::Node→对象字面量（Pair/Pos/Name 三形态，loop 保守 null，ts_adapter:2254 同构）；oracle test_plan051_store_struct_default_kept（ref<any>({ id: 0, ... }) 保真）+ non_struct_defaults_unchanged 过。
- [x] T-02（owner_stage=work；AC-02）：main.ts 分隔修复 + 四组合单测。[✅ 2026-11-04] store_ctx_import 前导 
；oracle main_ts_imports_newline_separated_four_combinations 过（四组合无拼接 import）。
- [x] T-03（owner_stage=work；AC-03）：③a 有界调查决策稿。[✅ 2026-11-04] 根因=VM 视图 `DynamicMessage::TypedInst.instance` 携页名（`author`）而 `child_instances` 键=plan-041 T-04 键控身份（路径/#N 形）→格式脱致合法 press 被 late-event 纪律误丢；消费方实录 `[VM-INSTANCE] author.GoNew dropped: instance released (late event)`（native-first.log；三轮 deterministic）。证据链=mcp_server.rs:3928 TypedInst 编码、dynamic.rs:2928 丢弃点、vm_bridge.rs:2131 键查找。
- [x] T-04（owner_stage=work；依赖T-03；AC-03）：③b 修复（或拆分记录）。[✅ 2026-11-04] dynamic.rs dispatch：身份查找 miss→name-only 存活解析重定向（恰一存活；0/≥2 保持丢弃=真迟到纪律保留）；下游 input back-write 随解析后实例。runtime lib 2210/2220（10 败全部 base 同败，stash A/B 两批实证）；消费方 native 回放=跨仓 oracle 指针（merge 后 BLOG-005 执行）。
- [x] T-05（owner_stage=work；依赖T-01/02/04；AC-04）：组回归 + 新测试。[✅ 2026-11-04] compiler lib 268/268、tools lib 205/205（5 既有基线红同集）、runtime lib 2210/2220（10 既有基线红同集）。
- [ ] T-06（owner_stage=review；依赖T-05；AC-04）：同树独立上下文最终复审。
- [x] T-07（owner_stage=merge；依赖T-06 pass；AC-05）：SD-01 落地 + 归档清理。（landing 见 merge receipt）

## 9. 复审记录

- stage: new | plan_id: PLAN-051 | plan_revision: r1 | outcome: pass（授权已记录 §4：消费方用户 2026-11-04 深夜路线①同构） | next: work（T-01/T-02 立即开工；T-03 调查并行）。

stage: review | review_scope: final | reviewer_identity: 独立 general-purpose 子代理上下文（与实现无共享记忆） | plan_id: PLAN-051 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-051/auto-ui | branch: codex/plan-051-dev | reviewed_commit: 79e1057e | base_commit: 4e5e6796 | spec_inputs: SD-01 新建确认 + SD-02 C-02 修订（F-01 merge 义务，已在 delivery descendant 4102d8aa 落地） | acceptance_results: AC-01 PASS（闭环探针 ref<any>({...}) 保真）、AC-02 PASS（四组合真链路探针，字节级换行复核）、AC-03 PASS（press 链静态实证+契约注释对照；集成 oracle=消费方 native 回放）、AC-04 PASS（compiler 268/268、tools 205/205、runtime 2212/2220——败集均 base 同败 stash 实证，含 reviewer 自发现的 t3_examples_queue_end_to_end 同败） | findings: F-01（中，SD 补齐→已落地 4102d8aa）、F-02/F-03/F-04（低；F-03 注释已刷新、F-04 决策稿内容已内联 Plan）、F-06/F-07 信息 | evidence: D:/autostack/plan-artifacts/plan-051/review/final-review.md | pending_closeout_gates: merge 所有（landing、归档、清理） | next: merge

stage: work | plan_id: PLAN-051 | plan_revision: r1 | outcome: pass（T-01..T-05 完成，execution_done 待 T-06 复审） | worktree_path: D:/autostack/.wt/ui-051/auto-ui | branch: codex/plan-051-dev | base_commit: 4e5e6796 | code_commit: 79e1057e（T-01/T-02=b8e23710，T-03/T-04=79e1057e） | dependency_revisions: auto-lang da4c98b51（master detached 同级）、auto-web 37d32c2、auto-down 7c4d5cb | task_ids: T-01..T-05（T-06=review 所有） | evidence: 新 oracle 三枚全绿（struct_default_kept/non_struct_defaults_unchanged/main_ts_imports_newline_separated_four_combinations）；组回归 compiler 268/268、tools 205/205、runtime 2210/2220（15 败全部 base 同败 stash A/B 两批实证）；③根因与修复见 T-03/T-04 行 | blockers: 无 | next_responsible_role: review（T-06 同树独立上下文最终复审；消费方 oracle=native 回放 22/22+B-01 冷 scaffold 在 BLOG-005 簿记执行）

## 10. 待澄清事项

- ③根因未定位（T-03 有界调查产出）；若调查指向超出本仓（如 auto-lang VM 指令层），具名拆分并回告消费方。
- worktree：`D:/autostack/.wt/ui-051/auto-ui`，branch `codex/plan-051-dev`（含 auto-lang/auto-web/auto-down 同级，按需 detached）。
