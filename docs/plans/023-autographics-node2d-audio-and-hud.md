---
plan_id: PLAN-023
status: executing
feature_name: AutoGraphics二维Node声音与公共HUD作者能力
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 3
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-013]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-audio-and-hud.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-023：二维Node声音、HUD与有限呈现接线

## 0. 变更摘要

为[AG006 r2](../../../ag-examples/docs/plans/006-autographics-gallery-event-hud.md)补齐普通use、node/model/view-2d/on/Life.Tick下的具名音频/cue、固定字体/HUD槽、局部事件与一次呈现收口；复用已交付012/013底层audio2d/hud2d，不重做声音或字形系统。硬等020实际交付，固定消费它的组件合同，不追开发tip。

当前仅规划drafting r1、0/10，不建树、不写产品源码/夹具、不构建或运行GPU/音频/agent。[能力审计与分仓设计](../design/autographics/node-audio-hud-and-gallery-followups.md)是规划输入；其语法示意不是已实现接口。

## 1. 目标

1. 用可复用Node局部model/msg/on表达Player走跳、物品一次事件与HUD；普通use导入资源/组件/纯函数，作者不写整数资源ID或app_start/app_update。
2. 具名bank/cue/Font/HudSlot与必要ASCII表达式、输入快照/类型化触发路由真实接到既有Auto→Host→GPU/PCM接受链。
3. 世界内多个组件可以产生声音请求和HUD状态，但最后只形成一次有界Auto呈现调用，保原费用、精确整数、生命周期、owner及故障语义。
4. 向AG006提供实际版本、公共准备/来源记录和离线消费合同；旧016/017/019及012/013原模式保持。

非目标：Gallery项目实现、G05查询、G09精灵作者能力、PNG或可选sprite呈现写入、3D作者语法、音乐/空间音频/任意codec/任意字体/CJK/生产AutoUI、通用事件总线/动态Node子树、多呈现/Life.Frame、ACE/AOT/ECS/CPC、抬预算或修改外仓/固定SDK。

## 2. 架构方案

普通Auto模块闭包→020共享组件/receiver表示＋本计划typed资源/服务适配→生成Auto局部状态与回调末有限呈现函数→原scene7/audio2d或scene8/hud2d assembly→公共Play/Host/GPU/原mixer/native stream。Rust只做编译/资源准备/调度与服务校验，不决定计分、胜败、cue、gain或文字。

新模式显式选择，不覆写旧CLI。无精灵的呈现沿既有ag_present2d(kind=-1)；当前G12 audio的sprite全为null、只需程序化图元，故不硬依赖022。未来实际需要sprite写入须另明确消费022，不在此复制图像/帧实现。

拟唯一实施/顺序复审树D:/autostack/.wt/ui-023/auto-ui、codex/plan-023-dev；同组只读固定依赖，主检出只簿记，禁止链接，清理先fresh guard。

## 3. 技术栈

已存在pilot src/{load,translate,lower_basic2d,lower_g11,main}.rs和dialect node2d；driver api/public_{audio,audio2d,hud,hud2d}_v1.at、authoring/public_{audio,hud}/、runtime_objects/runtime_round/behavior/play、samples/public_hud与public_audio。020共享组件表示待实际交付冻结。

拟新增pilot声音/HUD资源与service lowering模块、持久fixtures/audio_hud2d/、tests/node_audio_hud_gates.rs、driver tests/node_audio_hud_flow.rs、tools/verify_plan023.py；具体文件/CLI/typed拼写由T-02冻结。永久回归夹具归产品；临时探针只在独有scope。无Gallery/SDK写面。

## 4. 需求分析与背景调查

用户2026-11-01授权修订AG006为新作者路线，核声音/HUD Node缺口、必要时另立引擎前置，并建立下一轮内容计划；明确暂不实施。本Plan只认领引擎，AG006保原内容分母，AG009/010分别消费021/022，无反向依赖。

只读基线auto-ui695ce0b（013 delivery8b1e799aaa158c8ffed12f95173a859be590ceb6、019 c93adb7）；020已存在ui-020权威树与T-01记录、未交付，不接管其修改。UI014亦有自身树，按真实共享写面协调，不凭标题加串行前置。

权威：本仓[019基础](../specs/auto-graphics/node2d-basic-authoring.md)、[消费](../specs/auto-graphics/node-authoring-consumption.md)、[020合同](020-autographics-stateful-node-components.md)；旧引擎规则在[public-audio](../../../auto-lang/docs/specs/auto-graphics/public-audio.md)和[public-hud](../../../auto-lang/docs/specs/auto-graphics/public-hud.md)，分别SHA256 5a802e532a63bd2ad1d9999d3ce8cf6acc8f054280c6f0c7bb47cf514eebacb5、4e787ed23872e682f79069e0d6bdcd00f364ab4e5c3cb58dade0c0df1fb2f6a9。

实际缺口：load::ResourceKind只有Quad/Circle/Color；CLI只有legacy/g11-2d/scene-2d/character-2d/camera-2d；translate无声音/HUD/字符串接线；020非目标明确排除声音/HUD。因此底层交付不等于Node已支持。load.rs SHA4837d29c0cfe863dc9db307b1ff160d05b84bc0213fe4c3278166d267405d3d9，translate.rs SHA894dd1473916177e9c07f23d7a95831352a07eee710cf025a001a239138ceddb，public_hud2d_v1.at SHA54f297a400f0d254bc8699f62413917515a486cee3f3691d8be72eaa0a505982。

预算风险已具名：旧G12a源注释的32和式漏列event_pair；其32/33原oracle使用独立grounded循环，不能证明所有新组件组合最坏分支。新链必须含自动触发分派/绑定/字符串/收口全部成本实测，不延用注释当go；不由本Plan修历史内容。

## 5. 详细设计

### 5.1 硬门与有界先证

T-01核020 final独立pass/冻结门/master真实delivery/Spec/清理及013、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467的681内容锁、两依赖图/锁与实际owner。历史对象不可达用受检固定副本，不换SDK tip。

T-02最多两种既有机制接法：一个局部Player＋三个可销毁Coin＋终点/失败条件＋四槽HUD＋jump/coin/land，先走一次真实最坏收集分支与音频单独入口。冻结§9准确语法、资源/事件/字符串表示、调度/CLI/API、费用、GPU字形ROI/独立PCM oracle及完整V/S命令。不能在原额内闭合则needs_replan，禁止调32、隐藏handler费用、Rust玩法或删原内容。可行性不是当前已执行证据。

### 5.2 声明、组件与服务

- 复用020同定义实例隔离/局部字段/typed事件、原≤8有状态Node包络和世界单Tick。角色自己的vy/grounded/输入规则不搬回根；Coin拥有已收集状态，根只协调计数/结果，HUD组件读取typed属性并持本地呈现状态。
- typed AudioBank/Cue/Font/HudSlot为候选端口种类：资源由普通use共享，cue引用同一bank的半开帧区间和gain；HUD以具名槽定义anchor/offset/size/rgba/初始文字。编译器映射ID/slot，作者不能依赖Run ordinal。全部资源与许可进入输入闭包，非任意文件直通/网络URI。
- 最小ASCII字符串支持包括字面量、Auto分支/局部变量与函数返回、计数0..3所需有限格式表达（可用普通Auto helper选择文字，不必须新增插值语法）。保模块和实例作用域、类型/长度/字符域；不把计分或文字选择写进Rust。精确表达式支持面T-02冻结，不新增另一表达式parser。
- 必要输入快照一次消耗→明确typed只读值供多个合法组件使用；请求谁读、顺序/费用公开，不重复免费采样。复用原triggers2d的named实例/shape/主体/有限spawn/despawn和N→N+1路由；event_next/pair/phase等实际收费，不照搬017自动多读三事件后遗漏账。只读一事件的收集/销毁分支与需phase但不销毁的goal分支分别证明。
- cue业务请求和HUD状态由Auto节点产生，回调末生成的有限Auto收口读取最终局部状态，≤4个请求、四槽完整状态、一次ag_present2d(kind=-1)，不在每Node各发bundle。可显式standalone模式验证旧服务，但同callback混用/重复bundle必须按原规则拒绝。收口不是Rust回看score推断声音，也不是新全局游戏事务策略。
- world内ordinal为精确int，Run内1..2147483646严格递增，多个组件确定顺序共用序列、不各自从1发号；生成Auto协调，typed Run/slot/generation不经float往返。耗尽/旧代/重复ordinal/第五请求/第九voice受控拒绝，不丢音、抢占或积压无限队列。

### 5.3 固定资源、发布与寿命

音频沿012：WAV PCM16或有限Ogg/Vorbis、mono/stereo、22050/44100/48000Hz、单raw≤8MiB/≤10s/decoded≤16MiB，container/span/预测先于分配并复用现有decoder。G12 bank b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d，90060B/44100Hz stereo/22504frames；jump[0,7403)、coin[7403,20046)、land[20046,22504)，CC0来源保持。共用PCM不按cue复制。

字体仅Lilita One 26828B，SHA4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa；OFL/RFN4391B，SHA31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b。恰四槽，每槽0..32可见ASCII32..126、总≤128；hidden/empty不同，四槽完整状态，未占槽显式默认隐藏。错字/未知字形/越限具名拒绝，不替换空格。

原8资源/32MiB raw、64MiB CPU/GPU、8磁盘文件32MiB与残留/预留规则不变；四atlas key(font,size16..48,device)、单边1024/4MiB，warm文字更新不重栅格/传图。HUD画游戏viewport内最后overlay，四anchor/letterbox/DPI/resize/clip独立world camera，不声称可采样深度UI。

preaccept失败保完整last-good visual/HUD、不插新voice；postaccept设备故障具名audio Fault、停Run voice，已出PCM不可撤回、不重播。Pause无候选，neutral Step一次visual/HUD并suppressed_by_step消费ordinal，Resume不补播；Stop恢复文档默认，fresh Run重建model/计数/ordinal/实例，旧Run消息与声音拒绝。20短轮与合法外部资源pin最后owner分开观察。

### 5.4 计量、消费与保持

共享32/callback、char move26、snapshot1、grounded/event getter/despawn各原价、bundle1/HUD set1/emit1不变。候选收集分支1snapshot+26move+1grounded+1next+1pair+1despawn+1bundle=32；goal需要phase但不despawn时亦单列。此是先证约束，不保证隐藏分派/初始化或并发事件天然不超额；不成立回规划。原VM1000steps/2ms分片、100000steps/120resumes/250ms累计、init30s/tick2s及源码/managed/128场景节点包络保持，完整Tick/字符串/收口与异步mixer CPU分列，8ms目标非硬保证。

公开准备生成scene7/8、Auto、模块/资源/许可closure、来源角色/槽映射及实际generator/runner/Cargo锁/toolchain记录；A→B冷再生/真Run/SaveAs重开/坏件拒绝→精确恢复。显式source-build模式，不篡改012/013/017/019 exact锁或自动fallback；旧API字节/价格按受影响路径验证。023不等待021/022或任何Gallery完成；020交付后与021/022可按独立文件并行，shared AST/load/translate/CLI只能owner短窗顺序接线，重型门单实例。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-audio-and-hud.md | 底层012/013与Node入口分离→有限typed资源/局部事件/四槽/一次收口与公开消费，保原额 | AG006有可固定的实际引擎依赖 | AC-01..06 |

## 6. 测试设计

拟pilot --test node_audio_hud_gates、driver --test node_audio_hud_flow -- --test-threads=1、verify_plan023.py --controls；T-02冻结非零完整case集合/命令与超时，当前未运行。旧public_hud/audio/resource/compat及020/019/017只选真实修改触面，OS键鼠/焦点自动化继续暂停，GPU/音频门单实例。

| V/S | AC / owner_stage / T | 输入→真实路径与边界 | 区分oracle/关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 普通use的bank/cue/font/slot与本地字符串→binder/准备/资源消费；错类型/缺许可/越界span/坏WAV/TTF | 原素材合法、错误在对应准入阶段具名拒绝；无decoder复制/任意hash信任，counter/源定位可核 |
| V-02/S-02 | AC-03 work T-04 | Player/Coin/父计数/HUD组件→真实Auto→接受Frame/新鲜GPU | 独立输入轨迹与0/1/2/3、Playing/Won/Failed字形/ROI相符；错receiver/旧capture/错字与相机带动HUD负控红 |
| V-03/S-03 | AC-03/05 work T-04/05 | 两组件各发可辨cue、同tick双cue/gain、四完整槽与坏/重复bundle | 独立PCM16直读混音对照≤2e-5、ordinal全序；同tick一次接受，混写/重复/容量越界拒绝、不部分发音 |
| V-04/S-04 | AC-04 work T-05 | Pause/Step/Stop/fresh/旧Run、preaccept坏件/postaccept设备Fault、20轮及外部pin | 无重复Start/旧声/补播；最后owner正确；真实native stream非零帧，不把manager=true或零PCM当成功，设备不可用保持gap |
| V-05/S-05 | AC-01/05 work T-02/05 | 完整最坏收集/goal/双cue/HUD绑定、32/33、8状态实例/资源边界 | 全生成Auto成本实测且语义完整；独立循环32/33不能替代业务分支；VM/异步mixer耗时与warm0重解码/上传另报 |
| V-06/S-06 | AC-06 work T-06 | 固定工具source-build A→B、冷closure/保存重开、坏source/font/bank/license/runner/记录 | 真实Auto/GPU/PCM与恢复再Run，报告绑定实际exe；旧模式按真实触面保持，不写Gallery或借旧绿 |

work自查→同一权威树独立上下文final review→merge冻结门，模型/人员未指定，不创建或联系agent/chat。全AG属里程碑pending，不跑语言根测试。所有临时consumer/SDK/target/PCM/PNG/log仅D:/autostack/plan-artifacts/plan-023/auto-ui/，开工登记owner/用途/在途；禁止attachments含md。摘要折Plan§9；merge对账全部实际输出目录并删自有scope/构建缓存，仅具名未解决问题按owner/理由/关闭条件登记持有。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 020/013真实依赖与T-02全链预算go，不能根/Rust代业务或默提额 |
| AC-02 | work | typed资源/槽、局部ASCII表达式/普通use与严格定位准入，V-01 |
| AC-03 | work | 局部组件事件/计数/真实字形/一次bundle与独立PCM正确，V-02/03 |
| AC-04 | work | 生命周期/旧Run/容量故障/20轮owner及真实native非零PCM，V-04 |
| AC-05 | work | 原费用/CPU/资源与完整最坏分支、warm和负控，V-03/05 |
| AC-06 | work | 公共准备/实际身份、离线A→B/保存重开/坏件恢复及旧模式兼容，V-06 |
| AC-07 | review | 独立逐AC/V/S/SD/原额/遗漏/格式/warnings复审，绑定最终提交，零未处置finding |
| AC-08 | merge | 冻结定向门/有据复用后实际合master、Spec/来源/归档导航准确 |
| AC-09 | merge | fresh guard/clean/祖先核后清树分支空组及所有材料，登记/10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续启动、020实际交付 | 核020/013/SDK/两图依赖/owner，建唯一ui-023树、冻结源与资源 | AC-01 | [x] 2026-11-01：授权=用户/goal续指令"先交付auto-ui/PLAN-023（work→review→merge链）后回到AG006 T-01"。020交付核验：archived 10/10、master交付16ddfa0在基9e347d3祖先链（merge-base实证）、canonical spec stateful-node-components.md sha256 5dd9e372@HEAD逐字节相符；013 archived（8b1e799）。owner核：.wt仅edit-029/lang-904/os-052，021/022仍drafting无树无分支→shared AST/load/translate/CLI窗口归023；ui-020组已清。ui-023组建成：auto-ui@9e347d3分支codex/plan-023-dev（无链接、状态clean）+只读依赖auto-lang@add1df088/auto-web@ec9c4ba（detached）；组内相对路径解析实测（pilot Cargo.lock重绑+1行=a2r-std getrandom，与主检出同一漂移，提交cf87440）。SDK：driver .sdk-source由tracked sdk/frozen prepare=681文件PASS（frozen tree 18c8f5eb自4adf2d03未变；source-lock lineage标签7b987ca8按specs）。源冻结：load.rs 4837d29c=§4、public_hud2d_v1.at 54f297a4=§4、lang specs public-audio 5a802e53/public-hud 4e787ed2=§4；translate.rs §4值894dd147→实测8210f297=020交付的call_scope/child_emit钩子（预期依赖交付漂移，按基线重冻）；新增观察public_audio2d_v1.at dab843e7。基线绿：pilot gates 9 basic+10 g11+9 lower+6 components全过（59s check）。主检出观察：搭建期间并发docs提交88c96ba（registry修复，docs-only不在依赖图）；根Cargo.lock 1行+docs WIP=他会话（020 merge收据载明保留不接管），零代码WIP。scratch登记D:/autostack/plan-artifacts/plan-023/auto-ui/README.md | AC-01 |
| T-02 | work | T-01 | 最多两接法完整Player/三Coin/HUD/PCM先证，冻结§9语法/顺序/计量/命令 | AC-01..06 | [x] 2026-11-01：GO（两半均真实编译/执行，协调者亲跑复验）。**gameplay半**：pilot components-2d降层→手扩world.at→Hud2V1准入→driver CPU会话，闭在原32计费内；walk tick恰30（budget-oracle green/+2/+3→fault10三验）、最坏收集tick恰32（+1恰t=85 fault10）、双cue单bundle=32（ordinal严格递增）、goal(phase不despawn)=32、hazard=31；naive不门控=34→fault10（idle-turn门为闭合关键）。采纳Wiring 1；closeout=每回调恰一次ag_present2d(kind=-1,≤2 cue+4槽)整体1计费（_ag_hud_bundle_begin 0+_ag_audio_begin 1+puts 0=§4 API实测一致）；字符串=分支选择字面量helper fn（裸参数RHS赋值被拒，if形替代）；goal/hazard须运行时实例非静态叶。10/10（16.6s亲验）。**audio半**：bank b0b740dc独立RIFF解析=90060B/PCM16/stereo/44100Hz/22504帧、三cue字节切片[44,29656)/[29656,80228)/[80228,90060)零缝隙；真实服务路径(load_project→resolve_cues→Mixer)jump/coin/land gain1.0对bank切片**逐位相同（delta 0.000e0，0 LSB）**、三voice混合亦0；gain 0.5/1.0实测out=slice·request_gain·cue_gain（0.5 vs 半幅=0）；同tick重复cue=两独立voice（2×切片逐位exact，Accepted::Replay非合并）；具名拒绝52 ordinal/53 cue/54 gain(NaN,±INF,-0.1,1.01)/51 packet全观测，拒后无副作用（delta 0）；authoring层52/53/54/51数字码经delivered测试三绿复证。native：真实默认输出设备开流成功（ToDesk Virtual Audio虚拟设备，44100/2ch/F32）970/970帧非零、ring对jump切片head offset 0内1e-6、callback warmup 849µs/steady 104-108µs/0 miss——**虚拟设备警告**：提交流PCM已观测，声学维度留T-05具名gap。5/5（1.5s亲验）。冻结=F-1..F-9+audio事实（cue/bank配置在scene.agscene assets[].audio+顶层cues数组、cue id=数组下标、cue名不入mixer::Cue仅场景层、profile `components-2d-game`/API autographics.hud2d/1）；V-01..06 oracle映射29案例落§9。两探针均未跟踪未提交，零tracked改动。scratch t02/{gameplay,audio}/ | AC-01..06 |
| T-03 | work | T-02 go | 共用IR新增typed bank/cue/font/slot、ASCII与公开准备，V-01 | AC-02 | [x] 2026-11-01：work四提交（rebase后sha：7914204/b3610ca/9995a76/0d61afd+followup 4175ce9，详见§9对账记录）。Authoring语法：根node内game2d块——audio{bank,font,license}（入口相对路径，RIFF/PCM16校验）、cue <name>{span:(s,e) gain}（id=声明序=scene cues下标）、hud{slot 0..3{text:<field>? init:?}, text <field>{int:\"str\"… default}}→hud_<field>_text字面量helper、trigger <name>{shape circle r|box, target, event, on: collect|fail|goal, guard?, cue/full_at/full_cue, value, when_count, phase}；F-4所有权强制：作者必须声明count/state（缺者具名拒绝neg-missing-count/state），gate/ord/np/p0/p1生成器注入（作者声明同名→具名拒绝neg-infra-field）；全部诊断module/line/symbol形+负夹具（neg-vis-cache/neg-region/neg-nonroot/neg-cue-unknown）。生成world sha钉ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0（6556B；与T-02手扩world行为等价+六美容delta文档化：根标量/getter序、无需强制r_state==9发射点（F-8.2生效）、pair=表序升序派发、hazard j=1纠正手写j=0、注释头、helper参数n）；两次再生逐字节等值。旧profile冻结验证：generated-components2d七产品字节等值、probe sha 9241d013不变；pilot gates 9+10+9+3+6绿+021 queries 7绿（协调者亲跑）；driver node_components_flow 9/9+t03_game_gates 4/4亲跑（V-01字节钉+3负控；V-02 Hud2V1准入green含5 binds、envelope 7/8有状态+5/8对象槽、9th slot越池fault31——F-9原文\"33rd/5/8\"按真实池容修正，实测数字为准）。合同冲突三处文档化未静默偏离（visq改名限game profile保冻结字节/池容修正/7-8修正）。021并发落地已rebase对账（§9）；ui-021树先核零src改动。scratch t03/。遗留T-04..06：CPU通关/GPU字形/PCM/生命周期/预算/收口纪律/verify_plan023 | AC-02 |
| T-04 | work | T-03 | 局部事件/输入/typed触发与一次Auto收口、字形/PCM，V-02/03 | AC-03 | [ ] |
| T-05 | work | T-04 | 真实最坏预算/故障/生命周期/native/owner/必要旧兼容，V-04/05 | AC-04/05 | [ ] |
| T-06 | work | T-05 | source-build A→B/保存重开/坏件恢复、SD与handoff折Plan，V-06 | AC-01..06 | [ ] |
| T-07 | review | T-06 | 同树独立final逐项复审，finding回work，pass后merge门pending | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结实际提交/依赖/输入，定向门或有据复用后合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 沉淀新Spec/来源、归档导航与证据摘要，不写派生JSON | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard清树分支空组、路径全对账/删scope/登记与最终计数 | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-023 / plan_revision: 1 / outcome: blocked（实施硬等020实际交付；不是设计或预算验证已通过） / next: 后续用户启动后T-01/T-02。仅规划，所有新增源码/夹具/命令未创建或运行，0/10；AG006依赖此真实交付，不把013底层交付误记为Node支持。

2026-11-01规划预检：四合同0..10节、各10任务及work/review/merge归属、006保8AC/其它各9AC、各6组V-S/1SD与drafting0/10通过；新增正文/共同设计32个本地链接可解析，本仓编号独占分配后复扫唯一（UI下一024、AG下一011），依赖无环。G09原18名与G12音频原8名逐字各保留一次，G05原九require族保留；均是静态合同检查，不是新实现/可行性go/独立终审。本轮未创建实施树或提案Spec、未运行新增产品验证。

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-023 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing（并发会话T-01已关） | outcome: blocked（并发冲突让位） | worktree: D:/autostack/.wt/ui-023/auto-ui（另一会话所有） | commit: cf87440（他会话T-01 lock rebind） | next: user。独立验证020交付前置全部成立（archived 10/10、16ddfa0在master祖先、013 8b1e799祖先、SDK 7b987ca8=681文件锁、font/license/bank-audit与load.rs/public_hud2d_v1.at四SHA逐字节=§4冻结、translate.rs漂移8210f297=020钩子交付预期）。随后发现另一授权会话正同树驱动本Plan（其T-01收据引用户/goal续指令，tree/SDK/gates/收据皆其在管），本会话即停写让位，未覆盖其任何记录；ui-023组内auto-lang@add1df088为其显式冻结（主检出另一ui-021会话树亦在飞，注意共享面owner短窗）。重启本goal前须确认唯一driver。

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-023 | plan_revision: r1 | round: 2 | from: user | to: work | status: executing | outcome: pass（唯一driver确认，接管待静默验证） | worktree: D:/autostack/.wt/ui-023/auto-ui | commit: cf87440 | next: work(T-02)。用户决议"确认唯一driver"落§10 D-05：本会话为唯一driver，从T-02恢复；原并发会话（12:14:43写T-01收据后处于轮间）读到D-05即停止写入。接管前观察窗：若Plan/树/分支出现任何新写入即中止恢复回报user；静默则核对T-01成果并开始T-02（两接法先证冻结§9合同）。

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-023 | plan_revision: r1 | round: 3 | from: work | to: wait | status: executing（他会话T-02在飞） | outcome: waiting（分支一生效：等待原会话完成T-02..T-10后核对归档） | worktree: D:/autostack/.wt/ui-023/auto-ui（他会话所有） | commit: cf87440 | next: wait(monitor)。接管尝试按用户分支一判定回退：12:39:04他会话已产出T-02探针（树内未跟踪tests/t02_probe.rs+scratch t02/gameplay/经pilot生成worlds），证明其长work轮仍在飞、12:16的D-05停止指令在其轮界前不可达；本会话无从停止其goal（无进程/relay手段），并行写树将双写损坏。本会话已：删自备探针三文件（scratch t02/根，未触碰其gameplay/）、改D-05为分支一事实化（他=唯一driver继续，本会话只等待+归档核对，并告知他忽略12:16停止指令）。后续轮：仅监控Plan mtime/树/分支推进，Plan状态到archived后按Done标准核对（merge pass、Spec落地、归档、清树、主检出净）；若长时间无推进则携精确状态回报user。

2026-11-01（唯一driver续记，读到D-05分支一，按其末句"忽略12:16停止指令，按原授权继续"接管唯一driver角色；监视会话的round1..3三条收据原样保留）。T-02 gameplay半GO：真实编译执行链（pilot components-2d降层→手扩world.at→Hud2V1准入→driver CPU会话）闭在原32计费内——walk tick恰30（budget-oracle三验：green/+2 green/+3 fault10）、最坏收集tick恰32（+1变体恰在t=85 fault10钉界）、双cue单bundle=32（ordinal严格递增52负控）、goal分支(phase不despawn)=32、hazard=31、pad-stay=32；naive不门控held=34→fault10（idle-turn门是闭合关键，冻结为F-6）。采纳Wiring 1（两接法限额内）：手扩生成world.at按Hud2V1准入（assembly供triggers+audio+hud单元），拟新profile `components-2d-game`/API autographics.hud2d/1；closeout=每回调恰一次ag_present2d(kind=-1, ≤2 game cue+4槽完整状态)整体1计费（_ag_hud_bundle_begin 0+_ag_audio_begin 1+puts 0，实测与§4 API原文一致）；ordinal=根单计数器r_ord全组件共享。字符串=分支选择字面量helper fn返回str（实测接受；裸参数RHS赋值.halt=n被拒，if形替代）。区域规则F-6：goal/hazard触发区必须是运行时实例（Zone组件）非静态叶——角色sweep视每叶为solid实测不可重叠。复现三生成器缺陷（T-03必修，均未修改树文件）：①model字段名vis与present缓存重名→重复var <prefix>_vis admission致命；②无emit点的msg handler被丢弃→须为全部声明handler发receiver fn；③文档：CPU会话spatial box=平移相对半边长。独立复验：协调者亲跑tests/t02_probe.rs 10/10（16.6s）；树内零tracked改动零提交（唯一??=t02_probe.rs），lock rebind cf87440不变。材料：scratch t02/gameplay/（commands.log/measurements.md/freeze-draft.md F-1..F-9/world-src/gen world sha 9a6cbd84）。audio半（bank身份/真实PCM≤2e-5/gain变体/具名拒绝52-54/native设备观察）另一探针在飞，落定后T-02整行收口。共享面短窗观察：主检出.ui-021会话树已在飞（.wt/ui-021），021=独立文件并行；T-03起触碰shared load/translate/CLI前核对其owner窗口，merge按rebase顺序协调。

2026-11-01 T-02整行收口（audio半GO，协调者亲跑5/5=1.5s复验；见§8 T-02行实测数字）。**§9冻结合同**（F-1..F-9+audio，全文 scratch t02/gameplay/freeze-draft.md，以下为约束性摘要）：F-1 profile `node2d-lower --profile components-2d-game`（不覆写components-2d冻结字节）、driver准入Api::Hud2V1（autographics.hud2d/1，assembly沿audio模式衍生链供给triggers+audio+hud单元，profile 24/v113）；F-2 降层新增：按authoring触发表发ag_trigger_bind（对序=表序、主体恒为角色实例、shape/radius来自authoring入口）、为全部声明msg handler发receiver fn（现状只发有emit点者）、app_update框架=game_events() poll前奏+根lifetick+声明先序实例tick/present+game_closeout()收尾；F-3 closeout骨架冻结（每回调恰一次ag_present2d(kind=-1, r_np, o/c/g×4, v/t×4)，整体1计费，携带≤2 game cue+四槽完整状态；kind 0/1保留可用）；F-4 根合同字段gate/count/state/ord/np/p0/p1（r_前缀+get_r_*()观测getter）；F-5 字符串=字面量+分支helper fn返回str+f"$expr"插值（legacy同款）；裸参数RHS赋值拒绝不变；F-6 计费组织（32合同）：pressed 1+axis 2、每回调恰一次ag_event_next（首读）+事件tick才pair、goal分支才phase、latch tick经player.Idle(1)选择器调用的idle-turn门跳过两个held读、closeout每回调1；最坏tick全=32（walk 30+poll+bundle=32、collection 28+next+pair+despawn+bundle=32、goal 28+next+pair+phase+bundle=32）；region规则：触发区=运行时实例；F-7 计费表冻结（§8行）；F-8 三生成器缺陷=T-03必修（vis重名/无emit点handler丢弃/spatial box文档）；F-9 V-01..06 oracle=29案例（V-01降层语义1+3负控、V-02 world合同3、V-03 CPU通关6、V-04宿主GPU4、V-05计费5、V-06收口纪律7），命令=cargo test --locked --offline --test t0N_*单实例。**audio冻结事实**：cue/bank配置权威在scene.agscene（assets[].audio元数据+顶层cues数组，cue id=数组下标）；cue名不入mixer::Cue（仅场景层），组件profile以场景声明为真源；PCM capture oracle=load_project→resolve_cues→Mixer渲染对照bank切片≤2e-5（T-02实测0.000e0）；gain语义out=slice·request_gain·cue_gain；重复cue=独立voice不合并；native提交流已观测（虚拟设备ToDesk Virtual Audio），声学维度留T-05具名gap（D-04部分闭合）。D-02闭合（语义层）：ASCII=字面量/局部/返回/f插值、快照=沿frozen pressed/axis（snapshot变体需新语法出scope）、typed触发语义冻结（authoring精确拼写=T-03在F-2界限内的普通实现选择）、收口拼写/费用实测冻结。D-03闭合：完整账=§8 T-02实测表（每tick形状逐列），旧G12注释32/33独立循环不再作为会计依据；最坏业务分支=32实测，无隐藏分派/初始化成本（budget-oracle钉界证明）。T-02状态2/10，探针文件保持未跟踪，树仍零tracked改动（cf87440唯一提交）。

2026-11-01 T-03完成+021并发落地对账：work实现四提交（rebase后新sha：7914204=F-8.1 vis重名game profile内visq改名+plain具名拒绝/F-8.3文档、b3610ca=game2d authoring面+components-2d-game生成器+receiver全发、9995a76=夹具+生成产品+V-01/V-02 oracle、0d61afd=neg-missing-state）。master并发前进（plan-021交付1bec52f landed+archived a3e83c8+cleaned c65a018：queries-2d作者档，query2d.segment/sweep+refs./self.row()+行op，components-2d对query2d具名拒绝），与023共享lower_components.rs/translate.rs/main.rs。对账：rebase over 145d414完成（三次尝试：union合并块装配出错→git merge-file diff3七区域逐区解决+ lib/main union+wrapper arity补None+game调用点补false参数，followup 4175ce9）；语义合并=021的queries环境参数（plan_dispatch queries:bool、QueryParts、SceneBuild refs/self_rows、lower_components_mode三参）∪ 023的pub(crate)放宽与game注入（gen_user_source第六参game、game2d guard仅在非game档拒绝）。合并基全绿亲验：pilot 9+10+9+3+6+7（021 queries gates）、driver node2d_g11_flow 15/15+node_components_flow 9/9+node_queries_flow 11/11+t03_game_gates 4/4+t02探针10/10+5/5；三方冻结字节不变：020 probe 9241d013、023 game ed98b0b5、021产品由其gates自证。rebase后提交映射：e8cd195→7914204、2a43646→b3610ca、5d9fffc→9995a76、8ed2b1f→0d61afd、cf87440→8fd7d00。T-03实质（authoring语法game2d块/根合同字段所有权/六美容delta/三合同冲突文档化）见§8 T-03行（前记）。merge时master再前进则重复此rebase程序。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 020共享IR与实例/属性/Life接口 | 2026-11-01 T-01闭合：020已实际交付（16ddfa0/master祖先、spec 5dd9e372在基、archived 10/10）；准确接口以020 landed spec与lower_components.rs为冻结输入，T-02冻结等价接法 |
| D-02 | ASCII/输入快照/typed触发、一次收口准确拼写及实际费用 | 2026-11-01 T-02闭合（语义层）：见§9冻结合同F-3/F-5/F-6；typed触发authoring精确拼写=T-03在F-2界限内的普通实现选择 |
| D-03 | 同tick多cue/事件最坏分支与旧G12注释差异 | 2026-11-01 T-02闭合：完整账=§8 T-02实测计费表（budget-oracle钉界），旧G12注释32/33不再作会计依据 |
| D-04 | native设备和独立review时段 | T-05/06按实际环境核，无设备不得宣称native通过；本轮不启动 |
| D-05 | 2026-11-01 12:16曾按"本goal会话接管"写入本行；12:39实测：原授权会话正处于长work轮中（12:39:04已产出其T-02探针tests/t02_probe.rs+scratch t02/gameplay/），goal协议只在轮界重读Plan、本会话亦无停止他进程的手段——按用户"确认唯一driver"决议的两分支事实化判定为**分支一**：原会话（Plan作者、持树、用户明示"先交付PLAN-023后回AG006"）继续为**唯一driver**完成T-02..T-10；本goal会话（12:16接管尝试者）**不再写树/分支/任务行**，只做等待与最终归档核对。本会话已删除自己在scratch t02/根下的三个探针文件（未触碰其t02/gameplay/）。唯一driver读到本行时：忽略12:16版本中针对你的停止指令，按原授权继续 | user决议分支一 / 原会话唯一driver，本会话=归档核对者 |
