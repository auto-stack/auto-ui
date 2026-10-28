"""Verify the committed PLAN-860 source closure, including text transport rules."""
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
try:
 identity=json.loads((root/'research/plan860-identity.json').read_text())
except Exception as e:
 raise AssertionError(f'source set drift: identity unreadable: {e}')
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
try:
 sdk_pin=json.loads((root/'research/baseline.json.txt').read_text())['sdk']['repository_commit']
except Exception as e:
 raise AssertionError(f'source set drift: SDK baseline unreadable: {e}')
assert identity['sdk_commit']==sdk_pin,'SDK artifact pin mismatch'
assert identity['base_commit']=='81b0974ac2512126790b0ef2af79235edf45b485','PLAN-860 base is the 857 delivery commit'
for probe in ['g2_route_a.at','g2_route_b.at','g3_route_a.at','g3_route_b.at','character_probe_unit.at']:
 assert f'autographics-auto-driver/research/game-runtime/{probe}'in identity['files'],f'probe source unlisted: {probe}'
print(f"PASS PLAN-860 source/asset identity: {len(identity['files'])} hashes; probe unit and route sources bound; corrected SDK frozen")

if '--controls' in sys.argv:
 paths=[root/'src/research/game_runtime.rs',root/'src/bin/autographics-game-probe.rs',root/'research/game-runtime/character_probe_unit.at',root/'research/plan860-identity.json',root/'research/baseline.json.txt']
 original=[p.read_bytes()for p in paths]
 added=root/'src/research/unlisted_control.rs'
 assert not added.exists()
 def rejected():
  result=subprocess.run([sys.executable,__file__],capture_output=True)
  assert result.returncode!=0 and any(v in result.stderr for v in [b'source hash mismatch',b'missing/outside source',b'source set drift',b'SDK artifact pin mismatch']),result.stderr
 try:
  for i,p in enumerate(paths):
   p.unlink();rejected();p.write_bytes(original[i])
  added.write_text('// unlisted source');rejected();added.unlink()
  d=json.loads(original[3]);d['files']['autographics-auto-driver/src/research/game_runtime.rs']='0'*64
  pathlib.Path(paths[3]).write_text(json.dumps(d));rejected();paths[3].write_bytes(original[3])
  d=json.loads(original[4]);d['sdk']['repository_commit']='0'*40
  pathlib.Path(paths[4]).write_text(json.dumps(d));rejected();paths[4].write_bytes(original[4])
 finally:
  added.unlink(missing_ok=True)
  for p,b in zip(paths,original):p.write_bytes(b)
 subprocess.run([sys.executable,__file__],check=True)
 print('PASS probe source deletion, unlisted source, wrong identity hash/SDK pin and restored control')
