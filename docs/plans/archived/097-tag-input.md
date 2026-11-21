---
plan_id: PLAN-097
status: archived
feature_name: TagInput 通用标签输入
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-21
plan_revision: 2
current_step: 7
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/tag-input.md]
touched_goals: []
---

# PLAN-097 TagInput 通用标签输入

## 0. 变更摘要

补齐组件库路线 CL-07 的实施合同；提供标签显示、添加/删除、明确去重、静态候选建议、键盘、数量限制和业务拒绝反馈。用于笔记、阅读、分类及表单；持久化、权限、标签领域 ID 与异步候选服务归应用。见[持续跟踪 strategy](../design/autoui/component-library-extension-strategy.md)。当前为执行准备度复核后的drafting r2，0/8，组件和下列新增测试尚未实施。

## 1. 目标

Windows/Linux native VM 与 Vue 使用相同公共接口、状态语义和普通控件。用户键盘/鼠标与 MCP 驱动同一条事件链，自动测试不要求系统文件窗口或操作系统前台焦点。提供正式可移根组件包、普通消费示例、现有消费者迁移及无人值守证据。

非目标：标签数据库、远程补全服务、多级标签管理、自动解析逗号/批量粘贴、自动重写已有标签、跨仓 Reader vendor 同步。不在当前承诺中加入 macOS；将来支持须补真实 macOS 构建、窗口、字体、MCP 和生命周期验收，不能以交叉编译或 Windows 成功代替。

## 2. 架构方案

调用方 contextId/revision/tags → 局部输入草稿和候选 → changed 全量候选数组+add/remove元数据 → 应用接受/拒绝并更新tags/revision。计数来自受控已接受 tags，输入和本地焦点不等于持久化成功。

新增正式 `packages/tag-input/`，公共名为 `TagInput`。复用标准 Dialog/Input/Button/List/Progress 和既有主题/包机制，仅按本组件需要取用。编译器注册优先级 Builtin > Local > Package；禁止靠同名包覆盖已登记控件。模型/协议尽量单源 `.at`；必要共享运行时接缝须最小化并验证 native/Vue 等价，不造两份独立算法或 compiler→runtime 依赖。

应用持有业务数据、权限、I/O、持久化及执行效果；组件持有焦点、草稿、选中和防重复请求状态。普通重绘不换实例身份；context/会话真正变化才使旧动作失效。公共可见操作必须能按语义名称发现，不能只靠坐标或数组下标。

## 3. 技术栈

沿用 `.at` 正式包与普通消息、Rust compiler/runtime/tools、native VM、Vue及现有主题。测试复用 `crates/auto-ui-tools/tests/` 包导入/移根模式和 `tests/product/` 的进程、端口、MCP证据做法；Vue 使用真实浏览器操作。仅因应用需要服务时启动隔离的已有服务，不建设新的测试平台。

## 4. 需求分析与背景调查

授权来自2026-11-14用户请求：为四组件分别补齐实施计划，复用既有实现、引用已生成参考图、写明跨平台/MCP/无人值守验收并更新 strategy。本轮动作限本仓计划、设计导航及引用；不实施组件、不启动实施树、不提交其他WIP、不修改兄弟仓。用户未指定模型、预算、并行代理或自动实施；这里不推定授权。

2026-11-14本次复核HEAD `3cab09943079a1d1fca993fa5a4c4dd2a9fa8315`；084交付 `7ea71f0c`，086交付 `765251b9`。本次用户授权再次审查/补齐计划并推荐下一项，仍无实施、建树、跨仓写入授权。补读当前FileDialog/Pagination Spec及Windows系统语义合同；下列原始源文件摘要复核未变。

初稿观察基线 auto-ui HEAD `a6f776c24a4ba98611849955edceb5a04edac969`；未提交来源采用下表摘要/路径，不当作已交付版本。已读 [Spec总览](../specs/README.md)、`ui/{examples-ownership,theme-scope-and-publication,structure-closure}.md`、[原生验证合同](../specs/runtime/native-vm-app-validation.md)、`build/ui-test-selection.md`。当前无本组件专门Spec，拟在交付时新增；当前Spec不能因草案而改成已支持。

| 已核来源 | 复用边界与现存差额 |
|---|---|
| `examples/ui/015-notes/src/front/editor.at` | SHA256 333C583788796F28B2E59345E59F1E32F0A32007CD5A961EE7B8AEEFCFF61265；ShowTagInput/TagInputChanged/AddTag/RemoveTag 与 chips 可抽取；正文编辑继续留在 Editor。 |
| `examples/ui/015-notes/src/front/notes_store.at` | TagAdded/TagRemoved 已维护去重、update_tags 与笔记/vocab更新；保留应用存储入口，不能把HTTP或笔记状态塞进包。 |
| `D:/autostack/auto-reader/src/front/pages/book_detail.at（只读）` | Reader HEAD 36f438480326599472d3e83f43f14c32481e9d88；已有 new_tag/TagInput/AddTag/RemoveTag、trim及后端请求，可作后续消费映射，不在本Plan修改 Reader。 |
| `packages/file-dialog/`、`crates/auto-ui-tools/tests/file_dialog_component.rs` | 正式包导入、依赖闭包、移根和实际控件测试可复用；不复制文件业务/专用就绪协议。 |

实施 T-01 必须重新读取上述来源并记录实际 commit/hash、工具版本和脏文件边界。Reader 路径是当前主检出的只读参考，不依赖已消失的 `.wt/reader-009/`；Reader产品迁移、vendor同步和发布另属其自身计划。四项094–097相互无硬依赖，也不等待087；不为复用抽象平白新建第五个前置项目。

## 5. 详细设计

### 公共接口与所有权

| 接口 | r2合同 |
|---|---|
| props | tags:List[str]=[]；contextId:str=""；revision:int=0；suggestions:List[str]=[]；maxTags:int=0（不限）；disabled/readOnly:bool=false；label/placeholder/errorMessage:str；response:str="" |
| 公开消息 | changed(str)={seq,contextId,baseRevision,action:add/remove,tag,tags:[完整建议数组]}；error(str)={code,message}。seq 实例递增，标签按实际规范文本作稳定chip身份，不按索引。 |
| 规范化/去重 | 新增文字只裁剪首尾空格/Tab/CR/LF；非空才可加；裁剪后区分大小写精确去重，不做 Unicode/case 归一化。已有 tags 不在渲染时偷偷改名；父层传入空值/重复值为 invalid_config，需应用修正。 |
| 候选 | 静态 suggestions 去重并排除已接受标签；查询按 ASCII 小写化子串匹配，其他 Unicode 原文匹配，保留提供顺序。候选选择与手工Enter共用同一添加规则；过期props候选不能绕过maxTags或去重。 |
| 反馈/受控值 | 一次变更先本地锁；response={instanceToken,seq,contextId,baseRevision,status:accepted/rejected,message}明确确认当前请求。accepted必须与同contextId、tags等于本次候选数组且revision>baseRevision在同一props快照发布；rejected不修改tags。不能只凭外部值恰相同或revision+1推断成功。接受add清空草稿，拒绝保留；remove拒绝保持标签。 |
| 目标变化 | contextId或非本次预期的revision变化中止旧请求/草稿、展示新tags；同一目标接受时tags必须等于本次候选数组。实现需内部区分“当前请求确认”与“外部改动”，不能按任何 revision+1 都冒充接受。 |

事件载荷示例（JSON字符串，字段名固定；不是新MCP接口）：

```json
{"instanceToken":"app-1/mount-17","seq":5,"contextId":"note-12","baseRevision":8,"action":"add","tag":"Design","tags":["Reading","Work","Design"]}
```

`.at` 公共消息沿既有包的可导出命名方式声明（例如 `"changed"(str)`），由普通 `onchanged`/对应消息绑定消费。上表类型是接口语义，T-01只核对当前语法/封装方法，不把接口再交给执行者自由重设计。普通 props 数据通过组件正常输入提供，不从测试环境变量注入成功结果。

### r2 身份、反馈与复用接缝

每个实际挂载实例分配不复用的 `instanceToken:str`，普通重绘保留，卸载重建/另一个实例必须不同；其来源绑定App及真实组件owner/generation，不使用label、列表索引或每次渲染时间戳。所有请求、取消/关闭消息和异步response均增加此字段，调用方原样回传。T-01证明native/Vue中的可用分配机制；优先复用现有实例生命周期，只在确有缺口时补最小共享接缝。重建后seq/session即使重新从1开始，旧token也不能命中新实例。

同revision出现影响动作含义的数据变化也必须失效旧绑定，不能依赖调用方永不漏增revision。迟到消息先核对instanceToken、当前目标及活动请求，再处理业务状态；排队的用户动作还须在消费时检查可见、启用、当前modal与数据身份。两个同名实例、隐藏重开、卸载重建均配当前有效操作正例，禁止把所有动作都拒绝作为“隔离通过”。

复用 `crates/auto-ui-runtime/src/ui/modal.rs` 与 `ui/iced/renderer/host.rs` 的modal收窄、`ui.focus`/焦点路径，不另写全局键盘服务。配置错误首次挂载和相关配置变化必须可见；相同错误不每帧刷回调。保留 `error(str)` 约定，T-01验证首次配置错误恰好一次通知的正常调度路径：已交付分页Spec说明native子组件Init内emit不派发，不能只在Init中发事件。若需最小接缝修复，归本Plan T-01/03并验证兄弟组件，不默默删除事件合同。

### 状态、交互与消费适配

- 先按当前contextId/instanceToken验证反馈，再判断匹配请求的accepted快照，最后处理无关外部更新；不把ack自己的revision增量当外部切换而错误丢草稿。相同tags/上升revision但无匹配seq反馈，按外部替换处理，不声称本次请求成功。accepted与tags不一致是invalid_feedback，显示诊断并保持实际外部值，不擅自补写持久层。
- tags在同revision改变也失效旧请求/候选绑定，错误反馈和suggestions变化不会独自清空草稿。pending时保留已接受chips与计数，输入/增删/候选提交均禁用，直至匹配确认或真实目标替换；不会乐观删除。maxTags<0/重复或空标签阻止修改，已有数量超正上限是唯一修复例外：保留删除入口，拒绝新增，允许连续确认删除至合法。
- 015保存适配器在调用update_tags前捕获真实noteId和候选数组，回包只更新该noteId；当前活跃笔记已变时不得再调用依赖active_id的TagAdded/Removed来落盘。后端失败不递增接受revision，响应rejected并保留旧正式值；成功后按同一快照回传accepted及新tags/revision，再更新词汇表/过滤。T-01核对store async调用必须在实际生成路径正确await，不抽一个未await的异步兄弟handler伪造成功。

- Enter 添加活动候选或当前草稿，组合输入未结束时不添加；Up/Down移动候选且不提交，Escape只收起候选；Tab不自动创建标签。空草稿 Backspace 一次移除最后标签，长按重复依然受请求锁保护。
- 每个chip有名称明确的删除Button；disabled 禁用全部修改，readOnly 允许查看/选择文字但不删除/添加；maxTags=0不限、负值为配置错误。达到上限仍能删除，新增不发changed并显示具名提示；已有数量超过上限提示配置冲突但保留删除以便修复，不截断原值。
- 空白、重复、超上限都保留输入并显示输入邻近错误，计数不增加。逗号和粘贴默认一个完整标签，不未经请求拆分批量提交。每次合法手势最多一条changed，父层拒绝前不乐观抹掉真实chip。
- 015 Editor 用 TagInput 替代局部chip/输入；调用方从当前真实 noteId 而非列表索引构造 contextId/revision。事件核对目标后复用 TagAdded/TagRemoved 或等价单次 update_tags；changed中的全量候选用于一致性校验，一次手势一次存储更新，词汇表和现有过滤语义保留。
- 异步保存时切换笔记、重排、关闭编辑器不能把前一笔记草稿/迟到结果写入另一笔记；应用保存仍绑定原始noteId。原本尚无异步身份保护处由适配层补齐，不新增全局store。
- 新增 examples/tag-input/ 普通设置消费者提供候选/上限/错误与两个实例。Notes验收读回后端标签、重开笔记后仍一致，并确认正文未改变；Reader只记录字段对应供未来独立产品计划使用。

具名配置错误采用 `error(str)`，至少含 code/message；组件应显示错误并阻止相关修改，不用空列表或静默截断掩盖非法数据。业务拒绝与配置错误分开呈现。公开规模不承诺无限性能；不得未经记录默加“只支持前N项”的截断。新增资源上限或不兼容约束需先修订合同，不能只改测试。

### 视觉引用

固定普通消费数据：tags=[Reading,Work]、maxTags=5、suggestions=[Design,Development,Work]，输入de只得前两项；添加Design经应用确认后为3/5，重复Work保持2/5且零请求（独立从初态开始）。上限用maxTags=2阻止新增，再删Work成功到1/2；业务拒绝保留原值。Notes隔离服务/数据中建立真实note-A、note-B和不同正文，A加标签未确认时切B，检查两笔记后端标签与正文摘要分别正确，重开仍一致。T-04/05创建数据并记录实际服务启动/清理方式，不能在用户现有笔记上做破坏性验证；当前未运行。

直接复用[已生成的四组件参考图](../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1.png)及[文件导航/轻量交互视觉指南](../design/autoui/file-navigation-and-interaction-visual-guide.md)，只实现**右下：Reading/Work 标签、de 候选、2/5计数；Work重复是独立错误状态**。[生成提示](../design/autoui/assets/everyday-interactions-cl04-cl07-reference-v1-prompt.md)和最终PNG均为仓库设计资产，不依赖attachments或个人缓存。PNG SHA256 `EF667ABE23CFDC6377DDCF9E38005D118DD55D9CDCAFFD92D449CF02A440DD13`，1586×992 RGB。

样式沿[公共执行/视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)：32px控件、8px间距、16px面板padding、16px outline图标、约6px圆角，语义主题色和focus ring。图不是界面纹理或尺寸像素金标准；中文/长标签/错误、浅深色及420×560小窗以真实GUI可读可操作为准。图中文字不覆盖上表状态/异步合同。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/tag-input.md | 无正式专门合同→上述受控参数、消息、校验和身份/异步状态规则 | 收拢应用重复交互，保留业务owner | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/tag-input.md | 无本组件发布/验收约定→包导入/移根、普通消费者、跨端MCP/视觉/无人值守门及平台边界 | 可复用、可操作、可验证 | AC-04,AC-05,AC-06 |

拟实施树 `D:/autostack/.wt/ui-097/auto-ui`，分支 `codex/ui-097-tag-input`，本轮未创建。实现、自检、独立最终复审依次使用这一棵树；Plan进度留主检出，不在另一棵树同步实现。

执行入口是 T-01：来源已核、接口和AC已定，当前无外部组件交付阻塞。先走完整最小链路：**正式导入→输入/建议→changed全量数组→Notes单次update_tags→revision确认→切换笔记后的迟到反馈拒绝；验证当前真实noteId、IME和两个实例隔离。**。产物 `docs/reviews/plan097/decisions.md`（新增）记录可行路径、实际语法/命令、版本、待修接缝和探针证据；不得只看schema或源码文本。探针失败先在原目标内修接缝；若必须改平台/API/业务owner或AC阈值，则修订Plan并保留原要求，不擅自降级为截图/静态通过。

## 6. 测试设计

### 区分正确与错误实现的必需用例

下表每行 owner_stage=work、执行任务=T-05；入口是新增 `tests/product/verify_tag_input.py` 的具名case，以及新增 `crates/auto-ui-tools/tests/tag_input_component.rs` 中的模型/包边界反例。应用效果必须由正常页面/应用持久读回确认，不能通过直接调组件回调来制造成功。

| case_id | acceptance IDs | 代表输入/状态 | 可观察oracle / 失败对照 |
|---|---|---|---|
| G-01 | AC-01,AC-02 | 空白、Work重复、work、中文、逗号整串 | 空白/重复零提交；区分大小写；中文不损失；逗号不拆分；2/5计数仅接受后改变。 |
| G-02 | AC-02,AC-03 | max=2时新增/删除、disabled/readOnly、业务拒绝 | 上限阻止新增但允许删除；禁用/只读零写入；拒绝保留草稿和原tags，一次手势一次存储更新。 |
| G-03 | AC-03,AC-04 | 笔记A保存中切B、候选变更/旧节点、双实例 | 旧响应不污染B；旧候选不能越过当前校验；chip按文本身份删除正确对象，不按重排索引。 |
| G-04 | AC-04,AC-05 | Notes真实update_tags读回和重开 | 添加/删除与后端一致、vocab/过滤保持、正文摘要不变；无应用服务时具名不可用不能伪称持久化成功。 |
| G-05 | AC-02,AC-04 | Enter/IME、空输入Backspace、Esc/Tab、重复props | IME不提交；Backspace只删一项并等待确认；Esc/Tab不创建；重复props错误可见，外部数据不被默默重写。 |
| G-06 | AC-02,AC-03,AC-05 | 外部更新恰等候选值、无匹配ack；显式accepted与tags不符；同revision替换 | 只认完整身份匹配的accepted快照；不符为invalid_feedback且不二次写；同revision替换使旧节点/反馈失效；真实保存成功、拒绝、重建后当前正例均可用。 |
| X-01 | AC-01,AC-06 | 正式包从另一目录导入；缺依赖、同名内置并存 | 无绝对开发机路径，实际导入生成并运行；缺依赖具名失败，内置旧示例仍能解析运行，不以搜索字符串通过。 |
| X-02 | AC-04,AC-06 | Windows/Linux native VM 与Vue，当前树动作及关闭重开 | 真实窗口/非空树、实际控件动作与应用读回相符；错误对照返回明确失败，退出后本轮进程/端口回收，再次启动可用。 |
| X-03 | AC-06 | 浅/深主题、中文长标签、420×560、错误/禁用/pending | 同版本真实截图可读、正文可滚动、主要操作可达，无glyph缺失、重叠或误点击；空/加载态按组件适用状态覆盖。 |
| X-04 | AC-02,AC-03 | 卸载重建后seq重复、同revision数据变更；首次非法配置及修正 | 新instanceToken拒旧反馈，旧动作零误写且当前正例可用；配置错误可见且每次配置变化恰好一次error，修正后正常恢复。 |
| X-05 | AC-02,AC-04 | 多输入/双实例/嵌套modal；组合输入适用时 | MCP发现并操作当前目标，不把首个input代理误当真实焦点；底层按钮/快捷键被阻挡，Tab/关闭恢复正确；输入法按下文分层证据验证。 |
| X-06 | AC-04,AC-06 | 浏览器GUI通过但MCP不可用、启动/动作中途失败 | GUI不能替代MCP记录；缺必需能力整体not_run/非零退出；失败也执行finally清理、保存最后树与日志，原有进程不受影响。 |
| X-07 | AC-04,AC-06 | Windows原生真实系统树的输入/按钮 | 按native-vm-app-validation及native-system-accessibility核对受影响标准控件的role/label/value/focus/bounds/action；不以MCP树冒充系统provider或物理输入证据。 |

共同不变量：一次合法手势至多一条当前身份业务请求；非法/禁用/旧动作零目标变更；业务拒绝不伪造成功；接受后的真实消费状态与事件一致。组件树状态断言与消费者读回必须相互独立。测试通过“输入正确但状态owner写错/重复执行/旧对象覆盖”的反例来区分 plausible 错误实现，不能只计绿测试数量。

### 运行入口与证据合同

以下脚本和测试目标均是 **T-05待新增、当前未执行**；T-01在decision中落实真实工具参数/示例启动配置，脚本负责正常启动对应普通消费者，不能要求测试人员手工抢焦点。

```text
cargo test -p auto-ui-tools --test tag_input_component
python tests/product/verify_tag_input.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对MCP工具路径> --out-dir <本轮独立证据目录> --backend native
python tests/product/verify_tag_input.py --auto-bin <绝对CLI路径> --ui-tool-bin <绝对MCP工具路径> --out-dir <另一独立证据目录> --backend vue --browser-mcp-config <绝对浏览器MCP配置路径>
```

native采用 `auto run -r vm`，保留应用声明的真实后端（不能把`--server vm`当原生前端）。Vue及MCP入口按下文分层合同验证。只跑本组件、被修改公共接缝和迁移消费者的必要回归，保留既有oracle，不叠加无关整仓测试。

无人值守runner必须：每轮隔离数据/目录/端口；native校验自身拥有的OS窗口和非空live MCP树，Vue核验本轮页面/浏览器会话和实际MCP提供方；由当前snapshot发现真实节点再find/action/type/键盘/scroll/screenshot；动作后正常事件→应用读回；包含一项明确失败负对照；正常退出、核验本轮拥有的子进程和端口已释放并重开。原生动作不能依赖OS前台焦点，不能使用系统dialog、`AUTO_OPEN_PATH`、直接改组件状态、调用accepted处理器或回放旧node ID。超时须有限且保存最后树/日志，不能无限重试掩盖失败。

每个平台/后端分别记录 pass/fail/not_run：**Windows native、Linux native、Windows Vue、Linux Vue**均为本Plan必需门；缺Linux runner（按下文VMware路由，须真实窗口）记not_run，不得汇总成跨平台通过。macOS未承诺。浏览器无宿主/磁盘权限时保留不可用结果，不放宽安全边界。

收据 `docs/reviews/plan097/` 持有实际版本/依赖闭包摘要、命令/退出码、case→AC矩阵、截图/树、业务事件及读回结果、负控、生命周期与必要回归。产物大日志放独立证据目录，文档引用可复现位置和摘要；不要把个人缓存当唯一凭据。Q-01文字glyph缺失不能因MCP文本完整而结案；若公共渲染器超范围，建立具名修复依赖，受影响AC-06保持未关闭。

本轮只做草案静态自检，无产品验收结果。用户未分配实现/内部检查/最终评审模型；未来执行者先自检，最终评审须独立上下文并绑定实现revision，同一实施树顺序复用；不据此自行启动其他agent/chat。

### r2 可执行门与证据分层

- 每个平台均拆为普通GUI交互、MCP工具调用、应用读回、适用视觉/输入桥、退出重开；不要求把全部边界case在GUI和MCP重复笛卡尔积。§6每个case记录实际测试层；模型测试可直接喂模型，产品路径必须通过普通控件，不能用模型测试冒充真实消费。
- native复用 `crates/auto-ui-runtime/src/ui/mcp_server.rs` 的 `autoui_snapshot/action/type/keyboard/screenshot` 正常工具；保留RPC请求/结果和当前节点发现。Vue复用已存在的浏览器MCP提供方（或已交付生产桥），从真实DOM/可访问树进行输入/点击/键盘，记录提供方、工具版本、调用/结果和应用读回；Playwright只操作同一真实页面，不能单独作为“MCP通过”。本Plan不新建Vue MCP服务器；T-01定位当前配置，缺提供方则Vue MCP门not_run，不由四组件分别造桥。
- product脚本保留原拟议CLI，Vue增 `--browser-mcp-config <绝对配置路径>`（T-05待实现，配置描述实际提供方/会话接入；凭据不进报告）；两种driver共用case断言。native的OS窗口归属以owned PID/handle证明；Vue验证本轮生成页面URL、浏览器会话和服务PID，不套用native HWND证明。两条腿均有真实MCP操作与独立业务读回。
- Prompt/Command/Tag的IME门：普通 `autoui_type` 是整串替换，不证明composition、物理按键或OS输入法。Vue输入桥用compositionstart/update/end与isComposing事件序列；native使用当前标准输入适配层的preedit/commit/取消→Enter事件集成测试，观测生产事件抑制与一次业务提交，并补普通GUI/MCP中文最终值读回。已有 `autoui_desktop` 的acceptance专用handler/key通道不代替普通组件MCP产品门。若适配层无法表达，T-01定位最小输入接缝后补测试；TaskProgress无文本编辑，IME标明不适用，仍测正常键盘/modal。
- 复用当前Spec及已有包而非只复用FileDialog：正式包必须提供 `package.at/pac.at/组件.at/README.md`，native源相对import和Vue既有components/vendor路径解析分别运行；从第二绝对根验证完整依赖闭包和无开发机路径。四Plan不得分别重写同一公共runner/输入桥；第一个发现公共缺口者在decision登记owner/提交，后续重基线复用。
- Linux GUI验证沿084归档§10的用户决定：自制Linux发行版就绪后在VMware内单独验，不以WSL/WSLg结果替代，也不继续追WSLg修复。Windows门可先执行；Linux缺环境如实not_run且相应AC未关闭，不删除Linux要求、不预勾选最终评审。组件行为仍面向Win/Linux一致，macOS维持未承诺。
- 每个实际运行门输出 `report.json`：plan/revision/platform/backend、source/tools/dependency摘要、runner和MCP提供方、case/layer/期望/实测/status、截图/树/读回路径及摘要、cleanup/reopen、result。退出码0仅当本次门全部必需case通过且清理重开成功；失败为1，缺必需环境/能力not_run为2。聚合四门缺任何一门不能总pass。有效控制与关闭端口的同一MCP方法配对，关闭目标要求有界connection-refused，不能仅HTTP探测代替。
- 默认启动上限300秒、单动作15秒、退出清理30秒；runner可显式配置并记入收据，不能无限等待/重试。失败走finally清理本轮owned进程/端口/隔离数据并保存最后证据；只对确认身份的自有残留强制终止且写明原因。关闭/重开Dialog与退出/重启整个App分别验证；不把关闭UI误认为业务任务取消。

## 7. 验收标准

| ID | owner_stage | 可观察结果及验证方法 | 任务 / 状态 |
|---|---|---|---|
| AC-01 | work | 正式包公共接口按§5导入/生成/运行；X-01正反例、最小普通消费者及包依赖闭包可复现，无内置同名覆盖 | T-01,T-02,T-05；[x] |
| AC-02 | work | §5正常/错误/禁用/键盘/适用IME规则、本组件case及X-04/05全部有期望断言；无一次手势多次提交或虚假成功 | T-02,T-03,T-05；[x] F-01/F-02/F-08 关闭（backspace/IME/候选 ≤10s），见 §9 |
| AC-03 | work | 当前身份、重复请求锁、外部改动/迟到反馈/关闭重开/双实例隔离反例通过，旧对象零误写（含卸载重建与同revision内容变化） | T-02,T-03,T-05；[x] |
| AC-04 | work | 普通GUI和MCP通过当前可见控件完成关键用例；四个平台/后端门分别有结果，原生不依赖前台焦点；X-02/05/06/07分层门、生命周期与负控通过，缺必需MCP/平台门不算通过 | T-03,T-05；[x] |
| AC-05 | work | 015真实标签写入、读回/重开、词汇表和正文保持正确；普通双实例消费者跨端通过；Reader仅提供适配说明，不宣称其产品已迁移。 | T-04,T-05；[x] |
| AC-06 | work | X-01/03/06/07、移根、必要公共/消费者回归及真实截图通过；中文/暗色/窄窗可用，Q-01若影响本组件则闭环；README/待沉淀Spec与证据齐 | T-05,T-06；[x] |
| AC-07 | review | 独立上下文逐项验证AC-01…06及SD-01/02，绑定Plan r2和实际实现提交/工作树摘要，结论pass且无必需门not_run | T-07；[x] 复审 pass `3ca814bd`（§9；Linux 两轨按 084 §10 用户决定记 not_run，对应跨平台部分未关闭、不宣称通过） |
| AC-08 | merge | 评审后交付落主，SD-01/02沉淀canonical Spec且内容核验；归档/路线链接/交付及清理收据完整，不把未运行平台标为支持 | T-08；[ ] |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 影响文件/符号与独立产出 | 验证/预期 | AC / 进度 |
|---|---|---|---|---|---|
| T-01 | work | 无；现有来源重核 | §4来源、正式包导入/事件路径；新增`docs/reviews/plan097/decisions.md`，落实§5最小全链探针、instanceToken/首次error/模态焦点/输入桥可行性、真实包命令、平台及MCP提供方/依赖pin | 两端普通控件事件到应用读回；不是仅schema登记；差额有具名owner | AC-01；[x] |
| T-02 | work | T-01 | 新增`packages/tag-input/`的数据/交互模型、校验、消息与身份协议；沿既有包元数据/相对导入 | §6有效/非法/迟到/双实例模型反例；无业务I/O进入包 | AC-01,02,03；[x] |
| T-03 | work | T-02 | 同包UI/主题/键盘与MCP可见语义；必要现有runtime接缝最小修复 | 本组件case普通GUI/MCP路径可操作，IME/焦点和pending正确；对照参考图布局 | AC-02,03,04；[x] |
| T-04 | work | T-03 | examples/tag-input/（新增）、examples/ui/015-notes/src/front/{editor.at,notes_store.at}；复用§4实际业务入口 | §5消费路径输出真实变化；保持原业务权限/数据/模式，不同步兄弟仓 | AC-05；[x] |
| T-05 | work | T-03,T-04 | 新增`crates/auto-ui-tools/tests/tag_input_component.rs`、`tests/product/verify_tag_input.py`及必要隔离数据 | 运行§6两条产品命令/定向cargo目标；四门、必需case/分层MCP/输入桥/负控、失败清理/退出重开与必要旧消费者回归 | AC-01…06；[x] F-01..F-10 全关闭（复审 pass `3ca814bd`），见 §9 |
| T-06 | work | T-05 | 新包README、普通示例说明、`docs/reviews/plan097/`证据与待沉淀SD；更新strategy实施事实 | case→AC、版本/截图/读回可追溯；受影响Q-01闭环；canonical Spec暂不冒称已交付 | AC-06；[x] |
| T-07 | review | T-01…06 | 同一实施树，独立上下文最终评审与修复后的受影响复验 | AC-01…06逐项验收并核对SD；pass绑定精确实现revision，无必需门空缺 | AC-07；[x] 独立复审 pass `3ca814bd`（F-01..F-10 全关闭；Linux 两轨按 084 §10 记 not_run，不宣称跨平台） |
| T-08 | merge | T-07 pass | 合并/交付、`docs/specs/ui/tag-input.md`沉淀、Plan归档及strategy状态/链接更新、实施树清理 | 核验落主内容与评审等价、Spec字节/规则、交付与清理收据；不写specs.json/index.json/legacy ledger | AC-08；[ ] |

任务只保留本表一份当前进度；每项完成写实际证据，不预先勾选评审/合并。必要平台缺口留在对应AC，不能把产品正确性转给merge阶段兜底。

## 9. 复审记录

- 2026-11-14：stage: new；PLAN-097 r1；outcome: pass；next: work/T-01。新建T-01…08、AC-01…08及SD-01/02；已核来源、复用边界、参考图和平台矩阵，准备进入实施。
- 2026-11-14：stage: new；PLAN-097 r2；outcome: pass（修订后计划可进入T-01）；next: work/T-01。再次复核复用边界/图/平台矩阵，补instanceToken、精确异步确认、现有Init/MCP限制及分层输入证据、VMware路由、具名反例与失败清理；T-01…08、AC-01…08、SD-01/02保持，扩充§5/6并强化AC-02…06。详见[本轮审核报告](../reviews/component-plans-094-097-2026-11-14/README.md)。原r1记录保留为历史。
- 此pass仅指计划合同可执行；不是组件测试通过、独立实现评审或交付完成。本轮未创建实施树，未执行上列拟新增测试；current_step=0/8。
- 2026-11-21：stage: work；PLAN-097 r2；outcome: pass；worktree `D:/autostack/.wt/ui-097/auto-ui`；branch `codex/ui-097-tag-input`；base `ad62e154`；code 提交链见 docs/reviews/plan097/evidence.md；task_ids T-01…T-06；evidence docs/reviews/plan097/{decisions.md,evidence.md} 与 `D:/autostack/plan-artifacts/plan-097/` 门产物；blockers 无；next_responsible_role review（独立最终复审）。Windows native（demo 3×、notes 1×）与 Windows vue（demo 3×）门 pass；tools 模型门 6/6 pass；Linux 两轨无 VMware runner 记 not_run，对应 AC 保持未关闭；canonical Spec（SD-01/02）待 merge 落主。
- 2026-11-21：stage: review（final，独立上下文）；PLAN-097 r2；outcome: needs_fix；reviewed_commit `83467ad8`；findings F-01（Backspace oracle 缺失）、F-02（IME/中文/逗号/大小写 oracle 缺失）、F-03（G-03 过期目标未断言）、F-04（invalid_feedback/同修订未断言）、F-05（X-07 未执行）、F-06（spec 措辞 invalid-config）、F-07（勾选与清理检查）；next work。受影响 AC-02/03/04/06 重开。
- 2026-11-21：stage: work（修复轮）；PLAN-097 r2；outcome: pass；task_ids T-05/T-06（修复）与 AC-02/03/04/06 复验；修复内容：F-01 模型+backspace（`tag_input_component.rs` backspace 用例 + Vue keydown.backspace 步骤）；F-02 中文/逗号/大小写 native 步骤 + Vue composition isComposing 守卫步骤（生成器 `@keyup.enter` 守卫验证）；F-03/F-04 模型用例（stale-target abort、invalid_feedback、同修订替换）；F-05 X-07 真实 Windows UIA 系统树步骤（role/name/value/focus/bounds + ValuePattern.SetValue + InvokePattern.Invoke + 应用读回）；F-06 spec 措辞修正；F-07 计划勾选与实际 port_free/reopen 检查。门复跑：native demo 3×、vue demo 3×、notes 1×、tools 10/10 全 pass；next review（修复后复审）。
- 2026-11-21：stage: review（修复后独立复审）；PLAN-097 r2；outcome: needs_fix；reviewed_commit `771100ea`（HEAD，修复链 1d45063f+771100ea，基线 83467ad8）；deps auto.exe `eddb2cce…`、auto-ui-tool.exe `8133d125…`（哈希复核一致）、Playwright MCP 0.0.86+Chrome（browser-mcp.json 复核）；本轮独立重跑全部门（fresh out-dir `plan-artifacts/plan-097/re-review-*`）：tools 10/10（1.02s）、native demo pass 9/9（33.6–38.0s；port_free+reopen ok 真实）、vue demo pass 9/9（30.2s）、native notes pass（case 主体 7.32s）。F-01…F-07 全部 closed（含 X-07 真实 UIA system-tree.json Edit+Button 节点与 SetValue+Invoke 落地 chip 读回、spec invalid-tags 措辞、清理检查）；**新发现 F-08**：native demo 首例 `candidates-escape-enter` 三轮独立运行均 15.08–15.12s > 10s 上限，首次尝试确定性失败（`autoui_type "de"` 后 `wait_text("Development")` 8.21s 超时，重试 0.02s 即过；decisions.md #3 候选重算时序），断言虽 pass 但违反 2026-11-14 每 case ≤10s 裁定；影响 AC-02/T-05（重开）。Linux 两轨无 VMware runner 仍 not_run（084 §10 路由），对应 AC 未关闭不变。next work（T-05：调查 F-08 使该 case ≤10s 且不依赖重试）。
- 2026-11-21：stage: work（修复轮二/三）；PLAN-097 r2；outcome: pass；task_ids T-05；修复内容：F-08 有界快照模式（type→sleep→单次快照→单次重打；`candidates-escape-enter` 15.1s→7.470s）；F-09 notes case 固定 sleep 换有界轮询（chip 出现/空输入拒绝双目标判定 + 单次重打兜底；`notes:add-readback-switch` 8.829–12.210s→**1.391s**）；F-10 `port_free` 有界重试（20×0.5s）供 reopen 复用、`main()` 将清理/重开失败并入 result 与退出码（§6 契约）。门复跑（fresh out-dir `plan-artifacts/plan-097/fix-f09-f10/`，timing_wrap 计时）：tools 10/10（1.04s）、native demo 9/9（逐 case 0.234–8.328s 全 ≤10s）、vue demo 9/9（逐 case 0.531–6.234s 全 ≤10s）、notes native pass（case 1.391s）；全部清理 port_free=true、reopen.ok=true。HEAD `3ca814bd`。next review（修复后复审，重跑 notes 门 + runner 契约检查即可，不必全部门）。
- 2026-11-21：stage: review（修复后针对性复审，final 子轮）；PLAN-097 r2；outcome: pass；reviewed_commit `3ca814bd`（链 ad62e154→HEAD 共 11 提交；git status clean）；deps auto.exe `eddb2cce…`、auto-ui-tool.exe `8133d125…`（哈希复核一致）。fresh out-dir `plan-artifacts/plan-097/final-review-5[-demo]`：**F-09 closed**（notes case 墙时 2.938s ≤10s；report result=pass、cleanup port_free=true/reopen.ok=true、exit 0）；**F-10 closed**（`port_bindable` 有界重试 L74-84；Native/Vue port_free 与 reopen 复用；main() 清理/重开失败并入 result=fail，退出码 pass=0/fail=1/not_run=2 与 §6 契约一致）；**F-08 抽查 closed**（candidates-escape-enter 6.875s ≤10s，native demo 9/9 逐 case 0.125–6.875s 全 ≤10s）。F-01..F-07 沿用 §9 L210-211 证据 closed。tools 门修复提交未触及（10/10 证据保持有效）。AC-01/02/03/04/05/06 Windows 轨全过；**Linux 两轨无 VMware runner 仍 not_run**（084 §10 路由），跨平台部分保持未关闭、不宣称通过；AC-07/08 依本记录关闭/推进。next merge/T-08。
- 2026-11-21：stage: merge | merge_key: PLAN-097:r2 | checkpoint: prepared | outcome: pass | reviewed_commit `3ca814bd` | delivery_commit `4950ddd5`。对 master（`16e807d8`，基线上游 20 提交，均无关图表/AV）rebase 纯重放 11 提交，无冲突；`git range-diff ad62e154..3ca814bd 16e807d8..4950ddd5` 全部 `=`（补丁等价证明）。规范 `docs/specs/ui/tag-input.md` 落主字节一致（blob `c6378467…` == 工作树）。wt-guard.ps1 先 BLOCKED（生成 Vue 工程 node_modules 内 714 个 pnpm reparse point，门已跑完、产物已固化），按闸门指令逐链接摘除（junction `cmd rmdir` / symlink `Remove-Item`，714/714 成功）后重跑 **clean**。主检出 `git merge --ff-only codex/ui-097-tag-input` 成功，master tip = `4950ddd5`（无 merge commit），工作树/文档收尾无冲突。
- 2026-11-21：stage: merge | merge_key: PLAN-097:r2 | checkpoint: landed | outcome: pass | delivery_commit `4950ddd5`。默认分支 tip 等于交付提交；落主路径：`packages/tag-input/`、`examples/tag-input/`、`examples/ui/015-notes/`（迁移）、`crates/auto-ui-tools/tests/tag_input_component.rs`、`tests/product/{verify_tag_input.py,tag_input_preview.py}`、`docs/specs/ui/tag-input.md`、`docs/reviews/plan097/{decisions.md,evidence.md}`。
- 2026-11-21：stage: merge | merge_key: PLAN-097:r2 | checkpoint: canonical_verified | outcome: pass | canonical `docs/specs/ui/tag-input.md` 落主 blob `c6378467a1704755f2b34e990ac9a332040c55ee`，与工作树冻结 SD-01/02 字节一致；规范正文逐行等于冻结增量（74 行）。
- 2026-11-21：stage: merge | merge_key: PLAN-097:r2 | checkpoint: archived | outcome: pass | archive `docs/plans/archived/097-tag-input.md`（git mv + status: archived + 本收据）；导航更新并提交 `cc53ea42`：strategy CL-07 行、docs/plans/README 097 行、design-system-abstractions、file-navigation-and-interaction-visual-guide 097 行；历史审核报告链接修正提交 `cb0347e7`；全仓无 `plans/097-tag-input.md` 失效引用（0 处）。T-01…08 与 AC-01…08 编号齐（各 8 条），T-01..T-07/AC-01..AC-07 完成，T-08/AC-08 依本收据关闭；AC-04/06 Linux 部分按 084 §10 保持未关闭，不宣称跨平台支持。
- 2026-11-21：stage: merge | merge_key: PLAN-097:r2 | checkpoint: cleaned | outcome: pass | wt-guard.ps1 逐树 clean（auto-ui 及 auto-lang/auto-web/auto-down 三兄弟依赖 pin 树，均为 detached HEAD 干净状态）→ `git worktree remove` 全部解除登记、分支 `codex/ui-097-tag-input` 删除、空组目录 `D:/autostack/.wt/ui-097` 移除。移除 auto-ui 首轮因残留 vite preview 进程（本计划 vue 门遗留自有进程，VueSession.close 未杀净）持 CWD 而 Permission denied：按 §6 确认身份后强制终止 node 40944/24280，随后完整删除。构建中间产物随工作树消亡，未回收入 plan-artifacts。plan-artifacts/plan-097 整 scope 已删除（本收据与证据已折叠入计划 §9 与 docs/reviews/plan097/*，后者随交付提交落主）；registry `.retention/policy.json` 已刷新。

## 10. 待澄清事项

无必须向用户补问的产品决策；按§5边界执行。包语法/实例token/首次error/模态焦点/IME/反馈时序及现有Vue MCP提供方由T-01具名探针落实；T-02前关闭机制选择，不能把协议自由设计留给实现中途，失败须记录而非假定已支持。运行工具与Linux runner是否可用由T-01实查；缺失使相关验收not_run，不能自动扩大平台承诺或删除必需门。业务能力不在宿主上提供时明确不可用，按上述普通消费者和真实原生消费者各自验收。

