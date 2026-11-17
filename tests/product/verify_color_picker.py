#!/usr/bin/env python3
"""PLAN-088 product gate — ColorPicker/ColorDialog + Paint consumption.

Drives the real generated apps through MCP (native) or Playwright (vue):
  V-01/06 known-color math (hue/surface/alpha pen picks, HSV edge, palette)
  V-02 preview/commit-once gestures; external reset revokes drafts
  V-03 two instances; strict business-reject keeps formal value; dialog
       draft/confirm-once/cancel-zero
  V-04 Paint consumption: picker commit -> pixel; save/load round-trip;
       legacy class-string adaptation (fixture-seeded); undo restores
  V-05 dual screenshots (default + 420x560 narrow), quit/reopen (native)

Interface: --auto-bin --ui-tool-bin --out-dir --backend native|vue
The legacy fixture case needs AUTOUI_TEST_FIXTURES=1 (set for native).
Exit 0 only when every case for the requested backend passes. Linux tracks
run in the merge-owned regression batch (not_run here is recorded, not
silently dropped).
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import verify_file_dialog as vfd  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
DEMO_APP = REPO / 'examples' / 'color-picker'
PAINT_APP = REPO / 'examples' / 'ui' / '031-paint'
SEP = '\u001f'


def pen_value(handlers, points):
    """Widget␟Start␟Move␟End␟points（5 段；scoped 解析只依赖 element_id，
    widget 段保留 Plan 563 形态占位）。"""
    return SEP.join(['ColorPicker', handlers[0], handlers[1], handlers[2],
                     ';'.join(f'{x},{y}' for x, y in points)])


def bindable_port(preferred):
    """087 同款：自 preferred 起找可绑定端口。"""
    import socket
    for port in range(preferred, preferred + 400):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            try:
                sock.bind(('127.0.0.1', port))
            except OSError:
                continue
        return port
    raise RuntimeError('no bindable port found')


class Session:
    """Native `auto run -r vm` window driven through MCP. Per-plan fixture:
    NOT the FileDialog session — no FileDialog env, per-app ready text."""

    def __init__(self, args, out, index, app_dir, ready_text, name='session'):
        self.args = args
        self.out = out
        self.name = name
        self.ready_text = ready_text
        self.app_dir = Path(app_dir)
        self.port = bindable_port(vfd.free_port())
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port),
                   AUTO_UI_TOOL_BIN=str(args.ui_tool_bin),
                   AUTOUI_TEST_FIXTURES='1')
        self.log = open(out / f'{name}-{index}.log', 'wb')
        self.process = subprocess.Popen(
            [str(args.auto_bin), 'run', '-r', 'vm'], cwd=str(self.app_dir), env=env,
            stdout=self.log, stderr=subprocess.STDOUT,
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
                if ready_text in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.stop()
            raise RuntimeError(f'{name} never rendered; last: {last[:300]}')
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)
        vfd.require(self.windows, f'{name}: no owned native window')

    def stop(self):
        try:
            self.process.terminate()
            self.process.wait(timeout=10)
        except Exception:
            try:
                self.process.kill()
            except Exception:
                pass

    def call(self, tool, allow_error=False, latency=False, **argv):
        started = time.monotonic()
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argv}}).encode()
        req = urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body,
                                     headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req, timeout=12) as r:
            out = json.loads(r.read().decode())
        elapsed = time.monotonic() - started
        error = 'error' in out or out.get('result', {}).get('isError', False)
        text = '\n'.join(c.get('text', '') for c in out.get('result', {}).get('content', [])
                         if c.get('type') == 'text')
        vfd.require(allow_error or not error, text)
        if latency:
            vfd.require(elapsed < 1, f'ordinary action budget exceeded: {tool} {elapsed:.3f}s')
        return (error, text) if allow_error else text

    def snapshot(self):
        return self.call('autoui_snapshot', include_styles=False)

    def canvases(self):
        return re.findall(r'canvas #vnode_(\d+)', self.snapshot())

    def press_button(self, label):
        s = self.snapshot()
        m = re.search(r'button #vnode_(\d+) "[^"]*' + re.escape(label), s)
        vfd.require(m, f'button {label!r} not found')
        self.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='press', latency=True)

    def pen(self, canvas_idx, points, handlers=('SurfDown', 'SurfMove', 'SurfUp')):
        cans = self.canvases()
        vfd.require(len(cans) > canvas_idx, f'canvas #{canvas_idx} not found ({len(cans)})')
        (err, text) = self.call('autoui_action', allow_error=True, latency=True,
                                element_id=f'vnode_{cans[canvas_idx]}', action='pen',
                                value=pen_value(handlers, points))
        vfd.require(not err, f'pen failed: {text[:200]}')

    def hit(self, canvas_idx, node_id):
        cans = self.canvases()
        vfd.require(len(cans) > canvas_idx, f'canvas #{canvas_idx} not found ({len(cans)})')
        (err, text) = self.call('autoui_action', allow_error=True, latency=True,
                                element_id=f'vnode_{cans[canvas_idx]}', action='press',
                                value=node_id)
        vfd.require(not err, f'hit press failed: {text[:200]}')

    def ink_text(self):
        m = re.search(r'当前画笔 [^"]*', self.snapshot())
        vfd.require(m, 'ink text not found')
        return m.group(0)

    def screenshot(self, name):
        shot = ''
        for _ in range(8):
            (_se, shot) = self.call('autoui_screenshot', allow_error=True)
            if re.search(r'(\S+\.(?:png|jpg))', shot):
                break
            time.sleep(1.0)
        m = re.search(r'(\S+\.(?:png|jpg))', shot)
        vfd.require(m, f'screenshot failed: {shot[:160]}')
        src = Path(m.group(1))
        vfd.require(src.exists(), f'screenshot missing: {src}')
        dst = self.out / name
        dst.write_bytes(src.read_bytes())
        vfd.require(dst.stat().st_size > 5000, f'screenshot too small: {dst}')
        return dst

    def quit(self):
        try:
            s = self.snapshot()
            m = re.search(r'button #vnode_(\d+) "退出"', s)
            if m:
                self.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='press')
                time.sleep(1.0)
        except Exception:
            pass
        self.stop()


def log_lines(s):
    snap = s.snapshot()
    i = snap.find('提交日志')
    block = snap[i:i + 8000] if i >= 0 else ''
    return re.findall(r'^\s*text #vnode_\d+ "(\[\d+\].*)"\s*$', block, re.M)


def wait_log(s, pred, tries=14, wait=0.7):
    for _ in range(tries):
        if pred(log_lines(s)):
            return True
        time.sleep(wait)
    return False


def commits(lines):
    return [l for l in lines if 'ink 接受' in l]


def input_values(s):
    """Current input values in declaration order (HEX, R, G, B[, A[, H,S,V]])."""
    vals = []
    for _ in range(3):
        ids = re.findall(r'input #vnode_(\d+)', s.snapshot())
        vals = []
        for i in ids:
            (_e, t) = s.call('autoui_inspect', allow_error=True, element_id=f'vnode_{i}')
            m = re.search(r'value:\s*(\S*)', t)
            vals.append(m.group(1) if m else '?')
        if vals and all(v != '?' for v in vals):
            return vals
    return vals


def wait_snapshot(s, pred, tries=12, wait=0.4):
    snap = ''
    for _ in range(tries):
        snap = s.snapshot()
        if pred(snap):
            return True
        time.sleep(wait)
    return False


def toggle_checkbox(s, label):
    m = re.search(r'checkbox #vnode_(\d+) "[^"]*' + re.escape(label), s.snapshot())
    vfd.require(m, f'checkbox {label!r} not found')
    s.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='toggle', latency=True)


def hex_type(s, text):
    """Type a HEX draft and verify the input really carries it (retry once)."""
    hex_id = ''
    for _ in range(3):
        ids = re.findall(r'input #vnode_(\d+)', s.snapshot())
        vfd.require(ids, 'hex input missing')
        hex_id = 'vnode_' + ids[0]
        s.call('autoui_type', element_id=hex_id, text=text)
        time.sleep(0.4)
        (_e, t) = s.call('autoui_inspect', allow_error=True, element_id=hex_id)
        m = re.search(r'value:\s*(\S*)', t)
        if m and m.group(1).upper() == text.upper():
            return hex_id
    return hex_id


def hex_commit(s, text, expected):
    """Type+commit a HEX draft; retry until the formal ink text shows the value."""
    for _ in range(3):
        hex_id = hex_type(s, text)
        for _ in range(2):
            s.call('autoui_action', allow_error=True, element_id=hex_id,
                   action='submit', latency=True)
            s.call('autoui_keyboard', key='Enter')
            time.sleep(0.5)
        for _ in range(10):
            if expected.upper() in s.ink_text().upper():
                return True
            time.sleep(0.5)
    return False


def hex_submit(s, text):
    """Type a HEX draft into the first input and commit with submit + Enter."""
    hex_id = hex_type(s, text)
    for _ in range(2):
        s.call('autoui_action', allow_error=True, element_id=hex_id, action='submit', latency=True)
        s.call('autoui_keyboard', key='Enter')
        time.sleep(0.5)


# ── demo cases (native) ──

def case_known_colors(s, report):
    """V-01/06: fresh session; hue pen -> surface -> alpha -> hue edge."""
    vfd.require(s.ink_text() == '当前画笔 #3974E6CC', f'initial ink wrong: {s.ink_text()}')
    # canvas order: [0]=surf [1]=hue [2]=alpha [3]=swatch [4]=palette
    # hue bar (8,6) -> h=15; surface (96,95) -> s=50 v=50; alpha baseline CC
    s.pen(1, [(8, 6)])
    s.pen(0, [(96, 95)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #805040CC' in x for x in l)),
                f'hue+surface commit missing: {log_lines(s)[-3:]}')
    # alpha bar (96,6) -> a=127
    s.pen(2, [(96, 6)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8050407F' in x for x in l)),
                f'alpha commit missing: {log_lines(s)[-3:]}')
    # hue right edge (192,6) -> 360 -> normalized 0; alpha stays 7F
    s.pen(1, [(192, 6)])
    s.pen(0, [(96, 95)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8040407F' in x for x in l)),
                f'hue-edge commit missing: {log_lines(s)[-3:]}')
    # F-16: corner (x=192,y=0) must reach S=100,V=100 with the hue carried
    # through the gesture (H=0 here) -> #FF0000 with the current alpha 7F
    s.pen(0, [(192, 0)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #FF00007F' in x for x in l)),
                f'surface corner commit missing: {log_lines(s)[-3:]}')
    # r2 rule: a gray draft keeps the last valid hue (S=0 drag then back)
    s.pen(1, [(8, 6)])      # H=15, S/V from #8040407F (S=50,V=50)
    s.pen(0, [(0, 95)])     # S=0 -> gray #8080807F
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8080807F' in x for x in l)),
                f'gray draft commit missing: {log_lines(s)[-3:]}')
    s.pen(0, [(96, 95)])    # S/V back; H must still be 15 -> #8050407F
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8050407F' in x for x in l)),
                f'held-hue commit missing: {log_lines(s)[-3:]}')
    report['cases'].append('V-01/06 known-colors pass')


def case_gestures(s, report):
    """V-02: one gesture = one commit; preview carries surface source; external
    reset revokes drafts."""
    time.sleep(1.5)
    before = len(log_lines(s))
    before_commits = len(commits(log_lines(s)))
    s.pen(0, [(30, 30), (60, 120)])
    vfd.require(wait_log(s, lambda l: len(commits(l)) > before_commits),
                'gesture did not commit')
    lines = log_lines(s)
    new_commits = [x for x in lines[before:] if 'ink 接受' in x]
    vfd.require(len(new_commits) == 1, f'one gesture must commit once, got {new_commits}')
    new_previews = [x for x in lines[before:] if 'ink 预览' in x]
    vfd.require(len(new_previews) >= 1, 'gesture had no preview')
    vfd.require('source":"surface"' in new_previews[-1], f'preview source wrong: {new_previews[-1]}')
    vfd.require('"baseRevision":' in new_previews[-1], 'preview missing baseRevision')
    # external reset: value/revision advance; component Tick re-baselines
    s.press_button('重置画笔')
    vfd.require(wait_log(s, lambda l: any('外部置值 ink=#3974E6CC' in x for x in l)),
                'external reset not logged')
    time.sleep(1.0)
    vfd.require(s.ink_text() == '当前画笔 #3974E6CC', f'external reset ink: {s.ink_text()}')
    report['cases'].append('V-02 gestures + external reset pass')


def case_channel(s, report):
    """V-01 (r3): builtin-input channel entry — precise value, invalid reject."""
    snap = s.snapshot()
    inputs = re.findall(r'input #vnode_(\d+)', snap)
    vfd.require(len(inputs) >= 5, f'channel inputs missing: {len(inputs)}')
    r_id = 'vnode_' + inputs[1]  # inputs[0] = HEX; inputs[1] = R
    committed = False
    for _ in range(3):
        s.call('autoui_type', element_id=r_id, text='64')
        time.sleep(0.5)
        s.call('autoui_action', element_id=r_id, action='submit', latency=True)
        if wait_log(s, lambda l: any('ink 接受 #4074E6CC' in x for x in l), tries=5):
            committed = True
            break
    vfd.require(committed, f'channel commit missing: {log_lines(s)[-3:]}')
    invalid_ok = False
    for _ in range(3):
        s.call('autoui_type', element_id=r_id, text='abc')
        time.sleep(0.5)
        s.call('autoui_action', element_id=r_id, action='submit')
        time.sleep(0.6)
        snap2 = s.snapshot()
        if 'invalid_channel' in snap2:
            invalid_ok = True
            break
    vfd.require(invalid_ok, 'invalid channel error missing')
    vfd.require(wait_log(s, lambda l: any('ink 错误' in x and 'invalid_channel' in x for x in l)),
                'invalid channel error event missing')
    vfd.require(s.ink_text() == '当前画笔 #4074E6CC', f'invalid channel changed ink: {s.ink_text()}')
    report['cases'].append('V-01 channel entry pass')


def case_two_instances(s, report):
    """V-03: inline picker palette commits ink; dialog label independent."""
    s.hit(len(s.canvases()) - 1, 'p0')
    vfd.require(wait_log(s, lambda l: any('ink 接受 #EF4444FF' in x for x in l)),
                f'inline palette p0 commit missing: {log_lines(s)[-3:]}')
    label = re.search(r'当前标签色 (\S+)', s.snapshot()).group(1)
    vfd.require(label != '#EF4444FF', f'label leaked inline color: {label}')
    report['cases'].append('V-03 two-instance isolation pass')


def case_business_reject(s, report):
    """V-03: strict mode business-reject keeps formal value."""
    ink0 = s.ink_text()

    def toggle_strict():
        s.press_button('业务拒绝：关')

    toggle_strict()
    wait_snapshot(s, lambda snap: '业务拒绝：开' in snap)
    s.hit(len(s.canvases()) - 1, 'p1')
    vfd.require(wait_log(s, lambda l: any('ink 拒绝' in x for x in l)),
                f'strict reject not logged: {log_lines(s)[-3:]}')
    time.sleep(1.0)
    vfd.require(s.ink_text() == ink0, f'business reject changed formal ink: {ink0} -> {s.ink_text()}')
    s.press_button('业务拒绝：开')
    wait_snapshot(s, lambda snap: '业务拒绝：关' in snap)
    report['cases'].append('V-03 business-reject pass')


def case_dialog(s, report):
    """V-02/03: dialog draft/confirm-once/cancel-zero; dialog pen scoping."""
    label0 = re.search(r'当前标签色 (\S+)', s.snapshot()).group(1)
    inline_cans = len(s.canvases())
    vfd.require(inline_cans >= 4, f'inline canvases missing: {inline_cans}')
    s.press_button('打开标签色对话框')
    cans = []
    deadline = time.time() + 10
    while time.time() < deadline:
        cans = s.canvases()
        if len(cans) == inline_cans * 2:
            break
        time.sleep(0.4)
    vfd.require(len(cans) == inline_cans * 2,
                f'dialog canvases missing: {len(cans)} (inline {inline_cans})')
    # native window capture is unavailable while the modal is active (the
    # capture reports a zero-size window); the dialog-open state is asserted
    # structurally here and captured in the vue track (vue-dialog.png).
    vfd.require('取消' in s.snapshot() and '确认' in s.snapshot(), 'dialog footer missing')
    # dialog picker canvases are the last inline_cans: [surf, hue, (alpha), sw, pal]
    base = len(cans) - inline_cans
    s.pen(base + 1, [(8, 6)])
    s.pen(base, [(96, 95)])
    before_cancel = len([x for x in log_lines(s) if 'label 取消' in x])
    s.press_button('取消')
    deadline = time.time() + 8
    while time.time() < deadline:
        if len(s.canvases()) == inline_cans:
            break
        time.sleep(0.4)
    time.sleep(0.5)
    time.sleep(1.0)
    cancels = [x for x in log_lines(s) if 'label 取消' in x]
    vfd.require(len(cancels) - before_cancel == 1,
                f'cancel must emit exactly once, got {cancels[before_cancel:]}')
    label1 = re.search(r'当前标签色 (\S+)', s.snapshot()).group(1)
    vfd.require(label1 == label0, f'cancel changed formal label: {label0} -> {label1}')
    s.press_button('打开标签色对话框')
    deadline = time.time() + 10
    while time.time() < deadline:
        cans = s.canvases()
        if len(cans) == inline_cans * 2:
            break
        time.sleep(0.4)
    vfd.require(len(cans) == inline_cans * 2,
                f'dialog canvases missing on reopen: {len(cans)}')
    base = len(cans) - inline_cans
    s.pen(base + 1, [(8, 6)])
    s.pen(base, [(96, 95)])
    s.press_button('确认')
    vfd.require(wait_log(s, lambda l: any('label 接受' in x for x in l)),
                f'confirm commit missing: {log_lines(s)[-3:]}')
    deadline = time.time() + 8
    while time.time() < deadline:
        if len(s.canvases()) == inline_cans:
            break
        time.sleep(0.4)
    time.sleep(0.5)
    label2 = re.search(r'当前标签色 (\S+)', s.snapshot()).group(1)
    vfd.require(label2 != label0, f'confirm did not change label: {label2}')
    report['cases'].append('V-02/03 dialog draft/confirm/cancel pass')


# ── hex / alpha oracles (native) ──

def case_hex(s, report):
    """V-01/06: short HEX, exact known color + channel readback, same-value
    zero event, invalid text -> named error without commit."""
    s.press_button('重置画笔')
    vfd.require(wait_log(s, lambda l: any('外部置值 ink=#3974E6CC' in x for x in l)),
                'hex: reset missing')
    time.sleep(1.0)
    vfd.require(hex_commit(s, '#F00', '#FF0000FF'),
                f'hex: short-form commit missing: {log_lines(s)[-3:]}')
    vfd.require(hex_commit(s, '#3974E6CC', '#3974E6CC'),
                f'hex: known-color commit missing: {log_lines(s)[-3:]}')
    vals = ['?'] * 5
    for _ in range(16):
        vals = input_values(s)
        if vals[:5] == ['#3974E6CC', '57', '116', '230', '204']:
            break
        time.sleep(0.5)
    vfd.require(vals[:5] == ['#3974E6CC', '57', '116', '230', '204'],
                f'hex: channel readback {vals}')
    before = len(commits(log_lines(s)))
    hex_submit(s, '#3974E6CC')
    time.sleep(1.2)
    vfd.require(len(commits(log_lines(s))) == before, 'hex: same value committed again')
    hex_submit(s, '#GG0000')
    vfd.require(wait_log(s, lambda l: any('ink 错误' in x and 'invalid_color' in x for x in l)),
                'hex: invalid_color error event missing')
    time.sleep(0.6)
    vfd.require('invalid hex color' in s.snapshot(),
                'hex: visible invalid_color label missing (F-13/F-19)')
    vfd.require(len(commits(log_lines(s))) == before, 'hex: invalid text committed')
    vfd.require(s.ink_text() == '当前画笔 #3974E6CC',
                f'hex: invalid text changed ink: {s.ink_text()}')
    report['cases'].append('V-01/06 HEX exact + invalid + same-value pass')


def case_alpha(s, report):
    """V-06: showAlpha=false hides editing but must not drop the existing alpha."""
    s.press_button('重置画笔')
    vfd.require(wait_log(s, lambda l: any('外部置值 ink=#3974E6CC' in x for x in l)),
                'alpha: reset missing')
    time.sleep(1.0)
    s.press_button('隐藏 alpha')
    shown = wait_snapshot(s, lambda snap: len(re.findall(r'input #vnode_', snap)) == 4, tries=20)
    vfd.require(shown, 'alpha: A input still shown after hiding alpha')
    s.pen(0, [(96, 95)])
    vfd.require(wait_log(s, lambda l: any(re.search(r'ink 接受 #[0-9A-F]{6}CC', x) for x in l)),
                f'alpha: hidden alpha dropped the value: {log_lines(s)[-3:]}')
    s.press_button('显示 alpha')
    back = wait_snapshot(s, lambda snap: len(re.findall(r'input #vnode_', snap)) == 5, tries=20)
    vfd.require(back, 'alpha: A input did not come back')
    report['cases'].append('V-06 showAlpha retention pass')


# ── paint consumption (native) ──

def case_narrow(args, out, report):
    """V-05/06: native narrow window (640x560) + light/dark theme; the picker
    stays reachable and the content color is theme-independent."""
    import shutil
    root = out / 'narrow-app'
    if root.exists():
        shutil.rmtree(root)
    (root / 'src' / 'front').mkdir(parents=True)
    for entry in (DEMO_APP / 'src').rglob('*'):
        rel = entry.relative_to(DEMO_APP / 'src')
        if entry.is_dir():
            (root / 'src' / rel).mkdir(parents=True, exist_ok=True)
        else:
            (root / 'src' / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(entry, root / 'src' / rel)
    pac = (DEMO_APP / 'pac.at').read_text(encoding='utf-8')
    # native narrow: the demo stacks its two sections so 640x560 keeps the
    # channel labels/inputs and the dialog footer reachable.
    pac = re.sub(r'window:\s*"\d+x\d+"', 'window: "640x560"', pac)
    dep = (REPO / 'packages' / 'color-picker').resolve().as_posix()
    pac = re.sub(r'path:\s*"[^"]*packages/color-picker"', f'path: "{dep}"', pac)
    (root / 'pac.at').write_text(pac, encoding='utf-8')
    page = root / 'src' / 'front' / 'pages' / 'demo.at'
    if page.exists():
        txt = page.read_text(encoding='utf-8')
        txt = re.sub(r'from "[^"]*packages/color-picker"', f'from "{dep}"', txt)
        page.write_text(txt, encoding='utf-8')
    s = Session(args, out, 7, root, '选色设置表单', name='narrow')
    try:
        snap = s.snapshot()
        deadline = time.time() + 15
        while time.time() < deadline:
            snap = s.snapshot()
            if len(s.canvases()) >= 5:
                break
            time.sleep(0.4)
        vfd.require('当前画笔 #3974E6CC' in snap, 'narrow: picker ink missing')
        vfd.require(len(s.canvases()) >= 5, 'narrow: picker canvases missing')
        vfd.require('RGB/HSV' in snap, 'narrow: channel toggle unreachable')
        s.screenshot('demo-narrow-native.png')
        report['visual'] += ['demo-narrow-native.png']
        s.press_button('切换明暗')
        time.sleep(0.8)
        snap_dark = s.snapshot()
        vfd.require('当前画笔 #3974E6CC' in snap_dark,
                    f'narrow: theme changed content color: {snap_dark[:160]}')
        s.screenshot('demo-native-dark.png')
        report['visual'] += ['demo-native-dark.png']
        s.press_button('切换明暗')
        time.sleep(0.8)
        vfd.require('当前画笔 #3974E6CC' in s.snapshot(),
                    'narrow: theme round-trip changed content color')
        report['cases'].append('V-05 narrow + light/dark pass')
    finally:
        s.stop()


def case_paint(args, out, report):
    """V-04: legacy adaptation, picker commit -> pixel, save/load, undo."""
    paint = Session(args, out, 9, PAINT_APP, 'Paint', name='paint')
    try:
        # legacy sample: seed px with OLD class-string format, Save via fixture
        # trigger, then Load through the app's adaptation path.
        legacy = [' bg-[#ef4444] '] * 256
        (err, text) = paint.call('autoui_fixture', allow_error=True, **{
            'schema_version': 1,
            'state': {'px': legacy, 'saved': '1'},
            'trigger': {'handler': 'Save'},
        })
        vfd.require(not err, f'legacy fixture failed: {text[:200]}')
        time.sleep(1.0)
        paint.press_button('Load')
        time.sleep(1.0)
        st = re.search(r'text #vnode_\d+ "(loaded|no saved canvas|bad save data)"', paint.snapshot())
        vfd.require(st and st.group(1) == 'loaded', f'legacy load failed: {st.group(1) if st else "?"}')
        # legacy load already migrated every cell to #EF4444FF
        vfd.require('px42=#EF4444FF' in paint.snapshot(), 'paint: legacy px42 readback missing')
        # picker palette p7 (#3B82F6FF) then paint cell 42 through canvas onhit
        paint.hit(6, 'p7')
        time.sleep(1.0)
        paint.hit(0, 'px42')
        time.sleep(1.2)
        snap = paint.snapshot()
        vfd.require('px42=#3B82F6FF' in snap, 'paint: painted pixel color mismatch (V-04)')
        vfd.require('painted px42=#3B82F6FF' in snap, 'paint: commit status mismatch (V-04)')
        # V-04 cancel oracle: a picker draft cancelled with Escape changes no
        # document pixel and adds no history entry.
        pids = re.findall(r'input #vnode_(\d+)', paint.snapshot())
        vfd.require(pids, 'paint: picker HEX input missing')
        paint.call('autoui_type', element_id=f'vnode_{pids[0]}', text='#ZZZZZZ')
        time.sleep(0.5)
        paint.call('autoui_keyboard', key='Escape')
        ok = False
        for _ in range(10):
            if 'picker cancelled' in paint.snapshot():
                ok = True
                break
            time.sleep(0.4)
        vfd.require(ok, 'paint: picker cancel path did not run')
        vfd.require('px42=#3B82F6FF' in paint.snapshot(),
                    'paint: cancel changed the document pixel')
        # undo history: Load pushed one snapshot, the paint pushed one. The
        # cancelled picker must not push. So: undo -> legacy red, undo -> the
        # pre-load canvas, undo -> empty history.
        paint.press_button('Undo')
        time.sleep(1.2)
        vfd.require('px42=#EF4444FF' in paint.snapshot(), 'paint: undo did not restore px42')
        paint.press_button('Undo')
        time.sleep(1.2)
        # exactly two pushes exist (Load + the paint); a cancelled picker draft
        # must not add a third, so the next undo reports an empty history.
        paint.press_button('Undo')
        time.sleep(1.2)
        vfd.require('nothing to undo' in paint.snapshot(),
                    'paint: cancel added an unexpected history entry')
        # save/load round-trip in the new format keeps the picked color
        paint.hit(0, 'px42')
        time.sleep(1.2)
        paint.press_button('Save')
        time.sleep(1.2)
        paint.press_button('New')
        time.sleep(1.2)
        paint.press_button('Load')
        st = None
        for _ in range(12):
            time.sleep(0.5)
            st = re.search(r'text #vnode_\d+ "(loaded|no saved canvas|bad save data)"', paint.snapshot())
            if st and st.group(1) == 'loaded':
                break
        vfd.require(st and st.group(1) == 'loaded', f'round-trip load failed')
        vfd.require('px42=#3B82F6FF' in paint.snapshot(), 'paint: save/load lost the pixel color')
        paint.screenshot('paint-native.png')
        report['cases'].append('V-04 paint consumption pass')
    finally:
        paint.stop()


# ── vue track ──

def run_vue(args, out, report):
    from playwright.sync_api import sync_playwright
    env = dict(os.environ, AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
    log = open(out / 'vue-run.log', 'wb')
    proc = subprocess.Popen([str(args.auto_bin), 'run'], cwd=str(DEMO_APP), env=env,
                            stdout=log, stderr=subprocess.STDOUT,
                            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
    url = None
    deadline = time.time() + 150
    ansi = re.compile(r'\[[0-9;]*m')
    while time.time() < deadline:
        text = ansi.sub('', open(log.name, 'r', encoding='utf-8', errors='ignore').read())
        m = re.search(r'Local:\s*(http://127\.0\.0\.1:\d+/)', text)
        if m:
            url = m.group(1)
            break
        if 'Err' in text and 'Local:' not in text and time.time() > deadline - 140:
            pass
        time.sleep(0.5)
    vfd.require(url, 'vite URL never appeared')
    time.sleep(4)
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch()
            page = browser.new_page(viewport={'width': 1100, 'height': 800})
            page.goto(url, timeout=30000)
            time.sleep(4)
            body = page.evaluate('document.body.innerText')
            vfd.require('选色设置表单' in body, 'vue demo text missing')
            vfd.require('#3974E6CC' in body, f'vue initial ink missing: {body[:160]}')
            # real-mouse drag on the SV surface (canvas 0)
            box = page.evaluate('''() => { const c = document.querySelectorAll('canvas')[0];
                const r = c.getBoundingClientRect(); return {x: r.x, y: r.y, w: r.width, h: r.height}; }''')
            page.mouse.move(box['x'] + box['w'] * 0.5, box['y'] + box['h'] * 0.5)
            page.mouse.down()
            for k in range(1, 5):
                page.mouse.move(box['x'] + box['w'] * (0.5 + 0.04 * k),
                                box['y'] + box['h'] * 0.5)
                time.sleep(0.04)
            page.mouse.up()
            time.sleep(1.5)
            body2 = page.evaluate('document.body.innerText')
            vfd.require('ink 预览' in body2, f'vue drag no preview: {body2[:200]}')
            vfd.require('ink 接受' in body2, 'vue drag did not commit')
            # reset to the controlled baseline before the channel-exact check
            page.get_by_role('button', name=re.compile('重置画笔')).click()
            time.sleep(1.2)
            body_reset = page.evaluate('document.body.innerText')
            vfd.require('#3974E6CC' in body_reset, 'vue reset ink missing')
            # r3 channel entry: R field (inputs[1]) -> 64 -> commit #4074E6CC
            page.locator('input').nth(1).fill('64')
            time.sleep(0.4)
            page.keyboard.press('Enter')
            time.sleep(1.5)
            body3 = page.evaluate('document.body.innerText')
            vfd.require('#4074E6CC' in body3, f'vue channel commit missing: {body3[:240]}')
            # vue equivalent of native case_hex: short HEX + exact channel readback
            page.locator('input').nth(0).fill('#F00')
            time.sleep(0.4)
            page.keyboard.press('Enter')
            time.sleep(1.5)
            body_hex = page.evaluate('document.body.innerText')
            vfd.require('#FF0000FF' in body_hex, f'vue short-hex commit missing: {body_hex[:200]}')
            page.locator('input').nth(0).fill('#3974E6CC')
            time.sleep(0.4)
            page.keyboard.press('Enter')
            time.sleep(1.5)
            vals = page.evaluate('Array.from(document.querySelectorAll("input")).map(i=>i.value)')
            vfd.require(vals[:5] == ['#3974E6CC', '57', '116', '230', '204'],
                        f'vue channel readback {vals}')
            # dialog-open state (native capture is zero-size under a modal)
            page.get_by_role('button', name=re.compile('打开标签色对话框')).click()
            time.sleep(1.2)
            body_dlg = page.evaluate('document.body.innerText')
            vfd.require('选择标签颜色' in body_dlg and '确认' in body_dlg,
                        'vue dialog state missing')
            page.screenshot(path=str(out / 'vue-dialog.png'), full_page=True)
            page.get_by_role('button', name=re.compile('取消')).click()
            time.sleep(0.8)
            # dark theme: content color must stay identical
            page.get_by_role('button', name=re.compile('切换明暗')).click()
            time.sleep(1.0)
            body_dark = page.evaluate('document.body.innerText')
            vfd.require('#3974E6CC' in body_dark, 'vue dark theme changed content color')
            page.screenshot(path=str(out / 'vue-dark.png'), full_page=True)
            page.screenshot(path=str(out / 'vue-light.png'), full_page=True)
            page2 = browser.new_page(viewport={'width': 420, 'height': 560})
            page2.goto(url, timeout=30000)
            time.sleep(3)
            page2.screenshot(path=str(out / 'vue-narrow.png'), full_page=True)
            # narrow dialog footer (V-06/AC-06): 取消/确认 reachable at 420x560
            page2.get_by_role('button', name=re.compile('打开标签色对话框')).click()
            time.sleep(1.2)
            body_n = page2.evaluate('document.body.innerText')
            vfd.require('取消' in body_n and '确认' in body_n,
                        'vue narrow dialog footer missing')
            page2.screenshot(path=str(out / 'vue-dialog-narrow.png'), full_page=True)
            browser.close()
        report['cases'].append('V-05 vue drag + screenshots pass')
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--auto-bin', required=True)
    ap.add_argument('--ui-tool-bin', required=True)
    ap.add_argument('--out-dir', required=True)
    ap.add_argument('--backend', choices=['native', 'vue'], required=True)
    args = ap.parse_args()
    args.ui_tool_bin = Path(args.ui_tool_bin)
    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    (out / 'gate-inputs.json').write_text(json.dumps({
        'auto_bin': str(args.auto_bin), 'ui_tool_bin': str(args.ui_tool_bin),
        'demo_app': str(DEMO_APP), 'paint_app': str(PAINT_APP),
    }, indent=2))

    report = {'plan': '088', 'revision': 3, 'backend': args.backend,
              'platform': 'windows' if os.name == 'nt' else 'linux',
              'source': {}, 'tools': {}, 'cases': [], 'visual': [],
              'cleanup': {}, 'reopen': {}, 'result': 'not_run'}
    (args.auto_bin and report['source'].__setitem__('auto', Path(args.auto_bin).name))
    failures = []

    if args.backend == 'native':
        s = Session(args, out, 0, DEMO_APP, '选色设置表单', name='demo')
        try:
            for name, fn in [('known_colors', case_known_colors),
                             ('gestures', case_gestures),
                             ('channel', case_channel),
                             ('hex', case_hex),
                             ('alpha', case_alpha),
                             ('two_instances', case_two_instances),
                             ('business_reject', case_business_reject),
                             ('dialog', case_dialog)]:
                try:
                    fn(s, report)
                except AssertionError as e:
                    failures.append(f'{name}: {e}')
            try:
                s.screenshot('demo-native.png')
                report['visual'].append('demo-native.png')
            except AssertionError as e:
                failures.append(f'screenshot: {e}')
        finally:
            s.quit()
        # quit/reopen (V-05)
        try:
            s2 = Session(args, out, 1, DEMO_APP, '选色设置表单', name='reopen')
            s2.quit()
            report['reopen'] = 'pass'
        except (AssertionError, RuntimeError) as e:
            failures.append(f'reopen: {e}')
        # paint consumption
        try:
            case_paint(args, out, report)
        except AssertionError as e:
            failures.append(f'paint: {e}')
        # narrow window + light/dark (V-05)
        try:
            case_narrow(args, out, report)
        except (AssertionError, RuntimeError) as e:
            failures.append(f'narrow: {e}')
    else:
        try:
            run_vue(args, out, report)
            report['visual'] += ['vue-light.png', 'vue-narrow.png', 'vue-dark.png', 'vue-dialog.png', 'vue-dialog-narrow.png']
        except AssertionError as e:
            failures.append(f'vue: {e}')

    try:
        report['commit'] = subprocess.check_output(
            ['git', '-C', str(REPO), 'log', '-1', '--format=%H', '--',
             'packages/color-picker', 'examples/color-picker',
             'tests/product/verify_color_picker.py', 'examples/ui/031-paint'],
            text=True).strip()
        report['head'] = subprocess.check_output(
            ['git', '-C', str(REPO), 'rev-parse', 'HEAD'], text=True).strip()
    except Exception:
        report['commit'] = ''
        report['head'] = ''
    hashed = {}
    for rel in ('packages/color-picker/color_picker.at',
                'packages/color-picker/color_model.at',
                'packages/color-picker/color_dialog.at',
                'tests/product/verify_color_picker.py'):
        fp = REPO / rel
        hashed[rel] = hashlib.sha256(fp.read_bytes()).hexdigest()[:16] if fp.exists() else 'missing'
    report['input_hashes'] = hashed
    binaries = {'auto': str(args.auto_bin), 'ui_tool': str(args.ui_tool_bin)}
    report['tools'] = {k: v for k, v in binaries.items()}
    if failures:
        report['result'] = 'fail'
        report['failures'] = failures
        (out / f'report-{args.backend}.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
        print(json.dumps(report, ensure_ascii=False, indent=2))
        print('GATE FAIL: ' + '; '.join(failures))
        sys.exit(1)
    report['result'] = 'pass'
    (out / f'report-{args.backend}.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))
    print('GATE PASS')


if __name__ == '__main__':
    main()
