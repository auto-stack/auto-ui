# Current F-837-R1-01 correction

Independent final R1 proved that the original visible move oracle inspected state,
picking and commands and recorded hashes but never compared endpoint/origin pixels.
The original pixel coverage claim below/in the first handoff was invalid. Repair
adds actual final-top pixel assertions at temporary, move, undo, redo, hidden,
show and saved/unselected states; initial colors must exist and differ, missing
readback fails. The aggregate requires seven independently recorded state checks.
Original R1 transparent GPU replay is rejected at phase9 after repair; permanent
transparent/old-position-ghost rendering controls preserve valid state/commands
and each fail the same native/report/aggregate pixel gate. Source and graph/Auto/
transaction/SDK/resource semantics are unchanged. Detailed closure and reuse
boundaries are in hierarchy-repair-r1.md; final re-review and merge gates pending.

# PLAN-837 T-02 feasibility and execution contract

Input is G836 delivery52ec4c7c1, fixed SDK9576b4d; preparation and all inherited
227 source/asset fingerprints passed before edits (hierarchy-source.md).
No SDK, AGP, root workspace, games, original assets or registry locks changed.

One driver-local approach was needed: profile4 using the existing public
four-scalar callback limit and bounded Session. identity(ordinal,kind,dimension,
parent) plus visibility(int), append_field(float), derive(), complete queries.
Root parent=-1; parents are zero-based ordinals in a deterministic parent-first
forest. Kind5 means group, dimensions2/3. IDs are persistent nonzero u64 mapped
bidirectionally by host order, never cast to SDK int.12 float fields per node;
group unused fields are fixed. Auto itself adds each parent's f32 world once
and computes effective visibility. Rust independently admits every output field;
actual returned world/effective drive render/pick/overlay and group origins.

The initial scratch test hierarchy_feasibility passed actual Session, real Iced
button, original GLB/GPU publication, rejected resource candidate, undo and
scratch topology encoding. It intentionally did not claim production migration
or native acceptance. Its prototype was extended in-place through the actual
Document/Building/Editor/Host chain to close the whole path, rather than invent
a second editor or change AGP. Formal codec, transactional and ownership hardening
then uses authoring_hierarchy's permanent assertions.

Current closure evidence:

| Rule | Actual path/owners/states | Distinguishing closing observation |
|---|---|---|
| graph and migration | bounded unique-key JSON -> Document v2 -> hierarchy::ordered/derive; no VM/GPU before validate | strict_v2_forest_and_explicit_v1_migration plus shared_capacity_deep_chain_hidden_world_and_u64_max accept128/deep/u64MAX/v1, reject cycle/missing/leaf/cross-dimension/unknown transform/129/hidden world overflow |
| real Auto and failure isolation | local nodes -> Session profile4 -> Building::finish -> returned Preview -> Host::prepare/publish | actual_auto_world_output_and_mutation_controls: wrong accumulation/AND semantics/parent/ordinal/profile each rejects; accepted control recovers |
| structure/history | stamped Edit -> before/after transaction -> full candidate Auto/resource -> atomic commit | reparent_keep_world_visibility_precision_and_atomic_rejections and subtree_history_order_branch_and_empty_groups; f32 cancellation at0.002 vs parent100000 explicitly rejects; valid10 remains accepted; one history step/order/IDs restored |
| UI and gestures | real tree/parent widgets -> shared Editor; presented actual derived snapshot -> capture -> one inflight/one latest/one shown VM | native hierarchy positive creates group, attaches leaf, moves group, undo/redo/hide/show/save;10 scoped tests also exercise nested2D and3D axes, cancellation and resource failure followed by recovery |
| persistence | original v1 bytes retained by unchanged Storage -> explicit v2 save -> new process inspect | raw-byte external modification rejects without overwrite; v2 nested scene hash/world/unselected2D+3D captures identical, clean/empty history/selection |
| lifetime | accepted/shown/inflight owners stay bounded; fixed7 mesh pins and target identities |20 hierarchy rounds include move/cancel/reparent/failure/delete/rebuild; actual retained-VM and fresh-mesh source mutations fail this same test; exact source restoration passes; legal runtime/mesh pins remain until last drop |

Native hierarchy positive completed phase16/204frames; wrong-parent phase5,
drop-release timeout phase9, and frames1 incomplete all exit1/passed=false.
Original native property probe phase9 and836 interaction phase27 also pass.
Widget geometry comes from actual runtime layout traversal through driver-local
tagged wrappers; it does not replace controls, events, focus, rendering or AGP.
Initial container-only ID lookup failed; a first dropdown keyboard probe selected
Root and correctly failed the parent oracle. Explicit stable-ID parent buttons
were chosen as the ordinary finite UI. The old property fixture initially still
queried an unsupported container lookup; routing Apply through the same actual
layout witness fixed that fixture. No acceptance weakened.

No architecture/semantic blocker remained: group translation only, same dimension,
keep-world position with specified precision rejection, fixed resources and old
gesture cancellation semantics suffice. There was no SDK/API expansion and no
second competing approach. Near/far-straddling model bounds use all projected
corners for conservative aggregate outline; group handle origin has strict clip
checks. Tree indentation is visually capped at8 while IDs/complete128-node graph
remain present, traversable and editable. Parent options are explicit IDs.

Compile remains synchronous; pump rounds remain8ms/call budgets unchanged.
No claim of8ms frame latency, GPU timing, Iced internal allocation accounting,
production I2/I3/Forge or general TRS. Native timing measurements belong to the
bound report, not a performance guarantee. Independent review and complete
merge-owned gates are pending.
