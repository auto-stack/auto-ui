"""Run after adding final authored files to Git, before freezing work evidence."""
import hashlib,json,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1];base=root.parent
text=['.rs','.at','.md','.toml','.lock','.json','.txt','.py','.csv','.tsv','.wgsl','.gitignore']
paths=subprocess.check_output(['git','ls-files','--','experimental/autographics-auto-driver','experimental/autographics-prototype'],cwd=root.parents[1],text=True).splitlines()
files={}
for s in paths:
 p=root.parents[1]/s
 rel=p.relative_to(base).as_posix()
 if rel.endswith('research/plan835-identity.json'):continue
 b=p.read_bytes()
 if p.suffix.lower() in text:b=b.replace(b'\r\n',b'\n')
 files[rel]=hashlib.sha256(b).hexdigest()
d={'plan':'PLAN-835','base_commit':'19adeebf1b1031554c71acabaa219731b3a28b96','sdk_commit':'9576b4dad5ceb8b201adedd46f9689f25c2dd5f9','text_suffixes':text,'files':files}
(root/'research/plan835-identity.json').write_text(json.dumps(d,indent=2)+'\n')
print(f'Frozen {len(files)} files')
