---
plan_id: PLAN-096
status: executing
feature_name: CommandPalette 通用命令面板
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-23
plan_revision: 3
current_step: 1
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/command-palette.md]
touched_goals: []
---

# PLAN-096 CommandPalette 通用命令面板

## 0. 变更摘要

补齐组件库路线 CL-06 的实施合同；提供命令搜索、分组、稳定选中、禁用状态、上下选择/Enter执行/Escape关闭、快捷键提示。应用提供命令并持有执行权限、全局快捷键、历史记录和业务动作。见[持续跟踪 strategy](../design/autoui/component-library-extension-strategy.md)。r3 按用户确认将交付平台限定为 Windows Native VM 与 Windows Vue。

## 1. 目标

Windows Native VM 与 Windows Vue 使用相同公共接口、状态语义和普通控件。用户键盘/鼠标与 MCP 驱动同一条事件链，自动测试不要求系统文件窗口或操作系统前台焦点。提供正式可移根组件包、普通消费示例、现有消费者迁移及无人值守证据。

非目标：全局命令总线、应用启动服务、任意脚本执行、全局快捷键注册器、替换 Launcher 的网格/分类/最近使用业务、统一所有 builtin Command 子元素实现。Linux 和 macOS 在当前交付范围之外。新增平台支持时需在对应系统完成构建、真实窗口、字体、MCP 和生命周期验收。

## 2. 架构方案

调用方命令记录 → local过滤或provided结果 → 以commandId持有活动项 → invokeRequested → 调用方执行并反馈/关闭。标准交互由正式包持有；Launcher 自己的排序和 recent 服务在 provided 模式保留。

新增正式 `packages/command-palette/`，公共名为 `CommandPalette`。复用标准 Dialog/Input/Button/List/Progress 和既有主题/包机制，仅按本组件需要取用。编译器注册优先级 Builtin > Local > Package；禁止靠同名包覆盖已登记控件。模型/协议尽量单源 `.at`；必要共享运行时接缝须最小化并验证 native/Vue 等价，不造两份独立算法或 compiler→runtime 依赖。

应用持有业务数据、权限、I/O、持久化及执行效果；组件持有焦点、草稿、选中和防重复请求状态。普通重绘不换实例身份；context/会话真正变化才使旧动作失效。公共可见操作必须能按语义名称发现，不能只靠坐标或数组下标。

## 3. 技术栈

沿用 `.at` 正式包与普通消息、Rust compiler/runtime/tools、native VM、Vue及现有主题。测试复用 `crates/auto-ui-tools/tests/` 包导入/移根模式和 `tests/product/` 的进程、端口、MCP证据做法；Vue 使用真实浏览器操作。仅因应用需要服务时启动隔离的已有服务，不建设新的测试平台。

## 4. 需求分析与背景调查

2026-11-23 用户通过 `/goal 完成计划096` 授权本仓实施、独立评审、合同内修复、合并、归档与清理。同日明确选择“调整为仅 Windows”，r3 将验收限定为 Windows Native VM 与 Windows Vue，保留其余产品、MCP、视觉与生命周期要求，并在恢复后明确授权以一个独立子代理只读复审。AutoOS 发行版 PLAN-001 尚待硬盘安装与重启验收，`vmrun list` 显示 0 台运行中的虚拟机。本计划沿用 `D:/autostack/.wt/ui-096/auto-ui`，分支 `codex/ui-096-command-palette`，基线 `4dd9696da6a6289795ba9d6f57e603c24d99114a`。主检出另有计划 118 和计划导航的文档改动。

授权来自2026-11-14用户请求：为四组件分别补齐实施计划，复用既有实现、引用已生成参考图、写明跨平台/MCP/无人值守验收并更新 strategy。本轮动作限本仓计划、设计导航及引用；不实施组件、不启动实施树、不提交其他WIP、不修改兄弟仓。用户未指定模型、预算、并行代理或自动实施；这里不推定授权。

2026-11-14本次复核HEAD `3cab09943079a1d1fca993fa5a4c4dd2a9fa8315`；084交付 `7ea71f0c`，086交付 `765251b9`。本次用户授权再次审查/补齐计划并推荐下一项，仍无实施、建树、跨仓写入授权。补读当前FileDialog/Pagination Spec及Windows系统语义合同；下列原始源文件摘要复核未变。

初稿观察基线 auto-ui HEAD `a6f776c24a4ba98611849955edceb5a04edac969`；未提交来源采用下表摘要/路径，不当作已交付版本。已读 [Spec总览](../specs/README.md)、`ui/{examples-ownership,theme-scope-and-publication,structure-closure}.md`、[原生验证合同](../specs/runtime/native-vm-app-validation.md)、`build/ui-test-selection.md`。当前无本组件专门Spec，拟在交付时新增；当前Spec不能因草案而改成已支持。

| 已核来源 | 复用边界与现存差额 |
|---|---|
| `examples/ui/028-launcher/src/front/app.at` | SHA256 4EAEC20D6F87C52B53D37D6AB7CF2DD82E7BFC087B67C52F5F656F5565D0206E；ApplyFilter/MoveUp/MoveDown/Pick/Launch 与 q/ranked/sel/nres 已存在。Launch 写 __desktop_cmd，业务调用保留应用。 |
| `crates/auto-ui-compiler/schema/aura.at、crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | Command/command_input/item/list/group 已声明且 Builtin 优先；新名 CommandPalette 不遮蔽旧名。声明/浏览器映射不证明原生已具备完整键盘流。 |
| `packages/file-dialog/、现有 Input/List/Dialog` | 使用正式包方式和稳定可见操作；必要时组合基础控件，不给两端叠两套搜索/Enter处理。 |
| `packages/file-dialog/`、`crates/auto-ui-tools/tests/file_dialog_component.rs` | 正式包导入、依赖闭包、移根和实际控件测试可复用；不复制文件业务/专用就绪协议。 |

实施 T-01 必须重新读取上述来源并记录实际 commit/hash、工具版本和脏文件边界。Reader 路径是当前主检出的只读参考，不依赖已消失的 `.wt/reader-009/`；Reader产品迁移、vendor同步和发布另属其自身计划。四项094–097相互无硬依赖，也不等待087；不为复用抽象平白新建第五个前置项目。

## 5. 详细设计

### 公共接口与所有权

| 接口 | r3合同 |
|---|---|
| props | open:bool=false；revision:int=0；title/placeholder:str；commands:List=[]；filterMode:str=local；query:str=""；wrapNavigation:bool=false；resultsSession/resultsQuerySeq:int=0；resultsQuery:str=""；response:str=""；errorMessage:str="" |
| 命令记录 | {id,label,groupId,groupLabel,keywords:List[str],shortcut,disabled,icon}；id 唯一稳定，label 可重复；groupId为空表示无分组；同组非空groupLabel必须一致。local分组按输入首次出现顺序、组内应用下述排序；provided严格保留调用方全局顺序，以相邻相同groupId生成分组标题（同组可分段），不得重新聚合改变原评分顺序。shortcut 只是文字提示，icon 使用现有图标体系且保持可访问 label。 |
| local 搜索 | 查询裁剪首尾空格/Tab/CR/LF，ASCII A-Z 转小写，其他 Unicode 原文匹配；在 label/id/keywords 取子串匹配。空查询保留输入顺序；非空按 label 完全匹配、label 前缀、其余匹配分层，层内稳定输入序，组按首次出现组织（排序只在组内，不把组标题当命令）。不引入locale依赖模糊匹配。 |
| provided 搜索 | 组件在打开和每次查询变化时发 queryChanged，调用方提供已经过滤/排序的 commands 和递增 revision；组件不二次过滤或排名。query 普通输入即时显示，parent 回传相同 query 不重置输入；新查询后旧结果暂时不可执行，只有 resultsSession/resultsQuerySeq/resultsQuery 全部对应当前请求才启用。仅比较文字不足以防止 A→B→A 的迟到结果。 |
| 公开消息 | queryChanged(str)={session,querySeq,query}；invokeRequested(str)={seq,session,commandId,baseRevision,querySeq,query}；closeRequested(str)={session,reason:escape/cancel}；error(str)。seq 实例递增、session 每次打开递增；querySeq 每次发出查询递增，包括打开时的初次查询。 |
| 异步确认 | response={seq,session,commandId,status:accepted/rejected,message}；本地锁定一次调用，accepted 等父层关闭，rejected 留窗可重试。command/revision 或 session 改变使旧未执行请求/反馈失效；应用执行前重新核对命令存在/权限。 |

事件载荷示例（JSON字符串，字段名固定；不是新MCP接口）：

```json
{"instanceToken":"app-1/mount-17","seq":4,"session":2,"commandId":"theme.dark","baseRevision":9,"querySeq":3,"query":"theme"}
```

`.at` 公共消息沿既有包的可导出命名方式声明（例如 `"changed"(str)`），由普通 `onchanged`/对应消息绑定消费。上表类型是接口语义，T-01只核对当前语法/封装方法，不把接口再交给执行者自由重设计。普通 props 数据通过组件正常输入提供，不从测试环境变量注入成功结果。

### 实例身份、反馈与复用接缝

每个实际挂载实例分配不复用的 `instanceToken:str`，普通重绘保留，卸载重建/另一个实例必须不同；其来源绑定App及真实组件owner/generation，不使用label、列表索引或每次渲染时间戳。所有请求、取消/关闭消息和异步response均增加此字段，调用方原样回传。T-01证明native/Vue中的可用分配机制；优先复用现有实例生命周期，只在确有缺口时补最小共享接缝。重建后seq/session即使重新从1开始，旧token也不能命中新实例。

同revision出现影响动作含义的数据变化也必须失效旧绑定，不能依赖调用方永不漏增revision。迟到消息先核对instanceToken、当前目标及活动请求，再处理业务状态；排队的用户动作还须在消费时检查可见、启用、当前modal与数据身份。两个同名实例、隐藏重开、卸载重建均配当前有效操作正例，禁止把所有动作都拒绝作为“隔离通过”。

复用 `crates/auto-ui-runtime/src/ui/modal.rs` 与 `ui/iced/renderer/host.rs` 的modal收窄、`ui.focus`/焦点路径，不另写全局键盘服务。配置错误首次挂载和相关配置变化必须可见；相同错误不每帧刷回调。保留 `error(str)` 约定，T-01验证首次配置错误恰好一次通知的正常调度路径：已交付分页Spec说明native子组件Init内emit不派发，不能只在Init中发事件。若需最小接缝修复，归本Plan T-01/03并验证兄弟组件，不默默删除事件合同。

### 状态、交互与消费适配

- local匹配时查询与label/id/keywords均使用同一ASCII折叠规则，非ASCII原文保留；组标题默认不作为隐含搜索词。provided保持原ranked全局次序，包括不同groupId交错的情况；分组是显示，不得覆盖业务排序。resultsQuery以原始输入比较、querySeq区分相同文本的多轮查询；错序revision/旧session结果不能把revision推进后再污染新数据。
- query初次打开取props，parent回传相同值不重置编辑；真正外部query改变按普通新查询处理。local也发queryChanged，供调用方观察，但不等provided结果。全禁用时active为空；命令id缺失/重复、groupLabel冲突为配置错误，未知可选icon使用已有通用图标或无图标降级且label保留，不直接假定动态icon name可生成。
- invoke pending期间搜索、选择、Enter/点击及用户Escape/取消都禁用，避免已发请求被普通查询变更解锁；父层可主动关闭。accepted响应与应用关闭/结果同一快照，或保持成功待关闭锁；reject留窗恢复输入。非pending关闭先锁并只发一次closeRequested。外部命令数据/权限真变化中止旧请求身份；业务已经开始执行不能靠UI关闭撤销。

- 打开聚焦搜索；Up/Down 在当前可执行结果中移动，默认到首尾停止，wrapNavigation=true 时首尾环绕；禁用项和分组标题不参与。Enter 仅执行当前有效命令，空结果/全部禁用零事件；IME composing 的 Enter 不执行。
- 选中身份是 commandId，过滤/重排后仍存在则保留，否则选择第一个可执行项；同label不同id能分别操作。provided查询改变后在父层提供新结果前禁用旧结果；session/querySeq/query任一不匹配均不可执行，local不等待外部结果。
- Escape/取消发关闭请求；背景点击不执行任何命令。Tab 限于弹层，关闭恢复触发器焦点；非 pending 时关闭可用。invoke pending 禁止重复 Enter/点击，允许父层关闭使旧会话失效，不承诺撤回已由应用执行的动作。
- 028 palette 模式接入 CommandPalette provided 模式，设置 wrapNavigation=true 保留现有首尾环绕，保留 ApplyFilter 的 recent/分类/原有评分，把 ranked 映射为稳定命令 id 的记录，并为每次返回标记 resultsSession/resultsQuerySeq/resultsQuery。原 palette 的上下/Enter处理停用或受焦点模式严格保护，避免一次按键触发两次；grid 模式不改。
- 调用方收到命令ID后复用原 Launch/__desktop_cmd 逻辑；组件不直接访问宿主启动总线。Launcher 的 host 不在普通浏览器时显示实际不可用，另有 examples/command-palette/ 普通设置消费者在两端真实修改主题选择/设置模型，作为独立业务成功证据。
- local 与 provided 是显式单一过滤 owner；优先复用现有 Command 能力，若其原生键盘或过滤与合同冲突，T-01记录用基础控件组合的必要边界，不新增编译器依赖，不宣称本计划补齐整个 Command family。

具名配置错误采用 `error(str)`，至少含 code/message；组件应显示错误并阻止相关修改，不用空列表或静默截断掩盖非法数据。业务拒绝与配置错误分开呈现。公开规模不承诺无限性能；不得未经记录默加“只支持前N项”的截断。新增资源上限或不兼容约束需先修订合同，不能只改测试。

### 视觉引用

固定普通消费命令：Appearance组theme.dark/“Dark theme”、theme.light/“Light theme”，Settings组theme.settings/“Theme settings”，另有files.open/“Open file”。查询theme只得前三项；theme.dark执行后真正更新页面受控theme=dark，重开读回，files.open不进入该查询结果。补两个相同label但不同ID与一个disabled项，验证ID/禁用；provided用A→B→A加不同querySeq模拟正常异步结果迟到，不能直调执行回调。Launcher迁移需对空查询recent、分类和既有评分准备前后相同输入的结果序列对照，首尾环绕也必须保留。数据与执行入口在T-04/05新增，当前未运行。

直接复用[已生成的四组件参考图](../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1.png)及[文件导航/轻量交互视觉指南](../design/autoui/file-navigation-and-interaction-visual-guide.md)，只实现**左下：theme 搜索、匹配结果分组、选中行、快捷键与键盘提示**。[生成提示](../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1-prompt.md)和最终PNG均为仓库设计资产，不依赖attachments或个人缓存。PNG SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`，1586×992 RGB。

样式沿[公共执行/视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)：32px控件、8px间距、16px面板padding、16px outline图标、约6px圆角，语义主题色和focus ring。图不是界面纹理或尺寸像素金标准；中文/长标签/错误、浅深色及420×560小窗以真实GUI可读可操作为准。图中文字不覆盖上表状态/异步合同。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/command-palette.md | 无正式专门合同→上述受控参数、消息、校验和身份/异步状态规则 | 收拢应用重复交互，保留业务owner | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/command-palette.md | 无本组件发布/验收约定→包导入/移根、普通消费者、跨端MCP/视觉/无人值守门及平台边界 | 可复用、可操作、可验证 | AC-04,AC-05,AC-06 |

实施树 `D:/autostack/.wt/ui-096/auto-ui`，分支 `codex/ui-096-command-palette`。实现、自检、独立最终复审依次使用这一棵树；Plan进度留主检出，不在另一棵树同步实现。

执行入口是 T-01：来源已核、接口和AC已定，当前无外部组件交付阻塞。先走完整最小链路：**正式导入→local搜索/箭头/Enter→设置模型读回；provided输入→ApplyFilter结果revision/resultsQuery→Launch入口；同时验证现有Command默认过滤和键盘是否需要关闭以免双发。**。产物 `docs/reviews/plan096/decisions.md`（新增）记录可行路径、实际语法/命令、版本、待修接缝和探针证据；不得只看schema或源码文本。探针失败先在原目标内修接缝；若必须改平台/API/业务owner或AC阈值，则修订Plan并保留原要求，不擅自降级为截图/静态通过。

## 6. 测试设计

### 区分正确与错误实现的必需用例

下表每行 owner_stage=work、执行任务=T-05；入口是新增 `tests/product/verify_command_palette.py` 的具名case，以及新增 `crates/auto-ui-tools/tests/command_palette_component.rs` 中的模型/包边界反例。应用效果必须由正常页面/应用持久读回确认，不能通过直接调组件回调来制造成功。

| case_id | acceptance IDs | 代表输入/状态 | 可观察oracle / 失败对照 |
|---|---|---|---|
| C-01 | AC-01,AC-02 | theme、空查询、中文、空结果、重复label/唯一ID | 结果按规则，所有可执行行匹配；组标题不可执行，同名命令事件ID准确；无结果Enter零调用。 |
| C-02 | AC-02,AC-03 | Up/Down首尾、禁用项、IME、连续Enter | 默认边界停住，028显式环绕，均跳过禁用；composition不执行；一次pending只有一条业务调用，无旧palette双重处理。 |
| C-03 | AC-03,AC-04 | provided查询A→B→A，迟到第一次A结果，重排/命令移除 | 文字相同但querySeq不同也不可执行；按ID恢复选择，移除后重选合法项；旧MCP节点和反馈不能调用另一命令。 |
| C-04 | AC-04,AC-05 | 028实际宿主调用与普通设置消费者 | 宿主可用时原Launch链一次命令；无宿主明确不可用；普通设置模型真实变更并读回，不能只看选中行。 |
| C-05 | AC-02,AC-04 | 两个面板、关闭重开、重复ID配置 | 会话隔离、焦点恢复；重复ID具名错误且不可执行，不按数组下标临时造身份。 |
| C-06 | AC-02,AC-03,AC-05 | provided跨组交错排名、pending输入与关闭、成功反馈同帧 | 交错顺序与原ApplyFilter一致，分组不重排；pending用户不能另发查询/调用/关闭，accepted不被revision更新重新解锁；应用主动关闭后旧反馈无效。 |
| X-01 | AC-01,AC-06 | 正式包从另一目录导入；缺依赖、同名内置并存 | 无绝对开发机路径，实际导入生成并运行；缺依赖具名失败，内置旧示例仍能解析运行，不以搜索字符串通过。 |
| X-02 | AC-04,AC-06 | Windows Native VM 与 Vue，当前树动作及关闭重开 | 真实窗口/非空树、实际控件动作与应用读回相符；错误对照返回明确失败，退出后本轮进程/端口回收，再次启动可用。 |
| X-03 | AC-06 | 浅/深主题、中文长标签、420×560、错误/禁用/pending | 同版本真实截图可读、正文可滚动、主要操作可达，无glyph缺失、重叠或误点击；空/加载态按组件适用状态覆盖。 |
| X-04 | AC-02,AC-03 | 卸载重建后seq重复、同revision数据变更；首次非法配置及修正 | 新instanceToken拒旧反馈，旧动作零误写且当前正例可用；配置错误可见且每次配置变化恰好一次error，修正后正常恢复。 |
| X-05 | AC-02,AC-04 | 多输入/双实例/嵌套modal；组合输入适用时 | MCP发现并操作当前目标，不把首个input代理误当真实焦点；底层按钮/快捷键被阻挡，Tab/关闭恢复正确；输入法按下文分层证据验证。 |
| X-06 | AC-04,AC-06 | 浏览器GUI通过但MCP不可用、启动/动作中途失败 | GUI不能替代MCP记录；缺必需能力整体not_run/非零退出；失败也执行finally清理、保存最后树与日志，原有进程不受影响。 |
| X-07 | AC-04,AC-06 | Windows原生真实系统树的输入/按钮 | 按native-vm-app-validation及native-system-accessibility核对受影响标准控件的role/label/value/focus/bounds/action；不以MCP树冒充系统provider或物理输入证据。 |

共同不变量：一次合法手势至多一条当前身份业务请求；非法/禁用/旧动作零目标变更；业务拒绝不伪造成功；接受后的真实消费状态与事件一致。组件树状态断言与消费者读回必须相互独立。测试通过“输入正确但状态owner写错/重复执行/旧对象覆盖”的反例来区分 plausible 错误实现，不能只计绿测试数量。

### 运行入口与证据合同

以下入口用于 T-05。程序通过隔离的准备目录运行，默认执行相应后端整套用例。`--suite-root` 指定隔离的三项目根（缺省为输出目录父目录下的 `windows`）。单案可用 `--case` 和 `--app-dir`。实际运行证据见 §9。

```text
cargo test -p auto-ui-tools --test command_palette_component
python tests/product/verify_command_palette.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对MCP工具路径> --out-dir <本轮独立证据目录> --backend native
python tests/product/verify_command_palette.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对MCP工具路径> --out-dir <另一独立证据目录> --backend vue --browser-mcp-config <绝对浏览器MCP配置路径>
```

native采用 `auto run -r vm`，保留应用声明的真实后端（不能把`--server vm`当原生前端）。Vue及MCP入口按下文分层合同验证。只跑本组件、被修改公共接缝和迁移消费者的必要回归，保留既有oracle，不叠加无关整仓测试。

无人值守runner必须：每轮隔离数据/目录/端口；native校验自身拥有的OS窗口和非空live MCP树，Vue核验本轮页面/浏览器会话和实际MCP提供方；由当前snapshot发现真实节点再find/action/type/键盘/scroll/screenshot；动作后正常事件→应用读回；包含一项明确失败负对照；正常退出、核验本轮拥有的子进程和端口已释放并重开。原生动作不能依赖OS前台焦点，不能使用系统dialog、`AUTO_OPEN_PATH`、直接改组件状态、调用accepted处理器或回放旧node ID。超时须有限且保存最后树/日志，不能无限重试掩盖失败。

Windows Native VM 和 Windows Vue 两轨分别记录 pass/fail/not_run；两轨均为必需门。浏览器无宿主/磁盘权限时保留不可用结果，不放宽安全边界。

收据 `docs/reviews/plan096/` 持有实际版本/依赖闭包摘要、命令/退出码、case→AC矩阵、截图/树、业务事件及读回结果、负控、生命周期与必要回归。产物大日志放独立证据目录，文档引用可复现位置和摘要；不要把个人缓存当唯一凭据。Q-01文字glyph缺失不能因MCP文本完整而结案；若公共渲染器超范围，建立具名修复依赖，受影响AC-06保持未关闭。

本轮只做草案静态自检，无产品验收结果。用户未分配实现/内部检查/最终评审模型；未来执行者先自检，最终评审须独立上下文并绑定实现revision，同一实施树顺序复用；不据此自行启动其他agent/chat。

### 可执行门与证据分层

- 每个平台均拆为普通GUI交互、MCP工具调用、应用读回、适用视觉/输入桥、退出重开；不要求把全部边界case在GUI和MCP重复笛卡尔积。§6每个case记录实际测试层；模型测试可直接喂模型，产品路径必须通过普通控件，不能用模型测试冒充真实消费。
- native复用 `crates/auto-ui-runtime/src/ui/mcp_server.rs` 的 `autoui_snapshot/action/type/keyboard/screenshot` 正常工具；保留RPC请求/结果和当前节点发现。Vue复用已存在的浏览器MCP提供方（或已交付生产桥），从真实DOM/可访问树进行输入/点击/键盘，记录提供方、工具版本、调用/结果和应用读回；Playwright只操作同一真实页面，不能单独作为“MCP通过”。本Plan不新建Vue MCP服务器；T-01定位当前配置，缺提供方则Vue MCP门not_run，不由四组件分别造桥。
- product脚本保留原拟议CLI，Vue增 `--browser-mcp-config <绝对配置路径>`（T-05待实现，配置描述实际提供方/会话接入；凭据不进报告）；两种driver共用case断言。native的OS窗口归属以owned PID/handle证明；Vue验证本轮生成页面URL、浏览器会话和服务PID，不套用native HWND证明。两条腿均有真实MCP操作与独立业务读回。
- Prompt/Command/Tag的IME门：普通 `autoui_type` 是整串替换，不证明composition、物理按键或OS输入法。Vue输入桥用compositionstart/update/end与isComposing事件序列；native使用当前标准输入适配层的preedit/commit/取消→Enter事件集成测试，观测生产事件抑制与一次业务提交，并补普通GUI/MCP中文最终值读回。已有 `autoui_desktop` 的acceptance专用handler/key通道不代替普通组件MCP产品门。若适配层无法表达，T-01定位最小输入接缝后补测试；TaskProgress无文本编辑，IME标明不适用，仍测正常键盘/modal。
- 复用当前Spec及已有包而非只复用FileDialog：正式包必须提供 `package.at/pac.at/组件.at/README.md`，native源相对import和Vue既有components/vendor路径解析分别运行；从第二绝对根验证完整依赖闭包和无开发机路径。四Plan不得分别重写同一公共runner/输入桥；第一个发现公共缺口者在decision登记owner/提交，后续重基线复用。
- 每个实际运行门输出 `report.json`：plan/revision/platform/backend、source/tools/dependency摘要、runner和MCP提供方、case/layer/期望/实测/status、截图/树/读回路径及摘要、cleanup/reopen、result。退出码0仅当本次门全部必需case通过且清理重开成功；失败为1，缺必需环境/能力not_run为2。两轨须全部通过方可汇总为 pass。有效控制与关闭端口的同一MCP方法配对，关闭目标要求有界connection-refused，不能仅HTTP探测代替。
- 默认启动上限300秒、单动作15秒、退出清理30秒；runner可显式配置并记入收据，不能无限等待/重试。失败走finally清理本轮owned进程/端口/隔离数据并保存最后证据；只对确认身份的自有残留强制终止且写明原因。关闭/重开Dialog与退出/重启整个App分别验证；不把关闭UI误认为业务任务取消。

## 7. 验收标准

| ID | owner_stage | 可观察结果及验证方法 | 任务 / 状态 |
|---|---|---|---|
| AC-01 | work | 正式包公共接口按§5导入/生成/运行；X-01正反例、最小普通消费者及包依赖闭包可复现，无内置同名覆盖 | T-01,T-02,T-05；[x] |
| AC-02 | work | §5正常/错误/禁用/键盘/适用IME规则、本组件case及X-04/05全部有期望断言；无一次手势多次提交或虚假成功 | T-02,T-03,T-05；[x] |
| AC-03 | work | 当前身份、重复请求锁、外部改动/迟到反馈/关闭重开/双实例隔离反例通过，旧对象零误写（含卸载重建与同revision内容变化） | T-02,T-03,T-05；[x] |
| AC-04 | work | 普通GUI和MCP通过当前可见控件完成关键用例；Windows Native VM 和 Windows Vue 两轨分别有结果，原生不依赖前台焦点；X-02/05/06/07分层门、生命周期与负控通过，缺必需MCP/平台门不算通过 | T-03,T-05；[x] |
| AC-05 | work | 028保留recent/分类/评分/grid并正确转发一次宿主动作；无宿主明确报不可用；普通设置示例在两端读回真实模型修改，源码匹配或高亮不算执行成功。 | T-04,T-05；[x] Windows 宿主单次启动、双轨独立消费及排序对照见 §9 |
| AC-06 | work | X-01/03/06/07、移根、必要公共/消费者回归及真实截图通过；中文/暗色/窄窗可用，Q-01若影响本组件则闭环；README/待沉淀Spec与证据齐 | T-05,T-06；[x] |
| AC-07 | review | 独立上下文逐项验证AC-01…06及SD-01/02，绑定 Plan r3和实际实现提交/工作树摘要，结论pass且无必需门not_run | T-07；[ ] |
| AC-08 | merge | 评审后交付落主，SD-01/02沉淀canonical Spec且内容核验；归档/路线链接/交付及清理收据完整，发布说明限定为已验收的 Windows Native VM 和 Windows Vue | T-08；[ ] |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 影响文件/符号与独立产出 | 验证/预期 | AC / 进度 |
|---|---|---|---|---|---|
| T-01 | work | 无；现有来源重核 | §4来源、正式包导入/事件路径；新增`docs/reviews/plan096/decisions.md`，落实§5最小全链探针、instanceToken/首次error/模态焦点/输入桥可行性、真实包命令、平台及MCP提供方/依赖pin | 两端普通控件事件到应用读回；不是仅schema登记；差额有具名owner | AC-01；[x] Windows 普通设置与 Launcher provided 双轨探针见 §9；实际宿主启动由 T-04 验收 |
| T-02 | work | T-01 | 新增`packages/command-palette/`的数据/交互模型、校验、消息与身份协议；沿既有包元数据/相对导入 | §6有效/非法/迟到/双实例模型反例；无业务I/O进入包 | AC-01,02,03；[x] 9 个模型及包边界用例通过，含缺失 ID、分组冲突、同修订数据变化及重挂载 | 独立复审退回，见 §9。
| T-03 | work | T-02 | 同包UI/主题/键盘与MCP可见语义；必要现有runtime接缝最小修复 | 本组件case普通GUI/MCP路径可操作，IME/焦点和pending正确；对照参考图布局 | AC-02,03,04；[x] Windows 普通控件、模态焦点、IME、系统树、选择和请求锁双轨通过。宿主真实启动属 T-04 | 独立复审退回，见 §9。
| T-04 | work | T-03 | examples/command-palette/（新增）、examples/ui/028-launcher/src/front/app.at；复用§4实际业务入口 | §5消费路径输出真实变化；保持原业务权限/数据/模式，不同步兄弟仓 | AC-05；[x] Windows 两轨设置写回与 028 独立模式通过，排序对照一致。真实 Windows AutoOS 桌面搜索 `cal` 后点击一次已打开一个 Calculator 窗口 | 独立复审退回，见 §9。
| T-05 | work | T-03,T-04 | 新增`crates/auto-ui-tools/tests/command_palette_component.rs`、`tests/product/verify_command_palette.py`及必要隔离数据 | 运行§6两条产品命令/定向cargo目标；Windows 双轨、必需 case/分层MCP/输入桥/负控、失败清理/退出重开与必要旧消费者回归 | AC-01…06；[x] 9 个模型、Native VM 14/14、Vue 13/13，失败对照和共用嵌套模态回归符合预期 | 独立复审退回，见 §9。
| T-06 | work | T-05 | 新包README、普通示例说明、`docs/reviews/plan096/`证据与待沉淀SD；更新strategy实施事实 | case→AC、版本/截图/读回可追溯；受影响Q-01闭环；canonical Spec暂不冒称已交付 | AC-06；[x] 窄窗两轨实图未见 Q-01 字形缺失，验收和待发布 SD 正文已准备 | 独立复审退回，见 §9。
| T-07 | review | T-01…06 | 同一实施树，独立上下文最终评审与修复后的受影响复验 | AC-01…06逐项验收并核对SD；pass绑定精确实现revision，无必需门空缺 | AC-07；[ ] |
| T-08 | merge | T-07 pass | 合并/交付、`docs/specs/ui/command-palette.md`沉淀、Plan归档及strategy状态/链接更新、实施树清理 | 核验落主内容与评审等价、Spec字节/规则、交付与清理收据；不写specs.json/index.json/legacy ledger | AC-08；[ ] |

任务只保留本表一份当前进度；每项完成写实际证据，不预先勾选评审/合并。必要平台缺口留在对应AC，不能把产品正确性转给merge阶段兜底。

## 9. 复审记录

- 2026-11-14：stage: new；PLAN-096 r1；outcome: pass；next: work/T-01。新建T-01…08、AC-01…08及SD-01/02；已核来源、复用边界、参考图和平台矩阵，准备进入实施。
- 2026-11-14：stage: new；PLAN-096 r2；outcome: pass（修订后计划可进入T-01）；next: work/T-01。再次复核复用边界/图/平台矩阵，补instanceToken、精确异步确认、现有Init/MCP限制及分层输入证据、VMware路由、具名反例与失败清理；T-01…08、AC-01…08、SD-01/02保持，扩充§5/6并强化AC-02…06。详见[本轮审核报告](../reviews/component-plans-094-097-2026-11-14/README.md)。原r1记录保留为历史。
- 此pass仅指计划合同可执行；不是组件测试通过、独立实现评审或交付完成。本轮未创建实施树，未执行上列拟新增测试；current_step=0/8。

- 2026-11-23：stage: new | plan_id: PLAN-096 | plan_revision: r3 | outcome: pass | 用户明确授权仅 Windows。调整 AC-04/06/07/08、T-05/07/08、X-02 及 SD-02 的平台适用范围，保留其余验收入口与阈值。next: work/T-01。
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: 4dd9696da6a6289795ba9d6f57e603c24d99114a | next: work。继承 `/goal 完成计划096` 的全链路授权和用户确认的 Windows 交付范围。

- 2026-11-23：T-01 机制探针完成。Windows Native VM 与 Vue 经相对包导入、MCP 普通搜索/键盘/点击和设置真实写回；Launcher 独立运行经原 `ApplyFilter` 发布 provided 结果且具名拒绝缺宿主启动。原生定向键接缝复用当前模态焦点，初次配置通知使用正常 Tick。证据 `plan-artifacts/plan-096/native-settings-2/report.json`、`vue-settings-2/report.json`、`native-launcher-2/report.json` 与 `vue-launcher-2/report.json`。T-02/T-03 仍在闭合本轮模型、输入桥和视觉回归；T-04 宿主真启动待验证。实施决策在实施树 `docs/reviews/plan096/decisions.md`。

- 2026-11-23：stage: work | plan_id: PLAN-096 | plan_revision: r3 | outcome: blocked | worktree_path: D:/autostack/.wt/ui-096/auto-ui | branch: codex/ui-096-command-palette | base_commit: 4dd9696da6a6289795ba9d6f57e603c24d99114a | code_commit: a5929d9b8c4196e976a20580710182a4b64679a6 | dependency_revisions: auto-lang 8384ea064、auto-web 0cfc7c784、auto-down 456c86ac、AutoOS cdb4883 | task_ids: T-01…T-03 verified, T-04/T-05 open | evidence: 实施树 `docs/reviews/plan096/evidence.md` SHA256 `6880f4f2801039fbfa4964e4b3e347e56e2fc6185028ef6514fbe11cffd53c3a`。模型 9/9；Windows Native VM 12/12；Windows Vue 11/11；原生 IME、现有模态与提交定向回归和内置 Command 共存通过。028 空查询、`to` 和 tool 分类的排序结果与基线一致。 | blockers: AutoOS Windows `ui-desktop` 实际宿主未绘制。独立示例及 028 无宿主下的可用路径均已核验。真实操作和启动一次 Calculator 的宿主验收待补 | next_responsible_role: work after host prerequisite.
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 2 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: a5929d9b8c4196e976a20580710182a4b64679a6 | next: user
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 3 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: a5929d9b8c4196e976a20580710182a4b64679a6 | next: user。AutoOS 仍为 `cdb4883`，当前没有可接续的 AutoOS 桌面进程，§10 无新决定；未重跑验收。
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 4 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: a5929d9b8c4196e976a20580710182a4b64679a6 | next: user。连续第三个目标轮次遇到同一宿主前置条件；§10 尚无可用的 Windows AutoOS 会话或宿主启动修复授权。实施树干净，AC-05 及后续交付门保持待验。
- 2026-11-23：用户恢复目标并确认 Windows 桌面已可测试。T-04 通过。现有 `ICED_BACKEND=tiny-skia` 使 AutoOS `cdb4883` 的 `ui-desktop` 渲染出真实窗口。打开壳 Launcher 后，MCP 读回 `q = "cal"`；通过可见结果点击一次，宿主输出一行 `launch_app(inproc) 011-calculator`，取得实际 Calculator 窗口、键盘和独立嵌入树，随后宿主正常退出，原 PID、MCP 端口均回收。`hosted-resume-07/report.json` SHA256 `a72f63b1e8615235d568b70facf16ad38a4cfd929550e6975d01384300080e5b`；实施树证据提交 `3c032f2e519362131d7e1819688c9e0f0ed154ff`，无行为代码变更。
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: 3c032f2e519362131d7e1819688c9e0f0ed154ff | next: work/T-05
- 2026-11-23：stage: work | plan_id: PLAN-096 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/ui-096/auto-ui | branch: codex/ui-096-command-palette | base_commit: 4dd9696da6a6289795ba9d6f57e603c24d99114a | code_commit: a2a31db67d588299f5b951bf6d3f34e1fb0c3a24 | dependency_revisions: auto-lang 8384ea064、auto-web 0cfc7c784、auto-down 456c86ac、AutoOS cdb4883 | task_ids: T-01…T-06 | evidence: 实施树 `docs/reviews/plan096/evidence.md` SHA256 `3e067f2aa813954d012daa94e138d14a8c6a4f973d177bf8b9c708085909fcaf`；完整 Native VM 套件 14/14（68.962 秒，单案 3.839–8.417 秒），Vue 13/13（49.472 秒，单案 2.075–5.553 秒），缺提供方按预期退出 2；既有嵌套模态用例 Native VM 4.961 秒、Vue 4.022 秒。新同名命令分别读回 General/Layout（4.875/3.620 秒），中途失败保存最后树和回收进程端口（3.839/2.075 秒）。复用未变行为的 9 项模型与公共输入桥证据，实际宿主门见上项。候选 Spec Git blob `36786f200914e3f822e085c5c0779f3159bc35e3`。 | blockers: none | next_responsible_role: independent final review, authorized by user.
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 6 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: a2a31db67d588299f5b951bf6d3f34e1fb0c3a24 | next: review/T-07

- 2026-11-23：stage: review | review_scope: final | reviewer: /root/plan096_review（独立，用户授权） | plan_id: PLAN-096 | plan_revision: r3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-096/auto-ui | branch: codex/ui-096-command-palette | reviewed_commit: a2a31db67d588299f5b951bf6d3f34e1fb0c3a24 | base_commit: 4dd9696da6a6289795ba9d6f57e603c24d99114a | dependency_revisions: auto-lang 8384ea064、auto-web 0cfc7c784、auto-down 456c86ac、AutoOS cdb4883 | spec_inputs: 候选 blob 36786f200914e3f822e085c5c0779f3159bc35e3，目标 canonical 路径尚不存在 | acceptance_results: AC-01 pass，AC-02/03/06 fail，AC-04/05 partial | pending_closeout_gates: 受影响回归、落主等价、规范、归档和清理 | findings: R096-F01…05 | next: work。
  - R096-F01（AC-06/T-05）：新增聚合测试入口的 Native VM 68.961684400 秒、Vue 49.472130900 秒超过包装器 10 秒门。既往 `native-provided-3` 11.614346700 秒及动作失败保留。关闭条件为两轨逐案运行、全断言及结果传播保留。
  - R096-F02（AC-02/T-02/05）：首次打开 provided，重复 ID、尚无当前结果，实窗只呈现“Searching…”且零错误通知；先拦结果元组后校验所致。复现 `reviewer-provided-config/report.json`（SHA256 `286fdbefff29b8253993c19f70c34f7bfc1d37c5c4b77b50f7d602e6f74c0b18`，3.787364 秒，清理通过）。关闭条件为首次可见、一次通知、修正后实际可操作。
  - R096-F03（AC-02/03/T-02/03/05）：A 结果调用待确认时，旧查询序号的高修订结果使原生实窗放行 Escape。结果被拒绝前原始修订/命令属性已进入解锁条件。复现 `reviewer-stale-result-pending/report.json`（SHA256 `f16dca9ec13acf18ba9f09e6d80a290fdc8b7e5a390b83cfb9e63d24569b5cbb`，4.055839 秒，清理通过）。关闭条件为迟到结果不解锁、同会话当前变更仍失效、accepted 同帧继续保持。
  - R096-F04（AC-04/06/T-05）：runner 未核验准备收据，过期 Vue bundle 或不匹配工具可启动。Native 接受任意项目并继承分轨环境，可触发后端编译。关闭条件为绑定来源与正式预构建工具，缺失/过期/分轨构建配置明确拒绝。
  - R096-F05（AC-03/05/06/T-04/05/06）：补 provided 选中 ID 重排与移除断言、错误态实图、Calculator 稳定后的宿主截图。原始日志单次分派和应用身份有效，持久截图未显示计算器内容，另有实际操作窗口观察成功。
- stage: goal | plan_id: PLAN-096 | plan_revision: r3 | round: 7 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-096/auto-ui | commit: a2a31db67d588299f5b951bf6d3f34e1fb0c3a24 | next: work/R096-F01…05

## 10. 待澄清事项

r3 的 Windows Native VM 和 Vue 范围已确定。包语法/实例token/首次error/模态焦点/IME/反馈时序及现有Vue MCP提供方由T-01具名探针落实；T-02前关闭机制选择，不能把协议自由设计留给实现中途，失败须记录而非假定已支持。Windows Native VM 和 Vue 的工具及 MCP 提供方由 T-01 实查。缺失的必需能力记为 not_run。业务能力不在宿主上提供时明确不可用，按上述普通消费者和真实原生消费者各自验收。


### AutoOS 宿主首帧（已解决）

默认渲染器启动时观察到隐藏窗口且无首帧，恢复后使用现有软件渲染后端 `ICED_BACKEND=tiny-skia` 完成真实宿主验收，证据见 §9。没有修改 AutoOS 宿主源码或默认渲染设置。
