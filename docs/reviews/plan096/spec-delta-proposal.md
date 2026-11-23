# PLAN-096 待核验规范增量

规范落点为 `docs/specs/ui/command-palette.md`。以下内容待最终评审及合并。

| ID | 内容 | 验收 |
|---|---|---|
| SD-01 | 发布 `CommandPalette` 受控参数、命令记录、local/provided 排序、实例与会话身份、异步反馈、禁用与配置错误。组件持有 UI 状态与请求锁，应用持有权限、结果生产与业务效果。 | AC-01、AC-02、AC-03 |
| SD-02 | 发布相对路径导入、默认 slot、独立设置消费示例、028 Launcher 迁移及 Windows Native VM/Vue 的真实控件、MCP、独立业务读回与生命周期门。 | AC-04、AC-05、AC-06 |

下列正文供评审。验收后由 merge 发布至上述规范路径。

# CommandPalette 命令面板

正式包 `packages/command-palette` 提供 `CommandPalette`。应用在 `pac.at` 声明包依赖，在 widget 中通过相对路径导入。移根保留 `pac.at`、`package.at` 和 `.at` 源的完整依赖闭包。`CommandPalette` 使用标准模态、输入和按钮，已有内置 `Command` 仍按自身规则过滤和运行。

## 数据与责任

调用方拥有命令数据、过滤结果的业务来源、权限、执行、持久化和全局快捷键。组件管理查询草稿、焦点、活动命令和请求锁。应用通过 `open` 控制是否显示，通过 `query` 同步查询。`commands` 默认空列表，`filterMode` 默认 `local`，`revision` 默认 `0`，`wrapNavigation` 默认关闭。`title` 和 `placeholder` 为空时分别显示 “Command palette” 和 “Search commands”。`errorMessage` 用于应用错误提示，`response` 用于当前请求反馈。默认 slot 可在编辑阶段放置应用控件。

每条命令提供非空唯一 `id` 和可见 `label`。同名命令可分别运行。可选字段为 `groupId`、`groupLabel`、`keywords`、`shortcut`、`disabled` 和 `icon`。空 `groupId` 表示无分组。同一组的非空标题须一致。`shortcut` 是文字提示，图标缺失不影响命令名称。无效过滤模式、空或重复 ID、组标题冲突都会显示具名错误并阻止调用。`error` 每次相应配置变化发布一次 JSON `{code,message}`，相同错误在重绘时不重复发布。Provided 模式首次收到结果前即可提示现有配置错误，应用仍会收到当前查询请求。

## 搜索与选择

Local 模式裁剪查询两端的空格、Tab、CR 和 LF，在 `label`、`id`、`keywords` 做子串匹配。ASCII 字母折叠大小写，其他 Unicode 保持原值。分组按输入首次出现顺序。非空查询在组内按标签完全匹配、标签前缀、其他命中排序，各层保留输入顺序。空查询保留组内输入顺序。

Provided 模式由应用先过滤和排序。每次打开及查询变化均发 `queryChanged`，应用随 `commands` 和递增 `revision` 回传 `resultsSession`、`resultsQuerySeq`、`resultsQuery`。三项必须对应当前查询，结果顺序由应用决定，相邻同组行共享组标题。新查询的旧结果不可执行，包括查询 A 经 B 再回到 A。应用回传同一查询后保留草稿，外部改变查询会产生新查询。

打开时焦点进入搜索输入。Up 和 Down 跳过禁用结果，默认到首尾停住，可开启环绕。重排保留仍有效的活动 ID，移除后选择首个可执行结果。Enter 或点击提交当前有效命令，空结果、禁用项、旧节点和输入法组合中的 Enter 均不提交。Tab 保留在活动弹层，关闭后恢复触发焦点。背景点击不调用命令。

## 事件与反馈

公开消息参数为 JSON 字符串。挂载实例拥有稳定且不复用的 `instanceToken`，每次打开生成递增 `session`，每次查询递增 `querySeq`，每次有效调用递增 `seq`。

| 消息 | 字段 |
|---|---|
| `queryChanged` | `instanceToken`, `session`, `querySeq`, `query` |
| `invokeRequested` | `instanceToken`, `session`, `seq`, `commandId`, `baseRevision`, `querySeq`, `query` |
| `closeRequested` | `instanceToken`, `session`, `reason`（`escape` 或 `cancel`） |

应用核对当前请求、命令及权限再执行。同一修订的命令内容若已改变，旧绑定同样失效。等待反馈时，用户输入、移动、执行及关闭均受锁保护。过期 provided 结果不能解除当前请求锁，当前结果发生有效变更时仍可使旧动作失效。应用可关闭面板，重新打开或重挂载后拒绝旧反馈。`response` 须包含本次 `instanceToken`、`session`、`seq`、`commandId`、`status` 和 `message`。`accepted` 保持锁直到应用关闭，`rejected` 显示消息并允许重试。普通 Escape 或取消仅发一次关闭请求。

## 消费与交付范围

设置示例在 Windows Native VM 和 Windows Vue 通过普通控件更改设置模型并读回。028 Launcher 使用 provided 模式并保留既有排序、分类、最近使用、环绕及 Grid view。Windows AutoOS 桌面仍由应用的 `Launch` 链发起宿主动作，未连接宿主的独立入口显示不可用反馈。

Windows Native VM 和 Windows Vue 验收使用真实窗口或页面、当前 MCP 元素、独立业务读回及应用生命周期检查。Windows Native VM 另验系统可访问性和输入桥，Vue 使用已存在的真实浏览器 MCP 提供方。窄窗、浅深色、长中文、错误状态和处理中状态检查真实渲染。原生与 Vue 均要求缺能力明确失败、进程和端口回收、再次启动可用。当前交付范围为 Windows Native VM 与 Windows Vue。
