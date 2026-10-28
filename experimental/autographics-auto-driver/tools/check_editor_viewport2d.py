"""Run the bounded real OS/GPU viewport oracle with immutable runner identity."""
import argparse
import json
import os
import pathlib
import subprocess
import m2_evidence

p = argparse.ArgumentParser()
p.add_argument('output', type=pathlib.Path)
p.add_argument('--executable', required=True, type=pathlib.Path)
p.add_argument('--controls', action='store_true')
a = p.parse_args()
assert not a.output.exists(), 'fresh output required'
a.output.mkdir(parents=True)
runner = pathlib.Path(a.executable).resolve()
identity = m2_evidence.record_runner(runner, a.output / 'runner.json')
runner = pathlib.Path(m2_evidence.verify_runner(identity))

def run(name, flags, expected):
    report = (a.output / (name + '.json')).resolve()
    env = dict(os.environ, RUST_MIN_STACK='16777216')
    command = [str(runner), '--viewport2d-check', '--report', str(report), *flags]
    with (a.output / (name + '.log')).open('w') as log:
        result = subprocess.run(command, env=env, stdout=log, stderr=log, timeout=90)
    assert report.is_file(), f'{name}: missing complete report'
    data = json.loads(report.read_text())
    assert data['runner_sha256'] == m2_evidence.sha(runner), 'runner provenance drift'
    if expected:
        assert result.returncode == 0 and data['passed'] and data['error'] is None, (name, data.get('error'))
        assert data['viewport2d_phase'] == 20, 'partial viewport oracle'
        records = data['viewport2d_records']
        assert {r['phase'] for r in records if 'phase' in r} == set(range(20))
        assert {r['case'] for r in records if 'case' in r} == {'grid-composite', 'auto-drag-composite'}
        assert report.with_suffix('.png').is_file(), 'final composite missing'
    else:
        assert result.returncode != 0 and not data['passed'] and 'viewport2d incomplete' in data['error'], 'partial false green'
    return {'name': name, 'exit': result.returncode, 'report': str(report), 'sha256': m2_evidence.sha(report)}

rows = [run('normal', ['--frames', '1000'], True)]
if a.controls:
    rows.append(run('partial', ['--frames', '1'], False))
(a.output / 'results.json').write_text(json.dumps(rows, indent=2))
print(f'PASS viewport2d: {len(rows)} completed OS/GPU runs; actual transform/content/pixels')
