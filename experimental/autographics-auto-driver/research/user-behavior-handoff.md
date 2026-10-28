# PLAN-840 r2 work handoff

Authoritative source: plan-840-dev in the lang-840 worktree. Parent formal delivery
is PLAN-839 a239653365a0c3451c85aa9921189fcf011bd8cf. Frozen SDK artifact is
932ad4877e5968f1aa7227d9dcd7fb69b4c69a21, old pin plus exactly the scanner patch.
SDK681 files /479 dependency packages /17 locals /29 edges; original two locks
and AGP source are retained. Canonical scanner fix is separately committed so
merge can land that SDK-only change on master while graphics stays autographics-dev.
The artifact ref is provenance, not evidence of final reviewed SDK delivery.

## Public contract and implementation

See ../api/README.md for the exact public signatures and units, and
behavior-spec-proposal.md for proposed SD01..05. User source remains byte-exact:
strict UTF-8 no BOM, one regular .at, max256KiB; trusted bridge max64KiB and
combined max320KiB. Parser AST rejects duplicate/reserved roots, wrong callbacks,
imports/foreign declarations/top execution. Real Core helpers/state/control flow,
comments and strings remain supported. Trusted transport alone is privately
namespaced. Typed SDK span diagnostics map into named source regions; no-span
errors say so. Runtime/SDK diagnostic text is bounded to8KiB. This is trusted local
code, not an untrusted sandbox or arbitrary VM heap/native-allocation hard quota.

Profile7 calls app_start once after complete graph/actor configuration and
app_update once per tick. Public handle0 is valid; -1 is absent. API reads derive
world/effective state after writes. First API error sticks, rejects the entire
callback frame and closes the VM, preserving last-good. New Run resets globals.
Source metadata includes raw/API/bridge/combined hashes, source ranges, path,
version and base scene/assets/actors/stamp. Host uses actual Storage scene root;
selection changes only while Editing. Missing/bad source never falls back.
Scene Save/Save As does not write source or persist selection. Builtin6 and
profiles1..5 retain their original routes and complete merge denominators.

## Scoped work checks

Run from this driver with locked dependencies; set RUST_MIN_STACK=16777216 and
Windows executable stack32MiB when SDK stack needs it. No root full tier belongs
to this work/review stage.

```powershell
python tools/verify_source.py
python tools/verify_plan840.py --controls
cargo check --locked --all-targets
cargo fmt -- --check
cargo test --locked --test user_behavior_feasibility
cargo test --locked --test authoring_behavior --test authoring_behavior_host --test authoring_play --test authoring_play_host
cargo test --locked --lib user_profile_rejects_actual_wrong_profile_stale_run_and_resource_output
cargo test --locked --bin autographics-authoring
python tools/check_play_owners.py <new-output-directory> --user
python tools/check_user_behavior.py <new-output-directory>
python tools/package_consumer.py <new-external-directory>
```

The native aggregate has13 required cases: Move, Patrol, speed-only90 variant,
relocated project/different cwd, missing script, missing callback, real API fault,
actual callback CPU budget, frames1 partial, transparent/missing-part/wrong-asset
GPU faults, and wrong canvas focus. Four positives complete27 phases each, with
six actual pixel records and independent endpoint/origin/all-part checks. Same
runner SHA and actual source hash are recorded. Negative exit1 is accepted only
at the required specific failing oracle. Partial/zero-case/skip cannot pass.
Real focus loss auto-pauses; preserve failed attempts and rerun with quiet desktop.

User tests cover raw bounds/metadata tampering, correct source spans after multiline
comments, globals/callbacks/handle0/read-after-write, sticky first errors,128nodes,
600tick independent trace, fault/Stop/reload, actual Editor/GPU restoration and
20sourceA/B owner cycles. Owner controls retain a real live Play VM or deliberately
replace uploaded GPU meshes; each must fail its owner/Arc assertion, then the
unmodified control must pass. Consumer corruption test deliberately bypasses
source admission in an internal test to feed actual wrong profile/run/resource
VM output into the real profile7 consumer. Production source admission rejects
forged metadata before VM construction.

## Independent review and merge-owned gates: pending

Review must independently verify every AC01..08 and proposed SD01..05 against the
committed revision, including dropped items/workarounds/warnings/fmt. Work receipts
are claims until checked. Full gates execute only after independent pass, frozen
code/dependencies/config/input; code changes require affected re-review/re-freeze.

Retain the ENTIRE exact matrix in scene-play-handoff.md, hierarchy-handoff.md and
scene-assets-handoff.md: locked full driver tests/builds; all16AGP cases; old two
games, authoring window/interaction, assets/hierarchy/scene-play native positives,
real GPU sabotage, input sabotage/partial/reopen/owner, original counter AND actual
composed-content oracles; independent source consumer. Replace receipt directories
with fresh840 directories. AGP sabotage processes remain separate and restored.
Then add verify_plan840 --controls, the full13case check_user_behavior aggregate,
check_play_owners --user, and full new source/API/Host tests. Export a fresh frozen
consumer with package_consumer; use its own target and SDK, repeat identity controls,
locked check/tests, old impacted native gates plus user native aggregate/owner
controls. No old denominator is replaced by a new one.

Only after those gates: source lands on autographics-dev, scanner-only patch can
land independently on master, canonical Specs/overview/plans/goals and ledger
receive the approved delta, Plan is archived, guarded clean tree cleanup and
batch-due receipt are performed. No merge, canonical publication, archive or tree
cleanup is authorized by a work-stage completion alone.
