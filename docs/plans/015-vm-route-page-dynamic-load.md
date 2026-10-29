---
plan_id: PLAN-015
status: drafting
feature_name: vm-route-page-dynamic-load
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: ["auto-ui/PLAN-003"]
supersedes_spec_components: []
new_spec_components: ["docs/specs/runtime/vm-route-page-loading.md"]
touched_goals: ["auto-blog/BLOG-004"]
---

# PLAN-015：VM 渲染器路由页动态加载静默失败（阻塞 BLOG-004 原生腿）

## 0. 变更摘要

BLOG-004 原生腿新鲜证据：多页 VM 应用（auto-blog，app.at 声明 8 路由 + outlet）下，导航到 '/' 的 **author 页静默加载失败**——无 Init 派发、无 panic/诊断输出；对照同机制 '/demo' 的 home 页加载成功（9555 字符快照、Init parked/resumed 正常）、PLAN-003 无路由单页夹具正常。静默失败使"原生窗口执行作者任务"（BLOG-004 V-03/AC-04）不可达。本计划：①有界调查定界根因；②修复路由页动态加载的静默失败并把"页不可加载"改为具名诊断；③以 BLOG-004 native-vm-rust profile 消费者闭环验收。只修 VM 渲染器动态页加载；不改 Blog .at（其接缝歧义已由 BLOG-004 T-03 修复并实测）、不触 Vue 轨（PLAN-010 已闭合）、不改 BLOG-004 验收口径。

## 1. 目标

多页 VM 应用中，任何已声明路由的页面要么完整加载（渲染并派发 Init），要么产生**具名诊断**（不再静默占位/吞错）；author 类页面（含 timer/多条件/列表模板特性）可正常加载。成功标准：auto-ui 侧多页路由 fixture 正控全绿 + 不可加载页负控具名报错；消费者侧 BLOG-004 `verify_dual.py --profile native-vm-rust` 全 profile EXIT=0（真实窗口 MCP 驱动作者任务 + 退出重开）。

非目标：Vue 轨（PLAN-010 已交付）；Blog 仓 .at/源码变更；VM 渲染语义重设计；单页无路由应用（已正常）。

## 2. 架构方案

**观测事实（2026-10-29 BLOG-004 会话实测，证据见 §4）**：
- F1 窗口/壳/MCP/后端集成全通：`auto run -r vm --server rust` 真实窗口"博客"启动、MCP `/mcp` JSON-RPC snapshot/press 到达 App 壳 handler、Rust 后端 api=200。
- F2 导航 '/demo' → home 页完整加载（快照 9555 字符，`handler_home_Init` parked/resumed 日志）。
- F3 导航 '/' → author 页失败两种表现：run5 整应用静默退出（exit 1，无错误输出）；t06b 应用存活但快照恒 `<outlet: page author not loaded>`，**无 `handler_author_Init` 日志**。
- F4 PLAN-003 无路由单页夹具（PLAN-003-native-vm-control）正常。

**代码锚点（observed）**：`crates/auto-ui-runtime/src/ui/aura_view_builder.rs` 6652 附近 outlet 渲染：`registry.get_by_route_module(&route.module)` 命中→构建页组件；未命中→`"<outlet: page {} not loaded>"` 文本占位（6693-6697）。

**假设（hypotheses，待 T-01 定界）**：
- H1 注册缺失：author 页 widget 未进 route-module 别名注册表（但 home 同路径注册成功，若注册同源则 H1 弱）。
- H2 页构建期错误被吞：author 页 render/build（timer `every_ms`、多 `if save_state` 条件、`for` 列表、嵌套插值 f-string 等 home/夹具没有的特性）在动态组件构建路径 panic/Err 且被上层吞掉——t06b"占位依旧"与 run5"整应用退出"可为同一根因的两种表现（吞错 vs 致命）。
- H3 路由模块名/别名解析在特定页名（author）下失配。

**修复面（H 定界后收敛）**：动态页加载路径的注册/构建错误改为具名传播（`Err` 上浮或快照内诊断文本携带原因），消除静默；错误根因按 T-01 结论修复。

## 3. 技术栈

既有 auto-ui-runtime（VM 解释器/aura_view_builder）+ auto-ui-tools 测试夹具模式（PLAN-003 前例：fixture 目录 + `git archive` 导出 + `auto run -r vm` + MCP 驱动）。复用 BLOG-004 的 MCP 驱动模式（autoui_snapshot/autoui_action/autoui_type，已实证）。

## 4. 需求分析与背景调查

**授权**：BLOG-004 会话用户指示（2026-10-30）——按"选项 2 保守方式"先起草本计划；**实现开工前须与 auto-ui 在途 UI-005 会话协调**（该会话 2026-10-29/30 多次推进 master：2c5e4cb/6e9716b/a052bde/d69b7e8/f7c83c6 等，且 docs/plans/007、008、014 及 .next-id 为其未提交 WIP——编号避让已执行：009 留其用，011 已被占用，本计划取 015=当时最大前缀 014+1）。无额外预算/自动续跑授权。

**证据（auto-blog `docs/plans/attachments/004/`，本地不入库）**：
- `t06-run4.log`（`ee69070e`）：多 store 裸调用歧义错误（该 .at 缺陷已由 BLOG-004 T-03 修复 `7f9b675`，非本计划范围）。
- `t06-run5.log`：窗口/MCP 就绪后导航 '/' → 整应用静默 exit 1。
- `t06b.log`（`0fa4784a`）：home 加载成功快照 + author 导航后仍 "not loaded"、无 author Init。
- 消费端登记：auto-blog `docs/plans/004-dual-runtime-acceptance.md` §10 第 3 项 + T-03 行（commit 7f9b675）。

**Spec 现状**：`docs/specs/runtime/` 有 native-vm-app-validation.md（PLAN-003）——不覆盖多页路由动态加载语义；本计划补 SD-01。

## 5. 详细设计

### 执行入口

- 实现工作树（开工时创建）：`D:/autostack/.wt/ui-015/auto-ui`，分支 `plan-015-dev`；组内兄弟依赖树按需（参考 ui-010 前例：auto-lang/auto-down detached）。
- 开工前置：UI-005 会话协调确认（见 §10）。

### T-01 定界设计（有界调查，决策工件）

在 auto-ui 测试树建**多页路由 fixture**（仿 PLAN-003 夹具形态）：app.at（2 路由 + outlet）+ home 页（简，同 Blog home 形）+ author 页（简，但带 Blog author 的嫌疑特性，按 timer→条件→列表→f-string 逐项二分）。运行 `auto run -r vm` + MCP 驱动导航，捕获：
1. 各特性组合下 author-like 页的加载结果（Init 派发/静默/崩溃）；
2. 失败路径的精确吞错点（在 aura_view_builder/解释器动态组件路径插桩或读源定位）；
3. 输出 `docs/plans/attachments/015/root-cause.md`（决策工件：根因结论 + 修复面 + 具名错误设计）。

### 修复设计（T-02，依 T-01 结论）

- 静默失败具名化：页不可注册/构建失败不得回落 "not loaded" 占位了事——以具名诊断上浮（快照诊断文本携带原因 + 日志 error 行），保留占位仅为"路由未匹配"语义。
- 根因修复：按 T-01 定界结果（H1 注册补全 / H2 构建路径错误传播与特性修复 / H3 别名解析）。

### 规范增量

| delta_id | action | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/vm-route-page-loading.md | 无约定 → 多页 VM 应用路由页动态加载合同：已声明路由页必须完整加载并派发 Init，不可加载必须具名失败（禁止静默占位/吞错）；timer/条件/列表页为受支持特性面 | BLOG-004 类多页消费者可复验；防回归 | AC-01..04 |

## 6. 测试设计

证据政策：原始日志/快照入 `docs/plans/attachments/015/`（不入库）；Plan 内记命令/退出码/关键结果/本地路径+hash。

| case_id | owner / AC | 输入与入口 | 可区别错误实现的判据 |
|---|---|---|---|
| V-01 | T-01 / AC-01 | 多页路由 fixture 逐特性二分（timer/条件/列表/f-string），`auto run -r vm` + MCP 导航 | 失败特性组合可稳定复现；吞错点定位到文件:行；决策工件给出唯一根因结论 |
| V-02 | T-02 / AC-02 | 修复后同 fixture 全特性页 | author-like 页完整加载并派发 Init（快照含页内容+CALL_HANDLER 日志）；构造不可加载页 → 具名诊断（非静默占位/非整应用退出） |
| V-03 | T-02 / AC-03 | 多页回归：home 正控保持 + 新增 timer 页正控 + 负控 | 三者全绿；PLAN-003 单页夹具回归不破坏 |
| V-04 | T-03 / AC-04 | 消费者：auto-blog `python tests/dual/verify_dual.py --profile native-vm-rust`（真实窗口 MCP 驱动作者任务 + 正常退出 q/硬终止 + 重开数据保持） | 全 profile EXIT=0；author 页在原生窗口完成创建/保存并经独立 HTTP+数据目录对照 |

负控说明：V-02 的"不可加载页"用 T-01 定界的真实失败形态构造（如缺失页注册），非人为 timeout；跳过/0 场景不可计 pass。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 根因定界完成：V-01 稳定复现 + 吞错点文件:行 + 决策工件（root-cause.md） |
| AC-02 | work | 修复 + 具名错误化落地：V-02 全绿，静默路径消除（代码级：无 fallback-to-placeholder 的吞错分支） |
| AC-03 | work | 多页/单页回归全绿：V-03（含 PLAN-003 夹具不破坏） |
| AC-04 | work | 消费者闭环：V-04 EXIT=0（BLOG-004 原生腿全绿，其 T-03/T-06 可关闭） |
| AC-05 | review | 固定实现复验 AC-01..03 与 SD-01，无越界改动；说明上下文独立性限制 |
| AC-06 | merge | 门（适用本仓门）+ Spec/索引/归档/worktree 清理 + 向 BLOG-004 登记精确交付版本 |

## 8. 执行步骤

- [ ] T-01（owner_stage=work；依赖无；AC-01）：多页路由 fixture + 逐特性二分定界，输出 root-cause.md 决策工件（§5 T-01 设计）。
- [ ] T-02（owner_stage=work；依赖T-01；AC-02/03）：按定界结论修复 + 具名错误化；V-02/V-03。
- [ ] T-03（owner_stage=work；依赖T-02；AC-04）：重建交付二进制（auto-ui-tool 部署到消费发现路径），BLOG-004 native-vm-rust 全 profile 闭环（V-04）；提交消费回执。
- [ ] T-04（owner_stage=review；依赖T-03；AC-05）：同树最终复审。
- [ ] T-05（owner_stage=merge；依赖T-04；AC-06）：门/SD-01 Spec/归档/worktree 清理/BLOG-004 登记。

## 9. 复审记录

stage: new | plan_id: PLAN-015 | plan_revision: 1 | outcome: blocked（计划就绪；唯一缺项=开工协调确认） | next: user 确认 auto-ui UI-005 会话状态（暂停/结束）后 → work/T-01。起草为保守模式：仅本文件，未触实现、未建 worktree、未动他人在途文件（007/008/012/014、.next-id 等）。T-01 为有界调查（决策工件制），T-02 起的修复机制留给 T-01 结论定界，不预设实现细节。

## 10. 待澄清事项

1. **开工协调（唯一前置）**：auto-ui 在途 UI-005 会话状态。确认后本计划即可进 work；若其仍在途，可协商页面级并行（本计划触面 = run_vm 动态页加载/aura_view_builder outlet/注册表，与其 UI-005 的 G2 预算/回调相位面预计不重叠，但 master 移动需 rebase 纪律）。
2. 未解假设 H1/H2/H3 已绑定 T-01 有界调查定界；不预设根因。
3. 消费者闭环（T-03）需与 BLOG-004 会话（本起草者）协同执行 native profile；该侧接入面与 MCP 驱动模式已实证可用。
