"""Run identity, incompleteness, missing-artifact and pixel controls through the real comparator."""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parents[1]
COMPARATOR = ROOT / 'tools/check_default_3d_baseline.py'

def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()

def fail(message):
    raise RuntimeError(message)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline-runner', required=True, type=pathlib.Path)
    parser.add_argument('--reset-dir', required=True, type=pathlib.Path)
    parser.add_argument('--commit', required=True)
    parser.add_argument('--output', required=True, type=pathlib.Path)
    parser.add_argument('--allow-reuse', action='store_true')
    args = parser.parse_args()
    out = args.output.resolve()
    if out.exists(): fail('fresh output required')
    out.mkdir(parents=True)
    source = args.reset_dir.resolve()
    for name in ('runner.json', 'results.json', 'normal.json', 'normal.png', 'partial.json'):
        if not (source/name).is_file(): fail(f'fixture missing {name}')
    rows = []
    for case, expected in [('missing-report','Home report missing or linked'), ('partial','Home report incomplete or failed'), ('wrong-runner','Home runner identity mismatch'), ('missing-screenshot','Home screenshot missing or linked'), ('wrong-source','current runner source identity hash'), ('wrong-baseline','baseline frozen identity mismatch'), ('double-wrong-locks','unexpected, duplicate or aliased lock path'), ('double-wrong-values','actual lock bytes mismatch'), ('missing-lock','exactly two lock identities required'), ('alias-locks','unexpected, duplicate or aliased lock path'), ('corrupt-screenshot','Home screenshot receipt hash mismatch'), ('different-pixel','Home composite pixels differ')]:
        fixture = out / case
        fixture.mkdir()
        for name in ('runner.json','results.json','normal.json','normal.png'):
            shutil.copyfile(source/name, fixture/name)
        report = fixture/'normal.json'
        shot = fixture/'normal.png'
        receipt = json.loads((fixture/'results.json').read_text(encoding='utf8'))
        normal = next(x for x in receipt if x['name'] == 'normal')
        normal['report'] = str(report.resolve())
        if case == 'missing-report':
            report.unlink()
        elif case == 'partial':
            shutil.copyfile(source/'partial.json', report)
        elif case == 'wrong-runner':
            data = json.loads(report.read_text(encoding='utf8'))
            data['runner_sha256'] = '0'*64
            report.write_text(json.dumps(data), encoding='utf8')
        elif case == 'missing-screenshot':
            shot.unlink()
        elif case == 'wrong-source':
            path = fixture/'runner.json'
            data = json.loads(path.read_text(encoding='utf8'))
            data['source_identity_sha256'] = '0'*64
            path.write_text(json.dumps(data), encoding='utf8')
        elif case in ('corrupt-screenshot','different-pixel'):
            image = Image.open(shot).convert('RGB')
            pixel = image.getpixel((10,410))
            image.putpixel((10,410), (pixel[0]^255, pixel[1], pixel[2]))
            image.save(shot)
            if case == 'different-pixel':
                normal['screenshot_sha256'] = sha(shot)
        baseline_record = args.baseline_runner.resolve()
        if case in ('wrong-baseline','double-wrong-locks','double-wrong-values','missing-lock','alias-locks'):
            data = json.loads(baseline_record.read_text(encoding='utf8'))
            if case == 'wrong-baseline':
                data['code_commit'] = '0'*40
            else:
                current_path = fixture/'runner.json'
                current = json.loads(current_path.read_text(encoding='utf8'))
                if case == 'double-wrong-locks':
                    locks = {k: sha(ROOT.parent/k) for k in ['autographics-auto-driver/README.md','autographics-prototype/Cargo.toml']}
                elif case == 'double-wrong-values':
                    locks = {k: '0'*64 for k in data['locks']}
                elif case == 'missing-lock':
                    locks = {next(iter(data['locks'])): next(iter(data['locks'].values()))}
                else:
                    value = next(iter(data['locks'].values()))
                    locks = {'autographics-auto-driver/Cargo.lock': value, 'autographics-auto-driver\\Cargo.lock': value}
                data['locks'] = locks
                current['locks'] = locks
                current_path.write_text(json.dumps(current),encoding='utf8')
            baseline_record = fixture/'wrong-baseline.json'
            baseline_record.write_text(json.dumps(data),encoding='utf8')
        if report.exists(): normal['sha256'] = sha(report)
        (fixture/'results.json').write_text(json.dumps(receipt),encoding='utf8')
        result_dir = out / (case+'-comparison')
        command = [sys.executable, str(COMPARATOR), '--baseline-runner', str(baseline_record), '--reset-dir', str(fixture), '--commit', args.commit, '--output', str(result_dir)]
        if args.allow_reuse: command.append('--allow-reuse')
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, env=dict(os.environ,RUST_MIN_STACK='16777216'),timeout=120)
        (out/(case+'.stdout')).write_text(result.stdout,encoding='utf8')
        (out/(case+'.stderr')).write_text(result.stderr,encoding='utf8')
        if result.returncode == 0 or expected not in result.stderr or (result_dir/'summary.json').exists():
            fail(f'{case} did not reject at expected oracle: {result.returncode} {result.stderr}')
        rows.append({'case':case,'exit':result.returncode,'expected':expected,'stderr_sha256':sha(out/(case+'.stderr'))})
    positive_dir = out/'positive-comparison'
    command = [sys.executable,str(COMPARATOR),'--baseline-runner',str(args.baseline_runner.resolve()),'--reset-dir',str(source),'--commit',args.commit,'--output',str(positive_dir)]
    if args.allow_reuse: command.append('--allow-reuse')
    positive = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,env=dict(os.environ,RUST_MIN_STACK='16777216'),timeout=120)
    (out/'positive.stdout').write_text(positive.stdout,encoding='utf8')
    (out/'positive.stderr').write_text(positive.stderr,encoding='utf8')
    if positive.returncode != 0 or not (positive_dir/'summary.json').is_file():
        fail(f'positive did not pass: {positive.returncode} {positive.stderr}')
    rows.append({'case':'positive','exit':positive.returncode,'summary_sha256':sha(positive_dir/'summary.json')})
    (out/'results.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf8')
    print(f'PASS {len(rows)-1} real comparator negative controls and actual frozen/default/Home positive')

if __name__ == '__main__':
    main()
