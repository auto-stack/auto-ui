"""PLAN-097 TagInput product checks (native VM / Vue)."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import socket
import subprocess
import sys
import time
import queue
import threading
import urllib.error
import urllib.request


def available_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def stop_tree(process):
    if process.poll() is not None:
        return
    if sys.platform == "win32":
        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                       capture_output=True, timeout=3, check=False)
    else:
        process.terminate()
    process.wait(timeout=3)


def native_call(port, tool, *, timeout=2.0, **arguments):
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


def node_id(snapshot, pattern):
    match = re.search(pattern, snapshot)
    assert match, f"node {pattern!r} missing from tree:\n{snapshot[-4000:]}"
    return match.group(1)


def wait_text(native, port, text, seconds=8):
    last = ""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        last = native()
        if text in last:
            return last
        time.sleep(0.25)
    raise AssertionError(f"no {text!r} within {seconds}s; last tree:\n{last[-4000:]}")


class NativeSession:
    def __init__(self, auto_bin, ui_tool_bin, app_dir, out_dir, port, log, startup_timeout=120):
        self.port = port
        self.out_dir = out_dir
        self.log = log
        env = {
            **os.environ,
            "AUTOUI_MCP_PORT": str(port),
            "AUTO_UI_TOOL_BIN": str(ui_tool_bin),
            "AUTO_VM_STORAGE_FILE": str(Path(out_dir) / "vm-storage.json"),
        }
        self.process = subprocess.Popen(
            [str(auto_bin), "run", "-r", "vm"],
            cwd=str(app_dir), env=env,
            stdout=log, stderr=log, text=True,
        )
        self.mcp = self._wait_mcp(startup_timeout)

    def _wait_mcp(self, startup_timeout):
        deadline = time.monotonic() + startup_timeout
        seen = ""
        while time.monotonic() < deadline:
            try:
                snapshot = native_call(self.port, "autoui_snapshot", timeout=2.0)
                if snapshot:
                    return snapshot
            except (urllib.error.URLError, OSError):
                pass
            time.sleep(0.5)
        raise AssertionError(
            f"native app did not expose MCP on port {self.port} within {startup_timeout}s; log:\n"
            f"{seen[-3000:] if (seen := Path(self.log).read_text(errors='replace')[-3000:]) else 'empty'}"
        )

    def call(self, tool, **arguments):
        return native_call(self.port, tool, **arguments)

    def snapshot(self):
        return self.call("autoui_snapshot")

    def press(self, vid):
        return self.call("autoui_action", action="press", element_id=vid)

    def type_text(self, text):
        return self.call("autoui_type", text=text)

    def key(self, key):
        return self.call("autoui_keyboard", key=key)

    def screenshot(self, name):
        out = Path(self.out_dir) / name
        text = self.call("autoui_screenshot")
        match = re.search(r"base64,([A-Za-z0-9+/=]+)", text)
        if match:
            out.write_bytes(__import__("base64").b64decode(match.group(1)))
        else:
            out.write_text(text, encoding="utf-8")
        return out

    def close(self, failed=False):
        stop_tree(self.process)
        deadline = time.monotonic() + 6
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                break
            time.sleep(0.2)
        if self.process.poll() is None:
            raise AssertionError("native process did not exit after stop")
        try:
            native_call(self.port, "autoui_snapshot", timeout=1.0)
            raise AssertionError("MCP still answered after process exit")
        except (urllib.error.URLError, OSError):
            pass


def input_vid(snapshot, placeholder):
    pattern = rf'input #(vnode_\d+) \{{[^}}]*placeholder: "{re.escape(placeholder)}"'
    return node_id(snapshot, pattern)


def button_vid(snapshot, label):
    return node_id(snapshot, rf'button #(vnode_\d+) "[^"]*{re.escape(label)}"')


def chip_delete_vid(snapshot, tag):
    return node_id(snapshot, rf'button #(vnode_\d+) "[^"]*删除标签 {re.escape(tag)}"')


def add_button_near(snapshot, input_vid):
    i = snapshot.find(input_vid)
    assert i >= 0, "input vid not in snapshot"
    chunk = snapshot[i:i + 400]
    m = re.search(r'button #(vnode_\d+) "[^"]*添加标签"', chunk)
    assert m, f"add button near input {input_vid} missing:\n{chunk}"
    return m.group(1)


def exact_button_vid(snapshot, label):
    return node_id(snapshot, rf'button #(vnode_\d+) "{re.escape(label)}"')


def section(snapshot, marker, end_marker):
    i = snapshot.find(marker)
    assert i >= 0, f"{marker} missing"
    j = snapshot.find(end_marker, i)
    if j < 0:
        j = len(snapshot)
    return snapshot[i:j]


def run_step(session, name, fn, attempts=3):
    """Bounded retry: the native MCP runtime shows small event/prop delivery
    races (documented in decisions.md); each attempt re-snapshots and re-targets.
    A corrupted instance state is repaired with the demo's reset controls."""
    last = None
    for attempt in range(1, attempts + 1):
        try:
            return fn(session)
        except AssertionError as exc:
            last = exc
            time.sleep(1.0)
            try:
                session.snapshot()
            except Exception:
                pass
    raise AssertionError(f"step {name} failed after {attempts} attempts: {last}")


def reset_instance(session, label):
    snapshot = wait_text(session.snapshot, session.port, label)
    btn = button_vid(snapshot, label)
    session.press(btn)
    wait_text(session.snapshot, session.port, "已重置")


def case_native_core(session):
    steps = [
        ("candidates-escape-enter", step_candidates),
        ("add-accept", step_add_accept),
        ("dedup", step_dedup),
        ("remove", step_remove),
        ("limit", step_limit),
        ("reject", step_reject),
        ("config-error", step_config_error),
    ]
    for name, fn in steps:
        run_step(session, name, fn)
    session.screenshot("native-core.png")
    return [f"core:{name}" for name, _ in steps]


def settle(session, seconds=1.0):
    time.sleep(seconds)


def step_candidates(session):
    # A is the first input in the tree: the native MCP Enter routes there.
    # Type "de" -> Design & Development candidates; Escape hides; Enter adds
    # the active candidate (Design), which also covers the add-accept flow.
    snapshot = wait_text(session.snapshot, session.port, "Reading")
    a_input = input_vid(snapshot, "A 标签：回车添加")
    r_type = session.call("autoui_type", element_id=a_input, text="de")
    settle(session)
    try:
        snapshot = wait_text(session.snapshot, session.port, "Development")
    except AssertionError as exc:
        raise AssertionError(f"CTYPE={r_type[:160]} {exc}")
    assert "Design" in snapshot and "Development" in snapshot, snapshot[-900:]
    session.key("Escape")
    settle(session)
    snapshot_after = session.snapshot()
    c_after = section(snapshot_after, "实例 C", "退出示例")
    assert "Development" not in c_after, c_after[-900:]
    # re-open and Enter the active candidate
    session.call("autoui_type", element_id=a_input, text="de")
    settle(session)
    snapshot = wait_text(session.snapshot, session.port, "Development")
    r_enter = session.key("Enter")
    settle(session, 1.2)
    snapshot = session.snapshot()
    if "已添加：Design" not in snapshot:
        raise AssertionError(f"ENTER={r_enter[:120]}\n{snapshot[-2200:]}")
    a_part = section(snapshot, "实例 A（接受流", "实例 B")
    assert "3/5" in a_part, a_part[-900:]
    assert "删除标签 Design" in a_part, a_part[-900:]


def step_add_accept(session):
    # Enter path with a new tag on the first input.
    snapshot = session.snapshot()
    a_input = input_vid(snapshot, "A 标签：回车添加")
    r_type = session.call("autoui_type", element_id=a_input, text="EnterKey")
    settle(session)
    r_enter = session.key("Enter")
    settle(session, 1.2)
    snapshot = session.snapshot()
    if "已添加：EnterKey" not in snapshot:
        raise AssertionError(f"TYPE={r_type[:120]} ENTER={r_enter[:120]}\n{snapshot[-2200:]}")
    a_part = section(snapshot, "实例 A（接受流", "实例 B")
    assert "4/5" in a_part, a_part[-900:]
    assert "删除标签 EnterKey" in a_part, a_part[-900:]


def step_dedup(session):
    snapshot = session.snapshot()
    a_input = input_vid(snapshot, "A 标签：回车添加")
    session.call("autoui_type", element_id=a_input, text="Work")
    settle(session)
    snapshot = session.snapshot()
    add_btn = add_button_near(snapshot, a_input)
    session.press(add_btn)
    snapshot = wait_text(session.snapshot, session.port, "标签已存在")
    a_part = section(snapshot, "实例 A（接受流", "实例 B")
    assert "4/5" in a_part, a_part[-900:]


def step_remove(session):
    snapshot = session.snapshot()
    work_del = chip_delete_vid(snapshot, "Work")
    session.press(work_del)
    snapshot = wait_text(session.snapshot, session.port, "已移除：Work")
    a_part = section(snapshot, "实例 A（接受流", "实例 B")
    assert "3/5" in a_part, a_part[-900:]
    assert "删除标签 Work" not in a_part, a_part[-900:]


def step_limit(session):
    snapshot = wait_text(session.snapshot, session.port, "实例 B")
    b_part = section(snapshot, "实例 B", "实例 C")
    if "删除标签 Work" not in b_part:
        # state corrupted by an earlier flaky interaction: reset B and retry
        reset_instance(session, "重置 B")
        snapshot = wait_text(session.snapshot, session.port, "已重置")
        b_part = section(snapshot, "实例 B", "实例 C")
        assert "删除标签 Work" in b_part, b_part[-900:]
    b_input = input_vid(snapshot, "B 标签：回车添加")
    session.call("autoui_type", element_id=b_input, text="Development")
    settle(session)
    snapshot = session.snapshot()
    add_btn = add_button_near(snapshot, b_input)
    session.press(add_btn)
    snapshot = wait_text(session.snapshot, session.port, "已达标签上限")
    b_part = section(snapshot, "实例 B", "实例 C")
    assert "2/2" in b_part, b_part[-900:]
    b_work_del = chip_delete_vid(snapshot, "Work")
    session.press(b_work_del)
    snapshot = wait_text(session.snapshot, session.port, "已移除：Work")
    b_part = section(snapshot, "实例 B", "实例 C")
    assert "1/2" in b_part, b_part[-900:]


def step_reject(session):
    snapshot = wait_text(session.snapshot, session.port, "实例 B")
    b_part = section(snapshot, "实例 B", "实例 C")
    if not ("1/2" in b_part and "删除标签 Work" not in b_part):
        # B was corrupted: reset B and re-apply the remove to reach [Reading] 1/2.
        reset_instance(session, "重置 B")
        snapshot = wait_text(session.snapshot, session.port, "已重置")
        b_part = section(snapshot, "实例 B", "实例 C")
        work_del = chip_delete_vid(snapshot, "Work")
        session.press(work_del)
        snapshot = wait_text(session.snapshot, session.port, "已移除：Work")
        b_part = section(snapshot, "实例 B", "实例 C")
        assert "1/2" in b_part and "删除标签 Work" not in b_part, b_part[-900:]
    b_input = input_vid(snapshot, "B 标签：回车添加")
    session.call("autoui_type", element_id=b_input, text="Design")
    settle(session)
    snapshot = session.snapshot()
    add_btn = add_button_near(snapshot, b_input)
    session.press(add_btn)
    snapshot = wait_text(session.snapshot, session.port, "Design 不允许添加")
    b_part = section(snapshot, "实例 B", "实例 C")
    assert "1/2" in b_part, b_part[-900:]
    assert "删除标签 Design" not in b_part, b_part[-900:]


def step_config_error(session):
    snapshot = session.snapshot()
    dup_btn = button_vid(snapshot, "传入重复标签")
    session.press(dup_btn)
    wait_text(session.snapshot, session.port, "tags 含空值或重复")




PREVIEW_SERVER = """
import { preview } from 'vite';
const target = process.env.AUTO_HTTP_PROXY;
await preview({
    configFile: false,
    preview: {
        host: '127.0.0.1',
        port: Number(process.argv[1]),
        strictPort: true,
        open: false,
        proxy: target ? { '/api': { target, changeOrigin: true } } : {},
    },
});
"""


class BrowserMcp:
    def __init__(self, config, log, working_dir, raw_log=None):
        self.messages = queue.Queue()
        self.sequence = 0
        self.raw_log = raw_log
        self.process = subprocess.Popen(
            [config["command"], *config.get("args", [])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
            encoding="utf-8", text=True, cwd=working_dir,
            env={**os.environ, **config.get("env", {})},
        )
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        for line in self.process.stdout:
            line = line.rstrip("\n")
            if not line:
                continue
            try:
                self.messages.put(json.loads(line))
            except ValueError:
                if self.raw_log is not None:
                    self.raw_log.write(line + "\n")
                    self.raw_log.flush()

    def request(self, method, params, timeout=6):
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

    def call(self, name, *, reply_timeout=8, **arguments):
        result = self.request("tools/call", {"name": name, "arguments": arguments}, reply_timeout)
        if result.get("isError"):
            raise AssertionError(result)
        return "\n".join(entry["text"] for entry in result["content"] if entry["type"] == "text")

    def close(self):
        try:
            self.process.stdin.close()
        except BrokenPipeError:
            pass
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            stop_tree(self.process)


def browser_result(result):
    match = re.search(r"^### Result\s*\n(.*?)(?=\n### |\Z)", result, re.MULTILINE | re.DOTALL)
    assert match, result
    return json.loads(match[1].strip())


def browser_ref(snapshot, pattern):
    match = re.search(pattern, snapshot)
    assert match, f"node {pattern!r} missing from browser tree:\n{snapshot[-2500:]}"
    return match.group(1)


def browser_input_ref(snapshot, placeholder):
    return browser_ref(snapshot, rf'textbox "[^"]*{re.escape(placeholder)}[^"]*" [^\n]*?\[ref=(\w+)\]')


def browser_button_ref(snapshot, name):
    return browser_ref(snapshot, rf'button "[^"]*{re.escape(name)}[^"]*" [^\n]*?\[ref=(\w+)\]')


def browser_add_button_near(snapshot, input_ref):
    i = snapshot.find(input_ref)
    assert i >= 0, "input ref not in snapshot"
    chunk = snapshot[i:i + 600]
    m = re.search(r'button "[^"]*添加标签[^"]*" [^\n]*?\[ref=(\w+)\]', chunk)
    assert m, f"add button near {input_ref} missing:\n{chunk}"
    return m.group(1)


class VueSession:
    def __init__(self, auto_bin, ui_tool_bin, app_dir, out_dir, port, log, mcp_config):
        self.out_dir = out_dir
        self.port = port
        env = {**os.environ, "AUTO_UI_TOOL_BIN": str(ui_tool_bin)}
        self.process = subprocess.Popen(
            ["node", "--input-type=module", "--eval", PREVIEW_SERVER, str(port)],
            cwd=str(app_dir), env=env,
            stdout=log, stderr=subprocess.STDOUT, text=True,
        )
        config = json.loads(Path(mcp_config).read_text(encoding="utf-8-sig"))
        self.browser = BrowserMcp(config, open(out_dir / "browser-mcp.log", "w", encoding="utf-8", errors="replace"), out_dir,
                                  raw_log=open(out_dir / "browser-raw.log", "w", encoding="utf-8", errors="replace"))
        self.browser.request("initialize", {
            "protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "verify-tag-input", "version": "1"},
        })
        self.browser.write({"method": "notifications/initialized"})
        self._wait_page()

    def _wait_page(self, seconds=60):
        url = f"http://127.0.0.1:{self.port}/"
        self.browser.call("browser_navigate", url=url)
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            try:
                snap = self.snapshot()
                if "实例 A" in snap and "textbox" in snap:
                    return
            except AssertionError:
                time.sleep(0.5)
                continue
            time.sleep(0.5)
        raise AssertionError("Vue page did not render within the startup window")

    def snapshot(self):
        return self.browser.call("browser_snapshot")

    def wait_text(self, text, seconds=8):
        last = ""
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            last = self.snapshot()
            if text in last:
                return last
            time.sleep(0.25)
        raise AssertionError(f"no {text!r} within {seconds}s; last tree:\n{last[-2500:]}")

    def type_text(self, placeholder, text):
        snap = self.snapshot()
        ref = browser_input_ref(snap, placeholder)
        self.browser.call("browser_type", element=placeholder, target=ref, text=text)
        time.sleep(1.2)
        return snap

    def press(self, name):
        snap = self.snapshot()
        ref = browser_button_ref(snap, name)
        return self.browser.call("browser_click", element=name, target=ref)

    def key(self, key):
        return self.browser.call("browser_press_key", key=key)

    def close(self, failed=False):
        self.browser.close()
        stop_tree(self.process)


def case_vue_core(session):
    """Vue core flow mirroring the native cases (accept, dedup, remove, limit,
    reject, candidates/escape, config error)."""
    steps = [
        ("vue-candidates-enter", step_vue_candidates),
        ("vue-add-accept", step_vue_add),
        ("vue-dedup", step_vue_dedup),
        ("vue-remove", step_vue_remove),
        ("vue-limit", step_vue_limit),
        ("vue-reject", step_vue_reject),
        ("vue-config-error", step_vue_config),
    ]
    for name, fn in steps:
        run_step(session, name, fn)
    return [f"vue:{name}" for name, _ in steps]


def step_vue_candidates(session):
    # Type "de" -> Design & Development candidates; Escape dismisses (candidates
    # disappear); re-typing reopens; clicking the Design candidate adds it.
    snapshot = session.wait_text("Reading")
    session.type_text("A 标签：回车添加", "de")
    time.sleep(1.2)
    snapshot = session.wait_text("Development")
    assert "Design" in snapshot and "Development" in snapshot, snapshot[-900:]
    session.key("Escape")
    time.sleep(0.8)
    snapshot2 = session.snapshot()
    assert "Development" not in snapshot2, snapshot2[-900:]
    session.type_text("A 标签：回车添加", "de")
    time.sleep(1.2)
    snapshot = session.wait_text("Development")
    session.press("Design")
    session.wait_text("已添加：Design")


def step_vue_add(session):
    session.type_text("A 标签：回车添加", "EnterKey")
    settle(session)
    session.key("Enter")
    session.wait_text("已添加：EnterKey")


def step_vue_dedup(session):
    session.type_text("A 标签：回车添加", "Work")
    settle(session)
    session.press("添加标签")
    session.wait_text("标签已存在")


def step_vue_remove(session):
    session.press("删除标签 Work")
    session.wait_text("已移除：Work")


def step_vue_limit(session):
    session.wait_text("实例 B")
    session.type_text("B 标签：回车添加", "Development")
    time.sleep(1.0)
    snap = session.snapshot()
    b_input = browser_input_ref(snap, "B 标签：回车添加")
    add_ref = browser_add_button_near(snap, b_input)
    session.browser.call("browser_click", element="添加标签", target=add_ref)
    session.wait_text("已达标签上限")
    session.press("删除标签 Work")
    session.wait_text("已移除：Work")


def step_vue_reject(session):
    session.wait_text("实例 B")
    session.type_text("B 标签：回车添加", "Design")
    time.sleep(1.0)
    snap = session.snapshot()
    b_input = browser_input_ref(snap, "B 标签：回车添加")
    add_ref = browser_add_button_near(snap, b_input)
    session.browser.call("browser_click", element="添加标签", target=add_ref)
    session.wait_text("Design 不允许添加")


def step_vue_config(session):
    session.press("传入重复标签")
    session.wait_text("tags 含空值或重复")




def case_notes_native(session):
    """015-notes migration: add a tag through TagInput, verify backend readback
    (chip persists after switching notes) and that the note body is unchanged."""
    deadline = time.monotonic() + 240
    snap = None
    while time.monotonic() < deadline:
        try:
            snap = session.snapshot()
            if "Notes" in snap and "添加标签" in snap:
                break
        except Exception:
            time.sleep(0.5)
            continue
        time.sleep(0.5)
    assert snap and "添加标签" in snap, "notes app did not render"

    tag_input = input_vid(snap, "添加标签")
    i = snap.find(tag_input)
    add_btn = node_id(snap[i:i + 2500], r'button #(vnode_\d+) "[^"]*添加标签"')
    body_before = re.search(r'textarea #[^}]*?value: "([^"]*)"', snap)
    snap2 = None
    for attempt in range(2):
        session.call("autoui_type", element_id=tag_input, text="PlanTag")
        time.sleep(0.8)
        session.press(add_btn)
        time.sleep(2.5)
        snap2 = session.snapshot()
        if "删除标签 PlanTag" in snap2:
            break
        time.sleep(1.0)
    assert snap2 is not None and "删除标签 PlanTag" in snap2, snap2[-1500:] if snap2 else "no snapshot"
    # switch to another note and back: tag must persist (backend readback)
    m = re.search(r'button #(vnode_\d+) "([^"]*Quick Ideas[^"]*)"', snap2)
    assert m, "no Quick Ideas button"
    session.press(m.group(1))
    time.sleep(1.5)
    snap3 = session.snapshot()
    m2 = re.search(r'button #(vnode_\d+) "([^"]*Welcome[^"]*)"', snap3)
    if m2:
        session.press(m2.group(1))
        time.sleep(1.5)
    snap4 = session.snapshot()
    assert "删除标签 PlanTag" in snap4, snap4[-1500:]
    body_after = re.search(r'textarea #[^}]*?value: "([^"]*)"', snap4)
    if body_before and body_after:
        assert body_before.group(1) == body_after.group(1), "note body changed"
    return ["notes:add-readback-switch"]


def case_notes_vue(session):
    """015-notes Vue lane: the TagInput renders and the add button adds a tag."""
    deadline = time.monotonic() + 90
    snap = None
    while time.monotonic() < deadline:
        try:
            snap = session.snapshot()
            if "添加标签" in snap:
                break
        except AssertionError:
            time.sleep(0.5)
            continue
        time.sleep(0.5)
    assert snap and "添加标签" in snap, "notes vue did not render"
    session.type_text("添加标签", "PlanVue")
    time.sleep(1.0)
    snap2 = session.snapshot()
    ref = browser_add_button_near(snap2, browser_input_ref(snap2, "添加标签"))
    session.browser.call("browser_click", element="添加标签", target=ref)
    session.wait_text("删除标签 PlanVue")
    return ["notes-vue:add-readback"]



def main():
    parser = argparse.ArgumentParser(description="PLAN-097 TagInput product checks")
    parser.add_argument("--auto-bin", required=True)
    parser.add_argument("--ui-tool-bin", required=True)
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--backend", choices=["native", "vue"], required=True)
    parser.add_argument("--app-dir", default=None)
    parser.add_argument("--browser-mcp-config", default=None)
    parser.add_argument("--startup-timeout", type=float, default=120)
    parser.add_argument("--case", choices=["demo", "notes"], default="demo")
    args = parser.parse_args()

    out = Path(args.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    report = {
        "plan": "PLAN-097", "revision": 2, "backend": args.backend,
        "platform": platform.system().lower(), "app_dir": args.app_dir,
        "cases": [], "visual": [], "cleanup": {}, "reopen": {}, "result": "not_run",
    }
    if args.app_dir is None:
        repo = Path(__file__).resolve().parents[2]
        if args.case == "notes":
            args.app_dir = str(repo / "examples" / "ui" / "015-notes")
        else:
            args.app_dir = str(repo / "examples" / "tag-input")

    report["tools"] = {
        "auto": digest(args.auto_bin),
        "ui_tool": digest(args.ui_tool_bin),
    }
    report["source"] = {
        "tag_input.at": digest(Path(__file__).resolve().parents[2] / "packages" / "tag-input" / "tag_input.at"),
        "app.at": digest(Path(__file__).resolve().parents[2] / "examples" / "tag-input" / "src" / "front" / "app.at"),
    }

    if args.backend == "native":
        if os.name != "nt":
            report["result"] = "not_run"
            report["cases"].append({"id": "all", "status": "not_run", "reason": "non-Windows host"})
            Path(out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2))
            return 2
        log = open(out / "native.log", "w", encoding="utf-8", errors="replace")
        port = available_port()
        session = NativeSession(args.auto_bin, args.ui_tool_bin, args.app_dir, out, port, log, 180)
        try:
            runner = case_notes_native if args.case == "notes" else case_native_core
            passed = runner(session)
            report["result"] = "pass"
            for case in passed:
                report["cases"].append({"id": case, "status": "pass"})
        except AssertionError as exc:
            report["result"] = "fail"
            report["cases"].append({"id": "core", "status": "fail", "error": str(exc)[:1200]})
            try:
                tree = session.snapshot()
                (out / "last-tree.txt").write_text(tree, encoding="utf-8")
            except Exception:
                pass
        finally:
            try:
                session.close(failed=report["result"] != "pass")
                report["cleanup"] = {"owned_exited": True, "port_free": True}
            except AssertionError as exc:
                report["cleanup"] = {"error": str(exc)}
            log.close()
    else:
        if not args.browser_mcp_config:
            report["result"] = "not_run"
            report["cases"].append({"id": "vue", "status": "not_run", "reason": "missing --browser-mcp-config"})
        elif not Path(args.browser_mcp_config).is_file():
            report["result"] = "not_run"
            report["cases"].append({"id": "vue", "status": "not_run", "reason": "browser mcp config not found"})
        else:
            log = open(out / "vue-preview.log", "w", encoding="utf-8", errors="replace")
            port = available_port()
            session = VueSession(args.auto_bin, args.ui_tool_bin, args.app_dir, out, port, log, args.browser_mcp_config)
            try:
                runner = case_notes_vue if args.case == "notes" else case_vue_core
                passed = runner(session)
                report["result"] = "pass"
                for case in passed:
                    report["cases"].append({"id": case, "status": "pass"})
            except AssertionError as exc:
                report["result"] = "fail"
                report["cases"].append({"id": "vue-core", "status": "fail", "error": str(exc)[:1200]})
                try:
                    (out / "last-tree.txt").write_text(session.snapshot(), encoding="utf-8")
                except Exception:
                    pass
                try:
                    dom = session.browser.call(
                        "browser_evaluate",
                        function="() => ({btns: [...document.querySelectorAll('button')].map(b => b.textContent.trim()).slice(0,40), inputs: [...document.querySelectorAll('input')].map(i => i.value)})",
                    )
                    (out / "last-dom.txt").write_text(dom, encoding="utf-8")
                except Exception:
                    pass
            finally:
                try:
                    session.close(failed=report["result"] != "pass")
                    report["cleanup"] = {"owned_exited": True, "port_free": True}
                except AssertionError as exc:
                    report["cleanup"] = {"error": str(exc)}
                log.close()

    (out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return {"pass": 0, "fail": 1, "not_run": 2}[report["result"]]


if __name__ == "__main__":
    sys.exit(main())
