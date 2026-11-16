"""R64-6：把 overlay 从一份 auto-lang 检出物化进 `.sdk-source`（可复现步骤）。

两层快照的第二层（ACE crates + auto-tool-execution + stdlib/ace）声明的哈希在
`sdk/overlay/overlay.json`。本脚本从给定的 auto-lang 检出复制这些文件，然后调用
`verify_overlay.py` 复核；任何哈希不符都会红（不会留下半应用状态：先复制到临时
目录、全量校验后再替换）。

用法：`python tools/apply_overlay.py <auto-lang-checkout>`（cwd = 驱动目录）。
"""
import hashlib
import json
import pathlib
import shutil
import sys

root = pathlib.Path(__file__).resolve().parents[1]
sdk = root / ".sdk-source"
manifest_path = root / "sdk" / "overlay" / "overlay.json"
if len(sys.argv) != 2:
    print("usage: python tools/apply_overlay.py <auto-lang-checkout>")
    sys.exit(2)
source = pathlib.Path(sys.argv[1]).resolve()
manifest = json.loads(manifest_path.read_text())
files = manifest["files"]
staging = root / ".sdk-overlay.tmp"
if staging.exists():
    shutil.rmtree(staging)
staging.mkdir()
problems = []
for rel, want in files.items():
    src = source / rel
    if not src.is_file():
        problems.append(f"missing in source: {rel}")
        continue
    got = hashlib.sha256(src.read_bytes()).hexdigest()
    if got != want:
        problems.append(f"hash mismatch: {rel}")
        continue
    dst = staging / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)
if problems:
    shutil.rmtree(staging, ignore_errors=True)
    print("FAIL overlay not applied:")
    for p in problems[:20]:
        print(" ", p)
    sys.exit(1)
for rel in files:
    dst = sdk / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(staging / rel, dst)
shutil.rmtree(staging, ignore_errors=True)
print(f"APPLIED overlay: {len(files)} files from {source}")