---
plan_id: PLAN-094
status: archived
completion_kind: delivered
feature_name: PromptDialog 文本输入对话框
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-20
plan_revision: 3
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/prompt-dialog.md]
touched_goals: []
---

# PLAN-094 PromptDialog 文本输入对话框

## 0. 变更摘要

补齐组件库路线 CL-04 的文本输入对话框，提供标题、说明、单文本输入、本地必填校验、业务错误、确认/取消和提交中状态。用于重命名、创建名称和分组命名，文件名规则、唯一性、权限与实际写入由应用负责。见[持续跟踪 strategy](../../design/autoui/component-library-extension-strategy.md)。r3 按用户决定仅交付 Windows。Windows Native VM 与 Vue 已通过独立复审并落入主分支。交付、归档和清理均已核对，8/8。

## 1. 目标

Windows native VM 与 Vue 使用相同公共接口、状态语义和普通控件。用户键盘/鼠标与 MCP 驱动同一条事件链，自动测试不要求系统文件窗口或操作系统前台焦点。提供正式可移根组件包、普通消费示例、现有消费者迁移及无人值守证据。

非目标：多字段表单、密码/富文本输入、通用验证语言、文件系统服务、撤销文件操作、系统原生弹窗。本计划平台范围为 Windows。Linux 与 macOS 留待独立验收。未来平台结论需绑定该平台的真实构建、窗口、字体、MCP 和生命周期证据。

## 2. 架构方案

调用方 open/contextId/revision/initialValue → 实例草稿和局部校验 → submitRequested → 应用校验并提交 → 匹配的 response 与受控 open。普通重绘保留草稿，新的会话或目标修订重新初始化。

新增正式 `packages/prompt-dialog/`，公共名为 `PromptDialog`。复用标准 Dialog/Input/Button/List/Progress 和既有主题/包机制，仅按本组件需要取用。编译器注册优先级 Builtin > Local > Package；禁止靠同名包覆盖已登记控件。模型/协议尽量单源 `.at`；必要共享运行时接缝须最小化并验证 native/Vue 等价，不造两份独立算法或 compiler→runtime 依赖。

应用持有业务数据、权限、I/O、持久化及执行效果；组件持有焦点、草稿、选中和防重复请求状态。普通重绘不换实例身份；context/会话真正变化才使旧动作失效。公共可见操作必须能按语义名称发现，不能只靠坐标或数组下标。

## 3. 技术栈

沿用 `.at` 正式包与普通消息、Rust compiler/runtime/tools、native VM、Vue及现有主题。测试复用 `crates/auto-ui-tools/tests/` 包导入/移根模式和 `tests/product/` 的进程、端口、MCP证据做法；Vue 使用真实浏览器操作。仅因应用需要服务时启动隔离的已有服务，不建设新的测试平台。

## 4. 需求分析与背景调查

2026-11-20 用户以 `/goal 完成计划094` 授权在 r2 合同内执行、独立复审、限内修复、合并、归档及清理。实施树 `D:/autostack/.wt/ui-094/auto-ui`，分支 `codex/ui-094-prompt-dialog`，基线 `9bdbbcf859e5185335014425af5e74f5512e8c29`。用户在 T-01 确认“只做win的吧”。r3 将交付及验收范围改为 Windows native VM 和 Windows Vue，继承实现、复审、合并及清理授权。T-01 确认 VMware 中的 AutoOS 测试机有 Live 和安装器欢迎页记录，发行版 PLAN-001 的硬盘安装与重启仍待验收；Linux 两门退出本计划验收矩阵。

授权来自2026-11-14用户请求：为四组件分别补齐实施计划，复用既有实现、引用已生成参考图、写明跨平台/MCP/无人值守验收并更新 strategy。本轮动作限本仓计划、设计导航及引用；不实施组件、不启动实施树、不提交其他WIP、不修改兄弟仓。用户未指定模型、预算、并行代理或自动实施；这里不推定授权。

2026-11-14本次复核HEAD `3cab09943079a1d1fca993fa5a4c4dd2a9fa8315`；084交付 `7ea71f0c`，086交付 `765251b9`。本次用户授权再次审查/补齐计划并推荐下一项，仍无实施、建树、跨仓写入授权。补读当前FileDialog/Pagination Spec及Windows系统语义合同；下列原始源文件摘要复核未变。

初稿观察基线 auto-ui HEAD `a6f776c24a4ba98611849955edceb5a04edac969`；未提交来源采用下表摘要/路径，不当作已交付版本。已读 [Spec总览](../../specs/README.md)、`ui/{examples-ownership,theme-scope-and-publication,structure-closure}.md`、[原生验证合同](../../specs/runtime/native-vm-app-validation.md)、`build/ui-test-selection.md`。当前无本组件专门Spec，拟在交付时新增；当前Spec不能因草案而改成已支持。

| 已核来源 | 复用边界与现存差额 |
|---|---|
| `examples/ui/027-file-manager/src/front/app.at` | SHA256 6306D7D624ABA15DED6F2B990EEA11FBEADBF362A634185F570A8E637DE1F7B8；StartRename 捕获 rename_path，CommitRename 现在先关窗再验证并 fs.rename；新建也有独立 AlertDialog/Input。复用稳定路径及应用操作，改为失败保留表单。 |
| `crates/auto-ui-runtime/src/ui/aura_view_builder/tags_panels.rs` | 复用现有 Dialog/AlertDialog、模态阻挡及焦点机制，不另造系统窗口。 |
| `packages/file-dialog/、examples/ui/027-file-manager/` | 参考正式包导入和受控对话框；027 的非 VM 写操作明确不可用，此边界必须保留。 |
| `packages/file-dialog/`、`crates/auto-ui-tools/tests/file_dialog_component.rs` | 正式包导入、依赖闭包、移根和实际控件测试可复用；不复制文件业务/专用就绪协议。 |

实施 T-01 必须重新读取上述来源并记录实际 commit/hash、工具版本和脏文件边界。Reader 路径是当前主检出的只读参考，不依赖已消失的 `.wt/reader-009/`；Reader产品迁移、vendor同步和发布另属其自身计划。四项094–097相互无硬依赖，也不等待087；不为复用抽象平白新建第五个前置项目。

## 5. 详细设计

### 公共接口与所有权

| 接口 | r3 合同 |
|---|---|
| props | open:bool=false；contextId:str=""；revision:int=0；title/description/label/placeholder:str；acceptLabel:str="确定"；cancelLabel:str="取消"；initialValue:str=""；required:bool=true；trimInput:bool=false；pending:bool=false；errorMessage:str=""；response:str=""（JSON） |
| 公开消息 | submitRequested(str)、cancelled(str)、error(str)。提交携 seq/session/contextId/baseRevision/value；取消也携身份和 reason=cancel/escape。seq 在同实例内递增，session 每次 false→true 加一。 |
| 反馈 | response={instanceToken,session,seq,contextId,status:"accepted"或"rejected",message,retryable:true}；只接纳当前未完成请求。accepted 后等待应用将 open=false，保持锁定；rejected 解锁并保留草稿/显示 message。外部 pending=false 不能提前解除本地重复提交锁。 |
| 校验/文字 | 原文默认保留，trimInput=true 时只裁剪首尾空格/Tab/CR/LF；required 检查裁剪后的非空性。中文按原文往返，不做 Unicode 归一化。唯一性、文件名、长度等业务规则由应用响应；组件不默认套用 Windows 文件名规则。 |
| 受控变更 | contextId/revision/initialValue 的显式改变开启新编辑基线并撤销旧请求身份；open=false、卸载也使旧反馈失效。errorMessage 仅当前目标的外部错误，不能替代携身份的异步 response。 |

事件载荷示例（JSON字符串，字段名固定；不是新MCP接口）：

```json
{"instanceToken":"app-1/mount-17","seq":3,"session":2,"contextId":"notebook-7","baseRevision":4,"value":"Reading journal"}
```

`.at` 公共消息沿既有包的可导出命名方式声明（例如 `"changed"(str)`），由普通 `onchanged`/对应消息绑定消费。上表类型是接口语义，T-01只核对当前语法/封装方法，不把接口再交给执行者自由重设计。普通 props 数据通过组件正常输入提供，不从测试环境变量注入成功结果。

### 身份、反馈与复用接缝

每个实际挂载实例分配不复用的 `instanceToken:str`，普通重绘保留，卸载重建/另一个实例必须不同；其来源绑定App及真实组件owner/generation，不使用label、列表索引或每次渲染时间戳。所有请求、取消/关闭消息和异步response均增加此字段，调用方原样回传。T-01证明native/Vue中的可用分配机制；优先复用现有实例生命周期，只在确有缺口时补最小共享接缝。重建后seq/session即使重新从1开始，旧token也不能命中新实例。

同revision出现影响动作含义的数据变化也必须失效旧绑定，不能依赖调用方永不漏增revision。迟到消息先核对instanceToken、当前目标及活动请求，再处理业务状态；排队的用户动作还须在消费时检查可见、启用、当前modal与数据身份。两个同名实例、隐藏重开、卸载重建均配当前有效操作正例，禁止把所有动作都拒绝作为“隔离通过”。

复用 `crates/auto-ui-runtime/src/ui/modal.rs` 与 `ui/iced/renderer/host.rs` 的modal收窄、`ui.focus`/焦点路径，不另写全局键盘服务。配置错误首次挂载和相关配置变化必须可见；相同错误不每帧刷回调。保留 `error(str)` 约定，T-01验证首次配置错误恰好一次通知的正常调度路径：已交付分页Spec说明native子组件Init内emit不派发，不能只在Init中发事件。若需最小接缝修复，归本Plan T-01/03并验证兄弟组件，不默默删除事件合同。

### 状态、交互与消费适配

- 初始open=true也算新会话。Cancel/Escape先设置cancelled-await-close锁，再发一次cancelled；父层尚未关窗时重复按键零事件。accepted响应必须在同一props快照与成功后的open=false/数据发布，或保持accepted-await-close锁等待关闭；同快照的revision更新不能先解锁导致再次提交。外部关闭不再额外发cancelled，不把已成功写入解释为取消。
- 业务拒绝可带retryable：重名/非法名称使用false，当前被拒草稿的确认按钮禁用，改输入后清错并恢复本地校验；可恢复I/O错误使用true，保留草稿并允许同值重试。外部errorMessage仅展示当前目标一般错误，不是异步请求确认。必填空白禁用确认；required=false允许空串，trimInput=false仍原文提交。027传“重命名”/“新建”等acceptLabel，不能把业务名称写死在通用包。

- 状态 closed→editing→invalid 或 submitting→editing/rejected 或 accepted-await-close。合法点击/Enter 先同步锁定，再发一次请求；失焦不提交。无效必填保留输入、不发 submitRequested。组件不会因为发出请求就关闭。
- 编辑时 Escape/Cancel 放弃草稿并发 cancelled，由父层关闭；背景点击不关闭。submitting 期间输入、Confirm、Cancel、Escape 均不可产生新动作；应用可主动关闭，关闭不代表撤销已经发生的业务写入。父层必须在执行前核对 contextId/revision，且负责事务和失败恢复。
- 打开聚焦输入（默认选中初值）；Tab 在弹窗内循环，关闭恢复到仍存在的触发控件，否则回到应用安全焦点。IME 组合输入中的 Enter 不提交，错误关联输入。确认与取消是实际 Button，错误是可见文本。
- 新建普通 examples/prompt-dialog/ 消费者，持 notebook/group 稳定 ID 和名称集合，以真实模型修改演示重名拒绝/成功；其结果可由普通页面列表读回。027 重命名和新建名称迁入本组件，操作仍在应用内；Vue 保留只读文件系统提示，不用内存成功冒充真实写盘。
- 027 原生实测在本轮创建的目录中重命名/新建：校验源/目标存在性、文件内容摘要、重复名称失败以及取消后目录不变；使用完整 rename_path 和当前目标身份，刷新/重排不能按旧行号重命名另一个文件。

具名配置错误采用 `error(str)`，至少含 code/message；组件应显示错误并阻止相关修改，不用空列表或静默截断掩盖非法数据。业务拒绝与配置错误分开呈现。公开规模不承诺无限性能；不得未经记录默加“只支持前N项”的截断。新增资源上限或不兼容约束需先修订合同，不能只改测试。

### 视觉引用

固定普通消费数据：`notebook-7=Inbox`、`notebook-8=Notes`。对7输入Notes由应用拒绝并留窗，输入Reading journal成功且只更新7，取消时两项不变。文件消费夹具在独立目录建立`old.txt`（已知内容）及`occupied.txt`；重命名成`new.txt`保留内容，重名不能覆盖occupied，取消不能改变目录。这些是T-04/05待建数据，不是已运行证据。

直接复用[已生成的四组件参考图](../../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1.png)及[文件导航/轻量交互视觉指南](../../design/autoui/file-navigation-and-interaction-visual-guide.md)，只实现**左上：重命名表单；Notes 重名错误与 Reading journal 有效状态是两个独立示例**。[生成提示](../../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1-prompt.md)和最终PNG均为仓库设计资产，不依赖attachments或个人缓存。PNG SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`，1586×992 RGB。

样式沿[公共执行/视觉指南](../../design/autoui/common-components-execution-and-visual-guide.md)：32px控件、8px间距、16px面板padding、16px outline图标、约6px圆角，语义主题色和focus ring。图不是界面纹理或尺寸像素金标准；中文/长标签/错误、浅深色及420×560小窗以真实GUI可读可操作为准。图中文字不覆盖上表状态/异步合同。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/prompt-dialog.md | 无正式专门合同→上述受控参数、消息、校验和身份/异步状态规则 | 收拢应用重复交互，保留业务owner | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/prompt-dialog.md | 无本组件发布/验收约定→包导入/移根、普通消费者、跨端MCP/视觉/无人值守门及平台边界 | 可复用、可操作、可验证 | AC-04,AC-05,AC-06 |

拟实施树 `D:/autostack/.wt/ui-094/auto-ui`，分支 `codex/ui-094-prompt-dialog`，本轮未创建。实现、自检、独立最终复审依次使用这一棵树；Plan进度留主检出，不在另一棵树同步实现。

执行入口是 T-01：来源已核、接口和AC已定，当前无外部组件交付阻塞。先走完整最小链路：**正式导入 PromptDialog→初值→普通输入→submitRequested→应用拒绝/接受→关闭重开；native VM/Vue 均验证 session、焦点和 JSON 消息，确认027 VM 检测及 fs.rename 实际入口。**。产物 `docs/reviews/plan094/decisions.md`（新增）记录可行路径、实际语法/命令、版本、待修接缝和探针证据；不得只看schema或源码文本。探针失败先在原目标内修接缝；若必须改平台/API/业务owner或AC阈值，则修订Plan并保留原要求，不擅自降级为截图/静态通过。

## 6. 测试设计

### 区分正确与错误实现的必需用例

下表每行 owner_stage=work、执行任务=T-05；入口是新增 `tests/product/verify_prompt_dialog.py` 的具名case，以及新增 `crates/auto-ui-tools/tests/prompt_dialog_component.rs` 中的模型/包边界反例。应用效果必须由正常页面/应用持久读回确认，不能通过直接调组件回调来制造成功。

| case_id | acceptance IDs | 代表输入/状态 | 可观察oracle / 失败对照 |
|---|---|---|---|
| P-01 | AC-01,AC-02 | required 空白输入、中文名称、有效重命名 | 空白零提交且错误紧邻；中文完整往返；合法修改仅一条业务事件，列表显示新名称。 |
| P-02 | AC-02,AC-03 | 重名拒绝、Enter 后再次点击、pending 中 Escape | 重名留窗留草稿；一个 seq 最多一项写入；pending 禁用关闭而不虚报取消。 |
| P-03 | AC-03,AC-04 | 打开 A 后切 B/主动关闭重开，迟到 A response 与旧 MCP 节点 | B 草稿不变；旧节点不得写 B；连续正常输入不因每次重绘失焦。 |
| P-04 | AC-04,AC-05 | 027 原生隔离文件 rename/new，Vue 同动作 | 原生源消失/目标内容摘要相同，冲突和取消零写入；Vue 明确不可用且无磁盘或后台替代写入。 |
| P-05 | AC-04,AC-05 | 普通示例双实例、IME、Tab/Enter/Escape | 模型按正确 contextId 修改；IME 未完成不提交；焦点围栏/恢复正常，无系统前台焦点前置。 |
| P-06 | AC-02,AC-03,AC-05 | 成功反馈与revision/open同帧、连按取消；重名与临时I/O拒绝 | 成功不重新解锁/重复写入，取消仅一次；重名相同草稿禁用但编辑后恢复，临时失败同值可重试；两种按钮文字由props提供。 |
| X-01 | AC-01,AC-06 | 正式包从另一目录导入；缺依赖、同名内置并存 | 无绝对开发机路径，实际导入生成并运行；缺依赖具名失败，内置旧示例仍能解析运行，不以搜索字符串通过。 |
| X-02 | AC-04,AC-06 | Windows native VM 与 Vue，当前树动作及关闭重开 | 真实窗口/非空树、实际控件动作与应用读回相符；错误对照返回明确失败，退出后本轮进程/端口回收，再次启动可用。 |
| X-03 | AC-06 | 浅/深主题、中文长标签、420×560、错误/禁用/pending | 同版本真实截图可读、正文可滚动、主要操作可达，无glyph缺失、重叠或误点击；空/加载态按组件适用状态覆盖。 |
| X-04 | AC-02,AC-03 | 卸载重建后seq重复、同revision数据变更；首次非法配置及修正 | 新instanceToken拒旧反馈，旧动作零误写且当前正例可用；配置错误可见且每次配置变化恰好一次error，修正后正常恢复。 |
| X-05 | AC-02,AC-04 | 多输入/双实例/嵌套modal；组合输入适用时 | MCP发现并操作当前目标，不把首个input代理误当真实焦点；底层按钮/快捷键被阻挡，Tab/关闭恢复正确；输入法按下文分层证据验证。 |
| X-06 | AC-04,AC-06 | 浏览器GUI通过但MCP不可用、启动/动作中途失败 | GUI不能替代MCP记录；缺必需能力整体not_run/非零退出；失败也执行finally清理、保存最后树与日志，原有进程不受影响。 |
| X-07 | AC-04,AC-06 | Windows原生真实系统树的输入/按钮 | 按native-vm-app-validation及native-system-accessibility核对受影响标准控件的role/label/value/focus/bounds/action；不以MCP树冒充系统provider或物理输入证据。 |

共同不变量：一次合法手势至多一条当前身份业务请求；非法/禁用/旧动作零目标变更；业务拒绝不伪造成功；接受后的真实消费状态与事件一致。组件树状态断言与消费者读回必须相互独立。测试通过“输入正确但状态owner写错/重复执行/旧对象覆盖”的反例来区分 plausible 错误实现，不能只计绿测试数量。

### 运行入口与证据合同

验收脚本负责启动对应普通消费者。先准备工具、消费者及其声明的后端，再逐项运行。原生验收不要求手工切换前台焦点。

Vue 准备阶段在生成项目中安装锁定依赖，再运行 `python tests/product/prompt_dialog_preview.py <已生成的 Vue 目录>`。该命令使用 `NODE_ENV=development` 构建开发包，并为输入和全部 `dist/` 资产生成 SHA256 收据。验证器检查准备收据，再启动禁用配置文件加载的静态 preview。`reopen` 是两轨分别运行的整应用生命周期用例，确认首个会话退出并释放资源，再在新会话取消编辑并读回应用的取消状态及零保存计数。其他具名用例的退出码表示该用例的结果，两轨验收各自还须通过 `reopen`。

027 验收使用事先准备的 Rust API（Native）或 VM 只读 API（Vue）、独立文件数据及图形缓存。Windows Native 的夹具 `LOCALAPPDATA` 与 `APPDATA` 均限制在该夹具根，在测量前只预置已确认来源的图形缓存与种子文件。准备与用例实际墙时分别记录。

```text
cargo test --locked --offline -j 2 -p auto-ui-tools -p auto-ui-runtime --features auto-ui-tools/ace-vue --test prompt_dialog_component --test prompt_dialog_ime --test file_dialog_component --no-run
python tests/product/verify_prompt_dialog.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对工具路径> --out-dir <本轮独立证据目录> --backend native --app-dir <隔离应用源码目录> --case <用例>
python tests/product/verify_prompt_dialog.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对工具路径> --out-dir <另一独立证据目录> --backend vue --app-dir <已生成的Vue目录> --case <用例> --browser-mcp-config <浏览器MCP配置路径>
```

native采用 `auto run -r vm`，保留应用声明的真实后端（不能把`--server vm`当原生前端）。Vue及MCP入口按下文分层合同验证。只跑本组件、被修改公共接缝和迁移消费者的必要回归，保留既有oracle，不叠加无关整仓测试。

无人值守runner必须：每轮隔离数据/目录/端口；native校验自身拥有的OS窗口和非空live MCP树，Vue核验本轮页面/浏览器会话和实际MCP提供方；由当前snapshot发现真实节点再find/action/type/键盘/scroll/screenshot；动作后正常事件→应用读回；包含一项明确失败负对照；正常退出、核验本轮拥有的子进程和端口已释放并重开。原生动作不能依赖OS前台焦点，不能使用系统dialog、`AUTO_OPEN_PATH`、直接改组件状态、调用accepted处理器或回放旧node ID。超时须有限且保存最后树/日志，不能无限重试掩盖失败。

分别记录 **Windows native VM** 和 **Windows Vue** 的 pass/fail/not_run，两门均为必需。Linux 和 macOS 不在 r3 验收矩阵内。浏览器无宿主/磁盘权限时保留不可用结果，不放宽安全边界。

收据 `docs/reviews/plan094/` 持有实际版本/依赖闭包摘要、命令/退出码、case→AC矩阵、截图/树、业务事件及读回结果、负控、生命周期与必要回归。产物大日志放独立证据目录，文档引用可复现位置和摘要；不要把个人缓存当唯一凭据。Q-01文字glyph缺失不能因MCP文本完整而结案；若公共渲染器超范围，建立具名修复依赖，受影响AC-06保持未关闭。

本轮只做草案静态自检，无产品验收结果。用户未分配实现/内部检查/最终评审模型；未来执行者先自检，最终评审须独立上下文并绑定实现revision，同一实施树顺序复用；不据此自行启动其他agent/chat。

### 可执行门与证据分层

- 每个平台均拆为普通GUI交互、MCP工具调用、应用读回、适用视觉/输入桥、退出重开；不要求把全部边界case在GUI和MCP重复笛卡尔积。§6每个case记录实际测试层；模型测试可直接喂模型，产品路径必须通过普通控件，不能用模型测试冒充真实消费。
- native复用 `crates/auto-ui-runtime/src/ui/mcp_server.rs` 的 `autoui_snapshot/action/type/keyboard/screenshot` 正常工具；保留RPC请求/结果和当前节点发现。Vue复用已存在的浏览器MCP提供方（或已交付生产桥），从真实DOM/可访问树进行输入/点击/键盘，记录提供方、工具版本、调用/结果和应用读回；Playwright只操作同一真实页面，不能单独作为“MCP通过”。本Plan不新建Vue MCP服务器；T-01定位当前配置，缺提供方则Vue MCP门not_run，不由四组件分别造桥。
- product脚本保留原拟议CLI，Vue增 `--browser-mcp-config <绝对配置路径>`（T-05待实现，配置描述实际提供方/会话接入；凭据不进报告）；两种driver共用case断言。native的OS窗口归属以owned PID/handle证明；Vue验证本轮生成页面URL、浏览器会话和服务PID，不套用native HWND证明。两条腿均有真实MCP操作与独立业务读回。
- Prompt/Command/Tag的IME门：普通 `autoui_type` 是整串替换，不证明composition、物理按键或OS输入法。Vue输入桥用compositionstart/update/end与isComposing事件序列；native使用当前标准输入适配层的preedit/commit/取消→Enter事件集成测试，观测生产事件抑制与一次业务提交，并补普通GUI/MCP中文最终值读回。已有 `autoui_desktop` 的acceptance专用handler/key通道不代替普通组件MCP产品门。若适配层无法表达，T-01定位最小输入接缝后补测试；TaskProgress无文本编辑，IME标明不适用，仍测正常键盘/modal。
- 复用当前Spec及已有包而非只复用FileDialog：正式包必须提供 `package.at/pac.at/组件.at/README.md`，native源相对import和Vue既有components/vendor路径解析分别运行；从第二绝对根验证完整依赖闭包和无开发机路径。四Plan不得分别重写同一公共runner/输入桥；第一个发现公共缺口者在decision登记owner/提交，后续重基线复用。
- 每个实际运行门输出 `report.json`：plan/revision/platform/backend、source/tools/dependency摘要、runner和MCP提供方、case/layer/期望/实测/status、截图/树/读回路径及摘要、cleanup/reopen、result。退出码0仅当本次门全部必需case通过且清理重开成功；失败为1，缺必需环境/能力not_run为2。两条 Windows 验收均通过才可汇总为 pass。有效控制与关闭端口的同一MCP方法配对，关闭目标要求有界connection-refused，不能仅HTTP探测代替。
- 默认启动上限300秒、单动作15秒、退出清理30秒；runner可显式配置并记入收据，不能无限等待/重试。失败走finally清理本轮owned进程/端口/隔离数据并保存最后证据；只对确认身份的自有残留强制终止且写明原因。关闭/重开Dialog与退出/重启整个App分别验证；不把关闭UI误认为业务任务取消。

## 7. 验收标准

| ID | owner_stage | 可观察结果及验证方法 | 任务 / 状态 |
|---|---|---|---|
| AC-01 | work | 正式包公共接口按§5导入/生成/运行；X-01正反例、最小普通消费者及包依赖闭包可复现，无内置同名覆盖 | T-01,T-02,T-05；[x] 见修复收据 |
| AC-02 | work | §5正常/错误/禁用/键盘/适用IME规则、本组件case及X-04/05全部有期望断言；无一次手势多次提交或虚假成功 | T-02,T-03,T-05；[x] 焦点与输入 RPC 原始收据已复查 |
| AC-03 | work | 当前身份、重复请求锁、外部改动/迟到反馈/关闭重开/双实例隔离反例通过，旧对象零误写（含卸载重建与同revision内容变化） | T-02,T-03,T-05；[x] 见修复收据 |
| AC-04 | work | 普通GUI和MCP通过当前可见控件完成关键用例；Windows native VM 和 Windows Vue 分别有结果，原生不依赖前台焦点；X-02/05/06/07分层门、生命周期与负控通过，缺必需MCP/平台门不算通过 | T-03,T-05；[x] 独立会话读回与失败清理对照通过 |
| AC-05 | work | 普通名称集合与027原生真实重命名/新建读回一致；027 Vue 保留写操作不可用，Windows 双端普通消费者成功；只改弹窗文案不足以通过。 | T-04,T-05；[x] 见修复收据 |
| AC-06 | work | X-01/03/06/07、移根、必要公共/消费者回归及真实截图通过；中文/暗色/窄窗可用，Q-01若影响本组件则闭环；README/待沉淀Spec与证据齐 | T-05,T-06；[x] 准备、实际行为和资源回收的定向复验通过 |
| AC-07 | review | 独立上下文逐项验证AC-01…06及SD-01/02，绑定 Plan r3和实际实现提交/工作树摘要，结论pass且无必需门not_run | T-07；[x] `b71ff791`，独立最终复审通过 |
| AC-08 | merge | 评审后交付落主，SD-01/02沉淀canonical Spec且内容核验；归档/路线链接/交付及清理收据完整，不把未运行平台标为支持 | T-08；[x] 交付、规范、归档与清理见 §9 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 影响文件/符号与独立产出 | 验证/预期 | AC / 进度 |
|---|---|---|---|---|---|
| T-01 | work | 无；现有来源重核 | §4来源、正式包导入/事件路径；新增`docs/reviews/plan094/decisions.md`，落实§5最小全链探针、instanceToken/首次error/模态焦点/输入桥可行性、真实包命令、平台及MCP提供方/依赖pin | 两端普通控件事件到应用读回；差额已记录，完整产品门见 T-05 | AC-01；[x] `76a05fd5`，见决策记录 |
| T-02 | work | T-01 | 新增`packages/prompt-dialog/`的数据/交互模型、校验、消息与身份协议；沿既有包元数据/相对导入 | §6有效/非法/迟到/双实例模型反例；无业务I/O进入包 | AC-01,02,03；[x] `8f1bb7fe`，包路径具名拒绝，跨目标及同名双实例模型通过 |
| T-03 | work | T-02 | 同包UI/主题/键盘与MCP可见语义；必要现有runtime接缝最小修复 | 本组件case普通GUI/MCP路径可操作，IME/焦点和pending正确；对照参考图布局 | AC-02,03,04；[x] `8f1bb7fe` 和 `8a51fcdf`，原生输入兼容性、双轨嵌套焦点、实际滚动与原生系统动作通过 |
| T-04 | work | T-03 | examples/prompt-dialog/（新增）、examples/ui/027-file-manager/src/front/app.at；复用§4实际业务入口 | §5消费路径输出真实变化；保持原业务权限/数据/模式，不同步兄弟仓 | AC-05；[x] `8f1bb7fe`，027 原生创建失败保留草稿，成功写盘与 Vue 只读对照通过 |
| T-05 | work | T-03,T-04 | 新增`crates/auto-ui-tools/tests/prompt_dialog_component.rs`、`tests/product/verify_prompt_dialog.py`及必要隔离数据 | 运行§6两条产品命令/定向cargo目标；Windows 两门、必需 case/分层MCP/输入桥/负控、失败清理/退出重开与必要旧消费者回归 | AC-01…06；[x] `b71ff791`，修复定向验证见 §9 |
| T-06 | work | T-05 | 新包README、普通示例说明、`docs/reviews/plan094/`证据与待沉淀SD；更新strategy实施事实 | case→AC、版本/截图/读回可追溯；受影响Q-01闭环；canonical Spec暂不冒称已交付 | AC-06；[x] 最新准备入口、证据绑定及原始收据在 §6/9 |
| T-07 | review | T-01…06 | 同一实施树，独立上下文最终评审与修复后的受影响复验 | AC-01…06逐项验收并核对SD；pass绑定精确实现revision，无必需门空缺 | AC-07；[x] `b71ff791`，独立复审 AC-01…06、SD-01/02 通过 |
| T-08 | merge | T-07 pass | 合并/交付、`docs/specs/ui/prompt-dialog.md`沉淀、Plan归档及strategy状态/链接更新、实施树清理 | 核验落主内容与评审等价、Spec字节/规则、交付与清理收据；不写specs.json/index.json/legacy ledger | AC-08；[x] 检入 `a5e12d3b`，归档与清理见 §9 |

任务只保留本表一份当前进度；每项完成写实际证据，不预先勾选评审/合并。Windows 必需门缺口留在对应 AC，不能把产品正确性转给merge阶段兜底。

## 9. 复审记录

- 2026-11-14：stage: new；PLAN-094 r1；outcome: pass；next: work/T-01。新建T-01…08、AC-01…08及SD-01/02；已核来源、复用边界、参考图和平台矩阵，准备进入实施。
- 2026-11-14：stage: new；PLAN-094 r2；outcome: pass（修订后计划可进入T-01）；next: work/T-01。再次复核复用边界/图/平台矩阵，补instanceToken、精确异步确认、现有Init/MCP限制及分层输入证据、VMware路由、具名反例与失败清理；T-01…08、AC-01…08、SD-01/02保持，扩充§5/6并强化AC-02…06。详见[本轮审核报告](../../reviews/component-plans-094-097-2026-11-14/README.md)。原r1记录保留为历史。
- 此pass仅指计划合同可执行；不是组件测试通过、独立实现评审或交付完成。本轮未创建实施树，未执行上列拟新增测试；current_step=0/8。

- 2026-11-20。stage: new | plan_id: PLAN-094 | plan_revision: r3 | outcome: pass | next: work/T-01。用户明确授权仅做 Windows。X-02、AC-04/05/06/07、T-05/07 及 SD-02 的验收和发布证据按 Windows native VM / Vue 两门执行。保留现有任务、组件接口、身份与反馈语义、真实 MCP、Windows 系统可访问性及定向回归要求。r2 下的 Windows 探针保留为机制证据，最终门执行时绑定 r3 与实际实现提交。基线 `9bdbbcf8`，实施树 `D:/autostack/.wt/ui-094/auto-ui`。
- stage: goal | plan_id: PLAN-094 | plan_revision: r3 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-094/auto-ui | commit: 9bdbbcf8 | next: work。授权来自 `/goal 完成计划094` 与“只做win的吧”，覆盖 r3 执行、独立复审、合同内修复、合并、归档及清理。
- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | checkpoint: T-01 | code_commit: 76a05fd5 | outcome: pass（仅 T-01）| next: work/T-02…06。正式包探针在 Windows Native VM（9.334 s）和 Vue 浏览器 MCP（6.818 s）完成受控会话、重名拒绝、中文写回、重开及配置错误一次。原生已证明旧会话节点拒绝与新节点成功，Vue 已证明背景点击保留、Escape 关闭和焦点全选。原生 027 的普通 Rust API 后端加载入口和 Vue 真实目录读取入口分别定位，实际写盘验收属于 T-05。决策及依赖指纹见实施树 `docs/reviews/plan094/decisions.md`。
- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | checkpoint: T-02/T-04 | code_commit: 0e5bd791 | outcome: pass（仅模型、包与消费者迁移）| next: work/T-03/T-05/T-06。模型与移根 2/2，测试总执行 1.08 s，进程墙时 1.386 s。模型反例验证同修订换初值、卸载重挂、旧 token、重复提交、两种重试、成功与 revision 同帧保持锁定。Windows 原生 027 的重名保护与重命名 9.963 s，新建与取消 8.610 s，真实文件校验通过。Windows Vue 对相同 027 操作保持只读，8.101 s、7.869 s。原生标准输入法集成测试 1/1，1.58 s，Vue 组合输入 4.475 s。文件管理准备采用独立的包源、实际 Rust API（原生）及只读 VM API（Vue）。Windows 系统树及 SetValue/Invoke 已取得有效读回，首次完整用例 11.247 s，超过 10 s 门，仍须改善计时和复验。全部报告存于 `D:/autostack/plan-artifacts/plan-094/probe/`。

- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | checkpoint: T-03/T-05 | code_commit: `a9dc2b85a28c3f8448263209ba8be7465e589b5e` | outcome: blocked | next: work/T-05。Windows 名称管理示例六个 Native VM 用例和六个 Vue 用例通过。两轨均核对了错误、中文、Tab、取消、提交锁定与浅深主题。Vue 在关闭后恢复触发按钮焦点。原生实际 UIA 输入节点名称“分组名称”，值 `Research`，焦点为真，输入与确认按钮边界为正。系统 SetValue/Invoke 后应用列表显示“系统输入”。Vue 真实目录读取和写入负控均通过。027 原生最终复验在 MCP 监听后无控件树。按普通 `auto run -r vm --back-port` 使用已准备 Rust API，已重试固定端口、动态端口和新隔离应用根。新根探针 3 秒时已找到本轮 VM 子进程，未找到所属可见窗口，至 8 秒仍返回 `No UI available yet`。两次 20 秒复验亦未出现树。负控后的本轮 MCP 与 API 端口已释放。保留早期成功写盘收据供对照，最终门仍需通过 027 Native。

  | Windows 执行对象 | 用例与墙时（秒） | 证据 |
  | --- | --- | --- |
  | 名称示例 Native VM | names 9.747、cancel 5.862、tab 5.361、visual 6.656、pending 5.044、system 9.061 | `accessible-native-*/report.json`，均 exit 0。实际 UIA 证据见 `accessible-native-system/system-tree.json` |
  | 名称示例 Vue | names 5.381、cancel 4.335、tab 3.181、visual 8.631、pending 3.379、ime 3.776 | `accessible-vue-*/report.json`，均 exit 0。Playwright MCP 在生成页面操作并读回 |
  | 027 Vue | file-rename 8.462、file-create 7.520 | 两项 exit 0，隔离目录内容保持一致。另准备声明的 `stylekit` 后生成和 `vue-tsc --noEmit` 通过 |
  | 027 Native VM | file-rename 23.947、file-create 24.021，启动门重复失败 | 两项 exit 1，`accessible-native-file-*/report.json`。固定端口及新根复核在 `known-port-*` 和 `fresh-root-027.log` |
  | 正式包、输入法与旧组件定向回归 | PromptDialog 2/2，墙时 1.328。Native IME 1/1，墙时 1.540。FileDialog 3/3，墙时 8.664 | 准备耗时 88.356 秒，测试体直接运行准备好的 debug 测试程序，均 exit 0 |
  | 依赖与 MCP 负控 | 缺包 Vue 类型检查 exit 2，明确缺 `PromptDialog.vue`。缺浏览器 MCP 配置 exit 2，报告 `not_run` | `missing-package-typecheck.log`、`negative-no-browser-mcp/report.json` |

  收据根 `D:/autostack/plan-artifacts/plan-094/probe/`。通过用例使用 `auto.exe` SHA256 `eddb2cce50fd8afe08cd26959c2fe52e090eb833b5c8b9a3f27538b28d947fcf` 与 `auto-ui-tool.exe` SHA256 `b27d87af4c04024bff76ebb388683fd50d1b96a49ca53d3a56cc200f1b8fc251`。后续定向测试构建的工具 SHA256 `b50c023e9118a2943156402ea25a6ca5bd532160ff2c2ae1c6e6d135c81fd27a` 上 027 Native 启动亦失败。各用例的源、脚本、截图、树和日志 SHA256 见同目录实际报告。准备与机上其他 Plan 构建错开。规范增量提案暂存在实施分支 `docs/reviews/plan094/spec-delta-proposal.md`。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: work | to: work | status: executing | outcome: blocked | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `a9dc2b85` | next: T-05。继续定位 027 原生进程启动到首个可见窗口与 MCP 同步之间的停滞。通过后仍需独立复审及合并。已请求独立复审代理授权。共享主检出的路线文档存在并行编辑，保留现场。

- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | checkpoint: T-05/T-06 | outcome: pass | next: review/T-07。027 Native 恢复验证使用干净应用根 `accessible-vue-root/examples/ui/027-file-manager` 与预热图形缓存的独立目录。一次启动预热诊断在第 4 秒找到真实窗口和实时树。重新启动本轮应用执行普通控件动作，重命名 9.707 秒、新建 9.662 秒，均 exit 0。读回证明源消失、目标内容摘要一致、冲突文件不受影响、新文件为空、取消无写入。MCP 与 API 端口关闭且本轮应用正常退出。未预热启动失败保留在前项，复验未断言缓存是唯一原因。

  两项报告分别为 `warm-027-file-rename/report.json` SHA256 `8d99eec95ba58dc300af00808b890729a179c34be1a2fcfa875a6948e83c7350` 与 `warm-027-file-create/report.json` SHA256 `3fb569156d47813c415072d3ac176cab4e3c5eadadd3ad2a77761be56249e9ea`。工具 `b50c023e9118a2943156402ea25a6ca5bd532160ff2c2ae1c6e6d135c81fd27a`，Rust API `5e00a19496d77fffb23c8832b32d934dbaad51c6ac167071de9a8db2aed94512`。测试采用同次编译源 `a9dc2b85a28c3f8448263209ba8be7465e589b5e`。待审分支尖端 `3de71f8429bc4f6cc49f85de3977124e78835564` 仅在其上更新路线文档。Windows Native VM 8/8、Windows Vue 8/8。

  | 用例 | 证据及验收 |
  | --- | --- |
  | P-01/P-02/P-06 | 两轨 names、pending，精确提交锁模型通过。校验拒绝、重试与重复取消覆盖 AC-02/03 |
  | P-03/X-04 | 模型移除重挂及同修订身份反例，原生 cancel 旧节点负控，T-01 实际配置错误探针覆盖 AC-02/03/04 |
  | P-04 | 两轨 027 隔离目录读回和写权限覆盖 AC-04/05 |
  | P-05/X-02/X-05/X-07 | 两轨 Tab、读回、IME 分层验证、原生实际系统树和动作、生命周期覆盖 AC-04/05/06 |
  | X-01/X-03/X-06 | 移根、缺包负控、双主题截图、MCP 缺失及启动失败清理覆盖 AC-01/04/06 |

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: work | to: review | status: execution_done | outcome: waiting | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `3de71f84` | next: T-07。Windows 产品验证已恢复。等待独立复审授权。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | waiting_round: 2 | from: review | to: review | status: execution_done | outcome: waiting | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `3de71f8429bc4f6cc49f85de3977124e78835564` | next: user。候审工作树干净，主检出仍保留并行文档编辑。T-07 等待用户明确授权启动独立复审代理，或由独立复审者提供绑定该提交的结论。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | waiting_round: 3 | from: review | to: review | status: execution_done | outcome: blocked | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `3de71f8429bc4f6cc49f85de3977124e78835564` | next: user。T-07 的独立复审尚缺明确的代理授权或针对候审提交的复审结论。候审提交和主分支尖端均与上一轮一致。

- 2026-11-20。stage: review | review_scope: final | reviewer: `/root/review094` 独立上下文 | plan_id: PLAN-094 | plan_revision: r3 | outcome: needs_fix | worktree: `D:/autostack/.wt/ui-094/auto-ui` | branch: `codex/ui-094-prompt-dialog` | reviewed_commit: `3de71f8429bc4f6cc49f85de3977124e78835564` | base_commit: `9bdbbcf859e5185335014425af5e74f5512e8c29` | next: work/T-02…06。用户已明确授权启动独立复审代理。AC-01/02/04/05/06 fail，AC-03 和 SD-01/02 partial。报告 `D:/autostack/plan-artifacts/plan-094/review-t07/review.md` SHA256 `a0052ea3f2da1f093a48fff5037dd2f64d492300c0172425b25949225fa97bde`，包含依赖和规范输入摘要。七项预编译 Rust 用例全部通过。真实文件新建反例 8.200 秒，MCP 输入 300 个 ASCII 字符后提示窗关闭，隔离目录无变化。

  | 发现 | 修复及复验关闭条件 |
  | --- | --- |
  | F-01 | 027 原生文件和目录创建失败后保留草稿，给出错误，目录零变化。保留原生真实正例和 Vue 只读对照。实际底层调用会丢弃 std::fs 错误。 |
  | F-02 | 两轨整个应用退出再启动，通过真实新会话控件读回，核验本轮子进程与旧端口释放。 |
  | F-03 | 原生缺声明的相对包时报具名错误并退出失败，移根正例及同名内置优先级有正反对照。 |
  | F-04 | 给出可复现的 027 图形缓存准备和严格墙时，保留已知冷启动失败。原生 rename/create 冷态 11.901/11.826 秒，热态原证据缺复开。 |
  | F-05 | 补当前配置错误至恢复、跨目标迟到反馈、同名双实例隔离、嵌套焦点与实际滚动 oracle。 |
  | F-06 | 禁用 Vite 门内隐式依赖构建或要求显式准备并检查证据，验证缺依赖具名失败。 |
  | F-07 | Native 共享布尔输入恢复 `on` 与既有真值，保留新绑定输入和 Enter 控制。 |

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `3de71f84` | next: T-02…06。保留独立通过的对照，按 F-01…07 修复并独立复验受影响范围。

- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | repair_round: 1 | outcome: pass | worktree: `D:/autostack/.wt/ui-094/auto-ui` | branch: `codex/ui-094-prompt-dialog` | code_commit: `8a51fcdf629981b2de6b61af23b7c3208dddf456` | git_tree: `1194bd5fa1d1dfc12a744e829bb916abd46e25d6` | task_ids: T-02…06 | next: review/T-07。依赖 pins 与首轮复审相同。原生工具 SHA256 `f6e263f370d90cd59ca540352c2aed5aad0daff66d0cba4a23464a61b0f7668d`。冻结规范增量 SHA256 `c658a862d19a2e77229fcbedfbf3920aaf079c4f7c060980c67e0a3ea64221cc`。证据根 `D:/autostack/plan-artifacts/plan-094/repair-r1/`。

  | 发现 | 修复与受影响范围复验（外层实际墙时，秒） |
  | --- | --- |
  | F-01 | 027 创建在实际目录/文件读回前保持未确认。独立反例的 300 ASCII 字符通过原生写文件拒绝 8.525、建目录拒绝 8.859 复验，保留草稿和重试能力，目录不变。成功创建 9.804、成功重命名 9.012、同名拒绝 9.166。Vue 普通重命名 7.171、冲突同名 6.761、创建 7.948 均保持只读。收据 `final-native-file-*`、`scoped-native-file-*`、`scoped-vue-file-*` 和 `single-vue-conflict`。 |
  | F-02 | 两轨独立 `reopen` 先正常退出并确认旧端点拒绝连接、子进程和原端口释放，再启动新 PID 与新窗口/页面并通过普通控件读回。Native 9.347，Vue 6.077。收据 `final-native-reopen`、`scoped-vue-reopen`。 |
  | F-03 | 原生启动前确认清单的本地相对路径存在，真实 CLI 缺包具名拒绝 `prompt_dialog`，exit 1，0.621。移根公开包真实解析/生成 1.049，通过后启动正例。已有内置遮蔽控制 0.399、解析优先级控制 0.145 均实际执行一项并通过。Vue 旧具名 TS2307 仍适用。 |
  | F-04 | 夹具显式隔离 `APPDATA` 与 `LOCALAPPDATA`，准备图形缓存。所有新产品用例的可接受样本以外层墙时核验。保留构建并行期间及未预热的超时，包含原组合 visual 10.068、rename 10.136、combined system 10.498、Vue 关闭慢样本 11.418。将浅/深、冲突/成功、两种 UIA 模式分别作为独立行为用例复验。必要生产断言保留。 |
  | F-05 | 同名实例、首次配置错误一次、修正恢复与跨目标旧回复的生产模型测试 0.096。实窗嵌套焦点与下层动作隔离 Native 7.765、Vue 4.459。长说明真实滚动和确认读回 Native 7.322、Vue 3.904。浅/深 Native 7.296/6.594、Vue 3.858/4.654。原生真实 UIA SetValue 表单读回 8.256，Invoke 应用读回 7.042。已核对四张暗色与长说明截图的文字、裁剪和主要动作。 |
  | F-06 | 日常 Vue 闭环仅运行 `vite preview`。开发构建与输出文件摘要独立准备，缺 bundle exit 2，0.173 秒。原先关闭 Vite dev 依赖发现的对照耗时 20.473 秒，保留失败证据，采用静态 preview 后解决。 |
  | F-07 | 恢复既有 `on`，布尔处理的真/假值、静态参补入、显式 checked、零参 Enter 的实际 VM 派发均通过，0.304。原锁定与旧绑定控制 0.090。初次测试误将 VM 内部整型表示要求为 Bool，已改为断言应用分支读回，保留原失败。 |

  作用域准备记录：针对工具、运行时、两个 Rust 测试目标的增量 debug 准备 96 秒，校正测试断言后增量准备 37.950 秒。修复验收场景及 Demo Vue 生成、类型检查和 preview 预构建约 12.538/10.149 秒，027 Vue 准备 42.431 秒。准备耗时与产品用例墙时分列。初始复审已通过且未受影响的 IME、传统 Dialog、原停留锁、页面应用读回证据继续由 §9 原收据覆盖。没有在本轮执行完整仓库回归。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: work | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `8a51fcdf` | next: T-07。请求同一独立复审者核对修复及受影响对照。

- 2026-11-20。stage: review | review_scope: final | reviewer: `/root/review094` 独立上下文 | plan_id: PLAN-094 | plan_revision: r3 | repair_round: 1 | outcome: needs_fix | reviewed_commit: `8a51fcdf629981b2de6b61af23b7c3208dddf456` | next: work/T-05/T-06。F-01/03/07 pass，F-02/F-05 partial，F-04/F-06 fail。实际逐项证据 `D:/autostack/plan-artifacts/plan-094/repair-r1/review-t07-r1.md` SHA256 `fc7215d7dc3c43f33a1313e9b2c0e6438448a3916c8b20b4c9b42b519611b94a`。本轮独立复审检查 25 份报告和 111 个附件摘要，无重复构建或产品执行。

  | 第二次未通过后的诊断 | 先前漏检原因与关闭条件 |
  | --- | --- |
  | F-02 应用状态和失败清理 | 重启用例把初始值当作新的业务读回，收尾异常跳过检查。新会话取消操作须出现用户可见的业务状态和零写入。故障回放必须执行所拥有进程核验和端口重绑。 |
  | F-04 Vue 新用例墙时 | 原版重复先执行 `browser_close` 再关闭 stdio。反例中两次实际旧端点拒绝均约 0.02 秒，慢样本 `browser_close` 为 4.188 秒。改为一次标准 stdio 正常退出并验本轮所有进程，在相同 Vue 目录/同名用例验证外层墙时。 |
  | F-05 Vue RPC 可复查性 | 为取结构化值有若干调用绕过统一记录，导致缺滚动和焦点原始请求/结果。所有关键调用从同一记录出口进入，复测两类实际 DOM 操作并查原始记录。 |
  | F-06 源绑定预览 | `vite preview` 虽不构建资产，默认会编译项目 TS 配置，且单查 index 文件无法识别陈旧/缺少其它资产。外部准备冻结项目输入和全部静态产物摘要，runner 只验收一致的完整包，并使用禁用 configFile 的预览。复验当前 Demo/027 后端、缺清单和陈旧/缺资产对照，以及运行进程树。 |

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `8a51fcdf` | next: T-05/T-06。按诊断只复验准备、运行、关闭链和受影响的产品用例。

- 2026-11-20。stage: work | plan_id: PLAN-094 | plan_revision: r3 | repair_round: 2 | outcome: pass | code_commit: `b71ff7914f800c26a2b0b06358bf10732f59e171` | git_tree: `7e301dc07813a8160bc1c17acfb2efe436567642` | next: review/T-07。实施与依赖继承前轮，变更限于测试准备、运行、清理和证据。runner SHA256 `2b5507105139f7166aca77de76078c9245b0ccc94d80d8e086bc5bdaa89da19e`，准备工具 SHA256 `3aed958b63cf12eb1a9f05903d8b3726245ca34c8e1b682f9f9eba223f167371`。规范增量仍为 `c658a862d19a2e77229fcbedfbf3920aaf079c4f7c060980c67e0a3ea64221cc`。

  | 发现 | 定向闭环（外层墙时，秒） |
  | --- | --- |
  | F-02 | Native 8.044、Vue 4.423 整应用重启，并在新会话真实打开分组、Escape 取消，读回 `已取消分组重命名`、名称和零保存，均 exit 0。真实不存在节点负控 exit 1、3.750，清理通过。额外在停止真实自有应用后注入关闭异常，仍核验四个 PID 与旧端口重绑，1.493，原关闭异常保留。 |
  | F-04 | 去掉重复的 browser_close RPC，实际 Playwright stdio EOF 请求观察到正常退出码 0。旧慢用例相同操作复验两次外层 7.110/6.463，正常浏览器退出分段 0.200/0.146，均验证旧端点拒绝和 PID 回收。保留旧 11.418 秒反例。 |
  | F-05 | Vue 真实上层焦点 4.160、实际滚动 3.344、暗色外观 4.033、标准 IME 3.793 均通过，`browser_run_code_unsafe` 请求和返回写入各自 `rpc.ndjson`，焦点和滚动仍读回实际动作结果。 |
  | F-06 | Demo、嵌套、027 准备外层 10.585/3.294/3.486 秒，独立记录。运行 preview 禁用 configFile，无 esbuild 子进程，实际 027 文件拒绝及新建只读负控 7.110/6.453 秒。缺收据、源码变更、缺资产均在启动前具名拒绝，exit 2，0.151/0.148/0.139。 |

  证据根 `D:/autostack/plan-artifacts/plan-094/repair-r2/`，含 `vue-timings.txt`、`negative-controls.log`、`replay-preparation-and-cleanup.py` 及每用例原始报告和附件摘要。Python 精确源 py_compile 和 git diff --check 通过。F-01/03/07 及其余未受影响的 Rust、系统 UIA 和原生产品证据沿用前两轮复审，未重复整仓测试。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: work | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `b71ff791` | next: T-07。提交第二轮定向修复供独立复验。

- 2026-11-20。stage: review | review_scope: final | reviewer: `/root/review094` 独立上下文 | plan_id: PLAN-094 | plan_revision: r3 | repair_round: 2 | outcome: pass | reviewed_commit: `b71ff7914f800c26a2b0b06358bf10732f59e171` | git_tree: `7e301dc07813a8160bc1c17acfb2efe436567642` | base_commit: `9bdbbcf859e5185335014425af5e74f5512e8c29` | next: merge/T-08。AC-01…06、SD-01/02 全通过，AC-07/T-07 关闭。依赖 auto-lang `68fd4c59222c77a5de0963ff5827f1353abd7b4f`、auto-web `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`、auto-down `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`、auto-man `4a30483a590d81d5724cb453a0fd1fbc98d9aaf2`。冻结规范提案 SHA256 `c658a862d19a2e77229fcbedfbf3920aaf079c4f7c060980c67e0a3ea64221cc`，目标 `docs/specs/ui/prompt-dialog.md`。独立最终复审证据 `D:/autostack/plan-artifacts/plan-094/repair-r2/review-t07-r2.md` SHA256 `a2ec69936d8a3dfba4309c0f0ec4ef1d591877d2c42bc1a6ef650740ccb11dbc`。核对 13 个报告和 50 项附件摘要。独立复验只执行预览完整包、修改一个资产字节拒绝及恢复三态和四份 RPC 原始结果解析，0.039 秒，无启动、构建或资源等待。Windows 产品准备和测量分段、旧失败以及本轮通过的关键时间保留在前两项收据。待合并阶段验证规范、落主、归档和清理。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `b71ff791` | next: T-08。独立最终复审通过，待落主。

- 2026-11-20。stage: merge | merge_key: PLAN-094:r3 | checkpoint: prepared | outcome: blocked | reviewed_commit: `b71ff7914f800c26a2b0b06358bf10732f59e171` | delivery_commit: `a5e12d3b2cdfe1b6371f567cef80acc617f58686` | next: user。正式规范 `docs/specs/ui/prompt-dialog.md` 已在实施树准备并提交，SHA256 `def5c2bb72ad8979e9c5e64a4dee8fdeeacf9ac1e5c6387816312bbd19200b95`。规范正文逐行等于冻结的 SD-01/02。差额只涉及规范与路线文档。内容与链接检查、`git diff --cached --check`、`wt-guard.ps1` 均通过。对 `master` rebase 时分支已最新，提交未重写。主分支仍为 `9bdbbcf859e5185335014425af5e74f5512e8c29`。在主检出执行 `git read-tree -n -m -u HEAD a5e12d3b` 无写入预检，exit 128，因 `docs/design/autoui/component-library-extension-strategy.md` 有其他未提交编辑而拒绝。其他计划草案与已存在的主检出工作保持原位。规范发布、归档和清理尚待此项解决。该合并准备只检查文档和 Git，未新增产品或构建测试。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: merge | to: merge | status: reviewed | outcome: waiting | worktree: `D:/autostack/.wt/ui-094/auto-ui` | commit: `a5e12d3b` | next: user。等待用户授权将其他未提交草案保存到命名本地 Git stash，或由工作区所有者先整理主检出。候审及交付提交保留。

- 2026-11-20。stage: merge | merge_key: PLAN-094:r3 | checkpoint: landed | outcome: pass | delivery_commit: `a5e12d3b2cdfe1b6371f567cef80acc617f58686`。按用户授权把本计划以外 23 条已修改或未跟踪路径保存在具名本地 Git stash `e56eeac0ffaf5ee07492fa463da141adf2a77195`，逐项核对 Git 对象和路径，保留该 stash。主检出从 `9bdbbcf859e5185335014425af5e74f5512e8c29` 快进，独立复审提交 `b71ff7914f800c26a2b0b06358bf10732f59e171` 完整保留，差额仅为路线文档及规范沉淀。

- 2026-11-20。stage: merge | merge_key: PLAN-094:r3 | checkpoint: canonical_verified | outcome: pass。核验 [PromptDialog 规范](../../specs/ui/prompt-dialog.md) SHA256 `def5c2bb72ad8979e9c5e64a4dee8fdeeacf9ac1e5c6387816312bbd19200b95`，正文与复审提案一致。未写知识索引或旧台账。合并后从主检出复制示例和包到隔离目录，使用复审的同一组调试工具执行 `verify_prompt_dialog.py --backend native --case reopen --startup-timeout 5`，exit 0，进程墙时 8.530 s，报告内墙时 8.271 s。首轮/重开启动分别 1.340/1.079 s，重开后输入和取消及应用读回 0.364 s，首轮/重开清理 2.658/2.593 s。报告 SHA256 `12cd9121c1225e2b99cf3228c091489092d0da2da018ff9c7ffa65a3d60b3e3e`。本阶段无新增测试用例或构建，Vue 和其他 Native 结果沿用绑定交付代码的最终复审。

- 2026-11-20。stage: merge | merge_key: PLAN-094:r3 | checkpoint: archived | outcome: pass | archive: `docs/plans/archived/094-prompt-dialog.md`。实际归档路径与状态一致，T-01…08 和 AC-01…08 编号各 8 条，已核验每表 7 条完成且仅清理待办。主检出中 5 处计划导航及历史引用解析到本归档。归档中 9 处相对链接和正式规范中 2 处相对链接均解析成功。规范 SHA256 与交付提交一致。

- 2026-11-20。stage: merge | merge_key: PLAN-094:r3 | checkpoint: cleaned | outcome: pass。归档提交 `15f88456c54b7b0315557621404545a953beaa6a`。AutoUI 实施树及 auto-lang、auto-web、auto-down、auto-man 四个固定依赖树均为干净状态，逐树执行路径核对、`wt-guard.ps1` 与普通 `git worktree remove`，确认全部解除登记，已合入分支及空组目录移除。五次 Git 移除的进程墙时合计约 10.23 s。实施、复审和冒烟证据已折叠入本节。临时目录 `D:/autostack/plan-artifacts/plan-094/` 清理前有 135228 个文件、3956 个重解析点、3956277151 字节去重文件数据。清理逐项核对路径句柄，不跟随重解析点，核验目录和父目录中的条目均已消失，登记 `scopes.plan-094` 于 22:52:02 +08:00 关闭。清理脚本在目录移除后遇到登记行缩进不符，随后单独修复条目，删除精确计时未留存。无产物保留事项。23 条其他改动仍存于具名本地 Git stash。

- 2026-11-20。交付后检查了生产产物。主检出的 `target/release/auto-ui-tool.exe`、AutoMan 的 `target/release/auto.exe` 和名称管理例的 `dist` 均缺失。027 的 `dist` 目录最后修改于 2026-11-18 22:59:21 +08:00，早于交付，内无可部署文件。发布构建和重启沿用用户本次范围，均未运行。AutoLang 批量回归到期检查不适用，本次交付只涉及 AutoUI 仓。

- 2026-11-20。stage: goal | plan_id: PLAN-094 | plan_revision: r3 | from: merge | to: done | status: archived | outcome: pass | commit: `a5e12d3b2cdfe1b6371f567cef80acc617f58686`。T-08/AC-08 完成，Windows Native VM 与 Vue 均经独立复审通过，计划 8/8。

## 10. 待澄清事项

无必须向用户补问的产品决策；按§5边界执行。包语法/实例token/首次error/模态焦点/IME/反馈时序及现有Vue MCP提供方由T-01具名探针落实；T-02前关闭机制选择，不能把协议自由设计留给实现中途，失败须记录而非假定已支持。Windows 构建工具及浏览器 MCP 由 T-01 实查。Windows 必需能力缺失时，对应验收保留 not_run。业务能力不在宿主上提供时明确不可用，按上述普通消费者和真实原生消费者各自验收。

独立复审通过。Git 快进预检被主检出的既有路线文档编辑拦截。用户可授权将其他未提交草案完整保存到一个命名本地 Git stash，或先整理主检出后再继续合并。
