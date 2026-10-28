"""Verify the committed PLAN-848 source closure, including text transport rules."""
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
identity=json.loads((root/'research/plan848-identity.json').read_text())
for relative,expected in identity['files'].items():
 p=(base/relative).resolve();assert base.resolve()in p.parents and p.is_file(),f'missing/outside source: {relative}'
 assert not p.is_symlink()and not p.is_junction(),f'linked source: {relative}'
 b=p.read_bytes()
 if p.suffix.lower()in identity['text_suffixes']or p.name.lower()in identity['text_suffixes']:b=b.replace(b'\r\n',b'\n')
 assert hashlib.sha256(b).hexdigest()==expected,f'source hash mismatch: {relative}'
for folder in identity['folders']:
 actual={p.relative_to(base).as_posix()for p in (base/folder).rglob('*')if p.is_file()and '__pycache__'not in p.parts}
 expected={p for p in identity['files']if p.startswith(folder+'/')}
 assert actual==expected,f'source set drift: {folder}: {actual^expected}'
for scene in list((root/'samples/behavior_projects').glob('*/scene.agscene'))+list((root/'samples/spatial_projects').glob('*/scene.agscene'))+list((root/'samples/segment_projects').glob('*/scene.agscene')):
 d=json.loads(scene.read_text());ref=d['behavior']['source'];blob=scene.parent/'scripts'/(ref['sha256']+'.at');raw=blob.read_bytes()
 assert len(raw)==ref['byte_length']and hashlib.sha256(raw).hexdigest()==ref['sha256'],f'raw project source identity: {blob}'
assert identity['sdk_commit']==json.loads((root/'research/baseline.json.txt').read_text())['sdk']['repository_commit'],'SDK artifact pin mismatch'
print(f"PASS PLAN-848 source/asset identity: {len(identity['files'])} hashes; corrected SDK frozen; .agscene LF/CRLF transport equivalent")

if '--controls' in sys.argv:
 paths=[root/'api/behavior_segment_v3.at',root/'research/baseline.json.txt',root/'research/source-lock.json',root/'samples/segment_projects/sight3d/assets/42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671.glb']
 original=[p.read_bytes()for p in paths]
 try:
  for i,p in enumerate(paths):
   if p.suffix=='.glb':p.unlink()
   elif p.name=='baseline.json.txt':
    data=json.loads(original[i]);data['sdk']['repository_commit']='932ad4877e5968f1aa7227d9dcd7fb69b4c69a21';p.write_text(json.dumps(data))
   elif p.name=='source-lock.json':
    data=json.loads(original[i]);data['files']['crates/auto-val/src/nano_value.rs']='0'*64;p.write_text(json.dumps(data))
   else:p.write_bytes(original[i]+bytes([10])+b'// controlled source drift'+bytes([10]))
   r=subprocess.run([sys.executable,__file__],capture_output=True);assert r.returncode!=0 and (b'source hash mismatch'in r.stderr or b'missing/outside source'in r.stderr),f'control accepted or wrong oracle: {p}: rc={r.returncode} stdout={r.stdout!r} stderr={r.stderr!r}'
   p.write_bytes(original[i])
 finally:
  for p,b in zip(paths,original):p.write_bytes(b)
 subprocess.run([sys.executable,__file__],check=True)
 print('PASS new API / stale SDK pin / source-lock drift / missing project GLB controls and restored accepted control')
