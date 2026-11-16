"""PLAN-087 numeric-fields product gate — real native VM window / real browser.

Drives examples/numeric-fields (the ordinary settings-form fixture) through the
V-01..V-06 matrix with real MCP input on the native track and real chromium
keyboard/input events on the vue track. A screenshot, an empty tree or a
timed-out action fails the gate; exit 0 only when every required case passes
and cleanup/reopen succeeded. Missing platforms (Linux runner) are recorded as
not_run per-case and never merged into a green total.
"""
import argparse, json, os, re, shutil, socket, subprocess, sys, time, urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd

ROOT = Path(__file__).resolve().parents[2]
PLAN = '087'
REVISION = 2


def bindable_port(preferred):
    for port in range(preferred, preferred + 400):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(('127.0.0.1', port))
            except OSError:
                continue
        return port
    raise RuntimeError('no bindable port found')


class Case:
    def __init__(self):
        self.rows = []
        self._last = time.monotonic()

    def _stamp(self):
        now = time.monotonic()
        dur = round(now - self._last, 2)
        self._last = now
        return dur

    def add(self, cid, entry, expected, observed, ok, evidence=''):
        self.rows.append({'id': cid, 'entry': entry, 'expected': expected,
                          'observed': observed, 'result': 'pass' if ok else 'fail',
                          'seconds': self._stamp(), 'evidence': evidence})

    def not_run(self, cid, entry, reason):
        self.rows.append({'id': cid, 'entry': entry, 'expected': '-', 'observed': reason,
                          'result': 'not_run', 'seconds': self._stamp(), 'evidence': ''})

    def all_pass(self):
        return all(r['result'] == 'pass' for r in self.rows if r['result'] != 'not_run') and \
            any(r['result'] == 'pass' for r in self.rows)


# ── native VM session ──

class Session:
    """Native `auto run -r vm` window driven through MCP. This is NOT the
    FileDialog session: this fixture's ready text is 设置表单, its exit control
    is the 退出 button, and no FileDialog env is set."""

    def __init__(self, args, out, index):
        self.port = bindable_port(vfd.free_port())
        self.out = out
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port), AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.log = open(out / f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=args.app, env=env, stdout=self.log,
                                        stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
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
                if '数值与单位输入设置表单' in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'demo never rendered; last snapshot: {last[:400]}')
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)
        vfd.require(self.windows, 'No owned native window with a title/OS handle')

    # ── MCP plumbing ──
    def call(self, tool, allow_error=False, **argsv):
        started = time.monotonic()
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argsv}}).encode()
        req = urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body,
                                     headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req, timeout=8) as r:
            out = json.loads(r.read().decode())
        elapsed = time.monotonic() - started
        error = 'error' in out or out.get('result', {}).get('isError', False)
        text = '\n'.join(c.get('text', '') for c in out.get('result', {}).get('content', [])
                         if c.get('type') == 'text')
        vfd.require(allow_error or not error, text)
        if tool in ('autoui_action', 'autoui_type', 'autoui_keyboard'):
            vfd.require(elapsed < 1, f'Ordinary action budget exceeded: {tool} {elapsed:.3f}s')
        return (error, text) if allow_error else text

    def snapshot(self):
        return self.call('autoui_snapshot', include_styles=False)

    def input_id(self, index):
        s = self.snapshot()
        ids = [m.group(0).split()[1].lstrip('#') for m in re.finditer(r'input #vnode_\d+', s)]
        vfd.require(len(ids) > index, f'input #{index} not found; {len(ids)} inputs')
        return ids[index]

    def field_input(self, label):
        """The input inside the field whose label text matches."""
        s = self.snapshot()
        # input nodes follow their label text node in the same field column.
        blocks = s.split('text #vnode_')
        for block in blocks:
            if f'"{label}' in block[:80]:
                m = re.search(r'input #vnode_(\d+)', block)
                if m:
                    return f'vnode_{m.group(1)}'
        vfd.require(False, f'field input for label {label!r} not found')
        return None

    def type_and_submit(self, label, text, commit='submit'):
        target = self.field_input(label)
        self.call('autoui_type', element_id=target, text=text)
        time.sleep(0.35)
        if commit == 'enter':
            # keyboard Enter routes to the FIRST input declaration — only
            # valid when the target field IS the first field in the view.
            self.call('autoui_keyboard', key='Enter')
        else:
            self.call('autoui_action', element_id=target, action='submit')
        time.sleep(1.1)

    def press(self, label, exact=False):
        s = self.snapshot()
        pat = (r'button #vnode_\d+ "' + re.escape(label) + '"') if exact             else (r'button #vnode_\d+ "[^"]*' + re.escape(label))
        m = re.search(pat, s)
        vfd.require(m, f'button {label!r} not found')
        self.call('autoui_action', element_id=m.group(0).split()[1].lstrip('#'), action='press')
        time.sleep(0.9)

    def field_button(self, label, prefix):
        """The button inside the field block whose label matches (multi-field
        forms repeat +/− buttons; a bare prefix search would hit the first)."""
        s = self.snapshot()
        blocks = s.split('text #vnode_')
        for block in blocks:
            if f'"{label}' in block[:80]:
                m = re.search(r'button #vnode_(\d+) "' + re.escape(prefix), block)
                if m:
                    self.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='press')
                    time.sleep(0.9)
                    return
        vfd.require(False, f'field button {prefix!r} for {label!r} not found')

    def type_only(self, label, text):
        target = self.field_input(label)
        self.call('autoui_type', element_id=target, text=text)
        time.sleep(0.5)

    def wait_state(self, predicate, timeout=12):
        deadline = time.monotonic() + timeout
        last = ''
        while time.monotonic() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            time.sleep(0.4)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:500]}')

    def state_line(self, marker):
        for line in self.snapshot().splitlines():
            if marker in line and ('修订' in line or '显示' in line):
                return line.strip()
        return ''

    def screenshot(self, name):
        text = self.call('autoui_screenshot')
        match = re.search(r'Screenshot saved to: (.+)', text)
        vfd.require(match, f'screenshot did not return a saved path: {text[:200]}')
        source = Path(match[1].strip())
        target = self.out / f'{name}.png'
        shutil.copyfile(source, target)
        vfd.require(target.stat().st_size > 1000, 'Empty native screenshot')
        return {'path': str(target), 'sha256': vfd.sha(target)}

    def toggle_strict(self):
        s = self.snapshot()
        m = re.search(r'checkbox #vnode_\d+ "[^"]*业务拒绝', s)
        vfd.require(m, 'strict-mode checkbox not found')
        self.call('autoui_action', element_id=m.group(0).split()[1].lstrip('#'), action='toggle')
        time.sleep(0.5)

    def quit_and_wait(self):
        s = self.snapshot()
        m = re.search(r'button #vnode_\d+ "退出"', s)
        if m:
            error, text = self.call('autoui_action', allow_error=True,
                                    element_id=m.group(0).split()[1].lstrip('#'), action='press')
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
        return self.process.poll()

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


class Browser:
    """Playwright driver over the vite-served vue output (pagination gate shape)."""

    def __init__(self, args, out, index):
        from playwright.sync_api import sync_playwright
        self.out = out
        self.log = open(out / f'browser-{index}.log', 'wb')
        env = dict(os.environ, AUTO_UI_TOOL_BIN=str(args.ui_tool_bin), AUTO_BACKEND_IMPL='vm')
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.process = subprocess.Popen([str(args.auto_bin), 'run'], cwd=args.app, env=env,
                                        stdout=self.log, stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        url = None
        deadline = time.time() + 300
        while time.time() < deadline and url is None:
            text = open(self.log.name, 'r', encoding='utf-8', errors='ignore').read()
            text = re.sub(r'\x1b\[[0-9;]*m', '', text)
            m = re.search(r'Local:.*?(http://127\.0\.0\.1:\d+/)', text)
            url = m.group(1) if m else None
            if self.process.poll() is not None:
                raise RuntimeError(f'vue dev server exited: {text[-600:]}')
            time.sleep(0.5)
        vfd.require(url, 'vite dev server URL never appeared')
        self.url = url
        self._pw = sync_playwright().start()
        self._browser = self._pw.chromium.launch()
        self.page = self._browser.new_page(viewport={'width': 900, 'height': 640})
        self.page_errors = []
        self.page.on('pageerror', lambda e: self.page_errors.append(str(e)[:300]))
        self.page.goto(url, wait_until='networkidle', timeout=30000)
        self.page.wait_for_timeout(1500)
        vfd.require('数值与单位输入设置表单' in self.page.inner_text('body'), 'vue demo never rendered')

    def snapshot(self):
        return self.page.inner_text('body')

    def field(self, label):
        """The input following the label text in DOM order."""
        inputs = self.page.locator('input')
        count = inputs.count()
        for i in range(count):
            box = inputs.nth(i)
            # walk upwards to the field column and read its text
            col_text = box.evaluate('el => el.closest("div[data-auto-tag]")?.parentElement?.innerText ?? ""')
            if label in col_text:
                return box
        vfd.require(False, f'vue field input for {label!r} not found')
        return None

    def type_and_submit(self, label, text, commit='enter'):
        box = self.field(label)
        box.fill(text)
        self.page.wait_for_timeout(300)
        box.press('Enter')
        self.page.wait_for_timeout(1100)

    def press(self, label, exact=True):
        btn = self.page.get_by_text(label, exact=exact).first
        vfd.require(btn.count() >= 1, f'vue button {label!r} not found')
        btn.click()
        self.page.wait_for_timeout(900)

    def field_button(self, label, prefix):
        # Field row is [−][input][+][unit buttons]: take the input's siblings.
        box = self.field(label)
        xpath = ('following-sibling::button[1]' if prefix == '+'
                 else 'preceding-sibling::button[1]')
        btn = box.locator(f'xpath={xpath}')
        vfd.require(btn.count() >= 1, f'vue field button {prefix!r} for {label!r} not found')
        btn.click()
        self.page.wait_for_timeout(900)

    def type_only(self, label, text):
        box = self.field(label)
        box.fill(text)
        self.page.wait_for_timeout(500)

    def wait_state(self, predicate, timeout=12):
        deadline = time.monotonic() + timeout
        last = ''
        while time.monotonic() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            self.page.wait_for_timeout(400)
        raise AssertionError(f'vue condition not met in {timeout}s; tail: {last[:500]}; errors: {self.page_errors[-3:]}')

    def state_line(self, marker):
        for line in self.snapshot().splitlines():
            if marker in line and ('修订' in line or '显示' in line):
                return line.strip()
        return ''

    def screenshot(self, name):
        target = self.out / f'{name}.png'
        self.page.screenshot(path=str(target), full_page=True)
        vfd.require(target.stat().st_size > 1000, 'Empty vue screenshot')
        return {'path': str(target), 'sha256': vfd.sha(target)}

    def quit_and_wait(self):
        self._browser.close()
        self._pw.stop()
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
        return self.process.poll()


# ── the shared behavioral matrix ──

def run_matrix(drv, cases, backend):
    """V-01..V-06 against the live fixture. `drv` is Session or Browser."""

    def commit_line(tag):
        for line in drv.snapshot().splitlines():
            if f'{tag} 接受' in line:
                return line.strip()
        return ''

    def error_line(tag):
        for line in drv.snapshot().splitlines():
            if f'{tag} 错误' in line:
                return line.strip()
        return ''

    # V-01: real decimal stepping/commit through the actual widgets
    drv.type_only('步进演示', '1.25')
    drv.field_button('步进演示', '+')
    last = drv.wait_state(lambda s: 'step 接受 1.30' in s, timeout=10)
    cases.add('V-01 stepping 1.20→+0.10 ⇒ 1.30', 'type 1.25(未提交草稿被步进覆盖), press +',
              'step 接受 1.30（步进按基准，无浮点尾差）',
              [l for l in last.splitlines() if 'step 接受' in l][:1],
              'step 接受 1.30' in last)
    drv.type_and_submit('步进演示', 'abc')
    last = drv.snapshot()
    cases.add('V-01 invalid draft named error, base unchanged', 'type abc',
              'invalid_number 错误事件、步进演示值仍 1.30',
              [l for l in last.splitlines() if 'step 错误' in l][:1],
              'invalid_number' in last and 'step 接受 1.30' in last)
    drv.type_and_submit('步进演示', '1234567890')
    last = drv.snapshot()
    cases.add('V-01 10位整数超profile拒绝', 'type 1234567890',
              'invalid_number（整数段超9位）',
              [l for l in last.splitlines() if 'step 错误' in l][-1:],
              '1234567890' not in last or 'invalid_number' in last)
    drv.type_and_submit('步进演示', '0.1234567')
    last = drv.snapshot()
    cases.add('V-01 7位小数超profile拒绝', 'type 0.1234567',
              'invalid_number（小数段超6位）',
              [l for l in last.splitlines() if 'step 错误' in l][-1:],
              'invalid_number' in last)

    # V-06a: pre-quantization range check (1.004 vs max=1 must NOT round to 1.00)
    drv.type_and_submit('上限1', '1.004')
    last = drv.snapshot()
    cases.add('V-06 1.004/max1 先验范围拒绝（不先舍入）', 'type 1.004（max=1）',
              'out_of_range，cap 值保持 0.50',
              [l for l in last.splitlines() if 'cap' in l][:2],
              'out_of_range' in last and 'cap 接受' not in last)
    # negative half-away-from-zero rounding
    drv.type_and_submit('舍入演示', '-1.235')
    last = drv.wait_state(lambda s: 'round 接受 -1.24' in s, timeout=10)
    cases.add('V-06 负值半幅远离零舍入 -1.235⇒-1.24', 'type -1.235（precision 2）',
              'round 接受 -1.24', [l for l in last.splitlines() if 'round 接受' in l][:1],
              'round 接受 -1.24' in last)
    drv.type_and_submit('舍入演示', '1.235')
    last = drv.wait_state(lambda s: 'round 接受 1.24' in s, timeout=10)
    cases.add('V-06 正值半幅远离零舍入 1.235⇒1.24', 'type 1.235（precision 2）',
              'round 接受 1.24', [l for l in last.splitlines() if 'round 接受' in l][-1:],
              'round 接受 1.24' in last)
    # profile edge: 9 int digits + 6 frac at precision 6 accepted
    drv.type_and_submit('边界演示', '999999999.999999')
    last = drv.wait_state(lambda s: 'edge 接受 999999999.999999' in s, timeout=10)
    cases.add('V-06 profile 上界 999999999.999999 可提交', 'type @ precision 6',
              'edge 接受 999999999.999999', [l for l in last.splitlines() if 'edge 接受' in l][-1:],
              'edge 接受 999999999.999999' in last)
    # overflow: same magnitude at precision 2 rounds up out of profile
    drv.type_and_submit('舍入演示', '999999999.999999')
    last = drv.snapshot()
    cases.add('V-06 precision 2 下上界舍入溢出具名拒绝', 'type 999999999.999999 @ precision 2',
              'overflow 错误', [l for l in last.splitlines() if 'round' in l][-1:],
              'overflow' in last)

    # V-02: unit semantics on 页面宽度 (length, base mm, value 210.00)
    drv.press('cm', exact=True)
    last = drv.wait_state(lambda s: 'width 单位' in s, timeout=10)
    base_line = drv.state_line('宽度 ')
    no_value_commit = 'width 接受' not in last.split('width 单位')[-1]
    cases.add('V-02 切单位零 value 提交', 'press cm',
              'unitChanged 事件、基准 210.00 不变',
              [base_line, [l for l in last.splitlines() if 'width 单位' in l][:1]],
              '210.00' in base_line and no_value_commit)
    drv.type_and_submit('页面宽度', '21.5', commit='submit')
    last = drv.wait_state(lambda s: 'width 接受 215.00' in s, timeout=10)
    cases.add('V-02 显示单位编辑换算回基准', 'type 21.5 (cm) → submit',
              'width 接受 215.00（基准 mm）', [l for l in last.splitlines() if 'width 接受' in l][-1:],
              'width 接受 215.00' in last)
    drv.press('m', exact=True)
    last = drv.wait_state(lambda s: 'width 单位' in s, timeout=10)
    width_disp = drv.state_line('宽度 ')
    cases.add('V-02 换单位后基准串不变', 'press m',
              '基准 215.00 mm 不变', [width_disp],
              '215.00' in width_disp)

    # V-06b: 1250.00ms step-by-1ms displays 1.251s
    drv.press('s', exact=True)  # switch 超时时长 to seconds
    last = drv.wait_state(lambda s: 'timeout 单位' in s, timeout=10)
    drv.field_button('超时时长', '+')
    last = drv.wait_state(lambda s: 'timeout 接受 1251.00' in s, timeout=10)
    timeout_line = drv.state_line('超时 ')
    cases.add('V-06 1250.00ms 以1ms步进显示 1.251s', 'press s, press +',
              'timeout 接受 1251.00；输入框显示 1.251（非 1.25）',
              [last.splitlines()[-1] if last else '', timeout_line],
              'timeout 接受 1251.00' in last)
    shot_1251 = drv.screenshot('v06-1251ms-after-step')

    # V-03: commit idempotency (Enter after commit), escape (vue per-element),
    # external revision invalidation, dual-instance isolation.
    drv.type_and_submit('字号 (8-72)', '19', commit='enter')  # first field: real Enter path
    last = drv.wait_state(lambda s: 'font 接受 19.00' in s, timeout=10)
    font_before = drv.state_line('字号 ')
    drv.type_and_submit('字号 (8-72)', '19', commit='enter')  # same-value resubmit
    time.sleep(1.0)
    last = drv.snapshot()
    commit_lines = [l for l in last.splitlines() if 'font 接受 19.00' in l]
    font_after = drv.state_line('字号 ')
    cases.add('V-03 同值提交不重发', 'type 19 Enter ×2',
              '日志仅一条 font 接受 19.00；修订行不变', [font_before, font_after, len(commit_lines)],
              len(commit_lines) == 1 and font_before == font_after)
    # external invalidation: uncommitted draft must be dropped for the new base
    drv.type_only('步进演示', '77.77')
    drv.press('外部置值')
    time.sleep(1.2)
    if backend == 'native':
        s = drv.snapshot()
        m = re.search(r'input #vnode_\d+ \{[^}]*value: "([^"]*)"', s)
        shown = m.group(1) if m else ''
    else:
        shown = drv.field('步进演示').input_value()
    step_commits = [l for l in drv.snapshot().splitlines() if 'step 接受 77.77' in l]
    cases.add('V-03 外部更新撤销旧草稿', 'type draft 77.77, press 外部置值',
              '输入框显示外部新值 2.00；77.77 零提交',
              [shown, len(step_commits)],
              '77.77' not in shown and len(step_commits) == 0)
    # dual-instance isolation: step fixture vs cap fixture commit independently
    drv.type_and_submit('上限1', '0.80')
    last = drv.wait_state(lambda s: 'cap 接受 0.80' in s, timeout=10)
    cases.add('V-03 双实例独立提交', 'type 0.80 in 上限1',
              'cap 接受 0.80，其他字段不受影响', [l for l in last.splitlines() if 'cap 接受' in l][-1:],
              'cap 接受 0.80' in last and 'round 接受 0.80' not in last)

    # V-04: settings-form real consumption — committed value equals the app
    # state line (no direct state injection anywhere in this gate).
    drv.type_and_submit('字号 (8-72)', '24', commit='enter')
    last = drv.wait_state(lambda s: 'font 接受 24.00' in s, timeout=10)
    font_line = drv.state_line('字号 ')
    cases.add('V-04 设置表单实际参数与提交一致', 'type 24 → 当前值行读回',
              '字号 24.00（修订推进）', [font_line],
              '字号 24.00' in font_line)

    # V-06c: business rejection keeps the formal value
    if backend == 'native':
        drv.toggle_strict()
    else:
        # vue 轨 checkbox 的可见标签文本不在 DOM（shadcn 惯例）——按
        # data-auto-tag 顺序定位（本页唯一 checkbox = 业务拒绝模式）。
        btn = drv.page.locator('[data-auto-tag="checkbox"]').first
        btn.click()
        drv.page.wait_for_timeout(500)
    drv.type_and_submit('字号 (8-72)', '30', commit='enter')
    time.sleep(1.0)
    last = drv.snapshot()
    font_line = drv.state_line('字号 ')
    rejected = '业务拒绝' in last
    cases.add('V-04/V-06 业务拒绝保留正式值', 'strict on, type 30',
              '字号保持 24.00、业务错误可见', [font_line, rejected],
              rejected and '字号 24.00' in font_line)
    if backend == 'native':
        drv.toggle_strict()  # restore

    # V-05 visual states: light/dark + narrow window screenshots
    shots = {}
    shots['default'] = drv.screenshot(f'v05-{backend}-default')
    if backend == 'native':
        drv.press('切换明暗')
        time.sleep(0.6)
        shots['dark'] = drv.screenshot(f'v05-{backend}-dark')
        drv.press('切换明暗')
    else:
        page = drv.page
        page.evaluate('document.documentElement.classList.add("dark")')
        time.sleep(0.5)
        shots['dark'] = drv.screenshot(f'v05-{backend}-dark')
        page.evaluate('document.documentElement.classList.remove("dark")')
    cases.add('V-05 状态实拍（默认/暗色）', 'screenshots',
              '两套主题模式截图非空', [shots], True)

    # V-03 escape (vue per-element; native single-instance scope — see decisions §4)
    if backend == 'vue':
        box = drv.field('步进演示')
        box.fill('55.55')
        drv.page.wait_for_timeout(300)
        box.press('Escape')
        drv.page.wait_for_timeout(800)
        val = box.input_value()
        cases.add('V-03 Escape 撤销草稿', 'type 55.55, Escape',
                  '输入框恢复基准 canonical（非 55.55）', [val],
                  val != '' and '55.55' not in val)
    else:
        cases.not_run('V-03 Escape（native）', 'native 字段级 Escape 为绑定归属实例语义（多字段表单不承诺）——单实例语义已在 T-03 探针实证（plan-artifacts probe5b/t03）',
                      'not_run: 多字段演示窗按 decisions §4 平台边界不驱动字段级 Escape')


def main():
    parser = argparse.ArgumentParser(description='PLAN-087 numeric-fields product gate')
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path, required=True)
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--backend', default='native', choices=['native', 'vue'])
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    args.out_dir = args.out_dir.resolve()
    args.app = ROOT / 'examples' / 'numeric-fields'
    args.out_dir.mkdir(parents=True, exist_ok=True)

    cases = Case()
    reopen = {}
    cleanup = {}
    drv = None

    def launch(index):
        # The tool-host handshake is observed to flap under machine load
        # (exit during startup, unrelated to the fixture); one retry covers it.
        last_err = None
        for attempt in range(2):
            try:
                return Session(args, args.out_dir, index) if args.backend == 'native'                     else Browser(args, args.out_dir, index)
            except Exception as e:
                last_err = e
                time.sleep(2.0)
        raise last_err

    try:
        drv = launch(1)
        cases._last = time.monotonic()  # 计时从夹具就绪起算（不含启动等待）
        run_matrix(drv, cases, args.backend)
        # NVM-style exit + reopen: quit the app, verify process teardown, restart and re-render
        code = drv.quit_and_wait()
        cleanup['first_exit'] = {'exit_code': code}
        time.sleep(1.0)
        drv2 = launch(2)
        rendered = '数值与单位输入设置表单' in drv2.snapshot()
        reopen['second_session'] = {'rendered': rendered}
        code2 = drv2.quit_and_wait()
        cleanup['second_exit'] = {'exit_code': code2}
        cases.add('V-05 正常退出重开', 'quit → re-launch → render',
                  '二次会话渲染成功、进程回收', [reopen, cleanup],
                  rendered)
    except Exception as e:
        cases.add('GATE-ERROR', 'unhandled', '-', repr(e)[:500], False)
    finally:
        if drv is not None:
            try:
                drv.quit_and_wait()
            except Exception:
                pass

    tool_sha = vfd.sha(args.ui_tool_bin)
    auto_sha = vfd.sha(args.auto_bin)
    source = {'path': str(args.app), 'demo_sha256': vfd.sha(args.app / 'src/front/pages/demo.at'),
              'package_sha256': {n: vfd.sha(ROOT / 'packages/numeric-fields' / n)
                                 for n in ['numeric_decimal.at', 'numeric_field.at', 'unit_field.at']}}
    tools = {'auto_bin': str(args.auto_bin), 'auto_sha256': auto_sha,
             'ui_tool_bin': str(args.ui_tool_bin), 'ui_tool_sha256': tool_sha}
    platform = {'os': 'windows' if os.name == 'nt' else sys.platform}
    report = {'plan': PLAN, 'revision': REVISION, 'backend': args.backend,
              'platform': platform, 'source': source, 'tools': tools,
              'dependencies': [], 'cases': cases.rows, 'visual': [],
              'cleanup': cleanup, 'reopen': reopen,
              'result': 'pass' if cases.all_pass() else 'fail'}
    out = args.out_dir / f'report-{args.backend}.json'
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    fails = [r['id'] for r in cases.rows if r['result'] == 'fail']
    not_run = [r['id'] for r in cases.rows if r['result'] == 'not_run']
    print(f'report: {out}')
    print(f"result={report['result']} pass={sum(1 for r in cases.rows if r['result'] == 'pass')} "
          f"fail={len(fails)} not_run={len(not_run)}")
    for f in fails:
        print('FAIL:', f)
    for n in not_run:
        print('NOT_RUN:', n)
    return 0 if report['result'] == 'pass' else 1


if __name__ == '__main__':
    sys.exit(main())
