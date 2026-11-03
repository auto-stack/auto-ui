#!/usr/bin/env python3
"""PLAN-037 T-02: 016-calendar VM regression entry (first real gate — the
example previously had no tests/ at all).

Starts `auto run -r vm`, drives the real calendar via AutoUI MCP:
  C1 boot: month grid renders (weekday headers + day cells + Selected line);
  C2 select day: SelectDay updates store.selected_date;
  C3 month nav: NextMonth/PrevMonth move month, selection preserved;
  C4 Today: selected_date becomes today;
  C5 PLAN-037 theme: family/mode round-trip (fixture channel) preserves
     year/month/selected_date (换肤不改变日期/选中/事件);
  C6 settings popup Dark/Light buttons still drive dark_mode under the
     design binding.

Usage: AUTO_BIN=<auto> python test_016_vm.py
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

    def find_button_id(self, label):
        """Find the element id of a button whose rendered text equals label."""
        snap = self.snapshot()
        current_id = None
        for line in snap.splitlines():
            for token in line.split():
                if token.startswith("#aura_") or token.startswith("#vnode_"):
                    current_id = token.rstrip(":,{").lstrip("#")
            if current_id and label in line and "button" in line.lower():
                return current_id
        return None


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../"))
    auto_bin = os.environ.get("AUTO_BIN") or os.path.join(root, "target/debug/auto.exe")
    app_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../"))

    port = pick_free_port()
    env = dict(os.environ, AUTOUI_MCP_PORT=str(port), AUTOUI_TEST_FIXTURES="1")
    print(f"[*] Starting 016 VM on MCP port {port}...")
    proc = subprocess.Popen([auto_bin, "run", "-r", "vm"], cwd=app_dir, env=env,
                            stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)

    failures = []
    ck = []

    def check(name, cond, detail=""):
        ck.append((name, bool(cond)))
        print(f"  {'PASS' if cond else 'FAIL'}  {name}  {str(detail)[:140]}")
        if not cond:
            failures.append(name)

    try:
        mcp = Mcp(port)
        ready = False
        for _ in range(40):
            time.sleep(1)
            try:
                if mcp.state("year"):
                    ready = True
                    break
            except Exception:
                pass
        if not ready:
            print("[-] MCP/app not ready", file=sys.stderr)
            sys.exit(1)

        # C1 boot: grid + selected line render.
        snap = mcp.snapshot()
        check("C1 weekday headers render", all(d in snap for d in ("Su", "Mo", "Tu")), snap[:120])
        check("C1 Selected line renders", "Selected:" in snap)

        s = mcp.state("year", "month", "selected_date", "today")
        year, month = s.get("year"), s.get("month")
        sel0 = s.get("selected_date")
        check("C1 store state readable", bool(year and month and sel0 and s.get("today")), s)

        # C2 select day 15.
        mcp.press("15")
        time.sleep(0.4)
        s = mcp.state("selected_date")
        sel15 = s.get("selected_date", "")
        check("C2 SelectDay(15) updates selection", "-15" in sel15, sel15)

        # C3 month nav preserves selection.
        mcp.press(">")
        time.sleep(0.4)
        s = mcp.state("month", "selected_date")
        check("C3 NextMonth advances month", s.get("month") != month, f"{month} -> {s.get('month')}")
        check("C3 selection preserved across month nav", s.get("selected_date") == sel15,
              f"{sel15} -> {s.get('selected_date')}")
        mcp.press("<")
        time.sleep(0.4)
        s = mcp.state("month")
        check("C3 PrevMonth returns", s.get("month") == month, s.get("month"))

        # C5 PLAN-037: theme round-trip preserves calendar identity.
        mcp.fixture(theme_name="Solarized", dark_mode=True)
        time.sleep(0.6)
        s = mcp.state("theme_name", "dark_mode", "year", "month", "selected_date")
        check("C5 fixture switches to Solarized/dark",
              s.get("theme_name") == '"Solarized"' and s.get("dark_mode") == "true", s)
        check("C5 year/month/selection preserved across theme switch",
              s.get("year") == year and s.get("month") == month
              and s.get("selected_date") == sel15, s.get("selected_date"))
        snap = mcp.snapshot()
        check("C5 grid still renders under Solarized/dark", "Selected:" in snap)

        for fam in ("Solarized", "Auto"):
            for dm in (False, True):
                mcp.fixture(theme_name=fam, dark_mode=dm)
                time.sleep(0.4)
        s = mcp.state("theme_name", "dark_mode", "selected_date")
        check("C5 2x2 round-trip applied (Auto/dark final)",
              s.get("theme_name") == '"Auto"' and s.get("dark_mode") == "true", s)
        check("C5 selection preserved after round-trip", s.get("selected_date") == sel15,
              s.get("selected_date"))

        # C6 settings popup Dark/Light still drive the mode.
        mcp.press("⚙ Settings")
        time.sleep(0.4)
        dark_btn = mcp.find_button_id("Dark")
        if dark_btn:
            mcp.call("autoui_action", {"element_id": dark_btn, "action": "press"})
            time.sleep(0.5)
            s = mcp.state("dark_mode")
            check("C6 settings Dark button sets dark_mode", s.get("dark_mode") == "true", s)
        else:
            check("C6 settings Dark button found", False)
        light_btn = mcp.find_button_id("Light")
        if light_btn:
            mcp.call("autoui_action", {"element_id": light_btn, "action": "press"})
            time.sleep(0.5)
            s = mcp.state("dark_mode")
            check("C6 settings Light button clears dark_mode", s.get("dark_mode") == "false", s)
        else:
            check("C6 settings Light button found", False)

        # C4 Today: selected_date becomes today.
        mcp.press("Today")
        time.sleep(0.4)
        s = mcp.state("selected_date", "today")
        today = (s.get("today") or "").strip('"')
        check("C4 Today selects today", s.get("selected_date") == f'"{today}"',
              f"selected={s.get('selected_date')} today={today}")

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()

    passed = sum(1 for _, ok in ck if ok)
    print(f"\n==== 016 VM results: {passed}/{len(ck)} ====")
    sys.exit(0 if not failures else 1)


if __name__ == "__main__":
    main()
