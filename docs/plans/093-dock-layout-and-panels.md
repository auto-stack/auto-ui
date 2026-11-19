---
plan_id: PLAN-093
status: drafting
feature_name: DockLayout 与 DockPanel 通用工作区布局
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-14
plan_revision: 2
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/dock-layout.md]
touched_goals: []
---

# PLAN-093 通用工作区停靠布局

## 0. 变更摘要

提供DockLayout / DockPanel，供编辑器、文件管理器、调试台、数据工作台在单应用窗口内组织侧栏/主区/底栏。复用现有可调整面板、tabs、slots与布局机制，首版不做OS浮动窗口。路线CL-16，见[组件库strategy](../design/autoui/component-library-extension-strategy.md)。本轮只立项，0/8。

## 1. 目标

水平/垂直分割、调整比例、标签分组、同窗口移靠、显示/关闭/恢复面板、默认布局重置、布局序列化交给应用保存。普通拖动、键盘及命名菜单操作共用同一布局命令，MCP不必靠屏幕坐标猜拖动终点。

非目标：OS级窗口停靠、跨应用窗口嵌入、多窗口浮动/跨屏、Dock工具栏启动器、专业场景渲染、关闭业务文档、重写所有示例。Windows/Linux native VM及Vue同义，macOS支持不以无实测声明。

## 2. 架构方案

应用提供面板注册/内容和受控layout树 → 校验 → 现有布局/slots呈现 → 用户操作发layout command → 应用接受新layout并可持久化。组件只持手势/焦点临时态，不持第二份业务文档/console/editor对象。

新增`packages/dock-layout/`，公共名字避免resizable内置冲突。现有runtime `native_dock`是OS原生窗口槽/Win32功能，不是编辑器DockPanel，不将其Windows专有机制用作跨平台实现。T-01必须证明任意面板slot及稳定内容身份的真实路径。

## 3. 技术栈

`.at`正式包、现有tabs/布局/slot及必要通用resize/指针事件、主题/MCP、VM/Iced和Vue。布局变更为数据，不增加语言语法或全局window manager。应用保存布局使用现有Storage，组件不写用户全局布局。

## 4. 需求分析与背景调查

用户要求约一半常见跨应用组件先建实施计划，本合同为通用工作台布局，当前阶段计划/路线，后续按Plan实施。未指定模型/预算/子代理或跨仓发布。观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读Specs索引、结构/主题/示例归属、native验证；无正式DockLayout Spec，schema声明不代表完整运行交互。

| 已核路径 | 观察和边界 |
|---|---|
| `crates/auto-ui-compiler/schema/aura.at` resizable/resizable_panel/resizable_handle | 有方向/比例/min/max声明，元数据web/iced为none；需沿实际converter验证，不据此假定可拖动 |
| `crates/auto-ui-runtime/src/ui/aura_view_builder/{slots.rs,child_widgets.rs}` | 现有组件slot和子组件装配位置，必须验证动态移靠不丢内容身份 |
| `crates/auto-ui-runtime/src/ui/native_dock/mod.rs` | NativeSlot/NativeHwnd/Win32 slot，不是本Plan要抽取的面板布局 |
| `examples/ui/041-auto-edit/src/front/{app.at,console_panel.at,editor_store.at}` | 已有编辑主区/控制台/侧栏；app SHA256 `24E261227E59B5E17B98E059C56BD59EFA5E64F5B259FFDC3096BA3E9B45C929` |
| PLAN-084/085及041现有actions | 前两计划也会改041，迁移消费时须等待其合入并重基线；动作/文档仍归Editor，布局不关闭文档 |

## 5. 详细设计

- 布局为有version的数据树，r2固定为可序列化的节点表+root引用；split含axis/weights/children，tabs含稳定panelId/active，面板元数据与slot内容分开。T-01验证slot机制能承接，不把代码字符串存入布局。
- 注册及布局中ID唯一、引用有效、每面板最多挂一次；重复/未知/循环/非法split比例具名拒绝，不静默丢失内容。恢复坏布局保留上一有效布局；首次坏布局使用明确的调用方默认布局并报告错误，不半发布。
- 首版在同窗口左右/上下移靠或合并标签组，移动与resize走统一命令模型；提供菜单“移到左/右/下/并入组”和普通可调比例入口/键盘操作，拖动也必须实装并复用该命令，不以只有静态分栏代替停靠。
- resize夹取面板minSize，窗口过窄时提供明确紧凑/可滚动策略，不能把活动内容或恢复操作挤出不可达区。Escape取消当前拖动恢复手势起点；没有业务值提交副作用。
- 关闭仅隐藏closable面板，可从可见面板菜单恢复，不关闭文档/丢数据；不可关闭主面板没有close action。默认布局重置同样通过受控请求，保留面板内容/文档状态。
- 面板移动/切标签/隐藏恢复不重建应用文档/播放器等业务会话；仍可见控件焦点按panelId恢复。若底层必须重挂视图，保留业务owner和稳定内容身份，真实Editor文档、光标/撤销不得丢失。
- 外部layout修订取消旧拖动；旧节点/命令绑定实例、layout revision、panelId，不能按tab位置命中另一个面板。同页双DockLayout、同名面板互不串扰。
- 布局序列化不含应用数据/凭证/原生句柄，应用持有按App/workspace区分的Storage键。重开恢复与未知版本拒绝由示例真实演示，不直接改主机用户工作区。
- 新独立工作台示例先证明通用slots，041消费再接其现有侧栏/编辑器/控制台，保留actions字符串锚及业务handler。T-04等待084/085交付后重基线，不并发覆盖其消费改动。

### r2 执行接口与样例

按用户2026-11-14再次审核与样式参考要求细化；先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)中093区域/窗口内布局，图中“文件/属性/console”只是应用传入的内容。

| 接口 | r2合同 |
|---|---|
| DockLayout props | `layout/defaultLayout:str`（JSON）, `panels:List=[]`, `revision:int=0`, `disabled:bool=false`；应用持有正式layout和存储键 |
| panel元数据 | `{"id":"assets","title":"资源","slotName":"panel-assets","closable":true,"minWidth":160,"minHeight":80}`；id唯一，slotName对应调用方实际提供内容 |
| DockPanel | 薄内容容器，接收`panelId:str`并保留默认slot；不自动另建文档、对象或一套title/closable真值，chrome由DockLayout的panel元数据生成 |
| 请求 | `layoutRequested(str)`含seq/baseRevision、command和完整合法candidate layout；command类型activate/move/resize/hide/restore/reset。应用只在身份仍匹配时原子接受整个candidate |
| 错误 | `invalid_layout/unknown_panel/invalid_slot/unsupported_layout_version/invalid_size`；坏候选不发布半树；无合法default时显示可恢复错误壳，不捏造面板 |

布局样例：

```json
{"version":1,"root":"split-main","nodes":[
 {"id":"split-main","kind":"split","axis":"horizontal","children":["g-assets","g-main"],"weights":[2500,7500]},
 {"id":"g-assets","kind":"tabs","panels":["assets"],"active":"assets"},
 {"id":"g-main","kind":"tabs","panels":["notes","preview"],"active":"notes"}
],"hidden":["console"]}
```

split至少两个children，weights为正整数、和10000，分别对应可分配尺寸比例；tabs的active必须在非空panels中。nodeId与panelId是不同命名空间，引用字段不得混用；每个注册panel恰好在一个tabs或hidden中。所有node从root可达、不得成环/多父；全隐藏时root=null/nodes=[]为正常空布局，恢复菜单仍可用；不可关闭面板不得隐藏。

将console恢复并移到g-main下方的完整请求例：

```json
{"seq":6,"baseRevision":2,"command":{"type":"move","panelId":"console","targetGroupId":"g-main","placement":"bottom"},"layout":{"version":1,"root":"split-main","nodes":[
 {"id":"split-main","kind":"split","axis":"horizontal","children":["g-assets","split-work"],"weights":[2500,7500]},
 {"id":"g-assets","kind":"tabs","panels":["assets"],"active":"assets"},
 {"id":"split-work","kind":"split","axis":"vertical","children":["g-main","g-console"],"weights":[7500,2500]},
 {"id":"g-main","kind":"tabs","panels":["notes","preview"],"active":"notes"},
 {"id":"g-console","kind":"tabs","panels":["console"],"active":"console"}
],"hidden":[]}}
```

placement为left/right/top/bottom/tab；tab目标为现有tabs。移动后空组删除、单子split折叠；hide只改布局，不删除业务内容。resize先按minSize夹取，再以固定10000权重规范化；Escape撤销未提交手势，普通绘制不发layoutRequested。上述样例registry应同时包含assets/notes/preview/console，不能只注册表中单条assets元数据。

尺寸分配先扣除split handle，再按权重分配；不足子树最小尺寸者固定到min，剩余空间按其余权重重新分配。若最小尺寸之和仍大于视口，则工作区保留该最小内容范围并提供可达滚动/面板菜单，不能把最小尺寸降到0。序列化权重按最大余数法归一到10000，余数相同按children顺序；T-01探针须用二分和三分布局核对实际bounds，而不是只比权重数值。

内容接法沿已存在的调用位`slot(name: "panel-assets") { … }`及outlet，不把组件函数/原生句柄塞JSON。**当前`aura/types.rs::slot_element_name`只处理字面量/Ident，字段表达式不能被当作已经支持的动态outlet**。T-01需验证按slotName绑定，必要时在UI compiler/Vue与runtime普通/tracked两路径作最小动态name求值适配，保留原字面量slot行为；不是新增语言动态代码或跨App窗口功能。

业务内容owner/身份按panelId持有，不依tab序号/当前路径；slots必须回调用方作用域求值。外部revision变化取消拖动，旧candidate不能覆盖新布局。拖动和可寻址菜单走同一candidate生成/验证，resize普通精确入口可用比例预设/键盘，不必另加087硬依赖。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/dock-layout.md | 无正式工作台布局→面板/分割/标签/移靠/校验/序列化合同 | 通用工作区复用 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/dock-layout.md | 无专门门→内容owner/焦点/MCP修订、Editor消费、平台/持久化门 | 调布局不丢业务状态 | AC-04,AC-05,AC-06 |
| SD-03 | add | docs/specs/ui/dock-layout.md | 字面量slot路径的现有边界→记录本组件所需动态outlet绑定及ordinary/tracked/Vue兼容义务 | 让面板可按稳定ID移靠 | AC-01,AC-04 |

拟树`D:/autostack/.wt/ui-093/auto-ui`、分支`codex/ui-093-dock-layout`，未创建。实现/最终独立review顺序同树，本轮不写canonical。

## 6. 测试设计

T-04新`examples/dock-workbench/`，T-05新`crates/auto-ui-tools/tests/dock_layout_component.rs`及`tests/product/verify_dock_layout.py`，统一`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`；均拟执行。T-01先以不同内容的多panel证明slot/事件/移动/销毁整条链，再确定必要引擎适配。

| Case ID | AC / 实际入口与输入 | 区分性断言 | work owner / 边界 |
|---|---|---|---|
| V-01 | AC-01,02 正式包多panel，重复ID/未知引用/循环/非法比例/坏恢复版本 | 合法slot实际显示；非法树受控拒绝且上一有效布局及内容完整，不半发布 | T-01/02/05 |
| V-02 | AC-02,03 普通拖动与菜单/键盘移靠、resize边界/Escape、标签/隐藏/恢复 | 两入口产生等价布局；比例/命中与实际bounds一致；取消恢复；不可关闭主面板无动作 | T-02/03/05 |
| V-03 | AC-04 同名双实例、layout换代、旧tab/handle action、移动正在编辑panel | 旧动作拒绝当前正例可用；另一实例不变；输入焦点/文档与撤销状态不丢 | T-03/05；真实内容owner |
| V-04 | AC-05 041编辑未保存文本→移靠/隐藏恢复→继续编辑/撤销；布局应用Storage后重开 | 文档内容/dirty/撤销准确，原actions仍可用；恢复布局正确、不把布局当文档数据 | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，浅深/窄窗/移根、正常退出重开 | 所有标题/菜单/resize可达，bounds/focus真实；owned进程/端口清理和新会话成功 | T-05/06；Q-01受影响项闭环 |

拟命令：`cargo test -p auto-ui-tools --test dock_layout_component`；`python tests/product/verify_dock_layout.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue另跑。T-01核定现有slot/布局回归目标及041真实读回入口，不把native_dock_e2e当跨平台面板验收。

测试通过当前树普通命令/手势修改布局，不注入layout字段。比较真实panel bounds和业务内容，不只比较序列化字符串。NVM身份/失败对照/正常退出重开必需，缺runner/slot能力保持缺口；最终review独立上下文同树，不在本轮创建代理。

### r2 补充验证与启动交接

- **V-06**（AC-01/02/04/06，work，T-01/05）：同一注册内容移到另一组时调用方变量/CodeEditor文档、光标/撤销仍正确；动态slotName真正取对应内容，不全部落默认slot；literal slot正例保留。对r2样例执行hide/restore/resize后序列化再读回，比较真实bounds；按指南§3.2拍窄窗/菜单/分割handle。
- T-01探针必须同时验证实际slot查找、ordinary/tracked及Vue、内容scope/存活；这里只证明静态三栏或JSON解析不够。失败若属于可在UI内修复的最小适配列精确路径；超出原范围则先needs_replan，不丢弃内容保持AC。
- T-04仍等084/085正式交付后重基线041；原actions锚、文档dirty/undo和console业务归应用。先跑独立workbench再迁Editor，不能以独立demo通过替代041消费门。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 正式包真实slots/解析/运行/移根，旧resizable和native_dock不被混淆或遮蔽；V-01/05 |
| AC-02 | work | 布局校验/拒绝保旧、split/tab/移靠/resize符合§5；V-01/02 |
| AC-03 | work | 拖动与菜单/键盘同义、取消/隐藏恢复/重置/不可关闭边界正确；V-02 |
| AC-04 | work | panel内容owner/焦点/双实例/layout修订与旧action安全；V-03/05 |
| AC-05 | work | 041真实编辑内容/dirty/撤销、原actions和应用布局保存/重开正确；V-04 |
| AC-06 | work | Win/Linux native+Vue、真实bounds/视觉/移根/退出重开证据齐全，受影响Q-01闭环；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立复核AC-01…06/SD、slot/内容生命周期及bounds oracle，固定提交，无必需门缺口 |
| AC-08 | merge | Spec、合并/归档/strategy最终链接及实际清理准确，不以merge补正确性 |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan093/decisions.md`验证动态slots/内容owner/指针和resize、恢复schema、041oracle/runner/Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 多panel整链真实探针，固定可行API/最小引擎改动；不假定schema声明已可拖动 | 01…06 |
| T-02 | work / T-01 | 新`packages/dock-layout/`树/命令/验证/受控布局，必要通用slot/resize修复由探针定位 | V-01/02坏输入保旧、真实移靠/比例断言通过 | 01…03 |
| T-03 | work / T-02 | DockPanel命中/标签/拖动/菜单/键盘/MCP/内容身份与focus | V-02/03两入口同义、旧事件拒绝/内容保留 | 03,04 |
| T-04 | work / T-03 + 084/085评审交付并重基线041 | 新`examples/dock-workbench/`先消费；041原侧栏/编辑器/console接包和应用Storage | V-04真实文档/撤销/布局读回，保留原actions锚和业务 | 05 |
| T-05 | work / T-04 | 新上述tools/product门，全部V落地及必要原slot/Editor回归 | §6入口逐平台/后端真bounds/内容/失败及退出重开；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 | work / T-05 | README、`docs/reviews/plan093/`证据、最终SD和strategy摘要；README含r2 props/events、可运行消费例和视觉指南 | 实际依赖/工具/布局版本、Q-01受影响项闭环可读回 | 06 |
| T-07 | review / T-06 | 同树独立review，暂停实现写入 | reviewed commit逐AC/SD结论；失败回work | 07 |
| T-08 | merge / T-07 pass | 新Spec、合并/归档/strategy交付及实际清理 | 最终版本/文件/树状态核对，不写派生索引 | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-093 r1；outcome: pass（规划可交接）；next: work，从T-01开始，T-04等待084/085的041消费改动。新增T-01…08、AC-01…08、SD-01/02，0/8；本轮无实施或产品测试。

2026-11-14修订交接：stage: new；PLAN-093 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-08/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。本修订新增SD-03并保留SD-01/02。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

## 10. 待澄清事项

r2已固定layout数据、消息与窗口内范围；T-01验证实际动态slots/稳定内容身份/指针resize及runner，在T-02前形成决策。无阻碍立项的用户选择；OS浮动窗口/跨屏另留后续，不能因槽机制难实现交付静态三栏或丢文档状态来通过。
