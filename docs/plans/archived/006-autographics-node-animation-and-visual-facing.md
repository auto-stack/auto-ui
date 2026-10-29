---
plan_id: PLAN-006
status: archived
feature_name: AutoGraphics无蒙皮节点动画与视觉朝向
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30
plan_revision: 2
current_step: 9
total_steps: 10
completion_kind: delivered
delivered_commit: c681663e6ff392d70471abbec560e9d9daf12622
knowledge_commit: 664145b791a3bb2576a13414e0c2929203c74e56
depends_on: [auto-ui/PLAN-005]
supersedes_spec_components:
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/scene-assets.md
new_spec_components:
  - docs/specs/auto-graphics/node-animation-and-facing.md
touched_goals: [GOAL-008]
---

# PLAN-006：无蒙皮节点动画与视觉朝向

## 0. 变更摘要

消费005实际交付的纹理/节点资源，支持冻结character.glb所需的有限节点TRS clip播放与独立视觉yaw。真实Auto选clip/phase/朝向，Host只验证与采样，不嵌入玩家状态机。只改引擎及验证夹具；G10b内容以后另属ag-examples。

## 1. 目标

原始固定character的static/idle/walk/jump可辨识地播放，3D相机相对移动时角色面向Auto给定方向，停止/暂停/单步/重启有确定结果。保留2D精灵、3D纹理和对象/触发共同组合；面向与动画只影响视觉，不改变既定角色碰撞/最终支撑规则。clip/phase和物理/相机在同tick整体提交，不能画面滞后一帧。

非目标：skin/骨骼/IK/morph、混合树/根运动/动画事件、任意glTF扩展或编辑器、Rust内判玩法、G10产品目录/离线ZIP、正式2D/3D游戏、声音/HUD、新平台、生产UI/HIR/ACE。动画fixture不等同最终关卡。

## 2. 架构方案

GLB原始层级/clip轨道→有界声明/数值/资源准入→不可变model+animation payload→真实Auto有限视觉状态→Host验证Run/实例/clip/phase→确定性采样TRS/世界矩阵→共同Prepared→GPU颜色与阴影同pose。不可沿用static loader不可逆的顶点烘焙来伪造节点轨道；005保留节点primitive关系的接口作为输入。本期每个可视模型实例每tick最多一个clip和一个yaw，无混合。

唯一实现/顺序复审树拟`D:/autostack/.wt/ui-006/auto-ui`，分支`codex/plan-006-dev`；等005实际交付后核master再建。共用assembly/Frame/资源/renderer写面串行，不与005开发tip并行接线；主检出簿记，SDK/target/报告独立，无链接，清理先guard。

## 3. 技术栈

既有driver `authoring/{assets,behavior,play,runtime_objects,runtime_camera,runtime_round}.rs`、`behavior/assembly.rs`、`host/{viewport,play_adapter}.rs`；AGP `assets/glb.rs`、`render3d/{pipeline,anim}.rs`以及005新节点/纹理表示。现有anim.rs是Squash专用BobAnimation，不是公共GLB clip服务，不能复制专用动作作为本计划成果。

拟新增`experimental/autographics-auto-driver/tests/public_node_animation.rs`、`experimental/autographics-prototype/tests/node_animation.rs`、driver `tools/check_node_animation.py`与`verify_plan006.py`；为T-03..06待建入口。采样数学复用glam，实际支持模式必须与准入表相符。

## 4. 需求分析与背景调查

2026-10-29 执行授权：用户 `/goal 完成AG计划006` 授予本计划 work→独立review→有界repair→merge/规范/归档/guard清理全链授权，后续阶段继承；目标、范围、验收与预算保持r1。005现已实际交付，前置核验与原件实读见[执行基线](../attachments/006/work/source-preflight.md)。

2026-10-30 reviewer上下文授权：用户明确答复“允许独立子代理复审”。据此启用 `/root/review_plan006`（fork_turns:none），在唯一实施树只读实现、独立重建验收结论；后续受影响复审可沿用该独立上下文，无需再问同一授权。主代理未将实现会话自检冒充独立最终复审。

2026-10-30 r2有界修订：F-006-W1-INPUT-BUDGET实测证明r1逐键读取设计无法承载原完整组合。按原§5.1路由，仅调整尚未发布新profile的输入读取设计；目标、AC阈值、32credits、CPU/源/内存、旧接口兼容、仓库及动作范围均保持，继承上述授权。事实、Spec指纹与消费规则见[输入预算修订](../attachments/006/work/input-budget-revision-r2.md)。修订当时getter待实现，现已在04042f8实现，完整当前关闭及最终消费证据见§9，不以设计和式代替实测。

2026-10-29规划背景（历史，当前执行进度见§8/9）：用户要求规划下一阶段；规划轮仅合同/设计/引用，无实施、agent、窗口/GPU/Cargo门或发布。来源和指纹见[基线](../attachments/004/planning/baseline.md)，资源与阶段边界见[设计](../../design/autographics/m2-presentation-and-resources.md)。当时005/004尚未交付，未填写未来SHA/接口价格；实际前置现已由T-01核验。

现有权威：[样本研究](../../../../auto-lang/docs/specs/auto-graphics/m2-game-sample-research.md)、[游戏分母](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[运行角色](../../../../auto-lang/docs/specs/auto-graphics/game-actions-and-character2d.md)、[触发新局](../../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[静态资源](../../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)。研究记character含static/idle/walk/jump、无skin；具体轨道interpolation/keys/时间/层级尚须读取固定原始字节，不能从“有动画”推断是骨骼或只需移动根。旧858源清单缺失风险由005 T-01先恢复，006再核该收据，不换来源。

## 5. 详细设计

### 5.1 前置与首个可执行入口

T-01只接受005已合master、独立review与冻结门完成的实际delivery，以及004/005资产/SDK/两锁/consumer身份、素材逐文件来源/许可/closure表。T-02先完成`attachments/006/work/node-animation-contract.md`和整路径probe：纹理character原始clip→Auto同时运动/跳/相机/触发/一次读取/视觉setter→采样→同帧输出→实际内容。固定profile/CLI、wire表、clip稳定ordinal/名称、phase单位/loop/one-shot、计价和所有output校验后才进入实现。

保持32credits、原callback/step/时间与CPU目标、受管源96KiB/总352KiB、用户256KiB；不因006叠加再次抬限。应有单次有限setter设置clip/phase/yaw，输入实体绑定静态3D叶或合法001实例完整Run/slot/generation；旧身份拒绝，销毁撤销，复用槽从模板初始视觉重新开始。Auto计算角色状态、选择idle/walk/jump并由fixed tick推进phase；Host只执行明确的格式/采样，不按角色速度/grounded替Auto选动作。T-02证明真实组合预算不足则带反例回new，不能省相机/触发/原clip或转专用Rust业务。

r2固定公开profile为`autographics.node_animation3d/1`（version110/profile21/CLI `node-animation3d`），视觉wire/phase/yaw和原`ag_visual_pose(kind,entity,clip,phase,looping,yaw)`成本1保持。只在该未发布profile的`api/node_animation3d_v1.at`新增成本1的`ag_actions_snapshot() int`（已由T-02实现/实测）：读取既有6held+6尚未消费pressed，返回`held + 64*pressed`（0..4095）；bit0..5为左/右/前/后/跳/重启。成功读取消费本tick所有未读pressed、保留held；重复和旧getter混读不得重放边沿，旧getter价格/旧14profile装配字节不改。app_start无运行输入为0，Pause/neutral Step/fresh Run沿原输入路由/clear规则，无跨tick或跨Run缓存，也不增Host wire或Rust玩法。

修复关闭条件是同一活动回调实测 `1 snapshot +26 move +1 grounded +1 event_next +1 pair +1 phase +1 visual =32`，四向/跳跃解码、相机相对运动、选择clip/phase/yaw均由Auto完成；必须有真实事件读取分支、真实输入敏感结果，并用额外1credit超额负控证明没有旁路。此条件已通过T-02及T-06当前冻结重放，原逐键error10反例保留；完整S-06/owner证据见r2关闭附件。

### 5.2 支持表与确定性

有限无skin/morph的节点translation/rotation/scale动画；首版支持LINEAR与STEP，quaternion线性轨道按glTF语义最短弧slerp并正规化，不能逐分量lerp后当等价。keys时间有限、非负严格递增，output计数/类型和维度匹配，单clip同node同path重复轨道拒绝，动画作用节点与原始层级唯一映射。CUBICSPLINE/weights/skin/其他目标在本期显式unsupported；T-02必须先确认冻结原clip不依赖这些模式，若确有必需模式则回new扩合同，不能丢该clip或用未批准转换代替原件。

上限仍nodes128；新增有限域<=16 clips、<=128 channels/asset、全部channel累计<=8192 key samples；input时间/output TRS与必要索引/采样scratch按实际bytes计入既有CPU64MiB/GPU64MiB总额，不另给动画免费空间。005纹理/geometry/原始资源8身份32MiB、单raw8MiB/JSON1MiB、16 expanded primitives/512潜在mesh实例等上限全部保持。先按声明与span预检再分配；动画过程中不得制造超出已准入包络的资源。选定原件若越限只能回new，不临时松限。

phase固定为Auto显式给出的有限非负f32 seconds，不能同时存在两套不一致时钟；loop以duration取模、one-shot夹到末端，精确末端按loop回0/one-shot保持最后key，duration0单pose保持；切clip立刻采样当次给定phase。Stop/fresh Run恢复Auto app_start初值；Pause不更新phase，Step仅一个fixed dt；不以render次数/wallclock漂移。静态无clip保持基础pose。

### 5.3 视觉/碰撞/发布

视觉yaw为显式有界有限角度（T-02冻结单位与归一到[-pi,pi)规则），只绕公共模型视觉pivot旋转；不改变001实例位置或字符的运动方向输入，不将yaw写回solid/collider transform。Auto可据相机相对运动向量选yaw，静止保留上次合法朝向。父层级变换×视觉pivot yaw×动画节点TRS顺序写入合同；负缩放/奇异/非有限pose按原资产边界拒绝，不能进入颜色或shadow pass。不会为动画姿势重算角色碰撞：仍用冻结静态碰撞定义，G3支撑/墙/平台结果在动画开关/yaw变化下相同。Editor拾取/包围展示若需动画包络，与物理包络分别命名并定向核验，不偷改碰撞。

模型、UV/纹理、node pose、相机/角色/对象/事件一次prepare/publish。坏clip/phase/旧代/预算/prepare/OOM不发布部分pose，last-good整帧不变；Stop能恢复编辑。每帧仅更新有界pose/instance数据，不能逐tick重解码、上传mesh或texture；同一asset两个实例可独立phase/yaw而共享immutable payload。20轮Play/新局/Stop/关闭检查实际VM/pose/texture/mesh owner，合法Editor/外部pin保留，最后drop释放。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang知识仓） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-animation-and-facing.md | 无公共clip→有限TRS/Auto phase/视觉yaw/碰撞独立/事务 | M2 G3-01/07前置 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/behavior-api.md | 新显式视觉组合profile/单visual setter与聚合6held+6pressed读取各1credit/身份与pressed混读规则；旧装配/逐键价不变 | 公共Auto而非Bob专用路径，真实完整组合保持32credits | AC-02/03/05 |
| SD-03 | modify | docs/specs/auto-graphics/scene-assets.md | 新profile可声明受检动画；旧静态/纹理支持域继续显式分列 | 无静默忽略clip | AC-01/04 |

## 6. 测试设计

当前已建立并执行新模块30case及3个继承定向case，`--locked --offline -- --test-threads=1`，实际结果/受影响范围见§9及t06-consumer-and-identity.md。正式运行工作目录为包目录以启用原32MiB主栈/16MiB worker配置。`check_node_animation.py --controls`与`verify_plan006.py --controls`均由T-06实测通过，绑定1477文件当前身份、实际runner/log/数值/栈；最终仓外消费A→B亦通过。仅受影响面与明确旧样本，新AC正反例不省；无窗口真实GPU单实例、OS抢焦点/置顶自动化暂停、全AG里程碑pending、根零改不跑根档。

| V/补充case | AC/owner_stage/启用任务 | 输入→实际路径 | 区分性判据/关闭条件 | 覆盖边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01 work T-02/03 | 固定character四clip、两层节点fixture、坏跨度/非递增时间/错计数/重复target/越限/skin/cubic | 支持原件真准入；unsupported与invalid/resource_limit区分且先检后分配；节点关系不烘焙丢失 | 如必需原件不在域内回new，不能无动画正控代替 |
| V-02/S-02 | AC-02 work T-04 | 真实Auto切idle/walk/jump/static、边界0/mid/end/loop、STEP/LINEAR旋转，两个共享asset实例不同phase | CPU独立矩阵/顶点期望与实际GPU轮廓/marker ROI同时敏感；改clip/phase应变图；去掉node轨道负控必红 | 不以tick/命令/hash变化代替动画画面 |
| V-03/S-03 | AC-03 work T-04 | 相机yaw改变/前后左右移动/停、±pi边界、动画开关、墙地接触 | Auto提供yaw后的角色面向可见；动画/视觉yaw变化不改相同输入的角色位置/支撑/触发；颜色与shadow同pose | 无根运动/姿势碰撞，非通用刚体 |
| V-04/S-04 | AC-04 work T-05 | 坏clip/phase/旧Run/旧slot代、prepare失败、Pause/Step、20轮新局/Stop/最终drop | 全旧或全新，neutral Step一次；新局初pose、暖mesh/texture零重传；真实weak owner/上传及预算账 | 合法外部pin最后drop后查，不测驱动内部保留 |
| V-05/S-05 | AC-05 work T-02/06 | 完整真实Auto四向/跳跃输入+相机+move/grounded+event_next/pair/phase+visual的32credits组合、fresh SDK consumer A→B、004精灵/005纹理/旧profile定向 | CPU/步数/源/内存实测与既定额、公开入口离线保存/重开、identity controls/源变异敏感；原固定运动向量probe不能代替输入实测 | 不等于完整G3关卡或G10b内容交付 |
| S-06（补充V-05） | AC-05 work T-02；输入状态影响T-04/05 | 原逐键反例；新聚合0/63/各bit、mixed/重复pressed；完整键控活动回调、额外1credit、Pause/neutral Step/fresh Run | 原逐键仍error10；新完整真实事件分支fault0、输入改变运动/clip/yaw且同tick；已消费pressed不重放、held保持；neutral/fresh无旧边沿；新source<=96KiB/旧14profile hash保持 | r2已通过，public8case/Host与20轮owner见r2-input-pixels-and-lifecycle.md |

同一唯一树顺序实施和独立上下文最终review，未指定实现者/模型，不为规划创建agent；独立review逐AC/V/S/SD核遗漏、warnings/fmt/动画静默替代、CPU与owner。review pass标merge冻结门pending，修代码后重新绑定证据/复审受影响面。

## 7. 验收标准

| ID | owner_stage | 结果与证据 |
|---|---|---|
| AC-01 | work | R2 pass：F-006-R1-UNUSED-SAMPLER关闭；原始14fixture/22内置反例及合法边界重放、独立7个末尾clip/offset/共享accessor/索引前插兄弟输入通过，见review/r2.md；原资源/保存证据有效保留 |
| AC-02 | work | V-02已通过：原四clip21状态/双实例、独立源顶点/GPU、STEP旋转实际GPU与错误插值负控，见t04-pixels-and-input-diagnosis.md及r2-input-pixels-and-lifecycle.md |
| AC-03 | work | V-03已通过：四向真实位移/可见朝向、stop/±pi、240tick动画开关物理/支撑/触发相同、独立同pose阴影ROI0误差及两负控，见r2-input-pixels-and-lifecycle.md |
| AC-04 | work | V-04已通过：完整last-good/坏输出/prepare失败、Pause/neutral Step/fresh、20轮Play/restart/Stop真实weak/暖mesh与texture上传0增量/最终pin释放，见r2-input-pixels-and-lifecycle.md |
| AC-05 | work | 已通过：T-02完整32/33真实预算、T-06仓外完整消费/冷重开/真实Auto源变异、继承兼容、1477文件current identity/controls与CPU包络，见t06-consumer-and-identity.md；非Rust玩法 |
| AC-06 | review | R2 independent final pass，a85980c/e2994efa；R1唯一finding及共因/兄弟路径关闭，SD通过，无开放finding；T-07完成，新34case冻结merge门通过 |
| AC-07 | merge | pass：新34case/controls exit0，源码等价rebase并ff-only合master，完整AG里程碑仍pending |
| AC-08 | merge | SD/ledger/索引及归档交付已验证；清理见最终T10收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 005实际交付、用户启动 | 核005/004/SDK/两锁/素材收据与owner，建唯一ui-006树 | AC-01/05 | [x] 11bc18b；source-preflight.md |
| T-02 | work | T-01；r2设计已修订 | node-animation-contract：原clip/phase/yaw/wire保持，新增聚合input getter及S-06，真实完整32credits整链probe，失败回new | AC-01..05 | [x] 04042f8；r2-input-pixels-and-lifecycle.md；原逐键反例仍error10，新真实事件32成功/额外1失败 |
| T-03 | work | T-02通过 | 有界节点/clip准入与持久化/资源计账，V-01 | AC-01/04 | [x] 82ba57e修R1声明级sampler共同准入；resources6case/core5case与R1原件重放通过，原额度/持久化证据保持 |
| T-04 | work | T-02 r2恢复通过、T-03 | Auto公开视觉输出/Host采样/renderer色图与shadow，V-02/03；r2输入路径驱动朝向/物理 | AC-02/03 | [x] ccfe6d9+04042f8；原四clip/顶点/GPU+STEP/四向/±pi/240tick物理/同pose阴影ROI与负控 |
| T-05 | work | T-04 | 事务/实例代次/新局/暂停单步/owner与20轮V-04 | AC-04 | [x] 04042f8；完整last-good/新槽初pose/Pause/Step/40VM与pose释放/暖mesh与texture0上传/预算账 |
| T-06 | work | T-05 | fresh消费/预算/兼容/identity controls、fmt/warnings/提交、SD与下游接口 | AC-01..05 | [x] 82ba57e/a85980c修复交接；e2994efa当前1477身份/SD/新仓外消费1/1，未变CPU/兼容/画面owner有效证据保留；新34case冻结门归merge |
| T-07 | review | T-06 | 同树独立review全部V/S与遗漏，merge门pending | AC-06 | [x] a85980c上R2受影响独立final pass；原R1完整矩阵按不变路径复用，修复/SD/身份/消费关闭，review/r2.md |
| T-08 | merge | T-07 pass | 冻结提交定向门/有效复用，通过才合本仓master | AC-07 | [x] verified merge receipt below |
| T-09 | merge | T-08 | Spec/ledger/索引、本仓归档与真实delivery SHA/clip接口/消费包身份 | AC-08 | [x] verified merge receipt below |
| T-10 | merge | T-09 | wt-guard clean后清唯一树/分支/空组与恢复收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: blocked | next: 等005实际delivery后work T-01/T-02`。本轮0/10、未建树/实现/运行所列测试。动画模式和固定原件差异必须前置探针解决，不预称性能/美术/Gallery通过。

2026-10-29 合同预检：三份Plan的0..10节、各10任务/8AC/5V、work/review/merge职责、依赖与SD引用检查通过；本批49个局部链接可解析，新增入口均标待建。此为文档预检，非实现测试或独立final review。

2026-10-29 work T-01 checkpoint：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: progress | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe3095892fcf5507dd1b9e26af523111fd4 | code_commit: 11bc18bc0c0b5d8cde54c2ef09a98f27f2df68d9 (source receipt only) | dependency_revisions: 005=351919c76b4ef1d940fc7b8120254e321c41cba2,004=e0fce138/fix0261ae9,SDK7b987ca8/681,two locks unchanged | task_ids: T-01 | evidence: attachments/006/work/source-preflight.md; verify_plan005 --controls exit0 1456 hashes/all controls; prepare_sdk --from-snapshot exit0; raw-source audit exit0 8nodes/4LINEAR clips/44channels/337samples/6456B, local report SHA2560ca84f059687a7e472d5750e4fc7553858edcd5c19953d18067b19e1973f24f4 | blockers: none | next_responsible_role: work T-02`。历史new记录的005等待已由实际归档/交付/清理收据解除，非用户待答；无relay并行记录。唯一树clean，未修改产品/SDK/锁或其他计划WIP，GPU/实际Auto整组合与其预算均尚未验收。

2026-10-29 goal round 1：`stage: goal | plan_id: PLAN-006 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass (T-01 only; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 11bc18bc0c0b5d8cde54c2ef09a98f27f2df68d9 | next: work T-02`。1/10；全链授权已记§4，未缩减合同，非等待轮。

2026-10-29 work T-02进展：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: progress | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe | code_commit: c7dceef2f64a76589aad4eb2473dc3b904ddb2cd | dependency_revisions: 005/004/SDK/两锁与T-01相同 | task_ids: T-02 (partial) | evidence: attachments/006/work/node-animation-contract.md,core-probe-progress.md | blockers: none | next_responsible_role: work T-02 Host/resource/GPU`。本轮建立有限TRS通用核心和公开Auto视觉producer，既有14profile装配字节保持。`cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --locked --offline --test node_animation -- --test-threads=1` exit0 4/4（原四clip/层级、独立slerp点oracle/STEP/边界、19结构+4二进制反例、8192额度/原matrix父）；`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test public_node_animation -- --test-threads=1 --nocapture` exit0 3/3（真实Auto运动含垂直位移+grounded+相机读取+触发一次读取+单visual setter连续3tick，79567B受管源/538B用户源；坏参数/旧token；共享32credits超额error10负控）。assembly定向4/4、旧profile hash定向1/1及两包fmt/diff检查通过，新产品代码无warning，SDK既有146 warnings未改。最终日志均在`D:/autostack/plan-artifacts/auto-ui/plan-006/`：node-core-final.log SHA256 ac4b7a533f27f520c6fa0da15ea9e8b6109062510fb9cfd5f6c65c24cd860747，auto-core-final.log e1a5cdc70385138a5c00b3a85a429df980385fa06336ff3e921e6f33c7770c7c，assembly-compat.log 57be5a37416dcaf931e6ac0440194e8bcac7f10d1a2bd9f11796d30c0ed45eb9，frozen-profiles.log 2553258bce0433ab9ce167af92bd18f97214f0730764fae7c6449c93ad28b9e7。尚未接入生产assets/Host/Frame/GPU，不能使用本轮结果代替完整T-02预算/实际像素/CPU步数或AC完成；T-03..10仍开放。

2026-10-29 goal round 2：`stage: goal | plan_id: PLAN-006 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass (verified T-02 progress; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: c7dceef2f64a76589aad4eb2473dc3b904ddb2cd | next: work T-02 integration`。前轮有效进展，本轮新增实际实现与可区分已执行证据，非等待/无进展；保留1/10，不提前复审/合入。

2026-10-30 work T-02完成：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: pass (T-02 only; work incomplete) | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe | code_commit: 9354e9ef482f433257c3445a6472d37a027adb44 | dependency_revisions: SDK/两锁/原件身份与T-01相同 | task_ids: T-02 | evidence: attachments/006/work/t02-integration.md,node-animation-contract.md | blockers: none | next_responsible_role: work T-03`。最小整链实际交付：显式animated closure身份/公开import-animated/保存搬迁冷重开4clip→真实Auto运动/垂直位移/grounded/相机relative basis/触发一次读取/单visual setter→Host独立wire验证与采样→同tick Frame/Prepared→颜色/shadow共用矩阵。旧import/旧Play拒新动画payload，保留005旧identity。public_node_animation 4/4已过；最终Host case定向1/1：19坏wire+最终矩阵溢出拒、原GLB不变、共享节点payload静态/实例独立、去掉node delta且固定相机/位置/yaw/纹理的真实GPU负控必变图，mesh上传/解码零增量；单tick3836us，原运行限额未放宽；资源union raw2/97732B、CPU1102359B、GPU1101700B。最终日志`t02-final-profile.log` SHA256 2d0085e066ffe57fa7911091f422b19c5f9d1db5c4dd6ae2bb4254a0db37ffc4（目录D:/autostack/plan-artifacts/auto-ui/plan-006）；4/4历史`t02-closure-final.log` d560afc0456d28b3496f670bb5f4a1fd6b6ce7d0ddd2cc1db5e5aab90e21784f，其余3case producer代码/输入未变复用。005共享保存/identity定向1/1，日志`t02-legacy-closure.log` SHA2568c9d9823d8a6d00d72f6c253441b788c1f45f1773e199350f76c9994ea5cd3c8。初次测试碰撞AABB未按原character域配置及未dirty重绘的问题均按正式路径纠正，保留失败日志，不改验收。2/10；T-03..06完整准入/像素物理/owner/消费验收尚未关闭，未交review。

2026-10-30 goal round 3：`stage: goal | plan_id: PLAN-006 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass (T-02 complete; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 9354e9ef482f433257c3445a6472d37a027adb44 | next: work T-03`。前轮有效实现进展，本轮取得生产整链/真实像素/独立坏输出证据与提交，无用户待答问题；目标和预算未缩减。

2026-10-30 work T-03完成：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: pass (T-03 only; work incomplete) | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe | code_commit: 27393245118b0708b6f0b9c7ede66025d130598a | dependency_revisions: SDK/两锁/固定原件未变 | task_ids: T-03 | evidence: attachments/006/work/t03-admission-and-storage.md | blockers: none | next_responsible_role: work T-04`。生产准入23结构+6二进制反例正确分类且0关键帧分配/0图像geometry解码；原件恢复44channel真分配；8192/8193 samples、16clips、128nodes/channels、STEP/无纹理多层及保存冷重开通过。Animation精确预检与完整resident/依赖union门先于key/output载荷；64MiB满resident拒且0载荷分配、48MiB正控接受、暖Asset/Animation Arc共用。pose元数据/key/Arc/header/target/wire补齐：旧reserve21888B、实际headroom25501B、新reserve29112B，隐藏也拒；实际变异回旧公式该永久用例exit101，字节还原后exit0，加65536B余量正控绿。5保存故障/重试/pending损坏PNG verify/修复、暖PNG与GLB原字节身份及profile降级连id重算均真实拒绝，修复后冷开4clip。core5/5含预检字节/大有限LINEAR中点；Host受影响重放1/1，19wire/去轨道实际GPU负控、同帧与搬迁重开保持，tick3617us。driver测试在包目录以正式栈配置运行，PE头实读33554432B；worker配置16777216B；无warning新增，两包fmt/diff干净。日志均在`D:/autostack/plan-artifacts/auto-ui/plan-006/`：t03-resources-final.log（4/4，未变3case复用+storage后定向更新）SHA25627a90289b68091e24eff7569c7b517fd4193597e3a14214fef8cddc9261a5945；t03-core-final.log 49e1f4ca359a9ffb2a47def20435369bb114f38730d334d2da566880289cfbaf；t03-reservation-negative.log 0180f05b317aa6feb8233d5d6033148ef7cbc91c60eb820d6d0d9ab37a207092；t03-reservation-restored.log 3b28f6889c698c394ab8c8a9f2ac6c7f1fbe6c894a4b75f92b2f66ace21b71a5；t03-storage-final.log bbe6c03fd8bcdd65355508c5380f3252eedcd77a68e874691e279d1d9179b529；t03-host-replay.log 80bb38d9871d2adb7080fa080c1c60a023a8c4e16296ed6c7dc3a3e8ed91df95；t03-stack-profile.json d4429dba411d89a7ad82167bd55e92b1b32afc0e3783e692aa58e7b25d35b170。T-04..06/独立review/merge均pending，不把本准入成果冒充完整游戏画面/owner/发行通过。

2026-10-30 goal round 4：`stage: goal | plan_id: PLAN-006 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass (T-03 complete; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 27393245118b0708b6f0b9c7ede66025d130598a | next: work T-04`。前轮进展有效，本轮完成生产边界与预检/总账修正、实际负控与持久化证据；3/10，无用户待答/无范围变化。

2026-10-30 work T-04进展及needs_replan：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 1 | outcome: needs_replan | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe | code_commit: ccfe6d9c0e80616af344ce8898a674a380903ffe | dependency_revisions: SDK/两锁/固定原件未变 | task_ids: T-02 reopened,T-04 partial | evidence: attachments/006/work/t04-pixels-and-input-diagnosis.md | blockers: none (bounded design revision within existing scope) | next_responsible_role: new`。T-04已实际核原4clip的20时间状态+非零phase立即切换、共享asset独立实例：独立原JSON/BIN f64插值/层级，不复用生产sample；43596顶点误差max2.4978897e-7<2e-5；2074207稳定像素probe通过；去node轨道故障使idle/walk/jump的独立轮廓错644/1277/215像素，均拒，21PNG只本地保留。非均匀缩放法线独立GPU反例119≠预期163已用inverse transpose修复，最终163±2；旧005深度/双面4朝向/2顺序/1reveal定向绿max_error0。实际日志目录D:/autostack/plan-artifacts/auto-ui/plan-006/：t04-pixels-final.log SHA25600ef4c3bb76ece0017ecaa915c358fb1685ef7a7d1585fa7d03fdff53534f1b4（2/2）；t04-normal-red.log d327f84c5e7262e7b2a989bba125e84b83dc6d3a6d0a770df976389aab1fa253（预期红）；t04-legacy-normal.log 588e717dfa041d85dd17a665af848a15c6ff1bb3e622ccd87c11754b844e946a（1/1）。修复影响的旧T-02/T-03像素hash只作历史，不作当前颜色hash；几何/准入/存储/owner路径未改。

F-006-W1-INPUT-BUDGET：补入真实4向held+跳跃pressed后，即使不读event pair/phase，更弱的move/grounded/event_next/visual组合已经实际error10；`public_node_animation::full_input_movement_trigger_and_visual_combination_exposes_scalar_read_budget_limit`在真实Auto Session运行exit0（断言预期error10），日志t04-scalar-input-budget-counterexample.log SHA256ac7ae59e56d0c733e8c35fdf8f655ccffb87cd1cd103d2d946aa9c5bf3b0e030。T-02旧probe读相机basis但用固定运动向量，漏算既有逐键1credit，故其“完整组合预算通过”结论失效，T-02重开；素材/采样/同帧/wire/保存证据保留，T-01/T-03仍完成，计2/10。按§5.1原定路由回new，不增加32credits、不省输入/相机/触发/原clip、不转Rust玩法。候选是在尚未发布的新profile追加成本1的6held+6pressed聚合读取（旧14profile/旧getter价格/输入语义保持），新完整组合可1+26+1+1+1+1+1=32；该候选尚未实施，须new有界修订合同并明确mixed/重复消费与neutral/fresh Run oracle后work。现有全链授权覆盖相同目标/验收/范围/预算的设计修订，不需用户重批。

2026-10-30 goal round 5：`stage: goal | plan_id: PLAN-006 | plan_revision: r1 | round: 5 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: ccfe6d9c0e80616af344ce8898a674a380903ffe | next: new (bounded input design revision)`。本轮有独立源数据/GPU证据、法线修复及改变下一动作的实际预算反例，属有效进展；并非用户等待或外部阻塞。T-04完整朝向/物理/阴影、T-05..10仍开放。

2026-10-30 bounded new r2：`stage: new | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: pass (contract ready, not implemented) | base_commit: ccfe6d9c0e80616af344ce8898a674a380903ffe | worktree: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | changed_design: new-profile-only bounded action snapshot, original32credits and all acceptance/scope preserved | affected_task_ids: T-02,T-04,T-05 input state,T-06 current identity; T-01/T-03 retained | evidence: attachments/006/work/input-budget-revision-r2.md | next: work T-02`。已核canonical三份Spec及实际GameInput.fold→Play输入调用→character3d私有held/pressed→旧1credit getter路径。新profile聚合一次有限输入读取，不改旧价格，不增Host玩法/wire；0..4095编码、混读/重复消费、neutral/fresh、完整键控/真实事件分支和额外1credit负控已在§5/6绑定S-06，源/CPU/内存上限不改。原预算pass作废，其余原件/准入/存储与采样/像素证据按相关算法、输入、依赖不变明确保留；新API/新组合尚未实施/运行，T-02不勾选，2/10。实施树HEAD clean ccfe6d9；本轮仅主Plan/执行合同文档，无代码/Spec/ledger/构建/GPU写入。

2026-10-30 goal round 6：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 6 | from: new | to: work | status: executing | outcome: pass (bounded contract revision; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: ccfe6d9c0e80616af344ce8898a674a380903ffe (frozen implementation baseline) | next: work T-02 then T-04..06`。前轮实测反例为有效进展，本轮完成其有界设计和可执行关闭合同；保持原授权/范围/预算，无用户待答问题，不将设计可行和式当作完成。

2026-10-30 r2合同预检：现盘校验exit0：0..10节顺序、10个唯一T/8个唯一AC、2项已完成与current_step一致、S-06单一当前行、9个本地Markdown链接均可解析；实施树HEAD仍ccfe6d9且clean，源内尚无ag_actions_snapshot定义，符合“待work实现”。修订附件SHA256659e4c3bd353ec5c0a622496b6897eb59ccfcb4b4cc9c1f4261aa7019d2e6a66；本轮不改实施、canonical或ledger，不运行构建/测试来冒充新功能已过。共享主检出另有其它计划README文档WIP，定向diff检查只覆盖本Plan文件，保留其原有改动。

2026-10-30 work r2 T-02/T-04/T-05完成：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: pass (T-02,T-04,T-05 only; work incomplete) | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe | code_commit: 04042f827a1efd68b3e9d2188b97b2fa8f4e2100 | dependency_revisions: SDK/两锁/原件身份未变 | task_ids: T-02,T-04,T-05 | evidence: attachments/006/work/r2-input-pixels-and-lifecycle.md,node-animation-contract.md | blockers: none | next_responsible_role: work T-06`。F-006-W1-INPUT-BUDGET已闭：新profile独占1credit聚合读，旧14profile装配hash保留；原逐键反例仍10、新真实事件分支完整32成功、额外1失败且整组旧画面/位置/相机/事件/pose保留。managed80021B；真实tick4607us；raw2/97732B，CPU含animation1102359B+pose32240B=1134599B，GPU1101700B，原额度不变。public8/8；lifecycle当时4/4+独立新增四向位移1/1（未声称曾执行5/5整文件）；240tick开关动画对比floor/wall/platform与trigger完全相同；20轮Play/restart/Stop释放40VM/pose/descriptor，warm mesh/texture上传和decode均0增量，最终外部pin释放真实weak失效。四向/stop/±pi固定相机独立GPU轮廓<=2误差，错yaw负控1341/1380/1486/1458；STEP旋转GPU边界与持有通过，错插值608；原walk同pose shadow独立87196ROI/2171阴影内部错误0，冻结pose负控263/关闭shadow2171，已目视角色与阴影。日志命令与7个SHA256逐项见附件；日志/PNG本地D:/autostack/plan-artifacts/auto-ui/plan-006不入Git。原四clip21状态/43596源顶点/2074207ROI、T-01/T-03按未变算法/输入/依赖复用。fmt/diff exit0，无本轮新增warning（SDK146旧warning保留）。实施已提交且clean，共享主检出其它计划WIP保留；5/10，T-06完整消费/身份/SD与review/merge仍pending，不提前execution_done。

2026-10-30 goal round 7：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: pass (T-02,T-04,T-05 complete; work incomplete) | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 04042f827a1efd68b3e9d2188b97b2fa8f4e2100 | next: work T-06`。本轮有界输入修复与当前关闭反例、可见朝向/阴影、物理/所有权均取得实际新证据，非等待/无进展。AC-05的仓外fresh consumer/current identity及完整性能/SD尚待，不转交复审裁定未完成工作。

2026-10-30 work T-06完成/ready-for-review：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe3095892fcf5507dd1b9e26af523111fd4 | code_commit: 475d376978c6af5acc87a73b277fc68c018c21be | handoff_commit: 3a6404ef24468cad081ec3332acdef83230b04e7 | dependency_revisions: SDK7b987ca8/两锁/005delivery351919c76b/004fix0261ae9未变 | task_ids: T-06 (T-01..05 current replay) | evidence: attachments/006/work/t06-consumer-and-identity.md | blockers: none | next_responsible_role: independent final review T-07`。源身份50a87a3c973b93f4a961fc00b81b39788cbd785f3775fba01811f6edc192a5da，1477完整文件均Gittracked，SDK681原字节/17本地包/29路径边、479 metadata packages、禁用UI/Man/旧生产包0。公开bound scene保留87624B原character四clip+PNG、内容寻址Auto；真实用户速度4→1变异改轨迹，60tick公开入口/保存A搬迁B/新Registry冷重开/初pose通过。最终仅用t06-closed-*：新30case+旧14profile/真实sprite/纹理项目3定向case=33通过，无新模块过滤/忽略；identity变异/删除/集合/pins/escape/PID与真实Auto blob LF→CRLF负控拒；gate stale/partial/runner/zero/pixel/warm/CPU/resource负控与实际零选执行均拒。producer/Git blob/最终仓外B身份文件逐字节相同；SDK/raw行为blob按原字节，身份LF稳定，历史001..005收据不改。仓外源码A→B重新prepare SDK、verify/source与公开项目check后locked/offline重新编译51.28s/执行10.79s，consumer2/2，实际exe manifest_dir在B，PE33554432；用户60tick6370steps/7slices、managed80021B、暖上传0。regular3/wide120(+8slots)/deep64分别7025/91382/68304steps；producer中位4106/42772/26949us，max4559/46616/36868us；deep120按原100000步guard拒且last-good保持，不把debug宽布局称60FPS。CPU含pose1131471B，GPU1101700B；原额度/源/CPU不变。B运行后producer冻结报告validate仍通过，两包fmt/差异check通过；SDK146旧warning保留、无新交付warning。SD-01..03与下游profile/签名/wire/限制/样例入口已备在research/node-animation-spec-delta.md，canonical/ledger未发布。最终log/manifest/portability-audit路径及8个SHA256见附件；早期identity换行与blob检查问题已实际修复并重新冻结/全新消费，不重标旧报告。AC-01..05 work-owned当前通过，开放finding/覆盖缺口/合同决策为空；实施已提交clean。T-07独立逐AC/V/S/SD审查、T-08新冻结merge门/有效复用、T-09规范ledger、T-10归档/guard清树仍pending，执行的work声明门不能代替review verdict。主检出其它计划011/012/013及README/.next-id文档WIP保留，无本期根UI或SDK代码改动。

2026-10-30 goal round 8：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 8 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 3a6404ef24468cad081ec3332acdef83230b04e7 | next: independent final review T-07`。前轮为有效实施进展，本轮补齐完整消费/身份/原CPU包络/SD并关闭实际字节稳定性问题；6/10，无用户待答。保持原全链授权及范围，不提前合入或声称目标已达成。

2026-10-30 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent agent /root/review_plan006 (separate implementation context; model/session identity not otherwise asserted) | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | reviewed_commit: 3a6404ef24468cad081ec3332acdef83230b04e7 | base_commit: 80d9bbe3095892fcf5507dd1b9e26af523111fd4 | dependency_revisions: 005=351919c76b4ef1d940fc7b8120254e321c41cba2,004fix=0261ae9dacf786beb6dceb630be6af267f242625,SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/two locks unchanged | spec_inputs: auto-lang d54c257a7c126731c2d0a3c6d5250ebecfc44c7a; six raw hashes in attachments/006/review/r1.md | acceptance_results: AC-01 fail,AC-02/03/04/05 pass on frozen baseline,AC-06 fail,AC-07/08 merge pending | pending_closeout_gates: T08 new frozen targeted gate/valid reuse before merge,T09 SD/derived ledger/index/archive/delivery,T10 guard cleanup | findings: F-006-R1-UNUSED-SAMPLER(P2,AC01,T03;T06 dependent evidence) | evidence: attachments/006/review/r1.md; external review-r1/baseline-audit.json SHA25696a7fe2185eb68e01b551b62611bbec9112097c032bd640833b4a890e788f626; frozen SD SHA2563296188be91b303723235396a2394bda22c54c499b05cb88d43f0e245ecc9859 | next: work targeted admission repair then affected final re-review`。完整检查全部AC/V/S、固定原件、资源/采样/真实GPU色图阴影/朝向物理/输入32credits/事务owner/消费/旧兼容/身份及validator控制；同源closed门重新validate+controls exit0，无完整回归重跑。独立正式import反例：未引用LINEAR sampler的重复/负/NaN时间、output count错/NaN/SCALAR均被接受并decode_delta2；同数据被channel引用正确invalid拒且decode0，unused合法接受、cubic与坏span拒均为控制。具体fixture/生成器/入口/期望与实际/日志hash/共因及局部关闭oracle完整保存在review/r1.md与仓外review-r1。不新增未引用sampler样本quota，不改变8192按channel累计口径。T03/T06重开，T01/T02/T04/T05未受影响保留，4/10，状态executing；实施HEAD/clean及1477源身份复核未漂移，未修代码/SDK/锁/规范/ledger或提交。现有绿色冻结报告不覆盖新发现的声明路径，不能给pass或提前merge。

2026-10-30 goal round 9：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 9 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: 3a6404ef24468cad081ec3332acdef83230b04e7 | next: work F-006-R1-UNUSED-SAMPLER, then affected final re-review`。上一轮为实际工作/消费证据进展；本轮经用户明确授权独立子代理完成final R1，取得正式生产入口的可复现时间/output共因反例与完整其余矩阵核验，属于改变下一动作的有效进展。R1报告SHA256 c61557fdb07a6a82c8a82dbecf96ddec02f2bc09cfc1430cfe775f4f442d31a8；唯一P2 finding重开T03/T06至4/10，未修实现/变预算/扩大范围，未发pass/merge。实际review上下文曾被系统中断，恢复同一代理并复用其已存原始证据后完成；未用中断作产品阻塞或新增复审次数。当前第1次failed final review/第1个有界repair cycle，后续依goal cap12继续work→review→merge，不重置计数。

2026-10-30 work R1 repair ready：`stage: work | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | base_commit: 80d9bbe3095892fcf5507dd1b9e26af523111fd4 | repair_base: 3a6404ef24468cad081ec3332acdef83230b04e7 | code_commit: 82ba57e8e51b83a3715aabd03ca0b9f4f38b08b7 | handoff_commit: a85980c1c123c793188623d9a5d3df8dd117d2d7 | dependency_revisions: SDK/two locks/005delivery/004fix unchanged | task_ids: T03,T06 | evidence: attachments/006/work/r1-admission-repair.md | blockers: none | next_responsible_role: same independent final reviewer R2`。唯一finding F-006-R1-UNUSED-SAMPLER共因已补齐：所有声明sampler先做type/span/count共同元数据准入及严格非负递增有限keys/finite FLOAT VEC3/VEC4 output，再按channel角色检查scale/unit quat并retention，used/unused共用sampler_key。合法unused不误拒、不留track、不改四clip duration/pose/CPU账，不新增unused quota，不改变8192按channel累加。永久新case22个used/unused反例均0channel allocation/0decode拒；4个LINEAR/STEP/VEC3/4合法unused、2个共享sampler/不同node与translation-scale角色正控、target-specific坏scale/quat拒、8192channel之外unused keys仍接受；原R1 14个冻结GLB正式import重放13拒/1接受。资源共同admission模块6/6（最终只扩新case duration/pose断言后该case1/1重放，其余5未变复用），AGP core5/5、identity --controls及SDK/source通过。现源身份e2994efa1b1ee6d55f79de7bd194bf59c1745c41f60b6c8bbaac7a67197e7243，1477文件/SDK681/17local/29edges/479metadata与两锁保持；SD-01补充既有unused/target角色规则，新SD SHA2563d6af002bef2c30fb84417287be8422f13468f2b82f6b18f7d2734bbf28e7621，canonical/ledger未写。全新source-only r1-repair-a→b，包内SDKprepare/sourceverify后，B离线locked重新编译41.07s/执行4.82s，公开bound项目60tick/Auto速度变异/SaveAs搬迁/冷重开4clip初pose1/1（1filtered，未重复CPU矩阵），实际manifest_dir在B，6370steps/7slices/managed80021/warm上传0，B/producer/Git身份原字节一致。新6资源case令最终门计数变新31+继承3=34；修复期未重复整组GPU/20owner/CPU矩阵或给旧33报告重标身份，T08在R2后核新冻结门/有效复用。Auto/Host/sample插值/renderer/physics/VM/owner/SDK/锁及固定原件未改，R1已确认未变AC02/03/04及AC05输入/CPU/旧兼容证据据此保留。两包fmt/diff exit0，SDK旧146warning保留、无新交付warning，实施已提交clean；work-owned开放finding/覆盖缺口/合同决策为空，T03/T06恢复至6/10。局部命令、实际数值、8份外置log SHA256及完整closure表见附件；未提前发独立pass或合入。

2026-10-30 goal round 10：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: a85980c1c123c793188623d9a5d3df8dd117d2d7 | next: affected independent final review R2`。前轮独立反例为有效进展，本轮第1个有界repair cycle完成同机制/兄弟路径与正式原始fixture重放、新当前身份/SD及受影响仓外消费；没有改目标、AC、额度或收缩支持。R1原始失败记录保留，review计数不重置；仍待T07..10。主检出并发的其它Plan/design/README文档改动保留。

2026-10-30 independent final R2：`stage: review | review_scope: final | reviewer_identity: same independent R1 agent /root/review_plan006, separate from implementation; no other model/session identity asserted | plan_id: auto-ui/PLAN-006 | plan_revision: 2 | outcome: pass | worktree_path: D:/autostack/.wt/ui-006/auto-ui | branch: codex/plan-006-dev | reviewed_commit: a85980c1c123c793188623d9a5d3df8dd117d2d7 | base_commit: 80d9bbe3095892fcf5507dd1b9e26af523111fd4 | repair_base: 3a6404ef24468cad081ec3332acdef83230b04e7 | dependency_revisions: 005=351919c76b4ef1d940fc7b8120254e321c41cba2,004fix=0261ae9dacf786beb6dceb630be6af267f242625,SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/two locks unchanged | spec_inputs: auto-lang3e2b8b62053ab1724454bc6fe252987ed39c6ab4; six relevant raw hashes identical to R1, table in attachments/006/review/r2.md | acceptance_results: AC01..06 pass,AC07/08 merge pending | pending_closeout_gates: T08 new34case frozen gate+controls/valid reuse bound to e299 identity before merge,T09 SD/derived ledger/index/archive/delivery,T10 guard cleanup | findings: F-006-R1-UNUSED-SAMPLER closed; no open findings | evidence: attachments/006/review/r2.md; review-r2/baseline-audit.json SHA25621da77e1cbb1a64779105746670200f16e6ab87c164277e93c4382368a6ce7ed; frozen SD SHA2563d6af002bef2c30fb84417287be8422f13468f2b82f6b18f7d2734bbf28e7621 | next: merge T08`。独立在当前代码重跑新resources case，原14冻结GLB与22个used/unused反例/4合法模式/共享target/8192口径通过，1/1（5filtered）；另新编译链接当前producer的外置probe，独立7个末尾clip/非零offset/最后VEC4分量Inf/跨clip共享accessor/前插sampler索引重编号用例全部符合：3坏例invalid且decode0拒，4合法例原44channels、四clip duration、phase0/.19/2 pose及retained CPU bytes相同。当前identity verify exit0；逐文件核1477集合/Git/B和原14hash；旧50a/33整体报告在当前validator预期exit1拒绝stale身份，未给旧报告换标签。源码实际仅共同admission变化，sample/Auto/Host/renderer/physics/VM/owner/SDK/锁/素材均未改，故R1已核其余AC/V/S逐项有依据复用；受影响core5/资源6（最终新case扩断言1/1）与新B消费1/1（1filtered）原日志/数值/hash核验成立，不重复GPU/20owner/CPU矩阵/完整34case门。SD新增四行忠实阐明既有unused/target角色口径，三个目标/GOAL-008正确，无合同删弱或升额度。实施HEAD/clean结论前复核无漂移；T07完成，7/10、reviewed，未合入/发布规范/提交或清树。

2026-10-30 goal round 11：`stage: goal | plan_id: PLAN-006 | plan_revision: r2 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-006/auto-ui | commit: a85980c1c123c793188623d9a5d3df8dd117d2d7 | next: merge T08..10`。上一轮为实际修复与当前消费证据进展；本轮原独立上下文R2按受影响面重放原始输入和7个自选兄弟控制，唯一R1 finding已闭且无新finding，全部work/review-owned AC通过，7/10。R2 Markdown SHA256 b5958ab5c8e3371f40045c6e898ccde811b5e3221dcfa926276628203eb51897、独立audit21da77e1cbb1a64779105746670200f16e6ab87c164277e93c4382368a6ce7ed、SD3d6af002...均已核原文件；实施HEAD/clean与e299身份未漂移。下一轮必须执行新34case冻结门+controls后才合入，再规范/ledger/索引/归档/delivery/guard清理；不以本pass提前算目标完成。用户已授权的全链与独立review授权继续有效，无待答问题；保留R1失败计数和所有历史证据，不重置goal cap12。

2026-10-30 `stage: merge | PLAN-006:r2 | round: 12 | prepared: verified | landed: verified | ledger_refreshed: verified | archived: delivered | cleanup: pending`。独立R2冻结a85980c，SD3d6af002bef2c30fb84417287be8422f13468f2b82f6b18f7d2734bbf28e7621；rebase至主干6a8db648，只保留主干已更新合同文档，9个实现提交range-diff全部等价，experimental整树仍2b31946b0c129c0d549ce8286bd952455f3bc7a7。旧→新：c7dceef→d5b32c1、9354e9e→1b58c89、2739324→a28025b、ccfe6d9→edcc9ac、04042f8→8586b20、12414c0→ee160d8、ab48d2e→e5e2a01、475d376→26e72a6、82ba57e→c681663。主干ff-only实际交付c681663e6ff392d70471abbec560e9d9daf12622，无merge commit；pre-landing.bundle已验证可恢复旧review基线。

当前merge-gate全新冻结34/34（新31+继承3），public8/resources6/pixels5/lifecycle5/consumer2/core5/旧profile1/sprite1/texture1，新组0filtered，identity、runner、log/exe/count、PE32MiB/worker16MiB、oracle和预算负控及真实零选拒绝均通过，exit0。源e2994efa1b1ee6d55f79de7bd194bf59c1745c41f60b6c8bbaac7a67197e7243，1477文件，SDK681原字节、17包/29边/479metadata及两锁未变。实际main上verify_plan006与样例generate_project --check均exit0。正式规范在同组auto-lang独立树准备并精确核SD后，458276fd38b844b1f769cec50c7d9276e5b08fdf提交规范，664145b791a3bb2576a13414e0c2929203c74e56提交台账并ff-only入auto-lang/master；GOAL-008保持ongoing，AG/Gallery/G3/G10b与完整游戏pending。仅文档投影依AGENTS无需root测试，spec-index 62项目/14既有未分组警告，规范索引内容无实质变化。

派生台账通道为guarded offline projection（无可达store writer，非HTTP成功），精确备份SHA951691355d75802680a0c275965a3c6f4089c2111ae5cb78036c27a8a8d55014；version142→143仅+1，六段完整schema/ID/file/tags回读通过，1449无关条目及其它顶层/段元数据逐字节保留。更新P838-ASSETS/P840-BEHAVIOR-API，新增P006-NODE-ANIMATION/P006-T/P006-R2/P006-M；最终ledger SHAde927a0be1073ed91ec6888e1c0a11c89f36f064bb7d4ddaa4c69c8f0ece3c34。review/report链接指向本归档文件；规范为当前知识源，无复制Plan章节。全部原始日志/manifest/备份/源码在D:/autostack/plan-artifacts/auto-ui/plan-006，未入Git。

实盘证据SHA256：
- `D:\autostack\plan-artifacts\auto-ui\plan-006\merge-gate\manifest.json`：`ea1e56943473e9383301e7a99264761d999de926b283d0cd970be28688579044`
- `D:\autostack\plan-artifacts\auto-ui\plan-006\merge-gate.log`：`14c3ea4606a08beb048021cf7aa551c5908575fb8f3beeb185c309a3b672195b`
- `D:\autostack\plan-artifacts\auto-ui\plan-006\landing-range-diff.txt`：`8fba087b89d77074276a69106d6c1379b385cb4cf7558283d7ba76c5daf19061`
- `D:\autostack\plan-artifacts\auto-ui\plan-006\ledger-projection.json`：`cc55e280133e57ffd2d00b08717b4cb3ad28bf9e9290e88999f825c8c4a6cda9`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\node-animation-and-facing.md`：`42db8e3774d7bf6dba1d0ee8dda7ad2590a003ed67af79fd2d52042364933822`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\behavior-api.md`：`3a1ead71ea2b21f14c199915c7043a12d55dae0642174815c73f39abcd8ec1bc`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\scene-assets.md`：`b5362e41bb90ec6c9414792888a052ea566035f345d71732d8066a842f0b0a14`

归档收据以本文件实际提交为准；工作树、分支与构建中间物清理完成后另记cleaned。生产backend/daemon/root UI未变，无需部署或重启；本期实验AG输出保留。语言批量回归按既有明确排除/延后不运行，不记为通过。

## 10. 待澄清事项

当前无用户待答问题。T01..09已交付验证，9/10；仅T10安全清理与最终收据待完成。本计划只交付节点动画/视觉朝向引擎能力，后继产品里程碑保持开放。

原规划的005交付身份与原clip实读风险已关闭；如后续发现必需clip需本期外插值或无法在既有预算内组合，仍带原件与反例回new，显式提出合同/预算变化，不能丢动作或换成Bob动画。声音/HUD、G09/G10内容、完整两关与最终发行另立后继合同，跨端不插入桌面M2前置。
