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




def main():
    parser = argparse.ArgumentParser(description="PLAN-097 TagInput product checks")
    parser.add_argument("--auto-bin", required=True)
    parser.add_argument("--ui-tool-bin", required=True)
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--backend", choices=["native", "vue"], required=True)
    parser.add_argument("--app-dir", default=None)
    parser.add_argument("--browser-mcp-config", default=None)
    parser.add_argument("--startup-timeout", type=float, default=120)
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
        args.app_dir = str(repo / "examples" / "tag-input")

    report["tools"] = {
        "auto": digest(args.auto_bin),
        "ui_tool": digest(args.ui_tool_bin),
    }
    report["source"] = {
        "tag_input.at": digest(Path(args.app_dir).parents[1] / "packages" / "tag-input" / "tag_input.at"),
        "app.at": digest(Path(args.app_dir) / "src" / "front" / "app.at"),
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
            passed = case_native_core(session)
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
        report["result"] = "not_run"
        report["cases"].append({"id": "vue", "status": "not_run", "reason": "vue lane pending in probe"})

    (out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return {"pass": 0, "fail": 1, "not_run": 2}[report["result"]]


if __name__ == "__main__":
    sys.exit(main())
