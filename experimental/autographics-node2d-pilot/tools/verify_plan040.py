#!/usr/bin/env python3
"""PLAN-040 verifier: regeneration identity, frozen G04 product shape and
mapping, A→B cold regeneration, negative controls, and native GPU smoke for
the editor-projects entry (pure-declarative dual-view node projects).

Run from the pilot crate directory:
    python tools/verify_plan040.py            # full controls (needs the built
                                              # autographics-authoring binary)
    python tools/verify_plan040.py --controls # identity/shape only (no engine)

The original ag-examples G04 probe route (default-document 34-phase / project
20-phase, real OS input) stays the OLD positive control owned by the gallery
acceptance; OS input automation stays paused (plan D-02) and is never
replaced or deleted here.
"""
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
DRIVER = REPO / "experimental/autographics-auto-driver"
FIXTURES = ROOT / "fixtures/editor-projects"
GENERATED = ROOT / "generated-editor-projects"
GENERATOR = "autographics-node2d-pilot plan040 editor-projects"
DOC_PREFIX = "plan040-editor-projects"
SCHEMA = "autographics.scene/6"

ENTRIES = ("g04-editor-2d", "g04-editor-3d")

# Frozen original G04 object values (ag-examples@501fca0 examples/g04-editor,
# T-01 freeze): name -> (kind, translation, scale, rgba).
ORIGINALS = {
    "g04-editor-2d": {
        1: ("circle", [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        2: ("quad", [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        3: ("player", [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        4: ("mob", [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        5: ("quad", [400.0, -300.0, 0.0], [30.0, 30.0, 1.0], [0.2, 0.3, 0.9, 1.0]),
        6: ("circle", [-400.0, 300.0, 0.0], [20.0, 20.0, 1.0], [0.2, 0.8, 0.3, 1.0]),
    },
    "g04-editor-3d": {
        1: ("circle", [-55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        2: ("quad", [55.0, 0.0, 0.0], [50.0, 50.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        3: ("player", [-1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        4: ("mob", [1.5, 0.0, 0.0], [1.0, 1.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
        5: ("cylinder", [30.0, 0.0, 0.0], [1.0, 2.0, 1.0], [0.9, 0.3, 0.2, 1.0]),
    },
}

# Explicit ID mapping (generated row id -> original id).
MAPPING = {
    "g04-editor-2d": {1: 3, 2: 4, 3: 1, 4: 2, 5: 5, 6: 6},
    "g04-editor-3d": {1: 3, 2: 4, 3: 5, 4: 1, 5: 2},
}


def sha256_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def engine() -> pathlib.Path:
    exe = DRIVER.joinpath("target/debug/autographics-authoring.exe")
    if not exe.exists():
        exe = DRIVER.joinpath("target/debug/autographics-authoring")
    return exe


def lower(entry_dir: pathlib.Path, out: pathlib.Path, profile: str = "editor-projects") -> None:
    exe = ROOT.joinpath("target/debug/node2d-lower.exe")
    if not exe.exists():
        exe = ROOT.joinpath("target/debug/node2d-lower")
    cmd = [str(exe), "--profile", profile, "--entry", str(entry_dir / "main.at"), "--out", str(out)]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    if proc.returncode != 0:
        raise SystemExit(f"lower failed for {entry_dir}: {proc.stdout}{proc.stderr}")


def check_scene(which: str, scene: dict) -> None:
    assert scene["behavior"] is None, f"{which}: behavior null"
    assert scene["schema"] == SCHEMA, f"{which}: schema"
    assert scene["document_id"] == f"{DOC_PREFIX}-main", f"{which}: doc id"
    assert scene["groups"] == [] and scene["assets"] == [], f"{which}: no groups/assets"
    objects = scene["objects"]
    assert len(objects) == len(ORIGINALS[which]), f"{which}: object count"
    for o in objects:
        orig = MAPPING[which][o["id"]]
        kind, t, s, rgba = ORIGINALS[which][orig]
        assert o["kind"] == kind, f"{which} orig {orig} kind"
        assert o["translation"] == t, f"{which} orig {orig} translation"
        assert o["scale"] == s, f"{which} orig {orig} scale"
        assert o["rgba"] == rgba, f"{which} orig {orig} rgba"
        assert o["visible"] is True and o["parent"] is None and o["resource"] is None
        assert o["yaw"] == 0.0 and o["pitch"] == 0.0
    # 2D draw order: 2D-kind rows keep the original relative order.
    two_d = [o["id"] for o in objects if o["kind"] in ("circle", "quad")]
    two_d_orig = [MAPPING[which][i] for i in two_d]
    assert two_d_orig == sorted(two_d_orig), f"{which}: 2D draw order {two_d_orig}"


def regenerate_identity() -> None:
    with tempfile.TemporaryDirectory() as td:
        for name in ENTRIES:
            out = pathlib.Path(td) / name
            lower(FIXTURES / name, out)
            fresh_scene = (out / "scene.agscene").read_bytes()
            fresh_identity = (out / "identity.json").read_bytes()
            frozen_scene = (GENERATED / name / "scene.agscene").read_bytes()
            frozen_identity = (GENERATED / name / "identity.json").read_bytes()
            assert fresh_scene == frozen_scene, f"{name}: scene regeneration drift"
            assert fresh_identity == frozen_identity, f"{name}: identity.json drift"
            check_scene(name, json.loads(fresh_scene.decode()))
        print(f"regeneration: {len(ENTRIES)}/{len(ENTRIES)} byte-identical, shape/mapping green")


def cold_regen_a_to_b() -> None:
    """A→B cold regeneration: only the closure modules travel; B regenerates
    byte-equal products via the same profile."""
    with tempfile.TemporaryDirectory() as td:
        b_dir = pathlib.Path(td) / "b-side"
        (b_dir / "assets").mkdir(parents=True)
        for name in ENTRIES:
            closure_src = GENERATED / name / "closure"
            work = b_dir / name
            work.mkdir()
            for module in closure_src.glob("*.at"):
                shutil.copyfile(module, work / module.name)
            out = pathlib.Path(td) / f"{name}-b"
            lower(work, out)
            assert (out / "scene.agscene").read_bytes() == (
                GENERATED / name / "scene.agscene"
            ).read_bytes(), f"A→B cold regen: {name} scene byte equality"
    print("A→B cold regen: byte-equal products from closure alone (both projects)")


def support_domain_negatives() -> None:
    """Named rejections through the real CLI entry (no engine needed)."""
    cases = [
        ("neg-behavior", "editor-projects accepts pure declarative projects only"),
        ("neg-child-dual", "view-2d is only allowed on the queries-3d root node"),
        ("neg-marker-prop", "unknown marker prop"),
        ("neg-marker-dynamic", "marker numeric prop must be a literal number"),
        ("neg-duplicate-name", "duplicate scene row name"),
    ]
    with tempfile.TemporaryDirectory() as td:
        for name, needle in cases:
            out = pathlib.Path(td) / name
            proc = subprocess.run(
                [
                    str(ROOT.joinpath("target/debug/node2d-lower.exe")
                        if ROOT.joinpath("target/debug/node2d-lower.exe").exists()
                        else ROOT.joinpath("target/debug/node2d-lower")),
                    "--profile", "editor-projects",
                    "--entry", str(FIXTURES / name / "main.at"),
                    "--out", str(out),
                ],
                capture_output=True, text=True, timeout=300,
            )
            assert proc.returncode != 0, f"{name}: must be rejected"
            assert needle in proc.stderr, f"{name}: named rejection missing '{needle}': {proc.stderr}"
    print("support-domain negatives: 5/5 named rejections through the CLI")


def native_runs_and_rejects() -> None:
    """The real engine opens each project (60-frame GPU loop, report ok); a
    tampered scene is rejected at load; the exact restore runs green."""
    exe = engine()
    assert exe.exists(), f"engine binary missing: {exe}"
    runner_sha = sha256_bytes(exe.read_bytes())
    with tempfile.TemporaryDirectory() as td:
        for name in ENTRIES:
            report = pathlib.Path(td) / f"{name}.json"
            proc = subprocess.run(
                [str(exe), "--open", str(GENERATED / name / "scene.agscene"),
                 "--frames", "60", "--report", str(report)],
                capture_output=True, text=True, timeout=600,
            )
            assert proc.returncode == 0, f"{name}: 60-frame smoke failed: {proc.stdout}{proc.stderr}"
            rep = json.loads(report.read_text())
            assert rep.get("error") is None, f"{name}: smoke error {rep.get('error')}"
        # Negative: 1-byte scene tamper must be rejected at load; exact
        # restore runs green again.
        proj = pathlib.Path(td) / "tamper"
        proj.mkdir()
        target = proj / "scene.agscene"
        original = (GENERATED / "g04-editor-3d" / "scene.agscene").read_bytes()
        blob = bytearray(original)
        blob[len(blob) // 2] ^= 0xFF
        target.write_bytes(bytes(blob))
        cmd = [str(exe), "--open", str(target), "--frames", "1",
               "--report", str(pathlib.Path(td) / "tamper.json")]
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        assert proc.returncode != 0, "tampered scene bytes must be rejected at load"
        target.write_bytes(original)
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        assert proc.returncode == 0, f"exact restore must run green: {proc.stdout}{proc.stderr}"
    print(f"native smoke: 2x60-frame green + tamper rejected + restore green (runner {runner_sha[:16]})")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true", help="identity/shape only")
    args = ap.parse_args()
    regenerate_identity()
    support_domain_negatives()
    cold_regen_a_to_b()
    if not args.controls:
        native_runs_and_rejects()
    print("verify_plan040: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
