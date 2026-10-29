"""Validate the PLAN-001 env-gated host/sampling reports.

Usage (after the gated cargo test run):

  RUNTIME_OBJECTS_REPORT_DIR=<dir> python tools/check_runtime_objects.py

Checks the 20-cycle churn report (owner release, flat upload deltas) and the
five-fresh-process sampling matrix (8 live instances per tick, per-shape
medians inside the 250ms whole-tick line).
"""
import json
import os
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parents[1]
directory = pathlib.Path(os.environ["RUNTIME_OBJECTS_REPORT_DIR"])

host_path = directory / "runtime-objects-host.json"
host = json.loads(host_path.read_text())
assert host["schema"] == "autographics.runtime-objects-host/1", host["schema"]
assert host["completed"] is True
assert host["cycles"] == 20, host["cycles"]
assert host["births"] >= 20, host["births"]
for label in ["upload_deltas_3d", "upload_deltas_2d"]:
    rows = host[label]
    assert len(rows) == 10, (label, len(rows))
    warm = next(r["delta"] for r in rows if not r["faulted"])
    for i, row in enumerate(rows):
        if not row["faulted"]:
            assert row["delta"] == warm, (label, i, row["delta"], warm)
print(f"PASS churn: 20 cycles, births={host['births']}, warm upload delta flat ({warm})")

sampling_path = directory / "runtime-objects-sampling.json"
sampling = json.loads(sampling_path.read_text())
assert sampling["schema"] == "autographics.runtime-objects-sampling/1", sampling["schema"]
assert sampling["completed"] is True
shapes = {s["shape"]: s for s in sampling["shapes"]}
expected = {
    "objects2d-regular",
    "objects2d-wide120",
    "objects3d-regular",
    "objects3d-dense64",
}
assert set(shapes) == expected, set(shapes) ^ expected
for name, shape in sorted(shapes.items()):
    assert shape["processes"] == 5, name
    for sample in shape["samples"]:
        assert sample["live_instances"] == 8, (name, sample)
        assert sample["objects_block"] is True
    assert shape["all_under_limit"], (name, shape["tick_ns_p95_median"])
    print(
        f"PASS sampling {name}: p50 median {shape['tick_ns_p50_median']}ns, "
        f"p95 median {shape['tick_ns_p95_median']}ns < 250ms, 8 live instances"
    )
print("PASS PLAN-001 runtime-objects reports")
if "--jq" in sys.argv:
    print(json.dumps({"host": host["births"], "shapes": sorted(shapes)}))
