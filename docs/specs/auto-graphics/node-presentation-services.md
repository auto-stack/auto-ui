# node-presentation-services

状态：提案（PLAN-025 SD-01 准备稿；canonical 化归 merge 门）。本文描述
PLAN-025 交付后 auto-graphics 的通用二维 Node 声音/HUD 接线服务面，替代
「023 固定收集游戏作者档」作为后续内容消费的唯一通用入口；023 的
components-2d-game 档与其冻结产物、价格、负控原样保留，不受本服务影响。

## 1. 服务定位

在 020 有状态组件与 012/013/023 已交付媒体服务之上，提供显式、可复用的
二维 Node 声音/HUD 接线：

- 计数、收集/销毁、胜败、cue 选择、gain、文字和输入规则全部由作者的普通
  Auto model/msg/on/helper 决定。编译器只映射 typed 资源/引用、事件、
  有界请求收集与每回调恰一次收口；不存在 Collect/Goal/Fail 业务枚举，
  不存在根 count/state 要求，不存在按业务名字（Coin/Player/goal）分派的
  生成规则。
- 同一定义多实例天然状态/接收方隔离（020 机制）；声音 ordinal 为单 Run
  共享的精确 int 序列，非每 Node 独立发号。
- 两种受检组合：
  - 无角色（actor-free）：场景 `behavior.api = autographics.presentation2d/1`、
    `actors.two_d = null`；无 character2d、无 camera2d、无触发行。
  - 声明了角色（受检组合）：`behavior.api = autographics.hud2d/1`（原装
    配不变）；恰一 character2d 且恰一 camera2d；触发行 subject 恒为该角色。

## 2. 作者入口（presentation2d 块）

根 node 的 `presentation2d { ... }` 块（root-only；非根具名拒绝）：

- `audio { bank/font/license }`：与 023 同规则（有限 WAV PCM16、
  mono/stereo、22050/44100/48000Hz、raw ≤8MiB、≤10s；font/license 原样）。
- `cue <name> { span: (s, e) gain: g }`：cue id = 声明序 = scene cues 数组
  下标；span 半开对界 bank 帧数；gain ∈ [0,1]。作者 handler 内以
  `sfx.play("<cue>", <gain 表达式>)` 发起 typed 请求——cue 为字符串字面量
  由编译器映射为 id（作者不写数字 ID；未知 cue 具名拒绝）。
- `hud { slot <0..3> { ... } }`：版面（anchor 0..3、offset、size 16..48、
  rgba 0..1）+ `owner`（根视图组件实例或根本身）+ 可选文字绑定对
  `text_fn`（作者 helper，签名恰为 `(int) -> str`）与 `text_field`
  （owner 的 int model 字段）+ 可选 `visible`（owner 的 int model 字段，
  缺省恒可见）+ `init`（≤32 可见 ASCII）。未声明槽默认隐藏/空。运行期
  活文字 >32 字符为原 fault 56。
- `trigger <name> { shape: circle|box, radius?, target, event, guard? }`：
  generic 触发行（pair = 声明序 0..7）。无业务载荷——`on:`/`cue:`/
  `value:`/`when_count:`/`phase:` 在本块内解析层具名拒绝。目标必须为根视
  图运行时实例（恰一动态 shape2d；区域规则沿 023 F-6）；event 必须是目标
  定义的已声明 msg；guard 为目标 model 字段（==1 才派发）。事件 tick 由
  生成 poll 前奏派发 `<leaf>_<event>(j, phase)`——phase（1 enter / 2 stay /
  3 exit）作为 handler payload 转发；每回调恰一次 `ag_event_next` 首读，
  事件 tick 恰一次 pair + phase。
- 保留名（本 profile 具名拒绝）：根字段 ord/np/o0..o3/c0..c3/g0..g2；作者
  fn 名 sfx_play/pres_events/pres_closeout；model 字段沿用 020 F-8 生成
  缓存保留名（handle/px/py/visq）。

## 3. 请求收集与一次收口（生成基础设施）

- `sfx.play` → 生成 `sfx_play(<cue_id>, <gain>)`：纯 Auto 计算（0 额外服
  务费；CPU 计入原 32/callback 池），向有界缓冲追加 (ordinal, cue, gain)。
  单 Run ordinal 由共享计数器严格递增（1..2147483646 由原 API 边界强
  制）；缓冲存储恰 4 行并计数至 5——第 5 请求只递增计数，使收口以真实
  计数 n=5 进入 `_ag_audio_begin` 的具名 fault 51（包界），第 6+ 请求保
  持 n=5。
- 每回调末恰一次 `ag_present2d(kind=-1, n, o0..o3, c0..c3, g0..g3, 四槽
  完整状态)`（bundle 整体 1 计费，沿原 API 价格）。四请求全部传输、精确
  ordinal/gain；五请求具名拒绝、无部分声音接受；n=0 空收口仍为一次合法
  bundle。原 standalone `ag_audio_emit` 仍是公开逃生口（1/次），与 bundle
  混用为具名 fault 50；同回调第二 bundle 同样 50。
- 视觉发布不变：shape/character 仍走原 ag_object_*/character 路径；本期
  无 sprite 视觉写（kind=-1 仅声音+HUD）。
- 作者销毁面：`render2d.despawn(.handle)`（本 profile 仅开放 despawn）；
  `.handle` 为只读生成句柄（本定义恰一动态 shape2d）。

## 4. 原预算与验收保持

- 世界共享 32/callback 及原价格不变：move 26、snapshot/grounded/next/
  pair/phase/despawn 各原价、bundle 1。实测参考程序（G12 等价作者程序，
  含收集/未齐到终点/站立/重入/持续按/Jump-land 分支）walk tick 恰 32，
  第 33 credit 为具名 fault 10；事件 tick 经作者自有输入规则（halt 跳过
  压感+双 held 读）恰 32。无自动 Idle：halt 是作者 Auto 逻辑。
- 原 VM 1000steps/2ms 分片、100000steps/120resumes/250ms、init 30s/
  tick 2s、源 256KiB/managed 96KiB/combined 352KiB、128 场景行/8 有状态
  组件/8 对象槽全部不变；presentation2d/1 装配为 hud2d/1 派生（去相机单
  元），managed 面不增。
- 原媒体/寿命/故障/资源/预算规则不变；8 资源身份/32MiB、CPU/GPU 各
  64MiB、8 磁盘文件 32MiB、四 atlas key 等沿 012/013/023。native 路径
  无新代码（同一 Mixer/native 单元），声学维度 gap 沿 023 具名保留。

## 5. 验收锚点

- V-01：任意命名/typed 资源/严格诊断（tests/node_presentation_gates.rs，
  18 控制组）。
- V-02：无角色真实 Auto/GPU/PCM（tests/node_presentation_flow.rs
  t02_*；真实 GPU 字形经 Host capture）。
- V-03：作者 G12 等价全分支 + 独立 f32 轨迹 + 32/fault10 边界
  （t03_*）。
- V-04：四请求非截断 + 第五请求 fault 51 无部分接受（t04_*）。
- V-05：fresh Run ordinal 隔离、standalone/bundle fault 50、ordinal 重放
  fault 52（t05_*）；pause/step/20-round/native 沿 023 共享机制证据
  复用。
- V-06：closure-only 冷再生字节等价、坏件拒绝/精确恢复、旧 023 消费者
  全绿（tools/verify_plan025.py --controls）。
