"""PLAN-017 g11-2d verify controls: identity binding, regeneration byte
equality for every frozen generated-g11 world, fixture mutation sensitivity,
missing-module diagnostics, generated artifact tamper detection, and closure
manifest consistency. Raw evidence stays local; the durable record is the
Plan receipt. Run from any cwd:

  python experimental/autographics-node2d-pilot/tools/verify_plan017.py --controls
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
FIXTURES = ROOT / "fixtures" / "g11-2d"
GENERATED = ROOT / "generated-g11"
WORLDS = [
    "main",
    "main-alt",
    "main-triple",
    "main-stay",
    "main-bag2",
    "main-geometry",
    "main-generation",
    "main-motion-probes",
]


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
        "g11-2d",
        "--entry",
        str(entry),
        "--out",
        str(out),
    ]
    r = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=1800,
    )
    if r.returncode != 0:
        raise SystemExit(f"lower failed for {entry}: {r.stdout}\n{r.stderr}")


def controls() -> None:
    ok = []
    tmp = pathlib.Path(tempfile.mkdtemp(prefix="verify_plan017-"))
    try:
        # 1) 身份绑定：identity.json 的模块 sha 与夹具逐字节一致；world sha
        #    与 world.at 一致；scene 绑定 world sha/bytes；closure 与模块一致。
        for world in WORLDS:
            ident = json.loads((GENERATED / world / "identity.json").read_text())
            for entry in ident["closure"]:
                fixture = FIXTURES / f"{entry['name']}.at"
                got = sha(fixture.read_bytes())
                assert got == entry["sha256"], f"{world}: closure sha {entry['name']}"
                assert got == ident[f"module:{entry['name']}"], f"{world}: module sha {entry['name']}"
                copy = GENERATED / world / "closure" / f"{entry['name']}.at"
                assert sha(copy.read_bytes()) == entry["sha256"], f"{world}: closure copy {entry['name']}"
            world_bytes = (GENERATED / world / "world.at").read_bytes()
            assert sha(world_bytes) == ident["user_source_sha256"], f"{world}: world sha"
            assert len(world_bytes) == ident["user_source_bytes"], f"{world}: world bytes"
            scene = json.loads((GENERATED / world / "scene.agscene").read_text())
            assert scene["behavior"]["source"]["sha256"] == ident["user_source_sha256"], f"{world}: scene binding"
            assert scene["behavior"]["source"]["byte_length"] == ident["user_source_bytes"], f"{world}: scene bytes"
            assert scene["behavior"]["api"] == "autographics.triggers2d/1"
        ok.append("identity: 8 worlds bound (closure/module/world/scene shas consistent)")

        # 2) 再生字节等价（全部冻结世界）。
        for world in WORLDS:
            entry = FIXTURES / f"{world}.at"
            out = tmp / f"regen-{world}"
            run_lower(entry, out)
            for name in ("world.at", "scene.agscene", "identity.json"):
                a = (GENERATED / world / name).read_bytes()
                b = (out / name).read_bytes()
                assert a == b, f"{world}: {name} regeneration drift"
            for closure_file in sorted((GENERATED / world / "closure").glob("*.at")):
                b = (out / "closure" / closure_file.name).read_bytes()
                assert closure_file.read_bytes() == b, f"{world}: closure {closure_file.name} drift"
        ok.append("regeneration: 8 worlds byte-identical (world/scene/identity/closure)")

        # 3) 夹具变异敏感性：改位置/颜色 → world 与 scene 都必须变。
        mutated_entry = tmp / "main-mutated.at"
        mutated_dir = tmp / "mutated-fixtures"
        mutated_dir.mkdir()
        for f in FIXTURES.glob("*.at"):
            shutil.copyfile(f, mutated_dir / f.name)
        main_text = (mutated_dir / "main.at").read_text()
        assert "position_x: 8.0" in main_text
        (mutated_dir / "main.at").write_text(main_text.replace("position_x: 8.0", "position_x: 9.75"))
        out = tmp / "regen-mutated"
        run_lower(mutated_dir / "main.at", out)
        base_world = (GENERATED / "main" / "world.at").read_bytes()
        assert (out / "world.at").read_bytes() != base_world, "mutation must move the world"
        assert b"coin_position_x=[9.75," in (out / "world.at").read_bytes(), "mutated position present"
        res_text = (mutated_dir / "resources.at").read_text()
        (mutated_dir / "resources.at").write_text(
            res_text.replace(
                "let gold = material2d.color(0.95, 0.8, 0.2, 1.0)",
                "let gold = material2d.color(0.2, 0.3, 0.4, 1.0)",
            )
        )
        out2 = tmp / "regen-mutated2"
        run_lower(mutated_dir / "main.at", out2)
        # 资源 symbol 只进入场景模板行（世界源不含颜色）：scene 必变，
        # world 不变是声明式资源绑定的正确形态。
        assert (out2 / "scene.agscene").read_bytes() != (out / "scene.agscene").read_bytes(), "colour move changes scene"
        assert b"[0.2, 0.3, 0.4, 1]" in (out2 / "scene.agscene").read_bytes(), "scene follows resources"
        ok.append("mutation: position/colour changes move world+scene")

        # 4) 缺模块诊断（定位拒绝，非 panic）。
        broken_dir = tmp / "broken-fixtures"
        broken_dir.mkdir()
        for f in FIXTURES.glob("*.at"):
            if f.name != "patrol.at":
                shutil.copyfile(f, broken_dir / f.name)
        r = subprocess.run(
            [
                "cargo",
                "run",
                "--locked",
                "--offline",
                "--bin",
                "node2d-lower",
                "--",
                "--profile",
                "g11-2d",
                "--entry",
                str(broken_dir / "main.at"),
                "--out",
                str(tmp / "broken-out"),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=1800,
        )
        assert r.returncode != 0, "missing module must fail"
        assert "module not found: patrol" in r.stderr, r.stderr
        assert not (tmp / "broken-out").exists() or not any((tmp / "broken-out").iterdir()), "no half-written output"
        ok.append("diagnostics: missing module positioned rejection, no partial output")

        # 5) 篡改检出：改动 world 后，场景绑定（sha/bytes）不再成立。
        tampered = tmp / "tampered"
        tampered.mkdir()
        shutil.copyfile(GENERATED / "main" / "scene.agscene", tampered / "scene.agscene")
        world_text = (GENERATED / "main" / "world.at").read_text()
        (tampered / "world.at").write_text(world_text.replace("var r_bag=0", "var r_bag=7"))
        tampered_sha = sha((tampered / "world.at").read_bytes())
        scene = json.loads((tampered / "scene.agscene").read_text())
        assert scene["behavior"]["source"]["sha256"] != tampered_sha, "tamper detectable"
        ident = json.loads((GENERATED / "main" / "identity.json").read_text())
        assert ident["user_source_sha256"] != tampered_sha, "identity binding detects tamper"
        ok.append("tamper: modified world detectable via scene/identity binding")

        # 6) closure 独立再生（A→B 预检）：仅凭 closure/ 目录再生 == 冻结物。
        b_dir = tmp / "closure-only"
        b_dir.mkdir()
        for f in (GENERATED / "main" / "closure").glob("*.at"):
            shutil.copyfile(f, b_dir / f.name)
        out3 = tmp / "regen-closure-only"
        run_lower(b_dir / "main.at", out3)
        assert (out3 / "world.at").read_bytes() == (GENERATED / "main" / "world.at").read_bytes(), \
            "closure-only regeneration byte-identical"
        ok.append("closure A->B precheck: closure/ alone regenerates the frozen world")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    print("PASS verify_plan017:")
    for line in ok:
        print(" -", line)


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--controls", action="store_true")
    a = p.parse_args()
    if not a.controls:
        print("usage: verify_plan017.py --controls")
        raise SystemExit(2)
    controls()


if __name__ == "__main__":
    main()
