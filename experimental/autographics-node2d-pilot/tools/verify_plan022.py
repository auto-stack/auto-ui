#!/usr/bin/env python3
"""PLAN-022 sprites-2d consumption verifier (--controls).

Identity chain, cold A->B regeneration, mutation sensitivity, license/provenance
closure and named negative replays for the sprites-2d product family. The run
level (real Auto/GPU oracle replay of the original 18 G09 acceptance names)
lives in the driver test node_sprites_flow; this verifier pins the file
identity those products must keep.

Run from the pilot directory:
    python -I tools/verify_plan022.py --controls
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile

PILOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURES = os.path.join(PILOT, "fixtures", "sprites2d")
PRODUCTS_DIR = os.path.join(PILOT, "generated-sprites2d")

PRODUCTS = [
    "atlas-g09",
    "atlas-alt",
    "atlas-flip-control",
    "atlas-slot",
    "character-g09",
    "character-alt",
    "character-wrong-air",
    "character-budget-32",
    "character-budget-33",
    "pair-isolated",
]

NEGATIVES = [
    ("neg-frame-rect", "frame rect [90, 0, 32, 32] exceeds image 'atlas_img' (96x32)"),
    ("neg-frame-lit", "literal frame ordinal 7 exceeds the frame table of image 'atlas_img' (3 frame(s))"),
    ("neg-image-type", "binds 'resources.marker_color' which is not an image2d.png resource"),
    ("neg-flip-lit", "literal flip must be 0/1, got 2"),
    ("neg-unknown-spawn", "unknown render2d.spawn target 'no_such_leaf'"),
    ("slot-9", "runtime object slots exceed the frozen 8-slot pool"),
]

ORIGIN_PNGS = {
    "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.png": 258,
    "8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.png": 735,
}

ORIGIN_LICENSES = {
    "95a08525ebe48c2423f04ecb8105965109c5a1c6b55fb6d15c154f3338a5bbb1.LICENSE":
        "203e1a7fe8bdc911f2cf316ed15524f29609c9aea1570485ff2cecd504e01fdb",
    "8fde96f66bf875ec06cfe7058348cd572c469413381e8204b1cd5973058998df.LICENSE":
        "18f4ad513a717b421bee861681fc6c5d2468bc715a9fda1a7b82b2169e766596",
}

ORIGIN_MANIFESTS = {
    "atlas-asset-manifest.json":
        "d2d3abd32eadeb8fdda4da71ac99312f178f6d0b7c99b89e3c9c6f1be658ca19",
    "character-asset-manifest.json":
        "5badccf1a4796e356006f18f1981ce12f0c633b61cae7760a397e5ef964faa30",
}


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def lower(entry, out):
    exe = os.path.join(PILOT, "target", "debug", "node2d-lower.exe")
    return subprocess.run(
        [exe, "--profile", "sprites-2d", "--entry", entry, "--out", out],
        capture_output=True, text=True,
    )


def read(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def jload(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def control_identity(results):
    for prod in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        scene = jload(os.path.join(pdir, "scene.agscene"))
        identity = jload(os.path.join(pdir, "identity.json"))
        world_path = os.path.join(pdir, "world.at")
        world_bytes = open(world_path, "rb").read()
        world_sha = sha256_bytes(world_bytes)
        bound = scene["behavior"]["source"]["sha256"]
        assert bound == world_sha, f"{prod}: scene binds {bound} != world sha {world_sha}"
        assert identity["user_source_sha256"] == world_sha, f"{prod}: identity world sha mismatch"
        assert identity["api"] == "autographics.sprites2d/1", f"{prod}: api mismatch"
        assert scene["schema"] == "autographics.scene/6", f"{prod}: schema mismatch"

        # closure: every module file equals the frozen fixture module
        fixtures = {}
        fdir = os.path.join(FIXTURES, prod)
        for name in sorted(os.listdir(fdir)):
            if name.endswith(".at"):
                fixtures[name[:-3]] = os.path.join(fdir, name)
        closure = {}
        cdir = os.path.join(pdir, "closure")
        for name in sorted(os.listdir(cdir)):
            if name.endswith(".at"):
                closure[name[:-3]] = os.path.join(cdir, name)
        assert set(closure) == set(fixtures), (
            f"{prod}: closure modules {sorted(closure)} != fixture modules {sorted(fixtures)}"
        )
        for mod, cpath in closure.items():
            assert sha256_file(cpath) == sha256_file(fixtures[mod]), (
                f"{prod}: closure module {mod} drifted from the fixture"
            )
        id_closure = {c["name"]: c["sha256"] for c in identity["closure"]}
        for mod, cpath in closure.items():
            assert id_closure[mod] == sha256_file(cpath), f"{prod}: identity closure sha {mod}"

        # assets: scene entries == bytes on disk == fixture origin bytes
        fixed_assets = os.path.join(FIXTURES, "assets")
        for asset in scene["assets"]:
            assert asset["format"] == "png", f"{prod}: asset format"
            blob = os.path.join(pdir, "assets", asset["name"])
            data = open(blob, "rb").read()
            assert len(data) == asset["byte_length"], f"{prod}: asset byte length"
            assert sha256_bytes(data) == asset["sha256"], f"{prod}: asset sha"
            origin = os.path.join(fixed_assets, asset["name"])
            assert sha256_file(origin) == asset["sha256"], (
                f"{prod}: product asset drifted from the fixed origin"
            )
        id_assets = {a["sha256"] for a in identity.get("assets", [])}
        scene_assets = {a["sha256"] for a in scene["assets"]}
        assert id_assets == scene_assets, f"{prod}: identity assets != scene assets"

        # every bound leaf carries the frozen document sprite schema
        bound_rows = [o for o in scene["objects"] if o["sprite"]]
        assert bound_rows, f"{prod}: no sprite bindings"
        for o in bound_rows:
            sprite = o["sprite"]
            assert sprite["asset"].startswith("sha256:"), f"{prod}: asset ref form"
            frame = sprite["frame"]
            assert len(frame) == 4 and all(isinstance(v, int) and v >= 0 for v in frame), (
                f"{prod}: frame must be integer pixels"
            )
        results[f"{prod}/identity"] = {
            "world_sha256": world_sha,
            "closure_modules": len(closure),
            "assets": len(scene["assets"]),
            "bound_rows": len(bound_rows),
        }


def control_cold_regen(results, workdir):
    for prod in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        identity = jload(os.path.join(pdir, "identity.json"))
        b = os.path.join(workdir, f"{prod}-B")
        os.makedirs(b, exist_ok=True)
        # B holds ONLY the closure copy + asset bytes (no producer tree)
        for name in os.listdir(os.path.join(pdir, "closure")):
            shutil.copyfile(
                os.path.join(pdir, "closure", name), os.path.join(b, name)
            )
        bassets = os.path.join(b, "assets")
        os.makedirs(bassets, exist_ok=True)
        # B 只持闭包 + 图像字节（内容身份 sha 名）；声明路径由闭包模块文本
        # 与 identity 的 declared 映射恢复（B 不借生产者源树）。
        id_assets = {
            a["sha256"]: a.get("declared", a["sha256"] + ".png")
            for a in identity.get("assets", [])
        }
        for name in os.listdir(os.path.join(pdir, "assets")):
            sha = name[:-4] if name.endswith(".png") else name
            declared = id_assets.get(sha, name)
            shutil.copyfile(
                os.path.join(pdir, "assets", name),
                os.path.join(bassets, os.path.basename(declared)),
            )
        r = lower(os.path.join(b, "main.at"), os.path.join(workdir, f"{prod}-B-out"))
        assert r.returncode == 0, f"{prod}: cold regen failed: {r.stderr}"
        for artifact in ("world.at", "scene.agscene"):
            a = open(os.path.join(pdir, artifact), "rb").read()
            c = open(os.path.join(workdir, f"{prod}-B-out", artifact), "rb").read()
            assert a == c, f"{prod}: cold A->B {artifact} not byte-equal"
        results[f"{prod}/cold_regen"] = "byte-equal world.at + scene.agscene from closure+assets"


def control_mutation_sensitivity(results, workdir):
    for prod in ["atlas-g09", "character-g09"]:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        fdir = os.path.join(FIXTURES, prod)
        mut = os.path.join(workdir, f"{prod}-mut")
        os.makedirs(mut, exist_ok=True)
        for name in os.listdir(fdir):
            src = os.path.join(fdir, name)
            if os.path.isfile(src):
                shutil.copyfile(src, os.path.join(mut, name))
        os.makedirs(os.path.join(mut, "assets"), exist_ok=True)
        for name in os.listdir(os.path.join(fdir, "assets")):
            shutil.copyfile(
                os.path.join(fdir, "assets", name), os.path.join(mut, "assets", name)
            )
        # mutate one semantic literal: the generated world must change
        # (comment-only edits are correctly dropped by the generator).
        mutations = {
            "atlas-g09": ("atlas_sprite.at", ".phase > 5", ".phase > 4"),
            "character-g09": ("player.at", ".vy = 5.0", ".vy = 6.0"),
        }
        module_name, old_frag, new_frag = mutations[prod]
        module = os.path.join(mut, module_name)
        text = read(module)
        assert old_frag in text, f"{prod}: mutation anchor missing"
        text = text.replace(old_frag, new_frag, 1)
        with open(module, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        r = lower(os.path.join(mut, "main.at"), os.path.join(workdir, f"{prod}-mut-out"))
        assert r.returncode == 0, f"{prod}: mutant regen failed: {r.stderr}"
        frozen_world_sha = jload(os.path.join(pdir, "identity.json"))["user_source_sha256"]
        mutant_world_sha = jload(
            os.path.join(workdir, f"{prod}-mut-out", "identity.json")
        )["user_source_sha256"]
        assert mutant_world_sha != frozen_world_sha, (
            f"{prod}: mutated module must change the world identity"
        )
        results[f"{prod}/mutation_sensitivity"] = "source byte change -> new world sha"


def control_negatives(results):
    for prod, needle in NEGATIVES:
        entry = os.path.join(FIXTURES, prod, "main.at")
        r = lower(entry, os.path.join(tempfile.gettempdir(), "plan022-neg"))
        assert r.returncode != 0, f"{prod}: negative fixture must be rejected"
        combined = (r.stderr or "") + (r.stdout or "")
        assert needle in combined, (
            f"{prod}: rejection '{combined.strip()}' does not name '{needle}'"
        )
        results[f"{prod}/negative"] = needle


def control_provenance(results):
    for name, expect_sha in ORIGIN_LICENSES.items():
        path = os.path.join(FIXTURES, "licenses", name)
        assert os.path.isfile(path), f"license file missing: {name}"
        got = sha256_file(path)
        assert got == expect_sha, f"license {name} drifted: {got} != {expect_sha}"
        results[f"license/{name[:12]}"] = expect_sha
    for name, expect_sha in ORIGIN_MANIFESTS.items():
        path = os.path.join(FIXTURES, "manifests", name)
        assert os.path.isfile(path), f"manifest missing: {name}"
        got = sha256_file(path)
        assert got == expect_sha, f"manifest {name} drifted: {got} != {expect_sha}"
        results[f"manifest/{name[:12]}"] = expect_sha
    for name, size in ORIGIN_PNGS.items():
        path = os.path.join(FIXTURES, "assets", name)
        assert os.path.isfile(path) and os.path.getsize(path) == size, (
            f"origin png {name} must stay {size} bytes"
        )
        results[f"png/{name[:12]}"] = f"{size} bytes, sha pinned by name"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if not args.controls:
        print(__doc__)
        return 2

    results = {}
    with tempfile.TemporaryDirectory(prefix="plan022-verify-") as workdir:
        control_identity(results)
        control_cold_regen(results, workdir)
        control_mutation_sensitivity(results, workdir)
        control_negatives(results)
        control_provenance(results)
    print("PASS verify_plan022 --controls:", json.dumps(results, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
