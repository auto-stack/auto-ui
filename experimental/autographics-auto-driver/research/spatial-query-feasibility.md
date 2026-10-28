# PLAN-846:r2 spatial feasibility

Status: feasible; work-owned implementation checks are recorded on main in
`docs/plans/attachments/846/work-r2`. This report supersedes the r1 needs_replan
state stored in Git118530ee5 and the preserved work-r1 artifacts.

The observed native sqrt(-1) f64 NaN formerly encodedfff8000000000000, collided
with the i64 tag and reached float queries as zero. The authorized upstream
fix4cef80c1b canonicalizes only NaN sign and excludes raw negative infinity from
tags. SDK artifact7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 has exactly one file
difference from932ad; its permanent local tag preserves future Git preparation.
17 scalar tests, actual old-encoder mutation and both actual Auto query guards
with accepted finite controls distinguish the fix. Parameters/locals/helper
returns/containers and nonfinite siblings retain error semantics. No source,
function support, numerical tolerance, query quota or CPU budget was weakened.

V1 assembly remains byte-identical in its managed units. V2 shares those units
and adds an explicit API/profile8. SDK/source/hash/version/base/geometry identity
is rebuilt before Run. Actual Host models and accepted Registry ModelData produce
one local-box scan per resource per Run, one eight-corner transform per node.
Auto executes world derivation, endpoint arithmetic, closed comparisons and
private trigger state. All configuration precedes app_start. Malformed metadata
was additionally tested with an actual startup counter: before the final guard
invalid configuration ran app_start once; the fixed guard runs it zero times,
while valid configuration runs it exactly once. Incomplete/duplicate/out-of-order
configuration never publishes. Two earlier attempts to inspect VM state after
public failure had invalid setup because failure already closes the VM; those
logs are not counted as the distinguishing red.

128-node wide and deep actual Host initializations measured302918/240148us,
897 configuration calls,12835/12624us spatial configuration time. Original
32-mixed-query/two-parent-write controls pass within original callback budgets;
33rd attempts fail before geometry access. Whole-callback fault, last-good,
Starting cancellation, Stop, new source/new Run and all-mesh GPU controls pass.
Logical static box payload is3584B at128 nodes; queue/JSON/VM containers are extra
bounded overhead, not represented as raw-float bytes.

Both source projects support real OS input, enter/stay/exit, neutral Step,
Pause/Resume/Stop/new Run. Real two-part GLB pixels are independently inspected.
A source-only overlap-to-contains change is accepted by OS Bind/update+Save and
reopened by a fresh process with the same runner after removing its original
file. Native pixel controls actually corrupt GPU content; setup failures do not
count. An initial native readback while Running triggered the existing backlog
auto-Pause. Moving oracle readbacks to Paused/Editing fixed the probe, without
relaxing runtime policy. All original failed raw reports are retained.

Twenty actual v1/v2/Starting-cancel/Fault/Undo/Redo/Save/Open/Stop cycles verify
source/VM/spatial/asset/mesh final owners and warm geometry reuse. Five actual
retained-owner/rescan/upload mutations fail their specific assertions, followed
by a restored20-cycle pass. Separate legitimate external source/asset/mesh pins
remain until their last owner drops. No arbitrary private VM heap quota or GPU
driver residency claim is inferred. Full package/legacy/root regression remains
merge-owned pending after final independent review.
