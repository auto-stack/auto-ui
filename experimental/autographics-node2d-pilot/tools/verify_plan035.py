#!/usr/bin/env python3
"""PLAN-035 components-3d-audio-hud 生成链控制门（--controls）。

产品（generated-node3d-audio-hud/3个永久入口）经三步管线再核：
声明 → resource-meta（既有公开导入准入面）→ 最终产品；identity/场景行/
world数值needle逐字节、A→B冷再生逐字节、冻结源（driver受信API字节：
composition3d/audio3d/hud3d、SDK681文件数、素材7身份子集、cue权威span）
与旧档冻结（verify_plan034 --controls 全门作为受影响回归并集）。

运行期（真Host/Auto/GPU/PCM/native）检查在 driver 侧冻结入口：
  cargo test --locked --offline --test node3d_audio_hud_gates
  cargo test --locked --offline --test node3d_audio_hud_flow -- --test-threads=1
  cargo test --locked --offline --test public_3d_composition_api -- --test-threads=1
  cargo test --locked --offline --test plan035_probe -- --test-threads=1
  cargo test --locked --offline --test node3d_audio_hud_storage -- --test-threads=1
用法：python -I tools/verify_plan035.py --controls
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
PRODUCTS_DIR = PILOT / "generated-node3d-audio-hud"
DRIVER = REPO / "experimental" / "autographics-auto-driver"

PRODUCTS = [
    "actorfree",
    "batch-tick",
    "g3-full-loop",
]
FIXTURES = {
    "actorfree": PILOT / "fixtures" / "node3d-presentation-services" / "actorfree",
    "batch-tick": PILOT / "fixtures" / "node3d-presentation-services" / "batch-tick",
    "g3-full-loop": PILOT / "fixtures" / "g3-full-loop",
}

GENERATOR = "autographics-node2d-pilot plan035 components-3d-audio-hud"
API = "autographics.components3d/1"
CONSUMERS = (
    "experimental/autographics-auto-driver/tests/node3d_audio_hud_flow.rs",
)

# 冻结源（T-03 冻结身份；变更即失约）。
DRIVER_API_PINS = {
    "public_composition3d_v1.at": "f46fdb5aa2241a7beb10a2b622c8e453e7b0cad377dd8dba8ca26b5f7ab280b3",
    "public_audio3d_v1.at": "035b0d2c5b09bf4927f8c7868a5cec804fb220f8fd6d81ae3cb2d79deed9dbfb",
    "public_hud3d_v1.at": "3b0e63cf3b97c9a87c9e69af94af49f53c62ac0125de6b0285b575355c3636eb",
}
SDK_FILE_COUNT = 681
MATERIAL_PINS = {
    "character.glb": ("7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1", 87624),
    "flag.glb": ("0ba84ff43f0fea0a9f0b080cfa1be54ebda36c8c35174d1ac2553ae6da9f65ae", 13276),
    "Textures/colormap.png": ("106cf02e0d6dccded6d9f90c2ae6a51eb94c6301645fea159e834c44ed4708a3", 10108),
    "b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d.wav": ("b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d", 90060),
    "4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa.ttf": ("4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa", 26828),
}
# cue 权威 span（§5.6：project2d 权威表，帧域 44100Hz/2ch/22504帧）。
G3_CUES = {
    "jump": (0, 7403, 1.0),
    "coin": (7403, 20046, 1.0),
    "land": (20046, 22504, 1.0),
    "win": (7403, 20046, 0.75),
    "fail": (20046, 22504, 0.75),
}

results = {}


def sha256_file(path: Path) -> str:
    return sha256(path.read_bytes()).hexdigest()


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def jload(path: Path):
    return json.loads(read(path))


def lower(exe: Path, entry: Path, out: Path, resources: Path | None = None) -> tuple[int, str]:
    cmd = [str(exe), "--profile", "components-3d-audio-hud", "--entry", str(entry), "--out", str(out)]
    if resources is not None:
        cmd += ["--resources", str(resources)]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    return proc.returncode, (proc.stdout + proc.stderr)


# ---------------------------------------------------------------- controls --


def control_identity() -> None:
    for product in PRODUCTS:
        identity = jload(PRODUCTS_DIR / product / "identity.json")
        assert identity["generator"] == GENERATOR, (product, identity["generator"])
        assert identity["api"] == API, (product, identity["api"])
        assert identity["provenance"]["consumer"] in CONSUMERS, (
            product,
            identity["provenance"]["consumer"],
        )
        assert (PRODUCTS_DIR / product / "world.at").exists(), product
        assert (PRODUCTS_DIR / product / "scene.agscene").exists(), product
    results["identity"] = f"{len(PRODUCTS)} products generator/api/consumer pinned"


def control_scene_rows() -> None:
    # actorfree：kind=-1 合法路径（scene 三维 actor 恒 null）；batch/g3：恰一角色。
    af = read(PRODUCTS_DIR / "actorfree" / "scene.agscene")
    assert '"three_d": null' in af, "actorfree must bind no 3D actor"
    assert '"schema": "autographics.scene/8"' in af, "scene/8 identity"
    for product in ("batch-tick", "g3-full-loop"):
        scene = read(PRODUCTS_DIR / product / "scene.agscene")
        assert '"three_d": 1' in scene, f"{product} binds exactly one 3D actor"
        assert '"schema": "autographics.scene/8"' in scene, product
        # 恰四槽完整 HUD 状态（含未声明槽的缺省隐藏行）。
        assert scene.count('"slot":') == 4, f"{product} must carry four slots"
        assert '"font": "sha256:4444c23631884f411aa7f2fee35061b1756a9927940c62d2a5572acf470b3ffa"' in scene, product
        assert '"sha256:b0b740dc49e51abefff9bb1af830080fe00f817b468614992040d438d6171c3d"' in scene, product
    # g3 权威 cue 表（名称自由，span-事件对应随作者声明）。
    g3 = jload(PRODUCTS_DIR / "g3-full-loop" / "scene.agscene")
    cues = {c["name"]: (c["start_frame"], c["end_frame"], c["gain"]) for c in g3["cues"]}
    assert cues == G3_CUES, f"g3 cue spans drifted: {cues}"
    results["scene_rows"] = "actors/null|1, four slots, font/bank identity, g3 authoritative cue spans"


def control_world_needles() -> None:
    af = read(PRODUCTS_DIR / "actorfree" / "world.at")
    bt = read(PRODUCTS_DIR / "batch-tick" / "world.at")
    g3 = read(PRODUCTS_DIR / "g3-full-loop" / "world.at")
    # kind=-1 / kind=0 收口（每回调恰一次 ag_present3d bundle）。
    assert "ag_present3d(-1" in af, "actor-free closeout is kind=-1"
    assert "ag_char3_bind" not in af, "actor-free binds no character"
    for w in (bt, g3):
        assert "ag_present3d(0" in w, "character closeout is kind=0"
        assert "ag_events_snapshot()" in w, "bounded event snapshot port"
        assert "ag_object_despawn_batch(" in w, "whole-batch despawn port"
        # 恰 5 行缓冲（第 5 行只把 n=5 送入具名 fault 51）。
        assert "if r_np < 5" in w, "the sfx collector buffers exactly five rows"
        # ahud 门输入快照前奏（一次包读替代五次计费动作读）。
        assert " let packet=ag_actions_snapshot()" in w, "recv input-snapshot prelude"
    assert "ag_round_request()" in g3, "g3 fresh-Enter new run keeps round.request"
    results["world_needles"] = "kind=-1/0 closeouts, snapshot/batch ports, 5-row buffer, recv prelude, g3 round.request"


def control_cold_regen(work: Path) -> None:
    exe = PILOT / "target" / "debug" / "node2d-lower.exe"
    meta_exe = DRIVER / "target" / "debug" / "resource_meta.exe"
    assert exe.exists() and meta_exe.exists(), "build pilot + resource-meta first"
    for product in PRODUCTS:
        fixture = FIXTURES[product]
        out = work / f"regen-{product}"
        entry = work / f"closure-{product}"
        if entry.exists():
            shutil.rmtree(entry)
        shutil.copytree(fixture, entry)
        # A→B：B 只拿输入闭包（作者.at + 声明的资源文件），无生产者源树。
        code, log = lower(exe, entry / "main.at", out)
        assert code == 0, f"{product} pass A: {log}"
        proc = subprocess.run(
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
        got_identity = jload(out / "identity.json")
        want_identity = jload(PRODUCTS_DIR / product / "identity.json")
        assert got_identity.pop("provenance")["product"] is not None
        want_identity.pop("provenance")
        assert got_identity == want_identity, f"{product}/identity cold regen drift"
        regen_assets = sorted(p.name for p in (out / "assets").iterdir())
        want_assets = sorted(p.name for p in (PRODUCTS_DIR / product / "assets").iterdir())
        assert regen_assets == want_assets, f"{product} asset set drift"
        for name in want_assets:
            assert (out / "assets" / name).read_bytes() == (
                PRODUCTS_DIR / product / "assets" / name
            ).read_bytes(), f"{product}/assets/{name} byte drift"
    results["cold_regen"] = f"{len(PRODUCTS)} products byte-equal from closure (A->meta->B)"


def control_frozen_sources() -> None:
    for name, want in DRIVER_API_PINS.items():
        got = sha256_file(DRIVER / "api" / name)
        assert got == want, f"driver api {name}: {got} != {want}"
    sdk = DRIVER / ".sdk-source"
    count = sum(1 for p in sdk.rglob("*") if p.is_file())
    assert count == SDK_FILE_COUNT, f"SDK frozen file count {count} != {SDK_FILE_COUNT}"
    assets = FIXTURES["g3-full-loop"] / "assets"
    for name, (want_sha, want_len) in MATERIAL_PINS.items():
        blob = (assets / name).read_bytes()
        got = sha256(blob).hexdigest()
        assert got == want_sha and len(blob) == want_len, f"material {name}: {got}/{len(blob)}"
    results["frozen_sources"] = "composition3d/audio3d/hud3d api bytes, SDK681 count, material subset pinned"


def control_old_profile_union() -> None:
    # 受影响旧档回归并集：034 生成链控制门整门复跑（旧 profile 字节安全）。
    proc = subprocess.run(
        [sys.executable, "-I", str(PILOT / "tools" / "verify_plan034.py"), "--controls"],
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"verify_plan034 --controls failed:\n{proc.stdout}\n{proc.stderr}"
    results["old_profile_union"] = "verify_plan034 --controls green (old-profile bytes safe)"


CONTROLS = [
    control_identity,
    control_scene_rows,
    control_world_needles,
    control_cold_regen,
    control_frozen_sources,
    control_old_profile_union,
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    if not args.controls:
        print(__doc__)
        return 2
    with tempfile.TemporaryDirectory(prefix="verify-plan035-") as td:
        work = Path(td)
        for control in CONTROLS:
            if control is control_cold_regen:
                control_cold_regen(work)
            else:
                control()
    print("verify_plan035 --controls: PASS")
    for key, val in results.items():
        print(f"  {key}: {val}")
    print("runtime gates (driver side, run separately, single instance):")
    print("  cargo test --locked --offline --test node3d_audio_hud_gates -- --test-threads=1")
    print("  cargo test --locked --offline --test node3d_audio_hud_flow -- --test-threads=1")
    print("  cargo test --locked --offline --test public_3d_composition_api -- --test-threads=1")
    print("  cargo test --locked --offline --test plan035_probe -- --test-threads=1")
    print("  cargo test --locked --offline --test node3d_audio_hud_storage -- --test-threads=1")
    return 0


if __name__ == "__main__":
    sys.exit(main())
