"""PLAN-002 batch gate: frozen source identity plus the trigger/round test surface.

Run on the GPU machine, single instance:

  python tools/check_runtime_triggers.py [--controls]

The gate is the frozen test entry from the T-02 contract section 8: the three
PLAN-002 suites single-threaded, the affected PLAN-001 suites replayed (the
assembled compositions changed), the managed-envelope guard and the lib
admission tests. --controls additionally re-runs verify_plan002's deliberate
mutation controls and the SDK-only metadata closure.
"""
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SUITES = [
    "--test runtime_triggers_api",
    "--test runtime_round_host",
    "--test runtime_trigger_samples",
]
REPLAY = [
    "--test runtime_objects_api",
    "--test runtime_objects_host",
    "--test runtime_objects_samples",
]
LIB_FILTERS = [
    "managed_source_envelope",
    "triggers_admission",
    "runtime_round",
]


def run(cmd, label):
    print(f"[check_runtime_triggers] {label}: {' '.join(cmd)}", flush=True)
    result = subprocess.run(cmd, cwd=ROOT)
    assert result.returncode == 0, f"{label} failed with {result.returncode}"


def main():
    controls = "--controls" in sys.argv
    run([sys.executable, str(ROOT / "tools/verify_plan002.py")], "identity")
    for suite in SUITES:
        run(
            ["cargo", "test", "--locked", *suite.split(), "--", "--test-threads=1"],
            f"suite {suite}",
        )
    for suite in REPLAY:
        run(
            ["cargo", "test", "--locked", *suite.split(), "--", "--test-threads=1"],
            f"replay {suite}",
        )
    for pattern in LIB_FILTERS:
        run(
            ["cargo", "test", "--locked", "--lib", pattern, "--", "--test-threads=1"],
            f"lib {pattern}",
        )
    if controls:
        run([sys.executable, str(ROOT / "tools/verify_plan002.py"), "--controls"], "identity controls")
        run(
            [sys.executable, str(ROOT / "tools/verify_source.py"), "--sdk-only"],
            "sdk closure",
        )
    print("PASS check_runtime_triggers: identity, 3 PLAN-002 suites, 3 PLAN-001 replays, lib gates")


if __name__ == "__main__":
    main()
