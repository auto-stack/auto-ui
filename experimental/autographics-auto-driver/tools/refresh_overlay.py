"""PLAN-981：把两层快照的 overlay 清单刷新到一份 auto-lang 检出（可从落主提交逐字节复现）。

用法（cwd = 驱动目录）：python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]

- 省略 stamp 时用 `git -C <checkout> rev-parse --short HEAD` 推导（不接受自由文本冒充来源）；
- overlay 覆盖路径 = 该检出 HEAD 里 `experimental/ace`、`crates/auto-tool-execution`、
  `stdlib/ace` 下**全部已跟踪文件**（新文件自动纳入）；
- 要求这些路径在工作树里干净；随后用 `git archive HEAD` 抽取到临时目录，从**blob 字节**
  计算哈希并交给 `apply_overlay.py` 物化——工作树的 CRLF/编码差异不会污染清单
  （快照里少数故意 CRLF 的 fixture 保持原字节）；
- 写回 `sdk/overlay/overlay.json` 后调用 `apply_overlay.py` 物化并复核。

注意：`prepare_sdk.py --verify-only` 只比较 `.sdk-source` 与 `source-lock.json` + 本清单，
**不会**回查 stamp 对应的 git blob；要证明"清单字节 == 落主提交字节"，另跑逐 blob 保真核验。
"""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOTS = ("experimental/ace", "crates/auto-tool-execution", "stdlib/ace")

root = pathlib.Path(__file__).resolve().parents[1]
manifest_path = root / "sdk" / "overlay" / "overlay.json"
if len(sys.argv) not in (2, 3):
    print("usage: python tools/refresh_overlay.py <auto-lang-checkout> [commit-stamp]")
    sys.exit(2)
source = pathlib.Path(sys.argv[1]).resolve()


def git(*args: str, binary: bool = False):
    out = subprocess.run(["git", "-C", str(source), *args], capture_output=True)
    if out.returncode != 0:
        print(f"FAIL git {' '.join(args)}: {out.stderr.decode(errors='replace').strip()}")
        sys.exit(1)
    return out.stdout if binary else out.stdout.decode()


head = git("rev-parse", "--short", "HEAD").strip()
stamp = sys.argv[2] if len(sys.argv) == 3 else head
if stamp != head:
    print(f"FAIL stamp `{stamp}` does not name the checkout HEAD `{head}`")
    sys.exit(1)

status = git("status", "--porcelain", "--", *ROOTS)
dirty = [line for line in status.splitlines() if line.strip()]
if dirty:
    print(f"FAIL {len(dirty)} overlay paths are dirty in {source}; commit them first:")
    for line in dirty[:10]:
        print(" ", line)
    sys.exit(1)

extract = pathlib.Path(tempfile.mkdtemp(prefix="plan981-overlay-"))
try:
    archive = git("archive", "--format=tar", "HEAD", *ROOTS, binary=True)
    tar_path = extract / "overlay.tar"
    tar_path.write_bytes(archive)
    with tarfile.open(tar_path) as tf:
        tf.extractall(extract)
    tar_path.unlink()

    files = {}
    missing = []
    for rel in [
        line.strip()
        for line in git("ls-files", "--", *ROOTS).splitlines()
        if line.strip()
    ]:
        blob = extract / rel
        if not blob.is_file():
            missing.append(rel)
            continue
        files[rel] = hashlib.sha256(blob.read_bytes()).hexdigest()
    if missing:
        print(f"FAIL {len(missing)} overlay paths are missing from the archive:")
        for rel in missing[:10]:
            print(" ", rel)
        sys.exit(1)

    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    manifest["source"] = (
        f"auto-lang {stamp}: experimental/ace + crates/auto-tool-execution + stdlib/ace"
    )
    manifest["files"] = dict(sorted(files.items()))
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(f"REFRESHED overlay manifest: {len(files)} files from {source} ({stamp})")
    rc = subprocess.call([sys.executable, str(root / "tools" / "apply_overlay.py"), str(extract)])
    sys.exit(rc)
finally:
    shutil.rmtree(extract, ignore_errors=True)