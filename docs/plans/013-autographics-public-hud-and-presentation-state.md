---
plan_id: PLAN-013
status: drafting
feature_name: AutoGraphics公共HUD与共同呈现状态
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
current_step: 0
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

用户2026-10-30授权公共声音/HUD后继规划、引擎与Gallery分仓、暂不实施。本轮只取号/Plan/设计/附件/引用；不建树/改源/生成字体或素材/编译/GPU/播放/agent/发布。当前012是新draft，006仍在途；不能消费其候选consumer。

共同有限协议、资源帽与声学不可回滚边界见[设计](../design/autographics/m2-public-audio-and-hud.md)，素材/代码观察见[基线](attachments/012/planning/service-baseline.md)。知识权威在auto-lang：[overview](../../../auto-lang/docs/specs/auto-graphics/overview.md)、[旧专用HUD](../../../auto-lang/docs/specs/auto-graphics/auto-3d-game-profile.md)、[资产](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)、[原游戏分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。本Plan SD以知识仓为根，本轮不发布canonical新能力。

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
| T-01 | work | 用户启动、012实际delivery | 核公开合同/SDK/锁/identity/owner，建唯一ui-013树 | AC-01/05 | [ ] |
| T-02 | work | T-01 | public-hud-contract先冻结font/slots/viewport/完整bundle/PCM图像oracle，两维整链先证 | AC-01..05 | [ ] |
| T-03 | work | T-02通过 | 固定font/slot文档与有界glyph/存储/预算V-01 | AC-01/02 | [ ] |
| T-04 | work | T-03 | Auto/Host/GPU overlay及audio+HUD+visual共同接受、32/33、V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | viewport/生命周期/故障/20轮owner/warm，V-03/04 | AC-04 | [ ] |
| T-06 | work | T-05 | fresh两维消费/预算/兼容/identity、fmt/提交、SD与G12b接口摘要V-05 | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立final review，merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门与复用审计后合master，不带Gallery实现 | AC-07 | [ ] |
| T-09 | merge | T-08 | SD/ledger/索引、本仓归档与实际delivery | AC-08 | [ ] |
| T-10 | merge | T-09 | Git Bash wt-guard clean后清唯一树/分支/空组，保恢复材料 | AC-08 | [ ] |

## 9. 复审记录

2026-10-30 new r1：`stage: new | plan_id: auto-ui/PLAN-013 | plan_revision: 1 | outcome: blocked | next: 等012实际delivery后用户启动T-01/T-02`。0/10，无实施/构建/图声验证；新profile/函数/字体导入/bundle仅为提案，旧音画路径不作公共能力通过证据。G12b归ag-examples/006。

2026-10-30 文档预检：本批五Plan各0..10节、10个T/8个AC/5个V和S、职责/SD/依赖入口完整，87个本地链接可解析；编号/计数一致、依赖无环。9份固定原始文件与Git对象逐字节一致。仅只读身份/文档检查，未解码/播放/生成bank-font/运行GPU；预计32和式/资源数不是通过证据，此结果非独立final review。

## 10. 待澄清事项

无新增方向问题。012尚未实施，本Plan不预填delivery/selector/schema；T-02确认完整有限包/字体/atlas和32credit可行，失败回new。固定font/ASCII/overlay是明确支持域，不通过静默缺字/系统font/额外资源额度扩大；未来CJK/生产UI/深度UI或完整游戏另立合同。
