---
plan_id: PLAN-012
status: execution_done
feature_name: AutoGraphics公共音频资源与接受后一次播放
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-006]
supersedes_spec_components: [docs/specs/auto-graphics/scene-assets.md, docs/specs/auto-graphics/behavior-api.md]
new_spec_components: [docs/specs/auto-graphics/public-audio.md]
touched_goals: [GOAL-008]
---

# PLAN-012：公共音频资源与接受后一次播放

## 0. 变更摘要

在006实际交付上新增有限音频文件/cue与真实Auto请求、接受后一次播放、暂停/Stop/新Run和fresh消费。分别提供2D/3D显式组合profile，旧API/预算不变；必要的visual+audio有界bundle用于同callback整链。只做引擎与必要夹具，G12a另归ag-examples/005。

## 1. 目标

公共项目可以导入有限Ogg/Vorbis或PCM16 WAV、声明有限cue，通过Auto决定何时/哪段/音量播放；正常公开Run默认走真实本服务后端，设备不可用具名失败，不要求probe旗标才听到声音；解码/实际混音PCM和本服务native输出可观察。重复绘制/传输重放不多播，坏候选不发声，旧Run/Stop无残留队列。32credits下真实输入/运动/相机/事件/visual/音效的代表性完整组合成立；资源保持原额且离线闭包可保存/搬迁/重开。

非目标：音乐循环、流媒体、麦克风/系统录音、空间化3D混音、声音编辑器、公共HUD、G12目录/产品ZIP、完整游戏、生产UI/跨端或调SDK/VM预算。旧Breakout/Squash专用音效不作为本公共合同的通过证据。

## 2. 架构方案

音频raw/声明→先检预算/有界解码→不可变Registry/cue→Auto有限请求→Host验证/资源与voice容量prepare→共同accept→一次mixer提交。所有玩法决策仍在Auto。完整设计与边界见[公共声音/HUD设计](../design/autographics/m2-public-audio-and-hud.md)，本Plan §5..8为当前执行合同。

唯一实施/顺序复审树拟`D:/autostack/.wt/ui-012/auto-ui`，分支`codex/plan-012-dev`；等006实际delivery后核master创建。主检出仅簿记，SDK准备/target/报告自有；不写006在途树或Gallery，不创建链接，清理先Git Bash wt-guard clean。

## 3. 技术栈

已核driver `src/host.rs`的私有Audio/enable_audio和`squash_host.rs`专用Music、AGP `src/render2d/text.rs`；两包kira0.9（ogg）已存在。公共owner应落driver `authoring/{assets,document,storage,behavior,play,host}.rs`、`behavior/assembly.rs`与`host/{viewport,play_adapter}.rs`，新音频模块/可信unit和工具由T-02冻结准确新路径，不复制专用游戏判断。

拟新增`driver/tests/public_audio.rs`、`driver/tools/check_public_audio.py`、`verify_plan012.py`及两维Auto夹具（driver=experimental/autographics-auto-driver）。当前都不存在，命令/selector/wire/schema号不预造为已支持。

2026-10-30 T-02现状：`authoring/public_audio/{mod,codec,mixer,native,bank}.rs`、`tests/public_audio{,_codec,_host,_compat}.rs`及`examples/audio_decode.rs`已建立并验证；显式audio2d/audio3d、scene/7及固定16scalar wire见执行合同当前关闭节。两个最终validator工具、正式两维bound样例与fresh consumer仍属T-06待建；上段“当前不存在”为规划时事实，不是已交付API声明。

## 4. 需求分析与背景调查

2026-10-30 复审例外授权：用户对已具体化的只读复审问题明确回答“允许只读子代理独立复审”。这是§6“不新建agent/chat”的限定例外：仅同一唯一实施树/冻结commit的只读独立复审，不新建用户聊天、不复制代码、不修改实现；不改变功能、验收、阈值、仓库或merge前finalpass要求。根会话仍是共享Plan唯一写者。此前授权阻塞现已解除，可继续T07及原全链后续。

2026-10-30 执行授权：用户 `/goal 完成UI的计划012` 授予既有合同的work→独立review→有界repair→merge/规范/台账/归档/guard清理全链。006已实际交付c681663并归档cleaned，master祖先与独立R2/冻结34case/SDK锁及current identity已经实读核验；历史“仅规划/006未交付”描述保留为当时背景。唯一ui-012树已由master f7c83c6创建，分支codex/plan-012-dev；主检出其他所有者文档WIP保留，未发现代码WIP。当前入口与T-02未关闭项见[执行合同](attachments/012/work/public-audio-contract.md)。

用户2026-10-30要求“规划G10b和公共声音/HUD后继计划，按引擎与Gallery分仓，暂不实施”。本轮只新Plan/设计/附件/引用/取号，不建树、改源、生成资源、编译、输出声音、GPU/OS/agent或远端发布；未授权自动实施。005已交付351919c；006仍executing r2、最新记录T-05进行/完成，实际delivery尚待核；本Plan不能追其tip或按某个勾选认定交付。

知识权威仍在auto-lang：[overview](../../../auto-lang/docs/specs/auto-graphics/overview.md)、[音画旧专用边界](../../../auto-lang/docs/specs/auto-graphics/auto-source-driver.md)、[资产](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)、[事件/新Run](../../../auto-lang/docs/specs/auto-graphics/runtime-triggers-and-rounds.md)、[G2/G3分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。006候选仅node-animation3d提供1credit输入快照，不能默认为旧sprites2d也有该函数。公共服务新profile必须显式定义2D有限输入快照/3D继承语义，旧profile字节与价格保持。

来源/当前代码指纹与Kenney声音/字体只读审计见[规划基线](attachments/012/planning/service-baseline.md)。SD路径以现有auto-lang知识仓为根，本轮不改canonical现状Spec。

## 5. 详细设计

### 5.1 首任务与整链先证

T-01硬核006实际交付：独立review、冻结定向门、master代码SHA、SDK/锁/identity/消费回执与公开节点视觉/聚合输入合同；没有delivery则保持drafting。T-02先写`attachments/012/work/public-audio-contract.md`，冻结2D/3D显式profile/CLI/schema/函数签名、有限packet/计价、codec/voice/queue资源账、真实PCM/native测试入口与failure图，整路径probe成立才进入T-03..06。

保持32credits、1000steps/slice、100000steps/callback、120resumes/250ms、30s初始化/2s tick和8ms pump目标；目标met/missed/unmeasured与硬限通过分列，不拿2s代8ms。保持256/96/352KiB源码、8资源身份/32MiB raw、CPU/GPU各64MiB、项目assets磁盘8文件/32MiB等旧帽。新增raw/PCM/queue/voice owner全入同一账。完整事件分支实测`1 snapshot+26 move+1 grounded+1 next+1 pair+1 phase+1 bundle=32`；bundle本期至多1visual+4音效、每callback一次，显式1credit，有限CPU/步数照计。新2D snapshot沿原0/1左右、2跳/3重开且bit4/5为0，3D沿006六action顺序，pressed混读/重复消费不重放；不得为统一布局重排旧action。旧getter价格不降、旧profile不获得新API，不许省输入/visual/event字段来凑数；超额再添一次charged调用必须拒绝。new bundle与独立emit同类混写/两次bundle/重复目标具名拒绝。若新2D输入快照、codec或整组合不成立，带反例回new，不直接抬限。

### 5.2 支持、闭包与预算

只接受Ogg/Vorbis和RIFF/WAVE PCM16、小端mono/stereo、22050/44100/48000Hz，单raw<=8MiB、时长<=10s、解码float PCM<=16MiB。header/packet/span/乘法/codec setup与预测帧数先检，解码逐块受限、decoded数量/length后检；不以无界from_cursor后置计账。格式unsupported/损坏invalid/超额resource_limit/设备unavailable分开命名。旧文档不悄悄接受新format/cue字段。

项目最多8 cue，指向已接受raw身份及[start_frame,end_frame)有效范围，gain有限[0,1]；音频可为一个含多cue的bank，多cue共享同一PCM不重复拷贝。每tick<=4请求、每Run同时<=8 voice，溢出拒绝候选，不抢占/丢弃/无界排队。素材转换成bank是正式可审计派生，不改原件来源或宣称已生成；5模型/图+bank1+font1的7身份组合只作资源设计，T-02/06实测。不引入隐藏免费音频cache。

raw SHA/长度/format/采样参数/cue域与版本化文档绑定；安全本地路径/祖先reparse检查、blob先落地、单entry最后提交/读回确认、历史/pending/current union与损坏修复再打开沿既有规则。SaveAs用已接受字节，不能在播放时重读导入目录；完整engine consumer/项目/许可离线搬迁。

### 5.3 效果接受和生命周期

Auto决定事件ordinal（Run内正整数严格递增，bundle内有序唯一），同逻辑结果的传输/呈现重放按(Run,tick,bundle,ordinal)幂等，跨tick复用ordinal/旧Run拒绝。Run/tick/ordinal/cue/slot走精确整数wire，不经f32，拒分数/非有限/精度损失，T-02冻结整数上界/耗尽拒绝。Host校验全部资源/身份/queue/voice容量后才accept；预接受失败visual/状态保持last-good且无新cue提交，已有voice按Fault停止策略处理，不倒退其已发样本。同tick以逻辑接受的Run/tick绑定，PCM从下一可用mixer quantum生效，实际延迟另报，不承诺和GPU硬件同纳秒。接受后只发送一次；render/capture/多挂载不驱动发声。物理已发样本不可回滚，postaccept设备断开/输出失败具名audio Fault，停止该Run剩余voice/队列、保最后接受视觉，禁止重试补播；不能假称旧声音被撤回。

manager由Host显式持有可跨Run复用，Stop撤销Run voice/queue而Host最终关闭再释放manager；生命周期oracle区分这两种owner。Pause/Resume/Stop实际输出以下一本服务quantum生效、已交硬件缓存单列。Pause冻结voice播放头、不推进新请求；neutral Step为静音调试，更新可视一次，新音效标suppressed_by_step并消费ordinal、Resume不补播。Resume仅续已有合法voice尾部。Stop/Fault/关闭/新Run从下一本服务mixer quantum起不再产旧Run PCM，硬件已缓存边界单列；新Run ordinal/输入/队列清零、旧handle无权操作。Editor/外部pin可留静态raw/PCM，但不能留旧Run发声owner。20轮/暖触发查真实voice/manager/PCM弱引用和decode计数。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/public-audio.md | 专用音效→有限公共资源/cue/真实接受后播放/PCM与生命周期 | G2-09/G3-08/G12a前置 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/scene-assets.md | 旧GLB/PNG域保持，新版本音频闭包/共享旧预算 | 可搬迁且不隐藏开销 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-graphics/behavior-api.md | 新显式2D/3D音频与有限bundle/输入快照，旧装配不变 | 32credit组合可审计 | AC-03/05 |

## 6. 测试设计

待T-03..06建立`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test public_audio -- --test-threads=1`、`python experimental/autographics-auto-driver/tools/check_public_audio.py --controls`及`verify_plan012.py --controls`。T-02核真实参数与PCM观察点；零选择/partial/旧runner不绿。只改动面/新增AC直接正反例与可复现旧样本，不跑根Cargo/全AG，OS抢焦点自动化暂停。真实GPU/音频设备单实例。异步音频线程解码/混音CPU、每quantum样本数/耗时/deadline或underrun与VM/Host/publish分列，不能用异步线程隐藏开销。

| V/S | AC/owner_stage/T | 输入→真实路径 | 区分oracle/关闭条件 | 边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01/02 work T-02/03 | 合法Ogg/WAV/cue、坏span/channels/rate/duration/codebook/超额/损坏重开 | 先检/受限decode、真实样本与预算账；资源/保存损坏拒绝、修复再读成功 | 有限codec，不任意文件 |
| V-02/S-02 | AC-03 work T-04 | 真实Auto两cue/gain、重复capture/传输、重复ordinal/旧Run、32/33与坏bundle | 真mixer PCM波形/位置/幅值及native本服务stream，不只play计数；重复不多播、原语义/旧profile保持 | 不录音/系统混音或保证物理扬声器声压 |
| V-03/S-03 | AC-03/04 work T-04/05 | preaccept坏资源/容量与postaccept设备失败、Pause/Step/Resume/Stop/新Run | 前者visual全last-good/无新cue PCM；后者具名Fault不补播；Step静音且Resume不积压，Stop下一quantum无旧Run样本 | 硬件已缓冲音不作可回滚承诺 |
| V-04/S-04 | AC-04 work T-05 | 20轮、暖触发/voice8/第9拒绝、raw/PCM共享与外部pin | 实际owner/manager/queue释放和有界PCM、warm不重解码；合法pin最后drop后查 | 驱动内部缓存不作内存归零证据 |
| V-05/S-05 | AC-05 work T-02/06 | 两维完整输入/运动/事件/visual/audio组合、fresh独立consumer离线A→B | 原32/源/CPU步数与资源额，真正事件分支有效；两cue换源敏感、identity错pin/partial拒绝、旧装配保持 | 不等于完整两关或HUD已交付 |

work自查后同树独立上下文review全部AC/V/S/SD、计价/codec/不可逆效果边界与遗漏；不新建agent/chat，review pass显式merge门pending。原始PCM/PNG/log/consumer放本地plan-artifacts/auto-ui/plan-012，attachments仅md摘要；native设备不可用要具名阻塞/记录未覆盖，不能拿假sink成功冒充真实输出。

复审还需核新改面无未处置新增warning、格式正确、无杂散debug输出；既有SDK警告按原来源单列，不借本内容/服务计划批量修无关路线。实现/内部自查/最终review同树顺序，未指定模型，独立上下文不是另开同步代码树。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 006实际输入与支持域/共享预算先冻结，V-01；不改旧API/预算 |
| AC-02 | work | 音频/cue身份、准入/受限解码、保存搬迁/修复闭包，V-01 |
| AC-03 | work | 公开Auto请求、完整接受后一次真实PCM与native本stream输出、bundle与32/33，V-02/03 |
| AC-04 | work | 容量/前后接受故障/Pause/静音Step/Stop/新Run/owner完整，V-03/04 |
| AC-05 | work | 两维整链/预算/fresh离线消费/current identity/旧profile兼容及下游说明，V-05 |
| AC-06 | review | 独立核完整AC/V/SD/边界/负控/warnings/fmt，无未处置finding，merge门pending |
| AC-07 | merge | 冻结提交定向门/复用审计通过后仅合本仓引擎，全AG里程碑pending |
| AC-08 | merge | SD/ledger/索引、归档/准确delivery、guard clean清树/分支/空组及恢复收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、006实际delivery | 核公开合同/SDK/锁/identity/设备条件与owner，建唯一ui-012树 | AC-01/05 | [x] f7c83c6基线；006独立R2/实际c681663/1477identity controls与SDK681/17包29边479metadata通过；native输出设备可用，执行合同含边界 |
| T-02 | work | T-01 | public-audio-contract先冻结支持/计价/bundle/效果时序/PCM入口，两维整链可行性probe，不足回new | AC-01..05 | [x] 67ddbe2修正F01服务独立owner预算；原ef438f9支持/计价/整链32-33/PCM/旧15证据复用，现账/兄弟路径见r1-repair-closure.md |
| T-03 | work | T-02通过 | 有限codec/cue/asset闭包与存储版本化，V-01 | AC-01/02 | [x] 67ddbe2，Registry预算上下文贯通共同Usage/解码前，资源9/9回归；codec及存储不改，原证据复用 |
| T-04 | work | T-03 | Auto/Host有界请求与mixer/native输出、接受幂等/32-33/错bundle，V-02/03 | AC-03 | [x] b10cc6a；两维各24实际VM输出破坏、精确大ordinal/4-5请求、整段PCM span/gain/tail/三rate及真实native/multimount重放通过；t03-t04-closure.md |
| T-05 | work | T-04 | 生命周期/设备故障/容量/20轮owner及完整账，V-03/04 | AC-04 | [x] 67ddbe2，New/Open/empty-cue/首次audio/duplicate/combined-pose及legacy64MiB控制3新case通过，start-restart2/postfault1通过；40轮未变证据复用 |
| T-06 | work | T-05 | fresh两维消费/预算/兼容/identity controls、fmt/提交、SD与G12a交付说明，V-05 | AC-01..05 | [x] 67ddbe2，capture仍比15golden/错golden必败/拒旧capture-only；1534新identity及SD，source-only A→B coldSDK/target离线functional1/1filtered通过；完整44case门merge pending |
| T-07 | review | T-06 | 同树独立final review，merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/有效复用后合master，不带Gallery内容 | AC-07 | [ ] |
| T-09 | merge | T-08 | 知识SD/ledger/索引、本仓归档与实际delivery | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后清唯一树/分支/空组，保必要本地恢复材料 | AC-08 | [ ] |

## 9. 复审记录

2026-10-30 new r1：`stage: new | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: blocked | next: 等006实际delivery后用户启动T-01/T-02`。0/10；所有新API/codec/PCM/bundle均为提案，本轮无实现或性能/设备测试，不以预计32和式宣布可行。G12a由ag-examples/005承接。

2026-10-30 文档预检：本批五Plan各0..10节、10个T/8个AC/5个V和S、职责/SD/依赖入口完整，87个本地链接可解析；编号/计数一致、依赖无环。9份固定原始文件与Git对象逐字节一致。仅只读身份/文档检查，未解码/播放/生成bank-font/运行GPU；预计32和式/资源数不是通过证据，此结果非独立final review。

2026-10-30 work foundation checkpoint：`stage: work | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: pass (checkpoint only; work_complete:false) | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | code_commit: fe59bc0 | dependency_revisions: 006=c681663e6ff392d70471abbec560e9d9daf12622,SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467; direct cpal/Symphonia references only, no package version change | task_ids: T-01 closed,T-02 partial | evidence: attachments/012/work/public-audio-contract.md; t02-current.log SHA256540b86452b6126274d6cfabed8b4bec969c12c4a7b1e4a9377973bea9a7b656 | blockers: none | next_responsible_role: work T-02 authoring Host/ledger/support closure`。用户全链授权已记录§4；006独立R2/冻结34case/实际master祖先/归档cleaned及1477identity controls实读通过，SDK681/17包29边479metadata通过。主检出其他所有者文档WIP保留；另一会话新增014/.next-id等只读观察，未纳入本提交。新显式audio2d/version111/profile22、audio3d/version112/profile23；固定1visual+4sound bundle共1credit，旧15profile文件未改。真实两维完整活动tick2均event101、32成功/再添1credit error10，managed85055/84143B；6producer+3codec共9case/0failed/0filtered通过。真实Auto受检packet经共同accept闭包到同一PCM mixer/native服务stream，stereo[0.1,-0.4]、970nonzeroframes、max135us、deadline_misses0，设备为ToDesk虚拟扬声器，不宣称物理声压或作者Host已接线。原3Ogg hash匹配，受限decode实际jump7872mono/coin12864stereo/land2496stereo均44100Hz；恶意codebook/维度/channels/blocksize/时长/CRC在decode前拒，合法重CRC控制仍接受。合法Vorbis overlap/padding用有界窗口预留并后检真实长度，不截断样本假装等于granule。fmt/diff/SDK681检查通过，SDK146既有warning保留、未见本期新增warning。合同新增codec边界解释与尚未关闭项；T-02作者Host完整同帧/资源存储账与旧15字节/支持域仍开放，T-03..10均pending，状态executing、1/10，不提交review/merge、不发布规范或新identity。原始log/PCM/EXE仅仓外本地保留。

2026-10-30 goal round1：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: fe59bc0 | next: work T-02`。本轮为实质实施/准入/真实输出可行性进展，不是work-complete verdict；1/10，0failed final review/0repaircycle，无用户待答。下一轮复读本Plan与唯一树，继续T-02 Host/计账/旧装配域，仍须先关闭T-02才开放后续正式交付任务。

2026-10-30 work T-02 closure：`stage: work | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: pass (T-02 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | code_commit: ef438f9cff1d6313100ca0e1873079da31b7bd09 | dependency_revisions: SDK681/two lock package versions unchanged; no dependency code writes | task_ids: T-02 closed | evidence: attachments/012/work/public-audio-contract.md current closure | blockers: none | next_responsible_role: work T-03`。scene7资源采样元数据/cue、共享Registry/raw/CPU账、Editor stamped audio edit和作者Host普通Run默认native已接通；共16typed scalar wire随同visual/motion/camera/event候选，prepare/voice检查→共同accept后才提交一次。Pause单个在途候选沿原Play完成并在同锁保持新voice暂停；neutral Step标suppressed_by_step、消费ordinal无voice/Resume补播，Stop后manager保留但Run/voice清零、真实下一服务quantum观测Run0/nonzero0；旧begin/control受Run high-water拒。33rd调用或GPU prepare失败保last-good完整Frame/GPU图且Fault停止声音，后续pump不覆盖原因。原声native最后2D/3D各nonzero441frames、max83/78us、deadline_misses0；manager33504B，2D CPU249993B，3D含pose1381208B；VM/Host8ms目标仍unmeasured，T06负责完整性能与consumer门。warm新增tick零decode/mesh上传。保存复用已接受音频字节，ogg/wav根磁盘清单/配额/保留名/verify闭包接通，A→B冷重开并再Run通过。

原round1 padded样本计数经独立soundfile0.13.1/libsndfile1.2.2确认非最终interval：修正pinned demux单个first+last音频page将tail误归start的边界，raw blocks后显式使用受限容器有效区间，保留意义样本、去声明padding；临时前后window先预测计账，保留PCM仍10s/16MiB。三原件正确7403/12643/2458frames、44100Hz；额外三rate×mono/stereo×单/多页12控制共15组完整PCM shape与波形对照通过，maxabs1.788139343e-7，非本解码器自我对照。原始完整PCM/生成源/外置oracle工具只本地，report SHA256f5b5a439dd8e82bad9729e6d3b748247defe23fe7d58c9f8207a4826cf5da476；小型独立预期样本/长度JSON是永久测试夹具，不以旧padding结果作新cue域。受限VQ/VLC/payload检查仍先于decode；pinned lookup1无法表达的整数运算具名unsupported，支持域已写合同，不宣称任意Vorbis配置。

实际三cue bank22504frames/90060B/SHA b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d，量化误差<=1.525878906e-5、无重采样/超幅削波；公开三cue共用同一PCM Arc。真实4模型/共图5raw+bank1为6raw/236544B、CPU1448422B/GPU1145956B；同一生产candidate预算接受第7字体slot26828raw/保守4MiB CPU+GPU预留，字体准入渲染仍属013，未假称本期已实现。旧15profile实际装配combined/api/bridge/version/profile均与同树临时编译pristine f7assembler的基线相同，临时enum穷举兼容只覆盖从未选择的新variant，finally逐字节恢复当前实现，无第二检出/agent；永久旧hash基线与原capture收据可复验。

当前14integration(6producer/3codec/1compat/4Host)0failed/0filtered，额外bank1selected/33unrelated filtered通过；Run watermark影响重放audio+Host10/10，去除未使用假decode字段后只重放受影响Host预算1/1和bank预算1/1，其余证据按未变路径保留。主要log：t02-integration-frozen SHA2d17ea34091cc47e10ddf85c1d1f5d938f69a116433917033fd4e9374d718268；t02-final-audio-host SHA f5a7d9bb65c309384b1520c97f70a1c3c34ab324d9b6f6b9412fcb8deb9199e1；最后metrics logs SHA c52523438c6d0b9e77b6406924f84dd67093adc356bf2ff59276cfb80195927e /364a06b36954c3b50d3efa15fb31fc43e2cc7c9be3b9e92b028dd3c28cbbbdc4；全在D:/autostack/plan-artifacts/auto-ui/plan-012。fmt/diff和SDK681通过，SDK146既有warning保留，无本期新增warning。新源已提交clean，未生成最终012identity、未把旧006验证器当当前新版验收，未运行root/全AG门、未发布Spec/ledger、未交复审或merge。T03..10保持开放，不把可行性等同完整AC交付。

2026-10-30 goal round2：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: ef438f9cff1d6313100ca0e1873079da31b7bd09 | next: work T-03`。前轮是基础实施进展，本轮解决真实作者Host/持久化接线、取得独立Vorbis格式反证并修正语义，再冻结完整组合/支持域/共享资源账，属实质进展；2/10，0failed final review/0repaircycle，无用户待答，目标未缩减。下一轮按T03关闭准入/保存损坏修复完整负控，后继AC/gates不提前打勾。

2026-10-30 work T-03..05 closure：`stage: work | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: pass (T-03/04/05 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | code_commit: 4400661a04c456b710e757cfa697876bc98adcc9 | dependency_revisions: SDK681/two lock versions unchanged,no dependency code writes | task_ids: T-03,T-04,T-05 closed | evidence: attachments/012/work/t03-t04-closure.md,attachments/012/work/t05-lifecycle-closure.md | blockers: none | next_responsible_role: work T-06`。T03提交0bb1ce1，resources9/codec6实际schema/cue/采样参数/容器边界/保存损坏修复/冷重开及共享raw8/32MiB/CPU账负控通过。T04提交b10cc6a，两维各24实际VM输出破坏经普通Play→独立mixer prepare→共同accept，禁止部分状态/PCM；public_audio9case、native格式unit、multimount实际1case通过，完整独立波形span/gain/tail三rate及本stream真实提交波形/重放一次性通过；观察点修到设备channel mapping/量化之后，mono相消不能假非零。

T05提交4400661，3Host定向case（两维各8/9voice、真实已发PCM后同CPAL错误终端路径注入、各20轮启停）+1非恒定waveform暂停cursor/Step+2启动/回合重启unit通过。warm实际0decode/0mesh上传，Stop实际VM owner死/Run0/voices0、下一服务quantum无旧PCM，Host最终drop才manager弱引用死；合法外部PCM pin保留并最后drop后弱owner死。postaccept Fault保最后Frame/GPU/已发计数，无重试补播，freshRun重建故障stream。统一begin先drop坏stream再open，避免清fault假恢复/瞬时双manager；回合重启同样恢复，失败保旧展示；初次音频启动准入失败重装真实editing GPU camera。控制变异复用坏stream确实在永久unit弱owner断言失败，finally恢复后通过。设备均真实打开成功；注入的是CPAL实际错误回调共用终端方法，不假称物理断开。硬件已缓冲样本不计可撤回边界。

详细命令/exit/count/SHA在两份摘要；T05关键log：t05-owners SHA adfb9af16495645eb0200d348bc2d6d722a704c6defb0f7c90b3e7a2c7360308；t05-device-fault SHA728a17ae37a240da541789c720090dba441b99025bf8f0c1c92fb6f6421e34d8；t05-start-restart-final SHA1c2cc055bc33c67ee32e42fc689785ffb969ab4019bb38a2d5b06222b96d1ebb；均本地D:/autostack/plan-artifacts/auto-ui/plan-012。fmt/diff通过，SDK146既有warning保留，未见本期新增warning。变更后的成功路径与前期资源/codec/wire隔离，未重跑无关族/根Cargo/全AG；仅最终camera失败支路额外定向复验，冻结果门仍review/merge所有。当前5/10，未交review、未发布Spec/ledger/新identity；T06 fresh两维独立consumer/实际性能/012identity controls/SD与下游说明开放，T07..10仍pending。

2026-10-30 goal round3：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 4400661a04c456b710e757cfa697876bc98adcc9 | next: work T-06`。本轮关闭T03/04/05并以设备恢复反例修正同源边界，实质进展；5/10，0failed final review/0repaircycle，无用户待答。下一轮关闭两维fresh离线消费/完整预算/identity/规范增量后才交独立复审，不把本轮定向绿等同全部AC通过。

2026-10-30 work final handoff：`stage: work | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | code_commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | dependency_revisions: SDK7b987ca8/681/17local/29edges/479metadata,driver lock4a0b3040,AGP lockc3df168e,no package version/dependency code change | task_ids: T-06 closed,T01..06 complete | evidence: attachments/012/work/t06-readiness.md | blockers: no work-owned runtime/contract finding; review-context action authorization pending §10 | next_responsible_role: final review on frozen same tree`。AC01..05/V01..05工作侧全闭合映射见readiness；新增两份普通公开scene7接受项目，production bank派生与import/CodePayload/Editor/Storage保存生成，原始模型/图/声音及许可不变，bank-audit与项目脚本/资产为永久承重夹具，不是本地probe日志。真实Auto的tick1选cue0+2、event101后tick2选cue1+2，改源码选cue2/gain0.75/speed0.25时实际12645frames完整独立PCM16波形与运动随之变化；SaveAs到A、drop旧Host、搬B冷读/新VM复验成功。

源identity现1529files，SHA a5a861cd5275c8f8d1d02cea7deae49a3ea67c986a7513c712c183fb33cffbe6；正确012完整源guard已取代旧006默认guard，SDK/两locks全绑定。Python compile/identity edit-delete-unlisted-stable-scene-wrongpins-escape-target-PID-exact-source controls通过。真实complete2case log接受、partial1/wrongrunner/实际不存在filter0selected及6观察字段突变共9负控拒绝；最终整体manifest/41case gate不以这些局部控制冒充已执行。fresh工具只导出identity列出源码+SDK快照，A→B前无preparedSDK/target，再在B冷准备SDK/离线独立重编译、真实2tests/0filtered通过，PEmain32MiB/worker16MiB，实际B binarySHA97157b1f651845820bdb3b1dfd0a93786fba3a2f8c3416fb9247997227ff536d；manifestSHA d2c02d7a905ea3ac9d2f7278dc3b852b212aff83f506505d6fd0a7d24e9fa108。实际破坏B preparedSDK被拒、finally修复后重新verify_source/metadata接受，先前保护修订前的fresh旧identity拒绝。未复制producer target/.sdk-source/Git或第二实施树，所有consumer/PCM/log仅仓外本地。

两个12tick完整组合callback分别5092steps/6slices、7037/8，managed85055/84143B，资源CPU308844/1479646B、GPU256/1145956B、raw2/90202B及6/236544B，warmdecode/mesh/texture上传0。最终独立副本tickmax4860/13443us；Hostpumpmax4867us/met与8046us/missed，3D有1次yield故13pump观察/12tick，8ms目标超46us明确记录不拿2s硬限替代。VM-output/admit/prepare/publish/pump与native分别观测，native441frames/44100Hz stereoF32、max621/169us、deadline_misses0、nonzero9699各；Resume到首次PCM观察上界8390/3169us，1ms轮询/硬件缓冲及无CPAL underrun计数明确边界。producer先前两维pumpmet结果不覆盖fresh3Dmissed。全部硬帽保持，导入decode同步计时不藏到异步线程。

SD01/02/03与ag-examples005/G12a下游说明冻结在research/public-audio-spec-delta.md，SHA551376a3654baf3c8f916ff1c858879b6660c10dbd598361dab2e23337a89f82。fmt/diff/源与SDK校验通过，无本期新增warning，SDK146既有warning保留。除4行有界opt-in Hostpublish观察外此前资源/codec/wire/native/lifecycle/legacy实现与输入不变，相关已标识T02..05证据按路径复用，最终冻结定向门明确merge所有且pending。实现已提交clean并暂停写入；主检出其他所有者文档WIP原样保留。当前6/10、execution_done；未做独立review、未生成finalpass、未跑root/全AG、未发布canonical Specs/ledger、未merge归档清树。

2026-10-30 goal round4：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: waiting | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | next: user review-context authorization then review`。前轮分类progress；本轮关闭最后work任务、独立消费/性能/身份控制与SD实质推进。0failed final review/0repaircycle；首次等待§10上下文约束决定，不因未经授权新建agent/chat，亦不把实施自查称独立复审。所需下一动作已具体到冻结树/commit的只读独立审查；未消耗finalreview尝试或开展未授权复审。若用户允许例外则进入T07；若保持限制则等待用户安排独立复审，不改验收/阈值。

2026-10-30 goal round5：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 5 | from: review | to: review | status: execution_done | outcome: waiting | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | next: user review-context authorization then review`。前轮分类progress（已提交实现与实测证据），但末尾开始等待§10决定；本轮复读§9/10仍无用户答复，未运行阶段工作或任何检查、未启动agent/chat、未合并，代码冻结状态不变。相同工作流阻塞连续第2轮（round4/5），目标保留active；不以自动继续消息或经过时间代替人工授权。已有具体只读复审提议，继续等待同一答复，不重复提问、不修改验收或测试阈值。

2026-10-30 goal round6：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 6 | from: review | to: review | status: execution_done | outcome: blocked | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | next: user review-context authorization then review`。前轮分类no progress（仅等待/簿记，不是已验证的运行中进程等待）；本轮仅复读§9/10确认仍无人工答复，无阶段工作/测试/agent/chat/合并。相同阻塞已连续round4/5/6三轮，且缺少授权无法满足既定独立复审要求，没有不依赖答复的必要交付工作可继续，现将host goal标为blocked，保留完整目标与全部实现/证据/唯一树。解除动作：用户明确允许仅本计划的同树只读复审子代理，或安排既有独立上下文完成冻结87414431的T07复审。不是因难度/预算/耗时阻塞，不宣称完成或降低AC06；0failed final review/0repaircycle。

2026-10-30 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent read-only /root/plan012_final_review, fork_turns:none; no other model/session identity asserted | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | reviewed_commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | dependency_revisions: SDK7b987ca8/681/two locks unchanged | spec_inputs: six canonical hashes and frozenSD551376a3 in attachments/012/review-r1.md | acceptance_results: AC01/02/04/05 partial(F01);AC03 path passes subject F01 shared budget;AC06 fail(F01/F02);AC07/08 merge pending | pending_closeout_gates: repaired frozen directed gate/valid fresh proof,canonical SD/ledger/archive/guard cleanup | findings: F012-R1-01(P2 sharedCPU),F012-R1-02(P2 legacy capture-only evidence) | evidence: attachments/012/review-r1.md SHA c961eb3ff600d85b37d081e338433c7caa2e5d082718e6ffccadd2c8e326e380;external review-r1 raw inputs/logs/probe hashes | next: work bounded repair then same independent affected re-review`。

F01实际正式路径Run→Stop→New后manager仍存33528B但资源CPU0；四张合法2048²PNG均普通import接受为67108864B，实际契约内最低67142392B越64MiB。空cue无音频资产的合法Audio2Run在同64MiB图像上也新开manager并成功，manager-free legacy同四图合法正控通过。原因是service预算只嵌在音频Asset，而生命周期属于Host。F02实际PLAN012_CAPTURE_LEGACY环境使兼容test捕获后提前return，未比旧15golden却1passed，真实gate counts接受；普通无变量比较通过。两个反例/控制/定位/原始fixture/命令/预期实测与最小关闭范围完整在R1报告，不能以现有绿覆盖。

其余codec预分配/有限域、schema/身份/存储闭包、精确int/共同accept/PCM实际native/重放/8-9容量/Pause-Step-Stop-Fault/新Run/实际owner/变源消费/SD边界完整核查，无其它发现。独立定向Pause1/9filtered、wire1/37filtered、normal legacy1/0filtered、source/SDK1529/681/479及fresh-final验证通过，known146SDK warnings保留；未改实现或跑41case/root/全AG。原错误selector0selected与修正probe另记，不作通过证据。结论前HEAD/clean再核8741443未漂移。T02/03/05/06重开至2/10，保留未受影响输入证据和历史收据，未发布规范/merge。

2026-10-30 goal round7：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 7 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 87414431b9a1dbda6eb54aad999af7f734a14516 | next: work F012-R1-01/02 then affected independent review`。用户明确授权解决此前阻塞，本轮实际独立全矩阵审查取得改变下一动作的正式反例，属progress；第1次failed final review/第1个有界repair cycle，计数不重置。host仍blocked无自动续轮，按原全链授权在当前人工继续回合顺序执行后继逻辑round8work，不创建第二goal、不伪称active。修复不改目标/预算/支持域：将真实required/resident服务owner计入共同准入/解码前/报告，保持legacy64MiB无manager正控与Stop复用；兼容捕获不得绕过golden且validator拒capture-only。受影响consumer/source绑定刷新，未受影响GPU/PCM/40轮证据复用，最终统一门仍merge所有。

2026-10-30 work R1 repair ready：`stage: work | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-012/auto-ui | branch: codex/plan-012-dev | base_commit: f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131 | repair_base: 87414431b9a1dbda6eb54aad999af7f734a14516 | code_commit: 67ddbe2e61693773900af9630ef455d636d55108 | dependency_revisions: SDK/two locks unchanged | task_ids: T02,T03,T05,T06 closed | evidence: attachments/012/work/r1-repair-closure.md | blockers: none work-owned | next_responsible_role: same independent reviewer affected R2`。F01将Host required/live服务需求放入非持久Registry预算上下文，共同Usage在无audioAsset覆盖时计Native33528B，Asset保守service预留覆盖后不重复；上下文clone经import/峰值预测传递，新增服务在open前检查，Run/restart合并asset+service+poseCPU再准入。未造假asset/blob/cache、抬额或拒原合法legacy域。New/Open真manager0asset时CPU33528；三原PNG后第四全图/少4行图先于decode拒，少5行真第四图/duplicate/窄余额首次32frame音频接受且计一次服务。原R1四PNG逐字节成为84769B永久回归输入，非附件证据倾倒。另真实character+shared图+RGBA构造两个独立预算都fit但三者和越64M的组合，Audio3在Native创建前拒、同资产pose的无manager旧node-animation接受。原64MiB四图无manager仍成功。最终budget2/9filtered、pose1/10filtered、共同resources9/0filtered、start-restart2/36filtered/postfault1/10filtered通过。

F02移除capture提前return，实际capture仍断言未改旧15golden并输出完成marker；gate清环境变量、要求marker且拒旧capture-only，永久控制从真实log移除marker并更新log hash应拒。实跑normal/capture各1/0filtered通过，capture+故意坏golden实际exit101/failed，finally原bytes恢复后normal通过，旧R1capture-log与failed-log被当前predicate拒。未改变旧profile源或期望baseline。1534新完整源SHAfbea61472b0e28ec113b93d4dd0d207d17a2c817ed1a7f39168df4997374c9b4，SDK681/17/29/479及两locks不变，identity controls/fmt/diff/Python compile通过；SD既有owner/组合pose与proof规则补充，SHA6b693eeadabbf68bf1d22f8cd28b913c1d0189b1507d99ad0a943a32fb24f8f2。R1原始外部报告SHAc961eb3f保留，持久Markdown GitLF摘要SHAd2dc3db85b84aa32c41b659187ce3b8334443b139599d2cd04cb60c2ac4123f5（文字等价，不伪称同原始字节）。

受影响source-only新A→B在prepare前无SDK/target，在B自有snapshot准备/verify源及metadata后locked/offline重编译95s、functional真实1passed/1filtered/6.17s，两维变源cue/gain/运动及12645framePCM/保存搬迁冷开通过。实际B manifest_dir和PE32MiB/worker16MiB，exeSHAb6f359ad085fa149420050d5971855de7c08703a5a0406ce06d86f0a566c0dd7；B/producer identity逐字节相同，r1-functional-proof.json SHAe83c55aea93699acc1473dd9d2f83c33593dd8b84fde3cef2abdf0aec4eabfe7明确functional-only，不冒充完整44case门或给旧full报告改身份。未变Auto/wire/codec/Mixer/native转换/GPU视觉/40轮owner及旧性能原始证据按路径复用，完整current performance/fresh gate为merge-owned pending。所有局部命令/结果/LogSHA见closure；无本期新增warning，SDK146旧warning保留。

初始cargo check在Windows依赖metadata遭D盘满而失败（非通过证据），随后定向编译补齐Registry map-array转换兼容后通过。自动审批拒绝两种缓存删除且均未执行；改以不删除的NTFS压缩过期首consumer target，8666files/885dirs，5093094599逻辑B→2618700258存储B，129EXE哈希前后相等/路径保留，无源码证据删除，磁盘恢复可用后完成检查。此环境处理不增加review失败次数、不作产品finding。

2026-10-30 goal round8：`stage: goal | plan_id: PLAN-012 | plan_revision: r1 | round: 8 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-012/auto-ui | commit: 67ddbe2e61693773900af9630ef455d636d55108 | next: same independent affected review R2`。第1个有界repair cycle完成两finding共因/兄弟路径/正反例与受影响消费。work-owned开放finding/覆盖缺口/合同决策为空，6/10；1failed finalreview计数保留，T07..10pending；无新Agent/chat或第二树，无规范/ledger/merge/归档。host无续轮（仍blocked）故继续按已授权全链顺序进入逻辑round9独立R2，不创建第二目标或伪报host active。

## 10. 待澄清事项

当前两R1finding已在67ddbe2完成工作侧修复及局部验证/新消费，6/10、execution_done，等同一独立上下文R2。无用户待答，原独立只读例外和全链授权仍有效。T08新44case冻结门、T09知识ledger归档、T10guard清理仍pending。下列待答/blocked及needs_fix文字是历史；原始目标和所有失败证据保留，不缩范围或改变阈值。

最新状态（2026-10-30）：用户已明确回答“允许只读子代理独立复审”，解除下列历史待答项。已核唯一树仍为87414431b9a1dbda6eb54aad999af7f734a14516且clean；只读独立上下文`/root/plan012_final_review`已开始T07，未新建用户聊天、源码副本或实施树。根会话暂停实现写入，reviewer只读源/允许最小定向检查，最终41case门仍merge-owned pending。host目标当前仍显示blocked（工具不提供恢复active接口），本次由用户明确答复继续授权任务，不创建第二目标；不声称host状态已恢复。无工作侧或复审方式待答事项。

待答：T01..06已完成，6/10，实施树87414431clean/冻结。§6同时规定“同树独立上下文review”与“不新建agent/chat”。auto-plan-review技能Entry还规定“Do not create another task or agent without the applicable authorization.” 仅实施会话变更角色不能声称独立；当前没有可用的既有独立复审上下文。已通过异步问题请求：是否允许一个只读复审子代理使用同一唯一树/冻结commit（不新建用户聊天、不复制代码、不修改实现），作为此项禁止新agent的明确例外？或用户保持限制并安排独立复审。未得到答复不启动agent、不把elapsed视为同意、不降低AC06。此为T07复审方式决策，无工作侧功能/证据缺口；全链merge授权仍有效，review pass前不得merge。首次waiting round4；目标仍active，未达到blocked阈值。

round6现状：同一待答事项连续三轮未解决，host goal转blocked（Plan保持execution_done，6/10）。上述“目标仍active，未达到blocked阈值”为首次等待时记录，现已由round6收据更新。用户答复后恢复目标，按当前唯一树/冻结commit核对后进入T07；未经独立finalpass不得merge。

参考约束：[本Plan§6](#6-测试设计)，[auto-plan-review技能](C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md)。下一步答复后同树独立review全部AC/V/SD，T08冻结定向门与T09/10知识ledger/归档guard清理仍pending。完整游戏G2-09/G3-08、011/HUD/font/完整AG里程碑不自动升级。
