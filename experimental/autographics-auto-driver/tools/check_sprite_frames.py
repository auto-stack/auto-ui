"""PLAN-004 sprite-frame acceptance gate.

Runs the driver's public sprite suite with report emission and validates the
emitted evidence strictly: the profile/admission/budget/pixel/warm-owner cases
must exist, name the right runner, and carry values that a plausible wrong
implementation could not produce. Partial runs, missing cases, a foreign runner
or a tampered pixel/budget value are rejected.

  python experimental/autographics-auto-driver/tools/check_sprite_frames.py
  python experimental/autographics-auto-driver/tools/check_sprite_frames.py --controls
  python experimental/autographics-auto-driver/tools/check_sprite_frames.py <report-dir>

With no directory the suite is executed fresh; with one, existing reports are
validated (used by --controls, which restores every mutation).
"""
import json
import os
import re
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = "autographics.plan004.report/1"
RUNNER = "public_sprite_frames"
CASES = ["admission", "budgets", "pixels", "profile20", "warm"]
DECODED = 16 * 1024 * 1024
UNION = 64 * 1024 * 1024
RAW_TOTAL = 32 * 1024 * 1024
SHEET_BYTES = 96 * 32 * 4


def load(directory):
    reports = {}
    for case in CASES:
        path = directory / f"{case}.json"
        assert path.is_file(), f"missing case report: {case}"
        report = json.loads(path.read_text())
        assert report.get("schema") == SCHEMA, f"wrong report schema: {case}"
        assert report.get("case") == case, f"case name mismatch: {case}"
        assert report.get("runner") == RUNNER, f"wrong runner: {case}: {report.get('runner')}"
        reports[case] = report["value"]
    return reports


def hex64(value):
    return isinstance(value, str) and len(value) == 64 and all(
        c in "0123456789abcdef" for c in value
    )


def red(pixel):
    return pixel[0] > 150 and pixel[2] < 70


def blue(pixel):
    return pixel[2] > 150 and pixel[0] < 70


def validate(reports):
    profile = reports["profile20"]
    assert hex64(profile["managed_hash"]), "profile identity"
    assert 60 <= profile["block_len"] <= 116, f"block length {profile['block_len']}"
    frames = profile["frames"]
    assert {0, 32, 64} <= set(frames), f"frame walk incomplete: {set(frames)}"
    assert profile["ticks"] >= 24, "probe ticks"
    assert profile["vm_owner"] is True, "the Run VM owner was released mid-run"

    pixels = reports["pixels"]
    assert red(pixels["frame0_left"]), f"frame 0 left not red: {pixels['frame0_left']}"
    assert blue(pixels["frame0_right"]), f"frame 0 right not blue: {pixels['frame0_right']}"
    assert blue(pixels["flipped_left"]), f"flipped left not blue: {pixels['flipped_left']}"
    assert red(pixels["flipped_right"]), f"flipped right not red: {pixels['flipped_right']}"

    budgets = reports["budgets"]
    assert budgets["max_side"] == 2048, "side ceiling moved"
    assert budgets["max_decoded"] == DECODED, "decoded ceiling moved"
    assert budgets["max_cpu_gpu_union"] == UNION, "CPU/GPU union moved"
    assert budgets["max_raw_total"] == RAW_TOTAL, "raw budget moved"
    assert budgets["max_assets"] == 8, "identity budget moved"
    assert budgets["accepted_16mib"] == DECODED, "the 2048x2048 image is not 16MiB"
    assert budgets["fifth_rejected_before_decode"] is True

    admission = reports["admission"]
    assert admission["rejected_cases"] >= 9, "admission counterexamples missing"
    assert hex64(admission["accepted_sha"]), "accepted identity"
    assert (admission["accepted_width"], admission["accepted_height"]) == (96, 32)

    warm = reports["warm"]
    assert warm["rounds"] == 20, "round count"
    assert warm["textures_created"] == 1, f"textures created: {warm['textures_created']}"
    assert warm["bytes_uploaded"] == SHEET_BYTES, "pixel re-upload on warm frames"
    assert warm["textures_after_close"] == 0, "texture outlived its payload"
    assert warm["owner_released"] is True, "weak owner never released"
    print(
        "PASS PLAN-004 sprite frames: 5 cases; frames 0/32/64, flipped pixels, "
        "2048/16MiB/64MiB/8 budgets, 20 rounds = 1 texture / 12288 bytes"
    )


def run_suite():
    directory = pathlib.Path(tempfile.mkdtemp(prefix="plan004-reports-"))
    environment = dict(os.environ, PLAN004_REPORT_DIR=str(directory))
    environment.pop("RUSTC_WRAPPER", None)
    environment.setdefault("RUST_MIN_STACK", "16777216")
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--locked",
            "--test",
            "public_sprite_frames",
            "--",
            "--test-threads=1",
        ],
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
    )
    output = result.stdout + result.stderr
    assert result.returncode == 0, output[-4000:]
    # The case reports below are the real coverage gate; the count only proves
    # the binary ran its suites rather than a filtered subset.
    match = re.search(r"test result: ok\. (\d+) passed; 0 failed", output)
    assert match, output[-2000:]
    assert int(match.group(1)) >= 17, f"only {match.group(1)} cases ran"
    return directory


def controls(directory):
    """Tampered, partial and foreign reports must all be rejected."""

    def rejected(reports_dir, needle):
        try:
            validate(load(reports_dir))
        except (AssertionError, KeyError, json.JSONDecodeError) as error:
            assert needle in str(error), f"unexpected rejection: {error}"
            return
        raise AssertionError(f"tampered reports were accepted: {needle}")

    work = pathlib.Path(tempfile.mkdtemp(prefix="plan004-controls-"))
    try:
        # A pixel that a wrong frame/flip would produce.
        copy = work / "pixel"
        shutil.copytree(directory, copy)
        report = json.loads((copy / "pixels.json").read_text())
        report["value"]["frame0_left"] = [0, 0, 255, 255]
        (copy / "pixels.json").write_text(json.dumps(report))
        rejected(copy, "frame 0 left not red")

        # A missing case is a partial run, not a pass.
        copy = work / "partial"
        shutil.copytree(directory, copy)
        (copy / "warm.json").unlink()
        rejected(copy, "missing case report: warm")

        # A report produced by another runner is refused.
        copy = work / "runner"
        shutil.copytree(directory, copy)
        report = json.loads((copy / "budgets.json").read_text())
        report["runner"] = "sprite_resources"
        (copy / "budgets.json").write_text(json.dumps(report))
        rejected(copy, "wrong runner: budgets")

        # A silently raised budget is refused.
        copy = work / "budget"
        shutil.copytree(directory, copy)
        report = json.loads((copy / "budgets.json").read_text())
        report["value"]["max_side"] = 4096
        (copy / "budgets.json").write_text(json.dumps(report))
        rejected(copy, "side ceiling moved")

        # A warm run that re-uploaded pixels is refused.
        copy = work / "warm"
        shutil.copytree(directory, copy)
        report = json.loads((copy / "warm.json").read_text())
        report["value"]["bytes_uploaded"] = SHEET_BYTES * 2
        (copy / "warm.json").write_text(json.dumps(report))
        rejected(copy, "pixel re-upload")

        # Restored evidence still passes.
        validate(load(directory))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    print("PASS PLAN-004 report controls: pixel/partial/runner/budget/warm all rejected")


if __name__ == "__main__":
    arguments = [a for a in sys.argv[1:] if not a.startswith("--")]
    with_controls = "--controls" in sys.argv
    if arguments:
        reports_dir = pathlib.Path(arguments[0]).resolve()
    else:
        reports_dir = run_suite()
    try:
        validate(load(reports_dir))
        if with_controls:
            controls(reports_dir)
    finally:
        if not arguments:
            shutil.rmtree(reports_dir, ignore_errors=True)
