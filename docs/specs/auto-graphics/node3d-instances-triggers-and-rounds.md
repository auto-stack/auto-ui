# AutoGraphics 三维实例/触发事件/新局作者接线（auto-ui PLAN-028 SD-01 候选）

Status: candidate（auto-ui PLAN-028 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node3d-instances-triggers-and-rounds.md 的
SD-01 候选文本，merge 阶段按收据落位。只描述实际交付并验证的能力，
不声称任何未实现的语言/运行时特性。）

## 0. 范围

`node2d-lower --profile components-3d-events` 在 024/027 的共享三维组件
管线上新增四类声明接线，把**已交付 objects3d/1、triggers3d/1 与
accepted 新局**接到普通 `use`、`node`/`model`/`msg`/`view-3d`/`on` 和
typed 名字引用上；作者不写数字 slot/pair/scene ID：

- `object3d { }`（node 级标记）：本定义为 spawn 模板。模板定义恰一
  `view-3d` 叶（v1 kinds：cylinder3d/player3d/mob3d），不得携带
  params/model/msg/on/character3d/camera3d/solid，不得同时在任何视图
  实例化；叶的静态 scale/yaw/pitch 即**模板身份**（内置视觉在此档
  开放静态 scale/yaw/pitch——旧 024/027 档保持具名拒绝），行值全部
  为已求值常量。模板行按（模块表序 × 文件内声明序）追加在 root/实例
  行之后 park 在场景中（可见性/位置由作者声明携带，降层不造魔法）；
  模板行文档 id 即 spawn 的模板号。
- `objects3d { a, b, … }`（仅 root）：具名对象槽池。名字按声明序映射
  运行槽 0..8（>8 具名拒绝）；同名重复具名拒绝；非 root 携带具名
  拒绝。同名槽重生 = 同槽新代（rebirth 通道）。
- `triggers { p, q, … }`（仅 root）：具名触发 pair 池。名字按声明序
  映射 pair 0..8（>8/重复/非 root 具名拒绝）。
- 作者调用端口（全部 1:1 映射受信 units，零隐藏读取、零新单价）：
  - `spawn("<slot>", "<Template>", x, y, z)` →
    `ag_object_spawn(<slot>, <模板行 id>, x, y, z)`；未注册 slot/
    模板名具名拒绝。
  - `bind("<pair>", refs.<elem>, aabb, 0.0, <src>)` /
    `bind("<pair>", refs.<elem>, sphere, r, <src>)` →
    `ag_trigger_bind(<pair>, <主题文档 id>, <source_kind>, <src>,
    <shape>, r)`。主题 `refs.<elem>` 必须解析为本定义的常量行
    （root 视图元素或本实例元素；运行期值/他定义元素具名拒绝），
    生成侧经 `var <prefix>_<elem>_subject_ids=[…]`（每实例 1-based
    文档 id，def_idx 序）寻址；形状 aabb 固定半径 0.0，sphere 半径
    必须为 (0,100000] 有限字面量；源 = 作者句柄变量（source_kind 1，
    全代次 live 绑定）或 `refs.<elem2>` 静态叶（source_kind 0）。
  - `event.next()/pair()/phase()/generation()/source_kind()/source()/
    geometry()` → `ag_event_next/ag_event_pair/…`（各计 1；缺席的
    next 同样计 1；游标不跨 tick 排队）。
  - `object.move(h,x,y,z)/despawn(h)/alive(h)/visible(h,f)/x(h)/y(h)/
    z(h)` → `ag_object_*`（各计 1；`move` 为语言关键字，解析为
    `Expr::Move` 调用头，同端口映射）。
  - `round.request()`（027 已有）→ `ag_round_request()`。
- Start 初始放置：`refs.<elem>.place(x, y, z)`（024 空间写，events
  档静态元素亦发行表）在 Life.Start 配置期展开
  `ag_set_local(<行>, <轴>, …)` 三条；编辑 Document 位姿与 tick0
  接受位姿的区别保留（原 G11：行 y=1、Start place y=0.5）。不通过
  model 绑定循环写位置、不冻结后间接移动（角色/相机 freeze 仍在
  app_start 之后，对象/触发 registry 冻结在其之前——装配序沿
  Triggers3V1 原样）。

行为/装配身份恒为 `autographics.triggers3d/1`（108/profile 19：
motion+char3d+cam3d+objects3d+triggers 组合；无新增运行时版本、
无抬额）。业务全部留在作者 Auto（model/msg/on/helper 任意合法
Node/字段名）：收集/计数/胜败/巡逻/重开条件没有编译器特判，事件
消费的有界策略（旧 G11 移动读一/idle 至多三；第二合法程序可组织
不同有限策略，如每 tick 至多二的无条件读）完全由作者显式写成。

## 1. 执行证据（只述实测）

**真 GPU 链**（driver `node3d_events_flow` 8/8，单线程 64MiB 栈）：
原 G11/3d 25 项 oracle 逐名在新作者链复刻成立——import_three_templates
（8 行 kind 序 Cylinder×2+Mob×3+Cylinder×3）、three_distinct_shard_
templates（0.5/0.6/0.7 两两互异）、start_without_fault、tick0_six_
pairs_bound、tick0_six_instances、orbit_camera_from_source（yaw0/
pitch45/dist8/perspective 源配置）、walk_collects_three_and_wins
（≤8000 步真实相机相对行走，live 6→3 + Beacon enter）、three_distinct_
ids_each_once（pair {0,1,2} 各一、句柄互异；另证 despawn 后无
stay/exit）、windowless_png_three（>3000B 离屏 PNG）、alt_fails_at_
hazard（Saw 球对 x∈[7.0,9.6)）、failed_state_freezes_motion（失败态
10 步零位移）、restart_new_run_tick0（fresh Enter 边沿 → 泵内消费
marker → 新 Run tick0）、restart_rebirths_items（六实例复活、slots
0..2 代 1、模板 A/B/C）、orbit_changes_pixels、camera_relative_walk_
rotates_world_direction（90° orbit 后同键 |Δx|<1e-2、|Δz|>0.02，在
新 Playing host 上复刻）、camera_relative_walk_reaches_the_void_zone
（z<−6）、sphere_corner_outside_does_not_trigger（对角 +0.9/+0.9
d>0.5 球静默）、sphere_tangent_hits（+0.9/+0.5 d≈0.5±0.01 enter）、
aabb_and_sphere_are_distinguishable（同实例 AABB enter 球静默）、
same_tick_three_engine_block（单 tick 三 enter 块）、same_tick_three_
counted_once_each（live 6→3）、old_generation_never_auto_reattaches
（tick4 同槽重生 2..7 无 enter）、explicit_rebind_enters_new_
generation（tick8 重绑二度 enter ≥8）、same_slot_new_generation_
identity（slot0 代 2）、stale_handle_operation_is_rejected（旧代句柄
move → fault 33）。**typed/预算拒绝**：双 move 33 计费 → fault 10；
Start 内 3 place + 1 spawn + 29 object 写 → 第 33 写 → fault 37
（写帽先于计费）；slot 8 → 31；tick0 round.request → 41。**补充
（§6 另列）**：隐藏实例仍触发（Start `ag_object_visible(h,0)` 不撤销
资格，walk 照常胜）；sensor 非游戏正控（swept visitor、静态叶主题、
向下投递两同定义 Marker + 向上 `.Pinged` 计数、每 tick 至多二的第二
有限策略、无 collect/goal 命名业务）。

**编译门**（pilot `node3d_events_gates` 7/7）：g11-main 两次再生长
字节一致 + 全端口发射钉死；alt/triple/oldgen 参数面（speed 2.6、
hy/hbase 0.9、hdir 0.08、三碎片 x=3.0、同槽重生/重绑行）；sensor
接收器隔离（`marker_pings=[0,0]`、`m_a_ping/m_b_ping` 表投递）；14
条具名拒绝（槽/pair 池包络、root-only、模板实例化/params/model/
双叶/solid、未知 slot/模板/pair、bind 主题/形状/半径域）；旧档边界
（components-3d-runtime 下 mob3d scale 仍具名拒绝）。

**Verifier**（`tools/verify_plan028.py --controls` 33 checks PASS）：
七产品 identity（generator 串/api=triggers3d/1/user_source sha/closure
模块哈希）、g11-main 模板行（8 行、kind 序、0.5/0.6/0.7、park 不可见、
编辑位姿 y=1）、七产品闭包冷再生（closure-only B 目录 → 字节相等）、
变异敏感（speed 2.0→2.5 移动 world sha；注释独占编辑字节不变）、14
负控、冻结来源（driver runtime_objects_v1.at/runtime_triggers_v1.at
SHA 不变、Gallery 只读基线四文件 SHA 不变、027 canonical spec SHA
不变）、旧档输出冻结（027 g07a 闭包再生与在库产品字节相等）。

**生命周期/来源**：Stop 文档逐字节一致；20 轮 start/stop 暖上传
mesh 账平坦；A→B 闭包冷再生字节相等（verifier 全 7 产品 + driver
in-process B 目录）；SaveAs 关开再 Run（driver vs05/027 同型）。

## 2. 冻结面与不承诺

- 旧 024/027/020 档：字节不变（`Cfg3.events=None` 全路径；gates/
  verifier 双重复核）。内置视觉静态 scale/yaw/pitch 仅 events 档
  开放；动态 scale/TRS 在两档仍具名拒绝。
- 原额：全 Node 共享 32 credits/callback（move 26+held 1+next 1+
  pair 1+despawn 1=30 原价目原样）；typed 端口零额外读取；第 33
  计费 fault 10、第 33 lifecycle write fault 37；8 槽/8 pair/120+8
  基座/512 mesh/8 有状态 Node 沿既有受信单元（本计划零运行时改动，
  runtime_objects/runtime_triggers/runtime_round 源字节锁不变）。
- 不承诺：Gallery 内容迁移（另立合同）、多物理角色/动态 Node 拓扑/
  自动销毁 SoA/跨维触发/3D 查询/纹理动画声音组合、抬额、OS 键鼠
  自动化（窗口期缺失证具名 gap，与本仓 025/027 边界一致）。
