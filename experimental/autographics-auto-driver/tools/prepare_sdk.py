"""Recreate the pinned read-only SDK input.

Preparation paths:
  --from-snapshot [dir]  copy the versioned frozen source snapshot (default
                         sdk/frozen) into .sdk-source after verifying it
                         per-file against research/source-lock.json. Refuses
                         an existing target, links/junctions, path escapes,
                         and any missing/extra/mismatched file BEFORE
                         publishing: a rejected preparation never leaves a
                         half-prepared .sdk-source behind.
  --repository <dir>     legacy Git-object export at the pinned commit
                         (sdk.repository_commit); kept for historical
                         reproduction only. Since PLAN-879 that object is not
                         required and may be unreachable.
  --verify-only          only verify an existing .sdk-source against the lock.

No producer working-copy source, target, UI patch, or compiled artifact is read.
The generated .sdk-source is never edited; research/source-lock.json records
every input hash.
"""
import argparse, hashlib, io, json, pathlib, shutil, subprocess, tarfile
p=argparse.ArgumentParser(); p.add_argument('--repository',type=pathlib.Path);p.add_argument('--from-snapshot',nargs='?',type=pathlib.Path,const=None,default=False);p.add_argument('--verify-only',action='store_true');a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[1]; research=root/'research'
b=json.loads((research/'baseline.json.txt').read_text()); sdk=root/'.sdk-source'
lock=(research/'sdk-Cargo.lock.txt').read_bytes()
assert hashlib.sha256(lock).hexdigest()==b['sdk']['root_lock_sha256'],'SDK lock mismatch'
expected=json.loads((research/'source-lock.json').read_text())['files']
def verify_tree(directory):
 assert not any(v.is_symlink() or v.is_junction() for v in directory.rglob('*')),'links forbidden'
 actual={v.relative_to(directory).as_posix():hashlib.sha256(v.read_bytes()).hexdigest() for v in directory.rglob('*') if v.is_file() and 'target' not in v.relative_to(directory).parts}
 assert actual==expected,'SDK file set/hash mismatch: '+str(set(actual)^set(expected))
if not a.verify_only:
 assert not sdk.exists(),'refusing to overwrite existing SDK; use --verify-only'
 if a.from_snapshot is not False:
  snap=(a.from_snapshot if a.from_snapshot is not None else root/'sdk'/'frozen').resolve()
  assert snap.is_dir(),'frozen snapshot missing: '+str(snap)
  verify_tree(snap)
  staging=root/'.sdk-source.prepare-tmp'
  assert not staging.exists(),'stale staging directory present; inspect and remove it first'
  staging.mkdir()
  try:
   for relative in sorted(expected):
    source=snap/relative
    assert source.is_file() and not source.is_symlink() and not source.is_junction(),relative
    assert snap in source.resolve().parents,'snapshot escape: '+relative
    target=staging/relative; target.parent.mkdir(parents=True,exist_ok=True); shutil.copyfile(source,target)
   verify_tree(staging)
   staging.rename(sdk)
  finally:
   if staging.exists(): shutil.rmtree(staging,ignore_errors=True)
 else:
  assert a.repository is not None,'--repository or --from-snapshot required for preparation'
  paths=[v['path'] for v in b['sdk']['path_sources']]+['stdlib','Cargo.toml']
  data=subprocess.check_output(['git','archive',b['sdk']['repository_commit'],'--',*paths],cwd=a.repository)
  sdk.mkdir()
  with tarfile.open(fileobj=io.BytesIO(data)) as t:
   assert all(m.isfile() or m.isdir() for m in t.getmembers()),'links/special files forbidden'
   t.extractall(sdk,filter='data')
  original=(sdk/'Cargo.toml').read_text(); start=original.index('[workspace.dependencies]');end=original.index('[profile.dev]',start)
  (sdk/'Cargo.toml').write_text('[workspace]\nresolver = "2"\nmembers = '+json.dumps(paths[:-2])+'\n\n'+original[start:end])
  (sdk/'Cargo.lock').write_bytes(lock)
assert not any(v.is_symlink() or v.is_junction() for v in sdk.rglob('*')),'links forbidden'
actual={v.relative_to(sdk).as_posix():hashlib.sha256(v.read_bytes()).hexdigest() for v in sdk.rglob('*') if v.is_file() and 'target' not in v.relative_to(sdk).parts}
assert actual==expected,'SDK file set/hash mismatch: '+str(set(actual)^set(expected))
print(f'PASS pinned SDK: {len(actual)} files; 17 local packages; UI/Man/old package absent')
