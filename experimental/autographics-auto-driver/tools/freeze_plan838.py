"""Work-owned source freeze; never invoked implicitly by verification."""
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
old=json.loads((root/'research/plan837-identity.json').read_text())
files=set(old['files'])|{'autographics-auto-driver/research/plan837-identity.json'}
folders=['autographics-auto-driver/src','autographics-auto-driver/tests','autographics-auto-driver/samples','autographics-auto-driver/tools','autographics-prototype/src','autographics-prototype/tests','autographics-prototype/assets','autographics-prototype/testdata']
for folder in folders:
 files.update(p.relative_to(base).as_posix()for p in (base/folder).rglob('*')if p.is_file()and '__pycache__'not in p.parts)
files.update('autographics-auto-driver/research/'+p for p in ['asset-import-feasibility.md','scene-assets-handoff.md','scene-assets-spec-delta.md','asset-probe/two-parts.glb','asset-probe/generate.py','asset-probe/README.md'])
suffixes=old['text_suffixes']+['.agscene']
hashes={}
for relative in sorted(files):
 p=base/relative;assert p.is_file(),relative;assert not p.is_symlink()and not p.is_junction(),relative
 b=p.read_bytes()
 if p.suffix.lower()in suffixes or p.name.lower()in suffixes:b=b.replace(b'\r\n',b'\n')
 hashes[relative]=hashlib.sha256(b).hexdigest()
identity=dict(plan='PLAN-838',plan_revision=1,base_commit='cf2e99f50ee2a70b70ca2cfc23a5af56b405638b',sdk_commit=old['sdk_commit'],text_suffixes=suffixes,folders=folders,files=hashes)
(root/'research/plan838-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
print(f'Frozen PLAN-838 {len(hashes)} source/asset hashes')
