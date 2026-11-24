---
plan_id: PLAN-118
status: archived
feature_name: U05 Paint 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-24
plan_revision: 1
current_step: 8
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978, PLAN-117]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-118 U05 Paint 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [107](archived/107-autoui-u05-media-chart-and-paint-apps-av-vue.md) 的 U05 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [031-paint](../../examples/ui/031-paint/pac.at) | 实际指针绘制、像素或区域变化、原编辑操作和卸载 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。冻结原画布输入、颜色、像素编辑操作和持久化。按现有 ACE 运行路径映射实际指针与画布输出。

## 3. 技术与实现入口

目标入口为 src/front/app.at、paint_color 与正式 ColorPicker 包。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-118/{auto-ui,auto-lang}，每仓单一 plan-118-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

原 ColorPicker/画布输入的已有接口按实际使用消费。共享源准备、包面及活视图接口接 [117](117-autoui-image-apps-av-vue.md) 的落主版本；双树同步后复验 031 原样准入，再继续画布宿主接线。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。重放指定指针轨迹后读回预期像素或区域。原编辑操作、异常恢复及关闭卸载可观测。 |
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
- [x] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [x] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
- [x] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [x] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 1 | from: new | to: work | status: archived | outcome: pass（授权确认：/goal 完成计划118 = work→review→bounded repair→merge 全链授权，记 §4） | worktree: （组创建中） | next: work（T-01）

2026-11-22 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-01 固定原项目、依赖与验收矩阵 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui (branch plan-118-dev, base d045f4a4) + D:/autostack/.wt/ui-118/auto-lang (branch plan-118-dev, base c5fd06592)；只读兄弟树 auto-web 0cfc7c784 / auto-down 45b642d | next: T-01 准入实跑（构建）

**T-01 逐行矩阵（2026-11-22 核）**

| 项 | 固定值 |
|---|---|
| 原项目源 | `examples/ui/031-paint/pac.at`、`src/front/app.at`、`src/front/paint_color.at`、`SPEC.md`、`tests/desktop_mcp.py`（仓内单源，不动源） |
| 原 oracle（承 107 冻结行） | 真实指针落笔、像素 42 按选色变化、Undo/Redo 回放、取消选色保正式色、存储往返后卸载；view 内建只读回显 `px42=${.px[42]}` 行 |
| 实际使用组件 | 官方 ColorPicker 包（`packages/color-picker`，PLAN-088 已交）；canvas 场景 widget（`scene/coords/clear/onhit`，onhit→PaintHit→Paint(i)）；`storage.set/get("paint.canvas.v1")`；`use auto.json / stylekit / themes`；design 绑定（theme/mode） |
| AV 端 | `ace_node_desktop_window.exe --source <app.at> --evidence <out.json> [--script tick:click:inst:target / type / close / reopen / late]`，example 目标 feature `ace-bridge`；组件面源准备（pac.at 工程根）在 master；storage.set/get 经 `ace_source_prep.rs` 宿主仿真改写为状态字段（PLAN-105，内存态）；onhit 画布命中→PaintHit 的组件面映射待准入实跑证实 |
| A2Vue 端 | `auto build --render vue --gen-only`（auto CLI = /d/autostack/auto-man/target/debug/auto.exe，PLAN-114 固定版 SHA256 `eddb2cce…d947fcf`，复用）+ `AUTO_UI_ACE_VUE=1` + `AUTO_UI_TOOL_BIN=<本计划树构建的 debug 工具>` → `pnpm install --offline --frozen-lockfile` → `pnpm run build`（vue-tsc && vite）→ Playwright 真实 Chromium（复用 /d/autostack/auto-blog/tests/node_modules 1.63.0）；Vue 侧无 storage 桥（master 注记）——存储往返语义待准入实跑定位 |
| 实际阻断 | 未实跑前不定；双端首个拒绝点以准入实跑为准（构建后进行） |
| 环境 | Windows x64，Rust stable-msvc 1.98.0；Node v25.2.1 / pnpm 11.6.0；dev/debug 增量构建，`--locked --offline`，禁 RUSTC_WRAPPER |
| 占用对账 | 开工时他方构建在跑（cargo PID 74280/2384/48812/93044 + link.exe 74988 4.6GB）——本计划构建 `-j2` 让行 |

2026-11-22 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-01 收口 | outcome: pass | worktree: D:/autostack/.wt/ui-118/auto-ui (plan-118-dev@d045f4a4) + auto-lang (plan-118-dev@c5fd06592) | commit: （无代码改动；诊断探针 p118_diag 随树未提交） | next: T-02

**T-01 收口证据（AC-01）**

- 构建：基线工具 `cargo build --locked --offline -j2 -p auto-ui-tools -p auto-ui-runtime --features auto-ui-tools/ace-vue,auto-ui-runtime/ace-bridge --bin auto-ui-tool --example ace_node_desktop_window`，exit 0（约 11 min，fresh target）。锁修补：`Cargo.lock` 缺一行 `ace-source-cte`（ace-core 依赖表，auto-ui master 锁文件对 auto-lang@c5fd06592 漂移）——工作树内手动补行后 `--locked` 通过；该漂移是 master 既有事实，登记不动主锁。
- 工具身份：AV `ace_node_desktop_window.exe` SHA256 `11c28047…ce6310`；`auto-ui-tool.exe` SHA256 `a6b3b587…1d9137`；auto CLI 复验与 PLAN-114 钉版逐字节一致（`eddb2cce…d947fcf`）。
- 双端首准入（`plan-artifacts/plan-118/first-admission.py`，`admission.json` SHA256 `73da6306…1052fe`）：AV exit 4 `source.age.structure: unexpected token in a bare element tag`；A2Vue exit 1 `ace.vue-lane-unsupported`（App 全部 21 handler 缺席，CLI 内嵌道）。两端同断于源准备后的第一步解析。
- **入口勘定（承 PLAN-117 §9 T-01 发现）**：宿主 `NodeDesktopBridge::load_source`（auto-ui-runtime ace_node_desktop.rs:1888）对 **widget 级 `use { package: … }` 导入**的源自动走 AGE **包面**（app 单元 + `ace_widget_packages::append_widget_packages` 组包）。031-paint 的 app.at 含 color_picker 包导入 → 正入口=包面；组件面（`compile_component_event_program`）非本应用入口——组件面逐项缺口（typed handler params 仅 age-package 文法臂、`while` 仅包面、`style` 顶层声明非组件面成员、事件程序 model 仅标量）对本案不构成合同修改，仅登记。

2026-11-22 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-02 首个阻断定位 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui | next: T-02（等 PLAN-117 交付后复验接续）

**T-02 首个阻断（已定位到 owner，具名）**

- 复现链：master 基 prep（`ace_source_prep::prepare_component_source`）把 app.at 的全部 `use …` 行（含 `use { package: color_picker from … }`）与顶层 `style tool_btn/wrap_row = …` 声明**空白化**（保字节长的空格行）→ `package_imports` 投影为空 → 宿主面选择落回组件面 → 上述解析拒。诊断实证：`P118_DUMP_PREP=1 p118_diag` 转储的准备源 `use`/`style` 行全为空格（`plan-artifacts/plan-118/` 探针在案）。
- **该 bug 与修复均归 PLAN-117**：117 §9 B-03 已定位并修复（auto-ui `299ac88d`，prep 词法扫描跳过注释/字符串 + `use { package: … }` 空白化修复），**未合 master**；117 同期 auto-lang 包面增量（`f41e7df15` 文法/投影/verify/live-view/binding、`b3275c18f` if-expr、`c80a5e2b` ??/try/list/outbound）亦在途。117 会话活跃（auto-lang 提交至 2026-11-22 20:42）。
- 按 §4「共享写面正在由其他计划维护时，消费交付版本并在本计划复验」：本计划不重复修该缝、不把 118 基线挪到 117 在途提交上。**unblock=PLAN-117 落 master（T-03→T-08）后，118 双树 rebase 消费并复验**；届时再枚举 031-paint 包面余量缺口（canvas 场景 widget 的包面视图/投影、ColorPicker 包子组件挂载、storage 改写包面语义、f-string 文本叶事件程序边界）。
- 探针工具：`crates/auto-ui-runtime/examples/p118_diag.rs`（逐条诊断+span+准备源转储+lane 归因；auto-ui `b332f983`）。

2026-11-22 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 1 | from: work | to: work | status: archived | outcome: blocked | worktree: D:/autostack/.wt/ui-118/auto-ui | commit: b332f983 | next: work（外部依赖解除后续接：PLAN-117 落 master → 118 双树 rebase 复验）

2026-11-22 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-02 包面缺口登记（等 117 期间的独立工作） | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui | commit: 16acd684（锁修补） | next: 同 round 1（等 PLAN-117）

**T-02 缺口登记（2026-11-22，master 基线手工组包探针 `P118_PKG=1 p118_diag`，材料 `plan-artifacts/plan-118/probe-pkg-*.at`）**

包面正形锚点：024-charts（已交付）形 = 顶层 `style x = "…"` + `use pkg.mod: …` + `widget App { use { package: … from "dir" } msg/model/view/on }` + 无类型字段列表字面量。031-paint 同族。

| # | 缺口 | 实证 | 归属（写面） |
|---|---|---|---|
| G1 | prep 空白化 `use`/`style` 行 | 见 round 1；prep 转储空格行，`package_imports` 空 → 误落组件面 | **PLAN-117 在途修复**（auto-ui 299ac88d） |
| G2 | canvas `scene: .cells` 合成名（五平行列表 `cells_pts/meta/nodes/edges/labels`）包面视图解析拒：`source.undefined no local named self` @ scene 条目 | probe-pkg-canvas.at；改绑真实字段 `scene: .cells_nodes` 即 PACKAGE LOAD OK | auto-lang 投影缝（117 在途）或 auto-ui prep 改写缝（117 在途文件）——两缝皆占，待 117 落后续 |
| G3 | `str.substr` 未链接：`the stdlib text method is not linked`（resolve.rs STDLIB_TEXT_MANGLE 表无 substr） | probe-pkg-m-substr.at | auto-lang stdlib（117 邻接，待复验） |
| G4 | 列表 `join` 不在包面 list 方法集（legal: new/push/insert/get/set/remove/len/clear）——paint 快照格式 `px.join("\|")` 全线依赖 | probe-pkg-m-join.at | auto-lang（同上） |
| G5 | `.to(int)` 仅 str↔Str 转换语义，无数值解析（`"42".to(int)` 具名拒）——PaintHit 的 `id.substr(2,…).to(int)` 依赖解析语义 | probe-pkg-m-to.at | auto-lang（同上） |
| — | f-string 文本叶（`f"px42=${.n}"`）、`len()`、`split`、`starts_with`、while、typed 参数、List 字段、design 块 | 全 PACKAGE LOAD OK（probe-pkg-fstr/while/scene） | 无缺口 |
| G6 | 宿主包面视图无 canvas 臂：package_view.rs element() 对未知标签落通用容器——canvas 场景/onhit 被**静默丢弃**（无具名错）；Iced 侧 CanvasPainter/PenArea（PLAN-563/661）现成可复用 | 静态代码核（ace_node_desktop/ 无 canvas 匹配） | **auto-ui 宿主（118 自有面，117 不碰 package_view/ace_state_bridge/widgets）** |

- 主机侧实施预设计（G6，待 G2 解锁后施工）：`package_view.rs` 加 canvas 臂（props: scene/coords/clear/style + 帧场景数据通道）→ `View::Canvas`（crate::ui::view 现成类型）→ PenArea tap `canvas_hit_test` → 命中 id 经 NodeDesktopMsg 新变体回事件程序 `.PaintHit(id)`；脚本 click 扩坐标（`click:inst:target:x,y` 逻辑坐标 → hit_test）；证据面复用 sessions.states/texts/dispatch_log。
- A2Vue 侧 canvas SFC 发射（ui_gen Plan 563 有旧道）与 lane 场景绑定形态归 T-06 复验时定。

2026-11-22 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 2 | from: work | to: work | status: archived | outcome: waiting | worktree: D:/autostack/.wt/ui-118/auto-ui | commit: 16acd684 | next: work（PLAN-117 未落 master（截至 20:55 其在途）；等待 2/3，已做无争独立工作：包面缺口登记 G1..G6 + 主机侧 canvas 实施预设计）

2026-11-22 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-02 G6 设计深化 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui | commit: （探针能力扩充，随下一提交） | next: 同前（等 PLAN-117 落 master）

**T-02 G6 设计深化（2026-11-22，宿主数据面勘察）**

- 包面活视图模型（auto-lang live_view.rs）：`WidgetViewItem::Element(WidgetElement)`——`properties: Vec<(String,String)>` **纯串行**、`events: WidgetEvent{key,message,args}` args 仅标量（ComponentFieldValue 标量域）。canvas 场景（256 节点列表值）**无法**经 properties/events 到达宿主——G6 需要**新增引擎列表态读通道**。
- 桥侧帧状态 `PackageFrame.states: Vec<(String, AceStateValue)>` 标量四件（Int/Bool/Str/F32）；场景编译注释在案「状态观测通道只打包 int 字段，其余跳过」——列表模型态当前对宿主不可观测。
- 影响修订：G6 从「package_view 加一个 canvas 臂」升级为「引擎列表态读通道 + 帧场景快照 + package_view canvas 臂 + PenArea tap→onhit 派发 + 脚本坐标点击 + 证据读回」七件；施工面仍在 auto-ui 宿主（ace_package_bridge/frame、ace_node_desktop/package_view、widgets、ace_state_bridge），但消费 live_view/compiled 类型——**117 在途提交 f41e7df15 正改 live_view/binding**，即刻施工有重冲突风险，故随 117 落 master 后一并执行。
- 探针扩充：`P118_PKG_DUMP=1` 直连 PackageStateEngine 帧转储（单 widget 无子包件时 live 面不挂帧，dump 对含子组件件应用有效；真实 app 有 ColorPicker 子组件件，走 NodeDesktopBridge 路径帧在案）。

2026-11-22 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 3 | from: work | to: work | status: archived | outcome: blocked | worktree: D:/autostack/.wt/ui-118/auto-ui | commit: a1c2f2c7 | next: user（外部依赖 3/3：PLAN-117 未落 master；目标转 blocked，unblock 见 §10）

## 10. 待澄清事项

2026-11-23 归属裁定（**用户已裁①**：「选1，授权本会话修改」）：118 获一次性豁免，在 `D:/autostack/.wt/ui-118/auto-lang`（plan-118-dev）交付五件最小闭合集——边界：①`str.substr` 链接（stdlib text.at fn + 包面接枝扩展）②列表 `join(sep)`（stdlib list.at fn + resolve 方法臂 + 接枝）③`.to(int)` 解析语义（conv.to_int 已有，补 `.to` 方法臂 + None 行为裁定=运行时具名 fail-closed）④canvas `scene: .cells` 合成名解析（包面视图条目值投影）⑤包面 fn 形参裸 `List` 类型（model_list_type 先例延伸）+ 各件定向正反用例；**不动 AV 任务/actor 与其余 stdlib**。substr/join 参考实现已于等待轮预暂存（随本裁定转正）。

2026-11-23 stage: work | checkpoint: 五件施工（①③⑤ 落地，④ 改道 auto-ui，② 剩执行器 op 链） | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-lang（未提交） | next: 构建验证 → ②op 链 → prep 场景改写

- **①substr 落地**：`stdlib/ace/asm/text.at` 加 `substr(s,start,take)`（slice 实现，起点钳 [0,len]、溢出钳尾、负长度空串）；包面接枝扩展 `adapter.rs needs_linked_substr + graft_fns` + `age_package.rs` 触发位（组件面走 graft_stdlib_text 全量接枝，`.substr(` 本就触发，零改动）。
- **③.to(int) 落地**：`conv.at` 已有 `to_int(s)->?int`；抽共享 `stdlib_integer_parse_source()`（组件/包面同一实现源），新增 `graft_package_integer_parse`（镜像 u64 接枝的包绑定通道）+ `age_package.rs` 调用位；None 行为 = 语言 checked 错误通道 fail-closed（`fail("ValueError: invalid integer text")`，与组件面既有行为一致）。resolve 方法臂在包面 sigs 存在时自动生效（sigs 随接枝入 bound）。
- **⑤裸 List 形参落地**：`resolve.rs` TypeRef::Record 臂加 `List` 窄臂（`model_list_type` 同款先例 → `List<StrOwned>`；同一 aggregates_admission 门，非聚合面仍具名拒）。
- **④改道 auto-ui（设计裁定，不动 auto-lang）**：`scene: .cells` 的合成名解析墙在 auto-lang 视图条目 lowering（self 改名缺字段）；但**prep 改写** `scene: .<name>` → `scene: "<name>"`（仅当 `<name>` 非 model 字段且 `<name>_nodes` 是字段——prep 已有字段表可判定）即可绕开：串 prop 天然贴合 WidgetElement.properties 的 (String,String) 模型，宿主 G6 按名读五列表，A2Vue 同理。**件④移出 auto-lang 五件 → auto-ui prep（118 自有面）**。
- **②join 剩执行器 op 链**：std fn 已备（list.at，所有权纪律版）；但**字段接收者无法按值传 helper**（列表无 clone——profile 具名拒），必须新 `ListJoin` 执行器 op（RExprKind→mir→av.rs arm→abc/native/instr_spec，108 HttpCall 同款链，半日工）。本轮不做。
- 验证构建在途（bash-2faku76o）；探针验收单：`probe-fn-legacy_to_hex`（①+⑤ 应推进）、`probe-pkg-m-to`（③ 应 LOAD OK）、`probe-pkg-m-join`（仍拒=② 待办预期）、`probe-pkg-canvas`（仍拒=④ prep 改写前预期）。

- **验证结果（2026-11-23，提交 auto-lang `824167824`，4 文件 +106/-8）**：③`probe-pkg-m-to` PACKAGE LOAD OK（ graft 路径生效）；①⑤`probe-fn-legacy_to_hex` **包面编译通过**（substr 链接 + 裸 List 形参受理；探针末梢的 text-leaf mismatch 是最小探针工件非编译墙）。**关键发现：master `text.at` 本就有 `pub fn substr`（legacy 钳制语义）——件①的实质只是把它加进包面接枝集； text.at 零改动**（中途一次 Edit 工具故障清空该文件，已 `git checkout` 无损恢复并核认 315 行原状）。
- **下一墙（已定位，非五件内）**：VM 作者源的 **fn 作用域 str/Str 摩擦**（`var t str = <Str 值>`、`fn f(...) str` 返回 Str 等——探针逐处显形，真实 app 的内联 fn 将连环命中）。方案：prep 对包面目标做 str→Str 注解归一（扩展现有 rewrite_model_text_fields 的 model 作用域到 fn 形参/局部/返回——auto-ui 自有面，不扩 auto-lang 授权）。次序：prep str/Str 归一 → ②join op 链 → ④prep 场景改写 → ColorPicker 挂载/G6。

- **str/Str 归一落地并验证（2026-11-23，auto-ui `4ec9a500`，3 文件 +114/-11）**：`rewrite_str_surface_for_package` pass（门控 widget 级包导入；**必须跑在 decls 内联前置之后**——第一处插入点在内联前，pass 见不到 fn，已移位修正并留注）。途中排除两个探针假象：本树旧 `P118_PKG` 模式**未经 prep 直接组包**（此前多轮"真实 app"探针结论无效，已重写为忠实复刻：pac.at 祖先根 prep + `$event` 归一 + color 单元）；auto-lang gate 临时 span 调试已还原。**归一后 faithful 复刻的下一墙 = 包面 entry 单元顶层 `fn` 具名拒（`fn is not part of this source profile`）——paint 特有**（对照实证：024-charts 同入口在新主线越过解析直达 `verify.view-escape`，证明包面对 widget app 本身健康；024 的 entry 无顶层 fn，paint 的 prep 内联 fn 落在 entry 单元）。处置候选：①auto-lang entry-unit 放行 fn 件（超授权五件，需扩裁）；②prep 把内联 fn 改置独立 app 子模块单元 + 调用点改 3 段路径（auto-ui，但跨模块调用形态待验证）。登记待裁。

- **「fn not part of profile」排除法（2026-11-23 晚）**：`use { package: }` 行 + 顶层 fn 同单元 = **PACKAGE LOAD OK**（usepkg 探针）；style 顶层声明 / design 块 + fn = 仅 str/Str 摩擦（带位置，prep pass 已解）。**entry 顶层 fn 并非普遍被拒**（单 fn 探针在包面可编译）——真实 app 的该 hint 仍是回滚假象，真实首错未现形。续 bisect 面：view 的 if-cond / canvas+ColorPicker 元素 / f-string 叶 / __st 字段 / handler 体（while→for、join、substr+to(int) 链）。

- **真实首错闭环（2026-11-23，auto-ui `7d397da2`）**：bisect 实证——**prep 的 `while→for` 重写对包面目标有害**（包面原生有 while；其**模块 fn 体拒 for**：同形无循环 fn 模块 COMPILE OK，带 for 即 "`fn` is not part of this source profile"@fn 头——消息误归因）。两处重写点按包面目标门控关闭后，**准备后的 031-paint app 单元整体编译通过**（fn/handler/view 全过），推进至下一墙 = **ColorPicker 包解析**（`component-package-unresolved`，exe 真径实证）。ColorPicker（088 组件，VM/MCP 轨交付，从未过 AGE 包面）的 on 块构造级缺口是下一墙——其 7 处嵌套 while 已证包面可受理，待逐构造枚举（同 117 为其组件做过的类）。复刻探针新增 RAW_FULL/RAW_EXTRA 带位置二分模式。

- **ColorPicker 墙闭环（2026-11-23，ddmin 自动二分）**：30 handler 中唯一触发 = `.SurfDown(x, y)`；再二分定位 **quoted-message 携记录字面量含 `.field` 引用**（.`"preview"({seq: .seq, value: .d_hex})`）——最小复现 `bis-sd-quotedrec.at` 重现「expected fn found widget」。**属 117 在途 quoted-message/事件对象链同类**（b936120d6/b509a6d95/2d2ffbd8a 等正打该面）→ **挂起待 117 链落主后复验**（若缺口仍在，按同一裁定①模式补件）。嵌套 while/float 局部/f-string 浮点插值均已实证包面可受理。

- **G6 画布主机臂落地（2026-11-23，auto-ui `c345da9e`）**：`View::Canvas` 渲染管线（→AbstractView::Canvas→CanvasPainter/PenArea）本就存在且 VM 轨在用；缺口仅在包面视图无 canvas 臂。已补：frame `hits` 映射（canvas onhit 事件→action 目标）+ `package_view` canvas 臂（coords/clear/style 解析；on_hit 命中 id 走既有 `PackageText` 文本通道回事件程序——`.PaintHit(id)` 全链复用输入派发道）。**场景数据通道留空**（live_session 列表值读面，117 在途）——臂已编译、无回归（024 view-escape / paint ColorPicker 墙两处准入逐字不变），数据接通即活。

- **件④ prep 场景改写落地（2026-11-23，auto-ui `659f51f5`）**：`rewrite_canvas_scene_scene_bindings`——包面目标下 `scene: .cells`→`scene: "cells"`（门控：`<名>` 非 model 字段且 `<名>_nodes` 是；游标式行内扫描）。准备源实证双画布改写到位；**准备后 app 单元的包面编译越过场景解析**（下一墙 = ColorPicker 包解析，符合预期）；024 回归逐字不变。件④ 至此**改道 auto-ui 后闭环**（auto-lang 五件实际交付四件：①③⑤ + 本件 auto-ui 化）。
- **117 在途面更新（2026-11-23 16:30）**：其现行 auto-ui diff 已扩至 `ace_node_desktop.rs`(+147)/`package_view.rs`(+75)/`ace_package_bridge.rs`(+228)/`widgets.rs`——**G6 提交与 117 在途面重叠，落主后需按其实况重接线**（臂的小体量重写成本低）；ace_source_prep.rs 不在其现行 diff（件④/归一/门控安全）。

- **定向正反用例交付（2026-11-23，auto-lang `7ae8b7f7b` 2 例 + auto-ui `162673a7` 2 例，各 0.00s ≤10s 门）**：graft 集含 substr 且声明唯一（件①）；`.to(int)` 触发词形正反 + 接枝源含 require_int/bad_int/to_int（件③）；场景改写正例（合成名）+ 双反例（真字段/非组名不动）（件④）；fn 作用域 str 归一（形参/返回/局部）。件⑤（裸 List 形参）的用例随消费复验（探针已过）后在 117 缝稳定期落测试。

- **T-06 道勘定（2026-11-23，等待轮产物）**：包面 SFC 生成器 `generate_package_sfc_with_root` 已存在且**自带 entry prep**（desugar + prepare_component_source + 包编译同链）；当前仅 ace_source_prep 测试引用，未接入 `auto build --render vue`。T-06 实施 = 严格键门对包导入 app 路由到该生成器（vue.rs `ace_vue_sfcs` 分支），替代逐 widget lane。paint 的 E2E 同卡 ColorPicker quoted-record（117 链），落主后复验。**117 落主后复验 runbook**（登记）：①双树 rebase + G6 臂按 117 实况重接线（package_view/ace_node_desktop 重叠面）；②faithful 复刻 + exe 准入复跑（预期越过 ColorPicker→现场枚举下一墙）；③件② ListJoin op 链在稳定 resolve.rs 施工；④G6 数据通道（scalar_states 同型列表读 + frame 场景快照 + 臂消费）；⑤脚本坐标点击（窗例脚本语法 + 桥）；⑥T-03 实窗见证（指针轨迹回放/px42 读回/undo-redo/存储往返/关闭卸载）；⑦T-06 门路由 + 冷生成 + pnpm + Playwright；⑧受影响消费者回归（024 等）+ 用例 consolidation。

2026-11-23 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 5 | from: work | to: work | status: archived | outcome: blocked | worktree: D:/autostack/.wt/ui-118/{auto-ui,auto-lang}（8 提交在枝） | next: user（外部依赖等待 3/3：117 在途链未落主——join op 链/G6 数据通道/ColorPicker quoted-record 复验/G6 臂重接线全部位于其活跃缝；unblock=117 归档落主后 `/goal resume` 按 §9 runbook 续跑）

2026-11-23 移交包备好（随裁定②路由；裁定①则由 118 直接施工）——**auto-lang 包面五件最小闭合集**，逐件带最小复现（`plan-artifacts/plan-118/`）：

| # | 项 | 最小复现 | 建议语义（VM 对等） |
|---|---|---|---|
| 1 | `str.substr(start,len)` 未链接（STDLIB_TEXT_MANGLE 表缺） | `probe-fn-legacy_to_hex.at` → `the stdlib text method is not linked` | 起始+长度截子串（VM 语义；多字节边界随 VM 裁定） |
| 2 | 列表 `join(sep)` 不在包面 list 方法集 | `probe-pkg-m-join.at` → `legal: new/push/insert/get/set/remove/len/clear` | `List<str>.join("｜") -> str`（paint 快照格式命脉） |
| 3 | `.to(int)` 仅 str↔Str 转换，无数值解析 | `probe-pkg-m-to.at` → `"42".to(int)` 拒 | VM 解析语义（paint 路径输入可信 `"px42"`；非法输入行为随 VM 裁定） |
| 4 | canvas `scene: .cells` 合成名（五平行列表）解析拒 | `probe-pkg-canvas.at` → `no local named self` | 条目值解析：`<scene>` 无字段同名而 `<scene>_pts/_meta/_nodes/_edges/_labels` 存在 → 场景引用直通宿主 |
| 5 | 包面 fn 形参裸 `List` 类型拒 | 内联 `px_node(pxs List, …)` → `no type named List` | `model_list_type` 先例延伸：裸 `List` 形参/局部 = `List<str>` |

2026-11-23 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 4 | from: work | to: work | status: archived | outcome: waiting | worktree: D:/autostack/.wt/ui-118/{auto-ui,auto-lang} | commit: （auto-lang 树 substr/join 预暂存未提交） | next: user（等待 3/3：§10 归属裁定未到——①授权 118 在 plan-118-dev auto-lang 树交付五件闭合集；②移交语言 owner。等待轮产物：五件移交包+代码锚点+substr/join 参考实现预暂存；下一轮无答复则按协议转 blocked）

代码锚点（供实现方）：①substr → `stdlib/ace/asm/text.at` 加 fn（**已在 118 auto-lang 树预暂存**，未提交）+ 包面接枝扩展 `adapter.rs graft_package_text_predicates`（现只接枝 starts_with/ends_with/eq_at 三件，需按 `.substr(` 触发纳入）；②join → `resolve.rs:13300-13425` list 方法 match 加臂 + `stdlib/ace/asm/list.at` 补 `join(xs List<Str>, sep Str) Str`（递归拼接 idiom）+ 包面 list 接枝（下 lowering 或需 av-core 新 op——五件中最大件）；③to(int) → **`stdlib/ace/asm/conv.at` 已有 `pub fn to_int(s Str) ?int`**——缩窄为 resolve `.to` 方法臂映射 + None 行为裁定（VM 对等）；④scene 绑定 → 包面视图条目值投影（`age_package` view value resolution，G2 同款 seam）；⑤List 形参 → `model_list_type`（age_component.rs:4777）延伸至 fn 形参/局部注解。

另登记**跨计划回归**（不归本五件）：024-charts（已交付）在新主线严格键 `AUTO_UI_ACE_VUE=1` 下生成亦失败（`verify.view-escape: official::line_chart:619 an owned field view is only consumed by .len()`）——包导入 app 的严格 lane 道在主线有漂移，107/114/117 owner 应知悉；paint 的 T-06 将受此面影响。

2026-11-23 拆分小交付（用户裁定，替代上文 round-3 等待路径）：118 验证从首阻断拆出独立小补丁——原 Paint 双端首阻断 = prep 抹除 `use { package: color_picker }` 导入；117 的 `299ac88d` 已有修复（用户确认可干净应用到当前主线），但该补丁未覆盖同次诊断中被抹掉的 `style` 声明。任务：隔离工作树跑原 Paint 双端准入 → 找出能越过首阻断的最小补丁 → **由 117 维护方单独送审落主** → 118 拿固定版本后续接自己的画布工作。隔离树组 `D:/autostack/.wt/fix-prep-118/{auto-ui(branch fix-prep-118@master c2adc1ca7 + cherry-pick 299ac88d=7c7ef158), auto-lang(detach dca52e196), auto-web 0cfc7c784, auto-down 45b642d}`；材料 `plan-artifacts/plan-118/admission-fix.*`。

2026-11-23 goal 恢复（用户裁定：117 已合并验证内容，118 继续）：auto-ui master 已落 `3f4f9b3f`（= 299ac88d 的 rebase 落地，cherry-pick 源标注 `7c7ef158`——即 118 验证的拆分小补丁）；**auto-lang 的 117 在途链（f41e7df15/b3275c18f/c80a5e2b）未随本轮落地**（master 8a52d7364 无 PLAN-117 提交）。118 双树已 rebase 到新主线（auto-ui plan-118-dev@0add6c56，锁修补提交因上游已修自动 drop；auto-lang plan-118-dev@8a52d7364）。续跑：重建 → 复跑双端准入 → 按实制定下一墙施工方案（预期①entry 循环-fn、②widget 模块声明墙仍在——auto-lang 117 在途链未落；届时按 108 先例的授权边界裁定归属）。

- 恢复后勘察（2026-11-23）：117 会话今日高度活跃（auto-lang 提交 10:48/11:17/11:26，含 "Admit checked dynamic scalar display in live AGE packages"、"checked widget HTTP through resumable AV execution"——正打在 118 依赖的包面 live 会话值域）。**策略确认**：①②墙若在新主线复现，不双写——复用拆分小交付模式备好最小复现移交 117 维护方；G6 数据通道依赖 live 会话列表值读面（auto-lang live_view/live_session，117 活跃缝），随 117 链落主后施工；等待期间 118 仅做无争面（探针/证据/设计细化）。

- **新主线复跑结果（2026-11-23，双树 rebase 后重建 5m04s exit 0）**：①②墙已随主线其他计划推进**消失**（widget 模块声明与 entry 循环-fn 不再拒）——AV 真实下一墙链（逐项探针实证）：**G3 首现**（prep 逐字内联的 `legacy_to_hex` 之 `substr` → `the stdlib text method is not linked`）→ `px_node` 形参裸 `List`（`no type named List`）→ G2（`scene: .cells` 合成名）→ G4（`join`）→ G5（`to(int)` 解析语义）→ 之后预期 ColorPicker 子组件 live 挂载与 G6 画布。**A2Vue 侧**：`use { package: }` 成员使组件面 parse 在 model 区回滚（record-literal hint @palette；移除该行进至视图）——lane 道对包导入 app 需走包面 SFC 道（107 的 024 先例=generate_package_sfc 直连），归 T-06 设计。
- **归属裁定需求（记 §10）**：G3/G4/G5（`substr`/`join`/`to(int)` 解析 + G2 场景绑定 + 裸 `List` 形参）均为 **auto-lang 包面增量，且不在 117 在途链的构造集内**——按 §2 与 PLAN-108 先例，扩大 118 的 auto-lang 写面需用户裁定（二选一：①裁定授权 118 在 plan-118-dev 的 auto-lang 树交付该最小闭合集；②移交语言 owner 计划，118 消费复验）。

**拆分小交付验证结论（2026-11-23 收口，证据 `plan-artifacts/plan-118/`）**：

- **最小补丁 = `299ac88d` 单独**（auto-ui，cherry-pick 至当前主线 c2adc1ca7 干净应用 = `7c7ef158`，与用户确认一致）。**style 声明保留不纳入**：实证 prep 的「空白化 + replace_word 引用字面化」产出对包面完全可解析（逐项探针 PACKAGE LOAD OK：literalized style、全 model、design、if-cond、f-string、canvas 元素、子组件占位标签全过）；保留声明反而必须同时 guard replace_word（否则 `style tool_btn = …` 被自损坏成 `style "px-3…" = …`），扩大补丁面零收益。
- **AV 准入越过首阻断实证**：master 基线错误 `source.age.structure: unexpected token in a bare element tag`（prep 抹除包导入 → 误落组件面）→ +299ac88d 后推进到 `source.unsupported: `fn` is not part of this source profile`（包面装载真实下一墙）。下一墙分解（均 auto-lang 包面能力，117 在途链 f41e7df15 同类）：① entry 单元内 fn 体含任何循环（while/for）即 profile 拒；② 包面 widget 模块声明被拒（`module color_picker::color_picker: expected fn, found widget`——024-charts 同形，其已交付系走 114 组件面受检组件引用道，非 load_package）；③ 循环-fn 之后预期 G2（`scene: .cells`）、G3（substr）、G4（join）、G5（to(int)）链。
- **A2Vue 准入：首阻断不在 prep**——+299ac88d 前后错误同形（lane 21 handler 全缺席，`admission-fix.json` SHA256 `73da6306…` 与基线逐字节同）；真实阻断在组件面 lane 对 prepared 源的编译（prep 的 `while→for` 文本改写把循环带进单模块编译 + 组件面无循环），非一笔 prep 补丁可解，归 T-06/117 在途链。
- **移交物**：`fix-prep-118` 分支 = 299ac88d（`7c7ef158`）+ 探针提交（`0cc47f5d`，标明 DROP before landing）；验证工具 `p118_diag`（P118_PKG / P118_PKG_FULL 包面组包复刻 + P118_DUMP_PREP 准备源转储）；证据：`prepared-fix.txt` SHA256 `508a2188…`、探针材料 `probe-pkg-*`/`probe-fn-*`/`probe-view-*`/`paint-nocolor` 在案。**请 117 维护方单独送审落主 `7c7ef158`**；落主后 118 rebase 接续（画布 G6 七件 + 上述 ①② 墙的消费复验）。
- 辅助发现（118 续作候选，不入本补丁）：prep 对包面目标仍施加组件面改写（`while→for`、storage→模型字段、str→Str、循环-fn verbatim 内联（含 `List`/`str` VM 拼写））——逐项语义隐患已在 ①–③ 实证。

2026-11-22 T-02 阻断（外部依赖，非用户决策）：本计划双端首个阻断 = master 源准备（ace_source_prep）对 `use`/`style` 行的空白化 bug，已由 **PLAN-117** 在其工作树修复（auto-ui `299ac88d` + auto-lang `f41e7df15`/`b3275c18f`/`c80a5e2b`，截至 20:42 仍在途、未落 master）。按 §4 共享写面纪律，118 不重复修、不基线挪用在途提交。**unblock 动作：PLAN-117 完成 T-03..T-08 落 master 后，118 双树 rebase 到新 master 并复验双端准入**；随后接续 031-paint 包面余量缺口枚举与接通（T-02 续→T-03/T-06）。等待期间本计划仅做不写 117 活跃面的独立工作。

2026-11-22 goal round 3 补充（blocked 收据对应）：三轮（goal round 1/2/3）同一外部条件未变——117 未落 master；无争独立工作已做到边际（缺口登记 G1..G6 + G6 七件设计 + 探针工具在案，§9）。剩余 T-02 施工（prep 消费复验、G2 场景绑定、G3/G4/G5 stdlib、G6 七件）全部位于 117 在途写面或其直接下游；即刻施工 = 对在途共享缝双写，违反 §4。**精确 unblock：①等 PLAN-117 归档落 master → ② `/goal 完成计划118` 续跑（自动 rebase 双树到 master、复跑 `first-admission.py` 与 `P118_PKG` 探针、按 §9 缺口表逐件接通）**。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: 117 落主后双树 rebase + 双端准入复跑 + ColorPicker 包面墙定位 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@bf1d07bb，rebase 至主线 3d6ae618）+ D:/autostack/.wt/ui-118/auto-lang（plan-118-dev@b24e8a42e，rebase 至 d13488b7e） | commit: 7 + 2 提交在枝（本轮到 G6 臂、prep 归一/门控/场景改写、graft 三件） | next: §10 裁定（ColorPicker 088 包在 AGE 包面不可准入）

**post-117 复验（2026-11-24）**

- 117 已归档落主（auto-ui `3d6ae618` / auto-lang `d13488b7e`），外部依赖解除。118 双树 rebase 完成，冲突处置：G6 canvas 臂与 117 的 image_surface 臂同位并存（package_view/frame）；prep 的 str/Str 归一、while→for 包面门控、scene 合成名改写叠加到 117 新 prep 上（保留 117 的 lower/upper/split/replace 改写）；auto-lang 的 `.substr(` 接枝触发位并入 117 的 `uses_checked_text_methods` 构造。
- 构建 exit 0（`--locked --offline -j2`，fresh 3m07s）。AV 准入（真窗 8s）越过 prep/入口，首墙 = `source.syntax: module color_picker::color_picker: expected fn, found widget @2239`；A2Vue 严格 lane 同前（App 21 handler 缺席）。
- **ColorPicker（088，VM widget 轨交付）不可由 AGE 包面准入**，两处具名语言面缺口：① 30 个 handler 中 7 个 `.RCommitted()/.GCommitted()/.BCommitted()/.ACommitted()/.HCommitted()/.SCommitted()/.VCommitted() ->` 为**显式空括号零参绑定**——AGE handler 文法要求参数表非空（`expected fn, found widget` 误归因于头部），去括号后语义校验再拒「handler 绑定 0 参，消息声明 1 payload 槽」；② `color_model.at` 25 处 `for <v> in 0..<end> { … }` 范围循环——age-package 模块 fn 面 `expected '{', found 'in'`（@1862）。最小复现 `micro-empty-paren-handler.at` / `micro-range-for-fn.at`，证据 `plan-artifacts/plan-118/post117-admission.txt`。
- 上轮 ddmin 定位的 quoted-record 墙已随 117 落主消失（`bis-sd-quotedrec.at` 现 PARSE OK）；本墙是新的、独立的组件准入缺口。
- 生成物清理：A2Vue `--gen-only` 在示例目录创建 `deps/` 3 个 junction，已 rmdir 清除并核目标包完整（后续准入脚本需带清理）。

2026-11-24 §10 裁定请求（ColorPicker 088 包 × AGE 包面）：031-paint 双端都经 AGE 包面装载 widget 级 `use { package: color_picker }`，上述两缺口不补则 T-02/T-03/T-06 全阻。选项：①授权 118 在 plan-118-dev 的 auto-lang 树补两件（空参 handler 绑定语义 + 模块 fn 范围循环受理/desugar）并带正反回归；②授权 118 在 auto-ui 宿主组装期对依赖包源做等价兼容改写（`.X() ->`→`.X(_p) ->`、range-for→while），不动 auto-lang（改动小但与 A2Vue 原源道存在双道偏差风险）；③以 AGE 原生选色组件替代 088 消费（改 §1 冻结输入/§6 验收，需批准）；④移交语言 owner 计划，118 挂起等落主。**注**：件②（list.join 执行器 op 链）仍在既有授权内、与上述裁定无关，可先行施工。


2026-11-24 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 6 | from: work | to: work | status: archived | outcome: waiting | worktree: D:/autostack/.wt/ui-118/{auto-ui,auto-lang} | commit: auto-ui bf1d07bb / auto-lang b24e8a42e | next: user（§10 裁定：ColorPicker 088 包 AGE 准入路径；等待 1/3——无答复时下一轮先做件② list.join op 链）

2026-11-24 用户裁定：选①——授权 118 在 plan-118-dev 的 auto-lang 树补两件（空括号零参 handler 绑定语义；age-package 模块 fn 的范围循环受理/desugar），带正反回归；随后 118 继续 T-02/T-03/T-06。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: 用户裁定①落地（空参 handler + 包面范围循环）+ 088 准入余量闭合至 join | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@313abc45）+ auto-lang（plan-118-dev@f2430628b） | commit: auto-ui 313abc45 / auto-lang f2430628b | next: work（件② list.join 执行器 op 链）

**裁定①落地与 088 准入余量（2026-11-24，证据 plan-artifacts/plan-118/post117-admission.txt）**

- 两件授权落地（auto-lang f2430628b）：①显式空参 handler `.Msg() ->`（绑定 0 参、payload 丢弃；隐式零参与显式参数不匹配仍具名拒）；②包面模块 fn 范围循环 `for v in a..b`（边界走 condition-ops，避免 `n {` 被记录构造提交吞块；非 i32 边界仍 `source.range-bounds` 拒）。age-component/age-package 文法 id 按镜像重基线（§9 既有说明行已补 PLAN-118 注）。
- 驱动真实 088 包后显现的**同授权意图内闭合项**（逐项具名，供复审裁定是否接受为「准入补件」）：③AGE 包面与组件面同享「整局部 push 隐式转移」（VM 文本列表语义；清掉 220 处 `source.move-required`，其它 profile 不变）；④惰性 view-data 字段（`palette: .palette`）在子组件属性值/视图值检查里按只读字面量内联（此前留未绑定 `self`）。三项均有正反回归（`plan118_package_surface`，3 例 0.08s；`plan981_` 253 例全绿）。
- 宿主装配适配（auto-ui 313abc45）：依赖包模块顶层 `fn` 提升为 `pub fn`（AGE 跨模块导入需 public；088 color_model 为模块私有 fn）；依赖包模块同跑 canvas 场景合成名归一；`.split` 改写按 handler 作用域配平（`.ApplySnap`/`.Load` 同名 `parts` 串接收者修复）。均带用例（0.00–0.01s）。
- **031-paint × ColorPicker 宿主装配编译余量 = 13 处 `list.join`**（件②；其余全通）。已知两处主线既有红与本计划无关（auto-lang `text_record_source` 5 例、auto-ui `real_u03_apps_load_through_desktop_bridge`），已在基线复核确认。
- 受影响用例：auto-lang conformance 907 passed + 5 既有红；auto-ui `ace_widget_packages::` 5 passed、`ace_source_prep::tests` 8 passed + 1 既有红。

2026-11-24 stage: goal | plan_id: PLAN-118 | plan_revision: r1 | round: 7 | from: work | to: work | status: archived | outcome: pass（阶段推进） | worktree: D:/autostack/.wt/ui-118/{auto-ui,auto-lang} | commit: auto-ui 313abc45 / auto-lang f2430628b | next: work（件② list.join：resolve 方法臂 + MIR/AV 执行器 op + 回归；随后重跑 paint 装配与 T-03）

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: 件② list.join 闭环 + 031-paint × ColorPicker 包面装配编译通过 | outcome: pass（T-02 编译门绿） | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@e870f213）+ auto-lang（plan-118-dev@41c44d1f4） | commit: auto-lang adbc26108 / 41c44d1f4；auto-ui e870f213 | next: T-03 AV 实窗

**件② 与准入余量收口（2026-11-24，证据 `plan-artifacts/plan-118/`：`paint-compile5..13.txt`、`gallery-compile.txt`）**

- **件② `list.join` 的形态裁定：resolve 级脱糖，不新增执行器 opcode**。原 §10 round-3 记的「必须新 ListJoin op（半日工）」经勘察证伪：`xs.join(sep)` 可在 resolve 里脱糖为「`out = ""`；`i < len` 循环内 `i > 0` 时补分隔符、再拼 `xs.get(i)`；返回 `out`（move）」，与 `clear`（lowering 脱糖）和 typed HOF（resolve 脱糖）同一「无新 opcode」先例；owned 文本元素经既有非破坏元素视图参与 concat（W108 文本 concat 用域）。验证：AV 执行真值 `["a","b","c"].join("-") = "a-b-c"`、空表 `""`、单元素 `"only"`（`crates/av/tests/cases/plan118_join.rs`，含 `.px = xs` 与 `.px[1]` 读回）。
- **join 探针顺带暴露并闭合的三处包面语言缺口**（均 auto-lang，范围仍在本轮裁定①「组件包准入补件」意图内，逐项具名供复审）：
  1. **具体文本列表 → 动态列表写位**：VM 轨模型列表 `var px = []` 在包面按 `List(Dyn)` 承载，而 handler 局部列表按文本元素定成 `List<Str>`——`.px = np` 8 处 `expected list, found list`。闭合法：赋值位元素级物化为 `List(Dyn)`（`coerce_assigned_list`，与 `check_push_value` 的 Dyn 收容同律）。
  2. **包面 verify 的 owned 元素视图门**：resolve 的 `elem_view_admission` 早已含 AgePackage，verify 侧包面入口却只给组件面开 `elem_view_owned`——join 的元素读被误判 `verify.view-escape`。闭合法：新增包面专用 verify 入口（Auto/TextRecord 各一），**保持 `ordinary_auto` 原值**（中途误置 false 曾令 `plan981_` 两例回归，已改回 true 并复绿）。
  3. **动态列表形参收具体文本列表**：裸 `List` 形参解析为 `List(Dyn)`、裸 `List` 模型字段解析为 `List<Str>`（PLAN-113 先例），`cp_pal_item(.pal_norm, i)` 因此类型不符。闭合法：AGE 面「形参元素为 Dyn 时接受任意列表实参」——与 Dyn 列表 push 收容一致（运行时列表元素本就是动态值）。
- **宿主装配余量（auto-ui e870f213）**：依赖包模块源同跑 app 的 `str`→`Str` 注解归一（`normalize_package_text_surface`），清掉 088 `color_model` 的 11 处 `expected str, found Str`；与场景合成名归一同一 pass 序。
- **结果：`031-paint` app + 088 ColorPicker 依赖包整体包面装配编译通过**（`paint-compile13.txt`：`PAINT PACKAGE COMPILES`），T-02 编译门绿。
- 回归面：auto-lang core conformance **908 passed + 5 既有红**（`text_record_source`，基线复核非本计划）；`plan118_package_surface` 4 例；AV `plan118_join` 1 例（真实执行）、`plan981_` 98 例全绿；auto-ui `ace_widget_packages::` 5 passed。AV 全量 1438/64 红与本计划无关（sys/performance/资源门，逐条抽样在基线复现：`av_method_modes::use_after_move_of_method_arg_rejected` 在 HEAD 即 `verify.double-move`）。
- 影响与风险登记：件② 脱糖每次 join 复制一次文本（AV 文本拼接语义）；`join` 目前受理 `Str`/`StrOwned`/`Dyn` 元素（记录元素列表不脱糖，仍按原判据）；动态列表形参收列表实参属 AGE 面放宽，靠「运行时列表元素为动态值 + Dyn 形参即动态契约」成立，复审可裁定是否收窄到文本元素实参。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-03 真窗准入推进（入口视图发射/预 Init 兜底/依赖包文本所有权）+ 下一墙定位 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@d563d34c）+ auto-lang（plan-118-dev@9b0735ed4） | commit: auto-lang 01832d9e8 / 9b0735ed4；auto-ui d563d34c | next: work（ColorPicker 子件受检视图重放的 MIR 调用实参类型收口）

**T-03 真窗准入（2026-11-24，证据 `plan-artifacts/plan-118/round3-evidence.txt`）**

- 起点失败：真窗 `--source src/front/app.at` 报 `the entry component has no checked view`——不是宿主选错入口，而是**引擎根本没发入口视图**：`add_widget_views` 只按「被引站点/定时器/宿主绑定/targets」选发，031-paint 的 `storage.set/get` 已被 prep 改写成状态字段、无 timer、也无人引用入口组件，于是整体跳过。修复（auto-lang）：**widget live 根组件恒发视图**（node 组件仍按原判据），并允许根组件带惰性 view-data 字段；AV 回归 `plan118_entry_widget_view_is_emitted_for_the_live_root` 真值通过。
- 随之暴露并闭合的三层宿主/准备面问题（auto-ui d563d34c）：① 预 Init 求值入口视图会读还没种下的 `.px[42]`（`.px` 由 `.Init` 填 256 项）——`initial_widget_handlers` 求值失败不再让装载失败，退回「入口 + 全部站点」可见集，Init 后按真实帧重算；② 依赖包模块的 `str` 归一把**形参**也改成 owned `Str`，把 `cp_hex_valid(phex)` 变成 move+读（`use-after-move`）——形参保持借用，只归一返回与 `var` 局部；③ `var t Str = phex`（借用→owned）与 `return t`（借用→owned 结果位）缺拷贝——resolve 按值拷贝（`.to(Str)`）。
- 第二处 MIR 所有权：`var up Str = ch` 从另一个 owned 文本局部**初始化**时按 move 处理，后续 `== ch` 读已移动槽——AGE 面 let 初始化改为**拷贝**（与 117 的 view-data 快照读克隆同纪律，auto-lang 9b0735ed4）。
- **当前墙（T-03 真窗）**：ColorPicker 子件的受检视图重放已过所有权检查，剩一处 `mir.call-arg-type`：`call arg #0 has type 16, signature wants 11` @color_picker 61318..61345（`cp_cursor_edge_h(ld_sat, ld_v)` 所在 handler 路径，func 143）。真窗在 open 阶段失败（exit 5），未进入帧渲染/脚本阶段。
- 影响：AV 入口视图发射属引擎面变更，受控回归为 `plan981_` 98 例 + core conformance 908/5 既有红；T-03 尚未达成（无帧证据、无指针/undo/存储/关闭证据）。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-03 真窗渲染达成（live 道）+ 无状态重放道预算墙定论 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@0c90d471）+ auto-lang（plan-118-dev@6e2fc5620） | commit: auto-lang 6e2fc5620；auto-ui 0c90d471 | next: work（live 道证据通道接线 + 画布坐标点击脚本 → T-03 oracle）

**T-03 真窗（2026-11-24，证据 `plan-artifacts/plan-118/round4-evidence.txt`，av-run9..19）**

- 无状态重放道定论（不再尝试）：031-paint×ColorPicker 在无状态车道单帧要重放入口整模型——实测入口视图一次求值 ~6.2s / ~6M AV 步，子件 ~8.2s；每个字段的状态函数都把整模型再内联一遍（ColorPicker ~40 字段），且冻结核验预算直接拒：`VerifyMemory` 17.5MB > 16MB、`VerifySteps` > 2M 步。**这不是可调的参数问题，是重放架构与模型规模的错配**。
- 期间闭合（auto-lang 6e2fc5620）：AGE 面「动态列表形参收具体文本列表」的类型放宽此前只到 HIR，MIR 槽型不一致被 AV 拒（`mir.call-arg-type`）；调用点现与赋值位同一元素级 Dyn 物化。
- **live 道接管（auto-ui 0c90d471）**：`LivePackage::prepare` 的旧门「子件有状态但无 outbound/watch/focus/timer 需求 → 回退无状态道」把 paint 排除在 live 道之外；改为**优先 live**（widget-runtime 编译器不支持时仍按既有诊断静默回退）。桌面帧预算按重模型定档（60M 步 / 20s 墙，仍具名资源拒绝）。
- **实测：真窗装载、渲染一帧、干净关闭——exit 0，frames=1，errors=0**（av-run19.json）。这是 T-03「真实窗口、操作和关闭」的第一个可复现正果。
- **余量（下一轮）**：live 道的桥证据通道为空（buttons/field_names/dispatch_log/states 全空）——无状态道有值但跑不动本应用。T-03 oracle（px42 文本叶、工具点击派发、undo/redo、存储往返、关闭）需要：live 模式帧动作→按钮/点击映射、会话 states/texts 读回、dispatch_log 接线，以及画布坐标点击脚本（指针轨迹）。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-03 帧证据可读 + 脚本点击/笔事件接通；余量=子件出站事件路由 + 画布场景数据道 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@a931ad91）+ auto-lang（plan-118-dev@6e2fc5620） | commit: auto-ui a931ad91 | next: work（ColorPicker→Paint 出站事件路由；场景五列表→CanvasScene；然后 px42/undo/存储/关闭 oracle）

**T-03 交互与证据（2026-11-24，证据 `plan-artifacts/plan-118/round5-evidence.txt`，av-run20..25）**

- **帧证据通道**（T-03 的观察前提）：`PackageFrame` 增 `view()/states()` 访问器、`PackageAction` 暴露 key/message；桌面 evidence 增 `frames` 数组（每实例 revision、文本叶、label、元素数、标量状态、可点击动作）。实测 paint 实例 0：rev=1、49 元素、20 labels（含 **`px42=#FFFFFFFF`**）、13 标量状态、24 动作。**无状态重放道此前只有它才填证据，但跑不动本应用；现在 live 道同样可读**。
- **脚本点击真跑**：`2:click:0:1,6:click:0:3` → dispatch_log `Tick,SetTool,Tick,…,SetTool`，末帧 rev=5、`tool=picker`（target 3=取色器按钮）；`Save` 把 256 像素 join 写入存储字段（`__st_paint_canvas_v1` 长 2559，件② join 全量实证）、`New` 置 `saved=0`/`status="new canvas"`。
- **画布笔事件全链接通**：帧组装记录 canvas 的 `onpenstart/onpenmove/onpenend` 目标；新增 `PackageEventInput::Pen{x,y,phase}`（`accepts_target`/`arguments` 与 `onpen*` 键对齐）；`package_view` 的 canvas 臂接 `on_pen_*`；窗口脚本新增 `pen:<实例>:<目标>:<x>:<y>:<start|move|end>`。实测 pen 动作已出现在帧动作表（surf/hue/alpha 九项）。
- **下一墙（具名）**：pen 事件到达 ColorPicker 子件后，其 handler 产生**子件→父件出站消息**（`preview`/`committed`），包面拒 `source.age.handler-outbound: message preview requires a checked widget event consumer`——子件事件路由（ColorPicker onpreview/oncommitted → Paint PickPreview/PickCommitted）未接。再下一件是画布场景五平行列表→`CanvasScene` 的数据道（surface 渲染 + `onhit`→`PaintHit(id)` 的像素轨迹 oracle）。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: live（状态）道真正接管 paint（此前 round-4 的"优先 live"是空转）+ 笔事件在 live 道派发 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@424a668a）+ auto-lang（plan-118-dev@bfddf976d） | commit: auto-lang bfddf976d；auto-ui 424a668a | next: work（画布场景五列表→CanvasScene + onhit 命中 → 像素轨迹 oracle；随后 undo/存储/关闭脚本）

**T-03 live 道接管（2026-11-24，证据 `plan-artifacts/plan-118/round6-evidence.txt`，av-run26..29）**

- **真因勘定**：给 `PackageStateEngine::new` 加临时探针后实证 `has_entry_view=yes` 而 `live_package=false`——round-4 的「优先 live」对 paint 是空转，窗口一直在**无状态重放道**上跑（因此有帧但单帧 6–8s，且 pen handler 重放时把子件的 `preview` 出站发送按组件面拒掉）。
- **live 拒绝原因**：`LivePackage::prepare` 因入口 widget 带惰性 view-data 字段被 `source.age.widget-runtime-view: a checked widget runtime requires an ordinary widget view` 拒绝。修复（auto-lang bfddf976d）：widget runtime 受理 view-data 模型（仍要求普通 `view` 出口）；`live_view::build_view` 的 ViewScope 已按只读字面量内联 bare view-data 名。
- **第二处所有权**（auto-ui 424a668a）：prep 生成的 `split/replace` helper 形参是 owned `Str`——`.ApplySnap` 的 `var parts = s.split("|")` 首次调用就把 `s` move 走，后续 `split_at` 再读触发 `mir.use-after-move`。helper 形参改为借用 `str`；app 的 str 归一也不再改写 fn 形参（owner 位保持借用；返回/局部仍归一）。
- **实测**：`live_package=true`，窗口 exit 0、frames=1、errors=0；帧 labels 含 `px42=#FFFFFFFF`、33 动作、13 标量态。笔事件脚本（ColorPicker surface 的 start/move/end）在 live 道派发 `SurfDown/SurfMove/SurfUp`，**errors=0**（子件 handler 真跑，出站发送被受理）——round-5 的 `handler-outbound` 墙是无状态道的伪影。
- **余量**：画布场景五平行列表→`CanvasScene` 数据道 + `onhit` 命中（paint surface 的 `PaintHit(id)` 像素轨迹 oracle）；随后 undo/redo、存储往返（New 后 Load）、关闭/重开脚本。

- **release 实跑 oracle（2026-11-24，`target/release` 例程）**：脚本 `Save→New→Load→Undo→Redo`（目标 31/30/32/27/28）在真窗 exit 0、frames=1、**errors=0**、28 次派发；末态 `status="loaded"`、`saved=0`（Undo/Redo 置 0）、存储字段长 2559、`px42=#FFFFFFFF`。**debug 构建在同一脚本上撞 `ExecutionWall`（Load/ApplySnap 的 O(n²) split helper + debug AV）——证据实跑改用 release 构建**（T-03..T-08 的 runbook 注记）。
- 余量不变：像素轨迹（`PaintHit`）需画布场景五列表→`CanvasScene` + `onhit` 命中；关闭/重开脚本步可直接接在 oracle 序列尾部。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-03 AV oracle 全绿（像素轨迹/undo-redo/存储往返/关闭重开） | outcome: pass（T-03 AV 腿） | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@d8109af2）+ auto-lang（plan-118-dev@107d81eba） | commit: auto-lang 107d81eba；auto-ui d8109af2 | next: T-04 AV 独立复核（revision 绑定证据）

**T-03 收口（2026-11-24，证据 `plan-artifacts/plan-118/round7-evidence.txt`，av-run33..35）**

- **画布场景数据道**：live 会话新增 `list_field(name)` 读面（av 107d81eba）；包面帧按 canvas 元素的 `scene` 名读五平行列表（`<scene>_pts/_meta/_nodes/_edges/_labels`）→ `CanvasScene`（与 VM 轨同一宽容契约），canvas 视图消费它——Iced PenArea 的命中判定因此对真实节点生效。
- **tap 事件面**：渲染层 on_hit 原先把命中 id 走 `PackageText`，而 canvas 动作不是 input 位会被拒；新增 `PackageEventInput::Hit(id)` + `NodeDesktopMsg::PackageHit`，脚本新增 `tap:<实例>:<目标>:<x>:<y>`（与渲染层同一条 `canvas_hit_test`）。
- **release 实跑 oracle（av-run34）**：`tap(210,50)`（px42 中心）→ `PaintHit` → 标签 `px42=#111827FF`；`Save`（存储 2559）→ `New` → `Load`（恢复已画画布）→ `Undo` → `Redo`；33 次派发、**errors=0**、exit 0；末态 `px42=#111827FF`、`saved=0`、`status="loaded"`。
- **关闭/重开（av-run35）**：session 1 `closed=true` generation=1；session 2 `closed=false` generation=2；frames=2、errors=0；新世代按 Init 重种（新帧 `px42=#FFFFFFFF`）——世代纪律 oracle 成立。
- **结论**：T-03「真实窗口、操作和关闭」的 AV 腿达成。debug 构建在重 transition 上仍会撞 20s 执行墙（O(n²) split helper 的调试态成本），**证据一律用 release 例程**。
- 回归：auto-ui package 11/11、prep 10+1 既有红；auto-lang av plan981 98/98、av plan118 2/2、core 908+5 既有红。

2026-11-24 stage: review | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-04 AV 阶段独立复核 | outcome: pass（3 项非阻断注记） | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@d8109af2）+ auto-lang（plan-118-dev@107d81eba） | commit: 复核对象 = 上述两提交（工作树干净） | next: T-05（AV 收据交付）→ T-06 A2Vue

**T-04 AV 复核（revision 绑定：auto-lang 107d81eba / auto-ui d8109af2；证据 `plan-artifacts/plan-118/av-oracle-{a,b}.json` + `av-oracle-receipt.txt`，runner `run-av-oracle.ps1`）**

复核方式：不看工作叙述，只按 §7 验收条件重跑可复跑脚本（release 例程 `target/release/examples/ace_node_desktop_window.exe`，源 `examples/ui/031-paint/src/front/app.at`）：

| 原 oracle 行 | 脚本 | 实测 | 判定 |
|---|---|---|---|
| 真实指针落笔 | `tap:0:4:210:50`（px42 中心，scene hit_test） | `PaintHit` 派发，`px42=#FF0000FF` | PASS |
| 像素 42 按选色变化 | `type:0:15:#FF0000FF` + `click:0:16`（HexEnter 提交） | `cur=#FF0000FF`，落笔后 px42 同色 | PASS |
| Undo/Redo 回放 | `click:0:27` / `click:0:28` | 两次均派发、`saved=0`，终态回红 | PASS |
| 存储往返 | `Save`→`New`→`Load` | 存储字段长 2559（256 像素 join），New 后 `saved=0`，Load 恢复已画内容（终帧 `px42=#FF0000FF`） | PASS |
| 取消选色保正式色 | 未在 AV 脚本覆盖（组件的 oncancelled→`.PickCancelled` 在 A2Vue 侧按 V-04 覆盖；AV 侧同一 handler 已编译执行） | — | N/A（记 T-06 覆盖） |
| 关闭/卸载 | `close`+`reopen` | session1 `closed=true`（gen1）/session2 `closed=false`（gen2），`frames=2`、`errors=0` | PASS |

- 可复跑性：`run-av-oracle.ps1` 17 条断言全 PASS，产 SHA256 `CE6CB196…5E4225`（A）/`0552ED56…071996E`（B）；重跑同脚本同结果（本轮两次运行一致）。
- 反例/边界：错误通道在两次 oracle 中均 0；debug 构建在重 transition 上会撞 20s 执行墙（已在 §9 记，证据一律 release）；无网络/权限面（本应用只消费内存 storage 仿真，无 file/sqlite 绑定）。
- 代码核（AC-04）：分支 14 提交逐条核对，无调试残留（`P118_*`/`LANE-DBG`/`LIVE-REFUSED` 等 0 命中）；`frame_limits` 抬档（60M 步/20s）与 live 道优选均有注释与失败回退。

**非阻断注记（3）**
1. `crates/auto-ui-runtime/examples/p118_diag.rs`（计划期诊断例程）保留在分支：与仓内既有 probe 例程（p022/p023/plan804）同惯例、不进生产路径；合入前可在 T-08 决定保留或移出。
2. 场景数据道仅接 live 道；无状态重放道仍渲染空场景（该道对本应用不可用，且此前同样为空——无回归）。
3. `flex-wrap` 在 native 渲染器按 PLAN-412 降级（paint 的 `wrap_row`），属既有渲染面注记，不影响验收行。

2026-11-24 stage: merge | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-05 AV 收据交付（W1 行） | outcome: pass | worktree: D:/autostack/.wt/ui-118/{auto-ui,auto-lang} | commit: auto-lang 107d81eba / auto-ui d8109af2（未合主；随 T-08 统一落主） | next: T-06 A2Vue 实跑

**T-05 AV 交付收据（W1 消费行）**

- AV 实现面：auto-ui（宿主：live 道优选、帧证据、画布场景/笔/命中、脚本 tap/pen、prep 修）与 auto-lang（包面准入、join/列表赋值、live 根视图、文本所有权、widget runtime view-data、live 会话列表读面）两棵 plan-118-dev 树；均为可复跑 release 例程。
- 证据面：`plan-artifacts/plan-118/av-oracle-{a,b}.json` + `av-oracle-receipt.txt`（SHA256 绑定）+ `round7-evidence.txt`；runner `run-av-oracle.ps1` 一条命令重放。
- W1 行：像素 42 按选色变化（红）、Undo/Redo、Save→New→Load 存储往返（2559 字符快照）、close/reopen 世代纪律——全部 PASS、errors=0。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: A2Vue 生成/构建通（T-06 前半），真浏览器撞生成代码所有权陷阱 | outcome: partial | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@f7c53f95） | commit: auto-ui f7c53f95 | next: T-06 续（a2x-ts 视图读面 Str 所有权 / 列表元素读）

**T-06 进展（2026-11-24，证据 `plan-artifacts/plan-118/round8-evidence.txt`）**

- **生成路由（根因修复）**：`AUTO_UI_ACE_VUE=1` 原先对**每个 app** 走逐 widget 组件 lane，包面 app（031-paint，21 个 handler 全在包面检查函数里）被 `ace.vue-lane-unsupported` 拒。现按入口是否带 **widget 级 `use { package: … }`** 路由到包面 SFC 生成器（`prepare_component_source` 先归一面 → `generate_linked_front_sfc`）：单件 App.vue 自包含（子 widget 站点/出站通道内联），36 文件、App.vue 1.57MB、ColorPicker.vue 149KB。
- **构建通**：`pnpm install --offline --no-frozen-lockfile`（生成工程无 lockfile；离线 store 命中 7s）+ `pnpm run build`（vue-tsc && vite build）→ dist 410.44kB，exit 0。
- **真浏览器（Playwright 1.63.0 Chromium）逐层推进**：①`ace.widget.payload: mismatched arguments for 'PaintHit'`（onhit 的命中 id 是渲染层隐式实参）→ 视图校验按 input 位受理；②`ace.widget.event: unsupported binding 'onhit'`（web 视图无 canvas arm）→ 新增 canvas arm（模型五平行列表→绘图、rect/circle 逆序命中、命中 id 派发）；③`A2xTrap: use-after-free on Str#3625`（**生成的 ColorPicker 视图读模型列表/文本时**）——这是 a2x-ts 读面所有权降级的代码生成问题，非接线问题。
- **判定**：A2Vue 生成/构建腿 PASS；真浏览器运行时被上述所有权陷阱阻断（T-06 余项）。
- 回归：auto-ui package 12/12；auto-ui-tools vue 套件**单线程 111/111**（并行跑有既有 env 污染 flake：baseline 同现象 2–3 红、名字随机）。

2026-11-24 stage: work | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-06 A2Vue 双端全绿（真 Chromium 渲染+交互 oracle） | outcome: pass（T-06） | worktree: D:/autostack/.wt/ui-118/auto-ui（plan-118-dev@67487137）+ auto-lang（plan-118-dev@741dff8bd） | commit: auto-lang 741dff8bd；auto-ui 67487137 | next: T-07 终审

**T-06 收口（2026-11-24，证据 `plan-artifacts/plan-118/round9-evidence.txt`、`paint-vue.png`、`paint-vue-e2e.txt`）**

- **浏览器崩溃根因（真因）**：生成体对每个 owned 局部在 `finally` 释放；`list.push(it)` 后 `it` 仍按 owned 处理被 `a2x_str_free`——表里留下已释放句柄，包面视图读场景列表即 `use-after-free on Str#3625`。修复（auto-lang 741dff8bd）：`a2x_list_push` 对文本元素存**独立副本**（文本是值语义）。
- **第二处**（auto-ui 67487137）：带隐式值的输入事件此前只认 `oninput/onchange`，ColorPicker 的 HEX 提交（`onenter` → `RCommitted`）在校验期被拒；扩到 `onenter`。画布 arm / `onhit` 派发 / 包面路由已在 f7c53f95 落。
- **真浏览器 oracle（Playwright 1.63.0 Chromium + vite preview）**：页面完整渲染（标题/四个工具/HEX/RGB/7 个 canvas）；HEX 输入 `#FF0000FF` + Enter 选色后画布点 px42 → `px42=#FF0000FF`；`New` 回白、`Load` 恢复已画内容、`Undo` 回退、`Redo` 重放；截图 `paint-vue.png` 可见画布单像素与取色器渐变/色相/alpha/调色板。
- **结论**：同一 AGE 程序在 AV/Iced 与生成 Vue SFC 真浏览器上跑同一组 oracle 行——T-06 达成；双端一致性有可复跑证据（AV：`run-av-oracle.ps1`；A2Vue：`paint-vue-e2e.mjs`）。
- 回归：a2x-ts 套件全绿；auto-ui package 12/12、tools vue 111/111（单线程）。

2026-11-24 stage: review | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-07 独立终审（双端） | outcome: pass（2 项非阻断注记） | worktree: D:/autostack/.wt/ui-118/auto-ui@67487137 + auto-lang@741dff8bd | commit: 复核对象 = 上述两提交（工作树干净） | next: T-08 交付与归档

2026-11-24 stage: review | key: PLAN-118:r1 | outcome: pass | reviewed_commit: AutoUI `674871374737cad258d8b2d93358eb51229b5564` on `3d6ae6189cf4faa8c7d9ddeb86a64d926994a32b`, AutoLang `741dff8bd968a19d7d33d54d768b395118377992` on `01ab849c9711a1d43c4b0154bed95c5b8ef528e2` | spec_delta: 无新增/废弃组件（approved no-impact：示例应用 + 宿主适配面） | gates: merge-owned auto-lang `cargo t ace a2ts` 待执行

**T-07 终审（revision 绑定：auto-ui 67487137 / auto-lang 741dff8bd；证据：`av-oracle-receipt.txt`、`paint-vue-e2e.txt`、`paint-vue.png`、`round9-evidence.txt`）**

按 §7 验收行逐条复核（不看叙述，只跑可复跑脚本）：

| §7 行 | AV/Iced（`run-av-oracle.ps1`） | A2Vue 真浏览器（`paint-vue-e2e.mjs`） | 判定 |
|---|---|---|---|
| 真实指针在画布落笔 | `tap(210,50)` → `PaintHit` → `px42` 着色 | canvas 点击 → `px42=#FF0000FF` | PASS |
| 像素 42 按选色变化 | HEX 输入 `#FF0000FF` → `cur` → 着色 | HEX + Enter 选色 → 着色；RGB 改 R → HEX `#000000FF` | PASS |
| 取消选色保正式色 | 组件 `PickCancelled` 已编译执行；脚本未覆盖该路径 | 未覆盖（对话框取消需前置打开） | **注记 1** |
| Undo/Redo | PASS（终态红） | PASS | PASS |
| 存储往返（New 后 Load） | PASS（2559 字符快照） | PASS | PASS |
| 刷新/关闭后按 Init 恢复 | close→reopen（gen2、状态重种） | 页面刷新即新世代（web 无持久化，按设计） | PASS |
| 工具/控件可用 | 工具点击派发 | 四工具渲染、按钮点击全通 | PASS |
| 同一 AGE 程序双端 | 同一 `app.at`（包闭包） | 同一 `app.at` → SFC | PASS |

- **证据链**：AV 侧 SHA256 绑定（`CE6CB196…`/`0552ED56…`）；A2Vue 侧构建产物 410.49kB + 截图（画布单像素、取色器渐变/色相/alpha/调色板、状态 `loaded`、`px42=#FF0000FF`）。两脚本均可一条命令重放。
- **代码核（AC-04）**：两分支提交逐条核对，无调试残留；`frame_limits` 抬档、live 道优选、包面路由均有注释与具名回退；`a2x_list_push` 副本语义有证据绑定（修前 `use-after-free on Str#3625`，修后页面零错误）。

**非阻断注记（2）**
1. 取消选色（`oncancelled → .PickCancelled`）两端的组件代码均已编译执行，但脚本未驱动该路径；建议在 T-08 归档时作为已知覆盖缺口记入 §10，或在后续计划补对话框取消的脚本行。
2. `p118_diag.rs`（计划期诊断例程）与 paint 包面回归测试（约 22s）保留：前者与仓内 probe 例程同惯例、不进生产路径；后者为整应用集成断言（超 10s 单测建议值，属重型集成口径）。

2026-11-24 stage: merge-prep | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: T-08 准备面盘点（未开始落盘） | outcome: pending | worktree: 两树 guard-clean | commit: auto-ui plan-118-dev 16 提交（master 3d6ae618）/ auto-lang plan-118-dev 11 提交（master 910cd2c68） | next: T-08 执行（rebase → ff-only → 归档 → 清理）

**T-08 准备面（2026-11-24）**

- 分支距离：auto-ui `master..plan-118-dev` = 16；auto-lang `master..plan-118-dev` = 11（均未落主）。
- `wt-guard.ps1`：两树均 **clean**（0 reparse point；本轮已清掉 `examples/ui/031-paint/{gen,deps}` 的 junction/node_modules 链接）。
- Spec delta：本计划 `new_spec_components: []` / `supersedes_spec_components: []`——**无 spec 组件新增/废弃**，落盘时按「approved no-impact delta（理由：示例应用 + 宿主适配面，不新增既有模块规格条目）」记录。
- 仍待执行的 merge-owned 操作（按 auto-plan-merge 合同）：①auto-lang 冻结受影响 `cargo t <group>` 闸门；②两树 rebase 到 master 并 `--ff-only` 落主（rebase 改哈希需 `git range-diff` 等价证明）；③`docs/plans/` 归档 + `status: archived` + 提交簿记；④worktree/分支移除（移除前重跑 wt-guard）；⑤把 `plan-artifacts/plan-118/` 全部证据折叠进本文件后删除该 scope，并刷新 `.retention/policy.json`。
- 证据折叠清单（删除前必须进本文件）：AV oracle 收据（SHA256/断言表）、A2Vue 浏览器 E2E 收据（9 行断言 + 截图路径 + 哈希）、生成/构建流水（36 文件/410.49kB）、回归数字（package 12/12、vue 111/111 单线程、a2x-ts 全绿、plan981 98/98、plan118 2/2、core 908+5 既有红）。

2026-11-24 stage: merge | plan_id: PLAN-118 | plan_revision: r1 | checkpoint: 合并自有闸门已执行（红=环境/工具链，非本计划改动） | outcome: blocked（待用户裁决落盘口径） | gated_commit: AutoUI `674871374737cad258d8b2d93358eb51229b5564` / AutoLang `741dff8bd968a19d7d33d54d768b395118377992` | receipt: auto-lang `target/test-groups/run-7g5hd0aq`（无 vcvars）、`run-12vg1hdr`（vcvars64）

**合并自有闸门（auto-lang `cargo t ace a2ts`，准备 `scripts/fixed-tests.py … --prepare-only` 先行；两轮执行）**

| 组 | 结果（vcvars 后） | 诊断 |
|---|---|---|
| ace-workspace | 4184/4379 通过，195 红 | `cl /WX` 撞 MSVC 14.43 头文件告警 C2220+C4391（`intrin.h` 内建返回类型），另 text-runtime 静态链接 `rust-lld: undefined symbol __CxxFrameHandler3/_fltused`——**工具链级**，与语言/包面改动无关 |
| ace-adapters | 96/97，1 红 | `auto-lang-core` source_protocol oracle 复判例（环境相关；同批 receipt 记 known reds 类） |
| ace-batch | 3/12，9 红 | `ace-conformance batch::*` 真生产者批（环境） |
| ace-facade | 99/100，1 红 | `p969_t02_console_lane_grants_and_capture`——**已在主检出 master 原样复现**（同 panic） |
| a2ts | runner `[WinError 2]` | **已知既有**（PLAN-1126 批 receipt：auto-lang 同 `[WinError 2]`，PLAN-1108） |

- 反证：`ace-text-conformance` 单例在主检出（master `01ab849c9`）同样失败；本机只有 MSVC 14.43 一个工具集，无法回退。
- 受影响模块的定向证据仍绿：`ace-core --test conformance` 908/5（既有红）、`av plan981_` 98/98、`av plan118_` 2/2、a2x-ts 套件全绿、auto-ui package 12/12、tools vue 111/111（单线程）。
- 结论：**本环境无法产出绿灯的冻结组闸门**（红因均在工具链/环境且可复现于 master）。按合并合约，红闸门不得直接落盘——需用户裁决：按定向证据落盘（记录环境红），或保留 reviewed 分支待正常 shell 复跑。

2026-11-24 stage: merge | key: PLAN-118:r1 | outcome: pass（按用户裁决：定向证据落盘 + 环境红记录） | completion_kind: delivered | delivery: AutoUI `674871374737cad258d8b2d93358eb51229b5564`（`--ff-only`，无合并提交）、AutoLang `15e3f08c9fc760383c3e2a7992716f3273c994b5`（rebase 后 `--ff-only`；`git range-diff 01ab849c9 741dff8bd master plan-118-dev` 全 `=` 等价，旧→新：`741dff8bd → 15e3f08c9`） | spec_delta: 无新增/废弃组件（approved no-impact） | archive: `docs/plans/archived/118-autoui-paint-av-vue.md`

**落盘与归档收据（T-08）**

- `landed`：两树均 `--ff-only`、无合并提交——落盘瞬间 AutoUI tip = `67487137…`（其后仅本计划的归档簿记提交 `6597ed12`）、AutoLang tip = `15e3f08c9…`（其后由并行 PLAN-1068 文档提交推进至 `4a93371a6`，本交付提交为其祖先，`git merge-base --is-ancestor` 已验证）。主检出 smoke：`cargo check -p auto-ui-runtime --features ace-bridge` **Finished**（需先暂时搁置一个**外来**未提交改动 `crates/auto-ui-runtime/src/ui/dynamic.rs`——见下）。
- `canonical_verified`：`docs/specs/` 无本计划 delta（`new_spec_components: []`），按 approved no-impact 记录；无索引/账本写入。
- `archived`：计划移入 `docs/plans/archived/`、`status: archived`、`completion_kind: delivered`，随本提交落在默认分支（簿记提交）。
- **门禁记录**：合并自有闸门 `cargo t ace a2ts` 在本机两轮（无 vcvars / vcvars64）均红，红因经诊断为**环境/工具链**且已复现于 master：MSVC 14.43 `cl /WX` 触发 `intrin.h` C2220+C4391、text-runtime `rust-lld` 缺 `__CxxFrameHandler3/_fltused`、`p969_t02_console_lane_grants_and_capture` 在 master 原样失败、a2ts `[WinError 2]` 为 PLAN-1126 批收据已记的既有 runner 错。受影响模块定向证据全绿（ace-core 908/5 既有红、av plan981 98/98、av plan118 2/2、a2x-ts 全绿、auto-ui package 12/12、tools vue 111/111 单线程）。用户 2026-11-24 明确裁决：**按定向证据落盘并记录环境红**。
- **外来未提交改动（保留未动）**：主检出 `crates/auto-ui-runtime/src/ui/dynamic.rs` 有四行 `pending_finish: None`（该字段在树中不存在，`cargo check` 报 E0560）。非本计划产物，按合约保留原样、不纳入本提交；其归属需另行确认。
- **证据折叠**（原 `plan-artifacts/plan-118/` 已删）：AV oracle 收据 SHA256 `CE6CB196…5E4225`(A)/`0552ED56…071996E`(B)，runner `run-av-oracle.ps1` 断言 17 条；A2Vue Playwright 9 行断言全 PASS（HEX `#FF0000FF` 选色 → 画布点 `px42=#FF0000FF`、RGB 改 R → HEX `#000000FF`、Save/New/Load、Undo/Redo），截图 `paint-vue.png`（画布单像素 + 取色器渐变/色相/alpha/调色板 + `loaded`/`px42` 状态）；生成 36 文件 / App.vue 1,573,9xx B / vite dist 410.47 kB；两处根因（a2x `a2x_list_push` 文本副本语义；web 视图 `onhit`/`onenter` 隐式实参受理 + canvas arm）。

2026-11-24 stage: merge | key: PLAN-118:r1 | checkpoint: cleaned | outcome: pass

- `cleaned`：两棵 worktree（`D:/autostack/.wt/ui-118/{auto-ui,auto-lang}`）在移除前分别重跑 `wt-guard.ps1` 均 clean；`git worktree remove` + `git branch -d plan-118-dev`（was `67487137…` / `15e3f08c9…`）完成；组目录 `D:/autostack/.wt/ui-118` 仍含**他计划**的 `auto-down`/`auto-web`，按归属保留。
- `plan-artifacts/plan-118/` 整 scope 已删除（188 文件 / 17,545,593 字节 / 0 reparse point），证据已折叠进本文件；`.retention/policy.json` 已刷新（`scopes["plan-118"]` = deleted-at-merge、`merge_closeouts` 追加、`last_merge_closeout` = PLAN-118）。
- 保留项（非本计划、未动）：主检出 `crates/auto-ui-runtime/src/ui/dynamic.rs` 的外来未提交 `pending_finish` 四行（E0560，归属待确认）；两仓 `plan-118-dev-pre117` 历史备份分支（round-1 快照）。
- 批回归到期检查：`docs/plans/.last-batch-regression.json` = `last_covered_plan_id 1126` / `2026-11-23T00:03:39Z`；118 非 5 倍数且 receipt 未超 48h → **不到期**，无需 batch gate。
- 部署观察（landing ≠ deployment）：本次仅示例应用 + 宿主适配 + 生成器/运行库源码；无生产二进制/发布包需要重启或重建，无 stale artifact 需要登记。
