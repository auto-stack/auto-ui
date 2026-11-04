"""PLAN-044 node source-diagnostics verify controls: the --diagnostics-map
sidecar is opt-in (default CLI emits no map and byte-identical world), the
sidecar binds to the world/closure identities and tiles the world source,
regeneration is byte-identical, and the checked-in diag-fault product is
current. Driver-side consumption/mutation gates live in
autographics-auto-driver/tests/node_source_diagnostics.rs (invoked here).
Run from any cwd:

  python experimental/autographics-node2d-pilot/tools/verify_plan044.py --controls
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
FIXTURES = ROOT / "fixtures" / "components2d"
GENERATED = ROOT / "generated-components2d" / "diag-fault"
DRIVER_DIR = REPO / "experimental" / "autographics-auto-driver"

WORLD_SHA = "4b509064e65ac6b7cd0fdbf75e518e9b325dda2775fbbafad89ebb32ed8f24ff"
CLOSURE = {"main", "helpera", "helperb", "broken", "resources"}


def sha256(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def run_lower(entry: pathlib.Path, out: pathlib.Path, diagnostics_map: bool) -> None:
    cmd = [
        "cargo", "run", "--locked", "--offline", "--bin", "node2d-lower", "--",
        "--profile", "components-2d",
        "--entry", str(entry),
        "--out", str(out),
    ]
    if diagnostics_map:
        cmd.append("--diagnostics-map")
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    if r.returncode != 0:
        raise AssertionError(f"lowering failed for {entry}: {r.stderr.strip()[:400]}")


def tmpout(tmp: pathlib.Path, name: str) -> pathlib.Path:
    out = tmp / name
    out.mkdir(parents=True, exist_ok=True)
    return out


def control_default_emits_no_sidecar_and_frozen_world(tmp: pathlib.Path) -> None:
    out = tmpout(tmp, "default")
    run_lower(FIXTURES / "diag-fault" / "main.at", out, diagnostics_map=False)
    world = (out / "world.at").read_bytes()
    assert sha256(world) == WORLD_SHA, "default world.at drifted from frozen sha"
    assert not (out / "source-map.json").exists(), "default CLI must not emit a sidecar"


def control_map_sidecar_binds_and_tiles(tmp: pathlib.Path) -> None:
    out = tmpout(tmp, "mapped")
    run_lower(FIXTURES / "diag-fault" / "main.at", out, diagnostics_map=True)
    world = (out / "world.at").read_bytes()
    assert sha256(world) == WORLD_SHA, "--diagnostics-map must not change world bytes"
    m = json.loads((out / "source-map.json").read_text(encoding="utf-8"))
    assert m["format"] == "node-source-map/v1", m["format"]
    assert m["world_sha256"] == WORLD_SHA
    assert m["user_source_sha256"] == sha256(world)
    assert m["user_source_bytes"] == len(world)
    assert set(m["modules"].keys()) == CLOSURE, m["modules"].keys()
    # 闭包身份与 closure/ 目录逐字节一致。
    for key, entry in m["modules"].items():
        body = (out / "closure" / f"{key}.at").read_bytes()
        assert entry["sha256"] == sha256(body), f"closure sha drift: {key}"
        assert entry["bytes"] == len(body), f"closure bytes drift: {key}"
    # 段铺瓦：有序、连续、半开、全覆盖。
    cursor = 0
    for seg in m["segments"]:
        assert seg["start"] == cursor, f"segments must tile the world at {cursor}"
        assert seg["end"] > seg["start"]
        assert seg["end"] <= len(world)
        cursor = seg["end"]
    assert cursor == len(world), "segments must cover the whole world"
    # 精度域：generated 段不得携带 module/source；mapped 段必须有源区间。
    for seg in m["segments"]:
        p = seg["precision"]
        if p in ("exact", "range", "context"):
            assert seg["module"] in m["modules"], seg
            if p != "context":
                assert seg["source_start"] is not None and seg["source_end"] is not None
        else:
            assert seg["module"] is None and seg["source_start"] is None


def control_regen_byte_identical(tmp: pathlib.Path) -> None:
    a = tmpout(tmp, "regen-a")
    b = tmpout(tmp, "regen-b")
    run_lower(FIXTURES / "diag-fault" / "main.at", a, diagnostics_map=True)
    run_lower(FIXTURES / "diag-fault" / "main.at", b, diagnostics_map=True)
    for f in ["world.at", "scene.agscene", "source-map.json"]:
        assert (a / f).read_bytes() == (b / f).read_bytes(), f"regen drift: {f}"
    # identity.json 的 provenance.product 随 out 目录名变化，归一后比较。
    ia = json.loads((a / "identity.json").read_text(encoding="utf-8"))
    ib = json.loads((b / "identity.json").read_text(encoding="utf-8"))
    ia.get("provenance", {}).pop("product", None)
    ib.get("provenance", {}).pop("product", None)
    assert ia == ib, "regen drift: identity.json (product-normalized)"


def control_checked_in_product_current(tmp: pathlib.Path) -> None:
    out = tmpout(tmp, "product")
    run_lower(FIXTURES / "diag-fault" / "main.at", out, diagnostics_map=True)
    for f in ["world.at", "scene.agscene", "source-map.json"]:
        assert (GENERATED / f).read_bytes() == (out / f).read_bytes(), (
            f"checked-in diag-fault product stale: {f}; regenerate with node2d-lower"
        )
    gin = json.loads((GENERATED / "identity.json").read_text(encoding="utf-8"))
    oin = json.loads((out / "identity.json").read_text(encoding="utf-8"))
    gin.get("provenance", {}).pop("product", None)
    oin.get("provenance", {}).pop("product", None)
    assert gin == oin, "checked-in identity.json stale (product-normalized)"
    for name in CLOSURE:
        assert (GENERATED / "closure" / f"{name}.at").read_bytes() == (
            out / "closure" / f"{name}.at"
        ).read_bytes(), f"checked-in closure stale: {name}"


def control_driver_gates() -> None:
    r = subprocess.run(
        [
            "cargo", "test", "--locked", "--offline",
            "--test", "node_source_diagnostics", "--", "--test-threads=1",
        ],
        cwd=DRIVER_DIR, capture_output=True, text=True, timeout=3600,
    )
    if r.returncode != 0:
        raise AssertionError(f"driver gates failed:\n{r.stdout[-2000:]}\n{r.stderr[-1000:]}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if not args.controls:
        parser.error("only --controls is supported")
    import tempfile

    checks = [
        control_default_emits_no_sidecar_and_frozen_world,
        control_map_sidecar_binds_and_tiles,
        control_regen_byte_identical,
        control_checked_in_product_current,
    ]
    with tempfile.TemporaryDirectory(prefix="verify-plan044-") as t:
        tmp = pathlib.Path(t)
        for check in checks:
            check(tmp)
            print(f"ok: {check.__name__}")
    control_driver_gates()
    print("ok: control_driver_gates")
    print("verify_plan044: all controls passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
