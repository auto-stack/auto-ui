# ASE M0 同宿主 frame 公共消费（UI owner 侧）已验现状

> 来源：PLAN-1022（2026-11-19 交付，复审/交付身份见 auto-lang 侧归档计划 §9）。
> 本文件只记录**已验证**的消费约定与边界；语言/ASE 侧现状见 auto-lang
> `docs/specs/ase/m0-text-browser.md`。

## 消费约定（已验）

`crates/auto-ui-runtime/src/ui/ase_m0_frame.rs` 是 ASE 宿主的**薄消费适配**：
纯 980 MIN-A-UI 公共面消费（`NodeDesktopBridge::load/open/close_session/step` +
全局 HOST 的 `current_session/host_session_stats/host_session_display`），不复制
FFI/解析/窗口，不改 980 模块，**不安装 980 全局 BRIDGE 槽**（自有 `ASE_HOST`
槽；桥由适配器独占持有并只在窗口 tick 主线程步进）。

- 页面注册：`AsePage{key, path}`——源经**显式路径**（权威 fixtures 在 auto-lang
  `experimental/ase-m0/fixtures/text-{a,b}/`；`ASE_M0_TEXT_ROOT` 可覆盖根）。
- 导航代次：受理指令（open/reopen/close）推进；`begin_open`→`AseLoadTicket`→
  `mount` 票据协议——代次不符整票拒绝（不 open、零 session、零 UI 效果）。
  未知 key 具名拒（零装载、代次不动）。轮换/重开先 `close_session` 释放旧 owner。
- 窗口：`AseM0TextWindow`（地址行/错误行/导航按钮只发导航/frame 区渲染全局
  HOST 的当前 session）；单真实宿主窗口，每页新窗/第二事件环不算嵌入。
- 原生入口：`examples/ase_m0_text_window.rs`（`--page k=<path> --evidence
  <json> [--script "t:open:k|close|reopen"] [--hold-ms] [--min-frames]`，
  `--prep-only` 预备档；窗口标题 "ASE M0 文字浏览器"）。

## 已验范围与 oracle

`tests/ase_m0_text.rs`（完整目标，`--test-threads=1`；排窗档
`AUTO_UI_ASE_M0_E2E=1` 单实例）：同宿主轮换 A→空→B（AV 真值文本字节级一致，
中文/ASCII、函数叶）、反硬编码（换根/函数名/改文字）、改A不改B、非法→修复、
未知零执行、代次迟到拒绝、20 轮 owner 释放、真窗单窗跨轮换重渲染。新增 case
全部 ≤10s（2026-11-14 裁定；预备档=构建示例+`--prep-only`，日常档零 rustc）。

## 边界

页面为静态面（无 model/msg）：页内事件/输入不在本域（1023 承接）；多窗口、
后台服务、保留状态、业务按钮（导航之外）不承诺。resize/焦点已验为 OS 级注入
档，真人走查档 pending。HOST 已关闭 session 记录随轮次累积属 980 既有 late 靶
设计，裁剪义务归 980 owner。
