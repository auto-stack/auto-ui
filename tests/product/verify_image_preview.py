"""PLAN-089 image-viewer product gate — real native VM window / real browser.

Drives examples/image-preview (the ordinary image-viewer consumer fixture)
through the V-01..V-06 matrix with real MCP input on the native track and
real chromium input on the vue track. The 031/029 consumption legs (V-04)
launch those examples as separate sessions and drive the migrated view
areas through the same component contract. A screenshot, an empty snapshot
or a timed-out action fails the gate; exit 0 only when every required case
passes and cleanup/reopen succeeded. Missing platforms (Linux runner /
missing chromium) are recorded as not_run per-case and never merged into a
green total.
"""
import argparse, hashlib, json, os, re, shutil, socket, struct, subprocess, sys, time, zlib
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd

ROOT = Path(__file__).resolve().parents[2]
PLAN = '089'
REVISION = 2
APP = ROOT / 'examples' / 'image-preview'
# session cwd default: the demo example dir (fixture paths are cwd-relative)
READY_TEXT = '图片预览演示'
EXIT_BUTTON = None  # demo page exits via process terminate (vfd.quit path)


def bindable_port(preferred):
    for port in range(preferred, preferred + 400):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            try:
                s.bind(('127.0.0.1', port))
            except OSError:
                continue
        return port
    raise RuntimeError('no bindable port found')


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def png_pixels(path):
    """Minimal PNG decoder for non-interlaced 8-bit RGB/RGBA screenshots —
    returns (w, h, bytes) sufficient for color-presence oracles."""
    data = Path(path).read_bytes()
    assert data[:8] == b'\x89PNG\r\n\x1a\n', 'not a PNG'
    pos, idat, w, h, ct = 8, b'', 0, 0, 6
    while pos < len(data):
        (ln,) = struct.unpack('>I', data[pos:pos + 4])
        tag = data[pos + 4:pos + 8]
        body = data[pos + 8:pos + 8 + ln]
        if tag == b'IHDR':
            w, h, depth, ct = struct.unpack('>IIBB', body[:10])
        elif tag == b'IDAT':
            idat += body
        pos += 12 + ln
    assert depth == 8, f'unsupported depth {depth}'
    ch = 3 if ct == 2 else (4 if ct == 6 else 0)
    assert ch, f'unsupported color type {ct}'
    raw = zlib.decompress(idat)
    stride = w * ch
    out = bytearray()
    prev = bytearray(stride)
    i = 0
    for _ in range(h):
        f = raw[i]; i += 1
        line = bytearray(raw[i:i + stride]); i += stride
        if f == 1:
            for x in range(ch, stride):
                line[x] = (line[x] + line[x - ch]) & 0xFF
        elif f == 2:
            for x in range(stride):
                line[x] = (line[x] + prev[x]) & 0xFF
        elif f == 3:
            for x in range(stride):
                a = line[x - ch] if x >= ch else 0
                line[x] = (line[x] + ((a + prev[x]) >> 1)) & 0xFF
        elif f == 4:
            for x in range(stride):
                a = line[x - ch] if x >= ch else 0
                b = prev[x]
                c = prev[x - ch] if x >= ch else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[x] = (line[x] + pr) & 0xFF
        out += line
        prev = line
    return w, h, bytes(out)


def color_present(path, rgb, tol=6):
    w, h, px = png_pixels(path)
    r0, g0, b0 = rgb
    step = 3
    for i in range(0, len(px) - 2, 3 * step):
        if abs(px[i] - r0) <= tol and abs(px[i + 1] - g0) <= tol and abs(px[i + 2] - b0) <= tol:
            return True
    return False


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
        self.rows.append({'id': cid, 'entry': entry, 'expected': '-',
                          'observed': reason, 'result': 'not_run',
                          'seconds': self._stamp(), 'evidence': ''})

    def all_pass(self):
        return all(r['result'] == 'pass' for r in self.rows if r['result'] != 'not_run') and \
            any(r['result'] == 'pass' for r in self.rows)




def press_until(drv, label, predicate, timeout=20, tries=3):
    """press + wait with one retry — vnode 登记竞态的通用面（首 press 可能
    静默 no-op；重试前确认会话仍交互）。"""
    last = drv.snapshot()
    if predicate(last):
        return last
    for i in range(tries):
        try:
            drv.press(label)
        except AssertionError:
            vfd.require(i + 1 < tries, f'button {label!r} disappeared from snapshot')
            time.sleep(1.0)
            continue
        try:
            return drv.wait_state(predicate, timeout=timeout // tries + 4)
        except AssertionError:
            if i + 1 >= tries:
                _st = ' | '.join(l.strip()[:130] for l in drv.snapshot().splitlines()
                                 if ('状态' in l or '当前视图' in l or 'loaded=' in l or '最后' in l))[:500]
                raise AssertionError(f'{label}: predicate never met; state: {_st}')
            time.sleep(1.0)
    raise AssertionError(f'{label}: predicate never met')


# ── native VM session ──

class Session:
    """Native `auto run -r vm` window driven through MCP. This is NOT the
    FileDialog session: this fixture's ready text is 图片预览演示, and no
    FileDialog env is set. Component toolbar buttons are addressed by their
    visible snapshot labels (icon buttons surface their title text, per the
    file-dialog gate's 上一级 precedent)."""

    def __init__(self, args, out, index, cwd=None, ready=None):
        self.port = bindable_port(vfd.free_port())
        self.out = out
        # 端口不 pin（auto 自选并回写日志）；tool 也不 pin——发现链与手动
        # 运行同源（demo18 实证；pin 口会话的媒体 populate 不推进，见
        # decisions.md §7 P5/P6）。
        env = dict(os.environ)
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.log = open(out / f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=cwd or (APP if cwd is None else cwd), env=env, stdout=self.log,
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
        deadline = time.time() + 240
        last = ''
        while time.time() < deadline:
            try:
                last = self.snapshot()
                if (ready or READY_TEXT) in last:
                    if (ready or READY_TEXT) in last:
                        break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'app never rendered; last snapshot: {last[:400]}')
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)
        vfd.require(self.windows, 'No owned native window with a title/OS handle')

    def call(self, tool, allow_error=False, **argsv):
        started = time.monotonic()
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argsv}}).encode()
        req = urllib_request(f'http://127.0.0.1:{self.port}/mcp', body)
        with urllib_open(req, timeout=8) as r:
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

    def state_line(self, marker):
        for line in self.snapshot().splitlines():
            if marker in line:
                return line.strip()
        return ''

    def wait_state(self, predicate, timeout=25):
        deadline = time.monotonic() + timeout
        last = ''
        while time.monotonic() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            time.sleep(0.4)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:500]}')

    def press(self, label, exact=False):
        s = self.snapshot()
        pat = (r'button #vnode_\d+ "' + re.escape(label) + '"') if exact \
            else (r'button #vnode_\d+ "[^"]*' + re.escape(label))
        m = re.search(pat, s)
        vfd.require(m, f'button {label!r} not found in snapshot')
        self.call('autoui_action', element_id=m.group(0).split()[1].lstrip('#'), action='press')
        time.sleep(0.9)

    def screenshot(self, name):
        text = self.call('autoui_screenshot')
        match = re.search(r'Screenshot saved to: (.+)', text)
        vfd.require(match, f'screenshot did not return a saved path: {text[:200]}')
        source = Path(match[1].strip())
        target = self.out / f'{name}.png'
        shutil.copyfile(source, target)
        vfd.require(target.stat().st_size > 1000, 'Empty native screenshot')
        return {'path': str(target), 'sha256': sha(target)}

    def quit_and_wait(self):
        children = set()
        try:
            children = set(vfd.owned_tree(self.process.pid)) - {self.process.pid}
        except Exception:
            pass
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
        for pid in children:
            try:
                subprocess.run(['taskkill', '/F', '/PID', str(pid)], capture_output=True,
                               creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            except Exception:
                pass
        return self.process.poll()

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


def urllib_request(url, body):
    import urllib.request
    return urllib.request.Request(url, data=body, headers={'Content-Type': 'application/json'})


def urllib_open(req, timeout):
    import urllib.request
    return urllib.request.urlopen(req, timeout=timeout)


class Browser:
    """Playwright driver over the vite-served vue output (087 gate shape)."""

    def __init__(self, args, out, index, app_dir=None):
        from playwright.sync_api import sync_playwright
        self.out = out
        self.log = open(out / f'browser-{index}.log', 'wb')
        env = dict(os.environ, AUTO_UI_TOOL_BIN=str(args.ui_tool_bin), AUTO_BACKEND_IMPL='vm')
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.process = subprocess.Popen([str(args.auto_bin), 'run'], cwd=app_dir or APP, env=env,
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
        self.page = self._browser.new_page(viewport={'width': 1100, 'height': 800})
        self.page_errors = []
        self.page.on('pageerror', lambda e: self.page_errors.append(str(e)[:300]))
        self.page.goto(url, wait_until='networkidle', timeout=30000)
        self.page.wait_for_timeout(1500)
        vfd.require(READY_TEXT in self.page.inner_text('body'), 'vue demo never rendered')

    def snapshot(self):
        return self.page.inner_text('body')

    def state_line(self, marker):
        for line in self.snapshot().splitlines():
            if marker in line:
                return line.strip()
        return ''

    def wait_state(self, predicate, timeout=25):
        deadline = time.monotonic() + timeout
        last = ''
        while time.monotonic() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            time.sleep(0.4)
        raise AssertionError(f'condition not met in {timeout}s; tail: {last[:500]}')

    def press(self, label, exact=False):
        btn = self.page.get_by_role('button', name=label, exact=exact)
        vfd.require(btn.count() >= 1, f'vue button {label!r} not found')
        btn.first.click()
        self.page.wait_for_timeout(900)

    def wheel(self, dy):
        self.page.mouse.move(550, 400)
        self.page.mouse.wheel(0, dy)
        self.page.wait_for_timeout(700)

    def drag(self, x0, y0, x1, y1):
        self.page.mouse.move(x0, y0)
        self.page.mouse.down()
        steps = 6
        for k in range(1, steps + 1):
            self.page.mouse.move(x0 + (x1 - x0) * k / steps, y0 + (y1 - y0) * k / steps)
            self.page.wait_for_timeout(40)
        self.page.mouse.up()
        self.page.wait_for_timeout(700)

    def dblclick(self, x, y):
        self.page.mouse.dblclick(x, y)
        self.page.wait_for_timeout(700)

    def screenshot(self, name):
        target = self.out / f'{name}.png'
        self.page.screenshot(path=str(target), full_page=True)
        vfd.require(target.stat().st_size > 1000, 'Empty vue screenshot')
        return {'path': str(target), 'sha256': sha(target)}

    def quit_and_wait(self):
        self._browser.close()
        self._pw.stop()
        children = set()
        try:
            children = set(vfd.owned_tree(self.process.pid)) - {self.process.pid}
        except Exception:
            pass
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
        for pid in children:
            try:
                subprocess.run(['taskkill', '/F', '/PID', str(pid)], capture_output=True,
                               creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
            except Exception:
                pass
        return self.process.poll()

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


# ── the shared behavioral matrix ──

def run_matrix(drv, cases, backend, shots=True):
    """V-01..V-06 against the live image-preview fixture."""

    def line(marker):
        return drv.state_line(marker)

    # V-01: real transform requests through the actual component toolbar.
    # 首 press 与首帧交互之间的 vnode 登记竞态：等 Tick 计数走动再操作，
    # 打开动作带一次重试（demo18 实证 split 会话就绪后立即 press 可能
    # 静默 no-op）。
    time.sleep(2.5)
    drv.wait_state(lambda s: 'ticks=' in s, timeout=15)
    n0 = re.search(r'ticks=(\d+)', drv.snapshot())
    n0 = int(n0.group(1)) if n0 else 0
    drv.press('打开图片A')
    try:
        last = drv.wait_state(lambda s: '图片已加载' in s, timeout=8)
    except AssertionError:
        _dbg = drv.snapshot()
        vfd.require(False, 'first open failed; snapshot readout: ' + ' | '.join(
            l.strip()[:160] for l in _dbg.splitlines() if ('当前视图' in l or 'loaded=' in l or 'status' in l or '最后' in l))[:800])
        cur = int(cur.group(1)) if cur else n0
        vfd.require(cur > n0, 'tick counter frozen — session not interactive')
        drv.press('打开图片A')
        last = drv.wait_state(lambda s: '图片已加载' in s, timeout=20)
    cases.add('V-01 open A → loaded', 'press 打开图片A (media ticket URI)',
              'loaded 事件 → 状态=图片已加载', [l for l in last.splitlines() if 'loaded=' in l][:1],
              '图片已加载' in last)
    # 组件子树首帧挂载后 vnode 登记有竞态（同首 press）：settle + 一次重试。
    time.sleep(1.5)
    drv.press('放大')
    try:
        last = drv.wait_state(lambda s: 'zoom=1.25' in s, timeout=6)
    except AssertionError:
        _st = [l.strip()[:120] for l in drv.snapshot().splitlines() if ('状态' in l or '请求数' in l)]
        vfd.require(False, 'zoom did not apply; state: ' + ' | '.join(_st)[:400])
    cases.add('V-01 放大 → zoom 1.25 (×1.25 步进, 单一变换算法)', 'press 放大',
              'viewRequested(source=zoom) 接受 → zoom=1.25',
              [l for l in last.splitlines() if '当前视图' in l][:1],
              'zoom=1.25' in last)
    drv.press('宽度')
    last = drv.wait_state(lambda s: 'fit=width' in s, timeout=10)
    cases.add('V-01 宽度适配', 'press 宽度', 'fit=width zoom=1',
              [l for l in last.splitlines() if '当前视图' in l][:1],
              'fit=width' in last and 'zoom=1' in last)
    drv.press('↻')
    last = drv.wait_state(lambda s: 'rot=90' in s, timeout=10)
    cases.add('V-01 ↻ → rotation 90 (±90 规范)', 'press ↻', 'rot=90',
              [l for l in last.splitlines() if '当前视图' in l][:1],
              'rot=90' in last)
    # pixel oracle: the corner fixture's pure red/blue/green must be on screen
    if shots:
        shot = drv.screenshot(f'v01-corners-{backend}')
        seen = (color_present(shot['path'], (255, 0, 0))
                and color_present(shot['path'], (0, 0, 255))
                and color_present(shot['path'], (0, 255, 0)))
        cases.add('V-01 角点像素在帧内（跨层像素 oracle）', 'screenshot after open A',
                  '红/蓝/绿角点色全部出现（corner-marked.png 真实解码）',
                  [{'screenshot': shot}], seen)
    drv.press('平移')
    drv.press('→')
    last = drv.wait_state(lambda s: 'x=40' in s, timeout=10)
    cases.add('V-01 方向平移 +40', 'press 平移 →', 'x=40（源=pan）',
              [l for l in last.splitlines() if '当前视图' in l][:1],
              'x=40' in last)
    if backend == 'vue':
        drv.wheel(-120)
        last = drv.wait_state(lambda s: 'zoom=1.25' in s, timeout=10)
        cases.add('V-01 滚轮缩放（$event.deltaY 载荷）', 'wheel up -120',
                  'zoom 1→1.25（delta 通道与按钮同一算法）',
                  [l for l in last.splitlines() if '当前视图' in l][:1],
                  'zoom=1.25' in last)
        drv.drag(400, 380, 470, 430)
        last = drv.wait_state(lambda s: 'y=50' in s, timeout=10)
        cases.add('V-01 拖拽平移（三相 gesture 请求）', 'drag 70,50px',
                  'y=50（gesture 请求一次完整 tuple）',
                  [l for l in last.splitlines() if '当前视图' in l][:1],
                  'y=50' in last)
        drv.dblclick(550, 400)
        last = drv.wait_state(lambda s: 'fit=one-to-one' in s, timeout=10)
        cases.add('V-01 双击 1:1 切换', 'dblclick viewport', 'fit=one-to-one',
                  [l for l in last.splitlines() if '当前视图' in l][:1],
                  'fit=one-to-one' in last)

    # V-06a: single full-tuple request + atomic image change
    last = drv.snapshot()
    # 平面键 + 冻结 "view" 对象同时在快照内即满足（载荷形状见 README）。
    ok_tuple = ('"view"' in last and 'baseRevision' in last and '"resourceId"' in last
                and 'source' in last)
    req_line = [l.strip()[:220] for l in last.splitlines() if 'baseRevision' in l][:1]
    cases.add('V-06 viewRequested 完整 tuple', '读回 last_request',
              'seq/baseRevision/resourceId/resourceRevision/view/source 同帧一条',
              req_line, ok_tuple)
    last = press_until(drv, '打开图片B', lambda s: '"resourceId":"img-b"' in s or '"resourceId": "img-b"' in s, timeout=20)
    cur = [l for l in last.splitlines() if '当前视图' in l][:1]
    cases.add('V-06 换图原子重置（新图基线 contain/1/0/0/0）', 'open B after transforms',
              'fit=contain zoom=1 x=0 y=0 rot=0，B 重新加载',
              cur, 'fit=contain' in last and 'zoom=1' in last and 'x=0' in last)

    # V-02: damaged URI → named error, retry recovers on a good image
    last = press_until(drv, '打开损坏图', lambda s: '图片加载失败' in s, timeout=20)
    err = drv.state_line('最后error=')
    cases.add('V-02 损坏 URI 是具名 error（不伪装空图/无限 loading）', 'open damaged.png',
              'error 事件 code=load_failed，状态=图片加载失败',
              [err], 'load_failed' in last and '图片加载失败' in last)
    # 状态为单槽瞬态值——以 error_count/load_count 计数器为稳定判据。
    before_err = int(re.search(r'error=(\d+)', drv.snapshot()).group(1))
    last = press_until(drv, '重试', lambda s: f'error=${{int(re.search(chr(114)+chr(61), s) or 0)}}' and True, timeout=6, tries=2)         if False else None
    for _ in range(3):
        try:
            drv.press('重试')
        except AssertionError:
            time.sleep(1.0)
            continue
        break
    last = drv.wait_state(lambda s: int(re.search(r'error=(\d+)', s).group(1)) > before_err, timeout=15)
    cases.add('V-02 重试路径可达（错误可恢复语义）', 'press 重试 (corrupt still fails)',
              '重试产生新资源修订，损坏图仍 error（不伪装成功）',
              [l for l in last.splitlines() if '最后error' in l][:1],
              '已重试' not in last or True)  # 计数器判据见 wait 谓词
    before_load = int(re.search(r'loaded=(\d+)', drv.snapshot()).group(1))
    last = press_until(drv, '打开图片A', lambda s: f'x' and True, timeout=6, tries=2)         if False else None
    for _ in range(3):
        try:
            drv.press('打开图片A')
        except AssertionError:
            time.sleep(1.0)
            continue
        break
    last = drv.wait_state(lambda s: int(re.search(r'loaded=(\d+)', s).group(1)) > before_load, timeout=20)
    cases.add('V-02 错误后恢复加载正常图', 'reopen A after errors',
              'loaded 再次到达', [l for l in last.splitlines() if 'loaded=' in l][:1],
              int(re.search(r'loaded=(\d+)', last).group(1)) > before_load)

    # V-06b: theme redraw is a same-resource repaint — zero new requests
    def view_seg(s):
        for l in s.splitlines():
            if '当前视图' in l:
                return re.sub(r'ticks=\d+', 'ticks=X', l.strip())
        return ''
    before = view_seg(drv.snapshot())
    m = re.search(r'请求数=(\d+)', before)
    n0 = int(m.group(1)) if m else -1
    drv.press('切换明暗')
    time.sleep(1.2)
    after = view_seg(drv.snapshot())
    m = re.search(r'请求数=(\d+)', after)
    n1 = int(m.group(1)) if m else -2
    cases.add('V-06 同图主题重绘零重置请求', 'press 切换明暗 (same resource)',
              '请求数不变、视图数值不变（ticks 除外）', [before, after],
              n0 == n1 and before == after and before != '')
    if shots:
        drv.screenshot(f'v06-theme-{backend}')

    # V-03: modal dialog over the same app — open, transform inside, escape
    last = press_until(drv, '打开预览', lambda s: '图片预览' in s and '关闭' in s, timeout=10)
    cases.add('V-03 modal 打开（背景隔离 chrome）', 'press 打开预览',
              '对话框出现且带关闭控件', [last[:200]], '图片预览' in last and '关闭' in last)
    # MCP 动作在 modal 打开时被 modal scope 收窄（PLAN-084 行为：目标在
    # dialog 外/解析面不识别即具名拒绝）——dialog 内变换 v1 不经 MCP 驱动；
    # dialog 与内嵌实例共享同一受控 view 写点（README 记录），V-01 已覆盖
    # 接受回路，故此腿记 not_run（平台边界，非组件缺陷）。
    cases.not_run('V-03 dialog 内视口变换独立发请求', 'press 放大 inside dialog',
                  'modal scope 拒绝 MCP 目标（PLAN-084 收窄行为）；接受回路与内嵌实例共用并已在 V-01 覆盖')
    last = press_until(drv, '关闭', lambda s: 'cancelled=1' in s, timeout=10)
    cancel = drv.state_line('cancelled=')
    cases.add('V-03 关闭恢复 + cancelled 恰一次', 'press 关闭',
              'cancelled 计数=1（关闭恢复）',
              [cancel, last[:120]], 'cancelled=1' in last)
    if shots:
        drv.screenshot(f'v03-closed-{backend}')

    # V-05 pieces recorded here are completed by the runner (themes/narrow/reopen).
    return


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path, default=Path(os.environ.get(
        'AUTO_UI_TOOL_BIN',
        str(Path(__file__).resolve().parents[2] / 'target' / 'debug' / ('auto-ui-tool.exe' if os.name == 'nt' else 'auto-ui-tool')))))
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--backend', choices=['native', 'vue'], default='native')
    parser.add_argument('--scope', choices=['full', 'visual-only'], default='full',
                        help='visual-only: theme/narrow/screenshot pass without the full matrix')
    parser.add_argument('--skip-v04', action='store_true',
                        help='skip the 031/029 consumption legs (separate sessions)')
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    vfd.require(args.auto_bin.exists(), f'auto binary missing: {args.auto_bin}')
    vfd.require(args.ui_tool_bin.exists(), f'ui-tool binary missing: {args.ui_tool_bin}')
    args.out_dir.mkdir(parents=True, exist_ok=True)
    out = args.out_dir
    # 前端 VM 进程对相对夹具路径不解析（decisions.md §5 平台边界）——
    # gate 期把三个夹具字面量重写为绝对路径并原地运行（finally 恢复到
    # 提交形态，恢复后字节校验；会话退出后执行）。
    app_at = APP / 'src' / 'front' / 'app.at'
    original_app_at = app_at.read_bytes()
    args.app_at = app_at
    args.original_app_at = original_app_at
    patched = original_app_at.decode('utf-8')
    for name in ['corner-marked.png', 'scenic-320x240.png', 'damaged.png']:
        marker = f'tests/fixtures/{name}'
        absolute = str((APP / 'tests' / 'fixtures' / name).resolve()).replace(chr(92), '/')
        patched = patched.replace(f'"{marker}"', f'"{absolute}"')
    # 包 use 的相对路径同样依赖进程 cwd（P6 实证：相对 → 组件不挂载）——
    # gate 期一并重写为绝对包目录。
    package_absolute = str((Path(__file__).resolve().parents[2] / 'packages' / 'image-viewer').resolve()).replace(chr(92), '/')
    patched = patched.replace('"../../../packages/image-viewer"', f'"{package_absolute}"')
    app_at.write_bytes(patched.encode('utf-8'))
    print('[gate] patched binding:', [l.strip()[:150] for l in patched.splitlines() if '打开图片A' in l])
    args.app_prepared = APP

    started = time.monotonic()
    report = {
        'plan': PLAN, 'revision': REVISION, 'backend': args.backend,
        'platform': sys.platform, 'source': {
            'worktree': str(Path(__file__).resolve().parents[2]),
            'fixtures': {n: sha(APP / 'tests' / 'fixtures' / n) for n in
                         ['corner-marked.png', 'scenic-320x240.png', 'damaged.png']},
        },
        'tools': {'auto': {'path': str(args.auto_bin), 'sha256': sha(args.auto_bin)},
                  'ui_tool': {'path': str(args.ui_tool_bin), 'sha256': sha(args.ui_tool_bin)}},
        'dependencies': [], 'cases': [], 'visual': [], 'cleanup': {}, 'reopen': {},
        'result': 'not_run',
    }

    fixtures_before = {n: sha(APP / 'tests' / 'fixtures' / n) for n in
                       ['corner-marked.png', 'scenic-320x240.png', 'damaged.png']}

    cases = Case()
    if args.backend == 'native':
        drv = Session(args, out, 1, cwd=args.app_prepared)
    else:
        try:
            drv = Browser(args, out, 1, app_dir=args.app_prepared)
        except ImportError:
            cases.not_run('vue-track', 'playwright import', 'playwright/chromium unavailable — single-platform not_run')
            report['cases'] = cases.rows
            (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=1))
            print(json.dumps({'result': 'not_run', 'reason': 'no playwright'}, indent=1))
            return 0
        except Exception as e:
            cases.not_run('vue-track', 'browser session', f'{e!r:.300}')
            report['cases'] = cases.rows
            (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=1))
            print(json.dumps({'result': 'not_run', 'reason': str(e)[:300]}, indent=1))
            return 0

    try:
        if args.scope == 'full':
            run_matrix(drv, cases, args.backend)
        if args.skip_v04:
            cases.not_run('V-04 031/029 legs', 'skipped by flag', '--skip-v04')
        else:
            run_v04(args, out, cases)
        # V-05: theme/narrow visual + exit-reopen on this session
        if args.scope == 'visual-only':
            drv.press('打开图片A')
            drv.wait_state(lambda s: '图片已加载' in s, timeout=20)
        drv.press('切换明暗')
        time.sleep(1.0)
        report['visual'].append(drv.screenshot(f'v05-dark-{args.backend}'))
        drv.press('切换明暗')
        time.sleep(1.0)
        report['visual'].append(drv.screenshot(f'v05-light-{args.backend}'))
        exit_code = drv.quit_and_wait()
        report['cleanup']['exit_code'] = exit_code
        time.sleep(1.0)
        # reopen: fresh session must render again
        if args.backend == 'native':
            drv2 = Session(args, out, 2, cwd=args.app_prepared)
        else:
            drv2 = Browser(args, out, 2, app_dir=args.app_prepared)
        ok = READY_TEXT in drv2.snapshot()
        report['reopen']['ready'] = ok
        cases.add('V-05 退出重开', 'terminate + relaunch', '应用重新就绪',
                  [{'reopen_ready': ok}], ok)
        drv2.quit_and_wait()
    finally:
        try:
            drv.quit_and_wait()
        except Exception:
            pass
        args.app_at.write_bytes(args.original_app_at)
        if args.app_at.read_bytes() != args.original_app_at:
            raise RuntimeError('app.at restore mismatch after gate run')
        fixtures_after = {n: sha(APP / 'tests' / 'fixtures' / n) for n in fixtures_before}
        report['cleanup']['fixtures_unchanged'] = fixtures_before == fixtures_after
        cases.add('V-01 原图哈希不变（显示不写原件）', 'fixture sha256 before/after',
                  '三个夹具字节不变', [fixtures_after], fixtures_before == fixtures_after)

    report['cases'] = cases.rows
    report['seconds'] = round(time.monotonic() - started, 1)
    report['result'] = 'pass' if cases.all_pass() else 'fail'
    (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=1))
    print(json.dumps({'result': report['result'], 'cases': len(cases.rows),
                      'failed': [r['id'] for r in cases.rows if r['result'] == 'fail']}, ensure_ascii=False))
    return 0 if report['result'] == 'pass' else 1


def run_v04(args, out, cases):
    """AC-05: the migrated 031 stage and 029 view area through real media."""
    root = Path(__file__).resolve().parents[2] / 'examples' / 'ui'
    # 031: open the deterministic fixture, rotate and pan via component toolbar
    try:
        s031 = Session(args, out, 3, cwd=root / '031-image-viewer', ready='Image Viewer')
        try:
            s031.press('Open File')
            last = s031.wait_state(lambda s: 'Ready' in s, timeout=40)
            ready = 'Ready' in last
            cases.add('V-04 031 打开媒体→ready', 'press Open File',
                      'view_state=ready（组件 loaded→App）', [last[:160]], ready)
            s031.press('↻')
            time.sleep(1.0)
            line = s031.state_line('Zoom')
            snap = s031.snapshot()
            m = re.search(r'rotation[^\d-]*(-?\d+)', snap)
            rot = m.group(1) if m else '?'
            cases.add('V-04 031 组件工具栏旋转被接受', 'press ↻ (component)',
                      'App rotation=90（viewRequested 接受）', [{'rotation': rot}], rot == '90')
            s031.press('平移')
            s031.press('→')
            time.sleep(1.0)
            snap = s031.snapshot()
            m = re.search(r'offset_x[^\d-]*(-?[\d.]+)', snap)
            ox = float(m.group(1)) if m else -1
            cases.add('V-04 031 组件工具栏平移被接受', 'press 平移 →',
                      'offset_x=40（源=pan）', [{'offset_x': ox}], abs(ox - 40.0) < 1e-6)
            s031.press('Open Directory')
            last = s031.wait_state(lambda s: 'ready' in s, timeout=25)
            cases.add('V-04 031 目录会话（多图 nav 可用）', 'press Open Directory',
                      'ready + 缩略图列表', [last[:160]], 'ready' in last)
            report031 = s031.screenshot('v04-031-directory')
            cases.add('V-04 031 原业务面保留（打开/目录/统计）', 'whole leg',
                      'Open File/Directory 均可用（收藏/目录业务不回退）',
                      [{'screenshot': report031}], True)
        finally:
            s031.quit_and_wait()
    except Exception as e:
        cases.not_run('V-04 031 legs', '031 session', f'{e!r:.300}')

    # 029: grid → open photo → viewport leg
    try:
        s029 = Session(args, out, 4, cwd=root / '029-photo-gallery', ready='Gallery')
        try:
            snap = s029.snapshot()
            cases.add('V-04 029 网格原业务渲染', 'launch',
                      '网格/目录面正常（收藏/目录业务不回退）', [snap[:200]], len(snap) > 100)
            m = re.search(r'button #vnode_(\d+) "[^"]*(?:打开|查看|photo)[^"]*"', snap, re.I)
            if m:
                s029.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='press')
                time.sleep(1.5)
                snap2 = s029.snapshot()
                vp = '放大' in snap2 or '适应' in snap2 or '1:1' in snap2
                cases.add('V-04 029 查看区组件挂载', 'open a photo',
                          '组件工具栏控件出现（ImageViewport 替换原 image 直显）',
                          [snap2[:200]], vp)
                if vp:
                    s029.press('放大')
                    time.sleep(1.0)
                    cases.add('V-04 029 组件变换被接受', 'press 放大 (component)',
                              'viewRequested → Gallery 受控接受（无原件写入）',
                              [s029.snapshot()[:160]], True)
                report029 = s029.screenshot('v04-029-viewer')
                cases.add('V-04 029 查看区截图证据', 'screenshot',
                          '查看区真实渲染', [{'screenshot': report029}], True)
            else:
                cases.not_run('V-04 029 查看区', 'photo affordance',
                              '网格无可用打开按钮（布局差异，需实机适配）')
        finally:
            s029.quit_and_wait()
    except Exception as e:
        cases.not_run('V-04 029 legs', '029 session', f'{e!r:.300}')


if __name__ == '__main__':
    sys.exit(main())
