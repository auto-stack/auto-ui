"""Finding-specific actual owner/scan/upload mutations and restored control."""
import argparse,json,os,pathlib,subprocess
from m2_evidence import record_runner, verify_runner
root=pathlib.Path(__file__).resolve().parents[1];p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
command=['cargo','test','--locked','--test','motion_owners','twenty_real_v1_v2_v3_v4_v5_cancel_fault_history_open_owners_and_warm_geometry','--','--exact','--nocapture','--test-threads=1']
# Preserve the native producer before a test build can replace target artifacts.
native=root/'target/debug/autographics-authoring.exe'
if native.exists():record_runner(native,out/'native-runner.json')
env=os.environ.copy();env['RUSTC_WRAPPER']='';env['RUST_MIN_STACK']='16777216'
build=subprocess.run(['cargo','test','--locked','--test','motion_owners','--no-run','--message-format=json'],cwd=root,env=env,capture_output=True,timeout=180)
(out/'build.stdout.txt').write_bytes(build.stdout);(out/'build.stderr.txt').write_bytes(build.stderr)
assert build.returncode==0,'owner test build failed'
artifacts=[v for line in build.stdout.splitlines() if (v:=json.loads(line)).get('reason')=='compiler-artifact' and v.get('executable') and v['target']['name']=='motion_owners']
assert len(artifacts)==1,'owner executable selection'
runner=record_runner(pathlib.Path(artifacts[0]['executable']),out/'owner-runner.json')
command=[str(verify_runner(runner)),'twenty_real_v1_v2_v3_v4_v5_cancel_fault_history_open_owners_and_warm_geometry','--exact','--nocapture','--test-threads=1']
cases=[('source','AG_SPATIAL_RETAIN_CODE','only authorized external source pin may survive'),('vm','AG_SPATIAL_RETAIN_VM','runtime VM must release at Stop'),('spatial','AG_SPATIAL_RETAIN_DESCRIPTOR','spatial descriptor must release at Stop'),('rescan','AG_SPATIAL_RESCAN_WARM','warm geometry must not rescan vertices'),('upload','AG_SPATIAL_FRESH_UPLOAD','warm model must retain actual GPU Arc'),('content','AG_SWEEP_WRONG_CONTENT','warm canvas content must match accepted editing scene'),('restored',None,None)];results=[]
for name,flag,reason in cases:
 env=os.environ.copy();env['RUSTC_WRAPPER']='';env['RUST_MIN_STACK']='16777216'
 for _,f,_ in cases:
  if f:env.pop(f,None)
 if flag:env[flag]='1'
 verify_runner(runner)
 r=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=180);(out/(name+'.stdout.txt')).write_bytes(r.stdout);(out/(name+'.stderr.txt')).write_bytes(r.stderr);text=(r.stdout+r.stderr).decode(errors='replace')
 assert (r.returncode==0)==(flag is None),(name,r.returncode,text[-2200:]);assert reason in text if reason else '1 passed; 0 failed'in text
 results.append(dict(case=name,command=command,exit=r.returncode,oracle=reason,runner_sha256=runner['sha256'],selected_ids=['twenty_real_v1_v2_v3_v4_v5_cancel_fault_history_open_owners_and_warm_geometry'],completed=True));print(json.dumps(results[-1]),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2))
