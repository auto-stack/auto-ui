"""PLAN-981：把两层快照的 overlay 清单刷新到一份 auto-lang 检出（可从落主提交复现）。

用法（cwd = 驱动目录）：python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]

- 省略 stamp 时用 `git -C <checkout> rev-parse --short HEAD` 推导（不接受自由文本冒充来源）；
- overlay 覆盖路径 = 该提交里 `experimental/ace`、`crates/auto-tool-execution`、`stdlib/ace`
  下**全部已跟踪文件**（新文件自动纳入，避免新 stdlib 文件漏拷）；
- 要求这些路径在该检出里干净；哈希前把 CRLF 规范成 LF（与 git blob 字节一致），
  `apply_overlay.py` 同样规范化写盘，因此"清单 == 落主 blob"可被逐 blob 核验；
- 写回 `sdk/overlay/overlay.json` 后调用 `apply_overlay.py` 物化并复核。

注意：`prepare_sdk.py --verify-only` 只比较 `.sdk-source` 与 `source-lock.json` + 本清单，
**不会**回查 stamp 对应的 git blob；要证明"清单字节 == 落主提交字节"，另跑逐 blob 保真核验。
"""
import hashlib
import json
import pathlib
import subprocess
import sys

ROOTS = ("experimental/ace", "crates/auto-tool-execution", "stdlib/ace")

root = pathlib.Path(__file__).resolve().parents[1]
manifest_path = root / "sdk" / "overlay" / "overlay.json"
if len(sys.argv) not in (2, 3):
    print("usage: python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]")
    sys.exit(2)
source = pathlib.Path(sys.argv[1]).resolve()


def normalize(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def git(*args: str) -> str:
    out = subprocess.run(
        ["git", "-C", str(source), *args], capture_output=True, text=True
    )
    if out.returncode != 0:
        print(f"FAIL git {' '.join(args)}: {out.stderr.strip()}")
        sys.exit(1)
    return out.stdout


head = git("rev-parse", "--short", "HEAD").strip()
stamp = sys.argv[2] if len(sys.argv) == 3 else head
if stamp != head:
    print(f"FAIL stamp `{stamp}` does not name the checkout HEAD `{head}`")
    sys.exit(1)

paths = [
    line.strip()
    for line in git("ls-files", "--", *ROOTS).splitlines()
    if line.strip()
]
if not paths:
    print(f"FAIL no tracked overlay paths under {ROOTS} in {source}")
    sys.exit(1)

# dirty check over the whole overlay roots (single status call; Windows cmdline caps out)
status = git("status", "--porcelain", "--", *ROOTS)
dirty = [line for line in status.splitlines() if line.strip()]
if dirty:
    print(f"FAIL {len(dirty)} overlay paths are dirty in {source}; commit them first:")
    for line in dirty[:10]:
        print(" ", line)
    sys.exit(1)

files = {}
missing = []
for rel in paths:
    src = source / rel
    if not src.is_file():
        missing.append(rel)
        continue
    files[rel] = hashlib.sha256(normalize(src.read_bytes())).hexdigest()
if missing:
    print(f"FAIL {len(missing)} overlay paths are missing in {source}:")
    for rel in missing[:10]:
        print(" ", rel)
    sys.exit(1)

manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
manifest["source"] = f"auto-lang {stamp}: experimental/ace + crates/auto-tool-execution + stdlib/ace"
manifest["files"] = dict(sorted(files.items()))
manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
print(f"REFRESHED overlay manifest: {len(files)} files from {source} ({stamp})")
rc = subprocess.call([sys.executable, str(root / "tools" / "apply_overlay.py"), str(source)])
sys.exit(rc)