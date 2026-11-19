---
plan_id: PLAN-092
status: drafting
feature_name: PropertyInspector 通用属性与设置面板
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-14
plan_revision: 2
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/property-inspector.md]
touched_goals: []
---

# PLAN-092 通用属性与设置面板

## 0. 变更摘要

提供由数据描述字段的PropertyInspector，组合现有Input/Checkbox/Select及087 NumericField/UnitField、088 ColorPicker，用于普通设置页和对象属性表单。首版不要求使用者熟悉游戏/CAD领域。路线CL-14，见[组件库strategy](../design/autoui/component-library-extension-strategy.md)。本轮只立项，0/8。

## 1. 目标

统一字符串、整数/小数、单位数值、布尔、枚举、颜色字段；分组/折叠、搜索、label/help、只读/禁用、默认值恢复、字段验证。应用声明字段及受控值，组件选择合适编辑控件，跨Win/Linux native VM及Vue保持同义并可被MCP操作。

非目标：任意对象反射/脚本执行、表达式求值、嵌套集合编辑、多对象混合值、批量属性事务、游戏组件系统、CAD参数求解、任意自定义renderer插件和资源类型深度编辑。091资源字段适配留后续，不作为本Plan硬依赖。

## 2. 架构方案

纯数据字段schema+受控values+targetId/revision → 验证schema → 按kind组合标准编辑器 → 发fieldId/targetId/revision/value请求 → 应用校验并更新对象/设置。PropertyInspector不通过字符串路径写对象，不eval消息名，不拥有应用状态或撤销历史。

新增`packages/property-inspector/`。T-01可独立核定数据驱动widget选择；T-03后接入必须消费087/088确切交付版本。用显式kind分支实现首版，不为此新增语言反射或运行时动态代码。

## 3. 技术栈

`.at`正式组合包、普通表单控件、087/088、现有主题和组件状态/MCP、VM/Iced与Vue。应用持久化继续用现有Storage机制，本包不访问全局设置或文件系统。

## 4. 需求分析与背景调查

用户要求约一半常见跨应用组件先建计划，本合同为设置/属性面板；当前阶段为计划与路线，后续按合同实施。无指定模型、预算、代理或跨仓发布。观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读Specs索引、主题作用域/结构/示例归属、NVM验证和当前设计体系文档。当前无PropertyInspector正式Spec；已有debug inspector不作为用户属性编辑器冒充复用。

| 已核来源 | 可复用/边界 |
|---|---|
| `crates/auto-ui-compiler/schema/aura.at` Input/Checkbox/Select/number_field等 | 基础类型已有声明，需走真实双端路径；不重新做一套输入控件 |
| `crates/auto-ui-runtime/src/ui/debug/{inspector.rs,inspector_cache.rs}` | 调试UI/缓存用途，不是应用对象编辑器，不把debug可写能力开放给正式属性数据 |
| `examples/ui/026-database/src/front/app.at` | 有实际数据编辑表单和业务验证模式，作为设计输入；本批不重构数据库业务或抢086/085写入 |
| `examples/ui/041-auto-edit/src/front/app.at`、`editor_store.at` | actions/业务状态归应用，组件不自行执行字段名对应命令 |
| `docs/specs/ui/theme-scope-and-publication.md` | App/session与实例作用域，当前对象属性不能污染其他App/theme |
| PLAN-087/088 | 数值/单位和颜色的单源基础；正式面板不得拷入实现 |

## 5. 详细设计

- 字段schema与values分开，targetId/revision标识当前对象；公开record/事件按r2表，T-01验证真实数据分支和消息接线，不再自行设计另一套类型系统。
- schema只允许数据。重复fieldId、未知kind、缺枚举值、非法范围/默认值给具名schema错误，不渲染歧义可编辑字段；不能悄悄转成文本绕过类型。空schema为正常空态。
- 每个编辑器共用其原验证/草稿/提交事件；面板按稳定fieldId组合请求，值的类型不能由label或字符串猜测。不存在字段拒绝写入；readOnly/disabled既禁止控件操作，也在消费action时重验。
- 面板默认按字段逐项提交请求，应用决定是否接受及何时持久化；业务错误以字段错误反馈，不先改正式values。连续输入/Enter+blur不重复业务提交。
- “恢复默认值”同样发普通变更请求，默认值须通过同一类型验证。没有默认值不显示恢复按钮。搜索/折叠只改变展示，不清空值、提交草稿或误恢复默认。
- 切target/revision或schema修订失效旧草稿和action；同label不同ID不串写；两个面板独立。目标变更期间原字段提交不能落到新对象；焦点修复不提交未验证草稿。
- 通道可读：目标、字段ID/label/kind、当前值/单位、只读/禁用/错误/默认值差异等。普通键盘/MCP通过实际子控件编辑，无set_property后门；错误可定位到字段且Tab可离开。
- 普通设置消费示例显示“阅读与外观”设置，含字号/行距/显示辅助/对齐枚举/文本色，实际预览使用对应参数，应用handler通过Storage保存；关闭重开按应用自己的持久化合同读回。控件不保存全局偏好，不将schema当通用应用框架。

### r2 执行接口与样例

按用户2026-11-14再次审核和样式参考要求细化；先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)的092区域。生成图的组级恢复图标不增批量事务，v1仍按字段恢复默认。

| 接口 | r2合同 |
|---|---|
| Props | `fields/values/groups/fieldErrors:List=[]`, `targetId:str=""`, `revision:int=0`, `schemaRevision:int=0`, `disabled/readOnly:bool=false`, `showSearch:bool=true` |
| 字段record | `id,label,kind,groupId,help,required,readOnly,disabled,hasDefault,defaultValue`，再按kind附precision/min/max/step/unitGroup/unit/options；groups为`{id,label}`，不嵌套对象schema |
| value record | `{"id":"fontSize","value":"16"}`、`{"id":"showHints","value":false}`；每个字段恰好一个value，重复/缺失/未知字段值为invalid_values，不偷偷以default填充正式数据 |
| 首版kind | string→JSON字符串；integer/number/unit→087的十进制字符串（integer precision=0）；bool→JSON布尔；enum→候选项id字符串；color→088的8位HEX字符串 |
| 编辑/预览事件 | `changeRequested(str)`、`previewRequested(str)`、`previewCancelled(str)`及`error(str)`；请求如`{"seq":8,"baseRevision":3,"schemaRevision":1,"targetId":"reading","fieldId":"fontSize","kind":"integer","value":"18","source":"edit"}`，source为edit/reset |
| 业务错误反馈 | fieldErrors含`{targetId,fieldId,baseRevision,seq,code,message}`；只显示匹配当前目标/版本/已发请求者的错误，迟到错误不挂到新对象 |

字段例：`{"id":"showHints","label":"显示辅助提示","kind":"bool","groupId":"appearance","hasDefault":true,"defaultValue":false}`。hasDefault明确区分“没有默认值”与false/0/空字符串，不能用truthy判断。enum options为`[{id,label}]`且id唯一，default/value必须属于其中；kind=string的数字文本不自动升级成number。

integer零默认值的实际编码是`{"id":"count","label":"数量","kind":"integer","hasDefault":true,"defaultValue":"0"}`，按087保留十进制字符串；bool的false仍是JSON布尔，不能把两者都转成任意文本再交给应用猜。

Schema错误如重复ID/未知kind/非法默认值禁用歧义编辑并显示面板错误；value类型不合法为对应字段invalid_value，不能转文本绕过。字段/values需由调用方原子发布，initial空字段是正常空态。面板只组合087/088等真实子控件，不复制其输入/颜色算法。

编辑先由子控件完成草稿验证，再发含完整目标身份的changeRequested；父根据baseRevision决定接受，原子更新values/revision；拒绝只更新匹配fieldErrors，正式values不变。preview只能用于临时可见预览，不能写Storage；目标/schema变化及取消令该预览失效。恢复默认也走changeRequested(source=reset)，false/0默认值必须可恢复。

搜索按label/help匹配，不改变字段ID或values；折叠以groupId维持本地展示状态。目标或schemaRevision换代撤销旧字段草稿，普通同值重绘不重建所有输入。r2不增加多对象、资源字段或动态renderer插件。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/property-inspector.md | 无统一面板→数据schema/受控值/字段类型/默认值/错误/请求合同 | 设置和属性表单复用 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/property-inspector.md | 无专门门→目标revision/实例/MCP、普通设置真实消费与双端证据 | 不写错对象 | AC-04,AC-05,AC-06 |

拟树`D:/autostack/.wt/ui-092/auto-ui`、分支`codex/ui-092-property-inspector`，未创建；实现/独立最终review顺序同树，canonical本轮不改。

## 6. 测试设计

T-04新`examples/settings-inspector/`真实设置/预览/应用Storage消费，T-05新`crates/auto-ui-tools/tests/property_inspector_component.rs`及`tests/product/verify_property_inspector.py`，统一`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`；全部拟执行。

| Case ID | AC / 真实入口和输入 | 区分性断言 | work owner / 边界 |
|---|---|---|---|
| V-01 | AC-01,02 真实包schema组合，各kind、空schema、重复ID/未知kind/非法默认值 | 类型精确、标准子控件来自087/088；非法schema具名拒绝，不降级为可写文本 | T-02/05 |
| V-02 | AC-02,03 普通编辑/范围错误/只读/业务拒绝/恢复默认/折叠搜索 | 请求含正确field/value类型，拒绝保持正式值；恢复走同一验证链；折叠搜索不产生业务写入 | T-02/03/05 |
| V-03 | AC-04 编辑A字段后换B/改schema，同label两面板、旧action | 旧请求不能写新对象/另一面板，当前正例仍通过；丢焦不误提交草稿 | T-03/05 |
| V-04 | AC-05 设置示例字号/颜色等→实际预览→Storage→正常重开 | 真实文本尺寸/颜色与值对应，业务存储读回一致；错误/取消草稿不持久化，第二设置实例不污染 | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，浅深/小窗/键盘、移根依赖、退出重开 | 字段语义/label/错误可读，普通MCP子控件可操作；版本绑定窗口/树，owned进程/端口回收成功 | T-05/06；Q-01受影响项闭环 |

拟命令：`cargo test -p auto-ui-tools --test property_inspector_component`；`python tests/product/verify_property_inspector.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue独立跑。T-01确定实际Storage落点/读回和预览像素/几何oracle，不能只比较schema字符串或表单label。

普通输入经正式事件到应用handler，不直接调用写属性或修改Storage当操作。记录NVM证据和087/088来源闭包；没有必需runner、视觉或真实消费结果不记pass。独立review在同树独立上下文核对，未指定模型/代理。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/04/06，work，T-02/05）：bool默认false、integer默认0与hasDefault=false严格区别；恢复发精确类型值，数值文本不误作字符串字段；A对象的迟到fieldErrors/commit不能出现在B；按指南§3.2拍错误/只读/恢复/窄窗。
- T-01用一组string/integer/bool/enum/color实际record证明`.at`按kind分支到标准控件、回调带value类型且可生成两端；不能只证明JSON能解析。087/088未交付时probe标注probe_only，最终子控件须用正式依赖。
- T-04设置预览同时记录实际字号几何/颜色像素和Storage读回；不能把面板自己的label变了当作设置已影响应用。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 正式包及087/088真源导入运行/移根，不依赖debug写接口；V-01/05 |
| AC-02 | work | 首版各字段类型、schema错误、范围及业务拒绝合同正确；V-01/02 |
| AC-03 | work | 默认值恢复、搜索/折叠、草稿提交/只读行为正确且无隐式写入；V-02 |
| AC-04 | work | 目标/schema revision、双实例、MCP/键盘与旧action安全；V-03/05 |
| AC-05 | work | 普通设置预览实际变化、Storage及正常重开精确读回；V-04 |
| AC-06 | work | Win/Linux native+Vue、依赖闭包/视觉/生命周期证据齐全，受影响Q-01闭环；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立复核AC-01…06/SD及真实目标写入/预览oracle，固定提交，无必需门缺口 |
| AC-08 | merge | Spec、合并/归档/strategy交付及实际清理记录一致，不手写派生索引 |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan092/decisions.md`验证数据分支/消息/目标revision、Storage预览oracle、runner/Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 有边界的schema→真实子控件→应用更新探针，不eval字符串、不需新反射 | 01…06 |
| T-02 | work / T-01 | 新`packages/property-inspector/`schema验证/受控字段投影/错误状态 | V-01/02类型/反例通过，不先造自有数值/选色实现 | 01…03 |
| T-03 | work / T-02 + 087/088评审交付pin | 正式字段组合/默认值/搜索分组/MCP/焦点/目标修订 | V-02/03旧目标/双实例/只读正反例通过 | 02…04 |
| T-04 | work / T-03 | 新`examples/settings-inspector/`普通设置预览和应用Storage | V-04实际参数、可见效果、保存/重开读回，不改用户系统设置 | 05 |
| T-05 | work / T-04 | 新上述tools/product门，落实所有V及必要基础组件回归 | §6入口Win/Linux+Vue有真消费、反例和退出重开；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 | work / T-05 | README、`docs/reviews/plan092/`证据、最终SD/strategy摘要；README含r2 props/events、可运行消费例和视觉指南 | 基础包/工具/存储身份可复核，Q-01受影响项闭环 | 06 |
| T-07 | review / T-06 | 同树独立review，停止实现写入 | reviewed commit逐AC/SD结论；失败返work | 07 |
| T-08 | merge / T-07 pass | Spec、合并/归档/strategy链接及实际清理 | 最终提交/文档/树状态核验，不以合入补产品验收 | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-092 r1；outcome: pass（规划可交接）；next: work，从T-01开始，T-03等087/088。新增T-01…08、AC-01…08、SD-01/02，0/8；本轮无实施或产品测试。

2026-11-14修订交接：stage: new；PLAN-092 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-07/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

## 10. 待澄清事项

r2已固定字段/value类型与请求/错误身份；T-01验证数据驱动分支、设置实际预览/Storage入口、必要适配和runner。无阻碍立项的用户决策；不扩大到任意反射、自定义脚本、对象系统或业务框架，发现跨仓依赖明确owner后接续。
