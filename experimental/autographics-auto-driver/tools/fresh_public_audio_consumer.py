"""Export A, move to B, cold-prepare pinned SDK, independently compile/run offline.

No producer target/.sdk-source or Git checkout is copied. Destination must be new
and outside the producer. Evidence remains external to the source packages.
"""
import argparse,json,os,pathlib,re,subprocess,sys
import check_public_audio as gate
import plan012_source_policy as p
def invoke(command,cwd,log,env):
    result=subprocess.run(command,cwd=cwd,env=env,capture_output=True,text=True)
    out=result.stdout+result.stderr;log.write_text(out,encoding='utf-8');assert result.returncode==0,out[-6000:]
    return out
def run(directory):
    directory=directory.resolve();assert not directory.exists() and p.ROOT.resolve() not in directory.parents,'fresh destination must be new/outside producer'
    directory.mkdir(parents=True)
    env=dict(os.environ,CARGO_TERM_COLOR='never',RUST_MIN_STACK='16777216');env.pop('CARGO_TARGET_DIR',None);env.pop('RUSTC_WRAPPER',None)
    a=directory/'a';b=directory/'b'
    invoke([sys.executable,str(p.ROOT/'tools/package_consumer.py'),str(a),'--snapshot-only'],p.ROOT,directory/'export.log',env)
    a.rename(b);assert not a.exists()
    root=b/'experimental/autographics-auto-driver'
    assert not(root/'.sdk-source').exists() and not(root/'target').exists(),'cold SDK/target prerequisite'
    invoke([sys.executable,'tools/prepare_sdk.py','--from-snapshot'],root,directory/'prepare.log',env)
    invoke([sys.executable,'tools/verify_source.py'],root,directory/'source.log',env)
    env['UI012_REPORT_DIR']=str(directory)
    out=invoke(['cargo','test','--locked','--offline','--test','public_audio_consumer','--','--test-threads=1','--nocapture'],root,directory/'consumer-run.log',env)
    gate.counts('consumer',out);gate.data(directory,root.resolve())
    m=re.search(r'Running [^\r\n]*\(([^\r\n()]+)\)',out);assert m,'actual fresh executable missing'
    binary=pathlib.Path(m[1]);binary=(root/binary).resolve() if not binary.is_absolute() else binary.resolve()
    assert(root/'target').resolve() in binary.parents and gate.stack(binary)==33554432,'foreign fresh target/stack'
    v=dict(schema='autographics.plan012.fresh/1',source_identity=gate.identity(),manifest_dir=str(root.resolve()),binary=str(binary),binary_sha256=gate.digest(binary),worker_stack=16777216,cold_sdk=True,cold_target=True,moved_a_to_b=True,logs={n:gate.digest(directory/(n+'.log')) for n in ['export','prepare','source','consumer-run']},data={n:gate.digest(directory/(n+'.json')) for n in ['consumer','performance']})
    (directory/'manifest.json').write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8');validate(directory)
def validate(directory):
    directory=directory.resolve();v=json.loads((directory/'manifest.json').read_text())
    assert v['schema']=='autographics.plan012.fresh/1' and v['source_identity']==gate.identity(),'stale fresh consumer'
    root=directory/'b/experimental/autographics-auto-driver';assert pathlib.Path(v['manifest_dir']).resolve()==root.resolve() and not(directory/'a').exists(),'fresh relocation'
    assert p.ROOT.resolve() not in root.parents and all(v[k] is True for k in ['cold_sdk','cold_target','moved_a_to_b']),'fresh prerequisite'
    for n,h in v['logs'].items():assert gate.digest(directory/(n+'.log'))==h,'fresh log changed'
    assert set(v['logs'])=={'export','prepare','source','consumer-run'} and set(v['data'])=={'consumer','performance'},'partial fresh proof'
    for n,h in v['data'].items():assert gate.digest(directory/(n+'.json'))==h,'fresh data changed'
    assert gate.digest(root/'research/plan012-identity.json')==gate.identity(),'foreign exported source identity'
    env=dict(os.environ);env.pop('CARGO_TARGET_DIR',None)
    result=subprocess.run([sys.executable,'tools/verify_source.py'],cwd=root,env=env,capture_output=True,text=True);assert result.returncode==0,'fresh source/SDK changed: '+result.stderr[-2000:]
    gate.counts('consumer',(directory/'consumer-run.log').read_text(encoding='utf-8'));gate.data(directory,root.resolve())
    binary=pathlib.Path(v['binary']).resolve();assert(root/'target').resolve() in binary.parents and binary.name.startswith('public_audio_consumer-'),'foreign fresh executable'
    assert gate.digest(binary)==v['binary_sha256'] and gate.stack(binary)==33554432 and v['worker_stack']==16777216,'changed fresh executable/stack'
    print('PASS PLAN-012 independent consumer: A→B, cold SDK/target, offline2tests, both profiles/source-sensitive PCM/native/performance')
if __name__=='__main__':
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('destination',type=pathlib.Path);ap.add_argument('--validate',action='store_true');a=ap.parse_args()
    validate(a.destination) if a.validate else run(a.destination)
