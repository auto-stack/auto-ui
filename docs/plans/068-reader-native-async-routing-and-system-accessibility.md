---
plan_id: auto-ui/PLAN-068
status: drafting
feature_name: Reader 原生异步路由状态、结果投影与系统无障碍收口
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 2
current_step: 0
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
| T00 | work | 授权已给 | 主干/pin同因复现、有效最小HTTP routed fixture、真实owner流与R01/02定位；先证明root正控和实际原红，不继续错误prototype盲重试 | AC01/02 |
| T01 | work | T00 | 最小实例/root续体与结果投影修复、永久V01/02及必要同因合法/失效对照；Reader无特判 | AC01/02/05 |
| T02 | work | T00；T01可独立推进 | 真实系统可访问整链有界可行核查，冻结角色/事件/实例/失效/布局owner、可用库/feature与拒绝行为；不可行则具名设计阻挡，不先写虚假Spec | AC03/05 |
| T03 | work | T02 | 实现最小Windows provider/语义发布及真实系统树/焦点/操作V03；按需处理wry正文已有接缝，保原native表面合同 | AC03/05 |
| T04 | work | T01/T03 | 新tool从独立树实际构建，固定Reader消费V04及受影响V05；源/依赖前后不漂移，补Spec提案，work-owned全通过才execution_done | AC01～05 |
| T05 | review | T04 | 同树独立终审，重放高风险原红/合法控制、真实产品与系统树；绑定reviewed_commit/当前revision | AC06 |
| T06 | merge | T05 pass | 精确落地、冻结受影响并集与canonical核验、归档导航/计数；需要Lang变更则同门成对交付 | AC07 |
| T07 | merge | T06 | Reader009更新只读消费pin/工具并记录可用交接；原证据不动，guard clean后清本树/分支/自有scope，最终8/8 | AC08 |

当前0/8，所有探针/新测试/实施均待work；已给授权下下一依赖就绪任务T00/T02。没有以本草案新建worktree或修改实现。

## 9. 复审记录

- stage:new | plan_id:auto-ui/PLAN-068 | plan_revision:r1 | outcome:pass | source:master7ac0455bf4cd743248749e4bc4eb9d0aed657266 | next:work T00/T02 | scope:用户2026-11-06新授权；8任务/8AC/V01～05/SD01～04；不改变009门或终审标准。此为规划交接，不是实现/独立终审成绩。

- stage:new | plan_id:auto-ui/PLAN-068 | plan_revision:r2 | outcome:pass | correction:009是当前可用消费基线，不是本前置的已交付依赖；移除r1误列以消除009↔068依赖环，T00～07顺序/授权/验收与8任务分母不变 | next:work T00/T02。

## 10. 待澄清事项

无待答范围授权。T00须确定真实续体/字段归属原因；T02须先证明Windows provider可行边界，当前无主观选定依赖。Sky输入未达须区分工具与产品；工具限制不授予绕过系统安全或伪造输入的许可。发现范围/兼容/验收必须再变更时按auto-plan:new处理，不能静默扩大。
