# Window runtime resilience

Status: current contract, introduced by `auto-ui/PLAN-098` (2026-11-16).

This specification defines the native window process's resilience contract:
render-loop degradation, screenshot bounding, and MCP action-injection
delivery. It exists because DEBT-008 diagnosis pinned three failure modes to
the auto-ui-tool vm-session window process (GPU fence wedge, wgpu validation
panic on the offscreen screenshot path, silent event-injection loss), each of
which killed or wedged the whole window while the backend stayed healthy.

## WRR-01 — wgpu errors degrade, never kill the process

A wgpu validation or device error must not terminate the window process. The
patched `iced_wgpu` installs a non-panicking uncaptured-error handler at
device creation (`[WGPU-DEGRADE]` log + eprintln) and continues: the failing
operation degrades (screenshot returns an empty frame), the process survives,
and the MCP surface keeps serving. Upstream default behavior (panic on
uncaptured error) is the defect class here — the historical
`iced_wgpu.offscreen.source_texture is invalid` panic during teardown races
killed the window silently.

## WRR-02 — window screenshots are bounded and zero-size-safe

`Renderer::screenshot` (the offscreen capture path behind
`iced::window::screenshot`) must return promptly and safely:

- A zero-width or zero-height viewport returns an empty frame with a
  `[WGPU-DEGRADE]` log line; it must not call `create_texture` with zero
  dimensions (PLAN-019 crash class).
- The device poll waits at most 2 seconds; on timeout it returns an empty
  frame and must not read a buffer whose map has not completed. An unbounded
  `Wait { timeout: None }` on the UI thread is a defect (DEBT-008 mode-①
  family blocking surface).
- The periodic snapshot pump (`service_snapshot_requests`) issues at most one
  full-window screenshot at a time; an in-flight task suppresses re-issue for
  up to 10 seconds (self-healing), and the `SnapshotShot` callback clears the
  flag.

## WRR-03 — MCP action injection is loss-free and loss-detectable

Actions enter the window process through the MCP action gate
(`crate::ui::mcp_server::ActionGate`):

- The action channel receiver lives inside the gate for the process lifetime
  and is never moved out or dropped. Subscription identity churn (primary
  AppId change, app add/remove, subscription-table rebuild) only changes which
  stream polls the gate; actions arriving while no stream is attached stay
  queued and are delivered in order on re-attach. The retired take-once
  receiver handoff (which permanently killed the injection chain on
  resubscription) is a defect class here.
- The push subscription polls the gate without blocking; wakeups come from a
  sender-side notify (performed under the same receiver lock as the poll's
  waker registration, so no wakeup can be missed) plus a 200 ms watcher
  heartbeat that covers notify/registration interleavings.
- Delivery is observable: every accepted send increments the sent counter,
  every consumption (iced poll, devtools tick, headless source) increments
  the gate delivered counter, and unmapped targets increment a visible
  `mapped_none` counter with a log line. Action tool responses carry a
  delivery snapshot (see `desktop-acceptance-input.md`); a socket write alone
  is not delivery evidence. An applied-ack timeout with an undelivered
  sequence sets a stale flag and emits a `[PLAN098]` diagnostic line.
- The frame heartbeat (`FRAME_SEQ`/`FRAME_LAST_TICK_MS`, bumped once per
  rendered frame, zero-lock) is the render-stall triage entry: a frame_seq
  frozen for ≥30 s while the process and MCP surface stay alive indicates a
  render-loop wedge; the diag log emits a per-second `[SCHED-DIAG]
  frame_seq` signature when `AUTO_SCHED_DIAG` is enabled.

## WRR-04 — vm-session GPU backend selection

The vm-session child process defaults to the dx12 wgpu backend (injected via
`proc_host::set_vm_session_env`); an explicit user `WGPU_BACKEND` is never
overridden. Rationale: the DEBT-008 freeze signature (`ash::wait_for_fences ←
wgpu acquire_texture`) pinned the wedge to the Vulkan path; dx12 ×3
reproduction rounds showed zero freezes while the cohort Vulkan-era toolchain
wedged at R-03 boundary restarts. The frame heartbeat (WRR-03) is the
detection entry for any recurrence on any backend.
