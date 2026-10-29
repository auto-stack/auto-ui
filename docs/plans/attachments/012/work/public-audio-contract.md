# PLAN-012 T-02 public audio execution contract (in progress)

**Current status: T-02 frozen/pass at `ef438f9` (2026-10-30), see the current
closure at the end.** Earlier foundation observations below are historical;
in particular the padded Vorbis lengths are superseded by the independent
gapless interval evidence. T-03..06 still own full feature acceptance.

Baseline: auto-ui master `f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131`,
006 delivery `c681663e6ff392d70471abbec560e9d9daf12622` verified ancestor.
One tree `D:/autostack/.wt/ui-012/auto-ui`, branch `codex/plan-012-dev`.
SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`, 681 frozen files,
17 local packages/29 path edges/479 metadata packages; SDK and both existing
dependency locks initially unchanged. No Gallery or dependency code write.

## Entry and prerequisites

The user `/goal 完成UI的计划012` on 2026-10-30 authorizes the existing full
work/review/bounded repair/merge/archive/guard cleanup chain. Planning-only
restrictions in historical r1 receipts no longer prevent authorized execution.
Scope, acceptance, limits and native-output requirement remain unchanged.

006 is archived 10/10 with independent final R2 pass on a85980c, equivalent
rebase delivery c681663, fresh frozen34/34 gate, knowledge commit664145b and
cleaned receipts. Archive and source read directly, not inferred from a checkbox.
On pristine 012 tree, `verify_plan006.py --controls` exit0 verifies1477 hashes,
edits/deletions/extra input/wrong pin/path escape/raw LF-CRLF negative controls;
`prepare_sdk.py --from-snapshot` and `verify_source.py` exit0 verify actual SDK
and lock closure. Baseline source hash is e2994efa1b1ee6d55f79de7bd194bf59c1745c41f60b6c8bbaac7a67197e7243.
These establish dependency input only, not new012 acceptance.

## Profile / wire / price decisions

New explicit `autographics.audio2d/1` (CLI `audio2d`, version111/profile22)
derives from delivered sprites2d; `autographics.audio3d/1` (CLI `audio3d`,
version112/profile23) derives from delivered node-animation3d. Old15 assembled
profiles and source files remain unchanged. The registry had no111/112 or22/23
before this work. New selectors do not yet imply complete Host delivery.

`ag_audio_emit(ordinal int, cue int, gain float) int` costs one shared credit.
Ordinal1..2147483646 strictly increases across the Run; cue0..7 must exist in
the frozen accepted document; gain finite0..1. No integer is routed via f32.
Max4requests/callback, max8simultaneous voices/Run. Voice overflow rejects the
candidate; no stealing or pending audio backlog.

`ag_present3d(kind,entity,clip,phase,looping,yaw,n,o0,c0,g0,...,o3,c3,g3)`;
`ag_present2d(kind,entity,x,y,w,h,flip,n,o0,c0,g0,...,o3,c3,g3)` each costs one
shared credit, including its optional visual and0..4sounds. Argument types
match existing visual integer/float signatures; triples are int,int,float.
kind=-1 omits visual; kind0=document id; kind1=complete object token.
The fixed four triples avoid unbounded arrays or iteration. The n selected
triples must have increasing unique ordinals; unused triples have no effect.
At most one bundle/callback including empty; standalone audio + bundle rejects,
two bundles reject, repeated visual targets reject, bundle and standalone visual
mix rejects. A bundle helper bypasses only its own internal visual charge;
all public standalone calls retain their original charges. Any validation error
is sticky and rejects the complete candidate, not just the failed field.

Audio output block (to be wired in Play): four header ints Run,tick,bundle,count,
then count rows of ordinal int,cue int,gain float. Host checks exact Scalar types,
length/count, expected Run/tick, accepted cue identities and capacity before
accept. Same accepted logical packet replay is a no-op; different packet at the
same Run/tick or stale Run/ordinal fails. Exhaustion rejects, never wraps.

2D snapshot uses original Left/Right/Jump/Restart bit0..3; bit4/5 remain0.
3D inherits delivered Left/Right/Forward/Back/Jump/Restart bit0..5. Both return
held+64*unread_pressed with shared consume-once state, one credit per read.
Full two-dimensional input/move/ground/event/visual+audio activity and extra
charged-call negative control must execute before T-02 is closed.

## Actual producer/consumer path and owners

New proposed driver modules: `src/authoring/public_audio/{mod,codec,mixer,native}.rs`.
Existing integration owners remain assets/document/storage, behavior/assembly,
play Frame and host/play_adapter + viewport. Tests: `tests/public_audio.rs`
(producer), additional codec/storage/Host/lifecycle/consumer test entrypoints
named when enabled, tools/check_public_audio.py + verify_plan012.py.

Raw bytes → container/span/codec setup prediction → bounded decode → immutable
Registry audio payload → versioned cue refs → real Auto requests → independent
Host validation/voice-capacity prepare → common logical accept → bounded mixer
voice insertion exactly once → this service's native callback PCM. Raw/decoded
owner is shared across cues and warm voices. Manager belongs to Host and outlives
Stop; voice/Run command owner does not. All retained and temporary raw/PCM/setup/
queue/voice data must be reserved within existing64MiB CPU, not a separate cache.

Native backend will use cpal0.15.3 (already transitively locked by kira0.9.6),
with the service's bounded mixer directly inside the output callback. This gives
one controllable quantum and an observation point on actual submitted PCM;
no microphone/system capture. Opening a default output is normal Run behavior
for audio profiles; unavailable/fault is named, never a successful mock fallback.

Pause freezes voice cursors; Step consumes ordinals with suppressed_by_step and
no new voice; Resume continues only existing tails. Stop/Fault/newRun invalidate
old voices/queue at the next service quantum. Already-buffered hardware output
is explicitly outside rollback. Preaccept failure preserves complete last-good;
postaccept output error retains accepted visual, faults audio, clears this Run
and does not retry. Host final drop releases manager/stream.

## T-02 native prerequisite observation

Local source/EXE retained at
`D:/autostack/plan-artifacts/auto-ui/plan-012/native-preflight`.
`cargo run --offline --manifest-path .../native-preflight/Cargo.toml` exit0.
Default `扬声器 (ToDesk Virtual Audio)`, stereo44100Hz F32;35 actual output
callbacks,3527 nonzero frames,peak0.014999996,max callback24us,deadline_misses0
for a finite80ms quiet440Hz tone, followed by silence. Device is a virtual output;
this establishes native stream availability only, not physical speaker sound,
public Auto integration, cue correctness or lifecycle acceptance. Callback reads
only its own generated PCM. Full service output proof remains T-04/T-05.

## Outstanding T-02 closing conditions

2026-10-30 foundation checkpoint:9 directed tests passed (`public_audio`6,
`public_audio_codec`3,0failed/0filtered), log
`D:/autostack/plan-artifacts/auto-ui/plan-012/t02-current.log`, SHA256
`540b86452b6126274d6cfabed8b4bec969c12c4a7b1e4a9377973bea9a7b656`.
Current full2D/3D activity at tick2 observes actual event101, valid32calls
status0; the same branch plus one charged input read status10. Managed source
85055B/84143B, below98304B. Initial probe mistakes (wrong row count/missing
character-camera binding/overlapping template solids) were fixed in the
fixture; the final scene actually moves before testing the full26cost path.
Six producer cases include2D bits0/1/2/3/15, pressed consumption, gain/cue/
ordinal/mode conflicts, preaccept failure/noPCM, replay/noextra voice and real
Auto→typed packet→common accept closure→shared mixer→native stream. Native
latest callbacks1/nonzero970/peak0.4/max135us/deadline_misses0/quantum970;
stereo samples match[0.1,-0.4]. This is not yet the authoring Host closure.

Codec foundation now exists at `public_audio/codec.rs`. RIFF spans/alignment/
rates, Ogg CRC/serial/sequence/packet/granule/header and expansive codebook/VQ
prediction precede decoder construction. Codeword prefix counts bound pinned
Symphonia8bit VLC tables instead of a falsely excessive all-disjoint-block
estimate; fixed floor/residue/window/MDCT scratch reserves12MiB conservatively.
Actual source setup reservations: jump27276448B,coin/land27141808B. Actual
decoded frames: jump7872mono,coin12864stereo,land2496stereo,all44100Hz;
retained stereo float bytes62976/102912/19968. Channel/domain/CRC/codebook
dimension/entries/blocksize/duration mutations reject before decode counter
increment; legitimate CRC-recomputed control decodes. Ogg granules are
presentation timestamps rather than exact decoder output length: jump granule
7403 vs actual7872. Preflight reserves one extra max8192sample window capped
at10s, then checks actual finite frames against that envelope and10s; no samples
are silently discarded to satisfy the timestamp. Actual count becomes the
decoded header/cue bound. PCM peak reservation includes Vec→Arc copy.
The pinned decoder's unrepresentable lookup1 integer arithmetic is currently
named audio_unsupported before construction, and needs explicit supported-domain
review in T-02; this is not a claim to arbitrary Vorbis decoder support.

Only direct references to already-locked cpal0.15.3 and Symphonia0.5.5 were added;
no package version changes, AGP lock unchanged. Package-only fmt and diff check
pass; SDK verify-only681passes after formatting, SDK146existingwarnings retained,
no new delivery warnings observed. `cargo fmt --all` attempted dependency
traversal and refused nested frozen SDK workspace before edits; package-only
format is the correct entry. Canonical Specs/ledger/identity publication pending.

1. Compile/run real2D/3D Auto bundle, input sensitivity, event branch,32/33 and
   conflict controls; independently check old15assembled source bytes.
2. Container/setup preflight before potentially expansive Vorbis codebook/VQ
   allocations, bounded per-packet decode, actual fixed source Ogg/WAV samples
   and malformed/overbudget controls. Existing kira from_cursor is insufficient:
   Symphonia0.5.5 codebook.rs allocates entries and entries*dimensions, so raw
   size or final PCM checks alone cannot close the memory invariant.
3. Real Auto packet through prepare/accept to this service mixer PCM/native
   output, simultaneous visual path and resource ledger envelope. Production
   T-03..06 cannot be declared enabled merely from the isolated preflight.

All remain open until actual evidence is recorded. No user decision or native
environment blocker discovered. Targets8ms pump, callback limits and audio
deadline are reported separately; no full AG/root regression is run here.

## Current T-02 closure — ef438f9

All three T-02 closing conditions above are now met; this is not AC01..05 final
acceptance. The committed development implementation is on the single
authoritative tree. No canonical Spec, ledger, final identity or Gallery write.

### Frozen resource/project and effect contract

- `autographics.scene/7` requires explicit `cues` array (including empty).
  Older schemas reject cue fields/audio metadata/audio formats even if empty,
  and keep their old encoded bytes. Audio asset identity contains raw SHA,
  length, `ogg`/`wav` format plus `audio:{rate,channels,frames}`; image dimensions
  and model closure are absent. Cue fields are name, asset key, start_frame,
  end_frame, gain; <=8 unique nonempty names of<=128bytes, valid half-open range,
  finite gain0..1. All counts/rates/ranges persist as exact integers.
- `public_audio::import` uses the shared bounded RawPool and accepted immutable
  Registry. PCM is normalized stereo float; mono duplicates its channel. A bank
  resource is shared by all cue Arcs; no per-cue PCM copy or hidden decode cache.
  Encoded audio, actual PCM/asset headers, conservative manager/observation/
  cue/packet/voice space join the existing CPU account. Service reserve is
  conservatively charged per audio asset; this is not a claim of multiple actual
  Host managers. Imports/history/pending/current still share the8/32MiB raw pool.
- Host now supplies cue count before app_start. Play reads a fixed16-scalar
  block: Run,tick,bundle,count then four typed ordinal/cue/gain rows. Count0..4;
  unused rows canonical int0,int0,float0. Active rows go to independent typed
  admission and mixer resource/voice prepare before `Play.accept`; only then can
  bounded voice insertion occur. All GPU preparation and deadlines precede it.
  Real startup opens the native manager for these profiles without a probe flag.
- Public `import-audio` and `audio-cues` stamped edit commands use normal Editor
  candidate/GPU/accept history. The panel has Import Audio beside the local asset
  path. Save As uses accepted bytes and emits canonical SHA.ogg/SHA.wav blobs;
  closure verification, disk quota, reserved names and explicit GC recognize the
  new formats. No import directory reads during playback.
- `audio_disposition` on accepted runtime Frame records played/suppressed_by_step/
  replay. Neutral Step consumes ordinals, inserts no new voice, and Resume has no
  backlog. Existing in-flight candidate at Pause may finish under inherited Play
  semantics: insertion stays paused under the same mixer lock, with no callback
  observing a temporary unpause. No later callback is dispatched while Paused.
  Host retains manager on Stop; mixer clears Run/voices/packet and retains Run
  high-water so old begin/control cannot reactivate a stopped Run. A terminal
  Fault does not re-enter mixer controls and overwrite its causal diagnostic.
  Next real native quantum after Stop observes Run0/nonzero0. Hardware already
  buffered is outside this boundary. Full capacity/fault/20cycle owner tests are
  still T-05.

### Codec interval correction and independent oracle

The round1 sample counts7872/12864/2496 included decoder block padding and are
not the final clip intervals. Source inspection of pinned Symphonia Ogg
`logical.rs::inspect_start_page/inspect_end_page` established a first-and-last
audio page delay ambiguity. We now decode raw codec blocks and explicitly apply
the bounded container interval: leading delay only for a multi-page start;
first-and-last audio page tail padding is not misclassified as start delay.
Positive start timestamp is subtracted; final length must fit the decoded
interval with at most one declared max block window of trailing padding.
Temporary leading/trailing windows are predicted before allocation and charged;
the final retained PCM still obeys10s/16MiB. Ogg reserve uses8raw lengths (codec/
demux/packet/extra-data/copy peaks) plus bounded PCM copies and setup scratch.
CRC/sequence/serial/continuation/granule/header/codebook/VQ prediction remains
before decoder construction. Nonmonotonic granules are invalid, over-duration
resource_limit. Lookup1 arithmetic unrepresentable by the pinned decoder remains
explicit unsupported before construction (finite supported codec-setup domain).

Independent local oracle is soundfile0.13.1/libsndfile1.2.2, not another call to
the production decoder. All15 actual full PCM comparisons pass: three original
Oggs plus22050/44100/48000Hz × mono/stereo × single/multiple audio page generated
controls. Shape/rate agree; largest absolute sample difference1.788139343e-7
(threshold2e-5), largest original RMS3.312014556e-8. Correct source lengths are
jump7403mono,coin12643stereo,land2458stereo at44100Hz. Report is local-only at
`codec-oracle-1793304858143097800/report.json`, SHA256
`f5b5a439dd8e82bad9729e6d3b748247defe23fe7d58c9f8207a4826cf5da476`.
The small committed `research/public-audio-codec-reference.json` is a permanent
test input (raw SHA + independently expected length and67 sampled stereo points
per original), not a log/report replacement. It rejects the pre-fix padded
implementation and altered source; full reference PCM/tool installation remains
local. `examples/audio_decode.rs` is the bounded export entry for replay.

### Real composition and resource measurements

The14 integration cases (public_audio6/codec3/compat1/Host4) pass,0failed/0filtered.
Full2D/3D producer tick2 has actual event101 and32credits status0; extra charged
read on that branch yields error10. Managed85055/84143B; callback/tick hard guards
unchanged. Host tick2 actually uses the read event fields to select the other
cue, with shared motion/camera/visual/audio frame. Original Ogg PCM reaches the
native stream; duplicate rendering has no audio hook. Bad33rd call or injected
GPU prepare failure preserves last-good full frame and GPU capture, stops voices
under the Fault policy, and repeated pumps preserve the causal Fault text.

Current final native positive:44100Hz stereo, quantum441frames,2D max83us/3D
max78us,0deadline misses, nonzero441frames observed. Host/VM8ms pump goal and full
T-06 CPU matrix remain unmeasured here, not inferred from native deadline. 2D
resource CPU249993B/GPU256B/raw12567B/3identities; 3D CPU including pose1381208B,
GPU1101700B/raw110157B/4identities. Warm new ticks add0decode attempts and0mesh
uploads. Native manager reservation33504B, actual one manager per Host.

The additional bank/union unit case passes (1selected/33 unrelated filtered).
Actual derivation retains22504 valid source frames, stereo PCM16 WAV90060B,
SHA256 `b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d`.
Cue ranges[0,7403),[7403,20046),[20046,22504); quantization error<=1.525878906e-5,
no resampling/amplitude editing/clipping of out-of-range source. Its three public
cue refs resolve to exactly the same immutable PCM Arc. Four actual original
models plus their one shared colormap give5raw identities; bank adds1, actual
raw236544B/CPU1448422B/GPU1145956B. The same production candidate budget accepts
the frozen font raw length26828B as seventh identity with conservative4MiB CPU/
4MiB GPU future reservation. This does not pretend012 imports/renders fonts;
013 owns that new resource service and final combined consumer.

Legacy byte compatibility was captured by compiling the actual pristine
f7c83c6 assembler on the same tree, adapting only three exhaustive Rust match
arms for never-selected new enum variants, then restoring current source byte
for byte in finally. All15 earlier SourceSnapshot combined/api/bridge hashes
and version/profile numbers match the permanent baseline. Capture receipt
`legacy-capture-receipt.json` SHA256
`972ad057108264f3f4b9c0738e8cd5fb82402b7b3e9f657a3899f192a6c64092`;
no second checkout/tree/agent was created.

### Evidence anchors and remaining stage owners

All paths below are relative to local
`D:/autostack/plan-artifacts/auto-ui/plan-012`:

| Command/result | Log | SHA256 |
|---|---|---|
| cargo test --locked --offline --test public_audio --test public_audio_codec --test public_audio_host --test public_audio_compat -- --test-threads=1 --nocapture;14/14 exit0 | t02-integration-frozen.log | 2d17ea34091cc47e10ddf85c1d1f5d938f69a116433917033fd4e9374d718268 |
| Final Run watermark replay + audio Host affected replay10/10 exit0 | t02-final-audio-host.log | f5a7d9bb65c309384b1520c97f70a1c3c34ab324d9b6f6b9412fcb8deb9199e1 |
| Final no-dummy-counter manager/CPU metric case1/1,3filtered exit0 | t02-host-final-metrics.log | c52523438c6d0b9e77b6406924f84dd67093adc356bf2ff59276cfb80195927e |
| cargo test --locked --offline --lib fixed_three_cue_bank -- --test-threads=1 --nocapture;1/1,33filtered exit0 | t02-bank-final-metrics.log | 364a06b36954c3b50d3efa15fb31fc43e2cc7c9be3b9e92b028dd3c28cbbbdc4 |

Tests are scoped/reused by affected behavior, not repeated as a root/fullAG gate.
Package fmt/diff checks and frozen SDK681 verification pass; SDK146 existing
warnings remain, no new delivery warning. Driver/AGP lock versions are unchanged.
T-03 still owns exhaustive cue/version/identity/budget/save failure and durable
repair closure. T-04 owns full request/admission/PCM/native replay controls,
T-05 capacity8/9, postaccept device error, complete lifecycle20cycles/weak owners,
T-06 fresh bound2D/3D consumer/identity/CPU/source sensitivity/SD delivery. All
remain open until their own evidence; review/merge gates likewise pending.
