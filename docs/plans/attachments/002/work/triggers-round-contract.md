# PLAN-002 triggers+rounds 冻结合同（T-02 产出）

计划：`auto-ui/PLAN-002`（r4）。本文档冻结有限触发事件与受检新局的公开合同：profile、
公开签名、8对资源表、事件字段/顺序/读取/重绑、几何边界、restart状态图/失败路径、
消费者与计价。冻结基线：worktree `D:/autostack/.wt/ui-002/auto-ui`，分支
`codex/plan-002-dev`，base `78ee3b7`（master：001 交付 `537122b` + 两笔 docs-only）。
前置核实（T-01）：001 实际交付 `537122b`（身份清单 `research/plan001-identity.json`），
runtime objects 单元/准入/渲染适配器按其 r4 交付原样复用，本计划零字节修改既有
`api/*.at` 与 `src/authoring/runtime_objects.rs`；SDK 闭包 `.sdk-source`（source-lock
冻结）与两锁不变。

## 1. Profile 与装配

| 候选 | 版本串 | API number | wire profile | 组合 |
|---|---|---|---|---|
| triggers2d | `autographics.triggers2d/1` | 107 | 18 | motion(v5)+character2d+camera2d+objects(dim2)+triggers(dim2) |
| triggers3d | `autographics.triggers3d/1` | 108 | 19 | motion(v5)+character3d+camera3d+objects(dim3)+triggers(dim3) |

- 新单一可信单元 `api/runtime_triggers_v1.at`，两维共用；装配期 `var _ag_trg_dim=0`
  替换为 2/3（同 objects）。triggers profile 隐含 objects profile：`Api::objects()` 对
  triggers 也返回 Some，槽容量/网格包络/对象块全部照旧生效。
- 入口链（唯一新增改写，全部 require 恰好一次）：
  - start：`_ag_obj_freeze(); _ag_trg_freeze(); app_start();`（注册表先于绑定存在）。
  - tick 前置：`_ag_char*_tick_begin(); _ag_obj_tick_begin(); _ag_trg_tick_begin(); app_update(dt)`。
  - tick 后置（derive 之后、worlds 已含当步位移）：`…; _ag_dirty=0; _ag_trg_evaluate(); toggled=0}`，
    唯一标记 `toggled=0}`。`_ag_start` 不注入 evaluate（tick 0 无事件）。
- 评估后锁（"评估后无新的几何状态混入本轮"）：装配期向 objects/character 单元文本
  **派生**注入 `if _ag_obj_locked==1 {_ag_fail(40,…)}`（仅 `ag_object_move`、
  `ag_object_spawn`、`ag_char_move`/`ag_char3_move` 三处；despawn/visible 不锁——
  "evaluate 后允许按结果销毁物品"）。`var _ag_obj_locked=0` 随 `var _ag_obj_dim={dim}`
  替换一并注入；`_ag_trg_tick_begin()` 每tick清零。磁盘共享文件零修改，001 身份不受影响。
- 既有约束沿用：单元内禁止字符串字面量与无空格减号；`-1` 一律写 `0 - 1`。

## 2. 公开函数（全部先查 sticky error；`ag_` 前缀）

| 函数 | 返回 | 语义 |
|---|---|---|
| `ag_trigger_bind(pair int, subject_id int, source_kind int, source_ref int, shape int, radius float) int` | 1/0 | 注册/重绑。pair∈[0,8)；subject_id=同维非组叶稳定文档id（含角色节点）；source_kind 0=静态叶(source_ref=稳定id) 1=运行实例(source_ref=完整对象句柄，绑定其当次generation)；shape 0=AABB闭区间 1=2D圆/3D球；shape0 要求 radius==0.0，shape1 要求 radius∈(0,100000] 有限。重绑新代：bind_gen+1，prev 态清零，旧事件不跨绑定代。计 1 op+1 credit |
| `ag_event_next() int` | 1/0 | 出队当前冻结快照的下一条事件（pair 升序）；无事件返回显式 0（absence，不设 sticky）。每次 1 credit |
| `ag_event_pair() int` | pair | 当前事件只读字段 |
| `ag_event_phase() int` | 1/2/3 | 1=enter 2=stay 3=exit |
| `ag_event_generation() int` | gen | 事件绑定代（绑定代，非实例代） |
| `ag_event_source_kind() int` | 0/1 | 静态叶/实例 |
| `ag_event_source() int` | ref | 绑定时原值：稳定id 或 完整对象句柄（可直接 despawn） |
| `ag_event_geometry() int` | 0/1 | AABB / 圆球 |
| `ag_round_request() int` | 1/0 | 新局请求标记；每tick至多1个待处理，同tick多次幂等合并；tick 0（app_start）具名拒绝。1 credit |

- getter 语义：仅当前快照有效；`next` 成功前/耗尽后一律返 0，不设 sticky；每事件只出队
  一次，第二次 evaluate 重建快照并重置游标（旧游标作废，不重发）。
- 内部：`_ag_trg_map`/`_ag_trg_freeze`/`_ag_trg_tick_begin`/`_ag_trg_evaluate` +
  各发射器，host 装配注入，非公开。

## 3. 资源表与传输块（rounds block，位于 objects block 之后）

- 最多 8 对（`PAIRS=8`），稳定 pair 序号=注册表顺序；每对 = 一个同维 subject 叶 +
  一个触发源（静态叶或 001 活实例）。实例身份完整绑定 Run/slot/generation：绑定时刻
  校验 `slot 活 && generation 匹配句柄`；实例死亡 → 该 pair 本tick失效未绑定态
  （绑定行发 canonical 0 行），同槽新代不自动纳入，显式重绑后可重新 enter。
- 块布局 `ROUND_BLOCK_LEN = 8 + 8*9 + 8*8 = 144`：
  - header 8 int：`status, run, tick, event_count, restart_flag, restart_tick, evaluate_seq, 0`。
    status=1 当冻结且无 sticky；evaluate_seq：tick0=0（仅注册表），tick N≥1 必须等于 N
    （每次 update 恰好一次 evaluate；第二次 evaluate → 40）。
  - bindings 8 行 × (8 int + 1 float)：`bound, bind_gen, subject_index, src_kind, src_ref,
    src_tmpl, src_gen, shape` + `radius`。src_kind0：src_ref=叶 run index，src_tmpl=同值，
    src_gen=0；src_kind1：src_ref=slot，src_tmpl=模板 run index，src_gen=实例 generation。
    未绑定/失效行全 canonical 0。bind_gen∈[1,65535]，host 侧单调（同代内配置冻结，
    与 objects 模板冻结同规则）；耗尽 → 42，不回绕。
  - events 8 行 × 8 int（pair 升序，仅活跃行）：`pair, phase, bind_gen, src_kind,
    src_ref, src_tmpl, src_gen, geometry`。每对每 tick ≤1 条，总数 ≤8。
- Frame 携带 `rounds: Option<RuntimeRoundFrame>`（dim/run/tick/seq/restart 标记/事件表），
  与对象、角色、相机同一候选同一事务 prepare/accept/publish。

## 4. 几何与时序边界

- AABB（shape 0）：沿用旧闭区间语义——逐活跃轴 `hi_a < lo_b || hi_b < lo_a` 才分离，
  相切=命中；±1ULP 分离必有反例。2D 只用 x,y 两轴。
- 圆/球（shape 1）：中心向 subject 实际 AABB 逐轴 clamp 得最近点，平方距离
  `<= radius*radius` 命中（相切命中；角点在包围盒内、圆/球外不命中）。2D 圆心 z 恒 0、
  只算 xy。半径只来自显式 bind 配置，**绝不**隐式取 GLB 包围盒。
- 位置来源：subject/静态源 = `_ag_worlds[node*3+axis]`（当步 derive 后值）+
  `_ag_spatial_boxes[node*7+1..]`；实例源 = 模板节点 descriptor 行 + `_ag_obj_x/y/z[slot]`。
- 时序：evaluate 每次 update 恰好一次、读当步最终位置；evaluate 后 move/spawn/bind/
  char_move → 40（despawn/visible 允许）；第二次 evaluate → 40。evaluate 后结果冻结。
- app_start 只建注册表：tick0 event_count 恒 0、restart_flag 恒 0（host 准入强制）；
  首次 update 与 false 前态比较（初始重叠在首次 update enter）。
- 相位机：`false→true=enter(1)，true→true=stay(2)，true→false=exit(3)，false→false=无`。
  prev 态 host/unit 两侧一致维护；重绑/despawn 失效即视 false。
- 隐藏不取消触发（显隐与生命周期分离）；触发非 solid，永不进入碰撞世界。

## 5. Host 准入（runtime_round::admit_round_block，拒绝具名）

顺序/唯一/容量/活性/几何来源/合法相位转移全部核对，**不是只数 count**：
1. header：status/run/tick/seq/canonical 尾域；tick0 强制 count=0、restart=0。
2. bindings：域检查（索引、维度、非组、shape/radius 域、3D 无资源叶 bind 时 9）；
   与上一 admitted 帧比 bind_gen 单调、同代配置冻结；实例源与当tick候选 instances 帧
   的 slot/template/generation 精确一致，死实例行必须 canonical。
3. 几何重算：host 用候选帧 node 世界平移 + descriptor 行 + instances 帧独立重算每对
   overlap，与相位机期望比对——事件表必须与期望**逐行相等**（count、pair 升序、phase、
   身份镜像、geometry），多报/漏报/假 enter/旧 tick/重复 pair 全部拒绝。
4. restart：flag∈{0,1}；flag1 ⇔ restart_tick==tick。

## 6. 新局（restart）状态图与失败路径

- 用户 Auto 拥有 Playing/Won/Failed；样板只在 Won/Failed 读取一次 Enter 边沿
  （`ag_action_pressed`，2D action3 / 3D action5）后 `ag_round_request()`。Host 不识别
  胜负语义。
- 请求 = 本 callback 有界输出标记：≤1 待处理，同 tick 幂等合并，不建队列；携带当前
  Run/tick 于块内；**当前完整候选 accept 之后**才处理。
- Host（pump_play）：accept 后若 `frame.rounds.restart_requested && state==Running`：
  - 停旧局：`old.stop()`（关 VM/清 held/pending/camera capture，owner 释放，owner_probe
    断言）；**不**恢复编辑 Prepared（`play_restore` 原样保留）、不重发编辑帧。
  - 建新局：同一已接受 SourceSnapshot 内存字节 + 原 actors + 由未变编辑文档重建
    descriptor → `Play::new_user_spatial`：新 run_id（原子自增，耗尽具名拒绝）、新 VM
    （app_start 重执行、用户 globals 重置）、对象表/事件前态/相机初值全新；期间保持
    旧 last-good 显示，直到新 tick0 完整 prepare+accept 才替换呈现。
  - 失败路径：新 Run 构造或 tick0 准备失败 → 具名 Fault（前缀 `round restart:`）、关闭
    新 VM、保留旧最后完整呈现与旧 last-good GPU 像素；不假装完成新局、不自动重试；
    Stop 仍恢复原编辑 Prepared。
- 迟到隔离：旧 Run 句柄在新 Run 是外来 Run（负 rel）→ 33；旧事件/旧 input/旧 camera
  随旧 VM 销毁，绝不进入新 Run。held Enter 不连续 restart（repeat 不产生边沿、新 VM
  边沿缓冲为空，恢复需新按下）；Starting/Paused 不处理新请求（仅 Running accept 后处理；
  step tick accept 后 state=Paused → 不处理）；Stop 取消尚未完成的新局。
- 中立 Step 语义不变：Pause 不检测不推进，Step 只中立输入执行一个逻辑步，不合成按键。

## 7. 消费者与计价

- 共享预算不变：32 credits/tick 时钟预算（`_ag_cast_charge`）、1000 steps/slice、
  100000 steps/callback、120 resumes/250ms、30s 初始化/2s 整tick/8ms host slice、
  128 节点/512 mesh 实例包络。源限（r5 显式上调，反例见 Plan §9）：用户源 256KiB 不变；
  受管桥接 64KiB→**96KiB**，组合 320KiB→**352KiB**——triggers 组合实测 72141/74050B
  在 64KiB 下不可行（Objects2 61950B 对照），功能分母与运行时预算零变化。
- 本代计价（实测冻结）：bind=1 op（每 callback ≤32，超 37）+1 credit；event_next=1
  credit；getter 各 1 credit；round_request=1 credit；evaluate=0 credit（引擎内部、≤8
  对几何，受 2s/步数预算约束）。既有成本实测：ag_char_move≈26（1+24 sweep+1 grounded
  扫描）、对象操作各 1——一次带移动的 callback 至多配一事件的有界读取。整组合 probe
  （角色移动+相机+8对象+8对触发+有界读取+销毁+rebind+restart 请求）跨 tick 分摊后全部
  在 32 credits 内成立（runtime_triggers_api::whole_path_combination 实测），不足回 new。
- 读取时序实测冻结：用户游标消费**上一次** evaluate 的冻结快照（迟一拍）；admitted 块
  发射器恒显示当前拍。宿主 credit tick 可在同一帧内追加派发，故轮询必须逐帧进行。
- sweep 世界实测：静态模板叶（含隐藏）在 sweep 世界内，角色在闭区间接触/净距 0.002 内
  无法推进——引擎夹具的全部交互几何（币/危险/终点）必须是运行实例，静态模板放图外。
- 新错误码（sticky first error）：40 TriggerLocked（evaluate 后写/二次 evaluate）
  · 41 TriggerBinding（pair 域/subject 源域/stale 句柄/shape radius 域/tick0 restart）
  · 42 绑定代耗尽。既有 31–38（objects）与 1–15（Rust 侧命名）原样保留。
- 下游交付接口（ag-examples/001 冻结消费用）：本计划最终合入 commit、profile
  `autographics.triggers2d|3d/1`（107/108，wire 18/19）、上表公开签名与限额、
  samples/runtime-triggers/{two,three} 引擎夹具、verify_plan002/check_runtime_triggers
  门与 own-SDK 离线消费路径。G11 不得以 review pass 或候选包为实施依据。

## 8. 冻结的测试入口（T-03/04/05 建立）

```
cargo check --locked
cargo fmt -- --check
cargo test --locked --test runtime_triggers_api --test runtime_round_host --test runtime_trigger_samples -- --test-threads=1
```
另：受影响 001 旧门 `--test runtime_objects_api --test runtime_objects_host --test
runtime_objects_samples -- --test-threads=1`（组合改动波及装配，必须回放）；
本代 checker `tools/check_runtime_triggers.py`（含 `--controls`）与
`tools/verify_plan002.py --controls`；own-SDK 离线消费
`tools/package_consumer.py <dir>` + 消费树 `cargo build --locked` + 夹具运行。
GPU 机器单实例；原始证据本地 `D:/autostack/plan-artifacts/auto-ui/plan-002/`。
