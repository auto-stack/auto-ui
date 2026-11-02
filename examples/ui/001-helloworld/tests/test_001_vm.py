# PLAN-033 T-03: 001-helloworld VM leg (theme migration acceptance).
# Original business preserved: greeting text/structure/selectable text only —
# no new handlers, menus or business state. Theme: family/mode round-trip via
# the PLAN-623 fixture channel (state write -> design binding -> atomic scope
# publish -> projection paint), never CSS tampering.
import os
import socket
import subprocess
import sys
import time
import shutil


def pick_free_port():
    # Windows reserved ranges raise EACCES on bind — retry until a bindable
    # ephemeral port is handed out.
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
    print(f'[*] Starting VM for 001 on MCP port {port}...')
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

        # Original content preserved: greeting + selectable text, token style.
        if 'Hello, World!' not in snap:
            failures.append('greeting text missing')
        if 'text-primary' not in snap:
            failures.append('text-primary token style missing')
        if 'bg-background' not in snap:
            failures.append('root bg-background (public role adoption) missing')
        print('[+] content markers checked')

        # No new business surface: no menus/settings/handlers in the tree.
        for banned in ('Settings', 'ExampleHeader', 'ThemeMenu'):
            if banned in snap:
                failures.append(f'new business surface present: {banned}')
        print('[+] no new business surface')

        # Family/mode round-trip via fixture (state write -> design publish).
        client.call('autoui_screenshot', {'name': '001_vm_auto_light'})
        fixture(client, theme_name='Solarized', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Solarized"':
            failures.append('family switch to Solarized not applied')
        client.call('autoui_screenshot', {'name': '001_vm_solarized_light'})
        fixture(client, theme_name='Solarized', dark_mode=True)
        time.sleep(1.0)
        if state_field(client, 'dark_mode') != 'true':
            failures.append('mode switch to dark not applied')
        client.call('autoui_screenshot', {'name': '001_vm_solarized_dark'})
        # Round trip back to declared default.
        fixture(client, theme_name='Auto', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Auto"' or state_field(client, 'dark_mode') != 'false':
            failures.append('round trip back to Auto/light not applied')
        client.call('autoui_screenshot', {'name': '001_vm_auto_light_back'})
        print('[+] family/mode round-trip applied (states read back)')

        # Instance/content preservation across switches: greeting still present.
        snap2 = client.text(client.call('autoui_snapshot', {}))
        if 'Hello, World!' not in snap2:
            failures.append('greeting lost after theme round-trip')
        print('[+] content preserved across round-trip')

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
