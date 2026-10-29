# PLAN-012 r1 independent final acceptance review

Verdict: **needs_fix**, revision-bound to `87414431b9a1dbda6eb54aad999af7f734a14516`.
Review scope: final. Reviewer: independent read-only subagent `/root/plan012_final_review` of the authorized root chat; no implementation-session narrative used as authority. No model identity claim. Human explicitly authorized this one same-tree review agent; no child agents or new user chats were created. Root owns shared Plan receipt/status edits. No implementation, Plan, canonical Spec or tracked/untracked source edit was made by this reviewer.

## Baseline and authority

- Main shared Plan: `D:/autostack/auto-ui/docs/plans/012-autographics-public-audio-and-accepted-cues.md`, revision1; initially execution_done/T01..06 complete. Read current Plan and all four work summaries. Later authorization bookkeeping on main does not alter r1 runtime contract.
- Authoritative implementation: `D:/autostack/.wt/ui-012/auto-ui`, registered worktree, branch `codex/plan-012-dev`, HEAD `87414431b9a1dbda6eb54aad999af7f734a14516`; diff base `f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131`. `git worktree list --porcelain`, branch/HEAD/status checked on entry and HEAD/status rechecked at completion. Source status clean both times; no implementation drift.
- SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`; 006 delivery `c681663e6ff392d70471abbec560e9d9daf12622`. Source guard binds 1529inputs, SDK681,17local packages/29path edges/479metadata closure and both locks. No repository AGENTS.md found in tree or checked Windows ancestors. Read auto-plan-review skill fully and verification-contract reference.
- Every Cargo invocation used package `experimental/autographics-auto-driver`, `--locked --offline`, target `D:/autostack/plan-artifacts/auto-ui/plan-012/target-driver`, `RUST_MIN_STACK=16777216`, single test thread. No root/workspace/fullAG tests or frozen41case gate run.

## F012-R1-01 — P2 — Host-owned native manager can escape the shared64MiB CPU ceiling

Requirement: Plan5.1/5.2/5.3, AC01/02/04/05, T02/03/05/06, V01/S01 and V04/S04; all native manager/observation/queue/voice owners must remain charged in the unchanged shared CPU envelope, including after Stop/project changes. SD02 proposes this same enduring rule.

Locations (driver-relative): `src/authoring/public_audio/mod.rs:137-148` and `:165-170` reserve service bytes only inside each audio Asset; `src/authoring/host.rs:474-484` builds an asset-only union and `:574-581` reports asset+pose CPU only; `src/authoring/host/play_adapter.rs:5-22` opens/reuses the manager without independent CPU admission. `src/authoring/host/commands.rs:37-45` New/Open can replace every audio asset while keeping Host.audio; Stop intentionally retains it (`play_adapter.rs:308-309`).

Reproduction is durable in `retained_manager.rs`, linked against the current library freshly compiled by the review's selected Cargo tests. It uses real CPAL Host state and ordinary stamped `new`/`import-sprite` commands, not fabricated resource counters. Four independent valid2048x2048RGBA PNGs are84,769raw bytes total and64MiB decoded CPU/GPU. Their raw files/hashes are included below. Inputs stay inside the supported existing image domain.

1. Accepted control: run `retained_manager.exe --cold-control`. Fresh manager-free legacy Host accepts all four imports, reports CPU67,108,864 and manager=false (exit0).
2. Run `retained_manager.exe` (exit101 as expected for the violated invariant). Ordinary bound project2d Run→Pause→Stop leaves native manager healthy. Normal New drops all audio assets: asset_union CPU0, manager=true, reserved_cpu_bytes33,528. Four normal imports then all publish successfully: asset CPU67,108,864, native manager33,528, minimum in-contract CPU67,142,392 >67,108,864. Final assert fails `retained native manager omitted from shared CPU budget`.
3. Run `retained_manager.exe --empty-audio` (exit101). Fresh valid scene7 with cues=[]/no audio asset, legal0.8x1actor, same four image imports, valid Auto character/camera binding, then Audio2 Run. Actual Play reaches Paused/error=None/native run1, manager=true. The same ceiling is exceeded immediately on opening the newly required service. This closes the sibling where fixing New/Open cleanup alone would miss the invariant.

Expected: retain/reuse manager only with its actual service reservation charged; opening a service or accepting the boundary import must refuse before exceeding the cap. Manager-free legacy64MiB control remains accepted. Observed: both paths accept and underreport33,528bytes. Established cause: reservation follows audio asset lifetime, while manager lifetime follows Host and an empty-cue audio profile can own no audio asset. Not a hypothesis about OS/backend hidden allocations; the discrepancy uses Native::reserved_bytes itself, the amount the contract says is charged.

Minimal correction: make required/resident native service ownership participate in Host admission and reporting independently of audio-asset presence; preserve existing Stop reuse and old-profile budgets without double charging the already conservative per-asset reserve. Replay cold-control, retained/New, empty-audio Run, analogous Open-to-no-audio and failure/restart paths where service reservation changes. Recheck existing audio import peak, pending/history union and healthy/fault manager ownership tests affected by the chosen accounting repair. No fullAG gate needed for this repair diagnosis.

## F012-R1-02 — P2 — Legacy capture mode is accepted as compatibility evidence

Requirement: AC05/06, T06/T07, V05/S05; acceptance validator must enforce the actual old15profile compatibility oracle and reject incomplete/non-verifying evidence. Existing test explicitly says capture is NOT verification.

Locations: `tests/public_audio_compat.rs:18-21` returns success after capture without evaluating baseline assertion at25. `tools/check_public_audio.py:67-73` inherits os.environ (including PLAN012_CAPTURE_LEGACY) into the actual runner; `:25-35` validates only counts for legacy, and `:98` reuses that same insufficient check during validation.

Durable actual input: set `PLAN012_CAPTURE_LEGACY=D:/autostack/plan-artifacts/auto-ui/plan-012/review-r1/captured-legacy.json`, then `cargo test --locked --offline --test public_audio_compat -- --test-threads=1 --nocapture`. Actual exit0/1passed/0filtered log says `Captured15 legacy snapshots; capture mode is NOT verification`; no golden comparison executes. The variable was scoped to that child shell and output was external, so no source mutation occurred.

Accepted control: remove the variable and repeat the same command. `legacy-normal.log` contains actual normal golden comparison1passed/0filtered. `legacy_capture_validator.py` calls the gate's real counts('legacy',...) on both logs: normal succeeds; capture also incorrectly succeeds, so the independent closing assertion fails exit1 in `legacy-validator.log`. Source inspection establishes the actual final runner inherits this input and both run/validate use this exact accepting function; the whole pending gate was not executed or claimed false-green as a whole.

Expected: gate execution must force verification and validation must refuse capture-only proof. Observed: capture-only success meets the legacy evidence predicate. Established cause is the explicit early return plus unfiltered inherited environment and count-only validator. Current normal old15byte compatibility itself passes; this finding concerns proof integrity, not a demonstrated old-profile semantic regression.

Minimal correction: separate/disable capture from the acceptance test or scrub capture mode in the gate and reject capture-only logs during validation. Add actual normal/capture controls with external output so the validator demonstrates correct failure propagation. Replay only compatibility/validator affected checks; then regenerate frozen source identity and affected frozen consumer/gate proof on the repaired commit.

## Complete acceptance and risk coverage

`pass` below means the stated sub-invariant was inspected and supported; overall rows marked partial retain the two explicit failures above. This is not a claim that the pending unified gate ran.

| AC / task / V,S | result | inspected implementation, actual assertions and evidence |
|---|---|---|
| AC01 / T01,02,03,06 / V01,S01,V05,S05 | partial | Prior006actual delivery/base pin, explicit111/112and22/23profiles, old caps/prices/input order pass. Bounded codec support and source limits inspected. Shared CPU invariant fails F01. |
| AC02 / T02,03 / V01,S01 | partial | `assets`, `document`, `public_audio`, `storage`, `session` enforce scene7cue array, legacy field/format refusal, exact metadata/raw hash even warm, shared PCM,8cue/8raw/32MiB and safe paths. Resource9and codec6actual assertions/logs confirm save failure, pending repair/cold reopen, disk quotas and junction rejection. Resource service lifetime union fails F01. |
| AC03 / T02,04 / V02,S02,V03,S03 | pass except shared-budget dependency F01 | Real Auto producers and independently typed Host wire, canonical unused rows, exact16777217/2147483646and overflow, cue/gain/span,4/5requests, bundle conflicts, true event101full32/extra33 path. Common accept is after all visual/resource/voice checks and callback deadline; voice insertion follows successful Play.accept under same manager mutex. Complete independent analytic PCM ramps at3rates, post-channel/quantization actual native ring, replay after voice completion, changed same-tick packet and multimount captures are asserted. No render/capture submission hook. |
| AC04 / T05 / V03,S03,V04,S04 | partial |8/9voice whole-frame refusal, last-goodFrame/GPU on preaccept failure, postaccept real output-error terminal path clears voice/Run/no retry, next quantum silent. Pause preserves nonconstant cursor, Step suppresses/consumes ordinal and Resume emits only prior tail.20runs PER dimension probe actual VM/manager/PCM weak owners; healthy Stop retains manager; final Host drops it; external PCM pin legitimate. Faulted manager replaced before reopen, failed start restores camera, failed restart last-good preserved. Runtime lifetime behavior passes; retained service accounting F01 fails. |
| AC05 / T06 / V05,S05 | partial | Actual two bound scene7projects, event-selected different cue, source edit changes chosen half-openPCM12645frame waveform/gain/motion, SaveAs A→B/cold project reopen. Current fresh-final source export/noSDK-noTarget A→B then cold ownSDK/offline2tests proof revalidated independently; final binary/stack/data/log/source hashes valid. Source/step/VM/Host/native metrics separated and hard guards intact. Identity controls and data mutations reject changed inputs. Old15normal compatibility passes, but F02 defeats capture-only evidence predicate; F01 affects budget claim. |
| AC06 / T07 | fail | Independent final review completed; F01/F02 remain open. Root must reopen affected work and dependent evidence rather than grant reviewed. |
| AC07 / T08 | pending merge | `check_public_audio.py --controls`41case unified frozen gate, correct current identity/fresh proof required after repairs. |
| AC08 / T09,10 | pending merge | Canonical SD/ledger/index publication, delivery/archival and guarded worktree cleanup/recovery receipt. |

Codec/preallocation audit: raw read is bounded; WAV chunk lengths/alignment/rate/bit depth/duration checked before Vec allocation; Ogg CRC/serial/sequence/continuation/BOS/EOS/granules/packet spans checked before Symphonia construction. Expansive VQ/VLC/setup preflight uses conservative fixed scratch and checked growth; invalid pinned lookup1 arithmetic named unsupported. Actual blocks limit prediction/8192frames/domain/finiteness; Vec→Arc copy and overlap windows reserved. Supported10s/rate/channel bounds imply finalPCM<16MiB. Read independent soundfile/libsndfile oracle producer and15comparison report contract; original waveform/length reference is not production-decoder self-comparison. No new codec gap found.

Version/storage audit: explicit7cues including[], legacy rejection before normalization; raw content identity plus channels/rate/frames and cue spans; loading verifies hash/length/metadata even cached. RawPool/asset union preserves duplicate streaming/no new decode, pending/history/current owners, disk8/32MiB and blob-first/entry-last/readback. Audio extends existing canonical filename reservation/reparse protections. No imported path is used by playback/SaveAs. No additional persistence defect found.

Lifecycle/irreversibility audit: native callback holds mixer for a whole quantum; sample observations follow channel mapping/format conversion. Preaccept failure cannot insert a voice; already emitted PCM is never rolled back. Output error calls same terminal method tested by injection; this is not a physical-disconnect claim. Fault does not retry/overwrite last-good, manager replacement drops old stream first. Native format F32/I16/U16 logic has mono-cancellation/quantization/extra-channel unit controls; actual tested device is ToDesk virtual stereoF32, not physical speaker pressure. Hardware-buffer latency and backend underruns remain explicitly unmeasured.

Performance audit: current fresh-final2D/3D managed85055/84143B, steps5092/7037, slices6/8, raw2/90202B and6/236544B, nominal sample CPU308844/1479646B; all warm decode/mesh/texture deltas0. Pump4867us/met and8046us/missed are honestly separate from2stickhard limit;3Dtarget miss46us is not a new acceptance failure. Native quantum441at44100Hz, nonzero9699each,max621/169us,0measured deadline misses; no CPAL backend-underrun count inferred. These finite sample results do not close F01 at the actual limit.

## Spec delta and downstream boundary

Read current canonical scene-assets/behavior-api plus relevant overview/auto-source-driver/runtime-triggers/node-animation rules and proposed `research/public-audio-spec-delta.md`.

- SD01 add `docs/specs/auto-graphics/public-audio.md`: profile/schema/finite codec/accept/native/irreversible/lifecycle semantics match implementation apart from the CPU envelope promise F01. Keep proposed support restriction for pinned lookup1 arithmetic and explicit native hardware boundaries. Partial until repair.
- SD02 modify `docs/specs/auto-graphics/scene-assets.md`: scene7identity/versioning/shared raw/disk/storage/PCM rules match; native service joining CPU envelope currently false on the independently reproduced sibling paths. Partial until repair; do not weaken the promise.
- SD03 modify `docs/specs/auto-graphics/behavior-api.md`: explicit profiles, inherited action ordering/consume-once, one-credit bounded bundle,32full event branch and caps match actual code/assertions. Pass content review, subject to final repaired baseline bindings.
- Exact supersedes paths/new path and GOAL008 agree with Plan. No canonical file modified. G12a/ag-examples005 consumes actual later delivered engine SHA, plus its own content/event/cue tests and013font/HUD; fixtures do not claim complete G2-09/G3-08,GalleryZIP,G12registration,011or fullAGmilestone. No downstream scope inflation found.

## Commands actually run in this review

- Read-only git identity/worktree/diff/stat/source reads throughout; before/after status clean and exact HEAD.
- `python tools/verify_plan012.py`: exit0,1529hashes.
- `python tools/fresh_public_audio_consumer.py D:/autostack/plan-artifacts/auto-ui/plan-012/t06-fresh-final --validate`: exit0; current movedBsource/SDK/binary/log/data proof validated. No new consumer copy/build created by review.
- `cargo fmt --check`, `git diff --check`: exit0 initially and finally.
- `cargo test --locked --offline --test public_audio pause_freezes_sample_cursor_and_step_consumes_ordinals_without_backlog -- --test-threads=1 --nocapture`: exit0,1passed/9filtered (`pause-cursor.log`).
- `cargo test --locked --offline --lib real_audio_wire_mutations_cannot_accept_partial_frame_or_insert_pcm -- --test-threads=1 --nocapture`: exit0,1passed/37filtered (`host-wire.log`). Actual both-dimension24wiremutations plus legitimate integer control assertions read.
- Normal and capture-variable compatibility commands as F02: each exit0,1passed/0filtered; capture bypass independently fails the closing validator oracle exit1.
- External `rustc --edition=2021 retained_manager.rs --extern autographics_auto_driver=<current lib4e3...rlib> -L dependency=<target/debug/deps> -L native=<registry/windows_x86_64_msvc0.52.6/lib> -C link-arg=/STACK:33554432 -o retained_manager.exe`: exit0. Then manager-free control exit0; retained/new and valid empty-audioRun budget probes exit101 on explicit expected invariant.
- `python tools/verify_source.py`: final exit0, source1529/SDK681/metadata479and17/29; source-final.log.
- Actual existing raw log SHA/counts independently checked for T03resources9/codec6,T04wire/native-waveforms/render-replay,T05capacity/fault/20cycles/start-restart,T06identity/validator/freshcontrols and fresh-final2tests. All match cited work-summary hashes. Read the actual assertions, not counts alone.

Probe corrections/limitations: initial mistakenly named Pause selector selected0tests/10filtered; it was not credited and was immediately corrected to actual selector above. Direct external link initially lacked registry Windows import-lib search path, then compiled correctly. Initial empty-audio probe had no required character binding (API25), and next used square actor (API24); neither is product evidence. Corrected final legal0.8x1actor+character/camera binding reaches Paused/errorNone/run1. Final retained_manager.rs/logs contain the valid reproduction. No source controls/generators/formatting writes were run. Builds show known SDK146warnings plus existing shim/linker diagnostics; no new implementation warning observed.

## Next and pending gates

Return to work for F012-R1-01 and F012-R1-02 within existing r1 contract. Reopen affected T02/03/05/06 evidence as appropriate (root owns exact task-state normalization) and affected AC/V/SD rows. Keep untouched codec/wire/PCM/lifecycle evidence with its bound inputs; do not rerun global suites to diagnose these defects. Repair in the same authoritative tree, commit, refresh identity and invalidated consumer/source proof, then targeted independent re-review including stated sibling/accepted controls. Only after all work-owned findings close may the merge owner run the frozen41case gate on the repaired HEAD, followed by canonical knowledge/ledger/archive/guard-clean stages. No finalpass or merge authorization is issued by this review.

## Frozen input and evidence SHA256

All exact input/probe/log/binary hashes are also in `evidence-hashes.json`.

- `D:\autostack\auto-lang\docs\specs\auto-graphics\scene-assets.md`: `b5362e41bb90ec6c9414792888a052ea566035f345d71732d8066a842f0b0a14`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\behavior-api.md`: `3a1ead71ea2b21f14c199915c7043a12d55dae0642174815c73f39abcd8ec1bc`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\overview.md`: `66d90c6c5947cfa54c6daaa828b679bb654797ab9458eb9a2ee81b4ed2361ef3`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\auto-source-driver.md`: `f0555a887362c92af8520af13443fb0ba61f28b7070677556c95bb06bf6d79d1`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\runtime-triggers-and-rounds.md`: `1d0e37f3a641d33fd6f4408bbcf039f28cc41b90c6765127bd04ad8585eb2e44`
- `D:\autostack\auto-lang\docs\specs\auto-graphics\node-animation-and-facing.md`: `42db8e3774d7bf6dba1d0ee8dda7ad2590a003ed67af79fd2d52042364933822`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\public-audio-spec-delta.md`: `551376a3654baf3c8f916ff1c858879b6660c10dbd598361dab2e23337a89f82`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\research\plan012-identity.json`: `a5a861cd5275c8f8d1d02cea7deae49a3ea67c986a7513c712c183fb33cffbe6`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-auto-driver\Cargo.lock`: `4a0b3040fffa2523f3932f2ab2b427e3c13285abce751813147cfe28000d9b06`
- `D:\autostack\.wt\ui-012\auto-ui\experimental\autographics-prototype\Cargo.lock`: `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\target-driver\debug\deps\libautographics_auto_driver-4e3cabd7db5fe52a.rlib`: `f6ff8bc2d712b5ab0dab698ea613ba133c24bea6df7eb11679edc3c991135739`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\budget0.png`: `6a6515adc7bc31b5043f660e1a805746f9381068ff2acd7b9637d7620f0b0b66`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\budget1.png`: `45717ba6bb85dded4c61b955fbb2362078fc86db4f37482af6170e0f23705827`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\budget2.png`: `dca0132f0add1dea263fa9c7f6f090d2f2e2efbc263a76c2e765962dfbc5427e`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\budget3.png`: `1ebb90a4a4b5bc1f61632fa354bf04dc3d7e932656e3b087c6cdd291a1ae6275`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\captured-legacy.json`: `1f5159df61d8075ee9ee1db9a000acca0fe239745b897f3101e8d3ffd9f566fe`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\empty_audio_manager.log`: `ab8df0cc1cc508b89f2e134bef3cea98c617face60ba42a3274fdb011fd00ab4`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\host-wire.log`: `fb71b1dc3d5ebf38c88ac6ecf5fde47b729c7d26768ac9454bc3cb5fe4309b81`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\legacy-capture.log`: `525f899798469b953185cb3297899cecd88b5b4c6b31414686e8af84a519e6e6`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\legacy-normal.log`: `89c40ee515365eff77a43bfad2137b75e7c8c24b059431a9b510d599bdf0c344`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\legacy-validator.log`: `149c0c92aaa899868fadee72e2d2024785c9478961cf55febec66f34485ded47`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\legacy_capture_validator.py`: `2cbbb261224617bbc4ea4fc7fff6de5f805c0e3fe1cdb66a054519b08553c709`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\pause-cursor.log`: `dd87c705e5a97688290b852e14165405aa54df2bc5dd294c5a1c2268d947b2af`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\retained_manager.exe`: `bd9fbf8e793c5f00ce09c78b63d1d9cb279d672aae081fb2ef61479d79ebcbd6`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\retained_manager.log`: `cfe346c9629cfcfdc3a8ebe7658184a4f35a7c0d652efcf434d87a717e731427`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\retained_manager.rs`: `53947715a7d4b701a8cb4006d600fac266ec0c25022b988fe13e731eb38c47bb`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\retained_manager_control.log`: `27ca67d59e65b4bae0aa0d5c463f840da93bfe92b40f66557d701942ca5bb740`
- `D:\autostack\plan-artifacts\auto-ui\plan-012\review-r1\source-final.log`: `32670de2817e262dc9dbac6c4749fcff269c609d1673ba22fc7c390fe43be1e6`
