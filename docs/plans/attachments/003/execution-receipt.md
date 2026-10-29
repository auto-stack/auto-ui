# PLAN-003 execution receipt

Date: 2026-10-29, Plan revision r2. AutoUI worktree `D:/autostack/.wt/ui-003/auto-ui`, branch
`codex/plan-003-dev`, execution baseline `a226962eae3caf319ebe3e1a2b91ed380eab40cb`;
AutoLang sibling `09bb414875d4e15050ecbbaf3959b1d72976d857`.

## Work and acceptance results

- T-01: traced `auto run -r vm` through `auto-ui-tool`, `run_vm_ui_flow`,
  native VM child, MCP tree/action/screenshot tools, and API startup. Corrected
  the run command in the Plan because `--server vm` selects a backend and
  `-r vm` selects the native frontend. No AutoUI-owned runtime defect was
  reproduced.
- T-02/T-04: added the durable native-control fixture and replayed it from its
  tracked path. Actual visible HWNDs, current-tree discovery, Chinese
  multiline input, button action, Rust API response, close/reopen, process-tree
  exit, and listener release all passed. Evidence:
  `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-acceptance-fixture.log`
  SHA-256 `7569161DF6CFD0232923B52ECAE6F173303A5A9BA006A279D54E8D66FCE01427`.
- T-02/T-04 U-03: unused MCP endpoint returned connection refusal in 2.052s;
  paired live control passed. Evidence:
  `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-negative.log`
  SHA-256 `FE24A7EECB00DDF11AB5FD54CF38EA4FA5916C12FB0326163B2A678C6422C88C`.
- T-04 Blog: isolated source commit `2a7c9586924a2d42e5d0a9360bcbe265ba90dcb0`
  failed before native UI initialization because its generated Rust backend
  did not compile (37 errors, 31 warnings); API readiness timed out at 120s,
  and the CLI exited 1. This is recorded as a Blog-004 consumer blocker, not
  as an AutoUI pass. Run summary SHA-256
  `0DE7F9625BCA4833D450C370621A4E5A2604703901EFD16B01314D46431BD70D`;
  compiler log SHA-256
  `0CF38E0CB057ADEAD4A88457994E5438C699C3DB71FF7E476DBF20115C0C18D1`.
- Screenshots are native tool captures copied from the tool-returned paths:
  `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-final-initial.png`,
  `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-final-edited.png`,
  and `D:/autostack/auto-ui/docs/plans/attachments/003/work/native-control-final-reopened.png`;
  initial and reopened each SHA-256
  `C833FDB2E791167A8972B17368C732AC97147A22E03C578892B255459F51FEDA`;
  edited view SHA-256
  `0967B47D0CDB26A681DC5D2D9035F75D1CE779AF6FC9AD533301F6E91E7B626F`.
  The live snapshot and API response, not the screenshot, prove both input
  lines reached the backend.
- T-03: no UI source patch was warranted; the control and lifecycle succeeded
  on the recorded revisions. BLOG-004 remains responsible for its pre-window
  backend build and persistent drafts.
- SD-01 prepared at `docs/specs/runtime/native-vm-app-validation.md`, indexed
  by `docs/specs/README.md`. Canonical publication and ledger refresh remain
  merge-owned.

The native-control API store is intentionally in-memory. The Blog data root
was isolated but the app never reached a native window, so no claim is made
about durable Blog data. BLOG-004 must close that consumer-specific gate.
