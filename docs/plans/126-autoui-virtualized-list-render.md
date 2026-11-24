---
plan_id: PLAN-126
status: executing
feature_name: VM 动态视图的虚拟列表（VList）
author: [lead]
created_at: 2026-11-24
updated_at: 2026-11-24
plan_revision: 2
current_step: 4
total_steps: 8
depends_on: [auto-ui/PLAN-112]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/virtualized-list-rendering.md]
touched_goals: [GOAL-002]
---

# PLAN-126 VM 动态视图的虚拟列表（VList）

## 0. 变更摘要

VM 动态 UI 当前会把进程表及其 `for` 展开为列、行和单元格。025 的典型样本有约 314 行、5 列，即约 1,570 个表体单元格。引入固定行高的纵向虚拟列表，AutoUI 组件名为 `VList`，Rust 类型名为 `VirtualList`。先用实际视口算出行区间，再从 VM 数据源读取、构建和布局区间内的行。025 复用已有页面滚动容器，另用一个普通列表验证同一组件可复用。

> T-01 实测修正（2026-11-24）：025 的 `sys_store.at:263` 在 `.ApplySort` 里把 `procsView` 钳到 120 行，徽标仍显示原始进程数（本次 324→328）。因此**渲染行数上限是 120 行**（600 单元格 / 1200 节点，占整页 1390 个元素的 86%），不是本段记的约 314 行；314 是 VM 列表长度。目标与验收不变，V-01/V-04 的样本与对照按实测基线登记。

## 1. 目标

- **G1** 以有限的纵向滚动视口为边界，只构建可见行及两侧各最多 2 行的预渲染缓冲区（overscan）。窗口外的总高度参与滚动条计算。
- **G2** 对已发布、可按索引读取的集合，固定视口下，纯滚动的 VM 取行、绑定求值、行构建和布局工作量随窗口内行数变化。空闲时复用运行时现有的帧缓存，窗口内行允许在需要构建时重新求值。
- **G3** 025 的行内容、点击选中、右键菜单、事件参数和表头排序入口保持现有 VM 行为。用带自定义行内容的普通列表验证组件可以脱离表格使用。
- **G4** 用同机 GL 对照量化渲染收益。纯滚动、数据刷新及消息泵响应分别取证，端到端改善达到 2× 为性能目标。
- **G5** 普通非虚拟列表与表格保持原行为。显式使用 VList 的固定行高场景在无有界纵向滚动视口或诊断开关关闭时，保留完整内容与交互。

首期范围为 AutoUI 的 Windows VM/Iced 路径、固定行高和纵向虚拟化。变高行及测量锚点、行列双向虚拟化、任意 `for` 的自动虚拟化、跨帧整行 `View` 缓存、远程按需取数和排序搬离 UI 线程各自独立评估。Vue/A2Vue、merged API 与 MCP 快照发布频率沿用各自现有合同。

## 2. 架构方案

- **渲染入口**。VList 接收行数、可按索引读取的数据源、行构建器和滚动状态。动态 VM 的入口需要在 `AuraNode::ForLoop` 展开前识别虚拟化的循环体。表体和普通列表用同一个窗口计算及按索引构建实现。
- **视口所有权**。纵向滚动容器保有偏移、视口和滚动条。Iced 的实际布局提供视口矩形和列表内容起点，二者使用同一逻辑像素坐标系。以视口和列表区域的交集计算行区间。
- **布局**。行数与固定行高相乘得出逻辑总高度。窗口外空间由总高度或上下至多两段占位表达，屏外行没有单独控件。表头、表前摘要等普通内容按既有布局计算，属于视口坐标换算的一部分。
- **状态与事件**。滚动状态属于容器实例，行内事件从当前行的数据和绑定生成。需要按键匹配的行状态使用可靠身份，数据、排序或视图实例更新时遵循 §5.2。
- **后端边界**。现有 `View::Table` 的 `table_resize` 仍逐格测量、布局。它提供列宽交互，本计划复用的是表体的行构建行为。PLAN-112 T-09 的片派发与本计划削减的视图工作量分别测量。

## 3. 技术栈

Rust 2021、仓库内补丁版 iced（T-02 实测版本为 iced/iced_core/iced_runtime 0.14.0、本地补丁 iced_widget 0.14.2；草案记为 0.13 系笔误，合同语义不变）、AutoUI Aura 动态构建和 AutoLang VM 现有数据读取能力。生产验收从 025 的 `auto run -r vm` 进入。定向自动测试使用 `auto-ui-runtime` 和 `cargo nextest`，真实窗口对照使用 `WGPU_BACKEND=gl`。

## 4. 需求分析与背景调查

**当前基线与规范**

- AutoUI 主检出基线 `faa2ff0e7cddf4a71f0d5c579cf6608dfe2747d6`。[VM handler 片派发规范](../specs/runtime/handler-dispatch-async.md)（Git blob `8892858f31f03c11b3d1bf1801a12e64bbc3a852`）提供现有派发和完成语义。[CollectionView 规范](../specs/ui/collection-view.md)（Git blob `af23044adada3876e3312e4f9062eb7bc89fc03f`）提供集合身份与交互的项目先例。本计划增补底层渲染合同。
- [PLAN-112](archived/112-autoui-sys-monitor-av-vue.md) §9 round 7 的 12 行样本记录的是截图探针耗时，且有远程显示环境的对照。§10 D-7 记录片派发使消息泵卡死由 8/8 降为 0/8。二者不能单独确定本次渲染改造的端到端收益。GL 对照、新的分段计时与数据规模实验由 T-01/T-05 取得。[DEBT-012](debts.md#debt-012-mcp-快照同步的按需限流plan-112-t-10) 跟踪快照开销。
- 025 的 [app.at](../../examples/ui/025-sys-monitor/src/front/app.at) 以 `overflow-y-auto` 的祖先容器承载 `ProcessesView()`。[processes.at](../../examples/ui/025-sys-monitor/src/front/processes.at) 在表体上方有摘要和表头。VM 的 [tracked.rs](../../crates/auto-ui-runtime/src/ui/aura_view_builder/tracked.rs) 会将这个样式包装为 `View::Scrollable`，该隐式分支的 `on_scroll` 缺省为空。T-01 实测这一层的布局与滚动边界，T-02 确定运行时内部如何接入实际测量。
- [convert_element.rs](../../crates/auto-ui-runtime/src/ui/aura_view_builder/convert_element.rs) 递归转换表格和表体，并从行绑定提取点击和右键消息。[tracked.rs](../../crates/auto-ui-runtime/src/ui/aura_view_builder/tracked.rs) 当前循环路径可能取出整份 VM 列表、逐行过滤并展开。[vm_bridge.rs](../../crates/auto-ui-runtime/src/ui/vm_bridge.rs) 已提供 `index_list`，T-02 验证 025 实际表示的长度读取和区间读取成本。
- 025 放在 `table-row` 上的 `key: p.pid` 与现有 `for ... key: ...` 是两处不同的声明。[route_memo.rs](../../crates/auto-ui-runtime/src/ui/aura_view_builder/route_memo.rs) 的 keyed memo 会检查外部依赖和索引，并以全表预扫描建立缓存计划。它用于参考身份和失效语义。VList 的滚动路径须单独验证按窗口访问。
- [Zed GPUI UniformList](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/elements/uniform_list.rs) 按行高和视口计算区间，并调用区间构建函数。其 [List](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/elements/list.rs) 用尺寸索引、估算及滚动锚点支持变高行。VList 首期采用前一种模型。

**授权**。2026-11-24 用户提出为 025 的虚拟化与排序开计划，并要求按设计审查和 `VList` 命名修订本草案。本次合同限 AutoUI 虚拟化。025 优先沿用现有页面布局和共享源。若唯一可行接线涉及改动 025 冻结模板或改变滚动归属，按 §10 Q-02 记录影响和所需裁定。现有授权覆盖 T-01/T-02 的基线与可行性调查。

## 5. 详细设计

### 5.1 按视口构建

1. 动态 VM 识别一个可虚拟化的重复体，向组件提供当前长度、索引读取、固定行高和行构建器。一次窗口构建使用同一已发布版本的长度、行顺序与索引数据。首期支持已排序或过滤完成、可随机访问的有序集合。现有循环若在视图层执行过滤、按条目生成不定数量的布局项，或其数据源只能全量求值，则沿用原循环语义。T-02 明确可支持的数据表示及其实际读取路径。
2. 滚动容器提供本次布局的有限视口和列表起点。按半开区间计算可见行，包含部分可见行，夹紧到 `[0, row_count)` 后加入两侧缓冲区。列表本体完全在视口之外时，可见行和缓冲区均为空。内容保留全部逻辑高度。
3. 只对计算区间读取条目、绑定当前循环变量和原始索引、构建、布局，并在滚动视口中裁剪绘制和命中。缓冲行可提前构建，视口外不能接收指针事件。表头等固定工作量另行计数。
4. 首帧和窗口缩放以实际布局更新范围。具有有效滚动容器但尚未取得布局时保留待测状态。已确认无有界纵向视口时走完整构建。滚轮、滚动条跳转及数据缩短导致的偏移钳制都更新当前区间。快速跳转可直接访问目的区间。
5. T-02 确定普通列表显式选择 VList 的组件参数、行高接线及 025 的最小接线。025 必须走与普通列表相同的组件和窗口逻辑。环境开关 `AUTOUI_VIRTUAL_LIST=0` 对两个入口启用完整构建对照，保留内容、既有行事件和滚动归属。

固定行高下，可见区间和总高度计算为 O(1)。对已发布的受支持数据源，一次列表更新的取行、绑定、构建和行布局量随窗口项数变化。数据源发布、排序、过滤、独立快照及其他页面内容按实测单独归因。

### 5.2 行身份与交互

- 025 的稳定行身份取模板声明的 `p.pid`。普通列表使用消费点明确提供的稳定键。滚动、排序、插入、删除后，已物化行的显示、事件目标与列表当前顺序一致。
- 缺键或遇到重复键时，保留每一行。位置只在当前列表实例及当前顺序内区分行。无法确认键唯一或顺序稳定时，不让位置身份承接重排前另一行的状态或绑定。支持路径的纯滚动不能靠逐帧扫描整表验证键。
- 选中等应用状态由应用继续持有。当前可见行使用当帧绑定求值，保持已有输入、事件及应用状态流，包含 `SelectProc(pid)`、`RowCtx(pid)`、菜单 `RequestKill` 与现有 emit。列表挂卸、页面切换或同屏使用两个 VList 时，滚动和行状态只归属各自实例。
- 若后续实测显示窗口内重建仍占主要开销，可另行设计有上限的 `View` 缓存，定义行值、外部绑定、原始索引和事件生命周期的失效规则，再纳入新的验收合同。

### 规范增量

| delta_id | 操作 | 目标 | 变更规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/runtime/virtualized-list-rendering.md` | 记录 VList 的固定行高合同、按实际视口构建、按索引读取、屏外逻辑高度、行事件和无有界视口时的完整渲染回退 | 让列表和表格共享可验证的 VM 虚拟化行为 | AC-01、AC-02、AC-04、AC-05 |

## 6. 测试设计

work 负责实现与 V-01..V-05 取证。review 在同一工作树独立复审。merge 负责规范发布、归档与清场。拟用工作树 `D:/autostack/.wt/ui-126/auto-ui`，分支 `plan-126-dev`。建立后在 §9 记录路径、代码、依赖与基线。

| ID | 验收 | 入口与区分性判据 |
|---|---|---|
| V-01 可伸缩性 | AC-01、AC-02 | 固定视口和行高，使用走真实 VM 构建及 Iced 布局路径的普通 VList 样本（如 1,000 行与 10,000 行），从首帧、滚动一行和拖到远端采样。分别记录底层读取和展开条目数、行/单元格构建量、行布局节点数，检测整表拷贝、扫描或每行占位。可见范围相同的采样中，大样本的工作量仍受可见行加缓冲区及常数开销约束。用关闭虚拟化的完整构建做计数对照。 |
| V-02 视口与内容 | AC-01、AC-04 | Iced 组件层验证空表、短表、一部分行露出、列表全在窗口外、尾部及缩短列表后的夹紧。025 实窗验证首帧、窗口缩放、连续滚轮和拖动滚动条。对同一数据与偏移，用实际裁剪区域内的可见行、顺序和滚动范围与完整构建比对。用行标识及文本作为主判据，可同时保留截图。 |
| V-03 行交互与实例隔离 | AC-04 | 025 实窗点选、滚动后右键、排序后再点选，检查收到的 PID、可观察选中状态和菜单后续动作。普通 VList 的定向回归覆盖外部绑定变化、同屏双实例、重复或缺键、重排后的事件身份及视口外命中。保留 PLAN-112 T-03 式拒绝、放行、关闭回归。 |
| V-04 实窗性能对照 | AC-03 | 同机 GL、相同窗口尺寸/DPI、固定可复现的约 314 行输入与滚动序列，对照 `AUTOUI_VIRTUAL_LIST=0` 和启用版本，记录 revision、MCP 模式、行数与样本数。独立记录纯滚动、数据刷新各自的 VM 读取/构建、Iced 布局与帧耗时，以及 `WM_NULL` RTT 和挂起样本。至少报告中位、P95、最大值及是否达到 2×。另跑 025 的实时进程数据验证真实入口。 |
| V-05 兼容回归 | AC-04、AC-05 | 普通 `for`、关闭开关、无有界纵向视口和不支持的数据源走完整行为，内容和事件保持。运行 `auto-ui-runtime` 的相关定向用例和 025 实窗回归。代码冻结后保留 r1 的完整回归范围，使用 `cargo nextest run -p auto-ui-runtime --features ace-bridge --lib` 和同参数的 `--test desktop_behavior` 目标。 |

T-03/T-04 建立 V-01..V-03 和 V-05 所需的定向 fixture。T-01 准备绑定当前源和构建配置的 GL 运行产物，T-05 使用绑定实施 revision 的产物。日常测试使用 debug 增量准备，测试体消费已准备的产物。每条新增自动用例记录含准备好的输入装配、断言及清理的实测时长，遵循每例 ≤ 10 秒约束。实窗性能取证记录命令、各阶段用时、大小对照及原始证据位置。上述测试和测量均待实施。

## 7. 验收标准

| ID | owner_stage | 通过条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 025 与普通 VList 共享固定行高窗口机制。只构建、布局可见行与最多两侧各 2 行缓冲。屏外逻辑高度正确，首帧、缩放、跳转、尾部和空列表均有可观察结果。 | V-01、V-02 |
| AC-02 | work | 同一有限视口下，受支持数据源的纯滚动取行、构建和行布局量随窗口项数变化。首帧和远端跳转可直接读取目标区间，规模实验能识别隐藏的全量展开、扫描和逐行占位。 | V-01 |
| AC-03 | work | 025 真实应用走 VList 路径。提交可复查的同机 GL 对照，量化构建与布局减少，并分别报告纯滚动、刷新帧和消息泵响应。2× 作为端到端性能目标，未达到时记录差距及已测到的主要开销。 | V-04 |
| AC-04 | work | 当前可见行显示及事件与完整构建一致。排序、滚动、刷新、键冲突及实例切换不会把状态或事件交给别的行。关闭开关、无有界视口和不适用路径保持完整构建。 | V-02、V-03、V-05 |
| AC-05 | review | SD-01 文档覆盖已验证的 VM/Iced 合同，并与 025 生产入口、普通列表消费路径、实测证据和回退条件一致。 | V-05、独立复审 |

## 8. 执行步骤

开工时在当前工作环境可用的任务面板登记以下 ID。从主检出基线派生实施树后启动 T-01。

- [x] T-01 确认 025 实窗与性能基线
  - **work**。在已准备的 025 VM/GL 环境测量实际滚动容器、视口尺寸、表体起点、现有整表构建量与 V-04 分段耗时。记录运行器、版本、开关、MCP 状态和输入。给 AC-01、AC-03 提供基线。
  - 结果：pass。视口 (193,60.4,1087,739.6)、内容高 3669、表体 content 起点 177px、行高 29px、渲染 120 行 ×5 格（1200 节点 = 整页 1390 元素的 86%）、可见行 20（顶部）～27（尾部）。首帧 s4 element 435.9ms / s5 layout 95.4ms；MCP scroll 往返 717.6ms。证据见 §9 T-01 与 `D:/autostack/plan-artifacts/plan-126/t01/baseline2/`。
- [x] T-02 确认可按窗口取行的接线
  - **work**。依赖 T-01。验证 025 列表的实际表示、取得长度和按索引读取的路径、表体相对滚动坐标，以及首帧和缩放的测量来源。将两种消费入口、行高单位与间距口径、回退条件和正反示例写入 §9。涉及 §10 Q-02 的实质变更时记录所需裁定，再推进受影响的 025 接线。覆盖 AC-01、AC-02、AC-04。
  - 结果：pass。注入点 = `tracked.rs:40` ForLoop 臂（生产轨；table-body 正走此臂），调试轨需在 `convert_for_unkeyed`/`convert_for_keyed`/`for_loop_iterations` 保持同窗口语义；数据读取需新增 O(1) 长度与按索引取元素；测量来源 = ScrollStateReader pane 读回 + LayoutCollector content 空间 bounds；120 行上限、行高 29px、10,000 行样本需 f32 高度占位。025 源不需要改（Q-02 无需裁定）。详见 §9 T-02。
- [x] T-03 实现固定行高 VirtualList
  - **work**。依赖 T-02。接入 VM 展开前的区间入口和 Iced 布局，完成 `AUTOUI_VIRTUAL_LIST=0` 对照和普通列表 fixture。按 V-01 验证底层取行、构建和布局三个维度。覆盖 AC-01、AC-02。
  - 结果：pass。025 实窗窗口化 `[0,22)…[93,120)`、`row_h=28.97 list_top=176.20`、内容高≈3669、clamp 不变（`41d6138a`）；显式 `scroll (row-height: …)` 入口 + 1k/10k fixture：窗口化两侧均 31 行/37 vnode/3 layout nodes、0.43–0.75ms，关闭对照 1003/10003 vnode、12.8/125.6ms（`fbabf41d`）；V-02 六例 PASS。见 §9。
- [x] T-04 接通 025 与行交互
  - **work**。依赖 T-03。让 025 的表体接入共享组件，使用既有视口，保留表头布局与排序入口。验证滚动、缩放、点击、右键和排序后的 PID，补齐普通列表正反对照。覆盖 AC-01、AC-04。
  - 结果：pass。真实树目标 + 三条独立 PID 证明（handler 载荷 / `selected_pid` / `tasklist` oracle）在 offset 0、滚动到 2400、按名重排后均一致；滚出窗口的旧行 id 不可命中（`VNode not found`）；修掉同帧兄弟 pane 共用 scope key 的隔离缺陷（`ce6d6234`）。右键菜单在窗口内不可编程驱动（MCP 无 context-menu 动作、OS 输入不可达），以构建期单测 + 标注的 fixture 注入取证。见 §9。
- [ ] T-05 验证规模、交互和性能
  - **work**。依赖 T-03、T-04。运行 V-01..V-05 的 work 验证，登记样本、计数、差值、未观测项及诊断。原始证据写入 `D:/autostack/plan-artifacts/plan-126/`，结果摘要写入 §9。覆盖 AC-01..AC-04。
- [ ] T-06 准备规范与复审收据
  - **work**。依赖 T-05。在实现树准备 SD-01 和 revision 绑定证据，记录实际新增用例与时长、受影响回归、已知边界与待执行合并门。为 AC-05 提供输入。
- [ ] T-07 独立复审
  - **review**。依赖 T-06。按 AC 和 V 编号核对实现、回归证据、性能口径与 SD-01，记录审查结论和实际覆盖范围。关闭 AC-05。
- [ ] T-08 合并与归档
  - **merge**。依赖 T-07 通过。完成合并负责的剩余回归，按当前仓库流程发布规范、合并、归档并清理工作树及证据。

## 9. 复审记录

2026-11-24 stage: new，草案。来源 = PLAN-112 的性能观察（§10 D-6/D-7/D-9 与归档记录），未开始实现。

2026-11-24 stage: new | PLAN-126:r1 | outcome: pass（可进入 work，两个待确认项见当时 §10 Q-01/Q-02） | plan_revision: 1 | tasks: T-01..T-08 | acceptance: AC-01..AC-05 | spec_delta: SD-01 | next: work（T-01 基线测量），Q-01（排序独立立项）、Q-02（025 是否改滚动容器）待用户确认，不阻塞 T-01。

r1 基线 Git blob `c667888299e6fb283f23233b13532a77b665605e`。r1 的 AC-02 承诺未变化行命中缓存、零重建，AC-03 设同机 GL 的 ≥ 2× 硬门槛。r2 按用户要求落实设计审查，AC-02 改为固定视口下的工作量约束，AC-03 改为分段实测并报告 2× 达成情况。变高行和任意循环自动虚拟化移出首期。r1 曾使用父容器高度的视口假设，r2 以 025 的既有滚动容器和实际布局为依据。r1 没有实现及产品测试证据。

2026-11-24 stage: new | PLAN-126:r2 | outcome: pass（可进入 T-01/T-02 调查） | plan_revision: 2 | tasks: T-01..T-08 | acceptance: AC-01..AC-05 | spec_delta: SD-01 | next: work/T-01。状态 drafting，产品验证待执行。若 T-02 触发 Q-02，先处理受影响的接线决策。

2026-11-24 stage: work | PLAN-126:r2 | outcome: in_progress（T-01、T-02 pass；T-03..T-06 未开始） | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | base_commit: 2e9c5dcf7c38428d697bc4e6bf6319f73aa9ce88 | code_commit: 2e9c5dcf（尚无实现提交） | dependencies: auto-lang 816585732（plan-126-dev）、auto-web 0cfc7c7、auto-down 456c86a | tasks: T-01、T-02 | next_responsible_role: work（T-03）

**T-01（pass）025 实窗与性能基线**

- 入口与运行器：`auto run -r vm`（`auto.exe 0.1.0+plan-890-verified-r7-t19-116-gc879b3e`，auto-man 检出）+ `AUTO_UI_TOOL_BIN=<worktree>/target/debug/auto-ui-tool.exe`（SHA256 `C8754E92B4F99CAC5E56D2B3923C156B6C1A37A25EC6A9BDB8D9AF97A35CFF27`）；`AUTOUI_MCP_PORT=9400`、`AUTOUI_TEST_FIXTURES=1`、`AUTO_VM_STORAGE_FILE=<temp>`、`WGPU_BACKEND=gl`、`AUTO_FRAME_BENCH=1`。窗口标题「系统监视器」；启动到 MCP 可用 3.1s，整轮 25.8s。
- 滚动容器：`app.at:146` 的页面列隐式 `View::Scrollable`（`vnode_4289777634216808055`），视口 rect `(193, 60.4, 1087, 739.6)`，内容 `1087×3669.0`；表体 content 空间起点 y=177px（视口顶到表体顶）；行高固定 29px。
- 现有构建量：表体 120 行 × 5 单元格 = 600 cell + 600 text = 1200 节点，占整页 1390 个元素的 86%；快照 2217 行。渲染行数上限来自 `sys_store.at:263`（`if n > 120 { n = 120 }`，`.ApplySort` 的 O(n²) 选择排序），徽标显示原始进程数（本次 324→328）；**故 §0/§4 的「约 314 行」是列表长度，实际渲染 120 行**（已在 §0 加实测修正注）。
- 可见行：offset=0 时 20 行；offset 由 7200 clamp 到 2929（尾部）时 27 行；`autoui_scroll_state` 的 pane offset 依次 0→600→2400→2929，content 高恒定 3669。
- 分段耗时（114 帧 `[P725-FRAME]`，µs）：s4 element p95 413282 / max 435890；s5 layout p95 1583 / max 95439；s3b build p95 10393 / max 13542；s3a mcp p95 15077；s2 vm median 100481（`Tick` handler parked）。
- MCP 往返：snapshot 32.1ms、`scroll` action 717.6ms（4 次 667–770ms）、screenshot 3251.6ms、scroll_state 3.3ms。
- 原始证据：`D:/autostack/plan-artifacts/plan-126/t01/baseline2/`（`summary.json`、`snapshot_off*.txt`、`screenshots/off{0,600,2400,7200}.png`、`tool-stderr.log`）；采集脚本 `plan-126/t01_baseline.py`（同一协议供 V-04 改后对照复用）。
- 观察（记录，不在本计划修复）：滚动/刷新期间反复出现 `[VM-PARKED] handler_App_Tick re-entry ignored`，即 `Tick` 段被 CPU 类工作占住、重入被忽略；V-04 的「数据刷新」取证须把该停顿与渲染开销分开归因。排序搬离 UI 线程仍按 §10 Q-01 独立立项。

**T-02（pass）可按窗口取行的接线**

- 注入点：生产轨 `convert_node_with` 的 `ForLoop` 臂（`tracked.rs:40–172`）；`table`/`tbody` 在 `convert_element.rs:1215–1253` 落为 `View::Column`，行循环正走该臂（tracked 分派对 table 无专用臂，转 untracked `convert_element`）。调试/MCP 轨需同语义：`convert_for_unkeyed`（`tracked.rs:413`）、`convert_for_keyed`（`route_memo.rs:560`）、`for_loop_iterations`（`tags_container.rs:333`）。
- 行身份：025 的 `key: p.pid` 在 VM 表格路径当前不生效（untracked 臂不读 `key_expr`），现状是每帧全量重建、无复用。窗口化须保留绝对索引语义（`Value::Int(i)`、`path.push(i)`、`ForIter.index`，行点击/右键参数在 `events.rs:34–55` 按当帧绑定烘焙）。
- 数据读取：`.store.procsView` 解析为堆 `ListData<Value>`/`Value::Array`；现有 `index_list`/`index_list_all`/`read_state_as_vec` 均整表 clone（O(N)），无 `list_len`/单元素读取。窗口化需在 `vm_bridge` 增 O(1) 长度与按索引取元素，并保留既有权重依赖记录。
- 测量来源：pane 几何（viewport rect / offset / content）取自 `ScrollStateReader` 读回（`autoui_scroll_state` 的 panes，key=vnode_…，本次 frames 1–9 可见）；元素 bounds 取自 LayoutCollector（`@rect`，content 空间、不随滚动平移；首帧未回填）。因此：首帧/未测得 bounds 时保留完整构建，取得测量后的下一帧起窗口化（对应 §5.1.4）。
- 表体相对滚动坐标：content 空间常量 177px；行窗口 = `(offset −177, offset+739.6−177]` 除以 29px，两侧各加 overscan（2 行）。
- 大范围占位：`View::Container.height` 为 `Option<u16>`（≤65535px），10,000 行 ×29px = 290,000px 超出；V-01 的 10,000 行样本需要 f32 高度占位表达式（`View::SpacePx` 一类），并在投影层落 `Length::Fixed`。
- 回退条件：无有界纵向视口 / 数据源不可按索引读取 / 行高未测 / `AUTOUI_VIRTUAL_LIST=0` → 完整构建（即现状），内容、滚动范围与事件保持。
- 两种消费入口：025 走「表格体自动识别」（不改 025 源，§10 Q-02 无需裁定）；普通列表用显式 `vlist` 元素（`row-height` 显式声明），两者共用同一窗口计算与占位实现。
- 正反示例（T-03/T-05 实取证）：正 = 视口内 20–27 行照常显示，点击/右键 PID 与排序后行序一致，滚动范围仍为 3669px；反 = 关闭开关或去掉滚动边界时保持 120 行完整构建与现有滚动/事件。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: 2e9c5dcf | next: work

2026-11-24 stage: work | PLAN-126:r2 | outcome: in_progress（T-03 骨架落地，端到端激活未验证） | plan_revision: 2 | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | code_commit: e491ab6a | dependencies: auto-lang 816585732、auto-web 0cfc7c7、auto-down 456c86a | tasks: T-03（进行中） | next_responsible_role: work（T-03 续）

**T-03（进行中）已落地**

- `vm_bridge`：新增 `list_len_of(&Value)` / `list_get(&Value, i)`——四种列表表示（`Array` / 堆 `ListData<Value>` / 堆 `ListData<i32>` / `VmRef`）的 O(1) 长度与单元素读取，保留容器读依赖记录（`record_dep_any`）。
- `View` 新变体：`SpacePx { width, height }`（f32 高度占位，绕过 `Container.height: Option<u16>` 的 65535px 上限）与 `VirtualMark { key, child }`（零布局构建带标记，投影为 `vlist_mark:{key}` 合成 iced id）。全部穷尽匹配点（projection/vnode_converter/coverage/snapshot/devtools/rq_probe/map_msg）已补齐，能力分母同步为 `cap.view.space_px` / `cap.view.virtual_mark`。
- 新增 `ui/virtual_list.rs`：环境开关 `AUTOUI_VIRTUAL_LIST`（缺省启用、`0` 关闭）、诊断 `AUTOUI_VLIST_DIAG`、纯函数 `visible_range`（含 out-of-view/夹紧/退化输入 11 条单测，全部通过）、进程级几何台账（pane 偏移/视口/内容高 + 构建带 rect → 反解 `row_h`/`list_top`）、`VListProbe`（一次遍历收 pane 与 mark）、消息 `__vlist_measure`。
- 构建侧 `aura_view_builder/virtual_rows.rs`：pane 作用域（线程本地栈，child-widget builder 共享；`overflow-y-auto`/`overflow-auto` 列与 `scroll` 元素压栈，Drop guard 出栈）、`try_virtual_for`（解析可索引数据源 → 台账几何 → 窗口构建 `[start,end)` + 首尾 `SpacePx` + `VirtualMark`；无几何时做一次「顶部 ≤256 行」标定构建）。挂接 `convert_node_with`、`convert_for_unkeyed`、`convert_for_keyed` 与 tracked 列双胎。
- host：每帧按请求位推 `VListProbe`，`__vlist_measure` 回执落台账（不脏帧、不派发 VM）。
- 编译与单测：`cargo check -p auto-ui-runtime --lib --tests --features ace-bridge` 通过；`nextest --lib virtual_list` 11/11 pass。提交 `e491ab6a`。

**遗留（下一轮必须先解决）**

1. 实窗（vlist3）证明机制未端到端生效：`[VLIST] measure panes=0 bands=0`（probe 收不到 `vlist_mark:` 容器），快照仍是 120 行、无 `Space` 占位。需定位 `VirtualMark`→container id 是否进入 iced 树、probe 是否在窗口化树上运行（猜想：MCP 轨与生产轨各自建树，probe 只遍历其一）。
2. 观察到的异常：同一轮里 `s4 element` 中位 12.6ms（基线 104ms 均值）与个别快照只剩 2 行，说明窗口/标定路径偶发走进错误几何；修复 probe 后需重跑基线协议确认行数与滚动范围。
3. 未做：普通列表 fixture（`row-height` 显式入口）、V-01 规模样本（1,000/10,000 行）、V-02 视口用例、`AUTOUI_VIRTUAL_LIST=0` 对照取证。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 2 | from: executing | to: executing | status: executing | outcome: in_progress | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: e491ab6a | next: work

2026-11-24 stage: work | PLAN-126:r2 | outcome: in_progress（T-03 阻塞点已定位：窗口不经过 View→Element 投影） | plan_revision: 2 | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | code_commit:（本轮诊断提交，见 git log） | tasks: T-03（进行中） | next_responsible_role: work（T-03 续）

**T-03 诊断结论（本轮实质进展）**

- 增加探针后实窗（`t01/vlist4`）拿到决定性反证：`[VLIST-MARK] projecting id=…` **一次都没有打印**，而 `[VLIST] measure panes=0 bands=0` 持续出现。前者说明窗口的 element 树不是由 `AbstractView → IntoIcedElement`（`iced/renderer/projection.rs`）为窗口化子树构建的；后者说明 probe 的 `container` 回调也拿不到 `vlist_mark:` 容器。
- 结合代码：`render_dynamic_view` 全部调用点都在 `iced/renderer/devtools.rs` 内部或 `renderer/mod.rs` 的测试里，生产窗口另有一条构建入口（MCP 快照用的 `styled_vtree` 也说明存在 VTree 面）。因此 `View::VirtualMark` / `View::SpacePx` 在**快照/vtree 面**按我加的臂透明化（子件透传 / Text-Empty），在**窗口面**则根本没走到我加的投影臂——窗口化视图建出来了（`request_measure()` 被调用、`[VLIST] measure` 有行），但窗口渲染看不到标记与占位。
- 另一个确认：`record_build` 的 `[VLIST] scope=…` 行一次都没出现 → 只有「标定帧」在跑（`plan()` 恒 `None`，因为 probe 从未回执几何）；这解释了为什么行数仍是 120、偶尔出现 2 行（标定窗口与缓存帧交错）。

**下一轮的可执行路线（按优先级）**

1. 先定位窗口 element 树的真实构建入口（`DynamicComponent::view()` → ??? → `Element`），把 `VirtualMark`（合成 id 容器）与 `SpacePx`（f32 高度 space）在**该入口**同样落下；这是让 probe 与占位生效的最小修正。
2. 若该入口以 VNode/VTree 为中间层：给 `VirtualMark` 分配稳定 vnode id 并在 vtree→element 层保留，`SpacePx` 映射为固定高度间隔节点；probe 改读 vtree bounds（`__bounds_collected` 已有同源数据）。
3. 退化方案（若 1/2 代价过大）：给虚拟化 pane 的 `View::Scrollable` 加 `measure_key` 字段（约 20 处构造点，机械补齐），由 `scroll_state_reader` 拿到视口/偏移；行高改由**显式 `row-height`**（普通列表入口 + 025 表体默认值）提供，`list_top` 用「pane 内容高 − 行数×行高」反解。该方案不需要 mark。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 4 | from: executing | to: executing | status: executing | outcome: in_progress | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: e375863f | next: work

2026-11-24 stage: work | PLAN-126:r2 | outcome: in_progress（T-03 核心窗口化已实窗生效；普通列表 fixture 与 V-01 规模取证未做） | plan_revision: 2 | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | code_commit: 41d6138a | tasks: T-03（进行中） | next_responsible_role: work（T-03 收尾）

**T-03 核心窗口化（实窗已验证）**

- 根因修正：窗口 element 树确经 `DynamicComponent::view_with_debug_gated` → `dynamic_view_impl_inner`（`iced/renderer/projection.rs:3784`）→ `render_dynamic_view`（`iced/renderer/devtools.rs:3461`）。上一轮判断的「不经投影」不完全准确——窗口化 band 先被 `convert_view_messages` 的 `_ => AbstractView::Empty` 兜底吞掉（`iced/renderer/mod.rs:1606`，与 PLAN-082 Rich / PLAN-029 Custom 同类），且标记子件走通用 `into_iced` 丢了 vnode id。两处已修：`convert_view_messages` / `render_dynamic_view` 显式透传 `SpacePx`/`VirtualMark`（mark 递归渲染并挂 `vlist_mark:{key}:{start}:{end}`）；`View::Scrollable` 增 `measure_key`，`build_scrollable` 把既有 `vnode_*` id 登记进台账（滚动目标、bounds id、快照 id 均不变名）。
- 标定/反解：行高只由「全列表标定」且取两个时间上分离的样本 + 3s 复验（启动期字体/世界瞬态实测 3456 vs settle 3477）；`list_top = band_top − start*row_h − viewport_y`；视口宽变化使行高失效；**偏移变化置脏帧**（否则滚动后窗口不跟随，截图为空——本轮抓到并修）。
- 实窗证据（GL，025，offsets 0/600/2400/7200，`t01/probe11`，`t01/off10` 为关闭对照）：`[VLIST] measure panes=1 bands=1` ×63；`scope=vlist_0 n=120 window=[0,22)`（顶）/`window=[93,120) rows=27`（尾）；`row_h=28.97 list_top=176.20`，标定带 rect `(209,237,1055×3477)` = T-01 基线；pane 内容高 3663.6–3685.1（基线 3669，差值等于换行成两行的进程名各 +21px）；偏移 clamp 0→0 / 600→600 / 2400→2400 / 7200→2946。截图中各偏移约 20–28 行填满视口。
- 关闭对照（`AUTOUI_VIRTUAL_LIST=0`，需 `AUTOUI_KEEP_VLIST=1` 因 harness 会 pop 该变量）：120 行、内容 3689.8、clamp 2950——完整构建行为逐项未变。
- 测试：`virtual_list` 16/16（新增标定/复验/宽度失效/mark id 用例）、`scrollable` 9/9、`vnode` 46/46、`cargo check --lib --tests --features ace-bridge` 通过；全 lib 套件 2666/2680，13 失败 +1 超时在 `e375863f` 干净树上同样复现（剪贴板/OS 环境、AV 编译器切片、gallery/media），非本计划引入。
- 证据索引：`D:/autostack/plan-artifacts/plan-126/t03/README.md`；原始运行 `t01/probe9|10|11`、`t01/off10`。

**T-03 余项（下一轮收尾后转 T-04）**

1. 普通列表 fixture：显式 `vlist`/`scroll` 入口的 `row-height` 参数与同一窗口/占位实现（当前显式 `scroll` 元素在 tracked/MCP 轨仍走完整构建，属已记录的边界）。
2. V-01 三维取证：1,000 / 10,000 行样本的「取行数 / 行构建数 / 行布局节点数」计数与 `AUTOUI_VIRTUAL_LIST=0` 对照；10,000 行需要把总高度 289,700px 走 `SpacePx`（已具备）并确认滚动条范围与远端跳转。
3. V-02 首帧/缩放/尾部/空表的方向性用例（窗口跟随滚动已在实窗抓过一次回归，需纳入定向用例）。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 5 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: 41d6138a | next: work

2026-11-24 stage: work | PLAN-126:r2 | outcome: pass（T-03 完成，T-01..T-03 已闭） | plan_revision: 2 | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | code_commit: fbabf41d | tasks: T-03 | next_responsible_role: work（T-04）

**T-03 收尾（显式入口 + V-01/V-02 取证）**

- 显式入口：`scroll (row-height: 28[, list-top: 0])`（别名 `scrollable`/`row_height`/`list_top`）。声明 `row-height` 的 pane 视为可信几何：首帧只建顶部 `min(n,256)` 行 + 尾部 `SpacePx` 给出精确总高，首个 probe（视口/偏移）后切真实窗口；不标定、不周期复验，band 仅细化 `list_top`；非法值忽略。解析 `virtual_rows.rs:113`（`vlist_explicit_geometry`）、untracked `tags_container.rs:186`、tracked `tracked.rs:2601`（pane `measure_key` `tracked.rs:2660`）、台账 `virtual_list.rs:533/:563`。无该 prop 时逐字节保持既有行为；`AUTOUI_VIRTUAL_LIST=0` 完全忽略。
- Fixture：`crates/auto-ui-runtime/src/ui_tests/plan126_vlist_scale_tests.rs`（inline `.at` widget，1k/10k `Value::Obj` 行经 `write_state_vec` 注入，真实 `convert_node_with` ForLoop → `View` → `view_to_vtree`）。
- V-01（row_h 28、list_top 0、viewport 740、offset 700、overscan 2 ⇒ 窗口 `[23,54)`）：1,000 与 10,000 行窗口化**逐维相同**——取行 31 / `list_get` 31 / 构建 31 / vnode 37 / layout 3，0.745ms 与 0.428ms；关闭开关为 1003 与 10003 vnode、12.771ms 与 125.564ms（线性）；占位合计 `(n-31)*28` = 精确总高。
- V-02（6/6 PASS）：窗口跟随滚动（偏移脏帧 + `[0,29)`→`[23,54)`，host `host.rs:2047`）、视口高变化（`[0,29)`→`[0,13)`）、尾部夹紧 `[971,1000)`、空表（无窗口零读取）、短表（10 行无占位）、列表整体在视口外（空 band、内容高保持）。
- 测试：`virtual_list` 33/33（16 旧 + 17 新，~0.8s）；同过滤在 `AUTOUI_VIRTUAL_LIST=0` 下 33/33（8 例打印 skip 理由 + 1 例断言 `enabled()==false` 与完整构建数字）；`plan081` 3/3、`scrollable` 9/9、`vnode` 46/46、`plan656` 1/1；全 lib 2697 run / 2685 pass / 11 fail + 1 timeout，全部落在既有类别（AV 切片、剪贴板/OS、gallery/media、plan657），无 vlist/scroll 失败。
- 证据：`D:/autostack/plan-artifacts/plan-126/t03-scale/README.md`（props/API、fixture、数字、命令、退出码）。
- 遗留（T-04/T-05）：tracked 轨窗口行经 untracked 转换构建，逐行 `BuildProbe`/debug-id 条目（及窗口内行点选 PID）仍缺，025 同款；显式 `row-height` 列表的实窗 GL 验证需要 harness 应用源（examples 不在本计划范围）；固定行高在换行双行处仍有 +21px/行漂移；Vue 生成器无窗口化/row-height 提示。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 6 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: fbabf41d | next: work（T-04）

2026-11-24 stage: work | PLAN-126:r2 | outcome: pass（T-04 完成，T-01..T-04 已闭） | plan_revision: 2 | worktree: D:/autostack/.wt/ui-126/auto-ui | branch: plan-126-dev | code_commit: ce6d6234 | tasks: T-04 | next_responsible_role: work（T-05）

**T-04（pass）025 行交互与实例隔离**

- 方法：真实树目标（`autoui_snapshot(include_bounds)` 取行 `vnode_*` → `autoui_action press`），PID 三路互证：handler 载荷 `ProcessesView.SelectProc␟i␟<pid>`、`autoui_state.selected_pid`、独立 `tasklist` oracle。脚本 `plan-126/t04/t04_interact.py`（39/40 检查通过，69s；1 项为已标注的环境限制）。
- 结果：offset 0（窗口 `[0,22)`）`explorer.exe`→7360、窗口底行 `sccache.exe`→1069196、`python.exe`→240744；滚动到 2400（窗口 `[74,105)`，31 行）`conhost.exe`→22952、顶边 `chrome.exe`→27156、底边 `vmcompute.exe`→8920——无错位 PID；**旧窗口行 id 不可命中**（`Error: VNode not found: vnode_…`），选中态不变；按名重排后 `WmiPrvSE.exe`→8548、`WindowsTerminal.exe`→31472 跟随新顺序。
- 右键：`oncontextmenu` 与点击同 PID 由构建期单测证明；窗口内不可编程驱动（MCP 无 context-menu 动作、OS 合成输入不能进窗——`SetForegroundWindow` 失败、行中心合成左键不选中），菜单路径以标注的 `autoui_fixture` 注入取证（`ctx_open=true`、菜单可见、`CloseCtx` 关闭）。登记为宿主能力缺口，不改本计划合同。
- 实例隔离修复（AC-04）：同帧兄弟 pane 曾共用 `vlist_0`——旧 `pop_scope` 在「栈空」时清零计数，两个兄弟 `scroll (row-height:)` 抢同一台账（120/60 行实测 `last_plan("vlist_0").total==60`）；改为帧级编号（`virtual_list.rs:184` `enter_build_frame()`、`push_scope` 只递增、`pop_scope` 不清零，两处构建入口 `aura_view_builder/mod.rs:455,515` 装）后 025 仍单 `vlist_0`，几何不变。
- Probe 对齐：未实现并如实记录——窗口行经 `try_virtual_for`→untracked 转换构建，无 `for`/debug-id 条目；诚实条目需把 VTree 路径（`path+[band_idx, i-start]`）穿进该臂，否则会伪造。025 不受影响（其 `table-row` 本就委托 untracked `convert_element`）。以 `plan126_vlist_windowed_rows_have_no_for_probe_entries` 钉住。
- 测试：新增 `ui_tests/plan126_vlist_interact_tests.rs`（绝对索引点击/右键参数 `[23,54)`、数据源重排后参数跟随、双 pane 隔离、窗口行 vnode 路径可解析、probe 缺口刻画，均 <1s）；`plan126_vlist` 14/14、`virtual_list` 33/33、`plan081` 3/3、`scrollable` 9/9、`vnode` 47/47；全 lib 2691/2702，11 项既存失败（其中 8 项在 `fbabf41d` 干净树同样失败）。
- 另记（先于本计划的缺陷，不在本计划修）：`table-row` 生成 `View::Row{style: None}`（`convert_element.rs:1269`），025 行高亮样式在 VM 轨两模式下都被丢弃；状态半边（`selected_pid`）正常跟随。纳入 V-05 兼容回归观察项与后续债务记录。
- 证据：`D:/autostack/plan-artifacts/plan-126/t04/README.md`（`final4/`、`final4_stdout.txt`、`t04_interact.py`、`t04_diag.py`）。

2026-11-24 stage: goal | plan_id: PLAN-126 | plan_revision: 2 | round: 7 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-126/auto-ui | commit: ce6d6234 | next: work（T-05）

## 10. 待澄清事项

- **Q-01 排序**。将排序移出 UI 线程是原始需求中的独立工作。由后续计划根据 AutoLang 与 025 实际排序路径确定边界，PLAN-126 可以独立推进。
- **Q-02 025 接线**。源码已确认存在滚动祖先。T-01 测实际边界，T-02 判断能否在运行时共享路径接入并保留当前 025 源。如果确需修改冻结模板、调整布局或改变滚动归属，记录最小变更、兼容影响和对照，待裁定后实施该接线。
  - T-02 结论（2026-11-24）：025 源无需修改。表体行循环本来就走生产轨 `tracked.rs:40`，运行时可在该臂按实测窗口裁行；025 的滚动归属、页面布局与模板均保持。Q-02 暂不需要用户裁定；若 T-03 实作发现必须改 025 源或滚动归属，再回到本项。
- **Q-03 2× 目标的口径（记录，非阻塞）**。T-01 实测 025 渲染行上限为 120（`sys_store.at:263`），而 s4 element 峰值 435.9ms 包含整页 1390 个元素。V-04 的端到端 2× 目标按同一协议（GL、同窗口尺寸、同滚动序列）对照 `AUTOUI_VIRTUAL_LIST=0` 与启用版；未达 2× 时按 AC-03 记录差距与已测到的主要开销，不据此下调验收。
