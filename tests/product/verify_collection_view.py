
"""PLAN-109 collection demo and FileManager gate using current accessibility trees.

Prepare the CLI and UI tool before running. The Vue leg takes a prepared URL
and a Playwright MCP stdio configuration (command and args). This gate
uses prepared providers. The 091 integration build has its own entrypoint.
"""
import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import queue
import re
import shutil
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

    def request(self, method, params, timeout=4):
        self.sequence += 1
        self.write({"id": self.sequence, "method": method, "params": params})
        deadline = time.monotonic() + timeout
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

    def call(self, name, *, reply_timeout=4, **arguments):
        result = self.request("tools/call", {"name": name, "arguments": arguments}, reply_timeout)
        if result.get("isError"):
            raise AssertionError(result)
        return "\n".join(entry["text"] for entry in result["content"] if entry["type"] == "text")

    def evaluate(self, function):
        return self.result_value("browser_evaluate", function=function)

    def result_value(self, tool, **arguments):
        result = self.call(tool, **arguments)
        match = re.search(r"^### Result\s*\n(.*?)(?=\n### |\Z)", result, re.MULTILINE | re.DOTALL)
        assert match, result
        return json.loads(match[1].strip())

    def close(self):
        try:
            self.process.stdin.close()
        except BrokenPipeError:
            pass
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
def browser_session(args, report):
    config = json.loads(args.browser_mcp_config.read_text(encoding="utf-8-sig"))
    index = len(report["sessions"]) + 1
    with (args.out_dir / f"browser-{index}.log").open("w", encoding="utf-8") as log:
        browser = BrowserMcp(config, log, args.out_dir)
        session = {"pid": browser.process.pid, "prepared_url": args.url, "owns_service": False}
        report["sessions"].append(session)
        try:
            browser.request("initialize", {
                "protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "plan109-product", "version": "1"},
            })
            browser.write({"method": "notifications/initialized"})
            try:
                browser.call("browser_navigate", url=args.url)
            except (AssertionError, queue.Empty) as error:
                raise AssertionError(f"preparation failure: cannot open {args.url}: {error}") from error
            yield browser
        except BaseException:
            try:
                session["failure_snapshot"] = browser.call("browser_snapshot")
                browser.call("browser_take_screenshot", type="png", filename=f"failure-{index}.png")
            except (AssertionError, BrokenPipeError, queue.Empty):
                pass
            raise
        finally:
            browser.close()
            session["closed"] = browser.process.poll() is not None


def test_core_browser(args, report):
    with browser_session(args, report) as browser:
        inspect = lambda: browser.call("browser_snapshot")
        first = wait_for(inspect, lambda s: s.count('button "同名条目"') == 2)
        report["initial_snapshot"] = first
        row_b = browser_button(first, "同名条目", 1)
        browser.call("browser_click", target=row_b, element="第二个同名条目")
        chosen = wait_for(inspect, lambda s: "已选择 b" in s)
        browser.call("browser_click", target=browser_button(chosen, "列表"), element="列表")
        current = wait_for(inspect, lambda s: "文档" in s and "已选择 b" in s)
        assert browser.evaluate("() => document.activeElement.title") == "b", "view change lost the item focus"
        browser.call("browser_press_key", key="ArrowUp")
        browser.call("browser_press_key", key="Space")
        current = wait_for(inspect, lambda s: "已选择 a" in s)
        browser.call("browser_click", target=browser_button(current, "同名条目", 1), element="第二个同名条目")
        current = wait_for(inspect, lambda s: "已选择 b" in s)
        browser.call("browser_click", target=browser_button(current, "名称"), element="按名称排序")
        assert browser.evaluate("() => document.activeElement.title") == "b", "sort lost the item focus"
        ordered = browser.evaluate("() => [...document.querySelectorAll('button[id^=cv-][title]')].map(node => node.title)")
        assert ordered == ["b", "a", "c"], ordered
        browser.evaluate("() => { [...document.querySelectorAll('button')].find(b => b.textContent.trim() === '移除所选').click(); return null }")
        assert browser.evaluate("() => document.activeElement.title") == "a", "removed focus did not move to the first enabled item"
        browser.call("browser_press_key", key="Space")
        current = wait_for(inspect, lambda s: "已选择 a" in s)
        browser.call("browser_click", target=browser_button(current, "打开"), element="打开第一项")
        report["final_snapshot"] = wait_for(inspect, lambda s: "已选择 a" in s and "已打开 a" in s)
        assert any(
            "button " in line and "不可用条目" in line and "[disabled]" in line
            for line in report["final_snapshot"].splitlines()
        ), "the unavailable entry is missing or enabled"
        browser.call("browser_close")
    with browser_session(args, report) as browser:
        inspect = lambda: browser.call("browser_snapshot")
        first = wait_for(inspect, lambda s: s.count('button "同名条目"') == 2)
        browser.call("browser_click", target=browser_button(first, "同名条目"), element="第一个同名条目")
        report["reopen_readback"] = wait_for(inspect, lambda s: "已选择 a" in s)
        browser.call("browser_close")
    assert report["sessions"][0]["pid"] != report["sessions"][1]["pid"]
    assert all(session["closed"] for session in report["sessions"]), "browser service did not exit"


def test_file_manager_browser(args, report):
    with browser_session(args, report) as browser:
        directory = json.dumps(args.fixture_dir.as_posix())
        result = browser.result_value("browser_run_code_unsafe", reply_timeout=7, code=f"""async (page) => {{
            const button = name => page.getByRole('button', {{ name, exact: true }}).first();
            const text = value => page.getByText(value, {{ exact: true }}).first();
            const selectedNotes = page.getByText(/选定:\\s*notes\\.txt/).first();
            await button('主目录').waitFor({{ timeout: 4000 }});
            await text('条目加载中').waitFor({{ state: 'hidden', timeout: 4000 }});
            await button('编辑地址').click();
            await page.getByPlaceholder('输入路径后回车...').first().fill({directory});
            await button('跳转').click();
            await button('notes.txt').click();
            await selectedNotes.waitFor({{ timeout: 2000 }});
            await button('名称').click();
            await page.waitForFunction(() => {{
                const names = [...document.querySelectorAll('button')].map(node => node.textContent.trim());
                return names.includes('sample.png') && names.includes('broken.png') &&
                       names.indexOf('sample.png') < names.indexOf('broken.png');
            }}, undefined, {{ timeout: 2000 }});
            await selectedNotes.waitFor({{ timeout: 2000 }});
            await button('folder').click({{ button: 'right' }});
            await button('关闭菜单').waitFor({{ timeout: 2000 }});
            await selectedNotes.waitFor({{ timeout: 2000 }});
            await button('关闭菜单').click();
            await button('网格').click();
            await button('名称').waitFor({{ state: 'hidden', timeout: 2000 }});
            await selectedNotes.waitFor({{ timeout: 2000 }});
            const summary = await selectedNotes.textContent();
            await button('folder').dblclick();
            await button('inside.txt').waitFor({{ timeout: 2000 }});
            return {{ selectionBeforeOpen: summary,
                      openedDirectoryEntry: (await button('inside.txt').innerText()).trim() }};
        }}""")
        report["readback"] = result
        assert "选定: notes.txt" in result["selectionBeforeOpen"] and result["openedDirectoryEntry"] == "inside.txt", result

def test_picker_browser(args, report):
    with browser_session(args, report) as browser:
        snapshot = lambda: browser.call("browser_snapshot")
        page = wait_for(snapshot, lambda s: 'button "打开选择器"' in s, 3)
        browser.call("browser_click", target=browser_button(page, "打开选择器"), element="打开选择器")
        picker = wait_for(snapshot, lambda s: "Lake.jpg" in s and "共 1003 项" in s, 3)
        decoded = browser.evaluate("""() => [...document.images].some(img =>
            (img.dataset.autoImageSrc || img.currentSrc).includes('scenic-320x240.png') &&
            img.complete && img.naturalWidth === 320)""")
        assert decoded is True, "authorized image did not decode at its expected natural width"
        def provider_readback():
            return browser.evaluate("""() => document.body.innerText.split('\\n').filter(line =>
                line.startsWith('selection=') || line.startsWith('附件日志:'))""")
        before = provider_readback()
        assert before == ["selection=", "附件日志: 0 条 · 最近="], before
        browser.call("browser_click", target=browser_button(picker, "预览"), element="首项预览")
        preview = wait_for(snapshot, lambda s: 'button "关闭预览"' in s)
        assert provider_readback() == before, "preview changed the selection or attach log"
        browser.call("browser_click", target=browser_button(preview, "关闭预览"), element="关闭预览")
        report["readback"] = wait_for(snapshot, lambda s: "Lake.jpg" in s and 'button "关闭预览"' not in s)
        assert provider_readback() == before, "closing preview changed the selection or attach log"
        report["thumbnail_decoded"] = decoded
        browser.call("browser_close")


def test_consumer_browser_reopen(args, report):
    for _ in range(2):
        with browser_session(args, report) as browser:
            inspect = lambda: browser.call("browser_snapshot")
            page = wait_for(inspect, lambda s: 'button "打开选择器"' in s, 3)
            browser.call("browser_click", target=browser_button(page, "打开选择器"), element="打开选择器")
            report["sessions"][-1]["readback"] = wait_for(inspect, lambda s: "共 1003 项" in s)
            browser.call("browser_close")
    require_browser_reopened(report)


def test_file_manager_browser_reopen(args, report):
    directory = json.dumps(args.fixture_dir.as_posix())
    with browser_session(args, report) as browser:
        actions = browser.result_value("browser_run_code_unsafe", reply_timeout=8, code=f"""async (page) => {{
            const control = (p, name) => p.getByRole('button', {{ name, exact: true }}).first();
            const ready = async p => {{
                await control(p, '主目录').waitFor({{ timeout: 3000 }});
                await p.getByText('条目加载中', {{ exact: true }}).waitFor({{ state: 'hidden', timeout: 3000 }});
            }};
            await ready(page);
            await control(page, '名称').waitFor({{ timeout: 2000 }});
            await control(page, '网格').click();
            await control(page, '名称').waitFor({{ state: 'hidden', timeout: 2000 }});
            const old = page.context();
            const next = await old.browser().newContext({{ storageState: await old.storageState(), viewport: page.viewportSize() }});
            const url = page.url();
            await page.close();
            if (!page.isClosed()) throw Error('first application page still exists');
            try {{
                const reopened = await next.newPage();
                await reopened.goto(url, {{ waitUntil: 'domcontentloaded' }});
                await ready(reopened);
                if (await control(reopened, '名称').count() !== 0) throw Error('grid mode was not restored');
                await control(reopened, '编辑地址').click();
                await reopened.getByPlaceholder('输入路径后回车...').first().fill({directory});
                await control(reopened, '跳转').click();
                const row = control(reopened, 'notes.txt');
                await row.waitFor({{ timeout: 2500 }});
                return {{ firstPageClosed: page.isClosed(), freshContext: reopened.context() !== old,
                          restoredGrid: true, realDirectoryEntry: (await row.innerText()).trim() }};
            }} finally {{
                await next.close();
            }}
        }}""")
        report["reopen_readback"] = actions
        assert actions == {"firstPageClosed": True, "freshContext": True,
                           "restoredGrid": True, "realDirectoryEntry": "notes.txt"}, actions
    assert report["sessions"][0]["closed"], "prepared browser session leaked"

def require_browser_reopened(report):
    assert report["sessions"][0]["pid"] != report["sessions"][1]["pid"]
    assert all(session["closed"] for session in report["sessions"])


def test_browser_failure(args, report):
    try:
        with browser_session(args, report) as browser:
            inspect = lambda: browser.call("browser_snapshot")
            wait_for(inspect, lambda s: 'button "列表"' in s)
            browser.call("browser_click", target="missing-ref", element="不存在的条目")
            raise AssertionError("unexpected successful action")
    except AssertionError as error:
        assert str(error) != "unexpected successful action", error
        report["expected_failure"] = str(error)
    session = report["sessions"][0]
    assert 'button "列表"' in session.get("failure_snapshot", "") and session["closed"], session
    assert args.out_dir.joinpath("failure-1.png").stat().st_size > 0, "missing failure screenshot"


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
    shutil.copytree(ROOT / "packages/collection-view", app / "vendor/collection-view")
    manifest = (src / "pac.at").read_text(encoding="utf-8")
    (app / "pac.at").write_text(
        manifest.replace("../../packages/collection-view", "vendor/collection-view"), encoding="utf-8"
    )
    entry = (src / "src/front/app.at").read_text(encoding="utf-8")
    (front / "app.at").write_text(
        entry.replace("../../../../packages/collection-view", "../../vendor/collection-view"),
        encoding="utf-8",
    )
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
    endpoint_result = queue.Queue(maxsize=1)
    closed_port = None

    def inspect_closed_endpoint(port):
        try:
            native_call(port, "autoui_snapshot", timeout=2.6)
        except urllib.error.URLError as error:
            endpoint_result.put(error.reason)
        except OSError as error:
            endpoint_result.put(error)
        else:
            endpoint_result.put("endpoint still responds")

    for index, select in enumerate(("b", "a"), start=1):
        with native_session(args, app, index) as (process, port):
            assert port != closed_port, "reopen reused the endpoint under verification"
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
            else:
                response += "\n" + native_call(port, "autoui_keyboard", key="ArrowRight")
                # The native focus operation is applied on the next rendered frame.
                time.sleep(0.7)
                assert "已选择 a" in inspect(), "arrow movement changed selection"
                response += "\n" + native_call(port, "autoui_keyboard", key="Space")
                current = wait_for(inspect, lambda s: "已选择 b" in s)
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
            closed_port = port
            # The first process has exited. Check its endpoint while the second
            # process starts on its own endpoint.
            Thread(target=inspect_closed_endpoint, args=(port,), daemon=True).start()
    refusal = endpoint_result.get(timeout=2.7)
    assert isinstance(refusal, ConnectionRefusedError), f"old endpoint {closed_port}: {refusal!r}"
    report["closed_endpoint"] = {"port": closed_port, "result": "connection refused"}


def test_native_failure(args, report):
    app = stage_core(args.out_dir)
    with native_session(args, app, 1) as (process, port):
        inspect = lambda: native_call(port, "autoui_snapshot")
        wait_for(inspect, lambda s: "同名条目" in s and "退出演示" in s, 3.2)
        owned = set(product.owned_tree(process.pid)) | {process.pid}
        rejected = native_call(port, "autoui_action", element_id="vnode_0", action="press")
        assert "not found" in rejected.lower(), rejected
        report["expected_failure"] = rejected
        report["failure_snapshot"] = inspect()
        shot = native_call(port, "autoui_screenshot", timeout=2.6)
        path = re.search(r"saved to:[ \t]*(.+\.png)", shot)
        assert path, shot
        copied = shutil.copyfile(path[1], args.out_dir / "failure.png")
        report["failure_screenshot"] = product.sha(copied)
    assert process.poll() is not None and not product.windows_for(owned), "failed session still owns a window"
    require_closed_endpoint(port)
    report["sessions"].append({"pid": process.pid, "port": port, "closed": True, "endpoint": "connection refused"})


def require_closed_endpoint(port):
    deadline = time.monotonic() + 3
    while (remaining := deadline - time.monotonic()) > 0:
        try:
            native_call(port, "autoui_snapshot", timeout=remaining)
        except (ConnectionResetError, urllib.error.URLError) as error:
            reason = error.reason if isinstance(error, urllib.error.URLError) else error
            if isinstance(reason, ConnectionRefusedError):
                return
            if not isinstance(reason, ConnectionResetError):
                raise
        else:
            raise AssertionError(f"closed endpoint {port} still responds")
        time.sleep(0.06)
    raise AssertionError(f"closed endpoint {port} did not refuse a fresh connection")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", type=Path, required=True)
    parser.add_argument("--ui-tool-bin", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--backend", choices=("native", "vue"), required=True)
    parser.add_argument("--consumer", choices=("demo", "file-manager", "asset-picker"), default="demo")
    parser.add_argument("--scenario", choices=("main", "reopen", "failure"), default="main")
    parser.add_argument("--browser-mcp-config", type=Path)
    parser.add_argument("--url", help="prepared Vue application URL")
    parser.add_argument("--fixture-dir", type=Path, help="isolated FileManager fixture directory")
    parser.add_argument("--native-port", type=int, help="MCP port of the prepared native FileManager")
    parser.add_argument("--native-pid", type=int, help="CLI PID of the prepared native FileManager")
    args = parser.parse_args()
    if args.backend == "vue" and not (args.browser_mcp_config and args.url):
        parser.error("preparation failure: prepare Vue and set --browser-mcp-config and --url")
    if args.backend == "vue" and not args.browser_mcp_config.is_file():
        parser.error("preparation failure: browser MCP config is missing")
    if args.consumer == "asset-picker" and args.backend != "vue":
        parser.error("this prepared AssetPicker gate takes a Vue URL")
    if args.scenario == "reopen" and (args.backend != "vue" or args.consumer == "demo"):
        parser.error("select a prepared Vue consumer for the reopen scenario")
    if args.scenario == "failure" and args.consumer != "demo":
        parser.error("the failure scenario owns the independent demo session")
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
        "plan": "109", "revision": 2, "scope": args.consumer, "scenario": args.scenario, "backend": args.backend,
        "platform": sys.platform, "cli_sha256": product.sha(args.auto_bin),
        "tool_sha256": product.sha(args.ui_tool_bin), "sessions": [],
    }
    try:
        if args.scenario == "reopen":
            (test_file_manager_browser_reopen if args.consumer == "file-manager" else test_consumer_browser_reopen)(args, report)
        elif args.scenario == "failure":
            (test_native_failure if args.backend == "native" else test_browser_failure)(args, report)
        elif args.consumer == "file-manager":
            run = test_file_manager_native if args.backend == "native" else test_file_manager_browser
            run(args, report)
        elif args.consumer == "asset-picker":
            test_picker_browser(args, report)
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
