# TagInput 受控标签输入（docs/specs/ui/tag-input.md）

来源为 PLAN-097 r2 的 SD-01 与 SD-02。实现见[正式包](../../../packages/tag-input/README.md)、
[普通设置消费者](../../../examples/tag-input/README.md)与 015-notes 迁移。

## 包及应用责任

正式包 `packages/tag-input` 提供 `TagInput`。调用方通过应用清单声明依赖，组件经
相对路径导入。组件保留局部草稿、候选、选中与防重复请求状态；调用方拥有业务数据、
权限、I/O 与执行效果（持久化）。公开消息沿既有包的可导出命名方式声明
（`"changed"(str)` / `"error"(str)`），由普通 `onchanged`/`onerror` 绑定消费。

## Props（冻结契约）

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `tags` | `List = []` | 受控已接受标签；去重保证唯一，作为稳定 chip 身份（不按重排索引） |
| `contextId` | `str = ""` | 业务目标身份；变化中止旧请求与草稿并展示新 tags |
| `revision` | `int = 0` | 受控目标修订；accepted 确认要求 revision 已上升 |
| `suggestions` | `List = []` | 静态候选：去重、排除已接受、ASCII 小写子串查询、保留提供顺序 |
| `maxTags` | `int = 0` | 上限；`0` 不限，`<0` 具名配置错误 |
| `disabled` / `readOnly` | `bool` | 禁用全部修改 / 允许查看但禁止增删 |
| `label` / `placeholder` / `errorMessage` | `str` | 展示与外部错误提示 |
| `response` | `str = ""` | 应用确认回执 |

## 消息与确认

- `"changed"(str)` 载荷：`{"instanceToken","seq","contextId","baseRevision",
  "action":"add"|"remove","tag","tags":[完整候选数组]}`。一次合法手势最多一条；
  父层拒绝前不乐观抹掉真实 chip。
- `"error"(str)` 载荷：`{"code","message"}`。具名配置错误（`invalid-max-tags`、
  `invalid-tags`——tags 含空值/重复）恰好一次通知并阻止相关修改。
- 确认：`response` 必须完整身份匹配（instanceToken+seq+contextId）；`accepted`
  还要求当前 props `tags` 等于本次候选数组且 `revision` 已上升（同快照发布），
  否则视为 `invalid_feedback`（显示诊断、保持外部值、不二次写）。`rejected`
  保留草稿与原 tags。畸形响应不结算。
- 实例身份：每个实际挂载实例分配不复用的 `instanceToken`；普通重绘保留，
  卸载重建/另一实例不同。所有请求与 response 均带该字段。

## 交互与校验

- 新增文字只裁剪首尾空格/Tab/CR/LF；非空才可加；裁剪后区分大小写精确去重，
  不做 Unicode/case 归一化。父层传入空值/重复值为 `invalid-tags`（error 通道）。
- Enter 添加活动候选或当前草稿（活动候选须匹配当前草稿查询，组合输入未结束时
  不添加）；Up/Down 移动候选不提交；Escape 只收起候选；Tab 不创建；空草稿
  Backspace 一次移除最后标签（受请求锁保护）。
- 每个 chip 有名称明确的删除 Button；`disabled` 禁用全部修改；`readOnly` 允许
  查看但不删除/添加；上限阻止新增但仍可删除（新增不发 changed 并显示具名提示）；
  超过上限提示配置冲突但保留删除，不截断原值。
- 空白、重复、超上限均保留输入并显示输入邻近错误；逗号和粘贴默认一个完整标签。

## 消费方式与依赖闭包

1. 消费方 `pac.at`：`dep tag_input { path: "…" }`。
2. widget 内：`use { package: tag_input from "…" }`（相对声明文件解析；native VM
   sweep 与 vue Phase-1c 传递装载）。
3. 受控回路：`changed` → 应用核对 contextId/baseRevision 与业务数据 → 单次写入
   （Notes：`update_tags` 一次手势一次）→ 同快照推进 tags/revision 并回 accepted；
   业务拒绝回 rejected。

移根/vendor：整目录（`package.at`/`pac.at`/`tag_input.at`/`tag_text.at`）复制到
第二绝对根并保持字节一致；缺依赖在 build 轨具名失败。

## 交付门（SD-02）

- Win native VM：真实窗口、MCP 快照/操作、增删/去重/上限/拒绝/候选键盘/配置错误、
  应用读回与退出回收（`tests/product/verify_tag_input.py --backend native`）。
- Win Vue：vite 开发构建（`tag_input_preview.py` 收据）+ 真实浏览器
  （Playwright MCP 提供方）同矩阵。
- Notes 迁移：015-notes 编辑区真实标签写入、切换笔记读回一致、正文摘要不变。
- tools 门：`cargo test -p auto-ui-tools --test tag_input_component`（模型反例 +
  移根生成）。
- Linux native/vue 轨按 084 归档 §10 用户决定走 VMware 路由；无 runner 记
  `not_run`，相应能力不宣称已验收。macOS 未承诺。
