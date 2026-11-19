# AGE M4：与现有AutoUI正式融合

> **2026-11-17 当前路线已统一**：[三路线strategy](../../../../auto-lang/docs/design/strategy/age-m2-three-route-strategy.md)、[共同架构](../../../../auto-lang/docs/design/autographics/age-m2-three-route-architecture.md)、[首批后引擎扩展合同](age-m2-engine-extension-contract.md)、[新版AutoForge产品架构](../../../../auto-forge/docs/design/architecture.md)为当前target-state入口。本页下方原M2/M3/M4排期、旧Forge定位及计划状态为对应日期历史快照；已交范围/标签/预算/研究保留，旧快照不作为当前阻塞或完成依据。

R1复用980/981/1068/OS064/985–987：ACE M2-A真实基础→完整UI/Widgets AV→AC→A2R→真实AutoOS全量迁移/安装退役。R2复用UI058→AG023→AG029与**已交UI080**→AG030；新增UI099地图、100场景资源、101 PBR/多灯、102通用TRS动画，AV包可完整归档，981/1068后续接AC/A2R，AG031保原门并接四扩展三端。R3新版AutoForge按工程/人工编辑→Agent提案/真实预览→App/Game AV创作→AC/A2R输出，独立产品验收；不把纯静态场景编辑或旧Agent冒新产品。

新cap/Forge不增加985/A前置；原UI各波不等新cap/Forge全交，AGE与Forge最终签章不互等。UI055/059/AG022研究不重做；621项是研究目录，不是承诺本轮全部实现。此次仅规划，原执行状态/已完成任务保持，不启动/恢复实施。

> **2026-11-11当前口径**：AGE只有M1/M2。M1已由AG026完成11/11并签三仓v0.6-AGE-M1，双债已闭；M2仍仅规划。981 r10十批34任务；UI058/AG023 r3；国风AG029、正式光影UI080/AG030、联合AG031已建drafting合同且全部0/10，复用研究、未实施。当前职责/状态/最小接口见[两阶段路线](age-three-milestones-and-ace-m2.md)，下方所有旧日期的状态/编号/“当前”字样均为历史快照，不作为今日调度或重开M1依据。


> **2026-11-09 完整性补充**：当前AGE M2必须同时通过传统AutoUI全语法/组件/模式迁移（980）和完整AGE Node语法/新旧组件互操作（981/LANGUAGE/FUSION），不是只接2D/3D或四类基础控件。纯传统UI源无需改Node，普通Node view可使用完整UI能力；五view、选择/条件与Start/Tick/Frame纳完整分母。两项硬门及未决策略见[当前路线§6](age-three-milestones-and-ace-m2.md)，本页历史编号/有限profile范围不覆盖新目标。

> **2026-11-09 当前两阶段口径**：本页完整融合现与2D/3D增强合入AGE M2，详[当前路线§6/§13](age-three-milestones-and-ace-m2.md)。979/980/981承共同前门、AV开发/A2R发行与生产融合，复用M1基础，不反向等新增二维/光影切片终验；增强服务并行、最终作者链消费ACE接面。不再单设AGE M3，Schema、组件/主题/Store/输入、direct/RQ及真实宿主兼容义务全部保留。下文M3/M4为历史编号，旧日期授权由当前路线覆盖；本轮只规划，不代表ACE已具备完整AGE能力或已获实施授权。

2026-11-04，路线决议v3。用户调整里程碑：M3先增强2D/3D与更复杂游戏/图形能力，AGE与现有AutoUI正式融合顺延M4。原M3融合目标与ACE前置保留，现归M4。用户进一步确认：Schema方言与ACE替换旧编译器应统一考虑，M4正式接入等待相应前置。本页确定下一阶段方向与边界，不是执行合同；本轮不取Plan号、不建树、不实现或切换默认入口。

## 1. 阶段定位

| 阶段 | 主目标 | 与下一阶段的边界 |
|---|---|---|
| M2 | 新Auto Node/use/model/msg/on下，完成两款游戏、Gallery及基础UI＋2D＋3D同应用组合（M2-01..10） | 048/049的有限基础UI与显式研究入口证明组合链；不据此称现有AutoUI生产体系已融合 |
| M3 | 增强AGE 2D/2DHD与更复杂3D，推动国风种田承载游戏和three.js逐项能力覆盖 | 具体游戏切片/画面与样本分母另立合同冻结；不等待ACE正式融合 |
| M4 | 等ACE的Schema方言体系成熟且旧编译器替换实际启动后，统一设计AGE与AutoUI正式融合 | 采用共同Schema方言/编译链，接通现有UI体系与2D/3D；生产消费/兼容与具体组件后端覆盖另立合同验收 |

AGE源码已在auto-ui，纯源码归仓879与未来内容目录更名020分别保原身份。M4的交付重点是语言、编译、运行和组件体系接通；目录更名不会替代这些功能验收，也不是融合的必要前置。内容示例继续归内容仓，M4不自动把示例搬入引擎仓。

## 2. 主要工作面

1. **Schema方言与共同作者模型**：UI/2D/3D沿ACE的Schema语言包/方言体系定义并接入共同编译链；不将现有手写UI parser继续扩充作为M4正式目标。对齐Node的model/msg/on/use与view/view-2d/view-3d，普通AutoUI组件可与图形节点组合；生产widget与新Node的语义/表示需明确映射，兼容迁移后再决定旧名字退役，不凭改关键字宣称融合。
2. **复用现有UI体系**：接通已存在的组件、布局、属性绑定、事件、样式/主题与资源。按真实公共能力选代表性组件做端到端集成，避免在AGE维护第二套彼此漂移的UI业务/控件实现；全量组件覆盖单列矩阵，不能用M2四类基础元素代替生产能力。
3. **统一状态与生命周期**：保留AutoUI已交付的组件实例隔离、props/emit与Store的App归属、迟到回调隔离、主题scope；协调图形实例、共享挂载、Life.Tick、UI事件/重绘、应用/窗口寿命。一个状态owner不因多出口重复推进，不以同字段名自动共享状态，也不强迫声明树等于运行存储。
4. **生产宿主与渲染接线**：将AGE嵌入现有AutoUI编译/运行/窗口、布局、输入、焦点/捕获、资源/device/合成路径；保已有UI行为与实际图形输出。以生产公开入口运行和现有应用回归证明，不止研究入口或Rust定制demo。
5. **编译与开发链收口**：在ACE实际替换旧编译器的路线中，统一对齐UI/2D/3D Schema、领域表示/IR投影、诊断、产物身份、模块/资源闭包和构建配置。给出旧UI工程的迁移/并存/默认切换/回退路径，核实际交付接口和兼容证据，不把两个不兼容语言依赖图强塞同一Cargo图。

以上为工作方向，不冻结具体API、工期、预算或实施Plan数量。旧实验profile价格/32credits等不是永久生产语言规则；M2交付保持其原合同，M4若调整资源/数值/执行支持必须显式设计和版本化验收，不能静默抬预算或删旧样本。

M3独立增强路线见[2D/3D能力路线](m3-2d-3d-capability-roadmap.md)。本页由旧`m3-autoui-integration-roadmap.md`迁出，旧路径只保历史定位；旧决议正文可从Git历史追溯。此次只调整目标阶段，不改现有Plan号、交付身份或M2合同。

## 3. 实施次序与进入门

M2按既有048→049→AG021及原必需门收口，不把M4新增生产融合要求倒灌在途Plan。UI043/044、UI状态/主题/Reload等路线分别保持自己的owner和合同，不以标题当成整项串行前置。

### 3.1 正式接入前置

2026-11-04用户明确：现有AutoUI仍以旧的手写语法实现为主，AGE建议与ACE一样采用Schema驱动的方言体系，因此应等ACE完善并开始替换现有Auto编译器时，再统一考虑AGE/AutoUI合入。下列条件共同成立后，才进入M4正式接入设计与实施立约：

- M2与已冻结的M3阶段必需交付面按各自合同收口，M3新增能力的来源/兼容证据可供融合消费；不以M4等待为由删改M2-01..10，也不将生产UI全量迁移追加给048/049/AG021。
- ACE的Schema语言包/方言与相关编译接口已有可消费的实际交付证据。具体支持域、Schema/语法规则、CST/AST与IR投影、诊断/源码位置、模块/增量与失效边界由ACE owner提供；不把设计草案或局部实验当完整平台已成熟。
- 替换现有Auto编译器的工作已经实际启动，明确所用入口/版本、迁移owner、兼容/并存/回退路线；AGE与AutoUI随这条共同编译链统一接入，不提前在旧链上做一次完整生产融合再全量重做。

“ACE完善”指正式接入所需能力达到可消费成熟度，具体门在当时实施合同冻结，不泛化成所有未来ACE特性都必须完成；“已开始替换”也不等于旧编译器已全部退役。此次明确了等待时序，不宣称上述条件已经具备，不自动更换SDK或切默认。

### 3.2 等待期间与启动后的路径

等待期间可以整理现有Demo/组件/Blueprint能力覆盖、旧行为与资源/输入/状态合同、源码样本及Schema需求；不预先为M4扩充完整手写UI parser、不启动全量Demo引擎迁移。M2已有有限研究入口继续服务其既定验证，M3继续做独立引擎/游戏能力；两者均不当长期Schema实现或生产UI已统一。新能力优先复用通用语义/数据/资源接口，未来Schema投影接线单独验收，不提前重做整套生产UI parser。

前置具备后，先做**Schema方言接入＋现有UI语义映射的有界端到端先证**，再按真实缺口分批立中等规模Plan。统一入口形成后，从`examples/ui/001-helloworld`开始，按能力依赖由浅入深推进全部UI Demo兼容：文字/布局/主题→状态/事件→表单/列表/滚动→菜单/actions/Blueprint→媒体/阅读/编辑等。编号作导航，批次按真实依赖冻结；原Demo业务、样式/主题、交互及验收保持，001当前的文字选择/复制、主题/模式绑定等也须承接，不能仅画出一行字就关闭整例。

Demo通过指在AGE公共路径**正常运行**，覆盖原有显示、绑定、交互、资源与生命周期；逐项形成Demo→组件/属性/事件→状态/资源→后端覆盖表。全部Demo绿形成强兼容证据，但未被Demo覆盖的组件变体/错误态仍须具名补验；Feature/FeaturePack领域能力不因Demo显示成功自动交付。此处只定推进方式，不新增计划号，不提前冻结全部组件/后端矩阵。

后续再完成生产宿主/资源/输入接线、真实UI＋图形应用、必要迁移与默认切换。源码合入、默认入口切换和旧路径退役分别验收，失败可回退。相应独立review与源/版本/SDK身份按真实合同核验。

## 4. 完成判据的方向

代表性现有AutoUI应用从生产公开入口使用普通组件/主题/状态与AGE 2D/3D，在同应用中保持绑定、事件、焦点、实例/Store隔离和正确画面/资源寿命；业务由Auto执行，已有UI消费者有明确兼容证据。新旧工程/源闭包、诊断、资源发行、必要修改重建与坏件恢复可复验；独立review和冻结交付齐全。具体必需组件集/原生输入与后端矩阵在M4实施合同明确，不能用“源码同仓”或M2少量基础UI绿灯认领M4。

首批融合以当前桌面能力为落点；Vue/DOM与WebGPU、移动原生/Web分别按既有平台边界研究/立约。“同一个程序跑在任何屏幕”仍是长期方向；DOM overlay、可采样UI和深度正确的三维UI分别验收，不自动混称。M4等待ACE所需Schema能力与编译器替换实际启动，并统一接线；ACE所有后端/AOT全面替换、完整HMR、CPC、ECS、全平台产品化的全量交付不随M4命名自动捆绑。相关接缝按真实依赖纳入具体合同。

## 5. 当前知识与来源

本页为目标设计，当前知识仍见`docs/specs/`，本轮不修改canonical或旧Plan：

- [M2混合组合及已核缺口](m2-mixed-ui-2d-3d-composition.md)与[M2当前路线](m2-game-first-three-lane-roadmap.md)。
- [ACE语法/Schema语言平台方向](../../../../auto-lang/docs/design/39-ac-incremental-syntax-and-language-platform.md)（目标态，非完整交付声明）；当前UI的`crates/auto-ui-dialect/src/grammar/mod.rs`及`grammar/{view,widget}.rs`已是领域接口分离下的手写规则，不等于Schema生成语法。AST codec的schema版本也不等于文法Schema。
- [共同作者模型](declarative-authoring-model.md)；[UI结构/状态资源owner](../../specs/ui/structure-closure.md)。
- [组件/Store实例](../../specs/runtime/component-and-store-instances.md)、[主题归属/发布](../../specs/ui/theme-scope-and-publication.md)、[原生路由/生命周期](../../specs/runtime/vm-route-page-loading.md)。
- [Reload实际就绪边界](../../specs/runtime/reload-readiness-and-lifecycle.md)：研究有界结论不等完整HMR已交付。

本轮只登记M4目标并更新路线/索引，不改变M2验收或现有Plan依赖，不迁目录/升级SDK/构建运行/创建agent。


## 6. RQHost接缝（2026-11-05新增设计关联）

用户要求未来ACE替代VM/a2r、AGE替代当前AutoUI时仍支持App不自绘的RQ形态，详[完整RQ设计§8](../autoui/renderqueue-complete-fidelity.md)与[子路线](../autoui/renderqueue-refactoring-roadmap.md)。执行器和图形后端分别适配，surface/session/输入/资源寿命控制面稳定、绘图payload能力协商演进；当前DisplayList v2不冒完整3D/AGE合同。当前UI061–063/OS063完善iced/wgpu路线，不等待M4，不提前实施AGE生产融合；本页既有Schema/旧编译器替换进入门不变。未来M4合同须包含direct/RQ语料对照、两类现行Host及Linux compositor实际支持面、版本并存/回退，不能只把AGE嵌入独立窗口就关闭RQ义务。
