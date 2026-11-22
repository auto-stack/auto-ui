---
plan_id: PLAN-123
status: archived
completion_kind: delivered
feature_name: U06 布局与 Kitchen Sink Widgets 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 7
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978]
supersedes_spec_components: [docs/specs/runtime/ace-route-page-loading.md, docs/specs/runtime/ace-package-widgets.md, docs/specs/ace-migration/m2-age.md]
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-123 U06 布局与 Kitchen Sink Widgets 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [108](../108-autoui-u06-complex-interactive-apps-av-vue.md) 的 U06 原矩阵承接本页所列行。 [原分母与交付映射](../README.md)。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。关联 Widgets 为 [absolute](../../../../auto-os/widgets-gallery/src/front/pages/absolute.at)、[alignment](../../../../auto-os/widgets-gallery/src/front/pages/alignment.at)、[center](../../../../auto-os/widgets-gallery/src/front/pages/center.at)、[col](../../../../auto-os/widgets-gallery/src/front/pages/col.at)、[flex](../../../../auto-os/widgets-gallery/src/front/pages/flex.at)、[grid](../../../../auto-os/widgets-gallery/src/front/pages/grid.at)、[grid-span](../../../../auto-os/widgets-gallery/src/front/pages/grid-span.at)、[position](../../../../auto-os/widgets-gallery/src/front/pages/position.at)、[responsive](../../../../auto-os/widgets-gallery/src/front/pages/responsive.at)、[row](../../../../auto-os/widgets-gallery/src/front/pages/row.at)、[sizing](../../../../auto-os/widgets-gallery/src/front/pages/sizing.at)、[spacing](../../../../auto-os/widgets-gallery/src/front/pages/spacing.at)、[kitchen-sink](../../../../auto-os/widgets-gallery/src/front/pages/kitchen-sink.at)。

本计划逐页验收原 Widgets Gallery 布局与组合页面。

## 2. 架构与跨仓职责

AutoUI 持原项目、Widget、Iced 宿主与 Vue 消费端。AutoLang 的公共语言与领域接缝由现有 owner 单写，本计划在同组接入使用路径和永久回归。AutoOS Widget 使用原页和依赖，归其组件 owner。先验布局页，再按 kitchen-sink 实际组合项冻结对应组件来源。逐页独立交 AV/Iced 收据，组合页引用已有资源/组件版本。

## 3. 技术与实现入口

目标入口为 AutoOS widgets-gallery/src/front/pages 的原页面及其组件闭包。AV 走现有受检 AGE 和 ace_node_desktop 生产宿主。A2Vue 走现有 ace_package_sfc、Vue/TS 与 auto-ui-tools 并用真实浏览器验收。有待补的能力按 §5 冻结后实施。使用 D:/autostack/.wt/ui-123/{auto-ui,auto-lang}，每仓单一 plan-123-dev，附属仓按实际输入钉版本。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

布局页与生产桌面入口已在 ui-123 工作树推进。完整原画廊与 kitchen-sink 接 114 所需的树组件、包闭包和受检事件，以及 107 实际使用的图表、handler 和画廊页接面。先把路由宿主/主题和独立布局页可复用证据固定，再逐项接入两边的已审版本。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

## 5. 实施与交付边界

T-01 对本计划范围冻结每条来源、两端、原运行模式、资源、独立预期、首阻断与 owner。已有语言组和组件先证实能力，缺具体能力时明确负责计划、版本和最小复现。AV 达成后独立复审并可正式交给 W1。A2Vue 依据其适用源及真实浏览器证据继续关闭。其余 demo 可独立排程，同组范围以 README 交付表为准。

### 原入口与验收输入

由原 `widgets-gallery/pac.at` 启动，两端使用同一份 `src/front/app.at` 的路由，入口默认 1440×900。对每条路由从原侧栏进入，观察该页而且跨页返回，再沿各行的特有判据验收。布局与组件页无需文件根、写权限或业务后端。图标资源沿原 `src/front/public/icon.png`，组件闭包沿原 `official` 清单及实际引用，Kitchen Sink 的 AutoDown 编辑器沿 PAC 声明的本地 `@autodown/engine` 0.5.0。缺宿主数据的 `workspace_preview` 按原页的提示与回退验收，外链和嵌入保留原配置。

| 原路由及页面 | AV/Iced 及 A2Vue 原判据 |
|---|---|
| `/absolute` · `absolute.at` | 三个预览。卡片关闭按钮和行内徽标浮于常规内容之上；无 z 层的对照在 AV 留在流内。 |
| `/alignment` · `alignment.at` | 两个预览。对比 3×4 的交叉轴/主轴矩阵和 around/evenly 的边缘与内部间隔。 |
| `/center` · `center.at` | 四个预览。显式 Center 和 flex 类都产生居中，约束宽度内仍保留两轴对齐。 |
| `/col` · `col.at` | 六个预览。纵向顺序、间距梯度、横向对齐、定高分布与行中嵌套。 |
| `/flex` · `flex.at` | 五个预览。等分、固定与弹性对照、嵌套及 spacer 将尾项推至边缘。 |
| `/grid` · `grid.at` | 四个预览。语义和类声明的三列、等轨分配和视口变化的 1/2/4 列。 |
| `/grid-span` · `grid-span.at` | 四个预览。跨两列和跨三列宽度均计入列间距，后续单元格仍按网格顺序填充。 |
| `/position` · `position.at` | 两个可操作视觉预览和原降级矩阵。浮层保留底层内容，Vue 按 z-index，AV 按原页约定的绘制顺序和定位降级。 |
| `/responsive` · `responsive.at` | 四个预览。窄→中→宽→窄见 1/2/4/1 列和隐藏/显示/显示/隐藏，检查响应式内距和字号。第二个色块沿用户批准的 `hidden md:flex` 源修复。 |
| `/row` · `row.at` | 六个预览。横向顺序、间距梯度、两轴对齐和列中嵌套。 |
| `/sizing` · `sizing.at` | 四个预览。宽高梯度，长条遵守最大宽度，最小尺寸约束可见。 |
| `/spacing` · `spacing.at` | 五个预览。内距梯度、横纵内距、嵌套外距，以及统一 gap 与分轴 gap 的区别。 |
| `/kitchen-sink` · `kitchen-sink.at` | 原组合页可从首区滚到工作区预览。验证原 dialog/drawer/sheet 与 dropdown/popover/select 的触发、内容和关闭，输入焦点、编辑器、图标、链接及原宿主回退在各适用端运行。路由往返后控件状态恢复、旧页面卸载，原窗口和浏览器会话正常关闭。 |

进场诊断沿上一轮版本留存：12 个独立布局页已有局部 Iced 渲染收据，原路由集成待复验；完整原画廊当时在 AV 拒绝语法、严格 A2Vue 缺 AreaChart handler，组合页没有可用的原入口收据。现在从 107 和 114 的正式主线版重放。共享解析、包闭包及受检事件归 AutoLang；两端宿主、布局和控件消费归 AutoUI；原页及资产归 AutoOS。
### 规范增量

| ID | 规范路径 | 增量与验收 |
|---|---|---|
| SD-01 | AutoUI `docs/specs/runtime/ace-route-page-loading.md` | 保留路由包绑定与单 outlet，两端使用同一原前端模块查找规则（AC-01/02/03）。 |
| SD-02 | AutoUI `docs/specs/runtime/ace-package-widgets.md` | 按源投影原生布局与受检内置控件，描述页面会话、编辑器和原前端资源的归属（AC-02/03）。 |
| SD-03 | AutoLang `docs/specs/ace-migration/m2-age.md` | 描述两条 AGE 前门的标签体块、属性与 link 形，明确未知裸标签的诊断与 AAC 同源（AC-02/03）。 |

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。重放原视口变化和原操作，核布局可见状态、适用事件、焦点与卸载。kitchen-sink 用原组合页面核集成效果；其它布局页按各自原 oracle 出结果。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

## 7. 验收标准

| ID | owner_stage | 通过条件 |
|---|---|---|
| AC-01 | work | 每个原项目/页面、原 oracle、适用端、实际前置和资源映射可复查。 |
| AC-02 | work | 所用接口及组件各有正反验证，真实 AV/Iced 的原行为、权限、恢复和关闭通过。 |
| AC-03 | work | A2Vue 的实际生成、构建、交互、失败和恢复按原矩阵通过。 |
| AC-04 | review | 收据与冻结版本对证，逐行核验缺失和临时代码，复审通过。 |
| AC-05 | merge | 可消费版本及两个目标的各自结果落主，各登记下游交接。 |

## 8. 执行步骤

当前 harness 如提供任务面板，开工时登记以下任务 ID。各目标按实际接口推进，阶段复核与最终复审分别覆盖对应结果。

- [x] T-01 固定原项目、依赖与验收矩阵
  - work，AC-01，核 §1 现有源和 §6 原运行入口，输出逐行的源、权限、预期和实际阻断。 2026-11-22 固定 §5 的 13 行、原入口和资源，诊断见 §9。
- [x] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 接实际缺口，使用已有 owner 的实现和定向正反用例。
- [x] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
- [x] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [x] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。 2026-11-22 原 PAC 冷构建与 Chrome 走查完成，具体见 §9。
- [x] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-21 路线收据。AutoUI ui-123/plan-123-dev 至 9d5ea5b3，承接 ad522af8 布局预览、ffbb488c 生产路由宿主及主题修复。核对 plan-artifacts/plan-123/layout-native-theme-matrix.log，12 个原布局页浅深两态的 Iced 尝试均退出 0，且代表像素可区分。原完整画廊在该基线上 AV 报 source.syntax，严格 Vue 在 AreaChart handler 缺席处拒绝（同目录 r2-raw-av-run.log、r2-raw-vue-run.log）。局部主题与路由收据可复用，完整 AV/A2Vue 交付、独立复审与归档待完成；原未勾任务保持待核。

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-22 恢复实施。此前的外部依赖已解除。核实 PLAN-107 的 AutoUI `fb343c48`、AutoLang `dd5f3eff`、AutoOS `680a6ae3` 及 PLAN-114 的 AutoUI `3dbdc756`、AutoLang `0046408c`、AutoOS `734ad776` 均在当前默认分支。AutoLang 专属树快进至 `0046408c5bc1225e1d6f899dbabba297137e3ab9`。AutoOS 专属树从 `734ad776e6ef51ace79507f92bd69cd777b325fe` 重放已授权的 `/responsive` 修复到 `cdb4883812229576b1f21e6124cd900bfad7c630`，range-diff 补丁等价。AutoUI 在 `8dd99503` 重放三个本计划提交至 `c7482d11`，保留主线的条件样式、文件权限、指针与窗口卸载、受检 Vue 工厂以及大组件配置，合用原路由模块发现。AutoDown 固定 `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`，AutoWeb 固定 `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`。各专属树干净，主检出发现的其他改动均在文档或其他工作树目录。

冻结输入见 `D:/autostack/plan-artifacts/plan-123/r3-original/manifest.json`（SHA-256 `D7902742D3DD83F50A1C7DBD01EABA6ACD8D0BE232CC0B6CE7C31C0E4663BF8F`）。AutoOS 92 个跟踪文件、AutoDown 295 个跟踪文件与固定工作树逐文件 SHA-256 一致。AutoUI Aura Schema SHA-256 `2B0FC537B18B942658A0789F05F4567817A3893FBD78351D8AE2E8DD20B50B6E`。复用已安装的 CLI `auto.exe`（SHA-256 `EDDB2CCE50FD8AFE08CD26959C2FE52E090EB833B5C8B9A3F27538B28D947FCF`），正式工作树 `auto-ui-tool` 正在重建。固定 AutoDown Engine 使用 pnpm 9.0.0 的离线锁定依赖与独立准备构建，构建后的 parser、编辑器以及包指纹检查通过。布局原页及组合页仍需用这一候选执行两端运行验证。
2026-11-22 T-01 完成，T-02 开始。冻结的 debug 工具 `auto-ui-tool.exe` SHA-256 `51FE1427B1215B906B3609BC8F5CA499FFD2D29AE388317BA835A8A3E9158601`（准备 7m13s）。严格 `AUTO_UI_ACE_VUE=1 auto build --render vue --gen-only` 在原项目生成 70 条路由、90 文件，退出 0，153.723 s。生成器为 Kitchen Sink 产出具名的语法诊断组件，故生成成功只证明入口和路由表可运行。原 Iced 画廊窗口通过侧栏回调打开 12 个布局页，均挂载且无顶层宿主错误，实际窗口帧为 1800×1125、DPI 倍率 1.25。原首页保留与 107/114 一致的单独诊断。Kitchen Sink 进出两次均命中同一语法拒绝，可返回布局页。首个窗口探针在收齐截图后因探针自己的路由计数越界退出 101（15.798 s），该次不作正常关闭证据。

复用当前 `ace_core` 对原 Kitchen Sink 小节做逐个隔离，原拼写 `divider {}`、`spacer (size: 16) {}`、`label (for: "sample")`、`link (href: "/button")` 和 `nav-item (to: "/button")` 的最小形重现 `source.syntax`，同一探针中现有路由 link、label 文本与 nav-item 其它属性通过。第一项修复在 AutoLang 的 AGE 视图语法槽位，保留裸标签分支和非 AGE 语法。窗口原始输出 `r3-original-av-window.log`，最小重放 `r3-kitchen-parse.log`，Vue 产物及日志均在当前计划材料目录。
### 2026-11-22 Kitchen Sink 原页准入和投影

- AutoLang `92b23bd64` 接通 AGE 视图中的 `divider`、`spacer` 属性/体块及 `for`、`to` 属性，接受 link 的 href 形。裸标签及未知裸标签的拒绝原行为保持。原五节最小拒绝被关闭，正式两端均进入下一阶段。按仓内 AAC 生成器重发历史数据，只改变 `age-package`、`aac` 两个 profile，另六个逐段字节一致，数据 SHA-256 `C7830EE07A0A8656BD738E31C9466F30947750B7BF01C4D4F7A9BA618D88128B`。
- 原画廊 R4 Iced 实窗 12 个布局路由均挂载，Kitchen Sink 过解析后报静态树 31 个文本叶、AV 只观察 30 个。定位至 `text (selectable: true) {}` 不产生文本观察。AutoUI `0edf0dc4` 同步绑定计数、可见性与光标。R5 原路由已能挂载、来回切换且正式关闭，18.321 s（R4）和 20.639 s（R5），日志分别在 `r4-original-av-window.log` 与 `r5-original-av-window.log`；R5 日志 SHA-256 `B2904A5942BB709829D19777C08C3E0D72ED1ADB6DA5C306877940406AE1236D`。R5 的组合页仍是平面观察视图，实际控件闭环待验。
- A2Vue R3 严格冷生成、冻结安装、`vue-tsc && vite build` 通过，原 Chrome 5.815 s 从侧栏进入 12 个布局路由及 AreaChart、FileTree、TreeView，目标均有对应预览，页面错误与 HTTP 失败 0。未知路由具名拒绝，从同页可返回 Grid。Kitchen Sink 在 R3 显示原语法诊断。R4 刚过语法时，Vite 以 hover-card 连字符和下划线别名生成重复声明拒绝构建。AutoUI `8efce552` 令已由 AGE 检查的静态组合视图调用现有 Vue 控件映射，保留带实际处理器的 route 编译。R6 正式冷生成把组合页及组件接到真实产物，70 路由、235 文件，208.100 s，确认 Vue 编译器在链接 scaffold、ImageSurface 字面量比较与 Select 字面禁用值三处拒绝。诊断已在当前候选修复，原包构建与浏览器交互待复验。两次浏览器布局几何探针访问本地临时 server 超时，当前仅计已通过的正式 R3 路由与控件页挂载证据。
- R6 初次 AutoUI lib 测试准备发现两处基线测试 fixture 编译故障，已在 `2429f9a1`、`828f07eb` 按当前构造器修正。重新准备 `--lib --test plan124_routes --test plan980_package_sfc` 退出 0、196.607 s。精确用例执行含进程启动退出的墙钟如下。未执行完整测试组，未计作产品验收通过。

| 当前定向用例 | 观察行为 | 墙钟与结果 |
|---|---|---|
| `plan104_bare_tags`，5 例 | 旧闭集、跟随属性和拒绝的原义，新增注册标签正文与保留字属性在两 AGE 前门均投影 | 5/5，0.141 s；新增单例 0.178 s |
| `empty_text_keeps_the_following_av_observation` | 空可选文本不占观察叶，后续文本仍取真实 AV 值 | 0.224 s，通过 |
| `native_layout_preview_projects_absolute_children_without_changing_flow` | 有 z 层的关闭按钮浮在右上，对照色块保留流内布局 | 0.134 s，通过 |
| `native_layout_hidden_views_preserve_following_av_text` | 隐藏子树占其编译结果，下一可见文本不串位 | 0.220 s，通过 |
| `authored_front_resolves_nested_pages_and_source_paths` | 两宿主共用模块寻址，嵌套路由页保留其真实文件路径 | 0.321 s，通过 |
| `p123_literal_route_uses_registered_vue_widgets` | 别名指向同一已注册控件，动态页保留受检处理器 | 0.379 s，通过；后续链接改为原 Vue Router 控件待复验 |
| `p123_package_sfc_projects_layout_primitives` | 固定/动态网格、间距、中心和 spacer 均生成实际布局元素 | 0.540 s，通过 |
### 2026-11-22 双端控件与资源

- AutoUI `c9168fb1` 经 R7 production debug 工具构建通过（106.018 s），原 Iced 画廊窗口回调打开原对话框、取消，页面会话 2 的 disclosure 从空→true→false。返回布局路由后重开为会话 15、初值清空。绘制探针捕获了实际 modal，中途过早读取的最后截图仍含前一路由的绘制，故重开只计会话/状态证据。探针关闭、退出 0，20.844 s；日志 SHA-256 `5DEAAFEDCE89AD32E53646F35A5C8F75D6F76DADB73279EABCFA982F73A02733`。原项目正式 `auto run --render vm` 实际创建 Iced 窗，按窗口关闭并退出 0，13.737 s。
- AutoUI `b872040d` 完成经核对源码的 RouterLink、ImageSurface fit 与 Select disabled 产物。仅生成后手工运行 Vite 首次发现生产资源阶段未执行，原 `auto build --render vue` 自动携带 `icon.png` 后通过冻结安装、类型检查和 Vite，139.948 s、退出 0，`dist/icon.png` 和原文件 SHA-256 同为 `AE92E43B43B2B8229A8A9E59853357DA2F7BD2A3A017D791963DFA227DCEA7B0`。
- Chrome 在该正式产物看到原组合页 54 个小节。对话框 1.000 s、Drawer 0.886 s、Dropdown 0.531 s、Hover Card 1.843 s、Popover 0.263 s、Select 0.547 s、Sheet 0.407 s 的原操作均通过。真实输入聚焦与回读、AutoDown 编辑器挂载、图标自然宽度和原工作区提示均已经过对应断言。原页的固定定位 Sidebar 示例挡住外层 Grid 链接，路由重开判据未通过。当前候选在页内约束预览 Sidebar，冷构建复验仍在运行。
- 原 Vue 布局几何在同一正式产物上回放，12 页预览数量匹配，Grid 三轨 gap 16 px，GridSpan 双跨宽 296 px = 2×144+8，居中偏差 0 px。Responsive 在 650/850/1200/650 的列 1/2/4/1、显隐 false/true/true/false、字号 16/20/30/16 px，内距 16/32/32/16 px。几何结果已输出，浏览器关闭仍待该回放进程终态确认。
- R8 已接编辑器会话键与关闭释放、同前端目录的 public 资源映射和页内受检链接，工具构建通过（88.179 s）。原 Iced 页面仍能挂载，但新增的合成路由按钮改变了进入复合视图的判定，导致首次 R8 窗口探针找不到对话框触发器。该脚本退出 0 只说明窗口退出成功，控件结果标为未通过。下一修复把路由按钮和实际作者消息按钮区分，用同一原页重新执行。
### 2026-11-22 A2Vue 原画廊验收

AutoUI `f8a4952b` 把原组合页各 Sidebar 示例留在各自的局部视口。R8 用逐文件核对一致的 AutoOS 92 个源与资源文件，从空生成目录执行正式 `AUTO_UI_ACE_VUE=1 auto build --render vue`，原 `auto` 工具复制 public 资产，运行 `vue-tsc`、Vite 成功，退出 0、236.250 s。正式构建日志 `r8-original-vue-product-build.log` SHA-256 `F5643A3DAB8D8D0611DF924911651B3B9626D25FF4F87D88ACD316EFFACCC2E4`。

Chrome 从原侧栏走 Kitchen Sink 的 54 行。对话框取消与确认、Drawer 关闭、菜单选择、悬浮及离开、Popover 关闭、Select 选中、Sheet 退出、输入焦点、编辑器、原图标和无工作区宿主时的说明通过。原 Grid 链接可以点击，离开并重开后输入恢复为空、Select 恢复原占位。异常 0、失败资源 0、浏览器正常关闭，38.287 s。日志 `r8-original-vue-kitchen.log` SHA-256 `FCDD99EA04B99EDFD3968232587B2C29AFA3FFD9049B733800063369889F3E53`。

同一冷构建跑全 12 个布局页，独立观察预览数量和几何。绝对层 z=10，12 个交叉轴/主轴组合逐格匹配，居中偏差 0 px，spacer 把尾项推到右沿，Grid 三轨间距 16 px，GridSpan 双跨 296 px，浮层 z=50，行列顺序、64/128 px 宽度和 8/16/32 px 内距通过。Responsive 650→850→1200→650 四次验证列数、显隐、内距和字号。未知路由显示 `ace.route.no-match`，仍可回到原 Grid。浏览器正常关闭，3.220 s。输出 `r8-original-vue-layout.log`。App.vue、路由、CSS、12 个布局 SFC 与前次结果逐字节相同。

AutoUI R8 定向准备 167.844 s，`plan124_routes` 5/5、39.324 s，`plan980_package_sfc` 20/20、1.206 s。新增 `p123_literal_route_uses_registered_vue_widgets` 以原链接、局部 Sidebar、ImageSurface、Select 和仍带 checked handler 的邻页验证源投影，精确单跑 0.606 s。AV 路径的 R9 工具修正合成路由按钮分类后，在正式窗口回放附件中复现 12 个布局页和组合页的 alert/drawer/dropdown/popover/sheet 触发与关闭，视口变化和路由切换均无宿主错误，退出 0、40.080 s。R10 正在复验原输入、组件链接、原资产及已交图表/树页。T-06 完成，T-02、T-03 的 AV 收口仍在继续。

### 2026-11-22 当前主线候选的 AV/Iced 回放

- R10 原窗口回放退出 0，58.129 s，发现原进度条说明占用 AV 的文本观察，但原生投影未推进光标，末尾工作区预览的提示无法显示。R11 在生成进度条字幕时消费原文本叶，保留后续观察的顺序。
- AutoUI 将本计划的 12 个补丁重放到 `95401c108915d0981aaff13696ca06786e6a722b`，逐补丁 range-diff 等价，候选为 `3fb6dda42e14b0e265b319d69178b5a6a014c7ae`。AutoLang 的语言改动重放至 `ff116ef24`，补丁等价，加 SD-03 后候选 `d60578d8380b0aabe9607cec6b1459f7a903355e`。AutoOS 原页及只读依赖保留在 T-01 的冻结版本。
- R12 以 debug 配置执行 `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool`，49.036 s、退出 0，工具 SHA-256 `2E5022A9DD14CA3B1CB64CEE83FC21468D68244044CEDE1D595691EDC29BDE06`。用原 `pac.at` 执行 `AUTO_UI_ENGINE=ace auto run --render vm` 实际创建 1440×900 Iced 窗口，发送正常关窗事件后解释器返回 `ok=true`，退出 0，5.421 s。正式 CLI 日志 `r12-cli-native-routed.log` SHA-256 `5E92CCB72ACDD90636559CB3C48BEC0E1757E2FB84E260F19EDF695085BE7AFF`。
- 同一生产库在真实 Iced 窗口中加载冻结 `app.at`，经作者侧栏回调打开 13 个目标页和 AreaChart、FileTree、TreeView。Kitchen Sink 的 dialog、drawer、menu、popover、sheet 触发和关闭、Select 选择、输入和多行文本回读、原链接切换通过。外部页 `/` 和 `/button` 保留路由诊断，窗口内可进入目标页。Kitchen Sink 卸载后重开有新会话及原初始状态，三个图标资源引用均可读且与原 SHA-256 一致，编辑器与无 workspace 数据时的说明都在已挂载视图。原响应式宽度 650→850→1200→650 DIP 各观察 1/2/4/1 列和隐藏/显示/显示/隐藏，屏幕倍率 1.25。实窗回放关窗并确认会话为空，错误 0，57.171 s、退出 0。日志 `r12-original-av-window.log` SHA-256 `D80921C3AEA894262B96E27F443679D5D587E2DBAE29DE88A66F0AB84D47D2B6`。独立结果解析器对帧、会话、控件状态及原资产断言全通过。正式窗滚动收据随后补记。
- 在同一生产 Iced 链，用滚轮从 Kitchen Sink 头部实际滚到 sheet 区。另用现有 Iced 滚动操作在目标页主滚动容器（视口 795 DIP，内容 6987.31 DIP）滚至末端，真实窗口帧呈现 `workspace_preview` 和原提示 “Workspace preview needs a live host snapshot.”。经侧栏进入 Grid 并重开 Kitchen Sink，最后关闭，12.519 s、退出 0、错误 0。日志 `r12-scroll-window.log` SHA-256 `7F5B010F11CD14C4484BE58867A131994C05DABE34241564B25E32480F3BE825`，末端帧 `r12-scroll-window/02-kitchen-sink.png` SHA-256 `59EC3089281A7481F126A5D437B71E952AC82900473D7621C54D17ED230B338F`。关闭与会话状态由实际宿主回读。
- AutoUI R12 显式构建 `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --lib --test plan124_routes --test plan980_package_sfc --no-run`，等待其他桌面运行时构建后开始，准备 130.595 s、退出 0。以已准备的 test exe 加 `--exact <完整 ID> --test-threads=1` 独立计时。相较 R6/R8 的新旧用例逐例通过。原始 `r12-exact-tests.json` SHA-256 `4FB9CBE3124DC67E79E50E56EFEA0A8FC3CD194F1F991A5C96FAC68A4887932D`。

| R12 新增用例（精确运行） | 验证行为 | 墙钟 |
|---|---|---:|
| `authored_front_resolves_nested_pages_and_source_paths` | 两宿主共用原前端模块发现与文件回指 | 0.144 s |
| `native_layout_preview_projects_absolute_children_without_changing_flow` | 作者浮层与无 z 对照的流内布局 | 0.118 s |
| `native_layout_hidden_views_preserve_following_av_text` | 隐藏子树保留后续 AV 文本位置 | 0.120 s |
| `empty_text_keeps_the_following_av_observation` | 空文本不占用后续观察槽 | 0.110 s |
| `progress_caption_preserves_following_av_text` | 进度字幕及其后工作区提示各取原 AV 文本 | 0.109 s |
| `p123_literal_route_uses_registered_vue_widgets` | 作者链接、局部侧栏、图片、选择器和受检事件邻页 | 0.406 s |
| `p123_package_sfc_projects_layout_primitives` | 响应式及固定网格、间距、居中与 spacer | 0.179 s |

2026-11-22 stage: work | plan_id: PLAN-123 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-123 | commit: 3fb6dda42e14b0e265b319d69178b5a6a014c7ae | next: review。完成 T-01、T-02、T-03、T-06。复审接 AC-01–04，合入门保留待执行。A2Vue R8 冷构建和浏览器复用原冻结输入，后续计划生产变更仅在 AV、宿主与规范，待独立 reviewer 核版本适用性。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-ui | commit: 3fb6dda42e14b0e265b319d69178b5a6a014c7ae | next: review

2026-11-22 stage: review | review_scope: phase (T-04 AV) | reviewer_identity: Codex 独立上下文 /root/plan123_review | plan_id: PLAN-123 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-123 | branch: plan-123-dev | reviewed_commit: 3fb6dda42e14b0e265b319d69178b5a6a014c7ae | base_commit: 95401c108915d0981aaff13696ca06786e6a722b | next: work。

- 依赖精确提交见 R12。核 92 个 AutoOS 文件在 AV 和 A2Vue 副本与冻结输入一致，295 个 AutoDown 文件与冻结输入一致，AC-01 通过。复审认可 R12 的原入口、13 页挂载、响应式四档、原工作区提示滚动、GridSpan、控件状态、失败恢复和关窗证据。独立 Playwright 验原 Vue 的真实链接、提示滚至视口、输入焦点和卸载重开，页面异常及 HTTP 失败均为 0，关闭后退出 0，73.812 s。AC-03 通过，沿用原冻结产物。T-07 在当前候选有相同 AV 缺口，待修后终审。
- R123-AV-01 原组合页 Select 初始和重开时应显示 `Choose an option`，当前实窗为空。R123-AV-02 普通 Image 能加载原文件，ImageSurface 收到本地路径但仅消费媒体票据或显式 HTTP 来源。需分别验实际图片。R123-AV-03 `/sizing` 的 `w-4 min-w-20` 实窗仅 16 DIP，原判据 ≥80 DIP。R123-AV-04 受检图片的本地来源未通过 public 目录校验时可能落入通用本地读取，需要正反例对证。R123-AV-05 `/position` 的负边距在 Iced 三个色块上丢失，须观察相交且按源序绘制。R123-AV-06 `/center` 的双轴组合在实窗偏左，须验证容器内两轴中心。R123-AV-07 输入实窗焦点证据缺失，现有回放仅直接触发消息。AC-02、AC-04 保留未通过。
- SD-01/02/03 原文件 SHA-256 分别为 `406A3D9A6F264CAE5CE651CEA5D1692522CBE99B76A364E1925E4825F17B3A98`、`E1999DFFFE9B3F1D6AFFA059DF6803CF3ECD25F4D0FE70FD0A7338BCED826CBF`、`DC517F7DC2B7AD445FFCBC0201459EFAD5288D1FC0EBC4AE3F30DB8C5A5CEC5B`。SD-02 须随修复核实 public 约束和控件呈现。七个 AutoUI 新增用例 0.109–0.406 s，AutoLang 新增 0.178 s，准备路径和运行拆分。合入推荐冻结语言组为 `ace-source ace-syntax ace-av ace-aac-syntax`，原原生 AAC 驱动须显式准备。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-123/auto-ui | commit: 3fb6dda42e14b0e265b319d69178b5a6a014c7ae | next: work

### 2026-11-22 AV 复审修复与重放

- AutoUI `887b50c43c50e207a5e743a2a0ef1bcd0efc182b` 接通原 Select placeholder、项目 public 栅格媒体票据与资源释放、固定尺寸下限、负边距叠放和双轴居中。`e949d674` 将所有作者图标名送入已有 Lucide 字形入口。`a62379feb28e427b76218ea1bdb24f5a8fbdc8bd` 额外核对 canonical public 根的作者位置，拒绝将父目录或兄弟目录当作 public。定向用例以无关的临时文件验证绝对路径、父级跳转、缺失、无 public、图标文件名及原 glyph；原图标名的投影不变。
- AutoLang 主线新增 PLAN-1050 的 AAC MIR、独立 AAC 测试和 ACE workspace 的锁文件记录。计划的语言和规范补丁重放完全等价，`cc7478479 → f93ea302b`，`d60578d83 → 4e7ce3480be2366241b981e3d2dd11c9b8a0e454`，基线 `0abc54a593ce26daa3183c975f48475d0dcd9237`。AutoOS `cdb4883812229576b1f21e6124cd900bfad7c630`、AutoDown、AutoWeb 和原 92 文件清单保持原冻结身份。后续变动只触及 AutoUI 的 AV/Iced 路径和规范，Vue 继续复用 R8 原生产构建和独立 Playwright 收据。
- R14 的 13 个目标页及 3 个配套页实窗回放 58.414 s、退出 0、会话关闭。原响应式四档断言重复通过，首次和重开组合页会话独立，控件变更和回退均通过。当前普通 Image/Avatar 原图还保留两个 public 路径，ImageSurface 使用媒体 URI。日志 SHA-256 `45B2E8FAFDF005CEEF93745F745A785969FD5AF7706F3524D207767CA8BE8F0B`，检查输出 SHA-256 `1B2BF1C7F21508B776FE3386C7E9AB9B9221359E32E1F5AAD23904539F415597`。
- R15 将两仓最终候选编为 debug 工具，62.096 s、退出 0、SHA-256 `DFA4C2E9C6A7841DE8195871EE110ABE5E183066D2AE501525FF796961C044E3`。原 PAC 执行 `auto run --render vm` 创建 1440×900 Iced 窗口，普通系统关窗退出 0，5.682 s。日志 SHA-256 `D48F1CD73D06DED6561C2E44C4B94E9A0506D341A344F8AFAE0F6854D74B4A76`。
- R15 实窗沿原侧栏与控件定位重放修复场景，24.684 s、退出 0。R123-AV-01 的原 Select 提示、原选择值与重开原样一致；AV-02 的两处 raster 像素都与同一冻结 icon 对应；AV-03 紫色填充 96 物理像素、含边框的原盒 100 物理像素（80 DIP）；AV-05 总着色宽 216 像素，两处相交色带各 36 像素；AV-06 居中横轴误差 0、纵轴误差 4 物理像素。AV-07 由实际 Iced 控件接收鼠标点击与键入事件，未聚焦字符无消息，点击原 `sample` 输入后收到 `f→fo→foc→focu→focus`，原页面读回最终值，退出再挂载时清空。焦点回放用 `iced_test::Simulator` 对全原挂载视图注入 Iced 事件，窗口截图单独捕获。日志 SHA-256 `3BDBFABEBA7E34AF61AE692853086F29DE6EA19162A13B9C72E57113CA23E185`，像素检查 SHA-256 `C28EDB66C9A0F9770AF0EAF64B338D49ED03EE7371D46F5DEB48C27FBF3E30AC`。
- R14 定向 AutoUI 测试准备 114.156 s，R15 `--lib --no-run` 准备 67.660 s，debug/test。7 个新增 AV 例最终逐例运行，墙钟 0.169/0.125/0.125/0.140/0.206/0.145/0.124 s，依次对应模块发现、绝对定位、隐式文本光标、空文本、进度字幕、Select 原提示重开、public 资产与图标反例，均退出 0。新增 Vue 路由和布局用例沿 R12 的 0.406/0.179 s。结果 `r15-exact-tests.json` SHA-256 `2BAE1C8F71EFB4D49D9937CC43E25E02EF38D44062547E7B980064EF14188C30`。

2026-11-22 stage: work | plan_id: PLAN-123 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-ui | commit: a62379feb28e427b76218ea1bdb24f5a8fbdc8bd | next: review。R123-AV-01–07 关闭候选，与独立复审核对。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-ui | commit: a62379feb28e427b76218ea1bdb24f5a8fbdc8bd | next: review

2026-11-22 stage: review | review_scope: phase AV (T-04), final (T-07) | reviewer_identity: Codex 独立上下文 /root/plan123_review | plan_id: PLAN-123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-123 | branch: plan-123-dev | reviewed_commit: a62379feb28e427b76218ea1bdb24f5a8fbdc8bd | base_commit: 95401c108915d0981aaff13696ca06786e6a722b | next: merge。

- 冻结依赖 AutoLang `4e7ce3480be2366241b981e3d2dd11c9b8a0e454`（基线 `0abc54a593ce26daa3183c975f48475d0dcd9237`）、AutoOS `cdb4883812229576b1f21e6124cd900bfad7c630`（基线 `734ad776e6ef51ace79507f92bd69cd777b325fe`）、AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`、AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`。五树终审时干净，三个发布仓差量检查通过。
- AC-01 通过。原内容清单和两个产品输入副本逐文件比对，AutoOS 各 92 个、AutoDown 295 个均与冻结版本一致。
- AC-02 通过，R123-AV-01–07 关闭。独立检查 R14 的完整回放和 R15 的修复帧、设备输入、正常退出，复跑图像判据 0.920 s。核已安装 state 释放媒体票据、并发消费者的 registry 引用、Windows 存量路径与未知字形反例。9 个新增 AutoUI 例及 1 个 AutoLang 例均低于 1 s，扩展用例未在运行体触发构建。
- AC-03 通过。核冻结的正式 R8 Vue 构建、12 布局路由尺寸与 Kitchen Sink 54 小节，独立浏览器重放原链接、焦点、工作区滚动及卸载重开 73.812 s。复用浏览器产物与输入。原 `/` 和 `/button` 的具名诊断在 13 个目标页之外，可原侧栏恢复。
- AC-04 通过。冻结 SD-01 `406A3D9A6F264CAE5CE651CEA5D1692522CBE99B76A364E1925E4825F17B3A98`、SD-02 `BFEF4DE28A838DB5D1D67820E6E4DD03DE874D966A36B2DAD9388B3711581820`、SD-03 `DC517F7DC2B7AD445FFCBC0201459EFAD5288D1FC0EBC4AE3F30DB8C5A5CEC5B`。复审未启用重复的构建或全组测试。
- AC-05 的冻结合入门为 AutoLang `ace-source ace-syntax ace-av ace-aac-syntax`，另按 Windows AAC 语法消费者独立准备并绑定原生驱动。`ace-source` 包含源语法 conformance 等既有集成范围。AutoUI 用受检路由与 package SFC 的受影响集成及原两端收据；AutoOS 用原资产和唯一类差量核对。等待合入门后快进、确认规范、登记 AV 供 W1 和 A2Vue 供 Vue/CLI，归档及清理。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-ui | commit: a62379feb28e427b76218ea1bdb24f5a8fbdc8bd | next: merge

2026-11-22 stage: merge | plan_id: PLAN-123 | plan_revision: r1 | checkpoint: preflight | outcome: in_progress。

- AutoUI 专属树 29 个发布路径，AutoLang 4 个，AutoOS 1 个。三仓默认分支是冻结候选祖先。主检出另有 16、25、22 个变动路径，与发布路径交集为零。
- 使用已准备的 `plan124_routes` 与 `plan980_package_sfc` test 可执行文件及源绑定 AV provider，`--test-threads=1`。5/5，3.672 s（SHA-256 `18447C8A1972B1866D64A6FF25352762155EB1093852258EB0B9978A6BF30D4C`）；20/20，1.135 s（SHA-256 `6676E6720E2C164E290AE1F4124B6B32B993EC0E2A3E43EF9D3708EFBDEE0280`）。这些集成二进制取 R14，R15 的 public 路径边界由 R15 的单例与原窗重放覆盖。
- AutoOS `pwsh -NoProfile -File scripts/check-app-submodules.ps1` 在默认检出报告 24 项已有未初始化组合模块，27 个 manifest 条目、缺失 `pac.at` 为 0，退出 1、2.159 s（SHA-256 `39B34E9149A7F0247357B458E70F3CE76D32C94D08C36D7CB1E07A621BD77A2F`）。本次 AutoOS 差量仅为用户批准的 `widgets-gallery/src/front/pages/responsive.at` 一行，gitlinks、子模块配置和 manifest 零差量。原独立画廊 PAC 的 92 个原文件参与两端正式验证。
- AutoLang 按正式固定组 driver 的 `--select-only` 检查 `ace-source ace-syntax ace-av ace-aac-syntax`，输出 4 个 Windows nextest 调用。12:00（UTC+8）发现 `ui-108` 编译桌面工具（PID 32004）、`lang-1050` 准备 ABI worker（PID 41508）、`lang-1115` 执行 nextest（PID 82448）。等待占用释放后开始统测准备。

2026-11-22 stage: merge | plan_id: PLAN-123 | plan_revision: r1 | checkpoint: grammar identity | outcome: needs_fix | finding: R123-LANG-01 | next: work。全组准备尚未启动。对 R15 的实际 `ace_core` 执行 15 面文法身份读回，并重放既有 `plan981_grammar_scope.rs` 四例，2/4、退出 101、0.603 s。唯一两条冻结断言失败在本计划变更的 AgeComponent 和 AgePackage，12 个非 AGE 邻居与 AgeH0 保持。定向更新这两面的实测身份并复审差量，AV 阶段证据继续有效。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 5 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-123/auto-lang | commit: 4e7ce3480be2366241b981e3d2dd11c9b8a0e454 | next: work

2026-11-22 stage: work | plan_id: PLAN-123 | plan_revision: r1 | outcome: pass | finding: R123-LANG-01 | worktree: D:/autostack/.wt/ui-123/auto-lang | commit: bf6ac4cfdaf1f1c5c9d9e2c74e370aafd4731c39 | next: review。

- 测试差量仅 `plan981_grammar_scope.rs` 两个旧冻结值及原因说明。固定 AgeComponent `b9d1d2d892cf70a63e90cd4c96f9050b017e6ade7e09a9ab46da6d202e12372f` 和 AgePackage `363fd950c81824d010f776c8b9a877548b59943b95dd70404dcc8a75be8879ff`。实测 15 面结果 SHA-256 `74029399BDEF4F268CBE8C4FCEF48EAF977387DB95983DCE4A58385FD20B9CCC`，13 个邻居继续沿原值。文法源、AAC 生成数据、运行时、规范和原输入字节相同。
- 用 R15 生产 `ace_core` 绑定现有 Rust 测试模块，准备 1.016 s 和 1.267 s。身份冻结 4/4、0.899 s（SHA-256 `E97F0BEF8CEC534A3DD56C20E36ACE15B9937FAABE202CE34B3D78833DE9BCD0`），原裸标签和新表面 5/5、0.575 s（SHA-256 `9BDC13060263F7504A94DDB92281316B77425DDE6D298C48CB33A34AF61E5389`），新增 `p123_gallery_tags_project_blocks_and_reserved_properties` 精确 0.316 s，通过。未加入额外测试用例。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-lang | commit: bf6ac4cfdaf1f1c5c9d9e2c74e370aafd4731c39 | next: review

2026-11-22 stage: review | review_scope: final repair (T-07) | reviewer_identity: Codex 独立上下文 /root/plan123_review | plan_id: PLAN-123 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-123 | branch: plan-123-dev | reviewed_commit: a62379feb28e427b76218ea1bdb24f5a8fbdc8bd | dependency_revisions: AutoLang bf6ac4cfdaf1f1c5c9d9e2c74e370aafd4731c39, AutoOS cdb4883812229576b1f21e6124cd900bfad7c630, AutoDown/AutoWeb 沿 R15 | next: merge。

独立对账新旧冻结值与生产读回 15 面，AgeComponent、AgePackage 两面符合有意扩展，另 13 面不变。原断言结构、邻居隔离和负控无删减。核原失败 2/4 与修后 4/4、邻近 5/5、唯一新增语言用例 0.316 s。差量仅既有测试的预期值和注释，无新增用例。规范三份、生产实现、冻结输入及 AC-01–04 按上轮终审字节身份沿用。AC-05 的四个固定组合入检查仍待运行。

stage: goal | plan_id: PLAN-123 | plan_revision: r1 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-123/auto-lang | commit: bf6ac4cfdaf1f1c5c9d9e2c74e370aafd4731c39 | next: merge

2026-11-22 stage: merge | plan_id: PLAN-123 | plan_revision: r1 | checkpoint: prepared | gate: known-reds-only | outcome: pass。

- 等待整机既有任务约 692 s，随后于 12:11（UTC+8）复查并启动。Windows AAC 原生测试驱动以 `cargo run --locked --offline -j2 -p ace-aac-syntax-conformance --bin prepare-native -- <准备目录>` 显式准备，173.726 s、退出 0，驱动 SHA-256 `F612A687914D5CB0CFE5DADDC81F99E22BFF7D828402F4D8577B496EC1E44191`。设置 `AAC1046_NATIVE_DIR` 并绑定其文法源码与 Rust 工具链。
- `cargo t ace-source ace-syntax ace-av ace-aac-syntax --prepare-only --debug-tools --out <准备目录>` 独立准备四个 nextest 归档和 debug worker，734.653 s，退出 0，manifest SHA-256 `0DAB77A7E43448B82CA134E1A86376F98D8DA77E9B6BE2105886A0CBDC3D8853`。随后用同一组 `--run-prepared <prepared.json> --out <结果目录>`、`NEXTEST_TEST_THREADS=2` 执行，无新一轮构建。准备和执行 source_state 同为 `de352d5a56ee6120debc3b7d596d1377ad11854505f9c141cb428121180d0c7c`（终审 AutoLang `bf6ac4cf`），组配置 SHA-256 `78a5b6d2f9c0625abeba565f3146e7119fee83fc07628ba9565b160b51a99c9a`。总墙钟 275.337 s，原始退出 1、原始 `gate_passed=false`，25 项已知失败原样记录。结果 SHA-256 `A7AC7BE073E531462A4EBF76D08D5B1D6C5B8D4F6B181B625D6A4F5D37DFC98F`。

| 固定组 | 通过 | 已知失败 | 排除 | 发现墙钟 | 执行墙钟 |
|---|---:|---:|---:|---:|---:|
| `ace-source` | 976 | 5 | 1 | 2.364 s | 36.828 s |
| `ace-syntax` | 159 | 0 | 0 | 2.430 s | 7.943 s |
| `ace-av` | 1563 | 20 | 0 | 4.954 s | 193.362 s |
| `ace-aac-syntax` | 51 | 0 | 0 | 1.488 s | 24.138 s |

五项 `text_record_source` 按 [PLAN-1092](../../../../auto-lang/docs/plans/archive/1092-av-parity-attributes-and-metadata.md) 的已知失败逐 ID 匹配。AV 的 20 项与 [PLAN-1050](../../../../auto-lang/docs/plans/archive/1050-aac-shared-typed-mir-lowering.md) 最终固定组原结果逐 ID 比对相同，包括 `av_method_modes` 1、`av_method_owned` 4、`av_owned_first_case` 1、`av_owned_recipe` 1、`av_test_batch` 1、`component_event_sessions_1106` 1、`py_lane_probe_1094` 7、`resource_915` 3 和 `ace-av-core layout` 1。比对读取时基线原结果 SHA-256 `5C91EECC750F316C529D24E4F380ABFBE2250E0266A90F2507C98AC717C86489`。当前新增失败 0，两个更新后的身份测试以及邻居隔离和新增语法测试均通过。新增语法单例 nextest 测试体 0.077 s，外层精确计时 0.316 s。

AutoLang 默认在回归期间更新至 `5c968b701e439523051277908914a6afd3e9de28`，新增两提交仅归档 PLAN-1050 文档。复查 rebase 的三条记录完全等价（`f93ea302b→2989e6780`、`4e7ce3480→025fa4229`、`bf6ac4cfd→3e001de00`），计划文档之外逐字节无差、SD-03 哈希不变，允许复用本次冻结执行。交付候选分别为 AutoLang `3e001de0073338d78aca05d401461039d820553e`、AutoUI `a62379feb28e427b76218ea1bdb24f5a8fbdc8bd`、AutoOS `cdb4883812229576b1f21e6124cd900bfad7c630`。三仓 `wt-guard` 通过。

合入前再次核默认，AutoUI 更新至 `60dacc6fe50d6338772e6cefd296fb6c47df2f8a`，仅修改 PLAN-106 文档。专属树 15 个提交逐条 range-diff 相等，`a62379fe→8f1ebe7d60a338acc6dd05c0fbb21d17879f5493`。除计划文档外完整树无差，沿用既有原项目、两端执行和规范证据。再次通过 AutoUI 工作树守卫。

2026-11-22 stage: merge | plan_id: PLAN-123 | plan_revision: r1 | checkpoint: landed | outcome: pass。

三仓依次执行 `git merge --ff-only plan-123-dev`，AutoLang `master → 3e001de0073338d78aca05d401461039d820553e`，AutoUI `master → 8f1ebe7d60a338acc6dd05c0fbb21d17879f5493`，AutoOS `main → cdb4883812229576b1f21e6124cd900bfad7c630`。各默认分支 HEAD 与交付提交相同。29 个 AutoUI、5 个 AutoLang 及 1 个 AutoOS 交付路径无交叠；合入前后，62 个并行未提交路径的文件字节、索引和状态相同。合入日志 SHA-256 `967467500BA94EEC556660999561A550EFE486C6D3433945E1D4DAC928C1B1C8`。

2026-11-22 stage: merge | plan_id: PLAN-123 | plan_revision: r1 | checkpoint: canonical_verified | outcome: pass。

在实际合入主检出逐一核准 SD-01/02/03 源哈希与终审记录相同。AutoOS 主检出的原画廊 92 个跟踪文件和只读 AutoDown 的 295 个文件均与冻结输入清单逐文件相同。主检出执行 `cargo tree --locked --offline -p auto-ui-tools --features ace-vue --depth 1`，退出 0、0.737 s，日志 SHA-256 `2070C6FDCE1176BDB43CC0E21C4A30DC3D6724192EA23BF7D6F22215600B6553`。

- W1 接收本计划 §5 原画廊 13 行的 Windows AV/Iced 收据，生产入口使用原 PAC；运行验证和各原行预期见 R14/R15。目标 13/13，含经授权修复的 `/responsive` 窄/中/宽/窄回放。
- Vue/CLI 接收同一冻结原项目的 A2Vue 13/13 行，含正式 TS/Vite 构建、实际浏览器与原资源。现行交付表登记以上两项。T-05 完成，T-08 待档案入库和清理回读。

## 10. 待澄清事项

107、114 的正式提交和消费接口已核定。用户已授权 `/responsive` 原页修复及重新冻结。当前无待用户决定事项。
