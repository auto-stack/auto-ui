"""Verify PLAN-837 sources/assets; all old SDK and registry checks stay in verify_source."""
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1]
base=root.parent
identity=json.loads((root/'research/plan837-identity.json').read_text())
for relative,digest in identity['files'].items():
 p=(base/relative).resolve()
 assert base.resolve() in p.parents and p.is_file(),f'missing/outside source: {relative}'
 assert not p.is_symlink() and not p.is_junction(),f'linked source: {relative}'
 b=p.read_bytes()
 if p.suffix.lower() in identity['text_suffixes'] or p.name.lower() in identity['text_suffixes']:b=b.replace(b'\r\n',b'\n')
 assert hashlib.sha256(b).hexdigest()==digest,f'source hash mismatch: {relative}'
for subtree in ['autographics-auto-driver/src','autographics-auto-driver/tests','autographics-auto-driver/samples','autographics-prototype/src','autographics-prototype/tests','autographics-prototype/assets','autographics-prototype/testdata']:
 actual={p.relative_to(base).as_posix() for p in (base/subtree).rglob('*') if p.is_file()}
 expected={p for p in identity['files'] if p.startswith(subtree+'/')}
 assert actual==expected,f'source set drift: {subtree}: {actual^expected}'
print(f"PASS PLAN-837 source/asset identity: {len(identity['files'])} hashes; SDK unchanged")

if '--controls' in __import__('sys').argv:
 import subprocess,sys
 source=root/'samples/authoring_hierarchy/main.at'
 fixture=root/'samples/authoring_hierarchy/example.agscene'
 asset=root.parent/'autographics-prototype/assets/squash_the_creeps/art/player.glb'
 original=source.read_bytes();saved=fixture.read_bytes();saved_asset=asset.read_bytes()
 try:
  source.write_bytes(original+b'\n// controlled source drift\n')
  result=subprocess.run([sys.executable,__file__],capture_output=True)
  assert result.returncode!=0,'source drift was not rejected'
  source.write_bytes(original)
  fixture.unlink()
  result=subprocess.run([sys.executable,__file__],capture_output=True)
  assert result.returncode!=0,'missing fixture was not rejected'
  fixture.write_bytes(saved);asset.unlink()
  result=subprocess.run([sys.executable,__file__],capture_output=True)
  assert result.returncode!=0,'missing original GLB was not rejected'
 finally:
  source.write_bytes(original);fixture.write_bytes(saved);asset.write_bytes(saved_asset)
 subprocess.run([sys.executable,__file__],check=True)
 print('PASS permanent source-drift/missing-fixture/missing-GLB controls; restored accepted control')
