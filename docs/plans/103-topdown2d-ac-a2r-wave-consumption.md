---
plan_id: PLAN-103
status: drafting
feature_name: 俯视二维组合 AC/A2R 波内消费与触碰路由
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [auto-ui/PLAN-058, auto-lang/PLAN-1068, auto-lang/PLAN-981]
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-008]
---

# PLAN-103：俯视二维组合 AC/A2R 波内消费与触碰路由

## 0. 变更摘要

2026-11-19 首版执行合同（r1），drafting/0 完成。用户裁定（同日）：把 PLAN-058 的 AC/A2R 内容整体转移到本计划（转移清单见 §5 继承表，原封不减），PLAN-058 收口 AV 面先行合并；本计划等 **1068/T-04＋981 AC 域端口＋UI-W2-AC**（进而 **981 W3 A2R 端口＋UI-W3-A2R**）落地后再开工。承接四类义务：①V-S01..08 各行的 AC 格/A2R 格（行 ID/输入/expected/独立 oracle/包络逐行原封）；②AC-10（同一 Node 源经 981 在 AC 与 A2R 真实运行）；③双后端 AG023 配方与冷包；④触碰路由语言缺口（IF-TOPDOWN-TOUCH-ROUTE）。仅规划；未建树、未跑 Cargo/GPU、未动代码/Spec/价目/文法。

## 1. 目标

同一 UI058 最低链 Node 源（1Player+2障碍+2同定义 Target 实例＋可见非阻挡地面；`fixtures/plan058/topdown-node/main.at`）经 **981 AC 机器车道**真实执行、经 **981 A2R→rustc** 真实编译运行：行为/预算/呈现/故障/恢复与 AV 面已交事实逐格对齐（同一输入→同一独立 oracle 值），并补作者面**触碰路由**（玩家接触命中目标→业务派发 Touch→实例状态 2:1）完整形，产出 AG023 双后端准确版本/命令/配方。

非目标：不重复 1068 的 AC compiler 实现与 981 的语言/A2R 后端实现；不改 AV 面已交事实（`350af5b8`/`12cb146b` 与 UI058 SD-01 落地文本）；不动旧 021/022/032 档与价目；不扩 tilemap/存档/光影。

## 2. 架构方案

消费既有交付，不建第二实现：AC 车道=1068/T-04 的 AC 机器执行（同 host 资源/事件、独立 CPU/GPU/错误/恢复/包）；A2R 车道=981 RUST/FUSION 域端口＋978 构建（真实生成→rustc→无 VM 执行）；触碰路由=981 图形端口新增 dispatch 类 op（IF-TOPDOWN-TOUCH-ROUTE，具名请求见 §5）。夹具/资产/独立 oracle/预算口径全部复用 UI058 已交付件（`fixtures/plan058/topdown-node/`、`tests/plan058_t04_*`、`tests/plan058_t05_*`、`tools/verify_plan058.py`），不复制成新框架。唯一执行树与分支按 auto-ui 惯例届时新建（`<group>-103`）。

## 3. 技术栈

消费面：1068 AC 机器执行接口（含 host ABI/provider 形）、981 `IF-AGE-GFX-AC`/`IF-AGE-GFX-A2R` 域端口（GFX_CALLBACK_CREDIT_BUDGET=32、segment8/sweep16/其余1/字段读0、`AGE_PORT_FIELDS` record ABI 同源）、978 A2R 构建链、UI058 的 AV 正式来源（pin ≥`0a167f611` 的 `.sdk-source` 基座）。UI/Man/旧包不涉。

## 4. 需求分析与背景调查

**授权（2026-11-19 用户裁定）**：AC/A2R 从 PLAN-058 整体转移至本计划；PLAN-058 收口 AV 面先合并；本计划等外部波次落地后再开工。本轮只规划。
**转移来源**：PLAN-058 r8（git `<(058 r8 提交)>` §5 r8 块转移清单五项；其 §6 V 表、§7 AC-10/AC-PHASE-GFX 原文为继承底本）。
**AV 正式来源（已交付，本计划消费不改写）**：正常 Node 源经 981 AV 真跑（40 Tick 逐 Tick oracle/停位 8.6/每 Tick 32 credits；GPU 同宿主连续帧；use 实例 2:1；epoch/fault-13/33 消费），证据见 UI058 §9 各轮收据（`b071634c`→`12cb146b`）。
**外部前置现状（2026-11-19 观察即冻结基线）**：PLAN-1068 `drafting` 0/10；981 `IF-AGE-GFX-AC`/B-RUST* 未交付；UI-W2-AC/W3-A2R 未落地。开工栅栏=T-01 核对全部落地。
**未决**：无（等待项即 §8 T-01 的开工栅栏，不是合同未决）。

## 5. 详细设计

### 5.1 继承表（UI058 r8 转移清单 → 本计划义务，原封不减）

| # | 承接项 | 原位置（PLAN-058 r8） | 本计划落点 |
|---|---|---|---|
| 1 | V-S01..S08 各行 **AC 格** | §6 V 表各行 | T-03（V 各行 AC 列补格；V-PHASE-GFX ②同源 AC 禁 AV guest 仍须独立预期、假机器执行须红） |
| 2 | V-S01..S08 各行 **A2R 格**（含 V-S07 A2R 腿） | §6 V 表各行 | T-04（真实生成/rustc/无 VM 运行/冷包） |
| 3 | **AC-10**（同一 Node 源经 981 在 AV 和 A2R 真实编译运行；AG023 两入口） | §7 AC-10 | T-03/T-04/T-05 合并关闭 |
| 4 | V-PHASE-GFX 的 AC/A2R 格与多端"整包拒绝"条款（任一原 required 格缺失/错源/ABI/资源/候选或只贴截图→整包拒绝） | §6 | §6 本计划 AC-PHASE-GFX 承接行 |
| 5 | **双后端 AG023 配方**（两入口准确版本/命令/冷包/失败恢复） | §8 T-06 双后端部分 | T-05 |
| 6 | **触碰路由缺口**（作者面业务→实例事件派发；17-op 表无 dispatch 端口） | D-06 具名缺口 | T-02 IF-TOPDOWN-TOUCH-ROUTE |

### 5.2 阶段门（等待项即开工栅栏）

| 阶段 | 前置 | 本计划产出 |
|---|---|---|
| 栅栏核对（T-01） | 1068/T-04 交付＋981 `IF-AGE-GFX-AC` 落地＋UI-W2-AC 收据 | 唯一树/scope/正式来源 pin 记录；未齐=blocked（不动 AV 面已交事实） |
| AC 格（T-03） | T-01；1068 AC 机器接面与同 host 资源/事件 | 原 V 行 AC 格逐格闭合＋独立 phase 复核 fold 交 `IF-AGE-GFX-AC` |
| 触碰路由（T-02） | T-01 后可与 T-03 并行 | IF 并入 981→语言交付→作者面 V-S03 完整形（接触→派发→2:1 状态）真跑 |
| A2R 格（T-04） | T-03 闭合（A2R 后于 AC）＋981 W3 A2R 域端口＋UI-W3-A2R 收据 | 实际生成/rustc/最终包/无 VM 执行证据＋独立 phase 复核 fold 交 `IF-AGE-GFX-A2R` |
| 配方（T-05） | T-03/T-04 | AG023 双后端准确版本/命令/冷包/失败恢复 |

### 5.3 IF-TOPDOWN-TOUCH-ROUTE 具名请求内容（并入 981 §9）

请求：AGE 图形/事件端口新增**作者面实例事件派发**最小面（候选形：`instance.dispatch(node, instance, msg)` 或同族 typed op），携带：预算（计 1 或 0 credit 须 981 冻结并同表）、身份（按受检行描述的实例事实寻址，不猜行边界）、G2 语句面兼容（hook-local/handler 内可调用）、TS/WEB 车道对应拒/复放语义、永久用例（provider 只收一次、命中目标恰收一次、未命中零派发）。UI058 侧事实基础：`__age_state_<inst>_hits` 状态面与 ForNode 场景分派已证（`12cb146b`）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/node2d-topdown-composition.md | 支持域行由"AV 面（PLAN-058 SD-01 落地）"扩为"AV/AC/A2R 三端同源支持＋触碰路由作者面"；各端身份/命令/配方行随 T-03/T-04 证据填入 | 结论和实际支持域一致（AC-08 同则） | AC-01..06/10/11（本计划承接面） |

（本目标文件由 PLAN-058 SD-01 先行落地；本计划只修订其支持域行，不新建文件。）

## 6. 测试设计

V 行底本=PLAN-058 r8 §6 原表（行 ID/输入/expected/独立 oracle/包络原封）；本计划补各行的 AC/A2R 列执行证据与下列新增用例。AC-PHASE-GFX 承接行：任一原 required 格缺失、错源/ABI/资源/候选或只贴截图，**整包拒绝**；同源 AC 禁 AV guest 仍得独立预期、假机器执行必须红；A2R 保真实生成/rustc/无 VM 运行/冷包。

| case | AC | owner_stage | 输入/入口与可区分预期 | 覆盖边界 |
|---|---|---|---|---|
| V-T1 | AC-10 | work | 同一 main.at 源经 AC 机器执行 40 Tick：逐 Tick 位/sweep/segment/credits 与 AV 面独立 oracle 全同；超界 fault 语义同表 | 不以 AV 代 AC；假机器执行须红 |
| V-T2 | AC-10 | work | 同一源 A2R 生成→rustc→运行：同 oracle 逐格同值；无 VM/解释器执行证据 | 旧已验 A2R 能力只复用同身份证据 |
| V-T3 | AC-03/04 | work | 触碰路由作者面：玩家接触命中目标→派发 Touch→A 2/B 1 状态独立；未命中零派发；预算同表 | 不手造HIR/不建私有 runner |
| V-T4 | AC-04/05 | work | 双后端冷 A→B/坏源修复下一 Run/最终包失败恢复 | 沿 PLAN-058 V-S04/V-S05 原输入 |

计划级：只跑受影响 case/必要消费门；GPU/窗口重门按机器单实例；不为 merge 重复全量回归（周期批量归 merge 后 due 门）。

## 7. 验收标准

- [ ] **AC-PHASE-GFX（承接） / work**：原 V 行的 AC/A2R 格按 §5.2 阶段门闭合，原独立 oracle/包/资源与失败义务守恒；缺任一原 required 格→整包拒绝。由 V-T1..T4、T-03/T-04 与 T-06 final 核验。

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 1068/981 AC 域正式接口真实闭合且本计划版本化支持表可审 | T-01/T-03 |
| AC-02/03/04/05/11 的 AC/A2R 格 | work | 各 V 行 AC/A2R 列补格：同源同 oracle、预算同表、故障/恢复/冷包双端成立 | V-T1..T4 |
| AC-06 | work | AG023 双后端准确版本/命令/配方/冷包齐 | T-05 |
| AC-10 | work | 同一 Node 源经 981 在 AV（已交）与 AC、A2R 真实运行；无新旧生成链/重复语言后端 | V-T1/T2 |
| AC-07 | review | 同树独立 final 逐承接 AC/转移表/反例 pass | T-06 |
| AC-08/09 | merge | 落主/canonical（SD-01 支持域行）/归档/对账/清理 | T-07/T-08 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | **开工栅栏**：1068/T-04＋981 IF-AGE-GFX-AC＋UI-W2-AC 收据落地 | 只读核对正式来源/接口/命令/树与 scope，登记 pin 与身份；未齐=blocked 具名记录 | AC-01 | [ ] |
| T-02 | work | T-01 | IF-TOPDOWN-TOUCH-ROUTE 并入 981→语言交付→作者面触碰路由 V-S03 完整形真跑（T-03 预算/身份同表） | AC-03/04 | [ ] |
| T-03 | work | T-01 | 原 V 行 **AC 格**逐格闭合（同源 AC 机器执行、同 host 资源/事件、真实包/失败恢复）；独立 phase 复核 fold 交 IF-AGE-GFX-AC | AC-02/03/04/05/11 格 | [ ] |
| T-04 | work | T-03 pass；981 W3 A2R 端口＋UI-W3-A2R 收据 | 原 V 行 **A2R 格**逐格闭合（实际生成/rustc/无 VM/冷包）；独立 phase 复核 fold 交 IF-AGE-GFX-A2R | AC-10 | [ ] |
| T-05 | work | T-03/T-04 | AG023 双后端准确版本/命令/配方/冷包/失败恢复成文并实测 | AC-06 | [ ] |
| T-06 | review | T-05 | 同树独立 final：逐承接 AC/继承表/反例核验；finding 回同员有界复核 | AC-07 | [ ] |
| T-07 | merge | T-06 pass | SD-01 支持域行落 canonical＋落主/归档/导航 | AC-08 | [ ] |
| T-08 | merge | T-07 | 全输出对账/摘要折 Plan/fresh guard 清树/scope 清理/hold 登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-19 / stage: new / PLAN-103 r1 / outcome: **pass（合同可审阅；drafting，开工等栅栏）**。承接 PLAN-058 r8 转移清单五项（继承表逐项对得上原位置）；8 任务/7 AC 行/4 新用例；依赖 1068（drafting 0/10）、981 AC/A2R 域端口、W2/W3 波全部未落地——**blocked before T-01 by design**（等栅栏，不阻塞 PLAN-058 合并）。仅规划：未建树、未跑 Cargo/GPU、未动代码/Spec/价目/文法、未派 agent。next: T-01（栅栏核对，外部落地后）。

## 10. 待澄清事项

W-01：开工栅栏三项（1068/T-04、981 AC 域端口、UI-W2-AC）均未落地——本计划按用户裁定显式等待，不构成任何在途 Plan 的阻塞。W-02：IF-TOPDOWN-TOUCH-ROUTE 的 op 形/预算归属由 981 冻结（本计划 §5.3 只提请求面）；若 981 判定需替代机制，回本计划 needs_replan 有界修订。W-03：A2R 的最终包形态（exe+资源 ZIP，沿 981 B-RUST2D 惯例）与 AG023 消费格在 T-04 冻结命令后回填本表。
