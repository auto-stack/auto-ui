# node3d-audio-hud-composition

SD-01 candidate（PLAN-035 交付；canonical 落地由 merge 核验）。
三维通用声音/HUD/姿态组合接线：普通 Node/use/局部 model-msg-on 上的
typed 声音请求、四槽 HUD 收口、有界事件快照与整批销毁，装配身份
`autographics.components3d/1`（number 117 / profile 28 / cli
`components3d`）。原 32 计费池、物理/资源/VM 包络与全部旧档字节保持
不变；正式 G3 游戏内容仍归未来独立合同，本 Spec 只新增作者面规则与
公共引擎事实。

## 1. presentation3d 根块（025 同文声明面）

- `presentation3d { audio { bank/font/license } cue <名> { span, gain }
  hud { slot 0..3 { owner/anchor/offset/size/rgba/text_fn/text_field/
  visible/init } } }`：恰一根 node 恰一块；块归属缺失/重复/非根具名拒绝。
- 无 trigger 行（具名拒绝）：三维触发归属 030 `triggers { }` 池与作者
  `bind()`，事件消费走有界快照面——双机制回流不开放。
- cue id = 声明序（typed 映射，作者不写数字 id）；跨模块同名解析，
  重名具名拒绝；span/gain 域沿 025 语义（gain 域外运行时 fault 54）。

## 2. 装配身份与 actor-free

- components3d/1 由本块触发；char3/cam3 装配改为 app_start 条件化：
  恰一 character3d 在案绑定角色+轨道相机（bundle kind=0 携姿态）；
  无角色 Run 为唯一合法 actor-free 3D 路径（kind=-1，scene
  `actors.three_d=null`），同一源在旧 hud3d/1 下保持 fault 25。
- 有角色必须根 camera3d 块、无角色禁止 camera3d（双向具名拒绝）。

## 3. 声音请求与 HUD 收口

- `sfx.play(<cue>, <gain>)` → 生成收集器 `sfx_play(<cue_id>, <gain>)`
  （0 计费）；恰 5 行缓冲把 n=5 送入具名 fault 51（≤4 请求/回调）；
  未知 cue 具名拒绝。8 voice/Run、audio_unavailable/audio_fault 沿 012。
- 每回调末恰一次 `ag_present3d(kind, entity, pose…, n, 4×(o,c,g),
  4×(visible,text))`（bundle 整体 1 计费）：四 HUD 槽完整状态，未声明
  槽缺省隐藏空文本；文字 helper `text_fn(text_field)` 沿 025 签名门。

## 4. 有界事件快照与整批销毁（032 同构三维绑定）

- `events.snapshot()`（1 计费，≤1/回调，0..8 行全字段）+
  `events.count/run/tick/pair/phase/source/generation/source_kind/
  template/source_generation/geometry`（快照 getter 0 计费 Auto 计算）；
  第二次快照/与单 cursor 混读具名 fault 58。
- `objects.despawn_batch(h1..h8)`：1..8 同 Run 活句柄整批全有全无，
  1 计费/批；第 9 项具名 fault 59，陈旧代次 fault 33，物理写仍按 n 计
  （第 33 写 fault 37 不变）。

## 5. 姿态随 bundle 收口

`refs.<body>.pose(clip, phase, loop, yaw)` 降层为生成基础设施写
（0 额外计费），独立视觉 pose 不再发出；actor-free 组合 pose 具名拒绝
（无视觉可请求）。clip/序数/叶规则沿 034 Spec 不变。

## 6. 原预算完整 G3 链与引擎事实

- 原 32 计费池/物理写/节点/槽/资源/VM 上界全部保持；收集 Tick 实测
  31/32、边界变体恰 32/32、稳态 30；原 G3-01..08 在同一程序逐项成立
  （独立 f32 轨迹 oracle、真失败负控、20 轮 churn、真实 native PCM）。
- 引擎事实（永久夹具已按此适配）：场景圆柱皆为角色 sweep 的物理盒
  （visible 无关）——危险/坑体积用生成隐形标记物 + sphere bind 表达；
  角色 sweep 盒在面向轴向可达标称半深 ~1.0 之外，固 z=2 航道需要
  标记球半径余量；goal 火 半径 1.8（coin-3 点 2.06 在外、3/3 航道
  ~1.63 进火）；ended 轮询要求 hero.Move 先于 status 门；逐币销毁须
  清柄（陈旧句柄 fault 33）。

## 7. 验收与永久门

生成门 `python -I tools/verify_plan035.py --controls`（三永久夹具
A→meta→B 冷再生逐字节、identity/场景/world needle、冻结源
composition3d/audio3d/hud3d 字节、SDK681、素材与 cue 权威 span、
verify_plan034 旧档并集）；运行门 driver 侧单实例
`node3d_audio_hud_gates`（字节冻结+具名拒绝矩阵 26）、
`node3d_audio_hud_flow`（原 G3 八项+oracle+生命周期 9）、
`public_3d_composition_api`（域/容量负控 5）、
`node3d_audio_hud_storage`（版本化闭包保存/搬移/重开/坏件拒绝→精确
恢复→同入口再绿 3）。未来游戏消费面：公共链 + 三永久夹具 + 上述
冻结命令；正式内容交付仍须独立合同。

## 独立窗口消费者（PLAN-072 SD-03 增补）

`age-game-player`（docs/specs/auto-graphics/standalone-windows-game-player.md）
是本组合的新增独立窗口消费者：以 `Host::with_project_player` 复用同一
components3d 装配与同一 Play/运行相机/音频机制；左拖 orbit、滚轮 zoom、
固定 dt 与预算沿原契约（0.25°/逻辑px、×1.1^-1 每格、距离夹 4..12、俯仰
15°..75°），无第二实现。原规则/预算/触发/回合语义逐字节不变。迁移归属
auto-lang/PLAN-981（SOURCE/SESSION/CONTENT/AUDIT）。
