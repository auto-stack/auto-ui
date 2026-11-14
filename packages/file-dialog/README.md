# packages/file-dialog — FileDialog（PLAN-084）

官方应用内文件/目录选择组件：Windows 与 Linux 原生 VM 上，经 App 持有的
异步目录服务完成**打开文件（单/多选）、选择目录、保存位置选择（含覆盖
确认）**。它不是系统文件对话框，不替代 `dialog_open_file`/
`dialog_open_files` 及其既有 provider（旧 `dialog_*` 系统 API 保持原入口）。

视觉参考：[文件导航与轻量交互组件视觉参考 §2.1](../../docs/design/autoui/file-navigation-and-interaction-visual-guide.md)
（目录选择 / 保存 / 覆盖确认三区，浅深两态与 420×560 窄窗；图不是运行截图）；
既有打开模式的改进记录：[file-dialog-ui-polish](../../docs/design/autoui/file-dialog-ui-polish.md)；
合同与执行记录：[PLAN-084](../../docs/plans/084-file-dialog-folder-and-save-modes.md)。

## 使用

```auto
use { package: file_dialog from "path/to/file-dialog" }
```

包内 `file_dialog.at` 以 widget 级 `use { package: pagination from "../pagination" }`
装载官方 PaginationControl（PLAN-086）——**依赖闭包必须整组交付**：消费方
把 `packages/file-dialog` 与 `packages/pagination` 两目录一同 vendor（字节
一致），缺依赖是具名构建失败（vue 轨）/静默缺席（native 轨由 build 轨负例
门兜底）。

## 四种 selectionMode（会话冻结）

| 值 | 行为 | accepted 返回 |
|---|---|---|
| `singleFile`（默认） | 单文件选择 | `{"files":[{name,path}]}` 恰一条 |
| `multipleFiles` | checkbox 多选 | 按列表顺序的多条 |
| `singleDirectory` | 仅选目录：行选择与进入独立 + “选择当前目录”（根可选）；文件行只作上下文不可选 | 一个经重验的真实目录 |
| `saveFile` | 文件名输入 + 现有过滤组；覆盖确认嵌套 modal（Esc=返回修改，外点吞掉）；**选择器零写入**——目标存在与否、是否创建，全部由消费应用持有 | 确切目标路径（可能尚不存在） |

新增 props：`defaultFileName`（显式默认名，优先于 initialDirectory 文件路径
的 basename 预填）、`defaultExtension`（仅当输入名缺扩展名时在**结果**中补全，
不改写输入；与活动组不兼容是不可恢复配置错误）。未知 `selectionMode` 是具名
错误；模式在会话 open 时冻结，open 期间父组件改 prop 不切换当前会话。

覆盖确认绑定目标身份（长度+mtime 指纹）：目标被替换/改名后旧确认失效并
重新出层，确认过的新身份才接受；输入/目录变化使旧确认失效。接受只是选择
与意图——TOCTOU 的最终冲突处理归消费应用。

## 验证

- Runtime `file_dialog` / `file_dialog_model` 集成目标：四模式、保存命名/
  扩展名/只读/符号链接反例、覆盖身份竞态（受控 FS + 真实拒绝）、生命周期。
- Tools `file_dialog_component`：双生成器 + 依赖闭包。
- `tests/product/verify_file_dialog.py`：真实 native VM/MCP 全门（Win/Linux
  分别跑；缺 runner 记 not_run 不记 pass）；420×560 窄窗与浅/深截图走
  `--visual-only`。
- `tests/product/verify_file_dialog_modes_consumer.py`：041 auto-edit 真实
  消费门（Ctrl+S / 另存为 / 取消 / 覆盖 Back+Replace / 只读失败读回）。
