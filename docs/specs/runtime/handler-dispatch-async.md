# VM 动态 UI 的 handler 派发（片派发 / 完成收尾）

> 状态：PLAN-112 T-09（2026-11-24）。片派发**默认关闭**（`AUTO_CPU_ASYNC=1` 显式
> 开启）；默认仍是同步段入口。原因见 §2「帧泵饿死」。

## 1. 契约

1. **片派发（opt-in）**：`AUTO_CPU_ASYNC=1` 时，动态 UI 的事件 / store handler 经
   `VmBridge::call_handler_for_cpu_slice` 派发（预算 `CpuSliceBudget::d2_default()`）。
   片预算耗尽时 handler park 成 `ParkedWait::CpuRunnable`，帧泵
   (`resume_cpu_slices`) 在后续帧续驱——UI 线程不再被长 handler 占满。
2. **默认**：同步入口 `call_handler_for`，行为与 PLAN-112 之前逐字一致。
3. **完成收尾只一次**：handler 真正跑完（未 park 或泵报告 completed）后执行
   `DynamicComponent::finish_handler_dispatch`：置脏、`dispatch_pending_emit`、
   `on<event>` 实例回调、被剥离 payload 回送、`sync_route_params`、
   `consume_nav_back_pending`。
4. **park 的完成点**：未 park → 派发返回即收尾；park → 记 `PendingFinish`，
   在 `poll_frame_pump` 里 `resume_cpu_slices` 报告 `completed > 0` 的帧补收尾。
5. **`__` 前缀的框架探针事件**不走片派发，保持同步旧路径。

## 2. 已知限度

- **帧泵饿死（默认不开启的原因）**：`poll_frame_pump` 只在 present（帧）时运行，
  而 handler park 后空闲应用不再出帧 → pending 片等不到续驱。025 实测：开启片
  派发后 `Tick`/`Refresh` 停车，`tickN` 冻结在 1。要让片派发默认开启，需先让
  帧泵在「有 parked 片」时自唤醒/自出帧。
- **单 pending 槽**：一个组件同一时刻只跟踪一个被 park 的 handler；多 handler
  同时 park 时按先完成者收尾。
- **同线程**：片派发让 UI 在与片之间呼吸，但不是并行；merged 的 `#[api]` 调用
  仍是同步内联（`host_bridge::call_host` → `f(args_json)`）。
- **错误路径**：片派发返回 `Err` 时打印 `[VM-HANDLER] … failed` 并置脏（不中止 UI）。

## 3. 证据

- 025（313 行进程表）真实窗口（`AUTO_CPU_ASYNC=1`）：`IsHungAppWindow` /
  `SendMessageTimeout(WM_NULL)` 从 8/8 卡死降到 **0/8**（最坏 ~0.6–0.9s）。
- 动态 UI lib 用例 256/256、`desktop_behavior` 11/11（默认同步路径）。
- PLAN-112 T-03 的证据（默认同步路径 + 软拒绝）见 Plan §9。
