# VM 动态 UI 的 handler 派发（片派发 / 完成收尾）

> 状态：PLAN-112 T-09（2026-11-24）。片派发**默认开启**（`AUTO_CPU_LEGACY=1`
> 回退旧同步入口）；park 后由本帧有界泵 + 帧泵续驱，空闲应用不再饿死。

## 1. 契约

1. **默认片派发**：动态 UI 的事件 / store handler 经
   `VmBridge::call_handler_for_cpu_slice` 派发（预算 `CpuSliceBudget::d2_default()`）。
   片预算耗尽时 handler park 成 `ParkedWait::CpuRunnable`。
2. **续驱（F-01）**：park 后调用方在**本帧内**用
   `pump_cpu_slices_bounded(100ms)` 推进 CPU 泵；帧泵
   (`resume_cpu_slices`) 在后续 present 帧继续续驱。两者共同保证空闲应用
   （无脏帧）不会让 parked handler 饿死。
3. **回退**：`AUTO_CPU_LEGACY=1` 时走旧的同步入口 `call_handler_for`，行为与
   PLAN-112 之前逐字一致。
4. **完成收尾只一次**：handler 真正跑完（未 park 或泵报告 completed）后执行
   `DynamicComponent::finish_handler_dispatch`：置脏、`dispatch_pending_emit`、
   `on<event>` 实例回调、被剥离 payload 回送、`sync_route_params`、
   `consume_nav_back_pending`。
5. **park 的完成点**：未 park → 派发返回即收尾；park → 记 `PendingFinish`，
   在 `pump_cpu_slices_bounded` / `poll_frame_pump` 报告的完成点补收尾。
6. **`__` 前缀的框架探针事件**不走片派发，保持同步旧路径。

## 2. 已知限度

- **单 pending 槽**：一个组件同一时刻只跟踪一个被 park 的 handler；多 handler
  同时 park 时按先完成者收尾。
- **同线程**：片派发让 UI 在与片之间呼吸，但不是并行；merged 的 `#[api]` 调用
  仍是同步内联（`host_bridge::call_host` → `f(args_json)`）。
- **吞吐**：每 tick 的 100ms 泵预算低于同步执行，刷新节拍会比同步路径慢
  （025 实测 tickN ~0.3–0.7/s vs 同步 ~0.7/s），换取消息泵不被长 handler
  卡死（8/8 → 0/8）。
- **错误路径**：片派发返回 `Err` 时打印 `[VM-HANDLER] … failed` 并置脏（不中止 UI）。

## 3. 证据

- 025（313 行进程表）真实窗口（GL 后端，默认派发）：`IsHungAppWindow` /
  `SendMessageTimeout(WM_NULL)` **0/8** 卡死（最坏 ~0.36s）。
- T-03 全量 **21/21**（Phase A 拒绝：具名 + pid + victim 存活 + 自动关框 +
  刷新继续 5→6 / 7→13 + 关窗退出；Phase B/C：授权结束、关框、无拒绝、关窗、
  MCP 消失、无残留、重启 speedDiv=10）。
- 动态 UI lib 用例 256/256、`desktop_behavior` 11/11。
