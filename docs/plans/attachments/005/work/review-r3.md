# PLAN-005 r3 independent affected final review R3

**Verdict: pass.** Both stable findings F-005-R1-01 and F-005-R1-02 are closed. The original examples, the full affected identity invariant and the previously missed next operations are verified. No open in-contract finding or unexamined affected risk remains. Required merge-owned gates remain pending; this review does not itself land or publish anything.

## Baseline and independence

- stage: review; review_scope: final; reviewer/context: `/root/ui005_final_review`, independent of implementation. Model/session identifiers are not exposed and are not invented.
- Authoritative Plan: `D:/autostack/auto-ui/docs/plans/005-autographics-textured-models-and-uv.md`, **revision3**, execution_done6/10 at entry. Existing whole-goal authorization and explicit independent review requirement persist. Root is the sole shared Plan writer.
- Existing implementation tree **D:/autostack/.wt/ui-005/auto-ui**, branch **codex/plan-005-dev**, reviewed commit **74378791b353cd285168fa43fb7fe4b27cd208fe**, previous R2 **8809e545830a2d0ed6f587517e6d4d8a29a575ca**, original base **2b88bd69b2a48172bf761615cc4e78a1df563b62**. Worktree inventory/branch/HEAD verified; no new checkout or implementation edits.
- The complete eight-file repair diff was read, together with `attachments/005/work/repair-r2-diagnosis.md` and `repair-r2.md`. Diagnosis preceded implementation, identifies why Open-only coverage missed subsequent Save/Play, inventories all consumers and defines a shared content policy plus real lifecycle oracles. The adjustment was verified in code and execution, not accepted merely as prose.
- Final HEAD/cleanliness recheck: **74378791… unchanged**, `git status --porcelain` empty; `git diff --check` exit0. No competing implementation writer was active. GPU execution was serialized.

## Identity and specification binding

Current complete source identity SHA256 **134b4f731889a3980557037e3ea3bdfddb81196f858c3d98c8c04a10c26a0fbc**; producer `verify_source.py` passed1456 hashes, frozenSDK681files/17local packages,479packages/29path edges, forbidden0. SDK logical pin remains **7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467**. Driver Cargo.lock SHA256 **2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db**, AGP lock **c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf**, both unchanged.

All four canonical Spec input hashes bound in R1 were independently rechecked unchanged: scene-assets.md, sprites-and-image-resources.md, source-repository-and-sdk.md and m2-game-sample-research.md. Exact hashes and paths are retained in `review-r3/spec-input-recheck.json`; R1 retains full baseline/version/instruction fingerprints. The proposed SD remains SHA256 **14b5aa1a6a03751c1e7ffea3df07051c33e8b9aea3540de315795e3c81cf8a9a**, frozen at `review-r3/reviewed-spec-delta.md`. SD-01 add `docs/specs/auto-graphics/textured-model-resources.md` and SD-02 modify `docs/specs/auto-graphics/scene-assets.md` remain valid, additive to legacy rules, with actual GOAL-008. No semantic support/budget/AC revision or canonical publication occurred during review.

## Finding closure and invariant audit

| Finding | R3 result |
|---|---|
| F-005-R1-01, material-channel classification | **closed**. The earlier fix is unchanged; independent original21-input holdout replay again passes, including all four unsupported channels in used/unused materials and5 accepted controls. Invalid declarations still fail before decode; valid normalized U8/U16/default-scene/sampler controls remain accepted. |
| F-005-R1-02, nonbinding display name | **closed**, including full lifecycle. Shared `AssetRef::same_content` compares exactly SHA, length, format, width, height and full ordered closure; only name is excluded. Exact Eq/serialization/document names remain intact. All six admission sites use the helper: PNG load, textured load, session, snapshot save, generic Play and sprite target Play. No remaining full AssetRef equality exists at those boundaries. |

The repair does not mutate cached immutable names or remove disk/edge/blob validation. `Snapshot::capture`, serialized snapshot/hash/stamp admission, entry-last install, pending Verify/readback, resource budgets and geometry/URI admission are unchanged. Existing same-root/different-image closure distinctions, missing/corrupt cached dependency refusal, forged/rehashed wrong edge refusal and restore/re-read evidence remain applicable; the targeted cached dependency test reran successfully.

Actual extended permanent tests now settle initialization and assert non-Faulted state. The textured workflow exercises cached Host Open→Play/Stop→Save current→SaveAs with pending verification→corrupt PNG Verify refuses→restore Verify confirms→cached reopen→Play. Alias document names, original Arc reuse and exact raw GLB bytes are asserted. Shared caller changes also execute real legacy flat GLB and PNG Sprites2V1: generic and sprite-target admission initialize without Fault, then Stop/save/reopen. Wrong declared length is still rejected at actual snapshot and Play boundaries. The PNG setup satisfies the existing character/camera profile; no fault assertion was weakened to make a setup error green.

## Actual independent commands and reusable evidence

All new evidence is outside source closure under **D:/autostack/plan-artifacts/auto-ui/plan-005/review-r3/**. Original R1/R2 reds and reports remain preserved. Replays were copied to this directory, changing output locations only, and rebuilt against the current producer library; library fingerprint and build helper are retained.

| Command / actual path | Result |
|---|---|
| Producer driver cwd: `cargo test --locked --offline --test textured_model_resources cached_ -- --test-threads=1 --nocapture` | exit0, **3 passed/18 intentionally filtered**: textured full cached lifecycle; missing/corrupt/forged-edge recovery; legacy GLB/PNG shared-name full lifecycle. Log `../review-r3-targeted.log`. Current library rebuilt; inherited SDK146 warnings only. |
| `review-r3/build_review_probes.ps1` then `name_cache_probe.exe --expect-lifecycle` | exit0; original R2 full-state counterexample now reports **all12 operation lines Ok**, including warm/current/SaveAs/Play and actual Host Play/Save. Output checked for any `Err(`; none. `lifecycle-current.log`. This replay preserves the exact prior failing path; permanent tests above additionally settle/Stop and verify subsequent disk usability. |
| `python review-r3/admission_holdouts.py --expect-contract` | exit0, **21 inputs/5 accepted controls**; `admission-current.log`, input manifests/raw holdouts retained. |
| Producer driver: `python tools/verify_source.py` | exit0, current complete source/SDK/dependency closure; `source-current.log`. |
| Current consumer-repair-r2 driver: `python tools/check_textured_models.py --validate <repair-r2-consumer-gate> --controls` | exit0, current source, exact executables/logs/stacks,12reports and pixel/partial/runner/budget/warm/stale/actual-zero-selected controls; `consumer-validation.log`. |
| `git diff --check`, final HEAD/status; canonical Spec hash audit | exit0/unchanged clean reviewed SHA; all4 relevant canonical inputs unchanged from R1. |

Current fresh cold consumer source identity134b4f73… and manifest **01eaf90e08370b6ccfc81f6fb639a5cd77b27022686f86a4bb83b516495e2c25** are bound. Raw reports assert **21driver+4AGP**,12semantic report cases, zero failed/ignored/filtered, actual executable hashes and32MiB PE/16MiB worker profile. The checker count was changed strictly20→21 for the new permanent test; source policy/checker thresholds and SDK/locks were not loosened. Cold snapshot/independent targets and actual fixed-source public project workflow were inspected; prior physical A→B proof remains valid for unchanged export/runtime mechanics.

R1 numerical GPU sampling/transform/depth/double-sided shadow, resource/owner/transaction and compatibility evidence is reused on its unchanged code/input/config/dependency basis. R2/R3 change identity comparison in admission, not shader/sampler/mesh/texture owner mechanisms. Changed name-bearing Save/Play states received fresh direct execution above. Current full consumer additionally reran all required modules. No full producer frozen gate was duplicated during affected review; no full AG/root/auto-lang tier, OS automation or 003 batch gate ran.

## Complete acceptance map and next owner

| AC / V / task | Current result |
|---|---|
| AC-01 / V-01,S-01 / T-01..03 | **pass**: original identity/licenses, bounded all-declaration admission/union, URI/extension/type/span/UV/tangent controls; F01 remains closed. |
| AC-02 / V-02,S-02 / T-03 | **pass**: immutable full closure, schema6 compatibility fence, cached Open, current/save-as/pending Verify, corrupt refusal+restore, reopen and name independence; F02 closed. |
| AC-03 / V-03,S-03 / T-04 | **pass**: real textured pixels/UV/sampling/transform/depth/back-facing normal/shadow controls and retained006 associations; unchanged evidence reused with current consumer confirmation. |
| AC-04 / V-04,S-04 / T-05 | **pass**: staged last-good transaction/union/actual owners/warm uploads and20Runs retained; previously rejected cached-name Play path now initializes and Stops correctly, including sprites sibling. |
| AC-05 / V-05,S-05 / T-06 | **pass**: current source/SDK/runner/consumer closure, compatibility, public save/move/reopen, retained node/primitive metadata and actual legacy/sprite lifecycle. No complete game/PBR/animation/platform expansion is implied. |
| AC-06 / T-07 | **pass**: independent affected R3 closes both findings, checks mandatory diagnosis/design adjustment and binds all inherited/fresh evidence to r3/currentSHA. |
| SD-01/SD-02 | **pass**: proposed canonical delta matches implemented finite support/identity/owners/boundaries and unchanged canonical targets; publication belongs to merge. |
| AC-07 / T-08 | **pending merge-owned**: approved frozen producer scoped gate on reviewed/current source, with strict21driver+4AGP/12reports and applicable compatibility/fmt/source checks, then landing. |
| AC-08 / T-09,T-10 | **pending merge-owned**: canonical Spec/derived ledger/index, local Plan archive/deliverySHA and guarded cleanup/recovery receipt. |

Root may record final pass and transition the Plan to reviewed with T-07 complete (**7/10**). Merge must complete its remaining gates before delivery/cleanup. Full AG milestone and user-deferred003 batch remain pending in their assigned workflows. This reviewer changed no implementation, shared Plan, canonical Spec or live ledger and performed no merge/archive/cleanup.
