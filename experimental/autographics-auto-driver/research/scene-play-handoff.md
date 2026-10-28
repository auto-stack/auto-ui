# PLAN-839:r1 work handoff

Authoritative implementation/review tree: D:/autostack/.wt/lang-839/auto-lang,
branch plan-839-dev, source base37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c.
Shared Plan/evidence live on D:/autostack/auto-lang. No other writer or agent was
created. SDK/locks/AGP unchanged; exact source closure is plan839-identity.json.
Source destination remains autographics-dev. No implementation lands on master.

## Runtime contract and operation

Run/Pause/Resume/Step/Stop and two actor-ID cycling controls are in the existing
authoring panel. Defaults choose the first eligible same-dimension leaf. A canvas
click focuses play without starting an editor gesture. Arrows move only that actor;
V parity toggles visibility.2D X/Y60 units/s,3D X/Z3 units/s, diagonal axes independent.
Pause/Step use whole-tick boundaries; Step is a neutral actual Auto tick. Esc Stops
once. All editor/resource/file commands are disabled and independently rejected
while play exists. Unapplied property draft/pending edit/import/gesture/dialog or
unverified save receipt blocks Run; dirty and accepted unsaved assets are allowed.

Profile6 is the included Auto behavior sample. One retained Session owns runtime
state. Local coordinates/visibility mutate only there; a typed complete frame is
independently admitted, prepared and published. Stop restores the retained accepted
GPU scene without compile/import/prepare; faults close the runtime VM and keep the
last accepted running frame. See scene-play-feasibility.md and the proposed
scene-play-spec-delta.md for exact queues/layout/limits and ownership. A fixed sample
does not imply arbitrary user script binding/packaging. No runtime result is saved.

Clock: dt=f32(1/60), one inflight, four credits/four dispatches per round, shared8ms
pump target, inherited SDK slices/callback limits,30s initialization/2s full-frame
deadline. Display gap>250ms/backlog>4, blur/leave/resize/DPI/occlusion/minimize/Pause
clear held/parity/debt. Resume uses a new wall-clock origin, requiring fresh nonrepeat
keys after cancellation. Synchronous compile/native GPU calls remain nonpreemptible.
Starting cancellation is retained through initialization under its30s budget and
publishes only the complete initial frame into Paused. It preserves the reason and
does not dispatch until explicit Resume. Starting Resume/Step reject; Stop/fault
release normally. Later runtime/Step transactions still have the2s deadline.
The maximum graph is128 nodes and2820 typed output values. Trace/measurement data
is bounded4096 summaries, not4096 retained frames. Tick positive-i32 exhaustion
closes before wrapping. Existing asset/512 potential-instance limits remain.

Actual Editor content/hash/stamp/selection/saved hash/dirty/undo+redo identity are
captured at Run. Host gates every mutation path, checks the frozen epoch/revision
while pumping, and permanent tests compare complete before/after fingerprints.
The full16MiB history is not reserialized every display round. Runtime owns one
additional VM, candidate and last-good frame, plus Arc pins; the original Editor
preview/assets and directly restorable GPU presentation remain valid owners.

Actual20-cycle tests check runtime VM weak owners, source/mesh Arc lifetime, original
GPU mesh Arc identity, target build_seq IDs and pipeline object identity, vertex/index
write counters and legal per-frame instance/content/composition writes. Final Editor
close releases source payloads; the last explicit external mesh pin releases on drop.
Both actual retained-VM and forced-reupload mutations make the same owner test red.
Run/Stop/pump timings, full transaction/compile timings, native frame P50/P95 and
Windows working set are reported. GPU timestamps, driver residency and Iced internal
buffers/atlas are excluded.60Hz logical dt is not a universal60FPS guarantee.

## Work-owned reproducible checks

From experimental/autographics-auto-driver in this tree, fresh output directories:

```powershell
python tools/prepare_sdk.py --repository D:/autostack/auto-lang
# The SDK command above is only for a fresh tree. Existing trees: --verify-only.
python tools/verify_source.py
python tools/verify_plan839.py --controls
cargo fmt -- --check
cargo check --locked --all-targets
cargo test --locked --test scene_play_feasibility --test authoring_play --test authoring_play_host -- --nocapture --test-threads=1
cargo test --locked --lib authoring::play::deadline_tests -- --nocapture --test-threads=1
cargo test --locked --bin autographics-authoring completion_tests -- --nocapture --test-threads=1
cargo test --locked --test authoring_host --test authoring_session --test authoring_interaction --test scheduler_lifecycle -- --nocapture --test-threads=1
cargo build --locked --bin autographics-authoring
python tools/check_scene_play.py research/play-NEW
python tools/check_play_owners.py research/play-owners-NEW
python tools/package_consumer.py D:/autostack/.wt/lang-839/consumer-NEW
```

Use `cargo fmt`, without --all: --all walks into the pinned SDK's nested vendor
workspace and is not the inherited driver formatting entry. Do not change the SDK
to make that unrelated invocation work. The pinned SDK inherits146 compiler warnings;
work-authored Rust must introduce none. No complete driver/AGP test suite or root
t/tv/tt/tb/tu/taa/docs_gen runs at implementation/review time.

Startup cancellation regressions: `authoring_play_host` includes the four real
Host window edges during Starting, unchanged Editor/canvas, neutral Step, repeat
rejection, explicit Resume with fresh input, Stop before/after initial publication,
wrong-profile and resource-prepare failure plus fresh successful Run. The inline
deadline family distinguishes initialization>2s/<30s, initialization>30s before
output or after preparation, and later Step>2s with actual VM owner release.
F-839-R1-01's original review-only client remains a separate replay; it must pass
with all four edges Paused/tick0 rather than background tick2.

The new native positive must reach27 phases; six independent top-composition pixel
records have six checks each (small2D endpoint/origin, imported red/green endpoints
and originals). It additionally verifies actual Auto Step+1, paused stability,
UI focus isolation, two V edges cancel, single V hides actual pixels, V restores,
OS focus-loss auto-Pause, Stop original canvas hashes, new Run nonce/initial locals,
Save and directory relocation/source removal/fresh-process inspect. Native negatives
wrong-focus/drop-tick/transparent/missing-part/wrong-GPU-model/frame1-incomplete
must exit nonzero, passed=false and reach their named oracle. Root reports/counters
or two equally incorrect hashes do not replace these independent checks.

Native pixel readback is synchronous and can itself create a>4-credit display gap.
The probe records auto-Pause and uses actual Resume/refocus before sending more
keys; hidden readback occurs Paused. This preserves the production overload guard.
Common OS helper now posts actual scan codes; printable V also posts WM_CHAR.
Panel layout witnesses observe stock Scrollable's actual clamped translation via a read-only public operation, including content shrink; the original operation then forwards unchanged. Existing fixed-coordinate
list/text probes were adjusted to locate and scroll real widgets; content/focus
assertions are retained. Development failures and fixes remain in main work evidence.

Fresh consumer: source-only package has its own target and all680 SDK files.
Set RUST_MIN_STACK=16777216 and RUSTFLAGS="-C link-arg=/STACK:33554432" there,
then verify_source/verify_plan839 --controls, all-target check, the two formal play
test targets and native check_scene_play. These are worker-stack and Windows PE
stack requirements of the pinned SDK, not new scene-schema requirements.

## Final review / merge-owned matrix — pending

Work readiness does not mean independent review or completed delivery. Review the
same committed tree with implementation writes paused; compare AC01..08/SD01..04,
including omissions/deferrals/workarounds and health. Then merge owns:

```powershell
$env:RUST_MIN_STACK="16777216"
cargo test --locked -- --test-threads=1
cargo build --locked --bins
cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432
cargo rustc --locked --offline --bin autographics-authoring -- -C link-arg=/STACK:33554432
python tools/verify_source.py
python tools/verify_plan839.py --controls
python tools/check_scene_play.py research/merge839-play
python tools/check_play_owners.py research/merge839-play-owners
python tools/check_hierarchy.py research/merge839-hierarchy
python tools/check_hierarchy_pixel_faults.py research/merge839-hierarchy-pixels
python tools/check_scene_assets.py research/merge839-assets
python tools/check_asset_owners.py research/merge839-asset-owners
./target/debug/autographics-authoring.exe --window-probe --report research/merge839-window.json
./target/debug/autographics-authoring.exe --window-probe --sabotage-target --report research/merge839-window-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --report research/merge839-interaction.json
./target/debug/autographics-authoring.exe --interaction-check --sabotage-target --report research/merge839-interaction-wrong.json
./target/debug/autographics-authoring.exe --interaction-check --drop-release --report research/merge839-interaction-drop.json
./target/debug/autographics-auto-driver.exe --window-probe --report research/merge839-game1
./target/debug/autographics-auto-driver.exe --window-probe --sabotage-focus --report research/merge839-game1-wrong
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --report research/merge839-game2
./target/debug/autographics-auto-driver.exe --sample squash --window-probe --sabotage-focus --report research/merge839-game2-wrong
python tools/package_consumer.py D:/autostack/.wt/lang-839/consumer-merge839-NEW
```

The sabotage/drop commands are expected red at the actual named oracle; positive
and restoration commands must pass with nonzero selected cases, no required skips.
Fresh merge consumer repeats source controls/check/play/old-asset/native gates on
the frozen identity outside the producer, using its own target and stack settings.

From sibling read-only AGP: cargo test --locked -- --test-threads=1; cargo build
--locked --bins; graphics-probe --list; then --case <ID> --replay canonical --report
<new-dir> --mode auto for all16 IDs: gpu-foundation, shared-runtime, ag2d-breakout,
ag3d-risk-slice, ag3d-squash, lifecycle, metrics, verifier-negative-controls,
iced-shared-device, independent-consumer, surface-offscreen, surface-compose,
surface-input, surface-scenarios, surface-lifecycle, surface-ime-os. Use mode window
for surface-offscreen/input/ime-os. Set AGP_BASE_COMMIT to the actual frozen commit
for independent-consumer. iced-shared-device positive plus separate processes with
AGP_SABOTAGE_ICED=blank_blit and transparent_blit must check both counters AND actual
composed content; clear overrides for positive/other cases. Inherited exact gate
forms remain in hierarchy-handoff.md and scene-assets-handoff.md, with839 paths
and identity. This document adds play rather than removing any old denominator.

Only after final pass and all frozen complete gates: autographics-dev source landing,
reviewed Spec delta/overview/plans/goals/ledger deposit, main Plan archival, fresh
wt-guard clean and actual owned-tree/branch cleanup, batch-due receipt (tf main-only
single instance when due). Master receives knowledge only. Production I2/I3, Linux,
ACE/Shader/CAP and arbitrary script product remain pending/outside this slice.
