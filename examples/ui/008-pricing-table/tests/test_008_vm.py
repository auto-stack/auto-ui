# PLAN-033 T-04: 008-pricing-table VM leg (theme migration acceptance).
# Original business preserved: BuyPlan1/2/3 print feedback ("Selected Plan:
# ..."), RECOMMENDED badge, plan names/prices/features; equal-height cards
# (items-stretch + flex-1). Theme family/mode round-trip via fixture channel.
import os
import socket
import subprocess
import sys
import time
import shutil


def pick_free_port():
    for _ in range(64):
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            s.bind(('127.0.0.1', 0))
            return s.getsockname()[1]
        except OSError:
            continue
        finally:
            s.close()
    raise RuntimeError('no bindable port found')


class Mcp:
    def __init__(self, port):
        import json
        import urllib.request
        self.url = f'http://127.0.0.1:{port}/mcp'
        self.req_id = 1
        self.json = json
        self.urllib = urllib.request

    def call(self, tool, args=None):
        req = {
            'jsonrpc': '2.0', 'id': self.req_id, 'method': 'tools/call',
            'params': {'name': tool, 'arguments': args or {}},
        }
        self.req_id += 1
        data = self.json.dumps(req).encode('utf-8')
        http = self.urllib.Request(self.url, data=data,
                                   headers={'Content-Type': 'application/json'})
        with self.urllib.urlopen(http, timeout=10) as r:
            res = self.json.loads(r.read().decode('utf-8'))
            if 'error' in res:
                raise RuntimeError(f"MCP {tool} error: {res['error']}")
            return res.get('result', {})

    def text(self, res):
        return res.get('content', [{}])[0].get('text', '')


def fixture(client, **state):
    return client.call('autoui_fixture', {'schema_version': 1, 'state': state})


def state_field(client, field):
    txt = client.text(client.call('autoui_state', {}))
    for line in txt.splitlines():
        parts = line.strip().split(':', 1)
        if len(parts) == 2 and parts[0].strip() == field:
            return parts[1].split('(')[0].strip()
    return None


def collect_buy_buttons(snapshot):
    """Order-stable BuyPlan1/2/3 ids: first two 'Buy Now' buttons then 'Contact Us'."""
    buys = [None, None, None]
    for line in snapshot.splitlines():
        if 'button #' not in line:
            continue
        pid = None
        for p in line.split():
            if p.startswith('#'):
                pid = p.rstrip(':,{')
        if pid is None:
            continue
        if '"Buy Now"' in line:
            if buys[0] is None:
                buys[0] = pid
            elif buys[1] is None:
                buys[1] = pid
        elif '"Contact Us"' in line:
            buys[2] = pid
    return {'BuyPlan1': buys[0], 'BuyPlan2': buys[1], 'BuyPlan3': buys[2]}


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../'))
    auto_bin = os.environ.get('AUTO_BIN') or os.path.join(root_dir, 'target/debug/auto.exe')
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../'))
    shot_dir = os.path.join(app_dir, 'tests', 'screenshots')
    os.makedirs(shot_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTOUI_TEST_FIXTURES='1')
    print(f'[*] Starting VM for 008 on MCP port {port}...')
    out_path = os.path.join(shot_dir, '008_vm_app_output.log')
    out_file = open(out_path, 'w', encoding='utf-8')
    proc = subprocess.Popen([auto_bin, 'run', '-r', 'vm'], cwd=app_dir, env=env,
                            stdout=out_file, stderr=subprocess.STDOUT)

    failures = []
    try:
        client = Mcp(port)
        snap = ''
        deadline = time.time() + 20
        while time.time() < deadline:
            try:
                snap = client.text(client.call('autoui_snapshot', {}))
                if 'tree:' in snap:
                    break
            except Exception:
                pass
            time.sleep(0.3)
        else:
            print('[-] VM did not become ready', file=sys.stderr)
            sys.exit(1)
        print('[+] VM ready')

        # Original content: pac dark default boot, plan data, badge.
        for marker in ('Pricing Plans', 'Buy Now', 'RECOMMENDED', 'Single Developer',
                       'Team', 'Enterprise', '$39', '$99', 'Lifetime Support'):
            if marker not in snap:
                failures.append(f'content marker missing: {marker}')
        for banned in ('Settings', 'ExampleHeader', 'Subscribe', 'Checkout'):
            if banned in snap:
                failures.append(f'new business surface present: {banned}')
        # Equal-height layout classes (items-stretch + flex-1 cards).
        if 'items-stretch' not in snap or 'flex-1' not in snap:
            failures.append('equal-height layout classes missing (items-stretch/flex-1)')
        print('[+] content markers + equal-height classes checked')

        # BuyPlan1/2/3 original print feedback: handler dispatch + output log.
        btn_ids = collect_buy_buttons(snap)
        if not all(btn_ids.values()):
            failures.append(f'buy buttons not all found: {btn_ids}')
        else:
            expects = {'BuyPlan1': 'Selected Plan: Single Developer',
                       'BuyPlan2': 'Selected Plan: Team',
                       'BuyPlan3': 'Selected Plan: Enterprise'}
            for name in ('BuyPlan1', 'BuyPlan2', 'BuyPlan3'):
                bid = btn_ids[name]
                res = client.text(client.call('autoui_action',
                                              {'element_id': bid.lstrip('#'), 'action': 'press'}))
                time.sleep(0.6)
                printed = False
                try:
                    with open(out_path, encoding='utf-8', errors='replace') as f:
                        printed = expects[name] in f.read()
                except OSError:
                    pass
                if 'ok' in res and printed:
                    print(f'[+] {name} print feedback captured: {expects[name]}')
                else:
                    failures.append(f'{name} feedback missing (press ok={"ok" in res}, printed={printed})')

        # Theme round-trip with content preservation.
        client.call('autoui_screenshot', {'name': '008_vm_auto_light'})
        fixture(client, theme_name='Solarized', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Solarized"':
            failures.append('family switch not applied')
        client.call('autoui_screenshot', {'name': '008_vm_solarized_light'})
        fixture(client, theme_name='Solarized', dark_mode=True)
        time.sleep(1.0)
        if state_field(client, 'dark_mode') != 'true':
            failures.append('mode switch not applied')
        client.call('autoui_screenshot', {'name': '008_vm_solarized_dark'})
        fixture(client, theme_name='Auto', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Auto"' or state_field(client, 'dark_mode') != 'false':
            failures.append('round trip back to Auto/light not applied')
        client.call('autoui_screenshot', {'name': '008_vm_auto_light_back'})
        snap_final = client.text(client.call('autoui_snapshot', {}))
        for marker in ('Pricing Plans', 'RECOMMENDED', 'items-stretch'):
            if marker not in snap_final:
                failures.append(f'content not preserved across round-trip: {marker}')
        print('[+] family/mode round-trip with content preservation')

        shots = os.path.join(app_dir, 'src/front/tmp')
        if os.path.isdir(shots):
            for f in os.listdir(shots):
                if f.endswith('.png'):
                    shutil.copy2(os.path.join(shots, f), os.path.join(shot_dir, f))

        if failures:
            print(f'[-] {len(failures)} assertion(s) failed: {failures}', file=sys.stderr)
            sys.exit(1)
        print('[+] All VM assertions passed!')
    finally:
        out_file.close()
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == '__main__':
    main()
