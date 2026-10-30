# PLAN-013 T-02 public HUD execution contract (foundation; probe pending)

Status: T-01 dependency gate passed; T-02 design frozen for implementation, but
this task remains open until the real 2D/3D producer-to-GPU/PCM probes and
negative controls below pass. No acceptance is inferred from this document.

Baseline: auto-ui master `df39593b53412fed10055d18c0aab90b0590bd27`, branch
`codex/plan-013-dev`, sole tree `D:/autostack/.wt/ui-013/auto-ui`. PLAN-012 is
archived 10/10 with delivery `78632af39de28881618f9219d07e20a9b7a5b56f`, an
ancestor of the baseline. `verify_plan012.py --controls` exit0 verified 1536
source hashes and edit/delete/unlisted/wrong-pin/path/exact-source controls.
Driver lock SHA256 `4a0b3040fffa2523f3932f2ab2b427e3c13285abce751813147cfe28000d9b06`;
AGP lock SHA256 `c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf`.
SDK remains `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`, 681 frozen files.
No dependency or Gallery implementation change is authorized.

The user's `/goal 用 /auto-plan-goal 完成UI的计划013` authorizes work, independent
review, bounded repair and merge/archive/guard cleanup for this existing r1
contract. It does not relax any limit, review requirement or scope boundary.

## Profiles, public Auto operations and exact prices

Add explicit `autographics.hud2d/1` (CLI `hud2d`, version113/profile24) derived
from delivered audio2d, and `autographics.hud3d/1` (`hud3d`, 114/25) derived
from delivered audio3d. All17 earlier assembled profiles, public signatures,
source bytes and prices remain unchanged. The existing 32-credit,
1000-step/slice,100000-step/callback,120-resume/250ms,30s-start/2s-tick limits
and256/96/352KiB source envelopes remain. The8ms Host pump is a separately
reported target, never replaced by the2s hard limit.

New `ag_hud_set(slot int, visible int, text str) int` costs one credit. Slot is
0..3, visible0/1, text0..32 visible ASCII bytes32..126. It updates one Run-owned
slot; every candidate still emits the complete four-slot state. A standalone
HUD write mixed with a bundle, a second bundle, or any existing forbidden
visual/audio combination rejects the complete callback.

The new profiles keep the audio profile's visual+audio argument prefix but append
four `(visible int,text str)` pairs to `ag_present2d` and `ag_present3d`. The
complete call costs one credit, including at most one visual, four sounds and
all four HUD values. Each slot is explicitly visible-with-text (including empty
text) or hidden; there is no partial/delta form. The four pairs are processed by
trusted Auto code, not Rust gameplay. Existing audio profiles retain their old
shorter signatures and do not acquire HUD operations.

The representative real event branch must measure exactly
`1 snapshot + 26 move + 1 grounded + 1 next + 1 pair + 1 phase + 1 bundle = 32`
with nonempty visual, sound and fresh HUD data. One additional charged call on
that branch must fail. No getter/event/visual/HUD field may be removed to fit.

## Scene8, fixed font and shared resource account

Add `autographics.scene/8`, inheriting scene7's explicit `cues` and adding one
required `hud` object. Older schemas reject `hud` and `ttf`; scene7 behavior and
bytes stay unchanged. Scene8 has exactly one font binding and exactly four slot
rows with unique explicit slot ids0..3. Unknown fields, missing/duplicate rows,
bad references and noncanonical values reject. The HUD object also persists the
exact4391-byte OFL/RFN `license` string, so SaveAs/relocation carries the license
inside the scene rather than depending on the source font directory.

Each slot persists `visible`, `text`, `anchor` (0 top-left,1 top-right,
2 bottom-left,3 bottom-right), logical-pixel `offset:[x,y]`, `size` in16..48,
and finite `rgba` channels0..1. Text follows the same0..32 visible ASCII rule;
all four total at most128. Offsets and all derived coordinates must be finite.
The font and slot layout are accepted document state; Auto controls only the
complete visible/text state, never gameplay, camera or persistent layout.

Only the audited Lilita One TTF from Kenney commit
`3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0` is admitted:26828 bytes, SHA256
`4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa`, format
`ttf`. The OFL/RFN license (4391 bytes, SHA256
`31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b`) travels
with source fixtures and the independent consumer. No system font, arbitrary
TTF, font modification or silent glyph substitution is allowed.

Font import/readback verifies length and SHA before fontdue. Raw bytes share the
existing8-identity/32MiB pool and8-file/32MiB asset disk budget. Persist only
`assets/<sha>.ttf` using accepted bytes, blob-first/entry-last, readback and
repair rules; SaveAs and playback never reread the import directory. Font raw,
CPU layout/atlas scratch, glyph tables, vertices, GPU buffers/textures, PCM and
all existing resources join the unchanged64MiB CPU/GPU accounts. The feasibility
case is4 models+shared image+bank+font=7 identities, not an extra free font slot.

## Complete wire and independent Host admission

After the existing16-scalar audio block, append four fixed34-scalar HUD rows
(152 scalars total): `visible int`, `length int`, then exactly32 integer byte
codes. Length0..32; active codes32..126; every unused code is canonical0. Fixed
rows make missing, duplicate or unknown slot representations impossible; Host
still checks exact types, row count, canonical padding, visible0/1, lengths,
ASCII and total<=128 independently. Layout/font/identity comes only from the
accepted scene8 document, never from less trusted output.

Host config pushes document initial visible/text as bounded integer calls before
`app_start`; no strings cross the existing numeric Session ABI. Auto expands and
checks public strings internally. Unupdated slots retain their current complete
state. A fresh Run starts from document defaults then runs `app_start` once;
Stop republishes editing defaults. Old Run handles cannot mutate a fresh Run.

Admission order is: complete VM output -> independent document/font/HUD/audio/
visual validation -> atlas/layout/GPU/voice preparation -> one common
`Play.accept` -> publish Frame and staged GPU state -> one mixer insertion.
Any preaccept error preserves the entire last-good visual/HUD/audio state and
inserts no new voice. Postaccept native output fault follows012: named Fault,
last accepted visual/HUD retained, already emitted samples not rolled back.

## GPU layout, atlas ownership and viewport

Use the audited font bytes with a fixed ASCII32..126 glyph set. Atlas keys are
`(font SHA, rounded logical px size16..48, GPU device ordinal)`; at most four
current/history/pending/external-pin keys are retained. Slots with equal font
and size share one atlas. A key has one atlas of at most1024 per side and4MiB,
with checked packing and named capacity rejection rather than clamping. Warm
text/visibility changes do not rasterize or upload the atlas again. Atlas
creation/upload counters and weak owners distinguish fresh, warm and final drop.

Rasterization happens once at the document's logical px size. Layout uses
logical surface coordinates; the final orthographic transform maps that space
to the current physical target. Resize/DPI therefore does not create unbounded
font sizes or atlases. For 2D, the accepted camera letterbox computes the game
viewport and clip in logical coordinates; for3D it is the full game surface.
Anchors and offsets are resolved against that viewport, not the world camera.
The text pass is last on the game surface, after world/sprite/legacy overlay
content, with a scissor rectangle transformed to physical target pixels. No
HUD pixel may enter letterbox bars or an editor panel; HUD has no3D depth.

The public implementation uses a strict checked layout entry. The existing
TextRenderer fallback that advances unknown characters as spaces remains only
for legacy private games and is not a public-HUD success path.

## Lifecycle and required distinguishing oracles

Pause dispatches no candidate. Neutral Step publishes exactly one visual/HUD
 tick, consumes new audio ordinals as `suppressed_by_step`, creates no voice and
has no Resume backlog. Stop/new Run/round restart isolate HUD, VM, PCM, voice
and atlas device owners. Twenty cycles per dimension check actual weak owners;
legitimate Editor/history/external pins are distinguished from Run leaks.

Permanent tests and tools will include `tests/public_hud.rs`, affected Host/
consumer/compat tests, `tools/check_public_hud.py --controls` and
`verify_plan013.py --controls`; exact final case counts are reported only after
they exist. Raw PNG/PCM/log/fresh-consumer evidence stays local under
`D:/autostack/plan-artifacts/auto-ui/plan-013`; only deliberate Markdown
summaries/contracts are committed.

T-02 cannot close until all of these real probes pass:

1. Real2D/3D Auto executes the32-credit event branch and rejects33, with all
   output fields consumed and no Rust-derived HUD/gameplay state.
2. Accepted scene8 fixed-font import and full resource union fit; malformed SHA,
   TTF, license/closure, bad slot/ASCII/length/nonfinite/atlas cases reject before
   their respective decode/raster/upload side effects, with accepted controls.
3. Fresh GPU captures show actual0/1/2/3 andPlaying/Won/Failed glyphs in both
   dimensions; altered text/state and wrong/stale pixels fail the oracle. Real
   accepted PCM is observed for the same candidate; duplicate presentation does
   not replay it.
4. World-camera changes do not move logical HUD anchors; resize/DPI/letterbox
   clips at all four corners. Bad candidates keep the complete old image and add
   no PCM; postaccept device fault follows012.
5. Pause/Step/Stop/newRun and20-cycle owner/warm checks pass, then a source-only
   A-to-B consumer independently prepares SDK/target offline and reopens/runs
   the moved project. Old17 profile assembly and old scene compatibility remain.

No root/full-AG suite, production UI, CJK/IME, arbitrary fonts, full game,
Gallery implementation or new audio channel is included.

## T-02 closure and stage-owner correction

T-02 is closed at the real two-dimensional feasibility boundary in Plan task
T-02. The earlier checklist above incorrectly folded detailed T-03/T-05/T-06
acceptance into this entry gate. This is a stage-owner correction only, not a
reduction or deferral outside the approved Plan: exhaustive slot/resource and
nonfinite negatives remain T-03; postaccept device fault, resize/DPI, Pause/
Step/Stop/newRun and20-cycle ownership remain T-05; source-only A-to-B fresh
SDK/target consumption and final identity remain T-06. They are not checked off
here.

Passed evidence: `public_hud`6/6 and `public_audio_compat`2/2, zero filtered,
plus the preceding `public_audio`10/`public_audio_host`12 legacy replay. Real
2D/3D Auto executes the32-credit event branch and rejects33, produces dynamic
HUD text and sound in one bundle, and fits the fixed-font scene8 resource union.
Host independently admits complete HUD rows and publishes actual GPU glyphs and
native PCM; letterbox pixels stay unchanged and warm text does not rebuild the
atlas. Bad text/GPU candidates preserve last-good without a new voice. Scene8
save, corruption refusal, repair and relocated cold reopen pass.

All17 pre-HUD profile combined/API/bridge hashes equal a separately executed
pristine `df39593` assembler captured in `research/plan013-legacy-profiles.json`.
The baseline probe is local-only under `plan-artifacts/auto-ui/plan-013/legacy-baseline`;
it is not a second implementation checkout. The document decode scene8 routing
omission found by this probe was fixed and the complete test rerun passed.

Final command log: `D:/autostack/plan-artifacts/auto-ui/plan-013/t02-closure.log`.
No final gate, independent review, canonical Spec/ledger or merge is implied.
