"""Focused lock identity and runner build/reuse provenance controls."""
import argparse,json,os,pathlib,shutil,subprocess,sys
import m2_evidence
import runner_provenance as p
ROOT=pathlib.Path(__file__).resolve().parents[1]

def main():
 q=argparse.ArgumentParser(description=__doc__);q.add_argument('--output',required=True,type=pathlib.Path);q.add_argument('--old-runner',required=True,type=pathlib.Path);q.add_argument('--baseline-proof',type=pathlib.Path);a=q.parse_args()
 out=a.output.resolve();p.need(not out.exists(),'fresh output required');out.mkdir(parents=True)
 old=p.load('D:/autostack/plan-artifacts/plan-856/merge-r1/viewport2d/runner.json')
 current=p.load('D:/autostack/plan-artifacts/plan-857/repair-r2/frozen-native/runner.json')
 old_id=p.load(old['source_identity_path']);current_id=p.load(ROOT/'research/plan857-identity.json')
 good=p.check_locks(old['locks'],current['locks'],old_id,current_id);p.need(set(good)==set(p.LOCKS),'valid lock control')
 rows=[]
 def rejected(name,fn,fragment):
  try:fn()
  except (p.ProvenanceError,AssertionError,ValueError) as e:
   p.need(fragment in str(e),f'{name}: wrong rejection {e}');rows.append({'case':name,'exit':1,'reason':str(e)});return
  raise RuntimeError(f'{name} incorrectly accepted')
 forged={k:m2_evidence.sha(ROOT.parent/k) for k in ['autographics-auto-driver/README.md','autographics-prototype/Cargo.toml']}
 rejected('both-wrong-lock-names',lambda:p.check_locks(forged,forged,old_id,current_id),'unexpected, duplicate or aliased lock path')
 one={next(iter(good)):next(iter(good.values()))}
 rejected('missing-lock',lambda:p.check_locks(one,one,old_id,current_id),'exactly two lock identities required')
 aliases={'autographics-auto-driver/Cargo.lock':good[p.LOCKS[0]],'autographics-auto-driver\\Cargo.lock':good[p.LOCKS[0]]}
 rejected('aliased-locks',lambda:p.check_locks(aliases,aliases,old_id,current_id),'unexpected, duplicate or aliased lock path')
 changed=dict(good);changed[p.LOCKS[0]]='0'*64
 rejected('one-sided-wrong-lock',lambda:p.check_locks(good,changed,old_id,current_id),'lock identity mismatch')
 rejected('both-wrong-lock-values',lambda:p.check_locks(changed,changed,old_id,current_id),'actual lock bytes mismatch')
 duplicate=out/'duplicate.json';duplicate.write_text('{"locks": {"x":1,"x":2}}',encoding='utf8')
 rejected('duplicate-json',lambda:p.load(duplicate),'duplicate JSON key')
 unknown=m2_evidence.record_runner(a.old_runner,out/'unverified-old.json')
 p.need(unknown.get('origin_verified') is False and unknown.get('code_commit') is None and unknown.get('source_identity_sha256') is None,'unverified old runner was attributed')
 rows.append({'case':'old-runner-without-proof','exit':1,'reason':'unverified; no build commit or source claim'})
 fixture=out/'old-home-no-proof';fixture.mkdir();shutil.copyfile(out/'unverified-old.json',fixture/'runner.json')
 original=pathlib.Path('D:/autostack/plan-artifacts/plan-857/repair-r1/committed-native')
 for name in ['normal.json','normal.png']:shutil.copyfile(original/name,fixture/name)
 report=fixture/'normal.json';shot=fixture/'normal.png'
 (fixture/'results.json').write_text(json.dumps([{'name':'normal','exit':0,'report':str(report.resolve()),'sha256':m2_evidence.sha(report),'screenshot_sha256':m2_evidence.sha(shot)}]),encoding='utf8')
 command=[sys.executable,str(ROOT/'tools/check_default_3d_baseline.py'),'--baseline-runner','D:/autostack/plan-artifacts/plan-856/merge-r1/viewport2d/runner.json','--reset-dir',str(fixture),'--commit',p.git('rev-parse','HEAD').decode().strip(),'--output',str(out/'old-home-comparison'),'--allow-reuse']
 result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=120)
 (out/'old-home.stderr').write_text(result.stderr,encoding='utf8')
 p.need(result.returncode!=0 and 'current runner source identity missing' in result.stderr and not (out/'old-home-comparison/summary.json').exists(),'unproven old real Home report was accepted')
 rows.append({'case':'actual-old-home-without-proof','exit':result.returncode,'reason':result.stderr.strip()})
 if a.baseline_proof:
  proof=p.load(a.baseline_proof);old_binary=pathlib.Path(proof['binary_path'])
  rejected('wrong-binary-with-proof',lambda:p.verify_build(a.baseline_proof,a.old_runner,current_id,allow_reuse=True),'runner does not match controlled build')
  forged=dict(proof);forged['commit']=p.current_commit();forged_path=out/'forged-build-commit.json';forged_path.write_text(json.dumps(forged),encoding='utf8')
  rejected('forged-build-commit',lambda:p.verify_build(forged_path,old_binary,current_id,allow_reuse=True),'build context commit mismatch')
  rejected('cross-commit-without-approval',lambda:p.verify_build(a.baseline_proof,old_binary,current_id,allow_reuse=False),'cross-commit runner reuse requires explicit approval')
  attested=m2_evidence.record_runner(old_binary,out/'approved-reuse.json',build_proof=a.baseline_proof,allow_reuse=True)
  p.need(attested['code_commit']==proof['commit'] and attested['verification_commit']==p.current_commit() and attested['build_provenance']['reuse'] is True,'reuse did not retain actual build identity')
  m2_evidence.verify_runner(attested,allow_reuse=True)
  rows.append({'case':'explicit-verified-reuse','exit':0,'build_commit':attested['code_commit'],'verification_commit':attested['verification_commit']})
 (out/'results.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf8')
 print(f'PASS real lock, unproven old runner, and {len(rows)} provenance controls')

if __name__=='__main__':main()
