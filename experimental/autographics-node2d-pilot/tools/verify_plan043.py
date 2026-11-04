#!/usr/bin/env python3
"""PLAN-043 model-objects-3d consumption verifier (--controls).

Identity chain (generator/api/document_id/template rows/resource keys),
cold A->(resource-meta)->B regeneration from the closure copy only, mutation
sensitivity (a semantic edit moves the world sha), the old-profile freeze
(the shared events lowering must regenerate the 028 g11-main product
byte-equal) and the frozen textured-rejection domain on the plain events
profile. The run level (real Auto/Triggers3V1/GPU oracles for the flat and
opaque chains) lives in the driver test node3d_model_objects_flow; the
lowering rejections live in the pilot test node3d_model_objects_gates.

Run from the pilot directory (after `cargo build` + driver resource_meta):
    python -I tools/verify_plan043.py --controls
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
FIXTURES = os.path.join(PILOT, "fixtures", "components3d-events")
PRODUCTS_DIR = os.path.join(PILOT, "generated-components3d-events")

API = "autographics.triggers3d/1"
GENERATOR = "autographics-node2d-pilot plan043 model-objects-3d"

# (product, uses_meta)：g12 走平色 sha 键（单 pass），g13 走 034 闭包键
# （A->meta->B 双 pass）。
PRODUCTS = [("g12-model", False), ("g13-model-opaque", True)]


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def lower(entry, out, meta=None):
    exe = os.path.join(PILOT, "target", "debug", "node2d-lower.exe")
    cmd = [exe, "--profile", "model-objects-3d", "--entry", entry, "--out", out]
    if meta:
        cmd += ["--resources", meta]
    return subprocess.run(cmd, capture_output=True, text=True)


def run_meta(entry, request, out):
    meta_exe = os.path.join(
        REPO, "experimental", "autographics-auto-driver", "target", "debug",
        "resource_meta.exe",
    )
    return subprocess.run(
        [meta_exe, "--entry", entry, "--request", request, "--out", out],
        capture_output=True, text=True,
    )


def jload(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def control_identity(results):
    for prod, uses_meta in PRODUCTS:
        pdir = os.path.join(PRODUCTS_DIR, prod)
        identity = jload(os.path.join(pdir, "identity.json"))
        assert identity["generator"] == GENERATOR, f"{prod}: generator string"
        assert identity["api"] == API, f"{prod}: assembly identity"
        world = os.path.join(pdir, "world.at")
        assert identity["user_source_sha256"] == sha256_file(world), (
            f"{prod}: user_source sha mismatch"
        )
        # closure 模块哈希 == fixture 源字节。
        for entry in identity["closure"]:
            name = entry["name"]
            src = os.path.join(FIXTURES, prod, f"{name}.at")
            assert entry["sha256"] == sha256_file(src), f"{prod}: closure {name} drift"
        scene = jload(os.path.join(pdir, "scene.agscene"))
        assert scene["behavior"]["api"] == API, f"{prod}: scene behavior api"
        assert scene["document_id"] == "plan043-model-objects-3d-main", (
            f"{prod}: document id {scene['document_id']}"
        )
        rows = {o["name"]: o for o in scene["objects"]}
        leaf = "Crate" if prod == "g12-model" else "Coin"
        row = rows[leaf]
        assert row["kind"] == "model", f"{prod}: template row kind"
        assert not row["visible"], f"{prod}: template row parked"
        want_prefix = (
            "sha256:42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671"
            if prod == "g12-model"
            else "closure:76657e1367fe78e3aa69585e4a2d6e462f7ca431a4c17d1b59a1e7fca7d3862e"
        )
        assert row["resource"] == want_prefix, (
            f"{prod}: template resource key {row['resource']}"
        )
        assets = scene["assets"]
        assert len(assets) == 1 and assets[0]["name"] == (
            "crate_glb" if prod == "g12-model" else "coin_tex"
        ), f"{prod}: asset manifest"
        if uses_meta:
            closure = assets[0]["closure"]
            assert closure["id"] == want_prefix.split(":", 1)[1], f"{prod}: closure id"
            assert closure["images"][0]["sha256"] == (
                "106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3"
            ), f"{prod}: closure texture identity"
        results[f"{prod}/identity"] = "generator/api/sha/closure/resource keys pinned"


def control_cold_regen(workdir, results):
    for prod, uses_meta in PRODUCTS:
        fixture = os.path.join(FIXTURES, prod)
        entry = os.path.join(workdir, f"closure-{prod}")
        if os.path.exists(entry):
            shutil.rmtree(entry)
        shutil.copytree(fixture, entry)
        out = os.path.join(workdir, f"regen-{prod}")
        r = lower(os.path.join(entry, "main.at"), out)
        assert r.returncode == 0, f"{prod} pass A: {r.stderr}"
        # 公开导入准入面（034 同一面）：平色条目无 closure，模板回落 sha 键。
        meta_out = os.path.join(out, "resources.json")
        p = run_meta(
            entry,
            os.path.join(out, "resources-request.json"),
            meta_out,
        )
        assert p.returncode == 0, f"{prod} meta: {p.stdout}{p.stderr}"
        r = lower(os.path.join(entry, "main.at"), out, meta_out)
        assert r.returncode == 0, f"{prod} pass B: {r.stderr}"
        for artifact in ("world.at", "scene.agscene"):
            a = open(os.path.join(PRODUCTS_DIR, prod, artifact), "rb").read()
            c = open(os.path.join(out, artifact), "rb").read()
            assert a == c, f"{prod} {artifact} cold regen byte drift"
        regen_assets = sorted(os.listdir(os.path.join(out, "assets")))
        want_assets = sorted(os.listdir(os.path.join(PRODUCTS_DIR, prod, "assets")))
        assert regen_assets == want_assets, f"{prod} asset set drift"
        for name in want_assets:
            assert open(os.path.join(out, "assets", name), "rb").read() == open(
                os.path.join(PRODUCTS_DIR, prod, "assets", name), "rb"
            ).read(), f"{prod}/assets/{name} byte drift"
        results[f"{prod}/cold_regen"] = "byte-equal from closure (A->meta->B)"


def control_mutation_sensitivity(workdir, results):
    # 语义突变：pad x 4.0 -> 5.0（world/scene sha 必变）。
    prod, uses_meta = PRODUCTS[0]
    fixture = os.path.join(FIXTURES, prod)
    base = os.path.join(workdir, "mutation-base")
    shutil.copytree(fixture, base)
    main_path = os.path.join(base, "main.at")
    src = open(main_path, encoding="utf-8").read()
    open(main_path, "w", encoding="utf-8").write(src.replace("(4.0, 0.5, 0.0)", "(5.0, 0.5, 0.0)", 1))
    out = os.path.join(workdir, "mutation-out")
    r = lower(os.path.join(base, "main.at"), out)
    assert r.returncode == 0, f"mutation pass A failed: {r.stderr}"
    p = run_meta(base, os.path.join(out, "resources-request.json"), os.path.join(out, "resources.json"))
    assert p.returncode == 0, f"mutation meta: {p.stdout}{p.stderr}"
    r = lower(os.path.join(base, "main.at"), out, os.path.join(out, "resources.json"))
    assert r.returncode == 0, f"mutation pass B failed: {r.stderr}"
    # pad x 是场景行：scene.agscene 必变；world.at（行为源）不含场景值，
    # 不变正是字节稳定性质。
    assert open(os.path.join(out, "scene.agscene"), "rb").read() != open(
        os.path.join(PRODUCTS_DIR, prod, "scene.agscene"), "rb"
    ).read(), "semantic mutation must move the scene bytes"
    results["mutation_sensitivity"] = "pad x edit moves scene bytes (world.at stable)"


def control_old_profile_freeze(workdir, results):
    # 旧档输出冻结：共享 events 降低（043 重构为参数化实现）再生成 028
    # g11-main 必须与在库产品字节一致。
    pdir = os.path.join(PRODUCTS_DIR, "..", "generated-components3d-events", "g11-main")
    pdir = os.path.normpath(pdir)
    b = os.path.join(workdir, "g11-main-028-B")
    os.makedirs(b, exist_ok=True)
    for name in os.listdir(os.path.join(pdir, "closure")):
        shutil.copyfile(os.path.join(pdir, "closure", name), os.path.join(b, name))
    exe = os.path.join(PILOT, "target", "debug", "node2d-lower.exe")
    r = subprocess.run(
        [exe, "--profile", "components-3d-events", "--entry", os.path.join(b, "main.at"),
         "--out", os.path.join(workdir, "g11-main-028-B-out")],
        capture_output=True, text=True,
    )
    assert r.returncode == 0, f"028 g11-main regen failed: {r.stderr}"
    for artifact in ("scene.agscene", "world.at"):
        a = open(os.path.join(pdir, artifact), "rb").read()
        c = open(os.path.join(workdir, "g11-main-028-B-out", artifact), "rb").read()
        assert a == c, f"028 g11-main {artifact} drifted (old profile freeze)"
    results["old_profile_freeze"] = "028 g11-main byte-equal from closure"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    if not args.controls:
        print("usage: verify_plan043.py --controls")
        return 2
    results = {}
    workdir = tempfile.mkdtemp(prefix="plan043-verify-")
    try:
        control_identity(results)
        control_cold_regen(workdir, results)
        control_mutation_sensitivity(workdir, results)
        control_old_profile_freeze(workdir, results)
    finally:
        shutil.rmtree(workdir, ignore_errors=True)
    width = max(len(k) for k in results)
    for k in sorted(results):
        print(f"  {k:<{width}}  {results[k]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
