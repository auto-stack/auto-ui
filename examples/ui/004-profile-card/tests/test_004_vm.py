# PLAN-033 T-03: 004-profile-card VM leg (theme migration acceptance).
# Original business preserved: profile fields display only — Follow/Message
# buttons have no handlers (no new business). Remote avatar keeps real-URL
# semantics: success path renders the image, failure path (fixture-rewritten
# URL) shows the placeholder — no permanent source replacement. Theme:
# family/mode round-trip via the fixture channel.
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
    print(f'[*] Starting VM for 004 on MCP port {port}...')
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

        # Original profile content + action buttons (no new business).
        for marker in ('Jane Cooper', 'Full Stack Developer', 'Follow', 'Message', 'Active'):
            if marker not in snap:
                failures.append(f'content marker missing: {marker}')
        for banned in ('Settings', 'ExampleHeader', 'Subscribe', 'Payment'):
            if banned in snap:
                failures.append(f'new business surface present: {banned}')
        print('[+] content markers + no-new-business checked')

        # Follow/Message carry no onclick in source — pressing must be a safe
        # no-op (no handler registered, app stays alive, content unchanged).
        follow_id = None
        for line in snap.splitlines():
            if 'Follow' in line:
                for p in line.split():
                    if p.startswith('#'):
                        follow_id = p.rstrip(':,{')
        if follow_id:
            client.call('autoui_action', {'element_id': follow_id.lstrip('#'), 'action': 'press'})
            time.sleep(0.5)
            snap_after = client.text(client.call('autoui_snapshot', {}))
            if 'Jane Cooper' not in snap_after:
                failures.append('app state broken after handler-less button press')
            print('[+] handler-less Follow press is a safe no-op')
        else:
            failures.append('Follow button not found')

        # Avatar: success path (real remote URL) + failure placeholder via
        # controlled fixture rewrite (original source untouched afterwards).
        client.call('autoui_screenshot', {'name': '004_vm_avatar_success'})
        fixture(client, avatar_url='https://invalid.invalid/no-such-avatar.png')
        time.sleep(2.5)  # DNS failure path
        snap_fail = client.text(client.call('autoui_snapshot', {}))
        if 'Jane Cooper' not in snap_fail:
            failures.append('profile content lost during avatar failure path')
        client.call('autoui_screenshot', {'name': '004_vm_avatar_failure'})
        fixture(client, avatar_url='https://cn.cravatar.com/avatar/e1e7ba949ade0936e071132d2edd3b3c.png')
        time.sleep(2.5)
        print('[+] avatar success/failure paths exercised (source restored)')

        # Theme round-trip: Auto/light (original look) -> Solarized/dark -> back.
        fixture(client, theme_name='Solarized', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Solarized"':
            failures.append('family switch not applied')
        client.call('autoui_screenshot', {'name': '004_vm_solarized_light'})
        fixture(client, theme_name='Solarized', dark_mode=True)
        time.sleep(1.0)
        client.call('autoui_screenshot', {'name': '004_vm_solarized_dark'})
        fixture(client, theme_name='Auto', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Auto"' or state_field(client, 'dark_mode') != 'false':
            failures.append('round trip back to Auto/light not applied')
        snap_final = client.text(client.call('autoui_snapshot', {}))
        for marker in ('Jane Cooper', 'Full Stack Developer'):
            if marker not in snap_final:
                failures.append(f'content not preserved across theme round-trip: {marker}')
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
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == '__main__':
    main()
