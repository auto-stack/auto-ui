# 3D角色、运行相机与Gallery三维迁移

2027-11-01，设计v1；本次仅规划，不实施。承接[作者模型](declarative-authoring-model.md)、[024基础与025服务设计](node3d-foundation-and-generic-presentation.md)和[逐入口路线](gallery-node-authoring-migration.md)。设计/引擎合同归auto-ui，内容合同归ag-examples。

## 2026-11-02当前交付与后继

027实际交付f3dea0d，Spec SHA95a86660dfa15a09206baac45d8901a9f515fbe871cbdc676a210afed14ff621、独立final/冻结/归档清理齐；025交付e9cc929并清理。AG010交付8bfddca，源码/metadata当前11/22（原21迁10＋新增HUD1），剩11。AG011用户告知开发中，已核见唯一ag-011树，合同任务进度待owner同步，本轮不接管或翻状态；下文首次规划的0/10/9/22和“等027交付”仅为历史拆分输入。后继见[UI028与AG012设计](node3d-instances-triggers-and-gallery-motion.md)：012固定已交付027迁G07/G08-3d，028补三维实例/触发/新局为G11-3d前置，各0/10、暂不实施。

## 已交付事实与本轮拆分（首次规划快照）

auto-ui基线98cbcd2，024已实际交付64166db，canonical node3d-foundation.md SHA256 9451cf0f3c6a82505caa771b54728f1a6fb06a2ff74343052dd164dd2aee155a。025有唯一ui-025树在途，不因旧导航drafting推断无人实施。ag-examples基线4784d0c：009交付062081d、006交付e574b58，均独立review/merge/清理；010为executing、2/10，有唯一ag-010树并固定022（含修复c6aa63a），不追本轮引擎tip。

Gallery当前12逻辑项/22入口；原21入口已迁8个，另新增HUD一个，共9/22为新作者形式，剩余13个。新增计划不计交付。

| 合同 | 所有权与交付 | 实施前置 |
|---|---|---|
| [auto-ui/027](../../plans/archived/027-autographics-node3d-character-and-runtime-camera.md) | 公共3D角色/运行相机Node接线；必要的原内置3D模型声明透传；只写引擎与永久消费夹具 | 已交付020/024；不等待025/AG010 |
| [ag-examples/011](../../../../ag-examples/docs/plans/011-autographics-gallery-node3d-basics.md) | 仅G02/main、G03/3d迁移，普通use、资源symbol、组件及原主/父移/隐藏变体；保完整G02/G03包 | G02可先固定024先证；G03及整体交付硬等027内置模型支持实际交付 |

两份均drafting r1、0/10。不得把“可以规划”写成“已授权开工”。025、027共享AST/load/translate/CLI，010、011共享Gallery工具/注册/包，只在各自独立树开发并按实际owner短窗串行接线。GPU/窗口/音频重型门同机单实例，OS键鼠/焦点自动化继续暂停。

## 保真缺口：G03模型不是圆柱

G03旧scene使用Kind::Player和Kind::Mob，host.rs映射到既有内置模型；024仅提供cylinder3d/model3d/group3d，其层级夹具用了圆柱。它证明层级规则，不证明原几何等价。原G03 scene SHA b20e9723fc4a94a77a803ff3793ba6f1de0e214a4bedd2e7711c78948bded185；两个内置GLB的SHA分别为05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942（52392B）与8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c（63828B）。它们含doubleSided与roughness=0.5，不在024既有plainGLB域，不能当普通平色外部GLB导入。

027须提供具名内置视觉资源/typed引用到原Kind的声明映射，复用原渲染器、模型与许可来源；它与物理character3d分开，静态Player/Mob无需角色/相机/VM。新显式模式保024冻结输出和负控，不静默拓宽旧profile；不换圆柱、不重编码资源、不放宽plainGLB导入器。011只能消费实际交付的这一能力，不能在内容仓补引擎。此证据修正了“G02/G03都只依赖024”的初步预期，原内容/验收优先保持。

## 027的作者与运行边界

复用020/024普通use、固定Node组件、局部model/msg/on、Life.Start/Tick、typed事件与view-3d；移动/重力/跳跃/输入组合均由作者Auto决定，编译器不按Player等业务定义名合成控制器。Node组合与显式group变换分开，声明树不是VM单文件限制或未来ECS约束。

新增明确选择的3D角色档和运行相机档，复用已交付character3d/1、runtime_camera3d/1。G07固定方向基且无运行相机，G08/3d才有相机相对运动。类型区分scene stable ID、从零的row handle、Run角色引用与move token；token0初始支撑有效。角色位置由move拥有，不能视图绑定再次place，solid冻结后不能间接移动。一个物理角色不等于一个Node实例，纯视觉组件可复用多实例但沿原8有状态组件包络。

原预算全保：一个3D角色、共享32空间credits/callback，六动作每held/pressed各1、move26、grounded/support各1；四held+一次Jump+move+grounded=32。相机basis/orbit读本来不占空间池，不等于其它getter免费。128稀疏/宽布局与64密集重叠是已批准包络，不保证任意128布局；127 solid初始化拒绝仍是反例。VM/CPU、100000 steps、120 resumes、250ms、8ms目标与资源/512潜在mesh/128scene/8有状态Node各账独立，不因拆组件放大预算。

按原输入/接触/支撑/相机配置、Pause/Step/Stop/fresh Run、pointer清理、同Run/tick frame+camera共同发布及Document/Undo隔离验证。保G07两布局11项、G08/3d13项原oracle，原G08斜向二次n、alt的speed5/jump7/gravity18/ortho16均按真实源码保留，不借迁移纠正公式。

## 011的内容与闭包

G02保1528B原GLB SHA42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671、红绿部件/原MIT与遮挡；G03保原内置几何/材质、group20/21、深层world(1.8,1,0)、父x+2→3.8、隐藏祖先、Stop不改文档。保G02四原require+坏GLB一项、G03六原require族共十一义务，并增加把Player/Mob替成圆柱必须失败的反例。

两个静态入口不强制用户VM/假角色/Tick。G03/2d继续固定019，包内按子入口选锁，不能整项跟升；G02可用024兼容模式或已交付027中的冻结兼容能力，T-02冻结实际source-build/CLI。生成JSON仍是生成物，use模块和具名资源才是活作者源。

完整G02、G03单项离线包含全部子入口/变体、模块与资源许可闭包、源/工具/SDK681/依赖锁/构建身份、收据/manifest/必要验证工具。实际公开启动、A→B真Run、保存关开/冷再生、坏件拒绝→精确恢复→同入口再Run；旧exact锁严格保留，不用任意当前runner或字面SHA冒充来源。引擎绿不代替Gallery接受。

## 后续与记录规则

027实际交付后另立G07两布局/G08-3d内容合同，再按证据规划3D实例/触发→G11/3d、三维查询、纹理/动画、编辑器教程；本轮不预取这些编号。025通用服务收口不自动重开006，当前M2完整游戏/预算证明仍pending，ACE/AOT/ECS/跨端不是新增前置。

只新增上述两份Plan、导航与路线簿记。canonical Spec增量在各Plan中提案，独立review通过并merge后才沉淀。不创建实施树、探针/源码/资产、构建、GPU或agent。未来材料唯一plan-artifacts/plan-NNN/<repo>/，开工登记owner/用途/在途，merge摘要折Plan后清理自有scope及全部实际输出，仅具名未解决问题持有；禁止链接/attachments/派生JSON，清树先fresh guard。
