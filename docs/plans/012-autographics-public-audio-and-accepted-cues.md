---
plan_id: PLAN-012
status: drafting
feature_name: AutoGraphics公共音频资源与接受后一次播放
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
current_step: 0
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

## 4. 需求分析与背景调查

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
| T-01 | work | 用户启动、006实际delivery | 核公开合同/SDK/锁/identity/设备条件与owner，建唯一ui-012树 | AC-01/05 | [ ] |
| T-02 | work | T-01 | public-audio-contract先冻结支持/计价/bundle/效果时序/PCM入口，两维整链可行性probe，不足回new | AC-01..05 | [ ] |
| T-03 | work | T-02通过 | 有限codec/cue/asset闭包与存储版本化，V-01 | AC-01/02 | [ ] |
| T-04 | work | T-03 | Auto/Host有界请求与mixer/native输出、接受幂等/32-33/错bundle，V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | 生命周期/设备故障/容量/20轮owner及完整账，V-03/04 | AC-04 | [ ] |
| T-06 | work | T-05 | fresh两维消费/预算/兼容/identity controls、fmt/提交、SD与G12a交付说明，V-05 | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立final review，merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/有效复用后合master，不带Gallery内容 | AC-07 | [ ] |
| T-09 | merge | T-08 | 知识SD/ledger/索引、本仓归档与实际delivery | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后清唯一树/分支/空组，保必要本地恢复材料 | AC-08 | [ ] |

## 9. 复审记录

2026-10-30 new r1：`stage: new | plan_id: auto-ui/PLAN-012 | plan_revision: 1 | outcome: blocked | next: 等006实际delivery后用户启动T-01/T-02`。0/10；所有新API/codec/PCM/bundle均为提案，本轮无实现或性能/设备测试，不以预计32和式宣布可行。G12a由ag-examples/005承接。

2026-10-30 文档预检：本批五Plan各0..10节、10个T/8个AC/5个V和S、职责/SD/依赖入口完整，87个本地链接可解析；编号/计数一致、依赖无环。9份固定原始文件与Git对象逐字节一致。仅只读身份/文档检查，未解码/播放/生成bank-font/运行GPU；预计32和式/资源数不是通过证据，此结果非独立final review。

## 10. 待澄清事项

无新增方向问题。函数/selector/编号、codec受限实现、整组合能否在原额内成立由T-02有界验证；若失败回new，不把本期扩大为录音/空间音频或抬预算。006未交付，当前不创建实现树。011不自动升级，完整游戏G2-09/G3-08仍需真实消费者后继验证。
