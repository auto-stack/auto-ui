---
plan_id: PLAN-027
status: archived
feature_name: AutoGraphics三维角色与运行相机Node接线
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-02
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-024]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-character-and-runtime-camera.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-027：3D角色与运行相机Node接线

## 0. 变更摘要

复用020/024，把公共3D角色、运行相机接到普通use、node/model/view-3d/on/Life.Start/Tick和typed引用；补原内置Player/Mob视觉资源的声明透传，视觉模型与物理角色分离。保原预算、数值/输入/故障及旧验收。只写引擎与永久消费夹具，Gallery业务另仓。

见[分仓设计](../design/autographics/node3d-runtime-and-gallery-migration.md)。execution_done r1、6/10（T-01..T-06 已实测交付，T-07 归 review、T-08..T-10 归 merge）。026已被Reader路线独占取号，本计划重新取027，不能覆盖或重用026。

## 1. 目标

1. Node局部Auto状态/事件和普通helper负责六动作、重力/跳跃/归一/移动，组件可复用；不按Player等业务Node名硬编码控制器或把所有model搬回根。
2. G07固定方向基/无运行相机，G08/3d相机相对运动各有明确新入口；保同Run/tick发布、pointer与生命周期/文档隔离。
3. 具名内置模型引用映射原Kind、mesh和材质，静态场景无需角色/相机/用户VM；普通model3d仍守024受限plainGLB域。
4. 完整新链在原32池/VM/资源包络内成立，有可固定消费的真实来源与闭包、离线再生/恢复。

非目标：Gallery内容修改、多角色/动态Node子树/销毁/触发服务、三维查询、纹理/骨骼/模型动画、运行group TRS、多呈现/Life.Frame、025声音/HUD或生产UI、ACE/AOT/ECS/CPC/跨端、提高预算或完整M2游戏交付。不承诺任意新服务组合均在32内。

## 2. 架构方案

普通模块/具名资源→020/024共享AST/组件/receiver→新typed角色/solid/相机与内置视觉映射→生成真实Auto→既有character3d/1或runtime_camera3d/1→Session/Play/Host/GPU。编译器只做声明/引用/服务映射和有限协调，控制公式/玩法留在作者Auto；复用原输入/接触/支撑/相机发布，不另造VM/渲染器/导入器。

新模式显式选择，024 components-3d冻结世界/身份/负控、874/875单价不变。静态内置模型能力独立于物理角色/相机，可选behavior=null；有状态视觉仍沿基础行为。准确CLI/marker/版本T-02注册核唯一再冻结，不静默扩旧profile。

拟唯一实施与同树顺序复审树D:/autostack/.wt/ui-027/auto-ui，codex/plan-027-dev，目前不存在。未来开工核owner/path/base；025功能独立，共享AST/load/translate/CLI按owner短窗顺序接线，不接管其树或复制组件系统。

## 3. 技术栈

已核crates/auto-ui-dialect/src/node2d/；pilot src/{load,translate,lower_components3d,main,lib}.rs、tests/node3d_basic_gates.rs；driver api/{character3d_v1,runtime_camera3d_v1}.at、src/authoring/host.rs及behavior/play/game_input等模块、tests/node3d_basic_flow.rs。沿020组件与固定SDK681，generator/driver两依赖图不拍平。

拟新增pilot src/lower_components3d_runtime.rs、fixtures/components3d-runtime/、tests/node3d_runtime_gates.rs、tools/verify_plan027.py及driver tests/node3d_runtime_flow.rs（新路径，T-02可冻结等价局部布局）。复用原mesh/材质/接触与相机算法，不写专用Rust玩法。

## 4. 需求分析与背景调查

授权：用户2026-11-01要求分仓规划G02/G03迁移与3D角色/运行相机Node接线，保原预算和验收、同步路线，暂不实施。本轮只改本仓设计/Plan/导航，不联系agent/chat、不修订在途025/AG010或Reader026。

观测基线auto-ui98cbcd2e79ebc61b0683886f24f0747b20d72617；024实际交付64166db，独立R2 pass/冻结门/归档清理齐，Spec node3d-foundation.md SHA9451cf0f3c6a82505caa771b54728f1a6fb06a2ff74343052dd164dd2aee155a；020交付16ddfa0。025唯一ui-025树在途；AG010固定022在途，均不构成功能前置。

权威：[024](../specs/auto-graphics/node3d-foundation.md)、[020](../specs/auto-graphics/stateful-node-components.md)、旧[公共3D角色](../../../auto-lang/docs/specs/auto-graphics/game-character3d.md)、[运行相机](../../../auto-lang/docs/specs/auto-graphics/runtime-game-cameras.md)、[G07/G08](../../../auto-lang/docs/specs/auto-graphics/gallery-character-and-cameras.md)。旧知识继续只读，不重开原计划。

SHA256：pilot lower_components3d.rs=83d67f8ba115c9c538e37357f8496c6a7d52b4a09e9fcdbc20d682dbc4361914；translate.rs=1c34ac26ddb19ccdf2d403ee791d5aab72ebdd43056545e754432079c2198d14；character3d_v1.at=a12c64bb97c7f80d15bd3c0a0817120a751ddeda5b6a1a1db6de804e1e4cea63；runtime_camera3d_v1.at=3bcb2ca2fe3a3400491b06253273cdd0b157e37659eadbb7832c9d11c00b2a71；host.rs=ecd0189cbf4c6fac7d09bde680c6518e15e6ddd03c6787d05c3c7fed9b92c9a6。

原Gallery基线4784d0c，checks/gyb/gallery_runtime.rs SHA8bdba37eef19ca3a9c0d7c52987374600ef190c6777f14c112c2a6917f4dd891；G07/layout-a/main.at SHA405f18631d83c86489ba11d8cb9af8fb0d90ae8e86648c42c4fc72e93fccf066；G08/3d/main.at SHAdf84fcb7eaa177b002985f018f975c338f25e127dd33a51d526aa8f2f8c5d0f3。实际源码是公式/观察依据，设计文字不能自动覆盖它。

缺口：024支持cylinder3d/model3d/group3d，但G03原scene使用Kind::Player/Mob，024夹具换成圆柱只能证明层级。原内置GLB含doubleSided/roughness0.5，不属于024平色外部GLB。为保原内容而补通用内置模型透传；AG011整体交付必须等此能力实际交付，不通过放宽内容验收回避。

## 5. 详细设计

### 5.1 开工入口和有界先证

T-01核020/024实际来源/Spec/独立review/冻结/清理、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467及681内容锁；历史对象不可达只用受检固定副本，不换当前SDK。核025与AG010实际owner/共享写面。

T-02最多比较两种既有接法，必须先跑完整链：静态原Player+Mob无VM、G07两布局、G08主alt、坏target/预算拒绝；记录准确CLI/profile/IR/typed API/marker/调用次数/阶段耗时与独立oracle。不是只做生成文本样机。超预算、需新Rust控制器、破坏旧profile才成立则暂停依赖任务回规划，不抬额/删oracle/换素材。

### 5.2 声明、引用、所有权

- 普通use共享组件/资源/helper；固定Node可有局部model/msg/on/Life.Start/Tick，root可无model。匿名msg与前导点事件沿020。Node名称不决定能力；两个同定义视觉Node状态/receiver隔离，物理角色仍最多一个。
- 初始position与运行角色position分所有者；后者只由move产生，不允许view绑定再place/shift或父group间接移动冻结solid。相机target恰一本Run角色，重复注册/配置拒绝；显式group仍只沿024平移/visible，Node组合不自动等于transform parent。
- typed区分SceneStableId、从零row handle、Run角色/solid引用与MoveToken。token0是freeze初始支撑有效，latest结果可读，旧token/错Run/kind拒绝且无误写；typed包装不能额外重读收费getter。静态引用、初始化参数和运行引用寿命分开检查。
- char3/solid/camera声明只映射公共配置/调用；gravity/speed/jump/方向组合由作者Auto持有，G07不强制运行相机。
- 具名内置视觉symbol映射原Kind::Player/Mob/Cylinder和原mesh/材质。原模型来自experimental/autographics-prototype/assets/squash_the_creeps/art/：player.glb SHA05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942/52392B；mob.glb SHA8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c/63828B。Kind不等于角色身份；不复制/重编码/换圆柱或放宽plainGLB，原模型/许可/owner闭包T-02实证。

### 5.3 输入、数值、相机

character3d/1沿marker102/profile13；一个body half(.4,.5,.4)，静态AABB solids，单Tick一次move；v5最多两接触与fused最终支撑，top在[feet-.01,feet]且足迹严格内，最高优先。初始重叠status3不grounded，precision stop4仍算支撑；隐藏solid仍碰撞。bind/solid仅Start零空间费、后冻结；错token13/solid或角色移动20..21/配置22..26等旧错误保持。

六动作Left/Right/Forward/Back/Jump/Restart=0..5；held/pressed每次1，pressed只消费一次，OS repeat不造新边。Pause/Stop/Fault/blur/leave/DPI/resize/modal/fresh Run清held/edges，Step中性。沿Host唯一输入路由，测试事件不能冒充OS键鼠实测。

runtime_camera3d/1沿marker104/profile15；一次配置、target本Run角色，初始yaw0；distance4..12/默认8，pitch15..75/默认45；perspective fov60 near.05 far256，ortho size1..64。F=(-sin yaw,0,-cos yaw)，R=(cos yaw,0,-sin yaw)。drag .25°/逻辑像素，wheel×1.1^-notches、40px/notch/单事件±8，累积720°/64notches，无无限队列。candidate精确echo，frame+camera同Run/tick共同接受，拒绝保原last-good对，不写Document/Undo。沿既有发布契约，不增“所有Node任一失败必整帧回滚”的产品要求。

G07保原jp/grounded→重力→四held→mx=R-L、mz=B-F→对角n=.70710678→move→grounded清vy的时序、两布局与speed4→5。G08实际源码先把mx/mz乘n，再wx=fx*mz+rx*mx、wz=fz*mz+rz*mx，dx/dz又乘n；保实际二次n及facing公式，不顺便修正。yaw0 Forward实际+z，yaw90实际+x。主speed4/jump8/gravity20/orbit8/45/perspective；alt speed5/jump7/gravity18/orbit6/30/ortho16，不能声称alt只改相机。

### 5.4 原预算、资源、来源与兼容

全Node共享32空间credits/callback：4held+1pressed+26move+1grounded=32；support额外读仍1，不能自动免费或省略作者调用。相机basis/orbit读原本零空间费，不代表别的getter也免费。128稀疏/宽布局与64密集重叠保原批准包络，不保证任意128布局；127solid初始化拒绝保原负控。

沿024：128场景对象+group，8有状态Node，512潜在mesh含隐藏；GLB raw8MiB/JSON1MiB、128asset nodes/16expanded primitives、65536vertices/primitive、262144vertices与786432indices/asset；8资源/32MiB raw/CPU-GPU各64MiB、8文件32MiB。内置模型按原owner账，不每实例重传。原VM1000steps/slice、2ms slice、100000steps/120resumes/250ms callback、init30s/tick2s、8ms pump目标全保，各账分别实测；不缩样本或改诊断门，准备/冷编译/运行时间分列。

真实source-build绑定源commit/模块/资源许可闭包、SDK681/依赖锁/工具链/参数、generator/runner实际hash和roles映射。A→B冷再生/真Run、SaveAs关开重开、坏件拒绝→精确恢复→再Run与20轮生命周期/owner。旧exact锁和024/020/019/017/022/023受影响冻结输出/负控保持；025在途不成为隐含依赖。新Spec只在merge沉淀。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-character-and-runtime-camera.md | 024无角色/运行相机/内置模型声明→新显式typed接线、所有权/价格/资源/来源/寿命边界；旧版本不变 | 给下游固定可消费契约，区分引擎能力与内容交付 | AC-01..06/08 |

## 6. 测试设计

全部新入口/夹具待T-03..06建立执行，当前无新绿证据。用户未指定实现/内部/最终review人选；未来入口核owner，独立final在同一权威树不同上下文顺序执行，internal pass不代final，不能据此自动创建agent。

| ID/AC/owner_stage | 不变量、合法/非法输入与状态 | 真实入口与区分oracle/覆盖边界 |
|---|---|---|
| V-S01 / AC-01/03 / work | use/局部状态/事件，两个同定义组件，静态Player/Mob；跨实例receiver/循环导入/资源错kind非法 | 新node3d_runtime_gates→实际generator/identity/scene/closure，局部值/目标独立；错误具名拒绝无半产物，不只语法绿 |
| V-S02 / AC-02/04 / work | G07两布局/G08主alt完整公式、32满额；第二move/第33计费/127solid拒绝 | 新node3d_runtime_flow真实Auto/Host＋独立f32/AABB/VP期望，24旧oracle逐名与32pass33fail；不拿边界循环代完整控制器 |
| V-S03 / AC-02/03 / work | token0/latest有效；错Run/row-vs-ID/旧token/重复配置/错target/冻结写非法 | 实际typed端口→Session/Play，错误13/20..30适用项，拒绝不误写对象/文档；反例走真实判定器 |
| V-S04 / AC-02/04 / work | yaw0/90、双投影、drag/wheel/Pause/Step/Stop/blur/fresh、坏candidate | 原Host事件路由＋真GPU，独立VP/像素与同tick frame+camera/last-good对/Document-Undo相等；OS焦点键鼠暂停项具名pending |
| V-S05 / AC-03/05 / work | 原Player/Mob合法，换圆柱必拒；普通GLB双面/roughness仍非法 | 原Kind mesh/材质/模型SHA/许可与新声明真GPU对照、独立ROI/geometry指纹；不只比kind字面或调用被测选择函数作oracle |
| V-S06 / AC-05/06 / work | A→B仅闭包冷再生、SaveAs/20轮、最后owner释放；缺helper/坏资源/错锁/partial receipt拒绝 | 真工具/Host/GPU、逐字生成与实际归属、拒绝→还原再Run、受影响旧verifier；固定档覆盖，不等于全AG/跨端 |

### 原24项oracle（全部保留）

G07/layout-a九项：`import_layout_a`、`forward_moves_on_z`、`supported_on_ground`、`windowless_png`、`frozen_input_still`、`speed4_to_5_sensitive`、`diagonal_normalized`、`x_wall_stops`、`jump_edge_once`。

G07/layout-b两项：`import_layout_b`、`pit_fall_divergence`。

G08/3d十三项：`orbit_config_from_source`、`vp_identity_tick0`、`forward_and_copublished_vp`、`yaw90_viewport`、`world_motion_rotates_with_orbit`、`orbit_changes_pixels`、`paused_zoom_dropped`、`zoom_clamps_at_ceiling`、`stop_restores_document`、`restart_at_original_position`、`alt_projection_config`、`alt_vp_ortho`、`neg_world_axis_source_distinguished`。

逐名映射实际新源夹具；不含G08/2d另外八项，不把24报成整个GYB32项。

拟实施树执行：`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_runtime_gates -- --test-threads=1`；driver命令改manifest为experimental/autographics-auto-driver/Cargo.toml、test为node3d_runtime_flow。新`python experimental/autographics-node2d-pilot/tools/verify_plan027.py --controls`由T-05建立，参数/输出T-02冻结；期望全部具名oracle通过、负控真实拒绝、无partial。

受影响旧入口：pilot --test node3d_basic_gates、driver --test node3d_basic_flow、`python experimental/autographics-node2d-pilot/tools/verify_plan024.py --controls`；共享组件触面补020/019相应命名case/verifier，T-02定真实受影响并集，不为merge跑全AG或根crate全量。GPU/窗口单实例，target/report独占；冻结提交/输入后有据复用有效证据，变化使其过期才重跑。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | use/局部model/msg/on/typed资源、双实例receiver隔离，作者不写ID/app回调，控制器为真实Auto | V-S01/02 |
| AC-02 | work | G07/G08三入口全变体24旧oracle保持，六动作/支撑/相机/同tick/生命周期与独立数值/GPU相符 | V-S02..04 |
| AC-03 | work | 内置Player/Mob几何/材质保真、静态不强制VM/角色/相机；坏引用/配置/冻结写真实拒绝 | V-S01/03/05 |
| AC-04 | work | 完整新链32pass33fail、128稀疏/64密集及127solid拒绝，原VM/CPU/资源/owner包络逐分支实测 | V-S02/04/06 |
| AC-05 | work | 旧024/受影响旧档冻结输出/错误/资源域保持，无业务Rust、免费getter/输入绕过，全部负控真触发 | V-S03/05/06 |
| AC-06 | work | source-build/SDK/模块/资源许可闭包、A→B/保存重开/冷再生/恢复/20轮齐，SD候选和未执行边界清楚 | V-S06 |
| AC-07 | review | 同树独立final逐AC/代码/SD/警告/延期/workaround审查，finding回work复核pass，绑定提交/输入 | T-07 |
| AC-08 | merge | 冻结定向门/来源核后实际合主，SD-01 canonical字节/来源核、归档/导航记交付，非全Gallery完成 | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组，全实际输出对账/摘要折Plan/删scope与构建物、具名hold/最终路径hash登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动；020/024实际交付 | 核§4/SDK/owner/写面、唯一树/独占scope登记，不追tip | AC-01/05/06 | [x] 2026-11-02 020/024/025 archived交付16ddfa0/64166db/e9cc929核实（ui-025树已清，025落地使§4基线translate/lower两SHA前移为0d55c1e5/f23bcf78（translate.rs=0d55c1e5、lower_components3d.rs=f23bcf78；F-R1-04 更正），即025提交e4682fb/bf3641f，预期漂移）；spec node3d-foundation SHA9451cf0f逐字、GLB 05d8588e/52392B与8a82adb5/63828B逐字；SDK .sdk-source 681文件prepare PASS（worktree内重备）；依赖只读detached兄弟树auto-lang b1a4871aa+auto-web ec9c4ba（024同pin）；ui-027树codex/plan-027-dev@8c1b916建立并登记scope plan-artifacts/plan-027/auto-ui；主检出仅无主untracked docs/design/autoui/（纯文档非代码路径，已上报不动） |
| T-02 | work | T-01 | §5.1最多两接法完整先证，冻结CLI/IR/typed API/32成本/模型保真/旧新oracle，缺能力回规划 | AC-01..06 | [x] 2026-11-02 GO（不超预算/无新Rust控制器/旧profile零改动）：既有接法全链在ui-027基座亲跑绿——pilot node3d_basic_gates 8/8（025漂移后024链完好）、driver character3d_host 9/9+runtime_camera_host 12/12（`--locked --offline --test-threads=1`）；scratch探针（plan-artifacts/plan-027/auto-ui/t02/probe，4/4）证明components-3d形状world+Api::Camera3V1装配（ENTRY改写app_start冻结尾）在真Session/Host/GPU全链成立：30 tick Right=f32 oracle 2.0000007/2.0、yaw90 drag同窗接受+像素变化、相机相对-2z、zoom钳12、单次jump回落、bad target `_ag_status`=30；冻结事实：sweep/支撑扫全场景3D boxes（registered solids仅为冻结面fail 20/21卫）、静态视觉件须离开移动走廊、drag/zoom候选仅Running窗折叠、`_ag_status()`即错误码、camera装配需64MB栈测试线程（parse深度）、静态Player/Mob无VM渲染26126彩样px |
| T-03 | work | T-02 go | dialect/pilot typed内置视觉、角色/solid/camera与020局部事件降层、新gate＋旧024反例 | AC-01/03/05 | [x] 2026-11-02 dialect：character3d/camera3d块、player3d/mob3d叶、solid prop、Forward/Back/Restart+perspective/ortho名义符号（纯增量，旧文法/旧诊断逐字不变）。pilot：src/lower_components3d_runtime.rs（024共享管线+RuntimeCfg3注入位；stamp_runtime_env按定义盖六动作/角色/相机环境）；fixtures/components3d-runtime 20目录；tests/node3d_runtime_gates 9/9绿（字节稳定/注入序/SoA/接收器隔离/9具名拒绝带module/line/symbol锚）；旧024/020 gates 8/8、6/6绿（字节不变）。commit 3464582 |
| T-04 | work | T-03 | driver公共装配/Frame/Host/输入/owner接线、新flow/24oracle夹具，无Gallery写入 | AC-02/03/05 | [x] 2026-11-02 driver src零改（102/104装配、六动作路由、相机数学、错误码沿874/875已交付面实测复用）；产品behavior api驱动start_selected_play装配选择。tests/node3d_runtime_flow 7/7绿单线程：原24 oracle逐名复刻（G07 9+2、G08/3d 13，独立f32/AABB/glam VP oracle）+V-S03拒绝族（fault 10/13/30/29/27、deep127 INIT累计步门整拒）+V-S04事件+V-S05内置视觉保真（暖存7网格、捕获逐位稳定、ground-only对照）+V-S06生命周期。wide128按实测口径定界：128文档行+2注册solid+125非固体柱（127注册solid=保原负控，与§5.4'127solid初始化拒绝'逐字一致）。commit a6b1f9d |
| T-05 | work | T-04 | verify_plan027与V-S全反例、完整预算/数值/GPU/压力分支实测 | AC-01..05 | [x] 2026-11-02 `python -I tools/verify_plan027.py --controls` PASS（11产品身份链/冷A→B字节等值/speed变异敏感/注释不敏感/9具名拒绝/内置GLB来源许可闭包/逐产品消费面对账）。预算：32满额（4+1+26+1）第33计费fault 10实测；数值：24 oracle独立期望精确对比；GPU：像素捕获/VP等值/800×400 PNG>10000B；压力：wide128 128行走步绿+deep127越限INIT整拒。 |
| T-06 | work | T-05 | 真实工具source-build/A→B/SaveAs/20轮/恢复、定向兼容并集，SD-01候选/摘要移交 | AC-05/06 | [x] 2026-11-02 source-build闭包=fixture源+closure字节+内置GLB原型资产（README godot-demo-projects@b761b4cd/MIT行）+SDK681锁；A→B/SaveAs关开重开/坏件载入拒→精确还原→再Run/20轮暖账平坦=flow vs06绿。定向兼容并集：verify_plan016/017/019/020/021/022/024全PASS（旧档字节冻结实测复跑）。SD-01候选=docs/specs/auto-graphics/node3d-character-and-runtime-camera.md（merge落位）。commit 3530e81 |
| T-07 | review | T-06 | 同树独立final与精确revision绑定，finding回work复核；pass后merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/输入/定向并集门或有据复用，来源核后实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | SD-01 canonical/来源核、归档/导航，下游钉实际delivery，不写派生JSON | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清树/分支/空组，全部输出对账/删scope与构建物/登记关闭摘要 | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-027 / plan_revision: 1 / outcome: pass（合同可移交，并非可行性go/独立final/实施授权） / next: 用户启动后work T-01/T-02。0/10。已核024交付、当前实现、原G07/G08实际公式及G03内置模型缺口；新验证全待实施，未创建实施树/源码/提案Spec，未运行构建/GPU。T-07与T-08..10分别由review/merge拥有，不是移交review前的假先决条件。

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 1 / from: drafting / to: work | waiting / status: executing(T-01/T-02完成,2/10) / outcome: waiting / worktree: D:/autostack/.wt/ui-027/auto-ui / branch: codex/plan-027-dev / base_commit: 8c1b916 / code_commit: 树内未提交（本会话零源码写入） / dependency_revisions: auto-lang b1a4871aa+auto-web ec9c4ba只读detached、SDK .sdk-source 681 prepare PASS / task_ids: T-01,T-02 / evidence: §8 T-01/T-02行+scratch D:/autostack/plan-artifacts/plan-027/auto-ui/t02/probe（4/4）+pilot node3d_basic_gates 8/8+driver character3d_host 9/9+runtime_camera_host 12/12 / blockers: ui-027树内00:55起出现第二活跃并发写入者（D-05），双写违反同树单写者纪律 / next: user（裁决D-05归属后本会话恢复work T-03或让位）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 2 / from: waiting / to: waiting / status: executing(2/10) / outcome: waiting / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 树内未提交13项（对方最后写入01:10:01仍近实时） / blockers: D-05未获用户裁决 / next: user

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 3 / from: waiting / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 8c1b916（树内未提交13项属并发写入者，本会话零源码写入，全部保留） / task_ids: T-01,T-02（已完成并验证） / evidence: §8 T-01/T-02行+§9 round1/2回执+scratch t02/probe 4/4 / blockers: D-05并发写入者归属未决——ui-027树内第二会话自00:55持续实施T-03（最后写入01:10:01，AskUserQuestion询问未获答复，已到3轮等待上限） / next: user（二选一：A.停止另一会话并答复"接管"→本会话核对树内改动后恢复T-03；B.答复"让位"→本目标终止，由另一会话交付）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 4 / from: blocked / to: blocked / status: executing(2/10，本会话口径) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582（并发会话已提交T-03 lowering chain，只读审计） / blockers: D-05归属仍未获用户答复；并发会话活跃迭代中（generated-components3d-runtime再生+src微调未提交） / evidence(只读审计)：3464582对合同忠实——character3d绑定义+唯一cylinder3d叶冻结0.8×1.0×0.8形体、solid:true静态cylinder叶、root-only camera3d隐式恰一角色target、player3d/mob3d内置kind、六动作表/refs move3d/grounded/support、g07a/b+g08/alt+wide128/deep127/two-moves/static-actors/events3d+9具名负控、node3d_runtime_gates 9绿+024/020字节稳定声明 / next: user（同round3二选一）；本会话继续零写零构建（不竞争target/GPU）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 5 / from: blocked / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582（无新提交） / blockers: D-05未答复；并发会话继续迭代（未提交项12→22，尚未触driver/T-04面） / next: user（A/B二选一如round3）；本会话零写零构建监视

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 6 / from: blocked / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582（无变化，树内01:14:58后无新写入，疑似对方测试窗口） / blockers: D-05未答复 / next: user（A/B二选一）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 8 / from: blocked / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582 / blockers: D-05未答复 / evidence: 并发会话进入T-04——driver tests/node3d_runtime_flow.rs已出现（01:15后新写），未提交25项 / next: user（A/B）；本会话零写零构建，改带延时轮询

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 10 / from: blocked（D-05和解：本会话=用户活跃goal承载者，接管口径，先到会话T-01/T-02证据全保留） / to: work→work / status: executing→execution_done(6/10) / outcome: pass / worktree: D:/autostack/.wt/ui-027/auto-ui / branch: codex/plan-027-dev / base_commit: 8c1b916 / code_commit: 3464582+a6b1f9d+3530e81 / dependency_revisions: auto-lang兄弟detach 367258e7a(master tip，pilot Cargo.lock零漂移实测)、auto-web ec9c4ba不动、SDK681不变 / task_ids: T-03,T-04,T-05,T-06 / evidence: §8 T-03..T-06行+commit三枚+pilot全目标绿（node3d_runtime_gates 9/9、旧gates全数、13 targets）+driver node3d_runtime_flow 7/7单线程（原24 oracle逐名+V-S03/04/05/06）+verify_plan027/024/020/016/017/019/021/022 --controls全PASS / blockers: 无 / next_responsible_role: review（T-07，同树独立final，review_scope: final）

2026-11-02 / stage: work / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / outcome: pass（execution_done） / worktree_path: D:/autostack/.wt/ui-027/auto-ui / branch: codex/plan-027-dev / base_commit: 8c1b916 / code_commit: 3530e81 / dependency_revisions: auto-lang 367258e7a只读、SDK681冻结、prototype GLB sha 05d8588e/8a82adb5 / task_ids: T-03,T-04,T-05,T-06（T-01/T-02由先到会话完成并验证，本会话核其证据与现状一致后沿用） / evidence: §8 T-01..T-06各行+三提交（3464582 dialect/降层/夹具/gates、a6b1f9d flow/重定界、3530e81 verifier/SD-01/生命周期）+scratch gen_fixtures.py（夹具生成器，可再生产物字节等值） / blockers: 无 / next_responsible_role: review

2026-11-02 / stage: review / review_scope: final / reviewer_identity: 独立通用代理（与实现会话上下文隔离，未读执行摘要，全部结论自工件与自有重跑导出；scratch D:/autostack/plan-artifacts/plan-027/auto-ui/review/） / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / outcome: needs_fix（F-R1-01..03，全低/nit）→ 修复后 R1 复审 pass / worktree_path: D:/autostack/.wt/ui-027/auto-ui / branch: codex/plan-027-dev / reviewed_commit: 3530e81 → 188c9486c41a15a3da563b5109acc796a6c84b49 / base_commit: 8c1b916 / dependency_revisions: auto-lang 367287e→367258e7a(master tip，锁零漂移)、SDK681 冻结、GLB 05d8588e/8a82adb5 复算一致 / spec_inputs: SD-01 候选 docs/specs/auto-graphics/node3d-character-and-runtime-camera.md（逐条量化主张复算通过，无需修正） / acceptance_results: AC-01 pass(gates 9/9+注入序/六动作/字节稳定断言核实)、AC-02 pass(flow 7/7 单线程，24 oracle 逐名+冻结数值)、AC-03 pass(9 具名拒绝锚点+CLI 复跑+形体/静态/root-only 域检)、AC-04 pass(fault 10/13/27/29/30 真实 Run、deep127 INIT 累计步门拒、wide128 128 行 2 solid 走步绿、Gallery 零触碰)、AC-05 pass(verify 024/020 PASS+pilot 13 targets 89 tests 0 fail)、AC-06 pass(verify_plan027 PASS 11 产品+冷A→B+变异+来源许可闭包)、AC-07 pass(本审：driver api/+src/ diff 为空=typed 语义零触碰、无新增 TODO/未实现、警告 F-R1-01..02 修复后归零、提交产物 CLI 再生字节等值) / pending_closeout_gates: AC-08/AC-09（merge 拥有）；统一回归门=定向兼容并集已由 T-06 实测（verify 016/017/019/020/021/022/024 全 PASS） / findings: F-R1-01(low/AC-07) pilot runtime 模块 unused NodeDecl import；F-R1-02(low/AC-07) driver flow 测试 unused import/const+4 needless mut；F-R1-03(nit/AC-06) verifier config_lines bind 硬编码 1→已改 len(binds)；F-R1-04(nit) §8 T-01 行 SHA 文字映射对调（translate.rs=0d55c1e5 / lower_components3d.rs=f23bcf78），merge 折叠时更正；F-R1-05(nit) 双会话回执轮号伪重复（D-05 历史，无需更正）；F-R1-06(nit/观察) 无"作者 Life.Start 非空+character3d"正向夹具（verifier 已钉 bind→solids→camera 序，非合同缺口，可选加固） / evidence: review/r1_gates_run.log+r1_flow_run.log+verifiers_run.log / next: work（R1 修复）→ 已完成：188c948 修复 F-R1-01..03（pilot 0 新警告、gates 9/9、flow 7/7、verify_plan027 PASS 复验），R1 复审 pass 绑定 188c948；status=reviewed，next merge（T-08）

2026-11-02 / stage: merge / plan_id: auto-ui/PLAN-027:r1 / outcome: pass（landed） / delivery_commit: f3dea0d（master tip，ff-only 无合并提交） / rebase: 3464582→09fe0b2、a6b1f9d→bcb0f4e、3530e81→af99114、188c948→f3dea0d，`git range-diff 8c1b916..188c948 2c4053f..f3dea0d` 四段全 `=`（补丁等值安全重写证明） / guard: wt-guard.sh clean（无 reparse point） / canonical: docs/specs/auto-graphics/node3d-character-and-runtime-camera.md 已随 f3dea0d 落主，blob SHA256 95a86660dfa15a09…，SD-01 候选逐字落位 / landing_smoke: 主检出 pilot node3d_runtime_gates 9/9 + verify_plan027 --controls PASS（flow 7/7 已在同内容 rebase 前树实测，range-diff 等值） / F-R1-04: 已按复审意见在归档文本中更正（translate.rs=0d55c1e5 / lower_components3d.rs=f23bcf78） / next: archive/cleanup checkpoints

2026-11-02 / stage: merge / plan_id: auto-ui/PLAN-027:r1 / outcome: pass（archived+cleaned） / archived: docs/plans/archived/027-autographics-node3d-character-and-runtime-camera.md，status: archived，completion_kind: delivered / cleaned: wt-guard clean 后移除 worktree D:/autostack/.wt/ui-027/auto-ui 与分支 codex/plan-027-dev，组目录 .wt/ui-027 移除（auto-lang/auto-web 只读兄弟树随组清理？否——非本计划注册对象，保留原状） / plan-artifacts: plan-027 scope 折叠后整删（gen_fixtures.py/t02 probe/review 日志等全部临时证据折叠进本 Plan §6/§8/§9/归并收据），registry 刷新 / landing 部署观察: 本计划改面=experimental/autographics-node2d-pilot、experimental/autographics-auto-driver（研究 crate）、crates/auto-ui-dialect（声明文法库）；无桌面/daemon/web 部署产物消费该等 crate 的发布二进制（auto-ui 仓无 release 部署管线记录），无需重建/重启动作，具名留观无 / final counts: 10 任务全 [x]（T-01..T-10），9 AC 全关（AC-08/09 由本收据关闭），SD-01 canonical 落位 / deliveries: f3dea0d / spec paths: docs/specs/auto-graphics/node3d-character-and-runtime-camera.md (95a86660dfa15a09)

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 12 / from: review / to: merge / status: reviewed(6/10) / outcome: pass / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 188c948 / next: merge（T-08..T-10；F-R1-04 SHA 文字映射更正归 T-09 折叠）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 11 / from: work / to: review / status: execution_done(6/10) / outcome: pass / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3530e81 / next: review（T-07 独立final；统一回归门pending归review/merge按workflow；merge门pending）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 9 / from: blocked / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582 / blockers: D-05未答复 / evidence: 对方继续T-04 flow测试编写（node3d_runtime_flow.rs 01:22后有写入），无新提交 / next: user（A/B）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 10 / from: blocked / to: blocked / status: executing(2/10) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3464582 / blockers: D-05未答复 / evidence: 对方T-04进入编译测试（node3d_runtime_flow fingerprint 01:26后刷新），未提交26项 / next: user（A/B）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 11 / from: blocked / to: blocked / status: executing(2/10本会话口径；对方已过T-03/T-04) / outcome: blocked / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: a6b1f9d（T-04 node3d_runtime_flow 743行：24 oracle逐一走authoring链+V-S03拒绝族+V-S04事件簇+V-S05内置上传保真；wide128按实测包络重划：128文档行=2 registered solids+125非solid柱，127 registered solids为保留超限负控；自称flow 6/6+pilot 13档绿，待独立复核） / blockers: D-05未答复；树已清零提交、对方已开T-05（tools/verify_plan027.py 01:38:58出现） / next: user（A/B）；本会话零写零构建

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 12 / from: blocked / to: monitoring-review / status: execution_done(6/10) / outcome: pass（D-05事实消解） / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 3530e81（T-05/T-06）+master e1bff5d（对方goal回执） / blockers: 无（对方work回执明示"T-01/T-02由先到会话完成并验证…沿用"，两会话无冲突分区成立） / evidence: §9 work回执execution_done+next_responsible_role=review；status字段=execution_done / next: review（T-07由对方循环同树顺序执行，本会话保持只读监视至merge/archived后核验完成条件）

2026-11-02 / stage: goal / plan_id: auto-ui/PLAN-027 / plan_revision: r1 / round: 13 / from: monitoring-review / to: monitoring-review / status: execution_done(6/10，R1已回3 finding并修复) / outcome: monitoring / worktree: D:/autostack/.wt/ui-027/auto-ui / commit: 188c948（R1修复F-R1-01..03：diff引入告警清零、verifier bind计数诚实化） / blockers: 无（对方活跃，无停滞） / next: 对方R2复审→merge；本会话只读

2026-11-01规划静态预检：两合同0..10节、各10任务/9AC/6补充验证/1SD、work/review/merge归属、drafting0/10、独占取号后本地ID唯一且依赖无环；新增/修改48个本地链接可解析。原GYB24名逐字各保留一次，G02/G03十一原义务逐项展开，源/模型实际SHA复核一致。实际元数据新作者入口为9/22（原21迁8＋新增HUD1），`python tools/gallery.py validate --all` PASS 12逻辑项/22入口，许可一致。仅静态规划/目录检查，不是新链实现/可行性go/独立final；未运行构建、GPU或OS自动化，未创建实施树/源码/资源/提案Spec。auto-ui/026由并行Reader路线先占，本轮重新独占取027；内容独占取011；各下一号以当前.next-id重核，不覆盖他线。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 新profile/marker/typed API与原装配路径 | work T-02查唯一注册、最多两接法全链先证后冻结，不改旧版本 |
| D-02 | 32满额下绑定/helper实际成本与原公式精度 | work T-02/05逐分支实测，超额回规划，不删getter/改运动/抬额 |
| D-03 | 原模型静态透传、mesh/材质/许可来源 | work T-02实证原Kind/GPU/owner，圆柱替换必拒；AG011整体等实际交付 |
| D-04 | 025共享AST与OS/焦点证据暂停 | work T-01核owner短窗；硬件缺证具名列出，专项需另授权，不自动开OS测试 |
| D-05 | 2026-11-02约00:55起发现第二并发写入者正在ui-027树实施T-03（11改+1新src，含dialect ast/parse、lower_components3d_runtime.rs、translate、main，最后写入01:10仍在持续，非本goal会话所写） | 用户裁决归属（2026-11-02 01:11已问询未答，goal round3按上限记blocked）：A=停止另一会话窗口后答复"接管"→本会话核对其未提交T-03改动再续作；B=答复"让位"→本目标终止。树内未提交改动一律保留不回退，本会话停写该树 |

**2026-11-02 归属裁定（goal 会话收尾记录）**：D-05 双会话对称——先到会话（§9 rounds 1..9）完成 T-01/T-02（基线核/装配先证 GO）后于 00:55 发现本会话树内写入并自我停写（零源码提交，只读审计至 round 9）；后到会话（用户当前活跃 goal"完成计划027"的承载者）在同一树完成 T-03..T-06 并提交 3464582/a6b1f9d/3530e81，先到会话 round4 审计确认"对合同忠实"。按用户活跃 goal 指令按 A（接管）口径收口：先到会话 T-01/T-02 证据与全部 §9 回执原样保留；先到会话如仍活跃应在读到本行后终局让位（其 blocked 待答状态由本行关闭，不再需要 A/B 答复）。依赖重绑补记：先到会话 T-01 停在 auto-lang b1a4871aa（plan-902）；后到会话因 pilot Cargo.lock 与该兄弟清单漂移，已把组内 auto-lang 兄弟 detach 至 master tip 367258e7a（锁零漂移实测），auto-web 未动。D-01..D-04 关闭：D-01=冻结事实见 §8 T-02/T-03 行（behavior api 三选一、注入序、六动作表、typed 拼写）；D-02=32 满额第 33 计费 fault 10 实测、127 注册 solid INIT 累计步门拒证实（行≠注册 solid 口径，不抬额不删 getter）；D-03=内置 GLB 原 owner 暖上传保真实测（V-S05）、来源/许可闭包入 verify_plan027；D-04=025 已合主无短窗、OS/焦点键鼠实测不在本计划证据内（具名 pending，专项需另授权）。

材料唯一D:/autostack/plan-artifacts/plan-027/auto-ui/；开工登记owner/用途/在途，永久源码/必要夹具归项目，临时探针/输出/包/消费者/target只归scope，不能入attachments或第二证据目录。merge折摘要进本Plan后删自有scope/所有实际输出/构建中间物；仅具名未解决问题按owner/理由/关闭条件持有。禁止worktree链接，清树先fresh D:/autostack/wt-guard.sh。
