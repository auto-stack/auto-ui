# PLAN-839 retained scene play feasibility

Source base: 838 fully delivered `37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`.
One tree `D:/autostack/.wt/lang-839/auto-lang`, branch `plan-839-dev`.
SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`, 680 pinned files;
metadata closure479, 17 local packages,29 path edges. Driver lock
`2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db`;
AGP lock `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`.
No SDK, AGP, root dependency, production crate or persisted schema changes.

Two local approaches were inspected: taking over Editor.preview.vm would mutate
the retained accepted owner and make restoration depend on reconfiguration of a
possibly contaminated VM. Chosen: a separate Session and bounded scalar queue,
with one complete private Frame and last-good Frame, retaining the accepted Editor
and a directly restorable Prepared draw/instance/store snapshot. This uses the
existing MeshStore::staged() Arc-sharing seam on the same device, not another world
engine or a different renderer. `prepare_frame` is a local generalization of the
existing Preview preparation; old Preview callers retain the same wrapper.

`scene_play_feasibility::retained_auto_current_imported_scene_restore_and_faults`
first exercised an independent scratch Session and an actual accepted two-part
GLB under parent translation+1, local-1, plus a2D leaf.61 real ticks, exact neutral
step, actual red/green all-part GPU centroids, no Editor writes, original composed
image restoration without compile/import, last VM owner release, type/budget
fault and fresh initialization passed. The scratch test remains explicitly a
probe, not a second portable-scene assertion. It was followed by the incremental
profile6/Host prototype and actual native Run→2D/3D keys→Pause/Step→Stop→Run
positive at phase16; subsequent formal27-phase probe adds UI/V/focus cases.
These observations closed the local mechanism decision before promoting the
prototype into the formal retained play module and task acceptance cases.

Actual path: accepted Document/Registry → Play::new snapshot and parent-first
ordinal configuration → samples/authoring_play/main.at profile6 → retained Session
start/pump → typed scalar queries → full admission → Host::prepare_frame → accept
and publish. CPU hierarchy derivation only rejects inconsistent world/effective;
actual Auto outputs feed renderer, bounds and presented snapshots. One runtime
VM closes on Stop/fault, while Editor.preview.vm and accepted mesh Arcs remain
legitimate owners. Stop publishes retained Prepared data with no VM call, resource
prepare or disk read. New Run creates another nonce and Session at initialtick0.

Profile6 header is four integers: profile/run/tick/count. Each node returns seven
integers (ordinal/kind/dimension/parent/resource/local-visible/effective-visible),
three local f32 coordinates and twelve world/static f32 fields. Maximum128 nodes
therefore gives2820 typed results, under4096; integer identities never pass through
f32. Reset has3 arguments; action(space,x,y,toggle) has4; tick(next,dt) has2 and gets
the actual fixed f32 1/60. All queries are queued once per transaction and share
one8ms pump round. Init30s and whole-tick2s include output/prepare, with synchronous
compile/native limitations retained. Clock has one inflight, four credits and at
most four dispatches per round; gap>250ms/backlog>4 auto-Pause clears input/debt.

Formal checks additionally establish600-tick independent f32 traces, two display
batch schedules,128 nodes, all immutable/world/resource/nonce rejection branches,
budget/type/start/prepare/Stop/stale-token paths, dirty/redo/unsaved-asset/file
isolation, actual changed-Auto-speed/visibility GPU consumption,20 VM/resource
cycles and final external-pin release. Actual retained-VM and fresh-upload faults
make the same owner gate red; restored controls pass.

Native probe repairs concerned the oracle and input harness: use sRGB expected
colors for linear cyan; post real scan codes and WM_CHAR for printable keys;
separate Resume and canvas clicks across event rounds (a canvas click correctly
releases Iced focus and clears the old widget event batch); pause before hidden
pixel readback. Multiple synchronous GPU readbacks can trigger the specified
four-credit overload protection. The probe records this and resumes via actual
widgets/refocuses before keys, without relaxing the production clock. Raw failed
attempts remain in the work receipt. This is development verification, not final
independent review; review and complete frozen merge gates remain pending.

Timing: real compile and each full frame transaction are in native tick records;
Host records bounded Run/Stop/pump durations and frame CPU distributions; native
report records Windows working set.128-node timing comes from the actual128-node
case. Model writes are measured at existing GPU buffer upload calls; per-instance,
content/composition writes remain separately reported. No GPU timestamp, Iced
internal allocation or driver residency claim is made.
