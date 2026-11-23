"""Windows Native VM and Vue product gates for the prepared command palette apps."""

import argparse
import json
import os
import platform
import re
import time
from pathlib import Path

from verify_collection_view import wait_for
from verify_prompt_dialog import ProductSession, SYSTEM_READER, digest


class PaletteSession(ProductSession):
    demo_anchor = "Theme setting: light"
    demo_exit = "Exit example"
    browser_client_name = "verify-command-palette"
    native_window_exit_cases = ProductSession.native_window_exit_cases | {"reopen-app"}

    def startup_anchor(self):
        return {"launcher": "Open launcher", "provided": "Open provided palette"}.get(
            self.args.consumer, self.demo_anchor)

    def system_reader(self):
        script = SYSTEM_READER.replace("分组名称", "Search commands").replace("保存分组", "Theme settings")
        return script.replace("系统输入", "外观")

    def open_palette(self):
        if self.args.consumer == "launcher":
            tree = self.snapshot()
            if self.browser is not None:
                node = re.search(r'button ".*Open launcher[^"]*"[^\n]*\[ref=(\w+)\]', tree)
                self.call("browser_click", element="Open launcher", target=node[1])
            else:
                node = re.search(r'button #(vnode_\d+) "[^"]*Open launcher', tree)
                self.call("autoui_action", element_id=node[1], action="press")
            self.wait_text("Search apps…", "Calculator")
        elif self.args.consumer == "provided":
            self.press("Open provided palette")
            self.wait_text("Search commands", "First result")
        else:
            self.press("Open command palette")
            self.wait_text("Search commands", "Dark theme")

    def search(self, value, *expected, absent=()):
        caption = "Search apps…" if self.args.consumer == "launcher" else "Search commands"
        self.enter_text(value, caption)
        def current(tree):
            if self.browser is None:
                field = rf'input #[^ ]+ \{{[^}}]*placeholder: "{re.escape(caption)}"[^}}]*value: "{re.escape(value)}"'
            else:
                field = rf'textbox "{re.escape(caption)}"[^\n]*\]: {re.escape(value)}(?:\n|$)'
            return re.search(field, tree) and all(fragment in tree for fragment in expected) and not any(
                fragment in tree for fragment in absent)
        wait_for(self.snapshot, current, seconds=3)

    def key(self, key, modifiers=()):
        result = super().key(key, modifiers)
        assert not result.startswith("Error:"), result
        if self.browser is None:
            # The native RPC reports enqueueing. Iced then resolves and visits
            # the focused widget before publishing the generated message.
            time.sleep(0.22)
        return result


def local_settings(session):
    session.open_palette()
    session.search("theme", "Dark theme", "Light theme", "Theme settings", absent=("Open file",))
    session.key("ArrowUp")
    session.key("Enter")
    session.wait_text("Theme setting: dark", "Applied commands: 1", absent=("Search commands",))
    session.open_palette()
    session.search("外观", "Theme settings", absent=("Light theme", "Dark theme"))
    session.key("Enter")
    session.wait_text("Theme settings are open", "Theme setting: dark", "Applied commands: 2")


def reopened_settings(session):
    session.open_palette()
    session.search("night", "Dark theme", absent=("Light theme",))
    session.key("Enter")
    session.wait_text("Theme setting: dark", "Applied commands: 1")


def empty_disabled_and_reopen(session):
    background = session.button("Open command palette") if session.browser is None else None
    session.open_palette()
    old_cancel = session.button("Cancel")
    if background is not None:
        refused = session.call("autoui_action", element_id=background, action="press")
        assert refused.startswith("Error:"), refused
    session.search("管理", "由组织管理的设置不能从这个设备修改", absent=("Open file",))
    session.key("Enter")
    session.wait_text("Applied commands: 0", "Search commands")
    session.search("no-such-command", "No matching commands")
    session.key("Enter")
    session.wait_text("No matching commands", "Applied commands: 0")
    session.key("Escape")
    session.wait_text("Theme setting: light", "Applied commands: 0", absent=("Search commands",))
    if session.browser is not None:
        result = session.browser.evaluate("() => document.activeElement?.textContent?.trim()")
        assert result == "Open command palette", result
    session.open_palette()
    assert session.button("Cancel") != old_cancel
    if session.browser is None:
        refused = session.call("autoui_action", element_id=old_cancel, action="press")
        assert refused.startswith("Error:"), refused
    session.search("布局", "Reset settings")
    session.press("Reset settings")
    session.wait_text("Layout settings reset", "Applied commands: 1")


def rejection(session):
    session.open_palette()
    session.search("file", "Open file", absent=("Theme settings",))
    session.key("Enter")
    session.wait_text("File access is unavailable in Settings.", "Applied commands: 0")
    session.search("night", "Dark theme", absent=("Open file",))
    session.key("Enter")
    session.wait_text("Theme setting: dark", "Applied commands: 1")


def visual(session):
    session.open_palette()
    session.search("theme", "Theme settings", absent=("Open file",))
    session.screenshot("theme-light")
    session.key("Enter")
    session.wait_text("Theme setting: dark")
    session.open_palette()
    session.search("管理", "由组织管理的设置不能从这个设备修改")
    if session.browser is not None:
        rgb = session.browser.evaluate("""() => getComputedStyle(
            document.querySelector('[role="dialog"]')).backgroundColor""")
        assert max(map(int, re.findall(r"\d+", rgb))) < 64, rgb
    session.screenshot("narrow-dark-cjk")
    session.press("Cancel")
    session.wait_text("Theme setting: dark", absent=("Search commands",))


def composition(session):
    session.open_palette()
    session.search("night", "Dark theme", absent=("Light theme",))
    assert session.browser is not None
    result = session.browser_result("browser_run_code_unsafe", code="""async page => {
      const input = page.getByRole('textbox', { name: 'Search commands' });
      return await input.evaluate(el => {
        el.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
        el.dispatchEvent(new CompositionEvent('compositionupdate', { data: '外观', bubbles: true }));
        el.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', isComposing: true, bubbles: true }));
        el.dispatchEvent(new CompositionEvent('compositionend', { data: '外观', bubbles: true }));
        return { enabled: !el.disabled };
      });
    }""")
    assert result == {"enabled": True}, result
    session.wait_text("Applied commands: 0", "Search commands", absent=("Executing…",))
    session.search("外观", "Theme settings", absent=("Dark theme",))
    session.key("Enter")
    session.wait_text("Theme settings are open", "Applied commands: 1")


def system_accessibility(session):
    session.open_palette()
    action = "system-invoke" if session.args.case == "system-invoke" else "system-input"
    initial = "外观" if action == "system-invoke" else "theme"
    session.search(initial, "Theme settings")
    window = session.windows[0]
    identity = session.output / "system-window.pending"
    identity.write_text(f'{window["handle"]} {window["pid"]} {action}', encoding="ascii")
    identity.replace(session.output / "system-window.txt")
    raw, _ = session.system_helper.communicate(timeout=6)
    assert session.system_helper.returncode == 0, "Windows UIA helper failed"
    evidence = json.loads(raw.decode("utf-8-sig"))
    (session.output / "system-tree.json").write_text(
        json.dumps(evidence, ensure_ascii=False, indent=2), encoding="utf-8",
    )
    field = next(row for row in evidence["nodes"] if row["role"] == "ControlType.Edit")
    button = next(row for row in evidence["nodes"] if row["role"] == "ControlType.Button")
    assert field["value"] == initial and field["focus"] and field["width"] > 0 and field["height"] > 0, field
    assert button["name"] == "Theme settings" and button["enabled"] and button["width"] > 0 and button["height"] > 0, button
    if action == "system-invoke":
        session.wait_text("Theme settings are open", "Applied commands: 1")
    else:
        session.wait_text("外观", "Applied commands: 0", absent=("Dark theme",))


def provided_results(session):
    session.open_palette()
    session.search("A", "First result", "Middle result", "Final result")
    session.search("B", "First result")
    session.search("A", "Searching…", absent=("Old result",))
    session.key("Enter")
    session.wait_text("Provided commands applied: 0")
    tree = session.wait_text("First result", "Middle result", "Final result", absent=("Searching…",))
    assert tree.index("First result") < tree.index("Middle result") < tree.index("Final result")
    session.key("ArrowUp")
    session.key("Enter")
    session.wait_text("Provided commands applied: 1", "three", absent=("Search commands",))


def launcher_no_host(session):
    session.open_palette()
    session.search("cal", "Calculator", absent=("Stopwatch", "Weather"))
    session.key("Enter")
    session.wait_text("App launching requires the AutoOS desktop.", "Search apps…", absent=("Last launched:",))
    session.press("Grid view")
    session.wait_text("Calculator", "Weather", "Tab palette")


CASES = {
    "settings": (local_settings, "C-01/C-02/C-04", "MCP and application model"),
    "reopen-dialog": (empty_disabled_and_reopen, "C-01/C-02/C-05", "MCP and application model"),
    "rejected": (rejection, "C-04/C-06", "MCP and application model"),
    "visual": (visual, "X-03", "rendered screenshots"),
    "ime": (composition, "C-02/X-05", "Vue input adapter"),
    "system-input": (system_accessibility, "X-07", "Windows UI Automation"),
    "system-invoke": (system_accessibility, "X-07", "Windows UI Automation"),
    "provided": (provided_results, "C-03/C-06", "provided query and current command model"),
    "launcher": (launcher_no_host, "C-04", "standalone Launcher no-host and grid"),
}


def run_session(args, output, check):
    output.mkdir(parents=True, exist_ok=True)
    result = {}
    with (output / "rpc.ndjson").open("w", encoding="utf-8") as journal:
        session = PaletteSession(args, output, journal)
        start = time.monotonic()
        try:
            session.start()
            result.update(identity=session.identity, provider=getattr(session, "provider", None),
                          startup_seconds=time.monotonic() - start)
            check_started = time.monotonic()
            check(session)
            result.update(result="pass", action_seconds=time.monotonic() - check_started)
        except Exception as error:
            result.update(result="fail", error=repr(error))
        finally:
            (output / "last-tree.txt").write_text(session.last_snapshot, encoding="utf-8")
            cleanup_started = time.monotonic()
            try:
                session.close(failed=result["result"] != "pass")
                result["cleanup"] = "pass"
            except Exception as error:
                result.update(result="fail", cleanup=repr(error))
            result["cleanup_seconds"] = time.monotonic() - cleanup_started
            result["elapsed_seconds"] = time.monotonic() - start
    return result


def source_hashes(path, backend):
    files = ("pac.at", "src/front/app.at") if backend == "native" else (
        "package.json", "pnpm-lock.yaml", "src/App.vue", "src/components/CommandPalette.vue",
        "dist/index.html", "dist/assets/index.css", "dist/assets/index.js",
    )
    return {name: digest(path / name) for name in files if (path / name).is_file()}


def main():
    started = time.monotonic()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", required=True, type=Path)
    parser.add_argument("--ui-tool-bin", required=True, type=Path)
    parser.add_argument("--out-dir", required=True, type=Path)
    parser.add_argument("--backend", required=True, choices=["native", "vue"])
    parser.add_argument("--app-dir", required=True, type=Path,
                        help="isolated native project or prepared Vue project")
    parser.add_argument("--browser-mcp-config", type=Path)
    parser.add_argument("--case", choices=[*CASES, "reopen-app"], default="settings")
    parser.add_argument("--startup-timeout", type=float, default=20)
    args = parser.parse_args()
    if not 0 < args.startup_timeout <= 300:
        parser.error("--startup-timeout must be within (0, 300]")
    for option in ("auto_bin", "ui_tool_bin", "out_dir", "app_dir", "browser_mcp_config"):
        path = getattr(args, option)
        if path is not None:
            setattr(args, option, path.resolve())
    args.consumer = args.case if args.case in {"launcher", "provided"} else "demo"
    args.out_dir.mkdir(parents=True)
    report = {"plan": "PLAN-096", "revision": 3, "platform": platform.system(),
              "backend": args.backend, "case": args.case, "app_dir": str(args.app_dir),
              "runner": digest(__file__), "startup_limit_seconds": args.startup_timeout}
    if os.name != "nt" or not args.auto_bin.is_file() or not args.ui_tool_bin.is_file() or not args.app_dir.is_dir():
        report.update(result="not_run", error="Windows and prepared source-bound tools and project required")
    elif args.backend == "vue" and (not args.browser_mcp_config or not args.browser_mcp_config.is_file()
                                    or not (args.app_dir / "dist/index.html").is_file()
                                    or not (args.app_dir / "node_modules/vite/bin/vite.js").is_file()):
        report.update(result="not_run", error="Prepared Vue frontend and browser MCP provider required")
    elif (args.case.startswith("system-") and args.backend != "native") or (args.case == "ime" and args.backend != "vue"):
        report.update(result="not_run", error="This check needs its declared input provider")
    else:
        report["tools"] = {"auto": digest(args.auto_bin), "auto-ui-tool": digest(args.ui_tool_bin)}
        if args.browser_mcp_config:
            report["tools"]["browser-mcp-config"] = digest(args.browser_mcp_config)
        report["source"] = source_hashes(args.app_dir, args.backend)
        if args.case == "reopen-app":
            report.update(coverage="X-02/X-06", layer="MCP, app and lifecycle")
            first = run_session(args, args.out_dir / "first", lambda s: s.open_palette())
            report.update(first=first, result=first["result"])
            if first["result"] == "pass":
                report["reopen"] = run_session(args, args.out_dir / "reopened", reopened_settings)
                report["result"] = report["reopen"]["result"]
                if report["result"] == "pass":
                    previous = first["identity"]
                    current = report["reopen"]["identity"]
                    if previous["front_pid"] == current["front_pid"]:
                        report.update(result="fail", error="Reopen reused the prior application PID")
        else:
            check, coverage, layer = CASES[args.case]
            report.update(coverage=coverage, layer=layer)
            report.update(run_session(args, args.out_dir, check))
        report["artifacts"] = {
            file.relative_to(args.out_dir).as_posix(): digest(file)
            for file in args.out_dir.rglob("*")
            if file.is_file() and (file.suffix in {".png", ".log", ".ndjson"} or
                                   file.name in {"last-tree.txt", "system-tree.json"})
        }
    report["wall_seconds"] = time.monotonic() - started
    if report["result"] == "pass" and report["wall_seconds"] > 10:
        report.update(result="fail", error="Product case exceeded the 10-second wall-time limit")
    (args.out_dir / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return {"pass": 0, "fail": 1, "not_run": 2}[report["result"]]


if __name__ == "__main__":
    raise SystemExit(main())
