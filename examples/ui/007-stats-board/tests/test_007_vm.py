# PLAN-033 T-04: 007-stats-board VM leg (theme migration acceptance).
# Original business preserved: dashboard display only — no handlers, no new
# filter/refresh/network interactions. Category-identity colors (stat chips,
# avatar fills) stay fixed literals per the mapping table (documented in
# app.at); theme family/mode round-trip via the fixture channel.
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


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../'))
    auto_bin = os.environ.get('AUTO_BIN') or os.path.join(root_dir, 'target/debug/auto.exe')
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../'))
    shot_dir = os.path.join(app_dir, 'tests', 'screenshots')
    os.makedirs(shot_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTOUI_TEST_FIXTURES='1')
    print(f'[*] Starting VM for 007 on MCP port {port}...')
    proc = subprocess.Popen([auto_bin, 'run', '-r', 'vm'], cwd=app_dir, env=env)

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

        # Original dashboard content (numbers/labels/graphics fields).
        for marker in ('Dashboard', 'Campaign Insights', 'Campaign Alerts', 'System Alerts',
                       'Promotional Offers', 'Traffic Distribution', 'Campaign Performance',
                       'Team Performance', 'Sarah Johnson', 'Mike Chen', 'Emily Davis',
                       'Overall Progress', '85.7%'):
            if marker not in snap:
                failures.append(f'content marker missing: {marker}')
        print('[+] content markers checked')

        # Category-identity colors stay fixed literals (mapping table): all four
        # stat chips + three avatar fills present in snapshot styles.
        for cls in ('bg-red-100', 'bg-blue-100', 'bg-green-100', 'bg-purple-100'):
            if cls not in snap:
                failures.append(f'category chip literal missing: {cls}')
        # Themed surfaces follow the package tokens.
        for cls in ('bg-card', 'bg-background', 'border-border'):
            if cls not in snap:
                failures.append(f'themed token class missing: {cls}')
        print('[+] category literals + themed tokens present')

        # No new business surface.
        for banned in ('Settings', 'ExampleHeader', 'Refresh', 'Filter'):
            if banned in snap:
                failures.append(f'new business surface present: {banned}')
        print('[+] no new business surface')

        # Theme round-trip via fixture with content preservation.
        client.call('autoui_screenshot', {'name': '007_vm_auto_light'})
        fixture(client, theme_name='Solarized', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Solarized"':
            failures.append('family switch not applied')
        client.call('autoui_screenshot', {'name': '007_vm_solarized_light'})
        fixture(client, theme_name='Solarized', dark_mode=True)
        time.sleep(1.0)
        if state_field(client, 'dark_mode') != 'true':
            failures.append('mode switch not applied')
        client.call('autoui_screenshot', {'name': '007_vm_solarized_dark'})
        fixture(client, theme_name='Auto', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Auto"' or state_field(client, 'dark_mode') != 'false':
            failures.append('round trip back to Auto/light not applied')
        client.call('autoui_screenshot', {'name': '007_vm_auto_light_back'})

        snap_final = client.text(client.call('autoui_snapshot', {}))
        for marker in ('Dashboard', 'Campaign Alerts', 'Sarah Johnson', 'bg-red-100'):
            if marker not in snap_final:
                failures.append(f'content/literals not preserved across round-trip: {marker}')
        print('[+] family/mode round-trip with content + literals preserved')

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
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == '__main__':
    main()
