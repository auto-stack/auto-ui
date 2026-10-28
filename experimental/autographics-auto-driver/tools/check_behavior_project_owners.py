"""Targeted owner and GPU reuse mutation checks, never the entire test package."""
import argparse,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);results=[]
command=['cargo','test','--locked','--test','behavior_project_host','bound_project_whole_path_gpu_history_and_twenty_owner_cycles','--','--exact','--nocapture','--test-threads=1']
for name,flag,reason in [('retained-source','AG_PROJECT_RETAIN_CODE','project source must release after last internal owner'),('retained-vm','AG_PLAY_RETAIN_VM','runtime VM must release at Stop'),('fresh-upload','AG_PLAY_FRESH_UPLOAD','warm model must retain actual GPU Arc'),('restored',None,None)]:
 env=os.environ.copy();env['RUSTC_WRAPPER']=''
 for key in ['AG_PROJECT_RETAIN_CODE','AG_PLAY_RETAIN_VM','AG_PLAY_FRESH_UPLOAD']:env.pop(key,None)
 if flag:env[flag]='1'
 r=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=180);(out/(name+'.stdout.txt')).write_bytes(r.stdout);(out/(name+'.stderr.txt')).write_bytes(r.stderr);text=(r.stdout+r.stderr).decode(errors='replace')
 assert (r.returncode==0)==(flag is None),(name,r.returncode,text[-1800:])
 assert (reason in text)if reason else('1 passed; 0 failed'in text)
 results.append(dict(case=name,exit=r.returncode,reason=reason,command=command));print(json.dumps(results[-1]),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2))
