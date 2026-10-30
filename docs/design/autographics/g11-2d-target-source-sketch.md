# G11/2d 目标源码示例：node / view2d / Life / msg

2026-10-30，讨论稿 v2，取代同路径v1草图；v1保留在Git历史。依据[共同作者模型](declarative-authoring-model.md)和用户本轮命名决定。只写Markdown，不创建或运行产品源码，不分配Plan或更改在途合同。

## 1. 已确认写法与仍属提案的部分

用户已选择：以node统一组件定义，widget经明确迁移后退役；view/view2d/view3d分别声明UI/2D/3D内容；view2d内不再套scene2d；model、呈现和on默认内聚同一Node；Life作为系统事件命名空间；msg { Collect(Key) }直接声明本Node事件；前导点引用当前实例成员。use用于复用真正的组件、资源与函数，不把关卡view另外拆成需重复映射全部model的MainScene。

以下源码是**目标表示，当前不能直接编译运行**。资源构造器、节点名/refs/公开端口、输入服务、集合API、emit和key重建仍为语义占位。msg属性的类型化签名也尚待收敛。Life.Update是否按呈现帧、Life.FixedUpdate按固定逻辑步长的具体调度/顺序未最终冻结；本例用FixedUpdate明确运动时钟，不混入渲染回调。

规则参照[原G11/2d](../../../../ag-examples/examples/g11-runtime-objects/2d/main.at)：左右运动、三个不同外观物品一次收集、独立巡逻危险物、集齐才通过的终点、左侧界外失败、终局Enter按下沿重开。无跳跃/新增声音/HUD验收。去除原预算导致的漏处理/跳巡逻技巧，故不是原逐tick轨迹等价声明。

```text
src/
  app.at                  应用/新局边界
  resources.at            不可变资源描述和具名symbol
  game_types.at           玩法类型
  g11_round.at            一局：model + view2d + on
  components/player.at    角色组件：model + view2d + on
  components/coin.at      可复用拾取触发组件
  components/hazard.at    巡逻/失败触发组件
  logic/patrol.at         可复用纯函数
```

文件数量只是示意；组件可合写或拆文件，语义不因文件布局改变。以下只列普通use，不引入use.ag。

## 2. 资源与类型

`resources.at`：沿用程序化图元，避免虚构新素材文件。共享描述由资源owner准备，use不隐式创建GPU设备。

```auto
use auto.graphics: mesh2d, material2d

let player_mesh = mesh2d.quad(width: 0.8, height: 1.0)
let ground_mesh = mesh2d.quad(width: 40.0, height: 1.0)
let amber_mesh = mesh2d.circle(diameter: 0.6)
let azure_mesh = mesh2d.quad(width: 0.7, height: 0.7)
let rose_mesh = mesh2d.circle(diameter: 0.9)
let hazard_mesh = mesh2d.circle(diameter: 0.7)
let goal_mesh = mesh2d.quad(width: 0.5, height: 1.2)
let void_mesh = mesh2d.quad(width: 2.0, height: 3.0)

let orange = material2d.color(0.9, 0.5, 0.25, 1.0)
let ground_color = material2d.color(0.55, 0.62, 0.7, 1.0)
let gold = material2d.color(0.95, 0.8, 0.2, 1.0)
let cyan = material2d.color(0.2, 0.8, 0.9, 1.0)
let pink = material2d.color(0.9, 0.35, 0.85, 1.0)
let red = material2d.color(0.9, 0.2, 0.2, 1.0)
let green = material2d.color(0.3, 0.9, 0.5, 1.0)
let dim = material2d.color(0.1, 0.1, 0.12, 1.0)
```

未来PNG/GLB导出类型化资源symbol，构建/加载阶段解析成资源键。资源symbol、显示名、内容hash与运行句柄分离；不每帧用字符串找对象。资源/常量表达式的求值范围由公共语言/领域合同约束，不将资源声明当任意顶层I/O。

`game_types.at`：

```auto
enum Key { Amber, Azure, Rose }
enum Result { Playing, Won, Failed }
```

Key只表示玩法中哪枚物品，不是手写引擎slot。实际组件/物理对象仍有独立实例身份。

## 3. 关卡：状态、2D声明、规则保持局部性

`g11_round.at`：

```auto
use auto.graphics: solid2d, trigger2d, camera2d
use resources
use game_types: Key, Result
use components.player: Player
use components.coin: Coin
use components.hazard: Hazard

node G11Round(on_finished: msg) {
    model {
        var remaining = set(Key.Amber, Key.Azure, Key.Rose)
        var result = Result.Playing
    }

    msg { Collect(Key), HitDanger, ReachGoal }

    view2d {
        solid2d ground {
            position: (15.0, -0.6)
            mesh: resources.ground_mesh
            material: resources.ground_color
        }

        Player player {
            spawn: (2.0, 1.0)
            speed: 2.0
            enabled: .result == Result.Playing
        }

        if .remaining.contains(Key.Amber) {
            Coin amber {
                position: (8.0, 1.0)
                mesh: resources.amber_mesh
                material: resources.gold
                target: refs.player.actor
                enabled: .result == Result.Playing
                on_collect: .Collect(Key.Amber)
            }
        }
        if .remaining.contains(Key.Azure) {
            Coin azure {
                position: (10.0, 1.0)
                mesh: resources.azure_mesh
                material: resources.cyan
                target: refs.player.actor
                enabled: .result == Result.Playing
                on_collect: .Collect(Key.Azure)
            }
        }
        if .remaining.contains(Key.Rose) {
            Coin rose {
                position: (12.0, 1.0)
                mesh: resources.rose_mesh
                material: resources.pink
                target: refs.player.actor
                enabled: .result == Result.Playing
                on_collect: .Collect(Key.Rose)
            }
        }

        Hazard hazard {
            x: 9.0
            base_y: 2.4
            target: refs.player.actor
            on_hit: .HitDanger
        }

        trigger2d goal {
            position: (16.0, 1.0)
            mesh: resources.goal_mesh
            material: resources.green
            on_enter: (actor) => {
                if actor == refs.player.actor { emit .ReachGoal }
            }
        }
        trigger2d void_zone {
            position: (-1.0, 1.0)
            mesh: resources.void_mesh
            material: resources.dim
            on_enter: (actor) => {
                if actor == refs.player.actor { emit .HitDanger }
            }
        }

        camera2d camera {
            target: refs.player.actor
            bounds: (-4.0, 0.0, 30.0, 12.0)
            view_size: (12.0, 8.0)
        }
    }

    on {
        .Collect(key) -> {
            if .result == Result.Playing && .remaining.contains(key) {
                .remaining.remove(key)
            }
        }
        .HitDanger -> {
            if .result == Result.Playing {
                .result = Result.Failed
                emit .on_finished(Result.Failed)
            }
        }
        .ReachGoal -> {
            if .result == Result.Playing && .remaining.is_empty() {
                .result = Result.Won
                emit .on_finished(Result.Won)
            }
        }
    }
}
```

model和view2d同属当前实例，.remaining/.result直接局部引用，不经MainScene参数再次转发。计数派生为3 - remaining.count()；条件分支消失对应真实销毁Coin的trigger/实体，不是隐藏。其他分支保持实例身份，不每tick全量重建。

props/事件传递只出现在真实组件边界。on_collect绑定当前G11Round实例与Key；三枚Coin共享定义但不是同一个状态实例。on_finished: msg只是占位写法，正式端口须约束payload为Result。重复Collect因remaining守卫不重复计数；失败/胜利后请求忽略。

## 4. Coin：事件绑定而非全局回调

`components/coin.at`：

```auto
use auto.graphics: ActorRef2D, Mesh2D, Material2D, trigger2d

node Coin(position: float2, mesh: Mesh2D, material: Material2D,
          target: ActorRef2D, enabled: bool, on_collect: msg) {
    msg { Touched(ActorRef2D) }

    view2d {
        trigger2d body {
            position: .position
            mesh: .mesh
            material: .material
            enabled: .enabled
            on_enter: .Touched
        }
    }

    on {
        .Touched(actor) -> {
            if .enabled && actor == .target { emit .on_collect }
        }
    }
}
```

Coin不维护重复collected/bag状态，也不直接改父model。根的remaining是物品存在与计数的唯一依据。emit代表有接收者/参数的事件分派，不是广播；具体关键字未定。不能单凭这段表示就承诺跨VM原子回滚。

## 5. Player：Life系统事件直接进入on

`components/player.at`：

```auto
use auto.graphics: character2d, ActorRef2D, input
use resources: player_mesh, orange

node Player(spawn: float2, speed: float, enabled: bool) {
    model { var move_axis float = 0.0 }

    export { actor: ActorRef2D = refs.body }

    view2d {
        character2d body {
            initial_position: .spawn
            mesh: player_mesh
            material: orange
        }
    }

    on {
        Life.Start -> {
            .move_axis = 0.0
        }
        Life.FixedUpdate(dt) -> {
            .move_axis = 0.0
            if .enabled { .move_axis = input.axis(Left, Right) }
            refs.body.move_and_slide((.move_axis * .speed * dt, 0.0))
        }
    }
}
```

Life.*是系统事件，不在msg里声明、不经fixed_update/lifecycle辅助节点转发。Left/Right是本应用具名动作，正式动作配置语法待定。input读取世界在本tick采样的同一快照，不让各Node分别消费原生按键。

initial_position只在新实例初始化生效，不把spawn持续写回运动位置；物理body保存真实位置，camera/trigger读取它。character与solid的默认碰撞匹配只是本例所需支持域，复杂层/掩码后议。refs/export仍是类型化公开actor端口的候选表示，不能访问任意子私有字段。

Life.Start不同于Mounted：编辑器可已挂载但未进入Run。Start在必要引用/资源就绪后、首次FixedUpdate前一次；Pause不推进，Step只推进一次中性输入逻辑tick，多Portal不重复更新。动态创建、重挂载、HMR及Update/FixedUpdate的正式顺序须另冻，不凭命名假定已完成。

## 6. Hazard与纯逻辑复用

`logic/patrol.at`：

```auto
fn patrol_y(base float, phase float, extent float) float {
    if phase <= extent { return base + phase }
    if phase <= 3.0 * extent { return base + 2.0 * extent - phase }
    return base + phase - 4.0 * extent
}
```

`components/hazard.at`：

```auto
use auto.graphics: trigger2d, ActorRef2D
use resources: hazard_mesh, red
use logic.patrol: patrol_y

node Hazard(x: float, base_y: float, target: ActorRef2D, on_hit: msg) {
    model { var phase float = 0.0 }
    msg { Touched(ActorRef2D) }

    view2d {
        trigger2d body {
            position: (.x, patrol_y(.base_y, .phase, 0.2))
            shape: circle(radius: 0.5)
            mesh: hazard_mesh
            material: red
            on_enter: .Touched
        }
    }

    on {
        Life.FixedUpdate(dt) -> { .phase = (.phase + 3.0 * dt) % 0.8 }
        .Touched(actor) -> {
            if actor == .target { emit .on_hit }
        }
    }
}
```

巡逻按时间步独立执行，不因收集事件跳过。三角波保名义范围/速度，但不同于原来先越界再翻转的轨迹；只保玩法意图，不冒认逐tick一致。main-alt未来改base_y/速度参数验证真实结果敏感性，不复制整个逻辑文件，也不只看参数不同就判通过。

## 7. App：新局重建实例子树

`app.at`：为避免纯组合的跨领域包装，本例根也使用view2d；完整UI/2D/3D混合呈现端口另行定义。

```auto
use game_types: Result
use g11_round: G11Round
use auto.graphics: input

node App {
    model {
        var round_key int = 0
        var last_result = Result.Playing
    }
    msg { Finished(Result), Restart }

    view2d {
        G11Round current_round {
            key: .round_key
            on_finished: .Finished
        }
    }

    on {
        Life.FixedUpdate(dt) -> {
            if input.pressed(Enter) { emit .Restart }
        }
        .Finished(result) -> { .last_result = result }
        .Restart -> {
            if .last_result != Result.Playing {
                .round_key += 1
                .last_result = Result.Playing
            }
        }
    }
}
```

input.pressed读取本tick一致的边沿事实，不因某个Node先读就吃掉其他订阅者；新局不重放旧沿，held不连续重开。key重建当前局的model/物理body/订阅，保留App和合法共享资源。迟到旧代事件拒绝；key溢出不静默回绕。创建新实例不意味着重启整个VM。

## 8. 如何实现：源码触点与实际工作量

只读观察auto-ui基面cf77a99及AG固定SDK路径；不是已经运行的可行性探针。现有基础与目标差额：

| 触面 | 现有证据 | 实施判断 |
|---|---|---|
| 匿名msg与前导点 | [declarations.rs](../../../crates/auto-ui-dialect/src/declarations.rs)的parse_msg_decl已接受msg { ... }，旧命名块为兼容；[widget.rs](../../../crates/auto-ui-dialect/src/grammar/widget.rs)已解析点前缀带参handler | 主要是复用/明确约定，不必当作全新功能 |
| node与view域 | widget解析入口固定widget；[AST](../../../crates/auto-ui-dialect/src/ast.rs)为WidgetDecl、单view及named_views | node可先映射共同组件表示，补view域而非复制三套AST；named view不是view2d/3d，须区分。最终退役widget需另核旧读者/兼容 |
| Life系统事件 | [Vue生成器](../../../crates/auto-ui-compiler/src/ui_gen/vue.rs)已有Init/Destroy到挂载钩子的接线；[VM桥](../../../crates/auto-ui-runtime/src/ui/vm_bridge.rs)有Init demand/代次/任务泵 | 可复用机制，但Life命名空间、Start/FixedUpdate调度、目标实例/代次不能只靠字符串改名；需区分系统事件与用户同名事件 |
| 实例状态与事件 | VM桥存在child实例支持，也有将子model字段合入root的merged兼容路径；[Design47](../../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md)已明确目标差额 | 不能宣称所有组件已完全隔离；必须用两同类型实例验证receiver/model/订阅，不批量修无关UI |
| 模块到AG执行 | [AG准入器](../../../experimental/autographics-auto-driver/src/authoring/behavior.rs)拒绝用户import；[Session](../../../experimental/autographics-auto-driver/src/lib.rs)直接编译组合源码并取root签名 | 不是删掉一条禁止即可：要接模块解析/依赖快照/命名空间、领域lowering与实例handler；当前AG固定SDK与生产UI编译链不同，不能盲接当前语言tip |
| 图形运行 | 已有角色、对象/trigger、相机、资源/GPU、运行输入与寿命处理 | 大部分图形算法可复用；新增组件实例↔运行对象映射、条件差量、具名ref、事件绑定和新局子树管理。不能让bridge用Rust代替Auto玩法 |
| ECS | 本草图不依赖ECS | 简单实例表和已有对象存储即可；优化后置，不强制一模块一VM |

建议实现路径（只是工作分解，不取编号或启动）：

1. **语法与语义纵向探针**：node/view2d/Life/既有msg，模块导入、实例receiver和诊断；先一个Player、两个Coin/两个有状态Hazard，证明同定义不同实例不串状态。纯解析通过不算闭环。
2. **共同领域模型到AG adapter**：具名节点/ref和资源描述lowering到现有图形对象，接世界时钟/输入快照/触发到事件；事件、model、运动仍由Auto执行。使用生产共同模型端口，不把旧单文件代码拼接包装成正式模块。
3. **条件实例与重开**：remaining变化只移除对应trigger，key变化重建一局；绑定/资源代次、Stop/Pause/Step/旧事件隔离有定向正反例。跨VM事务/全世界回滚不作为前置。
4. **内容仓真实消费**：auto-ui能力实际交付后，ag-examples独立后继写真实.at/资源和新锁；保原玩法分母，显式说明新旧调度/轨迹差额，再决定旧入口何时退役。源码只变漂亮、暗中Rust补逻辑不算完成。

进入门：沿既有ACE/统一领域模型成熟度重核实际端口，或另行明确批准有限研究adapter；文档示例不自动放行原先延后融合的工作。不能先在旧SDK造第二套永久Node编译器，再称已接共同体系。

**工作量结论：语法层可以较小；完整八模块示例涉及中等范围的编译/执行接线，不能承诺只是几处改名。** 不必重写renderer/碰撞/相机，也不必先做ECS/完整CPC或全量AutoUI迁移；但实例状态、事件目标、依赖模块、条件销毁与生命周期都必须真实成立。尚无探针/测量，不能据代码行数估天数或宣称性能提升。

## 9. 仍需收敛的语义与本轮验证

待决点：refs与公开端口语法、事件payload类型、emit复用现有分派的写法、输入动作配置、资源声明/准备边界、条件节点身份/key规则；多个view域是否允许同Node同时声明及跨域呈现接口，不在此示例中偷定。

候选顺序：输入快照→FixedUpdate处理→位置/变换同步→触发产生与有序投递→model/on→声明增量与呈现准备。旧G11的N→N+1事件消费不被本文改变；同tickCollect/HitDanger/ReachGoal的顺序、终局优先级与故障策略须在实施合同明确，不能靠源码树遍历顺序暗定。Start前必要引用就绪、初始化依赖顺序和新局旧输入清除也需正反验证。

本轮只核对既有语法/源码触点、原G11规则意图、模块引用/命名一致、Markdown链接/围栏/限定diff。未运行编译、项目测试、GPU或窗口，未建立实现树、改变预算/SDK、现有示例/引擎锁或其他owner的在途合同。
