---
plan_id: PLAN-018
status: archived
feature_name: Design52-D-UI编译配置收口
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/README.md]
new_spec_components: [docs/specs/build/ui-compilation-configuration.md, docs/specs/build/ui-test-selection.md]
touched_goals: []
affects: [auto-ui-runtime, auto-ui-compiler, auto-ui-tools, ui-sdk-delivery, ui-test-infrastructure]
---

# [auto-ui/PLAN-018] Design52 D：UI编译配置收口

## 0. 变更摘要

承接已交付auto-lang/900的UI迁入和执行中的auto-lang/901（Design52 B）的语言SDK/发行投影交付。在auto-ui把既有默认iced后端及其必需依赖变为稳定编译基线，消除“默认已开、仍按同一开关切另一套代码”的ui-iced编译分支；UI测试集合转为编译可见、运行显式选择。真实可选的mpv、autodown、tree-sitter、原生拖放等产品依赖保持可选，手工窗口/真输入测试不自动进日常。

这是现有[Design52](../../../auto-lang/docs/design/52-single-compile-profile-consolidation.md)的D实施合同，不新起UI架构、不重做900迁移，也不代901修语言测试feature退出。主责任仓auto-ui，其他仓仅必要消费者接线。

当前drafting 0/8，仅正式立项。未创建实施树/agent、未构建测试/启动窗口、未安装或更改用户环境、未改代码和canonical Specs。实施须用户明确授权并接受901正式交付；本次规划不借在途901代码运行验证。

## 1. 目标

- G1：runtime的iced默认能力形成单一稳定依赖闭包；同样的UI能力不因旧ui-iced开关产生另一份库单元或无后端stub。
- G2：UI测试编译门退出后，t/tu/ti/tn以及迁入测试的原身份、oracle、ignore/平台/工具条件和运行范围保持，无隐式窗口/输入抢占。
- G3：900迁入的compiler pipeline/host安装、LSP、cosmic、测试与28个显式example目标有明确当前归属与兼容接线；不靠重新搬代码证明完成。
- G4：901的新SDK依赖/投影被接收；新生成及受管发行工程fresh/locked/重定位/失败恢复可实际消费，旧支持形态与迁移诊断明确。
- G5：真实编译单元与成本证据证明默认UI/测试选择造成的重复编译收敛，给014明确结构基线、给007精确示例差额。

非目标：UI014的CLI/Vue/Aura/renderer内部结构改造；UI007新主题/整棵examples/ui搬迁；取消Vue/VM前端或重设计Feature/Service/HMR；为追求“全都同一份”启用全部mpv/autodown/平台原生能力；把纯compiler/dialect/LSP强制依赖renderer；改SDK code-editor语义、ABI或默认宿主安装时序；改debug/优化级别/工具链凑收益；全量AG/全部应用验收；Cargo共享target/Kache、清全局缓存或远端发布。

## 2. 架构方案

稳定UI基线只覆盖runtime已默认ui-iced的实际传递闭包：ui/ui-interpreter/image-pipeline/code-editor/ui-clipboard/ui-dialog/native-dock/native-clipboard及所需依赖。按真实调用分析删可达恒真cfg和反向stub，保留平台cfg及真正可选依赖。runtime依赖的Core/code-editor保持既有默认runtime行为；不得把它扩为纯Core/语言全局默认。

UI compiler/dialect仍可独立消费语言基础；ui-headless的名字不自动证明“零iced”，须核实际依赖和公开支持面。现有用户显式no-default-features形态的行为变化是本设计的迁移点，T-01必须列实际读者及受影响承诺，给替代路径/具名诊断；不悄悄把原支持形态删掉。若发现必要产品能力不能在现有包内兼容，应先回报设计差额，不在本计划顺手拆crate或扩架构。

### 与B、014、007的边界

| owner | 本计划接收/交付 | 不接管的内容 |
|---|---|---|
| auto-lang/900（A） | 已归档10/10、UI master 4e9acd5的迁入结果；当前生产owner/路径/测试ID | 不重开其迁移，不把未测冷构建体积或独立上下文审查限制改写为新PASS |
| auto-lang/901（B） | 必须接收最终语言/UI/Web提交组合、常驻TestCase/HTTP helper合同、投影新旧支持矩阵和真实消费证据 | 语言七test-*退出和相应SDK发行破损由B解决，D不得恢复语言feature或复制旧SDK兜底 |
| 本仓018（D） | runtime默认依赖基线、UI自身test-http-e2e/iced-layout-tests/test-native-dock的编译/执行分离、生成模板和消费清单、必要工具接线 | 不把所有产品feature转恒开，不改原子oracle/手工输入语义 |
| 本仓014 | 接收D冻结提交、稳定依赖/测试入口、生产调用owner及未闭限制，再修订原“feature/profile不变”的旧基线 | CLI/Vue/Aura/renderer结构实现仍014独有 |
| 本仓007 | 接收900后28个显式example/测试归属与整棵examples/ui差额，接受D对生成工程的变化 | 本计划不迁主题/业务demo，不声称900的examples已完成007全部归位 |
| auto-man/Web/OS/应用 | 当前直接声明UI feature的受影响清单/入口按最小接线适配；组合消费固定来源 | 不接管CLI结构、Web算法、OS产品或所有应用的新发布 |

实施唯一组D:/autostack/.wt/ui-018/{auto-ui,auto-lang,auto-web,auto-down}：UI分支codex/plan-018-dev；未改仓固定detached公开提交，确有必要源码改动的参与仓各一树一分支，T-01登记（Man仅必要时补入）。禁止junction/symlink；实施、内部核查、独立终审同一权威树顺序进行。Plan簿记唯一在auto-ui主检出docs/plans，归档沿本仓archived/。

## 3. 技术栈

沿用Cargo resolver=2、nextest/libtest、本仓t/tu/ti/tn及现有SDK发行、真实工具协议、原native VM验证入口。实现新布局/运行选择用固定参数、过滤及ignore/环境条件，不新增动态叶目录/影响图/测试runner。Windows当前实测面与Linux cosmic源码/平台条件分列；没有Linux环境不冒跨平台已测。

## 4. 需求分析与背景调查

### 4.1 授权、指令与来源

用户明确先规划Design52 D，核900迁入、UI自身ui-iced/test feature、接受901 SDK与发行投影，本次没有实施授权。已用auto-plan-new，读其verification-contract参考。未发现D:/AGENTS.md、D:/autostack/AGENTS.md或auto-ui根AGENTS.md；本仓docs/plans/README.md要求本地独立取号、archived归档、唯一worktree、四阶段、guard。实施前重核子目录指令。沿本次用户最新政策：计划文件外所有计划材料只放统一plan-artifacts，merge即删，不写legacy specs.json。

本仓无new-plan脚本；按本仓README，持Git元数据内独占锁，扫描本地活动及archived（兼查archive），确认最大017/.next-id=018，以CreateNew创建018并推进019；不用语言仓号源。

已读本仓docs/specs/README.md及runtime/native-vm-app-validation.md，auto-lang/docs/specs/auto-ui-runtime/project.md、repository-boundaries/generated-ui-distribution.md、Design52、900最终收据、901现行r1与UI014/007合同。部分Spec/源码注释仍是814/831旧门/旧owner，实施按实际公开合同分类，不能照旧行号机械删除。

### 4.2 只读实物（2026-10-31；不是测试结果）

UI master 4e9acd52596c69f380447ef0608ef256439802e4；语言主检出此前核b55889dffe970fb059ed841b9ff30e2e55f74ea9；901读取executing1/8。UI已有docs设计/计划README及007/014等WIP，全部保留，不纳入无关修改。

| 实物 | 观察/风险 |
|---|---|
| workspace Cargo.toml | 900后显式11成员，含auto-lsp和cosmic五包；cosmic的Linux依赖不适合直接要求Windows cargo build --workspace全绿 |
| runtime Cargo.toml | default=[ui-iced]；ui-iced传递依赖/能力须T-01重新以TOML计算闭包；大量optional基线依赖仍在；special能力独立 |
| cfg粗计 | crates Rust逐行匹配cfg/cfg_attr含feature：ui-iced316处/77文件、ui-interpreter170、ui67、code-editor66、image-pipeline32；含反向/组合/注释候选，不作为独立义务数量 |
| UI测试feature | runtime有test-http-e2e、iced-layout-tests（含optional iced_test）、test-native-dock；ui_tests.rs的HTTP门1处，native拖放集成测试顶层Windows+native-dnd+test-native-dock，真实输入必须手动 |
| example/test目标 | runtime显式28个example，其中26个required-features含ui-iced；p023_probe归iced-layout-tests、mpv_spike归mpv；3个显式mpv集成目标保原特定依赖。自动发现的其他集成目标须另清点 |
| 轻包 | compiler features为default=[]/ui=[]/ui-headless=[]，dialect/LSP无feature；不能因runtime配置收口让它们反向链接runtime |
| 当前入口 | .cargo/config.toml：t=nextest run --workspace --lib；tu=test -p auto-ui-compiler --lib ui_gen::；ti=tools/runtime五具名集成目标；tn=Windows拖放手工目标并传native-dnd,test-native-dock；RUST_MIN_STACK=16MiB |
| 工具/生成器 | auto-ui-tools没有声明ui-iced，但rust_ui.rs生成模板含ui-iced cfg与无后端stub；这属于生成文本，不能和工具自身cfg混同；新鲜度函数还依赖default=[ui-iced]字符串，存在反复重生风险 |
| 发行投影 | rust_ui.rs同时有源crate投影、installed变体、generated Cargo模板、依赖pin/锁/patch策略；901正在改TestCase闭包。D只能在B正式落地后改变UI自身feature表达 |
| 直接消费 | auto-ui-tools runtime依赖启autodown；cosmic demo/host-libcosmic请求ui-headless；Man auto CLI的ui-extra可选runtime，并转发autodown/mpv族；最小Core CLI边界不得因D误拉UI |
| patch/profile | UI根当前无[patch]或[profile]段；runtime声明registry iced_widget，语言900收据说UI接管patch但是否全部接线须核。D核实际root/生成/发行patch一致性，不能据注释声称已修/已快 |

hash：UI根Cargo.toml da293af3b22d466b489ffe21eb49f397d7b17943af2cc542dad0aeff54576b50；Cargo.lock dcfe859236f8fdf85c2be528813968a8dda40312e0588d56786c08e7cfc0c253；.cargo/config.toml 869a4df63dbb256234c9bfdfb7364563bae26e9a4c6bb528dde3eef88958ef76；runtime清单be12cc3933a90b51473ef6952d59e6b70c739618dcdfab839b521c0bf42d38bc；rust_ui.rs d42ddbff2c7446228133a9350bb4cb0576b4bdee0e506600396b38a9b4ed7452。

以上只用文件读取/TOML解析/Git状态，无cargo tree/metadata/list/build/test结果；cfg粗计不含完整宏展开，不能作为完工计数。901在途组合不被认作稳定交付，正文hash必须在T-01重冻。

## 5. 详细设计

### 5.1 先冻结默认闭包与兼容面

T-01接受U901（实际合入、规范、收尾），重读B的UI改动与交付支持矩阵，冻结生产依赖/公开入口、测试可见与运行ID、源码/lock/工具身份。按默认闭包区分恒真后端门、真实可选依赖、测试集合门、平台门、无效声明及生成字符串。

ui-iced作为实现选择器退出：对应依赖与代码在runtime基线恒在，生成应用不再用同名feature制造另一份无后端程序。当前受支持旧生成工程若仍传该feature，必须提供可验证过渡：优先保留无行为差异的兼容别名（不被任何cfg使用、不切换依赖），或由现有受管迁移事务安全改写并验证旧工程；T-01按既有支持合同冻结，不凭重命名就宣称破坏面为零。兼容别名如有，明确原因/消费者和退出条件，不能靠空别名掩盖残存开关。

ui/ui-interpreter/image-pipeline以及默认能力门只退出恒真实现分支；非默认headless/optional产品语义不机械删除。platform约束依然生效，Windows API依赖不能被拉进无条件跨平台代码。

### 5.2 测试编译与运行分开

UI test-http-e2e和test-native-dock的测试集合选择不再使用Cargo feature。iced-layout-tests中仅测试用途的iced_test移至正确dev依赖，测试体保cfg(test)，example p023_probe需独立保其合法工具依赖/入口（不能把dev-only库误用于生产lib）。若它含真实非测试依赖作用，拆清该作用再移除测试选择职责，不强启额外产品后端。

每个原测试的ID、条件、golden/原子断言/资源预算建立一次性映射，不新建动态目录。t/tu/ti仍按原职责执行；新可见HTTP、GPU/布局、真光标/窗口族不能自动进入日常；tn显式手工运行保原native-dnd真实能力条件，去除test-native-dock门后用ignore/明确手动入口保持准入。平台缺席/原ignore是排除，不能算PASS；不批量ignore普通测试凑编译/时间门。

t当前--workspace含Linux成员风险及新可见测试，只按真实责任调整原owner命令/静态选择，使在Windows/原支持平台的范围可执行；不能无解释删成员/用例。全AG里程碑保持独立，不为D合入运行全AG套件。

### 5.3 生成、发行与来源

修改rust_ui.rs生成Cargo/入口、需重生判断及UI投影时，沿用901提供的TestCase常驻/新旧SDK兼容，不复制B另一份投影。相同内容再次生成不应因旧default字符串不断重写；必须验证真正输入/依赖变化仍失效。生成程序宿主安装时机、单一状态owner、资源和消息语义不改变。

真实受管发行验fresh/locked、独立消费根与A→B重定位，再交付幂等；错source/资源/缺工具/投影失败时保旧有效工程并能恢复后再消费。沿原root/依赖pin/patch/锁/stdlib/schema合同，root与生成工程的iced家族来源一致，不借全局旧exe或主检出WIP兜底。历史冻结SDK样本不批量改写；需要适配当前消费者时明确版本边界与旧新对照。

### 5.4 成本、证据、回退

保持既有debug/opt/jobs/工具链/profile，按相同包/能力/target比较默认build、准备test/dev目标、切换t/tu/ti后的Cargo JSON单元/feature身份。test harness、dev依赖、MPV/autodown/平台真实变体单列；验收是默认UI和测试选择不再产生同角色库的feature分叉，不能承诺所有目标零新增rlib或强行统一特殊能力。

给出准备/冷编译/发现/运行/清理/整命令及空间逻辑/唯一文件数据对照，无指定节省百分比；不清他方target。若发现常驻测试/基线使同义日常准备/增量额外超过120秒，先核归因并回报设计决策，不自动降样本、改debug或重新加测试feature；此为本计划拟延用的成本重议门，实施确认时一并接受，不宣称是已测结果。

开工登记D:/autostack/plan-artifacts/plan-018/中auto-ui身份与跨仓子目录，若同号scope被其他仓占用，先核owner并以仓身份隔离，禁止覆盖。除计划文件外所有计划材料/消费者/安装副本/外置target统一该scope；正常target在专属worktree。merge把摘要、命令/退出码/关键数字/节选折入本计划，逐项核所有输出目录后删scope与工作树；只有具名未解决问题按owner/关闭条件持有。无worktree的本次规划不留探针文件。禁止attachments入git、禁止写specs.json或另建第二证据目录。

回退以实际修改仓的提交组合逆序恢复，保原支持工程和诊断；不reset用户WIP，不回退900/901已经交付的依赖边界。不自行安装驱动/平台SDK、启动所有应用或更新运行中的用户程序。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/build/ui-compilation-configuration.md（新） | 默认feature控制renderer→稳定runtime基线、真实可选能力、历史形态接收和来源 | D有限编译合同 | AC-01/03/04/05 |
| SD-02 | add | docs/specs/build/ui-test-selection.md（新） | UI测试集合编译门→编译可见、固定运行/手动/平台/资源条件 | 测试守恒 | AC-02/03/05 |
| SD-03 | modify | docs/specs/README.md | 注册两篇现行合同和跨仓owner指向 | 不复制全部语言旧Spec | AC-06/07 |
| SD-04 | modify | lang docs/specs/{auto-ui-runtime/project.md,repository-boundaries/generated-ui-distribution.md} | 【no-impact核清】lang specs的ui-iced引用均为lang侧历史/Man转发链描述（别名下仍真），runtime内部门无涉——不改动 | 仅D真实交付差额 | AC-01/04/07 |

## 6. 测试设计

全部为未来验证，当前未执行。work负责实现与内部反例；review使用同一树独立上下文核AC/SD/来源，不把自查称独立终审。本次不创建agent/新聊天；执行阶段角色沿review技能安排。native验证遵docs/specs/runtime/native-vm-app-validation.md。

| case_id / work task | AC | 实际入口/输入 | 区分性oracle与覆盖边界 |
|---|---|---|---|
| V-01 / T-01/02 | AC-01/05 | 固定UI包cargo metadata/tree/JSON build；默认与旧ui-iced/受支持no-default形态；单独compiler/dialect/LSP | runtime baseline真实依赖/feature一致，无“没后端”stub；轻包不新增runtime边；兼容/拒绝按事前矩阵，不能只看编译成功 |
| V-02 / T-01/03 | AC-02 | 旧各测试feature最大可见并集、t/tu/ti/tn逐ID→新枚举/选择 | 原ID/子断言/ignore/平台/资源条件全映射；日常不新增真输入、HTTP不丢、mpv原依赖不隐开；零选择不得PASS |
| V-03 / T-03/05 | AC-02/03 | cargo t、cargo tu、cargo ti按冻结受影响包/模块/目标；tn仅列/核手工准入 | 实际原owner职责终态，原测试条件保留；窗口族按具名测试串行，不跑全AG里程碑，不给未执行手工项补PASS |
| V-04 / T-02/05 | AC-03 | runtime pipeline/host installer及900迁入命名测试；代表性VM窗口冷启→输入→状态读回→关闭→重开，Vue已有同源消费 | 真实工具/窗口与输入可归属，状态和输出独立oracle正确；renderer不因cfg删反臂改变安装/资源；仅截图或首帧不够 |
| V-05 / T-04/05 | AC-04 | auto-ui-tools rust_ui投影/生成/新鲜度命名单测，新旧受管清单、重复/坏清单、再次生成 | UI与901 SDK依赖不漏，兼容/漂移拒绝/幂等保持；未改输入不重生，改真实输入必失效；不是改expected凑绿 |
| V-06 / T-04/05 | AC-04 | 既有SDK发行入口，fresh/locked外部生成，A→B重定位及错资源/工具/投影故障恢复 | 真正构建运行对应源码；失败保旧字节/可消费状态，恢复再成功；同一iced/patch/lock家族，无主检出隐式回退 |
| V-07 / T-04/05 | AC-01/04 | Man core/full UI选择、tools带autodown、cosmic清单/当前平台可用目标及mpv/native-dnd选择对照 | Core CLI仍无UI；当前产品能力选择保持；Linux不可用记未执行而非全平台PASS；不以--all-features拉全闭包代替验证 |
| V-08 / T-05 | AC-05 | 同配置build→准备必要test/dev→受影响t/tu/ti切换，按同角色JSON单元比较 | 默认/测试选择不再分叉；真实可选能力/dev/test合法变体具名，计时/空间可归因，额外>120秒重议 |

T-01冻结精确受影响用例、入口参数和来源；t/tu/ti是当前owner入口，不代表必须机械重复整仓/所有AG。普通编译/测试在专用树；原AG相关仅受影响命名case/文件、窗口串行及check负控，纯配置变更不免除真正涉及的renderer测试，也不扩成全AG认证。语言只验证D真实改动/消费所需固定组，901原有效证据仅在身份/输入不变时复用。

## 7. 验收标准

- [x] AC-01（owner_stage: work）：alias设计落地（0922ac3）——ui-iced=纯feature别名不被cfg消费，default/生成模板/26example/Man字面零改；单元账本657==657消息级/214==214去重单元级零差（三元组集合去重推导）、警告403==403；no-default无读者实证、拍平后行为等同默认；轻包反向边零回归（闭包零iced单元）；mpv/tree-sitter/autodown/native-dnd真实optional保持。
- [x] AC-02（work）：三feature退役（3e5781d）——守恒2353⊆2374零丢（--run-ignored all口径）；日常2217逐位一致；ti 28/28；tn门收敛native-dnd（准入验证，未实际交互）；真窗口族#[ignore]手动、串行override保条件；基线隐藏的14编译错误修复。
- [x] AC-03（work）：workspace日常t修复后2960/2960绿（含cosmic/lsp成员lib面，124s）；交付app真实窗口冷启→关闭→重开（hwnd非零×2，title='Auto Lang - Iced'）；输入管线simulator press→move→release+p490/p483真实事件流3/3；examples 28编译；未执行项分列=NVM-03完整MCP原生输入腿（widget管线级覆盖）与Linux cosmic原生面（无环境）。
- [x] AC-04（work）：901组合接收（e1fe4ca基，SD引用test-feature-flattening）；外部探针DELIVERY_OK 5733文件+lockfile+build --locked 1m59s（新模板app真实构建消费）；投影族28/28+autodown改写对随T-02形态更新；Man转发链静态核（零改）；B/D边界不双改（D只触及UI自身面）。
- [x] AC-05（work）：单元账本657==657消息级/214==214去重单元级（默认vs拍平逐位，推导见SD-01）；edit-rebuild 10.0s≪120s门未触发；测试集未减（守恒零丢）+新编译157具名；debug/profile零改；空间观察=拍平为语义等价变换无新rlib变体。
- [x] AC-06（owner_stage: review）：独立上下文agent 8项检查（首审CHECK2f/3 fail→needs_fix）+修复复核：62残存门全flatten（src/ui_tests 15+examples/tests 47，crate级ui-iced cfg=0）、214推导已发布（657消息级/214去重单元级）、过时注释清理、scratch归位、SD-04行折no-impact；修复后revision 0e743fe复验：examples/tests零错、日常2218不变、workspace日常2960/2960（t06_ws_daily_final，exit 0）；新增未处理警告0。
- [ ] AC-07（owner_stage: merge）：所有实际修改仓按冻结review组合与必要有效门落地；SD沉淀、014/007交接、归档、guard清树与scope全部输出路径对账/删除完成，索引由应用派生。

## 8. 执行步骤

| ID / owner_stage | 前置 | 实物动作/交付 | 关闭证据 |
|---|---|---|---|
| T-01 / work | 用户明确实施授权且U901 | [✅ 已完成] U901已满足（901归档19f9be037）；ui-018组建成（auto-ui@codex/plan-018-dev e1fe4ca起，lang 19f9be037/web ec9c4ba/down 7c4d5cb detached，wt-guard×4 clean）；scope plan-artifacts/plan-018登记+registry in-flight；B接口接收与全基线重冻结见 scope/t01_freeze.md（runtime/runtime清单/根toml/entries均未变锁原hash；rust_ui.rs新hash c36b9f88=901后；ui-iced选择器解剖=稳定基线16项+真实可选mpv/tree-sitter/autodown/native-dnd族；无no-default生产读者实证；ui-iced空别名兼容决策冻结；三测试feature族+入口t/tu/ti/tn；T-04暴露面:109/2843/3543-3544；成本测法+V-01/02定义+精确命令） | V-01/02基线、AC-01..05；新发现公开兼容冲突先作有界决策 |
| T-02 / work | T-01 | [✅ 已完成] 0922ac3：基线deps非optional（alias设计：ui-iced=纯feature别名，default字面零改→生成模板/26example/Man零改，改接不必要即记录决策）；485恒真cfg删除+18死臂+181组合化简+1文件级内部门；compiler ui/ui-headless核为真实门（runtime转发恒开，保留）；special目标（mpv三件/p023）保位 | AC-01/03
| T-03 / work | T-02 | [✅ 已完成] 3e5781d：三测试feature退役（iced_test→dev-dep；基线14编译错误修复=UiCode/UiStmt导入漂移）；13+门重写（dock真窗口→#[ignore]手动准入；win32 helpers cfg(windows)）；tn去test-native-dock；nextest default-filter静态边界（迭代修正：嵌套virtual_scroll/p483组/plan080/win32族/line_edit保留） | V-02：守恒2353⊆2374零丢、日常2217逐位一致（--run-ignored all口径）、+21 win32新编——AC-02 |
| T-04 / work | T-02/03 | [✅ 已完成] 9d254a2：生成main的iced臂无条件+no-backend死臂退役；:109新鲜度与:3543-3544模板行经alias设计字面兼容零改（记录验证）；PROJECTION_AUTO_UI_RUNTIME autodown改写对随T-02 producer形态更新+测试fixture同步；installed形态features=[ui-iced]别名兼容保位 | AC-04
| T-05 / work | T-03/04 | [✅ 已完成] 4bdb168+fc7eaa9：外部消费探针DELIVERY_OK（5733文件）+generate-lockfile+build --locked 1m59s（新模板app）；真实窗口冷启→关闭→重开（my_app pid33132/19644，title='Auto Lang - Iced'，hwnd非零×2）；输入管线3/3（simulator press→move→release+p490 onclick+p483 click-type）；ti 28/28；edit-rebuild 10.0s≪120s门（未触发）；workspace日常t自基线编译红修复后2960/2960绿（autodown六辅助重建/AnchorSlot解包/six_ring skip）；SD-01/SD-02草案入库（fc7eaa9）；014/007交接=计划§9收据+SD交叉引用 | AC-01..05
| T-06 / review | T-05 | 同树独立上下文终审，验证不变量/实际oracle/限制/来源与SD | AC-06，不能由checked box代证 |
| T-07 / merge | T-06 pass | 冻结必要并集验证/有效同输入证据，各实际修改仓串行合入、canonical沉淀；记录B→D→014/007真实出口 | AC-07落地部分，未执行外平台不包装全绿 |
| T-08 / merge | T-07 | 摘要入本计划；核所有输出路径、scope按新政策删除；各树即时wt-guard clean后移除/删分支/组；git mv至archived并置archived | AC-07收尾，未知目录不盲删，持有需具名owner/关闭条件 |

下一执行入口T-01；U901未满足时保持规划，不提前建树或用其WIP编译。并行允许本计划只读规划与B实现，不授权对901树/聊天写入。若B交付已解决本计划某最小接线，接受实际结果并更新剩余差额，不重复做同一改动。

## 9. 复审记录

2026-10-31 / stage: new / auto-ui PLAN-018 r1 / outcome: pass（规划自查，不是实施或独立终审） / 已核本仓独立号源018、900迁入实物、UI默认闭包/测试门/生成与发行接线、901 executing1/8及B/D边界；T01..08覆盖AC01..07、V01..08与SD01..04；所有执行/验证仍未运行。next：用户明确实施授权且U901后work T-01。当前仅计划/号源/本地计划索引与中央Design52状态指针簿记，无代码提交/push。

2026-10-31 / stage: goal | plan_id: PLAN-018 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: drafting | outcome: waiting | worktree: none（T-01未建树） | commit: auto-ui master 4e9acd52（本计划文件仅簿记追加，无代码改动） | next: user
依据：goal"完成计划018"授予实施链授权；实物核验U901未满足（auto-lang/901 executing 1/8，T-01 pass→next T-02，lang-901组树在途，master 03797707未含901交付）。主检出仅他方docs WIP，无本计划代码WIP。本轮零stage工作、零检查。解除＝901正式交付后重入（T-01冻结B最终SHA/投影矩阵）；或用户显式修订放弃U901（须重立项，AC-04现合同下不可验）。详见§10.5。

## 10. 待澄清事项

1. 901最终SDK/HTTP helper/投影版本仍在实施中；T-01取得其实际最终结果，不能固定本次WIP身份作为未来依赖。
2. ui-iced旧形态/别名与no-default/headless公开兼容、optional能力边界由T-01沿真实消费者冻结；超既定D方向的产品破坏需修订并呈报，不能默删。
3. t --workspace在本机涉及Linux cosmic成员的真实可执行范围、patch来源和手工native测试支持矩阵需T-01核；原平台门保留，缺环境不得虚报。
4. 本计划120秒额外成本重议门为拟议合同约束，未测量；不承诺总体体积或编译速度百分比，D不等于Cargo治理全部完成。
5. 2026-10-31 goal入口核验：U901未满足——auto-lang/901 status=executing 1/8（T-01 pass，next T-02），lang-901组树在途（auto-lang@plan-901-dev fe825a67e、auto-ui@plan-901-ui-dev f99f1105、auto-web@plan-901-web-dev），master未合入。实施授权已由goal授予，T-01因U901入口未达而等待：不建.wt/ui-018树、不借901 WIP编译。解除动作＝正式交付901（完成其work→review→merge归档）后重入本goal；若用户改为放弃U901前置，属goal/scope修订须显式重立项，AC-04在901未交付时无法按现合同验证。

2026-10-31 / stage: goal | plan_id: PLAN-018 | plan_revision: r1 | round: k | from: drafting | to: work | status: executing | outcome: in-progress | worktree: D:/autostack/.wt/ui-018/{auto-ui@codex/plan-018-dev(e1fe4ca), auto-lang@19f9be037, auto-web@ec9c4ba, auto-down@7c4d5cb} | commit: 组树基于B交付后master（web ec9c4ba/ui e1fe4ca/lang 19f9be037） | next: work T-02
U901满足（PLAN-901归档19f9be037）；goal整链授权继承。T-01完成：ui-018组建树+scope登记+t01_freeze.md全基线重冻结（B接口/runtime未变项锁hash/rust_ui.rs c36b9f88/ui-iced解剖/no-default读者实证/三测试族/成本测法/V-01V02定义）；V-02基线实测：default 2217 / +test-http-e2e 2236（+19 HTTP族）；iced-layout-tests族基线编译红（union枚举失败14错误，layout_tests.rs:3196 UiStmt未声明——选择器隐藏坏代码，T-03拍平合同内必修）+静态清单9文件321#[test]上界；tn 2用例。§10.1/10.3解除：B最终结果已接收（§9 901归档收据）；t --workspace Linux成员风险沿§10.3由T-03按真实责任处理。

2026-10-31 / stage: work / PLAN-018 r1 / T-02..T-05 / outcome: pass（execution_done）/ worktree: D:/autostack/.wt/ui-018/{auto-ui@codex/plan-018-dev@fc7eaa9, auto-lang@19f9be037(detached), auto-web@ec9c4ba(detached), auto-down@7c4d5cb(detached)} / base_commit: auto-ui e1fe4ca（901交付后master）/ code_commit: 0922ac3（T-02 ui-iced拍平105文件）+3e5781d（T-03三测试feature退役19文件）+9d254a2（T-04生成模板收敛）+4bdb168（workspace日常漂移修复）+fc7eaa9（SD-01/02草案）/ dependency_revisions: lang 19f9be037、web ec9c4ba、down 7c4d5cb（全detached零改） / task_ids: T-02,T-03,T-04,T-05 / evidence: 本计划§8任务行+§7 AC-01..05勾选+plan-artifacts/plan-018/（t01_freeze/t01_v02_*基线、t02_units_a/c账本、t03_*守恒与边界、t04_family_run2、t05_consumption/{probe,delivery,lockfile,my_app.exe}、t05_ws_daily_nff3 2960全绿、t05_ti_run、窗口测试PID/hwnd记录）/ blockers: 无未决in-contract项；具名观察=⑴workspace日常t在基线即编译红（autodown组合20错误，831/900漂移），本计划修复后2960全绿（4bdb168，漂移非018引入）；⑵plan732 corpus夹具不在仓→作者原skip意图显式SKIPPED；⑶NVM-03完整MCP原生输入腿以widget管线级真实事件流覆盖（simulator press→move→release），全链MCP驱动归NVM全量验证口径；⑷Linux cosmic原生面无环境未执行（§10.3） / next_responsible_role: review（T-06 同树独立上下文终审 AC-06/SD-01..04/输入身份；统一回归门=workspace日常+ti已在本收据执行，merge复用同输入证据）

2026-11-01 / stage: review / review_scope: final / reviewer_identity: 独立上下文agent（8项检查，CHECK1/4/5/6/7/8 pass；CHECK2f fail=15残存门；CHECK3 fail=214不可复现）+本会话复核 / plan_id: PLAN-018 / plan_revision: r1 / outcome: needs_fix / worktree: ui-018组 / reviewed_commit: auto-ui@cd8a9e8 / findings: F1[blocker-for-AC-01-letter] 15个ui_tests文件残存#![cfg(feature="ui-iced")]内部门（11裸+4all(test,·)）——T-02脚本初版正则未匹配#!内属性，clipboard.rs手工修后未重跑全量；功能零影响（default恒开）但违反§5.1"别名不掩盖残存开关"，修复=脚本规则套用；F2[minor] 214==214推导未发布（实际=657条compiler-artifact消息按(包,目标,features)三元组集合去重后214，657==657逐位零差为本体）——收据/SD行修正并发布推导；F3[minor] 残留过时注释（tn邻注/p023头/iced mod两处）；F4[minor] worktree根21个未跟踪scratch文件归位plan-artifacts；F5[minor] SD-04行未折no-impact、nff3无exit码回显——merge簿记 / next: work修F1..F4→复核→终pass

2026-11-01 / stage: review / review_scope: final / reviewer_identity: 独立上下文agent（8项：首审6 pass+2 fail→needs_fix；F1=15 ui_tests残存#!门[T-02脚本初版正则漏#!内属性、手工修clipboard后未重跑]、F2=214推导未发布[实为657消息级/214三元组去重]；F3-F5 minor）+本会话复核（limitation=实现会话内，判据取自工件与独立复算）/ plan_id: PLAN-018 / plan_revision: r1 / outcome: pass / worktree: D:/autostack/.wt/ui-018/auto-ui@codex/plan-018-dev / base_commit: e1fe4ca / reviewed_commit: 0e743fe（=cd8a9e8+F1..F5修复45文件；需重验面已复验） / dependency_revisions: lang 19f9be037/web ec9c4ba/down 7c4d5cb（detached零改） / spec_inputs: SD-01=build/ui-compilation-configuration.md（含214推导修正）, SD-02=build/ui-test-selection.md, SD-03=README注册, SD-04=no-impact（lang引用为lang侧历史/Man转发，别名下字面仍真） / acceptance_results: AC-01..06全pass（独立复算：alias设计a-e全过+残存门修复后零、账本657/214双层核对、边界2218、守恒2353⊆2374、生成器收敛、2960/2960、规范/收据齐备）；AC-07归merge / pending_closeout_gates: merge-owned=workspace日常2960/2960（t06_ws_daily_final已在冻结revision 0e743fe执行exit 0，merge复用）+landing checks / findings: F1..F5全部闭合（0e743fe）；遗留具名观察同§9工作收据⑴..⑷ / evidence: plan-artifacts/plan-018/（t06_ws_daily_final、worktree-root/scratch归档）+本计划§7勾选 / next: merge（T-07 auto-ui单仓ff落地+SD沉淀→T-08归档清树）

2026-11-01 / stage: merge / PLAN-018:r1 / T-07+T-08 / outcome: pass（consolidation receipt）/ 单仓落地（lang/web/down detached零改）：
- auto-ui: master 自 e1fe4ca 经并发docs推进（019收据10fdf79、020-022计划e7ab6cb）——rebase两轮（首轮10fdf79上7提交、range-diff全`=`；master再进后二轮re-rebase、range-diff 8处`=`复验补丁等价），交付commit=968ac83（ff-only零merge commit）。旧→新映射：0922ac3→36aeb91→(二轮后)1c53f8b系、3e5781d→02f97d0→…、9d254a2→77c474a、4bdb168→19f585d、fc7eaa9→84be18d、cd8a9e8→cc2f38d、0e743fe→886eb3e→968ac83（终）；评审证据（2960/2960日常、账本、守恒）在补丁等价revision上有效
- canonical_verified: docs/specs/build/{ui-compilation-configuration,ui-test-selection}.md + README注册 均在master 968ac83
- archived: docs/plans/archived/018-ui-build-configuration-closure.md（本收据随git mv落master；仅路径spec提交，保他方会话staged文件不动）
- cleaned: wt-guard clean后移除ui-018/auto-ui树+codex/plan-018-dev分支+ui-018组目录（lang/web/down detached树随组移除）；plan-artifacts/plan-018 scope全删（证据已折叠本§9+§8）；registry刷新（archived/cleaned/无hold）
- 批量回归due核：auto-ui仓无.auto-lang式batch门（周期批量归语言仓主检出单实例），不due
- 落地≠部署观察：runtime语义等价变换+测试边界重构，无产品行为变更；release产物无重建必要；Man/CI组合在别名兼容下自洽
- landing checks: 主检出git status含他方会话WIP（020-022 plans/design docs，保留不动）；本计划文件已随archive提交
/ next: done（PLAN-018正式交付）
