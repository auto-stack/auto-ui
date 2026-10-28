"""Compare frozen PLAN-856 pixels against the current default and mandatory Home capture."""
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys

from PIL import Image, ImageChops
import runner_provenance as provenance

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE_COMMIT = '4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7'
BASE_RUNNER_SHA = '05372dfc9f36a9f2ab92a1b1200544f133dabff57d0b486ef29b04facb5a9424'
BASE_IDENTITY_SHA = 'f710d2a0d8c26bc7b33be57dad8911a1b88a90fa49a56a629f35ed25295ba195'
CASES = {'grid-composite', 'grid-occlusion', 'auto-drag', 'save-reopen', 'focus-cancel', 'play-restore', 'cross-button-capture'}

class EvidenceError(Exception):
    pass

def need(condition, message):
    if not condition:
        raise EvidenceError(message)

def digest(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()

def read_json(path, label):
    path = pathlib.Path(path)
    need(path.is_file() and not path.is_symlink() and not path.is_junction(), f'{label} missing or linked')
    try:
        return provenance.load(path)
    except (ValueError, OSError) as e:
        raise EvidenceError(f'{label} invalid: {e}') from e

def runner(path, label):
    d = read_json(path, label)
    need(d.get('schema') == 'autographics.runner/1', f'{label} schema')
    binary = pathlib.Path(d.get('path', ''))
    need(binary.is_file() and not binary.is_symlink() and not binary.is_junction(), f'{label} binary missing or linked')
    need(digest(binary) == d.get('sha256') and binary.parent.name == d.get('sha256'), f'{label} binary identity')
    identity = pathlib.Path(d.get('source_identity_path') or '')
    need(identity.is_file() and not identity.is_symlink() and not identity.is_junction(), f'{label} source identity missing')
    need(digest(identity) == d.get('source_identity_sha256'), f'{label} source identity hash')
    return d, binary

def prepare(reset_dir, baseline_record, expected_commit, allow_reuse=False):
    reset_dir = pathlib.Path(reset_dir).resolve()
    home_path = reset_dir / 'normal.json'
    home = read_json(home_path, 'Home report')
    need(home.get('viewport3d_check') is True and home.get('passed') is True and home.get('error') is None, 'Home report incomplete or failed')
    need(home.get('viewport3d_phase') == 34, 'Home report incomplete phase')
    records = home.get('viewport3d_records')
    need(isinstance(records, list), 'Home report missing phases')
    phases = [x['phase'] for x in records if isinstance(x, dict) and 'phase' in x]
    cases = {x['case'] for x in records if isinstance(x, dict) and 'case' in x}
    need(len(phases) == 34 and set(phases) == set(range(34)) and cases == CASES, 'Home report incomplete phases or cases')
    screenshot = home_path.with_suffix('.png')
    need(screenshot.is_file() and not screenshot.is_symlink() and not screenshot.is_junction(), 'Home screenshot missing or linked')
    current, binary = runner(reset_dir / 'runner.json', 'current runner')
    old, old_binary = runner(baseline_record, 'baseline runner')
    need(old.get('code_commit') == BASE_COMMIT and old.get('sha256') == BASE_RUNNER_SHA and old.get('source_identity_sha256') == BASE_IDENTITY_SHA, 'baseline frozen identity mismatch')
    need(current.get('origin_verified') is True, 'current runner build provenance required')
    need(current.get('verification_commit') == expected_commit, 'current commit mismatch')
    git = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True, check=False)
    need(git.returncode == 0 and git.stdout.strip() == expected_commit, 'checkout commit mismatch')
    status = subprocess.run(['git', 'status', '--porcelain', '--untracked-files=all'], cwd=ROOT, capture_output=True, text=True, check=False)
    need(status.returncode == 0 and not status.stdout.strip(), 'checkout is not clean')
    need(current.get('source_base_commit') == BASE_COMMIT, 'current base commit mismatch')
    need(current.get('verification_source_identity_sha256') == digest(ROOT / 'research/plan857-identity.json'), 'current source identity mismatch')
    verify_env = dict(os.environ)
    verify_env.pop('PYTHONOPTIMIZE', None)
    for script, flags, label in [('verify_plan857.py', [], 'current source closure mismatch'), ('prepare_sdk.py', ['--verify-only'], 'current SDK closure mismatch')]:
        result = subprocess.run([sys.executable, str(ROOT/'tools'/script), *flags], cwd=ROOT, env=verify_env, capture_output=True, text=True, check=False)
        need(result.returncode == 0, label)
    need(current.get('sdk_sha256') == old.get('sdk_sha256') == digest(ROOT / 'research/source-lock.json'), 'SDK identity mismatch')
    build = current.get('build_provenance')
    need(isinstance(build, dict), 'current runner build provenance required')
    proof_path = pathlib.Path(build.get('proof_path', ''))
    need(proof_path.is_file() and digest(proof_path) == build.get('proof_sha256'), 'build proof identity mismatch')
    validated = provenance.verify_build(proof_path, binary, provenance.load(ROOT / 'research/plan857-identity.json'), allow_reuse=allow_reuse)
    need(validated == build and current.get('code_commit') == validated['build_commit'], 'runner build attribution mismatch')
    need(validated['target_kind'] == 'bin' and validated['target_name'] == 'autographics-authoring', 'wrong runner Cargo target')
    need(current.get('source_identity_sha256') == validated['source_identity_sha256'], 'runner build source mismatch')
    locks = provenance.check_locks(old.get('locks'), current.get('locks'), provenance.load(old['source_identity_path']), provenance.load(ROOT/'research/plan857-identity.json'))
    need(current.get('configuration') == old.get('configuration'), 'baseline configuration mismatch')
    need(home.get('runner_sha256') == current.get('sha256'), 'Home runner identity mismatch')
    need(pathlib.Path(home.get('runner_path', '')).resolve() == binary.resolve(), 'Home runner path mismatch')
    receipt = read_json(reset_dir / 'results.json', 'Home results receipt')
    normal = [x for x in receipt if isinstance(x, dict) and x.get('name') == 'normal'] if isinstance(receipt, list) else []
    need(len(normal) == 1 and normal[0].get('exit') == 0, 'Home results receipt incomplete')
    need(pathlib.Path(normal[0].get('report', '')).resolve() == home_path, 'Home report receipt path mismatch')
    need(normal[0].get('sha256') == digest(home_path), 'Home report receipt hash mismatch')
    need(normal[0].get('screenshot_sha256') == digest(screenshot), 'Home screenshot receipt hash mismatch')
    return old, old_binary, current, binary, home, home_path, screenshot, locks

def compare(reset_dir, baseline_record, expected_commit, output, allow_reuse=False):
    old, old_binary, current, binary, home, home_path, screenshot, locks = prepare(reset_dir, baseline_record, expected_commit, allow_reuse)
    output = pathlib.Path(output).resolve()
    need(not output.exists(), 'output already exists')
    output.mkdir(parents=True)
    env = dict(os.environ, RUST_MIN_STACK='16777216')
    rows = {}
    for name, exe, expected_sha in [('856', old_binary, old['sha256']), ('857', binary, current['sha256'])]:
        report = output / f'{name}.json'
        with (output / f'{name}.log').open('w', encoding='utf-8') as log:
            result = subprocess.run([str(exe), '--frames', '80', '--report', str(report)], env=env, stdout=log, stderr=log, timeout=90)
        need(result.returncode == 0, f'{name} default run failed')
        data = read_json(report, f'{name} default report')
        need(data.get('error') is None and data.get('frames') == 80, f'{name} default run incomplete')
        need(data.get('runner_sha256') == expected_sha, f'{name} default runner identity mismatch')
        need(report.with_suffix('.png').is_file(), f'{name} default screenshot missing')
        rows[name] = data
    need(rows['856'].get('three') == rows['857'].get('three') == home.get('three'), 'default or Home 3D texture differs')
    images = [Image.open(p).convert('RGB') for p in [output/'856.png', output/'857.png', screenshot]]
    need(all(im.size == (1200, 800) for im in images), 'screenshot dimensions differ')
    crop = (0, 400, 800, 800)
    base, default, reset = [im.crop(crop) for im in images]
    need(ImageChops.difference(base, default).getbbox() is None, 'default composite pixels differ')
    need(ImageChops.difference(base, reset).getbbox() is None, 'Home composite pixels differ')
    summary = {'baseline_runner_sha256': old['sha256'], 'baseline_commit': old['code_commit'], 'current_runner_sha256': current['sha256'], 'current_commit': expected_commit, 'binary_build_commit': current['code_commit'], 'reuse': current['build_provenance']['reuse'], 'reuse_scope': current['build_provenance']['reuse_scope'], 'build_proof_sha256': current['build_provenance']['proof_sha256'], 'source_identity_sha256': current['verification_source_identity_sha256'], 'build_source_identity_sha256': current['source_identity_sha256'], 'sdk_sha256': current['sdk_sha256'], 'locks': locks, 'home_report_sha256': digest(home_path), 'home_screenshot_sha256': digest(screenshot), 'home_results_sha256': digest(pathlib.Path(reset_dir)/'results.json'), 'three': home['three'], 'crop': crop, 'default_equal': True, 'reset_equal': True}
    (output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n', encoding='utf-8')
    print('PASS frozen default and mandatory Home pixels, runner and source identity')

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline-runner', required=True, type=pathlib.Path)
    p.add_argument('--reset-dir', required=True, type=pathlib.Path)
    p.add_argument('--commit', required=True)
    p.add_argument('--output', required=True, type=pathlib.Path)
    p.add_argument('--allow-reuse', action='store_true')
    a = p.parse_args()
    try:
        compare(a.reset_dir, a.baseline_runner, a.commit, a.output, a.allow_reuse)
    except (EvidenceError, provenance.ProvenanceError, OSError, ValueError, KeyError, subprocess.TimeoutExpired) as e:
        print(f'FAIL: {e}', file=sys.stderr)
        return 1
    return 0

if __name__ == '__main__':
    sys.exit(main())
