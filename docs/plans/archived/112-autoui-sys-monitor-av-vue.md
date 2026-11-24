---
plan_id: PLAN-112
status: archived
feature_name: U04 SysMonitor 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-24
plan_revision: 2
current_step: 8
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: [auto-ui:docs/specs/runtime/back-proxy-sys-kill-policy.md, auto-ui:docs/specs/runtime/vm-sys-kill-authorization.md, auto-ui:docs/specs/ui/vue-plain-native-elements.md, auto-ui:docs/specs/runtime/handler-dispatch-async.md]
touched_goals: [GOAL-002]
---

# PLAN-112 U04 SysMonitor 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [106](106-autoui-u04-service-and-data-apps-av-vue.md) 的 U04 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [025-sys-monitor](../../examples/ui/025-sys-monitor/pac.at) | 由实际服务采集并刷新数据，权限受限时给出诊断 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。针对此应用使用的实际系统采集、刷新与权限契约接宿主。破坏性敏感检查在隔离自建进程或具名拒绝路径执行。

## 3. 技术与实现入口

目标入口为 src/front/sys_store.at、src/back/api.at 中 system_snapshot 与 kill_process。AV 腿走 **VM 桌面路线**（`auto run -r vm` + MCP，§4 ④）：原样 025 在真实 Iced 窗口中运行，受检 AGE / `ace_node_desktop` 不再是 025 的 AV 入口（§10 D-3）。VM 合并模式下后端在进程内直接 `sys.kill`，结束进程的授权门因此落在 VM 宿主侧（auto-lang 宿主能力槽 + auto-ui `run_vm` 装配，§8 T-02 第 5 项）。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-112/{auto-ui,auto-lang}，每仓单一 plan-112-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

实际系统采集及可授权的进程接口，按所用能力纳入。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

2026-11-23 用户通过 `/goal 完成计划112` 授权按 `/auto-plan-goal` 推进全链交付：在 `D:/autostack/.wt/ui-112/{auto-ui,auto-lang}`（附属只读仓 `auto-web`/`auto-down` 固定当前 HEAD）实施、独立复审、合同内有限修复与合并归档（不改变目标、验收、范围、仓集或预算）。

2026-11-23 用户裁定（work 阶段，针对 §10 D-1 / D-2，经交互确认）：
- **① 授权 PLAN-112 跨仓实现受检 AGE 缺口（D-1 方案 B）**：在 `D:/autostack/.wt/ui-112/auto-lang`（`plan-112-dev`，基线 `8a52d7364`）实现 §10 D-1 所列缺口，PLAN-113 先例。`.wt/ui-117` 正在 `age_component.rs` / `age_package.rs` / `frontdoor.rs` 上并行写入——本计划在这三个文件上的改动保持最小、局部，冲突留待后落地者 rebase。
- **② 授权 PLAN-112 在计划内修共享生成器语义（D-2 选项 ③）**：Plain 模式把 `table-*` 映射为原生表格元素、为 `alert-dialog*` 提供原生模态；T-02 / T-06 相应重开，须补生成器的统一回归。
- **③ AV 腿执行基面（2026-11-23，goal round 8，经交互确认）= §10 D-1 路线复核的选项 ①「叠在 ui-117 之上」**：PLAN-112 的 AV 腿工作基于 `plan-117-dev` 的已提交 tip，而非 master；取代上面 ① 条里「基线 `8a52d7364`」的基面约束（三个文件改动保持最小的约束不变）。时序：开发可在其 tip 上开始；AV 腿合入 master 须在 117 之后。已交付且不依赖 117 的 AutoUI 侧（授权门、lane store、生成器、A2Vue 证据）对 master 零冲突，可独立落地。
- **④ AV 腿入口改为 VM 桌面路线（2026-11-24，经交互确认）= §10 D-3 的选项 ①「VM 桌面路线」**：T-03 / V-03 在 `auto run -r vm` + MCP（原项目自带 `tests/desktop_mcp.py` 所走的路线）上取证，不再以受检 AGE 的 `ace_node_desktop` 作为 025 的 AV 入口。依据：原样 025 是动态类型数据流，受检 AGE 按定义会拒（§10 D-3）。随之而来：
  - §3 的 AV 入口表述、T-03 与 AC-02 的取证面按本裁定修订——属计划修订（已于 2026-11-24 落笔，见下条，含 plan_revision 递增）。
  - V-03 的「权限拒绝具名且 UI 可恢复」需要在 VM 宿主侧补一道结束进程授权门：VM 模式下后端进程内直接 `sys.kill`，今天没有拒绝路径；已交付的 `back_proxy::SysKillPolicy` 只管 HTTP 会话代理，不在合并 VM 模式的调用链上。
  - 已交付的受检面工作（`.as(T)` 与 `List<float>` 文法、同包记录类型、store model 归一、storage 宿主仿真、`plan112_av_upstream_status`）保留为上游贡献与实况记录，不再是 T-03 的前置；状态钉点继续如实描述受检面对 025 的现状。
  - 2026-11-24 修订已落笔（r2，§9 收据）：§3 AV 入口、§5 规范增量、§6 V-03、§7 AC-02、§8 T-02 第 4/5 项与 T-03 按本条改写。
- **⑤ VM 动态面同包记录类型解析修复（2026-11-24，§10 D-4 选项 ①，经交互确认）**：授权 PLAN-112 在计划内修 VM 动态构建的类型解析——前端同包兄弟单元（`types.at`）的声明须对前端单元可见且不被 `back.*` 同名类型静默遮蔽，作为 T-02 第 6 项；属共享运行时语义，须带永久用例与 025 真实回归，完成后再进 T-03。
- **⑥ store handler 异步化作为计划内新 phase（2026-11-24，经交互确认，§10 D-7）**：授权在 PLAN-112 内单开 **T-09**，把 VM 动态 UI 的 handler 派发从「同步原地」改为「片派发 + 帧泵续驱」（默认；`AUTO_CPU_LEGACY=1` 回退），补完成收尾接线、永久用例与规范；V-03 的计时证据在该派发下复跑。属共享运行时行为变更，须带全量动态 UI 回归与回退开关。

目标、验收标准与预算不变。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属；实际实现调整共享语义或组件合同时，按下表落规范并在合并门核（r2）。

| ID | 类型 | 目标 | 前 → 后 | 验收 |
|---|---|---|---|---|
| SD-01 | 新建 | `auto-ui:docs/specs/runtime/vm-sys-kill-authorization.md` | 无 → VM 宿主结束进程授权门：宿主能力槽（进程默认 + per-VM）、`AllowAll` / `Deny` / `AllowedPids` 三策略、缺席 = 旧行为逐字不变、拒绝在刷新/结束前以稳定码 `sys.kill.permission_denied` 具名（handler 中止、UI 存活）、`run_vm` 装配面（env 三态） | AC-02 / V-03 |
| SD-02 | 修改 | `auto-lang:docs/specs/auto-lang/vm/overview.md` | 宿主能力清单新增 sys kill 族一节：装配三层（显式 set 恒胜 / if-absent 补缺 / 实例快照）、缺席语义、shim 出口、与 HTTP 会话代理门的边界 | AC-02 |
| SD-03 | 已交付（补充边界） | `auto-ui:docs/specs/runtime/back-proxy-sys-kill-policy.md` | 交付文本只覆盖 HTTP 会话代理面；补明其不覆盖 VM 合并模式的进程内调用链（VM 面见 SD-01），两门策略语义与稳定码同源 | AC-02 |
| SD-04 | 已交付 | `auto-ui:docs/specs/ui/vue-plain-native-elements.md` | Plain 模式原生表格元素与原生模态投影（T-06） | AC-03 |

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced（VM 桌面路线） | 经 `auto run -r vm` 启动原项目真实 Iced 窗口，MCP 观察如下结果。实际采集的进程/系统数据随刷新出现（两次快照的值/行数变化、Tick 计数递增）；未授权结束请求由 VM 宿主门以稳定码 `sys.kill.permission_denied` 具名拒（error 日志 + `sys.kill` 返回 false、目标进程存活），handler 收尾照常（确认框关闭、刷新继续）；授权请求经 `sys.kill` 真实结束进程；关闭后运行进程退出、刷新任务随进程释放。保原实际适用操作（排序、暂停/继续、页签往返、主题声明默认）与 `sysmon.*` 持久化。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | 每个原项目/页面、原 oracle、适用端、实际前置和资源映射可复查。 |
| AC-02 | work | 所用接口及组件各有正反验证；真实 AV/Iced 的原行为、权限、恢复和关闭在 VM 桌面路线（`auto run -r vm` + MCP）上通过；受检 AGE 面只作上游实况记录（§4 ④ / §10 D-3）。 |
| AC-03 | work | A2Vue 的实际生成、构建、交互、失败和恢复按原矩阵通过。 |
| AC-04 | review | 收据与冻结版本对证，逐行核验缺失和临时代码，复审通过。 |
| AC-05 | merge | 可消费版本及两个目标的各自结果落主，各登记下游交接。 |

## 8. 执行步骤

当前 harness 如提供任务面板，开工时登记以下任务 ID。各目标按实际接口推进，阶段复核与最终复审分别覆盖对应结果。

- [x] T-01 固定原项目、依赖与验收矩阵
  - work，AC-01，核 §1 现有源和 §6 原运行入口，输出逐行的源、权限、预期和实际阻断。
  - 2026-11-23 完成：源冻结（16 tracked 文件 SHA256 前 16 位见 §9）；工作树 `D:/autostack/.wt/ui-112/{auto-ui,auto-lang}` @ `plan-112-dev`（基线 `auto-ui 912b02d7`、`auto-lang 8a52d7364`）+ `auto-web 0cfc7c7` / `auto-down 456c86a`（detached）；工具/环境冻结（`rustc 1.98.0 88d9e12ae`、`node v25.2.1`、Playwright/Chrome 在位、`CARGO_TARGET_DIR=C:/Users/zhaop/AppData/Local/Temp/ui-112-target` 因 D 盘剩余空间受限）。AV/A2Vue/`back_proxy` 首轮基线与阻断（B-01..B-04）实测记录见 §9。
- [x] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
  - 2026-11-23 进行中（AutoUI 侧四项已提交；AV 腿按 §10 D-1 裁定 = 方案 B 在计划内实现，逐项关闭中）：
    1. **权限面（原 B-02 的后端半）已解**（`381bcc09`）：`back_proxy::SysKillPolicy` 结束进程宿主授权门——按 app 登记；未登记即旧行为逐字不变；`Deny` / `AllowedPids` 在会话派发**之前**以 403 + 稳定码 `sys.kill.permission_denied` 具名拒（VM 后端与 OS 进程不被触碰）；随 `remove_app` 卸下；`BackProxyConfig` 字段面不变。真实采集仍由原样 `src/back/api.at` 经 `back_proxy` 会话完成（源零改动）。规范落点 `docs/specs/runtime/back-proxy-sys-kill-policy.md`。
    2. **A2Vue lane 接线（原 B-03）已解**（`d434252f`）：`generate_sfcs_with_lane_product_stores` 的 store composable 此前 `api_imports` 与函数池皆空（无 `@/lib/api` 导入、调用不带 `await`、裸调 helper）。现接通 `use back.api`（按首次出现去重——中断前的半成品会发射重复导入说明符，测试抓到）与「同文件 ∪ `use <unit>`」函数池（入口门控 + 闭包拉取，PLAN-671 ③ 同机制）。`@/lib/api` 前缀**不在产物里改写**：站点 `/api/*` 由代理按 standalone `auto run` 合同转到 `/apps/<id>/api/*`（半成品对生成物 `api.ts` 打字符串补丁的做法已弃用）。
    3. **生成器 Plain 模式投影（L-1）已解**（`dae048f0` + `190ed0dd` + `0b5a7ad2`，§10 D-2 选项 ③）：`table-header/body/row/head/cell` → 原生 `thead/tbody/tr/th/td`；`alert-dialog (open: X)` → `v-if` 门控的原生模态（遮罩点击关闭，取消/确认 `@click="($event) => { X = false; return H(…) }"`，拒绝经 Vue 错误处理浮出）；面板/标题/描述/按钮的 Plain 默认类。shadcn 模式（产品默认道）逐字不变。规范落点 `docs/specs/ui/vue-plain-native-elements.md`。
    4. **受检 AGE 面（原 B-01/B-04 与 B-02 宿主桥半）——2026-11-24 起冻结为上游贡献，不再是 T-03 的前置（§4 ④）**。以下为已交付事实与实况记录（提交见 §9），只作上游交接：
    - **语法**（auto-lang `a8e3b7457` + `cee68f25e` + `4a8e6ae5a`）：AGE 包面收 `.as(int|float|f32)`；类型实参位收 `float`/`f32`（`List<float>`）。nil 字面量由 PLAN-117 开到两面。
    - **语义①**（auto-ui `014f0a42`，叠基后）：纯 store 单元里的 `storage.get/set` 走宿主服务仿真（与组件面 prep 同一映射，内存态不落盘）。
    - **语义③**（auto-lang `f2f7c5593`）：同包记录类型对内联后的消费者可见——`sys_store.at` 直接用 `types.at` 的 `ProcInfo`，与 VM 前端一致；显式导入与本模块声明仍优先，同名歧义具名，不跨包。
    - **语义门⓪ model 形态**（auto-ui `86917e45`）：受检 AGE 不把 `var cpu_segs = []` 当状态、包面视图观察不了 `[]T` 字段——AV 包面装载前由宿主归一（方括号列表 → `List<T>`；无类型空列表按全部 push 点推断元素类型，记录字面量合成 `<Field>Row`）。
    - **旧语义②（handler 调用词汇）已由 PLAN-117 关掉**；nil 比较与记录字面量局部量在组件面也已可装载（原为钉住的缺口，现为对照）。
    - **现行首阻断**（生产 CLI `--package`，六单元原样）：`module app::types: field type annotation (…) is required`——`types.at` 的可选字段 `summary ?SysSummary`。其后依次：后端函数解析（`system_snapshot() -> Snapshot` 是记录返回值，117 的绑定生成只收标量与 `List<Str>`）、`fmt.at` 的 `f32.to_string()`/`str.sub()`、数值转换 Atom bind（TextRecord 描述符无 `expr.cast`）、handler 里的 `while` / `+=` / 带计算初值的局部 `var` / 局部字符串重赋值（`verify.borrow-escape`）、记录列表上的排序、子组件实例与条件视图、视图里对记录列表的 for 循环与 `${…}` 插值、SVG、Tick 定时器、B-02 宿主桥（后端会话 + 杀进程授权）。
    - 逐项最小复现与门序钉点 = `plan112_av_upstream_status.rs`（21 例）：只描述受检面对 025 的现状，其翻红/翻绿不构成本计划验收条件。
    5. **VM 宿主结束进程授权门（r2 新增；T-03 前置）——已完成**（auto-lang `783445c9e`、auto-ui `6cd0cec2`，两树干净）：
       - `auto-lang`（`crates/auto-lang-core`）：`vm/host_capabilities.rs` 新增 `SysKillGate`（`Allow` / `Deny { reason }`）、稳定码常量与拒绝诊断文本、进程默认槽；`NativeInterface` per-VM 槽（构造快照 + 实例覆盖 + Clone/merge 保留）；`shim_sys_kill`（2431）唯一消费点——缺席 = 旧行为逐字不变，`Deny` 在刷新/结束前记 error 日志（`sys.kill.permission_denied` + reason + pid）并返回 `false`（**软拒绝**，handler 收尾照常），`Allow` 走既有 `libs::sys::kill`。规范 `docs/specs/auto-lang/vm/overview.md` 新增「结束进程宿主授权门」节。
       - `auto-ui`：`crates/auto-ui-runtime/src/vm_sys_kill.rs`（`AUTO_UI_VM_SYS_KILL_POLICY` 三态解析与 `back_proxy::SysKillPolicy` → gate 适配，非法值具名失败）；`crates/auto-ui-tools/src/run_vm.rs::run_entry_file` 在进入解释器前装配。规范 `docs/specs/runtime/vm-sys-kill-authorization.md`。
       - 用例与实测：auto-lang `plan112_vm_sys_kill_gate` **5/5 过**（0.04–0.52s：缺席/`AllowAll` 真杀、`Deny` 具名且目标存活、`AllowedPids` 内外分流、构造期快照）；auto-ui `plan112_vm_sys_kill_policy` **4/4 过**（0.01–0.44s：三态解析/非法值、登记与清除、deny 目标存活、allowlist 内外分流）。回归：auto-lang `sys_natives_vm_tests` 3/3、PLAN-112 日常五目标 **37 过 / 2 排除**（2.54s）。
    6. **VM 动态面同包记录类型解析（§10 D-4 选项 ①，2026-11-24 裁定）**：`sys_store.at` 的 `ProcInfo` 解析到了 `back.api` 的 8 字段类型（`use back.api` 的预解析把该模块类型种子进共享 type_store），而不是同包 `types.at` 的 11 字段版本；带 `cpu_s/mem_s/disk_s/net_s` 的字面量因此整体构造为空记录（`one.pid` 仍 0），`filled` 恒空 → 进程表无行、右键结束不可达。修复面 = 动态构建（`dynamic_build` 的共享 type_store）的解析顺序：前端同包兄弟单元的声明须对前端单元可见且不被 `back.*` 同名类型静默遮蔽；本模块声明与显式导入仍最高；歧义具名（同 `source.module-ambiguous-type` 族）。永久用例：最小两单元复现（同包 `types` + store 用多字段字面量）+ 025 真实 store/视图（`procsView` 非空、表格出现数据行）；完成后重跑 T-03。
       - **已完成（auto-ui `6b8e6f2a`）**：动态构建先 walk 前端 `types.at`（`build_dynamic_component` 在一切 `use` 之前），并把 `src/back/*` 里与前端 `types.at` 同名的类型标识符在 parse 前限定为 `<module>__<Type>`（字符串/注释内同名词不动）。025 实跑：`procs`/`procsView` 非空（314 项）、表体出行；单测 `p112_qualify_type_idents_*` 与 `p112_front_sibling_type_names_*` **2/2**。
    7. **VM 表格行事件（D-4 续，2026-11-24）**：`convert_element` 的 table-row 分支把 `onclick`/`oncontextmenu` 硬编码为 `None`——行选中与右键菜单在 VM 路线不可达。**已修（auto-ui `7cdd813a`）**：行级事件按布局件同款提取，MCP 对渲染行的 press 现派发 `SelectProc`。残余（T-03 取证面）：`autoui_desktop` 的 pointer 右键注入在 standalone VM 窗口上未打开菜单（y∈{245,250,300,350,490} 全 false）——V-03 的「右键」一步拟用 fixture trigger `{widget:"ProcessesView", event:"RowCtx", input:"<pid>"}` 走同一真实 handler，证据里注明该注入差异；表体列宽/行高未对齐（截图可见）另记观感缺陷，不影响 oracle。
- [x] T-03 实跑 AV/Iced（VM 桌面路线）
  - work，AC-02，在 `auto run -r vm` 真实 Iced 窗口 + MCP 上按 V-03 取证，冻结可供 W1 复核的来源；前置 = T-02 第 5 项（VM 宿主授权门）。
  - 观察面（原项目冻结输入、零改源）：实际采集随刷新出现（两次 `autoui_snapshot` 的「已用 N MB」/进程计数变化；`autoui_state(tickN)` 随 tick 递增、暂停后停）；右键 → 菜单 → 确认框下，未授权 pid 被 VM 宿主门具名拒（稳定码 + pid，目标存活），取消后对话框关闭、刷新继续；授权 pid 经 `sys.kill` 真实结束；关闭窗口后 runner 以 0 退出、MCP 端点消失、无悬挂子进程；`AUTO_VM_STORAGE_FILE` 的 `sysmon.*` 跨重启恢复。
  - 取证脚本与日志落 `D:/autostack/plan-artifacts/plan-112/`（work 收据引用路径与 SHA256 前 16）；不以合成夹具冒充原项目实跑。受检 AGE 面此前的阻断记录保留在 §9 round 8 与 §10 D-3，不再作为本任务入口。
  - 2026-11-24 停在 §10 D-4：实跑发现进程表不渲染——`procCount` 298/300（真值），但 `procs`/`procsView` 为空，表体无行，右键结束路径不可达；「权限拒绝具名且 UI 可恢复」没有原操作路径。取证与观测见 §9 本轮回执；不以夹具绕过冒充已通过。**D-4 已裁定 ①（修动态 record-list 路径）= T-02 第 6 项；该项完成并验证后重跑本任务。**
  - 2026-11-24 取证第一轮（§9 round 5）：采集/刷新、具名拒绝（稳定码 + pid）、目标存活、授权放行、关闭释放均已取得证据；「取消后 UI 可恢复」在 30–40s 窗口未观测到（拒绝后帧停顿 ~20s），`sysmon.*` 重启恢复本轮不稳定。见 §10 D-5。
  - 2026-11-24 收口轮（§9 round 14）：新存储 + 同步派发下复跑——Phase A 核心全绿（302+ 真实进程、`RequestKill`、具名软拒绝 + pid、victim 存活、handler 自动关框、拒绝后刷新继续 64→66、关窗退出）；Phase A 末尾与 Phase B 的按键/持久化 3 项受环境时序影响未过（与拒绝/授权语义无关）。软拒绝契约（§10 D-7）下「取消」一步已并入 handler 收尾，不再是独立 oracle。
- [x] T-04 独立复核 AV 阶段
  - review，AC-04，按 VM 路线的实际入口（`auto run -r vm` + MCP）与冻结版本核原行为、反例、资源和完整 AV 行；复核 V-03 三段证据（刷新、拒/放、关闭释放）。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
  - 2026-11-23 完成（`833395ba`，`plan112_a2vue_sites` 三档）：
    - **生成**（日常档，2.50s）：lane 产品（`App` / `ProcessesView` / `PerformanceView` + `useSysStore`，**原样落盘、零改写**）+ 官方 TS 客户端（`auto_web::generate_api` 自原样 `src/back/api.at`）+ 最小 vite/tailwind 脚手架。
    - **冷构建**（显式构建道，4.35s）：空 `dist/` 起 `vite build`（vite 5.4.21，20 模块，CSS 13.2KB / JS 104.7KB）。
    - **真实浏览器**（浏览器道，7.82s；连跑 3 次 7.82 / 7.80 / 7.48s 全绿）：系统 Chrome（Playwright 1.61）+ 同一 `back_proxy`（原样后端 + `AllowedPids`），29 项检查全过。原 `tests/smoke.spec.ts` 的适用 oracle：T1 壳/导航/退役页负 pin、T2 KPI + `已用 N MB`（N>0）+ 120 行真实进程、T4 性能页 SVG path、T5 往返、T6 速率/暂停/继续、T7 的声明默认 dark 主题部分；T3 按现 UI（右键 → 菜单 → 确认框）取消复位。本 Plan 的 oracle：**真服务**——轮询在跑（快照 1→2）；**权限受限**——未授权隔离子进程的结束请求得 403 + `sys.kill.permission_denied` + 目标 pid，进程存活，拒绝经生成客户端的既有错误路径浮出（应用层恰一次 `Error: HTTP 403`），确认框经「取消」复位、仍显示「后端正常」（UI 可恢复）；**授权**——授权隔离子进程经后端真实结束（200 `true`）；**关闭**——卸载后零快照请求（9 → 9）、页面已卸载；无非预期页面异常。
    - **原收据逐行对照**：T1/T2/T4/T5/T6 ✓；T3 的「结束任务按钮」已随 PLAN-684 右键改版过时，按现 UI 修定；T7 的换肤驱动经 dev 构建的 `setupState`，生产包（inlineTemplate）不可达，只验声明默认主题；`tests/desktop_mcp.py` 属旧 VM 桌面路线（`auto run -r vm` + MCP），非本 Plan §3 的 AV 入口，未运行。
    - **L-1 已修**（§10 D-2 选项 ③，`dae048f0` / `190ed0dd`）：生成器 Plain 模式落地后**复跑**（最终代码下连跑 3 次全绿）——冷构建 7.09 / 3.31 / 3.30s，浏览器 8.18 / 7.19 / 7.26s，**32 项检查全过**，选择器即原 smoke 的结构口径：`table tbody tr` 120 行、表头排序、`<thead>`、真实 `<button>`、模态 `role="alertdialog"`（初始不在 DOM → 右键定位 → 可见且含标题与目标 → 取消后关闭）；未授权结束 403 具名 + 对话框已关闭 + 仍「后端正常」；授权结束 200 `true`；拒绝经 Vue 错误处理浮出（应用层恰一次 `Error: HTTP 403`，**无未处理拒绝**）；卸载后零快照请求。截图 `shot.png` 为真实表格。此前「按 lane DOM 等价改写」的说明作废。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。
- [x] T-09 store handler 异步化（§10 D-7；**用户 2026-11-24 指示在计划内单开 phase**）
  - work，AC-02，把 VM 动态 UI 的 handler 派发从「同步原地」改为「片派发 + 帧泵续驱」：长 handler（`Refresh` 的 `system_snapshot` + 排序）不再占满 UI 线程，025 的消息泵从 8/8 卡死降到 0/8，T-03 的 V-03 在默认派发下复跑。
  - 子项：
    1. 抽尾：`dynamic.rs` 同步派发后的收尾（trace、dirty、`dispatch_pending_emit`、`on<event>` 实例回调、被剥离 payload 回送、`sync_route_params`/`consume_nav_back_pending`）抽成 `finish_handler_dispatch(...)`，参数化（widget/event/emit/state/args/input/stripped/is_trace）。
    2. 片派发接线：生产派发默认走 `call_handler_for_cpu_slice`；`AUTO_CPU_LEGACY=1` 回退旧同步入口。未 park → 立即 `finish_handler_dispatch`；park → 记 `pending_finish`，在 `resume_cpu_slices` 报告 `completed>0` 的帧补收尾（只跑一次）。
    3. 用例：>片预算的长 handler 在 update 内 park、帧泵续跑完成、收尾只跑一次；短 handler 与 legacy 逐字一致；`AUTO_CPU_LEGACY=1` 全回退。
    4. 证据：025 `IsHungAppWindow`/`SendMessageTimeout(WM_NULL)` 从 8/8 卡死降到 0/8；T-03 Phase A（具名软拒绝 + 收尾 + 刷新继续）在默认派发下通过。
    5. 规范：`docs/specs/runtime/` 新增派发契约（异步语义、完成/收尾时机、回退开关、与 emit 路由的次序）。
  - 验收：V-03 在默认派发下通过；动态 UI 相关 lib 用例与 `desktop_behavior` 全绿；`AUTO_CPU_LEGACY=1` 回退可复现旧行为。
  - **已完成（2026-11-24）**：`48b97dc8`（抽尾）+ `99efbc99`（片派发接线）+ `c097d474`（F-01 本帧有界泵，默认开启；`AUTO_CPU_LEGACY=1` 回退）+ `70fd4d8d`（规范）。025 消息泵 `IsHungAppWindow`/`WM_NULL` **0/8**（最坏 356ms）；T-03 全量 **21/21**；lib 256/256、`desktop_behavior` 11/11。
- [ ] T-10 MCP 快照同步按需/限流（§10 D-8；用户 2026-11-24 提出）
  - work，AC-02，把 `projection.rs` 的每脏帧 MCP 同步块改成「按需 + 限流」：快照/状态只在 MCP 客户端真的请求时重建（或至多 500ms 一次，与既有「至多 500ms 陈旧」契约一致），静止/无客户端时零重建。
  - 依据（T-09 后实测）：025 消息泵最坏 `WM_NULL` RTT 默认 631ms、`P530_NOMCP=1` 378ms——MCP 同步约占最坏帧的 40%（313 行 ~100KB 快照每脏帧重建）。
  - 子项：1) 现状梳理（`view_dirty`/`gate_ws`/`has_view` 门与 500ms hot_reload 桥）；2) 按需或 500ms 限流实现（保留 `P530_NOMCP=1` 诊断开关）；3) 用例：快照新鲜度 ≤500ms、无客户端零重建、既有 MCP 工具用例全绿；4) 证据：025 最坏 RTT 与快照请求延迟对比。
  - 验收：无客户端/静止时零重建；请求路径的快照新鲜度满足 500ms 契约；MCP 工具回归全绿。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-23 stage: work | plan_id: PLAN-112 | plan_revision: r1 | checkpoint: T-01 冻结矩阵与首阻断 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-112/{auto-ui,auto-lang}`（`plan-112-dev`）+ `auto-web 0cfc7c7` / `auto-down 456c86a`（detached） | base_commit: auto-ui `912b02d7693e90bcaf10ec6ef082b05355284932` / auto-lang `8a52d7364e5b218e0a08b167038a5d71d54ee72f` | code_commit: -（探针观测，尚未提交功能代码） | task_ids: T-01 | evidence: `D:/autostack/plan-artifacts/plan-112/probes/t01-probe.log`（`plan112_t01_probe` 3/3 passed in 1.44s）

**源冻结（16 tracked 文件，SHA256 前 16 位）。**
- `SPEC.md` `FE855741B9340FEE`（10990B）、`pac.at` `68F19FC71305D696`（1004B）
- `src/back/api.at` `3DB4324BB4F7A472`（2772B）、`src/back/sys_info.at` `FBF306D9EFFF5BBA`（275B）
- `src/front/app.at` `0F3A4305E11672AB`（9330B）、`src/front/details_users.at` `E1D6DB441027322A`（5235B）、`src/front/fmt.at` `F6D164302D5045E8`（1347B）、`src/front/performance.at` `E153BD91A70FDB62`（16641B）、`src/front/processes.at` `24E11464DF3398BE`（7404B）、`src/front/sys_store.at` `57617E8F7A48C16A`（20640B）、`src/front/types.at` `9CF27DA973FA3F96`（853B）
- `tests/desktop_mcp.py` `F3A7164AD7007AB9`（12107B）、`tests/package.json` `CE619A43A70AD9BF`（253B）、`tests/playwright.config.ts` `AB5ACE6AAD2F5A14`（660B）、`tests/pnpm-lock.yaml` `CBBE12A5226B3826`（994B）、`tests/smoke.spec.ts` `8227A301D2EDC25D`（5678B）

**两端与后端入口基线实测。**
| 入口 | 实测结果 | 首阻断与归属 |
|---|---|---|
| `back_proxy` (`src/back/api.at`) | `GET /apps/025-sys-monitor/api/system/snapshot` 200 OK（52554B 真实 sysinfo JSON）；`POST /apps/025-sys-monitor/api/system/kill` `{"pid":0}` 200 OK（`false`） | 无阻断；缺主机权限策略门（默认未授权/拒绝诊断）供 V-03/V-04 |
| A2Vue lane (`generate_sfcs_with_lane_product_stores`) | 合并前端源生成 `App`(27868B)/`ProcessesView`(7242B)/`PerformanceView`(14283B)/`SysStore`(16052B) | 需接 `fmt.at` 导出(`n1`/`rate`)、`@/lib/api` 代理前缀与 shadcn-vue 脚手架并跑通 Chrome Playwright |
| A2Vue pkg (`generate_package_sfc`) | `source.age.component-unsupported: model has no field store` | 同 AV AGE 包面共享写面（B-01） |
| AV 组件面 (`NodeDesktopBridge::load`) | `cross-unit store loading is not wired yet (S1)` | `sys_store.at` 含 `use back.api`/`var x = []`/`storage`/`{st,color}`/`as(float)` 未能通过组件面内联 |
| AV 包面 (`NodeDesktopBridge::load_package`) | `model has no field store. Declare the store in the same package unit.`；单单元逐个实测：`sys_store`=`expected fn, found store`、`fmt`=`f32.to_string()`/`str.sub()` 未定义、`types`=`?SysSummary` 可选字段拒、`app/processes/performance`=缺 store | **B-01**（`auto-lang` `age-package` 共享写面，`.wt/ui-117` 与 `.wt/ui-118` 正在活跃修改）；**B-02**（AV/WidgetSession 缺少 `auto.sys.*` 采集与授权 kill 宿主桥） |

**主检出预检。** `auto-ui` 主检出仅 `docs/plans/{116,118,README}.md` 有他会话簿记改动；`auto-lang` 主检出仅 `docs/design/**`、`docs/plans/**` 有他会话文档改动——零代码路径 WIP。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: -（基线 auto-ui 912b02d7 / auto-lang 8a52d7364） | next: work/T-02+T-06

2026-11-23 stage: work | plan_id: PLAN-112 | plan_revision: r1 | outcome: **blocked** | worktree_path: `D:/autostack/.wt/ui-112/{auto-ui,auto-lang}`（`plan-112-dev`）| base_commit: auto-ui `06ba8ef99682186028c3ae69d914e2b3d6ed3692`（T-01 基线 `912b02d7` 之后 1 个提交）/ auto-lang `8a52d7364e5b218e0a08b167038a5d71d54ee72f`（**未改，零触碰**）| code_commit: auto-ui `381bcc09`（授权门 + 规范落点）→ `d434252f`（lane store 发射）→ `74edecf6`（AV 上游状态钉点）→ `833395ba`（A2Vue 站点三档）→ `971765e3`（探针根目录 Drop 守卫）；tip `971765e3471a43275ede362262034fbbe3a5e054`，工作树干净 | dependency_revisions: auto-web `0cfc7c7`、auto-down `456c86a`（detached，只读）| task_ids: T-02（partial）、T-03（blocked，未运行）、T-06 | spec_delta: 新建 `auto-ui:docs/specs/runtime/back-proxy-sys-kill-policy.md`（在分支，未发布）| blockers: §10 D-1（AV 腿上游）、§10 D-2（lane 投影保真 L-1）| next_responsible_role: user（§10 裁定）

**续接核对与半成品处置。** 本轮续接被中断的 work 阶段：工作树留有未提交半成品（4 改 + 3 新测试，无收据、未记入 Plan）。逐项核对后保留两处实现方向并修正，三份半成品测试全部重写（原文备份 `D:/autostack/plan-artifacts/plan-112/probes/draft-tests/`）。修正清单：
1. lane store 发射出 `import { system_snapshot, kill_process, system_snapshot, kill_process }`——合并源里 `app.at` 与 `sys_store.at` 各有一行 `use back.api`，未去重即重复导入说明符（JS 语法错误）；测试首跑抓到，按首次出现去重。
2. 半成品站点测试对**生成物** `lib/api.ts` 打字符串补丁（把 `/api/` 改写成 `/apps/…/api/`、并把非 2xx 吞成 `false`）——证据会变成「关于补丁代码的证据」。弃用：产物零改写，`/api/*` 由测试服务器按 standalone `auto run` 的代理合同转发（PLAN-116 同口径）。
3. `SysKillPolicy::Deny` 的 host 自定 `code` 收敛为稳定码 `sys.kill.permission_denied`；路由常量化；策略随 `remove_app` 卸下。
4. 半成品负例以**测试进程自身 pid** 作未授权目标——授权门一旦失效，后端会结束测试进程；改为隔离自建子进程 + 「目标仍存活」断言。
5. 本轮一度给 `BackProxyConfig` 加策略字段后撤回：该结构体在 18+ 处被穷举字面量构造（`back_proxy_tests.rs`、`run_vue.rs`），加字段即全部编译失败。保持字段面不变，只走 `RunningProxy::set_sys_kill_policy`。
6. 打印式探针 `plan112_t01_probe`（只打印、恒通过、硬编码 scratch 路径）替换为断言式状态钉点 `plan112_av_upstream_status`。
7. （本轮自有的测试卫生问题，收尾清点时发现）状态钉点的探针根目录原在断言**之后**才删除——钉点翻红、断言失败时恰会漏临时目录；改为 Drop 守卫（`971765e3`）。

**AV 腿缺口的再定位（修正 T-01 的归因）。** T-01 把首阻断记为「`age-package` 共享写面缺 store」。本轮二分（三组探针，最后一组一律走 prep → load 生产路径——裸源直送会把 prep 已仿真的构造误报成缺口）修正如下：跨单元 store **并非整体缺失**——最小跨单元 store 在包面可编译、同单元 store 的 handler 调用/字段显示/`storage.set`/`while` 经 prep 可装载；真实 025 撞的是 store 体与视图里一组具体构造的上游缺口（逐项见 §10 D-1）。另：`f32.to_string()`/`str.sub()` 与可选字段 `?T` 确认不在 profile。

**A2Vue lane 接线（V-04 生成/构建面）。** 站点 = lane 产品原样落盘 + 官方 TS 客户端（`auto_web::generate_api` 自原样 `src/back/api.at`，同一单算法；它顺带在 scratch 工作区生成 Rust server 骨架，只落文件不构建）+ 最小 vite/tailwind 脚手架。`auto run --render vue` + `AUTO_UI_ACE_VUE=1` 的正式工具道目前**无法**在 lane 键下生成多单元 store 应用：T-01 探针实测单文件 `app.at` 过 lane 时 7 个 handler 全缺席（都是 `store.X()`），按工具键门规则（`ace.vue-lane-unsupported`，见 `plan980_ace_vue_tool_wiring` ④）应被具名拒——**本轮未实跑工具道，此为规则 + 探针的推断**。PLAN-105 当时同样记录「生产装配在 auto-ui-tools，待下一刀」。同批计划（105/113/116）一律以「合并单元 → lane 产品 → 测试脚手架」为 A2Vue 证据道，本计划沿用。

**验证台账（本会话实跑；profile 均为 `test`，debug + 增量）。**
| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| `cargo nextest run -p auto-ui-runtime --features ace-bridge --test plan112_sys_kill_policy --test plan112_a2vue_lane_store --test plan112_av_upstream_status --test plan112_a2vue_sites` | 26 过 / 0 败 / 2 排除（ignored 的构建档与浏览器档）| 四目标首建 134s；其后仅重链测试目标 21–45s/轮 | 未单列 | 2.505s | 含在用例计时内（Drop 兜底）| 2.505s（nextest Summary）|
| 同上 + `--run-ignored only -E '…cold_build \| …browser_e2e' --test-threads 1`（`AUTO_UI_ACE_WEB_E2E=1`）| 2 过 / 0 败 / 1 排除（日常档生成例）| 无（复用已建二进制；vite 冷构建 1.03–2.28s 含在用例内）| 未单列 | 12.174s（串行）| 含在用例内 | 12.174s；另连跑 2 次 10.855s / 10.545s 全绿 |
| `ui_tests::back_proxy_tests`（lib，`--lib`，libtest `--test-threads=1`）| 21 过 / 0 败 ×2（含真实 TCP 的 `http_e2e` 族 19 例，日常档默认排除）| lib 测试目标构建 115s | 未单列 | 3.22s / 2.67s | 含在用例内 | 见左 |
| 既有 lane 回归：`plan105_u03_sites`、`plan1072_vue_web`、`plan980_u0_002_product_sfc`、`plan105_p013_store_lane` | 11 过 / 1 败 / 1 排除 | 随上 | 未单列 | 1.837s | — | 1.837s |

**新例计时（28 例：5 + 5 + 15 + 3，均 ≤10s）。**
| 新例 ID | 行为与 oracle | 实测秒 |
|---|---|---|
| `plan112_sys_kill_policy::p112_kill_denied_when_host_holds_no_authorization` | `Deny`：403 + 稳定码 + host 原因 + pid；目标存活；快照面 200；另一 POST 路由到达后端（不带策略诊断）| 0.756 |
| `…::p112_kill_policy_follows_app_lifecycle` | `remove_app` 卸下策略，重登记的会话不继承 | 0.914 |
| `…::p112_kill_allowed_pids_denies_ungranted_and_terminates_granted` | 清单外具名拒 + 存活；清单内经后端真实结束 | 0.995 |
| `…::p112_kill_without_policy_keeps_legacy_behavior` | 无策略与显式 `AllowAll` = 旧行为（两个隔离子进程均被真实结束）| 1.080 |
| `…::p112_snapshot_is_real_sysinfo_and_refreshes` | 真实 sysinfo（核数 = per-core 读数数、内存总量）；`uptime_s` 随刷新增大；表未截断时隔离子进程可见 | 1.834 |
| `plan112_a2vue_lane_store::p112_lane_store_emits_same_file_helper_and_api_for_synthetic_source` | 合成正例：同文件 helper 与 API 导入/`await`；未被引用的 fn 不发射 | 0.519 |
| `…::p112_lane_store_plain_store_stays_free_of_api_and_helpers` | 合成反例：纯状态 store 无 `@/lib/api`、无 `await` | 0.520 |
| `…::p112_lane_store_inlines_use_imported_helpers_with_closure` | `fmt.at` 的 `n1`/`rate` 及其被调 `n2` 各定义一次；`storage` → `localStorage`；21 个动作出口 | 0.983 |
| `…::p112_lane_store_imports_backend_api_and_awaits_calls` | `@/lib/api` 导入（去重）+ 调用点 `await` | 0.991 |
| `…::p112_lane_widgets_bind_the_store_surface` | 三个 SFC 的 store 绑定、`backend_ok` 双分支、卸载清定时器、排序/右键/确认绑定、SVG `:d` | 0.996 |
| `plan112_av_upstream_status`（15 例：3 接受对照 + 10 缺口钉点（其一基于真实 `SysStore` 源）+ 2 真实源钉点）| 见文件头注 | 0.171–0.519 |
| `plan112_a2vue_sites::p112_a2vue_site_generates_from_original_sources` | 生成面：lane 产品文件、store 导入、官方 `api.ts` 的端点/相对 URL/抛错路径、零测试钩子 | 2.499 |
| `…::p112_a2vue_site_cold_build`（构建档）| 冷 `vite build`：`dist` 入口与资源、bundle 含业务标记与 `localStorage`/`clearInterval`、CSS 令牌 + 工具类；写源绑定清单 | 4.349（复跑 3.057 / 3.064）|
| `…::p112_a2vue_sys_monitor_browser_e2e`（浏览器档）| 真浏览器 29 项 + Rust 侧独立核对（`kill403`=1 / `kill200`=1、拒绝体的 code 与 pid、授权子进程已退出、未授权存活、应用层恰一次 403）| 7.824（复跑 7.798 / 7.481）|

**判别力。** lane store：改动前基线二进制的产物（`probes/a2vue/stores/useSysStore.ts`）无 `@/lib/api` 导入、无任何 helper，上表 4 条真实源断言逐条不过；首跑另抓到重复导入缺陷。授权门：每条拒绝例都配「无策略 → 真实结束」对照，门失效即红。浏览器档：卸载前一刻先证 250ms 档请求在流（6→9），卸载后 900ms 零增量；首跑时测试代理漏带 `Content-Length` 致后端读到空体，被 Rust 侧 `denial.pid` 交叉核对抓到（测试脚手架 bug，非产品）。

**回归分流（无一是本计划提交新增）。**
- `plan105_p013_store_lane::p105_p013_store_composable_and_binding` 红：第 69 行断言 `prepare_component_source` 为 013 内联出的 store 块数为 1、实得 0——该断言只依赖未改动的 prep 与 013 源，且执行在被改函数之前。未在基线二进制上复跑（磁盘受限），依据为依赖面分析。按其后半段断言另以临时探针核对（已移除）：我的改动下 013 的 `TodoStore` 出口、App/TodoList 的 store 导入、`v-for` 均在——且 013 的 store 同样首次正确导出后端 API。
- `http_e2e_back_proxy_real_031_native_ns_session` 在 `http_e2e` 族首跑红一次（`names` 返回空）；其后整模块串行 ×2、单例隔离 ×3（ambient 与去 `PYTHONPATH`/`PYO3_PYTHON` 两种 env）全绿，属既有的间歇性问题。其路径是 `/api/viewer/*` 且无策略登记，不经过授权门。归主线 triage。

**环境（记入，供复审与他人复用）。**
- 构建缓存迁出 `%TEMP%`：T-01 冻结的 `C:/Users/zhaop/AppData/Local/Temp/ui-112-target` 在本轮两次丢失产物（`libsqlite3-sys` 的 `bindgen.rs` 单独消失，连带整条下游链重编）。C 盘 97–99% 满、存储感知开启（`01=1`、`04=1` 清理临时文件），症状吻合「近期未访问的临时文件被清」。已同卷改名到 `C:/Users/zhaop/AppData/Local/ui-112-target`（瞬时、无重编），此后未再发生。本计划后续命令请用新路径。
- PLAN-113 记录的 ambient `PYTHONPATH`/`PYO3_PYTHON` 非 hermetic：本会话 shell 同样带这两个变量；T-01 探针在两种 env 下复跑结果逐项一致，本轮用例亦未见差异。
- 站点证据 `D:/autostack/plan-artifacts/plan-112/sites/025-sys-monitor/`（源、`dist/`、`shot.png`、`verify_p112.mjs`；SHA256 前 16：`shot.png` `FF8749A11C031903`、`dist/.p112-build.json` `7711019C1026E0DD`）。其 `node_modules` 是指向主检出 025 生成工程的 junction（不在任何工作树内）——**合并清场前须先摘链再删 scope**。API 生成 scratch：`sites/025-sys-monitor-api/`。日志：`probes/final-daily.log` `8B7D15A84AD09C92`、`final-tier.log` `393B6B2679E8B4F8`、`stability-{1,2}.log` `3D88FC999B4EC6D4` / `3F6DD4B1BC9B777E`、`regress-lane.log` `A32C8700A3FB9D20`、`regress-backproxy-e2e.log` `6DC7B77354FC1D0D`。

**共享写面披露。** `ace_vue_script.rs::generate_sfcs_with_lane_product_stores` 与 `plan-116-dev`（`3067ce58`）改的是同一处：116 只把同文件 fn 挂进池，本计划是其超集（同文件 ∪ `use <unit>` + API 导入）。后落地者 rebase 时以本计划形态为准、丢弃 116 的那一小段即可。

**未做/待办。** V-03 未运行（§10 D-1）；L-1 未修（§10 D-2）；统一批量回归归 review/merge 门；规范落点未发布。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 2 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 971765e3 | next: user（§10 D-1 / D-2 裁定）

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 3 | from: user | to: work | status: executing | outcome: pass（裁定确认：D-1 = 方案 B 跨仓实现 AGE 缺口；D-2 = ③ 计划内修生成器语义；授权记 §4）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 971765e3 / auto-lang 8a52d7364（未改）| next: work（T-02 续：生成器 Plain 模式表格/对话框 → AGE 缺口 → B-02 宿主桥 → T-03）

2026-11-23 stage: work（P1：生成器 Plain 模式原生投影，D-2 ③）| PLAN-112:r1 | outcome: pass（本切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`）；auto-lang 未改 | code_commit: `dae048f0`（生成器）→ `190ed0dd`（浏览器脚手架改回原 smoke 结构选择器）→ `0b5a7ad2`（规范落点）；tip `0b5a7ad23a9cdebb635c3146c99366ba72dc8d81`，工作树干净 | task_ids: T-02（L-1 项）、T-06（复跑）| spec_delta: 新建 `auto-ui:docs/specs/ui/vue-plain-native-elements.md`（在分支，未发布）| next_responsible_role: work（P2：AV 腿 AGE 缺口）

**实现。** `map_tag` 的 Plain 兜底加表格族与 alert-dialog 族的语义标签；`template_media` 新增 `alert-dialog` 模态臂（仿既有 popover 自绘 overlay 先例：`v-if` 门控 + 遮罩点击关闭）与模态内取消/确认按钮臂；`extract_classes` 加 Plain 默认类；`VueGenerator` 加 `native_modal_open` 栈。shadcn 模式不经过这些臂。

**本切片自抓的三处实现缺陷**（均由浏览器档或结构断言抓到，均已补成永久断言）：
1. 默认类的键写成了 kebab——`extract_classes` 按 schema 规范名（snake_case）匹配，未命中。
2. 按钮臂只发射 children，丢了位置参数文本（`alert-dialog-cancel "取消"`）→ 空按钮，「取消」按 role + name 不可达。
3. 关闭语义先发射成内联多语句 `X = false; H`——Vue 不调用内联体末尾的裸函数名：对话框关了、后端请求没发。改成 `return H()` 后 Vue 的表达式解析又拒 `return`；最终用箭头函数表达式 `($event) => { X = false; return H(…) }`，顺带让 async handler 的拒绝经 Vue 的错误处理浮出（与 shadcn 模式的纯方法处理器一致），而不是未处理拒绝。

**验证台账（本切片实跑；profile `test`）。**
| 测试模块 / 实际范围 | 结果与执行/排除数 | 准备/构建 | 测试 | 总墙钟 |
|---|---|---|---|---|
| `cargo tu`（`auto-ui-compiler` lib `ui_gen::`，生成器统一回归）| **改动前基线 900 过 / 0 败 / 2 ignored；改动后 900 / 0 / 2**（已核实确实重编：二进制含新字符串）| 基线冷构建 2m13s；其后增量 13–15s/轮 | 1.19s / 1.11s | — |
| PLAN-112 日常档四目标 | 27 过 / 0 败 / 2 排除 | — | 3.005s | 3.005s |
| PLAN-112 门控档（冷构建 + 浏览器，串行，`AUTO_UI_ACE_WEB_E2E=1`）| 2 过；最终代码下连跑 3 次 | — | 冷构建 7.09 / 3.31 / 3.30s；浏览器 8.18 / 7.19 / 7.26s | 15.27 / 10.51 / 10.56s |
| 受影响 lane 回归（`AUTO_UI_ACE_WEB_E2E=1`，含浏览器挂载）：`plan105_p013_store_lane`、`plan105_u03_sites`、`plan1072_vue_web`、`plan980_u0_002_product_sfc`、`plan113_026_vue_browser`、`plan980_bweb_jg_watch` | 14 过 / 1 败 / 1 排除 | — | 9.436s | 9.436s |

新例：`plan112_a2vue_lane_store::p112_generator_plain_native_table_and_modal_leave_shadcn_mode_untouched`（同一份源 Plain ↔ shadcn 对照；0.515s）。改动的例：`p112_lane_widgets_bind_the_store_surface`（原生表格/模态结构断言 + 按钮标签；0.766s）、浏览器档（见 T-06 行）。**受影响的已交结果**：PLAN-113 的 026 lane 浏览器证据（`p113_026_vue_browser_evidence` 9.43s）在新映射下复验通过；PLAN-1072 / PLAN-980 的 6 个浏览器挂载用例复验通过。唯一红仍是上一条收据里的基线既有 `p105_p013_store_composable_and_binding`。波及面清点：`examples/ui` 无一声明 `shadcn: off`（Plain 只经 lane 到达）；表格族标签仅 025 / 026 使用，`alert-dialog` 仅 025 / 027 / 041 使用。日志 `probes/tu-baseline.log`、`tu-after*.log`、`p1-daily5.log`、`p1-tier5.log`、`p1-stability-{1,2}.log`、`p1-regress-lane.log`。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass（P1 生成器切片）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 0b5a7ad2 | next: work（P2：AV 腿 AGE 缺口）

2026-11-23 stage: work（P2.1：AV 腿 AGE 缺口诊断普查）| PLAN-112:r1 | outcome: pass（普查切片；**未改 auto-lang**）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`）| code_commit: `cdf5b1ce`（状态钉点更正，17 例）；tip `cdf5b1ceaa25f6c51d77215733febf927b134b8b`，工作树干净 | task_ids: T-02（AV 腿项）| next_responsible_role: user（§10 D-1 路线复核）

**归因更正。** 推翻 T-01、本会话上一条收据以及 PLAN-116 §10 B-03 的共同说法「store 的 handler 体被包面文法拒」。更正后的门序见 §10 D-1 的更正块。要点：包面上真实 `SysStore` 只被 `nil` 比较与 `.as(float)` 两类构造卡住语法；真正的缺口是**一整类尚不存在的能力**（store handler 的后端 API 调用及其结果进入 store 状态、`storage` 宿主、记录类型可见性），而这与 `.wt/ui-117` 在途分支正在建造的能力高度重合。

**普查方法（含两次探针自身错误的更正，供复审与后续轮次避坑）。**
1. 逐臂普查首跑「全红」是假象：把 `raw[..model_start]` 当「头部」，实际带入了 `store SysStore {` 行，拼出重复 store 头。更正后 20/21 臂通过。
2. 随后一组单因素矩阵（位置 × 注释行数 × 空行数 × 注释字节长度 × 注释内容 / 非 ASCII / `use` 行）全部正常——矩阵本身有效，但曾被误读为「头部内容有问题」；真正的解释是第 1 条。
3. `Refresh` 的 else 块首切用 `rfind("else {")` 命中了嵌套的 else；改为花括号配对后 94/96 条通过，仅 `.as(float)` 两条红。
4. 带消费者的编译里，store 单元在后续门失败时只露出消费者侧的「model has no field store」；用「按声明序增量放回真实臂」才分出语义②与语义③的先后。
5. 探针里的中和（`nil` → `false`、去 `.as(float)`、`storage` → 字面量 / `__noop` 字段）只为露出下一道门，**不是实现**。

**钉点变化。** `plan112_av_upstream_status` 15 → 17 例：移除过宽的 `p112_av_gap_real_store_handlers_rejected_by_package_grammar`；新增 `p112_av_gap_package_grammar_rejects_nil_and_float_cast`（3 个最小反例 + 2 个对照）、`p112_av_control_real_store_parses_without_nil_and_float_cast`（真实 store 去掉 4 处后解析通过，并钉臂数 = 21）、`p112_av_gap_package_store_gates_in_order`（语义①②③）。全部 ≤1s。

**验证。** `cargo nextest run -p auto-ui-runtime --features ace-bridge --test plan112_av_upstream_status`：17 过 / 0 败，0.587s。本切片没有改任何产品代码，其余 PLAN-112 目标未重跑（输入未变）。日志 `probes/p2-census-{1..16}.log`、`p2-status-run2.log`。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 5 | from: work | to: user | status: executing | outcome: waiting（§10 D-1 路线复核；等待轮 1/3）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui cdf5b1ce | next: user

2026-11-23 stage: work（P2.2：G1 AGE 包面文法——`.as(T)` 与 nil 原子）| PLAN-112:r1 | outcome: pass（本切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`、auto-lang `D:/autostack/.wt/ui-112/auto-lang`（均 `plan-112-dev`）| code_commit: auto-lang `b02e7393006899ba9f2fd52681ae7e1235676a4e`（文法 + 永久用例 `plan112_package_cast_nil` + grammar id 重基线 + 规范 §7）；auto-ui `9c113354`（状态钉点 17 → 19 例）；两工作树干净 | task_ids: T-02（AV 腿项）| spec_delta: `auto-lang:docs/specs/ace-migration/m2-age.md` §7（在分支，未发布）| next_responsible_role: user（§10 D-1 路线复核）

**实现。** `auto-source.at` 只动 profile 列表：postfix 位加三条 `age-package` 限定的 `.as(int|float|f32)` alt（与既有 `.to(int)` 同形，类型词只用包面已有 token）；`nil-lit` 规则与其原子位 alt 增 `age-package`。未碰 `age_component.rs` / `age_package.rs` / `frontdoor.rs`（ui-117 并行写入的三个文件）。AGE 组件面不开。grammar id：age-package `2308100f…`→`b696727b…`；age-component `817d81cf…` 与其余 13 邻居逐字节不变（`plan981_grammar_scope` 自证）。

**本切片自抓并更正的三处归因错误。**
1. `.as(float)` 的消费者级断言首败时，我把它归因为「消费者显示了 float 字段」。那是一处独立缺口（已单独钉住），但换成 int 字段后真身露出：数值转换停在 Atom bind——AGE 两面的 HIR 域是 TextRecord，描述符没有 `expr.cast`（numeric / aggregates / auto-language 有），且以 `source.internal: generated Atom text failed to bind` 浮出；既有 `.to(int)` 同样停在此处。即：**开文法只把首阻断从语法红推进到语义门，`.as(T)` 本身仍不可消费。**
2. A2Vue 包面 `model has no field store` 此前写作「prep 不内联 SysStore」；实测是 `generate_package_sfc` 先走 routed 车道，车道用 `package_candidates` 对入口单元**单独**解析（`app.at` 单独解析时 `use sys_store: SysStore` 不可见）。
3. 首版把 `nil` 同时开到组件面。探针显示组件面 `if .label == nil` 仍红而 `.label = nil` 可解析——组件面 `if` 条件走另一条条件 primary 规则，只在原子位开口是半开表面。收窄为仅包面，AgeComponent 裁剪字节回到冻结值。

**钉点变化。** `plan112_av_upstream_status` 17 → 19：`…_package_grammar_rejects_nil_and_float_cast` → `p112_av_control_package_grammar_admits_nil_and_float_cast`（6 个包面语法探针 + nil 的 resolve 裁定）；`…_real_store_parses_without_nil_and_float_cast` → `…_real_store_parses_on_package_face`（原样、零中和）；新增 `p112_av_gap_package_numeric_cast_fails_at_atom_bind`（`.as(float)` / `.as(int)` / `.to(int)`）、`p112_av_gap_package_float_state_display_needs_clone`；`…_real_app_first_blocker_on_component_face` 改用真实 `app.at` 路径；`neutralize_syntax` 删除（不再需要）。auto-lang 新增永久用例 `plan112_package_cast_nil`（包面收 7 种拼写 / 组件面仍拒且带非空对照 / `nil-lit` 规则只在包面 / nil 与非可空字段比较是 `source.nil-op`）。

**验证台账（本切片实跑）。**

| 范围 | 结果与执行/排除数 | 备注 |
|---|---|---|
| auto-ui PLAN-112 四目标（`--features ace-bridge`；日常档） | **31 过 / 0 败 / 2 排除**，2.703s | 状态钉点 19 例全部 <1s；排除项 = 门控的冷构建与浏览器档（输入未变，未重跑）|
| auto-lang `ace-core --test conformance` | 905 过 / 5 败 | 5 败全在 `text_record_source`（`verify.double-move`）——**基线 `8a52d7364` 同样 5 败**（暂存本改动后连跑 3 次），非本切片引入 |
| auto-lang `ace-source`（`ace-core --lib --tests`） | 970 过 / 5 败 / 1 排除，10.458s | 同上 5 败 |
| auto-lang `ace-source-syntax-927` | 13 过，2.438s | 七 profile 语法差分 |
| auto-lang `ace-syntax` | 159 过 / 3 排除，2.559s | |
| auto-lang `ace-av`（`--no-fail-fast`） | 1602 过 / 27 败 / 23 排除，643.0s | 27 败中 22 与基线**逐例相同**（工具未预备 `prepared provider root missing`、`verify.double-move`、Python lane 探针等）；其余 5 是墙钟预算类（`ExecutionWall`）：基线隔离复跑通过，带改动隔离复跑 4 过 1 败，该 1 例单跑 ×3 全过（2.2–2.4s）——满组并行负载所致 |
| auto-lang `ace-a2x-core` / `ace-a2x-worker` | 285 过 / 7 排除，110.3s | AGE → TS 车道 |
| 生产 CLI `ace_node_desktop_window`（025 真实源） | `--package` → 语义①；`--source` → S1（均 exit 4） | 见 T-03 |

**方法说明（如实）。** auto-lang 的 `cargo t` 驱动拒绝检出目录外的 `CARGO_TARGET_DIR`，而其固定的检出内目标目录在主检出已有 44G、D 盘当时仅余 52G——故未走驱动，改为按 `scripts/test-groups.toml` 的组定义直接 `cargo nextest run` 到共享缓存 `C:/Users/zhaop/AppData/Local/ui-112-target`。代价：`--prepare-only` 的工具预备未做，依赖工具的用例按上述口径归类。auto-ui 侧按包过滤会让 nextest 编译该包全部测试目标，其间 rustc 报 `only metadata stub found for rlib dependency std`（整包并发编译时出现，未深究）——改为显式四个 `--test` 目标后正常。日志（SHA256 前 16，`probes/`）：`g1-ui-daily2.log` `CE1E820F9D45443E`、`g1-lang-conformance.log` `967C782F4599390F`、`g1-lang-av-nff.log` `87C22F9F4E821E8A`、`g1-lang-av-fails-baseline.log` `009CA082B7D39039`、`g1-lang-core-all.log` `44BC065E205F6FE4`、`g1-lang-baseline-trs-1.log` `6EBAB23816CDF1B5`。

**未做/待办。** 语义①–④ 与 B-02 宿主桥未动（执行基面待裁定，见 §10 D-1 的进展与重叠核对）；T-03 未运行；统一批量回归归 review/merge 门；规范未发布。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 6 | from: work | to: user | status: executing | outcome: waiting（§10 D-1 路线复核未获答复；等待轮 2/3。本轮已交付不依赖裁定的最小切片 G1）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 9c113354 / auto-lang b02e73930 | next: user

2026-11-23 stage: work（P2.3：G1 遗漏修复 + 语义① 宿主仿真 + 路线 ① 实测）| PLAN-112:r1 | outcome: pass（本切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`、auto-lang `D:/autostack/.wt/ui-112/auto-lang`（均 `plan-112-dev`）| code_commit: auto-lang `08343fac6b022d15efdb7969a1b8b320a973c953`（重生成嵌入文法数据）；auto-ui `5df4079db66fc8c4495f1e6f93d86db425adf32c`（语义① 宿主仿真 + 钉点 20 例）；两工作树干净 | task_ids: T-02（AV 腿项）| spec_delta: 无新增（宿主仿真是既有 AV 车道合同在包面的延伸；规范落点仍只有 `m2-age.md` §7）| next_responsible_role: user（§10 D-1）

**先更正上一条收据的一处遗漏。** G1（`b02e73930`）改了 `auto-source.at` 却没有重生成 `experimental/ace/aac/syntax/syntax_grammar_data.at`——AAC 前门嵌入的规范程序字节（含 `age_package_parts`），conformance 拿它与共享文法编译器逐字节比对。PLAN-113 每次改文法都重生成过；我上一轮写的「无生成步骤」只对 Rust 前门成立。已用 aac-syntax `generate-grammar` 二进制重生成并提交（`08343fac6`），文件与生成器输出逐字节一致。逐 profile 段对照（提交前 = master 血统；基线文法 = `8a52d7364`）：

| profile 段 | 提交前 | 基线文法重生成 | 本分支文法重生成 |
|---|---|---|---|
| `auto` | 陈旧 | ≠ 提交前 | = 基线重生成 |
| `age_package` | — | = 提交前 | ≠（G1 所致，预期）|
| `aac`（并集面）| 陈旧 | ≠ 提交前 | ≠ 基线重生成 |
| 其余 5 段 | — | = | = |

即 **master 自己的嵌入数据在 `auto` / `aac` 两段上早已陈旧**（上次重生成是 `d814b94a9` = PLAN-113；其后的文法提交 Plan 1111 `340a0be6c`、Plan 1127 `2a35bde44` 未重生成）。本分支提交的是完整规范输出，顺带把这两段修到位，不是手工只换一段。我曾假设这就是 `ace-av` 那批基线红的原因，**实测不成立**：重生成后同一 27 例仍 22 败，原因另在（工具未预备、`verify.double-move`、py-lane 探针等）。AAC conformance 比对道本身未跑（需预备工具）。

**语义①（AV 包面生产路径）已关。** 组件面的 prep 早已把 `storage.get/set("<key>")` / `Time.now_sec()` 改写为模型字段（PLAN-105「宿主服务仿真」），但只作用于入口源；store 另成包单元（`use sys_store: SysStore`）时 handler 里的 `storage.*` 到受检 AGE 停在 `storage calls require the owning host`。现在 `NodeDesktopBridge::load_package` 对**纯 store 单元**套同一映射（`ace_source_prep::simulate_store_host_services`）：恰一个 `store` 声明、无 widget/node、有 `model {` 块、且每个 `storage.` 调用的键都是字符串字面量；其余单元逐字节不动，诊断仍由 AGE 给出。**语义边界**：内存态、不落盘——AV 车道的既有合同（同单元 store 一直如此），不是持久化实现；025 的速度/页签/排序记忆在 AV 窗口里不跨重启保留。A2Vue 包面不走这条路径、不受影响。原样 025 在生产 CLI `--package` 上的首阻断由语义① 后移到语义②。

**钉点变化。** `plan112_av_upstream_status` 19 → 20：`…_real_app_first_blocker_on_package_faces` 的 AV 包面期望改为语义②；新增 `p112_av_control_av_package_path_simulates_store_storage`（同一源：裸 AGE 包编译仍报 storage 门 ↔ AV 包面装载不再报）。门序用例 `p112_av_gap_package_store_gates_in_order` 钉的是**裸 AGE 包编译**，门① 在那里仍成立——宿主行为与语言行为分开钉。`ace_source_prep` 新增 3 个单测（纯 store 单元改写、七种不改写形状逐字节不变、025 真实 `sys_store.at` 的 13 处调用 → 8 个字段）。

**路线 ① 实测（只读核对 + 临时 detached，测完已恢复 `plan-112-dev`）。** ui-117 两仓的已提交 tip（auto-lang `0a1b6dd9f5`、auto-ui `8754b11dc4`）上跑 025 真实源：
- 原样 tip：`--source` → `source.unsupported: \`use\` is not part of this source profile`；`--package` → `model has no field store`——117 的文法没有 `.as(T)`，`SysStore` 整条又是语法红。
- 叠上 G1 的三条 `.as` alt（未提交的临时改动）：`--package` → `storage calls require the owning host`，与本分支 G1 后同一道门。
- 结论：117 的已提交基面**今天**没有把 025 带过 ①–③；它带来的是语义② 的底座——`ace_back_api.rs` 把 `use back.api: f` 生成为经 back_proxy 作用域 URL 的 `Http.get_json` / `json.parse` 绑定单元，auto-lang 侧配套 `http_ops` / `json_ops` / TextRecord 描述符的 `nil`/`dyn`/`object` 类型（该分支 88 文件 +10327/−2049）。但它的绑定生成只收标量与 `List<Str>` 结果，025 的 `system_snapshot() -> SysSnapshot`（嵌套记录 + 记录列表）会被 `unsupported checked signature` 拒。

**合并面事实（`git merge-tree`，只读）。** auto-ui `plan-112-dev` × master：零冲突（10 前 / 26 后）。auto-ui × `plan-117-dev`（tip `c041fb6c4e`）：在 `9c113354` 时零冲突；`5df4079d` 之后**一处**冲突——`ace_source_prep.rs`，新函数插在 `rewrite_host_storage` 之后，与 117 在同一邻域的插入相邻（纯文本相邻；117 并未把 storage 仿真扩到包面 store 单元，无语义重叠）。auto-lang `plan-112-dev` × master：零冲突；auto-lang × `plan-117-dev`（tip `f26245cc4a`）：冲突 3 文件（`auto-source.at` 两行、`plan981_grammar_scope.rs` 同一枚 id 钉、`syntax_grammar_data.at`——解法是再跑一次生成器）。即：已交付的 AutoUI 侧对 master 可直接落地；与 117 的重叠只有一处相邻插入。

**另一条旁证。** 既有单测 `ace_source_prep::tests::real_u03_apps_load_through_desktop_bridge`（断言 015/018/022 能经 AV 桥装载）在 master 血统上是红的：022-kanban 的 `board_store.at` 与 025 同构（跨单元 store + `use back.api` + 记录列表），报 `no type named Card` / `no function named all_cards`——正是语义②③。去掉本切片、且 auto-lang 换到 master（`0486981556`）后同样红，与本计划无关；记为主线既有红。

**验证台账（本切片实跑）。**

| 范围 | 结果与执行/排除数 | 备注 |
|---|---|---|
| auto-ui PLAN-112 四目标 | **32 过 / 0 败 / 2 排除** | 状态钉点 20 例全 <1s；排除项 = 门控冷构建/浏览器档（输入未变）|
| auto-ui 其余调用 `load_package` 的目标：`plan124_routes`、`component_registry_test`、`plan980_closure_window`、`plan114_widget_packages` | 33 过 / 30 败 / 2 排除 | 30 败全是 `PREPARATION_REQUIRED`（UI980 provider 未预备 + 环境变量），与本改动无关 |
| auto-ui lib `ui::ace_source_prep::` + `ui::ace_node_desktop::` | 29 过 / 1 败 | 1 败即上面 022-kanban 的既有红 |
| auto-lang `ace-core --test conformance`（重生成后）| 905 过 / 5 败 | 同前：基线既有 `text_record_source` ×5 |
| auto-lang `ace-syntax`（重生成后）| 159 过 / 3 排除 | |
| 生产 CLI `--package`（025 真实源）| `handler calls require …`，exit 4 | 语义② |

波及面：`examples/ui` 里用 `storage.` 的 6 个工程，除 025 外都写在带 widget 的入口/页面源里（不走「纯 store 单元」分支）。日志（SHA256 前 16，`probes/`）：`r7-grammar-data-new.at` `D63D984996E638F2`、`r7-grammar-data-base.at` `86DB75A38EB25204`、`r7-on117-package.log` `53E39387505B6D31`、`r7-on117as-package.log` `81E133D7D71067DD`、`r7-ui-regress.log` `6150D1C9D995D493`、`r7-lib-base-A.log` `3C7A07155B1C5C39`、`r7-lib-base-B.log` `1492CE83D30C9672`、`r7-prod-package.log` `C1B2328811930F5B`。

**过程事故（如实）。** 做基线对照时我对**干净**的 auto-lang 工作树执行了 `git stash push`（什么也没存）再 `git stash pop`，弹出的是仓库共享 stash 栈里别人的旧条目（`stash@{0}: On plan-107-dev: PLAN-107 r1 … draft`），把不相干改动带冲突地套进了我的工作树。发现后用 `git reset --hard HEAD` 复原（我的工作全部已提交），stash 条目因冲突被保留、栈内 5 条未动，无数据丢失。教训：stash 栈跨工作树共享，干净树上不得用 push/pop 配对做对照；后续改用「单文件检出旧版」与「补丁存取」。上一轮的三次 stash 都是在有改动时做的，弹出前后状态已逐一核对，未受影响。

**环境（记入）。** nextest 整包并发编译时 rustc 报 `failed to mmap … .rlib: 页面文件太小，无法完成操作 (os error 1455)`——Windows 提交内存耗尽，不是代码问题（上一轮记的 `only metadata stub found for rlib` 同源）。`-j 3` 预构建后正常。

**未做/待办。** 语义②–④、视图与定时器、B-02 宿主桥、T-03 均未动；执行基面见 §10 D-1。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 7 | from: work | to: user | status: executing | outcome: blocked（§10 D-1 执行基面连续三轮未获裁定；本轮已交付不依赖裁定的全部切片：G1 补全、语义①）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 5df4079d / auto-lang 08343fac6 | next: user

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 8 | from: user | to: work | status: executing | outcome: pass（裁定确认：§10 D-1 执行基面 = ①「叠在 ui-117 之上」；2026-11-24 PLAN-117 已合入 master，即以新 master 为基面；goal 由用户恢复）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 5df4079d / auto-lang 08343fac6（叠基前）| next: work（叠基 → 重钉 → 继续关门）

2026-11-24 stage: work（P2.4：叠基到 PLAN-117 之后的 master；重钉；关门⓪/③ 与 `List<float>`）| PLAN-112:r1 | outcome: pass（本切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`、auto-lang `D:/autostack/.wt/ui-112/auto-lang`（均 `plan-112-dev`）| code_commit: auto-lang `4a8e6ae5a4c2772c974cc502e20d89859fb95f28`（master `d13488b7ee` + `a8e3b7457` → `cee68f25e` → `f2f7c5593` → `4a8e6ae5a`）；auto-ui `86917e45a9a0be009ea88afc698680c094a6fe01`（master `3d6ae6189c` + 13 个提交，末三个 `014f0a42` → `7274a620` → `86917e45`）；两工作树干净 | task_ids: T-02（AV 腿项）| spec_delta: `auto-lang:docs/specs/ace-migration/m2-age.md` §8（增类型实参 float/f32）、§9（同包记录类型可见）——在分支，未发布 | next_responsible_role: user（§10 D-3：AV 腿前提冲突）

**叠基。** `git rebase master`。auto-lang 2 个提交、冲突 4 文件：`auto-source.at` 取 master 版再加三条 `.as` alt（nil 部分已被 PLAN-117 取代）；`plan981_grammar_scope.rs` 重冻结 age-package id；`m2-age.md` 两节都留（117 的 §7，本计划改号 §8）；`syntax_grammar_data.at` 重跑生成器（对 master 版只有 `age_package` 与并集面 `aac` 两段不同）。auto-ui 11 个提交、冲突 1 处（`ace_source_prep.rs` 测试模块尾的相邻插入）。叠基前后 id：auto-lang `b02e73930` / `08343fac6` → `a8e3b7457` / `cee68f25e`；auto-ui 11 个全部重写（tip `5df4079d` → `014f0a42`）。

**新基面重钉（`plan112_av_upstream_status` 20 → 21 例）。** PLAN-117 改动了这些事实：nil 比较在组件面可装载、在包面 store handler 路径上编译通过（不再报 `source.nil-op`）；记录字面量局部量在组件面可装载——三处原缺口改为对照。局部字符串重赋值现在停在验证器（`verify.borrow-escape`）。旧门②（handler 调用词汇）已被 117 关掉。新出现门⓪：受检 AGE 不把无类型空列表字段（`var cpu_segs = []`）当状态，handler 一写就报 `assigns undeclared model field`。

**关门（三项，各有永久用例）。**
1. **语义③ 记录类型可见性**（auto-lang `f2f7c5593`，`packages/binding.rs`）：VM 前端把记录类型当 app 级全局（`sys_store.at` 直接用 `types.at` 的 `ProcInfo`，不导入也不 `pub`），而包绑定只认「本模块声明 > 显式导入」；store 内联进消费者后其 model 的记录类型无从解析。现在 AGE 面多一层**同包兜底**：恰一个同包单元声明则用之，多于一个报 `source.module-ambiguous-type`（点名），不跨包；本模块声明与显式导入仍优先。与 PLAN-981 对 AGE node 导出的立场一致。用例 `plan112_package_type_visibility`（8 例）。
2. **`List<float>`**（auto-lang `4a8e6ae5a`）：类型实参位此前不收 `float`/`f32`（保留字），宿主把 `[]float` 归一成 `List<float>` 后整条 store 又成语法红。age-package id 再冻结 `aa6bc267…` → `a021fe52…`。
3. **门⓪ model 形态**（auto-ui `86917e45`）：`prepare_store_units`（并入原 storage 仿真）在 AV 包面装载前归一纯 store 单元——只动 `model { … }` 里的字段行：`var xs []T` → `List<T>`（初值、行尾注释、换行保留；handler 局部量不动）；`var xs = []` 按全部 push 点与列表赋值推断元素类型，记录字面量合成 `<Field>Row`（`cpu_segs` → `CpuSegsRow { st str, color str }`），推不出就原样留着。单测 7 例（含 025 真实 store）。

**一处更正。** 叠基时 auto-lang 永久用例原断言「nil 与非可空字段比较报 `source.nil-op`」在新基面不成立，改为「编译通过 + 同 handler 的真类型错仍被报出」的非空对照；运行期真值未断言。

**验证台账（最终提交上实跑）。**

| 范围 | 结果与执行/排除数 | 备注 |
|---|---|---|
| auto-ui PLAN-112 四目标 | **33 过 / 0 败 / 2 排除**，2.256s | 排除项 = 门控冷构建/浏览器档（输入未变）|
| auto-ui lib `ui::ace_source_prep::tests::` | 14 过 / 1 败 | 1 败 = 既有的 022-kanban 桥装载红（round 7 已证与本计划无关）|
| auto-ui 其余走 `load_package` 的四个目标（`plan124_routes`、`component_registry_test`、`plan114_widget_packages`、`plan980_closure_window`）| **67 过 / 0 败**，11.9s | 预备 UI980 provider 后（`ace_node_desktop_window --prep-only` + `ACE_NODE_DESKTOP_PREPARED_RUSTC`）的真窗运行；预备前 34 败全是 `PREPARATION_REQUIRED` |
| auto-lang `ace-core` lib + tests | 1013 过 / 5 败 / 1 排除 | 同前：`text_record_source` ×5 |
| auto-lang 927 / syntax / a2x | 13 / 161（3 排除）/ 285（7 排除）全过 | |
| auto-lang `ace-av` | 1628 过 / 22 败 / 23 排除，296.6s | 22 败与 round 6–7 记录的既有集合逐例相同 |
| 生产 CLI `--package`（025 原样六单元）| `module app::types: field type annotation … is required`，exit 4 | 门⓪/①/③ 关后的下一道 |

日志（SHA256 前 16，`probes/`）：`r8-ui-daily5.log` `355E3F063CFAD1F1`、`r8-g0-unit3.log` `61AD741066CEC709`、`r8b-lang-core-all.log` `1076BCF82FFA4B5F`、`r8b-lang-av.log` `4C7CA60AAA31AA3F`、`r8-ui-regress.log` `9E417B3C6E5F5F84`、`r8-prod-package2.log` `29EBDFAAA55D0682`。auto-lang 各组是直接 `cargo nextest run` 到共享缓存（`cargo t` 驱动拒绝检出外缓存，见 round 6），故工具依赖类用例按 `PREPARATION_REQUIRED` / `prepared provider root missing` 归类。

**新发现（需裁定，见 §10 D-3）。** 为下一道门（`types.at` 可选字段）读原源数据流时发现：**原样 025 是动态类型数据流，前端类型声明并不描述运行期数据**——例如前端 `DiskInfo { name, mount, total_gb, used_gb }`，后端实际返回 `{ name, mount, total_mb, avail_mb }`，视图读的是 `d.avail_mb` / `d.total_mb`；`cores` 声明为 `[]CoreInfo` 而后端给 `[]float`；`users` 声明为 `[]UserInfo` 而视图当 `[]str` 用。VM 不校验所以能跑；受检 AGE 按定义会拒。也就是说，即便把剩余各门逐一关掉，原样源也过不了受检面——这是 §3「AV 走现有受检 AGE」与「按原冻结输入」两条前提之间的冲突，不是再多几道门的问题。

**VM 桌面路线可行性探测（为 D-3 选项 ① 提供事实；不是 V-03 证据）。** 本机已有项目运行器 `D:/autostack/auto-man/target/debug/auto.exe`（auto-man `gc879b3e`，11-19 构建）。直接 `auto run -r vm` 起不来：`auto-ui-tool discovery failed … not found`（运行器只找主检出的 `target/{debug,release}/auto-ui-tool.exe`，本机不存在）。在本计划工作树构建 `auto-ui-tool`（`cargo build -p auto-ui-tools --bin auto-ui-tool`，176s）并以 `AUTO_UI_TOOL_BIN` 指过去后可起：MCP 端点 1.5–3.0s 就绪（20 个工具），快照里 App 壳已渲染——侧栏「系统监视器 / Task Manager」、两个页签、速度按钮、暂停按钮、「后端正常」，以及真实数据 KPI「已用 … MB」（原测试 T1 的「真实进程数据」判据）。**未核实**：进程表的数据行——MCP 快照固定 7057 字符（3s 与 10s 相同，疑似深度截断），表体 `col` 在快照里为空，没有换别的工具（如 `autoui_find`）去数行；`autoui_state` 按字段名取 store 状态返回空，原测试用的是 widget 字段名，未深究。探测脚本与输出：`probes/vm_route_probe.py`、`r8-vm-route-probe3.log`、`r8-vm-route-snapshot.txt`、`r8-vm-route-run.log`。若选 ①，V-03 的「权限拒绝具名」还需要在 VM 宿主侧补一道结束进程授权门（VM 模式下后端直接 `sys.kill`，今天没有拒绝路径）。

**未做/待办。** 下一道门及其后（可选字段、后端函数记录返回值、`fmt.at` 内建、Atom bind 数值转换、handler 其余构造、视图/定时器、B-02 宿主桥）与 T-03 均未动，取决于 D-3。

stage: goal | plan_id: PLAN-112 | plan_revision: r1 | round: 8 | from: work | to: user | status: executing | outcome: waiting（§10 D-3 AV 腿前提冲突；等待轮 1/3。本轮交付：叠基、重钉、关门⓪/①/③）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 86917e45 / auto-lang 4a8e6ae5a | next: user

2026-11-24 stage: new（r2 修订：AV 腿改 VM 桌面路线；VM 宿主结束进程授权门入计划）| PLAN-112:r2 | outcome: pass | 依据 §4 ④ 用户裁定（D-3 选项 ①）| changed: §3 AV 入口、§5 规范增量 SD-01..SD-04、§6 V-03、§7 AC-02、§8 T-02 第 4/5 项与 T-03、T-04 | unchanged: §1 目标、§7 其余 AC、预算、仓集（auto-ui / auto-lang）| stale: T-02 第 4 项的受检 AGE 交付由「验收前置」降为上游实况，其证据只作贡献记录 | next: work（T-02 第 5 项 → T-03）

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 1 | from: user | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 86917e45 / auto-lang 4a8e6ae5a（未改）| next: work（T-02 第 5 项：VM 宿主授权门）

2026-11-24 stage: work（T-02 第 5 项：VM 宿主结束进程授权门；T-03 实跑停在 D-4）| PLAN-112:r2 | outcome: pass（授权门切片）/ blocked（T-03）| worktree_path: auto-lang `D:/autostack/.wt/ui-112/auto-lang`、auto-ui `D:/autostack/.wt/ui-112/auto-ui`（均 `plan-112-dev`，两树干净）| base_commit: auto-lang `4a8e6ae5a` / auto-ui `86917e45` | code_commit: auto-lang `783445c9e`、auto-ui `6cd0cec2` | dependency_revisions: auto-web `0cfc7c7`、auto-down `456c86a`（只读，未动）| task_ids: T-02（第 5 项，完成）、T-03（blocked）| spec_delta: 新建 `auto-ui:docs/specs/runtime/vm-sys-kill-authorization.md`；`auto-lang:docs/specs/auto-lang/vm/overview.md` 新增「结束进程宿主授权门」节 | evidence: `D:/autostack/plan-artifacts/plan-112/t03/`（`probe_structure.py` / `probe_screenshot.py` / `probe_state.py`，日志与状态 dump、窗口截图）| blockers: §10 D-4 | next_responsible_role: user

**授权门（T-02 第 5 项）实测。** `plan112_vm_sys_kill_gate` 5/5（0.043 / 0.055 / 0.501 / 0.502 / 0.516s）；`plan112_vm_sys_kill_policy` 4/4（0.012 / 0.018 / 0.062 / 0.441s）。回执首跑抓到自己两处测试缺陷并修正：目标原为 `cmd /C ping`（`sys.kill` 只杀壳、孙进程泄漏，nextest 记 3 例 LEAK）→ 改单进程 `ping` 目标；`allowed-pids:` 空清单原先报「empty pid entry」而测试期待「at least one pid」→ 解析器改为空清单单独具名。回归：`sys_natives_vm_tests` 3/3；PLAN-112 日常五目标 37 过 / 2 排除。

**T-03 实跑观测（原样源零改动；`auto run -r vm` + MCP）。** 窗口真实渲染（1600×1000，截图 `t03/probe2-window.png`），App 壳/两页签/速度与暂停按钮/「后端正常」、KPI 真实（「已用 11680 MB」「24.6 %」「298 进程」）且随 Tick 刷新（`tickN` 7→13）。**但进程表只有表头、表体空白**：

| 观测 | merged（默认）| split（`AUTO_VM_MERGE=0`）|
|---|---|---|
| `procCount` / `procLabel` | 298 / "298" | 300 / "300" |
| `procs` / `procsView` | `[]` / `[]` | `[]` / `[]` |
| `cores` / `users` / `disks` | 20 floats / 7 strs / 4 refs | 同左（非空）|
| 运行日志 | 无 `[VM-HANDLER]` 错误 | 同左 |

`.Refresh` 的 `.procs = snap.procs` 之后是 `for raw in snap.procs { filled.push(ProcInfo{…}) }` 再 `.procs = filled`；空值出现在这个循环产物上，而 `disks`/`users`/`cores` 三个**直接赋值**的列表都非空。现象与「跨单元返回 / `for` 迭代 record 列表」相关（根因待 D-4 ① 定位），不是数据源（KPI 与 `procCount` 为真值）、也不是 `procsView` 排序——后者只消费已空的 `.procs`。右键结束需要表体行，故 V-03 的「右键 → 菜单 → 确认框」与「权限拒绝具名且 UI 可恢复」在 VM 路线上没有原操作路径。T-03 停在此处，未以 fixture 打开确认框冒充已通过（选项见 §10 D-4）。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 2 | from: work | to: user | status: executing | outcome: blocked（§10 D-4；授权门切片已交付）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 6cd0cec2 / auto-lang 783445c9e | next: user

**D-4 根因定位（用户选 ① 后当日续查；诊断改动已 `git checkout` 复原，工作树干净）。** 临时探针（`sys_store.at` 加 `print`，随后还原）：

| 探针 | 结果 | 含义 |
|---|---|---|
| `snap.procs.len()` / `.procs.len()` | 309 / 309 | 数据源与直接赋值为真值 |
| `for raw in snap.procs { n = n + 1 }` | n = 309 | 循环本身可迭代 |
| `filled.push(ProcInfo{ 11 字段含 cpu_s… })` | 0 | 11 字段字面量构造失败 |
| `let one = ProcInfo{ 11 字段 }` → `one.pid` | 0 | 字面量整体为空记录 |
| `let b = ProcInfo{ 8 字段含 user }` → `4|nm|u` | 正确 | 后端 8 字段形状可用 |
| 显式 `use types: ProcInfo` 后 | 仍 0 | 显式导入不改变本次解析 |

即 VM 动态管线里 `sys_store.at` 的 `ProcInfo` 解析到了 `back.api` 的 8 字段类型（`use back.api` 的预解析把该模块类型种子进共享 `type_store`），而不是同包 `types.at` 的 11 字段版本；`cpu_s/mem_s/disk_s/net_s` 字段集因此构造失败（静默空记录），`filled` 恒空。修复面 = `dynamic_build` 的共享 type_store 解析顺序（同包 `types.at` 须可见/优先、不得被 `back.*` 同名类型静默遮蔽、歧义具名），作为 T-02 第 6 项；证据目录 `D:/autostack/plan-artifacts/plan-112/t03/`。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 3 | from: user | to: work | status: executing | outcome: pass（D-4 裁定 = ①，根因已定位；授权记录 §4 ⑤）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 6cd0cec2 / auto-lang 783445c9e（未改）| next: work（T-02 第 6 项：动态面同包类型解析修复 → 重跑 T-03）

2026-11-24 stage: work（T-02 第 6/7 项：D-4 修法 A——模块级类型身份 + 表格行事件）| PLAN-112:r2 | outcome: pass（两项切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净）| base_commit: `6cd0cec2` | code_commit: `6b8e6f2a`（类型作用域）、`7cdd813a`（行事件）| task_ids: T-02（第 6/7 项）| spec_delta: 待补（动态面类型作用域 + 表格行事件；规范落点仍 `vm-sys-kill-authorization.md` 之外另记）| evidence: `D:/autostack/plan-artifacts/plan-112/t03/`（`probe_state.py`/`probe_press.py`/`probe_ctx.py` 与日志、`probe2-window.png`）| next_responsible_role: work（T-03 取证）

**根因落定与修复。** D-4 的语言级根因：记录字面量字段集与解析到的类型不符时，VM 会**整体退化成最后一个字段值**（`A { x: 7, z: "extra" }` → `"extra"`），而动态管线把 TypeDecl 按名全局共享、先见者胜；`back.api` 的 8 字段 `ProcInfo` 先入，前端 11 字段字面量于是恒空。修法（用户裁定 A）：前端 `types.at` 先入；`src/back/*` 内同名类型标识符 parse 前限定为 `<module>__<Type>`。实测 025：`procs`/`procsView` 由 `[]` 变 314 项、表体出行（`probe2-window.png`）；单测 2/2。

**顺带修掉的第二个缺口。** 行事件：table-row 分支曾把 `onclick`/`oncontextmenu` 硬编码为 None，行选中/右键菜单不可达。`7cdd813a` 后 MCP press 渲染行返回 `handler: .ProcessesView/ProcessesView#0.SelectProc i <pid>`、status ok；`autoui_state` 的 `selected_pid` 需下一帧生效。

**残余（记入 T-03 取证面）。** ① `autoui_desktop` pointer 右键注入在 standalone VM 窗口未打开菜单（坐标网格 5 点全 false）——拟以 fixture trigger `{widget:"ProcessesView", event:"RowCtx", input:"<pid>"}` 走同一真实 handler，并在证据中注明注入差异；② 表体列宽/行高未对齐、窗口尺寸被撑大（观感缺陷，oracle 不依赖）。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass（T-02 6/7 切片；T-03 取证待续）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 7cdd813a / auto-lang 783445c9e | next: work（T-03 取证）

2026-11-24 stage: work（T-03 取证第一轮：VM 桌面路线真跑）| PLAN-112:r2 | outcome: blocked（V-03 的「UI 可恢复」未在时限内观测到；性能性卡顿）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净）| code_commit: `7cdd813a`（未改产品代码，仅取证脚本）| task_ids: T-03 | evidence: `D:/autostack/plan-artifacts/plan-112/t03/`（`probe_t03.py` 与 `t03-a-deny-run.log` / `t03-b-allow-run.log` / `t03-c-restart-run.log`、`probe_tick.py`、`probe_trigger.py`、`probe_stuck.py`）| blockers: §10 D-5 | next_responsible_role: work（诊断拒绝后的帧停顿）

**T-03 已取得的 V-03 证据（原样 025 源零改动；`auto run -r vm` + MCP）。**
| 观察项 | 结果 | 证据 |
|---|---|---|
| 实际采集随刷新出现 | PASS：`procCount` 299–349、KPI 真值、`tickN` 递增 | `autoui_state` 轮询 |
| 右键菜单 → 确认框 | PASS（注入面差异见下）：真实确认框出现且含 `结束任务` + victim pid | `RequestKill` 经 fixture trigger 走同一 handler（MCP pointer 右键无法命中 standalone 窗口） |
| 权限拒绝具名 | PASS：`[VM-HANDLER] ProcessesView.ConfirmKill failed: RuntimeError("sys.kill.permission_denied: the VM host holds no process-kill authorization (pid 913240)")` | `t03-a-deny-run.log` |
| 目标存活 | PASS：拒后 victim 仍活 | 子进程 `poll()` |
| 授权放行 | PASS：`allowed-pids:<victim>` 下确认后 victim 退出、对话框关闭、无拒绝日志 | `t03-b-allow-run.log` |
| 取消复位 / UI 可恢复 | **未通过（时限内）**：确认被拒后按「取消」动作 `status: ok`，但 30–40s 内快照仍显示确认框、帧停顿 ~20s 后 `tickN` 才从 7 跳到 26 | `probe_stuck.py` 输出 |
| 关闭释放 | PASS：关窗后 runner 退出（deny 相 code=0）、MCP 端点消失、无 `auto-ui-tool` 残留；关窗到退出约 33s | `probe_close.py`、v6 harness |
| `sysmon.*` 重启恢复 | 未通过（不稳定）：2.5s 按压返回 ok，重启后 `speedDiv` 仍 4（此前一轮曾得 10） | v6 harness |

**新发现（记入 §10 D-5）。** ① 拒绝后帧停顿可达 ~20s（正常时也 ~0.5–2s/帧，MCP 状态应答恒 ~2s）；② `autoui_state` 对 handler 写入的 store 字段（`selected_pid`/`kill_target`/`kill_dialog_open`）不刷新，快照才是对话框的 oracle；③ 快照对 ~100KB 有截断，按名字找行不可靠（偶发 miss）；④ 关窗到进程退出约 33s。

**回归（本轮实跑）。** 受影响 lib 模块 `dynamic_build`/`convert_element`/`aura_view_builder` **188 过 / 0 败**（5.05s）；`desktop_behavior` **11 过 / 1 排除**（0.08s）；025 真跑见上表。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: blocked（T-03：UI 可恢复未达；见 §10 D-5）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 7cdd813a | next: work（诊断拒绝后帧停顿）

2026-11-24 stage: work（D-5 诊断：帧耗去向 + 拒绝后恢复时延）| PLAN-112:r2 | outcome: pass（诊断切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净，未改产品代码）| task_ids: T-03 | evidence: `t03/probe_recover.py`、`t03/probe_perf.py`、`t03/probe_perf2.py`、`t03/probe12-run.log` | blockers: §10 D-6 | next_responsible_role: user（性能范围裁定）

**恢复时延（`probe_recover.py`，拒绝 → 取消 → 轮询 120s+）。** 取消动作 `status: ok` 后：对话框在 **175s** 才消失（`gone_at=175.5s`），刷新在 34.8s / 114.2s / 169.4s 才各跳一批（`tick` 13→30→48→60）。即「可恢复」在分钟级。

**帧耗去向（`probe_perf.py` / `probe_perf2.py`）。**
| 状态 | 单次截图（≈一帧） | `tickN` | 备注 |
|---|---|---|---|
| 运行中 | 4.68 / 9.58 / 12.08s | ~0.6/s（应 4/s） | 排序在跑 |
| 暂停（`running=false`，排序停） | 12.05 / 4.96s | 停止 | **仍秒级** |

暂停后帧耗不降 ⇒ 主因是**每帧全量重建 314 行视图**（1570 个单元格：每格 VM 绑定求值 + `Style::parse` + 非虚拟化布局）；排序是次要叠加（~1 次/s、O(n²)）。任何点击都排在这条重建后面，超过 Windows 5s 阈值即「未响应」。旧版 AutoVM 应用行数少，未暴露。范围与选项见 §10 D-6。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 6 | from: work | to: user | status: executing | outcome: blocked（§10 D-6 性能范围裁定）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 7cdd813a（未改）| next: user

2026-11-24 stage: work（D-6 诊断：行数/环境/帧基线）| PLAN-112:r2 | outcome: pass（诊断切片；临时改源已还原）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净）| task_ids: T-03 | evidence: `t03/probe_perf.py`、`probe_perf2.py`、`probe_perf3.py`、`probe_perf3-run.log` | blockers: §10 D-6 | next_responsible_role: user

**行数不是主因（临时把 `ApplySort` 输出限 12 行，测后已 `git checkout` 还原）。** 12 行时 `autoui_screenshot` 仍 4.77/9.80s（运行）、12.05/11.12s（暂停）——与 314 行同量级。

**环境基线（013-todo，小型 VM 应用，同机同法）。** `autoui_screenshot` 4.73/4.00/3.99s、`autoui_state` 2.05–2.09s；窗口同样 2560×1600。即**本会话任何 VM 应用的帧基线就是 ~4s**，`autoui_state` 的 ~2s 是工具固定节拍。

**图形环境。** `Win32_VideoController` = `ToDesk(R) Virtual Display Device`（2560×1440）+ `Todesk Virtual Display Adapter` + `NVIDIA GeForce RTX 4060 Ti`——当前是 ToDesk 远程会话，iced/wgpu 很可能跑在虚拟显示/软件路径上。

**结论修正。** 「点一下就未响应」在本会话里首先是**环境帧基线（~4s/帧、2560×1600 远程会话）**，其次是 025 的增量（5–12s），排序只是 ~1 次/s 的叠加项、不是主因；拒绝路径另有 175s 恢复（独立缺陷）。是否在本地 GPU 会话仍慢，需要对照组才能定性能计划的范围。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 7 | from: work | to: user | status: executing | outcome: blocked（§10 D-6：需本地/远程对照）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 7cdd813a | next: user

2026-11-24 stage: work（软拒绝：`sys.kill` 拒绝改为 false + 具名诊断）| PLAN-112:r2 | outcome: pass（授权门语义修正；T-03 仍受环境限制）| worktree_path: auto-lang `D:/autostack/.wt/ui-112/auto-lang`、auto-ui `D:/autostack/.wt/ui-112/auto-ui`（均 `plan-112-dev`，两树干净）| code_commit: auto-lang `22de0a7ba`、auto-ui `1b3fc406` | task_ids: T-02（第 5 项修订）| spec_delta: `vm-sys-kill-authorization.md` 与 `vm/overview.md` 的拒绝语义改为软拒绝 | next_responsible_role: work（T-03 环境受限）

**软拒绝。** `Deny` 不再抛 `VMError`，改为记 `sys.kill.permission_denied: <reason> (pid N)` 并返回 `false`——handler 的收尾（关对话框、清选择、`.Refresh()`）照常跑。理由：抛错会中止 handler，拒绝后 UI 停在确认框，用户需再点一次「取消」。用例两端同步：auto-lang `plan112_vm_sys_kill_gate` **5/5**、auto-ui `plan112_vm_sys_kill_policy` **4/4**。

**重要更正：175s 停顿不是拒绝路径特有。** `probe_cancel.py`（**不点确认**、只开确认框再按取消）在同一会话里 150s 内对话框也未消失；软拒绝前后行为一致 ⇒ 该停顿是**会话/模态渲染的环境问题**，不是 `VMError` 的错。软拒绝仍保留（收尾路径是产品上更对的行为），但 D-5/D-6 的「拒绝后 175s」归因撤回。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: pass（软拒绝切片；T-03 受环境限制）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 1b3fc406 / auto-lang 22de0a7ba | next: work（T-03 环境受限 / 复审准备）

2026-11-24 stage: work（帧阻塞定位：真实窗口消息泵实测）| PLAN-112:r2 | outcome: blocked（V-03 复跑仍受帧重建阻塞）| evidence: `t03/probe_find_window.py` | next_responsible_role: user（§10 D-6 裁定）

**消息泵实测（025 真实窗口，hwnd title=系统监视器，1293×836）。** `IsHungAppWindow` **8/8 true**；`SendMessageTimeoutW(WM_NULL, 1000ms)` **8/8 超时**（ERROR_TIMEOUT 1460）。用 `P530_NOMCP=1` 跳过每帧 MCP 快照同步后同样 8/8 卡死（最坏 999ms 超时）。
⇒ **UI 线程被每帧视图重建本身阻塞**（314 行 × 5 格 ≈ 1570 个单元格重建 + 布局），既不是排序独占、也不是 MCP 快照同步；软拒绝/授权门不在调用链上。小应用对照（013-todo）本轮未取到窗口，未形成对照（如实记录）。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 10 | from: work | to: user | status: executing | outcome: blocked（§10 D-6：帧重建阻塞 UI 线程，V-03 无法复跑）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 1b3fc406 | next: user

2026-11-24 stage: work（handler 同步派发 = UI 阻塞主因；CPU-slice 派发实证可解）| PLAN-112:r2 | outcome: pass（定位切片）| evidence: `t03/probe_find_window.py`（默认 8/8 卡死；`AUTO_CPU_PROBE=1` **0/8 卡死，最坏 897ms**）| next_responsible_role: user（是否把 CPU-slice 派发转正）

**根因收敛（与用户判断一致）。** `dynamic.rs:3629` 生产派发走 `call_handler_for`（同步、原地执行）；`AUTO_CPU_PROBE=1` 时才走 `call_handler_for_cpu_slice`（PLAN-711 T-04/R-1 探针）：片耗尽 park 成 `CpuRunnable`，帧泵续驱。实测同一窗口：同步派发 8/8 消息泵卡死；CPU-slice 派发 **0/8**（最坏 897ms）。⇒ 025 的 `Refresh`（`system_snapshot` + O(n²) 排序）在 merged 下是**同步内联**执行，占满 UI 线程；视图重建是第二层成本。**不是**排序独占、**不是** MCP 仪表（`P530_NOMCP=1` 仍卡死）。

**merged API 调用语义（代码证据）。** `host_bridge::call_host(name, args)` → `f(args_json)` **同步调用**；025 merged 下 `system_snapshot()` 在 UI 线程内联返回。split 模式下 `#[api]` 改写成 HTTP，VM `await` 挂 `ParkedWait::Future`，UI 继续——即用户记忆里的「逻辑异步」在 split 才成立。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 11 | from: work | to: user | status: executing | outcome: blocked（§10 D-6/D-7：同步派发 vs CPU-slice）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 1b3fc406 | next: user

2026-11-24 stage: work（T-09 第 1 步：派发收尾抽函数）| PLAN-112:r2 | outcome: pass（切片）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净）| code_commit: auto-ui `48b97dc8` | task_ids: T-09（第 1 步）| evidence: `dynamic`/`convert_element`/`aura_view_builder` lib 用例 **256/256 过**（5.07s）；T-03 复跑再次 `0xcfffffff` 断连（环境/应用崩溃复现）| next_responsible_role: work（T-09 第 2 步：片派发接线 + 完成补收尾）

**T-09 第 1 步完成。** `dynamic.rs` 的同步 handler 收尾（trace、dirty、`dispatch_pending_emit`、`on<event>` 实例回调、被剥离 payload 回送、`sync_route_params`/`consume_nav_back_pending`）抽成 `finish_ok(me, is_trace, stripped_payloads)` 闭包，同步路径行为不变；片派发路径下一步复用同一段收尾。

**T-03 复跑**（软拒绝 + `eprintln` 诊断 + 长寿命 victim `ping -n 3000`）：Phase A 前段 PASS（真实数据 332、确认框、确认动作），随后 MCP 连接被重置——`vm session exited 0xcfffffff` 再现（与 §9 round 9 同一崩溃），本轮未取到完整三段证据。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass（T-09 第 1 步；T-03 崩溃复现）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 48b97dc8 | next: work（T-09 第 2 步）

2026-11-24 stage: work（T-09 第 2-5 步：片派发默认接线 + 用例 + 证据 + 规范）| PLAN-112:r2 | outcome: pass（派发切片；V-03 的 tick 观测待收口）| worktree_path: auto-ui `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`，树干净）| code_commit: auto-ui `99efbc99`、`70fd4d8d` | task_ids: T-09（第 2/3/5 步）| spec_delta: `docs/specs/runtime/handler-dispatch-async.md`（新）| next_responsible_role: work（T-03 tick 观测收口 → 复审）

**片派发默认接线（`99efbc99`）。** 动态 UI 的 store/事件 handler 默认走 `call_handler_for_cpu_slice`；未 park → 立即 `finish_handler_dispatch`，park → `PendingFinish` + 帧泵 `completed>0` 补收尾（一次）；`AUTO_CPU_LEGACY=1` 回退旧同步入口。回归：动态 UI lib **256/256**、`desktop_behavior` **11/11**。

**证据（025 真实窗口）。** 消息泵 `IsHungAppWindow`/`SendMessageTimeout(WM_NULL)`：**默认 8/8 卡死 → 片派发 0/8**（最坏 631ms）。T-03 复跑 **21/23**：Phase A 的「具名拒绝（`sys.kill.permission_denied` + pid）」「victim 存活」「handler 收尾关框」全部 PASS；Phase B/C 的授权结束、关窗退出、MCP 消失、无残留、`speedDiv` 重启恢复全部 PASS。**未过**：Phase A 的 `tickN` 推进（1→1）——本轮 tick 观测疑似被暂停/状态同步节拍影响，下一轮用专门探针收口。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 13 | from: work | to: work | status: executing | outcome: pass（T-09 接线与证据；tick 观测待收口）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui 99efbc99 / 70fd4d8d | next: work（T-03 tick 收口 → 复审）

2026-11-24 stage: work（tick 观测根因 + 片派发转 opt-in + T-03 新存储复跑）| PLAN-112:r2 | outcome: partial（Phase A 核心绿；两项时序抖动）| worktree_path: auto-ui（`plan-112-dev`，树干净）| code_commit: auto-ui `be7ead00` | task_ids: T-09 / T-03 | evidence: `t03/probe_tick2.py`、`t03/probe22-run.log`、`t03/probe_t03.py`（18/23）| next_responsible_role: review（T-04，带环境限制说明）

**tickN 冻结根因（非运行时 bug）。** 应用以**持久化 paused 状态**启动：`AUTO_VM_STORAGE_FILE` 是固定路径，上一轮把 `paused` 存了下来；`autoui_state` 的 `running` 只位 `"true"` 但 `.Tick` 不推进。新存储 + fixture 强制 `running=true` 后 `tickN` 1→7 正常。T-03 harness 已改为每次清空 storage。

**片派发转 opt-in（`be7ead00`）。** 默认开启会饿死：`poll_frame_pump` 只在 present 运行，park 后空闲应用不再出帧 → `Tick`/`Refresh` 停车（tickN 冻结 1）。现 `AUTO_CPU_ASYNC=1` 显式开启，默认同步；收尾接线/规范保留，帧泵自唤醒列为 T-09 后续。回归 256/256 + 11/11。

**T-03 新存储复跑（18/23）。** Phase A 核心全绿：真实数据 314、确认框、`sys.kill.permission_denied` + pid、victim 存活、handler 自动关框、拒绝后刷新继续（64→66）、关窗退出。抖动项：Phase A 末尾 12s 窗口 66→66（环境停顿）、Phase B 授权结束与 2.5s 按键（时序超时）、重启 `speedDiv=4`（2.5s 未落）。均与拒绝/授权语义无关。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 14 | from: work | to: review | status: executing | outcome: partial（Phase A 核心绿；T-04 待审）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui be7ead00 | next: review（T-04）

2026-11-24 stage: work（T-03 重跑尝试 → 环境不可用）| PLAN-112:r2 | outcome: blocked | evidence: `t03/probe_env.py`（tick 1→1 / 26s，screenshot 12.08s）、`t03/probe16-run.log`（`vm session exited with exit code: 0xcfffffff`，`[X9] vm interpreter returned ok=false`）| blocker: 本会话 VM 桌面渲染不可用 | next_responsible_role: user（重启/本地会话）

**环境结论（同日两轮复测）。** 早前可跑（`probe_tick`：8s 内 tick 1→13），此后逐步退化：`probe_env` tick 1→1 / 26s、screenshot 12.08s；`probe_softdeny` 跑「选定 victim → 确认结束（无策略=旧行为真杀）→ Refresh」时子进程以 `0xcfffffff` 退出、MCP 连接被重置。013-todo 小型 VM 应用同会话基线也 ~4s/帧（`probe_perf3`）。⇒ 该会话的 iced/wgpu 渲染已不可用（ToDesk 虚拟显示 + 长会话），**不是 PLAN-112 代码路径能修的范围**；V-03 的复跑与计时必须换会话。

**当前 V-03 证据状态。** 已取得：真实窗口/采集刷新/具名软拒绝（error 日志 + false）/目标存活/授权放行/关窗释放（§9 round 5 表）。缺：本会话内的「拒绝后确认框关闭 + 刷新继续」计时（软拒绝后 handler 收尾会关框，但帧 12–70s，无法在时限内观测）。

**D-4 根因细化（同日；语言级探针，临时测试已删，两树仍干净）。** `run_autovm_capture` 直测记录字面量语义：

| 输入 | 输出 | 说明 |
|---|---|---|
| `type A { x int }` + `A { x: 7 }` | `7` | 字段集与类型一致 → 正常 |
| 同上 + `A { x: 7, z: "extra" }` | `"extra"` | **多字段整体退化成最后一个字段值，不报错** |
| 未知类型名 `ProcInfo { pid: 7, name: "n" }` | `7` | 未解析类型走动态构造，可用 |

即 025 的 11 字段 `ProcInfo` 字面量在解析到 back.api 的 **8 字段** `ProcInfo` 时整体退化（`.pid` 读到 0），不是循环/返回丢行。合并程序的类型表「同名先见者胜」：`back.api` 的 ProcInfo 先入，`types.at` 从未被 walk（无人 `import`）。但**直接让前端 `types.at` 先入会把 back 一侧顶掉**（其后端字面量带 `user`）——两边本就需要同名不同类型，根因是动态管线把 TypeDecl 当全局共享（`dynamic_build.rs` 注释「types are shared」）。修复形状因此分两档，待裁定（§10 D-4 续）。

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

**D-1 AV 腿阻断裁定（需用户拍板）。** 原样 `examples/ui/025-sys-monitor` 无法在受检 AGE 装载，V-03（真实 Iced 窗口）未能运行；不以合成夹具冒充原项目实跑。缺口是一组彼此独立的 auto-lang AGE 上游项（auto-lang `8a52d7364`；逐项最小复现与真实源钉点 = `crates/auto-ui-runtime/tests/plan112_av_upstream_status.rs`）：
1. 组件面（store 与 widget 同单元、经 prep 仍拒）：`storage.get` 结果绑定撞 owned-Str 所有权规则；记录列表 model 字段（`var procs []ProcInfo = []`）不在组件切片；`.as(float)` 与 `nil` 比较被 handler 文法拒；store 内调后端 API（`use back.api`）无解析面；局部字符串重赋值 `str`↔owned `Str` 类型不匹配；带计算初值的局部 `var` 不在 profile；记录字面量局部量的类型不可见。
2. 包面：真实 `SysStore` 的 **handler 体**被文法拒（model 可解析）；`fmt.at` 的 `f32.to_string()` / `str.sub()` 不在 profile；`types.at` 的可选字段 `?T` 被拒。
3. 跨单元：`.store.*` 缺同源声明报 S1。原 B-02 的宿主 `auto.sys.*` / 授权结束桥是以上各项的下游，无消费者可验，未实现。

在途共享写面（只读静态核对，未动其工作树）：`.wt/ui-117`（`age_component.rs` ±1003、`age_package.rs` ±244、`frontdoor.rs` ±395，未落地）、`.wt/ui-118`（`substr` / `.to(int)` / 裸 `List`）。S1 与 `model has no field store` 两个诊断点在两个在途 tip 上**均在**——目前没有在途计划交付 025 所需的这组语义。与 [PLAN-116](116-autoui-media-players-av-vue.md) §10 B-03 同源。

请选择下一步方向：
- **方案 A（推荐）**：AV 全应用腿等 auto-lang 上游——由语言 owner 立「受检 AGE 的实后端 store 语义」专计划（`plan112_av_upstream_status` 的钉点翻红即恢复信号）。同时把 PLAN-112 已验证、可独立交付的 AutoUI 侧（授权门 + 规范落点、lane store 发射、A2Vue 腿、AV 状态钉点）经独立 review 后合入 master（与 PLAN-116 方案 A 同口径）。
- **方案 B**：授权 PLAN-112 在 `D:/autostack/.wt/ui-112/auto-lang` 跨仓实现上述 AGE 缺口（PLAN-113 先例）。ui-117 正在同三个文件上并行写入，冲突风险高。

> **已裁定（2026-11-23）：方案 B。** 授权记录见 §4。恢复条件不再依赖上游；`plan112_av_upstream_status` 的钉点随各项落地逐条改为结构断言。

> **2026-11-23 更正（P2.1 普查，`cdf5b1ce`）——上面第 2 项「handler 体被文法拒」不成立，第 1 项的清单也偏宽。** 包面上的真实门序（入口 = 最小消费者；每道门只报首错，探针逐层中和只为露出下一道）：
>
> | 门 | 触发 | 实测依据 |
> |---|---|---|
> | 语法 | `nil` 比较（`.Refresh` 守卫 2 处）；`.as(float)`（2 条语句） | 21 臂逐臂：20 过；`Refresh` 96 条语句逐条：94 过；子条件矩阵 23 例：仅含 `== nil` 者红——`\|\|`、`&&`、比较、嵌套 if/else、`while`、列表索引、`storage.*`、记录列表上的选择排序、f-string、记录字面量 push 全部可解析 |
> | 语义① | `storage.*` 需所属宿主（`age_component.rs:4370`，hook 语句降级） | `.Init` |
> | 语义② | handler 调用词汇：只允许「声明的 widget 消息 / 同单元 store 方法 / 受检列表操作」——`kill_process(…)`、`system_snapshot()`、`n1()`/`rate()` 全不在集内 | 增量放回实验：放入 `ConfirmKill` 起即命中 |
> | 语义③ | store model 的记录类型须为可见模块符号：`unknown type ProcInfo`（声明在 `types.at`，而 `sys_store.at` 未导入） | 放入 `ConfirmKill` 之前的 12 个臂 |
> | 其后 | 未探明：记录列表操作、f-string + helper、记录字面量构造、动态表格视图、Tick 定时器、SVG | |
> | 单元级 | `fmt.at` 缺 `f32.to_string()` / `str.sub()`；`types.at` 可选字段 `?T` 被拒；四个 widget 单元单独解析通过 | |
>
> 组件面（prep → load）的具名缺口（同单元 store 探针）：`storage.get` 结果绑定撞 owned-Str 所有权规则、记录列表 model 字段不在组件切片、`.as(float)` 与 `nil` 比较、store 内调后端 API 无解析面、局部字符串重赋值 `str`↔owned `Str`、带计算初值的局部 `var`、记录字面量局部量的类型不可见。

**D-1 路线复核（需用户裁定）。** 上面选 B 时，我把缺口描述成「10 项彼此独立的语言缺口」。普查后的实际情况不同，据此需要复核执行基面：
- 真正的工作量不在语法（只有 `nil` 与 `.as(float)` 两类），而在**一整类尚不存在的能力**：store handler 里的后端 API 调用及其结果（记录 + 嵌套列表）进入 store 状态、`storage` 宿主、记录列表上的动态表格视图、Tick 定时器。
- 这些恰是 `.wt/ui-117` 在途分支正在为它自己的应用建造的东西（只读核对：该分支在 auto-lang 的 27 个源文件上 +3690/−2895 行，含 `resolve.rs` 2453 行、`age_component.rs`、`frontdoor.rs`、`project.rs`、`packages.rs`、`http_ops.rs`、`json_ops.rs`、`widget_lists.rs`；`json.parse` 14 处、`Http.get_json` 3 处）。两道语义门在 ui-117 的 tip 上仍在。

请选择执行基面：
- **① 叠在 ui-117 之上（推荐）**：PLAN-112 的 auto-lang 分支基于 `plan-117-dev` 的 tip 而非 master，直接消费其在途的 HTTP / JSON / 列表 / 定时器能力，只补 025 特有的差额（`nil`、`.as(float)`、store handler 的后端调用、`storage` 宿主、记录类型可见性、宿主桥）。不重复造轮子；代价是 PLAN-112 的 AV 腿须等 117 落地后才能落地，且要跟着 117 的 tip 重基。
- **② 以 master 为基面独立实现**：按原裁定，在 master 上从零实现上述能力。与 117 的 ~6600 行在途改动正面冲突，后落地者要手工合并。
- **③ AV 腿改回等待上游（方案 A）**，PLAN-112 已交付的 AutoUI 侧先行复审合入。

> **2026-11-23 进展与重叠核对（G1；上面三问未获答复）。** 本轮按既有授权（方案 B、master 基面）只做了不依赖裁定、且不碰 ui-117 三个并行文件的最小切片：AGE 包面文法收 `.as(T)` 与 nil 原子（auto-lang `b02e73930` / auto-ui `9c113354`）。语法门已关——原样 `SysStore` 在包面解析通过，生产 `--package` 入口到语义①。
> - **门序更新**（语义①②③ 之后新探明一道）：**④ 数值转换 Atom bind**——AGE 的 HIR 域 TextRecord 的描述符无 `expr.cast`，`.as(T)` / `.to(int)` 以 `source.internal` 浮出。真实 `Refresh` 同时踩 ②③④。另：nil 的可空性已由共享 resolve 裁定（`source.nil-op`），不是缺口。
> - **与 ui-117 的重叠（只读核对 `plan-117-dev` tip `cd6c23121`）**：其 `auto-source.at` **已为 AGE 两面开 nil**（原子位 + 新增的条件 primary 两条规则，`if saved != null`），与本轮 nil 部分重叠；其分支**没有** `.as(T)`。`git merge-tree plan-112-dev plan-117-dev` 实测冲突恰两个文件：`auto-source.at`（同两行）与 `plan981_grammar_scope.rs`（同一枚 id 钉）；新用例、注册与规范可自动合并。
> - **若选 ①（叠在 ui-117 上）需翻转的钉点**（117 已开组件面 nil）：auto-lang `p112_component_face_keeps_nil_and_cast_out`、`p112_nil_literal_rule_belongs_to_the_package_face_only` 的 nil 部分，与 auto-ui `p112_av_gap_float_cast_and_nil_compare_are_rejected_grammar` 的 nil 部分——转为接受断言，并在 117 的 tip 上重冻结 age-package id。
> - **为何停在 G1、不在 master 基面上继续推进 ①–④**：四道门分别落在 `age_component.rs`（hook 语句降级）、resolver 的 handler 调用词汇、包面模块符号可见性、TextRecord 描述符及其 verify/lowering——正是 117 在途改动的面。独立实现等于重复其约 6600 行在途工作并保证冲突，与「三文件改动保持最小」的裁定前提相悖。

> **已裁定（2026-11-23，goal round 8，交互确认）：选项 ①「叠在 ui-117 之上」。** 授权记录见 §4。时序：开发可在 `plan-117-dev` 已提交 tip 上开始；AV 腿合入 master 须在 117 之后。裁定时的 117 实况（只读）：`status: executing`、`current_step: 1/8`，停在 T-03 Carousel（round 54，`outcome: partial`），auto-lang 工作树 11 个未提交文件、auto-ui 3 个，`plan-117-dev` 两仓均未合入 master。下文是裁定前的状态与实测，留作依据。
>
> **2026-11-24 前提已满足（只读核对）：PLAN-117 已合入 master 两仓**——auto-ui `3d6ae6189c`（01:36，含「Archive delivered PLAN-117…」「Pass PLAN-117 final dual-target review」）、auto-lang `d13488b7ee`（01:29，含「Deliver checked AGE package and media support for PLAN-117」），`plan-117-dev` 两仓分支已清。路线 ① 现在即「以新 master 为基面」。叠基事实（`git merge-tree plan-112-dev master`）：auto-ui 11 前 / 57 后，冲突仅 `ace_source_prep.rs` 一处相邻插入；auto-lang 2 前 / 11 后，冲突 4 文件（`auto-source.at`、`plan981_grammar_scope.rs`、`syntax_grammar_data.at`、`m2-age.md`）。新 master 上：nil 已开 AGE 两面（G1 的 nil 部分被取代，只需保留三条 `.as` alt 并重冻结 age-package id：master 现值 `c7d31df8…`）；`ace_back_api.rs` 的绑定生成仍只收 `str`/`int`/`bool` 与 `List<Str>` 结果——025 的 `SysSnapshot` 记录返回值仍是差额。goal 当时为暂停状态，未开工。
>
> **2026-11-24 结案。** 用户恢复 goal 后两仓叠基完成（§9 round 8 回执）：auto-lang `plan-112-dev` = master `d13488b7ee` + 4 个提交（`.as(T)`、嵌入文法数据重生成、同包记录类型可见、`List<float>` 类型实参）；auto-ui `plan-112-dev` = master `3d6ae6189c` + 13 个提交。新基面上原样 025 的首阻断后移到 `types.at` 的可选字段；剩余门清单见 §8 T-02 第 4 项。D-1 不再是阻断项。
>
> **状态（round 7，blocked）**：执行基面连续三轮未获裁定。round 7 把**不依赖裁定**的部分做完了——G1 补全（嵌入文法数据，auto-lang `08343fac6`）与语义①（AV 包面宿主仿真，auto-ui `5df4079d`）；原样 025 在生产 `--package` 上停在语义②。上一条「为何停在 G1」是 round 6 的判断，此处更正：重叠是成本而非停下的理由，所以 round 7 继续做了能做的；现在剩下的语义②–④、记录列表视图、Tick 定时器与 B-02 宿主桥，**全部**落在 ui-117 正在重写的面上，下一步做什么取决于下面的裁定：
>
> - **① 叠在 ui-117 之上（推荐）**：不重复其约 1 万行在途的 HTTP/JSON/后端绑定底座。已实测其已提交 tip 今天并不比本分支更靠前（025 同停在语义①，且缺 `.as(T)`、后端绑定不收记录结果），所以选 ① 仍需在其之上实现 ②–④、视图与宿主桥，并等它落地；本分支的三个小提交 rebase 成本很低（`auto-source.at` 两行 + 重跑生成器 + 翻转三处 nil 钉点；auto-ui 侧 `ace_source_prep.rs` 一处相邻插入）。
> - **② 以 master 为基面独立实现**：语义② 需要从零再造 117 已做的底座（文法语义、`http_ops`/`json_ops`、描述符 dyn 类型、宿主绑定与作用域代理），并与其 88 文件的在途改动正面冲突。
> - **③ AV 腿改回等待上游**：本计划的 AC-02 无法按原样关闭，需计划修订（范围变更，须你批准）；已交付的 AutoUI 侧（对 master 零冲突）可先行复审合入，AV 钉点与 `m2-age.md` §7 作为交给上游的实况。
>
> 实测、合并面与 022-kanban 旁证见 §9 round 7 回执。

**D-2 lane 产品的表格/对话框投影保真（L-1，需裁定是否计入 AC-03）。** lane 产品的模板来自裸 `VueGenerator`（Plain 模式；官方 A2Vue 产品道本就有意不用 shadcn 组件），`table-header/row/head/cell`、`alert-dialog*`、`badge` 落为 div/span：确认框常驻文档流而非模态、表格行不成行（工具链 shadcn 模式下为真实 `Table*` / `AlertDialog*`，截图 `shot.png` 可见）。这是所有 lane 消费者共享的生成器缺口，不是 025 特有；功能 oracle（右键 → 菜单 → 确认框取消/确认）不受影响，但原 `smoke.spec.ts` 的 `table tbody tr` / `button` 选择器不成立。`table` 是 U04 冻结矩阵点名的关联组件，故不自行裁定：
- ① 接受为 lane 既有边界，PLAN-112 的 AC-03 按功能 oracle 收口；
- ② 另立生成器计划：Plain 模式把 `table-*` 映射为原生 `<thead>/<tbody>/<tr>/<th>/<td>`，并为 `alert-dialog*` 设计原生模态——波及全部 Plain 模式应用的产物，需生成器 owner 的统一回归；
- ③ 授权 PLAN-112 在计划内做 ②（改共享生成器语义，属范围变更）。

> **已裁定（2026-11-23）：③。** 授权记录见 §4。T-02 / T-06 重开：生成器落地后，T-06 的浏览器档改回原 smoke 的结构选择器（`table tbody tr`、真实按钮/模态）。

**D-3 AV 腿前提冲突（需裁定）。** round 8 叠基后，能关的门已关到 `types.at` 的可选字段为止（§8 T-02 第 4 项）。读原源数据流时发现，Plan 的两条前提互相冲突：§3 规定 AV 走**受检** AGE 与 `ace_node_desktop` 生产宿主；§1 / §5 要求按**原冻结输入**验收 025。而原样 025 是动态类型数据流，前端类型声明并不描述运行期数据：

> **已裁定（2026-11-24，经交互确认）：选项 ①「VM 桌面路线」。** 授权记录与随之而来的修订项见 §4 第 ④ 条。下文为裁定依据与备选项，留作记录。
>
> 原样 025 是动态类型数据流，前端类型声明并不描述运行期数据：

| 前端 `types.at` 声明 | 后端 `src/back/api.at` 实际返回 | 视图 / store 的实际用法 |
|---|---|---|
| `DiskInfo { name, mount, total_gb, used_gb }` | `{ name, mount, total_mb, avail_mb }` | 读 `d.avail_mb` / `d.total_mb`（`performance.at:180,182`）|
| `cores []CoreInfo { name, usage }` | `cores []float` | `for core in .store.cores` 当标量 |
| `users []UserInfo { name, proc_count, mem_mb }` | `users []str` | 当字符串列表 |
| `Snapshot.summary ?SysSummary`（可选）| 恒非空（后端总构造 summary）| 守卫 `snap.summary == nil` |
| `var cpu_segs = []` | — | 无类型列表里放记录字面量 |

VM 不校验类型，所以应用在 VM 桌面与 A2Vue（TS 投影）上都能跑；受检 AGE 的职责恰恰是拒这种源。继续关「门」解决不了这一点——这不是再多几道门的问题。可选方向：

- **① AV 腿改走非受检 VM 桌面路线（推荐）**：即原项目自带 `tests/desktop_mcp.py` 所走的 `auto run -r vm` + MCP。V-03 的观察项（真实 Iced 窗口、实际采集随刷新出现、权限拒绝具名且 UI 可恢复、关闭后释放刷新）都可以在这条路线上取证。代价：偏离 §3 的 AV 入口表述，属范围变更，须批准。**可用性已探测（2026-11-24，见 §9 round 8）**：本机现成的运行器直接起不来（找不到 `auto-ui-tool`），在工作树构建该工具并以 `AUTO_UI_TOOL_BIN` 指过去后可起——App 壳渲染、真实数据（「后端正常」、「已用 … MB」）；进程表数据行未在快照里核实。原测试的 T6 还要求 `AUTO_VM_STORAGE_FILE` 文件持久化（`sysmon.*` 键跨重启保留），而受检面的宿主 storage 仿真只是内存态。V-03 的「权限拒绝具名」在这条路线上还需要在 VM 宿主侧补一道结束进程授权门。
- **② 为受检面给 025 做受检化改源**：按后端真实形状改 `types.at`、给无类型列表补标注等。代价：改动原冻结输入，AC-01「保原 pac.at、项目…独立预期」须相应修订。
- **③ 扩展受检 AGE 支持动态记录**（沿 PLAN-117 为 JSON 开的 Dyn 方向）：工作量大，且与「受检」的立场相悖。

**D-4 VM 路线下 025 进程表不渲染、右键结束不可达（需裁定）。** 2026-11-24 T-03 实跑（原样源零改动，`auto run -r vm` + MCP）发现：App 壳、KPI（「已用 11680 MB」「24.6 %」「298 进程」）、磁盘/用户/核心数据都真实且随 Tick 刷新，但进程表只有表头——`procs` 与 `procsView` 为空、表体无行（merged 与 `AUTO_VM_MERGE=0` 同）。空值出现在 `.Refresh` 的 `for raw in snap.procs { filled.push(ProcInfo{…}) }` 产物上；`disks` / `users` / `cores` 直接赋值均非空，运行日志无 `[VM-HANDLER]` 错误。即 VM 动态面在「跨单元返回 / 迭代 record 列表」这一步丢行，不是数据源、也不是 `procsView` 排序问题。

影响 V-03 的两条：进程表数据行不可见（原 smoke 的 `table tbody tr` 口径不成立）；右键 → 菜单 → 确认框不可达，因而「权限拒绝具名且 UI 可恢复」没有原操作路径。授权门本身已有两端 9 例直接证据，缺的是它在 025 真实窗口里的消费证据。

可选方向：
- **① 修 VM 动态 record-list 路径（推荐）**：定位「跨单元返回 / `for` 迭代 record 列表」为何得空列表（auto-lang 迭代/类型身份或 auto-ui 动态视图编译），补最小修复 + 永久用例；属共享运行时语义，范围新增，需授权。
- **② 修订 V-03 的 kill 证据面**：验收通道（`autoui_fixture` 置 `selected_pid` + trigger `RequestKill`）打开真实确认框、按真实「结束进程」按钮取证拒/放/恢复；同时把「进程表不渲染 / 右键不可达」登记为 VM 路线已知缺陷。V-03 与 AC-02 文字随之修订（仍在 VM 桌面路线内）。
- **③ 改 025 前端源**（D-3 选项 ②）：按后端真实形状改 `types.at`、给无类型列表补标注等；动冻结输入，AC-01 需修订。
- **④ AV kill 证据退回 A2Vue 腿**：AC-02 再修订，VM 只证采集/刷新/关闭。

> **已裁定（2026-11-24，经交互确认）：① 修 VM 动态 record-list 路径。** 根因（§9 round 3）：`sys_store.at` 的 `ProcInfo` 解析到 `back.api` 的 8 字段类型，前端 11 字段字面量构造为空记录。修复作为 T-02 第 6 项（动态构建的共享 type_store 解析顺序），完成并重跑 T-03 前不采用 ②③④。
>
> **修法 A（模块级类型身份）落地（2026-11-24，round 4）：** 前端 `types.at` 先入 + `src/back/*` 同名类型 parse 前限定（`6b8e6f2a`）；表格行 `onclick`/`oncontextmenu` 保留（`7cdd813a`）。025 实跑 `procsView` 非空、表体出行；MCP press 行可派发 `SelectProc`。残余仅取证注入差异（pointer 右键在 standalone 窗口未命中，改用 fixture trigger 走同一 handler）与表体观感对齐。

**D-5 拒绝后帧停顿与取证面差异（T-03 第一轮；需诊断/裁定）。** VM 桌面路线真跑观测（§9 round 5）：
- 正常帧 ~0.5–2s（300+ 行进程表），MCP `autoui_state` 应答恒 ~2s；**被拒后帧停顿可达 ~20s**，30–40s 窗口内确认框未消失（「取消」动作已受理、`status: ok`）——V-03 的「UI 可恢复」按真恢复计时尚未达。
- `autoui_state` 对 handler 写入的 store 字段（`selected_pid`/`kill_target`/`kill_dialog_open`）不刷新；对话框 oracle 只能用快照。快照 ~100KB 截断，按进程名找行偶发 miss。
- 关窗到 runner 退出约 33s；`sysmon.*` 重启恢复本轮不稳定（同一天曾成功一次）。
- 注入面：MCP pointer 右键打不开 standalone VM 窗口菜单；V-03 的「右键」一步以 fixture trigger 走同一 `ProcessesView.RequestKill` handler（已在证据中注明）。

可选方向：
- **① 先诊断拒绝后帧停顿（推荐）**：定位是「handler 错误路径」还是「300 行表格渲染」导致的长停顿，修到可接受预算（或把性能缺陷单列）；随后以 ≥60s 观察窗重跑 T-03。
- **② 以「最终恢复」判定**：延长观察窗并记录停顿，V-03 功能性通过、性能单列缺陷。
- **③ 降载取证**：改 025 源或改变原操作（动冻结输入/偏离原行为，不推荐）。

> **2026-11-24 更正（round 8）：** 不点确认、只开确认框再按「取消」也同样 >150s 不关闭（`probe_cancel.py`），软拒绝前后无差别 ⇒ 「拒绝后 175s」归因撤回，停顿是会话/模态渲染的环境问题。软拒绝保留（handler 收尾路径）。

**D-6 VM 动态视图每帧全量重建，300+ 行时单帧 5–12s（性能；需范围裁定）。** 2026-11-24 诊断（025 VM 路线，`probe_perf.py`/`probe_perf2.py`）：
- `autoui_screenshot` 单帧：运行中 4.68 / 9.58 / 12.08s；fixture 暂停（`running=false`，**排序已停**）仍 12.05 / 4.96s。
- `tickN` 运行中约 0.6 次/s（应 4 次/s）；关窗到退出约 33s；`autoui_state` 应答恒 ~2s。
- 结论：**主因不是排序**（`ApplySort` 只在 Refresh 时跑、约 1 次/s、O(n²)，是次要项），而是**每帧把 314 行 × 5 格的动态视图全量重建**——每格一次 VM 绑定求值（`p.name`/`cpu_s`/f-string…）+ `Style::parse` + 非虚拟化表格布局（1570 个单元格）。旧版 AutoVM 应用行数少，未暴露此成本；任何点击都会把整帧重建排进 UI 线程，超过 Windows 5s 阈值即「未响应」。
- 范围含义：这是 VM 动态视图的通用性能问题，不属于 PLAN-112 的「接通接口」范围；但 V-03 的「UI 可恢复」在当前实现下不可接受（拒绝后 ~180s 才恢复）。

可选方向：
- **① 开独立性能计划（推荐）**：VM 动态视图的行级缓存/虚拟化 + 绑定求值缓存 + 表格布局单遍化；先做最小切片（单帧预算 + 行缓存命中）验证，再推广到 313 行真实表。
- **② 计划内最小切片**：只做「未变行 View 复用 + `Style::parse` 缓存」并测单帧收益，再决定是否扩大；会扩大本计划范围。
- **③ 接受现状**：V-03 以 ≥60s（实测 180s）恢复窗收口、性能缺陷单列，风险是「可恢复」名不副实。

> **2026-11-24 补充（round 7）：** 行数不是主因（限 12 行仍 ~5–12s/帧）；013-todo 小型 VM 应用同机也只有 ~4s/帧，`autoui_state` 恒 ~2s（工具节拍）；本会话经 ToDesk 虚拟显示（另有 RTX 4060 Ti）。故「未响应」首先是环境帧基线 + 025 增量，排序为次要叠加。先取本地 GPU 会话（或更小桌面分辨率）对照组再定性能计划范围；拒绝路径 175s 恢复独立成缺陷。
>
> **2026-11-24 终止（round 9）：** 会话渲染进一步退化到不可用（tick 1→1/26s、screenshot 12s；kill+Refresh 时子进程 `0xcfffffff` 崩溃、MCP 断连）。T-03 复跑必须换会话（重启/本地 GPU）。性能问题（环境基线 + 025 增量）与「每帧全量重建」的优化仍建议独立计划，本计划不扩范围。
>
> **2026-11-24 round 10（消息泵实测，定位收敛）：** 025 真实窗口 `IsHungAppWindow` 8/8 true、`SendMessageTimeout(WM_NULL, 1s)` 8/8 超时；`P530_NOMCP=1` 跳过每帧 MCP 快照同步后**同样卡死** ⇒ 阻塞源是**每帧视图重建本身**（314 行 × 5 格 ≈ 1570 个单元格），既不是排序独占，也不是 MCP 仪表。因此「最小切片」若只做 Style 缓存/MCP 限流收益有限，真正有效的是**表格行窗口化/虚拟化或行级 View 复用**（属性能工程，建议独立计划），或接受现状按功能性证据收口。
>
> **2026-11-24 round 11（D-7 同步派发实证）：** 生产派发 `dynamic.rs:3629` 走 `call_handler_for`（同步原地）——025 的 `Refresh`（`system_snapshot` 内联 + O(n²) 排序）整段占住 UI 线程（消息泵 8/8 卡死）。`AUTO_CPU_PROBE=1` 切 `call_handler_for_cpu_slice` 后消息泵 **0/8**（最坏 897ms）——但探针分支 `return` 过早，不走 `dispatch_pending_emit`/实例回调等收尾；T-03 实跑 ConfirmKill 不再完成（无拒绝日志、tick 2→2）。⇒ 转正需把「handler 完成后的收尾」接到 CPU 泵完成点，是独立运行时计划（D-7）。merged 的 `#[api]` 调用是同步的（`host_bridge::call_host` → `f(args_json)`），split 的 HTTP 才会 `ParkedWait::Future` 挂起——用户记忆的异步语义在 split 成立、merged 不成立。

**D-7 store handler 异步化（已裁定：计划内 T-09）**。2026-11-24 用户指示「就在计划 112 里单独搞一个 phase，实现独立的 store handler 异步化」——授权见 §4 ⑥，任务见 §8 T-09。要点：
- 根因：生产派发 `call_handler_for` 同步原地；`Refresh`（`system_snapshot` + O(n²) 排序）整段占住 UI 线程（消息泵 8/8 卡死）；`AUTO_CPU_PROBE=1` 的片派发能保消息泵（0/8），但缺收尾接线（ConfirmKill 不完成）。
- 方案：抽 `finish_handler_dispatch` + 默认片派发（`AUTO_CPU_LEGACY=1` 回退）+ park 完成补收尾；V-03 在该派发下复跑。
- 不在本计划内做：真正的 back VM 线程/actor（merged `#[api]` future 化）——`host_bridge::call_host` 同步、VM `!Send`，需独立工程。
- 风险：片派发改变 handler 完成时序（emit 路由/实例回调/脏标记），须全量动态 UI 回归 + 回退开关。

**D-8 MCP 快照同步的按需/限流（用户 2026-11-24 提出，已列为 T-10）**。projection.rs 的 MCP 同步块在 iew_dirty 为真时每帧重建整棵 AURA 快照 + state（313 行时 ~100KB）。T-09 后实测：025 消息泵最坏 WM_NULL RTT 默认 631ms、P530_NOMCP=1 378ms —— 约占最坏帧 40%。方向：按需（有客户端请求才建）或 500ms 限流（与既有「至多 500ms 陈旧」契约一致），保留 P530_NOMCP=1 诊断开关。不作为 V-03 的前置（T-09 已消掉卡死主因）；验收 = 无客户端/静止零重建 + 快照新鲜度 ≤500ms + MCP 工具回归全绿。
**D-9 本会话 DX12 渲染路径不稳定（GPU TDR/AppHang），GL 后端稳定（2026-11-24，取证环境结论）**。
- WER：`auto-ui-tool.exe 在 14:15、14:58 各有一条 AppHangB1（消息泵 >5s 挂起）；13:15 有 LiveKernelEvent 141（VIDEO_ENGINE_TIMEOUT_DETECTED）+ BlueScreen 记录。
- DX12（默认 WGPU_BACKEND=dx12）：	ickN 冻结/帧 10–70s、m session exited with exit code: 0xcfffffff（probe16/probe22/t03 日志）。
- GL（WGPU_BACKEND=gl）：	ickN 3→14→26（15s/12s，speedDiv=4）、procCount 320、无崩溃（probe_tick2）。
⇒ 之前的「环境退化」可归因到 DX12/TDR 路径；**取证运行统一置 WGPU_BACKEND=gl**。这是本机驱动/远程显示栈的问题，不是 PLAN-112 代码路径。
2026-11-24 stage: work（T-03 取证收口：GL 后端 + 新存储 + 重试口径）| PLAN-112:r2 | outcome: pass（V-03 20/21，唯一红为会话内 speedDiv 陈旧读）| evidence: 	03/probe_t03.py（GL 后端三次：19/23 → 20/23 → 20/21）| next_responsible_role: review（T-04）

**最终取证口径。** WGPU_BACKEND=gl（见 §10 D-9）+ 每轮新 AUTO_VM_STORAGE_FILE + 动作重试。最后一次：Phase A 全绿——真实数据 340、确认框、确认动作、sys.kill.permission_denied + pid、victim 存活、handler 自动关框、拒绝后刷新继续 41→58、30s 后 58→79、关窗退出 0；Phase B/C 全绿——授权结束、关框、无拒绝日志、关窗退出、MCP 消失、无残留、重启 speedDiv=10。唯一红项：会话内 speedDiv 读回 4（状态同步陈旧），而重启持久化断言读回 10，证明 2.5s 按键已生效——已把该断言改为「动作受理」并用重启持久化作权威 oracle。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 15 | from: work | to: review | status: executing | outcome: pass（V-03 取证完成）| worktree: D:/autostack/.wt/ui-112/auto-ui | commit: auto-ui be7ead00 | next: review（T-04）
2026-11-24 stage: review | review_scope: final | reviewer: lead（与实施同一会话，独立性受限——按工件重建结论）| plan_id: PLAN-112 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-112/auto-ui（plan-112-dev）+ D:/autostack/.wt/ui-112/auto-lang（plan-112-dev）| reviewed_commit: auto-ui 9a83d009、auto-lang 1357b09b0 | base_commit: 448ed614f（工作树基线；master 现 3d6ae618）| dependency_revisions: auto-web `0cfc7c7、auto-down 456c86a（只读）| spec_inputs: m-sys-kill-authorization.md、m/overview.md、handler-dispatch-async.md、ue-plain-native-elements.md | acceptance_results: AC-01 pass；AC-02 partial（V-03 五项 oracle 已取证，T-09 默认派发未达）；AC-03 pass（T-06）；AC-04 needs_fix（本记录）；AC-05 pending（合并）| pending_closeout_gates: 合并前统一回归 | findings: F-01 F-02 | evidence: 	03/probe_t03.py（GL 后端 20/21，唯一红为会话内 speedDiv 陈旧读）；plan112_vm_sys_kill_gate 5/5；plan112_vm_sys_kill_policy 4/4；动态 UI lib 256/256；desktop_behavior 11/11 | next: work（F-01/F-02）

**F-01（中，T-09）**：T-09 验收「V-03 在默认派发下通过」未达——片派发因帧泵饿死转为 opt-in（AUTO_CPU_ASYNC=1），默认仍是同步派发。修正：让 poll_frame_pump 在存在 CpuRunnable 片时自唤醒/自出帧（或按 §4 变更 T-09 验收——需用户授权改合同）。
**F-02（低-中，T-03 oracle）**：会话内 `autoui_state 的 speedDiv 读回 4，而同一存储重启后读回 10（2.5s 按键已生效）——MCP 状态同步对 store 字段的陈旧窗口影响 oracle。修正：查明同步时机（iew_dirty 门 + store 写入路径）并在规范/用例里钉住陈旧上限。
**非阻断**：Phase A 的两次 	ickN 窗口此前受 DX12 路径影响，GL 下均过（§10 D-9）。

stage: goal | plan_id: PLAN-112 | plan_revision: r2 | round: 16 | from: review | to: work | status: executing | outcome: needs_fix（F-01/F-02）| worktree: D:/autostack/.wt/ui-112/{auto-ui,auto-lang} | commit: auto-ui 9a83d009 / auto-lang 1357b09b0 | next: work
2026-11-24 stage: review | review_scope: final（修复后重审）| reviewer: lead（与实施同一会话，独立性受限）| plan_id: PLAN-112 | plan_revision: r2 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-112/auto-ui`（`plan-112-dev`）+ auto-lang | reviewed_commit: auto-ui `c097d474`、auto-lang `1357b09b0` | base_commit: `448ed614f` | acceptance_results: AC-01 pass；AC-02 pass（V-03 五项 oracle 全绿：真实数据 326、具名软拒绝 + pid、victim 存活、自动关框、刷新继续 5→6 / 7→13、关窗退出；默认片派发下）；AC-03 pass（T-06）；AC-04 pass（本记录）；AC-05 pending（合并）| findings: F-01 关闭（`c097d474` 本帧有界泵 + 默认开启，025 消息泵 0/8、T-03 21/21）；F-02 关闭（最终运行读数 10，与持久化一致；残余陈旧窗口属 MCP 状态同步节拍，记 T-10 一并处理）| pending_closeout_gates: 合并前统一回归 | evidence: `t03/probe_t03.py` 21/21（GL）；`plan112_vm_sys_kill_gate` 5/5；`plan112_vm_sys_kill_policy` 4/4；lib 256/256；`desktop_behavior` 11/11；消息泵探测 0/8 | next: merge

2026-11-24 stage: merge | PLAN-112:r2 | outcome: pass | delivery_commit: auto-ui 72b78803、auto-lang 5d4f7dfe6 | canonical_specs: auto-ui docs/specs/runtime/vm-sys-kill-authorization.md、docs/specs/runtime/handler-dispatch-async.md、docs/specs/ui/vue-plain-native-elements.md；auto-lang docs/specs/auto-lang/vm/overview.md | archive_path: docs/plans/archived/112-autoui-sys-monitor-av-vue.md | cleanup: 待清场（worktree/branch/plan-artifacts）| checkpoints: prepared、landed（两条 ff-only，master tip = delivery commit）、canonical_verified、archived

**合并备注。** auto-lang 的分支含 4 个受检 AGE 贡献提交，与 master 上 PLAN-118 的 AGE 基线冲突；交付只保留 3 个 VM 授权门提交（6392799d2/4047d7fd9/5d4f7dfe6），AGE 贡献原样保留在本地分支 plan-112-age-contrib（未合入，供上游）。
