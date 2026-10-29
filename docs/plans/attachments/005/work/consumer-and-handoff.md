# UI-005 T-06 consumer and review handoff

Authoritative implementation: `D:/autostack/.wt/ui-005/auto-ui`, branch
`codex/plan-005-dev`, frozen commit `9d680f9574275d13a0f776ee321534f5a9d4c465`;
Plan r3. No implementation writer remains active. Review/merge are pending.

## Current source and portable public project

Current identity binds 1456 source/asset files, including 681 frozen SDK files,
the unchanged two package locks, original three GLBs/shared PNG, project,
report checker and proposed Spec delta. Identity SHA-256:
`6d0fc6d69d73c1c51453cb31254bea3edc3ef0d32b1323e6887a2994fd4f4bf3`.
`verify_plan005.py --controls` passed again on the frozen tree: actual edits,
deletions, unlisted/stable scenes, incorrect pins, escapes and pinning volatile
PID outputs are rejected; only the bounded volatile PID output is ignored.
`generate_project.py --check` and both packages' `cargo fmt -- --check` exit 0.

`samples/textured_models/scene.agscene` is a public scene/6 project with three
original model roots, one shared PNG and an actual bound objects3d Auto script.
The permanent public-project test opens it, runs Auto instances/camera, saves
to A, moves to B without the original model/Textures directories, reopens,
compares real GPU output and runs the bound Auto again. Original GLB bytes and
node/mesh/primitive/world associations remain available for 006. Public README
commands explicitly select the package Cargo configuration (32MiB main stack,
16MiB workers) even when launched from the repository root.

## Fresh consumer evidence

Artifacts below are under `D:/autostack/plan-artifacts/auto-ui/plan-005/`.
Export used `package_consumer.py --snapshot-only`, confirmed `.sdk-source`
absent, then physically moved consumer-A-final to **consumer-B-final**.
The latter owns independent driver/AGP targets. From its driver package:

- `python tools/prepare_sdk.py --from-snapshot`: exit 0, 681 files/17 crates;
  `t06-consumer-cold-sdk.log`, SHA256
  `b43c91c33da5bd2c006890ced32450e70194be174fa4d20ccfee01ad2d98bb53`.
- `python tools/verify_plan005.py --controls`: exit 0, 1456 hashes and all
  actual source controls; `t06-consumer-identity.log`.
- `python tools/check_textured_models.py --reports <t06-consumer-gate> --controls`:
  exit 0, driver18/AGP4, no failures/ignored/filtered, 12 report cases. Includes
  36 sampling combinations/144 probes, depth/backface/shadow, real original
  GLB transforms, union budgets, transaction failures, 20 actual Auto Runs,
  final-owner release and portable bound project. Pixel, partial, runner,
  budget, warm-upload, stale and actual zero-selected executable controls
  all refuse. Source verification also checks SDK479 packages/29 local edges.
  Both actual PE test programs reserve 33554432 bytes; worker environment
  16777216 bytes. Source identity, actual executable hashes, logs and fresh
  run nonce are bound in `t06-consumer-gate/manifest.json`, SHA256
  `ff76986eb9615d0c4120ee2ee9c2dc60f974f86f872e6925bee9df6192d95ef1`.
  Gate log `t06-consumer-gate.log`, SHA256
  `661096bf6dc87fbee401a3979ef77cd95a5f5a53035bb34f210236003be938d6`.

Producer `t06-producer-final` also passed 18+4 and report controls. It predates
the README explicit-config correction and addition of PE-stack fields to the
checker; it is supporting runtime evidence only, not the final-source proof.
The fresh consumer gate above runs the complete current source/checker.
An earlier producer checker attempt failed because its zero-selected Cargo
control relinked the bound executable; the repaired control invokes that
recorded executable directly, retains its hash and still rejects zero tests.
No validation threshold was weakened. Merge owns a new frozen producer gate.

## Affected compatibility

Producer driver commands, all `--locked --offline -- --test-threads=1`:
`public_sprite_frames::older_profiles_keep_their_exact_assembled_bytes_and_the_new_one_fits`
1 passed (`t06-old-profile-bytes.log`);
`authoring_assets::interleaved_sparse_uv_and_single_transform_controls_preserve_original_geometry_and_bytes`
1 passed (`t06-old-glb-uv.log`). Consumer B independently passed the same old
profile case and both `public_sprite_frames::png_closure*` cases (2 passed),
including save/reopen/move and missing/tampered/foreign/budget refusal;
logs `t06-consumer-old-profile.log`, `t06-consumer-png-closure.log`.
Producer AGP `cargo test --locked --offline --test asset_resources -- --test-threads=1`
with dev debug=0: 1 passed, actual u16 boundary/legacy vertex draw/device and
last-pin ownership; `t06-legacy-agp.log`, SHA256
`cd81d143957fc7130b712eb7a5dfcc408821591c2a4b530340fb6f6fb4e94520`.
Earlier T-03 raw/geometry boundary samples remain valid because their code,
inputs, SDK and locks are unchanged; final suite covers the new union domain.

## Readiness and stage ownership

AC-01/V-01: fixed-source and admission-and-storage summaries plus full current
identity/budget suite. AC-02/V-02: storage negatives/recovery and public moved
project. AC-03/V-03: pixels-and-lifetimes numerical GPU oracle and controls.
AC-04/V-04: real transaction, union-accounting, 20 Runs and actual Weak owners.
AC-05/V-05: fresh consumer above, compatibility, public commands and retained
006 associations. All work-owned criteria complete; no open in-contract
finding, decision or work coverage gap is known.

Canonical proposed SD-01/02 is committed in
`experimental/autographics-auto-driver/research/textured-model-spec-delta.md`;
canonical Specs have not yet been published. Independent review must inspect
actual AC/V/SD/source and evidence, then bind verdict to Plan r3/current commit.
T-07 review, T-08 frozen scoped gate/landing, T-09 canonical Specs/ledger/archive,
T-10 guarded cleanup remain pending. Full AG milestone and 003's deferred
batch regression are outside this scoped work-complete claim.
