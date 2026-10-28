# Self-authored static geometry fixture

two-parts.glb is original test geometry authored for PLAN-838, MIT licensed.
No third-party model, texture, game asset, converter or network input is used.
Two independent nodes use identically named meshes: a red triangle at x=-1.5
and a green triangle at x=+1.5, both y=0..1.5, normal +Z. Each has three u16
indices (six source index bytes), exercising aligned GPU tail padding and
distinct expanded primitive keys. The combined X bounds are -2.2..2.2.

`python research/asset-probe/generate.py --check` verifies deterministic bytes.
Raw test/build/native outputs in this folder are ignored; work receipts preserve
them under the shared Plan attachments. This fixture proves only the declared
static flat-color subset, not glTF/PBR/material/animation conformance.
