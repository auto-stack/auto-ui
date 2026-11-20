# PLAN-094 实施决策与探针

当前合同为 r3，交付及验收范围经用户确认只包含 Windows Native VM 和 Vue。实施工作树为 `D:/autostack/.wt/ui-094/auto-ui`，分支 `codex/ui-094-prompt-dialog`，基线 `9bdbbcf859e5185335014425af5e74f5512e8c29`。

## 依赖与基线

| 来源 | 核对值 |
| --- | --- |
| auto-lang | `68fd4c59222c77a5de0963ff5827f1353abd7b4f` |
| auto-web | `26d085f7e54ea20c6997edf2dc23eef0cb57faf6` |
| auto-down | `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5` |
| auto-man | `4a30483a590d81d5724cb453a0fd1fbc98d9aaf2`，组内只读依赖检查 |
| 编译与浏览器工具 | rustc 1.98.0，Node 25.2.1，Python 3.14.2，Playwright MCP 0.0.86，Chrome |
| 027 `src/front/app.at` | 工作树基线 SHA256 `8EAD8E0FD3610F461A830F9E1B755950F4E3D1467A4AAFA206461B204952E775` |
| `tags_panels.rs` | 基线 SHA256 `20DD72D5C89C13DFB19042E8DA87B1271B436BA3F513CF4543FC4CFD881850E0` |
| `file_dialog.at` | 基线 SHA256 `F6C027CD4EA27515CA62A7D87609AEF24361ECFA08A12B58979FA77E4A5A0E00` |
| `file_dialog_component.rs` | 基线 SHA256 `8027BF8C34D90CE183D88EA92CF988E67FBCE268B334240AC7A81EED500F1318` |
| 参考图 | SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13` |

主检出的其他计划和设计文件已有改动。组件和测试在本计划工作树中执行，Plan 进展留在主检出。三个编译依赖来自组内固定的分离工作树。早期诊断使用了既有 CLI 和旧工具，后续探针已改用计划工作树构建的 `auto-ui-tool.exe`。

## 机制

- 正式包采用 `pac.at` 相对路径和 widget 内的包导入。Native VM 和 Vue 均执行正常控件的文本输入、请求、拒绝显示、应用成功写回及关闭重开。
- 真实挂载实例在第一次 Tick 生成并保存 UUID。Native 使用现有 `str.uuid()`，Vue 编译器将同一入口降低为浏览器 UUID。普通重绘和关闭重开保留挂载身份，挂载重建得到新身份。每个编辑基线另分配控件绑定值，作为普通键控视图的 key 与事件参数。输入、提交及取消消费时核对当前绑定和使用条件。
- 初次配置错误从正常 Tick 派发，避开 native 子组件 Init 的 emit 限制。相同确认、取消标签的错误只派发一次。有效配置恢复操作。
- Native 的按键走现有模态与焦点路径。Vue 复用 Reka Dialog 的 Portal、焦点、原生按钮和输入。显式 onclose 消费 Reka 关闭请求，父层控制 open。禁用背景关闭时阻止 Reka 的 outside dismiss。输入适配继续保留 composition Enter 门控。
- 027 使用 `Env.track()` 判定实际能力。Windows VM 保留现有 `fs.rename`、`file.create_dir` 和 `file.write_text` 入口。调用方提交前核对目录、目标和修订。Windows Vue 仍走原有只读 API，不启用写操作。

## 已运行的探针

原始产物位于 `D:/autostack/plan-artifacts/plan-094/probe/`。以下为机制探针，产品验收按 Plan §6 单独记录。

| 入口 | 观察 |
| --- | --- |
| `python probe.py native --tool-bin <实施树工具> --output keyed-native-2` | 退出 0，9.334 秒。有本轮所属真实窗口，Enter 提交、重名保留、中文写回、配置错误一次、关闭重开均有记录。旧确认地址被当前模态拒绝，当前确认可用，退出后原 MCP 方法收到 Windows 10061。 |
| `auto gen -p`、`pnpm exec vue-tsc --noEmit`、`python probe.py vue --app app-revised --output keyed-vue-1` | 类型检查退出 0，浏览器探针 6.818 秒。读取的初值 `Inbox` 全选，背景点击保留窗口，Escape 关闭，应用读回正确，配置错误一次。 |
| Vue 编译器目标 | 受控关闭新增用例 0.16 秒，旧双向弹窗 0.14 秒，解析器通知路径 0.17 秒，以及有载荷 Enter 回归 0.16 秒，均通过。 |

旧原生工具的退出调用可能在响应刷新前关闭连接。通过独立的 CLI 退出码和同一 MCP 端口的拒绝连接验证生命周期。最初的 Vue 诊断二进制曾漏出 `@vueuse/core`，当前树重新生成项目已包含依赖并通过类型检查。
