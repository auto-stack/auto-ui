# numeric-fields（PLAN-087）

AutoUI 官方数值与单位输入包：`NumericField`（十进制受控输入）与 `UnitField`
（带显示单位的受控输入，基准单位 canonical）。无浮点：引擎以
`(sign, int_part, frac6)` 三分量 i32 承载全部中间量，单位换算为字符串级
十进制小数点移位，native VM 与 vue 双轨逐位一致。

## 快速消费

```at
use auto.json

widget Settings {
    msg { SizeCommitted(str) }
    model {
        var size str = "16.00"
        var revision int = 1
    }
    use { package: numeric_fields from "<本包目录>" }
    view {
        NumericField (value: .size, revision: .revision, precision: 2,
            min: "8", max: "72", step: "1", label: "字号",
            oncommitted: .SizeCommitted($event)) {}
    }
    on {
        .SizeCommitted(ev str) -> {
            // 提交体 {"seq":N,"baseRevision":R,"value":"canonical","source":"input|step"}
            // 受控回路：按 baseRevision 验证后原子更新 value/revision。
            .size = json.parse(ev).value
            .revision = .revision + 1
        }
    }
}
```

vue 轨消费需在 `pac.at` 声明 `dep numeric_fields { path: "<本包目录>" }`
（auto-man 物化按 `<dep.path>/pac.at` 读取）。可运行完整示例：
`examples/numeric-fields/`（native `auto run -r vm`；vue `auto run`）。

## props（r2 合同）

| prop | 类型/默认 | 说明 |
|---|---|---|
| `value` | `str = "0.00"` | 受控基准值 canonical（UnitField 为基准单位） |
| `revision` | `int = 0` | 调用方修订；变化经 Tick（100ms）撤销旧草稿重投影 |
| `label` | `str = ""` | 字段标签（空则不渲染） |
| `precision` | `int = 2` | 0..6；提交量化位数（非法配置具名锁定） |
| `min` / `max` | `str = ""` | 空为无边界；须能以 precision 精确表达 |
| `step` | `str = "1"` | 步长（基准单位）；>0 且同上可表达 |
| `disabled` | `bool = false` | 禁用（移除全部修改动作） |
| `readOnly` | `bool = false` | 只读（输入与步进均锁） |
| `errorMessage` | `str = ""` | 调用方业务错误文本（独立于组件草稿错误） |
| `unitGroup` | `str = "length"` | UnitField：`length`/`time`/`px`/`percent` |
| `unit` | `str = "mm"` | UnitField：当前显示单位（受控） |

## 事件（msg 引号变体，载荷 JSON 字符串）

| 事件 | 载荷 | 触发 |
|---|---|---|
| `"preview"(str)` | `{seq, baseRevision, value, source:"input"}` | 每次合法草稿（不写文档） |
| `"committed"(str)` | `{seq, baseRevision, value, source:"input"\|"step"}` | Enter/提交（同值不发）；步进按钮 |
| `"unitChanged"(str)` | `{seq, baseRevision, unit}` | 仅 UnitField 切单位（零 value 提交） |
| `"cancelled"` | 无 | Escape 撤销未提交草稿（平台边界见下） |
| `"error"(str)` | `{seq, baseRevision, code, message}` | 草稿错误（同错误不重发） |

错误码：`invalid_number` / `out_of_range` / `invalid_config` /
`unsupported_precision` / `overflow`。可见错误文本与事件双通道呈现。

## 语义要点

- **校验顺序**：配置 → 十进制语法/profile（整数≤9位、小数≤6位）→ 未量化
  min/max 比较 → 量化（最近值、半幅远离零）→ 范围复核 → 构造请求。
  precision=2、max=1 时输入 1.004 拒绝（不先舍入成 1.00 冒充合法）。
- **同值抑制**：提交值与基准数值相等（文本不同亦同）零事件；步进夹取到
  边界后同值零事件。
- **业务拒绝**：调用方不推进 value/revision 即为拒绝；errorMessage 反馈；
  组件不把请求当成功，正式值不被覆盖。
- **UnitField**：单位表固定 length（基准 mm ×1/×10/×1000）、time（基准 ms
  ×1/×1000）、px/percent（单单位组）；步进增量按基准单位；显示去尾零且
  至少保留 precision 位（1250.00ms 按 1ms 步进后显示 1.251s）。
- **平台边界（r3 修订项）**：
  - native input 无 onblur —— 双轨统一 Enter/步进提交，失焦不丢草稿；
  - native 字段级键绑定全局首见优先、不区分实例 —— Escape 撤销为"绑定
    归属实例"语义（多实例表单不承诺字段级路由；单字段视图可用）；箭头键
    不绑定（多实例会串扰）；vue 逐元素可路由（Escape 用例在 vue 轨验收）；
  - v1 显示输入小数上限 6 位（超出具名 `invalid_number`，不静默截断）；
  - 引擎 fn 形参名与消费方 state 同名会在 vue 轨被误加 `.value`
    （T-02 实证）——包内已用低碰撞形参名（`pfix`/`u_shift` 等），
    编译器级参数遮蔽修复为通用债务。
- **集成约束**：MCP 驱动多字段表单时，键盘 Enter 路由到第一个 input 声明；
  非首字段提交用 `autoui_action action='submit'`（按元素路由）。
- **提交时序**：Enter 提交后组件不本地改 draft，等调用方接受（value/
  revision 推进）后由 Tick 重投影——拒绝场景正式值天然保留。

## 依赖闭包

自足：仅标准 `input`/`button`/`text` + 本包引擎（`numeric_decimal.at`，
模块级纯 fn，双轨内联）。无其他包/Plan 依赖。交付身份见
`docs/reviews/plan087/evidence.md`。

## 支持矩阵（交付时实证口径）

| 轨 | Windows | Linux | macOS |
|---|---|---|---|
| native VM | ✅ 产品门通过（含视觉/退出重开） | not_run（本组无 Linux runner，不冒 pass） | 不宣称 |
| vue（浏览器） | ✅ 产品门通过（playwright chromium） | not_run | 不宣称 |
