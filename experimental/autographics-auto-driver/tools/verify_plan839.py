"""Verify the committed PLAN-839 source closure, including text transport rules."""
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
identity=json.loads((root/'research/plan839-identity.json').read_text())
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
print(f"PASS PLAN-839 source/asset identity: {len(identity['files'])} hashes; SDK unchanged; .agscene LF/CRLF transport equivalent")
if '--controls'in sys.argv:
 paths=[root/'samples/authoring_play/main.at',root/'research/asset-probe/two-parts.glb',base/'autographics-prototype/assets/squash_the_creeps/art/player.glb',root/'Cargo.lock']
 saved=[p.read_bytes()for p in paths]
 fixture=root/'samples/authoring_hierarchy/example.agscene';fixture_bytes=fixture.read_bytes()
 try:
  fixture.write_bytes(fixture_bytes.replace(b'\r\n',b'\n'));subprocess.run([sys.executable,__file__],check=True);fixture.write_bytes(fixture_bytes)
  for i,p in enumerate(paths):
   if i in [1,2]:p.unlink()
   else:p.write_bytes(saved[i]+b'\n// controlled source drift\n')
   bad=subprocess.run([sys.executable,__file__],capture_output=True);assert bad.returncode!=0,f'control was accepted: {p}'
   p.write_bytes(saved[i])
 finally:
  for p,b in zip(paths,saved):p.write_bytes(b)
  fixture.write_bytes(fixture_bytes)
 subprocess.run([sys.executable,__file__],check=True)
 print('PASS LF scene transport, Auto-source/missing-own-GLB/missing-original-GLB/lock-drift controls and restored accepted control')
