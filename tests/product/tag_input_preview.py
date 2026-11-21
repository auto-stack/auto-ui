"""Prepare and identify the development bundle used by the TagInput gate (PLAN-097)."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

RECEIPT = "tag-input-preview.json"
ROOT_INPUTS = {
    "index.html", "package.json", "pnpm-lock.yaml", "postcss.config.js",
    "postcss.config.cjs", "tailwind.config.js", "tailwind.config.ts", "vite.config.ts",
    "node_modules/vite/package.json",
}


def fingerprints(root, files):
    result = {}
    for path in sorted(set(files)):
        if path.is_file():
            with path.open("rb") as stream:
                result[path.relative_to(root).as_posix()] = hashlib.file_digest(stream, "sha256").hexdigest()
    return result


def source_fingerprints(root):
    files = [root / name for name in ROOT_INPUTS]
    files.extend(root.glob("tsconfig*.json"))
    for directory in ("src", "public"):
        files.extend((root / directory).rglob("*"))
    return fingerprints(root, files)


def bundle_fingerprints(root):
    return fingerprints(root, (root / "dist").rglob("*"))


def checked_preview(root):
    receipt_path = root / RECEIPT
    if not receipt_path.is_file():
        raise ValueError(f"prepared Vue receipt {RECEIPT} is missing")
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    if not isinstance(receipt, dict) or receipt.get("profile") != "development":
        raise ValueError("prepared Vue receipt does not describe a development build")
    if receipt.get("source") != source_fingerprints(root):
        raise ValueError("prepared Vue source changed since the preview build")
    assets = bundle_fingerprints(root)
    if "dist/index.html" not in assets or assets != receipt.get("dist"):
        raise ValueError("prepared Vue bundle is missing or differs from the preview receipt")
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app_dir", type=Path, help="generated Vue project with installed dependencies")
    root = parser.parse_args().app_dir.resolve()
    before = source_fingerprints(root)
    cli = root / "node_modules" / "vite" / "bin" / "vite.js"
    command = ["node", str(cli), "build", "--mode", "development", "--minify", "false"]
    started = time.monotonic()
    subprocess.run(command, cwd=root, env={**os.environ, "NODE_ENV": "development"}, check=True)
    if before != source_fingerprints(root):
        raise RuntimeError("generated Vue inputs changed during preview preparation")
    assets = bundle_fingerprints(root)
    if "dist/index.html" not in assets:
        raise RuntimeError("preview preparation produced no dist/index.html")
    receipt = {"source": before, "dist": assets, "command": command,
               "profile": "development", "seconds": time.monotonic() - started}
    pending = root / (RECEIPT + ".pending")
    pending.write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    pending.replace(root / RECEIPT)


if __name__ == "__main__":
    main()
