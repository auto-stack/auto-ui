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
`CARGO_TARGET_DIR` to its `target` directory. Export the committed fixture to
a unique directory under the target tree before running it:

```powershell
$fixturePath = 'crates/auto-ui-tools/tests/fixtures/plan003-native-vm-control'
$runRoot = Join-Path $env:CARGO_TARGET_DIR ('plan003-native-vm-control-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runRoot | Out-Null
$archivePath = Join-Path $runRoot 'fixture.tar'
git -C $env:AUTO_UI_SOURCE_ROOT archive --format=tar "--output=$archivePath" HEAD -- $fixturePath
if ($LASTEXITCODE -ne 0) { throw 'Could not export the committed control fixture' }
tar.exe -xf $archivePath -C $runRoot
if ($LASTEXITCODE -ne 0) { throw 'Could not unpack the control fixture' }
Remove-Item -LiteralPath $archivePath
$appRoot = Join-Path $runRoot $fixturePath
Push-Location $appRoot
try {
    $env:AUTOUI_MCP_PORT = '13140'
    $env:AUTO_HTTP_PORT = '13240'
    $env:AUTO_FRONT_PORT = '13340'
    $cli = Join-Path $env:CARGO_TARGET_DIR 'debug/auto-ui003.exe'
    & $cli run -r vm -B 13240 -F 13340
} finally {
    Pop-Location
}
```

The `pac.at` `api: "rust"` declaration selects the generated Rust API while
`-r vm` selects the native VM frontend. Do not add `--server vm`; that option
selects a VM backend and does not request the native VM frontend. Keep generated
`.am/`, `rust-workspace/`, `dist/`, and screenshot files under the exported
target-tree copy; running from the source fixture leaves ignored files that
repository delivery tests also copy.

The existing AutoLang verifier driver is at
`$env:AUTO_LANG_ROOT/.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`. Its
`AutoUiMcpClient` speaks the running server's JSON-RPC `/mcp` endpoint. Find
the textarea and Save button from the live snapshot, send input with
`autoui_type`, press Save with `autoui_action`, then verify the exact value
through `GET /api/todos`. For lifecycle checks, send `q` to the started CLI,
confirm its owned process tree exits, and relaunch against a fresh port set.
