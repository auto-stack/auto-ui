# PLAN-033 T-04: 009-article-feed VM leg (theme migration acceptance).
# Original business preserved: ReadArticle1/2/3 print feedback ("Reading
# article N"), article/author/date/tag fields, emoji thumb placeholders
# (original content assets — no remote images). Theme family/mode round-trip
# via fixture channel.
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


def find_button(snapshot, label):
    """Last-resort label lookup: the button line containing the label text."""
    for line in snapshot.splitlines():
        if 'button #' in line and label in line:
            for p in line.split():
                if p.startswith('#'):
                    return p.rstrip(':,{')
    return None


def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../../../../'))
    auto_bin = os.environ.get('AUTO_BIN') or os.path.join(root_dir, 'target/debug/auto.exe')
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '../'))
    shot_dir = os.path.join(app_dir, 'tests', 'screenshots')
    os.makedirs(shot_dir, exist_ok=True)

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTOUI_TEST_FIXTURES='1')
    print(f'[*] Starting VM for 009 on MCP port {port}...')
    out_path = os.path.join(shot_dir, '009_vm_app_output.log')
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

        # Original article content (three articles, fields, emoji thumbs).
        for marker in ('Blog Posts', 'Read more', 'Getting Started with shadcn/ui Components',
                       'Building Accessible Web Applications',
                       'Modern Design Systems with Tailwind CSS',
                       'Sarah Chen', 'Marcus Rodriguez', 'Emma Thompson',
                       'Tutorial', 'Accessibility', 'Design Systems'):
            if marker not in snap:
                failures.append(f'content marker missing: {marker}')
        for banned in ('Settings', 'ExampleHeader', 'Route', 'Fetch'):
            if banned in snap:
                failures.append(f'new business surface present: {banned}')
        print('[+] content markers + no-new-business checked')

        # ReadArticle1/2/3 original print feedback (three "Read more" buttons
        # in document order -> articles 1/2/3).
        read_ids = []
        for line in snap.splitlines():
            if 'button #' in line and '"Read more →"' in line:
                for p in line.split():
                    if p.startswith('#'):
                        read_ids.append(p.rstrip(':,{'))
        if len(read_ids) != 3:
            failures.append(f'expected 3 Read more buttons, found {len(read_ids)}')
        else:
            expects = ['Reading article 1', 'Reading article 2', 'Reading article 3']
            for i, (bid, expect) in enumerate(zip(read_ids, expects), 1):
                res = client.text(client.call('autoui_action',
                                              {'element_id': bid.lstrip('#'), 'action': 'press'}))
                time.sleep(0.6)
                printed = False
                try:
                    with open(out_path, encoding='utf-8', errors='replace') as f:
                        printed = expect in f.read()
                except OSError:
                    pass
                if 'ok' in res and printed:
                    print(f'[+] ReadArticle{i} print feedback captured: {expect}')
                else:
                    failures.append(f'ReadArticle{i} feedback missing '
                                    f'(press ok={"ok" in res}, printed={printed})')

        # Theme round-trip with content preservation.
        client.call('autoui_screenshot', {'name': '009_vm_auto_light'})
        fixture(client, theme_name='Solarized', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Solarized"':
            failures.append('family switch not applied')
        client.call('autoui_screenshot', {'name': '009_vm_solarized_light'})
        fixture(client, theme_name='Solarized', dark_mode=True)
        time.sleep(1.0)
        if state_field(client, 'dark_mode') != 'true':
            failures.append('mode switch not applied')
        client.call('autoui_screenshot', {'name': '009_vm_solarized_dark'})
        fixture(client, theme_name='Auto', dark_mode=False)
        time.sleep(1.0)
        if state_field(client, 'theme_name') != '"Auto"' or state_field(client, 'dark_mode') != 'false':
            failures.append('round trip back to Auto/light not applied')
        client.call('autoui_screenshot', {'name': '009_vm_auto_light_back'})
        snap_final = client.text(client.call('autoui_snapshot', {}))
        for marker in ('Blog Posts', 'Sarah Chen', '📖'):
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
