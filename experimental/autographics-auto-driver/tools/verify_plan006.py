"""Verify current complete source/asset closure; --controls restores mutations."""
import json
import subprocess
import sys
import plan006_source_policy as p

IDENTITY = p.ROOT / "research/plan006-identity.json"


def verify():
    identity = json.loads(IDENTITY.read_text())
    assert identity["plan"] == "auto-ui/PLAN-006", "wrong Plan"
    assert identity["base_commit"] == p.BASE_COMMIT, "wrong base pin"
    assert identity["plan_pin"] == p.PLAN_PIN, "wrong 005 delivery pin"
    assert identity["identity_fix_pin"] == p.FIX_PIN, "wrong 004 identity-fix pin"
    assert identity["sdk_pin"] == p.SDK_PIN, "wrong SDK pin"
    assert identity["text_suffixes"] == list(p.TEXT), "wrong text normalization policy"
    assert identity["raw_exact_rules"] == list(p.RAW_EXACT_RULES), "wrong immutable byte policy"
    for relative in identity["files"]:
        p.safe_path(relative)
    actual, folders = p.collect()
    assert actual.keys() == identity["files"].keys(), "source set drift"
    assert folders == identity["folders"], "source folder drift"
    for relative, expected in identity["files"].items():
        assert actual[relative] == expected, f"source hash mismatch: {relative}"
    assert identity["sdk_frozen_files"] == 681 == sum(n.startswith("autographics-auto-driver/sdk/frozen/") for n in actual), "SDK snapshot size"
    assert all(n in actual for n in p.REQUIRED), "required source unlisted"
    assert all(actual[n]==value for n,value in p.LOCKS.items()),"frozen dependency lock changed"
    assert json.loads((p.ROOT / "research/baseline.json.txt").read_text())["sdk"]["repository_commit"] == p.SDK_PIN
    print(f"PASS PLAN-006 source/asset identity: {len(actual)} hashes; 005 delivery/004 fix, SDK681, complete animation/input/renderer/runner/sample inputs bound")


def rejected():
    result = subprocess.run([sys.executable, __file__], capture_output=True, text=True)
    assert result.returncode != 0 and "AssertionError" in result.stderr, result.stderr[-2000:]


def controls():
    verify()
    for relative in ["api/node_animation3d_v1.at", "src/authoring/runtime_node_animation.rs", "tests/node_animation_pixels.rs", "samples/node_animation/main.at", "samples/node_animation/assets/7112f6a08400914f9da546f3e6029e947cc9eab2b4a6da5eb99776111289efb1.glb", "samples/textured_models/models/Textures/colormap.png"]:
        path = p.ROOT / relative
        original = path.read_bytes()
        try:
            path.write_bytes(original + b"\ncontrol\n")
            rejected()
            path.unlink()
            rejected()
        finally:
            path.write_bytes(original)
    for relative in ["samples/textured_models/unlisted-control.bin", "research/authoring-native-stable.agscene"]:
        path = p.ROOT / relative
        assert not path.exists()
        try:
            path.write_bytes(b"unlisted source control\n")
            rejected()
        finally:
            path.unlink(missing_ok=True)
    original = IDENTITY.read_bytes()
    try:
        for field in ["base_commit", "plan_pin", "identity_fix_pin", "sdk_pin"]:
            identity = json.loads(original)
            identity[field] = "0" * 40
            IDENTITY.write_text(json.dumps(identity))
            rejected()
        identity = json.loads(original)
        identity["files"]["../outside.rs"] = "0" * 64
        IDENTITY.write_text(json.dumps(identity))
        rejected()
    finally:
        IDENTITY.write_bytes(original)
    volatile = p.ROOT / "research/authoring-native-987654321.agscene"
    assert not volatile.exists()
    try:
        volatile.write_bytes(b"temporary native probe\n")
        verify()
        identity = json.loads(original)
        identity["files"][volatile.relative_to(p.BASE).as_posix()] = p.digest(volatile)
        IDENTITY.write_text(json.dumps(identity))
        rejected()
    finally:
        volatile.unlink(missing_ok=True)
        IDENTITY.write_bytes(original)
    verify()
    print("PASS PLAN-006 identity controls: edited/deleted/unlisted/stable-scene/wrong-pins/escape/PID-pin rejected; volatile PID output ignored")
    for path in (p.ROOT / "samples/node_animation/scripts").glob("*.at"):
        original = path.read_bytes()
        assert b"\r\n" not in original and b"\n" in original
        try:
            path.write_bytes(original.replace(b"\n", b"\r\n"))
            rejected()
        finally:
            path.write_bytes(original)
    verify()
    print("PASS PLAN-006 exact-byte control: content-addressed Auto LF-to-CRLF mutation refused")


if __name__ == "__main__":
    controls() if "--controls" in sys.argv else verify()
