# PromptDialog

PromptDialog 是 AutoUI 的受控文本输入对话框。Windows Native VM 和 Vue 使用同一份 Auto 源码。重命名、名称唯一性和持久化由调用方处理。

在应用的 `pac.at` 中声明 `prompt_dialog`，路径指向本包。在调用方 widget 的 `use` 块中按源文件相对路径导入同一目录。移植时复制 `package.at`、`pac.at`、`prompt_dialog.at` 和 `prompt_text.at`。名称管理示例同时展示了完整导入和应用回调。 [示例应用](../../examples/prompt-dialog/src/front/app.at)

| 参数 | 默认值与用途 |
| --- | --- |
| `open`, `contextId`, `revision`, `initialValue` | 默认 `false`、空字符串、`0`、空字符串。应用控制可见性和编辑基线。基线变更时重新读取初值。 |
| `title`, `description`, `label`, `placeholder` | 展示文本，可省略。`label` 同时作为输入框的可访问名称。 |
| `acceptLabel`, `cancelLabel` | 默认“确定”“取消”。提供可读的操作名称。 |
| `required`, `trimInput` | 默认必填并保留输入原文。必填检测忽略首尾空格、Tab、CR 和 LF。启用 `trimInput` 后提交裁剪结果。 |
| `pending`, `errorMessage`, `response` | 默认 `false`、空字符串、空字符串。分别表示调用方提交中、当前目标的外部错误和有请求身份的反馈。 |

`submitRequested(str)` 提供 JSON 字符串，包含 `instanceToken`、`session`、`seq`、`contextId`、`baseRevision` 和 `value`。调用方先核对上下文和修订号，再尝试业务操作。反馈原样回传 `instanceToken`、`session`、`seq`、`contextId`，并附上 `status`、`message` 和 `retryable`。成功使用 `accepted` 并由应用关闭对话框。拒绝使用 `rejected`，保留草稿。若 `retryable` 为 `false`，需要编辑草稿后才能再次提交。

`cancelled(str)` 携带当前身份及 `reason`，取值为 `cancel` 或 `escape`。应用收到取消后将 `open` 设为 `false`。`error(str)` 报告配置错误，包含 `code` 和 `message`。提交中与等待关闭时，组件会阻止重复动作。

当前错误显示在输入框下方，辅助技术可随输入框读取错误说明。打开时选中初值，Tab 在弹窗内移动，关闭时恢复可用的焦点。
