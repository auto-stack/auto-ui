# PLAN-017 g11-2d 作者合同（T-02 冻结）

status: frozen（T-02 go；完整 main/alt 整链实测通过）
plan: PLAN-017 r1 · auto-ui · tree `D:/autostack/.wt/ui-017/auto-ui` · branch `codex/plan-017-dev`
work base_commit: 7a53d13d5e6da6eedaa9b1e9de0a3fdcaca089c4（master tip，含 PLAN-016 交付 932bd5f）
code_commit: T-02 提交（见 §9 收据）
SDK pin: `7b987ca8`（driver `.sdk-source` 681 files，tools/prepare_sdk.py --from-snapshot PASS）
只读依赖: auto-lang `98270a4f5`（delivered master，plan-890 closed；auto-lang-core 现以 path 引入 auto-web-contract → ui-017 组含 auto-web `f29c7d1` 兄弟树）；auto-down `c3464d0`（只读兄弟树）。两图规则不变：生成器（tip 语言 crate）与 driver（pinned SDK）绝不同图。
实测证据: driver `tests/node2d_g11_flow` 三探针（walk/win/restart、alt fail/freeze/restart、CPU 恰 32 校准）全绿；legacy 电池全绿（dialect 6、compiler 6、pilot 9、verify_plan016 --controls PASS、node2d_flow 6/6）。

## 1. 接法比较（T-02 上限两种，实测裁定）

**(a) 016 式"处理器内嵌图形调用"（Hazard Life.Tick 内 refs.body.move_to 每 tick 写位置）——排除。**
预算反例：移动收集 tick = held2 + sweep26 + next1 + pair1 + phase1 + despawn1 + 危险 move1 = **33 > 32**，第 33 个计费调用为 sticky fault 10（实测技术，flow_budget 同款）。且与原 001 "有事件 tick 不写危险图形位置"的兼容要求冲突（计划 §5.3）。

**(b) 声明位置 + 编排图形同步（事件 tick 跳过）——选中，已实测。**
动态位置（声明引用 model 可变字段）的触发实例由编排段在**首条 event_next==0 的 tick** 执行一次 `ag_object_move`（位置=声明表达式按实例当前状态求值）；有事件 tick 跳过写、巡逻状态照常推进 —— 原 001 等价策略，逐字记录于本合同 §4。静态位置（仅字面量/参数）只在 spawn 时落位。
同步时点在实例 Life.Tick 之后（post-advance）：与原 001 相比，危险评估位置每 tick 领先一个巡逻增量（0.05），对原 51 项语义的可达带核查全部无影响（main 危险最低触达 1.75 > 角色顶 1.0；alt 死亡 x 带 [8.5,8.6] ⊂ (7,9.6)）；该 0.05 领先为本合同的**声明调度差量**，被下列 oracle 观测位置覆盖：G11-2D-13/22/23/42/43（映射见 §6）。

## 2. CLI 与输出（冻结）

- legacy（默认，016 字节）：`cargo run --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --bin node2d-lower -- --entry <dir>/main.at --out <dir>`
- g11-2d（017）：`cargo run --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --bin node2d-lower -- --profile g11-2d --entry <dir>/main.at --out <dir>`
- g11-2d 输出：`world.at`（生成用户源）、`scene.agscene`（autographics.scene/6，behavior 绑定 world sha/bytes）、`identity.json`（generator/api/sha/bytes、逐模块 sha、closure 清单、roles：character/grounds/pairs→instance+template）、`closure/`（输入模块闭包副本，A→B 冷消费用）。
- 正常运行（无生成器）：`autographics-authoring --open <scene> --behavior <world.at> --behavior-api triggers2d` 进入真实 Run；driver 侧消费为 `Storage::load_project(scene) → Host::with_project → start_selected_play`（实测路径）。

## 3. 研究端口与文法面（冻结，全部类型驱动、无名字特判）

| 端口/形态 | 识别 | 生成 |
|---|---|---|
| `character2d <name> { position, mesh, material }` | 新研究元素（词法零改动） | 场景 row id=1（在位=声明位置）+ `ag_char_bind(1)`；宿主定义内 `refs.<elem>.move_and_slide((dx,dy))` → 置 `_n2d_moved=1` + `ag_char_move(dx,dy,0.002)` |
| `solid2d`（仅根 view） | 地面端口 | 场景 row id=2..（在位）+ `ag_char_solid(<id>)` |
| `trigger2d` | shape 缺省 AABB(0,0.0)；`shape: circle(radius: r)` → (1,r) | 场景模板 row（停靠 (-30,2)、invisible）+ spawn + `ag_trigger_bind(pair,1,1,handle,shape,radius)` |
| `camera2d`（≤1，仅根） | bounds 4 元 + view_size 2 元 + target | `ag_camera_follow_2d(1, minx,miny,maxx,maxy,vw,vh)` |
| 参数 float/int | Number | SoA 数组（字面量绑定）；表达式绑定在 owner 上下文逐用点内联（无 SoA 无 getter） |
| 参数 `msg` | MsgEvent 事件端口 | emit → 绑定表投递；载荷常量列 `<prefix>_bind_<EVT>_<n>=[..]`（枚举内联判整值）；无载荷根父直调 |
| 参数 `ActorRef2D` | actor 端口，绑定 `refs.<inst>.actor` | 角色宿主 → `ag_actor(2)`（零计费）；触发实例 → 该实例 handle 数组 |
| 参数 `Mesh2D`/`Material2D` | 资源 symbol 端口，绑定 `resources.<name>` | 模板行 kind/scale/rgba；实例 props 可覆盖定义元素缺省 |
| `input.axis(Left, Right)` | g11-2d 差值轴 | `(ag_action_held(1) - ag_action_held(0))`（held 计费 2；Left=-1/Right=+1/都无与同时=0）。legacy 冻结保持 `ag_action_held(1)` |
| `input.pressed(Enter)` | 消耗性边沿 | `ag_action_pressed(3)`（计费 1；动作表 0=Left/1=Right/2=Jump/3=Enter，与宿主 harness 一致） |
| `round.request()` | 重开服务 | `ag_round_request()`（计费 1，accepted 候选由宿主消费） |
| trigger `on_enter/on_stay/on_exit: .Evt` | phase 绑定 | 路由读 `ag_event_phase()`：1=enter/2=stay/3=exit → 对应事件处理器；未绑定 phase 消费不投递 |
| 模板 id | 声明序 | 角色=1、根 solid=2..、触发实例模板按根 view 声明序（G11 夹具=原 001 的 3..8） |
| pair/slot | 声明序 | 触发实例发射顺序 0..7（G11 夹具 coins=0/1/2、hazard=3、goal=4、void=5 = 原 001 pair 语义位） |

诊断（执行前带 file/line/构造定位拒绝）：缺/错导出模块、未知实例属性、错端口类别值（如事件端口绑 actor）、事件未声明/无处理器、phase 绑定带载荷、shape 非法、role 元素越界（solid/camera 出根 view）、character2d 非 1 个、模板几何缺失、绑定事件混载/混合事件名、condition 不可译。

## 4. 调度与预算（冻结，全部实测）

生成 `app_update` 固定顺序：根 Life.Tick（重开检查读 tick 起点状态）→ 实例 Life.Tick（声明序；Hazard 仅推进巡逻状态）→ 事件段 → 动态位置同步（仅 next==0）→ 条件差量（真 spawn/despawn+重绑）。
事件段：`n=ag_event_next()`；n==1 → next 的 pair+phase 路由；`_n2d_moved==0` 且 n==1 → 再读最多 2 条（静止 3 条封顶）；moved==1 → 恰 1 条。`_n2d_moved` 由 move_and_slide 包装前置置位、每 tick 开头清零。
重开：根 Tick 在 tick 起点状态 ≠Playing 时读 `ag_action_pressed(3)==1` → `ag_round_request()`。**声明差量**：与原 001 相比，"胜/败同 tick 的 Enter 边沿"不被消费（原 001 在事件处理后读边沿）；下一新鲜边沿照常重开。51 项中无同 tick 胜+Enter 用例；`held_enter_no_double_restart`/`won_enter_opens_new_round` 语义不变（实测）。

| 分支 | 账（计费） | 实测 |
|---|---|---|
| 初始化 app_start | init 预算（非 32 池） | 6 spawn+6 bind+char bind+solid+camera 无故障 |
| 普通移动 tick | held2+sweep26+next1+sync1=30 | 无故障 |
| 普通静止 tick | held2+next1+sync1=4 | 无故障 |
| **移动收集 tick** | held2+sweep26+next1+pair1+phase1+despawn1=**32 恰满** | 第 33 计费调用=fault 10（g11_cpu_budget 实测） |
| 静止三收集 tick | held2+3×(next+pair+phase)+3×despawn=14 | 无故障（V-03 三收集用例覆盖） |
| 胜 tick（移动） | 31（重开检查读 tick 起点状态，无 pressed） | 无故障 |
| 终局后 tick | next1+pair1+phase1(goal stay)+pressed1=4 | 无故障 |
| 重开 tick | next1+pair1+phase1+pressed1+request1=5 | 无故障 |
| 危险/越界失败 tick（移动） | 31 | 无故障 |

VM/生命周期全部复用未改 SDK：1000steps/2ms 分片、100000steps/120resumes/250ms、30s init/2s tick、8 slots/8 pairs、代次 handle（旧代跨 Run fault 33/32）、Pause/Step/Resume/Stop 语义、8ms pump 目标独立报告。用户源闭包 ≤256KiB（G11 world 9350B）。

## 5. 夹具与生成物（冻结）

`fixtures/g11-2d/{main,main-alt,resources,types,player,coin,hazard,goal,void_zone,patrol}.at` + 变体根（triple/stay/bag2/geometry/generation，均真实作者源变体——非生成源文本替换）。几何/参数/颜色=原 001 基线逐值；角色声明 spawn=(2,0.5)（原 001 场景 (2,1)+app_start 写 y=0.5 的两步合并为声明式单点；x=2 与行走线 y=0.5 可观测面不变，51 项无 y=1 观测点）。生成物 `generated-g11/<world>/`（world.at/scene.agscene/identity.json/closure/）为冻结数据，再生必须逐字节等价。

## 6. 原 51 项 → 新链关闭映射（owner: driver tests/node2d_g11_flow + pilot gates + verify_plan017）

| 原 oracle | 新链关闭（观测位置等价适配：pair=0/1/2/3/4/5 同语义位；模板 id=3..8 同分配规则） |
|---|---|
| 01 import_two_templates / 02 three_distinct_item_templates | 场景文档 8 行 + 物品三异色/异形断言（冻结 scene.agscene） |
| 03 start_without_fault / 04 six_pairs_bound / 05 six_instances / 06 template_ids 3,4,5 / 07 first_generation | g11_walk init 段（实测） |
| 08 walk_collects_three_and_wins / 09 three_items_destroyed / 10 three_distinct_ids_each_once / 11 collections_are_ordered_ticks / 12 destroyed_never_stay_or_exit / 13 hazard_never_touched / 14 win_alone_restarts_nothing / 15 no_fault_after_win | g11_walk（实测） |
| 16 windowless_png_two | GPU 用例（T-05）：won 后 capture，尺寸/字节/命令数 |
| 17 won_state_freezes_motion / 18 won_enter_opens_new_round | g11_walk（实测） |
| 19 goal_entered_without_three / 20 goal_requires_three / 21 bag2_enter_no_round | bag2 变体世界（源级：rose 缺席→旗标 1→goal 永不赢；enter+stays>0；Enter 不换局） |
| 22 alt_fails_at_hazard / 23 alt_trajectory_differs / 24 failed_freezes | g11_alt（实测；hazard 进触 x 带 (7,9.6)） |
| 25 restart_new_run_tick0 / 26 restart_rebirths / 27 restart_restores_initial / 28 restart_admits_empty_tick0 / 29 old_run_not_replayed / 30 held_no_double_restart | g11_alt（实测） |
| 31 void_zone_fails_oob / 32 oob_freezes | 主世界左行用例（T-04/05） |
| 33 paused_no_motion / 34 single_step_one_tick / 35 stop_restores_document / 36 single_step_geometry_result | triple 世界 harness 用例（T-05） |
| 37 same_tick_three_engine_block / 38 same_tick_three_counted_once_each | triple 变体世界（源级：三币同位 (2,0.5)）：单 Step evaluate 发 3 enter、下一静止 tick 3 读 3 销毁 |
| 39 stay_not_recounted_one_enter / 40 stay_cannot_be_counted_as_enter | stay 变体世界（源级：amber 永不清旗标→不销毁；goal 需三旗清→不赢）：pair0 恰 1 enter + ≥5 stay、活体保持、goal 进入不重开 |
| 41 geometry_variant_admits / 42 circle_corner_silent / 43 circle_tangent_hits / 44 aabb_circle_distinguishable | geometry 变体世界（源级：AABB@角(2.9,1.4)、circle@角、circle@切(2.9,1.0)，静态 dir0=0）：三 pair 进/不进/进 |
| 45 hidden_still_triggers | engine-fact 原始 Auto 探针（spawn+ag_object_visible(0) 后 enter 照发、slot visible=false）——SDK 事实，非作者能力 |
| 46 old_gen_never_auto_reattaches / 47 explicit_rebind_new_gen / 48 same_slot_new_gen_identity | generation 变体世界（源级：旗标 t==3 清/t==4 置 → despawn/重生同 slot 重绑；enter ticks [1,≥4]、gen2 观测） |
| 49 stale_handle_rejected | engine-fact 原始 Auto 探针（旧代 handle 操作=fault 33） |
| 50 same_tick_both_moved_final_positions / 51 previous_position_not_used | bothmove/away 变体世界（源级：对走组件/每 tick 远移组件）：enter tick==首重叠 tick；away 永不 enter |

变体世界全部经同一生成器 lowering（作者源级受检变体，无 app_start 文本替换）；engine-fact 探针（45/49）为 pinned SDK 事实的等价测试输入，不计作者能力。

## 7. 阶段门

- T-02 go：本合同 + 三探针（已满足）。
- T-03..T-05：按 §3/§4/§6 实施；改 §3/§4 规则或预算先修订本合同再实施。
- verify_plan017 --controls：身份绑定、再生字节等价（全部 generated-g11）、夹具变异敏感、缺模块/坏端口诊断、篡改检出、closure 清单一致。
- 消费 A→B（T-06）：B = 独立导出目录 + closure/ + 冷 .sdk-source（prepare_sdk --from-snapshot）+ 独立 target，`--locked --offline` 同输入同结果；B 不借生产者源树/target/网络。
