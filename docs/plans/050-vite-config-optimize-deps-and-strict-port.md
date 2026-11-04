---
plan_id: PLAN-050
status: drafting
feature_name: 生成Vue工程vite配置发射收口（optimizeDeps.include + strictPort）
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 3
current_step: 4
total_steps: 5
depends_on: []
supersedes_spec_components: []
new_spec_components: ["docs/specs/build/generated-vite-config.md"]
touched_goals: []
---

# PLAN-050：生成 Vue 工程 vite 配置发射收口（optimizeDeps.include + strictPort）

## 0. 变更摘要

唯一生产 vite 模板 `auto-ui-tools/src/vue.rs::generate_vite_config()` 增发两项：`optimizeDeps.include`（源=项目 `npm_deps` 中非 `link:` 注册表键集，两路径同字段、过滤在模板内，空集/全 link: 省略键）与 `server` 加固（`host: '127.0.0.1'` + `strictPort: true`，恒发射）。两条写入路径（生成期 `write_project_files`、run 期自愈 `run_vue.rs`）同源消费。动机=跨仓消费方 auto-blog BLOG-005 的 F-2 dev 假绿（pac.at npm_deps 动态导入链在 vite dev 冷扫描不可见 → CJS 互操作断裂、作者页白壳），cand5 六轮控制变量已在消费方逐字验证 include 修复全绿（决策稿指针见 §4）。附带 `host` 发射的有界评估（T-05 决策稿）：探针实证 strictPort 单独不闭 E-4a，host 固定发射。

## 1. 目标

- 生成/自愈写入的 `vite.config.ts`（r3）：`npm_deps` 解析对中**非 `link:` 注册表键**非空时 `optimizeDeps.include` 必须出现，键集 = 该项目 `VueProject::npm_deps` 中非 `link:` 的注册表键（`link:` 过滤在模板函数内实施，两条写入路径传**同一字段**，发射逐字一致；trio 形态即消费方 cand5 已验证集；键集 ⊆ 实装依赖；`link:` 声明保持 vite 默认源态加载；Plan-482 推导的 engine 同级键仅进 package.json 永不进 include；框架基础依赖不在内——静态可达本就预打包）；过滤集为空省略 `optimizeDeps` 键。
- 端口被占不再静默改道：`strictPort: true` 发射后，vite 对指定 `AUTO_FRONT_PORT` 端口占用必须启动失败（非零退出、vite 具名报错），不得自增端口、不得跨协议栈静默绑定同端口。
- 消费方指针：auto-blog BLOG-005（其簿记合同）在 owner 交付后冷缓存重跑 `vue-dev-rust` 门至绿。该验证属消费方计划，不在本仓执行。

非目标：不改 build/rollup 输出配置；不改 npm 安装/审批（.npmrc/pnpm-workspace）语义；不动 compiler 侧 test-only 旧副本 `auto-ui-compiler/src/ui_gen/vue/project_files.rs:73`（生产零调用方，已漂移为观察项记录于 §10）；除 T-05 评估结论支持外不做 `host` 发射；不改 `AUTO_FRONT_PORT` 选端口机制。

## 2. 架构方案

- **唯一生产模板**：`crates/auto-ui-tools/src/vue.rs:1012` `pub fn generate_vite_config(npm_deps: &[(String, String)])`，调用方仅两处：
  1. **生成期**：`vue.rs` `write_project_files` 内（`extra_deps` 参数）。
  2. **run 期自愈**：`run_vue.rs`（`refresh_vite_config` 助手，config 已存在才重写；写失败=具名警告不中断）。
- **include 源恒一（r2/F-1，r3/F-5）**：两路径传同一字段 `&VueProject::npm_deps`（`write_project_files` 两调用点均显式传 `&self.npm_deps`，自愈路径同字段），`link:` 过滤在 `generate_vite_config` 模板内部实施 → 发射键集逐字一致且与调用点上下文无关。生成路径在 Plan-482 遮蔽**前**捕获原参数（`vite_npm_deps`）——推导的 engine 同级键仅进 package.json。`package.json` 的 `dependencies` = npm_deps ∪ 推导键 ∪ 框架基础集（`generate_package_json` 内部并集）——include 刻意取其中非 link: 注册表子集：框架基础依赖静态可达（include 多余），link: 依赖强制预打包会改变其 watch/更新语义（保持源态加载现状），全量集未经消费方 cand5 验证。
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
| `crates/auto-ui-tools/src/run_vue.rs` | `aad0b87f7d3072c5e8c8bae593e3d9413e48280b044f8f8c5c4d23c034400a52`（r2 修正 F-2：初版误抄消费方 BLOG-005 冻结表旧值 `0c325ac1…`；本值=base `da252a98` blob 实测） |
| `crates/auto-ui-compiler/src/ui_gen/vue/project_files.rs` | work 首日实测记录（T-01 前置核对） |

## 5. 详细设计

1. **T-01 模板发射（生成路径）**：`generate_vite_config` 签名改为接收 `npm_deps: &[(String, String)]`，`link:` 过滤（`version.contains("link:")`）在模板内实施；过滤集非空时在 `export default defineConfig({` 后注入 `optimizeDeps: { include: [<键序列>] }`（键为合法 TS 字符串字面量，经 `serde_json::to_string` 序列化防注入）；过滤集为空省略整个 `optimizeDeps` 键。`server` 块 `port:` 行后加 `host: '127.0.0.1'` 与 `strictPort: true`（带注释：端口占用=显式失败，禁止静默自增/跨栈改道）。`write_project_files` 在 Plan-482 遮蔽**前**捕获原参数 `vite_npm_deps` 传入（=`&self.npm_deps`，r2/F-1 + r3/F-5）。
2. **T-02 自愈路径同源（r2）**：`run_vue.rs` 自愈块提取为 `refresh_vite_config(output_dir, npm_dep_keys)` 助手（流级可测）：`vite.config.ts` 存在才以当前模板重写；写失败=具名警告不中断 run。include 键传 `&project.npm_deps`——与生成路径 `extra_deps` **同一字段逐字同源**（初版 r1 以磁盘 package.json 全量 deps 为源，独立复审 F-1 实证两路径发射不一致（3 vs 8 键）且 link: 依赖强制预打包未验证，已弃用并删除该读取器）。
3. **T-03 单测**：vue.rs 测试模块新增——(a) 非空键集发射含全部键且字面量合法；(b) 空集省略 `optimizeDeps`；(c) `strictPort: true` 恒存在；(d) 键序确定性（同输入两次输出逐字一致）。run_vue 侧 package.json 解析：正常/损坏/缺字段三分支。
4. **T-05 host 有界评估**：决策稿（scratch `D:/autostack/plan-artifacts/plan-050/`）实测 strictPort 发射后 127.0.0.1 被占场景 vite 实际绑定行为（是否仍 `::1` 同端口静默就绪）；评估 `host: '127.0.0.1'` 固定发射对 tauri.rs 消费方与 `localhost` 就绪判据的影响；结论=发射或不发射的具名决定 + 证据，写入 §9。
5. **T-04 回归与组收口**：`cargo t -p auto-ui-tools` 全组零回归 + 新增用例全绿；`cargo t -p auto-ui-compiler project_files` 确认旧副本未动不漂移。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/build/generated-vite-config.md | 新增：生成/自愈 vite.config 发射合同——include 键集=`VueProject::npm_deps` 中非 `link:` 注册表键（两路径同字段，过滤在模板内，空集/全 link: 省略；⊆ 实装依赖）；host+strictPort 恒发射（端口占用=具名失败） | dev 冷启动预打包动态导入链 + 端口故障显式化；`docs/specs/` 现无任何 vite 覆盖 | AC-01/02/03 |

## 6. 测试设计

最终 review 由独立上下文在同一 worktree 核对发射契约与组回归；不新建 agent/会话，不跑无关全仓 Cargo 批次。

| Case ID | AC / invariant | 输入/正式入口 | 区分性 oracle / 预期 | 边界 |
|---|---|---|---|---|
| B-01 | AC-01 | 含 npm_deps（mermaid/lowlight/katex 实值）调用模板函数 | 发射 `optimizeDeps.include` 含全部三键，TS 字面量合法 | 键序确定性 |
| B-02 | AC-01 | 空键集 | `optimizeDeps` 键整体省略（standalone 面 diff 仅 server 加固：host+strictPort） | 不发 `include: []` |
| B-03 | AC-02 | 任意输入 | `server.host: '127.0.0.1'` 与 `server.strictPort: true` 恒存在 | 与 port 行共存 |
| B-04 | AC-01 | run 自愈流（refresh_vite_config）：config 缺失 / config 已存在 | 缺失=静默 no-op 不写文件；已存在=重写为当前模板（include 键+host+strictPort 透传）；写失败分支=具名警告不中断（代码级核验，与相邻 index.html 自愈同级语义） | 不中断 run |
| B-05 | AC-02 | `cargo t -p auto-ui-tools` 全组 | 零新增失败（对照 §4 冻结基线） | 既有组 oracle 不改 |

消费方区分性验证（冷缓存 F-2 复现→绿）属 auto-blog BLOG-005 簿记（其 vue-dev-rust 门），本计划以 §10 指针承接，不在本仓复跑。

## 7. 验收标准

- AC-01（owner_stage=work；r3 修订）：两条写入路径发射的 `optimizeDeps.include` 键集=`VueProject::npm_deps` 中非 `link:` 注册表键集（同一字段、过滤在模板内，逐字一致；消费方 cand5 trio 形态成立）；键集 ⊆ 实装依赖；过滤集为空省略键；B-01/B-02/B-04 过。
- AC-02（owner_stage=work）：`strictPort: true` 恒发射；`cargo t -p auto-ui-tools` 全组零回归（B-03/B-05 过）。
- AC-03（owner_stage=work）：T-05 host 决策稿有具名结论（发射或证据化不发射）。
- AC-04（owner_stage=review）：独立上下文核对发射契约、组回归与 SD-01。
- AC-05（owner_stage=merge）：SD-01 spec 落地；消费指针（BLOG-005 恢复验证）在其簿记留痕。

## 8. 执行步骤

- [x] T-01（owner_stage=work；依赖无；AC-01）：模板签名与 include/server 加固发射 + write_project_files 接线；B-01/B-02/B-03 单测。[✅ 已完成 2026-11-04，r3 修订] `generate_vite_config(npm_deps: &[(String,String)])`：非 `link:` 注册表键发射 `optimizeDeps{include:[…]}`（serde_json 转义，过滤在模板内），过滤集为空整体省略；`host: '127.0.0.1'`+`strictPort: true` 恒发射；write_project_files 在 Plan-482 遮蔽前捕获 `vite_npm_deps` 原参数传入（r3/F-5：推导键仅进 package.json）。
- [x] T-02（owner_stage=work；依赖T-01；AC-01）：run_vue.rs 自愈接线 + B-04 流分支单测。[✅ r2 完成 2026-11-04] r1 实现为磁盘 package.json 源（`package_json_dep_keys`），独立复审 F-1 实证与生成路径键集不一致（3 vs 8 键）且 link: 强制预打包未验证 → r2 弃用删除，改 `refresh_vite_config(&project.output_dir, &project.npm_deps 键集)`——与生成路径 `extra_deps`（=`&self.npm_deps`）同一字段逐字同源；流级测试覆盖缺失 no-op/重写含 include 两分支，写失败分支代码级核验（与相邻自愈同级语义）。
- [x] T-03（owner_stage=work；依赖T-01/02；AC-02）：`cargo t -p auto-ui-tools` 全组回归（B-05）+ `-p auto-ui-compiler project_files` 未漂移确认。[✅ 已完成 2026-11-04] 新增 4 单测全绿；全组 `-p auto-ui-tools --lib` 203/203 绿（5 基线红：display_reader_v01_corpus、gallery_demo_row_routable、try_full_parse_018_api、shell_pack_lib_freshness、plan_662_gallery_apps_dir_layouts——均经 stash 回 base 实证同败，与本改动无关）；`-p auto-ui-compiler` test_generate_project_files 1/1 绿（旧副本未动）。
- [ ] T-04（owner_stage=review；依赖T-03/05；AC-04）：同树独立上下文最终复审。
- [x] T-05（owner_stage=work；依赖T-01；AC-03）：host 有界评估决策稿 + §9 具名结论。[✅ 已完成 2026-11-04] 结论=**发射 `host: '127.0.0.1'`**（并入 T-01）。探针 4 组（vite 5.4.21 实测，plan-artifacts/plan-050/host-probe/DECISION.md）：strictPort 单独不闭 E-4a——127.0.0.1 被占仍静默绑 ::1 同端口报 ready（curl ::1→200 实证）；host 固定后占用=具名失败 EXIT=1（CLI 与 config 形双验）、空闲=localhost 经双栈回落 200、tauri webview（localhost: URL）兼容、Local: 行由歧义变明确。

## 9. 复审记录

- stage: new | plan_id: PLAN-050 | plan_revision: r1 | outcome: pass（授权已在 §4 记录：消费方用户 2026-11-04 拍板路线①） | next: work（T-01/T-02 可立即开工；T-05 评估可与 T-01 并行）。

stage: work | plan_id: PLAN-050 | plan_revision: r1 | outcome: pass（T-01/T-02/T-03/T-05 完成，execution_done 待 T-04 复审） | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | base_commit: da252a98 | code_commit: 1a9131fc | dependency_revisions: auto-lang f6b0f918d（master，detached 同级）、auto-web 3de5524（master，detached 同级）、auto-down 7c4d5cb（HEAD，detached 同级） | task_ids: T-01、T-02、T-03、T-05（T-04=review 所有） | evidence: 新增 4 单测全绿（vite_config include/omit/deterministic+quoting、package_json_dep_keys 三态）；全组 nextest `-p auto-ui-tools --lib` 203/203 绿、5 基线红 stash 回 base 实证同败；`-p auto-ui-compiler` test_generate_project_files 1/1 绿；host 探针 4 组（DECISION.md：strictPort 单独 E-4a 复现→host 固定具名失败/localhost 回落兼容）；SD-01 spec 草案在树（docs/specs/build/generated-vite-config.md，merge 时发布） | blockers: 无 | next_responsible_role: review（T-04 同树独立上下文最终复审；统一回归门=复审/merge 按仓档位）

stage: review | review_scope: final | reviewer_identity: 独立 general-purpose 子代理上下文（新会话，与实现无共享记忆；局限=同物理机） | plan_id: PLAN-050 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | reviewed_commit: 1a9131fc | base_commit: da252a98 | spec_inputs: SD-01 草案（树内冻结副本） | acceptance_results: AC-02 pass（全组 208 run/203 绿/5 基线红——stash detach 回 base 复跑 5/5 同败含同 panic 位；compiler 1/1 绿且 diff 零改动）、AC-03 pass（DECISION.md 复跑独立重现：占用 EXIT=1 具名、空闲 localhost 回落 200）、AC-04 pass 带 F-3/F-4 低项 | findings: **F-1（中）**生成路径 include 键集（探针 3 键=extra_deps）≠该次写入 package.json dependencies 键集（8 键——generate_package_json 内部并集框架基础集），两路径发射不一致、§2"逐字同源"前提失实；**F-2（低）**§4 run_vue.rs 冻结哈希误抄消费方旧值（实测 base blob=aad0b87f…）；**F-3（低）**§6 B-02 oracle 文本未随 host 发射更新；**F-4（低）**B-04 流级 oracle 仅 parser 单测间接覆盖 | evidence: D:/autostack/plan-artifacts/plan-050/review/（ac01-probe、host-probe-rerun）；结束态树恢复干净 | next: work（F-1 修复方向两择：生成路径改传全量 deps 键集，或契约修订为 npm_deps 同源子集——r2 取后者，理由：cand5 已验证形态即 npm_deps 三键，全量集含 link: 依赖强制预打包未验证）

stage: work | plan_id: PLAN-050 | plan_revision: r2 | outcome: pass（R1 修复 F-1..F-4 完成，重交 T-04 复审） | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | base_commit: da252a98 | code_commit: 2f280f20 | dependency_revisions: 同 r1 receipt | task_ids: T-02（r2 重做）、T-01/03/05（措辞/簿记同步） | evidence: include 源改 `refresh_vite_config(&project.output_dir, &project.npm_deps…)`——与生成路径同一字段逐字同源（两 write_project_files 调用点均 `&self.npm_deps`）；`package_json_dep_keys` 删除；新增流级测试 refresh_vite_config_flow_branches（缺失 no-op/重写含 include+host+strictPort）；4/4 新测试绿；全组 203/203 绿（同 5 基线红排除集）；compiler 1/1 绿；SD-01 草案改"同源同一来源"契约；F-2 哈希修正（aad0b87f…，git show base blob 实测）、F-3 §6 文本更新 | blockers: 无 | next_responsible_role: review（T-04 复审 #2，核查 r2 修订合同与 F-1 闭合）

stage: review | review_scope: final | reviewer_identity: 独立 general-purpose 子代理上下文 #2（与实现/复审#1 无共享记忆） | plan_id: PLAN-050 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | reviewed_commit: 2f280f20 | base_commit: da252a98 | spec_inputs: SD-01 草案（树内） | acceptance_results: F-1 部分闭合（trio/empty 两路径逐字一致实证 true，editor-link 案 false）；F-2/F-3/F-4 闭合；AC-02 pass（208 run/203 绿/失败集恰 5 基线红，4 新测试重跑绿）；AC-03 pass（维持）；AC-04 pass 带 F-5 | findings: **F-5（中，新）**生成路径消费 Plan-482 遮蔽后集合（vue.rs 推导块在 write_project_files 内、vite config 写在其后）：editor-link 类项目生成发射 [editor,mermaid,engine] vs 自愈 [editor,mermaid]——两路径发散且 link: engine 被强制预打包（合同自述要避免的语义改变）；仓内 015-notes 直接声明 engine（推导不激活）暂不受影响，消费方 trio 不受影响 | evidence: D:/autostack/plan-artifacts/plan-050/review2/（REVIEW2.md、f1-probe） | next: work（修复方向=遮蔽前捕获/或模板内过滤，修后应可直接过）

stage: work | plan_id: PLAN-050 | plan_revision: r3 | outcome: pass（R2 修复 F-5 完成，重交 T-04 复审 #3） | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | base_commit: da252a98 | code_commit: 8638863a（实现 c0301533 + SD-01 措辞） | dependency_revisions: 同 r1 receipt | task_ids: T-01/T-02（r3 措辞同步） | evidence: `generate_vite_config` 签名改收 `(name,version)` 对、`link:` 过滤在模板内实施（`version.contains("link:")` 排除）；生成路径在 Plan-482 遮蔽前捕获 `vite_npm_deps` 原参数（推导键仅进 package.json）；run 路径直传 `&project.npm_deps`；新增 oracle `vite_config_excludes_link_deps_from_include`（editor/engine link 排除、mermaid 保留、全 link: 省略键）；测试 5/5 绿；全组 **204/204** 绿（同 5 基线红排除集）；SD-01 草案 r3 措辞同步 | blockers: 无 | next_responsible_role: review（T-04 复审 #3，scoped=F-5 闭合与 r3 合同）

## 10. 待澄清事项

- host（E-4a）发射与否=T-05 决策稿产出，非用户待决；证据支持则随 SD-01 边界记录，不支持则具名不发射并留消费方自愈检测（run 期解析实际绑定 URL）说明。
- strictPort 的多实例语义变化（第二实例显式失败）为**有意合同**（§2），与消费方 BLOG-005 B-01 故障门一致，不视为回归；若复审发现本仓存在依赖静默自增的测试/文档，按零回归原则报 work 处置。
- 观察项（不在本计划处置）：compiler `project_files.rs:73` test-only 旧副本与生产模板已漂移（无 server 块）。

stage: review | review_scope: final | reviewer_identity: 独立 general-purpose 子代理上下文 #3（与实现/复审#1#2 无共享记忆） | plan_id: PLAN-050 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/ui-050/auto-ui | branch: codex/plan-050-dev | reviewed_commit: 8638863a | base_commit: da252a98 | dependency_revisions: auto-lang f6b0f918d、auto-web 3de5524、auto-down 7c4d5cb（同 r1） | spec_inputs: SD-01 草案（树内冻结副本，r3 措辞） | acceptance_results: AC-01(r3) **pass**（F-5 closed：独立探针 21/21——editor-link/direct-engine-link/all-link/registry-engine/trio 五类输入两路径输出逐字 byte-equal，include 零 link: 键，捕获点 vue.rs:2563 在遮蔽块 2565-2589 之前实证；附加纵深=遮蔽集渲染与 raw 集相同）；AC-02 pass（209 run/204 绿/失败集恰 5 基线红，5 新测试重跑绿）；AC-03 pass（r3 diff 未触 server 块，host 探针证据复用）；AC-04 pass | findings: F-6（低，文档残留）§0/§2 签名/§5 T-01/§8 注记残留 r1 措辞——纯措辞零行为影响，本 receipt 前已在主检出 Plan 修正（非规范段对齐，不改语义契约，评审 commit 不变） | evidence: D:/autostack/plan-artifacts/plan-050/review3/（REVIEW3.md、probe-run.log） | pending_closeout_gates: merge 所有（SD-01 发布、landing 检查、归档、worktree 清理） | next: merge
