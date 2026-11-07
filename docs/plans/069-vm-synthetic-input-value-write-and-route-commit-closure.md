---
plan_id: auto-ui/PLAN-069
status: execution_done
feature_name: VM合成输入值写与路由状态提交缺陷收口（068后续，round-19家族）
author: [ZCode]
created_at: 2026-11-07
updated_at: 2026-11-07
plan_revision: 1
base_branch: master
base_commit: ab19ff63
current_step: 6
total_steps: 8
depends_on: [auto-ui/PLAN-068, READER-009]
supersedes_spec_components: [docs/specs/runtime/component-and-store-instances.md, docs/specs/runtime/vm-route-page-loading.md, docs/specs/runtime/native-vm-app-validation.md]
new_spec_components: []
touched_goals: []
---

# auto-ui/PLAN-069：VM合成输入值写与路由状态提交缺陷收口

## 0. 变更摘要

READER-009 T-03原生UI门在真实1000件夹具上实证了四个同族runtime缺陷（round-19家族），用户2026-11-07 round27裁定②A授权本计划（068后续，auto-ui域）打包修复：①query绑定extract缺口（嵌套store路径value绑定静默不注册）；②VM-mode MCP type_text值写臂root fix（clear_first双派发+parked in-flight guard丢值）；③阅读页route_id丢失→详情页不可达（高严重，阻断009 W-05/W-03/W-07详情腿）；④合成派发路由域（MCP press对部分元素handler执行ok但路由变更不提交）。修复+永久反例+消费门重跑+独立终审+落地后，009消费新pin重测受阻断叶子。本计划继承并取代PLAN-068重开的T08 phase（typing值写臂修复，见§2/§10）。

## 1. 目标

让VM-mode应用的合成输入（MCP type_text/press/action）与真实输入具备同等的状态写与路由提交语义：typed值真实到达目标store状态（含嵌套路径绑定）；clear/type默认路径不静默丢值；route参数模型var跨页装载与状态投影一致；路由状态变更经合成派发与真实输入同语义提交。保持既有合法行为：直field绑定注册、真实单飞（single-flight）parked语义、press类既有通路、gate侧单派发绕行（009 25a80ce）并存有效。

非目标：不改009产品代码与验收分母；不动auto-lang公共边界（确证必须时先具名决策再按068先例兄弟树处理）；不修性能预算（cold-load归PLAN-995）；不新增UI功能/导航能力；不把MCP状态可读写冒充真实OS输入已达（Sky外部限制具名照旧）。

## 2. 架构方案

主责auto-ui。唯一实施树 D:/autostack/.wt/ui-069/auto-ui，分支 codex/plan-069-dev；实施/内检/独立终审顺序共用同树。共享Plan留本主检出。原始probe/诊断材料归 D:/autostack/plan-artifacts/plan-069/auto-ui/（登记in-flight）；READER-009既有反例工件（t03 scope）仍归009，只引用不搬移。关闭时耐久摘要进Plan，009消费所需固定工具/源码pin在清理前交接至009自有scope。

**与PLAN-068的接替**：068已于r2交付归档8/8（master 0b3ee62d），后因typing值写臂缺陷重开r3/T08（AC09，同缺陷域）。用户round27裁定将runtime root fix并入本计划家族包——本计划为该runtime域唯一写入者；068/T08 phase应由其所有者记superseded回执（其授权由本计划继承，见§10协调项），不得双树并行修值写臂。

缺陷族→改面假设（T00逐族机制确认后冻结实际改面，观测锚点见§4）：

```
真实/MCP输入 → mcp_server(动作派发,clear_first) → vm_bridge(call_handler,parked guard)
  → handler执行(.at) → store/route状态写 → dirty/rebuild → 路由outlet挂载/视图
F1 extract：dynamic.rs scan_node_for_inputs 仅注册直field绑定 → input_state_map缺口
F2 值写臂：clear_first双派发 × parked guard → 第二派发Ok(None)丢值
F3 route_id：route param→模型var→autoui_state投影链 某环丢失（根因未定谳）
F4 合成派发：action路径 vs 真实输入路径 在路由状态提交处分歧（机制未定位）
```

## 3. 技术栈

auto-ui Rust runtime/compiler（crates/auto-ui-runtime、auto-ui-dialect/auto-ui_compiler aura AST）；VM-mode应用经MCP工具驱动；验证复用既有命名cargo组与009交付的UI门（verify_workspace_ui_vm.py，vm profile）。不跑auto-lang cargo/docs_gen（无语言改面前提）；不改生成Rust/Vue产物或主题快照。

## 4. 需求分析与背景调查

**授权记录**：用户2026-11-07 READER-009 round27裁定②A——授权框架/runtime缺陷修复计划（068后续，auto-ui域），打包round-19家族四缺陷；修复去向与并存注记（gate侧25a80ce单派发绕行已绿，runtime root fix与其并存）为既定范围。同域更早授权：round22用户裁定"在reader 009用过的对应计划上直接加新phase修复"（068/T08重开）——该授权由本计划继承。无用户指定预算/模型/自动续跑限制，不虚构。

**证据源**（全部实测，工件归009 t03 scope，回执见READER-009 §9 rounds 19～27，commit链 d591b348→fe8b3a51→25a80ce6→6c52dfc→17e6b62→4e1ddf7→6a9cba2）：

| 族 | 实测事实 | 本轮核查锚点（auto-ui master ab19ff63） | 状态 |
|---|---|---|---|
| F1 query绑定 | 1000件稳定渲染后type进搜索框ok但store.query恒""；prefs落盘query:""；嵌套`.store.query`不注册，ann/import直field均注册 | crates/auto-ui-runtime/src/ui/dynamic.rs:4196-4203——value prop仅接受`Expr::Ident`或`Dot(Ident(_),field)`，嵌套`Dot(Dot,field)`→None静默；调用点dynamic.rs:4031/4046/4178 | 根因已定谳（观测） |
| F2 typing值写臂 | clear_first=true=Clear+type两次op=input派发；clear段在prefs POST park，type派发撞同handler guard→`re-entry ignored (segment in flight)`→值丢弃；clear_first=false单派发0.27-0.31s正确应用 | crates/auto-ui-runtime/src/ui/mcp_server.rs:3500-3525（clear_first分支二次派发）；crates/auto-ui-runtime/src/ui/vm_bridge.rs:4210-4217与4401（parked guard→Ok(None)） | 根因已定谳（009 round22 instrumented tool T08DBG trace+代码锚点） |
| F3 route_id丢失 | 阅读页正常渲染且按正确book_id拉取books/:id+progress+toc全200，但autoui_state实测route_id=""→Back push"/book/"+""="/book/"→outlet no route→详情页不可达；A/B排除009回归（25a80ce版store文件同行为）；068会话（595eb627源）同链路曾成功 | Reader侧src/front/pages/reading_page.at:420（var route_id=""）/1087（.route_id=router.param("id")）/1091/1118/1167（handler内正确id请求）；runtime侧路由参数var投影链与回归窗口（0b3ee62d→HEAD？工具cohort 6ce87feb→现？）未定位 | 根因未定谳（假设：回归窗口或状态依赖，二选一待T00证据） |
| F4 合成派发路由 | MCP press卡行row[for:book]与阅读页←——ActionResult ok含handler名但路由变更不提交（无页面挂载/无数据请求）；shelf CardClick同构push正常；068 UIA真实输入同版本曾达详情页 | crates/auto-ui-runtime/src/ui/aura_view_builder/slots.rs:444（outlet "no route for {}"为结果表现）；action派发路径（execute_action_vnode/execute_action_on_shared）与真实输入路径在路由状态提交处的分歧点未定位 | 机制未定位（假设：派发路径分歧） |

009现状约束：W-05/W-03详情腿/W-07详情页UI腿阻断至本计划交付+009消费pin；W-04其余五叶已绿不回退；009 gate已按uia_act真实通道推进不受阻叶子（round27后work继续，与本计划无写入交集）。auto-ui master当前存在他计划未提交WIP（desktop_protocol/session等文件，起草时观测），T00实施起点不得触碰，基准重冻结。

## 5. 详细设计

| Rule / task | 实际路径与owner | 代表状态 / 区分性关闭条件 |
|---|---|---|
| R01 / T00 | 四族机制确认：F1嵌套路径注册键格式与值写落点（store实例状态）；F2 root fix设计三选一（clear改状态写不走handler重入/queued-merged重入策略/直写值路径）以证据定案；F3路由参数var生命周期插桩+窗口二分（0b3ee62d→HEAD、cohort 6ce87feb→现）；F4 action vs 真实输入派发分歧点 | 每族决策工件（机制注记+最小复现）；F3/F4由假设转定谳或具名分叉；受影响验证矩阵与base pin冻结（master tip重冻结，避开他计划WIP文件） |
| R02 / T01 | F1：dynamic.rs scan_node_for_inputs注册形状扩展（嵌套store路径解析为路径键）+vm_bridge对应值写落点；永久反例 | 嵌套`.store.query`经MCP type_text默认路径真实写入store状态（Reader搜索框同型最小case）；直field注册行为不变（首匹配优先语义dynamic.rs:4213-4217保持） |
| R03 / T02 | F2：按R01定案实施root fix；永久反例；guard单飞语义按设计保持并注记 | clear_first=true对parks型handler输入后typed值落状态（不静默丢）；真实并发重入策略变化仅限冻结设计内；gate侧单派发绕行并存绿（009 25a80ce不回退） |
| R04 / T03 | F3：按R01定谳根因修复（route参数var写/投影/回归点）；永久反例 | route param→模型var→autoui_state一致（所见=handler所用）；Back链构建正确路径；合成press链详情页可达 |
| R05 / T04 | F4：按R01定位分歧点修复或具名合同边界（须证真实输入同样受影响才可立边界，不得单为合成通道开洞）；永久反例 | 路由提交型handler经合成press与真实输入同语义提交；原红元素（卡行/←）不复现 |
| R06 / T05 | 交付源构建tool；Reader消费验证：warm-query默认clear_first路径绿、嵌套绑定实时过滤、详情链可达；009 pin/工具/hash交接 | 消费证据独立于009会话自证；交接件hash在案 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/runtime/component-and-store-instances.md | value绑定注册：仅直field → 嵌套store路径同注册（路径键格式/写落点/首匹配语义保持） | 修extract缺口而非Reader特判 | AC-02 |
| SD-02 | modify | docs/specs/runtime/vm-route-page-loading.md | route参数模型var生命周期与状态投影一致；合成派发路由提交与真实输入同语义（或具名边界+理由） | 路由状态写为输入语义的一部分 | AC-04/05 |
| SD-03 | modify | docs/specs/runtime/native-vm-app-validation.md | MCP输入值写合同：type/clear默认行为不静默丢值、parked重入策略、gate绕行并存注记、消费pin交付 | 验证合同覆盖值写臂 | AC-03/06 |

具体Spec条目文字由T00机制结论后细化，终审前定稿；不改其余Spec。

## 6. 测试设计

| Case | AC / task owner | 输入与真实入口 | 区分性oracle / 边界 |
|---|---|---|---|
| V01 | AC-02/03；work T01/T02 | 最小VM-mode app：嵌套store路径value绑定input（oninput→QueryChanged型handler）+parks型handler（HTTP park）；MCP type_text默认与clear_first=true两路 | 修前红（值不达/丢）→修后状态精确值落store并渲染；直field对照case不回退；guard对真并发重入行为按冻结设计具名 |
| V02 | AC-04/05；work T03/T04 | 路由链最小app：shelf→route param页→Back；卡行loop元素press；同链真实输入（UIA invoke通道）对照 | param var三处一致（handler内/autoui_state/渲染）；合成press路由提交=真实输入；outlet无"no route" |
| V03 | AC-02～05；work T05 | 交付源工具+009冻结夹具（workspace_fixture.generate 1000件）真实app消费：warm-query默认路径、嵌套绑定过滤、卡行→阅读→Back→详情链 | 独立端口/进程/隔离目录；渲染snapshot判定（不用源文本/静态模板冒充）；W-04既有五叶不回退 |
| V04 | AC-02～05；work T01～04 | 受影响compiler/runtime/tool命名组回归：plan015_route_loading_tests、plan041_instance_ownership及本计划新增永久case组 | 原case不删不减；新增组登记入§5矩阵；未影响族沿同源证据复用，不跑全仓替代定向集 |

测试入口（既有组实测存在）：`cargo test -p auto-ui-runtime --lib plan015_route_loading_tests -- --test-threads=1`、`cargo test -p auto-ui-runtime --test plan041_instance_ownership -- --test-threads=1`；新增永久case模块名T00登记。实施/内检由本执行上下文；终审为独立上下文同权威树（写入暂停后复演V01～03高风险路径与Spec增量）；不得同会话自审授pass。窗口操作仅自有夹具与隔离端口，不操作用户系统安全设置；Sky物理输入未达仍具名外部限制。

## 7. 验收标准

| ID | owner_stage | 可观察标准与验证 |
|---|---|---|
| AC-01 | work | 四族机制确认与设计冻结：决策工件在案（F1/F2定谳复核，F3/F4假设→定谳或具名分叉）；受影响矩阵与base pin冻结；验证见V01～04 |
| AC-02 | work | 嵌套store路径value绑定经MCP type_text默认路径真实写入对应store状态（Reader搜索框消费反例同型绿）；直field注册与首匹配语义不变；验证见V01/03/04 |
| AC-03 | work | clear_first=true默认路径typed值不静默丢弃（永久反例红→绿）；guard单飞语义按冻结设计；gate侧单派发绕行并存绿；验证见V01/03/04 |
| AC-04 | work | route参数模型var跨页装载与autoui_state投影一致；Back链构建正确路径；合成press链详情页可达；验证见V02/03/04 |
| AC-05 | work | 路由提交型handler经合成派发与真实输入同语义提交（或具名合同边界+区分性证据）；原红反例不复现；验证见V02/03/04 |
| AC-06 | work | 交付源tool构建；Reader真实消费（V03三腿）绿；009可消费pin/工具/hash耐久交接在案；验证见V03 |
| AC-07 | review | 独立上下文同权威树复演V01～03与Spec增量，绑定reviewed_commit；AC-01～06全过才reviewed，否则needs_fix/needs_replan/具名阻挡 |
| AC-08 | merge | 精确主干交付、冻结受影响并集回归、canonical核验、归档/计数、guard后清自有树/分支/scope、009交接；不纳入他计划WIP |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际动作 / 关闭条件 | AC |
|---|---|---|---|---|
| T-00 | work | 授权已给（round27②A+继承round22 T08路由） | 新树于master tip重冻结（避开他计划WIP）；四族机制确认与设计定案（R01），F3窗口二分/F4分歧点定位probe入本scope；受影响矩阵登记 | AC-01 |
| T-01 | work | T-00 | F1嵌套路径注册修复+永久反例（V01）；直field不回退 | AC-02 |
| T-02 | work | T-00 | F2值写臂root fix+永久反例（V01）；并存注记落Spec提案 | AC-03 |
| T-03 | work | T-00 | F3 route_id根因修复+永久反例（V02）；根因若证实为窗口回归须定位引入提交并注记 | AC-04 |
| T-04 | work | T-00 | F4合成派发路由提交修复或具名边界+永久反例（V02） | AC-05 |
| T-05 | work | T-01～04 | 交付源tool构建+Reader消费验证（V03）+pin/hash交接件制备；work-owned全过才execution_done | AC-06 |
| T-06 | review | T-05 | 同树独立终审复演V01～03高风险与Spec增量，绑定reviewed_commit/revision | AC-07 |
| T-07 | merge | T-06 pass | 精确落地/冻结受影响并集/canonical/归档计数/清理/009交接（068先例：工具与pin交009自有scope，耐久摘要在案） | AC-08 |

起草时0/8。优先级：用户明示本计划优先于PLAN-995（perf）——解锁面更大（W-05/W-03/W-07详情腿+typing/query家族）。009不受阻叶子与本计划无写入交集，可并行。

### 执行证据（append-only）

- **T-00（进行中：F1/F2 定谳，F3/F4 待探针）** 2026-11-07：新树 `D:/autostack/.wt/ui-069/auto-ui`（分支 `codex/plan-069-dev`）于 master tip `37a7aa43` 冻结；组内依赖兄弟树 detached 只读（auto-lang `160ed38f`、auto-web `2d69841`、auto-down `58ea33f6`）。机制工件 `D:/autostack/plan-artifacts/plan-069/auto-ui/t00-mechanism-decision.md`。F1 定谳=`scan_node_for_inputs` 嵌套 store 路径静默丢弃（`dynamic.rs` 旧 4196-4203）；F2 定谳=clear_first 同 handler 双派发 + parked 单飞 guard（`mcp_server.rs` 旧 3500-3529 × `vm_bridge.rs:4208-4216`），009 并发会话实跑日志（`plan-009/t03/ws-ui-vm-29-w05/app.log` 51-55 行，sha256 `0CE94AA2…C3CC`）直证 `re-entry ignored (segment in flight)`。F3/F4 当前实测：真实输入卡点击同样不可达详情页（`ws-ui-vm-28/29 results.json`），故不可单为合成通道开洞；下一轮按工件「下一轮探针」1→3 定位。
- **T-01（完成）** 嵌套 store 路径绑定注册为展平根字段 + 写通道「实例优先、缺字段回退根态」：`dynamic.rs`（`scan_node_for_inputs` 新 store 别名臂；`on_with_input_for_instance` 写回）+ 永久反例 `crates/auto-ui-runtime/src/ui_tests/plan069_synthetic_input_tests.rs`（3 例）。`cargo test -p auto-ui-runtime --lib plan069_synthetic_input_tests -- --test-threads=1` → 3 passed；修前基线（注册臂短路）2 failed/1 passed（直 field 对照绿=正确红绿分离）。
- **T-02（完成）** `clear_first` 不再重入派发 Clear，单次 TypeText 携带全文（显式 `autoui_action clear` 保留 Clear 单派发；parked guard 语义不变）：`mcp_server.rs` `tool_type` + 工具描述 + 永久反例 `tests_plan069_f2`（3 例）。`cargo test --lib tests_plan069_f2` → 3 passed；修前基线（临时恢复旧双派发）实测 `[Clear(""), TypeText("reader-query")]` 2 条 → 默认路径红。
- **受影响回归（本轮，均绿）**：plan739_input_pin_tests 1、tests_plan483_d4 4、plan712_route_param_tests 3、plan749_read_semantics_tests 13、plan066_filter_projection_tests 4、plan370_015_behavior_tests 12、plan624_cross_state_tests 5（`--test-threads=1`）。
- **T-00（完成）** 四族机制定案：F1/F2 第 1 轮定谳；F3 第 2 轮定谳；F4 具名分叉（见下）。机制工件 `t00-mechanism-decision.md`（第 2 轮追加段含 F3 探针与 009 runs 25/31 日志引用）。
- **T-03（完成）** F3 根因=路由页实例身份带 outlet 路由装饰（`Page@/book/7/chapter/1`），名字类解析从身份**后缀**猜类型 → 取到路由尾段（"1"/""）→ 名字合成派发与「身份 miss 重定向」两臂恒 miss → 落根态 → 页 `.route_id` 空 → Back 构建 `/book/`。修复：`ChildInstanceRecord` 记录注册类型名，`VmBridge::child_instance_identities_of_type`/`child_instance_widget_name` 类型精确匹配；7 处身份名过滤点（派发解析/重定向、`call_widget_handler`、instance-type 派发、重载迁移 schema、iced devtools 输入缓存键、native a11y 重定向）统一改用该口径，身份串与 `@route`/`#key` 装饰格式不变。永久反例 `plan069_route_dispatch_tests`（4 例）：修前 4/4 红（Back 实测 `/book/`，与现场逐字符一致），修后 4/4 绿。
- **T-04（完成，附具名分叉）** F4 卡行（for 内 row 双参 onclick）派发 → 路由提交 + 目标页 Init 读回同一 param：VM 链路修前修后均绿（`f4_loop_card_click_commits_route_and_param`，契约 oracle 非红证）；009 runs 25/31 真实点击已可提交路由。**残留**：runs 25/31 在 阅读页→详情页 换代后出现 `[RC canary] use-after-free: heap object 4000013` + `app view/update panicked`（详情页不落屏）；本 scope 两条最小链路（parked Init 换代取消 / parked UI handler 卸载取消）**未复现**。判定为 VM RC 生命周期域（auto-lang 资产），待 T-05 用本树工具重跑 009 门复核：消失即收口；仍在则具名范围分叉交用户裁定（不静默扩面）。
- **受影响回归（第 2 轮，全绿）**：plan069 10、plan051_p2 3、plan051_ext 3、plan051_timer 9、plan437 5、plan492 9、plan056 15、plan606 1、plan622 8、plan633 4、plan712_route_param 3、plan712_back_prefix 1、plan739 1、plan624 5、plan370_store_vm 6、plan370_015 12、plan066 4、plan089 1（`--test-threads=1`）；集成档 `--test plan041_instance_ownership` 4。
- **T-03 追加（第 3 轮，RC 生命周期定谳）** 详情腿残留根因不是 route 链而是**根态被释放**：实例对象拥有自己的字段引用（`materialize_init_value` 造容器 retain；`__auto_parent`/`__auto_root` VmRef 分配处 retain；auto-lang `free_heap_id` 逐字段递归 release），而 teardown 又「字段循环 + 显式 parent/root release」释放一遍 → 每次页换代根态净 -4（挂载 +2），RC `3→5→3→…→0`，根态被回收，之后每帧读根（`dark_mode` 等）即 `[RC canary] use-after-free: heap object 4000013`，详情页体恒空。修复 `01c0ab2c`：teardown 只释放挂载 stake（字段引用留给对象回收级联释放一次）；`ensure_child_instance` parent-change 臂同规律去重。永久反例 `f3_page_transition_keeps_root_state_alive`（8 次换代后根态 RC>0 且可读；修前第 2 次换代归零，实测 rc 3→1→0）。实证链：根态 id=4000013（teardown 打印 parent_id）、RC 打点轨迹、`RUST_BACKTRACE=1` 栈顶（projection→read_state→get_heap_object→canary）、`P419_TRACE=1` 释放轨迹（级联释放早期 id 4000004..11）。
- **T-05（完成）** 交付源 tool 自 `01c0ab2c` 构建（sha256 `3204811e…68d4`），以 009 组依赖 pin（auto-man `03bb65e`/auto-lang `373506568`/auto-web `bf72baa`/auto-down `58ea33f`）+ 冻结 Reader 源（`cf4677e` + 其未提交 UI 门改动）复跑 UI 门：**warm-query 默认 clear_first 路径 PASS**（`matched=193/193, 0.30s`）、grid/list 21 页 PASS、**group-create-rename PASS / membership PASS**（需详情页真实装载——W-05 详情腿）、exit PASS；整轮 `RC canary`/`app view panicked` = **0**。具名残留：cold-load `19.23s>15s`→PLAN-995；W-06 批量腿（全选 1000 vs 193、移除 503）→READER-009 W-06/其后端时延；W-07 `未读` 控件时序→READER-009 W-07。交接件 `plan-artifacts/plan-069/auto-ui/t05-consumption-handoff.md`（pin/tool/hash/命令/边界），原始证据 `consumption/run-03-final`（results `AD131B00…`、app.log `BC5C3A62…`）。
- 代码提交：`c283dbcb`（F1/F2，4 files）、`370948d3`（F3/F4，6 files）、`01c0ab2c`（RC 生命周期 + SD 草案，5 files）。


## 9. 复审记录

2026-11-07：stage: new | auto-ui/PLAN-069 r1 | outcome: pass（草案可交接，未执行） | changed: 新建T-00～07、AC-01～08、V01～04、SD-01～03 | next: work T-00（新树建立+四族机制确认）。授权依据READER-009 round27②A（含round22 T08路由继承）；证据锚点为本轮起草实测核查（§4表），F3/F4根因未定谳已如实标注假设状态，T-00定谳前不实施依赖修复。

- stage: goal | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | round: 1 | from: new | to: work | status: drafting→work待启 | outcome: pass | worktree: 待T-00创建（D:/autostack/.wt/ui-069/auto-ui拟） | commit: 本主检出簿记提交 | next: work T-00。

- stage: work | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | outcome: pass（T-01/T-02 完成；T-00 部分——F1/F2 定谳，F3/F4 待探针） | worktree: D:/autostack/.wt/ui-069/auto-ui | branch: codex/plan-069-dev | base_commit: 37a7aa43（master tip 重冻结；起草 anchor ab19ff63 后仅 docs 提交） | code_commit: c283dbcb | dependency_revisions: auto-lang 160ed38f / auto-web 2d69841 / auto-down 58ea33f6（组内 detached 只读兄弟树） | task_ids: T-01, T-02（T-00 部分） | evidence: plan-artifacts/plan-069/auto-ui/t00-mechanism-decision.md；cargo test plan069_synthetic_input_tests 3 passed、tests_plan069_f2 3 passed（各自修前红在案）；受影响组 plan739 1/tests_plan483_d4 4/plan712 3/plan749 13/plan066 4/plan370_015 12/plan624 5 全绿 | blockers: 无用户待答；F3/F4 机制未定谳=下一轮 T-00 探针（route param 派发对象/卡点击链路） | next_responsible_role: work（T-00 F3/F4 探针 → T-03/T-04）

- stage: goal | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass（work 阶段进行中：T-01/T-02 落地，T-00 F3/F4 待续） | worktree: D:/autostack/.wt/ui-069/auto-ui | commit: c283dbcb | next: work T-00（F3/F4 探针）→ T-03/T-04。

- stage: work | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | outcome: pass（T-00 完成、T-03/T-04 完成；F4 具名分叉在案） | worktree: D:/autostack/.wt/ui-069/auto-ui | branch: codex/plan-069-dev | base_commit: 37a7aa43 | code_commit: 370948d3 | dependency_revisions: auto-lang 160ed38f / auto-web 2d69841 / auto-down 58ea33f6 | task_ids: T-00, T-03, T-04 | evidence: plan-artifacts/plan-069/auto-ui/t00-mechanism-decision.md（第 2 轮追加段；探针日志 probe-2/probe-5 与 009 runs 25/28/29/31）；cargo test plan069 10 passed、plan069_route_dispatch_tests 修前 4/4 红（Back=`/book/`）修后 4/4 绿；受影响 18 组 + plan041 集成档全绿 | blockers: 无用户待答；F4 残留 RC canary UAF（heap object 4000013）未在本 scope 复现，属 VM RC 生命周期域，T-05 复核后处置 | next_responsible_role: work（T-05 交付源构建 + 009 消费复核 + pin/hash 交接）

- stage: goal | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-03/T-04 落地；T-05 待启） | worktree: D:/autostack/.wt/ui-069/auto-ui | commit: 370948d3 | next: work T-05。

- stage: work | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | outcome: pass（T-03 追加 + T-05 完成；全部 work-owned 任务完成，转复审） | worktree: D:/autostack/.wt/ui-069/auto-ui | branch: codex/plan-069-dev | base_commit: 37a7aa43 | code_commit: 01c0ab2c | dependency_revisions: 交付树 auto-lang 160ed38f；消费复跑按 009 组 pin auto-man 03bb65e / auto-lang 373506568 / auto-web bf72baa / auto-down 58ea33f | task_ids: T-03（追加）, T-05 | evidence: plan-artifacts/plan-069/auto-ui/t00-mechanism-decision.md（第 3 轮段：UAF 定谳链 + 修复 + 回归）、t05-consumption-handoff.md（pin/tool/hash/命令/边界）；plan069 11 passed；plan041_instance_ownership 4 / plan068_routed_continuations 14 / 其余 17 组全绿；消费 run-03-final（warm-query 默认路径 PASS、group-create-rename/membership PASS、canary=0，tool sha256 3204811e…） | blockers: 无用户待答；具名外部残留 cold-load→PLAN-995、W-06/W-07→READER-009，均与本计划改面无交集 | next_responsible_role: review（T-06 独立终审：复演 V01～V03 高风险 + Spec 增量；树内写入暂停）

- stage: goal | plan_id: auto-ui/PLAN-069 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass（work 完成，交独立终审） | worktree: D:/autostack/.wt/ui-069/auto-ui | commit: 01c0ab2c | next: review T-06。

## 10. 待澄清事项

无待答范围授权。三项协调注记（非用户待答，责任方明确）：

1. **PLAN-068 T08接替**：068所有者需在其§9/§8记T08 superseded回执（授权由本计划继承，理由=round27②A家族包）；在该回执前，068/T08不得与新树并行写入值写臂域（一写入者）。若用户希望改由068/T08继续typing修复而本计划排除F2，则为范围变更，须用户明示后修revision。
2. **auto-ui master他计划WIP**：起草时master存在未提交改面（desktop_protocol/headless|host|rqhost、session.rs、docs/design等）。本计划T-00起点重冻结master tip；若该WIP后续落地触碰同文件（vm_bridge/mcp_server/dynamic），T-00须rebase并复核锚点行号（§4锚点为ab19ff63观测）。
3. **009依赖pin身份（已定）**：009回执的 dependency `auto cb31bc2a` = **工具链 CLI `auto.exe` 的 sha256**，实体为 `plan-artifacts/plan-009/t01/toolchain-target/debug/auto.exe`（本轮 T-05 消费复跑即用此 CLI；`auto-ui-tool` 另行自本树构建，sha256 `3204811e…`）。消费基准已按此重冻结。

## 11. 交付后具名残留（非本计划 AC，责任方已定）

1. cold-load 预算 `19.23s > 15s` → **PLAN-995**（VM json/装载 perf），本计划未触碰。
2. READER-009 W-06 批量腿：`全选当前结果` 得 1000（期望 193）、`移除（可恢复）` 出现 503 取消 → **READER-009 W-06/其后端时延**（本轮环境单请求 2–17s，批量 RESTORE 超时）。
3. READER-009 W-07 `未读` 控件时序（`control not visible in time`）→ **READER-009 W-07**。
4. OS 三通道（UIA invoke/物理 click/sendclick）在卡行上的投递面仍归 READER-009 门的通道选择；本计划只交付 VM 链路与合成派发的路由提交（不把「物理输入已达」冒充为交付，Sky 外部限制照旧）。
