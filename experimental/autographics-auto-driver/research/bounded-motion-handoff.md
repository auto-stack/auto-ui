# PLAN-851 r4 work handoff

Next responsible role: independent final review, in this same authoritative checkout. Work ownsT-01..08; T-09 review andT-10 merge remain pending. Review baseline is the commit containing this handoff and plan851-identity.json; its full SHA is recorded in the shared main-checkout Plan section9. Implementation writes stop after that commit. No merge/archive/cleanup or canonical/ledger publication is claimed.

## Identity and scope

- Repository/worktree: D:/autostack/.wt/lang-851/auto-lang; branch plan-851-dev.
- Formal849/base:6bb85f354e718f6541256f4b3d75ca97af75b8c4. User authorized revision and all work on2026-10-26. r1 A/B needs_replan and r2/r3 cost findings are retained in feasibility and raw evidence; final contract is r4, not those superseded strategies.
- SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467; three lockfiles and source-lock remain byte-identical to motion-baseline.json.txt. SDK/auto-val/AGP/root Rust/production trees were not edited. Only driver delivery files changed; main checkout holds shared Plan bookkeeping.
- Current source/asset closure: research/plan851-identity.json, verified by tools/verify_source.py and verify_plan851.py --controls. Original849 closure/denominators retained; no old test/source asset dropped. Old reader's frozen behavior.rs has versions1..4 and explicit unsupported default, so /5 is not silently dropped/fallback.
- Frozen native runner SHA256:13a6d0cb5da88d73fb7d77148337ff5a62201a810b302a9c4d1d1110e25dd375. All current native controls use these exact bytes. External native root:D:/autostack/.verify/lang-851-native-r4-1; raw reports, accepted/changed source inputs and scene closures are copied into research/motion-evidence/native-external.

## Work-owned acceptance and permanent evidence

| AC/tasks | Actual observation and distinguishing oracle | Evidence |
|---|---|---|
| AC-01/T-01 | Formal849 archived/cleaned, SDK681files/17local packages, original three pins/source-lock and original producer/fresh-consumer247/247 denominator retained | motion-baseline.json.txt; source gate raw; plan851-identity.json |
| AC-02/T-02/T-04 | Real Auto(2.75,4) versus stop-only(2.75,1.1); removed-second-scan mutation red and restored green; six signed axes, XY/XYZ and ordinal ties, initial strict overlap/whole-box boundary versus outside tangent, zero D, thin wall and two-contact/three-plane cap | motion_scan; motion_geometry; tangent red/green raw; final-work.log |
| AC-03/T-03 | /5/profile11 actual Session and typed Host path; all v1..v4 headers and old motion helpers retained; forged source/version/hash/regions rejected before runtime; no raw rewrite | motion_api4tests; sweep_api3tests; legacy raw |
| AC-04/T-04/T-05 | One virtual-local action publication and original-world-row sync; clean-axis leaf/group/visibility propagation; full snapshot and token/kind/axis/sequence/callback checks; second motion exceeds32 and rejects first motion's frame, owner closes and last-good retained | motion_geometry; motion_host4tests; 128Host direct actor AND target final-world assertions |
| AC-05/T-02/T-05 | 128nodes:12 signed/equal-axis diagonal/late-miss/decreasing controls plus mixed-sign unequal(2000,1500,-1200) decreasing candidates, all under original100000steps/120resumes/250ms and24+8credits; worst98yield slices, not exact steps; actual Host wide/deep125groups+3leaves/two contacts and multiple parents; geometry/nonfinite/shape/finite precision distinct | motion_feasibility3tests; final feasibility raw; motion_host; native NaN/Inf argument/local/return/container assertions |
| AC-05/T-04 | Actual local=-1e8 plus parent=1e8 gives origin0; safe math advance5.75 quantizes local to publish world8 inside wall[6,8.2]. Final admission returns PrecisionStop4 and restores origin0; removing the actual guard produces Clipped/world8; restored control passes | motion_geometry::actual_parent_rounding_final_admission_and_removed_guard_counterexample; admission-control.log |
| AC-06/T-03/T-06 | Same raw Arc across API transaction; Undo/Redo/save/reopen and failure isolation; explicit v5 UI/CLI and malformed/inspect usage errors; Maze2D/Corridor3D user Auto logic with frozen eccentric multipart GLB, parent/TRS and real XZ walls | motion_project1test; motion_cli4tests; samples/motion_projects; external native6cases |
| AC-07/T-08 | 64callbacks no heap-object growth and VM last owner released; twenty mixed v1..v5 real Run/Fault/Stop/cancel/history/Open cycles preserve source/VM/Descriptor/GPU pins and actual canvas content;6 deliberate retention/rescan/upload/content mutations each fail at intended assertion, restored control passes | motion_geometry64case; motion_owners1test; owner-controls/results.json and raw stdout/stderr |
| AC-07/T-08 |29retained query/dirty scalars=116logical bytes;232NanoValue bytes,696String structure bytes,408key capacity bytes; shared map capacity/40byte structure separately observed. Managed49223bytes<65536; program/stack/allocator/GPU/Iced exclusions explicit | lib::segment_memory_measurement::motion_slot_real_storage_and_owner_boundary; storage-final.log |
| AC-08/T-07 | Six external-root normal/bind/change cases plus4additional new-process reopens; OS v5/Bind/SaveAs/Run/Pause/Step/Resume/Stop, same runner, every actor part's pixels and independent f32 world/node/normal/status replay; source-only speed/clearance changes;8 intended missing/source/API/asset/partial/transparent/part/content rejections | native-external6case results; native-negative8required controls; each report contains actual pixel samples and input/action replay |
| AC-08/T-08 | Spec deltaSD-01..04 and merge matrix below; original Dash2D/Dash3D native replay and19affected old sweep/API/project/CLI/Host tests pass | bounded-motion-spec-delta.md; legacy-native2cases; legacy-scoped.log |

Open in-contract work findings: none. Required work coverage gaps/unresolved product decisions: none. These are executor observations, not an independent review pass. Review must inspect the actual frozen code/oracles and proposed Spec delta; it must not treat checked tasks as evidence.

## Verification executed, and reuse boundaries

33 new unique Rust tests passed: motion_api4, motion_cli4, motion_feasibility3, motion_geometry10, motion_host4, motion_project1, motion_scan5, motion_owners1, lib storage1. These are scoped work checks, not a full driver suite. Final-work.log covers29integration cases; the additional unequal-axis/admission tests and final feasibility/Host assertions have separate passing raw logs. Owner restored run and lib storage add2. Zero-selected earlier filtered API invocation was not counted; the actual4API tests ran in final-work.log.19old affected tests passed: sweep_api3/cli4/geometry7/host4/project1. Full suites were not run.

Commands run from driver folder:
- cargo test --locked --test motion_api --test motion_project --test motion_scan --test motion_feasibility --test motion_geometry --test motion_host --test motion_cli -- --nocapture --test-threads=1 (plus later changed/new scoped filters identified in raw logs).
- cargo test --locked --test motion_owners twenty_real_v1_v2_v3_v4_v5_cancel_fault_history_open_owners_and_warm_geometry -- --exact --nocapture --test-threads=1 via tools/check_motion_owners.py,6expected red mutations + restored green.
- cargo test --locked --lib motion_slot_real_storage -- --nocapture --test-threads=1.
- cargo test --locked --test sweep_api --test sweep_geometry --test sweep_host --test sweep_project --test sweep_cli -- --nocapture --test-threads=1.
- cargo check --locked --all-targets; cargo fmt -p autographics-auto-driver -- --check; git diff --check.
- python tools/check_bounded_motion.py D:/autostack/.verify/lang-851-native-r4-1 --cases maze2d,corridor3d,maze2d-bind,corridor3d-bind,maze2d-short,corridor3d-short; missing/graphics controls recorded in native-negative. python tools/check_swept_aabb.py target/motion-legacy-native --cases dash2d,dash3d.
- SDK prepare verify-only, verify_source, verify_plan851 --controls; source/pin/GLB mutations reject then restore, never modify SDK or cargo lockfiles.

146pinned SDK warnings are unchanged; no new driver warnings. No root cargo aliases/docs_gen, full driver suite or AGP full suite executed during work. Reused unaffected evidence is limited to unchanged code/SDK/locks/test configuration/input families; legacy original source/API files and AGP source/assets remain byte-identical and are closed by manifest. New native reports use their exact runner/source/scene fingerprints, not an older cached binary.

Performance observations: Session compile_us, private first-scan wall time and full-callback wall time are logged separately in feasibility; full Host initialization/configuration and callback/pump/resource/working-set observations are in Host/native/owner raw. These are wall-clock observations under machine load, not exact CPU instructions or GPU timestamps. Logic60Hz dt is not a real-time guarantee. Original R849-NATIVE-CLOCK scheduling risk remains; no threshold is widened.

## Review and merge-owned pending gates (not waived)

T-09 independent context reviews AC-01..08/SD-01..04, omissions/deferrals/workarounds and health on this same committed tree; bind Plan r4 + code/SDK/config/fixtures/raw identities. No final pass is claimed by this implementation session.

After final pass, T-10 freezes the reviewed revision and runs complete locked driver tests/build and required AGP gates, original frozen dual-game/content AND counters and835..849native/owner/source/package/fresh own-SDK own-target consumer closures, plus all added motion checks/external relocation/owners. Baseline849 denominator247producer+247fresh consumer is preserved.33new tests imply280driver tests expected; merge must enumerate the actual selected denominator and report actual execution, not claim280passed from this work. AGP unchanged23semantic slots/163tests/16native and original87graphics semantics remain in the full matrix; new motion native case groups add14distinct groups (6positive,8negative, with4extra reopens) alongside old2Dash native passes and all old originals. No missing/original family is silently dropped.

Reuse after code change requires checking relevant code/dependencies/config/input; a repaired affected path returns to review then new frozen gates. No root t/tv/tt/tb/tu/taa/docs_gen851gate; batch due check belongs to merge/main single-instance under850 orchestration. Only after complete merge gates may code land autographics-dev, canonical/spec-index/ledger update, Plan archive, guard+worktree cleanup happen. Master gets knowledge/bookkeeping only.
