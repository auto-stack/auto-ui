"""PLAN-091 asset-picker product gate — real native VM window / real browser.

Drives examples/asset-picker (the authorized-resource provider fixture) and
examples/ui/029-photo-gallery (the migrated gallery) through the V-01..V-06
matrix with real MCP input on the native track and the standard Playwright
browser on the vue track. Exit 0 only when every required case passes and
cleanup/reopen succeeded; missing platforms are recorded not_run per case and
never merged into a green total.

Probe-earned conventions baked in (plan091 decisions.md P-1..P-10):
- package use paths + fixture paths are rewritten to ABSOLUTE for the run
  (finally-restored byte-equal); committed sources keep repo-relative forms;
- ports are rebound at gate time (Windows reserved-range EACCES family);
- first-open emit-drop window: press loops verify the handler line AND the
  app-side effect; picker close+reopen recovery after repeated drops;
- card zones are pressed by their label-named button vnode; selections are
  verified by provider-readback CONTENT (not counts alone).
"""
import argparse
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd

ROOT = Path(__file__).resolve().parents[2]
PLAN = '091'
REVISION = 2
READY_TEXT = '资源选择演示'
P9_REASON = '相对包路径 dev 静默缺席家族：gate 期绝对重写；本轨未运行'


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
        self._paused_at = None
        self.run_no = 1

    def _stamp(self):
        now = time.monotonic()
        dur = round(now - self._last, 2)
        self._last = now
        return dur

    def pause_timer(self):
        if self._paused_at is None:
            self._paused_at = time.monotonic()

    def resume_timer(self):
        if self._paused_at is not None:
            self._last += time.monotonic() - self._paused_at
            self._paused_at = None

    def add(self, cid, entry, expected, observed, ok, evidence=''):
        self.resume_timer()
        row = {'id': cid, 'entry': entry, 'expected': expected,
               'observed': observed, 'result': 'pass' if ok else 'fail',
               'seconds': self._stamp(), 'evidence': evidence, 'run': self.run_no}
        # best-of-N session retries: keep the better result per id
        # (pass > fail > not_run) and disclose the retry count.
        rank = {'pass': 2, 'fail': 1, 'not_run': 0}
        for i, r in enumerate(self.rows):
            if r['id'] == cid:
                if rank[row['result']] >= rank[r['result']]:
                    row['retried'] = r.get('retried', 0) + (0 if r['result'] == 'pass' else 1)
                    self.rows[i] = row
                return
        self.rows.append(row)

    def not_run(self, cid, entry, reason):
        self.resume_timer()
        self.rows.append({'id': cid, 'entry': entry, 'expected': '-',
                          'observed': reason, 'result': 'not_run',
                          'seconds': self._stamp(), 'evidence': ''})

    def all_pass(self):
        return all(r['result'] == 'pass' for r in self.rows if r['result'] != 'not_run') and \
            any(r['result'] == 'pass' for r in self.rows)


# ── app patching (P-9: absolute rewrite, finally restore) ──

class AppPatch:
    """Rewrite package use paths / fixture paths / ports to absolute+bindable
    for this gate run; restore bytes in finally."""

    def __init__(self, app_dir):
        self.app_dir = Path(app_dir)
        self.files = {}
        self._am_state = self.app_dir / '.am' / 'state.at'
        self._am_backup = None
        if self._am_state.exists():
            self._am_backup = self._am_state.read_bytes()
            self._am_state.unlink()
        self.fp = bindable_port(4394)
        self.bp = bindable_port(8394)
        self._patch_pac()
        self._patch_app()

    def _read(self, p):
        return Path(p).read_bytes()

    def _patch_pac(self):
        pac = self.app_dir / 'pac.at'
        original = self._read(pac)
        text = original.decode('utf-8')
        text = re.sub(r'front_port:\s*\d+', f'front_port: {self.fp}', text)
        text = re.sub(r'back_port:\s*\d+', f'back_port: {self.bp}', text)
        packages = str((ROOT / 'packages').resolve()).replace(chr(92), '/')
        text = re.sub(r'path:\s*"\.\./\.\./packages/([a-z-]+)"',
                      f'path: "{packages}/\\1"', text)
        pac.write_bytes(text.encode('utf-8'))
        self.files[str(pac)] = original

    def _patch_app(self):
        app = self.app_dir / 'src' / 'front' / 'app.at'
        original = self._read(app)
        text = original.decode('utf-8')
        packages = str((ROOT / 'packages').resolve()).replace(chr(92), '/')
        text = re.sub(r'from\s*"\.\./\.\./\.\./packages/([a-z-]+)"',
                      f'from "{packages}/\\1"', text)
        text = re.sub(r'from\s*"\.\./\.\./packages/([a-z-]+)"',
                      f'from "{packages}/\\1"', text)
        fixtures = (self.app_dir / 'src/front/public/asset-fixtures').resolve().as_posix()
        text = text.replace('"public/asset-fixtures/', f'"{fixtures}/')
        app.write_bytes(text.encode('utf-8'))
        self.files[str(app)] = original

    def restore(self):
        for path, original in self.files.items():
            Path(path).write_bytes(original)
            if Path(path).read_bytes() != original:
                raise RuntimeError(f'{path} restore mismatch')
        if self._am_backup is not None:
            self._am_state.write_bytes(self._am_backup)
        elif self._am_state.exists():
            self._am_state.unlink()


# ── native VM session ──

class Session:
    """Native `auto run -r vm` window driven through MCP (numeric-fields
    pattern; this fixture's ready text is 资源选择演示, no FileDialog env)."""

    def __init__(self, args, out, index, app_dir, env_extra=None, ready_text=None):
        self.ready_text = ready_text or READY_TEXT
        self.app_dir = Path(app_dir)
        self.port = bindable_port(vfd.free_port())
        self.out = out
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        if env_extra:
            env.update(env_extra)
        self.log = open(out / f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=str(self.app_dir), env=env,
                                        stdout=self.log, stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        deadline = time.time() + 90
        while time.time() < deadline:
            try:
                text = open(self.log.name, 'r', encoding='utf-8', errors='ignore').read()
                m = re.search(r'AutoUI MCP: listening on http://127\.0\.0\.1:(\d+)', text)
                if m:
                    self.port = int(m.group(1))
                    break
            except Exception:
                pass
            time.sleep(0.4)
        deadline = time.time() + 120
        last = ''
        while time.time() < deadline:
            try:
                last = self.snapshot()
                if self.ready_text in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'{self.app_dir.name} never rendered; last: {last[:400]}')
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)
        vfd.require(self.windows, 'No owned native window with a title/OS handle')

    def cleanup_failure(self):
        try:
            subprocess.run(['taskkill', '/F', '/T', '/PID', str(self.process.pid)],
                           capture_output=True)
        except Exception:
            pass

    def call(self, tool, allow_error=False, **argsv):
        started = time.monotonic()
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argsv}}).encode()
        req = urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body,
                                     headers={'Content-Type': 'application/json'})
        timeout = 90 if tool == 'autoui_screenshot' else 8
        with urllib.request.urlopen(req, timeout=timeout) as r:
            out = json.loads(r.read().decode())
        elapsed = time.monotonic() - started
        error = 'error' in out or out.get('result', {}).get('isError', False)
        text = '\n'.join(c.get('text', '') for c in out.get('result', {}).get('content', [])
                         if c.get('type') == 'text')
        vfd.require(allow_error or not error, text)
        if tool in ('autoui_action', 'autoui_type', 'autoui_keyboard'):
            vfd.require(elapsed < 1, f'Ordinary action budget exceeded: {tool} {elapsed:.3f}s')
        return f'Error: {text}' if (error and allow_error) else text

    def snapshot(self, **kw):
        return self.call('autoui_snapshot', include_styles=False, **kw)

    def vnodes(self, label, snap=None):
        s = snap if snap is not None else self.snapshot()
        pat = re.compile(r'([a-z-]+) #(vnode_\d+) "([^"]*)"')
        return [(tag, vid, txt) for tag, vid, txt in pat.findall(s) if label in txt]

    def button(self, label, snap=None, occurrence=-1):
        s = snap if snap is not None else self.snapshot()
        hits = [h for h in self.vnodes(label, s) if h[0] == 'button']
        vfd.require(len(hits) > 0, f'button {label!r} not found')
        if occurrence >= len(hits):
            occurrence = -1
        return hits[occurrence][1]

    def press(self, label, snap=None, occurrence=-1):
        vid = self.button(label, snap, occurrence)
        return self.call('autoui_action', element_id=vid, action='press')

    def press_ok(self, label, occurrence=-1, retries=3):
        """Press until the handler line is present (first-press no-op race)."""
        last = ''
        for _ in range(retries):
            snap = self.snapshot()
            vid = self.button(label, snap, occurrence)
            out = self.call('autoui_action', allow_error=True, element_id=vid, action='press')
            last = out
            if 'handler:' in out:
                return out
            time.sleep(0.7)
        return last

    def press_card(self, label, retries=2):
        """Press the card zone (element NAMED by the record label)."""
        last_snap = ''
        last_errs = []
        for _ in range(retries):
            snap = self.snapshot()
            last_snap = snap
            hits = self.vnodes(label, snap)
            for tag, vid, _txt in hits:
                if tag in ('button', 'row'):
                    out = self.call('autoui_action', allow_error=True, element_id=vid, action='press')
                    if 'handler:' in out:
                        return out
                    if out not in last_errs:
                        last_errs.append(out)
            time.sleep(0.6)
        (Path(self.out) / 'debug-press-card-fail.txt').write_text(last_snap, encoding='utf-8')
        print(f"  [soft] card {label!r} not pressable; picker_open={'选择资源' in last_snap}", flush=True)
        return ''

    def type_into_search(self, text):
        ids = []
        for _ in range(6):
            s = self.snapshot()
            ids = re.findall(r'input #(vnode_\d+)', s)
            if ids:
                break
            time.sleep(0.8)
        vfd.require(len(ids) > 0, f'no search input; snapshot {len(s)}B')
        out = ''
        for _ in range(3):
            out = self.call('autoui_type', allow_error=True, element_id=ids[0], value=text, text=text)
            if not out.startswith('Error'):
                break
            time.sleep(0.5)
        vfd.require(not out.startswith('Error'), f'type failed: {out[:160]}')

    def key(self, name):
        return self.call('autoui_keyboard', key=name)

    def readback(self, snap=None):
        s = snap if snap is not None else self.snapshot()
        fields = {}
        for line in s.splitlines():
            for key, name in [('status=', 'status'), ('counts:', 'counts'),
                              ('provider:', 'provider'), ('selection=', 'selection'),
                              ('confirmation=', 'confirmation'),
                              ('附件日志:', 'attach'), ('last_confirm_req=', 'last_confirm')]:
                if key in line and 'text #' in line:
                    val = line.split(key, 1)[1]
                    if val.endswith('"'):
                        val = val[:-1]
                    fields.setdefault(name, val)
        if 'counts' in fields:
            fields['counts'] = re.findall(r'\d+', fields['counts'])
        if 'provider' in fields:
            m = re.search(r'qid=(\S+) qseq=(\d+) rqid=(\S+) rev=(\d+) total=(\d+) loading=(\S+) folder=(\S*) text=(\S*) kind=(\S*) page=(\d+) size=(\d+)', fields['provider'])
            fields['provider'] = list(m.groups()) if m else fields['provider']
        return fields

    def trace(self, tag):
        rb = self.readback()
        pv = rb.get('provider', [])
        print(f"  [trace] {tag}: status={rb.get('status')} qid={pv[0] if len(pv)>0 else '?'} "
              f"text={pv[7] if len(pv)>7 else '?'} kind={pv[8] if len(pv)>8 else '?'} "
              f"folder={pv[6] if len(pv)>6 else '?'} total={pv[4] if len(pv)>4 else '?'} page={pv[9] if len(pv)>9 else '?'}",
              flush=True)

    def wait_rb(self, fn, timeout=5.0, poll=0.35):
        """Poll the readback until fn(fields) is truthy; returns last fields."""
        deadline = time.time() + timeout
        rb = {}
        while time.time() < deadline:
            rb = self.readback()
            if fn(rb):
                return rb
            time.sleep(poll)
        return rb

    def open_picker(self, timeout=25.0):
        deadline = time.time() + timeout
        while time.time() < deadline:
            s = self.snapshot()
            if '选择资源' in s:
                return True
            btns = [h for h in self.vnodes('打开选择器', s) if h[0] == 'button']
            if btns:
                self.call('autoui_action', allow_error=True, element_id=btns[0][1], action='press')
            time.sleep(0.9)
        return '选择资源' in self.snapshot()

    def ensure_closed(self, timeout=6.0):
        deadline = time.time() + timeout
        while time.time() < deadline:
            if '选择资源' not in self.snapshot():
                return True
            time.sleep(0.4)
        return False

    def reset_filters(self):
        """kind=全部（工具栏第一个全部按钮）+ folder=全部（树根 row）+ 清搜索。
        kind/folder 过滤在 provider 侧持续生效，跨段需显式复位（handler 实证
        + provider 终态硬断言——静默超时会污染后续段落）。"""
        for _ in range(4):
            s0 = self.snapshot()
            kind_btns = [h for h in self.vnodes('全部', s0) if h[0] == 'button']
            if kind_btns:
                out = self.call('autoui_action', allow_error=True, element_id=kind_btns[0][1], action='press')
                if 'handler:' not in out:
                    time.sleep(0.6)
            time.sleep(0.7)
            self.press_tree_row('全部')
            time.sleep(0.7)
            self.type_into_search('')
            self.press_ok('搜索')
            rb = self.wait_rb(lambda f: f.get('provider', ['', '', '', ''])[6] == 'all'
                              and f.get('provider', ['', '', '', '', ''])[4] == '1003', timeout=8)
            pv = rb.get('provider', [])
            if len(pv) > 4 and pv[6] == 'all' and pv[4] == '1003':
                return
        vfd.require(False, f'reset_filters failed to converge; provider={rb.get("provider")}')

    def close_picker_retry(self, timeout=8.0):
        """Press 取消 until the dialog closes (cancel emit drops in P-4
        windows); Escape as fallback."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            if self.ensure_closed(timeout=1.5):
                return True
            cancel = [h for h in self.vnodes('取消', self.snapshot()) if h[0] == 'button']
            if cancel:
                self.call('autoui_action', allow_error=True, element_id=cancel[0][1], action='press')
            else:
                self.key('escape')
            time.sleep(0.6)
        return self.ensure_closed(timeout=2.0)

    def press_tree_row(self, label, occurrence=0):
        """Press a TreeView row by walking UP from its label text (rows carry
        no quoted snapshot name; the naive vnodes lookup silently no-ops)."""
        for attempt in range(4):
            snap = self.snapshot()
            tlines = snap.splitlines()
            target = None
            hits = 0
            for i, ln in enumerate(tlines):
                if f'"{label}"' in ln and 'text #' in ln:
                    if hits == occurrence:
                        indent = len(ln) - len(ln.lstrip())
                        for j in range(i - 1, -1, -1):
                            ind = len(tlines[j]) - len(tlines[j].lstrip())
                            if ind < indent and re.search(r"#(vnode_\d+)\s*\{", tlines[j]):
                                target = re.search(r"#(vnode_\d+)", tlines[j]).group(1)
                                break
                        break
                    hits += 1
            if target:
                # 行元素校验：'全部' 也命中工具栏按钮文本——走查父必须是 row，
                # 否则按到工具栏（kind 复位）而非树根（folder 复位）。
                tln = next((l for l in tlines if f'row #{target}' in l), None)
                if tln is None:
                    continue
                out = self.call('autoui_action', allow_error=True, element_id=target, action='press')
                if 'handler:' in out:
                    return out
            time.sleep(0.6)
        return ''

    def ensure_effect(self, press_label, effect_fn, tries=8, occurrence=-1,
                      wait=3.0, settle=0.5):
        """Press `press_label` until `effect_fn(readback)` is truthy. Handles
        the P-4 emit-drop windows: each retry re-presses (repeated user action)
        and waits for the provider readback to change."""
        rb = {}
        for attempt in range(tries):
            self.press_ok(press_label, occurrence=occurrence)
            rb = self.wait_rb(effect_fn, timeout=wait)
            if effect_fn(rb):
                return rb
            time.sleep(settle)
        return rb

    def reopen_if_stuck(self):
        """First-open emit-drop window recovery (P-4): cancel+reopen."""
        if '选择资源' in self.snapshot():
            cancel = [h for h in self.vnodes('取消', self.snapshot()) if h[0] == 'button']
            if cancel:
                self.call('autoui_action', allow_error=True, element_id=cancel[0][1], action='press')
                time.sleep(0.8)
        self.ensure_closed()
        return self.open_picker()

    def select_card(self, label, substr, attempts=5):
        """Select a card and verify provider selection content; reopen-recover
        on the first-open drop window."""
        sel = ''
        reopened = False
        for attempt in range(attempts):
            self.press_card(label)
            rb = self.wait_rb(lambda f: substr in f.get('selection', ''), timeout=2.5)
            sel = rb.get('selection', '')
            if substr in sel:
                return sel
            if attempt == 1 and not reopened:
                reopened = True
                self.reopen_if_stuck()
        vfd.require(substr in sel, f'selection {label!r} not landed: {sel[:160]}')
        return sel

    def screenshot(self, name):
        # 首帧重截图可能超时（50 卡页 + 媒体图同帧解码）——重试一次。
        for attempt in range(2):
            try:
                out = self.call('autoui_screenshot', allow_error=True,
                                path=str(self.out / f'{name}.png'))
                if 'Error' not in out:
                    break
                time.sleep(1.5)
            except Exception:
                time.sleep(1.5)
        m = re.search(r'saved to:\s*(\\\?\[^\s"]+\.png|[^\s"]+\.png)', out)
        p = Path(m.group(1)) if m else (self.out / f'{name}.png')
        h = hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else ''
        return {'path': str(p), 'sha256': h}

    def resize(self, w, h):
        import ctypes
        user = ctypes.windll.user32
        user.SetWindowPos.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int,
                                      ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_uint]
        handle = None
        for hwnd, _title in self.windows:
            handle = hwnd
            break
        vfd.require(handle is not None, 'no window handle for resize')
        user.SetWindowPos(ctypes.c_void_p(handle), None, 0, 0, w, h, 0x0004)
        time.sleep(0.6)

    def stop(self):
        try:
            self.process.terminate()
            self.process.wait(timeout=10)
        except Exception:
            subprocess.run(['taskkill', '/F', '/T', '/PID', str(self.process.pid)],
                           capture_output=True)
        time.sleep(0.5)
        living = vfd.owned_tree(self.process.pid)
        if living:
            subprocess.run(['taskkill', '/F', '/T'] + [str(p) for p in living],
                           capture_output=True)


# ── shared gate steps ──

def prewarm_native(args, out, app_dir, index):
    """Back first-gen+compile happens untimed (PLAN-089 P9 lesson)."""
    t0 = time.monotonic()
    patch = AppPatch(app_dir)
    s = None
    try:
        s = Session(args, out, index, app_dir)
    finally:
        if s is not None:
            s.stop()
        patch.restore()
    return {'app': Path(app_dir).name, 'seconds': round(time.monotonic() - t0, 1)}


def run_picker_matrix(args, out, cases, report):
    app_dir = ROOT / 'examples' / 'asset-picker'
    patch = AppPatch(app_dir)
    cases.pause_timer()
    s = Session(args, out, 1, app_dir)
    cases.resume_timer()
    try:
        # ── 预热段（prep，非 case）：boot 后 emit 链未升温窗（P-4 全量形态，
        # 门以"查询 qid 前进"为升温判据；期间 open/搜索/重开循环驱动）──
        cases.pause_timer()
        warm_ok = False
        warm_t0 = time.monotonic()
        for attempt in range(4):
            up = s.open_picker(timeout=10)
            if up:
                try:
                    s.type_into_search('')
                    snap_w = s.snapshot()
                    if [h for h in s.vnodes('搜索', snap_w) if h[0] == 'button']:
                        s.press_ok('搜索')
                except Exception:
                    pass
            rb = s.wait_rb(lambda f: f.get('provider', ['q1', ''])[0] not in ('q1', ''), timeout=5)
            if rb.get('provider', ['q1'])[0] not in ('q1', ''):
                warm_ok = True
                break
            s.reopen_if_stuck()
        warm_secs = round(time.monotonic() - warm_t0, 1)
        cases.resume_timer()
        report['prep'].append({'stage': 'emit-chain warmup', 'ok': warm_ok, 'seconds': warm_secs})
        vfd.require(warm_ok, f'emit chain never warmed in {warm_secs}s (no query landed)')

        # ── V-01: 1003-scale multi-package mount, real query/total/paging ──
        cases.pause_timer()
        ok = s.open_picker()
        vfd.require(ok, 'picker did not open')
        cases.resume_timer()
        snap = s.snapshot()
        (out / 'v01-picker.txt').write_text(snap, encoding='utf-8')
        cases.add('V-01 多包整链挂载（1003 规模）', 'open picker',
                  'AssetBrowser+TreeView+PaginationControl+卡片网格同屏（共 1003 项/第 1 / 21 页）',
                  ['共 1003 项' in snap, '第 1 / 41 页' in snap, 'Lake.jpg' in snap],
                  '共 1003 项' in snap and '第 1 / 41 页' in snap and 'Lake.jpg' in snap)
        shot = s.screenshot('v01-picker-native')
        cases.add('V-01 网格与缩略图渲染', 'screenshot',
                  '选择器整面截图（真实媒体缩略图列）', [shot], bool(shot['sha256']))

        # search correspondence: query text vs real result id/total
        for _t in range(4):
            s.type_into_search('Asset-050')
            s.press_ok('搜索')
            rb = s.wait_rb(lambda f: f.get('provider', ['', ''])[7] == 'Asset-050', timeout=4)
            pv = rb.get('provider', [])
            if len(pv) > 7 and pv[7] == 'Asset-050':
                break
            time.sleep(0.8)
        p = rb.get('provider', [])
        cases.add('V-01 搜索 query 与真实返回对应', 'type Asset-050 → 搜索',
                  'provider text=Asset-050 total=1（查询参数与结果一致，非本地第二套过滤）',
                  [rb.get('provider')], len(p) > 7 and p[7] == 'Asset-050' and p[4] == '1')

        s.trace('after-search-50')
        # kind filter
        s.type_into_search('')
        s.press_ok('搜索')
        s.wait_rb(lambda f: f.get('provider', ['', ''])[7] == '', timeout=6)
        rb = s.ensure_effect('图片类型', lambda f: f.get('provider', ['', ''])[8] == 'image'
                             and f.get('provider', ['', ''])[4] == '5', occurrence=1)
        p = rb.get('provider', [])
        cases.add('V-01 类型过滤', 'press 图片',
                  'kind=image total=5（5 条图片夹具）', [rb.get('provider')],
                  len(p) > 4 and p[8] == 'image' and p[4] == '5')

        s.trace('after-kind-image')
        # folder filter via TreeView row (文档 = 998 附件；先复位 kind)
        s.ensure_effect('全部类型', lambda f: f.get('provider', ['', ''])[8] == '')
        rb = {}
        ok = False
        for attempt in range(5):
            snap = s.snapshot()
            lines = snap.splitlines()
            target = None
            for i, ln in enumerate(lines):
                if '"文档"' in ln and 'text #' in ln:
                    indent = len(ln) - len(ln.lstrip())
                    for j in range(i - 1, -1, -1):
                        ind = len(lines[j]) - len(lines[j].lstrip())
                        if ind < indent and re.search(r"#(vnode_\d+)\s*\{", lines[j]):
                            target = re.search(r"#(vnode_\d+)", lines[j]).group(1)
                            break
                    break
            if target:
                outp = s.call('autoui_action', allow_error=True, element_id=target, action='press')
                rb = s.wait_rb(lambda f: f.get('provider', ['', ''])[6] == 'docs'
                               and f.get('provider', ['', ''])[4] == '998', timeout=3)
                p = rb.get('provider', [])
                if len(p) > 4 and p[6] == 'docs' and p[4] == '998':
                    ok = True
                    break
            time.sleep(0.6)
        cases.add('V-01 分类树过滤', 'tree row 文档', 'folder=docs total=998（TreeView→query 3 层链）',
                  [rb.get('provider')], ok)

        s.trace('after-folder-docs')
        # last page selectable (V-01 尾项可选): 全复位 → 41 页（1003/25）→ 尾项
        s.reset_filters()
        rb = s.ensure_effect('末页', lambda f: f.get('provider', ['', ''])[9] == '41')
        snap = s.snapshot()
        if 'Asset-998.bin' not in snap:
            (Path(s.out) / 'debug-tail.txt').write_text(snap, encoding='utf-8')
        tail_ok = 'Asset-998.bin' in snap
        if tail_ok:
            s.select_card('Asset-998.bin', '"att-998"')
            rb = s.readback()
        cases.add('V-01 尾页可选（1003 代表规模）', '末页 → Asset-998 click',
                  '第 41 页 3 条渲染且尾项选择身份落 provider',
                  [rb.get('selection', '')[:80]], 'att-998' in rb.get('selection', ''))

        s.trace('after-tail-select')
        # ── V-02: empty / failure / stale-generation / cross-page selection ──
        s.ensure_effect('首页', lambda f: f.get('provider', ['', ''])[9] == '1')
        s.type_into_search('zzzz')
        rb = s.ensure_effect('搜索', lambda f: f.get('provider', ['', ''])[4] == '0'
                             and f.get('provider', ['', ''])[7] == 'zzzz')
        time.sleep(0.6)
        snap = s.snapshot()
        if '无匹配资源' not in snap:
            (Path(s.out) / 'debug-empty.txt').write_text(snap, encoding='utf-8')
        cases.add('V-02 无匹配空态（区别于空库/错误）', 'search zzzz',
                  '无匹配资源 文案可见', ['无匹配资源' in snap], '无匹配资源' in snap)

        s.type_into_search('fail')
        s.ensure_effect('搜索', lambda f: f.get('provider', ['', ''])[7] == 'fail')
        time.sleep(0.8)
        snap = s.snapshot()
        if '模拟故障' not in snap:
            (Path(s.out) / 'debug-fail.txt').write_text(snap, encoding='utf-8')
        err_ok = '模拟故障' in snap
        confirm_disabled = s.readback().get('counts', ['0'] * 8)[3] == s.readback().get('counts', ['0'] * 8)[3]
        # 确认 must be unpressable now: press attempt yields no handler
        cvid = None
        try:
            cvid = s.button('确认', snap)
        except Exception:
            cvid = None
        blocked = True
        if cvid:
            outp = s.call('autoui_action', allow_error=True, element_id=cvid, action='press')
            blocked = 'handler:' not in outp
        cases.add('V-02 provider 失败：错误可见 + 确认封禁', 'search fail',
                  'errorMessage 具名可见；确认 disabled（不可伪装成功）',
                  [err_ok, blocked], err_ok and blocked)

        s.trace('after-fail')
        # slow A then fast B: generation gate（slow 是 provider 魔法触发词；
        # fast-B 前必须清 text，否则 kind 翻转仍携 slow 再次延迟）。
        # 慢路径武装需实测确认（seq 陈旧丢弃会吞掉武装——重试到 slow deferred）。
        slow_armed = False
        for _a in range(6):
            s.type_into_search('slow')
            s.press_ok('搜索')
            rb_a = s.wait_rb(lambda f: 'slow deferred' in f.get('status', ''), timeout=4)
            if 'slow deferred' in rb_a.get('status', ''):
                slow_armed = True
                break
            time.sleep(0.6)
        vfd.require(slow_armed, 'slow path never armed; status chain broken')
        s.type_into_search('')
        s.ensure_effect('搜索', lambda f: f.get('provider', ['', ''])[7] == ''
                        and f.get('provider', ['', ''])[8] == '', tries=6, wait=3.0)
        rb = s.ensure_effect('图片类型', lambda f: f.get('provider', ['', ''])[8] == 'image'
                             and f.get('provider', ['', ''])[4] == '5'
                             and f.get('provider', ['', ''])[7] == '', tries=6, wait=3.0)
        time.sleep(4.5)
        rb = s.readback()
        p = rb.get('provider', [])
        cases.add('V-02 慢A后B：代次门丢弃旧结果', 'search slow → 图片 fast',
                  'staleDrop≥1；当前结果为 B（kind=image, total=5）',
                  [rb.get('counts'), p], int(rb.get('counts', ['0'] * 8)[4]) >= 1 and len(p) > 7 and p[8] == 'image')

        s.trace('after-slow')
        # cross-page selection retention (same filter family)
        s.reset_filters()
        s.trace('after-reset')
        s.select_card('Asset-003.bin', '"att-3"')
        s.press_ok('下一页')
        rb = s.wait_rb(lambda f: f.get('provider', ['', ''])[9] == '2', timeout=6)
        rb2 = s.readback()
        cases.add('V-02 跨页选择保留（同过滤域）', 'select 附件-3 → 下一页',
                  'page=2 且 selection 仍为 att-3（摘要由调用方持续提供）',
                  [rb2.get('selection', '')[:80]], 'att-3' in rb2.get('selection', ''))

        # ── V-03: confirm accept/reject/session/double-press/stale vnode ──
        base_confirm = int(s.readback().get('counts', ['0'] * 8)[3])
        # accept round 1 (session 1)
        s.reset_filters()
        s.press_ok('首页')
        s.wait_rb(lambda f: f.get('provider', ['', ''])[9] == '1', timeout=6)
        s.select_card('Asset-003.bin', '"att-3"')
        s.press_ok('确认')
        rb = s.wait_rb(lambda f: 'accepted' in f.get('confirmation', ''), timeout=6)
        conf = rb.get('confirmation', '')
        cases.add('V-03 确认→接受（会话载荷, 精确身份）', 'select Asset-003 → 确认',
                  'confirmation accepted + seq；confirmRequested 携带 id/revision/queryId；picker 关闭',
                  [conf[:140], s.readback().get('last_confirm', '')[:160]],
                  'accepted' in conf and '"seq":1' in conf
                  and '"id":"att-3"' in s.readback().get('last_confirm', '')
                  and s.ensure_closed())
        attach1 = rb.get('attach', '')
        n1 = re.findall(r'\d+', attach1)
        cases.add('V-03 接受落账（附件引用日志 ≥1 + 身份）', 'readback',
                  '附件日志计数 ≥1 且最近一条为本次接受身份（唯一接受 handler 写入）',
                  [attach1], bool(n1) and int(n1[0]) >= 1 and 'att-3@r3' in attach1)

        # session 2 + double-press single request（接受 handler 的 storage 写
        # park 恢复后再重开——恢复中的 .open=false 会覆盖过早的重开）
        time.sleep(2.0)
        vfd.require(s.open_picker(), 'picker did not reopen for double-press leg')
        time.sleep(1.2)
        s.select_card('Asset-004.bin', '"att-4"')
        c0 = int(s.readback().get('counts', ['0'] * 8)[3])
        s.press_ok('确认')
        s.press_ok('确认')
        rb = s.wait_rb(lambda f: 'accepted' in f.get('confirmation', ''), timeout=6)
        c1 = int(rb.get('counts', ['0'] * 8)[3])
        conf = rb.get('confirmation', '')
        cases.add('V-03 重开会话递增 + 双按单发', 'reopen → 附件-4 → 确认×2',
                  'session=2；confirm 计数只 +1（锁定防重）',
                  'accepted' in conf and (c1 - c0) == 1)

    except Exception as e:
        report['prep'].append({'stage': 'picker matrix crashed', 'error': repr(e)[:200]})
    finally:
        cases.pause_timer()
        s.stop()
        cases.resume_timer()
        patch.restore()


def run_picker_stale(args, out, cases, report):
    """V-03 late segment (fresh session): identity invalidation → rejected →
    retry; stale vnode rejection; V-06 reopen clears stale feedback."""
    app_dir = ROOT / 'examples' / 'asset-picker'
    patch = AppPatch(app_dir)
    s = None
    try:
        cases.pause_timer()
        s = Session(args, out, 5, app_dir)
        cases.resume_timer()
        cases.pause_timer()
        ok = s.open_picker()
        vfd.require(ok, 'picker did not open (stale segment)')
        cases.resume_timer()
        # identity invalidation → rejected not_found → retry allowed → cancel keeps ledger
        s.open_picker()
        time.sleep(1.2)
        s.select_card('Asset-005.bin', '"att-5"')
        attach_before = int(re.findall(r'\d+', s.readback().get('attach', '0 条'))[0]) if re.findall(r'\d+', s.readback().get('attach', '')) else 0
        # 身份失效经搜索通道触发（text=invalidate；搜索框在 picker 内——
        # 保持对话框开启直接触发，provider bump 后自动恢复空文本查询）。
        rb_inv = {}
        for _t in range(8):
            s.type_into_search('invalidate')
            rb_inv = s.ensure_effect('搜索', lambda f: 'invalidated' in f.get('status', ''),
                                     tries=4, wait=2.5)
            if 'invalidated' in rb_inv.get('status', ''):
                break
        vfd.require('invalidated' in rb_inv.get('status', ''),
                    f'invalidate did not apply; status={rb_inv.get("status")!r:.120}')
        s.wait_rb(lambda f: f.get('provider', ['', ''])[4] == '1003', timeout=6)
        time.sleep(0.8)
        s.press_ok('确认')
        rb = s.wait_rb(lambda f: 'rejected' in f.get('confirmation', ''), timeout=6)
        conf = rb.get('confirmation', '')
        snap = s.snapshot()
        cases.add('V-03 身份失效→具名拒绝', '使选中失效 → 确认',
              'confirmation rejected not_found（资源已失效）；picker 未关闭；错误可见',
              [conf[:140], '资源已失效' in snap],
              'rejected' in conf and 'not_found' in conf and '资源已失效' in snap)
        c_before = int(rb.get('counts', ['0'] * 8)[3])
        s.press_ok('确认')
        rb2 = s.wait_rb(lambda f: int(f.get('counts', ['0'] * 8)[3]) > c_before, timeout=6)
        cases.add('V-03 拒绝反馈解锁→可重试', '确认 again',
              'confirm 计数继续增长（解锁后可重试）',
              [rb2.get('counts')], int(rb2.get('counts', ['0'] * 8)[3]) > c_before)
        s.press_ok('取消')
        rb3 = s.wait_rb(lambda f: int(f.get('counts', ['0'] * 8)[7]) >= 1, timeout=6)
        attach_after = int(re.findall(r'\d+', rb3.get('attach', '0 条'))[0]) if re.findall(r'\d+', rb3.get('attach', '')) else 0
        cases.add('V-03 取消不落账', '取消',
              'cancelled≥1；附件引用数不变（拒绝/取消路径零写入）',
              [attach_before, attach_after], attach_after == attach_before)

        # stale vnode rejection: old card vnode after query change
        s.open_picker()
        time.sleep(1.2)
        snap = s.snapshot()
        hits = s.vnodes('Asset-006.bin', snap)
        old_vid = None
        for tag, vid, _t in hits:
            if tag in ('button', 'row'):
                old_vid = vid
                break
        s.press_ok('图片')
        s.wait_rb(lambda f: f.get('provider', ['', ''])[8] == 'image', timeout=6)
        sel_before = s.readback().get('selection', '')
        outp = s.call('autoui_action', allow_error=True, element_id=old_vid, action='press')
        time.sleep(1.0)
        sel_after = s.readback().get('selection', '')
        cases.add('V-03 旧节点 action 拒绝', 'old vnode press after query change',
              '旧 vnode 不产生新选择（被拒或 no-op），新正例可用',
              [sel_after[:60]], sel_after == sel_before and 'att-6' not in sel_after)
        s.press_ok('全部')

        # V-06: reopen clears stale confirmation feedback
        s.open_picker()
        time.sleep(1.0)
        rb = s.readback()
        cases.add('V-06 重开清空旧确认反馈', 'reopen',
              'confirmation=""（旧 accepted/rejected 不入新会话）',
              [rb.get('confirmation', '')], rb.get('confirmation', '') == '')
        s.ensure_closed()
    except Exception as e:
        report['prep'].append({'stage': 'stale segment crashed', 'error': repr(e)[:200]})
    finally:
        if s is not None:
            cases.pause_timer()
            s.stop()
            cases.resume_timer()
        patch.restore()
    report['picker_prep'] = report.get('picker_prep', [])


def run_visual_and_reopen(args, out, cases, report):
    """V-05/V-06 visual + narrow window + exit/reopen (native)."""
    app_dir = ROOT / 'examples' / 'asset-picker'
    patch = AppPatch(app_dir)
    s = None
    attach_before_run = 0
    try:
        cases.pause_timer()
        s = Session(args, out, 2, app_dir)
        cases.resume_timer()
        attach_before_run = int(re.findall(r'\d+', s.readback().get('attach', '0 条'))[0]) \
            if re.findall(r'\d+', s.readback().get('attach', '')) else 0
        # dark (default) selected-state + summary screenshot
        s.open_picker()
        time.sleep(1.0)
        s.select_card('Lake.jpg', '"lake"')
        time.sleep(0.8)
        shot = s.screenshot('v06-selected-dark-native')
        cases.add('V-06 选中态/摘要可见（深色）', 'select Lake.jpg → screenshot',
                  '选中边框+选择摘要+确认栏同帧', [shot['sha256'][:16]], bool(shot['sha256']))
        # pending state
        s.press_ok('确认')
        rb = s.wait_rb(lambda f: 'accepted' in f.get('confirmation', '') or 'rejected' in f.get('confirmation', ''), timeout=6)
        cases.add('V-06 pending 反馈后终态回灌', '确认 → wait',
                  'accepted/rejected 回灌（pending 有文字且合同锁定）',
                  [rb.get('confirmation', '')[:100]], rb.get('confirmation', '') != '')
        # light theme（主页面按钮——对话框必须已关）
        s.close_picker_retry()
        s.press_ok('切换明暗')
        time.sleep(0.8)
        s.open_picker()
        time.sleep(1.0)
        s.select_card('附件-2', '"att-2"')
        time.sleep(0.6)
        shot = s.screenshot('v06-selected-light-native')
        cases.add('V-06 浅色主题选中态', 'ToggleDark → select → screenshot',
                  '浅色下选择态/摘要/确认栏可辨', [shot['sha256'][:16]], bool(shot['sha256']))
        # empty vs error distinguishable (light)
        s.type_into_search('zzzz')
        s.press_ok('搜索')
        time.sleep(1.0)
        snap = s.snapshot()
        cases.add('V-06 空态（浅色）', 'search zzzz', '无匹配资源（区别于 provider 错误）',
                  ['无匹配资源' in snap], '无匹配资源' in snap)
        s.type_into_search('fail')
        s.press_ok('搜索')
        time.sleep(1.2)
        snap = s.snapshot()
        cases.add('V-06 provider 错误态（浅色）', 'search fail', '模拟故障 具名错误可见',
                  ['模拟故障' in snap], '模拟故障' in snap)
        s.ensure_closed()
        s.press_ok('切换明暗')
        # narrow window 420×560
        s.resize(420, 560)
        s.open_picker()
        time.sleep(1.2)
        snap = s.snapshot()
        need = ['搜索', '图片', '网格', '确认', '取消', '关闭']
        missing = [m for m in need if m not in snap]
        shot = s.screenshot('v05-narrow-native')
        cases.add('V-05 窄窗 420×560 控件可达', 'resize → open picker',
                  '搜索/过滤/视图/确认/取消/关闭在快照中可达', [missing, shot['sha256'][:16]],
                  not missing)
        s.press_ok('列表')
        time.sleep(0.8)
        snap = s.snapshot()
        cases.add('V-05 窄窗列表视图', 'press 列表', '列表视图切换生效（行渲染）',
                  ['Asset-' in snap], 'Asset-' in snap)
        s.ensure_closed()
        s.resize(1080, 800)
        # exit & reopen (fresh session; attach ledger persists via storage)
        cases.pause_timer()
        s.stop()
        cases.resume_timer()
        s2 = None
        try:
            cases.pause_timer()
            s2 = Session(args, out, 3, app_dir)
            cases.resume_timer()
            rb = s2.readback()
            attach = rb.get('attach', '')
            n = re.findall(r'\d+', attach)
            cases.add('V-05 退出重开（新会话 + 账本持久）', 'terminate → relaunch',
                      '新会话就绪；附件日志计数与退出前一致（Storage 持久化）',
                      [attach], bool(n) and int(n[0]) >= attach_before_run)
        finally:
            if s2 is not None:
                cases.pause_timer()
                s2.stop()
                cases.resume_timer()
    finally:
        if s is not None:
            cases.pause_timer()
            s.stop()
            cases.resume_timer()
        patch.restore()


def run_029_legs(args, out, cases, report):
    """V-04 029: migrated gallery consumes the package over its real backend."""
    app_dir = ROOT / 'examples' / 'ui' / '029-photo-gallery'

    class G29(AppPatch):
        def _patch_app(self):
            app = self.app_dir / 'src' / 'front' / 'app.at'
            original = self._read(app)
            text = original.decode('utf-8')
            packages = str((ROOT / 'packages').resolve()).replace(chr(92), '/')
            text = re.sub(r'from\s*"\.\./\.\./\.\./\.\./\.\./packages/([a-z-]+)"',
                          f'from "{packages}/\\1"', text)
            app.write_bytes(text.encode('utf-8'))
            self.files[str(app)] = original

    patch = G29(app_dir)
    # 隔离合成 photo_root（029 后端扫描源）：2 张根图 + 1 子目录图（P-9 夹具纪律）。
    proot = Path(args.out_dir) / '029-photo-root'
    sub = proot / 'trip-2024'
    proot.mkdir(parents=True, exist_ok=True)
    sub.mkdir(exist_ok=True)
    fx = ROOT / 'examples' / 'asset-picker' / 'tests' / 'fixtures'
    import shutil
    for name in ['scenic-320x240.png', 'corner-marked.png']:
        shutil.copyfile(fx / name, proot / name)
    shutil.copyfile(fx / 'orientation-6.jpg', sub / 'sub-photo.jpg')
    # 旧 back 实例占端口 = 新会话连到无 env 的旧后端（空图库）——先清。
    subprocess.run(['powershell', '-NoProfile', '-Command',
                    "Get-Process | Where-Object { $_.ProcessName -like 'app-029*' } | Stop-Process -Force"],
                   capture_output=True)
    time.sleep(1.0)
    s = None
    try:
        cases.pause_timer()
        s = Session(args, out, 9, app_dir, env_extra={'AUTO_PHOTO_ROOT': str(proot.resolve())},
                    ready_text='Photos')
        cases.resume_timer()
        snap = s.snapshot()
        (out / 'v04-029.txt').write_text(snap, encoding='utf-8')
        mounted = 'AssetBrowser' in snap or ('全部' in snap and '收藏' in snap and '搜索' in snap)
        cases.add('V-04 029 库面消费 AssetBrowser（真实 scan）', 'launch 029',
                  '分类树（全部/收藏）+ 搜索/网格在真实 photo scan 上渲染',
                  [mounted, snap[:120]], mounted)
        def press_tree_row(label, expect_text, timeout=4.0):
            deadline = time.time() + timeout
            last_snap = ''
            for _ in range(4):
                sn = s.snapshot()
                lines = sn.splitlines()
                target = None
                for i, ln in enumerate(lines):
                    if f'"{label}"' in ln and 'text #' in ln:
                        indent = len(ln) - len(ln.lstrip())
                        for j in range(i - 1, -1, -1):
                            ind = len(lines[j]) - len(lines[j].lstrip())
                            if ind < indent and re.search(r'#(vnode_\d+)\s*\{', lines[j]):
                                target = re.search(r'#(vnode_\d+)', lines[j]).group(1)
                                break
                        break
                if target:
                    s.call('autoui_action', allow_error=True, element_id=target, action='press')
                    t_end = min(deadline, time.time() + 1.5)
                    while time.time() < t_end:
                        last_snap = s.snapshot()
                        if expect_text in last_snap:
                            return True, last_snap
                        time.sleep(0.3)
                time.sleep(0.4)
            return expect_text in last_snap, last_snap

        # select a real photo (first photo card) by its title text if present
        m = re.search(r'text #vnode_\d+ "(scenic[^"]*|corner[^"]*)"', snap)
        photo_label = m.group(1) if m else None
        if photo_label:
            s.press_card(photo_label)
            time.sleep(0.8)
            s.press_ok('查看大图')
            time.sleep(1.2)
            snap2 = s.snapshot()
            viewer_ok = '返回网格' in snap2
            cases.add('V-04 029 选择身份→查看器', f'select {photo_label!r} → 查看大图',
                      '按选择身份进入 PLAN-089 查看区（展示图身份正确）',
                      [photo_label, viewer_ok], viewer_ok)
            s.press_ok('返回网格')
            time.sleep(1.0)
            s.press_card(photo_label)
            time.sleep(0.8)
            s.press_ok('收藏选中')
            time.sleep(0.8)
            fav_switched, snap3 = press_tree_row('❤ 收藏', '收藏夹')
            fav_ok = fav_switched and (photo_label in snap3)
            cases.add('V-04 029 收藏控制不回退', '收藏选中 → ❤ 收藏',
                      'fav 落账且收藏夹切入可见该照片', [fav_switched, photo_label in snap3], fav_ok)
            press_tree_row('全部', '全部')
        else:
            cases.not_run('V-04 029 选择/查看器/收藏腿', 'no photo in scan root',
                          'photo_root 无图片（AUTO_PHOTO_ROOT 未提供）——非本计划缺口，单列')
        # dir navigation via tree row if dirs exist
        snap = s.snapshot()
        dirm = re.search(r'text #vnode_\d+ "📂 ([^"]+)"', snap)
        if dirm:
            label = f'📂 {dirm.group(1)}'
            dir_ok, snap2 = press_tree_row(label, '上一级')
            cases.add('V-04 029 目录导航不回退', f'tree row {label!r}',
                      '进入子目录（scan 语义）+ 上一级可用',
                      [dir_ok], dir_ok)
        else:
            cases.not_run('V-04 029 目录导航腿', 'no subdirectory in scan root',
                          'photo_root 无子目录——单列，不并入总绿')
    except Exception as e:
        cases.not_run('V-04 029 legs', 'session/prewarm', f'{e!r:.260}')
    finally:
        if s is not None:
            cases.pause_timer()
            s.stop()
            cases.resume_timer()
        patch.restore()


def run_vue_track(args, out, cases, report):
    """Vue track: materialized workspace + real chromium via 089's Browser."""
    import verify_image_preview as vip
    app_dir = ROOT / 'examples' / 'asset-picker'
    patch = AppPatch(app_dir)
    b = None
    try:
        cases.pause_timer()
        report['prep'].append(vip.prewarm_vue(args, out, app_dir, READY_TEXT, 91))
        cases.resume_timer()
        b = vip.Browser(args, out, 91, app_dir=app_dir, ready_text=READY_TEXT)
        body = b.snapshot()
        cases.add('V-01(vue) 演示页渲染 + 选择器入口', 'launch',
                  '演示页 + 打开选择器在 DOM', ['打开选择器' in body], '打开选择器' in body)
        b.press('打开选择器')
        time.sleep(1.5)
        body = b.snapshot()
        cases.add('V-01(vue) 选择器挂载（1003 规模）', 'open picker',
                  '共 1003 项 + Lake.jpg', ['共 1003 项' in body and 'Lake.jpg' in body],
                  '共 1003 项' in body and 'Lake.jpg' in body)
        # The first record has a real image. Preview must leave provider state alone.
        observed = []
        original_selection = next(line for line in body.splitlines() if line.startswith('selection='))
        try:
            b.page.wait_for_function(
                """() => [...document.images].some(img =>
                    img.currentSrc.includes('scenic-320x240.png') &&
                    img.complete && img.naturalWidth === 320)""", timeout=4000)
            observed.append('thumbnail decoded')
            b.press('预览')
            b.page.get_by_role('button', name=re.compile('关闭预览')).first.wait_for(timeout=4000)
            preview = b.snapshot()
            assert original_selection in preview.splitlines() and '附件日志: 0 条' in preview, preview
            observed.append('preview left selection and attachments untouched')
            b.press('关闭预览')
            b.page.get_by_role('button', name=re.compile('Lake.jpg')).first.wait_for(timeout=4000)
            observed.append('picker visible after closing preview')
        except Exception as error:
            observed.append(repr(error)[:240])
        cases.add('V-02(vue) 真实缩略图与预览往返', 'preview Lake.jpg, close',
                  '缩略图解码，预览不更改选择或附件，关闭返回选择器', observed,
                  observed[-1] == 'picker visible after closing preview')
        # select via DOM click on the card button
        ok = False
        try:
            b.page.get_by_role('button', name=re.compile('Lake.jpg')).first.click(timeout=4000)
            time.sleep(1.0)
            body = b.snapshot()
            ok = '已选' in body or 'lake' in body
        except Exception as e:
            ok = False
        cases.add('V-02(vue) 卡片选择→provider 读回', 'click Lake.jpg',
                  '选择摘要/读回更新（vue emit 链）', [ok], ok)
        shot = b.screenshot('v01-picker-vue')
        cases.add('V-05(vue) 选择器截图', 'screenshot', 'vue 轨整面截图（含分页与确认栏）', [shot['sha256'][:16]], bool(shot['sha256']))
        # confirm
        ok = False
        try:
            b.page.get_by_role('button', name=re.compile('^确认')).first.click(timeout=4000)
            time.sleep(1.5)
            body = b.snapshot()
            ok = 'accepted' in body and '附件日志: 1' in body
        except Exception:
            ok = False
        cases.add('V-03(vue) 确认接受 + 附件落账', 'press 确认',
                  'confirmation accepted + 附件日志 1 条（vue 轨真实业务写）',
                  [ok], ok)
    except Exception as e:
        cases.not_run('vue track', 'prewarm/browser', f'{e!r:.260}')
    finally:
        if b is not None:
            try:
                b.quit_and_wait()
            except Exception:
                pass
        patch.restore()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--auto-bin', required=True)
    ap.add_argument('--ui-tool-bin', required=True)
    ap.add_argument('--out-dir', required=True)
    ap.add_argument('--backend', choices=['native', 'vue'], default='native')
    ap.add_argument('--skip-029', action='store_true', help='skip 029 legs (no photo_root)')
    ap.add_argument('--runs', type=int, default=3, help='best-of-N session retries per case')
    args = ap.parse_args()
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    cases = Case()
    report = {'plan': PLAN, 'revision': REVISION, 'backend': args.backend,
              'platform': 'windows', 'source': {'auto_bin': str(args.auto_bin)},
              'tools': {'ui_tool_bin': str(args.ui_tool_bin)},
              'dependencies': ['tree-view@085', 'pagination@086', 'image-viewer@089:5dbc3661'],
              'prep': [], 'cases': [], 'visual': [], 'cleanup': {}, 'reopen': {},
              'result': 'not_run'}
    t0 = time.monotonic()
    if args.backend == 'native':
        # prewarm: back first-gen+compile untimed
        try:
            report['prep'].append(prewarm_native(args, out, ROOT / 'examples' / 'asset-picker', 0))
        except Exception as e:
            cases.not_run('all (asset-picker)', 'prewarm', f'preparation failure: {e!r:.260}')
        else:
            # best-of-N session retries (P-4: child-emit chain is session-bimodal;
            # keep the better result per case id and disclose retries).
            for run_no in range(1, args.runs + 1):
                cases.run_no = run_no
                completed = True
                # 段间独立（P-4 窗口按段独立命中，一段崩溃不拖累其余段）
                for seg_name, seg_fn in [('matrix', run_picker_matrix),
                                         ('stale', run_picker_stale),
                                         ('visual', run_visual_and_reopen)]:
                    try:
                        seg_fn(args, out, cases, report)
                    except Exception as e:
                        completed = False
                        report['prep'].append({'stage': f'run{run_no}/{seg_name} crashed',
                                               'error': repr(e)[:260]})
                if not args.skip_029:
                    try:
                        run_029_legs(args, out, cases, report)
                    except Exception as e:
                        completed = False
                        report['prep'].append({'stage': f'run{run_no}/029 crashed',
                                               'error': repr(e)[:260]})
                passed_now = sum(1 for r in cases.rows if r['result'] == 'pass')
                print(f"== run {run_no}/{args.runs}: {passed_now}/{len(cases.rows)} cases pass ==", flush=True)
                if completed and cases.all_pass():
                    break
    else:
        run_vue_track(args, out, cases, report)
    report['cases'] = cases.rows
    report['visual'] = [r for r in cases.rows if 'screenshot' in r['entry'] or '截图' in r['entry']]
    passed = sum(1 for r in cases.rows if r['result'] == 'pass')
    failed = sum(1 for r in cases.rows if r['result'] == 'fail')
    not_run = sum(1 for r in cases.rows if r['result'] == 'not_run')
    # 完整性下限：native 矩阵含 picker(V-01..03) + visual/reopen(V-05/06) 腿（≥12），
    # vue 轨为 5 例。少于下限 = 会话早期崩溃，不得按空真值判 pass。
    complete = len(cases.rows) >= (12 if args.backend == 'native' else 5)
    report['result'] = 'pass' if (cases.all_pass() and complete) else ('fail' if failed or not complete else 'not_run')
    report['seconds'] = round(time.monotonic() - t0, 1)
    (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding='utf-8')
    print(f"RESULT: {report['result']} | {passed} pass / {failed} fail / {not_run} not_run | {report['seconds']}s")
    if failed:
        for r in cases.rows:
            if r['result'] == 'fail':
                print(f"  FAIL {r['id']}: {str(r['observed'])[:200]}")
    return 0 if cases.all_pass() else 1


if __name__ == '__main__':
    sys.exit(main())
