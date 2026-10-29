"""Freeze the PLAN-004 source closure (finite PNG visuals and 2D sprite frames).

Run once per semantic revision, in the plan worktree, after the source set is
stable:

  python tools/gen_plan004_identity.py

The identity lists every file of both experimental packages (the same closure
policy as PLAN-002) plus the two pins the closure claims: the auto-ui commit the
worktree branched from and 002's actual delivered commit. The frozen SDK
snapshot is included, so the identity also proves the SDK bytes did not move.
Volatile run reports and the identity file itself are excluded from the walk.
"""
import json
import hashlib
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
    "plan004-identity.json",
    "runtime-camera-reports",
    "runtime-objects-reports",
    "runtime-triggers-reports",
    "sprite-frames-reports",
}
# auto-ui master at this worktree's base (docs-only commits above 002's archive).
BASE_COMMIT = "d3bd6b495dc8b690925e88deee9fbf2c42e7f991"
# The actual delivered dependency this plan builds on (not a development tip).
PLAN_PIN = "0ee5ad0747ec3bb67e1288401fcc31daf3901c6e"
ADAPTATIONS = {
    "autographics-auto-driver/api/sprite2d_v1.at": "new: PLAN-004 trusted sprite unit (bounded frame/flip writes, slot defaults, frozen table emitters)",
    "autographics-auto-driver/src/authoring/runtime_sprites.rs": "new: frozen target table, admitted sprite block, independent identity/bounds/run/generation oracle",
    "autographics-auto-driver/tests/public_sprite_frames.rs": "new: V-01..V-05 driver suite (admission, closure, editor binding, real-Auto probe, pixels, lifecycle, owners)",
    "autographics-auto-driver/samples/sprite_frames/main.at": "new: sprites2d/1 public engine fixture (frame walk, direction flip, instance write, slot respawn)",
    "autographics-auto-driver/samples/sprite_frames/sheet.png": "new: self-made 96x32 three-frame fixture (asymmetric halves, per-frame marker, transparent row)",
    "autographics-auto-driver/samples/sprite_frames/other.png": "new: self-made 16x16 second identity fixture",
    "autographics-auto-driver/samples/sprite_frames/padded.png": "new: self-made 32x32 transparent-border fixture for the pixel oracle",
    "autographics-auto-driver/samples/sprite_frames/generate.py": "new: deterministic fixture generator with --check",
    "autographics-auto-driver/tools/check_sprite_frames.py": "new: report gate (fresh run, strict values, tampered/partial/foreign rejection)",
    "autographics-auto-driver/tools/gen_plan004_identity.py": "new: identity generator",
    "autographics-auto-driver/tools/verify_plan004.py": "new: identity verifier with mutation controls",
    "autographics-prototype/src/renderer/sprite.rs": "new: content-addressed texture store, nearest/clamp sprite pipeline, UV-only flip",
    "autographics-prototype/src/assets/png.rs": "new: bounded static-PNG preflight/decode and fixture encoder",
    "autographics-prototype/tests/sprite_resources.rs": "new: AGP pixel/identity/device/owner suite",
    "autographics-auto-driver/src/authoring/assets.rs": "modified: PNG payload beside the model payload, shared 8/32MiB/64MiB budget, format dispatch",
    "autographics-auto-driver/src/authoring/document.rs": "modified: scene/5 sprite binding, legacy rejection, frame/identity validation",
    "autographics-auto-driver/src/authoring/storage.rs": "modified: PNG blobs in the project closure, shared file budget, foreign/bad preservation",
    "autographics-auto-driver/src/authoring/session.rs": "modified: Edit::SpriteObject create/rebind/detach rules",
    "autographics-auto-driver/src/authoring/behavior.rs": "modified: Api::Sprites2V1 (profile 20, number 109) and the sprites() gate",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs": "modified: sprites composition and entry hooks (freeze/tick budgets) without touching earlier profiles",
    "autographics-auto-driver/src/authoring/play.rs": "modified: Frame.sprites, sprite queries every tick, admit_sprites with the frozen table",
    "autographics-auto-driver/src/authoring/host/viewport.rs": "modified: sprite commands from the accepted frame or document default, prepare-before-accept",
    "autographics-auto-driver/src/authoring/host.rs": "modified: Prepared sprites/store, prune keeps textures only while an owner holds the payload",
    "autographics-auto-driver/src/authoring/host/commands.rs": "modified: import-sprite/sprite-bind/sprite-create/sprite-detach",
    "autographics-auto-driver/src/authoring/host/panel.rs": "modified: PNG path field and per-image bind/create/detach buttons",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs": "modified: play prepare passes the admitted sprite frame",
    "autographics-prototype/src/host/surface_runtime.rs": "modified: Game2dContent sprite pass/store, device and upload accounting, slot-preserving failure",
    "autographics-prototype/src/renderer/mod.rs": "modified: sprite module exports",
    "autographics-prototype/src/assets/mod.rs": "modified: PNG exports",
    "autographics-auto-driver/tests/authoring_assets.rs": "modified: current schema assertion follows scene/5",
    "autographics-auto-driver/tests/authoring_document.rs": "modified: codec mutation targets the current schema",
    "autographics-auto-driver/tests/authoring_hierarchy.rs": "modified: v1 fixture drops the post-v4 field; current schema assertions follow scene/5",
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
    baseline = json.loads((ROOT / "research/baseline.json.txt").read_text())
    identity = {
        "plan": "auto-ui/PLAN-004",
        "base_commit": BASE_COMMIT,
        "plan_pin": PLAN_PIN,
        "sdk_frozen_files": sum(
            1 for f in files if f.startswith("autographics-auto-driver/sdk/frozen/")
        ),
        "sdk_pin": baseline["sdk"]["repository_commit"],
        "adaptations": ADAPTATIONS,
        "text_suffixes": TEXT_SUFFIXES,
        "folders": sorted(set(folders)),
        "files": files,
    }
    out = ROOT / "research/plan004-identity.json"
    out.write_text(json.dumps(identity, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"frozen {len(files)} files -> {out}")
    assert identity["sdk_frozen_files"] == 681, identity["sdk_frozen_files"]


if __name__ == "__main__":
    main()
