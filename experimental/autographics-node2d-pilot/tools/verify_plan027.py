#!/usr/bin/env python3
"""PLAN-027 components-3d-runtime consumption verifier (--controls).

Identity chain (generator/api/document_id/behavior-api selection), cold A->B
regeneration (closure only; runtime fixtures carry no GLB assets), mutation
sensitivity, built-in model origin/license closure and named negative replays
for the components-3d-runtime product family (g07a/g07a-fast/g07b/g08/
g08-alt/static-actors/events3d/wide128/deep127/two-moves/stale-token). The
run level (real Auto/GPU oracle replay of the original G07 two-layout 11 and
G08/3d 13 oracles, typed/budget faults 10/13/27/29/30 and the 127-solid
cumulative-guard refusal) lives in the driver test node3d_runtime_flow; this
verifier pins the file identity those products must keep.

Run from the pilot directory:
    python -I tools/verify_plan027.py --controls
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
REPO = os.path.dirname(os.path.dirname(PILOT))
FIXTURES = os.path.join(PILOT, "fixtures", "components3d-runtime")
PRODUCTS_DIR = os.path.join(PILOT, "generated-components3d-runtime")

PRODUCTS = [
    "g07a",
    "g07a-fast",
    "g07b",
    "g08",
    "g08-alt",
    "static-actors",
    "events3d",
    "wide128",
    "deep127",
    "two-moves",
    "stale-token",
]

CAMERA_PRODUCTS = {"g08", "g08-alt"}
BEHAVIOR_ONLY_PRODUCTS = {"events3d"}
CHARACTER_API = "autographics.character3d/1"
CAMERA_API = "autographics.runtime_camera3d/1"
BEHAVIOR_API = "autographics.behavior/1"

NEGATIVES = [
    ("neg-two-characters", "character3d must bind exactly one instance per Run"),
    ("neg-camera-without-character", "camera3d requires exactly one character3d instance"),
    ("neg-camera-non-root", "camera3d block is root-only"),
    ("neg-solid-dynamic", "model ref outside an instance context"),
    ("neg-solid-on-character", "character body 'body' must not also be marked solid"),
    ("neg-character-shape", "character body 'body' scale must be 0.4 x 0.5 x 0.4"),
    ("neg-character-dyn", "character body 'body' position must be static"),
    ("neg-solid-builtin", "'solid' is a cylinder3d prop, not player3d"),
    ("neg-camera-domain", "camera3d distance must be within [4, 12]"),
]

# 具名内置视觉的原始模型闭包（原 owner：driver 暖上传同字节；声明不携带
# 资源字节）。来源：experimental/autographics-prototype（README "Asset
# provenance & licenses"：assets/squash_the_creeps/** 冻结自
# godotengine/godot-demo-projects@b761b4cd，模型 MIT）。
BUILTIN_MODELS = {
    "player.glb": "05d8588e8032a674e309ccb4d04578fb578b8da1629096b1036f628593801942",
    "mob.glb": "8a82adb5bfba8c22f9c1a0d692e4d25c2f9055cb7e6f6acd6f61ac169079818c",
}
BUILTIN_SIZES = {"player.glb": 52392, "mob.glb": 63828}


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
        [exe, "--profile", "components-3d-runtime", "--entry", entry, "--out", out],
        capture_output=True, text=True,
    )


def read(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def jload(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def expected_api(prod):
    if prod in CAMERA_PRODUCTS:
        return CAMERA_API
    if prod in BEHAVIOR_ONLY_PRODUCTS:
        return BEHAVIOR_API
    return CHARACTER_API


def control_identity(results):
    for prod in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        scene = jload(os.path.join(pdir, "scene.agscene"))
        identity = jload(os.path.join(pdir, "identity.json"))
        assert identity["generator"] == (
            "autographics-node2d-pilot plan027 components-3d-runtime"
        ), f"{prod}: generator identity"
        assert scene["schema"] == "autographics.scene/6", f"{prod}: schema mismatch"
        assert scene["document_id"].startswith("plan027-components-3d-runtime-"), (
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
            assert scene["behavior"]["api"] == expected_api(prod), (
                f"{prod}: behavior api must match the declared surface"
            )
            assert scene["behavior"]["actors"] == {"two_d": None, "three_d": None}
            # 配置注入序：bind -> solids -> camera（若声明）。
            world = world_bytes.decode("utf-8")
            lines = [l.strip() for l in world.splitlines() if "ag_char3_" in l or "ag_camera_orbit_3d" in l]
            binds = [i for i, l in enumerate(lines) if l.startswith("ok=ok+ag_char3_bind(")]
            solids = [i for i, l in enumerate(lines) if l.startswith("ok=ok+ag_char3_solid(")]
            cams = [i for i, l in enumerate(lines) if l.startswith("ok=ok+ag_camera_orbit_3d(")]
            assert len(binds) == (0 if prod in BEHAVIOR_ONLY_PRODUCTS else 1), (
                f"{prod}: bind count"
            )
            if cams:
                assert cams[0] > binds[0], f"{prod}: camera config follows the bind"
            if solids:
                assert min(solids) > binds[0], f"{prod}: solids follow the bind"
            results[f"{prod}/config_lines"] = {"bind": len(binds), "solids": len(solids), "camera": len(cams)}
        else:
            assert identity["user_source_sha256"] is None, f"{prod}: pure scene binds no source"
            assert not os.path.exists(os.path.join(pdir, "world.at")), (
                f"{prod}: pure scene must not ship a world.at"
            )
        # 静态场景：内置视觉叶不携带资源字节（原 owner 暖上传）。
        if prod == "static-actors":
            assert scene["assets"] == [], "built-in visuals carry no scene assets"
            kinds = {o["kind"] for o in scene["objects"]}
            assert kinds == {"player", "mob", "cylinder"}, f"static kinds {kinds}"

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

        # assets: scene entries == bytes on disk（runtime 夹具无 GLB 资源）。
        for asset in scene["assets"]:
            blob = os.path.join(pdir, "assets", asset["sha256"] + ".glb")
            data = open(blob, "rb").read()
            assert len(data) == asset["byte_length"], f"{prod}: asset byte length"
            assert sha256_bytes(data) == asset["sha256"], f"{prod}: asset sha"
        id_assets = {a["sha256"] for a in identity.get("assets", [])}
        scene_assets = {a["sha256"] for a in scene["assets"]}
        assert id_assets == scene_assets, f"{prod}: identity assets != scene assets"

        for o in scene["objects"]:
            assert o["sprite"] is None, f"{prod}: 3D leaf sprite must be null"
            assert o["rgba"] == [0.9, 0.3, 0.2, 1.0], f"{prod}: 3D material is fixed"
        for g in scene["groups"]:
            assert g["dimension"] == "3d", f"{prod}: group dimension"
        results[f"{prod}/identity"] = {
            "closure_modules": len(closure),
            "assets": len(scene["assets"]),
            "objects": len(scene["objects"]),
            "behavior": has_behavior,
            "api": scene["behavior"]["api"] if has_behavior else None,
        }


def control_cold_regen(results, workdir):
    for prod in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        identity = jload(os.path.join(pdir, "identity.json"))
        b = os.path.join(workdir, f"{prod}-B")
        os.makedirs(b, exist_ok=True)
        # B holds ONLY the closure copy (runtime fixtures declare no GLB
        # resources; a scene that did would restore assets via identity
        # declared paths as in verify_plan024).
        for name in os.listdir(os.path.join(pdir, "closure")):
            shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(b, name))
        r = lower(os.path.join(b, "main.at"), os.path.join(workdir, f"{prod}-B-out"))
        assert r.returncode == 0, f"{prod}: cold regen failed: {r.stderr}"
        for artifact in ("scene.agscene",):
            a = open(os.path.join(pdir, artifact), "rb").read()
            c = open(os.path.join(workdir, f"{prod}-B-out", artifact), "rb").read()
            assert a == c, f"{prod}: cold A->B {artifact} not byte-equal"
        if identity["user_source_sha256"] is not None:
            a = open(os.path.join(pdir, "world.at"), "rb").read()
            c = open(os.path.join(workdir, f"{prod}-B-out", "world.at"), "rb").read()
            assert a == c, f"{prod}: cold A->B world.at not byte-equal"
        results[f"{prod}/cold_regen"] = "byte-equal products from closure"


def control_mutation_sensitivity(results, workdir):
    # g07a: the speed constant 4.0 -> 5.0 must change the world sha AND reach
    # the scene row; a comment-only edit must NOT change the generated world.
    prod = "g07a"
    pdir = os.path.join(PRODUCTS_DIR, prod)
    mut = os.path.join(workdir, f"{prod}-mut")
    os.makedirs(mut, exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "closure")):
        shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(mut, name))
    module = os.path.join(mut, "character.at")
    text = read(module)
    old_frag = "var speed = 4.0"
    assert old_frag in text, f"{prod}: mutation anchor missing"
    text = text.replace(old_frag, "var speed = 5.0", 1)
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
    mutant_world = read(os.path.join(workdir, f"{prod}-mut-out", "world.at"))
    assert "var character_speed=[5.0]" in mutant_world, (
        f"{prod}: mutated speed reaches the generated SoA row"
    )
    results[f"{prod}/mutation_sensitivity"] = "semantic source change -> new world sha"

    cm = os.path.join(workdir, f"{prod}-comment")
    os.makedirs(cm, exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "closure")):
        shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(cm, name))
    module = os.path.join(cm, "main.at")
    text = read(module)
    assert text.startswith("//"), "main.at must start with a comment to mutate safely"
    text = "// comment-only mutation probe (plan027)\n" + text
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
        out = os.path.join(tempfile.gettempdir(), "plan027-neg", prod)
        r = lower(entry, out)
        assert r.returncode != 0, f"{prod}: negative fixture must be rejected"
        combined = (r.stderr or "") + (r.stdout or "")
        assert needle in combined, (
            f"{prod}: rejection '{combined.strip()}' does not name '{needle}'"
        )
        assert "symbol '" in combined or ".at" in combined, (
            f"{prod}: rejection lacks a module/line/symbol anchor"
        )
        results[f"{prod}/negative"] = needle


def control_provenance(results):
    # 具名内置视觉闭包：原 GLB 字节身份 + 原始来源/许可记录（README 行）。
    proto_assets = os.path.join(
        REPO, "experimental", "autographics-prototype",
        "assets", "squash_the_creeps", "art",
    )
    for name, sha in BUILTIN_MODELS.items():
        path = os.path.join(proto_assets, name)
        assert os.path.isfile(path), f"built-in origin missing: {name}"
        assert os.path.getsize(path) == BUILTIN_SIZES[name], f"{name}: size drift"
        got = sha256_file(path)
        assert got == sha, f"{name}: sha drifted: {got}"
        results[f"builtin/{name}"] = f"{BUILTIN_SIZES[name]} bytes, sha {sha[:12]}"
    readme = read(os.path.join(REPO, "experimental", "autographics-prototype", "README.md"))
    assert "godot-demo-projects@b761b4cd" in readme and "MIT" in readme, (
        "built-in model origin/license must stay recorded in the prototype README"
    )
    results["builtin/origin"] = "godot-demo-projects@b761b4cd, MIT (prototype README row)"

    # provenance 消费面：identity.consumer 指向真实 include_str! 消费文件。
    consumer = "experimental/autographics-auto-driver/tests/node3d_runtime_flow.rs"
    flow_path = os.path.join(REPO, consumer)
    assert os.path.isfile(flow_path), "flow consumer file missing"
    flow_src = open(flow_path, "r", encoding="utf-8").read()
    gates_path = os.path.join(PILOT, "tests", "node3d_runtime_gates.rs")
    gates_src = open(gates_path, "r", encoding="utf-8").read()
    flow_products = [
        p for p in PRODUCTS
        if p not in ("deep127", "two-moves", "stale-token", "events3d")
    ]
    for prod in PRODUCTS:
        pid = jload(os.path.join(PRODUCTS_DIR, prod, "identity.json"))
        assert pid["provenance"]["consumer"] == consumer, f"{prod}: consumer record"
        if prod in flow_products:
            assert f"generated-components3d-runtime/{prod}/" in flow_src, (
                f"{prod}: flow test does not consume the product"
            )
    for prod in ("deep127", "two-moves"):
        assert prod in flow_src, f"{prod}: flow test consumes the product"
        assert prod in gates_src, f"{prod}: gates test consumes the product"
    # stale-token：flow 消费（运行时 fault 13 反例）；编译面经 two-moves 同
    # 构造族覆盖，无独立 gates 用例。
    assert "stale-token" in flow_src, "stale-token: flow test consumes the product"
    # events3d：仅编译面消费（gates 接收器隔离形状）；无 Run 消费。
    assert "events3d" in gates_src, "events3d: gates test consumes the product"
    results["provenance/consumer"] = consumer


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if not args.controls:
        print(__doc__)
        return 2

    results = {}
    with tempfile.TemporaryDirectory(prefix="plan027-verify-") as workdir:
        control_identity(results)
        control_cold_regen(results, workdir)
        control_mutation_sensitivity(results, workdir)
        control_negatives(results)
        control_provenance(results)
    print("PASS verify_plan027 --controls:", json.dumps(results, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
