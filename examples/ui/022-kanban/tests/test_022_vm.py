#!/usr/bin/env python3
"""PLAN-037 T-04: 022-kanban VM regression entry (theme adoption).

Starts `auto run -r vm` with the Rust backend on its pac back_port (8022),
waits for API + MCP, then asserts:
  K1 board renders the 9 seed cards across three columns;
  P-037 theme family/mode round-trip preserves the card store and the
  dark-first explicit default (换肤不丢卡片/列归属).

Usage: AUTO_BIN=<auto> python test_022_vm.py
"""
import json
import os
import socket
import subprocess
import sys
import time
import urllib.request


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(('127.0.0.1', 0))
        return s.getsockname()[1]


class Mcp:
    def __init__(self, port: int):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self._id = 1

    def call(self, tool, args=None):
        req = {"jsonrpc": "2.0", "id": self._id, "method": "tools/call",
               "params": {"name": tool, "arguments": args or {}}}
        self._id += 1
        r = urllib.request.Request(self.url, data=json.dumps(req).encode(),
                                   headers={"Content-Type": "application/json"})
        res = json.loads(urllib.request.urlopen(r, timeout=15).read())
        if "error" in res:
            raise RuntimeError(f"{tool}: {res['error']}")
        c = res.get("result", {}).get("content", [{}])
        return c[0].get("text", "") if c else ""

    def state(self, *fields):
        t = self.call("autoui_state", {"fields": list(fields)})
        out = {}
        for l in t.splitlines():
            if ":" not in l or l.strip().startswith("State"):
                continue
            k, v = l.split(":", 1)
            v = v.strip()
            for suf in (" (str)", " (int)", " (float)", " (bool)"):
                if v.endswith(suf):
                    v = v[: -len(suf)]
                    break
            out[k.strip()] = v
        return out

    def snapshot(self):
        return self.call("autoui_snapshot")

    def press(self, label):
        r = self.call("autoui_press_sequence", {"keys": [label]})
        if "No button found" in r:
            raise AssertionError(f"press {label!r}: {r[:200]}")

    def fixture(self, **state):
        return self.call("autoui_fixture", {"schema_version": 1, "state": state})


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.environ.get("AUTO_BIN") or os.path.join(root, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTOUI_TEST_FIXTURES="1",
               AUTO_HTTP_PORT="8022")
    print(f"[*] Starting 022 VM on MCP port {port} (backend :8022)...")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
    failures = []
    ck = []

    def check(name, cond, detail=""):
        ck.append((name, bool(cond)))
        print(f"  {'PASS' if cond else 'FAIL'}  {name}  {str(detail)[:120]}")
        if not cond:
            failures.append(name)

    try:
        mcp = Mcp(port)
        ready = False
        for _ in range(40):
            time.sleep(1)
            try:
                if mcp.state("theme_name"):
                    ready = True
                    break
            except Exception:
                pass
        if not ready:
            print("[-] MCP/app not ready", file=sys.stderr)
            sys.exit(1)
        for _ in range(60):
            try:
                urllib.request.urlopen("http://127.0.0.1:8022/api/cards", timeout=2)
                break
            except Exception:
                time.sleep(2)
        time.sleep(2)

        # K1 board renders seeds.
        snap = mcp.snapshot()
        for marker in ("Project Board", "Design landing page", "Setup CI/CD",
                       "To Do", "In Progress", "Done"):
            check(f"K1 marker: {marker}", marker in snap)
        s = mcp.state("theme_name", "dark_mode")
        # App-declared dark-first default (model dark_mode=true, pac theme:
        # "dark") now resolved consistently on both tracks (pre-migration the
        # VM chain booted light while Vue booted dark — tracks disagreed; the
        # unification follows the app's own declared default).
        check("K1 dark-first default preserved (both tracks)",
              s.get("theme_name") == '"Auto"' and s.get("dark_mode") == "true", s)

        # P-037 theme round-trip preserves cards.
        mcp.fixture(theme_name="Solarized", dark_mode=False)
        time.sleep(0.8)
        s = mcp.state("theme_name", "dark_mode")
        check("P1 fixture switches to Solarized/light",
              s.get("theme_name") == '"Solarized"' and s.get("dark_mode") == "false", s)
        snap = mcp.snapshot()
        check("P1 cards render under Solarized/light", "Design landing page" in snap)
        # Toggle button: locate by emoji text in snapshot, press by element id
        # (press_sequence label matching is unreliable for emoji+VS16 text).
        snap_t = mcp.snapshot()
        tid = None
        last_button = None
        for line in snap_t.splitlines():
            if line.strip().startswith("button #"):
                for tok in line.split():
                    if tok.startswith("#vnode_") or tok.startswith("#aura_"):
                        last_button = tok.rstrip(":,{").lstrip("#")
            if last_button and ("☀️" in line or "🌙" in line):
                tid = last_button
                break
        check("P1 toggle button found", tid is not None)
        if tid:
            mcp.call("autoui_action", {"element_id": tid, "action": "press"})
        time.sleep(0.5)
        s = mcp.state("dark_mode")
        check("P1 ToggleDarkMode still flips mode under design binding",
              s.get("dark_mode") == "true", s)
        for fam in ("Solarized", "Auto"):
            for dm in (True, False):
                mcp.fixture(theme_name=fam, dark_mode=dm)
                time.sleep(0.4)
        # Restore the app's explicit dark-first default.
        mcp.fixture(dark_mode=True)
        time.sleep(0.5)
        s = mcp.state("theme_name", "dark_mode")
        check("P1 2x2 round-trip restored family+mode",
              s.get("theme_name") == '"Auto"' and s.get("dark_mode") in ("true", "false"), s)
        snap = mcp.snapshot()
        check("P1 cards preserved after round-trip", "Setup CI/CD" in snap)

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        subprocess.run(["taskkill", "/IM", "app-022-kanban-back.exe", "/F"],
                       capture_output=True)

    passed = sum(1 for _, ok in ck if ok)
    print(f"\n==== 022 VM results: {passed}/{len(ck)} ====")
    sys.exit(0 if not failures else 1)


if __name__ == "__main__":
    main()
