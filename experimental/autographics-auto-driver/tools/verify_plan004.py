"""Verify the PLAN-004 frozen source closure (PNG visuals and 2D sprite frames).

Run in the plan worktree (or a fresh consumer export of the same closure):

  python tools/verify_plan004.py [--controls]

The identity binds every file of both experimental packages (including the
frozen SDK snapshot), the auto-ui commit the worktree branched from and 002's
delivered commit. --controls re-runs the verifier against deliberate mutations
(edited source, deleted source, unlisted file, wrong pin) and restores
everything before the final pass.
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PLAN_PIN = "0ee5ad0747ec3bb67e1288401fcc31daf3901c6e"
BASE_COMMIT = "d3bd6b495dc8b690925e88deee9fbf2c42e7f991"
REQUIRED = [
    "autographics-auto-driver/api/sprite2d_v1.at",
    "autographics-auto-driver/src/authoring/runtime_sprites.rs",
    "autographics-auto-driver/src/authoring/assets.rs",
    "autographics-auto-driver/src/authoring/document.rs",
    "autographics-auto-driver/src/authoring/storage.rs",
    "autographics-auto-driver/src/authoring/session.rs",
    "autographics-auto-driver/src/authoring/behavior.rs",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs",
    "autographics-auto-driver/src/authoring/play.rs",
    "autographics-auto-driver/src/authoring/host/viewport.rs",
    "autographics-auto-driver/src/authoring/host/commands.rs",
    "autographics-auto-driver/src/authoring/host/panel.rs",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs",
    "autographics-auto-driver/tests/public_sprite_frames.rs",
    "autographics-auto-driver/samples/sprite_frames/main.at",
    "autographics-auto-driver/samples/sprite_frames/sheet.png",
    "autographics-auto-driver/samples/sprite_frames/other.png",
    "autographics-auto-driver/samples/sprite_frames/padded.png",
    "autographics-auto-driver/samples/sprite_frames/generate.py",
    "autographics-auto-driver/tools/check_sprite_frames.py",
    "autographics-auto-driver/tools/gen_plan004_identity.py",
    "autographics-prototype/src/assets/png.rs",
    "autographics-prototype/src/renderer/sprite.rs",
    "autographics-prototype/src/host/surface_runtime.rs",
    "autographics-prototype/tests/sprite_resources.rs",
]
# Two old-profile constants the suite must still pin, proving the pre-existing
# assembled bytes were not moved by this plan.
OLD_PROFILE_TOKENS = [
    "c51700b106466b43dfc757fa5b1fb6c3ca1d6ac75ac2cd56e0fcf12b8a2652aa",  # triggers2d combined
    "08d83c162b20f3a96a49f0a1ffd78f18cfbd1210cd103d30230fa6b7f19459e2",  # objects2d combined
]


def digest(path, suffixes):
    data = path.read_bytes()
    if path.suffix.lower() in suffixes or path.name.lower() in suffixes:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def verify():
    identity_path = ROOT / "research/plan004-identity.json"
    identity = json.loads(identity_path.read_text())
    suffixes = identity["text_suffixes"]
    skip_dirs = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
    skip_names = {
        "window-report.json",
        "plan875-identity.json",
        "plan879-identity.json",
        "plan001-identity.json",
        "plan002-identity.json",
        "plan004-identity.json",
        "runtime-camera-reports",
        "runtime-objects-reports",
        "runtime-triggers-reports",
        "sprite-frames-reports",
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
    assert identity["plan_pin"] == PLAN_PIN, "PLAN-004 must consume 002's delivered commit"
    assert identity["base_commit"] == BASE_COMMIT, "PLAN-004 worktree base commit"
    sdk_pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    assert identity["sdk_pin"] == sdk_pin, "SDK artifact pin mismatch"
    for required in REQUIRED:
        assert required in identity["files"], f"004 source unlisted: {required}"
    assert identity["sdk_frozen_files"] == 681, "frozen SDK snapshot size"
    test_source = (ROOT / "tests/public_sprite_frames.rs").read_text()
    for token in OLD_PROFILE_TOKENS:
        assert token in test_source, f"old-profile baseline token missing: {token}"
    print(
        f"PASS PLAN-004 source/asset identity: {len(identity['files'])} hashes; "
        "sprite profile/tools/samples bound; 002 delivery and old-profile hashes pinned"
    )


def controls():
    identity_path = ROOT / "research/plan004-identity.json"
    original_identity = identity_path.read_bytes()
    victims = [
        ROOT / "api/sprite2d_v1.at",
        ROOT / "samples/sprite_frames/main.at",
        ROOT / "samples/sprite_frames/sheet.png",
        ROOT / "tests/public_sprite_frames.rs",
    ]
    originals = [p.read_bytes() for p in victims]
    added = ROOT / "samples/sprite_frames/unlisted_control.png"

    def rejected():
        result = subprocess.run([sys.executable, __file__], capture_output=True)
        assert result.returncode != 0 and any(
            v in result.stderr
            for v in [
                b"source hash mismatch",
                b"missing/outside source",
                b"source set drift",
                b"SDK artifact pin mismatch",
                b"PLAN-004",
                b"004 source unlisted",
                b"old-profile baseline token missing",
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
    assert not added.exists()
    added.write_bytes(b"\x89PNG control\n")
    try:
        rejected()
    finally:
        added.unlink()
    verify()
    print("PASS PLAN-004 identity controls: edited/deleted/unlisted/wrong-pin all rejected")


if __name__ == "__main__":
    if "--controls" in sys.argv:
        controls()
    else:
        verify()
