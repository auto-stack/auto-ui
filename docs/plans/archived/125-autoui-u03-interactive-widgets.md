---
plan_id: PLAN-125
status: archived
completion_kind: delivered
feature_name: AutoUI U03 独立交互 Widgets 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-20
plan_revision: 2
current_step: 7
total_steps: 7
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/ace-package-widgets.md]
touched_goals: []
---

# PLAN-125 AutoUI U03 独立交互 Widgets

## 0. 变更摘要

2026-11-20 r2 经用户确认纳入 AutoLang 共用 debug/test 合并门修复。七页范围和原验收义务继续按 r1 执行。

2026-11-20 从 [105 r2](../105-autoui-u03-local-interactive-apps-av-vue.md) 移交 7 个 Widget 原页的组件缺口、AV/Iced 与 A2Vue 验收、复审和交付。正式结果回到 U03 原矩阵。

## 1. 目标

| 原页 | 可观察行为 |
|---|---|
| [accordion](../../../../auto-os/widgets-gallery/src/front/pages/accordion.at) | 单展开组在切换条目时更新内容与展开状态，允许按原设置折叠。 |
| [collapsible](../../../../auto-os/widgets-gallery/src/front/pages/collapsible.at) | 触发动作驱动受控展开，内容和可访问状态一致。 |
| [scroll](../../../../auto-os/widgets-gallery/src/front/pages/scroll.at) | 在原有有界区域确实滚动，目标内容进入可见区。 |
| [scrollarea](../../../../auto-os/widgets-gallery/src/front/pages/scrollarea.at) | 独立滚动区域显示后段条目，外层不误滚。 |
| [pagination](../../../../auto-os/widgets-gallery/src/front/pages/pagination.at) | 前后页与原页码选择更新激活项，边界行为一致。 |
| [progress](../../../../auto-os/widgets-gallery/src/front/pages/progress.at) | 原值 33 产生正确可见进度及可读值。 |
| [slider](../../../../auto-os/widgets-gallery/src/front/pages/slider.at) | 原区间和值可读，输入更新受边界与步长约束。 |

## 2. 架构方案

以 AutoOS Widgets Gallery 原页及其现有组件依赖做输入。优先复用 AutoUI 既有组件与注册项，在可定位的组件写面补实际缺口。两个后端各自运行同一原页和原控件，保留事件、输入、焦点、无障碍及生命周期的适用原义务。

125 持这 7 页的交付行。105 持六个原应用、其余 7 个 Widget 页和 U03 合计。123 的布局页复用共同控件时按交付版本消费。说明区域共用的 table 仍走既有组件，独立 table 页沿原验收与相关 owner 对账。若更改涉及在途共享编译/宿主接口，由既有 owner 单写，其余页继续独立验收。

## 3. 技术栈

原源来自 AutoOS widgets-gallery/src/front/pages/。AV 使用既有 ace_node_desktop，A2Vue 使用 generate_package_sfc 和 1072 的真实构建/浏览器入口。可复用 AutoUI 的 ace_node_desktop、plan980_ace_vue_tool_wiring 回归，以及已有 ui_accordion、ui_scroll、ui_progress、ui_slider 示例定位部件行为。

实施树拟为 D:/autostack/.wt/ui-125/{auto-ui,auto-lang}，每仓分支 plan-125-dev，从各自 master 起步。AutoOS 原源按提交只读消费。涉及公开接口时，按实际修改选固定组及两端消费者。

## 4. 需求分析与背景调查

用户授权移出可并行子项。核对 AutoUI master 448ed614、AutoLang master 2c6120d8b、AutoOS efcfced51b58。7 个原页的预览使用局部属性、固定条目或控件事件，无 105 应用 Store、后端 API、路由或运行时业务列表的前置。原型收据采用 [1071 冻结矩阵](../../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md) 与 [原源归属](../../specs/ui/examples-ownership.md)，浏览器使用 [1072 正式交付](../../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)。T-01 钉依赖版本、原预览与组件登记，核对 086 分页等已有能力。

2026-11-20 `/goal 完成计划125` 授权 r1 的实施、独立复审、范围内修复和合并归档。开工基线为 AutoUI `b1444d13edceca2c5284fd31beea5b52e6947f9a`、AutoLang `dcd4fdafd412be911dc44ba1fee911a916d9c198`。实际工作树为 `D:/autostack/.wt/ui-125/{auto-ui,auto-lang}`，均为 `plan-125-dev`。同组只读构建依赖为 AutoDown `72b097006f34db3a871449f10e213820d51abef6`、AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`。

### T-01 原页与控件依赖

从 AutoOS `efcfced51b58a1174f0b4fd3c0288c589495f70e` 导出原 `pac.at` 和 `src/front/`。七页可各自作为 widget 单元，页文件没有外部 `use`，Gallery 的原路由与 `preview-card`、`codeblock`、`table` 说明组件仍在源内。下表 Git blob 固定到 `widgets-gallery/src/front/pages/`，全部对应 1071 §9 冻结矩阵的 U03 行。

| 原页及 Git blob | 原预览所用控件 | 可区分错误实现的读回 |
|---|---|---|
| accordion `a24e3b0e482b23687c507c0903a8330921ed601e` | `accordion`、`accordion-item/trigger/content` | 依次打开 item-1、item-2，前一项内容消失且展开状态翻转。再次操作 item-2 时全部收起。 |
| collapsible `f58b325f0928024b8dce6ff1f56a2822dcff7277` | `collapsible`、`collapsible-trigger/content` | 初次关闭，触发后读回打开状态和完整内容，再触发都恢复。复验按键操作。 |
| scroll `ef1a2b9d7ca6d4cda83c08220c17202b233bb4b1` | `scroll`、`div overflow-{x,y}-auto`、`div overflow-auto`、`square` | 独立检查语义竖向、样式竖向、横向和双轴四个区域，滚动前后核实际偏移和尾项是否进入视口。 |
| scrollarea `37c27ee51a05a06f1d1685da277033abb77a3ba7` | 有界 `scroll`、`col`、`separator` | 区内操作后 Item 12 可见，同时外层位置保持。 |
| pagination `93fffac4939f8928ba7c37f9bf7febb8d9ff7a21` | `pagination`、`pagination-list/previous/item/ellipsis/next` | 初始 1，下一页变 2，返回 1，选 10 后下一页仍为 10，前一页后回到 9。固定导航槽保留原 1、2、10。 |
| progress `ba418e206853fdff38d0d869100806eb9b96de84` | `progress` | 可读值 33，默认上限 100，可见填充比例 0.33。 |
| slider `14146a9a05b2737c4c14e03edc069a089da6f16e` | `slider` | 初始 50，指针或按键改变可读值，默认步长为 1，到达 0、100 后继续同向操作不越界。 |

登记依赖在 AutoUI `b1444d13` 固定。Vue 的 `WidgetRegistry::with_defaults` 与 shadcn 属性发射已有 accordion、collapsible、pagination、progress、slider 条目。AV/Iced 既有 `View::accordion/scrollable/progress_bar/slider` 可用于组件投影。PLAN-086 的 `PaginationControl` 是受控高层包；原页调用内置 `pagination` 家族，见 [分页规范](../../specs/ui/pagination.md) 的共存规则。实际验收使用原控件标签。共享 AGE 包语法与通用源准备由 105 持有；124 的已落主路由版本包含在上述基线内。

## 5. 详细设计

逐页对账发布原输入、实际模块和组件依赖，对每页记录两端可见结果、独立操作预期、失败恢复及关闭终态。组件缺口就地修复后回到原页实跑；已有证据按同源版本和实际入口复用。

### 规范增量

- SD-03 对应 AC-05。在 AutoLang `docs/specs/auto-test-case/module-selection.md` 补记固定组显式 debug/test 准备入口、工具 profile 和运行库身份。执行端按已准备收据核验，缺件时报准备错误。

七页沿 [原源归属](../../specs/ui/examples-ownership.md) 的单源合同。拟沉淀以下增量，T-05 按实际消费入口和证据核对。

- SD-01 对应 AC-01、AC-02。在 AutoLang `docs/specs/ace-migration/m2-autoui.md` 补记 AGE 组件面和包面的 `text "literal" (style: "…") {}`。CST 保留 literal 叶和属性，原有花括号内属性形式继续可用。两 profile 的身份变化由 `plan981_grammar_scope` 锁定。
- SD-02 对应 AC-02。写入 AutoUI `docs/specs/runtime/ace-package-widgets.md`，记录受控展开、滚动边界、原分页导航项、数值上限及步长。AV 按活跃会话和实例持有控件状态，A2Vue 按挂载页面持有状态。两端各从原控件输入更新，关闭重开恢复初值。Windows 的展开状态、当前页和数值范围由系统 UI Automation 观察，按冻结的实际支持面说明。VM `PaginationControl` 继续采用 [独立分页契约](../../specs/ui/pagination.md)。

## 6. 测试设计

| 用例 | 观察结果 |
|---|---|
| V-01 | 原页、组件闭包、版本和 1071 对应行齐全。 |
| V-02 | 真实 AV/Iced 窗口逐页核 §1，保适用的键盘/指针与关闭重开；根壳出现或静态截图不足以证明交互。 |
| V-03 | 经 A2Vue 生成、实际构建、浏览器运行同源页，观察相同状态及适用的错误、关闭。 |
| V-04 | 新修组件保永久回归；原页结果附构建和运行退出码、独立读回、版本及供 W1 和 A2Vue 消费的逐页状态。 |

已完成 T-01 和 T-02 的部分组件与首失败验证，实测范围见 §9。实际构建前核生产入口和平台条件，真实窗口单实例运行。若改 AutoLang，按写面选 cargo t ace-source、ace-av 和 ace-a2x 等实际受影响的固定组，与 AutoUI 消费回归同证。

## 7. 验收标准

| ID | owner_stage | 完成条件 |
|---|---|---|
| AC-01 | work | 7 页原源与所用组件可追溯，既有共享写面和复用版本已对账。 |
| AC-02 | work | V-02、V-03 的各适用原页和控件行为各有真实可复跑结果，新增组件缺口的正反例按 V-04 检查。 |
| AC-03 | review | 核逐页可达、原行为、两端证据和失败恢复，确认写面与 105/123 的交付一致。 |
| AC-04 | merge | 落主并发布逐页 AV 和 A2Vue 收据，供 105 汇总。 |
| AC-05 | work/review | `ace-source`、`ace-av`、`ace-a2x` 保持固定选择，用显式 debug/test 准备和只读运行完成正式回归。诊断现有红例，复核工具和运行库身份、缺件拒绝及原有默认入口。 |

## 8. 执行步骤

如当前工具支持原生 TODO，开工前登记下列任务 ID。

- [x] T-01 冻结七页原源和验证矩阵
  - owner_stage: work。AC-01。§4 固定原源码、控件闭包、各仓版本及独立判据，§9 记录双端共同的包入口首阻断。
- [x] T-02 关闭实际组件缺口
  - owner_stage: work。AC-02。复用已有实现，回归 §1 所列的实际行为。
- [x] T-03 实跑 AV/Iced 七页
  - owner_stage: work。AC-02。七个原页前台窗口输入、独立状态和画面验收见 §9。修复后实跑受影响的两个折叠原页，并核嵌套反例。
- [x] T-04 实跑 A2Vue 七页
  - owner_stage: work。AC-02。七页原始字节经 `auto gen`、Vue SFC/esbuild/Tailwind 构建和 Edge 交互均通过，含实际状态读回和关页，见 §9 恢复收据。
- [x] T-05 独立复审逐页原义务
  - owner_stage: review。AC-03。正反例与证据全部绑定冻结版本。
- [x] T-07 修复 AutoLang debug/test 合并门
  - owner_stage: work/review。AC-05。显式 debug 工具准备沿用固定组、nextest archive、工具收据和运行库内容身份。独立验证旧预期、合法准备、产物变更拒绝及只读运行。此项完成后执行 T-06。
- [x] T-06 正式交付并回填 U03
  - owner_stage: merge。AC-04。按仓规落主、归档与清理。AV 可先独立复审交 W1，A2Vue 沿原两端合同继续收口。

## 9. 复审记录

2026-11-20 stage: new/work | PLAN-125 r2 | 用户确认纳入 debug 合并门修复。r1 双端产品与复审证据保留。同一 `ui-125/auto-lang` 树修正式入口与已证回归缺口，变更范围见 AC-05、T-07 和 SD-03。

2026-11-20 stage: work | T-07 | AutoLang `e44e7accd`。`--prepare-only --debug-tools` 保留原固定组选择，以 debug worker、`runtime-dev` 静态库和 test archive 准备，`--run-prepared` 核工具 profile、产物与执行环境的绑定。运行库缺失或身份过期时具名拒绝，热路径不启动 Cargo。A2TS 端口预期补入 PLAN-982 已交付的 `process.run_argv_w16` 并继续检查唯一性。`test_prepare_tools` 10/10（0.143 s），`PreparedRunGate` 6/6（3.254 s）。正式三组准备 263.700 s，`t07-debug-prepared/prepared.json` 入证；三库冷准备各一次，第二组准备与直接运行态复查均 `builds=0`。在独立缺件源树三库全部 `PREPARATION_REQUIRED`，`t07-runtime-reject-missing.log`，退出 1。运行前确认共享机器上的其他 AutoUI/AV 构建结束。组级执行结果待录。

2026-11-20 stage: work | T-07 首轮正式执行 | `e44e7accd`，退出 1。ace-source 931/932（11.565 s），20,000 个连续前缀 `!` 的已有用例在 debug 线程栈上溢出。ace-av 1484/1491（39.610 s），五个真实原生用例因运行库身份失配拒绝，两例具名控制台入口的旧断言仍期待 `lower.entry`。ace-a2x 300/300，五项保持原筛选跳过。将 `RUSTUP_TOOLCHAIN` 显式加到直接预热程序的 run-debug 环境后三库复现失配。绕开 Cargo shim，直接用同归档跑 `stdlib_list_946_native_end_pop` 通过，nextest 用例 0.498 s，定位为 Cargo shim 给执行阶段插入了不同环境。

2026-11-20 stage: work | T-07 修复 | AutoLang `3e0b5985ede370f77cd005beaa926c6cd0fdd4c6`。debug 清单直接调用 nextest，沿用预热阶段的编译器环境。CST 构造器解析逐层迭代穿过连续前缀节点，保留叶及其既有分派，原 20,000 层用例未改。按 PLAN-982 改写 D-1098-01 的两例，实际选中 helper 后核返回值和 stdout，分别验证不存在的函数在 lower 拒绝、整数形参在 verify 拒绝，保留原 `main`、脚本和通用 owned 入口。待新候选正式验证。

2026-11-20 stage: work | T-07 复测与修复 | `3e0b5985e` 上五个真实原生用例全部绿，AV 1490/1491，A2X 300/300。脚本的返回值由读回证实是 `Value::Unit`，其自定义 `PartialEq` 不承认此种值的等价，现改按枚举分支核验，具名入口和 main 仍比较实际整数。Microsoft CDB 在相同 16 MiB 测试线程栈首次拦到 `0xc00000fd`，调用栈连续重复 `closure_capture::scan_shared_capture_cells`。AutoLang `c8dbf5c92` 让共享 cell 扫描与自由变量扫描迭代越过一元链，在末端维持原作用域和记录顺序，继续以同一 20,000 层用例验证。

2026-11-20 stage: review | PLAN-125 r2 | `c8dbf5c92` 首次三组同一清单全通过，准备 58.869 s，正式执行总墙钟 157.990 s。ace-source 932/932（8.126 s），ace-av 1491/1491（39.357 s），ace-a2x 300/300（80.914 s），按原选择分别跳过 1、23、5 项。20,000 层用例 0.655 s，两条控制台用例 0.187 / 0.180 s。复审发现从 debug 清单移除归档字段或错填调用序号时，旧执行端可能进入构建路径。交付候选 `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28` 在调用前明确拒绝这两种状态，并把新开关限定为有归档支持的 nextest 固定组。工具 profile、运行库、代码和用例与本轮相同。等最终清单的三组执行，作为 AV 同用例的第二次独立整组确认。

2026-11-20 stage: review | key: PLAN-125:r2 | outcome: pass | reviewed_commit: AutoUI `e99e6d99963b7df080a0bfa20b6e59eb4c439bf9` on `15fa7857`，AutoLang `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28` on `89f702282`。当前会话凭独立反例、实际输入和原始收据复审。AutoOS、AutoDown、AutoWeb 钉住 §4，SD-01/02 与 r1 复审一致，SD-03 对账 `module-selection.md` SHA256 `B9BAEA06864DADB5B0FC5F84828711EC171F9E11A5D801E618202C38AF1A7790`。ACE 控件业务入口、原源与产物的原复审边界不变。新编译器步进仅越过不携带构造器或作用域的一元节点，继续在同一操作数消费事实。AC-01/02/03/05 pass，AC-04 由 T-06 落主和交接收口。

固定组执行使用 `python scripts/fixed-tests.py ace-source ace-av ace-a2x --prepare-only --debug-tools --out <scope>` 与 `python scripts/fixed-tests.py --run-prepared <scope>/prepared.json --out <run>`，两命令退出 0。准备 14.968 s，Cargo 使用 test/dev/debug 与 panic-abort `runtime-dev`。执行前确认机器空闲，前一准备窗口等待 PLAN-1086 的 source/semantics 组结束。第三组运行期间另一个工作树开始 AutoUI 构建，既有 `scalar_semantics::full_corpus_true_chain` 为本次唯一 slow 用例。原测试筛选和测试身份与上一轮逐项相同。

| 固定组 | 执行 / 原筛选跳过 | 归档发现 | nextest 测试 | 执行调用总时，含解包与清理 |
|---|---:|---:|---:|---:|
| ace-source | 932/932 PASS，1 | 1.648 s | 9.789 s | 11.480 s |
| ace-av | 1491/1491 PASS，23 | 5.313 s | 47.934 s | 53.202 s |
| ace-a2x | 300/300 PASS，5 | 7.195 s | 140.143 s | 148.200 s |

全组执行总墙钟 227.160 s。最终收据 SHA256 `4f3e49493105fc2abbc9373d3911fb462e09abd45e90d39c2361ed78cacda8b6`，准备清单 `430eb12ca3b765b9b1571d3a1ad5329ded99000b0e26e4173887b6b54be7fc52`，源 / 输入态 `08993c13300c11553680855723e7a2088beb53c4ea1a78d2f674567517363d44`。固定配置 `443d4580de083384b42072c97c55e0932c24112ec9ceb8cc1ecd740b9513f0e4`，工具链身份 `1fb10dcd2dd67ff767f95bad0b2d205e34c03d0a346f60593f2cbc35abf4bf52`。归档依序 `df6fb3d3e6581ba72dfdd7051d259bea10d91e4199ced8bcd85bbc265b6ba170`、`63650fb7a86b57ec9748f790dd7a8b3025297831cd320bd80642af411b10c51e`、`1295bdadf94ab61e27b14f38e3f9e47b93b9e166412d1ecade69dabbfff506d8`；三段均直接执行 nextest 归档。清理阶段未单独计时。

新配置模式检查 0.004 s，清单重绑、缺归档、调用序号和运行态拒绝 1.002 s，整个 PreparedRunGate 6/6（3.547 s）。`plan125_text_literal_parentheses_keep_the_label_and_style` 0.256 s。既有深链 0.763 s；D-1098-01 的两条 AV 修订用例本轮分别 0.249 / 0.302 s，两次相同成员的全组验证均通过。未重跑该债务表提到的独立 ace-facade 用例。

2026-11-20 stage: merge | key: PLAN-125:r2 | checkpoint: prepared | outcome: pass | T-07 已过。两个最终交付候选与上述复审提交一致，完全固定组运行与 AutoUI 原 r1 两端取证、runtime/tools 回归满足 T-06 落地主线的验证前置。

2026-11-20 stage: merge | key: PLAN-125:r2 | checkpoint: landed, canonical_verified | outcome: delivered, archive and cleanup pending。AutoLang master 从 `89f70228289c314ac24b74ff96b89f15d1360167` 快进至 `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28`，AutoUI master 从 `15fa7857a411707c44ceb6e1a37d1822808bef53` 快进至 `e99e6d99963b7df080a0bfa20b6e59eb4c439bf9`。两树干净并实跑 wt-guard，分别执行 `git merge --ff-only plan-125-dev`，未改写复审提交。AutoLang master 的 select-only 与正式执行的三组、表达式和配置哈希一致；AutoUI master 的 `cargo tree --locked --offline -p auto-ui-runtime --features ace-bridge --depth 1` 退出 0，连接到 AutoLang 主检出的 ace-core、ace-av 和 ace-a2x-ts。

读回 AutoLang `docs/specs/ace-migration/m2-autoui.md` SHA256 `F045D0823403E5C86C24922DDF18783AF30B828CDE579FC32A69FFF8F2EAB386`、`docs/specs/auto-test-case/module-selection.md` `B9BAEA06864DADB5B0FC5F84828711EC171F9E11A5D801E618202C38AF1A7790`，AutoUI `docs/specs/runtime/ace-package-widgets.md` `F85E9C1A0A07FFB1F70F7C9B6083CE2F9E65072D62A98BED38FA2B8ABAC15C01`；三份在 master 与各自复审工作树逐字节一致。

2026-11-20 stage: merge | key: PLAN-125:r2 | checkpoint: archived | outcome: delivered, cleanup pending。归档文件实际位置为 `docs/plans/archived/125-autoui-u03-interactive-widgets.md`，检查了原页、105、1071、1072 和规范链接。任务 6/7，T-06 待清理后关闭。PLAN-105 §9 和计划 README 指向本档，PLAN-105 的既有编辑另外留在原工作区。

2026-11-20 stage: merge | key: PLAN-125:r2 | checkpoint: cleaned | outcome: pass | archive_commit: `41b2a0b2fd561a641723ca65b25bab7d2f0416ac`。四个工作树按绝对路径、Git 注册、预期提交、干净状态和独立 wt-guard 检查通过后移除。确认两条已落主的 `plan-125-dev`、四个工作树和空组目录均已消失。证据折入本档，已登记的 `plan-artifacts/plan-125` 经 guard 后删除，保留登记状态为 `deleted-at-merge`。AC-04 和 T-06 已交付，全项 7/7。其他进行中计划的既有主检出文件没有随归档提交。

发布产物检查。AutoOS Gallery 的现存 `gen/front/vue` 时间为 2026-09-28 21:41，早于此次交付。AutoUI 和 AutoLang 主检出的标准 release 路径未发现本次验证用的工具和 AV worker。验收基于独立 debug/test 产物。AutoLang 批量回归收据为 `2026-11-20T00:00:00Z`，`last_covered_plan_id=1098`，本次 AutoUI 编号及收据年龄未触发批量回归。

2026-11-20 stage: goal | plan_id: PLAN-125 | plan_revision: r2 | from: merge | to: done | status: archived | outcome: pass | AutoUI `e99e6d99` / AutoLang `a7ce84eff`。

2026-11-20 stage: new | PLAN-125 r1 | outcome: pass（计划及原页静态核对） | task_ids: T-01..T-06 | acceptance_ids: AC-01..04 | next: work/T-01。

2026-11-20 stage: work | PLAN-125 r1 | checkpoint: T-01 | outcome: pass | worktree: D:/autostack/.wt/ui-125/auto-ui | branch: plan-125-dev | base_commit: b1444d13 | dependency_revisions: AutoLang dcd4fdafd、AutoOS efcfced51b58、AutoDown 72b09700、AutoWeb 26d085f7 | authorization: `/goal 完成计划125` 继承全链路 | next: work/T-02。

`git archive efcfced51b58 widgets-gallery/pac.at widgets-gallery/src/front` 成功，`D:/autostack/plan-artifacts/plan-125/gallery-source.tar` SHA256 `FA7AFF78EF6F9B4F6E06ABE30F33180AC6FBB1E9AF97F0AA0D5D5B8C6EF58AEC`。使用本工作树从 `dcd4fdafd` 编出的 dev `ace_core`，探针直接调用 AV 和 A2Vue 共用的 `parse_package_source_age`，未改动七页。7/7 原页与 Gallery 壳回报 `source.syntax: expected fn, found widget`。四个对照全部符合预期：普通 widget 和花括号内属性通过；同一属性改用括号语法时包面拒绝；连字符无参数标签在两面拒绝。组件面另有 accordion 与 pagination 的连字符标签诊断。0.476 s 单独链接探针，执行 0.163 s，退出 0 表示对照符合预期。日志 `D:/autostack/plan-artifacts/plan-125/t01-admission.log` SHA256 `7CA5FABAA604312588630D20AD4E2FF3E268479131383B64D53514E3D5313025`，`ace_core` rlib SHA256 `9BCAF11D6EEACEA3F8C940C62887FB6321C5CFD1F6B0E5F4147A69400FD344D7`。首次 rustc 直链缺少 Windows SDK import-lib 路径，补 `-L native` 后成功。此轮无新增永久用例，无窗口或浏览器验收。

共享语法版本核对：105 分支的 `b8837722b` 收括号属性和连字符属性键，`aaa7763c1` 收连字符元素标签，两者当前均不在 AutoLang master。105 的 `ace_source_prep` 仍由该计划持有。125 使用现有主线入口建立独立组件对照，原页实跑待共享能力正式落地主线并复验。

2026-11-20 stage: work | plan_id: PLAN-125 | plan_revision: r1 | outcome: blocked | worktree_path: D:/autostack/.wt/ui-125/auto-ui | branch: plan-125-dev | base_commit: b1444d13 | code_commit: 21d174c1478cf27ca896fe34a540de360c0ff014 | dependency_revisions: AutoLang dcd4fdafd、AutoOS efcfced51b58、AutoDown 72b09700、AutoWeb 26d085f7 | task_ids: T-01 完成、T-02 部分、T-03/T-04 原页均未通过 | blockers: 105 正式交付共享语法 | next_responsible_role: 105 提供正式主线版本，125 从 T-02 继续。

**组件进展。** A2Vue 包模板透传 `progress` 的 `value`/`max`（默认 0/100），`slider` 发射浏览器原生 range 控件，保留 `min`、`max`、`step`、`value` 及现有样式和事件生成。通过 `auto gen` → `auto-ui-tool generate_ui` → 1072 的 `@vue/compiler-sfc`/esbuild/debug → Playwright Edge 复跑。合成页 progress 可见并读回 33/100；绑定值 3 → 4，上限 12。滑块 50 → 51，端点 0/100 锁定，指针选值 73；另一滑块 `step=3` 由 8 → 11，端点 2/20 锁定。返回对照路由后两控件恢复初值，页面关闭且无 `pageerror`。未接滑块投影的中间工具版本在 `getByRole('slider')` 失败（5.397 s，exit 1），证据 `t02-browser-negative.log` SHA256 `7B5FF4DE85A668432B1B9BF57EE9CEE053D14751C55D4228B57D4728C5670738`。

**原页失败。** 以七页原始字节作路由页，`ace_node_desktop_window --package ... --entry ... --instances 1` 7/7 在开窗前以 exit 4 拒绝，均为 `source.syntax: expected fn, found widget`（每页 0.103–0.331 s）。A2Vue 的 `auto gen` 用同源七页走真实包路由，退出 0 但产品为七个 `ace.route.page` 诊断组件。浏览器逐个进入时验证了对应路由的具名拒绝，每页通过数仍为 0。Native 合成数值控件窗见到 172 帧和关闭重开，两个事件的 `package observation produced no state row` 与空 dispatch 记录使该用例不能算通过。另有原页尚未到达的宿主行为待 T-02 实现与回验，其中 AV 当前 `PackageStateEngine` 仅读回 int，`collapsible.open: .isOpen` 的 bool 数据通路需核对。

| 会话实际范围 | 结果 | 准备与执行 |
|---|---|---|
| AutoUI `plan980_package_sfc` | 8/8，2 个旧浏览器用例未执行 | debug/test 与 `ace_node_desktop` 共用准备调用 179.227 s；执行 0.09 s、命令 0.178 s。 |
| AutoUI `ace_node_desktop` | 8/8，排除 2 个排窗用例和 1 个尚未准备 F0 provider 的 `byte_len` 用例 | ui980 独立准备 0.473 s；执行 0.65 s、命令 0.899 s。 |
| AutoUI `plan980_ace_vue_tool_wiring`、`plan124_ace_vue_routes` | 各 1/1 | 两目标共用准备 9.043 s；执行 2.52 / 0.86 s、命令 2.740 / 1.077 s。 |
| A2Vue 控件对照与七原页诊断 | 1 个真实浏览器对照及关页恢复通过，原页 0/7 | CLI 生成 1.969 s，编译和打包 0.403 s，浏览器执行及清理 1.355 s。工具冷构建 410.803 s、增量构建 23.885 s。 |
| AV/Iced 开窗前诊断与对照 | 原页 0/7；1 个数值合成窗的 AV 状态重放失败 | 本机窗口示例编译 5.132 s；合成窗调用 5.395 s。 |

测试发现和清理未单独计时，均计入各命令。新增永久用例 `p125_package_sfc_progress_preserves_percent_scale_and_bound_value` 单独调用 0.274 s（用例 0.04 s），核对原 33、缺省、非默认最大值和绑定发射；`p125_package_sfc_slider_emits_native_range_with_constraints` 单独调用 0.049 s（用例 0.03 s），核对原数值及非默认步长的 range 生成。两例均通过 10 s 限制。未更改 AutoLang；固定组、最终复审和合并检查待做。

证据根 `D:/autostack/plan-artifacts/plan-125/`。主日志 SHA256：`t01-native-originals.log` `0E5E1346CEDCDB04A92C63ADCB54264D6B39BACF0F055BFC3525660F5607592B`；`t02-native-control.json` `5D02DEF7CD74F0A94FFDB0AA86891F62346C50BEEC5A500BAEB3B1F32264F423`；`t02-vue-cli-final.log` `2852E5245251801B0DDF8DE3F88E50D59CB5B279787CE04F558867AF7A178C98`；`t02-browser-replay.log` `291C2C211E5FA04100AD2A2004E25619F6F99F3B60E89EBEBBD7C69AF6544D0C`；`t02-package-sfc-tests.log` `83F2FD9F159413A6F34F6B9F621615184E7D76ED2461FEBD5724467BB856ADC2`。AutoUI master 期间前移至 `15fa7857`（091），上述模板、宿主、工具入口与 Cargo.lock 在两基线相同；正式验收前按届时主线复核和同步。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-125/auto-ui | commit: 21d174c1478cf27ca896fe34a540de360c0ff014 | next: user/external 105 交付

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-125/auto-ui | commit: 21d174c1478cf27ca896fe34a540de360c0ff014 | next: 105 正式交付共享语法。2026-11-20 复核 AutoLang master 仍为 `dcd4fdafd`，两项语法提交仍仅见于 `plan-105-dev`，125 两仓工作树干净。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-125/auto-ui | commit: 21d174c1478cf27ca896fe34a540de360c0ff014 | next: 105 正式交付共享语法。2026-11-20 再核 AutoLang master `dcd4fdafd`，语法和 CST 投影无新提交，PLAN-105 r2 仍在 executing。三轮均遇到同一外部前置缺口，按用户确认的交付顺序保留检查点，等主线版本具备后恢复。

2026-11-20 stage: work | PLAN-125 r1 | checkpoint: T-02、T-04 | worktree: `D:/autostack/.wt/ui-125/{auto-ui,auto-lang}` | branch: `plan-125-dev` | base: AutoUI `15fa7857`、AutoLang `cf033adc4` | commits: AutoUI `1b0984f7`、AutoLang `d771f2200` | outcome: T-04 通过，T-02/T-03 待 Windows 真实输入复验。

共享前置已解除。确认 105 `ba462cee5`/`384609645` 是主线祖先，采用 `8560a6cf` 的共享交付收据。`inspect_pages` 在同步后编译 6/7，直接用诊断字节跨度定位到 `text "▸" (style: "transition-transform duration-200")`。`open: .isOpen` 投影是正常的 `self` 字段引用。AutoLang `d771f2200` 新增 literal text 属性规则，沿既有文本叶和 props 投影语义。七个冻结原页均在真实 `compile_age_package_with_scenario` 通过，投影含 `open`、glyph 和 style。证据 `t02-resume-projection.log`（SHA256 `684148B6C292384CAE4508ACB140B378BC6621E97754547F5F45E526E2B23282`）、`t02-widgets-projection.log`（`9C70996F0FF22AB2B1B1DA45F1835B098D2E8A772AF7457D1191D8A713D1ECD8`）。

**A2Vue 七页结果。** 在 `D:/autostack/plan-artifacts/plan-125/web-current/` 建空生成根，逐个核对路由页与冻结原源的 SHA256。使用小路由壳装入原页，设置 `AUTO_UI_ACE_VUE=1`、`AUTO_UI_TOOL_BIN` 后执行 `auto gen`，产出七个页面 SFC、一个数值对照 SFC 和真实 router。`browser-replay.cjs build` 编译实际生成的 `main.ts` 和 SFC、处理 Tailwind CSS，`replay` 以临时 HTTP 起 Edge。关页与清理均纳入执行计时。

| 原页 | 实际浏览器动作和独立读回 |
|---|---|
| accordion | 鼠标开第一项，Enter 开第二项且第一项关闭，Space 再收第二项。读回 `aria-expanded` 和真实可见面板，重挂回到全部收起。 |
| collapsible | Enter 打开，点击关闭。Vue `defineExpose.__state().isOpen` 为 false → true → false，与 `aria-expanded` 及内容可见性同步，重挂恢复关闭。 |
| scroll | 真实滚轮，语义竖向 `(0,234)`、样式竖向 `(0,30)`、横向 `(684,0)`、双轴 `(292,50)`。尾项 8、6、card 10、12 分别进入视口，外层偏移保持。 |
| scrollarea | 区内滚轮偏移 `(0,131)`，Item 12 进入视口，外层偏移保持。 |
| pagination | 鼠标/Enter 导航 1 → 2 → 1 → 10 → 9，读回页码和 `aria-current`。边界按钮禁用，重挂回页 1。 |
| progress | 原 `<progress>` 显示且独立读取 `value=33,max=100,position=.33`。 |
| slider | 初始 50，键盘 51、到两端后保持 0/100，鼠标选到 66，重挂回 50。合成步长 3 的对照 8 → 11，端点 2/20，关闭后页面终态可读。 |

**AV/Iced 当前证据。** 七个原页各用正常 `ace_node_desktop_window --package ... --instances 1` 开真实窗口，实际 54–78 帧，7/7 退出 0 且桥 `errors=[]`。四种 scroll 内容尺寸超出各自视口，双轴实测 960×160 对 708×128，scrollarea 192×378.4 对 192×288。工作树实现按原槽位复用 Iced 控件，资源状态按 session 和实例保有。已有宿主和错误恢复回归通过。Windows 当前有 `LogonUI` 锁屏，前台切换返回 0，现场取帧得到锁屏画面，故不把这些开窗或键盘状态机单测算作 V-02 的真实窗口交互。待解锁后逐页进行指针、键盘、偏移以及关开实测。

| 实际检查 | 结果与执行 | 准备/清理 |
|---|---|---|
| AutoLang `plan980_style_surface`、`plan981_grammar_scope` | 14/14，0.80 s，调用 1.304 s | 使用工作树 dev `ace_core`，`rustc --test` 单独准备 3.419 s；发现/清理未单测。 |
| AutoUI `native_widget_keyboard_targets_the_bound_disclosure_and_bounded_slider` | 1/1，0.03 s，调用 0.120 s | `--lib` 与下面三集成目标共用准备 27.763 s。 |
| AutoUI `plan980_package_sfc`、`ace_node_desktop`、`plan124_routes` | 9/9、8/8、2/2，执行 0.11/0.64/0.53 s，调用 0.243/0.787/0.564 s | 排除 2 个既有旧浏览器用例、2 个既有排窗用例和 1 个未准备 F0 的 byte_len 用例。路由首次未传 provider 编译器指纹而退出 101，准备后复跑 2/2。 |
| AutoUI `plan980_ace_vue_tool_wiring`、`plan124_ace_vue_routes` | 各 1/1，执行 2.55/0.80 s，调用 2.764/1.057 s | 独立准备共 49.552 s。 |
| A2Vue 原页与恢复 | 7/7、数值对照与关闭通过，replay 2.121 s，调用 2.177 s | `auto gen` 3.002 s，SFC/JS/CSS 构建调用 0.946 s。关页与服务回收计入调用，发现未独立计时。 |
| AV/Iced 原页开窗 | 7/7 开窗并退出，2.803–2.949 s/进程 | Iced 开窗检查构建 63.863 s，最终工作树冻结构建 16.777 s，测试已准备 ui980；锁屏导致真实指针、键盘及关开尚未验收。 |

本轮新增用例均通过 10 s 门槛。AutoLang `plan125_text_literal_parentheses_keep_the_label_and_style` 核 glyph、style 与下一叶，独立调用 0.213 s（执行 0.10 s）。AutoUI `p125_package_sfc_widget_slots_keep_their_state_and_native_roles` 核触发、ARIA、槽位及边界发射，独立调用 0.058 s（执行 0.04 s）。`native_widget_keyboard_targets_the_bound_disclosure_and_bounded_slider` 核焦点空态、布尔值、非默认步长和两端钳制，独立调用 0.120 s（执行 0.03 s）。模板回归测试源码曾误把新用例置入前一个原始字符串，修正后重新构建与执行均通过。

临时证据根 `D:/autostack/plan-artifacts/plan-125/`，`t04-current-clean-generate.log` SHA256 `EF07AD4CF9E16039C84D56571ABA2F523E697EC15076C5F934D6353E30BD6EFE`，`t04-current-clean-build.log` `FBCD3B879B5E0C90E953DBFB73CE5CF90B586B5D55658ED49CB3FC358CB42F3D`，`t04-current-clean-replay.log` `9205841F279BFD3B7050634B5265D8F380130416BA83579BF481E858DCC72670`，浏览器 JS `browser-current/app.js` `374CB375AFB828ECD71D36BE139CDF9EC2181637B7344CAF8AAF647417BA2F27`，`t02-native-active-layout.log` `013C748FA5634F1CAB862B1FEC3711CF409235C7C66228BD956E765C76FDDBEB`。拟规范增量为 AGE 双面文本 literal 括号属性，以及 ACE 包视图的组件状态、滚动和会话生命周期。原源归属、PLAN-086 独立 `PaginationControl`、PLAN-105 在途接口归属均按现行契约。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-125/auto-ui | commit: 1b0984f7；AutoLang d771f2200 | next: 解锁 Windows 后继续 T-02/T-03，完成后进入 T-05。

2026-11-20 stage: work | PLAN-125 r1 | checkpoint: T-02..T-04 | AutoUI `1d63b83926390655cb4ec32584d2cc12fd53692d` on `15fa7857` | AutoLang `d864428a41cfbe19919f8a9d6d3e7a2a8b8315b8` on `274f86183` | outcome: T-04 复跑通过，T-02/T-03 等待 Windows 前台。

**Iced 与无障碍。** Windows 短暂解锁时，七个原页在自有 Iced 前台窗口显示、截图和退出，证据 `t03-unlocked-layout.log`（SHA256 `06A11DDFD7655FF84A85DB9C5400837390F2858EF8E46C753928AF3E860916E0`）。输入开始前再次锁屏。补用现有 AccessKit 适配器发布控件角色、展开值、选中值和数值范围。Windows 适配器要求首次显示前安装，修正初始化顺序后可读到真实窗口 `AccessKit` 根。锁屏下 UI Automation 仍未读出具体控件。新增回归核可访问状态与原绑定共享、滑块步长、进度读值和旧会话动作拒绝。现场鼠标、键盘、控件读回和关闭重开继续待解锁验证。

**重新同步与检验。** 1085 合入的包绑定、CST 与 AV 代码触及本计划入口。125 AutoLang 工作树同步到 `274f86183` 后两个 AGE profile 的 grammar ID 与此前 125 值相同。七份原页包投影及 A2Vue 生成均用新基线重新验证。生成前以 SHA256 对比七份路由源与冻结原源。SFC 构建第一次因环境未提供既有 `tailwindcss-animate` 目录而失败，补上已准备的 Gallery `NODE_PATH` 后复跑通过。Edge 七页的状态、输入、滚动、数值边界及关页全部通过。

| 会话实际范围 | 执行 | 准备与清理 |
|---|---|---|
| AutoUI `native_widget_` | 初次和同步后均 6/6。同步后执行 0.03 s，调用 0.105 s | 首次准备及调用共 56.692 s，同步后准备 60.193 s。 |
| AutoUI `native_accessibility::tests` | 16/16，23.98 s，调用 24.175 s | 复用上述产物，清理计入调用。 |
| AutoLang `plan980_style_surface` 与 `plan981_grammar_scope` | 同步后 14/14，0.39 s，调用 0.458 s | `rustc --test` 准备 0.556 s。 |
| 七页 AGE 真包入口 | 同步后 7/7，调用 0.245 s | 探针准备 0.498 s。 |
| AV/Iced 更新构建 | debug 编译退出 0，待前台执行 | 同步后准备 71.340 s，F0 provider 准备 0.228 s。 |
| A2Vue 原页复跑 | 同步后 7/7，Edge 与服务回收 2.169 s | 生成工具增量准备 68.506 s，CLI 生成 3.527 s，SFC 构建准备 0.478 s。首次 SFC 构建辅助依赖缺失退出 1，调用 0.531 s。 |

发现和清理未独立计时。新增 `ui::ace_node_desktop::widgets::tests::native_widget_accessibility_shares_live_values_and_rejects_closed_sessions` 在同步版本独立调用 0.185 s，执行 0.03 s，符合 10 s 门槛。固定组已检查静态选择，尚未执行。证据目录 `D:/autostack/plan-artifacts/plan-125/`。`t03-uia-raw-walker.log` SHA256 `AF5BF977BABD1C4271B027C14527D76EF5928847A56494BE1293E5ACA58FB846`。`t03-native-a11y-tests.log` SHA256 `D15FC82F88E41CC7C73958459113B755DDB9FD820F36B4264E866F5E3E41ABDC`。`t03-a11y-existing-regression.log` SHA256 `4F1BD72705C9706CE5F1CF11B1A12CEBE8DA4501FE980B72E007E21F698743D1`。新基线证据见 `t02-rebased-{grammar,originals}.log`、`t03-rebased-native-{build,a11y-tests}.log`、`t04-rebased-{generate,build-ready,replay}.log`，最后者 SHA256 `89D823F50BDF7136D144F8FEAA8524B64E46D41A25107E27D02DFDE0E589129F`。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: waiting | commit: AutoUI `1d63b839` / AutoLang `d864428a4` | next: Windows 前台恢复后完成 V-02，再进入 T-05。

2026-11-20 stage: work | PLAN-125 r1 | checkpoint: T-02/T-03 | AutoUI `5fef8056` | AutoLang `d864428a4` | outcome: Windows 系统层读回和作用链得到实证，V-02 仍在执行。

同一个锁定桌面上，既有 PLAN-068 窗口可以枚举其 AccessKit 控件。125 的原初读回曾先遇到窗口无障碍树初始化延迟，部分短探针超时或在关闭时持有旧控件。后续按本进程 HWND、`AccessKit` framework ID 和实际子节点枚举验证。分页按钮的 `selected` 属性不映射到 Windows 的 Button SelectionItem，改成 `AriaCurrent::Page` 后，Windows `CurrentAriaProperties` 的 `current=page` 与 Vue `aria-current` 同义。

干净版本五页系统读回通过。初始 accordion 三项均收起，collapsible 唯一触发项收起。分页 1 为当前页、2/10 非当前页，Previous 禁用。进度条只读 33/100，滑块为 50、范围 0..100、步长 1。另发真实 UI Automation 动作核对四个更新，`Expand` 推进 accordion 与 collapsible 的展开值，分页 `Invoke` 从 1 进入 2，滑块 `SetValue(68.25)` 按步长读回 68。四个宿主的结束收据独立回读真实控件状态，均退出 0 且无桥错误。

| 本轮实际检查 | 结果及耗时 |
|---|---|
| PLAN-068 真实 Windows 窗口对照 | 读回 AccessKit 标题、按钮、输入等，退出 0，调用 3.402 s，准备 6.071 s。 |
| `native_widget_accessibility_marks_only_the_current_page` | 新增 dev/test 回归，1/1，独立调用 0.112 s，执行 0.03 s。 |
| `ui::ace_node_desktop::widgets::tests` | 3/3，调用 0.109 s，执行 0.03 s，准备见 `t03-uia-current-page-tests.log`。 |
| 五个原页 Windows UIA 初值 | 5/5，每页进程执行和回收 8.362–8.405 s。 |
| 四个原页 Windows UIA 更新与宿主状态 | 4/4，手工诊断回放的读回在 5.496–5.649 s；保留 10.2 s 宿主窗口以写结束收据，进程调用及回收 10.470–10.484 s/页。该回放不加入永久测试集。 |

发现和清理未单独计时。证据 `t03-uia-current-originals.log`、`t03-uia-transition-full.log`，干净 Iced 二进制 SHA256 `9ADC5BBF34D8821B7D9937CE4432CA9E112F06D67FFCC1725DAC3D2D04A102C3`。AutoLang 主线后续 `89f702282` 相对 `274f86183` 只有 1085 归档收据变动。A2Vue 继续采用 round 5 的真实七页证据。

2026-11-20 stage: work | PLAN-125 r1 | checkpoint: T-02/T-03 | code: AutoUI `5fef80564ee0f6f07e24d4593f110a524879e0f2`、AutoLang `d864428a41cfbe19919f8a9d6d3e7a2a8b8315b8` | outcome: pass | next: final review。

Windows 默认桌面恢复，使用冻结原页和预准备 F0 provider。运行 `native_probe.py` 在各自 Iced 窗口前台发送真实 Windows 鼠标、键盘和滚轮事件，每次确认前台 HWND 属于本次探针。用该窗口客户区截图、Windows UI Automation 和 `NodeDesktopBridge` 的会话收据完成独立读回。七个窗口运行 307–329 帧、退出 0、无桥错误。每页分别得到关闭的原会话和不同 ID 的重开会话，随后退出真实窗口。

| 原页 | AV/Iced 前台验收 |
|---|---|
| accordion | 点开第一项，Tab 和 Enter 开第二项并关闭第一项，Space 再收起，焦点与展开状态读回一致，重开全部收起。 |
| collapsible | 鼠标打开，Tab 聚焦，Space 关闭，Enter 打开，展开值与正文可见性一致。`isOpen` 在旧会话为 true，重开为 false。 |
| scroll | 语义竖向 `(0,216)`、样式竖向 `(0,120)`、横向 `(644,0)`、双轴 `(252,32)`，截帧可见 8、6、card 10、12。区内操作保留外层起点，可见外层定位动作另行记录。重开四组偏移均为 0。 |
| scrollarea | 区内滚轮偏移 `(0,90.400)`，Item 12 进入视口且外层标题保持位置，重开偏移为 0。 |
| pagination | 点击和 Enter 得 1 → 2 → 1 → 10 → 9，边界按钮禁用，Windows `current=page` 与当前可见原页码一致。第 9 页由正文页号读回，重开回 1。 |
| progress | UI Automation 和文本均显示 33/100，截图轨道色宽比 `.3277`，重开仍为 33。 |
| slider | 指针 50 → 67，Tab 聚焦，Home、Left、Right、End、Right 读回 0 → 0 → 1 → 100 → 100，重开为 50。 |

debug build 准备 21.476 s。单页执行含窗口回收 7.580–7.792 s，断言与收据哈希调用见 `t03-foreground-assertions.log`。首次未获 Windows 前台，无输入结果。取得前台后一个瞬态失焦使首次 accordion 回放退出，独立取窗像素及每步验证前台后通过。第一次全量动作证实 slider 指针目标偏离轨道，横向 scroll 移到 480 尚不足以显示 card 10，修正测试坐标和滚轮量后复跑二页。临时 `PostMessage` 探针只验证了键盘，未通过鼠标及滚动，最终不用于 V-02 判定。临时代码诊断已移除。

前台程序 SHA256 `CB27681604B08077E52A29A764BE48F8EF8DDAE8F69CBF3418AF63F6ADB86837`，`t03-foreground-capture.log` `8F72C1C0A7EFD2324ACB84CAA0861765AFCBA3CB7B7D0D2A6E900B220A14A7C3`，`t03-foreground-targeted-replay.log` `5AE4EDF71516D159407590A0CFC502E15C09DCC2CEC469222EDA71B0B81E3EA0`，`t03-foreground-assertions.log` `B9BABA38C9A43E3C56CA92266E9F71B4EA9F513BFFE32BC5C5BD3ECE13676B42`。原源、PNG 与单页 JSON 在 `D:/autostack/plan-artifacts/plan-125/`。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | commit: AutoUI `5fef8056` / AutoLang `d864428a4` | next: T-05。

2026-11-20 stage: review | scope: final | reviewer: 当前会话从原始结果和独立反例重建复审，未分配另一会话 | PLAN-125 r1 | outcome: needs_fix | reviewed: AutoUI `5fef8056` on `15fa7857`、AutoLang `d864428a4` on `274f86183` | AC-01 pass、AC-02 needs_fix（F-125-01）、AC-03 待复审、AC-04 待合并。两工作树已在同一路径复核，证据间无源码漂移。SD-01/02 草案在证据目录 `review-spec-autolang.md` / `review-spec-autoui.md`。正式落主固定组与现行规范发布由 merge 收口。

**F-125-01（T-02、T-03，AC-02）**。`review-nested.at` 在同源 ACE 包预览的折叠面板内放 slider 与 progress。输出窗口显示两个收起的触发项。预期 Windows 只见两个 Button，实际枚举到 Button、Slider、ProgressBar、Button，且交互顺序把 accordion 的内容排在触发项之前。复现 `t05-review-nested-negative.log`，程序退出 0，独立断言退出 1，调用 8.256 s。`Renderer::items` 为消费原文本叶，在关闭的面板也投影子节点，并将其加入同一 `targets`。验收要求关闭内容在画面、键盘和可访问面同步。修复后复跑上述反例、打开的正控、子节点顺序和关闭后旧节点请求拒绝。七原页当前面板只有文本叶，原结果保留供影响对照。

独立选择语法反例：`review_grammar.rs` 验 `text "escaped: \"A\"" (style: "text-sm") {}`、相邻旧属性体和后一裸叶，组件与包两面保持 literal、style 和后续叶。缺属性值时，两面具名拒绝（`source.age.structure` / `source.syntax`）。准备 0.439 s，执行调用 0.152 s，均退出 0。浏览器复用代码一致的同步产物，Edge 再次 7/7，执行及清理 2.340 s，未重建。证据 `t05-review-grammar.log`、`t05-review-browser-replay.log`。本次复审暂无新增永久用例。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 7 | from: review | to: work | status: executing | outcome: needs_fix | finding: F-125-01 | next: 关闭面板的可达性修复、定向测试和重审。

2026-11-20 stage: work | PLAN-125 r1 | finding closure: F-125-01 | code_commit: AutoUI `565e5802fa401312195adc7437e56f2e6a0ce84f` | outcome: fixed。消费关闭面板的原文本叶时回收其控件目标。accordion 的触发项可访问顺序置于自身已打开的内容之前。访问动作按当前树重新检查，因此关面板后到达的旧 slider 请求被拒。八项永久用例的新义务和计时已逐项记录在前序工作收据与下表。

2026-11-20 stage: review | scope: final | reviewer: 当前实施会话按实际代码、原始收据和独立反例复核 | PLAN-125 r1 | outcome: pass | branches: `plan-125-dev` | paths: `D:/autostack/.wt/ui-125/{auto-ui,auto-lang}` | reviewed: AutoUI `565e5802fa401312195adc7437e56f2e6a0ce84f` on `15fa7857a411707c44ceb6e1a37d1822808bef53`、AutoLang `d864428a41cfbe19919f8a9d6d3e7a2a8b8315b8` on `274f86183bfb5dce7118e95ccbd517c392873bdd`。AutoDown `72b097006f34db3a871449f10e213820d51abef6`、AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`、AutoOS `efcfced51b58a1174f0b4fd3c0288c589495f70e`。两个待合并工作树干净。

| AC | 结果 | 独立判据及绑定结果 |
|---|---|---|
| AC-01 | pass | T-01 七个 AutoOS blob 与双端输入同源。105 共享语法两个提交是主线祖先，125 单补 literal 括号属性，正负对照均具名。 |
| AC-02 | pass | T-02–T-04 两端七页行为、适用输入、读回、边界及关开均通过。修复后重跑前台 accordion 和 collapsible，十张代表帧与上一轮逐像素一致。其余五页代码路径与 A2Vue 产物按修复 diff 对账后复用。 |
| AC-03 | pass | F-125-01 由实际 HWND 点击链和 Windows UIA 正负控关闭，永久断言核焦点顺序、展开后值和关后旧请求拒绝。关联包路由旧分支、错误恢复和队列会话隔离通过。 |
| AC-04 | pending | T-06 在固定组与各仓落地主线后发布供 105/U03 消费的七行双端收据，归档并清理。 |

| 定向验证 | 实际结果与执行 | 准备、发现和清理 |
|---|---|---|
| `ui::ace_node_desktop::widgets::tests::native_disclosure_excludes_closed_panel_controls_from_keyboard_and_accessibility` | 新增 dev/test 用例，独立 1/1，执行 0.03 s，调用 0.108 s，符合 10 s 限制 | 与四个控件回归共用准备 37.221 s。 |
| `ui::ace_node_desktop::widgets::tests` | 4/4，执行 0.03 s，调用 0.109 s | 复用上述产物。 |
| `review_nested_uia.py collapsible accordion` | 2/2，分别 8.783、8.769 s。关闭均仅两个触发项，打开的 slider 45 和 progress 35 位于各自触发项后，退出 0 | 真窗口准备 22.747 s，执行与清理调用 18.001 s。 |
| 修复后原 accordion、collapsible 前台 | 2/2，7.657 / 7.666 s，310 / 323 帧，真实命中、焦点、展开、会话重开与客户区截帧符合原合同 | 复用修复版程序。首次逐项快照比较受关闭瞬态焦点不同影响，按计划的实质焦点及展开断言独立修正，十张代表帧无差。 |
| `plan980_package_sfc`、`ace_node_desktop`、`plan124_routes` | 9/9、8/8、2/2，执行 0.11 / 0.62 / 0.54 s，调用 0.247 / 0.770 / 0.679 s | 共用 debug 准备 10.242 s。排除 2 个旧浏览器、2 个旧排窗和 1 个旧隐式 F0 构建用例。实际原页与预准备 F0 走专门验证。 |

发现与回收未单测。修复版窗口 SHA256 `25B94C02C48689E14A1D98E22B22BDD3A76BA5D963C54C7ECEC79184C3884E5B`。定向证据 `t05-review-nested-positive.log` `AC38C25CDF8D93BC88B53B8543494B1CB5E1393A22892AE7E5FB157F8774CCAC`，源 `review-nested.at` `910E5215F897F58D009E91D90897DCD7570E0DD8210F405E65187426D8060826`，测试 `t05-nested-repair-tests.log` `C5F984E726B9F4CF85D72614F81C01B48CF912EC25163205CC895B3B1B18E833`，消费回归 `t05-review-consumer-regression.log` `97D33ADF0E6DC5FC8FD1B79C4E369CFE94BDDB936B04F44E6CBBEECE5FB747D4`，修复后二页重放及断言 `t05-review-native-originals{,-oracle}.log`。

SD-01 复审 `docs/specs/ace-migration/m2-autoui.md` §4（输入 SHA256 `61244FED365DF7ACD0CF2102833BBE938F94869267A7385ED48F495A0D4A7B9D`），插入增量 `review-spec-autolang.md` SHA256 `B765A5BB53E2475C00AED152AEB7DC6E09E0F3E36A16AD8898545F223CB93372`。SD-02 新增 `docs/specs/runtime/ace-package-widgets.md`，草案 `review-spec-autoui.md` SHA256 `F85E9C1A0A07FFB1F70F7C9B6083CE2F9E65072D62A98BED38FA2B8ABAC15C01`。核现行 ACE 路由生命周期与独立分页条目，规范保持各自的消费入口。U03 进度以 105 消费行更新，`touched_goals` 无独立规范改写。待合并门为 AutoLang 受影响固定组 `ace-source`、`ace-av` 与 `ace-a2x` 的冻结版本、AutoUI 的受影响 runtime/tools 消费面，以及两仓落主后的规范读回和清理。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 8 | from: work/review | to: merge | status: reviewed | outcome: pass | commit: AutoUI `565e5802` / AutoLang `d864428a4` | findings: F-125-01 closed | next: T-06。

2026-11-20 stage: merge | key: PLAN-125:r1 | checkpoint: prepared | outcome: pending gate | delivery_candidates: AutoUI `e99e6d99963b7df080a0bfa20b6e59eb4c439bf9`、AutoLang `19bc4b60d0d508c5424f45593078e276475c734f`。AutoLang 主线前移到仅有 1085 归档收据的 `89f702282`，等价重放把 `d864428a4` 改为 `c3d8979f8`，`git range-diff` 输出 `=`，见 `t06-autolang-rebase-equivalence.log`。两个候选头相对已审代码只带相应的规范提交，两工作树干净。

规范候选已经提交到各自 worktree。AutoUI `docs/specs/runtime/ace-package-widgets.md` 源 SHA256 `F85E9C1A0A07FFB1F70F7C9B6083CE2F9E65072D62A98BED38FA2B8ABAC15C01`，两条本仓链接已核对。AutoLang `docs/specs/ace-migration/m2-autoui.md` 源 SHA256 `F045D0823403E5C86C24922DDF18783AF30B828CDE579FC32A69FFF8F2EAB386`，核对为已审插入。两个目的 worktree 均通过 `wt-guard.ps1`。

固定组静态选择为 `ace-source`、`ace-av`、`ace-a2x`，`t06-fixed-group-selection.json` SHA256 `F2F15F33FC5BF46FE346B36BCA1EC161C76778DD45C86C354583B542BEBAB667`。`scripts/test-groups.toml` SHA256 `443D4580DE083384B42072C97C55E0932C24112EC9CEB8CC1ECD740B9513F0E4`，`scripts/test-governance/toolchains.json` `B4FA1D095FA698DC649A2E78994F673D0B0896610406C88A8266A591F8395BCB`。静态选择内 `abi`、`a2x`、`store-gc` 指向 release，`text-runtime`、`data-runtime` 指向 `runtime-release`。`native-runtimes` 自身是 debug，预热执行 collections、sched、typed 的三个 builder，冷路径均硬编码 `--profile runtime-release`。已核固定组入口只有选择、准备及准备后执行等参数，工具 profile 从仓内配置读取，无本次调为 debug 的公共入口。

2026-11-20 用户确认「继续只用 debug/test」。正式 AutoLang 固定组尚无覆盖该候选版本的准备清单。只读调用 driver 的 `find_matching_manifest` 返回 `None`，候选 source state `d1fb2b9e6071202298c5a8d113e11f96310a3d20a296c062b434894454f57ab7`，见 `t06-fixed-group-preparation-audit.log`。AutoLang PLAN-1098 §9 已做 A/B，`ace-a2x` 使用 debug worker 比 release 多出两条红例，因此现配置有意保留 ABI/A2X 的 release profile。正式组状态 pending，未触发 release 准备。共用 debug 准备流程及既有红例的修复涉及 125 产品范围外的测试设施，需决定是否扩范围。现行 Plan 状态仍为 `reviewed`。

设备占用检查先见 PLAN-109 原生 AssetPicker 进程，随后确认 PLAN-1086 `cargo t case-av-language-parity --prepare-only` 正在准备。准备与执行前复核机器空闲，再启动 AutoUI 影响面最后两个工具用例。`plan980_ace_vue_tool_wiring` 和 `plan124_ace_vue_routes` 均 1/1，执行 2.53 / 0.76 s，调用 2.747 / 0.989 s，共用 debug/test 准备 31.726 s，发现和清理未单计，无新增用例。复用同代码和依赖的 19 项 runtime 消费回归及七页双端收据。证据 `t06-autoui-tool-gate.log`。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | round: 9 | from: merge | to: merge | status: reviewed | outcome: waiting | checkpoint: prepared | next: AutoLang 三个固定组的 debug/test 入口与可验证产物。两个候选保留在各自工作树。

2026-11-20 stage: merge | checkpoint: debug 定向复验 | AutoLang `19bc4b60d`。PLAN-1098 §5.17 的 A2X debug 红例实测原因为 `cl -? spawn: Deadline`，其 §5.27 随后把 MSVC 探测时限从 10 s 调到 30 s。PLAN-1108 §4.3 将准备 profile 重验列为后续事项。当前固定组的 `ace-a2x` 未启用 `test-toolchain`，这两项 C 认证现归显式工具链验证。为核旧依据在当前候选上的适用性，空闲时显式使用 `cargo test -j4 --offline --locked --manifest-path experimental/ace/Cargo.toml -p ace-a2x-worker --test conformance --features test-toolchain --no-run` 准备 debug/test；运行该 test 可执行文件并串行过滤 `worker_ct_frontdoor_958::worker_ct_958_c_`。二项分别为 `worker_ct_958_c_certifies_seven_independent_oracles` 和 `worker_ct_958_c_rejects_wrong_oracle_and_reference`，2/2，275 项过滤。准备 51.854 s，运行调用 19.097 s 含回收，单项及发现未独立计时，无新增用例。debug worker SHA256 `9DBA080ACDFDFA50F7102BDC010B3C271589443D5A9FA9EA0027B2B86FE1F19E`，`t06-a2x-debug-certification-probe.log` SHA256 `4FF0A6CF08FE56C041F5C90184D1F18EC9F2682CB9E270F2EE4A613562C1954A`。

本次串行正反控通过，说明旧的探测超时在此配置未复现。三个正式固定组仍待现有仓库提供符合用户要求、带完整工具身份的 debug/test 准备入口。共用测试设施扩围问题待用户决定，两仓交付候选及已审规范均保留。

2026-11-20 stage: merge | checkpoint: debug 完整选择探针 | AutoLang `19bc4b60d`，source state `d1fb2b9e6071202298c5a8d113e11f96310a3d20a296c062b434894454f57ab7`。在空闲机器上复用 `fixed-tests.py` 的 `build_archive` 与 `run_rust`，保持 `ace-a2x` 的六包、默认 feature、nextest 配置和原筛选。构建 dev/test archive 后，以其 debug `a2x-worker.exe` 设置 `AC_A2X_HIR_WORKER`，仅从 archive 枚举及执行。准备 31.338 s，运行段 77.100 s 含发现及回收，nextest 测试 60.534 s。结果 300 个已执行中 299 PASS、1 FAIL，5 按原筛选跳过，无新增用例。唯一失败是 `ace-a2x-ts::ports::tests::canonical_fingerprint_is_stable_and_binds_runtime_sha`，共享端口表实际 72 项、该测试仍预期 71。PLAN-1098 §5.21、§5.28 已记录该项在两种 profile 下失败，125 未改变两侧端口表或断言。

worker SHA256 `C33222B585C58E4670EB1CEAE5917E37F3451172AF2F29A4E0B011EF2B25EB7E`，`t06-a2x-debug-full-selection/probe-result.json` SHA256 `E65AC253940120CECCA2648B45EEEC89F3463C86A3732B5DEF4A20C7359D1559`。本轮完整选择探针提供 debug 执行证据。正式准备配置与工具身份仍待解决，正式三组及落主待办。

stage: goal | plan_id: PLAN-125 | plan_revision: r1 | from: merge | to: merge | status: reviewed | outcome: blocked | checkpoint: prepared | next: 共用 debug/test 合并门就绪并通过后从 T-06 恢复。两仓工作树及交付候选保留。

### U03 逐页交付收据

原页使用 §4 固定的 AutoOS `efcfced51b58a1174f0b4fd3c0288c589495f70e` blob。AV/Iced 在前台原窗口从真实输入读回，A2Vue 从同一原页经公开 `auto gen`、SFC 构建及真实 Edge 读回。每页的精确证据、退出码、原页哈希、关页和重开结果见本节前面的执行表和复审记录。

| 1071/U03 原页 | AV/Iced | A2Vue/浏览器 |
|---|---|---|
| accordion | PASS。鼠标与键盘切换独占展开、再收起；UI Automation 同步；重开全收。 | PASS。点击、Enter、Space 和实际可见面板与 `aria-expanded` 同步；重挂全收。 |
| collapsible | PASS。鼠标、Space、Enter 驱动展开；正文与 UI Automation 同步；重开关闭。 | PASS。Enter 打开、点击关闭；受控 `isOpen`、`aria-expanded` 与正文同步；重挂关闭。 |
| scroll | PASS。四个独立区域实滚到对应尾项；外层位置可区分；重开各偏移归零。 | PASS。四区真实滚轮、双轴偏移与尾项可见；区内滚动保持外层位置。 |
| scrollarea | PASS。区域内滚轮后 Item 12 可见；外层标题保持位置；重开归零。 | PASS。区内偏移 131，Item 12 可见，外层位置不动。 |
| pagination | PASS。点击、Enter 和页码状态按 1 → 2 → 1 → 10 → 9 更新，检查边界并重开至 1。 | PASS。同一序列、原导航槽、`aria-current` 和两端禁用；重挂为 1。 |
| progress | PASS。系统读回 33/100，实际填充约 33%，重开仍为 33。 | PASS。可见原进度条，DOM 读回 33/100、比例 0.33。 |
| slider | PASS。鼠标从 50 拖到 67，键盘边界 0 和 100、步进 1，重开为 50。 | PASS。原值 50，鼠标和键盘更新，步进和 0、100 边界成立，重挂为 50。 |

供 PLAN-105 合并到 U03 原十四页矩阵。AV 行供 1071/W1 汇总，A2Vue 行供 1072 与 CLI 联验。归档交付身份以 §9 的 r2 合并检查点为准。

## 10. 待澄清事项

均已关闭。PLAN-105 先交付的括号属性和连字符支持已入 AutoLang master，125 基于该正式依赖完成七页双端验收。用户追加授权的 debug/test 合并门通过，见 §9 的 r2 收据。
