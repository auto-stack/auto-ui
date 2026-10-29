"""PLAN-006 frozen-source gate. --controls rejects altered and empty proofs.

Fresh reports are external to both packages; validation binds exact logs,
executables, source identity, stack profile, case set and observable oracles.
"""
import argparse
import copy
import hashlib
import json
import os
import pathlib
import re
import secrets
import struct
import subprocess
import sys
import tempfile
import plan006_source_policy as p

SCHEMA = "autographics.plan006.report/1"
# (package, test target, passed, filtered, exact optional selection)
RUNNERS = {
    "public":(p.ROOT,"public_node_animation",8,0,None),
    "resources":(p.ROOT,"node_animation_resources",5,0,None),
    "pixels":(p.ROOT,"node_animation_pixels",5,0,None),
    "lifecycle":(p.ROOT,"node_animation_lifecycle",5,0,None),
    "consumer":(p.ROOT,"node_animation_consumer",2,0,None),
    "core":(p.BASE/"autographics-prototype","node_animation",5,0,None),
    "legacy-profiles":(p.ROOT,"public_sprite_frames",1,16,"older_profiles_keep_their_exact_assembled_bytes_and_the_new_one_fits"),
    "legacy-sprite":(p.ROOT,"public_sprite_frames",1,16,"real_auto_drives_frames_flip_and_the_whole_2d_composition_in_one_tick"),
    "legacy-texture":(p.ROOT,"textured_model_resources",1,20,"public_bound_project_runs_saves_moves_and_reopens_without_original_import_paths"),
}

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(): return digest(p.ROOT/"research/plan006-identity.json")
def stack(binary):
    raw=binary.read_bytes();assert raw[:2]==b"MZ","PE runner required"
    pe=struct.unpack_from("<I",raw,0x3c)[0]
    assert raw[pe:pe+4]==b"PE\0\0" and struct.unpack_from("<H",raw,pe+24)[0]==0x20b,"x64 PE required"
    return struct.unpack_from("<Q",raw,pe+24+72)[0]
def guard():
    env=dict(os.environ);env.pop("CARGO_TARGET_DIR",None)
    result=subprocess.run([sys.executable,str(p.ROOT/"tools/verify_source.py")],cwd=p.ROOT,env=env,capture_output=True,text=True)
    assert result.returncode==0,(result.stdout+result.stderr)[-5000:]
    return result.stdout+result.stderr
def counts(name,output):
    row=RUNNERS[name]
    found=re.search(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out",output)
    assert found and tuple(map(int,found.groups()))==(row[2],0,0,0,row[3]),f"partial/zero-selected runner {name}"
    if row[4]: assert f"test {row[4]} ..." in output,"wrong selected case"
def run(directory):
    assert not any(directory.iterdir()),"fresh reports must be empty"
    (directory/"source.log").write_text(guard(),encoding="utf-8")
    proofs={}
    for name,(package,target,passed,filtered,selection) in RUNNERS.items():
        env=dict(os.environ,UI006_REPORT_DIR=str(directory),CARGO_TERM_COLOR="never",RUST_MIN_STACK="16777216")
        env.pop("RUSTC_WRAPPER",None)
        if name=="core": env["CARGO_PROFILE_DEV_DEBUG"]="0"
        command=["cargo","test","--locked","--offline","--test",target]
        if selection: command.append(selection)
        command += ["--","--test-threads=1","--nocapture"]
        print("RUN",name,flush=True)
        result=subprocess.run(command,cwd=package,env=env,capture_output=True,text=True)
        output=result.stdout+result.stderr
        log=directory/f"{name}.log";log.write_text(output,encoding="utf-8")
        assert result.returncode==0,output[-6000:]
        counts(name,output)
        m=re.search(r"Running [^\r\n]*\(([^\r\n()]+)\)",output);assert m,"missing actual runner"
        binary=pathlib.Path(m[1]);binary=(package/binary).resolve() if not binary.is_absolute() else binary.resolve()
        assert binary.name.startswith(target+"-") and binary.suffix==".exe","wrong actual runner"
        assert stack(binary)==33554432,"main stack changed"
        proofs[name]=dict(package=str(package.resolve()),target=target,passed=passed,filtered=filtered,binary=str(binary),binary_sha256=digest(binary),log_sha256=digest(log),stack_reserve=stack(binary),worker_stack=16777216)
    manifest=dict(schema=SCHEMA,source_identity=identity(),nonce=secrets.token_hex(16),runners=proofs,source_log_sha256=digest(directory/"source.log"),data_sha256={n:digest(directory/f"{n}.json") for n in ["consumer","performance"]})
    (directory/"manifest.json").write_text(json.dumps(manifest,indent=2)+"\n",encoding="utf-8")

def validate(directory,source_guard=True):
    if source_guard: guard()
    m=json.loads((directory/"manifest.json").read_text())
    assert m["schema"]==SCHEMA and m["source_identity"]==identity(),"stale/wrong source identity"
    assert re.fullmatch("[0-9a-f]{32}",m["nonce"]),"missing run nonce"
    assert set(m["runners"])==set(RUNNERS),"partial runner set"
    assert digest(directory/"source.log")==m["source_log_sha256"],"source log changed"
    logs={}
    for name,(package,target,passed,filtered,_) in RUNNERS.items():
        proof=m["runners"][name]
        assert (proof["target"],proof["passed"],proof["filtered"])==(target,passed,filtered),"wrong runner/count"
        binary=pathlib.Path(proof["binary"]).resolve()
        allowed=pathlib.Path(os.environ.get("CARGO_TARGET_DIR",str(package/"target"))).resolve()
        assert proof["package"]==str(package.resolve()) and allowed in binary.parents,"foreign runner path/package"
        assert binary.name.startswith(target+"-") and digest(binary)==proof["binary_sha256"],"changed/foreign executable"
        assert stack(binary)==proof["stack_reserve"]==33554432 and proof["worker_stack"]==16777216,"stack changed"
        log=directory/f"{name}.log";assert digest(log)==proof["log_sha256"],"proof log changed"
        output=log.read_text(encoding="utf-8");counts(name,output);logs[name]=output
    # Validate actual observable output as well as the complete assertion runners.
    shadow=re.search(r"ROI probes=(\d+), shadow_pixels=(\d+), errors=(\d+); wrong frozen-pose=(\d+), disabled=(\d+)",logs["pixels"])
    assert shadow,"missing shadow pixel oracle"
    probes,pixels,error,frozen,disabled=map(int,shadow.groups())
    assert probes>10000 and pixels>100 and error<=2 and frozen>20 and disabled==pixels,"shadow pixel oracle"
    step=re.search(r"interpolated45deg wrong-pose ROI errors=(\d+)",logs["pixels"])
    assert step and int(step[1])>2,"STEP distinguishing control"
    assert "four camera-relative directions + stop + +/-pi wrap" in logs["pixels"],"missing facing coverage"
    assert "source vertices=43596" in logs["pixels"] and "GPU stable ROI probes=2074207" in logs["pixels"],"missing original clip pixel coverage"
    assert "20 cycles: Play/restart/Stop; 40 VMs and poses dropped; warm uploads/decode delta0" in logs["lifecycle"],"owner/warm invariant"
    assert "240 real input ticks: animation/yaw on/off equal positions" in logs["lifecycle"],"physical invariance"
    assert "additional snapshot causes error10" in logs["public"],"32/33-credit control"
    for name in ["consumer","performance"]:
        assert digest(directory/f"{name}.json")==m["data_sha256"][name],"data proof changed"
    consumer=json.loads((directory/"consumer.json").read_text())
    assert pathlib.Path(consumer["manifest_dir"]).resolve()==p.ROOT.resolve(),"foreign compiled consumer"
    assert all(type(consumer[k]) is int and consumer[k]>=0 for k in ["clips","ticks","managed_bytes","callback_steps","callback_slices","warm_mesh_uploads","warm_texture_uploads"]),"invalid consumer numbers"
    assert 0<consumer["callback_steps"]<=100000 and 0<consumer["callback_slices"]<=121,"consumer callback work budget"
    assert all(consumer[k] is True for k in ["cold_reopen","moved_project","user_mutation_sensitive"]),"public consumer path"
    assert consumer["clips"]==4 and consumer["ticks"]==60 and consumer["managed_bytes"]<=96*1024,"public source/clip envelope"
    assert consumer["warm_mesh_uploads"]==consumer["warm_texture_uploads"]==0,"warm upload invariant"
    perf=json.loads((directory/"performance.json").read_text())
    assert perf["guards"]==dict(steps=100000,resumptions=120,callback_ms=250,tick_ms=2000),"guard changed"
    assert [(r["shape"],r["nodes"],r["accepted"]) for r in perf["layouts"]]==[("regular",3,True),("wide",120,True),("deep",64,True),("deep",120,False)],"CPU envelope layout coverage"
    for r in perf["layouts"]:
        if r["accepted"]:
            assert type(r["steps"]) is int and 0<r["steps"]<=100000 and 0<r["slices"]<=121,"callback work budget"
            assert 0<r["tick_median_us"]<=r["tick_max_us"]<2000000,"tick deadline"
            u=r["resources"];assert 0<u["cpu_total_with_pose_bytes"]<=64*1024**2 and 0<u["gpu_expected_bytes"]<=64*1024**2,"resource union budget"
            assert all(type(u[k]) is int and u[k]>0 for k in ["cpu_total_with_pose_bytes","cpu_geometry_bytes","pose_reserved_cpu_bytes","gpu_expected_bytes"]),"invalid resource numbers"
            assert u["cpu_total_with_pose_bytes"]==u["cpu_geometry_bytes"]+u["pose_reserved_cpu_bytes"],"pose missing from ledger"
        else: assert r["last_good_preserved"] is True and "cumulative CPU budget exhausted" in r["error"],"over-envelope refusal"
    print("PASS PLAN-006 bound source/runner/oracle reports",flush=True)

def controls(directory):
    original=json.loads((directory/"manifest.json").read_text())
    def rejected(mutator,needle):
        work=pathlib.Path(tempfile.mkdtemp(prefix="ui006-control-",dir=directory.parent))
        # Proof files only; binaries remain at their actual bound locations.
        for f in directory.iterdir():
            if f.is_file() and f.suffix in [".json",".log"]: (work/f.name).write_bytes(f.read_bytes())
        m=copy.deepcopy(original);mutator(work,m)
        (work/"manifest.json").write_text(json.dumps(m),encoding="utf-8")
        try: validate(work,False)
        except (AssertionError,KeyError,FileNotFoundError) as e: assert needle in str(e),str(e)
        else: raise AssertionError("accepted control: "+needle)
    rejected(lambda w,m:m.update(source_identity="0"*64),"source identity")
    rejected(lambda w,m:m["runners"].pop("pixels"),"partial runner")
    rejected(lambda w,m:m["runners"]["public"].update(passed=0),"wrong runner/count")
    rejected(lambda w,m:m["runners"]["pixels"].update(target="public_sprite_frames"),"wrong runner/count")
    def bad_pixel(w,m):
        f=w/"pixels.log";s=f.read_text(encoding="utf-8");s,n=re.subn(r"shadow_pixels=(\d+), errors=0;",r"shadow_pixels=\1, errors=999;",s);assert n==1;f.write_text(s,encoding="utf-8");m["runners"]["pixels"]["log_sha256"]=digest(f)
    rejected(bad_pixel,"shadow pixel oracle")
    def data_control(case,change):
        def mutate(w,m):
            f=w/f"{case}.json";v=json.loads(f.read_text());change(v);f.write_text(json.dumps(v),encoding="utf-8");m["data_sha256"][case]=digest(f)
        return mutate
    rejected(data_control("consumer",lambda v:v.update(warm_mesh_uploads=1)),"warm upload")
    rejected(data_control("performance",lambda v:v["guards"].update(steps=200000)),"guard changed")
    rejected(data_control("performance",lambda v:v["layouts"][0]["resources"].update(cpu_total_with_pose_bytes=128*1024**2)),"resource union")
    binary=pathlib.Path(original["runners"]["public"]["binary"])
    result=subprocess.run([str(binary),"__ui006_no_such_test__","--test-threads=1"],cwd=p.ROOT,capture_output=True,text=True)
    assert result.returncode==0
    try: counts("public",result.stdout+result.stderr)
    except AssertionError as e: assert "zero-selected" in str(e)
    else: raise AssertionError("accepted actual zero-selected execution")
    validate(directory,False)
    print("PASS PLAN-006 controls: stale/partial/runner/zero/pixels/warm/CPU/resource mutations and actual zero-selection refused")

if __name__=="__main__":
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument("--reports",type=pathlib.Path);ap.add_argument("--validate",type=pathlib.Path);ap.add_argument("--controls",action="store_true");args=ap.parse_args()
    assert not(args.reports and args.validate)
    if args.validate: directory=args.validate.resolve()
    else:
        directory=args.reports.resolve() if args.reports else pathlib.Path(tempfile.mkdtemp(prefix="ui006-reports-")).resolve()
        if args.reports: assert not directory.exists();directory.mkdir(parents=True)
        run(directory)
    validate(directory)
    if args.controls: controls(directory)
    print("PASS PLAN-006: new30 tests + 3 selected inherited cases; original clips, pixels, physical/transaction/owners, public consumer and unchanged CPU budgets")
