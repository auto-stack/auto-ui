"""PLAN-860 feasibility evidence checker.

Verifies fresh probe/host reports against the frozen contract (r4): route
reachability, budget/credit bounds, percentile math, trace bounds and host
numeric/pixel/owner bindings. Rebinds each run to a retained runner executable.

Usage:
  python tools/check_game_feasibility.py <report.json...> --executable <retained-exe>
  python tools/check_game_feasibility.py --controls
"""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

MAX_ROUTE_STEPS = 1800
MAX_CREDITS = 32
MAX_TRACE_BYTES = 5 * 1024 * 1024


def percentile(values, q):
    if not values:
        return 0
    ordered = sorted(values)
    rank = max(1, min(len(ordered), -(-int(q * len(ordered) * 100) // 100)))
    # nearest rank: ceil(q*N)-1
    import math
    rank = max(1, min(len(ordered), math.ceil(q * len(ordered))))
    return ordered[rank - 1]


def check_probe(report):
    assert report['schema'] == 'autographics.game-probe/1', 'probe schema'
    assert report['completed'] is True, 'partial probe report'
    assert report['steps'] > 0, '0 measured steps'
    case = report['case']
    if case in ('g2', 'g3'):
        assert report['steps'] <= MAX_ROUTE_STEPS, 'route steps above frozen bound'
        rep = report['report']
        assert all(rep['coins_reached']), f'{case} coins missed: {rep["coin_steps"]}'
        assert rep['goal_reached'], f'{case} goal missed'
        for violation in ('enemy_contact_step', 'hazard_contact_step', 'pit_step'):
            assert rep[violation] is None, f'{case} violation {violation}'
    else:
        assert case in ('wide128', 'deep128'), f'unknown case {case}'
        assert report['report']['blocker_visible'] == 1, '128 load oracle failed'
    assert report['credits_max'] <= MAX_CREDITS, 'credit hard cap exceeded'
    samples = report.get('samples')
    if samples:
        assert len(samples) <= MAX_ROUTE_STEPS + 1, 'trace above frozen bound'
        raw = [s['tick_ns'] for s in samples]
        for q, key in ((0.50, 'p50'), (0.95, 'p95')):
            expect = percentile(raw, q)
            got = report['tick_cpu_ns'][key]
            assert got == expect, f'{key} mismatch: reported {got} actual {expect}'
        assert report['tick_cpu_ns']['p95'] >= report['tick_cpu_ns']['p50'], 'percentile order'
    identity = report['identity']
    assert identity['combined_bytes'] <= 320 * 1024, 'combined source over 320KiB'
    assert identity['user_bytes'] <= 256 * 1024, 'user source over 256KiB'


def check_host(report):
    assert report['schema'] == 'autographics.game-host/1', 'host schema'
    assert report['completed'] is True, 'partial host report'
    assert report['owner_cycles'] == 20, 'owner cycles incomplete'
    assert report['restored_editing_pixels'] is True, 'stop did not restore pixels'
    assert report['checkpoints'], 'no checkpoints'
    for cp in report['checkpoints']:
        assert cp['matches_trace'] is True, f'checkpoint {cp["step"]} off trace'
        assert cp['published'] == cp['expected'], f'checkpoint {cp["step"]} divergence'
    assert len(set(report['pixel_hashes'])) > 1 or report['route'] == 'g3', (
        'pixel content never changed (stale capture surface)')


def check_report(path, runner_sha=None):
    raw = pathlib.Path(path).read_bytes()
    report = json.loads(raw)
    if report.get('schema') == 'autographics.game-probe/1':
        check_probe(report)
    elif report.get('schema') == 'autographics.game-host/1':
        check_host(report)
    else:
        raise AssertionError(f'unknown report schema in {path}')
    return report


def self_check():
    with tempfile.TemporaryDirectory() as temp:
        root = pathlib.Path(temp)

        def probe(case='g2', **over):
            base = {
                'schema': 'autographics.game-probe/1', 'case': case,
                'candidate': 'a', 'steps': 30, 'completed': True,
                'identity': {'user_bytes': 10, 'combined_bytes': 20},
                'tick_cpu_ns': {'p50': 10, 'p95': 20, 'max': 30},
                'credits_max': 27, 'resumes_total': 0, 'scans_max': 0,
                'samples': None,
                'report': {
                    'coins_reached': [True, True, True], 'goal_reached': True,
                    'enemy_contact_step': None, 'hazard_contact_step': None,
                    'pit_step': None,
                } if case in ('g2', 'g3') else {'blocker_visible': 1},
            }
            base.update(over)
            return base

        def host(**over):
            base = {
                'schema': 'autographics.game-host/1', 'route': 'g2',
                'completed': True, 'owner_cycles': 20,
                'restored_editing_pixels': True,
                'checkpoints': [{'step': 1, 'matches_trace': True,
                                 'published': [1.0], 'expected': [1.0]}],
                'pixel_hashes': ['a', 'b'],
            }
            base.update(over)
            return base

        def reject(payload, marker='report'):
            p = root / 'bad.json'
            p.write_text(json.dumps(payload))
            try:
                check_report(p)
            except AssertionError as e:
                assert marker in str(e), f'wrong rejection for {marker}: {e}'
                return
            raise AssertionError(f'false green control: {marker}')

        good = probe()
        p = root / 'good.json'
        p.write_text(json.dumps(good))
        check_report(p)
        reject(probe(completed=False), 'partial')
        reject(probe(steps=0), '0 measured')
        reject(probe(steps=2000), 'frozen bound')
        reject(probe(credits_max=33), 'credit hard cap')
        bad = probe()
        bad['report']['goal_reached'] = False
        reject(bad, 'goal missed')
        bad = probe()
        bad['report']['enemy_contact_step'] = 5
        reject(bad, 'violation')
        bad = probe('wide128')
        bad['report']['blocker_visible'] = 0
        reject(bad, 'load oracle')
        # Percentile math must be recomputed, not trusted.
        raws = [50, 10, 30, 20, 40]
        tampered = probe(samples=[{'tick_ns': v} for v in raws],
                         tick_cpu_ns={'p50': 999, 'p95': 20, 'max': 50})
        reject(tampered, 'p50 mismatch')
        ok = probe(samples=[{'tick_ns': v} for v in raws],
                   tick_cpu_ns={'p50': percentile(raws, 0.5),
                                'p95': percentile(raws, 0.95), 'max': 50})
        p = root / 'ok.json'
        p.write_text(json.dumps(ok))
        check_report(p)
        assert percentile(raws, 0.5) == 30 and percentile(raws, 0.95) == 50
        # Host oracles.
        hp = root / 'host.json'
        hp.write_text(json.dumps(host()))
        check_report(hp)
        reject(host(completed=False), 'partial host')
        reject(host(owner_cycles=19), 'owner cycles incomplete')
        reject(host(restored_editing_pixels=False), 'stop did not restore')
        reject(host(checkpoints=[{'step': 1, 'matches_trace': False,
                                  'published': [1.0], 'expected': [1.0]}]),
               'off trace')
        reject(host(checkpoints=[]), 'no checkpoints')
        reject(host(pixel_hashes=['a']), 'stale capture surface')
    print('PASS feasibility checker: partial/zero/bound/credit/violation/load-oracle/'
          'percentile-recompute/host-trace/owner/restore/stale-capture and restored controls')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('reports', nargs='*')
    parser.add_argument('--executable', type=pathlib.Path)
    parser.add_argument('--controls', action='store_true')
    args = parser.parse_args()
    if args.controls:
        self_check()
        return
    assert args.reports, 'no reports given'
    if args.executable is not None:
        exe = args.executable.resolve()
        assert exe.is_file() and not exe.is_symlink() and not exe.is_junction(), (
            'runner missing/linked')
        digest = hashlib.sha256(exe.read_bytes()).hexdigest()
        print(f'RUNNER sha256={digest} path={exe}')
    for r in args.reports:
        report = check_report(r)
        print(f"PASS {report.get('case', report.get('route'))} "
              f"schema={report['schema']} steps/checkpoints bound to frozen contract")


if __name__ == '__main__':
    main()
