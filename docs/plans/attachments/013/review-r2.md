# PLAN-013 independent affected re-review R2 — pass

## Binding

- Reviewed commit: `52f370a4af1a10d1af7ac4dff1e53aba105050d1`; tree object `c80aa130c2a4c0208c898a86bb2f4d31538bf44a`.
- Original base: `df39593b53412fed10055d18c0aab90b0590bd27`; R1 comparison: `cd72dc07a5a7ffb141dd5b07e2d0f87865c8cb94`.
- Worktree `D:/autostack/.wt/ui-013/auto-ui`, branch `codex/plan-013-dev`; final HEAD matched the frozen commit and `git status --porcelain=v1` was empty.
- Reviewer: independent read-only subagent `agent-0`, the same identity as R1.
- Scope: F-013-R1-01/02 and affected consumer/reporting/source/evidence paths. No implementation, test, golden, Spec, Plan or Git writes; no other worktree, commit or merge.

The repair diff is limited to the two findings and their required code, tests, golden, validator, identity and documents. SDK, both lockfiles, font, samples, API, budgets and existing gameplay/common-accept paths are unchanged. The 0.02 Jaccard threshold and 8ms/2s semantics are unchanged.

## Findings

No unresolved finding.

### F-013-R1-01 — closed

- `every_state_has_distinct_gpu_glyph_identity_and_wrong_stale_missing_controls` independently checks real GPU masks for slot0/1/3 in states 0..3 in both dimensions. All 24 current-state Jaccard distances were 0, satisfying <=0.02.
- Dynamic counters are explicitly compared with the preceding golden state and require stale distance >0.02.
- Missing glyphs are produced by truncating real quads; wrong glyphs by modifying real UVs. Both are re-rendered and rejected. Restoration replaces the original quads, invalidates content and requires exact equality with the accepted oracle.
- Consumer produces three-slot masks for states 0..2 in both dimensions. It derives colors from the accepted document and rejects empty masks; coordinates are nonempty and unique.
- `check_public_hud.py::mask_distance` rejects empty, wrong-type, malformed/negative and duplicate coordinates. Valid wrong or stale masks are rejected by distance; neither empty masks nor background pixels can produce a false pass.

### F-013-R1-02 — closed

- All four slots render visible `WWWW` with offset `[-4,-4]`, so glyph geometry crosses each accepted clip boundary.
- Both dimensions compare every exterior region with a no-HUD base and require zero pixel difference.
- The unclipped control renders the same layout without scissor. It must change the frame and genuinely leak into the specified exterior region; drawing nothing or making scissor-off identical to clipped cannot pass.
- Every exterior check is repeated at 2x physical DPI.
- `prepare_overlay` remains the production default. The test entry applies one clip to both layout and `PublicHudOverlay.clip`, and the production render path converts it to physical scissor.

## Independent commands and results

Affected tests, serially in `experimental/autographics-auto-driver`:

```bash
UI013_REPORT_DIR=/d/autostack/plan-artifacts/auto-ui/plan-013/r2-affected-evidence \
cargo test --manifest-path Cargo.toml --locked --offline \
  --test public_hud --test public_hud_consumer \
  -- --test-threads=1 --nocapture
```

Exit 0: `public_hud` 17 passed/0 failed/0 filtered; `public_hud_consumer` 2 passed/0 failed/0 filtered. No new autographics/agp warning; inherited SDK warnings remain outside scope.

An inline Python probe imported `tools/check_public_hud.py`, ran `data(r2-affected-evidence, frozen_manifest_dir)` and exercised empty, wrong-type, duplicate, valid-wrong, 2D-stale, 3D-stale and restored masks. Current data passed. Empty/type/duplicate controls were rejected. Wrong distance was 0.666666..., 2D stale 0.732851..., 3D stale 0.735714..., all rejected; restored distance was 0 and passed.

`python tools/verify_plan013.py --controls`: exit 0; 1577 source hashes and edit/delete/unlisted/stable-scene/wrong-pin/escape/target/PID/exact-source controls passed. The golden is in REQUIRED, complete identity and edit/delete controls. SD-01 now records 17 public_hud cases, matching the runner.

AGP `cargo fmt --all -- --check`, rustfmt checks for all changed driver Rust files, `git diff --check cd72dc0..52f370a` and final cleanliness checks passed. No root/full-AG, complete frozen gate, unrelated performance, 20-cycle or legacy suite was run.

## Evidence

| artifact | absolute path | SHA256 |
|---|---|---|
| affected test log | `D:/autostack/plan-artifacts/auto-ui/plan-013/r2-affected.log` | `3814d7a071967390ff62c8dda78e13e92a9f01cb3f6c706dce8bb95a3a8aa46b` |
| data/mask oracle log | `D:/autostack/plan-artifacts/auto-ui/plan-013/r2-data-oracle.log` | `47c84052a4681c99ff0e54586838a133cd2ef7f0ff46a85e773409c277334a34` |
| source controls log | `D:/autostack/plan-artifacts/auto-ui/plan-013/r2-source-controls.log` | `1a066a21505e35a9438c1f8b1ce9573e35094f6533055b637dfe384d586c7999` |
| consumer data | `D:/autostack/plan-artifacts/auto-ui/plan-013/r2-affected-evidence/consumer.json` | `a66bbc95bfaecc795487bd34e629290cc6ceeb3c8b38fd34285c23c793aeaaf8` |
| performance data | `D:/autostack/plan-artifacts/auto-ui/plan-013/r2-affected-evidence/performance.json` | `ec781ba3052d77c45f98f911b81f7e488c0c777e88d087d2942e76f7ab6b166a` |
| GPU golden | `D:/autostack/.wt/ui-013/auto-ui/experimental/autographics-auto-driver/research/public-hud-gpu-golden.json` | `1418b028033070bfa42d258f46d4b3268b96eabbac8cd752f6dc02102477e2b4` |
| source identity | `D:/autostack/.wt/ui-013/auto-ui/experimental/autographics-auto-driver/research/plan013-identity.json` | `93cd9137adb345cc856a66f95f0af5201a670bcb2d86727e34c823de212af249` |

## AC / SD results

| item | R2 verdict |
|---|---|
| AC-01/02 | Retain R1 pass; relevant inputs unchanged. |
| AC-03/V-02/V-03 | Pass; per-slot state glyphs and four-corner viewport/clip gaps closed. |
| AC-04 | Pass; clip gap closed and R1 fault/lifecycle/owner/warm results retained. |
| AC-05 | Retain R1 pass; new consumer mask/data oracle passed. |
| AC-06 | Pass; both R1 findings closed, no new finding. |
| SD-01 | Pass as proposal; evidence gap closed and case count corrected. |
| SD-02/03 | Retain R1 pass as proposals. |

## Next

R2 acceptance verdict is **pass**. Merge still owns the complete frozen directed gate or a strict valid-reuse audit, landing on this repository's master, canonical SD-01/02/03 publication, archive/actual-delivery receipt and guarded worktree/branch/empty-group cleanup. The full AG milestone remains pending under its existing tier rules.
