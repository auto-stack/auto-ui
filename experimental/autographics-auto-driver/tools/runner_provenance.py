"""Validate exact lock identities and controlled runner build provenance."""
import hashlib
import json
import os
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPO = ROOT.parent.parent
LOCKS = ('autographics-auto-driver/Cargo.lock', 'autographics-prototype/Cargo.lock')
EXPERIMENTAL_PREFIX = 'experimental/'
TOOL_PREFIX = 'autographics-auto-driver/tools/'
IDENTITY_PATH = 'experimental/autographics-auto-driver/research/plan857-identity.json'

class ProvenanceError(ValueError):
    pass

def need(ok, message):
    if not ok:
        raise ProvenanceError(message)

def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()

def load(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            need(key not in result, f'duplicate JSON key: {key}')
            result[key] = value
        return result
    p = pathlib.Path(path)
    need(p.is_file() and not p.is_symlink() and not p.is_junction(), 'provenance file missing or linked')
    return json.loads(p.read_text(encoding='utf-8'), object_pairs_hook=unique)

def lock_map(value):
    need(isinstance(value, dict) and len(value) == 2, 'exactly two lock identities required')
    normalized = {}
    for raw, digest in value.items():
        need(isinstance(raw, str) and isinstance(digest, str), 'invalid lock entry')
        key = raw.replace('\\', '/')
        need(key in LOCKS and key not in normalized, 'unexpected, duplicate or aliased lock path')
        normalized[key] = digest
    need(set(normalized) == set(LOCKS), 'exact lock paths required')
    return normalized

def identity_locks(identity):
    files = identity.get('files', {})
    need(isinstance(files, dict), 'source identity files missing')
    return {k: files.get(k) for k in LOCKS}

def check_locks(old_receipt, current_receipt, old_identity, current_identity):
    old = lock_map(old_receipt)
    current = lock_map(current_receipt)
    need(old == current, 'lock identity mismatch')
    old_files, current_files = identity_locks(old_identity), identity_locks(current_identity)
    for key in LOCKS:
        file = ROOT.parent / key
        need(file.is_file() and not file.is_symlink() and not file.is_junction(), 'lock file missing or linked')
        raw = file.read_bytes()
        canonical = hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()
        need(sha(file) == old[key], 'actual lock bytes mismatch')
        need(old_files[key] == canonical == current_files[key], 'source identity lock mismatch')
    return old

def git(*args):
    result = subprocess.run(['git', *args], cwd=ROOT, capture_output=True, check=False)
    need(result.returncode == 0, f'git {args[0]} failed')
    return result.stdout

def current_commit():
    commit = git('rev-parse', 'HEAD').decode().strip()
    need(not git('status', '--porcelain', '--untracked-files=all').strip(), 'checkout is not clean')
    return commit

def canonical_sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes().replace(b'\r\n', b'\n')).hexdigest()

def committed_identity(commit, identity_path=None):
    raw = git('show', f'{commit}:{identity_path or IDENTITY_PATH}')
    return hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest()

def verify_build(proof_path, executable, current_identity, allow_reuse=False, identity_path=None):
    identity_rel = identity_path or IDENTITY_PATH
    # Callers pass the driver-relative research path; git needs the repo path.
    identity_git = (EXPERIMENTAL_PREFIX + 'autographics-auto-driver/' + identity_rel
                    if identity_rel.startswith('research/') else identity_rel)
    need(current_identity == load(ROOT/identity_rel), 'current identity data mismatch')
    proof = load(proof_path)
    need(proof.get('schema') == 'autographics.controlled-build/1' and proof.get('clean_before_after') is True, 'controlled build proof required')
    context_path = pathlib.Path(proof.get('build_context_path', ''))
    need(context_path.is_file() and sha(context_path) == proof.get('build_context_sha256'), 'build context identity mismatch')
    context = load(context_path)
    for key in ['commit','source_identity_sha256','sdk_sha256','locks','configuration','rustc','cargo','binary_sha256','build_log_sha256','cargo_artifacts_sha256','clean_before_after']:
        need(context.get(key) == proof.get(key), f'build context {key} mismatch')
    built = pathlib.Path(proof.get('binary_path', ''))
    candidate = pathlib.Path(executable)
    need(built.is_file() and not built.is_symlink() and not built.is_junction(), 'built runner missing or linked')
    need(candidate.is_file() and sha(candidate) == proof.get('binary_sha256') == sha(built), 'runner does not match controlled build')
    for path_key, hash_key in [('build_log_path','build_log_sha256'),('cargo_artifacts_path','cargo_artifacts_sha256'),('source_identity_path','source_identity_sha256')]:
        path = pathlib.Path(proof.get(path_key, ''))
        need(path.is_file() and not path.is_symlink() and not path.is_junction() and sha(path) == proof.get(hash_key), f'build proof {path_key} mismatch')
    origin = proof.get('commit')
    need(isinstance(origin, str) and len(origin) == 40 and committed_identity(origin, identity_git) == canonical_sha(proof['source_identity_path']), 'build source does not match committed identity')
    old_identity = load(proof['source_identity_path'])
    current = current_commit()
    need(committed_identity(current, identity_git) == canonical_sha(ROOT/identity_rel), 'current manifest is not committed')
    need(proof.get('sdk_sha256') == sha(ROOT/'research/source-lock.json'), 'build SDK identity mismatch')
    need(proof.get('configuration') == {k: os.environ.get(k) for k in ['RUSTFLAGS','RUST_MIN_STACK','RUSTC_WRAPPER']}, 'build configuration mismatch')
    need(proof.get('rustc') == subprocess.check_output(['rustc','-vV'], text=True), 'rustc version mismatch')
    need(proof.get('cargo') == subprocess.check_output(['cargo','--version'], text=True).strip(), 'cargo version mismatch')
    check_locks(proof.get('locks'), proof.get('locks'), old_identity, current_identity)
    events = []
    for line in pathlib.Path(proof['cargo_artifacts_path']).read_text(encoding='utf8').splitlines():
        if line.lstrip().startswith('{'):
            events.append(json.loads(line))
    target_name = proof.get('target_name', 'autographics-authoring')
    target_kind = proof.get('target_kind', 'bin')
    need(target_kind in ('bin', 'test') and isinstance(target_name, str) and target_name, 'invalid Cargo target')
    artifacts = [e for e in events if e.get('reason') == 'compiler-artifact' and e.get('target', {}).get('name') == target_name and target_kind in e.get('target', {}).get('kind', []) and e.get('executable')]
    need(len(artifacts) == 1 and pathlib.Path(artifacts[0]['executable']).resolve() == built.resolve(), 'Cargo artifact does not bind runner')
    need(any(e.get('reason') == 'build-finished' and e.get('success') is True for e in events), 'Cargo build incomplete')
    changes = []
    if origin != current:
        need(allow_reuse, 'cross-commit runner reuse requires explicit approval')
        before = old_identity.get('files', {})
        after = current_identity.get('files', {})
        changes = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
        need(all(k.startswith(TOOL_PREFIX) and k.endswith('.py') for k in changes), 'cross-commit build inputs changed')
        diff = git('diff', '--name-only', origin, current).decode('utf8').splitlines()
        need(all(p == IDENTITY_PATH or (p.startswith(EXPERIMENTAL_PREFIX + TOOL_PREFIX) and p.endswith('.py')) for p in diff), 'cross-commit repository changes outside approved validation tools')
    return {'build_commit': origin, 'verification_commit': current, 'reuse': origin != current, 'reuse_scope': '3d-default-pixels-only' if origin != current else None, 'changed_validation_files': changes, 'proof_path': str(pathlib.Path(proof_path).resolve()), 'proof_sha256': sha(proof_path), 'source_identity_sha256': proof['source_identity_sha256'], 'target_name': target_name, 'target_kind': target_kind}
