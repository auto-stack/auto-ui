# AutoGraphics Node2D G11/2d 作者入口（auto-ui PLAN-017）

Status: delivered by auto-ui PLAN-017 r1（实现/验收提交见计划归档收据；本文件
只描述实际交付并验证的能力，不声称任何未实现的语言/运行时特性）。

## 0. 范围

显式研究入口 `node2d-lower --profile g11-2d` 接受普通 Auto 模块工程
（`node`/`view-2d`/`Life.Start`/`Life.Tick`/匿名 `msg`/前导点/普通 `use`），
按声明生成 AutoGraphics `triggers2d` 组合的真实用户源 + `autographics.scene/6`
场景工程 + `closure/` 输入闭包副本，经既有对象/触发/角色/相机/round 服务与
pinned SDK 真执行、真离屏 GPU 呈现。原 G11/2d 全部玩法（三异形物品差量销毁、
巡逻危险、集齐才赢、左侧越界失败、终局冻结、新鲜 Enter 经 accepted 请求重开）
与原 51 项 oracle 语义在新链完整重建；主/alt 参数孪生验证真实敏感性。

非范围（不冒称支持）：3D Node/G11-3d、跳跃/声音/HUD、完整 G2 平台游戏、
新物理/事件队列、无限节点、通用集合库、任意 key 子树重建、多呈现/混合 UI、
`Life.Frame`、生产 widget 退役、ACE/AOT/ECS/HMR、新平台。旧 016 pilot
（`--profile` 缺省）保持字节兼容（verify_plan016 再生控制）。

## 1. 声明面（类型驱动，无名字特判）

- `node Name(param: Type, ...) { model? msg? view-2d? on? }`；参数端口按
  类型识别：float/int 数值状态、`msg` 事件端口、`ActorRef2D` actor 引用
  （绑定 `refs.<实例>.actor`）、`Mesh2D`/`Material2D` 资源 symbol（绑定
  `resources.<name>`，决定该实例模板行的几何/颜色）。
- view-2d 元素：`solid2d`（仅根 view；地面端口）、`trigger2d`（触发体；
  `shape: circle(radius: r)` 或缺省 AABB；`position` 引用 model 可变字段
  即动态位置）、`character2d`（角色宿主端口；恰一个）、`camera2d`（≤1，
  仅根；`bounds`+`view_size`+`target`）、实例化与 `if` 条件（真
  spawn/despawn 差量）。
- 触发 phase 绑定：`on_enter`/`on_stay`/`on_exit: .事件`；事件路由读
  `ag_event_phase()`（1/2/3），未绑定 phase 的事件消费不投递。同定义
  多实例可绑定不同接收事件（逐实例路由）。
- 输入/重开服务：`input.axis(Left, Right)` → 真实差值轴（held 计费 2；
  Left=-1/Right=+1/都无与同时=0）；`input.pressed(Enter)` → 消耗性边沿
  （计费 1；动作 0=Left/1=Right/2=Jump/3=Enter）；`round.request()` →
  `ag_round_request()`。
- 校验在执行前带文件/行定位拒绝：缺模块/错导出、未知实例属性、错端口
  类别值、事件未声明/无处理器、接收方处理器形参数 ≠ 绑定载荷数、phase
  绑定带载荷、shape 非法、role 元素越界（solid/camera 出根 view）、
  character2d 非 1、坏 refs 引用等；无 panic/半写输出。

## 2. 降层形状与调度

- 模板表声明驱动：角色 row id=1（在位=声明位置）、根 solid=2..、触发实例
  模板按根 view 声明序停靠玩法区外；pair/slot 同声明序（G11 夹具 =
  原 001 的 pair 语义位 0..5 与模板 3..8）。
- 事件端口投递经逐实例常量表（枚举内联判整值）；根父直调/实例父走表。
- app_update 固定顺序：根 Life.Tick（重开检查读 tick 起点状态）→ 实例
  Life.Tick（声明序）→ 事件段（移动 tick 恰 1 条、静止最多 3 条，
  `_n2d_moved` 由 move_and_slide 包装置位）→ 动态位置图形同步（首条
  `ag_event_next()==0` 的 tick 才写；有事件 tick 跳过写、巡逻状态照常
  推进——原 001 等价策略）→ 条件差量。
- 重开：终局后新鲜 Enter 边沿 → `ag_round_request()` → 宿主 accepted
  开新 Run（物品重生、计数/订阅重置；held 不连开；旧 Run 事件不串局）。
  声明差量：胜/败同 tick 的 Enter 边沿不消费（下一新鲜边沿照常）。

## 3. 预算（pinned SDK 7b987ca8，triggers2d，全部实测）

共享 32 credits/callback 池：移动收集 tick 恰 32（held2+sweep26+next1+
pair1+phase1+despawn1，第 33 计费调用 = 命名 sticky fault 10）；普通移动
tick 30（含动态位置同步 1）；静止三收集 14；胜 tick 31；终局后 4；重开 5。
VM 分片 1000steps/2ms、累计 100000steps/120resumes/250ms、30s init/2s tick、
8 slots/8 pairs、代次 handle（旧代跨 Run fault 33）、Pause/Step/Resume/Stop
语义全部复用未改。

## 4. 验证面（全部在本仓）

- `cargo test -p auto-ui-dialect --test node2d_pilot`（6 例，legacy 兼容）
- `cargo test -p auto-ui-compiler --test node2d_pilot`（6 例，legacy 兼容）
- `cargo test --manifest-path experimental/autographics-node2d-pilot/
  Cargo.toml --locked --offline`（legacy 9 例 + `node2d_g11_gates` 10 例：
  诊断定位拒绝 + 改名/重排/换资源/移位敏感性 + 世界结构不变量）
- `python experimental/autographics-node2d-pilot/tools/verify_plan017.py
  --controls`（8 世界身份绑定、再生字节等价、变异敏感、缺模块诊断、
  篡改检出、closure-only A→B 预检）
- driver `cargo test --locked --offline --test node2d_g11_flow --
  --test-threads=1`（15 例：主链 walk→3 有序收集→胜→冻结→新鲜 Enter
  重开→重生；alt 危险失败/冻结/重开/旧局隔离/held 不连开；CPU 恰 32
  校准；左行越界失败；同 tick 三件 3 enter/3 读/3 销毁；stay 分类；
  bag2 终点门+exit phase；半径几何三态；同 slot 代次重绑；both-moved
  终位判定+away 永不进；Pause/Step/Stop；20 轮 owner 释放+旧代句柄
  fault 33；hidden-still-triggers engine fact；GPU 像素类+无窗 PNG）
- 旧触面：`node2d_flow` 6/6 与 verify_plan016 PASS（016 冻结字节不变）。

## 5. 消费与 Provenance

- 公开准备：`node2d-lower --profile g11-2d --entry <main.at> --out <dir>`
  （world.at/scene.agscene/identity.json/closure/）；正常消费
  `autographics-authoring --open <scene> --behavior <world.at>
  --behavior-api triggers2d` 进入真实 Run；生成物为冻结数据，再生逐字节
  等价，closure/ 单独即可冷再生（A→B 已实测：新根导出 + 冷 SDK 681
  files + 冷 target 全量复现并全量测试通过）。
- 依赖：pinned SDK `7b987ca8`（driver `.sdk-source`）；生成器只读引用
  auto-lang 98270a4f5（delivered master）+ auto-web f29c7d1 + auto-down
  c3464d0 兄弟树，语言仓零写面；两图（tip 生成器 / pinned-SDK driver）
  绝不同图。
- 历史 auto-lang AG Specs 保持只读知识根；本文件为 auto-ui 仓内新增
  （SD-01），未改写既有 Spec。原 51 项语义映射表见计划归档的执行合同。
