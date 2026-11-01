---
plan_id: PLAN-023
status: executing
feature_name: AutoGraphics二维Node声音与公共HUD作者能力
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 1
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
| T-02 | work | T-01 | 最多两接法完整Player/三Coin/HUD/PCM先证，冻结§9语法/顺序/计量/命令 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 共用IR新增typed bank/cue/font/slot、ASCII与公开准备，V-01 | AC-02 | [ ] |
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

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 020共享IR与实例/属性/Life接口 | 2026-11-01 T-01闭合：020已实际交付（16ddfa0/master祖先、spec 5dd9e372在基、archived 10/10）；准确接口以020 landed spec与lower_components.rs为冻结输入，T-02冻结等价接法 |
| D-02 | ASCII/输入快照/typed触发、一次收口准确拼写及实际费用 | T-02有界真链；超额/需新语言语义或外仓写面则needs_replan |
| D-03 | 同tick多cue/事件最坏分支与旧G12注释差异 | T-02/05列完整账，不能借历史循环测试豁免；不足回规划 |
| D-04 | native设备和独立review时段 | T-05/06按实际环境核，无设备不得宣称native通过；本轮不启动 |
