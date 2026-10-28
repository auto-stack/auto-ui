# PLAN-848 revision 3: repair F-848-R1-01

The r2 budget claim below was rejected by R1 and must not be read as current acceptance.
Work-r3 alternatives were not adopted. Work-r4 first established feasibility on60
actual Session controls and6 Host controls, then revised the shared Plan to r3.
The implementation keeps fixed SDK,128 nodes,32credits and all CPU deadlines.

The v3 shared derive rotates the original axis formula into per-dirty-axis
parent-first passes and updates the existing arrays in place. Four scalar flags
track X/Y/Z/visibility; scalar setters mark their component, move marks all XYZ,
initialization/incomplete shape or an explicit unmarked derive recomputes all.
There is no second graph or candidate/world-box cache. World addition remains
local+parent with the same f32 order, with parent reuse confined to one axis pass.

Only trusted v3 numeric units have compact generated line layout: the pinned SDK
charges SOURCE_LINE instructions, so avoiding redundant line changes reduces
actual charged instructions. The original parser/codegen/VM and cumulative
counter are unchanged. Raw user bytes and v1/v2 managed units remain untouched.
The readable source is retained; generated region diagnostics describe their
actual compact layout, without claiming original managed-line mapping.

The slab still validates full matching geometry before pruning and computes
actual f32 intervals. Directional offset selection removes swaps. It stores
only final t/node and materializes final point/XYZ-first normal once. Explicit
float bounds/quotients are essential to typed nonfinite comparisons on this SDK;
the old geometry error and quotient-overflow tests remain unchanged.

Permanent segment_budget and exact segment_counterexamples replays assert
success,32credits and the independent observable results, not just exit0.
segment_derive covers mixed components, move, visibility, final non-deriving
fields, f32 order, raw source preservation and missing-mark mutations.
The result11 scalars/44B plus dirty4 scalars/16B total60B logical payload.
Actual SDK storage and allocation boundaries are measured separately.
Work/internal/final stages remain scoped; unified regression is merge-owned.

---
Historical r2/r1 records follow; superseded budget assertions are not current pass.

# PLAN-848 revision 2: current implementation

The user authorized a new local scheme, revision of the same Plan, then implementation. The main Plan was revised before public v3 integration. Current code uses version-isolated 2D/3D unrolled kernels, geometry-first validation, safe nearest/interval pruning, callback-local token invalidation, and a specialization of the original shared derive with axis unrolling and last-parent coordinate reuse within one derivation. Clean callback exits do not repeat an already current derive. No SDK, AGP, lock, graph identity or support/budget change.

Actual v3 tests now pass wide/deep128, two group writes and exact32 credits through both Session and Host. Normal/point observations use the agreed float tolerance; Auto test source uses explicit ranges, avoiding an uncontracted float abs convenience call. Native samples and all-mesh/transparent/missing-part/wrong-asset counterexamples have run. Detailed commands and raw results are on the main checkout under attachments/848/work-r1. This is implementation evidence, not independent final review or merge regression.

The original investigation below is historical r1 and is superseded by revision 2.

# PLAN-848 T-02 bounded feasibility

Baseline: delivered graphics bf22db3465203f3babd649494f4b37aa2a012027,
SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. T-01 source and
dependency checks passed; main-checkout evidence is work-r1/frozen-source.json.txt.

Candidate A uses shared v2 spatial metadata and Auto world derivation, an
axis-loop closed slab, ascending ordinal traversal and eleven persistent scalar
result/sequence fields. No native intersection, per-cast arrays or heap objects
are introduced. This is a feasibility candidate, not a public v3 selection.

Path: SourceSnapshot v2 trusted assembly plus candidate unit -> actual Session
initialization -> accepted ordinal/box configuration -> callback -> Auto scan ->
snapshot getters -> existing whole-frame consumer (consumer integration pending).
The prototype test assembles trusted code separately without modifying the v2
API, transport or entry bytes. Public v3 admission and callback expiry remain
pending T-03/T-05 and must be included in this feasibility gate before closure.

Distinguishing first cases: identical boxes must retain ordinal zero, with
fraction .25, point (-1,0,0), normal (-1,0,0); a subsequent node move must not
change the snapshot. A last-wins tie mutation changes the asserted ordinal.
The 128-leaf case executes two full casts, ten getters and six old queries under
the original Session budget. This case does not replace deep hierarchy,
Descriptor/GLB, numerical producer, token expiry, GPU or Stop checks.

Closure requires all section 5.1 observations, including full consumer/GLB/GPU,
old-version legitimate helper, numerical and 128 wide/deep controls. Initial
No T-02 pass or public v3 behavior is claimed.

## Observed outcome: needs_replan

Candidate A: two-node nearest/tie/point/normal/snapshot control passes; 128
matching 3D leaves, two full casts, ten getters and six AABB getters exhaust
the unchanged cumulative 100000-step callback budget. Candidate B inlines
finite checks and spatial edges to remove repeated helper calls; the same
128-leaf case still exhausts the same budget. On the final formatted candidate
B, the complete targeted file selects three tests: two controls pass (two-node
geometry/snapshot and identical mixed workload with sixteen leaves), and the
required 128 case fails at the actual VM budget check. Exit 101 is retained.

The failing case is already weaker than the full acceptance workload: no
deep hierarchy and only one intervening leaf move, rather than two parent
writes. Its failure is enough to reject these candidates; no inference is made
that all possible Auto implementations are impossible. Initialization and
compilation complete successfully, distinguishing runtime budget failure from
setup/parse/admission failure. Earlier prototype setup mistakes were corrected
(spaced subtraction syntax, admitted user observation, namespaced tick entry).

Remaining required GPU/Descriptor/full lifecycle, nonfinite and mutation
controls were not run because T-02 cannot pass its CPU prerequisite. The
last-wins mutation was identified but not executed. Existing fixed SDK emits
146 compiler warnings; no SDK files or warning policy were changed.

Bounded next decision: revise T-02's local implementation investigation around
a measured Auto operation budget (e.g. specialized axis calculation/earlier
cheap rejection while retaining finite geometry validation for every matching
candidate). Preserve 128 nodes, 32 credits, original callback budget, actual
Auto computation and SDK/AGP/production read-only boundaries. Any SDK expansion
or changed numerical/support policy needs a concrete contract revision; none
is authorized by this probe. Do not advance T-03..08 or run merge tiers.
