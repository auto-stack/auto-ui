# PLAN-851 T-02 bounded-motion feasibility

Historical r1 outcome: needs_replan. The r1 A/B evidence below is preserved; current r4 closure follows at the end.

Baseline: e4e5a1d8c3224e60c3d7544a7283bd6706bd6819; authoritative tree D:/autostack/.wt/lang-851/auto-lang, plan-851-dev. SDK and lockfiles unchanged.

Actual path: SourceSnapshot MotionV5 -> trusted compact Auto API -> Session main/config/start -> _ag_tick -> ag_move_and_slide -> two private scans -> actual local + parent candidate -> admission -> existing dirty derivation. No Rust collision solver or SDK edits.

Candidate A added virtual-local two-contact motion, clearance projection, snapshot getters, 24-credit admission, and final strict-overlap admission. Its separate initial full geometry scan exhausts 100000 steps in the 128-node 2D two-contact case. Raw: motion-t02/candidate-a.log.txt.

Candidate B fuses initial strict-overlap detection into the first private scan and unrolls/caches final admission accesses. Same durable test passes 2D (99 yielded slices; this is not an exact step count) but 3D exhausts the original 100000-step callback budget. Raw: motion-t02/candidate-b.log.txt. No budget/config/source-size change was used.

Permanent entry: cargo test --locked --test motion_scan bounded_128_two_contacts -- --nocapture --test-threads=1. It intentionally remains a failing acceptance case, not ignored or inverted. 128 total nodes; actor offset [-1,1], 126 long X walls and a Y wall. D=(10,10,0), clearance=.25. Distinguishing expected actual world=(2.75,4.75,0), ContactLimit, contacts=2, 24+8 credits. Test uses actual callback and direct world fields. This relatively simple wide-tree failure precedes the still-required deep-tree/multiple-parent/late-miss/decreasing-nearest/Host campaign, so it cannot establish 128-node support.

Accepted controls: hand_computed_wall_slide asserts actual world=(2.75,4) (simple stop would yield y=1.1); directional_boundary_and_negative_scan covers 12 actual 2D/3D cases; private_scan_has_no_token_side_effect checks no internal token/credit issuance. Final scoped result: 3 passed, 1 failed, 0 ignored. Raw: motion-t02/scoped.log.txt.

Additional open correctness boundaries: all matching target geometry before early rejection; native finite/shape/error diagnostics; per-axis partial quantization and terminal PrecisionStop; exact fixed-state count; Host lifecycle/publication and legacy compatibility; projects/CLI/native/GPU/owners. Existing controls do not close these. At that r1 baseline Motion API was prototype only; T-02..08 stayed open.

Historical r1 section 5.1 permitted at most A/B and routed failure of both to needs_replan. Required next action: bounded revision of T-02 algorithm/execution-cost design preserving 128 nodes, two contacts, current SDK and original budgets, or explicit user decision if those contracts must change. Do not proceed to downstream delivery or acceptance review on this baseline. No full-tier tests run; review and merge gates remain pending.

Validation: cargo check --locked --all-targets passed (146 pre-existing SDK warnings; no new driver warning); cargo fmt -p autographics-auto-driver -- --check and git diff --check passed. Check raw: motion-t02/check.log.txt.

## Current r4 feasibility closure

The user authorized bounded plan revisions and continued work on2026-10-26. r2 cached mover offsets, reused first-pass geometry validation and unrolled final admission. Original128 two-contact input passed at78/98slices but the expanded12-case matrix exposed7 remaining CPU failures. r3 delayed second-pass later-axis expressions after an established separating axis; 2D6/6 passed but3D6/6 remained red. Standalone first-pass diagnostics44/54/61slices established that the first pass itself fit; these separate calls were diagnostic only, never presented as a whole callback pass.

R4 preserves one original graph and synchronizes actual leaf local+parent world on publication. A fixed marker lets final derive consume coherent world rows. A plain leaf setter on a clean axis can synchronize that row too; dirty/group/visibility paths still trigger original propagation. This removes repeated full-graph re-derivation of clean Y/Z axes when only one leaf changed. It does not alter Parser/SDK/VM counting or limits. All original constraint values and source locks remain.

Current actual whole-callback12-case matrix:2D/3D x diagonal/late-axis miss/decreasing-nearest x positive/negative unequal axes, all128nodes, group writes around motion,24+8credits:12/12 pass, maximum98yield slices (not exact steps). Original hand-computed wall slide now(2.75,4), with a real stop-only mutation producing(2.75,1.1) and restored control. Actual GPU Host covers128 wide/deep, parent switches and two contacts, with group writes before/after and direct final object world assertions. Geometry sources/tests stay in the source manifest.

Additional permanent cases cover whole-box initial boundary policy versus outside tangent arrival (pre-fix red and post-fix green retained), all six signed axes, XYZ/ordinal ties, three-plane second-contact cap, strict initial overlap, zero D, clearance-imposed clipping, finite quantization/shape collapse, native NaN/Inf local/argument/return/container, wrong token/kind/axis/exhaustion/callback lifetime, snapshots after setters, second-motion full-frame rejection, visibility filters and64object-stable callbacks. Play checks preserve last-good, close VM and Stop recovery. Runtime Slot measurement:29logical scalars/116bytes; actual SDK/key/map sizes separately reported. Current managed size is measured by the final storage check and remains below64KiB.

Raw current checks and native reports: research/motion-evidence/. Full acceptance details, case mapping, frozen identity and merge-owned pending gates: bounded-motion-handoff.md. Canonical/ledger edits and independent final review are not work-stage passes.
