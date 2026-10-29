# Public audio engine fixtures (PLAN-012, in progress)

These are finite engine verification inputs, not a Gallery/G12 release.
The development tree supports explicit audio2d/audio3d profiles, authoring Host
acceptance and scene/7 cue declarations. Full lifecycle, portable bound samples,
current identity and final acceptance gates remain in progress; this is not a
delivery receipt.

Unmodified sound sources are from `kenneyNL/Starter-Kit-3D-Platformer`, commit
`3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`. `SOURCE-README.md` is the original
README declaring the art/sound assets CC0; `SOURCE-LICENSE.md` is the separate
original code MIT license. No font is included or licensed under that MIT text.

| Original relative path | Bytes | SHA256 |
|---|---:|---|
| sounds/jump.ogg | 5805 | f090c16689e9ec04c4bef80bfcbf68bc4b63ab4ce29f7fbb5cee05a5641d5694 |
| sounds/coin.ogg | 6620 | df55e931e6b78e5aecc015922e873a0e8bfa4e97cfb19abc64149d54afe89216 |
| sounds/land.ogg | 5885 | 145a1c17125dc4d344ac135bc5073e35ea12e19700ca30ee6e1e477010b4dca0 |

Source bytes are retained unchanged. Decoder tests compare actual finite PCM;
public Auto tests currently use a separate distinguishable synthetic stereo
bank to measure exact cue/gain/accept/replay behavior, not these original effects.
No derived bank, product ZIP or full-game acceptance is claimed here.
