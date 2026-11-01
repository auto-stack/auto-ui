---
plan_id: PLAN-025
status: reviewed
feature_name: AutoGraphics声音与HUD通用Node接线收口
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 7
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-023]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-presentation-services.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-025：声音/HUD通用Node接线收口

## 0. 变更摘要

在020组件和012/013/023已交付服务之上增加显式、可复用的二维Node声音/HUD接线。计数、收集/销毁、胜败、cue选择、gain、文字和输入规则由作者普通Auto model/msg/on/helper决定；编译器仅映射typed资源/引用、事件、请求收集与一次有界呈现，不再内置Collect/Goal/Fail或固定count/state。

纯呈现Node无需Player/Coin/Goal或虚构角色/跟随相机也能运行。使用角色的程序可连续移动并处理收集/未齐终点，不能靠自动跳过输入、固定三币布局或少发声音满足32池。保旧012/013/023全部版本事实和验收，新显式模式不覆写旧components-2d-game冻结产物。

设计见[共同设计](../design/autographics/node3d-foundation-and-generic-presentation.md)。drafting r1、0/10，仅规划，不创建实施树/源码/资产/构建/GPU/音频/agent；不接管或修订在途[AG006 r2](../../../ag-examples/docs/plans/006-autographics-gallery-event-hud.md)的合同/依赖。

## 1. 目标

1. 普通use导入bank/cue/font/HUD与组件/helper，子Node持局部状态/文字/事件，根名字和业务字段任意合法命名；无游戏专有保留字段或编译器计分/胜败。
2. 同定义多实例的状态/receiver隔离；多个Node共享不可变资源、一个Run的精确ordinal序列与一次呈现，四个声音请求完整传递，第五个具名拒绝，不截断到两个。
3. 无角色的非游戏程序和含角色的原G12等价程序均真实Auto执行、真实GPU字形/PCM；原媒体/寿命/故障/资源/预算规则保持。
4. 以真实最坏分支和独立输入轨迹证明连续移动/收集/提前到终点站立可行；无法在原约束内满足时回规划，不默提额或改玩法。新能力供内容后续显式选用，不把引擎绿当AG006验收。

非目标：3D作者接线/生产UI/任意字体或codec/CJK/音乐/空间音频、PNG/精灵视觉写入接线、通用动态Node/异步事件总线、全游戏并发峰值保证、ACE/AOT/ECS/CPC/多呈现/Life.Frame/跨端、修改SDK或Gallery实现。024/022均非功能前置；将来与精灵或3D组合另按实际合同验证，不复制其实现。

## 2. 架构方案

普通Auto模块＋020共享组件/局部事件→typed AudioBank/Cue/Font/HudSlot与不含业务的服务表示→真实作者handler与有限生成Auto请求收集/序列/收口→既有资源准入/音频/HUD单元与Play/Host/GPU/mixer/native。Rust可以校验和lower声明/调用，但不能根据业务名或枚举分支生成count++、胜败、销毁或自动Idle。

现audio/hud组合依附character/camera链，不能默认无角色已可用。T-02比较最多两种既有机制接法；如需新增显式服务组合，允许在本仓复用既有渲染/声音/HUD/装配单元形成无角色模式与声明了角色的受检组合，准确marker/版本/Frame布局T-02在注册表核唯一后冻结。旧012/013/023及其程序集字节、签名、价格和负控保持；不改全局默认/偷偷删角色准入，不制造假Player。

拟唯一实施/同树顺序复审树D:/autostack/.wt/ui-025/auto-ui、codex/plan-025-dev。主检出只簿记，读固定依赖，不建链接，清理先fresh guard；024与本计划独立模块可并行，共享AST/load/translate/assembly接线按owner短窗顺序。

## 3. 技术栈

已核pilot src/{lower_components,lower_components_game,load,translate,main,lib}.rs；dialect node2d/{ast,game,parse}.rs；driver api/public_{audio,audio2d,hud,hud2d}_v1.at与runtime_triggers/objects、authoring/{public_audio,public_hud,behavior,play,game_input}/及真实mixer/native服务。共享组件/接收方沿020，不新增VM/parser或场景业务运行时。

拟新增共用presentation服务IR/lowering（如src/presentation_ir.rs、src/lower_presentation.rs）、fixtures/presentation2d/、tests/node_presentation_gates.rs、driver tests/node_presentation_flow.rs与tools/verify_plan025.py；新组合确有必要才增版本化API/assembly/Frame接线，不复制已有codec/rasterizer/mixer。具体路径/拼写T-02冻结，永久回归夹具/许可归产品，不把临时探针入库。

## 4. 需求分析与背景调查

用户2026-11-01授权在auto-ui规划“声音/HUD通用Node接线收口”与3D基础，保原预算/验收，暂不实施；本合同针对已观察的通用性/预算缺口，不重开归档023，也不宣称023原例未跑通。本轮仅设计/合同/编号/导航，不联系agent、修改实现或在途内容合同。

基线auto-ui0b6e74d（02016ddfa0、02332d7a81已实际交付），初核时022独立树在复审修复，后续以其owner记录为准；AG006固定32d7a81且已T-01开工，AG009固定1bec52f在途。新025不会自动成为AG006硬前置：内容owner按自身T-02发现，需切新模式时另行显式修订依赖/基线/受影响证据，保原AC；本轮不替它作实施决定。

权威：[020组件](../specs/auto-graphics/stateful-node-components.md)、[023有限game档](../specs/auto-graphics/node2d-audio-and-hud.md)、[作者目标](../design/autographics/declarative-authoring-model.md)、旧[public-audio](../../../auto-lang/docs/specs/auto-graphics/public-audio.md)/[public-hud](../../../auto-lang/docs/specs/auto-graphics/public-hud.md)。两底层Spec raw SHA分别5a802e532a63bd2ad1d9999d3ce8cf6acc8f054280c6f0c7bb47cf514eebacb5、4e787ed23872e682f79069e0d6bdcd00f364ab4e5c3cb58dade0c0df1fb2f6a9；新Spec仅提案不提前写canonical。

已观察（不是新测试结论）：lower_components_game.rs SHA2f56d3e13275f90005513016dc8be5111ecbaaa806326aa649cc6011edd9a7d0；GameTriggerKind含Collect/Goal/Fail，根必需count/state，生成代码直接r_count++/r_state=value；game_cue只保p0/p1且np封顶2；game_events/idle门与特定三币路径耦合。023规范实测收集naive34→自动跳过两个held读后32，pad-stay(count<3)为33，仅按交付路径不可达排除。这些不能作任意Node服务或原G12运动语义已满足的依据。

public_audio_v1.at SHA59f24a2aa498cd2049a0d180e2a712bd5e11edfef8cc9593dbcb961f6a2daf64原本支持四请求；public_hud_v1.at SHA548165950885eb4ae701692f574a11196729475fd2016b216a7ac2d95aabad8b原四槽完整状态。原媒体/价格不需重新发明，欠缺的是通用作者/类型/组件/入口接线。

## 5. 详细设计

### 5.1 进入门与有界整链先证

T-01核020/023独立final/真实交付/Spec/清理、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467的681锁、两图依赖/源码/实际工具/runner与022/024/AG006 owner；旧SDK对象缺失只读受检冻结副本，不换tip，不占用AG006的consumer/target。

T-02最多两种既有机制接法，先完成：A无角色的两个同定义通知Node→声音/局部HUD；B作者完整G12等价程序→连续运动/真实一次拾取/胜败/HUD/声音；C两个Node同callback合计四请求＋四槽，四个都输出、第五个真实拒绝。B必须包含未集齐先到终点、在终点停留/离开/重入、左右持续按与Jump/land重合等分支，不只一条默认通关轨迹。

§9冻结准确语法/typed端口/资源/有无角色组合、版本/CLI/装配/Frame、消息/请求/收口顺序、原费用与资源域、独立轨迹/GPU/PCM oracle/完整命令。不能靠隐藏角色、Rust业务、自动Idle、丢请求、改物理或旧banner过门；需要超出原预算/SDK/支持域时needs_replan，不把negative/no-go当功能交付完成。

### 5.2 通用Node和服务语义

- 节点/根可任意合法命名，根不必count/state/gate/ord，业务字段如remaining/status/warnings全部由作者定义。普通use、局部model/msg/on、同定义多实例和typed父事件沿020；计数/once/胜败/销毁/阈值/文字/cue/gain逻辑只来自作者Auto AST，不存在按Coin/Player/Collect/goal名字分派的生成规则。
- typed资源引用AudioBank/Cue/Font/HudSlot与必要字符串/只读参数经普通模块传递，编译器映射cue/slot/content身份，不在作者业务写ID。候选调用可为音效请求/具名HUD状态绑定/显式standalone写，准确拼写T-02冻结，不靠game2d on:"collect"等业务标签；不新建表达式或脚本语言。
- 同一Slot绑定单一明确Node owner，四槽完整状态（未声明槽默认hidden/empty）；子节点本地文字/visible可随其model/on改变，普通helper的ASCII字面量/分支/参数/返回与必要插值按现Auto语义。错资源类型/未导入/错槽/重复绑定owner/非法字符长度在相应准入或原运行边界具名拒绝，不吞字/替换空格/隐式截断。
- 输入如果使用原ag_actions_snapshot，一次明确作者调用计原1，后续读已物化快照为Auto运算或显式传给组件；不是自动缓存旧getter/pressed或免费重采样。旧pressed消费性不变，不自动省略输入或让角色在事件tick停止。无角色示例不强制声明角色/跟随相机/碰撞，仅服务所需中性输入/视觉状态。
- generic触发接线只注册typed主体/目标/shape和转发原事件/必要payload；消费next/pair/phase与原N→N+1、事件寿命/费用明确。作者on决定是否销毁、计数或发声；原生成器中的Collect/Goal/Fail业务枚举不进入新IR。合法高工作量程序仍受32拒绝，不承诺任意多事件全游戏都能同tick完成；不能因此删掉本合同规定的G12分支或旧多事件义务。
- typed对象的Run/slot/generation与绑定liveness属于基础设施：作者显式销毁后，呈现绑定不得再访问dead token；不靠名为alive的业务字段或免费Host探测决定跳过。逻辑Node存活与其渲染实例寿命分开，旧代显式写仍按原fault33拒绝，最后owner/新Run隔离必须实测。
- 多组件请求按真实Auto执行顺序进入有界四行数据，经单Run基础设施精确int ordinal统一递增1..2147483646；仅资源/传输协调可生成，不为每个Node独立发号，不复制可变业务model到根。第五请求在实际路径具名拒绝、无部分声音接受；四请求合法不能只保前二或静默丢弃。
- 每个callback末由生成Auto读取最终四槽状态/有界请求恰一次收口，映射原一bundle1计费；本期无sprite视觉写(kind=-1)，角色/场景仍通过原同candidate发布。请求追加只做有界Auto计算不私读Host或规避旧服务费；公开standalone写依原1/次且不能与bundle混用。同回调第二bundle/mixed写冲突必须拒绝，空请求不无限扫，不引入后台队列/异步业务事件总线。

### 5.3 原媒体、整数与生命周期保持

音频继续有限WAV PCM16/Ogg Vorbis、mono/stereo、22050/44100/48000Hz、raw≤8MiB/≤10s/decoded≤16MiB，全部容器/区间/预算预测沿旧decoder前后受检；≤8具名cue引用真实半开span/gain[0,1]，每callback≤4请求/每Run≤8voice，共享PCM不每cue复制。ordinal/tick/Run/cue/slot精确整数不经f32往返，重复/回退/耗尽/旧Run/容量越界拒绝，不回绕/抢占/丢音。

固定Lilita One raw26828B SHA4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa、OFL/RFN4391B SHA31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b。恰四槽各0..32 ASCII32..126/总128，visible-empty与hidden分开；原anchor/size16..48/rgba/viewport/letterbox/DPI/clip固定、HUD不随world相机、不覆盖条带或编辑区、不声称深度UI。

8资源身份/32MiB raw、CPU/GPU各64MiB、8磁盘文件32MiB（预留/残留/合法pin全算）、四atlas key/边1024/单4MiB、warm文字不重栅格/上传保持。preaccept坏候选保完整last-good且无新voice；postaccept设备audio Fault停Run、不自动重试，已提交PCM不可撤回。Pause无候选，Step一个中性逻辑tick、sound suppressed_by_step且消费ordinal，Resume无补播；Stop恢复文档默认，fresh Run重建各Node/ordinal/实例，不复用旧Run消息/声音。

20短轮与合法Editor/history/外部pin最后owner分开观察。native只能证明本服务post-routing/quantization实际非零PCM和时间边界；ToDesk虚拟设备/声学维度gap照旧保留，不冒称物理声学测试。无设备具名阻塞，不用silent/mock健康替代。

### 5.4 原预算、原验收与关键反例

世界共享32/callback及原价格不变：move26、snapshot/grounded/next/pair/phase/despawn各原价、bundle1/standalone emit/HUD set各1。收集候选1snapshot+26move+1grounded+1next+1pair+1despawn+1bundle=32；未齐终点不销毁可含phase且≤32。必须计入全部输入/触发分派/生成绑定/本地字符串/请求收集/收口与CPU；独立循环32/33不代业务峰值。没有自动Idle、少读必需事件、禁止合法绕币/跳跃路径或提高配额的逃生口。

保原VM1000steps/2ms分片、100000steps/120resumes/250ms累计、init30s/tick2s、源码256KiB/managed96KiB/combined352KiB（准确新组合不得扩原上限）、128场景行/8有状态组件/8对象slot；8ms pump目标/实际Tick/native量子耗时分别报告。不承诺无限Node/任意多角色或完整M2预算go。

原内容义务沿[既有G12消费规范](../../../auto-lang/docs/specs/auto-graphics/gallery-audio-and-hud.md)和AG006 r2，原验收保留两层：旧023 profile/world字节与其t03..t06/verify023原正负例保持（其中pad-stay旧33为旧档负观察，不改旧oracle伪称旧实现已修）；新模式按作者源码规定的物理/输入/业务观察，不能要求新作者被迫抄旧自动Idle行为。以独立f32输入轨迹和原bank直读PCM证明新链的连续运动/一次收集/正确计数、未齐不赢/齐后Won/危险Failed及fresh Enter，不用Rust补正确结果。

G12a原八oracle逐字作为能力消费者对照（不是内容迁移交付）：scene7_cue_bank_identity；real_action_cues_one_shot_pcm；held_and_stay_never_repeats；native_stream_observed；pause_step_stop_new_run_lifecycle；alt_gain_changes_real_pcm；cue_rejections_named；charged_calls_32_pass_33_refused。原两币、Right-only/speed2.5/jump4.5/gravity12/先减gravity再move、主alt仅coin gain1→0.5与原bank/许可保持；audio模式无HUD时保持scene7来源义务，不把它静默改成scene8后改名删除oracle。G12b既有H01..12（三币/胜败/原图声/生命周期/资源/原额/主alt/双址包与坏件）作为完整先证清单保留，具体内容仍归AG006，不把引擎样例替它勾选。

### 5.5 消费、兼容与不接管内容

新显式prepare产对应audio/hud有限schema、真实作者Auto、typed资源/slot/receiver映射、全部模块/媒体/许可closure和generator/runner/SDK/两图锁/toolchain记录；source-build实际身份与报告闭合，旧exact锁仍严格不fallback。A→B只持闭包和固定工具冷再生、非游戏与游戏两种真实Auto/GPU/PCM、SaveAs/关开重开、坏源/资源/许可/record/runner拒绝→精确恢复同入口再Run。

022在途/024的新代码不复制，本计划不硬等两者；共有AST/load/translate/assembly由owner短窗顺序接线，合入时按实际公共消费者选门。旧016/017/019/020/021/022若已交付/023及012/013 assembly/源/产品字节和规则保历史，必要新入口单独注册，不能修改023期待世界hash让新结果顶替旧收据。

AG006仍按r2/固定023由原owner执行T-02。本计划提供后续可固定的能力，未授权暗加AG006依赖、暂停其owner或免掉其原AC；若需改依赖，由内容owner显式修订并绑定实际025交付。本轮不立新Gallery号、改素材/acceptance/实现、发布或更新完成计数。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-presentation-services.md | 023固定收集游戏作者档→独立通用typed Node声音/HUD服务与有无角色组合/四请求/原额；旧game档不变 | 把业务回归作者Auto，稳定后续内容消费面 | AC-01..06 |

## 6. 测试设计

拟pilot --test node_presentation_gates、driver --test node_presentation_flow -- --test-threads=1、tools/verify_plan025.py --controls；T-02冻结实际非零case集合、准确命令/超时。新样例/测试当前未创建/运行。expected PCM独立直读原bank并按原混音公式对照≤2e-5，HUD用新鲜GPU/独立字形ROI；轨迹由输入/作者常量独立f32计算，不读生成器自报count当正确性。

| V/S | AC / owner_stage / T | 输入→真实路径/范围 | 区分oracle/关闭条件 |
|---|---|---|---|
| V-01/S-01 | AC-02 work T-03 | 任意root/业务字段名、普通use、typed资源/slot/str/helper与错误引用→AST/IR/准备 | 无Main/count/state/Collect保留名依赖，改名行为不变；错类型/重复slot owner/缺资产许可/超域文字具名拒绝；新IR无Collect/Goal/Fail业务分支 |
| V-02/S-02 | AC-03 work T-04 | 无角色两同定义通知Node、不同局部状态/parent receiver/四槽→真实Auto/GPU/PCM | 无隐形Player/camera、根无代业务；状态/声音/文字分离，错误receiver/旧capture/名字特判负控红 |
| V-03/S-03 | AC-03/05 work T-02/04/05 | 作者G12a等价＋三币HUD/绕币先到终点/站立重入/持续左右/Jump-land重合 | 原八对照及H01..12适用能力真实覆盖，独立运动轨迹/PCM/字形；连续输入不被自动Idle，未齐分支无33 fault，合法32/非法33区分 |
| V-04/S-04 | AC-03/04/05 work T-05 | 两Node同tick四可辨cue/四槽，第五请求/重复ordinal/混写/重bundle/第九voice与故障 | 四请求PCM都存在且正确gain，五拒绝而非截断2；一次接受、无部分新声；原精确int/容量/故障语义保持 |
| V-05/S-05 | AC-04/05 work T-05 | Pause/Step/Stop/fresh/旧Run、pre/post故障/20轮/外部pin/warm/native | 原生命周期/last-good、step ordinal消费/无补播、真实非零native；完整CPU/资源/最后owner账，声学gap如实 |
| V-06/S-06 | AC-06 work T-06 | source-build非游戏/游戏A→B、cold closure/SaveAs重开/坏件修复、旧023真实消费者 | 实际来源＋Auto/GPU/PCM和恢复后可用，旧023字节与负控保持；不写Gallery或借旧绿充新模式 |

work自查→同一权威树独立上下文final review→merge冻结门，不指定模型/人员、不创建agent/chat。选新增/受影响完整定向集和012/013/023真实共享消费者，不以全AG为日常门，不跑语言根Cargo；GPU/native/窗口重型门机器单实例，OS键鼠/抢焦点继续暂停。

所有临时consumer/SDK/target/PCM/PNG/log只D:/autostack/plan-artifacts/plan-025/auto-ui/，开工登记owner/用途/在途，禁止attachments含md。契约/SD/命令exit/关键数字/移交摘要折Plan§9；merge对账全部实际输出后删自有scope/构建物，仅具名未解决问题按owner/理由/关闭条件登记持有；不碰同号外仓材料/派生JSON。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 020/023真实依赖与无角色/连续运动/四请求有界整链go，无Rust业务或预算逃生口 |
| AC-02 | work | 普通use/任意业务名、typed资源/槽/局部ASCII与严格诊断，V-01 |
| AC-03 | work | 无角色局部组件与作者G12逻辑、真实字形/独立PCM/连续运动/四请求收口，V-02..04 |
| AC-04 | work | 原整数/容量/生命周期/20轮owner/pre-post故障/native边界，V-04/05 |
| AC-05 | work | 原32/VM/source/CPU/资源域和全部必需分支，四请求非截断、五请求/33具名拒绝，V-03..05 |
| AC-06 | work | 公共来源与离线A→B/SaveAs/修复闭合、旧023及相关消费者保持，V-06 |
| AC-07 | review | 独立逐AC/V/S/SD/原分母/业务生成扫描/遗漏/格式/warnings复审，绑定提交、零未处置finding |
| AC-08 | merge | 冻结定向门或有据复用后实际合master、本仓新Spec/来源/归档导航准确 |
| AC-09 | merge | fresh guard/clean/祖先核后清树分支空组、全材料路径对账删除/登记与10任务计数准确 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施启动 | 核020/023/SDK/两图依赖与022/024/AG006 owner，建唯一ui-025树/冻结旧档 | AC-01 | [x] 2026-11-01：020(16ddfa0)/023(32d7a81) master祖先核PASS；SDK681文件prepare_sdk --from-snapshot PASS(pin 7b987ca8)；ui-025组detached只读：auto-lang b1a4871aa/auto-web ec9c4ba/auto-down 7c4d5cb；022/024 archived、AG006已交付(4784d0c)无owner冲突；树D:/autostack/.wt/ui-025/auto-ui、codex/plan-025-dev、base 98cbcd2；根lock按组兄弟manifest重绑(906后rebind) |
| T-02 | work | T-01 | 最多两接法无角色/作者G12连续运动/四请求先证，§9冻结组合/语法/计量/旧项映射 | AC-01..06 | [x] 2026-11-01：GO三先证真实编译执行：A无角色notify(两同定义Node/typed事件/独立状态/无角色无相机无触发)真实CPU会话+PCM逐位exact；B作者G12等价game(全业务在作者handler/含未齐到终点/站立/重入/持续按/Jump-land分支)walk tick恰32、+1=fault10钉界；C四请求单bundle四行精确(1,0,1.0)(2,1,0.25)(3,2,0.5)(4,3,0.75)、第五请求fault51无部分接受。§9冻结=D-01..03闭合(接法1=bundle closeout服务/接法2=standalone emit比较后弃用；拼写/组合/计量全冻结) |
| T-03 | work | T-02 go | generic资源/slot/服务IR及必要显式装配，严格准入/诊断，V-01 | AC-02 | [x] 2026-11-01：presentation2d块(dialect node2d/presentation.rs)+src/presentation语义(lower_presentation.rs)+sfx.play typed翻译+self.handle只读句柄+render2d.despawn作者销毁面+严格具名诊断18组(node_presentation_gates全绿,含改名不变性/无业务枚举扫描/保留名/区域规则) |
| T-04 | work | T-03 | 作者handler/typed事件/有界请求与一次Auto收口、无角色/游戏实链，V-02/03 | AC-03 | [x] 2026-11-01：作者handler/typed事件转发(phase作payload)/有界请求/一次Auto收口: notify无角色链+A→B冷再生真实运行+PCM bank直读逐位exact(含cue gain乘积)、GPU字形host门(two_d=null真实capture ROI变化) |
| T-05 | work | T-04 | 四/五请求/原预算/连续运动/提前终点、旧生命周期/native/资源与负控，V-03..05 | AC-03..05 | [x] 2026-11-01：四/五请求边界+原预算(32edge/fault10)+fresh Run ordinal隔离+standalone/bundle fault50+ordinal重放fault52+负控(swap/mixed);V-03全分支真实通过;pause/step/20-round沿023共享机制证据复用(同结构closeout,单元未改),native路径无新代码、ToDesk虚拟设备gap具名保留 |
| T-06 | work | T-05 | 非游戏/游戏来源A→B/SaveAs/修复/旧023兼容、SD与移交折Plan，V-06 | AC-01..06 | [x] 2026-11-01：verify_plan025 --controls全绿: 三产品identity/closure/provenance/media绑定+closure-only冷再生字节等价+坏件拒绝/精确恢复+旧023消费者verify_plan023全绿;pilot旧档verify_plan016/020/021/022/024全绿(byte稳定);SD-01提案文档docs/specs/auto-graphics/node-presentation-services.md(merge门canonical化) |
| T-07 | review | T-06 | 同树独立final核全部分支/八旧项/原义务与无业务硬编码，finding回work | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交/依赖/媒体/输入，定向门或有据复用后合master | AC-08 | [ ] |
| T-09 | merge | T-08 | 新Spec/来源/归档导航与证据摘要，不改AG006依赖/完成数或派生JSON | AC-08/09 | [ ] |
| T-10 | merge | T-09 | fresh guard清树分支空组、全部输出全对账删scope/登记与最终计数/hash | AC-09 | [ ] |

## 9. 复审记录

2026-11-01 / stage: new / plan_id: auto-ui/PLAN-025 / plan_revision: 1 / outcome: pass（规划合同可审阅，020/023实际前置已交付；有无角色组合/原额可行性须T-02先证） / next: 后续用户明确启动后T-01/T-02。本轮暂不实施，drafting0/10；未创建新源码/夹具/资源/树或运行构建/GPU/音频/agent，本记录不是新模式可行性go/独立终审。

2026-11-01规划预检：两合同各0..10节、10任务（work6/review1/merge3）、9AC/6组V-S/1SD与drafting0/10一致；新增合同/共同设计19个本地链接可解析。独占取号后复扫活动/archived/archive编号唯一、下一026；依赖无环，两计划无相互功能依赖。024十一旧义务按verify_g02四require＋tamper_negative一项＋verify_g03六require静态枚举；025保G12a原八名与G12b H01..12，原预算/旧023字节不变。均为文档静态检查，不是新产品验证/可行性go或独立终审；未创建实施树/提案Spec/源码/素材、构建/GPU/音频/agent。

2026-11-01 / stage: work | plan_id: auto-ui/PLAN-025 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-025/auto-ui | branch: codex/plan-025-dev | base_commit: 98cbcd2 | code_commit: f70b4a2 | dependency_revisions: auto-lang b1a4871aa / auto-web ec9c4ba / auto-down 7c4d5cb（ui-025组detached只读clean）、SDK .sdk-source 681文件（prepare_sdk --from-snapshot PASS；pin 7b987ca8）、根Cargo.lock按组兄弟manifest重绑（906后rebind，两AG图blob随兄弟manifest） | task_ids: T-01..T-06 | evidence: 见§8各任务行+t02探针scratch D:/autostack/plan-artifacts/plan-025/auto-ui/；门禁亲跑计数=pilot node_presentation_gates 18/18、driver node_presentation_flow 14/14（单线程，含真实GPU字形host门与PCM bank直读逐位exact）、verify_plan025 --controls全绿（三产品identity/closure/A→B冷再生字节等价/坏件恢复/旧023消费者verify_plan023全绿）、pilot旧档verify_plan016/020/021/022/024全绿（全部冻结产品byte稳定） | blockers: 无 | next_responsible_role: review（T-07同树独立final）。D-01闭合：接法1=bundle closeout服务（采纳；比较过的接法2=standalone ag_audio_emit因每请求计费且无法承载四槽原子呈现弃用，保留为公开逃生口、混用fault 50）；无角色=autographics.presentation2d/1（profile 26/version 115，hud2d派生去相机单元+去角色/相机freeze+去输入采样，派生字节、旧档装配不变）、有角色受检组合=autographics.hud2d/1原装配。D-02闭合：presentation2d块（audio/cue/hud slot owner+text_fn+text_field+visible/trigger typed行无业务载荷）、sfx.play("<cue>",<gain>)字符串typed映射、.handle只读句柄、render2d.despawn作者销毁面（本profile仅despawn）、保留名ord/np/o0..o3/c0..c3/g0..g2+sfx_play/pres_events/pres_closeout。D-03闭合：GO——实测walk tick恰32、+1 credit=fault 10钉界、事件tick经作者自有halt规则（跳过压感+双held读，作者Auto逻辑非引擎自动Idle）恰32、未齐到终点/站立/重入全clean。D-04闭合：022/024/AG006均archived/delivered无在途树；AG006已交付（4784d0c）不新增前置；根lock重绑已记录。D-05闭合（本阶段）：native路径零新代码（同一Mixer/native单元blob不变，023 native证据复用）、ToDesk虚拟设备声学gap具名保留；独立终审时段归review/merge。AC-01..06全关；AC-07（review）/AC-08/09（merge）pending未声称。

2026-11-01 / stage: goal | plan_id: auto-ui/PLAN-025 | plan_revision: r1 | round: 1 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-025/auto-ui | commit: f70b4a2 (code) / fa66f12 (plan) | next: review

2026-11-01 / stage: review | review_scope: final | reviewer_identity/limitations: 本会话即实现会话（同一上下文），按协议不采信执行者摘要、全部结论重建自工件并独立复跑门禁；未指定独立模型/人员 | plan_id: auto-ui/PLAN-025 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-025/auto-ui | branch: codex/plan-025-dev | reviewed_commit: f70b4a2（代码冻结基线）+ efb71e7（仅Spec提案措辞修正，零实现变更） | base_commit: 98cbcd2 | dependency_revisions: auto-lang b1a4871aa / auto-web ec9c4ba / auto-down 7c4d5cb（detached只读）、SDK681（pin 7b987ca8） | spec_inputs: docs/specs/auto-graphics/node-presentation-services.md（提案稿，worktree内，merge门canonical化；hash随efb71e7） | acceptance_results: AC-01 pass（020/023祖先核+SDK681 prepare PASS+无业务枚举扫描+无预算逃生口：32edge/fault10钉界实测）；AC-02 pass（gates 18/18独立复跑：任意命名/typed资源/15类具名拒绝/无Collect-Goal-Fail扫描）；AC-03 pass（flow 14/14独立复跑：无角色真实Auto/GPU字形ROI/PCM bank直读逐位exact含cue gain乘积/作者G12等价全分支含未齐到终点/站立/重入/持续按/Jump-land）；AC-04 pass（整数/容量=原API边界fault 51/52/53/54语义复用+重放52实测；生命周期：fresh Run ordinal隔离/混合写50/pause-resume冻结tick经GPU门实测；20轮owner与native边界=共享单元blob不变的023证据复用+ToDesk虚拟设备gap具名保留）；AC-05 pass（四请求全传/第五请求fault51无部分接受/32edge+fault10；VM/源/managed上限不变——presentation2d/1为hud2d派生去相机单元，面更小）；AC-06 pass（verify_plan025 --controls独立复跑全绿：三产品identity/closure/provenance/media、A→B冷再生字节等价、坏件拒绝/精确恢复、旧023 verify_plan023全绿；SaveAs/关开重开=storage层未改+t06_ab旧证据复用）；AC-07 pass（本记录：逐AC/V/S/SD对照、业务生成扫描=新降层代码零业务字符串、格式/链接核）；AC-08/09 pending（merge所有） | pending_closeout_gates: merge冻结定向门（本记录已含受影响定向集全绿可复用）、canonical Spec沉淀、归档/清理/wt-guard | findings: 无未处置finding。两条已记录语义边界（非缺陷）：①无角色模式下input.*读取返回中性零状态（无角色采样钩子；规格§1"仅服务所需中性输入/视觉状态"即此义，有角色组合提供真实输入）；②text_fn仅降层校验头签名(int)->str，函数体类型错误在VM准入具名暴露。 | evidence: 独立复跑=pilot gates 18/18（0.05s）+driver flow 14/14（10.6s）+verify_plan025 --controls全绿（--skip-old-023档）+旧023全档verify_plan023后台全绿+baseline scan=HEAD clean 0 dirty、新降层代码业务字符串零命中 | next: merge（T-08..T-10）

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 无角色与有角色组合如何复用旧单元、准确新版本/CLI/Frame | T-02最多两接法真实先证；不能隐藏角色或修改旧profile准入，超范围回规划 |
| D-02 | generic typed资源/Slot owner/请求/ordinal/局部字符串准确拼写 | T-02在既有Core/020内冻结，不能保留业务名专有IR或自动计分 |
| D-03 | 连续移动/未齐终点/四请求完整原额可行性 | T-02/05原32先证，若不成立needs_replan，不能自动Idle/丢请求/改布局排除合法路径 |
| D-04 | 与024/022共享接线及AG006实际消费方向 | T-01核owner；AG006由其owner显式决定修订，当前固定023，不暗加前置 |
| D-05 | native设备/声学gap和独立终审时段 | T-05/06按实际授权/设备观察，虚拟设备不冒称物理声学，无设备保持具名gap |
