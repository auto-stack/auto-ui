# UI-005 cycle2 — shared content binding and usable cached projects

Plan r3, implementation `D:/autostack/.wt/ui-005/auto-ui`, branch
`codex/plan-005-dev`, frozen repair `74378791b353cd285168fa43fb7fe4b27cd208fe`.
R2 baseline `8809e545830a2d0ed6f587517e6d4d8a29a575ca`.
Required two-failure diagnosis was written before implementation in
[repair-r2-diagnosis.md](repair-r2-diagnosis.md), with a corresponding Plan
checkpoint before the repair commit. No semantic revision/support change.

## F-005-R1-02 complete invariant adjustment

One `AssetRef::same_content` now compares the complete binding: SHA256, byte
length, format, width, height and full ordered closure, excluding only name.
Full AssetRef Eq still includes metadata. Callers unified: PNG declared load,
textured declared load, Editor/session validation, snapshot save admission,
generic Play and sprites target admission. Whole-source comparison inventory
has no remaining full AssetRef equality at those admission boundaries.
Source/code identity, document hash and snapshot byte equality remain separate
and unchanged. No cached immutable name is rewritten to match a document.

All root/dependency readback, transaction entry-last/Verify, no-overwrite,
reservations, union budgets and typed geometry admission are retained. The
already-closed material classification fix is unchanged.

## Actual lifecycle and permanent coverage

Expanded `cached_closure_open_ignores_display_name_but_preserves_content_binding`:
different-name cached Host Open→Play startup/settle (assert not Faulted)→Stop→
Save current→SaveAs with ClosureAfterReplace pending→corrupt image Verify
refuses→exact restore Verify confirms→cold/cached reopen, another Play/Stop.
Document alias name, original shared Arc and exact GLB bytes are asserted;
the original cache-hit corrupted-dependency/restore checks remain.

New `legacy_model_and_sprite_cached_names_survive_save_reopen_and_actual_play`:
actual legacy two-parts.glb and PNG import through their declared public
entrances, independently imported aliases, exact metadata Eq unequal but
content equal. Saved alias projects warm-open over original cached Arcs.
Legacy generic Play and PNG public Sprites2V1 both initialize without Fault,
then Stop, Save/current, SaveAs and reopen retain alias name/cache Arc. PNG
reaches both generic and sprites-target binding comparisons. A changed declared
byte length is still refused by both actual snapshot save and Play admission.
PNG script binds the required0.8×1 character and follow camera per the existing
sprites contract; it does not use a no-op script that faults at initialization.

`cargo test --locked --offline --test textured_model_resources cached_ --
--test-threads=1 --nocapture`: exit0,3 passed,18 intentionally filtered;
`repair-r2-cached-final4.log`. This includes the independent existing
missing/corrupt dependency, forged rehashed edge, exact restore and legacy-schema
controls. The existing same-root/different-image coexistence test remains in
the current full resource suite.

Affected legacy compatibility: public_sprite_frames filter `real_auto_` exits0,
2 passed (actual composite API and actual frame/flip pixel behavior), log
`repair-r2-sprite-compat.log`; filter `older_profiles_keep` exits0,1 passed,
log `repair-r2-old-profiles.log`, retaining byte-exact old profile assembly.

## Independent counterexample replay

All raw paths below are rooted at
`D:/autostack/plan-artifacts/auto-ui/plan-005/`.
R2 `name_cache_probe.exe --expect-lifecycle` was first replayed on the baseline,
exit101 (`repair-r2-lifecycle-before.log`). Copies of R2 replay sources in
`repair-r2-replays/` change only output paths, preserving original reds. Rebuilt
against repaired current library; original21 material/admission holdouts with
5 accepted controls pass, plus extended cache lifecycle exits0. Every actual
line in `lifecycle-green.log` is Ok, including warm SaveAs/current, generic
Play, Host Play and Host Save; script additionally rejects any `Err(` output.
Original R1/R2 input manifests and executable sources remain reusable.

Probe setup errors retained: initial new sibling test sent PNG through the GLB
import API, then used a sprites script without required character/camera and
correct character size. Correcting these to the existing API/contract yielded
nonfaulted real startup. No product rule, fault assertion or support scope was
changed to accommodate the probes.

## Frozen source and evidence reuse

Eight changed driver files: shared binding helper and five callers, permanent
tests, exact checker count20→21, regenerated identity. Current identity1456
files SHA256 `134b4f731889a3980557037e3ea3bdfddb81196f858c3d98c8c04a10c26a0fbc`.
Source mutation controls/fmt/diff checks pass. SDK681/two locks, AGP renderer,
UV/material parser, raw/decoded budget implementation and SD draft are unchanged.
R1/R2 unaffected GPU sampling/owner/transaction, fixed-source licensing and
earlier actual consumer A→B relocation evidence remain applicable; this does
not waive the changed save/Play/name states, which are verified above.
Merge-owned new frozen producer gate and independent affected R3 remain pending.

## Current independent consumer and readiness

Fresh export `consumer-repair-r2` starts without prepared SDK or targets.
Snapshot-only cold preparation passes681files/17packages. Current source
verifier checks1456 hashes, SDK479packages/29local edges, forbidden0.
`check_textured_models.py --reports <repair-r2-consumer-gate> --controls`
exits0: **21 driver +4 AGP**, no failed/ignored/filtered cases,12 semantic
reports and all pixel/partial/runner/budget/warm/stale/actual-zero controls.
Manifest SHA256
`01eaf90e08370b6ccfc81f6fb639a5cd77b27022686f86a4bb83b516495e2c25`;
gate log SHA256
`bbf3bb0cd5163e0657d808024edf3888e32840b69b5482153b5e60359ad74464`.
Executable hashes, PE32MiB/worker16MiB, current source identity and log hashes
bind the run. Earlier consumer snapshots remain unchanged historical evidence.

Original independent21-input admission replay green log SHA256
`b7ee07e72a8d35f93877bc00d0f980eec0d5f135e1ce2a391d52424472070176`;
R2 full lifecycle replay green SHA256
`817525453c1e2d88a077ff282056d40bb2d980ab375870ac09186539bfd60a2e`.
Final producer fmt and source controls pass again (`repair-r2-final-source.log`),
HEAD7437879 clean. No known work-owned finding or coverage gap remains under
the expanded diagnosis oracle. This is work readiness, not independent AC-06
pass. R3 must independently verify the shared rule, all applicable callers and
the real post-Open operations; merge/Spec/ledger/archive/cleanup stay pending.
