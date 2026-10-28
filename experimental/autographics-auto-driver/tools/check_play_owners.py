"""Actual retained runtime VM / repeated model upload faults, with a restored control."""
import argparse,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);p.add_argument('--user',action='store_true');a=p.parse_args();dest=a.output.resolve();dest.mkdir(parents=True,exist_ok=False)
test,case=('authoring_behavior_host','actual_source_gpu_twenty_runs_stop_fault_and_source_selection_isolation')if a.user else('authoring_play_host','twenty_real_vm_resource_owner_cycles_and_last_external_pin')
command=['cargo','test','--locked','--test',test,case,'--','--exact','--nocapture','--test-threads=1'];results=[]
for name,flag,reason in [('retained-vm','AG_PLAY_RETAIN_VM','runtime VM must release at Stop'),('fresh-upload','AG_PLAY_FRESH_UPLOAD','warm model must retain actual GPU Arc'),('restored',None,None)]:
 env=os.environ.copy();env.pop('AG_PLAY_RETAIN_VM',None);env.pop('AG_PLAY_FRESH_UPLOAD',None)
 if flag:env[flag]='1'
 run=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=180);(dest/(name+'.stdout.txt')).write_bytes(run.stdout);(dest/(name+'.stderr.txt')).write_bytes(run.stderr);text=(run.stdout+run.stderr).decode(errors='replace')
 assert (run.returncode==0)==(flag is None),(name,run.returncode,text[-2000:])
 if reason:assert reason in text,(name,text[-2000:])
 else:assert '1 passed; 0 failed'in text
 results.append(dict(case=name,exit=run.returncode,reason=reason,command=command))
(dest/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(results,indent=2))
