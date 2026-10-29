"""Complete source closure policy shared by PLAN-012 freeze and verification."""
import hashlib
import os
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE = ROOT.parent
PACKAGES = ("autographics-auto-driver", "autographics-prototype")
BASE_COMMIT = "f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131"
PLAN_PIN = "c681663e6ff392d70471abbec560e9d9daf12622"
FIX_PIN = "0261ae9dacf786beb6dceb630be6af267f242625"
SDK_PIN = "7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467"
RAW_EXACT_RULES = ("autographics-auto-driver/sdk/frozen/**", "autographics-auto-driver/samples/**/scripts/*.at", "autographics-auto-driver/samples/public_audio/*.at")
LOCKS = {"autographics-auto-driver/Cargo.lock":"4a0b3040fffa2523f3932f2ab2b427e3c13285abce751813147cfe28000d9b06",
         "autographics-prototype/Cargo.lock":"c3df168e7234b8bfa876dfbab8afa71af8773d1d7241ac010ad5d8054930b3bf"}
TEXT = (".rs", ".at", ".py", ".md", ".json", ".toml", ".lock", ".txt", ".agscene", ".gltf", ".gitignore", ".gitattributes")
SKIP_DIRS = {"target", "__pycache__", ".sdk-source", ".sdk-source.prepare-tmp"}
SKIP_NAMES = {"window-report.json", "plan875-identity.json", "plan879-identity.json",
              "plan001-identity.json", "plan002-identity.json", "plan004-identity.json",
              "plan005-identity.json", "plan006-identity.json", "plan012-identity.json", "runtime-camera-reports", "runtime-objects-reports",
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
        relative.startswith("autographics-auto-driver/samples/") and p.suffix == ".at" and (
            "/scripts/" in relative or relative.startswith("autographics-auto-driver/samples/public_audio/")))
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

REQUIRED += ('autographics-auto-driver/api/public_audio_v1.at', 'autographics-auto-driver/api/public_audio2d_v1.at', 'autographics-auto-driver/api/public_audio3d_v1.at', 'autographics-auto-driver/src/authoring/public_audio/mod.rs', 'autographics-auto-driver/src/authoring/public_audio/codec.rs', 'autographics-auto-driver/src/authoring/public_audio/mixer.rs', 'autographics-auto-driver/src/authoring/public_audio/native.rs', 'autographics-auto-driver/src/authoring/public_audio/bank.rs', 'autographics-auto-driver/src/authoring/play_audio_tests.rs', 'autographics-auto-driver/tests/public_audio.rs', 'autographics-auto-driver/tests/public_audio_resources.rs', 'autographics-auto-driver/tests/public_audio_codec.rs', 'autographics-auto-driver/tests/public_audio_host.rs', 'autographics-auto-driver/tests/public_audio_compat.rs', 'autographics-auto-driver/tests/public_audio_consumer.rs', 'autographics-auto-driver/examples/audio_decode.rs', 'autographics-auto-driver/examples/build_public_audio_samples.rs', 'autographics-auto-driver/samples/public_audio/audio2d.at', 'autographics-auto-driver/samples/public_audio/audio3d.at', 'autographics-auto-driver/samples/public_audio/project2d/scene.agscene', 'autographics-auto-driver/samples/public_audio/project3d/scene.agscene', 'autographics-auto-driver/samples/public_audio/bank-audit.json', 'autographics-auto-driver/research/plan012-legacy-profiles.json', 'autographics-auto-driver/research/public-audio-codec-reference.json', 'autographics-auto-driver/research/public-audio-spec-delta.md', 'autographics-auto-driver/tools/plan012_source_policy.py', 'autographics-auto-driver/tools/gen_plan012_identity.py', 'autographics-auto-driver/tools/verify_plan012.py', 'autographics-auto-driver/tools/check_public_audio.py', 'autographics-auto-driver/tools/fresh_public_audio_consumer.py')
