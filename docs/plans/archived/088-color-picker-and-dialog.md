---
plan_id: PLAN-088
status: archived
feature_name: ColorPicker 与 ColorDialog 通用选色器
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-17
plan_revision: 3
current_step: 7
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/color-picker.md]
touched_goals: []
---

# PLAN-088 ColorPicker 与 ColorDialog

## 0. 变更摘要

将路线CL-08提前立项：提供内嵌ColorPicker及组合ColorDialog，用于笔记/文字/画笔/主题预览等通用选色。复用现有Dialog、Slider、色板和087数值输入，不调用系统颜色窗口。见[组件库strategy](../../design/autoui/component-library-extension-strategy.md)。本轮仅建计划。

r3（2026-11-17 有界修订）：通道输入由「内嵌 PLAN-087 NumericField」改为「内置 input + cp 引擎夹取校验」——depth-3 组件实例（App→ColorPicker→NumericField）Tick/实例状态解析缺口阻塞内嵌方案（实证 decisions §7，VM runtime 债）。087 保留为语义基准（Enter 提交一次/同值零事件/非法具名拒绝/夹取），不再引入其实例。目标、验收实质、仓集不变；AC-01 措辞随依赖面修订（请复审确认）。

## 1. 目标

sRGB普通颜色、透明度、HEX/RGB/HSV输入、色板、当前/新颜色预览；实时预览与最终确认/取消分开。Windows/Linux native VM及Vue同义，MCP能精确输入数值而不必依赖色面拖动。Paint示例真实使用正式组件。

非目标：HDR/广色域/CMYK/ICC色彩管理、跨屏取色、渐变、全局色板存储、重写主题系统或图像处理。应用持有文档颜色及撤销历史，组件不直接写业务文档。

## 2. 架构方案

受控颜色→实例局部草稿/颜色空间投影→色面或数值输入→同一preview/commit链。ColorDialog组合ColorPicker，内部picker的手势提交只更新Dialog草稿，Dialog确认才向应用提交；取消通知结束预览，不修改正式值。

新增`packages/color-picker/`，不把Paint的Tailwind颜色类串变成公共颜色类型。通道输入（R/G/B/A、H/S/V）为内置 input + cp 引擎夹取/校验（整数 0..255 / 0..360 / 0..100；非法文本保留草稿并具名 `invalid_channel`，不发有效事件；Enter 提交一次、同值零事件——087 语义基准）。不内嵌 NumericField 实例：depth-3 组件实例 Tick/实例状态解析为 VM runtime 缺口（decisions §7 实证），修订裁定不引入。色面经 canvas 场景数据渲染（runtime 已按串长判别修 8 位 hex alpha），保持 compiler 零 runtime 反依赖；runtime 最小适配（canvas 颜色解码 + 合成动作组件作用域派发）已随 T-01/T-03 入库。

## 3. 技术栈

`.at`官方包、既有主题、Dialog/标准输入/Slider、VM/Iced和Vue。颜色计算/验证单一语义实现；必要native色面与Vue绘制只负责呈现/坐标映射，不能各自定义通道范围或舍入。

## 4. 需求分析与背景调查

用户要求选择约一半较常见组件先开实现计划，本合同为通用选色。当前阶段为计划/路线文档，后续按计划实施；无指定模型、资源预算、子会话或跨仓发布。观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读canonical `docs/specs/README.md`、`ui/theme-scope-and-publication.md`、`ui/structure-closure.md`、`runtime/native-vm-app-validation.md`及示例归属；当前无正式ColorPicker Spec。

| 已核来源 | 当前观察 |
|---|---|
| `examples/ui/031-paint/src/front/app.at` | 固定色板和完整Tailwind类串表示颜色，不是完整可编辑选色器；SHA256 `F45EFCE12052F5FA22232799C8C4FAA8B681D340FB04787144D62531BEB41B13` |
| `crates/auto-ui-compiler/src/style/color.rs`、schema中的slider/Input/Dialog | 可复用颜色解析及基础交互；需核定动态颜色值如何真正进入两端绘制，不能假定任意动态class可用 |
| `packages/file-dialog/file_dialog.at`和`ui/aura_view_builder/tags_panels.rs` | 现有普通modal/焦点/主题机制，复用而不新造系统弹窗 |
| `docs/specs/ui/theme-scope-and-publication.md` | 文档内容色与主题选择分开，不能选画笔色时修改整个App主题 |
| PLAN-087、strategy Q-01 | 通道输入正式依赖；真实文字缺失不得以MCP文本完整结案 |

## 5. 详细设计

- 公共受控值为规范化`#RRGGBBAA`，输入兼容`#RGB`/`#RRGGBB`/`#RRGGBBAA`；输出固定8位。RGB/alpha整数通道0…255，HSV为H 0…360、S/V 0…100；360等价0。舍入和无彩色时色相规则按r2执行，并以独立已知色值验证。
- 默认支持透明度，隐藏alpha控制时不得静默丢失已传入alpha；若要求不透明由显式配置和值校验处理。透明预览用棋盘底，不用黑底伪装。
- NumericField通道编辑、HEX、色板、色面/色相条都走同一颜色模型。非法文本保留草稿并显示错误，不触发有效颜色提交；未变颜色不重复commit。
- Inline Picker手势有begin/update/commit/cancel；pointer-up或Enter结束当前编辑，一次手势最多一次commit，Escape取消。外部只读/disabled时不可改色。可通过普通数值框完成与拖动等价的编辑。
- Dialog打开以当前受控值建草稿；preview与正式值分离，确认提交一次，取消/关闭/Escape不提交并让应用结束预览。外部正式值在编辑中改变时重基线草稿，失效旧手势，不把旧值当作取消恢复目标。
- 色板数据和最近颜色由调用方持有，组件不写全局偏好。两个picker/dialog、卸载重开和旧action按实例/草稿修订校验。Dialog沿既有modal输入隔离、焦点环及关闭恢复；初始焦点为HEX输入。
- 主题只影响控件外观，不改变内容颜色；浅深主题选同一色应输出同一RGBA。窄窗通道标签、输入和确认按钮可读可达。
- Paint接入将文档颜色改为独立数值/规范颜色表示，原像素文档/存取兼容需有适配和旧样本控制；取消选择不增加绘画历史。不得只把新颜色拼成未经生成器支持的动态类。

### r2 执行接口与样例

按用户2026-11-14再次审核和生成样式参考要求补齐。先读[执行与视觉指南](../../design/autoui/common-components-execution-and-visual-guide.md)，图中088是色面/通道/透明度/底栏关系；可切RGB/HSV通道组，不要求将两组全部挤进图中的一行。

| 接口 | r2合同 |
|---|---|
| ColorPicker props | `value:str="#000000FF"`, `revision:int=0`, `showAlpha:bool=true`, `palette:List=[]`（规范颜色串）, `disabled/readOnly:bool=false` |
| ColorDialog props | 同上，加`open:bool=false`, `title:str="选择颜色"`；父组件控制正式value/open，Dialog只持草稿 |
| 普通事件 | `preview(str)`, `committed(str)`, `cancelled`, `error(str)`；体如`{"seq":9,"baseRevision":7,"value":"#3974E6CC","source":"hex"}`，source为hex/rgb/hsv/palette/surface/confirm |
| 错误 | `invalid_color/invalid_channel/invalid_config`；非法props不回落为黑色假成功；用户草稿错误不发有效preview/commit |

通道框实现（r3）：内置 input 绑定 ch_* 受控串（0..255 / 0..360 / 0..100）；文本非纯数字或越界 → 保留草稿 + `invalid_channel` 具名错误，不发有效 preview/commit；合法输入即时应用 + preview（source=rgb/hsv），Enter 提交一次。统一数值规则：RGB/alpha输入整数0…255；H支持0…360且360规范化0，S/V为0…100。转8位RGB按最近整数、恰好一半向上；RGBA输出大写8位HEX。HSV在当前草稿变灰/黑时保留最近有效H，打开灰/黑初始值则H=0；这只影响编辑器草稿，不改canonical颜色。`#3974E6CC`必须得到RGBA(57,116,230,204)。

色面坐标约定：去除边框后的局部x映射S从0到100，y向下映射V从100到0，越界夹取；hue条左0右360、alpha条左0右255。触摸/指针、键盘与数值框共同调用同一转换，不各算一套。disabled/readOnly取消活动手势并禁止后续提交。

Picker状态idle→editing（可preview）→commit/cancel；一次pointer手势或Enter最多一条committed，blur不能重复发。Dialog拦截子Picker的committed为本地草稿更新；只有底栏确认向父提交一次。取消/关闭只结束本次预览，不把过期打开值覆盖外部新value；外部revision/value变化时重基线并失效旧手势。

showAlpha=false仅隐藏编辑，不丢已有alpha；本版不新增“强制不透明”的隐式模式。palette由应用传入，规范化后去除同值重复项，保持先后顺序；空色板可以正常选色。Paint业务适配必须在确认后才改正式画笔色，cancel只撤销预览。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/color-picker.md | 无正式选色合同→RGBA/HSV、色板、预览/提交/取消及受控模式 | 通用选色 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/color-picker.md | 无组件门→modal/实例/MCP/主题、Paint内容颜色兼容和双端门 | 可操作且不改错文档 | AC-04,AC-05,AC-06 |

拟树`D:/autostack/.wt/ui-088/auto-ui`、分支`codex/ui-088-color-picker`，未创建。实现/独立最终复审共用该树；canonical Specs本轮不改。

## 6. 测试设计

T-04新建`examples/color-picker/`，并迁移Paint普通选色流程；T-05新建`crates/auto-ui-tools/tests/color_picker_component.rs`、`tests/product/verify_color_picker.py`，沿统一`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`接口，均尚不存在/未执行。

| Case ID | AC / 实际入口和输入 | 区分性断言 | work owner / 范围 |
|---|---|---|---|
| V-01 | AC-01,02 真实包及颜色输入：红/灰/透明、短HEX、HSV边界、非法通道 | 已知颜色精确RGBA；HSV回转在明示量化误差内；alpha保留；非法草稿不提交 | T-02/05；sRGB8位 |
| V-02 | AC-03 inline手势、Dialog预览→取消/确认、Enter+blur、外部更新 | preview不写正式值；取消零提交；确认/手势仅一次；外部新值不被旧草稿覆盖 | T-02/03/05 |
| V-03 | AC-04 两实例、过期色面/按钮、disabled、Dialog背景/焦点 | 旧action拒绝，当前正例仍可改目标实例；背景不可操作、焦点恢复 | T-03/05；ordinary/tracked实际路径 |
| V-04 | AC-05 Paint普通选色→画一个像素→保存/读回；取消对照 | 像素真实颜色和提交值一致，旧文档样本可打开；取消不改文档/历史，主题切换不改像素 | T-04/05；既有业务入口 |
| V-05 | AC-04,06 Win/Linux native+Vue，色面/数值两入口，浅深/窄窗、移根、退出重开 | 两入口同色值、截图有透明棋盘和完整文字；真实窗口/树及owned回收后重开 | T-05/06；无runner不算pass |

命令由T-05落地：`cargo test -p auto-ui-tools --test color_picker_component`；`python tests/product/verify_color_picker.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，另跑Vue。T-01定位颜色解析/Dialog相关已存在runtime目标，按实际改动复验；不得凭空记录测试通过。

每轨记录源/087包/工具摘要、当前树发现、普通动作→正式消费读回、负例、截图及正常退出重开。无路径/事件注入；不依赖系统颜色弹窗或物理窗口前台焦点。Q-01受影响文字项必须关闭。独立最终复审为后续独立上下文，不在本轮创建代理。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/06，work，T-02/05）：实际组件输入`#3974E6CC`得到57/116/230/204；隐藏alpha后仍为CC；H360/H0等价；Dialog草稿多次变化→取消零正式提交、确认一次；按指南§3.2拍摄各状态与窄窗footer。
- T-01优先检查`crates/auto-ui-compiler/src/style/color.rs`既有解析和真正的动态绘制通道；色面可行性与语义对照写入decisions，不能用动态CSS类串或图片贴图代替真实控件。
- T-03通道输入（r3：内置 input + cp 引擎校验）的整数范围与 087 语义基准一致（Enter 提交一次/非法拒绝），不复制 087 算法。T-04旧Paint文档样本与新颜色数据之间的适配单列，失败不能当本地环境忽略。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 官方包真实导入运行（r3：087 为语义基准，其组件不内嵌——depth-3 缺口裁定，见 decisions §7），旧颜色/基础元素兼容，第二根可消费；V-01/05 |
| AC-02 | work | RGBA/HSV/alpha、非法输入和各编辑入口同义；V-01/05精确数值及量化断言 |
| AC-03 | work | 预览、取消、确认、手势提交和外部更新遵循§5，单次提交；V-02 |
| AC-04 | work | MCP、键盘、modal焦点/背景、双实例和旧绑定安全；V-03/05 |
| AC-05 | work | Paint真实像素/保存读回、旧文档兼容、取消与主题对照通过；V-04 |
| AC-06 | work | Win/Linux native+Vue、浅深/小窗、移根、退出重开证据齐全，受影响Q-01无缺口；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立复审AC-01…06/SD及真实颜色/文档oracle，固定提交，无必需门缺失 |
| AC-08 | merge | Spec、交付/归档、strategy互链和实际清理准确；提供092可消费版本，不手写派生索引 |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan088/decisions.md`核定颜色解析/动态绘制/舍入/事件、Paint存取兼容、runner/Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 最小真实色面及数字输入探针，明确可行路径，不以class字符串模拟任意色 | 01…06 |
| T-02 | work / T-01 | 新`packages/color-picker/`颜色模型和局部编辑事务，必要底层绘制接线先由T-01定位 | V-01/02已知值及失败控制通过 | 01…03 |
| T-03 | work / T-02 | ColorPicker/ColorDialog、通道输入（r3：内置 input + 引擎校验，不嵌 NumericField）、焦点/键盘/MCP/实例修订 | V-02/03跨入口、旧事件、modal正反例通过 | 02…04 |
| T-04 | work / T-03 | 新`examples/color-picker/`、Paint正式包消费和旧文档适配 | V-04像素/存取精确读回，取消零业务修改 | 05 |
| T-05 | work / T-04 | 新上述tools/product门，所有V落地并按平台执行 | §6命令，双端真实颜色/业务结果、失败对照和退出重开；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 | work / T-05 | README、`docs/reviews/plan088/`证据、最终SD和strategy实施摘要；README含r2 props/events、可运行消费例和视觉指南 | 完整依赖/工具身份及Q-01闭环；不缺门冒称完成 | 06 |
| T-07 | review / T-06 | 同树独立复核，冻结实现写入 | reviewed commit逐AC/SD结论；失败回work修复 | 07 |
| T-08 | merge / T-07 pass | 新Spec、合并/归档/strategy交付链接及实际清理 | 最终版本/文件/树状态核验，给092交付pin | 08 |

r3 复审（2026-11-17，§9 末条）重开 T-02/T-03/T-04/T-05/T-06；current_step 按完成数重算为 1/8（仅 T-01 保持完成）。

r5 复审（2026-11-17，§9 末条）重开 T-03/T-05/T-06；current_step 按完成数重算为 3/8（T-01/T-02/T-04 保持完成）。

r6 复审（2026-11-17，§9 末条）重开 T-05/T-06；current_step 按完成数重算为 4/8（T-01…T-04 保持完成）。

r9 复审（2026-11-17，§9 末条）final pass：T-05/T-06 关闭，T-07 完成，current_step 7/8（待 T-08 merge；Linux/Spec 等 merge-owned 门未消）。

## 9. 复审记录

2026-11-14：stage: new；PLAN-088 r1；outcome: pass（规划可交接）；next: work，从T-01开始，T-03等087。新增T-01…08、AC-01…08、SD-01/02，0/8；本轮未实现或运行产品测试。

2026-11-14修订交接：stage: new；PLAN-088 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-03/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

2026-11-17（goal，终诊轮）：outcome: blocked（维持）| 终诊证据：索引修正后 hue+surface 精确通过，alpha/channel 的失败稳定复现——前一个实例作用域 handler 的整型字段写对后一个 handler 不可见（d_s 读 0、d_g 读 d_b；串字段 d_hex 写可见）——nanbox/绑定腐化家族，VM runtime 债（decisions §7.1 终诊段，提交 63034852）。§10 二选一仍待裁定；不可达验收（AC-02 native 通道证据、AC-06 native 完整性）在 VM 修复前无法诚实出具。

2026-11-17（goal，等待轮 3 → blocked）：stage: goal | round: 4 | outcome: blocked | §10 复读：仍无裁定。连续三轮等待，按协议标记 goal blocked。精确待决问题：通道输入（AC-02 编辑入口之一）在两套设计下均被 VM runtime 缺口阻塞——(a) 内嵌 NumericField：depth-3 实例 Tick/状态解析缺口；(b) 内置 input：Tick 重基线后实例 handler 字段读错位（native）+ 提交路径 codegen 'length' 错（vue）。二选一：①先立 VM runtime 修复计划（实例作用域 handler 状态一致性 + depth-3 实例语义），PLAN-088 挂起等其交付；或 ②通道入口验收降域（native 记债、Vue 轨补证——需先修 vue 提交 'length' 小错），随后继续 T-05 剩余门与 T-06..T-08。已交付并验证：T-01/T-02、T-04（Paint 消费门 V-04）、T-05 工具门 2/2、核心选色链双轨、vue V-05。恢复动作：回复 ① 或 ②。

2026-11-17（goal，等待轮 2）：stage: goal | round: 3 | outcome: waiting | §10 复读：无裁定答案；无阶段工作、无检查。等待轮计数 2/3。

2026-11-17（goal，等待轮 1）：stage: goal | round: 2 | outcome: waiting | 期间产出：门脚本 canvas 索引修正（hue/alpha 写反）→ hue+surface 精确通过；§7.1 精化诊断（Tick 重基线后实例 handler 字段读错位——alpha 读 d_s=0、channel 读 d_g 得 d_b 值；提交 03ed1d2f）。§10 二选一裁定仍待用户；继续等待。

2026-11-17（goal）：stage: goal | plan_id: PLAN-088 | plan_revision: r3 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-088/auto-ui | commit: f0142ba8 | next: user——裁定 §10 阻塞选项（①先立 VM runtime 修复计划 ②通道验收降域+记债）。

2026-11-17（work，r3 实施→blocked）：stage: work；PLAN-088 r3；outcome: blocked（通道输入双设计均撞 VM/生成器缺口，需用户决策）；worktree `.wt/ui-088/auto-ui`；code 分支头（r3 实施 WIP 已提交）；task_ids T-03 通道框（实施完成，验证受阻）T-05（native 门：V-04 paint✅、V-03✅、V-05 reopen✅；V-01/02 通道相关受阻；vue 门 V-05✅）；evidence decisions §6/§7 + probe-out/demo*.log + gate-native/gate-vue 报告。实证：native 内置 input 通道经实例 handler 读状态出别名值（R=64 提交 #40E6E67F——g/a 读到陈旧/错位字段），vue RPreview 出 #FF74E6CC 且 Enter 提交路径抛 codegen 'length' 错。对比：同 build 的 palette/pen/hex/dialog 提交链 native+vue 全对（demo9-11/20、gate V-04/V-05）。unblock=VM runtime 修实例作用域 handler 的状态读写一致性（+vue codegen 提交路径），或用户裁定通道验收降域（vue 单轨/记债）。

2026-11-17修订交接：stage: new；PLAN-088 r3；outcome: pass（有界修订合同就绪，授权沿 /goal 完成计划088 全链）；next: work。变更：§2/§5/§6/§7/§8 通道输入内置化 + AC-01 措辞（请复审确认）；T-01/T-02 完成证据与 T-04/T-05 已实施部分沿 r2 记录保留。depth-3 缺口登记为 VM runtime 债（decisions §7）。

2026-11-17（work，needs_replan）：stage: work；PLAN-088 r2；outcome: needs_replan（有界修订——仅通道输入设计）；worktree `.wt/ui-088/auto-ui`；branch `codex/ui-088-color-picker`；base 2f42d661；code 458bfc60；deps 087 已归档（r3）、auto-lang/auto-man @ master 组内；task_ids T-01✅ T-02✅ T-03（实施后被 depth-3 缺口阻塞→修订）T-04（示例+Paint 迁移已实施，门证据待修订后）T-05（工具门✅2/2；产品门脚本已写，待修订后实跑）T-06/T-07/T-08 未开始；evidence docs/reviews/plan088/decisions.md §3/§6/§7 + plan-artifacts/plan-088/{probe-out,gate-native}；blockers=depth-3 组件实例 Tick/状态缺口（VM runtime 债，实证在案，本计划不修 VM）；next_responsible_role=new（有界修订：通道输入内置化）。T-01/T-02 完成证据：双轨探针 PASS 13/13、引擎已知值逐字节核对、工具门 2/2（5.25s/4.29s）。T-04 现状：examples/color-picker 消费链双轨实证（palette p2→#3B82F6FF、pen 手势 preview/commit #405980FF、baseRevision 追踪）；Paint 迁移后 native 实跑（选色→落格→存→New→Load=loaded）。运行时修复（已入库）：canvas 8 位 hex alpha 串长判别、pen/move/hit TypedInst 基座 + MCP press/pen 组件作用域派发。

2026-11-17（work，进行中）：T-01 ✅。树 `.wt/ui-088/auto-ui`@`codex/ui-088-color-picker`（base 2f42d661；T-01 提交 aaf43e8c+cb8fd0cb，087 依赖已归档交付）。探针双轨 PASS：native MCP pen 选色逐字节对齐预计算引擎值（#405980CC/#805040CC/#8050407F）、onhit 色板、dialog 内嵌；vue Playwright 真实拖拽/点击/棋盘 alpha 渲染。平台缺陷发现与处置（runtime 最小适配 2 文件已修：canvas 8 位 hex alpha 按串长判别；`.split()` List 债与 vue 形参 `.value` 债以 .at 侧规约规避）详见 `docs/reviews/plan088/decisions.md` §3。当前 1/8，next T-02。

2026-11-17（goal）：stage: goal | plan_id: PLAN-088 | plan_revision: r3 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-088/auto-ui | commit: 32f59993 | next: review——T-05/T-06 收口，native+vue 产品门绿，进入独立复审。

2026-11-17（work，r3 续跑 → execution_done）：stage: work；PLAN-088 r3；outcome: pass；worktree `.wt/ui-088/auto-ui`；branch `codex/ui-088-color-picker`；commits `32f59993`（实现+README+产品门）/`433f5cfe`（decisions §8）；task_ids T-03✅（内置 input 通道；V-01 通道 #4074E6CC + invalid_channel 事件）、T-04✅（示例/Paint 消费门沿旧）、T-05✅（native `gate-native-r3t` 8 用例 + reopen + 640x560 窄窗浅深截图；vue `gate-vue-r3g` pass；`cargo test -p auto-ui-tools --test color_picker_component` 2 passed 7.02s；Linux not_run 如实记录）、T-06✅（`packages/color-picker/README.md` + decisions §8 证据/边界）；evidence `plan-artifacts/plan-088/gate-native-r3t/report-native.json`、`gate-vue-r3g/report-vue.json`、decisions §7.2/§8；实现要点：串字段单一真源（d_hex/ch_*/h_hold）+ 每 handler 入口解码（不再有跨 handler 整型草稿状态；VM 整型字段写可见性债未宣称修复但不再在关键路径）、HSV 解码饱和度系数 ×10 修正（51 处）、行/alpha 条高位半字节 `/16`、rgb6 `substr(1,6)`（28 处）、ColorDialog 补外向事件声明（preview/committed/cancelled/error）、通道错误渲染+error 事件、vue 第二解码块改赋值（重复 let）、示例窄窗堆叠与输入列 min-w-40；残余边界：初始焦点无平台 autofocus 通道、Linux not_run；§10 阻塞撤销；next: review（T-07 独立上下文）。

2026-11-17（review，final，独立上下文）：stage: review | review_scope: final | reviewer_identity: 独立子代理 reviewer-088（与实现者上下文分离；会话未暴露模型 ID，不臆造） | plan_id: PLAN-088 | plan_revision: 3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-088/auto-ui | branch: codex/ui-088-color-picker | reviewed_commit: 32f59993（HEAD 433f5cfe 为其 docs-only 子提交，仅 decisions §8 +40；`git status --porcelain` 空） | base_commit: 2f42d661 | dependency_revisions: PLAN-087 归档 45a80058（沿 decisions §4，本轮未复验）；auto-lang/auto-man 组构建（未复验） | spec_inputs: docs/specs/ui/*.md（当前，无 color-picker 合同）；docs/specs/ui/color-picker.md 在主树与工作树均不存在（SD-01/02 目标，merge 落文前需按实际行为成稿）；主树相对 base 在本计划触及路径（packages/color-picker、examples/color-picker、031-paint、runtime 4 文件、tests/product）无漂移（`git diff --stat 2f42d661..HEAD -- <paths>` 空） | acceptance_results: AC-01 pass；AC-02 partial；AC-03 partial；AC-04 partial；AC-05 partial；AC-06 partial；AC-07 needs_fix；AC-08 pending | pending_closeout_gates: Linux native 轨 not_run（AC-06，需 merge 批或补 runner 后才可称完整）；Spec 落文（AC-08/SD-01/02）；strategy CL-08 状态与 Q-01 关闭陈述；plan-artifacts 与 repo 临时产物清理 | findings: F-01…F-11（下详） | next: work 修复 F-01…F-11（T-02/T-03/T-04/T-05/T-06），重跑受影响门后回 final review；不合并、不改 Spec。

复审执行（可复核）：
- baseline：`git -C <tree> log --oneline 2f42d661..HEAD` = 9c37d07a/06b92b0b/458bfc60/f0142ba8/03ed1d2f/63034852/32f59993/433f5cfe；32f59993 仅改 6 文件（color_picker.at 2266 行变更、color_model.at、color_dialog.at、demo.at、README、verify_color_picker.py）；`git status --porcelain` 空；实现写入已全部提交。
- 工具门（重跑，未重复构建）：`PLAN088_ARTIFACT_ROOT=plan-artifacts/plan-088/review-tools ./target/debug/deps/color_picker_component-6f391a374d2b3593.exe --nocapture` → exit 0，2 passed，8.06s（用例 1 两生成器编译、用例 2 vendored closure+负例）。二进制建于 13:59（测试源 12:53、compiler/tools rlib 11:12/12:40，均早于构建且未再改），用例在运行期读取被审 .at，故对 32f59993 有效。另：lang-971/lang-1066/lang-977 的 cargo/rustc 在 15:16–15:18 占用，未另起 cargo 构建（避免重复构建/争用）。输出含 `S001 INFO ... prop 'title' not declared on <button>`（见 F-11）。
- native 产品门复跑：`python tests/product/verify_color_picker.py --auto-bin .wt/ui-088/auto-man/target/debug/auto.exe --ui-tool-bin .wt/ui-088/auto-ui/target/debug/auto-ui-tool.exe --out-dir plan-artifacts/plan-088/gate-native-review1 --backend native` → exit 0，GATE PASS，41.8s；8 用例 + reopen pass + 3 截图；`demo-native.png` sha256 b8340f445c428bdd098ba25a247a95c70df65785c118c32631bfe5cd6dcb7ff2，与 r3t 逐字节相同（状态确定性复现）。复跑在 Paint 树留下 1 张未跟踪截图（`examples/ui/031-paint/src/front/tmp/`，无 .gitignore），已删除；工作树恢复干净（F-10 的机制实证）。
- 引擎执行（生成物反例）：工具门重新生成 `plan-088/last-gen/ColorPicker.gen.vue`（15:19）后，node 抽取其中 cp_hex_valid/cp_hex_norm 执行（脚本 `plan-artifacts/plan-088/review-tools/hexcheck.js`）：`#3974E6CC→#3974E6CC`、`#3974E6→#3974E6FF`、`#F00→#FF0000FF`，但 `#F00A→#FF0000AAFF`、`#3974→#33997744FF`（10 位；validCode=0）——见 F-03。
- 反例探针（native，真实窗口+MCP；脚本在 plan-artifacts/plan-088/review-tools/）：对话框打开后按背景按钮被拒 `Modal dialog owns input; target is outside the active dialog or no longer exists`（modal 隔离反例成立）；一次「取消」出现两行 `label 取消（零业务修改）`（probe-dialog/after-cancel.png、probe5 日志）→ F-05；对话框确认可提交一次 `label 接受 #805040FF` 并关闭（probe-dialog4/5）；首次打开后立即 pen 会被 parked Init 吞掉（约 0.3s 内），非用户可达操作，未计缺陷。
- 视觉抽检：`demo-narrow-native.png`/`demo-native-dark.png`（640×560 浅/深）HEX #3974E6CC、R57 G116 B230 A204 可读，透明棋盘在；`gate-vue-r3g/vue-light.png` #4074E6CC、R64 G116 B230 A204；`vue-narrow.png`（420×560）HEX 输入与 RGB/HSV 按钮右侧被裁（F-08）；`demo-native.png` 在 HEX/RGB 已显示 #22C55EFF 时仍留红色 `invalid channel`（F-06）。
- 成本/口径：native 产品门是真实窗口验收 harness，非日常单测（重跑 41.8s 端到端）；工具门 2 用例共 8.06s，单用例均 <10s（计划记 7.02s，量级一致）。未跑全局回归（按指示）；Linux 无 runner 未跑（not_run）；vue 门未重跑（r3g 15:10–15:12 与 32f59993 输入一致，重审其报告与截图）。

| 范围 | 入口/命令 | 结果 | 时间 |
|---|---|---|---|
| 工具门（2 用例，含新增 2 例） | 预编译 `color_picker_component-6f39…exe --nocapture` | 2 passed / 0 failed，exit 0 | 8.06s（单用例 <10s；计划记 7.02s） |
| native 产品门（验收 harness，非日常） | `verify_color_picker.py --backend native` | GATE PASS，exit 0 | 41.8s 端到端 |
| vue 产品门（复用 r3g，未重跑） | `verify_color_picker.py --backend vue` | pass | 未在本轮观测（复用） |

发现（severity｜AC｜task｜证据｜修法）：
- F-01（high｜AC-05｜T-04/T-05）Paint 像素 oracle 缺失：`tests/product/verify_color_picker.py:426-462` 的 V-04 只断言 legacy fixture/Fixture save→`loaded`status、Undo、Save/New/Load 的 `loaded`，画 p1 后立刻 Undo，从未读回 px42 真实颜色（`paint-native.png` 也未采样）；V-04 要求的「像素真实颜色=提交值、取消不改文档/历史、主题不改像素」无断言。修：画后按像素/状态读回断言 #EF4444FF，再覆盖取消与主题对照，保留 legacy 样本。
- F-02（high｜AC-02/AC-06｜T-05）V-01/V-06 必测面在门中无行为断言：无 HEX 输入路径（门用例仅 known_colors/gestures/channel/two_instances/business_reject/dialog/paint/narrow），无短 HEX/非法 HEX、无 showAlpha=false 保 alpha、无同值零事件、无 Enter 单次提交；工具门只有生成串标记。`#3974E6CC⇒57/116/230/204` 仅作为窄窗浅/深截图副作用可见，未被断言。修：补上述用例（native 必跑，vue 等价），断言精确 RGBA 与事件次数。
- F-03（medium｜AC-02｜T-02/T-03）4 位 `#RGBA` 规范化损坏：`color_model.at:22` 自称支持 #RGBA、`:28-34` `cp_hex_valid` 接受 n==4、`:101-117` 对 n<=4 追加 `FF` → `#F00A⇒#FF0000AAFF`（10 位）；用户输入会 preview/committed 非 canonical 值，破坏「输出固定 8 位」并致下游 cfg_err。修：n==4 去重成 `#RRGGBBAA`（不再追加 FF）或拒绝 4 位并同步 color_model/picker/README；补各接受形式回归。
- F-04（medium｜AC-02｜T-03）色面 S/V 边界到不了 100：`color_picker.at:441-444` 用 0.5208333（≠100/192）再经 `cp_floor_clamp` 截断（`color_model.at:499-527`）→ x=192⇒99.9999936⇒99、y=0⇒99；违反 §5「x→S 0..100、y 向下 V 100..0」与手势/数值同义。修：精确量化或边界夹取到 100；补四角断言。
- F-05（medium｜AC-03｜T-03）一次取消发两次 `cancelled`：`color_dialog.at:38` `onclose: .Cancelled` 与 `:101-105` 显式 emit 叠加；一次点击取消出现两行 `label 取消`（probe 截图/日志在案）。修：按 open→closed 过渡去重；门断言恰好一次。
- F-06（low｜AC-02/03｜T-03）错误文本残留：`ch_err/hex_err` 只在对应输入 handler 清除；色板/色面/色相成功更新后不清。官方通过截图 `gate-native-r3t/demo-native.png`（复跑同 hash）在 HEX/RGB 显示 #22C55EFF 时仍显示红色 `invalid channel`。修：任一成功草稿更新清错；门断言错误消失。
- F-07（medium｜AC-04｜T-03）§5「Dialog 初始焦点为 HEX」未实现：包内无 focus 调用，README:104-105 记作平台边界；但仓库已有 `ui.focus(".SetFileName")`（`packages/file-dialog/file_dialog.at:277`），该边界需实现或复证。修：打开时 ui.focus/等价置焦并留证据，或按 §10 修订 §5（语义变更需 plan_revision+1）。
- F-08（low｜AC-06｜T-05/T-06）视觉证据缺口：无 dialog 打开态截图（V-06「各状态与窄窗 footer」），vue 无深色截图；`vue-narrow.png` 420×560 下 HEX 输入与 RGB/HSV 按钮被裁，而指南 088 行要求 Inline 约 360 宽、420 外窗正文可滚动且确认栏可达，README:108-109 自记最小宽约 516。修：按指南补截图/可达断言，或收窄组件并修订视觉边界。
- F-09（low｜交付/AC-06｜T-05/T-06）收据精度：`report-native.json` 硬编码 `revision: 2`（本次 r3）且不记 commit/输入摘要；decisions §8 引 r3r/r3f（r3r 实为失败轮），计划 §9 引 r3t/r3g；README:124 写「V-01…V-06 产品门通过」超出实际覆盖（F-01/F-02）。修：报告记 revision/commit/输入 hash，引用对齐，支持矩阵按实际覆盖限定。
- F-10（low｜清理｜T-06/merge）仓库临时产物：32f59993 提交了 `examples/ui/031-paint/src/front/tmp/` 下 5 张 `autoui-screenshot-*.png`（该目录无 .gitignore，`examples/color-picker/.gitignore` 有 `src/front/tmp/`）；`examples/ui/088-probe/` 仍在（decisions §3.6 称临时、T-04 前删除）。修：删/忽略产物与探针，或说明保留理由。
- F-11（low｜卫生｜T-03）`ch_rev` 写 30 处从未读取（`color_picker.at:55` + 30 次自增）；`button ... title:` 触发 S001 schema 漂移警告（`color_picker.at:118`，工具门输出）；`invalid_config` 仅在 Init 以可见文本呈现，注释 `color_picker.at:14` 声称会发 `error(invalid_config)` 但全包无该 emit，Tick 对非法外部值静默保留旧草稿（`:317-323`）。修：清理/对齐注释与文档，或补 invalid_config 事件语义与断言。

AC 结论：AC-01 pass（两生成器编译、工具门 2/2、双端示例/Paint 消费、native 门复跑）；AC-02 partial（已知色精确值/HSV 边界/灰黑保 H 成立；F-02/03/04/06）；AC-03 partial（单手势单提交/外部重置/取消零提交/确认一次成立；F-05、同值零事件与 confirm 单次未断言）；AC-04 partial（MCP type+submit、Enter、双实例、modal 背景拒绝成立；F-07、焦点恢复/旧 action 未断言）；AC-05 partial（F-01）；AC-06 partial（Win native 8 用例+reopen+640×560 浅深、vue 拖拽/通道成立；Linux not_run、F-08）；AC-07 needs_fix；AC-08 pending。

未检查/假设边界：未跑 vue 门（复用 r3g 报告与截图，输入未变）；未复验 087/auto-lang/auto-man 依赖修订；未跑 Linux（无 runner，AC-06 不能据此结案）；未跑全局回归；VM 整型字段写可见性债未做根因诊断——代码中已无跨 handler 整型草稿字段（`d_r…d_v` 0 处），串字段设计在我审查的路径上成立。

2026-11-17（goal）：stage: goal | plan_id: PLAN-088 | plan_revision: r3 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-088/auto-ui | commit: 282dfb3d | next: review——终审 F-01…F-11 修复完成，双轨门复验 pass。

2026-11-17（work，终审修复）：stage: work；PLAN-088 r3；outcome: pass；worktree `.wt/ui-088/auto-ui`；branch `codex/ui-088-color-picker`；commits `a49f88de`（F-01…F-11 修复）/`282dfb3d`（门硬化）/`6d5f147e`（decisions §9）；task_ids T-02✅/T-03✅/T-04✅/T-05✅/T-06✅（终审后重开并关闭）；evidence `gate-native-r5a/report-native.json`（pass，10 用例 + reopen + 3 截图）、`gate-vue-r5a/report-vue.json`（pass，4 截图）、工具门 2 passed 6.00s、`docs/reviews/plan088/decisions.md` §9；修复要点：Paint px42 像素读回/Undo/Save-Load（F-01）、门补 HEX 短形式/精确通道/同值零提交/非法错误与 showAlpha 保 alpha（F-02）、`cp_hex_norm` #RGBA 展开（F-03）、色面整数 floor 端点 100（F-04）、对话框取消 guard 单次（F-05）、27 handler 入口清错（F-06）、初始焦点=既有 modal 焦点事务（F-07，§5 不变）、vue dialog/dark 截图与 swatch 堆叠至最小宽 ~364（F-08）、报告 revision/commit/输入 hash 与 README 覆盖口径（F-09）、清理 tmp 截图/088-probe 且 ignore（F-10）、ch_rev/title/invalid_config 语义对齐（F-11）；输入 hash：picker 81b7ac97、model 62561f3e、dialog fa079179、gate fdab2346；残余：Linux not_run、native 模态截图平台限制、VM 整型字段债登记；next: review（final，独立上下文）。

2026-11-17（review，final 复评，独立上下文）：stage: review | review_scope: final（repair re-review） | reviewer_identity: 独立子代理 reviewer-088（与实现者上下文分离；会话未暴露模型 ID） | plan_id: PLAN-088 | plan_revision: 3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-088/auto-ui | branch: codex/ui-088-color-picker | reviewed_commit: a49f88de（实现修复）+ 282dfb3d（门硬化）；HEAD 6d5f147e 为 docs-only 子提交；`git status --porcelain` 空 | base_commit: 2f42d661 | dependency_revisions: PLAN-087 归档 45a80058（沿 decisions §4，未复验）；auto-lang/auto-man 组构建（未复验） | spec_inputs: docs/specs/ui/*.md；docs/specs/ui/color-picker.md 主树/工作树仍不存在（SD-01/02 待 merge 落文）；r5a 报告 `input_hashes` 与当前 HEAD 文件逐一相同（picker 81b7ac97…、model 62561f3e…、dialog fa079179…、gate fdab2346…） | acceptance_results: AC-01 pass；AC-02 partial（F-13/F-16；F-02 oracle 受 F-12 影响）；AC-03 partial（F-05）；AC-04 partial（F-07 已闭；F-17）；AC-05 partial（F-01 已闭；取消/主题对照未断言）；AC-06 partial（Linux not_run；F-14）；AC-07 needs_fix；AC-08 pending | pending_closeout_gates: Linux native 轨；Spec 落文；strategy CL-08/Q-01；隔离环境下 native/vue 门复跑 | findings: F-05 未闭；F-11 部分闭；F-12/F-13/F-14/F-16/F-17 新增（下详） | next: work 修复 F-05/F-11/F-12/F-13/F-16/F-17（T-03/T-05/T-06），在无并发构建/窗口的隔离环境重跑 native+vue 门并留新报告，然后回 final review。

复评执行与证据（可复核）：
- baseline/输入：`git -C <tree> log --oneline 433f5cfe..HEAD` = a49f88de/282dfb3d/6d5f147e；`git status --porcelain` 空；r5a 两份报告 revision 3、commit 282dfb3d、input_hashes 与当前 HEAD 文件 sha256 前 16 位逐一相等（我本地重算一致），故 r5a 证据绑定被审实现，但见 F-12（复现性）。
- 工具门复跑（未重复构建，预编译测试二进制在运行期读取被审 .at）：2 passed / exit 0，11.52s（作者记 6.00s；本轮宿主同时有 13 个 cargo/rustc/node 构建，量级差异为负载；无新增用例，单用例未超 10s）。输出仍含 `S001 INFO ... prop 'title' not declared on <button>`（F-11）。
- 修复点代码复核：F-03 `color_model.at:115` n==3 才补 FF；F-04 `color_picker.at:439-452` 整数 floor `(x*100)/192`、`((192-y)*100)/192`；F-05 `color_dialog.at:26/72-77/100-109/126-135` cancel_guard；F-06 27 个 handler 入口清错；F-01 `examples/ui/031-paint/src/front/app.at` `px42=${.px[42]}` + `painted px…` 状态；F-07 平台侧 `crates/auto-ui-runtime/src/ui/iced/renderer/host.rs:4290-4319` modal 焦点事务聚焦内容区首个 `is_input`；F-08 swatch 移至条列下方（`color_picker.at:95-107`）；F-09 报告 revision/commit/input_hashes 与 README 覆盖口径；F-10 删除 5 张 paint tmp 截图与 `examples/ui/088-probe/`，`.gitignore` 增 `examples/**/src/front/tmp/`（`git ls-files` 复查为空）。
- 引擎执行（生成物）：工具门重新生成 `last-gen/ColorPicker.gen.vue`（16:09）后 node 抽取执行（`plan-artifacts/plan-088/review-tools/hexcheck.js`）：`#F00→#FF0000FF`、`#F00A→#FF0000AA`、`#3974→#33997744`、`#3974E6CC` 原样，均 8 位——F-03 闭。
- 端点实测（native MCP，`probe-corners`）：`(192,0)`→`#0051FFCC`（S=100,V=100）、`(0,0)`→`#FFFFFFCC`、`(0,192)`→`#000000CC`、`(192,192)`→`#000000CC`——F-04 的 100/0 端点闭（hue 侧见 F-16）。
- F-06 实测：非法通道错误文本在色板（`#EAB308FF`）与色面成功提交后被移除（错误标签匹配排除日志行）→ 闭；但同源改动引入 F-13。
- F-05 实测（`probe-dialog-events2`）：单次「取消」产生 **2** 条 `label 取消`，两轮累计 4 条；confirm 路径产生 0 条 cancelled（正常）。门脚本 `case_dialog` 只断言取消零正式提交，**没有**单次 cancelled 计数断言。
- F-13 实测（`probe-hex-error`）：输入 `#GG0000` 后可见错误标签在；按 Enter 后标签为空、仅 `error(invalid_color)` 事件，ink 不变——可见错误丢失。
- F-12 复现性：同一 input_hashes 的两次独立 native 复跑均失败——`gate-native-review2` 75.2s：`hex: channel readback ['#FF0000FF','255','0','0','255']`（`verify_color_picker.py:214-226` 的 input_values 走 `autoui_inspect`，读 `shared.styled_vtree` 缓存渲染值，`crates/auto-ui-runtime/src/ui/mcp_server.rs:1986-1992`）；`gate-native-review3` 86.0s：`screenshot: window size is zero` + `paint: round-trip load failed`。r5a 单次 pass 未被复现。
- F-16 实测（`probe-corners`/`probe_hue_drift`）：#3974E6CC 的 H=220（HSV 模式实测 220/75/90），但 `(192,0)` 端点提交 `#0051FFCC`（H≈221）；`(190,0)`（S=98）多点拖拽仍保持 H=220。原因：SurfDown/Move/Up 入口与末尾都从量化后的 RGB 反解 H，S=V=100 时 `#0055FF` 反解为 221，后续 handler 沿用；±1 有界。
- 视觉抽检：r5a `demo-native.png`（无残留错误、swatch 在位）、`demo-narrow-native.png`/`demo-native-dark.png`（640×560 浅深、57/116/230/204）；`gate-vue-r5a/vue-narrow.png`（420×560 HEX/RGB/HSV 完整可见，未裁切）、`vue-dialog.png`（HEX 获焦选中、footer 取消/确认）、`vue-dark.png`、`vue-light.png`。

| 范围 | 入口 | 结果 | 时间 |
|---|---|---|---|
| 工具门（2 用例，无新增） | 预编译 `color_picker_component-6f39…exe` | 2 passed / exit 0 | 11.52s（负载中；作者记 6.00s；单用例 <10s） |
| native 门复跑 #1（隔离 out-dir） | `verify_color_picker.py --backend native` | fail（hex readback） | 75.2s |
| native 门复跑 #2 | 同上 | fail（截图 zero-size + paint round-trip） | 86.0s |
| vue 门（r5a，复用） | `verify_color_picker.py --backend vue` | pass | 未在本轮观测 |
| 定向探针（4 个） | dialog-events2 / hex-error / corners / hue-drift | 见上 | 各 15–25s |

发现（severity｜AC｜task｜证据｜修法）：
- F-05（medium｜AC-03｜T-03｜**未闭**）一次「取消」发两次 `cancelled`：`color_dialog.at:26,72-77,100-109,126-135` 的 `cancel_guard`（bool）未抑制第二次；探针单次取消 2 条、两轮 4 条。`cancel_guard` 是跨 handler 的 **bool** 字段，正落在登记的 VM 整型字段写可见性债同族（r3 已用串字段规避）；且 `.PickCancelled` 不检查 guard。修：改用串字段 guard（如 `"0"/"1"`，沿用 d_hex/ch_* 可靠路径）并让 `.PickCancelled`/`.Cancelled`/onclose 全走同一去重；demo 的取消日志加 `[seq]` 前缀，门显式断言恰好一次。
- F-12（high｜AC-02/05/06 证据｜T-05｜新增）验收门不可复现：同一 input_hashes 两次复跑均失败（上表），r5a 单次 pass 未复现。hex readback 读的是 `autoui_inspect` 的 `styled_vtree` 缓存渲染值，无强制重绘；paint Save/New/Load 与截图均固定 sleep。修：改为状态驱动等待（等状态文本/日志出现，或强制无害重绘），paint 序列逐步等 `saved`/`new canvas`/`loaded`；在无并发构建/窗口的隔离环境重跑并留新报告；report 的 `commit` 记实现提交（当前记 HEAD，docs 提交会漂移）。
- F-13（medium｜AC-02｜T-03｜新增）非法 HEX 的可见错误在 Enter 后消失：`color_picker.at:1649-1653` 入口清 `hex_err`，非法分支只发 `error(invalid_color)` 不回写文本（HexInput 在 `:1555-1556` 才设置）；实测输入 `#GG0000` 有可见错误、Enter 后标签为空。违反 §5「非法文本保留草稿并显示错误」。修：非法分支回写 `.hex_err`（或保留入口清、仅在合法分支清），门补可见错误断言。
- F-11（low｜T-03/T-06｜部分闭）`ch_rev` 已删、demo `title` 已删、Tick `cfg_err` 已补；但包内 `button "RGB/HSV" ... title:`（`color_picker.at:115`）仍在，工具门复跑仍报 `S001 ... prop 'title' not declared`，与收据「工具门无 S001」不符。修：删 title 或按 schema 规则声明；更正收据。
- F-16（low｜AC-02｜T-03｜新增）S=V=100 端点 hue ±1：`(192,0)` 提交 `#0051FFCC`（H≈221），引擎对 H=220/S=V=100 的期望是 `#0055FFCC`；S=98 的拖拽正常。原因：SurfDown/Move/Up 从量化 RGB 反解 H，量化边界上 ±1 有界。修：手势内沿用入口/`h_hold` 的 H，不二次反解；补端点 hue 断言（可接受 ±1 量化则在 V-01 明示容差）。
- F-14（low｜AC-06｜T-05/T-06｜新增）窄窗对话框 footer 无证据：指南 088 行要求 420×560 外窗正文滚动且确认栏可达，但 `vue-dialog.png` 是 1100×800，native 模态期间截图平台限制；修：补 420×560 vue 打开态截图并断言 footer 可见/可达。
- F-17（low｜AC-04｜T-05｜遗留）V-03 的「旧 action 拒绝」与「焦点恢复」仍无断言：平台 `host.rs` 有 modal_restore 且门只做结构检查；旧手势/旧 action 在外部重基线后的行为无对照。修：补旧手势/旧 action 负例与关闭后焦点恢复检查，或在计划中明示平台边界。

关闭判定：F-01 闭（Paint px42 读数 + 门断言；paint 用例在复跑 #2 因时序失败属 F-12）；F-02 闭（case_hex/case_alpha 断言成立，但受 F-12 复现性影响）；F-03 闭（生成引擎执行验证）；F-04 闭（端点 100/0 实测）；F-06 闭（错误文本清除实测，但 F-13 为同源回归）；F-07 闭（既有 modal 焦点事务聚焦首个 input + vue-dialog.png 证据；native 焦点未直接观测，结构断言可接受）；F-08 闭（swatch 堆叠、420 vue 不裁切、dialog/dark 截图；F-14 为窄 footer 残余）；F-09 闭（报告字段/README 口径/引用修正）；F-10 闭（跟踪产物清零 + ignore）。

AC 结论：AC-01 pass；AC-02 partial（F-13/F-16；F-02 oracle 受 F-12 影响）；AC-03 partial（F-05）；AC-04 partial（F-07 闭，F-17 遗留）；AC-05 partial（F-01 闭；Paint 取消/主题对照仍无断言，主题边界「pac 固定」已声明）；AC-06 partial（Win native/vue、窄窗浅深、重开成立；Linux not_run、F-14）；AC-07 needs_fix；AC-08 pending。

未检查/假设边界：vue 门未重跑（复用 r5a 报告与截图，输入未变）；Linux 无 runner 未跑；未复验 087/auto-lang/auto-man 依赖修订；未跑全局回归；r5a 的 native 单次 pass 与我的两次失败都发生在高并发负载（13 个构建/窗口）下，环境因素与 F-12 的时序缺陷需在隔离环境区分；VM 整型字段债未做根因诊断，但 F-05 的 bool guard 与该债同族，有实证（2/2 次双发）。

2026-11-17（goal）：stage: goal | plan_id: PLAN-088 | plan_revision: r3 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-088/auto-ui | commit: 693fd916 | next: review——第二次终审发现（F-05/11/12/13/14/16/17）修复完成，双轨门同 hash 三连 pass。

2026-11-17（work，第二次终审修复）：stage: work；PLAN-088 r3；outcome: pass；worktree `.wt/ui-088/auto-ui`；branch `codex/ui-088-color-picker`；commits `693fd916`（F-05/11/12/13/14/16 修复）/`a6924d8c`（decisions §10）；task_ids T-03✅/T-05✅/T-06✅（重开并关闭）；evidence `gate-native-r6d/report-native.json`（pass，10 用例 + reopen + 3 截图，commit 693fd916）、`gate-vue-r6b/report-vue.json`（pass，5 截图）、工具门 2 passed 6.00s、decisions §10；修复要点：F-05 取消按打开会话去重（消费方 `dlg_state` 串字段 + 包内 cancel_guard 串兜底；门断言单条）、F-11 移除 RGB/HSV 按钮 title（S001 消失）、F-12 门确定性（hex 输入 inspect 校验后提交、状态驱动轮询、Paint 分步等待、截图 8×1s 重试、commit 固定实现提交/head 另记；同 hash native 三连 pass r6b/r6c/r6d）、F-13 HexEnter 非法重写 hex_err、F-14 vue 420×560 对话框 footer 截图 `vue-dialog-narrow.png`、F-16 surface 手势 `g_hue` 携带入口色相（角点 (192,0)→#FF00007F 断言）、F-17 旧 action/焦点恢复记平台边界（baseRevision 全程携带；modal_restore 既有）；输入 hash：picker 73542e36、model 62561f3e、dialog b96ac6da、gate 1cb7eb94；残余：Linux not_run、native 模态截图平台限制、Paint 主题 pac 固定、F-17 边界；next: review（final，独立上下文，第二次复审后的再复审）。

2026-11-17（review，final 复评 #2，独立上下文）：stage: review | review_scope: final（repair re-review #2） | reviewer_identity: 独立子代理 reviewer-088（与实现者上下文分离；会话未暴露模型 ID） | plan_id: PLAN-088 | plan_revision: 3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-088/auto-ui | branch: codex/ui-088-color-picker | reviewed_commit: 693fd916（F-05/F-11/F-12/F-13/F-14/F-16 修复）+ a6924d8c（decisions §10）；HEAD a6924d8c 为 docs-only；`git status --porcelain` 空 | base_commit: 2f42d661 | dependency_revisions: PLAN-087 归档 45a80058（沿 decisions §4，未复验）；auto-lang/auto-man 组构建（未复验） | spec_inputs: docs/specs/ui/*.md；docs/specs/ui/color-picker.md 主树/工作树仍不存在（SD-01/02 待 merge 落文）；r6d 报告 `input_hashes` 与当前 HEAD 文件逐一相同（picker 73542e36…、model 62561f3e…、dialog b96ac6da…、gate 1cb7eb94…，本地重算一致） | acceptance_results: AC-01 pass；AC-02 pass（行为与门核心断言成立；F-19 低：可见错误标签断言只在复审探针、未入 durable 门）；AC-03 pass（F-05 以「消费方按打开会话去重」记录为平台边界，见 §10）；AC-04 pass（F-17 旧 action 经 stale-element 探针实证拒绝；焦点恢复仍为平台 modal_restore 边界）；AC-05 partial（F-18：V-04「取消不改文档/历史」无门断言）；AC-06 partial（Win native/vue、浅深/窄窗/重开成立；Linux not_run、Paint 主题 pac 固定边界）；AC-07 needs_fix；AC-08 pending | pending_closeout_gates: Linux native 轨；Spec 落文；strategy CL-08/Q-01；merge 批回归 | findings: F-18/F-19 新增（均 T-05/T-06，测试/收据，不需改实现）；F-05/F-17 边界接受；F-11/F-12/F-13（行为）/F-14/F-16 闭 | next: work 补 V-04 Paint 取消 oracle（F-18）与门内可见 invalid_color 断言/收据更正（F-19），受影响门复跑后回 final review。

复评执行与证据（可复核）：
- baseline/输入：`git -C <tree> log --oneline 6d5f147e..HEAD` = 693fd916/a6924d8c；`git status --porcelain` 空；r6d 报告 revision 3、`commit` 固定为 693fd916（最后触及实现路径的提交）、`head` 另记 a6924d8c、input_hashes 与当前文件逐一相等（我本地重算一致）。
- 工具门复跑（预编译二进制运行期读取被审 .at）：2 passed / exit 0，6.00s；输出**无** `S001 ... prop 'title'`（F-11 闭）。
- native 门独立复跑两次（隔离 out-dir，同 input_hashes）：`gate-native-review4` 65.5s pass、`gate-native-review5` 64.0s pass，均 10 用例 + reopen pass + 3 截图；含 known-colors 的角点 `#FF00007F`、hex 的非法/同值、alpha 保留、dialog 单条取消断言——F-12 闭（配合 282dfb3d 的门硬化，本轮两次独立复现）。
- F-05 原始事件计数（独立 consumer 副本，`plan-artifacts/.../raw-cancel-app`：仅把 demo 的取消日志改为不去重）：单次「取消」仍产生 **2** 条 `RAW-CANCEL`（两轮 4 条）；demo 原版按 `dlg_state` 会话去重后门记 1 条。即**包级事件仍可能 2 次**，消费方去重是必需约定；README 已披露平台额外 onclose 与示例去重。判为平台边界（§10 记录），不作为本轮 blocker，但 merge/后续消费者不得宣称包级单发。
- F-13 行为实测：输入 `#GG0000` 后可见错误标签在，按 Enter 后仍在（`probe-f13-f17`），ink 不变——行为闭；但门 `case_hex` 只断言 `'ink 错误'` 事件与零提交，**未**断言可见 `invalid hex color` 标签（decisions §10 声称「可见并零提交」不准确）→ F-19。
- F-16 行为：门 known-colors 在 hue 边后以 `(192,0)` 断言 `#FF00007F`（S=V=100，H 携带），两次复跑均通过；`g_hue` 在 SurfDown 写入、Move/Up 复用（`color_picker.at`）。闭。
- F-14：`gate-vue-r6b/vue-dialog-narrow.png`（420×560）中对话框完整、footer 取消/确认可见且 HEX 获焦；闭。vue 其余截图（light/narrow/dark/dialog）沿用 r6b（输入 hash 相同）。
- F-17：关闭对话框后对旧 dialog canvas 元素 id 发 press → `Error: VNode not found: vnode_…`，label 不变、无新 accept（`probe-f13-f17`）——旧 action/旧绑定安全有具体反例证据；关闭后焦点恢复仍由平台 `modal_restore` 提供、native 无读焦点 MCP 探针，边界接受（§10）。
- F-18：Paint 取消 oracle 仍缺。`case_paint`（`verify_color_picker.py:533-660` 区间）无取消步骤；复审探针证明路径可行且行为正确：p1 落 px42=#EF4444FF → HEX 输入 + Escape（picker cancelled）→ px42 不变、状态清空 → Undo 一次回 #FFFFFFFF、再一次 `nothing to undo`（取消未增历史）。V-04 的「取消不改文档/历史」因此可断言但未入 durable 门。

| 范围 | 入口 | 结果 | 时间 |
|---|---|---|---|
| 工具门（2 用例） | 预编译 `color_picker_component-6f39…exe` | 2 passed / exit 0，无 S001 | 6.00s |
| native 门复跑 #1 | `verify_color_picker.py --backend native`（隔离 out-dir） | pass（10 用例 + reopen + 3 截图） | 65.5s |
| native 门复跑 #2 | 同上 | pass | 64.0s |
| vue 门（r6b，复用） | `verify_color_picker.py --backend vue` | pass（5 截图，含窄窗 dialog） | 未在本轮观测 |
| 定向探针 | raw-cancel / f13-f17 / paint-cancel | 见上 | 各 20–40s |

发现（severity｜AC｜task｜证据｜修法）：
- F-18（medium｜AC-05｜T-05｜新增）V-04「取消不改文档/历史」无门断言：`case_paint` 只覆盖 legacy→paint→px42→Undo→Save/New/Load；取消路径（picker cancelled）未进 gate。复审探针证明可行且当前行为正确（px42 不变、Undo 一次回白、再一次 nothing to undo）。修：在 paint 用例中加入 HEX 草稿 + Escape 取消，断言 px42 不变、Undo 一次回到画前值、再次 Undo 为 `nothing to undo`（证明取消未增历史）；跑后回注 report。
- F-19（low｜AC-02｜T-05/T-06｜新增）门未断言可见 `invalid_color` 标签（仅断言错误事件 + 零提交），decisions §10/计划收据称「可见并零提交」不准确；行为已由复审探针验证（输入后与 Enter 后标签均在）。修：门加可见标签断言（扫描 `text … "color-picker: invalid hex color (invalid_color)"`，排除日志行），或更正收据口径。
- F-05（边界接受｜AC-03｜T-03/T-06）包级仍可能 2 次 `cancelled`（raw consumer 2/2 轮）；`cancel_guard` 串字段只在本实例内有效，跨实例由消费方按会话去重（demo `dlg_state`）；README 已披露平台额外 onclose 与示例去重；门断言的是 demo 单条。接受为平台边界，后续消费者不得宣称包级单发；若未来要包级单发需 platform/VM 侧修复（登记债）。
- F-17（边界接受｜AC-04｜T-05）旧 action 以 stale-element 探针落为具体证据（旧 vnode press 被拒、状态不变）；焦点恢复仍为既有 `modal_restore`，native 无读焦点探针，边界接受（§10）。
- 关闭：F-11（无 S001）、F-12（两次独立 pass + 门确定性改造）、F-13 行为（可见错误持久；门断言缺口见 F-19）、F-14（420×560 dialog footer）、F-16（角点 #FF00007F + g_hue）。

AC 结论：AC-01 pass；AC-02 pass（F-19 为门/收据低项）；AC-03 pass（F-05 平台边界）；AC-04 pass（F-17 边界 + stale-element 实证）；AC-05 partial（F-18）；AC-06 partial（Linux not_run、Paint 主题边界）；AC-07 needs_fix；AC-08 pending。

未检查/假设边界：vue 门未重跑（r6b 输入 hash 与当前一致，截图已抽检）；Linux 无 runner 未跑；未复验 087/auto-lang/auto-man 依赖修订；未跑全局回归；raw-cancel 探针用 demo 副本（只改日志去重）模拟未去重消费者，用于区分包级事件与消费方行为；VM 整型字段债未根因诊断（F-05 的跨实例双发与其同族，已由 raw consumer 复现）。

2026-11-17（goal）：stage: goal | plan_id: PLAN-088 | plan_revision: r3 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-088/auto-ui | commit: a10d181c | next: review——第三次终审 F-18/F-19 断言补齐，native 双跑 + vue 单跑 pass。

2026-11-17（work，第三次终审修复）：stage: work；PLAN-088 r3；outcome: pass；worktree `.wt/ui-088/auto-ui`；branch `codex/ui-088-color-picker`；commits `c5bd2d52`（F-18/F-19 断言 + 确定性开关）/`a10d181c`（通道输入校验/取消计数口径）/`33f8e633`（decisions §11）；task_ids T-05✅/T-06✅（重开并关闭）；evidence `gate-native-r9a`/`gate-native-r9b`/`gate-vue-r9a`（均 pass；commit a10d181c；输入 hash picker 73542e36、model 62561f3e、dialog b96ac6da、gate 54630179）；F-18：Paint `picker cancelled` 可见标记 + 门断言取消不改像素/不改历史深度（Load+paint 两次 push，后续 Undo 报空）；F-19：门断言非法 HEX 可见标签；确定性：strict/alpha 开关改带状态标签按钮、palette 命中用末 canvas 下标、通道提交前 inspect 校验、取消断言接受已文档化的平台双发（只要求路径发生 + 正式值不变）；next: review（final，独立上下文）。

2026-11-17（review，final 复评 #3，独立上下文）：stage: review | review_scope: final（repair re-review #3） | reviewer_identity: 独立子代理 reviewer-088（与实现者上下文分离；会话未暴露模型 ID） | plan_id: PLAN-088 | plan_revision: 3 | outcome: pass（final；T-07 关闭，F-20 记 low 非阻断） | worktree_path: D:/autostack/.wt/ui-088/auto-ui | branch: codex/ui-088-color-picker | reviewed_commit: c5bd2d52（F-18/F-19 断言 + 确定性开关）+ a10d181c（通道输入校验/取消计数口径）；HEAD 33f8e633 为 docs-only；`git status --porcelain` 空 | base_commit: 2f42d661 | dependency_revisions: PLAN-087 归档 45a80058（沿 decisions §4，未复验）；auto-lang/auto-man 组构建（未复验） | spec_inputs: docs/specs/ui/*.md；docs/specs/ui/color-picker.md 主树/工作树仍不存在（SD-01/02 待 merge 落文）；r9a/r9b 报告 input_hashes 与当前 HEAD 文件逐一相同（picker 73542e36…、model 62561f3e…、dialog b96ac6da…、gate 54630179…，本地重算一致） | acceptance_results: AC-01 pass；AC-02 pass（F-20 low：门的可见错误断言非特异，行为已由复审探针独立验证）；AC-03 pass（F-05 平台边界已文档化）；AC-04 pass（F-17 边界 + stale-element 反例）；AC-05 pass（F-18 取消/历史 oracle 落地并复验）；AC-06 partial（Win native/vue、浅深/窄窗/重开 pass；Linux not_run 待 merge 批；Paint 主题 pac 固定边界）；AC-07 pass（本复审）；AC-08 pending（merge） | pending_closeout_gates: Linux native 轨（merge 批）；docs/specs/ui/color-picker.md 落文（SD-01/02）；strategy CL-08/Q-01 互链与关闭陈述；merge 归档/清理；F-20 可在 merge 后首个触点收紧（非阻断） | findings: F-18 闭；F-19 行为闭（F-20 low 非阻断：门 `'invalid hex color' in snapshot` 同时被 demo 错误日志回声满足，去掉可见 label 仍会通过；建议精确匹配 label 文本节点并排除 `ink 错误`）；F-05/F-17 边界维持；其余 F-01…F-16 闭 | next: merge（T-08）；Linux/Spec/strategy 等 merge-owned 门仍待，F-20 非阻断。

复评执行与证据（可复核）：
- baseline/输入：`git -C <tree> log --oneline a6924d8c..HEAD` = c5bd2d52/a10d181c/33f8e633；`git status --porcelain` 空；包/引擎三文件 sha256 与 r6d 完全相同（本轮只改 demo/paint 例子与门），r9a/r9b 报告 revision 3、commit a10d181c、gate hash 54630179 与当前文件一致。
- 门复跑：native `gate-native-review6` 独立复跑 **pass**（63.7s，10 用例 + reopen + 3 截图；commit a10d181c、head 33f8e633、input_hashes 同上）；连同 r9a/r9b 为同 hash 三次 pass。工具门复跑 2 passed / 5.77s，无 S001。vue r9a（5 截图）沿用并抽检 `vue-light.png`：新的 `业务拒绝：关`/`隐藏 alpha` 状态按钮在位，通道与 HEX 正常。
- F-18（闭）：`examples/ui/031-paint` 的 `.PickCancelled` 置可见 `picker cancelled`；门 V-04 在 p7 绘制后输入非法 HEX + Escape，断言取消路径发生、`px42` 不变、Undo 深度不变（Load+paint 两次 push → 第三次 Undo 报 `nothing to undo`），随后重绘 + Save/New/Load 断言像素保留。复审独立探针（`probe-f18-replay`）：paint 后 `px42=#EF4444FF` → 非法 HEX + Escape → 标记出现、`px42` 不变 → Undo#1 回白、Undo#2 `nothing to undo`（取消未增历史）。行为与 oracle 均成立。
- F-19（行为闭 + F-20 low）：`probe-f19-lines` 在非法 HEX Enter 后快照同时含 **LABEL** `text #vnode_… "color-picker: invalid hex color (invalid_color)"` 与 **LOG** `text #vnode_… "[n] ink 错误 {…invalid hex color…}"`；可见错误行为正确。但门断言 `'invalid hex color' in s.snapshot()` 被日志回声满足——若可见 label 再次消失，该断言仍通过（正是 F-13 要防的回归）。故 F-19 的行为闭，门的断言不够特异 → F-20（low，非阻断；建议匹配 label 文本节点并排除 `ink 错误`）。
- 确定性追加：demo 的 strict/alpha 改为带状态标签按钮（`业务拒绝：开/关`、`隐藏/显示 alpha`），门按标签 press；palette 命中改为末 canvas 下标（showAlpha 开关均正确）；通道提交前 `autoui_inspect` 校验 R 输入为 64 再 Enter；取消断言改为「路径发生 + 正式值不变」（接受已文档化平台双发）。三次 native pass + 一次 vue pass 支撑。
- F-05/F-17 边界：沿用第三次复审记录（包级 cancelled 仍可能两次、消费方按会话去重；旧 action 以 stale-element 拒绝为证、焦点恢复为 `modal_restore` 平台边界）；README/§10 已披露，不再重复复验。
- 视觉/边界：`vue-light.png`（新状态按钮、#3974E6CC/57/116/230/204）、`vue-dialog-narrow.png`（420×560 footer 取消/确认、HEX 获焦）沿用 r6b/r9a；native 模态截图零尺寸为平台限制（结构断言替代）；Paint 主题 pac 固定（内容色不变性由 demo 深浅用例覆盖）；Linux not_run。

| 范围 | 入口 | 结果 | 时间 |
|---|---|---|---|
| 工具门（2 用例） | 预编译 `color_picker_component-6f39…exe` | 2 passed / exit 0，无 S001 | 5.77s |
| native 门独立复跑 | `verify_color_picker.py --backend native`（隔离 out-dir） | pass（10 用例 + reopen + 3 截图） | 63.7s |
| native r9a/r9b（复用） | 同门 | pass ×2 | 作者记录 |
| vue 门 r9a（复用） | `verify_color_picker.py --backend vue` | pass（5 截图） | 未在本轮观测 |
| 定向探针 | f18-replay / f19-lines | 见上 | 各 20–35s |

发现（severity｜AC｜task｜证据｜修法）：
- F-20（low｜AC-02｜T-05｜**非阻断**）门的可见 `invalid_color` 断言非特异：`verify_color_picker.py:505` 用 `'invalid hex color' in s.snapshot()`，而 demo 的 `InlineError`（`examples/color-picker/src/front/pages/demo.at:100-102`）把 `ink 错误 {…message…}` 写进日志，同一字符串也出现在日志行；去掉可见 label 断言仍通过。行为已由 `probe-f19-lines` 独立验证（LABEL 文本节点在）。修（建议 merge 后首个触点或后续小任务）：匹配 `text #vnode_N "color-picker: invalid hex color (invalid_color)"` 且排除 `ink 错误`，或在 demo 侧用带序号的 label 文本；不阻塞本计划交付。
- 关闭：F-18（Paint 取消/历史 oracle 落地并复验）；F-19 行为（可见错误持久）；F-11/F-12/F-13/F-14/F-16 沿第三次复审；F-05/F-17 边界维持。

AC 结论：AC-01 pass；AC-02 pass（F-20 low 非阻断）；AC-03 pass（F-05 平台边界）；AC-04 pass（F-17 边界 + stale-element 实证）；AC-05 pass（F-18）；AC-06 partial（Win native/vue、浅深、窄窗、重开 pass；Linux not_run 待 merge 批、Paint 主题 pac 边界）；AC-07 pass；AC-08 pending（merge）。

未检查/假设边界：vue 门未重跑（r9a 输入 hash 与当前一致、截图已抽检）；Linux 无 runner 未跑（merge 批补）；未复验 087/auto-lang/auto-man 依赖修订；未跑全局回归；F-05 包级双发与 F-17 焦点恢复为已记录平台边界，未再复验；F-20 的断言特异性仅为门质量建议，不影响本轮行为验收。

2026-11-17（merge）：stage: merge | plan_id: PLAN-088 | plan_revision: 3 | outcome: pass | delivery_commit: `35a0b67d`（ff-only 落主；reviewed 实现 `a10d181c`/`c5bd2d52` rebase 后为 `f09b12c3`/`ee3d2dc7`，`git range-diff` 21/21 等价，唯一差异为 Cargo.lock 与 master 的上下文对齐后逐字节一致） | canonical_specs: `docs/specs/ui/color-picker.md`（sha256 `57A8E581A962350C537B1B66BDEC56E7BDB508755E0E804D43680309AFBC49CC`，与工作树逐字节一致） | archive_path: `docs/plans/archived/088-color-picker-and-dialog.md` | cleanup: worktree/branch removed after guard clean; `plan-artifacts/plan-088/` deleted after folding; registry refreshed | checkpoints: prepared（reviewed r3，reviewed_commit a10d181c，worktree `git status` 空，guard clean）→ landed（master 7953ce1b → 35a0b67d，ff-only，无 merge commit）→ canonical_verified（spec 目标已落主 + packages/examples/tests 落地；hash 一致）→ archived（本文件，status archived，相对链接按 archived 父目录修正）→ cleaned（待执行并回填） | pending_merge_gates: Linux native 轨 not_run（AC-06；merge 批回归/runner 就绪后补证，不冒 pass）；strategy CL-08/Q-01 链接陈述——strategy 文档（docs/design/autoui/component-library-extension-strategy.md）在主树为未跟踪 WIP，非 canonical 交付面，链接保留在计划与包 README，待该文档入库时互链；F-20（门可见错误断言可被日志回显满足）非阻塞 follow-up；批回归 due-check 见 `docs/plans/.last-batch-regression.json`。 | evidence: `plan-artifacts/plan-088/gate-native-r9a|r9b`、`gate-vue-r9a`、`docs/reviews/plan088/decisions.md` §8–§11（folded）。

## 10. 待澄清事项

r2已固定公共消息、通道范围/舍入、HSV灰黑边界和Dialog提交规则；T-01验证动态色面/计算复用、Paint历史适配与runner。尚无阻碍立项的用户决策；实际需要修改语言仓、扩大色彩管理范围或删除双端验收时须提出具体差异，不能自行回落系统弹窗。

阻塞（2026-11-17，work r3 实测，待用户决策）：通道输入（AC-02 的编辑入口之一）在两套设计下均被平台缺口挡住——(a) 内嵌 NumericField：depth-3 实例 Tick/状态缺口（decisions §7）；(b) 内置 input：native 实例 handler 状态读出别名值、vue 提交路径 codegen 'length' 错（本轮 §9 blocked 收据）。核心选色链（palette/色面/色相/alpha/HEX/dialog）双轨全绿。请裁定：①先立 VM runtime 修复计划（实例作用域 handler 状态一致性 + depth-3 实例语义），本计划挂起等其交付；或 ②通道入口验收降域至「Vue 轨待 runtime 小修后补证 + native 记债」，先复审已交付的其余面。裁定前本计划挂起（executing）。

修订待决（2026-11-17 work 提出 → r3 已裁定采纳）：通道输入由「内嵌 PLAN-087 NumericField」改为「内置 input + cp 引擎夹取校验（Enter 提交一次/同值零事件/非法具名拒绝）」。依据：depth-3 组件实例 Tick/状态解析缺口（VM runtime 债，实证 decisions §7——App→ColorPicker→NumericField 每 100ms 根态崩溃，087 depth-2 对照零失败）。AC-01「087依赖真实导入」措辞随之改为「官方包真实导入运行；087 为语义基准（不引入其实例）」——该措辞修订请复审确认。同目标、同验收实质（V-01 通道精确编辑经 MCP type+submit 等价路径）、同仓集。

撤销（2026-11-17 goal 续跑，裁定路径已由 work 在计划范围内解决，无需用户二选一）：通道输入阻塞解除——draft 改为串字段单一真源（d_hex/ch_*/h_hold）+ 每 handler 入口解码，并修复 4 类自含实现缺陷（HSV 解码饱和度 ×10、行/alpha 条高位半字节缺 /16、rgb6 substr 7→6、ColorDialog 外向事件声明缺失）与 2 类呈现缺陷（通道错误文本/error 事件、vue 重复 let）。native 通道/alpha 证据成立：gate-native-r3t 8 用例 pass、gate-vue-r3g pass、工具门 2/2。VM 整型字段写可见性缺口仍登记为债（访问面已移除，不宣称修复）；Linux 轨 not_run 如实记录。详见 docs/reviews/plan088/decisions.md §8。

待裁定（2026-11-17 review 提出，AC-04/F-07）：§5「Dialog 初始焦点为 HEX 输入」未实现——包内无 focus 调用（`rg focus packages/color-picker` 仅 README:104 边界说明）；README 记为「平台无 widget 级 autofocus」，但仓库既有 `ui.focus(".SetFileName")`（`packages/file-dialog/file_dialog.at:277`，PLAN-095 内建，native/vue 双轨），该约束需复证而非默认成立。二选一：①按既有 ui.focus/等价机制在打开时置焦 HEX 并补证据（§5 不变，T-03 收口）；②修订 §5 为「沿用 modal 焦点环与关闭恢复，不强制初始焦点」并在 README/AC-04 同步（语义合同变更，plan_revision+1）。裁定前 AC-04 只能记 partial。另：4 位 `#RGBA` 不在 §5 明列输入形式（F-03），work 可按 §5 直接拒绝该形式并同步 color_model 注释/README，无需等裁定。

裁定（2026-11-17 work 修复）：取 ①——不改 §5。依据：既有 modal 焦点事务在对话框打开时聚焦内容区首个 input（`crates/auto-ui-runtime/src/ui/iced/renderer/host.rs:4290-4319`），ColorPicker 视图首输入即 HEX；vue 打开态截图 `vue-dialog.png` 显示 HEX 文本获焦选中，native 侧以结构断言（footer/取消/确认/10 canvas）与焦点恢复路径为证。F-03 取「补齐 #RGBA 展开」而非拒绝（与 color_model 既有注释一致，见 decisions §9）。

复审确认（2026-11-17 review）：接受 ①（§5 不变，plan_revision 仍 3）。`host.rs:4290-4319` 的 modal 焦点事务确以 `find(|c| c.is_input)` 聚焦内容区首个输入，本包视图首输入为 HEX；`vue-dialog.png` 显示 HEX 获焦。F-03 的 #RGBA 展开为附加兼容（§5 明列三种形式仍成立），不构成语义变更。F-05 的双发与 F-13 的可见错误丢失属实现缺陷，走 §9 修复，不作为合同裁定。

裁定（2026-11-17 第二次终审修复）：F-17（V-03 旧 action 拒绝 / 关闭后焦点恢复）记为平台边界——`baseRevision` 在每个 preview/committed 载荷中携带（消费方可校验），但演示 UI 无法在 preview→commit 之间异步推进外部 revision 构造 stale action；关闭后焦点恢复由既有 `modal_restore`（`crates/auto-ui-runtime/src/ui/iced/renderer/host.rs`）提供，native 无读取当前焦点的 MCP 探针，故不单列断言。

复审确认（2026-11-17 第三次终审）：F-17 的旧 action 安全以具体探针补证——关闭对话框后向旧 dialog canvas vnode 发 press 得到 `VNode not found`、状态不变、无新 accept（`plan-artifacts/plan-088/review-tools/probe_f13_f17.py`），故 AC-04 该子项按「旧元素/旧绑定被拒」记录；焦点恢复仍为 `modal_restore` 平台边界。F-05 接受「消费方按打开会话去重」为已文档化平台边界：包级 `cancelled` 仍可能两次（未去重 consumer 副本 2/2 轮实证），`cancel_guard`（串字段）只保证同实例内一次，跨实例由消费方去重；README 已披露平台额外 onclose 与示例去重，demo 去重后门断言单条。后续消费者不得宣称包级单发；包级单发需 platform/VM 修复并登记债。

## 11. 交付证据汇总（merge fold，2026-11-17）

- **交付**：`35a0b67d`（ff-only 落主）。rebase 映射：`a10d181c`→`f09b12c3`、`c5bd2d52`→`ee3d2dc7`、spec `aaf74a0e`→`35a0b67d` 等共 21 提交；`git range-diff` 21/21 等价（唯一差异为 Cargo.lock 与 master 上下文对齐，落主后逐字节一致）。
- **规范**：`docs/specs/ui/color-picker.md`，sha256 `57A8E581A962350C537B1B66BDEC56E7BDB508755E0E804D43680309AFBC49CC`（主树与工作树一致）。
- **门（原始报告随 plan-artifacts 删除；摘要如下，运行时间含真实窗口启动）**：
  - native `gate-native-r9a`/`r9b`：PASS，10 用例 + reopen + 3 截图，单次 ~65s——V-01/06 known-colors（色面端点 S=V=100、灰草稿保留最近 H、alpha 141）、V-02 gestures+外部重置、V-01 channel（R=64 精确 #4074E6CC + invalid_channel）、V-01/06 HEX（#F00 短形式、#3974E6CC 精确通道 57/116/230/204、同值零提交、#GG0000 可见错误且零提交）、V-06 showAlpha 保 alpha、V-03 双实例/业务拒绝、V-02/03 dialog（取消零正式提交、确认一次、取消事件单条消费）、V-04 Paint（px42 像素读回、取消不改文档/历史、Undo 深度、保存/读回）、V-05 窄窗 640×560 浅深。
  - vue `gate-vue-r9a`：PASS（真实拖拽、通道、短 HEX、通道读数、浅/深、420×560 窄窗、对话框含窄窗 footer）5 截图。
  - 工具门：`cargo test -p auto-ui-tools --test color_picker_component` 2 passed（5.77–6.00s，单用例 <10s）。
  - 输入 hash：picker `73542e36`、model `62561f3e`、dialog `b96ac6da`、gate `54630179`（r9a/r9b 一致）。
- **T-01 探针**（decisions §3，随工作树删除）：native MCP pen 选色逐字节对齐预计算引擎值（`#405980CC`/`#805040CC`/`#8050407F`）、onhit 色板、dialog 内嵌；vue Playwright 真实拖拽/点击/棋盘 alpha 渲染，PASS 13/13。
- **实现要点**（详见 §9 复审收据与 decisions §8–§11）：串字段单一真源（`d_hex`/`ch_*`/`h_hold`/`g_hue`）+ 每 handler 入口解码；HSV 解码饱和度 ×10、行/alpha 条半字节 `/16`、rgb6 `substr(1,6)`、ColorDialog 外向事件声明、取消按打开会话去重、surface `g_hue` 携带、`#RGBA` 展开；示例窄窗堆叠与 364px 最小布局宽。
- **平台债/边界**：VM 实例作用域整型字段写可见性（当前设计不依赖，未宣称修复）；package 级取消可能双发（消费方按会话去重，README 记录）；native 模态期间截图零尺寸；Linux `not_run`（无 runner，不冒 pass）。
- **清理**：工作树 `D:/autostack/.wt/ui-088/auto-ui` 在 wt-guard clean 后移除（含删除临时 `examples/ui/088-probe` 与生成 `examples/color-picker/gen` 中的 724 个 pnpm junction 链接）；分支 `codex/ui-088-color-picker` 已删除；`plan-artifacts/plan-088/` 删除；retention registry 已更新。

