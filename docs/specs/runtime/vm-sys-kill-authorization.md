# VM 宿主结束进程授权门（sys kill authorization，VM 面）

> PLAN-112 T-02 第 5 项的规范落点。只写已验行为与接口。025-sys-monitor 原样
> 源与 A2Vue lane 的 UI 行为不在本文件（后者的 HTTP 会话面见
> [back-proxy-sys-kill-policy.md](back-proxy-sys-kill-policy.md)）。

## 1. 合同

`crates/auto-ui-runtime/src/vm_sys_kill.rs` + core
`vm::host_capabilities::SysKillGate`。VM 合并模式（`auto run -r vm`）里
`back.api` 的 `kill_process` 在进程内直达 `sys.kill`，不经 HTTP 代理，因此
授权门落在宿主：

- **入口**：`run_entry_file`（auto-ui-tools `run_vm.rs`）在解释器（=VM 构造期
  宿主能力快照）之前调用 `install_vm_sys_kill_gate_from_env()`。非法策略具名
  中止，不静默退回无门。
- **宿主策略**（env `AUTO_UI_VM_SYS_KILL_POLICY`）：

| 值 | 含义 | 结果 |
|---|---|---|
| 未设置 / 空 / `allow-all` | 无门（并清除同进程上一轮登记） | 迁移前行为逐字不变 |
| `deny` / `deny:<reason>` | 该 VM 不持结束进程授权 | 一律具名拒 |
| `allowed-pids:<pid,…>` | 仅清单内 pid | 清单内放行；其余（含空清单/非法 pid）具名失败 |

- **拒绝语义**：在刷新采样与 OS 结束**之前**，core shim 抛 `VMError`，文本 =
  稳定码 `sys.kill.permission_denied` + 宿主 reason + pid。消费面 = VM 动态
  UI 的 `[VM-HANDLER] <widget>.<event> failed: …`（handler 中止、窗口与后续
  刷新存活；确认框由用户「取消」复位）。
- **与 HTTP 面关系**：同三态、同稳定码；`back_proxy::SysKillPolicy` 管会话
  派发前的 403，本门管 VM 内直调。两者互不替代。
- **边界**：非 OS 沙箱——`allow-all`/未登记时仍直接 `sys.kill`；只认
  `decision(pid)`，不做 pid 归属判定；不改写原项目源或生成物。

## 2. 判别

| 策略 | 目标 | 观测 |
|---|---|---|
| 未设置 | 隔离子进程 | 真实结束（旧行为） |
| `deny` | 隔离子进程 | `VMError` 含稳定码 + pid；目标存活；后续刷新继续 |
| `allowed-pids:{A}` | B（未授权） | 具名拒；B 存活 |
| `allowed-pids:{A}` | A（授权） | 真实结束 |
| 非法值 | — | 启动具名失败（列出期望形状） |

## 3. 永久用例

- `crates/auto-ui-runtime/tests/plan112_vm_sys_kill_policy.rs`（4 例）：三态解析
  与非法值具名、登记/清除、deny 目标存活、allowlist 内外分流。
- `crates/auto-lang/tests/plan112_vm_sys_kill_gate.rs`（5 例）：core 侧缺席/
  `AllowAll`/`Deny`/`AllowedPids` 与构造期快照。
- 真实消费证据 = PLAN-112 T-03 的 `auto run -r vm` + MCP 实跑（见 Plan §9）。
