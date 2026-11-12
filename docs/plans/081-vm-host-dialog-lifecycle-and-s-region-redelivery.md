---
plan_id: auto-ui/PLAN-081
status: executing
feature_name: S 区编辑链修复——路由/实例解析/派发域（READER-009 W-07 消费解除；r2 收窄）
author: [relay/009-session]
created_at: 2026-11-12T00:00:00Z
updated_at: 2026-11-12T15:30:00Z
plan_revision: 2
current_step: 1
total_steps: 3
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# PLAN-081：S 区编辑链修复（r2——路由/实例解析/派发域，READER-009 W-07 消费解除）

## 0. 变更摘要

**r2（2026-11-12，用户裁定"plan-081 只留 D3"）**：范围收窄为 **D3 S 区编辑链
修复**。r1 的 D1/D2（native 对话框生命周期/生成可靠性）按 T-01 仪器化调查
收束：

- 干净上下文（MCP 驱动、空库/21 本种子库、无矩阵）**16/16 行为正确**——
  spawn 0.5-0.8s、10s 无输入稳定窗全开、WM_CLOSE→IDCANCEL 干净取消、
  n=0 正确回传、无宿主挂起、跨轮零退化（dlg-stress/dlg-stress4 证据）。
- 两个根因候选被证据排除：**MTA 套间假设死**（CoGetApartmentType=MAINSTA）、
  **错窗假设死**（父窗=正确主窗同线程 enabled=1）。
- 历史故障未复现；mode B 现场签名再定谳=picksession log 中 PickFiles 段
  **81-200ms 内 resume**（宿主被调用、对话框路径瞬返无窗）——探针常驻
  （commit 3227af96，AUTO_DIALOG_DIAG 门控）供 009 现场再现时即时定谳。
- **用户裁定**：native IFileDialog 的 MCP/UIA 不可操作性（DirectUI 无 HWND，
  round48 三通道全败）是无人值守门禁死结；改道方向（AutoUI 自制文件选择器
  组件，跨平台一致、MCP 可操作）**以后另议**，不在本计划。桌面侵入式实验
  （动鼠标/焦点/窗口）今后须先征得用户同意。

D3 症状（run97，9dd3fc27 cohort 实测）：详情页在"卡行→阅读页→←返回"后
快照缺'编辑'/'继续阅读'→ W-11 级联 32 叶；W-07 五叶全部 `TapParagraph
handler node not visible`。考古定谳：r1 的"修复内容只存在于 7d3a73aa 致死线
捆绑 auto-ui"不成立（7d3a73aa 全线为 master 祖先）；实质=round47 具名的
**round-19 路由域缺陷从未实施**（route_id 丢失→详情不可达 / 合成派发路由域，
"069 的 F3/F4 为 T-00 有界定谳前不实施项"）。

## 1. 目标

- D3：S 区编辑链在 owner-filter 基线可用——详情页（/book/:id）在
  "卡行→阅读页→←返回"导航后渲染完整（'继续阅读'与'编辑'在场），
  编辑表单可开合；W-07 批注编辑链五叶（real-selection-save / edit-revisit /
  save-reselect / delete-revisit / return-library-preserves-state）翻绿。
- 非目标（r2）：native 对话框可靠性/可操作性（T-01 收束 + 用户裁定改道另议）；
  949/995 性能语义；batch write-503 族（995 后域具名，非本计划）；
  W-09 宿主 family+mode 面；非 Windows 平台；Vue 轨。

## 2. 架构方案

- **复现**：edit_chain_probe（boot 21 本库→卡行→阅读页→返回→详情快照
  断言'编辑元数据'/'继续阅读'→编辑表单开合），本机仪器 cohort 即红（run97
  同签名）。
- **定谳**：runtime 路由/实例解析/派发域有界调查——resolved_route 历史
  （7d3a73aa）、outlet 实例挂载/重建（4214340c F3 type-exact instance
  resolution）、合成派发路由提交（round-19 家族）三处优先；route dispatch
  oracle 底座已在（4214340c F4）。
- **修复 + 回归锚**：修复后补 route dispatch oracle 负例/正例（防止回归），
  单元组回归 + harness 消费验证（W-07 五叶 + S06/S07）。

## 3. 技术栈

Rust（auto-ui-runtime ui/dynamic.rs 路由与实例解析、ui/vm_bridge.rs 派发、
handler_codegen.rs 路由重写）、现有 `cargo t` 组 + reader-009 组 harness。

## 4. 需求分析与背景调查

- **授权（r2）**：用户裁定 2026-11-12"plan-081 只留 D3"——D1/D2 以 T-01
  调查收束，AutoUI 自制文件选择器组件方向缓议（后续计划，009 picker 八叶
  owner 届时随组件交付）；D3 继续在本计划执行。桌面侵入式实验须先征得
  用户同意。原始授权：READER-009 会话（round49"我在此起草（推荐）"先例 +
  round54 方向确认），执行由 auto-ui 会话接手。
- **证据链**（全部在案）：
  - 症状：t03/ws-ui-vm-97-nopicker/{results.json, w11-traceback.txt,
    fail-onclick-TapParagraph.txt}——S06-wide-light '编辑'不可见→W-11 级联
    32 叶；W-07 五叶 TapParagraph 中断；S10×2 搜索面板超时（同族疑似）。
  - 考古：7d3a73aa/8f993ccb/258bbd52/d0e1a688 全部为 master 祖先（r1 前提
    证伪）；round47 移交包具名"F3/F4 不实施项"。
  - T-01 仪器化（3227af96）：dlg-stress 10/10 + dlg-stress4 6/6 干净上下文
    全绿；picksession log mode B 快返签名；当日本机 Windows 事件日志含
    BlueScreen/LiveKernelEvent（机器级不稳，与 009"机器并行活动干扰"归因
    一致）。
- **约束**：兄弟只读（auto-lang@0b3fc9063/auto-down@58ea33f/auto-web@bf72baa）；
  不回退 accesskit；949/995 性能语义不变；不触 batch write-503 族。
- 观测 vs 假设：症状=run97 实测（观测）；根因定位=假设（T-04 定谳）。

## 5. 详细设计

### D3 路由/实例解析/派发域修复

- 调查入口：`ui/dynamic.rs`（resolved_route/route-history、outlet 实例
  挂载、sync_child_props_to_root）、`ui/vm_bridge.rs`
  （call_handler_for/prepare_handler_dispatch、HandlerNotFound 面）、
  `ui/handler_codegen.rs`（router.back 重写/app_root_field）。
- 症状 discriminating 问题：① back 后详情页是**实例未重建**（outlet 空）
  还是**渲染树塌缩到根 row**？② TapParagraph 行缺失是**列表渲染条件**
  （store 数据未达）还是**派发注册**（handler 行不在快照但页面在）？
  ③ 与 S10 搜索面板超时是否同根（导航后状态间隙家族）？
- 修复方向（按定谳）：route_id 生命周期（round-19 ③A）、实例 type-exact
  解析的过严分支（4214340c F3 回归面）、合成派发路由提交（round-19 ④）。

### D1/D2 收束记录（r2，非目标）

- 结论与证据见 §0/§9；探针（3227af96）常驻 `host_adapters/dialog.rs`——
  AUTO_DIALOG_DIAG=1 时输出套间/父窗/CBT 窗口生命周期/#32770 消息时序，
  009 现场再现时即时定谳。无代码修复项。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-03 | modify | docs/specs/runtime/component-and-store-instances.md（按定谳落点可调至路由/实例解析规范） | 路由导航（含 back/forward）后 routed 页实例完整重建；跨导航 handler 派发可达 | run97 S06/W-07 实测断裂 | AC-03 |

（r1 的 SD-01/SD-02——native 对话框生命周期与 bridge 送达——随 D1/D2 收束
撤销，未落任何规范。）

## 6. 测试设计

- D3 复现台：edit_chain_probe（plan-artifacts/plan-081/，boot→卡行→阅读→
  返回→详情断言）+ harness W-07 五叶/S06-S07 腿。
- 回归锚：route dispatch oracle（4214340c F4 底座）正例/负例 + 触达修复面
  的单元组。
- 消费回归（review）：009 换 pin 全矩阵 82 叶零死亡 + W-07/S06 翻绿
  （picker 八叶不在本轮）。

## 7. 验收标准

| AC | owner_stage | 内容 | 验证 | r2 状态 |
|---|---|---|---|---|
| AC-01 | work | （r1）mode A 根因定谳+修复 | 009 harness 压测 | **收束 r2**：干净上下文 16/16 行为正确；MTA/错窗排除；未复现历史事件（环境归因+探针常驻，§9） |
| AC-02 | work | （r1）mode B ≥10/10 生成 | 同上 | **收束 r2**：spawn 16/16 达标；可操作性死结→组件方向（用户裁定另议） |
| AC-03 | work | D3 S 区编辑链在基线可用 | edit_chain_probe 绿 + 009 W-07 五叶翻绿 | 执行中 |
| AC-04 | review | 009 换 pin 消费回归 | 82 叶零死亡 + W-07/S06 翻绿（picker 八叶除外） | 执行中（r2 修订范围） |

## 8. 执行步骤

| 任务 | owner_stage | 依赖 | 内容 | 验证 | AC |
|---|---|---|---|---|---|
| T-01 | work | — | （r1）D1 插桩调查 | 探针 3227af96 + 16/16 压测 + 定谳记录（§9） | AC-01/02（收束） |
| T-04 | work | — | D3 复现→定谳→修复（路由/实例解析/派发域） | edit_chain_probe 绿 + oracle/单元锚 + W-07 五叶 | AC-03 |
| T-05 | review | T-04 | 009 换 pin 消费回归（跨仓协作：READER-009 会话） | 82 叶零死亡 + W-07/S06 翻绿 | AC-04 |

- ~~T-02/T-03~~（r1）D1/D2 修复——**r2 撤销**（用户裁定：native 对话框改道
  另议；无修复项，见 §5 收束记录）。
- 实现worktree：`D:/autostack/.wt/ui-081/auto-ui`，分支 `codex/plan-081-dev`，
  基线 **9dd3fc27**（=计划声明基线 master@8389a1e5 内容；master@ad9c05da ×
  auto-lang@0b3fc9063 不编译——auto-ui-compiler 需 `discover_front_exe_identity`
  新 API——故分支驻 9dd3fc27，落 master 由 merge 处理）；兄弟
  auto-lang@0b3fc9063 / auto-down@58ea33f / auto-web@bf72baa。
- 消费方交接：交付 pin+驱动/工具哈希回执 → READER-009 换 pin 终收口
  （picker 门除外——组件方向另议）。

### 执行证据（work 阶段增量，2026-11-12）

- **worktree**：`D:/autostack/.wt/ui-081/auto-ui`，分支 `codex/plan-081-dev`
  （§8 原记 plan-080 名已被并行 AGE 会话 080 计划占用 `.wt/ui-080`+
  `codex/plan-080-dev`，不重用）。
- **分支基线修正**：master@ad9c05da × auto-lang@0b3fc9063 不编译——
  auto-ui-compiler `app_registry.rs:207` 需 `discover_front_exe_identity`
  （auto-generation-core 新 API，0b3fc9063 只有 `discover_front_exe`）→
  分支 rebase 到 **9dd3fc27**（=计划声明基线内容，009 cohort 实际构建点）；
  探针 commit **3227af96**。
- **T-01 探针**（`host_adapters/dialog.rs`，AUTO_DIALOG_DIAG 门控，零诊断
  零 OS 面）：①CoGetApartmentType（进/出两点）②父窗取证（类名/标题/可见/
  禁用/owner链/归属线程）③WH_CBT 线程钩子（create/destroy/activate/
  SC_CLOSE 时序）④首个 #32770 子类化（WM_CLOSE/COMMAND(IDCANCEL)/
  SYSCOMMAND(SC_CLOSE)/DESTROY 时序+前台窗+UIA 监听态）。
- **D3 前提修正（考古定谳）**：7d3a73aa 及其线（8f993ccb/258bbd52/d0e1a688）
  **全部为 master 祖先**——r1"修复内容只在致死线捆绑 auto-ui"不成立；
  7d3a73aa 内容（resolved_route/publish_completed_handler_updates/
  app_root_field/owner_state_id）全在 master。真实 S 区态=round47 具名
  "069 的 F3/F4（route_id/合成派发根因）T-00 有界定谳前**不实施**"项——
  round-19 路由域缺陷从未实施；run97 失败分解：S06-wide-light '编辑'不可见
  →W-11 级联 32 叶；W-07 五叶全部 `TapParagraph handler node not visible`；
  S10×2 搜索面板超时；batch-remove-restore=write-503 族（995 后域，非本
  计划）；cold-load 15.06s/15.0s（性能域，非本计划）。
- **复现台**：驱动=t01/toolchain-target auto.exe（009 8d7dc2cf，Nov11）+
  本组新构建 auto-ui-tool（仪器版 sha256 da154bfa…）+ reader-009 组 harness
  （dlg_stress.py / dlg_attach.py / edit_chain_probe.py，PICKER_ART 全部指
  plan-081 scratch，不污染 009 现场）。
- **T-01 干净上下文压测（定谳证据 A）**：dlg-stress（空库 10/10）+
  dlg-stress4（21 本种子库 6/6）——spawn 0.5-0.8s、10s 无输入稳定窗全开、
  PostMessage WM_CLOSE→IDCANCEL→destroy→n=0 干净、跨轮无退化。探针判读：
  **apt=MAINSTA（MTA 假设死）**、父窗=正确主窗同线程（`owner_tid=cur_tid`、
  enabled=1、owner=none）（错窗假设死）、uia_listening=1 为常态非判别量。
- **mode B 签名再定谳（定谳证据 B）**：picksession log（round53/54 现场，
  无 diag env）——10/10 次 PickFiles **81-200ms 内 resume**=对话框调用瞬返
  无窗（"bridge 未达宿主"假设死：宿主被调用、对话框路径瞬返；round53
  "PASS" 两腿实为书已在库的 oracle 污染）。
- **全矩阵上下文**：hctx-run1 在 batch write-503 区后端死亡（已知族，非本
  计划域）；hctx-run2 走完 72+ 叶后同样终殁（同族）——当日本机 Windows
  事件日志含 BlueScreen/LiveKernelEvent（12:48）——全矩阵消费回归的机器
  稳定性风险记档，review 阶段换 pin 消费时需独占稳定机器窗。
- **用户裁定（2026-11-12）**：D1/D2 收束，"plan-081 只留 D3"；AutoUI 自制
  文件选择器组件方向另议；桌面侵入式实验禁行（须先征得同意）。

### T-04b 定谳证据（2026-11-12 下午，root cause 已锁定到渲染投影层）

- **复现矩阵**：hctx-run3（仪器工具，全矩阵）——W-07 五叶全断
  `handler node not visible: TapParagraph`（与 run97 同签名）；
  本轮 S02-S12 级联为截图工具环境故障（非应用）；batch-503 族维持。
- **确定性廉价复现**：bookswitch_probe（boot→卡行→阅读页，plan-081/
  bookswitch-3）——**首访即复现**：¶ 行内容渲染（¶0/¶1 + 正文文本 + 书题）
  但行上 `onclick: .TapParagraph(p.idx)` 与 style 数组**整块缺失**，
  A/B/C 三书一致；vnode id 与 run3 失败快照逐位相同（模板稳定 id）。
- **快照忠实性定谳**：autoui_snapshot 来自 styled vtree（mcp_server.rs:1560
  `build_aura_from_styled_vtree`）——快照=渲染树忠实反映；PRE-RENDER 回退
  有显式标注（排除）。即 **View::Column.onclick 在这些行上确为 None**。
- **测量教训入档**：阅读页 Init ~16-18s（books/toc/chapter/annotations 四段
  HTTP）——Contents 壳先渲染；+5s 快照=壳态（早期探针的"有 onclick"误读
  已纠正：7 个 onclick 全是工具栏钮）。
- **排除项**：route_id 空指针说（probe 证伪：back push 落正确 /book/:id，
  详情页在 probe-3 编辑链全绿）；theme 切换（theme-probe-2：暗/亮后 60s 内
  详情正常）；套间/父窗（T-01）；`[VM-INIT] … cancelled (generation
  superseded)` 为设计内 teardown-cancel（079 owner-filter 域）非本因。
- **根因定位（假设→下一轮验证）**：渲染投影层丢属性——内层 col 的
  onclick+三元数组 style 同时缺失而子内容正常。两条候选：
  ① auto-ui-compiler 模板投影对该构造（for 内 onclick 带循环变量载荷 +
  嵌套三元数组 style）丢属性——查编译产物模板即判；
  ② runtime view-builder 的 set_layout_events/extract_style 在该形态下
  失败——单测二分（for + scroll-anchor 外层 + onclick(p.idx) 内层）。
  注：events.rs:28 在案同族前科（"onclick: .SetAccent(\"indigo\") were
  dropped (param not in bindings)"已修字面量臂；循环变量臂未证）。
- **下一步（T-04c）**：查编译产物（UI cache 定位或 compiler 单测直投
  reading_page.at 抽取块）→ 定谳①/② → 修复 + 单元锚（for 载荷 onclick
  渲染断言）→ bookswitch_probe 复跑（tap_rows>0 为过）→ W-07 五叶。

### T-04c 二分进展（2026-11-12 晚，worktree commit 单元锚 ×2）

- **单元锚已落**（`aura_view_builder/mod.rs`，cargo test plan081 双绿）：
  ①`plan081_reading_row_onclick_survives_projection_render_and_probe`——
  reading_page ¶ 行最小结构全链：**A 编译器投影 ✓**（模板带 onclick 载荷
  p.idx）、**B 渲染 View ✓**（内层 Column.onclick=Some，经
  ScrollAnchorMark 包装——其 vtree 转换透传，PLAN-781 已在 9dd3fc27）、
  **C probe 记录 ✓**（computed 面来源记录在案）；
  ②`plan081_reading_row_probe_survives_two_frame_timeline`——首帧空段落
  （Init park 形态）→ 帧 2 行出现，probe 记录完整。
- **定谳推进**：builder 单层（含两帧时序、scroll-anchor 包装、循环变量
  载荷）**全部清白**——缺陷只剩 outlet 页集成层三候选：
  ①outlet 页 memo 重放（`render_outlet_page_memo`，PLAN-045 缺省 on）的
  probe 再记录缺口；②outlet 子件 instance build 的 probe 路径合并；
  ③PLAN-725 提升帧构建（AbstractView 中间层）的探针旁路。
- **live 交互测（bookswitch-7，按 vnode id 直按内层 col）**：
  press 未触发 TapParagraph dispatch——但 MCP 对非 button 容器的 press
  语义可能本就不派发（歧义未消），不足以单独定谳。
- **下一轮执行序**：①outlet+memo 单测（App routes+registry+子页 widget
  +两帧，断言页内 for 行 probe 记录）复现→修复；②修复后 bookswitch 复跑
  （快照 `onclick: .TapParagraph` 在场为过）→ W-07 五叶（harness）；
  ③统一回归（review 面）。
- **回归锚现状**：plan081 双测=已清白层的永久锚；outlet 层修复后补
  对应锚。

### T-04c-2 复现器落地（2026-11-12 深夜，worktree b1f934f7）

- **红锚入册**（`#[ignore]`，reason 在案）：`plan081_outlet_page_for_rows_
  survive_memo_two_frame_timeline`——App081(routes+outlet)×reader081 子页
  （¶ 行结构）两帧时序：帧 1 段落空（memo 因 for_loop DEGRADE→page_full
  全渲染，0 行）；帧 2 段落写子页实例态（route 装饰 identity 经
  `child_init_identity`+`instance_identity_override` 对齐
  `prepare_child_render_state_snap`，`write_state_onto(child_state_id,...)`）
  → **probe2 全空（entries=0）=for 零迭代=子读仍见旧段落**。
- **排除**：memo 非凶手（两帧均 DEGRADE reason=for_loop → page_full 全渲染，
  MEMO-DIAG 实证）；builder 单层（锚①②）；容器 press 歧义保持记档。
- **下一轮第一步（精确）**：帧 2 子读为何不见写入——三个探查点：
  ①`prepare_child_render_state_snap_impl` 的 identity 归一（我的 prepare
  直调 vs 渲染路径 pinned 调用是否同对象）；②子读 scope 链
  （`slot_resolution_override`/`override_state_obj_id` 在 render_outlet_
  inner→page_full→child walk 三层的传递）；③`write_state_onto` 的
  stake/seq 面（seq 抬升是否落子对象账）。诊断打印已埋（plan081-diag）。
- **说明**：红锚 fail 属 TDD 红相（预期失败=证据）；修复转绿后删 ignore。

### T-04c-3 机制实测（同日晚，prepare 插桩轮）

- **实测机制**（MEMO-DIAG prepare 插桩，两帧六次 prepare 全录）：
  - 渲染路径 identity=**`reader081@/reader`**（路由装饰）→ 子状态对象
    **4000002**，其 `paragraphs` 字段=**VmRef(4000001)**（step-3 父同步把
    根数组引用传入——根上数组恒空，子写若落自己对象则与根脱钩）；
  - 测试侧 `child_init_identity()` 直调返回**未装饰 `reader081`** → 对象
    **4000003**——**写读对象错位即帧 2 空迭代直接原因**；
  - `child_init_identity` 同参不同果（渲染期装饰/直调不装饰）=状态ful
    判定，其输入面（__current_route 已 seed 仍不装饰）为下一步唯一疑点。
- **下一步（T-04c-4，唯一任务）**：读 `child_init_identity` 实现，解释
  装饰判定的状态依赖 → 测试侧以渲染同径取对象（或产品侧修 identity
  归一）→ 红锚转绿 → bookswitch 快照复跑 → W-07。
- **边界提醒**：live 失败态（行内容渲染但 computed 缺失）与红锚帧 2
  （零迭代全空）形态不同——转绿后若 bookswitch 快照仍缺 computed，
  下一嫌疑=PLAN-725 提升帧/bounds 回路，勿混两案。

### T-04c-4 outlet 层清白定谳（同日，worktree 提交）

- **红锚转绿**：`plan081_outlet_page_for_rows_survive_memo_two_frame_timeline`
  转 green 常驻锚（第三锚）。红相根因=**测试台三项伪影**（非产品缺陷）：
  ①直调 `prepare` 未套 `OUTLET_INIT_IDENTITY` 守卫（outlet 渲染期
  thread-local，装饰 identity 的唯一来源）→ 写读对象错位；②子字段段落=
  根数组 VmRef——写必须穿引用原地改根数组（对齐 VM SET_FIELD 语义）；
  ③子实例事件携带实例身份=`DynamicMessage::TypedInst`（PLAN-041），
  测试匹配器须双型接受。
- **累计清白面**：编译器投影 / 渲染 View（含 TypedInst/outlet 子件/
  ScrollAnchorMark 透传）/ probe 记录 / 两帧时序 / outlet+memo 集成——
  **全部单元级清白**（plan081 三锚常驻）。
- **live 缺陷（行内容在、computed 缺失）剩余嫌疑面**（唯一未测层）：
  ①PLAN-725 提升帧构建（frame_build/AbstractView 中间层的 probe 路径）；
  ②`__bounds_collected` from_live 覆盖（debug 门——harness 无 F12，疑可
  排除但未证）；③MCP autoui_action 对容器 press 的语义（歧义在案）。
- **测试台教训入档**（后续锚作者必读）：outlet 子件探针断言须 (a) 套
  OUTLET_INIT_IDENTITY 守卫、(b) 经 VmRef 语义写状态、(c) TypedInst 双型
  匹配；阅读页 Init ~16s（四段 HTTP）——壳先渲染，行判定须等
  `onclick: .TapParagraph` 在场而非 Contents。
- **下一步（T-04c-5）**：PLAN-725 提升帧 probe 路径核对（projection.rs
  frame_build 来源的 probe 与 build_with_debug 的 path 空间差异）——
  若提升层丢记录即 product fix 位；随后 bookswitch/W-07 复验。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-081 | plan_revision: r1 | outcome: pass
  （可交 work；T-01/T-04 为有界调查带决策工件） | 起草：READER-009 会话
  （round49 先例 + round54 方向确认） | next: work（auto-ui 会话接手，
  T-01 插桩定谳起步）。
- stage: new | plan_id: auto-ui/PLAN-081 | plan_revision: r2 | outcome: pass
  （范围收窄修订：用户裁定"只留 D3"——D1/D2 以 T-01 仪器化证据收束
  （16/16 行为正确、MTA/错窗排除、mode B 81-200ms 快返签名存档、探针
  3227af96 常驻）；T-02/T-03 撤销；AC-01/02 收束、AC-03/04 继续；
  AutoUI 组件方向另议） | 修订：auto-ui 会话（用户 2026-11-12 裁定） |
  next: work（T-04 D3 复现定谳起步）。
- stage: work | plan_id: auto-ui/PLAN-081 | plan_revision: r2 | outcome:
  进行中（T-04c 二分落定一半：**builder 单层全清白**——编译器投影/渲染
  View/probe 记录/两帧时序/scroll-anchor 包装五个维度单元锚全绿
  （plan081 双测入册）；缺陷隔离到 outlet 页集成层（memo 重放/子件 probe
  合并/P725 提升帧）三候选）| worktree: D:/autostack/.wt/ui-081/auto-ui |
  branch: codex/plan-081-dev | base_commit: 9dd3fc27 | code_commit:
  3227af96+b29af67e+锚测试 commit | dependency_revisions:
  auto-lang@0b3fc9063/auto-down@58ea33f/auto-web@bf72baa（组只读）+ 驱动
  8d7dc2cf + 仪器工具 da154bfa | task_ids: T-04 | evidence:
  plan-081/{bookswitch-7（live 按 vnode 直按未派发，MCP 容器 press 语义
  歧义）} + 单元锚两测 | blockers: outlet 集成层单测（routes+registry+子页
  +两帧）未写——下一轮首步 | next_responsible_role: work | next: work。

## 10. 待澄清事项

- ~~mode A 自毁与 evening commits 因果~~（r2 收束：探针常驻，未复现，
  归因环境/用户在场上下文；再现时 AUTO_DIALOG_DIAG=1 即时定谳）。
- ~~D3 S 区修复 diff 的提取源~~（r1 考古定谳：7d3a73aa 全线为 master 祖先，
  无缺失 diff；实质=round-19 路由域缺陷未实施——T-04 直修）。
- AutoUI 自制文件选择器组件：后续计划未立项——009 picker 八叶与 native
  对话框可操作性死结的最终解，owner=用户裁决后立项。
- 若 D3 根因落在 auto-lang 段记账交互（bisect 辅项），升级为跨仓协调。

> 编号注记（2026-11-12）：080 与并行会话的 080-age-node3d 冲突（本计划先
> 提交 10:46，对方 11:08 误占同号）——本计划重编号为 081，080 归 AGE 域计划。
