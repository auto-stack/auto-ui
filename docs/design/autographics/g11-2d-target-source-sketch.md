# G11/2d 目标源码草图：model / view / on 与普通 use

2026-10-30，讨论稿 v1。依据[作者模型与要求](declarative-authoring-model.md)，本稿只在 Markdown 内展示假想源码，不创建任何 `.at`、资源、场景或实施 Plan。当前 AG 不支持这些完整声明；不表示旧锁、SDK、Gallery 或编译器已接受。本稿不冻结关键词/签名，也不取代在途合同。

## 1. 阅读约定与示例范围

保留[G11/2d原示例](../../../../ag-examples/examples/g11-runtime-objects/2d/main.at)的规则意图：左右移动、三个不同外观物品各收集一次、巡逻危险物、集齐才通过的终点、左侧界外失败区、终局按Enter按下沿开新局。没有跳跃、敌人AI、声音或正式HUD新增验收。

草图借用AutoUI已有的widget/model/view/msg/on与use组织习惯。**以下图形节点/属性、命名节点写法、refs导出、emit、资源构造、输入事件、key重建和集合API均为候选表示**，不声称今天可编译。`float2`、ActorRef2D、Mesh2D等也是所需语义的占位类型名。每段代码的文件名仅示意未来目录，不是本轮落盘源码。

`widget`暂作共同组件定义名，`scene2d`只是view内的图形节点种类；不先增加一套scene类、scene专用模块系统或use.ag。点前缀指组件自身成员，`refs.xxx`明确指当前实例具名节点，普通局部变量/导入符号按词法作用域解析。

```text
src/
  app.at                   应用会话/重开边界
  resources.at             具名资源定义，共享且不存玩法状态
  game_types.at            CoinKey、Result
  g11_round.at             一局的组合、规则与事件绑定
  components/player.at     输入与角色运动；导出类型化actor引用
  components/coin.at       触发→收集请求，不掌管全局计数
  components/hazard.at     独立巡逻与触发→失败请求
  logic/patrol.at          可复用纯运动计算
```

这是一种职责拆分，不是强制“一组件一文件”。小型原型可以合写；逻辑依赖和实例语义不能因拆合文件改变。Ground/Goal/VoidZone暂直接写在关卡view，复用出现后才抽组件。

## 2. 具名资源与类型

`resources.at`：先用与原例对应的程序化图元，不虚构图片文件或资产hash。将来导入PNG/GLB仍导出同类类型化资源符号，路径/来源由资源模块和构建清单管理。

```auto
use auto.graphics: mesh2d, material2d

let player_mesh = mesh2d.quad(width: 0.8, height: 1.0)
let ground_mesh = mesh2d.quad(width: 40.0, height: 1.0)
let small_coin_mesh = mesh2d.circle(diameter: 0.6)
let square_coin_mesh = mesh2d.quad(width: 0.7, height: 0.7)
let large_coin_mesh = mesh2d.circle(diameter: 0.9)
let hazard_mesh = mesh2d.circle(diameter: 0.7)
let goal_mesh = mesh2d.quad(width: 0.5, height: 1.2)
let void_mesh = mesh2d.quad(width: 2.0, height: 3.0)

let player_material = material2d.color(0.9, 0.5, 0.25, 1.0)
let ground_material = material2d.color(0.55, 0.62, 0.7, 1.0)
let gold = material2d.color(0.95, 0.8, 0.2, 1.0)
let cyan = material2d.color(0.2, 0.8, 0.9, 1.0)
let pink = material2d.color(0.9, 0.35, 0.85, 1.0)
let red = material2d.color(0.9, 0.2, 0.2, 1.0)
let green = material2d.color(0.3, 0.9, 0.5, 1.0)
let dim = material2d.color(0.1, 0.1, 0.12, 1.0)
```

这些let表示不可变资源描述，实际GPU创建由资源owner准备/驻留，不在use求值时隐式打开设备。多个组件使用gold共享资源，不共享model。模块顶层资源声明的具体求值/打包语义待设计，不沿用旧profile“只允许纯常量”的限制来否决新模型。

`game_types.at`：

```auto
enum CoinKey { Amber, Azure, Rose }
enum Result { Playing, Won, Failed }
```

CoinKey是玩法中的稳定语义键，不是引擎对象槽/资源ID。组件引用、资源symbol和运行实例句柄各有类型，不能相互冒用。

## 3. 一局的根组件：规则、场景、事件放在一起

`g11_round.at`：

```auto
use auto.graphics: scene2d, solid2d, trigger2d, camera2d
use resources
use game_types: CoinKey, Result
use components.player: Player
use components.coin: Coin
use components.hazard: Hazard

widget G11Round(on_finished: msg) {
    model {
        var remaining = set(CoinKey.Amber, CoinKey.Azure, CoinKey.Rose)
        var result = Result.Playing
    }

    msg { Collect(CoinKey), HitDanger, ReachGoal }

    view {
        scene2d {
            solid2d ground {
                position: (15.0, -0.6)
                mesh: resources.ground_mesh
                material: resources.ground_material
            }

            Player player {
                spawn: (2.0, 1.0)
                speed: 2.0
                enabled: .result == Result.Playing
            }

            if .remaining.contains(CoinKey.Amber) {
                Coin amber {
                    position: (8.0, 1.0)
                    mesh: resources.small_coin_mesh
                    material: resources.gold
                    target: refs.player.actor
                    enabled: .result == Result.Playing
                    on_collect: .Collect(CoinKey.Amber)
                }
            }
            if .remaining.contains(CoinKey.Azure) {
                Coin azure {
                    position: (10.0, 1.0)
                    mesh: resources.square_coin_mesh
                    material: resources.cyan
                    target: refs.player.actor
                    enabled: .result == Result.Playing
                    on_collect: .Collect(CoinKey.Azure)
                }
            }
            if .remaining.contains(CoinKey.Rose) {
                Coin rose {
                    position: (12.0, 1.0)
                    mesh: resources.large_coin_mesh
                    material: resources.pink
                    target: refs.player.actor
                    enabled: .result == Result.Playing
                    on_collect: .Collect(CoinKey.Rose)
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

`on_finished: msg`沿用AutoUI事件参数外观；正式语义须推导或显式约束为接收Result的事件端口，不是任意消息或字符串广播。节点名amber/player是实例局部绑定，不是全局字符串查找。

计数从`3 - remaining.count()`派生；金币是否存在也只依赖remaining，不再同时维护bag、collected、alive三套事实。Collect重复投递不重复计数；条件分支退出必须真正销毁相应运行节点/trigger并解绑，**不是visible=false，也不是每帧销毁重建所有节点**。存活分支的命名节点与组件实例在更新中保持身份。

三枚Coin共享代码但有不同实例。未来如需伤害/重生等局部状态，可给Coin添加model；本例不为展示语法刻意增加与remaining重复的状态。原例三种外观保留，位置/形状的精确坐标约定须后续按渲染/碰撞实际合同校准；本草图不是逐像素等价证明。

## 4. 可复用Coin：事件向父组件显式绑定

`components/coin.at`：

```auto
use auto.graphics: trigger2d, ActorRef2D, Mesh2D, Material2D

widget Coin(position: float2, mesh: Mesh2D, material: Material2D,
            target: ActorRef2D, enabled: bool, on_collect: msg) {
    msg { Touched(ActorRef2D) }

    view {
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

`on_enter: .Touched`表示将触发事件payload传给当前Coin实例；`on_collect: .Collect(CoinKey.Amber)`在父作用域构造已绑定接收者和参数的事件端口。Coin不认识G11、不写父model，也不内置全局score。一次成功处理后父声明移除实例；在移除前已排队的重复请求由父的remaining检查去重。

这里只展示一个明确需求：一次收集使规则状态和声明结果一致。不是默认跨组件/VM的分布式回滚承诺；事件发送与handler失败的具体策略另需收敛。

## 5. Player：model保存意图，运动body保存物理状态

`components/player.at`：

```auto
use auto.graphics: character2d, fixed_update, InputFrame, ActorRef2D
use resources: player_mesh, player_material

widget Player(spawn: float2, speed: float, enabled: bool) {
    model { var move_axis float = 0.0 }
    msg { Step(float, InputFrame) }

    export { actor: ActorRef2D = refs.body }

    view {
        character2d body {
            initial_position: .spawn
            mesh: player_mesh
            material: player_material
            collision_layer: Character
            collision_mask: Solid
        }
        fixed_update { on_tick: .Step }
    }

    on {
        .Step(dt, input) -> {
            .move_axis = 0.0
            if .enabled { .move_axis = input.axis(Left, Right) }
            refs.body.move_and_slide((.move_axis * .speed * dt, 0.0))
        }
    }
}
```

Left/Right及Character/Solid代表具名输入动作与碰撞层，正式声明/导入位置尚待确定，不是隐含字符串或整数。图形profile提供公共运动服务；玩法意图仍由Auto决定。未加跳跃，避免把G11改成G09。

`initial_position`只在实例创建时生效；不能把spawn持续绑定到body.position而每帧拉回出生点。运动后的实际位置由character body持有，camera/trigger消费该实际状态；需要向外读取时提供受控查询/输出绑定，不在model再维护一套未碰撞的坐标。export/ref语法未定，但须表达类型化actor端口，而非任意访问子组件私有树。挂载/refs就绪后才可激活相机/trigger/tick，条件卸载后迟到事件不能命中新代实例。

## 6. Hazard与独立逻辑模块

`logic/patrol.at`：

```auto
// 纯函数：三角波巡逻；没有全局可变状态。
fn patrol_y(base float, phase float, extent float) float {
    if phase <= extent { return base + phase }
    if phase <= 3.0 * extent { return base + 2.0 * extent - phase }
    return base + phase - 4.0 * extent
}
```

`components/hazard.at`：

```auto
use auto.graphics: trigger2d, fixed_update, ActorRef2D
use resources: hazard_mesh, red
use logic.patrol: patrol_y

widget Hazard(x: float, base_y: float, target: ActorRef2D, on_hit: msg) {
    model { var phase float = 0.0 }
    msg { Step(float), Touched(ActorRef2D) }

    view {
        trigger2d body {
            position: (.x, patrol_y(.base_y, .phase, 0.2))
            shape: circle(radius: 0.5)
            mesh: hazard_mesh
            material: red
            on_enter: .Touched
        }
        fixed_update { on_tick: (dt, input) => { emit .Step(dt) } }
    }

    on {
        .Step(dt) -> { .phase = (.phase + 3.0 * dt) % 0.8 }
        .Touched(actor) -> {
            if actor == .target { emit .on_hit }
        }
    }
}
```

Hazard每个固定tick推进，不因Player移动或收集事件而跳过。这里刻意去掉旧32credits导致的调度绕行，不能宣称与旧轨迹逐tick一致；3.0单位/秒对应原60Hz每tick0.05的名义速度，三角波替代旧越界后翻转也会有小差异。危险物并非solid/自由刚体，只是自身位置更新的trigger。

main-alt可在同一G11Round定义中将base_y改为0.9、Player速度改为合法候选值来验证行为敏感性；实际测试仍需证明接触/结果改变，不能仅凭参数不同就标通过。参数可进一步上提为关卡props，不复制整份玩法代码。

## 7. 应用会话与新局

`app.at`：

```auto
use auto.graphics: graphics_app, action_input
use game_types: Result
use g11_round: G11Round

widget App {
    model {
        var round_key int = 0
        var last_result = Result.Playing
    }
    msg { Finished(Result), Restart }

    view {
        graphics_app {
            G11Round current_round {
                key: .round_key
                on_finished: .Finished
            }
            action_input {
                action: Enter
                on_pressed: .Restart
            }
        }
    }

    on {
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

key改变重建当前局的组件实例子树：remaining、result、Player body、Hazard phase、trigger状态回初值；共享不可变资源和App会话无需重建，不能等同全进程/全VM重启。旧局订阅/输入与迟到事件须按实例代次隔离，资源按owner释放。Enter为按下沿，不因held连续重开；在重开新局时不重放旧沿。round_key溢出处理沿未来整数/实例key合同，不能静默回绕复用旧身份。

本稿不强制加HUD。为讨论绑定可以以后增加只读展示`3 - remaining.count()`和result的组件；它不参与计分，正式文字/字体支持仍由对应能力交付决定。

## 8. 更新顺序、未定语法与讨论点

期望的语义顺序是：采样一次输入→Player/Hazard固定更新→同步位置/变换→触发计算→有序投递事件→处理model/on→按变更增量更新存活组件/绑定→准备呈现。**这是目标草图的候选顺序，旧G11的N几何→N+1回调消费不被本文改写。** 实施前须明确采用哪种事件时序、跨owner写入/通知边界、同时Collect/ReachGoal/HitDanger的顺序与终局优先级；本文不以树的遍历位置偷偷定义胜负规则。

fixed_update是每世界逻辑tick的订阅，同一实例被多Portal呈现不重复执行；多个组件不自动等于多个线程/VM。refs必须由领域分析解析到实例化关系，再由运行时管理代次；源码布局和最终ECS/数组存储无关。事件handler按真实接收实例执行，不能展平成一个全局字段表。

优先讨论四件事：

1. 图形组件是否继续叫widget；`scene2d`只作为view节点是否自然？
2. `Coin amber`和`refs.player.actor`能否清楚地区分定义、实例与公开端口；export是否需要另一种写法？
3. 事件端口用现有msg外观配类型推导，还是明确的带参数事件类型；emit的语法如何与AutoUI现有分派统一？
4. 条件view退出是真实销毁、key变化重建状态子树，这些规则是否作为UI/图形共同实例语义？

缺依赖、引用错型、重复局部绑定、资源导入失败、事件周期/消息过载、条件卸载、实例迁移和状态保留都需要后续具体诊断/生命周期合同。本稿先验证作者能否清楚表达规则，不预建通用runtime或声称已解决这些问题。

## 9. 本轮检查

仅核对原G11玩法意图、AutoUI写法参考、模块职责/资源与事件引用、文档链接与围栏。未编译或运行草图；其文件名、节点、构造器、refs/export和生命周期API均未落盘为产品源码。现有ag-examples示例、锁/验收、在途实现和合同保持原样。
