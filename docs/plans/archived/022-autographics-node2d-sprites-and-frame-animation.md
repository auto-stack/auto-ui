---
plan_id: PLAN-022
status: archived
feature_name: AutoGraphics Node二维精灵资源与局部帧动画
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-020]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-022：G09两入口所需的精灵/帧动画作者能力

## 0. 变更摘要

在020的真实有状态Node组件上补PNG图像symbol、具名帧表、sprite呈现/帧与flip绑定、必要typed渲染实例写端口。G09的atlas与idle/run/jump/fall规则由组件局部Auto model/on执行，经原sprites2d/1进真实GPU，保原两PNG/许可/18项oracle、碰撞/相机与原预算。

本轮只规划drafting0/10，不实施/建树/生成素材/GPU/agent；硬等020实际交付，不依赖021功能。AG008固定消费019不升级，Gallery内容迁移另仓后继管理。[共同设计](../design/autographics/node-components-queries-and-sprites.md)冻结组件局部性与并行边界。

## 1. 目标

1. 普通use可共享图像/帧定义与Player/AnimatedSprite组件，资源按typed symbol/完整内容身份引用，作者不写图像slot/文件hash作为业务索引。
2. Player自己持有vy/grounded/facing/phase/pose及输入逻辑，自己的view/on决定表现；主alt只改原节奏，运动不变；不回退为根全局model或Rust动画器。
3. 原PNG准入、帧/flip/透明度/碰撞盒/同tick发布与对象代次/寿命完整受检，原18项观察在新链可重建。
4. 准备产物包含原图/帧表/许可/来源闭包与实际source-build身份，离线冷重生/公开运行及坏件精确恢复可独立消费。

非目标：3D纹理/骨骼/GLB或任意图像格式、通用动画timeline/插值/渲染时钟、Life.Frame、声音/HUD、动态solid/多角色、通用动态Node子树/key重建、HMR/ECS/CPC/ACE/AOT、新平台、预算提升/修改SDK语义或Gallery内容实现。

## 2. 架构方案

普通模块/020组件IR→typed Image/SpriteSheet/Frame与实例局部状态→结构化scene/资源闭包/Auto帧选择代码→旧sprites2d/1 assembly、受检图像/物体/角色/相机服务→Host→真实GPU。准确声明/绑定拼写和现有持久schema由T-02冻结，不为新作者入口造另一套PNG decoder或复制实例运行时。

拟唯一实施/顺序复审树D:/autostack/.wt/ui-022/auto-ui、codex/plan-022-dev。020实际delivery作稳定基面；022可和021在独立模块写面并行，共享load/translate/main/identity/资源注册接线owner短窗，无法隔离就顺序。只读依赖固定SHA/锁/SDK，不追019/020开发tip、不触AG008树，禁止链接，清理先guard。

## 3. 技术栈

已存在pilot load::ResourceKind仅Quad/Circle/Color、lower_basic2d/translate/shared AST；driver api/sprite2d_v1.at、authoring/{assets,runtime_sprites,runtime_objects,runtime_round,play,storage}.rs、sprites2d/profile20与PNG准入/资源owner和现有sprites命名测试。020组件IR与事件/绑定公共接口待其实际交付冻结。

拟新增typed图像/帧声明与sprite service/lowering模块、fixtures/sprites2d/（持久产品夹具及CC0来源）、tests/node_sprites_gates.rs、driver tests/node_sprites_flow.rs、tools/verify_plan022.py；精确文件由T-02冻结。原G09的源/PNG/配方/许可只读固定复制必要测试输入，不在内容仓改代码或重新绘制替代图像。

## 4. 需求分析与背景调查

用户2026-11-01要求规划二维查询/精灵和组件局部性问题；本Plan认领精灵服务与G09原验收分母，不含实施/构建/GPU/agent/放宽验收授权。020未交付，因此实施有硬前置；与AG008的内容开发并行不共享产物/target，021不是前置。

基线auto-ui10fdf79、019交付c93adb7；内容仓912bd70、G09固定004历史消费。权威：[图像/帧资源](../../../auto-lang/docs/specs/auto-graphics/sprites-and-image-resources.md)、[G09消费](../../../auto-lang/docs/specs/auto-graphics/gallery-sprites-and-frame-animation.md)、[019消费身份](../specs/auto-graphics/node-authoring-consumption.md)、[020合同](020-autographics-stateful-node-components.md)与原G09 checks/README/asset-manifest。

原PNG：atlas SHA95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1，96×32、258B raw/12288B decoded RGBA、3帧；character SHA8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df，160×32、735B raw/20480B decoded RGBA、5帧。两者CC0-1.0，原完整许可/源配方与hash保留；不以重生成后不同bytes替代固定PNG。

raw SHA：atlas/main.at 2b89445204b88dd115cbf019564e72609223027ad05343775e7be8686891d9b5；character/main.at c051989bfaf353ec870a2e7c92d173103988a2741a60c3fbdb7179c5c26adf06；checks/g09/gallery_runtime.rs 9d7ce60243e3b373f689c458a1f08bd0c5229b5ba55aaf76f908a2e89a3bd7a8；原acceptance fd1bd9e9d38c2b4f3a9d9d27f5ac6bbb1fbfb74369e62175091dbe168c7e5315。

G09已有9+9=18项具名检查；其中两项slot重生/旧代的原测试通过注入真实Auto对象服务调用观测，不是原main业务已有动态Node树。新入口必须具备对应typed渲染实例端口/等价新源夹具，不能删掉这两项或重写为图片存在检查。

## 5. 详细设计

### 5.1 硬门和有界整链

T-01核020实际final pass/冻结门/master/Spec/清理收据和共用组件IR；重绑SDK7b987ca8/681受检副本、两图依赖锁/来源；核018/021/008实际owner。旧SDK历史对象不在时用冻结内容锁副本，不升级。

T-02最多两接法，先证明原atlas main/alt→frame/flip/透明像素、Player局部走跳→五pose/同tickcamera、两个同类sprite状态隔离和对象slot重生/旧代服务。冻结Plan§9的typed声明/资源准备/旧schema/profile/CLI/费用、18项映射/实际像素ROI与容差/来源链；完整31/32分支先实测。不成立带反例needs_replan，不用静态截图、Rust动画或提高预算补洞。

### 5.2 typed资源、呈现与实例

- PNG通过具名Image/SpriteSheet资源与帧表声明，typed端口可导入或传参；帧名映射到像素rect，初始frame/flip/尺寸来自声明。可将sprite附到已有character/shape叶子，不额外生成一份碰撞主体；准确属性名T-02冻结，不能拿名为Player/body等识别业务。
- frame/flip由本Node Auto状态驱动、合并为旧一次写服务，变换/碰撞盒不因crop/flip变形。frame字段类型/越界/未知帧/错image/mixed identity在旧受检边界拒绝，不做取模/截断或透明图替代。
- 支持本例需要的静态leaf frame写与typed渲染instance frame写；有限spawn/despawn的template/slot/Run/generation沿既有对象服务，句柄不以intID进入作者业务绑定。销毁/重生恢复模板默认sprite、旧代write故障33及last-good观察必须闭合；不是新增通用动态Node拓扑/模块实例重建。
- 同定义两个sprite组件有独立phase/facing/handler/owner，可共享PNG/帧表；不为每sprite创建纹理副本、VM、独立费用池或独立Tick。一个物理Player上限不变，多个表现组件不是多个物理角色。

### 5.3 原图像/状态机保真

PNG沿旧准入：静态8-bit RGB/RGBA、method0/noninterlace、边1..2048、单图encoded≤8MiB/decodedRGBA≤16MiB；APNG、palette/gray/16bit、错尺寸/截断/尾随/溢出在decoder分配前拒绝。用既有资源准入服务，不在生成器里写另一decoder或信任作者自报宽高。

沿原nearest/clamp/sRGB纹理与linear-space alpha混合；rect整数x/y/w/h与边界全链受检，flip仅交换U端点。八个内容身份/32MiB raw/CPU和GPU各64MiB、8sprite绑定、8文件32MiB磁盘预算及history/candidate/临时预留/残留规则保持，坏/外来文件具名拒绝不盲删；资源跨Editor/Run/外部pin按真实最后owner寿命。

Atlas main沿原phase0..5取三帧，alt改原cadence；左右同时按时按原right后left赋值，left最终决定flip，不能从axis=0误判“都没按”。Character沿原顺序：Jump沿/左右原flag→用vy求dx/dy→move与真实grounded→以该步vy/grounded选pose与phase→sprite写→grounded清vy或gravity9×dt；jump5、speed2、原碰撞与camera不改。主alt只改run cadence（原主0/1、alt0..2），同输入运动必须相同。

走跳/phase/facing属于Player组件自己的model/on；view绑定可读取已选pose/frame状态，不强迫把整模型搬回根。声明绑定不能在handler结束后用已更新vy错误重算空中pose。帧节奏仍固定Life.Tick，不引入Life.Frame/宿主墙钟或Rust动画器。

### 5.4 原18项映射（逐字保留，不抵扣）

| 原组/ID | 原oracle名（每个分号子项一项） | 新链关闭条件 |
|---|---|---|
| A01..05 | atlas_main_frame_flip_left_right；atlas_transparent_underlay_layer_order；atlas_half_alpha_cpu_gpu_composite；atlas_alt_cadence_same_tick；atlas_flip_negative_control | 实际frame+flip+左右像素、透明边/underlay与linear alpha，主alt同tick和固定flip负控 |
| A06..09 | atlas_destroy_respawn_default_sprite；atlas_stale_handle_rejected；atlas_pause_step_stop_fresh_run；atlas_alpha_negative_control | 新源typed实例真重生/默认帧与代次拒绝，生命周期/保存重开、实际alpha坏件控制 |
| C01..05 | character_idle_run_left_right；character_jump_fall_land；character_pose_gpu_pixels；character_reused_capture_negative_control；character_collision_and_camera | Auto姿态/移动/grounded与独立PNG采样新鲜GPU相符，旧capture/错误碰撞相机负控可区分 |
| C06..09 | character_pause_step_stop_fresh_run；character_main_alt_same_motion；character_wrong_air_run_negative_control；character_budget_32_pass_33_refused | 中性Step/新Run复原，主alt运动同但cadence不同，错误空中run与原32/33负控 |

代码改变导致变量名/生成ID变更时，通过具名源映射更新观察位置，不修改原像素期望/数值规则来迎合新实现。新增组件隔离、资源/type与身份负控另列，不用重复数凑18；不以同一条自报success代替独立像素/状态/资源观察。

### 5.5 原额、资源闭包与消费

原每sprite写1、callback最多32写且共享32池，不额外加免费绑定写。候选atlas held2+move26+grounded1+frame1=30；character pressed1+held2+move26+grounded1+frame1=31。左右held各采一次供axis/facing/run判断，不能先axis再重复held读导致33后把读取免费。完整主alt/各pose/初始化/slot分支T-02实测；多组件不承诺任意全组合都能跑满。

VM/时限/源码与managed/finite f32按所选旧API保持，统计全Tick的Auto/绑定/资源工作；8ms目标另列。位置/帧/相机与asset identity沿旧同candidate受检发布，不出现新位置配旧帧；沿该实验profile last-good/fault，不推广成全世界业务事务要求。

输出原受支持schema的scene、可执行Auto、image/frame/模块closure、asset bytes/hash/dimensions/许可来源和实际工具/runner/source-build记录。冷A→B只持完整闭包和固定工具/依赖再生并真Run，PNG来源/许可/帧/锁/runner坏件拒绝→精确还原→重开后同入口图像与Auto正确；不依赖生产者target/源码/网络。旧020/019/017/004实际受影响路径保留，Gallery所有源/锁不动。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md | 原sprite底层API→局部Node作者资源/帧/flip/typed实例/原18项/消费与未支持面 | 为G09迁移提供真实版本，原资源/逻辑规则保持 | AC-01..06 |

## 6. 测试设计

拟T-03..06创建pilot --test node_sprites_gates、driver --test node_sprites_flow -- --test-threads=1、verify_plan022.py --controls；T-02冻结真实路径/非零完整case集合/超时，目前未执行。所有像素预期由固定PNG独立解码、UV/flip/linear alpha计算，与新Auto状态/相机/实际GPU三方核；不调用被测帧选择公式来产生expected。

| V/S | AC / owner_stage / T | 实际输入→路径/边界 | 独立oracle/负控关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 具名PNG/帧/组件参数、缺asset/错type/错误rect/坏PNG/header或许可 | 正确原图可用，坏件分配前具名拒绝/decoder计数不增，无silent fallback或路径逃逸 |
| V-02/S-02 | AC-03 work T-04/05 | atlas main/alt、左/右/同时、三帧/透明边/half-alpha | A01..05/A09原像素与frame/flip变化，错误flip/alpha控制红；重复capture不得代实际帧 |
| V-03/S-03 | AC-03 work T-04/05 | Player局部model与五pose、主alt、双sprite同名字段、真实碰撞/相机 | C01..05/C07/C08逐项，运动相同而cadence差、错误空中姿态/根代业务/共享phase负控可区分 |
| V-04/S-04 | AC-04 work T-05 | typed slot销毁/同槽新代/旧代、Pause/Step/Stop/fresh、20短轮owner/外部pin | A06..08/C06与旧fault33/last-good、默认帧和最后资源释放，显隐不冒充销毁，不错误要求GPU缓存归零 |
| V-05/S-05 | AC-01/05 work T-02/05 | 主alt每pose/初始化/多组件/slot分支与32/33、VM/资源守护 | C09及真实全Tick计费/CPU、原资源阈值/合法值保持；不每Node reset或免费重复输入/图形写 |
| V-06/S-06 | AC-06 work T-06 | 完整模块/PNG/许可/帧/source-build消费A→B、SaveAs重开/坏件修复、旧consumer | 再生身份+实际Auto/GPU/资源和恢复后真Run闭合，旧模式受影响检查通过；不写Gallery内容 |

work→同树独立final review→merge冻结门；用户未指定人员/模型，本轮不创建agent/chat。选新增/受影响定向完整集合及必要共享消费者，不跑全AG；GPU/重型门单实例，OS抢焦点自动化继续暂停。

临时材料唯一D:/autostack/plan-artifacts/plan-022/auto-ui/，开工登记owner/用途/在途状态；外部consumer/SDK/target/PNG/日志显式此处，禁止attachments含md。契约/SD/执行/移交摘要折Plan§9；merge核所有输出目录后删自有scope，具名问题持有须owner/理由/关闭条件登记，未知/他仓目录不盲删。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 020稳定组件/SDK身份、完整atlas/Player/typed slot整链及原额先证，无Rust动画器 |
| AC-02 | work | typed图像/帧/端口/闭包与严格准入、正确定位诊断，V-01 |
| AC-03 | work | 局部状态驱动frame/flip/五pose，原atlas/character图像与同tick运动相机保真，V-02/03 |
| AC-04 | work | 原18项中slot/旧代/生命周期/20轮owner与资源寿命、默认帧/故障保持，V-04 |
| AC-05 | work | 18项逐项可追溯、原32/33/VM/资源/数值包络与全Tick报告，V-05 |
| AC-06 | work | 公共固定消费/资源许可闭包/A→B/SaveAs/坏件恢复与旧路径兼容，V-06 |
| AC-07 | review | 独立逐18原项/AC/V/S/SD/遗漏/格式/warnings复审，绑定提交，无未处置finding |
| AC-08 | merge | 冻结定向门/有效复用后实际合master，准确Spec/来源/归档导航 |
| AC-09 | merge | guard/clean/祖先核后清树分支空组、全部材料输出对账与清理/具名持有登记、10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续启动、020实际交付 | 核Spec/SDK/依赖与owner，建唯一ui-022树，固定两PNG/许可/源/18项 | AC-01 | [x] 2026-11-01：020实际交付核验（master祖先16ddfa0、canonical spec docs/specs/auto-graphics/stateful-node-components.md在案、goal收据done-verified）；唯一树D:/autostack/.wt/ui-022/auto-ui建（codex/plan-022-dev@145d414，含020/021交付）；SDK受检副本重绑（prepare_sdk --from-snapshot PASS 681文件/17本地包，.sdk-source git-ignored）；两PNG/许可/来源清单只读固定复制入fixtures/sprites2d/{assets,licenses,manifests}并逐hash核（95a08525…258B/8fde96f6…735B/两CC0 LICENSE/两asset-manifest）；原18项oracle源（ag-examples checks/g09/gallery_runtime.rs@9d7ce602…）逐字在案；scratch D:/autostack/plan-artifacts/plan-022/auto-ui登记。基线探索完成：driver已全链消费sprite场景（play.rs _ag_spr_map配置、runtime_sprites admit_block、assets.rs PNG准入、document.rs SpriteBinding）——实施集中在pilot生成器 | AC-01 |
| T-02 | work | T-01 | 最多两接法完整atlas/Player/typed slot先证与费用，Plan§9冻结接口/CLI/原18映射/像素/包络；不足回规划 | AC-01..06 | [x] 2026-11-01：两接法完整先证（接法一=字符行/静态叶 ag_sprite_frame 逐tick写；接法二=实例模板+render2d.spawn/despawn/frame typed slot 服务）：pilot --profile sprites-2d 落地（load.rs image2d.png/frame2d.at 具名资源、translate render2d/input.held、lower_components sprites 模式=声明序场景+文档sprite字段+帧表if-chain present+render2d plan验证），10产品生成（fixtures/sprites2d + generated-sprites2d）。冻结面折§9：接口/CLI/原18映射/预算（atlas 30=held2+move26+grounded1+frame1；character 31=+pressed1；32恰过/33=fault10实测）。T-02先证经真实链：driver node_sprites_flow 18项+pair+V-06一次通过（43~465s单线程实测） |
| T-03 | work | T-02 go | typed PNG/帧表/资源闭包与准入，复用020组件，不另造decoder/VM，V-01 | AC-02 | [x] 2026-11-01：typed PNG/帧表/资源闭包与准入全落地。图像身份=完整原始字节SHA256+byte_length+IHDR尺寸（生成器只读IHDR，不写第二套decoder）；缺文件/坏签名载入层具名拒绝（image resource ... cannot read/is not a readable PNG）；完整PNG准入沿driver既有资源服务。闭包=closure/<module>.at+assets/<sha>.png+identity.json（generator/api/逐模块sha/assets[sha,file,declared]/provenance）。V-01=pilot node_sprites_gates 10例全绿（neg-frame-rect/neg-frame-lit/neg-image-type/neg-flip-lit/neg-unknown-spawn/slot-9具名拒绝+missing-file/corrupt-png+确定性再生+固定PNG身份）；verify_plan022 --controls 6负控复放PASS |
| T-04 | work | T-03 | 局部Auto状态/绑定/frame/flip/必要typed实例与同tick GPU，V-02/03 | AC-03 | [x] 2026-11-01：局部Auto状态驱动frame/flip/五pose+同tick GPU全部实测绿。AtlasSprite/Player组件自带phase/facing/pose/vy/grounded model+Life.Tick；view image绑定经<prefix>_present(i)帧表if-chain每tick恰一次写（pose在handler内按序计算，present不重算）；字符行经ag_sprite_frame(行id)、实例模板经ag_sprite_slot(handle[i])。V-02/03=node_sprites_flow A01..A05+C01..C05全绿（五姿态GPU像素×独立PNG解码×容差3、透明边界露underlay、半alpha线性合成、左右像素对、同tick alt cadence、翻转负控） |
| T-05 | work | T-04 | 原18逐项/生命周期/slot旧代/owner20轮、预算/资源负控与真实旧消费者，V-04/05 | AC-04/05 | [x] 2026-11-01：原18项逐项+生命周期+slot旧代+owner20轮+预算负控+旧消费者全绿。V-04=A06/A07（render2d spawn/despawn/frame：代次1→2、默认帧恢复、旧代写fault 33、last-good保持accepted tick=2）+A08/C06（Pause不推进/Step恰一/Stop恢复文档默认/SaveAs重开fresh）+v04_twenty_round_owner_fresh_runs（20轮stop/fresh每轮tick0+空槽+文档默认sprite、场景sprite绑定逐轮字节稳定、共享PNG respawn后仍渲染）全绿。V-05=C09（budget-32恰过/33=fault10具名）+atlas 30/character 31计费与§5.5候选一致。旧消费者复跑：node_components_flow 9/9绿（driver dev-deps变更后受影响面重放）；pilot全套10+9+10+9+6+7测试绿；verify_plan016/017/019/020/021再生控制全PASS（旧profile字节冻结） |
| T-06 | work | T-05 | 完整资源/许可/source-build消费A→B/SaveAs/损坏恢复，SD/格式/移交折Plan，V-06 | AC-01..06 | [x] 2026-11-01 R2修复完成（F-R1-01..03关闭）：pilot/driver cargo fmt --check双clean且10个既有产品再生字节零diff（fmt纯格式）；flow收据补全（write_receipt移至v04/v06/runtime-oob测试末，receipt 22行=18原项+pair+v04+v06+runtime-oob，0失败行）；verify_plan022补注释变异控制（注释独改→world字节等值，×2产品）并纳入runtime-oob（38控制组exit 0）。原完成记录：2026-11-01：完整资源/许可/source-build消费A→B/SaveAs/损坏恢复+SD/格式/移交折Plan。V-06=verify_plan022 --controls PASS（10产品身份链world sha==场景绑定==identity、closure==夹具、资产sha==固定origin字节、闭包独占冷A→B字节等值×10、语义变异敏感性×2（注释变异被正确忽略）、6负控、CC0许可/来源清单/两PNG origin hash固定）+v06_damaged_copies_rejected_then_recovery_green（world/资产各一字节损坏→载入具名拒绝 behavior source closure identity/conflict: asset hash+length→精确还原→绿Run同入口图像正确）。SD-01候选落docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md（仅实测面）。fmt clean；pilot/driver新代码零新warning。scratch登记/在途状态见 D:/autostack/plan-artifacts/plan-022/auto-ui/README.md |
| T-07 | review | T-06 | 同树独立final逐18/AC/V/S/SD/遗漏，finding回work，pass标merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结源/资产/依赖/身份，定向门或有据复用后合master | AC-08 | [x] 2026-11-01：冻结定向门=ff8cd23受影响面全绿（评审者复跑+实现复跑：pilot 51、verify_plan022 38控制、node_sprites_flow 4、旧verifier 016/017/019/020/021、node_components_flow 9/9）；合master经rebase（023已先期交付32d7a81、master两度前进0b6e74d/818544f）：98d5edb→2f45baa、ff8cd23→c6aa63a，rebase解析=023 game-mode互通（pub(crate) IR、sprites=false线程化、game调用点arity更新），非等价range-diff故按冲突rebase规则在rebased态全量刷新受影响验证：pilot 54测试ok、七verifier全PASS（含023的10控制）、11产品字节零diff、GPU flow 4 passed 72.57s、node_components_flow 9/9；ff-only落地master=c6aa63a无merge commit | AC-08 |
| T-09 | merge | T-08 | 实际Spec/资源来源/归档导航与证据折Plan，不写外仓Spec/派生JSON | AC-08/09 | [x] 2026-11-01：canonical spec docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md @c6aa63a landed（工作树与落地字节一致，sha256 f997bae66305eef07d5eb902dcc5ba195350c2ffff38360fcd456dda9877f1b2）；本Plan归档docs/plans/archived/、status: archived、README注册表022行刷新；无外仓Spec/派生JSON写 | AC-08/09 |
| T-10 | merge | T-09 | fresh guard/clean/祖先核后清树分支空组，全部输出对账与自有材料清理/登记 | AC-09 | [x] 2026-11-01：三树fresh guard全clean（auto-ui/auto-lang/auto-web）→worktree remove×3（list 0项）→branch -d codex/plan-022-dev（was c6aa63a已landed）→空组D:/autostack/.wt/ui-022 rmdir；plan-artifacts/plan-022全删（delete-at-merge 2026-10-31，收据已折本文件）；registry .retention/policy.json刷新（auto-ui/PLAN-022 archived，无具名持有）；10任务10/10，AC-01..09全关，交付c6aa63a | AC-09 |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-022 / plan_revision: 1 / outcome: blocked（实施硬前置020尚未实际交付，不阻本轮规划） / next: 020实际交付并获启动后T-01/T-02。本轮仅合同/设计/导航/取号；两PNG/18项为原来源观察，未写新源码/素材/树/agent或运行构建/GPU，不宣称已迁移G09。

2026-11-01文档预检：0..10节、10任务/9AC/6组V-S/1SD、所有本地Markdown链接、取号唯一及drafting0/10通过。三计划依赖无环（019→020→021/022），AG008固定019不增前置；G05九类原断言已按工具AST枚举，G09原18项名字在022逐字保留。git文档diff检查通过；未创建实施树、源码/资产/探针、构建/GPU或agent，本记录不是实现/可行性go/独立终审。

2026-11-01 / stage: work / plan_id: auto-ui/PLAN-022 / plan_revision: r1 / outcome: pass（execution_done）/ worktree: D:/autostack/.wt/ui-022/auto-ui / branch: codex/plan-022-dev / base_commit: 145d414 / code_commit: 见worktree提交（本次移交提交）/ dependency_revisions: auto-lang b1a4871aa、auto-web ec9c4ba（ui-022组受检树，只读）、SDK 681文件受检副本（.sdk-source，prepare_sdk PASS）/ task_ids: T-01..T-06 / evidence: pilot node_sprites_gates 10/10、node_sprites_flow 18项+pair+v06+v04全绿（PLAN022_ARTIFACTS=D:/autostack/plan-artifacts/plan-022/auto-ui/flow/node-sprites-receipt.json）、verify_plan022 --controls PASS、verify_plan016/017/019/020/021全PASS、node_components_flow 9/9 / blockers: 无 / next_responsible_role: review（T-07 同树独立final）。

T-02 冻结（实测后冻结，全部经真实链先证）：
- CLI/生成器：`node2d-lower --profile sprites-2d --entry <dir>/main.at --out <dir>` → world.at + scene.agscene + identity.json（generator "autographics-node2d-pilot plan022 sprites-2d"、api "autographics.sprites2d/1"、逐模块sha closure、assets[sha,file,declared]、provenance）+ closure/ + assets/<sha>.png。identity generator 字符串区分档。
- 声明面：`image2d.png("<file>")`（相对工程根；身份=完整原始字节SHA256+byte_length+IHDR宽高）；`frame2d.at(resources.<img>, x, y, w, h)`（声明序=ordinal）；shape2d/character2d 携带 `image:`（resources直连或Image2D参数，多实例必须同图）+ `size:` + `frame:`/`flip:`（Lit/参数/.field）。静态绑定行落文档sprite字段；Field绑定行=模板+spawn+逐tick present帧写。
- typed渲染实例：`render2d.spawn("<根视图image叶名>", x, y)->token`（同元素多站点复用同槽=respawn）；`render2d.despawn(t)`；`render2d.frame(t, "<img>", ordinal, flip)` 语句形态=帧表if-chain（越界ordinal→退化矩形写→受信单元矩形域具名fault 42；用户源不可直呼_ag_fail——admission实测）。spawn槽预算=app_start模板槽+站点按元素去重≤8（第9降层具名拒绝）。
- 场景/层序：声明序=文档序=绘制序；文档sprite字段={asset:"sha256:…",frame:[x,y,w,h],flip}，默认frame/flip=声明初值经帧表解析；场景≤128行、8有状态实例包络不变。
- 原额（实测）：atlas tick=held2+move26+grounded1+frame1=30；character tick=pressed1+held2+move26+grounded1+frame1=31；32恰过、第33计费=具名fault 10（budget-32/33夹具实测）。
- 原18项映射（逐字名，全绿实测）：atlas→atlas-g09/atlas-alt/atlas-flip-control/atlas-slot四夹具+harness alpha重编码负控（A01..A09）；character→character-g09/alt/wrong-air/budget-32/33（C01..C09，数字沿原5.0/2.0/9.0，非020探针的8/4/20）；像素期望=固定PNG独立解码+linear-light合成（容差3），不调被测帧选择公式。字符行id按场景实例解析（atlas=2、character=4、slot=3），oracle从场景读出，非写死。
- 观察/像素映射变更记录：原main.at `if phase>1{x=32}` 落为pose ordinal模型字段（pose0/1/2↔[0,32,64]）；原`ag_char_bind(1)`裸id→视图实例名解析。未修改任何原像素期望/数值规则/删项。

2026-11-01 / stage: goal | plan_id: PLAN-022 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-022/auto-ui | commit: 98d5edb（codex/plan-022-dev；T-01..T-06全部落地，worktree clean） | next: review（T-07 同树独立final，review_scope: final）。硬门核验：020实际交付16ddfa0在master祖先（drafting→work入口合法）；主检出WIP仅他会话簿记（Cargo.lock/012附件/autoui/007设计稿），本Plan零代码WIP。证据锚：flow收据D:/autostack/plan-artifacts/plan-022/auto-ui/flow/node-sprites-receipt.json（18项+pair+v06+v04全绿）、verify_plan022 --controls PASS日志同目录。

2026-11-01 / stage: review / review_scope: final / reviewer_identity: 独立general-purpose子代理上下文（全新会话，只读；由实现会话协调分派——limitation：非独立人类/模型角色，判定重建自工件+独立复跑而非实现者摘要） / plan_id: auto-ui/PLAN-022 / plan_revision: r1 / outcome: needs_fix / worktree_path: D:/autostack/.wt/ui-022/auto-ui / branch: codex/plan-022-dev / reviewed_commit: 98d5edb8ad25ef2088b0e7f63330a507065b2555 / base_commit: 145d414 / dependency_revisions: auto-lang b1a4871aa、auto-web ec9c4ba、SDK 681文件受检副本 / spec_inputs: docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md（SD-01候选@98d5edb）+ 020 canonical spec + 原G09 oracle源9d7ce602… / acceptance_results: AC-01..06全pass（评审者独立复跑：pilot全套测试exit 0含gates 10/10、verify_plan022 exit 0且控制有效、GPU node_sprites_flow 3 passed 43.69s含18项全绿receipt 19/19、旧profile五verifier exit 0、node_components_flow 9/9、fixture PNG逐字节=origin、18 oracle名与像素期望逐字一致、预算delta恰一pad读）；AC-07 partial（F-R1-01格式失败）；AC-08/09 merge期 / pending_closeout_gates: merge冻结定向门 / findings: F-R1-01(P1,AC-07/T-06) pilot cargo fmt --check失败@translate.rs:410（T-06"fmt clean"记录不实；driver侧干净；纯机械）——修复：cargo fmt+重验；F-R1-02(P2,§9措辞) receipt实为19行（18+pair），v04/v06绿由测试输出为证、未入receipt（write_receipt先于其运行）——修措辞或移write_receipt至末测试；F-R1-03(P2,T-06/SD-01§6) "注释变异被正确忽略"无对应verifier用例——补注释变异断言或删该括注；F-R1-04(P2,T-02/SD-01§1) 运行时越界ordinal→退化矩形→fault 42已实现但无测试覆盖（仅字面量越界有）——补运行时用例或措辞降为"沿受信单元、字面量已测"；F-R1-05(P2,SD-01§4) render2d arity/每定义多图像元素/多动态元素/第9有状态实例的归因harness不确（gates+verifier仅8个具名负控；第9实例由020套件覆盖）——改归因措辞或补夹具 / evidence: 评审者复跑记录在其报告（87 tool calls；关键命令与exit code见本记录acceptance_results） / next: work（修复F-R1-01..05，回同树提交，受影响面重验后re-review）。

2026-11-01 / stage: goal | plan_id: PLAN-022 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-022/auto-ui | commit: 98d5edb（被审基线；未变） | next: work（修复F-R1-01..05后同树提交，受影响面重验，再re-review）。评审round小结：AC-01..06独立复跑全pass；5 findings（1×P1 fmt、4×P2措辞/用例）；无验收声称与证据矛盾。

2026-11-01 / stage: work（R1修复） / plan_id: auto-ui/PLAN-022 / plan_revision: r1 / outcome: pass（受影响面重验全绿）/ worktree: D:/autostack/.wt/ui-022/auto-ui / branch: codex/plan-022-dev / base_commit: 98d5edb（被审基线）/ code_commit: ff8cd23 / task_ids: T-06（重开项）+T-02/SD-01（措辞/用例）/ findings_closed: F-R1-01（pilot cargo fmt；--check双crate exit 0；10既有产品再生git零diff=输出字节不变）、F-R1-02（write_receipt移至v04/v06/runtime-oob测试末；receipt 22行=18+pair+v04+v06+runtime-oob、0失败）、F-R1-03（verify_plan022注释变异控制：注释独改→world字节等值×2产品；38控制组exit 0）、F-R1-04（runtime-oob夹具+产品+flow用例runtime_out_of_range_ordinal_faults_42_named：tick2 pose=9→退化矩形→error含42、accepted tick=1、last-good视觉保持）、F-R1-05（SD-01§4归因改写：gates 8具名负控/verifier复放6/render2d arity与多图像多动态=已实现未单列夹具/第9实例沿020套件/运行时42由runtime-oob覆盖） / 受影响面重验：pilot 51测试ok+fmt--check clean、verify_plan022 exit 0（38控制）、node_sprites_flow 4 passed单线程85.07s、verify_plan017/020/021 PASS / remaining_gap: 无 / next_responsible_role: review（targeted re-review @ff8cd23，评审者agent续跑中）。

2026-11-01 / stage: review（R1 targeted re-review）/ review_scope: final（repair re-review）/ reviewer_identity: 同独立general-purpose子代理上下文续跑（limitation同前轮） / plan_id: auto-ui/PLAN-022 / plan_revision: r1 / outcome: pass / worktree_path: D:/autostack/.wt/ui-022/auto-ui / branch: codex/plan-022-dev / reviewed_commit: ff8cd23 / base_commit: 98d5edb（R1被审基线，祖先核验）/ dependency_revisions: 不变（auto-lang b1a4871aa、auto-web ec9c4ba、SDK 681） / spec_inputs: SD-01候选@ff8cd23 / acceptance_results: 五finding全部closed（评审者逐项在新树复现：F-R1-01 fmt--check双0+10既有产品零diff+verify_plan022冷再生11产品字节等值；F-R1-02 flow 4 passed 46.84s、receipt 22行全true；F-R1-03 38控制组含comment_mutation_ignored×2；F-R1-04运行时42真实复现error code=42 node=1、tick=1、kept visual；F-R1-05 SD-01§4归因逐条属实）；受影响面抽核verify_plan016/017/020/021 PASS；repair diff未引入新finding / pending_closeout_gates: merge冻结定向门（T-08） / findings: 无 / evidence: 评审者re-review报告（17 tool calls；RE-VERDICT: pass） / next: merge（T-08..T-10）。

2026-11-01 / stage: goal | plan_id: PLAN-022 | plan_revision: r1 | round: 3 | from: review（re-review） | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-022/auto-ui | commit: ff8cd23（修复提交；评审者在 clutch 新树全复现） | next: merge（T-08 冻结定向门→合master、T-09 Spec/归档、T-10 清理）。re-review小结：5/5 closed、0新finding；工作计数 work↔review 1轮修复（≤3帽内）。

2026-11-01 / stage: merge / plan_id: auto-ui/PLAN-022:r1 / outcome: pass（landed+canonical_verified；archived/cleaned见下续）/ prepared: reviewed基线ff8cd23（reviewer RE-VERDICT pass绑定的交付提交）；SD-01候选=落地spec同字节 / rebase: master两度前进（0b6e74d=023交付32d7a81归档清理；818544f=024/025 docs-only规划）；rebase映射 98d5edb→2f45baa、ff8cd23→c6aa63a；rebase解析改写共享面（023 game-mode互通：IR字段pub(crate)、gen_user_source/gen_present_fn/lower_components_mode签名加参、game调用点传sprites=false、instance_ctx去重保sprites形）→range-diff非全等→按冲突rebase规则于rebased tip刷新受影响验证 / 冻结定向门（rebased tip实测）：pilot 10套件全ok（含023 game gates 3）、verify_plan016/017/019/020/021/022/023七verifier全PASS、11产品再生字节零diff（PRODUCT-BYTE-IDENTICAL diff -r对照）、GPU node_sprites_flow 4 passed 72.57s单线程（22行receipt 0失败）、node_components_flow 9/9 / landed: 主检出 git merge --ff-only codex/plan-022-dev → master tip=c6aa63a6bbf3363e0e53ef9538166638cd5dc06d（无merge commit；首次ff因818544f推进被拒→rebase后二次落地成功）/ canonical_verified: docs/specs/auto-graphics/node2d-sprites-and-frame-animation.md @c6aa63a sha256 f997bae66305eef07d5eb902dcc5ba195350c2ffff38360fcd456dda9877f1b2（工作树==落地）/ 交付后观察: 本仓为库/研究仓无驻留生产二进制交付面（017/019/020/021同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang），022不适用 / next: archived→cleaned（T-10）。

2026-11-01 / stage: merge（续）/ plan_id: auto-ui/PLAN-022:r1 / outcome: pass（archived+cleaned全核）/ archived: docs/plans/archived/022-autographics-node2d-sprites-and-frame-animation.md status:archived（本文件，git mv保留历史）/ cleaned: 三树fresh guard clean（见T-10行）→worktree remove×3→worktree list 0项→branch -d（was c6aa63a landed祖先）→组目录rmdir→plan-artifacts/plan-022删除→registry刷新（无具名未解决问题持有）/ 10任务10/10完成 / 完成核对: ①merge pass（re-review RE-VERDICT pass @ff8cd23内容=c6aa63a重base同content）②交付commit c6aa63a在master默认分支、ff无merge commit、线性史③canonical spec在案④归档+导航（README 022行archived）⑤树/分支/组/plan-artifacts清零⑥主检出遗留WIP均他会话簿记（Cargo.lock/attachments/012/autoui/007/024-025规划README行），本Plan零代码WIP / 报告: PLAN-022 r1 merge pass；delivery 2f45baa+c6aa63a；SD sha f997bae6…；archive docs/plans/archived/022-autographics-node2d-sprites-and-frame-animation.md；cleanup 0残留。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 020局部组件/事件/绑定时钟、实际source-build消费接口 | 已关闭2026-11-01：020实际交付16ddfa0在master祖先，T-01重绑完成（canonical spec+组件IR消费实测） |
| D-02 | typed资源/帧/绑定和instance端口、精确schema/CLI与完整成本 | 已关闭2026-11-01：§9 T-02冻结（image2d/frame2d/render2d拼写、sprites2d/1装配、atlas 30/character 31实测费用；typed slot沿对象服务，无免费写） |
| D-03 | 原18项的具名源/像素/slot观察映射 | 已关闭2026-11-01：§9 T-02映射（pose ordinal/实例名解析为具名定位变更，18项名与像素期望逐字未动） |
| D-04 | 021/018共享接线与独立review/GPU窗口 | T-01/T-06按实际owner/授权安排；不能隔离则顺序，不同树同步并发写 |

022不要求021或AG008完成；Gallery的G09迁移另仓另立后继，不预取编号。
