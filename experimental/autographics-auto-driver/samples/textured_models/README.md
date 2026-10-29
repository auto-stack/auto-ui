# Finite textured model input fixtures — PLAN-005

These are unmodified originals from Kenney's Starter Kit 3D Platformer,
Git commit `3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`:
https://github.com/kenneyNL/Starter-Kit-3D-Platformer

`SOURCE-README.md` declares the included models and images CC0;
`SOURCE-LICENSE.md` preserves the upstream code MIT notice separately.
This fixture set does not include a game or advertise animation support.

| Input | Original SHA256 |
|---|---|
| models/platform.glb | 78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c |
| models/coin.glb | a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5 |
| models/flag.glb | 0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae |
| models/Textures/colormap.png | 106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3 |
| models/character.glb | 7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1 |

The first three are required static textured controls. `character.glb` is a
deliberate unsupported-animation control and an unchanged source for PLAN-006;
its static/idle/walk/jump clips have not been removed to fabricate acceptance.
Tangents are checked as finite FLOAT VEC4 declarations but do not enable normal
mapping. UV0 and KHR_texture_transform determine base-colour sampling.

The original sampler minFilter 9987 is explicitly normalized to base linear
filtering at LOD0 in this finite, single-level profile; this is not mipmapping.
Raw root identity stays exact; a separate closure identity binds external PNGs.
The public import command/UI resolves relative PNGs once and project saves use
accepted immutable SHA blobs rather than the original import directory.

Run the first complete public path from the repository root:

```powershell
cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --locked --offline --test textured_model_resources fixed_sources_public_import_save_move_reopen_and_actual_gpu -- --test-threads=1 --nocapture
```

This test imports all three through the public stamped Host command, saves,
moves the project, reopens and compares actual GPU captures. It also changes
only the external image and requires both a new closure and changed pixels.
