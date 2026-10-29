# PLAN-005 r3 — T-03 closure

Implementation: `e1586602e2d48e01b3d5c80047c9d75ba74db4f6` in
`D:/autostack/.wt/ui-005/auto-ui`, `codex/plan-005-dev`.
This closes T-03 / V-01..02, not GPU sampling, lifetime, consumer or final review.

## Current producer/consumer evidence

| Invariant | Executed distinguishing observation |
|---|---|
| Raw roots + independent PNGs share 8 / 32MiB | A root with seven independent images accepts; its exact duplicate and a shared PNG sprite reuse actual Arc owners at full quota without another decode; a ninth image refuses. Three 8MiB PNGs alongside the accepted model fit; a fourth rejects before decode; exact large duplicate reuses its owner. |
| All images count, including unused declarations | Four 2048² images plus geometry reject before any mesh/PNG decode; three fit at >48MiB and <64MiB CPU/GPU. Adding the fourth through the legacy sprite importer refuses before decode. CPU image Arcs are shared; a PNG used in both 2D and 3D reserves both physical GPU-store copies. |
| Actual immutable identity and reuse | Two models sharing a PNG retain the same raw and decoded image Arcs. One root with different external PNGs retains distinct closure IDs and both environments survive one saved project. Roots remain exact original bytes/SHA; raw accounting deduplicates the root. |
| No unchecked dependency IO | URI/device/path/extension/reference/typed accessor/finite tangent/UV/UV-transform overflow/image bomb cases reject before decode. A test-only junction outside the worktree refuses at the PNG dependency path without touching its target. Embedded PNG imports and saves as one GLB blob; an unused missing external image still rejects. |
| Persisted binding and cache do not hide corruption | SaveAs uses accepted bytes after the import directory is changed. Missing/corrupted PNG refuses even with the original model cached; restoring exact bytes reopens through the same entry. A forged/rehashed wrong URI edge fails root binding. |
| Entry last, no false saved confirmation | AssetAfterPrepare/AssetAfterInstall/BeforeReplace leave no entry and retry succeeds. AfterReplaceRead/ClosureAfterReplace remain VerificationPending with no confirmed path; removing a PNG prevents Verify, restoration then Verify succeeds. Foreign SHA-named blob is never overwritten. |
| Legacy schemas reject new fields | Schema1..5 each has a valid empty-scene control; adding a model closure or `closure:null` gives `legacy unknown model closure field` before migration. Old scalar GLB/PNG formats remain readable. |

## Commands, current results and reuse

All commands use the plan worktree, `--locked --offline`, single test thread.
Evidence stays local in `D:/autostack/plan-artifacts/auto-ui/plan-005/`.

- `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test textured_model_resources -- --test-threads=1 --nocapture`: 11 passed on the final resource/storage implementation (`t03-delivery-candidate.log`, SHA256 `f64ee169d873904c3b2a539caac7a0202a72aa2982c823ec9c2406c609830660`). The subsequent legacy-field guard changes only rejection of older-schema closure fields; unchanged positive/schema6/resource/storage paths reuse this evidence.
- Added paired control replay `... --test textured_model_resources legacy_schema_controls`: 1 passed, `t03-legacy-schema-controls.log`, SHA256 `4ad2d7db695c555d7c3eee01895bc27e78ab4e48a87f22e76c386ca52306ef1f`. Current suite has 12 tests; this is 11+1 executed evidence, not a claim that one invocation selected twelve.
- Legacy model affected sample: `... --test authoring_assets budgets` selected 2/17 deterministically (no RNG). `exact_raw_json_active_union_and_predicted_gpu_budgets_before_decode` passed; the geometry case found a changed error prefix, now restored. Its targeted replay `... geometry_limits_sparse_bad_indices_and_raw_snapshot_budgets` passed; the only intervening legacy-model change was message text, so the first passing oracle is reused. Logs: `t03-legacy-budget.log` SHA256 `c45b7cbccb5ac3a1434f6126efda793bd27037552033cb1ce5a53ae7103c97de`; replay `t03-legacy-repair.log` SHA256 `a443c70046a6c4f238c74cf358321b730ab9e577dc223e52ba3baf94925801c6`.
- Necessary PNG call-path replay: `... --test public_sprite_frames png_closure`: 2 passed after restoring declared SHA validation before decoder dispatch. `t03-sprite-closure-repair.log`, SHA256 `5b0c12aaca8fa413237987d8ac3ccbdc4e7ea76a6a3d65990113e8830598c0cb`. Its initial failure was a wrong diagnostic stage on corrupted bytes, not missing corruption rejection. This repair is separate from the legacy-model prefix finding.
- Driver format check and `git diff --check`: exit0. No new product warnings; frozen SDK warnings remain inherited. No root-crate/all-AG/OS-input suite was run.

T-04 still owns independent pixel sampling / texture transforms / sampler modes /
depth / double-sided lighting and shadow oracles. T-05 owns complete prepared-frame
failure, actual GPU/last-owner and twenty-round Play checks; T-06 owns source
identity, offline consumer and public delivery instructions. These remain open.
