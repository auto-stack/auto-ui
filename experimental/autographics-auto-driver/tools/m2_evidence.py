"""PLAN-853 immutable runner and completed-selection evidence; no test scheduler."""
import argparse
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
CURRENT = 860 if (ROOT / 'research/plan860-identity.json').exists() or (ROOT / 'tools/verify_plan860.py').exists() else 857 if (ROOT / 'research/plan857-identity.json').exists() or (ROOT / 'tools/verify_plan857.py').exists() else 856 if (ROOT / 'research/plan856-identity.json').exists() or (ROOT / 'tools/verify_plan856.py').exists() else 855 if (ROOT / 'research/plan855-identity.json').exists() or (ROOT / 'tools/verify_plan855.py').exists() else 854 if (ROOT / 'research/plan854-identity.json').exists() or (ROOT / 'tools/verify_plan854.py').exists() else 853
ARTIFACTS = pathlib.Path(f'D:/autostack/plan-artifacts/plan-{CURRENT}')


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def record_runner(executable, output, role='producer', build_proof=None, allow_reuse=False):
    executable, output = pathlib.Path(executable).resolve(), pathlib.Path(output).resolve()
    assert executable.is_file() and not executable.is_symlink() and not executable.is_junction(), 'runner missing/linked'
    digest = sha(executable)
    target = ARTIFACTS / 'runners' / digest / executable.name
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        assert sha(target) == digest, 'immutable runner was modified'
    else:
        shutil.copyfile(executable, target)
    assert sha(target) == digest, 'copy changed runner'
    identity = ROOT / f'research/plan{CURRENT}-identity.json'
    assert identity.is_file(), 'current source identity missing'
    subprocess.run([sys.executable,str(ROOT/f'tools/verify_plan{CURRENT}.py')],check=True)
    subprocess.run([sys.executable,str(ROOT/'tools/prepare_sdk.py'),'--verify-only'],check=True)
    identity_copy = None
    if identity.exists():
        identity_copy=ARTIFACTS/'identities'/(sha(identity)+'.json')
        identity_copy.parent.mkdir(parents=True,exist_ok=True)
        if identity_copy.exists():assert sha(identity_copy)==sha(identity),'identity archive changed'
        else:shutil.copyfile(identity,identity_copy)
    git = subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,text=True)
    observed_commit = git.stdout.strip() if git.returncode==0 else None
    provenance = None
    if build_proof is None:
        configured = os.environ.get('M2_RUNNER_BUILD_PROOF')
        registered = ARTIFACTS/'build-proofs'/digest/'proof.json'
        build_proof = pathlib.Path(configured) if configured else (registered if registered.is_file() else None)
        allow_reuse = allow_reuse or os.environ.get('M2_ALLOW_RUNNER_REUSE') == '1'
    if build_proof is not None:
        from runner_provenance import verify_build
        provenance = verify_build(build_proof, executable, json.loads(identity.read_text()), allow_reuse=allow_reuse, identity_path=f'research/plan{CURRENT}-identity.json')
    # An arbitrary binary is useful as an unbound observation but cannot be
    # attributed to whichever checkout happens to be open.
    if provenance is None:
        result = dict(schema='autographics.runner/1', role=role, source_kind='unverified-executable', original=str(executable), path=str(target),
                      sha256=digest, origin_verified=False, code_commit=None, source_identity_path=None, source_identity_sha256=None,
                      sdk_sha256=None, locks={}, configuration={k:os.environ.get(k) for k in ['RUSTFLAGS','RUST_MIN_STACK','RUSTC_WRAPPER']},
                      observed_checkout_commit=observed_commit)
    else:
        proof = json.loads(pathlib.Path(build_proof).read_text(encoding='utf8'))
        build_identity = pathlib.Path(proof['source_identity_path']).resolve()
        result = dict(schema='autographics.runner/1', role=role, source_kind='controlled-build', original=str(executable), path=str(target),
                      sha256=digest, origin_verified=True, code_commit=provenance['build_commit'], verification_commit=provenance['verification_commit'],
                      source_identity_path=str(build_identity), source_identity_sha256=provenance['source_identity_sha256'],
                      verification_source_identity_path=str(identity_copy), verification_source_identity_sha256=sha(identity),
                      sdk_sha256=proof['sdk_sha256'], locks=proof['locks'], configuration=proof['configuration'],
                      source_base_commit=json.loads(build_identity.read_text())['base_commit'], build_provenance=provenance)
    with output.open('x', encoding='utf-8') as f:
        json.dump(result, f, indent=2)
    return result


def verify_runner(receipt, allow_reuse=False):
    assert receipt.get('origin_verified') is not False, 'unverified runner cannot be accepted as bound evidence'
    assert receipt['schema'] == 'autographics.runner/1', 'runner receipt schema'
    p = pathlib.Path(receipt['path'])
    assert p.is_file() and not p.is_symlink() and not p.is_junction(), 'runner missing/linked'
    assert sha(p) == receipt['sha256'], 'runner bytes changed'
    assert p.parent.name == receipt['sha256'], 'runner is not in SHA directory'
    if receipt.get('source_identity_path'):
        assert sha(receipt['source_identity_path'])==receipt['source_identity_sha256'], 'source identity archive changed'
    if receipt.get('origin_verified') is True:
        from runner_provenance import verify_build
        build = receipt.get('build_provenance')
        assert isinstance(build, dict), 'missing runner build proof'
        proof_path = pathlib.Path(build.get('proof_path', ''))
        assert proof_path.is_file() and sha(proof_path)==build.get('proof_sha256'), 'runner build proof changed'
        identity = ROOT / f'research/plan{CURRENT}-identity.json'
        assert receipt.get('verification_source_identity_sha256') == sha(identity), 'verification source identity changed'
        checked = verify_build(proof_path,p,json.loads(identity.read_text()),allow_reuse=allow_reuse or os.environ.get('M2_ALLOW_RUNNER_REUSE')=='1',identity_path=f'research/plan{CURRENT}-identity.json')
        assert checked == build and receipt.get('code_commit') == checked['build_commit'], 'runner build attribution changed'
    return p


def selected_tests(raw):
    """Parse top-level libtest blocks, excluding nested portable_child processes."""
    blocks, stack, binary = [], [], None
    for line in raw.splitlines():
        line = line.strip()
        header = re.search(r'(?:Running (?:unittests |tests[\\/])|Doc-tests )', line)
        if header:
            assert not stack and binary is None, 'interrupted test binary'
            binary = re.sub(r' \(target.*', '', line.strip())
        running = re.fullmatch(r'running (\d+) tests?', line.strip())
        if running:
            assert binary is not None, 'missing test entrypoint'
            assert not stack or stack[-1]['pending'] is not None, 'unattributed nested test block'
            stack.append(dict(count=int(running[1]), seen=set(), ids=[], pending=None))
            continue
        if not stack:
            assert line not in ('ok', 'FAILED', 'ignored'), 'unframed case completion'
            assert not re.match(r'(?:test result:|test .+? \.\.\.|failures:|error: test failed)',line.strip()), 'unframed test result'
            continue
        current = stack[-1]
        test = re.match(r'test (.+?) \.\.\.\s*(.*)', line)
        if test:
            assert current['pending'] is None, 'unfinished case before next case'
            assert test[1] not in current['seen'], 'duplicate announced case'
            assert len(current['seen']) < current['count'], 'case beyond declared count'
            current['seen'].add(test[1])
            current['pending'] = test[1]
            if test[2] in ('ok', 'FAILED', 'ignored'):
                assert test[2] == 'ok', 'failed/skipped selected case'
                current['ids'].append(test[1]); current['pending'] = None
        elif line.strip() in ('ok', 'FAILED', 'ignored'):
            assert current['pending'] is not None, 'unowned case completion'
            assert line.strip() == 'ok', 'failed/skipped selected case'
            current['ids'].append(current['pending']); current['pending'] = None
        end = re.match(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored;', line)
        if end:
            assert current['pending'] is None, 'unfinished case at summary'
            assert end[1] == 'ok' and int(end[3]) == 0 and int(end[4]) == 0, 'failed/skipped test block'
            assert int(end[2]) == current['count'], 'partial test block'
            assert current['seen'] == set(current['ids']) and len(current['ids']) == len(current['seen']) == current['count'], 'missing/duplicate selected IDs'
            stack.pop()
            if not stack:
                assert binary is not None, 'missing test entrypoint'
                blocks.append(dict(binary=binary, ids=current['ids']))
                binary = None
    assert not stack and binary is None and blocks, 'incomplete or absent test results'
    return blocks


def check_selection(raw, expected, exit_code=0):
    assert exit_code == 0, 'test command failed/interrupted'
    blocks = selected_tests(raw)
    actual = {b['binary']: set(b['ids']) for b in blocks}
    assert len(actual) == len(blocks), 'duplicate binary execution'
    for binary, ids in expected.items():
        assert binary in actual and (set(ids) <= actual[binary] if ids else not actual[binary]), '0selected or missing required test IDs'
    return blocks


def validate_run(receipt, runner):
    verify_runner(runner)
    assert receipt['completed'] is True and receipt['exit'] == receipt['expected_exit'], 'incomplete/failed invocation'
    assert receipt['runner_sha256'] == runner['sha256'], 'different runner'
    log = pathlib.Path(receipt['raw_log'])
    assert sha(log) == receipt['raw_sha256'], 'raw evidence changed'
    raw = log.read_text(encoding='utf-8')
    if receipt['expected_exit']:
        assert receipt.get('oracle') and receipt['oracle'] in raw, 'negative missed original oracle'
    if 'selected' in receipt:
        check_selection(raw, receipt['selected'], receipt['exit'])
    for item in receipt.get('reports', []):
        p = pathlib.Path(item['path'])
        assert sha(p) == item['sha256'], 'report bytes changed'
        d = json.loads(p.read_text())
        assert d['runner_sha256'] == runner['sha256'], 'report runner mismatch'
        assert d.get('completed', True) is True, 'partial report'
        if 'passed' in d:
            assert d['passed'] is item.get('expected_passed', receipt['expected_exit']==0), 'unexpected native outcome'
            if 'expected_phase' in item:assert d['spatial_phase']==item['expected_phase'], 'native completion phase'
        else:assert d.get('completed') is True, 'partial report'
    return True


def self_check():
    with tempfile.TemporaryDirectory() as temp:
        root = pathlib.Path(temp)
        payload = b'original actual runner fixture'
        digest = hashlib.sha256(payload).hexdigest()
        exe = root/digest/'runner.exe'; exe.parent.mkdir(); exe.write_bytes(payload)
        r = dict(schema='autographics.runner/1', path=str(exe), sha256=digest)
        verify_runner(r)
        def reject(fn):
            try: fn()
            except (AssertionError, FileNotFoundError, KeyError, ValueError): return
            raise AssertionError('false green control')
        # Rebuilding original target cannot change frozen payload.
        target = root/'target.exe'; target.write_bytes(b'rebuilt'); verify_runner(r)
        exe.write_bytes(b'tampered'); reject(lambda: verify_runner(r)); exe.write_bytes(payload)
        exe.unlink(); reject(lambda: verify_runner(r)); exe.write_bytes(payload); verify_runner(r)
        archive=root/'identity.json';archive.write_bytes(b'archived identity');r['source_identity_path']=str(archive);r['source_identity_sha256']=sha(archive);verify_runner(r)
        archive.write_bytes(b'changed identity');reject(lambda:verify_runner(r));archive.write_bytes(b'archived identity');verify_runner(r)
        raw = 'Running tests/example.rs (target/debug/deps/example.exe)\nrunning 2 tests\ntest outer ... \nrunning 1 test\ntest portable_child ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;\nok\ntest portable_child ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n'
        expected = {'Running tests/example.rs':['outer','portable_child']}
        assert len(check_selection(raw, expected)[0]['ids']) == 2
        for bad in [raw[:raw.rfind('test result:')], raw.replace('2 passed','1 passed'), raw.replace(' ... ok',' ... ignored'), 'Running tests/example.rs\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;']:
            reject(lambda: check_selection(bad, expected))
        reject(lambda: check_selection(raw, expected, 1))
        for tail in ['Running tests/unfinished.rs\n','Doc-tests unfinished\n','test result: FAILED. 0 passed; 1 failed; 0 ignored;\n']:
            reject(lambda tail=tail:check_selection(raw+tail,expected))
        for bad in [raw[:raw.rfind('test result:')]+'test interrupted ...\n'+raw[raw.rfind('test result:'):],
                    raw.replace('test outer ...','test interrupted ...\ntest outer ...',1),
                    raw.replace('test portable_child ... ok','test interrupted ...\ntest portable_child ... ok',1)]:
            reject(lambda bad=bad:check_selection(bad,expected))
        assert check_selection('Doc-tests example\nrunning 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;', {'Doc-tests example':[]})
        log=root/'raw.txt';log.write_text(raw)
        report=root/'report.json';report.write_text(json.dumps(dict(completed=True,runner_sha256=digest)))
        run=dict(completed=True,exit=0,expected_exit=0,runner_sha256=digest,raw_log=str(log),raw_sha256=sha(log),selected=expected,reports=[dict(path=str(report),sha256=sha(report))])
        validate_run(run,r)
        report.write_text(json.dumps(dict(completed=True,runner_sha256='0'*64)));run['reports'][0]['sha256']=sha(report);reject(lambda:validate_run(run,r))
        run['reports']=[];run['expected_exit']=1;run['exit']=1;run['oracle']='named original rejection';reject(lambda:validate_run(run,r))
        log.write_text('named original rejection');run['raw_sha256']=sha(log);run.pop('selected');validate_run(run,r)
        run['completed']=False;reject(lambda:validate_run(run,r));run['completed']=True;validate_run(run,r)
        report.write_text(json.dumps(dict(passed=False,runner_sha256=digest,spatial_phase=5)));run['reports']=[dict(path=str(report),sha256=sha(report),expected_passed=False,expected_phase=5)];validate_run(run,r)
        run['reports'][0]['expected_passed']=True;reject(lambda:validate_run(run,r));run['reports'][0]['expected_passed']=False;validate_run(run,r)
    print('PASS actual frozen-file drift/missing/restoration, target rebuild, nested child denominator, zero/skip/partial/interruption, report mismatch, named negative and restored controls')


if __name__ == '__main__':
    p=argparse.ArgumentParser();p.add_argument('--self-check',action='store_true');p.add_argument('--record-runner',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path);p.add_argument('--role',choices=['producer','consumer'],default='producer');p.add_argument('--verify-runner',type=pathlib.Path);p.add_argument('--validate-run',type=pathlib.Path);p.add_argument('--build-proof',type=pathlib.Path);p.add_argument('--allow-reuse',action='store_true');a=p.parse_args()
    if a.self_check:self_check()
    elif a.record_runner:
        assert a.output, '--output required'
        print(json.dumps(record_runner(a.record_runner,a.output,a.role,build_proof=a.build_proof,allow_reuse=a.allow_reuse)))
    elif a.verify_runner:
        runner=json.loads(a.verify_runner.read_text());verify_runner(runner)
        if a.validate_run:validate_run(json.loads(a.validate_run.read_text()),runner)
        print('PASS immutable runner/evidence')
    else:p.error('choose an action')
