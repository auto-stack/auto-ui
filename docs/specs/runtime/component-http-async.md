# 组件面 UI 异步 HTTP 与 A2Vue 承载策略（PLAN-108，2026-11-22）

本文件沉淀 PLAN-108（U06 032-handler-async-probe 双端交付）的两项
跨端 enduring 决定。语言侧机制（HttpCall/park-resume/保留-重播种）见
auto-lang `docs/specs/ace-migration/component-event-sessions.md` 的
`http.client/1` 节——本文件不重复，只记 AutoUI 消费侧合同。

## A2Vue 严格轨的 Http 效应承载（`AUTO_UI_ACE_VUE=1` 产品 lane）

组件面 lane 编译不认识（或不能派生）某 handler 时，缺席不得静默回落
旧道（980/982 双轨收紧）。承载（carried）是唯一的具名放行通道：

- 承载判定（两臂统一，F-1 修复后）：handler 体 **call-clean**（除闭合
  `Http.<verb>` 族外无任何调用）且 **widget 级确有 Http 效应**。纯状态
  兄弟（如 `.Bump`/`.Tick`）随 Http handler 一并承载；非 Http 的域外调用
  恒为具名缺席硬拒。`compile_failed` 相位（整体失败 vs 逐 handler 缺席）
  不改变判定——组件面 Http 准入后 lane 编译可对 Http widget 整体成功，
  逐 handler 缺席臂必须与整体失败臂同 predicate（对称化裁定）。
- 承载体的旧道 emission 必须是受检 TS 映射（`ts_adapter` 的
  `await fetch` 族，OS068 R-01/032 测试在案）；工具键门打印
  `ace.vue-lane-carried: <file>[<widget>[<handler>…]]` 诊断行——
  **逐名在案=非静默**。
- 已验入口：`cargo test -p auto-ui-runtime --features ace-bridge --test
  plan108_http_lane`（4 例：正例 carried 逐名 / 未知调用 / Http+未知
  同体 / 无 Http 失败不承载）。

## AV 生产宿主的组件 Http 派发与 Tick 合成（ace_node_desktop）

- 含 Http 效应的组件程序：派发经 **park 车道**——会话不上线程；
  `DispatchOutcome::InFlight` 表示 handler 在 HttpCall 挂起，完成经逐步
  `poll_http_completions` resume 同一会话并落账。pending 门按
  (identity, msg) 忽略同 msg 重入（HttpBusy 静默背压，非错误）。世代/
  存活核对不符=worker 结果丢弃（卸载后迟到结果隔离）。
- provider 为 job 式（`UiHttpProvider`：start 把明文 HTTP/1.1 请求
  （std TcpStream，闭合域=明文 http 慢端点；代理/TLS 不在 M 域）派上
  job 线程，poll 非阻塞收编）。
- **组件 Tick 合成**：程序含 `.Tick` handler + `interval` model 字段时，
  桥按 250ms 间隔（interval 语义默认）经 BridgeTick 逐步派发 Tick；
  close→reopen 窗口按宿主会话 closed 门跳过（对新会话 apply 会
  SESSION_CLOSED 噪音）。挂起期 Tick/Bump 照常交互（park 语义）。
- 已知缺口（显示层，记债）：reopen 后 provider 侧显示模型重绑不完整，
  tick 触发的 apply 可能 -2；模型语义不受影响（宿主门已兜 close 窗）。
EOF
