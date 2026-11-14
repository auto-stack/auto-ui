# PLAN-084 Spec 增量提案（T-06；canonical 合入在 T-08 merge 执行）

目标文件：`docs/specs/ui/file-dialog.md`。以下三处增量按 SD-01..03 组织；
合入时同批更新本提案为「已合入」并记录交付提交。本轮未修改 canonical。

## SD-01（modify · Public interface + modes）

`selectionMode` 扩为四值并新增 props（Public interface 表替换/追加）：

| Property | Default | Meaning |
|---|---|---|
| `selectionMode` | `singleFile` | `singleFile`、`multipleFiles`、`singleDirectory` 或 `saveFile` |
| `defaultFileName` | empty | 保存模式初始文件名草稿；显式非空值优先于 initialDirectory 文件路径的 basename 预填 |
| `defaultExtension` | empty | 不含点；仅当草稿名缺扩展名时在**结果**中补全；从不改写输入框 |

- 四种模式在新会话 open 时冻结；open 期间父组件改该 prop 不切换当前会话，
  关闭重开后采用新模式。未知字符串是具名错误（`error` 事件），不回落打开。
- `accepted(str)` 容器不变：`{"files":[{"name":"…","path":"…"}]}`；两个新
  模式均恰好一个条目。目录模式返回 canonical 目录（根目录可选，name 为其
  basename）；保存模式返回确切目标路径，目标可以不存在；Windows canonical
  父目录可带 `\\?\` 前缀。
- 目录模式：行选择（`选择 ${name}`）与进入（chevron，`进入 ${name}`）独立；
  “选择当前目录”控件选中当前列出目录；文件行仅作上下文不可选中；文件类型
  组不展示也不排除目录；名称过滤不影响进入父目录/Home/路径入口。
- 保存模式：`文件名` 输入行（初始焦点；其余模式沿路径输入）；确认（含
  文件名输入内 Enter）走同名流程。

## SD-02（modify · Directory and selection rules + validation boundary）

保存目标验证（新增小节“Save-target rules”，无写入边界贯穿）：

- 名称约束：仅 basename；路径分隔符、`.`、`..`、NUL、Windows 保留设备名是
  具名可恢复 `invalid_path`；不静默 trim/改名；空名禁用确认（无错误事件）。
- 扩展名：缺扩展名且 `defaultExtension` 非空才在结果中补全；显式扩展名不
  匹配活动组 = 可恢复错误；`defaultExtension` 与活动组不兼容 = 不可恢复
  配置错误；切换过滤组不重写输入。
- 目标解析（worker）：父目录 canonicalize 且必须为目录；目标经
  symlink_metadata 判定——符号链接 = `unsupported_path`；目录 = 可恢复
  `invalid_path`；特殊文件 = `unsupported_path`；普通文件 = 进入覆盖确认；
  只读属性（Windows `FILE_ATTRIBUTE_READONLY` / Unix 写位）= 可恢复
  `permission_denied`。选择器不 create/truncate/rename 任何路径（含权限
  探测）；最终写权限与写入成功由消费应用判断。
- 覆盖确认：默认总是确认，无跳过选项。确认层是同 App 内嵌套受控 modal
  （`closeOnOutside:false`；Esc/关闭路径 = 返回修改；层出现时底层路径/列表/
  保存动作在模型层与 modal scope 双重拒绝）。显示真实目标路径。绑定目标
  身份（长度+mtime 指纹）：“确认覆盖” 重验同一会话目标——同身份接受；
  目标消失按创建接受；身份变化重新出层，旧确认永不命中新目标。输入/目录/
  过滤/列表状态变化使旧确认失效。接受只是选择与意图，TOCTOU 最终处理归
  应用。

## SD-03（modify · Modal/focus/automation + Verification）

- 焦点：打开模式聚焦路径输入；保存模式聚焦文件名输入；每会话一次，
  经既有 `ui.focus` 内建（modal-aware 解析），不新增焦点 API。
- 覆盖层的自动化：层内 `返回修改`/`确认覆盖` 是普通可驱动控件；层打开时
  背景（含外层对话框控件）动作被具名拒绝；快照不含关闭层为可操作树。
- 验证章节追加：
  - `tests/product/verify_file_dialog.py` 四模式全门（目录/保存/覆盖/
    生命周期/平台视觉/字节基线不变），`--visual-only` 覆盖视觉参考 §2.1
    三区浅/深与窄窗实窗截图。
  - `tests/product/verify_file_dialog_modes_consumer.py`：041 auto-edit
    真实消费（Ctrl+S/另存为/取消/覆盖 Back+Replace/只读失败），Env 路径
    旁路不存在（`AUTO_SAVE_PATH` 退役）。
  - 消费边界不变：内容写入、发布格式、限制与业务记录归消费应用 spec。
