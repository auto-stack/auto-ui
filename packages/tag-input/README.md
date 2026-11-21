# packages/tag-input — TagInput 受控标签输入（PLAN-097）

正式包 `packages/tag-input` 提供 `TagInput`。组件复用标准 Input/Button；调用方通过
应用清单声明依赖，组件经相对路径导入。组件持有局部草稿、候选、选中与防重复请求
状态；调用方持有业务数据、权限、I/O 与执行效果（持久化）。

## 组件文件

| 文件 | 内容 |
|---|---|
| `tag_input.at` | TagInput 组件（状态机、校验、消息与身份协议） |
| `tag_text.at` | 文本工具：`tag_trim` / `ascii_lower` / `tag_contains` |
| `package.at` / `pac.at` | 包清单 |

## Props（冻结契约）

| prop | 类型 / 默认 | 语义 |
|---|---|---|
| `tags` | `List = []` | 受控已接受标签（去重保证唯一，作为稳定 chip 身份） |
| `contextId` | `str = ""` | 调用方业务目标身份；变化中止旧请求与草稿 |
| `revision` | `int = 0` | 受控目标修订；accepted 确认要求 revision 已上升 |
| `suggestions` | `List = []` | 静态候选；去重、排除已接受标签、ASCII 小写子串查询 |
| `maxTags` | `int = 0` | 上限；`0` 不限，`<0` 具名配置错误 |
| `disabled` / `readOnly` | `bool` | 禁用全部修改 / 允许查看但禁止增删 |
| `label` / `placeholder` / `errorMessage` | `str` | 展示与外部错误提示 |
| `response` | `str = ""` | 应用确认回执（accepted/rejected 身份匹配后才生效） |

## 事件

- `"changed"(str)`：JSON `{"instanceToken","seq","contextId","baseRevision",
  "action":"add"|"remove","tag","tags":[完整候选数组]}`。一次合法手势最多一条；
  父层拒绝前不乐观抹掉真实 chip。
- `"error"(str)`：JSON `{"code","message"}`。具名配置错误（`invalid-max-tags`、
  `invalid-tags`）恰好一次通知，并阻止相关修改。

确认语义：`response` 必须完整身份匹配（instanceToken+seq+contextId）；`accepted`
还要求当前 props `tags` 等于本次候选数组且 `revision` 已上升（同快照发布），否则
视为 `invalid_feedback`（显示诊断、保持外部值、不二次写）。`rejected` 保留草稿与
原 tags。目标/修订/标签变化中止旧请求；候选在草稿变化时重开，Escape 只收起候选。

## 消费方式（依赖闭包三件套）

```auto
// pac.at
dep tag_input {
    path: "../../packages/tag-input"
}

// widget 内
use { package: tag_input from "../../../../packages/tag-input" }

// 实例化（应用回执同快照推进 tags/revision）
TagInput (tags: .tags, contextId: "note-12", revision: .revision,
    suggestions: ["Design", "Development", "Work"], maxTags: 5,
    response: .reply, onchanged: .OnChanged, onerror: .OnError) {}
```

移根/vendor：整目录（`package.at`/`pac.at`/`tag_input.at`/`tag_text.at`）复制到第二
绝对根并保持字节一致；缺依赖在 build 轨具名失败。

## 已知边界

- 候选查询按 ASCII 小写化子串匹配（其他 Unicode 原文匹配），不做 Unicode/case 归一化。
- 逗号与粘贴默认一个完整标签，不未经请求拆分批量提交。
- Enter 在 native MCP 键盘路由下派发到树中首个输入（运行时行为，见
  `docs/reviews/plan097/decisions.md`）；真实 OS 焦点输入不受影响。
