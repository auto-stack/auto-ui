# Finite node-animation fixture

Open `scene.agscene` and Run its bound `autographics.node_animation3d/1` script.
The normal action/orbit routes drive movement, jumping and camera-relative
visual facing. Auto selects original idle/walk/jump clips and advances phase;
the Host validates and samples the finite node TRS. This is an engine fixture,
not a complete game or Gallery release. Static clip0 remains available through
the same public setter.

CLI: `autographics-auto-driver --open samples/node_animation/scene.agscene`.
An explicit source override uses `--behavior samples/node_animation/main.at
--behavior-api node-animation3d` with that same entry.

`python generate_project.py --check` rechecks the deterministic stored project.
`assets/` contains exact originals; `scripts/` contains the content-addressed
Auto source. SaveAs and cold reopen use those project bytes. The generator
reads fixed inputs from `../textured_models/models/`, never a network source.

Original `character.glb`: SHA256
`7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1`,
87624 bytes, 8 nodes, 4 clips, 44 LINEAR channels, no skin. Shared PNG SHA256
`106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3`.
The raw GLB and its URI strings remain unchanged. The original CC0 asset and
MIT sample-source provenance, upstream commit
`3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0`, and license texts are preserved in
[source receipt](../textured_models/SOURCE-README.md) and
[licenses](../textured_models/SOURCE-LICENSE.md). This engine fixture's Auto
controller is newly authored; it does not claim to port the complete sample.
