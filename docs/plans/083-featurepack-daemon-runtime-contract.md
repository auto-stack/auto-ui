---
plan_id: auto-ui/PLAN-083
status: executing
feature_name: FeaturePack daemon生产就绪、宿主配置与单实例
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-14
plan_revision: 2
current_step: 3
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

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 1 | from: drafting | to: work（T-01） | status: executing | outcome: pass（T-00 合同冻结） | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: a6f776c2 | next: work

2026-11-14 · stage: goal | plan_id: auto-ui/PLAN-083 | plan_revision: 2 | round: 2 | from: work（T-01/T-02） | to: work（T-03） | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-083/auto-ui | commit: 34884f99 | next: work（T-03 跨进程单实例与共享 provider）

## 10. 待澄清事项

T-00需裁定宿主配置的真实inproc/child/Vue生效机制、可选字段语法、JSON信封/HTTP支持边界和预算、实例范围/锁方案。失败产生精确前置，依原目标继续，不能默认把全局env注入当每宿主隔离。OS最终两真实应用编排位于本计划交付后；UI无需等待OS T-02整体关闭才能实现。
