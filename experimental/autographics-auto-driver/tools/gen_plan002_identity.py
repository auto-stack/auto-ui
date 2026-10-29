"""Freeze the PLAN-002 source closure (finite trigger events and audited rounds).

Run once per semantic revision, in the plan worktree, after the source set is
stable:

  python tools/gen_plan002_identity.py

The identity lists every source/evidence input of both experimental packages
plus the two pins the closure claims: the auto-ui master commit this plan
branched from (001's delivery plus docs-only commits above it) and the frozen
SDK artifact pin. Volatile run reports, plan-artifact outputs and the identity
file itself are excluded from the walk.
"""
import hashlib
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PACKAGES = ["autographics-auto-driver", "autographics-prototype"]
TEXT_SUFFIXES = [
    ".rs",
    ".at",
    ".py",
    ".md",
    ".json",
    ".toml",
    ".lock",
    ".txt",
    ".agscene",
    ".gltf",
    ".gitignore",
    ".gitattributes",
]
SKIP_DIRS = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
SKIP_NAMES = {
    "window-report.json",
    "plan875-identity.json",
    "plan879-identity.json",
    "plan001-identity.json",
    "plan002-identity.json",
    "runtime-camera-reports",
    "runtime-objects-reports",
    "runtime-triggers-reports",
}
# auto-ui master at the worktree base (001's delivery 537122b plus two
# docs/plans commits); the plan pin is 001's actual delivered commit.
BASE_COMMIT = "78ee3b70c38697e51181e296eb91e88ea1e761b4"
PLAN_PIN = "537122b09c025191355e260bc8e9880a4358cda2"
SDK_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
ADAPTATIONS = {
    "autographics-auto-driver/api/runtime_triggers_v1.at": "new: PLAN-002 trusted triggers unit (8 explicit pairs, bounded events, geometry lock, restart request)",
    "autographics-auto-driver/src/authoring/runtime_round.rs": "new: admitted rounds block types, independent geometry/phase oracle, binding-generation watermark, restart marker",
    "autographics-auto-driver/tests/runtime_triggers_api.rs": "new: V-01/V-02 session-level suite (geometry oracle, cursor, lock, rejections, budget probes)",
    "autographics-auto-driver/tests/runtime_round_host.rs": "new: V-03/V-04 host/GPU suite (restart swap, failure last-good, 20 cycles, paused/step gates)",
    "autographics-auto-driver/tests/runtime_trigger_samples.rs": "new: V-05 engine-fixture acceptance (two dimensions, parameter twin, template mutation)",
    "autographics-auto-driver/samples/runtime-triggers/two/layout_a.agscene": "new: 2D engine fixture layout (character/ground + coin/hazard/goal templates off the sweep line)",
    "autographics-auto-driver/samples/runtime-triggers/two/main.at": "new: triggers2d/1 public engine fixture (collect/win/Enter-restart)",
    "autographics-auto-driver/samples/runtime-triggers/two/main-alt.at": "new: triggers2d/1 parameter-edited twin (fails at the hazard)",
    "autographics-auto-driver/samples/runtime-triggers/three/layout_a.agscene": "new: 3D engine fixture layout (character/ground + shard/saw/beacon templates off the sweep line)",
    "autographics-auto-driver/samples/runtime-triggers/three/main.at": "new: triggers3d/1 public engine fixture",
    "autographics-auto-driver/samples/runtime-triggers/three/main-alt.at": "new: triggers3d/1 parameter-edited twin",
    "autographics-auto-driver/src/authoring/behavior.rs": "modified: Api gains Triggers2V1/Triggers3V1 (numbers 107/108, profiles 18/19), TRIGGERS_API include, MAX_BRIDGE 96KiB with the recorded counterexample",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs": "modified: triggers composition (motion+character+camera+objects+triggers), entry evaluate injection, derived post-evaluate geometry lock, managed-envelope guard",
    "autographics-auto-driver/src/authoring/play.rs": "modified: Frame.rounds, rounds block queries every tick, admit_rounds with independent geometry oracle, round_contact/bind_water tracking, round restart fields",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs": "modified: pump_play consumes the accepted restart marker and swaps in a fresh Run/VM keeping the editing restore and last-good",
    "autographics-auto-driver/src/bin/authoring/cli.rs": "modified: --behavior-api gains triggers2d/triggers3d",
    "autographics-auto-driver/tools/verify_source.py": "modified: current identity chain selects plan002; historical chain preserved behind it",
    "autographics-auto-driver/tools/package_consumer.py": "modified: latest identity list selects plan002",
}


def digest(path):
    data = path.read_bytes()
    if path.suffix.lower() in TEXT_SUFFIXES or path.name.lower() in TEXT_SUFFIXES:
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


def main():
    files = {}
    folders = []
    for package in PACKAGES:
        package_root = BASE / package
        for child in sorted(package_root.iterdir()):
            if child.is_file():
                files[f"{package}/{child.name}"] = digest(child)
            elif child.is_dir() and child.name not in SKIP_DIRS:
                folders.append(f"{package}/{child.name}")
        for path in sorted(package_root.rglob("*")):
            if path.is_dir():
                continue
            relative = path.relative_to(BASE).as_posix()
            if any(part in SKIP_DIRS for part in path.parts):
                continue
            if path.name in SKIP_NAMES:
                continue
            files[relative] = digest(path)
        for path in sorted(package_root.rglob("*")):
            if path.is_dir() and path.name not in SKIP_DIRS:
                relative = path.relative_to(BASE).as_posix()
                if relative not in folders:
                    folders.append(relative)
    identity = {
        "plan": "auto-ui/PLAN-002",
        "base_commit": BASE_COMMIT,
        "plan_pin": PLAN_PIN,
        "sdk_frozen_files": sum(
            1 for f in files if f.startswith("autographics-auto-driver/sdk/frozen/")
        ),
        "sdk_pin": SDK_PIN,
        "adaptations": ADAPTATIONS,
        "text_suffixes": TEXT_SUFFIXES,
        "folders": sorted(set(folders)),
        "files": files,
    }
    out = ROOT / "research/plan002-identity.json"
    out.write_text(json.dumps(identity, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"frozen {len(files)} files -> {out}")
    assert identity["sdk_frozen_files"] == 681, identity["sdk_frozen_files"]


if __name__ == "__main__":
    main()
