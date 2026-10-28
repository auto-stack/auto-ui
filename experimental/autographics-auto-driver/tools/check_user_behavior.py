"""Native user-source gate: exact runner bytes, portable projects, real OS/GPU oracles.
No build, source rewriting in the runner, or Rust behavior substitution is performed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('output',type=Path)
p.add_argument('--executable',type=Path,default=root/'target/debug/autographics-authoring.exe')
cases=['move','patrol','fast','relocated','missing','callback','api-error','cpu-budget','partial','transparent','missing-part','wrong-asset','wrong-focus']
p.add_argument('--cases',default=','.join(cases))
a=p.parse_args(); selected=a.cases.split(',');assert selected and set(selected)<=set(cases) and len(set(selected))==len(selected),'invalid/empty cases'
dest=a.output.resolve();dest.mkdir(parents=True,exist_ok=False)
exe=a.executable.resolve();exe_hash=hashlib.sha256(exe.read_bytes()).hexdigest()
results=[]
for name in selected:
    case=dest/name;case.mkdir();mode='patrol'if name=='patrol'else'move'
    project=case/'project';shutil.copytree(root/'samples/user_behavior'/mode,project)
    if name=='relocated':
        target=case/'relocated-project';project.rename(target);assert not project.exists();project=target
    source=project/'main.at';reference='patrol'if mode=='patrol'else'move';args=[]
    if name=='fast':
        data=source.read_bytes();assert data.count(b'speed=60.0')==1
        source.write_bytes(data.replace(b'speed=60.0',b'speed=90.0'));reference='move-fast'
        assert hashlib.sha256(data).digest()!=hashlib.sha256(source.read_bytes()).digest()
    if name=='missing':source.unlink()
    if name=='callback':source.write_text('fn app_start(){}\n')
    if name=='api-error':source.write_text('fn app_start(){ag_set_visible(-1,1)}\nfn app_update(dt float){}\n')
    if name=='cpu-budget':source.write_text('fn app_start(){while true {}}\nfn app_update(dt float){}\n')
    if name=='partial':args+=['--frames','1']
    if name in ['transparent','missing-part','wrong-asset']:args+=['--play-gpu-fault',name]
    if name=='wrong-focus':args+=['--wrong-play-focus']
    report=case/'report.json';command=[str(exe),'--behavior-check','--open',str(project/'scene.agscene'),'--behavior','main.at','--behavior-reference',reference,'--report',str(report),*args]
    run=subprocess.run(command,cwd=case,capture_output=True,timeout=120)
    (case/'process.json').write_text(json.dumps(dict(command=command,exit=run.returncode,executable_sha256=exe_hash),indent=2)+'\n')
    (case/'stdout.txt').write_bytes(run.stdout);(case/'stderr.txt').write_bytes(run.stderr)
    assert report.is_file(),(name,'native omitted report',run.returncode)
    d=json.loads(report.read_text());success=name in ['move','patrol','fast','relocated']
    assert (run.returncode==0)==success and d['passed']==success,(name,run.returncode,d.get('error'))
    if success:
        assert d['play_phase']==27
        pixels=[v for v in d['play_records']if v['case']=='play-pixels']
        assert len(pixels)==6 and {v['stage']for v in pixels}=={'editing','2d-moved','3d-moved','ui-isolated','shown','stopped'}
        assert all(v['passed'] and len(v['checks'])==6 and all(c['passed']for c in v['checks'])for v in pixels)
        traces=[v for v in d['play_records']if v['case']=='tick-trace'];assert len(traces)>=3
        expected_hash=hashlib.sha256(source.read_bytes()).hexdigest()
        assert all(t['behavior']['hash']==expected_hash and t['behavior']['version']=='autographics.behavior/1' and Path(t['behavior']['path']).samefile(source) for t in traces)
        assert any(v['case']=='stopped'for v in d['play_records'])
        saved=next(v for v in d['play_records']if v['case']=='saved')
        reopen=subprocess.run([str(exe),'--inspect',str(Path(saved['path']))],cwd=case,capture_output=True,timeout=30)
        assert reopen.returncode==0,reopen.stderr
        r=json.loads(reopen.stdout);assert r['document']==saved['document'] and r['two']==saved['two'] and r['three']==saved['three'] and r['history']==[0,0,0] and not r['dirty']
        (case/'reopen.json').write_bytes(reopen.stdout)
    else:
        reason={'missing':'read','callback':'missing callback app_update','api-error':'code=1','cpu-budget':'budget','partial':'incomplete','transparent':'actual play endpoint/origin/all-part pixels','missing-part':'actual play endpoint/origin/all-part pixels','wrong-asset':'actual play endpoint/origin/all-part pixels','wrong-focus':'wrong-focus'}[name]
        assert reason in d['error'],(name,d['error'])
        if name in ['transparent','missing-part','wrong-asset']:assert any(v['case']=='play-pixels'and not v['passed']for v in d['play_records'])
    assert hashlib.sha256(exe.read_bytes()).hexdigest()==exe_hash,'runner changed during the gate'
    result=dict(case=name,command=command,exit=run.returncode,expected_success=success,phase=d['play_phase'],frames=d['frames'],error=d.get('error'),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest()if source.exists()else None)
    results.append(result);print(json.dumps(result),flush=True)
    (dest/'results.json').write_text(json.dumps(dict(executable_sha256=exe_hash,selected=selected,completed=len(results),passed=len(results)==len(selected),cases=results),indent=2)+'\n')
