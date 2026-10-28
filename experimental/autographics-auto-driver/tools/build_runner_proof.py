"""Build an authoring runner from a clean checkout and record independently checkable inputs."""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys

import runner_provenance as provenance
ROOT = pathlib.Path(__file__).resolve().parents[1]

def sha(path): return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',required=True,type=pathlib.Path)
    target=p.add_mutually_exclusive_group();target.add_argument('--bin',default=None);target.add_argument('--test',default=None)
    a=p.parse_args();out=a.output.resolve()
    provenance.need(not out.exists() and ROOT not in out.parents, 'fresh external build directory required')
    commit=provenance.current_commit()
    identity=ROOT/'research/plan857-identity.json'
    provenance.need(provenance.committed_identity(commit)==provenance.canonical_sha(identity),'current manifest is not committed')
    env=dict(os.environ);env.pop('CARGO_TARGET_DIR',None);env.pop('PYTHONOPTIMIZE',None)
    for script,flags in [('verify_plan857.py',[]),('prepare_sdk.py',['--verify-only'])]:
        subprocess.run([sys.executable,str(ROOT/'tools'/script),*flags],cwd=ROOT,env=env,check=True)
    out.mkdir(parents=True)
    target=out/'target';events=out/'cargo-artifacts.jsonl';log=out/'cargo-stderr.log'
    target_kind='test' if a.test else 'bin';target_name=a.test or a.bin or 'autographics-authoring'
    command=['cargo','test' if a.test else 'build','--locked','--offline','--'+target_kind,target_name,*(['--no-run'] if a.test else []),'--message-format=json','--target-dir',str(target)]
    with events.open('wb') as stdout,log.open('wb') as stderr:
        result=subprocess.run(command,cwd=ROOT,env=env,stdout=stdout,stderr=stderr,check=False)
    provenance.need(result.returncode==0,'Cargo build failed')
    provenance.need(provenance.current_commit()==commit and provenance.committed_identity(commit)==provenance.canonical_sha(identity),'checkout changed during build')
    records=[json.loads(line) for line in events.read_text(encoding='utf8').splitlines() if line.lstrip().startswith('{')]
    found=[v for v in records if v.get('reason')=='compiler-artifact' and v.get('target',{}).get('name')==target_name and target_kind in v.get('target',{}).get('kind',[]) and v.get('executable')]
    provenance.need(len(found)==1 and any(v.get('reason')=='build-finished' and v.get('success') is True for v in records),'Cargo did not emit completed runner')
    binary=pathlib.Path(found[0]['executable']).resolve()
    provenance.need(target.resolve() in binary.parents and binary.is_file(),'Cargo executable outside target')
    archive=out/'source-identity.json';shutil.copyfile(identity,archive)
    proof={'schema':'autographics.controlled-build/1','commit':commit,'source_identity_path':str(archive),'source_identity_sha256':sha(archive),'sdk_sha256':sha(ROOT/'research/source-lock.json'),'locks':{str(p.relative_to(ROOT.parent)):sha(p) for p in [ROOT/'Cargo.lock',ROOT.parent/'autographics-prototype/Cargo.lock']},'configuration':{k:os.environ.get(k) for k in ['RUSTFLAGS','RUST_MIN_STACK','RUSTC_WRAPPER']},'rustc':subprocess.check_output(['rustc','-vV'],text=True),'cargo':subprocess.check_output(['cargo','--version'],text=True).strip(),'binary_path':str(binary),'binary_sha256':sha(binary),'build_log_path':str(log),'build_log_sha256':sha(log),'cargo_artifacts_path':str(events),'cargo_artifacts_sha256':sha(events),'command':command,'target_kind':target_kind,'target_name':target_name,'clean_before_after':True}
    context_keys=['commit','source_identity_sha256','sdk_sha256','locks','configuration','rustc','cargo','binary_sha256','build_log_sha256','cargo_artifacts_sha256','clean_before_after']
    context=out/'build-context.json';context.write_text(json.dumps({key:proof[key] for key in context_keys},indent=2)+'\n',encoding='utf8')
    proof['build_context_path']=str(context);proof['build_context_sha256']=sha(context)
    path=out/'build-proof.json';path.write_text(json.dumps(proof,indent=2)+'\n',encoding='utf8')
    provenance.verify_build(path,binary,json.loads(identity.read_text()),allow_reuse=False)
    registry=pathlib.Path('D:/autostack/plan-artifacts/plan-857/build-proofs')/proof['binary_sha256']
    registry.mkdir(parents=True,exist_ok=True)
    registration=registry/'proof.json'
    if registration.exists(): provenance.need(registration.read_bytes()==path.read_bytes(),'conflicting build proof registration')
    else: shutil.copyfile(path,registration)
    print(f'PASS controlled build commit={commit} runner={proof["binary_sha256"]} proof={path}')

if __name__=='__main__':
    try:main()
    except (provenance.ProvenanceError,OSError,ValueError,subprocess.CalledProcessError) as e:
        print(f'FAIL: {e}',file=sys.stderr);sys.exit(1)
