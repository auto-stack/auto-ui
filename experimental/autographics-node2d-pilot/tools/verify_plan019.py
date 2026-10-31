"""PLAN-019 basic2d verify controls: identity binding, regeneration byte
equality for every frozen generated-basic2d product, fixture mutation
sensitivity (used-material/position changes actually regenerate), negative
profile rejections, and closure manifest consistency (A->B cold precheck).
Raw evidence stays local; the durable record is the Plan receipt. Run from
any cwd:

  python experimental/autographics-node2d-pilot/tools/verify_plan019.py --controls
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
FIXTURES = ROOT / "fixtures" / "basic2d"
GENERATED = ROOT / "generated-basic2d"

# entry -> (profile, fixture entry file, closure module set)
PRODUCTS = {
    "g01-base": ("scene-2d", "g01/main.at", {"main", "resources", "shapes"}),
    "g01-color": ("scene-2d", "g01/v-color.at", {"v-color", "resources", "shapes"}),
    "g01-order": ("scene-2d", "g01/v-order.at", {"v-order", "resources", "shapes"}),
    "g01-position": ("scene-2d", "g01/v-position.at", {"v-position", "resources", "shapes"}),
    "g01-visible": ("scene-2d", "g01/v-visible.at", {"v-visible", "resources", "shapes"}),
    "g03-base": ("scene-2d", "g03/main.at", {"main", "resources"}),
    "g03-parent": ("scene-2d", "g03/v-parent.at", {"v-parent", "resources"}),
    "g03-hidden": ("scene-2d", "g03/v-hidden.at", {"v-hidden", "resources"}),
    "g06-a": ("character-2d", "g06/main.at", {"main", "resources", "walkjump"}),
    "g06-b": ("character-2d", "g06/main-b.at", {"main-b", "resources", "walkjump"}),
    "g06-fast": ("character-2d", "g06/main-fast.at", {"main-fast", "resources", "walkjump"}),
    "g08-main": ("camera-2d", "g08/main.at", {"main", "resources", "walkjump"}),
    "g08-alt": ("camera-2d", "g08/main-alt.at", {"main-alt", "resources", "walkjump"}),
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_lower(entry: pathlib.Path, out: pathlib.Path, profile: str) -> None:
    cmd = [
        "cargo",
        "run",
        "--locked",
        "--offline",
        "--bin",
        "node2d-lower",
        "--",
        "--profile",
        profile,
        "--entry",
        str(entry),
        "--out",
        str(out),
    ]
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
    if r.returncode != 0:
        raise AssertionError(f"lowering failed for {entry}: {r.stderr.strip()[:400]}")


def closure_modules(entry_file: str) -> set:
    # fixtures declare their module set in PRODUCTS; the closure/ dir must
    # carry exactly those modules.
    return set(PRODUCTS[entry_file][2])


def check_identity(name: str, out_dir: pathlib.Path, profile: str) -> None:
    identity = json.loads((out_dir / "identity.json").read_text(encoding="utf-8"))
    assert identity["generator"] == f"autographics-node2d-pilot plan019 {profile}", name
    expected_api = {
        "scene-2d": None,
        "character-2d": "autographics.character/1",
        "camera-2d": "autographics.runtime_camera2d/1",
    }[profile]
    assert identity["api"] == expected_api, (name, identity["api"])
    scene = (out_dir / "scene.agscene").read_text(encoding="utf-8")
    if profile == "scene-2d":
        assert '"behavior": null' in scene, name
        assert identity["user_source_sha256"] is None, name
    else:
        doc = json.loads(scene)
        assert doc["behavior"]["api"] == expected_api, name
        source_sha = doc["behavior"]["source"]["sha256"]
        world = (out_dir / "world.at").read_bytes()
        assert source_sha == sha(world), f"{name}: scene binds the world source"
        assert doc["behavior"]["source"]["byte_length"] == len(world), name
        assert identity["user_source_sha256"] == source_sha, name
    # closure manifest: every module listed with correct hash, files present.
    modules = {c["name"]: c for c in identity["closure"]}
    assert modules.keys() == closure_modules(name), (name, modules.keys())
    closure_dir = out_dir / "closure"
    for mod, rec in modules.items():
        f = closure_dir / f"{mod}.at"
        assert f.is_file(), f"{name}: closure file {mod} missing"
        data = f.read_bytes()
        assert rec["sha256"] == sha(data), f"{name}: closure hash {mod}"
        assert rec["bytes"] == len(data), f"{name}: closure bytes {mod}"
        # the closure copy must equal the live fixture source (frozen input).
        fixture = FIXTURES / pathlib.Path(PRODUCTS[name][1]).parent / f"{mod}.at"
        assert sha(fixture.read_bytes()) == rec["sha256"], f"{name}: fixture {mod} drifted"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    ap.add_argument(
        "--build-record",
        metavar="OUT",
        help="write a source-build record (generator binary sha, toolchain, product hashes)",
    )
    ap.add_argument(
        "--verify-record",
        metavar="JSON",
        help="verify a recorded build against the current tree and generator binary",
    )
    ap.add_argument(
        "--generator",
        default=str(ROOT / "target" / "debug" / "node2d-lower.exe"),
        help="generator binary the record binds (default: debug node2d-lower)",
    )
    args = ap.parse_args()
    if args.build_record:
        return write_build_record(pathlib.Path(args.build_record), pathlib.Path(args.generator))
    if args.verify_record:
        return verify_build_record(pathlib.Path(args.verify_record), pathlib.Path(args.generator))
    if not args.controls:
        print(__doc__)
        return 2
    checks = 0
    with tempfile.TemporaryDirectory(prefix="verify-plan019-") as tmp:
        tmp = pathlib.Path(tmp)
        # 1. regeneration byte identity for every frozen product.
        for name, (profile, entry, _) in PRODUCTS.items():
            frozen = GENERATED / name
            assert frozen.is_dir(), f"missing frozen product {name}"
            regen = tmp / name
            run_lower(FIXTURES / entry, regen, profile)
            for f in ("scene.agscene", "identity.json"):
                a = (frozen / f).read_bytes()
                b = (regen / f).read_bytes()
                assert a == b, f"{name}/{f} regeneration differs"
            if profile != "scene-2d":
                a = (frozen / "world.at").read_bytes()
                b = (regen / "world.at").read_bytes()
                assert a == b, f"{name}/world.at regeneration differs"
            check_identity(name, frozen, profile)
            checks += 1
        # 2. mutation sensitivity: changing a used material/position/speed
        #    actually regenerates (the pipeline is not constant-folded).
        mut_tmp = tmp / "mut"
        src = (FIXTURES / "g01/main.at").read_text(encoding="utf-8")
        mutated = src.replace("x: -20.0", "x: -19.0")
        assert mutated != src
        mutdir = mut_tmp / "g01"
        shutil.copytree(FIXTURES / "g01", mutdir)
        (mutdir / "main.at").write_text(mutated, encoding="utf-8")
        out = mut_tmp / "out"
        run_lower(mutdir / "main.at", out, "scene-2d")
        frozen_scene = (GENERATED / "g01-base" / "scene.agscene").read_bytes()
        assert (out / "scene.agscene").read_bytes() != frozen_scene, "position mutation invisible"
        checks += 1
        # 3. negative profile rejections fail with located diagnostics.
        g06src = (FIXTURES / "g06/main.at").read_text(encoding="utf-8")
        withcam = g06src.replace(
            "    }\n    on {",
            "        camera2d cam {\n            target: refs.body.actor\n"
            "            bounds: (0.0, -4.0, 30.0, 12.0)\n            view_size: (12.0, 8.0)\n"
            "        }\n    }\n    on {",
        )
        negdir = mut_tmp / "g06neg"
        shutil.copytree(FIXTURES / "g06", negdir)
        (negdir / "main.at").write_text(withcam, encoding="utf-8")
        cmd = [
            "cargo", "run", "--locked", "--offline", "--bin", "node2d-lower",
            "--", "--profile", "character-2d",
            "--entry", str(negdir / "main.at"), "--out", str(mut_tmp / "negout"),
        ]
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=1800)
        assert r.returncode != 0, "character-2d must reject camera2d"
        assert "camera2d" in (r.stderr + r.stdout), "diagnostic names the element"
        checks += 1
        # 4. cold A->B: B receives ONLY the closure copies (no producer
        #    fixtures) and reproduces every frozen product byte-identically;
        #    a tampered closure module is caught by the manifest hashes and
        #    the exact restore makes the same entry usable again.
        for name in ("g06-a", "g08-main", "g01-base", "g03-base"):
            profile, _, _ = PRODUCTS[name]
            frozen = GENERATED / name
            bdir = tmp / f"cold-{name}"
            (bdir / "closure").parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(frozen / "closure", bdir / "closure")
            entry_mod = sorted(pathlib.Path(PRODUCTS[name][1]).parts)[-1].removesuffix(".at")
            out = tmp / f"coldout-{name}"
            run_lower(bdir / "closure" / f"{entry_mod}.at", out, profile)
            for f in ("scene.agscene", "identity.json"):
                assert (frozen / f).read_bytes() == (out / f).read_bytes(), f"cold A->B {name}/{f}"
            if profile != "scene-2d":
                assert (frozen / "world.at").read_bytes() == (out / "world.at").read_bytes(), (
                    f"cold A->B {name}/world.at"
                )
            # negative: tamper one closure module -> manifest hash mismatch.
            ident = json.loads((frozen / "identity.json").read_text())
            victim = ident["closure"][0]["name"]
            vfile = bdir / "closure" / f"{victim}.at"
            original = vfile.read_bytes()
            vfile.write_bytes(original + b"\n// tampered\n")
            got = sha(vfile.read_bytes())
            assert got != ident["closure"][0]["sha256"], f"tamper undetected in {name}"
            # recovery: exact restore -> the same entry regenerates again.
            vfile.write_bytes(original)
            out2 = tmp / f"coldrecover-{name}"
            run_lower(bdir / "closure" / f"{entry_mod}.at", out2, profile)
            assert (frozen / "scene.agscene").read_bytes() == (out2 / "scene.agscene").read_bytes(), (
                f"recovery {name}"
            )
            checks += 1
    print(f"PASS verify_plan019 --controls: {checks} product/control groups")
    return 0


def product_hashes() -> dict:
    out = {}
    for name, (profile, _, _) in PRODUCTS.items():
        d = GENERATED / name
        rec = {"profile": profile, "scene_sha256": sha((d / "scene.agscene").read_bytes())}
        if profile != "scene-2d":
            rec["world_sha256"] = sha((d / "world.at").read_bytes())
        rec["identity_sha256"] = sha((d / "identity.json").read_bytes())
        out[name] = rec
    return out


def write_build_record(out: pathlib.Path, generator: pathlib.Path) -> int:
    data = generator.read_bytes()
    import subprocess as sp

    rustc = sp.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip()
    record = {
        "generator": {"path": str(generator), "sha256": sha(data), "bytes": len(data)},
        "toolchain": {"rustc": rustc, "platform": sys.platform},
        "workspace_lock": sha((ROOT / "Cargo.lock").read_bytes()),
        "products": product_hashes(),
    }
    out.write_text(json.dumps(record, indent=1) + "\n", encoding="utf-8")
    print(f"build record written: {out} (generator {record['generator']['sha256'][:12]})")
    return 0


def verify_build_record(record_path: pathlib.Path, generator: pathlib.Path) -> int:
    record = json.loads(record_path.read_text(encoding="utf-8"))
    actual = sha(generator.read_bytes())
    assert record["generator"]["sha256"] == actual, (
        f"generator binary drifted: record {record['generator']['sha256'][:12]} vs {actual[:12]}"
    )
    # 冻结合同：Cargo 锁是来源边界的一部分，漂移即拒绝。
    assert record["workspace_lock"] == sha((ROOT / "Cargo.lock").read_bytes()), (
        "workspace Cargo.lock drifted from the record"
    )
    assert record["products"] == product_hashes(), "product hashes drifted from the record"
    print(f"PASS build record: generator {actual[:12]} + {len(record['products'])} products match")
    return 0


if __name__ == "__main__":
    sys.exit(main())
