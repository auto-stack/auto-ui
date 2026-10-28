# PLAN-845 r2 implementation handoff

Authoritative tree D:/autostack/.wt/lang-845/auto-lang, branch plan-845-dev. Base8b62db5cba54b9bfe7f759f4920b22ccb9b75e88. SDK932ad4877e5968f1aa7227d9dcd7fb69b4c69a21; driver lock2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db, SDK lock5192d08d502a8f17e277ad8912f5884175659bef642b614e25b394243d1ff091. Actual frozen implementation commit and work evidence are recorded in the main-checkout Plan845. Only this driver changed; SDK/AGP/root Cargo sources and locks stay read-only. No links or additional Git implementation tree.

G840 accepts the explicitly disclosed same-implementation-session review limitation by user decision, retained in Plan r2. This does not waive845 final independent review. Work completion cannot merge/archive/deliver the Plan. T09/AC09 final review and T10/AC10 complete gates remain pending and separately owned.

## Implemented path and work-scoped verification map

* Document.decode/validate/encode: scene4 required behavior/null, explicit nullable leaf IDs and strict old versions. behavior_project_document tests2; authoring_document/authoring_hierarchy/authoring_assets preserve old geometry constraints and denominators (only schema expectations and synthetic v1 fixture construction updated).
* CodePayload::from_bytes/load_declared/snapshot: same public840 validator and compiler; hash/length before consuming declared bytes, raw identity and shared Arc payload; no callback execution at bind/Open. behavior_project_session raw/bounds, compile-failure and callback-infinite accepted controls.
* Editor.begin/commit/current/pending/Step: candidate binding, retarget/detach/cancel, original preview publication, dirty/hash/Undo/Redo/noop, references/ancestor deletion, weak pool quotas and last-owner release. Four session cases, two Host cases, owner mutation checks. Canonical64step/16MiB history remains independent of code8/2MiB. External pin_bytes is explicitly separate from the payload object, and does not prove a leak.
* Storage: original root lock and one scene rename; source install/readback and complete receipt; reserved target checked before receipt verification, budget8files/2MiB incl reservations/residuals. Five storage cases, including real GLB installed before code failure and corrupt GLB/code receipt verification, original asset/storage coverage retained. Generic pre-entry write/flush faults preserve original code-independent implementation; unchanged baseline controls remain in old modules.
* Host/UI/CLI: project default consumes accepted bytes/roles, explicit external/builtin override, default restoration, no busy queue; failed prepare/Open retains content, new epoch clears overrides, inspect rebuilds clean/no-history. Bind/update, Undo/Redo and Save As run through real OS/Iced controls in the native probe, then distinct logic/speed traces and all-part GPU pixels on the same runner.

## Full merge matrix — owner merge, all pending until independent review pass

Run on the reviewed frozen code/dependencies/configuration and record exact command, exit/count and artifact hashes. For an external Windows consumer, preserve840's stack configuration explicitly: `$env:RUST_MIN_STACK='16777216'` for test threads and `$env:RUSTFLAGS='-Clink-arg=/STACK:33554432'` for the executable main stack before build/test. The producer inherits the equivalent32MiB link flag from the repository config; an exported consumer has no parent repository config. The first consumer attempt without these settings overflowed its test stack and is retained as an environment failure, not an acceptance result. Preserve all original63 PLAN840 producer+independent-consumer invocations from user-behavior-handoff.md and its linked scene-play/hierarchy/scene-assets handoffs: locked complete driver test/build, complete AGP tests/build and all16 AGP native cases, original dual games and property/interaction/partial/reopen/owners, viewport/hierarchy/assets/play positives and sabotage, counter AND actual composed-content gates. Earlier passing work checks never replace this matrix. SDK scanner support/original source freeze history remains included; do not try to run verify_plan840 against intentionally modified845 source. verify_plan845 --controls is the current extended identity gate and retains prior receipt identities as provenance.

Add on BOTH producer and a fresh package_consumer own-SDK/own-target export:

```
python tools/verify_source.py
python tools/verify_plan845.py --controls
cargo check --locked --all-targets
cargo fmt --package autographics-auto-driver -- --check
cargo test --locked                 # complete driver, MERGE ONLY
cargo build --locked               # complete driver, MERGE ONLY
python tools/check_behavior_project_owners.py <fresh-dir>
python tools/check_behavior_project.py <fresh-dir> --cases bind-move,bind-patrol,update-fast,move,patrol,fast,partial,transparent,missing-part,wrong-asset,override,missing-source,bad-source,bad-actor,missing-glb
```

All15 project native cases are mandatory. Seven positives finish27phases/six all-part pixel stages; eight negative cases must fail their named oracle, partial is not success. Bind cases use actual OS Bind/Undo/Redo/Save As; update-fast changes user bytes only. Relocation uses no --behavior/original path, different cwd/new process, identical runner SHA. Explicit override preserves the project binding; inspect/new default reopening remains bound. Owner script has retain-source/retain-VM/reupload negative controls and restored positive, checking actual weak/Arc identities. Consumer must export only frozen inputs, rebuild its own SDK/target, retain original matrices plus new gates and verify source/lock drift controls.

No root Rust/schema generator changes: root t/tv/tt/tb/tu/taa/docs_gen are not triggered by this driver-only Plan. tf remains main-checkout single-instance batch, modulo845 due at successful landing; follow auto-plan:regress and record due receipt, not inside845 tree. Source only lands autographics-dev; canonical SD01..04/eight modified targets+new behavior-project, overview/goals/plans/index and derived ledger follow merge skill. Guard clean required before removing this tree. No exe installer, module packages, production I2/I3 or general engine guarantee.

## Known validation limits

SDK932's pinned compiler emits146 preexisting warnings; SDK source is immutable and outside this Plan, no new driver diagnostics accepted. cargo fmt --all encounters the pinned nested-workspace boundary; package-scoped fmt is the correct driver-only formatting check. Local Windows same-volume single cooperating writer only; compile/native/GPU calls are synchronous and not hard-preemptible. User private heaps/driver residency remain outside payload/GPU geometry measurements as in840. Reports retain initial fixture/counter failures; setup failures do not establish runtime failure coverage.

## F-845-R1-01 targeted repair

`behavior_project_host` filter `bound_source_origin` has two permanent lifecycle
cases: actual SourceSnapshot/Last Run/runtime callback error provenance, first
binding, unsaved update, geometry dirtiness, Undo/Redo, save, Open, missing original
and later removed blob (accepted bytes still execute), failed/pending Save As,
old-revision verification and root-change isolation. The original R1 external
counterexample remains unchanged and replayable from the main Plan attachments.
Old-origin controlled mutation fails both cases at the source-path assertion.
Storage whole-path fault recovery and reserved/old-revision receipt cases are the
only added legacy checks because load/verify update provenance. No SDK, locks,
source bytes, callback/API or GPU/owner lifetime mechanism changed. Unrelated R1
and work evidence remains applicable; complete campaign remains merge-owned.


Native Step regression (F-845-M1-04): `python tools/check_play_step_ack.py <fresh-output>` runs normal/delayed actual OS Step positives plus missing-click deadline and duplicate-tick negatives. These are probe-only fault controls, never Host-command replacements. Native authoring probes place the owned window within the desktop work area and reject clamped/foreign-hit pointer positions; normal application windows retain their placement. A posted click is asynchronous: phase6 awaits exactly one accepted Auto tick under the unchanged360-frame deadline, never retries the click or resets the budget. Original state/paused-tick/Stop/GPU/owner assertions remain. Merge includes this four-case regression for both producer and independent consumer in addition to the original81 invocations (83 total); work/review remain scoped.

Legacy native window regression (F-845-M1-05): the original --window-probe now uses the same verified OS click/scroll path and propagates every input error with its phase. Setup waits for window focus; no runtime focus/cancel semantics change. Actual End and Unicode character events edit the original field. The permanent authoring_probe_exit::legacy_scroll_errors_fail_before_global_timeout case checks a real clipped-field scroll failure before the unchanged1400-frame deadline; the full legacy positive still checks target/pixels, Save/Undo/Redo and resize. Merge retains83 invocations; full driver suites include this additional regression.

Native setup order regression (F-845-M1-06): hierarchy/assets prepare focus and the physical pointer before any automatic scroll, including a clipped first widget. Two permanent authoring_probe_exit tests run both actual accepted sequences with delayed setup and assert every mouse input follows preparation; over-budget setup still fails under the original240-frame phase deadline. Only initial readiness moved; later blur/cancel, pointer-owner guards and original pixel/acceptance checks remain. Complete driver count158; merge matrix stays83.
