# PLAN-837 r1 — F-837-R1-01 work closure

Entry: final independent R1 needs_fix, reviewed code
abe682c5a1144f7e9b72f4ffeb95ece19dd02841; same authoritative tree
D:/autostack/.wt/lang-837/auto-lang, plan-837-dev. Main Plan §9 records the repair
commit. SDK9576b4d, dependency locks, AGP, games, authoring library and assets are
unchanged. No new product decision or scope reduction. This is work-side closure;
independent affected re-review and all merge-owned complete gates remain pending.

## Reproduction, cause and affected states

The durable R1 input is main docs/plans/attachments/837/review-r1/
replay-pixel-oracle.py (SHA25660727913a42181040dd0b9041d6799899930386e4a1a7efb38b1f9757dddbd7a).
Its byte-exact copy ran in work-repair-r1/pre-fix, preserving original review
artifacts. It made the moved circle transparent in a real final GPU composition,
asserted the resulting endpoint was background, then restored inspected commands.
The old native/aggregate/reopen still passed phase16/204frames/exit0: reproduced.

Cause: the fixture collected initial leaf/background colors, but at move inspected
only local/world/history/picking/GPU commands and recorded a capture hash. Matching
wrong images across processes was an equality check, not a visible-content oracle.
The earlier work handoff overstated coverage; its visible-pixel claim is invalid.
Normal unmodified product rendering was not reported broken. The same missing
observation affected temporary move, accepted move, undo, redo, show and saved
unselected views. Hidden-state checking existed, but now validates both positions
and cannot accept two missing readbacks as equal.

| Finding | Invariant / affected paths | Fix and permanent entrypoint | Current observed closure / boundary |
|---|---|---|---|
| F-837-R1-01, AC05/06/08, V10, T07/08 | OS/Iced -> real Host.frame/Auto -> final top texture -> native report/exit -> Python aggregate; visible endpoint must match initial leaf color and vacated point background | native Probe::pixels reads final top at origin[345,190], endpoint[375,170], compares actual RGBA to independently read nonempty/distinct initial controls; seven stages temporary/move/undo/redo/hidden/show/saved. Failing values recorded before terminal error. check_hierarchy.py requires all seven current records and keeps actual saved new-process reconstruction | Native positive phase16/204frames,7 passed records, correct parent/release/partial negatives and reopened document/world/captures/clean history/selection. Finite2D fixture-center checks, not full-image/3D pixel equivalence |
| same F-837-R1-01 sensitivity | A downstream render fault must fail even if Auto/state/commands are correct | tools/check_hierarchy_pixel_faults.py builds test-only entry copies, leaves producer/SDK/AGP sources byte-exact; actual transparent endpoint and an extra old-location ghost each rendered through real GPU, checked physically, then original commands restored | Both native faults fail phase9 at move pixel comparison, passed=false/nonzero exit; unchanged aggregate positive expectation also fails. Transparent: endpoint becomes background. Ghost: correct colored endpoint remains, but origin wrongly has leaf color. Both independent sides distinguish the fault |

The byte-exact original R1 replay was also run in work-repair-r1/post-fix-review
against the repaired probe: native phase9/passed=false/exit1 and the original
aggregate fails before reopen equality can mask the error. This confirms the
repair with the review's durable input as well as the two permanent controls.
No production fault hook, source mutation, SDK/AGP edit or renderer workaround.
No artificial acceptance limit, weakened cancellation, tolerance or status check.

## Verification and evidence reuse

From experimental/autographics-auto-driver:

```powershell
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/check_hierarchy.py research/hierarchy-repair-native
python tools/check_hierarchy_pixel_faults.py research/hierarchy-repair-pixel-faults
cargo test --locked --bin autographics-authoring --test authoring_probe_exit -- --nocapture --test-threads=1
cargo check --locked --all-targets
cargo fmt -- --check
python tools/verify_source.py
python tools/verify_plan837.py --controls
python tools/package_consumer.py <fresh-external-source-directory>
```

Three targeted completion/CLI tests passed, zero ignored. All-targets check/fmt
passed. No whole driver/AGP/root tier ran. The unchanged SDK's146 existing warnings
remain, no new driver warning. Source manifest/consumer checks are recorded with
their exact count and hash in the main repair receipt. External source is verified
and checked independently; its complete native/build/compatibility matrix remains
merge-owned after final review.

The final R1's38-test graph/codec/real Auto/transactions/3D matrix/numeric/resource
results are reused only because no authoring library, test input/assertion, SDK,
dependency, assets or verification config changed. Their current byte fingerprints
are in the main repair receipt. New native probe/aggregate pixel evidence and SD05
supersede the invalid original V10 claim. Full original property/836 interaction/
games/composite gates remain merge-owned, not falsely counted as fresh repair tests.

Readiness: T07/08 work-side closure established; no open work finding, required
work oracle gap or product decision remains. Final review count remains1 failed
round (R1); work self-check is not a second final review or a reset. Same tree stays
for affected independent review, implementation stops after the repair commit.
No source landing, canonical Spec/ledger publication, archive or cleanup is claimed.
