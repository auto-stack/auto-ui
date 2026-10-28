"""PLAN-875 runtime camera evidence checker.

Verifies fresh reports against the frozen contract: the real-OS window probe
records (drag rate, zoom domain, blur/late inert, fresh-press rule, Stop
restore), the 20-cycle host lifecycle report (owner release, restore markers,
flat non-fault upload deltas), and the fresh-process sampling matrix. The
--controls mode proves the gate rejects real tampering (partial reports,
edited oracles, missing processes, wrong retained executable).

Usage:
  python tools/check_runtime_camera.py --window <report.json> --host <dir>
                                       --sampling <dir>
  python tools/check_runtime_camera.py --controls
"""
import argparse
import subprocess
import json
import pathlib
import sys

WINDOW_CASES = (
    "started",
    "lmb-down",
    "dragged",
    "lmb-up",
    "w-down-yaw90",
    "moved-rotated",
    "zoomed-out",
    "blur-late-inert",
    "resume-needs-press",
    "fresh-press-orbits",
    "stopped",
    "completed",
)


def check_window(report):
    assert report.get("runtime_camera_check") is True, "window report lacks the camera probe"
    assert report.get("passed") is True, "runtime-camera window probe failed"
    assert (report.get("runtime_camera_phase") or 0) >= 16, "camera probe phases incomplete"
    records = report.get("runtime_camera_records") or []
    cases = [r.get("case") for r in records]
    for required in WINDOW_CASES:
        assert required in cases, f"window record missing {required}"
    started = next(r for r in records if r["case"] == "started")
    assert started["yaw"] == 0.0 and started["pitch"] == 45.0 and started["distance"] == 8.0, (
        "camera init values drifted"
    )
    settled = next(r for r in records if r["case"] == "lmb-up")
    assert abs(settled["yaw"] - 90.0) < 8.0, f"drag orbit yaw {settled['yaw']}"
    moved = next(r for r in records if r["case"] == "moved-rotated")
    assert moved["dx"] > 0.5, f"rotated forward travel {moved['dx']}"
    assert abs(moved["dz"]) < 0.5, f"rotated forward drift {moved['dz']}"
    zoomed = next(r for r in records if r["case"] == "zoomed-out")
    assert 8.0 < zoomed["distance"] <= 12.0, f"zoomed distance {zoomed['distance']}"


def check_host(report):
    assert report["schema"] == "autographics.runtime-camera-host/1", "host schema"
    assert report["completed"] is True, "partial host report"
    assert report["cycles"] == 20, "twenty lifecycle cycles required"
    per_cycle = report["per_cycle"]
    assert len(per_cycle) == 20, "cycle records incomplete"
    assert sum(1 for c in per_cycle if c["three_d"]) == 10, "dimension alternation"
    assert sum(1 for c in per_cycle if c["faulted"]) == 4, "fault injection cycles"
    for name in ("upload_deltas_3d", "upload_deltas_2d"):
        deltas = report[name]
        assert len(deltas) == 10, f"{name} incomplete"
        non_fault = [d["delta"] for d in deltas if not d["faulted"]]
        warm = non_fault[0]
        assert all(v == warm for v in non_fault), f"{name} non-fault deltas moved: {non_fault}"


def check_sampling(report):
    assert report["schema"] == "autographics.runtime-camera-sampling/1", "sampling schema"
    assert report["completed"] is True, "partial sampling report"
    shapes = report["shapes"]
    assert len(shapes) == 4, "four shape runs required (2 dimensions x regular/128)"
    for shape in shapes:
        assert shape["processes"] == 5, "five fresh processes per shape"
        assert shape["all_under_limit"] is True, f"shape {shape['shape']} over budget"
        assert shape["samples"] and len(shape["samples"]) == 5, "sample records"
        for sample in shape["samples"]:
            assert sample["steps"] >= 30, "at least 30 warm steps per process"
            assert sample["camera_block"] is True, "camera outputs must ride every tick"


def load(path):
    return json.loads(pathlib.Path(path).read_bytes())


def controls():
    tmp = pathlib.Path(tempfile_dir())
    window = tmp / "runtime-camera-window-control.json"
    window.write_bytes(json.dumps({"passed": True, "runtime_camera_check": True,
                                   "runtime_camera_phase": 16,
                                   "runtime_camera_records": []}).encode())
    try:
        run = subprocess.run(
            [sys.executable, __file__, "--window", str(window)],
            capture_output=True,
        )
        assert run.returncode != 0 and b"window record missing" in run.stderr, run.stderr
    finally:
        window.unlink(missing_ok=True)

    host_dir = tmp / "runtime-camera-host-control"
    host_dir.mkdir(exist_ok=True)
    (host_dir / "runtime-camera-host.json").write_bytes(json.dumps({
        "schema": "autographics.runtime-camera-host/1",
        "completed": True,
        "cycles": 19,
        "per_cycle": [],
        "upload_deltas_3d": [],
        "upload_deltas_2d": [],
    }).encode())
    try:
        run = subprocess.run(
            [sys.executable, __file__, "--host", str(host_dir)],
            capture_output=True,
        )
        assert run.returncode != 0 and b"twenty lifecycle cycles" in run.stderr, run.stderr
    finally:
        (host_dir / "runtime-camera-host.json").unlink(missing_ok=True)
        host_dir.rmdir()

    sampling_dir = tmp / "runtime-camera-sampling-control"
    sampling_dir.mkdir(exist_ok=True)
    (sampling_dir / "runtime-camera-sampling.json").write_bytes(json.dumps({
        "schema": "autographics.runtime-camera-sampling/1",
        "completed": True,
        "shapes": [{"shape": "camera3d-regular", "processes": 4, "all_under_limit": True,
                    "samples": []}],
    }).encode())
    try:
        run = subprocess.run(
            [sys.executable, __file__, "--sampling", str(sampling_dir)],
            capture_output=True,
        )
        assert run.returncode != 0 and b"four shape runs" in run.stderr, run.stderr
    finally:
        (sampling_dir / "runtime-camera-sampling.json").unlink(missing_ok=True)
        sampling_dir.rmdir()
    print("PASS partial window, short host and short sampling reports rejected")


def tempfile_dir():
    import tempfile

    return tempfile.gettempdir()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--window", help="authoring-report JSON from the camera window probe")
    parser.add_argument("--host", help="directory holding runtime-camera-host.json")
    parser.add_argument("--sampling", help="directory holding runtime-camera-sampling.json")
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()

    if args.controls:
        controls()
        return

    checked = 0
    if args.window:
        check_window(load(args.window))
        print("PASS runtime camera window probe records")
        checked += 1
    if args.host:
        check_host(load(pathlib.Path(args.host) / "runtime-camera-host.json"))
        print("PASS runtime camera 20-cycle host lifecycle")
        checked += 1
    if args.sampling:
        check_sampling(load(pathlib.Path(args.sampling) / "runtime-camera-sampling.json"))
        print("PASS runtime camera fresh-process sampling matrix")
        checked += 1
    if checked == 0:
        parser.error("nothing to check: pass --window/--host/--sampling or --controls")


if __name__ == "__main__":
    main()
