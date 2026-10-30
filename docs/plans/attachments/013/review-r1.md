# PLAN-013 independent final review R1 — needs_fix

Reviewed commit: `cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94`, tree
`D:/autostack/.wt/ui-013/auto-ui`, branch `codex/plan-013-dev`, clean.
Base `df39593b53412fed10055d18c0aab90b0590bd27`; SDK/two locks unchanged.
Reviewer: independent read-only subagent `agent-0`; no code/tree/Plan writes.

Source identity1576 files SHA256
`606f52b6f6b91b4fcb39cd655c0c591ab7520ef2166d5c67032faca0ffa44f64`.
Gate manifest SHA256
`e9e6edda6ae8ee12d3b85588c3351e4be398eaa586288d1cd8f66f39fdd8ec2c`;
fresh manifest SHA256
`a01203dd913411c0df00b1496b79005a2ae2f150c56165dab5ea4103f119076e`.
Font and OFL identities match the contract. Existing51-case gate and fresh
consumer are bound to this implementation; final 8ms misses8036/8028us are
honestly reported and are not findings.

## F-013-R1-01 — state-text GPU identity and negative glyph controls missing

Severity: major, AC-03/V-02/T-04 and AC-06 blocked.

`public_hud.rs` asserts Host strings and only requires a broad top64px region to
change. Consumer records `hud_dynamic:true` without a GPU capture oracle; the
validator has no wrong/stale/missing text control. If slots1/3 were omitted or
corrupted, slot0 would still make the broad ROI change. The evidence therefore
does not prove actual0/1/2/3 orPlaying/Won/Failed glyphs in both dimensions.

Required closure: per-slot/state GPU pixel or equivalent independent glyph
proofs for both dimensions, with wrong, stale and missing text mutations red
and legitimate restoration green. Regenerate the affected directed evidence.

## F-013-R1-02 — four-anchor clipping and boundary-sensitive controls missing

Severity: major, AC-03/04, V-03, T-05 and AC-06 blocked.

Slot2 bottom-left is hidden during lifecycle tests and only made visible for
atlas counting. All visible fixtures use offsets8 inside the viewport, so the
existing zero letterbox-bar difference does not distinguish active scissor from
no scissor. Resize is uniform2x/restoration, not a four-corner boundary test.
Removing `set_scissor_rect` would not be caught by those inputs.

Required closure: visible text at all four anchors with geometry crossing each
clip edge, no bar/editor-panel leakage, and scissor-off/wrong-transform negative
control. Cover applicable2D/3D and DPI/resize paths, then replay affected tests.

## Independent checks and unaffected results

- `verify_plan013.py --controls`: exit0;1576 hashes and mutation controls pass.
- `check_public_hud.py --validate <frozen manifest> --controls`: exit0; bound
  runners/fresh/legacy/current controls pass.
- AGP fmt check and changed-driver rustfmt check pass. Driver cargo-fmt-all
  fails only because nested frozen SDK workspaces are discovered; direct changed
  rustfmt passes.
- `git diff --check` reports only the exact OFL's preserved trailing whitespace,
  not a new code formatting defect.
- No stray debug output or new delivery warning; inherited SDK146 warnings stay.
- AC-01/02/05 and AC-04 apart from clipping pass. SD-02/03 proposals pass;
  SD-01 evidence inherits the two findings. No other finding identified.

Next: work bounded repair on the same tree, then same independent reviewer
re-reviews affected AC/SD on a new frozen commit. AC-07/08, canonical Spec/
ledger/archive/cleanup and full AG milestone remain pending.
