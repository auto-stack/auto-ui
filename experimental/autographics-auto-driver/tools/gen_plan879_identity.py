"""Freeze the PLAN-879 migrated source closure and old→new move mapping.

Run once per semantic revision, in the plan worktree, after the source set is
stable (the captured source blob map from the migration export is required):

  python tools/gen_plan879_identity.py --source-blobs <plan879 source-blobs.json>

The identity lists every source/evidence input of both experimental packages
plus the versioned frozen SDK snapshot (sdk/frozen/**), and records the
migration provenance required by tools/verify_plan879.py: the auto-ui base
commit, the source repository/branch/commit, the old Git blob SHA and both
original-byte and CRLF-normalized content SHA256 for every migrated file, the
intentional PLAN-879 adaptations, and the frozen snapshot identity. Volatile
run reports and this identity file itself are excluded from the walk.
"""
import argparse
import hashlib
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PACKAGES = ["autographics-auto-driver", "autographics-prototype"]
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
SKIP_DIRS = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
SKIP_NAMES = {
    "window-report.json",
    "plan875-identity.json",
    "plan879-identity.json",
    "runtime-camera-reports",
}
BASE_COMMIT = "154a39657e96bb88806b4a31d105a91d633758fe"
SOURCE = {
    "repository": "auto-lang",
    "branch": "autographics-dev",
    "commit": "1cc68aba012f76951af2556763a5fb21be6a6c8b",
    "subtree_prefix": "experimental/",
}
SNAPSHOT_PREFIX = "autographics-auto-driver/sdk/frozen/"
ADAPTATIONS = {
    "autographics-auto-driver/.gitattributes": "modified: sdk/frozen/** -text (byte-exact snapshot round-trip against research/source-lock.json)",
    "autographics-auto-driver/.gitignore": "modified: /.sdk-source.prepare-tmp/ ignored (snapshot staging residue)",
    "autographics-auto-driver/.cargo/config.toml": "new: build-stack config restated from old repo root .cargo/config.toml (32MiB PE main-thread stack, 16MiB RUST_MIN_STACK); auto-ui root has no rustflags",
    "autographics-prototype/.cargo/config.toml": "new: build-stack config restated from old repo root .cargo/config.toml (same stack profile, workspace self-containment)",
    "autographics-auto-driver/tools/prepare_sdk.py": "modified: --from-snapshot preparation path (logical SDK pin 7b987ca8 Git object unreachable); legacy --repository path retained",
    "autographics-auto-driver/tools/verify_source.py": "modified: current identity chain selects plan879; historical chain preserved behind it",
    "autographics-auto-driver/tools/package_consumer.py": "modified: latest identity list selects plan879",
    "autographics-auto-driver/tools/gen_plan879_identity.py": "new: PLAN-879 source identity and move-mapping generator",
    "autographics-auto-driver/tools/verify_plan879.py": "new: PLAN-879 identity verifier with tamper controls",
    "autographics-auto-driver/research/plan879-identity.json": "new: PLAN-879 source identity + migration mapping (tracked; the move's own provenance deliverable)",
    "autographics-auto-driver/README.md": "modified: PLAN-879 section (auto-ui ownership, snapshot cold start, export stack self-containment)",
    "autographics-prototype/README.md": "modified: PLAN-879 repository ownership note",
}


def digests(path):
    data = path.read_bytes()
    normalized = data
    if path.suffix.lower() in TEXT_SUFFIXES or path.name.lower() in TEXT_SUFFIXES:
        normalized = data.replace(b"\r\n", b"\n")
    return {
        "sha256_bytes": hashlib.sha256(data).hexdigest(),
        "sha256_normalized": hashlib.sha256(normalized).hexdigest(),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-blobs", type=pathlib.Path, required=True)
    args = parser.parse_args()
    source_blobs = json.loads(args.source_blobs.read_text())

    files = {}
    folders = []
    for package in PACKAGES:
        package_root = BASE / package
        for child in sorted(package_root.iterdir()):
            if child.is_file():
                files[f"{package}/{child.name}"] = digests(child)
            elif child.is_dir() and child.name not in SKIP_DIRS:
                folders.append(f"{package}/{child.name}")
        for path in sorted(package_root.rglob("*")):
            if not path.is_file():
                continue
            relative = path.relative_to(BASE)
            if relative.as_posix() in files:
                continue
            if any(part in SKIP_DIRS or part in SKIP_NAMES for part in relative.parts):
                continue
            files[relative.as_posix()] = digests(path)

    migrated = 0
    added = 0
    snapshot_files = []
    snapshot_bytes = 0
    snapshot_max = 0
    for relative, entry in files.items():
        if relative in source_blobs:
            assert not ADAPTATIONS.get(relative, "").startswith(
                "new:"
            ), f"declared-new file also exists at source: {relative}"
            entry["blob"] = source_blobs[relative]
            migrated += 1
            continue
        reason = ADAPTATIONS.get(relative)
        if reason is None:
            assert relative.startswith(SNAPSHOT_PREFIX), f"undeclared new file: {relative}"
            snapshot_files.append(relative)
            size = (BASE / relative).stat().st_size
            snapshot_bytes += size
            snapshot_max = max(snapshot_max, size)
        else:
            entry["origin"] = reason
            added += 1

    aggregate = hashlib.sha256(
        "".join(
            f"{relative}={files[relative]['sha256_bytes']}\n"
            for relative in sorted(snapshot_files)
        ).encode()
    ).hexdigest()
    identity = {
        "plan": "PLAN-879",
        "base_commit": BASE_COMMIT,
        "source": SOURCE,
        "sdk_frozen": {
            "files": len(snapshot_files),
            "bytes": snapshot_bytes,
            "max_file_bytes": snapshot_max,
            "aggregate_sha256": aggregate,
            "logical_pin": "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467",
            "provenance": "versioned frozen copy of the materialized SDK consumer input; "
            "preserved copy at plan-860 consumer package (sha256 of its research/plan875-identity"
            " cross-check in plan receipts); the logical-pin Git object is unreachable, so the"
            " snapshot is the reproducible source; per-file hashes locked by research/source-lock.json",
        },
        "text_suffixes": TEXT_SUFFIXES,
        "folders": sorted(folders),
        "files": dict(sorted(files.items())),
    }
    out = ROOT / "research/plan879-identity.json"
    out.write_text(json.dumps(identity, indent=1) + "\n")
    print(
        f"frozen PLAN-879 source identity: {len(identity['files'])} files "
        f"({migrated} migrated from source blobs, {added} declared additions, "
        f"{len(snapshot_files)} frozen SDK snapshot files)"
    )


if __name__ == "__main__":
    main()
