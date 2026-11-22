---
plan_id: PLAN-113
status: archived
feature_name: U04 Database 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-23
plan_revision: 2
current_step: 8
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978, PLAN-085]
supersedes_spec_components: [auto-lang:docs/specs/ace-migration/m2-age.md]
new_spec_components: [auto-ui:docs/specs/runtime/ace-sqlite-host-bridge.md]
touched_goals: [GOAL-002]
---

# PLAN-113 U04 Database 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [106](106-autoui-u04-service-and-data-apps-av-vue.md) 的 U04 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。关联 Widgets 为 [datatable](../../../auto-os/widgets-gallery/src/front/pages/datatable.at)、[table](../../../auto-os/widgets-gallery/src/front/pages/table.at)。

| 原项目 | 原验收重点 |
|---|---|
| [026-database](../../examples/ui/026-database/pac.at) | 对真实数据库读写、事务或失败响应与资源回收 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。原 RunSql 依据字符串筛选内存样本，事务处理只更新状态。冻结隔离的实际 SQLite 库及授权能力。两端按已有可用数据库宿主/服务桥接摄取实际结果，使用相同数据与原 UI。树组件按已交 085 范围复用。

## 3. 技术与实现入口

目标入口为 src/front/app.at 的 RunSql、CommitTx、RollbackTx 及现有组件。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-113/{auto-ui,auto-lang}，每仓单一 plan-113-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

2026-11-22 用户授权（work 阶段，T-01 收据同批）：**113 计划内实现 AGE 组件/包面共享写面**（行尾注释 + `List` 类型 model 字段 + 嵌套 view-data 记录列表 + 预处理注入 fn 的组件/包面接纳），按 106 B2 先例；**A2Vue 腿独立推进**（lane 已实证可生成 026）。共享写面修复按 §5 边界沉淀规范增量，目标不变、范围不变。

已有受检 SQLite 路径优先；若两端缺少公共接口，由真正的数据库 provider 生产，先保源与独立验收。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。分别读和写真实临时数据库；用独立连接核对提交与回滚的实际数据，验失败恢复、错误诊断及连接释放。样本展示和真实 SQL 结果各有独立证据。 |
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
  - 2026-11-22 完成：源冻结（9 tracked 文件，SHA256 前 16 位见 §9 T-01 收据）；依赖冻结（auto-ui master `87282fb0`、auto-lang master `292575164`、auto-web detached `0cfc7c7`、auto-down detached `456c86a`，工作树 `.wt/ui-113/{auto-ui,auto-lang,auto-web,auto-down}`，auto-ui/auto-lang @ `plan-113-dev`）；入口实测（AV=`ace_node_desktop_window --source <026 app.at>`，构建 3m39s；A2Vue=`generate_sfc_with_lane` **已可生成** 68KB SFC，bodies=0/absent=32 待核）。
  - **AV 首阻断（具名，用户 2026-11-22 授权计划内修复）**：①组件面 CST——model「标量字段+行尾注释」后紧跟 `dbNodesFull List = [...]`（嵌套 children 记录列表）→ `source.age.structure: unexpected token in a `[ … ]` view-data list literal`（完整源实测首错 @6626 customers 记录区；最小复现 wb2 scalars+注释+dbNodesFull vs wc1 去注释即过）；②`List` 类型 model 字段（dbNodesFull/dbExpanded）组件/包面均拒（`component-unsupported … field X is List`，仅 `[]T`/`[N]T`）；③ace_source_prep 注入 `fn __p105_str_contains` 使包面首错 `source.unsupported: `fn` is not part of this source profile` @869。写面归属=auto-lang AGE 组件/包面文法、CST 投影与 verify（106/114 已交付归档，无在途计划持该写面），授权按 106 B2 先例在计划内实现。
  - 预期冻结：§6 V-01..V-05 判据按本表承接；026 的 RunSql/CommitTx/RollbackTx 为内存样本语义（字符串匹配 + 状态更新），真实 SQLite 宿主/服务桥接在 T-02 冻结。
- [ ] T-02 接通本计划的具体运行接口（2026-11-23 r2 修订中：⑥已收窄为组件面专属（p114 修复，f9dc0acf3）；剩余=裸 List 惰性化挂到新 frontdoor 包件投影道，修 026 装载墙，见 §9 r2 记录）
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
  - 2026-11-22 完成（AV 面六项 + 本批四项；A2Vue 包组件 SFC 见 T-06 前置剩余）：①`type` 记录键、②分支体 view-data 赋值臂、③handler 参数注解、④`$event` 归一化、⑤CtorNamed 记录元素改写（异构记录字段过共享 resolver）、⑥裸 `List` 惰性化（记录/嵌套元素 model 时点 + 非 view 循环迭代后置判定；`List<T>` 显式泛型=状态，p114 ids/expanded 契约保持）、⑦组件车道惰性列表 handler 跳过（检查函数 + 事件程序 retain，包面契约不变）、⑧整模块投影 AGE 面 age_face 开（内联 helper fn 调用语句可过）、⑨桌面桥循环载荷按钮跳过（DeleteRow(i) 不阻塞装载）、⑩注入 helper Str 参数。**验证**：核心 conformance 897 过/5 败（5 红=text_record_source 主线既有）；grammar id 重基线（age-component `817d81cf…`/age-package `9bd00590…`）入测试；auto-lang `208357194` + auto-ui `ceb9045f` 提交。**剩余（2026-11-22 全部关闭）**：⑪A2Vue 包组件 SFC 首错（`expected fn, found widget`）经 a20d4287d/eb260694a/f430818e 修复，`plan113_026_vue_browser` 18 项浏览器断言全绿复验（§9 T-03/T-06 记录）；V-03 宿主桥接同批落地。
- [x] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
  - 2026-11-22 真实 SQLite 宿主桥接冻结合同（T-02 遗留冻结义务，本批定位）：
    - **位置/归属**：auto-ui-runtime AV 桌面桥（`NodeDesktopBridge`）新增真实 SQLite 宿主桥，opt-in——由 V-03 测试入口显式安装桥规格，默认关闭，其他 app 零行为变化；026 原源 9 文件冻结不动，内存样本语义保持为「样本展示」层，真实 SQL 结果为独立的桥 evidence 层（V-03「样本展示和真实 SQL 结果各有独立证据」）。
    - **库实现**：复用 auto-lang `a2r-std::sqlite`（rusqlite 0.30 bundled，Plan 415-B1 受检面；a2r-std 本就非 optional 依赖（PLAN-716 frame_bench 契约），不新增 dep、不引入新外部依赖版本；per-op 诊断经 additive `clear_error()`）。
    - **真实库**：测试临时目录 `northwind.db`；种子 = 三表 fixture（customers 15 行 / products 12 行 / orders 10 行）逐行复制自冻结 app.at 模型字面量（总量约 4.7KB，review 可对回 T-01 钉版 app.at `D37E703351DEDFEB`），DDL 取 `.schemaDdl` 同款形状——「相同数据」由钉版 fixture 成立，不做运行时源解析。
    - **镜像规则**：桥挂在 `dispatch_event` 入口按消息名通知（app 面惰性 handler 跳过不阻塞真实执行——样本语义与真实结果两层各自独立）。`RunSql`→对真实库执行 `.sqlInput`（真实结果集/错误诊断/耗时入桥 evidence，含失败恢复：非法 SQL 得真实诊断且桥不崩、后续事务可续）；`SaveNewRow`→按 `.activeTable`+表单标量真实 INSERT（首个 staging 写 `BEGIN`）；`DeleteRow`→跳过（inert 循环载荷按钮，沿 ceb9045f 先例载明）；`CommitTx`→`COMMIT`；`RollbackTx`→`ROLLBACK`；`ResetDb`→重种子；窗口 close→DROP 连接（文件可删除/重开=连接释放证据）。
    - **独立核对（V-03）**：独立 rusqlite 连接开同一文件——commit 后新增行可见、rollback 后行数/内容回原样；桥 evidence 与独立连接两边对拍。
    - **A2Vue 腿**：浏览器无 rusqlite，真实库核对属 AV 腿（V-03）；A2Vue 腿按 V-04 保生成/构建/真实浏览器矩阵（内存语义），同合同数据shape但不重复真实库腿。
  - 2026-11-22 partial（装载达成，保留摘要）：026 原样源 `--source` exit 0、51 applies、errors=0、15 文本叶核心 marker 在案（"7 Tables · 2 Views · 3 Indexes · 10,240 Rows"、"Showing 1 - 10 of 91 rows"、"Presets:"、"Query OK" 等）；脚本 CommitTx/RollbackTx 派发 txStatusMsg 更新实证；惰性列表 handler（RunSql 等）fail-closed 跳过（app 面）。
  - 2026-11-22 完成（V-03 全项）：**真实 SQLite 宿主桥按冻结合同落地并全绿**——`ace_sqlite_bridge`（opt-in，`--sqlite-db`/`install_sqlite_bridge` 安装，默认 None 零行为变化）+ `ace_sqlite_bridge_seed`（37 行逐行复制自冻结源，校验脚本 identical）+ `a2r_std::sqlite::clear_error`（additive，per-op 诊断）；`dispatch_event` 引擎派发前镜像消息（app 面惰性跳过不阻塞真实层，两层证据独立；app 面 RunSql/SaveNewRow 具名拒 `no on … handler is declared` 为既有惰性语义，非回归）。**验证**：`plan113_026_sqlite_bridge` 1 例 4.91s——preset customers 真实查询 15 行；error preset 真实诊断 `no such table: non_existing_table`（失败恢复：桥不崩、后续事务可续）；SaveNewRow 真实 INSERT（BEGIN）→CommitTx COMMIT；第二个 SaveNewRow→RollbackTx ROLLBACK；独立 rusqlite 连接核对 customers=16（15 种子+1 已提交）、已提交行内容精确匹配、回滚行不存在、products/orders 种子未被污染；进程退出后临时库文件可删=连接释放。**输入目标探测实证**：modal 表单输入=目标 14..17（12/13=treeFilter/searchKw 具名拒）。
- [x] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
  - 2026-11-22 完成：与 T-07 合并为一次 final review（无分角色工作流），结论 pass（§9 review 记录）。
- [x] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
  - 2026-11-22 完成：`plan113_026_vue_browser` 1 例全绿（18 项浏览器断言 OK，RESULT failures=0）——lane 产品（原源含 use 导入池 + PaginationControl/TreeView/TreeIcon 包组件 SFC 同道生成）→ vite build → 真 Chromium 挂载：初始 marker（Database Studio/northwind.db/Tables/Views/Indexes/Data/Schema/SQL/customers）、Schema Tab 列定义+DDL、SQL Tab 成功路径、错误路径反馈、Add Row→Save→Commit 在位+事务反馈、零 page error。证据 `plan-artifacts/plan-113/sites/026-database/`（App.vue 69KB、a2x_rt.mjs、dist/、shot.png、verify.mjs）。**成本线修复**：首跑 11.87s 超 10s 新例线，10 处固定 wait 收敛（900→400 等）后 7.68s 全绿。⑪包组件 SFC 首错（`expected fn, found widget`）经 auto-lang `a20d4287d`（包面 widget lambda/合成消息变体/int list 默认值）+ `eb260694a`（legacy payload 回调按参数拼写接纳）+ auto-ui `f430818e`（lane 产品解析 use 导入模块 fn）修复并本批复验。
- [x] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
  - 2026-11-22 完成：与 T-04 合并为一次 final review，结论 pass（§9 review 记录；两端差额=样本语义层 vs 真实 SQL 桥层各自独立证据，共享 AGE 写面变更的受影响已交结果经 ace 组回归+2 例 p114 基线红具名分流核验）。
- [x] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 stage: work（T-01 冻结矩阵与首阻断）| PLAN-113 r1 | outcome: pass | worktree: `D:/autostack/.wt/ui-113/{auto-ui,auto-lang,auto-web,auto-down}`（auto-ui/auto-lang @ `plan-113-dev`，auto-web detached `0cfc7c7`、auto-down detached `456c86a`）| task_ids: T-01 | evidence: `D:/autostack/plan-artifacts/plan-113/`（探针 probes/、prepared 源 dump、bisect 脚本）**源冻结（9 tracked 文件，SHA256 前 16）**：pac.at `30C28A78A8677EF7`（1359B）、README.md `57ECEEDC71C5175B`（2146B）、SPEC.md `C1FD450E9F7611E2`（7193B）、src/front/app.at `D37E703351DEDFEB`（61073B）、src/front/components/filetree.at `BA5DE834CED6D3FC`（3167B）、package.at `B6FF9789C1EDA5AF`（136B）、tree_icon.at `28F42744A474EB99`（4163B）、tree_util.at `4F4BA6028CA4BC1A`（7701B）、tests/desktop_mcp.py `4D81E885187D1075`（13026B）。**两端入口实测**：AV=`ace_node_desktop_window --source <026 app.at>`（组件面，构建 3m39s；完整源首错 @6626）；A2Vue=`generate_sfc_with_lane` 生成 68KB SFC 成功（bodies=0/absent=32，真浏览器归 T-06）。**AV 首阻断（具名）**：①组件面 CST——「标量字段+行尾注释」+「dbNodesFull List 嵌套 children 记录列表」组合拒（`source.age.structure: unexpected token in a `[ … ]` view-data list literal`；最小复现 wb2=scalars+注释+dbNodesFull 拒、wc1=去注释过、z1/cu3=dbNodesFull 单独过）；②`List` 类型 model 字段组件/包面均拒（`component-unsupported … field X is List`，仅 `[]T`/`[N]T`）；③ace_source_prep 注入 `fn __p105_str_contains` 使包面首错 `source.unsupported: `fn` is not part of this source profile` @869（组件面受注入影响位置偏移）。**写面归属**：AGE 组件/包面文法、CST 投影与 verify=本计划组内 auto-lang 树（106/114 已交付归档，无在途计划持该写面）；**用户 2026-11-22 授权**：计划内实现该写面（106 B2 先例）+ A2Vue 腿独立推进。**主检出观察**：auto-ui 主检出 `docs/plans/`（110–122 拆分文档 + README + .next-id 等）为预存 WIP，非代码路径，本计划不动。`next_responsible_role: work（T-02：修复 AGE 面 + 冻结真实 SQLite 宿主/服务桥接合同）`

2026-11-22 stage: goal | plan_id: PLAN-113 | plan_revision: r1 | round: 1 | from: new | to: work | status: drafting | outcome: pass | worktree: D:/autostack/.wt/ui-113/{auto-ui,auto-lang} | commit: -（worktree 基线 auto-ui 87282fb0 / auto-lang 292575164，尚无代码改动）| next: work（T-02 AGE 写面修复 + T-06 A2Vue 独立可续）

2026-11-22 stage: work（T-02 第二批：AV 语义层四刀 + T-03 装载）| PLAN-113 r1 | outcome: partial | worktree: auto-lang `208357194` / auto-ui `ceb9045f`（plan-113-dev）| task_ids: T-02（完成）+ T-03（装载达成，宿主桥接待续）| evidence: `plan-artifacts/plan-113/`（av-evidence/av-ev2/av-script1.json 证据、scenario_diags 探针、grow/二分脚本）**AV 装载达成**：026 原样源 `--source` exit 0、51 applies、errors=0、15 文本叶核心 marker 在案；脚本 CommitTx/RollbackTx 派发 txStatusMsg 更新实证。**本批四项**：⑤记录元素 CtorNamed 改写（object-elem 消除）；⑥裸 `List` 惰性化（model 时点记录元素 + 后置非 view 循环迭代判定；p114 ids（view 迭代）与 expanded（显式 `List<str>`）契约保持）；⑦组件车道惰性列表 handler 跳过（handler_reads_view_data：检查函数 None + 事件程序 retain；包面 p105 list-iteration pending surface 契约不变）；⑧整模块投影 AGE 面 age_face 开（内联 helper fn 调用语句）；⑨桌面桥循环载荷按钮跳过；⑩helper Str 参数。**验证**：核心 conformance 897 过/5 败（5 红=text_record_source 主线既有，与 106 基线一致）；p105/p114 全绿。**剩余**：V-03 真实临时 SQLite 宿主桥接（读/写/独立连接核对提交回滚/失败恢复/连接释放）待 T-03 续；⑪A2Vue 包组件 SFC 待核。`next_responsible_role: work（T-03 续 SQLite 宿主桥接 + T-06 A2Vue 包组件）`

2026-11-22 stage: work（T-03 V-03 真实 SQLite 宿主桥 + T-06 A2Vue 浏览器腿收口 + T-02 ⑪ 复验）| PLAN-113 r1 | outcome: pass | worktree: auto-lang `403aa3724` / auto-ui `e9851d2c`（plan-113-dev；未入帐提交补验 a20d4287d/eb260694a/f430818e 同批）| task_ids: T-02（⑪关闭）/T-03（完成）/T-06（完成）| evidence: `plan-artifacts/plan-113/sites/026-database/`（App.vue 69KB/dist/shot.png/verify.mjs）、probes/gen_seed.py+check_seed.py（种子对源 identical 校验）、show_ev*.py（桥 evidence 读取）**T-03**：`ace_sqlite_bridge`+`ace_sqlite_bridge_seed`+`--sqlite-db` 入口 + a2r-std additive `clear_error`；`plan113_026_sqlite_bridge` 1 例 4.91s 全绿（真实查询 15 行/真实错误诊断/BEGIN→COMMIT/第二写→ROLLBACK/独立连接核对 16 行精确匹配/临时文件可删=连接释放）；modal 输入目标 14..17 探测实证。**T-06**：`plan113_026_vue_browser` 18 项浏览器断言全绿；首跑 11.87s 超 10s 新例线，10 处固定 wait 收敛后 7.68s。**验证台账**：auto-ui `cargo t` 1052 过/1 超时（rq_probe_t05_capability_rows，具名隔离 >286s 挂起于渲染清理，改动面无交，记主线待 triage）；auto-lang `cargo t ace` 四组（预构 663s）：ace-workspace 4097 过/120 红=source_foreign authority 系（1026 归档基线红）、ace-adapters 93/2（source_parent_oracle=1034/977/1098 master 基线红；hang_hits_exec_deadline 隔离复跑绿 0.65s=并行负载环境归因，939 先例）、ace-batch 3/9（KNOWN-DEBT 4928 "case fence missing" 基线债）、ace-facade 95/1（p969_t02_console_lane=D-1098-01 具名债务）——**全部具名分流，本计划三提交零新增红**；plan981 全套（grammar scope/event/a2x-worker/av）在 ace 组内绿，唯 2 例 p114 tree 红——**merge 档最终更正（两次前判均不完全）**：①初判「基线预存」错误；②二判「语法数据污染致 ~100 跨族红」亦错误（数据污染是真实独立缺陷、5981a0089 已修，但重生成后跨族红仍在）。**真因（2026-11-23 merge 门定谳）**：`1a9a18678`（=208357194，T-02 主批⑥裸 List 惰性化后置判定）与新 master 基线的 **p114 契约正面冲突**——分支内逐提交金丝雀二分（ce171c229 绿 → 1a9a18678 起红，master 绿）；⑥把「非 view 循环迭代的裸 `List` 字段」一律转惰性 view-data，p114 的 `expanded`（handler 引用 `.get()/.len()`/helper 传参）因此失去类型化状态语义而 Bind 红；rebase 后 ~95 例跨族红（text_record_source/sched_m2_1067/text-conformance/language_conformance/av_method/py_lane/resource_915/stdlib_list_946 等）同根待逐族复核。**冲突双向实锤**：merge 档试修（handler 引用豁免，未提交已还原）使 p114 双测转绿但 026 装载即拒（`source.undefined: no function named toggle_id`——dbExpanded 转类型化后 `.DbToggle` handler 进入受检，而 AV 组件面单源投影不可见 `tree_util.toggle_id`；⑥的惰性恰是该拒收的 fail-closed 屏蔽）。**结论：设计冲突，needs_replan**——判别式设计决策（单模块组件面 handler 解析 use 导入 helper 的授权/惰性豁免口径）超出 merge 档权限，见 §9 merge 记录。数据污染修复（5981a0089）与 spec §6 保留在分支上。本会话新例 2 例均 ≤10s（4.91s/7.68s）。`next_responsible_role: replan（new 技能，有界修订：⑥惰性判别口径）`

2026-11-22 stage: review（final；T-04/T-07 合并）| review_scope: final | reviewer: 实现同会话（限制已声明——结论以产物独立重建：两个 plan113 用例 + AV 裸装载由本审查独立复跑，批量证据复用本会话已钉命令/输出）| plan_id: PLAN-113 | plan_revision: r1 | outcome: **pass** | worktree: `D:/autostack/.wt/ui-113/auto-ui` @ plan-113-dev `e9851d2c`（base `87282fb0`）；`D:/autostack/.wt/ui-113/auto-lang` @ plan-113-dev `403aa3724`（base `292575164`）| dependency_revisions: auto-web `0cfc7c7`、auto-down `456c86a` | spec_inputs: 冻结源 9 文件 SHA256（§9 T-01）；钉版 app.at `D37E703351DEDFEB`；种子 fixture 校验 identical（check_seed.py）| acceptance_results: **AC-01 pass**（T-01 冻结矩阵+两端入口实测记录）；**AC-02 pass**（V-03 全项：独立复跑 `plan113_026_sqlite_bridge` 5.20s 绿=真实查询/真实错误诊断/失败恢复/提交/回滚/独立连接核对/连接释放；AV 裸装载独立复跑 exit 0/errors=0/7 核心 marker 全在/12 buttons；ace 组 plan981 消费面回归绿）；**AC-03 pass**（独立复跑 `plan113_026_vue_browser` 8.14s 绿=生成/vite build/真实浏览器 18 断言含交互/错误/关闭）；**AC-04 pass**（本记录：提交干净、HEAD 复校、132 红全部分流具名、零新增；~~p114 2 红文件级 bisect 证基线预存且 master 已修~~——**merge 档两次更正**：分流结论错误；最终定谳见 §9 merge 记录=⑥设计冲突 needs_replan；本复审的 p114 分流证据作废）| **新例成本门**：2 例均 ≤10s（5.20s / 8.14s；vite build 属显式 WEB_E2E 构建验证道）；无日常测试内编译前置 | pending_closeout_gates: 统一批量回归（`cargo t all` due-triggered 主检出单实例）+ 规范沉淀（merge 按前件沉积 `auto-lang:docs/specs/ace-migration/m2-age.md` 增量与新建 `auto-ui:docs/specs/runtime/ace-sqlite-host-bridge.md`；frontmatter 三字段已 finalize）| findings: 无 | evidence: §9 各 work 记录 + §10 triage 表 + probes/{gen_seed,check_seed,show_ev}.py | next: merge（T-05 AV 行交付 + T-08 A2Vue/归档）

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

2026-11-22 验证 triage（均具名分流，不阻塞本计划，无一是本计划提交新增）：
- `ace-abi::source_foreign::*` 120 红（foreign.authority.runtime-or-emitter）：PLAN-1026 归档已记基线红（base c02c6dc8f 复现）；owner=批量回归/原写面计划。
- `p114_tree_ids_toggle_with_the_checked_helper_loop` + `p114_tree_expanded_lists_replace_and_clear_in_separate_av_instances`（ace-av conformance）：~~基线预存~~ ~~语法数据污染~~ **merge 档定谳：1a9a18678 ⑥ 与 p114 契约的设计冲突**（逐提交二分实证；试修反伤 026；needs_replan，见 §9 merge 记录）。
- `source_parent_oracle_failure_rejudges…`：master 基线红（1034/977/1098 记录）。
- `hang_hits_exec_deadline_as_error`：并行负载红、隔离复跑绿（worktree 0.65s / master 0.57s；PLAN-939 环境归因先例）。
- `ace-batch` 9 红（"case fence missing"）：KNOWN-DEBT-AND-RISKS 4928 基线债（base d20cf3f87 8 红）。
- `p969_t02_console_lane_grants_and_capture`：D-1098-01 具名债务（KNOWN-DEBT 5028）。
- auto-ui `rq_probe_t05_capability_rows`：union 120s 超时 + 独立重跑 >286s 挂起于全部断言完成后的渲染清理；与本计划改动面（ace_node_desktop/sqlite 桥）无交；owner=auto-ui 主线 triage。

2026-11-23 stage: merge（启动后中止，needs_replan 转出）| PLAN-113:r1 | outcome: **needs_replan** | worktree: auto-lang `D:/autostack/.wt/ui-113/auto-lang` @ plan-113-dev（rebase 后链 ce171c229→1a9a18678→7c94009e3→ff72c5e15→551317467→b4bb561ae→5981a0089；range-diff 与复审基线全 "=" patch 等价）/ auto-ui @ plan-113-dev（rebase 后 tip `c0def548` 规范档）| prepared: 复审基线+冻结 delta 已核；worktree 两树干净 | landed: **未落地**（检出即停）| canonical_verified: 规范增量只在分支（auto-lang m2-age §6 @ b4bb561ae、auto-ui ace-sqlite-host-bridge.md @ c0def548），未合主 | 关键过程证据：①rebase 两仓零冲突、range-diff 全等价；②auto-ui rebase 后门首跑编译断（master 依赖 auto-lang 新 HttpProvider），auto-lang rebase 后恢复；③**检出 a20d4287d 提交的 `syntax_grammar_data.at` 系构建日志污染**（重定向事故）→ `5981a0089` 用 generate-grammar 重生成（保留在分支，真实修复）；④干净机复跑 ace 门（run-kkbei3k5）：基线红外仍 ~95 例跨族红（text_record_source 5/sched_m2_1067 23/text-conformance ~20/language_conformance ~10/av_method/py_lane/resource_915/stdlib_list_946/governance/native tiers/p114×2）——master 同窗口零新红（236ed3d84 批量收据），故全部归本分支；⑤**分支内逐提交金丝雀二分**：p114 在 `ce171c229` 绿、`1a9a18678`（⑥裸 List 惰性化）起红 → 锁定⑥；⑥**双向冲突实锤**：试修（handler 引用豁免，project.rs 未提交已还原）→ p114 双测绿但 026 AV 装载即拒 `source.undefined: no function named toggle_id`（dbExpanded 转类型化 → `.DbToggle` 进受检 → AV 组件面单源投影不可见 tree_util helper；⑥惰性恰是现架构下该拒收的 fail-closed 屏蔽）；⑦auto-ui 干净门未复跑（依赖修复定谳后一次跑）。| blockers: **设计冲突需有界修订（replan）**：判别口径二选一或第三方案——(a) ⑥惰性豁免扩到 handler 引用 + AV 组件面 handler 受检对单模块不可解析 helper 的具名降级（延 ⑦ skip 架构，触碰共享 verify 的 fail-closed 纪律，需 owner 裁决）；(b) p114 契约按 106 B2 惰性先例修订（改测试与树包合同，需 p114 owner 同意）；(c) AV 组件面单源投影解析 use 导入 helper（新能力，A2Vue 已有先例 f430818e）。| 分支/工作树**保留**待 replan；两仓主检出未动（auto-lang 主检出仅有他方 WIP，未碰）；plan-artifacts 保留。| next: /auto-plan:new 有界修订（⑥ 判别口径），修订后 work→review→merge 重来受影响面。

2026-11-23 stage: work（r2 有界修订第一批，replan 后续）| PLAN-113:r2（语义修订，frontmatter 待翻）| outcome: partial | worktree: auto-lang `f9dc0acf3`（=5981a0089+门面判别提交）/ auto-ui `c0def548` | 修订内容：⑥后置判定**收窄为组件面专属**（`profile != AgePackage` 才惰性化）——包面恢复 master 语义（裸 List 全类型化），p114 expanded 合同修复（双测绿复验）；026 组件面仍在惰性路径。**026 墙精确定位（全证据链）**：桥测试金丝雀逐提交二分（ce171c229/1a9a18678/7c94009e3/ff72c5e15 全红，同 `no type named List` 墙；plan085（pkg/tree-view）在 1a9a18678/7c94009e3 全绿、master 8/8 绿）→ 墙=**pagination 包**（`packages/pagination/pagination_control.at` 裸 List 空局部 `var size_candidates List = []`）且**只在 026 全量装载（frontdoor 包闭包道）出现**；master 自身装载 026 在更早的 `source.age.slot` 墙（无本计划 AGE 准入），故 master 门从未覆盖此道。**基线对拍**：old master+本计划提交（T-03 时）桥测试绿；new master+同文义提交（resolve.rs/project.rs/auto-source.at 逐字节同、grammar id 冻结同）桥红 → 差异收敛到 master 的 6 个 Plan-108 cst/frontdoor 提交与本计划⑥**包件投影路径安置**的交互：p114（`compile_age_package_with_scenario` 老 Engine 道）⑥仍生效（故红→门面判别修复），pagination（frontdoor 包闭包道）⑥不再覆盖（故墙）。**下一步（r2 第二批）**：把裸 List 惰性化挂到新 frontdoor 包件投影道（或在其上补同律 pass），复验桥+p114+plan085+ace 门。auto-ui 侧 prep 全段路径解析试修已还原（该修只在 r2-全类型化路线需要；现路线不需要）。本会话探针日志 /tmp/bridge_{ce,1a,7c,ff}.log、/tmp/p085*.log。

2026-11-23 stage: work（r2 第一批验证补充）| `f9dc0acf3` 复核：`plan981_age_package_av` 全 binary 37/37 绿（r1 时 2 红）——门面判别零回归且修复面完整；p114 双测绿复验。剩余唯一红=026 装载（pagination/frontdoor 包闭包道，见上条记录）。

2026-11-23 stage: work（r2 第二批诊断，未改码）| 墙进一步窄化：**base 对拍证明 cst/frontdoor.rs、cst/project.rs、resolve.rs、auto-source.at 在 292575164..ca7c8e07e 全部零改动**（grammar id 冻结自证）——此前「Plan-108 frontdoor 交互」假设排除；两基线间 av crate 仅 component_event_sessions.rs（+366，http 会话悬挂）与 cache.rs（+19）改动。**错误分类实证**（/tmp/bridge_ce.log 全诊断聚类）：026 装载墙全部为 **tree_view_util.at 包件 fn 体裸 List 方言错**（`no type named List`×12、`var out/stack` 局部、len/push/pop/contains/index 级联），**无 toggle_id 解析错**（app 层 handler 面干净）→ 墙=**pac.at dep 闭包道对包件 util fn 体的全量受检**，而 plan085（同包、`use { package: … from 相对路径 }` 于夹具 widget）在同事提交全绿——同包两上下文编译行为分歧。**下一步（精确切片）**：最小对拍探针——同一 tree_view 包分别经 (a) plan085 式夹具相对 use 与 (b) pac.at dep 闭包装载，定位两道的编译门差异（module reachability / fn 体受检开关），再决定修在 reachability  pruning 还是 fn 体受检门；修后复验桥+p114+plan085+ACE 门。

2026-11-23 stage: work（r2 第二批诊断②，判别实验定界）| **旧 auto-ui（e9851d2c，T-03 当时代码）+ 新 auto-lang（f9dc0acf3）→ 同墙红**（/tmp/bridge_oldui.log）→  culprit 在 **auto-lang 侧**，此前「新 auto-ui master」假设排除；而 cst/resolve/project/grammar 已证逐字节同 → 剩余最大嫌疑 = **297d4f1d8（PLAN-1052 AAC incremental driver & package closure Auto modules）**——包闭包道的模块提交/受理方式改动（tree_view_util 可能被作为普通 Auto 模块受理，裸 List fn 体在严格 profile 下拒；旧闭包按 AGE 包件宽受理）。**下一步切片**：读 297d4f1d8 的包闭包改动 + 对拍其前后闭包单元清单（026 装载的 units 构成），定位受理 profile 差异后修（或在闭包道补 AGE 同律宽受理）。实验矩阵已闭环：{旧ui+旧lang=绿（T-03）, 旧ui+新lang=红, 新ui+新lang=红, plan085 动态道任意=绿}。

2026-11-23 stage: work（r2 第二批诊断③，矩阵收官+新假设）| **决定性反转**：**旧+旧收官格（auto-lang 403aa3724 + auto-ui e9851d2c，T-03 当时完全相同的提交对）今夜复跑同墙红**（/tmp/bridge_oldold.log）——墙**非提交决定，是时间/环境依赖**；全部「新 master 交互」假设（frontdoor/resolve/project/grammar/av-crate/1052 AAC)已逐一排除（或纯增量、或零改动）。已验通道：026 源哈希=冻结钉版、无 env 门、provider 缓存 11-22 14:15 未动、syntax 编译器零改动。**当前假设=增量编译缓存损坏**（今夜 20+ 次 checkout 往返后 .fingerprint/incremental 失配 → 同源异码）；验证中=清 target/debug/incremental+.fingerprint 后 old+old 全量重跑（/tmp/bridge_clean.log）。若绿：全部门红需按此重估（含 95 跨族红与 p114 二分结论的可靠性）；若仍红：剩唯一外部通道=两仓 path-dep 之外的共享工件，转隔天环境对拍。

2026-11-23 stage: work（r2 第二批诊断④，假设检验收官）| **增量缓存假设死**：清 target/debug/incremental+.fingerprint 后 old+old 全量重编译仍同墙红（/tmp/bridge_clean.log）；**确定性确认**：同一测试二进制连跑 3 次同红（非哈希序非确定性）。已排除通道全集={提交内容（old+old 同提交复现）、frontdoor/resolve/project/grammar/syntax 编译器（逐字节同）、av-crate（http 谱系）、1052 AAC（纯新增）、Cargo feature/成员（纯新增）、026 源（哈希=钉版）、env 门（无）、provider 缓存（11-22 旧）、ui980 根（工作树本地）}。**剩余唯一方向**：包闭包编译的运行时外部输入——下一步=带名字的进程内探针（compile_age_package_with_scenario 挂真实 tree_view/pagination 包源，逐单元看哪个构造在哪个模块 Bind 拒），定锚后修。**工作树事故已处置**：误 pop 他方 stash（plan-107，仓级共享）致 3 文件冲突，`git reset --hard HEAD` 恢复，他方 stash 原样保留——教训：pop 前必查 stash list 且带 ref 名。

2026-11-23 stage: work（r2 第二批诊断⑤，探针尝试与边界）| 进程内探针（compile_age_package_with_scenario 挂真实 tree_view 包源，两种 package 形态） repro 的是**入口语义墙**（`expected fn, found widget`）而非真墙——探针缺 `parse_package_source_age` 预处理（auto-ui ace_widget_packages 提交前改写），与 026 真实闭包提交形态不等价；探针已还原。**诊断边界判定**：环境通道在穷举排除后仍 unidentified（提交/缓存/非确定性/兄弟仓/ junction /provider/env/语法编译器全部证否）；继续远程诊断的成本已超收益，剩余手段=文件访问跟踪（Procmon/handle）或环境 owner 对拍。**当前可交付态**：r2 第一批（p114+包面 37/37 绿、数据污染修复、两规范增量）稳固在分支；026 腿被环境墙阻塞（同提交昨夜绿今夜红，非代码回归）。**建议**：环境墙以 fresh 会话/机器状态对拍优先重试（可能是跨会话残留的进程级/系统级状态）；若 fresh 环境复绿，则今夜全部下游红需重估。

2026-11-23 stage: work（**环境墙破案**）| 环境二分实证：测试二进制在 `env -i` 或 PATH 去 autostack 或 **unset PYTHONPATH 或 unset PYO3_PYTHON**（任一）下→**桥绿（4.0-4.6s）**；本会话 shell 的 `PYTHONPATH=D:\gitee\z-lang\zc\lib\py` + `PYO3_PYTHON=C:\Python314\python.exe` 同时存在 → 墙。机制=ACE 运行道惰性嵌入 PyO3，Python 可用性改变装载/受理行为（裸 List fn 体拒）——**AGE 装载对 ambient Python env 非 hermetic 的卫生缺陷**（auto-lang 侧，跨计划）。昨夜 green=当时 shell 无这两个变量；今夜全红=Kimi 会话 shell 带它们。**重大含义**：今夜 ACE 门 ~95 跨族红与 p114 红均在本 env 下测得，疑同因；**重估门**：unset 二变量重跑 ACE 门（ prepares+4 组）；若绿，r2 门面判别降级为保险性修订（可保留），merge 路径重开。破案探针：/tmp/bridge_oldold.log（红基线）vs unset 系列绿。

2026-11-23 stage: work（**环境墙定谳 + r2 门重估**）| **干净 env ACE 门（run-m66090_t @ f9dc0acf3）终局对账**：source_foreign 117（1026 基线）/batch 9（4928）/adapters 1（source_parent_oracle）/facade 1（D-1098-01）=文档化基线；**av_method/av_owned/resource_915/component_event_sessions_1106/sched_m2_1067/text-conference 族（~78）=master 同环境同红**（av_method_owned/sched_sleep/textops_v07/resource_915/v02b/av_owned_first_case 七例主检出实测红——本环境基线红块，同 source_foreign 类，归批量回归 owner 做环境卫生）；py_lane 7 + stdlib_list_946 2 = **需 Python env**（unset 即红，full env 分支 31/31 绿）+ py_lane 尚需 worktree 跑 prepare-av-providers（制备缺口非代码）；**p114=绿、026 桥=绿（hermetic 测试 f865b20d 后全 ambient env 5.39s 绿）、plan085=8/8 绿。结论：r2 分支零可归因新红，merge 路径重开。** 门槛卫生要求（本计划实证发现）：ACE 相关门须在无 PYTHONPATH/PYO3_PYTHON 或与其兼容的受控 env 下运行——AGE 装载道对 ambient Python 非 hermetic（昨夜绿今夜红根因）；桥测试已自净（env_remove），门脚本卫生归 auto-lang infra owner。**待办（merge 续）**：master 又移至 ee433bfe2——land 前需再 re-sync rebase + 受影响门一轮（ace 组 prepare~10min + auto-ui 三门）；随后 ff-only 落地两仓、归档、清理（merge 技能流程）。

2026-11-23 stage: merge（收口，outcome: **pass**，completion_kind: delivered）| 检查点：`prepared`=复审基线+冻结 delta+规范增量已核 | `landed`=auto-lang master tip `d4bc60a89`（ff-only，rebase range-diff 全等价链 7a0799b5b→d4bc60a89）+ auto-ui master tip `0bbe4a50`（ff-only；land 前对 828dd33d 二次 re-sync 零冲突）| `canonical_verified`=auto-lang `docs/specs/ace-migration/m2-age.md` §6（10650f1f7）+ auto-ui `docs/specs/runtime/ace-sqlite-host-bridge.md`（24052cc4）均在落地提交 | 落地后烟（landed tip 实测）：ace_node_desktop 980 组 11/11、plan085 8/8、plan113 桥 1/1（4.01s）、plan113 浏览器 1/1（7.16s）全绿；auto-lang ACE 门（run-96m_ca0l @ d4bc60a89）128 红=文档化基线 128（source_foreign 117/batch 9/adapters 1/facade 1）+本环境 master 同红块 88（av_method/av_owned/resource_915/component_event_sessions_1106/sched/text-conference 族 + py_lane 制备缺口，逐族主检出实测 master 同红）——**零可归因新红** | `archived`=docs/plans/archived/113-*.md（本文件）+簿记提交 | 环境遗产（本计划实证，移交批量回归/auto-lang infra owner）：AGE 装载道对 ambient PYTHONPATH/PYO3_PYTHON 非 hermetic（昨夜绿今夜红根因；桥测试已自净 env_remove）；ACE 门须在受控 Python env 下运行 | 部署观察：landing≠deployment——生产宿主二进制/产物未重建（按规约不隐式触发）| 统一批量回归门=cargo t all（due-triggered 主检出单实例）留批量回归档。
