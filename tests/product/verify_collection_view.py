
"""PLAN-109 collection demo and FileManager gate using current accessibility trees.

Prepare the CLI and UI tool before running. The Vue leg takes a prepared URL
and a Playwright MCP stdio configuration (command and args). Consumer-level
gates for AssetPicker and Gallery remain under verify_asset_picker.py.
"""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import queue
import re
import subprocess
import sys
from threading import Thread
import time
import urllib.error
import urllib.request

import verify_file_dialog as product
from verify_pagination import bindable_port


ROOT = Path(__file__).resolve().parents[2]


def wait_for(read, predicate, seconds=2):
    deadline = time.monotonic() + seconds
    last = ""
    while time.monotonic() < deadline:
        try:
            last = read()
            if predicate(last):
                return last
        except OSError as error:
            last = str(error)
        time.sleep(0.06)
    raise AssertionError(f"no expected readback within {seconds}s: {last[-2000:]}")


def native_call(port, tool, *, timeout=1.0, **arguments):
    message = {"jsonrpc": "2.0", "id": 1, "method": "tools/call",
               "params": {"name": tool, "arguments": arguments}}
    request = urllib.request.Request(
        f"http://127.0.0.1:{port}/mcp",
        data=json.dumps(message).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        result = json.load(response)["result"]
    return "\n".join(entry["text"] for entry in result["content"] if entry["type"] == "text")


def native_button(snapshot, label):
    for match in re.finditer(r'button #(vnode_\d+) "([^"\n]*)"', snapshot):
        text = match[2].split("\uee03", 1)
        if label in text:
            return match[1]
    raise AssertionError(f"button {label!r} missing from rendered tree")


class BrowserMcp:
    def __init__(self, config, log, working_dir):
        self.messages = queue.Queue()
        self.sequence = 0
        self.process = subprocess.Popen(
            [config["command"], *config.get("args", [])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
            encoding="utf-8", text=True, cwd=working_dir,
            env={**os.environ, **config.get("env", {})},
        )
        self.reader = Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        for line in self.process.stdout:
            self.messages.put(json.loads(line))

    def request(self, method, params):
        self.sequence += 1
        self.write({"id": self.sequence, "method": method, "params": params})
        deadline = time.monotonic() + 4
        while (remaining := deadline - time.monotonic()) > 0:
            result = self.messages.get(timeout=remaining)
            if result.get("id") == self.sequence:
                if "error" in result:
                    raise AssertionError(result["error"])
                return result["result"]
        raise AssertionError(f"Browser MCP did not respond to {method}")

    def write(self, message):
        self.process.stdin.write(json.dumps({"jsonrpc": "2.0", **message}) + "\n")
        self.process.stdin.flush()

    def call(self, name, **arguments):
        result = self.request("tools/call", {"name": name, "arguments": arguments})
        if result.get("isError"):
            raise AssertionError(result)
        return "\n".join(entry["text"] for entry in result["content"] if entry["type"] == "text")

    def close(self):
        self.process.stdin.close()
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            stop_tree(self.process)


def stop_tree(process):
    if process.poll() is not None:
        return
    if sys.platform == "win32":
        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                       capture_output=True, timeout=3, check=False)
    else:
        process.terminate()
    process.wait(timeout=3)


def browser_button(snapshot, name, occurrence=0):
    matches = [
        re.search(r"\[ref=(\w+)\]", line) for line in snapshot.splitlines()
        if f'button "{name}"' in line and "[ref=" in line
    ]
    return matches[occurrence][1]


@contextmanager
def browser_session(args):
    config = json.loads(args.browser_mcp_config.read_text(encoding="utf-8-sig"))
    with (args.out_dir / "browser.log").open("w", encoding="utf-8") as log:
        browser = BrowserMcp(config, log, args.out_dir)
        try:
            browser.request("initialize", {
                "protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "plan109-product", "version": "1"},
            })
            browser.write({"method": "notifications/initialized"})
            browser.call("browser_navigate", url=args.url)
            yield browser
        finally:
            browser.close()


def test_core_browser(args, report):
    with browser_session(args) as browser:
        inspect = lambda: browser.call("browser_snapshot")
        first = wait_for(inspect, lambda s: s.count('button "同名条目"') == 2)
        report["initial_snapshot"] = first
        row_b = browser_button(first, "同名条目", 1)
        browser.call("browser_click", target=row_b, element="第二个同名条目")
        chosen = wait_for(inspect, lambda s: "已选择 b" in s)
        browser.call("browser_click", target=browser_button(chosen, "列表"), element="列表")
        current = wait_for(inspect, lambda s: "文档" in s and "已选择 b" in s)
        browser.call("browser_click", target=browser_button(current, "打开"), element="打开第一项")
        current = wait_for(inspect, lambda s: "已选择 b" in s and "已打开 a" in s)
        browser.call("browser_click", target=browser_button(current, "同名条目", 1),
                     element="列表中的第二个同名条目")
        browser.call("browser_press_key", key="ArrowUp")
        browser.call("browser_press_key", key="Space")
        report["final_snapshot"] = wait_for(inspect, lambda s: "已选择 a" in s)
        assert any(
            "button " in line and "不可用条目" in line and "[disabled]" in line
            for line in report["final_snapshot"].splitlines()
        ), "the unavailable entry is missing or enabled"
        browser.call("browser_close")


def test_file_manager_browser(args, report):
    with browser_session(args) as browser:
        snapshot = lambda: browser.call("browser_snapshot")

        def click(tree, label, **arguments):
            return browser.call("browser_click", target=browser_button(tree, label),
                                element=label, **arguments)

        report["stage"] = "open address editor"
        ready = wait_for(snapshot, lambda s: 'button "编辑地址"' in s and
                         'button "主目录"' in s and '条目加载中' not in s, 4)
        click(ready, "编辑地址")
        editing = wait_for(snapshot, lambda s: 'textbox "输入路径后回车...' in s)
        line = next(line for line in editing.splitlines() if 'textbox "输入路径后回车...' in line)
        browser.call("browser_type", target=re.search(r"\[ref=(\w+)\]", line)[1],
                     element="目录地址", text=args.fixture_dir.as_posix())
        report["stage"] = "navigate to directory fixture"
        click(snapshot(), "跳转")
        listed = wait_for(snapshot, lambda s: 'button "notes.txt"' in s and "sample.png" in s)
        report["stage"] = "select file"
        click(listed, "notes.txt")
        chosen = wait_for(snapshot, lambda s: "选定: notes.txt" in s)
        report["stage"] = "reverse name sort"
        click(chosen, "名称")
        sorted_tree = wait_for(snapshot, lambda s: "选定: notes.txt" in s and
                               0 <= s.find("sample.png") < s.find("broken.png"))
        report["stage"] = "switch to grid"
        click(sorted_tree, "网格")
        grid = wait_for(snapshot, lambda s: "选定: notes.txt" in s and 'button "名称"' not in s)
        report["stage"] = "open subdirectory"
        click(grid, "folder", doubleClick=True)
        report["readback"] = wait_for(snapshot, lambda s: 'button "inside.txt"' in s)
        report["selection_after_sort_and_view_change"] = grid
        browser.call("browser_close")


def test_file_manager_native(args, report):
    owned = set(product.owned_tree(args.native_pid)) | {args.native_pid}
    windows = product.windows_for(owned)
    assert windows, "no visible window owned by the prepared FileManager"
    report["window"] = windows
    snapshot = lambda: native_call(args.native_port, "autoui_snapshot")
    report["actions"] = []

    def press(tree, label):
        result = native_call(args.native_port, "autoui_action", action="press",
                             element_id=native_button(tree, label))
        report["actions"].append({"label": label, "response": result})

    ready = wait_for(snapshot, lambda s: ".AddrEdit" in s and "主目录" in s, 3)
    press(ready, "编辑地址")
    editing = wait_for(snapshot, lambda s: ".SetAddr" in s)
    field = re.search(
        r"input #(vnode_\d+)[^\n]*\n(?:(?!^\s*\}).*\n)*?\s*oninput: \.SetAddr",
        editing, re.MULTILINE,
    )
    assert field, "address input has no native node"
    native_call(args.native_port, "autoui_type", element_id=field[1], text=args.fixture_dir.as_posix())
    press(snapshot(), "跳转")
    listed = wait_for(snapshot, lambda s: "sample.png" in s and "notes.txt" in s)
    press(listed, "notes.txt")
    chosen = wait_for(snapshot, lambda s: "选定: notes.txt" in s)
    if '"名称"' not in chosen:
        press(chosen, "列表")
        chosen = wait_for(snapshot, lambda s: "选定: notes.txt" in s and '"名称"' in s)
    press(chosen, "名称")
    sorted_tree = wait_for(snapshot, lambda s: "选定: notes.txt" in s and
                           0 <= s.find("sample.png") < s.find("broken.png"))
    folder = re.search(r'button #vnode_\d+ "folder[^"\n]*"', sorted_tree)
    assert folder, "folder action cannot be bound to its row"
    press(sorted_tree[folder.start():], "菜单")
    menu = wait_for(snapshot, lambda s: "关闭菜单" in s and "选定: notes.txt" in s)
    report["selection_after_sort_and_menu"] = menu
    press(menu, "打开")
    report["readback"] = wait_for(snapshot, lambda s: "inside.txt" in s)


def stage_core(out):
    src = ROOT / "examples/collection-view"
    app = out / "core"
    front = app / "src/front"
    front.mkdir(parents=True)
    package = (ROOT / "packages/collection-view").resolve().as_posix()
    manifest = (src / "pac.at").read_text(encoding="utf-8")
    (app / "pac.at").write_text(manifest.replace("../../packages/collection-view", package),
                                encoding="utf-8")
    entry = (src / "src/front/app.at").read_text(encoding="utf-8")
    (front / "app.at").write_text(entry.replace("../../../../packages/collection-view", package),
                                  encoding="utf-8")
    return app


@contextmanager
def native_session(args, app, index):
    port = bindable_port(product.free_port())
    env = {**os.environ, "AUTOUI_MCP_PORT": str(port),
           "AUTO_UI_TOOL_BIN": str(args.ui_tool_bin),
           "AUTO_VM_STORAGE_FILE": str(args.out_dir / "storage.json")}
    command = [str(args.auto_bin), "run", "-r", "vm"]
    with (args.out_dir / f"native-{index}.log").open("wb") as log:
        process = subprocess.Popen(command, cwd=app, env=env, stdout=log,
                                   stderr=subprocess.STDOUT,
                                   creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
        try:
            yield process, port
        finally:
            stop_tree(process)


def test_core_native(args, report):
    app = stage_core(args.out_dir)
    for index, select in enumerate(("b", "a"), start=1):
        with native_session(args, app, index) as (process, port):
            inspect = lambda: native_call(port, "autoui_snapshot")
            initial = wait_for(inspect, lambda s: '同名条目' in s and '退出演示' in s, 3.2)
            owned = set(product.owned_tree(process.pid)) | {process.pid}
            windows = product.windows_for(owned)
            assert windows, "no visible native window owned by the prepared process"
            response = native_call(port, "autoui_action", action="press",
                                   element_id=native_button(initial, select))
            current = wait_for(inspect, lambda s: f"已选择 {select}" in s, 1.0)
            if index == 1:
                response += "\n" + native_call(
                    port, "autoui_action", action="press", element_id=native_button(current, "列表")
                )
                current = wait_for(inspect, lambda s: "文档" in s and f"已选择 {select}" in s)
                old_first = native_button(current, "a")
                native_call(port, "autoui_action", action="press",
                            element_id=native_button(current, "替换条目"))
                current = wait_for(
                    inspect, lambda s: native_button(s, "a") != old_first and "已选择 b" in s
                )
                rejected = native_call(port, "autoui_action", action="press", element_id=old_first)
                assert "not found" in rejected.lower(), rejected
                assert "已选择 b" in inspect(), "old binding changed the application selection"
                report["stale_action"] = rejected
            try:
                native_call(port, "autoui_action", action="press",
                            element_id=native_button(current, "退出演示"))
            except (ConnectionResetError, TimeoutError):
                # The application may close before its last action response.
                pass
            except urllib.error.URLError as error:
                if not isinstance(error.reason, (TimeoutError, ConnectionRefusedError)):
                    raise
            assert process.wait(timeout=3) == 0, "normal exit returned a failure"
            report["sessions"].append({"window": windows, "port": port, "actions": response,
                                       "readback": current})
        if index == 1:
            try:
                native_call(port, "autoui_snapshot", timeout=2.6)
            except urllib.error.URLError as error:
                if not isinstance(error.reason, ConnectionRefusedError):
                    raise
                report["closed_endpoint"] = {"port": port, "result": "connection refused"}
            else:
                raise AssertionError(f"closed process left its MCP endpoint live on {port}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", type=Path, required=True)
    parser.add_argument("--ui-tool-bin", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--backend", choices=("native", "vue"), required=True)
    parser.add_argument("--consumer", choices=("demo", "file-manager"), default="demo")
    parser.add_argument("--browser-mcp-config", type=Path)
    parser.add_argument("--url", help="prepared Vue application URL")
    parser.add_argument("--fixture-dir", type=Path, help="isolated FileManager fixture directory")
    parser.add_argument("--native-port", type=int, help="MCP port of the prepared native FileManager")
    parser.add_argument("--native-pid", type=int, help="CLI PID of the prepared native FileManager")
    args = parser.parse_args()
    if args.backend == "vue" and not (args.browser_mcp_config and args.url):
        parser.error("prepare Vue and set --browser-mcp-config and --url")
    if args.consumer == "file-manager":
        if not args.fixture_dir:
            parser.error("--fixture-dir is required for FileManager")
        args.fixture_dir = args.fixture_dir.resolve(strict=True)
        files = ("notes.txt", "sample.png", "broken.png", "folder/inside.txt")
        if not all((args.fixture_dir / name).is_file() for name in files):
            parser.error("FileManager requires isolated notes.txt, sample.png, broken.png and folder/inside.txt")
        if args.backend == "native" and not (args.native_pid and args.native_port):
            parser.error("--native-pid and --native-port are required for the prepared FileManager")
    args.auto_bin = args.auto_bin.resolve(strict=True)
    args.ui_tool_bin = args.ui_tool_bin.resolve(strict=True)
    args.out_dir = args.out_dir.resolve()
    args.out_dir.mkdir(parents=True, exist_ok=True)
    start = time.monotonic()
    report = {
        "plan": "109", "revision": 2, "scope": args.consumer, "backend": args.backend,
        "platform": sys.platform, "cli_sha256": product.sha(args.auto_bin),
        "tool_sha256": product.sha(args.ui_tool_bin), "sessions": [],
    }
    try:
        if args.consumer == "file-manager":
            run = test_file_manager_native if args.backend == "native" else test_file_manager_browser
            run(args, report)
        elif args.backend == "native":
            test_core_native(args, report)
        else:
            test_core_browser(args, report)
        report["result"] = "pass"
    except (AssertionError, IndexError, KeyError, OSError, ValueError,
            queue.Empty, subprocess.SubprocessError) as error:
        report["result"] = "fail"
        report["error"] = str(error)
    finally:
        report["seconds"] = time.monotonic() - start
        if report["seconds"] > 10:
            report["result"] = "fail"
            report["timing_failure"] = "case exceeded 10 seconds"
        (args.out_dir / "report.json").write_text(
            json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8"
        )
    print(f'{report["result"]} in {report["seconds"]:.3f}s')
    return 0 if report["result"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
