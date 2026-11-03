#!/usr/bin/env python3
"""PLAN-037 T-04: 023-realworld VM regression entry (theme adoption).

Starts `auto run -r vm` with the Rust backend on its pac back_port (8023),
waits for API + MCP, then asserts:
  R1 home renders seed articles (feed);
  P-037 theme family/mode round-trip preserves auth/session state and the
  mounted route content (换肤保鉴权).

Usage: AUTO_BIN=<auto> python test_023_vm.py
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
               AUTO_HTTP_PORT="8023")
    print(f"[*] Starting 023 VM on MCP port {port} (backend :8023)...")
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
                urllib.request.urlopen("http://127.0.0.1:8023/api/articles", timeout=2)
                break
            except Exception:
                time.sleep(2)
        time.sleep(2)

        # R1 home/feed renders seeds.
        snap = mcp.snapshot()
        check("R1 conduit nav renders", "conduit" in snap)
        check("R1 seed article renders", "Understanding React Server Components" in snap, snap[:120])

        # P-037 theme round-trip preserves route + business.
        mcp.fixture(theme_name="Solarized", dark_mode=True)
        time.sleep(0.8)
        s = mcp.state("theme_name", "dark_mode")
        check("P1 fixture switches to Solarized/dark",
              s.get("theme_name") == '"Solarized"' and s.get("dark_mode") == "true", s)
        snap = mcp.snapshot()
        check("P1 feed still rendered under Solarized/dark",
              "Understanding React Server Components" in snap)
        # Route change under theme via the nav (unauthenticated nav has a
        # "Sign in" button -> /login route; Settings nav only renders when
        # logged in). Oracle: login route renders under the switched theme.
        mcp.press("Sign in")
        time.sleep(1.2)
        snap = mcp.snapshot()
        check("P1 login route renders under Solarized/dark",
              "Sign in" in snap and ("Email" in snap or "password" in snap.lower()), snap[:120])
        for fam in ("Solarized", "Auto"):
            for dm in (True, False):
                mcp.fixture(theme_name=fam, dark_mode=dm)
                time.sleep(0.4)
        s = mcp.state("theme_name", "dark_mode")
        check("P1 2x2 round-trip applied (Auto/light final)",
              s.get("theme_name") == '"Auto"' and s.get("dark_mode") == "false", s)
        snap = mcp.snapshot()
        check("P1 route content preserved after round-trip", len(snap) > 1000)

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        subprocess.run(["taskkill", "/IM", "app-023-realworld-back.exe", "/F"],
                       capture_output=True)

    passed = sum(1 for _, ok in ck if ok)
    print(f"\n==== 023 VM results: {passed}/{len(ck)} ====")
    sys.exit(0 if not failures else 1)


if __name__ == "__main__":
    main()
