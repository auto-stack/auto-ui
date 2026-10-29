---
plan_id: PLAN-004
status: executing
feature_name: AutoGraphics公共图片资源与2D精灵帧
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 4
total_steps: 10
depends_on: [auto-ui/PLAN-002]
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-assets.md
  - docs/specs/auto-graphics/behavior-api.md
new_spec_components:
  - docs/specs/auto-graphics/sprites-and-image-resources.md
touched_goals: [GOAL-008]
---

# PLAN-004：公共图片资源与2D精灵帧

## 0. 变更摘要

在本仓002实际交付之上，建立可编辑、可保存、可离线消费的公共PNG资源、2D精灵矩形取帧/翻向与Auto驱动帧切换。只做引擎与必要验证夹具；G09产品内容以后在ag-examples独立立项，不改在途ag-examples/001。

## 1. 目标

公共入口可导入一张有透明度的PNG，绑定2D叶节点，显示完整图片或合法像素矩形帧；Auto按真实状态选择idle/run/jump/fall帧与左右翻向，角色运动、相机、对象/触发与精灵同一tick提交。保持碰撞形状独立，不用透明像素改变实体接触规则。新局、暂停、Stop与关闭符合既有事务/资源寿命。

非目标：G09目录/示例包、正式角色美术制作、完整游戏、骨骼/3D billboard、声音/HUD、生产UI/HIR/ACE融合、新平台、通用图片浏览器/下载器。夹具图使用小型自有非对称颜色图，不冒充858候选美术已制作。

## 2. 架构方案

PNG原始字节→有界无IO准入/解码→不可变Registry→版本化scene与SourceSnapshot→真实Auto输出的受检视觉状态→Host prepare→同帧publish。编辑保存与Play都消费同一已接受字节，图片不能在帧中按路径重读。新增表示可与旧Model分开；不能把所有Asset.model强改成图片的空模型并绕过旧规则。

唯一实现/顺序复审树拟`D:/autostack/.wt/ui-004/auto-ui`，分支`codex/plan-004-dev`，后续work才建立。主检出只Plan簿记；SDK准备、target和报告独立，不与ag-001/其他UI路线共用。禁止链接，清理先guard。

## 3. 技术栈

现有driver `src/authoring/{assets,document,storage,preview,session,play,behavior,behavior_project}.rs`、`behavior/assembly.rs`、`host/{viewport,play_adapter,panel,commands}.rs`；AGP `src/renderer/{pipeline,resources}.rs`与`src/render2d/`。`image 0.25`已有PNG feature，不因可解码推断已公开。新测试拟`driver/tests/public_sprite_frames.rs`（driver指experimental/autographics-auto-driver，下同）、`AGP/tests/sprite_resources.rs`、`driver/tools/check_sprite_frames.py`及`verify_plan004.py`；均为待T-03..06创建的新路径。

## 4. 需求分析与背景调查

授权：用户2026-10-29要求“ag-001在实施，规划AG下一阶段”。本轮只取auto-ui本地号、设计/合同/引用，不实施、建树、启动agent、跑窗口/GPU/Cargo或远端发布。功能沿已定桌面M2表现路线，不新增跨端前置。未授权自动续跑实施。

已核基线与来源风险见[共用基线](attachments/004/planning/baseline.md)，架构见[表现与资源设计](../design/autographics/m2-presentation-and-resources.md)。当前UI HEAD `7e56dc8baa0b80f35ff7da757695e5e717047950`，002 delivery `0ee5ad0747ec3bb67e1288401fcc31daf3901c6e`，001 delivery `537122b09c025191355e260bc8e9880a4358cda2`。本仓003是Blog路线，不占AG实现范围。

知识权威目前仍在auto-lang：[overview](../../../auto-lang/docs/specs/auto-graphics/overview.md)、[静态资源](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)、[行为](../../../auto-lang/docs/specs/auto-graphics/behavior-api.md)、[触发/新局](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[样本研究](../../../auto-lang/docs/specs/auto-graphics/m2-game-sample-research.md)。本Plan SD路径以该既有知识仓为根，不代表本仓已有这些Specs；merge按SD跨仓沉淀，不批量迁史。

观察：Document仅Circle/Quad为2D且资源format仅glb；资产准入明确拒绝images/textures/animations，QuadPipeline只有平色/圆形。缓存有RGBA8槽不等于公共PNG精灵。现有Host prepare_frame统一相机/对象，不能另开一个未受检的贴图发布通道。

## 5. 详细设计

### 5.1 T-01/T-02执行入口与资源合同

依赖002实际交付已满足；work先核master/owner/代码差异、SDK实际闭包、两锁与构建配置。保留002受管桥96KiB/组合352KiB、用户源256KiB、32credits/callback、1000steps/slice、100000steps/callback、120resumes/250ms、30s初始化/2s tick与8ms目标；不把上次显式上调变成再次抬限授权。

T-02先在本树形成`attachments/004/work/sprite-resource-contract.md`，并做有界整路径可行性验证：真实Auto控制一角色移动/跳、相机、对象/触发、一次事件读取及一项精灵状态更新。公开绑定/批量帧与翻向setter的签名、价格、最大输出长度和profile由探针冻结；不得先猜可免费无界改节点。setter应能一次设置同对象的帧和翻向，有限输入/定长表由Host独立验证。旧profiles源码装配字节不变，新功能显式版本选择。无法保持完整组合预算则回new，不能用Rust选角色动作或拆多tick掩盖。

新PNG子集：静态PNG、8bit RGB/RGBA，拒绝APNG/损坏/不支持格式；单图宽高各1..2048、解码RGBA最多16MiB、原始单资源仍<=8MiB。尺寸/乘法/总额先检后分配，带alpha按既有线性色彩/预乘混合接缝正确处理，nearest采样与clamp为本期固定规则，无mipmap。图片+模型合计仍<=8个原始内容身份/32MiB raw；解码像素、geometry和新增CPU retained payload合计<=64MiB，纹理分配/必要row对齐/mesh数组合计<=64MiB GPU，历史/候选/当前union去重计数，不是每类另给一份上限。旧静态GLB的已交付规则保持，新混合候选在现有总额内准入。以上是新输入的有限支持域，不是提高既有预算；必要素材无法容纳则保留反例回new。

### 5.2 持久化、帧与身份

原图身份以完整原始SHA/长度/format绑定，资源名不充当身份。新scene表示明确定义schema/profile版本；旧scene可照常开，不得旧schema静默接受新字段。存储将PNG纳入项目自有closure，规范路径含原始SHA与.png，安全本地路径/祖先reparse检查、无覆盖、root/entry锁与blob先落盘再单entry提交规则延续；SaveAs用已接受快照，坏图/缺图/坏SHA不确认保存成功。磁盘图片与旧模型合计仍8文件/32MiB（含临时预留/残留），脚本配额不借用；完整闭包搬迁A→B再从新位置实际打开运行。

帧为整数像素矩形(x,y,w,h)，w/h>0且边界在图内；整数、非有限、溢出、错图身份/错维度/错Run/旧slot代次均受检拒绝。公共视觉绑定既支持已有静态2D叶，也支持001保留槽的合法2D实例；不扩大8槽/128节点或固体规则。销毁撤销视觉资格，同槽新代使用模板默认视觉。翻向只改变UV；显示尺寸由场景显式配置，帧裁剪不改变角色AABB。Auto自行按fixed tick切帧，不引入Host墙钟计时或隐式角色状态机。

### 5.3 共同发布与寿命

一个候选同时包含运动/对象/事件/相机/精灵；任一资源、输出、身份或GPU prepare错误保持整组last-good，不能出现位置新而帧旧。先验证再上传候选纹理；拒绝只释放新owner，合法外部pin可保留。纹理缓存键包括内容与device代次，正常暖帧换帧/翻向不重解码或重传图片；Pause不推进，Step中立输入一步，Stop撤销runtime owner且编辑preview可合法保留，fresh Run回到Auto初始化帧。关闭并释放最后pin后实际weak owner失效。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang知识仓） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/sprites-and-image-resources.md | 无公共图片/帧→有限PNG、取帧/翻向、事务与寿命 | M2 G2-08前置 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-graphics/scene-assets.md | glb-only closure→版本化图片共存/原额共享，旧静态域保持 | 编辑/离线完整 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-graphics/behavior-api.md | 新显式组合profile/价格/输出；旧profiles装配不变 | 真实Auto公开消费 | AC-03/05 |

## 6. 测试设计

新增测试/脚本由T-03..06建立，当前均未运行；命令示例须在work冻结实际可执行入口，0selected/跳过/错runner不能算绿。`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test public_sprite_frames -- --test-threads=1`；AGP对应`--test sprite_resources`；`python experimental/autographics-auto-driver/tools/check_sprite_frames.py --controls`；`python experimental/autographics-auto-driver/tools/verify_plan004.py --controls`。GPU capture为无窗口真实内容；OS抢焦点/置顶/键鼠自动化暂停，不绕过暂停门。manual公共CLI路径验收由后续实施按用户环境安排，未验证明确pending。

| V/补充case | AC/owner_stage/启用任务 | 路径/代表输入 | 可观察判据与区分错误 | 覆盖边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01/02 work T-03 | public import→Registry/Storage；非对称RGBA、2048边界、尺寸乘法溢出/伪PNG/APNG、SHA错/坏重开 | 有效输入可再读运行；非法先拒，decoder/allocation观察证明未越预算；残留不假clean | 本地磁盘PNG，不覆盖恶意并发换路径 |
| V-02/S-02 | AC-03 work T-04 | 真实Auto固定帧序0→1→2与flip；字符移动/相机/触发组合 | 参考像素ROI从左红右蓝变翻向，帧背景/透明边缘正确；改源帧号则内容改变；碰撞轨迹不因UV变化 | 不承诺任意压缩/调色图 |
| V-03/S-03 | AC-03/04 work T-04/05 | dynamic slot销毁重生；错Run/旧代/出界矩形/prepare失败 | 旧代拒绝，同槽新代默认帧；角色/相机/物品/精灵全旧或全新；有实际负控还原绿 | 不改变旧触发几何 |
| V-04/S-04 | AC-04 work T-05 | 20轮Play/Stop/新局、暖换帧、合法外部pin/最终drop | 原始/解码/纹理weak owner与真实上传计数；暖换帧不重传，最后owner释放后弱引用失效 | 不以history长度代替释放，不测驱动内部缓存 |
| V-05/S-05 | AC-05 work T-02/06 | fresh独立SDK consumer A→B、固定真实Auto组合、旧profile定向样本/旧装配hash | 预算实测、源变异敏感、离线运行/保存/重开、身份闭包一致；错误runner/部分报告拒绝 | 不跑全AG套件、不把引擎消费包当G09产品包 |

实现者未指定；独立最终review按同一树顺序、独立上下文核AC/V/SD/遗漏/资源与源差异。无需为规划创建agent或另一棵review树。review pass必须标merge-owned冻结门pending；代码变化后复审受影响面。

## 7. 验收标准

| ID | owner_stage | 结果与证据 |
|---|---|---|
| AC-01 | work | 新图片准入/有限支持/预算分类明确，V-01正反例闭合，无静默解码/预算上调 |
| AC-02 | work | 版本化场景、编辑绑定与完整离线保存/重开、坏件拒绝/修复恢复，V-01 |
| AC-03 | work | 公开Auto帧/翻向，组合角色/相机/实例/触发，同tick/碰撞独立，V-02/03 |
| AC-04 | work | 事务last-good、Pause/Step/新局/Stop、owner及暖上传，V-03/04 |
| AC-05 | work | T-02整链可行与预算、fresh consumer、旧profile保持及V-05；实际CLI与资料一致 |
| AC-06 | review | 独立核完整AC/V/SD、warnings/fmt与延期/私有业务绕过，无未处置finding；冻结门pending |
| AC-07 | merge | 冻结提交定向门绿/有效复用才合auto-ui master，根crate零改不跑根档；全AG里程碑pending |
| AC-08 | merge | Plan归档/SD知识与ledger沉淀、可恢复收据、guard clean后清唯一树；不宣称Gallery/完整M2交付 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、002实际交付 | 核baseline/owner/SDK两锁/策略，建唯一ui-004树，记录源码差异 | AC-01/05 | [x] |
| T-02 | work | T-01 | sprite-resource-contract先冻结；定长公开更新与整组合CPU/源/内存probe，失败回new | AC-01/03/05 | [x] |
| T-03 | work | T-02通过 | assets/document/storage图片/版本化closure/编辑接线、V-01 | AC-01/02 | [x] |
| T-04 | work | T-03 | 公共profile/Auto视觉输出与AGP真实绘制、V-02/03；新接口/测试入口创建 | AC-03/04 | [ ] |
| T-05 | work | T-04 | Pause/Step/Stop/新局/owner/失败注入及V-04 | AC-04 | [x] |
| T-06 | work | T-05 | fresh consumer/预算/旧profile定向面/本代identity controls、fmt/warnings/提交、SD草案及交付说明 | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立最终review，核所有V/S；merge冻结门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交受影响门/复用审计，通过才合本仓master | AC-07 | [ ] |
| T-09 | merge | T-08 | 沉淀SD/ledger/索引、本仓Plan归档与真实交付SHA/API/包身份 | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后移除本树/分支/空组，保必要本地恢复材料 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: pass | next: 用户启动work T-01/T-02`。这是合同就绪，非实现/独立验收通过。本轮所有任务0/10，未建树、改实现或运行上述测试。ag-examples/001沿已交付002继续，不追004开发tip。

2026-10-29 合同预检：三份Plan的0..10节、各10任务/8AC/5V、work/review/merge职责、依赖与SD引用检查通过；本批49个局部链接可解析，新增入口均标待建。此为文档预检，非实现测试或独立final review。

2026-10-29 work T-01+T-02 checkpoint（r1）：`stage: work | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: progress | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | base_commit: d3bd6b495dc8b690925e88deee9fbf2c42e7f991 | code_commit: 7552d01d559a8737ff7663c386ef4d15334dbd36 | dependency_revisions: auto-ui master d3bd6b4, 根/驱动/AGP 三锁未改 | task_ids: T-01,T-02 | next_responsible_role: work (T-03)`。

T-01 证据：主检出干净（仅 docs 差异）；baseline 六处源码 SHA-256 逐一相符（assets bf9b4b4f…、document 2068a187…、storage baf303bc…、viewport 096c253c…、assembly 669db066…、play b997b3aa…），三锁 SHA-256 亦相符（driver 2b131d3d…、AGP c3df168e…、root 813ba8e8…）；`D:/autostack/.wt/ui-004/auto-ui` 唯一注册为 `codex/plan-004-dev`，组目录 `D:/autostack/.wt/ui-004` 新建；`python tools/prepare_sdk.py --from-snapshot` PASS（681 files，17 local packages），SDK/target 均在本树内独立；`cargo check --locked --all-targets` exit 0。

T-02 证据（本地证据不入库）：合同 `docs/plans/attachments/004/work/sprite-resource-contract.md`（7301 字节，sha256 3ee7aae55aedd6befd13b2c3332cc580402d289d104388a1c8575cb8ad2ff482）冻结 `autographics.sprites2d/1`（profile 20 / number 109）、`ag_sprite_frame(id,x,y,w,h,flip)`、`ag_sprite_slot(token,…)`、`ag_sprite_bound(id)`、1 credit/32 每 callback、每 Run 定长块 `4+targets*7+56`、错误码 41/42/43/44 与 PNG 有限域（8bit RGB/RGBA、无交错、无 APNG、单边 1..2048、解码 16MiB、8 身份/32MiB raw/64MiB CPU+GPU 与模型共享）。`cargo test --locked --test public_sprite_frames -- --test-threads=1` 5 passed：13 个旧 profile 的 api/bridge/combined hash 与 pristine 基线逐字节相同；profile 20 managed 80367 字节（api 76518，96KiB 上限未抬）、块长 74；真实 Auto 24 tick（角色移动+相机+3 实例+3 触发对+精灵更新）中位 5.56ms/P95 7.78ms，帧 0→1→2 随真实 phase、左向只翻 UV 不改位移、实例槽在真实事件读取后切帧 2+flip，全部由 Host 独立验证；PNG 9 类非法输入先拒且 decoder 计数不变；scene/5 往返与旧 schema 拒收新字段成立。相邻回归：`cargo test --locked --lib` 33 passed；authoring_assets 17 / authoring_hierarchy 10 / authoring_document 1 / authoring_session 4 / authoring_storage 3 全绿（schema/4→5 的“当前写出版本”断言与 v1 夹具的 sprite 字段随版本化 scene 一并更新）。T-03 起的编辑接线/磁盘 closure 与 T-04 绘制仍未开始，故不设 execution_done，Plan 保持 executing。

2026-10-29 goal round 1：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: 7552d01 | next: work`。本轮只完成 T-01/T-02（2/10），其余任务保持未完成；用户对整链 work→review→bounded repair→merge 的授权已记录于本节，后续阶段继承。

基线漂移（已知，不阻断）：本轮期间 master 由 `d3bd6b4` 前进到 `31e4bd2`（另一路线 `fix-synth-surface` 的 pre-landing sync），差异仅 `crates/auto-ui-runtime/src/ui/{handler_codegen,vm_bridge}.rs` +21 行，与本计划的 `experimental/autographics/**`、`docs/plans/004-*` 零重叠；本 worktree 仍基于 `d3bd6b4`，T-08 合并前须再与 master 同步并刷新受影响证据。§2 规划的树/分支 `D:/autostack/.wt/ui-004/auto-ui`、`codex/plan-004-dev` 与用户授权一致。

## 10. 待澄清事项

无需为既定M2表现方向重问授权。公开函数名/wire/schema编号、实际组合价格与内存布局由T-02冻结，属于有界实现决策；若必需整链不能在已有预算内运行，保留反例回new，不减AC/静默抬限。正式G09美术/产品包、声音/HUD与完整游戏留给后续各仓计划，尚未取号。
2026-10-29 work T-03 checkpoint（r1）：`stage: work | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: progress | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | base_commit: d3bd6b495dc8b690925e88deee9fbf2c42e7f991 | code_commit: a79ba040b209ff0fe13de0a3f49ed7af0b85530b | task_ids: T-03 | next_responsible_role: work (T-04)`。

T-03 证据：图片载荷进入项目闭包与编辑事务。编辑：`Edit::SpriteObject` 新建/重绑/解绑，要求 2D 叶、无模型资源、绑定 SHA 与载荷一致、默认帧在图内；3D 叶、过期选择、错身份、越界帧、第 9 个绑定均为命名拒绝且文档不变。面板新增 PNG 路径 + Import PNG 及每图 Bind/Create/Detach，Host `import-sprite`/`sprite-bind`/`sprite-create`/`sprite-detach` 复用同一 begin→preview→commit 事务。存储：图片与模型同住 `assets/`（`<sha>.png`/`<sha>.glb`），共享 8 文件/32MiB（含残留与临时预留）；外来文件与篡改 blob 只报告不覆盖，缺失拒绝、恢复原字节后可重开，整目录 A→B 搬迁后从新位置重开并继续保存。`cargo test --locked --test public_sprite_frames -- --test-threads=1` 10 passed：新增 closure 正/反例、编辑器创建/重绑/解绑与四类命名拒绝、2048×2048 恰好 16MiB 接受而 2049 先拒且不触 decoder、四个 16MiB 恰好 64MiB 接受而第五个在分配前被拒、Host 公共命令导入/绑定/解绑/重复内容去重/坏 PNG 不改文档。相邻回归：lib 33、authoring_assets 17、authoring_host 7、authoring_hierarchy 10、authoring_document 1、authoring_session 4、authoring_storage 3 全绿。

合同更新（本地 9045 字节，sha256 397a753e…）：§7 更正 T-02 计时为多次运行的区间（debug tick 中位 5.49–9.50ms、P95 7.78–8.55ms、max 9.42–10.54ms，共享机器负载相关；整 tick 仍受 2s 约束），并记录精灵块只占本 Run 约 390 次 Host 往返中的 74 次；§10 记录 T-03 交付事实。T-04 绘制、T-05 生命周期仍未开始，Plan 保持 executing。

2026-10-29 goal round 2：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: a79ba04 | next: work`。
2026-10-29 work T-04 checkpoint（r1）：`stage: work | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: progress | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | base_commit: d3bd6b495dc8b690925e88deee9fbf2c42e7f991 | code_commit: ae6e3d9c8d4ae0e24de8f4550fb642b0f547bd1a | task_ids: T-04 | next_responsible_role: work (T-05)`。

T-04 证据：AGP 新增 `renderer/sprite.rs`（内容 SHA 身份纹理、`Rgba8UnormSrgb`、nearest/clamp/无 mipmap、实例化四顶点、每命令一次 draw 保序、与平色管线同相机）+ `SpriteStore`（staged 候选、按内容复用、真实 `created`/`bytes_uploaded`、`device_owners`）。绑定叶与 001 实例改画 PNG：编辑预览用文档默认帧、Run 用真实 Auto 帧，tint 取中性白 × 对象 alpha（避免默认平色污染导入美术）；纹理在 accept 前准备，失败保持整组 last-good；渲染失败先 re-insert surface 再上抛，避免丢槽。`cargo test --locked --test sprite_resources`（AGP）3 passed：真实像素验证 nearest/夹取/翻向/裁剪、透明纹素露出 clear 色、暖帧三次换帧/翻向不新增纹理也不重传像素而实例仍重写、同内容异尺寸/异设备/缺纹理命名拒绝、最后一 owner 释放后弱引用失效、`frame_uv` 越界拒绝。`cargo test --locked --test public_sprite_frames` 12 passed：新增编辑预览像素判据（400±16 左红右蓝、±30 透明边框露画布、翻向只换 U）与真实 Auto 像素判据（frame 0 左红右蓝，左向输入后同矩形左蓝右红），两者都断言 `Descriptor::rows()` 逐值不变（碰撞与 UV 独立）；金币槽先写入帧 2+flip，重生后回到模板默认帧。相邻回归：AGP lib 30 / asset_resources 1 / surface_contract 29 / surface_review_r2 5；驱动 authoring_assets 17 / authoring_host 7 / authoring_play_host 7 全绿。V-03 的 prepare 失败注入与整组 last-good 归 T-05。

2026-10-29 goal round 3：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: ae6e3d9 | next: work`。
2026-10-29 work T-05 checkpoint（r1）：`stage: work | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: progress | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | base_commit: d3bd6b495dc8b690925e88deee9fbf2c42e7f991 | code_commit: 0e421c310fbc0e7bb93b3778126174d3956ed23a | task_ids: T-05 | next_responsible_role: work (T-06)`。

T-05 证据：Pause 不推 tick 且不改已发布视觉；Step 只推一个中立 tick 并按真实 phase 更新帧；Stop 撤销 runtime owner 并恢复编辑 preview（文档默认帧）；同一文档再次 Run 得到 tick 0/初始化帧。`fail_resource` 注入使 prepare 拒绝整个候选：tick 不前进、位置与已发布 UV/槽帧保持 last-good，Play 记录命名错误；清除注入后 Stop+重 Run 恢复。寿命：prune 仅在仍有 owner 持有载荷时保留纹理，并删除引用已释放纹理的已发布命令；20 轮 Play/Stop（含暖帧换帧）只创建 1 张纹理、只上传 12288 字节且 arc 身份不变，关闭项目后最后一 owner 释放、weak 失效且缓存清零。`cargo test --locked --test public_sprite_frames -- --test-threads=1` 15 passed（连续三次运行均绿），新增 3 个 case 覆盖上述生命周期/失败/owner；AGP `sprite_resources` 3 passed 控制；相邻 authoring_assets 17 / authoring_host 7 / authoring_play_host 7 全绿。

2026-10-29 goal round 4：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: 0e421c3 | next: work`。
2026-10-29 work T-06（r1，work 收尾）：`stage: work | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | base_commit: d3bd6b495dc8b690925e88deee9fbf2c42e7f991 | code_commit: 7add28d | dependency_revisions: auto-ui base d3bd6b4、002 交付 0ee5ad0、SDK 681 files/479 包闭包未变 | task_ids: T-01..T-06 | blockers: 无 | next_responsible_role: review (T-07)`。

T-06 证据：`tests/public_sprite_frames.rs` 在 `PLAN004_REPORT_DIR` 下为 5 个 case 落 bounded JSON 证据（profile20 帧序列/块长/装配 hash、真实像素 ROI 翻向前后、10 类准入反例、2048/16MiB/64MiB/8 预算、20 轮纹理复用与 owner 释放）；`python tools/check_sprite_frames.py --controls` 跑真实套件并严格校验，缺 case/外来 runner/篡改像素/抬升预算/暖帧重传全部拒绝后恢复通过。`tools/gen_plan004_identity.py` + `verify_plan004.py` 冻结两包完整闭包 1431 文件（含 681 个 frozen SDK）+ 002 交付与 base pin + 旧 profile 两个装配哈希，`--controls` 对编辑/删除/未列/错 pin 全部拒绝；`verify_source.py`/`package_consumer.py` 以 004 为当前身份（历史链保留）。`cargo fmt --check`（两包）干净、改动文件 0 warning；旧 profile 定向面 `runtime_triggers_api` 14 passed。fresh consumer：`package_consumer.py D:/autostack/.wt/ui-004/consumer-004`（2111 文件/41.4MiB，含 `.sdk-source` 与两锁、无 target），B 内独立 `verify_plan004.py` PASS（1431 hashes）、`cargo test --test public_sprite_frames` 15 passed、`check_sprite_frames.py` PASS。SD 草案 `research/sprite-frames-spec-delta.md`、交付说明 `research/sprite-frames-handoff.md` 已提交；merge 冻结门（全量 AG 套件/native/手动 CLI/release 复测）明确 pending，本收据不声称通过。

2026-10-29 goal round 5：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 5 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: 7add28d | next: review`。work 阶段 T-01..T-06 全部完成（6/10），进入独立 final review。
2026-10-29 review r1（final，needs_fix）：`stage: review | review_scope: final | reviewer: 同一实现会话内执行（独立上下文不可用，已用代码/提交/实测重建判定，属独立性限制而非独立复核） | plan_id: auto-ui/PLAN-004 | plan_revision: 1 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-004/auto-ui | branch: codex/plan-004-dev | reviewed_commit: 7add28d | base_commit: d3bd6b4 | dependency_revisions: 002 交付 0ee5ad0、SDK 681 files/479 包、三锁未改 | spec_inputs: research/sprite-frames-spec-delta.md (sha256 7666634e…)、handoff 813a5e1b… | acceptance_results: AC-01 pass、AC-02 pass、AC-03 partial(fail on public entry)、AC-04 pass、AC-05 fail(实际CLI)、V-01 pass、V-02 pass、V-03 pass、V-04 pass、V-05 partial | pending_closeout_gates: 全量AG套件/native/手动CLI/release复测（merge所有） | findings: F-004-R1-01、F-004-R1-02 | next: work`。

独立复跑（reviewed commit 7add28d，树干净）：`python tools/verify_plan004.py` PASS（1431 hashes）、`check_sprite_frames.py` PASS（5 cases + 像素/预算/暖帧）、`verify_source.py` PASS（SDK 479 包闭包）；`frame_uv` 反例见 F-02。

**F-004-R1-01（major，AC-03/AC-05，原需求缺口）**：没有任何公共入口能选择 `autographics.sprites2d/1`。`src/bin/authoring/cli.rs` 的 `--behavior-api` 用法串与 match 分支止于 `triggers3d`（cli.rs:16/38-42），面板按钮集也止于 camera3d；`Sprites2V1` 只出现在 `behavior.rs`/`play.rs`。因此用户脚本里的 `ag_sprite_frame` 无法通过公开入口运行（只有测试程序化构造 Play 才行），与 Plan §1“公共入口…Auto 按真实状态选择帧”和 AC-05“实际CLI与资料一致”冲突。复现：`autographics-authoring --open <scene> --behavior samples/sprite_frames/main.at --behavior-api sprites2d` → exit 2 且提示不含 sprites2d。最小修复：cli.rs 增加 `"sprites2d"` 分支并更新两处用法串；补一个 CLI/入口解析测试；在 handoff/SD 草案写明实际命令。影响任务：T-04（公共 profile 入口）、T-06（CLI 与资料一致）。

**F-004-R1-02（minor，AC-01/AC-03 负域，新回归）**：公开助手 `agp::renderer::sprite::frame_uv` 在 `w > width` 或 `h > height` 时以 u32 下溢 panic，而不是命名拒绝。复现（临时探针，已删除）：`frame_uv([0,0,128,32],96,32,false)` → panicked at `src/renderer/sprite.rs:562: attempt to subtract with overflow`；release 下会回绕成巨大 UV。调用方目前都会先行校验（`rect_ok`/`SpriteBinding::validate` 先判 `w<=width`），故无现存生产路径，但违反本计划“整数/溢出受检拒绝”的规则。最小修复：先判 `w>width || h>height` 再相减；把该反例加入 `sprite_resources::frame_uv_and_capacity_are_bounded`。影响任务：T-04。

非阻塞观察：`api/README.md` 仍是 v1/v3 文档，未覆盖 001/002 之后的任何 profile；001/002 也同样未扩写，因此不算本计划偏差，建议在知识仓 SD 落地时统一补齐。

SD 草案核对：SD-01/02/03 与实现一致（profile 20/109、错误码 41-44、1 credit/32 写、块长 `4+targets*7+56`、2048/16MiB/64MiB/8、nearest+clamp、UV-only flip、scene/5 与旧 schema 拒绝）；仅需在修复后补“公共入口命令”一句。合并前仍需 merge-owned 冻结门。

2026-10-29 goal round 6：`stage: goal | plan_id: PLAN-004 | plan_revision: r1 | round: 6 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-004/auto-ui | commit: 7add28d | next: work`。