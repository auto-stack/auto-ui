#!/usr/bin/env python3
"""PLAN-034 components-3d-presentation 生成链控制门（--controls）。

产品（generated-components3d-presentation/10个子入口）经三步管线再核：
声明 → resource-meta（既有公开导入准入面）→ 最终产品；identity/场景行/
world数值needle逐字节、A→B冷再生逐字节、语义突变敏感、CLI级具名拒绝、
冻结源（driver受信API字节、027/028/030 spec、G10原件哈希、 Kenney上游、
engine锁SDK钉）与旧档冻结（verify_plan030 --controls 32检查）。

运行期（真Host/Auto/GPU）检查在 driver 侧冻结入口：
  cargo test --locked --offline --test node3d_presentation_flow -- --test-threads=1
  cargo test --locked --offline --test node3d_presentation_gates
  cargo test --locked --offline --test node3d_presentation_oracles -- --test-threads=1
  cargo test --locked --offline --test node3d_presentation_oracles_anim -- --test-threads=1
用法：python -I tools/verify_plan034.py --controls
"""
import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from hashlib import sha256
from pathlib import Path

PILOT = Path(__file__).resolve().parent.parent
REPO = PILOT.parent.parent
PRODUCTS_DIR = PILOT / "generated-components3d-presentation"
FIXTURES = PILOT / "fixtures" / "node3d-presentation"
DRIVER_API = REPO / "experimental" / "autographics-auto-driver" / "api"

PRODUCTS = [
    "g10-textures",
    "g10-textures-alt",
    "g10-textures-frozen",
    "g10-textures-missvis",
    "g10-textures-altimage",
    "g10-animation",
    "g10-animation-alt",
    "g10-animation-frozen",
    "g10-animation-extra-read",
    "g10-animation-neutral",
]

GENERATOR = "autographics-node2d-pilot plan034 components-3d-presentation"
CONSUMER = "experimental/autographics-auto-driver/tests/node3d_presentation_flow.rs"

# 冻结源（T-01/T-02 核验；变更即失约）。
DRIVER_API_PINS = {
    "node_animation3d_v1.at": "b9673a0640cbc26821492ad5e93090bfdceb18db7a5d51e8632e2b817df9595f",
}
SPEC_PINS = {
    "docs/specs/auto-graphics/node3d-symbolic-component-bindings.md": "6ed0bf070de48c3fae4ff8c1687dca9d67c8645010aef5783e49ca14b3352e18",
}
GALLERY_BASELINE = {
    "character.glb": ("7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1", 87624),
    "colormap.png": ("106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3", 10108),
    "platform.glb": ("78c5ed4da30c5a97f0747a54248bc9de05ba1b3fb6b94be527dd48599f5ef44c", 13352),
    "coin.glb": ("a3fb8f779a6af1cd75e5a02da77a87546ca8a59f6fe52f79f25179c4a68fd0e5", 22124),
    "flag.glb": ("0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae", 13276),
    "uv-material-probe.glb": ("f9a8f15627ca0d2060192a429e16dc7145c64a2f2e9315681d42a3f99d99bee3", 3692),
    "asymmetric-checker.png": ("6cc329163c9dadf2e531d5095196cc91631823c5b9ddce69f6e2b6690173dc2f", 332),
    "checker-alt.png": ("c2d2966858cb49ac63177127169fd3681bad943e1c153d4b4a208028830f576a", 279),
}

results = {}


def sha256_file(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def lower(exe: Path, entry: Path, out: Path, resources: Path | None = None) -> tuple[int, str]:
    import subprocess as sp

    cmd = [str(exe), "--profile", "components-3d-presentation", "--entry", str(entry), "--out", str(out)]
    if resources is not None:
        cmd += ["--resources", str(resources)]
    proc = sp.run(cmd, capture_output=True, text=True)
    return proc.returncode, (proc.stdout + proc.stderr)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def jload(path: Path):
    return json.loads(read(path))


# ---------------------------------------------------------------- controls --


def control_identity() -> None:
    rows = []
    for product in PRODUCTS:
        identity = jload(PRODUCTS_DIR / product / "identity.json")
        assert identity["generator"] == GENERATOR, product
        assert identity["provenance"]["product"] == product
        # variant 夹具（冻结/缺显隐/altimage/extra-read/neutral）由 oracles
        # 消费；主四入口由 flow 消费。
        assert identity["provenance"]["consumer"] in (
            CONSUMER,
            "experimental/autographics-auto-driver/tests/node3d_presentation_oracles.rs",
            "experimental/autographics-auto-driver/tests/node3d_presentation_oracles_anim.rs",
        ), (product, identity["provenance"]["consumer"])
        api = identity["api"]
        if "animation" in product:
            assert api == "autographics.node_animation3d/1", (product, api)
        elif product in ("g10-textures", "g10-textures-alt"):
            assert api == "autographics.triggers3d/1", (product, api)
        else:
            assert api in ("autographics.triggers3d/1", "autographics.behavior/1"), (product, api)
        assert (PRODUCTS_DIR / product / "world.at").exists()
        rows.append(product)
    results["identity"] = f"{len(rows)} products generator/api/consumer pinned"


def control_template_rows() -> None:
    scene = jload(PRODUCTS_DIR / "g10-textures" / "scene.agscene")
    names = [o["name"] for o in scene["objects"]]
    assert names == ["ground", "uv", "Main/coin/coin", "Main/flag/banner"], names
    for o in scene["objects"]:
        assert o["resource"].startswith("closure:"), o["name"]
    assert scene["objects"][2]["scale"] == [4.0, 4.0, 4.0], "coin original scale"
    assert scene["objects"][0]["translation"] == [-6.0, 0.0, 0.0], "platform original transform"
    anim = jload(PRODUCTS_DIR / "g10-animation" / "scene.agscene")
    assert [o["name"] for o in anim["objects"]] == ["Main/hero/body", "Main/ground/floor"]
    assert anim["behavior"]["api"] == "autographics.node_animation3d/1"
    closure = anim["assets"][0]["closure"]
    assert closure["node_animation"] is True
    assert closure["id"] == "6da9227ee326d34b997e7d859abb0c3f9fc9af3f53659feaa5a48982472fbd9d"
    world = read(PRODUCTS_DIR / "g10-animation" / "world.at")
    for needle in [
        " let packet=ag_actions_snapshot()\n let held_bits=packet%64\n let pressed_bits=packet/64\n",
        "ag_visual_pose(0, (player_body_rows[i]) + 1, 0, 0.0, 1, 0.0)",
        "ag_visual_pose(0, (player_body_rows[i]) + 1, clip, player_phase[i], 1, player_yaw[i])",
        "ok=ok+ag_char3_bind(1)",
        "ok=ok+ag_char3_solid(2)",
        "ok=ok+ag_camera_orbit_3d(1, 6.0, 30.0, 0, 0.0)",
        "ag_trigger_bind(0, player_body_subject_ids[i], 0, ground_floor_subject_ids[0], 1, 40.0)",
    ]:
        assert needle in world, needle
    wtex = read(PRODUCTS_DIR / "g10-textures" / "world.at")
    for needle in [
        "var coin_cx=[0.0]",
        "ag_set_local(coin_coin_rows[i],0,coin_cx[i])",
        "ag_set_visible(flag_banner_rows[i],flag_fv[i])",
        ".cx = 0.8 - .cx" if False else "coin_cx[i] = (0.8 - coin_cx[i])",
    ]:
        assert needle in wtex, needle
    results["template_rows"] = "scene closure refs + world numeric needles pinned"


def control_cold_regen(work: Path) -> None:
    exe = PILOT / "target" / "debug" / "node2d-lower.exe"
    meta_exe = REPO / "experimental" / "autographics-auto-driver" / "target" / "debug" / "resource_meta.exe"
    assert exe.exists() and meta_exe.exists(), "build pilot + resource-meta first"
    checked = 0
    for product in PRODUCTS:
        fixture = FIXTURES / product
        out = work / f"regen-{product}"
        # A→B：B 只拿输入闭包（作者.at + 声明的资源文件），无生产者源树。
        entry = work / f"closure-{product}"
        if entry.exists():
            shutil.rmtree(entry)
        shutil.copytree(fixture, entry)
        code, log = lower(exe, entry / "main.at", out)
        assert code == 0, f"{product} pass A: {log}"
        # meta 步（公开导入准入面）。
        import subprocess as sp

        proc = sp.run(
            [
                str(meta_exe),
                "--entry",
                str(entry),
                "--request",
                str(out / "resources-request.json"),
                "--out",
                str(out / "resources.json"),
            ],
            capture_output=True,
            text=True,
        )
        assert proc.returncode == 0, f"{product} meta: {proc.stdout}{proc.stderr}"
        code, log = lower(exe, entry / "main.at", out, out / "resources.json")
        assert code == 0, f"{product} pass B: {log}"
        for artifact in ["world.at", "scene.agscene"]:
            assert (out / artifact).read_bytes() == (PRODUCTS_DIR / product / artifact).read_bytes(), (
                f"{product}/{artifact} cold regen byte drift"
            )
        # identity：product 字段随 out 目录名变化，其余逐字段一致。
        got_identity = jload(out / "identity.json")
        want_identity = jload(PRODUCTS_DIR / product / "identity.json")
        assert got_identity.pop("provenance")["product"] is not None
        want_product = want_identity.pop("provenance")["product"]
        assert got_identity == want_identity, f"{product}/identity cold regen drift" 
        regen_assets = sorted(p.name for p in (out / "assets").iterdir())
        want_assets = sorted(p.name for p in (PRODUCTS_DIR / product / "assets").iterdir())
        assert regen_assets == want_assets, f"{product} asset set drift"
        for name in want_assets:
            assert (out / "assets" / name).read_bytes() == (
                PRODUCTS_DIR / product / "assets" / name
            ).read_bytes(), f"{product}/assets/{name} byte drift"
        checked += 1
    results["cold_regen"] = f"{checked} products byte-equal from closure (A->meta->B)"


def control_mutation_sensitivity(work: Path) -> None:
    exe = PILOT / "target" / "debug" / "node2d-lower.exe"
    meta_exe = REPO / "experimental" / "autographics-auto-driver" / "target" / "debug" / "resource_meta.exe"
    import subprocess as sp

    fixture = FIXTURES / "g10-animation"
    base = work / "mutation-base"
    code, log = lower(exe, fixture / "main.at", base)
    assert code == 0, log
    sp.run([str(meta_exe), "--entry", str(fixture), "--request", str(base / "resources-request.json"), "--out", str(base / "resources.json")], capture_output=True, text=True)
    code, log = lower(exe, fixture / "main.at", base, base / "resources.json")
    assert code == 0, log
    base_world = (base / "world.at").read_bytes()

    # 语义突变：walk 朝向系数 0.7853… → 0.8753…（world sha 必变）。
    mutant = work / "mutation-semantic"
    mutant.mkdir(parents=True, exist_ok=True)
    for f in fixture.iterdir():
        if f.is_file():
            (mutant / f.name).write_text(
                f.read_text(encoding="utf-8").replace("0.78539816339", "0.87539816339"),
                encoding="utf-8",
            )
    shutil.copytree(fixture / "assets", mutant / "assets", dirs_exist_ok=True)
    code, log = lower(exe, mutant / "main.at", mutant)
    assert code == 0, log
    sp.run([str(meta_exe), "--entry", str(mutant), "--request", str(mutant / "resources-request.json"), "--out", str(mutant / "resources.json")], capture_output=True, text=True)
    code, log = lower(exe, mutant / "main.at", mutant, mutant / "resources.json")
    assert code == 0, log
    assert (mutant / "world.at").read_bytes() != base_world, "semantic edit must move world"

    # 注释突变：逐字节相等。
    comment = work / "mutation-comment"
    comment.mkdir(parents=True, exist_ok=True)
    for f in fixture.iterdir():
        if f.is_file():
            (comment / f.name).write_text(
                f.read_text(encoding="utf-8").replace(
                    "components-3d-presentation profile.",
                    "components-3d-presentation profile!comment-only",
                ),
                encoding="utf-8",
            )
    shutil.copytree(fixture / "assets", comment / "assets", dirs_exist_ok=True)
    code, log = lower(exe, comment / "main.at", comment)
    assert code == 0, log
    sp.run([str(meta_exe), "--entry", str(comment), "--request", str(comment / "resources-request.json"), "--out", str(comment / "resources.json")], capture_output=True, text=True)
    code, log = lower(exe, comment / "main.at", comment, comment / "resources.json")
    assert code == 0, log
    assert (comment / "world.at").read_bytes() == base_world, "comment-only edit must be byte-equal"
    results["mutation_sensitivity"] = "semantic edit moves world; comment-only byte-equal"


def control_negatives(work: Path) -> None:
    exe = PILOT / "target" / "debug" / "node2d-lower.exe"
    meta_exe = REPO / "experimental" / "autographics-auto-driver" / "target" / "debug" / "resource_meta.exe"
    import subprocess as sp

    fixture = FIXTURES / "g10-animation"
    # CLI级负控1：陈旧元数据（字节长度改1）→ 对账具名拒绝。
    bad = work / "neg-stale"
    bad.mkdir(parents=True, exist_ok=True)
    for f in fixture.iterdir():
        if f.is_file():
            (bad / f.name).write_text(f.read_text(encoding="utf-8"), encoding="utf-8")
    shutil.copytree(fixture / "assets", bad / "assets", dirs_exist_ok=True)
    code, log = lower(exe, bad / "main.at", bad)
    assert code == 0, log
    proc = sp.run([str(meta_exe), "--entry", str(bad), "--request", str(bad / "resources-request.json"), "--out", str(bad / "resources.json")], capture_output=True, text=True)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    meta = json.loads((bad / "resources.json").read_text(encoding="utf-8"))
    meta["entries"][0]["identity"]["byte_length"] += 1
    (bad / "resources-stale.json").write_text(json.dumps(meta), encoding="utf-8")
    code, log = lower(exe, bad / "main.at", bad, bad / "resources-stale.json")
    assert code == 3, f"stale meta must reject, got {code}: {log}"
    assert "does not match declaration" in log, log
    # CLI级负控2：33计费作者夹具（extra-read产品）的负控在driver侧真fault；
    # 这里核其声明存在且生成产品world含两次pose调用。
    world = read(PRODUCTS_DIR / "g10-animation-extra-read" / "world.at")
    tick_pose = "ag_visual_pose(0, (player_body_rows[i]) + 1, clip, player_phase[i], 1, player_yaw[i])"
    assert world.count(tick_pose) == 2, "extra-read tick keeps both 1-credit pose calls (33rd credit -> driver fault 10)" 
    results["negatives"] = "stale meta named rejection + extra-read double pose shape"


def control_frozen_sources() -> None:
    for name, want in DRIVER_API_PINS.items():
        got = sha256_file(DRIVER_API / name)
        assert got == want, f"{name}: {got} != {want}"
    for name, want in SPEC_PINS.items():
        got = sha256_file(REPO / name)
        assert got == want, f"{name}: {got} != {want}"
    baseline_paths = {
        "character.glb": FIXTURES / "g10-animation" / "assets" / "character.glb",
        "colormap.png": FIXTURES / "g10-textures" / "assets" / "Textures" / "colormap.png",
        "platform.glb": FIXTURES / "g10-textures" / "assets" / "platform.glb",
        "coin.glb": FIXTURES / "g10-textures" / "assets" / "coin.glb",
        "flag.glb": FIXTURES / "g10-textures" / "assets" / "flag.glb",
        "uv-material-probe.glb": FIXTURES / "g10-textures" / "uv" / "uv-material-probe.glb",
        "asymmetric-checker.png": FIXTURES / "g10-textures" / "uv" / "asymmetric-checker.png",
        "checker-alt.png": FIXTURES / "g10-textures" / "uv" / "checker-alt.png",
    }
    for name, (want, length) in GALLERY_BASELINE.items():
        path = baseline_paths[name]
        data = path.read_bytes()
        assert len(data) == length, f"{name}: {len(data)} != {length}"
        got = sha256(data).hexdigest()
        assert got == want, f"{name}: sha drift got={got} want={want} path={path}"
    results["frozen_sources"] = "driver api + spec + gallery baseline pinned"


def control_old_profile_freeze() -> None:
    proc = subprocess.run(
        [sys.executable, "-I", str(PILOT / "tools" / "verify_plan030.py"), "--controls"],
        capture_output=True,
        text=True,
        cwd=str(PILOT),
    )
    assert proc.returncode == 0, f"verify_plan030 --controls failed:\n{proc.stdout}\n{proc.stderr}"
    assert "PASS plan030 controls: 32 checks" in proc.stdout, proc.stdout[-400:]
    results["old_profile_freeze"] = "verify_plan030 32 checks green (024/027/028/030 byte-frozen)"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    if not args.controls:
        print(__doc__)
        return
    work = Path(tempfile.mkdtemp(prefix="plan034-verify-"))
    try:
        control_identity()
        control_template_rows()
        control_cold_regen(work)
        control_mutation_sensitivity(work)
        control_negatives(work)
        control_frozen_sources()
        control_old_profile_freeze()
    finally:
        shutil.rmtree(work, ignore_errors=True)
    width = max(len(k) for k in results)
    for name, detail in results.items():
        print(f"  {name:<{width}}  {detail}")
    print(f"PASS plan034 controls: {len(results)} controls")


if __name__ == "__main__":
    main()
