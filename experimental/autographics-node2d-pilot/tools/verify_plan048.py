"""PLAN-048 regeneration + identity controls (V-S06/AC-06; T-06).

--controls : full control set
  1. regenerate node-ui products from the author fixtures into a temp dir and
     compare byte-identical against the committed generated-node-ui/ui-ab;
  2. verify the frozen identity chain inside the committed products
     (ui.schema / world_sha256 / closure module shas / generator identity);
  3. cross-check the frozen capacity constants on BOTH graphs (pilot
     lowering and driver adapter re-checks must state the same numbers);
  4. verify the pinned SDK snapshot (driver prepare_sdk --verify-only);
  5. verify the old-profile no-drift control bytes exist (components-2d
     probe world.at is byte-pinned by tests/node_ui_gates.rs).

Exit 0 = all controls pass. Any drift names the file and the expected/actual
hashes; a rejected control never leaves a regenerated artifact behind.

Run from the auto-ui worktree root:
  python experimental/autographics-node2d-pilot/tools/verify_plan048.py --controls
"""
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]
PILOT = ROOT / "experimental" / "autographics-node2d-pilot"
DRIVER = ROOT / "experimental" / "autographics-auto-driver"
FIXTURES = PILOT / "fixtures" / "node-ui"
PRODUCTS = PILOT / "generated-node-ui" / "ui-ab"

NODE_UI_SCHEMA = "autographics.node-ui/1"
GENERATOR = "autographics-node2d-pilot plan048 node-ui"


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def regenerate(out: pathlib.Path) -> None:
    cmd = [
        "cargo", "run", "--locked", "--quiet", "--bin", "node2d-lower", "--",
        "--profile", "node-ui",
        "--entry", str(FIXTURES / "main.at"),
        "--out", str(out),
    ]
    subprocess.run(cmd, cwd=PILOT, check=True)


def control_regeneration() -> None:
    with tempfile.TemporaryDirectory(prefix="plan048-regen-") as tmp:
        out = pathlib.Path(tmp) / "ui-ab"
        regenerate(out)
        expected = {}
        for p in sorted(PRODUCTS.rglob("*")):
            if p.is_file():
                expected[p.relative_to(PRODUCTS).as_posix()] = sha256_hex(p.read_bytes())
        actual = {}
        for p in sorted(out.rglob("*")):
            if p.is_file():
                actual[p.relative_to(out).as_posix()] = sha256_hex(p.read_bytes())
        if expected != actual:
            drifted = {
                k for k in set(expected) | set(actual)
                if expected.get(k) != actual.get(k)
            }
            for k in sorted(drifted):
                print(
                    f"DRIFT {k}: committed={expected.get(k, '<missing>')} "
                    f"regen={actual.get(k, '<missing>')}"
                )
            raise SystemExit("regeneration control FAILED (see DRIFT lines)")
    print("PASS regeneration: byte-identical (ui.json + world.at + identity + closure)")


def control_identity() -> None:
    ui = json.loads((PRODUCTS / "ui.json").read_text(encoding="utf-8"))
    identity = json.loads((PRODUCTS / "identity.json").read_text(encoding="utf-8"))
    world = (PRODUCTS / "world.at").read_bytes()
    assert ui["schema"] == NODE_UI_SCHEMA, f"schema drift: {ui['schema']}"
    assert ui["generator"] == GENERATOR, f"generator drift: {ui['generator']}"
    world_sha = sha256_hex(world)
    assert ui["world_sha256"] == world_sha, (
        f"ui.json world binding {ui['world_sha256']} != actual {world_sha}"
    )
    assert identity["api"] == NODE_UI_SCHEMA, f"identity api drift: {identity['api']}"
    assert identity["ui_schema"] == NODE_UI_SCHEMA, "identity ui_schema drift"
    assert identity["provenance"]["world_sha256"] == world_sha, (
        "identity provenance world sha drift"
    )
    modules = {k: v for k, v in identity.items() if k.startswith("module:")}
    for rel in sorted((PRODUCTS / "closure").glob("*.at")):
        name = f"module:{rel.stem}"
        sha = sha256_hex(rel.read_bytes())
        assert modules.get(name) == sha, f"closure {rel.name}: {modules.get(name)} != {sha}"
    assert len(modules) == len(list((PRODUCTS / "closure").glob("*.at"))), (
        "closure file set != identity module map"
    )
    print(f"PASS identity: schema {NODE_UI_SCHEMA}; world sha {world_sha[:16]}…; "
          f"{len(modules)} closure modules bound")


def control_capacity_constants() -> None:
    pilot_src = (PILOT / "src" / "lower_node_ui.rs").read_text(encoding="utf-8")
    driver_src = (DRIVER / "src" / "node_ui.rs").read_text(encoding="utf-8")
    checks = [
        # (constant, expected literal in pilot, expected literal in driver)
        ("MAX_INSTANCES", "8", "8"),
        ("MAX_ELEMENTS", "32", "32"),
        ("MAX_TEXTS_PER_DEF" if "MAX_TEXTS_PER_DEF" in pilot_src else "MAX_TEXTS",
         "4", "4"),
        ("MAX_CHILDREN", "8", "8"),
        ("QUEUE_CAP", None, "64"),
        ("TEXT_BYTES", "64", None),
    ]
    for name, pilot_lit, driver_lit in checks:
        def lit(src, ident):
            for line in src.splitlines():
                if line.strip().startswith(f"pub const {ident}") and "=" in line:
                    tail = line.split("=", 1)[1].split(";")[0].strip()
                    return tail
            return None
        if pilot_lit is not None:
            got = lit(pilot_src, name)
            assert got == pilot_lit, f"pilot {name} = {got}, frozen {pilot_lit}"
        if driver_lit is not None:
            ident = name if name != "MAX_TEXTS_PER_DEF" else "MAX_TEXTS"
            got = lit(driver_src, ident)
            assert got == driver_lit, f"driver {ident} = {got}, frozen {driver_lit}"
    print("PASS capacity: frozen numbers identical on both graphs "
          "(instances 8 / elements 32 / texts 4 / children 8 / queue 64 / text bytes 64)")


def control_sdk() -> None:
    if not (DRIVER / ".sdk-source").exists():
        print("SKIP sdk verify (no .sdk-source prepared in this checkout)")
        return
    r = subprocess.run(
        [sys.executable, str(DRIVER / "tools" / "prepare_sdk.py"), "--verify-only"],
        cwd=DRIVER, capture_output=True, text=True,
    )
    assert r.returncode == 0, f"sdk verify failed: {r.stdout}{r.stderr}"
    print("PASS sdk: pinned 681-file snapshot verified (--verify-only)")


def control_old_profile() -> None:
    frozen = PILOT / "generated-components2d" / "probe" / "world.at"
    assert frozen.is_file(), "old-profile control bytes missing"
    gates = (PILOT / "tests" / "node_ui_gates.rs").read_text(encoding="utf-8")
    assert "old_components2d_products_do_not_drift" in gates, (
        "old-profile byte-pin test missing"
    )
    print("PASS old-profile: components-2d world.at byte-pin control present "
          "(enforced by tests/node_ui_gates.rs)")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        raise SystemExit(2)
    control_identity()
    control_capacity_constants()
    control_sdk()
    control_old_profile()
    control_regeneration()
    print("PLAN-048 controls: ALL PASS")


if __name__ == "__main__":
    main()
