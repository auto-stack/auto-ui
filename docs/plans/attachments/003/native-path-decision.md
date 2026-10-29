# PLAN-003 native path decision

Date: 2026-10-29
Plan revision at closure: r2 (execution began under r1; r2 records the
observed native frontend command and keeps Blog data persistence in the
consumer-owned BLOG-004 gate).
AutoUI worktree: `D:/autostack/.wt/ui-003/auto-ui`, branch
`codex/plan-003-dev`, baseline `a226962eae3caf319ebe3e1a2b91ed380eab40cb`.
AutoLang sibling: `D:/autostack/.wt/ui-003/auto-lang`, commit
`09bb414875d4e15050ecbbaf3959b1d72976d857`.
CLI/tool: `auto 0.1.0+v0.0.20-118-ge6cc06f`, `auto-ui-tool 0.1.0`, Cargo
`1.98.0`.

## Chosen run path

`auto run -r vm` launches the native VM frontend. A project's `pac.at`
`api: "rust"` declaration selects the generated Rust HTTP backend; the
run used `-B <api-port> -F <front-port>` and a separate
`AUTOUI_MCP_PORT`. `--server vm` is a backend selector and is not the native
frontend selector. It was removed from the documented command because this
acceptance run needs the app-declared Rust API and the real native frontend.

The concrete control is
`crates/auto-ui-tools/tests/fixtures/plan003-native-vm-control/`: its Rust
API is real generated backend code, its native app has a textarea and Save
button, and its state uses the existing AutoLang verifier protocol at
`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`. The event path is
textarea input → `App.InputChanged` → `AcceptanceStore.SetInput` → Save button
→ `AcceptanceStore.Submit` → generated Rust `POST /api/todos` → actual
`GET /api/todos` readback. The fixture is a framework positive control, not a
Blog replacement and not a persistent-data test.

## Native-session evidence

The durable fixture replay captured a visible native window in each session:
title `原生交互验收`, first handle `0xBC0C60`, reopened handle `0x1210FAA`.
The current-tree `autoui_type` event rendered both lines of
`UI003原生窗口 20261029-202348\n第二行确认中文换行经真实API到达后端。`; pressing
Save caused the Rust endpoint to return the exact two-line value. After `q`,
the CLI exited 0, all six PIDs in the owned process tree exited, and the API
and MCP ports could be rebound. A fresh native session reopened and accepted
the event again. Its API returned the exact value plus ` 重开`.

U-03 called `autoui_snapshot` through the real MCP client on an unused
loopback port. It returned `ConnectionRefusedError (WinError 10061)` in
2.052 seconds; the same fixture's live MCP action and API readback passed.

## Blog consumer diagnosis

The shared Blog checkout had an active release build and uncommitted
BLOG-004/attachment updates. It was not modified. The committed Blog source at
`2a7c9586924a2d42e5d0a9360bcbe265ba90dcb0` was extracted read-only into this
Plan's ignored scratch area and run with its own `AUTO_BLOG_DATA` directory,
the AutoLang sibling commit above, and the same AutoUI CLI/tool. This avoids
sharing its generated files, target directory, or draft data.

The Blog run selected native VM plus its declared Rust API, generated the Rust
backend, then failed its API readiness gate after the generated backend
reported 37 Rust compile errors (31 warnings). Diagnostics point to generated
`drafts.rs` and `works.rs` from Blog's `src/back/drafts.at` and
`src/back/works.at`—including owned `String` values passed where generated
helpers require `&str`, and an `Option<i64>` return where the generated
function requires `i64`. The CLI exited 1 with
`API backend not ready within 120s ... aborting before UI init`; there was no
Blog native window, MCP endpoint, or draft-persistence result. The failure is
pre-window and belongs on the Blog backend/source compatibility path owned by
BLOG-004 (with AutoLang lowering to be routed there if the source contract is
intended to accept those expressions). It is not evidence of an AutoUI native
window defect, and it is not a Blog pass.

An earlier `015-notes` probe also stopped at app-source parse errors; it was
discarded as an app/example input failure and is not used as a positive
control. No AutoUI runtime or process-host code change was justified by the
observed framework control.

## Consumer handoff

BLOG-004 still owns the actual Blog Rust backend build and its durable draft
acceptance. It should resolve the source/codegen errors above, then rerun the
author-window save/reopen scenario against Blog's isolated `AUTO_BLOG_DATA`.
PLAN-003 closes the AutoUI native-session prerequisite with its control
fixture and leaves no claim that Blog's Rust composition or persistence passed.
