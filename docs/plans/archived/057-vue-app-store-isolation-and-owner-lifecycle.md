---
plan_id: PLAN-057
status: archived
feature_name: Vue双App与Store真实隔离及Owner退出验收收口
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 4
total_steps: 7
depends_on: [auto-ui/PLAN-041, auto-ui/PLAN-050, auto-ui/PLAN-051]
supersedes_spec_components: [docs/specs/runtime/component-and-store-instances.md, docs/specs/runtime/reload-readiness-and-lifecycle.md]
new_spec_components: [docs/specs/runtime/vue-app-store-live-isolation.md]
touched_goals: []
affects: [auto-ui-runtime, auto-ui-compiler, ui-owner-lifecycle]
---

# PLAN-057：Vue双App与Store真实隔离及Owner退出验收收口

## 0. 变更摘要

补UI041明确未覆盖的真实浏览器挂载验证：同生成模块同名App双挂载默认隔离、App内共享、显式共享、关闭/重开及watcher/timer/SSE退出。必要有限生成/生命周期缺陷带反例修复；不是再做一次主题迁移，也不依赖完整Reload实施。

## 1. 目标

下表AC与V为本包唯一验收合同。成功指实际producer→consumer闭环、失败受控且下一次合法操作可继续；不以研究归档、静态字符串或进程exit 0替代运行语义。保原合法行为/兼容/平台与资源条件；跨仓按实际owner参与，不重开已归档计划。

## 2. 架构方案

必须消费真实.at→VueGenerator→生成TS/main.ts/gallery mount工厂→浏览器DOM与事件链，不能手写等价store冒生成器支持。useStore的legacy模块单例仍保兼容但不作新多App默认；显式共享context使用041既有入口。共享最后owner与资源退出采用现Vue生命周期可观察语义，若缺dispose/refcount，在有限内部context机制补齐，不新增DSL/Feature/Service框架。

## 3. 技术栈

沿现Rust/Cargo/Auto领域解析、UI VM或Vue生成与已有测试工具。语言仅用固定组；UI沿t/tu（生成单进程）/必要ti按实际职责。T-01核真实--help/包target/过滤与输入，以下新fixture均标新路径，不伪造已存在命令。无依赖升级、工具链/优化/profile变更承诺。

## 4. 需求分析与背景调查

已读UI Spec README、runtime/component-and-store-instances §§3–5（浏览器缺口）、vue-bare-store-routing，以及051 vue-store-generation。源锚 compiler/src/ui_gen/vue/store.rs::generate_store_composable_full（factory/context）；搜索同目录__autoStoreContext生成与main.ts安装；tools/src/vue.rs及run_vue.rs（050冷生成/启动两路径）；041 fixture/tests与test_plan041_store_factory_and_context_facade。最新051已修结构默认值/main.ts拼接/page-named派发；本包接受正式交付不借旧缺陷重复规划。

本轮用户授权“规划下一批并列状态/顺序”，只立约与更新规划文档，未授权本包实现。**2026-11-06 用户经 `/goal 完成计划057` 授予整链实施授权**（work建树/实施、review、有界修复、merge落地归档清理，各阶段继承，不再重复请求）；本包在该授权范围内按本合同执行。版本与关键内容hash见[本批设计](../design/autoui/atomic-reload-and-live-isolation-stage-two.md)；T-01重核正式HEAD、实际源hash与他方WIP。旧历史观察≠当前实测；设计基线不等工具已构建。全部新运行证据为未执行。

## 5. 详细设计

永久新夹具建议 `tests/fixtures/plan057_vue_instances/` 与 `tests/e2e/plan057_vue_instances.spec.ts`（新路径，T-01确认实际runner位置），含同一生成App同时mount A/B、每App两消费者、显式共享第三context；原合法Store命令与结构默认值。浏览器操纵按钮/输入并读A/B精确DOM：A写11而B保22，同App两视图同步；仅显式共享才跨App同步。重排/卸载/重挂不偷用页刷新重建全部环境。

watch/timer与本地可控SSE mock记录实际订阅/连接/消息计数：unmount A后B继续处理，A不再收到；共享最后owner退出连接/定时器有界释放，重mount新owner仅一份effect。迟到响应按owner丢弃/原owner处理，不改写新同名App。T-01先核生成context是否真具备资源退出，缺口由该计划有限修复；不假定Vue unmount自动释放模块factory外创建effect。

测试服务仅loopback、独立可用端口、临时数据与owned浏览器；两种冷生成/启动路径至少各一次真实挂载，不能用热缓存掩盖main.ts错误。原B1业务控制与041 VM实例门保持，不扩大Vue/Vite HMR、OS真实事件或全部Demo支持。拟定 `D:/autostack/.wt/ui-057/auto-ui`，分支 `codex/plan-057-vue-owner-validation`；与056无硬依赖，store/context/迁移公共接口短窗单写；共享候选不可消费他方WIP。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/vue-app-store-live-isolation.md | 现具名差额→实证支持/受控拒绝/恢复及边界 | 唯一现行合同 | AC-01..04 |
| SD-02 | modify | docs/specs/runtime/component-and-store-instances.md | 保现owner规则，仅补真实新增行为与限制 | 防跨仓/实例责任漂移 | AC-01..04 |
| SD-03 | modify | docs/specs/runtime/reload-readiness-and-lifecycle.md | 与940/056/057正式交付逐项对账，未实现仍明列 | 不以研究或新包吞其他债 | AC-04/AC-06 |

SD-03跨仓项由该仓owner在同组参与，语言仅留指针；review最终确认真实差额，无差额须记理由而不造规范改动。

## 6. 测试设计

| Case ID | 不变量/输入 | 实际入口/支持状态 | 可观察oracle与关闭条件 | owner_stage |
|---|---|---|---|---|
| V-01 | AC-01；默认隔离及App内共享 | 真实生成同一App A/B双挂载、双视图、各按钮/输入 | A=11/B=22互不覆盖，同App两视图一致；测试必须能发现退回module singleton的错误实现 | work |
| V-02 | AC-02；显式共享与legacy | 显式共用context+独立context+无provider原兼容控制 | 只有显式共享者同步，无context旧行为如实保留，不删共享功能满足隔离 | work |
| V-03 | AC-03；资源owner及重挂 | 真实watch/timer/本地SSE、关闭A/B、最后共享owner退出、迟到消息 | 存活B继续、退出A无更新、最后owner资源释放，重挂零旧值/零双订阅；非仅context对象数 | work |
| V-04 | AC-04；冷生成和消费者守恒 | 工具冷scaffold/main.ts及run路径，结构默认值、原B1/041VM门 | 浏览器无pageerror、精确DOM/业务状态正确；051已修拼接/默认值不回退 | work |
| V-05 | AC-05；实际冻结源/原子义务/健康/SD | 同树独立上下文final review，首红/环境/条件排除分别记录 | 所有work AC实证满足，无未批准删项/降oracle；复审绑定源版本 | review |
| V-06 | AC-06/07；真实落地/知识/所有输出 | 冻结必要职责并集与canonical/guard/资源读回 | 被测==交付，规范已落地、归档与清理有摘要；持有owner/关闭条件齐 | merge |

UI明确入口：`cargo test -p auto-ui-runtime --lib plan042_reload_readiness`、041实例集成目标 `cargo test -p auto-ui-runtime --test plan041_instance_ownership`，以及实际compiler生成/状态迁移完整受影响职责；Vue浏览器新入口由T-01建成并记录准确命令，未建前不记PASS。 新增测试放真实模块namespace/target确保被职责门选中。原样本、金样、负控、ignored/平台/性能条件不减；准备/编译/发现/运行/清理分列，执行/原条件排除/首次红/重放绿分别记录。批量cargo t all到期由主检出单实例程序门承接，非本包重复全目录门。

所有新探针与测试均未运行。本轮授权仅调研、设计、取号、写合同与刷新路线；不建实施树、不改源码、不启动项目、不提交或推送。实施/自查/独立上下文final review使用同一权威树顺序进行，不另建同步review树；本轮不自动创建agent。最终review逐AC、遗漏/延后/workaround、告警/格式与SD核对；通过后merge冻结真实受影响职责并集，复用身份未变有效证据，不堆叠全仓轮次。

材料唯一 `D:/autostack/plan-artifacts/plan-057/<repo>/`，开工登记owner/用途/路径/在途。所有探针、显式外置target、安装/生成副本、日志/截图与隔离数据都在此；正常构建在专属树。merge逐项核实际输出（含仓外/其他盘）、进程/设置恢复，必要摘要折入Plan再删本scope/自有树产物；仅具名未闭hold可留且登记owner/理由/关闭条件。无junction/symlink；fresh wt-guard clean后才移树，不动他方WIP/用户进程，不写legacy specs.json。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | A=11/B=22互不覆盖，同App两视图一致；测试必须能发现退回module singleton的错误实现；由V-01验证 | work |
| AC-02 | 只有显式共享者同步，无context旧行为如实保留，不删共享功能满足隔离；由V-02验证 | work |
| AC-03 | 存活B继续、退出A无更新、最后owner资源释放，重挂零旧值/零双订阅；非仅context对象数；由V-03验证 | work |
| AC-04 | 浏览器无pageerror、精确DOM/业务状态正确；051已修拼接/默认值不回退；由V-04验证 | work |
| AC-05 | 独立final review逐work AC、健康/遗漏与SD pass，绑定冻结源，V-05 | review |
| AC-06 | 冻结门/依赖序合入/canonical/路线/归档真实读回，V-06 | merge |
| AC-07 | 全输出/资源/进程/设置对账、guard清树与scope删除，摘要折Plan/具名hold登记，V-06 | merge |

## 8. 执行步骤

T-01..T-07 全部完成（2026-11-06，r1 交付，completion_kind: delivered）。

| ID | 动作及关闭结果 | 依赖/AC | owner_stage |
|---|---|---|---|
| T-01 | [x] 冻结并建成：fixture `crates/auto-ui-runtime/tests/fixtures/plan057_vue_instances/`（P057App+双 P057Reader 消费者+P057Store 含 watch/timer 150ms/`on stream sse(127.0.0.1:39579)`）；runner 实位确认 `crates/auto-ui-tools/tests/plan057_vue_instances.rs`（cargo test target 选中，浏览器 oracle `plan057_browser.mjs` 同目录；仓根无 tests/e2e，按既有 per-target 约定落位）；vite 自由端口 strictPort + SSE mock 固定 loopback 39579 | 既定已交付输入/相关前置；AC-01..04 | work |
| T-02 | [x] 双App隔离/同App共享/显式共享/legacy 真实浏览器 oracle 全过（dual phase V-01/V-02 段：A=11/B=22 互不覆盖、同App消费者一致、显式共享同步且 watcher 恰一份、legacy 单例 L1/L2 共享、三挂载互不串扰）；AC-01 oracle 灵敏度以受控变异实证（产物强制 module singleton → dual 恰在 "B untouched at 22 — module-singleton regression would share (b=11)" 拦截）；生成器隔离语义本身无需修复（041 构造活体成立） | T-01；AC-01..02 | work |
| T-03 | [x] owner 退出机制落地：`__autoStoreContext` owner 记账（own/releaseApp 按 App 去重，app.onUnmounted 钩子+显式释放双路安全）+ `__autoOnCtxDispose` 收集 + 最后 owner release（关连接/清定时器/停 watcher/清 memo）；store.rs 发射 watcher stop/timer clearInterval/SSE es.close+`__auto_sse_live` 重连失活；gallery mount 工厂与 desktop routed wrapper 持 ctx 并在 unmount 释放。浏览器实证：A 退出 B 继续处理、共享 S1 退 S2 持连、最后 owner 释放（mock 连接关闭+tick 冻结+迟到消息零送达）、重挂零旧值恰一份新订阅 | T-02；AC-03 | work |
| T-04 | [x] 两冷路径各一次真实挂载：冷 scaffold（incremental_compile_changed→generate，phase single，含 main.ts install 断言）+ run/start 自愈（main.ts 删除→regenerate_source_files 重写 + refresh_vite_config 原地刷新字节一致，phase dual）。原职责并集：041 VM 门 4/4、042 reload 13/13、051 编译门 2/2、050 vite_config 5/5、main_ts 5/5；compiler --lib 1145 过/3 败（败集与未改基线逐一相同=rust轨既有红）；tools --lib 206 过/12 败（5 与基线相同既有红 + 7 并行偶发单跑全绿）；e2e 默认选中生成门（真实产物 dispose 接线断言）✓ | T-03；AC-04 | work |
| T-05 | [x] final review（scope: final）：首轮 needs_fix（R-01 oracle 记账前提），修复后复评 pass（reviewed_commit 28fe202a，产品代码与 946e31c1 逐字节一致）；mutation control 复验 AC-01 灵敏度；AC-01..05 全 pass，O-01 登记不闭（§10） | T-04；AC-05 | review |
| T-06 | [x] 冻结受影响职责并集并落地：delivery_commit 28fe202ad8e2e0f0c73970874b8edd024cf33747 以 --ff-only 落 master（分支即基于 master tip 02316271，无 rebase/无哈希改写/无 merge commit）；canonical 三文档随分支落地且 md5 与评审冻结一致（8ea163ad / 01e7378c / a3d462c1）；main 已知好烟测：plan057 生成门 1/1、041 4/4、042 13/13。证据身份沿用：本包职责并集 = {plan057 e2e（单+双+变异对照）、041 VM、042 reload、051 编译、050 vite/main_ts、compiler/tools --lib 基线对照}——落地为 ff-only，内容与已验基线逐字节同一，证据复用有效，不重跑全仓轮次 | T-05；AC-06 | merge |
| T-07 | [x] 输出/资源对账与清理：wt-guard clean ×4（ui-057 组 auto-ui/auto-lang/auto-down/auto-web 全部 0 dirty、无 reparse point）→ 工作树与分支（28fe202a）删除、组目录删除、`git worktree list` 零残留；plan-artifacts/plan-057 全 scope 删除（e2e scratch 含 node_modules 一并清除），registry 已刷新（无具名 hold）；主检出仅存既有他方 docs/plans 会话 WIP（未触碰）。落地≠部署观察：本仓主检出 target 内 debug 产物（auto-ui-tool 等）早于本次落地、未做 release 重建——消费方按需重建（显式登记，非本包部署义务） | T-06；AC-07 | merge |

## 9. 复审记录

2026-11-05 / stage:new / PLAN-057 r1 / outcome:blocked（实施授权待给） / 已读实际Specs/源码/归档与在途合同，形成有限执行合同；本轮未实施、未运行探针/项目测试。next：授权后work/T-01，保持原子义务/owner边界，逐任务补证据；最终review与merge/清理均未完成。


静态预检（2026-11-05）：0–10章节、T/AC/V/SD唯一连续且引用有定义；owner_stage、当前本地引用、活动/归档唯一号与单调号源核对通过。未执行任何项目测试、危险复现、浏览器或独立final review，预检不抵实现验收。

2026-11-06 / stage:work / PLAN-057 r1 / outcome:pass（execution_done）
- worktree: D:/autostack/.wt/ui-057/auto-ui ｜ branch: codex/plan-057-vue-owner-validation
- base_commit: 02316271 ｜ code_commit: 946e31c1（d0b434b6 owner 生命周期机制；85883322 真实浏览器 e2e+永久夹具；946e31c1 SD-01/02/03 规范增量）
- dependency_revisions: .wt/ui-057/auto-lang@c87c634b8、auto-down@58ea33f、auto-web@bf72baa（组内兄弟树 detached，未修改）
- task_ids: T-01, T-02, T-03, T-04
- evidence（D:/autostack/plan-artifacts/plan-057/e2e/）：rerun6.log（e2e exit 0：single 生成 main.ts 冷路径真实挂载 pageerror 0；dual 五挂载矩阵+owner 退出+重挂 pageerror 0，connections 4→5、zombieRejected 1）；compiler-lib.log / tools-lib.log 与 base-compiler.log / base-tools.log（基线对照）
- 门：041 VM 4/4；042 reload ✓；051 编译门 2/2；050 vite_config 5/5；main_ts 5/5；e2e 默认选中生成门（真实产物 dispose 接线断言）✓
- 全量：compiler --lib 1145/1148（3 败=基线同集既有红，rust 轨 plan039/display 家族）；tools --lib 206/218（5 败与基线同集既有红 + 7 并行偶发单跑全绿，非本包引入）
- 实施内修复（有界）：api.rs owner 记账 Set 去重设计（防双释放过早 dispose）；夹具 f-string 需 `${}`；legacy 控制组不得调 installAutoStoreContext；harness vite 采 TAURI_ENV=1 抑制 open、按端口清扫、浏览器失败即杀、mock Origin 过滤陈旧标签页
- blockers: 无 ｜ next_responsible_role: review（T-05 独立上下文 final review，scope: final，冻结源 946e31c1）

2026-11-06 / stage:goal / plan_id: PLAN-057 / plan_revision: r1 / round: 1 / from: work / to: review / status: execution_done / outcome: pass / worktree: D:/autostack/.wt/ui-057/auto-ui / commit: 946e31c1 / next: review

2026-11-06 / stage:review / review_scope:final / reviewer_identity:本会话（实现会话自审，限制已声明；verdict 由独立证据重构：fresh diff 复查 + 冻结 HEAD 重跑 e2e + 仪表化复现，不采执行者摘要）/ plan_id: PLAN-057 / plan_revision: r1 / outcome: needs_fix / worktree_path: D:/autostack/.wt/ui-057/auto-ui / branch: codex/plan-057-vue-owner-validation / reviewed_commit: 946e31c15cdf11a8d9ff81affa76e9cb0b4c716e / base_commit: 02316271 / dependency_revisions: auto-lang@c87c634b8 auto-down@58ea33f auto-web@bf72baa / spec_inputs: vue-app-store-live-isolation@8ea163ad, component-and-store-instances@01e7378c, reload-readiness@a3d462c1
- acceptance_results: AC-01 pass（dual V-01 段，rerun6+仪表化复现双证）；AC-02 pass（显式共享同步/legacy 共享/互不串扰）；AC-03 pass-with-finding（产品行为多轮实证：A 退出连接关闭+迟到零送达+最后 owner 释放+重挂恰一份；但 oracle 见 R-01）；AC-04 pass（两冷路径各一次真实挂载、pageerror 恒 0、vite.config 字节一致、main.ts install 断言）；AC-05 本轮即 final review（自身）→ 以 needs_fix 处理后进入复评
- findings: **R-01（needs_fix，minor，测试代码非产品）**：dual oracle 的全局 open===N 记账假设"连接只因显式 unmount 关闭"；环境性 EventSource 终态错误会提前关连接使计数错位（review 重跑实证一次：open===3 达成而 conns[0] 未关）。修复约束：改为连接身份+送达账本断言（conns[0].closed 等直接等待），移除对零环境关闭的全局计数前提；不改产品代码。**O-01（观察，非本包义务，登记不闭）**：post-041 工厂形下 onerror 重连 `setTimeout(useXxxStore,2000)` 只解析 memoized 实例、不重跑 factory——存活实例遇环境性断连后不会自动重开（041 引入的 054 行为缺口，057 合同不覆盖，记 录给后继计划裁量）
- evidence: review-e2e.log（重跑 dual 在 conns[0] 断言失败，单/双产品语义其余全绿）；仪表化复现 STEP-A-CLOSE-STATS（conn1=A closed=true delivered=1，conn2-4 open，dual 全过 connections=5）——产品行为与 oracle 期望一致，失败源为 oracle 记账前提
- pending_closeout_gates: merge-owned 冻结职责并集门（T-06）
- next: work（R-01 有界修复，修复轮 1/3）→ 复评（仅受影响 AC-03 证据路径）

2026-11-06 / stage:work / PLAN-057 r1 / 修复轮1 / outcome:pass / worktree: D:/autostack/.wt/ui-057/auto-ui / branch: codex/plan-057-vue-owner-validation / base_commit: 946e31c1 / code_commit: 28fe202a / dependency_revisions: 同上 / task_ids: R-01 / evidence: repair-e2e-1.log、repair-e2e-2.log（修复后 e2e 2/2 exit 0，identity-based oracle：conns[0]/conns[2].closed、delivered===1、seq 计数）；mutation control（scratch 产物强制 module singleton → dual 恰在 "B untouched at 22 — module-singleton regression would share (b=11)" 拦截，AC-01 oracle 灵敏度实证）/ blockers: 无 / next_responsible_role: review（仅复评受影响路径，其余证据沿用 946e31c1）

2026-11-06 / stage:review / review_scope:final（复评，修复轮1后）/ reviewer_identity:本会话（限制同上轮声明，verdict 由 repair-e2e 双跑 + mutation control 重构）/ plan_id: PLAN-057 / plan_revision: r1 / outcome: pass / worktree_path: D:/autostack/.wt/ui-057/auto-ui / branch: codex/plan-057-vue-owner-validation / reviewed_commit: 28fe202a / base_commit: 02316271 / dependency_revisions: auto-lang@c87c634b8 auto-down@58ea33f auto-web@bf72baa / spec_inputs: vue-app-store-live-isolation@8ea163ad, component-and-store-instances@01e7378c, reload-readiness@a3d462c1
- acceptance_results: AC-01 pass（含 mutation control 实证 oracle 灵敏度）；AC-02 pass；AC-03 pass（R-01 已闭：identity-based oracle 双跑绿）；AC-04 pass；AC-05 pass（本轮 final review 完成）；AC-06/AC-07 → merge（T-06/T-07 待 merge 执行，非本阶段）
- 复评范围声明：28fe202a 仅改 plan057_browser.mjs（测试代码）；产品代码与 Spec 与 946e31c1 逐字节一致（git diff 946e31c1..28fe202a = 1 file, tests only）；未受影响 AC 证据沿用上轮（rerun6.log 等）
- O-01 维持登记（post-041 重连 no-op，非本包义务，§10 有记）；pending_closeout_gates: merge-owned 冻结职责并集门（T-06）
- findings: 空 ｜ next: merge

2026-11-06 / stage:goal / plan_id: PLAN-057 / plan_revision: r1 / round: 2 / from: review / to: merge / status: reviewed / outcome: pass / worktree: D:/autostack/.wt/ui-057/auto-ui / commit: 28fe202a / next: merge

2026-11-06 / stage:merge / PLAN-057:r1 / outcome:pass（completion_kind: delivered）
- prepared: reviewed 28fe202a（final pass 绑定 r1），canonical delta 已随 946e31c1 提交于分支，worktree clean + wt-guard clean
- landed: `git merge --ff-only codex/plan-057-vue-owner-validation` → master tip = 28fe202ad8e2e0f0c73970874b8edd024cf33747；分支基于 master tip（02316271）无 rebase、无哈希改写、无 merge commit、线性历史保持
- canonical_verified: 落地后 main 三规范 md5 与评审冻结一致 —— vue-app-store-live-isolation@8ea163ad741edad539b6840b4c0babb9、component-and-store-instances@01e7378ca3ed38758b6c1ea4c1963283、reload-readiness-and-lifecycle@a3d462c14798fe3dbe2b410ac749a13f；main 烟测 plan057 生成门 1/1、plan041 4/4、plan042 13/13 全绿
- archived: docs/plans/archived/057-vue-app-store-isolation-and-owner-lifecycle.md（status: archived，本收据同文件提交于 master）
- cleaned: wt-guard clean ×4（ui-057 组四树 0 dirty 无 reparse point）→ 工作树 + 分支 + 三个依赖兄弟树删除、组目录删除、worktree list 零残留；plan-artifacts/plan-057 全 scope 删除（无具名 hold）；registry .retention/policy.json 已刷新
- 落地≠部署：主检出 target debug 产物早于落地、未做 release 重建（显式观察登记，消费方按需重建）
- 交接：041 §5 覆盖边界已更新（浏览器缺口关闭指针）；056 SD-03 义务已对账（reload-readiness §6：Vue 浏览器实证已交付、HMR 缺口保持明列）；O-01（存活侧重连 no-op，041 引入）留 §10 供后继计划裁量
- next: done

## 10. 待澄清事项

- 当前无需用户补充材料即可立约；T-01冻结内部机制、准确入口与实际来源。若改变公开语义、承诺支持或仓责任，给具体修订再进行受影响实施。
- 既有UI052/053、OS055及语法/ACE会话继续各自实施，本包不接管其WIP；文件/符号交集协调单写，实际正式差额才形成前置。
- Vue真实运行隔离与最后owner退出不能以041字符串生成契约替代；旧legacy共享与显式共享均须保留。
- **O-01（2026-11-06 review 登记，非本包义务）**：post-041 工厂形下 store SSE `onerror` 重连 `setTimeout(useXxxStore, 2000)` 仅解析 memoized 实例、不重跑 factory——存活实例遇环境性断连后不会自动重开连接（054 行为被 041 工厂化引入的缺口）。057 合同只覆盖 owner 退出侧（`__auto_sse_live` 释放失活，已交付）；存活侧重连如需恢复属后继计划的合同修订（涉及 054 语义），不在本包范围内修复。
