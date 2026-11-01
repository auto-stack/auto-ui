---
plan_id: PLAN-028
status: drafting
feature_name: AutoGraphics三维实例触发与新局Node接线
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-024, auto-ui/PLAN-027, auto-ui/PLAN-001, auto-ui/PLAN-002]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-instances-triggers-and-rounds.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-028：三维实例、触发事件与新局Node接线

## 0. 变更摘要

在020/024/027上给普通Node局部model/msg/on、view-3d、use接入已交付objects3d/1、triggers3d/1和accepted新Run；typed模板/实例/绑定/事件与角色引用区分，保原slot/代次、几何/事件时序、32池、资源与VM包络。补原G11不同Mob模板静态缩放和Start初始放置能力，但不改旧027版本。

见[共同设计](../design/autographics/node3d-instances-triggers-and-gallery-motion.md)。drafting r1、0/10，仅规划，暂不实施。不建树/探针/源码/资产、不构建/跑GPU/自动键鼠、不创建agent；不接管AG011或AG012。G11/3d真实内容迁移待本计划实际交付后另立合同，不预取号。

## 1. 目标

1. 作者以typed模板symbol/实例引用声明或显式操纵有限对象与触发绑定，不手写数字slot/pair/scene ID；局部状态/事件/销毁和共享资源寿命明确、多实例隔离。
2. 输入/巡逻/计数/收集/胜败/重开条件均由真实Auto model/msg/on/helper定义，任意合法Node名/字段均可，不按示例或Coin/Goal/Fail命名生成业务。
3. 原G11/3d主alt、六实例/六pair、球/AABB、三物品与代次25项oracle在新作者链具名复现，保相机相对运动和原0/1tick、读事件与重开顺序。
4. 完整业务/phase分派/typed转换在原32内实测，120base+8slot、8pair、512mesh、原VM/源码额和owner/失败候选保持，提供可固定消费闭包。

非目标：Gallery项目修改、新几何算法/物理或对象运行时、多物理角色/无限集合/动态Node拓扑/自动销毁SoA/跨维触发、3D查询/纹理/动画/声音HUD组合、多呈现/Life.Frame、反向编辑同步、ACE/AOT/ECS/CPC/跨端、抬额或完整M2游戏交付。不把旧预算策略变成所有Node默认游戏规则。

## 2. 架构方案

普通use/组件/具名模板→020共享局部存储/receiver、027三维表示→新typed实例/触发/事件/round接线→真实生成Auto→既有Triggers3V1装配/Session→原Play独立几何准入/Host/GPU。复用001对象表和002触发/round服务，不造第二对象管理器/事件队列/VM，不复制017按例组织的业务降层或023有限game逻辑。

新显式作者profile暂称components-3d-events，准确名称T-02冻结；优先复用autographics.triggers3d/1（API108/profile19），不新增运行时版本/涨额度。旧027 components-3d-runtime及全部旧作者profile输出/负控保持；新语法能力只在新模式开放。对象运行事务沿既有准入，不扩为所有模块任意故障整帧回滚要求。

拟唯一实施与同树顺序复审树D:/autostack/.wt/ui-028/auto-ui、codex/plan-028-dev，当前未创建。独立final是同树不同上下文，写入在review期间暂停，不另建同步review树；AG011/012固定读027，本计划不要求它们跟升。

## 3. 技术栈

已核dialect src/node2d/{ast,parse}.rs；pilot src/{load,translate,lower_components,lower_components3d,lower_components3d_runtime,lower_g11}.rs、main/lib与fixtures/components3d-runtime/。driver api/{runtime_objects_v1,runtime_triggers_v1,character3d_v1,runtime_camera3d_v1}.at、authoring/{runtime_round,host,play,behavior}等实际模块，以及tests/{runtime_objects_api,runtime_objects_host,runtime_triggers_api,runtime_round_host,runtime_trigger_samples,node3d_runtime_flow}.rs。

拟新增pilot src/lower_components3d_events.rs、fixtures/components3d-events/、tests/node3d_events_gates.rs、tools/verify_plan028.py，driver tests/node3d_events_flow.rs（均新路径，T-02可冻结等价局部布局）；永久反例归产品测试，临时先证不入仓。SDK固定副本和generator两依赖图独立，不拍平。

## 4. 需求分析与背景调查

授权：用户2026-11-02告知027已完成、AG011开发中，要求规划下一阶段；延续上一轮分仓、原预算/验收、暂不实施边界。本轮只创建合同/设计/编号/导航与路线簿记，不启动实现或修改AG011合同，不联系其他agent/chat。

基线auto-ui867819bf21266593d2e8f57ae5626de0436083fb；027实际delivery f3dea0d，独立final/冻结/归档/清理均在案，canonical [027 Spec](../specs/auto-graphics/node3d-character-and-runtime-camera.md) SHA95a86660dfa15a09206baac45d8901a9f515fbe871cbdc676a210afed14ff621。025实际delivery e9cc929，后续canonical/锁54899e9，已清理，不再当在途前置。020/024、001=537122b、002=0ee5ad0均实际交付。

相关权威：[020组件](../specs/auto-graphics/stateful-node-components.md)、[017有限G11二维](../specs/auto-graphics/node2d-g11-authoring.md)、旧[运行对象](../../../auto-lang/docs/specs/auto-graphics/runtime-objects.md)、[触发与新局](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[G11内容规则](../../../auto-lang/docs/specs/auto-graphics/gallery-runtime-objects.md)。引用旧Spec知识，不改旧版本/历史收据。

已观察源SHA256：pilot lower_components3d_runtime.rs=da55160cab3da81492a73605a9d278bc4739210b4ea4412309ac7c09808a2f1c，translate.rs=6b668c6566fee257ca24d4f5c6d0d1a19fdb73cbbdca0cdfd0f4a2b7e6ea72c5；driver runtime_objects_v1.at=fe9a8948270d6a8756661830aeb1c6ce7f35c18eb2f3e65d5befaf8595408dc5，runtime_triggers_v1.at=b2b75b168570d8cf86a470b65511d3923a04d0ae0848caa694a2b619fc0b8ce1。

Gallery只读基线2fcbaa72f734ee4df40006cb3577b40334cd3106：checks/g11/gallery_runtime.rs SHAfae22bd4cdc2e3071e7c35004d75b4976b8a5bc158715be7b914993fecc9f90f，3d/main.at SHA83a210e0802255db93fed64e4b966c690153297a4bdffd5b65520b73df52f75d、main-alt.at SHAe5ea5c4cec89554028bfca677b71b06ed246ccfc561c8c3a1adfacfed1f3abc3、scene SHAa397079839ffc5d621be30d0a8d7c2a685d82e94fa2943340ad87242b02776fa。

真实缺口：027无对象/触发/round作者接线，player3d/mob3d拒绝scale/yaw/pitch；原G11三种Mob模板scale .5/.6/.7，编辑角色y=1而Start设y=.5。不能换圆柱/同尺度或把编辑初值改成.5冒充保真。017有硬连移动读一/idle读三、事件tick跳过巡逻位置提交的有限调度，不能原封照搬成通用组件默认业务。

## 5. 详细设计

### 5.1 可执行入口与先证

T-01核027/020/024/001/002的actual source/Spec/独立final/冻结与清理、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467与681文件内容锁、实际语言/依赖锁/toolchain与owner。旧历史对象缺失仅用受检固定副本，不能换当前SDK。唯一树/目标与材料scope开工才创建登记。

T-02最多比较两种复用接法，走完整源→降层/闭包→Triggers3V1→真实Auto/Host/准入/GPU：三种Mob模板、初始Start高度、六实例/六pair、相机运动、三收集胜与alt失败、球切触、三件静止同tick、旧代/显式重绑、新Run，并先证非游戏触发组件/两个同定义实例。记录准确profile/CLI/typed端口/对象和pair分配、每阶段时序/费用/耗时、原25项映射与失败反例。只生成漂亮源码不是go；任何超32或需改旧预算/玩法/几何才成立，暂停依赖任务回规划，不静默调额度。

### 5.2 组件、typed引用与模板

模板/具名视觉资源不可变共享，具名实例/触发绑定映射slot/pair表写identity；作者不填数字。类型区分静态场景row/stable ID、ActorRef3D、Template3D、带Run/slot/generation的InstanceRef3D、PairRef3D和事件游标/token；准确拼写/存储表示T-02冻结，沿现有数值ABI不借机扩大核心语言类型系统。原encoded int身份精确保留，不浮点截断、不以slot低位判等。

固定Node可持自己的实例引用/存活状态和事件handler；despawn对象不隐式销毁Node局部SoA/共享资源或产生exit；同定义多实例状态、receiver、引用代次隔离。对子节点typed端口和祖先msg复用020，不用根全局复制全部状态来伪装组件化。至少一个无收集/计分的传感例与Node/字段改名正控，排除Coin/Goal/Fail/固定state/bag编译器特判。

新模式支持内置Mob/Player原Document已准入的静态template scale/yaw/pitch（具体数值域沿原loader，不新造假上限），原Kind/mesh/材质/GLB/许可不变，动态scale/TRS仍拒绝，旧027拒绝仍保持。不同scale产生不同模板身份但共享不可变模型/上传owner。全GLB模板仍守旧plainGLB与mesh上限，不借内置模板绕开普通GLB域。

Start阶段typed初始放置只在原公共API允许配置期，必须保编辑Document初值与tick0接受位置的区别；不能通过改编辑场景、以model绑定循环写位置或冻结后间接group移动绕过角色/solid所有权。T-02冻结对象表/触发registry冻结、模板refs构建、用户Life.Start/spawn/bind与角色/相机freeze的真实producer-consumer顺序。

### 5.3 触发事件、作者时序、重开

八个显式pair，subject同维静态leaf（可角色），source静态leaf或全代次live instance；shape0闭区间AABB，shape1真3D sphere、显式半径(0,100000] finite，不从模型包围盒猜半径。exact tangency命中、1ULP间隙不命中，corner在AABB内但球外要区分。隐藏仍触发，despawn撤销资格；同slot新代不自动绑定，显式rebind新binding generation并fresh enter。

post-move/derive后每Tick唯一evaluate，N事件由N+1用户回调消费、pair顺序、游标每条一次、0显式absence、未读不排队跨tick；同代getter有效期/二次evaluate与post-evaluate geometry lock沿002。已evaluate后move/spawn/rebind/char move具名拒绝，despawn/visible允许；不改公共服务以美化作者示例。

新typed绑定可提供on_enter/stay/exit映射，但消费/分派/提交的有界策略必须显式可由作者组织，不根据“是否移动”“G11”隐式限一/三或自动暂停输入。T-02冻结一种普通调用/声明端口，使作者能表达旧G11移动一事件、idle最多三事件，事件tick不提交Saw位置但局部巡逻状态照常推进；分派读pair/phase/source每次真收费，不以“绑定”为名免费读/缓存旧事件。根/子Life.Tick→作者指定事件分派→位置提交/重开检查顺序须可重建原源码：终局判断与fresh Enter的先后不能因根Tick总先跑而延迟/提前。第二个合法程序须可用不同有限策略证明非G11专用调度。

胜败/计数/只收一次全部作者业务：Coin自己的model和on决定收集/销毁，关卡接msg决定count/result，危害/目标按作者处理。保原main speed2、hy/hbase2.4、hdir.05；alt speed2.6、hy/hbase.9、hdir.08且越界后方向重置仍按原±.05。原三件x8/10/12、Saw x9、Beacon x16、Void z=-8、ground/模板/scale/材质/隐藏与camera8/45保持；不能为了过预算改变合法轨迹。

round.request沿accepted marker：Running且完整候选接受后才换fresh Run，同源配置重跑Start、清旧实例/事件/输入/代次；Paused/Step不消费、held不连开、tick0请求拒绝，同tick重复幂等。构造/prepare失败保原last-good完整呈现、关闭新VM且不自动重试，Stop恢复原编辑Prepared。此为既有局部运行合同，不新增普遍事务要求。

### 5.4 原额与来源闭包

全Node共享32 credits/callback；原G11移动注释的held1+move26+next1+pair1+despawn1=30不含新phase/source/typed分派；必须完整实测并单列无事件/单收集/目标未齐/胜败/巡逻提交/idle三件/新局等分支。bind1、每event_next/每getter1、每objects调用1、round.request1，evaluate内部0，相机basis读0（既有单价）。typed包装不能额外漏算读取，别给每Node一池。旧例本来存在的作者有限策略可保，不能引擎自动Idle/少计事件/丢帧位移/删输入来凑32。

八slot/八pair、每callback32 lifecycle writes（第33写先错37）、instance generation1..65535/run1..4095不回绕；错误31..38/40..42及原sticky语义保持。保120base+8reserved=128（121base拒绝），512mesh按base＋8×最坏模板潜在mesh预留、live hidden也计入；不是027的128场景行再加八slot。对象非solid、不进入Document/Undo/角色扫掠或相机target；多mesh Mob/GLB按原共享owner暖上传。

原256KiB用户源/96KiB managed/352KiB组合、1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s/8ms pump目标不变；8有状态Node与8运行slot不同单位，T-02证明主组件划分/无model根不误占位，不额外给第九组件预算。GLB/资源8身份32MiB raw/CPU-GPU各64MiB等沿024，各账分别计，稀疏/密集压力沿001/027适用已批准样本，不扩大为任意128密集。

来源固定交付source、SDK681/锁/参数/toolchain、generator/runner实际hash、模块/模板资源/许可与roles闭包互绑。新profile A→B仅闭包冷再生/真实Run、SaveAs关开、坏件拒绝→精确恢复→同入口再Run、20轮newRun/churn/Stop最后owner账平坦。新SD review后merge沉淀；旧exact/027/025/017/020等受影响档冻结字节/错误保留，不混用API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-instances-triggers-and-rounds.md | 027无对象/触发/round作者接线且内置模型模板固定形态→新显式typed有限模板/实例/事件/新局、作者策略/费用/寿命/来源契约；旧档不变 | 支撑通用组件与后续G11/3d固定内容消费，不把能力绿当内容交付 | AC-01..06/08 |

## 6. 测试设计

新case/入口待T-03..06建立执行，非当前已绿。无用户指定实现/内部/最终review人选；开工核owner，同一权威树顺序、独立final不同上下文；internal pass不代final，本合同不授权创建agent。

| ID/AC/owner_stage | 不变量、合法/非法输入、状态 | 实际入口/区分oracle/覆盖边界 |
|---|---|---|
| V-S01 / AC-01/02 / work | use/局部状态/两个同定义组件、非游戏触发正控、改名重排；错维/错kind/越scope同type引用非法 | 新node3d_events_gates→真generator/scene/world/identity/closure与Auto状态，receiver/存活独立，位置诊断无半产物；不靠type mismatch代作用域反例 |
| V-S02 / AC-02/03/04 / work | 原G11主alt/25项及Start高度、三种Mob scale、原32完整分支；33计费/121base拒绝 | 新node3d_events_flow真实Session/Host/Auto/GPU，独立f32/AABB/sphere与mesh/材质/轨迹预期；不能旧源字符串替换仍当新作者链 |
| V-S03 / AC-02/03 / work | exact touch/±ULP/球corner、hidden live、despawn无exit、旧slot代/显式rebind、post-evaluate写 | 真typed对象/pair→公共108/19及原独立几何准入，事件tick/顺序/代次/锁错误13/31..42适用项，wrong frame/event变异真实被拒 |
| V-S04 / AC-03/04 / work | 主例移动读一/idle三与另一作者有限策略、phase/next/source单价、重复读取/超额/早目标 | 生成真实调用/VM计量＋Host事件轨迹、完整分支数值/32pass33fail，未读不跨Tick排队；拒绝hidden自动Idle、根先Tick改时序、免费getter和少提交巡逻的错误变体 |
| V-S05 / AC-03/05 / work | fresh Enter/held/Paused/Step、旧Run引用、新局构造/prepare失败、20轮churn | 原Play真实dispatch/Host/GPU与新Node源，Run/tick0/局部状态/槽代/相机重置、失败last-good/Stop原Document/Undo、资源/VM最后owner；不冒充OS键盘实跑 |
| V-S06 / AC-05/06 / work | A→B仅闭包再生/SaveAs/坏资源/模块/锁/partial收据拒绝→恢复、旧档 | 新verify_plan028与真工具/runner/受影响旧verifier，actual hash/资源许可/闭包/再Run，所有负控实际触发；不等于全AG/跨端/Gallery交付 |

### 原G11/3d二十五项（逐名保留）

`import_three_templates`、`three_distinct_shard_templates`、`start_without_fault`、`tick0_six_pairs_bound`、`tick0_six_instances`、`orbit_camera_from_source`、`walk_collects_three_and_wins`、`three_distinct_ids_each_once`、`windowless_png_three`、`alt_fails_at_hazard`、`failed_state_freezes_motion`、`restart_new_run_tick0`、`restart_rebirths_items`、`orbit_changes_pixels`、`camera_relative_walk_rotates_world_direction`、`camera_relative_walk_reaches_the_void_zone`、`sphere_corner_outside_does_not_trigger`、`sphere_tangent_hits`、`aabb_and_sphere_are_distinguishable`、`same_tick_three_engine_block`、`same_tick_three_counted_once_each`、`old_generation_never_auto_reattaches`、`explicit_rebind_enters_new_generation`、`same_slot_new_generation_identity`、`stale_handle_operation_is_rejected`。

全部绑定新Node源/真实公共链；原其余51项是G11/2d，不把25当全76。新增wrong-kind/early-goal/hidden/phase/两父receiver/round失败与压力补充另列，不抵扣旧25。

拟实施树命令：`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_events_gates -- --test-threads=1`；driver改manifest为experimental/autographics-auto-driver/Cargo.toml、test为node3d_events_flow（沿027实际64MiB测试线程栈，不能靠默认小栈红去减样本）；`python experimental/autographics-node2d-pilot/tools/verify_plan028.py --controls`。新入口T-03..05建立，准确CLI/输出T-02冻结，期望具名正控全部过、负控具名拒绝/恢复真Run，不接受partial。

实际受影响兼容面：027 node3d_runtime_gates/flow与verify_plan027，公共对象/触发API与runtime_round_host相关命名case、旧017/020/025共享AST/translate反例和verifier。T-02冻结受影响文件/case并集，旧单元字节未变可有据复用原证据，不为merge跑全AG/语言根crate全量。真实GPU/窗口单实例，OS键鼠/焦点自动化继续暂停，必需缺证具名gap；准备/冷编译/执行/清理/总时间与实际数分列。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通use/typed模板引用/局部model/msg/on，双实例状态/receiver/引用隔离、无手写ID；非游戏例/改名证明无业务特判 | V-S01 |
| AC-02 | work | 三种原Mob模板/Start高度/六实例六pair及原G11主alt25项全在新链保持，独立几何/数值/GPU正确 | V-S02/03 |
| AC-03 | work | 对象代次/隐藏/重绑/phase游标/下一Tick时序/geometry lock、accepted round失败/新Run/Stop原语义保持，反例实际拒绝 | V-S03..05 |
| AC-04 | work | 完整分支所有typed/phase成本原32内，33故障；8slot/8pair/120base+8/512mesh/8有状态组件/VM源码额与压力不降不抬 | V-S02/04/05 |
| AC-05 | work | 新旧显式profile隔离、旧027/受影响旧档冻结输出/负控不变，无Rust业务/免费getter/隐式停输入，owner与失败候选不泄漏 | V-S04..06 |
| AC-06 | work | actual source-build/SDK681/锁/工具/资源许可闭包、A→B/保存重开/冷再生/坏件恢复/20轮齐，SD候选/缺证边界清楚 | V-S05/06 |
| AC-07 | review | 同树独立final逐AC/实际diff/SD/告警/遗漏延期workaround审查，finding回work复核pass，绑定提交/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际合主，SD-01 canonical核字节/来源、归档/导航，下游钉actual delivery | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组，全实际输出对账/摘要折Plan/删scope构建物与具名hold/最终hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；020/024/027/001/002已交付 | 核§4来源/SDK681/依赖/owner，唯一树/独占scope登记，不追tip | AC-01/05/06 | [ ] |
| T-02 | work | T-01 | 最多两接法全链先证与非游戏组件，冻结typed API/CLI/生命周期/25oracle/Start/模板/32计量；缺口回规划 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | dialect/pilot shared typed模板/实例/事件/round与局部receiver降层，新gate/正负夹具，旧027字节/反例不变 | AC-01/02/05 | [ ] |
| T-04 | work | T-03 | 真Triggers3V1/Host/Play适配、node3d_events_flow原25oracle＋V-S，不写Gallery | AC-02/03/05 | [ ] |
| T-05 | work | T-04 | verify_plan028、完整分支原额/压力/typed/phase/round失败与全部负控、独立数值/GPU | AC-01..05 | [ ] |
| T-06 | work | T-05 | source-build/A→B/SaveAs/恢复/20轮、受影响兼容并集，SD-01候选及摘要移交 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同权威树独立final/精确revision绑定，finding回work再复核；pass后merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/输入/定向并集门或有据复用，来源核后实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01 canonical/来源核、归档/导航，不写派生JSON，G11内容仍后继未交付 | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清树/分支/空组，全实际输出对账/删scope构建物/登记与最终收据 | AC-09 | [ ] |

## 9. 复审记录

2026-11-02 / stage: new / plan_id: auto-ui/PLAN-028 / plan_revision: 1 / outcome: pass（合同可审阅移交，非可行性go/独立final/开工授权） / next: 用户启动后work T-01/T-02。0/10；已核027实际交付及旧001/002、原G11三维25项和模板缩放/初始化/调度缺口。新case/探针全部未实施，未建树/源码/资产/提案Spec、未构建/跑GPU/OS自动化。未来review/merge任务保持未勾选，不把收尾作为work移交前置。

2026-11-02规划静态预检：两合同0..10节、各10任务/9AC/6补充验证/1SD、work6/review1/merge3归属与drafting0/10通过；51个新增/修改本地链接可解析，依赖来源存在且无环。G07/G08三维24个旧oracle、G11三维25个旧oracle逐字各保留一次，source/Spec原始SHA逐项重核匹配；本地取号唯一，028/012独占分配，随后Reader并行另占029，号源不回退。实际metadata活源清单11/22（原21迁10＋新增HUD1），`python tools/gallery.py validate --all` PASS 12逻辑项/22入口、许可一致。AG011本轮已核见唯一ag-011树和固定f3dea0d消费树；其任务更新由owner负责，本轮不替其翻状态/改合同。以上是规划/目录核验，不是新能力可行性go、运行验收或独立final；未建实施树/探针/产品源码/资产/提案Spec，未构建或运行GPU/OS自动化。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 新profile与typed端口/有限事件分派/Start注入顺序 | work T-02最多两接法完整先证，冻结producer-consumer/准确拼写；不靠业务名/自动Idle |
| D-02 | phase/source/局部事件及巡逻提交的最坏32成本 | work T-02/05逐分支实测，超额回规划，不免费getter/少消费/改轨迹 |
| D-03 | 原Mob模板静态transform/8状态组件/八运行slot不同包络 | work T-02实证三scale与Start高度、identity/mesh账，旧027边界不拓宽 |
| D-04 | AG011/012的固定读027与共享接缝、真实OS证据暂停 | work T-01核owner/短窗，缺硬件证据具名列出；不接管内容、不自动开OS测试 |

材料唯一D:/autostack/plan-artifacts/plan-028/auto-ui/，开工登记owner/用途/在途；临时探针/日志/截图/包/消费者/外置target只归scope，永久产品/必要回归夹具归仓，禁止attachments/第二证据目录。merge摘要折Plan后全输出对账并删自有scope/构建中间物；仅具名未解决问题按owner/理由/关闭条件持有。禁止worktree链接，清树先fresh D:/autostack/wt-guard.sh；旧specs.json只读。
