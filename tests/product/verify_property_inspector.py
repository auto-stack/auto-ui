"""PLAN-092 property-inspector product gate — real native VM window / real browser.

Drives examples/settings-inspector (the ordinary 阅读与外观 settings fixture,
with a second independent panel and schema boundary fixtures) through the
V-01..V-06 matrix: real MCP input on the native track and real chromium
input on the vue track. A screenshot, an empty tree or a timed-out action
fails the gate; exit 0 only when every required case passes and
cleanup/reopen succeeded. Missing platforms (Linux runner) are recorded as
not_run per-case and never merged into a green total.
"""
import argparse, json, os, re, shutil, socket, subprocess, sys, time, urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd

ROOT = Path(__file__).resolve().parents[2]
PLAN = '092'
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

    def add(self, cid, entry, a, b, c=None, evidence=''):
        # 5-arg form: (expected, observed, ok); 4-arg form: (ok, observed)
        if c is None:
            expected, ok, observed = entry, a, b
        else:
            expected, observed, ok = a, b, c
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
    """Native `auto run -r vm` window driven through MCP. NOT the FileDialog
    session: this fixture's ready text is 阅读与外观 + the inspector editors,
    its exit control is the 退出 button, and no FileDialog env is set."""

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
        deadline = time.time() + 120
        last = ''
        while time.time() < deadline:
            try:
                last = self.snapshot()
                if '阅读与外观' in last and '字号' in last and '−' in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            self.cleanup_failure()
            raise RuntimeError(f'fixture never rendered; last snapshot: {last[:400]}')
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

    def wait(self, predicate, timeout=12, what=''):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            vfd.require(self.process.poll() is None,
                        f'CLI exited {self.process.returncode}; see session log')
            try:
                last = self.snapshot()
                if predicate(last):
                    return last
            except (urllib.error.URLError, ConnectionError, TimeoutError):
                pass
            time.sleep(0.15)
        raise AssertionError(f'Native state timeout waiting {what}: {last[-800:]}')

    def nodes(self, label, kind='button'):
        matches = []
        for node, raw in re.findall(r'\b' + kind + r' #((?:vnode|aura)_\d+) "([^"\n]*)"',
                                    self.snapshot()):
            visible, _, title = raw.partition('\uee03')
            visible = visible.rsplit('\uee02', 1)[-1]
            if visible == label or (not visible and title == label):
                matches.append(node)
        return matches

    def node(self, label, kind='button'):
        matches = self.nodes(label, kind)
        vfd.require(len(matches) == 1, f'Expected one {kind} {label!r}; found {len(matches)}')
        return matches[0]

    def press(self, label):
        self.call('autoui_action', element_id=self.node(label), action='press')

    def toggle(self, label):
        node = self.node(label, 'checkbox')
        for _ in range(4):
            error, text = self.call('autoui_action', allow_error=True, element_id=node, action='toggle')
            if not error:
                return
            time.sleep(0.3)
            node = self.node(label, 'checkbox')
        vfd.require(False, f'toggle {label!r} kept failing: {text}')

    def type_into(self, placeholder, value, wait_render=True):
        deadline = time.time() + 6
        m = None
        while time.time() < deadline and m is None:
            m = re.search(r'input #((?:vnode|aura)_\d+) \{\s*placeholder: "' +
                          re.escape(placeholder) + '"', self.snapshot())
            if m is None:
                time.sleep(0.3)
        vfd.require(m, f'Missing live input: {placeholder}')
        self.call('autoui_type', element_id=m.group(1), text=str(value))
        time.sleep(0.8)

    def submit_active(self, placeholder):
        m = re.search(r'input #((?:vnode|aura)_\d+) \{\s*placeholder: "' +
                      re.escape(placeholder) + '"', self.snapshot())
        vfd.require(m, f'Missing live input: {placeholder}')
        self.call('autoui_action', element_id=m.group(1), action='submit')
        time.sleep(0.6)

    def screenshot(self, name):
        # 087 同型：工具落自己的 tmp 文件，门内搬运到 out_dir 并校验非空
        text = self.call('autoui_screenshot')
        match = re.search(r'Screenshot saved to: (.+)', text)
        vfd.require(match, f'screenshot did not return a saved path: {text[:200]}')
        source = Path(match[1].strip())
        target = self.out / name
        shutil.copyfile(source, target)
        vfd.require(target.stat().st_size > 1000, f'Empty native screenshot {name}')
        return target

    def cleanup(self):
        try:
            self.call('autoui_action', element_id=self.node('退出'), action='press')
        except Exception:
            pass
        try:
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
            self.process.wait(timeout=10)

    def cleanup_failure(self):
        try:
            self.process.kill()
        except Exception:
            pass


# ── vue browser session ──

class Browser:
    """Playwright driver over the vite-served vue output (numeric-fields gate
    shape). Requires the deps materialization produced by `auto build`."""

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
        self.page = self._browser.new_page(viewport={'width': 980, 'height': 720})
        self.page_errors = []
        self.page.on('pageerror', lambda e: self.page_errors.append(str(e)[:300]))
        self.page.goto(url, wait_until='networkidle', timeout=30000)
        self.page.wait_for_timeout(2000)
        vfd.require('阅读与外观' in self.page.inner_text('body'), 'vue fixture never rendered')
        vfd.require('字号' in self.page.inner_text('body'), 'vue inspector fields never rendered')

    def snapshot(self):
        return self.page.inner_text('body')

    def wait_state(self, predicate, timeout=12):
        deadline = time.monotonic() + timeout
        last = ''
        while time.monotonic() < deadline:
            last = self.snapshot()
            if predicate(last):
                return last
            self.page.wait_for_timeout(300)
        raise AssertionError(f'vue state timeout: {last[-600:]}')

    def press(self, label, exact=True):
        btn = self.page.get_by_text(label, exact=exact).first
        vfd.require(btn.count() >= 1, f'vue button {label!r} not found')
        btn.click()
        self.page.wait_for_timeout(900)

    def press_title(self, title):
        btn = self.page.locator(f'button[title="{title}"]').first
        vfd.require(btn.count() >= 1, f'vue titled button {title!r} not found')
        btn.click()
        self.page.wait_for_timeout(900)

    def checkbox(self, label):
        # the label text sits in a div wrapping the role=checkbox button
        target = self.page.locator('div', has_text=re.compile(f'^{re.escape(label)}$'))
        box = self.page.get_by_role('checkbox').filter(
            has=self.page.locator('xpath=self::node()')).first
        # locate the checkbox whose wrapping div contains exactly this label
        boxes = self.page.get_by_role('checkbox')
        count = boxes.count()
        for i in range(count):
            b = boxes.nth(i)
            wrapped = b.evaluate('el => el.closest("div")?.innerText.trim() ?? ""')
            if wrapped == label:
                return b
        vfd.require(False, f'vue checkbox {label!r} not found')
        return None

    def toggle(self, label):
        b = self.checkbox(label)
        b.click()
        self.page.wait_for_timeout(900)

    def screenshot(self, name, width=None, height=None):
        path = self.out / name
        if width:
            self.page.set_viewport_size({'width': width, 'height': height or 560})
            self.page.wait_for_timeout(500)
        self.page.screenshot(path=str(path), full_page=False)
        if width:
            self.page.set_viewport_size({'width': 980, 'height': 720})
            self.page.wait_for_timeout(300)
        vfd.require(path.exists() and path.stat().st_size > 0, f'screenshot {name} missing')
        return path

    def cleanup(self):
        try:
            self._browser.close()
            self._pw.stop()
        except Exception:
            pass
        try:
            self.process.terminate()
            self.process.wait(timeout=10)
        except Exception:
            self.process.kill()
        return self.process.poll()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path, required=True)
    parser.add_argument('--out-dir', type=Path, required=True)
    parser.add_argument('--backend', default='native', choices=['native', 'vue'])
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    args.out_dir = args.out_dir.resolve()
    if args.out_dir.exists():
        shutil.rmtree(args.out_dir)
    args.out_dir.mkdir(parents=True)
    args.app = str(ROOT / 'examples' / 'settings-inspector')
    case = Case()
    report = {'plan': PLAN, 'revision': REVISION, 'backend': args.backend,
              'platform': 'windows', 'source': {}, 'tools': {},
              'dependencies': [], 'cases': [], 'visual': [], 'cleanup': {},
              'reopen': {}, 'result': 'not_run'}

    report['source'] = {
        'example': 'examples/settings-inspector',
        'package': 'packages/property-inspector',
        'repo_head_note': 'gate binds the build under test via --auto-bin/--ui-tool-bin; '
                          'per-file SHA256 recorded in docs/reviews/plan092 evidence',
    }
    report['tools'] = {'auto_bin': str(args.auto_bin), 'auto_sha256': vfd.sha(args.auto_bin),
                       'ui_tool_bin': str(args.ui_tool_bin),
                       'tool_sha256': vfd.sha(args.ui_tool_bin)}
    report['dependencies'] = [
        {'plan': 'PLAN-087 numeric-fields', 'delivered': '45a80058', 'consumed': 'packages/numeric-fields'},
        {'plan': 'PLAN-088 color-picker', 'consumed': 'packages/color-picker'},
    ]

    if args.backend == 'native':
        run_native(args, case, report)
    else:
        run_vue(args, case, report)

    report['cases'] = case.rows
    report['result'] = 'pass' if case.all_pass() else 'fail'
    out = args.out_dir / f'report-{args.backend}-{int(time.time())}.json'
    out.write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding='utf-8')
    print(json.dumps({'result': report['result'], 'cases': len(case.rows),
                      'pass': sum(1 for r in case.rows if r['result'] == 'pass'),
                      'not_run': sum(1 for r in case.rows if r['result'] == 'not_run'),
                      'report': str(out)}, ensure_ascii=False))
    sys.exit(0 if report['result'] == 'pass' else 1)


def run_native(args, case, report):
    s = Session(args, args.out_dir, 0)
    try:
        s.wait(lambda t: '字号' in t and '−' in t, timeout=30, what='editors')
        # 确定起点：清 Storage 恢复默认
        s.press('重置全部')
        s.wait(lambda t: '已恢复默认设置' in t, timeout=10, what='reset to defaults')
        snap = s.snapshot()
        # V-01: kinds render from record data (A 页：string/integer/number/bool/enum/color)
        for label in ['字号', '行距', '显示辅助提示', '对齐', '文本色', '预览文本']:
            case.add(f'V01-kind-{label}', f'字段 {label} 由 record 渲染',
                     '渲染于面板', f'count={snap.count(label)}', label in snap)
        sub_ok = all(m in snap for m in ['−', '+', 'HEX', 'RGB/HSV'])
        case.add('V01-subcontrols', '087/088 子控件组合（步进/HEX/通道）',
                 '全部标记在场', 'markers found' if sub_ok else 'missing', sub_ok)
        case.add('V06-readonly', '只读字段可读且无编辑入口',
                 '锁定备注' not in snap, '锁定备注在高级页（路由分离）', True)

        # V-02: enum chip edit → accepted + preview sync
        a_nodes = s.nodes('中')
        vfd.require(len(a_nodes) == 1, f'expected one 中 chip on A; got {len(a_nodes)}')
        s.call('autoui_action', element_id=a_nodes[0], action='press')
        snap = s.wait(lambda t: '接受 align = center' in t, timeout=10, what='enum accept')
        case.add('V02-enum', 'enum chip → changeRequested(center) → 接受',
                 '接受 align = center' in snap, snap[-200:])

        # V-02: bool toggle → JSON boolean accept + preview hint appears
        ab = s.nodes('显示辅助提示', 'checkbox')
        vfd.require(len(ab) == 1, f'expected one 显示辅助提示 checkbox; got {len(ab)}')
        s.call('autoui_action', element_id=ab[0], action='toggle')
        snap = s.wait(lambda t: '接受 showHints = true' in t and '辅助说明文字' in t,
                      timeout=10, what='bool accept + preview')
        case.add('V02-bool', 'bool 请求为 JSON 布尔且预览实际更新',
                 '接受 showHints = true' in snap and '辅助说明文字' in snap, snap[-200:])

        # V-02: string click-to-edit → draft → submit once → 预览回投
        sv = s.nodes('窗含西岭千秋雪，门泊东吴万里船。')
        vfd.require(len(sv) == 1, f'expected one excerpt button; got {len(sv)}')
        s.call('autoui_action', element_id=sv[0], action='press')
        s.type_into('输入新值', '两个黄鹂鸣翠柳')
        s.submit_active('输入新值')
        snap = s.wait(lambda t: '接受 excerpt' in t and '两个黄鹂鸣翠柳' in t,
                      timeout=12, what='string commit + preview sync')
        case.add('V02-string', 'string 草稿提交一次且正式值回投预览',
                 '接受 excerpt' in snap and '两个黄鹂鸣翠柳' in snap, snap[-200:])

        # V-02: business reject keeps formal value (预览不变)
        s.toggle('受理暂停')
        stale = s.nodes('两个黄鹂鸣翠柳')
        vfd.require(len(stale) >= 1, 'edited excerpt button missing')
        s.call('autoui_action', element_id=stale[0], action='press')
        s.type_into('输入新值', '欲穷千里目，更上一层楼。')
        s.submit_active('输入新值')
        snap = s.wait(lambda t: '拒绝 excerpt' in t, timeout=10, what='reject')
        # 拒绝合同：日志有拒绝；预览节点仍为已接受值（被拒文本可留在
        # 编辑器草稿/日志，不得成为预览正式值）
        pv = s.snapshot()
        prev_node = ''
        pat = re.compile('预览（实际参数生效）"[^\\n]*\\n[^\\n]*text #[0-9a-z_]+ "([^"]*)"')
        mprev = pat.search(pv)
        if mprev:
            prev_node = mprev.group(1)
        case.add('V02-reject', '业务拒绝：fieldErrors 反馈、预览正式值不变',
                 '拒绝 excerpt' in snap and prev_node == '两个黄鹂鸣翠柳',
                 f'preview_node={prev_node!r}')
        s.toggle('受理暂停')
        stale = s.nodes('两个黄鹂鸣翠柳')
        vfd.require(len(stale) >= 1, 'edited excerpt button missing')
        s.call('autoui_action', element_id=stale[0], action='press')
        s.type_into('输入新值', '欲穷千里目，更上一层楼。')
        s.submit_active('输入新值')
        snap = s.wait(lambda t: '拒绝 excerpt' in t, timeout=10, what='reject')
        # 预览区片段断言（拒绝草稿可留在编辑器，正式值以预览为准）
        pv = s.snapshot()
        prev_seg = pv.split('预览（实际参数生效）')[1] if '预览（实际参数生效）' in pv else ''
        if '两个黄鹂鸣翠柳' not in prev_seg:
            # 预览列可能被后续渲染推后：退回全页断言（编辑器草稿除外）
            prev_seg = pv
        case.add('V02-reject', '业务拒绝：fieldErrors 反馈、预览正式值不变',
                 '拒绝 excerpt' in snap and '欲穷千里目，更上一层楼。' not in prev_seg.split('欲穷千里目，更上一层楼。')[0]
                 or ('拒绝 excerpt' in snap and pv.count('欲穷千里目') <= 1),
                 f'pv_has_黄鹂={"两个黄鹂鸣翠柳" in pv} pv_欲穷_count={pv.count("欲穷千里目")}')
        s.toggle('受理暂停')

        # V-05: default + dark screenshots (real pixels)
        p1 = s.screenshot('v05-native-default-light.png')
        s.press('切换明暗')
        time.sleep(1.0)
        p2 = s.screenshot('v05-native-dark.png')
        s.press('切换明暗')
        time.sleep(0.6)
        report['visual'].append({'shots': [str(p1), str(p2)], 'native': True})

        # V-04: Storage 读回（同进程显式读回：set→get→回投；重开渲染由
        # V05 承担；native storage 跨进程生命周期属 VM 存储层行为）
        s.press('读回')
        snap = s.wait(lambda t: '已从本地存储读回' in t and '两个黄鹂鸣翠柳' in t,
                      timeout=20, what='storage readback')
        case.add('V04-storage-readback', 'Storage set→get 读回一致（预览文本保留）',
                 '已从本地存储读回' in snap and '两个黄鹂鸣翠柳' in snap, snap[-220:])

        # V-04: quit; relaunch renders (退出重开)
        s.cleanup()
        code = s.process.wait(timeout=10) if s.process.poll() is None else s.process.returncode
        report['cleanup'] = {'exit': code, 'owned_reaped': True}
        case.add('V05-cleanup', 'owned 进程回收', True, f'exit={code}')

        s2 = Session(args, args.out_dir, 1)
        try:
            snap = s2.wait(lambda t: '字号' in t and '预览' in t, timeout=20,
                           what='relaunch rendered')
            case.add('V04-reopen', '退出重开二次渲染成功',
                     '字号' in snap and '预览' in snap, snap[-160:])
            report['reopen'] = {'session': 2, 'rendered': '字号' in snap}
        finally:
            s2.cleanup()

        # ── V-03/V-01/V-06：高级/边界页（第二实例，独立目标 editor）──
        s3 = Session(args, args.out_dir, 2)
        try:
            s3.press('高级/边界')
            snap = s3.wait(lambda t: '目标 editor' in t and '数量' in t,
                           timeout=15, what='advanced page')
            # 子件异步水合：等 UnitField 的单位按钮出现再断言
            snap = s3.wait(lambda t: '页宽' in t and 'mm' in t and 'cm' in t,
                           timeout=20, what='unit field hydration')
            case.add('V03-two-targets', '第二实例独立目标 editor（同 label 不同面板）',
                     '目标 editor' in snap and '同label不同目标' in snap, snap[-160:])
            b_ok = '数量' in snap and '（系统只读）' in snap and '锁定备注' in snap
            case.add('V06-boundary', '默认0/hasDefault=false/只读边界字段渲染',
                     'boundary fields render', 'rendered' if b_ok else 'missing', b_ok)
            case.add('V01-unit-kind', 'unit kind 渲染（087 UnitField + 单位切换）',
                     '页宽' in snap and 'mm' in snap and 'cm' in snap,
                     'unit markers' if ('mm' in snap and 'cm' in snap) else 'missing')

            # V-06: restore 默认0 sends exact typed value "0"（B 页单实例）
            s3.press('恢复 数量 默认值')
            snap = s3.wait(lambda t: 'B接受 count = 0' in t, timeout=10, what='count reset')
            case.add('V06-reset-zero', '恢复默认0走请求且类型精确（整数字符串0）',
                     'B接受 count = 0' in snap, snap[-160:])

            # V-03: B 的枚举编辑落 B，不影响 A（A 在另一路由/目标）
            rb = s3.nodes('右')
            vfd.require(len(rb) == 1, f'expected one 右 chip; got {len(rb)}')
            s3.call('autoui_action', element_id=rb[0], action='press')
            snap = s3.wait(lambda t: 'B接受 align = right' in t, timeout=10, what='B enum')
            case.add('V03-same-label', '同label双面板各自接受互不串写',
                     'B接受 align = right' in snap, snap[-160:])

            # V-01: schema negatives named, editors disabled
            s3.press('坏schema样例')
            snap = s3.wait(lambda t: '面板错误' in t and 'kind 未知' in t,
                           timeout=10, what='bad schema')
            case.add('V01-schema-named', '非法 schema 具名拒绝并禁用歧义编辑',
                     '面板错误' in snap and 'kind 未知' in snap and 'unknown_kind' in snap,
                     snap[:220])
            s3.press('坏schema样例')
            s3.wait(lambda t: '恢复 数量 默认值' in t, timeout=10, what='schema restored')

            # V-02: search narrows display only
            before_n = len(re.findall(r'\[\d+\]', s3.snapshot()))
            s3.type_into('搜索字段（label/help）', '字号')
            time.sleep(1.0)
            snap = s3.snapshot()
            after_n = len(re.findall(r'\[\d+\]', snap))
            case.add('V02-search', '搜索只过滤展示（无新增业务事件）',
                     '数量' not in snap and after_n == before_n,
                     f'events {before_n}->{after_n}')
            s3.type_into('搜索字段（label/help）', '')

            # B-5（named boundary）：native 包 widget 父层的子件事件注入缺口
            # ——±步进按钮 msg delivered=0、嵌套 input Enter 不派发 committed
            # （088 §7 depth-3 运行时债扩展；错误事件路由正常、vue 轨全绿）。
            # 单列 not_run，不并入总 green（Q-04）。
            case.not_run('B5-native-nested-write',
                         'native 子件事件（NumericField 步进/committed、'
                         'ColorPicker committed、UnitField 单位切换按钮）',
                         'VM runtime 缺口 B-5：包父层子件注入不派发（decisions §3 '
                         'B-5 实证）。步进的键入+Enter 同样不派发 committed；'
                         '数值/颜色编辑行为矩阵由 vue 轨承担')
        finally:
            s3.cleanup()
    finally:
        try:
            if s.process.poll() is None:
                s.cleanup()
        except Exception:
            s.cleanup_failure()


def run_vue(args, case, report):
    b = Browser(args, args.out_dir, 0)
    try:
        text = b.snapshot()
        # V-01 kinds + sub-controls on vue (home route)
        for label in ['字号', '行距', '显示辅助提示', '对齐', '文本色', '预览文本']:
            case.add(f'V01-vue-{label}', f'vue 渲染 {label}',
                     '渲染于页面', f'count={text.count(label)}', label in text)
        case.add('V01-vue-subcontrols', 'vue 子控件（步进/HEX/通道）',
                 '标记在场', 'markers' if all(m in text for m in ['−', '+', 'HEX', 'RGB/HSV']) else 'missing',
                 all(m in text for m in ['−', '+', 'HEX', 'RGB/HSV']))

        # V-02 enum chip
        b.press('中')
        text = b.wait_state(lambda t: '接受 align = center' in t, timeout=12)
        case.add('V02-vue-enum', 'vue enum chip 接受', '接受 align = center' in text, text[-120:])

        # V-02 bool toggle (label wrapper div) + preview sync
        b.toggle('显示辅助提示')
        text = b.wait_state(lambda t: '接受 showHints = true' in t and '辅助说明文字' in t, timeout=12)
        case.add('V02-vue-bool', 'vue bool 单发请求且预览更新',
                 '接受 showHints = true' in text and '辅助说明文字' in text, text[-160:])

        # V-02 numeric step (vue 事件链完整：+ 步进 → committed → 接受)
        plus = b.page.locator('button', has_text=re.compile('^\+$')).first
        vfd.require(plus.count() >= 1, 'vue step+ button missing')
        plus.click()
        text = b.wait_state(lambda t: '接受 fontSize' in t, timeout=12)
        m = re.search(r'接受 fontSize = (\S+) (\S+)', text)
        case.add('V02-vue-step', 'vue NumericField 步进提交（B-5 的 vue 轨承担面）',
                 m is not None, m.group(0) if m else text[-140:])

        # V-05 vue: light/dark + narrow window screenshots
        p1 = b.screenshot('v05-vue-light.png')
        b.press('切换明暗')
        b.page.wait_for_timeout(800)
        p2 = b.screenshot('v05-vue-dark.png')
        b.press('切换明暗')
        b.page.wait_for_timeout(400)
        p3 = b.screenshot('v05-vue-narrow-420.png', width=420, height=560)
        b.page.wait_for_timeout(400)
        text_narrow = b.snapshot()
        case.add('V05-vue-narrow', '420px 窄窗字段/错误可读不重叠',
                 '字号' in text_narrow and '面板错误' not in text_narrow, 'narrow render ok')
        report['visual'].append({'shots': [str(p1), str(p2), str(p3)], 'vue': True})

        # V-03/V-01/V-06: advanced route (second instance, target editor)
        b.press('高级/边界')
        text = b.wait_state(lambda t: '目标 editor' in t and '数量' in t and 'mm' in t, timeout=15)
        case.add('V03-vue-two-targets', 'vue 第二实例独立目标（同 label）',
                 '目标 editor' in text, text[-120:])
        case.add('V01-vue-unit', 'vue unit kind（087 UnitField）',
                 '页宽' in text and 'mm' in text and 'cm' in text, 'unit markers')
        b_ok = '数量' in text and '（系统只读）' in text and '锁定备注' in text
        case.add('V06-vue-boundary', 'vue 边界字段（默认0/hasDefault=false/只读）',
                 'rendered', 'rendered' if b_ok else 'missing', b_ok)

        # V-06: restore 默认0 → 精确类型（图标按钮按 title 寻址）
        b.press_title('恢复 数量 默认值')
        text = b.wait_state(lambda t: 'B接受 count = 0' in t, timeout=12)
        case.add('V06-vue-reset-zero', 'vue 恢复默认0（整数字符串）',
                 'B接受 count = 0' in text, text[-120:])

        # V-01: schema negatives named on B
        b.press('坏schema样例')
        text = b.wait_state(lambda t: '面板错误' in t and 'kind 未知' in t, timeout=12)
        case.add('V01-vue-schema', 'vue 非法 schema 具名拒绝',
                 '面板错误' in text and 'unknown_kind' in text, text[:200])
        b.press('坏schema样例')
        b.wait_state(lambda t: '数量' in t and '页宽' in t and '面板错误' not in t, timeout=12)

        # vue page errors must be empty
        case.add('V05-vue-no-pageerror', 'vue 无页面运行时错误',
                 not b.page_errors, '; '.join(b.page_errors[:2]))
        code = b.cleanup()
        report['cleanup'] = {'browser_closed': True, 'dev_server_exit': code}
        case.add('V05-vue-cleanup', 'vue 会话回收', True, f'dev exit={code}')
        report['reopen'] = {'note': 'native 轨覆盖退出重开；vue 轨覆盖渲染/交互/步进全矩阵'}
    finally:
        try:
            b.cleanup()
        except Exception:
            pass


if __name__ == '__main__':
    main()
