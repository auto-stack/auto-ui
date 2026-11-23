"""Windows Native VM and Vue product gates for the prepared command palette apps."""

import argparse
import json
import os
import platform
import re
import time
from pathlib import Path

from prompt_dialog_preview import checked_preview
from verify_collection_view import wait_for
from verify_prompt_dialog import ProductSession, SYSTEM_READER, digest


class PaletteSession(ProductSession):
    demo_anchor = "Theme setting: light"
    demo_exit = "Exit example"
    browser_client_name = "verify-command-palette"
    # After invoking the Windows provider, close through the application's
    # normal Exit action so pending accessibility notifications can settle.
    native_window_exit_cases = (ProductSession.native_window_exit_cases - {"system-invoke"}) | {"reopen-app"}

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


def duplicate_labels(session):
    session.open_palette()
    session.search("重置", "Reset settings", absent=("Open file", "Dark theme"))
    session.key("Enter")
    session.wait_text("General settings reset", "Applied commands: 1", absent=("Search commands",))
    session.open_palette()
    session.search("重置", "Reset settings", absent=("Open file", "Dark theme"))
    session.key("ArrowDown")
    session.key("Enter")
    session.wait_text("Layout settings reset", "Applied commands: 2", absent=("Search commands",))


def rejection(session):
    session.open_palette()
    session.search("file", "Open file", absent=("Theme settings",))
    session.key("Enter")
    session.wait_text("File access is unavailable in Settings.", "Applied commands: 0")
    session.screenshot("rejected")
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


def scroll_results(session):
    session.open_palette()
    if session.browser is None:
        def overflow():
            panes = json.loads(session.call("autoui_scroll_state"))["panes"]
            return next((pane for pane in panes if pane["content"][1] > pane["viewport_rect"][3]), None)

        pane = wait_for(overflow, bool, seconds=3)
        assert pane["offset"][1] == 0, pane
        result = session.call("autoui_action", element_id=pane["key"], action="scroll", value=200)
        assert not result.startswith("Error:"), result
        wait_for(overflow, lambda current: current and current["offset"][1] > 0, seconds=3)
    else:
        moved = session.browser_result("browser_run_code_unsafe", code="""async page => {
          const list = page.getByRole('dialog').locator('.overflow-y-auto');
          await list.hover();
          await page.mouse.wheel(0, 200);
          await list.evaluate(el => new Promise(resolve => requestAnimationFrame(resolve)));
          return await list.evaluate(el => el.scrollHeight > el.clientHeight && el.scrollTop > 0);
        }""")
        assert moved is True
    session.press("Cancel")
    session.wait_text("Applied commands: 0", absent=("Search commands",))


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
    session.press("Final result")
    session.wait_text("Provided commands applied: 1", "three", absent=("Search commands",))


def provided_wrap(session):
    session.open_palette()
    session.key("ArrowUp")
    session.key("Enter")
    session.wait_text("Provided commands applied: 1", "three", absent=("Search commands",))


def provided_pending(session):
    session.open_palette()
    session.press("Middle result")
    session.wait_text("Executing…", "Provided commands applied: 0")
    session.screenshot("pending")
    session.key("Escape")
    session.wait_text("Provided commands applied: 1", "two", absent=("Search commands",))


def provided_stale_pending(session):
    session.open_palette()
    session.search("A", "Middle result")
    session.press("Middle result")
    session.wait_text("Obsolete result published", "Executing…", "Provided invocation requests: 1")
    session.key("Escape")
    session.key("Enter")
    session.wait_text("Provided commands applied: 1", "Provided invocation requests: 1", "two",
                      absent=("Search commands",))


def provided_configuration(session):
    session.press("Open invalid provided palette")
    session.wait_text("Command IDs must be unique.", "duplicate-command-id",
                      "Configuration notifications: 1", absent=("Searching…",))
    session.key("Enter")
    session.wait_text("Configuration notifications: 1", "Provided invocation requests: 0")
    session.screenshot("invalid-config")
    session.press("Repair configuration")
    session.wait_text("First result", absent=("Command IDs must be unique.", "Searching…"))
    session.press("First result")
    session.wait_text("Provided commands applied: 1", "Provided invocation requests: 1", "one",
                      absent=("Search commands",))


def launcher_no_host(session):
    session.open_palette()
    session.key("ArrowUp")
    session.key("Enter")
    session.wait_text("App launching requires the AutoOS desktop. Auto Edit")
    session.search("cal", "Calculator", absent=("Stopwatch", "Weather"))
    session.key("Enter")
    session.wait_text("App launching requires the AutoOS desktop. Calculator", "Search apps…", absent=("Last launched:",))
    session.press("Grid view")
    session.wait_text("Calculator", "Weather", "Tab palette")


class ExpectedActionFailure(Exception):
    pass


def interrupt_active_session(session):
    session.open_palette()
    session.search("night", "Dark theme", absent=("Light theme",))
    raise ExpectedActionFailure("controlled interruption after the live query")


CASES = {
    "settings": (local_settings, "C-01/C-02/C-04", "MCP and application model"),
    "reopen-dialog": (empty_disabled_and_reopen, "C-01/C-02/C-05", "MCP and application model"),
    "duplicate-labels": (duplicate_labels, "C-01", "MCP selection identity and application model"),
    "failure-cleanup": (interrupt_active_session, "X-06", "forced interruption after a live action"),
    "rejected": (rejection, "C-04/C-06", "MCP and application model"),
    "visual": (visual, "X-03", "rendered screenshots"),
    "scroll": (scroll_results, "X-03", "native scroll and browser wheel"),
    "ime": (composition, "C-02/X-05", "Vue input adapter"),
    "system-input": (system_accessibility, "X-07", "Windows UI Automation"),
    "system-invoke": (system_accessibility, "X-07", "Windows UI Automation"),
    "provided-pending": (provided_pending, "C-06/X-03", "pending close lock and appearance"),
    "provided-stale-pending": (provided_stale_pending, "C-03/C-06", "obsolete results during a live invocation"),
    "provided-config": (provided_configuration, "C-05/X-03/X-04", "first configuration error and recovery"),
    "provided-wrap": (provided_wrap, "C-02", "provided keyboard navigation"),
    "provided": (provided_results, "C-03/C-06", "provided query and current command model"),
    "launcher": (launcher_no_host, "C-04", "standalone Launcher no-host and grid"),
}

PROJECTS = {
    "provided": "tests/product/fixtures/command-palette-provided",
    "launcher": "examples/ui/028-launcher",
    "demo": "examples/command-palette",
}

EXIT_STATUS = {"pass": 0, "fail": 1, "not_run": 2}
PREPARED = "command-palette-prepared.json"
REPOSITORY = Path(__file__).resolve().parents[2]


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
            if isinstance(error, ExpectedActionFailure):
                result["expected_interruption"] = True
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


def source_hashes(path):
    manifest = path / "pac.at"
    if not manifest.is_file() or not (path / "src/front/app.at").is_file():
        raise ValueError("prepared application manifest or frontend source is missing")
    hashes = {"pac.at": digest(manifest)}
    for source in sorted((path / "src/front").rglob("*.at")):
        hashes[source.relative_to(path).as_posix()] = digest(source)
    root = next((parent for parent in path.parents if
                 (parent / "packages/command-palette/package.at").is_file()), None)
    if root is None:
        raise ValueError("prepared CommandPalette package closure is missing")
    for package in ("command-palette", "themes"):
        for source in sorted((root / "packages" / package).glob("*")):
            if source.is_file() and source.suffix in {".at", ".json"}:
                hashes[source.relative_to(root).as_posix()] = digest(source)
    return hashes


def preparation_inputs(args):
    if os.name != "nt" or not args.auto_bin.is_file() or not args.ui_tool_bin.is_file():
        raise ValueError("Windows and prepared Auto/AutoUI executables are required")
    inputs = {"backend": args.backend, "consumer": args.consumer,
              "tools": {"auto": digest(args.auto_bin), "auto-ui-tool": digest(args.ui_tool_bin)}}
    if args.backend == "vue":
        if args.app_dir.parts[-3:] != ("gen", "front", "vue"):
            raise ValueError("prepared Vue project must be the consumer's gen/front/vue directory")
        preview = checked_preview(args.app_dir)
        inputs["preview"] = {"source": preview["source"], "dist": preview["dist"]}
        source_dir = args.app_dir.parents[2]
    else:
        source_dir = args.app_dir
    inputs["source"] = source_hashes(source_dir)
    if inputs["source"] != source_hashes(REPOSITORY / PROJECTS[args.consumer]):
        raise ValueError("prepared sources differ from the checked-out consumer or package closure")
    return inputs


def checked_preparation(args):
    receipt_path = args.app_dir / PREPARED
    if not receipt_path.is_file():
        raise ValueError(f"prepared input receipt {PREPARED} is missing")
    expected = json.loads(receipt_path.read_text(encoding="utf-8"))
    actual = preparation_inputs(args)
    if expected != actual:
        fields = [name for name, value in actual.items()
                  if not isinstance(expected, dict) or expected.get(name) != value]
        raise ValueError(f"prepared input receipt differs: {', '.join(fields)}")
    if args.backend == "native":
        if (os.getenv("AUTO_BACKEND_IMPL", "vm") != "vm" or os.getenv("AUTO_VM_MERGE") == "0"
            or os.getenv("AUTO_VM_WITH_HTTP") == "1"):
            raise ValueError("CommandPalette Native VM cases require the merged VM backend")
    return actual


def run_case(args):
    started = time.monotonic()
    args.out_dir.mkdir(parents=True)
    report = {"plan": "PLAN-096", "revision": 3, "platform": platform.system(),
              "backend": args.backend, "case": args.case, "app_dir": str(args.app_dir),
              "runner": digest(__file__), "startup_limit_seconds": args.startup_timeout}
    try:
        if args.backend == "vue" and (not args.browser_mcp_config or not args.browser_mcp_config.is_file()
                                      or not (args.app_dir / "node_modules/vite/bin/vite.js").is_file()):
            raise ValueError("prepared Vue frontend and browser MCP provider are required")
        if (args.case.startswith("system-") and args.backend != "native") or (args.case == "ime" and args.backend != "vue"):
            raise ValueError("this case needs its declared input provider")
        prepared = checked_preparation(args)
    except (OSError, ValueError) as error:
        report.update(result="not_run", error=str(error))
    else:
        report["tools"] = prepared["tools"]
        report["preparation"] = digest(args.app_dir / PREPARED)
        if args.browser_mcp_config:
            report["tools"]["browser-mcp-config"] = digest(args.browser_mcp_config)
        report["source"] = prepared["source"]
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
            if args.case == "failure-cleanup" and report.get("expected_interruption") and report.get("cleanup") == "pass":
                report["interruption"] = report.pop("error")
                report["result"] = "pass"
        report["artifacts"] = {
            file.relative_to(args.out_dir).as_posix(): digest(file)
            for file in args.out_dir.rglob("*")
            if file.is_file() and (file.suffix in {".png", ".log", ".ndjson"} or
                                   file.name in {"last-tree.txt", "system-tree.json"})
        }
    report["wall_seconds"] = time.monotonic() - started
    if report["result"] == "pass" and report["wall_seconds"] > 10:
        report.update(result="fail", error="Product case exceeded the 10-second wall-time limit")
    write_report(args.out_dir, report)
    return report


def write_report(output, report):
    (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--auto-bin", required=True, type=Path)
    parser.add_argument("--ui-tool-bin", required=True, type=Path)
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument("--backend", required=True, choices=["native", "vue"])
    parser.add_argument("--app-dir", type=Path, help="prepared isolated project for this case")
    parser.add_argument("--project-root", type=Path, help="root of the three isolated projects")
    parser.add_argument("--browser-mcp-config", type=Path)
    parser.add_argument("--case", choices=[*CASES, "reopen-app"], default="settings")
    parser.add_argument("--prepare", action="store_true", help="record fingerprints for a prebuilt input set")
    parser.add_argument("--startup-timeout", type=float, default=20)
    args = parser.parse_args()
    if not 0 < args.startup_timeout <= 300:
        parser.error("--startup-timeout must be within (0, 300]")
    if args.out_dir is None and not args.prepare:
        parser.error("--out-dir is required to run a case")
    for option in ("auto_bin", "ui_tool_bin", "out_dir", "app_dir", "project_root", "browser_mcp_config"):
        path = getattr(args, option)
        if path is not None:
            setattr(args, option, path.resolve())
    args.consumer = "provided" if args.case.startswith("provided") else "launcher" if args.case == "launcher" else "demo"
    if args.app_dir is None:
        if args.out_dir is None and args.project_root is None:
            parser.error("--prepare requires --app-dir or --project-root")
        root = args.project_root or args.out_dir.parent / "windows"
        args.app_dir = root / PROJECTS[args.consumer]
        if args.backend == "vue":
            args.app_dir /= "gen/front/vue"
    if args.prepare:
        try:
            if args.app_dir.is_relative_to(REPOSITORY):
                raise ValueError("prepare an isolated copy of the project outside the repository")
            prepared = preparation_inputs(args)
        except (OSError, ValueError) as error:
            parser.error(str(error))
        (args.app_dir / PREPARED).write_text(json.dumps(prepared, ensure_ascii=False, indent=2), encoding="utf-8")
        return 0
    report = run_case(args)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return EXIT_STATUS[report["result"]]


if __name__ == "__main__":
    raise SystemExit(main())
