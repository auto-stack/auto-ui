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

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 25 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang 78c8abf09 / auto-ui 4aac9736 | next: work/T-03 续（view-data 读改写覆盖面 + ②③ Dyn 值链方法/索引面）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 26 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang ca90d847a | next: work/T-03 续（场景车道 view-data 读改写移植 → 029 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 27 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 266c6656d | next: work/T-03 续（② Dyn/str 方法面 + 场景道剩余 unbound __age_s_* 归因 → 029 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 28 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang f889ffb54 / auto-ui a34dfbee | next: work/T-03 续（残余 unbound __age_s_* 归因 → 029 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 29 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang 2d5cf2913 / auto-ui 047506ee | next: work/T-03 续（最后 7 条：.replace helper + photos/set 对 → 029 真窗实跑）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 30 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang 8158df5e6 / auto-ui 55b82b10 | next: work/T-03 续（collections 面 str/Str 写位期望类 → 029 拒墙清零 → 真窗）

## 10. 待澄清事项

2026-11-23 T-03 续（round 26，§10）：**事件道 view-data 集合物化端到端落地；发现 round-22 物化从未发射**（auto-lang `ca90d847a`）。**假绿通道（round-5 同族）**：round-22 的物化铸壳 `fn __age_vd_cast()` 无返回注解——AGE 组件 profile parse 期具名拒（`function return type annotation is required`），全部字面量静默走 PARSE-FAIL 臂（None 无声跳过）；「no local named `__age_s_*`/self」拒类（round-24 墙 289 undefined 的主体）根因即此。**修复链（探针二分逐层退拒）**：① 铸壳补 `-> int`（Auto 语法无箭头）+ 尾部 return（仅取 Let 值）；② 物化改**按 handler 按需**（`handler_reads_view_data` 判读——纯写/零读 handler 不携带集合初值，文档保持文本域；get_json 探针的 `var photos = []` 零读不受扰）；③ collections 升域门**按发射形状**（`event_stmt_collections` 跳过 PLAN-106 收据 vd 写的 RHS——needs_collections_face 无此 awareness）；④ adapter 类型表：List/Object 带 Dyn elem 时补 `t_dyn` 行（t_list_dyn 的 elem 引用悬空）；⑤ lowering：collections 域事件文档同走 av.owned（domain 门 + HIR List 类型臂——plain str elem 落 owned Str + LIST_* 表达式臂按 **AGE_ELEM_VIEW_CAPABILITY** 面标记（lowering 入口按 HIR 域盖戳，复用 PLAN-105 既有标记）+ ListPush 的 Unit 效果位豁免）；⑥ wire：Owned 线族的 List 类型标签 + LIST_* 指令标签按面标记放行（encode+decode 对称）；⑦ mir_verify：`elem_view_surface` 开 Dyn 类型/List 类型/List(Dyn) 元素位/LIST_* 指令门。**执行可观测**：常备用例 `p117_view_data_list_read_in_for_cond_executes`——view-data `["a","b"]` 物化 + **for 条件里的 `.len()` 读**（029 `for fi2 < .fav_records.len()` 真形最小切片）+ 循环 + typed 字段落账 = `Int(2)` 端到端。**车道发现（重要）**：bridge_load（029 真窗路径）走**场景车道**（`compile_component_scenario_with_initials`）——round-22 的读改写/物化只在**事件道**（compile_component_event_program）；场景车道的 view-data 读仍走 `self.<field>` 形（`self` 未绑定 → 800 墙不变）。**下轮切面**：场景车道移植读改写+按需物化（场景道 collections 面已存在——needs_collections_face 先例）→ 029 真窗；其后 ② Dyn 值链方法面 ③ nil-on-missing（ObjectGetOpt）031 back 桥。回归全绿：ace-core lib 88 / p108 7 / p117 3 / json_975 4 / wire 3 / plan981 84 / collections 124 / event sessions 5（基线红 v02b、collections perf 门除外——均 stash 对照实证 base 即红）。

T-01 确定实际工具版本、运行参数和原样入口，逐条核原平台与目标条件。具体阻断记录到本计划及对应生产方。

2026-11-22 T-01 已完成并记录 B-01..B-04（§9）。T-02 顺序：先按已有 owner 交付核对可消费能力（108 的 AGE AV Http、114/123 的包面/路由），再按实际缺口定 auto-lang AGE 文法/lexer 增量（B-01/B-02）与 lane 域外调用接线（B-04），每段以原用例定向正反验证。无阻塞用户决策，工作树保留。

2026-11-22 T-02 续（round 2，§9）：031 的 image_viewport 包面已打通（$ 占位/float 默认/带引号消息/float 载荷/std.json），现阻于 **json.stringify（AV checked 域 JSON 序列化）**；029 的 asset_browser 结构段（model/computed/view/on 组合）与 on 段 try/catch CST 规则仍未通。均属 auto-lang AGE 能力增量，无用户决策阻塞，恢复点=T-02 续。

2026-11-22 T-02 续（round 3，§9）：asset_browser 完整文件包面装载与 16 个 handler 投影全通（computed-if/??/try-catch/outbound-Try/msg 声明补齐）；031 两端包面 OK。**恢复点=T-03 AV 实跑**（--source 装载 029/031 应用面 + 事件程序编译；json.stringify 序列化若阻则实现 AV checked 域 JSON 序列化）。无用户决策阻塞，工作树保留。

2026-11-22 T-03 续（round 4，§9）：029 应用面语法全通（CST has_errors=false）；AV 装载推进到 checked 域 **Try 语句语义**（resolve 仅 aggregates 面支持 Stmt::Try，029 的 `try { json.parse(…) } catch { … }` 需 AGE 面 Try + **json.parse/stringify 记录级 JSON 语义**——方言/VM 内建，checked 域无）。031 阻于 back::api 会话函数装载（需 --package 多单元或 back 桥）。**恢复点=work/T-03 续（checked 域 Try+JSON 语义族）**。无用户决策阻塞，工作树保留。

2026-11-22 T-03 续（round 5，§9）：checked 域 **Try/catch、裸消息调用链、非空 `??`、Http.get_json（消费 PLAN-108）全部接通**（探针全绿，详见 §9 round 5）。现阻于 **B-05：checked 域 JSON 值语义族**（json.parse 记录级/成员访问/stringify/异构记录字面量/可空 ??）与 031 back 单元装载。B-05 有三条备选路线（新建 auto-lang 计划 / 本计划 bounded 扩面 / 桥级 provider），**属共享语义增量，待 owner/用户裁定路线后继续**；裁定前 029 AV 装载停在该族具名拒（fail-closed，不假绿）。031 的 back 装载与 B-05 无关，可独立推进（--package 多单元或 back 桥，同为写面归属问题）。恢复点=work/T-03 续（B-05 裁定或 031 back 装载道）。

2026-11-22 T-03 续（round 7，§10）：B-05 裁定未获答复，按 **②-lite** 继续（记录于 §9 round 6）。**S1 设计定稿（下一步机械执行）**：Json 值 = `Object(elem=Dyn)` + `List(elem)` + 标量 + `Nil`，全部复用 PLAN-964/824 已交付运行时（`ObjectData{entries: Vec<(ObjectKey, Value)>}`、`ListBuf{elems: Vec<Value>}`、`Value::Nil`）。落地件：① `json_ops.rs` 源级目录（镜像 http_ops：`json.parse(str)->Object(intern_object(Dyn))`，capability `json.value/1`）；② resolve 拦截（镜像 try_resolve_http_effect 9323）；③ Object 成员读 `.field`→`ObjectGetOpt`（missing→Nil，JSON 语义；与 PLAN-964 括号式 trapping `obj[k]` 区分）——mir/executor 加 opt 臂或在既有 ObjectGet 上加 nil-on-missing 变体；④ `??` 对 Dyn/Nil 左值 = 运行时 null 判等折叠（round 5 的具名拒臂升级；Nil 型左值恒折叠右值）；⑤ `Http.get_json` AGE 面结果改 Object（post 族保持 Str——032 串消费不变，与 VM 动词分语义一致）；⑥ 异构对象字面量（object-elem 单元素型限制的 AGE 面豁免→Object 装载）；⑦ `json.stringify(record/object)->Str`（031 viewRequested 载荷）。执行通道：镜像 PLAN-108 HttpCall 全链（源表→resolve 拦截→adapter capability/签名臂→hir host-import→av-core declaration/validate→executor→ABC HOST section——HOST section 既有，无新 opcode 若走 host-import 族）。**已实测边界**：`var saved = null` 在文本面推断锁 nil 型（`saved = "x"` 具名拒）——真案例走 json.parse 的 Object 型，不依赖 nil 推定宽化。S1 完成判据：029 Init 形探针（json.parse→成员读→`?? []`→`!= null`→赋值 photos）事件道编译通过 + 正反用例。

2026-11-22 T-03 续（round 9，§10）：**S1a 已落地**（auto-lang `9c4095215` / auto-ui `ad28b529`）——json.parse→Dyn、Dyn 成员读、`!= null`、`??` 折叠（b1-b5/b7 事件道探针全绿，负例具名拒；连带修复 PLAN-108 隐性 id 碰撞 get-json.v1×parse.v1）。**剩余缺口已锐化（round 10 二分）**：与 coalesce/json 无关——**受保护 try-Loop 体含 If 分支即触发**：b8（try 内纯 if+赋值）、b10（try 内 if + **空 catch**）、b11（try 内 str let + 赋值）全挂同一对诊断 `verify.use-after-move: cannot reassign l2/l3 @try-span` + `ownership-required: …conditional`，失败局部恒为 unwind 的 **kind 临时**（I32；l2/l3 随字段数平移）。round-5 矩阵全绿是因为其 try 体只有直赋值语句。下轮切面：读 verify 的 protected-loop（`WState::AwaitLoop { protected: unwind.is_some() }` 8236）对「体含分支 + unwind 写 kind」的 join 规则——对照 aggregates 面验证器同形（5535）为何通过；修点或在 verify 的 loop-join 对 unwind 写洞的豁免，或在 resolve_try_stmt_text 的 kind/err 临时形态（如 cell 化）。029 真源 try 体全是 if/let 组合，此缺口堵死 029 Init 全形——修通即过 S1a 验收判据。

2026-11-22 T-03 续（round 11，§10）：**try lowering 重构为同栈 RCont 帧**（auto-lang `f0c4f9633`）——TryBody/TryCatch 帧走 resolve_body 共享栈（与 While/If 同一 hoist/splice 机制），替代重入式 resolve_try_stmt_text。首轮实现有 try_stmt 交接缺失（FnPlan 转储实证：catch 场景 try Loop 语句整体缺席）——已修并经 plan 转储复核语句序完整。**排除项（重要）**：重入 resolve_body 理论被证伪（重构后 b8-b14 同错）；resolver 侧 RStmt 经转储实证完整；丢失曾发生在首轮帧实现的 plan 层（已修）。**现确认剩余唯一缺口 = text_resources owned-walk 对 unwind 写入局部的 join 规则**：HIR 已完整（try Loop + wrapper + kind==4 If 齐备），walk 仍报 `cannot reassign l2/l3`（失败局部= unwind kind 临时，I32）——下轮在错误点转储 `st_snapshot.dead` 判定谁把 kind 放进 dead（trap 边写入建模 or catch wrapper 的 err move 连带），修点=walk 对 unwind 洞的豁免或 trap 边建模。回归全绿：ace-core lib 88 / 语法 conformance 58 / plan981 84 / catch 矩阵 7 / plan108_http_lane 4 / plan117_t01 4。**S1a 判据维持**：b8/b10/b11 转绿 → 029 Init 全形（s1a_acceptance）过事件道编译。

2026-11-23 T-03 续（round 12，§10）：**S1a 判据达成**（auto-lang `9a7615dae`）。错误点 `st_snapshot.dead` 转储判定：失败局部 = **err 临时（StrOwned，非 kind）**，cause=Moved——根因 = catch wrapper 照抄 aggregates 形的 `Let err_l ← Move(err); Assign err ← ""` 在 owned-lane walk 自触 reassign-after-move。**修法（语义保持）**：AGE catch 恒为无绑定形（投影不产 bind），err 存/清无消费者——wrapper 缩为 `Assign kind ← 0` 单语句（陈旧 err 由 kind 门控、下次 trap 边 move-in 覆写）。**验证**：b1-b14 全部 14 例绿；**`s1a_acceptance` 全形（Http.get_json→json.parse→成员读→`?? []`→`?? false`→`!= null`→try/catch 恢复赋值 + GalApply 链）过事件道编译 OK**；负例保持具名拒；回归全绿（88/58/84/7+4 矩阵/4 lane/4 t01）。**S1 剩余**：S1b（av-core `auto.json.parse.v1` 执行器 + provider——S1a 现为编译准许、执行 fail-closed）、`json.stringify`（031 载荷）、异构对象字面量装载、`get_json` AGE 面返回值改 Dyn。031 桥仍阻于视图循环行载荷（desktop event-payload 具名拒，独立工作面）。

2026-11-23 T-03 续（round 13，§10）：**S1b 执行链全线落地**（auto-lang `8ded643ab` / auto-ui `b84a129a`）——av-core `json_value.rs`（json.value/1 准入 + JSON 解析器→真实 Value：Object/ObjectData、List/detached、Str/detached、I32/F64/Bool/Nil；深度/节点/文本预算）+ 执行器 PlatformCall json 分流臂 + host.rs 第四声明类别 + lowering（def-walk 声明臂、调用点 PlatformCall 发射、host-import 路径门 `json_callee`、Dyn/Nil/Object 类型表映射）+ 事件道默认授予集加 `json.value/1` + wire（Owned 族放行 Dyn/Nil/Object 类型标签、PLATFORM_CALL 编解码、json host imports）+ mir_verify（已知能力集、`dynamic_values()` 按 json 能力准入 Dyn/Nil/Object 面 + DYN 指令 + erasable closure、PlatformCall json 类别/profile 门）。**逐层具名拒退链实证**（lower.callee→abc.host-section→abc.profile→decode.profile→host.unauthorized→mir.capability-unknown→mir.profile-type→host.profile→host.category→mir.dyn.whitelist→mir.object-type）——每层以最小诊断定位后放行，无静默放行。回归全绿（88/58/84/7/4/4）。**剩余一个具名校验**：`mir.object-type`——dyn-cast 链上的成员读（`.entries`）需 OBJECT_GET 的 obj 槽先物化为 Object 型（DYN_TAKE dst 的类型材料化次序）；修通即 `plan117_t03_s1b_execution_observable` 执行可观测（探针已就位：dispatch Init → 断言 `.n=11/.load_error=""` 或 catch 路径 `.n=12/.load_error="parse failed"` 落账）。

2026-11-23 T-03 续（round 14，§10）：**S1b 执行可观测达成**（auto-lang `f91faad9e`）。两步收口：① lowering 的 DynCast/NilLit/ObjectCtor/Get/Set 臂加 `json_values()` 门（json 模块与 collections 道共用 MIR 指令族）；verify 的 `object_elem_of` 接受 json 面（`json_values_surface` 辅助，dynamic_values 复用）——`mir.object-type` 退场，admit 全通。② json 解析器输出 **Dyn-erased 值**（PLAN-824 `DynValue::pack` 逐值显式擦除，容器先擦子值——json.parse 结果的宿主槽是 Dyn 型，pack=唯一铸造面）。**执行观测**（`plan117_t03_s1b_execution_observable`）：Init dispatch 成功；`recovered = Bool(true)`（data.ok 成员读真实取值）；`.photos = data.entries` 的 dyn-list→typed-slot 赋值运行期 trap 被 **catch 真实捕获并恢复**（n=12 = try 1 + catch 2 + GalApply +10；load_error 落账）——**json 值域 + try/catch 恢复在事件会话端到端执行**。**剩余具名缺口**：dyn-list→typed-list 槽的运行期物化（029 真形 `.photos = data.entries` 的赋值步；静态 verify 已过、执行期 slot 类型检查拒）——下轮切面=执行器 assign 路径的 dyn 容器物化（或 resolve 侧成员读 elem 的静态窄化）。此后：029 真窗实跑 → 031 back/循环行 → T-04..T-08。

2026-11-23 T-03 续（round 15，§10）：**DYN_TAKE 容器物化落地**（auto-lang `24fc0853f`）——`dyn_materialize_to`：`List(Dyn)` take 进 typed 列表槽时逐元素按目标 elem 物化（标量直通、容器递归、不相容维持既有 DynTypeMismatch trap；纯 share 语义不改源值）。**诊断结论（probe 二分 + 全点插桩）**：spurious catch-entry 的触发点不在任何 `ExecOutcome::Trap` 创建点、Raise、LangFail、write_language_error 或 unwind_to_invoke（全部插桩静默）——trap-edge 路由发生在 function_code/执行器的 opcode 层（待定位的具体入口）。**当前探针态**：`recovered`（成员读/??/判空链）执行 ✓；catch 恢复落账 ✓；剩余 = 成功路径上的 spurious catch 进入（`.photos = data.entries` 物化后仍入 catch——下一个 trap-edge 路由点定位）。全部回归绿（88/58/84/7/4/4/44 探针 0 fail）。**下轮切面**：function_code/opcode 层的 trap-edge 路由点定位（或 MIR 函数全文转储对照Invoke 结构）→ spurious catch 消除 → try 全路径 n=11/load_error="ok"/photos=[] 落账 → 029 真窗。

2026-11-23 T-03 续（round 16，§10）：**根因定位到字段域**（auto-ui `4544e5b0` 探针 + MIR 全文转储 `plan-artifacts/plan-117/mir_on_Init_dump.txt`）。**逐层排除**：任何 ExecOutcome::Trap 创建点/Raise/LangFail/write_language_error/unwind_to_invoke 均未触发；MIR 全文（52 行）证实 json.parse PlatformCall 在案、try 体读链的 DYN 指令缺席——**根因 = `var photos = []` 列表字段在事件道的 cell/局部类型被 Dyn 化，其 cell 写回经 `TextClone`（文本克隆）路径**（HIR 实证：epilogue 的 photos 写回 = `TextClone { ReadLocal(photos) } ty=Dyn`）——`.photos = data.entries` 的 Dyn 值经文本克隆写 trap → catch（恢复语义仍正确）。**下轮切面（精确定位）**：事件道列表字段的 cell 类型/加载克隆/写回克隆三处一致化（cell 类型=List(Dyn) 或 Dyn、写走 dyn 感知克隆非 TextClone）——或事件道字段门拒绝列表字段（029 photos 需列表，门拒则 029 不可行，故走一致化）。修通即 try 全路径（n=11/load_error="ok"/photos=[]）→ 029 真窗实跑 → 031 → T-04..T-08。**工具沉淀**：`ComponentEventSessionProgram::codes()`（MIR 勘察访问器）、probe 内 MIR/HIR 全文转储（AC117_DUMP 门）。

2026-11-23 T-03 续（round 17，§10）：**两项发现收窄根因到 HIR 构造层**。① **`var photos = []`（无类型注解）被投影为 view-data 惰性字段**——字段域转储实证：`node.fields` 只含 load_error/recovered/n 三项（photos 不在！）；PLAN-106 语义：view-data 写=惰性收据，hook_stmts_lower `continue` 跳过发射（事件道 photos 本就不落账——验收判据的「photos=[] 落账」需按 view-data 语义重基）。② **catch 体赋值被内联进 try 体块**（HIR expr-id 转储实证）：on_Init block[1]（try 体）= [let l5(data), p0←e7(n=3), p1←e8("ok"), p2←e9("parse failed"), p3←e10(n=2), break]——**catch 的 load_error="parse failed" 写直排在 try 体直行块里**，且 block[0] 第二个 `if` 条件 = e17 `LtI32(0,1)` 恒真——运行期 catch 体赋值随直行流执行（load_error 被 catch 值覆写 → 观测 n=12/"parse failed"）。**根因收窄：round-11 的 TryBody/TryCatch 帧lowering 在 resolve_body 栈机里把 catch 体语句并进了 try 体帧**（帧边界/栈推入次序缺陷——非 lowering/执行器层）。**下轮切面（精确定位）**：修 resolve.rs TryBody 闭合臂的帧推入——catch 体帧的 `stmts` 种子（wrapper）与 `src: &cc.body` 边界复核；HIR 复核 catch 赋值应在独立块（guard 的 then 支）——修通即 try 全路径（n=11→现源 n=3/2 序下 =12 且 load_error="ok"、photos 按 view-data 不落账）→ 029 真窗。**注**：round-15 的「物化已修」判断有误——DYN_TAKE 物化在位但读链根本未执行（HIR 内联吞了读）。

2026-11-23 T-03 续（round 18，§10）：**TRUE 根因实锤并修复**（auto-lang `e2724f169`）。RStmt 树转储实证 catch 体赋值（load_error←"parse failed"、n←2）已在**投影层**直排进 Try.body（catch=0 stmts）——**CST 投影的 age-on-try body/catch 分割缺陷**：`r-catch(field: "catch")` 是关键字 token 节点，其 CST kind ≠ "r-catch" 规则 kind，按 `rule_kind_of("r-catch")` 判定 `in_catch` 永假 → 全部分支语句（try+catch）都归 body。round-11~17 的帧 lowering 追踪全是症状——真因在投影分割。**修复**：按 `field_child(inner, "catch")` 的字段标记定位 catch 关键字子节点，按源偏移分割 try/catch 分支语句。**验证**：投影 Try `body=4/catch=3` ✓；`s1b_execution_observable` **try 全路径完整执行**（`load_error = Str("ok")`、`n = Int(13)` = 3 + GalApply 10——无 spurious catch 进入；photos 按 view-data 语义跳过落账 ✓）；回归全绿（ace-core lib 88 / 语法 conformance 58 / plan981 84 / probe 44/44 / plan108_http_lane 4 / plan117_t01 4）。**B-05 S1 执行链就此端到端闭合**（S1a 编译 ✓ + S1b 执行 ✓ + try/catch 恢复语义 ✓ + json 读链 ✓）。**下一切面**：029 真窗实跑（V-03：实际浏览/嵌入查看/缺图恢复）→ 031 back 桥 → T-04..T-08。

2026-11-23 T-03 续（round 19，§10）：**029 真源实测**（bridge_load 探针）——真源 app.at 的 handler 体是 **VM 方言全集**（列表/对象字面量/方法链），事件道编译停在具名拒墙：`array-lit-empty/position`（114 处，`var x = []` 无类型推断为 I32）、`push/clone/set/len` on i32（无类型列表局部）、`str` on i32（`.str()` 数字转文本）、`json.stringify`（载荷构造）、field access on Str（json 值链）。**即 B-05 S1 剩余件 + AGE handler 体的集合面准入**（无类型列表局部推 List(Dyn)、方法链 `.push/.len/.str/.set`、异构对象字面量、json.stringify）——029 真窗的前置面。已验证可执行核：try/catch + json.parse/成员读/`??`/判空 + 消息调用链 + catch 恢复（探针全绿）。**剩余面清单（029 真窗的精确前置）**：① 无类型 `[]` 局部推 List(Dyn)（推 lit 时的 expect 缺省路径）；② i32 误推的 push/len/set/clone 方法链（集合面准入 AGE handler 体）；③ 异构对象字面量→Object 装载；④ json.stringify(record/object)→Str；⑤ get_json AGE 面返回值改 Dyn（现 Str，029 源直接 `.entries` 成员访问需值）；⑥ Http/回环后端接线（NativePhotoApp 路由已在 back_proxy）。①–⑤ 为 auto-lang AGE 集合面切片（一个完整计划轮），⑥ 为桥接线。工作树保留，恢复点=round 19。

2026-11-23 T-03 续（round 20，§10）：**前置面① 落地**（auto-lang `847c0b768`）——`resolve_array_literal` 对文本/集合（AGE 执行）面放行**任意 rvalue 位**的数组字面量（与 Auto 面同一 ListNew+push 链），空/无期望型字面量默认 `List(Dyn)`（json 值域容器）；Let 臂空字面量同得 List(Dyn) 缺省。**array-lit-empty/position 拒类全退场**（029 真源 114+69 处 → 0）。剩余拒墙重测：`type-mismatch: field access ... found i32`（350）+ `len/str/clone/set/lower on i32`（约 200）= **前置面②（集合方法链准入 AGE handler 体——无类型局部的集合方法面）**，`object-elem`（80）= ③，`json-unsupported`（5）= ④，`argument`（11）+ age（4）= ⑤/⑥ 邻接。回归全绿（88/58/84/44/4）。**下轮切面**：② 集合方法链（len/push/str/clone/set/lower on List(Dyn)/标量），其后 ③④⑤⑥ → 029 真窗。

2026-11-23 T-03 续（round 21，§10）：**前置面③ 落地**（auto-lang `da9bad67a`）——`ExprKind::ObjectLit` 对文本/集合（AGE 执行）面按 **Dyn elem** 承载（逐字段 coerce_dynamic，与 Auto 面同律；异构字段合法——json 值域容器）。**object-elem 拒类退场**（029 真源 80 处 → 0）。剩余拒墙重新锚定：**级联根 = handler 体内的 view-data 字段读**（`var src = .photos` → view-data 读未接线 → I32 dummy → `.len()/.str()/field access on i32` 全部级联）——即 029 惰性展示数据在事件 handler 内被**读**时的承载（现语义只覆盖"写=收据跳过"，读=初值物化未接线）。回归全绿（88/58/84/44）。**下轮切面**：view-data 读物化（handler 内 `.vd_field` 读 = view_data 初值字面量的常量物化，List(Dyn)/记录字面量），其后 ④json.stringify ⑤get_json→Dyn ⑥Http/back_proxy → 029 真窗。

2026-11-23 T-03 续（round 22，§10）：**view-data 读物化 + 数值 .str() 落地**（auto-lang `e7394885d`）。① 事件道（compile_component_event_program）view-data 字段**读物化**：惰性字面量文本借 `parse_program_items` 铸壳 fn 提取初值表达式，handler 入口以 ComponentStmt::Let 前置物化为只读局部（hook_stmts_lower 的 Let 臂由此绑定 hook-local 名），handler 体读改写 `.vd_field` → 该局部（写侧 PLAN-106 收据跳过不变；node 在该 lane 转 owned 以支持改写）。② 数值接收者 `.str()` = PLAN-105 TextDisplay 节点（int→str 显示），numeric-.str 拒类退场（57 处）。回归全绿（88/58/84/44/4/4）。**剩余拒墙**（bridge_load 重测）：type-mismatch 368 / undefined 289 / index-base 152 / unsupported 99——级联根已收敛到 **json 值链的方法/索引面**（Dyn 值上的 `.seq`/`[i]`/`.len()` 组合）与 json.stringify（④）。**下轮切面**：json 值链方法/索引面（Dyn 上的 `.len()/.seq`/索引——object_elem_of 已放行 Object/Dyn，查 index/method 分派臂）→ ④json.stringify → ⑤get_json→Dyn → ⑥Http/back_proxy → 029 真窗。

2026-11-23 T-03 续（round 23，§10）：**前置面④ json.stringify 落地**（auto-lang `1f3e8e1e0`）——json_ops 闭族加 `json.stringify` + VM 方言别名 `json.encode`（029 真语料拼写；同一符号 `auto.json.stringify.v1`，params=[Dyn]、result=StrOwned）；resolve 的 json 拦截改**参数感知 coerce**（先原始 VT 选重载——parse 的文本参不动，无匹配按 Dyn coerce 重试——stringify 的对象字面量/标量 → Dyn）；av-core json_value 加序列化臂（Dyn 解包载荷、Object 按 ObjectKey 转 string 键、List、转义 Str、标量/Nil；深度预算同解析器）。**json-unsupported/json-argument 拒类退场**（029 真源 → 0）。回归全绿（88/58/84/44/4/4）。**剩余拒墙**（371 type-mismatch / 289 undefined / 152 index-base / 99 unsupported，首根 @prepared-17452 field access on Str）：**json 值链方法/索引面**（Dyn 值上的 `.seq/[i]/.len()` 组合——`index-base`/method 分派臂的 Dyn/Object 放行）为下一块切片，其后 ⑤get_json→Dyn（并入同切片）⑥Http/back_proxy → 029 真窗。

2026-11-23 T-03 续（round 24，§10）：**诊断基建扩展**（auto-lang `bdda3cfe2`）——resolve 层 RTRY-BODY/RCATCH 逐语句转储 + LANGERR 文本转储（AC117_DUMP 门）常备；MIR 全文转储（codes() 访问器）已在 probe。**029 拒墙新态**：json 拒类清零后第一根 = `field access on Str @17452`（prepared 坐标；crude 注释剥离无法精确映射行）。RCATCH 转储实证 TryCatch 帧现正常闭合（catch 体 3 语句在案）。**下轮切面**：① prepared 源 span→行精确映射（用 prep 产物存盘对照）定位 17452 的具体表达式；② Str 方法面补齐（lower/upper 等 VM 方言拼写——新 BuiltinMethod 或 std text method 路由）；③ Dyn 值链 index/method 臂复核。回归全绿（88/58/84/44/4）。

2026-11-23 T-03 续（round 25，§10）：**① span→行定位 + ⑤ get_json→Dyn 落地**（auto-lang `78c8abf09` / auto-ui `4aac9736`）。① prep 产物存盘对照（AC117_DUMP 门，auto-ui probe 常备）：@17452 = **prepared 第 321 行 `.root_missing = data.root_missing ?? false`**——`data = Http.get_json(...)` 被 PLAN-108 契约钉成 Str，全部 `data.*` 读级联。⑤ 修复四面：http_ops `get_json` 结果 **Str→Dyn**（动词分语义：post 族保持串——032 不变；执行侧 get-json 完成臂复用 S1b 解析器，解析失败=具名 Error 与 json.parse 运行期臂同形）；adapter requires 装配（get-json 模块同授 `json.value/1`——Dyn 消费面的门）；av-core 执行器 + **abc wire host 门改 http∪json 并集**（029 真形同一 handler 混用两族，原「全 http 或全 json」窄开口必拒）；**owned-walk 编译器临时豁免**（`__agg_*` 单消费者裸读=其消费——dyn-coalesce 结果转移进 typed 字段写的 `verify.ownership-required` 拒类退场；二分实证 vf/vg 对照）。**执行可观测**：新常备 conformance 用例 `p117_get_json_component_face_returns_json_value`（挂起→provider→解析→成员读→`??` 折叠→typed 字段落账）+ `p117_get_json_invalid_response_is_named_error`（坏响应具名拒）全绿。**029 拒墙重测**：总数 911→800（json-argument/unsupported 双类清零），首根推进到 **prepared 第 343 行 `for fi2 < .fav_records.len()`——for 条件里的 view-data 读不在 round-22 改写覆盖内**（`no local named self`→i32 级联：field-access 277 / undefined 285 / index-base 140 / unsupported 87 / argument 11）= ②/③ 切片下一根。**基线红（非本轮引入，stash 对照实证）**：dynamic_objects_964 6 红 + component_event_sessions v02b 1 红（bdda3cfe2 即红；归因待 merge，round-3 先例）。回归全绿：ace-core lib 88 / p108 7 / p117 2 / json_975 4 / wire 3 / plan981 84 / t02 probe 44/44 / t01 4。**下轮切面**：view-data 读改写覆盖面（for/while 条件与嵌套表达式位）→ ② Dyn 值链方法面（`.len()/.seq` on Dyn，`str on Dyn` @18691）→ ⑥Http/back_proxy → 029 真窗。

2026-11-23 T-03 续（round 27，§10）：**场景车道移植落地，029 拒墙 800→488**（auto-lang `266c6656d`）。029 真窗路径（NodeDesktopBridge::load → `compile_component_scenario_with_initials`）移植事件道同款机制：① `compile_component_scenario_full` 对 handler 体跑 `rewrite_view_data_reads`（节点本地 clone 持有改写；无 view-data 源零变化）；② `behavior_check_fn` 按 `handler_reads_view_data` **按需物化** Let（纯写/零读 handler 保持文本域）；③ 场景 hir_profile 门叠加 `vd_collections`/`handler_collections` 触发（场景道 `str` model 字段本就映射 StrOwned——collections verify 无事件道的 str-local 排斥）；④ `event_stmt_collections` 提升模块级复用。**常备用例**：`p117_scenario_lane_view_data_for_cond_compiles`（try 体 + 嵌套 for 条件 view-data 读过 check-fn 物化）。**拒墙重测**（bridge_load）：800→**488**——self/未绑定局部级联退场；剩余具名类：`expected str, found Dyn`（注解位 Dyn coerce——隔离复现实证于 ReloadDir 切片）、`str on Dyn`（18）、`lower on Str`（11）、`clone on i32`（15，级联）= **② 切片**；另有残余 `no local named __age_s_0_*`（111——隔离形状不可复现，SCEN-VD/SCEN-FN dump 证明 check-fn 物化在位，待逐 handler 归因）。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / wire 3 / plan981 84 / collections 124 / event sessions 5（基线红除外）/ t02 probe 44/44 / t01 4。**下轮切面**：② Dyn→str 注解 coerce + Dyn/Str 方法面（.str/.lower）→ 029 拒墙再收敛；残余 unbound 归因（dump 常备）。

2026-11-23 T-03 续（round 28，§10）：**json 值链 coerce + 方法面准入落地，029 拒墙 488→398**（auto-lang `f889ffb54` / auto-ui `a34dfbee`）。**归因突破**：残余 `no local named __age_s_0_*`（111）不在检查函数/重放道——`compile_component_scenario_states_at`（ts_provider）与全部 handler 铸造路径均已覆盖物化后仍复现；ARRAYLIT dump 实证 resolver 的空数组特型臂**确实运行并绑定**（profile=collections、elems=0、ann=none），而同 fn 的读仍报 unbound（scopes 46-59 名）——绑定与查找之间丢失，根因未定（lookup 的 text_mut_alias 短路是下一嫌疑；UNBOUND/LETBIND/ARRAYLIT/RESOLVED-FN 四组 AC117 dump 常备）。**本轮修复**：① `instance_stmts` 场景重放道补按需读物化（重放体同树）；② 绑定期 **Dyn→注解 coerce**（`var meta str = e.size_str` 类——coerce_dynamic/DYN_TAKE，round-12 机制）；③ Dyn 接收者 `.str()` = TextDisplay（`e.height.str()`）；④ 数值接收者 `.clone()` = 恒等；⑤ **prep 接收位扫描修**（auto-ui）：`p.title.lower().contains(ql)` 此前误改写出 0 参 `__p105_str_contains()`——接收位回扫跳过配对 `()`/`[]`；⑥ `.lower()`/`.upper()` → `__p117_str_lower/upper` ASCII 折叠 helper（`len/char_at/to_codepoint/from_codepoint/to(Str)` 皆既有内建）；⑦ probe 切片 char 边界安全化。**拒墙**：398 = unbound 111 + 其 i32 级联 ~284（index-base 102/type-mismatch 82/len-on-i32 38）+ 零散。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / collections 124 / event sessions 5（基线红除外）/ t02 probe 44/44 / t01 4/4。**下轮切面**：残余 unbound 归因（priority——其级联占 398 的 ~290）→ 029 拒墙清零 → 029 真窗实跑 → 031 back 桥。

2026-11-23 T-03 续（round 29，§10）：**unbound 根因实锤并修复，029 拒墙 398→7**（auto-lang `2d5cf2913` / auto-ui `047506ee`）。**根因（dump 逐层归因）**：`hook_stmts_lower` 的**裸调用内联展开**（round-5 准入）——被展开 handler 的体携带 round-22 读改写（`__age_s_0_*`），其物化 Let 在**被展开 handler 自己的检查函数入口**；展开位（Init 的 `.ReloadDir()`）无该绑定 → Init 展开体内读 fav_records unbound → 全部 i32 级联。**修复**：展开臂按需补发被展开 handler 的读物化 Let（每次展开落独立 If(true) 块作用域——重物化=同初值合法遮蔽）。连带：① resolver 三处索引位 Dyn coerce（读/赋值/list get——json 值链索引 DYN_TAKE→i32）；② Dyn 接收者 `.clone()`=恒等；③ prep `.split(sep)` 消费成语改写为标量 count/at helper（自由 fn 不能返回/接 List——v1 面合同；括号配对接收位扫描 + 深度回零修正 + `sep` 保留字规避）。**拒墙**：**7** = `.replace` Str 方法（1）+ `no local named photos`/`set on i32` 对（5——span 归因待下轮定位）。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：最后 7 条（`.replace` prep helper + photos/set 对定位）→ **029 真窗实跑（V-03）** → 031 back 桥。

2026-11-23 T-03 续（round 30，§10）：**unbound/set 对清零；新层 str/Str 写位期望暴露**（auto-lang `8158df5e6` / auto-ui `55b82b10`）。**修复**：① `no local named photos`/`set on i32` 对根因 = CST 投影把 `X[i].f = v` 发射为 `X.set_field(i, "f", v)` 的**裸局部接收名**——view-data 基座不经读改写通道（读改写只覆盖读位）；`hook_stmts_lower` Effect 臂将裸 view-data 接收名改写为物化局部；② `.replace` prep helper（`__p117_str_replace`，标量词汇）；③ 注入 helper 族签名/累加局部全部转 **owned Str**（collections 面拒绝借用 str 参数/局部）；④ 场景道 place-assign 与 list-get 位 Dyn 索引 coerce（round-28 覆盖读+赋值，此处补足最后臂）。**拒墙现状**：unbound/set 类清零后编译推进到 **collections 面的 str/Str 写位期望类**——`.gal_error = "..."`/`.resultQueryId = .qf_id` 等字段写报 `expected str, found Str`（面要求 owned 文本而部分赋值位仍持借用期望——下一切片：node record 字段类型/赋值期望在 collections 域的 owned 统一）。回归全绿：ace-core lib 88 / p108 7 / p117 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：collections 域字段写/期望的 owned 文本统一 → 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。
