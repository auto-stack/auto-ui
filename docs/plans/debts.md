# auto-ui 债务/候选登记

非合同的候选功能与债在此登记；立项时按 `README.md` 取号走正式四阶段，
登记条目不构成合同、不算已授权工作。

---

## DEBT-001 autoui MCP 批量动作（一次请求多动作数组）

- **状态**：未立项（候选 003）。来源：PAINT-001（auto-paint）测试驱动经验 +
  用户 2026-10-29 提议。
- **动机**：网格/画布类 UI 的自动化目前一次请求只能做一个动作。256 格画板
  画一张棋盘格要 ~128×N 次 HTTP 往返，每次还夹一趟全量快照（重渲染改
  vnode 哈希 id，驱动器每步都得重新发现元素）。批量模式把"选色 → 画
  [0,0] → 画 [2,0] → …"压进一个请求，收益覆盖所有同类 UI 测试。
- **提议形状**（用户草图）：`autoui_action` 扩展或新增
  `autoui_action_batch`，`arguments.actions = [{element_id, action,
  value?}...]` 按序执行；返回逐项结果（element/status/handler/state_changes）。
- **必须解决的设计点**：
  1. **批内 id 失效**：任一动作触发重渲染后，后续动作里的 vnode id 可能
     全部失效。执行器需要：批内自动重发现（按位置/标签重寻址）、或声明
     id 在批内稳定、或允许条目用稳定定位子（文本/类/序号）替代裸 id。
  2. **失败语义**：单项失败时 abort+已执行清单，还是继续执行+逐项状态；
     调用方如何区分"第 k 项失败"。
  3. **与现有坐标合成的关系**：`pen`/`drag` 已是"一请求一串事件"的先例
     （canvas 类 widget 坐标合成）；批量动作是其按元素 id 维度的对偶，
     复用同一ActionResult 逐项回报形态。
- **边界**：批量的是**传输层**，每个动作仍走该元素绑定的真实 handler
  （press/press_many 不绕过 UI 事件）——测试语义保持"用户路径"。
- **关闭条件**：owner 仓立项（候选 003）并在 mcp_server 落地批量动作 +
  驱动侧（desktop_mcp/verify_basic 族）改用后，PAINT 类验收的点阵绘制
  从 O(N) 请求降到 O(1)。

## DEBT-002 ag-examples AG026/DEBT-001 关闭窗口（M1 N4 重判＋接收器重跑）

- **状态**：未立项（跨仓消费提示，UI076 落地解锁；改动面在 ag-examples，
  不在本仓合同内）。来源：UI076（076-m1-native-scale-trail-and-n4-semantics）
  T-07 消费提示，2026-11-10。
- **动机**：UI076 已落主（8b4ff336）：F-SCALE-02 修复（`observe_window`
  合并语义＋f64 真实读数跟踪——单次切档会话 `tier.scales_observed` 保留
  [起始档, 末档]、`scale_factor_changed` 记真实前后对，validator 单档误拒
  消失）；F-N4-01 语义裁定落地（方案 b：N4 检查表收窄为"空白区按下→离窗
  →窗外释放 ⇒ 无按钮激活、无区域点击"；onclick 区域按下即派发为 canonical，
  SD-01/SD-02 已沉淀入 `m1-native-evidence-support` 与
  `node-mixed-regions-lifecycle` 两规范）。
- **关闭条件**：ag-examples/PLAN-026（AG026）按本仓 8b4ff336+ 引擎
  （1）以新 N4 检查表文本重判 N4；（2）重跑双档接收器（双档会话报告应产
  `scales_observed=[2.0,2.5]`＋真实转换对，可 native-pass）；（3）关
  ag-examples DEBT-001（其 session-bundle 其余证据已齐）。若 AG026 改派发
  时机为 release-inside（方案 a）则与 canonical 冲突，需升格重验——不在
  本提示范围。

## DEBT-003 daemon 生产就绪检查升级：health body interface 断言（PLAN-067 P-03）

- **状态**：未立项（跨仓前置登记，来源：auto-os PLAN-067 T-02 P-03，2026-11-13）。
- **动机**：`osconfig_daemon.rs:137 tcp_ping` 读首 128 字节、响应行 `HTTP/1.x 2xx`
  前缀即算就绪，不解析 body——200 但 interface 错误的占用者会被
  `ensure_generic_io:349` 零打扰复用为就绪。FeaturePack 契约（featurepack-v0
  §2.2）要求 200 + body interface==`notes.fpack/1`；错版服务在位时 Reader/Notes
  的 fp 调用全数失败且桌面徽标仍显 ready。反例已具名实测（PLAN-067 V1e）。
- **提议形状**：`GenericDaemon` 增可选 `health_interface: Option<String>`
  （manifest daemon 同名可选字段）；tcp_ping 读满 body（分块至 Content-Length
  或连接关闭，超时上限）后 2xx ∧ interface 匹配才算 Running。无该字段的
  daemon（autoos/musk/jade-garden）行为零变化（D4 零回归门同律）。
- **边界**：改动面=本仓 runtime（osconfig_daemon.rs）+ compiler
  （app_registry.rs ManifestDaemon）；测试脚本自有 health_ready
  （auto-os verify_featurepack_host.py）不能替代生产检查。
- **关闭条件**：①200+错 interface 占用者→拒复用、占用者存活、Offline 具名；
  ②正确版本冷启与在位复用通过（含 spawn 后轮询期内 interface 不匹配不得
  提前就绪）；③无声明字段 daemon 全族零行为差。消费：auto-os apps.manifest
  notes daemon 加 `health_interface: "notes.fpack/1"`（交付后同批提交）。

## DEBT-004 daemon env 传递面：每宿主凭据注入与客户端地址键消费（PLAN-067 P-05）

- **状态**：未立项（跨仓前置登记，来源：auto-os PLAN-067 T-02 P-05，2026-11-13）。
- **动机**：`ensure_generic_io:358` spawn env 仅 `[env_port_key]`；无每宿主
  凭据/授权书注入。Notes 侧已交付受管 policy（服务私有
  `<data>/fp-policy.json`，cred=256-bit hex；OS 供给入口
  scripts/fp_provision_policy.{sh,ps1}），但宿主侧（Reader 后端）拿不到自身
  cred（NOTES_FP_CRED）——生产 launch 后 fp 调用全拒绝。
- **提议形状**：`GenericDaemon` 增可选 env 传递（如 `env: {KEY: VALUE}` 或
  `env_file`），spawn env 与 launch 后 App 会话 env 双面注入（会话写点同
  env_key 先例 `session.rs:4483` 单键短值）。地址键不改派生规则：客户端消费
  键冻结为 `daemon_env_key(manifest id)`=`015_NOTES_DAEMON`（Reader 后端
  fp_base() 增读该键，属 READER-013 T-02 消费面）。
- **边界**：凭据生成/撤销/书籍范围授予 owner=auto-os（已交付 v0 脚本面）；
  本仓只负责精确传递。
- **关闭条件**：生产 launch 后 Reader 后端首请求携合法 cred→capture ok；
  错书（scope 外）/撤销后拒绝；合法 receipt 跨重启可查。测试用非默认
  端口/目录排除硬编码回退掩盖。

## DEBT-005 daemon 跨进程单实例门（PLAN-067 P-06）

- **状态**：未立项（跨仓前置登记，来源：auto-os PLAN-067 T-02 P-06，2026-11-13）。
- **动机**：ensure 为 ping→spawn 无跨进程互斥；两独立宿主并发冷启在 ping
  间隙可双 spawn。服务端 bind-fail 自然收敛已实测兜底（败者 exit 1、恰一
  LISTENING，PLAN-067 V1d），但那是服务层兜底而非宿主门，且 detached
  spawn 失败对用户不可见。
- **提议形状**：spawn 前按 daemon id+数据根取进程间互斥（named mutex 或
  等价），或在二次健康确认仍冲突时具名 Offline；`DID_SPAWN` 进程内标志
  不升级为门。
- **边界**：错误端口占用不杀占用者（既有语义保持）；关闭主窗口后另一宿主
  可继续调用（detached 共享服务语义保持）。
- **关闭条件**：两独立真实宿主（双桌面会话）同时冷启同一 manifest daemon→
  全程恰一 LISTENING、归属同一 spawn；两宿主 App 会话均能完成业务调用。
  验收编排归 auto-os PLAN-067 T-04（C-01）。

## DEBT-007 桌面 boot 启动器宿主 env 投递缺口：pac daemon 声明不进 desktop-backends.json（PLAN-067 T-02 编排发现）

- **状态**：未立项（跨仓缺口登记，来源：auto-os PLAN-067 T-02 C-01/C-04 编排预备核查，2026-11-14）。
- **证据（源码行级）**：①`auto-ui-tools/desktop_backends.rs:11-16` `DesktopBackend{id,root,profile,port}`
  ——`DesktopBackend::read`（:18-53）读 pac.at 只取 `api`/`back_port`，**pac `daemon:` 声明被丢弃**；
  ②boot 启动器非共享臂 `resolve_declared_backend(Path::new(".."), &app.id)`（:193）按 **app 自身 id**
  查 manifest daemon 表——消费者 app（如 reader，pac `daemon: "015-notes"`）无自身 id 条目 →
  `NotDeclared` → 后端子进程 spawn 时**无宿主 env**；③session 窗口 launch 臂为 InProc 投递
  （session.rs:4305）——地址键写桌面进程全局（后端子进程已 spawn 读不到）、受管凭据按 SD-01 §5
  合同不写进程全局（具名缺口诊断）。净效果：真实 Vue 桌面流里 Reader 后端 fp 调用地址靠
  17819 硬编码回退侥幸命中、**凭据为空 → 全部 permission_denied**——Reader→Notes 产品链在
  桌面装配层断裂。083 SD-01 §11 将"真 Vue/Rust 后端启动档"显式划归 OS-067 联合验收，
  本缺口正在该被划出的面内。
- **提议形状**：`DesktopBackend` 行增 `daemon: Option<String>`（`DesktopBackend::read` 从
  pac `daemon:` 取值）；启动器非共享臂改为 `resolve_declared_backend(parent,
  row.daemon.as_deref().unwrap_or(&app.id))`——pac 声明了 daemon 的消费者按**被声明名**查表
  （reader → "015-notes" 条目 → 共享门 ensure → Ready(env) → `apply_host_env` 投递其
  后端子进程），未声明者维持 app.id 语义零变化；`SharedBackendDecision`（共享 provider 判定）
  不变。session InProc 臂不变。
- **边界**：改动面=本仓 tools（desktop_backends.rs）+ 可能的 `vue.rs` 行生产处；运行时
  osconfig_daemon 不动（resolve/apply 原语已就绪）；测试=启动器夹具行带 daemon 的投递断言
  + 无声明行零变化。
- **关闭条件**：真实桌面 boot 后 Reader 后端子进程 env 含 `015_NOTES_DAEMON`+`NOTES_FP_CRED`
  （行为面：经 Reader 后端 fp 代理调用 Notes 成功；无声明 app 后端 env 零变化）。
  消费：auto-os PLAN-067 T-02 C-01/C-04 桌面编排（凭据腿以此交付为前置）。

## DEBT-008 后端依赖不可达时 native UI 冻结（fp 流量 + notes kill，READER-013 T-08 复现）

- **状态**：已确定性复现（3/3 轮同点位）、归因未定（2026-11-16）。来源：
  READER-013 T-08 双端验收（证据包
  auto-reader/plan-artifacts/plan-013/BACK-STALL-BLOCKER.md）。
- **动机/症状**：`auto run -r vm --server vm` 原生窗口下，后端 HTTP 依赖
  （notes 服务）被 kill 后，UI 在 fp 流量中（检查点 submit / 抽屉 list）
  **整体失去响应**：后续所有 autoui 按压 30～120s 无投影、back 进程存活但
  books GET 亦不响应（监视 DOWN）、整窗重启才恢复。同一流程 vue 轨全绿
  （77/0/0 对照）——native 消息循环/同步 HTTP 语义与 vue 异步轨分歧。
- **复现入口**：auto-reader 仓
  `python tests/annotations/verify_featurepack_host.py --profile vm`
  R03-U3a 腿（notes kill 后 UI submit），3/3 确定性；跑后 SQLite 直接可证
  数据已提交、恢复意图恰好一条（产品合同满足，纯 UI 冻结）。
- **提议形状**：定位 native 窗口 VM 消息循环中同步 HTTP 出站（fp POST/
  GET）阻塞调度根因——候选：①出站无超时（后端依赖不可达时 handler 永久
  pending）；②消息循环对 pending handler 的排队策略；③axum 工作线程与
  VM owner 线程的交互。修复方向：出站超时 + 超时后 handler 可终止/降级
  （错误投影已在 .at 层就绪：transport 分类 + 错误信封）。
- **边界**：.at 层传输分类/错误投影/检查点恢复均已实现且 vue 轨验证
  （READER-013 修复在案）；本债只覆盖 native 运行时的出站阻塞语义。
- **关闭条件**：复现入口全绿（vm profile R-01～05 通过、无必需 skip），
  且 notes kill 场景下 native UI 保持可交互（错误投影可见）。
- **2026-11-16 engine 线诊断落点（归因修正，5 轮复现 + 停摆态线程栈；
  全文见 auto-reader plan-artifacts/plan-013/BACK-STALL-BLOCKER.md 末节；
  栈/dump/日志在 D:/autostack/plan-artifacts/fix-debt008-diag/，登记在
  .retention/policy.json）**：
  - **原候选①「出站无超时」排除**：引擎出站三重防御实锤（post_json 异步
    live-op + 共享 client 30s + 执行器 total_timeout + 非阻塞准入），且 UI
    失败期间 back 实测 0.02s 应答、front handler park/resume 毫秒级正常。
    原候选②③的「VM owner 线程排队/axum 交互」亦无栈证据。
  - **真根因 = 本仓窗口进程（auto-ui-tool vm-session 子进程）三失效模式**：
    1. iced 主循环卡 `wgpu acquire_texture`/GPU fence 30~120s（停摆态栈
       main 帧；**扰动即解冻**——1s 调试器挂起即可恢复，复发条件为连续
       执行窗）；
    2. `iced_wgpu.offscreen.source_texture` 无效 → wgpu Validation Error
       → 主线程 panic = 「进程静默退出」（panic 日志在案；iced 0.14 +
       wgpu 27.0.1）；
    3. 事件注入丢失：handler park 线停 + 按压被吞，但渲染活/back 健康，
       楔死态双进程栈显示**OS 层完全空闲**（无 VM 帧/锁/GPU 等待）——
       事件在 MCP→iced→VM 派发链路丢失（会话管道/订阅 desync 类）。
  - **触发器修正**：UI 侧重渲染搅动（抽屉 ToggleFp/错误横幅）即触发，
    notes kill 非必要（run5 R-01 U2a 未杀 notes 即复现）；vue 轨绿 =
    浏览器渲染无此三模式。
  - **修复面（拟）**：渲染环 wgpu 错误处理改非 panic（device-lost/offscreen
    texture 生命周期与重建）、acquire/帧等待加超时与恢复、事件注入链路
    （mcp-action-push→iced subscription→VM 会话派发）的丢失检测与重置；
    `AUTO_HTTP_STEP_BUDGET=1e9`/`AUTO_HTTP_REQUEST_TIMEOUT_MS=300000`
    （验收环境放大项）建议回设计值后复测。

  - **2026-11-16 PLAN-098 修复关闭（本仓义务面，master@1b3a02b6）**：
    三失效模式根修全落地——①vm-session 子进程默认 `WGPU_BACKEND=dx12`
    （Vulkan ash 楔死栈证据；用户显式设置不覆盖）+ 帧心跳判定入口
    （FRAME_SEQ + `[SCHED-DIAG] frame_seq` 签名）；②`crates/patches/
    iced_wgpu`（非 panic uncaptured handler `[WGPU-DEGRADE]` + screenshot
    零尺寸早退 + poll 2s 有界）+ snapshot 泵 in-flight 守卫；③事件注入
    gate 化（rx 终生锁内永不搬走，take-once 死亡类结构上不可达）+
    交付计数/停滞检测 + `ActionResult.delivery`。验证：gate 单测 4/4
    （断口期零丢失区分器）+ mcp_server 41/41 + vm profile ×3（设计值
    env）零冻结/零 panic/零事件丢失，cohort 旧链对照 R-03 boundary
    直接崩 vs 新链全推过。判定 artifact：
    docs/plans/evidence/plan-098-backend-decision.md；规范：
    docs/specs/runtime/window-runtime-resilience.md（WRR-01..04）。
    验收口径 r2（用户裁决）：「R-01～05 全绿」依赖仓外域
    （WebView2 CDP 起动摇摆→DEBT-009；reader 恢复语义→READER-013/
    auto-reader；notes read-snapshot stall→notes 仓 owner①），残红
    在 cohort 旧链同样红、非本计划回归。
    **状态：本仓义务关闭；G4 全绿门随仓外域修复另行收口。**

## DEBT-009 doc-surface WebView2 CDP 起动摇摆（PLAN-098 T-06 甄别移交，本仓 doc-surface 域）

- **状态**：已甄别、未修（2026-11-16）。来源：PLAN-098 T-06 vm profile
  ×3 轮（plan-artifacts/plan-098/ 取证，判定 artifact 在
  docs/plans/evidence/plan-098-backend-decision.md）。
- **症状**：doc-surface 的 WebView2 调试口（`--remote-debugging-port`）
  间歇性 180s 不可达（run2 首窗、run1 boundary 重启窗；run3 全通）——
  `browserType.connectOverCDP: ECONNREFUSED`。窗口进程本身健康
  （books GET 3-10ms、242 HTTP 行活跃）。同域既有不稳定：cohort 旧链
  对照轮直接 `boundary re-enter failed` 崩溃。
- **域**：auto-ui-runtime doc-surface（wry/WebView2 子表面生命周期）。
  非 DEBT-008 三模式（渲染环/注入链与它无关——窗口日志零冻结零丢失）。
- **关闭条件**：vm profile 连续 ≥3 轮 boundary 重启后 CDP 均可达，
  或 doc-surface 起动失败有显式重试/降级投影。

## DEBT-010 native 视图差分/子件注入缺口：组级结构重渲染与包父层子件事件（PLAN-092 甄别移交，本仓 auto-ui-runtime 域）

- **状态**：已甄别、未修（2026-11-20）。来源：PLAN-092 T-07 复审修复轮
  六形态系统复测（docs/plans/archived/092-property-inspector.md §9、
  docs/reviews/plan092/decisions.md §9）。
- **症状（两个互补面）**：
  ①**组级结构重渲染缺口（B-6）**：包 widget 内 `for g in .grows` 的组级
  结构变化（grows 重建、组记录成员条件、被移除子树的再插入）在 native
  视图不重渲染；仅现存子树内的标量驱动互换（edit_id 型）与 push 驱动的
  `List.contains` 首折可见。同类型多实例还伴随面板行重投影不稳定。
  ②**包父层子件注入缺口（B-5）**：父件为包 widget 时，子控件按钮的注入
  消息 `delivery: sent=1 delivered=0`、嵌套 input Enter 不派发 committed；
  错误事件路由正常。route 形态对照：local 父
  `.NumericFielddemo@//…`（通）vs 包父
  `.NumericField/PropertyInspector#0/…`（不派发）。
- **影响**：PropertyInspector 交互折叠 deferred（v1 静态分组标题）；native
  子控件（087/088）编辑事件矩阵由 vue 轨承担（产品门 B5 用例 not_run
  单列）。受影响消费面：任何包 widget 内嵌子控件按钮/输入的 MCP 无人值守
  驱动与组级动态 UI。
- **域**：auto-ui-runtime 视图差分（dynamic.rs 重渲染/diff）与子件事件
  注入链（dynamic.rs Plan 370/PLAN-098 注入族）。非 DEBT-008/009 面。
- **关闭条件**：①组级结构变化（列表重建/成员条件翻转）在包 widget 内
  可靠重渲染（受控对照用例：折叠→字段消失→展开→字段恢复，双实例共存）；
  ②包父层子件按钮 press 的 delivered=received 且嵌套 input Enter 派发
  committed（PLAN-092 复现脚本可复用）。修复后 PropertyInspector 交互
  折叠双轨补齐（合同文本不变）。

## DEBT-011 CollectionView 的 Linux 平台验收

- **来源**为 PLAN-109 r2。用户于 2026-11-20 批准将 Linux 平台验收随统一 Linux 开发补跑。
- **状态**为待补验。Linux native VM 与 Linux Vue 均记录为 `not_run`。
- **责任方**为 AutoUI 统一 Linux 开发与验收。
- **关闭条件**为在 VMware 的目标自制发行版上按 109 的 V-01…09 运行适用用例，分别验证独立组件、027 FileManager、AssetPicker 和 029 Gallery。保留真实窗口或浏览器、MCP 操作、实际状态读回、失败清理、退出重开的证据，记录工具和源码版本。
