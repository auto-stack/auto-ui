"""PLAN-839 native play acceptance. New output directory; negatives reach a named oracle."""
import argparse,hashlib,json,pathlib,subprocess,time
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=pathlib.Path);p.add_argument('--executable',type=pathlib.Path);a=p.parse_args()
dest=a.output.resolve();dest.mkdir(parents=True,exist_ok=False)
exe=(a.executable or root/'target/debug/autographics-authoring.exe').resolve();results=[]
for name,args,success,reason in [
 ('positive',[],True,None),('wrong-focus',['--wrong-play-focus'],False,'wrong-focus'),
 ('drop-tick',['--drop-play-tick'],False,'profile/run/tick/count'),
 ('transparent',['--play-gpu-fault','transparent'],False,'actual play endpoint/origin/all-part pixels'),
 ('missing-part',['--play-gpu-fault','missing-part'],False,'actual play endpoint/origin/all-part pixels'),
 ('wrong-gpu-asset',['--play-gpu-fault','wrong-asset'],False,'actual play endpoint/origin/all-part pixels'),
 ('partial',['--frames','1'],False,'incomplete')]:
 case=dest/name;case.mkdir();project=case/'project';project.mkdir();source=case/'owned.glb';source.write_bytes((root/'research/asset-probe/two-parts.glb').read_bytes());report=case/'report.json'
 command=[str(exe),'--play-check','--play-asset',str(source),'--asset-save',str(project/'scene.agscene'),'--report',str(report),*args]
 start=time.monotonic();run=subprocess.run(command,cwd=root,capture_output=True,timeout=90)
 (case/'stdout.txt').write_bytes(run.stdout);(case/'stderr.txt').write_bytes(run.stderr)
 d=json.loads(report.read_text());assert (run.returncode==0)==success,(name,run.returncode,d.get('error'));assert d['passed']==success,(name,d)
 if reason:assert reason in d['error'],(name,d['error'])
 if success:
  assert d['play_phase']==27
  pixels=[v for v in d['play_records']if v['case']=='play-pixels']
  assert {v['stage']for v in pixels}=={'editing','2d-moved','3d-moved','ui-isolated','shown','stopped'} and len(pixels)==6
  assert all(v['passed'] and len(v['checks'])==6 and all(c['passed']for c in v['checks'])for v in pixels)
  traces=[v for v in d['play_records']if v['case']=='tick-trace'];assert len(traces)==3 and all(v['tick']==len(v['trace'])-1 and all(t['tick']==i for i,t in enumerate(v['trace']))for v in traces)
  assert any(v['case']=='v-hidden'for v in d['play_records']) and any(v['case']=='focus-paused'for v in d['play_records'])
  saved=next(v for v in d['play_records']if v['case']=='saved');source.unlink();relocated=case/'relocated';project.rename(relocated)
  child=subprocess.run([str(exe),'--inspect',str(relocated/'scene.agscene')],cwd=dest,capture_output=True,timeout=30);assert child.returncode==0,child.stderr
  (case/'reopen.json').write_bytes(child.stdout);r=json.loads(child.stdout)
  assert r['document']==saved['document'] and r['two']==saved['two'] and r['three']==saved['three'] and r['history']==[0,0,0] and not r['dirty'] and r['selection']is None
 if name in {'transparent','missing-part','wrong-gpu-asset'}:assert any(v['case']=='play-pixels'and not v['passed']for v in d['play_records'])
 results.append(dict(case=name,command=command,exit=run.returncode,passed=d['passed'],phase=d['play_phase'],frames=d['frames'],error=d['error'],seconds=round(time.monotonic()-start,3)))
(dest/'results.json').write_text(json.dumps(dict(executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),cases=results,passed=True),indent=2)+'\n')
print(json.dumps(results,indent=2))
