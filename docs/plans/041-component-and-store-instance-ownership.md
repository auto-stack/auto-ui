---
plan_id: PLAN-041
status: executing
feature_name: 组件与Store实例所有权收口
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 6
total_steps: 10
supersedes_spec_components: [docs/specs/runtime/vue-bare-store-routing.md]
new_spec_components: [docs/specs/runtime/component-and-store-instances.md]
touched_goals: []
affects: [vm-bridge, child-widget-state, vue-store-context, component-owner-lifecycle]
depends_on: [auto-ui/PLAN-031, auto-ui/PLAN-033, auto-ui/PLAN-036]
---

# [auto-ui/PLAN-041] 组件与Store实例所有权收口

## 0. 变更摘要

主题作用域已交付后，收口**业务状态的实例归属**：同型child的model/props/handler receiver独立，store在同App实例context内保既有共享，跨App默认隔离；显式共享与最后owner退出有据。真实Vue/VM事件与keyed重排/关闭均验，不靠只换主题或对象ID冒完成。完整Feature/命令框架留后继。

drafting r1→executing（2026-11-04，`/goal 完成计划041` 全链授权：work/review/有界修复/merge）。T-01 完成 @ui-041 `0f2f9fa6`：实例/App/store调用面冻结（细节折入任务行与 plan-artifacts/t01-freeze.md），正负控制落地——控制腿绿、隔离腿红相实录（一次Inc双实例串写，props 仅视图级顺序掩蔽）。T-02..T-04 完成、execution_done（6/10）@ui-041 `ef73c7d4`→`dad974dc`→`b30c749a`→`ad163727`→`9cac1cf1`：三条具名红收口（plan437 真实修复/plan498_area 场景先物化/plan643 t01 基线即红）；child-emit 链修复（seed 布局并集+emit 桥根跳）plan058/063 全绿；plan841 三测双押修复；plan499/711/502/musk-p053-8 oracle 实例态调和；T-03 Vue store 厂形+context 注入+legacy 边界；T-04 V-03/V-05 永久 oracle；SD-01..04 草案落 worktree docs/specs。全库 lib 层 0 本包回归（余 10 项均 base 同败：plan643t01/plan714/t2_remote/shell_pack×2/typing_loop/tools×5 部分重合、gallery_vue_golden/vue_theme_container integration 基线漂移亦 base 同败）。T-07 独立 final review 待启。

## 1. 目标

- G1：两个同型组件可持相反props/同名model字段，view/computed/handler与写屏障始终指向本实例，顺序/重排不串。
- G2：同App实例内既有use同store继续共享；两个同名App/挂载root不隐式共用模块singleton；显式共享owner可注入并验证。
- G3：重建/删除/退出只释放自己对象/订阅/Tick/绑定，存活与显式共享owner继续可用；延迟回调不误投另一同名实例。
- G4：保原store方法路由、RC/heap引用、memo失效和031主题scope，给042稳定实例/状态描述出口。

非目标：新.at语法、全量状态事务/undo/持久化、通用Feature/Service/命令全局路由、完整HMR、迁移所有Demo/OS产品、改VM通用语义/GC体系、全AG/ACE回归。

## 2. 架构方案

App实例context→store owner及组件实例身份→真实state object/receiver→view与handler→mutation/memo/RC→owner退出。保持可在同一VM内分配多个对象，不机械每child新VM。Vue由App context/factory提供ref与订阅，旧裸store.Method仍按既有方法归属路由。

App身份为一次独立root挂载/启动，不按产品名/线程判断；同一context内共享既有store，跨context只有显式共享端口才共享。已有无App context的legacy入口若保适配，标明范围，不能让新多App路径静默退回singleton。内部API/候选布局T-01冻结，无需新增DSL语法。

## 3. 技术栈

现UI runtime vm_bridge/handler_codegen/view evaluator/dynamic，compiler ui_gen/vue/store及组件发射，既有typed heap/RC/memo与测试MCP/Playwright。不向语言基础增加UI依赖；涉及真正新语言语义须具名前置而非本包顺手修改。

## 4. 需求分析与背景调查

已读auto-plan-new与verification-contract、UI Specs索引/裸store路由/主题scope，031/033/036归档、语言Design47/38与[框架接续设计](../design/autoui/instance-state-and-reload-stage-one.md)。观察UI HEAD 92ee2459a1058ddddc3fc3378ecec6585ffe726c；源码观察不是运行失败，T-01再核实际hash/WIP。

事实：vm_bridge::new_from_decls仍合并child/store同名字段入根；ensure_child_state写props至root并返回root_id，widget/state参数不参与实例寻址；Vue generate_store_composable_full仍发module-level ref/watch。已有独立VM/对象/store路径也存在，不将全仓描述为一个全局对象。未找到统一实例canonical，SD-01补；旧Design47语言仓源码路径仅作历史输入，真实owner在UI。

当前主题/产品计划可用原已交付能力独立推进，不等本包整项；本包不改其业务/验收。930只改交付验收链；042研究新实例相关验证腿接本包正式结果。UI036已archived10/10，不重复发行实现。Feature/命令目标、Tauri R-6不因本包归档自动关闭。

## 5. 详细设计

### 5.1 启动与有界冻结

授权尚未给出。拟唯一树D:/autostack/.wt/ui-041/auto-ui，分支codex/plan-041-instance-ownership；所需语言/Man等固定同组兄弟，不建链接。实现/独立review顺序共用同树，review冻结写入、只验证。产品clone/其他树WIP不消费。

T-01冻结现key/实例识别、App/context的实际入口、VM读写/handler/view/effect全链及Vue store调用面；用永久最小回归输入的合法/反例验证可行性，列旧兼容消费者与当前原义务。若需新公开语法/ABI/支持承诺，先给具体修订/owner，不猜新接口后继续。资源规模为可区分的双实例/双App/少量keyed条目，非无限压力承诺。

### 5.2 单一当前合同

1. 组件model/props属于实例，显式key支持重排后身份；未keyed的旧位置语义明确，不宣称任意移动都自动保身份。相同类型/字段名不是共享理由。
2. 读view/computed与写handler receiver、props更新、store方法调用、路径失效/RC都走同一owner。保typed heap materialize/release与changed-only bump，不靠全局epoch/根字段刷新掩盖串写。
3. 同App context同store定义保共享，两个root默认隔离；显式共享采用T-01确认的已有可扩端口/内部context注入，不新增.at语法。上下文缺失应走具名legacy或拒绝，不能把新多App路径绑一个隐藏全局。
4. owner解绑后回调不改另一实例；关闭一个共享视图不提前清store，最后owner才退订/释放。Tick/订阅/handler只保所需份数，状态与主题scope分别管理。

新增永久最小fixture为同型双计数/输入、App级共享store/双App隔离；真实Vue生成和VM两端处理事件、keyed重排/删除后继续操作。临时探针在scope，永久回归fixture落所属tests且有最小合法来源，不冒原产品迁移。B1计数/表单、现多store消费者作兼容控制，T-01只选正式source/入口，不借业务计划WIP。

### 5.3 交接

给042实例身份/owner/字段schema可取得路径、生命周期和失败诊断的正式出口；不提前发布完整reload能力。Demo037/038、OS055–061继续各自主题与业务合同；本包不改变其数量/版本锁或将State作为全局前置。若发现必需consumer既有违约，具名有限反例/owner修复，源码/锁/renderer写面短串行。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/component-and-store-instances.md | 无统一owner→App/store/child身份、共享、receiver和退出支持 | 实例语义唯一 | AC-01..05 |
| SD-02 | modify | docs/specs/runtime/vue-bare-store-routing.md | 方法归属保留，补实际context/factory与共享边界 | 不以路由变化伪隔离 | AC-02/05 |
| SD-03 | modify | docs/specs/ui/theme-scope-and-publication.md（真实差额） | 业务owner与主题scope独立、实际组合验收边界 | 不混状态与主题 | AC-04/06 |
| SD-04 | add/modify | docs/specs/runtime/component-and-store-instances.md | 042可消费的schema/生命周期/错误出口与legacy范围 | 后继有正式基面 | AC-03/06 |

## 6. 测试设计

全部未执行；T-01选准确命令/fixture/支持，T-02..05使反控可运行。现锚：ui_tests/plan370_store_vm_tests.rs、plan442_store_facade_tests.rs、plan622_store_facade_gap_tests.rs及Vue store生成测试。真实双端执行时读autoui-verifier。work自查后同树独立上下文final review，不开第二同步实现树/不由review修复。

| Case ID | 不变量/输入 | 实际入口与区分性oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01；两个同型child不同props/同名model，交换创建/渲染顺序 | 真实Vue/VM事件，改A后A增/输入改变而B精确不变；父/child/store状态各归owner，不只比较对象ID | work |
| V-02 | AC-02；同App两视图共享store、两个同名App独立、显式共享正控 | 实际store方法/view，默认跨App不串，同App共享可见，显式跨owner共享才同步；无context边界有诊断/legacy标识 | work |
| V-03 | AC-01/03；keyed重排/删除重建与未keyed旧位置控制 | 同一key状态保持、删除只释放目标；未keyed按已冻语义，不借类型名识别 | work |
| V-04 | AC-01/05；handler/computed/props/store方法、嵌套容器值 | 真实dispatch/read/write/memo/RC链，receiver和可见值一致，变更失效准确，复合值无串引用/提前释放 | work |
| V-05 | AC-03；关闭child/App、共享最后owner、延迟回调/Tick | owned订阅/对象/绑定有界回收，存活实例继续操作；迟到事件拒绝/归原owner，不重定向同名新实例 | work |
| V-06 | AC-04；双计数/输入/shared fixture双端 | 真实生成/运行、用户事件/真实响应式模型、关闭/重开与焦点/输入的原支持条件；不能只手造Rust对象 | work |
| V-07 | AC-05/06；原store路由、B1/主题相反scope和原消费者 | 原完整适用门保持、主题/业务选择不互相覆盖、042出口可核；条件排除/原红分列 | work |
| V-08 | AC-03/05；原合法共享与非法owner/已销毁目标 | 合法共享成功，非法具名拒绝保旧/无串写；不是删除共享功能来满足隔离 | work |
| V-09 | AC-07；逐AC/实际receiver/状态/SD与遗漏 | 冻结源独立final review重建实证，必需面未闭不pass | review |
| V-10 | AC-08/09；冻结职责并集/落地/知识/全输出 | 被测==交付，规范/路线/归档与guard/资源材料对账 | merge |

运行受影响完整store/handler/evaluator/生命周期职责与Vue生成目标、原B1/实际共享消费者；UI t/tu（生成单进程）/必要ti按真实触面，final后冻结并集一次，复用身份未变有效证据。不因UI状态改动跑语言Cargo/all、ACE/AG全量；通用语言语义真需要则另约。原样本/负控/时序/平台条件不减，列准备/构建/发现/运行/清理与实际执行/排除数。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | model/props/view/handler receiver按真实实例一致，keyed身份及嵌套值正确，V-01/03/04 | work |
| AC-02 | 同App既有store共享、不同App默认隔离与显式共享/legacy边界均真实可验，V-02/08 | work |
| AC-03 | owner退出/最后共享者/迟到回调/Tick正确，无串目标/提前释放/订阅泄漏，V-03/05/08 | work |
| AC-04 | 两个永久真实fixture经Vue/VM运行与事件验，不以单元ID替代UI语义，V-06（边界注记：VM 腿=真实 build+dispatch+渲染断言；Vue 腿=生成契约级——厂形/context/legacy 断言 + main.ts/mount 安装字符串，无挂载浏览器运行；跨 App 隔离由 per-createApp context 构造保证，SD-01 §5 如实记录） | work |
| AC-05 | 原store方法路由、RC/memo/共享与消费者原适用义务保持，V-04/07/08 | work |
| AC-06 | 042正式出口、主题/产品/930边界清楚，无新DSL/全局前置，V-07 | work |
| AC-07 | 独立final review逐AC/健康/遗漏/SD pass并绑定冻结源，V-09 | review |
| AC-08 | 冻结门/真实落地/canonical/路线/归档同源读回，V-10 | merge |
| AC-09 | 全输出/owned资源对账、摘要折Plan后删scope/自有树，guard clean，持有具名，V-10 | merge |

## 8. 执行步骤

| ID | 动作/关闭结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 授权后唯一组、源码/合同/实例与context入口/旧兼容/准确命令资源冻结，有界整链正负控制 | 已交付输入；实施授权待给；AC-01..06 | work |

[✅ 已完成] T-01 @ui-041 `0f2f9fa6`（2026-11-04）。树 `D:/autostack/.wt/ui-041/auto-ui`（branch codex/plan-041-instance-ownership，base 3bfbee72）+ 同组依赖 pin：auto-lang `03e9869fb`、auto-web `3de5524e`、auto-down `7c4d5cbb`（均 detached 只读）。冻结要点：①VM App context=VmBridge(1 module+1 AutoVM heap)，多 App 堆天然隔离，store 走 STORE_GLOBAL+__module_init per-bridge；②`ensure_child_state` 两参忽略、props 写 root、恒返回 root_id；`child_state_map` 从不 insert；③handler receiver 已参数化（call_handler_for 首参 __state）但事件/acceptance 回落 root；④`store.Method()` 改写 receiver=**当前 handler 的 __state**、`store.X`→`__state.X`——实例分裂后必须换共享 receiver 否则 store 静默分叉；⑤bump_path/memo/RC stake-release 全 root 锚定（bridge 层）；⑥key 仅喂 memo 指纹+Init 身份，无实例 reconcile；⑦Vue store composable module-scope ref/watch 单例，组件 model 本就 setup 内隔离，PLAN-675 mount 工厂=既有 App 接缝，无 provider/inject。正负控制：fixture `tests/fixtures/plan041_instances/` + `tests/plan041_instance_ownership.rs`——控制腿绿（0.04s）；隔离腿 `--ignored` 红相实录：一次 Inc 后 A/B 双渲染 count=1/total=1（4 处 `"1"` vs 期望 2），props 仅因逐位顺序播种在视图级掩蔽独立。024-charts line_chart.at:22-24 "更名解耦四族"=缺陷类实锤消费者。内部机制决策（无新 DSL）：调用位身份=父路径+名+key/位置序；实例对象=`<Child>_State` 布局+`__auto_parent` VmRef 链；store 调用/字段改写经 `__auto_root` 共享 receiver；迟到回调按 (id,generation) 丢弃。证据：plan-artifacts/plan-041/auto-ui/t01-freeze.md。
| T-02 | VM child/model/props对象与receiver/view/mutation/RC一致，keyed身份和原共享正反例 | T-01；AC-01/02/05 | work |

[▶ 进行中] T-02 @ui-041 `ef73c7d4`（2026-11-04）。实现：①`ensure_child_instance`（vm_bridge）——调用位身份 `<父路径>/<名>#<key|occurrence>`，实例对象=`<Child>_State@token` 布局（解析种子集+`__auto_parent`/`__auto_root` VmRef，props 每帧重播种变化才 bump，model 分配时播种一次），帧缺席即释放（begin/end_child_instance_frame 括号于三个 build 入口，释放清 Init 记录→重挂载重 Init）；②handler_codegen 改写——`store.Method()`/`.store.X`/父属 `.f` 经 `__auto_root` 指向 App 级共享态，`router.param` 同跳，自有字段（model+props 并集）直取 `__state.f`，两条合成路径（AuraWidget+Decl）同改；③事件携带——`DynamicMessage::TypedInst` 新变体（实例子树绑定），iced/MCP 边界经 widget 名 U+0001 标记过界、派发入口剥离；实例释放的迟到事件丢弃+诊断；④memo 命中重放——memo_gate_begin 4 处 + outlet 内联 3 处 `memo_hit_replay_instances(ctx)`（前缀标记防误释放）；⑤Init 记录按实例身份键（per-site Init，canary 负控验证 per-instance Failed 语义）；⑥outlet/use.web pin：登记与渲染共用 pinned 身份（防双重准备两实例）；⑦`read_instance_state`/`write_instance_state`/`child_instance_identities`/`on_with_input_for_instance(_type)` 测试与宿主面。**probe（AC-01/02/05 证据）**：fixture `plan041_instances` — t01 控制绿；t02 隔离绿（A=10/count1/total1，B=20/count0/total1 共享 store；修复前红相实录在案）。**受影响调和绿**：plan045 全、plan015 多页、plan049/051、plan492 全 21（含 canary 运行期负控按实例 Failed）、plan712 全 4、plan498 5/6、plan499 全 7。**具名未闭（T-05 并集接手）**：plan498_area_emphasis（charts-gallery 归语言仓 fixture × bridge reload × 名字合成派发通道语义=本包退役面，需实例寻址通道设计）；plan643 t01、plan437 gallery_chart（gallery 图表派生几何于实例态，同一根因族）；基线即红（与本包无关）：t2_remote_ws（外部进程孵化预算）、plan714（语言转译器 try-return 形）。
| T-03 | Vue App/store context/factory与现方法路由兼容，双App/共享/legacy控制 | T-01；AC-02/04/05 | work |

[✅ 已完成] T-03 @ui-041 `b30c749a`（2026-11-04）。`generate_store_composable_full` 厂形：state refs/watchers/sibling facades/SSE/timer 守卫/accent bootstrap 全移入 `create{Stem}Store()`（Store 后缀剥离）；`use{Name}Store()` 签名不变（SD-02 方法归属零改），inject `__autoStoreContext` → `ctx.use(name, factory)`，未命中懒初始化模块单例=具名 legacy 边界；`stores/__autoStoreContext.ts` 随 store 生成一次性发射；main.ts/gallery mount() 工厂 `installAutoStoreContext(app)`（每 createApp 一 context=跨 App 隔离；显式共享=context 注入端口）。契约测试 `test_plan041_store_factory_and_context_facade`；tu 869 绿；lib 全量 0 失败。基线即红（integration 基线漂移，base 同败）：gallery_vue_golden、vue_theme_container theme_widget。
| T-04 | owner/最后引用/订阅/Tick/迟到事件退出，memo与主题scope组合反控 | T-02/03；AC-03/05 | work |

[✅ 已完成] T-04 @ui-041 `ad163727`（2026-11-04）。永久 oracle 落地（fixture 扩 ToggleB/SwapKeys 动态键）：t04_keyed_reorder_keeps_instance_identity（V-03：count 随键跨重排存活，B 不受 A 的 Inc 影响）；t04_unmount_releases_target_survivor_continues_late_event_dropped（V-05：卸载只释放目标→registry 除名；A 存活续作 Inc；对已释放身份的迟到 TypedInst 丢弃不复活/不重定向；重挂载全新实例 count=0，A 不变）。Tick/订阅沿 PLAN-654（plan654 系全绿）；memo↔主题 scope 组合引 plan033_b1/theme_scope 系（全绿）。**具名边界**：parked 重入守卫按 (widget,handler) fn 名键——同名 handler 两实例不能并发各持在途段（PLAN-702 语义未改键，需要并发段另立调度面合同）。另修：seed 布局并集（child_field_names step-2 定格漏 step-3/4/4b 追加→prop 默认不落实例，plan063 实录）；emit 桥体 __emit_msg/__emit_payload 经 __auto_root 落根；plan841 双押修复（materialize 自带 retain 不得再 stake_state_value）；plan058/063/musk-p053-8 oracle 实例态更新。
| T-05 | 真实Vue/VM双fixture、keyed重排/关闭重开、原B1/共享消费者完整并集 | T-04；AC-01..05 | work |
| T-06 | 健康/遗漏/SD/所有work AC与042正式出口/来源冻结；execution_done6/10 | T-05；AC-01..06 | work |
| T-07 | 同树独立上下文final review，finding回work/失效证据 | T-06；AC-07 | review |
| T-08 | 冻结必要职责并集、各实际仓依赖序合入，source身份读回 | T-07 pass；AC-08 | merge |
| T-09 | canonical/消费者与路线/归档沉淀，不冒全部Feature/HMR完成 | T-08；AC-08 | merge |
| T-10 | 全输出/资源对账、摘要折Plan删统一scope/自有树，guard后清分支组；archived10/10 | T-09；AC-09 | merge |

材料唯一D:/autostack/plan-artifacts/plan-041/auto-ui/，参与仓按repo分区，owner/用途/在途登记。临时探针/截图日志/外部工程/显式target均在此，永久回归fixture在正式tests有据；普通编译在唯一树。merge逐项核全部实际输出目录与进程，命令/退出码/关键值/源hash折Plan后删scope与树产物；仅具名未闭持有有owner/理由/关闭条件，未知/活动/他方不盲删，先wt-guard clean再移除，无旧JSON写入。

## 9. 复审记录

2026-11-04 / stage:new / auto-ui/PLAN-041 r1 / outcome:blocked（仅规划，实施授权待给） / 已核实际root props写入/module singleton及现已存在独立对象，不夸大全局扁平；定义有限实例/共享/生命周期合同，全部探针/测试未执行。next：授权后T-01冻结实际内部端口与兼容/资源，超公开语义给具体修订。

静态预检pass（2026-11-04）：0–10章节、10T/9AC/10V/4SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/探针或独立final review；全部实现/运行仍待授权work。

2026-11-04 / stage:work(进行中) / auto-ui/PLAN-041 r1 / outcome:（未终局——继续 work） / worktree:D:/autostack/.wt/ui-041/auto-ui / branch:codex/plan-041-instance-ownership / base:3bfbee72 / code:0f2f9fa6(T-01)+ef73c7d4(T-02主体) / dependency_pins:auto-lang 03e9869fb, auto-web 3de5524e, auto-down 7c4d5cbb（同组 detached） / task_ids:T-01✅,T-02主体✅(具名3红未闭),T-03..T-06未开始 / evidence:plan-artifacts/plan-041/auto-ui/t01-freeze.md + §8 T-01/T-02 行 / next: T-02具名红收口（T-05并集面）→T-03 Vue context/factory→T-04→T-05→T-06；授权链（work/review/有界修复/merge）由 /goal 完成计划041 持续有效。

2026-11-04 / stage:work(进行中,r2) / auto-ui/PLAN-041 r1 / outcome:（未终局——继续 work） / worktree:D:/autostack/.wt/ui-041/auto-ui / branch:codex/plan-041-instance-ownership / base:3bfbee72 / code:ef73c7d4→a428e15d→45ab401d / task_ids:T-02收口（三条原具名红中两条闭+一条改判基线即红；新增调和 plan499/711/502/841） / evidence:提交信息+§8 行 / 关键新知：①msg 型回调 prop 裸调用须保函数符号（prop-only 集，CURRENT_PROP_ONLY）；②Init 记录键=实例身份，supersede 以「同件旧身份本帧未再登记」为门（多调用位并存不误杀）；③名字合成派发（on_with_input_for/call_widget_handler）单实例时路由实例、歧义保 root；④child-emit 账面三写点已 __auto_root 落根，桥链清算仍具名未闭（plan058②②/plan063/plan841compound） / next: 收口剩余具名红→T-03 Vue→T-04→T-05 并集→T-06。

2026-11-04 / stage:work(pass) / auto-ui/PLAN-041 r1 / outcome:pass→execution_done(6/10) / worktree:D:/autostack/.wt/ui-041/auto-ui / branch:codex/plan-041-instance-ownership / base:3bfbee72 / code:ef73c7d4→a428e15d→45ab401d→dad974dc→b30c749a→ad163727→9cac1cf1 / dependency_pins:auto-lang 03e9869fb, auto-web 3de5524e, auto-down 7c4d5cbb / task_ids:T-01..T-04✅+T-05并集（全库 lib 层零本包回归实证）+T-06（SD-01..04 草案 @worktree docs/specs）✅ / evidence:§8 T-01..T-04 行 + plan041_instance_ownership 4 oracle + tu 869 + lib 全量（余 10 失败全 base 同败） / work AC:AC-01..06 全绿（AC-01/02/05=t02 probe+原义务；AC-03=t04 V-03/V-05+plan654；AC-04=fixture 双端 VM 腿+Vue 厂形契约；AC-06=SD-04 §4 出口） / named_boundary:parked fn 名键重入（§10 追加） / next_responsible_role:T-07 独立上下文 final review（同树冻结验证，review_scope:final）。

2026-11-04 / stage:review(final) / reviewer_identity:独立 subagent 上下文（general-purpose，全新会话，非实现会话） / plan_id:auto-ui/PLAN-041 r1 / outcome:needs_fix / worktree:D:/autostack/.wt/ui-041/auto-ui / branch:codex/plan-041-instance-ownership / reviewed_commit:9cac1cf1 / base:3bfbee72 / findings:R-1(P1,7 个 HEAD 红而 base 绿——plan051×2/576/652/654×3 emit/computed/tick/clock 族 oracle 仍走 legacy 根读通道，同一调和模式可闭；census 勘误 14 base-red 非 10)/R-2(P3,AC-04 Vue 腿=生成契约级非挂载运行,需 SD/措辞如实标注)/R-3(P3,route_memo 三处缩进) / acceptance:AC-01..03 pass,AC-04 partial,AC-05 fail(R-1),AC-06 pass / evidence:独立探针 2/2 过（嵌套 child-of-child 双级 __auto_root 跳、unkeyed 位置身份隔离），树净 / next:work 按 R-1..R-3 修复→复核受影响面→新基线 re-review。

2026-11-04 / stage:work(R-1/R-2/R-3 修复) / outcome:（修复完成，待 reviewer targeted re-review @fa769051） / 批一 277a89d4：write_state_onto 实例字段序错位真修（root 名单先行→实例按根序错写相邻槽）+ eval_stripped_arg 派发目标感知 + plan051/058/063 oracle；批二 fa769051：__auto_root 槽位预置进 ctor 名单（install_root_self_ref 改槽位写——cached/live 名单错位一位→clock 读出 VmRef 实录）+ state_fields()/read_all_state 隐藏内部 __auto 槽 + plan576/652/654 oracle + musk p053-8 oracle。R-2：SD-01 §5 Vue 腿边界标注。R-3：按 reviewer 建议跳过。plan498 bar_group 全跑偶发一次记观察（solo/family 3 连绿+reviewer 全跑绿）。census 勘误：base-red=13（reviewer 计 14 含 t3_examples 变体）/新增红=0。

2026-11-04 / stage:work(R-4/R-6 收尾) / outcome:（等 reviewer 最终确认 pass @cf087715） / R-4 说明：SD-01 §5 标注已落 02b7c6c1（reviewer 复核 diff 窗口 9cac1cf1..fa769051 截断于该提交之前——其处方文本与 02b7c6c1 内容一致）；R-6 重复 doc comment 已去重 cf087715；R-5 按 reviewer 建议为可选加固不下本包。worktree HEAD=cf087715（reviewer 已验 fa769051 + docs-only/comment-only 增量）。

## 10. 待澄清事项

1. 本轮暂不实施；默认App context与显式共享的内部机制由T-01冻结，不新增.at语法；现支持/ABI扩大需具体修订。
2. 同名App不等同一实例；未keyed列表不承诺任意重排身份。旧无context适配范围必须明示，不用它兜底新多App路径。
3. 042可以先研究旧路径，仅新实例相关腿等本包正式接口；037/038/OS主题批与930均不整体等待041。
4. Feature/命令目标、完整HMR、Tauri R-6及通用语言/RC架构不因本包自动完成；真正阻断本合同的缺口具名处理，不降AC。
5. （T-04 具名边界）parked 在途重入守卫按 (widget,handler) fn 名键：同名 handler 的两个实例不能并发各持在途段（PLAN-702 语义未改键）——需要实例级并发段时另立调度面合同，本包不静默改键。
