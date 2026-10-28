"""Verify the committed PLAN-879 migrated source closure and move mapping.

Checks that the current auto-ui experimental source/evidence set matches the
frozen identity byte-for-byte (original bytes AND CRLF-normalized text), that
no unlisted source was added and none is missing, that every migrated file
records its source blob SHA and the declared PLAN-879 adaptations/snapshot are
intact, and that the migration pins (auto-ui base, auto-lang source commit,
frozen snapshot identity) still hold. Old blob SHAs are provenance pointers
into the auto-lang repository at the recorded source commit; re-deriving them
is a cross-repository audit, not a local check.

  python tools/verify_plan879.py            # verify only
  python tools/verify_plan879.py --controls # tamper controls, then restore
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
BASE_COMMIT = "154a39657e96bb88806b4a31d105a91d633758fe"
SOURCE_COMMIT = "1cc68aba012f76951af2556763a5fb21be6a6c8b"
LOGICAL_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
MIGRATED_COUNT = 702
REQUIRED = [
    "autographics-auto-driver/Cargo.toml",
    "autographics-auto-driver/Cargo.lock",
    "autographics-auto-driver/.gitattributes",
    "autographics-auto-driver/.cargo/config.toml",
    "autographics-auto-driver/tools/prepare_sdk.py",
    "autographics-auto-driver/tools/verify_source.py",
    "autographics-auto-driver/tools/package_consumer.py",
    "autographics-auto-driver/tools/gen_plan879_identity.py",
    "autographics-auto-driver/tools/verify_plan879.py",
    "autographics-auto-driver/research/source-lock.json",
    "autographics-auto-driver/research/baseline.json.txt",
    "autographics-auto-driver/api/runtime_camera2d_v1.at",
    "autographics-auto-driver/api/runtime_camera3d_v1.at",
    "autographics-auto-driver/src/authoring/runtime_camera.rs",
    "autographics-auto-driver/src/bin/authoring/cli.rs",
    "autographics-auto-driver/samples/runtime-camera/two/main.at",
    "autographics-auto-driver/samples/runtime-camera/three/main.at",
    "autographics-auto-driver/sdk/frozen/Cargo.lock",
    "autographics-auto-driver/sdk/frozen/crates/auto-lang-core/src/lib.rs",
    "autographics-prototype/Cargo.toml",
    "autographics-prototype/Cargo.lock",
    "autographics-prototype/.cargo/config.toml",
    "autographics-prototype/testdata/breakout/golden_trace.json",
]


def digest(path, suffixes):
    data = path.read_bytes()
    normalized = data
    if path.suffix.lower() in suffixes or path.name.lower() in suffixes:
        normalized = data.replace(b"\r\n", b"\n")
    return (
        hashlib.sha256(data).hexdigest(),
        hashlib.sha256(normalized).hexdigest(),
    )


def verify():
    identity = json.loads((ROOT / "research/plan879-identity.json").read_text())
    suffixes = identity["text_suffixes"]
    skip_dirs = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
    skip_names = {
        "window-report.json",
        "plan875-identity.json",
        "plan879-identity.json",
        "runtime-camera-reports",
    }
    migrated = 0
    for relative, expected in identity["files"].items():
        raw = BASE / relative
        if not raw.is_file():
            raise AssertionError(f"missing/outside source: {relative}")
        assert not any(
            v.is_symlink() or v.is_junction()
            for v in [raw, *raw.parents]
            if v != BASE.parent
        ), f"linked source: {relative}"
        raw_hash, normalized_hash = digest(raw, suffixes)
        if raw_hash != expected["sha256_bytes"] or normalized_hash != expected["sha256_normalized"]:
            raise AssertionError(f"source hash mismatch: {relative}")
        if "blob" in expected:
            migrated += 1
    assert migrated == MIGRATED_COUNT, f"migrated file count: {migrated}"
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
    assert identity["base_commit"] == BASE_COMMIT, "PLAN-879 base is the auto-ui move base"
    assert identity["source"]["commit"] == SOURCE_COMMIT, "source commit is the 875 delivery"
    frozen = json.loads((ROOT / "research/source-lock.json").read_text())["files"]
    sdk_pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    snapshot = {
        relative: entry["sha256_bytes"]
        for relative, entry in identity["files"].items()
        if relative.startswith("autographics-auto-driver/sdk/frozen/")
    }
    mapped = {
        relative[len("autographics-auto-driver/sdk/frozen/"):]: sha
        for relative, sha in snapshot.items()
    }
    for relative, sha in mapped.items():
        assert frozen.get(relative) == sha, f"snapshot deviates from source lock: {relative}"
    assert len(mapped) == len(frozen) == 681, (len(mapped), len(frozen))
    declared = identity["sdk_frozen"]
    assert declared["files"] == 681 and declared["logical_pin"] == sdk_pin == LOGICAL_PIN
    aggregate = hashlib.sha256(
        "".join(
            f"{relative}={sha}\n"
            for relative, sha in sorted(snapshot.items())
        ).encode()
    ).hexdigest()
    assert aggregate == declared["aggregate_sha256"], "frozen snapshot aggregate drift"
    for required in REQUIRED:
        assert required in identity["files"], f"879 source unlisted: {required}"
    print(
        f"PASS PLAN-879 moved source identity: {len(identity['files'])} hashes; "
        f"{migrated} source blobs bound; 681 frozen snapshot files match source-lock"
    )


def controls():
    identity_path = ROOT / "research/plan879-identity.json"
    original_identity = identity_path.read_bytes()
    victims = [
        ROOT / "api/character3d_v1.at",
        ROOT / "sdk/frozen/stdlib/auto/char.at",
        ROOT / "tools/package_consumer.py",
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
                b"snapshot deviates from source lock",
                b"migrated file count",
                b"879 source unlisted",
                b"frozen snapshot aggregate drift",
            ]
        ), result.stderr

    try:
        for path in victims:
            saved = path.read_bytes()
            path.unlink()
            rejected()
            path.write_bytes(saved)
        for path, poison in [
            (victims[1], b"// snapshot tamper\n"),
            (victims[2], b"# consumer tamper\n"),
        ]:
            saved = path.read_bytes()
            path.write_bytes(saved + poison)
            rejected()
            path.write_bytes(saved)
        added.write_text("// unlisted source")
        rejected()
        added.unlink()
        data = json.loads(original_identity)
        first = next(iter(data["files"]))
        data["files"][first]["sha256_normalized"] = "0" * 64
        identity_path.write_bytes(json.dumps(data).encode())
        rejected()
        data = json.loads(original_identity)
        data["sdk_frozen"]["aggregate_sha256"] = "0" * 64
        identity_path.write_bytes(json.dumps(data).encode())
        rejected()
    finally:
        for path, saved in zip(victims, originals):
            path.write_bytes(saved)
        identity_path.write_bytes(original_identity)
        if added.exists():
            added.unlink()
    print(
        "PASS probe source/snapshot deletion, tampered bytes, unlisted source, "
        "wrong identity hash, wrong snapshot aggregate and restored control"
    )


if __name__ == "__main__":
    verify()
    if "--controls" in sys.argv:
        controls()
