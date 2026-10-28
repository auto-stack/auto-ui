"""Work-owned source freeze; never invoked implicitly by verification."""
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
old=json.loads((root/'research/plan840-identity.json').read_text())
files=set(old['files'])|{'autographics-auto-driver/research/plan840-identity.json'}
folders=['autographics-auto-driver/src','autographics-auto-driver/tests','autographics-auto-driver/samples','autographics-auto-driver/tools','autographics-prototype/src','autographics-prototype/tests','autographics-prototype/assets','autographics-prototype/testdata']
for folder in folders:
 files.update(p.relative_to(base).as_posix()for p in (base/folder).rglob('*')if p.is_file()and '__pycache__'not in p.parts)
files.update('autographics-auto-driver/research/'+p for p in ['behavior-project-feasibility.md','behavior-project-handoff.md','behavior-project-spec-delta.md'])
suffixes=old['text_suffixes']+['.agscene','.gitattributes']
hashes={}
for relative in sorted(files):
 p=base/relative;assert p.is_file(),relative;assert not p.is_symlink()and not p.is_junction(),relative
 b=p.read_bytes()
 if p.suffix.lower()in suffixes or p.name.lower()in suffixes:b=b.replace(b'\r\n',b'\n')
 hashes[relative]=hashlib.sha256(b).hexdigest()
identity=dict(plan='PLAN-845',plan_revision=2,base_commit='8b62db5cba54b9bfe7f759f4920b22ccb9b75e88',sdk_commit=old['sdk_commit'],text_suffixes=suffixes,folders=folders,files=hashes)
(root/'research/plan845-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
print(f'Frozen PLAN-845 {len(hashes)} source/asset hashes')
