# Back-proxy 结束进程授权门（sys kill policy）

> PLAN-112 T-02 的规范落点（Plan frontmatter `new_spec_components`）。只写已验
> 行为与接口。025-sys-monitor 原样源、A2Vue lane 的 UI 行为不在本文件。

## 1. 合同

`crates/auto-ui-runtime/src/back_proxy.rs`。结束进程是破坏性操作，宿主可在
**会话派发之前**对它设授权门：

- **路由**：`POST /apps/<id>/api/system/kill`（`SYS_KILL_ROUTE`）。其他方法、
  其他路由不经此门。
- **登记**：`RunningProxy::set_sys_kill_policy(app_id, SysKillPolicy)`，按 app。
  `BackProxyConfig` 字段面不变（既有穷举字面量构造点零改动）。
- **策略**（`SysKillPolicy`）：

| 策略 | 含义 | 结果 |
|---|---|---|
| 未登记 / `AllowAll` | 无门 | 请求原样到达 app 后端（既有 `auto run` 行为逐字不变） |
| `Deny { reason }` | 该 app 不持结束进程授权 | 一律具名拒 |
| `AllowedPids(set)` | 仅授权清单内的 pid | 清单内放行；其余（含缺失/非法 pid）具名拒 |

- **拒绝应答**：HTTP 403，JSON `{ ok: false, code, error, pid }`。
  `code` 恒为 `sys.kill.permission_denied`（`SYS_KILL_PERMISSION_DENIED`）；
  `error` 载具名原因；`pid` 取请求体 `pid`（缺失/非法为 `null`）。
- **不触碰**：拒绝发生在会话派发之前——VM 后端不被调用、OS 进程不被触碰、
  lazy 档不因被拒请求触发会话装载。
- **生命周期**：`remove_app` 随 app 供给面一并卸下策略；重新登记的会话不继承。

## 2. 判别

| 策略 | 目标 | 观测 |
|---|---|---|
| 未登记 | 隔离子进程 | 200 `true`，进程退出 |
| `Deny` | 隔离子进程 | 403 + 稳定码，进程存活；快照路由照常 200 |
| `AllowedPids{A}` | B（未授权） | 403 + 稳定码 + 具名 B，B 存活 |
| `AllowedPids{A}` | A（授权） | 200 `true`，A 退出 |
| `Deny` | 另一 POST 路由 | 到达后端，由后端应答（不带策略诊断） |

## 3. 永久用例

`crates/auto-ui-runtime/tests/plan112_sys_kill_policy.rs`（5 例，实测各
0.76–1.83s）。后端 = 原样 `examples/ui/025-sys-monitor/src/back/api.at` 经
`back_proxy` 会话运行（AutoVM + sysinfo 真实采集）。结束进程的目标一律是
测试自建的隔离子进程；被拒路径另以「目标仍存活」断言派发未被触碰。

真实消费证据在 A2Vue 腿的浏览器档
（`crates/auto-ui-runtime/tests/plan112_a2vue_sites.rs`，`AUTO_UI_ACE_WEB_E2E=1`）：
真 Chrome + 同一 `back_proxy`，`AllowedPids` 下未授权 pid 得 403 + 稳定码 +
目标 pid 且 UI 可恢复，授权 pid 经后端真实结束。

## 4. 边界（显名）

- 这不是 OS 级沙箱：未登记或 `AllowAll` 下，后端仍直接调用 `sys.kill`。
- 门只认声明路由与请求体的 `pid`；不解析 query、不做 pid 归属判定。
- A2Vue 客户端对非 2xx 抛错是既有生成行为：store handler 在 `await` 处中止
  （拒绝经 Vue 的 handler 错误处理落 console），确认框状态由用户「取消」
  复位（UI 可恢复）。本合同不改写生成物。
- AV 腿的宿主采集/结束桥不在本交付：025 的 store 独立单元尚不能在受检 AGE
  装载（auto-lang 上游，见 Plan §10）。
