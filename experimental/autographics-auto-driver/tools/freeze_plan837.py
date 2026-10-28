"""Freeze reviewed inputs after explicitly staging all intended authored files.
Historical manifests are immutable. The new manifest alone excludes itself.
"""
import hashlib,json,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1];repo=root.parents[1];base=root.parent
assert subprocess.check_output(['git','branch','--show-current'],cwd=repo,text=True).strip()=='plan-837-dev'
text=['.rs','.at','.md','.toml','.lock','.json','.txt','.py','.csv','.tsv','.wgsl','.gitignore']
paths=subprocess.check_output(['git','ls-files','--','experimental/autographics-auto-driver','experimental/autographics-prototype'],cwd=repo,text=True).splitlines()
files={}
for relative in paths:
 p=repo/relative;key=p.relative_to(base).as_posix()
 if key=='autographics-auto-driver/research/plan837-identity.json':continue
 assert p.is_file() and not p.is_symlink() and not p.is_junction(),key
 b=p.read_bytes()
 if p.suffix.lower() in text or p.name.lower() in text:b=b.replace(b'\r\n',b'\n')
 files[key]=hashlib.sha256(b).hexdigest()
d=dict(plan='PLAN-837',base_commit='52ec4c7c13b337992278b343ef89d8aa17102ff6',sdk_commit='9576b4dad5ceb8b201adedd46f9689f25c2dd5f9',text_suffixes=text,files=files)
(root/'research/plan837-identity.json').write_text(json.dumps(d,indent=2)+'\n')
print(f'Frozen {len(files)} source/asset hashes')
