# AutoGraphics 三维具名引用与跨组件接线（components-3d-bindings）

status: candidate SD-01 (PLAN-030 r1；merge 落位)
applies-to: autographics-node2d-pilot components-3d-bindings 档（`--profile components-3d-bindings`）
assembly: autographics.triggers3d/1（与 028 同一受信组合；零新版本、零运行时单元改动）

## 1. 目的与边界

在 028 components-3d-events 的全部面之上，为作者补三类能力：具名 Slot/Template/Pair
符号参数、跨 Node 静态 ActorRef3D 参数、以及有状态逻辑组件（可无可见 leaf）与根显式
调度。作者不写数字 slot/pair/scene ID；编译器把名义符号下降为原数值 ABI（池声明序、
模板行 id、主题行表），运行仍走原精确 int 服务调用。028 的字符串端口/数值行为/生成
字节/准入反例不变（`Cfg3.bindings=None` 全路径字节冻结）。

非目标：新对象运行时/触发算法/队列/round 系统、动态 Node 拓扑/跨场景多挂载/多角色、
完整借用系统、动态 TRS、3D 查询/纹理动画/声音 HUD 组合、抬额。

## 2. 名义引用端口（准确拼写）

| 端口 | 绑定值形态 | 下降 |
|---|---|---|
| `SlotRef3D` | `objects3d.<槽名>`（根池裸符号点根） | 槽位 0..8（声明序） |
| `TemplateRef3D` | `templates.<模板定义名>` | 模板行 1-based 文档 id |
| `PairRef3D` | `triggers.<pair名>` | pair 0..8（声明序） |
| `ActorRef3D` | `refs.<实例路径>`（声明先序名字链；嵌套 `refs.父.子`） | 静态实例（定义前缀, def_idx）；`.参数.<元素>` → `{目标前缀}_{元素}_subject_ids[{目标def_idx}]` |
| `msg`（017 血统） | `.根事件`（零预绑实参 v1） | 根成员事件 recv fn 直呼（`<root前缀>_<ev>`） |

规则：

- 参数只读（`.参数` 赋值具名拒绝）；初始化一次建立，各实例 SoA 列隔离，修改自身
  model 不影响他实例绑定。
- 类别错配具名拒绝：Slot 与 Pair 不能互传（`slot: triggers.x` / `pair: objects3d.y`
  均在绑定面拒绝）；Template 与运行 Instance handle 不能互传；kind 转传（父参转发）
  必须同类。
- 作用域门：池/模板符号必须存在（未知具名拒绝）；actor 路径必须解析到本 Run 静态
  实例（group/未知/错路径具名拒绝）；`.actor参数.<元素>` 必须是目标定义的静态
  view-3d 叶（subject 表键集）。
- msg 端口绑定必须：根 msg 变体 + 根 `on .Ev` handler 存在、变体零载荷；同一定义
  的端口绑定跨实例一致（uniform port）。
- 运行期身份仍原 Run 池/行表（registry 于 app_start 前冻结）；重排普通模块/组件/
  模板声明后所有引用随新表重算，物理 ID/生成字节可变，不承诺旧 ID 不变。

## 3. 逻辑组件与作者调度

- 定义可无 view-3d（纯逻辑 Node：局部句柄/状态/msg/on）；object3d 模板仍纯静态，
  不占有状态 Node 配额。
- 有状态 Node 配额沿 020（8，root 计入）；槽池/对池沿 028（8/8）；credits 沿 028
  （32 共享，move 26、每对象调用 1、next 与每 getter 1、bind 1、round 原价、相机
  basis 0；msg/emit 为下降期直呼零服务费）。
- 子 Node 不声明 Life.Tick（根/子双 Tick 与显式消息重复推进具名禁止）；周期逻辑以
  msg 接收器暴露，由根 Life.Tick 显式调度：先推进（如 `hero.Move(dt)`，经 msg 参数
  回报 moved），再有限消费事件（具名 pair 常量比较分派到组件 msg），再按原条件提交
  巡逻/读重开。028 阶梯形状（移动读 1/静止至多 3/事件拍跳过巡逻提交而 hy 照常）是
  作者面，不是编译器规则；第二有限策略（如至多 2 条非移动耦合）同样作者组织。
- spawn/bind 可携带引用参数实参（`.slot`/`.template`/`.pair` SoA 读）与点根符号
  （root 无参数面）：`spawn(objects3d.v, templates.T, ...)`、`bind(triggers.p, 主
  题, shape, r, src)`；主题 ∈ {`refs.<本定义元素>`, `<actor参数>.<元素>`}；字符串
  字面量名字面（028 面）继续接受。
- 事件游标实际读取（next/pair/phase/source…）仍各按原价计费；具名比较是常量解析，
  不重新读取。

## 4. 初始化与投递序

沿 020 冻结序：root lifestart → 声明先序实例 lifestart（各 owner 在自己的 Start
spawn/bind）；registry 在 app_start 前冻结；角色/相机注入位沿 027/028（实例
lifestart 后 `ag_char3_bind`/`ag_camera_orbit_3d`）。msg 投递：子向 `emit
<inst>.Ev(args)` 经下降期绑定表直呼（每实例接收下标）；子向祖先 `emit .Ev()` 经
最近祖先接收器表；msg 参数经绑定根事件 recv fn 直呼。

## 5. 验收映射

- AC-01（具名参数/转传/拒绝面）：pilot node3d_bindings_gates + verify_plan030 负控
  12 条（类别/未知/作用域/只读/载荷/转传/归属）。
- AC-02/03（八组件链 + 原 G11/3d 25 oracle）：driver node3d_bindings_flow（bg11-main/
  alt/triple/geometry/oldgen/stale + sensor2）。
- AC-04（预算/写帽/槽域/tick0 请求）：flow 内 fault 10/37/31/41 变异 + verifier
  引用 SoA 形状钉死。
- AC-05/06（旧档冻结/来源闭包）：verify_plan030 identity/冷再生/027 g07a + 028
  g11-main 旧档字节冻结、driver runtime api pins、Gallery 只读基线。
