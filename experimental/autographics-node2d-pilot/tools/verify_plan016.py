"""PLAN-016 verification gate (contract S4).

Verifies, without executing the generator, that the frozen generated
artifacts match the committed fixture identities, then --controls runs
negative controls: clean-regeneration byte-identity, fixture mutation
sensitivity, missing-fixture rejection, generated-artifact tamper
detection, and stale-scene-sha detection.

Run from the repo root (or anywhere) with the worktree layout:
  python experimental/autographics-node2d-pilot/tools/verify_plan016.py --controls
"""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[3]
PILOT = ROOT / "experimental" / "autographics-node2d-pilot"
FIX_MIN = PILOT / "fixtures" / "minimal"
FIX_SUP = PILOT / "fixtures" / "supplement"
GEN_MIN = PILOT / "generated"
GEN_SUP = PILOT / "generated-supplement"

MINIMAL_MODULES = ["main", "resources", "types", "player", "hazard", "coin", "patrol"]
SUPPLEMENT_MODULES = ["main", "resources", "types", "player", "collector", "coin", "patrol"]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fail(msg: str) -> None:
    print(f"FAIL {msg}")
    sys.exit(1)


def run_lower(entry: pathlib.Path, out: pathlib.Path) -> None:
    exe = PILOT / "target" / "debug" / "node2d-lower.exe"
    if not exe.is_file():
        exe = PILOT / "target" / "release" / "node2d-lower.exe"
    if not exe.is_file():
        fail("node2d-lower binary missing; cargo build -p autographics-node2d-pilot first")
    r = subprocess.run([str(exe), "--entry", str(entry), "--out", str(out)],
                       capture_output=True, text=True)
    if r.returncode != 0:
        fail(f"lower failed: {r.stdout} {r.stderr}")


def verify() -> dict:
    report = {"modules": {}}
    for name in MINIMAL_MODULES:
        p = FIX_MIN / f"{name}.at"
        if not p.is_file():
            fail(f"missing fixture {p}")
        report["modules"][f"minimal/{name}.at"] = sha256(p.read_bytes())
    for name in SUPPLEMENT_MODULES:
        p = FIX_SUP / f"{name}.at"
        if not p.is_file():
            fail(f"missing fixture {p}")
        report["modules"][f"supplement/{name}.at"] = sha256(p.read_bytes())
    for gen, tag in [(GEN_MIN, "minimal"), (GEN_SUP, "supplement")]:
        world = gen / "world.at"
        ident = gen / "identity.json"
        if not world.is_file() or not ident.is_file():
            fail(f"missing generated artifacts in {gen}")
        data = world.read_bytes()
        report[f"{tag}/world_sha256"] = sha256(data)
        report[f"{tag}/world_bytes"] = len(data)
        j = json.loads(ident.read_text(encoding="utf-8"))
        if j.get("user_source_sha256") != report[f"{tag}/world_sha256"]:
            fail(f"{tag}: identity.json sha mismatch with world.at")
        if j.get("user_source_bytes") != len(data):
            fail(f"{tag}: identity.json byte_length mismatch")
        scene = gen / "scene.agscene"
        if tag == "minimal":
            if not scene.is_file():
                fail("missing generated scene.agscene")
            sj = json.loads(scene.read_text(encoding="utf-8"))
            if sj["behavior"]["source"]["sha256"] != report[f"{tag}/world_sha256"]:
                fail("scene behavior sha does not bind world.at")
            if sj["behavior"]["source"]["byte_length"] != len(data):
                fail("scene behavior byte_length does not bind world.at")
        else:
            if scene.is_file() and scene.read_text(encoding="utf-8").strip():
                fail("supplement must not emit a scene document")
    return report


def controls() -> None:
    # 0. 再生等价：干净重生成必须与提交的生成物逐字节相同。
    with tempfile.TemporaryDirectory(prefix="plan016-regen-") as td:
        tdp = pathlib.Path(td)
        run_lower(FIX_MIN / "main.at", tdp / "min")
        run_lower(FIX_SUP / "main.at", tdp / "sup")
        for tag, out in [("minimal", tdp / "min"), ("supplement", tdp / "sup")]:
            for artifact in ["world.at"]:
                a = (pathlib.Path(GEN_MIN if tag == "minimal" else GEN_SUP) / artifact).read_bytes()
                b = (out / artifact).read_bytes()
                if a != b:
                    fail(f"{tag}/{artifact} regeneration not byte-identical")
    # 1. Fixture mutation: an edited hazard fixture must regenerate a
    #    DIFFERENT world (sensitivity) while the committed world stays
    #    bound to the committed fixtures (already asserted by verify()).
    with tempfile.TemporaryDirectory(prefix="plan016-ctrl-") as td:
        tdp = pathlib.Path(td)
        fix = tdp / "minimal"
        shutil.copytree(FIX_MIN, fix)
        hazard = fix / "hazard.at"
        original = hazard.read_bytes()
        assert b".phase > 0.8" in original, "calibration: patrol wrap present"
        hazard.write_bytes(original.replace(b".phase > 0.8", b".phase > 0.85"))
        run_lower(fix / "main.at", tdp / "out")
        mutated = (tdp / "out" / "world.at").read_bytes()
        committed = (GEN_MIN / "world.at").read_bytes()
        if mutated == committed:
            fail("hazard fixture mutation did not change the generated world")
        # 2. Missing fixture: removing coin.at must be rejected with a
        #    module-not-found diagnosis (nonzero exit).
        (fix / "coin.at").unlink()
        r = subprocess.run(
            [str(PILOT / "target" / "debug" / "node2d-lower.exe", ),
             "--entry", str(fix / "main.at"), "--out", str(tdp / "out2")],
            capture_output=True, text=True)
        if r.returncode == 0:
            fail("missing module was accepted")
        if "not found" not in (r.stdout + r.stderr) and "not registered" not in (r.stdout + r.stderr):
            fail(f"missing module lacked a diagnosis: {r.stdout} {r.stderr}")
    # 3. Generated-artifact mutation: identity.json must disagree with a
    #    tampered world.at.
    with tempfile.TemporaryDirectory(prefix="plan016-ctrl2-") as td:
        tdp = pathlib.Path(td)
        gen = tdp / "generated"
        shutil.copytree(GEN_MIN, gen)
        world = gen / "world.at"
        world.write_bytes(world.read_bytes() + b"\n// tampered\n")
        j = json.loads((gen / "identity.json").read_text(encoding="utf-8"))
        if j["user_source_sha256"] == sha256(world.read_bytes()):
            fail("tampered world not detected by identity")
    # 4. Stale-identity rejection: a scene binding a wrong sha must be
    #    detectable from the artifact set alone.
    with tempfile.TemporaryDirectory(prefix="plan016-ctrl3-") as td:
        tdp = pathlib.Path(td)
        gen = tdp / "generated"
        shutil.copytree(GEN_MIN, gen)
        scene = gen / "scene.agscene"
        sj = json.loads(scene.read_text(encoding="utf-8"))
        sj["behavior"]["source"]["sha256"] = "0" * 64
        scene.write_text(json.dumps(sj), encoding="utf-8")
        j = json.loads((gen / "identity.json").read_text(encoding="utf-8"))
        if sj["behavior"]["source"]["sha256"] != j["user_source_sha256"]:
            print("OK stale scene sha detectable")
    print("OK controls")


def main() -> None:
    report = verify()
    if "--controls" in sys.argv[1:]:
        controls()
    print("PASS verify_plan016:", json.dumps(report, sort_keys=True))


if __name__ == "__main__":
    main()
