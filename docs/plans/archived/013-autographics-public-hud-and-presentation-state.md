---
plan_id: PLAN-013
status: archived
feature_name: AutoGraphics公共HUD与共同呈现状态
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
current_step: 9
total_steps: 10
depends_on: [auto-ui/PLAN-012]
supersedes_spec_components: [docs/specs/auto-graphics/scene-assets.md, docs/specs/auto-graphics/behavior-api.md]
new_spec_components: [docs/specs/auto-graphics/public-hud.md]
touched_goals: [GOAL-008]
---

# PLAN-013：公共HUD与共同呈现状态

## 0. 变更摘要

在012实际交付上补有限GPU文字HUD/项目字体闭包与Auto状态，版本化有限bundle同时承载visual、音效和HUD，0..3与Playing/Won/Failed由真实Auto产生。不做生产UI或完整游戏；Gallery G12b另由ag-examples/006承接。

## 1. 目标

两维公共项目可用至多4个文本槽显示动态计数/状态，HUD位置不随世界相机移动，能按游戏viewport/letterbox/DPI正确裁剪和布局。HUD/visual/对象/相机/音频命令由同一候选接受，预接受错误不得发布半新半旧结果；保012后接受设备故障的明确边界。字体/布局有界、可保存/离线搬迁，暖帧不重建glyph atlas，旧profile装配保持。

非目标：生产AutoUI组件/DOM、交互控件/焦点/IME/CJK/富文本/任意字体、世界空间深度UI、HUD编辑器、声音新通道或音乐、完整两关、Gallery目录/ZIP、新平台/远端发布。

## 2. 架构方案

新版本项目HUD/固定字体身份→有界准入/字体字形atlas→真实Auto完整有限槽状态→Host独立验证→与012视觉/音效共同prepare/accept→游戏surface最后绘制HUD，声音只接受后一次送mixer。既有TextRenderer可复用数学/管线，但不能把Squash固定Score路径推广为已公开API。

唯一实现/顺序复审树拟`D:/autostack/.wt/ui-013/auto-ui`、`codex/plan-013-dev`；硬等012真实delivery后从核实master建立。主检出仅簿记，准备/target/报告自有，禁止链接，移树先Git Bash wt-guard clean。与012/006共享协议实施串行，Gallery005可固定消费012并行。

## 3. 技术栈

已核AGP `src/render2d/text.rs`的fontdue0.9/atlas/layout/render、`host/surface_runtime.rs`；driver `authoring/{assets,document,storage,play,behavior}.rs`、`host/viewport.rs`及未来012有界音效与bundle。现有layout未知字形按空格处理，不满足公共HUD“缺字显式拒绝”，必须准入先检。

拟新增driver `tests/public_hud.rs`、`tools/check_public_hud.py`、`verify_plan013.py`及两维最小Auto夹具；均待T-03..06建立。具体schema/profile/selector/输出表T-02在012实际交付后冻结，不沿用候选数字宣称已支持。

## 4. 需求分析与背景调查

2026-10-30 复审例外授权：用户明确回答“允许仅本计划使用同树只读复审子代理”，解除§10等待。例外仅限同一唯一实现树/冻结commit的只读独立复审，不新建用户聊天、不复制代码、不修改实现；不改变功能、验收、阈值、仓库或merge前final pass要求。根会话仍是共享Plan唯一写者。

2026-10-30 执行授权：用户 `/goal 用 /auto-plan-goal 完成UI的计划013` 授予既有合同的 work→独立 review→有界 repair→merge/规范/台账/归档/guard 清理全链，不改变验收、预算、仓库或范围。012已实际交付`78632af`并归档10/10，独立R3/45case/fresh/清理收据实读通过；当前master `df39593`包含该交付。唯一`ui-013`树与`codex/plan-013-dev`已从该基线建立，主检出其他所有者文档WIP保留，无代码WIP。T-01基线核验和T-02冻结合同见[执行合同](../attachments/013/work/public-hud-contract.md)；合同中的真实两维probe仍未完成，不据此提前关闭T-02或后续AC。

用户2026-10-30授权公共声音/HUD后继规划、引擎与Gallery分仓、暂不实施。本轮只取号/Plan/设计/附件/引用；不建树/改源/生成字体或素材/编译/GPU/播放/agent/发布。当前012是新draft，006仍在途；不能消费其候选consumer。

共同有限协议、资源帽与声学不可回滚边界见[设计](../../design/autographics/m2-public-audio-and-hud.md)，素材/代码观察见[基线](../attachments/012/planning/service-baseline.md)。知识权威在auto-lang：[overview](../../../../auto-lang/docs/specs/auto-graphics/overview.md)、[旧专用HUD](../../../../auto-lang/docs/specs/auto-graphics/auto-3d-game-profile.md)、[资产](../../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)、[原游戏分母](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。本Plan SD以知识仓为根，本轮不发布canonical新能力。

## 5. 详细设计

### 5.1 先冻结012与完整预算

T-01核012独立review/冻结门/master实际SHA/音频合同/SDK/锁/current identity/消费包与owner，没实际delivery不建实现树。T-02先写`attachments/013/work/public-hud-contract.md`：字符/slot/anchor/clip/resize、字体资源/atlas账、公开签名/selector/新wire、完整槽状态表示、音画HUD接受与失败图，然后真实两维整链probe。只允许一套完整槽状态语义：4槽均有显式值/隐藏或空文本，缺行/重复slot/未知字段拒绝；未更新时保留已接受状态，新Run回app_start/文档初值，Stop回编辑default，不混用未版本化delta。

保32credits、1000steps/slice、100000steps/callback、120resumes/250ms、30s初始化/2s tick和8ms pump目标；目标met/missed/unmeasured与硬限分列，不拿2s代8ms。保持256/96/352KiB源、8资源/32MiB raw、64MiB CPU/GPU/8文件32MiB磁盘。新profile独有有限bundle扩至1visual+<=4音效+<=4HUD槽，每callback一次/显式1credit，CPU/步数照计；旧012 bundle/profile字节和价格不改。完整真事件分支必须实测`1 snapshot+26 move+1 grounded+1 next+1 pair+1 phase+1 bundle=32`且HUD/visual/音频均有有效数据，额外charged调用拒绝。旧profile不获得新HUD API，混用standalone写与bundle同类写或重复bundle拒绝；不足回new，不能删音效/事件/visual或靠Rust计分凑数。

### 5.2 有限HUD、字体和资源

每Run最多4槽、每槽<=32个ASCII可见字符/总<=128；支持隐藏/显示、逻辑像素offset、四角anchor、有限RGBA。slot域0..3，位置/字号有限、颜色[0,1]、字符串长度/码点先检；未支持glyph/非ASCII/嵌入控制字符具名拒绝，不填空格/方块假绿。固定一个来自Kenney commit3fa8a04的Lilita One TTF（26828B、OFL/RFN，SHA在基线），raw<=1MiB且只接受该已审计内容身份；项目`assets/<sha>.ttf`真实保存，fontdue前核SHA/长度，非任意TTF导入。

字形集固定ASCII32..126、字号16..48、单atlas边<=1024且<=4MiB，CPU布局/atlas暂存、字形表/顶点/texture、字体raw与012PCM及模型/图全部计原有总额；不把内置字体排除计账。5模型/图+一个多cue声音bank+font为7身份是可行性候选，T-02实际核完整集合，不许以9身份超额仍忽略font。最多4个当前/history/pending/外部pin保留的(font,size,device代次)atlas key；DPI不自动生成无限字号cache，T-02冻结栅格px与逻辑/屏幕变换。多个槽共享同font/size atlas，暖换数/隐藏不重光栅化或重新上传；历史/pending/current union与device代次隔离仍受检。

TTF/项目路径、字体许可、保存/搬迁/缺损修复沿012与旧blob/entry最后提交规则；SaveAs使用已接受资源，不能依赖系统字体/生产者导入目录。保留OFL与Reserved Font Name，不修改字体原件后仍沿原名发行为合法；本期不做字体修改。

### 5.3 GPU、时间与故障

HUD绘制在本Run游戏surface viewport最后一层，2D/3D均screen-space；世界camera orbit/zoom/角色运动不能改变同一HUD逻辑位置。窗口DPI/resize/letterbox以同一已接受viewport变换计算四角/clip，HUD不画进黑边/编辑面板、不宣称参与3D深度。文本/阴影或轮廓若支持须有界，不能隐含无限draw；必要背景只有限quad。

Auto写0..3与Playing/Won/Failed，Host不从事件/score推导，字体布局也不改玩法。坏slot/字符/资源/atlas容量/prepare拒整个候选且声音未送出，保持完整last-good。接受后的音频设备故障沿012保最后接受的视觉/HUD并具名Fault、不承诺撤回声音；不能误要求已发样本回滚。Pause不推进、Step只一次可视tick且沿012静音消费新cue、Stop/fresh Run/旧Run隔离与原输入边沿保持。

20轮Play/新Run/Stop/关闭检查HUD/atlas/PCM/VM实际owner和队列，外部pin/Editor合法保留与最终释放分清；暖计数不重建glyph/静态纹理。与012codec/native未改证据可严格绑定复用，但新bundle入口/同tick及0..3实字形必须新验，不拿旧Score截图过门。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/public-hud.md | 专用Score→有限公共Auto文字/位置/clip/同tick与字体owner | G2-09/G3-08/G12b前置 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/scene-assets.md | 新版本固定TTF/slot完整闭包、原额共享与旧域保持 | 离线/许可与无隐藏缓存 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-graphics/behavior-api.md | 新profile完整bundle含HUD，旧音频/visual装配不变 | 同一callback预算可审计 | AC-03/05 |

## 6. 测试设计

待T-03..06创建`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test public_hud -- --test-threads=1`、`python experimental/autographics-auto-driver/tools/check_public_hud.py --controls`与`verify_plan013.py --controls`。T-02冻结case/坐标/字形/色彩空间/像素容差；0selected、partial、旧截图/runner拒绝。

| V/S | AC/owner_stage/T | 实际输入→消费者 | 区分oracle/关闭条件 | 边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01/02 work T-02/03 | 固定font/合法ASCII、坏SHA/TTF/缺许可、超slot/32字/缺字/非有限/容量 | fontdue/atlas前受检拒绝；合法字形/共享预算与修复再读有效 | 一个审计font、非任意TTF/CJK |
| V-02/S-02 | AC-03 work T-04 | 真Auto计数0/1/2/3、Playing/Won/Failed与一次事件音效/visual，32/33 | 新鲜GPU实际字形/不同状态ROI与真实PCM/ordinal匹配；旧像素/错字/漏位/重复声音负控红还原绿 | 不以文字状态日志/命令次数替代图与声 |
| V-03/S-03 | AC-03/04 work T-04/05 | 2D/3D相机/resize/DPI/letterbox/四角clip、坏候选与设备后故障 | HUD位置按viewport而非world，黑边不泄漏；预拒全旧/无新声，后故障遵012 | overlay非深度UI/生产控件 |
| V-04/S-04 | AC-04 work T-05 | Pause/Step/Stop/新Run/旧输入、20轮/暖换数/合法pin | 可视一次tick/Step静音不积压、fresh0初态；glyph/VM/PCM weak owner与上传有界 | 不声称驱动缓存归零 |
| V-05/S-05 | AC-05 work T-02/06 | 两维整链/原额、fresh独立consumer A→B、旧profiles/字体缺损/错identity | 真运行保存关开再Run；坏源/空oracle/部分报告拒绝；原限额/旧装配保持 | 非最终游戏/G12产品包 |

同一唯一树work自查后独立上下文review全部AC/V/SD/缺字退化/隐藏字体预算/时序和遗漏，pass显式merge门pending。未授权新agent/chat；OS抢焦点自动化暂停，GPU/音频设备单实例，只变更相关与新AC正反例，不跑根t/tf/tv/tu/docs_gen或全AG。raw PNG/PCM/log/consumer本地plan-artifacts/auto-ui/plan-013，attachments仅md摘要。

复审还需核新改面无未处置新增warning、格式正确、无杂散debug输出；既有SDK警告按原来源单列，不借本内容/服务计划批量修无关路线。实现/内部自查/最终review同树顺序，未指定模型，独立上下文不是另开同步代码树。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 012实际delivery与完整资源/组合合同先冻结，原额/旧语义保持，V-01 |
| AC-02 | work | 有限slot/ASCII/审计字体/准入/许可/保存搬迁闭包，V-01 |
| AC-03 | work | 真实Auto文字/字形、viewport/同tick画面声音HUD、32/33敏感，V-02/03 |
| AC-04 | work | 完整pre/postaccept失败、Pause/Step/Stop/新Run、owner/warm与资源账，V-03/04 |
| AC-05 | work | 两维整链/fresh消费/离线恢复/identity/旧profile兼容与G12b交付说明，V-05 |
| AC-06 | review | 独立核AC/V/SD/负控/范围/warnings/fmt，无未处置finding，merge门pending |
| AC-07 | merge | 冻结提交定向门/有效复用后合本仓master，全AG里程碑pending |
| AC-08 | merge | SD/ledger/索引、归档/actual delivery、guard clean清树/分支/空组及恢复收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户启动、012实际delivery | 核公开合同/SDK/锁/identity/owner，建唯一ui-013树 | AC-01/05 | [x] df39593基线；012实际78632af/1536 identity controls与两份锁通过，唯一树/分支已核 |
| T-02 | work | T-01 | public-hud-contract先冻结font/slots/viewport/完整bundle/PCM图像oracle，两维整链先证 | AC-01..05 | [x] b29f2f8；public_hud 6/compat 2、旧音频23、17基线装配、scene8闭包通过，后续详细AC仍归T-03..06 |
| T-03 | work | T-02通过 | 固定font/slot文档与有界glyph/存储/预算V-01 | AC-01/02 | [x] 0230a0c；public_hud 9/9，文档/许可/7身份/atlas/公共import/storage负控通过 |
| T-04 | work | T-03 | Auto/Host/GPU overlay及audio+HUD+visual共同接受、32/33、V-02/03 | AC-03 | [x] 52f370a；R1每槽/状态GPU mask、stale/wrong/missing/restore负控通过，既有wire/common accept证据保留 |
| T-05 | work | T-04 | viewport/生命周期/故障/20轮owner/warm，V-03/04 | AC-04 | [x] 52f370a；四角越界clip/scissor/2xDPI与unclipped leak对照通过，其余生命周期证据保留 |
| T-06 | work | T-05 | fresh两维消费/预算/兼容/identity、fmt/提交、SD与G12b接口摘要V-05 | AC-01..05 | [x] 52f370a；consumer 2/2与非空mask/data oracle、1577 identity controls/fmt通过；merge冻结门pending |
| T-07 | review | T-06 | 同树独立final review，merge门pending | AC-06 | [x] R2 agent-0 pass绑定52f370a；F-013-R1-01/02关闭，17+2/mask/source/fmt通过，无新增finding |
| T-08 | merge | T-07 pass | 冻结定向门与复用审计后合master，不带Gallery实现 | AC-07 | [x] 交付tip 8b1e799完整门exit0（首跑一次瞬时音频抖动、单例与完整门复跑通过，见§9），identity1577/fmt通过，ff-only落地master |
| T-09 | merge | T-08 | SD/ledger/索引、本仓归档与实际delivery | AC-08 | [x] 规范23ed93e6与台账v152(d910b869)已ff-only进auto-lang/master，索引62项目无结构变化；本仓归档见§9 |
| T-10 | merge | T-09 | Git Bash wt-guard clean后清唯一树/分支/空组，保恢复材料 | AC-08 | [ ] |

## 9. 复审记录

2026-10-30 new r1：`stage: new | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: blocked | next: 等012实际delivery后用户启动T-01/T-02`。0/10，无实施/构建/图声验证；新profile/函数/字体导入/bundle仅为提案，旧音画路径不作公共能力通过证据。G12b归ag-examples/006。

2026-10-30 文档预检：本批五Plan各0..10节、10个T/8个AC/5个V和S、职责/SD/依赖入口完整，87个本地链接可解析；编号/计数一致、依赖无环。9份固定原始文件与Git对象逐字节一致。仅只读身份/文档检查，未解码/播放/生成bank-font/运行GPU；预计32和式/资源数不是通过证据，此结果非独立final review。

2026-10-30 work foundation checkpoint：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (checkpoint only; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: 97a5a57d8abc21d70e882893a2973fa1d3970240 | dependency_revisions: 012=78632af39de28881618f9219d07e20a9b7a5b56f,SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467,driver_lock=4a0b3040,AGP_lock=c3df168e | task_ids: T-01 closed,T-02 contract frozen/probe open | evidence: attachments/013/work/public-hud-contract.md SHA256 2931a24c013a7b58151e9df5cb9ce0a7c4b6bdae5633a4222901d8f4e98557c1 | blockers: none | next_responsible_role: work T-02 real two-dimensional implementation/probe`。`python experimental/autographics-auto-driver/tools/verify_plan012.py --controls` exit0，1536源hash及全部变异负控通过；012交付为基线祖先，两锁SHA与归档一致。全链授权已记§4。冻结scene8、hud2d/3d 113/114与profile24/25、完整152 scalar wire、固定font/4槽/共同接受/GPU clip合同；尚未实现这些选择，未运行GPU/PCM/32-33，T-02保持开放，1/10，不送review/merge、不发布Spec/ledger。主检出其他所有者文档WIP保留，期间015由其既有owner落地，不纳入本计划基线或提交。

2026-10-30 goal round1：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: 97a5a57d8abc21d70e882893a2973fa1d3970240 | next: work T-02`。本轮完成依赖核验、唯一树和可执行合同，属实质基础进展但不是work-complete verdict；0failed final review/0repair cycle，无用户待答。下一轮复读主Plan与树状态，实施并实测合同中的真实两维整链probe后才可关闭T-02。

2026-10-30 work two-dimensional probe checkpoint：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (T-02 substantive probe; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: e9d5dc814e4314808f6af86db68dd782cdfe3a36 | dependency_revisions: SDK/two locks unchanged,no dependency code writes | task_ids: T-02 real probe passed,old-profile/project closure open | evidence: D:/autostack/plan-artifacts/auto-ui/plan-013/t02-public-hud.log SHA256 ec8db9303e924e0c79874f87490713a7664929e66b3ff8bf017993a8b0ce0fed | blockers: none | next_responsible_role: work T-02 compatibility/closure then T-03`。固定font 26828B/SHA4444c236与OFL4391B/SHA31d45880逐字节核验；GitHub连接重置后由npm镜像获得相同font，许可证仅移除首行尾空格后精确匹配既定原件，不替换支持域。`cargo test --locked --offline --test public_hud -- --test-threads=1 --nocapture` exit0，5passed/0failed/0filtered：真实Auto两维完整32/33、动态0/1/2/3与Playing/Won/Failed、完整152wire、GPU ROI、2D黑边零变化、实际native PCM、暖atlas不重建及非法文本/GPU prepare失败完整last-good通过。初次夹具括号与版本白名单/跨维相机问题均修正后重跑，未计为通过。T-02旧17装配与项目闭包probe未收尾，T-03..10未开放；未发布Spec/ledger、未送review/merge。

2026-10-30 goal round2：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: e9d5dc814e4314808f6af86db68dd782cdfe3a36 | next: work T-02 compatibility/closure`。本轮从设计推进到真实两维图声/状态/预算整链，属实质进展但仍为work checkpoint，1/10，0failed final review/0repair cycle，无用户待答。下一轮先核当前树与旧profile/项目闭包，完成后才勾选T-02并进入T-03，不把本轮5case扩展解释为全AC或最终门。

2026-10-30 work T-02 closure：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (T-02 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: b29f2f8c69f6eefa55e808558730d147034f5e2d | dependency_revisions: SDK/two locks unchanged,no dependency code writes | task_ids: T-02 closed | evidence: attachments/013/work/public-hud-contract.md SHA256 85f8a37fd2c939a4fb7ee507920a056c63894b0cae3088b885605d54b863ddb1;t02-closure.log 9682d703e3951aada90cc2e9fcbd94d9b8dd4c345ebfeb7c7771a1990a442d95;t02-legacy-audio.log 86efb0070180d5a2e43c8982c9fb00a483d1dd8c0daac12af66e9937f85bd11f | blockers: none | next_responsible_role: work T-03 exhaustive resource/document admission`。`public_hud`6/6、compat2/2与旧public_audio10/host12/compat1均exit0、0filtered。df39593真实装配器仓外最小probe生成17份combined/API/bridge基线，当前全部逐字节相等；永久golden SHA d5bd7ab1582e96750e18997f53de3896e5fa1ad64f1f21c4a92fc92656eda4c9。scene8保存/损坏拒绝/修复/搬迁冷开通过，并修复probe发现的decode版本路由遗漏。合同明确仅纠正阶段所有者：详细resource/nonfinite归T-03，postaccept/DPI/Pause/20cycle归T-05，fresh identity归T-06，均未提前打勾。当前2/10，不送review/merge、不发布Spec/ledger。

2026-10-30 goal round3：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: b29f2f8c69f6eefa55e808558730d147034f5e2d | next: work T-03`。本轮完成真实旧装配和项目闭包，T-02按原Plan入口门关闭，属实质进展；0failed final review/0repair cycle，无用户待答。下一轮执行T-03固定font/slot文档、资源预算与存储正反例，不能把T-02可行性等同全部AC通过。

2026-10-30 work T-03 closure：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (T-03 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: 0230a0c26193c1eeacb3e340ce1a0bc7fd48d549 | dependency_revisions: SDK/two locks unchanged,no dependency code writes | task_ids: T-03 closed | evidence: D:/autostack/plan-artifacts/auto-ui/plan-013/t03-public-hud.log SHA256 b6b79e8171ae867682501a193ce4b37a7ef8e9b0dce2ecc3996ad82bb898964f | blockers: none | next_responsible_role: work T-04 wire mutations/replay/common accept`。`cargo test --locked --offline --test public_hud -- --test-threads=1 --nocapture` exit0，9passed/0failed/0filtered。scene8现内嵌精确OFL/RFN文本，保存/搬迁不依赖源字体目录；`import-font`真实公共编辑命令与场景升级可达，音频/模型编辑不再误降级。16项文档/许可/非有限/缺字负控、7身份263372raw bytes资源union、同字号atlas共享/第5key拒绝、全ASCII合法控制与坏字节零atlas副作用通过。新增unused_mut已清理，SDK146旧warning保持。当前3/10，T-04..10未提前完成。

2026-10-30 goal round4：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: 0230a0c26193c1eeacb3e340ce1a0bc7fd48d549 | next: work T-04`。本轮完成T-03资源与项目闭包，属实质进展；0failed final review/0repair cycle，无用户待答。下一轮针对T-04补完整wire变异、重复呈现/音效一次性和共同接受验证，不将资源门当最终AC。

2026-10-30 work T-04 closure：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (T-04 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: b77584bbf12454bd88dacfecf36d06bf3f54f54c | dependency_revisions: SDK/two locks unchanged,no dependency code writes | task_ids: T-04 closed | evidence: t04-public-hud.log SHA256 81d73df50b92d0e632392e608acdcaff7e81e710b56b978669f596a37a2c424d;t04-legacy-audio.log d79e03ea240a5d4dc9efc8300a0e3f848c9b2b3ed770efa62f9313bc6494792d | blockers: none | next_responsible_role: work T-05 lifecycle/device/DPI/owners`。`public_hud`11/11 exit0、0filtered；四槽共46个visible/length/type/ASCII/canonical/完整长度变异均拒绝，每次恢复合法控制通过。重复render/capture保持图像、2 voices和atlas builds不变；真实Auto/GPU/PCM/common accept及32/33证据保持。共享编辑代码最终状态旧public_audio10/host12/compat1全通过。当前4/10，T-05..10未提前完成。

2026-10-30 goal round5：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: b77584bbf12454bd88dacfecf36d06bf3f54f54c | next: work T-05`。本轮完成T-04共同接受与wire/replay验证，属实质进展；0failed final review/0repair cycle，无用户待答。下一轮执行T-05 viewport/DPI、设备故障、生命周期和20轮owner/warm，不能把共同接受门当全部交付。

2026-10-30 work T-05 closure：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass (T-05 checkpoint; work_complete:false) | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: 27641757812ca4e2922ec5294699bae388b9b714 | dependency_revisions: SDK/two locks unchanged,no dependency code writes | task_ids: T-05 closed | evidence: D:/autostack/plan-artifacts/auto-ui/plan-013/t05-public-hud.log SHA256 bbe3611882b2c2dc5433facf0151b59ceca7e2e345e228c92116669af8e6d034 | blockers: none | next_responsible_role: work T-06 fresh consumer/performance/identity/SD`。`public_hud`15/15 exit0、0filtered。固定文字glyph dst不随世界相机变化；2×物理resize不新建atlas，恢复尺寸图像逐字节hash一致，错设备代次拒绝。Pause不推进，Step一次HUD tick且suppressed_by_step无补播，Stop/fresh回默认0/Playing。postaccept Fault保HUD/已发计数、不自动重试，fresh重开stream。两维各20轮VM/voice/manager/PCM/atlas weak owner与健康复用/合法外部pin最后释放通过。当前5/10，T-06..10未提前完成。

2026-10-30 goal round6：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: 27641757812ca4e2922ec5294699bae388b9b714 | next: work T-06`。本轮完成T-05生命周期与设备边界，属实质进展；0failed final review/0repair cycle，无用户待答。下一轮完成fresh独立消费、性能/身份工具、规范增量与G12b接口摘要后才交work handoff。

2026-10-30 work final handoff：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | dependency_revisions: SDK7b987ca8/681/two locks unchanged,no dependency code writes | task_ids: T-06 closed,T01..06 complete | evidence: attachments/013/work/t06-readiness.md | blockers: no work-owned runtime/contract finding; independent review context required by Plan §6 | next_responsible_role: final review on frozen same tree`。AC01..05/V01..05工作侧全闭合映射见readiness。`check_public_hud.py --controls` exit0，51case/全负控通过：public_hud15、audio10、codec6、resources9、host12、compat2、consumer2、audio units5；stale/partial/wrong-runner/capture-only/actual-zero/PCM/native/warm/guard/resource均拒绝。完整源identity1576files SHA606f52b6f6b91b4fcb39cd655c0c591ab7520ef2166d5c67032faca0ffa44f64，verify controls通过。最终gate manifest SHA e9e6edda6ae8ee12d3b85588c3351e4be398eaa586288d1cd8f66f39fdd8ec2c，fresh manifest SHA a01203dd913411c0df00b1496b79005a2ae2f150c56165dab5ea4103f119076e；均本地路径见readiness。

source-only A→B在无SDK/target下独立准备681文件并locked/offline重编译、2tests/0filtered通过；源码改变cue/gain/motion/HUD，实际GPU/PCM与项目/许可搬迁冷开通过。最终fresh 2D/3D Host pump max8036/8028us均missed8ms目标，硬门全过且不拿2s替代；native48000Hz stereoF32实际nonzero11036、deadline_misses0，underrun/物理延迟未测。SD01/02/03与G12b接口提案在research/public-hud-spec-delta.md。当前6/10、execution_done；未做独立review、未发布canonical Spec/ledger、未merge归档。实现冻结，主检出其他所有者WIP保留。

2026-10-30 goal round7：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 7 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | next: independent final review`。本轮完成fresh消费、性能、完整身份与work handoff，属实质进展；0failed final review/0repair cycle，无用户待答。下一轮按T-07在同一冻结树独立核全部AC/V/SD与遗漏，merge-owned规范/台账/归档/清理保持pending。

2026-10-30 goal round8：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 8 | from: review | to: review | status: execution_done | outcome: waiting | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | next: user review-context authorization then review`。已复读执行状态、§6与冻结树；本轮仅确认T-07要求独立上下文且禁止新agent/chat，当前实现会话不能满足独立性。未运行阶段工作/测试、未启动agent/chat、未给final verdict、未合并，代码冻结不变。所需下一动作具体为：用户允许仅本计划的同树只读复审子代理，或安排既有独立上下文完成冻结cd72dc0的T-07复审。首次waiting round，0failed final review/0repair cycle。

2026-10-30 goal round9：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 9 | from: review | to: review | status: execution_done | outcome: waiting | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | next: user review-context authorization then review`。本轮仅复读§9/10，仍无用户关于同树只读子代理或既有独立上下文的答复。未运行阶段工作/测试、未启动agent/chat、未给final verdict、未合并，代码冻结不变。相同工作流阻塞连续第2轮（round8/9），不将自动续轮或时间经过视为同意，不降低AC-06。

2026-10-30 goal round10：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 10 | from: review | to: review | status: execution_done | outcome: blocked | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | next: user review-context authorization then review`。相同阻塞已连续round8/9/10三轮，仍无人工答复；缺少独立上下文授权无法满足AC-06，没有不依赖该答复的交付工作可继续。未运行阶段工作/测试、未启动agent/chat、未给final verdict、未合并，保留完整实现/证据/唯一树，host goal标为blocked。解除动作：用户明确允许仅本计划的同树只读复审子代理，或安排既有独立上下文完成冻结cd72dc0的T-07复审；不是因难度、预算或耗时阻塞，不宣称完成或降低验收。

2026-10-30 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent read-only subagent agent-0 | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | reviewed_commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | dependency_revisions: SDK/two locks unchanged | spec_inputs: canonical current specs and SD proposal,source identity606f52b6 | acceptance_results: AC01/02/05 pass;AC03 fail F-013-R1-01;AC03/04 fail F-013-R1-02;AC04 retained except clip;AC06 fail;SD01 evidence correction,SD02/03 pass as proposals | pending_closeout_gates: affected repair/review,frozen directed gate,canonical/ledger/archive/cleanup | findings: F-013-R1-01 state-text GPU identity/negative controls,F-013-R1-02 four-anchor clip/boundary controls | evidence: attachments/013/review-r1.md SHA256 3db0b4f2af92da59396cc87162244119b56493be10915c38120a32274c0b1a3e | next: work bounded repair then same independent affected re-review`。R1实际独立检查verify controls、冻结validate、fmt/diff和全AC/V/SD；无其它finding。两finding均有具体反例与最小重放范围，详见R1。

2026-10-30 goal round11：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 11 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94 | next: work F-013-R1-01/02 then same independent affected re-review`。用户已授权同树只读子代理，本轮取得改变下一动作的正式反例，属实质进展；第1次failed final review/第1个有界repair cycle。T04/T05及受影响的T06重开至3/10，T01..T03和未受影响证据保留，不合并、不降低AC-06。

2026-10-30 work R1 repair handoff：`stage: work | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | code_commit: 52f370a4af1a10d1af7ac4dff1e53aba105050d1 | dependency_revisions: SDK7b987ca8/681/two locks unchanged,no dependency code writes | task_ids: T04/T05/T06 repaired,T01..06 complete | evidence: attachments/013/work/r1-repair-closure.md SHA256 dc35aac92e3c32d03f13d6384020dc1f833040fa476fe7bccf07f9fba2e0573a | blockers: no work-owned finding; final independent affected R2 and frozen merge gate pending | next_responsible_role: same read-only reviewer agent-0 on frozen repair`。public_hud17/17、最终consumer2/2及非空三槽mask/data oracle、显式stale/wrong/missing/restore和四角clip/DPI负控通过；identity1577 SHA93cd9137、verify controls与fmt/diff通过。未改变验收/阈值/支持域/字体/预算，未跑root/full-AG，未发布canonical或merge。

2026-10-30 goal round12：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 12 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: 52f370a4af1a10d1af7ac4dff1e53aba105050d1 | next: independent affected R2`。第1个repair cycle完成并冻结，工作树干净；修复新增真实GPU逐槽身份和边界负控，不是仅重放旧例。下一轮只读复审当前commit，pass前不merge。

2026-10-30 independent final R2：`stage: review | review_scope: final affected re-review | reviewer_identity: independent read-only subagent agent-0 | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-013/auto-ui | branch: codex/plan-013-dev | reviewed_commit: 52f370a4af1a10d1af7ac4dff1e53aba105050d1 | base_commit: df39593b53412fed10055d18c0aab90b0590bd27 | dependency_revisions: SDK7b987ca8/681/two locks unchanged | spec_inputs: canonical current specs and SD proposal,source identity93cd9137 | acceptance_results: AC01..06 pass;SD01/02/03 pass as proposals | pending_closeout_gates: frozen directed gate,canonical/archive/cleanup,full AG milestone | findings: none;F-013-R1-01/02 closed | evidence: attachments/013/review-r2.md SHA256 a3f32b93d992acdbec177d8c2fddcd7afbfc0f851be9e800f28769866c2be8ec | next: merge`。R2独立17+2、24个当前状态distance0、empty/type/duplicate/wrong/stale/restore、source1577 controls及fmt/diff通过；日志r2-affected SHA3814d7a0、r2-data-oracle SHA47c84052。报告证据commit123d908e，不改变reviewed implementation52f370a。

2026-10-30 goal round13：`stage: goal | plan_id: PLAN-013 | plan_revision: r1 | round: 13 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-013/auto-ui | commit: 123d908e2016c371e13173d5dc51c24ff59a0b54 | next: merge frozen gate then landing/canonical/archive/cleanup`。同一独立reviewer的R2 final pass已绑定r1与52f370a，T07完成、7/10；第1个repair cycle闭合，无用户待答。下一阶段核当前master/范围与完整冻结门，未通过前不landing。

2026-10-30 merge prepared/landed：`stage: merge | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | completion_kind: delivered | prepared: verified | landed: verified | delivery_commit: 8b1e799aaa158c8ffed12f95173a859be590ceb6`。R2 pass绑定52f370a（tree c80aa130），两级rebase映射range-diff全等：df39593..52f370a ≡ 229f71b..308c3f3（goal会话同步master的008/015/016与设计文档，10补丁等价；tip ec2ef51相对308c3f3仅+review-r2.md文档），229f71b..ec2ef51 ≡ master02cd088..8b1e799（本次落地前同步三份AG设计文档，11补丁等价）。代码/测试/golden/identity与R2冻结内容逐补丁等价，SDK 7b987ca8与两锁不变。规范目标按§4以知识仓auto-lang为根（012/008/015同款先例）：SD-01 add public-hud.md、SD-02 modify scene-assets.md、SD-03 modify behavior-api.md；主检出其他所有者WIP（007/014/autoui设计、012 r1附件）保留未并入。

T-08冻结定向门在交付tip实测：首次完整门在scene8_font_closure保存搬迁用例报`audio_resource_limit: eight voices`瞬时失败（gate-exit:1，日志plan-artifacts/auto-ui/plan-013/t08-gate.log SHA 85782c55e79027b9de27b9313fd9f25cd9a737e1d6a815adb8cd70eb59533f67）；该用例单独串行复跑通过，且rebase区间229f71b..02cd088对编译内容零变更（仅docs），判定为本机音频设备瞬时抖动而非内容回归，按完整门原样重跑。完整重跑exit0通过全部suite（public/codec/resources/host/legacy/consumer/units）+负控+fresh独立A→B cold SDK/target offline consumer（t08-gate-rerun.log SHA 5008c5ce5275d861e218c2eaeb11d56a207d988b0f7bec1d7c0168a30239a882）；`verify_plan013.py --controls` exit0（1577 hashes+edit/delete/unlisted/pin/escape/PID负控）；AGP `cargo fmt --all -- --check`与20个driver改动rs `rustfmt --edition 2021 --check`通过；git diff --check仅OFL许可原文行尾空格（许可SHA 31d45880逐字节冻结，保留原文为授权要求）。随后主检出`git merge --ff-only`落地，master tip==delivery提交、无merge commit；落地后主检出verify_plan013 --controls复核exit0，其他所有者WIP保持未提交。

2026-10-30 canonical/ledger：`checkpoint: canonical_verified verified`。ui-013/auto-lang规范树（codex/ui-013-specs，先自547e4b55e ff至master 6daa6bfe）按R2通过的SD精确正文发布：规范提交23ed93e60399ab3f893903cd0a2486d17a6b17e1（public-hud.md新增含来源交付绑定，scene-assets/behavior-api增量，overview/plans/GOAL-008关联更新）ff-only进auto-lang/master；台账guarded offline projection v151→v152提交d910b8696734f71038e81e802dbae1885334dff5：新建P013-PUBLIC-HUD/P013-T/P013-R2/P013-M、复用P838-ASSETS/P840-BEHAVIOR-API追加，1488未触及item与其余顶层/段字段逐字节保持，v151备份SHA 4b8272e20012e4410a90392d1ad18cd33c53a5f0e7032dc5c06e0a83bdcb541f、结果SHA 98fd1ac5dcf5b8c0c84dbd88b99ced1c009cbd9594fec7f9f9f99f25f5f9af11，通道无store writer（8080无监听、19999返回空{}非store，同008先例）；`scripts/spec-index.py` 62项目/14既有未分组warning，INDEX内容零变化（仅CRLF重写，未提交）。台账P013-M/R2绑定归档前plan source_sha256 3fd6ba7ad7f8f1d734cef3708cbb5ec073ede45d0e30493be774732b135d51e0。

2026-10-30 archive：`checkpoint: archived | completion_kind: delivered | cleanup: pending`。本Plan移入docs/plans/archived/并置status: archived；10个T中T-01..09以实际收据关闭，T-10 cleanup显式pending不提前声明；归档前sha 3fd6ba7a与台账绑定一致；附件md链接按012先例加`../`前缀后从archived/父目录解析；docs/plans/README 013行更新为archived。批次回归到期末到（auto-lang回执covered 887/2026-10-29T15:53:56Z，<48h且无新增>887五整除已交付计划，同008裁定）；本期为实验引擎源码，生产backend/daemon/rootUI不消费，无重建重启部署要求。

### spec-sync 回写记录

- Canonical auto-lang: public-hud.md（SD01新增）、scene-assets.md（SD02）、behavior-api.md（SD03）；正文与独立R2通过的冻结提案精确比对，额外仅来源交付绑定。
- Module overview/plans和GOAL-008关联更新；GOAL仍进行中，无新模块/全局结构变更，spec-index62项目/14既有warning。
- 23ed93e6规范/d910b869台账已进入auto-lang/master；derived view不是第二份Spec，六项溯源及未受影响逐字节保护见前收据。
- Actual auto-ui code delivery 8b1e799；未触碰Gallery实现，后继016与ag-examples/006(G12b)必须绑定实际交付后单独验收。

## 10. 待澄清事项

当前无需用户决策。R2 final pass已完成，Plan为reviewed；merge需先通过当前冻结完整定向门，再landing、发布canonical SD、归档与guard清理。
