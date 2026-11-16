"""PLAN-981：把两层快照的 overlay 清单刷新到当前 auto-lang 检出（A 正式源 + #6i）。

用法（cwd = 驱动目录）：python tools/refresh_overlay.py <auto-lang-checkout> <commit-stamp>

- 按既有清单的**路径集合**取当前源文件，重算 sha256，写回 `sdk/overlay/overlay.json`
  （`source` 记录 repo + 提交戳）；
- 随后调用 `apply_overlay.py` 物化到 `.sdk-source` 并复核（半应用不留状态）。
"""
import hashlib
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
manifest_path = root / "sdk" / "overlay" / "overlay.json"
if len(sys.argv) != 3:
    print("usage: python tools/refresh_overlay.py <auto-lang-checkout> <commit-stamp>")
    sys.exit(2)
source = pathlib.Path(sys.argv[1]).resolve()
stamp = sys.argv[2]

manifest = json.loads(manifest_path.read_text())
paths = sorted(manifest["files"].keys())
files = {}
missing = []
for rel in paths:
    src = source / rel
    if not src.is_file():
        missing.append(rel)
        continue
    files[rel] = hashlib.sha256(src.read_bytes()).hexdigest()
if missing:
    print(f"FAIL {len(missing)} overlay paths are missing in {source}:")
    for rel in missing[:10]:
        print(" ", rel)
    sys.exit(1)

manifest["source"] = f"auto-lang {stamp}: experimental/ace + crates/auto-tool-execution + stdlib/ace"
manifest["files"] = dict(sorted(files.items()))
manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
print(f"REFRESHED overlay manifest: {len(files)} files from {source} ({stamp})")
rc = subprocess.call([sys.executable, str(root / "tools" / "apply_overlay.py"), str(source)])
sys.exit(rc)