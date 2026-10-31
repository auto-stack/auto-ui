# AutoGraphics Node2D 研究 pilot（auto-ui PLAN-016）

Status: delivered by auto-ui PLAN-016 r1（实现提交见计划归档收据；本文件只描述
实际交付并验证的能力，不声称任何未实现的语言/运行时特性）。

## 0. 范围

显式研究入口 `autographics-node2d-pilot`（独立 workspace，node2d-lower 生成器）
接受一个普通 Auto 模块工程（`node` 定义 + `view-2d` + `Life.Start/Life.Tick`
+ 匿名 `msg` + 前导点成员/事件 + 普通 `use` 模块闭包），校验后降层为
AutoGraphics 既有 `triggers2d` profile 的单文件用户源 + `autographics.scene/6`
场景文档，经真 Auto 执行与真离屏 GPU 验证。一个 Player、两个同定义独立状态
Hazard、一个可销毁 Coin 为冻结验收夹具。

非范围（不冒称支持）：`view`/`view-3d`/`view-portrait`/`view-widget` 与混合
嵌套、多呈现选择器、`Life.Frame`、完整 G11（三 Coin/伤害/终点/Enter 新局）、
PNG/GLB 资源导入、生产 UI/ACE 融合、ECS 或一 Node 一 VM、`emit`/refs 作为
最终语言形态（二者在本 pilot 内按 §3 降实现）。

## 1. 研究文法面（显式入口，生产路径不变）

- `auto-ui-dialect::node2d::Node2DDialect` 仅经 `parse_domain(&Node2DDialect)`
  可达；`node` 不在生产 `UiDialect` 关键字表内，旧 widget/store 解析路径零变化。
- `node Name(param: Type, ...) { model? msg? view-2d? on? }`；参数与 model
  字段同为实例状态（`var phase = phase0` 允许引用参数）。
- `view-2d` 域标记按 token 组装识别（`View`+`Sub`+数字 `2`），共享词法零改动，
  普通表达式减号语义不变。元素：`solid2d`/`trigger2d`/`camera2d`、
  `NodeName instance { props }`、`if cond { ... }`。
- `on` 块为研究域自有解析：`Life.Start`/`Life.Tick(dt)` 系统事件（不在 msg
  声明）与前导点 `.Business` 业务事件（必须在匿名 `msg` 声明）混合；处理器体
  复用共享体文法（`emit .X`/前导点语句）。
- 校验在执行前给带文件/行号的拒绝：缺模块、错导出（具名 use 项不在目标
  模块导出面）、坏 actor 引用（`refs.<未知实例>`）、未知实例属性、重复
  node 定义、参数/model 字段重复绑定、非 UpperCamelCase node 名、未知
  view 元素、非法 on 模式、`Life.Frame` 越界显式拒绝。

## 2. 降层形状（真实 Auto 执行，无第二解释器）

- 每 node 定义一组 SoA 数组（`hazard_phase=[0.1,0.45]`），处理器为真实 Auto
  函数（首参实例索引 `i`）；根 node 用标量（`r_coin_present`）。根与实例同名
  字段不共享（哨兵 oracle）。
- 生成源 = 纯函数 verbatim 切片（如 `patrol_y`）+ 逐定义处理器 + 编排
  `app_start/app_update`。玩法规则（巡逻波、收集守卫、销毁）全部在 Auto；
  Rust 仅做解析/校验/降层与图形服务连接。
- 服务映射（冻结）：`input.axis(Left, Right)` → `ag_action_held(1)`（单次
  计费；完整差值轴为后继）；`refs.body.move_and_slide((dx,dy))` →
  `ag_char_move(dx,dy,0.002)`；`refs.body.move_to(x,y)` →
  `ag_object_move(<handle>,x,y,0.0)`（`move` 与 Core move 运算符冲突故更名）。
- 事件投递：`emit .X` 经绑定表（`coin_bind_GRAB[i]`）路由到所属父实例的
  处理器调用；同定义多父实例互不串线（两父夹具实测）。
- 条件分支（`if .coin_present == 1 { Coin ... }`）由差量执行器在事件处理后
  spawn/despawn 真实对象与触发对（非 visible=false/移出场外/日志代替）。
- 枚举降为 int 常量（`KEY_AMBER=0`）；`ActorRef2D` 为运行时 handle 采样
  （`ag_actor(2)`）；父子成员引用属性在 `app_start` 运行时初始化。

## 3. 执行与预算（pinned SDK 7b987ca8，triggers2d 组合，全部实测）

- tick 顺序固定：宿主输入采样（每 tick 一次 `_ag_char_input`）→ 根/Player/
  Hazard 逐实例 `Life.Tick` → AG end-of-tick 触发 evaluate → 下一 tick 事件
  读取与投递 → 条件差量 → 呈现随 admitted block。
- 共享 32 credits/tick 池：普通 tick ≈30；收集 tick 恰 32（held 1 +
  char_move 26＝24 sweep+2 scan + 2×hazard move + event_next 1 + event_pair 1
  + despawn 1）；第 33 个计费调用为命名 sticky fault 10（同 tick 内直接计费
  读叠加同样拒绝）。VM
  分片 1000 steps/2ms、累计 100000 steps/120 resumes/250ms、30s init/2s tick、
  8ms pump 目标照旧未改。
- 场景模板停靠玩法区外，实例从模板 spawn（born-overlap 会永久锁死扫掠运动，
  实测）；char 定义在冻结场景表 row 0，`ag_local(0,x)` 提供零计费位置读回；
  全部验收读回走纯 Auto getter（计费读取每 tick 限次）。
- 生命周期复用未改的 Play 语义：Start 每 Run 恰一次、Pause 不推进、Step 恰
  一次中性 tick、Resume 不重 Start、Stop/fresh 回初态；8 slot/8 pair、代次
  handle（旧代跨 Run 写为命名故障 33）。

## 4. 验证面（全部在本仓，命令见计划归档）

- `cargo test -p auto-ui-dialect --test node2d_pilot`（文法/诊断 6 例）
- `cargo test -p auto-ui-compiler --test node2d_pilot`（模块绑定 6 例）
- `cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml
  --locked --offline`（生成器单元/再控制）
- `python experimental/autographics-node2d-pilot/tools/verify_plan016.py
  --controls`（身份绑定、再生字节等价、夹具变异敏感性、缺模块诊断、
  生成物/场景篡改检出）
- driver 侧 `cargo test --locked --offline --test node2d_flow --
  --test-threads=1`（6 例：CPU 实例隔离/事件/幂等、预算满池负控、旧代
  隔离、GPU 像素类证据、双父 receiver 隔离、Pause/Step + 20 轮生命周期、
  CPU 20 轮 owner 释放）

证据分层：Auto 状态（纯 getter）↔ AG 帧状态（slot live/generation）↔ GPU
像素类（光照校准的相对通道规则：红/橙/金/地面），收集事件三层交叉一致。
生成物为冻结数据（`generated*/world.at` 等），再生必须逐字节等价。

## 5. Provenance

- auto-ui PLAN-016（drafting→executing，本 Spec 随交付归档）；合同与实测
  修订：`docs/plans/attachments/016/work/node2d-contract.md`。
- 依赖：pinned SDK `7b987ca8`（driver `.sdk-source`，source-lock 校验）；
  auto-lang 只读引用（生成器树 @ 计划记录的 commit），无语言仓写面。
- 历史 auto-lang AG Specs（overview/behavior-api/runtime-objects 等）保持
  只读知识根；本文件为 auto-ui 仓内新增，未改写任何既有 Spec。
