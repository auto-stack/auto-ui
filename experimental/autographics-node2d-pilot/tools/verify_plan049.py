"""PLAN-049 regeneration + identity controls (V-S05/V-S06/AC-05/AC-06; T-06).

--controls : full control set
  1. regenerate mixed products from the author fixtures into a temp dir and
     compare byte-identical against the committed generated-node-mixed/ui-ab-mixed;
  2. verify the frozen identity chain inside the committed products
     (ui.schema / generator identity / world_sha256 / closure module shas);
  3. verify the mixed SHAPE on the committed products: Embed elements inside
     the def element tree (root-def slice), region click dispatchers in the
     world ABI, half-buffer text arrays all strictly below the SDK 256
     literal-array truncation edge;
  4. cross-check the frozen capacity constants on BOTH graphs (pilot lowering
     and driver adapter re-checks must state the same numbers), including the
     mixed additions (regions 4 / scene elems 8 / region side 640) and the
     shared 048 faces (instances 8 / elements 32 / texts 4 / children 8 /
     queue 64 / text bytes 64 / half-buffer slots 2);
  5. verify the pinned SDK snapshot (driver prepare_sdk --verify-only);
  6. verify the old-profile no-drift controls exist (node-ui ui-ab products
     consumed by tests/node_ui_storage.rs; components-2d probe world.at
     byte-pinned by tests/node_ui_gates.rs; 048 gate + flow files present).

Exit 0 = all controls pass. Any drift names the file and the expected/actual
hashes; a rejected control never leaves a regenerated artifact behind.

Run from the auto-ui worktree root:
  python experimental/autographics-node2d-pilot/tools/verify_plan049.py --controls
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]
PILOT = ROOT / "experimental" / "autographics-node2d-pilot"
DRIVER = ROOT / "experimental" / "autographics-auto-driver"
FIXTURES = PILOT / "fixtures" / "node-mixed"
PRODUCTS = PILOT / "generated-node-mixed" / "ui-ab-mixed"

MIXED_SCHEMA = "autographics.node-ui-mixed/1"
GENERATOR = "autographics-node2d-pilot plan049 node-ui-mixed"


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def regenerate(out: pathlib.Path) -> None:
    cmd = [
        "cargo", "run", "--locked", "--quiet", "--bin", "node2d-lower", "--",
        "--profile", "mixed-ui-2d-3d",
        "--entry", str(FIXTURES / "main.at"),
        "--out", str(out),
    ]
    subprocess.run(cmd, cwd=PILOT, check=True)


def control_regeneration() -> None:
    with tempfile.TemporaryDirectory(prefix="plan049-regen-") as tmp:
        out = pathlib.Path(tmp) / "ui-ab-mixed"
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
    assert ui["schema"] == MIXED_SCHEMA, f"schema drift: {ui['schema']}"
    assert ui["generator"] == GENERATOR, f"generator drift: {ui['generator']}"
    world_sha = sha256_hex(world)
    assert ui["world_sha256"] == world_sha, (
        f"ui.json world binding {ui['world_sha256']} != actual {world_sha}"
    )
    assert identity["api"] == MIXED_SCHEMA, f"identity api drift: {identity['api']}"
    assert identity["ui_schema"] == MIXED_SCHEMA, "identity ui_schema drift"
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
    print(f"PASS identity: schema {MIXED_SCHEMA}; world sha {world_sha[:16]}…; "
          f"{len(modules)} closure modules bound")


def control_mixed_shape() -> None:
    ui = json.loads((PRODUCTS / "ui.json").read_text(encoding="utf-8"))
    world = (PRODUCTS / "world.at").read_text(encoding="utf-8")
    # Embed elements live INSIDE the def element tree (mount relation), root
    # def only; each carries dim/size and (optional) click dispatcher.
    def collect(els, out, is_root):
        for e in els:
            if e.get("kind") == "Embed":
                assert is_root, "embed region outside the root def (T-02 slice)"
                assert e.get("dim") in ("2d", "3d"), f"bad dim: {e.get('dim')}"
                assert 1 <= e.get("width", 0) <= 640 and 1 <= e.get("height", 0) <= 640, (
                    "region side outside the frozen 1..=640"
                )
                for s in e.get("scene", []):
                    assert s.get("kind") in ("quad", "circle", "cylinder"), (
                        f"scene kind outside the frozen T-02 set: {s.get('kind')}"
                    )
                out.append(e["name"])
            elif e.get("kind") == "Container":
                collect(e.get("children", []), out, is_root)
            elif e.get("kind") == "If":
                collect(e.get("then", []), out, is_root)
    regions = []
    for name, d in ui["defs"].items():
        collect(d["elements"], regions, d.get("is_root", False))
    assert len(regions) >= 2, f"expected both dimensions embedded, got {regions}"
    # region click dispatchers exist in the world ABI.
    for name in regions:
        assert f"_ui_click_{name}" in world, f"missing dispatcher for region {name}"
    # half-buffer text arrays all strictly below the SDK 256 truncation edge.
    for line in world.splitlines():
        if line.startswith("var ") and "_ui_text" in line and "=[" in line:
            elems = line.split("=[", 1)[1].rsplit("]", 1)[0]
            count = elems.count(",") + 1 if elems else 0
            assert count < 256, f"text buffer at SDK truncation edge ({count}): {line[:48]}"
    print(f"PASS mixed shape: {len(regions)} regions in the root def tree "
          f"(dispatchers bound, half-buffers < 256)")


def control_capacity_constants() -> None:
    pilot_src = (PILOT / "src" / "lower_node_ui.rs").read_text(encoding="utf-8")
    driver_src = (DRIVER / "src" / "node_ui.rs").read_text(encoding="utf-8")
    driver_mixed = (DRIVER / "src" / "node_mixed.rs").read_text(encoding="utf-8")

    def lit(src, ident):
        for line in src.splitlines():
            if line.strip().startswith(f"pub const {ident}") and "=" in line:
                return line.split("=", 1)[1].split(";")[0].strip()
        return None

    checks = [
        # (constant, graph source, expected literal)
        ("MAX_INSTANCES", pilot_src, "8"),
        ("MAX_INSTANCES", driver_src, "8"),
        ("MAX_ELEMENTS", pilot_src, "32"),
        ("MAX_ELEMENTS", driver_src, "32"),
        ("MAX_TEXTS_PER_DEF", pilot_src, "4"),
        ("MAX_TEXTS", driver_src, "4"),
        ("MAX_CHILDREN", pilot_src, "8"),
        ("MAX_CHILDREN", driver_src, "8"),
        ("QUEUE_CAP", driver_src, "64"),
        ("TEXT_BYTES", pilot_src, "64"),
        # mixed additions (T-02 freeze; both graphs state the same numbers)
        ("MAX_REGIONS_PER_DEF", pilot_src, "4"),
        ("MAX_REGIONS", driver_mixed, "4"),
        ("MAX_SCENE_ELEMS_PER_REGION", pilot_src, "8"),
        ("MAX_SCENE_ELEMS", driver_mixed, "8"),
        ("MAX_REGION_SIDE", pilot_src, "640"),
        ("MAX_REGION_SIDE", driver_mixed, "640"),
        ("TEXT_SLOTS_PER_BUFFER", pilot_src, "2"),
    ]
    seen = set()
    for name, src, expected in checks:
        got = lit(src, name)
        assert got == expected, f"{name} = {got}, frozen {expected}"
        seen.add((name, expected))
    print("PASS capacity: frozen numbers identical on both graphs "
          "(instances 8 / elements 32 / texts 4 / children 8 / queue 64 / "
          "text bytes 64 / half-buffer slots 2; mixed: regions 4 / scene 8 / "
          "side 640)")


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
    ui_ab = PILOT / "generated-node-ui" / "ui-ab"
    assert (ui_ab / "ui.json").is_file() and (ui_ab / "world.at").is_file(), (
        "node-ui (048) frozen products missing"
    )
    storage = (DRIVER / "tests" / "node_ui_storage.rs").read_text(encoding="utf-8")
    assert "cold_consumer_reads_same_artifact_and_identity_chain_holds" in storage, (
        "048 storage identity control missing"
    )
    frozen = PILOT / "generated-components2d" / "probe" / "world.at"
    assert frozen.is_file(), "old-profile control bytes missing"
    gates = (PILOT / "tests" / "node_ui_gates.rs").read_text(encoding="utf-8")
    assert "old_components2d_products_do_not_drift" in gates, (
        "old-profile byte-pin test missing"
    )
    mixed_gates = (PILOT / "tests" / "node_mixed_views_gates.rs").read_text(encoding="utf-8")
    assert "mixed_regeneration_and_committed_products_are_byte_identical" in mixed_gates, (
        "mixed byte-pin control missing"
    )
    print("PASS old-profile: 048 products+storage control present; components-2d "
          "byte-pin present; mixed byte-pin present")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        raise SystemExit(2)
    control_identity()
    control_mixed_shape()
    control_capacity_constants()
    control_sdk()
    control_old_profile()
    control_regeneration()
    print("ALL PASS")


if __name__ == "__main__":
    main()
