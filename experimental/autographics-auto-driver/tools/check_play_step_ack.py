"""Finding-specific native Step acknowledgment regression; never dispatches Host commands."""
import argparse,hashlib,json,shutil,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--executable',type=Path,default=root/'target/debug/autographics-authoring.exe');p.add_argument('--cases',default='normal,delayed,dropped,duplicate');a=p.parse_args()
cases={'normal':([],0),'delayed':(['--play-step-delay-frames','36'],0),'dropped':(['--drop-play-step'],1),'duplicate':(['--double-play-step'],1)}
selected=a.cases.split(',');assert selected and set(selected)<=set(cases)and len(set(selected))==len(selected)
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);exe=a.executable.resolve();identity=hashlib.sha256(exe.read_bytes()).hexdigest();rows=[]
for name in selected:
 d=out/name;d.mkdir();project=d/'project';shutil.copytree(root/'samples/user_behavior/move',project)
 args,expected=cases[name];report=d/'report.json';cmd=[str(exe),'--behavior-check','--open',str(project/'scene.agscene'),'--behavior','main.at','--behavior-reference','move','--report',str(report),*args]
 r=subprocess.run(cmd,cwd=d,capture_output=True,timeout=120);(d/'stdout.txt').write_bytes(r.stdout);(d/'stderr.txt').write_bytes(r.stderr);data=json.loads(report.read_text());row=dict(case=name,command=cmd,exit=r.returncode,expected=expected,binary=identity,phase=data['play_phase'],error=data['error']);rows.append(row);(out/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(row),flush=True)
 assert r.returncode==expected and data['passed']==(expected==0),row
 request=next(v for v in data['play_records']if v['case']=='step-request');baseline=request['tick'];observed=[v for v in data['play_records']if v['case']=='step-observed']
 b=request['bounds'];sx=data['window_px'][0]/1200;sy=data['window_px'][1]/800
 actual=[v for v in data['inputs']if v.get('play_phase')==6 and v['event']in ['down','up'] and (810+b[0])*sx<=v['point'][0]<(810+b[0]+b[2])*sx and b[1]*sy<=v['point'][1]<(b[1]+b[3])*sy]
 assert [v['event']for v in actual]==([]if name=='dropped'else['down','up','down','up']if name=='duplicate'else['down','up']),actual

 if expected==0:
  assert data['play_phase']==27 and [v['tick']for v in data['play_records']if v['case']=='step']==[baseline+1]
  assert any(v['tick']==baseline+1 and not v['pending']for v in observed)
  pixels=[v for v in data['play_records']if v['case']=='play-pixels'];assert len(pixels)==6 and all(v['passed']and len(v['checks'])==6 and all(c['passed']for c in v['checks'])for v in pixels)
  if name=='delayed':assert observed[0]['tick']==baseline and not observed[0]['pending']
 elif name=='dropped':
  assert data['play_phase']==6 and 'play timeout phase 6'in data['error']and all(v['tick']==baseline and not v['pending']for v in observed)
 else:
  assert data['play_phase']==6 and 'Single Step must advance exactly one actual Auto tick'in data['error']and any(v['tick']==baseline+2 for v in observed)
 assert hashlib.sha256(exe.read_bytes()).hexdigest()==identity
print('PASS native Step acknowledgment regression '+str(len(rows)),flush=True)
