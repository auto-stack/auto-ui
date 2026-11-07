# AutoGraphics 组件/实例活区域与条件挂卸生命周期（auto-ui PLAN-054 SD-01 候选）

Status: candidate（auto-ui PLAN-054 T-02..T-06 实测交付；本稿为
docs/specs/auto-graphics/node-mixed-regions-lifecycle.md 的 SD-01 候选文本，
merge 阶段按收据落位。只描述实际交付并验证的能力。v2 是 049 混合组合之上
的显式新版本：v1（`autographics.node-ui-mixed/1`）的产物身份、支持域与
具名拒绝语义逐字保持，本档不倒改 048/049。）

## 0. 范围

`node2d-lower --profile node-ui-mixed-regions` 在 049 混合语法之上把嵌入
区域放行到**任意 def**：普通 `use` 组件的 `view` 树内可声明
`view-2d name { width/height/onclick? + 场景元素 }` / `view-3d …`，随组件
实例化（实例总量 ≤8 含 root；每 def 区域 ≤4、每区域场景元素 ≤8、区域边长
1..=640——数字与 049 冻结面一致）。区域身份＝
`{Run 代次, 实例路径, 区域局部名, 挂载代次}`；行为 ABI 与 048/049 完全同面
（区域点击走同一 `{prefix}_ui_click_{region}(def_idx?)` 派发器——root 零参、
组件带实例下标；场景 Field 绑定走同一 int getter，键
`{实例路径}#{区域}#{元素}#{属性}`）——**零新增世界 ABI 行**。执行侧由
driver 的 `Api::MixedRegions2`（number 120 / profile 29）受检装配与
`node_mixed_regions.rs`（v2 解析＋实例寻址区域目录/场景状态/点击入队）＋
`node_mixed_regions_host.rs::MixedRegionsHost`（metadata 驱动装配，无固定
App/区域名/尺寸常数）消费：UI 为窗口面 Iced view；每个**活动**区域一
Game2d/Scene3d child surface（dim 只选 adapter；声明内容尺寸定 target 与
3D aspect），Flat2d 交互挂载跟随真实布局 bounds；CompositionInput 门户
优先命中——区域内点击回传 Auto（同一有界队列），区域外归 UI；一个
Session owner。

执行证据（只述实测）：**离屏 GPU**（`tests/node_mixed_regions_probe.rs`
6 例＋`tests/mixed_regions_lifecycle_flow.rs` 5 例）——非默认命名全链
（Dash/Gauge/Meter、Panel/Dial/Bar；无 Main/reg2/reg3）；同定义两实例
点击 2:1 状态隔离且像素分开（经门户合成的 UI 面阈值判色）；真实 UI 按钮
→ Auto 状态 → root 区域像素位移；显式步 N 步＝N Tick 逐实例推进且 2D/3D
合成像素随步刷新（呈现不滞留旧帧）；条件隐藏＝撤呈现（remove_portal＋
remove_view＋drop_surface，live_target_slots/owned_target_bytes 精确收
支），释放名额可被后续区域复用；卸载期间旧挂载代次区域事件 drain 前具名
拒绝（stale_rejected，不派发进新代次）；重挂新代次、实例状态跨挂卸保留、
重挂后点击正常生效；20 轮挂卸/步进寿命零 target/字节增长、零故障；陈旧
Run 代次与队列 64 上界具名拒绝；v1/v2/048 schema 双向互拒。**生成面**
（pilot `tests/mixed_regions_lifecycle_gates.rs` 7 例）——三份冻结产物再
生逐字节一致；世界 ABI 针脚（组件派发器 `i int`、实例路径 getter、SoA、
半缓冲）；v1 档对组件区域的 049 冻结诊断逐字；per-def 区域预算、边长界、
场景元素界（恰 8 合法/第 9 具名拒）、未知 prop/跨维元素/float 绑定具名
拒。**冷消费**（`tests/mixed_regions_lifecycle_storage.rs` 3 例）——独立
消费者读同一冻结产物同链；行为源 sha/生成器/闭包模块身份链在消费入口强
制（坏副本具名拒绝，精确修复后下一次实际 Run 恢复）；区域事件声明对账
fail-closed（挂载知识由消费者注册表声明）。**原生窗口**
（`bin/mixed_ui_window.rs --probe` 为 049 入口不变；v2 入口
`bin/mixed_regions_window.rs --probe`）——真实 OS 窗口 posted 注入六点击
＋五按键（Tab/Enter 经平台键盘路由驱动条件挂卸两轮），count=202（含 resize
后命中 +1）、p1 跨挂卸 x=12→24、挂载代次 3→7（未卸载区域代次不变）、真实
窗 resize 逻辑 480×560→400×420 且区域保持活跃、**resize 后控件命中按新布
局**（posted 物理坐标＝逻辑×scale）、空闲零 Tick、scale_factor 2.0，exit 0。
探针按状态门推进（隐藏/布局跟随/逻辑尺寸各有时限与具名失败），真实
`Focused(false)` 后由自适应 Tab 重新取得焦点而不是靠固定 700ms 节奏；同一
入口连续三次 `--probe` 均 exit 0 且 count=202（回归在案）。
PLAN-066 起焦点/尺寸面收口：`Tab` 才取得键盘焦点（未 Tab 的 Enter 不激活
任何按钮）；真实 `Focused(false)` 取消在途捕获并清键盘焦点，旧焦点按钮不
得在下一次 Enter 被激活；`CursorLeft` 只取消在途捕获/按下（不清键盘焦
点）。0×0（最小化）与亚像素逻辑尺寸（如 1px@scale2=0.5）被宿主具名拒绝并
保留上一可用尺寸/布局，下一次真实 resize 恢复（永久回归
`tests/mixed_regions_lifecycle_flow.rs`）。真实 native SendInput/焦点捕获/
DPI 真切换仍为手动门：`mixed_regions_window --manual` 是可执行记录器（关窗
写 `manual-native-gate.json`，`auto_pass:false`，记录真实 OS 键/点击/失焦/
离窗/resize/scale）；缺第二 DPI 设备或明确授权时保持 pending，本档不冒称
已验。
`tools/verify_plan054.py --controls` 全过：三 v2 产物身份链＋组件区域形
状（4 组件区域）＋半缓冲边界＋bool 高槽修正标记＋双图容量常数（含 v2 活
区域 4）＋SDK681 verify＋旧档控制在场＋全部再生字节同一（含 049 v1 夹
具）。

非范围（不冒称支持）：v1 产物经本档入口消费（schema 具名拒绝；v1 入口不
变）、区域内 If/Group/Instance（区域内容仍是冻结图元集）、坐标负载事件、
float 场景绑定、node 级 view＋view-2d/3d 并存、UI 贴 3D 网格/曲面、多窗
口、跨实例父→子直接消息（父子面＝既有实例化 props＋自身事件）、生产全量
UI/主题。**旧入口不变**：048/049 产物与控制在场并复跑全绿（driver 34/34，
其中 048/049 旧档并集 20 例、本 Plan 新门 probe 6＋flow 5＋storage 3；
pilot 16 例；v1 夹具再生字节同一）；文字越界与文本槽预算两处生成
侧修正在 T-05 注明兼容面（仅越界源从消费拒绝改为生成具名拒绝；bool 高槽
仅「恰为 bool 且槽 ≥2」的生成字节从错写改对写）。

## 1. 声明面（T-02 冻结拼写）

- v2 语法＝049 混合语法**零新增**：区域头 `view-2d/view-3d name { … }`
  在任意 def 的 view 树内放行（`Node2DMixedDialect` 不变；node 级双出口
  守卫不变）；条件区域＝`if` 包裹（区域无独立 visible prop；分支消失即
  该子树卸载＝撤呈现）。
- 区域 props 白名单不变：`width`/`height`（int 字面量 1..=640）、
  `onclick`（`.Ev`｜`inst.Ev`，同按钮 ABI；组件区域派发器带实例下标）。
  未知 prop/集外场景元素/float 绑定具名拒绝。
- 组件实例化＝普通 `use`（`use bar: Bar` ＋ `Bar b1 { }`）；实例表声明
  先序；实例参数/模型字段沿 048 面（含 bool 类型字段——T-05 修正后
  bool 文字在任意槽位走半缓冲正确面）。

## 2. 产物与消费（AG021 配方）

- 产物：`ui.json`（schema `autographics.node-ui-mixed-regions/2`；
  generator `autographics-node2d-pilot plan054 node-ui-mixed-regions`）＋
  `world.at`（行为源；sha 绑定于 ui.json/identity.json）＋ `closure/`
  （逐模块 sha）＋ `identity.json`（provenance：product/consumer/
  consumption/world_sha256）。冻结参考产物：
  `generated-node-mixed-regions/{ui-dash-regions,ui-bool-slots,
  ui-panel-swapped}/`。
- 消费配方（AG021 固定选择面）：`MixedRegionsUiApp::new(ui_json, world,
  name)`（v2 解析＋容量门＋身份链）→ `MixedRegionsHost::new(gpu, app,
  logical, window_format)`（start_run；区域目录来自实例行；逐帧真实布局
  ＋条件驱动激活/挂载/撤呈现；活区域全应用合计 ≤4，超界挂载具名拒绝、
  名额释放后可复用）；区域点击经 host 指针路由（命中即按当前挂载代次入
  队）；显式步 `host.step(dt)`；resize `host.resize(logical)`；冷消费面
  `parse_mixed_regions_artifact`＋`enqueue_region_click_at(path, region,
  epoch)`（声明代次注册表 fail-closed）。参考消费者：
  `tests/node_mixed_regions_probe.rs`＋`tests/mixed_regions_lifecycle_{
  flow,storage}.rs`＋`bin/mixed_regions_window.rs`（identity.json
  consumer 字段同记）。
- 身份链：行为源 sha＝ui.json.world_sha256＝identity 绑定；损坏副本消费
  入口具名拒绝，精确修复后下一次实际 Run 恢复；schema/生成器拼写漂移具
  名拒绝；v1 产物不可入本档入口。

## 3. 冻结预算与故障域

- 048/049 常数全部不动：实例 ≤8（root 计入）、产物元素 ≤32/def、文字
  ≤4 槽/def（64B×4 半缓冲拆分；**按钮标签同占预算**——T-05 起生成侧
  同面具名拒绝；PLAN-065 起 0..4 槽组件 def 全域可生成执行——≤2 槽
  def 单半分支剪裁不再引用未声明 `text_b`（AG021 D-06 关闭），双同
  定义组件实例隔离/文字更新经 few_slot_flow 实证）、容器子 ≤8、队列
  64、每回调 1000 steps/cumulative 100k；混合：区域 ≤4/def、场景元素
  ≤8/区域、区域边长 ≤640。v2 新增：**活动区域全应用合计 ≤4**（挂载时
  具名拒绝；不按实例复制额度）。
- 寿命语义：条件隐藏＝撤呈现（portal＋view＋surface 释放，最后 owner
  收支闭合）；实例行（状态 owner）不随挂卸生灭；重挂＝新挂载代次；旧代
  次区域事件在 drain 派发前具名拒绝（未挂载与重挂后都拒绝）；陈旧 Run
  代次沿 049 面；重绘/layout/挂载刷新永不代 Tick（空闲零回调实测）。
- 键盘焦点（PLAN-066）：`Tab` 取得焦点并循环，`Enter` 只在取得后激活当前
  按钮（与鼠标同队列同派发器）；真实窗口失焦（`Focused(false)`）清键盘焦
  点，指针离窗（`CursorLeft`）取消在途按下/捕获但不改键盘焦点；旧焦点按钮
  不跨失焦泄漏，无 Tab 的 Enter 不激活任何按钮。0×0/亚像素逻辑尺寸被
  `resize` 具名拒绝并保留上一可用布局（最小化不产生半像素表面）。
- 故障域：Session 故障 → 相关域停止（队列清空、最后可用呈现保留、具名
  错误）；激活资源失败（surface/view/portal）具名记账且应用保持可用；
  非法区域（越界/坏 dim/集外元素）生成与消费双侧具名拒绝。
