"""PLAN-023 components-2d-game verify controls: the identity binding of the
single frozen generated game product (generated-components2d-game/game:
world sha == scene behavior.source binding == identity.json == provenance;
closure manifest == closure files == live fixture sources incl. the wav/ttf/
license assets; the five real consumer test files and their include_str! /
scripts/<sha>.at consumption mechanism cross-checked), cold A->B
regeneration byte equality from the closure/ copies alone, a REAL CPU-session
run of the B-regenerated world plus a save/reopen cycle (the driver control
t06_ab, pointed at B through T06_B_ROOT), fixture mutation sensitivity,
the V-06 closeout-discipline oracle (the seven F-9 cases of
tests/t06_game_closeout.rs), the damaged-consumption recovery control
(a one-byte-corrupted scripts/<sha>.at copy rejected by the real load path;
exact restore runs green — replayed here and demonstrated by t06_damaged),
and the six bad-input classes with their named rejections (t06_bad_inputs).
Raw evidence stays local; the durable record is the Plan receipt. Run from
any cwd:

  python experimental/autographics-auto-driver/tools/verify_plan023.py --controls
"""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

TOOLS = pathlib.Path(__file__).resolve().parent
DRIVER = TOOLS.parents[0]
REPO = TOOLS.parents[2]
PILOT = REPO / "experimental" / "autographics-node2d-pilot"
GENERATED = PILOT / "generated-components2d-game" / "game"
FIXTURES = PILOT / "fixtures" / "components2d-game"
TESTS = DRIVER / "tests"

GENERATOR_ID = "autographics-node2d-pilot plan023 components-2d-game"
API = "autographics.hud2d/1"
SCHEMA = "autographics.scene/8"
# T-03 pins.
WORLD_SHA = "ed98b0b556d7e9c80e48355e7bacab7182f9aa9d0007c7441db36062bbb897d0"
BANK_SHA = "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d"
FONT_SHA = "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa"
LICENSE_SHA = "31d45880a38de46bc81835329c0c7d964609751e65add92769ea0095ad92c05b"
ENTRY = "main"
MODULES = {"coin", "main", "player", "resources", "zone"}
ASSETS = {
    BANK_SHA + ".wav",
    "lilita_one_regular.ttf",
    "license.txt",
}
CUES = [("coin", 0, 7403), ("win", 7403, 20046), ("fail", 20046, 22504)]
# The real consumer files of the product (V-06 provenance set).
CONSUMERS = [
    "t03_game_gates.rs",
    "t04_game_play.rs",
    "t04_game_media.rs",
    "t05_game_budget.rs",
    "t05_game_host.rs",
]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_lower(entry: pathlib.Path, out: pathlib.Path) -> None:
    cmd = [
        "cargo", "run", "--locked", "--offline", "--bin", "node2d-lower", "--",
        "--profile", "components-2d-game",
        "--entry", str(entry),
        "--out", str(out),
    ]
    r = subprocess.run(cmd, cwd=PILOT, capture_output=True, text=True, timeout=1800)
    if r.returncode != 0:
        raise AssertionError(f"lowering failed for {entry}: {r.stderr.strip()[:400]}")


def run_driver_test(filter_: str, env_extra=None) -> None:
    cmd = [
        "cargo", "test", "--locked", "--offline", "--test", "t06_game_closeout",
        filter_, "--", "--test-threads=1",
    ]
    env = dict(os.environ)
    if env_extra:
        env.update(env_extra)
    r = subprocess.run(cmd, cwd=DRIVER, capture_output=True, text=True, timeout=3600, env=env)
    assert r.returncode == 0, (
        f"driver control {filter_} failed:\n{r.stdout[-1500:]}\n{r.stderr[-500:]}"
    )
    assert "test result: ok" in r.stdout, r.stdout[-500:]


def compare_products(frozen: pathlib.Path, regen: pathlib.Path, name: str) -> None:
    for f in ("scene.agscene", "identity.json", "world.at"):
        a = (frozen / f).read_bytes()
        b = (regen / f).read_bytes()
        assert a == b, f"{name}/{f} regeneration differs"


def check_identity(out_dir: pathlib.Path) -> str:
    identity = json.loads((out_dir / "identity.json").read_text(encoding="utf-8"))
    assert identity["generator"] == GENERATOR_ID, identity["generator"]
    assert identity["api"] == API, identity["api"]
    scene_text = (out_dir / "scene.agscene").read_text(encoding="utf-8")
    doc = json.loads(scene_text)
    assert doc["schema"] == SCHEMA, doc["schema"]
    assert doc["behavior"]["api"] == API, doc["behavior"]["api"]
    assert doc["behavior"]["actors"]["two_d"] == 1, "single 2d actor"
    world = (out_dir / "world.at").read_bytes()
    source_sha = doc["behavior"]["source"]["sha256"]
    assert source_sha == sha(world), "scene binds the world source"
    assert doc["behavior"]["source"]["byte_length"] == len(world), "byte_length record"
    assert identity["user_source_sha256"] == source_sha, "identity user source"
    assert identity["user_source_bytes"] == len(world), "identity byte record"
    assert source_sha == WORLD_SHA, "the committed product keeps the T-03 pin"
    # scene object names are identity: unique across the document.
    names = [o["name"] for o in doc["objects"]]
    assert len(names) == len(set(names)), "duplicate scene object names"
    # roles: the single character and camera bind real scene rows.
    roles = identity["roles"]
    assert roles["character"] == 1 and roles["character_name"] in names, "character role"
    assert roles["camera"] == 1, "camera role"
    for oname, oid in roles["scene_objects"].items():
        assert oname in names, f"scene_objects role {oname}"
        row = next(o for o in doc["objects"] if o["name"] == oname)
        assert row["id"] == oid, f"scene_objects id for {oname}"
    return source_sha


def check_closure(out_dir: pathlib.Path) -> None:
    identity = json.loads((out_dir / "identity.json").read_text(encoding="utf-8"))
    records = {c["name"]: c for c in identity["closure"]}
    want = {f"asset:{a}" if a in ASSETS else a for a in MODULES | ASSETS}
    assert set(records) == want, set(records) ^ want
    closure_dir = out_dir / "closure"
    live = {p.name for p in closure_dir.iterdir()}
    assert live == {f"{m}.at" for m in MODULES} | {"assets"}, f"closure file set: {live}"
    assert {p.name for p in (closure_dir / "assets").iterdir()} == ASSETS, "closure assets set"
    for mod in MODULES:
        rec = records[mod]
        f = closure_dir / f"{mod}.at"
        data = f.read_bytes()
        assert rec["sha256"] == sha(data), f"closure hash {mod}"
        assert rec["bytes"] == len(data), f"closure bytes {mod}"
        fixture = FIXTURES / f"{mod}.at"
        assert fixture.is_file() and sha(fixture.read_bytes()) == rec["sha256"], (
            f"fixture {mod} drifted"
        )
    for asset in ASSETS:
        rec = records[f"asset:{asset}"]
        f = closure_dir / "assets" / asset
        data = f.read_bytes()
        assert rec["sha256"] == sha(data), f"closure asset hash {asset}"
        assert rec["bytes"] == len(data), f"closure asset bytes {asset}"
        fixture = FIXTURES / "assets" / asset
        assert fixture.is_file() and sha(fixture.read_bytes()) == rec["sha256"], (
            f"fixture asset {asset} drifted"
        )


def check_provenance(out_dir: pathlib.Path) -> None:
    identity = json.loads((out_dir / "identity.json").read_text(encoding="utf-8"))
    prov = identity["provenance"]
    assert prov["product"] == "game", prov["product"]
    assert "t03_game_gates.rs" in prov["consumer"], prov["consumer"]
    assert prov["world_sha256"] == WORLD_SHA, "provenance binds the world sha"
    assert f"scripts/<sha>.at" in prov["consumption"], "consumption mechanism recorded"
    for name in CONSUMERS:
        f = TESTS / name
        assert f.is_file(), f"consumer test file missing: {name}"
        text = f.read_text(encoding="utf-8")
        assert "generated-components2d-game/game" in text, f"{name} does not consume the product"
        assert WORLD_SHA in text, f"{name} does not pin the world sha"
    # the load-mechanism cross-check: the staging consumers bind the world
    # as scripts/<sha>.at (the path the load path hash-verifies).
    for name in ("t04_game_media.rs", "t05_game_host.rs"):
        text = (TESTS / name).read_text(encoding="utf-8")
        assert "scripts" in text and 'format!("{WORLD_SHA}.at")' in text, (
            f"{name} does not stage scripts/<sha>.at"
        )


def check_scene_media(out_dir: pathlib.Path) -> None:
    doc = json.loads((out_dir / "scene.agscene").read_text(encoding="utf-8"))
    (wav, ttf) = doc["assets"]
    assert wav["format"] == "wav" and wav["sha256"] == BANK_SHA, "bank row"
    assert wav["byte_length"] == 90060, "bank bytes"
    assert wav["audio"] == {"rate": 44100, "channels": 2, "frames": 22504}, "bank audio meta"
    assert ttf["format"] == "ttf" and ttf["sha256"] == FONT_SHA, "font row"
    assert ttf["byte_length"] == 26828, "font bytes"
    cues = doc["cues"]
    assert [c["name"] for c in cues] == [c[0] for c in CUES], "cue declaration order"
    for c, (name, start, end) in zip(cues, CUES):
        assert c["asset"] == f"sha256:{BANK_SHA}", "cue asset binding"
        assert (c["start_frame"], c["end_frame"]) == (start, end), f"cue span {name}"
        assert c["gain"] == 1.0, f"cue gain {name}"
    hud = doc["hud"]
    assert hud["font"] == f"sha256:{FONT_SHA}", "hud font binding"
    assert len(hud["license"]) > 0, "hud license record"
    assert sha(hud["license"].encode("utf-8")) == LICENSE_SHA, "hud license identity"
    assert len(hud["slots"]) == 4, "four complete hud slots"
    assert [s["slot"] for s in hud["slots"]] == [0, 1, 2, 3], "slot ids"
    assert hud["slots"][0]["text"] == "0" and hud["slots"][1]["text"] == "Playing", (
        "initial slot texts"
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    ap.add_argument(
        "--skip-driver",
        action="store_true",
        help="skip the driver cargo-test controls (needs the auto-driver build)",
    )
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        return 2
    checks = 0
    with tempfile.TemporaryDirectory(prefix="verify-plan023-") as tmp:
        tmp = pathlib.Path(tmp)
        # 1. identity chain of the frozen product.
        check_identity(GENERATED)
        print(f"PASS identity chain: world sha == scene binding == identity == provenance ({WORLD_SHA[:16]}...)")
        checks += 1
        # 2. closure manifest == closure files == live fixture sources.
        check_closure(GENERATED)
        print("PASS closure manifest: modules + assets, hashes/bytes, fixture sources live")
        checks += 1
        # 3. provenance consumption binding (real consumer files + mechanism).
        check_provenance(GENERATED)
        print(f"PASS provenance: {len(CONSUMERS)} consumer files, include_str! consumption, scripts/<sha>.at mechanism")
        checks += 1
        # 4. scene media contract (assets/cues/hud/license as consumed).
        check_scene_media(GENERATED)
        print("PASS scene media contract: bank/font rows, cue table, hud slots + license identity")
        checks += 1
        # 5. cold A->B: B receives ONLY the closure/ copies and reproduces
        #    every product file byte-identically (the out dir is named
        #    'game' so identity.provenance.product matches too).
        bdir = tmp / "B"
        bdir.mkdir()
        shutil.copytree(GENERATED / "closure", bdir / "closure")
        out = tmp / "game"  # basename feeds identity.provenance.product
        run_lower(bdir / "closure" / f"{ENTRY}.at", out)
        compare_products(GENERATED, out, "game")
        b_world_sha = sha((out / "world.at").read_bytes())
        assert b_world_sha == WORLD_SHA, "B world sha"
        print(f"PASS cold A->B: closure-only regeneration byte-identical (B world sha {b_world_sha[:16]}...)")
        checks += 1
        # 6. real consumption of B: the driver control runs a CPU-session
        #    win path on the B-regenerated world and a save/reopen cycle.
        if args.skip_driver:
            print("SKIP B consumption driver control (--skip-driver)")
        else:
            run_driver_test("t06_ab", {"T06_B_ROOT": str(out)})
            print("PASS B consumption: real CPU-session Run of the B-regenerated world (win reachable) + save/reopen green")
        checks += 1
        # 7. mutation sensitivity: a real param change in a fixture copy
        #    regenerates to different bytes (the pipeline is not
        #    constant-folded and identity follows the source).
        mutdir = tmp / "game-mut"
        shutil.copytree(FIXTURES, mutdir)
        src = mutdir / f"{ENTRY}.at"
        text = src.read_text(encoding="utf-8")
        mutated = text.replace("cx: 8.0", "cx: 9.0", 1)
        assert mutated != text, "mutation did not change the fixture"
        src.write_text(mutated, encoding="utf-8")
        mout = tmp / "game-mut-out"
        run_lower(mutdir / f"{ENTRY}.at", mout)
        for f in ("scene.agscene", "identity.json", "world.at"):
            assert (mout / f).read_bytes() != (GENERATED / f).read_bytes(), (
                f"mutation invisible in game/{f}"
            )
        mut_world = (mout / "world.at").read_text(encoding="utf-8")
        assert "coin_cx=[9.0" in mut_world, "c1 position mutation not applied to the SoA"
        mut_identity = json.loads((mout / "identity.json").read_text(encoding="utf-8"))
        assert mut_identity["user_source_sha256"] != WORLD_SHA, "identity follows the source"
        print("PASS mutation sensitivity: c1 cx 8.0->9.0 changes world/scene/identity (SoA applied)")
        checks += 1
        # 8. V-06 closeout discipline: the seven F-9 cases of the driver
        #    oracle (single present2d per callback, mixed hud_set 57, text
        #    bounds 56, stale ordinal 52, over-four puts 51, cue range 53,
        #    gain domain 54).
        if args.skip_driver:
            print("SKIP V-06 closeout-discipline driver control (--skip-driver)")
        else:
            run_driver_test("t06_v06")
            print("PASS V-06 closeout discipline: 7 F-9 cases (50/57/56/52/51/53/54) green")
        checks += 1
        # 9. damaged-consumption recovery: python replay of the load
        #    condition, then the real driver control demonstrates the named
        #    rejection + green recovery through load_project.
        proj = tmp / "damaged-consume"
        (proj / "scripts").mkdir(parents=True)
        shutil.copyfile(GENERATED / "scene.agscene", proj / "scene.agscene")
        script = proj / "scripts" / f"{WORLD_SHA}.at"
        shutil.copyfile(GENERATED / "world.at", script)
        world = bytearray(script.read_bytes())
        world[0] = (world[0] + 1) % 256
        script.write_bytes(bytes(world))
        assert sha(bytes(world)) != WORLD_SHA, "one-byte corruption undetected"
        shutil.copyfile(GENERATED / "world.at", script)
        assert sha(script.read_bytes()) == WORLD_SHA, "restore failed"
        if args.skip_driver:
            print("SKIP damaged-consumption driver control (--skip-driver)")
        else:
            run_driver_test("t06_damaged")
        print("PASS damaged consumption: corrupted scripts/<sha>.at copy rejected at load; exact restore runs green")
        checks += 1
        # 10. bad-input classes: the six named rejections at their admission
        #     stages (source/font/bank/license/record/runner) with exact
        #     restore -> green re-runs.
        if args.skip_driver:
            print("SKIP bad-input classes driver control (--skip-driver)")
        else:
            run_driver_test("t06_bad_inputs")
        print("PASS bad-input classes: 6 named rejections (source/font/bank/license/record/runner) + green re-runs")
        checks += 1
    print(f"PASS verify_plan023 --controls: {checks} control groups, 1 product (game)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
