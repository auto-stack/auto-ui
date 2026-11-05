---
plan_id: auto-ui/PLAN-052
status: drafting
feature_name: rust-ui 原生 exe 生成器对应用 back 源的编译修复
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 0
base_branch: master
base_commit: 8ed8a2e1f2dc93bf31d6f730d380f55bdbc9baa1
depends_on: []
affects: [rust-ui, a2r-codegen, plan-910-publish, auto-os/PLAN-054]
---

# auto-ui/PLAN-052：rust-ui 原生 exe 生成器对应用 back 源的编译修复

## 0. 变更摘要

`auto build -r rust`（PLAN-910 发布链）对 auto-os 统一组合 8 个基础应用
（notes/todo/calendar/paint/database/chat/blog/charts）中 6 个 Rust back
应用生成 68 个 Rust 编译错误；同一锁定工具链组合下 002-counter 对照
**PASS**（exe 发布成功、0 错误）。本计划修复 rust-ui/a2r 生成器对这些
app back 源 constructs 的失配，使 6 app 原生 exe 可构建——auto-os/PLAN-054
iced 轨 Rust 六雄 Outproc 嵌载的具名最小前置。

## 1. 事实与证据（2026-11-05 T-04 实机）

- 复现：候选组合 `D:/autostack/.wt/os-054/auto-os/apps/auto-notes` 执行
  `auto build -r rust` → cargo exit 101，生成 member `notes-45ee43a7` 68 个
  错误；同批 todo/calendar/database/chat/blog 全部同型失败。
- 错误签名（notes，去重后）：E0071/E0117/E0277/E0308/E0425/E0428/E0433/
  E0599/E0609；典型例：`expected Option<serde_json::Value>, found Value`
  （生成码返回类型失配，`serde_json::Value::Null` 未包 `Some`）。
- **合法对照**：`examples/ui/002-counter` 同锁定组合（auto-lang 70b4a1de7 /
  auto-ui b3414750 / auto-man 03bb65e / auto-web 37d32c22）`auto build -r rust`
  PASS，`rust-workspace/002-counter-6e81e48a/bin/debug/counter.exe` 发布成功。
  → 组合无恙，失配为 app back 源特化（这些 app 使用较新 back constructs：
  中文标识符/复合类型/Option 返回等，对照 counter 的极简 back）。
- 历史锚：NOTES-004 时代 rust-ui 重写（plan-910）后 notes 原生曾 19/19
  （tool=auto-ui 2f43b4f），当前 b3414750 对该源失败——疑 2f43b4f→b3414750
  间 rust-ui 生成器回归或对新增 constructs 未覆盖（T-00 定位现有端口后结论）。
- 生成器位置：`crates/auto-ui-tools/src/rust_ui.rs` + PLAN-910 发布链
  （`auto-generation-core::generated_build`，auto-lang 侧消费接口）。

## 2. 目标

- G1：6 app back 源在 master 当前工具链下 `auto build -r rust` 全部 PASS，
  exe 按 PLAN-910 约定路径发布。
- G2：002-counter 及既有 rust-ui 门零回归（对照组与受影响旧门全绿）。
- G3：修复为生成器侧通用能力（constructs 级），不改任何 app 仓源码
  （app 源在各自 002/004/005 交付基线上冻结）。

## 3. 边界

- 不评 8 app 业务行为；不动 auto-os pin；不碰 desktop-host（另案
  auto-ui/PLAN-053）；不在 auto-lang 跑全量 `cargo t`（Category A 纪律，
  只跑受影响固定组）。

## 4. 验收标准

- AC-01：6 app（notes/todo/calendar/database/chat/blog，交付 pin）exe
  构建 PASS，0 编译错误。
- AC-02：002-counter + 既有 rust-ui/910 单测与冻结门零回归。
- AC-03：notes exe 冷启动冒烟（发布路径发现 + 启动退出码 0）。
- AC-04：修复 diff 只落生成器（auto-ui 本仓），逐错误家族具名对账
  （68 错误分族→根因→修复→重测）。

## 5. 执行步骤（草案，待 work 细化）

- [ ] T-00：以 notes 为最小复现，68 错误分族（Option 返回/中文标识符/
  重复定义/未解析符号等），逐族定位生成器根因；产出 affected-constructs 表。
- [ ] T-01：生成器修复 + notes 单 app 构建绿。
- [ ] T-02：6 app 全量构建 + counter/旧门回归 + notes 冒烟。
- [ ] T-03：漂移核对/文档/收据，提交 execution_done。
- [ ] T-04/T-05：review/merge 按本仓流程。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-052 | outcome: drafting（事实证据随案；
  由 auto-os/PLAN-054 T-04 定界产出，消费方=054 恢复后 iced Rust 六雄） | next: work/T-00

## 10. 待澄清事项

编号 052 为立项时占用（.next-id 滞后记 050，以本会话冲突检查 052/053
无在飞分支/树为准）；如与他案撞号，执行前取新号并重命名。
