#!/usr/bin/env python3
"""PLAN-084 consumer gate: 041 auto-edit saves through the real FileDialog.

Drives ordinary visible controls only: Ctrl+S / menubar 另存为, the FileDialog's
own filename input, Back/Replace in the overwrite layer, and the documented
quit path. The app owns the write and its failure feedback; the selector must
never create or modify bytes. No Env path shortcut participates
(AUTO_SAVE_PATH / AUTO_OPEN_PATH are absent by construction and asserted).
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd


def require(condition, detail):
    vfd.require(condition, detail)


def state_str(text, field):
    match = re.search(rf'{re.escape(field)}:\s*"((?:[^"\\]|\\.)*)"', text)
    if match is None:
        return None
    try:
        return json.loads(f'"{match.group(1)}"')
    except Exception:
        return match.group(1)


def state_int(text, field):
    match = re.search(rf"{re.escape(field)}:\s*(-?\d+)", text)
    return int(match.group(1)) if match else None


class ConsumerSession(vfd.Session):
    """The fixture session entry waits for the FileDialog fixture page; the
    consumer waits for the editor workspace instead."""

    def __init__(self, args, app, out, index):
        self.port = vfd.free_port()
        self.trace = []
        self.out = out
        self.index = index
        self.output = out / f'consumer-{index}.json'
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port),
                   AUTO_UI_TOOL_BIN=str(args.ui_tool_bin),
                   AUTO_PROJECT_DIR=str(app))
        for shortcut in ('AUTO_SAVE_PATH', 'AUTO_OPEN_PATH'):
            env.pop(shortcut, None)
            require(shortcut not in os.environ, f'{shortcut} must not be set for this gate')
        if os.name != 'nt':
            env['WINIT_UNIX_BACKEND'] = 'x11'
            env.pop('WAYLAND_DISPLAY', None)
        self.log = open(out / f'consumer-{index}.log', 'wb')
        self.command = [str(args.auto_bin), 'run', '-r', 'vm']
        self.process = subprocess.Popen(self.command, cwd=app, env=env, stdout=self.log,
                                        stderr=subprocess.STDOUT,
                                        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        self.owned = {self.process.pid}
        try:
            self.wait(lambda tree: 'EXPLORER' in tree, timeout=90)
            self.owned |= set(vfd.owned_tree(self.process.pid))
            self.windows = vfd.windows_for(self.owned)
            require(self.windows, 'No owned native window with a title/OS handle')
        except Exception:
            self.cleanup_failure()
            raise

    def state(self, *fields):
        return self.call('autoui_state', fields=list(fields))

    def dirty_marks(self):
        return len(re.findall(r'text #[^ ]+ "\*"', self.snapshot()))

    def edits(self):
        return state_int(self.state('edits'), 'edits')

    def title(self):
        return state_str(self.state('title_active'), 'title_active')

    def console(self):
        return state_str(self.state('console'), 'console') or ''

    def dirt_by_cut(self):
        """The toolbar cut button is the documented dirty lever (its store
        handler also re-syncs src_active); ids drift, so rediscover always."""
        snapshot = self.snapshot()
        match = re.search(r'button #((?:vnode|aura)_\d+) "[^"]*\uee01scissors\uee02[^"]*"', snapshot)
        require(match, 'scissors toolbar button missing')
        self.call('autoui_action', element_id=match[1], action='press')
        deadline = time.monotonic() + 5
        before = None
        while time.monotonic() < deadline:
            current = self.edits()
            if current is not None and (before is None or current > before):
                if before is not None:
                    return current
                before = current
            time.sleep(0.05)
        raise AssertionError('cut did not raise the edit counter')

    def save_dialog(self):
        self.wait(lambda tree: '选择保存位置' in tree and '要保存的文件名' in tree)
        require('placeholder: "要保存的文件名"' in self.snapshot() or '要保存的文件名' in self.snapshot(),
                'save dialog must expose its filename input')

    def type_and_confirm(self, name):
        self.input('要保存的文件名', name)
        self.key('Enter')
        return self.wait(lambda tree: '载入中' not in tree)

    def close(self):
        """Documented quit: menubar 退出 with dirty tabs opens the quit
        confirm; 不保存退出 ends the process. Reaping and port refusal stay
        mandatory."""
        self.owned |= set(vfd.owned_tree(self.process.pid))
        self.press('文件')
        self.wait(lambda tree: '"退出"' in tree or '退出' in tree)
        self.press('退出')
        time.sleep(0.5)
        if self.process.poll() is None:
            tree = self.snapshot()
            if '不保存退出' in tree:
                self.press('不保存退出')
            try:
                code = self.process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                raise AssertionError('Documented quit did not end the process')
            require(code == 0, f'Documented quit must exit cleanly, got {code}')
        self.log.close()
        reaping = time.monotonic()
        remaining = self.owned & set(vfd.processes())
        while remaining and time.monotonic() - reaping < 5:
            time.sleep(0.05)
            remaining = self.owned & set(vfd.processes())
        self.trace.append({'quit_reap_seconds': time.monotonic() - reaping,
                           'remaining': sorted(remaining)})
        require(not remaining, f'Owned processes survived quit: {remaining}')
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': 'autoui_snapshot', 'arguments': {}}}).encode()
        try:
            urllib.request.urlopen(urllib.request.Request(f'http://127.0.0.1:{self.port}/mcp', data=body), timeout=5)
            raise AssertionError('Closed MCP still answered')
        except urllib.error.URLError as error:
            reason = error.reason
            require(isinstance(reason, ConnectionRefusedError) or getattr(reason, 'winerror', None) == 10061,
                    f'Negative control must be connection refused: {error}')
        self.save()


def run(args):
    root = Path(__file__).resolve().parents[2]
    out = args.out_dir.resolve()
    require(not out.exists(), f'Use a fresh output directory: {out}')
    out.mkdir(parents=True)
    app_source = root / 'examples/ui/041-auto-edit'
    app = out / 'app'
    shutil.copytree(app_source, app, ignore=shutil.ignore_patterns('deps', 'dist', 'gen', 'build'))
    entry = app / 'src/front/app.at'
    text = entry.read_text(encoding='utf-8')
    require('../../../../../packages/file-dialog' in text, 'expected the official package import')
    entry.write_text(text.replace('../../../../../packages/file-dialog',
                                  (root / 'packages/file-dialog').as_posix()), encoding='utf-8')
    existing = app / '已有文档.md'
    existing.write_text('consumer gate existing bytes', encoding='utf-8')
    readonly = app / '只读文档.md'
    readonly.write_text('consumer gate readonly bytes', encoding='utf-8')
    os.chmod(readonly, 0o444)
    probe = readonly.open('a')
    try:
        probe.write('x')
        raise AssertionError('run this gate as a user that cannot bypass the read-only attribute')
    except PermissionError:
        pass
    finally:
        probe.close()
    readonly_bytes = readonly.read_bytes()
    existing_bytes = existing.read_bytes()

    report = {'platform': sys.platform, 'scope': 'file_dialog_modes_consumer',
              'auto': {'path': str(args.auto_bin), 'sha256': vfd.sha(args.auto_bin)},
              'ui_tool': {'path': str(args.ui_tool_bin), 'sha256': vfd.sha(args.ui_tool_bin)},
              'workspace': str(app), 'checks': []}
    session = None

    def check(name, condition, detail=''):
        report['checks'].append({'name': name, 'ok': bool(condition), 'detail': str(detail)[:400]})
        require(condition, f'{name}: {detail}')

    try:
        session = ConsumerSession(args, app, out, 1)
        record = {'command': session.command, 'windows': session.windows, 'pid': session.process.pid}
        # ---- V-05: dirty the untitled tab, Ctrl+S opens the real dialog ----
        session.dirt_by_cut()
        require(session.dirty_marks() >= 1, 'dirty tab must show its * marker')
        expected_src = state_str(session.state('src_active'), 'src_active') or ''
        require(expected_src, 'editor content must be readable from state')
        session.key('s', modifiers=['ctrl'])
        session.save_dialog()
        # Background stays unreachable while the modal owns input.
        plus = re.search(r'button #((?:vnode|aura)_\d+) "[^"]*\uee01plus\uee02[^"]*"', session.snapshot())
        if plus:
            error, _ = session.call('autoui_action', allow_error=True, element_id=plus[1], action='press')
            check('background action rejected by the modal',
                  error or session.title() != 'WRONG', f'error={error}')
        # ---- save a new file through the ordinary handler ----
        target = app / '保存的文档.md'
        session.type_and_confirm('保存的文档.md')
        session.wait(lambda tree: '要保存的文件名' not in tree)
        require(target.exists(), 'the app must write the accepted path')
        written = target.read_text(encoding='utf-8')
        first_line = expected_src.splitlines()[0] if expected_src.splitlines() else expected_src
        check('saved bytes come from the editor', bool(written) and first_line in written,
              f'{first_line[:40]!r} vs {written[:40]!r}')
        check('path adopted by the store', session.title() == '保存的文档.md', session.title())
        check('dirty cleared after a verified save', session.dirty_marks() == 0, session.dirty_marks())
        check('console records the save', 'saved:' in session.console(), session.console()[-200:])
        # ---- 另存为 keeps the original file and writes a new path ----
        session.dirt_by_cut()
        session.press('文件')
        session.wait(lambda tree: '另存为' in tree)
        session.press('另存为…')
        session.save_dialog()
        require('保存的文档.md' in session.snapshot(), '另存为 must prefill the current basename')
        save_as_target = app / '另存为版本.at'
        session.type_and_confirm('另存为版本.at')
        session.wait(lambda tree: '要保存的文件名' not in tree)
        require(save_as_target.exists(), '另存为 must write the new path')
        check('另存为 switched the active path', session.title() == '另存为版本.at', session.title())
        check('original save untouched by 另存为', target.read_text(encoding='utf-8') == written, 'bytes changed')
        # ---- overwrite: Back keeps bytes, Replace hands the write to the app ----
        session.dirt_by_cut()
        session.type_and_confirm('已有文档.md')
        session.wait(lambda tree: '替换现有文件?' in tree and '返回修改' in tree)
        require(existing.read_bytes() == existing_bytes, 'validation must not write')
        session.press('返回修改')
        session.wait(lambda tree: '替换现有文件?' not in tree and '要保存的文件名' in tree)
        check('Back keeps the draft', '已有文档.md' in session.snapshot(), 'draft lost')
        check('Back keeps the target bytes', existing.read_bytes() == existing_bytes, 'bytes changed on Back')
        session.type_and_confirm('已有文档.md')
        session.wait(lambda tree: '替换现有文件?' in tree)
        session.press('确认覆盖')
        session.wait(lambda tree: '要保存的文件名' not in tree)
        replaced = existing.read_text(encoding='utf-8')
        check('Replace lets the app write the editor content',
              bool(replaced) and replaced != 'consumer gate existing bytes', replaced[:40])
        check('overwrite save clears dirty', session.dirty_marks() == 0, session.dirty_marks())
        # ---- cancel keeps the draft dirty and creates nothing ----
        session.dirt_by_cut()
        session.type_and_confirm('取消文档.md')
        session.press('取消')
        session.wait(lambda tree: '要保存的文件名' not in tree)
        require(not (app / '取消文档.md').exists(), 'cancellation must not create files')
        check('cancel keeps the dirty state', session.dirty_marks() >= 1, session.dirty_marks())
        check('cancel keeps the store path', session.title() == '另存为版本.at', session.title())
        # ---- read-only target: named recoverable failure, dirty retained ----
        session.dirt_by_cut()
        session.type_and_confirm('只读文档.md')
        session.wait(lambda tree: '保存目标为只读' in tree)
        check('readonly target rejected by name', 'permission_denied' in session.console() or '保存目标为只读' in session.snapshot(),
              session.console()[-200:])
        check('failed save keeps old bytes', readonly.read_bytes() == readonly_bytes, 'readonly file changed')
        check('failed save keeps dirty', session.dirty_marks() >= 1, session.dirty_marks())
        session.press('取消')
        session.wait(lambda tree: '要保存的文件名' not in tree)
        record['checks'] = len(report['checks'])
        session.close()
        report['sessions'].append(record)
        report['outcome'] = 'pass'
    except Exception as error:
        report['outcome'] = 'fail'
        report['failure'] = repr(error)
        if session is not None:
            session.cleanup_failure()
        raise
    finally:
        try:
            os.chmod(readonly, 0o644)
        except OSError:
            pass
        (out / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path, required=True)
    parser.add_argument('--out-dir', type=Path, required=True)
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    args.window = '960x640'
    run(args)
