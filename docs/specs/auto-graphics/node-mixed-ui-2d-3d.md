# AutoGraphics 基础 UI 内嵌 2D/3D 混合组合（auto-ui PLAN-049 SD-01 候选）

Status: candidate（auto-ui PLAN-049 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node-mixed-ui-2d-3d.md 的 SD-01 候选文本，merge
阶段按收据落位。只描述实际交付并验证的能力；混合组合是 048 基础 UI 之上
的显式嵌入切片，不声称跨维场景 parent、UI 贴 3D 曲面、实例级多挂载区域
或任何生产组件库能力。）

## 0. 范围

`node2d-lower --profile mixed-ui-2d-3d` 在 PLAN-048 基础 UI 之上放行
`view` 元素树内的 `view-2d`/`view-3d` **嵌入区域**（挂载关系，非跨维
parent）：`view-2d name { width/height/onclick? + 场景元素 }` 与
`view-3d name { … }`。业务仍全部落在生成的真实 Auto 用户源（`world.at`，
与 048 字节同面——区域点击走同一 `_ui_click_` 派发器 ABI，场景绑定复用
既有 int getter，**零新增世界 ABI 行**）；UI 结构/区域/场景绑定落在版本
化产物 `ui.json`（schema `autographics.node-ui-mixed/1`，Embed 元素在 def
元素树内）。执行侧由 driver 的 `Api::MixedUi2d3d` 受检装配（入口只有
`fn main()`；admission 仍强制 `app_start`/`app_update`）与
`driver/src/node_mixed.rs`（镜像）＋ `node_mixed_host.rs::MixedHost`
（单一生产链：装配/每帧桥/路由/resize）消费：UI 为窗口面 Iced view，
每区域一 Game2d/Scene3d child surface，区域实际 widget bounds 驱动
Flat2d 交互挂载（SetMount 跟随真实布局），CompositionInput 门户优先
命中——区域内点击回传 Auto（同一有界队列），区域外归 UI；一个 Session
owner（`MixedMirror` Send 桥只携版本化已接受状态副本——Session 持 Rc
线程受限，不进组合内容）。

执行证据（只述实测）：**离屏 GPU**（`tests/node_mixed_views_host.rs`
5 例＋`tests/node_mixed_views_flow.rs` 6 例）——按钮×2 → count=2 且
2D quad 每 click +8（像素新址红/旧址清退）与 3D cylinder 每 click +2
（投影位移像素对照）；双实例（Counter a/b）同定义独立推进互不串扰；区域
点击回传 Auto → UI 文字（互斥：count 不动）；Field 布局增长 → 挂载
SetMount 跟随（旧顶归 UI/新中心归区域）；resize 后路由随重读真实布局；
空闲 ≥60 帧零 Session 回调（重绘/layout 不代 Tick）；显式步恰一次
app_update；陈旧代次事件具名拒绝；Session 故障相关域停止＋最后可用呈现
保留；队列 64 上界具名丢弃计数。**原生窗口**（`bin/mixed_ui_window.rs
--probe`）——真实 OS 窗口内 posted OS 点击四相位（按钮×2/2D 区域/3D
区域），count=2、双区域回传=1/1、空闲零 Tick，exit 0；真实 native
SendInput 通道沿 048 手工取证先例（不在自动链）。**冷消费**
（`tests/node_mixed_views_storage.rs` 3 例）——独立消费者读同一冻结产物
同链（UI＋双维场景同状态）；行为源 sha 身份链在消费入口强制（坏 world
副本具名拒绝，精确修复后下一实际 Run 恢复）；schema/生成器漂移拒绝；
048 schema 与混合 schema 互不误收。
`tools/verify_plan049.py --controls` 全过：再生字节一致、身份链、混合
形状（root 切片/派发器绑定/半缓冲 <256）、双图冻结常数（含混合新增
区域 4/场景 8/边长 640/半缓冲槽 2）、SDK681 verify、旧档控制在场。

非范围（不冒称支持）：非 root def 区域（需实例挂载身份，v1 具名拒绝）、
区域内 If/Group/Instance、区域条件隐藏/卸载代次、坐标负载事件（v1 零
参）、浮点场景绑定（int getter 面）、`view` 与 view-2d/view-3d node 级
并存（任何 dialect 具名拒绝；039 查询档身份不扩大）、UI 贴 3D 网格/
曲面、多窗口。**旧入口不变**：048 产物/存储控制在场并复跑全绿；
components-2d probe 冻结 world.at 逐字节 pin；node-ui profile 对嵌入头
按基线语法面拒绝。

## 1. 声明面（T-02 冻结拼写）

- 区域：`view-2d name { props+场景元素 }` / `view-3d name { … }`（仅
  `Node2DMixedDialect`/mixed profile；基础 UI view 元素树内，挂载关系）。
- 区域 props 白名单（未知具名拒绝）：`width`/`height`（int 字面量
  1..=640；UI 占位 rect＝子内容逻辑尺寸）、`onclick`（`.Ev`｜`inst.Ev`，
  同按钮 ABI；区域派发器 `{prefix}_ui_click_{region}(i?)`）。
- 区域内场景元素（T-02 冻结集，超集具名拒绝）：2D `shape2d`（kind
  quad|circle；x/y/w/h/r：int 字面量或前导点 int 字段；color (r,g,b)
  0..255 字面量）；3D `cylinder3d`（translation (x,y,z) 三元，分量同上；
  color 同）。每区域 ≤8 元素；每 def ≤4 区域；仅 root def（非 root 具名
  拒绝——实例挂载身份待后续 plan）。
- 绑定：区域场景 Field 绑定经**既有** int getter（`get_r_*`/`get_*_*`）
  逐次状态接受后读取（键 `{path}#{region}#{elem}#{prop}`；cylinder 分量
  `#tx/#ty/#tz`）——世界 ABI 与 048 完全同面。
- 文字字节缓冲：半缓冲拆分（每半 ≤2 槽×64B=128）——SDK 字面量数组恰
  256 元素静默截断为 0（048 复审实测＋049 app_start 故障复证）；公开
  `len(i,e)/at(i,e,j)` ABI 不变。

## 2. 产物与消费（AG021 配方）

- 产物：`ui.json`（schema `autographics.node-ui-mixed/1`；generator
  `autographics-node2d-pilot plan049 node-ui-mixed`）＋ `world.at`
  （行为源；sha 绑定于 ui.json/identity.json）＋ `closure/`（逐模块 sha）
  ＋ `identity.json`（provenance：product/consumer/consumption/world_sha256）。
- 消费配方（AG021 固定选择面）：`MixedUiApp::new(ui_json, world, name)`
  ＝ `parse_mixed_artifact`（schema/容量门）→ `NodeUiApp::new_with_
  artifact(…, Api::MixedUi2d3d)`（身份链＋main-first）；呈现经
  `MixedMirror`（Send；已接受状态副本）＋ `MixedHost`（装配/帧/路由/
  resize 单链）；区域点击 `enqueue_region_click`；场景状态
  `scene_state()`（纯缓存读）。参考消费者：
  `tests/node_mixed_views_{host,flow,storage}.rs` ＋
  `bin/mixed_ui_window.rs`（identity.json consumer 字段同记）。
- 身份链：行为源 sha ＝ ui.json.world_sha256 ＝ identity 绑定；损坏副本
  消费入口具名拒绝，精确修复后下一实际 Run 恢复；schema/生成器拼写漂移
  具名拒绝。

## 3. 冻结预算与故障域

- 048 常数全部不动：实例 ≤8（root 计入）、产物元素 ≤32/def、文字 ≤4 槽
  /def（64B×4 半缓冲拆分）、容器子 ≤8、队列 64、每回调 1000 steps/
  cumulative 100k。混合新增：区域 ≤4/def、场景元素 ≤8/区域、区域边长
  ≤640。
- 故障域：Session 故障 → 相关域停止（队列清空、最后可用呈现保留、具名
  错误）；陈旧代次事件派发前具名拒绝；非法区域（边长越界/坏 dim/非
  root/集外元素）降层与消费双侧具名拒绝。
- 一次更新：`app_start` 每 Run 一次、`app_update` 仅显式逻辑步；重绘/
  layout/挂载刷新永不代 Tick（空闲零回调实测）；多 surface 共享同一
  owner 的已接受状态（呈现者零 VM 访问）。
