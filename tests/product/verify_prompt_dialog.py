"""Windows product checks for the prepared PromptDialog demo or File Manager."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import socket
import subprocess
import time
import urllib.error
import urllib.request

from verify_collection_view import BrowserMcp, browser_button, native_button, native_call, stop_tree, wait_for
from verify_file_dialog import free_port, owned_tree, windows_for
from verify_pagination import bindable_port


def available_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


class ProductSession:
    def __init__(self, args, output, journal):
        self.args = args
        self.output = output
        self.journal = journal
        self.port = bindable_port(free_port()) if args.backend == "native" else available_port()
        self.process = None
        self.backend_process = None
        self.system_helper = None
        self.backend_port = None
        self.browser = None
        self.last_snapshot = ""
        self.windows = []
        self.owned = {}
        self.logs = []
        self.ready = False

    def start(self):
        log = self.log_file("service.log")
        if self.args.case == "system":
            script = self.output / "system-tree.ps1"
            script.write_text(SYSTEM_READER, encoding="utf-8-sig")
            self.system_helper = subprocess.Popen(
                ["powershell.exe", "-NoProfile", "-STA", "-File", str(script)],
                stdout=subprocess.PIPE, stderr=self.log_file("system-tree.log"),
                creationflags=subprocess.CREATE_NO_WINDOW,
            )
        env = os.environ.copy()
        if self.args.consumer == "file-manager":
            env["USERPROFILE"] = str(self.args.fixture_dir)
            env["HOME"] = str(self.args.fixture_dir)
            self.backend_port = available_port()
            env["AUTO_HTTP_PORT"] = str(self.backend_port)
            env["AUTO_HTTP_BASE"] = f"http://127.0.0.1:{self.backend_port}"
            env["AUTO_HTTP_PROXY"] = env["AUTO_HTTP_BASE"]
            backend_log = self.log_file("api.log")
            if self.args.backend == "native":
                backend_command = [self.args.api_bin]
                env["AUTO_REUSE_BACKEND"] = "require"
            else:
                backend_command = [self.args.auto_bin, "service", "--server", "vm",
                                   "--back-port", str(self.backend_port)]
            self.backend_process = subprocess.Popen(
                backend_command, cwd=self.args.api_dir or self.args.app_dir, env=env,
                stdout=backend_log, stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW,
            )
            until = time.monotonic() + self.args.startup_timeout
            while not self.port_open(self.backend_port):
                if self.backend_process.poll() is not None or time.monotonic() >= until:
                    raise RuntimeError("prepared File Manager API did not start")
                time.sleep(0.04)
        if self.args.backend == "native":
            command = [self.args.auto_bin, "run", "-r", "vm"]
            if self.backend_port is not None:
                command += ["--back-port", str(self.backend_port)]
            self.process = subprocess.Popen(
                command,
                cwd=self.args.app_dir,
                env={**env, "AUTOUI_MCP_PORT": str(self.port),
                     "AUTO_UI_TOOL_BIN": str(self.args.ui_tool_bin),
                     "AUTO_VM_STORAGE_FILE": str(self.output / "storage.json")},
                stdout=log, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW,
            )
            listening = re.compile(r"AutoUI MCP: listening on http://127\.0\.0\.1:(\d+)")
            startup_log = wait_for(
                lambda: Path(log.name).read_text(encoding="utf-8", errors="replace"),
                lambda text: bool(listening.search(text)),
                seconds=self.args.startup_timeout,
            )
            self.port = int(listening.search(startup_log)[1])
            title = "名称管理" if self.args.consumer == "demo" else "主目录"
            with ThreadPoolExecutor(max_workers=1) as discovery:
                process_tree = discovery.submit(owned_tree, self.process.pid)
                wait_for(self.snapshot, lambda tree: title in tree, seconds=self.args.startup_timeout)
                self.owned = process_tree.result(timeout=self.args.startup_timeout)
            if len(self.owned) < 2:
                self.owned = owned_tree(self.process.pid)
            self.windows = wait_for(lambda: windows_for(self.owned), bool,
                                    seconds=self.args.startup_timeout)
        else:
            env["BROWSER"] = "none"
            command = ["node", "node_modules/vite/bin/vite.js", "--port", str(self.port), "--strictPort"]
            self.process = subprocess.Popen(
                command, cwd=self.args.app_dir, env=env, stdout=log, stderr=subprocess.STDOUT,
                creationflags=subprocess.CREATE_NO_WINDOW,
            )
            url = f"http://127.0.0.1:{self.port}/"
            wait_for(lambda: self.read_page(url), lambda body: '<div id="app">' in body,
                     seconds=self.args.startup_timeout)
            config = json.loads(self.args.browser_mcp_config.read_text(encoding="utf-8-sig"))
            self.browser = BrowserMcp(config, self.log_file("browser-mcp.log"), self.output)
            self.provider = self.browser.request("initialize", {
                "protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "verify-prompt-dialog", "version": "1"},
            })
            self.browser.write({"method": "notifications/initialized"})
            self.call("browser_navigate", url=url)
            self.snapshot()
        self.ready = True

    @staticmethod
    def port_open(port):
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                return True
        except OSError:
            return False

    @staticmethod
    def read_page(url):
        with urllib.request.urlopen(url, timeout=1) as page:
            return page.read().decode("utf-8")

    def log_file(self, name):
        log = (self.output / name).open("wb")
        self.logs.append(log)
        return log

    def call(self, tool, **arguments):
        started = time.monotonic()
        data = {"tool": tool, "arguments": arguments}
        try:
            if self.browser is not None:
                result = self.browser.call(tool, reply_timeout=15, **arguments)
            else:
                result = native_call(self.port, tool, timeout=3.5, **arguments)
            data["result"] = result
            return result
        except Exception as error:
            data["error"] = repr(error)
            raise
        finally:
            data["elapsed"] = time.monotonic() - started
            self.journal.write(json.dumps(data, ensure_ascii=False) + "\n")
            self.journal.flush()

    def snapshot(self):
        tool = "browser_snapshot" if self.browser is not None else "autoui_snapshot"
        self.last_snapshot = self.call(tool)
        return self.last_snapshot

    def wait_text(self, *fragments, absent=()):
        return wait_for(self.snapshot, lambda tree: all(part in tree for part in fragments)
                        and not any(part in tree for part in absent), seconds=3)

    def button(self, name):
        tree = self.snapshot()
        if self.browser is not None:
            return browser_button(tree, name)
        exact = [
            node for node, label in re.findall(r'button #(vnode_\d+) "([^"\n]*)"', tree)
            if name in [label.strip(), label.lstrip("\uee03").split("\uee03", 1)[0].strip()]
        ]
        assert len(exact) <= 1, f"Ambiguous button {name!r}: {exact}"
        return exact[0] if exact else native_button(tree, name)

    def press(self, name):
        target = self.button(name)
        if self.browser is not None:
            return self.call("browser_click", element=name, target=target)
        return self.call("autoui_action", element_id=target, action="press")

    def enter_text(self, text, label=None):
        if self.browser is not None:
            name = rf' "{re.escape(label)}"' if label else ""
            pattern = rf'textbox{name}[^\n]*\[ref=(\w+)\]'
            tree = wait_for(self.snapshot, lambda snapshot: bool(re.search(pattern, snapshot)), seconds=3)
            found = re.search(pattern, tree)
            return self.call("browser_type", element=label, target=found[1], text=text)
        native_labels = {
            "笔记本名称": "例如 Reading journal",
            "分组名称": "例如 项目分类",
            "名称": "请输入名称",
        }
        placeholder = native_labels.get(label, label)

        def inputs(snapshot):
            nodes = re.findall(r'input #(vnode_\d+) \{([^}]+)\}', snapshot)
            if placeholder is not None:
                nodes = [node for node in nodes if f'placeholder: "{placeholder}"' in node[1]]
            return nodes

        tree = wait_for(self.snapshot, lambda snapshot: len(inputs(snapshot)) == 1, seconds=3)
        nodes = inputs(tree)
        return self.call("autoui_type", element_id=nodes[0][0], text=text)

    def key(self, key):
        if self.browser is not None:
            return self.call("browser_press_key", key=key)
        return self.call("autoui_keyboard", key=key)

    def screenshot(self, name):
        if self.browser is not None:
            self.call("browser_take_screenshot", type="png", filename=name + ".png")
        else:
            output = self.call("autoui_screenshot")
            source = re.search(r"Screenshot saved to: (.+)", output)
            assert source, output
            (self.output / (name + ".png")).write_bytes(Path(source[1].strip()).read_bytes())

    def close(self, failed):
        try:
            if self.browser is not None:
                try:
                    self.call("browser_close")
                finally:
                    self.browser.close()
            if self.process is not None and self.process.poll() is None:
                if failed or not self.ready:
                    stop_tree(self.process)
                elif (self.args.backend == "native" and self.args.consumer == "demo"
                      and self.args.case != "system"):
                    try:
                        self.press("退出示例")
                    except (ConnectionResetError, TimeoutError):
                        pass
                    assert self.process.wait(timeout=5) == 0, "demo exit failed"
                elif self.ready and self.args.backend == "native":
                    assert len(self.windows) == 1, "Native app must own exactly one visible window"
                    assert ctypes.windll.user32.PostMessageW(self.windows[0]["handle"], 0x0010, 0, 0)
                    assert self.process.wait(timeout=5) == 0, "Native app window exit failed"
                else:
                    stop_tree(self.process)
            if self.args.backend == "native" and self.process is not None:
                deadline = time.monotonic() + 5
                while True:
                    try:
                        self.call("autoui_snapshot")
                    except urllib.error.URLError as error:
                        if isinstance(error.reason, ConnectionRefusedError):
                            break
                    except (ConnectionResetError, TimeoutError):
                        pass
                    assert time.monotonic() < deadline, "native MCP never returned connection refused after exit"
            if self.args.backend == "vue" and self.process is not None:
                assert not self.port_open(self.port), "Vue service port survived cleanup"
        finally:
            if self.process is not None:
                stop_tree(self.process)
            if self.system_helper is not None:
                stop_tree(self.system_helper)
            if self.backend_process is not None:
                stop_tree(self.backend_process)
                assert not self.port_open(self.backend_port), "prepared API port survived cleanup"
            for log in self.logs:
                log.close()


def demo_names(session):
    session.wait_text("notebook-7: Inbox", "notebook-8: Notes")
    session.press("重命名笔记本")
    session.wait_text("Inbox", "重命名笔记本")
    session.enter_text("   ", "笔记本名称")
    session.key("Enter")
    session.wait_text("请输入名称", "notebook-7: Inbox", "已保存 0 次")
    session.enter_text("Notes", "笔记本名称")
    session.key("Enter")
    session.wait_text("名称 Notes 已存在", "notebook-7: Inbox", "已保存 0 次")
    session.enter_text("读书日志", "笔记本名称")
    session.key("Enter")
    session.wait_text("notebook-7: 读书日志", "notebook-8: Notes", "已保存 1 次")
    session.press("重命名分组")
    session.wait_text("分组名称", "Research")
    session.enter_text("  项目分类  ", "分组名称")
    session.press("保存分组")
    session.wait_text("group-1: 项目分类", "notebook-7: 读书日志", "已保存 2 次")


def demo_cancel(session):
    session.press("重命名笔记本")
    session.wait_text("Inbox", "例如 Reading journal")
    old_id = session.button("重命名")
    session.enter_text("Should not survive", "笔记本名称")
    session.key("Escape")
    session.wait_text("已取消笔记本重命名", "notebook-7: Inbox", absent=("例如 Reading journal",))
    if session.browser is not None:
        focused = session.browser.result_value("browser_run_code_unsafe", code="""async (page) => {
          await page.waitForFunction(() => document.activeElement?.textContent?.trim() === '重命名笔记本',
                                     undefined, { timeout: 1500 });
          return await page.evaluate(() =>
            document.activeElement?.textContent?.trim() === '重命名笔记本');
        }""")
        assert focused is True, focused
    session.press("重命名笔记本")
    session.wait_text("Inbox", "例如 Reading journal", absent=("Should not survive",))
    current_id = session.button("重命名")
    assert old_id != current_id, "old confirmation retained its address after reopen"
    if session.browser is None:
        response = session.call("autoui_action", element_id=old_id, action="press")
        assert response.startswith("Error:"), response
    session.press("取消")
    session.wait_text("notebook-7: Inbox", "已保存 0 次")


def demo_tab(session):
    session.press("重命名笔记本")
    session.wait_text("例如 Reading journal")
    if session.browser is None:
        # MCP acknowledges queueing while Iced completes the focus operations
        # on its render loop. Space cannot submit from the starting input.
        time.sleep(0.35)
    session.key("Tab")
    if session.browser is None:
        time.sleep(0.35)
    session.key("Space")
    session.wait_text("已取消笔记本重命名", "notebook-7: Inbox", "已保存 0 次")


def demo_ime(session):
    session.press("重命名笔记本")
    session.wait_text("笔记本名称")
    assert session.browser is not None
    result = session.browser.result_value("browser_run_code_unsafe", code="""async (page) => {
      const box = page.getByRole('textbox', { name: '笔记本名称' });
      return await box.evaluate(el => {
        el.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
        el.dispatchEvent(new CompositionEvent('compositionupdate', { bubbles: true, data: '读书' }));
        el.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', bubbles: true, isComposing: true }));
        const disabledDuringComposition = el.disabled;
        el.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: '读书' }));
        return { disabledDuringComposition };
      });
    }""")
    assert result == {"disabledDuringComposition": False}, result
    session.wait_text("已保存 0 次", "笔记本名称", absent=("正在提交",))
    session.enter_text("读书日志", "笔记本名称")
    session.key("Enter")
    session.wait_text("notebook-7: 读书日志", "已保存 1 次")


def demo_visual(session):
    session.press("重命名笔记本")
    session.enter_text("Notes", "笔记本名称")
    session.key("Enter")
    session.wait_text("名称 Notes 已存在")
    session.screenshot("light-duplicate")
    session.press("取消")
    session.wait_text("已取消笔记本重命名")
    session.press("切换明暗")
    session.press("重命名分组")
    session.enter_text(" ", "分组名称")
    session.wait_text("请输入名称")
    if session.browser is not None:
        appearance = session.browser.result_value("browser_run_code_unsafe", code="""async (page) => {
          const dialog = page.getByRole('dialog', { name: '重命名分组' });
          return await dialog.evaluate(async el => {
            const animations = el.getAnimations({ subtree: true });
            await Promise.race([Promise.allSettled(animations.map(animation => animation.finished)),
                                new Promise(resolve => setTimeout(resolve, 400))]);
            return { background: getComputedStyle(el).backgroundColor,
                     fieldError: el.querySelector('input')?.getAttribute('aria-description') };
          });
        }""")
        assert max(map(int, re.findall(r"\d+", appearance["background"]))) < 64, appearance
        assert appearance["fieldError"] == "请输入名称", appearance
    session.screenshot("dark-required")
    session.press("取消")


def demo_pending(session):
    session.press("重命名笔记本")
    session.enter_text("待确认的名称", "笔记本名称")
    confirm = session.button("重命名")
    session.key("Enter")
    pending = session.wait_text("正在提交…", "已保存 0 次")
    if session.browser is None:
        response = session.call("autoui_action", element_id=confirm, action="press")
        assert response.startswith("Error:"), response
    else:
        assert 'button "重命名" [disabled]' in pending, pending
        assert 'button "取消" [disabled]' in pending, pending
    session.key("Escape")
    session.screenshot("pending")
    session.wait_text("notebook-7: 待确认的名称", "已保存 1 次", absent=("已取消笔记本重命名",))


SYSTEM_READER = r'''
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
$ErrorActionPreference = 'Stop'
$identity = Join-Path $PSScriptRoot 'system-window.txt'
$deadline = [DateTime]::UtcNow.AddSeconds(20)
while (!(Test-Path -LiteralPath $identity)) {
    if ([DateTime]::UtcNow -ge $deadline) { throw 'system window identity unavailable' }
    Start-Sleep -Milliseconds 30
}
$window = (Get-Content -Raw -LiteralPath $identity) -split " "
$root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]([long]$window[0]))
if ($root.Current.ProcessId -ne [int]$window[1]) { throw 'system window owner changed' }
$fieldName = [System.Windows.Automation.PropertyCondition]::new(
    [System.Windows.Automation.AutomationElement]::NameProperty, '分组名称')
$confirmName = [System.Windows.Automation.PropertyCondition]::new(
    [System.Windows.Automation.AutomationElement]::NameProperty, '保存分组')
$targets = [System.Windows.Automation.OrCondition]::new($fieldName, $confirmName)
$cache = [System.Windows.Automation.CacheRequest]::new()
$element = [System.Windows.Automation.AutomationElement]
$cache.Add($element::NameProperty)
$cache.Add($element::ControlTypeProperty)
$cache.Add($element::BoundingRectangleProperty)
$cache.Add($element::HasKeyboardFocusProperty)
$cache.Add($element::IsEnabledProperty)
$cache.Add([System.Windows.Automation.ValuePattern]::ValueProperty)
$cache.Add([System.Windows.Automation.ValuePattern]::Pattern)
$cache.Add([System.Windows.Automation.InvokePattern]::Pattern)
$cache.Push()
try {
    $nodes = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $targets)
} finally {
    $cache.Pop()
}
$rows = @()
$input = $null
$confirm = $null
foreach ($node in $nodes) {
    $current = $node.Cached
    $rect = $current.BoundingRectangle
    $role = $current.ControlType.ProgrammaticName
    $value = $null
    if ($role -eq 'ControlType.Edit' -and $current.Name -eq '分组名称') {
        $input = $node.GetCachedPattern([System.Windows.Automation.ValuePattern]::Pattern)
        $value = $input.Cached.Value
    }
    if ($role -eq 'ControlType.Button' -and $current.Name -eq '保存分组') {
        $confirm = $node.GetCachedPattern([System.Windows.Automation.InvokePattern]::Pattern)
    }
    $rows += [pscustomobject]@{role=$role; name=$current.Name; value=$value;
                              focus=$current.HasKeyboardFocus; enabled=$current.IsEnabled;
                              x=$rect.X; y=$rect.Y; width=$rect.Width; height=$rect.Height}
}
if ($null -eq $input -or $null -eq $confirm) { throw 'real system patterns are missing' }
$input.SetValue('系统输入')
$until = [DateTime]::UtcNow.AddSeconds(2)
while ($input.Current.Value -ne '系统输入') {
    if ([DateTime]::UtcNow -ge $until) { throw 'system SetValue failed' }
    Start-Sleep -Milliseconds 30
}
$confirm.Invoke()
[Console]::WriteLine((@{nodes=$rows; action='ValuePattern.SetValue and InvokePattern.Invoke'} | ConvertTo-Json -Depth 4))
'''


def demo_system(session):
    session.press("重命名分组")
    session.wait_text("例如 项目分类")
    window = session.windows[0]
    identity = session.output / "system-window.pending"
    identity.write_text(f'{window["handle"]} {window["pid"]}', encoding="ascii")
    identity.replace(session.output / "system-window.txt")
    raw, _ = session.system_helper.communicate(timeout=6)
    assert session.system_helper.returncode == 0, "Windows UIA helper failed"
    evidence = json.loads(raw.decode("utf-8-sig"))
    (session.output / "system-tree.json").write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2), encoding="utf-8",
    )
    field = next(row for row in evidence["nodes"] if row["role"] == "ControlType.Edit")
    confirm = next(row for row in evidence["nodes"] if row["role"] == "ControlType.Button")
    assert field["value"] == "Research" and field["focus"] and field["width"] > 0 and field["height"] > 0, field
    assert confirm["name"] == "保存分组" and confirm["enabled"] and confirm["width"] > 0 and confirm["height"] > 0, confirm
    session.wait_text("group-1: 系统输入", "已保存 1 次")


def file_manager_directory(session):
    session.wait_text("主目录", session.args.fixture_dir.name, "old.txt", "occupied.txt")
    return session.args.fixture_dir


def file_manager_menu(session, entry):
    tree = session.wait_text(entry)
    if session.browser is not None:
        session.call("browser_click", element=entry,
                     target=browser_button(tree, entry), button="right")
    else:
        start = re.search(rf'button #vnode_\d+ "{re.escape(entry)}[^"\n]*"', tree)
        assert start, f"No rendered entry {entry!r}"
        menu = native_button(tree[start.start():], "菜单")
        session.call("autoui_action", element_id=menu, action="press")
    session.wait_text("关闭菜单")


def file_manager_rename(session):
    directory = file_manager_directory(session)
    original = digest(directory / "old.txt")
    occupied = digest(directory / "occupied.txt")
    file_manager_menu(session, "old.txt")
    session.press("重命名")
    session.enter_text("occupied.txt", "名称")
    session.key("Enter")
    expected = "已存在同名项目" if session.browser is None else "写操作仅在 VM 轨可用"
    session.wait_text(expected)
    assert digest(directory / "old.txt") == original and digest(directory / "occupied.txt") == occupied
    session.enter_text("new.txt", "名称")
    session.key("Enter")
    if session.browser is None:
        session.wait_text("new.txt", absent=("old.txt",))
        assert not (directory / "old.txt").exists()
        assert digest(directory / "new.txt") == original
    else:
        session.wait_text(expected)
        assert digest(directory / "old.txt") == original and not (directory / "new.txt").exists()
        session.press("取消")


def file_manager_create(session):
    directory = file_manager_directory(session)
    initial = sorted(entry.name for entry in directory.iterdir())
    session.press("新建文件")
    session.enter_text("cancelled.txt", "名称")
    session.key("Escape")
    assert sorted(entry.name for entry in directory.iterdir()) == initial
    if session.browser is not None:
        session.wait_text('button "新建文件"', absent=('dialog "新建文件"',))
    else:
        session.wait_text(absent=('placeholder: "请输入名称"',))
    session.press("新建文件")
    session.enter_text("created.txt", "名称")
    session.press("创建")
    if session.browser is None:
        session.wait_text("created.txt")
        assert (directory / "created.txt").read_bytes() == b""
    else:
        session.wait_text("写操作仅在 VM 轨可用")
        assert sorted(entry.name for entry in directory.iterdir()) == initial
        session.press("取消")


CASES = {
    "names": demo_names,
    "cancel": demo_cancel,
    "tab": demo_tab,
    "ime": demo_ime,
    "visual": demo_visual,
    "pending": demo_pending,
    "system": demo_system,
    "file-rename": file_manager_rename,
    "file-create": file_manager_create,
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", type=Path, required=True)
    parser.add_argument("--ui-tool-bin", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--backend", choices=["native", "vue"], required=True)
    parser.add_argument("--app-dir", type=Path, required=True, help="isolated demo source, or prepared Vue project directory")
    parser.add_argument("--browser-mcp-config", type=Path)
    parser.add_argument("--fixture-dir", type=Path, help="isolated File Manager directory containing old.txt and occupied.txt")
    parser.add_argument("--api-bin", type=Path, help="prepared normal File Manager Rust API server")
    parser.add_argument("--api-dir", type=Path, help="source project directory for the prepared API server")
    parser.add_argument("--case", choices=CASES, default="names")
    parser.add_argument("--startup-timeout", type=float, default=20)
    args = parser.parse_args()
    args.consumer = "file-manager" if args.case.startswith("file-") else "demo"
    if args.consumer == "file-manager":
        if args.fixture_dir is None:
            parser.error("File Manager cases require --fixture-dir")
        args.fixture_dir = args.fixture_dir.resolve()
    if args.api_bin:
        args.api_bin = args.api_bin.resolve()
    if args.api_dir:
        args.api_dir = args.api_dir.resolve()
    if args.startup_timeout <= 0 or args.startup_timeout > 300:
        parser.error("--startup-timeout must be within (0, 300]")
    args.out_dir = args.out_dir.resolve()
    args.app_dir = args.app_dir.resolve()
    args.auto_bin = args.auto_bin.resolve()
    args.ui_tool_bin = args.ui_tool_bin.resolve()
    args.out_dir.mkdir(parents=True)
    started = time.monotonic()
    report = {"plan": "PLAN-094", "revision": 3, "platform": platform.system(), "backend": args.backend,
              "consumer": args.consumer, "case": args.case,
              "runner_sha256": digest(__file__)}
    if os.name != "nt" or not args.auto_bin.is_file() or not args.ui_tool_bin.is_file() or not args.app_dir.is_dir():
        report.update(result="not_run", error="Windows and prepared source-bound tools and project required")
    elif args.backend == "vue" and (not args.browser_mcp_config or not args.browser_mcp_config.is_file()
                                    or not (args.app_dir / "node_modules/vite/bin/vite.js").is_file()):
        report.update(result="not_run", error="prepared Vue project and browser MCP configuration required")
    elif args.consumer == "file-manager" and args.backend == "native" and (not args.api_bin or not args.api_bin.is_file()):
        report.update(result="not_run", error="prepared File Manager API binary required")
    elif args.consumer == "file-manager" and args.backend == "vue" and (not args.api_dir or not args.api_dir.is_dir()):
        report.update(result="not_run", error="File Manager VM backend source directory required")
    elif args.case == "ime" and args.backend != "vue":
        report.update(result="not_run", error="this composition case targets the Vue input adapter")
    elif args.case == "system" and args.backend != "native":
        report.update(result="not_run", error="this system tree case targets the Windows native provider")
    else:
        report["tools"] = {"auto": digest(args.auto_bin), "auto-ui-tool": digest(args.ui_tool_bin)}
        entries = ("pac.at", "src/front/app.at") if args.backend == "native" else (
            "package.json", "pnpm-lock.yaml", "src/App.vue", "src/components/PromptDialog.vue",
            "src/components/ui/dialog/Dialog.vue", "src/components/ui/dialog/DialogContent.vue",
        )
        report["source"] = {name: digest(args.app_dir / name)
                            for name in entries if (args.app_dir / name).is_file()}
        if args.api_bin:
            report["tools"]["file-manager-api"] = digest(args.api_bin)
        with (args.out_dir / "rpc.ndjson").open("w", encoding="utf-8") as journal:
            session = ProductSession(args, args.out_dir, journal)
            try:
                session.start()
                report["startup_seconds"] = time.monotonic() - started
                report["windows"] = session.windows
                report["owned_pids"] = list(session.owned)
                if session.browser is not None:
                    report["provider"] = session.provider
                action_started = time.monotonic()
                CASES[args.case](session)
                report["action_seconds"] = time.monotonic() - action_started
                report["result"] = "pass"
            except Exception as error:
                report.update(result="fail", error=repr(error))
            finally:
                (args.out_dir / "last-tree.txt").write_text(session.last_snapshot, encoding="utf-8")
                cleanup_started = time.monotonic()
                try:
                    session.close(failed=report["result"] != "pass")
                    report["cleanup"] = "pass"
                except Exception as error:
                    report.update(result="fail", cleanup=repr(error))
                report["cleanup_seconds"] = time.monotonic() - cleanup_started
        report["artifacts"] = {
            path.name: digest(path) for path in sorted(args.out_dir.iterdir())
            if path.is_file() and (path.suffix in {".png", ".log", ".ndjson"} or path.name in
                                   {"last-tree.txt", "system-tree.json"})
        }
    report["wall_seconds"] = time.monotonic() - started
    (args.out_dir / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return {"pass": 0, "fail": 1, "not_run": 2}[report["result"]]


if __name__ == "__main__":
    raise SystemExit(main())
