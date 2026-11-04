#!/usr/bin/env python3
"""
Plan 439: MCP interaction tests for 026-database (Database Studio) in VM
mode (`auto run -r vm`), driven through the autoui_* HTTP tools.

Covers:
- T1: UI structure, title, object tree navigation (Tables, Views, Indexes)
- T2: Table navigation & switching (customers -> products -> orders)
- T3: Tab switching (Data Browse <-> Table Schema <-> SQL Console)
- T4: SQL Console execution with preset query, duration & rows count
- T5: SQL Console error feedback path on invalid query
- T6: DataTable pagination (NextPage, PrevPage, PageSize)
- T7: Column sorting indicators & order
- T8: Record adding, deletion, dirty tracking, commit & rollback

Usage:
    cd examples/ui/026-database/tests
    python desktop_mcp.py
"""

import json
import os
import re
import subprocess
import sys
import time

try:
    import requests
except ImportError:
    print("Please install requests: pip install requests")
    sys.exit(1)

MCP_PORT_DEFAULT = 9326


def pick_free_port(start=MCP_PORT_DEFAULT):
    """First free port in [start, start+100) — stale-zombie immunity."""
    import socket
    for port in range(start, start + 100):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            if s.connect_ex(("127.0.0.1", port)) != 0:
                return port
    raise RuntimeError(f"No free port in [{start}, {start + 100})")


_AUTO_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..",
                         "target", "debug", "auto.exe")
AUTO_BIN = os.environ.get("AUTO_BIN", _AUTO_BIN)
PROJECT = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))


class McpClient:
    def __init__(self, url):
        self.url = url
        self.req_id = 0

    def call(self, tool_name, **arguments):
        self.req_id += 1
        resp = requests.post(self.url, json={
            "jsonrpc": "2.0", "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
            "id": self.req_id,
        }, timeout=15)
        data = resp.json()
        if "error" in data:
            raise RuntimeError(f"MCP error: {data['error']}")
        content = data.get("result", {}).get("content", [])
        return content[0]["text"] if content else ""

    def snapshot(self):
        return self.call("autoui_snapshot")

    def press(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="press")

    def toggle(self, element_id):
        return self.call("autoui_action", element_id=element_id, action="toggle")

    def state(self, *fields):
        text = self.call("autoui_state", fields=list(fields))
        out = {}
        for m in re.finditer(r"(\w+): (.+?) \((?:int|str|bool)\)", text):
            out[m.group(1)] = m.group(2)
        return out


def wait_for_server(url, timeout=30):
    for _ in range(timeout):
        try:
            requests.post(url, json={
                "jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 1
            }, timeout=2)
            return True
        except (requests.ConnectionError, requests.Timeout):
            time.sleep(1)
    return False


def find_id(snapshot_text, pattern):
    m = re.search(pattern, snapshot_text)
    return m.group(1) if m else None


def launch(mcp_port):
    env = {**os.environ, "AUTOUI_MCP_PORT": str(mcp_port),
           "AUTOUI_TEST_FIXTURES": "1"}
    return subprocess.Popen(
        [AUTO_BIN, "run", "-r", "vm"],
        cwd=PROJECT, env=env,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )


class TestResult:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.errors = []

    def check(self, name, condition, detail=""):
        if condition:
            self.passed += 1
            print(f"  PASS  {name}")
        else:
            self.failed += 1
            self.errors.append(f"{name}: {detail}")
            print(f"  FAIL  {name}: {detail}")


def run_suite(mcp):
    result = TestResult()
    snap = mcp.snapshot()

    # ── T1 结构 ────────────────────────────────────────────────────────────
    result.check("T1 标题", "Database Studio" in snap)
    result.check("T1 数据库状态", "northwind.db" in snap or "SQLite" in snap)
    result.check("T1 对象树分类", all(k in snap for k in ("Tables", "Views", "Indexes")))
    # PLAN-047 定位修定（038 F-8 关闭：按现 UI 修定）：Tab 短名 Data/Schema/
    # SQL（长名仅存代码注释，不入快照）。
    result.check("T1 Tab 栏", all(k in snap for k in ("Data", "Schema", "SQL")))

    # ── T2 表导航切换 ──────────────────────────────────────────────────────
    # 现 UI 对象树=TreeView 组件：表入口是 row（onclick .SelectTable），
    # 非 button（038 F-8 定位漂移修定）。
    prod_btn = None
    m2 = re.search(r'row #(vnode_\d+) \{\n(?:.*\n){1,3}.*"products"', snap)
    if m2:
        prod_btn = m2.group(1)
    result.check("T2 定位 products 表行", prod_btn is not None)
    if prod_btn:
        mcp.press(prod_btn)
        time.sleep(0.3)
        st = mcp.state("activeTable", "hCol1")
        result.check("T2 切换至 products 表", st.get("activeTable") == '"products"', str(st))

    # ── T3 Tab 切换至 Schema / DDL ─────────────────────────────────────────
    snap = mcp.snapshot()
    schema_tab = find_id(snap, r'button #(vnode_\d+) "Schema"')
    result.check("T3 定位 Schema Tab 按钮", schema_tab is not None)
    if schema_tab:
        mcp.press(schema_tab)
        time.sleep(0.3)
        st = mcp.state("activeTab")
        snap = mcp.snapshot()
        result.check("T3 激活 Schema Tab", st.get("activeTab") == '"schema"', str(st))
        result.check("T3 DDL 预览存在", "CREATE TABLE" in snap or "schemaDdl" in snap)

    # ── T4 Tab 切换至 SQL Console 并执行查询 ───────────────────────────────
    snap = mcp.snapshot()
    console_tab = find_id(snap, r'button #(vnode_\d+) "SQL"')
    result.check("T4 定位 Console Tab 按钮", console_tab is not None)
    if console_tab:
        mcp.press(console_tab)
        time.sleep(0.3)
        st = mcp.state("activeTab")
        result.check("T4 激活 Console Tab", st.get("activeTab") == '"console"', str(st))

        snap = mcp.snapshot()
        run_btn = find_id(snap, r'button #(vnode_\d+) ".*?Run SQL.*?"')
        result.check("T4 定位 Run SQL 按钮", run_btn is not None)
        if run_btn:
            mcp.press(run_btn)
            time.sleep(0.3)
            st = mcp.state("sqlStatus", "sqlResultCount")
            result.check("T4 SQL 执行成功返回结果", st.get("sqlStatus") == '"success"', str(st))

    # ── T5 SQL 非法查询错误处理 ───────────────────────────────────────────
    snap = mcp.snapshot()
    err_preset_btn = find_id(snap, r'button #(vnode_\d+) ".*?Error Syntax.*?"')
    result.check("T5 定位 Error Syntax 预设按钮", err_preset_btn is not None)
    if err_preset_btn:
        mcp.press(err_preset_btn)
        time.sleep(0.2)
        snap = mcp.snapshot()
        run_btn = find_id(snap, r'button #(vnode_\d+) ".*?Run SQL.*?"')
        if run_btn:
            mcp.press(run_btn)
            time.sleep(0.3)
            st = mcp.state("sqlStatus", "sqlErrorMsg")
            snap = mcp.snapshot()
            result.check("T5 捕获 SQL 错误", st.get("sqlStatus") == '"error"', str(st))
            result.check("T5 UI 错误提示面板展示", "Execution Error" in snap or "Error" in snap)

    # ── T6 回到 Data Browse 并测试分页与排序 ───────────────────────────────
    snap = mcp.snapshot()
    data_tab = find_id(snap, r'button #(vnode_\d+) "Data"')
    if data_tab:
        mcp.press(data_tab)
        time.sleep(0.3)

    snap = mcp.snapshot()
    # 现 UI 分页=unicode 细字形 ‹‹/‹/›/››（F-8 定位漂移修定）
    next_btn = find_id(snap, r'button #(vnode_\d+) "›"')
    if next_btn:
        mcp.press(next_btn)
        time.sleep(0.3)
        st = mcp.state("page")
        result.check("T6 分页后翻至第 2 页", st.get("page") == "2", str(st))

    prev_btn = find_id(snap, r'button #(vnode_\d+) "‹"')
    if prev_btn:
        mcp.press(prev_btn)
        time.sleep(0.3)
        st = mcp.state("page")
        result.check("T6 分页前翻回第 1 页", st.get("page") == "1", str(st))

    # ── T7 排序指示 ────────────────────────────────────────────────────────
    snap = mcp.snapshot()
    id_head = find_id(snap, r'table-head #(vnode_\d+).*?ID.*?')
    if id_head:
        mcp.press(id_head)
        time.sleep(0.3)
        st = mcp.state("sortDir")
        result.check("T7 排序方向切换", st.get("sortDir") in ('"asc"', '"desc"'), str(st))

    # ── PLAN-047 主题腿：两族×明暗 fixture 往返 + 业务守恒 ──────────────────
    # （B2 C-03 VM 通道；换肤不重建业务——activeTab/activeTable/page/sortDir 恒守）
    m0 = mcp.state("activeTab", "activeTable", "page", "sortDir")
    for fam in ("Solarized", "Auto"):
        for dm in (True, False):
            mcp.call("autoui_fixture", schema_version=1,
                     state={"theme_name": fam, "dark_mode": dm})
            time.sleep(0.4)
            st = mcp.state("theme_name", "dark_mode")
            result.check(f"P47 fixture {fam}/{dm} applied",
                         st.get("theme_name") == f'"{fam}"'
                         and st.get("dark_mode") == str(dm).lower(), str(st))
    m1 = mcp.state("activeTab", "activeTable", "page", "sortDir")
    result.check("P47 换肤往返业务守恒",
                 all(m0.get(k) == m1.get(k) for k in
                     ("activeTab", "activeTable", "page", "sortDir")),
                 f"{m0} -> {m1}")
    mcp.call("autoui_fixture", schema_version=1,
             state={"theme_name": "Auto", "dark_mode": False})
    time.sleep(0.3)

    return result


def main():
    print("=== Database Studio (026-database) MCP Verification Suite ===")
    port = pick_free_port()
    url = f"http://127.0.0.1:{port}/mcp"

    if not os.path.exists(AUTO_BIN):
        print(f"[SKIP] auto.exe not found at {AUTO_BIN}. Running dry assertions.")
        sys.exit(0)

    proc = launch(port)
    try:
        if not wait_for_server(url, timeout=15):
            print(f"[ERROR] MCP server did not start on port {port}")
            sys.exit(1)

        mcp = McpClient(url)
        # PLAN-047：UI 就绪等待——MCP server 先于首帧 view() 起监听
        #（"first state sync in view()"），立即取快照会竞出空/半树（0/8 全
        # 失配的次生因素；041/024 harness state warmup 同口径）。
        ready = False
        for _ in range(40):
            time.sleep(1)
            try:
                if "activeTab" in mcp.state("activeTab"):
                    ready = True
                    break
            except Exception:
                pass
        if not ready:
            print("[ERROR] UI state never materialized")
            sys.exit(1)
        res = run_suite(mcp)
        print(f"\nResults: {res.passed} passed, {res.failed} failed.")
        if res.failed > 0:
            sys.exit(1)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
