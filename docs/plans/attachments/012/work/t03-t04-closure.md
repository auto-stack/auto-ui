# PLAN-012 T-03/T-04 closure (2026-10-30)

Unique implementation tree remains `D:/autostack/.wt/ui-012/auto-ui` on
`codex/plan-012-dev`. T-03 commit0bb1ce1 adds permanent resource regressions;
T-04 commitb10cc6a binds real VM output admission and submitted-PCM observation.
This closes these tasks, not the later capacity/device/20cycle/fresh consumer gates.

## T-03 / V-01 / AC-01,02 resource path

`public_audio_resources`9/9 and `public_audio_codec`6/6 pass,0failed/0filtered.
Actual public paths are import/Registry, scene7 decode, Editor transactions,
Storage save/verify/load, plus the Host active union. Thirty serialized cue/schema/
sample-metadata controls and in-memory NaN/Inf refuse;8cue positive shares one
PCM owner. Wrong rate/channels/frame count, missing/truncated/changed blob refuse
even with warm cache; repaired bytes cold-reopen correctly. Five save fault
positions preserve entry-last commit, retry reuse and pending verification;
damaged pending closure cannot confirm and repair is followed by actual cold load.

Eight actual small audio identities accepted; ninth refuses before decode,
duplicate file streaming reuses original Asset Arc with no new decode. Undo/Redo
retains the eighth identity in history. Four actual valid8MiB PCM16 WAVs with RIFF
JUNK chunks fill32MiB raw exactly; duplicate still works, next unique raw refuses.
Three actual2048² decoded PNGs consume48MiB and leave insufficient peak CPU for
Vorbis: rejection precedes decode; removing one permits actual decode. This tests
real resident arrays, not made-up accounting values. Disk residual8file/32MiB
quotas, foreign path preservation and wav/WAV/.. reserved Save As aliases preserve
entry/blob bytes; freeing an explicitly owned residual allows save and cold reopen.

The real pending-save snapshot retains its one audio Asset after the Editor opens
an empty epoch. Its union with a candidate containing8new resources rejects the
ninth identity and preserves old accepted document. Old-epoch verification does
not mark the replacement saved or release the pending owner. Explicit Storage
replacement releases it; the same8resource candidate then publishes normally.
An actual junction was created only in a unique external temp directory: ancestor
reparse rejects import and save before decoder/lock/entry effects. Removing only
the junction preserves target bytes. No link/junction was created in the worktree.

Codec edges include all six channel/rate layouts at exact10s and10s+1sample;
duration/raw limit, invalid RIFF spans/padding/duplicate fmt/data/alignment,
unsupported PCM/channel/bit-depth and valid padded unknown-chunk control.
Ogg CRC/serial/sequence/continuation/truncated/EOS and expansive setup mutations
refuse before decode; a forged small granule is rejected by actual decoded block
bound after entering decode. Correct original intervals/waveforms still match
independent permanent libsndfile reference input. 16MiB final PCM is implied more
tightly by stereo48000Hz×10s=3.84MiB in this supported domain; no impossible
16MiB-and10s positive is invented.

| Actual command / result | Local log under D:/autostack/plan-artifacts/auto-ui/plan-012 | SHA256 |
|---|---|---|
| cargo test --locked --offline --test public_audio_resources -- --test-threads=1 --nocapture;9/9 exit0 | t03-resources-expanded.log | 653def2e3280c46c0d332cf59a1c21ff36be5d5f82117c8ac12b8cd558880d00 |
| cargo test --locked --offline --test public_audio_codec -- --test-threads=1 --nocapture;6/6 exit0 | t03-codec-edges.log | 3cdac5bd490938c549fe41134c9cb9892f638abfbdedb36dab97a36804c92e58 |

## T-04 / V-02,03 / AC-03 request and output path

The unit `real_audio_wire_mutations_cannot_accept_partial_frame_or_insert_pcm`
uses real Auto → Play read buffer, then24corruptions for EACH dimension through
ordinary Play admission / independent mixer prepare / common accept closure.
Malformed length/count, float integer identities, wrong Run/tick/bundle, cue,
ordinal/exhaustion/duplicates, nonfinite/out-of-domain gain and noncanonical unused
rows cannot change last-good tick or insert any PCM. Legitimate typed ordinal
16777217 remains exact and produces the independently expected4sample pulse then
silence. This is not only a disconnected Packet parser test. Its3D fixture was
corrected from a flat two-parts model to the actual accepted volumetric character;
the initial numeric-range fixture failure is not a product limitation.

`public_audio` now9/9 includes real Auto integer16777217 and2147483646, controlled
2147483647refusal,4/5requests, optional empty/4/5sound bundles, duplicate visual/
mixed mode refusals and legitimate standalone visual+standalone audio. Two cue
ranges on an actual decoded stereo PCM16 bank have different lengths and gains.
Their entire mixed waveform and silence tail match an independent analytic ramp
oracle at22050/44100/48000Hz for both profiles. Finishing then replaying an accepted
packet cannot restart a voice; changed same-tick content is rejected.

Actual native stream also contains the full distinctive decoded span waveform,
gain and tail, then transport replay causes no additional nonzero samples. Device
is ToDesk Virtual Audio,44100Hz stereo F32; span output8frames incl tail, max101us,
deadline misses0. A second native control observed970nonzero frames/max123us.
Mono/I16/U16 hardware outputs are NOT claimed: the exact callback helper is
separately verified with mono phase cancellation, signed/unsigned quantization
and extra-channel zeroing. Native observation was corrected to use the exact
post-routing/post-quantization samples written to this stream; pre-mapping
stereo would falsely report nonzero for a cancelling mono mix. Host snapshot now
reports selected channels/sample format as well as actual callback statistics.

Two additional real portals mount the same accepted2D/3D surface. Five render+
capture iterations keep the same GPU image, accepted tick, paused voice count and
nonzero PCM count; actual transport replay returns Replay without logical accept.
Original last-good GPU/33credit/Step/Stop tests remain valid on unchanged paths;
the rendering replay case exercises the updated actual native observer.

| Actual command / result | Local log | SHA256 |
|---|---|---|
| cargo test --locked --offline --lib real_audio_wire_mutations -- --test-threads=1 --nocapture;1/1,35unrelated filtered exit0 | t04-wire-final.log | 37b67731759ee5be50652ced9646a03994cec5df9563563ad9c27bb667c17d2c |
| cargo test --locked --offline --lib observed_native_pcm -- --test-threads=1 --nocapture;1/1,35unrelated filtered exit0 | t04-native-format.log | 594866a17638f001a56cad1b4f0848ccc92e5500899e5fa65211dcccd1c5a95e |
| cargo test --locked --offline --test public_audio -- --test-threads=1 --nocapture;9/9,0filtered exit0 | t04-pcm-and-requests.log | 908211cbda9522c1e61d1b8e08bb3288235a58ee8c269db394bf3c38643d0d60 |
| cargo test --locked --offline --test public_audio_host repeated_render_capture -- --test-threads=1 --nocapture;1/1,4filtered exit0 | t04-render-replay.log | a8f6c12a00065b6827014daa476e5c9cd3df41a39386bc751ef650fb2fcc573f |

Package fmt/diff pass. An initial fmt check accidentally addressed the unrelated
root manifest and refused missing sibling dependencies; no root build/test or
dependency edits occurred. The subsequent package-directory fmt check passes.
SDK146existingwarnings remain; no new delivery warning. Final identity/SD and
fresh consumers are T-06; final frozen full directed gate belongs to review/merge.
