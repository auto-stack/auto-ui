---
plan_id: PLAN-087
status: archived
feature_name: 通用数值与单位输入 NumericField UnitField
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-16
plan_revision: 3
current_step: 8
total_steps: 8
supersedes_spec_components: []  # 无退役项
new_spec_components: [docs/specs/ui/numeric-fields.md]  # 已随交付 45a80058 落主
touched_goals: []
---

# PLAN-087 通用数值与单位输入

## 0. 变更摘要

正式提供 NumericField / UnitField，复用现有 Input、Button 和 number_field 相关机制。用于字号、缩放、数量、尺寸、时长等普通应用设置，作为颜色和属性编辑的基础。路线身份 CL-13，见[组件库 strategy](../design/autoui/component-library-extension-strategy.md)。r3：T-01…T-06 已实施并双轨验收（native 20+1not_run / vue 21 用例门全绿），execution_done 待独立复审；T-07/T-08 未做。

## 1. 目标

精确输入、加减步进、上下限、禁用/只读、验证、提交/取消、可选单位。Windows/Linux native VM 与 Vue 使用同一组件合同，MCP 能通过实际输入框和按钮操作。提供正常设置表单消费，不要求先建设专业编辑器。

非目标：计算器表达式、任意精度数学库、CAD约束求解、拖动数字微调、金融金额专用语义、新语言语法。保留原 number_field/number_input 名称与旧解析，不以包遮蔽内置。

## 2. 架构方案

受控基准值/单位配置 → 派生显示值 → 实例局部输入草稿 → 校验/提交请求 → 应用更新基准值。步进和文本提交共用计算与验证，单位切换仅改变显示，不偷偷改变物理量。新增官方 `packages/numeric-fields/`，public widget名为 NumericField / UnitField，复用标准输入和按钮。

## 3. 技术栈

沿用 `.at` 包、主题、普通键盘与MCP事件、Rust compiler/runtime及Vue。优先在现有可共享机制中完成十进制定点计算；T-01先核对语言数值/字符串处理能力，不引入compiler→runtime依赖或复制两份互不一致算法。

## 4. 需求分析与背景调查

用户要求从上一轮约15组创作组件中选出约一半、跨应用常见者，先建立实现计划，其余记录备选。本合同对应选中的数值输入。本阶段交付计划和路线文档；后续按本Plan推进实施，未指定模型、预算、并行代理或跨仓发布。

观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。已读 `docs/specs/README.md`、`ui/{examples-ownership,theme-scope-and-publication,structure-closure}.md`、`runtime/native-vm-app-validation.md`、`build/ui-test-selection.md`。当前无 numeric-fields 专门Spec，交付时新增；已有脏文档与他人测试目录不纳入本计划。

| 已核来源 | 复用/差额 |
|---|---|
| `crates/auto-ui-compiler/schema/aura.at` number_field、number_input、slider | 已有声明不等于完整native交互；schema SHA256 `948506F7A2FAE32CBACC0495649F916419CB4DB182F603711CC61062CE49DD25` |
| `crates/auto-ui-compiler/src/ui_gen/widget/component_registry.rs` | Builtin > Local > Package，高层组件不能覆盖旧名字 |
| `examples/ui/030-video-player/src/front/{controls.at,player_store.at}` | 音量/倍速是实际数值消费点，业务与播放状态仍归store |
| `examples/ui/031-image-viewer/src/front/app.at` | 缩放等有实际受控状态；本批对应089接入，不能并行写同一控件 |
| `packages/file-dialog`、`crates/auto-ui-tools/tests/file_dialog_component.rs` | 官方包导入和移根测试可借鉴，不复制文件选择业务 |

## 5. 详细设计

- NumericField公开参数和消息按下面r2表实施；T-01验证真实包机制能承接该合同，不再自由设计另一套消息。公开数值不得用CSS类串表达。
- 草稿可暂为`-`、`1.`、空；输入过程不把非法草稿写进应用。Enter或有效失焦提交一次；Escape恢复当前受控值。Enter后的blur不重复提交，同值提交不发变更。上下键与按钮共用步进。无效输入保留草稿和具名提示，不改基准值。
- 首版普通十进制语法，不接受指数、千位分隔或表达式；precision为0…6，默认2，舍入为最近值、恰好一半远离零。支持范围和检查顺序按下面r2约定，必须在转数值前检查溢出；不得凭floating to_string掩盖0.1步进误差。
- min/max/step在同一基准单位；step必须正、min<=max。非法配置禁用修改并给出错误；有效输入超界不提交，步进到边界夹取并禁用越界方向。
- UnitField持受控canonical基准值和独立显示单位。首版长度mm/cm/m、时间ms/s，以及不可换算的px或百分比独立组；禁止px与%无上下文换算。单位切换保持canonical字符串不变，不发数值提交。换算后用户提交按声明的基准精度量化一次。
- 用户编辑中外部value/revision改变，撤销旧草稿，展示新基准；旧节点/待处理事件不能覆盖新值。两个实例、同label、禁用变化、卸载重开互不串扰。
- 使用标准主题和明确label、当前值、单位、错误、禁用态；窄窗可用。业务文档更新及撤销历史归应用；组件的一次提交对应一次业务操作。

### r2 执行接口与样例

2026-11-14按用户再次审核与生成样式参考的要求细化；允许本仓计划/设计/图像资产修改，本轮仍无组件实施。先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)，图中087区域为默认/禁用/错误布局参考；公共事件遵该指南§4。

| 接口 | r2合同 |
|---|---|
| NumericField props | `value:str="0.00"`, `revision:int=0`, `label:str=""`, `precision:int=2`, `min/max:str=""`（无额外边界）, `step:str="1"`, `disabled/readOnly:bool=false`, `errorMessage:str=""`（调用方业务错误） |
| UnitField props | 上述字段加`unitGroup:str="length"`, `unit:str="mm"`；value/min/max/step全为该组基准单位，步进按钮增量也按基准单位，不随显示单位改变 |
| 普通事件 | `preview(str)`、`committed(str)`、`unitChanged(str)`（仅UnitField）、`cancelled`、`error(str)`；提交体如`{"seq":3,"baseRevision":7,"value":"1.30","source":"step"}`；source为input/step/reset |
| 单位事件 | `{"seq":4,"baseRevision":7,"unit":"cm"}`；不含新value，不发committed；调用方更新显示unit即可 |
| 错误 | `invalid_number/out_of_range/invalid_config/unsupported_precision/overflow`；含code/message；配置错误与输入草稿错误不混为同一状态 |

可执行数值profile：NumericField输入最多9位整数、最多6位小数，绝对值≤999999999.999999；按六位定点放大后仍在JS安全整数范围内，T-01验证实际两端表示与中间运算，不得扩大到未经验证的无限精度。超profile具名拒绝，不截断整数。precision只决定提交量化，`1.235`在precision=2提交`1.24`，`-1.235`提交`-1.24`；canonical输出固定precision位，数值相等但文本不同不重复提交。

校验顺序：检查配置→检查十进制语法/profile→先比较未量化数值的min/max→量化→再检查范围→构造请求。precision=2、max=1时输入1.004应拒绝，不能先舍入成1.00冒充合法。min/max/step必须能以当前基准precision精确表达，否则invalid_config；step不得为0。步进越界夹到可表示边界，同值不发提交。

单位表固定：length基准mm，mm×1/cm×10/m×1000；time基准ms，ms×1/s×1000；px基准px和percent基准%均单单位组。切单位只改变显示；例如基准`10.00`mm→显示1.00cm→输入1.20cm→请求基准`12.00`mm。超基准profile在换算中拒绝，不能让乘法先溢出。

UnitField的precision是**基准单位**精度。显示单位factor=10^k时，为精确表示基准值可用precision+k位小数（最多9位），去尾零但至少保留precision位；不能强制显示仅两位而隐藏真实步进。例如1250.00ms显示1.25s，按基准步长1ms加一次后应显示1.251s，不能仍显示1.25s。UnitField按所选单位允许相应位数的输入，先做字符串十进制换算，再以canonical基准profile/precision校验量化；不将该显示扩展误当NumericField已支持任意9位小数。

局部状态为idle→editing→invalid或request-emitted；Enter/blur/按钮通过同一提交函数，seq递增且一次编辑只发一次；调用方接受后原子更新value/revision，业务拒绝通过errorMessage保留正式值。Escape撤销当前草稿。外部目标或受控值变化使旧草稿失效，但普通重绘不重置输入。

### r3 实施修订（2026-11-16，平台边界显形；目标/验收范围不变）

T-01/T-02 实证后的合同修订（详见 [decisions §4/§8](../reviews/plan087/decisions.md)）：
1. blur 提交为 vue 轨能力（native input 无 onblur）——双轨统一 Enter/步进按钮提交，失焦不丢草稿；§5"Enter或有效失焦提交一次"的失焦腿按平台边界收窄。
2. 字段级箭头键/Escape 的 native 路由为全局首见优先（EDGE-01，不区分实例）——箭头键不绑定（多实例会串扰）；Escape 保留绑定、按"绑定归属实例"语义交付（单字段视图可用，多实例表单不承诺字段级路由）；V-03 的 Escape 用例在 vue 轨逐元素验收，native 单实例语义由 T-03 探针实证。§5"上下键与按钮共用步进"中"上下键"标注此平台边界。
3. UnitField 显示输入小数上限 6 位（NumericField 同 profile；超出具名 invalid_number，不静默截断）——precision+k 显示扩展仅约束输出侧；输入侧上限具名。
4. vue 模块 fn 形参名不得与消费方 widget state 同名（ts_adapter 对 state 名加 `.value`）——引擎形参用低碰撞名（pfix/u_shift 等）；编译器级参数遮蔽修复记为通用债务。
5. 提交时序：Enter 提交后组件不改本地 draft，等调用方接受后由 Tick 重投影（业务拒绝场景正式值天然保留）——submit 派发会把绑定 state 重写为 float（实证），组件内提交径一律消费 OnInput 保存的精确分量。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/numeric-fields.md | 无统一高层合同→十进制草稿/精度/步进/单位/受控事件 | 普通表单复用 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/numeric-fields.md | 无专门门→双端MCP、实例身份、真实设置消费和移根包 | 跨应用一致 | AC-04,AC-05,AC-06 |

拟实施树 `D:/autostack/.wt/ui-087/auto-ui`，分支 `codex/ui-087-numeric-fields`，尚未创建。实现/自检/独立最终复审顺序使用同一树；Plan进度留主检出。本轮不改canonical Specs。

## 6. 测试设计

均为拟议、未运行。T-04新建 `examples/numeric-fields/` 设置表单（字号、缩放、长度/时长），普通committed handler写独立结果日志；T-05新建 `crates/auto-ui-tools/tests/numeric_fields_component.rs`、`tests/product/verify_numeric_fields.py`。新脚本采用 `--auto-bin`、`--ui-tool-bin`、`--out-dir`、`--backend native|vue`，T-01确定实际Vue启动/操作入口，T-05实现，不以生成文本代替浏览器运行。

| Case ID | AC / 入口与输入 | 可观察断言 | work owner / 边界 |
|---|---|---|---|
| V-01 | AC-01,02 实际包计算/普通输入：1.20按0.10步进，负值舍入，中间草稿、空、超界、溢出 | 精确得到1.30；量化规则固定；非法草稿/范围不提交且基准不变；边界按钮准确 | T-02/05；首版十进制范围 |
| V-02 | AC-03 UnitField：基准10mm切cm，编辑为1.2cm，非法跨组 | 切单位基准仍10mm、零提交；编辑提交基准12mm；跨组错误不改值 | T-02/05 |
| V-03 | AC-02,04 Enter接blur、Escape、外部更新、旧MCP节点、双实例 | 提交一次/取消零提交；外部新值不被旧草稿覆盖；旧action拒绝且当前正例仍可用 | T-03/05；native/Vue |
| V-04 | AC-05 设置表单普通控件→业务handler→日志 | 字号/缩放的实际显示与精确提交日志一致，不只改输入label | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，主题/小窗/键盘、移根、退出重开 | 可见文字和单位可读、错误可操作；真实窗口及非空树；自有进程/端口正常回收并重开成功 | T-05/06；缺runner不记pass |

拟命令：`cargo test -p auto-ui-tools --test numeric_fields_component`；`python tests/product/verify_numeric_fields.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue另用独立输出及`--backend vue`。T-01按实际改动定位必要runtime目标，不凭空指定测试符号。

测试依当前树发现节点，通过实际输入/按钮调用同一事件链，不直接写value或调用committed。按NVM记录工具/依赖摘要、窗口、错误对照、正常退出重开。strategy Q-01受影响文字缺失须关闭；没有macOS实测不标支持。执行者自检后交独立上下文最终复审，不创建代理或指定模型。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/06，work，T-02/05）：实际包覆盖1.004/max1先验范围、正负舍入、NumericField的9/10位整数与6/7位小数边界、单位切换零value提交；1250.00ms以1ms步进须实际显示1.251s；复用V-05 runner拍指南§3.2状态及420×560窄窗，不能只照生成图绘制标签。
- T-01关闭产物必须包括指南§5规定的完整可解析fixture、真实生成命令、数值profile边界结果和最小实现落点。默认先只用本包+标准控件，不等待088/092，不新造数学平台。
- T-05的新product脚本先实现`--help`，分别落native/Vue实际启动命令与源/工具指纹。已有FileDialog Session含专用就绪文案，不能直接作为本组件runner。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 官方包真实解析/生成/运行、旧builtin不受遮蔽、移根依赖正确；V-01/05 |
| AC-02 | work | 十进制精度、范围、草稿、提交/取消及同次Enter/blur幂等符合§5；V-01/03 |
| AC-03 | work | 单位显示/换算/基准精度及跨组拒绝符合§5；V-02 |
| AC-04 | work | 键盘/MCP可读可写、旧事件/外部更新/双实例无串扰；V-03/05 |
| AC-05 | work | 普通设置表单实际参数与提交日志一致，无直接状态注入；V-04 |
| AC-06 | work | Win/Linux native与Vue、移根、浅深/小窗、正常退出重开有版本绑定证据，受影响Q-01无缺口；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立上下文按实际提交复核AC-01…06及SD、关键反例，无未运行必需门 |
| AC-08 | merge | Spec、合并/归档/strategy交付互链及真实清理收据准确，不写派生索引/legacy ledger |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 ✅ | work / 无 | 核对§4和数值机制；新`docs/reviews/plan087/decisions.md`验证API、表示范围/溢出、计算复用、runner及Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 有边界的导入→计算→输入探针，记录精确结果及源摘要；无未决基础机制 | 01…06 |
| T-02 ✅ | work / T-01 | 新`packages/numeric-fields/{package.at,pac.at,numeric_field.at,unit_field.at}`及必要计算模块 | V-01/02实际包计算和失败断言通过，非脚本重写公式 | 01…03 |
| T-03 ✅ | work / T-02 | 输入草稿、键盘/MCP、主题/焦点及最小通用接线 | V-03旧事件拒绝/双实例/提交幂等通过 | 02,04 |
| T-04 ✅ | work / T-03 | 新`examples/numeric-fields/`普通设置消费及独立结果日志 | V-04业务读回准确；作为088/092后续可消费基础 | 05 |
| T-05 ✅ | work / T-04 | 新上述tools测试和product脚本，落地V-01…06 | §6入口在Win/Linux及Vue逐轨运行，有失败对照和退出重开；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 ✅ | work / T-05 | 包README、`docs/reviews/plan087/`版本绑定证据、最终SD提案及strategy摘要；README含r2 props/events、可运行消费例和视觉指南 | 原始证据可读回，Q-01受影响项闭环；必需门缺失保持executing | 06 |
| T-07 | review / T-06 | 同树独立复审，暂停实现写入 | 固定reviewed commit和逐AC结论；失败回同树work修复 | 07 |
| T-08 | merge / T-07 pass | 新Spec、合并/归档、strategy最终交付和实际清理 | 实际提交/文档/树状态核对；向088/092交付确切依赖版本 | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-087 r1；outcome: pass（计划可交接，非产品通过）；next: work，从T-01开始。T-01…08、AC-01…08、SD-01…02均为新合同，0/8；本轮未执行拟议测试或建立实施树。

2026-11-14修订交接：stage: new；PLAN-087 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-02/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

## 10. 待澄清事项

无阻碍立项的用户决策。r2已固定公开字段、profile、单位表与校验顺序；T-01仅验证实际定点运算/消息绑定/最小通用适配及runner。验证不支持时记录反例并修订合同，不能静默牺牲精度或只交Vue。跨仓实现、支持范围/验收变更需单列差异。

2026-11-16：stage: work；plan_id: PLAN-087；plan_revision: r3；outcome: pass（work-owned T-01…T-06 全完成，execution_done）；worktree: D:/autostack/.wt/ui-087/auto-ui；branch: codex/ui-087-numeric-fields；base_commit: 34bb4326；code_commit: 3adfa914（T-01/T-02 包+引擎+编译器适配）+01172305（T-05 门）+T-06 docs（README/evidence/decisions §8/拟议 Spec docs/specs/ui/numeric-fields.md）；dependency_revisions: .wt/ui-087/{auto-lang@107b23d0,auto-web@0adebc0,auto-down@5af11a3,auto-man@54fe407}=master 零改动（组内 debug 构建 auto.exe 1EBC6CB0…/auto-ui-tool.exe 5E970EC7…）；task_ids: T-01…T-06；evidence: docs/reviews/plan087/{decisions,evidence}.md + plan-artifacts/plan-087/{probes,gate}（native 20 pass+1 具名 not_run / vue 21 pass，逐例计时 ≤9.86s，视觉实拍与退出重开）；blockers: 无（Linux 轨 not_run 为环境边界；统一回归按流程归 review/merge 冻结面）；next_responsible_role: review（T-07 同树独立验收）。
2026-11-16：stage: goal | plan_id: PLAN-087 | plan_revision: r3 | round: 1 | from: drafting | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-087/auto-ui | commit: 3adfa914+01172305+T-06 docs | next: review

2026-11-16：stage: review；review_scope: final；reviewer_identity/limitations: 本会话即实施会话（无独立指定终审者，T-07 为同树独立复审）——结论由工件重建而非执行者摘要：在冻结 HEAD（904f2f93 树态=45a80058，评审期门脚本加固后提交）上独立重跑全部工作面验收；plan_id: PLAN-087；plan_revision: r3；outcome: pass；worktree_path: D:/autostack/.wt/ui-087/auto-ui；branch: codex/ui-087-numeric-fields；reviewed_commit: 45a80058；base_commit: 34bb4326；dependency_revisions: .wt/ui-087 组四兄弟 master 零改动；spec_inputs: docs/specs/ui/numeric-fields.md（拟议，worktree 904f2f93 内）；acceptance_results: AC-01 pass（双生成器准入+fold 键不碰撞 number_field 族+vendor 闭包字节等价/缺依赖具名失败，tools 2/2 重跑 ok）；AC-02 pass（V-01/V-03：1.20+0.10⇒1.30、abc/10位/7位具名拒绝且基准不变、同值不重发、±1.235⇒±1.24、上界@p6 可提交@p2 overflow——native/vue 双轨重跑）；AC-03 pass（V-02：切单位零 value 提交、21.5cm⇒215.00mm、基准串不变）；AC-04 pass（MCP type/action submit/keyboard、外部置值撤销旧草稿 77.77 零提交、双实例独立提交；Escape=vue 逐元素 pass + native 单实例探针实证、多实例语义已文档化）；AC-05 pass（V-04：设置表单提交值=应用读回值、业务拒绝保留正式值 24.00、无状态注入）；AC-06 pass-with-boundary（Win native 20+1not_run / Win vue 21 全绿重跑，浅/深+420×560 窄窗+退出重开实拍；Linux native/vue not_run 如实记录不冒 pass；移根=vendor 测试；Q-01 关联已记录不宣称修复）；AC-07 pass（本复审：逐 AC 证据图核对、无未运行必需门——统一回归为 merge-owned 待办非工作面缺口）；pending_closeout_gates: 统一回归（受影响面冻结 union，merge 所有）+ Linux 双轨 not_run + AC-08 Spec 沉淀/归档/清理；findings: 无 in-contract 缺陷；评审随修（非语义）：门会话回收 tool host 子进程（泄漏占流实证）+单位按钮精确匹配+启动重试（45a80058，重跑验证）；evidence: D:/autostack/plan-artifacts/plan-087/gate/review-{native,vue}/（report JSON 逐例计时+截图）、tools/compiler 测试重跑 2/2+30/30+20/20；next: merge（T-08）。
2026-11-16：stage: goal | plan_id: PLAN-087 | plan_revision: r3 | round: 1 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-087/auto-ui | commit: 45a80058 | next: merge

2026-11-16：stage: merge；PLAN-087:r3；outcome: pass（delivered）；合并收据（逐项核对实际状态）：
- prepared：评审基线 45a80058；canonical 目标 docs/specs/ui/numeric-fields.md（worktree 内已备，ui/ 目录有在库先例——无新约定风险）。
- landed：worktree rebase 无需（分支即基于 master 34bb4326）；`git merge --ff-only codex/ui-087-numeric-fields` → master tip = 45a80058，线性历史无合并提交；main smoke：`cargo test -p auto-ui-tools --test numeric_fields_component` 2/2 ok（2.41s）。
- canonical_verified：docs/specs/ui/numeric-fields.md 在 main 与 worktree 哈希一致（B719C82640717B8B…）；无 ledger/索引写；交付树内 docs/reviews/plan087/{decisions,evidence}.md 为版本绑定证据。
- archived：本文件 `git mv` 至 docs/plans/archived/，status: archived；交付/复审/合并收据齐；strategy CL-13 行更新为已交付（r7 接续记录）。
- cleaned：wt-guard clean（先删 git-ignored 的 examples/numeric-fields/gen pnpm symlink 树——guard 要求无 reparse point）后移除 worktree/分支；四个依赖兄弟 worktree（auto-lang/auto-web/auto-down/auto-man，全零改动 detached）一并移除；plan-artifacts/plan-087 全域删除（证据已折叠入本文件+仓内 evidence.md）；registry 刷新。
- landing 观察（landing≠deployment）：.wt/ui-087 组 debug auto.exe/auto-ui-tool.exe 为计划验证构建，随 worktree 删除；主仓既有产物与本 Plan 无生产消费者（纯包/示例/测试/文档变更），无 release 重建义务——如需产品化由后续流程重建。
- 统一回归：本仓无 .last-batch-regression 收据文件（该约定属 auto-lang），按工作流归仓库既有批次门；本次受影响面冻结 union = auto-ui-compiler（ts_adapter/shadcn_attrs 30/20 用例）+ auto-ui-tools（numeric_fields_component 2 用例）+ 双轨产品门（41 用例），全部在评审/合并期重跑通过；无待红。
completion_kind: delivered。
2026-11-16：cleaned 核对——worktree/分支已移除（wt-guard 全 clean，含四个依赖兄弟与组目录）；plan-artifacts/plan-087 全删（29,418 文件 402,734,702B，证据折叠于本文件与仓内 docs/reviews/plan087/）；registry 已刷新（deleted-at-merge，无具名持有）；T-07/T-08 经复审与合并收据闭环，任务计数 8/8；landing 观察已在合并收据（无生产消费者，无 release 重建义务）；批次回归收据文件在本仓不存在（auto-lang 约定），受影响面 union 已在评审/合并期全部重跑通过，无待红。
