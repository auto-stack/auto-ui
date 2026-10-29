# PLAN-005 r3 independent affected final review R2

**Outcome: needs_fix.** F-005-R1-01 is closed. F-005-R1-02's original warm-Open example is closed, but the same nonbinding-display-name invariant still fails at the next Save and Play boundaries. This is the second consecutive failed final review; work must complete the diagnosis/coverage adjustment before the next readiness handoff.

## Bound review baseline

- stage review; review_scope final; reviewer `/root/ui005_final_review`, independent of implementation; actual model/session identity not exposed.
- Authoritative main Plan: `D:/autostack/auto-ui/docs/plans/005-autographics-textured-models-and-uv.md`, r3, execution_done,6/10 at entry. Previous authoritative R1: `D:/autostack/plan-artifacts/auto-ui/plan-005/review-r1.md`.
- Same existing implementation tree `D:/autostack/.wt/ui-005/auto-ui`, branch `codex/plan-005-dev`; reviewed frozen commit **8809e545830a2d0ed6f587517e6d4d8a29a575ca**; repair base **9d680f9574275d13a0f776ee321534f5a9d4c465**; original diff base **2b88bd69b2a48172bf761615cc4e78a1df563b62**. No new checkout or implementation/Plan/Spec/ledger write.
- The complete four-file repair diff was independently inspected: `textured_models.rs`, its integration tests, checker count18→20 and regenerated source identity. SDK, locks, AGP/shaders, source policy, document/session/storage/runtime implementation and proposed SD are byte-unchanged from R1. Original-reference validation now precedes scratch; unsupported-channel checks use all original materials before dependency IO, without moving any typed geometry reader before common span/type admission.
- Current source identity SHA256 **19d19280dc2d908cd1e984603c97d2e00b1afbf95c25d3aa03771c7fb9e61e7f**. Producer verifier exit0:1456 hashes, frozenSDK681/17local packages,479packages/29path edges, forbidden0. Dependency locks/logical SDK pin and canonical Spec hashes are those bound in R1; no dependency or support revision occurred.
- SD draft still SHA256 **14b5aa1a6a03751c1e7ffea3df07051c33e8b9aea3540de315795e3c81cf8a9a**. Canonical paths/GOAL-008/SD-01 add and SD-02 modify remain valid; no semantic Plan revision is required to enforce the existing rule.
- Final recheck: HEAD8809e545… unchanged, status porcelain empty. Implementation writes were paused throughout.

## Independent affected checks and evidence

Evidence directory: `D:/autostack/plan-artifacts/auto-ui/plan-005/review-r2/`. R1 original red logs were preserved. External probe sources were copied from R1, redirected to this directory, compiled against the current producer library, then extended only to observe the next actual user operations. Exact reusable sources/build helper and library hash are retained.

| Check | Result |
|---|---|
| `build_review_probes.ps1` from review-r2 | exit0; compiles current-library admission and cache probes without touching source packages. |
| `python review-r2/admission_holdouts.py --expect-contract` | exit0,21cases including5 accepted controls. All four unsupported channels, used/unused, now produce named unsupported errors before missing-PNG IO/decode. Original, missing-default-scene, unused valid sampler and normalized U8/U16 UV controls remain accepted. Other pre-existing span/tangent/UV/extension/overflow negatives remain rejected with decoder delta0. `admission-current.log`, raw holdouts and manifest retained. |
| Original name-only warm-Open replay | Save Confirmed; cold Open accepted; different-name warm Open accepted; same-name warm Open accepted. Original R1 sample is closed. |
| Extended actual warm-Open→Save/SaveAs/Play replay | **fails**, see finding below. Cold Save+Play and same-name Save controls pass. `name-cache-lifecycle-red.log` retains actual values and assertion failure. |
| Producer `python tools/verify_source.py` | exit0; `current-source.log`. |
| From current `consumer-repair-r1/experimental/autographics-auto-driver`: `python tools/check_textured_models.py --validate <repair-r1-consumer-gate> --controls` | exit0; exact current source/runner/log/stack/report binding and report/actual zero-selection controls validated; `consumer-current-validation.log`. |

Current fresh snapshot-only consumer manifest SHA256 **d8005629f13b51791fc37ad62c66254f2f4c0146212a1cf84cac4391ab93becc**, identity19d19280…,20driver+4AGP,12semantic reports, no failures/ignored/filtered; two exact executable hashes and PE32MiB/worker16MiB profiles checked. Cold SDK proof and raw runner assertions were inspected. This is valid current-source evidence but its tests do not cover warm alias Open followed by Save/Play, so its green banner cannot close that invariant.

No full frozen gate was repeated, and no full AG/root/auto-lang tier or OS automation ran. R1 numerical GPU/owner/transaction and compatibility evidence remains applicable for unchanged ordinary inputs/configuration/dependencies; the newly identified warm-state exception is recorded explicitly below rather than claiming all runtime behavior passes.

## F-005-R1-01 — closed

The repair validates original glTF references before resource removal and explicitly classifies normalTexture/occlusionTexture/emissiveTexture/metallicRoughnessTexture for all materials. Permanent test assertions include8valid unsupported channels,8genuinely invalid references, absent PNG/decode0 and a valid original control. Independent original21case replay passes. Invalid index references remain `invalid`, while valid unsupported declarations are named `unsupported: material <channel>`. No geometry reading or IO admission was weakened. Old-example and full declared-channel invariant closure are established.

## F-005-R1-02 — P2 — partial repair; warm Open still cannot Save or Play

**Type:** incomplete invariant closure, newly reachable downstream states after the original Open repair. Same stable finding ID; this is not a new support promise.

**Existing rule:** canonical scene-assets.md says display names do not bind resources. PLAN005's finite closure adds root/image identity while preserving cached content reuse, editing/save and Play. A project admitted with unchanged content under another display name must remain usable at subsequent legitimate boundaries.

**Setup/entry:** `review-r2/name_cache_probe.rs` imports exact original platform/image under platform.glb and alias.glb, asserts equal closure key and different names, creates a visible Model leaf, saves the alias scene, and loads it with the original-name Asset cached. Editor admission and actual Host::open+settle pass, retain the alias document name and reuse the old immutable Arc. The probe then exercises actual Storage Save/current and SaveAs, actual Play::new, plus actual Host::start_play and Host's Storage save path. Inputs persist under `review-r2/name-cache/run-<pid>/`.

**Expected:** all these operations accept identical root SHA/length/format/dimensions/closure regardless of display name; no blob/edge validation relaxation. **Observed:**

```text
save=Ok(Confirmed)
cold_open=Ok("accepted")
warm_open=Ok("accepted")
save_after_warm_open=Err("snapshot asset identity")
cold_save_control=Ok(Confirmed)
cold_play_control=Ok("accepted")
warm_play=Err("play asset identity")
same_name_save_control=Ok(Confirmed)
warm_save_current=Err("snapshot asset identity")
same_name_warm_open=Ok("accepted")
host_warm_play=Err("play asset identity")
host_warm_save=Err("snapshot asset identity")
```

Assertion-enabled command `review-r2/name_cache_probe.exe --expect-lifecycle` exits101; raw evidence `name-cache-lifecycle-red.log`. Original `--expect-contract` checks only Open and passes, which demonstrates why closing the example alone was insufficient. Build helper must be rerun after rebuilding repaired current source; otherwise an old linked executable is only a historical replay.

**Established cause/caller inventory:**

| Path | Current behavior / boundary |
|---|---|
| textured_models.rs load_declared | Repaired comparison excludes name, retains SHA/length/format/width/height/closure; original warm Open now succeeds. |
| storage.rs:268 save_snapshot | Still compares `loaded.identity != *a`, including name. Rejects both Save/current and SaveAs before blob preparation; Snapshot::capture retains the cached Arc and the document's independent name. |
| play.rs:501 new_program | Same full AssetRef comparison rejects the actual generic/Host Play path before VM initialization. |
| play.rs:429 sprite target admission | Another full AssetRef equality in the sprites-only conditional. This 2D sibling is inspected, not independently executed by this 3D counterexample; generic line501 currently rejects the mismatch first. Preserve its content checks in a shared invariant audit. |
| session.rs validate_assets; assets import/insert/load | Session compares binding fields excluding name; registry insertion/reuse is key/content based; both permit the valid state. |
| storage pending Verify | Verifies exact snapshot bytes and root/dependency blobs, not another full AssetRef equality. The failing warm save cannot reach this state because earlier snapshot admission rejects it. Pending-save corruption/recovery behavior is unchanged and retains R1 evidence. |
| close/reopen | Close releases cache owners; cold reopen/control accepts. No separate close lifecycle rerun was needed; the observed defect depends on a legal retained cached Arc, not a leak. |

**Minimal closing oracle:** independently replay the original alias Open and then actual Host Save/current, SaveAs/readback/reopen, Play/Stop; retain document display name and actual immutable Arc reuse. Check all full AssetRef comparisons against the same binding semantics, including the sprite conditional where applicable. Preserve corrupt/missing dependency rejection+restore, wrong rehashed edge refusal, and same-root/different-image separation. Add permanent assertions covering the next operations rather than merely Host Open. The name repair must not mutate immutable payload metadata to force whole-record equality or skip disk/closure revalidation.

**Mandatory diagnosis after two failed finals:** the first repair changed one callee comparison and its test stopped at Open+settle/corrupt-restore, leaving two downstream public admission boundaries outside the closure oracle. Work should document the complete binding-comparison design/caller inventory and update the closing oracle before next execution_done. This is a targeted admission/storage/runtime check; unrelated GPU stress/global gates are not its repair loop.

## Acceptance / Spec / routing

| IDs | R2 status |
|---|---|
| AC-01 / V-01 / T-03 material branch | pass; F-005-R1-01 closed, other admission checks preserved. |
| AC-02 / V-02 / T-03 | **fail**: warm alias Open succeeds but Save/current and SaveAs fail. |
| AC-03 / V-03 / T-04 | pass; unchanged rendering/UV/source inputs and R1/fresh-consumer evidence reused. |
| AC-04 / V-04 / T-05 | existing transaction/owner/warm-upload invariants pass; **partial** for newly reachable alias warm state because actual Play is rejected. |
| AC-05 / V-05 / T-06 | source/fresh SDK/consumer/compatibility evidence pass on8809e54; **partial**, warm alias public workflow still incomplete and final identity must be rebound after repair. |
| AC-06 / T-07 | **fail / needs_fix**, second consecutive final failure. |
| SD-01/SD-02 | Approved proposed text remains aligned; implementation publication blocked by the existing display-name invariant. No semantic revision or Spec weakening required. |
| AC-07/08 / T-08..10 | pending merge-owned frozen producer scoped gate, landing, canonical Specs/ledger/index/archive/delivery and guarded cleanup. |

Return to work in the same tree; reopen T-03/T-05/T-06 and dependent readiness; preserve T-01/T-02/T-04 and applicable ordinary-state T-05 evidence with the warm-state exception. Root alone updates the authoritative Plan. After diagnosis and committed repair, re-review the affected boundary checks and new source/consumer binding, then leave the approved frozen producer gate to merge. Full AG milestone and 003's deferred BLOG-004 batch remain outside this review. No merge or canonical publication is authorized by this needs_fix verdict.

