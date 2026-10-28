# PLAN-830 T-01 — Surface/Portal 组合原型来源基线（2026-10-22）

本文件是 PLAN-830 的开工来源核对记录（T-01 / AC-01）。它证明本实施树从
G829 授权的冻结来源起步，而不是 master 空原型路径、已退役 829 树或任何
带 WIP 的树。

## 1. 冻结来源核实（G829）

| 项 | 值 | 核实方式 |
|---|---|---|
| 授权集成分支 | `autographics-dev`（本仓本地分支） | `git branch -a` |
| 分支 HEAD = 计划冻结源 | `ab56c32ec4cb28e01fbcca74e9861164ffad43f0` | `git rev-parse autographics-dev`；与计划 §2 G829 记载一致 |
| 829 复审/门禁冻结代码 | `ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74` | `git merge-base --is-ancestor ddbde3e62 ab56c32ec` → 通过（是分支祖先） |
| AGP 子树一致性 | `experimental/autographics-prototype/` 在 ab56c32ec 与 ddbde3e62 之间**逐字节一致** | `git ls-tree -r` 双侧 diff 为空 |
| 829 门禁收据 | 117/117 测试 + 10/10 探针 + consumer 重定位通过 | 前序收据（829 归档 + merge-t11 gates.md），本轮不重复执行（计划 §2 明确） |

## 2. 实施树与分支

| 项 | 值 |
|---|---|
| 实施树 | `D:/autostack/.wt/lang-830/auto-lang` |
| 分支 | `plan-830-dev` |
| 基提交 | `ab56c32ec4cb28e01fbcca74e9861164ffad43f0`（即冻结源本身，非 master） |
| 树状态 | 建树时 clean（`git status --porcelain` 空） |
| 主检出预检 | master 有 docs/ 簿记类未提交改动、无 `crates/**`/`test/**` 代码路径脏改（2026-10-22 核）；master 零 WIP 代码规则未被违反 |

建树命令：`git worktree add D:/autostack/.wt/lang-830/auto-lang -b plan-830-dev ab56c32ec…`。
按计划 §8：开工前已读取**主检出**的 PLAN-830 r3 文本（worktree 基面不含该
Plan 文件的 r3 修订）；合同以主检出 r3 为准，本基面不用旧 Plan 文本覆盖。

## 3. 依赖与锁

- AGP `Cargo.toml` 无任何 `[patch]` 段（`grep -c "\[patch"` = 0）——stock 依赖确认，
  未恢复 829 早期 vendored patch。
- `Cargo.lock`（独立 workspace，根仓锁不受影响）关键锁定：`iced 0.14.0`、
  `iced_wgpu 0.14.0`、`iced_widget 0.14.2`（stock）、`wgpu 27.0.1`。
- `Cargo.lock` sha256：`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`。
- 不追新升级（计划 §3）；后续提交若改动 AGP 依赖须保持 `--locked` 可构建。

## 4. 当前 Spec 状态（provisional 影响面核实）

- 本 worktree 基面（autographics-dev）**不含** `docs/specs/auto-graphics/`——
  829 的正式 Spec 沉淀在 master 侧（`project/prototype-profile/integration-contract/plans`）。
  计划 frontmatter 的 SD-03/SD-04 "modify" 目标指 master 上的 current Spec；
  SD-01/SD-02 为新增。
- 处置：work 阶段在 worktree 内 `docs/specs/auto-graphics/` 起草
  `surface-composition.md` 与 `nested-input.md` 草稿（可prepared，不发布）；
  正式沉淀/合并归属 T-10 merge 阶段（合回 autographics-dev 与 master 文档回合
  按其收据执行）。这与技能规则一致：canonical Spec 编辑可在 worktree 准备，
  但不作为独立需求来源发布。

## 5. 写面边界（开工前重申）

- 只改 `experimental/autographics-prototype/**`（AGP 自有 workspace）与本
  worktree 内的 docs/specs 草稿；不写 `crates/auto-ui-*`、`crates/auto-lang`、
  根 manifest/锁、Man/CLI 或其他仓。
- 单一 Device/Queue：宿主（本 Plan 为独立 winit 宿主，T-02 定）持有
  `GpuDevice`；iced 内容 adapter 经公开 `iced_wgpu::Engine::new(adapter,
  device, queue, …, Shell::headless())` 取受控借用，不创建第二设备。
- 不复制 829 `iced_spike.rs` 的顶层排布当嵌套系统；该文件与其 case/测试
  原样保留（AC-07 兼容面）。

## 6. 后续任务来源锚点

- T-02 可行性结论将写入 `research/surface-feasibility.md`（新）。
- 829 冻结 README 的两游戏/probe 命令继续适用（V-09 复跑时以其为准）。
- 829 已知接缝：`renderer/target.rs` 的 `OffscreenTarget` usage 仅
  RENDER_ATTACHMENT|COPY_SRC——采样 usage 由 T-04 按新合同增补，兼容旧调用。
