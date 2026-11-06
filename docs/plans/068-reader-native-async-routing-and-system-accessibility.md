---
plan_id: auto-ui/PLAN-068
status: executing
feature_name: Reader 原生异步路由状态、结果投影与系统无障碍收口
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 3
total_steps: 8
base_branch: master
base_commit: 7ac0455bf4cd743248749e4bc4eb9d0aed657266
depends_on: [PLAN-988]
supersedes_spec_components: [docs/specs/runtime/component-and-store-instances.md, docs/specs/runtime/vm-route-page-loading.md, docs/specs/runtime/native-vm-app-validation.md]
new_spec_components: [docs/specs/runtime/native-system-accessibility.md]
touched_goals: []
---

# auto-ui/PLAN-068：Reader 原生宿主收口

## 0. 变更摘要

READER-009 已实际完成原生三文件选择/导入与完整字节冷读回，但仍有原生路由页在HTTP续体中缺失宿主路由字段、局部结果状态未投影、系统控件树只有标题栏的缺口。用户明确授权另建框架前置，完成最小修复、独立复审与交付后续009。此计划不降低009原验收，不把消费验收转交本计划后从009分母移除。

## 1. 目标

恢复原生路由子页跨异步边界的正确实例/根态所有权与可见结果；让Reader使用的原生控件及正文在真实Windows无障碍树中有可读身份、角色、值/焦点和正确操作路由。保留同型双实例/跨App隔离、迟到事件、Store归属和Vue兼容性。

非目标：新导航/排版/业务功能、通用语言优化、第三方自动化助手修复、OS安全/隐私设置变更、用MCP状态/DOM ARIA冒充系统树。Sky输入未到达的问题单独取证，不能未定位就归因于应用。

## 2. 架构方案

主责auto-ui。唯一实施树拟为D:/autostack/.wt/ui-068/auto-ui，分支codex/plan-068-dev；开发/内检/独立终审顺序共用。共享Plan留本主检出。仅确证语言公共边界必须修改时，在同组新建auto-lang独立树并遵循其AGENTS/规范；否则语言只读。其余auto-man/down/web用显式环境根和精确pin，不创建junction/symlink。

真实事件→实例限定派发→parked HTTP/恢复→正确child或__auto_root读写→dirty/frame→rendered VTree/真实窗口；系统可访问语义→Windows provider→系统树/焦点/操作→同一生产事件路径。T00/T02先用最小整链证据确定机制，不能先把根态复制进每个child或用root回落掩盖实例隔离。

所有原始材料归D:/autostack/plan-artifacts/plan-068/auto-ui/，登记in-flight；Reader旧反例仍归009，不搬走/删除。关闭时耐久摘要进Plan，再删本scope；消费方需要的固定工具/源码pin须在清理前交接到009自有scope。

## 3. 技术栈

Rust AutoUI runtime/dialect/handler codegen、VM实例及parked续体、iced/winit、已有wry doc-surface。无障碍先调查既有生产布局/语义与Windows公开provider接法；不在本草案猜依赖版本或承诺尚未验证的库。必要新公共依赖/feature由T02给整链可行证据、支持范围和失败行为后再实施；仍在已授权最小原生宿主范围。

## 4. 需求分析与背景调查

**授权**：2026-11-06用户选择“授权最小原生宿主修复并继续”，明确包括auto-ui及仅必要的auto-lang独立工作树、路由状态/结果显示/系统无障碍、独立复审交付后续009。沿原goal继续，不创建第二goal；无用户指定token预算。既有PLAN-988多选/字节已交付，不重开；009终审修复轮仍0。本计划终审修复默认上限3，按记录累计。

主检出基线7ac0455bf4cd743248749e4bc4eb9d0aed657266。当前vm_bridge.rs SHA256=5d76a98fd812fe7af62dae7576c39f259ede9ea409ceda888f9212c22f8d0969、dynamic.rs=ff63380acacfd38957c8772ebc0df1d41b2a599f80c9dda829b633e4ef25e318，与009固定UI02316271相同，因此最新主干这两处未修复。最新Vue057交付不能替代原生证据。现有060仅a2r索引/desktop outproc输入，非当前独立原生VM异步路由/系统树；不抢其树。主检出另有他人文档及4份terminal_pixel PNG材料，原样保留，不纳入提交/构建证据。

规范源：runtime/component-and-store-instances.md §§1～4（子态与__auto_root、parked限制、视图fallback）；runtime/vm-route-page-loading.md（普通child/Init/条件列表）；runtime/native-vm-app-validation.md（真实native与消费）；document-surface-native-input.md（仅实验正文UIA合同，不代表壳控件已支持）；build/ui-test-selection.md/.cargo/config.toml（定向测试档）。

009真实反例，source bd2ed004f178dfadb080d7e3b31a3253fdbe2b6f，UI02316271/lang37350656/man03bb65ef/down58ea33f6/webbf72baab；tool启用现有doc-surface，SHA256=46f7f091a3595a63afb0227b3c67daae73670b5e1ae29112ff7c50a9a6688d31：

- vm-ui-04 CLI46900、UI36232/exec17836、窗口15339060，API11578/MCP11580；用户真实一次选择三本→三次200/code ok、3卡，三份managed字节完整相等。可作为路径/HTTP合法正控，不冒充时延/取消/无障碍。
- import_results实际三行已导入、is_importing=false；公开read-only MCP状态正确，rendered VTree和真实截图没有结果区。状态SHA256=c84508e19c138548931441402cea01f24fbb01b13c5d207eaad99b8371dccd2c。局部/根态或投影缓存归因仍待T00证实。
- 实际打开EPUB：books/progress/toc请求成功后handler_reading_page_Init续体报`Field '__current_route' not found on type instance reading_page_State@1d4abc66b0419902`；生产源随后调用router.push，vm_bridge.rs只在root声明routes时注入宿主字段。所有权/发射确切原因尚为待证假设。
- Sky系统树只有Window/标题栏/系统按钮，无产品控件；系统正文未装载前不能据此判定既有wry正文UIA也失效。
- root最小HTTP await后直接文本与if非空分支都正确显示AFTER_HTTP；否定“全局字符串guard坏”的泛化。009/routed prototype02在parse_event失败（未成有效反例），不得据此关闭/归因。T00改用已合法的plan015/plan041生产夹具建代表路径，先修探针而非让错误探针推动实现。

原始路径位009/t02/vm-ui-04及009/t03/native-status-probe-01/02，耐久细节见Reader009共享Plan末尾。开始work前重新核源与会话存活，不访问其它用户Readest/窗口/资料。

## 5. 详细设计

| Rule / task | 实际路径与owner | 代表状态 / 区分性关闭条件 |
|---|---|---|
| R01 / T00～01 | call_handler_for / pending task保留child实例receiver；宿主路由与Store经__auto_root；dynamic/handler_codegen/vm_bridge具体改面由整链探针定位 | root合法控制；路由页HTTP前/后本地字段与router.push/param、Store调用；同型A/B、退出后迟到、跨App。断言具体child值及根路由/另实例不变，不以root read_all值替代可见结果 |
| R02 / T00～01 | 子页局部结果→dirty/rebuild→实例视图 | multiline成功/失败/取消非空结果与空态；真实rendered标签与状态一致，Store reload不能抹掉/投错实例。不能仅让判断恒真或显示全局root镜像通过 |
| R03 / T02～03 | 生产VTree/真实布局的语义与稳定实例身份→Windows provider→系统查询/操作→生产事件路径 | Reader使用的button/edit/text/list/selected/focus及wry正文在系统树可发现；动作触发对应实例一次，stale/unmounted操作不得落新实例；脱离debug/MCP激活也工作。先确定实际支持/拒绝边界，再实现 |
| R04 / T04 | 新工具/显式依赖→真实Reader原生窗口→picker/阅读/批注/系统树→API/磁盘 | 真实输入与受管数据身份完整，原红不再复现；Sky未响应可具名外部阻挡，不能凭MCP语义树计系统AX通过 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD01 | modify | docs/specs/runtime/component-and-store-instances.md | 既有实例/root合同→明确异步续体局部/宿主字段归属与对应永久反例 | 修共享所有权而非Reader特判 | AC01/02 |
| SD02 | modify | docs/specs/runtime/vm-route-page-loading.md | 页执行合同→HTTP续体导航/结果可见与非法/卸载边界实证 | 页面生命周期正确 | AC01/02 |
| SD03 | add | docs/specs/runtime/native-system-accessibility.md（新） | 无壳控件系统树合同→实际支持的角色/值/焦点/操作、owner/失效与布局边界 | 不混淆MCP/DOM与系统树 | AC03/04 |
| SD04 | modify | docs/specs/runtime/native-vm-app-validation.md | 原native验证→新增受影响的续体/系统树证据、消费pin交付 | 不能以内部状态代理真实可用性 | AC04/05 |

## 6. 测试设计

| Case | AC / task owner | 输入与真实入口 | 区分性oracle / 边界 |
|---|---|---|---|
| V01 | AC01/02；work T00/01 | 新增最小合法路由/实例永久case，复用dynamic_build::plan015_route_loading_tests、tests/plan041_instance_ownership.rs的实际builder/派发；真实HTTP park/resume | 修前红与root/同步绿；子态结果、根route/params、Store共享及rendered文本精确值，HTTP确已完成。测试未来待实现，不预报通过 |
| V02 | AC01/02；work T01 | 同型两实例、同名字段、Store与router交错；owner卸载、迟到与换App | A只改A，宿主字段仅正确root，B/另一App不改；迟到丢弃具名，live实例续作仍绿。沿既有parked同名handler并发限制，不偷加并发承诺 |
| V03 | AC03；work T02/03 | 真实系统树provider可行整链；button/edit/text/list/selection/focus，隐藏/卸载/重新挂载；debug/MCP开关对照 | 系统树role/name/value/bounds/焦点与生产UI一致；真实激活/输入各一次写对应状态，旧身份不得操作新owner。Sky/系统工具不可用则pending，不用合成树或内部action替代该腿 |
| V04 | AC04；work T04 | 固定Reader新native，三文件/逐书结果、打开正文/真实选择保存/回访、系统树控件/正文，独立文件冷读 | 原缺路由异常消失、结果真实可见、book/annotation/source身份与字节不变；现有多选/取消和Vue代表控制保持。完整009其它门仍归009 |
| V05 | AC05；work T01/03/04 | 定向受影响compiler/runtime/tool模块、原实例/路由/表面/Windows provider合法与失败控制 | 原案例不删不减，枚举非零/实际运行数与条件排除分列；未影响族沿同源证据复用，不跑全仓/AG里程碑替代定向集 |

测试入口先枚举具体可执行名。候选固定边界：`cargo test -p auto-ui-runtime --lib plan015_route_loading_tests -- --test-threads=1`、`cargo test -p auto-ui-runtime --test plan041_instance_ownership -- --test-threads=1`；新增case/module由T00/T02登记。handler/compiler改面加入对应命名组，必要tool真实进程case加入；最后merge跑冻结受影响并集。若确改auto-lang，先`cargo t --list`选实际职责完整组与必要集成，明确理由；不凭包构建或相邻绿覆盖语言改面。

实施/内检由本执行上下文；终审为独立上下文同权威树，暂停实施写入后复演V01～04高风险路径和Spec增量；不得同会话自审授pass。用户未指定模型，不预设模型。测试/窗口操作仅自有夹具，保持真实输入/系统无障碍工具边界，不操作用户系统安全设置。

## 7. 验收标准

| AC | owner_stage | 可观察标准 |
|---|---|---|
| AC01 | work | V01/02异步子态/宿主路由与Store按实例/root正确归属，原真实Reader错误消失，合法控制仍绿 |
| AC02 | work | V01/02/04逐书结果/失败/取消状态在真实实例视图正确可见，reload/换页不向别实例泄漏，旧反例有区分力 |
| AC03 | work | V03全部支持表面有系统树/焦点/操作的直接证据，失效身份与非debug生命周期通过；具体边界在T02冻结，不降低Reader009要求 |
| AC04 | work | V04真实Reader消费新工具/依赖并独立冷读保身份/原件，原生页面可继续操作；环境未证项不能当PASS |
| AC05 | work | V05受影响合法/失败/恢复控制通过，Spec增量按实证，未变接口/阈值/公共支持承诺 |
| AC06 | review | 独立上下文对精确revision/commit/cohort核AC01～05与V01～04，全部通过才reviewed |
| AC07 | merge | 冻结受影响并集、精确主干交付/canonical/归档与计数正确；不纳入他人WIP |
| AC08 | merge | 009已收到可消费pin/工具/耐久交接，guard后清自有树/branch/scope，保Reader原始证据与活动会话 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际动作 / 关闭条件 | AC |
|---|---|---|---|---|
| T00 [x] | work | 授权已给 | 主干/pin同因复现、有效最小HTTP routed fixture、真实owner流与R01/02定位；先证明root正控和实际原红，不继续错误prototype盲重试 | AC01/02 |
| T01 [x] | work | T00 | 最小实例/root续体与结果投影修复、永久V01/02及必要同因合法/失效对照；Reader无特判 | AC01/02/05 |
| T02 [x] | work | T00；T01可独立推进 | 真实系统可访问整链有界可行核查，冻结角色/事件/实例/失效/布局owner、可用库/feature与拒绝行为；不可行则具名设计阻挡，不先写虚假Spec | AC03/05 |
| T03 | work | T02 | 实现最小Windows provider/语义发布及真实系统树/焦点/操作V03；按需处理wry正文已有接缝，保原native表面合同 | AC03/05 |
| T04 | work | T01/T03 | 新tool从独立树实际构建，固定Reader消费V04及受影响V05；源/依赖前后不漂移，补Spec提案，work-owned全通过才execution_done | AC01～05 |
| T05 | review | T04 | 同树独立终审，重放高风险原红/合法控制、真实产品与系统树；绑定reviewed_commit/当前revision | AC06 |
| T06 | merge | T05 pass | 精确落地、冻结受影响并集与canonical核验、归档导航/计数；需要Lang变更则同门成对交付 | AC07 |
| T07 | merge | T06 | Reader009更新只读消费pin/工具并记录可用交接；原证据不动，guard clean后清本树/分支/自有scope，最终8/8 | AC08 |

当前3/8，T00/T01/T02已闭，T03进行中。T01真实View carrier红绿闭环已补：14路由/9系统机制/11旧路径均通过，旧读回与真实系统选区证据保留。批注对话须用新二进制继续，T03/04仍未完整通过，尚未终审/交付。

## 9. 复审记录

- stage:new | plan_id:auto-ui/PLAN-068 | plan_revision:r1 | outcome:pass | source:master7ac0455bf4cd743248749e4bc4eb9d0aed657266 | next:work T00/T02 | scope:用户2026-11-06新授权；8任务/8AC/V01～05/SD01～04；不改变009门或终审标准。此为规划交接，不是实现/独立终审成绩。

- stage:new | plan_id:auto-ui/PLAN-068 | plan_revision:r2 | outcome:pass | correction:009是当前可用消费基线，不是本前置的已交付依赖；移除r1误列以消除009↔068依赖环，T00～07顺序/授权/验收与8任务分母不变 | next:work T00/T02。

### 2026-11-06 work入口与T00 baseline

- stage:work | plan_id:auto-ui/PLAN-068 | plan_revision:r2 | outcome:partial | worktree:D:/autostack/.wt/ui-068/auto-ui | branch:codex/plan-068-dev | base/code:05c62e4fd3b6bc0d73985c21346ca420f3bb92f8 | next:T00 baseline实际终态。开发树根/分支已断言；主干原4份terminal_pixel PNG/他人文档保留，未纳入。四依赖只读detached：lang3735065684968aad958140250e763ecea9838aa7/man03bb65ef7fb6181caa6a08d9752ac7d57c75516c/down58ea33f699efa5cb1194473a2d45dcec51006842/webbf72baabf28fa865b5a7c680b8a5fa4018a0a63d。
- scope为D:/autostack/plan-artifacts/plan-068/auto-ui，已登记唯一全局plan-artifacts/.retention/policy.json scopes字典，未覆盖别owner；第一次按list形态的注册断言失败且未写入，随后按实际dict受锁更新成功。五仓worktree创建均exit0，无junction/symlink。除本Plan外证据均在该scope。
- 实际源定位：handler_codegen::rewrite_stmt_with_locals将router.push/back及NavCall直接写__state隐藏字段，而既有实例合同要求宿主字段经__auto_root；dynamic::poll_parked_resumes只置dirty，未沿同步完成路径失效memo/处理导航后继，是否造成结果缺失待实际case证实。旧probe的outlet{}与合法plan015的bare outlet不同，新永久fixture沿生产合法形态，不用旧解析失败作原红。
- 新增tests/plan068_routed_continuations.rs与fixtures/plan068_routes/src/front三文件：真实延迟HTTP root正控、child本态+条件视图、child同步push、HTTP后push/params、HTTP后back/history；均尚未计PASS。baseline命令cargo +nightly -Z unstable-options test --offline --locked --lockfile-path <scope/locks/ui-minimal/Cargo.lock> -p auto-ui-runtime --test plan068_routed_continuations -- --test-threads=1 --nocapture，stable rustc/rustdoc显式固定、独立scope target/TMP；源before清单与log已开始写入，不复用009目标目录或改只读锁。

### 2026-11-06 T00实测完成：root正控1绿、真实路由4红

- T00永久case提交**7171ac31**，生产库仍与05c62e4f相同。首次完整冷构建已产出测试exe；第一次运行中的root HTTP正控因Windows accepted socket继承listener nonblocking而WouldBlock，析构二次panic导致STATUS_STACK_BUFFER_OVERRUN，该为测试夹具错误。显式set_nonblocking(false)、Drop不在unwind中二次panic后，baseline02真实终态**1 PASS/4 FAIL、0 ignored、exit101**，5项全部实际执行0.65s；编译14.22s，log SHA256=212bee8683063ca8fb397932c98eac9b1c9890a91d5976f6e147f1f93323c37f。
- root HTTP await→可见结果正控PASS。child同步push与HTTP后push分别重现子态缺__current_route；HTTP后back重现缺__nav_back_pending；child多行结果在自己的实例态已精确成功写入，但条件/正文VTree仍空。这与Reader真实已保存而未显示的反例同因，排除接口未返回/字节失败或root字段代理的归因。合法bare outlet夹具已通过实际production builder，旧prototype解析错误不再作依据。
- 已开始T01最小改面：handler_codegen宿主导航写经__auto_root，dynamic统一完成后继（参数/历史、memo与dirty），初次t01-route-fix-01为**4绿1红**；剩余back已从子态缺槽变为App_State缺__nav_back_pending，明确root宿主槽未注入。正在补root routes专属false槽并重跑同5项；未宣称T01完成或native产品/AX通过。无语言源码改动。

### 2026-11-06 T01闭环与T02整链核查

- T01代码提交9cb9722658d2bda61273837844091de3aa895c39（权威树codex/plan-068-dev）；child的导航写经原receiver的__auto_root，routes root补false back槽；HTTP/CPU完成统一发布route params/back/memo/dirty。有效新增边界先红：push→HTTP back仍停done；卸载→同key重挂后旧HTTP仍改root。修复为handler push同样进入50项返回历史、parked task显式绑定派发receiver堆id，卸载前取消其凭据/栈/busy、移除该receiver排队写。未改lang、重入键或阈值。
- 永久target plan068_routed_continuations最终**11/11 PASS，0 ignored，exit0，1.30s**：root HTTP与root自引用导航正控、child本态多行条件投影、同步/HTTP/CPU导航参数、push再back、双同型+共享Store/跨App、HTTP迟到重挂、CPU卸载；强制真实HTTP延迟与CPU max_steps=1让出。log t01-root-navigation-control.log SHA256=134fafb7d2bb74cebe4245fdb80c6f5ee800ed98d5858e834f53865b97bcbe58。旧实例target4/4 PASS、handler_codegen::tests15/15、plan015_route_loading_tests6/6（均exit0、0 ignored）；分别log SHA256=cd8f57a5f14ca0a51517ffb0eddec19d996495831eddf48d730e0c03ab3943e0/49bf96ff5d86220d3e35a502cf16fb4e2c520a7741ef005348409f191d35d8e4/c63def1b0a8840e2f7f4629c71e198b496391615cee3d575eaeb59a5970148bf。所有log在本scope。代表既有合法/失败/实例路径覆盖本次改面，未跑全仓；冻结并集仍归merge。
- SD01/02仅在实施树备好，尚未发布主干canonical。真实Reader缺路由/结果反例仍需T04消费新tool实际复演，不能凭组件绿标完整AC01/02通过。
- T02候选AccessKit官方Windows adapter0.35.2（accesskit0.25.2/consumer0.39.2、Rust>=1.87），公开SubclassingAdapter在窗口线程首次show前安装，action可跨线程故只入队，update返回events须在锁外raise。独立scratch winit0.30.13真实小窗零MCP/debug：build exit0，15.34s。Sky读取实际Windows树，按钮7、edit8、text9均出现；系统set_value中文→实际model、显示与AX值/焦点均更新。系统证据t02-system-tree-value.json SHA256=ac01ef670d91d6f4b6a07ed1587de3148d2275465b8f8188c585718d5c21436c。Sky click按钮仍0，不能称按钮操作通过，保留与Reader同形的输入工具疑点。
- T02下一步：把生产Iced布局、精确实例代际与系统action路由接法冻结，避免复用debug-only包装造成失焦；bounds必须真实布局且按窗口scale映射，stale node须丢弃，wry原生子窗保留自身provider。新库尚未加入生产manifest，无虚假AX Spec。

### 2026-11-06 T02实现接缝冻结（可行核查仍进行）

- Windows原生VM独立窗口先hidden创建，在window::run保证的窗口线程、首次show前安装AccessKit SubclassingAdapter，再恢复原visible意图；原pixels/hidden会话不显示。非Windows/其它宿主不预发新增能力。
- 生产render_dynamic_view使用恒定结构的零布局委托wrapper采真实bounds/clip/focus；不依赖F12/MCP InspectorCache，不开debug包装。每窗口的语义树随真实转换树建/撤，node id按path+原receiver堆id+事件稳定，移除后不复用；换代即新id。Tree/queue只承载Send数据，不跨线程访问VM。
- Windows activation从已建树快照返回，未建树时None，后续实际帧发布；action只入队，生产update实际消费时再次核node、原receiver存活及代际，再走原IcedMessage→实例派发或既有Focus operation，禁止直接写业务state。按钮/可点击行、input/textarea、静态/正文文本、checkbox/radio、list/group是首批Reader表面；Select/第三方定制控件不得伪报可操作，先保持具名未覆盖直至Reader消费核查决定是否必要。
- bounds以真实layout/draw和viewport裁剪为源、每窗口DPI映射；字段值/禁用/checked/selected与实际转换View同源。wry正文保持自身Windows子窗provider，父树合成须在T03/V03实际核对，不用DOM/MCP树替代。
- 为避免空闲定时泵，新provider使用每窗口有界合并的唤醒通道：实际树/bounds变化或系统动作才通知update，事件raise在快照锁外。关闭窗口清语义和动作，adapter仅在其窗口线程释放；多App/卸载/重挂/resize/焦点/非debug对照永久测试。
- 尚未将T02打勾：仍需生产接缝的小窗实际整链（含首次show时序、真实bounds/动作）；scope和009验收不变，独立终审仍未启动。

### 2026-11-06 T02关闭与T03当前检查点（未提交终审）

- T02实际生产整链：新增独立example plan068_system_accessibility和双同型Probe夹具，通过真实run_dynamic_iced→hidden open→window::run安装→首次show。第一版build exit0、34.10s，binary SHA256=1ecf1fe00dc80e2223fdde0bea7bb4554b31ffeacaec9ab3bbac3642014e3aae，PID2924（已按完整exe路径确认后停止）。实际Sky系统树有两个按钮/编辑/文本，系统set_value触发正常实例handler；不把internal树当系统树。
- 发现且保留真实新反例：甲状态/正文已改、乙状态/正文仍initial，但输入显示缓存按Edit事件共用让乙输入也显示甲值。before-cache-fix.json SHA256=5bd600add5588802344bc2e904931bcd7d8cf933a9d5aec3ca30ef9b6a105495。T03对同因缓存改按原receiver堆id+event，retain按基事件/存活receiver清理，patch按实际TypedInst绑定；旧根态裸事件接口保留，不改ADE不覆写合同。
- T03机制首轮native_accessibility::tests实际**4/4 PASS，0 ignored，exit0，4.41s**；生产Iced headless真实layout/draw/viewport边界及1.5scale、原实例动作、双App拒绝、卸载前/后旧node拒绝与新节点可作、输入缓存隔离/旧代移除。log SHA256=db66dffda1d77e2d224e9a918103b113977edaf931b46fa376bd1e16b9c1b633。此非系统操作成绩，不代替V03。
- 新版生产例build02 exit0、31.34s，binary SHA256=2e31d0d30f1e30f5f5301875fc724fab99284f90bd4c9f5fbd1643346c8203d1，PID15916/窗口3147796（当前新窗口名称P068 Native controls，以Sky新返回对象为准）、MCP11595，P530_NOMCP=1/AUTOUI_MCP_DISABLE=1，未启F12、未发MCP读取/action。实际系统set_value中文/引号/反斜杠→甲输入及正文正确、乙输入及正文保持initial、焦点正确。isolated-value.json SHA256=0101fc00cc6c465f56f63b129390c853fbc5348ea7289b2b4146ee04c5bf5987，jpg=387952ece19affcde8c743a4b4bc4592f059bba5e2011dbf36582a78f85038c7。MCP服务器仍由既有宿主启动，关闭的是捕获/轮询，不声称零server进程；新AX全不依赖其数据。
- T02选择依赖accesskit=0.25.2/accesskit_windows=0.35.2（Windows target精确pin）；实际consumer0.39.2/windows0.62.2，与既有windows0.58用raw HWND隔离，不改变其它平台。独立external lock更新SHA256=f8e51e1c6587604994b452a63d856b7fa7a821610444f0d32a7b16305e07be5d（后续tool feature可能正常再更新）。无源码/新依赖写入主干，只在权威ui-068树。
- T03尚未关闭：checkbox/radio/list与disabled/非可见拒绝、focus键控换位、关闭资源、真实wry正文合成、按钮实输入/原Reader均未满证。当前补生产List递归、语义widget位置复用时的owner校验与焦点重取；新tool（autodown+doc-surface）构建中，新的改动尚未重跑机制集。旧输入pin/ADE控制与受影响并集仍待对应阶段。不递交未完成实现给终审裁剪验收。
- 工具约束：Sky click生产按钮仍无变化；系统set_value可达。首次误向已关闭旧窗发手动请求已立即取消说明，不计成绩。当前另一次明确请求针对已确认可见P068 Native controls，保持PID15916窗口等用户手点First Increment与ToggleB两次，独立代码/构建工作继续；无新授权待答。先前一次example重新link因同名exe仍运行而LNK1104，已确认旧PID/路径并停止后build02通过，未称编译代码红。

### 2026-11-07 T03/Reader诊断与T01真实carrier缺口

- 当前权威代码573c0d460d67e8563a7645982117e1ff3817d267（原路由9cb97226之上，系统provider/持有原owner/按实例缓存/永久例）；仍executing，未独立终审。新增生产首次帧反例先红：完整DesktopSession::empty(None)→allocate/register→dynamic_view、零MCP/debug且不预渲染，AX未绑定子件（0目标，exit101）；根因owner快照取在View分配child之前。移至实际render入口后，连关闭后旧请求不复活registry一并加入，**native_accessibility::tests8/8 PASS、0 ignored、exit0、6.75s**（autodown/doc-surface同消费feature）；基线log SHA256=e5961a1b514126935add1ed78ce3898665d866eef2aecf5361aff8afa68eff6c，修复e1c86dae8a9e9b6d8736f12b5865367d50982281912eefa139c18a446bceab50。此前6/6及真实input pin/ADE旧3/3（无跳过，已核corpus存在）SHA256=45142dc7a4470bd57fade96551168b2b2bd88ef296cbfe5b54a3bf988dc7a65f/13e1cc0f9c49c93f126a0a5a802db957c0ca42559f9b4269ef7210ddc40d718c；未受之后scope放置/registry改面影响，复用不重跑全仓。
- 用户实际回报First=1，系统树冷读确认甲1、乙0，甲原中文输入和回显不变，乙重挂initial；乙系统元素索引14～18变19～23，甲身份稳定。lifecycle.json为有效系统读回；同名jpg实际抓到遮挡中的Reader，不能当计数窗视觉证据。独立手动回报与AX值可计原生输入对照，非MCP按钮事件。
- 新tool启autodown+doc-surface已build exit0，2m03s，SHA256=2b5b027f2061599ecb9dde1f3025a01f211235603b0b866d33a7ab062c0655ae；该诊断二进制早于最终573c0d46的首次帧修复/测试，不宣称最终冻结cohort。Reader a78c5cf源码未变，显式根指ui-068，其余依赖仍原pin。诊断01（CLI33068/UI28016+45276）复制数据后managed_path仍指009；已核完整exe/PID子树停止，无验收成绩。02先递归本地化原件/managed路径并全3份hash相等后启动，CLI16544，API7100/front7101/MCP7102，原生窗口18550458、title图书阅读器。data与fixtures全部本068自有，009原证据不改。
- 隔离夹具曾json.dumps加入空格，触发既有固定meta前缀守卫而目录[]/内容缺失；仅恢复自有3份索引紧凑格式并修启动脚本，接口已返回单章“封面书甲”。未为测试准备错误改Reader源码；当前已开阅读页标题仍旧，待正常重进，不计标题正确。原件/managed字节不改。
- 按既有NVM-03从真实MCP树发现书卡并press（非直接API写）→真实页面Init HTTP链完成，原缺__current_route异常不再复现；Windows实际树同时有壳button/edit/text与wry子进程Document，标题“封面书甲”和正文“封面提取验证章。”可读。Sky父窗拖选被明确拒绝`point (777,710) is over msedgewebview2.exe Chrome Legacy Window, not target window auto-ui-tool.exe`，激活/刷新后一次重试仍拒绝；子进程未在工具窗口清单可单独选择，未绕过工具限制。
- 用户已真实拖选整句并回复“正文已选中”；系统selected_text精确相等、独立GET locator found=true，book_id=18e579d2-a510-4fc1-a77b-76b49f829df8/source sha256:c4f323fc.../spine1/c1.xhtml/range start CFI epubcfi(/6/2!/4/4,/1:0,/1:8)/created_at1793980545。保存系统/原生API证据于02/real-selection-*.json。截图在用户回复后抓到遮挡Codex，不能冒称正文选区蓝色画面；以系统selected_text+实际adapter POST/locator冷读为真选区。API提案额外end.cfi="0"值得009后续严格核查，不预报保存/回访通过。
- 新反例：真实树“批注所选”press返回ok但无对话/无locator出站HTTP。读取生产Send链定位：dynamic::on_with_input_for_instance先split_once('@')再剥U+0001，误把实例内的reading_page@/book/...截断；该形态的live名字回落又不能从路由路径尾段辨件名，遂落root legacy，在root.is_epub=false直接返回。旧11项使用显式instance参数，未覆盖真实TypedInst→Send carrier，不能关闭此分支。T01重开（任务分母8不变，非终审repair轮）；新增真实View绑定HTTP与key含@的双实例永久反例，先跑原码基线，随后只修解析顺序，禁止复制root/改Reader规避。
- 用户手动两项均已答，不再悬置旧问；下一步由实现继续，暂不要求重复选区。最终新二进制/准确源、必要复演、Spec增量、独立复审/交付/009消费均仍在后续，不降低原门。

### 2026-11-07 T01真实carrier修复闭环

- 初次carrier基线在正常lib构建暴露本轮cfg(test) legacy patch_input_values的导出未同门，E0432；已把仅测试用再导出一并门控，未把编译失败计原行为反例。随后实际基线02为**11 PASS/2 FAIL、0 ignored、exit101**：路由HTTP确已完成却写root旧副本/child仍空，含@key同型双实例在途未建立；log SHA256=cee736720244c7e30c25f605fb455a7f3724753d8050b0eec4aec31258879279。
- 代码提交272c0e1b463be2974a801e4e5ae87336a6e50e89：先按U+0001剥件类型/不透明instance，再只在类型段解释@seq；显式instance覆盖carrier但类型仍剥离，完整实例地址不再截断，不加root状态镜像/歧义回落。新增真实View→Send的HTTP/含@key对照、显式instance覆盖+类型@seq正控；另9项系统机制包含完整生产dynamic_view/convert_view_messages实际路由按钮 carrier→系统动作入口→child本态，root旧副本保持空。
- 最终定向结果（autodown/doc-surface同消费feature）：plan068_routed_continuations **14/14 PASS，0 ignored，exit0，2.00s**，log SHA256=fd5578a78ab4ddf691fa8be1f43eec876f5d7da6e971b87a664a8583aeb14bad；旧plan041实例4/4 PASS（前次修复log，解析相同范围仍绿）；native_accessibility::tests **9/9 PASS，0 ignored，exit0，8.19s**，SHA256=ea994b60faee104ab49c1b6bd1786295deeaabc84df850dd21c23ccb496e6046（去空格）；旧plan654路径/时钟 **11/11 PASS，0 ignored，exit0，0.41s**，SHA256=ac567222f7fdc5f5e3648983c201015580802218da4624953edf7c41634490b0。未重跑全仓；统一冻结并集仍归merge。
- T01重新关闭3/8；源/Spec仅权威树已提交，当前运行Reader02仍旧2b5b027f诊断binary，必须重建/重开消费解析修复后才继续批注，旧失败不伪称通过。两个用户真实输入都已答且系统读回有效，暂无权限/新范围问题；终审repair轮仍0。

### 2026-11-07 新consumer真实批注保存与系统树滞后反例

- tool从272c0e1b精确clean源重新build exit0/44.74s；SHA256=d6261209eb9563d3b2ae68993a79649ec22a420a3f9c750998286bc6cc666e58，build log 7aa65205a93830c5c438caa99c2837d86bb5d61b85a49d8b81989d5137fd655c；锁仍f8e51e1c...。Reader源595eb627（正文标题/失败提示修正）。新03独立data/fixtures，CLI44060/UI30524/exec3576，API13957/front13958/MCP13959，窗口35916452。3份managed/原件本scope内字节相等，cold-read SHA256=a42bc7c714925e51240e7bb109f15970d734fd6dc738e5f9b1335d2b2da7721f。
- 实际Windows树读到正确章名、阅读正文Document与正文；无选区press真实进入child HTTP并显示未捕获提示。Sky element click先报缺geometry，补真实截图后一次click仍无响应；NVM-03公开MCP发现当前书卡/按钮后press有效，未冒充物理输入。用户新会话再次真实拖选并回复新版正文已选中；系统selected_text精确封面提取验证章。；新SID r7-1793982468725，locator created_at1793982839。系统选区SHA256=04c4fc69f35fb284423182547d5ea3ff5f747c18cdbd8c0dd4747989fe349ced；locator SHA256=6e4fab3d6fd9ba03edb6e3be07eb1c05c95c2a28a749ba8bd6cadb038af28909。
- 实际批注对话已可见（不是仅MCP状态）：按当前控件NVM-03输入“原生批注：中文 \"引号\" \路径”和四标签并保存。磁盘version2唯一annotation_id18e58207-c9dd-4797-97e3-982c0f2fa068，quote/comment/tags/locator逐字段断言一致、原件身份不变，原文件SHA256=2b5751caf5b0594179161e51d4fa647b44c207d1e3d9716e8d85951d00f080b4；readback c337c445bd802815245d2ce3b5413b40140c402fd60ba76d1ed746a7757176a9；有效实际保存截图1461cd28795afcb885fab4d5cbb3a8e9205c53ef79487f3337541fa2f7a37cfa。回访实际verify-ann anchored/match=true，UI提示v2锚定位匹配；当前Reader仅发goto spine index0，本书短章不足以证明长文CFI精确恢复，留009原门，不外推全W07。
- 新系统树反例：画面/当前MCP已显示批注框，Windows树仍为前次未捕获选区；保存后画面已显示注释抽屉/新comment，Windows树却还在旧批注框且输入空。after-save-system-lag SHA256=ba26130aee6c37a256bdd2a6efd7be101ba92fad9b0dd1d56404f8f609bad24f，与有效像素/MCP成区分性对照。T03/04仍pending，不把有控件旧树当实时系统通过；开始在自有provider临时trace定位发布时序，尚无最终归因。
- 内检额外真实红绿：释放B后只剩同型A，延迟converted显式B被provider bind误回落到A。新增永久实际layout/Click反例baseline **0绿1红，exit101/1.39s**；fix237ebdd9cd43a8502932c0e4e9f967e5fac45168仅禁止已带identity的名字回落、未绑定控制不授Focus。完整机制 **10/10 PASS、0 ignored、exit0/10.29s**，含新反例Click/Focus拒绝与A合法正控。baseline SHA256=9df7d84c7ca758aa1337825e0e78d224d2b6e116d4a571b077ee5b7e64bed067；final16de34e05a22021b85417fc89b2aad6df71340ff12f2311501fd362fe0aa6b75。SD03/04草案仅同权威树提交；此不是独立终审修复轮。
- 03已核完整自有tool/VM进程后关闭，CLI随断流退出；未递归按PID祖先误杀，Windows PID重用可能把无关旧进程列为子代，实际只按完整exe/命令和自有WebView profile关闭。03真实保存数据继续在04自有副本重开，不能拷贝旧提案假造新会话选区。当前没有待答问题，继续T03发布链诊断，3/8不变。

### 2026-11-07 provider重复刷新闭环与系统SetValue正控

- 04临时trace真实native显示Provider确实构建/发布了当前编辑字段；6,060,049bytes实际记录built8427/tick8424/publish8422，证明静态画面仍自激刷新。trace SHA256=73b6f3a73b549ed1c52bfce5d1364388d52eb765af7ab40a51692ec7f7793c2a。本证据并不能单独把03系统树滞后归因到provider，保留工具缓存/时序未决；没有为猜测重写整个renderer。
- 新永久unchanged_production_frames...实际baseline **0绿1红、exit101、2.07s**，不变布局/语义仍wake；hash166d7dfa5bc5bca1fbba719ecd7e126a53968a9129f6b598d92dd958228755c0。修复cd8b07756870746748b17cf5affbc497db3eddad：仅结构/角色/值等语义变化重新wake，真实bounds/focus/name变化仍发布；同一帧保更新handler/owner快照，未加空闲timer。永久正控值变→wake、再次不变→settle、卸载→wake。机制完整 **11/11 PASS、0 ignored、exit0/12.43s**，log SHA256=d250012995b38f58d1f09ac67ef73062ff4e58bc1546aff61a477a8bd4ea1f02；随后rustfmt仅格式变化。临时P068_AX_TRACE已全部移除，未交付诊断输出。
- 04系统Sky SetValue第一次AX-only缓存报element unavailable；重新hydrate/激活并截图+AX观察后一次输入实际成功，立即refresh暂null，重新观察系统值与当前MCP值精确一致。system-edit-value SHA256=245a8fd7f5d17f8314501caf371ccf80724098fa950ca048f70e9d170a8da14a。正常NVM-03保存后，磁盘comment为系统中文编辑核对含引号/反斜杠，annotation_id/created_at/kind/chapter/para/quote/occurrence/tags/完整locator逐字段保留；cold-read SHA256=1051799317aaa140c91a5b97d8279631f37a2a760d56eba88a3fd65ebe349941。保存后系统观察遇窗口已最小化，未当FAIL或PASS；当前04按精确自有tool/VM/profile已停，原数据保留。
- cd8b0775 clean源tool重新build exit0/38.58s，消费05将从04真实保存数据本地化重开验证当前更新/编辑，不需再次拖选。T03/04仍待实际cohort和角色边界收口；当前3/8、终审repair轮0，无待答用户问题。

### 2026-11-07 当前consumer06闭环与正文卸载修复检查点

- 05启动在0.0.0.0:2688出现10048，尚无native窗口；不算产品PASS/无障碍FAIL。核CLI31008/tool32496完整路径后停止，未杀端口其他进程。06改用三个预检独占空闲端口33941/33942/33943；CLI44388，窗口50856836，Reader595eb627/UI cd8b0775 clean工具d4b08489ff946642938bf5dbbeda5b119d0cd7ab347c1e46b0048a9e1fd601a5，真实04保存数据全路径本地化。当前06仍在运行，系统文件选择框已开，不能在同名exe运行时重链。
- 06系统冷重开直接读到已保存comment/quote/章名/正文，reopened-annotation-system SHA256=5cdb86b81dce2434b85c6a134b917a9ecb3aa074938a105b126a23029a628afc。真实NVM-03回访→等待实际HTTP续体完成、未发后续输入→Windows树已出现“已回访：v2锚定位匹配”，SHA256=a92007d41cf1ec605deaf363ba8fdffc8c23f26103649ea83cea857e8450c065；编辑框的当前值/四tags可读，取消后全部编辑控件消失，cancel-edit-system90389beba546caecc9388384a2405a05f87d7ce171cfc340bd01543bca919f3a。此具体异步系统更新/移除腿已通过，不外推全表面/所有时序。
- 06系统SetValue两次都在工具缓存层报element115 unavailable：第二次已重新hydrate当前窗口、截图+AX重新观察，未继续盲试或把错误归产品。04真实系统SetValue和独立落盘仍为同源action路径正控；06不能伪称新系统输入成功，已用正常控件取消未改数据。
- 新真实原生红：返回书库后，旧WebView/归属popup依然可见，白块覆盖书卡；系统树也保留旧正文。随后系统picker前景截图仍可直接看到残留，SHA256=ef5c6c64dffc41f2015dae629da7eb4e2e0341bd04f1971bbd7941a4449a1753；对应系统82145f900f05d6e6565a154fad8fb7a93e03d197462c04fb2580ed8c30d868c7。根因为doc_surface只在仍绘制的painter处理空URL/出界，组件消失不再draw，无卸载隐藏。
- 修复fbb65dc2fbe40de235e0055ace403d0313025fc5：真实Iced canvas State持有挂载lease（painter每帧重建不可作卸载），最后lease卸载同时隐藏webview/popup，key变更释放旧lease，普通重建不重复计数，同key另canvas仍挂时不误隐；显式release先隐藏再移除。保留原有进程内WebView复用语义，无定时器/Readest特判。2项永久机制实际PASS/0ignored/exit0，log SHA256=3440c0779d6f65eaa7b08ef2759f77a3b1aa0a0c483e4f987268beeef87b711a。此是空HWND机制对照；**新tool真实返回书库不残留腿仍pending**。
- 9aa7ab730a97ac5f81554faf7a169d103e767700扩已有真实生产fixture：checkbox选中态、multiline、disabled控制。新增fixture后完整11项机制PASS/0ignored/exit0/11.70s，log8ffa0ebe9eea41891fe03c3faf6e7b486fda8630d4c9e45583826d566d2d1c48；example build exit0/43.65s，尚未启动新角色窗，不能预报系统角色已证。旧counter15916按完整exe核后停止（其已答真实First=1证据保留），不冒充正常窗口关闭验收。
- 原生06 NVM-03当前树发现“导入书籍”后press，真实系统打开框出现；filename236的Sky SetValue同样报缓存不可用。已请求用户进入06/fixtures实际选择中文书2/笔记10/w03-cover/坏书4项，验证3重复+1坏书逐书结果；当前待答，选择框保持打开，无新范围/权限问题。用户输入前不改该活动窗口。实现树9aa7ab73 clean；T03/04未满证，仍3/8/终审轮0，下一执行者仍本work上下文。
- 独立原参考准备只读核验D:/autostack/readest HEAD精确3cc39ad7、无WIP；未启动用户E:/soft/Readest、未复制其profile/cookies、未建reference worktree或执行上游worktree脚本。该脚本会fetch/rebase origin/main、复制.env及创建junction，与本轮固定pin/只读/无junction约束冲突，不能直接运行；独立参考仍归009原门，未计通过。

### 2026-11-07 用户四文件已答与结果显示边界

- 用户明确“已选四个”。原生06实际PickFiles续体4次POST200依序w03-cover.epub duplicate / 笔记10.md duplicate / 坏书.epub epub_invalid（缺META-INF/container.xml） / 中文书2.txt duplicate，handler最终644.5785ms恢复。实际选取路径为009/t02/vue-ui-01/fixtures，不是提问中的06副本；已独立断言全部4份字节与06副本相等，输入均自有造书，未改原文件。磁盘书目仍3本，全部受管原件与各自original hash不变。
- 当前rendered MCP已准确有四行结果，four-picker-result-mcp SHA256=ad38cea5e758fb93fbbd3cebf835d6524082fb23d7411c92b001c13ffb9df359；Windows/真实像素仍未显示结果，并有已知旧doc popup遮书卡。cold-read SHA256=9234361f18c71418aec1e8d7c6691864fe7fb51eabcdbe3e2939aac8ac9f64c7；系统48e8c2cb2612b2ed564b56d4d7103a41e868a111ed106a5c632ce859eeb60379；真实显示差异图ae8c47255549cf3c56e736c785096d49b54802041ce54ecbbaf402c0d628cf1b。ViewList新动作真实改变卡片为列表但仍无逐书提示；Node/Sky会话重建后当前截图同样如此，不能仅归为工具旧缓存。旧06 CLI44388/tool9944/exec14968已按完整exe/命令与自有WebView profile停止，原资料全部保留。
- 新9aa7ab73角色例已真正启动PID42772/窗口77728396/P068 Native roles，binary e53b8756b4ae49c0827653fb9899bb07b8413e774ef72c14026fcfbea037a04e，capture禁用且未调用MCP。系统树/实际像素可见Checkbox已选、Multiline双行、Disabled action禁用；系统SetValue First中文引号反斜杠→仅甲输入/回显变、乙initial；系统SetValue Multiline→实际两行/系统值及焦点一致。Sky checkbox element点击无效；另坐标对照在正确截图后报failed to activate captured window，重选/激活一次仍失败，进程确认仍live，未继续盲输入或把click算PASS。
- 结果显示定位先做实际生产layout oracle：单root固定高、自动高+cap四行文本实测均可见，不支持“所有max-h滚动容器都会零高”的泛化。下一最小原生fixture覆盖实际App侧栏/行/子实例、空→事件四行、MCP tracked/no-capture及物理bounds；首次inline style语法探针被parse错误挡住，不计产品红。已换永久.at夹具，当前只改测试、尚未改布局生产实现；T03/04仍pending，3/8/终审轮0。

## 10. 待澄清事项

无待答范围或输入问题。用户已于本轮回复“已选四个”；真实trace为3duplicate+1epub_invalid，磁盘仍3本且原件完整，实际选取源为009自有fixtures（与06四份逐字节相等），未要求重选。06旧工具进程已核精确PID/命令关闭；当前work继续原生结果物理显示差异、已提交正文卸载修复的真实消费和系统角色/动作剩余腿。工具输入/激活失败不得绕过安全边界或伪造输入；完整T03/04、独立终审和交付均未通过。
