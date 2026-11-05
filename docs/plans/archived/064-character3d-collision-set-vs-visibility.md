---
plan_id: PLAN-064
status: archived
feature_name: character3d 隐形 solid 阻挡修复（可见行语义不变）
author: [agent]
created_at: 2026-11-05T14:55:00Z
updated_at: 2026-11-05T16:21:00Z
plan_revision: 2
current_step: 5
total_steps: 5
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-064：character3d 隐形 solid 阻挡修复（可见行语义不变）

## 0. 变更摘要

PLAN-043 的 T-04 把 `ag_char3_move` 的滑扫从 `visible_only=0` 改为
`visible_only=1`，使**所有**不可见文档行都不再阻挡角色。platformer-3d 的
三块平台板是 `solid: true` + `visible: false` 的 cylinder（这正是 035 契约
里“唯一碰撞原语”的写法），因此角色直接穿过平台、落到底面：

- 019 候选 `fc82061c`（043 之前）：platformer-3d oracle 11/11 绿；
- 048 交付 `cd1e161a` 起红；049 `b7f01267`、054 `2571556e` 同样红；
- 2026-11-05 git bisect first bad = `3842d93e`（043 T-04）。

本计划修正 043 的可见性近似：**可见行的 043 语义保持不变**（滑扫仍跳过
不可见行，parked 模板/隐藏表现行不阻挡）；`ag_char3_solid` 注册的 solid 行
在滑扫期间被临时视为可见（effective 置 1、返回前恢复），因此
`solid:true + visible:false` 的行（platformer-3d 的平台板）重新阻挡。
恢复 019 的 M2-05，同时保留 043 的“隐藏模板不阻挡”控制；不引入 scene
schema/生成器/host 改动。

## 1. 目标

- G-01：可见行的 043 语义不变（不可见行不阻挡）；`ag_char3_solid` 注册的
  solid 行在滑扫期间额外阻挡，**与 visible 无关**（含 invisible solid 与
  父级隐藏 solid）。未注册的**可见**行仍按 043 阻挡（可见性代理保留，
  不把可见 model 变成非碰撞）。
- G-02：platformer-3d 在修复后的候选上 oracle 11/11；043 的
  `g14-model-actor` hidden-template 控制 8/8。
- G-03：`ag_move_and_slide` 的 v5 公共面保持冻结；character3d 单元被
  Camera3D/Objects3D/Triggers3D 等 profile 共用，其装配行为随之变化
  （可见性语义不变，仅 invisible solid 行为修正）；这些 profile 的测试
  必须全绿。
- G-04：不改产物字节、scene schema、生成器或 host；all products 保持不变
  （AG021 的 M2-05 无需重新生成）。

**非目标**：不改 043 的 solid 预算/扫描口径；不改 generator profile；不改
AG 内容仓产品（除 T-01 选择的显式标记迁移）。

## 2. 架构方案

现状（读码确认）：

- `experimental/autographics-auto-driver/api/character3d_v1.at`
  - `ag_char3_solid(stable_id)`：把 `solid: true` 的文档 id 注册进
    `_ag_char3_solids`（≤128，重复/角色自身拒绝），供支撑/落地扫描使用。
  - `ag_char3_move(dx,dy,dz,clearance)` 调
    `ag_move_and_slide(_ag_char3_node, visible_only, …)`（043 起传 1）。
- `api/behavior_motion_v5.at`
  - `ag_move_and_slide(node, visible_only, …)` 对同维度、非 group 的全部
    `identities` 行做三轴 sweep；`_ag_motion_three` 的候选过滤为
    `if visible_only==1 && effective[n]==0 {continue}`（第 82 行）。
  - `effective` 是“有效可见性”（`flags[i]` + 父级传播），与 solid 无关。
- 043 的测试 `tests/node3d_model_objects_flow.rs` 断言隐藏模板与 live 实例
  不改变行走轨迹；platformer-3d 的 `solid: true` + `visible: false` 平台板
  则必须阻挡。

结论（rev 1 的旧表述，已被 rev 2 的 A2 取代）：`visible` 被两种语义共用
（“是否渲染”与“是否碰撞”），旧稿据此提出“碰撞 = 注册 solid 集合”。rev 2
的当前契约见 G-01/SD-01/§5：**可见行保持 043 语义；注册 solid 行在滑扫期
被临时置为 effective=1、返回前恢复，因此隐形/父级隐藏 solid 仍阻挡；可见
未注册行仍按可见性阻挡**。

rev 1 候选（历史，已否决，仅留档）：
- A（collision set = `_ag_char3_solids`）：探针不稳定、跨 profile 风险高，
  未采用（见 §9 的 F-06 更正；§9 的 globals 旧述为 rev 1 历史）；未声称其行为已通过。
- B（scene `parked` + host 零 box）：runtime-object 的 spawn/move 走
  `GeometryUnavailable`；且需 schema/产物迁移；否决。
- C（`ag_move_and_slide` mode 2）：`behavior_motion_scan_v5.at` 的
  visible_only∈{0,1} 校验拒绝；否决。

当前实现为 A2（rev 2）；详见 §5 与 §9。

## 3. 技术栈

Rust 1.98 / auto-ui `experimental/autographics-auto-driver`（API `.at` 受信
单元 + host）+ `experimental/autographics-prototype`（AGP 场景/物理）；
Cargo test 单线程；AG 侧 Python verifier。

## 4. 需求分析与背景调查

- 事实（2026-11-05 实测）：`3842d93e` 起 platformer-3d 5/11 红（hero rest
  y=-0.018 vs 0.5）；`fc82061c` 绿；identity/冷再生字节一致，断点只在角色
  滑扫的可见性过滤。
- 探针（scratch `.wt/ag-021/auto-ui-fix`@2571556e，未提交）：
  - 回退 `visible_only=0`：platformer-3d 11/11（churn 单跑复绿），但 043
    hidden-template 控制红（hero 被隐藏模板卡在 x=3.0）。
  - 仅跳过 invisible model（`kinds[n]==6`）：4/11，更差（隐藏模板与
    invisible 活模型不能按 kind 区分）。
- 规范：`docs/specs/auto-graphics/node3d-character-and-runtime-camera.md`
  §“solid: true（cylinder3d 叶 prop）：角色 solid”与 043 的 SD-02 段
  “滑扫跳过不可见文档行”直接冲突；本计划修正该段。
- 授权：用户在 2026-11-05 明确授权“新建 auto-ui 修复计划”并跨仓修复；
  允许修改本仓 master 与 experimental 代码，不修改 AG 内容仓。
- 影响面：任何“invisible solid”内容（platformer-3d 平台板；潜在其它 3D
  内容）恢复碰撞；任何“隐藏 model 模板”保持不阻挡。

## 5. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/node3d-character-and-runtime-camera.md | 前：`ag_char3_move` 滑扫跳过不可见行（visible_only=1），“全部已交付阻挡行均可见”；后：可见行语义不变；`ag_char3_solid` 注册的 solid 行在滑扫期被临时置为 effective=1（返回前恢复），因此 invisible/父级隐藏 solid 仍阻挡；可见的非注册行仍按 043 阻挡 | 043 的近似的反例是 platformer-3d 的 invisible solid deck；可见性不能代理 solidity | AC-01..AC-05 |

本计划不新增/不废止 spec 文档（supersedes/new components 为空）；只 modify SD-01 目标。

### 实现要点

1. `ag_char3_move` 保持 `ag_move_and_slide(_ag_char3_node, 1, ...)`（043 的
   可见行语义）。调用前遍历 `_ag_char3_solids`，把每个 solid 节点的
   `effective[node]` 保存在局部数组并置 1；调用后按同序恢复。恢复发生在
   所有正常返回（含 `token<=0`）之前；若 native 层 trap，下一次 tick 的
   `derive()` 会从 flags 重建 effective（已记录为残余风险）。
2. 空 `_ag_char3_solids` → 不写 effective；仅两个 ≤128 次循环，不改变
   token/charge/预算口径。`_ag_char3_scan_down` 仍按注册 solid 列表工作。
3. 不改 `ag_move_and_slide`、scene schema、生成器、host 或 products；
   character3d 仍使用 `autographics.character3d/1`（marker 102/profile 13）。
4. 测试必须证明：invisible solid 仍阻挡/支撑；隐藏非 solid model 不阻挡；
   可见非注册 cylinder 仍阻挡（A2 语义）；父级隐藏 solid 阻挡（默认）。

### 测试设计

- 单元/集成：`cargo test --test node3d_runtime_flow`、`character3d_*`、
  `camera_host`、`events_flow`、`node3d_model_objects_flow`（单线程）。
- 新负控（T-03，`tests/character3d_api.rs`）：`ag_set_visible` 真隐藏的
  solid 仍阻挡/支撑；隐藏非 solid model 不阻挡；可见未注册 cylinder 仍
  阻挡；父级隐藏 solid 阻挡（默认，待补测或具名 residual）。
- AG 端：`python games/platformer-3d/tools/verify_game3d.py --controls`
  11/11；`python tools/verify_m2_release.py --manifest …`。

## 6. 测试设计（角色/入口）

- 实现者：work（T-02）。
- 内部复核：work 自测 + T-03 回归。
- 最终复核：独立 review（不同上下文）。
- 合并：auto-plan-merge。
- 证据目录：`D:/autostack/plan-artifacts/plan-064/`（原始日志）；计划 §9 只
  记收据。

## 7. 验收标准

- AC-01：修复候选上 platformer-3d oracle 11/11（允许 churn 单测复绿并记录）。
- AC-02：`node3d_model_objects_flow` 8/8（043 hidden-template 控制不回归）。
- AC-03：driver 既有角色/运动/相机相关测试全绿（node3d_runtime_flow、
  character3d api/host/samples、camera_host、events_flow、basic、bindings、
  audio_hud）。
- AC-04：新增/修正负控通过：隐藏 solid 阻挡与支撑（`ag_set_visible` 真
  隐藏）；隐藏非 solid model 不阻挡；可见未注册 cylinder 仍阻挡（A2）。
- AC-05：products/scene schema/生成器/host 不变；`ag_move_and_slide` v5 公共
  面字节不变；character3d 单元注册的 plan874/879 identity pin 更新并自检。
- AC-06：新候选交付（merge、独立 final、canonical 已发布）；AG021 T-06
  可在其上重钉并重跑 platformer-3d + verify_m2_release。

## 8. 执行步骤

| ID | 任务 | 依赖 | 文件/符号 | 产出 | AC |
|---|---|---|---|---|---|
| T-01 | 有界调查与决策：A/B/C 的实测对比（platformer-3d 11 + 043 8），记录决策工件 | — | `api/character3d_v1.at`、`api/behavior_motion_v5.at`、`runtime_objects.rs` | §9 决策记录 + 探针日志 | AC-01..05 的前置 |
| T-02 | 实现 A2（滑扫期 reveal 注册 solid + 恢复）；保持 v5 公共面与产物 | T-01 | `character3d_v1.at` | 代码 + 单测 | AC-01..05 |
| T-03 | 回归与负控：platformer-3d、043 8、driver 套装、隐藏 solid/隐藏 model/可见未注册 row 负控 | T-02 | `tests/character3d_api.rs` | 全绿日志 | AC-01..04 |
| T-04 | SD-01 修订（character3d 规范段）+ 043 说明的历史化；交付文档 | T-03 | `docs/specs/auto-graphics/node3d-character-and-runtime-camera.md` | SD 落地 | AC-05 |
| T-05 | 收口：verify_plan064 控制、锁/身份、独立 final、merge/canonical | T-04 | engine-locks / docs | 交付收据 | AC-06 |

## 9. 复审记录

2026-11-05 / stage: new | plan_id: PLAN-064 | plan_revision: 1 | outcome: pass（进入 work；授权已记录） | tasks: T-01..T-05 | next: work（T-01 决策探针）

2026-11-05 / stage: work | plan_id: PLAN-064 | plan_revision: 1 | outcome: T-01 pass（决策修正为 A2） | task_ids: T-01 | evidence:
- A 案（v5 单元内 `_ag_solid_set` + 跨单元全局）被证伪：Auto API 单元间不共享 `var` 全局。`_ag_solid_only=1` + `ag_move_and_slide(...,0,...)` 时 platformer-3d 11/11（等价 pre-043），换成 `...,1,...` 立即回到 0.5667 的旧路径 → 单元内全局没有传到 v5 的过滤分支。
- visible_only=0 回退：platformer-3d 11/11，043 g14 红（hero 卡 x=3.0）——必须显式区分 parked 模板。
- 生成器已有 `template_rows`/`TemplateRowIn` 显式模板行（`lower_components3d_events.rs:45`），可在构建 scene 时给模板行写 `parked: true`；host 读取后把 parked 行的 spatial box 置零（不进入碰撞/查询），`ag_char3_move` 回退 visible_only=0。
- 选 B：`Object.parked`（serde default false）+ 生成器只对 template_rows 写 true + host 置零 spatial box + character3d 回退 0；platformer-3d 无 object3d 模板，产物字节不变；g14 等含模板的产物重生成。
| blockers: none（B 的实现在 T-02） | next: work（T-02 实现 B + T-03 回归）

2026-11-05 / stage: work | plan_id: PLAN-064 | plan_revision: 1 | outcome: T-02/T-03 pass（A2） | task_ids: T-02, T-03 | evidence:
- B 案（scene `parked` 列表 + host 零 spatial box）被实测否决：parked 行零 box 会让 runtime-object 的 spawn/move 走 `GeometryUnavailable`（g14 variant node=1 code=9），且要动 schema/产物；回退。
- 采用 A2（只改 `api/character3d_v1.at`）：`ag_char3_move` 仍以 `visible_only=1` 滑扫；调用前把 `_ag_char3_solids` 的 `effective[node]` 暂置 1、调用后恢复。效果：隐藏 parked 模板/隐藏表现行仍被跳过；隐形 solid（platformer-3d 的 `solid:true`+`visible:false` 平台板）在滑扫中重新阻挡，渲染仍按文档可见性（恢复在返回前完成）。
- 实测（064 worktree，产品字节未改）：platformer-3d oracle 11/11；043 `node3d_model_objects_flow` 8/8；driver 套装运行中。
- 产物/生成器/host/schema 均未改；只改 character3d 受信单元。
- driver 套装（单线程）：node3d_runtime_flow 7/7、basic 6/6、events 8/8、bindings 8/8、audio_hud 9/9、character3d api 16/16、host 9/9、samples 1/1、camera api 12/12、host 12/12、motion api 4/4、host 4/4、spatial api 4/4、host 3/3 → 首轮（api 16）103 例全绿；新增 2 例后 105 例（见下条）。
| blockers: none | next: work（T-04 SD-01 已改；T-05 交付）

2026-11-05 / stage: work | plan_id: PLAN-064 | plan_revision: 1 | outcome: T-04 pass | task_ids: T-04 | evidence:
- SD-01 已落地：`docs/specs/auto-graphics/node3d-character-and-runtime-camera.md` 的 043 段改为“跳过不可见行 + solid 行滑扫期临时可视（effective 置 1、返回前恢复）”，并明确 043 的“阻挡行均可见”前提被 platformer-3d 隐形平台板证伪。
- 观察（非本计划改动）：`tools/verify_plan043.py --controls` 在 master 上 cold-regen g12-model scene 字节漂移（生成器未改）；用 pristine master 基线对照确认归属，不属 064 的 character3d 改动。
| blockers: none | next: work（T-05 交付：merge/收据）→ review

2026-11-05 / stage: review | plan_id: PLAN-064 | plan_revision: 2 | outcome: pass（实现/规范/文本；T-05/AC-06 merge-owned） | reviewed_commit: 95b254d2 | reviewer: independent context | AC: AC-01..05 pass；AC-06 pending merge | evidence: 独立 review 两轮（needs_fix → 修复 → pass），实跑 character3d_api 18/18、node3d_model_objects_flow 8/8、platformer 11/11、verify_plan043 PASS、105 例套装、identity pin MATCH；`.gitattributes` drift 为 pre-existing（master-base 复现） | next: merge


2026-11-05 / stage: work | plan_id: PLAN-064 | plan_revision: 2 | outcome: review needs_fix 修复完成 | task_ids: T-02, T-03, T-04 | evidence:
- F-01（负控）：`tests/character3d_api.rs` 的 hidden_solid_still_collides_and_supports 改用 `ag_set_visible(1,0)/(2,0)` 真隐藏（原 `_ag_visibility` 是 append 的假控制）；新增 visible_unregistered_cylinder_still_blocks（A2 第三款）与 hidden_non_solid_model_does_not_block；character3d_api 18/18 绿（16+2）。
- F-02（契约）：本修订把 G-01/SD-01/§5/AC-04 改为 A2 语义（可见行不变 + 注册 solid 额外阻挡），不再声称“碰撞集合=仅 solid”；候选 A/B 历史见 §2/§9。
- F-03（identity）：plan874/879 的 character3d_v1.at pin 更新为 sha256 5e24af1c…、blob 13c07b05…；两 verifier 仍先撞 master 既有 `.gitattributes` drift（043 已记载），用独立 sha 自检记录 character3d pin 一致。
- F-04（spec/头注释）：SD-01 与 `api/character3d_v1.at` 头注释改为“注册 solid 滑扫期临时可视”，删除“可见性不再代理 solidity”的过强表述；可见未注册行仍按 043 阻挡。
- F-05（收据）：首轮（api 16）103 例（非 109），新增 2 例后 105 例；platformer 11/11 有一次全绿，另一轮 churn 因 audio 设备 `The requested device is no longer available` 10/11（行为/落地全绿，环境项）——收据如实记录两轮。
- F-06（理由）：更正 T-01 记录——API 单元是拼接共享的，A 案被否决是探针不稳定/跨 profile 风险，不是“全局不共享”。
- F-07（范围）：G-03/§5 明示 character3d 被 Camera3D/Objects3D/Triggers3D 共用，装配行为随之变化；products 不变。
| blockers: none | next: work 收尾（重跑 green 日志）→ review 复审

2026-11-05 / stage: work | plan_id: PLAN-064 | plan_revision: 2 | outcome: review-fix 证据补齐 | task_ids: T-03, T-05 | evidence:
- 绿日志（D:/autostack/plan-artifacts/plan-064/logs/）：platformer.log 11/11；g14.log 8/8；character3d_api.log 18/18；verify_plan043.log PASS；identity-character3d.log 两 pin MATCH。
- driver 套装：105 例（api 18 计）全绿；`node3d_audio_hud_flow` 首轮 1 红 = `audio_fault: The requested device is no longer available`（环境音频设备，非碰撞），单测复跑 9/9 绿（node3d_audio_hud_flow-rerun.log）。
- verify_plan874/879 在 pristine master（.wt/ag-064/master-base）同样先撞 `.gitattributes` drift（pre-existing，043 已记载）；character3d pin 自检 MATCH。
| blockers: none | next: review 复审（F-01..F-07）

## 9b. 合并前检查（T-05）

- merge-owned：merge 064 → auto-ui master，记录交付 commit；AG021 重钉该
  commit 后重跑 platformer-3d + verify_m2_release。
- `.gitattributes` drift（874/879）属 pre-existing（master-base 复现），merge
  收据中具名转交，不计 064 失败。

## 10. 待澄清事项

- 无阻塞性澄清。A/B/C 的历史决策见 §2/§9，当前实现为 A2（rev 2）。

### 具名 residual（owner=work；关闭条件）

- 父级隐藏 solid：实现按每节点 `effective=1` 覆盖，父级可见性不影响；已实现
  但未单测。关闭=补 “hidden parent group + solid child” 用例，或具名接受。
- `ag_move_and_slide` 内部 trap 跳过恢复：无 host 中途读取，下一 tick 的
  derive 从 flags 自愈；风险已记录在 §5。
- `_ag_char3_solids` 不在 `_ag_reset` 清空：当前每次 Run 新建 Session；若宿主
  复用 Session 跨文档，effective 写可能越界（低）。关闭=复用场景负控或宿主
  生命周期说明。
- 128 注册 solid 的预算路径未跑：每次 move 多两个 ≤128 循环；127-solid 目前
  在 INIT 被拒。关闭=127/128 边界探针。

2026-11-05 / stage: merge | plan_id: PLAN-064 | plan_revision: 2 | outcome: pass | delivery_commit: 9c13cca4 | reviewed_commit: 95b254d2 | rebase_map: 3bceb504→93c77121, 95b254d2→9c13cca4（range-diff fa6833bf..95b254d2 ↔ f9eaca5c..9c13cca4 全等；master 期间前进到 plan-059 的 f9eaca5c） | canonical: docs/specs/auto-graphics/node3d-character-and-runtime-camera.md（sha256 822e3b0de58925f20c1f51019f53e57754a38fa8e17039599bf98118eba9e861，SD-01 A2 段已落 9c13cca4） | checkpoints: prepared/landed（main tip=9c13cca4，ff-only）/canonical_verified（spec on main）| archive: docs/plans/archived/064-character3d-collision-set-vs-visibility.md | cleanup: worktree/branch removed; plan-artifacts/plan-064 folded then deleted; registry refreshed | AC-06: delivery + AG021 重钉待下一阶段（AG021 T-06） | production_artifact: auto-ui 为实验 crate（driver/prototype），无 release 二进制；消费方 AG021 重建二进制，记录为观察项 | batch_regression: auto-lang 专用批量门，对 auto-ui 不适用（N/A）
