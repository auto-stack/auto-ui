"""PLAN-981 SESSION S2：跑 `.sdk-source` 的第二层（overlay）校验。

第一层是 681 文件 frozen pin（`tools/prepare_sdk.py --verify-only` 的锁定集）。
S2 需要 ACE 的 typed 宿主，因此把 ACE crates、`auto-tool-execution`、`stdlib/ace`
以 **overlay** 形式叠在同一快照里（单一 auto-atom 源，不混 live/pinned 两份）。
本脚本对两层分别校验：

- base：`research/source-lock.json` 的 681 文件集合与哈希；
- overlay：`sdk/overlay/overlay.json` 记录的文件集合与哈希（overlay 缺/多/改都红）。

用法：`python tools/verify_overlay.py`（cwd = 驱动目录）。
"""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[1]
sdk = root / ".sdk-source"
lock = json.loads((root / "research" / "source-lock.json").read_text())["files"]
overlay_path = root / "sdk" / "overlay" / "overlay.json"
if not overlay_path.is_file():
    print("FAIL overlay manifest missing: sdk/overlay/overlay.json")
    sys.exit(1)
overlay = json.loads(overlay_path.read_text())["files"]
if not sdk.is_dir():
    print("FAIL .sdk-source missing (run tools/prepare_sdk.py --from-snapshot)")
    sys.exit(1)
actual = {
    v.relative_to(sdk).as_posix(): hashlib.sha256(v.read_bytes()).hexdigest()
    for v in sdk.rglob("*")
    if v.is_file() and "target" not in v.relative_to(sdk).parts
}
base = dict(actual)
problems = []
for rel, want in overlay.items():
    got = base.pop(rel, None)
    if got != want:
        problems.append(f"overlay mismatch: {rel}")
for rel in sorted(lock):
    got = base.pop(rel, None)
    if got != lock[rel]:
        problems.append(f"base mismatch: {rel}")
for rel in sorted(base):
    problems.append(f"unexpected file: {rel}")
if problems:
    print("FAIL")
    for p in problems[:20]:
        print(" ", p)
    sys.exit(1)
print(
    f"PASS two-layer SDK: base {len(lock)} files + overlay {len(overlay)} files"
    f" (overlay source: {json.loads(overlay_path.read_text()).get('source', '')})"
)