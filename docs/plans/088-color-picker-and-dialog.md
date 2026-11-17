---
plan_id: PLAN-088
status: execution_done
feature_name: ColorPicker 与 ColorDialog 通用选色器
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-17
plan_revision: 3
current_step: 6
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/color-picker.md]
touched_goals: []
---

# PLAN-088 ColorPicker 与 ColorDialog

## 0. 变更摘要

将路线CL-08提前立项：提供内嵌ColorPicker及组合ColorDialog，用于笔记/文字/画笔/主题预览等通用选色。复用现有Dialog、Slider、色板和087数值输入，不调用系统颜色窗口。见[组件库strategy](../design/autoui/component-library-extension-strategy.md)。本轮仅建计划。

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

按用户2026-11-14再次审核和生成样式参考要求补齐。先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)，图中088是色面/通道/透明度/底栏关系；可切RGB/HSV通道组，不要求将两组全部挤进图中的一行。

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

## 9. 复审记录

2026-11-14：stage: new；PLAN-088 r1；outcome: pass（规划可交接）；next: work，从T-01开始，T-03等087。新增T-01…08、AC-01…08、SD-01/02，0/8；本轮未实现或运行产品测试。

2026-11-14修订交接：stage: new；PLAN-088 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-03/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

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

## 10. 待澄清事项

r2已固定公共消息、通道范围/舍入、HSV灰黑边界和Dialog提交规则；T-01验证动态色面/计算复用、Paint历史适配与runner。尚无阻碍立项的用户决策；实际需要修改语言仓、扩大色彩管理范围或删除双端验收时须提出具体差异，不能自行回落系统弹窗。

阻塞（2026-11-17，work r3 实测，待用户决策）：通道输入（AC-02 的编辑入口之一）在两套设计下均被平台缺口挡住——(a) 内嵌 NumericField：depth-3 实例 Tick/状态缺口（decisions §7）；(b) 内置 input：native 实例 handler 状态读出别名值、vue 提交路径 codegen 'length' 错（本轮 §9 blocked 收据）。核心选色链（palette/色面/色相/alpha/HEX/dialog）双轨全绿。请裁定：①先立 VM runtime 修复计划（实例作用域 handler 状态一致性 + depth-3 实例语义），本计划挂起等其交付；或 ②通道入口验收降域至「Vue 轨待 runtime 小修后补证 + native 记债」，先复审已交付的其余面。裁定前本计划挂起（executing）。

修订待决（2026-11-17 work 提出 → r3 已裁定采纳）：通道输入由「内嵌 PLAN-087 NumericField」改为「内置 input + cp 引擎夹取校验（Enter 提交一次/同值零事件/非法具名拒绝）」。依据：depth-3 组件实例 Tick/状态解析缺口（VM runtime 债，实证 decisions §7——App→ColorPicker→NumericField 每 100ms 根态崩溃，087 depth-2 对照零失败）。AC-01「087依赖真实导入」措辞随之改为「官方包真实导入运行；087 为语义基准（不引入其实例）」——该措辞修订请复审确认。同目标、同验收实质（V-01 通道精确编辑经 MCP type+submit 等价路径）、同仓集。

撤销（2026-11-17 goal 续跑，裁定路径已由 work 在计划范围内解决，无需用户二选一）：通道输入阻塞解除——draft 改为串字段单一真源（d_hex/ch_*/h_hold）+ 每 handler 入口解码，并修复 4 类自含实现缺陷（HSV 解码饱和度 ×10、行/alpha 条高位半字节缺 /16、rgb6 substr 7→6、ColorDialog 外向事件声明缺失）与 2 类呈现缺陷（通道错误文本/error 事件、vue 重复 let）。native 通道/alpha 证据成立：gate-native-r3t 8 用例 pass、gate-vue-r3g pass、工具门 2/2。VM 整型字段写可见性缺口仍登记为债（访问面已移除，不宣称修复）；Linux 轨 not_run 如实记录。详见 docs/reviews/plan088/decisions.md §8。
