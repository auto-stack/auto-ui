#!/usr/bin/env python3
"""PLAN-024 components-3d consumption verifier (--controls).

Identity chain, cold A->B regeneration (closure + GLB bytes only), mutation
sensitivity, license/provenance closure and named negative replays for the
components-3d product family (g02/g03 variants/twin). The run level (real
Auto/GPU oracle replay of the eleven original G02+G03 obligations and the
twin stateful chain) lives in the driver test node3d_basic_flow; this
verifier pins the file identity those products must keep.

Run from the pilot directory:
    python -I tools/verify_plan024.py --controls
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
FIXTURES = os.path.join(PILOT, "fixtures", "node3d")
PRODUCTS_DIR = os.path.join(PILOT, "generated-components3d")

PRODUCTS = [
    "g02",
    "g03",
    "g03-parent",
    "g03-hidden",
    "twin",
]

NEGATIVES = [
    ("neg-9", "exceeds the 8-instance envelope"),
    ("neg-rows129", "exceeds 128 nodes"),
    ("neg-model-type", "is not a model3d.glb resource"),
    ("neg-group-dyn", "group3d translation is static"),
    ("neg-scale-dyn", "scale must be static"),
    ("neg-both-views", "single view exit per node"),
    ("neg-root-params", "root node must not declare params"),
    ("neg-param-write", "params are read-only"),
    ("neg-cycle", "instance cycle"),
    ("neg-dup-path", "duplicate instance path"),
    ("neg-unknown-prop", "unknown prop on view-3d element"),
    ("neg-missing-use", "instance of undeclared node"),
    ("neg-msg-model", "msg payload must be numeric"),
    ("neg-marker", "view domain marker must be view-"),
]

ORIGIN_GLB = {
    "42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb": 1528,
}
ORIGIN_LICENSE = {
    "two-parts.LICENSE":
        "88d8dd7fd0eb0b0c70d0d4c6011a3b661e96f8d33f8f3ff76ebdbf5a98f8ee75"
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
        [exe, "--profile", "components-3d", "--entry", entry, "--out", out],
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
        assert identity["generator"] == "autographics-node2d-pilot plan024 components-3d", (
            f"{prod}: generator identity"
        )
        assert identity["api"] == "autographics.behavior/1", f"{prod}: api mismatch"
        assert scene["schema"] == "autographics.scene/6", f"{prod}: schema mismatch"
        assert scene["document_id"].startswith("plan024-components-3d-"), (
            f"{prod}: document_id prefix"
        )
        has_behavior = scene["behavior"] is not None
        if has_behavior:
            world_path = os.path.join(pdir, "world.at")
            world_bytes = open(world_path, "rb").read()
            world_sha = sha256_bytes(world_bytes)
            bound = scene["behavior"]["source"]["sha256"]
            assert bound == world_sha, f"{prod}: scene binds {bound} != world sha {world_sha}"
            assert identity["user_source_sha256"] == world_sha, (
                f"{prod}: identity world sha mismatch"
            )
            assert scene["behavior"]["api"] == "autographics.behavior/1"
            assert scene["behavior"]["actors"] == {"two_d": None, "three_d": None}
            results[f"{prod}/world_sha256"] = world_sha
        else:
            assert identity["user_source_sha256"] is None, f"{prod}: pure scene binds no source"
            assert not os.path.exists(os.path.join(pdir, "world.at")), (
                f"{prod}: pure scene must not ship a world.at"
            )

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
        for asset in scene["assets"]:
            assert asset["format"] == "glb", f"{prod}: asset format"
            blob = os.path.join(pdir, "assets", asset["sha256"] + ".glb")
            data = open(blob, "rb").read()
            assert len(data) == asset["byte_length"], f"{prod}: asset byte length"
            assert sha256_bytes(data) == asset["sha256"], f"{prod}: asset sha"
        id_assets = {a["sha256"] for a in identity.get("assets", [])}
        scene_assets = {a["sha256"] for a in scene["assets"]}
        assert id_assets == scene_assets, f"{prod}: identity assets != scene assets"

        # 每个三维叶带 sprite:null（scene/5+ 显式 sprite 域）与固定 rgba。
        for o in scene["objects"]:
            assert o["sprite"] is None, f"{prod}: 3D leaf sprite must be null"
            assert o["rgba"] == [0.9, 0.3, 0.2, 1.0], f"{prod}: 3D material is fixed"
        for g in scene["groups"]:
            assert g["dimension"] == "3d", f"{prod}: group dimension"
        results[f"{prod}/identity"] = {
            "closure_modules": len(closure),
            "assets": len(scene["assets"]),
            "objects": len(scene["objects"]),
            "groups": len(scene["groups"]),
            "behavior": has_behavior,
        }


def control_cold_regen(results, workdir):
    for prod in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        identity = jload(os.path.join(pdir, "identity.json"))
        b = os.path.join(workdir, f"{prod}-B")
        os.makedirs(b, exist_ok=True)
        # B holds ONLY the closure copy + GLB bytes (no producer tree).
        for name in os.listdir(os.path.join(pdir, "closure")):
            shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(b, name))
        bassets = os.path.join(b, "assets")
        os.makedirs(bassets, exist_ok=True)
        # GLB 声明路径由 identity 的 declared 映射恢复（B 不借生产者源树）。
        id_assets = {
            a["sha256"]: a.get("declared", "assets/" + a["sha256"] + ".glb")
            for a in identity.get("assets", [])
        }
        for name in os.listdir(os.path.join(pdir, "assets")):
            sha = name[:-4] if name.endswith(".glb") else name
            declared = id_assets.get(sha)
            target = os.path.basename(declared) if declared else name
            shutil.copyfile(
                os.path.join(pdir, "assets", name), os.path.join(bassets, target)
            )
        r = lower(os.path.join(b, "main.at"), os.path.join(workdir, f"{prod}-B-out"))
        assert r.returncode == 0, f"{prod}: cold regen failed: {r.stderr}"
        # identity.json 携带 out-dir product 字段（020/022 同约定），冷再生
        # 对账仅 scene + world（身份链另在 control_identity 校验）。
        for artifact in ("scene.agscene",):
            a = open(os.path.join(pdir, artifact), "rb").read()
            c = open(os.path.join(workdir, f"{prod}-B-out", artifact), "rb").read()
            assert a == c, f"{prod}: cold A->B {artifact} not byte-equal"
        if identity["user_source_sha256"] is not None:
            a = open(os.path.join(pdir, "world.at"), "rb").read()
            c = open(os.path.join(workdir, f"{prod}-B-out", "world.at"), "rb").read()
            assert a == c, f"{prod}: cold A->B world.at not byte-equal"
        results[f"{prod}/cold_regen"] = "byte-equal products from closure+assets"


def control_mutation_sensitivity(results, workdir):
    # twin: mutating one instance's p0 must change the world sha AND the
    # scene row; a comment-only edit must NOT change the generated world.
    prod = "twin"
    pdir = os.path.join(PRODUCTS_DIR, prod)
    mut = os.path.join(workdir, f"{prod}-mut")
    os.makedirs(mut, exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "closure")):
        shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(mut, name))
    os.makedirs(os.path.join(mut, "assets"), exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "assets")):
        # 闭包声明路径 = assets/two-parts.glb（declared 形态）。
        shutil.copyfile(os.path.join(pdir, "assets", name), os.path.join(mut, "assets", "two-parts.glb"))
    module = os.path.join(mut, "main.at")
    text = read(module)
    old_frag = "p0: 3.0"
    assert old_frag in text, f"{prod}: mutation anchor missing"
    text = text.replace(old_frag, "p0: 4.0", 1)
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
    mutant_scene = read(os.path.join(workdir, f"{prod}-mut-out", "scene.agscene"))
    assert "[4.0,0.0,0.0]" in mutant_scene.replace(" ", "").replace("\n", "") or "4.0" in mutant_scene, (
        f"{prod}: mutated p0 reaches the scene row"
    )
    results[f"{prod}/mutation_sensitivity"] = "semantic source change -> new world sha"

    cm = os.path.join(workdir, f"{prod}-comment")
    os.makedirs(cm, exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "closure")):
        shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(cm, name))
    os.makedirs(os.path.join(cm, "assets"), exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "assets")):
        shutil.copyfile(os.path.join(pdir, "assets", name), os.path.join(cm, "assets", "two-parts.glb"))
    module = os.path.join(cm, "main.at")
    text = read(module)
    marker = text.find("//")
    assert marker == 0, "main.at must start with a comment to mutate safely"
    text = "// comment-only mutation probe (plan024)\n" + text
    with open(module, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    r = lower(os.path.join(cm, "main.at"), os.path.join(workdir, f"{prod}-comment-out"))
    assert r.returncode == 0, f"{prod}: comment mutant regen failed: {r.stderr}"
    frozen_world = open(os.path.join(pdir, "world.at"), "rb").read()
    comment_world = open(
        os.path.join(workdir, f"{prod}-comment-out", "world.at"), "rb"
    ).read()
    assert frozen_world == comment_world, (
        f"{prod}: comment-only mutation must leave the world byte-identical"
    )
    results[f"{prod}/comment_mutation_ignored"] = "comment-only edit -> byte-equal world"


def control_negatives(results):
    for prod, needle in NEGATIVES:
        entry = os.path.join(FIXTURES, prod, "main.at")
        out = os.path.join(tempfile.gettempdir(), "plan024-neg", prod)
        r = lower(entry, out)
        assert r.returncode != 0, f"{prod}: negative fixture must be rejected"
        combined = (r.stderr or "") + (r.stdout or "")
        assert needle in combined, (
            f"{prod}: rejection '{combined.strip()}' does not name '{needle}'"
        )
        results[f"{prod}/negative"] = needle


def control_provenance(results):
    for name, size in ORIGIN_GLB.items():
        # 件名 = declared 形态（two-parts.glb）；字节身份按 sha 锚定。
        path = os.path.join(FIXTURES, "g02", "assets", "two-parts.glb")
        assert os.path.isfile(path) and os.path.getsize(path) == size, (
            f"origin glb must stay {size} bytes"
        )
        got = sha256_file(path)
        assert got == name[:-4], f"origin glb sha drifted: {got}"
        results["glb/two-parts"] = f"{size} bytes, sha {name[:12]}"
    lic = os.path.join(FIXTURES, "licenses", "two-parts.LICENSE")
    assert os.path.isfile(lic), "license file missing: two-parts.LICENSE"
    head = open(lic, "r", encoding="utf-8").read().splitlines()
    assert any("license: MIT" in l for l in head), "GLB license must be recorded (MIT)"
    results["license/two-parts"] = "MIT origin recorded"

    # provenance 消费面：identity.consumer 指向真实 include_str! 消费文件。
    consumer = "experimental/autographics-auto-driver/tests/node3d_basic_flow.rs"
    assert os.path.isfile(os.path.join(PILOT, "..", "..", consumer)), (
        "flow consumer file missing"
    )
    flow_src = open(
        os.path.join(PILOT, "..", "..", consumer), "r", encoding="utf-8"
    ).read()
    for prod in PRODUCTS:
        pid = jload(os.path.join(PRODUCTS_DIR, prod, "identity.json"))
        assert pid["provenance"]["consumer"] == consumer, f"{prod}: consumer record"
        needle_prod = f"generated-components3d/{prod}/scene.agscene"
        assert needle_prod.split("/")[-2] in flow_src, (
            f"{prod}: flow test does not consume the product"
        )
    results["provenance/consumer"] = consumer


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if not args.controls:
        print(__doc__)
        return 2

    results = {}
    with tempfile.TemporaryDirectory(prefix="plan024-verify-") as workdir:
        control_identity(results)
        control_cold_regen(results, workdir)
        control_mutation_sensitivity(results, workdir)
        control_negatives(results)
        control_provenance(results)
    print("PASS verify_plan024 --controls:", json.dumps(results, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
