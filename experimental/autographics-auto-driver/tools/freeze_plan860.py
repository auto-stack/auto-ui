"""Regenerate research/plan860-identity.json from the working tree.

Run after the last source edit and before committing: the identity binds every
verified file hash at freeze time; verify_plan860.py then guards the committed
tree against drift.
"""
import hashlib, json, pathlib

base = pathlib.Path(__file__).resolve().parents[2]
FOLDERS = [
    'autographics-auto-driver/api',
    'autographics-auto-driver/src',
    'autographics-auto-driver/tests',
    'autographics-auto-driver/samples',
    'autographics-auto-driver/tools',
    'autographics-auto-driver/research/game-runtime',
    'autographics-prototype/src',
    'autographics-prototype/tests',
    'autographics-prototype/assets',
    'autographics-prototype/testdata',
]
TEXT = {'.rs', '.at', '.md', '.toml', '.lock', '.json', '.txt', '.py', '.csv',
        '.tsv', '.wgsl', '.gitignore', '.gitattributes', '.agscene'}

files = {}


def add(relative: str) -> None:
    data = (base / relative).read_bytes()
    if pathlib.Path(relative).suffix.lower() in TEXT or pathlib.Path(relative).name.lower() in TEXT:
        data = data.replace(b'\r\n', b'\n')
    files[relative] = hashlib.sha256(data).hexdigest()


for package in ['autographics-auto-driver', 'autographics-prototype']:
    for p in (base / package).iterdir():
        if p.is_file():
            add(p.relative_to(base).as_posix())
for folder in FOLDERS:
    for p in (base / folder).rglob('*'):
        if p.is_file() and '__pycache__' not in p.parts:
            add(p.relative_to(base).as_posix())

identity = {
    'plan': 860,
    'plan_revision': 1,
    'base_commit': '81b0974ac2512126790b0ef2af79235edf45b485',
    'sdk_commit': '7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467',
    'text_suffixes': sorted(TEXT),
    'folders': FOLDERS,
    'files': dict(sorted(files.items())),
}
out = base / 'autographics-auto-driver/research/plan860-identity.json'
out.write_bytes((json.dumps(identity, indent=1) + '\n').encode('utf8'))
print(f'FROZEN PLAN-860 identity: {len(files)} files')
