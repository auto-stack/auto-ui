"""Complete source closure policy shared by PLAN-006 freeze and verification."""
import hashlib
import os
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PACKAGES = ("autographics-auto-driver", "autographics-prototype")
BASE_COMMIT = "80d9bbe3095892fcf5507dd1b9e26af523111fd4"
PLAN_PIN = "351919c76b4ef1d940fc7b8120254e321c41cba2"
FIX_PIN = "0261ae9dacf786beb6dceb630be6af267f242625"
SDK_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
RAW_EXACT_RULES = ("autographics-auto-driver/sdk/frozen/**", "autographics-auto-driver/samples/**/scripts/*.at")
LOCKS = {"autographics-auto-driver/Cargo.lock":"2b131d3dd33a968b43d53664cb8649d6c24225fa0f2c28c1add603be98a358db",
         "autographics-prototype/Cargo.lock":"c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf"}
TEXT = (".rs", ".at", ".py", ".md", ".json", ".toml", ".lock", ".txt", ".agscene", ".gltf", ".gitignore", ".gitattributes")
SKIP_DIRS = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
SKIP_NAMES = {"window-report.json", "plan875-identity.json", "plan879-identity.json",
              "plan001-identity.json", "plan002-identity.json", "plan004-identity.json",
              "plan005-identity.json", "plan006-identity.json", "runtime-camera-reports", "runtime-objects-reports",
              "runtime-triggers-reports", "sprite-frames-reports", "textured-model-reports"}
VOLATILE = re.compile(r"autographics-auto-driver/research/authoring-native-[0-9]+\.agscene")
REQUIRED = (
    "autographics-auto-driver/src/authoring/textured_models.rs",
    "autographics-auto-driver/src/authoring/assets.rs",
    "autographics-auto-driver/src/authoring/document.rs",
    "autographics-auto-driver/src/authoring/storage.rs",
    "autographics-auto-driver/src/authoring/session.rs",
    "autographics-auto-driver/src/authoring/host.rs",
    "autographics-auto-driver/tests/textured_model_resources.rs",
    "autographics-auto-driver/tests/public_sprite_frames.rs",
    "autographics-auto-driver/samples/textured_models/scene.agscene",
    "autographics-auto-driver/samples/textured_models/main.at",
    "autographics-auto-driver/samples/textured_models/generate_project.py",
    "autographics-auto-driver/samples/textured_models/models/platform.glb",
    "autographics-auto-driver/samples/textured_models/models/coin.glb",
    "autographics-auto-driver/samples/textured_models/models/flag.glb",
    "autographics-auto-driver/samples/textured_models/models/character.glb",
    "autographics-auto-driver/samples/textured_models/models/Textures/colormap.png",
    "autographics-auto-driver/samples/textured_models/SOURCE-README.md",
    "autographics-auto-driver/samples/textured_models/SOURCE-LICENSE.md",
    "autographics-auto-driver/research/textured-model-spec-delta.md",
    "autographics-auto-driver/tools/check_textured_models.py",
    "autographics-auto-driver/tools/gen_plan005_identity.py",
    "autographics-auto-driver/tools/verify_plan005.py",
    "autographics-auto-driver/tools/plan005_source_policy.py",
    "autographics-prototype/src/assets/textured.rs",
    "autographics-prototype/src/render3d/pipeline.rs",
    "autographics-prototype/tests/textured_models.rs",
)


def digest(p):
    b = p.read_bytes()
    relative = p.relative_to(BASE).as_posix()
    exact = relative.startswith("autographics-auto-driver/sdk/frozen/") or (
        relative.startswith("autographics-auto-driver/samples/") and "/scripts/" in relative and p.suffix == ".at")
    if not exact and (p.suffix.lower() in TEXT or p.name.lower() in TEXT):
        b = b.replace(b"\r\n", b"\n")
    return hashlib.sha256(b).hexdigest()


def excluded(relative):
    p = pathlib.PurePosixPath(relative)
    return bool(set(p.parts) & (SKIP_DIRS | SKIP_NAMES)) or bool(VOLATILE.fullmatch(relative))


def safe_path(relative):
    p = pathlib.PurePosixPath(relative)
    assert not p.is_absolute() and p.parts and p.parts[0] in PACKAGES, "outside source package"
    assert "/".join(p.parts) == relative and ".." not in p.parts and "\\" not in relative, "unsafe source path"
    assert not excluded(relative), "ignored/volatile source must not be pinned"
    raw = BASE / relative
    for ancestor in [raw, *raw.parents]:
        assert not ancestor.is_symlink() and not ancestor.is_junction(), f"linked source: {relative}"
        if ancestor == BASE:
            break
    return raw


def collect():
    files, folders = {}, set()
    for package in PACKAGES:
        root = BASE / package
        assert root.is_dir()
        for current, dirs, names in os.walk(root, followlinks=False):
            current = pathlib.Path(current)
            dirs[:] = sorted(d for d in dirs if not excluded((current / d).relative_to(BASE).as_posix()))
            relative = current.relative_to(BASE).as_posix()
            safe_path(relative)
            folders.add(relative)
            for name in sorted(names):
                relative = (current / name).relative_to(BASE).as_posix()
                if not excluded(relative):
                    raw = safe_path(relative)
                    assert raw.is_file(), f"missing source: {relative}"
                    files[relative] = digest(raw)
    return dict(sorted(files.items())), sorted(folders)

REQUIRED += (
    "autographics-auto-driver/api/node_animation3d_v1.at",
    "autographics-auto-driver/src/authoring/runtime_node_animation.rs",
    "autographics-auto-driver/src/authoring/behavior/assembly.rs",
    "autographics-auto-driver/src/authoring/play.rs",
    "autographics-auto-driver/src/authoring/host/viewport.rs",
    "autographics-auto-driver/src/authoring/host/play_adapter.rs",
    "autographics-auto-driver/tests/public_node_animation.rs",
    "autographics-auto-driver/tests/node_animation_resources.rs",
    "autographics-auto-driver/tests/node_animation_pixels.rs",
    "autographics-auto-driver/tests/node_animation_lifecycle.rs",
    "autographics-auto-driver/tests/node_animation_consumer.rs",
    "autographics-auto-driver/samples/node_animation/scene.agscene",
    "autographics-auto-driver/samples/node_animation/main.at",
    "autographics-auto-driver/samples/node_animation/generate_project.py",
    "autographics-auto-driver/samples/node_animation/README.md",
    "autographics-auto-driver/samples/node_animation/assets/7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1.glb",
    "autographics-auto-driver/research/node-animation-spec-delta.md",
    "autographics-auto-driver/tools/check_node_animation.py",
    "autographics-auto-driver/tools/gen_plan006_identity.py",
    "autographics-auto-driver/tools/verify_plan006.py",
    "autographics-auto-driver/tools/plan006_source_policy.py",
    "autographics-prototype/src/assets/node_animation.rs",
    "autographics-prototype/tests/node_animation.rs",
)
