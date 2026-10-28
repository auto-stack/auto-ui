# Current repair handoff — F-837-R1-01

The first work handoff's visible-pixel assertion claim was incorrect: initial color
and background were only collected/reported, without post-move comparisons. Final
R1 needs_fix is authoritative. The actual final-composition oracle now compares
endpoint and origin at seven representative states; missing readback cannot pass.
Original durable review GPU replay and permanent transparent/old-position ghost
controls fail at the actual move-pixel assertion, with report passed=false/nonzero
native and aggregate. Normal positive, parent/release/partial and saved new-process
reopen remain accepted. See hierarchy-repair-r1.md and the main Plan for frozen
repair identity, exact current results, finding closure/reuse and next role.
Independent final re-review still pending; this work-context check is not a review
verdict. Source245/38-test and timing statements below describe the original work
baseline, not newly rerun whole-suite evidence. The corrected pixel oracle and
refreshed source manifest supersede the original V-10 evidence; unaffected R1 graph/
Auto/transaction/resource checks are reused with exact-input fingerprints.

# PLAN-837 r1 work handoff

Authoritative repository D:/autostack/.wt/lang-837/auto-lang; branch plan-837-dev;
base52ec4c7c13b337992278b343ef89d8aa17102ff6, SDK9576b4dad5ceb8b201adedd46f9689f25c2dd5f9.
Main Plan owns current code_commit/evidence identity; manifest plan837-identity.json
binds all authored inputs. Same tree retained for independent review and merge,
implementation pauses after handoff. No agent/new chat or independent review was
created by work. Source targets autographics-dev only. Main checkout code stays clean.

Work-owned T01..08/AC01..08 ready: strict v1 migration/v2 same-dimension forest,
actual profile4 world/effective output, independent complete admission, tree/local
properties/stable parents, keep-world precision rejection, subtree confirmation,
atomic history/dirty, presented-world picking and2D/3D group handles, bounded
gestures/failure recovery, original resources/owners, persistence/new-process reopen.
No open in-contract finding, required work coverage gap or product decision is
deferred to review. Proposed SD01..05 are hierarchy-spec-delta.md, not canonical
publication. The independent reviewer must validate those statements rather than
trust this implementation-context self-check.

Evidence and sensitivity:

| AC / case | Current assertion / source |
|---|---|
| AC01 / V01 | G836 archive/merge/cleanup checked; hierarchy-source.md; prepare680 SDK files, source manifest, actual479-package closure17local/29edges; drift/missing fixture/missing original GLB controls; external source export retains same closure |
| AC02 / V02,V07 | authoring_hierarchy strict codec/128-depth/shared budget/u64MAX/v1 raw bytes and illegal graph controls; valid controls accepted before Auto/GPU; all nodes retained |
| AC03 / V03,V08 | actual_auto_world_output_and_mutation_controls rejects wrong sum/AND/parent/ordinal/profile;3D group test asserts all original player mesh matrices with local rotation/nonuniform scale consume Auto world; native specific composed2D pixels separate world from local |
| AC04 / V04,V05,V06 | keep-world target+descendants, f32 cancellation rejection+accepted control, same-name siblings, one history/noop/stale parent, exact subtree Undo/Redo/order, high-water and capacity/history budgets;2D/3D group/leaf absolute gestures and last endpoint |
| AC05 / V04,V10 | native OS->Iced tree create/select/parent/attach, group move, undo/redo/hide/show/save; wrong-parent and drop-release are real failed versions of same gate; partial modes cannot succeed; original property native and836 interaction native pass |
| AC06 / V07,V08,V10 | inherited Storage unchanged; raw-byte external mutation, v1 import clean/no write, v2 nested new process exact document/world/unselected2D+3D captures/empty history+selection/clean; busy/fault/stale/last endpoint family replays |
| AC07 / V01,V09 |20 hierarchy rounds observe actual accepted/shown/inflight weak VM owners, seven GPU mesh Arc pointers/target identities and uploads. Actual source mutations retained old VM and reupload mesh each fail this same test; exact restoration passes. External runtime/mesh pins distinguish retained versus last-owner release. Original numeric/fixed game source/locks/assets/SDK/AGP remain unchanged |
| AC08 / V11 | README, hierarchy-feasibility.md and this handoff, proposed SD01..05; finite organization is additional to experimental I1, while production I2/I3/general engine work remains open |

Scoped work gate selected38 unique tests (2 completion +34 authoring/feasibility
+2 numeric), zero failures/ignored; strengthened3D consumer assertion then passed
the same focused test. Other code/dependencies/config/input remained unchanged,
so unaffected prior assertions are reused without repeating whole suites.
Final all-targets check and fmt pass. SDK146 inherited warnings remain unchanged;
no newly authored warning/debug print or workaround deferral. Initial compile/test
development failures are not passes: missing trait method, probe helper/ID API
fixups, stale old widget coordinates, unrendered pre-cancel capture in a test and
an incorrect expected conflict-error spelling were corrected without relaxing ACs.
The old native Apply locator also needed the actual layout witness; initial
phase3 timeout is retained, corrected phase9 pass is the current receipt.

Native hierarchy full positive phase16/204frames; wrong-parent exits1 phase5,
drop-release exits1 phase9 timeout, frame1 exits1 incomplete. New-process saved
reopen matches actual unselected captures as well as world and content. Captured
specific world pixel and vacated local pixel match independent initial color/
background controls. Native fixture only posts to its own PID/window, uses actual
widgets/router and never dispatches user operations directly. Environmental input
may cause a real failure; no cancellation branch is suppressed.

Representative positive measurement: CPU cold frame46021us, warm P50/P95
16543/17013us, compile25466us, working set366346240B, scale2.0. These are one
observed run; latest main receipts carry exact final run values. No hard latency
claim: compile is synchronous, separate from8ms bounded pump rounds. Selected
resource payload:4 targets7584000B;7 meshes136952B/14 initial vertex-index writes;
3D pipeline32992B, depth textures18057216B,2D buffers196672B. Per-frame buffer
writes are measured separately, never called zero. Exclusions: Iced internal
atlas/buffers, driver residency/allocations and GPU timestamps.

## Exact next-stage gates (pending, do not run as work whole tiers)

After independent final review passes, merge freezes the same commit. From driver:

```powershell
$env:RUST_MIN_STACK="16777216"
cargo test --locked -- --test-threads=1
cargo build --locked --bins
cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/verify_source.py
python tools/verify_plan837.py --controls
python tools/check_hierarchy.py research/merge-hierarchy
python tools/check_hierarchy_pixel_faults.py research/merge-hierarchy-pixel-faults
./target/debug/autographics-authoring.exe --window-probe --report research/merge-legacy.json
./target/debug/autographics-authoring.exe --window-probe --sabotage-target --report research/merge-legacy-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --report research/merge-interaction.json
./target/debug/autographics-authoring.exe --interaction-check --sabotage-target --report research/merge-interaction-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --drop-release --report research/merge-interaction-drop.json
./target/debug/autographics-auto-driver.exe --window-probe --report research/merge-game1
./target/debug/autographics-auto-driver.exe --window-probe --sabotage-focus --report research/merge-game1-wrong
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --report research/merge-game2
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --sabotage-focus --report research/merge-game2-wrong
python tools/package_consumer.py D:/path/to/fresh-frozen-consumer
```

The sabotage/drop commands must fail at the appropriate unchanged oracle. Reopen
saved files in a new process using --inspect; never accept counts or a stale report
alone. New consumer runs verify_source, all-targets check, scoped Host/hierarchy and
explicit32MiB native build plus the same check_hierarchy.py and GPU fault controls outside producer;
no producer target/rlib. Worker stack16MiB and native PE32MiB are distinct.

From sibling AGP (read-only inputs, independent workspace), complete locked test
and build, then all16 IDs printed by graphics-probe --list:

```powershell
cargo test --locked -- --test-threads=1
cargo build --locked --bins
./target/debug/graphics-probe.exe --list
./target/debug/graphics-probe.exe --case <ID> --replay canonical --report <new-dir> --mode auto
```

IDs: gpu-foundation, shared-runtime, ag2d-breakout, ag3d-risk-slice, ag3d-squash,
lifecycle, metrics, verifier-negative-controls, iced-shared-device,
independent-consumer, surface-offscreen, surface-compose, surface-input,
surface-scenarios, surface-lifecycle, surface-ime-os. Use --mode window for
surface-offscreen/input/ime-os, with unlocked desktop. Pin AGP_BASE_COMMIT to the
actual frozen code commit for independent-consumer. Counter AND composed-content
gates are both required: run iced-shared-device positive, then separate processes
with AGP_SABOTAGE_ICED=blank_blit and transparent_blit; both must fail. Clear that
environment override before the positive/control or other case. The transparent
fault must retain positive draw counters while failing actual composed content.
Use prior836 merge-r1 raw commands as the exact inherited gate matrix, refreshed
paths/identity; do not reuse its result for changed authoring/profile4.

No root production Rust/Schema/UI generator touched: root t/tv/tt/tb/tu/taa and
docs_gen are outside this experimental Plan's work gates. tf is main-checkout,
single-instance, batch-only when due. Actual landing autographics-dev, canonical
SD deposit/module/goals/ledger, archive, batch due receipt and wt-guard cleanup are
T-10 after review/full gates; none are claimed here. Retain this authoritative tree.
