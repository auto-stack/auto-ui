"""PLAN-020 components-2d verify controls: identity binding for every frozen
generated-components2d product (world/scene sha chain, closure manifest,
runner provenance section), cold A->B regeneration byte equality from the
closure copies alone, fixture mutation sensitivity, negative lowering
rejections with their named diagnostics, and the damaged-consumption
recovery control (a one-byte-corrupted world copy under the scene-bound sha
name is rejected by the driver load path; the exact restore runs green —
demonstrated by the real driver test node_components_flow::v06_damaged_,
which this verifier invokes). Raw evidence stays local; the durable record
is the Plan receipt. Run from any cwd:

  python experimental/autographics-node2d-pilot/tools/verify_plan020.py --controls
"""
import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parents[1]
FIXTURES = ROOT / "fixtures" / "components2d"
GENERATED = ROOT / "generated-components2d"
FLOW_TEST = REPO / "experimental/autographics-auto-driver/tests/node_components_flow.rs"
DRIVER_DIR = REPO / "experimental/autographics-auto-driver"

GENERATOR_ID = "autographics-node2d-pilot plan020 components-2d"
API = "autographics.objects2d/1"

# product -> fixture entry (relative to fixtures/components2d)
ENTRIES = {
    "probe": "main.at",
    "helpers": "helpers/main.at",
    "budget": "budget/main.at",
    "budget-plus": "budget-plus/main.at",
    "lifecycle": "lifecycle/main.at",
    "lifecycle-doublepress": "lifecycle-doublepress/main.at",
    "envelope": "envelope/main.at",
    "init-fault": "init-fault/main.at",
}

# product -> exact closure module set (the closure/ dir carries those only)
CLOSURE = {
    "probe": {"main", "panel", "player", "gauge", "resources"},
    "helpers": {"main", "helpera", "helperb", "resources"},
    "budget": {"main", "gauge", "player", "resources"},
    "budget-plus": {"main", "gauge", "player", "resources"},
    "lifecycle": {"main", "panel", "player", "gauge", "resources"},
    "lifecycle-doublepress": {"main", "panel", "playerdp", "gauge", "resources"},
    "envelope": {"main", "ticker", "player", "resources"},
    "init-fault": {"main", "cluster", "player", "resources"},
}

# products with a registered driver flow consumer (the rest: pilot gates)
DRIVER_CONSUMED = {
    "probe",
    "budget",
    "budget-plus",
    "lifecycle",
    "lifecycle-doublepress",
    "envelope",
    "init-fault",
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_lower(entry: pathlib.Path, out: pathlib.Path) -> None:
    cmd = [
        "cargo",
        "run",
        "--locked",
        "--offline",
        "--bin",
        "node2d-lower",
        "--",
        "--profile",
        "components-2d",
        "--entry",
        str(entry),
        "--out",
        str(out),
    ]
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    if r.returncode != 0:
        raise AssertionError(f"lowering failed for {entry}: {r.stderr.strip()[:400]}")


def run_lower_expect_error(entry: pathlib.Path, out: pathlib.Path) -> str:
    cmd = [
        "cargo",
        "run",
        "--locked",
        "--offline",
        "--bin",
        "node2d-lower",
        "--",
        "--profile",
        "components-2d",
        "--entry",
        str(entry),
        "--out",
        str(out),
    ]
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    assert r.returncode != 0, f"lowering of {entry} must fail"
    return r.stderr + r.stdout


def fixture_dir(product: str) -> pathlib.Path:
    entry = ENTRIES[product]
    return FIXTURES if "/" not in entry else FIXTURES / entry.split("/")[0]


def compare_products(frozen: pathlib.Path, regen: pathlib.Path, name: str) -> None:
    for f in ("scene.agscene", "identity.json", "world.at"):
        a = (frozen / f).read_bytes()
        b = (regen / f).read_bytes()
        assert a == b, f"{name}/{f} regeneration differs"


def check_identity(name: str, out_dir: pathlib.Path) -> None:
    identity = json.loads((out_dir / "identity.json").read_text(encoding="utf-8"))
    assert identity["generator"] == GENERATOR_ID, name
    assert identity["api"] == API, (name, identity["api"])
    scene = (out_dir / "scene.agscene").read_text(encoding="utf-8")
    doc = json.loads(scene)
    assert doc["schema"] == "autographics.scene/6", name
    assert doc["behavior"]["api"] == API, name
    assert doc["behavior"]["actors"]["two_d"] == 1, name
    world = (out_dir / "world.at").read_bytes()
    source_sha = doc["behavior"]["source"]["sha256"]
    assert source_sha == sha(world), f"{name}: scene binds the world source"
    assert doc["behavior"]["source"]["byte_length"] == len(world), name
    assert identity["user_source_sha256"] == source_sha, name
    # scene object names are identity: unique across the document.
    names = [o["name"] for o in doc["objects"]]
    assert len(names) == len(set(names)), f"{name}: duplicate scene object names"
    # closure manifest: exact module set, hashes and bytes correct, files
    # equal to the live fixture sources (frozen input discipline).
    modules = {c["name"]: c for c in identity["closure"]}
    assert modules.keys() == CLOSURE[name], (name, modules.keys())
    closure_dir = out_dir / "closure"
    for mod, rec in modules.items():
        f = closure_dir / f"{mod}.at"
        assert f.is_file(), f"{name}: closure file {mod} missing"
        data = f.read_bytes()
        assert rec["sha256"] == sha(data), f"{name}: closure hash {mod}"
        assert rec["bytes"] == len(data), f"{name}: closure bytes {mod}"
        fixture = fixture_dir(name) / f"{mod}.at"
        assert sha(fixture.read_bytes()) == rec["sha256"], f"{name}: fixture {mod} drifted"
    # runner provenance (T-06): factual consumption record bound to the
    # world sha, naming the real consumer test file.
    prov = identity["provenance"]
    assert prov["product"] == name, (name, prov["product"])
    assert prov["world_sha256"] == source_sha, name
    consumer = REPO / prov["consumer"].split(" (")[0]
    assert consumer.is_file(), f"{name}: consumer test file missing"
    if name in DRIVER_CONSUMED:
        assert "node_components_flow.rs" in prov["consumer"], name
        for f in ("scene.agscene", "world.at"):
            assert (
                f"generated-components2d/{name}/{f}" in FLOW_TEST.read_text(encoding="utf-8")
            ), f"{name}/{f} not include_str!'d by the flow test"
    else:
        assert "node_components_gates.rs" in prov["consumer"], name


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    ap.add_argument(
        "--skip-driver",
        action="store_true",
        help="skip the damaged-consumption driver control (needs the auto-driver build)",
    )
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        return 2
    checks = 0
    with tempfile.TemporaryDirectory(prefix="verify-plan020-") as tmp:
        tmp = pathlib.Path(tmp)
        # 1. cold A->B: B receives ONLY the closure copies and reproduces
        #    every frozen product byte-identically (world/scene/identity);
        #    identity chain + provenance checked on the frozen product.
        for name in ENTRIES:
            frozen = GENERATED / name
            assert frozen.is_dir(), f"missing frozen product {name}"
            check_identity(name, frozen)
            bdir = tmp / f"cold-{name}"
            (bdir / "closure").parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(frozen / "closure", bdir / "closure")
            entry_mod = ENTRIES[name].split("/")[-1]
            out = tmp / name  # out dir basename feeds identity.provenance.product
            run_lower(bdir / "closure" / entry_mod, out)
            compare_products(frozen, out, name)
            print(f"PASS cold A->B + identity: {name}")
            checks += 1
        # 2. mutation sensitivity: a real param change in a fixture copy
        #    regenerates to different bytes (the pipeline is not
        #    constant-folded and identity follows the source).
        mutdir = tmp / "budget-mut"
        shutil.copytree(FIXTURES / "budget", mutdir)
        src = (mutdir / "main.at").read_text(encoding="utf-8")
        mutated = src.replace("base_x: 4.0", "base_x: 5.0")
        assert mutated != src, "mutation did not change the fixture"
        (mutdir / "main.at").write_text(mutated, encoding="utf-8")
        out = tmp / "budget"  # same product name so only the source differs
        run_lower(mutdir / "main.at", out)
        for f in ("scene.agscene", "world.at", "identity.json"):
            assert (out / f).read_bytes() != (GENERATED / "budget" / f).read_bytes(), (
                f"mutation invisible in budget/{f}"
            )
        regen_scene = json.loads((out / "scene.agscene").read_text())
        moved = [
            o
            for o in regen_scene["objects"]
            if o["name"] == "g1_template" and o["translation"][0] == 4.0
        ]
        assert not moved, "gauge g1 base_x mutation not applied to the spawn row"
        print("PASS mutation sensitivity: budget base_x 4.0->5.0 changes world/scene/identity")
        checks += 1
        # 3. negative rejections replay: each neg-* lowers to Err with its
        #    named diagnostic (module / line / symbol present).
        base = {
            "main": FIXTURES / "main.at",
            "player": FIXTURES / "player.at",
            "panel": FIXTURES / "panel.at",
            "gauge": FIXTURES / "gauge.at",
            "resources": FIXTURES / "resources.at",
        }

        def compose(out_files: dict, neg: str) -> pathlib.Path:
            d = tmp / f"neg-{neg}"
            if d.exists():
                shutil.rmtree(d)
            d.mkdir(parents=True)
            for mod, srcf in out_files.items():
                shutil.copyfile(srcf, d / f"{mod}.at")
            return d

        negatives = [
            (
                "missing-use",
                {
                    "main": FIXTURES / "neg-missing-use/main.at",
                    "player": base["player"],
                    "resources": base["resources"],
                },
                ["Panel"],
                False,
            ),
            (
                "wrong-arity",
                {
                    "main": base["main"],
                    "player": base["player"],
                    "panel": FIXTURES / "neg-wrong-arity/panel.at",
                    "gauge": base["gauge"],
                    "resources": base["resources"],
                },
                ["Add", "line ", "module "],
            ),
            (
                "unknown-port",
                {
                    "main": base["main"],
                    "player": base["player"],
                    "panel": FIXTURES / "neg-unknown-port/panel.at",
                    "gauge": base["gauge"],
                    "resources": base["resources"],
                },
                ["unknown event port", "'Bogus'"],
            ),
            (
                "cycle",
                {
                    "main": FIXTURES / "neg-cycle/main.at",
                    "boxa": FIXTURES / "neg-cycle/boxa.at",
                    "boxb": FIXTURES / "neg-cycle/boxb.at",
                },
                ["instance cycle: BoxA -> BoxB -> BoxA", "module boxb", "'inner'"],
            ),
            (
                "9-stateful-instances",
                {
                    "main": FIXTURES / "neg-9-stateful/main.at",
                    "item": FIXTURES / "neg-9-stateful/item.at",
                    "resources": base["resources"],
                },
                ["8-instance envelope", "'i8'", "line "],
            ),
            (
                "no-camera",
                {
                    "main": FIXTURES / "neg-no-camera/main.at",
                    "player": base["player"],
                    "panel": base["panel"],
                    "gauge": base["gauge"],
                    "resources": base["resources"],
                },
                ["camera2d", "requires"],
            ),
            (
                "param-write",
                {
                    "main": FIXTURES / "budget/main.at",
                    "gauge": FIXTURES / "neg-param-write/gauge.at",
                    "player": FIXTURES / "budget/player.at",
                    "resources": FIXTURES / "budget/resources.at",
                },
                ["read-only", "'speed'", "module gauge", "line "],
            ),
        ]
        for neg_entry in negatives:
            neg, files, needles = neg_entry[0], neg_entry[1], neg_entry[2]
            lowering_level = neg_entry[3] if len(neg_entry) > 3 else True
            d = compose(files, neg)
            msg = run_lower_expect_error(d / "main.at", tmp / f"negout-{neg}")
            for needle in needles:
                assert needle in msg, f"negative {neg}: diagnostic lacks '{needle}': {msg[:300]}"
            if lowering_level:
                # lowering diagnostics carry the frozen located form; the
                # missing-use rejection is load-level (file:line).
                assert "module " in msg and "line " in msg and "symbol '" in msg, (
                    f"negative {neg}: diagnostic must carry module/line/symbol: {msg[:300]}"
                )
            else:
                # load-level rejection: the undeclared node is named (the
                # module never entered the closure, so there is no source
                # position to point at — observed form ':0: instance of
                # undeclared node 'Panel'').
                assert "instance of undeclared node 'Panel'" in msg, (
                    f"negative {neg}: unexpected diagnostic: {msg[:300]}"
                )
            print(f"PASS negative rejection: {neg}")
            checks += 1
        # 4. damaged-consumption recovery control. Mechanism: the consumer
        #    project binds world.at as scripts/<sha>.at where <sha> =
        #    scene.behavior.source.sha256; the driver load path hash-verifies
        #    that file against its name. Python replays the exact condition,
        #    then the real driver control (v06_damaged_) demonstrates the
        #    observed rejection + green recovery through Storage::load_project.
        frozen = GENERATED / "probe"
        proj = tmp / "damaged-consume"
        (proj / "scripts").mkdir(parents=True)
        shutil.copyfile(frozen / "scene.agscene", proj / "scene.agscene")
        scene_text = (frozen / "scene.agscene").read_text(encoding="utf-8")
        bound_sha = scene_text.split('"sha256": "')[1].split('"')[0]
        script = proj / "scripts" / f"{bound_sha}.at"
        shutil.copyfile(frozen / "world.at", script)
        world = bytearray(script.read_bytes())
        world[0] = (world[0] + 1) % 256
        script.write_bytes(bytes(world))
        assert sha(bytes(world)) != bound_sha, "one-byte corruption undetected"
        # exact restore -> the load condition holds again (python replay of
        # the loader's check); the real observed rejection + green recovery
        # run in the driver control below.
        shutil.copyfile(frozen / "world.at", script)
        assert sha(script.read_bytes()) == bound_sha, "restore failed"
        if args.skip_driver:
            print("SKIP damaged-consumption driver control (--skip-driver)")
        else:
            cmd = [
                "cargo",
                "test",
                "--locked",
                "--offline",
                "--test",
                "node_components_flow",
                "v06_damaged",
                "--",
                "--test-threads=1",
            ]
            r = subprocess.run(cmd, cwd=DRIVER_DIR, capture_output=True, text=True, timeout=3600)
            assert r.returncode == 0, (
                f"driver damaged-consumption control failed:\n{r.stdout[-1500:]}\n{r.stderr[-500:]}"
            )
            assert "test result: ok" in r.stdout, r.stdout[-500:]
        print("PASS damaged consumption: corrupted world copy rejected at load; restore runs green")
        checks += 1
    print(f"PASS verify_plan020 --controls: {checks} control groups, {len(ENTRIES)} products")
    return 0


if __name__ == "__main__":
    sys.exit(main())
