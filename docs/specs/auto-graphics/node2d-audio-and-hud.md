# AutoGraphics Node2D 声音与 HUD（auto-ui PLAN-023 SD-01 候选）

Status: candidate（auto-ui PLAN-023 T-03..T-06 实测交付；本稿为
docs/specs/auto-graphics/node2d-audio-and-hud.md 的 SD-01 候选文本，
merge 阶段按收据落位。只描述实际交付并验证的能力，不声称任何未实现的
语言/运行时/硬件特性。）

## 0. 范围

`node2d-lower --profile components-2d-game` 接受普通 Auto 模块工程 + 顶层
`node Main` 内的 `game2d` 块（bank/font/license 资产、cue 表、hud 槽、
authoring 触发表），把 020 的有状态组件模式升级为**带一次收口（closeout）
的完整游戏链**：生成 `autographics.scene/8` 工程（`behavior.api =
autographics.hud2d/1`，assets[].audio/ttf 行 + 顶层 cues 数组 + 四槽 hud）
与真实用户 Auto 源，经既有机制执行：每定义 SoA + 按实例索引 handler +
下降期绑定表同步直呼。唯一交付产品 = collect-the-coins 游戏（T-02 手证
链的生成器重写，world sha `ed98b0b5…`，FROZEN）。执行证据（只述实测）：
**离屏 GPU + 真实 PCM**（宿主 play 路径：全场景渲染、字形 HUD、真实混音
逐位对照、native 设备提交流观察——t04_game_media 2、t05_game_host 7）；
**CPU 会话**（受信 VM、状态 getters 逐值断言：通关/失败/once-only/双
cue bundle/序数全序/重启形——t04_game_play 7）；**计费与故障**
（t05_game_budget 5）；**消费与身份/收口纪律/坏件**
（t06_game_closeout 10 + `tools/verify_plan023.py --controls` 10 控制组）。

非范围（不冒称支持）：多角色/多 Run 并发、动态拓扑、查询/PNG/精灵/3D、
`Life.Frame`、ACE/AOT/ECS、音频合成/流式（只有限型 one-shot：3 cue 全部
来自同一冻结 bank 的声明切片）、**真实声学输出**（native 观察覆盖提交进
已开流的采样，不覆盖声学/物理设备维度——Plan D-04 具名 gap，见 §4）。
**旧入口不变**：016/017/019/020/021 既有 spec 与生成字节冻结（本任务
复跑 node_components_flow 9、node_queries_flow、node2d_g11_flow 全绿），
`components-2d-game` 不覆写 `components-2d` 冻结字节。

## 1. 声明面（T-03 冻结拼写，V-01 负控全部具名拒绝）

- `game2d` 块（仅根 Main）：`audio { bank, font, license }`（路径相对
  入口目录；license 须 UTF-8，进 scene `hud.license` 与 closure 副本）；
  `cue <名> { span: (s,e), gain: f }`（cue id = 声明序=数组下标；span 对
  真实 bank 帧数对界、gain∈[0,1]，越界具名拒绝）；`hud { slot N {…},
  text <名> {…} }`（恰四槽；字符串=字面量/分支 helper fn 返回 str/`f"$expr"`
  插值，legacy 同款；裸参数 RHS 赋值拒绝不变）。
- `trigger <名> { shape: circle|box, radius?, target, event, on, guard?,
  cue?, full_at?, full_cue?, when_count?, phase?, value? }`：authoring
  触发表按表序生成 `ag_trigger_bind`（对序=表序、主体恒为角色实例）。
  **区域规则（F-6）**：goal/hazard 触发区必须是运行时实例（如 Zone 组件），
  静态叶目标具名拒绝（角色 sweep 视每叶为 solid，实测不可重叠）。
- 组件定义/实例化/事件同 020（`model`/`msg`/`on`、SoA、选择器调用、
  typed 四类参数只读）；本产品实例化 Coin×3 + Player + Zone×2（7/8 有
  状态包络）+ 根。`guard: "alive"` 编译为实例 once 门。
- ASCII：字面量与全部 HUD 文本限于 code 32..126、单槽 ≤32 字符（§5）。

## 2. 生成/接线（T-03 生成，V-02/V-03 实测）

- 根合同字段（F-4）：作者 `r_count`/`r_state` + 生成 `r_gate`/`r_ord`/
  `r_np`/`r_p0`/`r_p1`，各恰一次声明，`get_r_<f>()` 观测 getter 齐备。
- 收口骨架（F-3）：生成 `game_cue(c)`（r_ord 递增、p0/p1 待发槽、np≤2）、
  `game_events()`（poll 前奏：`ag_event_next` 首读 + `ag_event_pair` 分派
  ——collect/despawn/count/cue/goal/hazard 全在生成侧）、`hud_*_text(n)`
  分支 helper、`game_closeout()`——**每回调恰一次
  `ag_present2d(kind=-1, r_np, o/c/g×4, v/t×4)`，整体 1 计费**，携带 ≤2
  game cue + 四槽完整状态（t06_v06_a 实证：全通关每 tick 恰一个 bundle，
  第二次收口 = 具名 fault 50）。
- idle-turn 门（F-6）：latch tick 置 `r_gate=1`，同轮根 dispatch
  `player_idle(1)` 令 halt 门跳过两个 held 读（t05 naive 变体实证：不门
  控 = 收集 tick 34 → fault 10；门为闭合关键）。
- world 接线：全部声明 handler 的接收 fn（F-8.2，无 forced dispatch 站
  点——T-02 手扩的 `r_state == 9` 站点生成侧不再出现）；zone 撞名缓存改
  `visq`（F-8.1 修正），present 比较 cache vs model 变更计费；app_update
  固定序 = `game_events()` → 根 lifetick → 声明先序实例 tick/present →
  `game_closeout()`；app_start 固定序 = char bind/solid → camera follow →
  逐实例 spawn（slot 0..4）→ 5 触发绑定。
- scene：`autographics.scene/8`（assets: wav 行含 audio 元数据 + ttf 行；
  cues 数组 id=下标、gain 1.0；hud.font=`sha256:<ttf>`、license 内嵌、
  四槽完整初值）。

## 3. 预算与包络（V-05 实测，F-7 修正后计费表）

32 credits/回调合同不变。生成游戏逐 tick 实测（t05_game_budget）：

| tick 形 | 冻结 | 实测 |
|---|---|---|
| walk（无事件） | 30 | **30 恰**（+2 恰在池内绿、+3 → fault 10 t=1） |
| 收集（单 cue） | 32 | **32 恰**（+1 → fault 10 恰 t=85） |
| 第 3 收集（双 cue 单 bundle） | 32 | **32**（t=207，rows (3,0)(4,1)） |
| goal latch（phase 门） | 32 | **32**（+1 → fault 恰在 win tick 251） |
| hazard fail | 31 | **31**（+1 绿仍 latch、+2 → fault 恰 t=152） |
| pad-stay（count<3） | 32 | **33 修正** → fault 10 恰在首个 pad 重叠 tick
  t=248（coin-bind-stripped 派生变体实测；去 closeout 对照绿、+1 恰同
  tick fault → 第 33 计费即 closeout）。该形在交付关卡**不可达**：coin 3
  在 t=207（x≈14.9）强制 count==3，早于 pad AABB 首次可达 t=248
  （x=17.667），交付游戏永不执行 pad-stay tick；可达 pad 形（goal 32、
  赢后站立 31）全绿。freeze F-7 原行记 32 系与其自身 F-6 组成
  （30+next+pair+bundle=33）不一致，此处以实测修正。 |

- 包络：8 有状态实例（游戏 7/8）、8 对象槽（游戏 5/8；第 9 slot → 具名
  fault 31 于 app_start，t03 实测）、128 场景行（7）、cue ≤8（3）、四槽、
  32 字符 ASCII、bundle ≤4 put。
- VM 守卫独立报告（非门，t05 全通关 238 tick）：max_call_steps 11747
  （guard 100k）、max_call_slices 15（guard ≤120）、calls 1471；CPU 会话
  整 tick 墙钟 **avg 2674 µs / max 3016 µs**，对宿主 8 ms 帧上限 ~2.7×
  余量（状态 getters 走独立调用，不计入）。
- 输入消耗性：`ag_action_pressed` 边沿同 tick 二读为 0（020 冻结沿用）。

## 4. 生命周期（V-04 实测，t05_game_host）

- Start 每 Run 恰一次（12 显式 + ~72 隐式 Resume×dispatch 后首次收集仍
  ordinal 1；重复 Start 将双生入 8 槽池 → fault 31 或 stale bind 无收集
  ——两败形均被排除）；Resume 不重复 Start。
- Pause/Step：paused probe-dispatch = 恰一中性 tick（无输入边沿）；render-
  only 帧不推进；single_step = 一中性 tick，其收口 packet 被
  **SuppressedByStep**；step 消耗无 ordinal（后续收集 ordinal 2，无回放/
  积压）；cue-bearing 抑制半在生成 cue 表上以交付 mixer 机制证明（step
  接受 → 无 voice、ordinal 耗尽、pause/resume 静音）。
- 20 轮 stop/fresh：每轮模型重建（count "0"/"Playing"、ordinal 重启 1、
  coin 复活、文档 hash 恒定、mixer Run 重建、packet run=活 Run id——无
  旧 Run 泄漏）；20 轮后共享合法资源存活（全 palette 复渲）。
- preaccept 坏件（派生 unknown-cue-3 变体）：收口自身 audio 单元在任何
  mixer accept 前具名 fault 53（code=53 node=-1 arg=3）；last-good 保留
  （上一 tick 帧、HUD "0"/"Playing"、无新 voice、设备流健康）；精确还原
  即绿。postaccept 设备 Fault 经交付注入机制（`Mixer::output_fault`——
  cpal 错误回调同钩；真实设备故障无法安全诱导）：具名 `audio_fault: …`
  latch、Run voice 停、此后静音（已提交 PCM 不可撤回、无补播）；下次
  start 释放坏流开替代流即绿。
- native 复观察：真实默认设备开流（ToDesk Virtual Audio，44100/2/F32）
  消费生成链首个收口 packet：970/970 帧非零、ring head 对 resampled
  slice 1e-6 内逐帧相符、0 deadline miss。**D-04 具名 gap（原样）**：观
  察设备为 ToDesk 远程桌面**虚拟**扬声器——观察覆盖提交进已开流的采样
  （post-routing/post-quantization），不覆盖声学输出；声学/物理设备维度
  保持具名 gap，不主张任何硬件输出。
- 失败分支：左行出沿入 hazard AABB → state=2 latch（恰一次）、fail buzz
  ordinal 1、hazard 消耗而 pad 存活（接收器索引可观察，t04 V-03 b）。

## 5. 诊断合同（V-01 + V-06，全部具名，不 panic）

- 降层（module/line/symbol 定位，pilot node_components_game_gates 3 测试
  7 负夹具）：区域规则静态叶目标、未知 cue 名、slot>3、缺 count/state、
  生成基础设施字段被作者声明（gate/ord/np/p0/p1/visq 撞名）、game2d 非
  root、game2d 进冻结档；cue span 越 bank 与 gain 越域以真实 bank 在临
  时目录验证。
- 收口纪律（世界/故障码，t06_game_closeout 7 案实测）：50（同回调第二
  bundle/第二次 present2d/模式冲突）、51（n>4 或第 5 put）、52（ordinal
  重复/回退）、53（cue 越上界/负 id）、54（gain <0/>1；**非有限值不可达
  54**——世界内计算值被 VM 除零陷阱 `call: DivisionByZero` 拒、注入值被
  受信会话调用门 `nonfinite argument` 拒，均为具名边界，实测记录）、56
  （文本 >32 或 code<32/>126，32 绿）、57（bundle 后混写 ag_hud_set/裸写
  后 bundle begin）；fault 10（计费越 32）、31（slot>7）。
- 载入/准予：`behavior source closure identity: <scripts/<sha>.at>`（坏
  件拷贝/byte_length 漂移）、缺 `scripts/<sha>.at` 具名 io 错、
  `hud_invalid: fixed font bytes`、`hud_invalid: font license identity`、
  `audio_invalid: declared hash/length`；错 runner（V1 编译期
  `Undefined symbol: ag_action_pressed`；audio-only 共享面编译过、收口
  运行期调用破 status=38——实测观察形）。
- mixer 侧（交付测试在案）：`audio_invalid: ordinal/cue identity/gain/
  packet identity-domain/bounded packet length`。

## 6. 消费与身份（V-06）

每产品一目录：`scene.agscene`（behavior.source 绑 world sha+byte_length）
+ `world.at` + `identity.json`（generator/api/user_source sha/逐模块 sha/
closure 清单含三资产/roles/**provenance**）+ `closure/<module>.at` +
`closure/assets/`（冷再生唯一输入）。身份链逐环可核（verify_plan023 控制
1-4）：world sha == 场景绑定 == identity == provenance == T-03 钉
`ed98b0b5…`；closure sha == manifest == 夹具源；5 个真实消费方测试文件
（t03_game_gates/t04_game_play/t04_game_media/t05_game_budget/
t05_game_host）各自 include_str! 并钉同一 sha，media/host 以
`scripts/<sha>.at` 暂存（载入路径逐文件哈希核对该名）。
`tools/verify_plan023.py --controls`（10 控制组、1 产品）：身份链、闭包
清单三重符合、provenance 消费绑定、场景媒体合同（bank 行/对界 cue 表/
license 恒等/四槽）、**闭包独占冷 A→B 字节等价**（B 仅收 closure/ →
三文件逐字节相等，B world sha 同钉）、**B 真 CPU 通关 + 保存/重开**
（T06_B_ROOT 驱动 t06_ab：settle 13/coins 85/146/207/win 251 绿，暂存
工程 load_project 两轮独立重开绿）、夹具变异敏感（c1 cx 8→9 改
world/scene/identity，SoA 落地）、V-06 收口纪律 7 案、坏件拒绝→精确还
原→再运行全绿、6 类坏输入具名拒绝（bad source/bad font/bad bank/bad
license/bad record（byte_length 与 sha 两形）/bad runner）。变体 = 声明
入口；任何被用源变化使旧身份失效。

## 7. 验收映射（work-owned）

- AC-01（真实依赖/两接法先证/原额，不动根、无 Rust 代业务、不默认提额）：
  T-02 冻结 + §2/§3（t05_game_budget 5 实测）。
- AC-02（typed 资源/槽、局部 ASCII 表达式/普通 use、严格定位准入）：§1/§5
  （t03_game_gates V-01/V-02 + pilot game gates 负控）。
- AC-03（局部组件事件/计数、真实字形、一次 bundle、独立 PCM 正确）：§2/§4
  （t04_game_play 7、t04_game_media 2：PCM 逐位 0 LSB ≤2e-5、GPU 字形
  viewport 锚定随状态变化）。
- AC-04（生命周期/旧 Run/容量故障/20 轮 owner/真实 native 非零 PCM）：§4
  （t05_game_host 7；D-04 声学 gap 保持具名）。
- AC-05（原费用/CPU/资源与完整最坏分支/warm/负控）：§3（t05_game_budget 5
  + VM 包络独立报告 + F-7 pad-stay 修正行）。
- AC-06（公共准备/实际身份、离线 A→B/保存重开/坏件恢复、旧模式兼容）：§6
  （t06_game_closeout 10 + verify_plan023 10 控制组；旧消费者复跑全绿，
  旧入口字节冻结未动）。

## 8. provenance

本候选由 auto-ui PLAN-023 T-06 建立：事实行为沿 T-02 冻结合同（Plan §9
F-1..F-9 + audio 事实）与 T-03..T-06 实测（pilot gates
9 basic + 10 g11 + 9 lower + 6 components + 7 queries + 3 game；
t03_game_gates 4、t04_game_play 7、t04_game_media 2、t05_game_budget 5、
t05_game_host 7、t06_game_closeout 10；verify_plan023 --controls 10 控制
组全绿；game world sha
ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0 自 T-03
交付未变）。016/017/019/020/021 既有 spec 与锁保持原状，不在本文件重述。
merge 阶段本稿落位 docs/specs/auto-graphics/node2d-audio-and-hud.md。
