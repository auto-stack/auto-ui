# 属性与设置面板（property-inspector）

> PLAN-092 新增合同。状态：随交付沉淀（merge 落主）。
> 分类：ui ｜ 来源计划：docs/plans/092-property-inspector.md

## 1. 合同

官方包 `packages/property-inspector/` 提供数据驱动的受控属性/设置面板
`PropertyInspector`：应用以记录声明字段与受控值，组件按 `kind` 组合标准
编辑控件，全部变更以带完整目标身份的请求交回应用裁决。组件不写对象路径、
不 eval 消息名、不拥有应用状态或撤销历史；native VM 与 vue 双轨同义。

- **受控面**：`fields/values/groups/fieldErrors` 全部为调用方 List 记录，
  需由调用方原子发布；`targetId/revision/schemaRevision` 标识当前对象与
  版本，变化撤销活动草稿与预览并重投影；空 schema 为正常空态。
- **字段记录**：公共成员 `id,label,kind,groupId,help,required,readOnly,
  disabled,hasDefault,defaultValue`；按 kind 附 `precision/min/max/step`
  （integer 强制 precision=0）、`unitGroup/unit`（unit）、
  `options:[{id,label}]`（enum，default/value 必须属于其中）。重复 id、
  未知 kind、缺值、非法默认值给具名面板错误（`duplicate_field_id`/
  `unknown_kind`/`invalid_values`/`invalid_default`）并禁用歧义编辑；
  `error` 事件每次配置变化至多一次。值类型不合法（enum 不在候选、色值非
  8 位 HEX）为字段级 `invalid_value`，不转文本绕过。
- **kind→编辑控件**（全部真实子控件，无自有数值/选色实现）：
  `string`→内置 input（单活动编辑、Enter 提交、Escape 撤销）；
  `integer`/`number`→087 NumericField；`unit`→087 UnitField；
  `bool`→内置 checkbox；`enum`→chip 按钮组（候选项 id 为值；
  selectitem 无 native 实现，见 T-01 D-2）；`color`→088 ColorPicker。
- **值类型**：bool 的 value 为 JSON 布尔；integer/number/unit 为十进制
  字符串；enum 为候选 id 字符串；color 为 `#RRGGBBAA`；`hasDefault`
  显式位区分"无默认值"与 false/0/空字符串。
- **请求合同**：`changeRequested` 载荷
  `{seq, baseRevision, schemaRevision, targetId, fieldId, kind,
  value[, unit], source}`；source ∈ `edit`/`reset`。业务拒绝由调用方经
  `fieldErrors`（`{targetId, fieldId, baseRevision, seq, code, message}`）
  反馈，只匹配当前目标/版本，正式 values/revision 不动；恢复默认走
  `source=reset`，无默认值不显示恢复入口。`previewRequested`/
  `previewCancelled`/`error` 为预览与错误面；预览不写 Storage。
- **搜索/折叠**：搜索按 label/help 过滤且只改变展示，不产生业务事件；
  v1 分组标题为静态展示——交互折叠依赖 VM 视图差分修复（B-6/D-10，
  见 `docs/plans/debts.md`），deferred 期间折叠语义以"组不收起"满足
  不误写约束。
- **同 label 多面板**：字段按 `targetId` 绑定，不同面板互不串写；迟到
  请求/错误不落新目标。

## 2. 平台边界（交付实证口径）

- **native 子件写入面（B-5，VM runtime 债）**：父件为包 widget 时，
  子控件按钮注入消息 delivered=0、嵌套 input Enter 不派发 committed；
  错误事件路由正常。数值/颜色编辑事件矩阵由 vue 轨承担。
- **组级结构重渲染（B-6，VM runtime 债）**：同类型多实例面板行重投影
  不稳定、组级结构变化不重渲染——消费方多实例按路由分离（示例即此
  形态）；行构造保持 handler 内联。
- 视图条件串 `.contains` → `.includes` 已在 vue 条件后处理映射（B-7）。

## 3. 验收锚点

| ID | 锚点 |
|---|---|
| V-01 | 六 kind 由 record 渲染（native+vue）；087/088 子控件组合；unit kind 单位表 |
| V-02 | enum chip/bool 布尔/string 草稿提交各走 changeRequested；业务拒绝保留正式值；搜索只过滤展示 |
| V-03 | 双目标（reading/editor）同 label 不串写；迟到请求不落新目标 |
| V-04 | 普通设置示例预览实际变化（字号/行距/对齐/文本色/提示）；Storage set→get 读回一致；退出重开渲染 |
| V-05 | Win native（MCP 真窗口）+ Win vue（chromium）；明暗/420 窄窗实拍；owned 进程回收 |
| V-06 | 默认 0/默认 false 恢复为精确类型；hasDefault=false 无恢复入口；只读可读；坏 schema 具名拒绝 |

门：`python tests/product/verify_property_inspector.py --auto-bin …
--ui-tool-bin … --out-dir <隔离目录> --backend native|vue`；
`cargo test -p auto-ui-tools --test property_inspector_component`。
交付证据：`docs/plans/archived/092-property-inspector.md` §9 与
`docs/reviews/plan092/{decisions,evidence}.md`。

## 4. 支持矩阵（交付实证口径）

| 轨 | Windows | Linux | macOS |
|---|---|---|---|
| native VM | ✅（面板契约/双目标/搜索/恢复/拒绝/重开全绿；子件写入面 B-5 not_run） | not_run（无 runner，不冒 pass） | 不宣称 |
| vue（浏览器） | ✅（全矩阵含步进/坏schema/窄窗） | not_run | 不宣称 |
