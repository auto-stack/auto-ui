---
plan_id: PLAN-006
status: executing
feature_name: AutoGraphics无蒙皮节点动画与视觉朝向
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-30
plan_revision: 1
current_step: 3
total_steps: 10
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

2026-10-29 执行授权：用户 `/goal 完成AG计划006` 授予本计划 work→独立review→有界repair→merge/规范/归档/guard清理全链授权，后续阶段继承；目标、范围、验收与预算保持r1。005现已实际交付，前置核验与原件实读见[执行基线](attachments/006/work/source-preflight.md)。

2026-10-29规划背景（历史，当前执行进度见§8/9）：用户要求规划下一阶段；规划轮仅合同/设计/引用，无实施、agent、窗口/GPU/Cargo门或发布。来源和指纹见[基线](attachments/004/planning/baseline.md)，资源与阶段边界见[设计](../design/autographics/m2-presentation-and-resources.md)。当时005/004尚未交付，未填写未来SHA/接口价格；实际前置现已由T-01核验。

现有权威：[样本研究](../../../auto-lang/docs/specs/auto-graphics/m2-game-sample-research.md)、[游戏分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)、[运行角色](../../../auto-lang/docs/specs/auto-graphics/game-actions-and-character2d.md)、[触发新局](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[静态资源](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)。研究记character含static/idle/walk/jump、无skin；具体轨道interpolation/keys/时间/层级尚须读取固定原始字节，不能从“有动画”推断是骨骼或只需移动根。旧858源清单缺失风险由005 T-01先恢复，006再核该收据，不换来源。

## 5. 详细设计

### 5.1 前置与首个可执行入口

T-01只接受005已合master、独立review与冻结门完成的实际delivery，以及004/005资产/SDK/两锁/consumer身份、素材逐文件来源/许可/closure表。T-02先完成`attachments/006/work/node-animation-contract.md`和整路径probe：纹理character原始clip→Auto同时运动/跳/相机/触发/一次读取/视觉setter→采样→同帧输出→实际内容。固定profile/CLI、wire表、clip稳定ordinal/名称、phase单位/loop/one-shot、计价和所有output校验后才进入实现。

保持32credits、原callback/step/时间与CPU目标、受管源96KiB/总352KiB、用户256KiB；不因006叠加再次抬限。应有单次有限setter设置clip/phase/yaw，输入实体绑定静态3D叶或合法001实例完整Run/slot/generation；旧身份拒绝，销毁撤销，复用槽从模板初始视觉重新开始。Auto计算角色状态、选择idle/walk/jump并由fixed tick推进phase；Host只执行明确的格式/采样，不按角色速度/grounded替Auto选动作。T-02证明真实组合预算不足则带反例回new，不能省相机/触发/原clip或转专用Rust业务。

### 5.2 支持表与确定性

有限无skin/morph的节点translation/rotation/scale动画；首版支持LINEAR与STEP，quaternion线性轨道按glTF语义最短弧slerp并正规化，不能逐分量lerp后当等价。keys时间有限、非负严格递增，output计数/类型和维度匹配，单clip同node同path重复轨道拒绝，动画作用节点与原始层级唯一映射。CUBICSPLINE/weights/skin/其他目标在本期显式unsupported；T-02必须先确认冻结原clip不依赖这些模式，若确有必需模式则回new扩合同，不能丢该clip或用未批准转换代替原件。

上限仍nodes128；新增有限域<=16 clips、<=128 channels/asset、全部channel累计<=8192 key samples；input时间/output TRS与必要索引/采样scratch按实际bytes计入既有CPU64MiB/GPU64MiB总额，不另给动画免费空间。005纹理/geometry/原始资源8身份32MiB、单raw8MiB/JSON1MiB、16 expanded primitives/512潜在mesh实例等上限全部保持。先按声明与span预检再分配；动画过程中不得制造超出已准入包络的资源。选定原件若越限只能回new，不临时松限。

phase由Auto显式提供的有限非负时间或等价整数tick表达，T-02选一种并固定，不能同时存在两套不一致时钟；本期数学语义为seconds下loop以duration取模、one-shot夹到末端，精确末端按loop回0/one-shot保持最后key，duration0单pose保持；切clip立刻采样当次给定phase。Stop/fresh Run恢复Auto app_start初值；Pause不更新phase，Step仅一个fixed dt；不以render次数/wallclock漂移。静态无clip保持基础pose。

### 5.3 视觉/碰撞/发布

视觉yaw为显式有界有限角度（T-02冻结单位与归一到[-pi,pi)规则），只绕公共模型视觉pivot旋转；不改变001实例位置或字符的运动方向输入，不将yaw写回solid/collider transform。Auto可据相机相对运动向量选yaw，静止保留上次合法朝向。父层级变换×视觉pivot yaw×动画节点TRS顺序写入合同；负缩放/奇异/非有限pose按原资产边界拒绝，不能进入颜色或shadow pass。不会为动画姿势重算角色碰撞：仍用冻结静态碰撞定义，G3支撑/墙/平台结果在动画开关/yaw变化下相同。Editor拾取/包围展示若需动画包络，与物理包络分别命名并定向核验，不偷改碰撞。

模型、UV/纹理、node pose、相机/角色/对象/事件一次prepare/publish。坏clip/phase/旧代/预算/prepare/OOM不发布部分pose，last-good整帧不变；Stop能恢复编辑。每帧仅更新有界pose/instance数据，不能逐tick重解码、上传mesh或texture；同一asset两个实例可独立phase/yaw而共享immutable payload。20轮Play/新局/Stop/关闭检查实际VM/pose/texture/mesh owner，合法Editor/外部pin保留，最后drop释放。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang知识仓） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-animation-and-facing.md | 无公共clip→有限TRS/Auto phase/视觉yaw/碰撞独立/事务 | M2 G3-01/07前置 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/behavior-api.md | 新显式视觉组合profile/价格/身份；旧装配不变 | 公共Auto而非Bob专用路径 | AC-02/03/05 |
| SD-03 | modify | docs/specs/auto-graphics/scene-assets.md | 新profile可声明受检动画；旧静态/纹理支持域继续显式分列 | 无静默忽略clip | AC-01/04 |

## 6. 测试设计

当前已建立并执行driver `--test public_node_animation` / `--test node_animation_resources`及AGP `--test node_animation`，`--locked --offline -- --test-threads=1`，实际结果/受影响范围见§9和T-02/T-03附件。driver正式运行工作目录为包目录以启用原32MiB主栈/16MiB worker配置。`python experimental/autographics-auto-driver/tools/check_node_animation.py --controls`与`verify_plan006.py --controls`仍由T-06待建，不提前称通过。只受影响面与可复现约1/10旧样本，新AC正反例不省；无窗口真实GPU单实例、OS抢焦点/置顶自动化暂停、全AG里程碑pending、根零改不跑根档。

| V/补充case | AC/owner_stage/启用任务 | 输入→实际路径 | 区分性判据/关闭条件 | 覆盖边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01 work T-02/03 | 固定character四clip、两层节点fixture、坏跨度/非递增时间/错计数/重复target/越限/skin/cubic | 支持原件真准入；unsupported与invalid/resource_limit区分且先检后分配；节点关系不烘焙丢失 | 如必需原件不在域内回new，不能无动画正控代替 |
| V-02/S-02 | AC-02 work T-04 | 真实Auto切idle/walk/jump/static、边界0/mid/end/loop、STEP/LINEAR旋转，两个共享asset实例不同phase | CPU独立矩阵/顶点期望与实际GPU轮廓/marker ROI同时敏感；改clip/phase应变图；去掉node轨道负控必红 | 不以tick/命令/hash变化代替动画画面 |
| V-03/S-03 | AC-03 work T-04 | 相机yaw改变/前后左右移动/停、±pi边界、动画开关、墙地接触 | Auto提供yaw后的角色面向可见；动画/视觉yaw变化不改相同输入的角色位置/支撑/触发；颜色与shadow同pose | 无根运动/姿势碰撞，非通用刚体 |
| V-04/S-04 | AC-04 work T-05 | 坏clip/phase/旧Run/旧slot代、prepare失败、Pause/Step、20轮新局/Stop/最终drop | 全旧或全新，neutral Step一次；新局初pose、暖mesh/texture零重传；真实weak owner/上传及预算账 | 合法外部pin最后drop后查，不测驱动内部保留 |
| V-05/S-05 | AC-05 work T-02/06 | 完整真实Auto32credits组合、fresh SDK consumer A→B、004精灵/005纹理/旧profile定向 | CPU/步数/源/内存实测与既定额、公开入口离线保存/重开、identity controls/源变异敏感 | 不等于完整G3关卡或G10b内容交付 |

同一唯一树顺序实施和独立上下文最终review，未指定实现者/模型，不为规划创建agent；独立review逐AC/V/S/SD核遗漏、warnings/fmt/动画静默替代、CPU与owner。review pass标merge冻结门pending，修代码后重新绑定证据/复审受影响面。

## 7. 验收标准

| ID | owner_stage | 结果与证据 |
|---|---|---|
| AC-01 | work | 固定四clip/层级/模式/数据/资源预检、原始身份与支持边界明确，V-01；T-03生产准入/边界/计账证据通过，见t03-admission-and-storage.md |
| AC-02 | work | 公开Auto切clip/phase，确定性loop/one-shot/双实例独立与真实动画，V-02 |
| AC-03 | work | 公共视觉朝向、相机相对运动示范、碰撞/支撑独立、同pose色图/阴影，V-03 |
| AC-04 | work | 同tick事务/身份/暂停单步/新局/Stop/owner/暖上传，V-04 |
| AC-05 | work | T-02整组合预算可行、仓外完整消费/重开与兼容/current identity，V-05；非Rust玩法 |
| AC-06 | review | 独立核全部AC/V/SD、无未处置finding，冻结门pending |
| AC-07 | merge | 新冻结提交定向门绿后合master，保留全AG里程碑pending |
| AC-08 | merge | SD/ledger/索引、本仓归档/delivery收据、guard clean清树，无Gallery/完整M2完成宣称 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 005实际交付、用户启动 | 核005/004/SDK/两锁/素材收据与owner，建唯一ui-006树 | AC-01/05 | [x] 11bc18b；source-preflight.md |
| T-02 | work | T-01 | node-animation-contract：原clip/interpolation、phase/yaw/碰撞/价格/预算整链probe，失败回new | AC-01..05 | [x] 9354e9e；t02-integration.md，原件Auto→Host→GPU、19wire负控/搬迁重开 |
| T-03 | work | T-02通过 | 有界节点/clip准入与持久化/资源计账，V-01 | AC-01/04 | [x] 2739324；t03-admission-and-storage.md |
| T-04 | work | T-03 | Auto公开视觉输出/Host采样/renderer色图与shadow，V-02/03 | AC-02/03 | [ ] |
| T-05 | work | T-04 | 事务/实例代次/新局/暂停单步/owner与20轮V-04 | AC-04 | [ ] |
| T-06 | work | T-05 | fresh消费/预算/兼容/identity controls、fmt/warnings/提交、SD与下游接口 | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立review全部V/S与遗漏，merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交定向门/有效复用，通过才合本仓master | AC-07 | [ ] |
| T-09 | merge | T-08 | Spec/ledger/索引、本仓归档与真实delivery SHA/clip接口/消费包身份 | AC-08 | [ ] |
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

## 10. 待澄清事项

当前无用户待答问题。T-01输入、T-02最小整链与T-03准入/持久化/资源账已通过，phase/yaw/wire合同已冻结。继续T-04..06各自完整验收矩阵，不将最小probe或准入成果当作后续AC完成。

原规划的005交付身份与原clip实读风险已关闭；如后续发现必需clip需本期外插值或无法在既有预算内组合，仍带原件与反例回new，显式提出合同/预算变化，不能丢动作或换成Bob动画。声音/HUD、G09/G10内容、完整两关与最终发行另立后继合同，跨端不插入桌面M2前置。
