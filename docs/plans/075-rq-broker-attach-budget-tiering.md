---
plan_id: auto-ui/PLAN-075
status: drafting
feature_name: RQ broker attach 预算可配置（OS063 DG-2 重产品嵌入闭合）
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [auto-ui/PLAN-074]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [desktop-broker-attach, rq-execution-input-and-host-services]
---

# PLAN-075：RQ broker attach 预算可配置（OS063 DG-2 重产品嵌入闭合）

## 0. 变更摘要

闭合 auto-os/PLAN-063 §10 **DG-2**：重产品（todo/video）宿主侧 attach 构建（
`resolve_incubation_component` → `build_dynamic_component` 进程内编译）超过固定 5s
attach 预算——两次复现、暖缓存同败（OS063 N 扫描 6/8，receipt c93a06c）。修复 =
attach 预算经 env `AUTO_BROKER_ATTACH_BUDGET_MS` 可配置（**默认 5000 零行为变化**），
OS063 战役对重产品扫描设 30000。7374 同域最小合同。

## 1. 目标

env 设定后：重产品（todo/video）attach 在预算内达 Active、虚拟窗落地（OS063 N=8
补全）。不改动：默认预算值、attach 协议/握手、broker 收纳、客户端、产品业务。

## 2. 架构方案

单写面 = `crates/auto-ui-runtime`（host.rs 的 attach_pending_incubations(5000) 调用点
改读 env 解析，缺省/非法 = 5000 原值）；实施树延用 `D:/autostack/.wt/os-063/auto-ui`
（branch auto-os-dev，== master 31e532e4 谱系）。

## 3. 技术栈

Rust、现有 harness；nextest sanctioned 上下文。

## 4. 需求分析与背景调查

### 授权

2026-11-10 用户裁定（OS063 goal 会话）：路径 1——开修复计划修 DG-2 后补齐 N=8。
前置：PLAN-074 已交付（5d8056f3）。

### 诊断事实

- OS063 N 扫描（t04-sweep-*）：todo（app-3/app-9）、video（app-8/app-10）四次 attach
  预算失败；同刻客户端本地构建成功（ Init 落地）——差异在宿主进程内编译时序。
- 预算源：host.rs `attach_pending_incubations(5000)` 硬编码调用点；
  `broker_attach_one(budget_ms)` 逐毫秒轮询 end。

## 5. 详细设计

调用点改：`budget = env AUTO_BROKER_ATTACH_BUDGET_MS parse u32 unwrap_or(5000)`（非法/
缺席 = 5000；0 视为非法回落 5000）。观测行：`[ui-desktop] broker attach budget {n}ms`
（仅非默认值时打）。红线测试用**可控延迟客户端**（手动 transport：connect app-N →
延迟 → 发 Hello）在 小预算/大预算 两档确定性验证（不依赖慢编译）。

## 6. 测试设计

| Case | AC | 输入 | oracle | 任务 |
|---|---|---|---|---|
| V-01 | AC-01 | harness + 延迟 Hello 客户端（默认 5000 档延迟 5.2s=红；env 大档=绿） | env 缺席/合法/非法三态预算值正确；attach 按 env 档通过/失败 | T-01/T-02 |
| V-02 | AC-02 | 真桌面 + todo/video 全方言 + env 30000 | attach Active + 虚拟窗 + 帧合成（DG-2 闭合） | T-03 |
| V-03 | AC-03 | 既有族门（desktop_protocol nextest + ui::session） | 零回归 | T-02 |

## 7. 验收标准

| ID | owner_stage | 判据 |
|---|---|---|
| AC-01 | work | env 三态解析正确；延迟客户端两档 attach 行为符合预算（V-01） |
| AC-02 | work | 真桌面 todo/video env 30000 嵌入运行（DG-2 闭合，V-02） |
| AC-03 | work | 族门零回归（V-03） |
| AC-04 | review | 独立复核 + OS063 消费确认 |
| AC-05 | merge | 落地/归档/清理 + OS063 回执 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖 | 文件/操作 | 关闭 |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 可立即 | 红线测试（env 三态 + 延迟客户端两档）——env 缺席态绿基线 + 大档红灯位 | V-01；AC-01 |
| [ ] | T-02 | work | T-01 | env 解析 + 调用点接线 + 观测行；V-01 全绿；V-03 | AC-01/03 |
| [ ] | T-03 | work | T-02 | 真桌面 todo/video env 30000 嵌入验证（OS063 基座）+ SD 草案 | V-02；AC-02 |
| [ ] | T-04 | review | T-03 | 独立复核 | AC-04 |
| [ ] | T-05 | merge | T-04 | 落地/归档/清理 + OS063 回执 | AC-05 |

## 9. 复审记录

2026-11-10 / stage:new / plan_id:auto-ui/PLAN-075 / revision:r1 / outcome:pass（草案合同，
全链授权随用户路径 1 裁定）。诊断事实绑定 OS063 t04-sweep-* 日志；红线用可控延迟客户端
（确定性，不依赖慢编译）。

## 10. 待澄清事项

- **T-03 实测推翻 AC-02 前提（2026-11-10）**：todo/video 在客户端覆盖门即拒（todo: `style:list-none`；video: `style:native-unstyled, style:stroke-1, tag:video`——禁静默错绘），**不到 attach**。预算旋钮机制已交付并五档验证（2ca3eea6；ui::session 110/0、p074 1/1），但 AC-02"todo/video env 30000 嵌入运行"按原文不可达——真实阻塞 = 产品视图能力在投影覆盖集之外（owner = 投影 coverage 域 + 产品 owner，非本 Plan 预算面）。处置建议：needs_replan（AC-02 改挂覆盖缺口关闭条件）或以旋钮交付 + 具名差额收口——待用户裁定。
