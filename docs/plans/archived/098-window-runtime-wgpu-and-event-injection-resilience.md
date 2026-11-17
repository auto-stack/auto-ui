---
plan_id: PLAN-098
status: archived
feature_name: 窗口运行时三失效模式修复（GPU fence 楔死 / wgpu offscreen 校验 panic / MCP 事件注入丢失）
author: [agent]
created_at: 2026-11-16T00:00:00Z
updated_at: 2026-11-16T14:00:00Z
plan_revision: 2
current_step: 6
total_steps: 8
supersedes_spec_components: []
new_spec_components:
  - docs/specs/runtime/window-runtime-resilience.md
touched_goals:
  - docs/specs/runtime/desktop-acceptance-input.md
---

# PLAN-098 窗口运行时三失效模式修复（DEBT-008 落点）

## 0. 变更摘要

DEBT-008（后端依赖不可达时 native UI 冻结）经 2026-11-16 engine 线 5 轮复现 +
停摆态线程栈诊断，归因反转：引擎出站路径无罪，三种失效模式全部钉死在本仓
auto-ui-tool vm-session 窗口进程（证据包 auto-reader plan-artifacts/plan-013/
BACK-STALL-BLOCKER.md 末节；栈/dump/日志在 D:/autostack/plan-artifacts/
fix-debt008-diag/）。本计划按三条独立失效面各落修复：

1. **事件注入丢失**（楔死主因，本仓自有代码）：`MCP_ACTION_RX` take-once +
   `mcp-action-push` 线程随订阅身份漂移静默死亡 → 改进程级常驻 forwarder +
   重挂载协议 + 丢失检测；
2. **wgpu 校验 panic**（「进程静默退出」）：`iced::window::screenshot` 的
   offscreen 路径在 device-lost/teardown 竞态下 create_view 打 invalid
   texture → uncaptured error 默认 panic → iced_wgpu 本地补丁改良性降级
   （复用 crates/patches/iced_widget 先例机制）；
3. **GPU fence 楔死**（「冻结 30~120s」，扰动即解冻）：iced_wgpu present →
   wgpu acquire → ash wait_for_fences（Vulkan 后端）上游/驱动交互 → 有界
   调查 + 后端决策 artifact（首选 dx12 env 注入，落点
   run_vm.rs:413 `proc_host::set_vm_session_env`）。

附带一个小的可观测性增量（帧心跳计数），支撑模式③判定与后续回归归因。

## 1. 目标

- **G1**：MCP 事件注入链在订阅身份漂移（primary AppId 变化 / App 动态增删 /
  订阅表重建）下**零丢失**——链路结构上不可楔死，且丢失可检测（不再有
  「send 成功但永不投递」的静默路径）。
- **G2**：wgpu 校验错误/device-lost/teardown 竞态不再杀死窗口进程——降级为
  日志 + 失败截图（或空帧），进程存活、MCP 面继续服务。
- **G3**：vm 轨冻结（30~120s GPU fence 楔死）在复现机上经后端决策消除或
  给出带证据的缓解方案 + 判定入口（帧心跳），决策 artifact 入档。
- **G4（收口门，r2 修订——本仓义务面）**：vm profile 复现轮（设计值
  env）×3 **三失效模式零复现**（零 30s+ 冻结 / 零静默退出 / 零注入
  丢失）+ notes kill 场景 native UI 保持可交互 + 诚实报错投影可见 +
  残红甄别矩阵入档（仓外域残红按域立债/移交，见 evidence artifact；
  r1 原文「R-01～05 全绿」经甄别依赖 WebView2/reader/notes 三仓外域，
  用户裁决修订——§10 Q5）。

**非目标**：
- 引擎侧（auto-lang/auto-web）出站语义——诊断已排除，无待办；
- vue 轨——浏览器渲染，三模式不可达，历史全绿；
- 验收环境放大项回归（`AUTO_HTTP_STEP_BUDGET=1e9` /
  `AUTO_HTTP_REQUEST_TIMEOUT_MS=300000` 回设计值）——属 auto-reader 仓
  READER-013 收口链，本计划只在 G4 复测时使用默认设计值 env；
- wgpu 上游 bug 的根治（device 重初始化/帧队列重设计）——本计划只做
  防崩 + 可恢复面，上游级修复留观。

**受影响仓库/模块**：仅本仓。`crates/auto-ui-runtime`（ui/iced/renderer/
subscriptions.rs、host.rs、ui/mcp_server.rs、shell_state.rs）、
`crates/patches/`（新增 iced_wgpu 补丁）、`crates/auto-ui-tools`（run_vm.rs
env 注入，模式③）。

**约束**：
- 受测工具链是钉死的 auto-man debug 构建——本仓修复落地后需经 auto-man
  重建工具链才能跑 G4 复测（跨仓协调项，见 §10）；
- crates/patches 补丁机制挂在根 Cargo.toml `[patch.crates-io]`，升级 iced
  时需重审（patches/README.md 既有约定）；
- MCP 工具面（验收脚本消费的 HTTP JSON-RPC 契约）**不得破坏向后兼容**：
  既有字段/语义保留，丢失检测只增不改。

**假设**：诊断证据物（minidump ×2、5 轮日志、探针工具
fix-debt008-diag/）在修复验证期内保留可用（已登记 .retention/policy.json，
待本计划消费后可删）。

**成功样貌**：复现机上连续 3 轮 vm profile 无冻结/无静默退出/无按压丢失；
楔死类症状若再现，帧心跳 + 注入 seq 日志能在 5 分钟内定位到失效面。

## 2. 架构方案

三模式三面，互不阻塞，可并行推进：

```
MCP HTTP 工具 ──send_action(seq)──▶ SharedState.action_tx (std mpsc)
                                        │
                                        ▼
                    [新] 进程级常驻 forwarder 线程（唯一 rx owner）
                                        │  交付计数 + 无消费者时入 backlog
                                        ▼
                    全局活跃发送槽 Mutex<Option<UnboundedSender<IcedMessage>>>
                                        ▲
        McpPushRecipe::stream()（可重入）：注册 sender → 排空 backlog
                                        │
                                        ▼
                              iced update 循环（handler park/resume）

渲染环：iced::window::screenshot ─▶ [补丁] iced_wgpu Renderer::screenshot
          （uncaptured error 非 panic handler + 有界 poll + 零尺寸守卫）
present：[决策] WGPU_BACKEND=dx12（env 注入 set_vm_session_env）或留观
```

- **模式③根修**：删除 take-once 语义。现状链（subscriptions.rs:288
  `MCP_ACTION_RX` → subscriptions.rs:835 `stream()` `guard.take()` →
  mcp-action-push 线程 → iced unbounded）中，rx 的所有权随第一次 stream()
  被搬走；订阅身份 = `McpPushRecipe { app: primary }`（host.rs:7231），而
  `primary_app()` = BTreeMap 最小 AppId（session.rs:6440），`apps.remove`
  （host.rs:4737/5904）即触发身份漂移 → iced 弃旧 stream（线程随
  unbounded_send 断裂退出、rx 随闭包 drop）→ 新 stream() 拿 None → 空流
  + eprintln，按压静默丢失。root fix 把「rx owner」与「iced 订阅生命周期」
  解耦：forwarder 线程 = 进程生命周期，订阅来了就挂、断了就 backlog。
- **模式②根修**：上游 iced_wgpu 0.14.0 `Renderer::screenshot`
  （lib.rs:180-230，label `iced_wgpu.offscreen.source_texture` 精确命中
  panic 日志）在 device 失效后 create_view 打 invalid texture，wgpu 默认
  uncaptured error handler panic。补丁三件套：device 建立时安装非 panic
  error handler（日志化）、screenshot poll 超时从 `Wait{timeout: None}` 改
  有界、零尺寸早退。与 iced_widget 补丁同哲学：良性降级，至多丢一拍截图。
- **模式①决策面**：栈顶 `ash::wait_for_fences` 实锤 Vulkan 后端；
  iced_wgpu compositor 已支持 `WGPU_BACKEND`/present_mode env（window/
  compositor.rs:279、with_backend）。先以 env 注入切 dx12 做有界验证
  （3× 复现轮），不行再评估补丁级 present 超时（受限：楔死发生在主线程
  acquire 内，同线程无法自救——预防优于恢复，见 T-06 决策 artifact）。

## 3. 技术栈

- Rust（stable，本仓既有 workspace）；iced 0.14 家族 + wgpu 27.0.1
  （patched iced_widget 0.14.2 本地补丁在案）；
- std::sync::mpsc + iced_futures（futures::channel::mpsc unbounded）——
  既有通道形态，不引入新依赖；
- 补丁机制：根 Cargo.toml `[patch.crates-io]` path 挂载（既有）；
- 测试：cargo test（auto-ui-runtime 既有测试面）+ 手工/脚本化 vm 复现轮
  （复用 fix-debt008-diag/stall_watch*.py 探针与 cdb 抓栈工具链）。

## 4. 需求分析与背景调查

**授权范围**（用户 2026-11-16：「在 auto-ui 新建计划解决这个问题」）：
- 仓库：D:/autostack/auto-ui（唯一实现仓）；
- 动作：新建 Plan、源码修复、补丁新增、测试编写；簿记在主检出，
  实现在唯一 worktree（plans/README.md:174 既有约定）；
- 预算：无用户指定上限；G4 复测依赖 auto-man 重建工具链（跨仓协调，
  阻塞时记 blocker 不硬闯）。

**背景调查（已核实的代码事实，均为本次实读观察）**：

| # | 事实 | 位置 | 性质 |
|---|---|---|---|
| F1 | `MCP_ACTION_RX` 全局 take-once receiver 槽 | auto-ui-runtime/src/ui/iced/renderer/subscriptions.rs:288 | 观察 |
| F2 | `stream()` `guard.lock().take()` 搬走 rx；None 时空流兜底 + eprintln `[PLAN740] mcp push stream: receiver not present` | subscriptions.rs:835-860 | 观察 |
| F3 | 订阅身份 = `McpPushRecipe { app: primary }`；primary = `apps.keys().next()`（BTreeMap 最小 AppId） | host.rs:7231 + session.rs:6440 | 观察 |
| F4 | `state.apps.remove()` 两处（App 动态移除 → primary 可漂移） | host.rs:4737、5904 | 观察 |
| F5 | `mcp-action-push` 线程 `unbounded_send` err 即 return（线程死、rx 随闭包 drop）；`send_action`（SharedState）对已死链仍报成功 | subscriptions.rs:862-905 + mcp_server.rs:405 | 观察 |
| F6 | `send_action_applied` 1s 超时**静默**（只覆盖 `__mcp_*` 合成事件） | mcp_server.rs:421 | 观察 |
| F7 | panic label `iced_wgpu.offscreen.source_texture` 精确命中 iced_wgpu 0.14.0 lib.rs:225 `Renderer::screenshot`；`device.poll(Wait{timeout: None})` 无界 | ~/.cargo/registry iced_wgpu-0.14.0/src/lib.rs:180-270 | 观察 |
| F8 | 仓内 screenshot 泵 `service_snapshot_requests` 的守卫查**记录值** window_size（与真实 surface 态可脱_sync），历史 PLAN-019 已有 0 尺寸硬崩前科 | shell_state.rs:117-152 | 观察 |
| F9 | iced_wgpu 支持 env 覆盖 backends/present_mode（`WGPU_BACKEND` 等） | iced_wgpu-0.14.0/src/window/compositor.rs:279 + with_backend | 观察 |
| F10 | vm-session 子进程 env 注入点 = `proc_host::set_vm_session_env`（run_vm.rs:413 既有 session_env 组装） | auto-ui-tools/src/run_vm.rs:384-413 | 观察 |
| F11 | 停摆态栈：main 卡 `ash::wait_for_fences ← wgpu acquire_texture`（Vulkan 后端）；楔死态双进程栈 OS 全空闲（无 VM 帧/锁/GPU 等待） | fix-debt008-diag/stacks_uifreeze3_a_44964.txt、stacks_vmwedge_095320_* | 观察（栈转储） |
| F12 | run5 未杀 notes 的 R-01 U2a 即楔死——触发器是 UI 侧重渲染搅动（抽屉 ToggleFp/错误横幅），notes kill 非必要 | BACK-STALL-BLOCKER.md 末节 | 观察（5 轮日志） |
| F13 | 事件楔死轮窗口日志签名：`handler_*` park 线全停 + 渲染活 + back 轮询照常 + 按压被吞 | 同上 preserve4/5 | 观察 |
| F14 | `[PLAN740] receiver not present` 是否在受测窗口日志中出现过——**未证实**（窗口日志仅部分保全） | — | **假设（H1）** |

**机制仍属假设的部分**：F14 的触发路径（primary 漂移是楔死主因）是强
推断（形态完全吻合 + 触发器吻合），非日志直证。T-01 先以复现日志 + 新增
seq 诊断在真实链上钉死，再落 root fix——避免修错分岔。

**规范现状**：docs/specs/runtime/desktop-acceptance-input.md 已立「queued
input 单独不构成 pass」原则（本计划丢失检测与其同向，spec 增量见 §5 表）；
窗口运行时韧性（wgpu 错误降级/注入链不丢）**无 spec 覆盖**——新增
window-runtime-resilience.md。

## 5. 详细设计

### 5.1 T-01/T-02 事件注入链 root fix（模式③）

**诊断步（T-01，有界）**：给 `SharedState::send_action` 加单调 seq 计数 +
forwarder 交付计数（投递到 iced 通道即 +1），楔死判定 = sent−delivered 差值
随时间增长。用 fix-debt008-diag 的 stall_watch5 日志签名探针 + 保日志，在
复现机上跑 ≤3 轮：若楔死轮出现「sent 增长、delivered 停、且
`[PLAN740] receiver not present`」→ H1 实锤；若 delivered 停但 receiver
在位 → 检查 `map_mcp_action` 返回 None 的丢弃臂（subscriptions.rs:891）与
update 侧吞没。**决策 artifact**：diag-note 入 plan evidence，注明实锤
路径。若 3 轮均不复现（扰动敏感），以 F1-F5 静态结构 + 单元级漂移模拟
（T-01b 测试，见 §6）作为充分依据落 fix，artifact 记录该降级依据。

**root fix（T-02）**：
1. `run_dynamic_iced`（host.rs:528 旁）启动**进程级 forwarder 线程**
   `mcp-action-forwarder`：唯一 rx owner（从 `start_mcp_server` 直取，
   不再过全局槽）；持有 `Mutex<Option<UnboundedSender<IcedMessage>>>`
   活跃发送槽 + `Mutex<VecDeque<IcedMessage>>` backlog（无消费者时入队，
   容量上限 1024，满则丢最旧 + 丢弃计数）；
2. `McpPushRecipe::stream()`（subscriptions.rs:835）重写：不碰
   MCP_ACTION_RX；向 forwarder 槽注册自己的 sender → 先排空 backlog →
   循环 forward。**身份漂移不再致命**：旧 stream 被 iced 弃掉时线程自然
   退出（slot 恢复 None），新 stream 重挂即续，backlog 保住断口期动作；
3. 丢失检测（mcp_server.rs）：`send_action` 附 seq；forwarder 写
   delivered 计数进 SharedState（原子）；`send_action_applied` 超时臂从
   「静默」改为：超时且 delivered 未覆盖该 seq → eprintln 诊断行 +
   SharedState 置 `injection_stale` 标志；MCP action 工具响应体增附
   `delivered: true/false` 字段（只增不改，向后兼容）；`map_mcp_action`
   丢弃臂补 eprintln（带 handler 名）；
4. 清理：删除 MCP_ACTION_RX 全局及其 take-once 语义；`[PLAN740]
   receiver not present` 分支退役（结构上不可达）。

### 5.2 T-03 iced_wgpu 补丁（模式②）

新增 `crates/patches/iced_wgpu`（diff 最小手术，挂根 Cargo.toml
`[patch.crates-io]`，同 iced_widget 先例；补丁 README 记升级重审义务）：
1. **非 panic error handler**：compositor `with_backend` 建 device 处安装
   `on_uncaptured_error`：`eprintln!("[WGPU-DEGRADE] ...")` + 继续——校验
   错误降级为日志，进程不死（wgpu 27 该 handler 替换默认 panic 路径）；
2. **screenshot 有界化**（lib.rs:180 `Renderer::screenshot`）：
   - `viewport.physical_size()` 宽/高为 0 → 直接返回空 Vec（PLAN-019
     前科的上游级兜底，补 shell_state.rs 记录值守卫的脱_sync 窗口）；
   - `device.poll(Wait{timeout: None})` → `Some(Duration::from_secs(2))`，
     超时返回空 Vec + 诊断行（截屏失败 ≠ 进程死亡，也不阻塞主线程 2s+）；
3. **补丁 README**：内容清单 + 与 panic 证据（preserve3/reader_vm_
   1794792834.log）的对应关系 + 升级重审注记。

配套仓内硬化（同一 T-03）：`service_snapshot_requests`
（shell_state.rs:117）守卫加一层「上轮 screenshot Task 未回（in-flight）
则本轮跳过」——teardown/重配竞态窗口收敛；不改既有零尺寸守卫语义。

### 5.3 T-04/T-05 GPU fence 楔死（模式①）与帧心跳

- **T-05 帧心跳**（先落，供 T-04 判定）：渲染帧完成路径（devtools.rs 帧钩
  子既有挂点）维护原子 frame_seq + 最近帧时刻，暴露进 SharedState（MCP
  面只读）；成本：两次原子写/帧。楔死判定入口 = frame_seq 停增 ≥30s。
- **T-04 有界调查 + 决策 artifact**：
  1. 候选 A（首选）：run_vm.rs session_env 注入 `WGPU_BACKEND=dx12`（尊重
     用户显式设置：`var_os` 在位则不覆盖），重建工具链后 3× vm profile
     复现轮观测：楔死次数 / frame_seq 停增事件；
  2. 候选 B（A 无效时）：present_mode env（`PRESENT_MODE=mail-box`）复测；
  3. 候选 C（A/B 无效，**本计划默认不做**，仅评估留档）：iced_wgpu present
     补丁级超时——受限分析：acquire 楔死在主线程内，同线程无法自救，
     补丁只能转「超时报错 → iced SurfaceError::Timeout 既有重建路径」，
     可行性存疑，写入 artifact 即可；
  4. **artifact 落点**：docs/plans/evidence/plan-098-backend-decision.md：
     各候选 ×3 轮结果、选型结论、遗留风险。3× 全绿即视为「决策消除」，
     否则按 artifact 记录的缓解面收口并在 §10 挂待澄清。

### 5.4 收口（T-07/T-08）

- G4 门：auto-man 重建工具链（含本仓补丁）→ auto-reader vm profile
  R-01～05 ×3 轮；绿 → T-08 勾 debts.md DEBT-008 关闭并注本计划号。
- 回归面：auto-ui-runtime 既有测试全绿 + vue 轨不受影响（本计划不触 vue
  路径；补丁只挂 iced_wgpu）。

### 规范增量

| delta_id | 类型 | 目标 | 前/后规则 | 依据 | 验收 |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/window-runtime-resilience.md | 无 → 窗口进程韧性合同：①wgpu 校验错误/device-lost 必须降级（日志+功能面收缩），禁止进程死亡；②screenshot 失败返回空帧不阻塞主线程无界时长；③MCP 动作注入链在订阅身份漂移/会话搅动下零丢失，丢失必须可检测（seq/delivered），「socket 写成功」单独不构成投递证据 | DEBT-008 诊断三模式；desktop-acceptance-input 既有 queued-input 原则 | AC-01, AC-02, AC-03 |
| SD-02 | modify | docs/specs/runtime/desktop-acceptance-input.md | MCP action 工具响应增附 `delivered` 字段（只增不改）；注入链丢弃必须有诊断日志 | 丢失检测落地后验收脚本可判别投递 | AC-01 |

## 6. 测试设计

**单元/集成（cargo test，auto-ui-runtime）**：
- T-01b：注入链漂移模拟——spawn forwarder + stream(A) → 收 1 条 → 弃
  stream(A)（drop）→ 断口期 send 3 条 → spawn stream(B) → 断言 3 条按序
  补交（backlog）+ delivered=4；二次 spawn（历史上拿 None 的死路径）同断言；
  backlog 满丢最旧 + 丢弃计数断言。
- T-02b：seq/delivered/injection_stale 状态机——超时未交付置位、交付清位、
  `delivered` 字段序列化形态。
- T-03b（补丁级，能落地多少落多少）：offscreen 零尺寸早退返回空 Vec
  （补丁源码测试或编译期断言 + 仓内集成冒烟：0 尺寸视口请求 screenshot
  进程存活）。wgpu 错误 handler 难以单测（需真 device lost）→ 由 vm 复现
  轮 + 帧心跳日志覆盖，标记为未执行的本轮手工 oracle。
- 回归：`cargo test -p auto-ui-runtime`（受影响组：subscriptions/mcp/
  snapshot 相关既有组全绿；按 ui-test-selection spec 选组）。

**手工/复现（复现机，工具链重建后）**：
- T-06：3× `verify_featurepack_host.py --profile vm`（R-01～05），探针：
  stall_watch5 日志签名检测 + 帧心跳 + 注入 seq 差值；判据见 AC。
- 验收环境用**设计值** env（不设 300s/1e9 放大项）。

**区分力说明**：T-01b 的漂移序列是「貌似正确的错误实现」的经典区分器
（take-once 实现在断口期直接丢 3 条、二次 spawn 全灭）；T-03b 零尺寸
区分「有守卫」与「裸 create_texture」。绿色测试族本身不是 oracle——
oracle 是 §7 各 AC 的可观测行为。

## 7. 验收标准

| ID | observable 行为 | 验证方法 | owner_stage |
|---|---|---|---|
| AC-01 | 注入链在订阅身份漂移/断口期零丢失；断口期动作按序补交；丢弃（若有）必有计数与日志；MCP action 响应含 `delivered` | T-01b/T-02b 自动化测试绿 + 楔死签名（handler park 线全停）在复现轮不再出现 | work |
| AC-02 | wgpu 校验错误/teardown 竞态下窗口进程**不退出**；screenshot 失败降级为空帧/空数据 + `[WGPU-DEGRADE]` 日志；主线程不被无界 poll 阻塞 | T-03b 冒烟 + 补丁 diff 审 + 复现轮无 panic 日志（preserve 窗口日志无 wgpu error 行尾随进程死亡） | work |
| AC-03 | 复现机连续 3 轮 vm profile 无 30s+ 冻结；若发生，帧心跳能定位；后端选型有 decision artifact | T-06 三轮日志 + docs/plans/evidence/plan-098-backend-decision.md 在档 | work |
| AC-04 | **（r2 修订）本仓义务面**：vm profile ×3（设计值 env）三失效模式零复现（零 30s+ 冻结/零静默退出/零注入丢失）；notes kill 场景 native UI 可交互、诚实报错投影可见（≥1 轮实录）；残红甄别矩阵入档且残红证明非本仓三模式回归（cohort 对照） | t06_run1~3 日志 + 甄别矩阵（evidence/plan-098-backend-decision.md）+ cohort 对照轮记录 | work |
| AC-05 | 既有测试面与 vue 轨无回归；MCP HTTP 工具契约向后兼容（旧字段/语义不变） | `cargo test -p auto-ui-runtime` 全绿；mcp_server 响应 schema diff 审（只增） | work |
| AC-06 | Spec 增量发布：window-runtime-resilience.md 在档、desktop-acceptance-input.md 增量并入 | merge 时 docs/specs 文件存在且与实现一致 | merge |

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 产出 | 验证 | AC | owner_stage |
|---|---|---|---|---|---|---|---|
| T-01 | 注入链 seq/delivered 诊断插桩 + ≤3 轮复现判定（H1 实锤或降级依据） | — | subscriptions.rs:835-905、mcp_server.rs:405；复现机跑轮 | diag-note（evidence/） | 复现轮日志 + 判定记录 | AC-01 | work |
| T-02 | 注入链 root fix：进程级 forwarder + backlog + 丢失检测 + take-once 退役 | T-01 | subscriptions.rs、host.rs:528、mcp_server.rs | 重构后链路 + T-01b/T-02b 测试 | cargo test 全绿 | AC-01, AC-05 | work |
| T-03 | iced_wgpu 本地补丁（非 panic handler + screenshot 有界/零尺寸）+ service_snapshot_requests in-flight 守卫 | — | crates/patches/iced_wgpu、Cargo.toml、shell_state.rs:117 | 补丁 + README + 冒烟测试 | cargo test + 补丁挂载构建绿 | AC-02, AC-05 | work |
| T-04 | 后端决策 artifact：dx12 env 注入 ×3 轮（→B→C 递进），结论入档 | T-05（判定入口）、工具链重建 | run_vm.rs:384-413 session_env | evidence/plan-098-backend-decision.md | 复现轮日志 | AC-03 | work |
| T-05 | 帧心跳（frame_seq + 最近帧时刻 → SharedState 只读） | — | devtools.rs 帧钩子、mcp_server.rs SharedState | 心跳字段 + 只读面 | 单测 + 复现轮可读 | AC-03 | work |
| T-06 | 复现轮总验证（vm profile ×3，设计值 env，探针全开） | T-02, T-03, T-04, T-05, 工具链重建 | auto-reader 复现入口（跨仓运行） | 三轮日志 + PASS 判定 | AC-03, AC-04 | AC-04 | work |
| T-07 | 独立复审（证据绑定 + spec delta 校验） | T-06 | 本文件 §9 | 复审记录 | auto-plan-review | AC-01..05 | review |
| T-08 | 合并收口：spec 发布、debts.md DEBT-008 关闭注记（r2 口径=本仓义务面，引 evidence artifact）+ **残红立债/移交**：DEBT-009 doc-surface WebView2 CDP 起动摇摆（本仓新债）；reader 恢复语义（→READER-013/auto-reader 移交注记）；notes stall（→notes 仓既有 owner）；证据物保留裁决 | T-07 | debts.md:134、docs/specs/、.retention | 关闭记录 + DEBT-009 条目 | auto-plan-merge | AC-06 | merge |

**Handoff（work 入口）**：
- 实现 worktree：`D:/autostack/.wt/ui-098/auto-ui`，分支 `plan-098-dev`
  （goal 授权全链；主检出只做簿记）。base = `c15fe0f5`；
- 依赖兄弟树（同组 ui-098，group 布局解析）：auto-lang `fe1fbeb74`、
  auto-web `0adebc0`、auto-down `5af11a3`、auto-man `54fe407`
  （工具链重建 = 同组建 auto-man，path 依赖自动解析本 worktree 修复）；
- 首个可执行任务 T-01：在 subscriptions.rs `McpPushRecipe::stream` 与
  mcp_server.rs `send_action` 落 seq/delivered 插桩（§5.1），复现机跑轮
  工具直接复用 `D:/autostack/plan-artifacts/fix-debt008-diag/stall_watch5.py`
  （探针脚本、cdb 抓栈、本地符号均就绪）；
- ~~跨仓协调（阻塞 T-06）~~：已本地闭环——auto-man 以兄弟相对 path 引用
  auto-ui，同组 worktree 构建即含修复；T-06 用 `AUTO_EXE`/`AUTO_UI_TOOL_BIN`
  指向 ui-098 产物 + **设计值 env 覆盖**（脚本 setdefault 语义）。

### 8.1 执行进度（2026-11-16 work 线，goal 授权）

- [x] **T-02**（注入链 root fix）✅ 零丢失 gate 方案落地：rx 终生锁内
  （`ActionGate`，mcp_server.rs）+ 非阻塞 poll 流（`McpActionStream`，
  subscriptions.rs）+ watcher 200ms 心跳唤醒 + seq/delivered 计数 +
  applied 栅栏超时停滞检测 + ActionResult.delivery 字段（只增）。三
  消费方（iced push/devtools 16ms tick/headless source）统改 gate 面。
  验证：`cargo test -p auto-ui-runtime plan098_gate_tests` **4/4 绿
  0.01s**（含断口期零丢失区分器：弃 stream→断口期发 3 条→重挂按序
  补交；旧 take-once 实现在此序列全灭）；`--lib mcp_server` **41/41 绿
  0.51s**。
- [x] **T-03**（iced_wgpu 补丁 + 在途守卫）✅ `crates/patches/iced_wgpu`
  （pristine 0.14.0 + 三处手术：非 panic uncaptured handler
  `[WGPU-DEGRADE]`、screenshot 零尺寸早退、poll 2s 有界）挂载根
  Cargo.toml `[patch.crates-io]`（与 iced_widget 同机制；补丁 README
  在档）；配套 `service_snapshot_requests` in-flight 守卫（10s 过期
  自愈，SnapshotShot 回调清零）。补丁随测试构建编译通过（wgpu 27
  handler 形态 Arc 已对齐）。进程存活/降级行为由 T-06 vm 轮覆盖
  （harness 周期 screenshot 探针在压）。
- [x] **T-05**（帧心跳）✅ `FRAME_SEQ`/`FRAME_LAST_TICK_MS` 全局原子
  （mcp_server.rs，零锁——view() 在 UI 线程不与 MCP 线程争锁）；
  `view_desktop_fn` 每帧打点 + AUTO_SCHED_DIAG 门控每秒一行
  `[SCHED-DIAG] frame_seq`（watch 探针同轴日志签名）。判定语义：seq
  停增 ≥30s = 渲染环停摆。
- [x] **T-01**（诊断判定）✅ 按 §5.1 降级依据收口：三轮零楔死（无停摆
  态可抓）→ F1-F5 静态结构 + T-01b 漂移模拟单测（区分器序列）+ 在线
  证据（boundary ×2/轮、冲突横幅、抽屉搅动下 park/resume 全完成）。
  `[PLAN098]` 诊断面常驻。判定全文见 evidence artifact。
- [x] **T-04**（后端决策）✅ 候选 A（dx12）采纳：dx12 ×3 轮零冻结；
  局限（Vulkan 基线为旧构建非同批对照臂）记录在案。artifact =
  [docs/plans/evidence/plan-098-backend-decision.md](../evidence/plan-098-backend-decision.md)。
- [x] **T-06**（复现轮总验证）✅ 三轮完成（设计值 env + dx12）：
  **零冻结 / 零 panic / 零事件丢失**；run1 54 PASS、run3 55 PASS
  （U3a 诚实报错投影 PASS、冲突横幅腿 U2b/U2c/U2d run1 全 PASS）；
  cohort 旧链对照 R-03 boundary 直接崩 vs 新链全推过。残红甄别矩阵
  （WebView2 CDP 起动摇摆 / reader 恢复语义 / notes stall——全部仓外
  域）见 artifact。**G4 前半（R-01～05 全绿）依赖仓外修复 → §10 Q5
  用户裁决**。
- [ ] T-07 / T-08：待 Q5 裁决后进入。

本会话测试面（test-discipline 报表）：plan098_gate_tests ×4（新增，
0.01s，均 <10s 门）；mcp_server 内联组 41 用例（存量，0.51s）；编译
检查 auto-ui-runtime/auto-ui-tools dev profile；vm profile ×3 + cohort
对照 ×1（验收运行，非 cargo 测试）。无 PERF/release。

## 9. 复审记录

- 2026-11-16 draft（revision 1）：/auto-plan:new 起草。三失效模式落点均
  已实读代码核实（F1-F13），模式③触发路径为强推断（H1，F14 未直证），
  已由 T-01 有界诊断步兜底。`stage: new`，`outcome: pass`（授权范围内），
  `next: work`（T-01 起步；T-06 依赖工具链重建协调项）。
- 2026-11-16 work（r1，goal 授权全链）：`stage: work | plan_id: PLAN-098 |
  plan_revision: r1 | outcome: blocked（仅 AC-04 前半，仓外依赖；详见
  §10 Q5）| worktree: D:/autostack/.wt/ui-098/auto-ui | branch:
  plan-098-dev | base_commit: c15fe0f5 | code_commit: 09bd4caf |
  dependency_revisions: auto-lang fe1fbeb74 / auto-web 0adebc0 / auto-down
  5af11a3 / auto-man 54fe407（同组 ui-098）| task_ids: T-01..T-06 完成、
  T-07/T-08 待 Q5 | evidence: docs/plans/evidence/plan-098-backend-decision.md
  + D:/autostack/plan-artifacts/plan-098/（t06_run1~3 + cohort_control +
  窗口/监视日志）| blockers: §10 Q5（AC-04 前半仓外域裁决）|
  next_responsible_role: user（Q5）→ review`。
  G1-G3 达成（gate 4/4 + mcp 41/41 + 三轮零冻结/零 panic/零事件丢失 +
  cohort 对照 boundary 崩 vs 新链全推过）；实现单 commit 09bd4caf
  （52 文件，含 iced_wgpu 补丁副本）。
- `stage: goal | plan_id: PLAN-098 | plan_revision: r1 | round: 1 |
  from: work | to: user（Q5 裁决） | status: executing | outcome: waiting |
  worktree: D:/autostack/.wt/ui-098/auto-ui | commit: 09bd4caf | next: user`。
  裁决问题见 §10 Q5：(a) 修订 AC-04 为本仓义务面收口继续 review/merge，
  (b) 维持原文等仓外修复后复跑（计划挂起）。
- 2026-11-16 revision 2（r1→r2，needs_replan 有界修订；**用户明示授权**：
  §10 Q5 裁决选 (a)）：目标/范围/任务集不变，仅验收措辞对齐证据现实——
  G4 与 AC-04 改为本仓义务面（三模式零复现 ×3 + 可交互 + 诚实投影 +
  甄别矩阵），T-08 增残红立债/移交（DEBT-009 新债 + reader/notes 移交
  注记）。r1 原文保留于本节历史（「AC-04 DEBT-008 关闭条件：auto-reader
  vm profile R-01～05 全绿…」）。受影响验收：AC-04（改）；其余 AC 不变。
- 2026-11-16 work（r2 复核，goal 授权）：r2 修订为簿记性（无代码变更，
  code_commit 仍 09bd4caf）；r2 下 AC-01..AC-06 全部达成——AC-01
  （gate 4/4 + 三轮零丢失签名）、AC-02（补丁挂载 + 三轮零 panic）、
  AC-03（dx12 ×3 零冻结 + artifact 在档）、AC-04 r2（三轮矩阵 + run3
  U3a 诚实投影 + cohort 对照）、AC-05（mcp 41/41 + 契约只增 diff 审）、
  AC-06 待 merge。`stage: work | plan_revision: r2 | outcome: pass |
  worktree: D:/autostack/.wt/ui-098/auto-ui | branch: plan-098-dev |
  base_commit: c15fe0f5 | code_commit: 09bd4caf | task_ids: T-01..T-06 |
  evidence: evidence/plan-098-backend-decision.md + plan-artifacts/plan-098/
  | blockers: 无 | next_responsible_role: review（review_scope: final）`。
  status: execution_done。
- 2026-11-16 **final review**（r2）：`stage: review | review_scope: final |
  reviewer_identity: 同会话复核（实现+终审同 session—— limitation 在案；
  判定由工件重构而非执行者摘要：独立复跑 + diff 复审 + wire 追踪）|
  plan_id: PLAN-098 | plan_revision: r2 | outcome: pass |
  worktree: D:/autostack/.wt/ui-098/auto-ui | branch: plan-098-dev |
  reviewed_commit: 09bd4caf45edb141b46341e3c49bcb9de80a3005（树干净）|
  base_commit: c15fe0f5 | dependency_revisions: 同 work receipt |
  spec_inputs: docs/specs/runtime/desktop-acceptance-input.md（现文已有
  queued-input 原则，与 SD-02 无冲突）+ SD-01 新文件（merge 发布）|
  acceptance_results: AC-01 pass（gate 4/4 独立复跑 0.01s + 三轮
  park/resume 全完成签名）/ AC-02 pass（补丁 diff 复审：Arc handler 形态
  对齐 wgpu 27、零尺寸早退不触碰未 map buffer、2s 有界 + 三轮零 panic）
  / AC-03 pass（dx12 ×3 零冻结 + artifact）/ AC-04 pass（r2 口径：三轮
  矩阵 + run3 U3a 诚实投影 + cohort 对照 boundary 崩）/ AC-05 pass
  （mcp 41/41 复跑 + wire 追踪：ActionResult 走 to_aura_string 文本
  信封，delivery 行纯增量，regex/contains 消费面零影响）/ AC-06 pending
  （merge-owned：spec 发布 + DEBT-009 立债）| pending_closeout_gates:
  AC-06 + 仓内统一回归（归 merge）| findings: 无需修复项；备注
  （非阻断）：终审期间机器上有他 session 的 cargo 构建在跑（PID 推进
  中），本审 scoped 复跑在其前后完成无锁冲突 | evidence: 本节 + evidence/
  plan-098-backend-decision.md + plan-artifacts/plan-098/ | next: merge`。
  **测试面报表**（test-discipline）：plan098_gate_tests ×4 独立复跑
  （新增用例：断口期零丢失区分器/mapped_none 计数/交付快照/wake 链——
  各 <0.01s，≤10s 门全过，dev profile，进程内无构建）；mcp_server 41
  （存量，0.51s 复用 work 轮证据，代码未变）。无新增慢用例。
- 2026-11-16 **merge**（PLAN-098:r2）：`stage: merge | plan_id: PLAN-098 |
  plan_revision: r2 | outcome: pass | delivery_commit: 1b3a02b6
  （master ff-only 线性落地；=09bd4caf 实现提交 + 1b3a02b6 spec 提交，
  docs-only 后裔核过实现/依赖零变更）| canonical_specs:
  docs/specs/runtime/window-runtime-resilience.md（新增，WRR-01..04）+
  docs/specs/runtime/desktop-acceptance-input.md（追加动作交付证据段）——
  landed 内容已核（HEAD:file 读取）| archive: docs/plans/archived/ |
  cleanup: ui-098 组五树（auto-ui + auto-lang/auto-web/auto-down/auto-man
  依赖兄弟）+ plan-098-dev 分支删除；plan-artifacts/plan-098/ 折叠后删除
  （判定 artifact 已入仓 docs/plans/evidence/）| debts: DEBT-008 本仓义务
  关闭注记 + DEBT-009（WebView2 CDP 摇摆）立债 | deployment_observation:
  修复在 master，用户日常工具链（钉死 debug 11-14 / cohort）为旧构建，
  需重建重部署后日常才生效（验收已在 ui-098 构建上完成）| 测试面：无新
  cargo 用例（复用终审复跑证据）；主检出 smoke check 提交后记录`。
  **landed/canonical_verified/archived/cleaned 全链核过**：master tip=
  1b3a02b6（ff-only 无合并提交）；canonical 两文件 HEAD 读取在案；
  归档提交 40c06eb5（status: archived + debts DEBT-008 关闭注记 +
  DEBT-009 立债 + evidence artifact 入仓）；主检出 smoke check
  `cargo check -p auto-ui-runtime` Finished dev 9m05s exit 0；
  ui-098 组五树（auto-ui + 四依赖兄弟）guard-clean 全拆、组目录删除、
  plan-098-dev 分支删除（was 1b3a02b6）；plan-artifacts/plan-098/
  1.5MB 整删（durable 证据在归档计划 + 仓内 artifact）、registry 刷新
  （deleted-at-merge no holds；fix-debt008-diag 释放建议已记录）。
  completion_kind: delivered。最终任务计数：T-01..T-08 全部完成
  （T-07 review pass r2、T-08 merge pass），current_step=8/8。

## 10. 待澄清事项

| # | 事项 | 影响 | 负责人/下一步 |
|---|---|---|---|
| Q1 | ~~auto-man 工具链重建时机~~ **已解**：auto-man 兄弟相对 path 引用 auto-ui，同组 worktree 构建即含修复（54fe407 + plan-098-dev@09bd4caf） | — | 已闭环 |
| Q2 | H1 证伪分支未触发（三轮零楔死，按 §5.1 降级依据收口） | — | 已闭环（artifact） |
| Q3 | dx12 副作用面：仅 vm-session 子进程 env，不污染全局 | 低 | artifact 记录在案 |
| Q4 | wgpu 上游 acquire fence 修复可升性 | 模式①根除可能 | 未检索（dx12 已消除症状面，留观） |
| **Q5** | ~~AC-04 前半仓外依赖裁决~~ **已裁决（2026-11-16 用户）：选 (a)** ——AC-04 修订为本仓义务面（G4/AC-04/T-08 r2 同步改），残红立债移交（DEBT-009 本仓新债 + reader/notes 移交注记），计划继续 review/merge。 | — | 已闭环 |
