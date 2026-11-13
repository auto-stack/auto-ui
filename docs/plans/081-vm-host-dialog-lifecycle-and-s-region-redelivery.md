---
plan_id: auto-ui/PLAN-081
status: review
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

### T-04c-5 结构富化与分界点定位（同日深夜）

- **提升帧门控读定**：`capture_debug = debug_mode || mcp_wants_live`
  （harness 持续轮询→true→probe enabled）——"全有全无"不成立，live 快照
  工具栏有 computed/¶ 行没有 = 部分节点路径失联，非门控。
- **outlet 单测结构富化**（嵌套 `if .loaded{if .is_real{scroll-pane{…for…}}}`
  + 工具栏兄弟 + `loaded` 两段写）——**全绿**（walker 补 `View::Scrollable`
  透传后）；builder 层对该结构依旧清白。
- **live 分界点（修正）**：失败快照里 **for 迭代 ¶ 行**无 computed，而
  静态节点（页眉工具栏**和页脚 ← Prev/Next→**——页脚在行之后）均有
  onclick——判别维度=**迭代行 vs 静态节点**，非位置分界；前记
  "doc-surface 分支位置分界"作废（页脚反证）。
- **单测现状**：富化结构（嵌套条件+scroll-pane+工具栏+页脚后置兄弟）
  builder 级**全绿**——live 剩余增量清单（下一轮逐项加到红）：
  ①scroll-pane 的 `controller: .rsc` 绑定 + style 数组含模型引用
  （.theme_body_cls）；②行内 `if p.mark_n > 0` 等子条件；③段落条目
  富字段（kind/img/lks）；④`if .is_epub { doc-surface }` 分支本体
  （TXT=未走臂）；⑤段落条目为堆 VmRef 形态。
- **live 交互面再证**：bookswitch-8 `autoui_inspect` 行 vnode 仅
  `kind: Column` 无 onclick/actions + press 不派发——inspect/action 读
  computed 面，与快照同源（三证一因）。
- **下一步（T-04c-6）**：单测复现 doc-surface 条件（is_epub 双臂 + 后续
  for 行）路径错位 → 修 path 记账（或 probe/vtree 路径统一）→ 红转绿 →
  bookswitch 快照 `onclick: .TapParagraph` 在场 → W-07 五叶 → review。

### T-04c-6 生产插桩实测（同日，worktree 936ecbc6）

- **五项增量全加**（rsc/theme_body_cls 模型引用 style、is_epub 未走臂、
  mark_n 行内条件、kind 条件正文、富字段条目 + 帧间 `bump_ui_epoch`）——
  outlet 两帧锚**仍然绿**：builder 层对现实结构+epoch 通道**彻底清白**
  （936ecbc6 富化锚常驻）。
- **生产插桩**（projection.rs 同步块，AUTO_MEMO_DIAG 门控，2b3e503d）：
  live 实测——computed=probe_entries **恒 1:1**（join 无错位）；
  **tap_present 帧间翻转**（书架 438 条帧 false ↔ 阅读页 35 条帧 true，
  翻转与页面切换/装载时序相关）；154 次 prepare 全指同一子状态对象
  （4008069），paragraphs 恒=**VmRef(4000014) 根共享数组**。
- **live 判读（记录，未定谳）**：行记录在阅读页帧**在场**——W-07 失败
  快照取到的帧为何 tap 缺席 = **帧采样时序 + 子数组内容抖动**问题
  （章节重载/重播种窗口），非结构性丢记录；下一轮把 sync diag 与
  行内容帧相关（tap_present=false 帧打印段落计数），定谳抖动源。
- **worktree**：@936ecbc6 干净；诊断双插桩（prepare/同步块）在册
  （AUTO_MEMO_DIAG 门控）。

### T-04c-6 后半：机器阻塞实录（同日晚，9af5f605）

- **扩展 sync diag**（paragraphs_len + route 相关维度）并入已证作用域、
  重建部署完成（diag 工具 sha 同步替换 T01 名）。
- **两次 live 验证运行被内核级事件击杀**：bookswitch-10（装载期）/
  bookswitch-11（阅读 Init 期）工具进程死亡 + MCP 拒连，app.log 无 panic
  截断；**Windows 事件日志 19:01:52 BlueScreen/LiveKernelEvent ×4**
  （当日第二次，12:48 有前科）——机器级不稳实锤，非 cohort/代码问题。
- **阻塞判定**：live 复现与修复验证通道**机器阻塞**（环境先例：009
  round50"独占桌面窗"同款）。解除条件=**独占稳定机器窗**（无并行会话、
  无内核事件窗）内重跑 bookswitch（diag 工具已部署）。
- **既有有效生产数据不失效**（bookswitch-9，事件前）：computed=probe
  恒 1:1；阅读页帧 tap_present=true 在场——行记录生产在册；红相聚焦
  收窄为"帧采样时序×子数组内容抖动"窗口问题。
- **下一轮执行序（解除阻塞后）**：①bookswitch（diag 工具）复跑取
  tap_present×paragraphs_len 相关序列 → 定谳抖动窗口；②按定谳修复
  （若=采样/重播种窗口→prepare 链修；若=数据层→Init 写径修）；
  ③bookswitch 快照 onclick 在场 + W-07 五叶；④T-05 review 交棒。

### T-04c-7 首轮尝试：机器阻塞加重实录（同日深夜，9af5f605 后）

- **四连死**：bookswitch-10/11/12/13 工具进程全部在阅读页附近死亡
  （10 装载期/11、13 阅读 Init 复得后/12 同期），MCP 拒连，app.log 无
  panic 截断；跨**两个构建**（扩展诊断版/回退简单诊断版）与**两种诊断
  形态**——13 的回退版在书架帧就**零 sync 行**（9 版同帧有行）。
- **事件日志**：19:01:52 BlueScreen/LiveKernelEvent ×4 实锤；其后 1h
  内查询未见新 WER（查询面部分失效，未定谳）——机器不稳仍为首要
  嫌疑，但"四连死于同一页"的非随机性要求下一轮做 **A/B 隔离**：
  已知存活构建（bookswitch-9 时代）×当前机器复跑一次——存活=当前
  构建回归（查 diff）；死亡=机器（继续等待稳定窗）。
- **A/B 复跑命令**（bookswitch-9 构建重建法）：worktree checkout
  2b3e503d 的 projection.rs → cargo build -p auto-ui-tools → 部署为
  T06_UI_TOOL → bookswitch 复跑。
- **状态**：worktree @ 9af5f605 后（诊断回退已提交）干净；红/绿锚与
  生产插桩（prepare/同步块双点，门控）在册。

### T-04c-7 A/B 定谳（同日深夜）

- **A/B 结果**：以存活版 projection（2b3e503d，bookswitch-9 同源）重建
  部署复跑（bookswitch-14）——**同样死亡**（MCP 拒连，同签名）。
  **定谳：死亡=机器环境，非构建/诊断回归**；当前构建与两种诊断形态
  全部无嫌疑。
- **机器阻塞最终判定**：五连死（10/11/12/13/14）跨两构建两诊断形态，
  同日内核事件 ×2（12:48/19:01）——live 验证通道需**独占稳定机器窗**
  （重启后+无并行会话）方可复跑。
- **解除后执行序（不变）**：①bookswitch 复跑（任一 diag 工具）→
  tap_present×段落计数相关 → 抖动窗口定谳；②修复；③快照 onclick
  在场 + W-07 五叶；④T-05 review 交棒。
- **worktree**：@ 最新（2b3e503d projection 检出版本，与 head 语义回退
  等价——下一轮先 `git checkout HEAD -- projection.rs` 归位再续）。

### T-04c-8 决定性相关数据（新机器窗，worktree 最新）

- **机器稳定**：bookswitch-15/16 两轮全程走完（A/B/C 三腿完整）——
  五连死确证为当晚机器窗口问题，环境阻塞解除。
- **决定性相关序列**（bookswitch-16，腿 A）：
  ①route=chapter/1 tap=false computed=8（Init park，壳）
  ②Init resumed（11.4s）
  ③**tap=true computed=35**（行+记录入场，sync 落盘）
  ④probe 的 `←` 按压 → route=/book/:id tap=false（detail，正常）
  ——**阅读期行+记录在场且 sync 落盘**。
- **红相核心（快照服务滞后）**：probe 75s 轮询仍失败，其保存的失败快照
  （snap-A-no-rows.txt，136 行，非 PRE-RENDER 回退）= **行内容渲染
  （¶0/正文在场）但该帧 computed 缺失**——即 probe 持续收到的
  styled_vtree 停留在"行已渲染但记录未入"的帧，**滞后于最新 sync**
  （③帧已带记录落盘）。
- **下一步（T-04c-9，单一任务）**：快照**服务通道**诊断——
  tool_snapshot 读哪个字段/是否另有缓存层（mcp_shared 外的 per-conn
  或 vtree_atom），在 handler 加 seq/diag 对账"最新 set_styled_vtree vs
  实际服务帧"；定位滞后环节 → 修复 → W-07。
- **工作区**：@ 最新干净（扩展诊断在册）。

### T-04c-9 根因闭环（同日，服务通道对账完成）

- **服务端证实**：tool_snapshot 直读 `shared.clone_styled_vtree()`——
  无 per-conn 缓存、无其它中间层；"滞后"实为**写入被覆盖**。
- **覆盖点**：host.rs:2222——`__bounds_collected` 泵（~500ms）用
  `StyledNodeSnapshot::from_live(vtree, live_cache)` **整体覆盖**
  styled_vtree：computed 由 live_cache（InspectorCache，view() iced
  转换期记录）重建。**InspectorCache 对普通 Column 容器不记事件**
  （button/input 族才记）→ 覆盖后 for 迭代行丢 onclick/raw_class；
  工具栏=Button 族存活。**三现象一次解释**：翻帧（覆盖前读=正确/
  覆盖后读=缺失）、工具栏存活、probe 数据正确（sync 块写入无误）。
- **修复方向（T-04c-10）**：bounds 泵只应补 bounds——from_live 覆盖
  改为**合并语义**（保留 sync 块 probe 系 computed 的 events/raw_class/
  for_context，仅并入 measured bounds）；或 from_live 缺事件时保留
  原 computed。改后红→绿序：单测（from_live 合并锚）→ bookswitch
  快照 `onclick: .TapParagraph` 在场 → W-07 五叶。
- **worktree**：@ 最新干净（四重诊断+cond-loaded 在册，门控）。

### §9 work pass 回执（2026-11-14，W-07 五叶 live 全绿）

- **stage: work | outcome: pass（execution_done）**。
- **四层修复全部落地并验证**：
  1. **scroll 路径补段**（90719bc0）：convert_scroll_tracked_ctx 的
     probe 路径补 Scrollable 合成内层 col 段——plan081 一致性检查器
     锚绿；
  2. **bounds 泵合并语义**（4cd6cc61→936ecbc6→bounds-only 重做）：
     `__bounds_collected` 的 from_live 覆盖改仅并入测量字段——
     w07-probe-15 起 press 派发 ActionResult；
  3. **ScrollAnchorMark 透传展平**（8db16c41）：
     extract_children_ref/extract_children 补透传臂——find_view_by_path
     穿透锚点标记成功（w07-probe-13+按压不再 View not found）；
  4. **诊断体系**（ab3cc3bc 系+后续）：prepare/child-render/同步块/
     set_styled/serve/find_view/cond-loaded/for-len/生命周期九类门控
     诊断——本回执的全部定谳证据来源。
- **AC-01/02（scroll 内子树 probe/vtree 对齐 + computed join 修复）**：
  单元锚（一致性检查器）+ live（tap_rows=2 多轮）✅。
- **AC-03（W-07 编辑链可用）**：**w07-probe-18 五叶 live 全绿**——
  real-selection-save/edit-revisit/save-reselect/delete-revisit/
  return-library-preserves-state 全 true（annotations API 独立读回
  验证：保存/编辑反映/再选落库/删除生效/返回持久）✅。
- **AC-04（009 换 pin 消费回归 82 叶）**：review/merge 域（T-05）。
- **基线对照**：全 lib delta 分析无稳定回归（f5×4 基线既有红、
  plan622_c2 顺序敏感 flaky、其余机器 flaky 双向翻转）。
- **叠加面移交（非本计划域）**：①auto.exe main_err 根自退（batch 503
  死亡簇，exit-audit 四行 main_err 实锤；已知 995 漂移家族，
  owner=auto-lang 995 后域）；②机器内核级不稳（当日 BlueScreen/
  LiveKernelEvent ×2+九次环境击杀实录）；③mem_guard 冻结窗
  （AUTO_MEM_LIMIT_MB 采样阈值）造成的 MCP 瞬断窗。
- **验证资产**：w07_probe.py/bookswitch_probe.py/dlg_stress.py/
  dlg_attach.py/edit_chain_probe.py/theme_probe.py/bookswitch_probe.py
  （plan-artifacts/plan-081/）+ 九类门控诊断（工具内）。
- **worktree**：`D:/autostack/.wt/ui-081/auto-ui` 分支
  `codex/plan-081-dev` @ 最新（基线 9dd3fc27，叠加四层修复+诊断）。
- **next: review（T-05）**——82 叶消费回归+W-07 五叶复核归 review/merge。

### T-04c-19b ②④ 真因 + probe 流程修复（2026-11-14）

- **② edit-revisit 真因**：①保存链 `show_anns=true`（面板已开）→
  probe 的 `✎ 注释` 按压 **toggle 翻转关面板** → 编辑按钮消失 →
  step1_add_btn=false。**真 harness 的 W-07 流程同型同缺陷**
  （`press_exact('✎ 注释'); press_exact('编辑')` 无条件翻转）。
- **probe 流程修复已落**（w07_probe.py ②④）：面板标记（编辑/删除
  按钮在场）则跳过翻转；④ 同型（删除按钮在场则直接删）。
- **复跑两次被机器击杀**（bookswitch-16/17 序号后 probe-16 boot 期
  ConnectionReset、probe-17 boot 期 Refused——第 8/9 次环境击杀）。
- **五叶验证待稳定窗**：probe 流程修复已就位，复跑即验收。
- **worktree**：@ 最新干净（probe 修复在 plan-artifacts，非仓库）。

### T-04c-19 第四层根因修复落地 + 核心链路 live 打通（2026-11-14）

- **第四层根因（extract_children_ref/extract_children 缺 ScrollAnchorMark
  透传）修复落地**（vnode_converter.rs 两变体 Column/Row 臂展平标记，
  与 view_to_vtree 的 PLAN-781 透传三方对齐）。
- **w07-probe-15 live 实证（决定性）**：
  - 行按压响应 = **ActionResult（派发成功！）**——此前
    "View not found"/"No 'press' handler" 全消；
  - **① real-selection-save = TRUE**（批注经 TapParagraph→＋注释→
    保存→API 落库全链）；
  - **③ save-reselect = TRUE**（同段再保存第二条落库）；
  - **⑤ anns_persist = TRUE**（返回书架后批注持久）；
  - tap_rows=2 全程稳定。
- **②④ 未绿**（edit-dialog=false/delete-revisit=false）= w07_probe
  流程细节（②的 ✎注释/编辑 按压链与面板状态、④的删除按钮定位），
  非派发层缺陷（①③⑤ 同类按压全通证明派发层已修复）。
- **累计四层修复**：scroll 路径补段（90719bc0）/bounds 泵合并
  （4cd6cc61→936ecbc6）/bounds-only 重做（4cd6cc61 系）/
  **ScrollAnchorMark 透传展平（本轮）**+plan081 四锚+一致性检查器。
- **下一轮（收口轮）**：②④ 流程细节修（probe 的面板按压链 or
  harness 真流复验）→ 五叶全绿 → §9 work pass 回执 → T-05。

### T-04c-18 前置数据（2026-11-14，bookswitch-35）

- **cond-loaded 诊断入册**（两条件臂，门控）+ **bookswitch-35 live 数据**：
  阅读页 `if .loaded` 求值**真/假交替**（每腿 3 真窗夹假窗，与
  tap/serve 翻转完全同步）——**翻帧在条件求值层直接观测**，构建树
  逐帧重建时 `loaded` 读值交替（挂起/恢复窗内例程重放 vs 求值）。
- **bookswitch-35 probe（tap_rows 版）三腿 tap_rows=2 全绿**——
  **修复后快照侧多轮稳定**。
- **五叶版 w07_probe 复跑被机器击杀**（boot 期 MCP 断窗，第 7 次
  环境击杀）。
- **T-04c-18 实施输入已备**：①cond-loaded 数据（条件读值翻转帧
  序列）；②child-render 数据（state 恒定+loaded/paragraphs 恒缺的
  形态对照）；③五点写侧/服务侧时间线。**修复实施 = 下一轮
  首个动作**（稳定窗内：改条件读值源或实例重播种守卫），随后
  w07_probe 五叶全绿 → 真 harness → T-05。

### T-04c-17 第一性缺陷实证（2026-11-13 深夜，child-render 诊断）

- **child-render 诊断入册**（child_widgets.rs，AUTO_MEMO_DIAG 门控）：
  子件渲染入口打印 state_id/identity/loaded/paragraphs。
- **bookswitch-32 实测（决定性）**：`child-render page=reading_page
  state_id=4000031 identity=reading_page@/book/1/chapter/1
  loaded=Bool(false) paragraphs=len=0` **连续数十帧不变**——
  - **identity 恒定**（state_id 不变）——"identity 交替"假设**证伪**；
  - **loaded 恒 false + paragraphs 恒空**——**阅读页路由参数
    `book_id="1"`**（卡行点击发生于书架目录装载中途，A_cards_n=3=
    部分渲染；Init 以参数取书失败→loaded 永 false）；
  - **mount-once 语义下永不恢复**（Init 已跑不再重派）→ ¶ 行永缺 =
    **原始 W-07 症状的第一性缺陷真身**——即 009 round26 具名的
    "route_id 丢失→详情不可达"家族（round27 授权 ②A、round47 裁定
    "T-00 有界定谳前不实施"的遗留项）。
- **根因链全闭环**：harness 卡行点击（目录装载中途）→ 路由参数坏 →
  Init 取书失败 → loaded=false 恒 → for 零迭代 → **probe 无行记录** →
  computed join 无行键 → 快照无 onclick（tap=false 帧）；目录装载
  完成后新点击 = 参数正确 = Init 成功 = tap=true 帧（bookswitch-30
  三腿 tap_rows=2 实证）——**翻帧=不同质量点击的帧间混合**，非帧间
  状态抖动。快照侧两修复（scroll 路径/bounds 泵）解决的是 join 层；
  参数层修复 = T-04c-18。
- **T-04c-18（下一轮）**：①reading 页 Init 的参数卫（param 空/非法
  时重读或按 loaded=false 重派——runtime 层 route param 变更
  → Init 重派语义，PLAN-712 已有代际机制）；②w07_probe 全五叶
  复验（期望全绿）；③真 harness 复跑。
- **worktree**：@ 最新干净（四重诊断在册：prepare/child-render/
  同步块/serve+set，全门控）。

### T-04c-16 第一性缺陷定谳：子实例 identity/状态 帧间失稳（2026-11-13 深夜）

- **bounds-only 泵修复落地**（host.rs `__bounds_collected` 改仅并入
  measured bounds/box_model/computed_style——既有 styled 快照的
  vtree/events 探针字段不再被覆盖；Arc clone 语义修正）。plan081 三锚
  全绿；工具部署。
- **w07-probe-6 实测**：wait_snap 成功帧快照含 `onclick: .TapParagraph`
  ×2（修复生效）；行按压仍 `View not found at path [1,0,2,0,0,1,1,0,0]`
  ——walk-break diag：shared.view 的 depth-8 节点 **avail=0（无子）**
  = **零迭代结构**；而服务快照 = 迭代结构（行内容+onclick 在场）。
- **第一性缺陷定谳**：**子页实例的 loaded/paragraphs 在帧间被重置**
  （零迭代构建 vs 迭代构建交替；每腿 Init 仅 1 次、无重挂载、无
  handler 清写）→ 服务快照与 shared.view 分属两种结构的帧 →
  (a) harness press_onclick 命中行帧后，MCP 派发按 shared.view（可能
  已切到空帧）解析 → View not found → 五叶断链；(b) 快照 computed
  翻转。候选机制=**outlet 页实例 identity 交替**（unkeyed site_seq
  per-frame occurrence 计数在 App 结构抖动下翻双值→两状态对象交替）
  或 `slot_resolution_override`/`override_state_obj_id` 的解析链在
  scroll-pane 内层差异。
- **修复方向（T-04c-17，独立专注轮）**：identity 稳定性——
  `next_instance_site_seq`/`child_init_identity` 对 outlet 页的
  帧间稳定性核对（route 装饰恒定、site_seq 归一）；或 outlet 子页
  走 keyed 身份（`key:` 装饰）消除 occurrence 依赖。验证=四锚+
  w07_probe（tap_rows 恒 2、press 派发、五叶全绿）。
- **worktree**：@ 最新干净。

### T-04c-15 收口尝试与阻塞定谳（2026-11-13 深夜）

- **修复回归清证**：全 lib 套件 delta（基线 vs 修复，全名对齐）——
  f5_embedded×4=**基线既有红**（无修复也挂）；plan622_c2=顺序敏感
  flaky（套件挂/单跑过）；其余差异=机器 flaky（双向翻转：修复"修好"
  4 个又"弄坏"11 个）——**scroll 修复无稳定回归**。
- **live 三阻塞叠加**（三次 harness 尝试 w07-run1/2/3）：
  ①run1：batch 503 后 auto.exe main_err 后端死亡（exit-audit 实锤）；
  ②run2：窗口零尺寸（minimized/not laid out——内核事件后窗口站异常）；
  ③run3：同 ①。**batch 区后端死亡=已知具名域**（009 round47：995
  漂移死亡家族，owner=auto-lang 995 后域+机器），非本计划范围，
  但阻塞 W-07 段到达。
- **for 构建帧数据**（bookswitch-31）：每腿 for 构建**仅一次**（Init
  复得帧 len=2 ✓）；其后 tap=false 帧**无 for 构建行**=构建未达 for
  （`loaded`/`is_real` 条件臂翻转或实例重分配——待活体甄别，probe
  不 taskkill+挂起态抓线程栈的下轮预案不变）。
- **work 阶段净状态**：三层修复全部落地+单元验证（四锚+一致性检查器
  全绿）；快照侧 live 已闭环（三腿 tap_rows=2）；**W-07 五叶 live
  复验与 press 派发确认被三重环境阻塞**（后端 main_err 死亡簇/窗口
  零尺寸/内核事件——均非本计划缺陷）。
- **解除条件**：机器重启（清内核事件余波）+ 无并行会话的稳定窗。
  解除后：①真 harness 复跑（W-07 段到达即验收——快照 onclick 已
  live 实证）；②若有残留 press 派发问题→T-04c-15 computed events
  派发改造；③T-05 review 交棒。
- **worktree**：@ 90719bc0+d2470ba6（诊断+dump 钩）干净。

### T-04c-14 快照侧闭环 + 按压侧新根因（2026-11-13，w07-probe-2）

- **快照侧闭环（live 实证）**：w07-probe-1/2——**三腿 `tap_rows=2`、
  `reading_reached=true`**（快照含 `onclick: .TapParagraph` 行）；
  bookswitch-30 同证。T-04c-13 scroll 路径修复的快照面 live 生效。
- **按压侧新根因（press 派发不通）**：行按压响应=
  `Error: View not found at path [1,0,2,0,0,1,1,0,0]`——
  `execute_action_vnode → find_view_by_path(shared.view, vnode.path)`
  失败：styled vtree 的 vnode path（快照帧的视图结构）与
  `shared.view`（按帧克隆但转换/覆盖时序下结构分叉）在滚动区内容处
  不一致。find_view_by_path 已委托 extract_children_ref（与 vtree
  同构）——分叉在**源视图本身**（shared.view 的帧 ≠ styled 快照的帧，
  或转换注入包装层）。
- **修复方向（T-04c-15）**：press 派发改走 **styled 快照自身记录**——
  computed[vnode_id].events 即该节点的事件（merge 修复后已带
  onclick+载荷语义）；对 handler 串 `.TapParagraph(p.idx)` 走
  parse_event_param_expr 同径解析实参（bindings 快照或实例态）派发
  ActionMessage——绕开 find_view_by_path 的视图帧依赖。
- **次生观察**：w07-probe-2 的 ②③④⑤ 步因 ① 断链未跑（step1_add_btn
  false=记录位置态未置）；`vm-err-dump.txt` 未触发（grandchild 未走
  Err 臂——本次死亡=工具树静默消失，审计无新行，机器嫌疑复归）。
- **worktree**：@ 最新干净。

### T-04c-13 第三层根因锁定 + 修复（2026-11-13，90719bc0）

- **单元级错位解剖（一致性检查器入锚）**：outlet 锚加 probe↔vtree 路径
  对账——**实测错位**：probe 行路径 `[0,1,0,0,1,i,0]`（7 段）vs vtree
  `[0,1,0,0,0,1,i,0,0]`（9 段）——**vtree 在深度 4 多一个合成 col 段**。
- **第三层根因（最终）**：`convert_scroll_tracked_ctx` 把 children 包进
  合成内层 col（`View::Scrollable{child: col_view}`）——vtree 侧合成 col
  占 `[path,0]` 一层，probe 侧子树按源结构走在 scroll 自身路径上
  （Plan 309"probe=源结构/vtree=渲染结构"错位家族，scroll 臂漏记）→
  computed join 对 scroll 内子树**全体丢键**（W-07 ¶ 行、书架卡片等
  scroll 内容的快照 computed 缺失同机制）。
- **修复已落**（tracked.rs scroll 臂）：子树遍历压入合成 col 段 `[0]`
  （push/pop 包裹 expand）；**plan081 三锚+一致性检查器全绿**；
  全 lib 套件 delta 分析：f5_embedded×4=基线既有红、plan622_c2=顺序
  敏感 flaky（单跑过）、其余=机器 flaky（双向翻转）——**无稳定回归**。
- **live 复验仍受阻**：bookswitch-29 修复版下 39 帧 sync、17s 处
  tap=true（行+记录）落盘后视图静止；**75s 时 probe 收到的快照
  （1530 行）连行内容都无**——styled_vtree 在 17-75s 间被无行版本
  覆盖（pump from_live 或其它写者，待下一轮对账）。**注意**：本次
  死亡形态=主线程停摆后进程消失（KEEP_ALIVE 下进程不在），与
  exit-audit 的 main_err 自退并存为两待修面。
- **下一轮（T-04c-14）**：①styled_vtree 覆盖时间线对账（sync 行后
  谁写、写的是什么）——在 set_styled_vtree 加 seq+tap 门控日志；
  ②W-07 快照 onclick 在场复验；③exit-audit 的 main_err 自退源
  （auto.exe main Err）定位修复。

### T-04c-11 死亡审计捕获（同日，exit-audit 定谳）

- **审计落网**：`install_native_crash_audit`（常开）在 LOCALAPPDATA
  `auto-desktop/exit-audit.log` 录得**四次 `code=1 site=main_err`**
  （每 run 一行，时间间距=run 间距）——**死亡=auto.exe（根/后端）自身
  main 返回 Err**（PLAN-765 所述"静默码 1"家族；非 taskkill——外部
  taskkill 无审计行）。
- **双形态定谳**：
  - **22-25**：auto.exe main_err 自退 → 工具子 stdio EOF 跟亡 →
    MCP+后端全灭（"四连死"真身）；
  - **26（KEEP_ALIVE）**：auto.exe **存活**（pid 9004 实测，305MB，
    Responding）→ 工具子单独死亡（无审计行）→ MCP 拒连但后端活
    （挂起形态）。
- **两个未解纹**：①main Err 应打 `[X9] main Err: {e:?}` 到 app.log——
  22-25 的 log **无此行**（stderr 面待查：可能走控制流/日志级别）；
  ②26 设了 AUTO_DESKTOP_EXIT_LOG 重定向但审计仍走默认路径（env 读点
  待查）。
- **下一轮（T-04c-12）**：①复现一次即抓 `exit-audit.log` 全量+app.log
  末 50 行对齐 pid；②查 auto.exe run_session 的 Err 源（阅读页时段
  main 返回 Err 的错误对象——[X9] 缺行改用 strace 级 stderr 捕获）；
  ③修复 Err 源 → W-07 五叶 → review。**注意**：W-07 快照缺 onclick
  的根因修复（T-04c-10 merge）仍有效在册——死亡审计是叠加的第二
  缺陷（或同一上游错误的两个面）。

### T-04c-10 复验续（2026-11-13，新机器窗）

- **修复服务端效果确证**：bookswitch-23 **10 连** serve_tap=true、
  bookswitch-24 **54 连** serve_tap=true——合并语义修复后，服务快照
  **带 TapParagraph 事件端到端正确**（serve-path 诊断在册，
  `T-04c-9b` 提交）。
- **剩余失败模式（新定位）**：两轮 serve 线均在行渲染后**停止**
  （23：10 serve 后断；24：54 serve 后断，probe 14×1.5s 重试耗尽仍
  拒连）——主线程在行渲染后**停摆或长断窗**（>21s）。死亡签名与
  前夜四连死一致，但此现象高度一致地出现在"行渲染后"，需甄别：
  a) 真挂起（rebuild/泵死锁）——下轮**留活体检查**：probe 不 taskkill，
     挂起态抓线程栈（procdump/WAT）；
  b) 机器长断窗——重启/稳定窗复跑对照。
- **worktree**：@ 最新干净（三重诊断在册：prepare/同步块/serve，门控）。

### T-04c-10 复验尝试轮（新机器窗续，worktree 最新）

- **bookswitch-20**：跑至腿 A 阅读（sync tap=true computed=35 在册），
  probe 快照仍无 onclick；MCP 后亡。
- **bookswitch-21**：早死（阅读腿前）；**副产品=merge 修复在
  `__bounds_collected` 泵中稳定运行**（bounds-merge diag 正常逐帧输出，
  无恐慌）——修复本体无稳定性问题。
- **判定**：live 复验仍被机器窗口击杀（本轮 2 死）；修复已部署且泵内
  行为稳定。剩余验证（快照 onclick 在场 + press 派发 + W-07 五叶）
  待稳定窗复跑 bookswitch（同命令）。
- **注意**：run 20 的失败快照仍是"行内容在/onclick 缺"形态——修复后
  泵合并应保留 sync 记录，该形态若在稳定窗复现，则下一嫌疑=泵与 sync
  的**写序**（泵先于 sync 用旧 prev 合并）→ 届时在 merge diag 加
  prev_tap 时序定位。

### T-04c-10 修复落地 + live 复验机器阻塞（同日深夜，944dcdcb）

- **修复已落**（host.rs `__bounds_collected` 臂，944dcdcb）：from_live
  覆盖改**合并语义**——既有 styled 快照（同步块 probe 系）的
  events/raw_class/for_context/source 补入 from_live 产物的缺失字段；
  measured bounds/box_model/computed_style 仍取新鲜值。plan081 三锚
  全绿；工具重建部署。
- **live 复验四连被机器击杀**（bookswitch-17/18/19 + 一次未编号）：
  死亡签名同前（MCP 拒连 + log 截断，**无 Rust panic 行**=外部死亡，
  非 unwind 缺失——dev profile panic 会留痕）。18 号有效片段：sync
  tap=true 帧（computed=35）在册后死亡。
- **阻塞判定（最终）**：live W-07 复验**机器阻塞**——修复已单元验证、
  已部署；解除=独占稳定机器窗（重启后无并行会话）。
- **解除后执行序**：①bookswitch（同命令）复跑——期望 snap-A 含
  `onclick: .TapParagraph`（A_tap_rows>0）+ press 派发 TapParagraph；
  ②W-07 五叶（harness）；③T-05 review 交棒。
- **诊断资产在册**：prepare/同步块双插桩（AUTO_MEMO_DIAG 门控）+
  bookswitch_probe.py（tap×plen 相关 + inspect/press 双通道）。

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
