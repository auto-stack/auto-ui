---
plan_id: PLAN-084
status: archived
feature_name: FileDialog 目录选择与保存模式
author: [Codex]
created_at: 2026-11-13T00:00:00Z
updated_at: 2026-11-14
plan_revision: 2
current_step: 8
total_steps: 8
supersedes_spec_components:
  - docs/specs/ui/file-dialog.md
new_spec_components: []
touched_goals: []
---

# PLAN-084 FileDialog 目录选择与保存模式

## 0. 变更摘要

在现有 FileDialog 上增加 `singleDirectory`、`saveFile` 两个 `selectionMode`，保留已交付的 `singleFile` / `multipleFiles` 和 accepted/cancelled/error 入口。复用现有路径、列表、过滤、异步服务、modal 生命周期、主题和 MCP 能力，不新增平行 FolderDialog/SaveDialog 实现。

所属路线：[组件库扩展 strategy / CL-01](../design/autoui/component-library-extension-strategy.md)。本轮仅建计划；未修改组件、启动实施或声明功能通过。

## 1. 目标

- Windows/Linux 原生 VM 中，通过普通控件完成目录选择、选择保存位置及覆盖确认，跨平台交互一致。
- 组件只返回选择结果；保存文件的实际写入及失败反馈由应用持有。取消和错误不产生选择结果，也不创建文件。
- 复用 PLAN-086 的 PaginationControl；保留此前文件窗的图标导航、独立 checkbox 列、同排过滤和状态栏、单页隐藏、固定操作栏。
- 为 041 Editor 的无路径保存/另存为提供真实接入，并为目录选择提供可无人值守的本仓实例。Reader 原有导入/封面选择继续兼容。

非目标：多目录选择、递归扫描、网络发现、文件管理（新建/删除/重命名目录）、组件代写文件、业务导入事务、全局替换 `dialog_*` 系统 API、浏览器任意磁盘 IO、macOS 支持验收。跨仓消费更新另由应用计划持有。

## 2. 架构方案

输入 props → `.at` fd_command 邮箱 → App-owned `DialogModel` / worker → FileSystem 验证 → Projection → 普通控件 → 组件回调。维持单一选择算法 owner，`.at` 不复制路径/可保存性验证。

目录模式复用导航，明确区分“进入目录”和“选择此目录”；保存模式在同一 Dialog 增加文件名输入。覆盖确认作为相同 App 内的 AutoUI 确认状态，沿用 modal 输入隔离，不能调用 rfd 或生成不可驱动的系统弹窗。

分页由 086 负责先迁移当前打开模式；本计划消费其已交付提交和公共包。T-01 可先研究，T-03 及以后必须核对 086 pin，避免两计划同时改同一文件。

## 3. 技术栈

沿用 `.at` 官方包、Rust runtime、现有异步 worker/generation、VM/Iced、主题、MCP；不引入新 UI 框架。Vue 仅保持源码生成与具名能力边界，不交付浏览器本地目录服务。测试复用 Rust 集成目标和 Python native/MCP 产品门。

## 4. 需求分析与背景调查

### 授权与来源

用户原意：“先为第一批的 FileDialog 目录选择/保存模式、TreeView 和 Pagination 分别建立实施计划，复用现有实现，并保证跨平台一致、MCP 可操作和无人值守测试。”本轮授权 auto-ui 计划/设计文档编写；没有实施、发布、修改 Reader/语言仓、启动其他会话的授权；未指定模型、预算或自动续跑次数。

观察基线 auto-ui `4dd64b5fc5fc0919bfe686115c75a5d8c7ac06b1`；本仓还有无关文档 WIP。开工须重新记录实际 HEAD/工作目录摘要，并区分本计划输入与他人变更。

| 已读取来源 | 当前事实 / 约束 |
|---|---|
| `docs/specs/ui/file-dialog.md`，SHA256 `29E268496503DD5348B3E1C813D3891C20AD826E18B5ABA8F91497A4AF836E02` | 已交 Win/Linux native VM 打开模式；回调 JSON、异步 owner、路径/焦点/MCP/1003 项门不得退化 |
| `packages/file-dialog/{package.at,pac.at,file_dialog.at}` | 正式包现为 0.1.0；props 已有 selectionMode、initialDirectory、filters；不需要另一选择器框架 |
| `crates/auto-ui-runtime/src/ui/file_dialog/{mod.rs,model.rs}` | SelectionMode 只有 SingleFile/MultipleFiles；FileSystem::validate 只验证已存在普通可读文件；保存不能误用打开验证 |
| `examples/ui/041-auto-edit/src/front/editor_store.at` | ActOpen/ActSave 使用 dialog_open/dialog_save，且有 AUTO_OPEN_PATH/AUTO_SAVE_PATH 测试捷径；新验收必须走可见 FileDialog，写入仍归 store |
| `crates/auto-ui-runtime/src/ui/iced/renderer/shell_state.rs::execute_wallpaper_browse_dir` | 壁纸目录仍调用 rfd::pick_folder；本批先交通用模式和夹具，不顺带改整个 Shell 路由 |
| `crates/auto-ui-runtime/src/host_adapters/dialog.rs` | 旧系统保存 provider 有独立兼容责任，保持原入口 |
| `tests/product/verify_file_dialog.py`、`examples/file-dialog/README.md` | 已有真实 native/MCP、隔离数据、退出重开和版本指纹设施，优先扩展 |
| strategy Q-01 / Reader009 实测报告 | 部分可见文字缺失未定位；MCP 文本完整不能代替视觉通过 |

只读参考 Reader009：`D:/autostack/.wt/reader-009/auto-reader` 的 009 用例及 `docs/reviews/file-dialog-ui-sync-2026-11-13/README.md`；不能假定 Reader 主检出已有 FileDialog，也不在本计划写其 vendor 或业务文件。

## 5. 详细设计

### 模式和数据合同

| 项 | 拟议首版规则 |
|---|---|
| 模式 | `selectionMode` 新增 `singleDirectory`、`saveFile`；未知值具名错误，不回落为打开。四种模式在新会话初始化时冻结；打开期间父组件改该prop不切换当前会话，关闭重开后才采用新模式 |
| 公共返回 | 保留 `accepted(str)` 的 `{"files":[{"name":"…","path":"…"}]}` 容器，两个新模式均恰好一个条目；调用方已知所用模式。旧两模式结果形状和含义不变 |
| 目录选择 | 仅选已存在可读目录；行的进入动作与选择动作分开，有“选择当前目录”的普通控件，根目录也可选择；确认前重验 canonical 身份/存在性/类型，文件不能作为目录返回 |
| 目录过滤 | 名称过滤只影响显示，不取消进入父目录/Home/路径入口；文件类型组在目录模式不展示也不排除目录；showHidden 按现有 OS 规则 |
| 保存输入 | 新增 `defaultFileName`（默认空）、`defaultExtension`（默认空、不含点）；目录从 initialDirectory 取得。已有完整文件路径可预填其 basename；显式 defaultFileName 优先。具体优先顺序由 T-01 固化反例，不改变打开模式 |
| 文件名约束 | 仅 basename，不接受路径分隔符、`.`/`..`、NUL；空名禁用确认，其他非法 OS 名具名错误。不静默 trim/改名；目录不存在、目标为目录/特殊文件、不可表示路径分别报错 |
| 类型/扩展名 | 缺扩展名且 defaultExtension 非空才补；切过滤组不重写用户输入。显式扩展名不匹配活动组则可恢复错误，不偷偷改扩展名。默认扩展名必须与活动组兼容，否则配置错误 |
| 保存目标 | 已存在的可写父目录 + 合法文件名；父目录先canonicalize，再拼接basename。目标可不存在；存在普通文件必须进入覆盖确认；目录/特殊文件拒绝。首版保存目标若为符号链接则具名unsupported_path，不静默改为另一目标；目录导航沿既有显式解析规则。不可写/只读目标给出可恢复错误；最终写权限及写入成功由消费应用判断 |
| 覆盖确认 | 默认总是确认；没有默认跳过覆盖选项。显示真实目标路径，选择“返回修改”保留目录/文件名；“确认覆盖”重验同一会话目标。输入/目录/目标身份变化使旧确认失效，不能用旧按钮确认新路径 |
| 竞争语义 | 验证期间新出现目标要先确认；确认后目标变化重验/重新确认；接受只是选择与意图，不能保证后续写入原子性或消除 TOCTOU。应用持有最终冲突处理 |
| 无写入边界 | 选择器不 create/truncate/rename 文件，包括权限探测；结果不能声称已保存。网络操作沿已有非即时取消边界，不承诺无限资源或任意网络时延 |

### 生命周期、视觉及交付

复用 session/generation 和 worker 协作取消。关闭、重开其他模式、新实例、旧覆盖确认、延迟 IO 返回均需重验 owner；关闭只产生一次 cancelled，已 accepted 不追加 cancelled/error。可恢复错误保留可编辑内容。两个并存实例和普通/tracked builder 路径都需覆盖。

保存模式初始焦点为文件名输入（若空则可直接输入），目录模式沿路径输入；Tab/Shift+Tab 限定活动 modal，Enter 执行聚焦操作，Esc 在覆盖确认中返回选择器、在选择器中取消。关闭恢复触发器。覆盖确认不能使背景可操作。

新行保持 FileDialog 现有 800px 首选宽与视口夹取、浅/深主题和小窗可达操作栏。Q-01 由 T-01 复现定位，受影响视觉 AC 必须闭环；若超范围需具名修复计划依赖，不能自行删验收。

041 Editor 接入用真实事件接续现有写入逻辑，禁用重入、区别取消/错误、保存失败保留文档脏状态。测试前清除两项路径捷径变量；夹具目录选择和保存路径只写回调结果日志，数据文件内容不变。包发行/消费说明记录完整依赖闭包（含 086 包）及工具版本，保持从第二绝对根导入。

### 视觉参考与实现边界（r2）

2026-11-14按用户要求补充[文件导航视觉指南](../design/autoui/file-navigation-and-interaction-visual-guide.md)及[GPT参考图](../design/autoui/assets/file-navigation-084-086-reference-v1.png)，仅使用图上方两个084区域。它们分别表达目录选择、保存/覆盖确认；不是已实现界面，不要求实现图中085/086的额外内容。

复用[旧FileDialog设计与实拍](../design/autoui/file-dialog-ui-polish.md)的800px/视口夹取、图标提示、独立滚动和固定footer。目录模式不画文件多选checkbox；打开模式原独立checkbox列保留。保存格式仍用现有filter语义；覆盖层出现后底层路径/列表/Save不可操作，Back保留输入，Replace不写文件。Windows/Linux路径内容按各自OS，不把图中路径硬编码。

该引用补视觉约定，不改变§5模式/验证/返回/应用写入owner；图例文件与大小不是验收夹具，覆盖场景需准备真实存在文件。生成图不能关闭Q-01文字缺失项。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/ui/file-dialog.md | 两打开模式 → 追加目录/保存模式、props 和相同回调容器；旧模式不变 | 通用选择交互 | AC-01, AC-02, AC-03 |
| SD-02 | modify | docs/specs/ui/file-dialog.md | 文件存在/可读验证 → 按模式验证、无文件写入、覆盖意图和竞态边界 | 防止误覆盖和错误兼容推断 | AC-02, AC-03 |
| SD-03 | modify | docs/specs/ui/file-dialog.md | 既有 modal/MCP/测试 → 补新模式焦点、确认失效、消费者及 Win/Linux 证据要求 | 保持真实可驱动 | AC-04, AC-05, AC-06 |

增量仅为提案，本轮不修改 canonical Specs。拟实施树 `D:/autostack/.wt/ui-084/auto-ui`、分支 `codex/ui-084-file-dialog-modes`，未创建；实施/独立复审在同一树顺序进行，Plan 进度在主检出维护。

## 6. 测试设计

以下均为拟执行，非本轮已通过。T-05 扩展现有脚本和 Rust 目标；不新增测试专用选择 API。原模式 1003 项/11 页、10 次取消重开、2 秒目录准备/1 秒普通动作的既有本地夹具门保持，不扩大为网络承诺。

| Case ID | AC / 不变量 | 输入与真实入口 | 区分性断言 / 证据 | 边界 / work owner |
|---|---|---|---|---|
| V-01 | AC-01,02 模式隔离 | runtime 目标 + 真实夹具选当前目录/子目录/根，尝试选择文件 | 一次恰好一个目录路径；文件不能接受；原两模式 JSON 和行为不变 | Win/Linux；T-02/05 |
| V-02 | AC-03 不代写 | 夹具保存新名、带空格中文名、补扩展名、错误扩展名、非法名/不存在父目录/符号链接目标 | 正例返回确切路径；反例具名错误且保留输入，链接目标不被接受；目录项/原文件逐字节哈希和名称集不变 | 实际本地 FS + 模型；T-02/05 |
| V-03 | AC-03 覆盖意图 | 既有文件→返回修改/确认覆盖；确认期间目标替换/改名 | 首次不发 accepted；返回不关闭；失效确认不能命中新目标，原字节不变 | controlled FileSystem 确定性竞态 + native 正例；T-02/05 |
| V-04 | AC-04 生命周期 | 慢返回、关闭重开、双实例、旧节点、旧覆盖确认、失去 OS 焦点 | 单次终结事件；旧 ID/owner/generation 拒绝；背景不可操作；仍能正常新选择 | actual service/普通及 tracked 构建；T-03/05 |
| V-05 | AC-05 真实消费 | 041 新文档普通 Save→FileDialog→接受；另测取消、覆盖返回和写失败 | 通过 Editor 保存 handler 写入，读回精确内容；取消/写失败不清脏标志；无 Env 路径旁路 | 本仓 Editor；T-04/05 |
| V-06 | AC-04,06 平台/视觉 | Win/Linux native：浅/深、420×560、长中文、键盘、MCP、正常退出重开 | 标签/文件名实际可读、图标有名称和提示、焦点顺序正确、进程/端口回收；记录 Q-01 闭环 | 不以快照文本代替截图；T-05/06 |
| V-07 | AC-01,06 原能力回归 | 现有两个 runtime 目标、tools file_dialog_component、原 native gate，Vue/Rust 生成控制 | 旧模式及真实目录尾项仍可用；包依赖闭包在移根后完整；无 native provider 的 Vue 不伪造保存成功 | 原门完整保留；T-05/06 |
| V-08 | AC-04,06 r2视觉参考 | 按指南中084区域驱动目录选择/保存/覆盖Back与Replace，浅深及420×560实窗 | 图标有名、文件名/格式/状态/固定footer可达，覆盖时底层action拒绝；真路径/回调符合原语义，不以生成图当产品截图 | work，T-05/06；Win/Linux native，保留原平台门 |

无权限用例必须证明测试进程确实受拒（Linux root 不算拒绝控制），不能只依赖 chmod 后推定。采用模型确定性反例配合普通用户实际拒绝证据；必需 runner 缺失记 not_run，不记 pass。

执行入口（在拟实施树；占位路径须由执行者换成记录过摘要的实际工具路径）：

```powershell
cargo test -p auto-ui-runtime --test file_dialog --test file_dialog_model
cargo test -p auto-ui-tools --test file_dialog_component
python tests/product/verify_file_dialog.py --auto-bin <auto-cli> --ui-tool-bin <built-ui-tool> --out-dir <isolated-evidence-dir>
```

V-05 的 Editor 产品脚本由 T-05 新增 `tests/product/verify_file_dialog_modes_consumer.py`，采用同样三个必需参数；复用现有启动/所有权机制，不把真实 API 写入当输入操作。平台分开运行；工具生成检查不能代替窗口/消费读回。

实施自检由执行者负责；最终复审为后续独立上下文，无指定模型/子代理。本轮不创建会话。独立复审在同一树冻结写入后核对源码/工具/依赖指纹和原始证据。

## 7. 验收标准

| ID | owner_stage | 可观察结果 / 关闭方法 |
|---|---|---|
| AC-01 | work | 旧公开 API、两打开模式和移根导入兼容；新模式具名解析；V-01/07及原回归全通过 |
| AC-02 | work | 目录只返回一个重验后的真实目录，进入与选择独立，根/隐藏/错误可操作；V-01及相应反例通过 |
| AC-03 | work | 保存名/过滤/扩展名/覆盖意图符合§5，选择不写文件且旧确认不能误用；V-02/03 字节与事件断言通过 |
| AC-04 | work | 新模式及覆盖确认的焦点、取消、双实例、旧节点、异步回收经 V-04/06 真实 MCP 与模型反例通过 |
| AC-05 | work | 041 普通保存/另存为和取消/写失败由真实可见控件驱动并读回结果；V-05无测试路径捷径 |
| AC-06 | work | Win/Linux 两条 native 门、原回归、包闭包/生成边界、主题和小窗视觉均有版本绑定证据，Q-01 在受影响路径无未关闭缺口；V-06/07/08及视觉指南 |
| AC-07 | review | 独立上下文复核 AC-01…06/SD-01…03、消费者和范围，记录 reviewed commit、反例、各平台结果；无未决必需门 |
| AC-08 | merge | 交付版本、canonical Spec、strategy 阶段及归档互链准确，实际合并/清理收据齐全；不以 merge 补做产品正确性验收 |

## 8. 执行步骤

任务均未完成；total_steps=8。T-01 是下一个可研究任务，T-03 的 086 交付依赖目前未满足；本轮不执行。

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证入口 / 预期结果 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | [✅ 已完成] 重核§4来源、现有 FS/模态焦点能力、Q-01；形成新 `docs/reviews/plan084/decisions.md`（14 节：props/优先级/覆盖/权限/焦点/runner/Q-01 全冻结），不在此任务重写服务 | 探针 P-1/P-2 pass（`file_dialog.rs` 嵌套 modal + ui.focus 断言，基线 8+10 门同批绿）；P-3/P-4 留 T-04/05；Reader009 证据树已不存在（已记录，截图对照改在本仓夹具） | 01…06 |
| T-02 | work / T-01 | [✅ 已完成] `ui/file_dialog/{model.rs,mod.rs,component_io.rs}` 追加四模式/保存验证/覆盖状态与 identity/会话冻结/目录验证；新命令 saveName/selectCurrent/confirmOverwrite/overwriteBack/requestClose；在现有两个 tests 落 V-01…04 反例（含受控 RaceFs 竞态、Win 只读真实拒绝、字节/名称集无写入对照） | `cargo test -p auto-ui-runtime --test file_dialog_model --test file_dialog`：16+10 全绿（基线 10+8 不退化） | 01…04 |
| T-03 | work / T-02 + PLAN-086 已交付并固定提交 | [✅ 已完成（R-1 已修复 c3f604fd）] `packages/file-dialog/file_dialog.at` 四模式视图（投影 fdMode 分支/目录选择+进入双动作/选择当前目录/保存文件名行/嵌套覆盖 modal/Esc=RequestClose/每会话 ui.focus）；package.at 0.2.0；mailbox 新命令 camelCase 对齐（selectCurrent 等 serde tag 实证修正）；086 pin=master 765251b9（packages/pagination 在树） | `cargo test -p auto-ui-runtime --test file_dialog` 13 绿（新增 V-01 组件目录回路/V-03 Back+Replace/模式冻结 V-04）；tools file_dialog_component 3 绿；PaginationControl 绑定逐字保留 | 01…04 |
| T-04 | work / T-03 | [✅ 已完成] `examples/file-dialog/selector.at` 加 选择目录/保存文件 入口+env 旋钮；`041-auto-edit` app.at 包导入（五级相对路径）+保存模式实例+另存为菜单/action，editor_store.at 写入臂 WriteActive（写后读回比对）/ActSave 重接/AUTO_SAVE_PATH 退役；视图落点=App 根视图（T-01 决议） | 组件级 V-05 断言随 T-03 测试绿；真实窗口两示例由 T-05 两门实跑（auto.exe=组内 auto-man 补丁版 + auto-ui-tool 本树构建） | 02,03,05 |
| T-05 | work / T-04 | [✅ 已完成（R-1 修复后两腿复跑 pass c3f604fd）] 三脚本扩展+新建 consumer gate；§6 适用门全执行 | Win：全门 900x700 **pass**（1003 项 11 页/取消重开 10/系统可达性/字节基线不变 + 目录/保存/覆盖全流程）；窄窗 420x560 visual-only **pass**（V-08 四态截图）；tools 3/3；consumer **pass** 15/15。Linux：构建链全通（auto-ui-tool/auto 本组码均建成）但 `auto run -r vm` 在 WSLg/Xvfb 下 wgpu `Invalid surface`（WGPU_BACKEND=gl 同）——**not_run**，显式 pending closeout gate（086 先例），补跑入口见 evidence.md | 01…06 |
| T-06 | work / T-05 | [✅ 已完成] `packages/file-dialog/README.md`（视觉指南 §2.1 链接/四模式/闭包）、夹具 README、`docs/reviews/plan084/{decisions,evidence,spec-delta-proposal}.md`；Q-01 复现×2 → **DEBT-006** 登记（docs/plans/debts.md 主检出），受影响视觉 AC 保持未关闭，Plan 保持 executing | 证据/命令读回核对；Q-01 未闭环 → 保持 executing（策略具名依赖路线） | 06 |
| T-07 | review / T-06 | [✅ 已完成] 两轮：final review needs_fix（R-1 独立复现）→ 修复后 targeted 复审 pass（oracle 控制变异：缺陷必红/修复必绿）；AC-01…06 逐项结论入档 | reviewed commit c3f604fd→7ea71f0c（rebase 映射记录）；AC-06 带 pending gates | 07 |
| T-08 | merge / T-07 pass | [✅ 已完成] `docs/specs/ui/file-dialog.md` SD-01..03 落 canonical（sha256 621f8324…）+ ff-only 合入 7ea71f0c + 归档/清理收据（见 §9 merge receipt）| 实际路径/提交/归档/树状态核对完成 | 08 |

## 9. 复审记录

- 2026-11-13：stage: new；PLAN-084 r1；outcome: pass（规划合同可交接，非产品通过）；next: work，在后续明确启动时从 T-01 开始，T-03 等待 086。
- 本轮新增 T-01…08、AC-01…08、SD-01…03；0/8。未实施、未运行拟议测试、未修改 canonical Specs、未创建工作树或触碰 Reader。

2026-11-14：stage: new；PLAN-084 r2；outcome: pass（视觉交接可供执行，不是实施通过）；next: work，从T-01进入、原086依赖保持。新增§5视觉引用和V-08，AC-06/T-05/06纳入对照；功能范围、数据owner、8任务与0/8不变。图与提示保存在仓库assets，本轮未运行产品门或创建组件实现。

2026-11-14：stage: work | plan_id: PLAN-084 | plan_revision: r2 | outcome: pass（含显式 pending closeout gates，见§10）| worktree: D:/autostack/.wt/ui-084/auto-ui | branch: codex/ui-084-file-dialog-modes | base_commit: a6f776c2 | code_commit: 16b03b6e / bfb3aec8 / 3d592322 | dependency_revisions: auto-lang e0cfa2d8 / auto-web 8b1742a / auto-down 58ea33f / auto-man 54fe407（全 detached 只读组内同级）；PLAN-086 pin=765251b9 | task_ids: T-01…T-06（T-07 review / T-08 merge 未动）| evidence: docs/reviews/plan084/{decisions,evidence,spec-delta-proposal}.md + plan-artifacts/plan-084/{vfd-full-win,vfd-narrow-win,consumer-win} 三 report.json outcome=pass | blockers: 无需用户决策的阻塞（三 pending gates 已具名，owner=review/merge/DEBT-006）| next_responsible_role: review（同树独立复审，AC-07）

## 10. 待澄清事项

T-01 决策已全部落 `docs/reviews/plan084/decisions.md`（14 节，探针 P-1/P-2 pass）。work 阶段完成时留三显式 pending closeout gates（均不删门、不静默）：

1. **Q-01 渲染器字形丢失（DEBT-006）**：本计划复现两实例（分页文本截断 + 路径反斜杠缺字），MCP/state 文本完整——归因公共渲染器文本 shaping（input/text 两原语），FileDialog/分页包非肇事方。具名修复依赖已登记 `docs/plans/debts.md` DEBT-006；AC-06 的 Q-01 项与受影响视觉断言保持未关闭，随该债关闭。
2. **Linux native 门 not_run（路由变更，2026-11-14 用户决定）**：不再使用 WSL 做 UI 端验证。等自制 Linux 发行版就绪后，在 VMware 虚拟机里单独验证 Linux 端 UI（三脚本同款入口，out-dir 换新目录）。此前 WSL 构建链工作（本组码 auto/auto-ui-tool 均建成）与 wgpu `Invalid surface` 现象记录保留在 evidence.md 供参考，WSLg 修复不再 chase。AC-06 的 Linux 条款未验收，owner=自研发行版就绪后的 VMware 验证轮。
3. **统一回归**：owner=review/merge（仓档位），work 冻结后执行。
4. **R-1（已修复闭环，2026-11-14 work commit c3f604fd）**：根因实证 = `if .fdMode != "singleDirectory"` 包裹把类型按钮组落进堆叠 Column（MCP 快照可见 col{书籍,图片}），input 随之越界拉伸——非渲染器 flex 缺陷。修复 = 类型组改嵌套 shrink-0 行（固定宽度单行）+ input 去 max-w 帽改 flex-1 min-w-0 全伸缩（用户冻结方向）。永久 oracle = 组件级 `filter_buttons_share_one_row`（堆叠列缺陷下必红）；900×700 与 420×560 两宽度实拍对照（r1-fixed-crop.png / r1-fixed-narrow-crop.png），全门+窄窗两腿复跑 pass。

非阻塞用户决策面：无。macOS/浏览器本地 FS 不在首版支持声明中。

stage: goal | plan_id: PLAN-084 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-084/auto-ui | commit: 3d592322 | next: review

stage: review | review_scope: final | reviewer_identity: 同会话独立重建（非独立模型/会话；verdict 全部基于工件：提交 diff、测试断言、report.json、截图重核，非执行摘要）| plan_id: PLAN-084 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-084/auto-ui | branch: codex/ui-084-file-dialog-modes | reviewed_commit: 3d592322 | base_commit: a6f776c2 | dependency_revisions: auto-lang e0cfa2d8 / auto-web 8b1742a / auto-down 58ea33f / auto-man 54fe407；086 pin 765251b9 | spec_inputs: docs/specs/ui/file-dialog.md（含 086 迁移后版本）+ 提案 docs/reviews/plan084/spec-delta-proposal.md | acceptance_results: AC-01 pass（旧模式回归 10+8 绿+tools 3/3）/ AC-02 pass（模型+组件+全门目录记录）/ AC-03 pass（V-02/03 反例+字节基线+consumer 读回）/ AC-04 pass（V-04+ui.focus 探针+生命周期门）/ AC-05 pass（consumer 15/15 真实窗口无 Env 旁路）/ AC-06 fail→partial（Win 两腿 pass，但 R-1 过滤行布局缺陷实锤；Linux 门 per 用户决定改道 VMware+自研发行版，显式 pending；Q-01=DEBT-006 开）/ AC-07 本 receipt / AC-08 pending | pending_closeout_gates: Linux 门（VMware 轮）+统一回归+DEBT-006 | findings: R-1（needs_fix，独立复现 r1-filter-row-crop.png，关闭 oracle 见§10.4）| evidence: plan-artifacts/plan-084/ 三 report.json + 截图 | next: work（R-1 修复后同树复提，重跑受影响门）

2026-11-14：stage: work | plan_id: PLAN-084 | plan_revision: r2 | outcome: pass（R-1 修复，repair scope=R-1+受影响腿）| worktree_path: D:/autostack/.wt/ui-084/auto-ui | branch: codex/ui-084-file-dialog-modes | base_commit: 3d592322 | code_commit: c3f604fd | task_ids: T-03/T-05 重开项 | evidence: 组件 oracle filter_buttons_share_one_row（13+16 测试全绿）+ 全门/窄窗 report.json outcome=pass + r1-fixed[-narrow]-crop.png | repair 检查面 = R-1 关闭 oracle + 两腿复跑；其余证据沿用（consumer 041 无 filters 不受影响；tools 不涉布局）| blockers: 无新增 | next_responsible_role: review（targeted 复审 R-1 闭环）

stage: review | review_scope: final（repair re-review, scope=R-1）| reviewer_identity: 同会话独立重建（oracle 控制变异验证）| plan_id: PLAN-084 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-084/auto-ui | branch: codex/ui-084-file-dialog-modes | reviewed_commit: c3f604fd | base_commit: 3d592322 | dependency_revisions: 同前 | spec_inputs: 同前（spec-delta-proposal.md 冻结副本随 merge 折叠）| acceptance_results: R-1 闭环（oracle 缺陷必红/修复必绿变异实证 + 两腿 pass + 两宽度截图）；AC-01..05 沿用不变证据；AC-06 pass-with-explicit-pending（Linux=VMware 轮 pending、Q-01=DEBT-006 开、统一回归=merge/ review 门）；AC-07 pass；AC-08 pending | pending_closeout_gates: Linux 门（VMware 轮）+统一回归+DEBT-006 | findings: 无未决 | evidence: plan-artifacts/plan-084/ | next: merge

2026-11-14：stage: merge | plan_id: PLAN-084:r2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-084/auto-ui | branch: codex/ui-084-file-dialog-modes | prepared: canonical diff docs/specs/ui/file-dialog.md (+60/−6, SD-01..03) 基于 reviewed c3f604fd，delivery commit 981ed4fd→rebase 7ea71f0c | landed: master ff-only tip=7ea71f0c（rebase old→new 映射 16b03b6e→5cfc5a6b / bfb3aec8→109a782e / 3d592322→8baebf52 / c3f604fd→5f700892 / 981ed4fd→7ea71f0c，git range-diff 全等）| canonical_verified: docs/specs/ui/file-dialog.md sha256 621f8324492ebb75398be5de3bb61d6326f6bc066ad38a2f215cd163823ae5dc（工作树与 landed commit 一致）| archived: docs/plans/archived/084-file-dialog-folder-and-save-modes.md（status: archived，current_step 8/8）| cleaned: 五工作树（auto-ui + 四 detached 依赖）+ dev 分支删除、组目录移除、wt-guard clean、plan-artifacts/plan-084/ 按 DEBT-006 具名 hold 保留（owner=DEBT-006 渲染器修复计划，closure=该债关闭时删）并登记 registry | pending_closeout_gates: Linux 门（VMware+自研发行版轮）+ 统一回归 + DEBT-006 | landing-not-deployment: 本仓无生产运行时制品消费此变更（组件库+示例；示例经源码门运行），无 stale artifact 项 | batch-regression-due: auto-ui 规则未命中（非 auto-lang 档；084 非 5 倍数）| next: done

stage: goal | plan_id: PLAN-084 | plan_revision: r2 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | commit: 7ea71f0c | next: merge

stage: goal | plan_id: PLAN-084 | plan_revision: r2 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: 已删除（wt-guard clean ×5，组目录移除，dev 分支删除）| plan-artifacts: DEBT-006 具名 hold 保留并登记 registry | 交付链: a6f776c2 → fd2ff183(083) → 7ea71f0c(084 delivery) → 1f4e8760(archive) 线性 | done
