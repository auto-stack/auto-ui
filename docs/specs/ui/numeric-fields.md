# 数值与单位输入（numeric-fields）

> PLAN-087 新增合同。状态：拟议（随 Plan 交付沉淀，merge 落主）。
> 分类：ui ｜ 来源计划：docs/plans/087-numeric-and-unit-fields.md

## 1. 合同

官方包 `packages/numeric-fields/` 提供 `NumericField` / `UnitField` 两个高层
受控输入。数值全程十进制定点（`(sign, int_part, frac6)` 三分量 i32），无浮
点参与；native VM 与 vue 双轨逐位一致。单位换算为字符串级小数点移位。

- **NumericField**：value/revision 受控；precision 0..6 量化（最近值、半幅
  远离零）；min/max/step 须能以 precision 精确表达；整数段 ≤9 位、小数段
  ≤6 位 profile，超界具名拒绝（`invalid_number`），绝对值上界
  999999999.999999；量化后越上界具名 `overflow`。
- **UnitField**：基准单位 canonical；单位表固定 length（mm×1/cm×10/m×1000）、
  time（ms×1/s×1000）、px/percent（单单位组）；切单位仅改显示
  （`unitChanged` 零 value 提交）；显示输入按所选单位换算回基准后以基准
  precision 量化一次；步进增量按基准单位；显示去尾零且至少保留 precision 位。
- **事件**：`"preview"/"committed"/"unitChanged"/"cancelled"/"error"`（引号
  变体，载荷 JSON `{seq, baseRevision, …}`）；同值提交零事件；Enter 后的
  同值重提交不重发；步进夹取边界后同值零事件。
- **校验顺序**：配置 → 语法/profile → 未量化 min/max → 量化 → 范围复核 →
  请求（precision=2、max=1 时 1.004 拒绝，不先舍入）。
- **业务拒绝**：调用方不推进 value/revision 即拒绝；errorMessage 反馈；
  组件不把请求当成功，正式值不被覆盖；旧 draft 由外部 value/revision
  变化撤销（Tick ≤100ms 重投影）。

## 2. 验收锚点

| ID | 锚点 |
|---|---|
| V-01 | 真实控件步进/提交/具名拒绝（1.20+0.10⇒1.30 精确；abc/10位整数/7位小数拒绝且基准不变） |
| V-02 | 单位切换零 value 提交；显示单位编辑换算回基准（21.5cm⇒215.00mm） |
| V-03 | 同值幂等；外部更新撤销旧草稿；双实例独立；Escape 撤销草稿（vue 逐元素；native 单实例语义） |
| V-04 | 普通设置表单实际消费：提交值=应用读回值；业务拒绝保留正式值 |
| V-05 | Win native + Win vue 真实窗口/浏览器；浅深主题、420×560 窄窗、正常退出重开 |
| V-06 | 1.004/max1 先验拒绝；±1.235 半幅远离零；上界 @p6 可提交 @p2 overflow；1250.00ms+1ms⇒显示 1.251s |

门：`python tests/product/verify_numeric_fields.py --auto-bin … --ui-tool-bin …
--out-dir … --backend native|vue`（Linux 轨 not_run 时相应能力不视为已验收）。

## 3. 平台与消费边界

- native input 无 onblur：双轨统一 Enter/步进提交；失焦不丢草稿。
- native 字段级键绑定为全局首见优先（EDGE-01 合并表）：Escape 撤销为绑定
  归属实例语义，多实例表单不承诺字段级路由；箭头键不绑定。vue 逐元素路由。
- MCP 多字段表单：键盘 Enter 路由第一个 input 声明；非首字段提交用
  `autoui_action action='submit'`。
- vue 模块 fn 形参名不得与消费方 widget state 同名（会被误加 `.value`）；
  本包形参已用低碰撞名，编译器级参数遮蔽修复为通用债务。
- 原内置 `number_field`/`number_input` 声明族不受本包影响
  （Builtin > Local > Package 优先级未动；包名 `NumericField`/`UnitField`
  不折叠碰撞任何 builtin 键）。
