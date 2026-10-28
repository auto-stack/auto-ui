"""Verify the committed PLAN-875 source closure, including text transport rules.

Checks that the current runtime-camera source/evidence set is byte-identical
to the frozen identity, that no unlisted source was added, that the two pins
still match (the PLAN-874 delivery base and the frozen SDK artifact), and that
the public runtime-camera inputs this plan claims are actually listed.

  python tools/verify_plan875.py            # verify only
  python tools/verify_plan875.py --controls # tamper controls, then restore
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
BASE_COMMIT = "c02a9c2227f54542082abb35608d828fc4dbf674"
REQUIRED = [
    "autographics-auto-driver/api/runtime_camera2d_v1.at",
    "autographics-auto-driver/api/runtime_camera3d_v1.at",
    "autographics-auto-driver/src/authoring/runtime_camera.rs",
    "autographics-auto-driver/src/authoring/play.rs",
    "autographics-auto-driver/src/authoring/behavior.rs",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs",
    "autographics-auto-driver/src/authoring/host/viewport.rs",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs",
    "autographics-auto-driver/src/authoring/host/commands.rs",
    "autographics-auto-driver/src/authoring/host/panel.rs",
    "autographics-auto-driver/src/authoring/host.rs",
    "autographics-auto-driver/src/authoring/mod.rs",
    "autographics-auto-driver/src/bin/authoring/cli.rs",
    "autographics-auto-driver/src/bin/authoring/probes.rs",
    "autographics-auto-driver/src/bin/authoring_probe/runtime_camera.rs",
    "autographics-auto-driver/src/bin/autographics-authoring.rs",
    "autographics-auto-driver/tests/runtime_camera_api.rs",
    "autographics-auto-driver/tests/runtime_camera_host.rs",
    "autographics-auto-driver/tests/runtime_camera_input.rs",
    "autographics-auto-driver/tools/check_runtime_camera.py",
    "autographics-auto-driver/samples/runtime-camera/two/main.at",
    "autographics-auto-driver/samples/runtime-camera/two/main-alt.at",
    "autographics-auto-driver/samples/runtime-camera/two/layout_a.agscene",
    "autographics-auto-driver/samples/runtime-camera/three/main.at",
    "autographics-auto-driver/samples/runtime-camera/three/main-alt.at",
    "autographics-auto-driver/samples/runtime-camera/three/layout_a.agscene",
    "autographics-auto-driver/tools/verify_plan875.py",
]


def digest(path, suffixes):
    data = path.read_bytes()
    if path.suffix.lower() in suffixes or path.name.lower() in suffixes:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def verify():
    identity = json.loads((ROOT / "research/plan875-identity.json").read_text())
    suffixes = identity["text_suffixes"]
    skip_dirs = {"target", "__pycache__", ".sdk-source"}
    skip_names = {
        "window-report.json",
        "plan875-identity.json",
        "runtime-camera-reports",
    }
    for relative, expected in identity["files"].items():
        raw = BASE / relative
        if not raw.is_file():
            raise AssertionError(f"missing/outside source: {relative}")
        assert not any(
            v.is_symlink() or v.is_junction()
            for v in [raw, *raw.parents]
            if v != BASE.parent
        ), f"linked source: {relative}"
        if digest(raw, suffixes) != expected:
            raise AssertionError(f"source hash mismatch: {relative}")
    for folder in identity["folders"]:
        actual = set()
        for path in (BASE / folder).rglob("*"):
            if not path.is_file():
                continue
            parts = path.relative_to(BASE).parts
            if any(part in skip_dirs or part in skip_names for part in parts):
                continue
            actual.add(path.relative_to(BASE).as_posix())
        expected = {p for p in identity["files"] if p.startswith(folder + "/")}
        assert actual == expected, f"source set drift: {folder}: {actual ^ expected}"
    assert identity["base_commit"] == BASE_COMMIT, "PLAN-875 base is the 874 delivery"
    sdk_pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    assert identity["sdk_commit"] == sdk_pin, "SDK artifact pin mismatch"
    for required in REQUIRED:
        assert required in identity["files"], f"875 source unlisted: {required}"
    print(
        f"PASS PLAN-875 source/asset identity: {len(identity['files'])} hashes; "
        "profiles/tools/samples bound"
    )


def controls():
    identity_path = ROOT / "research/plan875-identity.json"
    original_identity = identity_path.read_bytes()
    victims = [
        ROOT / "api/runtime_camera3d_v1.at",
        ROOT / "samples/runtime-camera/three/main.at",
        ROOT / "tests/runtime_camera_api.rs",
    ]
    originals = [p.read_bytes() for p in victims]
    added = ROOT / "src/authoring/unlisted_control.rs"
    assert not added.exists()

    def rejected():
        result = subprocess.run([sys.executable, __file__], capture_output=True)
        assert result.returncode != 0 and any(
            v in result.stderr
            for v in [
                b"source hash mismatch",
                b"missing/outside source",
                b"source set drift",
                b"SDK artifact pin mismatch",
                b"875 source unlisted",
            ]
        ), result.stderr

    try:
        for path in victims:
            saved = path.read_bytes()
            path.unlink()
            rejected()
            path.write_bytes(saved)
        added.write_text("// unlisted source")
        rejected()
        added.unlink()
        data = json.loads(original_identity)
        data["files"]["autographics-auto-driver/api/runtime_camera3d_v1.at"] = "0" * 64
        identity_path.write_bytes(json.dumps(data).encode())
        rejected()
    finally:
        for path, saved in zip(victims, originals):
            path.write_bytes(saved)
        identity_path.write_bytes(original_identity)
        if added.exists():
            added.unlink()
    print(
        "PASS probe source deletion, unlisted source, wrong identity hash and restored control"
    )


if __name__ == "__main__":
    verify()
    if "--controls" in sys.argv:
        controls()
