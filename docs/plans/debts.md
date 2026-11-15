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

## DEBT-006 native 渲染器文字字形丢失（strategy Q-01，PLAN-084 复现）

- **状态**：已复现、归因到公共渲染器文本路径；修复计划未立（2026-11-14）。
  来源：[PLAN-084](084-file-dialog-folder-and-save-modes.md) T-01/T-05（策略
  [component-library-extension-strategy Q-01](../design/autoui/component-library-extension-strategy.md)）。
- **动机**：真实 native 窗口出现可见文字缺字/截断，而 MCP/state 文本完整
  ——Reader009 首报，PLAN-084 在本仓 FileDialog 上双实例复现：①分页文本
  `第 11 / 11 页` 渲染为 `第 11 /`（FileDialog 尾页实拍，截图
  plan-artifacts/plan-084/q01-pagination-crop.png，源图 1-tail.png）；②路径
  输入值（verbatim 前缀 `\\?\` + 盘符路径）渲染丢失 `D:` 后那个反斜杠字符
  （源图 1-alternate-theme.png，裁 q01-pathinput-crop.png）。同帧其余中英文/数字
  正常；MCP value 含完整字符。归因：渲染器文本 shaping/布局（input 与
  text 两原语均现），FileDialog/分页包非肇事方。
- **提议形状**：定位 iced 文本 shaping/测量/裁剪管路中丢字形簇的根因
  （混合脚本行、`\` 与 CJK 相邻、页码 f-string 等触发面），补 renderer
  级回归断言（渲染像素/字形计数对照 state 文本），全量 UI 门对照。
- **边界**：组件包与消费方不做逐字形 workaround；MCP 文本完整性不作为
  视觉通过依据（既有规则）。
- **关闭条件**：两个复现实例在真实窗口字形完整（截图对照），且新增
  renderer 回归门在 Win/Linux 通过；PLAN-084 受影响视觉 AC（AC-06 Q-01
  项）随本债关闭。
