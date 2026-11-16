"""PLAN-981：把两层快照的 overlay 清单刷新到一份 auto-lang 检出（可从落主提交复现）。

用法（cwd = 驱动目录）：python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]

R65-7 之后本工具：
- 省略 stamp 时用 `git -C <checkout> rev-parse --short HEAD` 推导（不再接受任意自由文本冒充来源）；
- 刷新前要求 overlay 覆盖的路径在该检出里**干净**（`git status --porcelain` 为空），
  否则拒绝刷新——避免把工作树里的 CRLF/半成品写进“可复现”清单；
- 写回 `sdk/overlay/overlay.json` 后调用 `apply_overlay.py` 物化并复核。
"""
import hashlib
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
manifest_path = root / "sdk" / "overlay" / "overlay.json"
if len(sys.argv) not in (2, 3):
    print("usage: python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]")
    sys.exit(2)
source = pathlib.Path(sys.argv[1]).resolve()

manifest = json.loads(manifest_path.read_text())
paths = sorted(manifest["files"].keys())

# 1) the checkout must be clean for every overlay path (no dirty worktree capture).
#    (Windows command lines cap out at 32k, so query once and intersect locally.)
status = subprocess.run(
    ["git", "-C", str(source), "status", "--porcelain"],
    capture_output=True,
    text=True,
)
if status.returncode != 0:
    print(f"FAIL not a git checkout: {source}: {status.stderr.strip()}")
    sys.exit(1)
want = set(paths)
dirty = []
for line in status.stdout.splitlines():
    rel = line[3:].strip()
    if " -> " in rel:
        rel = rel.split(" -> ", 1)[1]
    if rel.strip('"') in want:
        dirty.append(line)
if dirty:
    print(f"FAIL {len(dirty)} overlay paths are dirty in {source}; commit them first:")
    for line in dirty[:10]:
        print(" ", line)
    sys.exit(1)

# 2) stamp is derived from the checkout unless explicitly given.
stamp = sys.argv[2] if len(sys.argv) == 3 else subprocess.run(
    ["git", "-C", str(source), "rev-parse", "--short", "HEAD"],
    capture_output=True,
    text=True,
    check=True,
).stdout.strip()
head = subprocess.run(
    ["git", "-C", str(source), "rev-parse", "--short", "HEAD"],
    capture_output=True,
    text=True,
    check=True,
).stdout.strip()
if stamp.split()[0] != head:
    print(f"FAIL stamp `{stamp}` does not name the checkout HEAD `{head}`")
    sys.exit(1)

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