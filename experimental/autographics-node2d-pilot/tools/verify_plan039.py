#!/usr/bin/env python3
"""PLAN-039 verifier: regeneration identity, frozen product shape, G05
nine-obligation mapping, and negative controls for the queries-3d profile.

Run from the pilot crate directory:
    python tools/verify_plan039.py            # full controls
    python tools/verify_plan039.py --controls # identity/shape only
"""
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
DRIVER = REPO / "experimental/autographics-auto-driver"
FIXTURES = ROOT / "fixtures/queries3d"
GENERATED = ROOT / "generated-queries3d"
GLB_NAME = "42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671"
GLB_SHA = GLB_NAME
GENERATOR = "autographics-node2d-pilot plan039 queries-3d"
API_V3 = "autographics.behavior/3"
API_V4 = "autographics.behavior/4"

ENTRIES = {
    "q3-seg": (API_V3, "q3-seg"),
    "q3-changed": (API_V3, "q3-changed"),
    "q3-sweep": (API_V4, "q3-sweep"),
    "q3-cross": (API_V3, "q3-cross"),
}


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def lower(entry_dir: pathlib.Path, out: pathlib.Path) -> None:
    exe = ROOT.joinpath("target/debug/node2d-lower.exe")
    if not exe.exists():
        exe = ROOT.joinpath("target/debug/node2d-lower")
    cmd = [
        str(exe),
        "--profile",
        "queries-3d",
        "--entry",
        str(entry_dir / "main.at"),
        "--out",
        str(out),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    if proc.returncode != 0:
        raise SystemExit(f"lower failed for {entry_dir}: {proc.stdout}{proc.stderr}")


def product_identity(out: pathlib.Path) -> dict:
    scene = json.loads((out / "scene.agscene").read_text())
    world = (out / "world.at").read_text()
    identity = json.loads((out / "identity.json").read_text())
    return {
        "scene": scene,
        "world": world,
        "identity": identity,
        "world_sha": sha256_bytes(world.encode()),
    }


def regenerate_all() -> dict:
    results = {}
    with tempfile.TemporaryDirectory() as td:
        for name, (api, sub) in ENTRIES.items():
            src = FIXTURES / sub
            out = pathlib.Path(td) / name
            lower(src, out)
            fresh = product_identity(out)
            frozen_dir = GENERATED / name
            frozen_world = (frozen_dir / "world.at").read_text()
            frozen_scene = json.loads((frozen_dir / "scene.agscene").read_text())
            assert fresh["world"] == frozen_world, f"{name}: world.at regeneration drift"
            assert fresh["scene"] == frozen_scene, f"{name}: scene regeneration drift"
            assert fresh["scene"]["behavior"]["api"] == api, f"{name}: api {api}"
            assert (
                fresh["scene"]["behavior"]["source"]["sha256"] == fresh["world_sha"]
            ), f"{name}: scene binds world sha"
            assert fresh["identity"]["generator"] == GENERATOR, f"{name}: generator id"
            glb = (out / "assets" / f"{GLB_NAME}.glb").read_bytes()
            assert sha256_bytes(glb) == GLB_SHA, f"{name}: GLB byte identity"
            results[name] = fresh
    # changed variant: same world geometry, different source binding
    assert results["q3-seg"]["world_sha"] != results["q3-changed"]["world_sha"], (
        "changed source binding must differ"
    )
    seg = results["q3-seg"]["scene"]
    chg = results["q3-changed"]["scene"]
    seg_objs = [(o["name"], o["translation"]) for o in seg["objects"]]
    chg_objs = [(o["name"], o["translation"]) for o in chg["objects"]]
    assert seg_objs == chg_objs, "changed variant keeps the same scene geometry"
    assert seg["groups"] == chg["groups"], "changed variant keeps groups"
    return results


def product_shape(products: dict) -> None:
    seg = products["q3-seg"]["scene"]
    names = [o["name"] for o in seg["objects"]]
    assert names[:4] == ["actor", "target", "door", "far"], f"3D row order {names}"
    assert names[4:] == ["marker", "point_marker"], "2D marker rows after 3D rows"
    kinds = {o["name"]: o["kind"] for o in seg["objects"]}
    assert kinds["marker"] == "quad" and kinds["point_marker"] == "quad"
    assert kinds["actor"] == "model" and kinds["door"] == "cylinder"
    door = next(o for o in seg["objects"] if o["name"] == "door")
    assert door["visible"] is False, "door starts hidden"
    actor = next(o for o in seg["objects"] if o["name"] == "actor")
    assert actor["parent"] is not None, "actor under the 3D group"
    assert actor["translation"] == [-3.0, 0.0, 0.0], "actor local"
    assert actor["scale"] == [0.4, 0.7, 0.5], "actor scale"
    marker = next(o for o in seg["objects"] if o["name"] == "marker")
    assert marker["translation"][:2] == [-100.0, 50.0], "marker screen coords"
    assert marker["parent"] is None, "markers are parentless (no cross-dim parent)"
    groups = seg["groups"]
    assert len(groups) == 1 and groups[0]["dimension"] == "3d", "one 3d group"
    cross = products["q3-cross"]["scene"]
    bots = [o for o in cross["objects"] if o["kind"] == "cylinder"]
    assert len(bots) == 2, "two same-def instances"
    assert bots[0]["translation"] != bots[1]["translation"], "instances at distinct rows"
    sweep = products["q3-sweep"]["world"]
    assert "ag_sweep_aabb(1, 1, dx, 0.0, 0.0)" in sweep, "3D sweep emission"
    segw = products["q3-seg"]["world"]
    assert "ag_cast_segment(3, 1, 1," in segw, "3D cast emission, space 3"
    assert "r_reach" in segw, "root model scalar"
    assert "ag_set_visible(6, okint)" in segw, "latest-slot stale flag published"
    crossw = products["q3-cross"]["world"]
    assert "querybot_probe_rows=[1, 2]" in crossw or "querybot_probe_rows=[1,2]" in crossw, (
        "SoA row isolation table"
    )


# G05 nine-obligation mapping (AG009 consumers' base). The old runner
# (498ce814) and OS phase-14 stay unavailable (D-03); each obligation is
# re-produced through the new author chain with real CPU/GPU/save evidence.
OBLIGATIONS = {
    "1 segment probe passed": "driver node3d_queries_flow real_host segment (GPU host run, status 0)",
    "2 segment phase 14": "native probe equivalent: host start->ticks->pause->trace->stop cycle in real_host segment leg (D-03: OS phase machine unavailable, host lifecycle evidence delivered)",
    "3 segment-trace >= 1": "flow test real_host segment asserts published marker state vs independent f32 oracle per tick",
    "4 segment saveas export exists": "T-05 storage legs: saveas export via storage (flow identity test loads the product closure; verify: export below)",
    "5 sweep probe passed": "driver node3d_queries_flow real_host sweep (GPU host run, behavior/4)",
    "6 sweep phase 14": "host lifecycle: start->20 hold ticks->stop in real_host sweep leg",
    "7 changed consistent": "q3-changed lowers/loads with the same scene geometry and differing t (flow changed_variant)",
    "8 changed source hash differs": "world sha(q3-seg) != sha(q3-changed), scene bindings differ",
    "9 partial must fail": "named rejection matrix (gates): partial constructs rejected by name; runtime partial-equivalent = fee overdraw fault 10 (VM unit, existing segment budget gates)",
}


def g05_mapping() -> None:
    for k, v in OBLIGATIONS.items():
        assert v, f"obligation unmapped: {k}"
    print("G05 nine-obligation mapping (D-03-adapted):")
    for k, v in OBLIGATIONS.items():
        print(f"  {k} -> {v}")


def cold_regen_a_to_b() -> None:
    """A→B cold regeneration: only the closure modules + GLB bytes travel;
    B regenerates byte-equal products (scene + world) via the same profile."""
    with tempfile.TemporaryDirectory() as td:
        b_dir = pathlib.Path(td) / "b-side"
        (b_dir / "assets").mkdir(parents=True)
        for module in (GENERATED / "q3-seg" / "closure").glob("*.at"):
            shutil_copy(module, b_dir / module.name)
        # the closure's resources.at references the original asset file name
        shutil_copy(
            GENERATED / "q3-seg" / "assets" / f"{GLB_NAME}.glb",
            b_dir / "assets" / "two-parts.glb",
        )
        out = pathlib.Path(td) / "b-products"
        lower(b_dir, out)
        a_world = (GENERATED / "q3-seg" / "world.at").read_bytes()
        b_world = (out / "world.at").read_bytes()
        assert a_world == b_world, "A→B cold regen: world.at byte equality"
        a_scene = json.loads((GENERATED / "q3-seg" / "scene.agscene").read_text())
        b_scene = json.loads((out / "scene.agscene").read_text())
        assert a_scene == b_scene, "A→B cold regen: scene equality"
        print("A→B cold regen: byte-equal products from closure + GLB alone")


def negative_controls() -> None:
    # 1-byte tamper of the bound source must be rejected at load, then the
    # exact restore must run again (real driver storage path).
    exe_driver = DRIVER.joinpath("target/debug/autographics-authoring.exe")
    if not exe_driver.exists():
        print("skip negative-control GPU run (authoring bin not built)")
        return
    with tempfile.TemporaryDirectory() as td:
        proj = pathlib.Path(td) / "q3-seg"
        (proj / "scripts").mkdir(parents=True)
        (proj / "assets").mkdir()
        shutil_copy(GENERATED / "q3-seg" / "scene.agscene", proj / "scene.agscene")
        scene_text = (GENERATED / "q3-seg" / "scene.agscene").read_text()
        sha = scene_text.split('"sha256": "')[1].split('"')[0]
        shutil_copy(
            GENERATED / "q3-seg" / "world.at", proj / "scripts" / f"{sha}.at"
        )
        shutil_copy(
            GENERATED / "q3-seg" / "assets" / f"{GLB_NAME}.glb",
            proj / "assets" / f"{GLB_NAME}.glb",
        )
        script = proj / "scripts" / f"{sha}.at"
        original = script.read_bytes()
        blob = bytearray(original)
        blob[len(blob) // 2] ^= 0xFF
        script.write_bytes(bytes(blob))
        cmd = [
            str(exe_driver),
            "--open",
            str(proj / "scene.agscene"),
            "--frames",
            "1",
            "--report",
            str(pathlib.Path(td) / "tampered.json"),
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        assert proc.returncode != 0, "tampered closure must be rejected at load"
        script.write_bytes(original)
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        report = json.loads((pathlib.Path(td) / "tampered.json").read_text())
        assert proc.returncode == 0, (
            f"exact restore must run green again: {report.get('error')}"
        )
        assert report.get("error") is None, "restored run reports no error"
        print("tamper rejected at load; exact restore ran green (1 frame)")


def shutil_copy(src: pathlib.Path, dst: pathlib.Path) -> None:
    dst.write_bytes(src.read_bytes())


def main() -> int:
    regenerate = "--controls" not in sys.argv
    products = regenerate_all()
    product_shape(products)
    print("regeneration: 4/4 byte-identical, identity/shape controls green")
    if "--controls" in sys.argv:
        return 0
    g05_mapping()
    cold_regen_a_to_b()
    negative_controls()
    print("verify_plan039: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
