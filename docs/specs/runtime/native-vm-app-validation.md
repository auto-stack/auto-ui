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
