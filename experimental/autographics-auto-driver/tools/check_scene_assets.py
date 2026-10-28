"""PLAN-838 actual OS/Iced/Auto/GPU and portable-closure acceptance.
The output directory must be new; every negative must reach its specific oracle.
"""
import argparse,hashlib,json,pathlib,shutil,subprocess,time
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--executable',type=pathlib.Path);a=p.parse_args()
dest=a.output.resolve();dest.mkdir(parents=True,exist_ok=False)
exe=(a.executable or root/'target/debug/autographics-authoring.exe').resolve()
results=[]
for name,args,success,reason in [
 ('positive',[],True,None),('wrong-asset',['--wrong-asset'],False,'wrong-resource'),
 ('missing-part',['--asset-gpu-fault','missing-part'],False,'actual asset part/origin pixels'),
 ('transparent',['--asset-gpu-fault','transparent'],False,'actual asset part/origin pixels'),
 ('wrong-gpu-asset',['--asset-gpu-fault','wrong-asset'],False,'actual asset part/origin pixels'),
 ('drop-release',['--drop-release'],False,'timeout'),('partial',['--frames','1'],False,'incomplete')]:
 case=dest/name;case.mkdir();project=case/'project';project.mkdir();source=case/'owned-input.glb';source.write_bytes((root/'research/asset-probe/two-parts.glb').read_bytes());report=case/'report.json'
 command=[str(exe),'--asset-check','--asset-file',source.as_posix(),'--asset-save',(project/'scene.agscene').as_posix(),'--report',str(report),*args]
 start=time.monotonic();run=subprocess.run(command,cwd=root,capture_output=True,timeout=90)
 (case/'stdout.txt').write_bytes(run.stdout);(case/'stderr.txt').write_bytes(run.stderr)
 d=json.loads(report.read_text());assert (run.returncode==0)==success,(name,run.returncode,d);assert d['passed']==success,(name,d)
 if reason:assert reason in d['error'],(name,d['error'])
 if success:
  assert d['asset_phase']==20
  pixels=[v for v in d['asset_records']if v['case']=='asset-pixels']
  assert len(pixels)==9 and {v['stage']for v in pixels}=={'import','temporary','move','undo','redo','hidden','show','replace','saved'}
  assert all(v['passed'] and len(v['checks'])==4 and all(c['passed']for c in v['checks'])for v in pixels)
  saved=[v for v in d['asset_records']if v['case']=='saved'];assert len(saved)==1;saved=saved[0]
  # Remove the original source and original project root before the new process.
  source.unlink();relocated=case/'relocated';project.rename(relocated);entry=relocated/'scene.agscene'
  child=subprocess.run([str(exe),'--inspect',str(entry)],cwd=dest,capture_output=True,timeout=30);assert child.returncode==0,child.stderr
  (case/'reopen.json').write_bytes(child.stdout);restored=json.loads(child.stdout)
  assert restored['document']==saved['document'] and restored['hash']==saved['hash'] and restored['world']==saved['world']
  assert restored['two']==d['two'] and restored['three']==d['three'] and restored['history']==[0,0,0] and restored['selection']is None and not restored['dirty']
  sha=saved['document']['assets'][0]['sha256'];blob=relocated/'assets'/f'{sha}.glb';original=blob.read_bytes();assert hashlib.sha256(original).hexdigest()==sha
  for failure in ['missing','tampered']:
   if failure=='missing':blob.unlink()
   else:
    changed=bytearray(original);changed[-1]^=1;blob.write_bytes(changed)
   bad=subprocess.run([str(exe),'--inspect',str(entry)],cwd=dest,capture_output=True,timeout=30)
   (case/(failure+'.stderr.txt')).write_bytes(bad.stderr);assert bad.returncode!=0,(failure,bad.stdout)
   blob.write_bytes(original)
  healed=subprocess.run([str(exe),'--inspect',str(entry)],cwd=dest,capture_output=True,timeout=30);assert healed.returncode==0 and json.loads(healed.stdout)['three']==d['three']
 if name=='wrong-asset':assert d['document']['assets']==[] and d['document']['objects']==[] and d['resources']['mesh_count']==7
 if name in {'missing-part','transparent','wrong-gpu-asset'}:
  pixel_failures=[v for v in d['asset_records']if v['case']=='asset-pixels'and not v['passed']];assert pixel_failures and d['asset_phase']==7
 results.append(dict(case=name,args=command,exit=run.returncode,passed=d['passed'],phase=d['asset_phase'],frames=d['frames'],error=d['error'],seconds=round(time.monotonic()-start,3)))
(dest/'results.json').write_text(json.dumps({'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'cases':results,'passed':True},indent=2)+'\n')
print(json.dumps(results,indent=2))
