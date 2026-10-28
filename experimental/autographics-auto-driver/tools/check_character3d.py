"""PLAN-874 character3d/1 evidence checker.

Verifies fresh reports against the frozen contract r1: action/trace binding,
three-dimensional kinematic oracles, the frozen 26-credit move path, layout
geometry sensitivity, the speed source-edit proof, host publication bindings,
the real-window probe records and the three-shape/ five-process sampling
matrix. --controls proves the gate rejects real tampering (partial reports,
edited oracles, trace divergence, wrong retained executable).

Usage:
  python tools/check_character3d.py <report.json...> --executable <retained-exe>
  python tools/check_character3d.py --controls --executable <retained-exe>
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

MAX_CREDITS = 32
MOVE_CREDITS = 26
MAX_STEPS = 1800
DT = 1.0 / 60.0
WINDOW_CASES = (
    "started",
    "w-down",
    "moved-forward",
    "space-tap",
    "jumped",
    "blur-paused",
    "blur-cleared",
    "stopped",
    "completed",
)


def sha256(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def check_actions(report):
    assert report["schema"] == "autographics.character3d-actions/1", "actions schema"
    assert report["completed"] is True, "partial actions report"
    assert 0 < report["steps"] <= MAX_STEPS, "step bound"
    assert abs(report["dt"] - DT) < 1e-9, "fixed dt drifted"
    assert 0 < report["credits_max"] <= MAX_CREDITS, "credit bound"
    assert report["credits_max"] >= MOVE_CREDITS, "move cost below the frozen path"
    identity = report["identity"]
    assert identity["version"] == "autographics.character3d/1", "profile version"
    assert identity["user_bytes"] <= 256 * 1024, "user source bound"
    checkpoints = report["checkpoints"]
    assert checkpoints and checkpoints[0]["step"] == 1, "checkpoints must start at 1"
    for cp in checkpoints:
        assert 0 < cp["step"] <= report["steps"], "checkpoint step bound"
        for axis in ("x", "y", "z"):
            assert axis in cp, f"checkpoint {cp['step']} missing {axis}"
        assert cp["y"] > -1.0, f"grounded path violated at {cp['step']}: {cp['y']}"
    apex = report["apex_y"]
    assert 1.9 <= apex <= 2.3, f"jump apex {apex} outside the frozen kinematics"
    assert report["forward_travel_z"] < -3.0, "no real three-dimensional travel"


def check_host(report):
    assert report["schema"] == "autographics.character3d-host/1", "host schema"
    assert report["completed"] is True, "partial host report"
    assert report["owner_cycles"] == 20, "owner cycles incomplete"
    assert report["restored_editing_pixels"] is True, "stop did not restore pixels"
    assert report["fault_last_good"] is True, "fault must keep last-good"
    assert report["profile"] == "autographics.character3d/1", "host profile identity"
    checkpoints = report["checkpoints"]
    assert checkpoints, "no checkpoints"
    for cp in checkpoints:
        assert cp["matches_trace"] is True, f"checkpoint {cp['step']} off trace"
        assert cp["published"] == cp["expected"], f"checkpoint {cp['step']} divergence"
        assert len(cp["published"]) == 3, "checkpoint must bind all three axes"


def check_layouts(report):
    assert report["schema"] == "autographics.character3d-layouts/1", "layouts schema"
    assert report["completed"] is True, "partial layouts report"
    assert report["layout_a"]["fell_in_pit"] is False, "layout A must stay grounded"
    assert report["layout_a"]["min_y"] > -1.0, "layout A geometry oracle"
    assert report["layout_b"]["fell_in_pit"] is True, "layout B must fall into the pit"
    assert report["layout_b"]["min_y"] < -3.0, "layout B pit oracle"
    tail_a = report["layout_a"]["path_tail"]
    tail_b = report["layout_b"]["path_tail"]
    assert len(tail_a) == len(tail_b) and tail_a != tail_b, "geometry oracle degenerate"
    speed = report["speed"]
    assert abs(speed["z60_fast"]) > abs(speed["z60_slow"]) * 1.2, "speed edit must move faster"


def check_sampling(report):
    assert report["schema"] == "autographics.character3d-sampling-matrix/1", "sampling schema"
    assert report["completed"] is True, "partial sampling report"
    assert report["warm_steps_dropped_each"] >= 30, "warm steps must be dropped"
    shapes = {s["shape"]: s for s in report["shapes"]}
    assert set(shapes) == {"regular", "wide128", "deep128"}, f"sampling shapes {sorted(shapes)}"
    for name, shape in shapes.items():
        assert shape["processes"] == 5, f"{name}: sampling must use five fresh processes"
        assert len(shape["samples"]) == 5, f"{name}: sampling matrix incomplete"
        assert shape["all_under_limit"] is True, f"{name}: a process exceeded the callback limit"
        assert shape["tick_cpu_ns_p95_median"] < report["callback_limit_ns"], f"{name}: P95 over budget"


def check_window(report):
    assert report.get("character3d_check") is True, "window report lacks the 3D character probe"
    assert report.get("passed") is True, "character3d window probe failed"
    assert (report.get("character3d_phase") or 0) >= 10, "character3d probe phases incomplete"
    records = report.get("character3d_records") or []
    cases = {r.get("case") for r in records}
    for required in WINDOW_CASES:
        assert required in cases, f"window record missing {required}"
    by_case = {r.get("case"): r for r in records}
    spawn = by_case["started"]["spawn"]
    assert by_case["moved-forward"]["z"] < spawn[2] - 0.5, "no real forward travel"
    assert by_case["jumped"]["apex_y"] > spawn[1] + 0.5, "no real jump"


def check_report(path, runner_shas):
    report = json.loads(pathlib.Path(path).read_bytes())
    schema = report.get("schema")
    if schema == "autographics.character3d-actions/1":
        check_actions(report)
    elif schema == "autographics.character3d-host/1":
        check_host(report)
    elif schema == "autographics.character3d-layouts/1":
        check_layouts(report)
    elif schema == "autographics.character3d-sampling-matrix/1":
        check_sampling(report)
    elif schema == "autographics-authoring-report/1":
        check_window(report)
    else:
        raise AssertionError(f"unknown report schema {schema}")
    bound = report.get("runner_sha256")
    if bound is not None:
        assert bound in runner_shas, "runner executable mismatch"
    return schema


REPORTS = []


def controls(exe_arg):
    with tempfile.TemporaryDirectory() as td:
        td = pathlib.Path(td)

        def must_fail(name, mutate):
            work = td / name
            work.mkdir()
            for src in REPORTS:
                data = json.loads(pathlib.Path(src).read_bytes())
                data = mutate(data, pathlib.Path(src).name)
                (work / pathlib.Path(src).name).write_bytes(json.dumps(data).encode())
            result = subprocess.run(
                [sys.executable, __file__, *[str(p) for p in work.glob("*.json")],
                 "--executable", exe_arg],
                capture_output=True,
            )
            assert result.returncode != 0, f"control {name} must fail"
            print(f"PASS control {name}: rejected")

        def drop_field(data, name):
            data.pop("completed", None)
            return data

        def zero_selected(data, name):
            if data.get("schema") == "autographics.character3d-actions/1":
                data["steps"] = 0
            if data.get("schema") == "autographics.character3d-sampling-matrix/1":
                data["shapes"][0]["samples"] = []
            return data

        def edit_oracle(data, name):
            if data.get("schema") == "autographics.character3d-layouts/1":
                data["layout_b"]["fell_in_pit"] = False
            if data.get("schema") == "autographics.character3d-host/1":
                data["restored_editing_pixels"] = False
            if data.get("schema") == "autographics-authoring-report/1":
                data["character3d_records"] = []
            return data

        def break_trace(data, name):
            if data.get("schema") == "autographics.character3d-host/1":
                data["checkpoints"][0]["published"] = [9.0, 9.0, 9.0]
            if data.get("schema") == "autographics.character3d-actions/1":
                data["credits_max"] = MAX_CREDITS + 1
            return data

        must_fail("partial", drop_field)
        must_fail("zero-selected", zero_selected)
        must_fail("oracle-edit", edit_oracle)
        must_fail("trace-divergence", break_trace)

        work = td / "wrong-exe"
        work.mkdir()
        fake = work / "fake-exe"
        fake.write_bytes(b"not the runner")
        tampered = json.loads(pathlib.Path(REPORTS[0]).read_bytes())
        tampered["runner_sha256"] = "0" * 64
        (work / "bound.json").write_bytes(json.dumps(tampered).encode())
        result = subprocess.run(
            [sys.executable, __file__, str(work / "bound.json"), "--executable", str(fake)],
            capture_output=True,
        )
        assert result.returncode != 0, "runner binding must reject mismatch"
        print("PASS control bound-runner-mismatch: rejected")


def main():
    global REPORTS
    parser = argparse.ArgumentParser()
    parser.add_argument("reports", nargs="*")
    parser.add_argument("--executable", action="append")
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if args.controls:
        here = pathlib.Path(__file__).resolve().parents[1]
        REPORTS = [str(p) for p in sorted((here / "research/character3d-reports").glob("*.json"))]
        assert REPORTS, "no fresh reports: run the report writers first"
        check_reports_main(REPORTS, args.executable)
        controls(args.executable[0])
        print("PASS PLAN-874 checker controls")
        return
    check_reports_main(args.reports, args.executable or [])
    print(f"PASS PLAN-874 character3d reports: {len(args.reports)} accepted")


def check_reports_main(reports, executables):
    assert reports, "no reports given"
    runner_shas = {sha256(e) for e in executables}
    seen = set()
    for path in reports:
        seen.add(check_report(path, runner_shas))
    expected = {
        "autographics.character3d-actions/1",
        "autographics.character3d-host/1",
        "autographics.character3d-layouts/1",
        "autographics.character3d-sampling-matrix/1",
    }
    assert expected <= seen, f"report families incomplete: {expected - seen}"


if __name__ == "__main__":
    main()
