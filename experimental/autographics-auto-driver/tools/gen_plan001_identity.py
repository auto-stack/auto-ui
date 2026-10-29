"""Freeze the PLAN-001 source closure (finite runtime object profiles).

Run once per semantic revision, in the plan worktree, after the source set is
stable:

  python tools/gen_plan001_identity.py

The identity lists every source/evidence input of both experimental packages
plus the two pins the closure claims: the auto-ui master commit this plan
branched from (the PLAN-879 migration delivery, docs-only commits above it)
and the frozen SDK artifact pin. Volatile run reports, plan-artifact outputs
and the identity file itself are excluded from the walk.
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
    "runtime-camera-reports",
    "runtime-objects-reports",
}
# auto-ui master at the planning pin (PLAN-879 delivery); this plan branched
# from a9e8d70 which adds two docs/plans commits with identical code.
BASE_COMMIT = "a9e8d7070ddefcf485bc6bbc664516a1c081dabe"
PLAN_PIN = "4adf2d037a1804741adc9c5a012f08cfa566439d"
SDK_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
ADAPTATIONS = {
    "autographics-auto-driver/api/runtime_objects_v1.at": "new: PLAN-001 trusted objects unit (finite run-object table, 8 slots, integer handles)",
    "autographics-auto-driver/src/authoring/runtime_objects.rs": "new: admitted block types, named admission, generation high-water, non-persistent render adapter",
    "autographics-auto-driver/tests/runtime_objects_api.rs": "new: V-01/V-02 session-level suite plus the T-02 whole-path probe",
    "autographics-auto-driver/tests/runtime_objects_host.rs": "new: V-03/V-04 host/GPU suite, envelopes, 20-cycle churn, env-gated sampler",
    "autographics-auto-driver/tests/runtime_objects_samples.rs": "new: V-04 sample acceptance (two dimensions, parameter sensitivity, template-swap mutation)",
    "autographics-auto-driver/samples/runtime-objects/two/layout_a.agscene": "new: 2D sample layout (character/ground + circle/quad templates)",
    "autographics-auto-driver/samples/runtime-objects/two/main.at": "new: objects2d/1 public sample",
    "autographics-auto-driver/samples/runtime-objects/two/main-alt.at": "new: objects2d/1 parameter-edited twin",
    "autographics-auto-driver/samples/runtime-objects/three/layout_a.agscene": "new: 3D sample layout (character/ground + mob/cylinder templates)",
    "autographics-auto-driver/samples/runtime-objects/three/main.at": "new: objects3d/1 public sample",
    "autographics-auto-driver/samples/runtime-objects/three/main-alt.at": "new: objects3d/1 parameter-edited twin",
    "autographics-auto-driver/src/authoring/behavior.rs": "modified: Api gains Objects2V1/Objects3V1 (numbers 105/106, profiles 16/17), OBJECTS_API include",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs": "modified: objects composition (motion+character+camera+objects), dim-literal freeze, entry freeze chain before app_start",
    "autographics-auto-driver/src/authoring/play.rs": "modified: Frame.instances, objects block queries every tick, admit_objects with generation high-water, slot capacity gate",
    "autographics-auto-driver/src/authoring/host/viewport.rs": "modified: prepare_frame renders admitted instances through the non-persistent adapter, hidden-template mesh upload, live-slot mesh denominator",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs": "modified: objects mesh envelope gate at start_program, instances ride prepare_frame",
    "autographics-auto-driver/src/bin/authoring/cli.rs": "modified: --behavior-api gains objects2d/objects3d",
    "autographics-auto-driver/tools/verify_source.py": "modified: current identity chain selects plan001; historical chain preserved behind it",
    "autographics-auto-driver/tools/package_consumer.py": "modified: latest identity list selects plan001",
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
        "plan": "auto-ui/PLAN-001",
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
    out = ROOT / "research/plan001-identity.json"
    out.write_text(json.dumps(identity, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"frozen {len(files)} files -> {out}")
    assert identity["sdk_frozen_files"] == 681, identity["sdk_frozen_files"]


if __name__ == "__main__":
    main()
