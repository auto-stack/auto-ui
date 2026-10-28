"""Native hierarchy acceptance: executable must be built with the documented stack.
Usage: python tools/check_hierarchy.py [output-directory]
No direct Host dispatch; each process uses its OS/Iced self-check.
"""
import argparse,json,pathlib,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',nargs='?',type=pathlib.Path);parser.add_argument('--executable',type=pathlib.Path);a=parser.parse_args()
dest=a.output.resolve() if a.output else root/'research'/'hierarchy-native-check'
dest.mkdir(parents=True,exist_ok=True)
exe=root/'target/debug/autographics-authoring.exe'
if a.executable:exe=a.executable.resolve()
results=[]
for name,args,success,reason in [
 ('positive',['--hierarchy-check'],True,None),
 ('wrong-parent',['--hierarchy-check','--wrong-parent'],False,'wrong parent'),
 ('drop-release',['--hierarchy-check','--drop-release'],False,'timeout'),
 ('partial',['--hierarchy-check','--frames','1'],False,'incomplete'),
]:
 report=dest/(name+'.json');start=time.monotonic()
 p=subprocess.run([str(exe),*args,'--report',str(report)],cwd=root,capture_output=True,timeout=90)
 (dest/(name+'.stdout.txt')).write_bytes(p.stdout);(dest/(name+'.stderr.txt')).write_bytes(p.stderr)
 d=json.loads(report.read_text());assert (p.returncode==0)==success,(name,p.returncode,d)
 assert d['passed']==success,(name,d)
 if reason:assert reason in d['error'],(name,d['error'])
 else:
  assert d['hierarchy_phase']==16
  records=d['hierarchy_records'];pixels=[r for r in records if r['case']=='composed-pixels']
  assert len(pixels)==7 and {r['stage'] for r in pixels}=={'temporary','move','undo','redo','hidden','show','saved'}
  for r in pixels:
   assert r['passed'] and r['color'] is not None and r['background'] is not None and r['color']!=r['background'],r
   assert r['origin']==r['expected_origin'] and r['endpoint']==r['expected_endpoint'],r
  saved_records=[r for r in records if r['case']=='saved'];assert len(saved_records)==1
  saved=saved_records[0]
  p2=subprocess.run([str(exe),'--inspect',saved['path']],cwd=root,capture_output=True,timeout=30)
  assert p2.returncode==0,p2.stderr
  restored=json.loads(p2.stdout);(dest/'reopen.json').write_bytes(p2.stdout)
  assert restored['document']==saved['document'] and restored['hash']==saved['hash']
  assert restored['world']==saved['world'] and restored['two']==d['two'] and restored['three']==d['three']
  assert restored['history']==[0,0,0] and restored['selection'] is None and not restored['dirty']
 results.append(dict(case=name,args=args,exit=p.returncode,seconds=round(time.monotonic()-start,3),phase=d['hierarchy_phase'],frames=d['frames'],error=d['error']))
(dest/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
