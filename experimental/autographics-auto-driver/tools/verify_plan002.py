"""Verify the PLAN-002 frozen source closure (trigger events and audited rounds).

Run in the plan worktree (or a fresh consumer export of the same closure):

  python tools/verify_plan002.py [--controls]

The identity binds every source/evidence input of both experimental packages,
the auto-ui planning pin this plan branched from (001's delivered commit), and
the frozen SDK artifact pin. --controls re-runs the verifier against deliberate
mutations (deleted source, unlisted source, wrong identity hash) and restores
everything.
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PLAN_PIN = "537122b09c025191355e260bc8e9880a4358cda2"
SDK_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
REQUIRED = [
    "autographics-auto-driver/api/runtime_triggers_v1.at",
    "autographics-auto-driver/src/authoring/runtime_round.rs",
    "autographics-auto-driver/src/authoring/behavior.rs",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs",
    "autographics-auto-driver/src/authoring/play.rs",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs",
    "autographics-auto-driver/tests/runtime_triggers_api.rs",
    "autographics-auto-driver/tests/runtime_round_host.rs",
    "autographics-auto-driver/tests/runtime_trigger_samples.rs",
    "autographics-auto-driver/samples/runtime-triggers/two/layout_a.agscene",
    "autographics-auto-driver/samples/runtime-triggers/two/main.at",
    "autographics-auto-driver/samples/runtime-triggers/two/main-alt.at",
    "autographics-auto-driver/samples/runtime-triggers/three/layout_a.agscene",
    "autographics-auto-driver/samples/runtime-triggers/three/main.at",
    "autographics-auto-driver/samples/runtime-triggers/three/main-alt.at",
    "autographics-auto-driver/tools/gen_plan002_identity.py",
]
TEXT_SUFFIXES = [
    ".rs",
    ".at",
    ".py",
    ".md",
    ".json",
    ".toml",
    ".lock",
    ".txt",
    ".agscene",
    ".gltf",
    ".gitignore",
    ".gitattributes",
]


def digest(path, suffixes):
    data = path.read_bytes()
    if path.suffix.lower() in suffixes or path.name.lower() in suffixes:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def verify():
    identity_path = ROOT / "research/plan002-identity.json"
    identity = json.loads(identity_path.read_text())
    suffixes = identity["text_suffixes"]
    skip_dirs = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
    skip_names = {
        "window-report.json",
        "plan875-identity.json",
        "plan879-identity.json",
        "plan001-identity.json",
        "plan002-identity.json",
        "runtime-camera-reports",
        "runtime-objects-reports",
        "runtime-triggers-reports",
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
    assert identity["plan_pin"] == PLAN_PIN, "PLAN-002 pin is 001's delivered commit"
    sdk_pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    assert identity["sdk_pin"] == sdk_pin, "SDK artifact pin mismatch"
    for required in REQUIRED:
        assert required in identity["files"], f"002 source unlisted: {required}"
    assert identity["sdk_frozen_files"] == 681, "frozen SDK snapshot size"
    print(
        f"PASS PLAN-002 source/asset identity: {len(identity['files'])} hashes; "
        "triggers profiles/tools/samples bound"
    )


def controls():
    identity_path = ROOT / "research/plan002-identity.json"
    original_identity = identity_path.read_bytes()
    victims = [
        ROOT / "api/runtime_triggers_v1.at",
        ROOT / "samples/runtime-triggers/three/main.at",
        ROOT / "tests/runtime_triggers_api.rs",
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
                b"PLAN-002 pin",
                b"002 source unlisted",
            ]
        ), result.stderr

    try:
        for path in victims:
            saved = path.read_bytes()
            path.write_bytes(saved + b"\n// control\n")
            rejected()
            path.write_bytes(saved)
        identity_path.write_bytes(original_identity.replace(b'"plan_pin": "', b'"plan_pin": "x'))
        rejected()
    finally:
        for path, original in zip(victims, originals):
            path.write_bytes(original)
        identity_path.write_bytes(original_identity)
    added.write_bytes(b"// unlisted control\n")
    rejected()
    added.unlink()
    verify()
    print("PASS PLAN-002 identity controls: deleted/unlisted/wrong-hash all rejected")


if __name__ == "__main__":
    if "--controls" in sys.argv:
        controls()
    else:
        verify()
