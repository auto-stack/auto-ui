---
plan_id: PLAN-030
status: drafting
feature_name: AutoGraphics三维具名引用与跨组件接线收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-027, auto-ui/PLAN-028]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-symbolic-component-bindings.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-030：三维具名引用与跨组件接线收口

## 0. 变更摘要

在028有限对象/触发端口上补Slot/Template/Pair具名引用、可传入普通组件的只读ActorRef3D与引用参数。作者以同定义多实例的普通逻辑Node持局部句柄/状态/msg/on，无数字slot/pair/scene ID或字符串拼接绑定，不把所有收集/巡逻状态塞回Player。

[共同设计](../design/autographics/node3d-symbolic-bindings-and-g11-migration.md)。drafting r1、0/10；仅规划、暂不实施，不建树/源码/资产/探针、不构建/跑GPU/OS自动键鼠、不创建agent。028已交付且不重开；AG013完整内容消费等本计划实际交付，AG012仍固定027独立在途。

## 1. 目标

1. 具名Slot/Template/Pair symbol与静态角色leaf引用可经只读参数传给普通有状态Node；作者不手填索引，编译器解析到原数值ABI和identity表。
2. 三同定义Coin、Player、Hazard/Goal/Void与根关卡状态分别局部持有，复用020消息/receiver；共享不可变模板与固定Node寿命分开。
3. 根显式组织推进/事件/提交/重开顺序，支持原G11玩法与25项旧oracle，另证非游戏例/跨父多实例；不生成Rust业务或自动读一/三策略。
4. 原32预算、八状态组件/八slot/八pair、120base+8reserved/512mesh/VM/源码/资源包络内全链先证，旧028等输出/反例冻结。

非目标：新对象运行时/触发算法/队列/反射/完整借用系统、动态Node拓扑/跨场景多挂载/多角色、动态TRS、3D查询/跨维组合/纹理动画/声音HUD、多呈现/Life.Frame、生产UI/ACE/AOT/ECS/CPC/跨端、抬额或Gallery内容修改。object3d仍纯模板，不为模板另塞业务model/on。

## 2. 架构方案

普通use/固定Node→020局部SoA/receiver＋027/028三维表示→下降期名义引用/实例路径/参数绑定表→生成真实Auto→既有Triggers3V1（108/profile19）→原Session/Play/Host/GPU。新增仅作者绑定表示/校验与转译；原objects/triggers/round单元/费用/代次/宿主准入保持。

新显式作者profile暂称components-3d-bindings，准确拼写T-02冻结，旧components-3d-events字符串端口/数值行为/生成字节/准入反例不变。不靠全局新增保留词改变旧程序，也不承诺任意Auto类型在当前旧SDK可用。名义引用由作者层检查，运行仍原精确int/常量表。

拟唯一实施/同树顺序复审树D:/autostack/.wt/ui-030/auto-ui，codex/plan-030-dev；本轮未创建。review为同树不同独立上下文，期间work停写，不另建同步review树。UI007/029及语言路线共享接缝开工核实际owner；AG012固定读027，不依赖030或跟升。

## 3. 技术栈

已核dialect src/node2d/{ast,parse}.rs；pilot src/{load,translate,lower_components,lower_components3d,lower_components3d_events}.rs，fixtures/components3d-events/{g11-main,sensor}/，driver tests/node3d_events_flow.rs及原runtime_objects/runtime_triggers/runtime_round服务。先复用020跨父receiver，不复制另一组件系统。

拟新增pilot src/node3d_binding_refs.rs、src/lower_components3d_bindings.rs、fixtures/components3d-bindings/、tests/node3d_bindings_gates.rs、tools/verify_plan030.py与driver tests/node3d_bindings_flow.rs（新路径，T-02可冻结等价布局）。永久回归夹具归产品；临时探针/输出不入项目。generator/固定SDK driver两图独立。

## 4. 需求分析与背景调查

用户2026-11-02告知028完成、AG012在途并要求规划后继；沿原分仓/原预算验收/暂不实施边界，本轮仅合同/设计/编号/导航，不接管在途实现/联系agent。补此能力为了既定可读性/局部状态目标，不认定028公共服务未交付，不追改其归档合同。

基线auto-ui01f99fdc9b77ed8fdb4fdbb82f1321f77d85e7e2；028实际delivery ebac1cd，独立final pass/冻结/归档/清理齐，canonical [028 Spec](../specs/auto-graphics/node3d-instances-triggers-and-rounds.md) SHAbe4d9c152871c70c19d3524237521f81e3cde0d3da956325e2bf7cf3119e540d。020/027已交付；UI007有独立树与主检出簿记WIP、未跟踪crates/auto-ui-runtime/test与docs/design/autoui/，本轮不碰。

权威：[020组件](../specs/auto-graphics/stateful-node-components.md)、[027角色相机](../specs/auto-graphics/node3d-character-and-runtime-camera.md)、[028](../specs/auto-graphics/node3d-instances-triggers-and-rounds.md)、旧[对象](../../../auto-lang/docs/specs/auto-graphics/runtime-objects.md)/[触发与新局](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)。

实际源码SHA256：lower_components3d_events.rs=f3f0dd89adf616b0087146e259f58aea0546f6cc74b819010b1d28b51f2b7fb9；translate.rs=bc16e8c70d42a24b803372f28555db4f9c3881672fadd4576c66f022f5a63ceb；028 g11-main/character.at=2f40c4288d34ad5da94b3184b93782a92909302e4bcc18c2ce1ca9ebbe102f0f。受信runtime_objects_v1.at SHAfe9a8948270d6a8756661830aeb1c6ce7f35c18eb2f3e65d5befaf8595408dc5、runtime_triggers_v1.at SHAb2b75b168570d8cf86a470b65511d3923a04d0ae0848caa694a2b619fc0b8ce1。

已观察缺口：028 object3d拒绝model/msg/on，spawn/bind要求字符串字面量，bind subject只认当前定义的refs元素，event.pair返回数值序号；现G11夹具Character持h0..h5/hy/bag/state并按p==0..5分派。普通组件/非游戏sensor已可用，不代表具名资源池参数或跨组件ActorRef已完成。本片只补这些边界，不让内容仓用常量0..5或自制转译器伪装具名作者体验。

只读Gallery基线82b881e3；原G11 checks SHAfae22bd4cdc2e3071e7c35004d75b4976b8a5bc158715be7b914993fecc9f90f，主/alt SHA83a210e0802255db93fed64e4b966c690153297a4bdffd5b65520b73df52f75d/e5ea5c4cec89554028bfca677b71b06ed246ccfc561c8c3a1adfacfed1f3abc3。AG011交付f35236dc并清理，实际当前13/22；AG012用户告知在途，任务和树以owner记录核实。

## 5. 详细设计

### 5.1 开工与有界先证

T-01核020/027/028实际源码/Spec/final/冻结/清理、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/681文件内容锁与准确依赖/工具链；历史对象缺失只用受检固定副本，不换当前SDK。核007/029/AG012真实owner/共享写面，开工才建唯一树/材料scope并登记。

T-02最多两种复用接法，全链证明符号参数→实例化/跨Node引用→生成Auto→真实Triggers3V1/Host/GPU：根+Player+三同定义Coin+Hazard+Goal+Void恰八状态Node、原六实例/六pair/三模板、Start高度/主alt/新局、非游戏双父receiver、全部32分支。冻结准确语法/profile/名义类型/路径作用域/初始化顺序/事件消费/费用/身份表。任何需加第九状态Node/抬额/改旧单元或隐藏调度才成立，暂停依赖任务回规划，不交给内容绕过。

### 5.2 引用支持域与解析

拟SlotRef3D/TemplateRef3D/PairRef3D/ActorRef3D四类只读端口（准确拼写T-02冻结）：根声明具名池和模板导入、静态实例路径提供绑定值；实例参数逐项校验类别/作用域/唯一性/必需性。Slot与Pair不能互传，Template与运行Instance不能互传，group/2D/未知/歧义元素不能当Actor；同类但越作用域引用必须由作用域门拒绝，不能靠其他type错误假证。

支持根向子、子向其子只读转传，以及同定义三实例各绑定不同symbol；不提供任意另一组件model的可写访问。初始化参数一次建立，修改自己的model不会改别的实例绑定；运行Instance handle仍原Run/slot/generation int，由逻辑Node局部字段持有，旧代/错Run的sticky33保持，不声称静态借用系统。

具名pair比较/常量解析不重新调用event.pair；事件游标实际读取pair/phase/source仍各1。符号降低为原slot/pair/template row/subject stable ID；identity记录声明路径→kind/ordinal映射和接收方，不让作者抄roles表数字。重排普通模块/组件/模板声明后，所有逻辑引用随新表正确更新；物理ID和生成字节可变，不承诺旧ID不变。重排pair池本身则事件仍按新pair注册序输出，不能假定并发胜败顺序不受显式注册序影响；无竞争轨迹/三收集语义与绑对目标必须保持。

### 5.3 组件与作者调度

object3d保持纯静态模板、支持域沿028；Coin等是独立普通逻辑Node，可无额外可见leaf，持实例handle/一次收集状态/msg/on；三个同定义Coin分别绑定槽/模板/pair/角色引用。Player持运动，Hazard持hy/hdir，根持bag/result与关卡调度，Goal/Void持自身事件规则；纯Ground/模板不占八状态Node计数。Node寿命与object despawn分开，不自动删除SoA/共享模型。

复用020同步子msg/祖先emit。作者可在根Life.Tick先显式调用Player消息推进运动并回报moved，再有限消费上一Tick事件并向目标组件投递，最后按原n==0提交Hazard位置、按原条件读Restart请求新局。接受等价的有限组织，但不得根/子Life.Tick与显式消息重复推进、根先Tick导致晚一拍、或编译器按名字生成bag++/胜败/自动Idle。

支持具名事件绑定或显式分派，准确方案T-02冻结；若新增便捷绑定，须公开其实际next/pair/phase/source调用次数、未绑定事件消费/顺序及预算，不能隐形读完八条/免费读取/跨Tick队列。保原G11移动读一、idle最多三、有事件不提交巡逻位置而hy照常推进；第二非游戏例能用不同有限策略，证明不是G11专有调度。

### 5.4 原预算、兼容与来源

全Node共享32credits；move26、每对象调用1、next与每getter1、bind1、round请求原价，相机basis读0。类型映射/具名常量无需新增服务读取，但VM发射/handler成本必须实测。保原main speed2/hbase2.4/hdir.05、alt speed2.6/hbase.9/hdir.08（折返仍±.05），三Mob模板scale .5/.6/.7、编辑角色y1/Start y.5、相机8/45、原轨迹和同tick事件/下一callback读取。

保8状态Node/8slot/8pair、120base+8reserved=128、512潜在mesh、代次不回绕/32 lifecycle写/第33写37、空间第33计费10、256KiB user/96KiB managed/352KiB组合、1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s、8ms pump与原资源8身份/32MiB raw/CPU-GPU各64MiB。分清各种计数单位，不每Node独立给池；原稀疏/密集批准样本和反例不减。

旧028字符串接口/有限夹具与027/020等冻结输出/拒绝域不变；复用原108/19与独立准入、事件/几何锁、同Run/tick发布/accepted新Run与last-good。受信单元零改为首选，若需语义变化回规划，不假借作者收口改变公共运行合同。精确source-build绑定SDK681/源码/依赖锁/toolchain/参数、generator/runner实际hash、模块/资源许可与identity；A→B仅闭包冷再生/真Run、SaveAs关开、坏件拒绝→还原再Run、20轮newRun/churn/Stop最后owner平坦。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-symbolic-component-bindings.md | 028有限字符串/序号/本定义subject→新显式名义symbol参数与跨NodeActor引用、局部逻辑组件及作者有限调度，旧档不变 | 落实可读性/局部状态并给AG013固定消费契约 | AC-01..06/08 |

## 6. 测试设计

全部新入口/夹具待T-03..06建立执行，非当前绿证据。无用户指定实现/内部/最终review人选；未来核owner，同权威树顺序、final独立上下文，internal pass不代final，本合同不授权创建agent。

| ID/AC/owner_stage | 不变量/合法与非法输入/状态 | 实际入口/区分oracle/边界 |
|---|---|---|
| V-S01 / AC-01/02 / work | 同定义三Coin、嵌套转传/静态跨NodeActor、逻辑Node无伪几何；错类/同类越scope/歧义/只读参数赋值非法 | 新node3d_bindings_gates→真generator/identity/closure与Auto局部值，位置诊断/无半产物；不是type错误代scope验证 |
| V-S02 / AC-01/02/03 / work | 八状态Node原G11主alt/三模板/Start/25旧oracle；改变普通声明顺序/绑定符号 | 新node3d_bindings_flow真实Session/Host/GPU＋独立几何/句柄/事件与业务预期；引用跟新表不串，不能要求原物理ID不变 |
| V-S03 / AC-02/03 / work | 两父同定义receiver、非游戏第二有限策略；错目标/双推进/旧slot代/重复读取非法 | 实际typed参数/子msg/祖先emit→新源Auto/原准入，逐实例值/事件顺序/错误33与锁40..42；坏绑定变异真实被判失败 |
| V-S04 / AC-03/04 / work | 所有移动/收集/idle三件/未齐目标/终局Restart/巡逻分支与额外读取 | 真生成源调用计数＋VM/Host轨迹，原32内/第33fault10、写帽37、8状态/8槽各界；拒绝免费getter/隐式Idle/少投递/迟一Tick变体 |
| V-S05 / AC-03/05 / work | Pause/Step/fresh/held、失败新局/旧Run、20轮owner、旧028/027档 | 原Host/真GPU＋新源：局部状态/代次/相机重置、last-good/Stop文档Undo不变，旧字节/错误域保持；不冒称OS键鼠实测 |
| V-S06 / AC-05/06 / work | A→B/SaveAs/仅闭包再生、缺helper/坏资源/错工具/partial收据拒绝→恢复 | 新verify_plan030＋真工具/runner/Host/GPU，实际来源与闭包/再Run；覆盖新固定模式，非全AG/跨端或Gallery交付 |

### 原G11/3d二十五项（全部保留）

`import_three_templates`、`three_distinct_shard_templates`、`start_without_fault`、`tick0_six_pairs_bound`、`tick0_six_instances`、`orbit_camera_from_source`、`walk_collects_three_and_wins`、`three_distinct_ids_each_once`、`windowless_png_three`、`alt_fails_at_hazard`、`failed_state_freezes_motion`、`restart_new_run_tick0`、`restart_rebirths_items`、`orbit_changes_pixels`、`camera_relative_walk_rotates_world_direction`、`camera_relative_walk_reaches_the_void_zone`、`sphere_corner_outside_does_not_trigger`、`sphere_tangent_hits`、`aabb_and_sphere_are_distinguishable`、`same_tick_three_engine_block`、`same_tick_three_counted_once_each`、`old_generation_never_auto_reattaches`、`explicit_rebind_enters_new_generation`、`same_slot_new_generation_identity`、`stale_handle_operation_is_rejected`。

25项消费新符号/组件源，不保留旧ag_*字符串注入当新作者链通过；旧二维51项不混入此25报数。新增scope/改名重排/双父receiver/八状态/早目标/phase与round失败补充另列，不抵扣旧25。

拟实施树命令：`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_bindings_gates -- --test-threads=1`；driver改manifest为experimental/autographics-auto-driver/Cargo.toml、test为node3d_bindings_flow（沿028 64MiB测试线程栈）；`python experimental/autographics-node2d-pilot/tools/verify_plan030.py --controls`。均为新入口，T-03..05建立、T-02冻结准确CLI/输出。期望所有具名正控绿、负控真拒绝、无partial，不能仅靠结构检查自证。

受影响旧面：028 gates/flow/verify_plan028、027/020共享AST/translate与receiver的命名case/verifier、公共对象/触发与runtime_round_host适用命名反例。T-02按真实diff冻结定向并集；不为merge跑全AG或语言根crate全量，旧单元零变有据复用有效证据。GPU/窗口单实例、target/报告独占；OS键鼠/焦点自动化继续暂停，缺必需证据具名gap，准备/冷编译/运行/清理/整命令耗时与实际执行数分列。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 具名slot/template/pair/Actor静态跨Node只读参数，作者不写索引；合法转传/重排不串引用，错类/越scope具名拒绝 | V-S01/02 |
| AC-02 | work | 八组件内三同定义Coin局部状态/句柄/receiver隔离、Player/Hazard/根职责正确、无伪几何；非游戏正控证明无业务特判 | V-S01..03 |
| AC-03 | work | 新组件链原G11主alt25项/模板/Start/几何/事件/新局和独立GPU数值保持，所有补充负控真实触发 | V-S02..05 |
| AC-04 | work | 完整分支原32及写帽/八状态/八slot/八pair/120+8/512mesh/VM源码资源额保持，无隐式少读取/迟Tick/抬额 | V-S04/05 |
| AC-05 | work | 旧028/027/020受影响冻结输出/错误域不变，公共108/19语义/owner/last-good保持，无Rust业务 | V-S03/05/06 |
| AC-06 | work | source-build/SDK/锁/工具/资源许可闭包、A→B/保存重开/冷再生/恢复/20轮齐，SD候选/缺证边界清楚 | V-S05/06 |
| AC-07 | review | 同树独立final逐AC/实际diff/SD/告警/遗漏延期workaround审查，finding回work复核pass，绑定提交/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际合主，SD-01 canonical核字节/来源、归档导航，下游钉实际交付 | T-08/09 |
| AC-09 | merge | fresh guard清树/分支/空组，全部实际输出对账/摘要折Plan/删scope构建物，具名hold/最终hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；020/027/028实际交付 | 核§4/SDK681/依赖/owner/写面，唯一树和独占scope登记 | AC-01/05/06 | [ ] |
| T-02 | work | T-01 | 最多两接法完整八组件/主alt/非游戏先证，冻结symbol/API/CLI/初始化调度/32费用；缺口回规划 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | dialect/pilot名义引用/实例路径/参数转传与校验、新gate/fixture；旧028字节/边界不变 | AC-01/02/05 | [ ] |
| T-04 | work | T-03 | 新flow真实Auto/Host/GPU原25oracle＋V-S，局部逻辑组件/显式调度，无Gallery写入 | AC-02/03/05 | [ ] |
| T-05 | work | T-04 | verify_plan030、全部负控/预算/压力/receiver/round失败与独立数值/GPU | AC-01..05 | [ ] |
| T-06 | work | T-05 | 实际source-build/A→B/SaveAs/恢复/20轮、受影响兼容并集，SD候选/摘要移交 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同权威树独立final/精确revision绑定，finding回work复核；pass后merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/输入/定向门或有据复用，来源核后实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01 canonical/来源核、归档导航，AG013再钉实际交付，不写派生JSON | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清树/分支/空组，全部输出对账/删scope构建物/登记最终收据 | AC-09 | [ ] |

## 9. 复审记录

2026-11-02 / stage: new / plan_id: auto-ui/PLAN-030 / plan_revision: 1 / outcome: pass（合同可审阅移交，非可行性go/独立final/开工授权） / next: 用户启动后work T-01/T-02。0/10。已核028实际交付与源码的字符串/数值分派/本定义subject边界；新symbol/组件接线全部待实施。未建树/源码/资产/探针/提案Spec，未构建或跑GPU/OS自动化；不修改028归档合同。

2026-11-02规划静态预检：两合同0..10节、各10任务/9AC/6补充验证/1SD、work6/review1/merge3归属与drafting0/10通过；48个新增/修改本地链接可解析，依赖无环。两合同原G11三维25名逐字各保留一次，028实际Spec/实现与G11原源SHA复核匹配；本地独占取号030/013且活动/归档复扫唯一，未预取后继号。当前metadata活源13/22（原21迁12＋HUD新增1），`python tools/gallery.py validate --all` PASS 12逻辑项/22入口、许可一致。已见AG012 executing/r1、1/10、唯一ag-012树及owner T-01/固定027消费记录，不改其合同/实现。以上是静态规划/目录验证，非新链可行性go/运行验收/独立final；未建实施树/源码/探针/资产/提案Spec，未构建或跑GPU/OS自动化。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 新profile/名义端口/静态路径与参数转传准确拼写 | work T-02最多两接法完整先证后冻结，不扩大成核心语言类型系统 |
| D-02 | 八状态组件/原额下显式运动→分派→巡逻提交/重开时序 | work T-02/05逐分支实测，超额/迟Tick/双推进回规划，不抬额改玩法 |
| D-03 | pair重排的逻辑绑定与显式注册事件顺序 | work T-02/04同类scope/重排变异/无竞争原轨迹对照关闭，不假定物理ID或事件顺序不变 |
| D-04 | 007/029/AG012 owner与OS证据暂停 | work T-01核实际写面/短窗；缺硬件证据具名列出，不接管或自动开OS测试 |

材料唯一D:/autostack/plan-artifacts/plan-030/auto-ui/，开工登记owner/用途/在途；临时探针/日志/截图/包/消费者/外置target只归scope，永久产品/必要回归归仓，禁止attachments或第二证据目录。merge折摘要后全输出对账并删自有scope/构建物，仅具名未解决问题登记owner/理由/关闭条件持有。禁止worktree链接，清树先fresh D:/autostack/wt-guard.sh；旧specs.json只读，canonical在merge沉淀。
