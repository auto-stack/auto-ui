---
plan_id: auto-ui/PLAN-053
status: drafting
feature_name: desktop-host 多 api surface 演进与 run_ui 栈溢出根修
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 0
base_branch: master
base_commit: 8ed8a2e1f2dc93bf31d6f730d380f55bdbc9baa1
depends_on: [auto-ui/PLAN-050]
affects: [desktop-host, vue-shell, mcp-server, auto-os/PLAN-054]
---

# auto-ui/PLAN-053：desktop-host 多 api surface 演进与 run_ui 栈溢出根修

## 0. 变更摘要

`auto run --desktop`（vue 桌面宿主，生成于 `auto-ui-tools/src/vue.rs`
desktop-host 模板）当前为 v1 单视图/单 api 形态，对 auto-os/PLAN-054 的
8 个基础应用组合形成两个硬障碍：①shared `@/lib/api` 命名空间全桌面仅一
个 api-client app 可嵌（其余 skip "API client namespace claimed by …"）；
②扫描/装载阶段 `work:run_ui` 线程默认 8MB 栈溢出（thread has overflowed
its stack），RUST_MIN_STACK=64MB 下不复现（深递归型，根因未定位）。本计划
演进宿主到 per-app api surface（或多 surface 并存）形态并根修栈溢出——
auto-os/PLAN-054 vue 轨 8 app 嵌载的具名最小前置。

## 1. 事实与证据（2026-11-05 T-04 实机）

- 障碍①：候选组合 8 app 全部预生成 `gen/front/vue/src/lib/api.ts` glue
  （6/6 补生成 PASS + notes 已有 + charts 不需）后，desktop-host 扫描仍
  只放行**第一个** api-client app（auto-blog 字母序胜出），其余 7 个 skip：
  `API client namespace claimed by …/auto-blog/gen/front/vue/src/lib/api.ts`。
  代码锚：vue.rs ~5940-6000（"v1: one api surface per desktop"，首赢者
  重写共享 `src/lib/api.ts`，后续 app 撞名 skip）。
- 障碍②：首次 `auto run --desktop`（AUTO_DESKTOP_APPS=候选 apps）在扫描
  24 app 后 `thread 'work:run_ui' (…) has overflowed its stack` →
  `run_ui failed: tool host closed the control stream`；同输入
  RUST_MIN_STACK=67108864 下不再溢出并走到 dev server 启动（端口另障见下）。
  属深递归/深嵌套（疑 corpus 合并或 widget 树递归），根因未定。
- 伴随观察（非本计划核心但顺带记录）：shell 默认端口 3000 落在本机
  Windows TCP 排除段 → EACCES；`AUTO_FRONT_PORT` 可覆盖（已实证绕过）。
- 消费方面：auto-os/PLAN-054 §5.5 明确——宿主不兼容时给最小反例与 owner
  前置，受影响任务保持未完成，不退为外部窗口冒称嵌入。本计划即该前置。

## 2. 目标

- G1：desktop-host 支持 N 个 api-client app 同场嵌入：每 app 独立
  api client 命名空间（如 `@/lib/api-<id>` 或 alias 表），store/ext 等
  共享面冲突策略显式化；8 目标 app 全部出现在 shell 并可打开。
- G2：`work:run_ui` 在默认栈深下不再溢出：定位递归根因改为迭代/限深，
  大 app 集（≥24）扫描装载为受面；回归用同输入候选组合。
- G3：v1 的其余守卫（router pages/ext/i18n skip）行为不变，除非逐案
  有界扩展并记录——不连带做大桌面重写。

## 3. 边界

- 只动 auto-ui（宿主生成 + mcp/tool host 如需）；不动 auto-os shell.at
  业务；不改 8 app 源码；不承诺双 surface 之外的桌面新形态。
- run_ui 栈修复不得以单纯调大默认栈交差（须根因修复 + 大输入回归）。

## 4. 验收标准

- AC-01：候选组合 8 app 在 desktop-host 全部可嵌可开（启动器列出、
  窗口打开、前后端各自 profile 工作）。
- AC-02：默认栈深（不设 RUST_MIN_STACK）扫描装载 24+ app 不溢出。
- AC-03：现有单 api-client app 组合（如仅 blog）零回归；既有
  desktop-host/vue.rs 单测全绿。
- AC-04：变更只落 auto-ui；命名空间策略有文档句（spec delta 按本仓流程）。

## 5. 执行步骤（草案，待 work 细化）

- [ ] T-00：多 surface 设计决策（alias 方案 vs 独立 chunk）+ 栈溢出
  根因定位（递归路径最小复现）；产出 decision 文档。
- [ ] T-01：实现 + 单 api 旧形态回归。
- [ ] T-02：候选组合 8 app 实机验证（嵌/开/基本任务）+ 大输入不溢出。
- [ ] T-03：漂移/文档/收据，execution_done。
- [ ] T-04/T-05：review/merge 按本仓流程。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-053 | outcome: drafting（证据随案；
  由 auto-os/PLAN-054 T-04 定界产出，消费方=054 恢复后 vue 轨） | next: work/T-00

## 10. 待澄清事项

编号 053 为立项时占用（撞号处理同 PLAN-052 §10）；G1 若评估为大型
架构改造（超出"最小前置"），回 auto-os/PLAN-054 §10 由用户裁量承载目标。
