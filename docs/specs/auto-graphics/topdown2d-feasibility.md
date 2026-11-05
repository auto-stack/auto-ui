# 俯视二维可行性与容量边界

PLAN-055 的有界研究知识；不新增生产 profile、语法、物理或 tilemap API。
适用版本：auto-ui `220115df62040f113bc689f0500af80fc11569a3`，
其 `lower_components.rs`、`translate.rs`、driver `api/` 与
`research/source-lock.json` 对计划输入 `e02ca9f8` 无差异。
作者编译器使用同组 auto-lang `a0750086c57c809a586884ac8b0519e8c162979b`；
运行 VM 是 SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，681 文件逐一哈希校验。
不能将作者编译器版本、旧 VM SDK 或 PLAN-049 的 UI ABI 混为同一运行合同。

## 结论及使用边界

**通用图元最小原型 bounded-go；按本研究“一格一可见图元、完整铺地”的
地图建模 no-go；现有角色服务的同回调移动与目标查询组合 no-go。**
结论只针对以下源码、规模和条件，不断言所有未来地图都不可行，也不表示
完整种田游戏、2DHD 画面或产品级俯视控制能力已交付。

| 机制 | 实际结果 | 限制 |
|---|---|---|
| A：普通 character2d / move_and_slide | queries-2d 的 x/y 输入驱动无重力移动；生成源接缝对照移除交互后，CPU 单次 26 credits，x/y 各从 2 到 2.0666666 | components-2d 的 `input.axis(Down, Up)` 在作者解析期拒绝 `Down`；没有该拼写的上下输入服务；未认领角色完整链 go |
| A：角色移动加 segment 最近目标查询 | 同回调 26+8+1=35，运行 fault 10 | 不能通过轮流停移动/停交互、加池或换 SDK 规避原额 |
| B：普通 shape2d 对象加 sweep、segment | Auto handler 决定位移和最近目标；无地面正控四向、斜向、停走、碰墙、目标状态及真实 Host/GPU 链成立 | 每回调 27 credits，剩余 5；不是 sprites-2d，也不是自动 slide |
| B：完整铺地 | 地面行参与 spatial 候选，起点 overlap，t=0，移动不发生 | 需要明确的渲染/碰撞层或等价公共合同；不能把不可见地面当已呈现地图 |

## 作者端口和数值

B 使用现有 `--profile queries-2d` / `autographics.objects2d/1`：
普通 `use/node/model/on Life.Tick(dt)`；`input.space()/x()/y()`；
`query2d.sweep(refs.runner, 1, dx, dy)`；结果 `.t()/.hit()`；
`refs.runner.shift(dx,dy)` / `.world(axis)`；
`query2d.segment(refs.runner, 1, ax, ay, bx, by)` / `.node()`。
没有自造端口或 Rust 玩法循环。

世界位置和尺寸单位均为 scene world units；dt 为秒，速度 4 或 8 world units/s。
space=2 是受控二维游戏输入层。斜向乘 f32 `0.70710677`；
碰撞后取 `max(t-0.001,0)` 的位移比例（0.001 为无量纲 t 裕量，
不是固定世界距离）；查询使用 inclusive f32 slab/Minkowski 语义。
独立宿主 oracle 同时核两轴轨迹、最近行 ordinal；不以截图或进程成功代替语义。
碰墙时本作者规则停止整个向量，后续纯沿边输入才沿墙移动；未实现自动 slide。
若每 tick 再增加第二次 sweep 及两 getter，总账由 27 增至 45，超过 32。
该账是此候选的成本，不是所有其他未来算法的下界证明。

最近交互使用向上 8 units 的 segment，近目标 (2,6)、远目标 (2,7)，
相同 t 按既有 ordinal 规则。目标 ordinal 存入普通 Auto model；交互呈现
对照以最近目标 3 / 错目标 4 区分显示状态。输入的相反方向抵消在这里以
轴值 0 的受控 fixture 重放，没有证明 OS 同时按键映射。

## 地图和原预算

| 样本 | 实际 scene 行 | 作者降层 | CPU 运行语义 |
|---|---:|---|---|
| 无地面、控制对象、障碍、近/远目标与相机跟随占位角色 | 5 | 接受 | 四向/斜向移动和目标查询成立；停墙约 x=4 |
| 8×8 单层单位图元地面，同一对象集 | 69 | 接受 | 地面 overlap，68 tick 坐标恒 (2,2) |
| 12×8 同规则 | 101 | 接受 | 地面 overlap，68 tick 坐标恒 (2,2) |
| 16×16 同规则 | 261 | exit 3，`scene exceeds 128 nodes (261 rows)` | 没有进入 VM/GPU，不报编译或绘制时长 |

图块没有独立预算。状态实例门包括根：根加 7 个带 model 图块（共 8）
接受；加第 8 个（共 9）exit 3，具名 `tile7_0 exceeds the 8-instance envelope`。
行/状态/资源限额是不同单位。沿旧门：128 行，8 状态实例/8 spawned slots，
32 credits/callback；VM 1000 steps/slice、100000 cumulative steps、120 slices
等原守护保持。没有分池、跳帧或提升额定值。

旧 SDK 数组字面量另有**已复放陷阱**：255 项 `.len()`=255，256 项=0，
257 项=1。256/257 不是受控超界拒绝，不能算合法地图存储；本研究不借截断
继续执行游戏，也不修旧 SDK。后继合同必须具名拒绝或修复编码并重验真实消费者。

## 测量口径

Windows / i5-13600KF，系统枚举 RTX 4060 Ti，实际呈现经 wgpu（未记录所选 adapter ID），rustc/cargo 1.98.0；
两套独占 target 首次离线锁定构建：pilot dev debug 6m11s，driver dev
debug=0 3m52s，二者不是同优化档的性能对比。
以下均为本机样本，不是产品 SLA 或跨机器容量承诺。

四条 B 输入各独立重建 Session 并运行 68 tick：

| 输入 | VM compile μs | 最大完整 CPU Tick μs | max steps/call | max slices/call | credits/tick |
|---|---:|---:|---:|---:|---:|
| control | 195402 | 1873 | 4337 | 5 | 27 |
| changed speed=8 | 168600 | 1763 | 4337 | 5 | 27 |
| 8×8 | 148935 | 13992 | 35142 | 36 | 27 |
| 12×8 | 158697 | 19629 | 50182 | 51 | 27 |

CPU Tick 包含完整回调，不等同 Host 单帧 pump 或 GPU 时间；8×8/12×8 的
本次完整 CPU Tick 已超过 8000μs 对照线，但仍低于 VM 步数/切片门，不能
将这一点伪称 VM 已拒绝。Host 可以跨帧 pump。冻结七条降层命令的端到端
样本范围为 23.3139–69.7427ms，其中五条接受样本为 38.5543–66.7361ms；
两条拒绝样本为 23.3139ms（标准角色输入）与 69.7427ms（16×16）。
这些时间与运行 VM compile/load/first-present 分开记录于归档 Plan。

真实离屏 GPU：同一 Host 20 轮 fresh Run/Stop，每轮斜向变换及橙色对象
像素中心 x=304→308；暂停呈现不推进状态，Stop 后 VM owner 和 spatial
descriptor 弱引用释放。每轮 target=4 / target bytes=7584000，mesh=7 /
136952 bytes，depth=18057216 bytes，two_buffers=196672 bytes；外部资产计数0。
这些计数是宿主显式拥有的资源，不涵盖驱动/iced 内部分配。
像素读回耗时不算稳态绘制时间，GPU timestamps 不可用。

同规则真实 Host 容量观测各一次：8×8 完整 load/compile/首次呈现
1809434μs、输入到呈现32444μs、10次暂停帧提交1266–1855μs；12×8 分别
1546319μs、45301μs、1595–2870μs。两者均保持 (2,2)，资源计数与上述
宿主相同，实例上传字节不同；这是可加载但不能遍历的地图。无地面交互
两变体各10次暂停帧提交1592–3905μs。暂停帧提交是宿主 CPU 墙钟，
不是 GPU 执行耗时或进行完整业务 tick 的帧率。

首版探针对每轮另建 GPU Host，完成 13 轮后创建变慢并停滞，已终止、保留
失败记录；它不证明整个 GPU runtime 的跨设备创建/驱动释放稳定。
正式 20 轮验证重用一个真实 Host，每次仍新建 Run/VM/query owner。
二者分别记录，不将 13 轮拼为 20 轮通过。

## 故障与未证范围

坏/缺世界源按绑定 SHA 拒绝，精确恢复后重新载入并真实 Run；过期 query
token fault 13 与新 Run 恢复、错 space 输入不移动分别复放。静态 row
引用不产生 spawned object handle，因此本候选没有冒认动态句柄代次的测试覆盖。
未引入 PNG/音频等资源，缺/坏外部资产分支不能由世界源故障代替；query 与
sprites 的 profile 组合缺口须在后继合同关闭。OS 键盘、窗口焦点、原生输入
未运行；所有输入是明确的 CPU transport / Host Action fixture。
本次另见 `.show(.target == 3)` 被降层接受但旧 SDK 在 Run 报
`runtime/_ag_error_argument: return type mismatch`；改用作者显式 int 0/1
局部变量后，正确/错目标的实际显隐与 GPU 像素对照通过。此处只支持经验证
的整数 flag 写法，不把布尔 ABI 缺口静默修入产品。

## AG022 与 M3 的合同输入

1. 先选地图表示，明确渲染图元与碰撞候选的分离、碰撞层/交互层、可遍历
   地面、地图坐标/尺寸、总行数、状态和每 callback 成本。完整铺地反例必须
   和无地面合法对照同时保留；不能仅提高行上限。
2. 角色与通用对象二选一冻结首切片控制语义：上下输入、斜向归一化、墙角/
   沿边、交互触发与最近/错目标、同 tick 规则、sprite/相机组合。
   角色 35-credit 反例和通用对象 27-credit 正控为预算设计输入。
3. 给数组/大地图字面量具名安全门；实际更大容量和任意精灵组合需要新的
   能力 Plan、ABI/锁身份和资源预算，不在本研究 merge 中新增。
4. 2DHD 可选分层图元/贴图 atlas、离线绘制背景、受控摄像机缩放与遮挡；
   光照、法线、景深、资源批绘、排序和美术路线仍是选择项，各自需要能力
   与成本证明。本 Plan 不替 AG022 选美术，也不交付完整效果。

完整研究源码、重建命令、输入哈希、失败样本与独立复审见
[PLAN-055](../../plans/archived/055-age-m3-topdown2d-feasibility-and-capacity.md)。
既有 [二维查询](node2d-queries.md)、[完整游戏组合](node2d-full-game-composition.md)
和 [精灵服务](node2d-sprites-and-frame-animation.md) 合同不变。
