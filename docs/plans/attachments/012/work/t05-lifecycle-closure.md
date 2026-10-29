# PLAN-012 T-05 lifecycle closure (2026-10-30)

Code `4400661a04c456b710e757cfa697876bc98adcc9`, unique
`D:/autostack/.wt/ui-012/auto-ui`, branch `codex/plan-012-dev`.
V-03/04 and AC-04 work-owned lifecycle/capacity conditions are closed.
T-06 consumer/identity/performance/spec-delta remains open. This is not final review.

The native manager now has one Host begin path for ordinary Run and round restart.
A faulted stream is dropped before its replacement is opened, avoiding both a
false recovery by clearing mixer fault and two simultaneously resident managers.
Fresh-run admission errors retain the previous round's complete presentation;
initial-start audio admission failure also reinstalls the actual editing camera
before returning idle. Healthy native managers still survive Stop for reuse.

Permanent scoped assertions:

- Both dimensions accept four real Auto/Host candidates with two voices each
  while the single in-flight candidate is accepted under Pause. No cursor can
  expire under the wall clock. The fifth has exactly one new request (the ninth
  voice): named eight-voice limit refuses whole candidate before common accept,
  last tick/nodes/GPU image stay identical, no PCM is submitted, and Fault clears
  existing voices. No stealing/dropping/queueing serves as a substitute.
- An independently specified nonconstant PCM ramp renders two frames, then
  pauses through4096silent output frames. A neutral Step consumes new ordinals
  but adds no voices. Resume emits exactly the original waveform from frame2 to
  its half-open tail; then silence. Reusing the Step ordinal refuses.
- Both dimensions first deliver actual nonzero samples to the real native stream,
  then invoke the same terminal `Mixer::output_fault` entry used by CPAL's error
  callback. Host produces named `audio_fault`, releases the VM and voices, keeps
  the accepted Frame/GPU image, and the next service quantum is Run0/silent.
  Emitted sample count is retained; repeated pumps/Resume do not retry. Stop+
  fresh Run opens a different real stream manager, resets ordinal to1 and does
  not decode PCM again. This injects the output-error callback's terminal path;
  it does not claim an actual physical disconnection of the user's device.
- Both dimensions run20actual start/accept/Stop cycles (40total). Every run has
  a new Run ID and ordinal1, previous control rejects, actual VM owner becomes
  dead at Stop, voices/Run become0 and the next native quantum stays silent.
  The same weak manager identity remains alive across healthy Stop, zero warm
  decode/mesh uploads. Host drop releases the actual manager and unpinned PCM.
  A legitimate external PCM pin survives Host drop; its final drop releases the
  last weak PCM owner. No polling timeout or logical live counter stands in for
  these owner probes.
- A deterministic private round-restart case injects error at the accepted round
  boundary, checks old stream weak owner and VM dead, new stream/no fault/new Run,
  old control denied. A controlled mutation that reuses the faulted manager
  fails specifically `failed stream must be dropped, not cleared by Mixer::begin`;
  restored production bytes pass. Separate actual begin-admission failure after
  runtime-view preparation verifies installed GPU camera restored and Host idle.

These checks use the real CPAL default stream on this host (ToDesk output;
44100Hz stereo F32 in previous native observations). Hardware already-buffered
PCM cannot be withdrawn; the asserted boundary is the next service quantum.
No device-unavailable positive is fabricated: actual opens succeeded. Native
open/config/build/play errors remain named `audio_unavailable` in production.
T-04 retains the full exact submitted waveform and channel/format oracle.

All commands below run from the driver package with the existing external
`CARGO_TARGET_DIR`. Logs are local-only under
`D:/autostack/plan-artifacts/auto-ui/plan-012`.

| command selection (`cargo test --locked --offline ... -- --test-threads=1 --nocapture`) | result | log / SHA256 |
|---|---|---|
| --test public_audio_host ninth_voice |1passed,7filtered,exit0|t05-capacity.log / 92d009643a64bcdda6f72de81de59b1a9501101a39a45ea37c1c48bd6aa13baa|
| --test public_audio_host postaccept_native_error |1passed,7filtered,exit0|t05-device-fault.log / 728a17ae37a240da541789c720090dba441b99025bf8f0c1c92fb6f6421e34d8|
| --test public_audio_host twenty_runs |1passed,7filtered,exit0|t05-owners.log / adfb9af16495645eb0200d348bc2d6d722a704c6defb0f7c90b3e7a2c7360308|
| --test public_audio pause_freezes |1passed,9filtered,exit0|t05-pause-cursor.log / 41b236b6d15cf5b109f0c14a256150c0f51d6dfd10a094ea713bc28a3d6e0891|
| --lib audio_restart_tests |2passed,36filtered,exit0|t05-start-restart-final.log / 1c2cc055bc33c67ee32e42fc689785ffb969ab4019bb38a2d5b06222b96d1ebb|
| --lib round_restart_reopens_faulted (deliberately mutated manager-reuse condition, restored finally) |1expected failed,36filtered,exit101|t05-restart-mutant.log / 44eebb0691fa8eaef024c7bfc1a268ab5fd2da9bc84561b1b99a948b72362f08|

Final package fmt/diff checks pass. SDK146existing warnings unchanged, no new
delivery warnings observed. The last start-error camera fix affects only that
failure branch and was checked by the final2case module; successful lifecycle,
capacity and postaccept cases above retain the identical path. Earlier T-03/04
resource/codec/wire/native cases are reused for unchanged paths, not rerun as a
whole campaign. Final frozen directed gate remains review/merge-owned.
