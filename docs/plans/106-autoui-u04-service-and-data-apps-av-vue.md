---
plan_id: PLAN-106
status: executing
feature_name: U04 Weather 双端交付
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-21
plan_revision: 2
current_step: 3
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-106 U04 Weather 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。本号继续承担本页范围，原组其余 demo 迁至 [110](110-autoui-chat-av-vue.md)、[111](111-autoui-realworld-av-vue.md)、[112](112-autoui-sys-monitor-av-vue.md)、[113](113-autoui-database-av-vue.md)、[114](114-autoui-file-manager-av-vue.md)。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。本计划的 Widgets 交接取实际使用组件；独立 Widgets 页面由交付表对应计划负责。

| 原项目 | 原验收重点 |
|---|---|
| [014-weather](../../examples/ui/014-weather/pac.at) | 固定真实服务输入、加载/失败/重试及可见数据 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。对照现有 weather_data.at 的确定性样本及原 106 的服务要求。原 UI 场景保留静态样本；给所需加载、失败、重试接可复跑的本地真实 HTTP 服务，明确宿主授权及数据适配。

## 3. 技术与实现入口

目标入口为 use weather_data 与原页面载入、主题和布局。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-106/{auto-ui,auto-lang}，每仓单一 plan-106-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

可独立进入 T-01。先用 014 原本地样本验证城市切换、小时/逐日预报和刷新，AV 与 A2Vue 分别记录首阻断。原 SelectCity 会重写 hourly 和 daily 的文本记录列表，再由视图循环渲染。若当前主线卡在共享类型化列表或视图接线，核对 105 已交语义及 114 同一写面的交付版本。再冻结本地真实 HTTP 服务合同，单独验加载、错误和重试；原 Refresh 的时间更新只作为样本交互收据。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。同一固定天气负载在两端显示一致；受控服务首请求失败、重试成功；关闭后迟到响应不改旧或新实例。源码 src/front/weather_data.at 当前为静态样本，报告里分别记静态 UI 与真实服务证据。 |
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
  - 2026-11-21 完成：源冻结（pac.at 901B、app.at 47371B、weather_data.at 17942B、types.at 510B、README.md 1507B、tests 2 个；SHA256 前 16 位见 §9 收据）；依赖冻结（auto-ui master `ad62e154`、auto-lang master `5e5121dac`、auto-web detached `0cfc7c7`、auto-down detached `45b642d`、auto.exe=auto-man debug `plan-890`，工作树 `.wt/ui-106/{auto-ui,auto-lang,auto-web,auto-down}`，两仓 `plan-106-dev`）。
  - 首阻断实测：**AV 组件面**（`ace_node_desktop_window --source`）在 `age-if-expr` 处回滚——`source.age.structure: unexpected token in a bare element tag`（最小复现：`style: if A {…} else if B {…}` 链；单 `else` 可过、组件面 `Str==str` 条件可过）；包面（`compile_age_package_with_scenario`）同因回滚到 `source.syntax: expected fn, found widget`，且单 `else` 的 `Str==str` 条件在包面 `verify.type-mismatch`（组件面同形可过）。写面=auto-lang ACE 文法/投影/verify（无在途计划持该写面），本计划组内 auto-lang 树实施。
  - A2Vue lane（`generate_sfc_with_lane`）实测**已可生成**：59.5KB SFC 含北京/上海/24小时/5日/刷新/`v-for`/`else if`/`:class`/SelectCity，英雄渐变按嵌套三元发射；真实浏览器验收归 T-06。包面 SFC（`generate_package_sfc`）仍被 else-if 链 + `Str==str` 条件拦（若用包面需同文法修复）。
  - 预期冻结：§6 V-01..V-05 判据按本表承接；数据行 `hourly`/`daily` 为**无类型 Obj 列表 model 字段**（SelectCity 重写），AV 桥循环叶展开与 105 已交语义（LIST_FIELD_VIEW/元素位 owned Str）对账，本地真实 HTTP 服务合同在 T-02 冻结。
- [x] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
  - 2026-11-21 完成（B1/B3 已修、HTTP 合同已冻结、B2 具名记录见 §10）：**B1 else-if 链文法/投影/verify**（auto-lang `010a14d35`）、**B3 包面 Str==str**、**object-lit 开 AGE 两面**、永久回归 `plan106_entry_if_chain` 5/5、文法 id 重基线；**本地真实 HTTP 服务合同**（auto-ui `92054337`，`plan106_weather_service` 2/2）。
- [ ] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
  - **BLOCKED（B2，owner 见 §10）**：014 原样源在 AV 事件会话引擎被列表状态标量门拒（`compile_component_event_program` 5085 门）。已交替代证据：全原样 view 真窗 exit 0/frames=2/applies=81/errors=0（北京 23° 全字段文本在位）。B2 解除后补原样装载与 SelectCity/Refresh 交互。
- [ ] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
  - 2026-11-21 完成（auto-ui `b18572c7`，`plan106_014_vue_browser` 14/14，~5s，`AUTO_UI_ACE_WEB_E2E=1`）：lane 产品 → vite 冷构建 → Playwright 真浏览器。初始 marker 8 项（北京/24小时/5日/刷新/当前详情/湿度/体感/演示样本）、城市切换按原样本合同（city_zh 切上海、英雄保持 beijing 默认 23°——原 pac.at 冻结行为，vm_smoke.py 同口径只断言 city_id/city_zh）、Tab 5日 切今天/周二、主题按钮在位、零 page error。证据站点 `plan-artifacts/plan-106/sites/014-weather/`。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-21 stage: work（T-01 冻结矩阵与首阻断）| PLAN-106 r2 | outcome: pass | worktree: `D:/autostack/.wt/ui-106/{auto-ui,auto-lang,auto-web,auto-down}`（auto-ui/auto-lang @ `plan-106-dev`，auto-web detached `0cfc7c7`、auto-down detached `45b642d`）| task_ids: T-01 | evidence: `D:/autostack/plan-artifacts/plan-106/t01-matrix.md` + 探针日志（`probe1-min.at`/`probe2-single-else.at`/`plan106_probe_*`）**源冻结（7 tracked 文件，SHA256 前 16）**：pac.at `1682E3C64FDE280E`（901B）、README.md `0B3425512B19F5CB`（1507B）、src/front/app.at `88841A8BB0904CF6`（47371B）、src/front/types.at `F95E759791A8257B`（510B）、src/front/weather_data.at `4CB2C1D7497689BF`（17942B）、tests/test_014_vue.mjs `D27B7F9A8D46C38F`（5692B）、tests/vm_smoke.py `FB1528A4570A071C`（18591B）。**两端入口实测（真实运行）**：AV=`ace_node_desktop_window --source <014 app.at>`（组件面，构建 3m06s）；A2Vue=`generate_sfc_with_lane`/`generate_package_sfc`（包面）+ lane 浏览器腿归 T-06。**首阻断（具名）**：①AV 组件面/包面共因=**AGE 文法 `age-if-expr` 仅收单 `else`**，014 有 29 处 `else if` 链（hero 渐变 landscape+portrait、AQI 徽章）→ 组件面回滚 `source.age.structure: unexpected token in a bare element tag`、包面回滚 `source.syntax: expected fn, found widget`；最小复现=`style: if .condition == "sunny" {…} else if .condition == "rain" {…} else {…}`（组件面/包面均拒）。②包面单 `else` 的 `Str==str` 条件另在 verify 拒（`verify.type-mismatch: eq operands must both be i32 or both be bool (found Str / str)`）；组件面同形可过（probe2 exit 0）。③A2Vue lane 已可生成（59.5KB，全部业务 marker 在案，英雄渐变=嵌套三元 `:class`）；包面 SFC 仍被①/②拦。**写面归属**：AGE 文法/投影/verify=本计划组内 auto-lang 树（无在途计划持该写面，105/1072 已归档交付）；A2Vue 生产链=lane 产品 + 真实浏览器（104/105 先例，T-06）；`hourly`/`daily` 无类型 Obj 列表 model 字段的 AV 循环展开按 105 已交语义对账（LIST_FIELD_VIEW/元素位 owned Str），本地真实 HTTP 服务合同 T-02 冻结。**主检出观察（非本计划改动，落地前由 owner 收口）**：auto-ui 主检出 `docs/plans/`（110–123 拆分文档 + README + .next-id）与 auto-lang 主检出 `docs/design|docs/plans/`（1097/1113 等在途）为预存 WIP，非代码路径；auto-lang 主检出另有在途计划文档。`next_responsible_role: work（T-02：AGE else-if 链文法/投影/verify + 包面 Str==str 条件 + 冻结本地 HTTP 合同）`

2026-11-21 stage: work（T-02 第一批：B1/B3 已修 + HTTP 合同冻结；B2 具名记录）| PLAN-106 r2 | outcome: partial（B2 为外部写面 blocker，owner 具名）| worktree: auto-lang `010a14d35`（plan-106-dev，9 文件 +272/−22）+ auto-ui `92054337`（服务测试）| task_ids: T-02 | evidence: `plan-artifacts/plan-106/`（t01-matrix.md、探针/隔离脚本、服务测试）+ 回归组 **B1/B3 修复**：①else-if 链文法（`age-if-else` chain 臂）+ `ComponentEntryValue::If.else_chain` 投影 + frontdoor 链臂 cond/样式收集；②包面 verify 改 `verify_text_record_age`（Str==str 与组件面同旗）；③`object-lit`/子规则开 AGE 两面（handler 表达式可解析对象字面量）。永久回归 `plan106_entry_if_chain` **5/5**；文法 id 重基线（age-component `24dedc6d…`、age-package `5340118d…`，13 邻居逐字节不变）；受影响组 340/340（plan106/105/1072/981/980/104）+ av plan105 13/13 + auto-ui plan980_package_sfc 11/11。**B2 三层核实（组件面 AV 事件会话）**：语法=已开；bind=无类型列表=只读 view-data（handler 写设计拒）；`compile_component_event_program` 5085 标量门拒任何列表字段（`[]str`/`[]Todo`/`[]HourPoint` 实测均拒，场景车道 2325 门放行但 AV 桥不用）——014 原样源无法 `--source` 装载。105 已交场景车道 `[]T` 内建写 + store/宿主 API 记录写；114 在途 age-package object-lit（可变列表未接通）；组件面事件会话列表状态写面未交。**已交替代证据**：全原样 view 真窗（`av-fullview`：exit 0/frames=2/applies=81/errors=0，北京 23° 全字段文本在位）；本地真实 HTTP 服务合同（`plan106_weather_service` 2/2：fail-once→retry-200 + 未知城市回落，真实 TCP 临时端口）。`next_responsible_role: A2Vue T-06（独立可续）已交；AV T-03 待 B2 解除`

2026-11-21 stage: work（T-06 A2Vue 浏览器证据）| PLAN-106 r2 | outcome: pass | worktree: auto-ui `b18572c7`（plan-106-dev）| task_ids: T-06 | evidence: `plan106_014_vue_browser` **14/14**（真实 Chromium，4.86s，`AUTO_UI_ACE_WEB_E2E=1`）+ 站点 `plan-artifacts/plan-106/sites/014-weather/`（App.vue/a2x_rt.mjs/vite 脚手架/verify.mjs）**判据（V-04）**：①lane 产品 SFC 含 北京/上海/24小时/5日/刷新/`v-for`/`:class`/SelectCity + 条件样式三元；②vite 冷构建成功；③真浏览器初始 marker 8/8（北京/24小时/5日/刷新/当前详情/湿度/体感/演示样本）；④城市切换按**原样本合同**（SelectCity 首段按 ASCII id 更新 city_zh→上海；第二段数据分支比较中文 id 故英雄保持 beijing 默认 23°——原 pac.at 冻结行为，vm_smoke.py 同口径只断言 city_id/city_zh）；⑤Tab 5日 切 今天/周二；⑥主题按钮在位；⑦零 page error。`next_responsible_role: review（T-04 AV 仍 B2 阻；T-07 终审待 A2Vue 落主）`

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

**B2（2026-11-21，AV 腿原样装载阻断，owner=共享 typed-list 写面/114 在途）**：014 原 SelectCity/Init 在 handler 重写无类型 Obj 记录列表（`var hourly = [...]`、`var daily = [...]`）。AV 事件会话引擎（`ComponentEventSessionProgram::compile` → `compile_component_event_program`）当前按 **5085 标量门**拒绝**任何**列表 model 字段（`[]str`/`[]Todo`/`[]HourPoint` 均拒；场景车道 2325 门放行 `[]T` 但 AV 桥不用该入口）——014 原样源无法在 AV `--source` 装载。三个层次均已核实：①语法（对象/数组字面量在 AGE 表达式面）已由本计划开（B1 同批）；②bind（无类型列表=只读 view-data，handler 写=设计拒 `undeclared model field`）；③AV 事件会话标量门。105 已交 `[]T` 内建元素写（012，场景车道）+ store/宿主 API 记录写（013）；114 在途（age-package object-lit，可变列表未接通）；组件面事件会话的列表状态字段写面两仓均未交。按 §4「核对 105 已交语义及 114 同一写面」+ §5「缺具体能力时明确负责计划、版本和最小复现」，本计划记录不重复实现。**unblock**：114 或共享 typed-list owner 交付组件面事件会话的列表状态写面后，014 原样源可在 AV 装载并复验；届时本计划 T-03/T-04 续跑。**已交替代证据**：固定负载显示（全原样 view 真窗 exit 0/applies=81/北京 23° 全字段）+ 本地真实 HTTP 服务合同（fail-once→retry-200 + 未知城市回落，2 测试通过）。
