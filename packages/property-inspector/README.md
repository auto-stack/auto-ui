# property-inspector（PLAN-092）

AutoUI 官方数据驱动属性/设置面板包：`PropertyInspector`。应用以记录
（fields/values/groups/fieldErrors）声明字段与受控值，组件按 `kind` 组合
标准编辑控件（087 NumericField/UnitField、088 ColorPicker、内置
input/checkbox/chip 按钮），全部变更以带完整目标身份的
`changeRequested` 请求交回应用裁决。组件不写对象路径、不 eval 消息名、
不拥有应用状态或撤销历史。

## 快速消费

```at
use auto.json

widget Settings {
    msg { OnChange(str), OnPreview(str), OnPreviewCancelled(str), OnErr(str) }
    model {
        var fields List = []
        var values List = []
        var groups List = []
        var ferrs List = []
    }
    use { package: property_inspector from "<本包目录>" }
    use { package: numeric_fields from "<numeric-fields 包目录>" }
    use { package: color_picker from "<color-picker 包目录>" }
    view {
        PropertyInspector (fields: .fields, values: .values, groups: .groups,
            fieldErrors: .ferrs, targetId: "reading", revision: 1, schemaRevision: 1,
            onchangeRequested: .OnChange($event), onpreviewRequested: .OnPreview($event),
            onpreviewCancelled: .OnPreviewCancelled($event), onerror: .OnErr($event)) {}
    }
    on {
        .OnChange(ev str) -> {
            var req = json.parse(ev)
            // req: {seq, baseRevision, schemaRevision, targetId, fieldId,
            //       kind, value[, unit], source}
            // 受控回路：校验 baseRevision/targetId 后原子更新 values/revision；
            // 拒绝时只推送 fieldErrors（{targetId,fieldId,baseRevision,seq,code,message}），
            // 正式 values 不变
        }
    }
}
```

**依赖声明（关键）**：消费方 pac.at 须 `dep` 全部三个包；消费 widget 级
`use` 全部三个包（跨包 depth-2 实例解析所需，T-01 D-3）。可运行完整示例：
`examples/settings-inspector/`（native `auto run -r vm`；vue `auto run`，
先 `auto build` 物化 deps）。

## props（r2 合同）

| prop | 类型/默认 | 说明 |
|---|---|---|
| `fields` | `List = []` | 字段 schema 记录（见下）；重复 id/未知 kind/非法默认值 → 具名面板错误并禁用歧义编辑 |
| `values` | `List = []` | 受控值记录 `{id, value[, unit]}`；每字段恰好一个；bool 的 value 为 JSON 布尔 |
| `groups` | `List = []` | 分组 `{id, label}`；v1 为静态分组标题（交互折叠 deferred，B-6） |
| `fieldErrors` | `List = []` | 业务错误 `{targetId, fieldId, baseRevision, seq, code, message}`；仅匹配当前目标/版本的显示 |
| `targetId` | `str = ""` | 当前对象标识；随请求回传 |
| `revision` | `int = 0` | 受控值修订；变化撤销活动草稿/预览并重投影 |
| `schemaRevision` | `int = 0` | schema 修订；变化同上 |
| `disabled` | `bool = false` | 整面板禁用 |
| `readOnly` | `bool = false` | 整面板只读 |
| `showSearch` | `bool = true` | 搜索框（label/help 过滤，仅改变展示） |

字段记录公共成员：`id,label,kind,groupId,help,required,readOnly,disabled,
hasDefault,defaultValue`；按 kind 附 `precision/min/max/step`（integer 强制
precision=0）、`unitGroup/unit`（unit）、`options:[{id,label}]`（enum，default/
value 必须属于其中）。

## 首版 kind 与编辑控件

| kind | value 类型 | 编辑控件 |
|---|---|---|
| `string` | JSON 字符串 | 内置 input（点击进入编辑、Enter 提交、Escape 撤销） |
| `integer` | 十进制字符串 | 087 NumericField（precision=0） |
| `number` | 十进制字符串 | 087 NumericField |
| `unit` | 十进制字符串 + `unit` 成员 | 087 UnitField（单位切换走 changeRequested，value 不变，决策 U-1） |
| `bool` | JSON 布尔 | 内置 checkbox |
| `enum` | 候选项 id 字符串 | chip 按钮组（T-01 D-2：selectitem 无 native 实现） |
| `color` | `#RRGGBBAA` | 088 ColorPicker |

## 事件（msg 引号变体，载荷 JSON 字符串）

| 事件 | 载荷 | 触发 |
|---|---|---|
| `"changeRequested"(str)` | `{seq, baseRevision, schemaRevision, targetId, fieldId, kind, value[, unit], source}` | 用户结束一次编辑/恢复默认；source ∈ `edit`/`reset` |
| `"previewRequested"(str)` | `{targetId, fieldId, kind, value}` | 数值/颜色合法草稿（不写文档） |
| `"previewCancelled"(str)` | `{targetId, fieldId}` | 目标/修订变化或子件取消使预览失效 |
| `"error"(str)` | `{code, message[, fieldId]}` | 面板 schema 错误（每次配置变化一次）+ 子件草稿错误透传 |

面板错误码：`duplicate_field_id` / `unknown_kind` / `invalid_values` /
`invalid_default`。字段级：`invalid_value`（enum 不在候选/色值非 8 位 HEX）。
业务拒绝由调用方经 fieldErrors 反馈，正式 values/revision 不动。

## 语义要点（T-01 实证）

- **受控回路**：编辑先由子控件完成草稿验证，再发含完整目标身份的请求；
  调用方按 baseRevision 裁决，原子更新 values/revision。拒绝不覆盖正式值。
- **目标/修订换代**：targetId/revision/schemaRevision/fieldErrors 变化撤销
  活动草稿与预览并重投影；同 label 不同 targetId 不串写。
- **恢复默认**：逐字段入口，`source=reset`，默认值经同一类型验证；
  `hasDefault=false` 不显示恢复入口。
- **默认值判别**：`hasDefault` 显式位区分"无默认值"与 false/0/""。
- **子控件修订槽位**：行身份编码进子控件 revision（`(rev+1)*1000+行序`），
  因 native 子件事件仅 `$event` 载荷可派发（B-5）。

## 平台边界（v1 实证）

- **B-5（VM runtime 债，088 §7 扩展）**：native 上父件为包 widget 时，子件
  按钮的注入消息 delivered=0、嵌套 input Enter 不派发 committed。数值/颜色
  编辑事件矩阵由 vue 轨承担；错误事件路由正常；真实指针点击路径不受本包
  控制面影响。MCP 无人值守驱动该子面时 native 记 not_run。
- **B-6**：native 同页双实例（同 widget 类型）的面板行重投影不稳定——
  消费方按路由分离实例（示例即此形态）；行构造保持 handler 内联。
- 交互折叠 deferred（B-6 同源：动态记录成员的视图条件不重渲染）。
- 错误事件路由正常；Escape 撤销草稿（string，inspector 自有 input）可用。

## 依赖闭包

- `packages/numeric-fields/`（PLAN-087 交付 45a80058）
- `packages/color-picker/`（PLAN-088 交付）
- 本包 `inspector_model.at` 模块 fn（pi_schema_error/pi_value_error）随
  SFC 内联，无其他依赖。

## 支持矩阵（交付时实证口径）

| 轨 | Windows | Linux | macOS |
|---|---|---|---|
| native VM | ✅ 产品门通过（面板契约/同 label 双目标/搜索/恢复/业务拒绝/退出重开；子件写入面 not_run 见 B-5） | not_run（无 Linux runner） | 不宣称 |
| vue（浏览器） | ✅ 产品门通过（全 kind 渲染/枚举/bool/步进/边界/坏schema/窄窗浅深；playwright chromium） | not_run | 不宣称 |

证据：`docs/reviews/plan092/decisions.md` 与 `evidence.md`；原始 gate 输出在
`D:/autostack/plan-artifacts/plan-092/`。
