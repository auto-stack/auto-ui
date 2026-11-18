"""PLAN-058 T-03 control verifier (`--controls`, frozen by this task).

Groups (each prints PASS/FAIL with the observed numbers):
  C1  new fixture identity chain: autographics.scene/9, topdown2d/1, world sha,
      7 rows, exactly one `"layer": "render-only"` ground, identity.json agrees.
  C2  frozen legacy product bytes (021 queries-2d / 022 sprites-2d / atlas
      asset) still hash to the T-03 baseline: old profiles are not drifted.
  C3  no legacy `generated-*` scene carries the new optional `layer` key.
  C4  authoring control: the render-only row (ground ordinal 1) is never used as
      a query mover/ignore/direct target in the fixture world.
  C5  frozen trusted-unit bytes + registry ids: v1/spatial/segment v3/sweep v4/
      motion/character/camera2/objects/triggers/sprite units are byte-identical;
      the new topdown2d profile is registered once (profile 30 / version 121);
      the render-only units exist with exactly the four skip sites.
  C6  behavioral gate: the scoped Rust flow test passes
      (`cargo test --locked --offline --test node_topdown2d_flow`).

Usage: python tools/verify_plan058.py --controls [--skip-rust]
Exit: 0 = all selected controls PASS, 1 = at least one FAIL, 2 = usage error.
"""
import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
DRIVER = ROOT / "experimental/autographics-auto-driver"
PILOT = ROOT / "experimental/autographics-node2d-pilot"
FIXTURE = PILOT / "fixtures/plan058/topdown-ground"

FROZEN_PRODUCTS = {
    "experimental/autographics-node2d-pilot/generated-queries2d/q-sweep/scene.agscene":
        "c6ec0341c22cbf420dfdbc5a68ad9024bbd96b95554c969ce56be1a35995e87c",
    "experimental/autographics-node2d-pilot/generated-queries2d/q-sweep/world.at":
        "f7dc9f1987b375156b0f9067c51a11e93cff60d46818fcf2d4a76637c18f1b62",
    "experimental/autographics-node2d-pilot/generated-queries2d/q-segment/scene.agscene":
        "63376c418323917dede6a8f0822a4a0522ae75bd5e46fd31b9c61af74e89a4db",
    "experimental/autographics-node2d-pilot/generated-queries2d/q-segment/world.at":
        "52a37119efd935d54e800cf399edd8d7ed58c94636be112a96f7cc8f4391d553",
    "experimental/autographics-node2d-pilot/generated-queries2d/q-cross-plus/scene.agscene":
        "4745970038134fedc651db10e0a398ef74d6deda0afe5e79d92d74c10260d3f3",
    "experimental/autographics-node2d-pilot/generated-queries2d/q-cross-plus/world.at":
        "a9b71b0a1714bc50687ef8bbf5353c7969e2cd8ef91f7ead25ff773ad529b66f",
    "experimental/autographics-node2d-pilot/generated-sprites2d/atlas-g09/scene.agscene":
        "838b31081ea50a3a92f0b09b3b5bbfebd8d7794af84109d40c866bbca3c6713c",
    "experimental/autographics-node2d-pilot/generated-sprites2d/atlas-g09/world.at":
        "7145d97868abea634e8d52f03285b671dba0b413582c1d6c614fd62e60ee9724",
    "experimental/autographics-node2d-pilot/fixtures/sprites2d/assets/"
    "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png":
        "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1",
}

# Trusted unit bytes that existed before this task and must never drift.
FROZEN_UNITS = {
    "behavior_v1.at": "343f34a27c62f58980ef9df34a009341b20ba47d5a1b38cbbde7a1c3d499939f",
    "behavior_spatial_v2.at": "90069ed72f4a5bcfb6e52567df8b7af1c16a2c13318176dbd9afe39bace933e4",
    "behavior_segment_v3.at": "c92df25b15d75c4e3b043034263753844ae8512e963a73aa5f3731606c6fe6b0",
    "behavior_sweep_v4.at": "064585a0cee0ca63d3eef6db5dbb1a3d022ff4f9799716ef073aa329cb076b2f",
    "behavior_motion_v5.at": "1639146f75c5e72dade71ebc4e96ed3fc71023375c01a57e8d318ca7b9a458c3",
    "character2d_v1.at": "9f7c1a6cb42d3bd856ac966224eed6487ca6ad69e6f0b1dd11abbe20c1e81c59",
    "runtime_camera2d_v1.at": "655cfd794a7edfbb19b3f01c3d8d5ad0ff7405bb40162ec1df10af640a24a72f",
    "runtime_objects_v1.at": "fe9a8948270d6a8756661830aeb1c6ce7f35c18eb2f3e65d5befaf8595408dc5",
    "runtime_triggers_v1.at": "b2b75b168570d8cf86a470b65511d3923a04d0ae0848caa694a2b619fc0b8ce1",
    "sprite2d_v1.at": "a237dbda0f3e162100b3e28eeb34f35f43863be0db6d5e5649d886e2d53ce29b",
}

RESULTS = []


def sha256(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


def check(name, ok, detail):
    RESULTS.append((name, bool(ok), detail))
    print(("PASS " if ok else "FAIL ") + name + ": " + detail)


def c1_fixture_identity():
    scene_p = FIXTURE / "scene.agscene"
    world_p = FIXTURE / "world.at"
    id_p = FIXTURE / "identity.json"
    for p in (scene_p, world_p, id_p):
        if not p.is_file():
            return check("C1 fixture identity", False, f"missing {p}")
    scene = json.loads(scene_p.read_text())
    world = world_p.read_bytes()
    ident = json.loads(id_p.read_text())
    bound = scene["behavior"]["source"]["sha256"]
    layers = [o.get("layer") for o in scene["objects"] if o.get("layer")]
    ok = (
        bound == hashlib.sha256(world).hexdigest()
        and scene["behavior"]["source"]["byte_length"] == len(world)
        and scene["schema"] == "autographics.scene/9"
        and scene["behavior"]["api"] == "autographics.topdown2d/1"
        and len(scene["objects"]) == 7
        and layers == ["render-only"]
        and "cues" not in scene
        and ident["user_source_sha256"] == bound
        and ident["user_source_bytes"] == len(world)
        and ident["scene_sha256"] == sha256(scene_p)
        and ident["scene_bytes"] == scene_p.stat().st_size
        and ident["api"] == "autographics.topdown2d/1"
        and ident["roles"]["render_only"] == ["ground.body"]
        and len(ident["roles"]["scene_objects"]) == 7
    )
    check(
        "C1 fixture identity",
        ok,
        f"scene/9 + topdown2d/1, world {bound[:16]} len {len(world)}, rows 7, "
        f"layer {layers}, scene sha {ident['scene_sha256'][:16]}",
    )


def c2_frozen_legacy_bytes():
    bad = []
    for rel, want in FROZEN_PRODUCTS.items():
        p = ROOT / rel
        got = sha256(p) if p.is_file() else "MISSING"
        if got != want:
            bad.append(f"{rel}: {got[:16]} != {want[:16]}")
    check(
        "C2 frozen legacy product bytes",
        not bad,
        f"{len(FROZEN_PRODUCTS)} files checked, {len(bad)} drifted" + ("" if not bad else f": {bad}"),
    )


def c3_no_layer_in_legacy_scenes():
    scenes = list(PILOT.glob("generated-*/**/scene.agscene"))
    hits = [str(s.relative_to(ROOT)) for s in scenes if '"layer"' in s.read_text()]
    check(
        "C3 legacy scenes carry no layer key",
        not hits,
        f"{len(scenes)} legacy scenes scanned, {len(hits)} with a layer key"
        + ("" if not hits else f": {hits}"),
    )


def c4_render_only_row_never_queried():
    world = (FIXTURE / "world.at").read_text()
    ident = json.loads((FIXTURE / "identity.json").read_text())
    scene = json.loads((FIXTURE / "scene.agscene").read_text())
    order = [o["name"] for o in scene["objects"]]
    ground = ident["roles"]["render_only"][0]
    if ground not in order:
        return check("C4 render-only row never queried", False, f"{ground} missing")
    ordinal = order.index(ground)
    sweep = [int(m) for m in re.findall(r"ag_sweep_aabb\((\d+)", world)]
    segment = [int(m) for m in re.findall(r"ag_cast_segment\(2,\s*(\d+)", world)]
    direct = re.findall(r"ag_world\((\d+),|ag_local\((\d+),", world)
    addressed = {int(a or b) for a, b in direct}
    ok = (
        ordinal == 1
        and sweep and set(sweep) == {2}
        and segment and set(segment) == {2}
        and ordinal not in addressed
        and ident["roles"]["render_only"] == [ground]
    )
    check(
        "C4 render-only row never queried",
        ok,
        f"ground ordinal {ordinal}, sweep movers {sweep}, segment ignores {segment}, "
        f"world/local rows {sorted(addressed)}",
    )


def c5_units_and_registry():
    bad = []
    for name, want in FROZEN_UNITS.items():
        p = DRIVER / "api" / name
        got = sha256(p) if p.is_file() else "MISSING"
        if got != want:
            bad.append(f"{name}: {got[:16]} != {want[:16]}")
    new_units = {
        "behavior_renderonly_v1.at": 1,
        "behavior_segment_ro_v1.at": 4,
        "behavior_sweep_ro_v1.at": 4,
    }
    unit_counts = {}
    for name, want in new_units.items():
        p = DRIVER / "api" / name
        if not p.is_file():
            bad.append(f"{name}: MISSING")
            continue
        unit_counts[name] = p.read_text().count("_ag_ro_is(")
        if unit_counts[name] != want:
            bad.append(f"{name}: {unit_counts[name]} skip sites != {want}")
    beh = (DRIVER / "src/authoring/behavior.rs").read_text()
    reg = (
        'TOPDOWN2_VERSION: &str = "autographics.topdown2d/1"' in beh
        and "Self::Topdown2V1 => 121" in beh
        and "Self::Topdown2V1 => 30" in beh
        and 'Self::Topdown2V1 => "topdown2d"' in beh
        and "Self::Topdown2V1" in beh
        and 'require(input.renderonly, "fn _ag_ro_reset(", 1)' in (
            (DRIVER / "src/authoring/behavior/assembly.rs").read_text()
        )
    )
    if not reg:
        bad.append("topdown2d registry/assembly wiring missing")
    check(
        "C5 frozen units + topdown2d registry",
        not bad,
        f"{len(FROZEN_UNITS)} frozen units checked, 3 render-only units, "
        f"profile 30/version 121; skip sites {unit_counts}"
        + ("" if not bad else f": {bad}"),
    )


def c6_behavioral_gate():
    proc = subprocess.run(
        ["cargo", "test", "--locked", "--offline", "--test", "node_topdown2d_flow",
         "--", "--test-threads=1"],
        cwd=DRIVER,
        capture_output=True,
        text=True,
        timeout=900,
    )
    tail = [l for l in (proc.stdout + proc.stderr).splitlines() if "test result:" in l]
    check(
        "C6 scoped flow test",
        proc.returncode == 0,
        f"exit {proc.returncode}" + (f", {tail[-1].strip()}" if tail else ""),
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true", help="run every control group")
    ap.add_argument("--skip-rust", action="store_true", help="skip C6 (no cargo run)")
    args = ap.parse_args()
    if not args.controls:
        print("usage: python tools/verify_plan058.py --controls [--skip-rust]")
        return 2
    c1_fixture_identity()
    c2_frozen_legacy_bytes()
    c3_no_layer_in_legacy_scenes()
    c4_render_only_row_never_queried()
    c5_units_and_registry()
    if not args.skip_rust:
        c6_behavioral_gate()
    failed = [r for r in RESULTS if not r[1]]
    print(f"\nPLAN058 controls: {len(RESULTS) - len(failed)}/{len(RESULTS)} PASS")
    return 0 if not failed else 1


if __name__ == "__main__":
    sys.exit(main())
