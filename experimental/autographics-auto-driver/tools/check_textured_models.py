"""PLAN-005 exact-source, exact-runner acceptance gate.

--reports DIR runs fresh and retains evidence outside the source packages.
--validate DIR checks an existing bound report set. --controls exercises real
zero-selection plus tampered pixels, budgets, warm uploads, runner and coverage.
"""
import argparse
import hashlib
import json
import os
import pathlib
import re
import secrets
import shutil
import subprocess
import struct
import sys
import tempfile
import plan005_source_policy as p

SCHEMA = "autographics.plan005.report/1"
RUNNERS = {"driver": (p.ROOT, "textured_model_resources", 20),
           "agp": (p.BASE / "autographics-prototype", "textured_models", 4)}
DRIVER_CASES = ["identity-budget", "decoded-budget", "raw-budget", "gltf-transform",
                "runtime-owners", "runtime-transaction", "import-transaction", "public-project"]
AGP_CASES = ["sampling", "depth-and-facing", "shadow", "gpu-owners"]
CASES = {**dict.fromkeys(DRIVER_CASES, "driver"), **dict.fromkeys(AGP_CASES, "agp")}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity_digest():
    return digest(p.ROOT / "research/plan005-identity.json")


def stack_reserve(binary):
    raw=binary.read_bytes()
    assert raw[:2]==b"MZ", "Windows PE runner required"
    pe=struct.unpack_from("<I",raw,0x3c)[0]
    assert raw[pe:pe+4]==b"PE\0\0" and struct.unpack_from("<H",raw,pe+24)[0]==0x20b,"Windows x64 PE runner required"
    return struct.unpack_from("<Q",raw,pe+24+72)[0]


def source_guard(directory=None):
    result = subprocess.run([sys.executable, str(p.ROOT / "tools/verify_source.py")],
                            cwd=p.ROOT, capture_output=True, text=True)
    output = result.stdout + result.stderr
    assert result.returncode == 0, output[-4000:]
    if directory:
        (directory / "source.log").write_text(output, encoding="utf-8")


def run_runner(name, directory, selection=None):
    package, runner, count = RUNNERS[name]
    environment = dict(os.environ, UI005_REPORT_DIR=str(directory), CARGO_TERM_COLOR="never")
    environment["RUST_MIN_STACK"]="16777216"
    environment.pop("RUSTC_WRAPPER", None)
    if name == "agp":
        environment["CARGO_PROFILE_DEV_DEBUG"] = "0"
    command = ["cargo", "test", "--locked", "--offline", "--test", runner]
    if selection:
        command.append(selection)
    command += ["--", "--test-threads=1", "--nocapture"]
    result = subprocess.run(command, cwd=package, env=environment, capture_output=True, text=True)
    output = result.stdout + result.stderr
    (directory / f"{name}.log").write_text(output, encoding="utf-8")
    assert result.returncode == 0, output[-5000:]
    check_count(name,output)
    match = re.search(r"Running [^\r\n]*\(([^\r\n()]+)\)", output)
    assert match, "missing actual test executable"
    binary = pathlib.Path(match.group(1))
    if not binary.is_absolute():
        binary = package / binary
    binary = binary.resolve()
    target = (package / "target").resolve()
    assert package.resolve() in target.parents and target in binary.parents, "runner target escapes package"
    assert binary.is_file() and binary.name.startswith(runner + "-"), "wrong actual runner"
    assert stack_reserve(binary)==33554432,"executable stack profile moved"
    return dict(runner=runner, passed=count, failed=0, ignored=0, filtered=0,
                pe_stack_reserve=stack_reserve(binary),worker_stack=int(environment["RUST_MIN_STACK"]),
                binary=str(binary), binary_sha256=digest(binary), log_sha256=digest(directory / f"{name}.log"))


def check_count(name,output):
    count=RUNNERS[name][2]
    match = re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", output)
    assert match and tuple(map(int, match.groups())) == (count, 0, 0, 0, 0), f"partial/zero-selected runner: {name}: {match.groups() if match else 'no result'}"


def run(directory):
    assert not any(directory.iterdir()), "fresh reports directory must be empty"
    source_guard(directory)
    nonce = secrets.token_hex(16)
    runners = {name: run_runner(name, directory) for name in RUNNERS}
    for case, owner in CASES.items():
        path = directory / f"{case}.json"
        assert path.is_file(), f"missing fresh case: {case}"
        value = json.loads(path.read_text())
        report = dict(schema=SCHEMA, case=case, runner=RUNNERS[owner][1],
                      source_identity=identity_digest(), run_nonce=nonce, value=value)
        path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    manifest = dict(schema=SCHEMA, source_identity=identity_digest(), run_nonce=nonce,
                    cases=sorted(CASES), runners=runners, source_log_sha256=digest(directory / "source.log"))
    (directory / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def validate(directory, guard=True):
    if guard:
        source_guard()
    manifest = json.loads((directory / "manifest.json").read_text())
    assert manifest["schema"] == SCHEMA and manifest["source_identity"] == identity_digest(), "stale source identity"
    assert re.fullmatch(r"[0-9a-f]{32}", manifest["run_nonce"]), "invalid run nonce"
    assert manifest["cases"] == sorted(CASES), "partial case set"
    assert set(manifest["runners"]) == set(RUNNERS), "wrong runner set"
    assert digest(directory / "source.log") == manifest["source_log_sha256"], "source proof log changed"
    for owner, (package, runner, count) in RUNNERS.items():
        proof = manifest["runners"][owner]
        assert proof["runner"] == runner, "wrong runner"
        assert (proof["passed"], proof["failed"], proof["ignored"], proof["filtered"]) == (count, 0, 0, 0), "partial/zero-selected runner"
        assert proof["pe_stack_reserve"]==33554432 and proof["worker_stack"]==16777216,"runner stack profile moved"
        binary = pathlib.Path(proof["binary"]).resolve()
        target = (package / "target").resolve()
        assert package.resolve() in target.parents and target in binary.parents, "foreign runner path"
        assert binary.name.startswith(runner + "-") and digest(binary) == proof["binary_sha256"], "foreign/changed runner executable"
        assert stack_reserve(binary)==33554432,"actual executable stack profile moved"
        assert digest(directory / f"{owner}.log") == proof["log_sha256"], "runner proof log changed"
    values = {}
    for case, owner in CASES.items():
        path = directory / f"{case}.json"
        assert path.is_file(), f"missing case: {case}"
        r = json.loads(path.read_text())
        assert (r["schema"], r["case"], r["runner"], r["source_identity"], r["run_nonce"]) == (SCHEMA, case, RUNNERS[owner][1], identity_digest(), manifest["run_nonce"]), f"foreign case/runner: {case}"
        values[case] = r["value"]
    numeric={"cases","probes","max_error","tolerance","mutation_rejections","texture_count","draw_order_controls","reveal_controls","facing_cases","culling_cases","limit","accepted_identities","duplicate_decodes","accepted_cpu","accepted_gpu","accepted_raw","rounds","vm_releases","shared_textures","warm_mesh_uploads","warm_texture_uploads","warm_decodes","cpu_bytes","gpu_bytes","actual_gpu_bytes","failure_stages","models","raw_identities"}
    for case,value in values.items():
        assert all(type(v) is int and v>=0 for k,v in value.items() if k in numeric),f"invalid numeric report: {case}"
    s = values["sampling"]
    assert s["cases"] == 36 and s["probes"] == 144 and 0 <= s["max_error"] <= 3 and s["tolerance"] == 3, "sampling pixel oracle"
    assert s["mutation_rejections"] == 2 and s["texture_count"] == 1, "sampling mutations/shared texture"
    d = values["depth-and-facing"]
    assert (d["draw_order_controls"], d["reveal_controls"], d["facing_cases"]) == (2, 1, 4) and 0 <= d["max_error"] <= 2 and d["tolerance"] == 2, "depth/facing oracle"
    s = values["shadow"]
    assert s["culling_cases"] == 3 and s["disable_control_rejected"] is True and s["restore_control_passed"] is True, "shadow controls"
    assert s["lit_pixel"][0] > s["shadow_pixel"][0] + 40, "shadow pixels"
    g = values["gltf-transform"]
    assert g["probes"] == 4 and 0 <= g["max_error"] <= 2 and g["tolerance"] == 2 and g["mutation_rejections"] == 2 and g["collision_bounds_unchanged"] is True and g["node_association_recoverable"] is True, "GLB transform oracle"
    i = values["identity-budget"]
    assert i == dict(limit=8, accepted_identities=8, ninth_refused_before_decode=True, duplicate_decodes=0), "identity budget"
    b = values["decoded-budget"]
    assert b["limit"] == 64 * 1024**2 and all(48 * 1024**2 < b[n] < b["limit"] for n in ["accepted_cpu", "accepted_gpu"]) and b["unused_images_counted"] is True and b["sprite_refused_before_decode"] is True, "decoded budget"
    b = values["raw-budget"]
    assert b["limit"] == 32 * 1024**2 and 24 * 1024**2 < b["accepted_raw"] < b["limit"] and b["over_limit_refused_before_decode"] is True and b["duplicate_decodes"] == 0, "raw budget"
    w = values["runtime-owners"]
    assert w["rounds"] == 20 and w["vm_releases"] == 20 and w["shared_textures"] == 1, "runtime owner rounds"
    assert all(w[n] == 0 for n in ["warm_mesh_uploads", "warm_texture_uploads", "warm_decodes"]), "warm static re-upload/decode"
    assert 0 < w["cpu_bytes"] <= 64 * 1024**2 and 0 < w["gpu_bytes"] == w["actual_gpu_bytes"] <= 64 * 1024**2, "actual resource budget"
    assert w["last_cpu_owner_released"] is True and w["last_gpu_owner_released"] is True, "last owner leak"
    for case, booleans in {
        "gpu-owners": ["failed_candidate_released", "failed_upload_counted", "malformed_before_upload", "foreign_device_refused", "last_external_pin_released"],
        "runtime-transaction": ["full_frame_preserved", "camera_preserved", "pixels_preserved", "recovered"],
        "import-transaction": ["new_upload_observed", "candidate_cpu_owners_released", "old_pixels_preserved", "successful_retry_changed_pixels"],
        "public-project": ["bound_auto", "runtime_instances", "runtime_camera", "saved_moved_reopened", "pixels_equal", "reopened_auto_runs"],
    }.items():
        assert all(values[case][n] is True for n in booleans), f"transaction/owner/project: {case}"
    assert values["gpu-owners"]["shared_textures"] == 1
    assert values["runtime-transaction"]["failure_stages"] == 2 == values["import-transaction"]["failure_stages"]
    assert values["public-project"]["models"] == 3 and values["public-project"]["raw_identities"] == 4
    return values


def controls(directory):
    work = pathlib.Path(tempfile.mkdtemp(prefix="ui005-report-controls-")).resolve()
    assert work.parent == pathlib.Path(tempfile.gettempdir()).resolve()
    def rejected(path, needle):
        try:
            validate(path, guard=False)
        except (AssertionError, KeyError, FileNotFoundError, json.JSONDecodeError) as error:
            assert needle in str(error), str(error)
        else:
            raise AssertionError(f"control accepted: {needle}")
    changes = [
        ("pixel", "sampling", "max_error", 255, "sampling pixel oracle"),
        ("warm", "runtime-owners", "warm_texture_uploads", 1, "warm static"),
        ("budget", "decoded-budget", "limit", 128 * 1024**2, "decoded budget"),
    ]
    for label, case, field, value, needle in changes:
        path = work / label
        shutil.copytree(directory, path)
        raw = json.loads((path / f"{case}.json").read_text())
        raw["value"][field] = value
        (path / f"{case}.json").write_text(json.dumps(raw))
        rejected(path, needle)
    for label in ["partial", "runner", "zero", "stale"]:
        path = work / label
        shutil.copytree(directory, path)
        if label == "partial":
            (path / "public-project.json").unlink()
            rejected(path, "missing case")
            continue
        raw = json.loads((path / "manifest.json").read_text())
        if label == "runner":
            raw["runners"]["driver"]["runner"] = "public_sprite_frames"
        elif label == "zero":
            raw["runners"]["driver"]["passed"] = 0
        else:
            raw["source_identity"] = "0" * 64
        (path / "manifest.json").write_text(json.dumps(raw))
        rejected(path, {"runner":"wrong runner", "zero":"zero-selected", "stale":"stale source"}[label])
    zero = work / "actual-zero"
    zero.mkdir()
    # Invoke the already bound executable. Asking Cargo to rebuild for a
    # negative selection can legitimately relink a PE and invalidate the
    # positive run's binary fingerprint, despite unchanged source inputs.
    proof=json.loads((directory/"manifest.json").read_text())["runners"]["driver"]
    binary=pathlib.Path(proof["binary"])
    assert digest(binary)==proof["binary_sha256"],"bound executable changed before zero control"
    result=subprocess.run([str(binary),"__ui005_no_such_test__","--test-threads=1","--nocapture"],cwd=p.ROOT,capture_output=True,text=True)
    output=result.stdout+result.stderr
    (zero/"driver.log").write_text(output,encoding="utf-8")
    assert result.returncode==0,output
    try:
        check_count("driver",output)
    except AssertionError as error:
        assert "zero-selected" in str(error), str(error)
    else:
        raise AssertionError("actual zero-selected cargo execution was accepted")
    validate(directory, guard=False)
    print(f"PASS PLAN-005 report controls: pixel/partial/runner/budget/warm/stale/zero and actual zero-selected execution refused; evidence {work}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports", type=pathlib.Path)
    parser.add_argument("--validate", type=pathlib.Path)
    parser.add_argument("--controls", action="store_true")
    args = parser.parse_args()
    assert not (args.reports and args.validate), "choose fresh run or existing validation"
    if args.validate:
        directory = args.validate.resolve()
    else:
        directory = args.reports.resolve() if args.reports else pathlib.Path(tempfile.mkdtemp(prefix="ui005-reports-")).resolve()
        if args.reports:
            assert not directory.exists(), "refuse to reuse a reports directory"
            directory.mkdir(parents=True)
        run(directory)
    validate(directory)
    if args.controls:
        controls(directory)
    print(f"PASS PLAN-005 textured models: 12 cases, driver20 + AGP4, 36/144 pixel sampling, portable bound project and 20-run owners; reports {directory}")
