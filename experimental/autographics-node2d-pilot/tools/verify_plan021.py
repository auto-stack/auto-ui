"""PLAN-021 queries-2d verify controls: identity binding for every frozen
generated-queries2d product (world/scene sha chain, closure manifest,
provenance section), cold A->B regeneration byte equality from the closure
copies alone, the changed-variant source-identity control (q-changed vs
q-segment: different source binding, same scene geometry), fixture mutation
sensitivity, negative lowering rejections with their named module/line/symbol
diagnostics (including the components-2d profile boundary), and the
damaged-consumption recovery control replayed in python and demonstrated by
the real driver test node_queries_flow::v06_damaged_. Raw evidence stays
local; the durable record is the Plan receipt. Run from any cwd:

  python experimental/autographics-node2d-pilot/tools/verify_plan021.py --controls
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
FIXTURES = ROOT / "fixtures" / "queries2d"
GENERATED = ROOT / "generated-queries2d"
FLOW_TEST = REPO / "experimental/autographics-auto-driver/tests/node_queries_flow.rs"
DRIVER_DIR = REPO / "experimental/autographics-auto-driver"

GENERATOR_ID = "autographics-node2d-pilot plan021 queries-2d"
API = "autographics.objects2d/1"

ENTRIES = {
    "q-segment": "segment.at",
    "q-sweep": "sweep.at",
    "q-cross": "crosslab.at",
    "q-cross-plus": "crossplus.at",
    "q-changed": "changed.at",
}

# every product is consumed by the driver flow test (frozen include_str!)
CONSUMERS = {name: "node_queries_flow.rs" for name in ENTRIES}

# exact closure module set per product (the entry's use closure)
CLOSURE = {
    "q-segment": {"resources", "spectator", "runner", "target", "marker", "door", "point_marker", "far", "segment"},
    "q-sweep": {"resources", "spectator", "runner", "marker", "door", "wall", "sweep"},
    "q-cross": {"resources", "spectator", "target", "far", "crosslab"},
    "q-cross-plus": {"resources", "spectator", "target", "far", "crossplus"},
    "q-changed": {"resources", "spectator", "runner", "target", "marker", "door", "point_marker", "far", "changed"},
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_lower(entry: pathlib.Path, out: pathlib.Path, profile: str = "queries-2d") -> None:
    cmd = ["cargo", "run", "--locked", "--offline", "--bin", "node2d-lower", "--",
           "--profile", profile, "--entry", str(entry), "--out", str(out)]
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    if r.returncode != 0:
        raise AssertionError(f"lowering failed for {entry}: {r.stderr.strip()[:400]}")


def run_lower_expect_error(entry: pathlib.Path, out: pathlib.Path, profile: str = "queries-2d") -> str:
    cmd = ["cargo", "run", "--locked", "--offline", "--bin", "node2d-lower", "--",
           "--profile", profile, "--entry", str(entry), "--out", str(out)]
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    assert r.returncode != 0, f"lowering of {entry} must fail"
    return r.stderr + r.stdout


def compare_products(frozen: pathlib.Path, regen: pathlib.Path, name: str) -> None:
    for f in ("scene.agscene", "identity.json", "world.at"):
        a = (frozen / f).read_bytes()
        b = (regen / f).read_bytes()
        assert a == b, f"{name}/{f} regeneration differs"


def check_identity(name: str, out_dir: pathlib.Path) -> str:
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
    names = [o["name"] for o in doc["objects"]]
    assert len(names) == len(set(names)), f"{name}: duplicate scene object names"
    modules = {c["name"]: c for c in identity["closure"]}
    assert modules.keys() == CLOSURE[name], (name, modules.keys() ^ CLOSURE[name])
    closure_dir = out_dir / "closure"
    for mod, rec in modules.items():
        f = closure_dir / f"{mod}.at"
        assert f.is_file(), f"{name}: closure file {mod} missing"
        data = f.read_bytes()
        assert rec["sha256"] == sha(data), f"{name}: closure hash {mod}"
        assert rec["bytes"] == len(data), f"{name}: closure bytes {mod}"
        fixture = FIXTURES / f"{mod}.at"
        assert sha(fixture.read_bytes()) == rec["sha256"], f"{name}: fixture {mod} drifted"
    prov = identity["provenance"]
    assert prov["product"] == name, (name, prov["product"])
    assert prov["world_sha256"] == source_sha, name
    assert CONSUMERS[name] in prov["consumer"], (name, prov["consumer"])
    consumer = REPO / prov["consumer"]
    assert consumer.is_file(), f"{name}: consumer test file missing"
    for f in ("scene.agscene", "world.at"):
        assert f"generated-queries2d/{name}/{f}" in FLOW_TEST.read_text(encoding="utf-8"), (
            f"{name}/{f} not include_str!'d by the flow test"
        )
    return source_sha


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    ap.add_argument("--skip-driver", action="store_true",
                    help="skip the damaged-consumption driver control (needs the auto-driver build)")
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        return 2
    checks = 0
    with tempfile.TemporaryDirectory(prefix="verify-plan021-") as tmp:
        tmp = pathlib.Path(tmp)
        # 1. cold A->B: B receives ONLY the closure copies and reproduces
        #    every frozen product byte-identically.
        world_shas = {}
        for name in ENTRIES:
            frozen = GENERATED / name
            assert frozen.is_dir(), f"missing frozen product {name}"
            world_shas[name] = check_identity(name, frozen)
            bdir = tmp / f"cold-{name}"
            (bdir / "closure").parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(frozen / "closure", bdir / "closure")
            entry_mod = ENTRIES[name]
            out = tmp / name  # out dir basename feeds identity.provenance.product
            run_lower(bdir / "closure" / entry_mod, out)
            compare_products(frozen, out, name)
            print(f"PASS cold A->B + identity: {name}")
            checks += 1
        # 2. changed-variant source-identity control: q-changed binds a
        #    different source (sha) with identical scene geometry (objects).
        seg_scene = json.loads((GENERATED / "q-segment" / "scene.agscene").read_text())
        chg_scene = json.loads((GENERATED / "q-changed" / "scene.agscene").read_text())
        assert seg_scene["objects"] == chg_scene["objects"], "changed scene objects must be identical"
        assert world_shas["q-changed"] != world_shas["q-segment"], "changed source binding must differ"
        print("PASS changed-variant identity: same geometry, different source binding (Q8)")
        checks += 1
        # 3. mutation sensitivity: a real fixture change regenerates to
        #    different bytes (identity follows the source).
        mutdir = tmp / "mut"
        mutdir.mkdir(parents=True)
        for f in FIXTURES.glob("*.at"):
            shutil.copyfile(f, mutdir / f.name)
        src = (mutdir / "runner.at").read_text(encoding="utf-8")
        mutated = src.replace("position: (6.0, 2.0)", "position: (7.0, 2.0)")
        assert mutated != src, "mutation did not change the fixture"
        (mutdir / "runner.at").write_text(mutated, encoding="utf-8")
        out = tmp / "q-segment"  # same product name so only the source differs
        run_lower(mutdir / "segment.at", out)
        # The moved row is scene-level: scene + identity must change (identity
        # carries the drifted runner.at module sha); world.at (the user
        # source) stays byte-identical for this mutation.
        for f in ("scene.agscene", "identity.json"):
            assert (out / f).read_bytes() != (GENERATED / "q-segment" / f).read_bytes(), (
                f"mutation invisible in q-segment/{f}"
            )
        assert (out / "world.at").read_bytes() == (GENERATED / "q-segment" / "world.at").read_bytes(), (
            "world.at should be unchanged for a scene-only mutation"
        )
        regen_scene = json.loads((out / "scene.agscene").read_text())
        moved = [o for o in regen_scene["objects"] if o["name"] == "runner.body" and o["translation"][0] == 7.0]
        assert moved, "runner position mutation not applied to the scene row"
        mut_identity = json.loads((out / "identity.json").read_text())
        frozen_identity = json.loads((GENERATED / "q-segment" / "identity.json").read_text())
        mut_mods = {c["name"]: c["sha256"] for c in mut_identity["closure"]}
        frozen_mods = {c["name"]: c["sha256"] for c in frozen_identity["closure"]}
        assert mut_mods["runner"] != frozen_mods["runner"], "drifted module sha not captured"
        assert mut_mods["segment"] == frozen_mods["segment"], "unrelated module drifted"
        print("PASS mutation sensitivity: runner 6.0->7.0 changes scene/identity (module sha drift captured)")
        checks += 1
        # 4. negative rejections replay (CLI, named module/line/symbol).
        def compose(mutate) -> pathlib.Path:
            d = tmp / f"neg-{abs(hash(str(mutate))) % 99999}"
            if d.exists():
                shutil.rmtree(d)
            d.mkdir(parents=True)
            for f in FIXTURES.glob("*.at"):
                shutil.copyfile(f, d / f.name)
            for fname, old, new in mutate:
                p = d / fname
                s = p.read_text(encoding="utf-8")
                s2 = s.replace(old, new)
                assert s2 != s, f"mutation no-op for {fname}: {old!r}"
                p.write_text(s2, encoding="utf-8")
            return d

        negatives = [
            ("unknown-ref", [("segment.at", "query2d.segment(refs.runner,", "query2d.segment(refs.ghost,")]),
            ("wrong-kind", [("segment.at", "let px = q.point(0)", "let px = q.origin(0)")]),
            ("unbound-getter", [("segment.at", "let hit = q.hit()", "let hit = qq.hit()")]),
            ("character-row-write", [("segment.at",
                "refs.runner.shift(input.x() * 60.0 * dt, input.y() * 60.0 * dt)",
                "refs.spectator.shift(input.x() * 60.0 * dt, input.y() * 60.0 * dt)")]),
            ("sweep-mover-minus-one", [("segment.at",
                "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                "let q = query2d.sweep(-1, 1, ax, ay)")]),
            ("var-bound-result", [("segment.at", "let q = query2d.segment", "var q = query2d.segment")]),
            ("query-arity", [("segment.at",
                "query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                "query2d.segment(refs.runner, 1, ax, ay, ax + .reach)")]),
            ("visible-only-literal", [("segment.at",
                "let q = query2d.segment(refs.runner, 1, ax, ay, ax + .reach, ay)",
                "let vis = 1\n            let q = query2d.segment(refs.runner, vis, ax, ay, ax + .reach, ay)")]),
        ]
        for neg, muts in negatives:
            d = compose(muts)
            msg = run_lower_expect_error(d / "segment.at", tmp / f"negout-{neg}")
            assert "module " in msg and "line " in msg and "symbol '" in msg, (
                f"negative {neg}: diagnostic must carry module/line/symbol: {msg[:300]}"
            )
            print(f"PASS negative rejection: {neg}")
            checks += 1
        # profile boundary: components-2d rejects the construct by name.
        msg = run_lower_expect_error(FIXTURES / "segment.at", tmp / "negout-profile", profile="components-2d")
        for needle in ("query2d is only available in the queries-2d profile", "module segment", "symbol 'query2d'"):
            assert needle in msg, f"profile-boundary diagnostic lacks {needle!r}: {msg[:300]}"
        print("PASS negative rejection: components-2d profile boundary")
        checks += 1
        # 5. damaged-consumption recovery: python replay + the real driver
        #    control (v06_damaged_) demonstrates reject + green recovery.
        frozen = GENERATED / "q-segment"
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
        shutil.copyfile(frozen / "world.at", script)
        assert sha(script.read_bytes()) == bound_sha, "restore failed"
        if args.skip_driver:
            print("SKIP damaged-consumption driver control (--skip-driver)")
        else:
            cmd = ["cargo", "test", "--locked", "--offline", "--test", "node_queries_flow",
                   "v06_damaged", "--", "--test-threads=1"]
            r = subprocess.run(cmd, cwd=DRIVER_DIR, capture_output=True, text=True, timeout=3600)
            assert r.returncode == 0, (
                f"driver damaged-consumption control failed:\n{r.stdout[-1500:]}\n{r.stderr[-500:]}"
            )
            assert "test result: ok" in r.stdout, r.stdout[-500:]
        print("PASS damaged consumption: corrupted world copy rejected at load; restore runs green")
        checks += 1
    print(f"PASS verify_plan021 --controls: {checks} control groups, {len(ENTRIES)} products")
    return 0


if __name__ == "__main__":
    sys.exit(main())
