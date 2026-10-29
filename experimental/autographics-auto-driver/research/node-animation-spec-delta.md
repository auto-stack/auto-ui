# PLAN-006 r2 proposed canonical delta

Prepared in the implementation tree. Publication belongs to merge after the
independent final review. SD-01 adds `node-animation-and-facing.md`; SD-02/03
append the sections below to `behavior-api.md` and `scene-assets.md` in the
auto-lang knowledge repository. Existing profile rules stay in force.

## SD-01 canonical content

# Finite node animation and visual facing

The Windows x64 authoring driver provides the explicit
`autographics.node_animation3d/1` behavior profile (version110, profile21, CLI
`node-animation3d`) on the existing character3d/camera3d/objects3d/triggers3d
composition. Auto owns movement, jump/gravity state, clip choice, phase and
camera-relative facing. Host code validates, samples and publishes those
requests; it does not infer an action from speed or grounded state. This is
an engine capability and a finite verification fixture, not the G3 game,
Gallery, G10 or full AG milestone delivery.

An explicit `import-animated` / Import Animated GLB admits opaque textured or
flat GLBs in the existing static geometry/material/image domain, extended
only with finite node translation, rotation and scale clips. The immutable
original GLB, hierarchy and source-node/expanded-primitive mapping remain
available. Original Kenney `character.glb` at sample source3fa8a04 retains its
87624 bytes and all four ordinal clips:0 static,1 idle,2 walk,3 jump, with
44 LINEAR channels/337 channel key samples and8 nodes. No skin is used.

Every declared animation sampler/channel is validated before retained key
or output allocation and before image/geometry decode. Keys are finite,
nonnegative and strictly increasing; counts, scalar time/TRS type, storage
spans and values agree. Same-node/same-path duplicates within a clip refuse.
Hierarchy has unique parents and no cycles. Positive scales and finite,
invertible composed transforms are required. LINEAR translations/scales use
stable convex interpolation; rotations use normalized shortest-arc slerp,
including quaternion sign equivalence. STEP holds the lower key until the
next exact key. CUBICSPLINE, weights, skin, morph, mixing and root motion are
explicitly unsupported, including unused declarations that would otherwise
be silently ignored. Invalid data, unsupported features and resource limits
retain distinct diagnostics.

`ag_visual_pose(kind, entity, clip, phase, looping, yaw) int` is one atomic
charged operation. kind0 identifies a stable document model leaf; kind1
identifies a complete Run/slot/generation object token. clip=-1 is the base
pose; otherwise it is the unchanged file ordinal0..clip_count-1. phase is
finite nonnegative f32 seconds, supplied by Auto at fixed tick. looping is
exactly0 or1: loop uses duration modulo (exact duration returns to start),
one-shot clamps at the last key, and zero-duration holds its sole pose.
Changing clip samples the supplied phase immediately. There is no Host or
render-time animation clock. yaw is finite radians, abs<=1024*TAU (f32 bound
6433.98175455), normalized to[-PI,PI). Auto retains facing while stopped.

The model visual pivot is its origin; document transform is translation ×
yaw × pitch × scale under the existing hierarchy rules. For each expanded
mesh the shared color/shadow matrix is document-or-instance world × visual
yaw × sampled source-node world × inverse(baked base-node world). It reuses
immutable geometry/UV/texture owners without per-tick decode or upload.
Nonuniform scale uses inverse-transpose normals in both relevant vertex
paths. Animation and visual yaw do not change frozen collision bounds,
solid transforms, character position, support or trigger evaluation.

The independent Host wire contains4 int header values (status1, target
count, Run, tick), then the frozen target rows and8 potential slot rows.
Each row has4 ints and2 floats: static node ordinal/asset ordinal/clip/loop,
or slot generation/asset ordinal/clip/loop, followed by phase/yaw. Free rows
are canonical zero. Run/tick, types, lengths, clip/bool/value domains,
resource identity, target coverage, actual instance template and generation
are independently checked. Each sampled pose travels in the same candidate
Frame as nodes, physics, camera, objects and events. Prepare and deadline
checks precede acceptance/publication. Invalid output, stale identity,
budget or preparation failure preserves the complete last-good presentation.

New slots begin from the template's initial base visual, never a prior live
template or old slot-generation pose. Destruction revokes that visual.
Pause does not advance phase; neutral Step emits one fixed tick with no
paused input edge or camera delta. New Run/round rebuilds Auto app_start
state; Stop restores the editing presentation. Twenty repeated
Play/restart/Stop cycles prove actual VM/descriptor/pose owner release and
zero warm mesh/texture upload/decode increments. Accepted Editor or external
pins remain legitimate; final payload/mesh/texture Weak references expire
only after those final owners are released. Driver residency is excluded.

Existing raw8 identities/32MiB union, single8MiB raw, GLB JSON1MiB, CPU/GPU
64MiB each, expanded primitives16 and potential mesh instances512 remain.
Animation adds at most128 nodes,16 clips,128 channels and8192 accumulated
channel key samples per asset. CPU admission includes animation containers,
names, topology, input/output arrays, metadata and conservative per-instance
last-good/candidate/prepare pose storage, Arc/key headers, target table and
wire. All8 potential slots pay the largest template even when hidden. Full
resident/dependency union admission precedes retained animation decode; shared
payloads are charged once, per-instance poses are reserved separately.
`resource_snapshot` exposes that same pose reservation and the total CPU
union, plus actual mesh/texture upload counters. Accepted CPU prediction and
actual immutable GPU allocation remain within the existing budgets.

Managed96KiB/user256KiB/combined352KiB,32 shared credits,1000 steps/slice,
100000 cumulative steps,120 resumptions,250ms callback,30s initialization,
2s whole tick and the existing shared pump limits are unchanged. The input
snapshot and single visual setter each cost1. A complete active Auto path
uses1 input snapshot+26 character move+1 grounded+1 event_next+1 event_pair+
1 event_phase+1 visual=32; an additional charged read faults10 and preserves
the complete last-good frame. Camera basis reads remain free under their
existing contract. With8 reserved instance slots, the node-animation Run
supports120 base nodes (128 potential) in wide/sparse layouts and64 base
nodes for the inherited deep/contact envelope. A120-node deep layout is
refused by the unchanged cumulative step guard; no clipping of the scene
or reduced physics is substituted.

Current regressions: driver `public_node_animation`,
`node_animation_resources`, `node_animation_pixels`,
`node_animation_lifecycle`, `node_animation_consumer`; AGP `node_animation`.
Their independent source-key/f64 vertex and real GPU silhouette/shadow
oracles distinguish removed tracks, wrong yaw, interpolated STEP and frozen
shadow pose. They also cover admission before allocation, exact limits,
closure corruption/save recovery, full-frame refusal, slot reuse, physical
invariance and real owners. `check_node_animation.py --controls` binds exact
source closure, actual runners/counts/stacks/logs and observed budgets and
rejects altered/partial/empty proofs. The public bound scene survives SaveAs,
move and cold reopen in a source-only exported, offline-locked consumer.

## SD-02 append to behavior-api.md

## Node animation3d/1

`autographics.node_animation3d/1` is version110/profile21; CLI selection is
`node-animation3d`. It extends the frozen triggers3d composition with the
finite visual setter and a profile-local `ag_actions_snapshot() int`.
Both operations cost1 from the unchanged32-credit budget. Old14 profile
assembly bytes and old per-key getter prices remain unchanged.

The snapshot packs six held bits and six unread pressed bits as
`held + 64*pressed` (0..4095). Bits0..5 are Left,Right,Forward,Back,Jump,Restart.
A successful read consumes all currently unread pressed bits and preserves
held. It shares the exact consume-once state with `ag_action_pressed`:
previously consumed bits are absent and later getters cannot replay them.
app_start/fresh Run and neutral Step see neutral input; Pause follows the
existing input-clearing route. There is no extra Host input wire, retained
cross-tick snapshot or Host gameplay. Auto decodes the finite packet.

`ag_visual_pose(kind,entity,clip,phase,looping,yaw)` uses static stable IDs or
full object tokens, original clip ordinals, explicit phase seconds and
visual-only yaw radians. Wire/identity, deterministic sampling, exact
resource and lifecycle rules are specified in
[Finite node animation and visual facing](node-animation-and-facing.md).
Old behavior profiles reject an admitted animated payload instead of
silently discarding its tracks. The profile does not combine2D sprite APIs
with3D animation in one Run; the old sprite profile remains supported.

## SD-03 append to scene-assets.md

## Explicit animated model closures

The new `import-animated` action admits finite no-skin node TRS animation in
the opaque textured or untextured GLB domain. Ordinary import keeps its
previous static domain and refuses animations. The original root bytes,
node/primitive association, image dependencies and source URI strings are
unchanged. A schema6 model closure marks `node_animation:true` and hashes
the same compact tuple as textured/1 with domain
`autographics.node_animation3d/1`. Default false is omitted and retains the
old `autographics.textured/1` identity exactly. The flag cannot downgrade an
animated payload into a static profile, even with a recomputed closure ID.

SaveAs and pending-save Verify retain/recheck original accepted GLB and PNG
bytes and the complete edge metadata. Cold reopen reconstructs all declared
clips from the same immutable closure; a bound Auto script is content
addressed and relocatable with the project. Raw/current/history/dependency
and decoded union accounting, no-overwrite installation and entry-last
transaction rules are inherited. Animation payload and conservative pose
reservation stay inside the existing64MiB CPU total. See
[Finite node animation and visual facing](node-animation-and-facing.md)
for support, limits and publication rules; this does not add skin/morph,
animation blending, a new collision domain or a complete game.
