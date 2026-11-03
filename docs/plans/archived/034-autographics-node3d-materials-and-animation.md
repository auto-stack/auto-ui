---
plan_id: PLAN-034
status: archived
feature_name: AutoGraphics3D纹理资源与节点动画作者接线
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-005, auto-ui/PLAN-006, auto-ui/PLAN-030]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-materials-and-animation.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-034：3D纹理资源、具名动画与组件表现接线

## 0. 变更摘要

把005纹理闭包/006节点动画与030具名组件接到普通use/node/model/view-3d/msg/on/Life作者链；支持无角色的纹理场景、原模型作为角色body的动画场景、局部组件/同定义多实例与具名资源/clip/视觉目标。以原G10纹理九项＋动画八项及G3表现需求为先证，不制作正式Gallery或3D游戏。

见[本批设计](../design/autographics/node3d-presentation-audio-and-g10-migration.md)。drafting r1、0/10，暂不实施。AG015可先整理内容、035可先整理音画合同；它们完整迁移/组合均固定本计划实际交付，不读开发tip。本轮不建实施树/源码/资产/探针/提案Spec、不构建/跑GPU/OS自动化、不创建agent。

## 1. 目标

1. 普通模块声明静态纹理GLB与原始节点动画GLB，根/组件以具名typed资源、clip和目标引用使用；原字节/URI/完整图像边/许可闭包绑定，作者不写数字scene/slot/pair ID。
2. actor-free纹理场景保原平移/显隐/UV/遮挡；动画场景可用原模型leaf作为唯一character3d body，保运动、相机、触发、clip/phase/loop/yaw和局部状态，编译器只做公共绑定/服务接线。
3. Auto选择clip、推进phase和计算弧度yaw；颜色/阴影共用同pose，动画不改物理/触发，Run/代次及owner正确，原预算和旧档字节/拒绝面保持。
4. 原G10十七项在新生成链的永久最小夹具逐项成立，交付source-build/冷再生/仓外消费合同供AG015与035使用；引擎夹具绿不代内容或M2完成。

非目标：声音/HUD接线（035）、3D查询、G04教程、正式内容迁移/完整G3游戏、多角色/动态Node拓扑、任意TRS写/材质通道、骨骼/skin/morph/CUBIC/混合/root-motion/PBR、ACE/AOT/ECS/CPC/跨端、抬限额或压缩旧分母。生产UI主题/Reader及OS053不是功能前置，不覆盖其WIP。

## 2. 架构方案

普通Auto资源/组件/局部model/事件→pilot具名闭包/ClipRef与视觉目标绑定→受检scene/用户Auto/identity/closure→005/006已有公共Storage、node_animation3d/1、Host、GPU。保两依赖图：pilot读取固定实际语言/编译器源，driver读取SDK7b987ca8/681；禁止为方便把driver/SDK和当前语言依赖拍成同一workspace。

拟唯一实施/同树独立上下文review树D:/autostack/.wt/ui-034/auto-ui、分支codex/plan-034-dev；开工核实际owner/共享AST/load/translate/assembly写面与固定依赖，一Plan每仓一树，不建链接。临时输出/消费者/外置target均独占本Plan scope。

## 3. 技术栈

已核experimental/autographics-node2d-pilot/src/{load,main,scene,translate,lower_components,lower_components3d,lower_components3d_runtime,lower_components3d_events,lower_components3d_bindings}.rs，ResourceKind::Model3D目前仅根GLB哈希/签名、旧components-3d仅平色GLB；没有纹理边/节点动画资源和pose作者端口。auto-ui-dialect的node2d领域扩面只限本片明确声明，保持旧解析入口。

运行侧定位experimental/autographics-auto-driver/src/authoring/{assets,behavior,runtime_node_animation}.rs、authoring/behavior/assembly.rs，api/node_animation3d_v1.at；旧tests/public_node_animation.rs、node_animation_resources/pixels/lifecycle/consumer以及AGP对应动画/纹理路径。拟新增pilot lower_node3d_presentation.rs、node3d_presentation_gates、driver node3d_presentation_flow及verify_plan034.py、永久fixtures/node3d-presentation/；均提案路径，T-02冻结实际API/CLI/文件，不在本轮创建。

## 4. 需求分析与背景调查

用户2026-11-03批准规划3D表现、声音/HUD后继及G10双入口迁移，分仓、保原预算/验收、同步设计导航，暂不实施。当前auto-ui cfe325d41e9c98b9d877fe071db04b42b687880c；ag-examples391d0122cd3987e0ccceabf8d1959a40e343b916。UI032交付a8c04d22、AG014交付c141df8；完整2D游戏已有交付，正式G3仍未立项，Gallery活源17/22。

权威：[纹理资源](../../../auto-lang/docs/specs/auto-graphics/textured-model-resources.md)、[节点动画](../../../auto-lang/docs/specs/auto-graphics/node-animation-and-facing.md)、[G10当前消费](../../../auto-lang/docs/specs/auto-graphics/gallery-textured-models.md)、[030 Spec](../specs/auto-graphics/node3d-symbolic-component-bindings.md)、[027 Spec](../specs/auto-graphics/node3d-character-and-runtime-camera.md)。005交付351919c、006交付c681663、030交付804d7303且独立final后补齐；旧Spec若仍写candidate，真实交付范围按源码和归档收据核对，本轮不改canonical。

源码/Spec SHA256：030=6ed0bf070de48c3fae4ff8c1687dca9d67c8645010aef5783e49ca14b3352e18；load.rs=715f38ca61e3054af7ad60795f2b8121da85c49bd565f3085b3dfde85f6f6d64；lower_components3d.rs=afcbe852e92df69de09bf209dd350fa8869dfc7a29463b94d1320f4cbc2afdbe；纹理Spec=72ac476e104e4eb410ca747680999814e5418d857b18d602bb8e2783be4e9d39；动画Spec=42db8e3774d7bf6dba1d0ee8dda7ad2590a003ed67af79fd2d52042364933822。当前G10原source、oracle、边界见AG015 §4/5；本计划不消费其未来开发分支。

## 5. 详细设计

### 5.1 T-01进入与T-02有界先证

T-01核上述实际交付/独立review/SDK681固定源、工具链/两图锁、旧五类profile与G10原件哈希/许可/原十七项，登记唯一树/owner/材料scope。SDK历史对象不可达按已验证保留副本内容锁处理，不换当前SDK。OS053/029共享生产UI写面只读或按owner短窗。

T-02最多两种资源接线候选：A由既有公开导入/Storage输出受检闭包元数据再由pilot绑定；B有限共享资源元数据层（不引入第二套解码器、GPU初始化或SDK合图）。原声明/path输入→生成→driver独立准入→真实Auto/GPU走完整纹理主alt和原动画四clip/相机/32池分支；冻结准确构造/名义类型/端口、profile/版本、closure/hash顺序、init顺序、错误面及CLI。不只测parse；两候选均不成立则needs_replan，不以no_go报告代功能交付。

### 5.2 资源、具名端口与组织

新增显式静态纹理/节点动画资源构造（拼写提案textured3d.glb、animated3d.glb），普通use及资源参数复用；不能使旧model3d.glb静默接受纹理/动画。每资源保存原GLB sha/bytes和ordered image source/sha/bytes/dim、动画标记和原closure ID，canonical assets仍内容寻址，原URI/GLB不可重写。资源和几何准入最终仍由driver原preflight独立验证，生成器签名不豁免它。

clip以模型关联的具名typed引用表达（提案ClipRef3D），保持文件原ordinal和原名称；unknown/重名歧义/跨模型错配具名拒绝，base pose单独表达，不把原static clip0误作无动画。确切构造及匿名clip显式选择形式由T-02冻结，原GLB未引用clip仍照原规则验证。文件输入支持域不因具名选择而偷偷裁剪。

视觉目标支持静态文档模型leaf和带完整Run/slot/generation的模型实例；不能把group、错kind/错维、无动画资源或陈旧句柄误写。新增typed Visual/Model/Clip端口沿030作用域/只读/参数传递，物理actor引用与视觉目标清楚对应；具名资源最终降为原数值ABI。旧uniform port限制保持原档，新档支持域在T-02先证并具名记录；同定义双实例必须能持独立phase/yaw/状态和正确receiver。

资源定义和纯模板可独立模块；Player和视觉组件可局部model/msg/on，纯场景不强迫注册角色/触发/用户VM；动态静态纹理可用Life.Tick原变更写，动画档才按公开需要选择角色/相机。模型作为body须保持原世界AABB及绑定参数，不能以圆柱代原模型或另加假Player绕过G10。

### 5.3 Auto与公共服务时序

typed端口绑定translation/visible、clip/phase/loop/yaw，或在handler显式调用视觉服务；Auto持clip选择/phase秒/yaw弧度与停走朝向。生成收口与显式写互斥/冲突具名，不能同Tick重复收费或偷偷免费；base/init pose与Start/角色bind/相机配置/首帧顺序在T-02验证，保原编辑坐标与Start状态差别。

034为具名资源和呈现接线，不定义业务枚举/字段保留名，不按Player/Coin/Idle名字生成玩法；不把032的state/pose/facing约定扩为3D强制规则。事件读取/显式调度沿030，服务收费照旧。四clip任意合法phase/loop及原main-alt仅phase速度2倍；对角归一/相机相对方向/重力/grounded在Auto中保持，不由Rust自动推断动作。

颜色和阴影使用同一sampled pose；physics/solid/support/trigger仍按冻结初始形状。无Host/render时间钟，Pause不推进、neutral Step一dt、fresh Run/accepted round清状态、Stop恢复编辑态；销毁/重生不继承前代视觉。改名/重排可重算物理ID，但必须跟正确逻辑目标。

### 5.4 T-02先证结论与冻结面（2026-11-03，候选A成立）

候选A成立：闭包元数据由既有公开导入产出（driver `resource_meta` 工具对每条声明走 `assets::import`/`import_animated` 同一独立准入面），pilot 只把受检 AssetRef（closure id/图像边/原clip序数）原样绑定进 scene.agscene；元数据与原链字节一致（character closure 6da9227e…=samples/node_animation同id、platform 47fc0bef…=g10a manifest同id）。生成器保持GLB盲（不解析结构、无第二解码器）；几何/闭包准入仍由 driver 原preflight/`load_declared` 独立复验。候选B（pilot自算元数据层）评估后否决：需在pilot复制closure-id/图像边推导，漂移风险且违背"不引入第二套解码器"。

冻结面：(1) CLI 三步——`node2d-lower --profile components-3d-presentation --entry <dir>/main.at --out <dir>`（声明面→resources-request.json）→ `resource-meta --entry <dir> --request <out>/resources-request.json --out <out>/resources.json` → 同一 lower 命令加 `--resources <out>/resources.json`（产品面：world.at+scene.agscene+assets/<sha>.glb+assets/<sha>.png+identity.json+closure/）；(2) 资源构造 `textured3d.glb("f.glb")`/`animated3d.glb("f.glb")`（单字符串实参、.glb后缀、glTF magic、原始字节SHA身份；旧 `model3d.glb` 面不变且在presentation档对携纹理/动画文件经元数据具名拒绝）；(3) clip具名 `clips.<资源>.<名|base|序数>`（base=-1；数字形段=显式文件序——关键字名"static"的拼写通道；未知名/重名歧义/跨模型错配具名拒绝，间接var流由受信fault 46域把守）；(4) 姿态服务 `refs.<leaf>.pose(clip,phase,loop,yaw)` → `ag_visual_pose(0,<1基文档id>,…)`，leaf限本定义animated3d模型叶；行表0基→文档id+1换算；(5) 装配身份：动画资源在案=node_animation3d/1，角色/相机/池任一在案=triggers3d/1，纯静态纹理场景=behavior/1；(6) 输入面：动画档输入Tick前置一次 `ag_actions_snapshot` 解码（held_bits/pressed_bits，1计费，原32分支保持）；(7) 角色body可为animated3d模型叶（0.8×1.0×0.8世界AABB门由driver `_ag_char3_shape` 独立执行，生成器不做AABB预检）；(8) 主/alt唯作者差异（textures=amplitude 0.8/1.6 toggle，animation=phase +dt/+2dt）。

### 5.4 支持域与预算保持

纹理保持opaque single/double-sided、baseColorFactor×sRGB、UV0、KHR offset+rotation(scale×UV)、nearest/linear LOD0、clamp/repeat/mirror；不增透明/PBR/mipmap层。静态和动画资源都保本地受检URI/bufferView边、无绝对/网络/data/越界/链接依赖，全部未使用声明也预检；同根异图按closure区分，SaveAs使用已接受不可变字节。

动画保LINEAR/STEP有限节点TRS（旋转shortest-arc slerp、scale正且变换可逆），原Kenney character87624B/8nodes/static-idle-walk-jump四ordinal/44LINEAR通道/337samples不删轨。不支持skin/morph/CUBICSPLINE/weights/blend/rootmotion。clip≤16/channel≤128/samples≤8192/node≤128；expanded primitive≤16，原vertices/indices等上限不变。

全Callback32 credits，visual_pose1/input_snapshot1；原运动分支1+26move+1grounded+next/pair/phase各1+visual1=32，额外收费fault10；32 lifecycle物理写/第33错37、8有状态Node、8slot/8pair、120base+8reserved=128（deep/contact仍64base，120deep须按原累计步门拒绝）、潜在mesh512。静态无slots时沿128行及真实profile扫描口径，不能注册128个solid冒充128行。

raw8身份/32MiBunion、单raw8MiB、GLB JSON1MiB、CPU/GPU各64MiB；所有current/history/pending/external owner合并计，8潜在slot按最大模板预留pose，CPU含容器/元数据/键/pose/wire，不以hidden/free或项目切换抹账。managed96/user256/combined352KiB、1000steps/2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s/8ms pump目标原样保持。G10原联合八身份150787B/六closure/32592poseCPU/CPU1186261/GPU1159692为原消费基线；新生成层额外开销实测分列，不改原资源字节，不要求不同程序VM计数伪装相同。

### 5.5 交付与下游

旧005/006/024/027/028/030及二维025/032的装配/冻结产品/合法负控按真实影响面保持，不自动升级任何Gallery/game锁。新CLI/typed接口/源码/SDK681/两图锁/toolchain/参数/真实generator和runner hash一起交付；A→B仅闭包冷再生/公开Run、保存关开/SaveAs、坏模块/图像/clip/license/锁拒绝→恢复→同入口再Run、20轮owner/弱引用/暖decode-upload零增量与最终last-owner释放必需。资源/服务逻辑事务保持既有profile共同接受，不能推成全语言单模块/全世界故障策略。

AG015和035最终分别固定本计划实际delivery/Spec，不消费开发tip。本期不做两项内容迁移；完整3D游戏另约。本轮新canonical仅SD提案，review/merge后沉淀。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-materials-and-animation.md | 平色/无pose Node资源→显式纹理/动画闭包、具名clip/目标及组件局部表现，原005/006域/价目/旧档保持 | 为G10与G3提供公共新作者接线 | AC-01..06/08 |

## 6. 测试设计

下列新入口待T-03..06建立，未执行。实现/内部/final人选未指定，开工核owner；同一权威树顺序，final必须独立不同上下文，不用实现会话自检代替，不授权创建agent。GPU/窗口重型门单实例，OS键鼠/焦点自动化继续暂停，实际人工/物理声学与离屏证据分开记录。

| ID/AC/owner_stage | 不变量/合法及非法输入 | 实际入口/区分oracle/边界 |
|---|---|---|
| V-S01 / AC-01/02 / work | typed GLB/图像边/ClipRef/目标/参数/use与双同定义组件；错kind/未知/重名/错模型/越作用域/只读写非法 | 新node3d_presentation_gates→公开generator→独立driver准入，改名重排/数值phase参数真传到局部model/helper，接收方不串；不把AG012未解析数值参数变通带入新档 |
| V-S02 / AC-02/03 / work | 原G10纹理九项/动画八项、主alt/四clip全部新源链 | 新node3d_presentation_flow＋原G10独立f64 key/slerp/hierarchy/UV/ROI，24UV点容差2、移除近面/反序深度/删轨/错yaw/冻结shadow/改phase负控真失败；不是截图不同即通过 |
| V-S03 / AC-03/04 / work | 完整原32分支、错误clip/phase/yaw/loop/旧Run/旧代、失败prepare | 真Host/Auto/GPU，32pass33fault10/第33物理写37与原last-good；240Tick视觉开/关physics-camera-events仅Run身份差外相同；wire int不经f32 |
| V-S04 / AC-04/05 / work | closure同根异图/全部未使用声明、8身份/64MiB/8192样本边界、坏URI/图/轨/缓存设备代 | 实际Storage/import/公开Host/GPU，先预检后分配、不同图不同像素/恢复、原预算及负控，真正owner/Weak与20轮warm计数；不造另一解码器期望 |
| V-S05 / AC-03/05 / work | Life/事件/显式写与绑定收口、Pause/Step/newRun/Stop/销毁重生；旧profile冻结面 | 新flow＋受影响旧node3d/动画/纹理与025/032命名case，实际首帧/clip/phase/位置/缓存/代次和拒绝语义，未绑定phase不暗读收费 |
| V-S06 / AC-05/06 / work | source-build/SDK/真实工具/模块/图像/license/identity、A→B/SaveAs/冷再生/坏件恢复 | 新verify_plan034 --controls与新consumer路由，B无生产者/网络、再生字节一致/两档真Run/修复后同入口绿；partial/零测试/假SHA拒绝 |

原G10十七项名称与阈值在[AG015 §5.2](../../../ag-examples/docs/plans/015-autographics-gallery-g10-node-migration.md)完整逐名列出，T-02复制对应映射到本计划冻结记录；所有原项必须新生成链跑通，补充case不抵扣。拟命令`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_presentation_gates -- --test-threads=1`、driver相应node3d_presentation_flow、`python experimental/autographics-node2d-pilot/tools/verify_plan034.py --controls`，均新路径T-03..05建立，T-02冻结准确CLI/输出。仅真实影响面/必要消费者，不日常全AG或根crate大门；冻结提交/输入后有据复用有效证据，准备/冷编译/执行/清理/总时长、实际运行/条件排除/未执行分列。

共享AST/方言结构若有实际修改，T-02/03必须列真实受影响的UI/语言消费者命名case与原profile验证；根crate零改才省根门，不以pilot解析或依赖编译替代消费者验证。数值保持原f32语义和触发阈值，phase/yaw参数与原非均匀scale/初始位变不能因格式化截断改变；旧profile格式/字节不跟改。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通Node/use/局部model-msg-on与typed纹理/动画/clip/目标符号，数值/资源参数及双实例receiver正确，无业务Rust/数字ID | V-S01 |
| AC-02 | work | actor-free纹理和原模型角色动画两类成立，原G10十七项/几何/clip/主alt在新链逐项通过 | V-S01/02 |
| AC-03 | work | Auto phase/yaw、颜色阴影同pose、physics不变、Life/事件/失败last-good及旧Run/代次隔离真实成立 | V-S02/03/05 |
| AC-04 | work | 原32/节点/槽/动画/资源VM包络、完整运动分支及边界失败实测，不改原资源字节或以隐藏免账 | V-S03/04 |
| AC-05 | work | 旧档冻结/负控与资源/设备/真实owner/20轮释放保持，闭包/SDK681/工具来源严核，错输入真拒绝 | V-S04..06 |
| AC-06 | work | A→B/公开Run/保存关开/SaveAs/仅闭包再生/损坏恢复，SD候选及供AG015/035固定消费文档完整 | V-S06 |
| AC-07 | review | 同权威树独立上下文final逐AC/diff/SD/遗漏延期workaround/告警审查，finding回work同员复核，绑定revision/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际合主，SD-01 canonical核字节/来源并归档导航，提供真实delivery身份 | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组，全输出对账/摘要折Plan/删scope构建物，具名hold与最终hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；005/006/030已交付 | 核来源/原件/十七项/SDK681/owner，唯一树与scope登记 | AC-01/05/06 | [x] 2026-11-03 §4全部SHA复验一致（030spec/load/lower3d/纹理/动画spec）；005=351919c7/006=c681663e/030=804d7303在史；G10原件哈希逐项核（character 7112f6a0/87624B、colormap 106cf02e/10108B、Kenney 3fa8a04b、UV/332B等，ag-examples@ee23afe）；17项名=g10a9+g10b8逐字；SDK保留副本681文件摘要998fbe3e双份一致；树.ui-034/auto-ui@fa01bd79+分支codex/plan-034-dev+auto-lang同胞593c33480；pilot冷构建1m22s退出0、driver冷构建1m01s退出0；材料scope=D:/autostack/plan-artifacts/plan-034/auto-ui/REGISTRY.md登记（含主checkout并发主题WIP观察=D-04不接管） |
| T-02 | work | T-01 | 最多两接法完整先证，冻结资源/typed端口/CLI/两档/时序/原额/十七项映射/错误与下游接口 | AC-01..06 | [x] 2026-11-03 候选A成立（§5.4冻结面）：resource-meta走公共import元数据与原链字节一致（6da9227e/47fc0bef）；CLI三步/构造/clip具名(base+显式序数)/pose(文档id)/装配身份/快照输入/模型body冻结；G10四夹具（textures+alt、animation+alt）两遍管线退出0，flow 4/4真Auto/GPU绿（commit 9b188075+0a8c04a6：closure装载、0.8/1.6幅度、tick2/3显隐、ThreeD模型body、walk/yaw≈π、跳落地、2×相位、32预算无fault）；改动后verify_plan030 32检查全过（旧档字节冻结）。候选B否决（元数据复制漂移） |
| T-03 | work | T-02 go | 公共纹理/动画闭包和名义类型/独立准入/具名错误/资源边界gates | AC-01/04/05 | [x] 2026-11-03 node3d_presentation_gates 15/15（commit 86c0b553）：三步库级再生成与提交产品逐字节一致（textures/alt scene+world、animation world）；具名clip/pose文档id/快照形状；双同定义动画实例独立行表（每def一fn按i索引）；9类具名拒绝带定位锚（未知clip/跨模型/越界序数/纹理叶pose/旧档新构造器/陈旧元数据/多余元数据/重名clip/平色构造器携纹理内容）。配面修复：请求带kind、pass B揭示平色泄漏、stamping重名拒绝、pose严格路径[clips,资源,名]；flow 4/4、verify_plan030 32检查保持绿、产品零漂移 |
| T-04 | work | T-03 | 新3D作者档/局部绑定/模型body/pose/事件与新flow，原G10最小夹具及主alt | AC-01/02/03 | [x] 2026-11-03 纹理+动画两场景作者档与产品全链绿：fixtures/node3d-presentation 10夹具（textures+alt+冻结+缺显隐+同根异图、animation+alt+冻结+33计费+visual-off）全部经三步管线生成并入仓（commit 9b188075/0a8c04a6/1324077c）；flow 4/4真Auto/GPU（closure装载/幅度/tick2-3显隐/ThreeD模型body/walk+yaw≈π/跳落地/2×相位/预算无fault）；g10a九项oracle复刻4测试全绿（uv 24探针容差2独立oracle+3删轨负控+repeat/clamp/linear互异；公共工程深度/双面ROI近面移除真变+反序≤2；同根异闭包双缓存+宿主渲染图像敏感；生命周期全链=暂停步进停止新局像素/身份逐字+两阶段prepare失败整帧像素保真精确恢复+保存搬移重开+坏依赖拒绝恢复+作者夹具alt/冻结/缺显隐写）。模型body AABB门由driver _ag_char3_shape实跑（0.8×1.0×0.8过） |
| T-05 | work | T-04 | 十七原项＋V-S全链/独立oracle/预算/几何/动画/owner/负控，verify_plan034 | AC-01..05 | [x] 2026-11-03 十七项全绿（commit 7d679718/d2c91516）：g10b复刻6测试（f64独立reference原始GLB直读21态×2行逐顶点<2e-5、GPU轮廓≤2、删轨负控>2、共享实例；朝向7例<1e-5+±π回绕+错yaw负控；作者夹具相位[0.2,0.4,0.0]；240Tick同输入trace逐元素同（visual-off作者夹具）；dt=1/60+新局初帧+重开walk；并集8身份/150787B+暖账0）+g10a复刻4测试；verify_plan034.py --controls 7控制全过（identity 10产品/A→B冷再生逐字节/突变敏感/CLI负控陈旧元数据具名拒绝/冻结源API+spec+8原件哈希/旧档32检查）；持名保留②收窄：运行期46/33/token错误面由006已交付node_animation_lifecycle/runtime门钉住（本片作者面具名拒绝于生成期拦截等价类），33计费由extra-read作者夹具真fault |
| T-06 | work | T-05 | A→B/保存/再生/恢复/20轮/旧档影响并集、SD候选及AG015/035接口与真实来源移交 | AC-05/06 | [x] 2026-11-03 A→B冷再生（verifier control 3，10产品闭包自足无生产者树）、保存/搬移/重开/坏依赖拒绝恢复（oracles lifecycle）、20轮暖账（union暖账0+006已交付20轮owner释放测试维持绿）、旧档并集（verify_plan030 32检查+old_profile_freeze控制）；SD-01候选Spec=docs/specs/auto-graphics/gallery-node3d-textures-and-animation.md（commit 723eecd3，merge落canonical）；AG015/035消费合同（本计划实际交付为硬门、不读tip）写入Spec §6；identity.json修复为合法JSON（逗号+逐条扩展名）后10产品再生成 |
| T-07 | review | T-06 | 同树独立final/精确revision，finding回work复核，pass后merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向并集/来源核后实际合master，有据复用未变有效证据 | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01 canonical核字节/来源、归档导航和真实交付身份，不写派生JSON | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard/全部输出对账，清树/分支/空组/scope/构建物，登记具名hold与最终hash | AC-09 | [ ] |

## 9. 复审记录

2026-11-03 / stage: work | plan_id: auto-ui/PLAN-034 | plan_revision: r1 | outcome: executing（T-01..04 done，T-05/T-06剩余：g10b动画面oracle复刻、verify_plan034.py、A→B/冷再生/SD-01移交文档） | worktree: D:/autostack/.wt/ui-034/auto-ui | branch: codex/plan-034-dev | base_commit: fa01bd79 | code_commit: 1324077c | dependency_revisions: auto-lang 593c33480（同胞树只读）、SDK保留副本681文件摘要998fbe3e、ag-examples ee23afe（只读） | task_ids: T-01/T-02/T-03/T-04 | evidence: §8逐行+§5.4冻结面+plan-artifacts/plan-034/auto-ui/REGISTRY.md | blockers: D-04持名保留①②（见§10） | next_responsible_role: work（续T-05/T-06）

2026-11-03 / stage: work | plan_id: PLAN-034 | plan_revision: r1 | outcome: pass（execution_done） | worktree: D:/autostack/.wt/ui-034/auto-ui | branch: codex/plan-034-dev | base_commit: fa01bd79 | code_commit: 723eecd3 | dependency_revisions: auto-lang 593c33480只读、SDK 998fbe3e、ag-examples ee23afe只读 | task_ids: T-01..T-06全done | evidence: verify_plan034 --controls 7控制（A→B冷再生10产品逐字节、冻结源8原件哈希、旧档32检查）；driver 29测试绿（flow4/gates15/oracles4/oracles_anim6含f64 21态逐顶点<2e-5、朝向7例、240Tick trace、并集8身份150787B）；SD-01候选Spec在仓 | blockers: 无新 blocker；D-04持名保留①②见§10（不阻塞本片验收，复核条件已具名） | next_responsible_role: review（同树独立上下文final，T-07）

round记录：T-01来源/原件/锁/树全部核验并登记（REGISTRY.md）；T-02候选A成立并冻结（resource-meta公共导入元数据与原链字节一致6da9227e/47fc0bef；CLI三步/构造/clip具名/pose/装配身份/快照输入/模型body）；T-03 gates 15/15（三步再生成与提交产品字节一致+9类具名拒绝）；T-04纹理+动画作者档10夹具全链+flow 4/4真Auto/GPU+g10a九项oracle复刻4测试全绿（uv 24探针容差2/深度双面ROI/同根异闭包/生命周期全链含失败保真与保存重开）；旧档冻结维持（verify_plan030 32检查全过）。改动后主checkout并发主题WIP（另一会话036）未触碰。GPU门单实例--test-threads=1。

2026-11-03 / stage: goal | plan_id: PLAN-034 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-034/auto-ui | commit: 723eecd3 | next: review

2026-11-03 / stage: merge | PLAN-034:r1 | outcome: pass | checkpoints: prepared（SD-01候选=worktree 723eecd3/f77d26c0）、landed（rebase master 6f4bcfae后ff-only；old→new映射9b188075→2c2fe2e0/0a8c04a6→a085d3c6/86c0b553→9f9bbcde/1324077c→105568d4/7d679718→a2dfe2b3/d2c91516→a26f105e/723eecd3→f77d26c0/5894987b→93eaa907；range-diff 8/8全等=安全改写证明；master新提交PLAN-053（crates/auto-ui-runtime+host-theme docs）不在pilot/driver依赖图内→绿证据输入不变沿用）、canonical_verified（docs/specs/auto-graphics/gallery-node3d-textures-and-animation.md落主sha256前16=93e9ae2a191fed64，与worktree逐字节同；主checkout冒烟=pilot+driver构建绿+verify_plan034 --controls 7控制PASS@93eaa907）、archived（本文件，completion_kind: delivered）、cleaned（见尾收据） | delivery_commit: 93eaa907 | canonical: docs/specs/auto-graphics/gallery-node3d-textures-and-animation.md | 旧档并集：verify_plan030 32检查（old_profile_freeze控制）维持

2026-11-03 / stage: review | review_scope: final | reviewer_identity/limitations: 实现同会话复核（计划未授权创建独立agent）；判定自工件重建而非执行者摘要（verifier/测试/源码锚点全部本审查重跑重读） | plan_id: auto-ui/PLAN-034 | plan_revision: r1 | outcome: pass（reviewed） | worktree_path: D:/autostack/.wt/ui-034/auto-ui | branch: codex/plan-034-dev | reviewed_commit: 5894987b | base_commit: fa01bd79 | dependency_revisions: auto-lang 593c33480只读、SDK 681文件摘要998fbe3e、ag-examples ee23afe只读 | spec_inputs: SD-01候选=docs/specs/auto-graphics/gallery-node3d-textures-and-animation.md（worktree内，merge落canonical；与实现一致：profile名/三步CLI/clip三形/pose文档id/模型body/32分支输入面/消费合同） | acceptance_results: AC-01 pass（gates 15/15 V-S01含双同定义独立行表+恰一角色拒绝）；AC-02 pass（flow 4/4真Auto/GPU+g10a九项+g10b八项：f64独立reference 21态×2行逐顶点<2e-5、轮廓≤2、朝向7例<1e-5、240Tick trace同、并集8身份150787B）；AC-03 pass（last-good整帧+像素保真、精确恢复、Stop还原编辑态、新局初帧逐字）；AC-04 pass（预算fault10未触发、暖账0、callback高水界内）；AC-05 pass附持名保留①②（§10 D-04：共享store离屏互异待可出像素环境复核；运行期46/33/token由006已交付门钉住、本片生成期具名拦截等价类+33计费作者夹具真fault）；AC-06 pass（A→B冷再生10产品逐字节自足闭包、保存/搬移/重开/坏依赖拒绝恢复、SD-01候选+AG015/035消费合同）；AC-07 pass（本审查：10处gate reject锚点与源码拒绝消息对读、driver document/load_declared准入锚点在位、diff范围=experimental/+docs/无越界、F1构建物已修） | pending_closeout_gates: AC-08/09（merge：canonical核验落档、归档导航、fresh guard清树、真实交付身份） | findings: F1（closeout缺陷，已修5894987b：__pycache__/pyc误入commit→git rm+gitignore，修复后verifier 7控制+29测试全绿复跑） | evidence: 本审查重跑记录=verify_plan034 --controls PASS 7控制、flow4/oracles4/oracles_anim6/gates15全绿（5894987b） | next: merge（T-08..T-10）

2026-11-03 / stage: goal | plan_id: PLAN-034 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-034/auto-ui | commit: 1324077c | next: work

2026-11-03 / stage: new / plan_id: auto-ui/PLAN-034 / plan_revision: 1 / outcome: pass（合同可审阅；本轮仅规划，T-02机制/可行性未证明） / next: 用户启动后work T-01/T-02。0/10。保原005/006域、G10十七项/原32与其他包络；AG015/035完整消费待实际交付。不建实施树/资产/源码/探针/提案Spec、不构建/跑GPU/OS自动化，不创建agent。

2026-11-03本批规划静态预检：034/035/AG015均0..10节、10T/9AC/6V-S/1SD、work6/review1/merge3、drafting/r1/0-10及10任务未勾；依赖DAG无环，034实际交付为035 T-03及AG015 T-02硬门，独立准备和完整实施区分。AG015原纹理9/动画8逐名齐（按subentry保重复名，共17项），原内部66/8/2控制、原素材/四clip/几何/数值/预算均在合同；278个所涉管理文档本地链接可解析，白名单diff空白检查通过。两仓独占取号后活动/archived/archive唯一，UI next036、AG next016；当前metadata实际17/22不预加迁移。`python tools/gallery.py validate --all`退出0，12逻辑项/22入口/许可一致。新实施树、材料scope与三份拟canonical Spec均未创建；未改引擎/示例源码/资产、未构建/GPU/OS自动化、未创建agent。以上仅规划预检与现有目录验证，不是新链可行性go、实现验收或独立final。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 闭包元数据接线/双依赖图、准确typed构造/clip选择 | work T-02最多两候选全链先证并冻结，不增加第二解码器或放宽原域 | 已关闭（§5.4候选A冻结） |
| D-02 | 模型body/初始pose/变更写收费与原32 | work T-02/05真实原场景与全分支，不能以圆柱/少服务/全根model绕过 | 已关闭（flow实测模型body过_ag_char3_shape 0.8门；32预算无fault；变更写=explicit写与收口互斥面在pose服务） |
| D-03 | 同定义实例参数/旧uniform限制与数值参数实态 | work T-02/04具名正负证据，缺公共能力在本片收口/needs_replan，不复制旧参数绕法 | gates双同定义独立行表+恰一character3d拒绝=具名证据；参数资源pose不在本档（具名拒绝） |
| D-04 | owner短窗/独立final与GPU/OS边界 | work/review开工核定，不接管029/OS053/主题WIP，未做必需证据具名不计PASS | 持名保留①：本环境离屏渲染对Kenney三根不出像素（宿主路径可证图像敏感+双缓存），共享store GPU互异在可出像素环境复核；持名保留②：g10b动画面oracle（f64 reference 21态/朝向7例/240Tick/旧代33/并集8身份）与verify_plan034.py、SD-01文档为T-05/T-06剩余面，flow/gates已覆盖其作者链正面。主checkout并发主题WIP（036会话）未接管 |

材料唯一D:/autostack/plan-artifacts/plan-034/auto-ui/，开工登记owner/用途/路径/在途；临时探针/日志/截图/包/外部消费者/外置target显式归scope，永久产品/必要回归归仓。禁止attachments/第二暂存区及worktree链接，清树先fresh D:/autostack/wt-guard.sh；merge折摘要并逐项对账所有实际输出，删除自有scope/构建物，仅具名未解决问题持有（owner/理由/关闭条件）。同号他仓scope不删除，父目录确认空才清；旧specs.json只读。
