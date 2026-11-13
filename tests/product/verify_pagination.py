"""PLAN-086 pagination product gate — real native VM window for examples/pagination.

Drives the official PaginationControl demo through its four states (multi-page/
first-page/single-page/empty-results) with real MCP input, asserts the
displayed page against the actually rendered data rows (cross-layer oracle),
exercises the capacity switch, stale-id rejection after the group unmounts,
and records V-08 visual evidence (light/dark, narrow window) plus NVM-05
exit/reopen. A screenshot, an empty tree or a timed-out action fails the gate.
"""
import argparse, json, os, re, shutil, subprocess, sys, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd  # require/sha/owned_tree/windows_for/free_port

ROOT = Path(__file__).resolve().parents[2]


def bindable_port(preferred):
    """First ACTUALLY-bindable port at/below preferred — connect_ex misses
    Windows reserved ranges (Hyper-V/WSL) where the runtime's MCP bind fails
    with os error 10013 and silently rescans to an unpredictable port."""
    import socket
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
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port), AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        if os.name != 'nt':
            env['WINIT_UNIX_BACKEND'] = 'x11'
            env.pop('WAYLAND_DISPLAY', None)
        self.log = open(out / f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=app, env=env, stdout=self.log, stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        self.index = index
        self.windows = []
        # The runtime may silently rescan ports; trust the logged listen line.
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
                if '四状态演示' in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'pagination demo never rendered; last snapshot: {last[:400]}')
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
        with urllib.request.urlopen(req, timeout=8) as r:
            out = json.loads(r.read().decode())
        content = out.get('result', {}).get('content', [])
        return '\n'.join(c.get('text', '') for c in content if c.get('type') == 'text')

    def snapshot(self):
        return self.call('autoui_snapshot')

    def press_by_label(self, label, timeout=12):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            snap = self.snapshot()
            m = re.search(r'button #(vnode_\d+) "[^"]*' + re.escape(label) + r'[^"]*"', snap)
            if m:
                return self.call('autoui_action', element_id=m.group(1), action='press')
            last = snap
            time.sleep(0.4)
        raise AssertionError(f'button {label!r} not found; snapshot head: {last[:300]}')

    def find_button(self, label):
        snap = self.snapshot()
        m = re.search(r'button #(vnode_\d+) "[^"]*' + re.escape(label) + r'[^"]*"', snap)
        return m.group(1) if m else None

    def toggle_checkbox(self, label):
        snap = self.snapshot()
        m = re.search(r'checkbox #(vnode_\d+) "' + re.escape(label) + r'"', snap)
        vfd.require(m, f'missing checkbox {label}')
        self.call('autoui_action', element_id=m.group(1), action='toggle')

    def wait(self, predicate, timeout=15):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            time.sleep(0.4)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:400]}')

    def rows(self):
        return sorted(re.findall(r'text #vnode_\d+ "(item \d+)"', self.snapshot()), key=lambda s: int(s.split()[1]))

    def first_disabled(self):
        snap = self.snapshot()
        return bool(re.search(r'button #vnode_\d+ "[^"]*首页[^"]*" \{\s*\n\s*disabled: true', snap))

    def disabled_count(self):
        return self.snapshot().count('disabled: true')

    def pager_page(self):
        m = re.search(r'第 (\d+) / (\d+) 页', self.snapshot())
        return (int(m.group(1)), int(m.group(2))) if m else None

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
    shutil.copytree(ROOT / 'examples/pagination', app, ignore=shutil.ignore_patterns('deps', 'dist', 'gen', 'build'))
    # The isolated copy leaves the repo: point the package import and the dep
    # root at the producer checkout (same bytes, absolute closure root).
    entry = app / 'src/front/pages/demo.at'
    entry.write_text(entry.read_text(encoding='utf-8').replace('../../../../../packages/pagination', (ROOT / 'packages/pagination').as_posix()), encoding='utf-8')
    pac = app / 'pac.at'
    pac.write_text(pac.read_text(encoding='utf-8').replace('../../packages/pagination', (ROOT / 'packages/pagination').as_posix()), encoding='utf-8')
    if window:
        pac.write_text(pac.read_text(encoding='utf-8').replace('window: "900x700"', f'window: "{window}"'), encoding='utf-8')
    return app


def run_matrix(session, record):
    # First page of multi-page: real data correspondence.
    page, total = session.wait(lambda t: bool(re.search(r'第 \d+ / \d+ 页', t)), timeout=20) and session.pager_page()
    vfd.require((page, total) == (1, 11), f'initial pager must be 第 1 / 11 页, got {(page, total)}')
    rows = session.rows()
    vfd.require(len(rows) == 100 and rows[0] == 'item 1' and rows[-1] == 'item 100', f'first page rows wrong: {len(rows)} {rows[:2]}')

    # Controlled loop: next → data advances with the page.
    session.press_by_label('下一页')
    session.wait(lambda t: 'item 101' in t and 'item 200' in t)
    vfd.require(session.pager_page() == (2, 11), f'pager must show 第 2 / 11 页, got {session.pager_page()}')
    vfd.require('显示第 101 - 200 项' in session.snapshot(), 'showing text must track the slice')

    # Last page: tail reachable (1003 → 11 pages, tail 3 rows).
    session.press_by_label('末页')
    session.wait(lambda t: 'item 1003' in t)
    vfd.require(session.pager_page() == (11, 11), f'last page pager wrong: {session.pager_page()}')
    tail = session.rows()
    vfd.require(len(tail) == 3 and tail[-1] == 'item 1003', f'tail rows wrong: {tail}')

    # First-page state: first/prev disabled.
    session.press_by_label('首页')
    session.wait(lambda t: 'item 1' in t)
    vfd.require(session.first_disabled(), 'first button must be disabled on the first page')

    # Capacity switch: 10 → 25 rows/page (consumer applies, then re-derives).
    session.toggle_checkbox('容量切换')
    session.press_by_label('25')
    session.wait(lambda t: '显示第 1 - 25 项' in t and bool(re.search(r'第 1 / 41 页', t)))
    vfd.require(session.pager_page() == (1, 41), f'capacity 25 must yield 41 pages, got {session.pager_page()}')
    rows = session.rows()
    vfd.require(len(rows) == 25 and rows[0] == 'item 1' and rows[-1] == 'item 25', f'capacity 25 rows wrong: {len(rows)} {rows[:2]}…{rows[-1:]}')

    # Empty results: the whole group leaves the tree.
    session.press_by_label('总量0')
    session.wait(lambda t: not re.search(r'第 \d+ / \d+ 页', t))
    vfd.require(session.rows() == [], 'empty total must render no rows')

    # Single page hidden by default.
    session.press_by_label('总量1')
    session.wait(lambda t: 'item 1' in t and not re.search(r'第 \d+ / \d+ 页', t))

    # hideWhenSingle=false keeps a fully disabled group visible.
    session.toggle_checkbox('单页隐藏')
    session.wait(lambda t: bool(re.search(r'第 1 / 1 页', t)))
    vfd.require(session.disabled_count() >= 4, 'single-page group buttons must all be disabled')

    # Error config is visible, not masked by hiding: drive a consumer-side
    # pageSize<=0 through the capacity list is impossible (component guards),
    # so the named-error oracle is covered by the tools test; here we restore.
    session.toggle_checkbox('单页隐藏')
    session.toggle_checkbox('容量切换')
    session.press_by_label('总量1003')
    # 容量仍是消费方应用的 25（组件不持有第二份 pageSize）。
    session.wait(lambda t: bool(re.search(r'第 1 / 41 页', t)))

    # Disabled consumer flag: presses reject, page unchanged.
    stale_next = session.find_button('下一页')
    session.toggle_checkbox('禁用')
    session.wait(lambda t: session.disabled_count() >= 4)
    before = session.pager_page()
    err, _ = _press_allow_error(session, stale_next)
    vfd.require(err or session.pager_page() == before, 'disabled pager must not advance pages')
    session.toggle_checkbox('禁用')

    # V-03: stale id after the group unmounts (total → 0 hides it).
    stale = session.find_button('下一页')
    session.press_by_label('总量0')
    session.wait(lambda t: not re.search(r'第 \d+ / \d+ 页', t))
    err, _ = _press_allow_error(session, stale)
    vfd.require(err, 'stale pager id after unmount must be rejected')
    # Recover to multi-page for the visual shot (capacity stays 25 → 41 pages).
    session.press_by_label('总量1003')
    session.wait(lambda t: bool(re.search(r'第 1 / 41 页', t)))


def _press_allow_error(session, target):
    """Attempt a press and report whether it was rejected. native: MCP action
    on the element id (stale/missing → named error). vue: locator click with a
    short timeout — a removed element or a disabled button surfaces as a
    playwright timeout, which counts as rejection for this oracle."""
    if not hasattr(session, 'port'):
        try:
            label = target.split('vue:', 1)[1] if target.startswith('vue:') else target
            session.page.locator(f'button[title="{label}"]').first.click(timeout=2000)
        except Exception as e:
            return repr(e), ''
        return None, 'clicked'
    import urllib.request, json as js
    body = js.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                     'params': {'name': 'autoui_action', 'arguments': {'element_id': target, 'action': 'press'}}}).encode()
    req = urllib.request.Request(f'http://127.0.0.1:{session.port}/mcp', data=body,
                                 headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=8) as r:
            out = js.loads(r.read().decode())
        text = '\n'.join(c.get('text', '') for c in out.get('result', {}).get('content', []))
        is_err = 'error' in out.get('result', {}) or 'Error' in text or 'not found' in text
        return (text if is_err else None, text)
    except Exception as e:
        return repr(e), ''


class Browser:
    """Playwright driver over the vite-served vue output; same matrix API as
    Session so run_matrix asserts identical behavior on the vue track."""

    def __init__(self, args, app, out, index):
        from playwright.sync_api import sync_playwright
        self.index = index
        self.out = out
        self.log = open(out / f'session-{index}.log', 'wb')
        env = dict(os.environ, AUTO_UI_TOOL_BIN=str(args.ui_tool_bin), AUTO_BACKEND_IMPL='vm')
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.process = subprocess.Popen([str(args.auto_bin), 'run'], cwd=app, env=env,
                                        stdout=self.log, stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        url = None
        deadline = time.time() + 300
        while time.time() < deadline and url is None:
            try:
                text = open(self.log.name, 'r', encoding='utf-8', errors='ignore').read()
                text = re.sub(r'\[[0-9;]*m', '', text)
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
        vfd.require('四状态演示' in self.page.inner_text('body'), 'vue demo never rendered')

    def snapshot(self):
        return self.page.inner_text('body')

    def press_by_label(self, label, timeout=12):
        # Icon buttons carry the accessible name in the title attribute.
        btn = self.page.locator(f'button[title="{label}"]')
        if btn.count() == 0:
            # Exact text: substring matching would hit "item 25" for "25".
            btn = self.page.get_by_text(label, exact=True)
        vfd.require(btn.count() >= 1, f'button {label!r} not found on vue page')
        btn.first.click()
        self.page.wait_for_timeout(500)

    def find_button(self, label):
        btn = self.page.locator(f'button[title="{label}"]')
        if btn.count() == 0:
            # Exact text: substring matching would hit "item 25" for "25".
            btn = self.page.get_by_text(label, exact=True)
        return f'vue:{label}' if btn.count() else None

    _CHECKBOX_ORDER = ['单页隐藏', '容量切换', '紧凑', '禁用']

    def toggle_checkbox(self, label):
        # vue 轨 checkbox 的 text: 是选中指示器 slot 内容（shadcn 惯例），
        # 可见标签文本不在 DOM——按声明顺序定位点击（与 native 断言解耦）。
        idx = self._CHECKBOX_ORDER.index(label)
        self.page.locator('[data-auto-tag="checkbox"]').nth(idx).click()
        self.page.wait_for_timeout(500)

    def wait(self, predicate, timeout=15):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            self.page.wait_for_timeout(400)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:400]}; page errors: {self.page_errors[-3:]}')

    def rows(self):
        return sorted(re.findall(r'item \d+', self.snapshot()), key=lambda s: int(s.split()[1]))

    def first_disabled(self):
        return self.page.locator('button[title="首页"]').first.is_disabled()

    def disabled_count(self):
        return self.page.locator('button[disabled]').count()

    def pager_page(self):
        m = re.search(r'第 (\d+) / (\d+) 页', self.snapshot())
        return (int(m.group(1)), int(m.group(2))) if m else None

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


def run(args):
    out = args.out_dir
    scope = 'full_native_vm' if args.backend == 'native' else 'full_vue_track'
    report = {'platform': sys.platform, 'scope': scope, 'auto': {'path': str(args.auto_bin), 'sha256': vfd.sha(args.auto_bin)},
              'ui_tool': {'path': str(args.ui_tool_bin), 'sha256': vfd.sha(args.ui_tool_bin)}, 'sessions': []}
    app = prepare(args, out, args.window)
    driver = None
    try:
        driver = (Session if args.backend == 'native' else Browser)(args, app, out, 1)
        record = {'command': [str(args.auto_bin), 'run', '-r' if args.backend == 'native' else '', 'vm' if args.backend == 'native' else ''][0:3 if args.backend == 'native' else 2],
                  'pid': driver.process.pid, 'backend': args.backend}
        if args.backend == 'native':
            record['windows'] = driver.windows
        run_matrix(driver, record)
        # V-08 visual evidence: light + dark on the current window (dark is a
        # native-runtime convention; the vue leg records the light surface).
        record['light'] = driver.screenshot('light-multi')
        driver.press_by_label('总量1')
        driver.wait(lambda t: 'item 1' in t and not re.search(r'第 \d+ / \d+ 页', t))
        record['light-single-hidden'] = driver.screenshot('light-single-hidden')
        driver.press_by_label('总量1003')
        driver.wait(lambda t: bool(re.search(r'第 1 / 41 页', t)))
        if args.backend == 'native':
            driver.call('autoui_action', element_id=driver.find_button('切换明暗'), action='press')
            time.sleep(1.2)
            record['dark'] = driver.screenshot('dark-multi')
            # NVM-05: documented exit input → clean exit → port rebinds. The app
            # exits mid-request (Process.exit tears down the HTTP server), so the
            # final response may reset — the exit code below is the oracle.
            try:
                driver.press_by_label('退出夹具')
            except (ConnectionResetError, ConnectionAbortedError):
                pass
            deadline = time.time() + 15
            code = None
            while time.time() < deadline and code is None:
                code = driver.process.poll()
                time.sleep(0.3)
            vfd.require(code == 0, f'documented exit must yield CLI exit 0, got {code}')
            import socket as _s
            with _s.socket(_s.AF_INET, _s.SOCK_STREAM) as s:
                s.bind(('127.0.0.1', driver.port))
            record['exit_code'] = code
            record['port_rebinds'] = True
            report['sessions'].append(record)
            # Fresh session accepts a new action (reopen discipline).
            session2 = Session(args, app, out, 2)
            page, total = session2.wait(lambda t: bool(re.search(r'第 \d+ / \d+ 页', t))) and session2.pager_page()
            vfd.require((page, total) == (1, 11), f'reopened session must start at 第 1 / 11 页, got {(page, total)}')
            session2.press_by_label('下一页')
            session2.wait(lambda t: 'item 101' in t)
            report['sessions'].append({'command': session2.command, 'pid': session2.process.pid, 'reopen_action': '下一页 → item 101',
                                       'narrow_screenshot': session2.screenshot('narrow-reopen') if args.window else None})
            session2.process.terminate()
            try:
                session2.process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                session2.process.kill()
        else:
            # vue track: behavioral matrix + controlled loop on the live page
            # (matrix left capacity at the consumer-applied 25 → page 2 = 26..50).
            driver.press_by_label('下一页')
            driver.wait(lambda t: 'item 26' in t and 'item 50' in t)
            vfd.require(driver.pager_page() == (2, 41), f'vue controlled loop wrong: {driver.pager_page()}')
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
        if driver is not None:
            driver.cleanup_failure()
        raise
    finally:
        (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path, default=Path(os.environ.get('AUTO_UI_TOOL_BIN', str(ROOT / 'target/debug' / ('auto-ui-tool.exe' if os.name == 'nt' else 'auto-ui-tool')))))
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--window', default=None, help='Optional pac window override, e.g. 420x560 for the narrow visual leg')
    parser.add_argument('--backend', default='native', choices=['native', 'vue'], help='native VM window (default) or vite+browser vue track')
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    run(args)
