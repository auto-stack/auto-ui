# PLAN-012 proposed canonical delta (r1)

Implementation and test inputs live in auto-ui, under
`experimental/autographics-auto-driver`. This is the proposed delta for merge
into auto-lang canonical Specs; it does not itself publish those Specs.

## SD-01 add docs/specs/auto-graphics/public-audio.md

The independent Windows authoring engine admits finite public one-shot sound
resources. Explicit `autographics.audio2d/1` (`--behavior-api audio2d`) and
`autographics.audio3d/1` (`audio3d`) profiles add public audio to the delivered
sprite2d and node-animation3d compositions respectively. A normal bound-project
Run opens the default native CPAL stream automatically. Unsupported/no output
device/config/format/build/play fails as `audio_unavailable`; it never selects a
silent substitute. Native output supports F32, I16, U16; mono output averages the
two mixer channels, stereo keeps them, additional device channels receive zero.
PCM observations describe this service's submitted samples after channel routing
and quantization; they are not system recording or physical speaker measurements.

Input is Ogg/Vorbis or little-endian RIFF/WAVE PCM16, mono/stereo,
22050/44100/48000Hz, <=8MiB raw, <=10s, <=16MiB decoded normalized stereo f32 PCM.
RIFF/container/CRC/packet spans, dimensions, multiplication, setup/VQ/VLC and
predicted working frames are checked before decoder construction/allocation.
Decode is bounded block-by-block and actual lengths/PCM are checked afterwards.
Pinned Vorbis lookup1 setup arithmetic not representable in checked integer
prediction is `audio_unsupported`; invalid data and exceeded budgets are
`audio_invalid` and `audio_resource_limit`. The bounded first/last Ogg valid
interval removes declared padding, including one-page first+last audio data;
meaningful samples are retained. No arbitrary Vorbis configuration is promised.
Import/decode is synchronous and timed separately from warm VM/native mixing.

Scene7 declares <=8 named cues, each referencing an accepted audio identity and
nonempty half-open `[start_frame,end_frame)` interval, finite gain `[0,1]`.
Names are unique/nonempty/<=128bytes. Multiple cues share one immutable PCM owner.
`ag_audio_emit(ordinal int,cue int,gain float) int` costs1credit, permits <=4requests
per callback and <=8simultaneous voices/Run. Ordinals are exact signed integers
in1..2147483646, strictly increasing across the Run and in each packet. Exhaustion,
duplicate/stale ordinal, bad identity/span/gain or the ninth voice refuses the
whole candidate, with no stealing/dropping/unbounded queue.

Wire Run/tick/bundle/count/ordinal/cue are typed integers (not round-tripped through
f32). Four fixed request rows accompany the whole logical candidate; unused rows
are canonical zero. Host independently validates identity/domain/capacity and
prepares the complete visual/resource candidate before common logical accept.
Only that accept inserts voices once. Exact latest transport replay is acknowledged
without reaccepting or inserting voices; older/replaced tick or old Run is rejected
without effects. Render/capture/multiple portals never resubmit sound.

The native callback and offline oracle share the same bounded mixer, linear
resampling, multiplied cue/request gain, summed/clamped stereo mix. Effect begins
at the next available service quantum after acceptance. Hardware already-buffered
sound cannot be withdrawn; visual/audio hardware simultaneity is not promised.
Preaccept failure keeps last accepted Frame/GPU image and adds no voice; Fault
cancels the Run's already-existing voices. Already emitted samples are not rolled
back. Postaccept native output error produces `audio_fault`, clears voices/queue,
preserves last accepted visual, and cannot retry/replay automatically.

Host owns one native manager, retained across healthy Stop and replaced (old
stream dropped before opening a new stream) after output fault, including a round
restart. Host final close drops it. Pause freezes voice cursor; Resume continues
existing valid tails. One already-dispatched candidate may finish under Pause and
its new voices remain frozen. Neutral Step is silent, marks `suppressed_by_step`
and consumes ordinals with no new voices/backlog. Stop/Fault/new Run yields no old
Run PCM from the next service quantum; new Run clears input/request/ordinal state
and old controls reject. Legitimate Editor/history/external static raw/PCM pins can
outlive a Run; they do not retain a playable Run owner. Actual VM/voice/manager/PCM
weak probes, 20cycles per dimension and warm decode/upload0 distinguish lifetimes.

Executable evidence: `public_audio`10, `public_audio_codec`6,
`public_audio_resources`9, `public_audio_host`8, `public_audio_consumer`2,
`public_audio_compat`1 and5selected `--lib audio` units. The fixed source/runner
gate is `tools/check_public_audio.py --controls` (final review/merge-owned), with
`verify_plan012.py --controls` and an exported A→B cold SDK/target offline consumer.
Reports bind actual executable/source hashes, counts, observations and both
dimensions. Empty/partial/old runner proof is rejected. Per-module scoped work
receipts and the final gate must not be confused.

## SD-02 modify docs/specs/auto-graphics/scene-assets.md

Add scene7 beside existing scene versions: audio AssetRef persists original raw
SHA256/length/format (`ogg`/`wav`) and `{rate,channels,frames}`; document `cues` is
explicit, including `[]`. Older schemas reject these new fields/formats. Declared
metadata is verified against raw/decoded bytes even when a same-content cache is
warm. Only generated `assets/<sha>.ogg|.wav` paths are implied. Safe local paths,
ancestor reparse refusal, blob-first/entry-last/save/readback/repair/SaveAs rules
extend to audio. Save As uses accepted bytes, never the import path.

All current/history/pending-save/candidate resources share unchanged8raw identities,
32MiB raw and64MiB CPU/GPU budgets; disk assets remain8files/32MiB. PCM/raw resident
owners, bounded decoder scratch/overlap windows, cue/request/voice metadata and
native observation ring/manager join the same CPU envelope, with no free global
cache. Multiple cues do not duplicate PCM. CPU pose reservation remains separately
added to the same64MiB total. Unknown/foreign residues are preserved, not silently
deleted. Damaged readback cannot confirm save; actual repaired cold reopen must
succeed before claiming closure.

The auditable derived bank is PCM16 stereo44100Hz,22504frames/90060bytes,
SHA256 `b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d`.
It concatenates unchanged jump7403mono, coin12643stereo, land2458stereo samples;
mono duplicates channels, with no resampling/clipping/gain edit. Quantization
error<=1/32768. `samples/public_audio/bank-audit.json` binds original hashes and
spans; `examples/build_public_audio_samples.rs` reproduces it via production decode
and public project save. The real4models+sharedPNG+bank use6raw identities and
236544raw bytes. The same production budget accepts a seventh26828byte font slot
with conservative4MiB CPU/GPU reservation; actual font import/render belongs to
013 and is not delivered here.

## SD-03 modify docs/specs/auto-graphics/behavior-api.md

Add explicit profiles audio2d(version111/profile22) and audio3d(112/23); all15old
profiles keep exact combined/API/bridge bytes. Existing prices and32credit,
1000steps/slice,100000steps/callback,120resumptions/250ms,30sinitialization/2stick,
256/96/352KiB user/managed/combined source limits remain unchanged. Host8ms pump
is a measured target reported met/missed separately from hard deadlines. VM/output,
admission, Host prepare/publish/pump and native quantum CPU/deadline observations
are separated; CPAL backend underrun counter/hardware latency are unavailable,
not assumed zero. Native mixing cost is not hidden as asynchronous VM work.

`ag_actions_snapshot()` costs1credit. 2D held bits0Left/1Right/2Jump/3Restart,
bits4/5zero; 3D inherits006six-action ordering. Return is held+64*pressed; pressed
is consumed exactly as in the existing getters, including mixed reads.

`ag_present2d(kind,entity,x,y,w,h,flip,n,o0,c0,g0,...o3,c3,g3)` and
`ag_present3d(kind,entity,clip,phase,looping,yaw,n,o0,c0,g0,...o3,c3,g3)` each return
int and cost1credit. Arguments preceding gains are int except3D phase/yaw float;
each request triple is `(ordinal int,cue int,gain float)`. Exactly one bundle per
callback, at most one visual plus4sounds; kind−1 has no visual,0is bound leaf,
1is runtime slot. Independent audio emit mixed with bundle, duplicate bundle,
duplicate visual target or standalone visual mixed with bundle rejects. Standalone
visual plus standalone audio remains legal at their ordinary prices.

The complete real event branch keeps snapshot1+move26+grounded1+event-next1+
event-pair1+event-phase1+bundle1=32credits. Both dimensions actually consume all
fields, move the actor, select another cue after event101, and accept visual/audio
together; adding one charged call fails. No getter discount, action reordering,
omitted event field or raised cap makes the sum pass.

## Downstream G12a / ag-examples005 handoff

Engine fixtures `samples/public_audio/project2d/scene.agscene` and `project3d/...`
are ordinary bound projects. Normal entry:
`cargo run --locked --offline --bin autographics-authoring -- --open <scene.agscene>` then Run;
temporary override uses `--behavior <audio2d.at|audio3d.at> --behavior-api audio2d|audio3d`.
The platformer art/sounds are CC0 per retained upstream README; upstream code MIT
is retained separately. The generated two-color2D PNG is original CC0 test art.
No font license is inferred from either. Projects, bank, scripts, image/GLB blobs
and provenance travel with the independent engine source consumer.

These are finite engine fixtures, not Gallery content, product ZIP, complete
G2-09/G3-08 games, G12 registration, public HUD or production/multiplatform support.
ag-examples005 must bind this actual delivered engine revision and independently
verify real level event/selected-cue/gain/once-only/Stop behavior after001/002/003
content changes, together with013font/HUD delivery. Do not upgrade011or the full
AG milestone from this engine gate alone. Merge records the actual delivery SHA
and canonical paths; this proposal contains no speculative delivery hash.
