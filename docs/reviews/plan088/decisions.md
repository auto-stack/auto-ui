# PLAN-088 T-01 决策记录 — ColorPicker 与 ColorDialog

status: work-in-progress（T-01 探针进行中；本文件随探针结论更新）
plan: docs/plans/088-color-picker-and-dialog.md（r2）
owner: work stage（PLAN-088 goal session）

## 1. baseline

- 树：`D:/autostack/.wt/ui-088/auto-ui`，分支 `codex/ui-088-color-picker`，
  base commit `2f42d6613c6b1d0f24f7bd5679979ae42601d0aa`（master @ 2026-11-17）。
- 组内 sibling（detached @ master）：`auto-lang` c2f3b2aa9、`auto-web` bdc48e8、
  `auto-down` 5af11a3、`auto-man` 11fe79c（`auto` CLI，path-dep 回本树 auto-ui）。
- 相关既有面（已读）：
  - `crates/auto-ui-compiler/src/style/color.rs`：`Color::from_hex` 接受
    #RGB/#RGBA/#RRGGBB/#RRGGBBAA（8 位 → `Hex(u32)`=0xRRGGBBAA）；`from_css`
    接受 rgb()/rgba()；style class 解析（bg-[#hex] 等）与本计划无关，不改。
  - canvas 场景契约（PLAN-563/661）：`canvas { scene: .p }` 读状态平行串表
    `<p>_pts/_meta/_nodes/_edges/_labels`；nodes 项
    `"id,x,y,shape,color[,w|h]"`（rect 中心+宽高，坐标 f32）；edges 项
    `"x1,y1,x2,y2[,color[,width]]"`；pen 事件
    `onpenstart/onpenmove/onpenend` 回调 (x:float,y:float)（coords "WxH"
    逻辑值域；move 仅 pen-down 期，≤30Hz，离界收笔）；`onhit(id str)`
    pointer 命中 nodes（声明序 topmost）。双端各自绘制，共享数据契约。
  - `dialog`/`dialog-content` builtin（file-dialog 既有用法）：`open`/
    `onclose`/`closeOnOutside`，modal 隔离由 `View::Popover` Modal placement
    承载；初始焦点走既有 `ui.focus(...)`。
  - PLAN-087 交付（archived，45a80058 落主）：`packages/numeric-fields`
    NumericField（precision=0 用于 0..255/0..360 通道输入）；引擎 fn 全部
    自含（包模块直引 fn 才注册裸名，内部互调失链——本包沿用该形态）。
  - `.at` 语言事实（087 决议 + 024-charts/031-paint 实证）：int=i32、无
    浮点 state 桥；vue widget 轨 `/` 为浮点（精确整除形
    `(a - a % b) / b`）；fn 单返回值、无状态访问；handler 不可互调（状态
    变更内联）；`f"${expr}"` 插值支持表达式；`.to(int)` 数字桥仅对已验证
    数字串；List 有 push/split/len/索引。

## 2. public interface（对照 r2 §5 表）

- ColorPicker props：`value="#000000FF"`、`revision=0`、`showAlpha=true`、
  `palette=[]`、`disabled/readOnly=false`；ColorDialog 增 `open=false`、
  `title="选择颜色"`。
- 事件：`preview(str)`/`committed(str)`/`cancelled`/`error(str)`，载荷
  `{seq, baseRevision, value, source}`，source ∈
  hex/rgb/hsv/palette/surface/confirm。
- 错误码：`invalid_color/invalid_channel/invalid_config`。
- 数值规则：RGBA 通道整数 0..255；H 0..360（360≡0）、S/V 0..100；RGB 输出
  大写 8 位 HEX；`#3974E6CC` ⇒ RGBA(57,116,230,204)；half-up 舍入；
  HSV 灰/黑保留最近有效 H（编辑器草稿级）。
- fixture：T-02/T-03 落 `packages/color-picker/`（本节随实现补真实路径与
  生成命令）。

## 3. mechanism probe

### 3.1 色面渲染：canvas 场景 + 8 位 hex alpha 分层（选定）

**发现的原生缺口（本计划最小适配，已完成）**：iced 侧
`canvas_css_color`（`crates/auto-ui-runtime/src/ui/iced/renderer/subscriptions.rs`）
原实现 `from_hex → to_rgb8 → from_rgb8`：8 位 hex 被解析为 0xRRGGBBAA 后
`to_rgb8` 只读低 24 位——通道错乱且 alpha 恒丢；vue 侧同一 CSS 串透传
`fillStyle` 本就半透明。软栅格孪生 `css_color_tiny`
（`crates/auto-ui-runtime/src/ui/desktop_protocol/native_projector.rs`）只认
6 位且强制 alpha=255。两处已按位取 RR/GG/BB/AA 修复：6 位路径逐字节不变，
8 位起 alpha 生效；canvas 场景色串之外（style class 解析）不经过这两处，
零外溢。修复前仓库内无任何 `.at` 使用 canvas 场景（grep 实证），无回归面。

**SV 方面合成（免网格、免逐像素）**：白底 + 24 列 hue 色 alpha=s_i +
24 行黑 alpha=(1-v_j)。alpha 合成下 result = v·[(1−s)·W + s·H] ≡ 精确
hsv(h,s,v)（逐通道恒等式，非近似）。仅 hue 变化重建列层（24 串）；S/V
拖动只动光标（2 条 edge）与 alpha 条/预览（~26 串/30Hz 节流）。

**透明预览**：棋盘 nodes（#ffffff/#c8c8c8）+ 整块 color8 覆盖 rect；
alpha 条同构（棋盘 + 当前色 24 级 alpha 段）。不用黑底伪装。

### 3.2 坐标 → 通道量化（选定，无 float→int 桥）

pen 回调给 float 逻辑坐标；语言无 floor/float→int 桥（stdlib math 无
floor；`.to(int)` 仅数字串）。选定：`f"${xf * k}"`（k=100/W 等）得十进制
串 → 整数部解析（nd_ip 同型）→ 夹取。值域 [0..360)+0.001，双轨均为十进制
短表示（JS <1e-6 才转指数，Rust Display 恒小数），无格式分叉面。边界
（恰好整数）由 +0.001 底噪偏置，量化稳定；MCP 精确通道编辑走
NumericField，不依赖像素精度。

### 3.3 HSV↔RGB 引擎（选定，纯整数 half-up）

`hsv→rgb`：sector=(h−h%60)/60、f=h%60，t/q/p 以
half-up(a·num,den)=((2a+num)−(2a+num)%(2num))/(2num)（精确整除形）。
`rgb→hsv`：max/min/delta 整数运算；delta=0（灰/黑）时 H 交由模型保留
最近有效值（r2 规则）。全部中间量 <2^31。单返回值限制：多分量经编码
int（r·65536+g·256+b 等）或直接返回 HEX 串。

### 3.4 色板：canvas nodes + onhit（选定）

palette 由应用传入（任意 CSS 串）。chip 不能用动态 `bg-[…]` class
（vue Tailwind JIT 扫不到运行时拼接类——031-paint 头注实证）；选定
canvas nodes + `onhit(id)`（PLAN-661 命中契约，双端交付）。选中态用
第二轮描边节点（labels/edges 组合）表达。

### 3.5 通道输入：PLAN-087 NumericField（依赖已交付）

precision=0、min/max 通道界、step=1；HEX 用普通 input + 自校验。087
已归档（r3 双轨全绿），无 stub。

### 3.6 探针执行记录（结论：双轨全通过，色面可行）

探针工程：`examples/ui/088-probe/`（临时，T-04 前删除；不属交付面）。
驱动脚本：`D:/autostack/plan-artifacts/plan-088/probe_native.py`（scratch）。

**native（Windows，auto.exe -r vm + MCP，RESULT: PASS 13/13）**：
- `pen` 合成动作（`App␟SurfDown␟SurfMove␟SurfUp␟x,y`）驱动三画布：
  surf (96,95)⇒ s=50 v=50 hex=#405980CC；hue (8,6)⇒ h=15 hex=#805040CC；
  alpha (96,6)⇒ a=127 hex=#8050407F——与预计算 half-up 引擎值逐字节一致。
- 色板 `press value=p0` ⇒ `p0=#ef4444`（onhit 命中）。
- Dialog 开合渲染 canvas 内容正常；截图
  `plan-artifacts/plan-088/probe-out/probe-native-final.png`。

**vue（Windows，vite + Playwright 真实鼠标）**：
- 5 画布渲染正确（SV 面/色相条/棋盘 alpha 条/棋盘预览/色板），CSS 8 位
  hex 半透明在棋盘上正确透出；截图 `probe-vue.png`。
- 真实鼠标拖拽 SV 面 ⇒ s/v/hex 更新 + surf-up + 日志；点击色板 ⇒
  `pal=p0=#ef4444`（`probe-vue-interact.png`）。

**探针期发现并处置的平台缺陷/约束（全部有实证）**：
1. canvas 场景色串 8 位 hex alpha 双缺陷——已修（§3.1，两处）。根因补充：
   `Color::Hex(u32)` 丢失 6/8 位形态信息（"#00000005"≡"#000005"），修复
   必须按**串长**判别，不能基于 u32 值域；首轮 u32 值域判别版被探针像素
   采样（行色 rgb(0,0,alpha) 不透明）证伪后改为串长判别。
2. `.split()` 产 List<Str> 经 `read_state_as_vec` 读回为垃圾
   `Value::Int`（VM 堆 List 债，P553-D1 同族；探针 §5 输出
   `pn first=Int(-156)` 实证）——**场景表一律 push 构建**（push 项以 Str
   读回，实证在案）；`.at` 侧规避，运行时根修留登记簿债务（owner: VM
   runtime，本计划不改 split 语义）。
3. vue 生成器对与 widget state 同名的模块 fn 形参/局部误加 `.value`
   （087 已录同类债务）：引擎 fn 形参/局部一律 `p` 前缀（`ph/psc/pv`…）
   规避；探针 `h=217 s=76 v=90 a=204 hex=#377AE6CC` 修复前后对照在案。
4. vue canvas 脚本无条件引用 `<prefix>_meta` 等全部五表——包/示例必须
   为每个 canvas 前缀声明 pts/meta/nodes/edges/labels 五表（native 侧
   meta/labels 可缺省）。进入 T-02/T-03 widget 设计约束。
5. `canvas` 场景此前无任何 `.at` 消费者（grep 实证）；本探针为首个真实
   整链样例（源→生成/解释→场景→渲染→事件→消费）。

**颜色语义注记**：hsv(217,76,90)=#377AE6（引擎逐字节核对）；计划示例色
#3974E6CC⇒RGBA(57,116,230,204) 属 hex 解析方向（V-06），由 hex 解析器
直接保证，与 HSV 引擎初值选样无关（r2 无矛盾）。

## 4. dependencies

| 依赖 | 状态 | 证据 |
|---|---|---|
| PLAN-087 numeric-fields（评审交付+固定版本） | 已归档交付 | `docs/plans/archived/087-numeric-and-unit-fields.md` r3（T-01…T-08 全闭环，native 20+1not_run / vue 21 绿；Spec 45a80058 落主） |
| canvas 场景契约（PLAN-563/661 已合并） | 在库 | `crates/auto-ui-runtime/src/ui/view.rs` Canvas/MouseArea；`aura_view_builder/mod.rs` parse_canvas_* |
| runtime 最小适配（本计划） | 已实施待验证 | subscriptions.rs `canvas_css_color`、native_projector.rs `css_color_tiny` 8 位 hex alpha |
| `auto` CLI（auto-man @ 11fe79c） | 组内构建 | `.wt/ui-088/auto-man/target/debug/auto.exe`（构建中） |

## 5. runners

- native（Windows）：`auto.exe run -r vm`（组内 `.wt/ui-088/auto-man` debug
  构建）+ `auto-ui-tool.exe`（`.wt/ui-088/auto-ui/target/debug/`），MCP 驱动
  （`autoui_snapshot/type/action/keyboard/screenshot`），门脚本沿
  `--auto-bin/--ui-tool-bin/--out-dir/--backend native` 接口（T-05 落地
  `tests/product/verify_color_picker.py`）。
- vue（Windows）：同示例 `auto run` + Playwright（真实浏览器拖拽/截图）。
- Linux 轨：本会话 Windows 主机，Linux 门在合并后回归批执行（not_run 如实
  记录，不视为已验收）。
- 占用观察：构建期间另代理在 `experimental/ace` 跑 release 构建（独立
  target 目录，无锁冲突）；已记录，不影响本计划 debug 轨。

## 6. next

- T-01 关闭：双轨探针 PASS（§3.6），放行 T-02/T-03。
- T-02/T-03 已实施并入库（分支提交 9c37d07a、06b92b0b）：包/引擎/widget/
  示例/工具门全链在案；palette/pen/HEX/dialog 提交链在 native MCP 与 vue
  Playwright 双轨实证。
- **T-03 通道输入设计被平台缺口阻塞（needs_replan 依据，实证 §7）**：
  depth-3 组件实例（App→ColorPicker→NumericField）的 Tick/实例状态解析
  失效——Tick 以根态运行每 100ms×8 实例刷 `Field 'revision' not found`
  （会话实录数千次），字段外部重投影不可用。修订方向（同目标/验收等价）：
  通道输入改为内置 input + cp 引擎夹取校验（Enter 提交一次/同值零事件/
  非法具名拒绝——087 语义基准），不嵌 NumericField 实例。
- 未决机制：§7 depth-3 缺口（VM runtime 债，owner: VM runtime）。

## 7. depth-3 组件实例缺口（T-03 实证，needs_replan 依据）

- 形态：App → ColorPicker（包 widget，depth-2 正常）→ NumericField
  （PLAN-087，depth-3）。
- 实证（`plan-artifacts/plan-088/probe-out/demo*.log`）：每 100ms
  `[VM-HANDLER] NumericField.Tick failed: Field 'revision' not found on
  type instance App_State`（单会话 894～11234 次）；8 个存活实例身份在册
  （`demo@//ColorPicker#0/NumericField#0..3` 等），但根 Tick 广播以
  instance=None 入 `on_with_input_for` → ≥2 存活实例歧义 → 根态回退 →
  读 props `.revision` 即错。depth-2 对照：087 demo（App→NumericField）
  同构建零失败（`nf87.log`）。
- 影响：depth-3 字段的外部重投影（Tick）不可用、错误风暴、会话不稳；
  通道框在 native 轨不可交付。palette/pen/HEX/dialog 提交链（depth-2
  picker 事件）不受影响，已双轨实证。
- 处置：本计划内不修 VM 实例语义（爆炸半径大）；按 needs_replan 做有界
  修订（通道输入内置化），缺口登记为 VM runtime 债（owner: VM runtime，
  关闭条件：depth-3 实例 Tick/状态解析正确后包内可嵌 NumericField）。

### 7.1 depth-2 实例 handler 的字段读错位（r3 门实测补充，同族证据）

- 修正记录：r3 门首輪「native 状态别名」结论部分源于门脚本 canvas 索引
  错置（hue=1/alpha=2 写反）；索引修正后 **hue+surface 链精确通过**
  （`ink 接受 #805040CC`，gate-native 报告 [4]）。
- 仍存在的缺口（序列相关、可复现）：Tick 重基线（外部 value/revision
  推进 → DECODE 重写 d_* 全字段）之后，下一个实例作用域 handler 对部分
  字段的读返回**默认值或相邻字段值**：alpha handler 读 `.d_s`=0（模型
  默认）⇒ 提交 `#8080807F`；channel handler 读 `.d_g` 得 `.d_b` 的值
  （`#40E6E67F`）。同会话中 hue+surface（同 handler 族、同状态对象）精
  确通过——错位随特定重播种/追加序列出现，与 ensure_child_instance 的
  字段追加序（props 重播种 push 路径）和 handler 编译期字段索引之间的
  一致性一致（VM runtime 定位面）。
- 影响：依赖 Tick 重基线后继续编辑的入口（alpha/通道框）在 native 轨
  不可靠；不依赖重基线时序的首笔编辑（hue+surface、palette、HEX 首提
  交、R 单字段）已双轨实证通过。
- **终诊（门三轮实证，索引修正后稳定复现）**：这不是 Tick 时序问题——
  同一门序列内，**前一个实例作用域 handler 的整型字段写对后一个
  handler 不可见**：surface handler 写 `.d_s=50`（提交 `#805040CC` ✓
  自己的读正确），紧随的 alpha handler 读 `.d_s`=0（模型默认）⇒
  `#8080807F`；channel case 同律（`#40E6E67F`，g/a 读陈旧）。字段写
  可见性仅限 handler 自身 → 实例状态对象的写传播/读一致性缺口（VM
  runtime）。d_hex（串字段）的写反而可见——整型 vs 串的绑定路径不同。
  定位面：bridge 实例状态对象的 int 字段写回（nanbox/绑定腐化家族，
  PLAN-043 T9 +0.001 绕道即同族先例）。
- 处置：并入 §7 同一 VM runtime 债（实例状态字段索引一致性），owner:
  VM runtime；PLAN-088 通道入口的 native 证据在其修复前无法诚实出具。
- 附带运行时修复（本计划内、已验证）：pen/mousemove/hit 处理器
  TypedInst 基座坐标/ID 追加 + MCP press/pen 按元素解析组件作用域派发
  （此前嵌套组件合成动作静默丢弃）；canvas 场景 8 位 hex alpha 按**串
  长**判别（Hex(u32) 丢形态信息）。

## 8. r3 解除阻塞（2026-11-17，goal 续跑；撤回 §10 二选一）

- 裁定路径：不立 VM runtime 修复计划、不降低验收域。做法是在计划范围内
  以「串字段单一真源 + 每 handler 入口解码」替代跨 handler 整型草稿状态
  （draft 不再有 `d_r…d_v` 整型字段；`d_hex`/`ch_*`/`h_hold` 串字段跨
  handler 可靠），并修复下列自含实现缺陷后，native 通道/alpha 证据成立。
  §7.1 的整型字段写可见性观察随之不再位于关键路径（债仍登记，未宣称修复）。
- 实测根因（native 黑块/别名值的直接原因，均已在包内修正）：
  1. **HSV 解码饱和度系数 ×10**：`((hsvv - hsvv%1000)/1000) -
     ((hsvv - hsvv%100000)/100000) * 1000` 应为 `* 100`（51 处）。h≥1 时
     `ld_sat` 为负 → `cp_hsv_hex` 夹到 0 → 灰度提交（H=15 提交 #E6E6E6）。
  2. **行/alpha 条高位半字节缺 `/16`**（`cp_row_item`/`cp_alpha_item`）：
     aj/ai≥0xA0 时输出 7 位色串（如 `#000000A`），runtime 的 8 位 alpha
     判别回退 `BLACK` → 色面下行全黑、alpha 条右半黑。
  3. **rgb6 截取**：`d_hex.substr(1, 7)`（7 字符）→ 9 位色串非法 → 黑；
     改 `substr(1, 6)`（28 处）。
  4. **ColorDialog 缺外向事件声明**：msg 未声明 `"preview"/"committed"/
     "cancelled"/"error"`，handler 内 `."cancelled"()` 未被改写为 emit 桥，
     落 `CALL_SPEC no function 'App.cancelled'`（取消不关闭、父 handler
     不触发）；补声明后 dialog 取消/确认/关闭/重开全通。
  5. **通道错误呈现**：补 `.ch_err` 文本节点；committed 错误路径发
     `error(invalid_channel)`，HEX Enter 非法发 `error(invalid_color)`。
  6. **vue 轨重复 `let`**：同一 handler 第二解码块与首块同名声明（23 处）
     → `Identifier 'ld_r' has already been declared`；第二块改赋值。
  7. **初始焦点=HEX**：平台无 widget 级 autofocus 通道（iced 焦点由点击/
     update 臂），未强制；Dialog 沿用既有 modal 焦点环与关闭恢复。记为边界。
- 证据（代码提交 `plan-088 r3 复跑`）：
  - native：`gate-native-r3t/report-native.json` → `result: pass`（r3r 为中间失败轮，仅诊断用）；用例
    V-01/06 known-colors（含灰草稿保留最近 H、alpha 141）、V-02
    gestures+外部重置、V-01 channel（#4074E6CC + invalid_channel 事件）、
    V-03 双实例/业务拒绝、V-02/03 dialog（取消零提交/确认一次）、V-04
    Paint 消费、V-05 native 640x560 窄窗 + 浅深主题 + 退出重开；visual
    `demo-native.png`/`demo-narrow-native.png`/`demo-native-dark.png`。
  - vue：`gate-vue-r3g/report-vue.json` → `pass`（真实鼠标拖拽 + 通道
    提交 #4074E6CC + 1100x800/420x560 截图）。
  - 工具门：`cargo test -p auto-ui-tools --test color_picker_component`
    2 passed（7.02s）。
  - 视觉抽检：色面二维梯度（白→色→黑）、alpha 条棋盘→实色、色板/预览
    正常；窄窗（640x560）通道标签与输入可达。
