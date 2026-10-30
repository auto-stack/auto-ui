# PLAN-013 proposed canonical Spec delta and G12b handoff

Status: work proposal for independent review and merge. This is not a published
canonical Spec or a claim that Gallery G12b is delivered.

## SD-01 — add `docs/specs/auto-graphics/public-hud.md`

Add a finite public HUD contract for the independent Windows authoring engine.
Explicit `autographics.hud2d/1` (`hud2d`, version113/profile24) and
`autographics.hud3d/1` (`hud3d`, 114/25) extend delivered audio2d/audio3d.
Scene8 carries one fixed font and exactly four HUD slots. Each slot persists
`visible`, `text`, `anchor`, logical-pixel `offset`, `size` and `rgba`; Auto
emits the complete visible/text state of all four slots, never a partial delta.

Public `ag_hud_set(slot,visible,text)` costs one shared credit. New-profile
`ag_present2d` and `ag_present3d` keep the audio visual+sound prefix and append
four `(visible int,text str)` pairs. One bundle costs one credit and includes at
most one visual, four sounds and all four HUD slots. Standalone HUD writes mixed
with a bundle, duplicate bundles and the existing mixed-write forms reject the
whole callback. The full event branch remains exactly32 credits; a 33rd charged
call rejects. Old17 profiles, signatures, prices and assembled bytes remain
unchanged.

Text is0..32 visible ASCII bytes32..126 per slot and at most128 total. Hidden
and visible-empty states are explicit. Fixed four34-scalar rows follow the
16-scalar audio block: visible,length and32 integer codes, with canonical zero
padding. Host independently validates types, lengths, domains and the accepted
font/layout before common `Play.accept`. Auto produces gameplay and HUD state;
Host never derives them from events or scores.

Only audited Lilita One TTF (26828bytes, SHA256
`4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa`) is admitted.
Scene8 embeds the exact OFL/RFN license (4391bytes, SHA256
`31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b`). No system
font, arbitrary TTF, silent missing-glyph replacement, CJK/IME/rich text or
production UI component is added.

Atlas keys are `(font SHA, rounded logical px16..48, device ordinal)`, at most
four across accepted/history/pending/external owners. Equal sizes share one
atlas. Atlas sides are at most1024 and4MiB; overcapacity and missing glyphs are
named errors. Warm text/visibility changes do not rerasterize/reupload. All raw,
layout, glyph, vertex, atlas, PCM and service owners share the unchanged64MiB
CPU/GPU and8-identity/32MiB raw budgets.

HUD draws last on the game surface, in logical coordinates with a physical
scissor. World cameras do not move HUD anchors. Two-dimensional letterbox and
all four anchors use the accepted viewport; resize/DPI maps that logical space
without creating new font sizes/atlases. HUD does not enter letterbox bars,
editor panels or3D depth. Preaccept failure preserves the complete last-good
visual/HUD and inserts no new voice. Postaccept native output failure follows
012: named Fault, last accepted visual/HUD retained, already emitted samples
not withdrawn and no automatic replay.

Pause dispatches no candidate. Neutral Step publishes one visual/HUD tick and
consumes sound ordinals as `suppressed_by_step`, without a new voice or Resume
backlog. Stop restores editing defaults; fresh Run starts from document defaults
then `app_start`. Twenty-cycle owner/warm evidence distinguishes healthy Host
reuse from VM/Run release and legitimate final external pins.

Executable directed evidence: `public_hud`15, `public_audio`10,
`public_audio_codec`6, `public_audio_resources`9, `public_audio_host`12,
`public_audio_compat`2, `public_hud_consumer`2 and5 selected audio units. Final
`tools/check_public_hud.py --controls` binds source/runners, actual GPU/PCM,
owners and fresh independent consumer; partial/zero-selected/stale reports are
rejected. `verify_plan013.py --controls` protects complete source identity.
Merge still owns the frozen gate; this proposal does not mark it passed here.

## SD-02 — modify `docs/specs/auto-graphics/scene-assets.md`

Add scene8 beside existing versions. It inherits scene7 cues and adds required
`hud` with fixed font reference, exact embedded license and four unique slot
rows. Older schemas reject HUD fields and `ttf`; scene7 and earlier behavior is
unchanged. The only generated font blob is `assets/<sha>.ttf`, installed from
accepted bytes with the existing safe local path, no-overwrite, entry-last and
complete readback rules. SaveAs and playback never reread the import directory.
Damage rejects reopen and restoring exact bytes recovers it.

The four models plus shared image plus bank plus font use7 identities and
263372 raw bytes, within the existing shared limits. Font atlas/service owners
are not a hidden free cache. All current/history/pending/external atlas keys
share the same CPU/GPU budget. OFL and Reserved Font Name are preserved; the
font is not modified.

## SD-03 — modify `docs/specs/auto-graphics/behavior-api.md`

Document HUD2D/HUD3D selectors and appended bundle arguments, complete wire,
`ag_hud_set`, exact credit/conflict rules and full-slot retention semantics.
The trusted Auto unit converts strings to bounded ASCII integer rows; strings
do not cross the numeric Session ABI. Fresh Run and Stop/Step behavior follows
the common candidate. Existing audio profiles never acquire HUD APIs or the new
longer signature. Source/managed limits and VM deadlines are unchanged; Host
8ms pump remains a separately reported met/missed target, not the2s hard limit.

## ag-examples/006 G12b downstream interface

G12b may bind the actual delivered engine revision only after this Plan merges.
Use `samples/public_hud/project2d` and `project3d` as finite engine fixtures,
with bound Auto scripts, visual assets, three-cue PCM bank and fixed font. The
scene embeds the font license and each project also retains `FONT-LICENSE.txt`;
other art/sound provenance and upstream code license remain separate. A normal
entry is `autographics-authoring --open <scene.agscene>` then Run. G12b must
independently verify real glyphs, event/selected-cue/gain/once-only audio and
Stop behavior after its own content changes. It must not infer delivery of a
full two-level game, production HUD/UI, arbitrary fonts or a complete AG
milestone from this engine-only gate.
