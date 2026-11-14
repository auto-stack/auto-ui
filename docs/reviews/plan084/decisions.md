# PLAN-084 决策记录（T-01）

2026-11-14，执行树 `D:/autostack/.wt/ui-084/auto-ui`，分支
`codex/ui-084-file-dialog-modes`，基线 `a6f776c2`（master）。本文冻结实施
语义；所有决策保持 Plan §5/§7 不变。探针均在本地 scratch
`D:/autostack/plan-artifacts/plan-084/` 执行，命令与结果摘录于 §9。

## 1. 来源复核（§4 逐项，实际 HEAD）

| 来源 | 当前事实（本树复核） |
|---|---|
| `docs/specs/ui/file-dialog.md` | 已交两打开模式；accepted 容器、异步 owner、MCP/分页/PaginationControl（PLAN-086 已迁移）条款均在位；SD-01..03 增量目标正确 |
| `packages/file-dialog/file_dialog.at` r237 行 | props 与 spec 一致；视图经 `fd_view` 投影；分页已接 `PaginationControl`（兄弟包导入） |
| `crates/auto-ui-runtime/src/ui/file_dialog/{model.rs,mod.rs,component_io.rs}` | `SelectionMode` 仅 SingleFile/MultipleFiles；`FileSystem` trait 只有 navigate/validate；`validate` 只验普通可读文件；worker/generation/cancel 机制完备可复用 |
| `crates/auto-ui-runtime/src/ui/modal.rs` | `active_content_path` 明确支持嵌套 modal 收窄（"Nested modals further narrow it"）；Esc 经 `escape_message` 路由到最内 modal 的 on_dismiss；`allows_path` 拒绝活动 scope 之外的动作 |
| `crates/auto-ui-runtime/src/ui/aura_view_builder/tags_panels.rs` | `dialog` + 显式 `onclose` + `closeOnOutside: false` → `PopoverPlacement::ModalExplicit`（外点吞掉不 dismiss）；Esc/外点折算 onclose 消息 |
| `crates/auto-ui-runtime/src/ui/iced/renderer/host.rs` | `ui.focus(key)` 内建消费槽（PLAN-095 T-04）→ `collect_focusable_inputs`（modal-aware）解析 → iced focus 任务；结果落 `__focus_result`；5 轮挂载重试 |
| `auto-lang native_catalog.rs` | `ui.focus` = native 9920 `auto.ui.focus`，普通 VM handler 可调用（本仓无内建注册表，语言仓只读复核） |
| `examples/ui/041-auto-edit` | `ActSave` 无路径时走 `dialog_save` + `AUTO_SAVE_PATH` 旁路；`file.write_text` 返回 Void（语言仓 native_catalog 实证）——应用侧无法从返回值感知写失败 |
| `examples/file-dialog` 夹具 | accepted 回调写 `FILE_DIALOG_RESULT_PATH`（内容 = 回调 JSON 原文），与选择模式无关，可直接复用 |
| `tests/product/verify_file_dialog.py` | 真实 CLI+MCP+窗口归属+退出回收设施完备；分页/焦点/主题/取消重开/1003 项门在位；`--visual-only` 窄窗档在位 |
| PLAN-086 交付 | master `5e3563db` 归档、交付 `765251b9`；`packages/pagination`（1-based `changed` 契约）与 `docs/specs/ui/pagination.md` 在位；086 的 Linux leg + 统一回归为显式 pending closeout 门（非本计划义务） |

## 2. 模式与 props（冻结）

- `SelectionMode` 追加 `SingleDirectory`、`SaveFile`；serde camelCase
  `singleDirectory`/`saveFile`。未知字符串 = serde 具名错误（`error` 事件，
  code `invalid_path`），不回落打开模式。
- `DialogConfig` 追加 `default_file_name: String`、`default_extension: String`
  （camelCase `defaultFileName`/`defaultExtension`，默认空）。`.at` props 同名
  追加；`open` 命令 config 载荷携带两新 props。
- 模式在会话 open 时冻结进 model config；父组件 open 期间改 prop 不切换当前
  会话（既有 Tick 仅在 `!seen_open` 时发 open）。投影新增 `mode` 字符串，
  视图按投影 mode 分支（不直接读 prop，保证与会话一致）。
- `initialDirectory` 语义不变：空 → home；完整文件路径在打开模式预选该文件
  （现行为）；在保存模式 → 目录取其父目录、文件名预填 basename（§4）。
- 其余 props（filters/activeFilterId/showHidden/acceptLabel/title）语义不变；
  `acceptLabel` 默认仍 `打开`，保存消费者显式传 `保存`。

## 3. 保存文件名优先级与扩展名（冻结）

1. 显式 `defaultFileName` 非空 → 会话初始 `saveName = defaultFileName`。
2. 否则 `initialDirectory` 指向已存在的普通文件 → `saveName = 其 basename`
   （目录取父目录）。
3. 否则 `saveName = ""`。空名禁用确认（不产生错误事件）。
4. 扩展名补全发生在确认验证时刻（不重写输入框内容）：名称无扩展名（含以
   `.` 结尾视为空扩展名）且 `defaultExtension` 非空 → 结果名 = 名称 +
   `.` + defaultExtension。切换过滤组不改写已输入名称。
5. 名称带显式扩展名且活动组（filters 非空时）不含它 → 可恢复
   `invalid_path` 错误（保留输入）；不偷偷改扩展名。
6. `defaultExtension` 非空且与当时活动组不兼容 → 不可恢复配置错误
   （retryable=false，`invalid_path`，消息具名 defaultExtension 与活动组）。
7. 名称约束：仅 basename；路径分隔符（`/`、`\`）、`.`、`..`、NUL → 具名
   `invalid_path`（可恢复，保留输入）；不静默 trim/改名。OS 层拒绝（如
   Windows 保留名/尾点尾空格）按 io 错误具名呈现。

## 4. 保存目标验证与覆盖状态（冻结）

- 验证入口 `FileSystem::validate_save(directory, file_name, cancel)`：
  父目录 canonicalize + 必须存在且为目录；目标 = 父目录 join 最终名；不创建、
  不截断、不重命名任何路径（含权限探测——只用 metadata 只读属性，绝不以
  建临时文件探写权限）。返回：
  - `Create { name, path }`：目标不存在，path = canonical 父目录 + 最终名
    （目标本身不再 canonicalize，不存在者无 canonical 身份）；
  - `Overwrite { name, path, identity }`：目标为普通文件；identity = 长度+
    mtime 指纹（Windows 只读属性 / Unix 写位 → `permission_denied` 可恢复
    错误）；目标为目录/特殊文件 → `invalid_path`/`unsupported_path` 具名
    错误；目标为符号链接 → `unsupported_path`，不静默改指。
- 确认（saveFile 模式 Confirm）→ worker `validate_save`：
  - `Create` → 直接 accepted（单条目）；
  - `Overwrite` → 进入覆盖确认状态（投影 `overwrite.open/name/path`），
    无 accepted/error；
  - 错误 → 可恢复 error，输入保留。
- 覆盖层内「确认覆盖」（`confirm_overwrite`）→ 重跑同一 `validate_save`：
  `Overwrite` 且 identity 相同 → accepted；`Create`（目标已消失）→ accepted；
  identity 不同（目标被替换/改名后重出现）→ 重新进入覆盖确认（旧确认失效，
  不能命中新目标）；目录/特殊/链接 → 具名错误。无计时器，全部以重验结果
  为准（接受只是选择与意图，TOCTOU 最终处理归应用）。
- 「返回修改」（`overwrite_back`）→ 仅清除覆盖状态，输入/目录保留，会话
  继续。任何会话内输入变化（`set_save_name`、navigate/enter/up/home/root/
  refresh/filter/page/hidden）清除覆盖状态（旧确认不能确认新路径）。
- 目录模式接受（`validate_directory`）：canonicalize + is_dir + UTF-8；
  文件不能作为目录返回。accepted 仍为 `{"files":[{name,path}]}` 恰一条目：
  目录 name = canonical 路径 basename；保存 name = 最终名。
- `FileSystem` trait 追加 `validate_save`/`validate_directory` 两方法；
  `LocalFileSystem` 实装，测试替身同步实装。旧 `validate` 语义不变。

## 5. 目录选择交互（冻结）

- 目录模式复用现有导航/侧栏/名称过滤/隐藏开关/分页/错误重试；文件类型组
  按钮不展示（类型过滤不影响目录行；目录行永不过滤）。文件行仍列出（目录
  内容上下文）但不可选中、无 checkbox。
- 目录行双动作：行按钮（可访问名 `选择 ${name}`）= 选中该目录（单选，选中
  背景）；行尾 chevron 图标按钮（tooltip `进入 ${name}`）= 进入。既有打开
  模式目录行保持整行「进入」。
- 新增 `选择当前目录` 按钮（目录模式可见）：选中当前目录（根目录亦可选）。
- `toggle` 在目录模式接受目录行 id（单选）；`selected_count`/`can_accept`
  语义按模式：目录模式 = 已选目录；保存模式 = 文件名非空且导航完成。
- 打开模式两模式行为/视图/返回形状逐字节不变（V-01/07 回归门）。

## 6. 覆盖确认 UI 接法（冻结）

嵌套 `dialog`（同包 dialog-content 内）：
`dialog (open: .overwriteOpen, onclose: .OverwriteBack, closeOnOutside: false)`
→ `PopoverPlacement::ModalExplicit`：Esc → `OverwriteBack`（返回选择器）；
外点吞掉；`active_content_path` 收窄到内层（背景动作/快捷键被既有 modal
隔离拒绝）。内容 = 标题 `替换现有文件`、目标路径文本、`返回修改` /
`确认覆盖` 按钮。模型层同时守卫：覆盖状态存在时 navigate/enter/toggle/
filter/page/confirm 等会话动作全部拒绝（几何隔离之外的第二道，防 stale
MCP 目标）。以探针 P-1 实证嵌套 modal 视图/焦点/Esc 路由（§9）。
外层 `dialog onclose` 改绑 `.RequestClose` → op `request_close`：模型在覆盖
状态时按返回修改处理，否则取消（保证 Esc 语义单一路径）；footer 取消按钮
仍直发 `Cancel`。

## 7. 焦点（冻结）

- 保存模式会话初始焦点 = 文件名输入；其余模式 = 路径输入（现 spec 条款）。
- 接法：`.at` Tick 内每会话一次 `ui.focus(".SetFileName")`（保存）/
  `ui.focus(".SetPath")`（其余）；内建消费槽 + modal-aware 解析为既有
  runtime 机制，不新增焦点 API。覆盖层打开时不抢焦点（层内按钮沿焦点链）。
- 以探针 P-2 实证 `ui.focus` 在包代码可编译/派发（§9）；若运行期无效则
  保留键盘 Tab 语义并如实记录（不删 AC-04 焦点断言）。

## 8. 权限探测边界与竞争（冻结）

- 选择器零写入：不 create/truncate/rename，不用临时文件探写权限；只读目标
  以 metadata 属性判（Windows `FILE_ATTRIBUTE_READONLY`；Unix mode 写位），
  给可恢复 `permission_denied`。最终写权限/写入成功由消费应用判断。
- 无权限反例（V-02）必须证明测试进程确实被 OS 拒绝（预断言 open/写失败）；
  Linux root 不算拒绝控制。采用受控 FS 确定性反例 + 真实 OS 拒绝双证。
- 网络/UNC 沿既有非即时取消边界；不承诺任意时延。

## 9. 探针清单（scratch `D:/autostack/plan-artifacts/plan-084/`）

| 探针 | 问题 | 结果 |
|---|---|---|
| P-1 嵌套 modal | dialog 内嵌 dialog（ModalExplicit）能否成树、`active_content_path` 收窄、Esc 路由内层 | **pass**（`file_dialog.rs::nested_overwrite_dialog_narrows_modal_scope_and_routes_escape_inward`：内层成树、scope 指向内层 content、`escape_message`→OverwriteBack、外层路径被 allows_path 拒绝；基线 8+10 门同批绿） |
| P-2 ui.focus | 包 `.at` handler 内 `ui.focus` 可编译且派发 | **pass**（`file_dialog.rs::package_handler_can_request_input_focus_by_event_key`：build_dynamic_component 编译成功，FocusNow 派发后 `focus_requested:1` 可见；`__focus_result` 落点在 native host 路径，T-05 实窗核） |
| P-3 041 根视图包组件 | FileDialog 在 041 App 根视图（render: vm）下视图成树、保存事件可达 | 待运行（T-04 runtime 集成测落永久断言；真实窗口在 T-05 consumer gate） |
| P-4 Linux runner | WSL Ubuntu-22.04 工具链/GUI 可用性 | 待运行（T-05；不可用则 Linux 门记 not_run，不记 pass） |

## 10. Q-01（受影响视觉 AC）

- Reader009 原始证据树 `D:/autostack/.wt/reader-009/auto-reader` 已不存在
  （.wt 现存清单复核）；strategy 规定不能依赖该工作树永久存在。本计划在
  本仓 FileDialog 夹具上以真实窗口截图复现检查（T-05/T-06，浅/深、420×560、
  长中文文件名），逐张核对标签/文件名文字完整性。
- 若本仓路径复现缺失 → 按策略定位归因；公共渲染器级且超范围 → 具名修复
  依赖，受影响视觉 AC 保持未关闭。不复现 → 以截图对照记录闭环证据，
  不以 MCP 文本完整代替视觉通过。

## 11. Runner 与平台门

- Windows：本机（win32）为主 native runner；`cargo test` 两目标 +
  `verify_file_dialog.py` + 新 consumer gate，全部实跑。
- Linux：WSL Ubuntu-22.04 在位（PLAN-086 T-01 结论）；工具链/GUI 由 P-4
  落实；不可用 → Linux 门如实记 not_run（Q-04：不隐藏缺失门）。
- 统一回归：work 冻结后由 final review / merge 按仓档位执行，不在本任务
  范围。

## 12. 041 接入决议（AC-05）

- App 根视图持 FileDialog（保存模式专用一个实例）：`render: "vm"` 下包
  组件 + 回调 props 在夹具已实证可用（examples/file-dialog 同形态）；
  不进局部 `use` 组件（vm 组件边界：快照不可见）。视图落点 = App 根视图
  （与既有 alert-dialog 同层），探针 P-3 复核。
- `editor_store`：`ActSave` 无路径 → 打开保存对话框（`save_open=true`，
  `defaultFileName=""`）；新增 `ActSaveAs`（menubar `另存为…`）→ 无论有无
  路径打开对话框，`defaultFileName` 预填当前 basename。accepted → 取
  `files[0].path` 写入（复用现有写路径），随后 `File.read_text` 读回比对
  （`file.write_text` 返回 Void，读回比对是应用侧可观测的失败反馈）：
  一致 → `dirty=false`；不一致/不存在 → console 具名错误、保留脏状态。
  cancelled → console 记录、不改脏状态；error → console 具名错误、对话框
  保留（可恢复）或按 retryable 关闭（取消语义不动数据）。
- `AUTO_SAVE_PATH` 旁路自 ActSave 退役（新验收无 Env 路径旁路）；
  `AUTO_OPEN_PATH`/`dialog_open` 打开路径非本计划范围，原样保留。
- 依赖声明：041 `pac.at` 增 `dep file_dialog`/`dep pagination` 不需要
  （render vm 非 vue 轨）；App widget 内
  `use { package: file_dialog from "../../../..../file-dialog" }` 相对路径
  直引官方包（与夹具同型），分页闭包由包内 `use` 自动装载。

## 13. 夹具与门扩展决议（AC-06）

- `examples/file-dialog/selector.at` 追加「选择目录」「保存文件」入口
  （含覆盖夹具文件），accepted/cancelled/error 计数与结果文件复用；
  数据文件内容不变。
- `verify_file_dialog.py` 扩展：目录选择正/反例、保存正/反例、覆盖
  Back/Replace、新 subtitle/焦点、420×560 保存/覆盖窄窗截图、浅深主题、
  新模式 MCP 全走 snapshot/action/type/keyboard，无注入 API。
- 新 consumer gate `tests/product/verify_file_dialog_modes_consumer.py`
  （三必需参数同现有门）：041 真实窗口 Save/另存为/取消/覆盖/写失败读回。
- tools 门 `file_dialog_component.rs` 追加新 props 生成断言（Vue 轨不改
  交互承诺：无 native provider 不伪造保存成功）。
- 原 1003 项/2s/1s、10 次取消重开、退出回收门原样保留。

## 14. Spec 增量提案（T-06 回填，本轮不写 canonical）

SD-01..03 按 Plan §5 规范增量表落 `docs/specs/ui/file-dialog.md` 提案稿
（`docs/reviews/plan084/` 内维护 delta 文本，merge 阶段合入 canonical）。
