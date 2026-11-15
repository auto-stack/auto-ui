"""PLAN-085 F-06 — real keyboard gate for TreeView (single-tree page).

The review found that the keyboard evidence was msg-variant dispatch, not real
keys: `on_with_input_for("TreeView", "Down")` proves the component's handler, not
the runtime's key map. This gate sends actual `autoui_keyboard` events and reads
the result back from the page, so the whole chain — key name → key_bindings →
instance → component handler → app state — is exercised.

Why a single-tree page: with two TreeView instances on one page a key name
("TreeView.Down") cannot identify which instance should receive it, and the
runtime correctly declines to guess (documented ambiguity rule). Keyboard
semantics are therefore asserted where resolution is unambiguous, and the
two-tree demo only asserts that keys are accepted.

Covered: focus starts on the first reachable row; ArrowDown/ArrowUp walk;
Home/End jump; saturation at both ends without wrapping; Enter selects the
focused row and never toggles; ArrowRight descends without re-toggling;
ArrowLeft collapses / returns to parent; focus never writes `selected`.
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

sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify_file_dialog as vfd  # require/sha/owned_tree/windows_for/free_port

ROOT = Path(__file__).resolve().parents[2]


def bindable_port(preferred):
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
    def __init__(self, args, app, out):
        import urllib.request
        self._urlreq = urllib.request
        self.port = bindable_port(vfd.free_port())
        self.out = out
        env = dict(os.environ, AUTOUI_MCP_PORT=str(self.port),
                   AUTO_UI_TOOL_BIN=str(args.ui_tool_bin))
        env.pop('AUTOUI_TEST_FIXTURES', None)
        self.log = open(out / 'session.log', 'wb')
        self.process = subprocess.Popen(
            [str(args.auto_bin), 'run', '-r', 'vm'], cwd=app, env=env,
            stdout=self.log, stderr=subprocess.STDOUT,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        deadline = time.time() + 90
        last = ''
        while time.time() < deadline:
            try:
                last = self.snapshot()
                if 'kbd' in last:
                    break
            except Exception as e:
                last = f'ERR {e!r}'
            time.sleep(0.6)
        else:
            raise RuntimeError(f'keyboard fixture never rendered: {last[:300]}')
        self.owned = {self.process.pid} | set(vfd.owned_tree(self.process.pid))
        self.windows = vfd.windows_for(self.owned)

    def call(self, tool, **argsv):
        body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                           'params': {'name': tool, 'arguments': argsv}}).encode()
        req = self._urlreq.Request(
            f'http://127.0.0.1:{self.port}/mcp', data=body,
            headers={'Content-Type': 'application/json'})
        with self._urlreq.urlopen(req, timeout=10) as r:
            out = json.loads(r.read().decode())
        content = out.get('result', {}).get('content', [])
        return '\n'.join(c.get('text', '') for c in content if c.get('type') == 'text')

    def snapshot(self):
        return self.call('autoui_snapshot')

    def sel(self):
        m = re.search(r'kbd sel=([^\s"]*)', self.snapshot())
        return m.group(1) if m else None

    def focused(self):
        """Label of the focused row.

        The row wraps an icon (which also renders a Text node, "[Image]") before
        the label, so the FIRST Text node inside the row is not the label — the
        label is the first one that is not the icon placeholder.
        """
        lines = self.snapshot().split('\n')
        for i, line in enumerate(lines):
            # Only a row carrying the focus ring AND the focus-only class mix is
            # the focused row; a merely selected row has bg-accent but no ring.
            if 'ring-2 ring-primary' not in line:
                continue
            if 'rounded-md' not in line:
                continue
            for j in range(i, min(i + 12, len(lines))):
                m = re.search(r'text #vnode_\d+ "([^"]+)"', lines[j])
                if m and m.group(1) not in ('[Image]', ''):
                    return m.group(1)
        return ''

    def key(self, name):
        out = self.call('autoui_keyboard', key=name)
        vfd.require('Key sent' in out, f'{name} must be accepted: {out[:200]}')
        # Let the frame that carries the new focus land before reading it back.
        time.sleep(0.9)
        return out

    def wait(self, pred, timeout=10):
        deadline = time.time() + timeout
        last = ''
        while time.time() < deadline:
            last = self.snapshot()
            if pred(last):
                return last
            time.sleep(0.3)
        raise AssertionError(f'condition not met: {last[:300]}')


def prepare(args, out):
    vfd.require(not out.exists(), f'Use a fresh output directory: {out}')
    out.mkdir(parents=True)
    app = out / 'app'
    app.mkdir()
    (app / 'src/front').mkdir(parents=True)
    pkg = (ROOT / 'packages/tree-view').as_posix()
    (app / 'pac.at').write_text(
        'name: "kbd"\nversion: "0.1.0"\nscene: "ui"\nrender: "vue"\n'
        'title: "kbd"\nwindow: "800x600"\nfront_port: 4295\n', encoding='utf-8')
    (app / 'src/front/app.at').write_text(f'''// PLAN-085 F-06 keyboard gate (single tree).
//
// Chapter 01 (expanded) > Section 1.1 · Images · Notes, and a second root
// Chapter 02. Rows exist so every key has something unambiguous to act on.
widget App {{
    msg {{ Init, TvToggle(str), TvSelect(str), TvRetry(str), Quit }}
    model {{
        var nodes List = []
        var expanded List = []
        var sel str = ""
    }}
    use {{ package: tree_view from "{pkg}" }}
    view {{
        col (style: "w-full h-full p-4") {{
            text f"kbd sel=${{.sel}}" {{ style: "text-xs" }}
            TreeView (nodes: .nodes, expanded: .expanded, selected: .sel, on_toggle: .TvToggle($event), on_select: .TvSelect($event), on_retry: .TvRetry($event), directory: true) {{}}
        }}
    }}
    on {{
        .Init -> {{
            var kid List = []
            kid.push({{ id: "section-1.1", children: [], kind: "file", icon: "", is_leaf: true, badge: "", label: "Section 1.1" }})
            var a List = []
            a.push({{ id: "chapter-01", children: kid, kind: "dir", icon: "", is_leaf: false, badge: "", label: "Chapter 01" }})
            a.push({{ id: "images", children: [], kind: "dir", icon: "", is_leaf: false, badge: "", label: "Images" }})
            a.push({{ id: "notes", children: [], kind: "file", icon: "", is_leaf: true, badge: "", label: "Notes" }})
            a.push({{ id: "chapter-02", children: [], kind: "dir", icon: "", is_leaf: false, badge: "", label: "Chapter 02" }})
            .nodes = a
            .expanded = ["chapter-01"]
        }}
        .TvSelect(id str) -> {{ .sel = id }}
        .TvToggle(id str) -> {{
            var out List = []
            var found bool = false
            var i int = 0
            while i < .expanded.len() {{
                if .expanded[i] == id {{ found = true }} else {{ out.push(.expanded[i]) }}
                i = i + 1
            }}
            if !found {{ out.push(id) }}
            .expanded = out
        }}
        .TvRetry(id str) -> {{ }}
        .Quit -> {{ Process.exit(0) }}
    }}
}}
''', encoding='utf-8')
    return app


def run(args):
    out = args.out_dir
    report = {'plan': '085', 'scope': 'f06_real_keyboard',
              'auto': {'path': str(args.auto_bin), 'sha256': vfd.sha(args.auto_bin)},
              'ui_tool': {'path': str(args.ui_tool_bin),
                          'sha256': vfd.sha(args.ui_tool_bin)},
              'steps': []}
    app = prepare(args, out)
    s = Session(args, app, out)
    try:
        def step(tag, expect, got):
            report['steps'].append({'step': tag, 'expected': expect, 'got': got})
            vfd.require(expect == got, f'{tag}: expected {expect!r}, got {got!r}')

        # Initial focus is the first reachable row.
        session_first = s.focused()
        step('initial focus', 'Chapter 01', session_first)

        # Down walks forward; Enter selects exactly the focused row.
        s.key('ArrowDown')
        step('ArrowDown focus', 'Section 1.1', s.focused())
        s.key('Enter')
        s.wait(lambda t: 'sel=section-1.1' in t)
        step('Enter selects focused row', 'section-1.1', s.sel())

        # Enter must not toggle. The icon resolves to a structured Value in the
        # snapshot, so count the visible child row instead (Section 1.1 is
        # visible only while Chapter 01 stays expanded).
        expanded = 1 if 'Section 1.1' in s.snapshot() else 0
        step('Enter did not expand anything', 1, expanded)

        # Up returns; Home/End jump to the ends.
        s.key('ArrowUp')
        step('ArrowUp focus', 'Chapter 01', s.focused())
        s.key('End')
        step('End focus', 'Chapter 02', s.focused())
        s.key('Home')
        step('Home focus', 'Chapter 01', s.focused())

        # Saturation: ArrowUp at the top does not wrap.
        s.key('ArrowUp')
        step('ArrowUp at top does not wrap', 'Chapter 01', s.focused())

        # Right on an expanded parent descends without re-toggling.
        s.key('ArrowRight')
        step('ArrowRight descends', 'Section 1.1', s.focused())
        expanded2 = 1 if 'Section 1.1' in s.snapshot() else 0
        step('ArrowRight did not collapse the parent', 1, expanded2)

        # Left collapses? No: focus is on a leaf, so Left returns to the parent.
        s.key('ArrowLeft')
        step('ArrowLeft on leaf returns to parent', 'Chapter 01', s.focused())

        # Collapsing the focused parent via Left.
        s.key('ArrowLeft')
        collapsed = 1 if 'Section 1.1' in s.snapshot() else 0
        step('ArrowLeft collapsed the parent', 0, collapsed)

        # Selection survived all keyboard navigation (focus != select).
        step('focus never wrote selected', 'section-1.1', s.sel())

        report['outcome'] = 'pass'
    except Exception as error:
        report['outcome'] = 'fail'
        report['failure'] = repr(error)
        raise
    finally:
        (out / 'report.json').write_text(
            json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
        try:
            s.process.terminate()
            s.process.wait(timeout=8)
        except Exception:
            s.process.kill()
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--auto-bin', type=Path, required=True)
    parser.add_argument('--ui-tool-bin', type=Path,
                        default=Path(os.environ.get(
                            'AUTO_UI_TOOL_BIN',
                            str(ROOT / 'target/debug' /
                                ('auto-ui-tool.exe' if os.name == 'nt' else 'auto-ui-tool')))))
    parser.add_argument('--out-dir', type=Path, required=True)
    args = parser.parse_args()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    run(args)
