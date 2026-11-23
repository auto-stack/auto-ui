---
plan_id: PLAN-117
status: executing
feature_name: U05 Photo Gallery 与 Image Viewer 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-23
plan_revision: 1
current_step: 1
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

2026-11-23 用户通过 `/goal 继续完成计划117` 授权接续 r1 的实施、复审、合同内修复和交付。

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

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 31 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang ab40746af | next: work/T-03 续（str/Str 写位 + bool-Dyn 条件逐位归因 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 32 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 98d670947 | next: work/T-03 续（bool-Dyn 条件 13 + Str-Dyn 写 11 逐位归因 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 33 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang e55fe42b0 | next: work/T-03 续（CallFn 实参位 Dyn coerce → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 34 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang c2a5ae187 / auto-ui 38ee35ff | next: work/T-03 续（collections descriptor f32 行 + text_to_borrowed 用例 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 35 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 96c036439 | next: work/T-03 续（INVOKE 门/内联 state fn 的 collections 车道收口 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 36 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang d5941a4f6 | next: work/T-03 续（owned-walk 克隆源读 modeling → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 37 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 6a198e5a2 | next: work/T-03 续（所有权分析资源预算 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 38 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 6a198e5a2（无代码变更，勘察） | next: work/T-03 续（资源门结构归因 → dynamic 路由或分析压缩）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 39 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang 3919a9652 / auto-ui（预算档 256MB 已入库） | next: work/T-03 续（List(Dyn) 物化局部的克隆表达式设计 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 40 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang e145fced7 | next: work/T-03 续（abc.rs Collections 门干净重开：host-section + DYN/OBJECT 类型行/指令）→ 029 拒墙清零 → 真窗

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 41 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 3f5d17944 | next: work/T-03 续（split-units host 表传播 → 029 拒墙清零 → 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 42 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 864557d4f | next: work/T-03 续（scenario driver 效应体执行语义收口 → 029 编译链全清 → V-03 真窗）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 43 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang 864557d4f（勘察轮，无代码变更） | next: work/T-03 续（状态道 host 模块执行边界 → V-03 真窗经事件道）

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 44 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-117/auto-lang | commit: auto-lang c2dcd2668 | next: work/T-03 续（模块级 host_imports 的入口隔离 → 029 状态观察全通 → V-03 事件道）

2026-11-23 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 续（round 45） | outcome: partial | worktree: `D:/autostack/.wt/ui-117/{auto-lang,auto-ui}` | branch: `plan-117-dev` | code_commit: auto-lang `d5a78732e` / auto-ui `63b2e299` | next: work/T-03。

- AV 将进程内 JSON 原语与 console provider 分类分开。029 原入口观察到 `dark_mode` 等初值，继续进入事件程序编译。console 声明的前置授权和缺 HTTP provider 的具名拒绝均通过原负控。事件程序保留场景前门已经解析的模块声明，消除搜索、筛选等处理器的 `source.undefined`。
- Debug 准备 `cargo test -p ace-av --test conformance --locked --no-run` 退出 0（32.667 s）。定向实测 9/9。新增 `p117_initial_state_with_json_imports_needs_no_provider` 0.112 s（初始 flag 与 label 真值），`p117_component_event_calls_module_function` 0.049 s（派发后计数为 3）。原例 `p117_get_json` 2/2（0.055 s）、`p108_http_post_json_admitted_and_fail_closed` 1/1（0.056 s）、`av_console_unreachable_host_declarations_still_need_authority` 1/1（0.073 s）、`plan1106` 3/3（0.052 s）。时间为单过滤命令墙钟，发现与清理合并计时。
- `cargo build -p auto-ui-runtime --features ace-bridge --example ace_node_desktop_window --locked` 退出 0（42.47 s）。该 exe 对两个原 `src/front/app.at` 顺序复跑。029 退出 4（16.717 s），进入集合事件函数的借用文本校验 `verify.local-type`。031 退出 4（0.280 s），单组件入口的 `SelectImage` 需要当前缩略图行。均在装载阶段失败，无窗口交互证据。原始日志在 `D:/autostack/plan-artifacts/plan-117/av/`，`r45-029-replay.log` SHA256 `CAA3A5D702CF4A991A539A2DA55544A28991F254098912C7CB6885415368C70E`，`r45-031-replay.log` SHA256 `972AA184638B070445EABC255E3B2F40DB4667D796198990301EE51440987481`。

2026-11-23 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 续（round 46） | outcome: partial | code_commit: auto-lang `3e0eff185` / auto-ui `63b2e299` | next: work/T-03。

- AGE 包面的普通函数 `for <condition>` 原先在同一规则中连续消费两个条件；处理器 `for` 则误用允许记录构造抢占花括号的普通表达式。分别保留单一条件并复用处理器 `if` 的条件入口。029 的七个文本函数及 `GalApply`、`OpenPhoto` 单项探针均通过；完整准备源投影为 1 个组件。031 原源包面还将后端函数 `request_view` 归为未知 widget 消息。
- `cargo test -p ace-core --lib --locked --no-run` 退出 0（6.062 s）。`source::packages::tests` 4/4（0.075 s）；新增 `age_package_function_accepts_a_single_for_condition` 0.104 s 核普通函数循环 AST，`age_package_handler_for_condition_ends_before_the_body` 0.045 s 核处理器循环和体内赋值。独立源探针执行 0.238 s，`scratch/r46-package-final.log` SHA256 `4F18039C960E128813EFFD58F95F39365059BDFC77C5F7B3825493094AED1EBF`。
- 原 029 窗口工具重建成功（45.50 s），重新进入 AGE 包装载并具名拒绝 `asset_browser::asset_browser` 缺 `tree_view` 传递包。`av/r46-029-window.log` SHA256 `AB0CA24AD918A683BA415D7D9B9950F9F69114CB533BA41D561F9B51130BBAB1`。

2026-11-23 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 续（rounds 47–48） | outcome: partial | code_commit: auto-ui `085c0a72` / auto-lang `42fea1599` | next: work/T-03。

- 组件包装载按引用继续装入所选包模块的传递组件，保留源身份并去重。`auto-ui-runtime --features ace-bridge --lib` Debug 准备 92.950 s；新增 `photo_gallery_loads_asset_browser_dependencies` 1.839 s，核原 029 的 AssetBrowser、TreeView、PaginationControl、CollectionView、ImageViewport 各一个实际源单元。兄弟 `ui::ace_source_prep::tests` 7/8（0.921 s），`real_u03_apps_load_through_desktop_bridge` 的 015 Notes 报普通函数 `update_note` 被误判为 widget 消息。
- 把前缀点保留为投影事实。带点的调用按声明消息核对，未接线的普通调用保留为域外调用。ACE core Debug 准备 12.71 s，`source::packages::tests` 5/5（0.073 s）；新增 `age_package_dotted_calls_are_messages_and_free_calls_are_foreign` 0.169 s。029 和 031 的作者源包投影均通过（定向 0.304 s）。
- Debug 窗口工具重建 63.719 s，029 装载失败转移到 PaginationControl 的整数列表默认值（退出 4，2.288 s），031 转移到后端 `close_session` 未绑定（退出 4，0.766 s）。015 的 `update_note` 不再被识别为消息，仍受 store 内联后错误清掉嵌套域外标记影响。日志 `av/r48-029-window.log` SHA256 `176AD9DEB2EC00E3390E2AA84335D4700ED7EF7A7A1ED42797027CDDEF0F645B`，`av/r48-031-window.log` SHA256 `9636769AFCA65D4E8648A72121CF30D830FB2DA597678ECF19D8165D290DE3D1`。

2026-11-23 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 续（rounds 49–51） | outcome: partial | code_commit: auto-lang `1495e443d` / auto-ui `cd4d8260` | next: work/T-03。

- 兼容包属性以同质整数或文本默认值建立列表元素类型，空默认值继续沿用文本元素，混合序列具名拒绝。AGE 包解析保留 `$event`，历史 `on_<name>` 属性在存在同名带载荷出站事件且未作为独立通道使用时按该事件绑定。核父子载荷与重复绑定。包绑定器沿现有 try/catch AST 走完作用域校验。store 内联保留分支里的域外调用标记。
- Debug `ace-core --lib` 准备 7.878 s。`source::packages::tests` 7/7（0.123 s）。新增 `age_package_legacy_list_default_types_match_their_elements` 独跑 0.103 s（整数、文本、空、混合四形），`age_package_legacy_event_prop_forwards_declared_child_payload` 独跑 0.152 s（真实包绑定命名与重复负控）。时间为过滤命令墙钟，发现与清理合并计时。
- 029 原入口越过分页、TreeView 事件和 try 块，当前包绑定停在 `asset_browser::asset_browser` 无法绑定 `std::json.parse`。最近 Debug 构建 42.218 s，执行退出 4（2.361 s），无窗口证据。`av/r51-029-window.log` SHA256 `44B23299BE6801B1803932438F4C06D3EA0A1F4C3DED2799E20F5B9FCF044498`。031 原入口重复报 `close_session` 缺符号（退出 4，0.729 s），`av/r50-031-window.log` 同 round 48 哈希。015 原入口越过域外标记错误后仍失败，首错为 `Note` 类型没有绑定（退出 4，0.476 s）。`av/r49-015-window.log` SHA256 `5B7BE39CAD8206CFABCD15DF4F5947FC6D177B86247ED545CE53A5CEEA2D0B2F`。015 后续类型错误在 PLAN-117 基线上的归因待复验。
- 独立 A2Vue 生成复验（`cargo test -p auto-ui-runtime --features ace-bridge --test plan117_t01_probe --locked --no-run` 17.943 s，随后 `_lane_probe --nocapture --test-threads=1` 2.630 s，2 条探针退出 0）。029 有 42,789 B SFC、18 个未生成的处理器，031 有 33,702 B SFC、13 个未生成的处理器。两个探针的可执行处理器数均为 0，浏览器验收尚未开始。`av/r51-lane-probes.log` SHA256 `EEC90BB2738785C354D7C3DEEF3AC456148F450E1BA25B1B4172B9E10808FD3B`。

stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 51 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-117/{auto-lang,auto-ui} | commit: auto-lang 1495e443d / auto-ui cd4d8260 | next: user

2026-11-23 stage: work | plan_id: PLAN-117 | plan_revision: r1 | checkpoint: T-03 Carousel（rounds 52–54） | outcome: partial | code_commit: auto-lang `0882bbe17`、`526599037`、`a50d5f6fd` / auto-ui `cd4d8260` | next: work/T-03 Carousel 原画廊闭包和运行控件。

- 只读 AutoOS `cdb4883` 的画廊目录干净，原 `/carousel` 页 SHA256 `F43F17C29A83924ABC3061494E873F54140982BA828367A4A33F820CAE891BF1` 与 T-01 一致，实际组件文件 SHA256 `E08B8205CD6E7E9EBE9B9D83E0D925AB334177DE1A96DCCB6D88036E0B0FE6C1`。轮前 Debug 窗口工具 SHA256 `2B6EABA512FC49B8B54936FC113339A41605A9B3452010D4B524E82C685AEBE2`，原画廊 `app.at` 3.836 s、退出 0、4 帧，证据里原首页 `/` 仍未挂载（`expected fn, found widget`）。原 Carousel 页单独装载 0.192 s、退出 4。真实窗口激活与重新绑定后激活均报 `failed to activate captured window`，观察窗口超时退出 6，未发生侧栏交互，JSON SHA256 `A6647EB2C576D477483A8F8A637E1281FD0DCFE167230C67D2F3C7C349ABECFC`。
- 诊断对照中只改材料副本的 `loop` 键即让原页通过包解析，改 `align` 不通过。复用 PLAN-107 r1 交接分支的 `d414aec0e` 中 `loop` 键切片和 `9f4a65ad3` 的 `text | …` 扫描、投影、编解码及回归；保留当前工作树的其他语法。组件 `Carousel` 剩余失败由换行分隔的 `Next` / `Previous` 消息触发，材料副本只加逗号即通过，改字面样式不通过。使用既有的换行列表规则，保留逗号、载荷、尾逗号和同行无分隔拒绝。最终原页及组件文件逐字不改，包解析分别产出 1 个与 6 个组件（0.149 s）。日志 `scratch/r54-carousel-package.log` SHA256 `A1CF4D2DBA2E20A4EB004277CD6D07DF02731C37564D734BA3A295976D3791E1`。
- `ace-core` conformance 最后准备 1.742 s。PLAN-107 属性、行内文本等 5/5（0.136 s），AGE 包身份冻结 1/1（0.042 s），新增换行消息 `p117_carousel_messages_accept_newline_or_comma` 1/1（0.073 s）。复用的 `p107_carousel_loop_is_a_view_property` 单跑 0.164 s；`ace-syntax-runtime` 编解码往返通过。组件模块 29/30（0.159 s），`p979_component_vdetail02_named_refusals_keep_original_spans` 的非文本叶诊断失配在本轮起点 `1495e443d` 独立重建（9.244 s）后同码同 span 再现（0.055 s，退出 101）；Scalar 冻结身份在导入 inline-text 前已红并前后相同。新 AGE 两面身份按工作树实测更新。
- 原画廊的 Iced 路由、可见项、键盘/指针、循环、卸载与 A2Vue 浏览器验收仍待。PLAN-107 保留 Iced `f5ea801f` 和 Vue `6e8f8d89` 的可复用实现；两项补丁与本工作树渲染器重构存在冲突，下一步按现行渲染入口迁入并以原页回放，旧计划的运行收据不计作本次通过。029/031 仍按 §10 等待共享运行时写面裁定。

### 2026-11-23 goal work round 55–57 — Carousel 原页双端运行回放

- 接续 r1 的 T-02/T-03，保留五个原 Carousel 示例。AutoUI `3e3421f5` 复用 PLAN-107 的 Iced 控件与状态，`96faac40` 复用 Carousel 组件族的 Vue 降低并补齐脚手架依赖闭包。两项保留现行路由、布局和指针接线。AutoLang 固定 `a50d5f6fd`，AutoOS 原页与组件的哈希同 round 54，AutoDown `45b642d` 的 engine 离线构建成功。Debug 生产工具和 Iced 窗口编译完成（1m51s），`av/r56-product-build.log` SHA256 `ED9D2090AA4BDDC466EC1A3B8D814BDB713C278DF7A0C152D829468303593848`。
- Iced 定向测试 `native_carousel_keeps_visible_slides_and_inputs_in_sync` 1/1，覆盖可见项、分组状态、拖拽、垂直键、错误轴、循环及断点。原画廊真窗中以实际视图回调驱动侧栏及五组按钮，`/carousel` 挂载为页面会话 2，五组各从 0 前进到 1 并退回 0，Options 单独从 0 循环到 4，外壳和页面会话关闭，共 20 帧（6.859 s，退出 0）。真窗此轮依靠回调回放，物理输入仍缺独立观察。脚本 `scratch/r57-carousel-window.rs`，日志 `scratch/r57-carousel-window-five.log` SHA256 `120EE2FD104C9F30EF3FEC2A71BB43AF85F8C1EDA65A5DBB42C40946B69EBFC0`。画廊首页原有的 `expected fn, found widget` 仍可见，Carousel 路由正常挂载。
- 从原仓 `cdb4883` 重新提取画廊，在空生成目录以 `AUTO_UI_ACE_VUE=1`、本次 `AUTO_UI_TOOL_BIN` 和离线包缓存执行 `auto build --dir <original-gallery> --render vue`，生成 70 条路由，正式流程自动复制源 public 资源并通过 Vue 类型检查和 Vite 构建（84.961 s，退出 0）。先前对 `--gen-only` 目录直接运行 pnpm build 缺 `/icon.png`，用同仓原资源复测通过，最终冷构建使用正式命令。生成的 `carousel.vue` SHA256 `532F99DF5A4E3347807294A226C28C8B30F36CC80D591887775BFD3388942952`。完整终态 `scratch/r56-vue/auto-build-cold.log` SHA256 `8FC339131F6EC40EAA2CED37E1C281484CC4F26D612268C6890B533F2AEE4993`。
- Chrome + Playwright 从冷构建产物的原侧栏进入 `#/carousel`，确认 Basic/Sizes/Spacing/Orientation/Options 分别 3/6/4/4/5 项，按钮、拖动、水平及垂直方向键、错误轴、尺寸断点、间距、Options 尾首循环、离页卸载及重进归位通过，浏览器关闭，无页面异常（6.404 s，退出 0）。控制台有一条浏览器资源 404，页面响应记录无失败。脚本 `scratch/r56-vue/browser-check/carousel.mjs`，日志 `scratch/r56-vue/browser-cold.log` SHA256 `D7D2C45F525DABC5F8A5248DB88DB95F5757C8397BDE548A0619A1CB474EBA8F`。
- 包 SFC 回归 20/21（0.764 s，退出 101），Carousel 用例通过。唯一 chart 条件样式列表拒绝，同一原测试输入在 round 52 轮前及当前 parser 均重现相同 `source.age.view-list` code 与 span。组日志 `scratch/r57-package-sfc-group.log` SHA256 `82579040C84E5AFD2BEB5DAC815C8833D6784AAAA20627D1EAB96A97AF4421D2`，轮前对照 `scratch/r57-chart-base.log` SHA256 `6A44CEB486D228423E3C69B28394FC39B556CDC90289F7662FC8DF70EA3DA527`。
- 用同次生产窗口工具逐个复跑 029/031 原入口。029 仍在 `asset_browser::asset_browser` 缺 `std::json.parse`（2.483 s，退出 4，`av/r57-029-window.log` SHA256 `0B57D5CAAA5015CCAEC757B5A3B075E55ACDEC2AD42AE56C4D981948A6D10CFC`），031 仍缺 `close_session`（0.785 s，退出 4，`av/r57-031-window.log` SHA256 `843251CE9BC610E5F2054E2FF66ADF62BF2F55C5D8C80AA4292D8FD48DACD062`）。共享写面见 §10。完整 AV 阶段尚不能送独立复审，未落主代码或交接收据。恢复点 `work / T-02–T-03`。

### 2026-11-23 goal work round 58 — 029 JSON 接线与 031 后端对照

- 受检包对照：`std.json` 的 `json_is_valid` 能绑定并编译，`std::json.parse` 拒绝，原样 `use auto.json` 报模块缺失，省略 VM 命名空间导入的值原语能在 AGE 包中编译。仅修改 AutoUI 组件包源准备（`5661ad14`），按已解析导入的 span 等长空白化裸 `use auto.json`，保留原源码、文本字面量及真正 `std.json` 导入。未更改共享 JSON 能力合同。对照（准备 2.068 s 内完成，运行 0.846 s）`scratch/r58-shared-binding.log` SHA256 `34BCE909C4F0F95E44E782D917509E3B4148662DB02F651E485CBFEC5FE3782F`。
- AutoUI `--lib --features ace-bridge --no-run` Debug 准备 69.048 s。新增 `checked_widget_modules_use_age_json_values` 1/1（墙钟 0.171 s），核受检包事件中的结构化解析、成员读、命名空间与文本保真。原 029 的组件包闭包 1/1（1.488 s），原 031 backend proxy session 1/1（0.123 s，health、原 fixtures 目录的 open 与六张现有图像名称）。日志分别在 `av/r58-json-prep.log`、`av/r58-photo-closure.log`、`av/r58-proxy-031-current.log`，每次单过滤，发现/清理未独立计时。
- 生产窗口 Debug 构建 72.830 s。029 真源越过 `std::json.parse`，首拒移至 AssetBrowser `provider_error` 的隐藏 computed 函数未能推断返回类型（2.764 s，退出 4）。031 仍缺 `close_session`（0.918 s，退出 4）。显式加入 031 原 `back::api` 的包探针在 `ViewerControl =` 被 AGE 语法具名拒绝，原生后端正控以上述 proxy 证据为准。窗口日志 SHA256 分别为 `D0A4FEBA4365A05DCE46388540B043B96C33435940AF1FA182F48979AC8E6962`、`F34AF1C5B472A3F760F142BC5A301596CC945140573BFE80DA999611466983F1`。
- 同次真实 A2Vue 组件包入口 `generate_linked_front_sfc` 回放，029 首拒 `storage calls require the owning host`，031 首拒 `unknown module auto`。脚本准备 17.255 s，两个入口合计执行 0.998 s。此探针直接调用 Vue 生产 lowering，未进入正式工程生成、构建或浏览器。日志 `scratch/r58-linked-vue.log` SHA256 `41F986D66981243ABD880CAD35B6E962A883351C7BC443288073A14C463170BB`。
- Carousel 新一次真窗输入观察确认 Windows 停在锁屏。重选目标后仍为 `failed to activate captured window`，依 computer-use 停止注入并请求解锁。真窗守卫退出 6，1,098 帧仍停在 `/`。`scratch/r58-carousel-physical.json` SHA256 `3AB61D1D09A545EE45B0CE6659C136430278ECAED8D0D9EF1719C6A979C2211C`。round 57 的回调证据有效，物理输入继续待验。

### 2026-11-23 goal work round 59 — 029 computed 分支负控

- 用与 029 同形的布尔和文本 `computed => if`（含 else-if）及其原视图读法验证。仅补签名推断后，文本分支报 `expected Str, found str`（0.106 s，退出 101）。复用受检语句分支的实验可越过该类型拒绝，但 view 内联仍发射 descriptor 未承认的 `expr.if`，所以同一场景继续拒绝（0.098 s，退出 101）。两次测试准备最终成功分别为 12.339 s、10.259 s。发现及清理时间未单计。本轮另有编译期实验拒绝，日志在 `av/r59-computed-tests-build.log`（1.722 s）、`av/r59-computed-tests-build-r3.log`（2.954 s）。
- 实验改动保存在 `scratch/r59-partial-computed-return.patch`，AutoLang 工作树恢复至 `a50d5f6fd`。当前诊断日志 `av/r59-computed-case-r2.log` SHA256 `A62C87786D39A8772ECD395BC0DD404E74E16AC29F1BCED4FDD011FED0103C8F`。后续要保证 computed 自身和所有视图读取共用受检分支语义，保留布尔条件、文本所有权、else-if 顺序及混合分支拒绝。未将只过隐藏函数的部分实验计作交付。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 59 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `a50d5f6fd` / auto-ui `5661ad14` | next: work/T-02–T-03（029 受检 computed，031 带授权后端接缝，Carousel 解锁后物理输入）。

### 2026-11-23 goal work rounds 60–61 — 029 条件计算与 031 文件授权对照

- AutoLang `285f4f134` 将有末尾 else 的 computed 返回值降到已有受检语句分支。每实例的计算函数和视图读取使用同一个受检结果，取消了对此计算值的重复表达式检查。保留其余成员检查。新增正反例核布尔值、文本值、else-if 及异型分支拒绝（0.094 s，1/1，Debug 准备 6.603 s）。AV 真执行的独立探针用三个实例分别观察 `ready`、`retry`、`warning + error`，含布尔可见性与文本分支（准备 0.898 s，执行 0.258 s，退出 0）。`scratch/r60-computed-av-observe.log` SHA256 `66588E416F1EDC020D69164F96FB465974368F0C5346CE2D33D489AE09F0F09D`。更早两次观察分别暴露测试须实际启用受检视图与计入 timer 行，日志 `scratch/r60-computed-av.log`、`scratch/r60-computed-av-live.log` 均保留。
- 029 原入口越过 `provider_error` 后到 PaginationControl `page_count`，其各返回臂出现同受检类型的 `int` / `i32`。AutoLang `e23c66e2d` 以既有 `vt_of_word` 等价关系合并内建类型别名，保留不同类型的拒绝。更新后的定向用例 1/1（0.093 s），AGE packages 8/8（0.129 s），981 members-slot 19/19（0.207 s），Debug 准备 7.845 s，发现/清理未单计。阶段日志 `av/r61-inference-case.log`、`av/r61-inference-package-group.log`、`av/r61-inference-members-slot.log`。
- 同工作树生产窗口 Debug 两次准备 76.882 s / 54.955 s。029 原入口 round 60 退出 4（2.839 s），随后 round 61 越过分页计算，仍在 admission 阶段，首拒变为 `asset_browser::asset_browser_util:47:39` 的裸 `List` 返回类型未绑定（3.069 s，退出 4）。记录在 `av/r60-029-window.log`、`av/r61-029-window.log`，最终日志 SHA256 `E88419E012E4D2A8C44B537265E5320CB05DFA1EE5A4D5112785F6D99E55DB96`。
- 031 对原 `src/back/api.at` 用当前 `RunningProxy::open_file_session` 创建隔离会话，仅使用已冻结的测试目录。拥有授权时观察到六个现有图像和不透明媒体 URI；首次关闭为 true，重复关闭为 false，旧会话名称为空，卸载代理后原会话路由 404。负控未通过，授予目录之前原后端已返回 200 并读取同一测试目录。准备 5.692 s，执行 0.558 s，退出 101。日志 `scratch/r61-031-file-backend.log` SHA256 `022E69C859DBF5CEF5217954B3BF418014B55454943482F30F7EBC3A47B8AF3F`。此对照是已有 VM 媒体后端，AGE 原前端仍待带授权接线。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 61 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `e23c66e2d` / auto-ui `5661ad14` | next: work/T-02–T-03（029 包函数的旧 List 语义，031 后端权限归属）。

### 2026-11-23 goal work rounds 62–63 — 029 动态集合签名与 AV 验证

- AutoLang `11177e8b0` 将 AGE 包辅助函数的裸 `List` 声明接到现有受检动态集合，按注解选择集合编译面。保留已有显式列表元素类型及未声明类型诊断。PLAN-681 的旧语义同为动态元素。仅补类型映射的初次用例仍走标量面（准备 10.530 s，执行 0.150 s，退出 101）；补齐选择后定向 1/1（0.089 s），AGE packages 9/9（0.164 s），准备 6.867 s。日志 `av/r62-list-case.log`、`av/r62-list-case-r2.log`、`av/r62-list-package-group.log`，发现/清理未单计。
- 独立对照中，完全相同的已受检输入经普通 Auto closures 降低、MIR 核准、ABC 编解码和 AV 执行得到原记录文本 `计划`，现行 AGE 转换到 dynamic 配置后在 `mir.type.list-elem` 拒绝。对照准备 2.386 s、执行 0.183 s，`scratch/r62-list-compare.log` SHA256 `670FC1E264061B9489880C6EF90A30B2F9FABC8C3CB7E04E004216F8CB96EA1B`。AutoLang `dfcc380f8` 允许已有共享集合参数的 dynamic 配置携带原 `av.language-values/1`，不改变普通 dynamic 配置准入。新增 `age_package_bare_list_preserves_dynamic_records_through_av` 1/1（0.229 s，准备 24.282 s），核 AGE 原观察入口成功及移除共享集合参数标记后拒绝；981 包 AV 38/38（0.749 s），dynamic 语义 21/21（0.096 s）。`av/r63-list-av-case.log` SHA256 `78383237DAB6D01341E8934B29CD72426FFD959FF8402527346D5F8894379832`。
- 原 029 生产窗口 Debug 先后准备 67.475 s / 60.842 s。原入口已越过裸 `List` 返回类型，仍在 admission 阶段退出 4（3.660 s / 3.008 s）。当前 `app::app:306:34` 的 `no local named self` 精确对上 `AssetBrowser(records: .gal_records)` 字节 13265。用现行生产源准备加 AGE 包投影核实，`gal_records` 与 `gal_folders` 的确有 handler 写入和实际 widget 参数读取，初值 `[]` 却只投影成惰性 view-data，两者均无状态槽。对照准备 7.289 s、执行 0.433 s，`scratch/r63-029-view-data.log` SHA256 `720C3707C69D09BA265D88156AD0B6DECF00A1DB82A8D1BC56CA277B563A1C9F`。窗口终态 `av/r63-029-window.log` SHA256 `E82E5A39C6E5420B240F17242DC4F400EC10C44F0E958FF12FDE118095F40784`。
- 恢复点为完整核对带空默认值的旧 widget List 输入，沿现有跨组件列表投影接入实际可写状态，验证 handler 更新后子组件可读到新列表。纯展示用惰性 view-data 仍按 PLAN-106 的既有合同。031 带授权后端和两个应用的 Vue 交互尚无新实现。Carousel 真窗物理输入仍待桌面解锁。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 63 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `dfcc380f8` / auto-ui `5661ad14` | next: work/T-02–T-03。

### 2026-11-23 goal work round 64 — 029 父组件列表更新

- AutoLang `14a0679a2` 将带空默认值、由 handler 写入且作为 widget 参数传递的旧动态列表投影为受检模型字段。新用例在两个实例中只向第二个发送 `Load`，子组件分别显示 `empty` 与 `新照片`。同一用例核实纯展示列表仍按惰性 view-data 投影。Debug 准备 23.176 s，定向 1/1（0.094 s），981 包 AV 39/39（0.748 s），发现和清理未单计。`av/r64-forwarded-list-case.log` SHA256 `17AF4FE1C40946637ED917D09FBC7EF3DFD51B81FDFEF08E3228B31788D7C211`。
- 029 生产窗口 Debug 准备 93.724 s。原入口越过 `AssetBrowser(records: .gal_records)` 绑定，仍在 admission 阶段退出 4（3.136 s）。首拒为 `app::app:399:29` 找不到 `json`，对上收藏恢复的 `json.parse`。`av/r64-forwarded-list-product-build.log`、`av/r64-029-window.log`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 64 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `14a0679a2` / auto-ui `5661ad14` | next: work/T-02–T-03。

### 2026-11-23 goal work round 65 — 029 集合路径接入 JSON 和 HTTP

- AutoLang `1d2253dd5` 让 AGE 包的集合路径复用原受检 JSON 和 HTTP 原语。嵌套表达式使用同一解析逻辑，带共享集合参数标记的 dynamic 字节码可携两族声明，宿主授权在执行时仍按原合同检查。定向用例验证字节码往返保留两族声明、移除标记即拒绝，以及收藏记录形状的 JSON 取值通过 AV。
- Debug 最终准备 7.280 s，定向 1/1（0.184 s），981 包 AV 40/40（0.859 s），dynamic 语义 21/21（0.113 s），HTTP 组 13/13（0.320 s），JSON 组 4/4（0.883 s），发现和清理未单计。定向日志 `av/r65-host-av-case-r3.log` SHA256 `A8D8D2139483857EF88D4B2812CDB2DD06705792D10C695D73AB8900D45B6D4E`。首测因嵌套表达式仍走另一解析路径而拒绝，见 `av/r65-host-av-case.log`。本组未用外部 HTTP。
- 029 生产窗口 Debug 准备 79.525 s。原入口越过收藏恢复的 JSON 绑定，仍在 admission 阶段退出 4（3.165 s）。首拒 `app::app:441:31` 的 `self` 对上目录扫描中 `.fav_records.len()`。该列表由 handler 更新并在另一个 handler 读，是私有业务状态。`av/r65-029-window.log` SHA256 `8B5766764BFF50793633839FCE7229A979F4E8516D508879991645CBE411C8F1`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 65 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `1d2253dd5` / auto-ui `5661ad14` | next: work/T-02–T-03。

### 2026-11-23 goal work round 66 — 029 私有收藏列表跨事件

- AutoLang `96147bf4b` 将空初始化、被 handler 替换且在 handler 内读取的私有列表纳入受检模型状态。定向 AV 用例在第二个实例恢复一条记录，随后两个实例分别用同形循环读列表长度，观察文本 `0` 与 `1`。Debug 首次准备 35.567 s、最终用例准备 3.753 s，定向 1/1（0.215 s），981 包 AV 41/41（1.008 s），HTTP 组 13/13（0.240 s），发现和清理未单计。`av/r66-private-list-av-case-r5.log` SHA256 `AA1CA4D303236581AD00AF1D5F2E20931C7C1970DF0B1A0281000324B7A2B819`。
- 另一对照在该循环内比较动态记录 `id` 与字符串，仍得到 `0`（0.171 s）。当前解析器将带任意 Dyn 操作数的相等比较交给判空分支，须单独验证非空值比较。未将该记录匹配算作通过。`av/r66-private-list-av-case-r3.log` SHA256 `16552EC098ACEC3CE32616C46BD51DA3B880CE4AB10ACD9BF15D29B35A0C0E38`。
- 029 生产窗口 Debug 准备 91.726 s。原入口越过收藏列表的绑定，仍在 admission 阶段退出 4（2.746 s）。首拒为 `asset_browser::asset_browser_util:34:23` 的 `r.id.replace(...)`，包工具函数原样进入编译，入口准备已有的文本替换能力未覆盖此处。`av/r66-029-window.log` SHA256 `40724275A1ECFF912AD325316CFAB0DD233DB82385F42F923D944ADAC2F7B31B`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 66 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `96147bf4b` / auto-ui `5661ad14` | next: work/T-02–T-03。

### 2026-11-23 goal work round 67 — AssetBrowser ID 校验

- AutoUI `1521f47a` 使用受检文本精确比较 ID，保留空 ID、空 revision 和首个重复 ID 的既有返回值。重复 ID 以拥有文本返回，避免从本地字段返回借用视图。列表元素投影按原 CollectionItem 结构核类型，预览动作仍依赖图片类型及非空缩略图 URI。
- 使用真实 AssetBrowser 工具模块和 CollectionView schema 的 AV 用例通过，核对带 Unicode 和旧转义字符的两个不同 ID、准确定位、未命中及重复 ID。Debug 准备 73.474 s，定向 1/1（0.271 s），029 包发现 1/1（1.327 s），发现和清理未单计。`av/r67-asset-browser-case-r4.log` SHA256 `83E63BFC7754B7EF4915494034CAB68AB26C280133FE7D06300DABAEB403194E`。
- 029 使用 round 66 生产二进制读取更新后的依赖源，越过 AssetBrowser 记录验证，仍在 admission 阶段退出 4（2.861 s）。首拒为 `image_viewer::image_viewport:263:16`，精确对上 `OnWheel` 中读取计算成员 `.c_ok` 的条件。`av/r67-029-window-r6.log` SHA256 `9CF3E2337930DF0E209E008DC8573C1EB6BA2ADB1488E74677B0C743A56A6CAB`。029 浏览、嵌入和缺图恢复尚未进入真窗验收。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 67 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `96147bf4b` / auto-ui `1521f47a` | next: work/T-02–T-03。

### 2026-11-23 goal work round 68 — ImageViewport 处理器读计算成员

- AutoLang `b936120d6` 在 AGE 包处理器复用组件的计算成员展开。定向 AV 用例先在错误态派发，再只修改第二个实例并重新派发，结果为 `0` 与 `1`。Debug 准备 14.406 s，定向 1/1（0.166 s），981 包 AV 42/42（0.870 s），发现和清理未单计。`av/r68-computed-handler-av-case.log` SHA256 `98D9307796749513631C507907EED676F222ECA9DD21D7D066945E278B709691`。
- 029 生产窗口 Debug 准备 54.911 s。原入口越过 `OnWheel` 内的 `.c_ok` 绑定，仍在 admission 阶段退出 4（2.509 s）。首拒在 `image_viewer::image_viewport:140:47`，对应 `onwheel: .OnWheel($event.deltaY)` 被归一为找不到的 `nevent`。现有 Iced 适配器已定义 wheel 的单 f32 和 pan 的 f32/f32/phase。029 的 checked 包视图由 `ace_node_desktop/package_view.rs` 投影，该路径尚无 `image_surface` 分支，队列目前承载点击、文本和指针坐标。恢复点为沿现有输入协议和媒体授权接入包视图、图像变换与事件回调。`av/r68-029-window.log` SHA256 `5A71F6BF5E532BA3484E2F15EDC0D9E03C2F1EA6E2116DB3000F2362E1E9A510`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 68 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `b936120d6` / auto-ui `1521f47a` | next: work/T-02–T-03。

### 2026-11-23 goal work round 69 — 图像滚轮与拖拽的受检事件载荷

- AutoLang `b086dd621` 按 `image_surface` 的元素和事件键识别加载器规范化后的 `$event.deltaY`、`$0, $1, $2`。验证目标消息分别接受 f32 与 f32/f32/str。AV 视图记录保留宿主载荷种类，普通双击和静态消息参数仍经现有值路径。
- 定向用例核滚轮、拖拽、无参双击及错误滚轮消息类型。Debug 准备 31.739 s / 11.385 s，首次用例揭示独立的消息实参检查仍在求值占位符（0.170 s），修正后定向 1/1（0.170 s），981 包 AV 43/43（0.808 s），dynamic AV 21/21（0.107 s），发现和清理未单计。`av/r69-image-event-case-r2.log` SHA256 `87DCE40E3834DF716CB8FDC875790159ECF247303DB6D2F72AC34F85C1C72C42`。
- 029 生产窗口 Debug 准备 86.603 s。原入口越过 `$event.deltaY` 及 `$0, $1, $2` 绑定后仍在 admission 阶段退出 4（2.512 s），首拒为 `pagination::pagination_control:102:38` 的 `size_candidates.push(size)`。原公开候选是整数列表，内部裸 List 将整数按动态所有权元素转移。`av/r69-029-window.log` SHA256 `68B4A89E1DDD0CA56778CD80D7912653DBD07CF1FC1967009CE271F2E8EF6379`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 69 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `b086dd621` / auto-ui `1521f47a` | next: work/T-02–T-03。

### 2026-11-23 goal work rounds 70–71 — Iced 图像事件与分页计算成员

- AutoUI `ee933de0` 给受检包视图接入现有 Iced ImageSurface，投影 zoom、offset、rotation、load/error 和输入回调。事件队列按绑定种类带入 f32 或 f32/f32/phase，检查视图版本。同一帧已入队的拖拽消息只在视图仍提供同一个资源绑定时重定位。图像来源走已有媒体 URI，或只读入口 `front/public` 内的文件并在关闭时释放其媒体票据。
- Iced 投影视图用例调用实际挂载的回调，验证缩放 1.5→2.0、旋转 -90→270、同帧入队的 start/move、偏移 (7, -3)、无参双击、异种载荷和旧帧拒绝。UI980 头部准备 0.262 s，最终 Debug 准备 12.635 s，定向 1/1（0.351 s），已有 pointer 与 SVG 分别 1/1（0.370 s、0.350 s）。首次用例因缺测试进程准备标记退出 101，重新以已准备的提供器执行。日志 `av/r70-image-desktop-case-final.log` SHA256 `DE82492E1A801630234CEA854436902360F7A5A15661E30443C7D4A76F8909E4`。用例验证投影回调及 AV 计算，未作为设备操作或远程媒体解码收据。
- AutoUI `25aeb019` 将内部分页候选改为 `List<int>`。029 用同一生产二进制读取更新源，越过 `.push(size)`，仍在 admission 阶段退出 4（2.529 s）。首拒对上分页 `changed` 的对象字段读取 `.current`。`av/r70-029-window.log`。同批 031 原入口仍在 `close_session` 符号解析退出 4（0.746 s），`av/r70-031-window.log` SHA256 `950350C76BA5F189E6F166B067D8812E0ED51518939CE43D2DF4BF2CE9A92853`。
- AutoLang `2d2ffbd8a` 在对象字面量的字段值中展开现有计算成员。回归以动态集合验证嵌套对象读取，Debug 最终准备 4.338 s，定向 1/1（0.190 s），包 AV 43/43（0.825 s）。`av/r71-computed-object-case-r2.log` SHA256 `2F52B985ACDFEDE18476C1F6CF6DC2A3E64AC556C59DE0606F73046E05F76EA2`。
- 029 最终生产 Debug 准备 30.782 s，越过分页，仍在 admission 阶段退出 4（2.482 s）。`tree_view::tree_view:110:62` 对上 `.WantToggle(r.id)`，受检消息实参在动态 ID 上生成了 `.len()` 验证。`av/r71-029-window.log` SHA256 `3A71ADA9A8518D586839D4040DC2EEC7AF1DA893CA047275FCDABBB550673248`。原 029 后端相对图像 URI 待接既有 HTTP 来源，真窗浏览、嵌入和缺图恢复仍未验收。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 71 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `2d2ffbd8a` / auto-ui `25aeb019` | next: work/T-02–T-03。

### 2026-11-23 goal work rounds 72–73 — TreeView 事件与遍历

- AutoLang `b509a6d95` 按受检消息签名在校验和视图求值时读取动态文本实参。AV 核从动态记录派出中文 ID `目录`，并拒绝以整数冒充的动态 ID。Debug 最终准备 6.048 s，定向 1/1（0.176 s），包 AV 44/44（0.828 s），发现和清理未单计。`av/r72-text-event-case-r3.log` SHA256 `4EFEFEBB4D3245799F124F67D645FC59B7DA3B61D5CB19196887F4720330D8DD`。
- 029 生产 Debug 准备 40.517 s，越过 `.WantToggle(r.id)` 后在 admission 阶段退出 4（2.511 s）。首拒为 `tree_view::tree_view_util:50:9` 对动态列表调用 `.pop()` 落到仅收整数的标准库接口。`av/r72-029-window.log` SHA256 `5ACF457CA3B37A140B5E3B0E70BEED0233EB3976678C842EB10A7B0BC69D0777`。
- AutoUI `9c4dcae7` 用现成的带索引移除取得遍历栈的栈顶。在显式旧 VM 入口以相同输入运行原包工具函数，对照核总节点 4、可达 3、行序、可见及结构父节点、重复 ID，前后输出一致（0.134 s / 0.384 s）。`scratch/r73-tree-stack-vm-base.log`、`scratch/r73-tree-stack-vm-r2.log`。Windows CLI 默认 ACE 脚本道先具名拒旧 `while`，该结果保留于 `scratch/r73-tree-stack-vm.log`。
- 029 使用 round 72 生产二进制加载当前依赖，越过 `.pop()` 后仍在 admission 阶段退出 4（2.493 s）。首拒在 `tree_view::tree_view_util:99:23` 对动态 ID 调 `.replace()`。恢复点为沿现有文本合同接入去重键转义，保留 TreeView 的 Unicode 和大树性能约束。`av/r73-029-window.log` SHA256 `7E7705B16C63980956FB4E58ADD3BFD4B27AD18067D9868E4BB0E78E30DD8F09`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 73 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `b509a6d95` / auto-ui `9c4dcae7` | next: work/T-02–T-03。

### 2026-11-23 goal work rounds 74–76 — TreeView 文本语义与目录列表接续

- AutoUI `579892e9` 保留 TreeView 的 `|id|` 去重标记和原生子串查询，通过受检文本操作转义 `!` 和 `|`。输出拼接原 ID 的完整片段。目录与焦点比较使用声明的文本值，诊断与图标等派生结果按拥有文本返回。
- 显式旧 VM 入口对照核中文、表情、分隔符与其转义拼写、空 ID、遍历和父节点、1,000 个非重复键，结果逐行一致（基线 0.307 s，变更后 0.750 s）。`scratch/r74-tree-escape-vm-base.log`、`scratch/r74-tree-escape-vm-final.log`。完整作者工具模块的 AV 用例同时核重复 ID、可达行数、中文行焦点及两个父节点查询。最终 Debug 准备 71.088 s，定向 1/1（0.459 s），发现和清理未单计。`av/r74-tree-escape-case-final.log` SHA256 `B992707E0986EE4E0191641BC88B23CEC80E5F81AE8FDFA55AD63B8963E1E090`。AssetBrowser 原 ID 和 029 包发现定向用例也分别通过（0.272 s、1.317 s）。
- 029 原入口 round 74 越过 TreeView 文本转义，停在 `GalApply` 向 `.gal_folders` 赋值（2.509 s，退出 4）。AutoUI `bd885aa3` 在同一个动态列表中追加“全部”“收藏”和实际目录项。round 75 单加 List 注解仍在同一处拒绝（2.512 s），改为在空列表追加后越过该处，下一拒为 AssetBrowser 将计算出的 `.folder_nodes` 传给 TreeView（2.525 s，退出 4）。`av/r74-029-window.log`、`av/r75-029-window-r2.log`。
- AutoLang `956141c3b` 以计算成员所调用函数的裸 List 返回声明为子组件列表参数提供检查上下文，导入仍由原包绑定器解析。Debug 准备 15.602 s，定向用例从实际导入函数收到“照片”并经 AV 子组件显示，通过 1/1（0.170 s），包 AV 45/45（0.850 s）。`av/r76-computed-list-case.log` SHA256 `71663E1FA2D582F73689A0FB3D1F426F0A948608C3A3D7A1A326238E42D8953A`。
- 029 生产 Debug 准备 59.621 s，越过 AssetBrowser 的目录列表传参，仍在 admission 阶段退出 4（2.554 s）。首拒为 `tree_view::tree_view:63:62`，对上未传入的空 `.disabledIds` 在 `tv_derive_rows` 调用中的元素类型。`av/r76-029-window.log` SHA256 `47FC474D515A452FC4EB68D9EF4C73FA2D5BE106C1F129FB193726762C53AE97`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 76 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `956141c3b` / auto-ui `bd885aa3` | next: work/T-02–T-03。

### 2026-11-23 goal work round 77 — TreeView 空列表默认值

- AutoLang `5972d904f` 对未在任何使用点传入的空 List 默认值，沿已声明的计算函数形参确定列表类型。回归同时核导入函数产生的目录记录和子组件省略的禁用列表，Debug 准备 17.308 s，定向 1/1（0.186 s），包 AV 45/45（0.897 s），发现和清理未单计。`av/r77-computed-default-case.log` SHA256 `E49EB7521E64F4420F69AF60D58705E625FE40BE0ED1D7A419012367DBFBFD40`。
- 029 生产 Debug 准备 71.805 s。原入口越过列表绑定和类型检查，仍在 admission 阶段退出 4（5.157 s）。生成的 Atom 文档有 8 处 `expr.if` 遭现有描述符具名拒绝，恢复点为将值位置的条件表达式按原分支顺序降低到已支持的控制流。`av/r77-029-window.log` SHA256 `DC285EFC4BD3A41218664AC81BE8E1428D59309065B21AB21262020ED11D175F`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 77 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `5972d904f` / auto-ui `bd885aa3` | next: work/T-02–T-03。

### 2026-11-23 goal work round 78 — 条件表达式接入受检控制流

- AutoLang `223997797` 将 AGE 包的值位置 `if` 降为现有条件块和结果槽，条件及分支值的预备语句保留在各自路径。定向用例核对象里的 `if/else if/else` 分别得到 `first` 和 `second`，未选中的非法 JSON 分支无效果。Debug 准备 18.658 s，定向 1/1（0.186 s），包 AV 46/46（1.128 s），HTTP 13/13（0.258 s），动态值 21/21（0.180 s），发现和清理未单计。`av/r78-if-value-case.log` SHA256 `2EFA7B8101385A372B3BEEF73262BFB96980161D236BFF77905F42D58768E8A8`。
- 029 生产 Debug 准备 77.079 s。原入口越过 Atom 的 8 处 `expr.if` 拒绝，仍在装载阶段退出 4（4.969 s），首拒 `source.age.widget-view-output: the entry component has no checked view`。`av/r78-029-window.log` SHA256 `AB76E238B086241D54CDE77AA4CDBB8A3FD3A5C602A75B0E8D53F106CDD72C4A`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 78 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `223997797` / auto-ui `bd885aa3` | next: work/T-02–T-03。

### 2026-11-23 goal work round 79 — 029 进入运行时组件挂载

- AutoUI `e6b56d9b` 在同一个动态列表中收集过滤排序结果，业务总数从发布的 `.view_list` 读取。列表进入受检模型，入口视图得以发射。直接给空模型列表加裸 List 注解的对照按文本列表解析，先后触发元素转移（2.499 s）和列表类型拒绝（2.512 s）。`av/r79-029-window.log`、`av/r79-029-window-r2.log`。
- 029 使用 round 78 生产二进制加载当前原入口，越过入口视图和编译检查。进程终态为 19.497 s、退出 4，首拒 `ace.desktop.widget-timer: child timers are unavailable for AssetBrowser`，无成功窗口收据。该定时器驱动查询镜像，既有场景调度按子组件路径调度，当前实时挂载明确拒绝子组件计时器。恢复点为在子组件当前模型上评估计时器条件，接现有调度并核挂载、卸载和代次。`av/r79-029-window-r3.log` SHA256 `A9210411A65C743B123C1EC597ED5F376E411A5134CF63AA3F079697A6B63B94`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 79 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `223997797` / auto-ui `e6b56d9b` | next: work/T-02–T-03。

### 2026-11-23 goal work round 80 — 子组件定时器进入 Iced 调度

- AutoLang `010391873` 用实时子组件的输入和模型评估计时条件。持续挂载沿用实例标识和定时截止时间，重排后按当前视图路径派发。AutoUI `10c6431a` 在提交新帧时同步启用的定时器，隐藏或卸载时移除，重新挂载时重新计时。
- 包 AV 46/46（0.447 s），UI980 准备 0.227 s，最终 Iced 测试 Debug 准备 46.318 s。定向子组件用例 1/1（2.079 s），核重排后到期派发、条件禁用、隐藏及重挂隔离。已有根定时器用例 1/1（1.427 s）。`av/r80-guarded_child_timer_follows_keyed_mounts_and_restarts_after_remount-r2.log` SHA256 `A60FD05B241CB2A7D070AE69EE7A3E92BCA2C4C17138280D947B79639C5B11BD`。
- 029 原入口用最终生产 Debug 二进制（准备 28.234 s）越过 AssetBrowser 定时器拒绝，10.800 s 退出 4。首拒为 `verify widget runtime MIR`，入口处理器 `d_p3_appm3_app___age_p_hidden_runtime_handler_0_0` 的 `mir.lang-fail-span`，MIR 报跨度 4996–5002。下一步以最小受检包复现并定位进入动态线族的失败位置与错误控制流。尚无成功窗口收据。`av/r80-029-window-final.log` SHA256 `ECB703430403EFD6D72FDAF270E0BBF4864DCDD113AFC267BC98CF8F8781E28F`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 80 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `010391873` / auto-ui `10c6431a` | next: work/T-02–T-03。

### 2026-11-23 goal work round 81 — 实时组件保留受检错误控制流

- AutoLang `1739516ef` 在已有普通语言值准入的动态线族中校验调用错误边、本地抛出边和失败位置。保留消息类型、目标块和跨度顺序校验。用例通过实时组件链接、派发和渲染，核对受保护调用的列表越界进入恢复分支，以及未捕获错误回滚模型。
- 最终 Debug 准备 5.594 s，定向用例 1/1（0.250 s），包 AV 47/47（0.397 s），PLAN-1087 错误控制流 6/6（0.160 s）。`av/r81-av-case-final.log` SHA256 `9BF7FAF5F152C2B99C6131B402AC1F983BCA98230792C829C95B6ACCE54CF833`。一次 PowerShell 筛选参数展开错误触发了未筛选的旧测试二进制，遇多处失败和栈溢出。上述定向结果来自修正后分别执行的筛选命令，全量回归未验收。
- 029 生产 Debug 准备 61.387 s，原入口越过错误位置校验，10.999 s 退出 4。首拒为相同处理器内的 `mir.text-display`，跨度 21091–21105。恢复点为动态记录标量的文本显示准入，仍无成功窗口收据。`av/r81-029-window.log` SHA256 `B2695C46570A59F3F0B67D56393E3C85D624ECF291E911AB173F66A0730CBCF4`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 81 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `1739516ef` / auto-ui `10c6431a` | next: work/T-02–T-03。

### 2026-11-23 goal work rounds 82–83 — 动态标量显示与分页所有权

- AutoLang `4bfd3b214` 让启用了受检普通语言值的实时 AGE 包复用现有动态标量文本指令。定向用例先复现 `mir.text-display`，修复后经实际挂载和事件派发显示 `20261123/未知`。Debug 准备 8.830 s，定向用例 1/1（0.104 s），包 AV 48/48（0.413 s），文本状态 8/8（0.305 s）。旧 `dynamic_objects_964` 组 7/14，七条失败与改动前 r81 广跑日志的同名失败一致，均在源码绑定阶段。 `av/r82-av-case.log` SHA256 `40909C026949FC6D88B85E35AFFFD56B3A352EAAB69DEEB3876FB7D9C6711A2C`。
- 029 生产 Debug 准备 48.398 s，原入口越过动态显示校验，10.947 s 退出 4，`mir.use-after-move` 指向发布后继续用于分页的 `vlist`。 `av/r82-029-window.log` SHA256 `EB40173E073239169CA00234E0B6DEEBA3465F13380CAC411EC78CCC54D1DA0C`。
- AutoUI `1ad8a8cd` 先从排序后的列表生成当前页，再把完整列表移入模型并公布总数。原入口使用同一生产二进制越过分页列表校验，11.414 s 退出 4。首拒为 `mir.use-after-move`，处理器返回模型时聚合读取已移出的 `__age_s_0_photos`。恢复点为保留原目录和收藏模型的所有权，完成列表来源切换的定向验证。 `av/r83-029-window.log` SHA256 `6BC99328E6C2A4CA46E1D16D6752F075BCD33D2F1BC3C1E4D797E68CAB9B90BB`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 83 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `4bfd3b214` / auto-ui `1ad8a8cd` | next: work/T-02–T-03。

### 2026-11-23 goal work rounds 84–85 — 查询通过加载并进入会话初始化

- AutoUI `3e3e4926` 将筛选、按 mtime 稳定排序和分页投影放到受检函数中。查询和选择读取目录或收藏记录并保留模型列表。用真实作者函数与现有源准备入口跑定向 AV，核相同 mtime 的顺序、查询大小写、升降序、分页边界、资源 URI 和输入列表仍可读。最终 Debug 准备 14.966 s，执行 0.236 s，输出 `acb/bac/1/0/c/20/ALPHA 同日/photo:c//c/4`。`scratch/r85-gallery-rows-final.log` SHA256 `47791668CDEDBEBCADD49AB5C7BA0FC8B894F861C1071F91BA04AAEDB981F460`。
- 029 去除模型列表移动后，查询处理器初次需约 32,234,872 字节验证内存。拆出排序后约 17,750,128 字节，再拆分页后通过既有 16,777,216 字节预算。中间顶部函数的索引赋值形曾在包投影时触发语法回退，最终排序以现有索引移除保留并列顺序。`av/r84-029-window.log`、`av/r85-029-window-r4.log`。
- 029 原入口使用 round 82 生产二进制越过全部加载检查，进入会话 `open` 后在 11.298 s 退出 5。首错误为收藏恢复 JSON 解析空文本（`json.parse: unexpected byte None at 0`）。现有桌面准备把源 `storage.get` 改写为初值为空的模型字段，Vue 原包入口又拒裸存储调用。恢复点为消费既有宿主偏好绑定，并处理两端恢复相对 `Init` 的顺序。尚无成功窗口收据。`av/r85-029-window-r5.log` SHA256 `8DEB6AB40C12693342C4C3715228E850461E0E36211DD176CB8BF1DDA091CF58`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 85 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `4bfd3b214` / auto-ui `3e3e4926` | next: work/T-02–T-03。

### 2026-11-23 goal work round 86 — 收藏绑定与 HTTP 接缝探查

- 收藏绑定试稿将同一个 `photo-gallery.favs` 存储键交给既有宿主偏好绑定，以一次启动计时调度首扫。Iced 是 Init 后恢复，Vue 是 Init 前恢复，两端均在初帧之后启动计时。试稿在定向隔离存储路径下绕过空收藏解析并建立了原 029 Iced 会话，录得 2 帧，17.635 s 后因 4 帧门退出 6。
- 窗口收据内 144 次同一首扫事件失败，未提交派发。首错误是 `http.client: no http provider installed in this session lane (plan108 M1 fail-closed)`。首扫错误回滚启动标志，试稿会反复重试，故将试稿从工作树收回并保存在 `scratch/r86-029-preference-prototype.at`（SHA256 `4CDACFB78584F6B91F7D5E69CFDAD960C12289FCC5CE47CAED3CC178C8F77586`）。`av/r86-029-window.json` SHA256 `C328FF3F610A910F82BE2779F2206204210E1ECEB119AF86CDA5A6A55F2A453B`。
- 同一收藏试稿经现有 `generate_linked_front_sfc` 越过裸存储拒绝，029 与 031 现在均报 `source.module-missing: unknown module auto`。探查命令正常结束（2.685 s），两条生成结果均失败。`scratch/r86-linked-vue.log` SHA256 `27B29F56FF084F3703514D62E8BF79B128190482CCC3D9803106A2574C460271`。
- 接续点为消费 PLAN-108 已有的 HTTP 挂起、轮询和恢复执行入口。现有注入仅覆盖桌面单组件会话，受检包应在自己的挂载代次接入宿主提供器及已配置的同源地址，并核关闭后的迟到结果。另需在 Vue 包链接上核 JSON 导入，及恢复空值、有效值和非法值的启动表现。本轮帧数只证明窗口建立，V-03 仍需实际原行为验收。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 86 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `4bfd3b214` / auto-ui `3e3e4926` | next: work/T-02–T-03。

### 2026-11-23 goal work round 87 — 受检包前门复用 JSON 导入准备

- AutoUI `bcbd4be7` 将组件依赖已有的裸 `use auto.json` 处理复用于包入口及项目模块。保留源跨度和显式标准库 JSON 导入。Debug 准备 49.648 s，声明样式依赖的两端入口定向回归 1/1（0.310 s），路由模块闭包回归 1/1（0.472 s）。`av/r87-linked-front-test.log` SHA256 `060FB2D586EFFDC5934287FB2C3B2851F40CA0B5DCCF1B5D38C64AE1BD74FE42`。
- 实际 Vue 生成探查（准备 14.541 s，运行 3.313 s）越过 `unknown module auto`。029 收藏试稿继续在文本大小写、`split` 和 `replace` 的受检链接上拒绝。031 原入口首拒为 `unknown module back.api`，仍需原媒体授权和会话接线。两条均未发出可构建产物。`scratch/r87-linked-vue.log` SHA256 `5832B8BAD53D7906C037E0330D74C51A0A5074350BCCDFE14FB06D84653A42D7`。
- 已交 `std.json.json_is_valid` 可用于接续核收藏恢复。无效偏好需在解析之前验证，现有 JSON 宿主 I/O 错误不会进入客体 catch。恢复试稿仍在 scratch，结合 HTTP 接缝后复验。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 87 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `4bfd3b214` / auto-ui `bcbd4be7` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 88 — 受检组件 HTTP 挂起与 Vue 效应定位

- AutoLang `3ce49b876` 让受检包会话的宿主工作线程复用 PLAN-108 挂起、轮询和恢复执行入口。提供器和取消标志随一次工作线程操作注入。定向回归核实际挂起两次后收到中文 JSON、只启动一次请求、取消回滚，以及退出注入作用域后仍具名拒绝无提供器请求。Debug 准备 19.120 s，定向用例 1/1（0.349 s），AGE 包 49/49（0.858 s），PLAN-108 HTTP 13/13（0.517 s）。`av/r88-worker-case.log` SHA256 `08C3811140C447ED7106039CE5D44A9E6DE701DE0C862D663223CF9DEB3B3F`。
- 029 收藏试稿复用现有桌面文本准备做 A2Vue 诊断，越过文本方法校验，在 `auto.http.get-json.v1` 被计算面拒绝（10.696 s）。诊断副本用同型的 `json.parse("{}")` 替换 HTTP 表达式后，首拒改为 `auto.json.parse.v1`（16.982 s）。此对照确认 HTTP 之外，Vue 还需 JSON 值运算。两份均无 SFC 产物，也未替换原项目的扫描。 `scratch/r88-linked-vue-prepared.log`、`scratch/r88-linked-vue-compute.log`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 88 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `3ce49b876` / auto-ui `bcbd4be7` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 89 — 真实扫描到达 Iced 包会话

- AutoLang `0f42de25c` 按保留的受检指令决定是否需要 HTTP 工作线程。AutoUI `999765a9` 接入按挂载代次提交的 Iced HTTP 工作线程。等待期间使用上次已发布帧，待处理视图事件按原有帧身份检查，再由当前代次接收结果。相对 URI 复用已有宿主基址解析。
- Iced 定向回归以真实本地 TCP 请求核首次派发不阻塞、旧代关闭后响应不发布、新代中文响应提交一次。最终 Debug 准备 84.315 s，定向 1/1（0.296 s）。根及子组件计时器回归各 1/1（3.384 s、3.862 s）。初版用例未选中受检挂载入口，改用带禁用计时器的普通受检入口后通过。 `av/r89-http-case-r4.log` SHA256 `E58CE7E304B5C8230FCCCF1BA595D5C77D034C8FD84C43613D51A7D311FAC93C`。
- 以原 029 后端 API 源和 pac 在隔离目录用 `auto service --server rust -B 34429` 启动原 Rust 服务，冷编译 8m32s。按原产品探针放入两张根图与一张子目录图。真实扫描返回 HTTP 200、两条目录直辖图片、一个子目录、`root_missing=false`。 `scratch/r89-photo-service/scan-root.json` SHA256 `30224323EA92CC4EF24990D57A77024CC1F6CE731226822424602B601847889B`。
- 029 用原入口和 round 86 收藏试稿运行实际宿主（最终生产 Debug 准备 105.675 s），通过加载、绘制 35 帧并由宿主脚本关闭（30.312 s，退出 0）。首扫的受检事件仍因 `ObjectMissing` 拒绝，源跨度 177376–177383，相同错误 11 次、无派发提交。退出码只证明帧数与脚本结束。同入口宿主轮询探针在打开后 0.859 s 复现，帧版本仍为 1。 `av/r89-029-window-r3.json` SHA256 `B60BB5115AA14A3B8519350A848E03498BE3F6D5E882615299961C0E5C27D239`。
- 保留真实 HTTP 和扫描转换，只将首扫的 `.GalApply()` 换成诊断性计数后，受检提交得到 `total=2`、`root_missing=false`、`load_error=""`，打开后 1.566 s 提交。这把拒绝定位在查询发布或其视图更新，首扫对照只作为定位证据。 `scratch/r89-029-scan-only.log` SHA256 `A91143B19094C0EAC527EE5C2A78149BDDEA82BB8D2F00DF3D7421CB2D303D85`。
- 将扫描局部列表显式声明为 `List` 的对照仍在同一调用上触发 `ObjectMissing`，跨度随增加的十个源码字节平移。省略分类发布的对照更早触发 `no local named self`，未能判定分类视图。继续沿原入口解析运行时函数和字段访问的跨度。两个诊断变体均在 scratch，原收藏试稿仍需整条查询通过后复验。 `scratch/r89-029-typed-scan-lists.log` SHA256 `6BAF8BA7698FB3D9ADFDB28353F6D1E9E3974B477EE8F2DC089CC9A5741FF604`。
- 收藏恢复独立 AV 探针通过现有 `std.json.json_is_valid` 与数组首标记核空值、损坏 JSON、非数组对象和有效中文记录（0.277 s）。直接使用 `json_kind_at(value, "")` 的对照没有恢复有效数组。此校验尚未进原项目。 `scratch/r90-preference-validation-r2.log` SHA256 `04D062019F4630B6C4B367BDE1A3D1F7AFB0666ABD23DCC296512661B13B3E81`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 89 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `999765a9` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 90 — 真实目录与空态使用同一行结构

- 临时 AV 诊断在原 029 上定位到 TreeView 的视图片段请求 `empty`，当前普通行的键包含 `depth`，缺少 `empty`。AutoUI `25f20411` 为普通行补 `empty: false`，给空树哨兵补 `depth: 0`。临时诊断已从工作树清除。 `av/r90-object-miss-window.log` SHA256 `AD3FAF007DC3DAF8E452CE1CE1906DD1F3CAA63B1089A03B697ADDD26F7C8C3A`。
- TreeView 原作者工具经 AV 验证普通父子行的标记和深度、空树哨兵、Unicode ID 和层级导航。Debug 准备 166.337 s，定向 1/1（0.402 s），AssetBrowser 记录验证与 029 依赖发现各 1/1（0.201 s、1.287 s）。 `av/r90-tree-rows.log` SHA256 `D9FD29E2B72B772F35C27AEEB163C80A89D16177C13D5C121934697C37FFBEC3`。
- 用真正 Rust 后端复跑 029 原入口和收藏试稿。窗口绘制 12 帧，完整首扫提交 `total=2`，状态文案为“2 项内容 · 全部 · 第 1 页”，无运行时错误，宿主脚本关闭（23.131 s，退出 0）。同一后端的原图和缩略图均返回 HTTP 200，原图 SHA256 与冻结夹具一致。收藏恢复和查看器的交互证据待接续。 `av/r90-tree-shape-window.json` SHA256 `A6EA12D3F47A0D396D8582BF089C67F5D200DEC4D40F13FAAD4803B61B5DACC5`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 90 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `25f20411` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 91 — 原 029 恢复收藏后执行首扫

- AutoUI `36c66449` 将原 `photo-gallery.favs` 键接入受检宿主偏好绑定。首个 Iced 计时事件在恢复偏好后启动原目录扫描。收藏恢复只解析语法有效的数组，保留已有收藏记录的序列化格式。受检包准备保留显式 `std.json` 导入，以复用现有校验。最终生产 Debug 准备 96.222 s。 `av/r91-gallery-window-build.log`。
- 用原 029 后端和干净生产二进制打开实际 Iced 窗口，隔离偏好含损坏的 `[broken`。窗口在空收藏下完成原首扫，提交 `total=2`，绘制 42 帧，无运行时错误，脚本关闭（48.480 s，退出 0）。 `av/r91-restore-solo.json` SHA256 `664B2A8AAC786C97E1266136370455739CA160F03B4725722A196DCF9893AA83`。
- 同一原入口的宿主探针也完成无偏好文件的首扫（24.682 s），以及含一条真实图片记录的偏好恢复和首扫（27.582 s）。有效收藏在首扫前后保持原记录。后一条探针随后调用当前帧的 `onclick` 照片选择回调，宿主提交了 `ItemInput`，当时根组件的 `selection` 仍为空，探针退出 101。查看器打开尚待验证。 `scratch/r91-gallery-fresh.log` SHA256 `0B45C339C9BA026CAC8A7AABC93950EA0DF9EA7E6F9A42585BB75A8FEAA05985`，`scratch/r91-gallery-callbacks-r3.log` SHA256 `8C3D303BD9E994155AAA2C824DD071101C00F0DDDDB2657081CCDB4230AEBCC0`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 91 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `36c66449` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 92 — 029 选择事件抵达内嵌查看器

- 隔离源追踪确认 CollectionView 发送了 `scopeId=q1`、`baseRevision=100` 和当前图片身份。AssetBrowser 收到后在 scope 和 revision 校验拒绝。受检解析器将含任一 `Dyn` 的 `==` 和 `!=` 路由到判空；声明 `str` 和 `int` 的对照通过这一层，并定位根相册的第二处身份比较。 `scratch/r92-gallery-predicates.log` SHA256 `6040B388229F23AF8FCE14B9C407D9E1B61F77C13D959E40E755DE558037E650`，`scratch/r92-gallery-typed.log` SHA256 `B003B2A0B291FB33F2E1261E0C9294B3C998F5D3608164721B93DA8D91F78EBA`。
- AutoUI `7be164c2` 在 AssetBrowser 选择校验和 029 的选择及打开路径按声明类型读取字段。原项目和原 Rust 后端复跑，宿主用当前帧按钮回调选中 `corner-marked`，打开原图视图，确认 `image_surface.src` 指向同一身份的真实 `/api/photos/full/` URI，返回图库并关闭会话（30.415 s，退出 0）。对应 URI 返回 200，原图 SHA256 与夹具一致，未知图片身份返回 404。真实窗口内的解码与缺图恢复待验。 `scratch/r92-gallery-original-callbacks.log` SHA256 `6F0AD9AAB50AE0FA0836A0949ED72B235CB7DC14CA545FB6EE9E8088DD45805F`，`scratch/r92-gallery-http.log` SHA256 `0CD7F54F670FE2B784F0F3552657D9494A44300677F5BBA16E70C51C40990AEC`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 92 | from: work | to: work | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `7be164c2` | next: work/T-02–T-03 / T-06。

### 2026-11-23 独立交付 — 118 消费源准备修复

- 复审范围为用户批准拆出的源准备修复。当前会话核对提交、独立断言和原始收据。隔离交付树 `D:/autostack/.wt/fix-prep-118/auto-ui-landing`、分支 `codex/fix-prep-118-land`，基线 `01eb8f4f59de3db4020ff801144be9ea0528de9b`，AutoLang 证据输入 `dca52e196a64f123ca057e1442c71c8f3e95ab30`。交付提交 `3f4f9b3f4b25cc61aa3c2ad4c8528aaa2a215d50` 的生产源与 `7c7ef158` 一致。复审补齐随附诊断测试的 `ace-bridge` 门控。该修复维持现有组件包导入和样式字面化合同，无单独 Spec 增量。
- 复用生产源一致的 118 Debug 库，直接断言验证保留包导入、样式引用换成字面量，以及含闭括号的注释、转义字符串和嵌套记录之后正确注入宿主字段。最终探针准备 7.398 s，执行 0.044 s。默认条件下测试探针准备 0.127 s、发现 0 项（0.061 s）。`ace-bridge` 诊断探针准备 16.059 s，四项执行分别为 0.398、0.106、0.237、0.264 s。三个探针打印原始准入诊断，其中未准备的 029 源仍报告 `caption_text` 未定义。命令结果和收据在 `scratch/prep-phase-review/`。
- 原 031-paint 源哈希与 118 收据一致。复放组包验证保留 `color_picker` 导入并进入下一处 `source.unsupported` 准入（`fn` 当前 profile 不支持）。组包诊断耗时 0.262 s，诊断程序退出 101。原 A2Vue 前后收据仍逐字节一致，SHA256 `73DA630642300CEF46818957AF61A4AFE110A65420CD0CB936D990BE1A1052FE`。AV 复放 `scratch/prep-phase-review/paint-full-replay.log` SHA256 `9EBAE93717C24FA2097C229FB5737E312BC4844ED60BD874899A002D4297ADF9`。
- stage: review/merge | review_scope: phase | plan_id: PLAN-117 | plan_revision: r1 | outcome: pass/landed | local master: `3f4f9b3f` fast-forward | consumer: PLAN-118。诊断提交 `0cc47f5d` 留在验证分支。117 整体验收状态沿用上一工作记录。

### 2026-11-23 goal work round 93 — 029 真窗解码与缺图恢复

- AutoUI `17e6de20` 将受检桌面视图中配置后端同源的 HTTP 图片交给现有异步请求和有界媒体解码管线。视图切换时释放当前资源，加载和失败事件按同一资源及当前受检控件重新定位。后台任务期间收到的普通点击可在同一控件仍有效时提交。首轮扩展普通点击保留使既有双击反例多出一次选择（退出 101），最终保留条件收窄后复验通过。
- 用原 029 入口和原 Rust 后端，通过真实 Iced 窗口的控件回调选图、打开查看器和切换图片。两张缩略图已解码。原图 `scenic-320x240` 解码为 320×240 并提交 `OnLoaded`。移开隔离夹具的第二张原图后收到真实 404 并提交 `OnError`。切回第一张可加载，放回第二张后重试解码为 64×48 并提交 `OnLoaded`，返回图库后会话关闭。窗口报告 1008 帧、零运行时错误（42.381 s，退出 0）。同一入口从第二个原后端连接解码 12/12 张缩略图（548 帧，32.060 s，退出 0）。`scratch/r93-photo-window/media-final.json` SHA256 `A3346E956453A4E6CEEF0EC49CC7C3154595EDE7D2C1FCD8A7FD2B04E002AF54`，`media-batch-12.json` SHA256 `8828B2B0F35CBD0D0B03B5C8ADD015D0D3A680166F94C53D9F7EE108C5DDA0DD`。
- Debug 集成准备 82.679 s，真窗探针连接最终库另计 8.673 s。现有 `plan114_widget_packages` 定向验证四项，HTTP 当前挂载与延迟点击 0.304 s，受检图像手势 0.325 s，双击与替换条目 2.520 s，集合回调与过期 JSON 2.103 s，均退出 0。 `scratch/r93-photo-window/targeted-regressions-final.log` SHA256 `C70D32F4A1669F716A61B21427B5C596968C0383E7D35018A3D36B9074E18650`。
- 当前源码重放生产 A2Vue 包前门（Debug 准备 4.859 s）。029 原源遇到 `lower`、`split`、`replace` 链接拒绝，031 仍拒 `back.api`（原源合计 3.534 s）。029 诊断额外经过桌面文本准备后，依然在 `auto.http.get-json.v1` 的计算面准入处拒绝（16.741 s）。两个探针退出 0 只表示完成诊断，尚无可构建 SFC。 `scratch/r93-linked-vue-original.log` SHA256 `F1915589F7FEC6F7C101A5D76A0FA4368EA8E590FCAFBD587AC61424CCF8E1B6`，`scratch/r93-linked-vue-prepared.log` SHA256 `0561F33C52D252F5EF284ECC8595CC0CA890E831D97C1FAC36C34489D84F00B0`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 93 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `17e6de20` | next: work/T-02–T-03 / T-06。真实窗口的回放使用宿主回调。031 带授权的媒体会话、Carousel 的物理输入和 Vue 原项目验收仍待完成。

### 2026-11-23 goal work round 94 — 029 目录导航与搜索提交

- 真窗基线的 `WantSelect("dir:trip-2024")` 已抵达原相册，合法查询被误判为“代次过期”，目录仍在根，结果 2 项（22.332 s，退出 6）。AutoUI `38210f00` 在查询比较前将代次、序列和筛选字段读为声明类型。导航按前缀和目录分隔符处理路径，替换 `split` 和 `replace`，返回上级时刷新查询 ID 和版本，并重置页码与选中状态。两次测试基于相同 029 生产宿主与原 Rust 后端。
- 真窗以现有控件回调进入 `trip-2024` 得到 1 项，返回根目录生成新查询并恢复 2 项，搜索 `SCENIC` 得到 1 项（Debug 探针准备 15.558 s，执行 29.213 s，退出 0）。独立目录夹具验证名为 `root` 的普通目录、其子目录 `root/相册`、逐级返回与拒绝旧查询版本（准备 5.973 s，执行 41.871 s，退出 0）。 `scratch/r94-gallery-query-r1.json` SHA256 `A032C01C88588C3E4EA4EE5B08FB1DBD5D958C58E1F03E2814B3764DA6AED918`，`scratch/r94-gallery-hierarchy.json` SHA256 `17849351BCD59D9C6C0FC1D5CD199EF627A0C03B2FA0869E09EED247EDB282EB`。
- 层级回放记录一条切换目录时的缩略图 404。独立调用原后端验证同一令牌在父目录扫描后返回 200，改扫子目录后返回 404，重新扫描父目录后恢复 200，原文件始终存在（0.200 s）。`auto-media-core/src/photo_service.rs` 按根整体替换令牌表。待由 PhotoService 写面核对跨目录浏览和跨目录收藏的令牌生命周期。 `scratch/r94-gallery-token-lifetime.log` SHA256 `742E513F09C3EB67B581A258043CCB0B2FF7F002AE2BB5A937CB93A2F50AACC0`。
- 原 029 源直调 A2Vue 生产包入口，`split` 和 `replace` 的拒绝消失，剩余文本拒绝对上 `lower` 及其连带类型诊断（2.647 s）。对照经过现有桌面文本准备后仍在 `auto.http.get-json.v1` 的计算面准入处拒绝（15.871 s）。两次执行均完成诊断，尚无 SFC。 `scratch/r94-linked-vue-path.log` SHA256 `4959005AB355869A69DC2F31D84DE2C866F7475B227D041C885101D0EC8AAF2E`，`scratch/r94-linked-vue-prepared.log` SHA256 `0561F33C52D252F5EF284ECC8595CC0CA890E831D97C1FAC36C34489D84F00B0`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 94 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `38210f00` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 95 — 029 收藏持久化与查看器选源

- 同一原 029 入口、Iced 真窗和原 Rust 后端的基线收到 `ToggleFavSelected` 后，收藏仍为空（33.338 s，退出 6）。保留 `photos.fav` 并原位更新的尝试在接下来读取 `photo.title` 时触发 `DynTypeMismatch`。AutoUI `1e9e8cd9` 统一用 `fav_records` 保存收藏，以类型明确的照片 ID 比较成员关系，继续沿原 JSON 和宿主偏好路径读写。查看器按当前结果行选取照片的 ID、标题和资源，取消收藏可直接找到已存记录。
- 三次独立进程依次提交收藏并核对磁盘（31.930 s）、重开并解码收藏原图后从查看器取消收藏（34.679 s）、再次重开验证收藏为空（23.940 s），均退出 0。使用实际窗口控件回调和同一隔离偏好文件。证据目录 `scratch/r95-gallery-favorites/`，`r2-add.json` SHA256 `44721F462B86D0E4DB1683C598C81F2F8CF0FA46FE62952DA6BDD546F37491D1`，`r2-remove.json` SHA256 `1FBE54A1C083F46BC7A55629DAD34F9652C026B8B294DF548A70251A782A0612`，`r2-empty.json` SHA256 `55F38B464E930FBA15AEC8FB83DF14440FCEC00FC4101E4AC2E964408F191A82`。
- 从根目录打开仅存在于收藏中的子目录照片，实际查看器投影已保存的 ID、标题和原图 URI，宿主收到该 URI 的 404 并提交 `OnError`。该结果与 round 94 记录的令牌生命周期一致。查看器取消收藏成功，返回收藏列表为空（准备 8.956 s，执行 43.017 s，退出 0）。`detached-r2.json` SHA256 `2AD43F0CFCC542D009B0423F71A9F53A520E779CEF4EA56F039E6AD896939720`。
- 在同一代码上重放原图缺失与恢复，依次观察 320×240 正常解码、真实 404、换回正常图、64×48 恢复解码、返回图库及关闭会话。夹具已复原，零运行时错误（40.346 s，退出 0）。`media-regression.json` SHA256 `6D3D53B0B4392093F33D124B063524D36B114827E48E2732D64B02FC67176360`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 95 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `1e9e8cd9` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 96 — Carousel 真窗渲染与鼠标拖动

- 原 AutoOS `cdb4883` 画廊经受检 Iced 宿主启动，Win32 鼠标可以从侧栏进入 `/carousel`。基线观察到 Basic、Sizes、Spacing 和 Options 的位置状态随按钮变化，幻灯片内容空白。真窗收据 `scratch/r95-carousel-physical/observed.json` SHA256 `9EB8700DB063C86DD72FF4C9DE5F2BF272C2BB84E806544FE8BD0DB195C41CB0`。画廊首页的 `expected fn, found widget` 本轮仍可见，Carousel 独立路由成功挂载。
- AutoUI `5026fdff` 使横向轮播项按内容取高，纵向固定轨道给可见项分配明确高度。仅 Carousel 启用 Iced 按下与释放位置的采样，保留鼠标移动限频。该选项在视图转换时随事件回调传递。仅修高度的真窗已看见五组幻灯片，并验证了按钮、方向键和首尾循环。完整补丁的 Release 位置跟踪也经真实鼠标拖动复验。
- 最终真窗观察到 Basic 横向左右拖动与方向键往返、Sizes 和 Spacing 按钮往返、Orientation 向上及向下拖动和方向键往返，Orientation 忽略右方向键。Options 从 `1,2,3` 循环至 `5,1,2`，经 `/avatar` 离页后重进恢复 `1,2,3`。两次向下拖动未翻页，第三次长距离拖动成功，纵向输入稳定性继续保留为复核项。退出时 6,725 帧、四个会话均关闭、零运行时错误、退出 0。受检库准备 45.646 s，观察程序链接 15.878 s。 `drag-final.json` SHA256 `C87B328FD562E7E66F5F8AB16A7DBA8BAFE4F66ED9561BCC20A28BE21353D938`。
- Debug 定向回归覆盖滑项高度及回调映射、可见项与导航，2/2 通过（编译和执行共 169.364 s）。既有指针区坐标、限频与去重测试 6/6 通过（1.700 s）。`carousel-tests-final-r2.log` SHA256 `DFF5389306F7498EEF15907561AB69E3AA7CE511E48B44C89EED4A74585FECCC`，`pointer-tests-final-r2.log` SHA256 `3E536FEFE1C7F41319D09336D92CD9EA45E2BE8175E97CCEE2BAFBA5747AC3E1`。
- 核对 PLAN-118 移交的最小 prep 补丁 `7c7ef158`（等同 `299ac88d`）已进入 AutoUI master，落主提交 `3f4f9b3f` 给 T-01 探针增加 `ace-bridge` feature 条件。独立验证探针 `0cc47f5d` 留在 `fix-prep-118`。补丁使用现有 style 声明处理。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 96 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `5026fdff` | next: work/T-02–T-03 / T-06。

### 2026-11-23 goal work round 97 — A2Vue 图片视口输入与原入口准入

- 029 和 031 当前原源经生产 `generate_linked_front_sfc` 分别重现 `lower` 的类型拒绝和 `source.module-missing: unknown module back.api`（双例 3.261 s）。029 隔离源将标题读入明确的 `str` 局部并复用 AV 的大小写助手，保留原 `.contains` 即推进到 `auto.http.get-json.v1` 的计算面准入（16.890 s）。完整桌面 prep 重放得到相同拒绝（15.810 s）。仅为诊断替换单处 HTTP 调用后，下一项为 `auto.json.parse.v1`（16.045 s）。上述诊断退出 0 表示探针正常收集报错。证据目录 `scratch/r97-vue-admission/`，`original-baseline.log` SHA256 `9B7677FD34DFB4D4E590620D696C68EA8B2954981C39B6F44B5107B3D06173F2`，`text-typed.log` 和 `prepared-current.log` SHA256 `EB594550DE3C7F8BBD25894F641E335F7801D50553F9CB45D3B90FFB84D5A907`，`no-http-diagnostic.log` SHA256 `0179C3907816660163261AE73B58522EF487E6A2BEC0EF5F70C61C97A929B4BD`。PLAN-108 的 HTTP 承载规范覆盖组件面 call-clean 条件，029 当前调用的是带子组件的受检包面。
- 同一受检 `image_surface` 夹具经生产生成器和 Vue SFC 编译器挂载，基线浏览器报 `ace.widget.view: unknown operation 'image-surface-event'`，图片节点缺席（生成和浏览器合计 1.571 s，退出 0 验证预期失败）。AutoUI `bb0b3708` 接收包面已有的 wheel/pan 记录，检查消息和载荷类型，用浏览器指针捕获投递 start/move/end。DOM 图像在裁切区应用 fit、缩放、偏移、旋转及 filter，按 URI 和资源身份更新节点，加载与错误沿当前投影回调投递。
- 在最终生产库（49.949 s，退出 0）和生成器（链接 13.570 s，退出 0）上，同一夹具冷生成 154,439 字节 SFC，经 Vue 编译器和 Chrome 挂载，观察 64×48 图像解码、320×240 裁切区、真实滚轮放大至 1.25、拖动 30×20、双击复位、坏图错误和换图复原。迟到的坏图在卸载并重建视口后没有增加错误计数。终态加载 3 次、错误 1 次、拖动更新 1 次、页面异常 0、关闭浏览器和回环服务（生成与执行共 2.555 s，退出 0）。`browser-baseline-final.log` SHA256 `15B8854E52E826E868A13B49FC3EC39603AC65B472488E1CC46E34BA64195EE9`，`browser-current-r3.log` SHA256 `BDA3218ECD94E6F6D3660200CA443F5AED0875D7DD7D86EA0023121ED467BA2A`，`after.vue` SHA256 `9782439EE3C2C9F7424DCFBB2B3A4575F5608188211582548A29EEB86DB65238`。
- 用正式 `packages/image-viewer` 导入未改写的 `ImageViewport`，当前生产包 SFC 入口明确拒绝其 `auto.json.stringify.v1`，尚不能生成原组件（1.686 s，探针因拒绝退出 101）。`official-current.log` SHA256 `2F2B3DC1B53421AF0380492C283A28D6C7697B41CB590630865025E570816FD2`。同 URI 换版本的重新读取仍待消费既有浏览器媒体重验证机制。029/031 的原项目浏览器验收继续保留。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 97 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `bb0b3708` | next: work/T-06 继续核对原包 JSON 值与 HTTP 承载；T-02–T-03 仍含 031 授权媒体绑定。

### 2026-11-23 goal work round 98 — 正式 ImageViewport 生成与受控视图复位

- 原 ImageViewport 经生产包入口仍在 `auto.json.stringify.v1` 被拒绝（1.715 s，退出 101）。AutoUI `cbbc6bec` 复用标准库 `json_str` 转义资源身份、错误文字等字符串字段，按组件既有结构编码四类事件。变换事件的嵌套 tuple 与平面别名共用一份编码。序列化接通后的首轮 Chrome 已通过加载、导航、滚轮和拖动，到复位请求超时（3.695 s，退出 1）。同资源接收新的受控 zoom 时，Vue 调用 `Init` 覆盖了复位基线。最终 `Init` 和 `Tick` 共用同步逻辑，仅首次初始化或资源身份变化时捕获基线并清理加载与手势状态。
- 使用直接导入工作树正式组件的父组件夹具，生产 `generate_linked_front_sfc` 生成 542,436 字节（嵌入原组件一件，2.001 s，退出 0），Vite 构建 1.932 s，真实 Chrome 运行 1.890 s，均退出 0。验证 64×48 图像、带引号、反斜线及换行的 Unicode 资源 ID、下一张、滚轮缩放 1.25、父组件接受后 24×−16 拖动、复位回 1.0、切到 320×240 新图后重新发 loaded、坏图报 `load_failed` 及卸载，Chrome 页面异常 0。产物目录 `scratch/r98-official-image/`，`Host.vue` SHA256 `09F2AC48F3F46B456B5F67A471E22EB9844383A20425EEC123F9709D663E4BB2`，`browser-r2.log` SHA256 `ACEA7BE012FE5A308DAE661A876FAE28FC12C422752B4009C76008731D3AB7C3`。
- 永久定向用例 `checked_vue_embeds_official_image_viewport` 验证生产入口嵌入仓内正式文件，使用受检库链接 12.406 s，执行 1.748 s、1/1 通过。原 029 同源 Iced 窗口和原 Rust 后端用隔离夹具复验首图解码、真实 404、换回及缺图复原、图库返回与会话关闭（23.603 s，退出 0，514 帧、零运行时错误）。`viewport-regression.log` SHA256 `DC319E815D97BA76E2C8AB02E299B526E4EECF5325B1588A845D2157A83F6EDF`，`gallery-final.json` SHA256 `E8C981157AA5E2D42A7F3470DE88182EA97C14C7164BB0E26DEDD1FCF0B04500`。
- 非法初始 fit 的对照 SFC 可以生成和构建。Chrome 负例两次导航超时（22.891 s、45.991 s，退出 1），未获得浏览器终态。独立执行同一生成的 setup，正常配置返回组件（0.555 s），非法配置触发 `ace.widget.init: outbound events require a mounted parent`（0.170 s），定位到初始化时上报 `invalid_view` 的宿主限制。`setup-invalid.log` SHA256 `2F7AB263EB0E4EB11BF3EB692F68E523AF5D8B3FA92FC3844BDB2B04B3A0BF7E`，`browser-invalid-r3.log` SHA256 `57E4D497500B48517DF28004497FE000560A12B2BF277D2D703262DBAC85E972`。同 URI 换版本重新读取、029 的受检文本及 HTTP/JSON、031 的带授权后端和两个原项目浏览器验收仍需交付。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 98 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `cbbc6bec` | next: work/T-06 核对初始化事件投递与版本化重读，继续原项目准入；T-02–T-03 仍含 031 授权媒体绑定。

### 2026-11-23 goal work round 99 — 正式 ImageViewport 同 URI 换版重读

- 直接导入原 ImageViewport 的生产 checked SFC 使用带缓存响应的回环夹具。基线在同 URL 从 `rev-1` 更新到 `rev-2` 后，上报新版本 loaded，Chrome 仍解码旧图 64×48，请求日志只有一次 `/marked.png`（生成 2.120 s，Vite 构建 6.704 s，Chrome 1.205 s，均退出 0，浏览器用例核对预期复现）。
- AutoUI `eab845c3` 将既有 Vue 图片重验证指令的 JavaScript/TypeScript 发射收拢到同一实现。受检 `image_surface` 使用 VNode 挂载、更新和卸载回调，沿原 URI 重新读取有版本的图片。保留 URI 与版本都未变时的加载状态。旧版生成器的 TypeScript 指令与基线 2,116 字节逐字相同。
- 以最终受检生产库和真实 Chrome 验证同 URL 重读得到 320×240，普通受控缩放保持同一对象 URL 且无额外请求。404 正确回传，过期或卸载的两次慢读取均在服务端观察到取消，三条旧对象 URL 均无法再读取，迟到响应未覆盖错误状态，浏览器和回环服务关闭。受检库准备 80.207 s，生成工具链接 10.804 s，测试 SFC 生成 1.865 s，Vite 构建 3.249 s，最终 Chrome 用例执行 1.331 s，均退出 0。日志 `scratch/r99-versioned-image/browser-before.log` SHA256 `81153B97E816CC519B9271706C49AFA874BABE35589607016973B3C3F38DC3BA`，`browser-final.log` SHA256 `1A1EED27F77CA4F64355ECA7CF9C8799A18D70866D2B4ACCED23891A9774195C`。
- 正式组件的导航、缩放、拖动、复位、换图、坏图和卸载回归仍通过（SFC 生成 1.862 s，Vite 构建 1.613 s，Chrome 执行 1.226 s）。永久生成用例 `checked_vue_embeds_official_image_viewport` 1/1 通过（链接 10.871 s，执行 1.777 s）。`gesture-browser.log` SHA256 `0E574F23E79454103D32D4A09B02C6FF5E77FBB83C683CA3CAEC92C8D5B72E29`，`viewport-regression.log` SHA256 `BB104A8CC40412B3325E2DAC06A82D73867CAF20F2E608807B39CA3DA9261B47`，`emitter-compat.log` SHA256 `32CC8AFC950D7EAA9B8F1F90CC569AE05C75FF8F0BF0B39C8B06DBB9E84A8F84`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 99 | status: executing | outcome: partial | code_commit: auto-lang `0f42de25c` / auto-ui `eab845c3` | next: work/T-06 核对初始化错误投递及原项目准入；T-02–T-03 仍含 031 授权媒体绑定。

2026-11-23 T-06 续（round 100）：正式 ImageViewport 的初始化错误回传与配置恢复。

- 以两处导入原 ImageViewport 的受检生产 SFC 对照验证。`eab845c3` 的 Chrome 基线复现 `ace.widget.init: outbound events require a mounted parent`，父组件的错误回执 DOM 未创建，图片请求为零。生成 2.103 s，Vite 构建 2.298 s，Chrome 复现 0.920 s，均退出 0（用例断言旧版错误）。`scratch/r100-init-events/baseline-chrome.log` SHA256 `96746A25739DE57B936E044FB8DB441B0C9220ACA0DE821069591A1E82F8D2B1`。
- AutoUI `77fbdc87` 在受检 Vue 提交视图后通过现有类型校验和消息事务批量投递 `Init` 的父组件回调。按实例路径与输入保留待送事件，卸载实例丢弃该事件。一批消息及其跨重绘的初始化级联共享现有的 4,096 次转移上限。目录读取存在未提交状态时，投递等待该状态结算。
- 真实 Chrome 中，两个子组件初次挂载各回传一次带资源 ID 和版本的 `invalid_view`，错误配置时不发起图片请求。第一次修正后解码 64×48 的图片并回传 `loaded`，重新引入错误与换代时追加一次错误，第二次修正后加载新版。两次普通重绘均未重发，浏览器无脚本错误。受检库准备 80.785 s，生成工具链接 9.719 s，SFC 生成 2.134 s，Vite 构建 1.925 s，Chrome 执行 1.149 s，均退出 0。`final-chrome.log` SHA256 `C6F4F4BFA5231A4756D7F6A58BCDA38CE4E9ED54C3E01FD50EBC1CB1987A0F2B`，同目录 `host.at` 和 `verify-init.mjs` 可复验。
- 原组件的导航、缩放、拖动、复位、换图及坏图浏览器回归通过（生成 2.113 s，构建 1.909 s，Chrome 1.781 s）。同 URL 版本重读、卸载取消及对象 URL 回收回归通过（生成 2.137 s，构建 1.927 s，Chrome 1.633 s）。`checked_vue_embeds_official_image_viewport` 1/1 通过（链接 4.405 s，执行 1.771 s）。`node --check` 和 `git diff --check` 通过。工作区 `cargo fmt --all -- --check` 遇到 Windows 错误 206，限定 `auto-ui-runtime` 后报告已有 Rust 格式差异。本次代码改动只涉及 JavaScript。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 100 | from: work/T-06 | to: work | status: executing | outcome: partial | worktree: `D:/autostack/.wt/ui-117/auto-ui` | commit: auto-lang `0f42de25c` / auto-ui `77fbdc87` | next: work/T-06 继续核对原 029 A2Vue 的 HTTP/JSON 语义准入；T-02–T-03 仍缺原 031 的授权媒体绑定。

2026-11-23 T-06 续（round 101）：受检脚本语言错误退出与 029 原项目准入复验。

- 受检 HIR 为 `try/catch` 的循环提供 `LoopUnwind`，旧脚本发射忽略该字段。实跑 Node 基线中，被调函数的 `Fail` 越过应有的 catch（源码检查和发射成功，定向用例执行 0.11 s，整体 1.172 s，退出 101）。`scratch/r101-gallery-search/protected-baseline-r4.log` SHA256 `AF82ED0CA13CC50996CF3F537BD3A15CB99E94BDEEE40A97ECB757E9FC9FAE15`。
- AutoLang `15d3d5139` 按已验证的退出信息发射受保护循环，只接住语言 `Fail`，以自有文本传递错误并保留空消息。显式所有权转移在实际执行时清空原槽，修复首版打通 catch 后暴露的释放错误。真实 Node 对生成的 JavaScript 与 TypeScript 都验证正常返回、跨函数 Unicode 和空错误消息捕获、`DivByZero` 传播及文本分配归零。`cargo test -p ace-a2x-ts --locked` 33/33，通过（准备 2.620 s，整命令 3.973 s）。`a2x-ts-final-r3.log` SHA256 `86CFC4452F6068A0143935A9C30073847E1EAA3704AE2825651CAD8FC1B3C07F`。
- AutoUI 生产库准备 83.615 s、生成工具链接 15.521 s。受检 Vue 输出含实际 catch 路径，带原 ImageViewport 的 Chrome 夹具验证成功结果、除零不写入模型及后续点击（SFC 生成 2.101 s，Vite 构建 9.386 s，Chrome 1.954 s，均退出 0）。正式 ImageViewport 的双实例初始化错误与两次修正回归通过（生成 2.156 s，构建 9.379 s，Chrome 1.993 s）。`browser-recovery.log` SHA256 `3B59852C092D5615403E50FDB6C98FD081EA6560E364740858D91EF9E4E058B2`，`init-browser.log` SHA256 `C6F4F4BFA5231A4756D7F6A58BCDA38CE4E9ED54C3E01FD50EBC1CB1987A0F2B`。
- 最终库从 029 原 `app.at` 仍首拒 `Dyn` 与 `Str` 的 `lower`（2.542 s），031 仍拒未绑定的 `back.api`（0.936 s）。029 原 VM 对 `lower()` 使用 Unicode 小写转换，桌面 prep 的 ASCII 助手只适合先前归因探针，保留原搜索合同需复用或实现 Unicode 转换。独立 HTTP 与 JSON 夹具分别在 `auto.http.get-json.v1`（2.049 s）与 `auto.json.parse.v1`（1.848 s）的 `SemanticView` 准入处具名拒绝。四项生成退出 101，无 SFC。`original-029-final.log` SHA256 `2FBADA7E7258AC7639F7B53CD8074B3E9C11EE917075C3D148BB19FF4AB71D36`，`original-031-final.log` SHA256 `4085300F0C8B87DA0D873DC78E814E92BFE7F648EE7B3F26940B207A80A8BFB2`，`http-effect.log` SHA256 `D91601557566D14F01B6A79D87FBBF7ABD1E8C6F276708ABA8BD2AA18278F56F`，`json-effect.log` SHA256 `010035D0D8FA93D0BEEE137745A37A81EE7FC5AC5392DB626D21FF1A975E5BD5`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 101 | from: work/T-06 | to: work | status: executing | outcome: partial | worktree: `D:/autostack/.wt/ui-117/{auto-ui,auto-lang}` | commit: auto-lang `15d3d5139` / auto-ui `77fbdc87` | next: work/T-06 保持 029 的 Unicode 搜索并接通受检包面 HTTP/JSON；T-02–T-03 仍缺 031 授权媒体绑定。

2026-11-23 T-06 续（round 102）：原 029 越过小写转换首拒，Iced 验证 Unicode 搜索。

- AutoLang `737620f8d` 沿 `av.text.case/1` 接入完整字符串的 Unicode `to_lower()`（兼容 `lower()`），覆盖 A2Vue 发射、AV 的指令、字节码和受检配置准入。新操作 `STR_LOWER=0x59` 在 closures 和携带 shared collection arguments 的 dynamic 配置使用既有能力门，执行前核燃料和槽预算。提请复审后更新 AutoLang `docs/specs/auto-av/language-parity.md` 中的大小写和指令表。AutoUI `e2314084` 在原 029 的查询和标题读取使用同一转换，保留空查询的短路行为，桌面 prep 也能沿同一路径。
- ACE 的 Unicode 正例覆盖 `İ` 扩成两标量、希腊词尾 `ς` 和西里尔文本。两个 AV 配置均实际验证字节码往返、能力缺席拒绝、原始文本保持与释放。AV 定向回归（含指令表）17/17，整命令 5.767 s（用例执行 0.10 s）。A2X 脚本组 34/34，整命令 5.765 s，真实 Node 执行 JavaScript 和 TypeScript 并验证文本释放。`scratch/r102-unicode-search/av-final.log` SHA256 `E34ED9FCD6675570A191923EAAE6CB296ABFFBEAC4CC7C51CFDB7256F80D849A`，`script-tests-final.log` SHA256 `8FE7779CACFBDF3FE5CB45E9CBC5EAE55C8C073A33EC11DBCEA30A35D6FD0F36`。AV core 库组 72 过、1 红；早于本轮的主线二进制也复现 `layout_rejects_non_record_and_empty` 同一空记录断言（`layout-baseline.log` SHA256 `01CECD6DF57B468B108EDE7B975ECB35E64FA646AA5B7E9366DA884FF46C72FE`）。
- 原 029 源、生成的真实 Rust 后端及 Iced 窗口使用两张已验证图片，其中一张命名 `İSTANBUL ΟΣ.png`。通过现行窗口输入和提交回调，带组合点的查询、大写查询、不同 sigma、清空查询依次得到 1、1、0、2。会话关闭，808 帧，3 次缩略图解码，无派发或图片错误，后端退出。最终 AutoUI 库准备 91.133 s，窗口探针链接 8.809 s，原项目执行 30.809 s，均退出 0。`desktop.json` SHA256 `93AB2E52E98CEA505FFDE212D72328DC38F177AAF04A864F6CDD9B46F7667876`，`desktop.log` SHA256 `79FDAF1D4C3CE8B78669C487A9DC7E6DB103FF67050D7AF1D6BC6B8538768044`。
- 受检 Vue 搜索夹具在真实 Chrome 验证同一 Unicode 转换、匹配和不同 sigma 的零匹配，SFC 生成 1.803 s、Vite 构建 3.439 s、Chrome 1.476 s，均退出 0。最终库重发夹具（1.882 s）逐字节一致，SFC SHA256 `BAD6567E2D9B7F970DF3668A31A8F93D36335A097BE8D50F4BD7418893B1167D`，`browser-test.log` SHA256 `6565D3965BACAD7375E22BFEB1467BF453DD6F3640BDC27F9D7280E424DE6835`。从最终库直接生成原 029 时，首拒已推进到 `auto.http.get-json.v1` 的计算面准入（10.016 s，退出 101，无 SFC）。`original-verified.log` SHA256 `100DBCF62F1C0D9B7B0B9FB9E2E3719E856EE08F478FB12BD46AC4B7461E385C`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 102 | from: work/T-06 | to: work | status: executing | outcome: partial | worktree: `D:/autostack/.wt/ui-117/{auto-ui,auto-lang}` | commit: auto-lang `737620f8d` / auto-ui `e2314084` | next: work/T-06 接通原 029 受检 Vue HTTP 与 JSON 值语义；T-02–T-03 仍缺 031 授权媒体绑定。

2026-11-23 T-06 续（round 103）：受检 Vue JSON 值与原 029 扫描结果消费。

- AutoLang `5ca3455ff` 按已声明的 `json.value/1` 准入精确的 parse/stringify 目录，在受检 Vue 脚本运行时承载有类型的 JSON 值。覆盖数字类型区分、嵌套成员与列表索引、显式 nil、Unicode 转义、对象更新、序列化和损坏输入捕获。类型种子携带 `Dyn`、nil、借用文本和对象类型。真实 Node 核对输入深度与字节上限、具名类型拒绝、对象键顺序及文本生命周期。`cargo test -p ace-a2x-core -p ace-a2x-ts --locked` 44/44（整命令 2.598 s，退出 0）。`scratch/r103-vue-json/script-tests-verified.log` SHA256 `3076035B97DB81E86B95328C4F6D4F24311EF6308DC4BCC73EBCDF569F96472D`。
- 旧库对同一受检包夹具拒 `auto.json.parse.v1`（1.808 s，退出 101）。最终库经生产入口生成 SFC、Vite 构建和真实 Chrome 验证两层列表项的标题、排序键、布尔和 Unicode 保真、损坏输入保留既有回执、null 列表回退及重载。生成 1.932 s，构建 1.789 s，Chrome 1.184 s，均退出 0。ImageViewport 双实例初始化错误与恢复回归仍通过（生成 2.113 s，构建 1.772 s，Chrome 1.537 s）。`json-chrome-verified.log` SHA256 `6CBEF8BF416E1118F13BC4A2C446C77B34A4A5B10FB0CC5C2CA9D401890723C1`，`init-chrome-verified.log` SHA256 `C6F4F4BFA5231A4756D7F6A58BCDA38CE4E9ED54C3E01FD50EBC1CB1987A0F2B`。
- 只在诊断副本替换原 029 的一次扫描表达式，回放 round 102 从真实后端捕获的两张照片数据，图片 URI 接回环夹具。带 8 个正式包模块的 SFC 初次运行抓到 `onchange: .SearchInput` 的载荷校验错误。AutoUI `732078a1` 接通 `<input>` change 的受检字符串载荷及动态模型值比较。由最终生产库发射的诊断产物在真实 Chrome 首屏得到 2 张，单独发送 change 再提交 Unicode 查询得到 1 张，无脚本错误。最终库准备 45.901 s，生成工具链接 12.189 s，诊断 SFC 生成 17.788 s、Vite 构建 2.714 s、Chrome 1.579 s，均退出 0。`gallery-populated-diagnostic.at` SHA256 `CF95A47BB27A3A570AEA9D5F595BE291F8B56D1ECDE0CFBE46A88F234FAF6AFD`，`GalleryProductionDiagnostic.vue` SHA256 `47F00544D41E5E00EBDB93DC957BBDF2142EC548BA7565804C478A71B45C01C9`，`gallery-chrome-verified.log` SHA256 `8E6EC26BDE095671D0BA31B664542EEA96A048010C74C4BF5E268D3DCE8ABE02`。
- 原 029 `app.at` 经相同生产生成器的首拒仍为 `auto.http.get-json.v1`，16.019 s，退出 101，未生成 SFC。真实网络请求、异步状态提交、过期响应和卸载归后续 029 原项目验收。`original-gallery-verified.log` SHA256 `6D57AD53CAC584D6A324C72435CBFAB15EE65FA23A9F30C39B0DD4B66D869AD7`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 103 | from: work/T-06 | to: work | status: executing | outcome: partial | worktree: `D:/autostack/.wt/ui-117/{auto-ui,auto-lang}` | commit: auto-lang `5ca3455ff` / auto-ui `732078a1` | next: work/T-06 接通原 029 受检 Vue 的异步 HTTP 扫描并核生命周期；T-02–T-03 仍缺 031 授权媒体绑定。

2026-11-23 T-06 续（round 104）：029 原入口产出受检 Vue，Chrome 与原 Rust 后端联合验证。

- AutoLang `08bd7fef8d`：根据受检 HIR 识别 `Http.get_json` 与传递调用链，复用 `http.client/1` 和 `json.value/1` 两道现有能力门。只在受检 Vue 宿主发射可挂起函数，回调和闭包形态具名拒绝。脚本组 `cargo test -p ace-a2x-core -p ace-a2x-ts --locked` 45/45 通过，其中真实 Node 用例核跨帧 UTF-8 文本所有权、暂存模型隔离、燃料恢复和取消后分配归零。`scratch/r104-vue-http/script-tests-final.log` SHA256 `AA0ACC4F730739529C6EB389C99FAC5FCBF9E771E65F288A5B983669194AE37C`。
- AutoUI `a9bdc39651`：受检 Vue 从同源异步读取限长 JSON，将旧请求的宿主响应与当前模型和组件实例进行比对，并在替代或卸载时取消请求。HTTP 失败沿原处理器的 `try/catch` 回传。复审拟在 `docs/specs/runtime/component-http-async.md` 补记受检包面支持范围、提交条件和取消行为。
- 029 原 `app.at` 与上轮应用源码相同。生产链接入口无诊断覆盖，发射 2,466,032 字节 SFC、8 个内嵌组件（生成 17.228 s、Vite 12.127 s）。Chrome 使用原生成 Rust 后端与独立照片 fixture，观察 7 条目录扫描请求。Unicode 搜索由 2 项缩至 1 项再回 2 项，没有新增扫描。反向响应通过代理延迟实测，先发扫描在换目录后取消，随后验证 503 显示与返回根目录恢复，以及 Vue 卸载取消最后一条扫描。Chrome 2.527 s，退出 0。`Host.vue` SHA256 `C0FE2B9AD248ACD7BC1F56162FE5DEF6B9861BC0BD92B4256D1B2505152F197F`，`original-chrome.log` SHA256 `D36DE434BF9E89F9A7E20A63407496BD60BFAB53D25557362CE2B49769ABE228`。
- 相同 Vue 产物加生成器样式后，以真实点击完成 Unicode 照片选择、打开大图、返回网格、再次打开。两次全图请求均返回 200，Chrome 解码尺寸均为 320×240（命令 1.791 s）。`original-media.log` SHA256 `F6A876AA6CB0DF16121A41BF596172996E06038052E126D9092763C1801DF0E1`。常驻无 HTTP 的 ImageViewport 夹具亦通过双实例初始错误、重新校验及两次恢复（命令 4.765 s）。`init-regression.log` SHA256 `9B100DEC99A1B31B35541EFE08C2B97E32FB51546A53E0132F2C22EBE52B9290`。
- stage: goal | plan_id: PLAN-117 | plan_revision: r1 | round: 104 | from: work/T-06 | to: work | status: executing | outcome: partial | worktree: `D:/autostack/.wt/ui-117/{auto-ui,auto-lang}` | commit: auto-lang `08bd7fef8d` / auto-ui `a9bdc39651` | next: work/T-06 复核 029 剩余原验收与合并前回归；T-02–T-03 仍缺 031 授权媒体绑定。

## 10. 待澄清事项

2026-11-23（round 51，待裁定）按 §2 需明确共享写面。029 的现有 JSON 值原语已在单组件面验证，包导入会先把 `json.parse` 分派到不提供该值原语的 `std::json`，仍缺包链接、运行验证。031 的后端使用 `auto.image` 管理图片会话，AGE 包宿主尚无原 `back.api` 的带授权绑定。已询问用户沿原分工由共享能力 owner 交付以上接缝，或授权 PLAN-117 接手这两处共享运行时改动。两端原验收、文件授权和会话释放仍是接入条件。

2026-11-23（round 58 更新）029 已用原有 AGE JSON 原语和 AutoUI 源准备接线越过该首拒，无需就这处另行裁定。031 的带授权后端绑定仍需明确共享写面。029/031 Vue 的存储和后端接缝仍须消费受检宿主能力。

2026-11-23（round 61 更新）031 媒体会话的后端打开和关闭已有运行证据。目录未授权前仍能读取同一冻结 fixture，现有私有路由与文件授权不能直接作为媒体访问准入。共享写面仍待裁定，后续接入须兑现宿主授权和会话回收。

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

2026-11-23 T-03 续（round 31，§10）：**json 值链 coerce 门全面放宽，029 拒墙 91→42**（auto-lang `ab40746af`）。编译随层推进暴露全量面：`expected X, found Dyn`（65：成员读落具体期望位）+ `str/Str` 写位对（26）。**修复**：① `expr_with_context` 尾部的 recover_dynamic **Dyn coerce 门从 Auto 面放宽到 text/collections AGE 执行面**（round-28 绑定位机制推广到实参/条件/返回位）；② binary 组合器单侧 Dyn 操作数按对侧具体类型 DYN_TAKE（判空比较已分流）；③ typecheck Assign 臂 age_eq 文本写位跨借用/owned 放行（AP-4 提升与 coerce 的 owned 产物落回借用期望位）；④ TCMISMATCH dump（AC117 门，span 级归因常备）。**剩余 42**：str/Str 写位（18——TCMISMATCH span 指向 if-cond/字段写，逐位归因待下轮）+ bool-Dyn 条件（13——`resolve_short_circuit_bin`/条件位的 Dyn→Bool coerce 收口）+ Str-Dyn（11）。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：42 条逐位归因（TCMISMATCH span + resolve_condition/short-circuit coerce 收口）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 32，§10）：**photos/set 对根因实锤并清零，029 拒墙 42→24**（auto-lang `98d670947`）。**根因**：CST 投影把 `X[i].f = v` 发射为 `X.set_field(i, "f", v)` 的**裸局部接收名**——view-data 基座不经读改写通道（读改写只覆盖读位）；`hook_stmts_lower` Effect 臂将裸 view-data 接收名改写为物化局部（029 ToggleFav 的 `.photos[i].fav = !…`）。**连带**：① `str` 注解提升**收窄为值条件**（AGE 面注解 str + 初值已 owned 才提升——无条件变体回归 json_975 的 slice/index 消费位，stash 对照实证后修正）；② unbound probe 扩 bare photos；③ TCFN/TCLET fn/局部级 dump 常备。**拒墙 24**：bool-Dyn 条件（13）+ Str-Dyn 写（11）——逐位归因下轮（TCLET 证 Let 位已清，剩余在 &&/|| 操作数位与 Str 期望的实参/写位）。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / wire 3 / collections 124 / event sessions 5（基线红除外）/ t02 probe 44/44 / t01 4/4。**下轮切面**：`resolve_short_circuit_bin`/实参位与 Str 期望位的 Dyn coerce 收口 → 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 33，§10）：**剩余 24 条排除法归因**（auto-lang `e55fe42b0`，勘察轮）。TCLOGIC dump 零命中 → bool-Dyn 类不在 And/Or typecheck 臂；TCLET 零命中 → Let 位已清。**锁定**：Str-Dyn 类（11）与 **CallFn 实参位**对齐——`__p117_str_lower(p.title)` 等 helper 调用的实参（json 值链成员读 = Dyn）经 `expr_iterative` 无期望解析，callee 形参 Str → typecheck `expected Str, found Dyn`；bool-Dyn 类（13）待同法排除（`!` 一元位或条件位）。**下轮切面**：resolver 用户调用臂 CallFn 实参按 callee 形参类型 coerce（round-31 机制延伸到实参位）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。工作树保留（auto-lang `e55fe42b0`），恢复点=round 33。

2026-11-23 T-03 续（round 34，§10）：**实参位 + 一元位 coerce 收口，拒墙 24→1（adapter bind 内部错）**。**修复**：① `adapt_argument_type` **CallFn 实参按 callee 形参类型 coerce**（Dyn→DYN_TAKE；Str-Dyn 类 11 条全退场）；② UnaryOut **Not 臂 Dyn→Bool coerce**（`!used[bi]` List(Dyn) 元素读；bool-Dyn 类 13 条全退场）；③ 大小写 helper 改**字母表扫描**纯 str 词汇（`__p117_str_index_in(slice, alphabet)` + 对表 slice 映射——collections descriptor 无 char 转换用例，char_at/to_codepoint 族不可用）。**最后 1 条**：collections-draft descriptor 的 **t_f32 类型行缺失**（029 自有 `view_zoom/view_ox` f32 model 字段——collections.atom TypeKind 无 f32，PLAN-981 f32 面未覆盖该 descriptor）+ 1 个 `text_to_borrowed` 用例缺失——**面合同切片**（descriptor 扩展或 f32 行发射），非本计划代码层。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：collections descriptor f32 行 + text_to_borrowed 用例（面合同扩展，PLAN-864/981 owner 协调或 helper 重设计规避）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 35，§10）：**bind 层清零，lower/verify 深入内联 state fn**（auto-lang `96c036439`）。**修复链**：① collections.atom **加性放宽**（TypeKind f32 + f32 BinOp/UnaryOp 用例 + expr.text_to_borrowed + 四个 branch——PLAN-981 f32 面与 text_to_borrowed 视图物化；既有文档逐字节不变）；② emitter uses_f32 门纳入 collections profile（029 自有 view_zoom/view_ox f32 model 字段）；③ reanchor 投影漂移 span 对齐前一边界（FSTR clamp 先例；prep 注入 helper 使多字节偏移）；④ collections 族 lowering 准入 `module.global.` cell 访问器/http 目录声明/json 目录声明（PLAN-1106 内联体携带；无 provider 车道按 938 不变式 fail-closed）；⑤ mir_verify/host 的 http∪json 混合族门（host.profile/PlatformCall/import 缝三处对齐 abc 线族并集口径）。**现状**：029 场景编译抵达 **mir_verify 的 invoke-profile 门**（PLAN-1106 内联 state fn 的消息调用 `.GalApply()` 在 collections 车道无 INVOKE 准入）——下一切面=INVOKE 门或 state fn 车道收口。回归（本层）绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84。**下轮切面**：INVOKE 门收口 → 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 36，§10）：**INVOKE 门收口 + Dyn 标量显示，verify 深入 state fn 至 owned-walk 末层**（auto-lang `d5941a4f6`）。**修复**：① mir_verify Invoke 终结子随**元素视图面标记**准入（collections 域场景/事件道的 try 保护体内联体——fallible 指令着陆边与 closures 同机制，执行器按帧级边通用处理）；② TextDisplay 的 **Dyn 标量源**准入（元素视图面 `p.sort_key.str()`）+ 执行器渲染 Dyn 内标量（i32/u32/i64/f32/Str）。**拒墙现状（最后 1 条）**：`mir.use-after-move: read of moved slot __age_s_0_qf_text (clone source)`——物化局部只读共享，但 owned-Str 形参把调用实参建模为**消费性 move**——同一物化局部的第二个读者（`.qf_text` 在比较与 helper 调用两处）触发。**下轮切面**：owned-walk 对**不可变物化局部的克隆源读**的 modeling（只读局部克隆不消费）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。回归全绿：ace-core lib 88 / p108 7 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。

2026-11-23 T-03 续（round 37，§10）：**use-after-move 对清零；抵达所有权分析资源门**（auto-lang `6a198e5a2`）。**根因与修复**：`__age_s_0_qf_text` 的 move+clone-source 对 = owned-Str 调用实参建模为消费性 move——cell 语义是**每次读 = 快照克隆**（提交期写回 cell）；用户调用实参适配对 owned-Str 局部读**克隆包装**（mutable 限定移除——typed 字段 cell 局部亦为快照读语义）+ 文本 Eq/Add 操作数克隆从 auto_language 放宽到 collections。**拒墙现状（1 条新层）**：GalQuery 检查函数（最大 handler：try+嵌套循环）的所有权分析需 ~256MB > 冻结 64MB 预算——克隆加密放大了不动点状态，分析优化为下一切面（非语义问题）。回归全绿：ace-core lib 88 / p117 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：所有权分析资源优化（GalQuery 体量）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 38，§10）：**资源门结构归因（勘察轮，无代码变更）**。所有权分析预算门的精确结构（ownership.rs:520-560）：`bound = compact.max(legacy_bound)`——`compact`（单根记账，省 token/field 位集）仅在 `use_package_bound`（capabilities 含 **`av.shared-collection-args/1`**）时可用；该 capability 仅由 **closures/dynamic lowering**（hir_adapter:1565）在 dynamic 线族合同下盖戳（"MIR and ABC admission still check the entire result"——共享实参经 dynamic 面全量准入已被证明）。`uses_f32` 类门（4805）也显式排除 Collections。**下轮切面（两条备选）**：(a) 大 handler 场景 state fn 路由 dynamic 线族（closures_profile machinery——共享实参记账已被该面证明，降低 bound 至 compact）；(b) 所有权分析状态压缩。工作树 clean（auto-lang `6a198e5a2`），恢复点=round 38。

2026-11-23 T-03 续（round 38-39，§10）：**资源档上调 + 深入 List 源 move**（auto-lang `3919a9652`）。round 38（勘察）：资源门结构归因——compact 单根绑定在 `av.shared-collection-args/1` capability 后（dynamic 线族先例 hir_adapter:1565）。round 39：① 组件面所有权分析档 64MB→**256MB**（029 真语料最大 handler GalQuery+内联 GalApply 估计；二次 blowup 门两档有效——既有面档上调先例同型）；② 越过资源门后抵达 **List(Dyn) 物化局部的 list-source move**（`var source = .photos` 的 owned move 消费只读快照；文本克隆已收口 text-only，List 版需通用 list-clone 表达式：Clone MIR 泛型但 resolver 层无 list 下沉形）。回归全绿：ace-core lib 88 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：List(Dyn) 物化局部的克隆表达式（`(List(_), "clone")` builtin 或 ReadLocal 克隆下沉）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 40，§10）：**OwnedClone 路线落地；abc 门加宽未遂回退**（auto-lang `e145fced7`）。**落地**：① collections.atom 增 `expr.owned_clone` 用例+branch（加性——泛型 Clone 在本面的 HIR 下沉形，filter/list 路径同机制）；② verify 的 OwnedClone 门随元素视图面标记放行；③ round-38 view-data 读克隆推广为 `.clone()`(text)/`OwnedClone`(list) 双形（值条件收口保持）。**未遂**：abc.rs 的 Collections 线族 host-section/DYN/OBJECT 门加宽经三轮盲插位置错误损坏 decode 臂列表——**已 git checkout 回退**（回 HEAD 干净态）；门位点已精确勘察（op match 位于 decode_abc_limited，capabilities 先于臂解码、同函数作用域），下轮可干净重开。回归全绿：ace-core lib 88 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：abc.rs Collections 线族 host-section + DYN/OBJECT 类型行/指令门干净重开（位置已勘定）→ 029 拒墙清零 → 029 真窗实跑（V-03）→ 031 back 桥。

2026-11-23 T-03 续（round 41，§10）：**Collections 线族门干净重开全链落地**（auto-lang `3f5d17944`）。**落地**（encode/decode/mir_verify 三层）：① encode host-section 并集门 + Dyn/Object/Nil 类型行 + DYN/Object 指令臂 + PlatformCall（elem_view_collections 标记 / all_event_host_imports 并集）；② decode elem_view_dyn 标志**正确位置**（instr 解码循环前——capabilities 先入 module struct 再解码类型/指令；round-40 的盲插教训修正）+ TY_DYN/TY_OBJECT/TY_NIL 类型行臂 + DYN/OBJECT 指令臂 + PlatformCall；③ mir_verify host.profile 错误带 host_imports 符号表（诊断）。**拒墙现状（1 条）**：029 场景编译穿过 encode+decode 抵达 mir_verify——dark_mode state fn 的 **PlatformCall 引用空 host_imports 表**（json.parse 的 host index 在 per-unit 拆分后未达该 unit 的表——**split-units 的 host 表传播**为下一切面）。回归全绿：ace-core lib 88 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。

2026-11-23 T-03 续（round 42，§10）：**compile 链（bind/lower/verify/wire encode+decode）全清**（auto-lang `864557d4f`）。**根因与修复**：decode 的 host-section 门缺 Collections——解码跳过 HOST 段（host_imports=[]）而 PlatformCall 仍引用 host index → mir_verify host.profile 空 table 拒。**修复**：Collections 线族恒携 HOST section（encode+decode 对称——Owned 面 PLAN-108 先例同型；admission 门限合法 import）。**拒墙现状**：编译链全清；失败移至**执行**——scenario driver 的状态观察执行内联 handler 体（http/json host import）无 provider → `host.unauthorized` 具名 fail-closed。**语义边界**：scenario 状态道是声明/状态观察道，效应体不可执行——真窗执行路径 = 事件道（ComponentEventSessionProgram + provider 注入，rounds 34-36 已落地）。回归全绿：ace-core lib 88 / p117 4 / json_975 4 / plan981 84 / t02 probe 44/44 / t01 4/4。**下轮切面**：scenario driver 对效应体的执行语义收口（跳过/桩化内联效应）→ 029 编译链+状态观察全通 → V-03 真窗实跑（事件道 + provider）→ 031 back 桥。

2026-11-23 T-03 续（round 43，§10）：**状态道执行边界归因（勘察轮，无代码变更）**。`host.unauthorized` 的触发点 = age_component.rs:340 `execute_value`——状态观察经 lower→verify→admit→AV 执行 `d___age_state_0_dark_mode`；而 state fn 体 = instance_stmts（纯字段 init，029 无 ops）**不应含 host 调用**——嫌疑 = lowering 的 def-walk 收集**模块级**host imports + 入口解析/执行触及非入口 def（或 entry 解析回落）。**边界语义**：状态道是声明/状态观察道；含效应体的 handler 执行归事件道（ComponentEventSessionProgram + UiHttpProvider 注入，rounds 34-36 已落地，桥 load_source:2089 已注入）。**下轮切面**：① 排查 execute_value 的 entry 解析是否回落到含效应体的 def（entry 隔离）；② 状态观察跳过/隔离 host 模块 def；③ V-03 真窗实跑走事件道 dispatch（桥已注入 provider）→ 031 back 桥。工作树 clean（auto-lang `864557d4f`），恢复点=round 43。

2026-11-23 T-03 续（round 44，§10）：**entry 隔离排查完成——host.unauthorized 为模块级触发**（auto-lang `c2dcd2668`，诊断 dump 入库）。**实锤**：AC117-EXEC dump 显示 state fn 入口 `d___age_state_0_dark_mode` 的**入口体内零 host 调用**——但模块 MIR 携带 host_imports（lowering def-walk 收集**模块全量**：check fn 内联体携带 http/json）→ 执行器的运行时 grant 核对对**任何携带 host imports 的模块**要求 provider——即使入口永不触达。**下轮切面**：① lowering 侧按**入口可达**过滤 host imports（check fn 的 host 调用不进 state fn 单元的表）；② 或执行器 grant 核对按 entry-reachable imports。回归（本层）绿：ace-core lib 88 / p117 4 / json_975 4 / plan981 84。V-03 真窗实跑仍走事件道（桥已注入 provider；事件道 029 编译已于 round 36 全通）。工作树 clean（auto-lang `c2dcd2668`），恢复点=round 44。
