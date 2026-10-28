"""Freeze the PLAN-874 source closure (character3d/1 public profile).

Run once per semantic revision, in the plan worktree, after the source set is
stable:

  python tools/gen_plan874_identity.py

The identity lists every source/evidence input of both experimental packages
(AGP is read-only and unchanged by this plan) plus the two pins the closure
claims: the PLAN-861 delivery commit this plan branched from and the frozen SDK
artifact pin. Volatile run reports and the identity file itself are excluded.
"""
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
SKIP_DIRS = {"target", "__pycache__", ".sdk-source"}
SKIP_NAMES = {"window-report.json", "plan874-identity.json", "character3d-reports"}


def digest(path):
    data = path.read_bytes()
    if path.suffix.lower() in TEXT_SUFFIXES or path.name.lower() in TEXT_SUFFIXES:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def main():
    files = {}
    folders = []
    for package in PACKAGES:
        package_root = BASE / package
        for child in sorted(package_root.iterdir()):
            if child.is_file():
                files[f"{package}/{child.name}"] = digest(child)
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
            files[relative.as_posix()] = digest(path)
    pin = json.loads((ROOT / "research/baseline.json.txt").read_text())["sdk"][
        "repository_commit"
    ]
    identity = {
        "plan": "PLAN-874",
        "base_commit": "8b12632d7b5b1b0df72fbb4eef84297a054de31d",
        "sdk_commit": pin,
        "text_suffixes": TEXT_SUFFIXES,
        "folders": sorted(folders),
        "files": dict(sorted(files.items())),
    }
    out = ROOT / "research/plan874-identity.json"
    out.write_text(json.dumps(identity, indent=1) + "\n")
    print(f"frozen PLAN-874 source identity: {len(identity['files'])} files")


if __name__ == "__main__":
    main()
