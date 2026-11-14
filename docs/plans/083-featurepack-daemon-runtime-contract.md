---
plan_id: auto-ui/PLAN-083
status: execution_done
feature_name: FeaturePack daemon生产就绪、宿主配置与单实例
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-14
plan_revision: 2
current_step: 6
total_steps: 8
depends_on: []
supersedes_spec_components: [docs/specs/build/desktop-api-surfaces.md]
new_spec_components: [docs/specs/runtime/featurepack-daemon-hosting.md]
touched_goals: []
---

# PLAN-083 · FeaturePack daemon生产就绪、宿主配置与单实例

## 0. 变更摘要

合并承接DEBT-003/004/005，提供 [OS-067](../../../auto-os/docs/plans/067-notes-featurepack-product-closure.md) T-02的运行时前置。三项共用app_registry/session/osconfig_daemon调用链，由同一计划按健康检查、配置传递和单实例三个可验任务交付。当前drafting、0/8。

本计划提供公共宿主能力与运行时验证，OS拥有Notes发布、受管授权及真实双应用联合验收，Reader拥有启动声明和客户端消费。[Reader-013](../../../auto-reader/docs/plans/013-notes-featurepack-host.md) T-01/T-06可先只读准备，正式产品实现等待009交付交接。

## 1. 目标

Windows本地FeaturePack声明可选择按接口版本核验daemon、向正确服务与宿主后端传递各自配置、在独立宿主同时冷启时只启动一份共享服务。无扩展声明的既有autoos/musk/jade-garden路径保持现行行为。普通占用端口不能被误认成目标服务或被清理。

沿现有构建时UI嵌入。AGE/ACE融合、动态插件加载、窗口内自动重ensure、UI081/082的路由/对话框能力由相应计划负责。v0杀后恢复消费OS冻结的重开窗口→launch→ensure入口。

## 2. 架构方案

compiler ManifestDaemon负责可选配置声明，runtime组装与ensure负责生命周期，受管配置由OS签发。凭据生成、授权书内容和书目范围授予归OS，运行时只验证声明并精确传递。Reader后端使用015_NOTES_DAEMON与自身凭据，NOTES_DAEMON保留独立/测试覆盖语义，优先级由T-00与消费方冻结。

拟唯一实现树D:/autostack/.wt/ui-083/auto-ui，分支codex/plan-083-dev，实施/内检/顺序复审共用。主检出master持计划。OS沿其现有唯一实现树，Reader沿独立clone/main；本计划实现只修改auto-ui，跨仓产品修改由各自任务交接。

## 3. 技术栈

Rust现有auto-ui-compiler/auto-ui-runtime、生产ManifestDaemon→UiSession::ensure_manifest_daemon→ensure_generic_io→RealDaemonIo路径。复用已有HTTP、JSON、文件和OS进程/互斥设施，优先已存在的受管配置和owned-process接口。T-00选择具体机制前先证完整生产路径。

## 4. 需求分析与背景调查

来源为用户本会话FeaturePack设计/分仓规划，以及2026-11-13要求分析OS T-02的新发现并明确下一步。本轮建立可评审前置合同、更新关联计划，不执行运行时实现或新产品测试。未指定模型、预算或新代理。

| 观察源 | 基线/事实 |
|---|---|
| auto-ui master | 86e12cb2036761b869705a8ad580adf285891c2c；债务登记提交f21ab5e7，三个债务均未立项实施；081/082不承接daemon |
| crates/auto-ui-compiler/src/app_registry.rs | ManifestDaemon现仅port/bin/env_port；manifest_daemon_lookup返回repo_dir/def；runtime ui/mod.rs re-export |
| runtime/src/ui/session.rs | ensure_daemon_if_declared/ensure_manifest_daemon依pac daemon触发；路径式bin；daemon_env_key派生地址后进程级set_var |
| runtime/src/ui/osconfig_daemon.rs | tcp_ping只核HTTP2xx首段；RealDaemonIo::spawn仅参数env；ensure_generic_io为ping→spawn→轮询，无跨进程门 |
| docs/specs/build/desktop-api-surfaces.md | Vue桌面后端自持Rust进程、占用拒绝与退出回收；需要先核共享daemon声明如何避免Notes后端被第二次拉起或被桌面退出误清理 |
| OS唯一树 | 72f962f70ea3060f0d734d57456f6c9fc2cc76cb，路径式Notes发布、AUTO_HTTP_PORT、policy脚本、生产形状探针与host脚本已落 |
| Notes | 006 r2归档；pac daemon声明6a7b0e74c52d5a84f984d1f506f4b46516893182；health返回值存在JSON字符串信封，探针需有限次解包 |
| 现有可复用测试 | osconfig_daemon.rs内tcp_ping_real_server_and_dead_port、泛化ensure测试；compiler app_registry.rs内manifest_daemon_lookup_via_sibling_probe |

当前测试计数来自OS执行记录：V1 11/11、host19/19。它们证明服务与spawn形状，不覆盖UiSession真实桌面链；单监听加败者bind失败不能证明宿主只spawn一次。原始粘贴记录是观察输入，本计划以实际源码、提交与当前合同为依据。

**授权记录（2026-11-14）**：用户以 `/goal 完成计划083` 交付本计划端到端——建/用唯一实现树、实现与内检、同树独立复审、批准范围内有界修复、合并（规范/归档/清理/落地检查）。该授权不改变目标、验收标准、阈值、兼容承诺、仓范围或预算，不豁免工作树与证据纪律，也不授权在无绑定当前 revision 与复审提交的最终 pass 时合并。

## 5. 详细设计

### D-01 可选能力与兼容

T-00冻结可选manifest描述及参数表。health_interface为待实现可选字段候选，缺席沿既有语义；存在时只有完整合法响应且interface匹配才Running。配置格式必须能表达服务进程配置、宿主后端配置、受管凭据引用、daemon实例键与就绪预算，具体字段由T-00在真实消费者先证后确定。复用路径式bin，命令行解析不在本前置。

新字段错误/未知能力给具名拒绝，不默默退回旧探活。旧autoos专用分支和普通daemon字段/环境派生按已支持契约回归。三个旧DEBT的提议形状属于候选，最终字段以T-00合同为准。

### D-02 生产就绪

健康检查受超时与响应字节预算约束。需要支持当前Notes实际响应和已声明支持的HTTP framing，读取分段响应后核status/body/interface。Notes双编码JSON按有限层数解析，坏JSON、错误interface、超限、超时及不支持的响应形态有明确结果。预算和支持集在T-00用真实响应冻结，不从任意无限body假定支持。

区分无人监听、正确服务、已占用但不匹配。错版/普通占用者存活，拒绝复用并避免向其发送凭据；不在占用端口继续无意义spawn。spawn后轮询也必须核相同接口，2xx早到不能提前ready。就绪不推导数据库写成功，业务提交依Notes回执。

### D-03 配置与凭据边界

服务只取其端口/数据根/policy等必要配置，宿主后端只取自身地址与凭据。公开manifest不包含明文凭据或服务完整policy；凭据从OS提供的受管引用解析，诊断与回执只显示非敏感身份及结果。不要直接把DEBT-004的进程全局set_var候选当最终机制。

T-00实际跟踪桌面inproc/native/outproc与Vue后端启动点，给出应用/后端归属和配置生效时序。私有配置必须在目标后端首次请求前可见；A/B宿主使用不同值或一个未授权时，后启动者不能改写前者的身份。若现有Env.get只支持进程全局，先证可行的后端初始化/显式上下文机制或给明确接口缺口，不能以轮流设置同名全局变量通过隔离验收。

在位复用与重开恢复都需配置注入，不能只在本进程spawn时注入。服务死亡后按v0重开窗口重新ensure，并用同operation取原receipt。隐私配置解析失败保旧运行对象，显示可恢复错误，未经授权的写请求不得提交。

### D-04 跨进程单实例

Windows按声明daemon与受管实例范围建立进程间启动协调。T-00冻结规范化身份（至少含用户、daemon、端点/数据根），避免仅靠路径文本别名或DID_SPAWN进程内标记。锁内重验health→仅赢家spawn→待ready；输家有限等待并复用已验证实例。失败/进程退出/超时释放所有权，可恢复重新启动，不能持锁无限等待。不同数据根与相同端点的冲突须明确拒绝或由profile限制，不能悄悄采用另一根的库。

共享daemon独立于任一主窗口寿命。已声明共享provider由其owner持有，普通app自有后端继续用原owned-process清理规则。特别核Notes主窗口和Vue桌面API启动器是否再次绑定17819；共享复用须在声明与身份校验之后发生，普通端口占用仍拒绝。

### D-05 分仓交付

UI先交T-00接口和runtime验证输入。Reader先只读核接口，待009主干交付/代码交接后再T-06/02适配；本Plan自身实现不等待009或013终态。UI T-01～04不等Reader整Plan完成，使用实际生产ensure/后端初始化路径的受控宿主测试。UI交付精确SHA/字段与测试结果后，OS消费manifest扩展和受管配置，Reader消费声明及地址/凭据，再由OS T-02真实双宿主验收闭合。OS T-04最后联合产品验收，不能作为UI实现/复审的前置。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/featurepack-daemon-hosting.md | 现有daemon基本启动→可选版本就绪、受管配置、宿主边界与单实例/恢复 | 公共运行时契约 | AC-01～05 |
| SD-02 | modify | docs/specs/build/desktop-api-surfaces.md | 自有API后端→在显式声明下接共享provider并分清配置/生命周期 | 避免双启动/误清共享服务 | AC-02/03/04/05 |

## 6. 测试设计

T-00给支持边界，T-01～03补区分错误实现的反例，T-04在真实运行时启动路径汇总。测试源码候选 runtime现有osconfig_daemon.rs单元测试、ui_tests/featurepack_daemon_hosting_tests.rs（新增）、compiler app_registry测试。多进程helper与产物为测试自有资源，起跑先核隔离端口、数据根、凭据和进程归属，结束仅清自有对象。

| Case / owner_stage | AC | 入口与独立oracle |
|---|---|---|
| U-01 / work | AC-01/05 | 真实TCP分段返回正确版/错版/坏JSON/截断/当前双编码/过大/超时；status2xx但interface错误不得Running或spawn，原占用者存活；无新字段旧路径行为相同 |
| U-02 / work | AC-02 | 两个不同宿主配置/凭据经生产后端初始化，非默认地址调用；A/B互不污染，未授权A不能借B成功，复用在位服务也正确注入；日志不含凭据。Notes集成由OS再验 |
| U-03 / work | AC-03 | 两独立测试宿主进程同刻入生产ensure，记录spawn总数必须等于1，均ready同实例；不同根/相同端点冲突、赢家中途退出、锁失败/超时、端口已有错版均可恢复且不杀无关进程 |
| U-04 / work | AC-04 | 关任一窗口共享服务存活、另一方可请求；杀自有服务→重开窗口→ensure恢复，原数据仍在；Notes服务被共享声明接入后没有第二份自有后端启动 |
| U-05 / work | AC-05 | autoos/musk/jade-garden兼容样例、现有app API生命周期、无扩展字段的旧序列回归，字段解析错误明确拒绝 |
| U-06 / work | AC-01～05 | 从UiSession真实launch路径和实际Vue后端启动器观察参数/事件/配置归属；不得只测另写的ensure函数。记录profile能力及无法覆盖的具体格 |

现有可执行入口可从auto-ui根使用 cargo test -p auto-ui-runtime osconfig_daemon、cargo test -p auto-ui-compiler manifest_daemon、cargo test -p auto-ui-runtime featurepack_daemon，后者由T-04新建测试模块。T-00按当前crate/feature验证命令并记录实际选择的测试列表；没有选中case、必要格skip或失败均非零。现有固定组仅按本仓约定选择受影响组，免重复全仓重跑；旧app缺证不能记pass。

原始材料统一D:/autostack/plan-artifacts/plan-083/，摘要/输入版本/命令/结果留本Plan。实现与内检后由独立上下文在同实现树顺序复审。当前会话仅起草，没有独立终审成绩，也不创建新代理。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | U-01正确版冷启/复用通过，错误版/不完整/超时拒复用；状态含失败原因且占用者存活 |
| AC-02 | work | U-02精确传递，服务/宿主配置分开、A/B隔离、首请求前可见；明文不进入公开manifest/日志 |
| AC-03 | work | U-03两个独立宿主完整竞争期间spawn=1，失败后可恢复，实例范围与端口冲突不误共享 |
| AC-04 | work | U-04共享生命周期与原数据保持，按v0重开恢复，声明共享provider时没有端口双绑/误杀 |
| AC-05 | work | U-05/06通过，旧声明兼容，真实生产launch及Vue后端路径有证据，OS/Reader可消费固定字段与版本 |
| AC-06 | work | SD-01/02、DEBT-003/004/005关闭映射、OS消费字段表与可复跑输入齐全 |
| AC-07 | review | 同实现树独立上下文核AC-01～06和高风险反例，绑定完整commit/revision与依赖，明确支持profile |
| AC-08 | merge | 精确主干交付、规范与归档核对，OS收到可引用SHA/兼容性/字段清单；债务在实现交付后关闭 |

## 8. 执行步骤

执行中（work 阶段）。下一入口 T-00，依赖当前代码和OS已形成字段/服务形状，无OS或Reader整Plan终态前置。

| ID | owner_stage | 依赖 | 实际改面/工作 | 关闭产出 | AC |
|---|---|---|---|---|---|
| T-00 | work | 无 | 核compiler/app_registry、runtime/session/osconfig_daemon、Vue后端启动器，先证D-01～04完整配置时序与资源范围，冻结字段和真实命令/profile | 输入/输出/失败样例、代码路径/owner、受管配置读取机制；未知机制有可行/最小缺口裁定 | AC-01～05 |
| T-01 | work | T-00 | ManifestDaemon可选能力解析与生产health检查，支持当前Notes实际信封及已声明HTTP边界 | U-01及旧健康路径合法对照 | AC-01/05 |
| T-02 | work | T-00、T-01 | 精确服务/宿主配置读取与后端初始化传递，清晰区分公开声明与私有凭据引用，避免进程全局覆盖宿主身份 | U-02、复用与冷启同律，接口可供Reader/OS消费 | AC-02/05 |
| T-03 | work | T-01/02 | 跨进程启动协调、锁内二次检查、失败释放/恢复、共享provider生命周期与Vue后端去重 | U-03/04，spawn事件总数=1且非只看listener个数 | AC-03/04/05 |
| T-04 | work | T-03 | 从真实UiSession/Vue初始化路径汇总U-01～06，运行受影响旧门，记录OS消费示例与字段/SHA | 实现全部work行为可交接，必需缺失非零 | AC-01～05 |
| T-05 | work | T-04 | 沉淀SD-01/02、兼容表、DEBT映射与OS/Reader接口交付材料 | 消费不需要猜新字段或重写ensure | AC-06 |
| T-06 | review | T-05 | 同树独立上下文复演高风险健康/凭据/跨进程/退出反例与支持profile | 明确review结论/精确SHA，保持OS最终联合验收边界 | AC-07 |
| T-07 | merge | T-06 pass | 按精确提交落master、核规范与归档/清理、将交付SHA发回OS计划 | 三债实现交付后关闭，OS再验证真实两应用 | AC-08 |

## 9. 复审记录

2026-11-13 · stage: new · plan_id: auto-ui/PLAN-083 · plan_revision: 1 · outcome: pass（草案可交接） · next: work T-00。0/8。原债务是non-contract，现以正式Plan承接；创建计划不表示实现或测试完成。用户本轮要求判断新发现的下一步，本会话在已有分仓规划范围内形成最小运行时合同，产品实现另交work。

原OS T-02的路径式服务发布、policy供给和Notes声明已提交；UI运行时三项缺口继续open。本次只读源码/脚本及提交，没有重跑11/11或19/19，没有把服务端bind-fail收敛作为单实例门。

2026-11-13 本次文档交接自检通过。UI083/OS067/Reader013的章节/任务ID/完成计数/相对链接和限定仓归属的任务依赖图已核对，依次0/8、2/7、0/8，无循环；083编号唯一，三债指针有效。仅文档与源码审核，运行时实现与产品测试未执行。

2026-11-13 · stage: new（Reader交接次序澄清） · plan_revision: 2 · outcome: pass（草案） · next: work T-00照常推进。Reader013产品写入等待009交付；083自身接口/实现/验证无009终态前置，原任务/AC与0/8保持。

2026-11-14 · stage: work（收尾） | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass（execution_done，6/8；T-06/T-07 归 review/merge） | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | base_commit: a6f776c2 | code_commit: 38ed6add（T-05）／14539ae7（T-04）／93280912（T-03）／34884f99（T-02）／c9238191（T-01）＋c76994b1（lock） | dependency_revisions: auto-lang e0cfa2d8c / auto-web 8b1742a0 / auto-down 58ea33f6 | next_responsible_role: review T-06

**Work-owned AC 结果（均绑当前提交字节）**
- AC-01（U-01）pass：`--lib osconfig_daemon` 39/39（正确版冷启/复用、错版/不完整/超时拒复用且占用者存活、状态含具名原因）。
- AC-02（U-02）pass（含一处具名缺口）：受管凭据解析+打码、A/B env 隔离+继承键剔除、真实 launch 地址/literal 注入且凭据不写全局；明文凭据进公开 manifest 被拒。
- AC-03（U-03）pass：多进程 4/4——spawn 日志 = 1、两宿主同实例、冲突/错版/宿主退出语义。
- AC-04（U-04）pass（边界记录）：宿主全退后 daemon 仍 ready；共享判定单测钉死 Notes 17819 去重；真实 Notes 双窗口/真 Vue 后端编排未跑（OS 联合验收）。
- AC-05（U-05/U-06）pass（边界记录）：旧声明兼容（无新字段逐字旧语义、旧集成档自跳过不计通过）；真实 `UiSession` launch 注入/复用 1 passed；真后端编译启动档未跑（`PLAN053_PROBE_DIR`）。
- AC-06（T-05）pass：SD-01/SD-02 已写入 worktree `docs/specs/**`；DEBT-003/004/005 映射见 SD-01 §9；OS/Reader 消费字段表见 T-04 回执与 SD-01 §10。

**Readiness record**：in-contract 发现 = 无未解决项；契约决定 = 无待决（R1～R4 已在 T-00 冻结，T-01～T-05 按此实现）；覆盖缺口 = in-proc 凭据隔离（另一仓接口缺口，已具名并禁止全局冒充）、真实后端启动档与 Notes 双窗口编排（OS 联合验收边界）；已跑测试 = 见上（另 `--lib launch_app` 10、`--lib native_launch` 1、tools `desktop_backends` 3+1 ignored、compiler 34）；未跑/未通过项 = `osconfig_integration` 材料缺席自跳过、`plan814` 三应用用例依赖原生产物（`AUTO_OS_ROOT` 指离真实清单即通过，与 daemon 无关）；统一回归门未跑，owner = review/merge（按仓约定）；代码/依赖身份 = 上述提交 + 组内依赖修订。

2026-11-14 · stage: work | task: T-05 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | code_commit: 38ed6add | next_responsible_role: review T-06

规范增量落地（在 worktree 内准备，随合并发布；未手动改任何 index/ledger）：
- **SD-01 add** `docs/specs/runtime/featurepack-daemon-hosting.md`：12 节——所有权、manifest 字段表与非法拒绝、健康合同（请求/预算/支持 framing/6 类具名裁决/轮询语义/无字段旧语义）、服务进程 spawn env、宿主配置分界（受管凭据解析/打码/in-proc 缺口/生效时序）、实例身份与跨进程门（锁/标记/输家/冲突/释放）、共享 provider 生命周期与后端去重、兼容矩阵、DEBT-003/004/005 关闭映射、OS/Reader 消费接口、当前缺口如实记录、证据索引。
- **SD-02 modify** `docs/specs/build/desktop-api-surfaces.md`：新增"声明共享 daemon 时的后端归属与去重（PLAN-083 SD-02）"节——`back_port == daemon.port` → 共享 provider（不起第二份后端/不判占用/路由 strip）、先声明与身份核验后复用、普通占用仍拒绝、配置归属（子进程投递；in-proc 缺口）、生命周期（detached vs owned-process）、既有构建纪律不变。
- 兼容表与 DEBT 映射已进 SD-01 §8/§9；OS/Reader 消费字段表见本 §9 T-04 回执，规范侧为 SD-01 §10。
文档校验：新增/修改文件仅 `docs/specs/**`；未创建 `docs/plans/attachments/`、未写 `.autoos/specs.json`。

2026-11-14 · stage: work | task: T-04 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass（work 行为可交接；缺口见边界） | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | code_commit: 14539ae7（前置 93280912/34884f99/c9238191） | next_responsible_role: work T-05

**U-01～06 汇总（全部在上述当前提交上执行）**
- U-01（AC-01/05）`cargo test -p auto-ui-runtime --lib osconfig_daemon` → **39 passed/0 failed**：双层信封 Ready、错版/缺席接口、坏 JSON、截断、CL/无长度超预算、占位监听者、死端口、503、chunked、gzip TE 拒绝、**旧 2xx 语义逐字保留**；strict 复用零 spawn、错版前置拒绝、spawn 后错接口立即拒、声明预算超时、bin 缺席具名。对照：无新字段声明走 `ensure_generic_io` 旧路径（同套件旧用例全绿）。
- U-02（AC-02/05）`daemon_instance`/`osconfig_daemon` T-02 用例（受管凭据解析+Debug/诊断打码、双宿主 A/B env 隔离 + `remove` 覆盖、受管配置缺席具名错误、端口权威）+ **真实 launch** 用例 `featurepack_daemon_launch`：地址键 `015_NOTES_DAEMON=<url>`、literal `NOTES_RETRY=2` 进 in-proc 会话，受管凭据 `NOTES_FP_CRED` **不**进进程全局（输出具名接口缺口 R3）；明文凭据进公开 manifest 被具名拒绝（compiler 34/34）。
- U-03（AC-03）`--test featurepack_daemon_hosting -- --test-threads=1` → **4 passed**：两独立宿主同刻冷启 → 均 Running 同 url、spawn 日志恰 1 行、端点 LISTENING=1；相同端点不同数据根 → 具名冲突；错版占用 → 不 spawn 且占用者存活。
- U-04（AC-04）同档"宿主进程全部退出后 daemon 仍 ready" + tools `shared_backend_decision`（`back_port==daemon.port` → 共享 provider，不 spawn 第二份、不把 daemon 端口当占用）+ `onnx` 豁免用例；共享 provider 由 detached 进程持有（`RealDaemonIo::spawn` detached，退出不杀）。**边界**：真实 Notes 桌面双窗口/Vue 启动器编排未在本机跑（需真后端产物），由 OS T-04 联合验收闭合。
- U-05（AC-05）旧声明兼容：`auto-musk`/`jade-garden` 无新字段仍按路径式 bin + env_port 派生（T-01 前既有用例 39 套件内保持绿）；`daemon_capability_fields_optional_and_parsed` 钉死缺席语义；`health_interface_grammar_frozen`/`instance_key` 非法值具名拒绝。
- U-06（AC-05）`--test featurepack_daemon_launch` → **1 passed**：从真实 `UiSession::launch_app`（`__test_session` + 注册表 resolver）进生产 ensure 链，观察 ①状态 Running+url ②地址/literal 注入 ③凭据不全局 ④首次 spawn=1、二次 launch 复用不 spawn。**边界**：真实 Vue `DesktopBackends::start` 的 Rust 后端编译/启动档未跑（`PLAN053_PROBE_DIR` 门）；该启动器的共享判定与端口豁免由 `desktop_backends` 单测覆盖，实际后端子进程 env 投递由 `rust_ui::start_desktop_api_server` 接线 + T-02 子进程 env 用例证明。

**受影响旧门**：`--test osconfig_integration`（autoos 旧链）→ 通过但**材料缺席自跳过**（相邻仓 `auto-os-config` 前端/daemon 二进制不在本组树；不计为通过格）；`--lib launch_app` 10 passed；`--lib native_launch` 1 passed；compiler `app_registry::tests` 34 passed；tools `desktop_backends` 3 passed/1 ignored（heavy probe）。既有环境依赖记录：`plan814_app_registry_launch_tests::launch_three_real_apps_via_registry_resolver` 在真实 `auto-os/apps.manifest` 下要求 013-todo 原生产物（本组树无），把 `AUTO_OS_ROOT` 指离真实清单即 1 passed——**与 daemon 改动无关的既存环境依赖**，非本计划回归。

**OS/Reader 消费字段表（固定名/语义，供 OS-067 T-02 与 Reader-013 消费）**
| 面 | 字段/键 | 语义 |
|---|---|---|
| manifest daemon | `port`(必填), `bin`(路径式相对 repo), `env_port` | 既有；`bin` 平台中性、缺 `.exe` 自动补 |
| 可选能力 | `health_interface` = `<name>/<n>`；`ready_timeout_ms` = 200..=60000 | 存在即严格核验；缺席 = 旧 2xx |
| 服务进程配置 | `env{KEY:VALUE}` | spawn 时投递；端口键由 url 权威覆盖 |
| 宿主后端配置 | `host.env{KEY:VALUE}`；`host.credential{env,path?,principal,field?}` | 仅投递独立进程后端；凭据从 OS policy 解析 |
| 实例身份 | `instance_key`（`[A-Za-z0-9._:-]`）、`data_root`（`%VAR%` 展开） | 跨进程门/身份核对；同端点不同根具名冲突 |
| 宿主注入键 | `<NAME>_DAEMON`（如 `015_NOTES_DAEMON`）、host literal、`host.credential.env` | in-proc 只投非敏感；子进程全量 |
| 受管 policy | `{"entries":[{principal,cred,operations,book_scopes}]}`；`NOTES_FP_POLICY` > `<data>/fp-policy.json` | OS `fp_provision_policy` 签发；cred 64 hex |
| 健康响应 | 2xx + JSON `{status:"ok", interface:"<name>/<n>"}`（允许 ≤2 层字符串信封） | 200/错版/坏 body/超预算均不等于 Ready |

交付 SHA（本任务代码）：`14539ae7`（T-04）、`93280912`（T-03）、`34884f99`（T-02）、`c9238191`（T-01），基线 `a6f776c2`。

2026-11-14 · stage: work | task: T-03 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | code_commit: 93280912 | next_responsible_role: work T-04

实现（U-03/U-04；"spawn 事件总数=1"而非 listener 个数）：compiler `ManifestDaemon` 增 `instance_key`（`[A-Za-z0-9._:-]`，≤96，具名拒绝）与 `data_root`（`%VAR%` 展开、空串/控制符拒绝）+ `manifest_daemon_lookup_in_root`（显式根查表，生产入口复用）。runtime 新模块 `daemon_instance.rs`：`InstanceIdentity`（user+instance_key+port+规范化 data_root → blake3 短哈希；仅靠路径文本别名或进程内 `DID_SPAWN` 不算身份）、`InstanceLock`（Windows `share_mode(0)` 独占打开=等价 named mutex，进程退出由 OS 释放；非 Windows `create_new`+TTL 回退）、`InstanceMarker`（就绪后写 pid/started_at；**按端口**定位 = 同端点唯一 owner，输家复用前核对身份）、`ensure_shared_io`（快路径复用 → 取锁 → 锁内二次探测 → 仅 NoListener 才 spawn → 同接口轮询到 Ready → 写标记 → 释放锁；取锁失败=输家在有界 `lock_wait` 内轮询复用，超时具名 Offline；失败回收标记并释放锁）。`osconfig_daemon::resolve_manifest_host_shared` 把 manifest/预算/宿主 env 接到该门；session 改用共享入口。tools `desktop_backends` 增 `shared_backend_decision`（pac `back_port` == manifest `daemon.port` → 共享 provider）：先解析/核验共享 provider（`resolve_manifest_host_shared`），启动期不再为其 spawn 第二份自有后端、不把 daemon 端口当"被占用"拒绝，routes 直接指向该 daemon（Notes 17819 主窗口/Vue 桌面 API 去重）；普通占用者仍拒绝。

证据（当前 commit 上执行）：`cargo test -p auto-ui-runtime --lib daemon_instance` → 7 passed（身份规范化/别名、锁互斥+释放、标记往返+身份不匹配、已就绪零 spawn、身份冲突拒绝、冷启只 spawn 一次+写标记、错版占用不 spawn）；`cargo test -p auto-ui-runtime --test featurepack_daemon_hosting -- --test-threads=1` → **4 passed**（真多进程：两独立宿主同刻冷启 → 两者 `running` 同 url、spawn 日志恰 1 行、端点 LISTENING=1、宿主退出后 daemon 仍 ready；相同端点不同数据根 → 具名冲突；错版占用 → 不 spawn 且占用者存活）；`--lib osconfig_daemon` 39 passed（回归）；tools `desktop_backends` 3 passed/1 ignored（含共享判定与"共享端口不算占用"豁免）。夹具 `fp_daemon_fixture` 为测试专用 bin（无 `FP_FIXTURE_PORT` 即 exit 2），spawn 事件日志是唯一计数 oracle。既有 `AUTO_OS_ROOT` 并行 env 踩踏（app_registry 模块 6 个用例）加进程内串行锁后 6/6 并行运行绿；未跑：真 Vue/Rust 后端编译启动档（`PLAN053_PROBE_DIR` 门，T-04/OS 联合验收边界）。

2026-11-14 · stage: work | task: T-02 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | code_commit: 34884f99 | next_responsible_role: work T-03

实现（U-02、复用与冷启同律）：compiler `ManifestDaemon` 增 `env`（服务进程非敏感 literal）与 `host{env, credential{env,path,principal,field}}`；`validate` 增加 env 键语法/空值/控制符/疑似凭据键（cred|token|secret|passw|apikey）/长 hex 值拒绝、凭据键与 literal 冲突拒绝、principal/field/path 校验——公开 manifest 不可能携带明文凭据。runtime `osconfig_daemon.rs` 增 `ManagedCredentialRef`（`%VAR%`/`$VAR` 展开、policy `entries[].principal` 命中取值、长度/空白/控制符校验；Debug与诊断打码，错误只回显主体/路径）、`HostBackendEnv{entries,remove,credential_env}`（Debug 打码、`non_secret_entries`）、`host_backend_env`（地址 → literal（键排序）→ 受管凭据；`remove` 覆盖全部投递键）、`GenericDaemon::spawn_env`（声明服务 literal + 端口键权威覆盖）、`generic_daemon_from_manifest`/`resolve_manifest_host`/`probe_manifest_host`（session 与桌面启动器同源）。session 以 `DaemonEnvDelivery::{InProc,Child}` 区分：in-proc 只写非敏感键并对凭据记具名接口缺口（R3，不写全局）；独立进程（native exe / Rust API 后端）随 spawn 精确投递并剔除继承同名键。tools `desktop_backends` 在后端启动前 `probe_manifest_host`（只探不孵）：Ready 投递、Invalid 拒绝启动、NotReady 保持既有链；`rust_ui::start_desktop_api_server` 接收 `HostBackendEnv` 并在 `TrackedSpawn`（`with_env`/`without_env`）与裸 `Command` 两臂投递。

证据（当前 commit 上执行）：`cargo test -p auto-ui-compiler --lib app_registry::tests::manifest_daemon` → 7 passed（含服务/宿主分离解析与 5 类公开凭据拒绝）；`cargo test -p auto-ui-runtime --lib osconfig_daemon` → **39 passed / 0 failed**（新增 T-02 6 条：受管凭据解析+打码、双宿主 A/B 隔离与 remove 覆盖、受管配置缺席具名错误、服务 env 端口权威、`%VAR%`/`$VAR` 展开、probe 三种形态不 spawn）；`cargo check -p auto-ui-tools --lib` 通过（后端子进程投递接线）。

2026-11-14 · stage: work | task: T-01 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | code_commit: c9238191（+ c76994b1 lock 同步） | next_responsible_role: work T-02

实现（U-01 与旧健康路径对照）：compiler `ManifestDaemon` 增 `health_interface`（可选，`<name>/<n>` 语法冻结，名 `[A-Za-z0-9._-]` 首字符字母/数字、版本十进制、≤128）与 `ready_timeout_ms`（200..=60000）；`ManifestDaemon::validate` + `manifest_daemon_lookup_checked` 给具名拒绝（Err），兼容入口仍折叠 None，session 把 Err 映射为 Offline（`session.rs` ensure_manifest_daemon）。runtime `osconfig_daemon.rs` 新增 `HealthSpec`/`HealthVerdict`（Ready/NoListener/OccupiedElsewhere/WrongInterface/NotReady/BadBody，均带具名 `reason`）、`read_health_response`（超时+heads 8192B+body 预算，支持 Content-Length/chunked/close-delimited，其他 TE 具名拒绝）、`decode_health_body`（≤2 层字符串信封）、`probe_health`（2xx + 合法 JSON + `interface` 逐字 + `status` 若在为 ok）；`DaemonIo::health` 默认由 ping 派生、`RealDaemonIo` 走 probe；`ensure_generic_strict` 冻结 spawn 前仅 NoListener 可 spawn（错版/未就绪/坏 body/占位监听者→Offline 且不 spawn），spawn 后同接口轮询、错接口立即拒绝、2xx 早到不提前 ready。`GenericDaemon` 增 `health_interface`/`ready_timeout`，`ensure_generic_io` 在 interface=None 时逐字保留旧 2xx 路径（U-01 旧路径行为相同）。

证据（当前 commit 上执行）：`cargo test -p auto-ui-compiler --lib manifest_daemon` → 5 passed / 0 failed（含 capability parse 与具名拒绝）；`... --lib health_interface` → 1 passed；`cargo test -p auto-ui-runtime --lib osconfig_daemon` → **33 passed / 0 failed**（新增 14 条：双层信封 Ready、错版/缺席接口、坏 JSON、截断、CL 超预算、无长度超预算、占位监听者、死端口、503、chunked、gzip TE 拒绝、旧 2xx 语义、strict 复用不 spawn、错版前置拒绝不复用、spawn 后错接口立即拒、声明预算超时、bin 缺席）。`Cargo.lock` 仅组内 auto-lang 清单同步（独立 chore 提交，无版本变化）。未跑：`--test osconfig_integration`（真 daemon 材料档，T-04 汇总）。

2026-11-14 · stage: work | task: T-00 | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass（合同冻结） | worktree: D:/autostack/.wt/ui-083/auto-ui（分支 codex/plan-083-dev，base a6f776c2） | dependency_revisions: auto-lang e0cfa2d8c / auto-web 8b1742a0 / auto-down 58ea33f6（组内 detached 树，与 ui-084 同源） | next_responsible_role: work T-01

生产路径（本次逐文件读过源码，非转述）：compiler `app_registry.rs:682 ManifestDaemon`→`:726 manifest_daemon_defs`→`:751 manifest_daemon_lookup`；消费点 runtime `session.rs:4415 ensure_daemon_if_declared`→`:4442 ensure_manifest_daemon` 组装 `GenericDaemon`→`osconfig_daemon.rs:344 ensure_generic_io`；就绪后 `session.rs:4483` 以 `std::env::set_var(<NAME>_DAEMON,url)` 进程全局注入。现行健康检查 `osconfig_daemon.rs:137 tcp_ping` 只读首 128 字节、`HTTP/1.x 2xx` 前缀即通、不读 body；`DID_SPAWN`(:161) 是进程内原子标记，不是跨进程门；spawn env 仅 `[env_port_key]`(:358)。

Notes 当前响应：`auto-notes/rust-workspace/auto-notes-30db9907-back/src/db.rs:448 fp_health` 返回 `{"status":"ok","interface":"notes.fpack/1"}`，路由 `main.rs:699 /api/health`，返回类型 `JsonResponse<String>` → 双层 JSON 信封（探测需有限层解包）。OS 声明：`D:/autostack/auto-os/apps.manifest` 的 `015-notes.daemon = {port:17819, bin:"../../../auto-notes/target/release/notes-service", env_port:"AUTO_HTTP_PORT"}`；OS 唯一树 HEAD `72f962f` 已提交路径语义（主检出副本仍是旧命令行 bin）。`auto-notes/pac.at` 为 `api:"rust"`、`back_port:17819`、`daemon:"015-notes"`、`desktop:"true"`，manifest `launch:"native"`——与 Vue/Rust 后端启动器共用 17819 是真实双绑面（T-03）。受管配置读取机制（OS 签发）：`auto-notes/.../repository.rs:1804 fp_policy_path`（`NOTES_FP_POLICY` > `<data>/fp-policy.json`），条目 `entries[].{cred,principal,operations,book_scopes}`，cred 为 64 hex；OS `scripts/fp_provision_policy.{sh,ps1}` 幂等签发 `principal=fp-reader`。宿主后端初始化：`crates/auto-ui-tools/src/desktop_backends.rs:72 start` 在桌面启动期逐 app spawn，`:110-121` 以进程全局 `AUTO_HTTP_PORT` set/restore 包裹 `start_desktop_api_server`（`rust_ui.rs:7272`），`:103/:165` 已有名拒绝占用端口；`Env.get` 语义见 `auto-lang/crates/a2r-std/src/env.rs:10` = `std::env::var`（VM `auto.env.get` 同源）→ 进程全局。

裁定（D-01～04 的最小可行合同）：**R1 字段冻结** `health_interface`（`<name>/<n>`：名称 `[A-Za-z0-9._-]` 首字符字母/数字、版本十进制、≤128；缺席=旧 2xx 语义）与 `ready_timeout_ms`（200..=60000，缺席=调用方 5s）；声明存在即严格核验 2xx + 合法 JSON（≤2 层字符串信封）+ `interface` 逐字相等 + `status`（若在）为 `ok`；支持集 = Content-Length | chunked | close-delimited，body 预算 8192B，其余 framing 具名拒绝；声明非法 → 具名 Offline，不退回旧探活。**R2 就绪/拒绝** spawn 前仅 `NoListener` 允许 spawn，`WrongInterface/NotReady/BadBody/OccupiedElsewhere` 一律 Offline 且不 spawn（占用者存活、不向其发配置/凭据）；spawn 后同接口轮询，只有 Ready 结束，错接口立即拒绝，2xx 早到不提前 ready。**R3 宿主配置边界** 进程全局 `Env.get` 无法给 in-proc VM 应用做到每宿主隔离，记为精确接口缺口（需 auto-lang 改动，属另一仓/另一计划）；凭据类宿主配置只投递给独立进程后端（native exe / Rust API launcher 子进程），in-proc 场景具名拒绝而非全局覆盖；在位复用/重开恢复要求解析发生在后端 spawn 之前。**R4 单实例** Windows 进程间互斥（等价 named mutex，`windows` 0.58 `Win32_System_Threading` 已在依赖内）+ 锁内二次 health 复核 + 实例标记文件（port/instance_key/data_root/pid）供输家核对身份；输家有限等待→复用已验证实例；进程退出由 OS 释放锁，超时具名 Offline。

测试命令冻结：`cargo nextest run -p auto-ui-runtime osconfig_daemon`、`cargo test -p auto-ui-compiler manifest_daemon`、`cargo nextest run -p auto-ui-runtime --test osconfig_integration`（真 daemon 材料缺席即跳过，零回归对照）；T-04 新增 `--test featurepack_daemon_hosting`（多进程 helper + 真 spawn 计数），显式枚举运行，零选择不算 PASS。本任务只读源码/脚本，未改实现、未跑运行时测试。

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 1 | from: drafting | to: work（T-01） | status: execution_done | outcome: pass（T-00 合同冻结） | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: a6f776c2 | next: work

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 2 | from: work（T-01/T-02） | to: work（T-03） | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: 34884f99 | next: work（T-03 跨进程单实例与共享 provider）

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 3 | from: work（T-03） | to: work（T-04） | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: 93280912 | next: work（T-04 真实 launch 路径汇总与旧门回归）

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 4 | from: work（T-04+T-05 收尾） | to: review（T-06） | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: 38ed6add | next: review（T-06 同树独立上下文复审）

2026-11-14 · stage: review | review_scope: final | reviewer: plan083-reviewer（独立子代理会话，未参与实现；Windows 本机，无真后端产物/双应用） | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | reviewed_commit: 38ed6add332d184b70f28910b5a23b6f2a7c0157 | base_commit: a6f776c24a4ba98611849955edceb5a04edac969 | dependency_revisions: auto-lang e0cfa2d8c / auto-web 8b1742a0 / auto-down 58ea33f6 | spec_inputs: SD-01 docs/specs/runtime/featurepack-daemon-hosting.md sha256 95C4F8753DFF67472A38164EECDDB69878BD1C3457F351F230BB0C6D1985F26E；SD-02 docs/specs/build/desktop-api-surfaces.md sha256 AB0246E557AEA4679D449799A940419794E73CDEFF94466FC0299F2CD4D110C3（均 @38ed6add） | next_responsible_role: work（T-01/T-02/T-04/T-05 修复）→ review T-06

**绑定与清洁度**：worktree 干净（Cargo.lock 已随 38ed6add），HEAD = 38ed6add；base..HEAD 14 文件 +4279/−82。未发现 `.autoos/specs.json` 等索引/台账写入；未新建 `docs/plans/attachments/083`（该目录仅有 10-29/30 历史内容）。复审临时探针源码运行后已删除，工作树复检干净；原始日志 `D:/autostack/plan-artifacts/plan-083/review/{tests,adversarial,osconfig_integration}.log`（该 scope 随 merge 删除）。

**独立复跑（均绑 38ed6add 字节，0 failed）**：`--lib osconfig_daemon` 39；`--lib daemon_instance` 7；`--test featurepack_daemon_hosting -- --test-threads=1` 4；`--test featurepack_daemon_launch -- --test-threads=1` 1；compiler `app_registry::tests` 34；tools `desktop_backends` 3 passed/1 ignored。另 `--test osconfig_integration` 报 "1 passed" 但材料缺席即早退、零断言（与 work 回执一致，不记旧链证据）。

**AC 结果**：AC-01 pass（错版/坏 body/超时/占位监听者拒复用且不 spawn；反例 P2：2xx + `status:"degraded"` + interface 匹配 → Offline、spawn=0、占用者存活）。AC-02 **partial**（F-2/F-3：in-proc 地址/literal 注入与凭据不落全局已钉死；受管凭据 Debug/诊断/错误不回显经 P5 复核；独立进程投递无行为证据）。AC-03 pass（4/4 + P6：旧声明 3 宿主并发冷启 spawn=1 同 url；P3：spawn 失败后锁释放、重试冷启 spawn=1；P4：同端点不同 instance_key 具名冲突、不 spawn）。AC-04 pass（含已记录边界：宿主全退后服务存活；P8：杀服务→再 ensure 冷启一次恢复 Running，spawn=2；真实 Notes 双窗口归 OS）。AC-05 **fail**（F-1）。AC-06 partial（SD-01/SD-02 落地、add/modify 与 §5 表一致、无索引/attachments；但 §3/§8 与 SD-02 时序句须随 F-1/F-2 修正）。AC-07 = 本记录。AC-08 待 merge。

**F-1（阻塞；AC-05，D-01；计划 §1:27、SD-01:76/141；T-01/T-03/T-04/T-05）**
违反既有规则：§1:27「无扩展声明的既有autoos/musk/jade-garden路径保持现行行为」、D-01:64「缺席沿既有语义」、SD-01:76「无 health_interface 时 2xx 即就绪，不读 body」、SD-01:141「逐字不变」、T-04 回执:177「无新字段声明走 ensure_generic_io 旧路径」。
观测：session.rs:4482 把每个非 autoos 的 manifest daemon 一律送 `resolve_manifest_host_shared` → `daemon_instance::ensure_shared_io` → `probe_health`（osconfig_daemon.rs:552）；后者在 interface 判断（562）之前已 `read_health_response`（553），把超预算/超时/不支持 framing 判成 BadBody；`ensure_generic_io` 旧分支（:1247）在生产已无调用方（仅测试与同样无人调用的 resolve_manifest_host:921）。
可复跑反例（真实 socket/进程）：
- P7：旧声明（无新字段）+ 服务前 3 次 503 后 200：旧规则 `ensure_generic_io` → Running（415ms、spawn=1）；生产入口 → Offline「不可复用：服务未就绪（HTTP 503）」（12ms、spawn=0）。
- P1：旧声明 + 2xx 但 health body 9024B：`tcp_ping`=true（旧规则），legacy HealthSpec=BadBody（响应体超预算 9024>8192），生产入口=Offline、未 spawn。
影响：旧 daemon 冷启/预热竞态由 Running 变 Offline 且不再拉起；大/慢 body 同理。旧链证据仅覆盖生产死路径的单测。
要求：行为与合同一致——恢复无新字段声明时的既有行为；若确要收紧，须经计划修订显式变更该兼容格并同步 SD-01 §3/§8 与回执（仅改文案不足以放行）。

**F-2（阻塞；AC-02/D-03；SD-02:72；T-02/T-05）**
违反规则：D-03「私有配置必须在目标后端首次请求前可见」、AC-02「首请求前可见」、SD-02:72「声明的宿主配置随该应用后端子进程 spawn 投递」。
观测：desktop_backends.rs:192 用 `probe_manifest_host`（只探不孵）；daemon 未就绪时 :206 分支令 host_env=None，:219 以 None 启动后端子进程——`<NAME>_DAEMON`、host literal、受管凭据均不投递；后续 session ensure 无法向已运行子进程补投（run_vue.rs:400 在 app launch 前启动后端）。仅共享格（`back_port == daemon.port`）做到先 resolve 后 spawn。
可达性：当前清单无「独立 daemon 端口 + rust 后端」条目，属潜在但 AC 未限定的合同格。要求：对声明 daemon 的 app 在后端 spawn 前 ensure/解析，或如实记该格为缺口并修订 AC-02/SD-02（语义变更需新 revision）。

**F-3（证据完整性；AC-02/U-02；T-02/T-04）**
T-04 回执:182 称「后端子进程 env 投递由 rust_ui::start_desktop_api_server 接线 + T-02 子进程 env 用例证明」；实际无子进程投递用例：`host_backend_env_isolates_two_hosts` 只测 env bag/remove，`spawn_exe_child` 与 `start_desktop_api_server(host_env)` 无测试（仅定义/调用点）。U-02 的「两个不同宿主配置/凭据经生产后端初始化」未执行。要求：补真实子进程投递 oracle（如测试专用 env dump bin 走 native outproc/后端启动器），或按其它边界同样如实记为未证。

**规范增量判定**：SD-01 新增（185 行/12 节）、SD-02 修改（+23 行）——add/modify、路径、rule change、rationale、AC 映射与设计表一致；§5（in-proc 凭据缺口）与 §11（真后端/Notes 双窗口）措辞与代码一致，未见假装隔离。但 §3:76 / §8:141 与实现矛盾（F-1），SD-02:72 为无条件承诺（F-2），故本次规范增量不通过，随修复更新后再验。

**pending_closeout_gates**：真实 Vue/Rust 后端构建启动与 Notes 双窗口 = OS 联合验收（保持）；统一回归门 = merge 所有；组内依赖树未改动。
**next**：work 按 F-1/F-2/F-3 修复（T-01/T-02/T-04/T-05 重开；T-00/T-03 保持完成 → 2/8），再回 review T-06；在绑定新提交的复审前不得给 pass。

2026-11-14 · stage: work（R-1 修复） | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | outcome: pass（F-1/F-2/F-3 关闭，回 review T-06） | worktree: D:/autostack/.wt/ui-083/auto-ui | branch: codex/plan-083-dev | base_commit: a6f776c2 | code_commit: b9999db9 | 修复周期: 1/3 | next_responsible_role: review T-06（第二次最终复审）

**修复闭合行（finding → invariant → affected paths → fix/permanent cases → 当前结果 → 剩余缺口）**
- **F-1 → 旧语义（无 `health_interface`）= 只看状态行，且在所有生产入口生效** · affected `osconfig_daemon.rs` `read_health_response`/`probe_health`、`daemon_instance.rs` `ensure_under_lock`、`session.rs:4482` 入口 · fix：①`read_health_response` 解析完状态行后，`interface=None` 立即返回（不读 body、不核 framing/预算，与 `tcp_ping` 逐字同义）；②共享门内旧声明除 `Ready` 外一律视为"未就绪→spawn/继续轮询"（初始 503 不再终局拒绝），只有声明了 interface 才保留"占用者不匹配→不 spawn"。permanent cases：`probe_health_legacy_reads_status_line_only`（20KB body/gzip TE/截断 body 均 Ready；声明时 BadBody）、`shared_ensure_legacy_waits_through_initial_503_and_spawns`、真夹具 `u01_legacy_declaration_converges_through_initial_503`（503×3→200，spawn=1，`tcp_ping` 观察就绪） · 结果：`--lib osconfig_daemon` **40 passed**、`--lib daemon_instance` **8 passed**、hosting 档 **5 passed** · 剩余缺口：无（`ensure_generic_io` 旧函数体未改，仍为无锁直接路径并保留测试）。
- **F-2 → 任何声明 daemon 的宿主配置必须在后端首次请求前可见** · affected `desktop_backends.rs` 非共享 rust 应用分支 · fix：启动期对已声明 daemon 一律先 `resolve_manifest_host_shared`（ensure+身份核验）再 spawn，`Ready` 投递 env；`Invalid` 拒绝启动其后端；`Offline` 按既有"Offline 不阻断"语义启动但打印"后端在无宿主配置下启动"具名降级（不再静默 None）。permanent cases：`shared_backend_decision_matches_declared_daemon_port_only`（判定/端口豁免）+ 子进程投递门（F-3）+ SD-02 新增同义条款 · 结果：tools `desktop_backends` 3 passed/1 ignored；代码面 8 行顺序可核（ensure→resolve env→spawn） · 剩余缺口：真实 Rust 后端编译/启动档未跑（`PLAN053_PROBE_DIR` 门，OS 联合验收边界）——该格不因修复扩大承诺。
- **F-3 → 宿主配置只经子进程精确投递、不写宿主全局，且有进程级 oracle** · affected `session.rs` `spawn_exe_child`、tools `rust_ui.rs` 两臂、`osconfig_daemon::apply_host_env`（新单源）、新档 `tests/featurepack_daemon_child_env.rs` · fix：投递逻辑收敛为 `apply_host_env(cmd, env)`（先 `env_remove` 同名继承键再写值），session 与桌面启动器共用。permanent cases：`child_env_delivery_is_per_child_and_never_global` —— 两个真子进程各带 A/B 地址+凭据；断言互不可见、宿主陈旧同名值被剔除、未投递键 `<unset>`、**父进程 env 未被改写**。 · 结果：`--test featurepack_daemon_child_env` **2 passed** · 剩余缺口：无。
- **证据更正（append-only）**：T-04 回执中"子进程 env 用例"当时不存在（复审 F-3 指出）；现由 `featurepack_daemon_child_env` 2 passed 兑现，原文保留不改，以本行更正。

**回归（b9999db9 上执行）**：`--lib osconfig_daemon` 40、`--lib daemon_instance` 8、`--test featurepack_daemon_hosting --test-threads=1` 5、`--test featurepack_daemon_launch --test-threads=1` 1、`--test featurepack_daemon_child_env --test-threads=1` 2、compiler `app_registry::tests` 34、tools `desktop_backends` 3+1 ignored。规范同步：SD-01 §3 旧语义措辞改为"只读状态行（不读 body/framing/预算；初始 503 不终局）"，SD-02 增"非共享声明 daemon 也在后端 spawn 前 ensure；Invalid 拒绝、Offline 具名降级"。未跑：`osconfig_integration`（材料缺席自跳过，不计通过）、真后端启动档、Notes 真实双窗口（OS 联合验收）。

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 5 | from: review（needs_fix） | to: review（T-06 第二次） | status: execution_done | outcome: pass（修复闭合） | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: b9999db9 | next: review

## 10. 待澄清事项

T-00需裁定宿主配置的真实inproc/child/Vue生效机制、可选字段语法、JSON信封/HTTP支持边界和预算、实例范围/锁方案。失败产生精确前置，依原目标继续，不能默认把全局env注入当每宿主隔离。OS最终两真实应用编排位于本计划交付后；UI无需等待OS T-02整体关闭才能实现。
