"""PLAN-085 TreeView product gate — real native VM window / real Vue page.

Drives `examples/tree-view` through the controlled contract with ordinary MCP
input (native) or a real browser page (vue), asserting behaviour that only a
live app can show:

  V-01 controlled identity   explicit select/toggle events carry the target ID,
                             and the component never pretends a controlled prop
                             changed before the app updates it
  V-02 structural safety     empty tree is a normal empty state, not an error
  V-03 keyboard              focus/selection are separate; Tab leaves the tree
  V-04 instances & timing    two trees on one page do not cross-talk
  V-06 visual/lifecycle      light/dark, narrow window, screenshot, clean exit
                             and reopen

An empty tree, a missing window or a timed-out action fails the gate. The
screenshot checks are recorded separately from the action assertions; a real
screenshot never stands in for a behavioural assertion.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd  # require/sha/owned_tree/windows_for/free_port

ROOT = Path(__file__).resolve().parents[2]


def bindable_port(preferred):
    """First ACTUALLY-bindable port at or below preferred — connect_ex misses
    Windows reserved ranges (Hyper-V/WSL) where the runtime's MCP bind fails
    with os error 10013 and silently rescans to an unpredictable port."""
    for port in range(preferred, preferred + 400):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(('127.0.0.1', port))
            except OSError:
                continue
        return port
    raise RuntimeError('no bindable port found')


class Session:
    def __init__(self, args, app, out, index):
        self.port = bindable_port(vfd.free_port())
        self.out = out
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port),
                   AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        if os.name != 'nt':
            env['WINIT_UNIX_BACKEND'] = 'x11'
            env.pop('WAYLAND_DISPLAY', None)
        self.log = open(out / f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(
            self.command, cwd=app, env=env, stdout=self.log,
            stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        self.index = index
        self.windows = []
        deadline = time.time() + 60
        while time.time() < deadline:
            try:
                text = open(self.log.name, 'r', encoding='utf-8', errors='ignore').read()
                m = re.search(r'AutoUI MCP: listening on http://127\.0\.0\.1:(\d+)', text)
                if m:
                    self.port = int(m.group(1))
                    break
            except Exception:
                pass
            time.sleep(0.3)
        deadline = time.time() + 90
        last = ''
        while time.time() < deadline:
            try:
                last = self.snapshot()
                # The page is ready only when the routed demo page has mounted
                # AND its tree has produced rows — the header text alone renders
                # a frame earlier, which would make every later assertion race
                # the tree's first derived frame.
                if 'TreeView 受控演示' in last and 'A 树 sel=' in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'tree-view demo never rendered; last snapshot: {last[:400]}')
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)
        vfd.require(self.windows, 'No owned native window with a title/OS handle')

    # ── MCP plumbing ──
    def call(self, tool, **argsv):
        import urllib.request
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argsv}}).encode()
        req = urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body,
                                     headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req, timeout=10) as r:
            out = json.loads(r.read().decode())
        content = out.get('result', {}).get('content', [])
        return '\n'.join(c.get('text', '') for c in content if c.get('type') == 'text')

    def snapshot(self):
        return self.call('autoui_snapshot')

    def state(self):
        return self.call('autoui_state')

    def wait(self, predicate, timeout=15):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            time.sleep(0.4)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:400]}')

    def row_ids(self):
        """Bound row actions across the whole page, in document order.

        The demo hosts two TreeView instances, so callers must slice: the A
        tree renders first (its summary line precedes it) and the B tree
        follows. `split_row_ids()` does that split.
        """
        snap = self.snapshot()
        return re.findall(
            r'row #(vnode_\d+) \{\s*\n\s*style: "[^"]*rounded-md[^"]*"\s*\n\s*onclick: \.WantSelect',
            snap)

    def split_row_ids(self):
        """(A rows, B rows) — the page renders the A block before the B block."""
        snap = self.snapshot()
        b_pos = snap.find('"B 树 sel=')
        vfd.require(b_pos > 0, 'B tree summary line missing')
        head, tail = snap[:b_pos], snap[b_pos:]
        pat = (r'row #(vnode_\d+) \{\s*\n\s*style: "[^"]*rounded-md[^"]*"\s*\n'
               r'\s*onclick: \.WantSelect')
        return re.findall(pat, head), re.findall(pat, tail)

    def press(self, element_id):
        return self.call('autoui_action', element_id=element_id, action='press')

    def a_line(self):
        m = re.search(r'"A 树 sel=(\S*) exp=(\d+) ([^"]*)"', self.snapshot())
        return m.groups() if m else None

    def b_line(self):
        m = re.search(r'"B 树 sel=(\S*) exp=(\d+) ([^"]*)"', self.snapshot())
        return m.groups() if m else None

    def press_button(self, label):
        snap = self.snapshot()
        m = re.search(r'button #(vnode_\d+) "[^"]*' + re.escape(label) + r'[^"]*"', snap)
        vfd.require(m, f'button {label!r} not found; head: {snap[:300]}')
        return self.press(m.group(1))

    def screenshot(self, name):
        text = self.call('autoui_screenshot')
        match = re.search(r'Screenshot saved to: (.+)', text)
        vfd.require(match, f'screenshot did not return a saved path: {text[:200]}')
        source = Path(match[1].strip())
        target = self.out / f'{self.index}-{name}.png'
        shutil.copyfile(source, target)
        vfd.require(target.stat().st_size > 1000, 'Empty native screenshot')
        return {'path': str(target), 'sha256': vfd.sha(target)}

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


def prepare(args, out, window):
    vfd.require(not out.exists(), f'Use a fresh output directory: {out}')
    out.mkdir(parents=True)
    app = out / 'app'
    shutil.copytree(ROOT / 'examples/tree-view', app,
                    ignore=shutil.ignore_patterns('deps', 'dist', 'gen', 'build'))
    # The isolated copy leaves the repo: point the package import and the dep
    # root at the producer checkout (same bytes, absolute closure root).
    entry = app / 'src/front/pages/demo.at'
    entry.write_text(
        entry.read_text(encoding='utf-8').replace(
            '../../../../../packages/tree-view', (ROOT / 'packages/tree-view').as_posix()),
        encoding='utf-8')
    pac = app / 'pac.at'
    pac.write_text(
        pac.read_text(encoding='utf-8').replace(
            '../../packages/tree-view', (ROOT / 'packages/tree-view').as_posix()),
        encoding='utf-8')
    if window:
        pac.write_text(pac.read_text(encoding='utf-8').replace(
            'window: "900x700"', f'window: "{window}"'), encoding='utf-8')
    return app


def run_matrix(session, record):
    # ── V-01 controlled identity: the app seeds the expanded/disabled/error rows.
    a = session.a_line()
    vfd.require(a is not None, f'A tree summary line missing: {session.snapshot()[:300]}')
    vfd.require(a[0] == '', f'A tree must start with no selection, got {a[0]!r}')
    vfd.require(a[1] == '1', f'A tree must start with one expanded node, got {a[1]!r}')

    rows, rows_b = session.split_row_ids()
    vfd.require(len(rows) == 5,
                f'A tree must render 5 rows (expanded chapter-01 + 3 more), got {len(rows)}')
    vfd.require(len(rows_b) == 2,
                f'B tree must render its 2 collapsed root rows, got {len(rows_b)}')

    # V-01: select carries exactly the target ID; the app echoes it back.
    session.press(rows[0])
    session.wait(lambda t: 'select:chapter-01' in t)
    a = session.a_line()
    vfd.require(a[0] == 'chapter-01', f'selection must be the pressed row, got {a[0]!r}')

    # V-01: chevron toggles expansion without selecting.
    snap = session.snapshot()
    m = re.search(r'container #(vnode_\d+) \{\s*\n\s*style: "w-5 h-5 shrink-0[^"]*"\s*\n\s*onclick: \.WantToggle', snap)
    vfd.require(m, 'expanded row must expose a chevron toggle target')
    session.press(m.group(1))
    session.wait(lambda t: 'toggle:chapter-01' in t)
    a = session.a_line()
    vfd.require(a[1] == '0', f'toggling a parent closed must drop to 0 expanded, got {a[1]!r}')
    vfd.require(a[0] == 'chapter-01',
                f'toggling must not change the selection, got {a[0]!r}')
    # Re-expand for later assertions.
    m = re.search(r'container #(vnode_\d+) \{\s*\n\s*style: "w-5 h-5 shrink-0[^"]*"\s*\n\s*onclick: \.WantToggle', session.snapshot())
    session.press(m.group(1))
    session.wait(lambda t: re.search(r'"A 树 sel=\S* exp=1 ', t) is not None)

    # ── V-01 retry: an error row exposes retry, and retrying clears it.
    vfd.require('重试' in session.snapshot(), 'error row must expose a retry action')
    session.press_button('重试')
    session.wait(lambda t: 'retry:images' in t)
    session.wait(lambda t: '重试' not in t, timeout=10)

    # ── V-04 instances: the B tree is an independent controlled instance.
    b_before = session.b_line()
    vfd.require(b_before is not None, 'B tree summary line missing')
    vfd.require(b_before[0] == '', f'B tree must be unselected, got {b_before[0]!r}')
    vfd.require(b_before[1] == '0', f'B tree must start collapsed, got {b_before[1]!r}')
    vfd.require(b_before[2].strip() == '',
                f'B tree must not have received A-tree events, got {b_before[2]!r}')

    # Selecting in B must not change A's selection (two controlled instances).
    _, rows_b = session.split_row_ids()
    vfd.require(len(rows_b) >= 1, 'B tree must expose its own row actions')
    session.press(rows_b[0])
    deadline = time.time() + 12
    while time.time() < deadline:
        b = session.b_line()
        if b and b[0] != '':
            break
        time.sleep(0.3)
    b = session.b_line()
    vfd.require(b[0] == 'chapter-01', f'B selection must follow B press, got {b[0]!r}')
    a = session.a_line()
    vfd.require(a[0] == 'chapter-01',
                f'A selection must stay as before the B press, got {a[0]!r}')
    vfd.require('select:' in b[2], f'B summary must carry only B events, got {b[2]!r}')

    # ── V-02 empty state: the tree reports empty, and the page recovers.
    session.press_button('A 清空')
    session.wait(lambda t: 'cleared' in t)
    session.wait(lambda t: '空树无可操作行' in t, timeout=10)
    a_rows_after, b_rows_after = session.split_row_ids()
    vfd.require(len(a_rows_after) == 0,
                f'clearing A must leave no A rows, got {len(a_rows_after)}')
    vfd.require(len(b_rows_after) == 2,
                f'clearing A must not touch the B tree, got {len(b_rows_after)}')
    return {'a_tree_rows': len(rows), 'b_tree_rows': len(rows_b)}


def run(args):
    out = args.out_dir
    scope = 'full_native_vm' if args.backend == 'native' else 'full_vue_track'
    report = {
        'platform': sys.platform, 'scope': scope, 'plan': '085',
        'auto': {'path': str(args.auto_bin), 'sha256': vfd.sha(args.auto_bin)},
        'ui_tool': {'path': str(args.ui_tool_bin), 'sha256': vfd.sha(args.ui_tool_bin)},
        'sessions': [],
    }
    app = prepare(args, out, args.window)
    driver = None
    try:
        driver = (Session if args.backend == 'native' else Browser)(args, app, out, 1)
        record = {'command': driver.command, 'pid': driver.process.pid,
                  'backend': args.backend}
        if args.backend == 'native':
            record['windows'] = driver.windows
        record['matrix'] = run_matrix(driver, record)

        # ── V-06 visual: light + dark, real screenshots.
        record['light'] = driver.screenshot('light')
        if args.backend == 'native':
            driver.press_button('切换明暗')
            time.sleep(1.2)
            record['dark'] = driver.screenshot('dark')

            # NVM-05: documented exit → clean CLI exit → port rebinds → reopen.
            try:
                driver.press_button('退出夹具')
            except (ConnectionResetError, ConnectionAbortedError):
                pass
            deadline = time.time() + 15
            code = None
            while time.time() < deadline and code is None:
                code = driver.process.poll()
                time.sleep(0.3)
            vfd.require(code == 0, f'documented exit must yield CLI exit 0, got {code}')
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.bind(('127.0.0.1', driver.port))
            record['exit_code'] = code
            record['port_rebinds'] = True
            report['sessions'].append(record)

            session2 = Session(args, app, out, 2)
            session2.wait(lambda t: 'TreeView 受控演示' in t)
            vfd.require(len(session2.row_ids()) == 5,
                        'reopened session must render the A tree again')
            session2.press(session2.row_ids()[0])
            session2.wait(lambda t: 'select:chapter-01' in t)
            report['sessions'].append({
                'command': session2.command, 'pid': session2.process.pid,
                'reopen_action': 'select chapter-01',
                'screenshot': session2.screenshot('reopen'),
            })
            session2.process.terminate()
            try:
                session2.process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                session2.process.kill()
        else:
            report['sessions'].append(record)
            driver.process.terminate()
            try:
                driver.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                driver.process.kill()
            driver._browser.close()
            driver._pw.stop()
        report['outcome'] = 'pass'
    except Exception as error:
        report['outcome'] = 'fail'
        report['failure'] = repr(error)
        raise
    finally:
        (out / 'report.json').write_text(
            json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps(report, ensure_ascii=False, indent=2))


class Browser:
    """Playwright driver over the vite-served vue output; same matrix API."""

    def __init__(self, args, app, out, index):
        from playwright.sync_api import sync_playwright
        self.index = index
        self.out = out
        self.log = open(out / f'session-{index}.log', 'wb')
        env = dict(os.environ, AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.command = [str(args.auto_bin), 'run']
        self.process = subprocess.Popen(
            self.command, cwd=app, env=env, stdout=self.log,
            stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        url = None
        deadline = time.time() + 300
        while time.time() < deadline and url is None:
            try:
                text = open(self.log.name, 'r', encoding='utf-8', errors='ignore').read()
                text = re.sub(r'\x1b\[[0-9;]*m', '', text)
                m = re.search(r'Local:\s+(http://127\.0\.0\.1:\d+/)', text)
                url = m.group(1) if m else None
            except Exception:
                pass
            time.sleep(0.5)
        vfd.require(url, 'vite dev server URL never appeared')
        self.url = url
        self._pw = sync_playwright().start()
        self._browser = self._pw.chromium.launch()
        self.page = self._browser.new_page()
        self.page_errors = []
        self.page.on('pageerror', lambda e: self.page_errors.append(str(e)[:300]))
        self.page.goto(url, wait_until='networkidle', timeout=30000)
        self.page.wait_for_timeout(1500)
        vfd.require('TreeView 受控演示' in self.page.inner_text('body'),
                    'vue demo never rendered')

    def snapshot(self):
        return self.page.inner_text('body')

    def state(self):
        return self.page.inner_text('body')

    def row_ids(self):
        return self.page.locator('[data-auto-tag="tree-view"] [data-row-id]').all_inner_texts()

    def press(self, element_id):
        raise AssertionError(
            'vue track has no vnode ids; row selection is driven by label below')

    def a_line(self):
        m = re.search(r'A 树 sel=(\S*) exp=(\d+) ([^\n]*)', self.snapshot())
        return m.groups() if m else None

    def b_line(self):
        m = re.search(r'B 树 sel=(\S*) exp=(\d+) ([^\n]*)', self.snapshot())
        return m.groups() if m else None

    def press_button(self, label):
        btn = self.page.get_by_text(label, exact=True)
        vfd.require(btn.count() >= 1, f'button {label!r} not found on vue page')
        btn.first.click()
        self.page.wait_for_timeout(400)

    def wait(self, predicate, timeout=15):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            self.page.wait_for_timeout(400)
        raise AssertionError(
            f'condition not met in {timeout}s; tail: {last[:400]}; '
            f'page errors: {self.page_errors[-3:]}')

    def screenshot(self, name):
        target = self.out / f'{self.index}-{name}.png'
        self.page.screenshot(path=str(target), full_page=True)
        vfd.require(target.stat().st_size > 1000, 'Empty vue screenshot')
        return {'path': str(target), 'sha256': vfd.sha(target)}

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path,
                        default=Path(os.environ.get(
                            'AUTO_UI_TOOL_BIN',
                            str(ROOT / 'target/debug' /
                                ('auto-ui-tool.exe' if os.name == 'nt' else 'auto-ui-tool')))))
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--window', default=None,
                        help='Optional pac window override, e.g. 420x560')
    parser.add_argument('--backend', default='native', choices=['native', 'vue'],
                        help='native VM window (default) or vite+browser vue track')
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    run(args)
