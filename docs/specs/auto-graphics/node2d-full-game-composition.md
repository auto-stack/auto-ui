# node2d-full-game-composition

状态：canonical 候选（PLAN-032 SD-01；交付即 canonical）。本文描述 PLAN-032
交付后的完整二维游戏组合服务面：普通 Node 作者链（use/node/model/view-2d/
msg/on/Life.Tick）加上 `components2d` 作者块，表达角色/敌人/同定义收集物/
目标/隐藏危险区与根关卡状态，在原 Callback32 池、原 VM/资源包络内以冻结
完整 G2 路线与合法峰值实测成立。020 局部组件、022 精灵、025 通用音画、
001 对象、002 触发/新局的原档案、价格与负控原样保留。

## 1. 服务定位与身份

- 受检组合：`behavior.api = autographics.components2d/1`（number 116 /
  profile 27 / cli `components2d`），恰一 character2d 且恰一 camera2d；
  装配 = 完整 hud2d 栈 + 有界组合单元（`api/public_composition2d_v1.at`），
  全部派生字节（旧 profile 装配字节不变）。
- 生成器：pilot `--profile components-2d-runtime`；作者块 root-only；
  骨架按 T-02 冻结合同生成，业务逻辑全部由作者普通 handler 决定——
  编译器无业务名特判。
- 冻结作者合同：根 model 恰声明 `state: int`（0=Playing/1=Won/2=Failed，
  生成巡逻门与终局 Enter 读它）；角色定义 model 恰声明 `pose: int`
  （0 idle/1 run/2 rise/3 fall）与 `facing: int`（1 右 / -1 左，生成帧
  收口读它们）。缺一具名拒绝。

## 2. 有界组合单元（components2d/1 新公共操作）

- `ag_events_snapshot() int`：每回调至多一次，1 credit。物化本回调继承的
  完整 0..8 条事件（pair 序、enter/stay/exit phase、source 身份 kind/ref/
  template/generation、绑定代、shape）+ run/tick；空合法返回 0；未读事件
  不跨 tick 排队。字段读（`ag_event_snap_pair/phase/generation/source_kind/
  source/template/source_generation/geometry`、`ag_events_run/tick/count`）
  属 Auto 计算（VM/CPU 全计，0 额外 credit）。第二次快照、快照与单条
  cursor（`ag_event_next` 等）混用、越界索引：具名 fault 58。快照是独立
  不可变值：保留句柄仍走对象单元原 alive/generation 验证。
- `ag_object_despawn_batch(n int, h0..h7 int) int`：显式整批销毁。1 credit
  （空批合法仍计 1）；长度域 0..8（第 9 项/负数具名 fault 59）；先全验证
  （alive、run/slot/generation 全身份、重复）后修改，全部销毁或可观察对象
  表原样；物理写计数加实际 n，第 33 物理写先具名 fault 37；旧代/错 Run
  沿对象单元具名 fault 33；不合成 exit、不自动重绑（与单条 despawn 同
  语义）。
- 作者服务（仅本 profile）：`input.snapshot()` → 读回调缓存快照 `r_snap`
  （生成前奏每回调恰一次 `ag_actions_snapshot`，pressed 位恰一次消费；
  0 额外计费）；`obj.despawn(<handle>)` → 单元素批调用。

## 3. 生成骨架（每回调恰一次语义）

- 前奏：`r_snap = ag_actions_snapshot()` + `cmp_events()`（快照 poll，
  事件 i 按 pair 派发 `<leaf>_<event>(def_idx, phase)`，guard 为目标定义
  model 字段 ==1）。
- 组件 Life.Tick（作者）：物理/规则；本 profile 的 G2 峰值账 = snapshot 1
  + char move 26 + grounded 1 + 巡逻 1 + events snapshot 1 + 批销毁 1 +
  收口 bundle 1 = 32（无任何职责跳过；无 idle-turn 门、无停走/停敌人/
  隔帧消费）。
- app_update 末：巡逻行（Playing 态每回调恰一次 `ag_object_move`）+
  终局 fresh Enter（`state != 0` 且 pressed Enter → `ag_round_request`）+
  收口 `ag_present2d(kind=0, …)`：角色四帧 + flip + ≤4 声音（恰 5 行缓冲，
  第 5 行具名 fault 51 无部分接受）+ 四槽 HUD（bundle 整体 1 credit）。
- `components2d` 块行：audio/cue/hud/trigger 与 presentation2d 逐字同语义；
  `patrol { node range at_y speed }`（恰一行；node = 根视图实例，单实例
  定义）；`atlas { asset frame_size idle run rise fall }`（有角色必声明；
  typed png 绑定在角色 scene 行）。

## 4. 预算与资源账（实测）

冻结完整 G2（m2 矩阵原几何：dt 1/60、速度 4、重力 20、跳跃 8、clearance
0.002、原坑/墙/顶、敌人 x=11..17 巡 1 单位/秒、三币 + 目标 r=0.5、相机
12×8 夹 [0,30]×[−4,12]）：

- 行走 tick = 31（+1 恰 32 池绿、+2 具名 fault 10）；收集 tick = 32（+1
  恰在首次收集处 fault 10）；胜/败 tick ≤ 32；app_start 22 credit。
- 手工先证世界与生成器产物同账：首次收集同 tick（108）；完整路线
  （G2-01..09：释放停走、地面单跳空中拒二跳、三币一次真销毁、巡逻触敌
  失败、未齐不赢/集齐触目标胜、Enter 新局、跟随夹位相机、idle/run/rise/
  fall+方向帧、跳/收声音与 HUD）全绿。
- 原资源/VM/source 包络不变：128 节点、对象 120+8 槽、八 slot/八 pair、
  八状态 Node、1000 步/slice、32 物理写/callback、256KiB/96KiB/352KiB
  source、8MiB/64MiB 资源等全部沿旧守护。

## 5. 兼容与消费

- 旧 profile（v1..presentation2d）装配字节逐字节不变（装配 pin 测试）；
  新操作仅存在于 components2d/1 装配；旧源不自动获得新保留函数。
- 受检负控：快照/批的全部具名拒绝（58/59/33/37/10/51）实测；损坏世界
  副本按 scene 绑定 sha 拒绝、精确恢复后真 Host 起跑。
- 正式游戏放独立仓（ag-examples/games/platformer-2d，AG014）；游戏 T-05
  才绑定本计划实际交付源/SDK/锁/身份。接口以本文 + identity.json 为准；
  变更仅走新 revision 与影响清单。
