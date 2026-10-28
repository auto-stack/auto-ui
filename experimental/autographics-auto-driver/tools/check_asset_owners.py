"""Replay actual retained-owner and GPU reupload faults against one formal test."""
import argparse,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);a=p.parse_args();dest=a.output.resolve();dest.mkdir(parents=True,exist_ok=False)
command=['cargo','test','--locked','--test','authoring_assets','twenty_real_cycles_vm_asset_mesh_last_owner_cancel_and_external_pins','--','--exact','--nocapture','--test-threads=1']
results=[]
for name,flag,reason in [('retained-owner','AG_ASSET_RETAIN_OWNER','last actual asset/raw/GPU owner must release'),('fresh-upload','AG_ASSET_FRESH_UPLOAD','warm existing resource must not upload again'),('restored',None,None)]:
 env=os.environ.copy();env.pop('AG_ASSET_RETAIN_OWNER',None);env.pop('AG_ASSET_FRESH_UPLOAD',None)
 if flag:env[flag]='1'
 run=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=180);(dest/(name+'.stdout.txt')).write_bytes(run.stdout);(dest/(name+'.stderr.txt')).write_bytes(run.stderr)
 text=(run.stdout+run.stderr).decode(errors='replace')
 assert (run.returncode==0)==(flag is None),(name,run.returncode,text[-2000:])
 if reason:assert reason in text,(name,text[-2000:])
 else:assert '1 passed; 0 failed'in text
 results.append(dict(case=name,exit=run.returncode,reason=reason,command=command))
(dest/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2))
