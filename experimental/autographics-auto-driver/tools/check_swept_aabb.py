"""Sweep-specific native cases; source-only mutation, same already-built binary."""
import argparse,hashlib,json,pathlib,shutil,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);p.add_argument('--executable',type=pathlib.Path,default=root/'target/debug/autographics-authoring.exe');p.add_argument('--cases',required=True);a=p.parse_args()
names=a.cases.split(',');allowed={'dash2d','dash3d','dash2d-short','dash3d-short','dash2d-bind','dash3d-bind','partial','transparent','missing-part','wrong-asset','missing-code','bad-code','bad-api','missing-glb'}
assert names and len(set(names))==len(names)and set(names)<=allowed
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);exe=a.executable.resolve();binary=hashlib.sha256(exe.read_bytes()).hexdigest();results=[]
for name in names:
 case=out/name;case.mkdir();mode='dash2d'if name.startswith('dash2d')else'dash3d';space='2'if mode=='dash2d'else'3';original=case/'original';shutil.copytree(root/'samples/sweep_projects'/mode,original);project=case/'moved';original.rename(project);assert not original.exists()
 scene=project/'scene.agscene';d=json.loads(scene.read_text());blob=project/'scripts'/(d['behavior']['source']['sha256']+'.at');raw=blob.read_bytes();source_hash=hashlib.sha256(raw).hexdigest()
 command=[str(exe),'--sweep-check','--spatial-space',space,'--open',str(scene),'--report',str(case/'report.json')]
 if name.endswith('-bind'):
  d['behavior']=None;scene.write_text(json.dumps(d));user=project/'new.at';user.write_bytes(raw)
  export=case/'export';export.mkdir();command+=['--behavior',str(user),'--behavior-api','v4','--sweep-update','--sweep-saveas',str(export/'scene.agscene')]
 if name.endswith('-short'):
  changed=raw.replace(b'var speed=100.0',b'var speed=50.0') if space=='2' else raw.replace(b'var speed=10.0',b'var speed=5.0');changed=changed.replace(b'var clearance=0.02',b'var clearance=0.04') if space=='2' else changed.replace(b'var clearance=0.002',b'var clearance=0.004');assert changed!=raw
  user=case/'changed.at';user.write_bytes(changed);command+=['--behavior',str(user),'--behavior-api','v4','--spatial-narrow','--sweep-update'];source_hash=hashlib.sha256(changed).hexdigest()
 if name=='partial':command+=['--frames','1']
 if name in {'transparent','missing-part','wrong-asset'}:command+=['--play-gpu-fault',name]
 if name=='missing-code':blob.unlink()
 if name=='bad-code':blob.write_bytes(b'broken')
 if name=='bad-api':d['behavior']['api']='autographics.behavior/999';scene.write_text(json.dumps(d))
 if name=='missing-glb':next((project/'assets').glob('*.glb')).unlink()
 r=subprocess.run(command,cwd=case,capture_output=True,timeout=180);(case/'stdout.txt').write_bytes(r.stdout);(case/'stderr.txt').write_bytes(r.stderr);report=json.loads((case/'report.json').read_text());success=name.startswith('dash2d') or name.startswith('dash3d')
 assert (r.returncode==0)==success and report['passed']==success,(name,r.returncode,report.get('error'))
 if success:
  assert report['spatial_phase']==14
  records=report['spatial_records'];pixels=[v for v in records if v['case']=='spatial-pixels'];assert len(pixels)==3 and all(v['passed']and all(c['passed']for c in v['checks'])for v in pixels)
  traces=[v for v in records if v['case']=='sweep-trace'];assert len(traces)==4 and all(v['api']=='autographics.behavior/4'and v['source']==source_hash for v in traces)
  assert all('expected' in v and v['tick']>=0 for v in traces)
  if name.endswith('-bind'):
   assert any(v['case']=='spatial-update-saved'for v in records)
   assert out in project.resolve().parents and out in export.resolve().parents
   relocated=case/'relocated';export.rename(relocated);retired=case/'retired-source-root';project.rename(retired)
   assert not project.exists()and not export.exists();scene=relocated/'scene.agscene'
   r2=subprocess.run([str(exe),'--sweep-check','--spatial-space',space,'--open',str(scene),'--report',str(case/'reopened.json')],cwd=case,capture_output=True,timeout=180)
   (case/'reopened.stdout.txt').write_bytes(r2.stdout);(case/'reopened.stderr.txt').write_bytes(r2.stderr);rr=json.loads((case/'reopened.json').read_text());assert r2.returncode==0 and rr['passed']and rr['spatial_phase']==14,(name,'reopened',r2.returncode,rr.get('error'))
  if name.endswith('-short'):
   assert any(v['case']=='spatial-update-saved'for v in records);bound=json.loads(scene.read_text());assert bound['behavior']['source']['sha256']==source_hash
   user.unlink();command2=[str(exe),'--sweep-check','--spatial-space',space,'--spatial-narrow','--open',str(scene),'--report',str(case/'reopened.json')];r2=subprocess.run(command2,cwd=case,capture_output=True,timeout=180);(case/'reopened.stdout.txt').write_bytes(r2.stdout);(case/'reopened.stderr.txt').write_bytes(r2.stderr);rr=json.loads((case/'reopened.json').read_text());assert r2.returncode==0 and rr['passed']and rr['spatial_phase']==14,(name,'reopened',r2.returncode,rr.get('error'))
 elif name in {'transparent','missing-part','wrong-asset'}:
  assert report['spatial_phase']==5 and 'spatial pixels final-content failure'in report['error']
  assert any(v['case']=='spatial-pixels'and v['stage']=='after-exit'and not v['passed']for v in report['spatial_records'])
 elif name=='partial':assert 'spatial check incomplete'in report['error']
 else:
  error=report['error'];assert error and report['spatial_phase']==0
  if name=='missing-code':assert error.startswith('io:')and blob.name in error
  elif name=='bad-code':assert error.startswith('behavior source closure identity:')and blob.name in error
  elif name=='bad-api':assert error=='unsupported behavior API'
  elif name=='missing-glb':assert error.startswith('io:')and(d['assets'][0]['sha256']+'.glb')in error
 assert hashlib.sha256(exe.read_bytes()).hexdigest()==binary
 item=dict(case=name,command=command,exit=r.returncode,binary_sha256=binary,user_sha256=source_hash,report='report.json',phase=report['spatial_phase']);results.append(item);print(json.dumps(item),flush=True)
(out/'results.json').write_text(json.dumps(results,indent=2))
