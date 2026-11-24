# VM 动态 UI 的 handler 派发（片派发 / 完成收尾）

> 状态：PLAN-112 T-09（2026-11-24）。替代「事件 handler 在 `update()` 内同步原地
> 执行」的旧行为；旧行为保留为 `AUTO_CPU_LEGACY=1` 回退。

## 1. 契约

1. **默认片派发**：动态 UI 的事件 / store handler 经
   `VmBridge::call_handler_for_cpu_slice` 派发（预算 `CpuSliceBudget::d2_default()`）。
   片预算耗尽时 handler park 成 `ParkedWait::CpuRunnable`，帧泵
   (`resume_cpu_slices`) 在后续帧续驱——UI 线程不再被长 handler 占满。
2. **回退**：`AUTO_CPU_LEGACY=1` 时走旧的同步入口 `call_handler_for`，行为与
   PLAN-112 之前逐字一致。
3. **完成收尾只一次**：handler 真正跑完（未 park 或泵报告 completed）后执行
   `DynamicComponent::finish_handler_dispatch`：置脏、`dispatch_pending_emit`、
   `on<event>` 实例回调、被剥离 payload 回送、`sync_route_params`、
   `consume_nav_back_pending`。
4. **park 的完成点**：未 park → 派发返回即收尾；park → 记 `PendingFinish`
   （clean_name/emit_widget/state_obj_id/args/input/stripped/is_trace），在
   `poll_frame_pump` 里 `resume_cpu_slices` 报告 `completed > 0` 的帧补收尾。
5. **`__` 前缀的框架探针事件**（`__hot_reload`/`__bounds_collected` 等）不走片
   派发，保持同步旧路径。

## 2. 已知限度

- **单 pending 槽**：一个组件同一时刻只跟踪一个被 park 的 handler（025 的
  `Refresh` 即该形态）；多 handler 同时 park 时按先完成者收尾。
- **同线程**：片派发让 UI 在与片之间呼吸，但不是并行；merged 的 `#[api]` 调用
  仍是同步内联（`host_bridge::call_host` → `f(args_json)`），一段不可抢占的
  原生调用仍会占住当前片。真正的 back 线程/actor 化不在本契约内。
- **错误路径**：片派发返回 `Err` 时打印 `[VM-HANDLER] … failed` 并置脏（不中止
  UI）；emit 路由的完整诊断面仍由回退路径覆盖。

## 3. 证据

- 025（313 行进程表）真实窗口：`IsHungAppWindow` / `SendMessageTimeout(WM_NULL)`
  从 8/8 卡死降到 **0/8**（最坏 ~0.6–0.9s）。
- 动态 UI lib 用例 256/256、`desktop_behavior` 11/11。
- PLAN-112 T-03 的证据在默认派发下复跑（见 Plan §9）。
