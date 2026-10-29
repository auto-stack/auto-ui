# Vue 轨裸 store.Method 跨 store 路由契约（PLAN-010 实测固化）

> 来源：BLOG-004 T-05 消费者验收暴露（auto-blog `app.at` 多 store 裸调用），PLAN-010 修复后实测固化。

## 多 store widget 的裸 store 调用解析

vue 轨 widget 的 handler/computed/watcher 体中，裸 `store.Method()` 调用按**方法归属**解析：

- widget 的 store 依赖序为 `deps[0..]`（`use x_store: Name` 声明序）。
- 方法唯一归属 `deps[1..]` 中某 store（其暴露面含该方法）且 `deps[0]` 未声明 → emit 该 store 的 facade 变量（`AuthStore → store`、`ArticleStore → articleStore`，与既有 facade const 配套）。
- 方法被多个依赖声明（歧义）、仅 `deps[0]` 声明、或不在任何暴露面 → emit `store.`（首 store，现状）。
- 限定头（`AuthStore.X()`）是显式意图，不参与路由。
- 单 store widget、或驱动未提供 store surface（`ComponentGenOptions.store_surface = None`）→ 不建路由，发射与旧版一致。

**边界**：暴露面取自各 store 的 handlers（msg 处理器名）；驱动解析失败的 store 静默跳过（其方法不路由，回落现状）。裸 `store` 头与局部变量重名的既有风险不变。

## 消费者验证边界

运行时语义链：生成器把 App 级 handler 的裸调用绑错 store 会在 mounted 钩子抛异常，中断 Vue post-flush 队列，丢弃同批后续页面组件的 mounted 回调（初始 LoadList、timer 注册）——表现为"首屏列表空、自动保存不触发"。复验入口：`python tests/dual/verify_dual.py --profile vue-rust`（auto-blog），要求 EXIT=0（含 `ui.autosave_3s`、`ui.refresh_reopen_persists`、`ui.restart_backend_persists`）。

**已知不归本契约**：App.vue mounted 的 auth `token` null 时序异常（独立观测项，不阻塞上述验收）。
