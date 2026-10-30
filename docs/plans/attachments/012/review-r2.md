# PLAN-012 r1 affected independent final R2

**Verdict: needs_fix**, bound to clean `67ddbe2e61693773900af9630ef455d636d55108` on `codex/plan-012-dev`, same authoritative worktree `D:/autostack/.wt/ui-012/auto-ui`. Repair base: R1 `87414431b9a1dbda6eb54aad999af7f734a14516`. Shared main Plan revision1 remains execution_done/current_step6, bookkeeping09211ad. Independent reviewer is the same authorized read-only subagent `/root/plan012_final_review`; no new agent/chat/tree/source copy or implementation edit. Root owns Plan receipt/task-state changes.

## Finding disposition

- **F012-R1-01: partial closure; remains open (P2)**. Original manager-retainedNew/Open, empty-cue native startup and combined service+pose over-cap cases now behave correctly. A sibling predecode construction still loses the service requirement for textured/animated GLB closures and load_declared. Real decoded allocations occur before the final budget rejection.
- **F012-R1-02: closed**. Capture now performs the golden comparison; runner clears ambient capture flag; validator rejects actual capture-only proof and requires the completed-comparison marker. The frozen old15golden did not change.
- No unrelated new defect found in the affected path/state audit. No final pass or merge approval is issued.

## F012-R1-01 remaining invariant and durable reproduction

Existing requirement: Plan5.1/5.2, AC01/02/04/05; T02/03/05/06; V01/S01 and V04/S04. Live native manager, PCM and decoded resource ownership must fit the unchanged shared64MiB CPU envelope, with forecast checks before actual decoding. The bounded repair contract explicitly promises shared predecode admission, not only rejection after allocation. SD02 repeats that enduring rule.

**Exact cause/location:** `experimental/autographics-auto-driver/src/authoring/textured_models.rs:908-911` still uses `Usage::default()` and loops over existing.values(), omitting `existing.audio_manager_required`. Its candidate closure check at923 therefore charges assets but not the live Host service. Animation allocation follows at926, PNG decode at952 and geometry decode at965. Final `assets::insert` at1060 uses the context-aware check and rejects, but only after the forbidden decoded payloads exist. This site bypasses the new `Usage::from_registry`, unlike flatGLB/PNG/audio forecasts. Normal textured and animated imports plus Storage load_declared share this actual production `decode` path.

External reproducible harness: `review-r2/textured_predecode.rs`, linked to the current library built by this review's package tests. Exact fullPNG fixtures are the R1corpus; it also generates/preserves tuned supported2048wideRGBA PNGs. It uses actual Host/CPAL and ordinary stamped commands, observes `assets::decode_attempts()`, and asserts that over-limit refusal happens with delta0. No fake accounting values or source mutation is used.

Replay commands from driver package (external exe absolute path):

1. `textured_predecode.exe` — real project2d Run/Pause/Stop, New, three normal2048²sprites and tuned2048x1918fourthsprite, then ordinary `import` of original platform.glb. Asset-only forecast: images66,043,904 + model1,058,985 =67,102,889 <=67,108,864. Actual service adds33,528 ->67,136,417. Import returns `resource limit: active asset union (including textures)`, preserves document/no candidate, **but decode_delta=2**. Exit101 at the closing predecode assertion.
2. `textured_predecode.exe --legacy-control` — same legal images/model, no manager; import succeeds/publishes (exit0), CPU67,102,889. This is a genuine accepted control inside unchanged legacy bounds.
3. `textured_predecode.exe --animated` — same mechanism with ordinary `import-animated` of original character.glb. ImageCPU66,002,944 + model1,102,359 =67,105,303; with service67,138,831. Named refusal but **decode_delta=2**, exit101. `--animated --legacy-control` accepts/publishes, exit0.
4. `textured_predecode.exe --open` — creates a valid scene6 saved closure with original platformGLB+PNG, then on the retained-manager Host invokes normal Open and Discard. The actual Storage/load_declared path rejects the union after **decode_delta=2**; exit101. `--open --legacy-control` accepts/publishes, exit0. Generated saved project and blobs are durable under review-r2/model-project.

Expected: all three manager-required over-limit paths refuse BEFORE animation/image/geometry allocation (decode counter unchanged) and preserve accepted state; manager-free matching controls remain legal. Observed: all reject only after two real decoding operations. A final cap rejection is real progress, but does not satisfy the preallocation requirement. The defect is an incomplete repair of the original invariant, not new scope or an asserted persistent leak.

Minimal correction/closing oracle: propagate the Registry's requirement into this candidate Usage before adding closure fields; audit every Usage construction rather than patching only the listed import command. Replay static, animated and load paths with their accepted controls, check unchanged decode counts on refusal and last-good state, and retain original PNG/New/Open/empty-cue/near-bound/duplicate/first-audio/combined-pose controls. No wholeAG/fullperformance run is required to diagnose/close this finding; final frozen gate stays separate.

## Repaired areas independently checked

Read full repair diff and work repair-contract/closure; checked actual permanent assertions, not their totals alone.

- Registry carries a transient manager-required budget context, preserves it across clone, and keeps resource identities/persistence separate. Host resets stale view context, merges current/candidate/pending assets, then reconstructs its actual native owner. Common `Usage::from_registry` charges uncovered service if no audio asset already reserves it; asset-backed audio avoids double charging. FlatGLB, standalonePNG/sharedimage and audio import forecasts now use this context. Registry conversions used as fresh identity collections intentionally reset context; actual Host views reconstruct ownership.
- `cargo test --locked --offline --test public_audio_host budget -- --test-threads=1 --nocapture`: exit0,2passed/9filtered. Inspected both tests: real healthy manager retained across2DNew/3DOpen; report33528with0raw identities; fourth fullPNG and2044highnear-bound refused before decode;2043highaccepted; duplicatefree; first32frameWAV replaces service reserve rather than double charging. Manager-free fourPNGexact64MiB accepted; empty-cue newRun refuses before native allocation and leaves Editor/camera/idle unchanged. Exact fixture hashes match R1inputs.
- `cargo test --locked --offline --test public_audio_host empty_audio_service_and_animation_pose_share_one_cpu_ceiling -- --test-threads=1 --nocapture`: exit0,1passed/10filtered. Actual animatedcharacter/PNGarrays distinguish asset+manager and asset+pose individually fitting from combined over-cap; Audio3Run refuses before native allocation. Same resources/pose on legacy node-animation profile initializes without a manager.
- Work resource9, start/restart2 and postfault1 actual logs independently hashed/read and assertions retained. New Host Run/restart checks combine asset+service+pose before opening; failed manager replacement still drops old before freshopen; healthyStop retention and causalFault behavior unchanged. No extra failure exit found outside the unconverted closure forecast.
- `cargo test --locked --offline --test public_audio_compat -- --test-threads=1 --nocapture` with `PLAN012_CAPTURE_LEGACY=<external review-r2/captured-legacy.json>`: exit0,1passed/0filtered. Actual output contains both capture notice and `Verified15 legacy profiles against frozen exact-byte baseline`. Test now reaches the unchanged baseline assert before the marker. Environment scoped to child shell; no source output.
- Current gate counts accepts actual new normal/capture-verified logs and rejects the actual R1capture-only log plus actual wrong-goldenWITHcapture1failed log. Wrong-golden was work's controlled actual mutation and final restoration; reviewer did not repeat mutating controls. Actual prior log SHA065c90... and goldenSHAe5312820... match expected. Gate run clears capture environment; validate calls the same strict marker predicate; controls mutate marker and rehash actual copied log. F02 complete.

## AC/V/S evidence map: updated versus retained

| criterion | R2 result | updated and retained evidence |
|---|---|---|
| AC01 / T01,02,06 / V01,S01,V05,S05 | partial | Explicit profiles, inherited prices/caps/source/input order and SDK/006pins retained. Original service ownership cases now pass; remaining predecode forecast F01 fails. |
| AC02 / T02,03 / V01,S01 | partial | Version7/cue/identity/raw8/32MiB/disk/schema/storage bounded codec and repaired-readback behavior retained from R1; repair resource9 and actual Registrycaller build cover changed container. Static/animated/load closure preallocation still fails F01. |
| AC03 / T02,04 / V02,S02,V03,S03 | retained pass except budget dependency | Auto assembly, exact wire, canonical rows,4/5requests, ordinal/replay/cue/gain/span, true32/33event composition, commonaccept/mixer/nativepostmappingPCM/render-multimount mechanisms and inputs unchanged. R1actual assertion/log evidence retained; current cold functional consumer independently validates both12645frame source-sensitive cue/gain/motion paths. |
| AC04 / T05 / V03,S03,V04,S04 | partial | Pause/Step/Resume/Stop/fault/nativequantum/40Runweakowners are unchanged and retained; changedRun/restart preflight verified by targeted pose/start/restart/postfault evidence. Live-manager lifetime now correctly reported/charged at original boundary examples, but predecode allocation sibling F01 open. |
| AC05 / T06 / V05,S05 | partial | Current identity1534/source/SDK/locks and fresh moved functionalconsumer valid; F02closes and old15golden unchanged. Full prior performance is historical with unchanged mechanics, not falsely relabeled currentcompleteproof; new44case final gate remains pending. Budget sibling F01 prevents completion. |
| AC06 / T07 | fail | Affected independent finalR2 complete; F01not fully closed. |
| AC07 / T08 | pending merge | New44case frozen `check_public_audio.py --controls`, complete current performance/freshproof after repair, no aggregategate run in this review. |
| AC08 / T09,10 | pending merge | Canonical Spec/ledger/index/delivery/archive/guard-clean/recovery. |

Explicit unchanged-path rationale: repair diff does not alter codec.rs, mixer.rs/native.rs, AutoAPI/assembly, Playwire/admission/accept state transitions, GPUvisual preparation, pausecursor/stepordinal behavior or original project/script/bank/provenance inputs. R1read assertions and bound raw codec/PCM/native/wire/40cycle/performance evidence remain applicable to those mechanisms. Container/budget changes are not covered solely by that reuse: checked new productioncallers/resources/Run/restart and independently exercised sibling closuredecode failure. No first-issue stopping or global replay substituted for this audit.

## Consumer/source/spec verification

Current `verify_source.py` from producer and movedB each exit0: identity1534, SDK681/17local/29edges/479metadata, both locks. Package fmt and diff checks exit0. Final branch/HEAD/status unchanged and clean. Existing SDK146warnings remain; no new implementationwarning observed.

Reviewed current functional-only proof `r1-functional-proof.json`: SHA/binary/PE33554432/actual1passed1filtered log/sourceidentity/manifestdir verified against currentB. Bothprofile consumer.json checks12645PCMframes, move/coldreopen/eventselect/source-cuegain-motion sensitivity and licenses. Bsourceguard independently rerun; Aabsent and Bcurrentidentity byte-equal. This proof does not contain complete performance and is NOT a complete gate manifest. Prior t06freshfinalfullproof is historical. No fresh sourcecopy/build or performancecampaign run by reviewer.

Spec input hashes are unchanged from R1 (see exact hashes below). Proposed SD SHA6b693eeadabbf68bf1d22f8cd28b913c1d0189b1507d99ad0a943a32fb24f8f2. SD01finiteaudio/accept/native/lifecycle retained; SD02adds correct intended liveHost/emptycue/combinedpose/predecode rule but implementation remains incomplete at closuredecode; do not weaken it. SD03profile/price/bundle/input/legacy compatibility stays accurate and F02proof clarification is correct. Exact supersedes/new paths and GOAL008 unchanged. G12a/ag-examples005/013HUDfont/fullgame/fullAG boundaries retained; canonical Specs untouched.

## Actual commands, limits and route

Commands are detailed above and raw logs/hashes below. All package tests used external target-driver, locked/offline, RUST_MIN_STACK16777216 and testthreads1. External Rust probe was compiled with current autographics library4e3...rlib plus existing agp73e...rlib, nativeWindowsimportlib path and mainSTACK33554432; outputs solely review-r2. No mutating sourcecontrols/generators/formatwrites, Planedit, childagent/chat/tree, fullworkspace/root/fullperformance or44casegate execution.

Outcome requires returning affected work to executing, preserving F02closed and original-exampleF01closure while reopening F01's full-invariant row and dependentAC/T/SD evidence. After a second failed final review, work should explicitly diagnose the missed Usageconstruction and adjust its coverage/design audit before another handoff; not merely add this one input. Repair remains within r1scope. Commit in same tree, regenerate invalidated identity/functionalproof, then targeted independent re-review. Only after work-owned closure may merge run its current44case gate and closeout stages.

## Exact SHA256 inputs/evidence

- `D:\autostack\auto-lang\docs\specs\auto-graphics\scene-assets.md`: `b5362e41bb90ec6c9414792888a052ea566035f345d71732d8066a842f0b0a14`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\behavior-api.md`: `3a1ead71ea2b21f14c199915c7043a12d55dae0642174815c73f39abcd8ec1bc`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\overview.md`: `66d90c6c5947cfa54c6daaa828b679bb654797ab9458eb9a2ee81b4ed2361ef3`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\auto-source-driver.md`: `f0555a887362c92af8520af13443fb0ba61f28b7070677556c95bb06bf6d79d1`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\runtime-triggers-and-rounds.md`: `1d0e37f3a641d33fd6f4408bbcf039f28cc41b90c6765127bd04ad8585eb2e44`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\node-animation-and-facing.md`: `42db8e3774d7bf6dba1d0ee8dda7ad2590a003ed67af79fd2d52042364933822`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\public-audio-spec-delta.md`: `6b693eeadabbf68bf1d22f8cd28b913c1d0189b1507d99ad0a943a32fb24f8f2`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\plan012-identity.json`: `fbea61472b0e28ec113b93d4dd0d207d17a2c817ed1a7f39168df4997374c9b4`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\plan012-legacy-profiles.json`: `e5312820e03c6f3aa0a5af15a240d42c6dfb70eb5b7c7f21a4bf092169d54158`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\Cargo.lock`: `4a0b3040fffa2523f3932f2ab2b427e3c13285abce751813147cfe28000d9b06`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-prototype\Cargo.lock`: `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\r1-functional-proof.json`: `e83c55aea93699acc1473dd9d2f83c33593dd8b84fde3cef2abdf0aec4eabfe7`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\animated-boundary.png`: `e0af3a2769b95bef1d090eb425369f2d79127c6ef700c83647c99559c819968f`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\animated_predecode.log`: `3d8b1b5ef181d6d0f629e06dc0c33a6c79a89f06b813611fe1ad99763ad56ab8`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\animated_predecode_control.log`: `291417d4e417a17083cc9901e47d8cf94f7ef88e21161a420da7f6187d98003a`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\budget.log`: `25e97a8695f78aada57aadd69844fad200ea50c9a57ae31a06737c9744839fd5`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\captured-legacy.json`: `1f5159df61d8075ee9ee1db9a000acca0fe239745b897f3101e8d3ffd9f566fe`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\fresh-functional-source.log`: `1168f7c6f65be5c5d934c0647bb3ea7dbb06b288f73d0b69261905d8a7f4896b`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\legacy-capture-verified.log`: `a6f944e26c53eeee974d5ce6a0036e955ae94c81c166039605e9c0477e380078`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\pose-service.log`: `c7dc3a169537d33642e1f0497cd57e999d7270f13e7f8986962657ca916ffe5f`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\source-final.log`: `47d12582dc36be6ea8dee4bba9aea499d202269c139f8d80ea0aabd0013fc1ba`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured-boundary.png`: `c6b5ec65c08c647084e172ea0eaac85dfeb07a98d7455e42bf286f79a4b0f1fb`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_open.log`: `f0a41cb0a825ab6dde1a8a355e6d40555ad0b94100c42bd6e58050b9ebfef31f`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_open_control.log`: `0195b455083c81fdd012136f65ebd2d4487494c065ffd588ebe29a998db62179`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_predecode.exe`: `8f1b18d7b94c187915c06fd83e803f6548065cb5a56d49c5edb492a085d07fda`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_predecode.log`: `3fa7debc27f011b30f2004f3c99fa5b9a4b7817aa3d1bb707cd67ec1ef580644`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_predecode.rs`: `007b79f5fb335805886b03cba7775d0e71eadfbfc99970376f9b95c948a900b3`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\textured_predecode_control.log`: `1aa6bc0832a55d2e3a1e4d5a52e37c159a039f50dba76125e83fa85502decd2f`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\model-project\scene.agscene`: `2fbf5eb26981188f62c8aea7af096e9ac37862345398567109acf7bf8d3cb0aa`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\model-project\assets\106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3.png`: `106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r2\model-project\assets\78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c.glb`: `78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c`
