"""Project-specific native OS/GPU gate; consumes one already built runner."""
import argparse,hashlib,json,pathlib,shutil,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);p.add_argument('--executable',type=pathlib.Path,default=root/'target/debug/autographics-authoring.exe');p.add_argument('--cases',required=True);a=p.parse_args()
names=a.cases.split(',');allowed={'bind-move','bind-patrol','update-fast','move','patrol','fast','partial','transparent','missing-part','wrong-asset','missing-source','bad-source','bad-actor','missing-glb','override'}
assert names and len(set(names))==len(names) and set(names)<=allowed
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);exe=a.executable.resolve();binary=hashlib.sha256(exe.read_bytes()).hexdigest();results=[]
for name in names:
 case=out/name;case.mkdir();mode='patrol' if name.endswith('patrol') else 'move';project=case/'original';shutil.copytree(root/'samples/behavior_projects'/mode,project)
 command=[str(exe),'--behavior-check','--open',str(project/'scene.agscene'),'--behavior-reference',mode,'--report',str(case/'report.json')]
 d=json.loads((project/'scene.agscene').read_text());blob=project/d['behavior']['source']['relative'] if 'relative' in d['behavior']['source'] else project/'scripts'/(d['behavior']['source']['sha256']+'.at')
 if name.startswith('bind-')or name=='update-fast':
  original=root/'samples/user_behavior'/mode/'main.at';source=project/'main.at';source.write_bytes(original.read_bytes());
  if name=='update-fast':source.write_bytes(source.read_bytes().replace(b'speed=60.0',b'speed=90.0'));command[command.index('--behavior-reference')+1]='move-fast'
  else:d['schema']='autographics.scene/3';del d['behavior'];(project/'scene.agscene').write_text(json.dumps(d));shutil.rmtree(project/'scripts')
  export=project/'export';export.mkdir();command+=['--behavior','main.at','--project-bind-check','--project-saveas',str(export/'scene.agscene')]
 else:
  moved=case/'moved';project.rename(moved);assert not project.exists();project=moved;command[command.index('--open')+1]=str(project/'scene.agscene');blob=project/'scripts'/(d['behavior']['source']['sha256']+'.at')
 if name=='fast':
  raw=blob.read_bytes();assert raw.count(b'speed=60.0')==1;raw=raw.replace(b'speed=60.0',b'speed=90.0');h=hashlib.sha256(raw).hexdigest();new=blob.with_name(h+'.at');new.write_bytes(raw);blob.unlink();blob=new;d['behavior']['source']['sha256']=h;d['behavior']['source']['byte_length']=len(raw);(project/'scene.agscene').write_text(json.dumps(d));command[command.index('--behavior-reference')+1]='move-fast'
 if name=='partial':command+=['--frames','1']
 if name in ['transparent','missing-part','wrong-asset']:command+=['--play-gpu-fault',name]
 if name=='missing-source':blob.unlink()
 if name=='bad-source':blob.write_bytes(b'broken')
 if name=='bad-actor':d['behavior']['actors']['two_d']=11;(project/'scene.agscene').write_text(json.dumps(d))
 if name=='missing-glb':next((project/'assets').glob('*.glb')).unlink()
 if name=='override':
  external=case/'override.at';external.write_bytes((root/'samples/user_behavior/patrol/main.at').read_bytes());command+=['--behavior',str(external)];command[command.index('--behavior-reference')+1]='patrol'
 before=(project/'scene.agscene').read_bytes();run=subprocess.run(command,cwd=case,capture_output=True,timeout=120);(case/'stdout.txt').write_bytes(run.stdout);(case/'stderr.txt').write_bytes(run.stderr)
 report=json.loads((case/'report.json').read_text());success=name in {'bind-move','bind-patrol','update-fast','move','patrol','fast','override'}
 assert (run.returncode==0)==success and report['passed']==success,(name,run.returncode,report.get('error'))
 if success:
  if name.startswith('bind-')or name=='update-fast':project=project/'export'
  assert report['play_phase']==27
  pixels=[r for r in report['play_records'] if r['case']=='play-pixels'];assert len(pixels)==6 and all(v['passed'] and len(v['checks'])==6 and all(c['passed']for c in v['checks'])for v in pixels)
  traces=[r for r in report['play_records']if r['case']=='tick-trace'];assert len(traces)>=3
  if name.startswith('bind-')or name=='update-fast':
   bound=json.loads((project/'scene.agscene').read_text());blob=project/'scripts'/(bound['behavior']['source']['sha256']+'.at')
  expected=hashlib.sha256((case/'override.at').read_bytes()if name=='override'else blob.read_bytes()).hexdigest();assert all(t['behavior']['hash']==expected and pathlib.Path(t['behavior']['path']).samefile(case/'override.at'if name=='override'else blob) for t in traces)
  if name.startswith('bind-')or name=='update-fast':
   bound=json.loads((project/'scene.agscene').read_text());blob=project/'scripts'/(bound['behavior']['source']['sha256']+'.at')
   assert any(r['case']=='project-bound-saved'and r['project_default']for r in report['play_records'])
  else:assert json.loads((project/'scene.agscene').read_text())==json.loads(before)
  reopened=subprocess.run([str(exe),'--inspect',str(project/'scene.agscene')],cwd=case,capture_output=True,timeout=45);assert reopened.returncode==0,reopened.stderr
  inspected=json.loads(reopened.stdout);assert inspected['closure_verified']and inspected['history']==[0,0,0]and not inspected['dirty']and inspected['document']['behavior'] is not None
  (case/'reopen.json').write_bytes(reopened.stdout)
 else:
  reason={'partial':'incomplete','transparent':'actual play endpoint/origin/all-part pixels','missing-part':'actual play endpoint/origin/all-part pixels','wrong-asset':'actual play endpoint/origin/all-part pixels','bad-actor':'behavior actor','missing-source':'.at','bad-source':'behavior source closure identity','missing-glb':'.glb'}[name]
  assert reason in report.get('error',''),(name,report)
 assert hashlib.sha256(exe.read_bytes()).hexdigest()==binary
 result=dict(case=name,command=command,exit=run.returncode,error=report.get('error'));results.append(result);print(json.dumps(result),flush=True);(out/'results.json').write_text(json.dumps(dict(binary=binary,selected=names,completed=len(results),passed=len(results)==len(names),cases=results),indent=2))
