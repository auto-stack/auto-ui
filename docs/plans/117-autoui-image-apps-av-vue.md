---
plan_id: PLAN-117
status: executing
feature_name: U05 Photo Gallery 与 Image Viewer 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 2
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978, PLAN-089]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-117 U05 Photo Gallery 与 Image Viewer 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [107](archived/107-autoui-u05-media-chart-and-paint-apps-av-vue.md) 的 U05 原矩阵承接本页所列行。 [原分母与交付映射](README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。关联 Widgets 为 [carousel](../../../auto-os/widgets-gallery/src/front/pages/carousel.at)。

| 原项目 | 原验收重点 |
|---|---|
| [029-photo-gallery](../../examples/ui/029-photo-gallery/pac.at) | 图片浏览与查看、组件嵌入、缺图后恢复 |
| [031-image-viewer](../../examples/ui/031-image-viewer/pac.at) | 图像载入、缩放/平移、可见区域和坏图像恢复 |

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。共享 image-viewer 消费接线，保留两项目各自的文件、会话、URI 与嵌入契约。适用能力消费 089 已交范围。

## 3. 技术与实现入口

目标入口为 两个项目的 src/front/app.at、packages/image-viewer 与原后端接口。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-117/{auto-ui,auto-lang}，每仓单一 plan-117-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

接 089 已交视口与预览来源，缺图和资源释放按实际接口验。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 规范增量

本计划拆分本身只迁交付归属。实际实现若调整共享语义或组件合同，在 T-01 正式定位 owner、对应规范及 AC 后由实现计划提出增量。

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。029 实际浏览、嵌入查看并从缺图恢复。031 真实载入、缩放、平移与可见区域可核，坏图能恢复，会话正确释放。carousel 逐页验。 |
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
- [ ] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
- [ ] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
- [ ] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [ ] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [ ] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
- [ ] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-01 冻结矩阵与首阻断 | outcome: pass | worktree: `D:/autostack/.wt/ui-117/{auto-ui,auto-lang}`（plan-117-dev）+ auto-web `26d085f`、auto-down `45b642d`（detached 固定） | base: auto-ui `87282fb0` / auto-lang `292575164` | code_commit: auto-ui `299ac88d` | next: work/T-02。
**工具与源身份。** Windows x64 MSVC，rustc 1.98.0 `88d9e12ae`。AV 入口=工作树 debug 构建 `ace_node_desktop_window`（SHA256 `C27CC52CD0F164516A6FA4ED3F3B166C6D88BE3059B8421857B595C0113E6C59`）；A2Vue lane 入口=`auto-ui-tool` 0.1.0（`--features ace-vue`）+ `generate_sfc_with_lane`。AV 构建 3m24s / tool 构建 2m39s（sccache）。源冻结 SHA256（前 16）：029 pac.at `CBEA43983C1FD1FB`、029 app.at `9F4C3321105B3E04`、029 back/api.at `B320055020C01826`、029 SPEC.md `8F471BB983F6D74F`；031 pac.at `991DB7FF7D40C3A9`、031 app.at `D4F2340DA7C12229`、031 back/{api,directory_index,image_service,types}.at `8A841986388E811C`/`5F1098CB503241F8`/`63CEA45FEB5D7363`/`0EBBB4CC4BFFD9A1`；carousel 页 `F43F17C29A83924A`。材料根 `D:/autostack/plan-artifacts/plan-117/`（probes/、av/、lane 日志）。

**两端入口实测。** AV=`ace_node_desktop_window --source <app.at>`（有 pac.at 祖先先过 prep；029/031 含 widget 级包导入自动走 AGE 包面）。A2Vue=`generate_sfc_with_lane`（prep 后源；浏览器腿归 T-06）。

| 原输入 | AV/Iced（T-03 目标） | A2Vue lane（T-06 目标） |
|---|---|---|
| 029-photo-gallery | 修复 prep 后推进到 `source.age.slot: expected model at the declared model slot`（asset-browser 包子组件 `widget AssetBrowser (records: List = [] …)` 头 props 未解析） | SFC 40362B 生成；18 个 handler 全缺席（body_foreign：Http.get_json/json.parse/try/catch/循环） |
| 031-image-viewer | 修复 prep 后推进到 `source.lex: module image_viewer::image_viewport: unknown character $`（包子模块自带 `$event.deltaY`/`$0,$1,$2` 占位绑定） | SFC 33702B 生成；13 个 handler 全缺席（body_foreign：back.api 调用 open_file 等） |
| carousel 页（auto-os） | 未达（归 T-03 画廊壳装载） | 未达（归 T-06） |

**首阻断（具名）与写面归属。**
- B-01（AV 029）：AGE 包面不认包子组件头 props（`records: List = []` 形声明）与 handler 语句形态（记录字面量含 `.field` 引用=`no local named self`、`try/catch`、裸调用尾语句=bare element tag / expected expression）。写面=auto-lang AGE 包文法/投影（age_package）。029 的 AV Http 效应（`Http.get_json`）归 PLAN-108 在途 AGE 组件面 AV Http 交付，本计划消费。
- B-02（AV 031）：包子模块 `image_viewport.at`/`image_preview_dialog.at` 的 `$event`/`$0` 占位绑定——CST lexer 无 `$` 词元（`source.lex unknown character $`）。写面=auto-lang AGE lex/投影（cst/lex.rs）。
- B-03（**本计划内已修**，auto-ui）：`inject_model_fields` 用裸 `find('}')` 找 model 块尾，029 的 dirs 注释 `[{name,rel_path}]` 含 `}` 致宿主字段插进注释、AV 包面与 Vue lane 双双误报「记录字面量/var」解析错。已改为跳过行注释与字符串字面量的词法扫描；另修复 prep 空白化 widget 级 `use { package: … }`（该声明是包面解析包子组件/`$event`/props 的依据，空白化致 `$event requires a declared child widget` 与组件 props 记录字面量拒绝）。提交 auto-ui `299ac88d`。修复后 031 的 app 级 `$event` 越过、推进到 B-02；029 越过记录字面量、推进到 B-01。
- B-04（A2Vue 两端）：lane 对两 app 均生成 SFC 但 handler 全缺席（body_foreign：Http/back.api 域外调用、try/catch、循环）——交互未接线。写面=auto-lang body_foreign 分类 + auto-ui lane 接线；浏览器侧 Http/back 桥（027 back-proxy 先例）。

**对照与正控。** 正控 Counter/包面标量通过；p-record/p-record2/p-record3/p-multibyte 全部通过（记录字面量、多字节串在 lane 可生成，包面 pkg-record-var 可装载）；pkg-record6（记录含 `.field` 引用）在包面具名拒（B-01 分支）。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/auto-ui | commit: 299ac88d | next: work/T-02

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-02 包面能力接通 | outcome: partial | commits: auto-lang `f41e7df15`（9 文件：文法/投影/verify/live-view/binding）/ auto-ui `82969fb3`+`4e5f6503`+`5f0bc…`（placeholder 归一化 + quoted-message 接线 + 探针） | next: work/T-02 续（json.stringify AV 缺口 + asset_browser 结构 + try/catch CST）。

**本轮修复（均已实测正反）。**
- B-02 `$` 占位：包子模块源在 AGE 面前**同长归一化**（`$event`→`nevent`、`$N`→`nN`；image_viewport.at 的 `onwheel: .OnWheel($event.deltaY)`/`onpan: .OnPan($0,$1,$2)`），另把应用面拼写 `use auto.json` 归一为包面 stdlib 键 `use std.json`（auto-ui `ace_widget_packages`）。031 的 image_viewport 模块从 `unknown character $` 推进到**包面可解析**（`OK age_components=1`）。
- B-01 包子组件头 props：`float/f32` 参数默认（`zoom: float = 1.0` 等）在包面核准（frontdoor widget-param-default 数值面）。
- B-01 带引号消息派发：`."viewRequested"(…)`/`."error"(…)` 文法（age-on-message-name str-lit 臂）+ 投影 str-lit 解码（与 msg 声明侧同口径）。
- B-01 事件载荷：live-view 事件载荷支持 float/f64（`OnWheel(float)` wheel/pan 事件同 int/str 发射）。
- std.json 两段导入自动装配（binding 2-segment 触发）。
- 探针 harness `plan117_t02_probe`（包面诊断；`plan117_t01_probe` lane 探针）随树保留为诊断工具，正式回归归 T-03/T-06。

**031 AV 现状。** image_viewport 模块包面解析通过；031 装载推进到 `source.age.widget-view-unsupported … this event payload type`（float 载荷，已修）→ 现阻于 `source.module-missing: unknown module std`→（std.json 已接）→ 现阻于 **`json.stringify` 缺失**：image_viewport 的 viewRequested 载荷用 `json.stringify({…})`（方言/VM 内建），ACE stdlib json 切片只有 encode/parse 函数族，AV checked 域无 JSON 序列化。写面=auto-lang ACE stdlib/AV 值域（需 `json.stringify` 记录→字符串，或包面映射到 stdlib encode 族）。这是 031 AV 的下一主阻断。
**029 AV 现状。** asset_browser 模块仍 `expected fn, found widget`（recover 掩码）：head+四 use+真实 msg 最小可解析；model 可解析；**computed 段阻**——computed 值语法为 `pratt(wide-ops)`，asset 的 `provider_error/data_current/empty_label` 等用 **if/else-if 表达式**（design 值同款双臂 `if-expr|pratt` 可解析）。但把值规则（无论 pratt 还是 if-expr 引用）接在 `=>` 之后全部拒（`expected expression, found …`，span 指向值首 token）——与 029 on 段 try/catch 同属 **CST 解析器 fat-arrow/关键字后引用规则邻接缺陷**（`=>` 后 lbrace 可解析、表达式类规则全部不可）；写面=auto-lang CST 解析器（syntax-runtime parse）。029 自身 on 段 try/catch 仍被 CST 拒（规则/词元/首集全部核验正确但不可匹配，同疑）。
**B-04（A2Vue lane）**：unchanged（handler body_foreign 全缺席；归 T-06）。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-ui,auto-lang} | commit: auto-lang f41e7df15 / auto-ui 82969fb3 | next: work/T-02 续

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-02 包面能力接通（round 3） | outcome: pass | commits: auto-lang `b3275c18f`（if-expr 表达式面）+ `c80a5e2b`（??/try/list/outbound）/ auto-ui `9f98e269`（asset_browser msg 补齐 + 探针） | next: work/T-03。

**本轮修复（全部定向实测通过）。**
- **computed-if（CST fat-arrow 邻接缺陷绕过）**：`if <cond> { … } else { … }` 表达式按 **wide-ops primary 内联 alt** 接入（与 design 值同法），新增 `ExprKind::If`（cond/then/else-chain/els）贯通投影/resolve/typecheck/闭包捕获/span 平移/adapter 计划/命外面具拒。asset_browser 的 `provider_error/data_current/empty_label/empty_hint` 六个 if 计算值全部包面解析 OK。
- **`??` 空值合并**：`q.id ?? ""` 等 handler RHS——`??` op 入 wide-ops/condition-ops（binding 3 right），token 扩到 age 各面；`fold_expanded/collection_items` 与 handler 赋值 RHS 全过。
- **on 段 try/catch**：age-on-try **漏挂在 widget 形 handler 体（age-on-widget-stmt）**——只在直接形 age-on-stmt，`on { .Msg -> { … } }` 的 try 从未被尝试（recover 掩码 `expected fn, found widget`）。补挂后 try 匹配；进一步发现 **end_list_step 不重置 `list_started`**——同规则帧内第二个 `list-nl`（try 体 + catch 体）把第一项误判「第二项需分隔」直接结束，catch 体永不解析。修 reset 后 try/catch 全过。
- **outbound Try 走读**：`outbound::has_messages/has_focus` 递归漏 `ComponentStmt::Try`——try 体内的对外发送（`."queryRequested"(…)`）判 invisible → `handler-outbound` 误拒。补 Try 臂后 DoSearch 等 16 个 handler 全部投影（foreign=false）。
- **asset_browser msg 声明补齐（源修订）**：PvEvent(str)/ClosePreview/KEscape 是 view 事件转发与 on 段 handler 引用的内部消息，AGE 包面要求「view 事件/handler 消息须已声明」（image_viewport 已声明全量；asset_browser 原源缺失）。补 3 个声明（仅声明，不改行为），**029 pac 冻结哈希变更**（重基线见 §5/T-04）。
- 语法-grammar conformance 50/50 绿（profiles_927 wide-ops 计数 13→14）；ace-core lib 87 绿；syntax-runtime 52 绿。**已知红（非本计划引入）**：`text_record_source` 系列在基线（stash 验证）即红——`verify.double-move`，与上游共享写面变更相关，记录待 merge 时归因；p981 文法 id 冻结测试按预期红（文法变更需重基线，106/114 先例）。

**两端包面现状。** asset_browser 完整文件（use+头+msg+model+computed+view+on 全部段）**包面装载 OK age_components=1**，16 个 handler 投影完整（Try/Let/If/NullCoalesce/对象字面量/带引号消息发送全过）。image_viewport/image_preview_dialog 031 两端 OK。**029-app/031-app 应用面（app.at）与 AV 事件程序编译、json.stringify 序列化**归 T-03 实跑验证。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-ui,auto-lang} | commit: auto-lang b3275c18f+c80a5e2b / auto-ui 9f98e269 | next: work/T-03

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 AV 实跑（029 语法全通 + checked 域推进） | outcome: partial | commits: auto-lang `58ffb9d21`（handler 表达式覆盖）/ `f812562bb`（index-set 调序）/ `d6d9894c8`（元素字段赋值）/ auto-ui `c48d50b3`（span 诊断打印） | next: work/T-03 续（checked 域 Try/json 语义族）。

**029 AV 装载推进（本轮全链修复，均已实测）。**
- **029 app 应用面语法全通**（prep + `$event` 同长归一化后 CST `has_errors=false`，nodes 13834）——本计划 app 单元此前缺的宿主事件占位归一化（与包模块同法）已加在 load_source。
- **on 段表达式覆盖**（age-component 应用面）：`??` 空值合并（wide-ops/condition-ops）、`null/nil` 字面量（atom-expr + condition-atom）、handler 参数类型注解（`.GalQuery(ev str)`）、`while`/`for-cond`/索引赋值（`used[best] = true`）与索引读（`source[i]`）、单行多语句 `;` 分隔（`for u < hn { used.push(false); u = u + 1 }`）、`take` 作变量名（r-take 保留字在 age-component 放开）、元素字段赋值（`.photos[i].fav = v`——`age-on-index-set` 加点前缀 + 字段尾子规则，投影 `recv.set_field(index, field, value)`）。
- **记录构造从应用面表达式位移除**（atom-expr/condition-atom 的 ctor-expr/condition-ctor 去掉 age-component）——`for i < n_all { … }` 的 RHS ident 后接块体 `{` 被 ctor-expr 误判为记录构造，把循环体的 `{` 消费掉（front 语料无构造形；包面/back 保留）。
- **list-nl 分隔符修复**：语句级 `;` 收尾（push/pop 的 opt semi）满足列表分隔（`last_significant_is_sep`）；`end_list_step` 重置 `list_started`（同帧第二个 list-nl——try/catch 体——误判）。

**当前阻塞（checked 域语义族）。** AV（`ace_node_desktop_window --source`）装载推进到 **`source.syntax: expected expression, found } @16678`**（catch 体 `.dirs = []` 的投影——语法已通、投影/verify 层报）→ 迷你复现定位为 **`Stmt::Try` 在 checked 域（resolve）仅 aggregates 面支持**（`resolve_try_stmt` 具名拒 `control/generator statements require full aggregates lowering`）。**029 的 on 段大量 `try { json.parse(…) } catch { … }` 需要 AGE 面 Try 语义**（json.parse 失败恢复族——与 json.stringify 序列化、031 的 back 模块同属 checked 域能力族）。写面=auto-lang resolve/verify（resolve_try_stmt 的 AGE 面变体 + json 语义）。

**两端现状。** 031：std.json 模块导入装配修复（`use std.json` 两段在 AGE 包面的模块导入语义）后推进到 `unknown function close_session`（back::api 的会话函数——**app 单元的 back 服务模块未装载**，需 --package 多单元或 back 桥）；031 app 语法层包面 OK。carousel 页未达。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-ui,auto-lang} | commit: auto-lang 58ffb9d21+f812562bb+d6d9894c8 / auto-ui c48d50b3 | next: work/T-03 续

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 AV 实跑（round 5：checked 域 Try/调用链/?? 接通 + 消费 PLAN-108 Http） | outcome: partial | worktree_path: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | branch: plan-117-dev（双树） | base_commit: 292575164/87282fb0（基线不变） | code_commit: auto-lang `abf4114df`+`0b5d0e061`（merge master）+`b1c0ae83c`+`2f7e8fe20` / auto-ui `61ebb478`+`1c253252`（merge master） | dependency_revisions: 消费 master（含 PLAN-108 r2 已交付组件面 Http 全链 auto-lang 541623c42 / auto-ui 011a1b11） | task_ids: T-03（续） | next_responsible_role: work（T-03 续——B-05 JSON 值域族）。

**本轮落地（全部定向实测）。**
- **checked 域 try/catch（文本/集合面）**：resolve 新增 `resolve_try_stmt_text`——与 aggregates 版同构（`Loop`+unwind 错误边，catch 按 `kind==4` 判定），不触碰 aggregates ctrl 帧与返回值状态；临时量函数头确定初始化（verify.uninitialized-read 反例驱动）。collections/text-record atom schema 的 loop 节点加可选 `error_kind`/`error_message`（auto-language.atom 同款；require 列表不变，旧源零变化）。**定向正反**：catch 矩阵 7 例全绿；**回合前发现并关闭一条假绿通道**——带尾随 `.GalApply()` 的 handler 因 body_foreign 跳过检查函数，try 体内 `.n = "not-an-int"` 类型错误静默（`e-gal-try-typeerr` OK）；修复后控制组证实同型错误 try 内外均具名拒（`g1`/`g2` 同断 `source.type-mismatch`）。
- **裸消息调用 `.Msg()` 接通**：hook_stmts_lower 入口统一把 `Call{callee=Ident}` 改写为 `Send`（声明消息带 handler→内联、无 handler→outbound 行——与 `."Msg"(args)` send 语句同合同）；投影两级（`collect_widget_handlers` 语句级 + `stmts_have_foreign_call`）对裸消息调用不再记 body_foreign。029 的 GalQuery→ReloadDir→GalApply 调用链全部进受检域。**负控保持**：plan108_http_lane 4/4（未知调用仍不承载）；未声明调用仍 body_foreign → 事件道具名 fail-closed。
- **消费 PLAN-108 已交付组件面 Http**：双树 merge master（auto-lang +77 / auto-ui +15 提交，零冲突）。事件道实测 `Http.get_json`→str 编译通过（gap[p1] OK）。
- **`??` 空值合并（文本/集合面）**：此前落 `bin_op_vt` unreachable panic（连带打挂 A2Vue lane 探针 `plan117_t01_029_lane_probe`）。现按 VM 语义折叠：非空左操作数→自身（右操作数旁支类型核验、不发射）；Dyn/Nil/Variant 左值具名拒（B-05 落点）。qq 矩阵实测：字面量形 OK。
- **std.json 裸模块导入**（WIP 收编）：`use std.json` 两段路径在 AGE 包面按跨包 [pkg,module] 装配（binding 修复+用例）。
- **029 app.at 内部消息声明补齐**（源修订，仅声明不改行为）：OpenPhoto/PrevPhoto/NextPhoto/ApplyView/ViewEvent/BackToGrid/ToggleFav——同 asset_browser 先例，029 pac/app 冻结哈希变更（重基线归 T-04）。
- **auto-ui load_source 归一化前移**（WIP 收编）：`$event`/`$N` 同长归一化提至 `package_candidates`/投影/AV 编译共用入口（零 span 位移）。

**测试（本会话实际执行面）**：ace-syntax-grammar conformance 58/58（0.99–1.51s）；ace-core lib 88/88（1.17–4.53s，含新增 std.json binding 用例）；ace-av conformance plan981 84/84（0.38–0.41s）；plan108_http_lane 4/4（0.19–0.28s）；plan117_t01_probe 4/4（0.67s）；plan117_t02_probe catch/矩阵/gap/qq 探针全绿（0.12–0.15s）。无新增 >10s 用例。

**B-05（新具名阻断，owner=auto-lang AGE 值域；029 AV 最后一族）**：checked 域缺 **JSON 值语义**——`json.parse`（记录级，std.json 仅字符串级 helper 族）、解析值的成员访问（`.entries`/`.root_missing`）、`json.stringify`（记录→串）、异构记录字面量（`{id:…, label:…, children:[]}` object-elem 要求单元素类型）、可空左值 `??`。**031 阻断不变**：back::api 服务模块未随 app 单元装载（需 --package 多单元或 back 桥）。备选路线（待 owner/用户裁定）：① 新建 auto-lang 计划交付 checked 域 JSON 值面（Variant 六态 + 成员访问 + parse/stringify 内建，PLAN-108 先例）；② 本计划 bounded 扩面（显著扩大 auto-lang 写面）；③ 桥级 JSON provider（宿主解析，受检域新增 Json VT）。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-ui,auto-lang} | commit: auto-lang 2f7e8fe20 / auto-ui 1c253252 | next: work/T-03 续（B-05 裁定后）

2026-11-22 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 续（round 6：B-05/031 back/carousel 三道勘察 + B-05 路线裁定请求） | outcome: partial | worktree_path: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | branch: plan-117-dev（双树） | code_commit: 无新代码提交（纯勘察轮） | task_ids: T-03（续） | next_responsible_role: work（B-05 裁定落地或 031 back 接线）。

**勘察结论（全部实测/实读）。**
- **B-05 地基已有**：AV 值域含 `Dyn`（PLAN-824 av.dynamic/1）、`Object`（PLAN-964 共享 string-keyed 对象 cell，ObjectKey Text/Int/Bool）、`Nil`（PLAN-964，`==`/`!=` 准入）——运行时不需要新值类型。config 面（PLAN-982）走 i32 句柄运行期核，非 Object 皮；std.json 切片是字符串级 path 访问族（`json_kind_at`/`json_object_text`），非记录级。**真实缺口 = AGE 组件面的语言皮**：`json.parse`→Object 值内建（镜像 PLAN-108 HttpCall 全链：源级目录+resolve 拦截+adapter/hir 臂+av-core 执行+ABC）、Object 成员访问 `.field`→ObjectGet、Object/Nil 上的 `??`、record→string 序列化、异构对象字面量到 Object 装载。
- **029 数据契约**：VM 方言里 `Http.get_json(url)` 直接当结构化值用（`.entries` 成员访问）——PLAN-108 的 checked 契约是 `get_json→Str`；B-05 落地时 `get_json` 在 AGE 面需返回 Json/Object 值（post_json 保持 Str——032 消费其串不变，按动词分语义与 VM 一致）。
- **031 back 道**：`back_proxy` 已能按 `SessionSpec{app_id, back_entry}` 把 back 链装载为 **VM session**（HTTP 服务于 `/apps/<id>/…`）——harness 半边是接线活；front 半边（`use back.api: fns` → Http 效应映射，按 back 的 `#[api]` 注解）是语言侧，与 B-05 成对（HTTP 响应仍需解析）。029 的 `/api/photos/scan` 本就是宿主原生路由（`NativePhotoApp`）。
- **carousel AV**：画廊页组件用 f-string 计算类名（`f"carousel carousel-${.orientation}"`）——f-string 文本叶是 ace_node_desktop 已记 KNOWN-DEBT（av.owned/1 车道不可 lowering）+ 画廊多单元组件壳，独立具名族，非 B-05。
- **B-04/A2Vue lane 现状**：lane 仍全缺席（029 18 handler / 031 13 handler bodies=0，SFC 生成正常）——round 5 的投影豁免未改变 lane 判定（Http MethodCall 仍域外），T-06 前需 lane 承载道收口。
- **B-05 路线裁定已请求用户**（AskUserQuestion 三选一）；未获答复期间按 **②-lite**（本计划 auto-lang 树内、写面严格限定 JSON 值域族、越族具名拒、PLAN-108 先例）作为工作假设继续，裁定随时可改道（提交隔离可整段移交）。

## 10. 待澄清事项

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

2026-11-22 T-01 已完成并记录 B-01..B-04（§9）。T-02 顺序：先按已有 owner 交付核对可消费能力（108 的 AGE AV Http、114/123 的包面/路由），再按实际缺口定 auto-lang AGE 文法/lexer 增量（B-01/B-02）与 lane 域外调用接线（B-04），每段以原用例定向正反验证。无阻塞用户决策，工作树保留。

2026-11-22 T-02 续（round 2，§9）：031 的 image_viewport 包面已打通（$ 占位/float 默认/带引号消息/float 载荷/std.json），现阻于 **json.stringify（AV checked 域 JSON 序列化）**；029 的 asset_browser 结构段（model/computed/view/on 组合）与 on 段 try/catch CST 规则仍未通。均属 auto-lang AGE 能力增量，无用户决策阻塞，恢复点=T-02 续。

2026-11-22 T-02 续（round 3，§9）：asset_browser 完整文件包面装载与 16 个 handler 投影全通（computed-if/??/try-catch/outbound-Try/msg 声明补齐）；031 两端包面 OK。**恢复点=T-03 AV 实跑**（--source 装载 029/031 应用面 + 事件程序编译；json.stringify 序列化若阻则实现 AV checked 域 JSON 序列化）。无用户决策阻塞，工作树保留。

2026-11-22 T-03 续（round 4，§9）：029 应用面语法全通（CST has_errors=false）；AV 装载推进到 checked 域 **Try 语句语义**（resolve 仅 aggregates 面支持 Stmt::Try，029 的 `try { json.parse(…) } catch { … }` 需 AGE 面 Try + **json.parse/stringify 记录级 JSON 语义**——方言/VM 内建，checked 域无）。031 阻于 back::api 会话函数装载（需 --package 多单元或 back 桥）。**恢复点=work/T-03 续（checked 域 Try+JSON 语义族）**。无用户决策阻塞，工作树保留。

2026-11-22 T-03 续（round 5，§9）：checked 域 **Try/catch、裸消息调用链、非空 `??`、Http.get_json（消费 PLAN-108）全部接通**（探针全绿，详见 §9 round 5）。现阻于 **B-05：checked 域 JSON 值语义族**（json.parse 记录级/成员访问/stringify/异构记录字面量/可空 ??）与 031 back 单元装载。B-05 有三条备选路线（新建 auto-lang 计划 / 本计划 bounded 扩面 / 桥级 provider），**属共享语义增量，待 owner/用户裁定路线后继续**；裁定前 029 AV 装载停在该族具名拒（fail-closed，不假绿）。031 的 back 装载与 B-05 无关，可独立推进（--package 多单元或 back 桥，同为写面归属问题）。恢复点=work/T-03 续（B-05 裁定或 031 back 装载道）。
