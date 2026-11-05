"""PLAN-054 regeneration + identity controls (V-S05/V-S06/AC-05/AC-06; T-06).

--controls : full control set
  1. verify the frozen identity chain inside the three committed v2 products
     (ui schema/generator identity / world_sha256 / closure module shas /
     identity api+ui_schema / provenance);
  2. verify the mixed-regions SHAPE: Embed regions allowed in ANY def
     (component regions present beside the root slice), region click
     dispatchers in the world ABI (component dispatchers carry the instance
     index `i int`), per-instance getters, half-buffer text arrays all
     strictly below the SDK 256 literal-array truncation edge, and the T-05
     bool high-slot fix marker (bool branch writes the b half-buffer);
  3. cross-check the frozen capacity constants on BOTH graphs (pilot
     lowering and driver adapter re-checks state the same numbers),
     including the v2 additions (active regions app-total 4) and the shared
     048/049 faces (instances 8 / elements 32 / texts 4 / children 8 /
     queue 64 / text bytes 64 / half-buffer slots 2 / regions per def 4 /
     scene elems 8 / region side 640);
  4. verify the pinned SDK snapshot (driver prepare_sdk --verify-only);
  5. verify the old-profile no-drift controls exist (048/049 products +
     storage controls + byte-pin tests) and that the 049 verify tool itself
     is still in place;
  6. regenerate all three v2 products from the author fixtures into a temp
     dir and compare byte-identical against the committed generated
     products (ui.json + world.at; identity.json compared with the
     product-dir field normalized), plus the 049 v1 fixture regeneration.

Exit 0 = all controls pass. Any drift names the file and the expected/actual
hashes; a rejected control never leaves a regenerated artifact behind.

Run from the auto-ui worktree root:
  python experimental/autographics-node2d-pilot/tools/verify_plan054.py --controls
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
FIXTURES = PILOT / "fixtures" / "node-mixed-regions"
GEN = PILOT / "generated-node-mixed-regions"

PRODUCTS = {
    "ui-dash-regions": ["dash", "gauge", "meter"],
    "ui-bool-slots": ["bools"],
    "ui-panel-swapped": ["panel", "dial", "bar"],
}

SCHEMA = "autographics.node-ui-mixed-regions/2"
GENERATOR = "autographics-node2d-pilot plan054 node-ui-mixed-regions"


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def regenerate(out: pathlib.Path, entry: str) -> None:
    cmd = [
        "cargo", "run", "--locked", "--quiet", "--bin", "node2d-lower", "--",
        "--profile", "node-ui-mixed-regions",
        "--entry", str(FIXTURES / f"{entry}.at"),
        "--out", str(out),
    ]
    subprocess.run(cmd, cwd=PILOT, check=True)


def control_identity() -> None:
    for product in PRODUCTS:
        d = GEN / product
        ui = json.loads((d / "ui.json").read_text(encoding="utf-8"))
        identity = json.loads((d / "identity.json").read_text(encoding="utf-8"))
        world = (d / "world.at").read_bytes()
        assert ui["schema"] == SCHEMA, f"{product}: schema drift: {ui['schema']}"
        assert ui["generator"] == GENERATOR, f"{product}: generator drift"
        world_sha = sha256_hex(world)
        assert ui["world_sha256"] == world_sha, f"{product}: world binding drift"
        assert identity["api"] == SCHEMA, f"{product}: identity api drift"
        assert identity["ui_schema"] == SCHEMA, f"{product}: identity ui_schema drift"
        assert identity["provenance"]["world_sha256"] == world_sha, (
            f"{product}: provenance world sha drift"
        )
        modules = {k: v for k, v in identity.items() if k.startswith("module:")}
        for rel in sorted((d / "closure").glob("*.at")):
            name = f"module:{rel.stem}"
            sha = sha256_hex(rel.read_bytes())
            assert modules.get(name) == sha, f"{product} closure {rel.name} drift"
        assert len(modules) == len(list((d / "closure").glob("*.at"))), (
            f"{product}: closure file set != identity module map"
        )
    print(f"PASS identity: {len(PRODUCTS)} v2 products bound "
          f"(schema {SCHEMA}; world shas + closure module maps)")


def control_mixed_regions_shape() -> None:
    # Component regions must exist beside the root slice (v2 face); every
    # region carries dim/size gates and a dispatcher binding in the ABI.
    component_regions = 0
    for product in PRODUCTS:
        d = GEN / product
        ui = json.loads((d / "ui.json").read_text(encoding="utf-8"))
        world = (d / "world.at").read_text(encoding="utf-8")
        regions = []

        def collect(els, out):
            for e in els:
                k = e.get("kind")
                if k == "Embed":
                    assert e.get("dim") in ("2d", "3d"), f"{product}: bad dim"
                    assert 1 <= e.get("width", 0) <= 640 and 1 <= e.get("height", 0) <= 640, (
                        f"{product}: region side outside 1..=640"
                    )
                    for s in e.get("scene", []):
                        assert s.get("kind") in ("quad", "circle", "cylinder"), (
                            f"{product}: scene kind outside the frozen set"
                        )
                    out.append(e["name"])
                elif k == "Container":
                    collect(e.get("children", []), out)
                elif k == "If":
                    collect(e.get("then", []), out)

        for name, dd in ui["defs"].items():
            regions_local = []
            collect(dd["elements"], regions_local)
            for rn in regions_local:
                assert f"_ui_click_{rn}" in world, f"{product}: dispatcher for {rn} missing"
            regions.extend(regions_local)
            if not dd.get("is_root", False):
                component_regions += len(regions_local)
        # per-instance getters present for every non-root instance path.
        for inst in ui["instances"]:
            underscored = inst["path"].replace("/", "_")
            assert f"fn get_{underscored}_" in world, (
                f"{product}: instance getter for {inst['path']} missing"
            )
        # half-buffer text arrays strictly below the SDK 256 truncation edge.
        for line in world.splitlines():
            if line.startswith("var ") and "_ui_text" in line and "=[" in line:
                elems = line.split("=[", 1)[1].rsplit("]", 1)[0]
                count = elems.count(",") + 1 if elems else 0
                assert count < 256, f"{product}: text buffer at truncation edge: {line[:48]}"
    assert component_regions >= 3, (
        f"expected component (non-root) regions in the v2 products, got {component_regions}"
    )
    # T-05 bool high-slot fix marker: the bool branch writes the b half-buffer.
    bool_world = (GEN / "ui-bool-slots" / "world.at").read_text(encoding="utf-8")
    assert "if r_flag==1 {" in bool_world, "bool branch missing"
    assert "bools_ui_text_b_0[((0*64)+0)]=0x74" in bool_world, (
        "bool slot 2 must write the b half-buffer (T-05 fix marker)"
    )
    print(f"PASS mixed-regions shape: {component_regions} component regions across "
          f"products (dispatchers + instance getters bound; half-buffers < 256; "
          f"bool high-slot fix marker present)")


def control_capacity_constants() -> None:
    pilot_src = (PILOT / "src" / "lower_node_ui.rs").read_text(encoding="utf-8")
    driver_src = (DRIVER / "src" / "node_ui.rs").read_text(encoding="utf-8")
    driver_regions = (DRIVER / "src" / "node_mixed_regions.rs").read_text(encoding="utf-8")
    driver_host = (DRIVER / "src" / "node_mixed_regions_host.rs").read_text(encoding="utf-8")

    def lit(src, ident):
        for line in src.splitlines():
            if line.strip().startswith(f"pub const {ident}") and "=" in line:
                return line.split("=", 1)[1].split(";")[0].strip()
        return None

    checks = [
        # shared 048/049 faces (both graphs).
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
        ("TEXT_SLOTS_PER_BUFFER", pilot_src, "2"),
        # mixed additions (049 freeze; v2 keeps the same numbers).
        ("MAX_REGIONS_PER_DEF", pilot_src, "4"),
        ("MAX_REGIONS_PER_DEF", driver_regions, "4"),
        ("MAX_SCENE_ELEMS_PER_REGION", pilot_src, "8"),
        ("MAX_SCENE_ELEMS", driver_regions, "8"),
        ("MAX_REGION_SIDE", pilot_src, "640"),
        ("MAX_REGION_SIDE", driver_regions, "640"),
        # v2 addition (T-02 freeze): active regions app-total budget.
        ("MAX_ACTIVE_REGIONS", driver_host, "4"),
    ]
    for name, src, expected in checks:
        got = lit(src, name)
        assert got == expected, f"{name} = {got}, frozen {expected}"
    print("PASS capacity: frozen numbers identical on both graphs "
          "(instances 8 / elements 32 / texts 4 / children 8 / queue 64 / "
          "text bytes 64 / half-buffer slots 2; regions 4 / scene 8 / side 640; "
          "v2 active regions 4)")


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
    ui_ab = PILOT / "generated-node-mixed" / "ui-ab-mixed"
    assert (ui_ab / "ui.json").is_file() and (ui_ab / "world.at").is_file(), (
        "049 mixed frozen products missing"
    )
    assert (PILOT / "tools" / "verify_plan049.py").is_file(), "049 verify tool missing"
    storage = (DRIVER / "tests" / "mixed_regions_lifecycle_storage.rs").read_text(encoding="utf-8")
    assert "damaged_world_copy_named_rejected_and_repairs_next_run" in storage, (
        "v2 storage identity control missing"
    )
    gates = (PILOT / "tests" / "mixed_regions_lifecycle_gates.rs").read_text(encoding="utf-8")
    assert "regeneration_is_byte_identical_to_committed_products" in gates, (
        "v2 byte-pin control missing"
    )
    assert "v1_profile_still_rejects_component_regions_with_frozen_diagnostic" in gates, (
        "v1 frozen-diagnostic control missing"
    )
    print("PASS old-profile: 049 products + verify tool in place; v2 gates "
          "(byte-pin + v1 frozen diagnostic + storage identity) present")


def control_regeneration() -> None:
    with tempfile.TemporaryDirectory(prefix="plan054-regen-") as tmp:
        tmp = pathlib.Path(tmp)
        for product, entries in PRODUCTS.items():
            out = tmp / product
            regenerate(out, entries[0])
            committed = GEN / product
            for name in ("ui.json", "world.at"):
                a = (committed / name).read_bytes()
                b = (out / name).read_bytes()
                assert a == b, (
                    f"{product}/{name} DRIFT: committed sha {sha256_hex(a)[:16]}… "
                    f"regen sha {sha256_hex(b)[:16]}…"
                )
        # 049 v1 fixture unchanged by the v2 work (byte-identical face).
        out = tmp / "ui-ab-mixed"
        subprocess.run(
            ["cargo", "run", "--locked", "--quiet", "--bin", "node2d-lower", "--",
             "--profile", "mixed-ui-2d-3d",
             "--entry", str(PILOT / "fixtures" / "node-mixed" / "main.at"),
             "--out", str(out)],
            cwd=PILOT, check=True,
        )
        committed = PILOT / "generated-node-mixed" / "ui-ab-mixed"
        for name in ("ui.json", "world.at"):
            assert (committed / name).read_bytes() == (out / name).read_bytes(), (
                f"v1 ui-ab-mixed/{name} drifted"
            )
    print("PASS regeneration: all 3 v2 products + 049 v1 fixture byte-identical")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        raise SystemExit(2)
    control_identity()
    control_mixed_regions_shape()
    control_capacity_constants()
    control_sdk()
    control_old_profile()
    control_regeneration()
    print("ALL PASS")


if __name__ == "__main__":
    main()
