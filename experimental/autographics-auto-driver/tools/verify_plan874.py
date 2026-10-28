"""Verify the committed PLAN-874 source closure, including text transport rules.

Checks that the current character3d/1 source/evidence set is byte-identical to
the frozen identity, that no unlisted source was added, that the two pins still
match (PLAN-861 delivery base and the frozen SDK artifact), and that the public
character3d inputs this plan claims are actually listed.

  python tools/verify_plan874.py            # verify only
  python tools/verify_plan874.py --controls # tamper controls, then restore
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
BASE_COMMIT = "8b12632d7b5b1b0df72fbb4eef84297a054de31d"
REQUIRED = [
    "autographics-auto-driver/api/character3d_v1.at",
    "autographics-auto-driver/src/authoring/game_input.rs",
    "autographics-auto-driver/src/authoring/play.rs",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs",
    "autographics-auto-driver/tests/character3d_api.rs",
    "autographics-auto-driver/tests/character3d_host.rs",
    "autographics-auto-driver/tests/character3d_samples.rs",
    "autographics-auto-driver/tools/check_character3d.py",
    "autographics-auto-driver/samples/character3d/main.at",
    "autographics-auto-driver/samples/character3d/main-fast.at",
    "autographics-auto-driver/samples/character3d/layout_a.agscene",
    "autographics-auto-driver/samples/character3d/layout_b.agscene",
    "autographics-auto-driver/tools/verify_plan874.py",
]


def digest(path, suffixes):
    data = path.read_bytes()
    if path.suffix.lower() in suffixes or path.name.lower() in suffixes:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def verify():
    identity = json.loads((ROOT / "research/plan874-identity.json").read_text())
    suffixes = identity["text_suffixes"]
    skip_dirs = {"target", "__pycache__", ".sdk-source"}
    skip_names = {"window-report.json", "plan874-identity.json", "character3d-reports"}
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
    assert identity["base_commit"] == BASE_COMMIT, "PLAN-874 base is the 861 delivery"
    sdk_pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    assert identity["sdk_commit"] == sdk_pin, "SDK artifact pin mismatch"
    for required in REQUIRED:
        # Complements are listed by the verifier that ships with this plan; the
        # checker is added by the same task, so tolerate its own listing only.
        assert required in identity["files"], f"874 source unlisted: {required}"
    print(
        f"PASS PLAN-874 source/asset identity: {len(identity['files'])} hashes; "
        "profile/tools/samples bound"
    )


def controls():
    identity_path = ROOT / "research/plan874-identity.json"
    original_identity = identity_path.read_bytes()
    victims = [
        ROOT / "api/character3d_v1.at",
        ROOT / "samples/character3d/main.at",
        ROOT / "tests/character3d_api.rs",
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
                b"874 source unlisted",
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
        data["files"]["autographics-auto-driver/api/character3d_v1.at"] = "0" * 64
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
