# PLAN-097 实施决策与探针

当前合同为 r2。实施工作树 `D:/autostack/.wt/ui-097/auto-ui`，分支
`codex/ui-097-tag-input`，基线 `ad62e15430ccb717fe6f7e06b2b26970f7811d45`。

## 依赖与基线

| 来源 | 核对值 |
| --- | --- |
| auto-lang（编译依赖） | `5e5121dac6d6034a7eeb28cf4ed8d71686d21cd9`（组内兄弟工作树 `D:/autostack/.wt/ui-097/auto-lang`） |
| auto-web（编译依赖） | `0cfc7c78446650631d244a9ea442d28635072958`（组内 `…/auto-web`） |
| auto-down（编译依赖） | `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`（组内 `…/auto-down`） |
| auto-man（CLI） | `D:/autostack/auto-man/target/debug/auto.exe` SHA256 `eddb2cce50fd8afe08cd26959c2fe52e090eb833b5c8b9a3f27538b28d947fcf` |
| auto-ui-tool（本工作树构建） | `D:/autostack/.wt/ui-097/auto-ui/target/debug/auto-ui-tool.exe` SHA256 `8133d12583e8f10be75f74525e00b6ff5877bdc0f08bf3fe1baa9b5fc68a7484` |
| 工具链 | rustc 1.98.0，Node 25.2.1，Python 3.14.2，Playwright MCP 0.0.86 + Chrome |
| 015-notes `editor.at` | SHA256 `333C583788796F28B2E59345E59F1E32F0A32007CD5A961EE7B8AEEFCFF61265`（与计划 §4 一致） |
| 015-notes `notes_store.at` | SHA256 `26D7A1B989EE1FED2A4BE063A691623C7D767CF5DB02B066BED1E0C6DB2FEF5E` |
| 参考图 | SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`（1586×992） |

构建依赖经 `../../../auto-lang/…` 相对路径解析；工作树内解析到组内兄弟工作树
（`D:/autostack/.wt/ui-097/`），与主检出同级布局等价。

## 机制与探针结论

- **正式包**：`packages/tag-input/`（`package.at`/`pac.at`/`tag_input.at`/
  `tag_text.at`/`README.md`）。消费方 `pac.at` 声明 `dep tag_input`，widget 内
  `use { package: tag_input from "<相对路径>" }`（相对声明文件；native VM sweep
  与 vue Phase-1c 均传递装载）。`TagInput` fold 为 `taginput`，无内置名冲突。
- **实例身份**：`str.uuid()` 在首个 Tick 生成（Vue 降级为浏览器
  crypto.getRandomValues UUID）。普通重绘保留，卸载重建/另一实例不同。
- **初次配置错误**：从正常 Tick 派发（native 子组件 Init 内 emit 不派发——
  与 pagination/numeric-fields 既有记录一致）；`error(str)` 载荷
  `{code,message}`，签名变化时恰好一次。
- **响应确认**：`response` 必须 instanceToken+seq+contextId 全匹配；`accepted`
  还要求当前 props tags == 本次候选数组且 revision 已上升，否则 invalid_feedback。
  `rejected` 保留草稿与原 tags；畸形响应不结算。
- **Enter/键盘**：native MCP 的键盘 Enter 经 `find_first_input_vnode` 路由到
  **树中首个 input**（运行时已知行为，非焦点 input）；多实例场景测试用带目标的
  按钮/候选点击，真实 OS 焦点输入不受影响。Enter 路径在首个 input 上单独验证。
- **运行时的交付时序抖动**（T-01 实证，非组件缺陷）：
  1. **MCP 快照调用会干扰输入状态**：type 与 Enter/按钮提交之间夹一次 snapshot
     会清空草稿/使提交失效；测试模式为 type→提交前不插快照，失败重试有界。
  2. **handler 内读取 List prop 存在时序抖动**（视图读取可靠、handler 偶发读到
     挂载默认/空值）；组件侧规避：Tick 内以循环计数、Handler 内不依赖 prop 直读
     做关键判定（计数/去重以循环为主），产品测试带前置状态检查与重置修复。
  3. **草稿输入后候选未及重算**：Enter 曾提交过期活动候选；组件增加
     `tag_contains(.active_tag, .draft)` 守卫，仅当活动候选匹配当前草稿查询时采用，
     否则回落草稿（本批工具测试覆盖）。
- **.at 语言面发现**：
  1. `req.tag` 是软关键字（TaggedCell.tag 特殊语法），JSON 对象字段 `tag` 须用
     `req["tag"]` 读取（Vue 端点号访问不受影响）。
  2. VM handler 内 JsonValue 方法调用（`as_string`/`get`/`keys`）不可靠
     （CALL_SPEC ''）；字段比较与 `json.get(req, key)` 为可用面。契约载荷字段名
     保持冻结，消费方按上述可用面读取。
  3. `str.char_at` 返回码值（拼接产出数字串）；字符遍历用 `substr(i, 1)`
     （与 prompt_trim 同款）。
  4. List prop 的 `len()` 在 handler 内不可靠；用循环计数。
- **候选/上限语义**：静态 suggestions 去重、排除已接受、ASCII 小写子串查询、
  保留提供顺序；上限（maxTags>0）阻止新增但仍可删除，超上限提示配置冲突不截断
  原值；配置错误（maxTags<0、tags 空/重复）经 `error(str)` 并阻止修改。
- **Notes 迁移**：015-notes 编辑区改用 TagInput；`contextId` 由 store 在
  LoadDraft 时按真实 noteId 构造（`tag_context`），store 的 TagRequested 核对
  当前活动笔记与修订后单次 `update_tags` 落盘并回 accepted；迟到请求（已切笔记）
  被拒。正文编辑留在 Editor，正文摘要不变。

## 已运行的探针与门

原始产物在 `D:/autostack/plan-artifacts/plan-097/`（探针/门独立目录）。产品验收按
Plan §6 记录。

| 入口 | 观察 |
| --- | --- |
| `verify_tag_input.py --backend native --case demo`（3 连跑） | 全 pass；新增/去重/删除/上限/业务拒绝/候选-Escape-重开-Enter/配置错误逐 case pass；退出后进程与端口回收 |
| `verify_tag_input.py --backend vue --case demo`（3 连跑） | 全 pass；真实浏览器（Playwright MCP 0.0.86 + Chrome）+ vite preview 同矩阵 |
| `verify_tag_input.py --backend native --case notes` | pass；015-notes 添加标签→chip→切换笔记读回→正文不变 |
| `tag_input_preview.py` | Vue 开发构建（vite build --mode development --minify false）与 `tag-input-preview.json` 收据 |
| `cargo test -p auto-ui-tools --test tag_input_component` | 6/6 pass；模型反例（身份/去重/上限/迟到反馈/配置错误一次/双实例隔离）+ 移根生成 |
| 平台 | Windows native + Windows Vue 已执行；Linux 两轨按 084 归档 §10 用户决定走 VMware 路由，当前无 runner 记 `not_run`，对应 AC 保持未关闭 |

## 待办/边界

- 空草稿 Backspace 删末位标签：组件已实现（受请求锁保护）；产品门当前未覆盖
  （键路由在运行时经 keydown.backspace 派发，行为与 Enter 同源，后续补 case）。
- Vue notes 轨（015-notes 真实后端 + 浏览器）：构建链路已通（vue-tsc/vite），
  完整门随最终交付回归补齐；notes native 已过。
