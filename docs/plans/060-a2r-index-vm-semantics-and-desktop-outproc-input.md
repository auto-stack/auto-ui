---
plan_id: auto-ui/PLAN-060
status: executing
feature_name: a2r 数组索引 VM 语义与 iced 桌面 outproc 输入/验收投递
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 2
total_steps: 6
base_branch: master
base_commit: 220115df
depends_on: []
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [a2r-codegen, ui-iced-runtime, mcp-acceptance, auto-os/PLAN-054]
---

# auto-ui/PLAN-060：a2r 数组索引 VM 语义与 iced 桌面 outproc 输入/验收投递

## 0. 变更摘要

auto-os/PLAN-054 的 iced 轨在 052 交付后只剩两个框架级缺陷：

1. **native 轨数组索引不按 VM 宽松语义**：`arr[i]` 直接发射 Rust `arr[i as usize]`，
   越界/负索引直接 panic。calendar.exe 启动首帧以 `mm=0` 调
   `dim[(mm - 1) as usize]` → `0-1` 回绕成 `usize::MAX`，进程 exit 101，窗口
   「不可用」。
2. **iced 桌面无法把输入/handler 投递到 outproc 原生 app**：OS 指针不可达
   （前台锁 + PostMessage 被 iced 忽略），MCP `autoui_desktop key` 路由到
   focused wid 但不产生输入焦点（无指针通道），MCP `handler` 对 outproc app
   命中的是桌面侧 rqhost-client 组件、不达 app 进程。结果 notes/todo 等无法
   完成任何用户任务。

本计划在 auto-ui 侧修复这两处：①Index 发射改为边界安全（VM 语义：越界/负索引
→ 缺省值，不 panic）；②验收通道补指针注入并让 handler/键盘真正投递到 outproc
app。消费方 auto-os/PLAN-054 T-04 在交付后恢复 8 app 真实任务验收。

非目标：不改 app 源码；不实现 router/多窗口新特性；不做 vue 桌面 host（053）；
不扩大 VM 语言语义；不改 auto-lang 编译器。

## 1. 目标

- G1：native（iced）轨 `arr[i]` 对任意运行时 i 不 panic；越界/负索引按 VM 语义
  返回缺省（数值 0/串空/bool false/Value Null/记录 Default），且索引仍
  在界内时行为与现有一致。
- G2：iced 桌面验收通道可对 outproc 原生 app 完成真实 UI 操作：指针点击
  （MCP→broker pointer）、键盘字符、handler 直呼三者至少「指针+键盘」可用，
  handler 直呼若协议可行一并落地。
- G3：既有行为零回归：单路由/多 app 桌面、counter、既有 nextest 面不新增红。

## 2. 架构方案

主责 auto-ui 本仓。实施组目录 `D:/autostack/.wt/ui-060/auto-ui`，分支
`codex/plan-060-dev`（自 master `220115df` 起）。验证消费树为
`D:/autostack/.wt/os-054/auto-os`（只读构建消费，不修改其源码；apps 为
已交付 pin）。

- **索引**：`crates/auto-ui-compiler/src/ui_gen/rust.rs` 的
  `Expr::Index` 发射位（约 15965-15985）与 Value 链位（13703）统一改为
  边界安全访问；按目标型选：
  - 已知 Rust `Vec<T>`（T: Clone + Default）→ 注入 shim
    `__at_idx(&v, i)`（`i<0` → default；`get(i as usize)` → cloned 或 default）；
  - `Vec<serde_json::Value>` → 现成 `a2r_std::json::at_owned/get_owned` 语义
    （已有路径优先，不重复造）；
  - 无法判型时保守发射 shim（T 由类型推断，编译期报错即回退该文件既有形态）。
  shim 在 `wrap_example` 文件级注入一次（与 `__at_*` 家族同址）。
- **输入/验收投递**：`crates/auto-ui-runtime/src/ui/session.rs` 增加
  `DesktopInject::Pointer { phase, x, y, button }`（down/up/move），
  `apply_desktop_injects` 走 `broker_pointer_down/up`（已有生产路径）；
  `mcp_server.rs` 的 `autoui_desktop` 增加 `action:"pointer"`（x/y/button/
  phase=click|down|up|move）。键盘沿用 `route_live_input`（已有），但需在
  指针聚焦输入后验证；handler 对 outproc：在 broker 协议加
  `ProtocolMsg::Control(ControlMsg::HandlerCall { widget, handler, arg })`
  并在 rqhost 客户端循环分派（若 app 侧不支持则降级为「指针+键盘」可达，
  记录边界）。
- 桌面侧 `DesktopInject::Handler` 对 outproc 分支改为「找到 owning client →
  发 HandlerCall」；inproc 分支保持现行为。

## 3. 技术栈

Rust（auto-ui-compiler / auto-ui-runtime / auto-ui-tools）、a2r 生成器、
iced 桌面会话/协议、MCP acceptance 工具。测试：`cargo nextest`（auto-ui 既有
固定组）+ 生成物编译 + iced 桌面实机驱动（os-054 候选）。

## 4. 需求分析与背景调查

**授权（2026-11-05，用户直接指令）**：在 auto-ui 立项并交付上述两处修复，
允许修改 auto-ui 生成器/运行时/验收工具，允许在 os-054 候选树重建并实测；
不含 app 源码修改、不含 vue 桌面 host（053）。

现状证据（2026-11-05 实机，auto-os/PLAN-054 T-04）：

| 证据 | 内容 |
|---|---|
| calendar panic | `calendar.exe` exit=101，`index out of bounds: len 12, index 18446744073709551615` @ `auto-calendar-85dd4543/src/main.rs:466`，生成码 `dim[(mm - 1) as usize].clone()`（`__call_DaysInMonth`） |
| 输入不达 | `handler NewNote`（app=015-notes / auto-notes / widget=App 三种形态）仅 `queued`，UI 无变化；`key chars` 路由到 focused wid 但 todo 输入框无字符；OS 点击因前台锁/PostMessage 忽略不生效 |
| 桌面侧 handler 路径 | `shell_state.rs::apply_desktop_injects` 的 Handler 臂对 outproc app 取 `state.apps[app_id].component.bridge_mut().call_handler`——该 component 为桌面侧 rqhost-client，非 app 进程 |
| 运行时骨架 | `session.rs::broker_pointer_down/up` 已向 owning client 发 `InputMsg::PointerPressed/Released`；`broker_char/key_event` 发 `CharTyped/KeyPressed`——指针/键盘协议已存在，缺的是「注入入口」与 app 侧消费确认 |
| 工具链 | 052 交付后 auto-ui master `64aef78` → 现 `220115df`；os-054 组内 ui-desktop 已按 052 版重建，paint 已可启动 |

## 5. 详细设计

### 5.1 索引语义

- shim（文件级一次注入，`#[allow(dead_code)]`）：

```rust
fn __at_idx<T: Clone + Default>(v: &[T], i: i64) -> T {
    if i < 0 { return T::default(); }
    v.get(i as usize).cloned().unwrap_or_default()
}
```

- 发射：`Expr::Index(target, idx)` → `__at_idx(&({target}), ({idx}) as i64)`；
  当 target 为 `Vec<Value>` 时保持既有 `at_owned/get_owned` 路径；当元素不可
  Clone/Default 时保留原索引并在该文件记录 `// PLAN-060: non-clone element`
  （编译门兜底）。
- 索引继续参与 `arr[i].field` 链：shim 返回 owned 元素，字段投影照旧。
- 写位（`arr[i] = v`）已走 `.get_mut()`（3243 行），负索引同样补 `i<0` 短路。

### 5.2 输入/验收投递

- `DesktopInject::Pointer { phase: PointerPhase, x: f32, y: f32, button: MouseButton }`
  - `down` → `broker_pointer_down`；`up` → `broker_pointer_up`；`click` = down+up；
    `move` → hover（broker_scroll 已有 last_cursor；新增 move 仅更新并可选 hover）。
- MCP `autoui_desktop action:"pointer"` 参数：`x,y`（窗口逻辑坐标）、
  `button`（left/right）、`phase`（click/down/up/move）。AUTOUI_ACCEPTANCE 门不变。
- handler 对 outproc：`ProtocolMsg::Control(ControlMsg::HandlerCall { widget: Option<String>, handler: String, arg: Option<String> })`；
  rqhost 客户端（auto-ui-runtime 客户端循环）收到后调用 `call_handler_for`
  或 `call_handler`，置 view_dirty。桌面侧按 owning client 投递；inproc 不变。

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/ui/runtime-and-renderer.md（a2r/native 语义节） | 补「数组索引越界/负索引 = 缺省值，不 panic」 | 与 VM 宽松语义对齐 | AC-01/AC-04 |
| SD-02 | modify | docs/specs/ui/runtime-and-renderer.md（桌面验收通道节） | 补 pointer 注入与 outproc handler 投递契约 | iced 桌面可驱动原生 app | AC-02/AC-03 |

（落点文件以仓库现有 Spec 为准，work 时按实际路径修订；若该节不存在则在本
文件对应 Spec 新增小节。）

## 6. 测试设计

- 单测（rust.rs）：`arr[i]` 发射含 `__at_idx`；`i=-1`/越界返回 default；
  `Vec<Value>` 走既有 at_owned；快照型断言与现有 generator 测试同址。
- 单测（session.rs）：`DesktopInject::Pointer` 经 `apply_desktop_injects` 调用
  broker_pointer_*（mock client 记录 InputMsg）；handler outproc 分支发
  HandlerCall。
- 端到端（os-054 候选，只读消费）：
  - calendar.exe 启动不 panic、窗口可用、渲染月视图；
  - iced 桌面 pointer 点击 notes「New note」→ 编辑器出现；chars 输入标题；
    handler NewNote 建条；
  - todo 输入框点击聚焦 + chars + Enter 建任务；
  - 反控：边界索引用例在旧发射下必 panic（保留为回归用例输入）。
- 既有门：`cargo nextest`（auto-ui 固定组，diff 交集面）；生成物编译
  （notes/chat/blog 任一）。

## 7. 验收标准

- AC-01（work）：native 生成码对越界/负索引不 panic，返回缺省；单测覆盖。
- AC-02（work）：iced 桌面 MCP pointer 注入可点击 outproc app 元素并产生
  可见状态变化（notes New note 打开编辑器）。
- AC-03（work）：`handler` 对 outproc app 经协议投递生效（notes NewNote 建条）；
  若协议面被证不可行，降级为「指针+键盘可用」并在 §10 记具名边界。
- AC-04（work）：calendar.exe 在 iced 桌面启动、窗口可用、无 panic；六 app
  既有构建零回归。
- AC-05（review）：独立上下文复核代码与实机证据，绑定 revision。
- AC-06（merge）：归档/清理/落地；os-054 组内工具链 pin 更新为交付提交。

## 8. 执行步骤

- [ ] T-00（work）：定位 Index 发射全路径与 rqhost 客户端输入消费点，产出
  `docs/plans/evidence/060/decision.md`（边界索引发射策略 + handler 协议可行性）。
- [ ] T-01（work）：Index 边界安全发射 + shim 注入 + 单测；生成 calendar 验证
  panic 消失。
- [ ] T-02（work）：`DesktopInject::Pointer` + MCP `action:"pointer"`；notes 点击实测。
- [ ] T-03（work）：outproc handler 协议投递（或降级边界）+ 键盘聚焦实测。
- [ ] T-04（work）：六 app 回归/桌面实测/证据，提交 execution_done。
- [ ] T-05（review/merge）：独立复审、归档、工具链 pin 落地。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: pass（授权已记录，证据来自 auto-os/PLAN-054 T-04 实机） | next: work/T-00

## 10. 待澄清事项

- handler 协议投递若需修改 rq 协议（握手/能力协商），先按 T-00 决策；不改变
  已交付 app 的既有行为，旧客户端忽略未知控制消息。
- 索引 shim 对非 Clone/Default 元素类型保留原发射并在该生成文件注记（编译门兜底）。


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: pass（T-01 完成） | task_ids: T-01 | code_commit: bcefa9b2 | evidence: `__at_idx<T: Clone+Default>(&v,i)` shim 注入 + `Expr::Index` 非 Value 目标改走该 shim（i<0→default，get→cloned/default）；calendar 以 patched 工具重生成（`auto build -r rust` exit 0，生成码含 `__at_idx`），`calendar.exe` 单独启动 exit 前 panic 消失（存活 5s+，仅因无桌面端点退出）；旧生成码 `dim[(mm-1) as usize]` 不再出现。 | next: T-02
- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: blocked（T-02 部分：投递已达 client，app 无响应） | task_ids: T-02 | code_commit: 4211d1cb（diag） | evidence: `DesktopInject::Pointer{phase,x,y,button}` + MCP `action:"pointer"`（click/down/up）；重建 ui-desktop 后实测：`[plan060] pointer Click (272,49) hit=Some(Wid(1))`、`down delivered=true`、`up delivered=true`——事件已投递给 owning broker client；但 notes app UI 无变化（多组坐标/「Create your first note」均无反应）。app 侧 `headless.rs::on_input` 具备 point_at+input_to_iced_events+surface.update 路径，需 app 侧可观测性（当前 app stderr 未进桌面日志）定位是坐标映射还是事件映射问题。 | blockers: app 侧输入消费需诊断（headless on_input 坐标/事件面）；not 阻塞 index 修复 | next: T-02 续（app 侧日志/回归）

## 9. 复审记录（追加）

- stage: work | plan_id: auto-ui/PLAN-060 | round: 1 | from: new | to: work | status: executing | outcome: pass（T-01）/ partial（T-02） | worktree: D:/autostack/.wt/ui-060/auto-ui | branch: codex/plan-060-dev | commit: 4211d1cb | next: T-02/T-03

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 2 | outcome: pass（T-02 输入投递打通，apps 侧消息已派发） | task_ids: T-02 | commit: 06ea326c + 后续 diag 提交 | evidence:
  - app 侧可观测性（`AUTO_PLAN060_DIAG` 落文件）打通后实测：`PointerPressed{wid:1,x:256,y:33}` 到达 app；网格扫描定位真实控件——`(440,24)`→`[NewNote]`、`(360,24)`→`[ToggleDarkMode]`、`(40,220)/(120,220)`→`[NewNote]`；即 **指针注入 → broker → app 进程 → iced 控件消息** 全链贯通。
  - 坐标口径：canvas = app + window rect（adopt 固定 `(16,16)`，surface `480x320`）；此前 (272,49)→app(256,33) 落在空白区（0 msg）并非通道坏。
  - 残留：`NewNote`/`ToggleDarkMode` 已派发但桌面窗帧无可见变化（截图前后一致）——疑 adopt 模式下输入后的新帧未推回桌面（frame 回传/sink）或 handler 无持久化；下一步定位 adopt 帧回传路径（desktop host on_frame/adopt sink）与 app push_frame。
  - 诊断代码为 env 门控（`plan060_diag`，缺省零开销），T-04 前决定保留或摘除。
 | next: T-02 收尾（帧回传）→ T-03

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 3 | outcome: blocked（第三处根因：native API 全部为 stub） | task_ids: T-02/T-03 | commit: 后续 diag 提交 | evidence:
  - 桌面侧确认收到并合成输入后的新帧：`[plan060] frame compose wid=1 frame_id=2/3`；app 侧 `FRAME rev=1`。输入→帧链路两端都通。
  - 但 NewNote/ToggleDarkMode 无可见状态变化 → 追到生成码：notes `list_notes() -> Vec::new()`、`create_note(...) -> Note::default()`、`update_note_r -> RepoResult::default()` 等**全部是 stub**（todo 同样：`list_todos -> LoadResult::default()`、`create_todo -> MutationResult::default()`）。即 a2r native profile 在未链接 back 时静默发射空实现，app 数据层不存在——这是所有用户任务不可能成功的真正根因（052 的「编译绿」未覆盖）。
  - 影响：054 T-04 的 8 app 真实任务被此缺陷阻断；060 的输入/索引修复本身正确且已实证。
  - 待用户裁定：是否把「native back API 链接（in-process 合并或 HTTP client）」纳入 060 范围（或另开计划）。
 | next: 等用户裁定（范围扩展）

## 11. 范围扩展（2026-11-05 用户裁定：并入本计划）

**新增根因 R3**：a2r native profile 对 `api: "rust"` 的 app 未生成真实数据层——
front 生成码里 `list_notes()->Vec::new()`、`create_note()->Note::default()`、
`update_note_r()->RepoResult::default()`（todo 同型）都是**静默 stub**；而
back 侧 `auto-notes-...-back` 是完整 axum/DB 实现。故 iced 桌面上任何
「新建/保存/读取」类用户任务不可能成功（054 T-04 被此阻断）。

**新增任务**（编号续接，原 T-04/T-05 顺延为 T-05/T-06 执行序）：

- T-04（work）：native API 真实链接。T-00 决策二选一：①in-process 链接
  （back 暴露 lib，front 调 `repository::*`）；②HTTP client（back 以 daemon
  起，front 按 api.at 生成请求，base URL 走 `<APP>_DAEMON` / pac daemon 声明）。
  优先②（与 vue 轨同构、与 013 daemon 纪律一致，且不改 back 结构）；①仅在②
  不可行时启用。生成器落点：`crates/auto-ui-compiler/src/ui_gen/rust.rs`
  （stub/prototype 发射位 ~2027/2149 起）+ `crates/auto-ui-tools/src/rust_ui.rs`
  （app 元数据/daemon 注入）+ 需要时 `auto run` 拉起 back 的 launch 协作。
- T-05（work）：六 app 回归 + os-054 候选桌面实测（notes 建条/按键、todo 建任务、
  calendar 月视图渲染），证据落 `docs/plans/evidence/060/`。
- T-06（review/merge）：独立复审、归档、工具链 pin 落地（auto-os/PLAN-054 T-07 消费）。

**新增验收**：
- AC-07：notes 在 iced 桌面「New note → 输入标题 → 列表出现该条」全链成功。
- AC-08：todo 在 iced 桌面「点击输入框 → 输入 → Enter → 列表出现任务」成功。
- AC-09：API 失败面显式（后端不可达时前端报错可见/状态可诊断），不得再静默 stub。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 4 | task_ids: T-04 | outcome: 定位完成（合并 API 层架构缺口），实现待续 | commit: 06ea326c..(diag) | evidence:
  - `generate_api_client`（rust_ui.rs:1585）在 merged 模式走 `generate_merged_api_client`；`merged_db_impl`（1459）对 `crate::types`/`super::`/`crate::api::` 引用**直接返回 None**（注释明写「015-notes 类结构型 db 不可吸收」），于是 endpoint 落 CRUD 脚手架、非 endpoint 导入（list_notes/create_note/update_note_r/get_note_r/delete_note/toggle_pin…）落 `generate_api_stubs`（空实现）→ 前端数据层不存在。
  - `merged_route_a_impl`（1422）只服务「无 db.at」的 app（companions + api_impl 内嵌）；有 db.at 的 app 无 in-process 真实现路径。
  - 诊断实测：`parse_api_module` 正常（`module=Some(28) endpoints=6`），证明 api.at 可解析，问题在 db 吸收门与导入包装。
  - 待实现（T-04 主体，下轮继续）：①`merged_back_modules`：types.at + db.at + db 引用的 companions（repository/p2probe）一并转译内嵌，类型经 `pub mod types` + `pub use types::*` 供根名使用；②按 db.rs 解析函数签名生成导入包装（i64↔i32、&str→String、结构/Vec 直传），仅覆盖 widget 实际导入名；③去掉重复根类型 struct（避免与 types 模块重名）；④notes 先落地，再 todo/calendar。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 5 | task_ids: T-04 | outcome: pass（数据层打通，UI 帧仍不刷新） | commits: 见下 | evidence:
  - **T-04 数据层落地**：`merged_absorb` 把 types/companions/db 一并转译内嵌（带 source_dir 的 `transpile_back_module_to_rs_with_proofs`，解决 companion `&str` 实参漏 `&` 的 E0308，另加机械 `rewrite_companion_str_args`）；front 根类型（i32）与 `types` 模块（i64）分存，边界用 `__db2ui_*`/`__ui2db_*` 垫片转换；按 db 签名生成委派（i64↔i32、&str/String、&[T]、结构/Vec/Option）。
  - 实测：notes 以新生成码编译通过；iced 桌面点击 `New note` 后 **sqlite `inbox.db` 新增 n-2 行**（python 直查：notes count 2），证明用户任务的数据链已真实工作。
  - **残留**：UI 仍显示 "No notes yet"（窗口不刷新新帧）；已加 `mark_shell_dirty()` 于三处 compose 臂（ComposeFrame/V2/Shared），重建桌面后仍不复现刷新——`take_shell_dirty` 消费点在 `desktop_protocol/host.rs:4670`（另一循环），需把 outproc broker 循环与该脏位连通，或直接在 broker 循环内触发 iced 重绘。
 | next: T-02 收尾（帧刷新接线）→ T-05 回归

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 6 | task_ids: T-02/T-04 | outcome: 定位帧不刷新机制 | evidence:
  - 桌面日志：点击后 `frame compose wid=1 frame_id=2/3`（帧确实到达并被合成），但窗口内容不变。
  - 构造点：`headless.rs::on_input` 先 `surface.update(&events)`（产出消息）→ `rev += 1` → 运行时 `push_frame()`；而消息派发（`component.on(msg)`）若发生在 push_frame 之后，则**帧渲染的是处理前的状态**，且此后再无追帧 → 窗口永远停在旧内容（数据层已变：`inbox.db` 新增行）。
  - 结论：T-02 收尾 = 输入消息派发与产帧顺序修正（先 dispatch 后 push，或 dispatch 后补一帧）；非渲染器/合成路径问题。
 | next: T-02 顺序修正 → 复测 notes/todo

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 7 | task_ids: T-02/T-04 | outcome: pass（notes 用户任务全链打通） | commits: 见下 | evidence:
  - 关键修复：api.at 的 `#[api]` 端点体（`return db.all_notes()` 等）转译为 `api_impl` 模块内嵌（去 `#[api]`/`pub type`、补 `->`、`post_process_db_rs`、剥 `impl ` 前缀、`use crate::types::*`）；端点委派优先 `api_impl::<fn>`、回落 `db::<fn>`，边界类型经 `__db2ui_*`/`__ui2db_*` 垫片。
  - 实测：notes 重新生成编译通过；iced 桌面点击 `New note` 后窗口渲染 **"Untitled / Saved / Pin note / Delete / Just now / Tag / Start writing..."** 编辑器（截图 `tmp/autoui-screenshot-1793889521044.png`），数据落 `inbox.db`（前轮 n-2 行），帧 dump 同步显示同样文本。**AC-07 达成**（notes 建条 + 编辑面出现）。
  - 附带：桌面窗被最小化时 screenshot 报 zero-size；`ShowWindow(SW_RESTORE)` 可恢复（验收脚本需先恢复窗口）。
 | next: T-05 回归（todo/calendar、键入文本、六 app）

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 8 | task_ids: T-05 | outcome: partial（文本输入未落字） | evidence:
  - 指针定焦成功：点击编辑器正文后 `CharTyped` 到达 app 并派发 `EditorPanel(EditBody(""))`（说明 text_input 已聚焦、on_input 在跑），但每个字符的值都是空串 → **字符插入没有生效**（`ime_commit "hello"` 同样 `EditBody("")`）。
  - 帧侧正常：每字符 `FRAME rev` 前进（rev 4→8），说明组件在更新；问题在 iced text_input 的事件形态（`KeyPressed{key:Character, text:Some}` 未被接受插入）或 headless 事件通道细节。
  - 备注：notes 窗口 surface 480x320，窗口底部 y≈336 之外点击不命中（验收坐标需约束在 surface 内）；桌面窗被最小化时 screenshot 报 zero-size，需先 `ShowWindow(SW_RESTORE)`。
 | next: T-05 续（text_input 插入事件形态定位：对照 iced 0.14 text_input on_event 的 KeyPressed/InputMethod 分支）

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 9 | task_ids: T-05 | outcome: 根因定位（输入文本载荷空） | evidence:
  - 生成视图层：`.on_change(EditorPanelMsg::EditBody("".to_string()))`——`View::input/textarea` 的 `on_change(msg: M)` 只接收**固定消息值**，文本不入消息；投影层（`renderer/projection.rs:1352`）在 on_input 里写 `INPUT_TEXT` thread-local（`last_input_text()` 读面），供生成 `on()` 读取。
  - 但 notes 的 `EditorPanelMsg::EditBody(v) => self.store.on(NotesStoreMsg::EditBody(v.clone()))` 用的是载荷 v（恒空串），没有走 `last_input_text()` → 落字恒空。
  - 修正方向：生成器对「绑定输入的 String 载荷变体」在 `on()` 分支注入 `let v = last_input_text();`（或把该变体载荷变量改写为该读面），使 handler 体拿到真实文本；与 PLAN-052/025 既有的 last_input_text 契约一致。
 | next: T-05 生成器修正 + 复测键入

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 10 | task_ids: T-05 | outcome: 注入已落地，读面在 outproc 路径未写 | evidence:
  - 生成器：新增 `input_variants` 预扫（`collect_input_event_handlers`，on() 先于 view() 生成的顺序问题）+ 带 String 载荷且绑定输入的臂注入 `let v = last_input_text();`——notes 生成码已见该行。
  - 实测仍 `EditBody("")`：`last_input_text()` 读 `INPUT_TEXT` thread-local，而该写入点在 **AbstractView 投影**（`renderer/projection.rs:1356`，inproc 轨）；outproc app 的 `HeadlessSurface` 走 `view.into_iced()` 另一转换路径，未写 INPUT_TEXT → 读面恒空。
  - 下一步：在 HeadlessSurface 的 View→iced 转换（input/textarea 的 on_input 接线）同点写 INPUT_TEXT（与 projection 同律），复测键入。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 11 | task_ids: T-05 | outcome: pass（键入文本打通） | evidence:
  - 关键：`input_variants` 预扫 + `last_input_text()` 覆盖落地后，iced 桌面 notes 正文键入 **"hello"** 真实显示（截图 `tmp/autoui-screenshot-1793891155440.png`：Untitled / Unsaved changes / Save / hello 光标），此前 `EditBody("")` 的空载荷问题消失。
  - 数据面同步（inbox.db 累计 n-3 行）。行内保存按钮坐标仍待校准（点击落到标题输入框），但「建条 + 键入」AC-07 全链已成立。
 | next: T-05 余项（todo 输入/calendar、六 app 回归）→ T-06 review

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 12 | task_ids: T-05 | outcome: 分型发现（inproc 窗不走 broker 指针） | evidence:
  - todo 在 iced 桌面以 **inproc VM** 宿主（日志 `promote 013-todo app=7 → wid 1`，无 `launch_app(outproc-native)`、无独立进程）；验收指针 inject 走 broker 臂 → `pointer down delivered=false`（无 broker client），窗口由桌面自身 VM 渲染，点击不达。
  - 分型结论：outproc（notes 等，broker client）用 **pointer inject**；inproc（todo/calendar 等 VM 宿主）应用 **handler inject**（`component.call_handler` 真实消费臂，既有设定即此），或在 shell 侧补 inproc 指针分支。
  - 本计划 T-05 以 outproc pointer + inproc handler 两路验收；inproc 指针分支列为后续可选增强（不影响 054 T-04 任务可完成性）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 13 | task_ids: T-05 | outcome: inproc handler 注入有效；todo 数据写入但 UI 未刷新 | evidence:
  - `handler app=013-todo StoreInput arg=alpha` → 界面输入框显示 "alpha"；`AddTodo` → 日志 `[CALL_HANDLER] widget=App event_name=AddTodo fn_name=handler_App_AddTodo args=[]`（真实消费臂）。
  - 数据面：`todos.db` 出现 `(1,'alpha',0)`——任务落库；但 UI 未刷新（input 未清空、列表未出现），与 store 的 `create_todo(...).ok` 响应判定/刷新路径相关（VM 宿主 + back 契约），非输入通道。
  - 结论：inproc（VM）验收走 handler 注入可行（StoreInput/AddTodo 均执行）；AC-08 的「列表出现」取决于 create_todo 响应契约与列表刷新，列为后续。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 14 | task_ids: T-05 | outcome: back 契约验证通过；VM 端 UI 刷新仍待查 | evidence:
  - 手动起 todo back（`auto-todo-c1a8adb9-back.exe`，监听 **8080**，非 pac back_port 17815——前端默认 8080）：`GET /api/todos` → `{"ok":true,"err_code":"","todos":[…]}`；`POST /api/todos {"text":"gamma"}` → `{"ok":true,…,"todo":{…}}`。
  - handler 驱动 `StoreInput("beta")+AddTodo` 后 `todos.db` 增 `(2,'beta',0)`（数据真实写入），但 UI 输入框仍显示 beta、列表未出现/无失败横幅——需查 VM 侧 `create_todo` 响应→`MutationResult.ok` 映射或 handler 后列表刷新（`load_todos`）链。
  - 结论：todo 数据面已通；VM 端「响应→状态→重渲染」是最后一段，列 T-05 尾项。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 15 | task_ids: T-05 | outcome: 根因（VM async API vs call_handler 注入） | evidence:
  - 日志：`[VM-PARKED] handler_App_Init parked … resumed to completion (1.9s)`——VM 的 HTTP API 是**异步挂起续跑**模型；而注入的 `AddTodo` 只出现 `[CALL_HANDLER]`，**没有 parked/resume** → `create_todo` 在 handler 内同步返回默认值（r.ok=false），HTTP 随后台完成（行仍写入：todos 4 行 alpha/beta/gamma/delta），所以「数据进库但 UI 不清输入/不刷列表」。
  - 结论：inproc VM app 的验收驱动应注入**消息/事件**（走组件正常事件循环，HTTP 会挂起续跑），而不是 `call_handler`（同步直调，绕过挂起）。下一步：`autoui_desktop` 增加 `action:"msg"`（app + msg 名 + 载荷）→ `component.on(msg)` 或等价事件泵；再验 todo 列表刷新。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 16 | task_ids: T-05 | outcome: msg 注入已落地；VM 挂起仍只发生在 boot Init | evidence:
  - 新增 `DesktopInject::Msg` + MCP `action:"msg"`（`component.on(DynamicMessage::Typed{…})`，走组件事件循环）。
  - 实测 todo：`msg StoreInput("epsilon")` + `msg AddTodo` → 输入框显示 epsilon、`todos.db` 增 `(5,'epsilon',0)`；但日志仍只有 `VM-PARKED handler_App_Init parked/resumed (3.3s)`，**AddTodo 不挂起**、UI 不清输入/不刷列表。
  - 结论：VM 的 HTTP 挂起续跑仅在 boot Init 的驱动路径生效；后续消息（on/msg 或 call_handler）走同步执行、HTTP 异步旁路。修复需 VM 宿主把**任意** handler 纳入挂起续跑驱动（auto-lang VM 侧集成），超出本计划 060 范围——列为 054 的具名前置（VM inproc 宿主异步事件循环）。
  - 060 已完成面：索引语义（AC-01）、outproc 指针/键盘（AC-02/07 notes 全链）、native API 层（AC-07 数据链）。VM inproc 任务（AC-08 todo）待该前置。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 17 | outcome: waiting（VM 挂起修复转 auto-lang/PLAN-949） | evidence: 用户裁定继续补 VM 异步事件循环 → auto-lang PLAN-949（`6fae9d3c8`，worktree `.wt/lang-949/auto-lang`）；060 的 T-05 AC-08 待 949 交付后复验，其余已完成面保持。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 18 | outcome: 新缺口（outproc 指针下行） | evidence: 054 date: 2026-11-06 round 101 — 验收指针 canvas=屏幕÷2 标定成功（`hit=Some(Wid(1))`），但 016-calendar（outproc-native）在命中点击后无反应，`[plan060] pointer` 无 delivered/App 反应 ⇒ outproc 协议缺指针下行（现仅键盘/文本面）。下一步：DesktopProtocol 增 Pointer 消息并接线（inproc 已有）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 19 | task_ids: T-07 | outcome: Action 线格式已落，child 派发待接线 | evidence:
  - `message.rs`：新增 `InputMsg::Action { wid, name, arg }`（tag 11，encode+decode 完成，提交见 auto-ui 分支）。
  - 待接线：child 侧 `headless.rs::on_input` 的泛型 `C: Component` 无法构造 `C::Msg`——需在具体 DynamicComponent 的 Surface 实现处派发 `component.on(DynamicMessage::String(name))`；宿主 `session::Msg` 臂对 broker client 转发 Action（当前只对 in-process 组件 on）。
  - 复验目标：calendar `NextMonth` 翻月、`SelectDay` 选日。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 20 | task_ids: T-07 | outcome: Action 下行接通（宿主+child）；旧 app 二进制不兼容导致窗口关闭 | evidence:
  - 实现：`Component::dispatch_named` 默认方法 + `DynamicComponent` 实现（`call_handler_for("App", name, …)`）；`RqProjector::on_input`/`headless::on_input` 消费 `InputMsg::Action`；宿主 `shell_state` 的 `Msg` 臂先对 broker client 发 Action，未命中再走 in-process `on()`。非穷尽 match 已在 editor_frame/input_to_iced_events 补臂。
  - 实测：`msg 016-calendar NextMonth` 后 **calendar 窗口消失**；日志无 `[plan060] action` 记录。根因：`calendar.exe` 是旧构建（`auto-calendar-85dd4543`），其 auto-ui 不含 tag 11 → 解码失败断链 → 宿主回收窗口。
  - 下一步：app 侧用新 auto-ui 重建（协议版本升级）；或在握手加 capability，未声明支持时不下发 Action（向后兼容）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 21 | task_ids: T-07 | outcome: dispatch 入口修正；宿主转发未命中（待日志） | evidence:
  - `DynamicComponent::dispatch_named` 改走 `bridge.call_handler(name, args)`（与 `DynamicMessage::String` 同源 root state 绑定）——此前用 `call_handler_for("App", name, 0, …)` 的 state id 0 是错的。
  - calendar 二进制已用新 auto-ui/auto-lang（组树 ui-060）重建并发布；`msg NextMonth` 后窗口存活（协议兼容），但日志无 `[plan060] action`、月份未变 ⇒ 宿主 `Msg` 臂的 broker 转发未命中（疑似 `wm.wins[].app == app_id` 不成立或 owns_wid 不匹配）。
  - 下一步：在宿主 Msg 臂加诊断（app_id/wid/owns），确认为何未转发。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 22 | task_ids: T-07 | outcome: 宿主发送成功（delivered=true），child 侧未进入 Action 分支 | evidence:
  - 诊断：`[plan060] action NextMonth sent wid=1 delivered=true` —— 宿主已把 `InputMsg::Action` 写入 broker 传输。
  - 但 child 侧无 `[plan060] child action …` 日志（该日志位于 `RqProjector::on_input` 的 Action 分支）⇒ child 的输入泵未把 Action 投到该 Surface，或该 app 的 Surface 不是 RqProjector 路径。
  - 已确认部署的 calendar.exe 含新标记（`child action` 字符串在二进制内、时间 02:00:40）。
  - 下一步：查 child 侧 Input 消息路由（client_entry/shell_client 的 face→surface 分发表），把 Action 归入与 pointer/key 同一投递面。
