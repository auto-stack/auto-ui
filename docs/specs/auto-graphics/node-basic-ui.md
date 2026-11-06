# AutoGraphics 基础 UI Node 作者入口（auto-ui PLAN-048 SD-01 候选）

Status: candidate（auto-ui PLAN-048 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node-basic-ui.md 的 SD-01 候选文本，merge 阶段按
收据落位。只描述实际交付并验证的能力；基础 UI 是显式 AGE 新作者入口的
最小可交付切片，不声称 2D/3D 嵌入（PLAN-049）或任何生产 UI 组件库能力。）

## 0. 范围

`node2d-lower --profile node-ui` 接受普通 Auto 模块工程，把基础 UI 呈现
升级为显式 Node 作者模式：`node Name(params) { model?/msg?/view/on? }`，
`view` 块声明 row/column/text/button、条件呈现（`if`）与同定义实例；
业务全部落在生成的真实 Auto 用户源（`world.at`），UI 结构/绑定/事件端口
落在版本化产物 `ui.json`（schema `autographics.node-ui/1`）。执行侧由
driver 的 `Api::NodeUi` 受检装配（入口只有 `fn main()`；admission 仍强制
`app_start`/`app_update`）与通用适配器（`driver/src/node_ui.rs`）消费：
真实 stock Iced 0.14 控件（真实 `UserInterface::update` 事件路径）、真实
鼠标点击与真实 OS 键盘（离屏与原生通道分开取证）、事件经有界队列按稳定
次序派发到固定 SDK `Session`。

执行证据（只述实测）：**离屏 GPU**（`tests/node_ui_flow.rs`）——V-S01 两
同定义 Counter 实例真实点击 A×2/B×1 → Auto 状态 2/1、getter 文字变化、
GPU 像素扫描变化；源级对照（改 Auto `step` ＋重签身份）同点击复放 4/2，
Rust 无业务常量。V-S02 真实键盘 Tab/Enter 路由激活（同队列同派发器）、
禁用按钮点击零消息、陈旧 Run 事件具名拒绝、队列 64 上界计数。V-S03 条
件卸载/重挂、重绘不代 Tick、resize 重收敛、20 轮 fresh Run 无跨 Run 泄
漏。**原生窗口**（`bin/node_ui_window.rs --probe`）——真实 OS 窗口内
SendInput/SetCursorPos 三击（A/A/B）＋真实 Tab/Enter 键（029 先例通道；
DPI 换算 client px），a=3（2 击＋Enter 激活）b=12，exit 0；
`--sabotage-focus` 正控失败 exit 1（无焦点通道不投递，计数保持基线）。
**冷消费**（`tests/node_ui_storage.rs`）——独立消费者读同一冻结产物同链；
行为源 sha 身份链在消费入口强制（坏 world 副本具名拒绝，精确修复后下一
实际 Run 恢复）；schema 拼写漂移拒绝。`tools/verify_plan048.py --controls`
：再生字节一致、身份链、双图冻结容量常数、SDK681 verify、旧档 byte-pin
控制全过。

非范围（不冒称支持）：`view` 与 view-2d/view-3d 同块（解析面具名拒绝；
混合嵌入是 PLAN-049）、生产 widget 全局退役、文本输入/IME、滚动/网格/
复杂布局、主题/全部组件库、多窗口、`Life.Frame`、文字 >4 槽/每 def、
payload 事件（v1 零参）、float 模型字段。**旧入口不变**：components-2d
probe 冻结 world.at 逐字节 pin（`node_ui_gates.rs` 内嵌对照），全套
pilot 旧档测试复跑全绿。

## 1. 声明面（T-02 冻结拼写）

- `view { … }`：单一基础 UI 出口；与 view-2d/view-3d 并存具名拒绝
  （`node 'X' declares view together with view-2d/view-3d …`）。
- 元素：`row name { props+children 混合块 }`、`column name { … }`
  （group2d 同文交错块）、`text name { text, visible? }`、
  `button name { text, enabled?, visible?, onclick }`、`if cond { … }`、
  `NodeName inst { params }`（同定义实例；其 view 并入宿主布局）。
- 属性白名单（未知具名拒绝）：容器 width/height/spacing/padding（数字
  字面量或前导点字段）；text 的 text（字符串字面量 | int/bool 字段 |
  常量文字字段内联）；button 增 enabled（bool/int 条件）与 onclick。
  小写属性名与元素头同形（`text`）由冒号前瞻消歧（后随 `:` = 属性）。
- `onclick`：`.Ev`（本实例）或 `inst.Ev`（子实例；经生成绑定表
  `{prefix}_bind_{child}_{EV}` 解析接收方）；v1 零参（声明载荷的变体具
  名拒绝）。派发 ABI 唯一：`{prefix}_ui_click_{elem}(i?)`——适配器只认
  派发器名，不直呼业务 fn。
- `model`：int/bool/常量 UTF-8 文字（str 字段按字面量内联进绑定文字元
  素，不过 VM）；int 运行时参数允许（SoA 参数数组供 handler/绑定读）；
  float 具名拒绝。
- `on`：`Life.Start`（每实例一次，`app_start` 派发）、`Life.Tick(dt)`
  （每逻辑步一次，`app_update` 派发；重绘/layout 永不代 Tick——流测试
  三帧重绘 ticks 不变即证）、`.Ev` 零参业务事件。保留名冲突具名拒绝。

## 2. 产物与消费合同（T-02/T-03 冻结）

- `ui.json`（`autographics.node-ui/1`）：生成器身份、`world_sha256`
  行为源绑定、实例表（路径→def_idx，声明先序）、每 def 元素树（稳定元
  素名；文字槽位 = 声明序）、事件派发器名与实参形态（`index`/`none`）。
- `world.at`：SoA 实例数组（root 标量 `r_<field>`）、运行时参数数组、
  绑定表、recv fn `{prefix}_{ev}(i)`、每实例 64B UTF-8 文字缓冲＋len
  （PLAN-049 起字节缓冲按半缓冲拆分：缓冲 A ≤2 槽×64B=128，缓冲 B
  `text_b` 仅在 def 文字 >2 槽时声明；PLAN-065 起 ≤2 槽 def 的
  digits/getter 高槽分支同步剪裁——e 合法域恒 < 槽数 ≤ 2，单分支直写
  缓冲 A，不引用未声明 `text_b`（AG021 D-06 具名反例关闭：曾生成
  SDK `Undefined variable: {p}_ui_text_b_0`；≥3 槽 def 生成段逐字不
  变））、有界数字/布尔格式化助手（最小位数十进制，负号，>9999999 →
  OVER；bool → true/false）、refresh fn、click 派发器、getter
  （`get_{prefix}_{field}_{idx}` / `get_{pathu}_{field}` / `get_r_{field}`
  ＋ `{prefix}_ui_text_len(i,e)` / `_at(i,e,j)`）。
- 消费入口（驱动侧 `NodeUiApp`）：`main()` 恰一次在任何回调前（全局数
  组字面量在此初始化——未调用即写数组 `Invalid array ID: 0`，探针实录；
  小数组未初始化读静默 0，不构成初始化证据）→ `app_start()` 每 Run →
  `app_update(dt)` 每逻辑步 → 事件经队列 → 生成派发器。
- 行为源 sha 身份链：`world_sha256` 必须与产物一致，消费入口具名拒绝坏
  副本（`behavior source identity: …`）；修复后下一实际 Run 恢复。

## 3. 有界资源与预算（T-02 冻结；双图常数互检）

| 资源 | 冻结界 | 位置 |
|---|---|---|
| 有状态实例 | ≤8（root 计入；沿 020 envelope） | VM SoA |
| 每 def 元素 | ≤32 | 适配器（产物驱动） |
| 每 def 文字槽 | ≤4（64B×4=256 = SDK 字面量数组硬上限；PLAN-049 起按两个 ≤128 半缓冲拆分声明） | VM 每实例数组 |
| 文字字节 | 64B/槽分配；内容 >63 字节（**按字节计数**，多字节 UTF-8 同界）具名拒绝——63 字节合法、64 字节拒绝，字面量与常量文字字段同界（PLAN-065 边界实证） | VM |
| 容器子元素 | ≤8 | 适配器 |
| 事件队列 | ≤64 待处理；满则新事件具名丢弃计数 | 适配器 |
| 每回调预算 | 不变（1000 steps/slice；cumulative 100k；≤120 slices；250ms） | pinned Session |

SDK 硬事实（T-03 探针实录，冻结消费合同依据）：字面量数组 >256 元素被
**静默截断**为 `n-256`（256→0、300→44、385→129 实测）；文字槽 ≤4 的冻
结即由此而来。费用：一次 UI 事件 = 一次 Session 回调（既有预算）；文字
重建在派发回调内（`_ui_refresh`），无新 VM opcode；旧预算（32 credits/
回调、旧 VM/物理/场景/资源界）不动，pilot/driver 两图锁不变。

## 4. 输入与呈现通道（V-S02 取证纪律）

- 离屏行：真实 iced_wgpu `UserInterface::update`（真实事件结构、真实焦
  点/命中、GPU 呈现与像素读回）——`node_ui_flow`。
- 原生行：真实 OS 窗口 + SendInput/SetCursorPos（DPI 换算 client px、
  前台核验）——`node_ui_window --probe`；正控 `--sabotage-focus` 必须失
  败。两通道分别记录，从不互代。
- 键盘：stock iced 0.14 button 无键盘/焦点支持（对照 iced_widget 0.14.2
  源码核实——update 仅鼠标/触摸路径）；Tab 焦点环与 Enter 激活由适配器
  平台输入路由承担（设计 §3：Rust 做调度/输入与协议适配），激活经**同
  一队列同一生成派发器**（真实 OS 键事件 → 环推进 → 入队 → Session，
  原生链 a=3 即证）。禁用/不可见按钮不入环；点击禁用按钮零消息
  （iced `on_press_maybe` 语义）。

## 5. 身份与消费（公开消费身份，供 049/AG021）

- 生成器：`autographics-node2d-pilot plan048 node-ui`；API：
  `autographics.node-ui/1`；CLI：`--profile node-ui`（driver profile 号
  118 / cli 名 `node-ui`）。
- 消费入口：driver `src/node_ui.rs::NodeUiApp`（产物复检 + 身份链 +
  队列 + 真实控件树）；公共窗口 `bin/node_ui_window`；验证入口
  `pilot tools/verify_plan048.py --controls`（再生/身份/容量/SDK/旧档全
  控）。
- 049 只读本交付：元素树/实例表/派发 ABI/容量常数即混合嵌入的稳定接缝；
  `view` 与图形出口的互斥守卫在 049 打开（本片具名拒绝保持）。
