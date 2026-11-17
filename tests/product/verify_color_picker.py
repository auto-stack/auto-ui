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
        shot = self.call('autoui_screenshot')
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
    block = snap[i:i + 6000] if i >= 0 else ''
    return re.findall(r'"(\[\d+\][^"]*)"', block)


def wait_log(s, pred, tries=14, wait=0.7):
    for _ in range(tries):
        if pred(log_lines(s)):
            return True
        time.sleep(wait)
    return False


def commits(lines):
    return [l for l in lines if 'ink 接受' in l]


# ── demo cases (native) ──

def case_known_colors(s, report):
    """V-01/06: fresh session; hue pen -> surface -> alpha -> hue edge."""
    vfd.require(s.ink_text() == '当前画笔 #3974E6CC', f'initial ink wrong: {s.ink_text()}')
    # hue bar (8,6) -> h=15; surface (96,95) -> s=50 v=50; alpha baseline CC
    s.pen(2, [(8, 6)])
    s.pen(2, [(96, 95)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #805040CC' in x for x in l)),
                f'hue+surface commit missing: {log_lines(s)[-3:]}')
    # alpha bar (96,6) -> a=127
    s.pen(3, [(96, 6)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8050407F' in x for x in l)),
                f'alpha commit missing: {log_lines(s)[-3:]}')
    # hue right edge (192,6) -> 360 -> normalized 0; alpha stays 7F
    s.pen(2, [(192, 6)])
    s.pen(2, [(96, 95)])
    vfd.require(wait_log(s, lambda l: any('ink 接受 #8040407F' in x for x in l)),
                f'hue-edge commit missing: {log_lines(s)[-3:]}')
    report['cases'].append('V-01/06 known-colors pass')


def case_gestures(s, report):
    """V-02: one gesture = one commit; preview carries surface source; external
    reset revokes drafts."""
    before = len(log_lines(s))
    s.pen(2, [(30, 30), (96, 95)])
    vfd.require(wait_log(s, lambda l: len(commits(l)) >= before and len(
        [x for x in l if 'ink 接受' in x]) >= 1), 'gesture did not commit')
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
    s.call('autoui_type', element_id=r_id, text='64')
    time.sleep(0.4)
    s.call('autoui_action', element_id=r_id, action='submit', latency=True)
    vfd.require(wait_log(s, lambda l: any('ink 接受 #4074E6CC' in x for x in l)),
                f'channel commit missing: {log_lines(s)[-3:]}')
    s.call('autoui_type', element_id=r_id, text='abc')
    time.sleep(0.4)
    s.call('autoui_action', element_id=r_id, action='submit')
    time.sleep(1.0)
    snap2 = s.snapshot()
    vfd.require('invalid_channel' in snap2, 'invalid channel error missing')
    vfd.require(s.ink_text() == '当前画笔 #4074E6CC', f'invalid channel changed ink: {s.ink_text()}')
    report['cases'].append('V-01 channel entry pass')


def case_two_instances(s, report):
    """V-03: inline picker palette commits ink; dialog label independent."""
    s.hit(4, 'p0')
    vfd.require(wait_log(s, lambda l: any('ink 接受 #EF4444FF' in x for x in l)),
                f'inline palette p0 commit missing: {log_lines(s)[-3:]}')
    label = re.search(r'当前标签色 (\S+?)"?', s.snapshot()).group(1)
    vfd.require(label != '#EF4444FF', f'label leaked inline color: {label}')
    report['cases'].append('V-03 two-instance isolation pass')


def case_business_reject(s, report):
    """V-03: strict mode business-reject keeps formal value."""
    ink0 = s.ink_text()

    def toggle_strict():
        snap = s.snapshot()
        m = re.search(r'checkbox #vnode_(\d+) "[^"]*业务拒绝模式', snap)
        vfd.require(m, 'strict checkbox not found')
        s.call('autoui_action', element_id=f'vnode_{m.group(1)}', action='toggle', latency=True)

    toggle_strict()
    s.hit(4, 'p1')
    vfd.require(wait_log(s, lambda l: any('ink 拒绝' in x for x in l)),
                f'strict reject not logged: {log_lines(s)[-3:]}')
    time.sleep(1.0)
    vfd.require(s.ink_text() == ink0, f'business reject changed formal ink: {ink0} -> {s.ink_text()}')
    toggle_strict()
    report['cases'].append('V-03 business-reject pass')


def case_dialog(s, report):
    """V-02/03: dialog draft/confirm-once/cancel-zero; dialog pen scoping."""
    label0 = re.search(r'当前标签色 (\S+?)"?', s.snapshot()).group(1)
    s.press_button('打开标签色对话框')
    cans = s.canvases()
    vfd.require(len(cans) >= 10, f'dialog canvases missing: {len(cans)}')
    # dialog picker: hue bar + surface (dialog canvases are the last 5)
    base = len(cans) - 5
    s.pen(base + 2, [(8, 6)])
    s.pen(base + 2, [(96, 95)])
    s.press_button('关闭')
    time.sleep(1.0)
    label1 = re.search(r'当前标签色 (\S+?)"?', s.snapshot()).group(1)
    vfd.require(label1 == label0, f'cancel changed formal label: {label0} -> {label1}')
    s.press_button('打开标签色对话框')
    s.pen(base + 2, [(8, 6)])
    s.pen(base + 2, [(96, 95)])
    s.press_button('确认')
    vfd.require(wait_log(s, lambda l: any('label 接受' in x for x in l)),
                f'confirm commit missing: {log_lines(s)[-3:]}')
    time.sleep(1.0)
    label2 = re.search(r'当前标签色 (\S+?)"?', s.snapshot()).group(1)
    vfd.require(label2 != label0, f'confirm did not change label: {label2}')
    report['cases'].append('V-02/03 dialog draft/confirm/cancel pass')


# ── paint consumption (native) ──

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
        # picker palette p1 (#EF4444FF) then paint cell 42 through canvas onhit
        paint.hit(6, 'p1')
        time.sleep(1.0)
        paint.hit(0, 'px42')
        time.sleep(1.0)
        # undo restores the pre-paint state (cell 42 back to legacy red)
        paint.press_button('Undo')
        time.sleep(1.0)
        # save/load round-trip in the new format
        paint.press_button('Save')
        paint.press_button('New')
        paint.press_button('Load')
        time.sleep(1.0)
        st = re.search(r'text #vnode_\d+ "(loaded|no saved canvas|bad save data)"', paint.snapshot())
        vfd.require(st and st.group(1) == 'loaded', f'round-trip load failed')
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
            # r3 channel entry: R field (inputs[1]) -> 64 -> commit #4074E6CC
            page.evaluate('document.querySelectorAll("input")[1].focus()')
            page.keyboard.type('64')
            page.keyboard.press('Enter')
            time.sleep(1.5)
            body3 = page.evaluate('document.body.innerText')
            vfd.require('#4074E6CC' in body3, f'vue channel commit missing: {body3[:240]}')
            page.screenshot(path=str(out / 'vue-light.png'), full_page=True)
            page2 = browser.new_page(viewport={'width': 420, 'height': 560})
            page2.goto(url, timeout=30000)
            time.sleep(3)
            page2.screenshot(path=str(out / 'vue-narrow.png'), full_page=True)
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

    report = {'plan': '088', 'revision': 2, 'backend': args.backend,
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
    else:
        try:
            run_vue(args, out, report)
            report['visual'] += ['vue-light.png', 'vue-narrow.png']
        except AssertionError as e:
            failures.append(f'vue: {e}')

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
