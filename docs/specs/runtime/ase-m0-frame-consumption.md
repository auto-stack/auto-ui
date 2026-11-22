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

## 交互会话消费（PLAN-1023 行为 + PLAN-1106 增量修复已交付）

`crates/auto-ui-runtime/src/ui/ase_m0_sessions.rs` 是交互会话宿主的薄消费
适配：纯 980/981 公共面（`NodeDesktopBridge::load/open/close_session/step`
+ `AceStateEngine::read_all`/`dispatch_log` + 全局 HOST 队列/显示），不复制
FFI/解析/窗口，不修改 980/981 模块。已验范围与判别重放边界见 auto-lang
`docs/specs/ase/m0-interactive-sessions.md`（单源权威）；增量会话能力合同见
auto-lang `docs/specs/ace-migration/component-event-sessions.md`。

> **2026-11-19 更正（历史保留）**：1023 的双应用切换/显示/关闭行为证据
> 保留；"外层计数已证明内部handler无重放"的解释撤回（旧 `dispatch_impl→
> refresh` 重编/重执行累计场景）。
> **2026-11-20 修复交付（PLAN-1106）**：`AceStateEngine` 重接 AV 增量事件
> 会话——一次编译、每实例独立 cell 状态根、每事件恰一次 handler 真实
> 派发、读取/切换零派发、卸载释放；外层数字与执行层证据现在一致。

- 接线（冻结）：`ui_apply_*` 钩子定向进程全局 current——双开显示车道被拒；
  **选中车道+引擎常驻**（每会话一 engine 为业务权威，切换=关出向+开目标
  车道，显示从引擎真值重提交；1106 起切换/读取零业务派发）。两活会话上限
  `session.limit` 具名拒；ASE 级关闭释放引擎 owner 与状态根，重开新代次
  种子 0（每代次 init 恰一次）。
- 窗口：`AseM0SessionsWindow`（app 打开行/会话 tab 行/frame 区渲染选中会话
  显示模型；frame 业务按钮→`push_event` 真实 typed 队列，与脚本注入同一
  边界；tick 20ms 受控步进）。原生入口 `examples/ase_m0_sessions_window`
  （`--app k=<path> --evidence <json> [--script "t:open:k|t:click:2|
  t:select:1|t:close:1"] [--hold-ms] [--min-frames]`，`--prep-only` 预备档；
  窗口标题 "ASE M0 交互会话"）。
- 完整目标 `tests/ase_m0_sessions.rs`（`--test-threads=1`；排窗档
  `AUTO_UI_ASE_M0_E2E=1` 单实例含 OS 级 resize/最小化-恢复焦点注入）：
  两活会话 3/7 真保留（loads 钉 2）、同 app 隔离/关闭重开种子 0、增量只落
  选中车道、第三会话拒绝、迟到事件 late_dropped 零效果、20 轮资源对账、
  重放/重建负控同观察口判红、无参轨迹+载荷近邻具名拒。新增 case 全部
  ≤10s（预备档=构建示例+`--prep-only`，日常档零 rustc）。
- **增量执行与执行层 oracle（1106）**：`tests/component_event_sessions_1106.rs`
  ——每事件恰一次入口（`EngineExecutionAudit` 不变量 events==handlers，
  含跨 unmount 折入）、读取/文本刷新零业务派发、同 owner 隐蔽重放/每事件
  重建/getter 派发负控经真实 producer 注入道判红（旋钮生产恒关）；执行层
  观察口与 `loads/dispatch_log` 分账。差量 UI 提交与业务增量执行是两个
  独立门，收据以 1106 验收为准。


## 可见 Clock 页签（PLAN-1109）

`ase_m0_sessions_window` 接收 `--app clock=<Clock 源路径> --app counter-a=<Counter 源路径> --clock-app clock`。在需要验证原生关窗清理时使用 `--await-window-close`，证据包括关窗前的会话和自然退出后的释放状态。现有 `--prep-only` 准备 UI980 provider。运行验证时将 `ACE_NODE_DESKTOP_ROOT` 指向准备目录，并将 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 设为准备阶段的 `rustc --version` 输出。缺失或过期的 provider 会具名拒绝。

Clock 使用现有 20 毫秒宿主循环消费单个已到期唤醒。宿主负责选中、单调截止点与 UTC 日内秒采样，`NodeDesktopBridge::dispatch_typed` 复用受检 AV 派发和文本差量提交。Clock 显示车道在重新选中时等待首个新样本。会话卸载用实际 AV close 记录已释放状态根。 Provider 拒绝关闭时，旧车道进入待关闭状态并失去定时资格。窗体退出继续回收宿主显示车道和所有会话根，证据保留关闭错误。CPU 生命周期入口为 `cargo test -p auto-ui-runtime --features ace-bridge --test ase_clock_lifecycle -- --test-threads=1`。适用范围和验收来源见 auto-lang `docs/specs/ase/visible-tab-timers.md` 与 PLAN-1109 归档。

## AutoScape 浏览器外壳消费（PLAN-1118）

`crates/auto-ui-runtime/src/ui/ase_browser_shell.rs` 是本地浏览器七区外壳的
宿主适配：导航/页签/历史/收藏是外壳自有元数据层，guest 会话经
`AseSessionHost` 的**分段打开面**（`begin_open` 装载成票据 → `mount_open`
容量复检 + 车道落位；票据未挂载即 drop = 引擎释放、零 session/零 UI 效果）
与 `deselect`（切到无 guest 页签撤出向车道、停时钟资格）驱动，复用
1023/1106/1109 的全部会话语义。严格 `app://` 地址解析消费 ase-m0 crate
（`parse_app_uri` 单一语法源，经组内兄弟树 path dep；不复制语法）。已验
范围/冻结身份/导航与收藏语义见 auto-lang
`docs/specs/ase/local-browser-shell.md`（单源权威）。

- 窗口：`AutoscapeWindow`（七区：菜单栏/地址栏/右侧工具/页签/收藏栏/
  内容视口/状态栏；`Ctrl+T/W/D/L` 快捷键经共享 keyboard_subscription 的
  声明式扩展——组件 `key_bindings` 声明 `Ctrl+<Key>` 才投递，未声明组件
  零影响）。原生入口 `examples/autoscape_window`（`--evidence
  <json> --config-home <dir> [--script tick:动作] [--prep-only]`；窄窗档
  `AUTO_VM_WINDOW=520x700`）。
- 收藏/配置：`BookmarkStore` 落宿主用户配置目录
  `~/.config/autoos/apps/autoscape/bookmarks.json`（env `AUTOSCAPE_HOME`
  覆盖），原子写经 `auto_lang_core::state_file::atomic_write`。
- 完整目标 `tests/autoscape_shell_navigation.rs`（`--test-threads=1`；排窗
  档 `AUTO_UI_ASE_M0_E2E=1` 单实例覆盖常用+窄窗）。新增 case 全部 ≤10s
  （预备档=构建示例+`--prep-only`，日常档零 rustc）。
- 给 1119 的公共接缝：导航请求/代次/取消（`PendingNav`/`nav_generation`/
  `stop`）、已验证目标提交（`CommittedPage`/`mount_open`）、视口/状态
  （内容视口二态 + 状态栏阶段）与配置接口（`BookmarkStore` 同型根 +
  原子写入口）。
