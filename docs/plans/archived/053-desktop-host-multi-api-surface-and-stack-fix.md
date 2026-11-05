---
plan_id: auto-ui/PLAN-053
status: archived
feature_name: desktop-host 多 api surface 演进与 run_ui 栈溢出根修
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 2
current_step: 6
total_steps: 6
base_branch: master
base_commit: 8ed8a2e1f2dc93bf31d6f730d380f55bdbc9baa1
depends_on: [auto-ui/PLAN-050]
affects: [desktop-host, vue-shell, mcp-server, auto-os/PLAN-054]
new_spec_components: [docs/specs/build/desktop-api-surfaces.md]
supersedes_spec_components: [docs/specs/build/generated-vite-config.md, auto-lang:docs/specs/auto-lang/frontend/overview.md]
touched_goals: [G1, G2, G3]
completion_kind: delivered
---

# auto-ui/PLAN-053：desktop-host 多 api surface 演进与 run_ui 栈溢出根修

## 0. 变更摘要

`auto run --desktop`（vue 桌面宿主，生成于 `auto-ui-tools/src/vue.rs`
desktop-host 模板）当前为 v1 单视图/单 api 形态，对 auto-os/PLAN-054 的
8 个基础应用组合形成两个硬障碍：①shared `@/lib/api` 命名空间全桌面仅一
个 api-client app 可嵌（其余 skip "API client namespace claimed by …"）；
②扫描/装载阶段 `work:run_ui` 线程默认 8MB 栈溢出（thread has overflowed
its stack），RUST_MIN_STACK=64MB 下不复现（深递归型，根因未定位）。本计划
演进宿主到 per-app api surface（或多 surface 并存）形态并根修栈溢出——
auto-os/PLAN-054 vue 轨 8 app 嵌载的具名最小前置。

## 1. 事实与证据（2026-11-05 T-04 实机）

- 障碍①：候选组合 8 app 全部预生成 `gen/front/vue/src/lib/api.ts` glue
  （6/6 补生成 PASS + notes 已有 + charts 不需）后，desktop-host 扫描仍
  只放行**第一个** api-client app（auto-blog 字母序胜出），其余 7 个 skip：
  `API client namespace claimed by …/auto-blog/gen/front/vue/src/lib/api.ts`。
  代码锚：vue.rs ~5940-6000（"v1: one api surface per desktop"，首赢者
  重写共享 `src/lib/api.ts`，后续 app 撞名 skip）。
- 障碍②：首次 `auto run --desktop`（AUTO_DESKTOP_APPS=候选 apps）在扫描
  24 app 后 `thread 'work:run_ui' (…) has overflowed its stack` →
  `run_ui failed: tool host closed the control stream`；同输入
  RUST_MIN_STACK=67108864 下不再溢出并走到 dev server 启动（端口另障见下）。
  属深递归/深嵌套（疑 corpus 合并或 widget 树递归），根因未定。
- 伴随观察（非本计划核心但顺带记录）：shell 默认端口 3000 落在本机
  Windows TCP 排除段 → EACCES；`AUTO_FRONT_PORT` 可覆盖（已实证绕过）。
- 消费方面：auto-os/PLAN-054 §5.5 明确——宿主不兼容时给最小反例与 owner
  前置，受影响任务保持未完成，不退为外部窗口冒称嵌入。本计划即该前置。

## 2. 目标

- G1：desktop-host 支持 N 个 api-client app 同场嵌入：每 app 独立
  api client 命名空间（如 `@/lib/api-<id>` 或 alias 表），store/ext 等
  共享面冲突策略显式化；8 目标 app 全部出现在 shell 并可打开。
- G2：`work:run_ui` 在默认栈深下不再溢出：定位递归根因改为迭代/限深，
  大 app 集（≥24）扫描装载为受面；回归用同输入候选组合。
- G3：v1 的其余守卫（router pages/ext/i18n skip）行为不变，除非逐案
  有界扩展并记录——不连带做大桌面重写。

## 3. 边界

- 修改范围为 auto-ui（宿主生成 + mcp/tool host 如需）及用户于 2026-11-05 明确追加授权的 auto-lang 最小解析器根修；不动 auto-os shell.at
  业务；不改 8 app 源码；不承诺双 surface 之外的桌面新形态。
- run_ui 栈修复不得以单纯调大默认栈交差（须根因修复 + 大输入回归）。

## 4. 验收标准

- AC-01：候选组合 8 app 在 desktop-host 全部可嵌可开（启动器列出、
  窗口打开、前后端各自 profile 工作）。
- AC-02：默认栈深（不设 RUST_MIN_STACK）扫描装载 24+ app 不溢出。
- AC-03：现有单 api-client app 组合（如仅 blog）零回归；既有
  desktop-host/vue.rs 单测全绿。
- AC-04：变更只落 auto-ui 与用户追加授权的 auto-lang 最小解析器根修；不改 8 app 源码，命名空间策略与解析器根修写入各自 canonical Spec。

## 5. 执行步骤（草案，待 work 细化）

- [x] T-00：多 surface 设计决策（alias 方案 vs 独立 chunk）+ 栈溢出
  根因定位（递归路径最小复现）；产出 decision 文档。
- [x] T-01：实现 + 单 api 旧形态回归。
- [x] T-02：候选组合 8 app 实机验证（嵌/开/基本任务）+ 大输入不溢出。
- [x] T-03：漂移/文档/收据，execution_done。
- [x] T-04：独立最终 review（owner_stage=review）。
- [x] T-05：跨仓 merge / canonical Specs / 归档与清理（owner_stage=merge）。scratch/空组最终清理收尾见下方「cleanup closeout / continuation 4」与 registry 回执。

## 6. 执行入口与验证合同

- 授权：用户 `/goal 完成计划053` 授权 work → 独立 review → 有界修复 → merge/归档/清理全链。
- 唯一实施树：`D:/autostack/.wt/ui-053/auto-ui`，分支 `codex/plan-053-dev`，基线 `e3f0933d2384921aa49df8007cf9730ad7723692`。
- 只读依赖树：同组 auto-lang（r1 基线 `5f6737467`，r2 实施前快进 master `a0750086c`；parser 文件 diff=0）、auto-down `7c4d5cb`、auto-web `37d32c2`。主检出既有文档 WIP 和四张 terminal_pixel PNG 测试产物保留；没有源码 WIP。
- V-01 / AC-01：真实候选 OS 8 app → from_workspace → desktop host 发射 → 启动器及逐窗打开、基本前后端操作；区分多 app API 误绑与各自正确 profile。work 负责，实际跑完才关闭。
- V-02 / AC-02：同候选 24+ app 集，在不依赖 RUST_MIN_STACK 的线程扫描装载；先定位实际溢出帧/最小反例，再验证根修。仅增加线程栈不算通过。
- V-03 / AC-03：单 api 组合、多个 api 客户端不同返回标记、无 glue/route/ext/i18n 守卫及现有 vue 模块测试。检查重复组件/store 文件的实际所有者，禁止通过共享首胜者把 app API 误绑。
- T-00 设计与根因已关闭；canonical Spec delta 已在实施树提交；最终统一受影响门由 review/merge 负责。

### T-00 已核验事实与有界设计（已关闭）

- 原始 `plan-artifacts/plan-054/t04/vue-desktop.log` 实际注册 27 项（23 个主根 + auto-musk/jade-garden/os-config/ui-gallery），不是正文估计的 24 项。原崩溃发生在主根 weather 后、额外根 auto-musk 的解析阶段。原始业务 8 app 不变。
- 默认 worker 栈对照：显式 2 MiB（不读 RUST_MIN_STACK）逐项扫描 23 主根 PASS，完整 host（不含 extras）PASS；auto-musk 单根立即 c00000fd。CDB 栈锚 `auto_ui_dialect::grammar::view::{parse_view_node,parse_view_conditional,parse_view_for_loop}`，parse_view_node debug 帧约 0x131a0=78 KiB；不是跨 app corpus 累积。
- 根修设计：元素嵌套改 heap frame 迭代，header/prop 临时量不跨下降保留；else-if 链迭代；后续三条 widget mint 遍历同因大帧改为保持先序的迭代。未提高生产线程栈、未引入语法拒绝阈值。
- 永久 `view_stack` 两例：2 MiB 上 64 元素、128 else-if、for/if/memo/link 混合；属性/事件/连续块与缺括号拒绝。首轮抓到 mint_modal_dialog_toggle_inner 的后续溢出并补同因修复；现 2/2 PASS，dialect lib 95/95 PASS（原始日志 plan-053/view-stack-tests-2.log 与 dialect-lib.log）。
- API 设计：每 app `apps/<id>/lib_api.ts` + 本地 components/stores；仅改已知自有模块 import，宿主 UI/运行时仍共享。HTTP/SSE 相对 API 字面量带 `/apps/<id>/api/`；Rust 保留 pac api:rust/back_port，VM 走现有 session proxy。复用单 app Rust launcher，不新建 supervisor；端口重复/已占用先拒绝。
- Blog 必需的有界 router 扩展：仅 API-backed routed app 复用现有 gallery memory-router/page 发射和薄挂载包装；纯前端 routes、ext/i18n 原 guard 保留。每挂载新 router；route wrapper 的 store context 沿既有 helper 安装。
- `file:`/`link:` npm 依赖由 app 生成目录解析到实际路径，防 host 移位断链；npm 同名包维持既有 first-wins。run 的宿主依赖合并调整为 package 同步之后，防刚合并的跨 app 依赖被清除。
- 永久 client 单测：同名 store 的 alpha/beta 各自 client owner 哨兵正确，随后单 alpha 注册不残留 beta；PASS（client-test.log）。后端/8 app 实机与完整 extras 栈验证仍待执行，以上不构成 AC-01/02 通过。
### r2 补充执行合同与规范增量

- 用户对 Q-053-01 明确答复“允许最小跨仓根修（推荐）”；授权范围新增 auto-lang parser，不改变 AC-01/02/03 的任何阈值。AC-04 仓库限定按该明确授权更新。
- auto-lang 唯一实施树继续同组 `D:/autostack/.wt/ui-053/auto-lang`，分支 `codex/ui-plan-053-parser-fix`，当前基线 `a0750086c`；r1 SDK 至该提交仅并入已交付 934 等变更，parser diff=0。主检出仅他人文档 WIP、导入预算数据/锁文件测试产物，无源码 WIP，全部保留。
- V-04 / AC-02 / owner_stage=work：合法嵌套 on-handler 的 if/loop + 嵌套命名调用参数，以实际 `Parser::node_or_call_expr/args/parse_stmt` 默认栈入口解析；永久区分反例与浅层合法控制，坏括号必须错误而非挂起。不得复制语法实现到 UI 或增加语法拒绝阈值。核心路径修复应缩减/消除无保护递归帧，或补齐现有动态栈机制漏掉的递归入口；不得仅增加 run_ui 默认栈。
- 语言组门：`cargo t parser`（完整固定组，owner_stage=merge；work 跑最小反例与相邻 parser 测试）；UI 消费门仍为 dialect 深度回归、现有 vue/desktop 单测与真实 27 项扫描。两个仓的独立最终 review 顺序共享原实施树，禁止新建 reviewer checkout。

| delta_id | action | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | auto-ui: docs/specs/build/desktop-api-surfaces.md | shared single API → app-owned API/modules/profile routes, bounded memory-router extension | 8 app 嵌载 | AC-01/03/04 |
| SD-02 | modify | auto-ui: docs/specs/build/generated-vite-config.md | host-only npm source → desktop admitted app union, generation/run identical | cold lazy imports | AC-01/03 |
| SD-03 | modify | auto-lang: docs/specs/auto-lang/frontend/overview.md | missing recursive call-stack safety → verified guarded/iterative entry contract | remaining core parser overflow | AC-02/04 |

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-053 | outcome: drafting（证据随案；
  由 auto-os/PLAN-054 T-04 定界产出，消费方=054 恢复后 vue 轨） | next: work/T-00

- stage: work | plan_id: PLAN-053 | plan_revision: r1 | outcome: blocked (V-02 cross-repository portion only; independent V-01 work continues pending answer) | worktree_path: D:/autostack/.wt/ui-053/auto-ui | branch: codex/plan-053-dev | base_commit: e3f0933d | code_commit: uncommitted checkpoint under verification | dependency_revisions: auto-lang 5f6737467 / auto-down 7c4d5cb / auto-web 37d32c2 | evidence: stack-full-r4-debug.log, candidate-default-stack 2 MiB c00000fd; dialect lib 95/95; desktop tests serial 7/7; eight-app generation 8/8 | blockers: Q-053-01 | next_responsible_role: user for repository-scope decision; work for independent API verification

- stage: goal | plan_id: PLAN-053 | plan_revision: r1 | round: 1 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/ui-053/auto-ui | commit: e3f0933d (working changes under verification) | next: new
- stage: new | plan_id: PLAN-053 | plan_revision: r2 | outcome: pass | authorization: 用户明确允许 auto-lang 最小跨仓根修；Q-053-01 resolved | changes: repository scope / AC-04 scope qualifier / V-04 / SD-03 | next: work
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 2 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-053/auto-ui + auto-lang | commit: UI e3f0933d / lang a0750086c | next: work

- work progress r2 / T-00 closed：core commit `61207576b7676148499854f9df4b2d019a476972`，仅 statements/scenario 的缺失动态栈检查入口、永久 parser 用例、frontend Spec；未增大默认线程栈或既有 512 KiB/1 MiB segment 参数。`ui053_nested_statement_calls_fit_default_worker_stack` 1/1 PASS（默认 2 MiB，48 calls + 12 if/loop pairs + 24 nested args，坏括号拒绝）。同版 UI consumer `candidate_default_stack` 完整原始冻结 27 项（23 主根+4成功extra）**PASS**，8 目标 registry assertions 全过，17 项获准嵌入，140.31s；命令未设置 RUST_MIN_STACK，worker 显式 2 MiB。证据 `core-parser-regression.log` / `stack-full-core-fix.log`。旧 r1 栈证据保留为敏感性反例，不再当当前失败。
- fixture 路径对账：8 app 验证输入从 UI target 的临时目录迁至唯一 scratch `D:/autostack/plan-artifacts/plan-053/candidate-os`，host 源副本在 `desktop-project`；pac/src/vendor 与候选源码同字节，不改真实 app 源码。迁移前按 no-follow 逐项移除自行 pnpm 生成的链接（两个生成目录各 723 项），不穿透目标；原预览与 live gen 仅作为过渡材料移至同 scratch，后续 PM 安装均在 scratch。正常 Cargo 中间产物仍在各实施树 target，随 worktree 清理。

- work progress r2：UI checkpoint `b892b4ed` 已无冲突 rebase 最新 master `426c1f70` 得 `9a94d568778793700b2718ec0a949d60a0d38bcf`；`git range-diff e3f0933d..b892b4ed master..HEAD` 为完全相等（=）。主干变化涉及 native theme/graphics，与 desktop 发射、view/parser 机制无交叉；最新实际运行重新覆盖运行时集成。
- `cargo test -p auto-ui-tools --lib vue:: -- --test-threads=1` 最终 103 passed / 0 failed / 1 ignored（唯一 ignored 为手工大输入 harness，已由 V-02 显式执行）。三项旧失败仅因示例 owner 从 auto-lang 迁至 auto-ui，改用 canonical 示例路径及同形布局，未跳过测试或减弱断言。`vm-body-regression.log` 2/2：本地/导入常量、命名字段优先、原始 JSON scalar/object/text 单 str whole-body、多参缺参/空体拒绝。真实 Charts 原先 503 为 Const 未初始化；随后 400 为桌面代理缺 standalone whole-body 兼容，均已补永久反例。
- 首轮八窗全部打开，无页面异常；六项实际任务通过（Notes/Todo 写入并 HTTP 读回、Paint SVG、Database SQLite 只读 hash 不变、Chat 测试身份注册、Blog memory-router 草稿自动保存）。Calendar 首轮 probe 误依赖可选 overlay 标记，改为可见日期/按钮 oracle；屏幕根布局适配亦改为模板根识别。Charts 与 Calendar 最新最终实机仍待通过，不以首轮六项结果关闭 AC-01。旧宿主 PID 31700 经绝对路径核对退出后 46253 及六个 API 端口全部关闭，未触及其他进程。


### r2 work 完成与证据冻结

- AC-01 / V-01 PASS：最新 UI `9a94d568` + lang `61207576b` 的真实 `auto run --desktop`，8/8 registry、8/8 窗口和基本任务通过；48 次 API 响应，浏览器 page/console error=0。Rust：Notes/Todo 本轮独有时间戳写入并读回、Calendar 当月日期加载及下一月切换且按钮在客户区、Database SQLite sentinel 表可见且文件 SHA256 不变、Chat 注册新测试身份（不保留凭据）、Blog 草稿自动保存。VM：Paint 绘制并导出 SVG、Charts 中文路径 CSV 导入显示 2 行。临时项目全部位于 scratch；八 app 共 82 个 pac/src 文件与原输入逐字节相等。
- AC-02 / V-02/V-04 PASS：完整原始冻结 27 项、显式 2 MiB worker、不设 RUST_MIN_STACK，扫描装载 140.31s、17 admitted、8 目标注册断言；原失败 CDB 路径已由永久 view_stack 与 core parser 最小反例覆盖。后续改动仅 API 发射/运行时/根布局，重基主干未动这些 parser 路径，故复用该有效大输入证据；不重复做无关规模承诺。
- AC-03 / V-03 PASS：vue 模块 103/103（手工扫描 harness 单独显式通过）；同名 store 双 client 哨兵归属、随后单 alpha 无 beta 残留；最新 desktop 模块 8/8（独立真实后端 probe 另显式执行）。单 Rust API probe 真实 HTTP 返回 53，drop 后端口关闭；原单-app launcher 默认行为不改，desktop 新入口禁止清既有端口。永久 duplicate/occupied 负例保留监听者。无 glue、route/ext/i18n guards 在现有模块测试覆盖。
- AC-04 PASS：只提交 UI 13 文件与 lang 4 文件；auto-os 八 app 源码未改，auto-web/auto-down 为只读 detached 树。规范为 SD-01/02/03 三个 exact canonical paths，已经提交；无 legacy ledger/index 写入。
- 受影响 UI union：view_stack 2/2 + dialect lib 95/95 + tools vue 103/103 + 最新 desktop 8/8 + runtime VM 2/2 + real Rust owned roundtrip 1/1。重基仅 native-theme/graphics/既有 launcher adjacent changes，range-diff 等价；最新 bin 的八应用真实运行与 runtime/desktop 重跑覆盖集成。merge-owned language union 尚待 `cargo t parser`，不能提前声称通过。
- 当前 readiness：T-00..T-03 4/6 done；T-04 review 和 T-05 merge 明确 pending；open implementation findings=0。生产发行物尚未部署，merge 记录实际陈旧/缺失观察。

证据根 `D:/autostack/plan-artifacts/plan-053/`（原始文件 merge 后按政策删除，本表与上述关键结果为持久证据）：

| 文件 | SHA256 |
|---|---|
| `eight-basic-final.log` | `7c9816b1f28a5f17016534f5b573625208a556b1acd20af80b872956b6499d5b` |
| `eight-app-basic.json` | `0a748379f36262c0b7fddc79ca4a4c90c14608702c02eb846db7ec5778f08795` |
| `candidate-inputs-final.json` | `ba8569e42f6ef28d378c0566faeb6b32790b2ca41daea7ecd35af99af86885a8` |
| `stack-full-core-fix.log` | `08858bafbe7f517a9fae806f8f392737107cd69c88f457abb129c92f2439c46a` |
| `core-parser-regression.log` | `33ceee2919bf01b7c0866c7b2926722bfc3ed722af0e66f8cddb0168ab8d939a` |
| `view-stack-tests-2.log` | `469c803ad1d334865e6af0f706d898a6830c7e693364b4010d10e627571739c7` |
| `dialect-lib.log` | `3966ca35733964387a129b75023b361292aae7298e0e9bdba790b912b2dec882` |
| `vue-final-tests-2.log` | `30b006859d5c117d203b0bc4dc39c91d7b95272f61f4e663baa477f27f2014de` |
| `desktop-final-unit.log` | `8e59817022c5c865bdc5c41333cbb0624e2de7fe78a837ba63ed70c1afb79e5c` |
| `vm-final-rebased.log` | `f3a58a613829c8e1400e2e40e336bc1739ab99594edfc5878e8b3bb94d0542be` |
| `rust-owner-roundtrip.log` | `dcebefde06e70f27865d8a5c882d77f241fe08facd2f7de00ca14b9a03254280` |
| `calendar-viewport-fixed.png` | `4fbe73ffc321a9041a104c830ad39a9857fa09d7d8ae79d167f7d893bf80b3a4` |

- stage: work | plan_id: PLAN-053 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-053/auto-ui | branch: codex/plan-053-dev | code_commit: 9a94d568778793700b2718ec0a949d60a0d38bcf | dependency_revisions: lang 61207576b7676148499854f9df4b2d019a476972 / down 7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c / web 37d32c22d1a4e7744710bf8f02f6c74d5a96c897 | acceptance_results: AC-01..04 pass | findings: none | next: final review
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-053/auto-ui + auto-lang | commit: UI 9a94d568 / lang 61207576b | next: review


### 最终 review r2（实施同会话 artifact audit，非独立上下文）

- reviewer_identity: 当前 Codex 实施会话；工具可用但未获独立 agent 授权，因此不冒称独立上下文。按 review skill 允许的同会话方式，从 Plan、提交 diff、原始日志与断言重建结论。
- baseline：UI `9a94d568778793700b2718ec0a949d60a0d38bcf`，base `426c1f708e7c966f243c86b1f38029ae47b0fa0a`；lang `61207576b7676148499854f9df4b2d019a476972`，base `a0750086c57c809a586884ac8b0519e8c162979b`；web `37d32c22d1a4e7744710bf8f02f6c74d5a96c897`、down `7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c`。两实施树 committed/clean；真实 worktree/branch 对账匹配，无第二 review 树。
- 检查矩阵：per-app client/store/component imports 与 URL 前缀；Rust/VM profile 分派；单 app launcher 不变、desktop 端口拒绝与 owned process 清理；有界 API route wrapper 与原 ext/i18n/无 glue 守卫；元素/else-if/三条 mint 遍历先序、属性事件/连续块及坏输入退出；core stacker 参数不变且漏入口补齐。永久合法/负例及 27 项实际调用链覆盖这些机制。
- 审阅新增对照：真实 host Notes `/api/notes` 与 Todo `/api/todos` 各 200；交换到另一 app 名下各 404，拒绝跨应用 endpoint 借用。原始 `review-route-controls.log` SHA256 `7e061fb287f40867e6a97a19782e1bf3c3f3aebe85ba715690667bc9a39a6939`。Calendar 截图核对可见月份、客户区与按钮；八操作原始 JSON 8/8、errors=[]、48API，不把注册或静态 inspection 充作业务通过。
- acceptance map：AC-01→T-01/T-02→host/backends/runtime→八应用真实操作 PASS；AC-02→T-00/T-02→view/parser→2 MiB 27项+永久最小反例 PASS；AC-03→T-01→vue/client/launcher→103 Vue、单 alpha哨兵、Rust53及owned退出 PASS；AC-04→T-03→source diff/hash+canonical paths PASS。
- SD-01/02/03 与实现、批准边界相符，无索引/ledger 写入。精确影响已写 frontmatter；三个 canonical source 冻结如下。

| Spec source | SHA256 |
|---|---|
| auto-ui:docs/specs/build/desktop-api-surfaces.md | `a92054257fd52e4da0be9059b59c58ec516b88771917602a2de8e40419313adb` |
| auto-ui:docs/specs/build/generated-vite-config.md | `c6e7be128ae678fa769b682a2661c177bd23aa16f7b8272080eebb897bdd9e99` |
| auto-lang:docs/specs/auto-lang/frontend/overview.md | `73912e5db10c82bac60c22fa04447ce173892f19d3b0703076469e394e73b401` |

- stage: review | review_scope: final | reviewer_identity: same-session Codex artifact audit; independent-context limitation explicit | plan_id: PLAN-053 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-053/auto-ui + auto-lang | branch: codex/plan-053-dev + codex/ui-plan-053-parser-fix | reviewed_commit: UI 9a94d568778793700b2718ec0a949d60a0d38bcf / lang 61207576b7676148499854f9df4b2d019a476972 | spec_inputs: frozen three-path SHA256 table above | acceptance_results: AC-01..04 pass | findings: none | pending_closeout_gates: language fixed-group parser; ff-only landing/canonical verification/archive/guarded cleanup; artifact stale observation; batch due check | next: merge
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-053/auto-ui + auto-lang | commit: UI 9a94d568 / lang 61207576b | next: merge


### merge preparation r2

- prepared：最终 review SHA 与当前 HEAD/clean 状态相同；UI master `426c1f70`、lang master `a0750086c` 均为各 delivery 的直接祖先，无需再改写；三个 canonical Spec hash 与 review 表相同。四个 group 树 `bash D:/autostack/wt-guard.sh <tree>` 均 clean。
- language merge-owned gate **PASS**：先 `cargo t --list`、`cargo t parser --select-only` 核对完整 parser 固定组（auto-lang/core 解析与语法集成、未加自选排除），再 `cargo t parser` exit 0。实际选择/执行 145，145 PASS，not_run=[]、excluded=[]、errors=[]；4027 为其它组未选择项，不计本组通过。准备 157.484s，执行 3.657s，nextest 断言阶段 0.689s，总计 161.270s。新 ui053 默认 worker 反例在该固定组内亦 PASS。
- gate identity：lang `61207576b7676148499854f9df4b2d019a476972`；config SHA256 `10d38c48ec83c707e0cf768a4ced9f516f32826e43d82a18dc305e76e723b47c`；source_state `4e699b58e3530319e0a3721f175a4ad2c01fcbcb63952020e168ccff110d5ad1`；`target/test-groups/run-q6ldrb2v/results.json` SHA256 `8fa5c65b9f60aa294a1cf603ce49a342295d4f315e76d81476a009fd64dfbb08`。`discovery_complete:false` 表示未发现整个 catalog，本次明确固定组的 145 选择与执行完整，并非 whole-catalog 通过。
- 最终测试宿主 PID 46152 经绝对路径核对退出，46253/六 Rust端口/VM5177 全关闭；只终止本计划测试进程。另一路 ui-060 和主检出正在运行的工具均未触及。
- 主检出状态：UI 及 lang 既有/并发文档 WIP 和 UI terminal_pixel 测试目录保留；无 tracked 源码 WIP、无本计划 staged 杂项。此目标只提交本计划代码/规范及其归档，不纳入他人文档。绝不以清空这些 WIP 取得表面 clean。


### merge landing / canonical / archive checkpoint r2

- landed：lang master `a0750086c → 61207576b7676148499854f9df4b2d019a476972`、UI master `426c1f70 → 9a94d568778793700b2718ec0a949d60a0d38bcf`，均 `git merge --ff-only`、无 merge commit。两个主分支 tip 与各 delivery commit 相等，tracked crate source clean。
- canonical_verified：主检出三个 Spec 实际内容与 reviewed worktree 逐字节相同，SHA256 完全匹配 final review 表；`git diff-tree --check <delivery>^ <delivery>` 均通过，generated-vite-config 的相邻 desktop Spec 链接可解析。完整 UI runtime/desktop tests、145 parser 与真实八应用 smoke 的 commit/dependency identities 与 landed 完全相同，复用同一有效运行，未以主分支名称重复跑测试。验证日志 `landing-canonical-check.log` SHA256 `53543749ee8983dfc35dd0e1845296c366336aae5a1ac8dc9dee938e74974118`。
- artifact observation（明确尚未部署）：主检出 `auto-ui/target/debug/auto-ui-tool.exe` 修改时间 2026-11-05 19:28:49，早于本次修复（测试用最新 bin 21:38:41）；主检出 UI release tool、lang release/debug auto 与默认 desktop-host 生成 dist 均未找到。本计划验证的是 scratch host 的最新 debug 实机，不声称现有主检出运行进程或发行包已刷新。受影响发行步骤为重建 UI tool（其中链接 lang parser）及消费方重新生成 Vue；主检出已有工具/其他计划进程未重启。8 app 快照中 vendor dist 未随源码复制，单独生产 bundle 探针报缺 engine dist；不把该非 dev 合同的打包输入探针算通过，也不修改业务 app 来规避。
- batch due check：auto-lang 主检出 `.last-batch-regression.json` timestamp `2026-11-05T12:21:00Z`、last_covered_plan_id=937，当前落地为 UI053 的最小 parser 修复，无新落地 %5 语言计划、距收据不足 48h；not due，不重复 whole catalog。既有批次 known_reds_only 保持原样，与本次 parser 145 PASS 区分。
- archival intent：`completion_kind: delivered`，从 active 路径移至 `docs/plans/archived/053-desktop-host-multi-api-surface-and-stack-fix.md`；T-00..T-04 5/6 done，T-05 仅 guarded worktree/scratch cleanup pending。archive/docs commit 与 cleanup 回执稍后补齐；没有未解决实施问题或证据 hold。


### cleanup recovery checkpoint（已交付，工具策略阻塞收尾）

- archived checkpoint：`50dfa8e6` 已在 master 提交真实归档文件，active Plan 不再存在。实际 gate 核对归档身份、status/completion_kind、5/6 task 状态、两个 delivery ancestry 与无相对坏链均通过。初版 probe 用原始 bytes 比较 Git LF 与 Windows CRLF 导致伪失败；按规范化文本重核后相同，没有文档内容缺失。首条组合命令未传播该 Python 非零码，已停止此类组合并在后续命令显式检查退出码；不得把初版 probe 记成 PASS。
- 已清理：四个树逐一 fresh `wt-guard.sh` clean、绝对路径/owner/clean status 核对后 `git worktree remove` 成功；UI `codex/plan-053-dev` 与 lang `codex/ui-plan-053-parser-fix` 均经 merged branch 检查删除。四仓 worktree 注册中无 ui-053，对应树已不存在；build intermediates 随树删除，未搬运。
- 未清理：空组目录 `D:/autostack/.wt/ui-053` 和整个 scratch `D:/autostack/plan-artifacts/plan-053`。工具执行自动审批两次拒绝：第一条为空组校验+清理及 scratch no-follow link 清理；第二条已缩为仅核验并删除显式空组目录。两次均在执行前返回 `Rejected ... blocked by policy`，没有提供更具体理由，不能通过换工具/换语法绕过。前面成功的 worktree 删除不意味着该最终目录删除已成功。
- 只读 no-follow inventory：scratch 92197 regular files、1145696660 bytes（约1.07GiB，不含链接目标），1446 symlink/junction entries；group_entries=[]。目录原始内容保留，没有盲删链接目标。registry `D:/autostack/plan-artifacts/.retention/policy.json` 记录具名 hold，owner=当前 PLAN-053 goal、reason=自动审批拒绝最终目录清理、closure=恢复该目录清理能力后 no-follow 摘本 scope 链接、全删 scratch 与空组、复核并提交 cleaned 收据。
- 恢复边界：只继续 cleanup / registry / final docs receipt，不重做实施、复审、145 parser 或八应用验收；若主分支后续前进，保留已交付历史，不重放旧 Spec。T-05 保持未勾选、current_step=5/6；代码与规范已交付，整个 goal 尚未全部完成。
- stage: merge | key: PLAN-053:r2 | outcome: blocked (cleanup only) | delivery_commit: UI 9a94d568778793700b2718ec0a949d60a0d38bcf / lang 61207576b7676148499854f9df4b2d019a476972 | checkpoints: prepared/landed/canonical_verified/archived done; cleaned partial | archive_commit: 50dfa8e6 | archive_path: docs/plans/archived/053-desktop-host-multi-api-surface-and-stack-fix.md | pending: empty group + scratch scope | next: restore deletion capability; cleanup recovery only
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 5 | from: merge | to: user | status: archived | outcome: blocked | worktree: removed all four; empty group remains | commit: UI 9a94d568 / lang 61207576b / archive 50dfa8e6 | next: user (cleanup tool policy)


- cleanup blocked audit / continuation 2：上一 goal turn 分类为 progress（已落地、归档并移除四树两分支，随后遇策略阻塞）；本轮只读复核两个目录仍存在且组为空，UI 主分支 `07ce7670`、lang `61207576b`，Q-053-02 无外部解除证据。未重试已拒绝操作、未改用其它工具绕过，未重跑通过的验收；当前没有可独立推进的剩余实施工作。相同阻塞连续第 2 个 goal turn，未达 host 的 3-turn blocked 阈值，goal 保持 active。
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 6 | from: user | to: user | status: archived | outcome: waiting | worktree: removed; empty group and scratch remain | commit: delivery UI 9a94d568 / lang 61207576b; archive recovery 07ce7670 | next: user (Q-053-02 cleanup policy)


- cleanup blocked audit / continuation 3：上一 goal turn 分类 no progress（只复核并记录同一状态，没有解除阻塞）；本轮再次只读确认空组与 scratch 均仍存在，Q-053-02 无解除答复或外部变化。相同策略阻塞已连续 3 个 goal turn；不存在已确认 live 的等待进程，也无可独立推进的剩余工作，达到 host blocked 阈值。停止自动重试；恢复目录清理能力后由用户 resume，仅继续已登记 cleanup。
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 7 | from: user | to: user | status: archived | outcome: blocked | worktree: removed all four; group/scratch cleanup pending | commit: delivery UI 9a94d568 / lang 61207576b; archive recovery 9e05bab3 | next: user (restore Q-053-02 cleanup capability, then resume goal)


### cleanup closeout / continuation 4（Q-053-02 关闭，6/6 完成）

- 清理执行：本轮以 reparse-tag 感知扫描（Python 3.14.2 `lstat().st_reparse_tag`；注意 `os.path.islink` 对 junction 返回 False，首轮 islink 扫描误报 0 链接并因穿透 junction 重复计数，已在任何删除动作前发现并纠正）复核 scratch，逐项吻合登记 inventory：92197 regular files、1145696660 bytes、1446 目录 junction。pre-delete 链接清单落盘 `.retention/receipts/plan-053-link-manifest-pre-delete.json`。
- no-follow 核验：`os.readlink` 解析全部 1446 个 junction，目标全部位于 `D:/autostack/plan-artifacts/plan-053` 内部（pnpm `node_modules/.pnpm` 内部链接），零外部目标——结构上排除穿透到任何仓库。随后逐个 `os.rmdir` 摘除 1446 junction（仅删链接本身），复查 reparse point=0，再删除剩余无链接常规树（92197 文件）与空组 `D:/autostack/.wt/ui-053`。
- 完好验证：plan-artifacts 其余 9 个 plan-* scope 与非 plan scope 在位（抽查 plan-054 内容正常）；auto-ui `git status` 与清理前会话快照一致；删除路径均在各仓之外。
- 登记/回执：registry `policy.json` plan-053 记录 → `archived` / scope deleted / holds=[] / disposition + inventory.verified / completed 2026-11-05，并追加 deletions、log（92197 files / 1145696660 bytes）、history 条目；closeout 回执 `.retention/receipts/plan-053-closeout-20261105.json`。恢复 hold（restore cleanup capability）由此回合解除；Q-053-02 关闭；T-05 勾选，6/6 done，goal 完成。
- stage: goal | plan_id: PLAN-053 | plan_revision: r2 | round: 8 | from: user | to: done | status: archived | outcome: completed | worktree: all removed (four trees, two branches, empty group, scratch scope) | commit: delivery UI 9a94d568 / lang 61207576b; archive 50dfa8e6; closeout docs commit this turn | next: none (6/6)

## 10. 待澄清事项

### Q-053-01（2026-11-05，已由用户允许最小跨仓根修关闭）

原 §3 只允许 auto-ui。迭代修复后，完整冻结候选 27 项仍溢出；CDB 已定位另一个通用解析器缺陷：`auto_lang_core::parser::Parser::{parse_body,parse_stmt,parse_stmt_inner,node_or_call_expr,args}`，由 UI `parse_on_block` / `generate_component_from_file` 消费触发。`parse_body` 已有 `stacker::maybe_grow(512 KiB,1 MiB)`，但 `node_or_call_expr ↔ args` 无对应入口保护，`parse_stmt_inner` debug 帧约 284 KiB。仅调大 worker 默认栈不满足原验收。

请求允许在同组 auto-lang 只读依赖树新增最小解析器根修及相应验证，保持目标与 AC 不变；或保持 auto-ui 范围、由上游独立计划交付后再消费。用户通过异步答复明确允许最小跨仓根修；当前执行合同已升 r2。完整 V-02 仍须真实通过，不能以授权代替验证。

证据：`D:/autostack/plan-artifacts/plan-053/stack-full-r4-debug.log` 567..622 行，首次/二次 c00000fd；先前 jade 单额外根 PASS（49.35s）不能替代真实 27 项嵌载调用栈。API 8 项生成在 2 MiB 上 PASS（22.19s）；真实启动首次为 Blog 冷后端构建超过既有 120s 就绪窗，失败可见且操作结束，正在单独预热该后端后重试。

编号 053 为立项时占用（撞号处理同 PLAN-052 §10）；G1 若评估为大型
架构改造（超出"最小前置"），回 auto-os/PLAN-054 §10 由用户裁量承载目标。





### Q-053-02（清理工具策略，已关闭 2026-11-05）

自动审批拒绝对已核验空组 `D:/autostack/.wt/ui-053` 的删除，返回 `blocked by policy`。代码/Spec/归档已交付，只需恢复最终目录清理能力后继续收尾；不需要再次批准跨仓根修。scratch 含1446目录链接，恢复后须先 no-follow 摘链接，再删除本 scope，不穿透外部目标。

关闭记录：本会话以 reparse-tag 感知扫描 + readlink 清单证明 1446 个 junction 全部为 scope 内部 `.pnpm` 链接后，逐个 no-follow 摘除、复查为零，再删除剩余树与空组；回执见 `.retention/receipts/plan-053-closeout-20261105.json` 与上方「cleanup closeout / continuation 4」。
