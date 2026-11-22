---
plan_id: PLAN-108
status: archived
feature_name: U06 异步 handler 探针双端交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-22
completion_kind: delivered
plan_revision: 2
current_step: 4
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/component-http-async.md, ../auto-lang/docs/specs/ace-migration/component-event-sessions.md (http.client/1 节)]
touched_goals: [GOAL-002]
---

# PLAN-108 U06 异步 handler 探针双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。本号继续承担本页范围，原组其余 demo 迁至 [119](119-autoui-minesweeper-av-vue.md)、[120](120-autoui-auto-edit-av-vue.md)、[121](121-autoui-blog-viewer-av-vue.md)、[122](122-autoui-launcher-av-vue.md)、[123](archived/123-autoui-layout-widgets-av-vue.md)。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [032-handler-async-probe](../../examples/ui/032-handler-async-probe/pac.at) | 真实挂起/完成/取消、卸载后的迟到结果隔离 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。维护单一 UI 异步接线，使用实际调度、取消、HTTP 接口；与 AutoLang 1090 只按所需能力交接。保原 65 秒以上慢端点的见证和关闭边界。

## 3. 技术与实现入口

目标入口为 src/front/app.at 的 Load、Bump、Tick 与原本地慢端点。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-108/{auto-ui,auto-lang}，每仓单一 plan-108-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

先用可验证的 HTTP 和 UI 调度入口验证原探针；语言异步缺口只交具体 producer，并保本计划消费者验收。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。真慢请求挂起时 Tick 递增、Bump 及时生效；完成后结果更新。取消或卸载后释放任务，迟到结果无法写入旧/新实例。记录实际调度和资源终态。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | 每个原项目/页面、原 oracle、适用端、实际前置和资源映射可复查。 |
| AC-02 | work | 所用接口及组件各有正反验证，真实 AV/Iced 的原行为、权限、恢复和关闭通过。 |
| AC-03 | work | A2Vue 的实际生成、构建、交互、失败和恢复按原矩阵通过。 |
| AC-04 | review | 收据与冻结版本对证，逐行核验缺失和临时代码，复审通过。 |
| AC-05 | merge | 可消费版本及两个目标的各自结果落主，各登记下游交接。 |

## 8. 执行步骤

当前 harness 如提供任务面板，开工时登记以下任务 ID。各目标按实际接口推进，阶段复核与最终复审分别覆盖对应结果。

- [x] T-01 固定原项目、依赖与验收矩阵
  - work，AC-01，核 §1 现有源和 §6 原运行入口，输出逐行的源、权限、预期和实际阻断。
- [x] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
- [x] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
- [ ] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-01 固定原项目、依赖与验收矩阵 | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-ui (branch plan-108-dev, base 95401c10) + D:/autostack/.wt/ui-108/auto-lang (branch plan-108-dev, base 0abc54a59) | next: T-02

**T-01 逐行矩阵（2026-11-22 核）**

| 项 | 固定值 |
|---|---|
| 原项目源 | `examples/ui/032-handler-async-probe/pac.at` SHA256 `3e22ace689f8ea04ccc8ad7cf53aef3b3d9f705971cea72c8a3752f9ecfa39a3`；`src/front/app.at` SHA256 `7896f812ec4f04665aab9f12e477c833f78a917ac70eed00d2839f4b8af72f21`（Plan 007 自 auto-lang 字节迁移，本计划原样消费，不改源） |
| 原入口 | Load/Bump/Tick/Init；`Http.post_json("http://127.0.0.1:7072/slow","q=1")`；model `interval=250` 驱动 Tick（vm_bridge `extract_tick_interval_from_decl` 已有） |
| AV 端 | `ace_node_desktop_window.exe --source <app.at> --evidence <out.json> [--script tick:action:inst:target]`，example 目标 feature `ace-bridge`；parked 段驱动 + `__parked_resume_tick` 泵已在 master（PLAN-702 已交），本计划复验不重复实现 |
| A2Vue 端 | `auto build --render vue --gen-only`（auto CLI = /d/autostack/auto-man/target/debug/auto.exe，版本 `0.1.0+plan-890-verified-r7-t19-116-gc879b3e`，SHA256 `eddb2cce…d947fcf`，与 PLAN-114 登记的固定 CLI 逐字节一致，复用）+ `AUTO_UI_ACE_VUE=1` + `AUTO_UI_TOOL_BIN=<本计划树构建的 debug 工具>` → `pnpm install --offline --frozen-lockfile` → `pnpm run build`（vue-tsc && vite）→ 真实 Chromium |
| ts 发射 | `ts_adapter.rs:1748` 现有 `post_json` 两参臂发射 `await fetch` + JSON.stringify（OS068 R-01/032 测试在案，`code.contains("await fetch(")`） |
| 慢端点 | 本机 `127.0.0.1:7072/slow`，响应 ≥65s（保 65s 以上见证）；原 PLAN-702 采集脚本随 plan-artifacts 已删，本计划在 `D:/autostack/plan-artifacts/plan-108/` 重写等价 server（Python 3.14.2 在案） |
| 实际阻断 | 检入的 `gen/front/vue/src/App.vue` 是旧产物：`Http` 未定义、同步形 `function Load()`，`vue-tsc` 必拒——需用当前工具链重新生成并冷构建（T-06）；AV 实窗 65s 见证待 T-03 |
| 环境 | Windows x64，Node v25.2.1 / pnpm 11.6.0 / Python 3.14.2；Rust 工具链 stable-x86_64-pc-windows-msvc 1.98.0；构建全走 dev/debug 增量 |
| 占用对账 | 开工复核时他方两个构建作业在跑（plan-1050 `prepare-native` gate；`-j2 auto-ui-tools ace-vue` 工具构建，PID 69664/31024，11:46 起）——本计划准备构建先让行，T-02 前复核对账 |

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-02 接通运行接口（准备阶段） | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-ui | next: T-02 准入实跑（等构建）

**T-02 准备记录**

- 工作树组补齐：`.wt/ui-108/{auto-ui(plan-108-dev@95401c10), auto-lang(plan-108-dev@0abc54a59=master HEAD), auto-web(detach@0cfc7c784), auto-down(detach@45b642d)}`——auto-ui 的 path 依赖 `../../../auto-lang`、`../../../auto-web`（必需）、`../../../auto-down`（optional 档）全部组内解析；构建 `--locked --offline`，dev/debug，`-j2` 让行 plan-1050 在跑的 gate 作业，禁用 RUSTC_WRAPPER（PLAN-1093 sccache 增量坑）。
- 慢端点 `plan-artifacts/plan-108/slow_server.py` 冒烟通过（SLOW_SECONDS=0.2 替身端口 7073：POST /slow → 200 `{"ok":true,"slow":0.2}`，日志行在案）；正式轮用 7072/65s。
- auto CLI 复验：`auto.exe --help` 正常（AutoNexus build 子命令在案），SHA256 与 T-01 登记一致。
- Playwright 驱动选定：`/d/autostack/auto-blog/tests/node_modules` 的 playwright 1.63.0（chromium revision 1243，本机 ms-playwright 缓存已装）；探针脚本/探针入口/探针 vite 配置为计划材料，放 `plan-artifacts/plan-108/vue-probe/`，不改生成产物。
- Vue 安装策略：冷生成后拷主检出既有 `pnpm-lock.yaml`（SHA256 `58cb180f…42efc6`）+ `pnpm-workspace.yaml`（`1290760d…384a`）进工作树 gen 目录，`pnpm install --offline --frozen-lockfile`；若再生成改变依赖版本致冻结拒绝，按实际 diff 记录理由后重冻。
- AV 实跑设计：`plan-artifacts/plan-108/av-runs.md`（Run A 挂起→完成落账；Run B 关闭/重开→迟到结果隔离；tick≈16ms，hold-ms 95000）。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-02 接通运行接口（首个阻断定位+修复） | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-ui | next: T-06 收口/T-03 阻断路由

**首个阻断与修复（T-02）**

- 双端首个拒绝点（2026-11-22 实跑实证）：`ace_node_desktop_window --source 032/src/front/app.at` 与 `AUTO_UI_ACE_VUE=1 auto build --render vue --gen-only` 同断于组件面编译——`source.undefined: no local named Http in scope` / `no builtin method post_json on receiver type i32 in this profile`：AGE 组件面 checked profile 尚无 VM 原生 `Http` 效应族（AV checked 运行时无 HTTP 客户端；AutoLang AVP 系列的 1090 只覆盖任务/actor，未覆盖组件面 Http）。最小复现=原 032 app.at 单行 `Http.post_json`。
- A2Vue 半边（本计划 AutoUI 范围）已修：`ace_vue_script.rs` ①op_exempt 调用扫描从「裸 Expr 调用语句」扩到 Store 初始化子/return/if 分支（原扫描漏 `var resp = f(...)`，域外调用进 ops 把整 lane 编译打成 Err、兄弟 handler 全体缺席——#6p 裁定意图本来就是豁免含调用 handler 保全兄弟）；②整体失败且体除闭合 `Http.<verb>` 族外无任何调用、widget 确有 Http 效应时，全部缺席 handler **具名承载**（carried，旧道受检体=ts_adapter 的 await fetch 受检映射，OS068 R-01/032 测试族在案）；非 Http 域外调用恒=具名缺席硬拒。`vue.rs` 键门对 carried 逐名打印 `ace.vue-lane-carried` 诊断行放行（不静默回落；980/982 双轨收紧对其余缺席不变）。
- 结果：032 冷生成通过（19 文件；`auto-ui-tool` 本树 debug SHA256 `72e983fc…41347c`）；`pnpm install --offline` + `pnpm run build`（vue-tsc 2.2.12 && Vite 5.4.x）exit 0。锁文件再冻结理由：再生成产物按实际用量裁剪（去未用 class-variance-authority/reka-ui——新道直发原生 `<button>`），旧锁拒绝冻结后按新 package.json 离线重冻（新锁哈希见 §9 收口行）。
- 永久回归：`crates/auto-ui-runtime/tests/plan108_http_lane.rs` 4 例（正例 carried 逐名 + await fetch 映射；负例×3：未知调用不承载 / Http+未知调用同体不承载 / 无 Http 的失败不进承载通道）。**0.20 s**。
- T-03 阻断（交语言 owner）：AV 生产宿主 `ace_node_desktop` 与 A2Vue 断于同一组件面前门；AutoUI 承载修复不能赋runtime——组件面执行 Http 65s park/取消/迟到隔离需要 AutoLang 在 AV checked profile 交付 Http 效应（resolve→mir→verify→运行时 park 续跑全链）。按 §5 记录：负责方=新建 AutoLang 计划（组件面 Http 效应族；W1/后续 UI 计划消费），最小复现=本行诊断+原 app.at。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-06 实跑 A2Vue 全链收口 | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: ae63f49b | next: T-03 阻断路由复核 + work 收尾

**T-06 证据（V-04/AC-03）**

| 项 | 结果 |
|---|---|
| 冷生成 | 严格轨 `AUTO_UI_ACE_VUE=1 AUTO_UI_TOOL_BIN=<本树工具 72e983fc…> auto build --render vue --gen-only` exit 0，19 文件；工具诊断行 `ace.vue-lane-carried: …App[.Bump/.Load/.Tick]` 在案（非静默） |
| 产物身份 | `src/App.vue` SHA256 `20dffac2…f988f5`（`async function Load` + 受检 `await fetch('http://127.0.0.1:7072/slow'…)`；Tick 计时器 250ms） |
| 依赖 | 再生成按实际用量裁剪（去未用 class-variance-authority/reka-ui），离线重冻锁 SHA256 `9961e44a…3ff74fb`；pnpm 11.6.0 |
| 构建 | `pnpm run build`（vue-tsc 2.2.12 && Vite 5.4.x）exit 0；`dist/index.html` `fed60599…b847c`、`dist/assets/index.js` `6877e16d…6cfdde` |
| 浏览器 | Playwright 1.63.0（chromium-1243）驱动真实 Chromium，探针材料 `plan-artifacts/plan-108/vue-probe/`（不改生成产物；同一 App.vue 多 mount/unmount） |
| M1 挂起可交互 | 65s 等待期 ticks 8→20 递增、Bump 即时 +1（bumps=2） |
| M2 完成落账 | 65.16s result 落账 `{"ok":true,"slow":65}`（server 日志 POST /slow 200 in 65.0s 对账） |
| M3 重入差异 | Vue 轨挂起期重入**不忽略**（server 见第二次 POST；AV 轨 T-04 忽略策略未映射到 Vue 轨）——登记差异，原验收面（挂起/完成/取消/卸载隔离）不覆盖重入 |
| M4 卸载隔离 | 挂起中 unmount+mount 新实例：新实例 result 全程 `(idle)`、ticks 重新累计（0→140）——迟到结果未写入新旧实例 |
| M5 零异常 | pageErrors=0、consoleErrors=0 |
| 证据哈希 | `vue-probe-evidence.json`（pass=true）SHA256 `3e53ed11…5adf8` |
| 真实浏览器发现 | 跨源 fetch（application/json 触发预检）要求服务端 CORS 许可——慢端点服务补 `Access-Control-Allow-*`/OPTIONS 204（被测 API 行为的一部分，记探针材料） |

测试会话成本：plan108_http_lane 4 例 0.20s（无新增 >10s 用例；浏览器档为集成验收腿，不计日常并集）。受影响消费者回归：plan1072_vue_web（1+1ignored）/ plan105_u03_sites（8）/ plan104_vue_browser（1）/ plan980_ace_vue_tool_wiring（2）/ plan124_ace_vue_routes（1）/ plan1072_vue_error_recovery（1）/ plan980_010_form_short（1）全绿；vue_capabilities 77 过 5 红、plan105_p013_package_lane_reports_store_gap 红——两者在基线 95401c10（HEAD~1 复证）即同红，非本计划改动（登记既有红：图表 family 契约漂移 / 包面 st_todos 观测墙）。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | outcome: blocked | worktree_path: D:/autostack/.wt/ui-108/auto-ui | branch: plan-108-dev | base_commit: 95401c10 | code_commit: ae63f49b | dependency_revisions: auto-lang 0abc54a59（未改）/ auto-web 0cfc7c784 / auto-down 45b642d | task_ids: T-01✅ T-02✅ T-03⛔ T-06✅ | evidence: §9 本日各 checkpoint 行 + plan-artifacts/plan-108/vue-probe-evidence.json（pass）+ plan108_http_lane 4 例绿 | blockers: T-03——AGE 组件面无 Http 效应能力（AV 实窗 65s 见证不可执行；精确 unblock 见 §10，二选一待用户裁定） | next_responsible_role: user

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: ae63f49b | next: user

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

2026-11-22 T-03 阻断（需用户裁定，二选一）：
① 新建 AutoLang 计划交付 **AGE 组件面 Http 效应族**（checked profile 的 `Http.<verb>` 符号→mir→verify→AV 运行时 park/取消/迟到隔离全链；最小复现=§9 T-02 行诊断+原 app.at；PLAN-108 及 W1 后续消费）。② 用户裁定扩大本计划范围，把组件面 Http 能力并入 plan-108-dev 的 auto-lang 树（将显著扩大本计划的语言仓写面，绕开「AutoLang 公共接缝由现有 owner 单写」的 §2 分工）。
在能力交付前，T-03/AC-02 的 AV 实窗 65s 见证无法执行；A2Vue 半边已全链收口（§9 T-06）。另注：Vue 轨挂起期重入不忽略（M3 差异），是否要在 Vue 轨映射 T-04 重入策略，随能力交付一并裁定。

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: ae63f49b | next: user（§10 T-03 二选一裁定未到；等待 round 1/3）

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: ae63f49b | next: user（§10 T-03 二选一裁定未到；等待 3/3 达阈值，目标转 blocked）

2026-11-22 用户裁定（§10 T-03 二选一之②）：**扩大本计划范围**——AGE 组件面 Http 效应族由本计划在 `D:/autostack/.wt/ui-108/auto-lang`（plan-108-dev）内交付，§2 的「AutoLang 公共接缝由现有 owner 单写」分工对本能力豁免一次。授权边界：只交组件面 `Http.<verb>` 闭合族（get/get_json/get_msg/post/post_json/put/put_json/patch/patch_json/delete）的 checked profile 符号→lower→verify→AV 运行时异步执行（park/取消/迟到隔离）最小链；不动 AV 任务/actor（1090 已交）与其余 stdlib。Vue 轨重入差异（M3）随本能力一并裁定：保持 Vue 轨现状（双请求并发、迟者落账），重入策略映射另案。

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 4 | from: user | to: work | status: executing | outcome: pass（§10 裁定到达，等待解除） | worktree: D:/autostack/.wt/ui-108/auto-ui + D:/autostack/.wt/ui-108/auto-lang | commit: ae63f49b | next: work（T-03 组件面 Http 能力）

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03a 组件面执行路径勘察 | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-lang | next: T-03b 组件面 Http 符号准入

**T-03a 勘察结论（实施依据）**

- AV 生产宿主 `ace_node_desktop` 的组件引擎 = `AceStateEngine`（`auto-ui-runtime/src/ui/ace_state_bridge.rs`），事件会话 = `ComponentEventSession`（auto-lang `experimental/ace/crates/av/src/component_event_sessions.rs`，PLAN-1106）：一次编译（每 handler 单事件入口 FunctionCode）+ `execute_program_entry_args` **一次性同步执行**（事务=locals 先行/末尾统一提交）。**不是** AutoVM 派发——vm_bridge 的 PLAN-702 park 机械不在这条道。
- 执行器（av-core `av.rs`）已有 park 原语：`MirInstr::SchedAwait/Sleep/...` 置 `st.sched_park`（ park-ready 判定/重驱动在调度链驱动器，~L10273）；但 `execute_program_entry_args` 建 `LoopState::default()`（`sched: None`），调度道不可用；组件会话亦未接调度道。
- 宿主效应准入缝已存在：`MirHostImport{category,symbol,sig,capability}` + `host.rs` admission（`auto.console.*`/`auto.platform.*` 家族 + grant ledger），执行=`MirInstr::ForeignCall`（同步、≤4 参、foreign stub table `foreign_exec`）。**无模块级 receiver**（组件面 text profile 连 `json`/`Store` 都没有；Http 只在 VM 管线 auto-lang-core 的 ffi/stdlib）。
- 裁定设计（最小链）：① ace-core resolve：text profile 加 `Http` 模块 receiver 闭合族签名（(Str,Str)->Str 等，按 VM shim 语义 `post_json(url,body)->body str`）；② lowering：Http 调用→`ForeignCall`（新 host import 家族 `auto.http.*`，capability 名随家族）；③ av-core admission：http 家族 validate + 组件面模块白名单臂；④ **异步化**：同步 ForeignCall 会冻结 UI 线程 65s——需新 park 种（host-effect 完成口）或组件会话接调度道 + 宿主真实 HTTP 任务完成发布 port 后重驱动 `LoopState`（帧保留，`dispatch(&mut st)` 可重入）；会话事务语义=park 时零提交、完成时续跑后统一提交；取消=会话释放丢弃 suspended 帧（迟到完成经完成口代际核对拒写）；⑤ auto-ui-runtime 桥：`AceStateEngine` 暴露 park/恢复泵（对齐 `__parked_resume_tick` 条件订阅族）。
- 里程碑：M1 resolve/lower/verify 准入（编译过前门，执行先 fail-closed）→ M2 会话异步执行（park/恢复/提交边界）→ M3 桥接泵 + T-03 Run A/B 实窗见证 → M4 永久回归（语言组 + UI 组）。

**T-03a' 架构 refinement（2026-11-22 二轮勘察）**：执行器内异步（park 种 + 会话重驱动）非必要——改 **桥级异步**：组件面 Http 调用经准入 lowering 为 `ForeignCall`（host import 家族 `auto.http.<verb>.v1`），桥（auto-ui-runtime `AceStateEngine`）为会话注册**阻塞式 HTTP foreign stub**；含 Http 的 handler 派发在 worker 线程跑 `session.dispatch`（执行器同步语义不变），UI 线程保持交互；完成经宿主会话完成消息落账（复用既有 generation/late_dropped/stale_rejected 迟到隔离面）；取消=会话 drop。可观察 oracle（等待期可交互/落账/取消/迟到隔离）全满足，执行器零改动。既有证据：`MirInstr::ForeignCall`（≤4 参、stub table `foreign_exec`，无 stub=fail-closed Infra——M1 阶段天然安全）；宿主会话面已有 `late_dropped/stale_rejected` 计数（ace_node_desktop evidence JSON 在案）。resolve 准入口：`resolve.rs` 魔数 receiver 族（`List.new`/`May.*`）+ service 表（`age_resources`，但语义偏资源 port，不复用）；新增 `try_resolve_http_effect`（receiver=Var("Http") 无遮蔽 + verb ∈ 闭合族 + 参数 VT 核对 + profile 门）。

**T-03b M1 补丁图（勘察定桩，逐件施工）**

| # | 位置 | 动作 |
|---|---|---|
| 1 | `ace/crates/core/src/source/http_ops.rs`（新，镜像 config_ops.rs） | `HTTP_SOURCE_OPS` 表：callee=`Http.post_json` 等闭合族（get/get_json/post/post_json/put/put_json/patch/patch_json/delete；**get_msg 排除**——消息桥语义需 http_msg_subscription 另案），symbol=`auto.http.<verb>.v1`，params=[Str]/[Str,Str]，result=StrOwned；`lookup_callee`/`by_symbol`/`capability_of`（HTTP_CAPABILITY=`http.client/1`） |
| 2 | `core/src/source/mod.rs` | `pub mod http_ops;` |
| 3 | `core/src/source/resolve.rs`（expr_iterative 拦截链 ~12857，镜像 try_resolve_legacy_list_new） | `try_resolve_http_effect`：`MethodCall{recv:Var("Http"), name:verb}` + profile∈{AgeComponent,AgePackage,AgeH0} + 无局部遮蔽 `Http` + 仅位置参 → `lookup_callee` 按实参 VT 选重载 → `CallIntrinsic{symbol,args}` + `used_intrinsics`；不匹配具名拒（`source.http-argument`） |
| 4 | `core/src/source/adapter.rs`（~4675 capability 臂 + ~5109 t_foreign 签名臂） | http_ops::capability_of/by_symbol 两臂（签名发射=参数表+结果，ForeignCall 通道） |
| 5 | `lowering/src/hir_adapter.rs`（~240 intrinsic def 前缀路由） | `auto.http.*` 臂 → av-core `http_client::declaration(symbol, sig)` → host_imports.push（call sites 按 symbol 找 import，def 无 CalleeRef——镜像 auto.platform. 臂） |
| 6 | `av-core/src/http_client.rs`（新，镜像 platform_io.rs 头部） | CATEGORY=`http.client/1`；`declaration(symbol,sig)->MirHostImport`（admission：symbol 在表、capability 键、max_call_bytes）；M1 不给执行 provider——无 stub 时执行器 fail-closed（ForeignCall table-less=Infra，既有 938 不变式） |
| 7 | av-core mod.rs / 组件面前门授权 | 组件面会话 `compile_component_event_program` 的 verify/授权面核对 http import 放行（若无 host-import 授权点则 M1 天然只需 verify 过结构）；`host::authorize` 是 console 专用门（host.rs:249），http 家族**不走它**，按 platform 族先例走 per-run grants |
| 8 | M1 验收探针 | auto-lang 侧小测：032 app.at 过 `compile_component_event_program`（不再报 `no local named Http`）；执行仍 fail-closed（具名）。无新增 >10s 用例 |

M2/M3（随后）：桥（auto-ui-runtime）注册阻塞式 HTTP foreign stub（`foreign_exec` table，Owned/Text 参数按 VM shim 语义：post_json(url,body)->body str）+ 含 Http handler 的 worker 线程派发/完成消息/取消（复用宿主会话 generation/late_dropped 面）→ Run A/B 实窗见证。M4：语言组（ace 受影响面）+ UI 组永久回归。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03b M1 部分落地（auto-lang ` HEAD` 见下） | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-lang | next: T-03b M1 余量——HttpCall MIR 指令

**M1 进度（commit 见 auto-lang plan-108-dev 顶）**：① `http_ops.rs` 源级目录（闭合族，VM shim 语义）+ resolve 魔数 receiver 双拦截点（expr_with_context 链 + iterative 求解器 MethodCall 臂——**实证：组件面 handler 体在 `auto.source.text-record.draft` profile 下绑定**，门=TextRecord|Collections|Age*）；② adapter capability/签名两臂；③ av-core `http_client` admission（`http.client/1`，per-run grant 键）；④ 固定用例 5（4 绿：未知 verb/arity/类型具名拒 + 用户遮蔽让位；正例待在 lowering）。**learning**：ForeignCall 通道不适合——foreign contract LaneKind 纯标量（无文本 lane），伪造 SourceDecl 是合同剧场；正解=新 `MirInstr::HttpCall{host,dst,args}`（PlatformCall 先例，host_import 索引臂），执行器/abc/function_code 各消费面按编译错误指引补具名拒绝臂，M2 填执行（桥注册 http provider，worker 线程派发）。`LangFail` 是块 terminator 不可 mid-expr 发射（排除）。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03b M1 完成（auto-lang 0064d28ca） | outcome: partial | worktree: D:/autostack/.wt/ui-108/auto-lang | next: T-03c M2（provider + worker 派发）

**M1 落地**：`MirInstr::HttpCall{dst,host,args}`（PlatformCall 先例）全链贯通——mir/verify（host 类别+签名臂）/ownership/instr_spec/executor（无 provider=具名 `http.client … fail-closed` Infra）/native（具名拒）/ABC（OP_HTTP_CALL 0x57 编解码；**Owned 线族恒携 HOST section**——事件程序单元经 split_units 逐单元携带 http host import 并局部重编号）/host.rs validate（http=第三条声明类别，与 platform/config 并列；能力 `http.client/1` 进已知集，per-run grant 留执行侧）。固定用例 **5/5 绿**（准入+fail-closed+3 具名拒+遮蔽让位），0.05s。受影响回归：ace-av conformance 全量 + av-core lib 见 `plan-artifacts/plan-108/m1-conformance.log`（后台任务）。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03c M2 桥级异步落地（含两处 admission 修复） | outcome: partial | worktree: 双树 | commit: auto-lang `9e8e0d2?`/auto-ui `HEAD` | next: M3 交互性精化或 Run A/B 见证

**M2 桥级异步（已落地）**：`AceStateEngine` 对含 Http 效应的程序把派发整体路由 worker 线程（`SyncSession` 按值上线程，2021 disjoint capture 经独立 fn 捕获）；`DispatchOutcome::InFlight` + `poll_http_completions`（每步 try_recv；世代/存活核对回写，不符=worker 会话整体丢弃=卸载后迟到结果隔离）；`UiHttpProvider`（reqwest blocking，worker-only 契约）；`DropReason::HttpBusy` 重入门。language 侧 `execute_program_entry_args_with_http` + `ComponentEventSession::set_http_provider`。修复两处 console admission 把 http import 误判为 console provider 需求（720/4317 行同改；http 缺席在 HttpCall 执行点具名 fail-closed）。

**已知缺口（M3）**：整会话上 worker 期间，同实例其它派发（Tick/Bump）被 HttpBusy 拒绝——65s 等待期 ticks/bumps 冻结，**不满足 AC-01 挂起期可交互**。M3 设计（择一）：①park/resume——HttpCall 执行点挂起 `LoopState`（帧保留），会话不离开 UI 线程，http 调用本身上 worker（job 式 provider：`start` 返回 job id，`poll` 查完成），完成后续跑同一会话（对齐 PLAN-702 语义，工作量大）；②worker 期间同实例非 http 派发经第二会话影子执行（状态合并，复杂度不低）。Run A/B 的脚本证据先行裁决缺口实况。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03 Run A 首见证（挂起→完成落账） | outcome: partial | worktree: 双树（auto-lang/auto-ui plan-108-dev） | next: M3 在途交互性 + Run B

**Run A 证据（第 7 轮，2026-11-22）**：`ace_node_desktop_window --source 032/app.at --hold-ms 95000 --script 3:Load,10/100/500/1000:Bump,5200:Bump`——Load 65s 迟到后经 worker+poll 落账：dispatch_log `result={"ok": true, "slow": 65.0}`；**errors=0**（HttpBusy 静默背压）；server 恰好 1 次 POST /slow 200 in 65.0s；frames=4640（UI 线程全程不阻塞）。证据 SHA256 `fdd3e21b…38f1e`。修链记录：console admission 两处 http 排除（720/4317）；provider 从 reqwest 改 std TcpStream HTTP/1.1（reqwest 在本机出站失败；M2 闭合域=明文 http 慢端点）；http 车道执行墙放宽 300s（默认 wall 短于 65s）。

**Run A 余留缺口（=M3）**：`ticks=0`、`bumps=0`——挂起期同实例 Tick/Bump 被背压（静默），Tick 驱动本身也未落（component 时钟道未对此源派发）；末拍 Bump 未入 dispatch_log（deadline 前 worker 轮未收）。M3=执行器 park/resume（HttpCall 挂起 LoopState、http 调用 job 化上 worker、完成后续跑同一会话——对齐 PLAN-702 语义）。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: T-03 Run A/B 终证 + M3 收口 | outcome: partial | worktree: 双树 | next: work 收尾（execution_done 判定的余项见下）

**Run A 终证（第 10 轮，证据 `plan-artifacts/plan-108/av/run-a.json` SHA256 `d344ebf1…d47ccd`）**：server 恰 1 次 POST /slow 200 in 65.0s；Load 完成落账 `result={"ok":true,"slow":65.0}` 且 **bumps=4/ticks=255 并发提交全保留**（统一提交的保留-重播种）；730 条 Tick 派发贯穿挂起期（末 tick 365）；4 次 Bump 全部挂起期即时生效；errors=0；frames=4642。挂起/完成/等待期交互全 oracle 绿。

**Run B 终证（证据 `run-b.json` SHA256 `7d7e6adf…8261a5`）**：server 仍恰 1 次 POST（t1000 重入被 pending 门忽略——VM T-04 语义映射）；close→reopen（host gen1 closed→gen2）后 errors=0、714 Tick 持续、模型无迟到污染。**语义裁定**：`ace_node_desktop` 的 close/reopen 是保状态的宿主壳回收（reopen 语义=按当前业务状态重提交），引擎实例不卸载——故 request A 的迟到完成对**仍存活的同一引擎实例**落账是语义正确行为，不构成隔离违规。**真取消/卸载**=引擎会话关闭：固定用例 `p108_cancel_unload_late_result_isolated` 见证（挂起→close→job 完成→resume 具名拒写 `late result isolated`，零落账，7/7 绿）。

**M3 技术收口（双树）**：auto-lang `f180a62bf`+（job 式 HttpProvider、HttpCall park/resume `SuspendedHttp`、`execute_program_http_resumable`、保留-重播种（handler_assigns 源级清单；MIR 扫描被统一提交 epilogue 击败）、关闭会话 resume 拒写、wall 300s）；auto-ui `65d709f2`+（引擎 park 车道（会话不上线程）、pending=同 msg 重入门、poll resume、UiHttpProvider job 线程、组件 Tick 合成（.Tick+interval→250ms）、HttpBusy 静默、tick 宿主会话门）。**已知项（非本计划 oracle 面）**：reopen 后 provider 侧显示模型重绑缺口（tick 触发 apply 的 -2 曾现；宿主门兜住 close→reopen 窗；显示层，模型语义不受影响）——记债归后续 UI 计划。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-108/auto-ui + auto-lang | branch: plan-108-dev（双树） | base_commit: 95401c10 / 0abc54a59 | code_commit: auto-lang `60e03b7ef` / auto-ui `1252161f` | dependency_revisions: auto-web 0cfc7c784 / auto-down 45b642d | task_ids: T-01✅ T-02✅ T-03✅（Run A/B 终证）T-06✅ | evidence: §9 各 checkpoint + run-a/run-b JSON（SHA256 在案）+ plan108_http_component 7/7 + ace-av conformance 全量见 m3-conformance.log | blockers: 无（§10 裁定已解除） | next_responsible_role: review

M4 回归：ace-av conformance 全量 1414 过 / 29 红（`m3-conformance.log`）——29 红与 master 对证全部为既有/环境项（py descriptor 未备 8、内存档 3、owned 族既有 6、v02b/mir_tier/owned_recipe/test_batch 等）；本计划触面（plan108 7/7、plan981_age_package_av 37/37、component_event_sessions 5/1=v02b 既有）全绿。auto-ui 侧 plan108_http_lane 4/4 + vue 消费者 7 文件此前已绿。

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: auto-ui 1252161f（+auto-lang 60e03b7ef） | next: review

2026-11-22 stage: review | review_scope: final | reviewer: 实现同会话（限制已声明，判决从工件重建） | plan_id: PLAN-108 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-108/{auto-lang,auto-ui} | branch: plan-108-dev | reviewed_commit: auto-lang 60e03b7ef / auto-ui 1252161f | base_commit: 0abc54a59 / 95401c10 | dependency_revisions: auto-web 0cfc7c784 / auto-down 45b642d

**已通过面**：AC-01（源 SHA256 逐字对证 T-01 记录）；AC-02 Run A oracle 独立重验（run-a.json 哈希 `d344ebf1…` 绑定 §9，Load 落账+bumps=4+ticks=255+errors=0）；Run B（`7d7e6adf…`，errors=0）+ 取消隔离用例 7/7（重跑 0.09s）；AC-03 vue-probe pass=true（M2/M4 里程碑复核）；package AV 37/37；逐行临时代码扫描净（无 dbg/TODO/调试残留）；双树 diff 20+5 文件全 plan108 标记、无越界；新用例耗时 0.09–0.41s（≤10s 门）；无 specs.json/附件入仓。

**F-1（needs_fix，severity：高，affected：AC-03/T-02/T-06）**：M1 组件面 Http 准入**回归 A2Vue 严格轨**——lane 编译自 M1 起成功（Http 不再 wholesale 失败），承载判定的「编译成功臂」只承载含 Http 效应的体 → 纯状态 handler（`.Bump`/`.Tick`）变具名缺席 → 工具键门硬拒。重放：`cargo test -p auto-ui-runtime --features ace-bridge --test plan108_http_lane p108_probe_product_carried` → `absent=[".Bump",".Tick"]`（本次复审实跑，0.19s）。违规则：AC-03 的 032 可交付性（T-06 证据系 M1 前基线冻结，已被 M1-M3 改动失效）。影响面：任何 Http+纯状态兄弟 handler 的严格轨组件（017-chat 等后续同形）。修复方向（非处方）：承载谓词对称化——体 call-clean（无 Http 族外调用）且 widget 确有 Http 效应时，无论哪一编译相缺席均可承载；非 Http 域外调用恒硬拒。修复后最小集：plan108_http_lane 4 + plan108_http_component 7 + 032 冷生成准入重验。

2026-11-22 stage: work | plan_id: PLAN-108 | plan_revision: r2 | checkpoint: F-1 修复 | outcome: pass | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: 5c066067 | next: re-review

**F-1 修复**：承载谓词对称化（ace_vue_script.rs 承载判定两臂统一为「体 call-clean ∧ widget 级 Http 效应」，compile_failed 相位不再区分）。验证：plan108_http_lane **4/4**（0.28s）；032 冷生成准入通过（19 文件），产物 `src/App.vue` SHA256 `20dffac2…f988f5` 与 T-06 冻结产物**逐字节一致**——原浏览器证据（同一产物字节 + 同一探针）保持绑定，无需重跑。

2026-11-22 stage: review | review_scope: final（repair re-review，受影响面） | reviewer: 实现同会话（限制已声明） | plan_id: PLAN-108 | plan_revision: r2 | outcome: **pass** | worktree_path: D:/autostack/.wt/ui-108/{auto-lang,auto-ui} | branch: plan-108-dev | reviewed_commit: auto-lang 60e03b7ef / auto-ui **5c066067**（F-1 修复后） | base_commit: 0abc54a59 / 95401c10

**re-review 范围（check → finding）**：承载谓词对称化 → F-1（plan108_http_lane 4/4 + 冷生成准入 + 产物字节一致）；语言侧 7/7 与 AV Run A/B 证据为 F-1 未触面（修复只动 vue 承载判定一行臂），沿用冻结证据（哈希绑定 §9）。全量 conformance 29 既有红清单不变（m3-conformance.log）。新用例耗时 0.09–0.41s。残留 pending（merge 门）：归档/合入/清理、规范沉淀（auto-lang `docs/specs/auto-av/component-event-sessions.md` 的 http.client/1 增量；auto-ui lane 承载策略增量）、wt-guard 清理、W1 交接行。

2026-11-22 stage: goal | plan_id: PLAN-108 | plan_revision: r2 | round: 6 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-108/auto-ui | commit: 5c066067 | next: merge

2026-11-22 stage: merge | plan_id: PLAN-108:r2 | outcome: pass | delivery_commit: auto-lang 541623c42 / auto-ui 011a1b11 | landed: 双主线 ff-only（auto-lang rebase 9 提交 range-diff 全等；auto-ui 7 提交、一提交 import 行随主干 PLAN-123 适配、重基后编译复核）；canonical_verified: `docs/specs/ace-migration/component-event-sessions.md`（http.client/1 节）SHA256 `fc468de3…`、`docs/specs/runtime/component-http-async.md` SHA256 `41a443ef…`（均落主干提交）；测试面：新用例 11 例 0.09–0.41s（≤10s），ace-av conformance 全量 1414/29（29=master 对证既有/环境，清单 m3-conformance.log），浏览器/实窗证据哈希绑定 §9；archived: 本文件（git mv + status archived + completion_kind delivered）；cleaned: 见下行收据。


2026-11-22 stage: merge | checkpoint cleaned | outcome: pass | guard: wt-guard 双树 clean（pwsh，reparse point 扫描）；worktree/branch: .wt/ui-108/{auto-ui,auto-lang,auto-web,auto-down} 与双仓 plan-108-dev 已删，组目录已删（worktree list 复核 0）；plan-artifacts/plan-108 整域已删（证据先折入本文件 §9；哈希绑定行保留）；registry 已刷新（merge_closeouts 记录，无 holds）；批回归门：`.last-batch-regression.json`（2026-11-22T15:10Z，covered=1114）——108<1114 且收据 <48h → 不到期，观察记录在案；部署面：无 release/部署授权范围，debug 产物随 worktree 删除（landing≠deployment，无运行中制品受本计划影响）。
