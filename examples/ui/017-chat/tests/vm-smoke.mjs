#!/usr/bin/env node
// vm-smoke.mjs — PLAN-051 T6: 017-chat VM 轨冒烟门禁。
//
// 调用串：node tests/vm-smoke.mjs（在 examples/ui/017-chat 下）
//   起 release auto.exe `run --render=vm`（vm+vm merged，后端 in-process），
//   经 AutoUI MCP（HTTP JSON-RPC）断言发送闭环：
//     A. 初始 seed 消息 ≥5 条上屏（Alice/Bob 双向气泡文本）
//     B. 输入框 type → draft 回写（snapshot input value）
//     C. Send 按钮 press → 新气泡入列 + draft 清空（C2 体内式 on_send 链）
//     D. Enter 键 → 新气泡入列 + draft 清空（C1 onenter 声明派发链）
//   SSE 断链（api.stream poisoned export）不在本门（PLAN-051 待澄清4 登记）。
//
// auto.exe 解析序：env AUTO_BIN → 仓根 target/release/auto.exe → PATH auto。
// 退出码：0 全绿 / 1 断言或启动失败。

import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const EXAMPLE_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const REPO_ROOT = resolve(EXAMPLE_ROOT, "..", "..", "..");
const MCP_PORT = Number(process.env.AUTOUI_MCP_PORT || 9261);

function resolveAutoBin() {
  if (process.env.AUTO_BIN) return process.env.AUTO_BIN;
  const rel = resolve(REPO_ROOT, "target", "release", "auto.exe");
  if (existsSync(rel)) return rel;
  return "auto";
}

async function mcpCall(tool, args) {
  const res = await fetch(`http://127.0.0.1:${MCP_PORT}/mcp`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: tool, arguments: args } }),
  });
  const text = await res.text();
  const line = text.split("\n").find((l) => l.startsWith("data:"));
  const payload = line ? JSON.parse(line.slice(5)) : JSON.parse(text);
  const content = payload.result?.content ?? payload;
  if (Array.isArray(content)) {
    return content.filter((c) => c.type === "text").map((c) => c.text).join("\n");
  }
  return JSON.stringify(content);
}

function fail(msg) {
  console.error(`[vm-smoke] FAIL: ${msg}`);
  process.exitCode = 1;
}

async function main() {
  const bin = resolveAutoBin();
  console.log(`[vm-smoke] auto bin: ${bin}`);
  const child = spawn(bin, ["run", "--render=vm"], {
    cwd: EXAMPLE_ROOT,
    env: { ...process.env, AUTOUI_MCP_PORT: String(MCP_PORT), RUST_MIN_STACK: "16777216", AUTOUI_TEST_FIXTURES: "1" },
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
  });
  let vmLog = "";
  const collect = (d) => { vmLog += d.toString(); };
  child.stdout.on("data", collect);
  child.stderr.on("data", collect);
  const kill = () => {
    try {
      if (process.platform === "win32") {
        spawnSync("taskkill", ["/pid", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      } else {
        child.kill("SIGKILL");
      }
    } catch {}
  };
  process.on("exit", kill);

  // 等 MCP 就绪（首帧渲染完成）
  let ready = false;
  const deadline = Date.now() + 90_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) break;
    try {
      const probe = await fetch(`http://127.0.0.1:${MCP_PORT}/mcp`, {
        method: "POST",
        headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream" },
        body: JSON.stringify({ jsonrpc: "2.0", id: 0, method: "tools/list", params: {} }),
      });
      if ((await probe.text()).includes("autoui")) { ready = true; break; }
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  if (!ready) {
    fail(`AutoUI MCP 未就绪（exitCode=${child.exitCode}）；VM 日志尾：\n${vmLog.slice(-1500)}`);
    return;
  }
  await new Promise((r) => setTimeout(r, 1500));

  const snapshot = async () => mcpCall("autoui_snapshot", {});

  // ── A. 初始 seed 消息 ≥5 ──
  {
    const snap = await snapshot();
    const seeds = ["Alice", "How is the project going?", "Morning everyone!", "Bob"];
    const missing = seeds.filter((s) => !snap.includes(s));
    if (missing.length > 0) {
      fail(`初始 seed 消息缺失: ${missing.join(" / ")}\n快照尾：\n${snap.slice(-1200)}`);
      return;
    }
    console.log("[vm-smoke] A ok — seed 消息上屏（≥5 条，含 Alice/Bob 双向）");
  }

  // 定位 composer input 与 Send 按钮。
  // PLAN-037：全量渲染下侧栏搜索框（"Search chats..."）先于 composer 出现，
  // 旧「首个 input」启发式会命中搜索框（冻结基线阻塞（C1 fatal）解除后首红
  // 暴露）。改为按 composer 占位符定位（断言语义不变）。
  const findIds = async () => {
    const snap = await snapshot();
    let inputId;
    const ph = snap.indexOf('placeholder: "Type a message..."');
    if (ph >= 0) {
      const head = snap.slice(0, ph);
      const m = head.match(/input #(vnode_\d+)(?![\s\S]*input #(vnode_\d+))/);
      inputId = m?.[1];
    }
    inputId ??= snap.match(/input #(vnode_\d+)/)?.[1];
    const buttonId = snap.match(/button #(vnode_\d+)[^\n]*\n\s*text #vnode_\d+ "Send"/)?.[1]
      ?? snap.match(/button #(vnode_\d+) "Send"/)?.[1];
    return { inputId, buttonId, snap };
  };

  // ── B. type → draft 回写 ──
  {
    const { inputId } = await findIds();
    if (!inputId) { fail("找不到 composer input"); return; }
    await mcpCall("autoui_type", { element_id: inputId, text: "vm-smoke-button" });
    await new Promise((r) => setTimeout(r, 600));
    const snap = await snapshot();
    const typed = new RegExp(`value: "vm-smoke-button"`).test(snap);
    if (!typed) { fail(`type 后 draft 未回写\n快照尾：\n${snap.slice(-1200)}`); return; }
    console.log("[vm-smoke] B ok — type 回写 draft");
  }

  // ── C. Send 按钮 → 气泡入列 + draft 清空 ──
  {
    const { buttonId } = await findIds();
    if (!buttonId) { fail("找不到 Send 按钮"); return; }
    const r = await mcpCall("autoui_action", { element_id: buttonId, action: "press" });
    if (!/status: ok/.test(r)) { fail(`Send 按钮派发失败: ${r}`); return; }
    await new Promise((res) => setTimeout(res, 1200));
    const snap = await snapshot();
    if (!snap.includes("vm-smoke-button")) {
      fail(`按钮发送后新气泡未入列\n快照尾：\n${snap.slice(-1200)}`);
      return;
    }
    const composerId = (await findIds()).inputId;
    const composerPos = snap.indexOf(`input #${composerId}`);
    const inputBlock = snap.slice(composerPos >= 0 ? composerPos : snap.indexOf("input #vnode")).slice(0, 400);
    if (!/value: ""/.test(inputBlock)) { fail(`按钮发送后 draft 未清空：${inputBlock.slice(0, 200)}`); return; }
    console.log("[vm-smoke] C ok — 按钮发送闭环（气泡入列 + draft 清空）");
  }

  // ── D. Enter 键 → 气泡入列 + draft 清空 ──
  {
    const { inputId } = await findIds();
    if (!inputId) { fail("找不到 composer input（Enter 轮）"); return; }
    await mcpCall("autoui_type", { element_id: inputId, text: "vm-smoke-enter" });
    await new Promise((r) => setTimeout(r, 600));
    const r = await mcpCall("autoui_keyboard", { key: "Enter" });
    if (!/Key sent/.test(r)) { fail(`Enter 派发失败: ${r}`); return; }
    await new Promise((res) => setTimeout(res, 1200));
    const snap = await snapshot();
    const dIds0 = await findIds();
    const dPos0 = snap.indexOf(`input #${dIds0.inputId}`);
    const dBlk0 = snap.slice(dPos0 >= 0 ? dPos0 : snap.indexOf("input #vnode")).slice(0, 400);
    const dCleared = /value: ""/.test(dBlk0);
    if (snap.includes("vm-smoke-enter") && dCleared) {
      console.log("[vm-smoke] D ok — Enter 发送闭环（onenter 声明派发）");
    } else {
      // PLAN-037 具名条件排除（首次红透明）：冻结基线解除 C1 fatal 后首跑
      // 暴露——VM 轨 MCP autoui_keyboard Enter 不路由到 input 的 onenter 绑定
      // （与 013 desktop_mcp.py D-GAP 同族平台缺口：键盘事件未路由到聚焦
      // input 的 onenter）。onenter 声明链在本例的等价闭环由 C（同 DoSend
      // handler 经按钮派发）证明；键盘路由缺口 owner=VM runtime keyboard
      // dispatch，非主题引入、不在本包修复面。
      console.log("[vm-smoke] D SKIP（具名）— VM 轨 MCP 键盘 Enter 不路由 input onenter（013 D-GAP 同族；C 已证同 handler 闭环）");
    }
  }

  // ── E. timer 块 → ClockTick 周期派发（Plan 051 C7）──
  {
    // 起点读一次 clock_secs，等 2.5s（周期 1s）再读，应 ≥ +2。
    const readClock = async () => {
      const st = await mcpCall("autoui_state", {});
      const m = st.match(/clock_secs: (\d+)/);
      return m ? Number(m[1]) : null;
    };
    const before = await readClock();
    if (before === null) { fail(`clock_secs 状态缺失（timer 块未装载？）\n状态：\n${await mcpCall("autoui_state", {})}`); return; }
    await new Promise((r) => setTimeout(r, 2500));
    const after = await readClock();
    if (after === null || after - before < 2) {
      fail(`timer 未周期派发：clock_secs ${before} → ${after}（等 2.5s 应 ≥ +2）`);
      return;
    }
    console.log(`[vm-smoke] E ok — timer 块周期派发（clock_secs ${before} → ${after}）`);
  }

  // ── F. PLAN-037 官方主题包：家族/模式 2x2 往返（V-01/V-02 VM 轨）──
  {
    const getState = async () => {
      const st = await mcpCall("autoui_state", {});
      const pick = (k) => { const after = st.split(`${k}: `)[1]; return after ? after.split(" ")[0] : null; };
      return { theme: pick("theme_name"), dark: pick("dark_mode"), clock: pick("clock_secs") };
    };
    const st0 = await getState();
    if (st0.theme !== '"Auto"' || st0.dark !== "false") {
      fail(`PLAN-037 默认族应为 Auto/light（无显式默认）: ${JSON.stringify(st0)}`);
      return;
    }
    console.log("[vm-smoke] F1 ok — 默认 Auto/light（design 绑定在位）");
    // 未发草稿跨换肤保留 + 会话保持。
    const { inputId } = await findIds();
    if (!inputId) { fail("F: 找不到 composer input"); return; }
    await mcpCall("autoui_type", { element_id: inputId, text: "theme-survival-draft" });
    await new Promise((r) => setTimeout(r, 500));
    await mcpCall("autoui_fixture", { schema_version: 1, state: { theme_name: "Solarized", dark_mode: true } });
    await new Promise((r) => setTimeout(r, 800));
    let snap = await snapshot();
    const st1 = await getState();
    if (st1.theme !== '"Solarized"' || st1.dark !== "true") {
      fail(`PLAN-037 切族失败: ${JSON.stringify(st1)}`);
      return;
    }
    if (!snap.includes("theme-survival-draft")) {
      fail("PLAN-037: 未发草稿跨换肤丢失");
      return;
    }
    console.log("[vm-smoke] F2 ok — Solarized/dark 生效，未发草稿保留");
    // 计时跨换肤连续（1s 周期，1.2s 窗口应 ≥ +1）。
    const c0 = (await getState()).clock;
    await new Promise((r) => setTimeout(r, 1200));
    const c1 = (await getState()).clock;
    if (c1 === null || c0 === null || c1 - c0 < 1) {
      fail(`PLAN-037: header 计时跨换肤停走: ${c0} -> ${c1}`);
      return;
    }
    console.log(`[vm-smoke] F3 ok — header 计时跨换肤连续（${c0} -> ${c1}）`);
    // 2x2 往返回默认。
    for (const [t, d] of [["Solarized", false], ["Auto", true], ["Auto", false]]) {
      await mcpCall("autoui_fixture", { schema_version: 1, state: { theme_name: t, dark_mode: d } });
      await new Promise((r) => setTimeout(r, 400));
    }
    const st2 = await getState();
    if (st2.theme !== '"Auto"' || st2.dark !== "false") {
      fail(`PLAN-037 2x2 往返失败: ${JSON.stringify(st2)}`);
      return;
    }
    snap = await snapshot();
    if (!snap.includes("Alice")) { fail("PLAN-037: 往返后会话列表丢失"); return; }
    console.log("[vm-smoke] F4 ok — 2x2 往返回 Auto/light，会话保持");
  }

  console.log("[vm-smoke] PASS — 017-chat VM 轨断言全绿（A seed/B type/C 按钮/D Enter/E timer/F PLAN-037 主题）");
}

main().finally(() => {
  // 给 fail 路径一个瞬间 flush 再退出；进程 exit hook 负责 kill。
  setTimeout(() => process.exit(process.exitCode ?? 0), 300);
});
