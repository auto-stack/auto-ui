# RenderQueue 执行/输入/宿主服务一致性（PLAN-063）

本页是 AutoUI RQ **公开入口、输入/服务/调度面** 的 canonical 规范：VM 与 Rust
两种执行形态的公开 RQ 入口语义、两 Host（standalone rqhost / desktop 会话
broker）的输入与服务桥、帧调度合同。来源：PLAN-063（r4）的实施与实测。
与 [保真合同](rq-fidelity-contract.md)（v2/v3 wire、字体、资源、P-01/02）及
[资源重放](rq-render-packets-and-resources.md) 的分工：那两页管载荷/资源，
本页管执行形态、输入/服务与调度。**未在本页写明的行为不视为已实现。**

## 1. 公开入口与底座选择（T-02 交付）

同一 App 无论 VM 解释执行还是 a2r 编译 exe，公开 RQ 入口走同一 remote 语义；
`rq`/`desktop` 底座声明恒显式，**无静默 Pixels/direct 降级**。

| 轨 | 入口 | 底座信号 | 语义 |
|---|---|---|---|
| VM | `auto run -r vm -q [--render=remote]` | `AUTO_RQHOST_PIPE` + `AUTO_VM_RENDER=remote` | headless v2 客户端采纳 standalone rqhost；无自窗 |
| VM | `auto run -r vm --render=desktop --desktop-endpoint <pipe>`（`-q` 可省） | `AUTO_VM_RENDER=desktop` + `AUTO_DESKTOP_ENDPOINT` | 采纳桌面合成器端点（不孵化）；端点缺席 = 具名报错 |
| VM | `--render=queue`（或 `-q` 无底座声明） | `AUTO_VM_RENDER` 缺席/`queue` | 显式 legacy queue（DrawList v1） |
| Rust | a2r exe `--render-mode rq\|desktop\|independent` / `-q` / `--desktop-endpoint` | CLI > `AUTO_VM_RENDER` env > pac `desktop_render` > independent | rq/desktop → remote:true headless 臂 |
| 两轨 | 无底座声明 | — | direct（iced 自窗）缺省不变 |

- 值域单源：`render_cli::RenderBase`（`rq|remote|rqhost`、`desktop`、
  `independent`）；VM 侧 `rqhost::resolve_vm_rq_base` 消费同一值域。
- 负控（AC-01）：rq 连不上 = adopt 具名失败；desktop 端点缺席/错名 = 具名
  报错含端点名与"先启动虚拟桌面"提示；未知 `AUTO_VM_RENDER` 值 = 观测行 +
  legacy queue（不冒 remote）；`remote` 缺 `-q` 在 Man 侧激活同一 gate
  （不再静默直渲染）。
- 来源观测：`[render] render-mode: <base> (AUTO_VM_RENDER env)`（VM）与
  CLI/env/pac 观测行（Rust exe）——运行选择来源可审计。

## 2. 输入/服务桥（T-03 交付）

线词汇 = InputMsg 1..11（指针/键/字符/滚轮/IME/Action）+ ControlMsg（窗服务）。
两 Host 生产面（对齐 direct iced 事件面）：

| 输入族 | standalone rqhost | desktop 会话 broker |
|---|---|---|
| PointerMoved（hover） | daemon CursorMoved 臂 | `broker_pointer_move`（`__mouse_moved` 臂 WM 未消费时，投影换算同 press） |
| PointerPressed/Released | 真修饰位（daemon `last_modifiers`） | 真修饰位（`desktop.current_modifiers` 事实源） |
| KeyPressed/CharTyped/IME 三态 | LiveInput 沿（既有） | LiveInput 沿（既有） |
| KeyReleased | `LiveInput::KeyReleased`（PLAN-063 首个生产者；Named→VK 同表） | `broker_key_released`（焦点窗） |
| 焦点 | `RqMessage::WindowFocus` → `ControlMsg::Focus` | `wm_focus`/`wm_focus_soft` 焦点变化沿旧窗 blur/新窗 focus |
| 剪贴板（widget 面） | `headless::OsClipboard`（arboard 桥；Primary 同 direct winit 律 None/忽略） | 同左（outproc 客户端同臂） |

- 客户端消费：headless v2 臂把线输入转 iced 事件（KeyReleased/焦点/修饰
  合成既有）；v1 queue 投影器忽略未知控制/释放（`_` 臂）——旧端安全。
- handler 面剪贴板/对话框等 VM native 沿既有 provider（060/418 交付）。
- 具名边界（不视为已实现）：`ControlMsg::TitleChanged` 端点仍丢弃
  （endpoint.rs 丢弃臂；无生产者，direct 同无运行时改题）；OS scale/DPI
  不下发客户端（daemon 侧 IME 坐标换算已用；客户端恒逻辑像素，Host 合成
  期缩放）；a11y 无协议面；文件对话框客户端进程 unparented。

## 3. 帧调度（T-04 交付；P-03）

- **输入→帧同拍**：rqhost daemon 六事件臂（Live/CursorMoved/Pointer 按松/
  WindowFocus/WindowResized）后联 `pump_window`——输入/焦点/resize 触发的
  帧在同一 update 拍合成，不经档位等待。
- **自适配兜底档**：`Every(duration)` 以时长哈希为订阅 id——有在册客户端
  = 活跃档 15ms（客户端自驱帧：timer/动画/异步产帧的排水节拍，与既有
  预算同律的具名 fallback）；零客户端 = 待命档 250ms（采纳巡逻，空闲近
  零开销）。固定 15ms 不再是唯一帧驱动。
- 客户端泵：`recv_wait(25)` 阻塞等待——wire 消息（输入/ACK/背压）到达即
  派发；组件 timer 粒度上限 25ms（具名边界：组件 timer deadline API 需
  跨 VM/Rust 全 producer 扩 trait，本轮未扩）。
- 呈现反馈：present 物理上屏无测量源——评测终点为合成/present 代理
  （062 r5 同律）；M-01 分段测量点以 T-01 冻结口径（t0..t7，代理具名）。
- P-04（per-App 公平/计量）：**部分交付**——per-client 单拍泵预算切片
  （`PUMP_MSG_BUDGET_PER_TICK=64`，§3.1）；per-App 计量/驻留上限/故障
  隔离机制未实现——后继轮次交付，最终规模组合归 OS063。

## 3.1 运行记录（2026-11-08，rqhost in-proc 真实采纳测量；环境 = 共享开发机安静窗 CPU~15%）

| 指标 | 记录 | 口径 |
|---|---|---|
| M-01 输入→帧到宿主 | p50=0.055ms / p95=3.18ms / max=4.42ms（n=40） | 代理终点=帧到宿主（含客户端 update/编码/线往返/泵收帧）；硬门 p95≤100ms（T-01 冻结）通过；物理上屏归 OS063 |
| M-02 帧节奏 | 重客户端 inter-frame p50=263.7ms（client timer 250ms 档） | 记录；真实刷新 deadline 组合归 OS063 |
| M-04 守恒 | 每点击 ≥1 帧状态推进、≤2 帧（无输入丢失/无洪泛） | 有界守恒（hover/release 语义可 +1 帧） |
| M-05 内存 | 测试进程 working set 36.5MB（守护+客户端 in-proc 合并） | per-App 隔离口径归 OS063 真进程 |
| M-06 启动 | 冷（首客户端 spawn→首帧）60.0ms / 热（第二客户端）36.9ms | 同守护热态；组合/规模化归 OS063 |
| M-07 公平 | 重载（timer 连续帧流）下轻 App 点击 p50=2.97ms / p95=3.11ms | 硬门 p95≤100ms 通过；mild 负载档，泛洪/1-4-8 规模归 OS063 |
| P-04 机制 | `PUMP_MSG_BUDGET_PER_TICK=64` 单客户端单拍泵预算 | 切片公平：重客户端泵不独占守护拍；FIFO 保序不丢 |
| V-03 timer | 零输入 timer 帧 ≥2/1.6s，ticks 逐帧恰 +1（订阅恰一次） | 不依赖输入/守护拍驱动 |
| V-12 重建 | 测试二进制 `27654085…`@`304e1fd3`、Man exe `5178a7b4…`@`347c29f`、auto-ui-tool `7aafb31a…` | t01-freeze §9；哈希不符 = 失败 |

具名未测（本页边界，不视为通过）：M-03 CPU/GPU 时间（GPU 计时平台能力缺席——062 同律具名）、高刷新率适用面（无实测高刷环境）、M-01 物理上屏终点、1/4/8 App 规模组合（全部归 OS063）。

## 4. 旧红适用与关闭（V-11，2026-11-08 全闭）

四个顺序滤程旧红全部定位到系统性根因并修复（非环境豁免）：

| 红 | 根因 | 修复 |
|---|---|---|
| autocenter_translates_hits_with_ops | env 门测试置 `AUTO_NO_AUTOCENTER` 不还原——顺序滤程同进程串染全部后续 `RqProjector::new` | 测试 panic 安全还原（回归保留） |
| stage3_multi_app_stress_n3/n5 | ①压测子臂 `proj_autocenter_off` 合同下，固定点击 (60,40) 未加宿主标题栏偏移（`broker_command_pointer_local` 减 36 → (60,4) 落在按钮上方）；②PLAN-678 居中孪生与 off 子臂错位 | 点击 = off 律孪生按钮中心 + 窗原点 + TITLEBAR_H |
| t3_examples_queue_end_to_end | 同一标题栏偏移缺失（twin 内容坐标 → desktop 点击未加 36）；005 submit 同 | 全部 twin 点击补 TITLEBAR_H；孪生视口改 `vwin_content_size(rect)`（place 后构建） |
| t2_remote_ws_session_end_to_end | **产品缺陷**：`ClientPump::dispatch` 无 `FrameMsg::Resize` 臂——宿主 `sync_broker_client_viewport` 的视口同步被静默丢弃，子面恒 Hello 尺寸（480×900）渲染，与宿主孪生（rect 482×712）错位 | 泵补 Resize 臂（转发端点 + 立即产帧）；孪生视口改内容尺寸同律 |

副产品：视口同步臂修复同时闭合 rqhost/桌面客户端**采纳期与窗 resize 后子面
不重排**的产品缺陷（此前 remote 子面永不跟随宿主窗 resize）。

## 5. 重建身份与计时条件（V-10/V-12）

- 重建集：auto-ui workspace（auto-ui-runtime/auto-ui-tools/auto-macros）+
  Man `auto` exe（ui-extra）+ `auto-ui-tool` + a2r 生成 exe。运行证据绑
  各仓 HEAD + cargo profile + 进程 PID；旧 exe 冒充 = 失败。
- 计时：性能测量仅隔离窗口（AUTO_RQ_TIMING_WINDOW=isolated + 环境负载
  记录）；共享窗 timing_gate=skipped；UI061 冻结预算（W-A/B/C、replay
  p50 门、上传硬零）继续有效，端到端 M 指标不以上游局部门抵扣。

## 6. 分工边界

- UI073：连续媒体/RemoteFrame、精确绘制收口（13 非 exact 族/glyph 残差/
  SVG produce）、资源乱序注入负控；本页供真实接口（generation/release/
  HostCaps/Metric 通道）。
- OS063：两 Host 最终产品组合、1/4/8 规模、M-02/06/07 终值与完整 P/M
  验收；通用 Host adapter 面（本页 §1-3）只做策略装配。
