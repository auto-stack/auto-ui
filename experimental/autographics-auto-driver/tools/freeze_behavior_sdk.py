"""PLAN-840: derive a pinned SDK commit from the original pin and scanner-only fix.
Never upgrades unrelated SDK files or patches the extracted SDK working copy.
Run from the driver after the canonical scanner fix is ready for verification.
"""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile

root = Path(__file__).resolve().parents[1]
repo = root.parents[1]
research = root / 'research'
baseline = json.loads((research / 'baseline.json.txt').read_text())
original = '9576b4dad5ceb8b201adedd46f9689f25c2dd5f9'
paths = ['crates/auto-lang-core/src/' + name for name in
         ['use_scanner.rs', 'dep_scanner.rs', 'dependency_scan.rs', 'lib.rs']]

def git(*args, data=None, env=None):
    return subprocess.check_output(['git', *args], cwd=repo, input=data, env=env)

assert git('rev-parse', '--show-toplevel').decode().strip().replace('\\', '/') == repo.as_posix()
assert git('branch', '--show-current').decode().strip() == 'plan-840-dev'
with tempfile.TemporaryDirectory(prefix='sdk-index-', dir=root / 'target') as temp:
    env = dict(os.environ, GIT_INDEX_FILE=str(Path(temp) / 'index'))
    git('read-tree', original, env=env)
    for path in paths:
        source = (repo / path).read_bytes().replace(b'\r\n', b'\n')
        if path.endswith('/lib.rs'):
            # The graphics tree is not the SDK baseline: take only this module line.
            assert b'mod dependency_scan;' in source
            source = git('show', f'{original}:{path}').replace(
                b'pub mod dep_scanner;', b'pub mod dep_scanner;\nmod dependency_scan;')
        blob = git('hash-object', '-w', '--stdin', data=source).decode().strip()
        git('update-index', '--add', '--cacheinfo', f'100644,{blob},{path}', env=env)
    tree = git('write-tree', env=env).decode().strip()
    ref = 'refs/heads/codex/plan-840-sdk'
    existing = subprocess.run(['git', 'rev-parse', '--verify', ref], cwd=repo, capture_output=True)
    if existing.returncode == 0:
        commit = existing.stdout.decode().strip()
        assert git('rev-parse', f'{commit}^{{tree}}').decode().strip() == tree, 'SDK ref already owns different work'
    else:
        commit = git('commit-tree', tree, '-p', original,
                     data=b'fix(sdk): ignore inert dependency text (Plan 840)\n').decode().strip()
        git('update-ref', ref, commit, '0' * 40)
changed = set(git('diff', '--name-only', original, commit).decode().splitlines())
assert changed == set(paths), changed
sdk = root / '.sdk-source'
if sdk.exists():
    old = (root / 'target/sdk-r1').resolve()
    assert root.resolve() in sdk.resolve().parents and root.resolve() in old.parents
    assert not old.exists(), 'preserve prior snapshot; refusing to overwrite'
    sdk.rename(old)
sdk.mkdir()
extract = [v['path'] for v in baseline['sdk']['path_sources']] + ['stdlib', 'Cargo.toml']
with tarfile.open(fileobj=io.BytesIO(git('archive', commit, '--', *extract))) as archive:
    assert all(m.isfile() or m.isdir() for m in archive.getmembers()), 'links forbidden'
    archive.extractall(sdk, filter='data')
manifest = (sdk / 'Cargo.toml').read_text()
start = manifest.index('[workspace.dependencies]')
end = manifest.index('[profile.dev]', start)
(sdk / 'Cargo.toml').write_text('[workspace]\nresolver = "2"\nmembers = ' + json.dumps(extract[:-2]) + '\n\n' + manifest[start:end])
(sdk / 'Cargo.lock').write_bytes((research / 'sdk-Cargo.lock.txt').read_bytes())
assert not any(p.is_symlink() or p.is_junction() for p in sdk.rglob('*'))
lock = json.loads((research / 'source-lock.json').read_text())
lock['files'] = {p.relative_to(sdk).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                 for p in sdk.rglob('*') if p.is_file()}
lock['plan840_sdk_commit'] = commit
baseline['sdk']['repository_commit'] = commit
baseline['sdk']['last_sdk_change'] = commit
baseline['sdk']['repository_role'] = 'PLAN-840 scanner-only SDK artifact; parent is original delivered pin; review/landing pending'
baseline['sdk']['plan840_original_pin'] = original
baseline['sdk']['plan840_patch_paths'] = paths
for item in baseline['sdk']['path_sources']:
    item['git_tree'] = git('rev-parse', f"{commit}:{item['path']}").decode().strip()
lock['baseline'] = baseline
(research / 'source-lock.json').write_text(json.dumps(lock, indent=2) + '\n')
(research / 'baseline.json.txt').write_text(json.dumps(baseline, indent=2) + '\n')
print(json.dumps(dict(commit=commit, parent=original, changed=sorted(changed), files=len(lock['files'])), indent=2))
