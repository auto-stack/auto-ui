# auto-ui 债务/候选登记

非合同的候选功能与债在此登记；立项时按 `README.md` 取号走正式四阶段，
登记条目不构成合同、不算已授权工作。

---

## DEBT-001 autoui MCP 批量动作（一次请求多动作数组）

- **状态**：未立项（候选 003）。来源：PAINT-001（auto-paint）测试驱动经验 +
  用户 2026-10-29 提议。
- **动机**：网格/画布类 UI 的自动化目前一次请求只能做一个动作。256 格画板
  画一张棋盘格要 ~128×N 次 HTTP 往返，每次还夹一趟全量快照（重渲染改
  vnode 哈希 id，驱动器每步都得重新发现元素）。批量模式把"选色 → 画
  [0,0] → 画 [2,0] → …"压进一个请求，收益覆盖所有同类 UI 测试。
- **提议形状**（用户草图）：`autoui_action` 扩展或新增
  `autoui_action_batch`，`arguments.actions = [{element_id, action,
  value?}...]` 按序执行；返回逐项结果（element/status/handler/state_changes）。
- **必须解决的设计点**：
  1. **批内 id 失效**：任一动作触发重渲染后，后续动作里的 vnode id 可能
     全部失效。执行器需要：批内自动重发现（按位置/标签重寻址）、或声明
     id 在批内稳定、或允许条目用稳定定位子（文本/类/序号）替代裸 id。
  2. **失败语义**：单项失败时 abort+已执行清单，还是继续执行+逐项状态；
     调用方如何区分"第 k 项失败"。
  3. **与现有坐标合成的关系**：`pen`/`drag` 已是"一请求一串事件"的先例
     （canvas 类 widget 坐标合成）；批量动作是其按元素 id 维度的对偶，
     复用同一ActionResult 逐项回报形态。
- **边界**：批量的是**传输层**，每个动作仍走该元素绑定的真实 handler
  （press/press_many 不绕过 UI 事件）——测试语义保持"用户路径"。
- **关闭条件**：owner 仓立项（候选 003）并在 mcp_server 落地批量动作 +
  驱动侧（desktop_mcp/verify_basic 族）改用后，PAINT 类验收的点阵绘制
  从 O(N) 请求降到 O(1)。

## DEBT-002 ag-examples AG026/DEBT-001 关闭窗口（M1 N4 重判＋接收器重跑）

- **状态**：未立项（跨仓消费提示，UI076 落地解锁；改动面在 ag-examples，
  不在本仓合同内）。来源：UI076（076-m1-native-scale-trail-and-n4-semantics）
  T-07 消费提示，2026-11-10。
- **动机**：UI076 已落主（8b4ff336）：F-SCALE-02 修复（`observe_window`
  合并语义＋f64 真实读数跟踪——单次切档会话 `tier.scales_observed` 保留
  [起始档, 末档]、`scale_factor_changed` 记真实前后对，validator 单档误拒
  消失）；F-N4-01 语义裁定落地（方案 b：N4 检查表收窄为"空白区按下→离窗
  →窗外释放 ⇒ 无按钮激活、无区域点击"；onclick 区域按下即派发为 canonical，
  SD-01/SD-02 已沉淀入 `m1-native-evidence-support` 与
  `node-mixed-regions-lifecycle` 两规范）。
- **关闭条件**：ag-examples/PLAN-026（AG026）按本仓 8b4ff336+ 引擎
  （1）以新 N4 检查表文本重判 N4；（2）重跑双档接收器（双档会话报告应产
  `scales_observed=[2.0,2.5]`＋真实转换对，可 native-pass）；（3）关
  ag-examples DEBT-001（其 session-bundle 其余证据已齐）。若 AG026 改派发
  时机为 release-inside（方案 a）则与 canonical 冲突，需升格重验——不在
  本提示范围。
