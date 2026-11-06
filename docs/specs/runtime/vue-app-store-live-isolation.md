# Vue App 级 Store 活体隔离与 Owner 退出（vue app store live isolation）

> PLAN-057 交付合同（SD-01）。真实浏览器挂载下的运行语义：同生成模块同名
> App 双挂载默认隔离、App 内消费者共享、显式共享、关闭/重开与
> watcher/timer/SSE 资源退出。上游形状合同（厂形/composable/context 注入/
> legacy 边界）见 component-and-store-instances.md §3；本文补其 §5 如实
> 记录的浏览器缺口。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/057-vue-app-store-isolation-and-owner-lifecycle.md

## 1. 活体隔离（AC-01 / V-01）

- 同一生成 App（同模块同名 store）两个 `createApp` 挂载各持独立 context：
  A 写 11 而 B 保持初始 22，互不覆盖。退回模块单例的错误实现必被该 oracle
  发现（双挂载写读对拍）。
- App 内多消费者（根视图两处直读 + 子组件各自 `use{Name}Store()`）解析同一
  App 实例：任一写入口后全部视图一致同步；store watcher 在该 App 恰触发
  一次。

## 2. 显式共享与 legacy（AC-02 / V-02）

- 显式共享仅经 context 注入端口：两个挂载复用同一外部 context（`install
  AutoStoreContext(app, existing)` 第二参，041 既有入口的owner化扩展）→
  唯一 store 实例跨 App 同步；共享实例的 watcher 跨两个消费 App 只有一份。
- 无 provider 挂载维持 041 具名 legacy 边界：模块懒单例，多个无 context
  挂载共享同一 legacy 实例（旧行为如实保留，不是隔离缺陷）。
- 三种挂载互不串扰：显式共享不泄漏进 per-App 挂载；legacy 单例不影响
  context 挂载；per-App 写不进入共享/legacy 实例。

## 3. 资源 owner 与退出（AC-03 / V-03）

- store 实例创建的全部 effect（EventSource、`setInterval`、`watch`）在
  factory 内经 `__autoOnCtxDispose(cb)` 登记进创建它的 App store context
  （`stores/__autoStoreContext.ts`）。
- context 按 owner App 记账（`own`/`releaseApp`，Vue ≥3.5 `app.onUnmounted`
  钩子与挂载工厂显式释放按 App 去重）；**最后 owner 退出 → `release()`**：
  连接关闭、定时器清除、watcher 停止、实例 memo 清空。
- 部分 owner 退出：资源保持归属，幸存 App 继续处理（unmount A 后 B 的
  SSE/timer/watch 照常；A 的连接由 mock 送达账本实证关闭）。
- 迟到消息按连接归属：已退出 owner 的连接不再收到消息，不复活、不改写
  重挂的新同名 App（新挂载新 context 新连接）。
- SSE onerror 自动重连在实例释放后失效（`__auto_sse_live` 守卫），不为已
  释放实例重开连接。
- 重挂（新 context）零旧值、恰一份订阅/一份 watcher/一条新连接；被释放旧
  实例保持静默（tick 冻结、sse_count 不再增长）。

## 4. 入口与两条冷路径（AC-04 / V-04）

- 生成链全程真实：.at → VueGenerator → `main.ts`（scaffold/regen）与
  gallery mount 工厂发射 `installAutoStoreContext(app[, existing])`；不
  以手写等价 store 冒充生成器支持。
- 冷 scaffold 路径（`incremental_compile_changed` → `generate()`）与
  run/start 自愈路径（main.ts/vite.config.ts 删除 →
  `regenerate_source_files()` + `refresh_vite_config()`）各至少一次真实
  浏览器挂载；refresh 重生的 vite.config 与冷生成字节一致（050 两路径
  合同不回退）。
- 浏览器 pageerror 恒零；051 修复（main.ts 拼接、结构默认值）不回退
  （`test_plan051_*` 编译门 + main.ts 安装断言共同覆盖）。

## 5. 验收锚点

- 永久 fixture：`crates/auto-ui-runtime/tests/fixtures/plan057_vue_instances/`
  （P057App + 两个 P057Reader 消费者 + 携 watch/timer/SSE 的 P057Store）。
- 生成契约门（默认 `cargo test` 选中）：`cargo test -p auto-ui-tools
  --test plan057_vue_instances` —— 冷 scaffold 真实产物上的 dispose 接线
  （`__autoOnCtxDispose`/`__auto_sse_live`/owner 记账）与 041 解析形状
  断言。
- 浏览器 e2e（计划记录命令，重环境显式运行）：`cargo test -p auto-ui-tools
  --test plan057_vue_instances -- --ignored --nocapture`。环境：node +
  pnpm（华为云镜像）+ playwright（`auto-lang/packages/auto-forge-ui`
  node_modules）+ msedge headless；oracle 脚本
  `crates/auto-ui-tools/tests/plan057_browser.mjs` 自持 SSE mock（固定
  127.0.0.1:39579，loopback；/stats 送达账本即资源 oracle）。
- 单挂载（生成 main.ts 冷路径）与五挂载对拍（A/B 独立、S1/S2 显式共享、
  L1/L2 legacy）两阶段各计一次真实挂载。
- 041 VM 门（`cargo test -p auto-ui-runtime --test
  plan041_instance_ownership`）与 compiler 厂形契约原样保持绿。
