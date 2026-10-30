# PLAN-012 R1 repair ready for affected independent review

Code `67ddbe2e61693773900af9630ef455d636d55108`, same ui-012tree/branch, clean.
Repair base `87414431b9a1dbda6eb54aad999af7f734a14516`. This is the first bounded
repair after the independent R1; not a separate review or final pass.
All raw files below live under `D:/autostack/plan-artifacts/auto-ui/plan-012`.

## Current finding closure

| finding | invariant / affected states | actual repair and durable cases | result / remaining gap |
|---|---|---|---|
| F012-R1-01 | required/live Host native manager must join the same CPU union independently of audio Asset lifetime, through predecode and Run/pose admission, with no double reserve or reduction of legacy limits | Registry now carries transient nonpersistent manager-required context; common Usage charges uncovered Native reservation only when no audio Asset already reserves it. Host rebuilds actual owner context for current/history/candidate/pending, imports inherit it, first audio candidate replaces that reserve, not doubles it. Native open preflight plus whole asset/service/pose Run/restart check occurs before opening. Three permanent Host tests use exact R1PNG bytes, normal stamped imports and real native owner. | final budget2cases/9filtered, combined pose1/10filtered pass; resource9/0filtered, start/restart2/36filtered, real postfault1/10filtered pass. No open work gap; independent re-review and final gate pending |
| F012-R1-02 | capture cannot replace old15exact-byte verification, and validator must refuse capture-only proof | Capture writes its optional output then still asserts unchanged frozen golden and prints completed-comparison marker. Gate clears inherited PLAN012_CAPTURE_LEGACY and requires marker/rejects old capture-only message. Permanent gate control actually removes marker from real legacy log/recomputes log hash and must reject. | real normal and capture runs each1passed/0filtered; deliberately wrong golden with capture exits101/failed, finally original bytes restored and normal passes; actual R1capture log and failed proof rejected by current counts, valid new normal/capture accepted. No old profile source/expected data change |

The original exact PNG inputs are now permanent fixtures under
driver/tests/fixtures/public_audio_budget (84769bytes total), with identical four
review hashes. They are required executable regression inputs, not a committed
probe/evidence dump. The independent raw probe/logs remain external and unchanged.

The boundary controls discriminate plausible incomplete repairs:

- Manager-free legacy accepts all four2048² images at exactly67108864CPU/GPU bytes.
  No manager is created, so the old64MiB domain is preserved.
- Both2DNew and3DOpen-to-empty after healthy Stop retain the same actual manager
  and report33528CPU bytes with0raw identities. Three images fit; fourth full image
  is refused BEFORE decode/candidate publication with last accepted document intact.
- A fourth2048x2044 image saves32768bytes, still too little, and refuses before
  decode.2048x2043 saves40960bytes and accepts. Duplicate import is still free.
  A first32frame WAV then accepts in that narrow remaining envelope, replaces the
  Host-only reservation with the conservative Asset service reserve, and total
  CPU equals actual image usage plus the audio Asset once. This catches a repair
  that double charges the existing manager or rejects every fourth resource.
- Empty scene7cues=[]/no audio Asset on the identical full64MiB image set refuses
  Audio2Run before native allocation and preserves editing document/camera/idle.
- Additional same-cause combined-owner control uses the real original animated
  character+sharedPNG and supported image arrays. Asset+manager and asset+pose each
  fit, but asset+manager+pose exceed64MiB: Audio3Run refuses before native opening.
  Identical assets/pose on the legacy node-animation profile with no manager accept
  and initialize. No fabricated byte counters or lowered caps establish this.

`Registry` remains a map-like resource container (Deref/iteration/collect/array and
BTreeMap construction retained), with transient service context separate from
persisted resource identities. Context cloning preserves predecode forecasts;
Host reconstructs actual live requirement instead of trusting stale views.
Audio candidates use a shared Usage path that recognizes their existing reserve.
No global budget/cache, fake resource/blob, profile/action/price/SDK change added.

## Current identity and consumer

New source identity1534files SHA256
`fbea61472b0e28ec113b93d4dd0d207d17a2c817ed1a7f39168df4997374c9b4`.
SDK681/17local/29edges/479metadata and both locks are unchanged.
Proposed SD adds the existing Host-owned requirement/combined-pose rule and proof
integrity clarification; no semantic requirement or quota changes. SD SHA256
`6b693eeadabbf68bf1d22f8cd28b913c1d0189b1507d99ad0a943a32fb24f8f2`.
The old15golden bytes are exactly those of the R1base (SHA256
`e5312820e03c6f3aa0a5af15a240d42c6dfb70eb5b7c7f21a4bf092169d54158`).

Affected functional consumer was exported as source-only A, moved to B before
preparation (no .sdk-source/target), SDK prepared from B's own snapshot, full
source/metadata verified, then locked/offline compiled in B-local target.
`public_audio_consumer bound_two` passes1case/1filtered after95s fresh compile and
6.17s execution. Both dimensions/cold save-move-reopen/source-sensitive chosen
cue/gain/motion and12645frame independent PCM comparisons pass; actual compiled
manifest_dir is B. Producer/B identities are byte-equal. Actual B executable
`public_audio_consumer-e5487636cc670b01.exe` SHA256
`b6f359ad085fa149420050d5971855de7c08703a5a0406ce06d86f0a566c0dd7`, PE32MiB/
configured worker16MiB. `r1-functional-proof.json` explicitly says functional-only,
not a complete merge-gate proof. The final44case frozen gate will create/validate
complete current performance/fresh proof; no old full manifest is relabeled.

Reuse is scoped: Auto assembly/wire, codec internals, mixer/native PCM conversion,
GPU visuals, lifecycle cursors/irreversibility and actual40Run churn are unchanged.
Their R1-inspected bound evidence is retained. The changed budget mechanism is
covered by original real PNG inputs plus New/Open/emptyRun/combined-pose/first-audio/
duplicate controls, common resources9, and affected start/restart/postfault checks.
Earlier observed performance remains historical and explicitly separated from the
pending new final gate; the unchanged original8ms target/hard limits are not waived.

## Commands and evidence (all exit0 unless marked expected101)

Each Cargo command uses `--locked --offline ... -- --test-threads=1 --nocapture`
from the driver package, producer target-driver or the cold B-local target.

| selected command | actual results | log / SHA256 |
|---|---|---|
| --test public_audio_host budget |2passed,9filtered|r1-budget-final.log / a8adafbc6d36f6c8b75ff5e2d927a9282bd28284ddd3ac36edc6087c47c2ccd7|
| --test public_audio_host empty_audio_service_and_animation_pose |1passed,10filtered|r1-pose-service.log / 52a0619765a3ebfebf12961ddef2bf13b28bfa3b299b3393a835a75f7c666b4e|
| --test public_audio_resources |9passed,0filtered|r1-resources.log / bd4826ccc842d7d936f97fe0ead3f60c1984d0446e2395c8af3539b00940e677|
| --lib audio_restart_tests |2passed,36filtered|r1-start-restart.log / 228cf66e14f753dff147a2339e1a315c6beeecce2d07144868c973c41b9aa802|
| --test public_audio_host postaccept_native_error |1passed,10filtered|r1-postfault.log / 67e9441560883be8ac3f897da1ef02ddd4c7d33b4eccac0e7b3ce3bf3dd52479|
| --test public_audio_compat, normal / capture |each1passed,0filtered|r1-legacy-normal.log / 6cf30744b2362dcd310c021102503d1620770281ec63237c452dc8ba87b8638b; r1-legacy-capture-verified.log / fbe2e03784b0921f9cffacfe58b357d3f67db7179d89b3246ff097241c11f39b|
| same target, mutated wrong golden WITH capture |expected exit101/1failed, then finally restored|r1-legacy-capture-bad-golden.log / 065c90e82fc0984f210b8a1689b55037a7a47ed60c1249c46a8da5ea8bf3ae38|
| current validator on actual valid/capture-only/failed logs |valid2accept/reject2|r1-legacy-validator.log / 3eb8d8a561d1edcd532aaf81338cd993759adb2b7ecea686e570059c4e170bc1|
| python tools/verify_plan012.py --controls |1534/681,controls pass|r1-identity-controls.log / 6e9c07d5e890fa9b9c7acda9d66040fb255d0d00c36be4c61bda656c226189a4|
| B python tools/verify_source.py |1534/SDK681/479metadata|r1-functional-source.log / b4a9cca198413029fd8b21bcdccef0b2542f8c366d5c25998166a645a204fc26|
| B --test public_audio_consumer bound_two |1passed,1filtered,functional scope only|r1-functional-consumer.log / 2d36b1a175aa268a7fabb611059050b89d705604e95fd4b147ffd4227b9bc751|

Final package fmt/diff and Python compile checks pass. SDK146known warnings remain,
no new delivery warning observed. An initial cargo check hit disk-full in Windows
dependency metadata; the next targeted build exposed missing map-array constructor
compatibility in the new container, corrected before the passing checks above.
These were repair diagnostics, not passing gate evidence or additional reviews.

Two proposed cache deletions were rejected by automatic policy review and did not
execute. A genuinely safer nondeleting NTFS compression of the obsolete first
t06-fresh-consumer Cargo target then completed:8666files/885dirs,5093094599logical
bytes stored in2618700258bytes;129EXE hashes verified unchanged before/after and
paths preserved. No source/evidence/cache directory was deleted. Available space
returned; no broader cleanup or unverified claim of deleting the final artifacts.

All work-owned AC01..05 are ready on the repaired baseline, open in-contract work
findings/coverage gaps/decisions empty. Independent affected R2 still required;
T07not complete. Final gate now44cases because3permanent Host regressions were added;
T08..10 remain merge-owned pending, no canonical Spec/ledger publication or landing.
