# Public audio engine fixtures (PLAN-012)

These are finite engine verification inputs, not a Gallery/G12 release.
Explicit audio2d/audio3d profiles use ordinary authoring Host acceptance and
scene/7 cue declarations. Open `project2d/scene.agscene` or `project3d/scene.agscene`
with `cargo run --locked --offline --bin autographics-authoring -- --open <absolute-scene-path>`,
then Run. The accepted project scripts decide motion, events, visual and sound;
native output is enabled automatically. Missing output device fails by name.

Unmodified sound sources are from `kenneyNL/Starter-Kit-3D-Platformer`, commit
`3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`. `SOURCE-README.md` is the original
README declaring the art/sound assets CC0; `SOURCE-LICENSE.md` is the separate
original code MIT license. No font is included or licensed under that MIT text.

| Original relative path | Bytes | SHA256 |
|---|---:|---|
| sounds/jump.ogg | 5805 | f090c16689e9ec04c4bef80bfcbf68bc4b63ab4ce29f7fbb5cee05a5641d5694 |
| sounds/coin.ogg | 6620 | df55e931e6b78e5aecc015922e873a0e8bfa4e97cfb19abc64149d54afe89216 |
| sounds/land.ogg | 5885 | 145a1c17125dc4d344ac135bc5073e35ea12e19700ca30ee6e1e477010b4dca0 |

Source bytes are retained unchanged. The two bound projects share the derived
PCM16 bank described in `bank-audit.json`: 22504frames, 90060bytes, three immutable
cue spans. It duplicates mono to stereo and quantizes without resampling/clipping;
original hashes/frame counts/quantization errors are audited. Reproduce to a NEW
directory with `cargo run --locked --offline --example build_public_audio_samples -- <destination>`.
The 3D project contains the four original GLB models and their shared PNG.
The 2D project uses original generated two-color test art (CC0). Each project
contains its own accepted scripts/assets and retained license texts.

`public_audio_consumer` verifies relocation/cold reopen, source-sensitive chosen
cue/gain/motion and full PCM against an independent PCM16 reader; it also reports
the unchanged budgets and VM/Host/native costs separately. `tools/fresh_public_audio_consumer.py <new-directory>`
exports A, moves to B and cold-prepares/recompiles offline without producer build
outputs. `verify_plan012.py --controls` checks current source/SDK/locks/projects;
the frozen final review/merge gate is `check_public_audio.py --controls`.
Final delivery is recorded by the Plan's archive/merge receipts. These fixtures
do not constitute a product ZIP, Gallery/G12 registration, HUD or full-game acceptance.
