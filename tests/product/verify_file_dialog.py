#!/usr/bin/env python3
"""Real native VM/MCP gate. No state injection or path-return shortcut.

All generated inputs, app copies, logs and receipts live under --out-dir.
An unavailable platform, empty tree, missing window or timed-out action fails.
"""
import argparse
import ctypes
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request


def require(condition, detail):
    if not condition:
        raise AssertionError(detail)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def processes():
    if os.name == 'nt':
        output = subprocess.check_output([
            'powershell.exe', '-NoProfile', '-Command',
            'Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,@{Name="Started";Expression={$_.CreationDate.ToUniversalTime().Ticks}} | ConvertTo-Json -Compress'
        ], creationflags=subprocess.CREATE_NO_WINDOW)
        return {int(p['ProcessId']): (int(p['ParentProcessId']), p['Name'], int(p['Started'])) for p in json.loads(output)}
    result = {}
    for entry in Path('/proc').iterdir():
        if entry.name.isdecimal():
            try:
                stat = (entry/'stat').read_text()
                rest = stat[stat.rfind(')')+2:].split()
                if rest[0] != 'Z':
                    result[int(entry.name)] = (int(rest[1]), stat[stat.find('(')+1:stat.rfind(')')], int(rest[19]))
            except (OSError, IndexError):
                pass
    return result


def owned_tree(pid):
    table = processes()
    owned = {pid}
    while True:
        # Parent PIDs can be reused by this run while an older unrelated
        # process still records that number. It cannot be our descendant.
        more = {p for p, (parent, _, started) in table.items()
                if parent in owned and parent in table and started >= table[parent][2]}
        if more <= owned:
            return {p: table[p] for p in owned if p in table}
        owned |= more


def windows_for(pids):
    if os.name == 'nt':
        user = ctypes.windll.user32
        user.GetWindowThreadProcessId.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ulong)]
        user.GetWindowTextW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_int]
        user.IsWindowVisible.argtypes = [ctypes.c_void_p]
        result = []
        @ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
        def visit(hwnd, _):
            pid = ctypes.c_ulong()
            user.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
            if pid.value in pids and user.IsWindowVisible(hwnd):
                text = ctypes.create_unicode_buffer(512)
                user.GetWindowTextW(hwnd, text, 512)
                if text.value:
                    result.append({'handle': hwnd, 'pid': pid.value, 'title': text.value})
            return True
        user.EnumWindows(visit, 0)
        return result
    # Force X11 when launching so actual XWayland windows can be attributed.
    tree = subprocess.check_output(['xwininfo', '-root', '-tree'], text=True)
    result = []
    for line in tree.splitlines():
        match = re.search(r'(0x[0-9a-f]+) "([^"]+)"', line)
        if match:
            prop = subprocess.run(['xprop', '-id', match[1], '_NET_WM_PID'], capture_output=True, text=True)
            pid = re.search(r'= (\d+)', prop.stdout)
            if pid and int(pid[1]) in pids:
                result.append({'handle': match[1], 'pid': int(pid[1]), 'title': match[2]})
    return result


def free_port():
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        return listener.getsockname()[1]


class Session:
    def __init__(self, args, app, out, index):
        self.port = free_port()
        self.trace = []
        self.out = out
        self.index = index
        self.output = out/'accepted.json'
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port), AUTO_UI_TOOL_BIN=str(args.ui_tool_bin),
                   FILE_DIALOG_INITIAL_DIRECTORY=str(out/'input'), FILE_DIALOG_RESULT_PATH=str(self.output))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        if os.name != 'nt':
            env['WINIT_UNIX_BACKEND'] = 'x11'
            # winit chooses Wayland when both variables exist.
            env.pop('WAYLAND_DISPLAY', None)
        self.log = open(out/f'session-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=app, env=env, stdout=self.log, stderr=subprocess.STDOUT,
                                       creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        try:
            self.wait(lambda tree: '选择文件' in tree, timeout=90)
            self.owned |= set(owned_tree(self.process.pid))
            self.windows = windows_for(self.owned)
            require(self.windows, 'No owned native window with a title/OS handle')
        except Exception:
            self.cleanup_failure()
            raise

    def call(self, name, allow_error=False, **arguments):
        started = time.monotonic()
        body = json.dumps({'jsonrpc':'2.0', 'id':len(self.trace)+1, 'method':'tools/call',
                           'params':{'name':name, 'arguments':arguments}}).encode()
        with urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body,
                                  headers={'Content-Type':'application/json'}), timeout=5) as response:
            result = json.load(response)
        elapsed = time.monotonic()-started
        self.trace.append({'tool':name, 'arguments':arguments, 'response':result, 'seconds':elapsed})
        error = 'error' in result or result.get('result', {}).get('isError', False)
        text = '\n'.join(item.get('text','') for item in result.get('result', {}).get('content', []))
        require(allow_error or not error, text)
        if name in ('autoui_action', 'autoui_type', 'autoui_keyboard'):
            require(elapsed < 1, f'Ordinary action budget exceeded: {name} {elapsed:.3f}s')
        return (error, text) if allow_error else text

    def snapshot(self):
        return self.call('autoui_snapshot', include_styles=False)

    def wait(self, predicate, timeout=5):
        deadline = time.monotonic()+timeout
        last = ''
        while time.monotonic() < deadline:
            require(self.process.poll() is None, f'CLI exited {self.process.returncode}; see session log')
            try:
                error, last = self.call('autoui_snapshot', allow_error=True, include_styles=False)
                require(not error or any(message in last for message in
                        ('No live VTree snapshot yet', 'No UI available yet')), last)
                if not error and predicate(last):
                    return last
            except (urllib.error.URLError, ConnectionError):
                pass
            time.sleep(.05)
        raise AssertionError(f'Native state timeout: {last[-4000:]}')

    def node(self, label, kind='button'):
        matches = []
        for node, raw in re.findall(r'\b'+kind+r' #((?:vnode|aura)_\d+) "([^"\n]*)"', self.snapshot()):
            # Icon-only controls keep their accessible name in the ordinary
            # tooltip metadata. Never depend on the icon glyph or node index.
            visible, _, title = raw.partition('\uee03')
            visible = visible.rsplit('\uee02', 1)[-1]
            if visible == label or (not visible and title == label):
                matches.append(node)
        require(len(matches) == 1, f'Expected one current {kind}: {label}; found {matches}')
        return matches[0]

    def press(self, label):
        self.call('autoui_action', element_id=self.node(label), action='press')

    def input(self, placeholder, value):
        match = re.search(r'input #((?:vnode|aura)_\d+) \{\s*placeholder: "'+re.escape(placeholder)+'"', self.snapshot())
        require(match, f'Missing live input: {placeholder}')
        self.call('autoui_type', element_id=match[1], text=str(value))

    def key(self, key, **args):
        # Keyboard focus changes are native widget Tasks, not business-state
        # writes. Allow a painted frame between separate synthetic key strokes.
        time.sleep(.12)
        self.call('autoui_keyboard', key=key, **args)
        time.sleep(.12)

    def toggle(self, label):
        # Snapshot publication and native view updates can cross at a frame
        # boundary. A named stale rejection is safe: rediscover the current
        # target, with a bound, rather than replaying that rejected old id.
        for attempt in range(4):
            node = self.node(label, 'checkbox')
            error, text = self.call('autoui_action', allow_error=True, element_id=node, action='toggle')
            if not error:
                return node
            require('Stale action identity' in text and attempt < 3, text)
            time.sleep(.1)

    def open(self, single=False, mode=None):
        # PLAN-084: one entry point per selectionMode button.
        label = mode if mode else ('单选文件' if single else '选择文件')
        self.press(label)
        return self.wait(lambda tree: 'placeholder: "目录路径"' in tree and '载入中' not in tree)

    def navigate(self, path, expected):
        self.input('目录路径', path)
        self.key('Enter')
        return self.wait(lambda tree: expected in tree and '载入中' not in tree)

    def count(self, name):
        return int(re.search(name+r' (\d+)', self.snapshot())[1])

    def selected(self, count):
        return self.wait(lambda tree: f'已选 {count} 项' in tree)

    def accepted(self, expected, label='打开'):
        before = self.count('确认次数')
        self.output.unlink(missing_ok=True)
        self.press(label)
        self.wait(lambda tree: f'确认次数 {before+1}' in tree and 'placeholder: "目录路径"' not in tree)
        actual = json.loads(self.output.read_text(encoding='utf-8'))['files']
        require(len(actual) == len(expected), actual)
        for choice, path in zip(actual, expected):
            require(choice['name'] == path.name, (actual, expected))
            # PLAN-084 save mode may return a not-yet-existing target; an
            # existing candidate must still be identity-checked.
            if os.path.exists(path) and os.path.exists(choice['path']):
                require(os.path.samefile(choice['path'], path), (actual, expected))
            else:
                require(not os.path.exists(choice['path']), f'selector created bytes: {choice}')
                # Windows canonical parents carry a \\?\ prefix; compare
                # normalized path identity instead of raw strings.
                left = choice['path'].replace('\\\\?\\', '')
                require(os.path.normcase(os.path.normpath(left)) ==
                        os.path.normcase(os.path.normpath(str(path))), (actual, expected))
        return actual

    def save_name(self, value):
        self.input('要保存的文件名', value)
        return self.wait(lambda tree: '保存' in tree and '载入中' not in tree)

    def screenshot(self, name):
        text = self.call('autoui_screenshot')
        match = re.search(r'Screenshot saved to: (.+)', text)
        require(match, text)
        source = Path(match[1].strip())
        target = self.out/f'{self.index}-{name}.png'
        shutil.copyfile(source, target)
        require(target.stat().st_size > 1000, 'Empty native screenshot')
        return {'path':str(target), 'sha256':sha(target)}

    def file_list_scroll(self):
        # Discover the actual file-list viewport. The locations sidebar has
        # its own scroll container and must not be mistaken for this list.
        lines = self.snapshot().splitlines()
        for index, line in enumerate(lines):
            match = re.match(r'(\s*)scrollable #((?:vnode|aura)_\d+)', line)
            if not match:
                continue
            indent = len(match[1])
            block = []
            for child in lines[index+1:]:
                if child.strip() and len(child)-len(child.lstrip()) <= indent:
                    break
                block.append(child)
            if any(re.search(r'checkbox #', child) for child in block):
                return match[2]
        raise AssertionError('No ordinary file-list scroll control')

    def system_accessibility(self):
        """Read the actual Windows provider and invoke one real Toggle action."""
        require(os.name == 'nt', 'Windows system tree requested on another platform')
        window = next(window for window in self.windows if window['title'] == 'FileDialog fixture')
        script = self.out/'system-tree.ps1'
        script.write_text('''
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
$root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]HANDLE)
$nodes = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
$rows = @()
$toggle = $null
$background = 'pattern unavailable'
foreach ($node in $nodes) {
    $current = $node.Current
    $value = $null
    $pattern = $null
    if ($node.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$pattern)) { $value = $pattern.Current.Value }
    $rows += [pscustomobject]@{ role=$current.ControlType.ProgrammaticName; name=$current.Name; value=$value; enabled=$current.IsEnabled; focus=$current.HasKeyboardFocus; x=$current.BoundingRectangle.X; y=$current.BoundingRectangle.Y; width=$current.BoundingRectangle.Width; height=$current.BoundingRectangle.Height }
    if ($current.Name -eq 'a.txt' -and $current.ControlType -eq [System.Windows.Automation.ControlType]::CheckBox) {
        $toggle = $node.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    }
    if ($current.Name -eq '底层计数') {
        $invoke = $null
        if ($node.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$invoke)) {
            try { $invoke.Invoke(); $background = 'invoked' } catch { $background = $_.Exception.Message }
        }
    }
}
if ($null -eq $toggle) { throw 'No real system TogglePattern for a.txt' }
$toggle.Toggle()
[Console]::WriteLine((@{ nodes=$rows; background_attempt=$background } | ConvertTo-Json -Depth 4))
'''.replace('HANDLE',str(int(window['handle']))),encoding='utf-8-sig')
        output = subprocess.check_output(['powershell.exe','-NoProfile','-STA','-ExecutionPolicy','Bypass','-File',str(script)],
                                         timeout=20,creationflags=subprocess.CREATE_NO_WINDOW)
        system = json.loads(output.decode('utf-8-sig'))
        rows = system['nodes']
        (self.out/'system-tree.json').write_text(json.dumps(system,ensure_ascii=False,indent=2),encoding='utf-8')
        path = next(row for row in rows if row['role']=='ControlType.Edit' and row['name']=='目录路径')
        require(path['value'] and path['width'] > 0 and path['height'] > 0, path)
        selection_cell = next(row for row in rows if row['role']=='ControlType.CheckBox' and row['name']=='a.txt')
        require(selection_cell['width'] > 0 and abs(selection_cell['width']-selection_cell['height']) <= 2,
                f'File checkbox must occupy a square selection cell: {selection_cell}')
        self.selected(1)
        require(self.count('底层次数') == 0,'System accessibility action escaped the modal boundary')
        self.toggle('a.txt'); self.selected(0)
        return {'path':str(self.out/'system-tree.json'),'sha256':sha(self.out/'system-tree.json'), 'system_toggle':'accepted; MCP readback selected 1; ordinary MCP toggle restored 0'}

    def close(self):
        self.owned |= set(owned_tree(self.process.pid))
        try:
            self.press('退出夹具')
        except (http.client.RemoteDisconnected, ConnectionResetError):
            # The successful exit can close the response socket before its
            # reply is flushed. CLI exit, all owned PIDs and port rebinding
            # below remain mandatory; EOF alone never counts as success.
            self.trace.append({'documented_quit_response':'connection closed during exit'})
        require(self.process.wait(timeout=15) == 0, 'Documented quit must return CLI success')
        self.log.close()
        reaping = time.monotonic()
        remaining = self.owned & set(processes())
        initially_remaining = sorted(remaining)
        while remaining and time.monotonic()-reaping < 5:
            time.sleep(.05)
            remaining = self.owned & set(processes())
        self.trace.append({'quit_reap_seconds':time.monotonic()-reaping,
                           'initially_remaining':initially_remaining,'remaining':sorted(remaining)})
        require(not remaining, f'Owned processes survived quit: {remaining}')
        body = json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'autoui_snapshot','arguments':{}}}).encode()
        started = time.monotonic()
        try:
            urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp',data=body),timeout=5)
            raise AssertionError('Closed MCP still answered')
        except urllib.error.URLError as error:
            reason = error.reason
            require(isinstance(reason, ConnectionRefusedError) or getattr(reason, 'winerror', None) == 10061,
                    f'Negative control must be connection refused, not timeout: {error}')
        require(time.monotonic()-started < 6, 'Negative target was unbounded')
        with socket.socket() as listener:
            listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            listener.bind(('127.0.0.1', self.port))
        self.save()

    def save(self):
        (self.out/f'trace-{self.index}.json').write_text(json.dumps(self.trace,ensure_ascii=False,indent=2),encoding='utf-8')

    def cleanup_failure(self):
        diagnostic = {'cli_exit_before_cleanup':self.process.poll(),
                      'owned_processes_before_cleanup':list(owned_tree(self.process.pid))}
        try:
            diagnostic['live_snapshot_before_cleanup'] = self.snapshot()
        except Exception as error:
            diagnostic['snapshot_error_before_cleanup'] = repr(error)
        (self.out/f'failure-state-{self.index}.json').write_text(json.dumps(diagnostic,ensure_ascii=False,indent=2),encoding='utf-8')
        self.save()
        if self.process.poll() is None:
            if os.name == 'nt':
                subprocess.run(['taskkill','/PID',str(self.process.pid),'/T','/F'],capture_output=True)
            else:
                # Only descendants captured from this owned CLI are eligible.
                for pid in reversed(list(owned_tree(self.process.pid))):
                    try: os.kill(pid, 15)
                    except ProcessLookupError: pass
            try: self.process.wait(timeout=10)
            except subprocess.TimeoutExpired: self.process.kill(); self.process.wait(timeout=5)
        self.log.close()


def run(args):
    root = Path(__file__).resolve().parents[2]
    # The isolated app is outside the worktree group: pin source discovery
    # explicitly instead of falling back to moving main checkouts.
    dependency_roots = {}
    for name in ['lang','ui','man','web','down']:
        source = root.parent/('auto-'+name)
        require(source.is_dir(),f'Missing pinned dependency root: {source}')
        os.environ['AUTO_'+name.upper()+'_ROOT'] = str(source)
        dependency_roots[name] = str(source)
    out = args.out_dir.resolve()
    require(not out.exists(), f'Use a fresh output directory: {out}')
    out.mkdir(parents=True)
    fixture = out/'input'
    child = fixture/"中文 子目录'引号"
    child.mkdir(parents=True)
    for name in ['a.txt','b.MD','other.epub','cover.png']:
        (fixture/name).write_text('independent input '+name,encoding='utf-8')
    (child/'a.txt').write_text('different parent identity',encoding='utf-8')
    # PLAN-084 fixtures: an overwrite target with known bytes and a hash
    # baseline proving the selector itself never writes any fixture bytes.
    overwrite_target = fixture/'覆盖目标.md'
    overwrite_target.write_text('plan084 overwrite target bytes',encoding='utf-8')
    large = out/'large'
    large.mkdir()
    for i in range(1003):
        (large/f'中文长文件名_{i:04}_分页完整性.txt').write_text(str(i),encoding='utf-8')
    app = out/'app'
    shutil.copytree(root/'examples/file-dialog',app,ignore=shutil.ignore_patterns('deps','dist','gen','build'))
    entry = app/'src/front/app.at'
    entry.write_text(entry.read_text(encoding='utf-8').replace('../../../../packages/file-dialog',(root/'packages/file-dialog').as_posix()),encoding='utf-8')
    pac = app/'pac.at'
    pac.write_text(pac.read_text(encoding='utf-8').replace('900x700',args.window),encoding='utf-8')
    if args.index_repo:
        # An existing local registry mirror supports offline CLI setup. This
        # is app-local configuration; do not rewrite the user's global config.
        (app/'am.at').write_text('index: { default: '+json.dumps(args.index_repo)+' }\n',encoding='utf-8')
    baseline = {str(path.relative_to(fixture)): sha(path) for path in sorted(fixture.rglob('*')) if path.is_file()}
    report = {'platform':sys.platform, 'scope':'narrow_visual' if args.visual_only else 'full_native_vm', 'graphics_backend':os.environ.get('ICED_BACKEND','default'), 'auto':{'path':str(args.auto_bin),'sha256':sha(args.auto_bin)},
              'ui_tool':{'path':str(args.ui_tool_bin),'sha256':sha(args.ui_tool_bin)},'dependency_roots':dependency_roots,'sessions':[]}
    session = None
    try:
        session = Session(args,app,out,1)
        record = {'command':session.command,'windows':session.windows,'pid':session.process.pid}
        session.open()
        require('onclick: .PreviousPage' not in session.snapshot()
                and 'onclick: .NextPage' not in session.snapshot(),
                'Single-page results must hide the entire pagination control')
        session.press('刷新')
        session.wait(lambda tree:'"a.txt"' in tree and '载入中' not in tree)
        session.press('上一级')
        session.wait(lambda tree:'"a.txt"' not in tree and '载入中' not in tree)
        session.navigate(fixture, '"a.txt"')
        if args.visual_only:
            record['initial_screenshot'] = session.screenshot('initial')
            session.call('autoui_action',element_id=session.file_list_scroll(),action='scroll',value=500)
            session.toggle('a.txt'); session.selected(1)
            record['footer_screenshot'] = session.screenshot('footer')
            record['accepted'] = session.accepted([fixture/'a.txt'])
            session.press('切换明暗'); session.open()
            record['alternate_theme_screenshot'] = session.screenshot('alternate-theme')
            session.press('取消'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
            # PLAN-084 V-08: directory / save / overwrite regions of the r2
            # visual guide, light and dark, real screenshots only.
            session.open(mode='选择目录')
            session.wait(lambda tree: '选择一个目录' in tree and '选择当前目录' in tree)
            record['directory_mode_screenshot'] = session.screenshot('directory-mode')
            session.press('取消'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
            session.open(mode='保存文件')
            session.wait(lambda tree: '选择保存位置' in tree and '要保存的文件名' in tree)
            record['save_mode_screenshot'] = session.screenshot('save-mode')
            session.save_name('覆盖目标.md')
            session.press('保存')
            session.wait(lambda tree: '替换现有文件?' in tree)
            record['overwrite_screenshot'] = session.screenshot('overwrite-layer')
            session.press('返回修改')
            session.wait(lambda tree: '替换现有文件?' not in tree and '要保存的文件名' in tree)
            session.press('取消'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
            session.press('切换明暗')
            session.open(mode='选择目录')
            session.wait(lambda tree: '选择一个目录' in tree and '选择当前目录' in tree)
            record['directory_mode_dark_screenshot'] = session.screenshot('directory-mode-dark')
            session.press('取消'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
            session.open(mode='保存文件')
            session.wait(lambda tree: '选择保存位置' in tree and '要保存的文件名' in tree)
            session.save_name('覆盖目标.md')
            session.press('保存')
            session.wait(lambda tree: '替换现有文件?' in tree)
            record['overwrite_dark_screenshot'] = session.screenshot('overwrite-layer-dark')
            session.press('返回修改')
            session.wait(lambda tree: '替换现有文件?' not in tree and '要保存的文件名' in tree)
            session.press('取消'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
            session.close(); report['sessions'].append(record)
            report['outcome'] = 'pass'
            print(json.dumps(report,ensure_ascii=False,indent=2))
            return
        if os.name == 'nt':
            record['system_accessibility'] = session.system_accessibility()
        error, _ = session.call('autoui_action',allow_error=True,element_id=session.node('底层计数'),action='press')
        require(error and session.count('底层次数') == 0,'Modal background action escaped')
        stale = session.toggle('a.txt'); session.selected(1)
        session.key('Space'); session.selected(0)
        session.key('Tab'); session.key('Space'); session.selected(1)
        session.key('Tab',modifiers=['Shift']); session.key('Space'); session.selected(2)
        session.press("进入 中文 子目录'引号")
        session.wait(lambda tree: "中文 子目录'引号" in tree and 'b.MD' not in tree)
        error, _ = session.call('autoui_action',allow_error=True,element_id=stale,action='toggle')
        require(error,'A stale parent-directory row selected a child with the same name')
        session.selected(0); session.toggle('a.txt'); session.selected(1)
        record['accepted_child'] = session.accepted([child/'a.txt'])
        record['restored_focus_screenshot'] = session.screenshot('focus-restored')
        session.press('底层计数'); session.wait(lambda tree:'底层次数 1' in tree)
        session.open()
        session.toggle('b.MD'); session.selected(1)
        session.toggle('a.txt'); session.selected(2)
        record['accepted_multiple'] = session.accepted([fixture/'a.txt',fixture/'b.MD'])
        session.open()
        session.toggle('a.txt'); session.selected(1)
        session.press('图片')
        session.wait(lambda tree:'"cover.png"' in tree and '"a.txt"' not in tree and '已选 0 项' in tree)
        session.press('书籍'); session.wait(lambda tree:'"a.txt"' in tree)
        session.input('名称过滤','no-match')
        session.wait(lambda tree:'没有匹配的文件' in tree and '"a.txt"' not in tree and "进入 中文 子目录'引号" in tree)
        session.input('名称过滤',''); session.wait(lambda tree:'"a.txt"' in tree)
        before_error = session.output.read_bytes()
        session.navigate(fixture/'missing-directory','重试')
        session.wait(lambda tree:'not_found' in tree and '错误事件' in tree)
        require(session.output.read_bytes()==before_error,'Recoverable error published acceptance')
        session.input('目录路径',fixture); session.press('重试')
        session.wait(lambda tree:'重试' not in tree and '"a.txt"' in tree)
        session.press('取消'); session.wait(lambda tree:'取消次数 1' in tree and 'placeholder: "目录路径"' not in tree)
        record['type_name_error_retry'] = 'pass; exact visible files and directories, selection cleared, public not_found error, unchanged durable acceptance, legal retry'
        before = session.output.read_bytes()
        cancel_before = session.count('取消次数')
        for cycle in range(10):
            session.open()
            session.key('Escape') if cycle % 2 == 0 else session.press('取消')
            session.wait(lambda tree:f'取消次数 {cancel_before+cycle+1}' in tree and 'placeholder: "目录路径"' not in tree)
            require(session.output.read_bytes() == before,'Cancellation changed accepted data')
        record['cancel_reopen_cycles'] = 10
        session.open(single=True)
        session.toggle('a.txt'); session.selected(1)
        session.toggle('b.MD'); session.selected(1)
        record['accepted_single'] = session.accepted([fixture/'b.MD'])
        # ---- PLAN-084 V-01: directory mode selects exactly one real directory ----
        session.open(mode='选择目录')
        session.wait(lambda tree: '选择一个目录' in tree and '选择当前目录' in tree)
        require(not re.search(r'checkbox #[^ ]+ "a\.txt"', session.snapshot()),
                'directory mode must not offer file selection')
        session.press('选择当前目录'); session.selected(1)
        record['accepted_directory'] = session.accepted([fixture], label='选择此目录')
        session.open(mode='选择目录')
        session.press("选择 中文 子目录'引号"); session.selected(1)
        record['accepted_child_directory'] = session.accepted([child], label='选择此目录')
        # ---- PLAN-084 V-02: save names, named errors, no-write boundary ----
        session.open(mode='保存文件')
        session.wait(lambda tree: '选择保存位置' in tree and '要保存的文件名' in tree)
        save_new = fixture/'新报告 v084.txt'
        session.save_name('新报告 v084.txt')
        record['accepted_save_new'] = session.accepted([save_new], label='保存')
        require(not save_new.exists(), 'the selector must not create the save target')
        accepted_before = session.count('确认次数')
        session.save_name('不匹配.xyz')
        session.press('保存')
        session.wait(lambda tree: '扩展名' in tree)
        require(session.count('确认次数') == accepted_before, 'extension mismatch must not accept')
        require('不匹配.xyz' in session.snapshot(), 'draft input must survive the error')
        session.save_name('a/b.txt')
        session.press('保存')
        session.wait(lambda tree: '路径分隔符' in tree)
        require(session.count('确认次数') == accepted_before, 'separator names must not accept')
        record['save_named_errors'] = 'extension mismatch + path separator; draft preserved; no acceptance'
        # ---- PLAN-084 V-03: overwrite intent, layer isolation, Esc-Back ----
        before_background = session.count('底层次数')
        session.output.unlink(missing_ok=True)
        overwrite_bytes_before = overwrite_target.read_bytes()
        session.save_name('覆盖目标.md')
        session.press('保存')
        session.wait(lambda tree: '替换现有文件?' in tree and '返回修改' in tree)
        error, _ = session.call('autoui_action', allow_error=True, element_id=session.node('底层计数'), action='press')
        require(error and session.count('底层次数') == before_background,
                'overwrite layer must reject background actions')
        session.key('Escape')
        session.wait(lambda tree: '替换现有文件?' not in tree and '要保存的文件名' in tree)
        require('覆盖目标.md' in session.snapshot(), 'Back must keep the draft')
        session.press('保存')
        session.wait(lambda tree: '替换现有文件?' in tree)
        session.press('确认覆盖')
        session.wait(lambda tree: f'确认次数 {accepted_before+1}' in tree and '要保存的文件名' not in tree)
        record['accepted_overwrite'] = json.loads(session.output.read_text(encoding='utf-8'))['files']
        require(overwrite_target.read_bytes() == overwrite_bytes_before,
                'the selector never writes the confirmed target')
        require(session.count('底层次数') == before_background,
                'layer rejection left the background untouched')
        session.open()
        started = time.monotonic()
        session.navigate(large,'中文长文件名_0000_')
        record['directory_ready_seconds'] = time.monotonic()-started
        require(record['directory_ready_seconds'] < 2,'1000-entry directory exceeded frozen 2s budget')
        seen = set()
        for page in range(11):
            tree = session.wait(lambda tree:f'第 {page+1} / 11 页' in tree)
            names = re.findall(r'checkbox #[^ ]+ "(中文长文件名_[^\"]+)"',tree)
            require(len(names) == (100 if page < 10 else 3),(page,len(names)))
            seen.update(names)
            if page < 10: session.press('下一页')
        require(seen == {path.name for path in large.iterdir()},'Paged visible set differs from independent directory')
        tail = large/'中文长文件名_1002_分页完整性.txt'
        session.toggle(tail.name); session.selected(1)
        session.call('autoui_action', element_id=session.file_list_scroll(), action='scroll', value=500)
        record['tail_screenshot'] = session.screenshot('tail')
        record['accepted_tail'] = session.accepted([tail])
        session.press('切换明暗')
        session.open(); record['alternate_theme_screenshot'] = session.screenshot('alternate-theme')
        session.press('关闭'); session.wait(lambda tree:'placeholder: "目录路径"' not in tree)
        session.close(); report['sessions'].append(record)
        # Fresh session with save knobs: an explicit defaultFileName wins over
        # a file-path initialDirectory basename; a bare name completes with
        # the default extension in the accepted result only.
        os.environ['FILE_DIALOG_INITIAL_DIRECTORY'] = str(overwrite_target)
        os.environ['FILE_DIALOG_DEFAULT_NAME'] = '显式默认.md'
        os.environ['FILE_DIALOG_DEFAULT_EXT'] = 'md'
        session = Session(args,app,out,2)
        record = {'command':session.command,'windows':session.windows,'pid':session.process.pid}
        session.open(); session.toggle('a.txt'); session.selected(1)
        record['fresh_accepted'] = session.accepted([fixture/'a.txt'])
        session.open(mode='保存文件')
        session.wait(lambda tree: '选择保存位置' in tree)
        require('显式默认.md' in session.snapshot(),
                'explicit defaultFileName must win over the file-path basename')
        session.save_name('补全测试')
        record['accepted_save_completion'] = session.accepted([fixture/'补全测试.md'], label='保存')
        require(not (fixture/'补全测试.md').exists(), 'the selector must not create the completed target')
        session.close(); report['sessions'].append(record)
        # The whole campaign must leave every fixture byte untouched.
        after = {str(path.relative_to(fixture)): sha(path) for path in sorted(fixture.rglob('*')) if path.is_file()}
        require(after == baseline, 'fixture bytes changed during the campaign: ' +
                str({key for key in set(after) | set(baseline) if after.get(key) != baseline.get(key)}))
        report['fixture_byte_baseline'] = 'unchanged'
        report['outcome'] = 'pass'
    except Exception as error:
        report['outcome'] = 'fail'; report['failure'] = repr(error)
        if session is not None: session.cleanup_failure()
        raise
    finally:
        (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False,indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin',type=Path,required=True)
    parser.add_argument('--ui-tool-bin',type=Path,default=Path(os.environ.get('AUTO_UI_TOOL_BIN',str(Path(__file__).resolve().parents[2]/'target/debug'/('auto-ui-tool.exe' if os.name=='nt' else 'auto-ui-tool')))))
    parser.add_argument('--out-dir',type=Path,required=True)
    parser.add_argument('--window',default='900x700')
    parser.add_argument('--index-repo',help='Optional existing registry mirror URL for offline CLI initialization')
    parser.add_argument('--visual-only',action='store_true',help='Scoped narrow-window/theme/visible-footer check; does not claim the full native gate')
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve(); args.ui_tool_bin = args.ui_tool_bin.resolve()
    run(args)
