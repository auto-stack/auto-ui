# PLAN-003 native VM control

This fixture is a small, real AutoLang UI backed by a generated Rust HTTP API.
It exercises the native VM event path without depending on Blog routing or
copying Blog's draft-storage contract. Its in-memory Todo store is deliberately
limited to the control run; persistent draft behavior remains the consumer
application's responsibility.

## Run

Use the AutoUI and AutoLang worktrees as siblings in the PLAN-003 group. Set
`AUTO_LANG_ROOT` to the AutoLang worktree, `AUTO_UI_SOURCE_ROOT` to the AutoUI
worktree, `AUTO_UI_TOOL_BIN` to its `target/debug/auto-ui-tool.exe`, and
`CARGO_TARGET_DIR` to its `target` directory. Then launch:

```powershell
$env:AUTOUI_MCP_PORT = '13140'
$env:AUTO_HTTP_PORT = '13240'
$env:AUTO_FRONT_PORT = '13340'
auto-ui003.exe run -r vm -B 13240 -F 13340
```

The `pac.at` `api: "rust"` declaration selects the generated Rust API while
`-r vm` selects the native VM frontend. Do not add `--server vm`; that option
selects a VM backend and does not request the native VM frontend.

The existing AutoLang verifier driver is at
`$env:AUTO_LANG_ROOT/.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`. Its
`AutoUiMcpClient` speaks the running server's JSON-RPC `/mcp` endpoint. Find
the textarea and Save button from the live snapshot, send input with
`autoui_type`, press Save with `autoui_action`, then verify the exact value
through `GET /api/todos`. For lifecycle checks, send `q` to the started CLI,
confirm its owned process tree exits, and relaunch against a fresh port set.
