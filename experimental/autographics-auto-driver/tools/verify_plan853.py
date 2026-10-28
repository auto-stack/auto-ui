"""Verify the committed PLAN-853 source closure, including text transport rules."""
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
identity=json.loads((root/'research/plan853-identity.json').read_text())
# Cargo discovers build.rs, examples/benches and package-local .cargo config
# outside src/. Close package roots as well as the recursive authored folders.
# Generated SDK/build outputs and research captures retain their existing homes.
assert not (base/'.cargo').exists(), 'source set drift: shared experimental .cargo configuration'
for package,allowed_dirs in {
 'autographics-auto-driver':{'api','research','samples','src','tests','tools','target','.sdk-source'},
 'autographics-prototype':{'assets','research','src','testdata','tests','target'},
}.items():
 directory=base/package
 expected={p for p in identity['files']if pathlib.PurePosixPath(p).parent.as_posix()==package}
 assert not any(p.is_symlink()or p.is_junction()for p in directory.iterdir()),f'linked source: package root {package}'
 actual={p.relative_to(base).as_posix()for p in directory.iterdir()if p.is_file()}
 assert actual==expected,f'source set drift: package root {package}: {actual^expected}'
 unexpected={p.name for p in directory.iterdir()if p.is_dir()}-allowed_dirs
 assert not unexpected,f'source set drift: package directories {package}: {unexpected}'
for relative,expected in identity['files'].items():
 raw=base/relative
 assert not any(v.is_symlink()or v.is_junction()for v in [raw,*raw.parents]if v!=base.parent),f'linked source: {relative}'
 p=raw.resolve();assert base.resolve()in p.parents and p.is_file(),f'missing/outside source: {relative}'
 assert not p.is_symlink()and not p.is_junction(),f'linked source: {relative}'
 b=p.read_bytes()
 if p.suffix.lower()in identity['text_suffixes']or p.name.lower()in identity['text_suffixes']:b=b.replace(b'\r\n',b'\n')
 assert hashlib.sha256(b).hexdigest()==expected,f'source hash mismatch: {relative}'
for folder in identity['folders']:
 actual={p.relative_to(base).as_posix()for p in (base/folder).rglob('*')if p.is_file()and '__pycache__'not in p.parts}
 expected={p for p in identity['files']if p.startswith(folder+'/')}
 assert actual==expected,f'source set drift: {folder}: {actual^expected}'
for scene in list((root/'samples/behavior_projects').glob('*/scene.agscene'))+list((root/'samples/spatial_projects').glob('*/scene.agscene'))+list((root/'samples/segment_projects').glob('*/scene.agscene'))+list((root/'samples/sweep_projects').glob('*/scene.agscene'))+list((root/'samples/motion_projects').glob('*/scene.agscene')):
 d=json.loads(scene.read_text());ref=d['behavior']['source'];blob=scene.parent/'scripts'/(ref['sha256']+'.at');raw=blob.read_bytes()
 assert len(raw)==ref['byte_length']and hashlib.sha256(raw).hexdigest()==ref['sha256'],f'raw project source identity: {blob}'
assert identity['sdk_commit']==json.loads((root/'research/baseline.json.txt').read_text())['sdk']['repository_commit'],'SDK artifact pin mismatch'
print(f"PASS PLAN-853 source/asset identity: {len(identity['files'])} hashes; corrected SDK frozen; .agscene LF/CRLF transport equivalent")

if '--controls' in sys.argv:
 import tempfile
 paths=[root/'src/authoring/measurement.rs',root/'research/baseline.json.txt',root/'research/source-lock.json',root/'samples/motion_projects/corridor3d/assets/42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb']
 original=[p.read_bytes()for p in paths]
 added=root/'src/m2_unlisted_control.rs'
 assert not added.exists()
 def rejected():
  result=subprocess.run([sys.executable,__file__],capture_output=True)
  assert result.returncode!=0 and any(v in result.stderr for v in [b'source hash mismatch',b'missing/outside source',b'source set drift']),result.stderr
 try:
  for i,p in enumerate(paths):
   p.unlink();rejected();p.write_bytes(original[i])
  added.write_text('// unlisted source');rejected();added.unlink()
  for i in [1,2]:
   d=json.loads(original[i])
   if i==1:d['sdk']['repository_commit']='0'*40
   else:d['files']['crates/auto-val/src/nano_value.rs']='0'*64
   paths[i].write_text(json.dumps(d));rejected();paths[i].write_bytes(original[i])
 finally:
  added.unlink(missing_ok=True)
  for p,b in zip(paths,original):p.write_bytes(b)
 subprocess.run([sys.executable,__file__],check=True)
 print('PASS deleted new observer / SDK manifest / SDK lock / GLB, unlisted source, wrong SDK/hash and restored control')
