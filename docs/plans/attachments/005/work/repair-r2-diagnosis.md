# UI-005 two-final-failure diagnosis and cycle2 closure contract

Before any cycle2 implementation edit: current clean implementation is
`8809e545830a2d0ed6f587517e6d4d8a29a575ca`, Plan r3, R1/R2 both needs_fix.
F-005-R1-01 is independently closed. F-005-R1-02 remains open as an invariant,
despite the original warm-Open example passing. No change to support or ACs.

## Diagnosis and caller inventory

The first repair changed textured load admission only. Its permanent test
stopped at Host Open+settle and dependency corruption/recovery. Neither that
test nor the fresh full suite ran subsequent Save/SaveAs/Play with a document
name different from the reused Asset Arc's import name. All ordinary fixtures
had matching names. Thus aggregate20+4 green did not establish the required
whole public workflow. Review R2's actual cold/same-name controls and warm
Save/Play counterexample identify a shared identity-policy mismatch, not an
environment failure. A second point fix would repeat the same design error.

Read-only inventory covers all `src/authoring` identity comparisons:

| Site | Required semantics / current problem |
|---|---|
| assets::AssetRef key/import/cache | content key; name already nonbinding |
| textured_models::load_declared | six content fields, repaired locally |
| session::validate_assets | same six content fields already, separate list |
| storage::save_snapshot | full AssetRef equality wrongly includes name |
| play::new_program | full equality wrongly includes name for every profile |
| play sprites target admission | full equality wrongly includes name; generic check currently masks it |
| storage pending Verify | exact serialized snapshot/blob readback, not cached metadata equality; preserve it |
| behavior_project source identity / document hash / stamps | distinct identities, not AssetRef comparisons; preserve them |

## Design adjustment and required closing oracle

Define one AssetRef content-binding comparison for SHA256, byte length, format,
width, height and the entire closure (including ordered edges and their data).
Exclude only display name. Use that same comparison for asset load, session,
snapshot and both Play boundaries. Keep full-record Eq for metadata/serialization
comparisons where appropriate; do not redefine Eq globally or mutate shared
immutable payload names to agree with a document. Preserve root/dependency disk
readback, entry-last save, pending Verify and all raw/decoded/GPU limits.

Permanent actual-flow checks must cover identical content with different names
across textured models, legacy flat GLB and 004 PNG sprites (the changed shared
admission sibling). For textured model: save alias project; cold/same-name and
different-name cached Open; real Host Play/Stop; save current and SaveAs;
reopen/readback retains document name and exact bytes with immutable cached Arc.
Exercise pending save Verify and corruption/restore. For sprites: real public
sprites profile must pass both generic and target admission after cached reuse.
For legacy flat GLB: public save/reopen and generic Play must remain accepted.
Content changes (root/length/format/dimensions/closure and forged edge) must
still reject; same root with different PNG closures remains distinct. Replay
the R2 assertion-enabled executable after rebuilding it against repaired code,
plus the original R1 material holdouts and accepted controls.

Only after these adjusted oracles execute successfully may cycle2 claim work
readiness. Rebind current source/consumer evidence; reuse unrelated GPU/SDK/lock
evidence explicitly. Independent affected R3 must assess full invariant closure
and this verified adjustment. Merge-owned frozen gate remains pending.

Verified adjustment: shared binding is implemented at all six admission sites;
textured model lifecycle including pending Verify corruption/restore, actual
legacy GLB and sprites profile startup/Stop/save/reopen pass. R2 original
lifecycle executable rebuilt against current library reports all operations Ok;
material21/5 controls remain green. New cold independent consumer21+4 and all
report controls pass at frozen7437879. Detailed closure/evidence binding is in
[repair-r2.md](repair-r2.md). This diagnosis is not merely a proposal retained
without implementation or verification.
