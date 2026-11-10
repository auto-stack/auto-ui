# Native VM application validation

Status: current contract, introduced by `auto-ui/PLAN-003` (2026-10-29).

This specification defines the AutoUI-owned portion of a native VM app
acceptance run. Application data semantics and consumer-specific backend
contracts remain with their application owner.

## NVM-01 — Select the native VM frontend

Launch the application with `auto run -r vm` and the application's declared
backend. `-r vm` selects the native VM frontend. `--server vm` selects the
backend implementation and does not select a native window; for an app whose
`pac.at` declares `api: "rust"`, keep the Rust backend selection. Record the
exact command, CLI/tool version, AutoUI revision, AutoLang revision, source
root, and isolated API/MCP/frontend ports.

## NVM-02 — Prove the window belongs to this run

Associate the started CLI PID with its owned descendant process tree. Identify
the visible native window by a nonzero OS window handle and title, and observe
a nonempty rendered root/control tree through the running session's MCP
endpoint. A screenshot, process-start message, open HTTP port, or empty tree
alone does not establish a usable native app.

## NVM-03 — Prove a real event reached the app and backend

Discover current controls from the live tree; do not hardcode element IDs.
Send text or activation through the running app's MCP action tools. Confirm the
rendered state reflects the event and read the exact resulting value through
the app's real API or its declared durable store. A direct API write followed
by a screenshot does not prove the native input path.

## NVM-04 — Pair failures with an accepted control

For an unavailable or closed MCP target, call the real MCP tool and require a
bounded connection-refused result. An HTTP response or an unbounded timeout is
not the negative oracle. Pair the same MCP method against the live target,
then require the positive native action and backend readback before
attributing the negative result.

## NVM-05 — Close and reopen only the owned session

Send the session's documented exit input, require a successful CLI exit, and
verify every PID in the captured owned tree has exited. Require the old MCP and
API ports to be available for rebinding. Reopen the same app/configuration in
a fresh session and prove it accepts a new native action and backend read.
When the application contract declares durable data, verify that data through
the application's own persistence contract; the AutoUI control fixture does
not claim application-level persistence.

## NVM-06 — Classify failures at the first failed boundary

If parsing, API generation, backend compilation/readiness, or another
pre-window dependency fails before a native root is created, record the
failing phase and its owning source. Do not count the run as a native-window
pass or attribute an application/backend failure to AutoUI without evidence
of an AutoUI-owned defect. Consumer issues remain open in the consumer's plan.

## Evidence record

The execution record must include command and pinned revisions, native window
identity, current-tree discovery, real input/action result, backend or durable
readback, negative-control result, owned process/port cleanup, reopen result,
and local evidence paths with hashes. Raw screenshots and logs remain local;
the Plan and execution receipt carry the concise results and hashes.

## Windows system accessibility and suspended routes

For changes to native semantics, query the actual Windows system tree and
record role, label, value, focus, visible bounds, and action outcomes on the
affected controls. MCP tree/action evidence under NVM-03 remains valid for
that event path, but does not prove a system provider or physical input.
Record tool refusal and user-performed input separately. See
[native-system-accessibility](native-system-accessibility.md) for the bounded
provider contract and supported converted views.

For suspended routed handlers, verify the actual child result is visible
after HTTP or CPU completion, navigation writes the app root route and
parameters, and back history works. Include a legal root control and an
unmount/remount late-completion control. A root state snapshot cannot prove
that a routed child's local result rendered correctly.

Consumer evidence must identify the exact tool binary hash as well as source
and dependency revisions. If a diagnostic binary predates a fix, retain its
evidence under that binary; rebuild and replay affected consumer behavior
before closing the fix. Deliver the consumer's usable pins and required
artifacts before removing a prerequisite plan's temporary scope.

For a max-height-capped overflow column, its background/border viewport wrapper
must preserve the scrollable's intrinsic-height contract. A later tall sibling
must not turn an earlier short result into a zero-height flex allocation.
Uncapped panes retain their stretching behavior. The permanent PLAN-068 layout
controls cover short and overflowing siblings, paint/no-paint wrappers,
explicit `h-full`, and captured/ordinary views; those measured controls still
require a real consumer replay before an application result-display gap closes.

## 原生多文件选择（PLAN-988）

公开 `dialog_open_files(filter)` 由 VM native/provider 与生成 Rust
`ui::dialog::open_files` 接到共同 rfd.pick_files 实现。一次选择返回 JSON
路径数组，取消返回 `[]`；filter 解析复用单选，不复制另一套规则。
VM 默认父窗绑定保持，生成 Rust 沿既有 unparented 门面语义；这不是生成
应用父窗接线的修复。旧 provider 未实现该能力时明确拒绝。

永久真实宿主测试：`plan988_real_multiselect_vm_and_rust_host`（ignored，
需明确 `--ignored --test-threads=1` 执行）。它操作自有进程的真实系统
模态，选择两份隔离文件/取消，检查实际返回路径、在位 owner 与关闭后
owner enabled；provider 单测与源码生成文本断言不能替代该层证据。
Reader 的真实按钮→picker→导入与持久化完整验收归 READER-009。

## 页面保留字段的宿主策略播种（PLAN-079 SD-02，D3 定谳）

- 页面创建/导航路径上，新页面 widget 的 `dark_mode` 保留字段按宿主策略
  重播种——与壳根同字段同源：均读 `style::theme::dark_mode()`（desktop
  宿主下由 AUTOOS_DESKTOP_CONFIG 派生链汇入该全局；`allocate_app` 对顶层
  App 的播种语义不变）。播种发生在 Init demand 派发时（Missing/正常路径
  均覆盖），值变才 `bump_path`（PLAN-045 T-02 纪律）。
- 未声明 `dark_mode` 的 widget 不被增字段（负控：无该 model 变量的页面
  播种为 no-op）。
- 显式选择的优先语义不变：`theme_explicit` 由页面自身派生逻辑裁决
  （059 follow tier），运行时只喂派生输入，不写主题选择本身。
- 永久反例：`ui::vm_bridge::tests::f2_navigation_seeds_host_policy_dark_mode`
  （宿主 light→dark→light 三次导航播种断言 + 未声明字段负控）。

## MCP synthetic text input contract (PLAN-069 SD-03)
- `autoui_type` performs exactly ONE input dispatch carrying the whole typed
  text. The typed value replaces the bound field (binding writeback precedes
  the handler); it must not be delivered as a clear dispatch followed by a
  second dispatch to the same handler. A handler that parked on HTTP between
  the two dispatches made the second dispatch hit the parked-segment
  single-flight guard (`re-entry ignored (segment in flight)`), silently
  dropping the typed value (READER-009 round-19 live evidence: the store query
  stayed empty and the preferences write persisted an empty query).
- `clear_first` remains accepted for API compatibility; both values take the
  single-dispatch path. An explicit empty-value dispatch is requested with
  `autoui_action` `clear`, which keeps its Clear semantics.
- The parked-segment re-entry policy is unchanged: a second dispatch to a
  handler whose segment is in flight is still ignored (one segment per
  `(widget, handler)` fn name). Callers needing a reset must wait for the
  segment to settle or use a distinct handler.
- A consumer gate that previously worked around the double dispatch with
  `clear_first=false` remains valid (single dispatch either way); the default
  path must reach the same observable state.
- Consumption evidence must use the exact tool binary hash and source
  revisions (see the evidence record above); a fixed runtime is not verified
  by an older consumer binary.
- Permanent regressions: `tests_plan069_f2` (dispatch count and value for the
  default path, explicit false, and explicit `autoui_action clear`).
