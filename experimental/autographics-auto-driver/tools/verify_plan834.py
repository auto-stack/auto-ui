"""Verify the frozen authored driver/AGP/assets, without producer paths or build outputs."""
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1]
identity=json.loads((root/'research/plan834-identity.json').read_text())
base=root.parent
for relative,digest in identity['files'].items():
 path=(base/relative).resolve()
 assert base.resolve() in path.parents and path.is_file(),f'missing/outside source: {relative}'
 data=path.read_bytes()
 if path.suffix.lower() in identity['text_suffixes']:data=data.replace(b'\r\n',b'\n')
 assert hashlib.sha256(data).hexdigest()==digest,f'source hash mismatch: {relative}'
print(f"PASS PLAN-834 authored source/assets: {len(identity['files'])} hashes; code {identity['code_commit']}")
