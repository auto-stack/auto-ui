---
plan_id: PLAN-050
status: drafting
feature_name: 生成Vue工程vite配置发射收口（optimizeDeps.include + strictPort）
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 4
total_steps: 5
depends_on: []
supersedes_spec_components: []
new_spec_components: ["docs/specs/build/generated-vite-config.md"]
touched_goals: []
---

# PLAN-050：生成 Vue 工程 vite 配置发射收口（optimizeDeps.include + strictPort）

## 0. 变更摘要

唯一生产 vite 模板 `auto-ui-tools/src/vue.rs::generate_vite_config()` 增发两项：`optimizeDeps.include`（源=最终 package.json dependencies 键集，空集省略键）与 `server.strictPort: true`（恒发射）。两条写入路径（生成期 `write_project_files`、run 期自愈 `run_vue.rs`）同源消费。动机=跨仓消费方 auto-blog BLOG-005 的 F-2 dev 假绿（pac.at npm_deps 动态导入链在 vite dev 冷扫描不可见 → CJS 互操作断裂、作者页白壳），cand5 六轮控制变量已在消费方逐字验证 include 修复全绿（决策稿指针见 §4）。附带 `host` 发射的有界评估（T-05 决策稿），未证实前不发射。

## 1. 目标

- 生成/自愈写入的 `vite.config.ts`：dependencies 键集非空时 `optimizeDeps.include` 必须出现，键集 = 写入该次运行的最终 package.json dependencies 键集（恒 ⊆ installed，零"include 引用未安装包"新失败面）；空集省略 `optimizeDeps` 键。
- 端口被占不再静默改道：`strictPort: true` 发射后，vite 对指定 `AUTO_FRONT_PORT` 端口占用必须启动失败（非零退出、vite 具名报错），不得自增端口、不得跨协议栈静默绑定同端口。
- 消费方指针：auto-blog BLOG-005（其簿记合同）在 owner 交付后冷缓存重跑 `vue-dev-rust` 门至绿。该验证属消费方计划，不在本仓执行。

非目标：不改 build/rollup 输出配置；不改 npm 安装/审批（.npmrc/pnpm-workspace）语义；不动 compiler 侧 test-only 旧副本 `auto-ui-compiler/src/ui_gen/vue/project_files.rs:73`（生产零调用方，已漂移为观察项记录于 §10）；除 T-05 评估结论支持外不做 `host` 发射；不改 `AUTO_FRONT_PORT` 选端口机制。

## 2. 架构方案

- **唯一生产模板**：`crates/auto-ui-tools/src/vue.rs:1012` `pub fn generate_vite_config()`，调用方仅两处：
  1. **生成期**：`vue.rs:2554`（`write_project_files` 内）。作用域内 `extra_deps`（`:2520` 遮蔽后）= Plan-482 引擎推导后的最终依赖集，与 `:2523` 写入 package.json 的集合逐字同源 → 直接作 include 源。
  2. **run 期自愈**：`run_vue.rs:283`（项目已存在，增量路径不重写生成物的自愈补偿）。磁盘 `package.json` 是唯一真源（不受手改/历史项目漂移影响）：serde_json 解析其 `dependencies` 键作 include 源。`serde_json`（含 `preserve_order`）已在 auto-ui-tools 依赖（Cargo.toml:37）。
- 不选 `VueProject.npm_deps`（vue.rs:3560）作源：该集缺 Plan-482 引擎推导且不反映 package.json 手改，存在 include ⊄ installed 漂移面。
- **strictPort**：写入 `server` 块（`port: Number(process.env.AUTO_FRONT_PORT || 3000)` 之后），恒发射。语义变化（有意）：多实例第二实例从"静默自增 3001（可能撞后端端口呈 split-brain，E-4b 实录）"变为"显式启动失败"。与消费方 BLOG-005 B-01 故障合同（端口冲突=失败非零可诊断）一致；选端口机制（env 注入）不变。
- **host（E-4a，同端口跨栈静默绑定）**：strictPort 只锁端口不锁栈。`host` 固定发射需评估其对 `localhost` 双栈解析、tauri.rs:374-377 消费、既有就绪判据的影响——T-05 有界评估出决策稿，未证实前不发射。
- 跨仓证据指针：auto-blog `docs/plans/evidence/005/startup-decision.md` §4/§6c（cand5：根声明 mermaid/lowlight/katex + include 三者 → dev 全绿；同稿 preview 对照全绿；原生轨不受影响）。本仓不重复该实验。

## 3. 技术栈

Rust（auto-ui-tools）；serde_json（已在依赖）；vite 配置模板字符串（TS 字面量输出，包名键经安全序列化）；测试=cargo test 固定组（§6），不加新工具链。

## 4. 需求分析与背景调查

- 观察事实（本仓 master `27b9c74a`，2026-11-04）：
  - `generate_vite_config` 生产调用方仅 vue.rs:2554 与 run_vue.rs:283（全仓 grep 具名）；无既有单测断言 tools 模板内容（grep 零命中）。
  - compiler 副本 `project_files.rs:73` 唯一调用方是其自身单测（ui_gen/vue/mod.rs:4702），模板已漂移（无 server 块/proxy/base）——生产死代码。
  - `AUTO_FRONT_PORT` 无 crate 内 `set_var` 写入方（仅 tauri.rs:377 读取；注入来自外部 CLI/用户），默认 3000。
  - `write_project_files:2503-2520`：`extra_deps_owned` 完成 Plan-482 `@autodown/engine` 同级推导后于 `:2520` 遮蔽 `extra_deps`，`:2523` package.json 与 `:2554` vite config 共享同一集合。
  - run 自愈块（run_vue.rs:272-289 注释）：vite.config/tailwind/index.css 自愈为确定性模板重写，`exists()` 门内，写失败=警告不中断。
  - `docs/specs/` 全树 grep `vite` 零命中——生成工程 vite 配置合同未规范，由 SD-01 新增。
- **授权**：用户 2026-11-04 在 auto-blog BLOG-005 goal round 3 显式拍板路线①——授权在 auto-ui 仓按其自身 Plan/worktree 规约新建本最小计划并交付，随后消费方恢复 work。本计划即该授权的直接产物；worktree 惯例位 `.wt/ui-050/`。
- 冻结观察基线：

| 源文件 | SHA256（完整值） |
|---|---|
| `crates/auto-ui-tools/src/vue.rs` | `7842103ef1392a46fe704c2dc678bcbd1462617e1665cc3f0416e38da055f406` |
| `crates/auto-ui-tools/src/run_vue.rs` | `0c325ac1272be01c01ea489d64bdaa3584a8d36ff0df80e46b5fe1572821aa84` |
| `crates/auto-ui-compiler/src/ui_gen/vue/project_files.rs` | work 首日实测记录（T-01 前置核对） |

## 5. 详细设计

1. **T-01 模板发射（生成路径）**：`generate_vite_config` 签名改为接收 `npm_dep_keys: &[String]`；非空时在 `export default defineConfig({` 后注入 `optimizeDeps: { include: [<键序列>] }`（键为合法 TS 字符串字面量，经 `serde_json::to_string` 序列化防注入）；空集省略整个 `optimizeDeps` 键。`server` 块 `port:` 行后加 `strictPort: true`（带注释：端口占用=显式失败，禁止静默自增/跨栈改道）。`write_project_files:2554` 传 `extra_deps` 键集。
2. **T-02 自愈路径同源**：`run_vue.rs` 自愈块读 `vite.config.ts` 同目录 `package.json` → `dependencies` 键集传入。解析失败（损坏 JSON/缺 dependencies 字段）：保留原文件 + 具名警告（与既有自愈写失败同级，不中断 run）；`package.json` 缺失：跳过（既有 `exists()` 门已保证，防御性同名处理）。
3. **T-03 单测**：vue.rs 测试模块新增——(a) 非空键集发射含全部键且字面量合法；(b) 空集省略 `optimizeDeps`；(c) `strictPort: true` 恒存在；(d) 键序确定性（同输入两次输出逐字一致）。run_vue 侧 package.json 解析：正常/损坏/缺字段三分支。
4. **T-05 host 有界评估**：决策稿（scratch `D:/autostack/plan-artifacts/plan-050/`）实测 strictPort 发射后 127.0.0.1 被占场景 vite 实际绑定行为（是否仍 `::1` 同端口静默就绪）；评估 `host: '127.0.0.1'` 固定发射对 tauri.rs 消费方与 `localhost` 就绪判据的影响；结论=发射或不发射的具名决定 + 证据，写入 §9。
5. **T-04 回归与组收口**：`cargo t -p auto-ui-tools` 全组零回归 + 新增用例全绿；`cargo t -p auto-ui-compiler project_files` 确认旧副本未动不漂移。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/build/generated-vite-config.md | 新增：生成/自愈 vite.config 发射合同——include 键集≡写入时 package.json dependencies 键集（空集省略）；strictPort 恒真（端口占用=具名失败）；host 发射状态按 T-05 结论记录边界 | dev 冷启动预打包动态导入链 + 端口故障显式化；`docs/specs/` 现无任何 vite 覆盖 | AC-01/02/03 |

## 6. 测试设计

最终 review 由独立上下文在同一 worktree 核对发射契约与组回归；不新建 agent/会话，不跑无关全仓 Cargo 批次。

| Case ID | AC / invariant | 输入/正式入口 | 区分性 oracle / 预期 | 边界 |
|---|---|---|---|---|
| B-01 | AC-01 | 含 npm_deps（mermaid/lowlight/katex 实值）调用模板函数 | 发射 `optimizeDeps.include` 含全部三键，TS 字面量合法 | 键序确定性 |
| B-02 | AC-01 | 空键集 | `optimizeDeps` 键整体省略（standalone 面 diff 仅 strictPort） | 不发 `include: []` |
| B-03 | AC-02 | 任意输入 | `server.strictPort: true` 恒存在 | 与 port 行共存 |
| B-04 | AC-01 | run 自愈：package.json 损坏 / 缺 dependencies / 正常 | 损坏与缺字段=警告保留原文件；正常=重写含 include | 不中断 run |
| B-05 | AC-02 | `cargo t -p auto-ui-tools` 全组 | 零新增失败（对照 §4 冻结基线） | 既有组 oracle 不改 |

消费方区分性验证（冷缓存 F-2 复现→绿）属 auto-blog BLOG-005 簿记（其 vue-dev-rust 门），本计划以 §10 指针承接，不在本仓复跑。

## 7. 验收标准

- AC-01（owner_stage=work）：两条写入路径发射的 `optimizeDeps.include` 键集≡该次写入的 package.json dependencies 键集；空集省略键；B-01/B-02/B-04 过。
- AC-02（owner_stage=work）：`strictPort: true` 恒发射；`cargo t -p auto-ui-tools` 全组零回归（B-03/B-05 过）。
- AC-03（owner_stage=work）：T-05 host 决策稿有具名结论（发射或证据化不发射）。
- AC-04（owner_stage=review）：独立上下文核对发射契约、组回归与 SD-01。
- AC-05（owner_stage=merge）：SD-01 spec 落地；消费指针（BLOG-005 恢复验证）在其簿记留痕。

## 8. 执行步骤

- [x] T-01（owner_stage=work；依赖无；AC-01）：模板签名与 include/strictPort 发射 + write_project_files 接线；B-01/B-02/B-03 单测。[✅ 已完成 2026-11-04] `generate_vite_config(npm_dep_keys: &[String])`：非空键集发射 `optimizeDeps{include:[…]}`（serde_json 转义），空集整体省略；`host: '127.0.0.1'`+`strictPort: true` 恒发射；write_project_files 传 `extra_deps`（:2520 遮蔽集=package.json 同源）。
- [x] T-02（owner_stage=work；依赖T-01；AC-01）：run_vue.rs 自愈路径 package.json 解析接线 + B-04 三分支单测。[✅ 已完成 2026-11-04] `package_json_dep_keys()` 读磁盘 dependencies 键集；解析失败/缺字段=保留原 config+具名警告不中断 run；`Some` 才重写。
- [x] T-03（owner_stage=work；依赖T-01/02；AC-02）：`cargo t -p auto-ui-tools` 全组回归（B-05）+ `-p auto-ui-compiler project_files` 未漂移确认。[✅ 已完成 2026-11-04] 新增 4 单测全绿；全组 `-p auto-ui-tools --lib` 203/203 绿（5 基线红：display_reader_v01_corpus、gallery_demo_row_routable、try_full_parse_018_api、shell_pack_lib_freshness、plan_662_gallery_apps_dir_layouts——均经 stash 回 base 实证同败，与本改动无关）；`-p auto-ui-compiler` test_generate_project_files 1/1 绿（旧副本未动）。
- [ ] T-04（owner_stage=review；依赖T-03/05；AC-04）：同树独立上下文最终复审。
- [x] T-05（owner_stage=work；依赖T-01；AC-03）：host 有界评估决策稿 + §9 具名结论。[✅ 已完成 2026-11-04] 结论=**发射 `host: '127.0.0.1'`**（并入 T-01）。探针 4 组（vite 5.4.21 实测，plan-artifacts/plan-050/host-probe/DECISION.md）：strictPort 单独不闭 E-4a——127.0.0.1 被占仍静默绑 ::1 同端口报 ready（curl ::1→200 实证）；host 固定后占用=具名失败 EXIT=1（CLI 与 config 形双验）、空闲=localhost 经双栈回落 200、tauri webview（localhost: URL）兼容、Local: 行由歧义变明确。

## 9. 复审记录

- stage: new | plan_id: PLAN-050 | plan_revision: r1 | outcome: pass（授权已在 §4 记录：消费方用户 2026-11-04 拍板路线①） | next: work（T-01/T-02 可立即开工；T-05 评估可与 T-01 并行）。

stage: work | plan_id: PLAN-050 | plan_revision: r1 | outcome: pass（T-01/T-02/T-03/T-05 完成，execution_done 待 T-04 复审） | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | base_commit: da252a98 | code_commit: 1a9131fc | dependency_revisions: auto-lang f6b0f918d（master，detached 同级）、auto-web 3de5524（master，detached 同级）、auto-down 7c4d5cb（HEAD，detached 同级） | task_ids: T-01、T-02、T-03、T-05（T-04=review 所有） | evidence: 新增 4 单测全绿（vite_config include/omit/deterministic+quoting、package_json_dep_keys 三态）；全组 nextest `-p auto-ui-tools --lib` 203/203 绿、5 基线红 stash 回 base 实证同败；`-p auto-ui-compiler` test_generate_project_files 1/1 绿；host 探针 4 组（DECISION.md：strictPort 单独 E-4a 复现→host 固定具名失败/localhost 回落兼容）；SD-01 spec 草案在树（docs/specs/build/generated-vite-config.md，merge 时发布） | blockers: 无 | next_responsible_role: review（T-04 同树独立上下文最终复审；统一回归门=复审/merge 按仓档位）

## 10. 待澄清事项

- host（E-4a）发射与否=T-05 决策稿产出，非用户待决；证据支持则随 SD-01 边界记录，不支持则具名不发射并留消费方自愈检测（run 期解析实际绑定 URL）说明。
- strictPort 的多实例语义变化（第二实例显式失败）为**有意合同**（§2），与消费方 BLOG-005 B-01 故障门一致，不视为回归；若复审发现本仓存在依赖静默自增的测试/文档，按零回归原则报 work 处置。
- 观察项（不在本计划处置）：compiler `project_files.rs:73` test-only 旧副本与生产模板已漂移（无 server 块）。
