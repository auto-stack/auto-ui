# PLAN-087 T-01 决议 — 数值/单位输入的机制核对与API承接验证

日期：2026-11-16。基线：master `34bb4326`（Plan §4 观察 HEAD `93e6b924` 的相关事实复核一致；086 的 eval 补臂已在库上）。
工作树：`D:/autostack/.wt/ui-087/auto-ui`，分支 `codex/ui-087-numeric-fields`。
依赖组：`D:/autostack/.wt/ui-087/{auto-ui@34bb4326, auto-lang@107b23d0, auto-web@0adebc0, auto-down@5af11a3, auto-man@54fe407}` 全部零源码改动（detached master），组内同级布局使相对 path 依赖原样解析（同 086 布局）。
探针工件：`D:/autostack/plan-artifacts/plan-087/probes/`（t01-probe 应用 + probe_run/probe_v2 驱动 + 会话日志，仓库外 scratch）。

## 1. baseline（实际观察）

- 语言面：`auto_val::Value::Int` = **i32**（auto-val value.rs:78；runtime state_migration 对 I64/U64 静默截断，086 decisions §6 同记）。`eval.rs` 字符串方法面 = `len/starts_with/substr/trim/str/to_string`（byte 语义，ASCII 数字安全）；无 `index_of/contains/slice`。
- 条件比较：runtime `>`/`<` 按显示串 parse f64（eval.rs:1473）；vue 为 JS 原生——**字符串数值比较不入合同关键路径**，一切大小比较用整数。
- computed 快速路径算术：Sub/Mul/Div/Mod 臂已在（086 T-3 补丁 `54712797`，本基线含）。probe v1 实测 computed `1000000*7 / (9999999-9999999%1000)/1000 / 1000003%1000 / 5-9` = `7000000 / 9999 / 3 / -4` 双语义一致。
- MCP 面：`autoui_snapshot/type/action/keyboard`（mcp_server.rs）；`autoui_type` 全文替换触发 oninput；Enter 路由焦点 input 声明（"routed to focused input declaration"）。
- 双轨执行：native = VM 字节码（vm_bridge `call_vm_fn`）；vue = handler_codegen JS 直译。widget `model.interval` + `Tick` 双轨均有（dynamic.rs:212 / vue script.rs:22 `tick_interval`）。

## 2. 机制探针（native VM 实测，probe v1/v2）

| # | 机制 | 输入 | 实测 | 决议 |
|---|---|---|---|---|
| P1 | range-for | `for i in 0..5 { … }` handler 内拼串 | `0;1;2;3;4;` | ✅ 逐字符解析可行 |
| P2a | 裸标量 json.parse | `json.parse("7")` | **0**（静默） | ❌ 禁用 |
| P2b | 包裹对象 json.parse | `json.parse("{\"d\":7}").d` | 7；`{"d":-42}` → -42 | ✅ 备用数字桥 |
| P2c | `.to(int)` | `"7".to(int)` / `"-42".to(int)` | 7 / -42 | ✅ **主用数字桥**（vue 侧 = `Number(...)`，trans/javascript.rs:96）；只对已验证数字串调用（`"1.25"`/`"abc"`/`"0x.."` 行为不依赖、不入合同） |
| P3 | 除法截断 | `7/2`→3，`(0-7)/2`→-3 | Rust 语义 | ⚠️ **JS `/` 为浮点**：一切除法必须用精确整除形 `(a - a % b)/b`；负数靠显式符号分量避开 |
| P3b | 取模符号 | `(0-7)%2` → -1 | Rust=JS 同 | ✅ 分量恒非负，仅偶发用 |
| P4 | 串机制 | `"123.45".len()`→6、`substr(0,3)`→"123"、`"-1.5".starts_with("-")`→true | 全对 | ✅ profile/语法检查用 |
| P4b | 零填充 | `"00"+f"${5}"` 再 `substr(len-2,len)` | "05" | ✅ canonical 定宽输出用 |
| P5 | json 对象回路 | `json.stringify({seq:3,baseRevision:7,value:"1.30",source:"step"})` → parse `.seq` | 3；键序规范化 | ✅ 事件载荷用 |
| P6 | input 输入 | `autoui_type '1.25'` | oninput 全文 `1.25`（每次变更全文）；清空发 `""` | ✅ 草稿主通道 |
| P6b | Enter | MCP Enter | onenter 派发（焦点 input 声明路由） | ✅ 提交主通道 |
| P7 | 字段键绑定 | `onkeydown.up/down/escape.prevent` | `ArrowUp`→UpKey、`ArrowDown`→DownKey、`Escape`→EscKey 全派发 | ✅（限额见 §4） |
| P8 | Init 链调 | `.Init -> { .Run }` | **不执行**（handler 非可调用体） | ❌ Init 自含初始化体（086 同形） |
| P9 | Int 宽度 | 源码 i32 | — | ⚠️ 全量定点 1e15 溢出 → **(int_part, frac_part) 双分量十进制定点** |

## 3. 表示与算法决议（承接 §5 r2 profile）

- **状态内不存 float**：数值全程 `sign`(bool) + `int_part`(0…999999999) + `frac6`(0…999999, 6位定点) 三分量；全部中间量 ≤ 9.99e8 < i32 上限。溢出检查在乘法/进位**之前**按 profile 上界比较（×10→int_part>99999999 拒、×100→>9999999 拒、×1000→>999999 拒），拒绝即 `overflow`。
- **解析**（纯串，无浮点）：可选 `+/-`；逐字符 range-for + `substr(i,i+1)`；`c=="."` 判小数点；digit 值 `c.to(int)`；整数段>9位或小数段>6位 → `invalid_number`（profile 上限拒，不截断）；分量构建 `int_part*10+d`、`frac6` 按位数 `*10^(6-d)` 补齐（乘法界内）。
- **比较**（min/max、step 边界）：先比符号，再比 int_part，再比 frac6——纯整数，双轨一致。
- **量化 precision p**：`pow = 10^(6-p)`；`q = (frac6 - frac6 % pow)/pow`（精确整除形）；`r = frac6 % pow`；`r*2 >= pow` → q+1（half away from zero，符号在分量外层）。q==10^p 进位 int_part+1（≤1e9，界内）。
- **步进**：step 解析为同型三分量；`value ± step` 按 frac6 对齐（取 max(frac6, step_frac6) 同标度后加减，进位入 int_part）；越界夹取到可表示边界；同值不发提交。
- **canonical 输出**：`f"${int_part}.${zero-pad(frac6_quantized, p)}"`；负号前置；数值相等文本不同不重发（对上一次 committed 值串比较）。
- **单位换算**（UnitField）：factor ∈ {1,10,1000}（mm/cm/m、ms/s）或 1（px/%单单位组）；显示→基准 = int_part×factor + frac6×factor 进位（先查上界再乘）；基准→显示 = 除以 factor 的精确整除形 + 余数降标；**全部整数**。
- **UnitField 显示精度**：factor=10^k 时显示可用 precision+k 位（≤9），去尾零但至少 precision 位；输入允许相应位数，先字符串十进制换算再按基准 profile/precision 校验量化。

## 4. 交互与平台边界（反例与修订项）

- **blur 提交**：native input **无 onblur**（tags_controls.rs convert_input 只有 onchange/onenter/disabled/password）；vue 有（shadcn_attrs `onblur`→`@blur`）。→ r3 修订：blur-commit 为 vue 轨增强；跨轨合同 = Enter（真实键或 MCP Enter）+ 步进按钮提交；失焦不丢草稿（保留编辑中状态）。
- **字段级箭头/Esc（native）**：机制存在（EDGE-01 element `onkeydown.*` → key_bindings，probe P7 实证单实例派发），但合并为**全局每 app 一张表、首见优先**（dynamic.rs:4338 `entry().or_insert`），多实例同 widget 时 handler 归属不区分实例。→ r3 修订：箭头步进/Esc 撤销按"单字段视图语义"交付（vue 逐元素可逐实例；native 多实例时不承诺字段级箭头路由），V-03 双实例键盘用 typing/Enter/按钮；步进按钮是双轨均真实的键盘/MCP 可达路径。合同不静默收缩：§5"上下键与按钮共用步进"中"上下键"标注平台边界（native 单字段、vue 全量）。
- **外部修订失效**：native 无 `watch`（watchers 恒空）；用 `model.interval` + `Tick`（双轨均在）对比 `.revision`/`.value` 与 seen 快照 → 撤销旧草稿、重投影。间隔 100ms（file-dialog 50ms 同量级；设置表单可接受）。
- **错误文本**：computed 内 f-string 插值 vue 轨生成 undefined（086 实录）→ 错误/单位等派生文本全部走 **handler 内 f-string 写 state** 或常量串；computed 只做纯整数/布尔投影。
- **负数字面量 prop**：VM 轨 prop 字面量负数静默失效（086 实录）→ min/max/step 负值测试一律走 state 绑定（真实消费形态）。
- **Init**：自含（不链调）；Init 内 emit 不派发（086 probe:no-err）→ 配置错误走可见文本，不走 error 事件。

## 5. 公共接口核对（§5 r2 表逐项）

| r2 合同 | 承接 |
|---|---|
| props `value:str/revision:int/label:str/precision:int/min,max:str/step:str/disabled,readOnly:bool/errorMessage:str` | ✅ widget prop 语法 `name: type = default`（086 踩坑形态） |
| UnitField 增 `unitGroup:str="length"/unit:str="mm"` | ✅ 同上；单位表固定（length 基准 mm ×1/×10/×1000；time 基准 ms ×1/×1000；px/% 单单位组） |
| 事件 `preview(str)/committed(str)/unitChanged(str)/cancelled/error(str)` | ✅ msg 引号变体声明（`"committed"(str)` 等；vue emit 重写只识别引号变体——086/file-dialog 同形）；载荷 JSON `{seq, baseRevision, value, source}` |
| 错误码 `invalid_number/out_of_range/invalid_config/unsupported_precision/overflow` | ✅ `{code,message}` 对象串；配置错误与草稿错误分状态 |
| 步进按钮/单位按钮/label/错误行 | ✅ 标准 button/text/input 组合（086 同构，无新 builtin） |

## 6. 依赖与 runner

- 无硬依赖 Plan/包（官方包自足：标准 input/button/text + 本包计算）；`packages/numeric-fields/` 需 `package.at`+`pac.at`（auto-man dep 物化按 `<dep.path>/pac.at` 读，086 实证）。
- native runner：`.wt/ui-087/auto-man/target/debug/auto.exe run -r vm` + `AUTOUI_MCP_PORT` + `.wt/ui-087/auto-ui/target/debug/auto-ui-tool.exe`（本组已构建，probe 实通）。
- vue runner：`auto run`（vite dev server）+ python playwright（本机可用，pagination 同路）；浏览器侧行为在 T-02/T-05 按 r3 接口实证（`@keydown` 透传、`.to(int)`=Number、Tick、computed/handler 对齐）。
- Linux native/Vue：本组无 Linux 环境实证 → 按 §10/计划如实记 not_run（不冒 pass）。

## 7. next（允许进入的任务）

- **T-02** 放行：按 §3 表示/算法与 §5 接口实现 `packages/numeric-fields/`（NumericField/UnitField + 计算模块），探针结论作硬约束（精确整除形、引号事件、Init 自含、错误走文本）。
- **T-03** 放行（依赖 T-02）：草稿/提交/取消/Tick 失效接线；键盘按 §4 边界。
- r3 合同修订项（§4 三条）随 T-02 提交一并落入 Plan revision（不改变目标/验收范围，仅平台边界显形）。

## 8. T-02/T-03 追加决议（2026-11-16，实施实证）

- **VM `substr` 语义 = (start, length)**（与 runtime eval.rs 视图快速路径的
  `[start, end)` 不同）：单字符提取须 `substr(i, 1)`。实证："16.00".substr(1,2)
  → "6."（probe v2/诊断）。引擎已按此编写；engine 内避免在 range 边界里调
  `.len()`（a2ts range 路径无 .len→length 映射——"digits.len is not a function"
  实录），先用局部变量承接。
- **提交时序重写绑定 state**：native submit 派发会把 input 绑定 state 重写为
  float 值（16.30 → "16.299999237060547"，f32 往返实录）——OnEnter 一律消费
  OnInput 保存的精确分量（ok_s/ok_ip/ok_f6），不重读 .draft。file-dialog 类
  非数值路径未受影响（解析失败保持字符串）。
- **vue 轨模块 fn 形参名 ≠ 消费方 state 名**：ts_adapter 对 state 名统一加
  `.value`，fn 形参同名会被误写（`p`/`klog` 实录为 undefined）→ 引擎形参定名
  `pfix`/`u_shift` 等低碰撞名；widget state `prec`。编译器级形参遮蔽修复记为
  通用债务（不在本 Plan 内改 ts_adapter 绑定语义）。
- **编译器最小适配（本 Plan 落地，仓库代码）**：
  1. `ts_adapter.rs` AuraTsContext.used_range 聚合 + finalize_with_range 内联
     range 助手（修复 vue 轨子件 Init ReferenceError——按钮全禁/单位表空实录）；
  2. `shadcn_attrs.rs` base_event_to_dom 从原始 base 剥 `on` 前缀（修复
     camelCase 事件名失配——`@unitchanged` vs emit('unitChanged') 实录）。
  回归：ts_adapter 30/30、shadcn_attrs 20/20、双轨产品门全绿。
- **多实例键盘路由**：EDGE-01 键绑定按 widget 声明级收集（实例不区分）——
  双 NumericField 实例下 Escape 派发到非预期实例（t02 实录），单实例正常
  （t03 实录：草稿恢复 5.00）。箭头键（会发 committed）不绑定；Escape 保留
  绑定并按"绑定归属实例"语义文档化。V-03 的 Escape 用例在 vue 轨逐元素验收。
- **MCP 非首字段提交**：`autoui_keyboard Enter` 恒路由第一个 input 声明
  （enter_dispatch_for_input）——多字段表单用 `autoui_action action='submit'`
  按元素提交（实证 .NumericField#1.OnEnter 正确路由）。
- **example 业务拒绝夹具**：演示窗「业务拒绝模式」= 业务侧受理暂停（无条件
  拒绝）——baseRevision 恒等于当前修订，"过期"判据永远不触发（首版演示逻辑
  缺陷已修）。
- **tool host 启动抖动**：门驱动在机器负载下偶发 "tool host closed the
  control stream"（启动期，与夹具无关）——Session/Browser 增加一次重启重试。
