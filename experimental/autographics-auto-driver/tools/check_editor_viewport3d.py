"""Validate complete native 3D editor probe and reject partial reports."""
import argparse,json,os,pathlib,subprocess
import m2_evidence
p=argparse.ArgumentParser();p.add_argument('output',type=pathlib.Path);p.add_argument('--executable',type=pathlib.Path,required=True);p.add_argument('--controls',action='store_true');p.add_argument('--build-proof',type=pathlib.Path);p.add_argument('--allow-reuse',action='store_true');a=p.parse_args()
assert not a.output.exists(),'fresh output required';a.output.mkdir(parents=True)
runner=m2_evidence.record_runner(a.executable.resolve(),a.output/'runner.json',build_proof=a.build_proof,allow_reuse=a.allow_reuse);exe=pathlib.Path(runner['path'])
if not runner.get('origin_verified'): raise SystemExit('runner build provenance required; use --build-proof')
def run(name,flags,expected):
 report=(a.output/(name+'.json')).resolve();cmd=[str(exe),'--viewport3d-check','--report',str(report),*flags]
 with (a.output/(name+'.log')).open('w') as log:r=subprocess.run(cmd,env=dict(os.environ,RUST_MIN_STACK='16777216'),stdout=log,stderr=log,timeout=120)
 assert report.is_file(),(name,'missing report');d=json.loads(report.read_text());assert d['runner_sha256']==m2_evidence.sha(exe)
 if expected:
  assert r.returncode==0 and d['passed'] and d['error'] is None,(name,d.get('error'))
  assert d['viewport3d_phase']==34,(name,d['viewport3d_phase'])
  rows=d['viewport3d_records'];assert {x['phase'] for x in rows if 'phase'in x}==set(range(34))
  assert {x['case'] for x in rows if 'case'in x}=={'grid-composite','grid-occlusion','auto-drag','save-reopen','focus-cancel','play-restore','cross-button-capture'}
  assert report.with_suffix('.png').is_file(),'missing final composed frame'
 else:assert r.returncode!=0 and not d['passed'] and 'viewport3d incomplete' in d['error'],(name,d.get('error'))
 row={'name':name,'exit':r.returncode,'report':str(report),'sha256':m2_evidence.sha(report)}
 if expected:row['screenshot_sha256']=m2_evidence.sha(report.with_suffix('.png'))
 return row
rows=[run('normal',['--frames','1200'],True)]
if a.controls:rows.append(run('partial',['--frames','1'],False))
(a.output/'results.json').write_text(json.dumps(rows,indent=2)+'\n');print('PASS viewport3d:',len(rows),'actual OS/GPU runs')
