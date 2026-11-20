# PromptDialog 文本输入对话框

来源为 PLAN-094 r3 的 SD-01 和 SD-02。实现见[正式包](../../../packages/prompt-dialog/README.md)和[名称管理示例](../../../examples/prompt-dialog/README.md)。

## 包及应用责任

正式包 `packages/prompt-dialog` 提供 `PromptDialog`。它使用标准 Dialog、Input 和 Button。调用方通过应用清单声明依赖，组件通过相对路径导入。组件保留本地草稿、校验、焦点和提交锁。调用方拥有业务规则、权限、状态发布和 I/O。

`open` 由调用方控制。每次打开产生新会话。打开态下 `contextId`、`revision` 或 `initialValue` 改变时重建编辑基线。有效关闭和重新挂载使旧反馈失效。实际挂载身份跨普通重绘保留，动作同时绑定当前编辑基线。

## 文本与请求

默认必填且保留文本。必填检测忽略首尾空格、Tab、CR 和 LF。启用 `trimInput` 时提交裁剪结果。组件保留其他字符和原有 Unicode 表示。调用方决定重名、文件名及其他业务条件。

确认按钮和输入的 Enter 都发出 `submitRequested(str)`。提交前组件先锁定，载荷为 JSON 字符串 `{instanceToken, session, seq, contextId, baseRevision, value}`。调用方核对当前目标和修订，再执行变更。反馈的 `instanceToken`、`session`、`seq` 和 `contextId` 必须对应当前请求。`accepted` 保持锁定，由调用方关闭。`rejected` 保留草稿并显示 `message`，`retryable` 决定是否允许原值重试。外部 `pending` 可以增加动作限制。外部 `errorMessage` 只显示当前目标的一般错误。

取消按钮和 Escape 发出 `cancelled(str)`，`reason` 分别为 `cancel` 和 `escape`。调用方关闭前，后续取消和确认均被锁定。外部关闭没有附加取消事件。配置错误使用 `error(str)`，提供 `code` 和 `message`，相同配置在普通重绘时保持一次通知。

## 窗口、输入与验证

打开时焦点进入普通输入控件并选中初值。Tab 限定在当前模态。关闭时恢复仍可用的触发控件焦点或应用安全焦点。背景点击不关闭。输入法组合中的 Enter 保留给输入法。提交中的输入、确认、取消及 Escape 均受当前会话的锁约束。PromptDialog 使用显示的字段标签作为输入的可访问名称，当前校验错误同时用作可访问说明。标准原生输入未提供显式标签时继续使用占位文案作为可访问名称。

名称管理示例通过普通界面读回应用状态。027 File Manager 在 Windows Native VM 中使用原有文件系统入口，Vue 维持只读能力。文件或目录创建失败时应用反馈拒绝，组件保留草稿供修正或重试。Windows Native VM 与 Windows Vue 的验收包含真实窗口或页面、普通控件、当前 MCP 节点、应用读回和失败清理。生命周期用例还会退出整个应用，确认原有进程和端口已释放，再启动独立会话并操作当前控件。Windows Native VM 另有独立的系统可访问性与输入桥验证。
