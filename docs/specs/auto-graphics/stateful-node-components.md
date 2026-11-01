# AutoGraphics 有状态 Node 组件作者入口（auto-ui PLAN-020 SD-01 候选）

Status: candidate（auto-ui PLAN-020 T-03..T-06 实测交付；本稿为
docs/specs/auto-graphics/stateful-node-components.md 的 SD-01 候选文本，
merge 阶段按收据落位。只描述实际交付并验证的能力，不声称任何未实现的
语言/运行时特性。）

## 0. 范围

`node2d-lower --profile components-2d` 接受普通 Auto 模块工程，把**基础二维
的"根逻辑 + 子纯 view"限制升级为显式有状态组件模式**：普通 `use` 导入的
组件定义可内聚 `model`/`msg`/`on`（含 `Life.Start`/`Life.Tick` 与匿名
`.Ev` 业务事件），view-2d 内可嵌套实例化其它定义（同定义多实例 = 独立
SoA 槽位），根节点同样可有状态（model 字段为 `r_<field>` 标量）。生成
`autographics.scene/6` 工程（`behavior.api = autographics.objects2d/1`）
与真实用户 Auto 源，经 T-02 冻结的两套既有机制执行：每定义 SoA 数组 +
按实例索引 handler，下降期绑定表同步直呼（无宿主队列）。八条产品链的执
行证据（只述实测）：**离屏 GPU 呈现**（宿主 play 路径、像素类与 slot 观
测）覆盖 probe（走跳/双 Gauge/隐藏/事件计数）、lifecycle（Pause/Step/
20 轮 fresh、Start 事件计数驱动位置）与 init-fault（宿主启动路径，故障
在初始化浮出，无通过帧）；**CPU 会话**（受信 VM、状态 getters 逐值断
言）覆盖 budget（32 计费 ×120 tick 墙钟实测）、budget-plus（fault 10）、
envelope（8 包络）、lifecycle-doublepress（消耗性边沿）与 helpers（同名
helper 作用域，pilot gates 降层门）；V-01 降层诊断与 V-06 消费/身份对全
部八链成立。

非范围（不冒称支持）：动态 Node 拓扑/key 重建/HMR、动态 solid、多角色、
查询/PNG/精灵/3D/声音/HUD、`Life.Frame`、trigger2d/group2d/if（具名拒
绝）、ACE/AOT/ECS。**旧入口不变**：016 legacy、017 g11-2d、019
scene-2d/character-2d/camera-2d 字节冻结（verify_plan016/017/019 再生控
制），不被自动升级；旧消费者（node2d_basic_flow 13、node2d_flow 6、
node2d_g11_flow 15）在 T-06 复跑全绿。

## 1. 声明面（T-03 冻结拼写）

- 组件定义：普通 `node Name(p: type, ...) { model?/msg?/view-2d/on? }`。
  参数四类 typed 端口（int/float/Material2D/Mesh2D）；**每个声明参数必须
  由实例提供**（数值字面量 / `resources.X` / 父参数引用 `.p` 或裸 `p`），
  未知 prop 具名拒绝。**参数只读（降层强制）**：`.p` 作为 handler 赋值目
  标具名拒绝（`params are read-only: '.p' cannot be assigned …`，module/
  line/symbol 形态；neg-param-write 夹具复放）——model 字段赋值不受影响
  （全部正面链的 `.ticks/.acc/.phase/...` 写入即证）；赋值右值的参数读取
  合法（`.draw_x = .phase0` 正面 pin）。模型初值 = 数值字
  面量或 `.param`，按实例在生成期解析内联进 SoA 数组字面量（每实例建立
  一次；动态输入属性更新不重置 model）。
- `model`：数值字段（int/float 按初值字面量形态）；`.field` 前导点即本
  实例成员（生成 `<prefix>_<field>[i]`；根为 `r_<field>` 标量）。同定义
  不同父路径的状态互不混淆（a/meter 与 b/meter 是 `gauge_*[0]` 与
  `gauge_*[1]`）。
- `msg { Add(int) }`：匿名业务事件，载荷数值类型（int/float）。
- `on`：`Life.Start` / `Life.Tick(dt)`（系统）与 `.Ev(p..)`（成员/业务）
  混合；业务事件名不得与保留 Life 名冲突（lifestart/lifetick 拒绝）。
- 实例化：view-2d 内 `NodeName instance_name { props }`，可嵌套（路径
  `a/meter`）；同 view 元素名唯一；重复实例路径具名拒绝。
- 事件发送：子定向 = **选择器调用** `meter.Add(.payload)`（Core 体文法接
  受选择器调用；`emit` 前缀仅用于前导点形态）；祖先定向 = `emit .Ev(args)`
  投递最近声明 `on .Ev` 的祖先实例（或根 handler，直呼无索引）。下降期
  解析：端口未声明、无 handler、发射/载荷/形参元数不一致均具名拒绝。
  同名子节点在两父实例下分别投递（绑定表逐发射实例列出接收下标）。
- view-2d：`shape2d`（`kind: circle|quad` + size，或 `mesh: <Mesh2D>` 资
  源/参数）；任一 prop 绑定本定义 model 字段（`.field`）即**动态**——
  每实例一个模板行（停靠玩法区外 `(-30, -40-2*idx)`、visible=false 保身
  份）+ app_start 运行时 `ag_object_spawn(slot, template, x, y, 0)` +
  每 tick 末 `<prefix>_present(i)` 同步（位置/可见按变更计费：平方差门 +
  int `!=` 门——钉死 VM 的"比较结果求和再关系化"与 float 元素相等比较
  均实测不可靠，生成门只用原始比较/纯算术）。完全静态 shape 按实例克隆
  为普通场景行（`instance.element` 命名）。`character2d` 仅根 view 或
  **恰一实例**的定义（character 服务 one-per-Run）；`solid2d`/`camera2d`
  仅根 view；`camera2d` 必须恰一（Objects2V1 入口冻结相机态，缺相机
  fault 27），target 须指向角色实例。
- 池化纯函数：作者 `fn` 按定义模块作用域池化为 `<module>_<fname>`（调用
  点按所在模块解析；两模块同名 helper 各自解析各自的）。handler 内裸调
  用不属本定义模块即具名拒绝（unknown helper fn in module scope）。

## 2. 两套接线（T-02 冻结，T-03/T-04 生成）

- 接线一（状态）：每定义 SoA 数组（model 字段、运行期参数数组
  `<prefix>_<param>[i]`、handle/px/py/vis 缓存）；`<prefix>_lifetick(i,
  dt)` / `<prefix>_lifestart(i)`（根为 `main_*` 无索引）；app_update 固
  定序 = 根 `main_lifetick(dt)` → 声明先序每实例 `tick` → `present`。
  Getters：`get_r_<f>`（根）、`get_<prefix>_<f>_<idx>` +
  `get_<路径下划线>_<f>`（每实例每 model 字段，供测试观测全部状态）。
- 接线二（事件）：绑定表 `var <child>_bind_<EV>=[接收下标…]`（发射实例
  i → 其名下子实例的 SoA 下标；祖先投递为 `<prefix>_emit_<EV>`；root 接
  收方/发射方直呼无表无索引）；`on .Ev(p)` → `fn <name>_<ev>(j int, p..)`
  同步直呼，零宿主队列，同 tick 内联送达。
- app_start 固定序：`ag_char_bind` → `ag_char_solid`（声明序）→
  `ag_camera_follow_2d` → 逐实例 spawn（slot 0 起连续）→ `Life.Start`
  们（根先、声明先序）。

## 3. 预算与包络（V-05 实测）

- **8 有状态实例**（含根）：第 9 个具名拒绝（8-instance envelope）。
- **8 运行时对象槽**（objects2d 冻结 SLOTS=8）：第 9 个 `ag_object_spawn`
  slot>7 → 具名 fault 31（init-fault 链实测；降层仍绿——这是运行时合
  同，不是降层拒绝）。
- **32 credits/世界 callback**（`_ag_spatial_begin` 每 app_update 重置，
  不可每 Node reset）：走跳 tick 30（char move 26 = 1+24+1、pressed 1、
  held 2、grounded 1）+ 每实例呈现写 1（ag_object_move/visible/spawn 各
  1）。budget 链每 tick 恰 32 全绿 120 tick；budget-plus（第 33 计费）
  在预测 tick 2 具名 fault 10。`input.pressed` 消耗性边沿：同 tick 第二
  次读取返回 0、计费 1（doublepress 链实测 p1=1/p2=0、单次起跳）。
- VM 守卫保持原额：1000 steps/slice、2ms 整 tick、callback 100000
  instructions/120 resumes/250ms（实测 max_call_steps ≤ 7808、
  max_slices ≤ 8 @120 tick——独立报告，不降样本）。
- 8ms 口径（实测）：budget 链 CPU 会话整 tick 墙钟 **avg 1761 us /
  max 3129 us**（120 tick，含三次宿主会话调用开销），对宿主 8ms
  （=8000 us）帧上限保有 ≥2.5× 余量；该数字是 CPU 会话整 tick 成本（状
  态 getters 走独立调用，不计入），与宿主帧上限不同口径、独立报告。

## 4. 生命周期（V-04 实测）

- `Life.Start` 每 Run 恰一次（Start 时事件同 tick 内联送达——panel Start
  `meter.Add(5)` 实测进入 count）；**Resume 不重复 Start**（跨多次
  resume 后 Start 计数仍为 1）。
- `Life.Tick` 与派发 tick 严格一一对应；暂停下的 probe 派发走中性输入
  （边沿不积压，019 冻结的 host 模式）；render-only 帧不推进任何 model；
  `single_step` 恰一个中性 tick。
- Stop/fresh：20 轮宿主驱动 stop→start 全部重建 model（计数器复位、参
  数/Start 事件重放、无旧 Run 泄漏）；宿主文档不变；共享合法资源全程存
  活（槽模板身份与渲染类别 20 轮后仍在——harness 可观测范围）。
- 初始化失败（fault 31 实测）：具名错误浮出（`error code=31 … argument=8
  APIError @ app_start`）；宿主保留 last-good（无半初始化 Run 发布为
  好）；还原后同工程再启动即绿。

## 5. 诊断合同（V-01）

非法图/引用在执行前具名拒绝，消息含 `module <m>, line <n>, symbol '<s>'`：
缺 use（载入层）、错参数元数/未知 prop、未知事件端口、发射/载荷/形参元
数不一致、未知成员引用、未知 helper fn（模块作用域）、实例环
（`instance cycle: A -> B -> A`）、重复实例路径、第 9 有状态实例、缺/
多相机、character 归属违规、solid/camera 位置违规、坏 kind/尺寸/资源、
重名元素、>128 场景节点。不 panic（负面集 tools/verify_plan020.py 与
pilot gates 双重复放）。

## 6. 消费与身份（V-06）

每产品一目录：`scene.agscene`（behavior.source 绑定 world sha256 与
byte_length）+ `world.at` + `identity.json`（generator/api/user_source
sha/逐模块 sha/closure 清单/roles/**provenance**）+ `closure/<module>.at`
（冷再生唯一输入）。身份链逐环可核：closure sha == manifest == 夹具源；
world sha == identity == 场景绑定；场景对象名唯一（名字即身份）。
**provenance**（T-06 新增，事实性）：记录消费方测试文件
（node_components_flow.rs / helpers 为 pilot gates）、include_str! 消费
方式、`scripts/<sha>.at` 喂入路径与载入核验机制（实测观察：坏件以
`behavior source closure identity: <scripts/<sha>.at>` 拒绝）。
`tools/verify_plan020.py --controls`（16 控制组、8 产品）：身份链、闭包
独占冷 A→B 字节等价、夹具变异敏感（非静默同字节）、6 条负面具名复放、
坏件拒绝→精确还原→再运行全绿（经真实驱动控制
`node_components_flow::v06_damaged_world_copy_rejected_then_recovery_green`
演示）。变体 = 声明入口；任何被用源变化使旧身份失效。

## 7. 验收映射

- AC-01（基线/两接法/原额先证）：T-02 冻结 + 本 spec §2/§3。
- AC-02（同类型多路径、局部状态/typed 引用/诊断）：§1/§5（V-01 gates +
  verifier 负面复放）。
- AC-03（局部走跳、双实例 GPU、typed 事件、参数绑定）：§1/§2（V-02/03
  flow 探针链 + budget 家族）。
- AC-04（Life/输入/Pause/Step/Stop/fresh、初始化失败、20 轮 owner、旧
  Run 隔离）：§4（V-04）。
- AC-05（原额/输入消耗/VM 守卫）：§3（V-05）。
- AC-06（公共消费合同、A→B/坏件恢复、旧消费者兼容）：§6（V-06，
  verify_plan020 + provenance + 旧入口字节冻结）。

## 8. provenance

本候选由 auto-ui PLAN-020 T-06 建立：事实行为沿 T-02 冻结合同（Plan §9）
与 T-03..T-06 实测（pilot gates 5、node_components_flow 9、
verify_plan020 --controls 16 控制组全绿；probe world sha
9241d0132f469b605f5d488650ebb40e86af34e79798feb615fc47599d76060b 未变）。
016/017/019 既有 spec 与锁保持原状，不在本文件重述。
