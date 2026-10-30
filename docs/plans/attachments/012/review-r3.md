# PLAN-012 r1 affected independent final R3

**Verdict: pass**, revision-bound to `7e9802bb55961bf1d22be77892e568f15c9437bf`.
Review scope: final / affected re-review. Reviewer: independent authorized read-only subagent `/root/plan012_final_review`, same context/tree as R1/R2; no additional agent/chat/tree, implementation-source copy, source edit, Plan edit or canonical publication. Root owns shared receipt/status changes.

## Frozen baseline

- Same registered authoritative worktree `D:/autostack/.wt/ui-012/auto-ui`, branch `codex/plan-012-dev`, current HEAD7e9802bb55961bf1d22be77892e568f15c9437bf; repair base67ddbe2e61693773900af9630ef455d636d55108. Entry and completion status clean; no code drift.
- Shared main Plan read at `D:/autostack/auto-ui/docs/plans/012-autographics-public-audio-and-accepted-cues.md`, revision1, execution_done/current_step6. Read actual `r2-diagnosis.md` and `r2-repair-closure.md`; retained both prior failed final reviews and both bounded repair cycles.
- SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 and006deliveryc681663e6ff392d70471abbec560e9d9daf12622 remain pinned. Both Cargo.lock files unchanged. Current complete identity1536files SHA640126305f058592ea02423c8888cefd6a8c26f68e99a09d3f4d90f02834dd6f.
- No new applicable repository instructions appeared. auto-plan-review and verification-contract instructions retained from R1. Implementation writes/builds by root paused during review. Reviewer tests used only driver package, external target-driver, locked/offline and one test thread; no root/workspace/fullperformance/unifiedgate run.

## Findings closed

### F012-R1-01 — closed, full reviewed invariant

Original rule: live/required native manager, decoded assets and runtime pose share64MiB CPU; predecode admission must preserve manager ownership through every resource consumer. Existing old-profile caps remain intact.

R1original examples closed in first repair: retained healthy manager after New/Open is independently charged, empty-cue Run preflights before native opening, first audio replaces conservative service reserve instead of double charging, legitimate duplicates/near-bound assets and manager-free64MiB remain legal. R2 found the standalone textured/animated/load forecast lost Registry context, allowing2decodes before final refusal. This repair closes that remaining cause.

Actual implementation diff inspected: only13lines in production `src/authoring/textured_models.rs`. `Usage` no longer derives/implements Default. Its only explicit initializer is `from_registry(&Registry)`, which initializes all5maps/sets and bothserviceflags explicitly, carries `audio_manager_required`, then adds every existing asset. The complete closure forecast now uses that initializer before adding candidate root/dependencies/images/geometry/animation totals and before actual animation/PNG/geometry allocation.

All construction/consumer sites audited, rather than only the known line:

1. `check_union` validates retained/current/candidate Registry with service context.
2. `candidate_budget` covers flatGLB and standalone/sharedPNG prediction.
3. `candidate_audio_budget` covers first/subsequent audio; conservative asset service reserve substitutes for uncovered manager.
4. `uncovered_audio_service_cpu` reports the same owner rule.
5. `usage` reports consistent service-inclusive CPU totals.
6. Complete `decode` covers textured import, animated import and declared closure load.

There is no remaining Usage::default, alternate Default implementation or independent literal initializer. The only `Self` initialization fills every field and requires Registry. Actual successful package compilation confirms all current callers use the guarded construction. Registry clone carries context; Host reconstructs actual ownership after current/history/candidate/pending union instead of trusting stale views. Run/restart separately combine asset+requiredservice+pose before native opening. Raw identity/count/disk semantics remain separate; no fake resource or global free cache was added.

**Independent targeted runtime replay:**

`cargo test --locked --offline --test public_audio_host retained_service_is_in_every_model_closure_predecode_forecast -- --test-threads=1 --nocapture`

Exit0,1passed/11filtered, log `closure-predecode.log`. Read the actual test assertions. It uses exact R2tunedPNG bytes and original models/R1PNG inputs, real normal bound-project Run→Pause→Stop→New and stamped public commands. Six actual scenarios discriminate the repaired temporal gate:

| path | manager-free accepted control | live-manager over-cap counterexample |
|---|---|---|
| ordinary textured platform import | decode_delta2, accepted/published | named refusal, decode_delta0, unchanged document/no pending |
| ordinary animated character import | decode_delta2, accepted/published | named refusal, decode_delta0, unchanged document/no pending |
| normal Open→Discard→Storage/load_declared | decode_delta2, accepted/published | named refusal, decode_delta0, unchanged document/no pending |

Actual output confirms all six, not just a final test count. The unchanged candidate-only sums fit the cap and candidate+33528service exceed it, so the controls reject a plausible unconditional refusal or late-only repair. Generated saved scene hash is exactly R2input2fbf5eb26981188f62c8aea7af096e9ac37862345398567109acf7bf8d3cb0aa. Independent byte-hash comparison confirms both permanent tunedPNG inputs equal R2raw replay files.

Original budget2case and combinedpose1case current work logs were hashed/read and inspected, not globally rerun: PNGpredecode refusal/New/Open/near-bound/duplicate/firstaudio/emptycue plus real asset+service+pose and manager-freelegacy controls retain their already-verified mechanisms. This repair changes only construction of the previously missed closure forecast; no other service arithmetic, Host owner, decoder, VM or lifetime path changed. Existing resource9/start-restart2/postfault1 evidence remains applicable to unchanged affected mechanisms. No remaining full-invariant gap found.

Two-failure requirement satisfied: work diagnosis correctly identifies the missed separate constructor and why normal pre-manager samples did not distinguish it; the design adjustment removes the unsafe default entry and expands oracles across the shared consumer matrix. Code and actual assertions implement that adjustment; this pass is not merely another replay of the first example.

### F012-R1-02 — remains closed

Compatibility capture still emits optional snapshots then executes unchanged old15golden assertion before the completed-comparison marker. Gate still scrubs inherited capture flag and validates that marker/rejects old capture-only proof. This implementation and old baseline were not changed by R3repair.

Independently replayed current validator predicate against actual durable logs: new normal and capture-verified logs accepted; actual R1capture-only and actual wrong-goldenWITHcapturefailed logs rejected. R2 independently executed capture+verification1passed; its evidence remains valid on byte-identical code/golden. No source-mutating control was rerun. Gate change in this repair is solely Hostrunner12count (complete gate45cases).

## Complete AC/V/S map

| criterion | R3 result | updated and retained evidence |
|---|---|---|
| AC01 / T01,02,06 / V01,S01,V05,S05 | pass |006actualdelivery/SDK/locks, explicit profiles, fixed caps/prices/input order unchanged. Source/dependency guards current. Full sharedCPU/preallocation invariant now closed as above, including legitimate legacy64MiB. |
| AC02 / T02,03 / V01,S01 | pass | Scene7/cue exactmetadata/hash/version/legacy rejection, raw8/32MiB/disk/persistence/safeancestor/pending/history/sharedPCM/repaircoldreopen and boundedcodec evidence retained from R1actualcode/assertions. Current Registry and all6forecasts audited; R3actualstatic/animated/load counterexamples now refuse before decode. |
| AC03 / T02,04 / V02,S02,V03,S03 | pass | Exactproducer/independentHostwire, canonical unusedrows, largeordinal/exhaustion/replay/4-5requests/cue/gain/span, commonaccept ordering, once-onlyirreversiblePCM, nativepostmappingwaveform,32-33completeevent composition and render/multimount behavior unchanged since R1. Full bound evidence retained; current movedfunctionalconsumer verifies both12645frame source-sensitive PCM/gain/motion paths. |
| AC04 / T05 / V03,S03,V04,S04 | pass |8-9capacity/last-good/pre-postacceptfault/Pausecursor/Stepordinal/no backlog/Stop-nextquantum/newRun/40Runactualweakowners retained on unchanged state/mixer/native paths. Firstrepair Run/restart/combinedpose checks retained; manager lifetime accounted independent of audioasset and now through every predecode consumer. |
| AC05 / T06 / V05,S05 | pass | Current1536identity/SDK/locks and movedfunctionalconsumer source/binary/log/PE binding verified. Old15golden stays unchanged with F02closed. Original fullsource/CPU/resource/performance tests remain historical evidence for unchanged mechanisms; new completeperformance/freshproof explicitly deferred to merge-owned45casegate. No historical proof relabeled currentfullgate. |
| AC06 / T07 | pass | Independent fullR1inspection plus affectedR2/R3review covers completeAC/V/S/SDmatrix; both stablefindings closed and no unresolved in-contract finding/decision. |
| AC07 / T08 | pending merge | Frozen45case `check_public_audio.py --controls` on this exact currentsource/binary baseline, completecurrentperformance/freshproof; must pass before landing. |
| AC08 / T09,10 | pending merge | CanonicalSD/ledger/index/delivery/archive and guardedcleanup/recovery receipts. |

Unchanged-path rationale: diff67ddbe2→7e9802 changes no codec/mixer/native/AutoAPI/assembly/Playwire or state machine/GPUvisual/projectscript/bank/provenance implementation. Their R1read actual assertions, PCM oracles/native observations/40Runweakowners and negativecontrols remain applicable. Full construction audit and currentclosure/runtimecontrols specifically cover the changed resource mechanism, avoiding unsupported blanket reuse. Full performance is not rerun as a repair prerequisite or claimed currentfinalgate.

## Source, dependency and moved functional consumer

Producer and movedB `python tools/verify_source.py` independently exit0:1536completeinputs, SDK681,17localpackages/29pathedges/479metadata, frozen AGPlock retained. Producer package `cargo fmt --check` and `git diff --check` exit0. Known SDK146warnings retained; no new deliverywarning observed. Source/status/HEAD rechecked clean at verdict.

Current work consumer proof: `r2-functional-proof.json`, schema autographics.plan012.work-functional/1, explicitly functional-only1passed/1filtered. Independently verified actual log count, currentproducer/Bidentity byte equality, Aabsent, Bmanifestdir, binarySHA73241006268eaa8a6ceaa7f61f64e20913e5a6cc7a23b18776b4d5b89f942828, PE main33554432 and worker16777216configuration. Bothconsumerproject entries assert move/coldreopen/event-selectedcue/source-sensitivegainPCM+motion/licenses and12645independentPCMframes. Bsource/SDKguard rerun directly, no sourcecopy/build created by reviewer. Source-only A→B/noSDK-noTarget then ownSDK/offlinecompile receipts were inspected; no old fullperformance manifest passed off as this proof.

Functionalproof SHA: c909bd9556ebcbb9bbbdcfb041ad7070b4e3b80148259e985000f472185b2b67. Actualconsumerlog SHA9c1db9226daf9d4537e57381e973a0eedde59c6e1e795f2cb3629dfd32623408;1passed/1filtered. Final full45casegate is explicitly still pending, including new complete currentperformance/freshconsumer proof. Previous 8ms3Dpump targetmiss and hardware/CPALunderrun limits remain honestly reported historical observations, not waived constraints or newly fabricated measurements.

## SD review

Proposed deltaSHA634542205cffdb4808e2e7cb63f20aa5d470c4525d93a7185ecac90d18c17711. Canonicalsource hashes recorded below; no canonicalpublication.

- SD01 add docs/specs/auto-graphics/public-audio.md: finitecodec/profile/cue/accept/native/once-only/lifecycle and hardwarelimitations verified in R1, unchanged; pass.
- SD02 modify docs/specs/auto-graphics/scene-assets.md: scene7portableclosure/sharedoldquotas plus required/liveHostservice/combinedpose/all-entrypreallocation now match actual code, including static/animated/declaredload. This is the original rule made explicit, not scope/cap revision; pass.
- SD03 modify docs/specs/auto-graphics/behavior-api.md: exactprofiles/input/prices/bundle/32credit/old15compatibility rules remain accurate; proofintegrity fixed; pass.
- Exact supersedes/newpaths and GOAL008 remain correct. Downstream ag-examples005/G12a must consume actualdelivery and verify content behavior with013font/HUD; no completegame/GalleryZIP/011/fullAGstatus claim. Boundary retained/pass.

## Route and pending closeout

Root may record finalreview pass / reviewed for this exactrevision. Both prior failures remain historical. No required work-owned criterion remains open; no implementation writes or broader tests needed before invoking the designated merge-owned gate. Landing remains conditional on frozen45case directed gate and valid completecurrentproof, then SD/ledger/index/delivery/archive/guardcleanup. Any code/dependency/verification-input change invalidates affected review bindings and requires scoped re-review. No sourcecontrols/generator/formatwrites/fullworkspace/root/performance/45casegate executed by reviewer.

## Exact SHA256 inputs/evidence

- `D:\autostack\auto-lang\docs\specs\auto-graphics\scene-assets.md`: `b5362e41bb90ec6c9414792888a052ea566035f345d71732d8066a842f0b0a14`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\behavior-api.md`: `3a1ead71ea2b21f14c199915c7043a12d55dae0642174815c73f39abcd8ec1bc`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\overview.md`: `66d90c6c5947cfa54c6daaa828b679bb654797ab9458eb9a2ee81b4ed2361ef3`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\auto-source-driver.md`: `f0555a887362c92af8520af13443fb0ba61f28b7070677556c95bb06bf6d79d1`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\runtime-triggers-and-rounds.md`: `1d0e37f3a641d33fd6f4408bbcf039f28cc41b90c6765127bd04ad8585eb2e44`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\node-animation-and-facing.md`: `42db8e3774d7bf6dba1d0ee8dda7ad2590a003ed67af79fd2d52042364933822`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\public-audio-spec-delta.md`: `634542205cffdb4808e2e7cb63f20aa5d470c4525d93a7185ecac90d18c17711`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\plan012-identity.json`: `640126305f058592ea02423c8888cefd6a8c26f68e99a09d3f4d90f02834dd6f`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\plan012-legacy-profiles.json`: `e5312820e03c6f3aa0a5af15a240d42c6dfb70eb5b7c7f21a4bf092169d54158`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\Cargo.lock`: `4a0b3040fffa2523f3932f2ab2b427e3c13285abce751813147cfe28000d9b06`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-prototype\Cargo.lock`: `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r2-functional-proof.json`: `c909bd9556ebcbb9bbbdcfb041ad7070b4e3b80148259e985000f472185b2b67`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r2-functional-consumer.log`: `9c1db9226daf9d4537e57381e973a0eedde59c6e1e795f2cb3629dfd32623408`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r2-original-budget.log`: `018be0ac014ef004a9767591ccc87c4d207af65162a00a6b7425885ac590f90b`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r2-pose-service.log`: `9d67467ceb203c4299c8c220428e5c96e1417593c07f04b32f2dbb9ed704e696`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r2-identity-controls.log`: `dc436331c34cc62067605389b85a36fba59c75ff0062facf1f5db20e764dbc5f`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\animated-boundary.png`: `e0af3a2769b95bef1d090eb425369f2d79127c6ef700c83647c99559c819968f`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\budget0.png`: `6a6515adc7bc31b5043f660e1a805746f9381068ff2acd7b9637d7620f0b0b66`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\budget1.png`: `45717ba6bb85dded4c61b955fbb2362078fc86db4f37482af6170e0f23705827`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\budget2.png`: `dca0132f0add1dea263fa9c7f6f090d2f2e2efbc263a76c2e765962dfbc5427e`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\budget3.png`: `1ebb90a4a4b5bc1f61632fa354bf04dc3d7e932656e3b087c6cdd291a1ae6275`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\tests\fixtures\public_audio_budget\textured-boundary.png`: `c6b5ec65c08c647084e172ea0eaac85dfeb07a98d7455e42bf286f79a4b0f1fb`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r3\closure-predecode.log`: `ee1ada13dd937b667e505c50577edab3a9b6a687b40c4e97865e91eb5952d46e`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r3\fresh-functional-source.log`: `cb74a758645a1baf7c0f5fb968b10e07877a9b3c010dd47acd80b84c14e78f84`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r3\source.log`: `cb74a758645a1baf7c0f5fb968b10e07877a9b3c010dd47acd80b84c14e78f84`
