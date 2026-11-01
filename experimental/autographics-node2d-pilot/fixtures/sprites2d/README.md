# fixtures/sprites2d — PLAN-022 fixed G09 inputs (read-only)

Fixed read-only copies of the original ag-examples G09 assets
(`examples/g09-sprites-and-frames`, ag-examples PLAN-002, CC0-1.0):

- assets/95a08525….png  atlas 96x32 RGBA8, 258 B (3 frames 32x32)
- assets/8fde96f66….png character 160x32 RGBA8, 735 B (5 poses 32x32)
- licenses/<sha>.LICENSE  full CC0-1.0 text per image
- manifests/{atlas,character}-asset-manifest.json  origin provenance
  (creator, generator recipe paths+sha, PNG facts, named frame rects)

Rules (Plan §4/§5.3): never regenerate or redraw; the bytes are identity.
Every product fixture copies these bytes under its own assets/ unchanged;
verify_plan022 controls the sha chain back to this directory.
