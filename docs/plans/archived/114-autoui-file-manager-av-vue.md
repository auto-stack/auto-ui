---
plan_id: PLAN-114
status: archived
completion_kind: delivered
feature_name: U04 FileManager 双端交付
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 2
current_step: 7
total_steps: 8
depends_on: [auto-lang/PLAN-980, auto-lang/PLAN-981, auto-lang/PLAN-1071, auto-lang/PLAN-1072, auto-lang/PLAN-976, auto-lang/PLAN-978, PLAN-085]
supersedes_spec_components: [docs/specs/runtime/ace-package-widgets.md, docs/specs/ui/collection-view.md]
new_spec_components: []
touched_goals: [GOAL-002]
---

# PLAN-114 U04 FileManager 双端交付

## 0. 变更摘要

2026-11-20 用户批准按 demo 和共用接口拆分 AutoUI 106–108。从 [106](../106-autoui-u04-service-and-data-apps-av-vue.md) 的 U04 原矩阵承接本页所列行。 [原分母与交付映射](../README.md)。

2026-11-20 r2。用户授权 PLAN-114 承担三条原用例需要的共享接面。原来源、AV/Iced 与 A2Vue 行为及 AC-01 至 AC-05 沿用。r1 探针保留为失败基线。

## 1. 目标

按原冻结输入验收本计划所列原项目或 Widgets。关联 Widgets 为 [filetree](../../../../auto-os/widgets-gallery/src/front/pages/filetree.at)、[treeview](../../../../auto-os/widgets-gallery/src/front/pages/treeview.at)。

| 原项目 | 原验收重点 |
|---|---|
| [027-file-manager](../../../examples/ui/027-file-manager/pac.at) | 真实文件树导航、原操作、权限拒绝和视图更新 |

## 2. 架构与跨仓职责

PLAN-114 在 AutoLang 承担三个原用例需要的 AGE 包闭包、节点集合和类型化事件，以及 027 所用受检文件操作的最小生产接面。在 AutoUI 承担原 027、AV/Iced 和 A2Vue 的消费与必要宿主接线。在 AutoOS 承担两个原画廊页及其所用树组件所需的适配。保留 085、109、125 已交组件行为与实现各自归属。相关计划正在写同一文件时先核对实际改动并交换交付顺序，各共享文件只由一方写入。

## 3. 技术与实现入口

027 入口为 `src/front/app.at`、`src/back/api.at` 中的 `fs_list/fs_home/fs_drives`，树组件使用原画廊页与其组件包。AV 走受检 AGE 和 `ace_node_desktop` 生产宿主。A2Vue 走 `ace_package_sfc`、Vue/TS 与 `auto-ui-tools`，用真实浏览器验收。使用 `D:/autostack/.wt/ui-114/{auto-ui,auto-lang,auto-os}`，每仓单一 `plan-114-dev`，AutoOS 在现有同组检出上建立开发分支。AutoDown、AutoWeb 是固定版本的只读依赖。

## 4. 来源与前置

用户本轮批准拆分计划。规划基面 auto-ui master 448ed614f388c1ee16c68d2ac7cd0da46c73d4bd。原合同、正负例与 Widgets 归属依据 [1071 W1 冻结](../../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md)、[980 基础](../../../../auto-lang/docs/plans/archive/980-ace-m2-autoui.md)、[981 基础](../../../../auto-lang/docs/plans/archive/981-ace-m2-age.md) 与 [示例归属](../../specs/ui/examples-ownership.md)。Vue 接 [1072 正式交付](../../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md)，按原 SDK、Schema 和源身份复用。

109 的改写按实需对账；报表同时列出选择的 027 源基线。文件 Provider 储备不阻独立树页。 前置按实际能力、原项目和目标逐项记录。共享写面正在由其他计划维护时，消费交付版本并在本计划复验。

2026-11-20 用户以 `/goal 完成计划114` 授权执行、独立复核、合同内修复和合并归档。执行基线为 AutoUI `9bdbbcf859e5185335014425af5e74f5512e8c29`，已包含 PLAN-109 `a6b166278dbdf59f5e7a3ac65955e20d579fa2f7`，选择迁移后的 027。AutoLang 固定 `68fd4c59222c77a5de0963ff5827f1353abd7b4f`。两仓均为 `D:/autostack/.wt/ui-114/` 下的 `plan-114-dev`。同组 AutoOS 原画廊依赖固定 `efcfced51b58a1174f0b4fd3c0288c589495f70e`，AutoWeb 固定 `26d085f7e54ea20c6997edf2dc23eef0cb57faf6`，AutoDown 固定 `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`，三者均为分离 HEAD。

2026-11-20 用户以「OK，授权」批准上述 r2 承接范围。开工复核 PLAN-123 正在改 AutoUI 的 `ace_node_desktop/widgets.rs`、`ace_package_sfc/widgets.rs`、`vue.rs` 和相邻回归，先从独立的 AutoLang 源前门切片开始。PLAN-105 的已交语法在基线内，其未交应用工作仍由 105 持有。AutoOS `os-064` 的提交不改变这两个原页与树组件闭包。r1 的工作收据和基线保持可复跑，r2 产物各自绑定新版本。

## 5. 实施与交付边界

T-02 先做无文件授权前置的树组件。先用 r1 的三个独立反例核实 AGE 语法、项目组件解析和运行状态的边界。按原页逐段接通受检节点集合、组件实例及事件，在 AV 与 A2Vue 运行同一输入。判定语法形态可接受后继续核对运行期资源和状态，保持当前 010 FAQ 静态视图数据的行为。

027 在现有 PLAN-973 文件与路径原语上核对实际调用能力。实现前用原 `NavTo` 与服务端读盘入口做贯通探针，确定类型化请求、目录隔离和权限归属。同一受检文件 API 为读取和原操作建立明确的桌面或服务端入口。Vue 读盘沿原后端，按原权限验收可用操作。任何请求在文件访问前校验活动会话、操作权限与授权根，写操作同时校验源和目标。失败时列表、当前路径和已选条目保持，合法请求重试可恢复。释放会话后旧事件与返回值不可改变当前状态。保留 109 的稳定文件身份。AV 实窗全行通过后独立复审并可交 W1。A2Vue 完成冷生成、构建和真实浏览器回放后终审。

### 规范增量

| ID | 操作 | 目标 | 交付规则及验收 |
|---|---|---|---|
| SD-01 | 修改 | AutoLang `docs/specs/ace-migration/m2-autoui.md` | 写入实际支持的 AGE 包、树节点集合、事件与文件调用边界及诊断，绑定 AC-02/03。 |
| SD-02 | 修改 | AutoUI `docs/specs/runtime/ace-package-widgets.md` | 写入实际支持的树组件、双端事件与实例持有和撤销语义，绑定 AC-02/03。 |
| SD-03 | 修改 | AutoUI `docs/specs/ui/collection-view.md` | 写入 027 真实文件接入和授权观察边界，在既有 CollectionView 权责内验证 AC-02/03。 |

规范差量随实际实现冻结，合并时沉淀验证通过的范围。

- SD-01 记录原画廊两个受检页使用的导入组件和嵌套树记录、属性与类型化事件。路由按原页面和实际组件依赖分别检查，无法装载的页面保留具名诊断。027 的受检宿主文件调用按活动实例转交；目标侧继续执行各自的文件和权限校验。
- SD-02 记录原 FileTree 的逐枝展开、badge、选中以及重开恢复与旧回调失效。原 TreeView 分别持有受控展开和选中，派发稳定节点 ID。原生受检布局把全高根接到有限窗口视口，内部滚动容器和网格采用原样式。Vue 按路由挂载独立实例。Windows 受检独立窗口使用原生消息时间和系统双击间隔，覆盖层处理按下后只把未捕获事件的时标按序交给基础视图。已入队的双击跨选择刷新时须仍命中同一有键控件与检查过的动作。
- SD-03 记录 027 的文件列表、稳定文件身份和状态发布由应用持有。原生窗口和 A2Vue 服务端在实际读写前检查会话和授权根，写操作同时检查许可及源、目标。拒绝维持旧路径和条目，释放会话使迟到请求失效。CollectionView 继续消费已发布的结果和身份。第 98 轮正式实窗核对 Windows 受检原生的 `file:open`。规范记录打开前的普通文件与授权校验、宿主回复和 A2Vue 的桌面使用提示。合流候选的受影响路径仍需复验。

### T-01 冻结来源与判据

三行保留 Windows 上的 AV/Iced 实窗及 A2Vue 冷生成、构建、真实浏览器验收。Linux 继续按原平台条件记录独立验证结果，PLAN-109 的 Windows 既有 VM/Vue 结果用于 CollectionView 行为基线。源文件 SHA256 和工具运行参数收录在 `D:/autostack/plan-artifacts/plan-114/`。

| 入口 | 源闭包与已有运行模式 | 独立判据 |
|---|---|---|
| `027-file-manager/pac.at` → `src/front/app.at` → `src/back/api.at` | 原生 VM 直接消费 `fs/file/env/json/image/storage` 与原 UI 宿主。既有 Vue 的真实读盘经 VM HTTP 或画廊代理走 `fs_home/fs_drives/fs_list`；保留 Vue 写操作权限。源依赖 `stylekit`、`themes`、`collection_view` 和同项目组件。 | 隔离目录固定真实条目并单独读回。地址栏进子目录，排序与换视图保留文件身份，前进/后退及原文件操作的成功与拒绝以磁盘和视图对证。撤权或拒绝时保持旧路径和条目，合法请求随后恢复。关闭观察会话及所持资源释放。 |
| `widgets-gallery/pac.at` → `/filetree` → `src/front/pages/filetree.at` | 画廊既有 VM/Vue，消费 `components/package.at` 的 FileTree、TreeIcon、tree_util 和页内固定节点，无磁盘探测。 | 首帧展开 `docs` 和 `docs/plans`，`docs/plans/archive` 的叶起初隐藏。展开后显示归档叶和 badge，收起后隐藏；选中作用于当前节点，关页重开恢复默认展开。 |
| `widgets-gallery/pac.at` → `/treeview` → `src/front/pages/treeview.at` | 原画廊组件以受控 `expanded/selected`、TreeIcon、tree_util 和页内固定节点运行。PLAN-085 的 `packages/tree-view` 正式合同用于核对重叠语义，原画廊页走其 `official` 组件包。 | 首帧展开数 2。分别触发选择与展开，核回传节点 ID 和独立的计数变化。展开全部后见 `bottom.txt`，收起全部后隐藏，关页重开恢复默认计数与空选择。 |

## 6. 测试设计

| 项 | 原义务与判据 |
|---|---|
| V-01 来源 | 保原 pac.at、项目/Widgets、原入口和独立预期。每条记录源、资产、SDK、Schema、平台与依赖。 |
| V-02 能力与组件 | 对实际修改选相应既有职责组和两个目标的消费回归。保原诊断、权限、关闭、失败恢复和生命周期。 |
| V-03 AV/Iced | 通过原项目实际启动 Iced 并观察如下结果。真实文件树导航及原操作前后数据一致，权限拒绝保持旧状态且能恢复，关闭释放资源。各端核原树组件交互。 |
| V-04 A2Vue | 同样的原项目和输入，经已交 A2Vue 生成、实际构建和真实浏览器观察适用行为、错误与关闭。引用证据逐行核 SDK、入口、资源和版本。 |
| V-05 交付 | 对所交目标记录实际命令、生成/构建/运行终态、独立预期、日志摘要及落主提交。保原平台条件、失败、资源与恢复。 |

| 补充判据 | 入口与预期 |
|---|---|
| V-06 树形数据与组件，AC-02/03 | AGE 包解析与双端消费者。原 611 文档树的节点、子树和 badge 从原页读取，嵌套节点身份完整；缺失组件、错事件载荷具名拒绝。两实例和关页重开观察展开、选中及迟到事件归属。 |
| V-07 受检文件域，AC-02/03 | 原 027 的 `fs_list/fs_home/fs_drives` 与原操作入口。隔离目录的合法导航、复制/重命名和删除以磁盘读回对证；撤权和超出授权根请求在访问前拒绝，保旧列表和选中，再授权后可合法重试，窗口/页面关闭后核资源和迟到请求。保留 §5 原操作和权限全矩阵。 |

T-02 的永久回归优先使用 AutoLang 已登记 `ace-source`、`ace-av`、`ace-a2x` 的实际受影响目标和 AutoUI 的 `ace_node_desktop`、`plan980_package_sfc`、`plan124_ace_vue_routes` 消费入口。文件接面变动按实际修改追加 `ace-stdlib`、`ace-abi` 的相关范围。完成提交冻结后运行合并所需固定组。补充判据的已验项目和待实窗观察的范围见 §9 最新收据。

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
  - work，AC-01。§4–5 固定输入、权限、资源和预期；§9 保存原样入口、独立对照、失败阶段和生产方。
- [x] T-02 接通本计划的具体运行接口
  - work，AC-02/03，按 §2–5 先接原树页，再接 027 的类型化事件、受检文件与 Vue 服务端通路。每段以原用例和 V-06/V-07 定向正反用例回验。
- [x] T-03 实跑 AV/Iced
  - work，AC-02，运行 V-03 真实窗口、操作和关闭，冻结可供 W1 复核的来源。
- [x] T-04 独立复核 AV 阶段
  - review，AC-04，按实际入口与冻结版本核原行为、反例、资源和完整 AV 行。
- [x] T-05 交付 AV 收据
  - merge，AC-05，已审 AV 实现与证据按真实 owner 合入，向 W1 提交实际行。A2Vue 后续使用同组工作树。
- [x] T-06 实跑 A2Vue
  - work，AC-03，运行 V-04 的原项目、生成、冷构建和真实浏览器，逐行核适用既有收据。
- [x] T-07 独立终审
  - review，AC-04，核两端差额、原验收和可复跑输入。共享实现变更后复核受影响的已交结果。
- [ ] T-08 正式交付与归档
  - merge，AC-05，发布 A2Vue 和最终主线收据，沉淀适用规范，归档和校验组清理。

## 9. 复审记录

2026-11-20 stage: new，拆分后的草案，实施及产品验证待执行。

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r1 | checkpoint: T-01 | outcome: pass | authorization: `/goal 完成计划114` | worktree_path: `D:/autostack/.wt/ui-114/auto-ui` | branch: `plan-114-dev` | base_commit/code_commit: `9bdbbcf859e5185335014425af5e74f5512e8c29` | dependency_revisions: §4 | next: T-02。

**工具与源身份。** PLAN-1072 的 AutoLang `9142f94bb` 和 AutoUI `67893861` 均为当前基线祖先，PLAN-105 的共享语法 `384609645` 也已进入所选 AutoLang。Windows x64，MSVC Rust 1.98.0 `88d9e12ae`，ACE host 0.1.0。Auto CLI 版本 `0.1.0+plan-890-verified-r7-t19-116-gc879b3e`，SHA256 `EDDB2CCE50FD8AFE08CD26959C2FE52E090EB833B5C8B9A3F27538B28D947FCF`。同树 debug 构建的 `auto-ui-tool` 和 `ace_node_desktop_window`、ui980 provider、Schema 来源、探针与报告的身份见材料根 `artifacts-sha256.txt`。AGE component grammar ID 为 `3abe2d4a53c607fee4ba474de2ee1cf3d8fc6e79d7f8b04648a632b0d9a30be7`，AGE package grammar ID 为 `42b6eedec484000eeff74717d8159ac514b9905eda18ea95dab9e87c63f672e5`。`source-sha256.txt` SHA256 为 `004AD72452A629F1D683D5E1EA60538F8CA54DDFC9971DADBA33A1B0EED0AFAC`。画廊外壳 `src/front/app.at` SHA256 为 `0578BC212943A3EB9212CBECD58F05808DB17DA4F4CC1D7535941ECEA48217AF`。

**本轮入口结果。**

| 原始输入 | AV/Iced | A2Vue | 对照与归属 |
|---|---|---|---|
| 027 | `ace_node_desktop_window --source` 在声明槽拒绝，exit 4。包入口 `--package … --entry app::main` 在 `app.at:425` 的 `$event` 词法拒绝，exit 4，四个 CollectionView 业务回调均使用此形态。 | `AUTO_UI_ACE_VUE=1 AUTO_UI_TOOL_BIN=<本树工具> auto build --render vue --gen-only` exit 1，`ace.vue-lane-unsupported` 含 `.BootTick/.NavTo/.CommitNew/.CommitRename/.CtxPaste/.ExecuteDelete`。直接包 SFC exit 4，拒于同一 `$event`。 | 在材料副本中仅用字面量替换 `$event` 后，两端包入口越过词法拒绝，继续报入口结构错误。`src/back/api.at:21` 的 `pub type … =` 独立经同一 AGE package 前门也被拒。共享投影归 AutoLang AGE，文件能力接入归现有文件/受检宿主和 A2TS 生产方，027 调用与回归归 PLAN-114。 |
| 原 FileTree、TreeView | 两个原页的单源入口均为 `source.age.slot: expected model`。以原页及所用树模块作包输入，均 exit 4，`source.syntax: expected fn, found widget`。 | 原画廊以 `--render vue --gen-only` 严格生成，exit 1，在外壳声明行 8 拒绝。用 `generate_package_sfc` 直接投原页及树依赖，两个页面都在独立包入口复现同一结构拒绝。 | 已知可解析的 Counter 同源标量对照通过。分别仅加入画廊 `use { package: … }`、隐式嵌套节点记录或 `expanded List`，三种最小对照各自失败；仅移除原页包声明仍失败。共享 AGE 与组件包适配由现有前门和 AutoOS 原组件/画廊 owner 接线，独立于文件能力。 |

`stdlib/ace/sys/{file,path}.at` 已有 PLAN-973 的 `dir_names/canonical/create_dir/rename` 等能力。该交付与 PLAN-1114 的 AV/RS I/O 草案都没有 027 受检处理器调度或浏览器文件授权的消费收据。PLAN-980 D-④-4 将具体签名、效应与权限留给 U04 的真实用例，D-④-5 和当前 `m2-autoui.md` 将 A2TS 外部调用边界留给后继目标域。已核对的旧 VM/Vue PLAN-109 收据覆盖 CollectionView 选择、排序、菜单与真实子目录，适用于既有模式基线。

| 本轮验证范围 | 实际结果 | 准备 | 执行与清理 |
|---|---|---:|---:|
| 本树 AV 装载原样输入与真窗正控 | 3 个原输入、两种入口共 6 次具名拒绝；Counter 真 Iced 窗 5 帧，6 次派发，两实例为 2:1，旧会话关闭且丢弃 1 条迟到事件，重开进入新会话，0 宿主错误。 | AV debug 121.874 s，ui980 0.815 s | 源/包入口共 0.718 s，正控 2.547 s |
| 本树严格 Vue CLI 生成 | 027 与原画廊拒绝，002 Counter 生成 `App.vue` 并导入 `a2x_rt.mjs`。 | Vue debug 247.800 s | 3 次共 20.967 s |
| 包入口对照及 ACE 前门形态 | AV 与 SFC 各 7 次，原样 3、变体 3、正控 1。另一次 ACE 前门探针记录 grammar ID 与原源失败行，标量通过，独立语法变体 3 次具名拒绝。 | SFC 直链 13.114 s；形态探针 0.447 s | 包入口共 2.337 s；形态探针 0.144 s |
| 目标产品交互、权限、恢复及关闭 | 两个目标各 0/3 原样行通过。Vite 构建、浏览器交互和原项目 AV 窗未到达。 | 未到达 | 未执行 |

材料根为 `D:/autostack/plan-artifacts/plan-114/`。证据 `pinned-av-only-probe.json` SHA256 `FAB32890D3A48A76B02D3429E769F43D34B4A12EC2FDFC75CCA870FCC416371A`；`pinned-probe.json` `F659059AAC5934DFEF1647111A8D984B74A4E4669B4D99551527569D4E8BBADC`；`closure-v2-probe.json` `0481218C692B5146EFB6B895A8CC34BA6168A0AA8EDBA87F47D8153C6E69F98E`；`shape-probe.log` `0269F10D106E06E3FAE3FD92D8A6F4C159DFBE42D876E22B7B26FC5A3D700E77`。前置失败另存：显式设置 incremental 被当前 sccache 拒绝（0.136 s，改用 Cargo dev 缺省），直链缺系统导入库（2.058 s，补路径），ui980 冷对照误传编译器路径（0.292 s，改传 `rustc --version`），初版闭包探针传了不存在的画廊 `file_icon.at`（改用实际 `tree_icon.at`），初版形态探针标量控制输入不合法（改用原 Counter）。执行时间含调用内发现及清理，未拆分。无新增永久用例，无统一组执行。公开 CLI 自动生成的四个本地依赖 junction 已逐个核实并仅移除链接，目标包保留。历史工具早探曾刷新原画廊既有的忽略态 `.am/state.at`，正式回放使用同组只读源。

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r1 | outcome: blocked | worktree_path: `D:/autostack/.wt/ui-114/auto-ui` | branch: `plan-114-dev` | base_commit/code_commit: `9bdbbcf859e5185335014425af5e74f5512e8c29` | dependency_revisions: §4 | task_ids: T-01 完成；T-02/T-03/T-06 待正式上游接面 | blockers: §10 B-01/B-02 | next_responsible_role: AutoLang AGE 与数据宿主、A2TS、AutoOS 画廊组件 owner；PLAN-114 消费者回验。

stage: goal | plan_id: PLAN-114 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 9bdbbcf859e5185335014425af5e74f5512e8c29 | next: user

stage: goal | plan_id: PLAN-114 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 9bdbbcf859e5185335014425af5e74f5512e8c29 | next: user

2026-11-20 核对 §10、五个工作树和 AutoUI、AutoLang、AutoOS 主线。基线与 T-01 收据相同，五个工作树干净。B-01 和 B-02 尚无正式接口交付或职责变更，继续等待两项上游接面。复用首轮探针证据，无产品测试重跑。

stage: goal | plan_id: PLAN-114 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 9bdbbcf859e5185335014425af5e74f5512e8c29 | next: user

2026-11-20 第三轮审计。§10 两项上游接面连续三轮缺少正式交付，未收到写面调整授权。五个工作树与各自主检出 HEAD 均匹配 §4 的固定提交，无在途本计划进程。保留工作树和首轮探针，恢复点为 T-02。没有重跑产品测试或变更验收范围。

2026-11-20 stage: new | plan_id: PLAN-114 | plan_revision: r2 | outcome: pass | task_ids: T-02（加入 §2/§5 的有限生产写面）；T-01 原始基线有效 | acceptance_ids: AC-01..05 保持 | authorization: 用户「OK，授权」 | next: work/T-02。

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原树节点静态数据 | outcome: partial | worktree_path: `D:/autostack/.wt/ui-114/auto-lang` | branch: `plan-114-dev` | base_commit: `68fd4c592` | code_commit: `23b3ffe57e021b2abb0490da01d00aad88abbea6` | next: T-02 解析并绑定原组件包与可变集合。

AGE 两个 profile 现在保留原声明省略 `var` 的嵌套节点列表，投影结果保留原始字面量且独立于可变状态。已知合法标量处理器继续通过，写入静态节点列表及不完整子树括号拒绝。复用首轮探针的已绑定基线，原 FileTree 的 116,497 字节和 TreeView 的 2,960 字节数据各自以原样字面量进入同一包前门，r1 两次拒绝，r2 两次通过。完整两个原页、单独组件包声明和有类型的可变列表仍具名拒绝，AV/A2Vue 交互尚未执行。下一切片核对 `ComponentRegistry::load_package` 产生的真实闭包与 `PackageModuleInput` 有限模块身份，然后贯通受检渲染与事件。现有 010 FAQ 数据语义沿用。

| 范围 | 实际结果 | 准备 | 执行 |
|---|---|---:|---:|
| `ace-core` conformance 的 `plan1072_component_style` | 20/20，827 项未选 | `cargo test --locked --offline -j 4 -p ace-core --test conformance --no-run`，初编 27.68 s，重编 1.715 s | 0.088 s |
| 同二进制 `plan981_grammar_scope` | 4/4，843 项未选。首轮 2/4，两个 AGE hash 按实际变更更新；邻居 profile 冻结值通过。 | 同上 | 首轮 0.379 s，终态 0.358 s |
| 新增 `p114_nested_tree_literal_is_preserved_as_read_only_view_data` | 1/1，逐字保留嵌套字面量、拒绝修改及坏输入、标量处理器可编译。 | 同上 | 单例 libtest 0.07 s，低于 10 s |
| 原画廊节点字面量提取对照 | 基线 0/2，候选 2/2。完整原页 0/2。 | 用同树 `ace_core` 链接探针 0.434 s | 0.191 s |

目录 `D:/autostack/plan-artifacts/plan-114/`。`r2-original-literal-baseline.log` SHA256 `2CD1FE2C58E3FEC10998521B659381DFEA42FB20BDE0734E5A8A3D9768C38736`，`r2-shapes.log` `46268E5F9590F1153DFAC7BDF258127B8EC7D0B22BD38BE0BA345DAA412F82AC`，`r2-final-plan1072_component_style.log` `8EDFEB64837422A823FAB1F8223528EF6C0F3E66F5189F0E16E8A3FA3066A123`，`r2-final-plan981_grammar_scope.log` `BDC9383C5F67A053022CDB0BB38CD1A20612F7432EC109BBC484CA56B2F4807C`。测试为 debug/test，发现与清理均计入调用，未独立计时。

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原组件包声明 | outcome: partial | worktree_path: `D:/autostack/.wt/ui-114/auto-lang` | branch: `plan-114-dev` | base_commit: `a3655eb2f` | code_commit: `91ef80ac450414c3aa0a223771d9ca96af0654a6` | next: work/T-02 树组件属性及包绑定。

对齐 AutoLang 主线的 PLAN-1086 所有权交付，`git range-diff` 确认静态节点数据提交 `23b3ffe57` 与重放后的 `f0284bc09` 补丁相同。AGE package 前门现在从原 `use { package: official from "../components" }` 投影包名、目录和源区间，两种 AGE 前门接受原 FileTree 的空消息块。无已组装接面时，`compile_age_package` 对该声明以 `source.age.component-package-unresolved` 拒绝，诊断定位到原声明。探针确认 r1 的包声明反例转为可解析，完整原页仍未通过。只替换 FileTree 的 `default_expanded: ["docs", "docs/plans"]` 的材料副本通过包前门，定位了列表属性语法的下一缺口，原文件及验收输入保留。TreeView 有类型可变列表仍未接通。PLAN-123 的 `ffbb488c` 已在其分支统一路由源闭包，改动覆盖 AutoUI 的 `ace_route.rs`、`ace_node_desktop.rs`、`ace_package_sfc.rs`、`run_vm.rs`、`vue.rs`，进入本计划写面前核对正式基线和合入顺序。

| 范围 | 实际结果 | 准备与执行 |
|---|---|---|
| `ace-core` conformance `plan1072_component_style` | 21/21，827 项未选 | 初编 18.47 s，重编 4.456 s；执行 0.102 s |
| 同二进制 `plan981_grammar_scope` | 4/4，844 项未选。首轮 2/4 红项为此次语法 ID 变更，两项换为实测值后邻居身份检查全通过。 | 首轮 0.359 s，终态 0.362 s |
| 同二进制 `plan981_event_a` / `plan981_event_b` | 10/10、9/9，分别 838/839 项未选 | 0.054 s、0.073 s |
| 新增 `p114_package_imports_preserve_the_authored_name_and_directory` | 1/1，核原包名及目录、空消息块、缺引号拒绝、未绑定拒绝和拒绝源区间 | 独立调用 0.041 s，libtest 0.03 s |
| 原页和最小语法对照 | pkg 声明形态通过，完整两页仍拒，FileTree 列表属性差分控制通过 | 链接 0.789 s，主探针 0.170 s |

编译为 `cargo test --locked --offline -j 4 -p ace-core --test conformance --no-run`，随后按过滤器直接执行 test 二进制。发现、执行和清理共同计时。材料根 `D:/autostack/plan-artifacts/plan-114/`，`r2-package-final-status.txt` SHA256 `A06F00FB594B25AA6FE81EDFE9878AF1513391DF4235DF92FFAD03C2BBFF172B`，`r2-package-shapes.log` `DCCE018ABE23AB653E43EB517B9824BB3393FC8DF497A7A052021F06C5560EBA`，`r2-tree-prop-next.log` `FBDCB424FF6A9DCCB4EF8498D797A3FABDB3736A661619A5EF01E084F64D2089`。本轮未到达原项目 AV 窗口及 Vue 浏览器。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 91ef80ac450414c3aa0a223771d9ca96af0654a6 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 FileTree 列表属性 | outcome: partial | worktree_path: `D:/autostack/.wt/ui-114/auto-lang` | branch: `plan-114-dev` | base_commit: `a3655eb2f` | code_commit: `283d1372925ed9e00fe1104283d0d3b436911ad6` | next: work/T-02 组件包组装、可变集合和受检宿主。

未改动的 AutoOS `pages/filetree.at` 在共享 AGE package 前门转为可解析。列表属性投影为现有 `ExprKind::ArrayLit`，各字符串保留源序、值和 span，空列表保持为空。原对照 `91ef80ac4` 拒绝整页，新候选通过。新增用例核 `default_expanded` 的顺序、转义、空值及混入数字时拒绝。首版把列表交给标量验证而被拒，随后按元素验证，原未绑定组件包的具名拒绝继续保持。AGE package grammar ID 更新为 `3777ce82c98fe3dc19980bc10684859dcdc09a18eb0a1190b7ae35be8a0c447f`，组件面与其他冻结 profile 通过原身份检查。

当前 `PackageStateEngine` 由累计事件重放生成每字段观察函数。027 文件写操作需要沿现有 `ComponentEventSessionProgram` 的单次派发和独立状态根接通包实例，或在受检状态转换后由同世代宿主提交一次操作，保证初始化、读取和重开不会再次执行文件操作。原 FileTree 的组件实现、TreeView 可变集合和原画廊闭包仍未通过。PLAN-123 的路由和主题实现仍在其分支，AutoUI 组件加载写面待对账。两端交互与权限验收未到达。

| 范围 | 实际结果 | 准备与执行 |
|---|---|---|
| `ace-core` conformance `plan1072_component_style` | 22/22，827 项未选。 | 初编 5.738 s，修正后 6.275 s；执行 0.093 s |
| 同二进制 `plan981_grammar_scope` | 4/4，845 项未选。首次 3/4，包面唯一 hash 变化按实测刷新。 | 首轮 0.379 s，终态 0.380 s |
| 同二进制 `plan981_event_a` / `plan981_event_b` | 10/10、9/9，分别 839/840 项未选。 | 0.059 s、0.067 s |
| 新增 `p114_package_view_preserves_expanded_node_ids_in_source_order` | 原型在标量检查拒绝，终态 1/1，核转义、顺序、空值与混合类型拒绝。 | 首轮 0.126 s，终态 0.041 s；单例 libtest 0.03 s |
| 原页和组件源探针 | 1/2 原页通过解析，FileTree、TreeView 组件源仍拒。 | 链接 0.427 s，执行 0.178 s |

编译 `cargo test --locked --offline -j 4 -p ace-core --test conformance --no-run`，test 二进制按上述过滤器执行。发现、执行和清理合计。中间编译因遗漏局部 AST 类型引入失败（9.662 s），已补足。材料根 `D:/autostack/plan-artifacts/plan-114/` 中，`r2-text-list-shapes.log` SHA256 `616B2445B54C6AE87515E98892BF5CEE7E0CCE66B6BC3C1C7E17654CE11CF3C9`；`r2-view-text-list-final-status.txt` `0135747744B5E0F22D70E9586D0D14ADBA927394B1BD8A224682542B61BB4D32`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 283d1372925ed9e00fe1104283d0d3b436911ad6 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 组件包组装及绑定 | outcome: partial | worktrees: `D:/autostack/.wt/ui-114/{auto-ui,auto-lang}` | branch: `plan-114-dev` | commits: AutoLang `b84a0a39cb629bf2e85d1bd3d289b04dbf6cca3f` / AutoUI `edd34fb09b1fe13b5ae12de78b42e13dc35191b8` | next: work/T-02 原树组件和集合检查/双端执行。

AutoUI 的组件注册表记录每个 widget 的源文件。AV 的按文件装载与 A2Vue 的带路径路由生成从声明所在路径复用注册表，核清单名，按已用 widget 和本地 helper 选择原模块。AutoLang 包前门按声明的可见包及模块绑定活动视图中组件标签，记录确切模块、名称和使用位置；保持未安装、名称冲突及缺失组件的具名诊断。受检组件引用暂以明确的运行端拒绝代替空壳生成，需后续接通实际 props、状态与事件。PLAN-123 §10 已将组件包写面交给 PLAN-114，原布局与主题实现仍留在 123 分支。A2Vue 源路径改动与 123 原路由入口在合入时需对账，未移植其在途提交。

原样 `pages/filetree.at` 通过 AV 的 `NodeDesktopBridge::load` 与 A2Vue 的 `generate_routed_front_sfc` 入参回放。A2Vue 采用最小路由壳装载原页以隔离原画廊外壳。两端均由组件包找到真实 `official::filetree`，首诊断从原页解析失败推进到 `module official::filetree: source.syntax: expected fn, found widget`，因此尚缺组件内 `List`、计算及视图循环等受检实现。原画廊壳、原组件行为、FileManager 权限与操作仍待验收。

| 范围 | 结果 | 准备与执行 |
|---|---|---|
| `ace-core` conformance `plan1072_component_style` | 23/23，827 项未选。 | 首轮编译 5.107 s（不可见私有类型失败），修正 14.832 s，终态 7.385 s；执行 0.109 s。 |
| 同二进制 `plan981_grammar_scope` / `plan981_event_b` | 4/4、9/9，分别 846/841 项未选。 | 同上；执行 0.401 s、0.076 s。 |
| AutoUI `plan114_widget_packages` / `plan980_package_sfc` / `plan124_routes` | 2/2、11/11、2/2。旧排浏览器用例未启用其环境门。 | 同一 debug/test 构建首轮约 61 s，终态 32.334 s；SFC 执行 0.129 s，路由 0.568 s。 |
| 新增 `p114_package_view_binds_widget_sites_to_the_declared_module_closure` | 核确切父子模块、源址、缺失及重复拒绝。 | 0.062 s。 |
| 新增 `bound_component_uses_name_the_missing_renderer` / `authored_source_paths_supply_only_the_referenced_package_modules` | 核两端确切模块且明确拒绝未实现渲染、实际清单查找、错误清单名和未使用包模块的隔离。 | 0.059 s、0.099 s。 |
| 原 FileTree 宿主探针 | 两个消费入口都到达原组件文件，具名拒绝于真实所需模块。 | 链接 5.745 s，执行 1.981 s。 |

准备分别调用 `cargo test --locked --offline -j 4 -p ace-core --test conformance --no-run` 与 `cargo test --locked --offline -j 4 -p auto-ui-runtime --features ace-bridge,autodown --test plan114_widget_packages --test plan980_package_sfc --test plan124_routes --no-run`，执行直接调用生成的测试二进制。发现、清理计入调用。材料根 `D:/autostack/plan-artifacts/plan-114/`，`r2-binding-final-core-status.txt` SHA256 `415DFB13751ED752F6B00E3A390042AA055B979C2AF1062B1E0B5DA48F01A29F`，`r2-binding-final-ui-status.txt` `E40A8804E23E66A6E9AC246C9928F6B9AD7F180C20153146B397E0BE6CA32D77`，`r2-filetree-host-final.log` `4B329BEA3D398E24B71290AC5202140D55F82B27A2385378F6DF323F836A954E`。未执行原项目 Iced 交互或浏览器验收。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: edd34fb09b1fe13b5ae12de78b42e13dc35191b8 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 树组件默认参数和循环绑定 | outcome: partial | AutoLang: `35142ba509677d6c3b954c227071d4b92d5ecf06` | AutoUI: `83ec3a51a223eb99eea8cb6e0fedaa879d3bb5b8` | next: T-02 原样插值、列表状态和实际组件渲染。

两种 AGE 声明面可区分 `for item in …` 和 `for index, item in …`，保留嵌套循环的绑定和作者 span。包面将字符串列表、文本和布尔型 widget 参数的字面量默认值留在组件参数中，错误默认值按原表达式 span 具名拒绝。包面的字符串列表表达式由原有视图属性共享到参数默认值、模型初值和赋值。新语法冻结值为 AgeComponent `469e49774627ba8267eef1ea7749747463868914c3c96f7eee17f3028b67cb63`、AgePackage `5204369de125dfaa1f633619891f942e2472e35bd48e8e49fef3a551636bac7b`，冻结邻居通过原断言。

原样回放仍只有 `pages/filetree.at` 通过前门。对照只在材料副本中把三个原文件的 f-string 换成普通文本。该对照的 TreeView 组件前门通过，FileTree 组件与 TreeView 页分别出现 `ftExpanded`、`tvExpanded` 类型 `List` 的具名诊断。保持原画廊工作树和源文件不变。类型核对现已越过列表字面量语法，下一段需接通受检的列表类型及状态、循环变量求值和原样插值。两端实际组件、027、交互权限及浏览器验收待完成。

| 范围 | 结果 | 准备与执行 |
|---|---|---|
| `ace-core` conformance | `plan1072_component_style` 24/24、`plan981_grammar_scope` 4/4、`plan981_event_a` 10/10、`plan981_event_b` 9/9。新增单例已计入前一组。 | `--no-run` 首编 10.143 s，终态 4.489 s；组执行分别为 0.098 s、0.366 s、0.054 s、0.062 s，新增单例 0.037 s。 |
| 原源及诊断副本 | 原样四个输入及三份只替换 f-string 的副本。正控使用 AutoLang Counter。 | 链接 0.646 s，原样及正控 0.219 s，诊断对照 0.095 s。 |
| AutoUI 直接装载 | 静态列表分别验证单绑定保持行顺序、双绑定仅在索引 0 出现条件叶。原 010 FAQ 随 `style_blank_tests` 5/5 回归，同时覆盖 006 和 007 源；未开启真窗。 | 终态 `--no-run` 23.951 s，装载用例 0.164 s，样式组 0.185 s。 |
| AutoUI 包入口 | `plan114_widget_packages` 2/2、`plan980_package_sfc` 11/11。旧浏览器用例环境门未开。 | 同次准备，执行 0.172 s、0.231 s。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。 `r2-widget-syntax-core-status.txt` SHA256 `3B9C6469EDA03ED240127BE8F5DB365033B99C0C120FEA0D590A0A809D3E5D7E`，`r2-widget-syntax-raw.log` `424B3B077DFBD9A640383108CBAE0A3FC7FD58F96C917F41EF79133A0D48F3C9`，`r2-widget-syntax-no-format.log` `1B70FFBEA487E4EF806532E7D07167B16890B9FAB188504DD86374BC9023CA1F`。首次误用含反引号的 002 为正控，终态已换回本仓 Counter。发现与清理计入执行。

AutoUI 以 `cargo test --locked --offline -j 4 -p auto-ui-runtime --features ace-bridge,autodown --lib --test plan114_widget_packages --test plan980_package_sfc --no-run` 单独准备。首次用例位置与测试输入导致的编译、列表项分行和裸标识符构造歧义失败均保存在同材料根，终态测试经直接装载获真实叶值。`r2-widget-syntax-verified-ui-status.txt` SHA256 `8ED0CD5351855E67AE6F5FABB69D1C828A1CD7D9633498CC76C524E0A033C0B5`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 83ec3a51a223eb99eea8cb6e0fedaa879d3bb5b8 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检插值及 AV 类型化派发 | outcome: partial | AutoLang: `6559fb2b155dc131ae53d5d9f0b1b160f38baa11` | AutoUI: `b71200b094e68b9f5dfd599a05ef3badf63926eb` | next: T-02 可变列表、组件实例执行与完整画廊接线。

AGE 包面直接解析原 `f"…"`，复用共享插值分段和表达式投影，保留 UTF-8 源 span，语法错误和未绑定符号仍在受检前门拒绝。原 `text (text: …)` 读取真实叶。包面 A2X 与 AV 各自使用受检表达式事实。AV 观察沿既有 owned MIR、ABC 验证与执行通道使用文本显示指令，既有独立组件面的诊断保持。包面归组按实例恢复各自的多条文本叶。AgePackage grammar ID 为 `e782b2c021b9e0ab57a3d19d5229ab941262900a0408b33834a371210b7d2f08`。其余冻结身份通过。

桌面包桥将宿主字符串 ID 装进已有 `SendArgs`，定向同一入口组件的当前实例。两个实例、辅助组件、错误数字载荷保持原值，以及正确 Unicode ID 只改目标实例均有回归。Vue 生成受检文本和动态 class 绑定，并保留字符串字段初值。用独立小组件实测浏览器，展开计数从 1 到 2，文字与 `visible-1` 到 `visible-2` 的 class 随同一次真实按钮点击更新，选中项 `src & docs` 保持，页面无报错。该浏览器样本用于验证插值和普通事件；原页和 027 仍未走到浏览器交互。

用未修改的四份 AutoOS 树源回放，FileTree 页和 TreeView 组件通过包前门。FileTree 组件 `ftExpanded` 与 TreeView 页 `tvExpanded` 报具体 `List` 类型诊断；原 FileTree 页的 AV 与 A2Vue 路由入口也都指向同一真实 `official::filetree` 字段。需要接入 checked model 的集合所有权、原 `tree_util` 与计算行，同时完善 Vue 的类型化组件事件。当前 AV 包事件沿旧纯状态场景重放，027 文件写入必须走实际一次性执行路径。

| 范围 | 结果 | 准备与执行 |
|---|---|---|
| `ace-core` | 组件样式 25/25、冻结语法 4/4、事件 19/19，既有插值 7/7。 | 与 AV 同次终态准备 18.879 s；各组执行 0.213 s、0.392 s、0.115 s、0.051 s。 |
| `ace-av` | AGE 包 9/9，独立组件 states-only 8/8。 | 执行 0.130 s、0.099 s。 |
| AutoUI | widget 包含新增两个定向例 4/4，包 SFC 11/11，closure SFC 2/2，closure bridge 2/2。 | 终态准备 35.340 s；执行 0.295 s、0.165 s、0.094 s、1.132 s。 |
| 既有浏览器回归 | 条件臂与计数器两个既有用例以 `AUTO_UI_ACE_WEB_E2E=1` 各自复跑，均通过。 | 执行 2.358 s、1.391 s，均已计入包 SFC 测试数。 |
| 原入口探针与插值样本浏览器 | 两端原 FileTree 指向真实集合字段；小样本经本树 SFC 生成、实际 Vue 编译、Chrome 挂载并点击。 | 原页链接 4.448 s，执行 3.081 s；浏览器样本链接 5.110 s，生成 0.211 s，编译、运行与清理 0.867 s。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-format-verified-lang-status.txt` SHA256 `804D61213348EB55FC36206317039FCB74BFCBF753FAACADF12361B6F0F8DA2C`；`r2-format-verified-ui-status.txt` `7EF231A7C2A4BF50BCDC8BE7D5DFA0143584E34201AC9C15962372C76744D024`；`r2-format-browser-verified.log` `EC8D16FC99A136609DB87E5420B16DA1D7DE96C78A10FB24E9CD521CDD590DA8`；`r2-format-filetree-host.log` `8F930392C19AA4A1EF39EB26486CEC6C9B0519095CE782192595C2CFB7378E9A`；`r2-format-raw.log` `D6AF3AB543CF641D7417C8A176F7C8F6F2C076756DEF98115A8DE74FAF5583DF`。各测试先 `cargo test --locked --offline -j 4 … --no-run`，再执行实际二进制过滤组。中间捕获旧 AV lowering 与 A2X 视图拒绝、002 遇非法字符时的上下文 token 越界、Vue 字符串初值为 undefined，以及 Playwright 默认浏览器版本缺失。前三项已修复，浏览器回放使用本机 Chrome 通道。

两个既有浏览器用例的 `r2-format-existing-browser-status.txt` SHA256 `70BB28B0CDA2E089F95B88F624A9317769F506C0B2CD66F3D8068A64C2BA10E9`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: b71200b094e68b9f5dfd599a05ef3badf63926eb | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检展开列表状态 | outcome: partial | AutoLang: `e9bd895531cf11f559be3aaf2ec18dcc19aac244` | AutoUI: `bcfc5fdeb255c5db534f3a1c20e8fea85155a77d` | next: T-02 原树辅助算法、组件参数及渲染。

原树状态的 `List` 拼写经 AGE 包前门进入普通 Auto 的 owned `List<Str>` 类型检查和所有权校验。初值、替换、`[]` 清空与长度读取都进入受检产物，给列表赋整数在原值 span 具名拒绝。AV 包观察使用支持集合和文本返回的既有 Dynamic MIR/ABC 合同，保持两种原 AGE 文法身份及独立组件轨。AV 回归验证两个实例首态均为 2，一个实例清空为 0，另一实例展开为 3 并读取列表第二项 `docs/计划`。

集合 SFC 使用官方 A2TS 运行时，在该 SFC 的模块脚本中提供它。状态在页面实例中持有，替换、撤载和部分构造失败均释放相应的文本所有者。Chrome 用本树真实生成的独立样本验证右实例展开、左实例清空、可见计数与实际 ID 顺序，并核对关页后旧按钮无效果、再次挂载恢复默认值。首次关闭累计分配与释放均为 7，重开再关闭均为 9，双重释放、存活文本与浏览器错误均为 0。

未修改的两个画廊页及两个树组件均通过包前门。原 FileTree 按文件装载和 A2Vue 路由生成共同推进到真实 `official::tree_util`，在第 26 行 `while` 处报当前受检方言的具名诊断。后续需要接通原辅助算法、嵌套节点集合、props 及循环渲染，然后回放原项目交互。027 的受检文件接面及两端操作仍待交付。

| 验证范围 | 实际结果 | 准备与执行 |
|---|---|---|
| AutoLang 核心与 AV | 组件 26/26、冻结语法 4/4、事件 19/19、AV 包 10/10、独立组件 8/8。 | 准备 27.242 s，组执行分别为 0.288 s、0.390 s、0.094 s、0.179 s、0.277 s。新增 AV 单例 0.108 s。 |
| AutoUI | widget package 5/5、SFC 11/11、closure SFC 2/2，包 SFC 启用 `AUTO_UI_ACE_WEB_E2E=1`，既有两项实际浏览器回归均执行。 | 准备 16.845 s，执行 0.314 s、3.311 s、0.059 s。 |
| 独立 Vue 集合样本和原源 | 独立样本 Chrome 验收通过。原树四份源通过投影，两个原 FileTree 入口到达共同辅助模块。 | 样本链接 5.708 s、生成 0.258 s、浏览器 1.102 s。原前门探针链接 0.474 s、执行 0.230 s。原双端入口探针链接 5.818 s、执行 3.186 s。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-list-final-lang-status.txt` SHA256 `7ABCC07C69BF48559350122201903AE6F4FCB651AE83D25BA609C4DE8AF67A42`，`r2-list-final-ui-status.txt` `40FFB1B0E03FC9C8F05F7E2D822FA0E4060E330E14D154916DCEC296F87D3F42`，`r2-list-browser.log` `2BAC344F8448666B4CEC9ED1E2ECE39B591BC9E054EA9ED055CEF229245164FD`，`r2-list-filetree-host.log` `D8DA375A857BEBF822A478150B69DA77DBABB46FD958EE13ACBBD330331A47C6`，`r2-checked-list-raw.log` `CE3CAF6A5D7F5FD1346737EBEE28DE760414441EE37E2BF3E92F67F666277C23`。动态载体首轮因函数 owner 与模块层 schema 不一致被 MIR 核查拒绝，已统一后完成正负回归。编译用 `cargo test --locked --offline -j 4 … --no-run`，执行直接调用二进制或实际 Chrome 脚本。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: bcfc5fdeb255c5db534f3a1c20e8fea85155a77d | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 树包辅助函数与双端列表操作 | outcome: partial | AutoLang: `af933b9cafe4e546566831ef46cce1cda4abea73` | AutoUI: `f7957fb641dd9b6cb2924be4b98765fbb7f0cd2b` | AutoOS 源: `efcfced51b58a1174f0b4fd3c0288c589495f70e` | next: T-02 组件视图事件载荷、受检节点记录和渲染。

AGE package 增加辅助函数中 `while`、索引和对象值的投影。原页声明的组件包目录名可用于引用本包辅助函数，限定在声明的模块及组件包内部，仍核对导出符号。两个原画廊页、两个原组件和原 `tree_util.at` 均通过 AGE package 投影。AGE 中列表文本比较复用受检内容比较，取值期间借用列表元素。保存到旧式文本列表时持有独立拷贝。A2Vue 把调用到的包辅助函数交给 A2TS 整函数生成，保留变量声明、控制流、清理和传递调用。AgePackage 文法 SHA256 `5385a91b135ef61c05b9d883de397eab08eebf4ea8ad04728b17ddd8b19f7d71`，相邻冻结面回归通过。

| 验证 | 结果 |
|---|---|
| `ace-core` conformance `plan981` 和 `plan1072_component_style` | 248 和 26 例，调用 998 ms 和 126 ms。验证组件目录限定名同时适用于页面与包内部，缺声明时具名拒绝。 |
| `ace-av` conformance `plan981_age_package_av` | 11 例，140 ms。跨模块运行保留原 `toggle_id` 函数体并标注 `List<str>`。两个实例按不同载荷分别产出 `1:src` 和 `4:docs/计划`。旧 VM 对函数原写法与补类型副本均产出 `1 / src / 4 / docs/计划 / 3 / true`。 |
| A2TS、AutoUI 定向组 | `ts_ui_fragment` 4 例 846 ms，`ts_runtime_and_provider` 3 例 248 ms。`plan114_widget_packages` 5 例 335 ms，`plan980_package_sfc` 11 例 133 ms，`plan980_closure_sfc` 2 例 52 ms。 |
| A2Vue Chrome 独立探针 | 1300 ms。正式 SFC 生成的检查样本导入加类型的原 ID 辅助函数副本。两实例分别追加 Unicode ID 和移除全部重复项，另一实例的状态保留。重开恢复原样，关闭后两次分别 `11/11`、`14/14` 次文本分配与释放，存活和重复释放均为 0，关页后的迟到点击无效，浏览器错误 0。 |
| 两个原画廊宿主探针 | `NodeDesktopBridge::load` 与最小 Vue 路由壳执行至 `official::filetree` / `official::treeview`。两端均把原 `.Toggle(r.id)` 当普通函数解析并报 `source.module-symbol`。未取得原窗口交互、原浏览器画廊或 027 的磁盘验收。 |

各 Rust 目标先 `cargo test --locked --offline -j 4 … --no-run`，再直接调用对应测试二进制，`--test-threads=1`。材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-tree-helper-fixed-lang-status.txt` SHA256 `BEB5D20AEDDB2882A0AD55EFA7D12AD42302B2489A7E9F0C82B77A6017D4FF4C`，`r2-tree-helper-fixed-ui-status.txt` `881DF3ED7B3C8D806C88AB81568974F0FB70C80D14F4F214602171781C7701F0`，`r2-tree-helper-fixed-browser.log` `B12EF896F1D7A44C169D8A01DF8B0AED8DADA0A667528C7E46EC01970FE6D01F`，`r2-tree-helper-fixed-original-status.txt` `2E9B34174F771575E5E86031F1354DB020D474ECCE63401F92967316630780A2`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: f7957fb641dd9b6cb2924be4b98765fbb7f0cd2b | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 视图消息与 AV 标量状态 | outcome: partial | AutoLang: `652c6ba7f9c3dff86c3a70e20fa8a45f1ab3fbe1` | AutoUI: `671411ed9848aafe0ae8613fec093cfe6a2968c0` | AutoOS 源: `efcfced51b58a1174f0b4fd3c0288c589495f70e` | next: T-02 原树节点记录、计算行与双端行事件。

AGE package 保留 `.Msg(arguments)` 的实参和原位置，核对消息声明、位置参数数量与类型。视图循环的值与可选索引进同一个受检包的词法域。模型和循环载荷的检查函数仅供编译检查。AV 包观察保留原 `int` 行并增加按声明序的标量快照，桌面按钮从目标实例的当前状态取载荷。当前桌面按钮消费标量字面量和状态字段。循环行渲染与 A2Vue 带参 handler 仍待接线。

| 验证 | 结果 |
|---|---|
| `ace-core` conformance `plan981` 和 `plan1072_component_style` | 248 和 27 例，调用 1428 ms 和 152 ms。`.Pick(row, pos)` 正例使用 `List` 文本元素与整数索引，拒绝错误数量、错误类型、越域变量和未声明消息。 |
| `ace-av` conformance `plan981` | 58 例，461 ms。不同实例的 ID 变更经 AV 状态函数读回原 Unicode 字符串。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 6、11、2 例，调用 387、152、575 ms。预备 UI provider 下通过实际 AV 事件队列定向点击，右实例取变更后的 `docs/计划`，左实例取自己的 `docs`，关闭成功。该档为桌面宿主测试。 |
| 原 FileTree、TreeView 宿主与最小路由 SFC 生成 | 两个入口均越过 `unknown function Toggle`，首次拒绝为 `source.undefined: no type named List in this module`，同时报告组件属性及计算行基座缺失。当前无原 Iced 窗口或原浏览器画廊结果。 |
| 节点表示可行性探针 | 递归 `List<TreeNode>` 通过源编译后拒于 `mir.type-cycle`。从列表元素读取其所拥有的 `children: List<int>` 拒于 `mir.field-type`。普通 Auto 的有类型节点列表以父索引表示边关系，真实 AV 扫描产出 `I32(1)`。这三个探针仅用于选择原树适配路径。 |

分别准备 `cargo test --locked --offline -j 4 … --no-run`，直接执行相应测试二进制 `--test-threads=1`。UI provider 使用已准备产物，`ACE_NODE_DESKTOP_PREPARED_RUSTC` 为 `rustc --version`。材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-tree-event-scalar-lang-status.txt` SHA256 `9316B8EBD7DC1C0F14A0D57007326D0E1EA298CE8CC42C84F8B231A8823B8D45`，`r2-tree-event-scalar-all-av981-status.txt` `EE5FC36A68771C06CD208CBD42D64335183192C4F2DDCE0DD88A0928599054DB`，`r2-tree-event-scalar-ui-status.txt` `39D5F89F5F0CA41B6035BF5DB547468CCA68806F8CEB3B5B7A27D57AA123281E`，`r2-tree-event-scalar-original-status.txt` `193EA8887EE6082CA227E214473AFDE12367A810FB83079999D8DE8E47A25B2B`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 671411ed9848aafe0ae8613fec093cfe6a2968c0 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 有类型的节点列表与逐实例计算值 | outcome: partial | AutoLang: `1cee3518e719f8910194759001a4d6ac4bce2bfa` | AutoUI: `283fea1a31fe5c29a64978889057bd60e2b568d3` | AutoOS: `ac9f73fdae1cfd31e40e890cd47fd6c01e1c3dfe` on `plan-114-dev` from `efcfced51b58a1174f0b4fd3c0288c589495f70e` | next: T-02 原树节点数据与组件参数的受检投影、行渲染。

AGE package 支持显式 `List<T>` 模型注解、共享列表字面量及其名义记录元素。计算属性由共享编译器推断返回类型，每个实例生成受检求值函数，支持视图文本和事件循环引用，读取当前实例的状态。节点文本字段在需要物化时走既有受检拷贝。原静态 FAQ 数据与文本列表属性边界有回归。AgePackage 文法 SHA256 `d28959c2d30d05ad3756f3be1cb857afb31c298e399f436bd69a1dd6a22a662e`，相邻冻结面通过。AutoOS 的 `widgets-gallery` 原 `has_id` 与 `toggle_id` 标注 `List<str>`，保留函数体与公开消息。A2Vue 对视图中实际引用的计算值给出 `source.age.sfc-computed`，未引用的计算声明可生成。

| 验证 | 结果 |
|---|---|
| `ace-core` conformance `plan981`、`plan1072_component_style`，`ace-av` conformance `plan981` | 248、27、59 例，调用 1114、143、443 ms。名义行列表驱动跨模块辅助查询及 `row.id` 载荷检查。两实例经 `computed` 输出子节点数 2，只有目标实例显示 `docs/计划`。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 7、11、2 例，调用 407、132、672 ms。预备宿主的两实例事件、路由和现有 SFC 通过。验证引用计算值给出具名诊断，以及未引用声明保留既有生成。 |
| 两个原画廊字面量的可逆对照 | 在材料中按原始嵌套顺序生成带父索引的名义节点，逆转换逐字段核对原值。AV 通过编译和执行读回 FileTree `614:604`、TreeView `20:12`，分别耗时 1204、13 ms。原页数据仍保留作者的嵌套输入。 |
| `widgets-gallery` 原辅助模块的旧 VM 兼容性 | 用原 TreeView 节点运行整份 helper 加独立调用脚本。标注前后输出 SHA256 一致 `EBC7357366331FBA31B65ABA524C1FC29CF11DB4A247187ADCC1ECFE6C04D852`。默认展开可见 9 行、全部目录 ID 12 项；重复 ID 移除、Unicode ID 追加及原列表保持均一致。 |
| 原 FileTree、TreeView 两端入口 | AV 宿主与最小 Vue 路由壳仍首拒于其余旧式无元素注解的 `List`，调用分别 2950、762 ms。原画廊实窗、浏览器和 027 文件矩阵待验收。 |

分别 `cargo test --locked --offline -j 4 … --no-run`，再直接调用测试二进制 `--test-threads=1`。证据根 `D:/autostack/plan-artifacts/plan-114/`。`r2-tree-records-final-lang-status.txt` SHA256 `6E7A849FA65F8B09DA1836E203F0C98387E10F23F7E7F5438CFB0170BBCAE0F5`，`r2-tree-records-vue-fixed-status.txt` `41D57F069369E3D320DC12F6B98C55B1B346F6C9BD3AD120F970E433EBD359E8`，`r2-tree-records-authored-final.log` `53D4B12C4FDFF2453F4EEEEE4E052030FEC48E7711D22E9A1841D40107DB0709`，`r2-tree-records-verified-original-status.txt` `8ADC88D6DD4B9BCB1B22D47F5EF2D5C01EE67E3AF19DCD66050069B832265CB2`。

本轮 PLAN-094 合入暂存工作区草稿。本计划正文从 `e56eeac0ffaf5ee07492fa463da141adf2a77195^3` 单文件恢复，核对原 blob `f65ffb40676966170b49fe8b1da81c5fb6770f03` 和第 11 轮收据后续写。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 283fea1a31fe5c29a64978889057bd60e2b568d3 | next: work/T-02

2026-11-20 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原树嵌套字面量与 AV 遍历预算 | outcome: partial | AutoLang: `7d6eabe4c03fc9326a54a2e34de14f9d07d8f34c` | AutoUI: `283fea1a31fe5c29a64978889057bd60e2b568d3` | AutoOS 源: `ac9f73fdae1cfd31e40e890cd47fd6c01e1c3dfe` | next: T-02 类型化组件属性、原树辅助模块与宿主渲染。

AGE package 以 `List<名义记录>` 注解把模型中的静态对象简写构造成受检记录，递归子列表保留原始顺序和字段。展开在共享包绑定后借用绑定器给出的记录布局，错误沿字段原位置归位。MIR 从 `List<Node>` 先进入时可复用已预留的记录类型索引。直接值递归仍由原循环检查拒绝。该轮改动位于 AutoLang，原画廊生产接线仍待实施。

| 验证 | 结果 |
|---|---|
| AutoLang core `plan981`、`plan1072_component_style`，AV `plan981`、`aggregates_t01_probe` | 248、27、60、33 例，1462、166、541、242 ms。验证两实例读取三级 Unicode 子节点、字段类型错误定位、递归列表释放平衡、Box 递归和直接值递归诊断。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 7、11、2 例，764、291、833 ms。桌面事件、A2Vue SFC、路由与既有视图数据继续通过。 |
| 两份原字面量的完整 AV 遍历材料 | 原字面量逐字节提取，独立 Python 计算节点、边、目录、`is_leaf`、非空 badge 数。AV 默认 100 万步返回 FileTree `614:613:3:0:3`、TreeView `20:17:12:1:2`，总调用 2626、41 ms。每个局部节点的 `children` 先绑定一次再取子项，诊断计量为 799408、28716 步。逐次透过字段取子项的 FileTree 变体超过默认额度，诊断执行记录 6234762 步。 |
| 深链编译资源 | 上述材料主跑的 Windows 调试线程为 16 MiB。2 MiB 的 TreeView 探针只调用共享包绑定器时已栈溢出，4 MiB 调试线程全链通过。生产入口接线时须核其实际栈配置与深链。 |
| 旧 VM 类型注解对照 | 同一个原 helper 加完整字面量，独立脚本有无 `List<TreeNode>` 注解时各页结果相同。TreeView 得到 20 个节点、12 个目录。独立 FileTree 基线与标注版都打印根 ID 为数字且 `collect_ids` 返回 102 项，故该对照仅检查注解的兼容差值。 |
| 原 FileTree、TreeView 两端入口 | 使用更新后的宿主探针，3363、787 ms。AV 与最小路由 Vue 均仍首拒其余未标注元素的 `List`，后续还缺组件属性绑定与行域求值。原产品 Iced 和浏览器验收待继续。 |

Rust 目标先 `cargo test --locked --offline -j 4 … --no-run`，再直接调用目标测试二进制 `--test-threads=1`。UI 测试使用已准备的 provider，`ACE_NODE_DESKTOP_PREPARED_RUSTC` 为 `rustc --version`。材料根 `D:/autostack/plan-artifacts/plan-114/`，下列记录按 SHA256 固定。

- `r2-tree-nested-lang-status.txt` `751918CF947E6FA7D318FCE3B16F3F760D812BADB1BD32C3573C14502941F874`
- `r2-tree-nested-ui-status.txt` `F59F65FAAA8CC2AA1A1FF718C4EB16E755A5DBE2436E75B93DC1B9614A3DB1FC`
- `r2-tree-nested-authored-bound-status.txt` `5D9AB6EF7C9EE7C294A172356A0F19D23D889D41F06C7FA6C26624A6A347E339`
- `r2-tree-nested-authored-status.txt` `622DB36EF30EA2B03B5D920CE214CDFB5141DE52BF90424BF0D0AFB44254D66A`
- `r2-tree-nested-stack-isolation-status.txt` `102698FB2D915D939B769BB20051E4F6A70DDEC3A57DA2B6AD4CA476AF762504`
- `r2-tree-nested-legacy-status.txt` `02DF7B4B2848C7501F510291716D8A161F58A99DB3466570661A31FF38C527F0`
- `r2-tree-nested-original-summary.txt` `5C83CA6AFD2051DF54FBA36CE19D02B644FAF2B352850612BA5CADA8DF78C1D4`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 7d6eabe4c03fc9326a54a2e34de14f9d07d8f34c | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检组件属性、行词法域和原画廊节点类型 | outcome: partial | AutoLang `466af425113a995966a0572c3dc98b38c35c6c20` → `b3316dc86a9e1305e800d9bc670cae3cce79fc74` | AutoUI `283fea1a31fe5c29a64978889057bd60e2b568d3` | AutoOS `37cdf5aaa10720dfe675420528adc66f6d392bcc` | next: T-02 原 `tree_util` 类型化遍历、子组件属性读取及双端渲染派发。

AGE package 按子组件声明解析显式 `List<T>`、标量属性、作者默认值和父消息绑定。传参表达式按父模块与实例生成，和子模块声明经同一包绑定及类型检查。AV 通过已验证的记录返回接面读取集合属性。循环中键、条件、文本、子组件属性与消息参数共享当前行和索引作用域。已将 `TreeNode` 名义类型用于原 FileTree 和 TreeView 的页面数据及组件参数。两页原字面量逐字节一致，调用表达式保留原样。

| 验证 | 结果 |
|---|---|
| AutoLang core `plan981`、`plan1072_component_style`，AV `plan981`、`aggregates_t01_probe`、`dynamic_objects_964`、`plan1082_collections_places` | 248、28、61、33、14、17 例，1068、152、556、159、131、440 ms。默认、显式、逐实例属性经 AV 返回真实列表；只派发到实例 1 时该实例读回 `changed/计划`，实例 0 保留 `ordinary`。覆盖缺参、重复、节点列表的类型不同、循环内错类型和越域、非布尔条件与坏通道。AgePackage 文法 SHA256 为 `ec375f3ec13c9c69be16457249aa0158d89cd96eea7dc214937d45a256ee5026`，相邻冻结面通过。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 7、11、2 例，544、171、723 ms。预备 UI provider 的实际桌面事件回归通过。 |
| 原字面量与原调用表达式的属性材料 | 在简单组件声明中保留原调用参数，AV 返回 FileTree 614 节点与 `[docs, docs/plans]`、TreeView 20 节点与 `[project, src]`，保留 TreeView 的 `TvToggle(str)` 和 `TvSelect(str)` 绑定事实。常规材料 3390、40 ms。此项覆盖属性输送，原组件的计算和事件派发仍待完成。 |
| 深链编译 | 将混合数组、对象子表达式改用既有投影工作栈。12 层回归在 2 MiB 通过。两份原字面量的属性材料在 Windows 调试 1 MiB 与 2 MiB 均通过。原宿主加载也运行到辅助函数的类型诊断。 |
| 原画廊旧 VM 构建 | 使用 `build_dynamic_component` 和 32 MiB 调试线程。旧、标注版页面均沿当前组件目录构建出可读取的视图树，分别观察到 `docs`、`project`。FileTree 867、545 ms；TreeView 532、502 ms。完整旧 UI 交互待产品验收。 |
| 原画廊 AV 入口及最小路由 SFC | 两页越过 `source.age.widget-prop-type`，仍拒于原辅助函数缺少元素类型的 `List`，2923、727 ms。子组件中访问属性的受检函数、嵌套列表只读遍历、渲染和点击派发是下一段。原 Iced 实窗、A2Vue 浏览器与 027 FileManager 验收未完成。 |

Rust 目标先 `cargo test --locked --offline -j 4 … --no-run`，再直接调用对应测试二进制 `--test-threads=1`。UI 测试使用已准备的 provider，`ACE_NODE_DESKTOP_PREPARED_RUSTC` 等于 `rustc --version`。材料根 `D:/autostack/plan-artifacts/plan-114/`，记录 SHA256 如下。

- `r2-props-iterative-lang-status.txt` `DE2B2F0708B6B5D141567A7984060FC8EB31906CC54FC155E4B9C9B8700AFD4F`
- `r2-props-iterative-places-status.txt` `496C2E96A7558E305149E6F78411CCF876323345D64326959B6920D3213B82E0`
- `r2-props-production-ui-status.txt` `9990D0001D1CAF82A98D08592EE98FD94ED693E0155854E7CBC1CEE65C53270E`
- `r2-props-authored-final-status.txt` `8F1802E21449AD7082F8C869C05FD2B6A6B97329C9BEC4B166B2C14598A9C71F`
- `r2-props-stack-status.txt` `A210EAF1D54BE9B7278DF7A8D7CB3E5D20D5BCFB8BEEFEF7C5FAAFE029EB5E2B`
- `r2-props-source-compare-status.txt` `7C8684C35E3271612459D04EF294AF76CF40B4F4374B2111C1FFE6A0A1D3826C`
- `r2-props-production-stack-legacy-status.txt` `B3DA1BDE597A7C4DB184C14DE80723182F02C01883B08B9FFE11FDFDBC383B1F`
- `r2-props-production-checked-status.txt` `B3A2FF028E3D703E701B4A349D0580FFB54904AA2086E392CD423C5429D6F18C`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 14 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: b3316dc86a9e1305e800d9bc670cae3cce79fc74 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原画廊树辅助函数双目标执行 | outcome: partial | AutoLang `e65a1d5a958a9d523c018c220db6e8ffa6e7b2d8` | AutoOS `827d574b27aae71399cb3f73b4f0290f3de214d8` | branch: `plan-114-dev` | next: T-02 受检子组件读取实例属性，再接原组件的循环渲染和事件派发。

原 `tree_util.at` 以 `TreeNode`、`TreeRow` 表达嵌套节点、行和缩进导轨。按原顺序收集 ID、生成可见行和过滤命中路径。AGE 通过显式能力保留普通集合参数的共享语义，并用受检的字段借用复制单个文本。短文本谓词走现有的 Unicode 扫描和标准库实现。A2TS 种子可表示列表字段，同一输出可在 JavaScript 执行树遍历，写入列表的文本遵循受检元素所有权。

| 验证 | 结果 |
|---|---|
| 使用原字面量和实际 `tree_util.at` 生成辅助函数材料 | 依次测目录 ID、全部 ID、默认可见行和搜索后的节点数。FileTree 在 AV 和 Node 均得到 `3,614,10,4`，TreeView 均得到 `12,20,9,12`。AV 使用默认执行上限，分别 6842、206 ms。Node 执行生成的 JavaScript 分别 74、50 ms。四个 AV 结果由各自的受检文本求值调用产生。 |
| AutoLang core `plan981`、`plan1072_component_style`，AV `plan981`、`aggregates_t01_probe`、`dynamic_objects_964`、`plan1082_collections_places`，AV core conformance | 248、28、62、33、14、17、51 例通过。新增 AV 回归覆盖共享的嵌套子列表、重复调用、字段文本复制、Unicode 包含与后缀。 |
| A2TS seed、runtime/provider、named rejections、UI fragments | 10、3、2、4 例通过。验证列表字段在记录克隆时保持共享，以及 Unicode 标量偏移和空模式边界。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 7、11、2 例通过，670、300、851 ms。 |
| 原画廊旧 VM 构建 | 从实际页面入口构建视图树，FileTree 含 `docs`，TreeView 含 `project`，用时 561、533 ms。早期独立旧 VM 脚本对完整 FileTree 的计数仍有 §9 已记载的基线偏差，因此完整计数对照采用受检 AV 和生成的 JavaScript。 |
| 原画廊 AV 及最小路由 SFC 入口 | 两页均越过旧的无元素类型 `List` 诊断。当前两端首个诊断均为 `source.undefined: no local named self in scope`，用时 3003、678 ms。受检子组件还需将父站点已经接通的实例属性绑定到子组件计算。原 Iced 实窗、A2Vue 浏览器和 027 FileManager 验收仍待实施。 |

Rust 目标先用 `cargo test --locked --offline -j 4 … --no-run` 准备，随后直接运行测试二进制 `--test-threads=1`。AutoUI 启用 `ace-bridge`，已准备的桌面 provider 以 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 指定当前编译器。材料根 `D:/autostack/plan-artifacts/plan-114/`，记录 SHA256 如下。

- `r2-tree-util-cross-status.txt` `B528AEDDE2C8C2627F481559FA82923EF28CF46FF21A1849D822EE6C6AC8F57F`
- `r2-util-lang-status.txt` `EC28CBA04635B1B47F30D10B64BF42517E7FEBCC1A536D9A182D6DD463496F8A`
- `r2-util-final-ui-status.txt` `D36AA8F8B7765E06F5D432BFBEB45BA821096B1712FE871C9A1E7249364A9999`
- `r2-tree-util-real-legacy-status.txt` `63300C7C56DD50F5C26D8A4F9E0A567704A24AD2D2716D085F6169A63F13A494`
- `r2-util-host-status.txt` `D5F091C83ECDFFF5EE5822AA0DC892584B574AA8CB9AB27141D1CDA72FE87239`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 15 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: e65a1d5a958a9d523c018c220db6e8ffa6e7b2d8 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检实例属性驱动原树组件计算 | outcome: partial | AutoLang `872441620b338bcf9d8d6107c1b5a657bd50ab12` | AutoOS `0483c6902bfc17b47bc80e602d7c1b560662a3ac` | branch: `plan-114-dev` | next: T-02 让生产宿主挂载子组件并执行受检循环视图与事件。

包内 widget 将值属性绑定到受检函数参数。派生计算可读取属性、状态与其他计算，事件检查和场景中的 `Init` 可读取同一属性。父站点实参和作者默认值按子声明顺序交给 AV 包内调用，MIR 与 ABC 对完整调用重新验证。循环行标签按文本读取，生成的 JavaScript 执行普通 Auto HIR 的局部释放。

| 验证 | 结果 |
|---|---|
| 原 FileTree 的默认与归档选中场景 | AV 与生成的 JavaScript 的行数据摘要一致。默认 10 行，只展开 `docs`、`docs/plans`。派发原 `Toggle` 和 `Select` 场景后为 614 行，显示归档叶与 badge `604`，选中 `docs/plans/archive/614-treeview-filetree.md`。AV 用时 1928、2026 ms。 |
| 原 TreeView 的默认、展开选中、收起场景 | 两个目标行数据摘要一致。默认 9 行、展开 `project` 和 `src`。展开全部并派发原选择后为 20 行，显示且选中 `bottom.txt`。收起后为 3 行，深层叶隐藏。AV 用时 300、304、309 ms。 |
| AV 指令预算 | 四个场景使用默认 100 万步。FileTree 的归档展开并选中同时物化父节点数据和子行。优先匹配 Markdown 图标后缀后，显式 200 万步时通过，150 万步仍具名拒绝。此前规则在 250 万步仍拒绝、300 万步通过。 |
| AutoLang core `plan981`、`plan979`、`plan1072_component_style`，AV `plan981`、`aggregates_t01_probe`、`plan1082_collections_places`，A2TS runtime、named rejections、UI fragments | 248、57、28、62、33、17、3、2、4 例通过。AV 属性用例覆盖不同使用点的默认和显式值、同站点两个实例的独立值及子组件 `Init` 派生。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` 与旧 VM 原入口 | 7、11、2 例通过，502、159、710 ms。旧 VM 实际页面构建用时 559、547 ms，仍观察到预期的 `docs`、`project`。 |
| 原 AV 桌面入口与最小路由 SFC | 双端原页越过 `self` 未绑定及循环标签借用错误，到达既有 `ace.desktop.widget-unsupported` 和 `source.age.sfc-widget-unsupported` 拒绝。上述 AV 和 Node 材料按场景定向调用原事件。生产挂载、视图事件路由、窗口及浏览器验收待做，027 仍在原范围内。 |

AutoUI 计划分支已经包含 PLAN-125 的交互控件基础 `1b0984f7`。PLAN-123 当前分支 `9d5ea5b3` 继续持有路由项目启动和主题工作。下一切片将从 AV 包桥与受检行表达式入手，并在接入共享渲染入口前复核并发改动。

Rust 目标先用 `cargo test --locked --offline -j 4 … --no-run` 准备，再直接调用测试二进制 `--test-threads=1`。原页材料从同组 AutoOS 业务源、组件声明、图标组件及实际帮助模块编译。材料根 `D:/autostack/plan-artifacts/plan-114/`，记录 SHA256 如下。

- `r2-widget-input-cross-status.txt` `A79250621B47AF8BCADF3FF3BDEC7056A4FE3AC618329CEE3C8A653A89D14A4A`
- `r2-widget-input-budget-status.txt` `FA65B4105761631984465B98D52887C06259CD579BE307A036F14445A86C0F2D`
- `r2-widget-input-budget-before-status.txt` `08445D004FBF6CD6501A8077CE2514437CBDCD9F0EEB0ED9716B1BE97F1DFF95`
- `r2-widget-input-lang-status.txt` `3DFC46AB2ADF6C7031A7A8759DCEDB8595CE371D3F056BE0ABBDA003E4D6B959`
- `r2-widget-input-ui-status.txt` `81625A1579F216948C8039F5D8385D887BC5333AB5DAC7DF3C871705ACDF6ACE`
- `r2-widget-input-host-status.txt` `4FF2AD1BB2B1068A5109B884AA6C390DD8DAAA0165A487901320331348661AE9`
- `r2-widget-input-legacy-status.txt` `CA2162C943A919ECFB93BBF14076B644EF16A15F2DB3398A74258D211F90CB43`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 16 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 872441620b338bcf9d8d6107c1b5a657bd50ab12 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检原页面与逐行子组件视图 | outcome: partial | AutoLang `e784c0e1f4ef820f6066c1932bfaf28900557d13` | branch: `plan-114-dev` | next: T-02 在 AutoUI 包桥上发布 AV 视图和按实例、世代及使用点派发的事件。

AGE 使用实际视图树生成受检的循环、分支、文本、属性及事件参数函数。循环索引来自当次迭代。AV 组装入口页、树组件与实际 `TreeIcon`，保留已绑定的源使用点、元素结构及类型化载荷。相同源使用点和相同标量输入的图标复用当次求值的结果。页面级属性继续走受检 getter，行内标量由受检作用域输出后按子声明类型传入。

| 验证 | 结果 |
|---|---|
| 原 TreeView 与 FileTree 视图的双目标求值 | 五个场景的 AV 与生成 JavaScript 摘要完全一致。分别观察可见行、原文件标签、选中样式、展开消息的节点 ID 和图标属性。FileTree 为 10 或 614 行，TreeView 为 9、20 或 3 行。 |
| 原入口及子组件组装 | 真实入口载有绑定的 FileTree/TreeView 使用点。TreeView 入口提供 `TvExpandAll` 和 `TvCollapseAll`。AV 完整组装的 TreeView 默认、选中有 13、32 个实际图标，FileTree 默认、归档选中有 13、617 个实际图标。Windows 调试用时 811、950、8148、7634 ms。 |
| AV 预算 | 默认 FileTree 与 TreeView 场景、TreeView 展开和收起均用 100 万步。FileTree 的 614 行视图物化曾在 300 万步拒绝，用显式 600 万步通过，包括逐行事件参数与嵌套图标求值。已记录每次调用预算，后续生产入口仍需核对响应时间。 |
| AutoLang core `plan981`、`plan1072_component_style`，AV `plan981`、`aggregates_t01_probe`，AV core conformance，A2TS runtime 与 UI fragments | 248、28、62、33、51、3、4 例通过。回归核两个循环索引、条件文本、循环内字符串/整数/布尔点击载荷和实际的嵌套子组件求值。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 7、11、2 例通过，用时 494、158、681 ms。 |
| 原生产入口 | 重新用实际原页面探测，仍在 AutoUI 包桥和 SFC 的 widget 挂载限制处拒绝，分别用时 3730、957 ms。上一行的宿主回归和上述组装材料未运行 Iced 窗口、浏览器或 027。 |

生产 AV 接线从 `ace_package_bridge.rs` 消费已组装的入口视图。宿主需用绑定使用点确定本地消息或父回调，并让同一实例的新视图和事件表同时提交。关闭或更换页面后要核对旧事件的世代。PLAN-123 的启动和主题仍由其分支持有。

Rust 目标以 `cargo test --locked --offline -j 4 … --no-run` 准备，再直跑测试二进制 `--test-threads=1`。原页和组件按当前工作树读取，Node 运行从相同受检 HIR 生成的脚本。材料根 `D:/autostack/plan-artifacts/plan-114/`，记录 SHA256 如下。

- `r2-live-view-cross-status.txt` `EAB421A654937FF0DA0776A0D3AEA028D61AAEFA883C4049417C6D5A0998111C`
- `r2-live-view-lang-status.txt` `89B318707860B7E54EC912A54F4E1176EF5690ADBA7C96970B9EAD303F98D3BB`
- `r2-live-view-ui-status.txt` `AF145F2E477EE3C3683BFBB5041AEE960DE70A68CA24CB03107894F3DBD60012`
- `r2-live-view-host-status.txt` `AB44D53F0EF45549DD00821FDED4DA41F0DD6A8F2A2A00D70C1B753835CBE50D`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 17 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: e784c0e1f4ef820f6066c1932bfaf28900557d13 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 AV 生产桌面挂载与点击派发 | outcome: partial | AutoLang `4d1883bb1d3e883441d7141e2222d19eee1baa6c` | AutoUI `b2d9e05ae40e36d0d16ac60545099fac158006a5` | branch: `plan-114-dev` | next: T-02 FileTree 交互时延及余下回放，并接 A2Vue 原页面。

真实桌面包入口消费受检的 AV 视图，按原使用点将点击送往子组件本地处理器或父级回调。发布时同时提交视图和点击表。实例分开保留事件历史，关闭时丢弃已发布视图与队列，重开按源声明重新执行 `Init`。宿主以会话、实例、视图版本核对点击。源中同一个含处理器的子组件在同一根实例中重复挂载时仍具名拒绝，避免共享事件历史。

| 验证 | 结果 |
|---|---|
| 原 TreeView 生产宿主 | 加载实际原页面与组件闭包，经 `AceNodeDesktopWindow` 的消息入口程序化操作。9 行默认，经 Expand All 到 20 行，`Select(bottom)` 经原回调成为 `TvSelect`，确认 `tvSelected=bottom` 和选中样式。收起为 3 行，重开为 9 行。旧视图和旧会话各拒绝一次点击。包含加载、挂载、操作和关闭总用时 6502 ms。 |
| 原 FileTree 生产宿主 | 加载实际原页面与组件闭包，首次真实事件发现把父页的源码位置用于子模块会拒绝。改为使用目标组件的源码锚点后，默认 10 行，归档 Toggle 显示 614 行和原归档文件，关闭会话通过。加载 1638 ms、挂载 11156 ms、点击 9761 ms，总用时 22694 ms，超过 10 秒预算。已复用同次组装的降级模块，并在可证明入口无条件挂载时省去额外渲染。继续定位逐个图标绑定时的重复校验和编码。原 FileTree 的选中、收起及重开尚缺生产宿主回放。 |
| AutoUI 和 AutoLang | `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` 为 7、11、2 例通过，用时 703、156、699 ms。新增行为在既有用例中验证，其中实例隔离与会话拒绝单跑 288 ms，嵌套组件闭包单跑 171 ms。AutoLang AV `plan981` 62 例通过，用时 906 ms。 |
| Windows Iced 实窗 | 原 TreeView 成功创建 Iced 窗口并发布 9 行。Computer Use 两次报告 `failed to activate captured window`，随后截图确认 Windows 锁屏。已停止输入并关闭本轮测试进程。真实鼠标操作和窗口正常退出仍待验证。 |

A2Vue 生产入口仍具名拒绝自定义 widget，027 真实文件和授权矩阵尚待执行。当前计划保持执行中。FileTree 生产探针仍需满足 10 秒成本要求后才能作合并凭据。

Rust 目标先用 `cargo test --locked --offline -j 4 … --no-run` 准备，随后直跑二进制 `--test-threads=1`。桌面 provider 使用 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 已准备路径。语义点击探针以实际原页面为输入，并从当前发布的点击表取得当次事件载荷。材料根 `D:/autostack/plan-artifacts/plan-114/`，记录 SHA256 如下。

- `r2-checked-host-ui-status.txt` `6922EF6670254F4920C5D6F58913D8C0B02C5644479CBC0F4F089F5863227A20`
- `r2-checked-host-lang-status.txt` `70016FC34136F47868BBA4B7E2A8DC2226F4D2758A9264F64F5B4488FADD0770`
- `r2-checked-host-treeview.txt` `5F5077C6D313825B4CFAF0BA07EF614BFF82E6BF7B6BCE6BDDC62AAF471437DF`
- `r2-checked-host-filetree.txt` `F97C724BAC7750B3DEC58EE3E368201D8527348B7049D30B7B172C267A4C2027`
- `r2-checked-host-filetree-expand.txt` `362628C217CC7F7214AD7E5CF5077D88A2B2C0A0C0B82328D4E72B0B44D6058C`
- `r2-checked-host-treeview-window.txt` `CBA3BA650CC6D30E9253E32E61D4B3FCEE70EAC50F10AA9905E739A3E202CC98`
- `r2-checked-host-window-status.txt` `68EB989B035175818D17F79D68129CD1F513CC056D4BB9AEF42FB6156A04726B`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 18 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: b2d9e05ae40e36d0d16ac60545099fac158006a5 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原树页完整桌面宿主回放 | outcome: partial | AutoLang `0ebc1fd5e17afb2980bbb8c881fd1fd6c99aa181`、`7385f5fbecf39fd2debf275ac49583e41eed2da0` | AutoUI `673259d897884f10702789c22c1ba3d50dbf4d8c` | AutoOS `0483c6902bfc17b47bc80e602d7c1b560662a3ac` | branch: `plan-114-dev` | next: T-02 A2Vue 受检包状态与原树页接线。

AV 的 widget 求值从入口和初始化器按直接调用及闭包目标保留可达函数。组件初次挂载从已受检的入口拓扑取得 `Init` 目标，每个实例仍独立创建场景和视图。MIR/ABC 所用的所有权校验按位集中的活跃借用检查冲突，保持原借用条件和校验入口。原 FileTree 的 `nodes` getter 对照以同一 318196 字节 ABC 测得 MIR 与 ABC 校验合计由 1765.6762 ms 降至 56.0773 ms，实际执行分别为 21.2585 和 20.8884 ms。耗时记录包括分别调用的阶段与整探针运行时间。

| 验证 | 结果和用时 |
|---|---|
| 原 FileTree，经生产 `AceNodeDesktopWindow` 消息、步进和视图入口 | 默认 10 行。归档展开 614 行并显示原文件，选中原归档文件后出现选中样式，收起恢复 10 行并隐藏该文件，重开重新为 10 行。旧视图和关闭后旧会话各拒绝一次点击。加载 1612 ms，打开 1085 ms，三个点击分别 1338、1294、930 ms，含关闭和重开总用时 **7502 ms**。仅裁减函数时展开并关闭为 9079 ms，加入启动拓扑复用后完整动作首次为 15694 ms。保留两项中间记录。 |
| 原 TreeView，经同一生产宿主 | 原页逐步呈现 9、20、3 行，`Select(bottom)` 到达原页 `TvSelect`，父模型与选中样式可观测，重开恢复 9 行。旧视图和关闭后旧会话各拒绝一次点击。加载 518 ms，打开 437 ms，三个点击分别 425、424、402 ms，总用时 **2717 ms**。 |
| AutoLang AV、AV core | debug/no-run 准备 12.04 s。AV core 单元 71 例通过，总用时 348 ms。AV core conformance 51 例通过，总用时 250 ms。AV conformance 筛选 `plan981`、`av_owned_semantics`、`av_closures_semantics` 和 `av_collections_prerequisites`，127 例通过、1244 例不在筛选范围，总用时 668 ms。新增稀疏借用位集用例单跑 10 ms，核跨字和清零结果。新增入口闭包用例单跑 12 ms，核初始化顺序和直接调用及闭包重编号。 |
| AutoUI `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` | 各 7、11、2 例通过，分别总用时 565、156、682 ms。debug/no-run 重试约 90 s，首次准备在 `sccache` 包装下报 `STATUS_ACCESS_VIOLATION`，重试仅在本次构建禁用包装器。AV 首次准备因重复 `--lib` 被拒，改用单个 `--lib`。诊断探针的首次直链缺 `windows.0.52.0.lib` 搜索目录，另一次首次编译误用了属性入口签名，修正后重新测量。 |

运行按 `cargo test --locked --offline … --no-run` 分离准备，Rust 测试二进制以 `--test-threads=1` 直接执行，计时覆盖本轮命令的启动、执行和清理。`r2-widget-verifier-host.exe` 以 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 使用已准备的 provider，分别传实际原 `filetree.at` 与 `treeview.at` 的绝对路径，不加 `expand` 或 `window`。两个完整宿主探针达到用例时间要求。Iced 真实鼠标验证仍受上轮 Windows 锁屏影响，A2Vue 冷构建及浏览器回放、027 权限与真实文件端到端仍待完成。

材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 清单如下。

- `r2-widget-verifier-filetree-full.txt` `3EAF1E52AA7AC832D9525E653FD2A661AD4F62A5CCB3D67C2C1719C5D691DEFE`
- `r2-widget-verifier-treeview-full.txt` `A56E9C761E5052A769B461F5CEF3B038AD3932E85CDAA863B6FE9FBB169CF97E`
- `r2-widget-getter-verifier-cost.txt` `26EF877C46AD24E1B15243EA50E711EABB0F217888457185BD9ECC87B069CC7D`
- `r2-widget-getter-cost.txt` `48AACEA294C1899A48D402885F2EC38F66E86087A168D0AD975089E87AB95877`
- `r2-widget-verifier-ui-status.txt` `C3F6B1D3BB78CB7D192DE901ECC5AEA43A35B29EDA7EF331EA3AD72E4884E6AE`
- `r2-widget-verifier-lang-status.txt` `97E244561BE343EFAB6644E09F7B0B1E56373EF4FBDDDC347E06E45325E52F53`
- `r2-widget-verifier-ui-build.log` `3B7134C88A27FAC702279B88D1C56D91B8E646CEE56B604EB45D0CC37968A92F`
- `r2-widget-verifier-av-build-first.log` `A4337F8D21D675FB3846D12053AF2F48D47E626696BB16365B2073A67B11EE4D`
- `r2-widget-seed-filetree-full.txt` `E5D3B1626336C8BCAE13C178A95198E5FEE78181DFB800B4198B915546F9883F`
- `r2-widget-closure-before-seed.txt` `F27F8D7C69281F360566C7E47D93B0CBB862AA5566D90053C487EAFEE5725869`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 19 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 673259d897884f10702789c22c1ba3d50dbf4d8c | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 A2TS 受检状态转换与动态树视图 | outcome: partial | AutoLang `fa1daa2dc8f87f0b8736549bae6b6beca92999fa` | AutoUI `673259d897884f10702789c22c1ba3d50dbf4d8c` | AutoOS `0483c6902bfc17b47bc80e602d7c1b560662a3ac` | branch: `plan-114-dev` | next: T-02 接入 `ace_package_sfc` 的 Vue 实例生命周期和点击回调，回放原页面。

`compile_age_package_widget_runtime` 从绑定的入口及子组件图生成可复用的函数。初始化器返回声明模型，消息函数按声明接收旧模型及类型化载荷并返回更新后的模型。视图和受控子组件属性用同一当前模型，复用已有的作用域绑定、视图生成和受检 A2TS 函数发射。旧场景入口继续使用原模型重放。当前可调用函数测试以 Node 执行，Vue 生产入口仍需接线并冷构建回验。

| 验证 | 结果 |
|---|---|
| 原 FileTree 与 `official` 闭包经 A2TS | 编译及输出 135 个函数，用真实节点数据推进 `Init`、归档 `Toggle`、原文件 `Select` 和收起。检查实时行数 10、614、614、10、10，归档叶显示及选中样式，另一个模型仍按默认展开。整命令用时 1217 ms。 |
| 原 TreeView 与 `official` 闭包经 A2TS | 编译及输出 175 个函数。展开 12 个真实目录 ID，`TvSelect(bottom)` 后用动态属性将选中值传入原子组件，检查 `bottom.txt`、选中样式和独立模型。视图行数 9、20、20、3、9，最终展开计数为 0。整命令用时 467 ms。 |
| AutoLang core 与 AV | core conformance 筛选 `plan981` 和 `plan1072_component_style`，276 例通过、579 例在筛选外，总用时 1036 ms。AV conformance 筛选 `plan981`，62 例通过、1309 例在筛选外，总用时 532 ms。两项 debug/no-run 准备分别为 8.54 s、24.46 s。 |
| A2TS `ts_ui_fragment` | 5 例通过，总用时 467 ms。新增 `checked_widget_view_and_child_property_follow_the_current_model` 单跑 143 ms，使用受检生成代码检验带载荷状态转换、两份模型独立和子组件读取更新后的受控属性。debug/no-run 增量准备 0.79 s。 |

保留首次失败。初版试用源级 `List.clone` 被前门具名拒绝，改用受检字段读取。首个 TreeView 对照错误地预期 `deep`，从原源核对目录 ID 后改为精确检查 `project`、`src`、`level-1` 至 `level-9` 和 `handbook`。新增永久回归的初版没有声明包导入，实际绑定图为空。测试按原页面的跨包用法声明导入并供应组件模块后通过。

Rust 测试以 `cargo test --locked --offline … --no-run` 准备，二进制直接以 `--test-threads=1` 开跑。原页诊断以本树 `ace-a2x-ts` debug 库准备，传 AutoOS `widgets-gallery/src/front` 绝对路径及 `filetree` 或 `treeview`，用 Node 实际执行本次生成的模块。准备不计入两个诊断的整命令，计时包含原源编译、A2TS 发射、Node 启动和退出。Iced 实窗、Vue 冷生成及真实浏览器、027 的真实文件与权限矩阵仍待完成。

材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-widget-runtime-js-view-filetree.txt` `7E340F3C164E1A9696CCA3108BFE4A3A04D9D5FEC12701BE03DCCD44F7261D34`
- `r2-widget-runtime-js-view-treeview.txt` `A2E53D97629426E9C9E29AA91031DCD9EAEB44895ADF2EDD4693EC16F3E4C561`
- `r2-widget-runtime-js-first-treeview.txt` `16422A753DF4137B5877B907F42C69AAD8E7C77D92C3FD8632BCD83DDE0A8825`
- `r2-widget-runtime-first-filetree.txt` `1A18C8F6BB2AD546368158C827E199891E4BF2C5D13F6107FB92BECE19A7CC4B`
- `r2-widget-runtime-fragment-case.txt` `EF207826B7F4E4521AB6219C6E9CA1FAFB2EAD2A05D5ABA97CE1FDB014560CD9`
- `r2-widget-runtime-fragment-all.txt` `2DF1E2FD8366798C03C0F12F31D4CEDB63C40A77F8DEEE7DA9EAA126359B8F29`
- `r2-widget-runtime-regression-status.txt` `101C0863737209EBA7E153A5BCB227D9F3DA540D3F09813CDE750F256C4363EB`
- `r2-widget-runtime-regression-av-status.txt` `6888A5D79FD1D8D760042FA3E8114418B0C11060CB705A285FF5B5C07AE0FEF0`
- `r2-widget-runtime-fixture-triage.txt` `03EEE8E1A7B9CF964803FC279F2BAE335A13AADF903FC3B7EBF36AB1D4CBC6D5`
- `r2-widget-runtime-emit.rs` `F9A84CCB0FEB18CB00BB9170D4A2C18A0B80BBDF0CB3CEFA35850359EA66F868`
- `r2-widget-runtime-model-oracle.js` `15A42E4137DCD69921CC833C3357411A14CAFB16A1B835A5551FBEABCE2CDF45`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 20 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: fa1daa2dc8f87f0b8736549bae6b6beca92999fa | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 Vue 生产包生成及两原树页浏览器回放 | outcome: partial | AutoLang `0a525158fdc5371c8ec96345d13613131388a07b` | AutoUI `f92545004356f0f0f652e8a16cfed52260107000` | AutoOS `0483c6902bfc17b47bc80e602d7c1b560662a3ac` | branch: `plan-114-dev` | next: T-02 对账原画廊路由入口，接 027 原文件与权限通路。

`ace_package_sfc` 现在为已绑定的普通 widget 页面生成 Vue SFC。Vue 实例编排受检初始化器、状态转换、受控属性及子组件消息，当前模型和视图一起发布。旧帧及卸载实例的点击闭包失效。A2TS 运行时按挂载隔离，在发布时回收未被当前模型与输入引用的文本，卸载清空该实例的文本账本。路由外壳及含路由命令或参数的复合 widget 页需现有路由接面继续承接。

| 验证 | 实际结果 |
|---|---|
| FileTree 原页 Vue 产品 | 直接传原 `filetree.at` 和 `official` 实际页面闭包到 `generate_package_sfc`，2607 ms。Chromium 播放默认 10 行、展开 614 行、原归档文件选中样式、收起 10 行，核对 badge `611` 与打开的目录图标。第二实例保持 10 行，已替换帧和卸载实例的真实旧回调无效，重开恢复 10 行。没有浏览器错误，含启动、回放和关闭总用时 1368 ms。首次探针误用较新 Lucide 类名，8717 ms 后在图标定位失败。依锁定 0.312.0 源核对实际 `lucide-folder-open-icon` 后通过全流程。 |
| TreeView 原页 Vue 产品 | 原 `treeview.at` 及实际组件闭包由同一 SFC 入口生成，781 ms。Chromium 核对 9、20、3、9 行与 2、12、0、2 的展开计数，在 `bottom.txt` 选中时观察父模型的 `selected: bottom` 与子组件选中样式。两套状态、卸载重开及旧回调拒绝通过，没有浏览器错误，总用时 999 ms。DOM 计数材料附加两个控制按钮，因此记录为 11、22、5、11。 |
| 冷构建 | 在新材料目录安装锁定的 Vue 3.5.35、Vite 5.4.21、Lucide 0.312.0 及既有 AutoUI 生成项目配置。`pnpm --dir … run build` 实际执行 `vue-tsc && vite build`，4609 ms，产出 302.50 kB JS 和 9.38 kB CSS。Playwright 1.64.1 驱动 Chromium 156.0.8078.4，载入实际 `dist`，同一页挂两份 SFC 产品。首次安装遗漏同一生成项目的 `pnpm-workspace.yaml`，锁文件校验拒绝。补齐既有项目配置后以 `--offline --frozen-lockfile` 安装成功。 |
| AutoUI | `plan114_widget_packages`、`plan980_package_sfc`、`plan124_routes` 各 7、11、2 例通过，执行分别 627、155、697 ms。debug/no-run 准备 124 s，沿上轮已验证安排在本次命令禁用 `sccache` 包装。复用已有测试观测现可生成复合组件及原路由错误。 |
| A2TS | `scoped_widget_runtime_releases_text_without_disturbing_other_mounts` 单跑 208 ms，核模型中共享文本寿命、未引用文本释放及另一挂载不受影响。原运行时合同 1 例和 UI fragments 5 例通过，分别 142、464 ms。debug/no-run 准备 3.51 s。 |
| 原画廊完整入口 | 直接对实际 `widgets-gallery/src/front/app.at` 及原 `pages/` 调用 `generate_routed_front_sfc`，`source.syntax: expected fn, found widget`，退出码 1。两页浏览器材料的装载壳仅用于实例对照，原 PAC 路由冷生成仍待贯通。 |

原文件与浏览器材料在 `D:/autostack/plan-artifacts/plan-114/`。Rust 目标先 `cargo test --locked --offline … --no-run`，再直接运行二进制 `--test-threads=1`。浏览器用 `python r2-widget-vue-browser.py filetree` 与 `… treeview`，各含独立本地回环服务的启停。027 实际文件、权限和失败恢复，以及 Windows Iced 实窗鼠标验收仍待完成。

- `r2-widget-vue-browser-filetree.txt` `5959DCA495507671A38ED0D82457B1997EA6727D0D773F6B9975C076F80E0D46`
- `r2-widget-vue-browser-treeview.txt` `A410C0ECCCCE06E88FD629155CBABF57FEAAEC44C7B2865ECFD8F914E65603F8`
- `r2-widget-vue-browser-first-filetree.txt` `8557B2F66B22B9CE39201023779F94D4216C2E2CDF6BDF908EC8735019B435D0`
- `r2-widget-vue-production-build.log` `F45B3E870AE0EE4F1EE9780EA13D5EC39798656B2858052C3A55042581C6540B`
- `r2-widget-vue-emit-filetree.txt` `C65EA3F73CC37F23A5CAF7287206CA29D88134ADBBD7639B32C9BA2A2DFFBD85`
- `r2-widget-vue-emit-treeview.txt` `9F7E7925AC359A5EA8D3C7929DEFD514E2C356E71A9ED9B6CD95F8A4835DFE1B`
- `r2-widget-vue-gallery-shell.txt` `CF8764EC87B358EFF1ADE12B873AD52C83467F2FA1C53F379C2DE42B1FE62454`
- `r2-widget-vue-ui-status.txt` `5B21E6A38CDCCDBE3FA2C1B76B8FFE5A9576F26B2E5CBF8F35FD0727BB4757D6`
- `r2-widget-vue-a2ts-status.txt` `3FED81DAD582812612917EC2057720C8875C59AA8082B56D899909EBFC1CCD1F`
- `r2-widget-vue-pnpm-install.log` `5CCCB9466D7EF182DF1BDF2AE366E114803E8566071B937173459EC7F3DFF76A`
- `r2-widget-vue-pnpm-install-retry.log` `17A619FA03D60A51643AEE62A3E799E3502CA31C7186DC14914E04E8085827FA`
- `r2-widget-vue-browser.py` `E94736F9A8ADEB4A571B44DC60D068C5E8F7625FC48A67A6819B2F6D649757D4`
- `widget-vue-sfc/pnpm-lock.yaml` `89214754957CF778416AFD6FF65FBAEE8B5AFA39AE6E8F888578A28A6B5A5D6B`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 21 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: f92545004356f0f0f652e8a16cfed52260107000 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原画廊路由标识和 027 真盘后端基线 | outcome: partial | AutoLang `8331f22571dfe5910c5fe5da513d9bc35c976093` | AutoUI `f0ac2e40527ea921fe0693eee94bc548070efa04` | AutoOS `0483c6902bfc17b47bc80e602d7c1b560662a3ac` | branch: `plan-114-dev` | next: T-02 保持原画廊壳语义接通路由，并在 027 的受检事件与文件服务入口接入会话授权。

AGE 路由保留文件名中的连字符。绑定器对连字符模块采用独立符号编码，`code-editor` 和 `code_editor` 继续指向各自文件。修改仅改变 AgeComponent 与 AgePackage 的语法身份，现行冻结测试内其他语法身份一致。

| 验证 | 结果 |
|---|---|
| 原画廊路由表 | 原 `app.at` 路由块分离探针投影出 70 条，`/code-editor`、`/filetree`、`/treeview` 分别指向原文件名。小对照原先只接受 `code_editor`，修改后两种文件名均被接受。带连字符与下划线的页面从同一生产包 SFC 入口生成两份真实页面，源函数 `seed` 在同一受检闭包拥有不同符号。新增用例单跑 94 ms。 |
| 完整画廊壳 | 原壳整体仍在包语法拒绝。分离对照确定旧 handler 中 `sidebarOpen = true` 的裸赋值会拒绝，点限定后需要显式消息声明。实际视图的 `outlet (memo: true)` 会单独拒绝。仅在材料副本中声明原消息、限定原赋值并将 outlet 简化后可投影全部 70 路由。此结果用于定位后续挂载与 memo 接面。组合语法探针总用时 202 ms。 |
| 原树页产品 | 用本轮 AutoUI 重新生成原 FileTree 与 TreeView，耗时 2642、767 ms，SFC 哈希与上轮 Chromium 所用版本逐字节一致。复用上轮两个浏览器回放证据。 |
| 027 原后端 | 从实际 `027-file-manager/src/back/api.at` 经 `back_proxy` 生产 VM 会话，在回环连接验证 `fs_home`、`fs_list`。`fs_home` 返回实验指定目录，`fs_list` 进入 `nested` 读到 `inside114.txt`，文件大小与独立磁盘元数据一致。缺失目录返回 `ok:false`，分离的另一实验目录当前也会被列出。移除并 join 会话后旧端点请求返回 404。退出码 0，含进程启动和退出总用时 105 ms，模型修改和读写授权未接入。原前端仍在第 425 行以 `source.lex` 拒绝 `$event`。 |
| 定向回归 | AutoLang `plan981`、`plan1072_component_style` 共 276 例通过，579 例在筛选外，总用时 1030 ms，最终增量 debug/no-run 准备 2.37 s。AutoUI `plan124_routes`、`plan114_widget_packages`、`plan980_package_sfc` 分别 3、7、11 例通过，732、596、166 ms，debug/no-run 准备约 66 s。 |

保留首次对照失败。最早测试输入把 `outlet` 写成 `outlet {}`，修正后建立有效正控。027 首次确认用 JSON 纯文本匹配，误将实际 `{"ok": true}` 当作不匹配，改为按 JSON 结构和值核对。两项目标语法身份因本轮改动触发既有冻结用例后，按实测值更新两处冻结常量并复跑定向回归。

Rust 目标经 `cargo test --locked --offline … --no-run` 准备，测试二进制以 `--test-threads=1` 直接运行。027 探针仅使用本计划创建的两个目录，后端原源、隔离目录及用户目录环境均为显式传参。材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-gallery-route-before-control.txt` `96EE0247BB73A17CBC6D73FEE38D0ABA85E2DA69815F55D44994C506E3BB587A`
- `r2-gallery-route-final-probe.txt` `4BF1BD006EA9A38F747EBF230516FD61211FB265D364FA9139A5E0727498AA18`
- `r2-gallery-route-core-status.txt` `8332738C9A293FE57F2DB508B91DAEB2350ADB4A0C421FCD367C7D5F30E61121`
- `r2-gallery-route-ui-status.txt` `034C7D8F840AC083B0502EEF7975BFC80CB006F165563BA7175907C69FD47545`
- `r2-gallery-route-leaf-identity.txt` `55C0A53785E150DA77FAEE995762AA79790F769282319BDAE6625B874C8B6BCD`
- `r2-gallery-grammar-final-shape-red.txt` `2304040693243943C55627C86596033EBDAE36C04659767B19DFDB1E6A6910B4`
- `r2-file-manager-original-backend-confirmed.txt` `B8578E7948886C8A8D36904FB5A9A0A3F7B901F725CAAEF5D353C1FBB4703232`
- `r2-file-manager-original-backend.txt` `2EF44B470283F104C8FFBD523CBCE213CEE81DE1A7D4C2A7706B5DF6F632880B`
- `r2-file-manager-original-backend.rs` `EB8388881142BE3F81D7F12BD178055885BEA55997C7942D82EC5585510F7DF0`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 22 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: f0ac2e40527ea921fe0693eee94bc548070efa04 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原画廊外壳进入受检路由入口 | outcome: partial | AutoLang `cc393e7a38d26d2bd01205606dc42359ad75ec9a` | AutoUI `368751647288c95c7d09149ac59aa639bcb14e59` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 接通外壳路由挂载与 SettingsPanel 回调。

原画廊 `app.at` 显式声明已有消息，保留原模型转换、路径表和 `outlet (memo: true)`。AGE 两种语法面投影带属性的 `outlet`，在语法树中保留布尔 `memo` 和后继节点。受检视图按布尔条件生成 `drawer.open` 的真实属性值。两种冻结语法身份按实测变化更新。

| 验证 | 结果 |
|---|---|
| 原画廊前门 | 对 AutoOS 实际 `app.at` 投影全部 70 条，含 `/code-editor`、`/filetree`、`/treeview`，22 ms。旧 UI 编译器 `parse_validated_ui` 对变更前后两份源均通过，分别为 6、4 ms，均保留 `memo: Some(true)`。本验收覆盖源语义检查。 |
| 原画廊完整 SFC | 语法通过后，生产 `generate_routed_front_sfc` 首次在两处 `drawer.open` 布尔值上报 `source.fstr-arg`。修复受检视图属性后，原入口进入 `source.age.sfc-widget-route`，由当前复合外壳挂载限制拒绝。另用 `compile_age_package_widget_runtime` 投原 `app.at` 及 `official/settings.at`，59 ms 定位 `source.age.widget-runtime-handler`；SettingsPanel 是无模型的显式零参数转发，须继续承接实际回调。 |
| 树页定向回归 | 原 FileTree 和 TreeView 独立经同版本重新生成，2547、762 ms，均与已经运行的浏览器 SFC 逐字节相同。Core `plan981`、`plan1072_component_style` 与 `plan104_bare_tags` 共 280 例通过，分别 0.95、0.14、0.03 s。AutoUI 路由、widget 和 SFC 定向组分别 3、8、11 例通过，0.74、0.59、0.21 s。新增 AV 用例用同一包的两实例执行已受检视图，观察初始 `open=false` 和收到 `Open` 后的 `open=true`。 |
| 027 调用入口 | 原 Vue 读盘走 `back_proxy` 的会话分发和实际 `fs_list`；`NavTo` 的 AV 原体及 `CommitNew`、`CommitRename`、`CtxPaste`、`ExecuteDelete` 的原生文件操作在前端处理器。授权待接到实际 I/O 边界。已有 Windows 工具目录通过平台句柄拒绝重解析点，但当前实现只持有自己的临时目录；027 访问用户授权目录还需要独立能力。原前端、写操作、撤权与恢复，以及实窗验收仍待执行。 |

保留语法冻结红灯和原入口的两层拒因。增量 debug/no-run 准备后，测试二进制以 `--test-threads=1` 运行，AutoUI 两次主准备为 93.00、52.33 s。材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-gallery-memo-core-status.txt` `FD6B91CD9FBC76FFBB70C193210D3F32340A62863AA558CFFC3AC1A0D1FAE79B`
- `r2-gallery-memo-admission.txt` `9AEB913BFF26B3944ACDF79B6157B54D9BC0E39EA169CC5153DA1C57F88A8752`
- `r2-gallery-memo-legacy.txt` `A0CB47849283E785CFDD007C08BE3DFD05DEDE74B96B39EC0D5A3E8DB59BAFC3`
- `r2-widget-vue-gallery-memo.txt` `1544A8246D5EBEC9A664BDFB3A27A4C505EE6010AF019E96EA4A406DAE7C6A4A`
- `r2-widget-vue-gallery-drawer.txt` `96BA8015C8E9FC03FDC96DE134A2F086908B2BC956ED49C8F1179A856B0B5104`
- `r2-gallery-shell-runtime.txt` `E1F4BD0CE271CB7514A75E56B297735895244C1794475E36A2A6F7068DCED176`
- `r2-gallery-drawer-ui-status.txt` `63E77EDE8B75AD07FBA1219A6049FDC9375C41044484E2094DE2141E63DA1870`
- `r2-gallery-drawer-check.txt` `9C919BB98AF8EDACFB9628F67F9663DA52CE4FB498B9F4942DEFE0AD020E885A`
- `r2-gallery-drawer-leaf-identity.txt` `15EF25E81963EBE7EF3BBEAE49A5D39DB68FC74C0117B2D650F197176F7E5297`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 23 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 368751647288c95c7d09149ac59aa639bcb14e59 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原画廊两条目标路由完成 A2Vue 冷构建与浏览器回放 | outcome: partial | AutoLang `668abc15d618b189052d2785f7ba7e21dd45b694` | AutoUI `4801091d2750878758023a19a18b4279fe453b66` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 AV 原路由装载与 027 受检文件会话。

受检包记录整段为单一零参数通道调用的显式处理器，按每个调用点绑定实际父消息。消息声明、处理器和目标的零参数约束均在包内检查。AV 与 A2Vue 共用精确通道映射，Vue Router 按路径持有 `outlet` 子实例。严格路由生成按已绑定组件闭包交付，同时保留逐页面诊断。CLI 的已求值 PAC 请求使用同一 npm 依赖解析规则，将 `file:` 路径按 PAC 所在目录定位；路由脚手架允许 JavaScript SFC，仍检查 TS 入口。

| 验证 | 结果 |
|---|---|
| 原始输入及冷生成 | 工作树原 `pac.at` 严格 `auto build --render vue --gen-only` 首次完整通过。最终在材料目录把 AutoOS 树 `8f660239a90d7fed4bad754b7c4b658e3e8bec3a` 原样导出，清空输出目标后经同一命令生成 **70** 条路由和 90 个文件，退出 0，56483 ms。26 条其他画廊路由仍显示具名页面诊断，含默认 `/`；`/filetree`、`/treeview` 均挂载原受检产品。工作树运行生成的已跟踪 `.auto/ui-cache.json` 已按开工状态恢复。 |
| 依赖及实际构建 | 固定 AutoDown `45b642dadcbd4fb71a8c7e6f09435faf7a65f5` 的 Engine 源树 `9e38490ff67e82c4e0be183cd9fbb89c00bca6c5` 在材料目录独立 `pnpm` 构建，7780 ms，产物戳 `237505cb8a84d56b0b4b4ad417bd7962ac22c5f113a890ed1e9cfb0030808b59`。工作区旧 dist 的戳不符并留为对照。严格原画廊的 `pnpm install --offline --frozen-lockfile` 为 3778 ms；`pnpm run build` 实际执行 `vue-tsc && vite build`，退出 0，6888 ms。版本为 Vue 3.5.35、Vue Router 4.6.4、Vite 5.4.21、vue-tsc 2.2.12、Lucide 0.312.0、Engine 0.5.0。 |
| 原项目浏览器回放 | Chromium 156.0.8078.4 通过原侧栏进入 `/filetree`、`/treeview`，再交替回访。FileTree 行数 10→614→614→10→614→10，核对 `611`、原归档文件选中及收起隐藏，退出后保存的旧回调未改变新实例。TreeView 展开数 2→12→0→2，选择 `bottom.txt` 后回传 `bottom`，切回初始选择恢复为空。设置面板操作覆盖亮色选中、Coral 选中及关闭，搜索抽屉打开和关闭均保留页状态。浏览器错误 0，含启动、导航与结束 2030 ms。主题面板验证了受检状态与样式分支，壳的全局主题 CSS 仍待对账。 |
| 定向回归 | Core 分组 12、28、4 例通过，分别 0.26、0.17、0.03 s。实际原壳与 SettingsPanel 的运行时闭包 527 ms 生成两个受检组件。AutoUI 路由、widget、SFC 分别 3、9、11 例通过，0.72、0.66、0.15 s。新增的零参转发用例单跑 0.26 s，实际 AV 帧和两个站点分别将同一子消息派给准确父消息，错配载荷被拒。`plan124_ace_vue_routes` 经生产请求验证冷/增量路由产物、排除不在闭包的组件、有作用域 PAC npm 依赖和 TS 配置，1 例通过，0.99 s。 |
| 剩余 | AV `NodeDesktopBridge::load` 仍需按原 front 路径取齐页面单元和各页面的组件依赖；其受检 `package_frames` 渲染分支目前直接返回，原布局中的 outlet 需承载活跃页。两页原 Iced 窗口交互、027 的 `$event`、会话授权与真盘文件操作、独立复核及交付归档仍待执行。 |

冷构建依次定位原独立组件扫描命中 `AreaChart` 的 ACE 缺席、双引号实体在 `togglegroup` 的 Vue 类型检查错误、空标签按钮的浏览器定位问题、PAC `file:` 被当作包名，以及官方脚手架的 `TS7016`。后续生产入口实际重跑成功。首次离线安装缺 `@babel/parser` 包，由官方 npm registry 补齐后冻结 lock，最终离线重装通过。Rust 仅做受影响 debug 目标准备与单进程回放，主准备分别为 142、44.25、33.79 s。CLI 二进制 SHA256 `EDDB2CCE50FD8AFE08CD26959C2FE52E090EB833B5C8B9A3F27538B28D947FCF`，最终 `auto-ui-tool.exe` 为 `E713A1D4075FB362B5DCDF96D4AB0F0EA225F8E755BF1F7C3698DB7584F6CEEB`。

材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-gallery-final-production-cli.txt` `6C276EC2FB011E7709B771E57D2487DF5A2448B2617628DE90576BB985B826AE`
- `r2-gallery-final-production-install.txt` `6741849BB788564937DDCFEA0342C8FF88098466EF418AD9183918C5B7F51B1C`
- `r2-gallery-final-production-build.txt` `82ACC5948DF6DF39C819BF039AA0749ADFBF5C10CB3B18802E631D30554AE16B`
- `r2-gallery-final-production-browser.txt` `DFBF05B8E56514B4A37268817332A4CA1E3E9F6A0E91BC4CE492B9A565491EA2`
- `r2-gallery-routed-browser.py` `F0D1607E37D8BA564AF1D165DB945DDD543F112E29A6928109A22757AD3833FC`
- `r2-gallery-engine-build.txt` `96C3946EB553AFA07E3EBBB43CFE55A4D670FA6DD5D481B80344D9BA33C8E554`
- `r2-gallery-callback-core.txt` `D6FAE22AD56A002A51C7650E9D10C9D0F55CC645839C638683FAB03882E2143D`
- `r2-gallery-routed-final-ui.txt` `6ECA382217DBC40BE3022961A7E8D174C564B544854D162AEA43E188BF037750`
- `r2-gallery-tsconfig-tool-test.txt` `0F9CAD254797CBAF16C85429746743694C55AF2D6BC1A60148F589D4626C19C9`
- `r2-gallery-stateless-callback.txt` `48D79B804E840878583EB6D90C12067F04F4BD05C838FA5FE6F34C942E8F2298`
- 最终 `src/App.vue` `87444B0E719AB245A67B276C6A05770D8C050BDDB76F2B57B55E712EC235FB7D`，`src/pages/filetree.vue` `C2BB3486F0C89B5493F4AD92D795BD2DF6E27131730A290358852311BE3BA843`，`src/pages/treeview.vue` `66599C416471B61FA210B0138D3465DF32E9E3BAD0D85B1227FFD1C135DEF1D0`，`src/router/index.ts` `A9FC68F87B6E524BDCBC661D2FFEB415DF451E78AF2E2E1A5FC8DCADE42F55D5`，`pnpm-lock.yaml` `D5823ED73ED826862E724DEEBCAEA323F358CE2FFEB36EE97F55F84AB0B2CF2A`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 24 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 4801091d2750878758023a19a18b4279fe453b66 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原画廊经 AV 桌面外壳装载两条原路由 | outcome: partial | AutoUI `9a916217a261cf14f3ec90d91255b2545cc2b3a5` | AutoLang `668abc15d618b189052d2785f7ba7e21dd45b694` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 027 受检文件会话及原前端事件。

`NodeDesktopBridge::load` 按原 `front/app.at` 发现页面模块，从每个页面的文件路径绑定其实际组件闭包。待挂载页面执行语法与执行资格检查，错误留在该路线。受检外壳从当前帧的可见链接产生修订绑定的宿主导航事件，原 `outlet` 位渲染当前页面。页面返回的事件仍属于自己的会话；退出释放页面并拒绝旧修订或旧会话事件。受检路径由包挂载执行一次 `Init`。

| 验证 | 结果 |
|---|---|
| 原 FileTree 桌面宿主 | 直接读取 AutoOS 原 `widgets-gallery/src/front/app.at`，使用 `AceNodeDesktopWindow.view()` 提供的原侧栏和树控件消息进入 `/filetree`。观察 badge `611`，展开并选中 `614-treeview-filetree.md`，收起后隐藏原文件；过期修订及切换页面后旧会话各拒绝一次。首个完整路线回放 **11136 ms**。延迟检查未访问页面后最终 **9354 ms**，加载 1140 ms，壳打开 936 ms，导航 2797 ms，三个页内动作 1356、1305、912 ms，最后导航 TreeView 830 ms。FileTree 原叶完整重开沿用本计划第 19 轮桌面宿主材料。 |
| 原 TreeView 桌面宿主 | 从同一原外壳进入 `/treeview`，展开后见 `bottom.txt`，选中派发 `TvSelect` 并核 `tvSelected=bottom`，收起后隐藏，切换再回访获得新会话。旧修订和关闭页各拒绝一次。最终全程 **8105 ms**，首轮 8222 ms。 |
| 回归 | `plan124_routes`、`plan114_widget_packages`、`plan980_package_sfc` 分别 4、9、11 例通过，1.29、1.05、0.37 s。新增的路径与视图用例以实际临时页面路径核嵌套目录、选定栏、outlet 前后位置、`Init` 一次、返回重开和旧点击，单跑 **0.39 s**。`cargo test --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge … --no-run` 两次准备约 98、63 s；执行使用 `--test-threads=1`，宿主依赖已准备。原外壳及两个目标路由重新生成的三个 Vue SFC 与第 24 轮实际浏览器回放产品逐字节相同。 |
| Windows Iced 实窗 | 原源启动创建唯一 `PLAN-114 Widgets Gallery` 窗口。平台状态只读接口报告 Default，但 `computer-use` 目标激活失败，刷新后截图显示 Windows 锁屏。未发送鼠标输入；停止本计划验证进程，物理点击和自然关闭未验收。已告知用户可解锁后继续，027 可独立推进。 |
| 027 剩余 | AutoLang 已确认 PLAN-973 受检 AV 后端带目录打开、读写、删除和替换调用；原 027 Web 后端仍经 VM `file_io` 执行全局文件操作。需要把根目录与操作授权接在真实调用层，建立请求与会话寿命、撤权和恢复验证。桌面与 Vue 端均仍有原 027 事件/文件验收。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-gallery-native-route-filetree-first.txt` `C573EBBE7952D8E40FFF754A43EC0F1C14F1A6125538974BAB5D6BE1CAF9D8C0`
- `r2-gallery-native-route-filetree-final.txt` `C4431B13A9388DD9DF11400AE46BBF4DD4F2BE77A860F4600599E6E4439D7B7F`
- `r2-gallery-native-route-treeview-final.txt` `E8F0237DD5027F4935490FA3B933E943E5BB35FC98B9F02DFAC07E1306BA7FE0`
- `r2-gallery-native-route-final-tests.txt` `4819F6A956CC30ACCDB5001A045D0EF89F53C0B9D1FF0C4B8011BE4F09212890`
- `r2-gallery-native-route-final-focus.txt` `4E2BD0BE3545111250BDF21929118352689F601C42D37C72191C1F98458DA259`
- `r2-gallery-native-window-observation.txt` `9C9736FE3B474182F494C47F62CB76E6BEA7AB1A0F1A043CB6C61097F608620C`
- `r2-gallery-native-route-vue-identity.txt` `6315E4DECED1D0562A5561A62D6045855122DE7CB47B620ACE2D17EC8A416CF8`
- `r2-gallery-native-route-probe.rs` `A1D6624479803637E3320451C90EAAD883B6572F93038A5748CDD6A74D58CC99`，对应验证二进制 `12EB16D704908C4DBF62394B02C13311BDD9600A05D9AFB626FDA381BE3C931F`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 25 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 9a916217a261cf14f3ec90d91255b2545cc2b3a5 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 后端目录读权限 | outcome: partial | AutoLang `f189dccfdcd456f6ed5ab186e62a3fea29730b40` | AutoUI `4c44e0c2fb0dc25415674b623e764e85f614a70b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 原前端事件、宿主接线与受检写操作。

AutoLang `FileSession` 由宿主持有目录句柄。VM 请求只在自己的处理线程绑定该会话，未授权和已撤权状态拒绝读取。目录导航限制在根句柄内，目录条目的元数据读取保留链接本身。AutoUI `RunningProxy::open_file_session` 分配随机的私有后端地址，销毁句柄移除 VM 会话。原 `src/back/api.at` 的三个入口使用会话读盘。未绑定会话的既有代理路由仍可加载原契约，读取请求返回拒绝。私有 URL 的 HTTP 响应及关闭后的 404 均禁缓存。

| 验证 | 结果 |
|---|---|
| 原 027 VM 后端 | 实际启动 `back_proxy`，经 TCP 命中原 `src/back/api.at`。确认授权前 Home 为空、Drives 为空、List 拒绝。授权后对根及子目录核真实中文文件名、绝对路径、长度、修改时间和空目录。未授权代理路由、另一个会话的根、父路径、Windows 目录联接及不存在目录均拒绝。撤权后拒绝，重新授权后恢复，显式关闭和销毁句柄后的旧地址均返回 404。替换会话地址变化且初始无权限。单进程用例 **2.94 s**。 |
| 普通代理回归 | `desktop_back_proxy_globals` 两例 **0.39 s**，本地与导入常量的求值、普通路由请求体和缺参语义通过。两测试目标的最终离线锁定准备 **20.35 s**。AutoLang `cargo check --locked --offline -p auto-lang-core --lib -j 2` **14.99 s**。 |
| 剩余 | 真实桌面和浏览器启动器尚未授予原 027 页面各自的目录和会话。原前端事件、AV 写权限和磁盘操作、撤权后的页面状态、晚到返回拒绝，以及真实双端验收继续按 V-07 执行。 |

最终命令为 `cargo test --locked --offline -j 2 -p auto-ui-runtime --test plan114_file_session --test desktop_back_proxy_globals -- --test-threads=1 --nocapture`。材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-file-session-final.log` SHA256 `DD8E9D7EDB78D935A29FCE61634521C9551382B4299FF5D4260974F00213B11C`，`r2-file-session-core-check.log` `06E2FEC1249EB8E882C938A1F1D984B8F5C940397152A9F27C7AEC4F1E305126`。原 `src/back/api.at` `322E6F6473AE2C5448E5E0AA919FD4DD484772705B15592A654FD31352A6D257`，`plan114_file_session.rs` `DB09D936C0158A157B03367EE52F7880AEAB5FCA6A9B53017CF6FEB3410564C3`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 26 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 4c44e0c2fb0dc25415674b623e764e85f614a70b | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检子组件事件转发 | outcome: partial | AutoLang `53f36cc3a4dae84ac98fa35b227a8ef497bf2b9d` | AutoUI `1bed133007de76fa5c8c0cfed6c49601af1b005f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 完成原 027 和 CollectionView 源的受检投影，并接桌面及浏览器文件会话。

AGE 包前门投影出 `.Selected($event)` 和子组件声明的命名事件。绑定在实际子组件引用处完成，核对声明、父消息、单个标量载荷类型及重复接线。宿主通过当前实例保留原载荷，将事件派发到核准的父消息。独立 `age-component` 和其他相邻 profile 的语法身份仍对应冻结值。ACE 工作区锁文件补入第 26 轮新增的目录句柄依赖，锁定构建复验通过。

| 验证 | 结果 |
|---|---|
| AutoLang `plan981` 与 `plan1072_component_style` | 276 例通过，580 例在筛选外。执行 **1.020 s**，外部计时 **1.109 s**。用新语法实际测得 `age-package` 身份 `06e60e27248addfee2b2e90cbee266b8f00c2900b8e3e76bca788ffd9b3901c5` 后重冻结，最终锁定 debug 准备 **7.25 s**。 |
| AutoUI 原生宿主和生成回归 | `plan114_widget_packages`、`plan124_routes`、`plan980_package_sfc` 分别 10、4、11 例通过，外部计时 **0.861、1.055、0.303 s**，最终 debug 准备 **78 s**。新增用例用已准备的 AV provider 将 `docs/计划` 派发到父消息并检查模型，错误类型、缺失父消息、未声明子事件、原生元素上的 `$event` 和重复绑定均在两端拒绝。新增用例单跑 **0.270 s**，外部计时 **0.447 s**。 |
| 真实浏览器，事件接线用例 | 从同一用例源生成 SFC **0.304 s**，Vite 构建 **3.010 s**。Chrome 点击后仅第一个实例收到原路径，关闭重开恢复初值，调用旧回调保持新实例初值。执行和清理 **3.084 s**，外部计时 **3.514 s**。首次观察定位器要求整元素精确文本，实际片段输出为相邻文本节点，改用同一 DOM 上的文本匹配后通过。 |
| 原 027 与 CollectionView | 原 `app.at` 越过 `$event` 词法拒绝后，在第 38 行的 Widget 回报 `source.syntax`，原 `collection_view.at` 第 7 行同类诊断。以原段落分离探针确认 027 启动计时器、视图和处理器各有语法阻塞，模型初值进入组件域后仍有未支持表达式。CollectionView 的带类型参数默认值、计算字段、视图和处理器仍需适配。子组件在处理器内发出命名消息的效果须继续接线。原前端接入、受检文件写操作和完整双端验收仍待交付。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-child-event-core-final.log` `D323CEDD1FBD8863E13192CCD981DEA04053AAAD76E225C4C58A686B53286E74`，`r2-child-event-ui-final.log` `66BF9A3B7199D32D9B8D35B6FF07E9A0214852EC60406D05E1347955AF48B142`，`r2-child-event-ui-focus.log` `564E931DE024A7EC5A165E4A9936B36A4E0CDDBFB764E071C36F7B8214808E7D`，`r2-child-event-browser.log` `94C44BB02E2BB08D06AEE4BC4E9D3C5D5857B19A82F3B359D59FC0ADC9633FEC`，`r2-child-event-browser.py` `BF2A30696FCF32EAA4D8F5D0F497B18B62FA1B324515D5917050D8AA88B6A884`，`r2-027-frontdoor.log` `9DD13F8A730A4D20642339205BF0B09FFEB4CD0D0DB14360D0E4B48584230C73`。生成 `App.vue` `F779CFED73305B30BB90C2AFCEBACC283D9D0EB0A76980D55D37C0CB69917E91`，构建 JS `A53470C6E9776CD609AE0730CF68F746FF1252B44184EF22EAAF0C51E9AF7F54`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 27 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1bed133007de76fa5c8c0cfed6c49601af1b005f | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 文件会话写权限与实际文件操作 | outcome: partial | AutoLang `9e105cd664002336456ac28be9e3a2a18f468690` | AutoUI `61c8b47c682ac691f09f9468037b7871f3067b94` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原 027 的消息、受检宿主操作和结果接入两端。

宿主持有的 `FileSession` 现在接受类型化的新建、复制、移动或重命名及删除请求。读取授权默认无写权限，宿主能逐项授予写权限。执行前核对当前授权、操作位和两端路径，使用目录句柄解析父级。复制在目标旁暂存，整项成功后以系统操作发布，同名目标保留原内容。非空目录删除由系统拒绝。Windows 首次用通用 Win32 接口传相对目录句柄返回 87，改为原生文件重命名信息接口后验证通过。 [Microsoft FILE_RENAME_INFORMATION](https://learn.microsoft.com/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)

| 验证 | 结果 |
|---|---|
| AutoLang `auto-lang-core --lib` 中的 `file_session::` | 3 例通过，1779 例在筛选外。核未授权、只读、分操作授权、越界源和目标、撤权、再授权和关闭。用真盘对证新建、递归复制、移动、重命名、删除、同名冲突、非空拒绝、目录联接及复制失败清理。外部计时 **0.833 s**，三例单跑 **0.046、0.161、0.024 s**。最终锁定 debug 准备 **63 s**。初次 `cargo check` 的 Windows rustc 退出 `STATUS_ACCESS_VIOLATION`，验证命令仅在此轮禁用增量后完成。 |
| 原 027 后端与普通代理 | `plan114_file_session` 1 例 **1.692 s**，`desktop_back_proxy_globals` 2 例 **0.863 s**，最终锁定 debug 准备 **88 s**。复核原后端的读权限、撤权、恢复和关闭。可写授权仅通过新宿主 Rust 接口授予。ACE 工作区相同依赖锁定解析通过。 |
| 剩余 | 原 AV 页面仍须以当前实例请求执行一次宿主操作，并将结果回送给受检状态更新。当前 AV 包渲染会重放历史消息，待接入的文件操作须由宿主在提交事件处执行。027 的前端投影、A2Vue 启动授权、两端原页面真盘操作与验收继续按 V-07。Linux 运行留专用环境。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-file-mutation-core-tests.log` SHA256 `BA5E8260DA5DA44010434A525702EA431582DA8FCDE94C3CEABAE334ED19CE1C`，`r2-file-mutation-core-pinned-build.log` `FDA84980564AA6B889EA1B6923769889D4D535BD8DE899B76E0A8B456C46DD3F`，`r2-file-mutation-proxy-tests.log` `5B87F8F23A8242E0FF9558C7DE60B1F6EDC7F91FDB61D257CA23DA2B2AE212EE`，首次相对重命名失败记录 `r2-file-mutation-fast-test.log` `49CFA6B207E2904FAD3461C183060FD672F8104C798C32E835A0CED2D5DA5AC6`。最终 `file_session/mutation.rs` `0CB3F3604B0772302CE63B32D8E366456D8AC747A0272B0DEAE898C8EEB23B8B`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 28 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 61c8b47c682ac691f09f9468037b7871f3067b94 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的 Vue 启动授权和浏览器会话 | outcome: partial | AutoLang `d01f4e3cb28138dede25aec7205264266f46ee28` | AutoUI `4c05d38e5194050f8a7ffe3e0a182f747f665ee5` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: work/T-02 原 027 的严格包生成入口、受检处理器与 AV 文件效果。

独立 Vue 启动器从 `AUTO_FILE_MANAGER_ROOT` 接收宿主指定的目录，保持其目录句柄，并在浏览器连接建立时派生独立只读会话。原 `api.at` 继续生成三条类型化 API，生成客户端把调用送到当前文档的私有会话。Vite 向本机同源请求附加启动凭证，凭证仅随子进程配置传递。页面关闭中断连接，代理销毁对应后端会话。启动器释放后撤销本次运行派生的权限。

| 验证 | 结果 |
|---|---|
| 原 027 的现有 Vue 启动链和 Chrome | 在字节相同的原项目副本上运行 `auto run --render vue --server vm --front-port <临时端口>`，本轮工具二进制经 `AUTO_UI_TOOL_BIN` 接入。真实页面显示 `visible114.txt`，原 `fs_list` 拒绝相邻目录。Chrome 在同一浏览器上下文打开两个页面，获得不同会话。关第一个页面后旧地址返回 404，第二个 `fs_home` 仍返回授权根。最终准备 **10.794 s**、浏览器执行 **6.640 s**。冷启动准备 **22.101 s**，首次用 2.8 s 等完整目录视图超时。复跑保留原视图断言，并用原类型化 API 独立对证目录数据。生成的 `file-browser.ts` 单独通过 `tsc --noEmit`。 |
| AutoUI 代理回归 | 新增 `browser_connection_owns_an_independent_read_grant` 1 例通过，外部计时 **0.939 s**。覆盖缺凭证、跨站、客户端提交根目录、真实列表、关闭单页及宿主关闭。既有 `original_file_manager_reads_only_from_the_live_host_grant` 1 例 **3.605 s**、`desktop_back_proxy_globals` 2 例 **1.950 s**。锁定 debug 测试准备 **311 s**，工具准备 **217 s**。 |
| 原 027 的 A2Vue 准入 | `AUTO_UI_ACE_VUE=1` 的原项目 `auto build --render vue --gen-only` 返回 1。现有独立 App 工具入口走 `ace_vue_app_sfcs`，在 `.BootTick`、`.NavTo`、文件操作及其他处理器的 `ace.vue-lane-unsupported` 诊断停下。浏览器证据绑定现有 Vue 生成链。严格包入口及源适配、AV 文件效果和最终双端验收仍待执行。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。`r2-file-browser-runtime-tests.log` SHA256 `D8138AB97350003C0FFCEA2305D63B956CBA5AA9C47B58C224E4D1AD8F106809`，`r2-file-browser-strict.log` `A33CDDB325C459D1B95587C306019248F6205364550467A6006F01CB3B61AAE8`，`r2-file-browser-original-browser.log` `472E31FE41874C2EA56D83FEC3D6B90A323249E26EB8893B5AD20DB97BA05295`，`r2-file-browser-original.py` `A0D34C5F090DA37966D084F448FEA726C9C954312C18CB4BCA26B480B3C05333`，本轮 `auto-ui-tool.exe` `BA24E575C1573BD5E458A6255EB1B6C3B170B9D56C8341CA2A2EBD130CEF3C62`，原 027 `src/front/app.at` `8EAD8E0FD3610F461A830F9E1B755950F4E3D1467A4AAFA206461B204952E775`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 29 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 4c05d38e5194050f8a7ffe3e0a182f747f665ee5 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 独立应用的 A2Vue 受检包入口 | outcome: partial | AutoLang `89cf2a994dc8c4d76f14014bd04884cd5b27498d` | AutoUI `5c19098229dcf3ec5a02bddf3338bfa90acbb388` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 原 027 启动计时器、受检前端和文件效果。

`AUTO_UI_ACE_VUE=1` 下显式导入 Widget 包的独立应用，初次生成和增量编译均采用包闭包。包名解析接受声明中可用的下划线拼写，例如 `collection_view` 对应 `collection-view`，仍拒绝其他包身份。AutoLang 受检 Widget 参数允许类型匹配的整数默认值，覆盖原 CollectionView 的 `revision: int = 0`。

| 验证 | 结果 |
|---|---|
| 包入口、原路由及相邻通道 | `plan124_ace_vue_routes` 2 例通过，外部计时 **2.479 s**。核初次输出、子包修改刷新、无变化跳过写盘，以及错误包身份返回具名拒绝并保留上次输出。既有路由用例仍通过。`plan980_ace_vue_tool_wiring` 1 例通过，外部计时 **5.367 s**。首次发现该测试从 002 工作目录复制了本地既有生成文件，改为只复制 `pac.at` 和 `src` 后复验。最终锁定 debug 准备 **24.944 s**。 |
| 独立包生产生成和 Chrome | `auto build --render vue --gen-only` **2.177 s**，Vite 构建 **3.046 s**。Chrome 点击子组件，把原样载荷 `docs/计划` 送给父组件，另一页和关闭后新开的页面均保留各自初值。执行及清理 **2.040 s**。该用例是独立包夹具。 |
| AutoLang 相邻域 | `plan981` 与 `plan1072_component_style` 276 例通过，外部计时 **1.100 s**，离线锁定 debug 准备 **9.83 s**。 |
| 原 027 的 A2Vue 准入 | 原 `app.at` SHA256 `8EAD8E0FD3610F461A830F9E1B755950F4E3D1467A4AAFA206461B204952E775`。实际 CLI 已进入 `ace.package.app`，受检前门以 `source.syntax` 拒绝，返回 1，**2.094 s**。四个隔离探针 **0.117 s**，无计时器和普通计时器通过，`when: true` 与原形 `when: .booting` 均拒绝。原页面的计时器运行语义、其余前端投影、宿主文件效果与双端真盘验收仍待执行。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。锁定准备命令为 `cargo test --locked --offline -j 2 -p auto-ui-tools --features ace-vue --test plan124_ace_vue_routes --test plan980_ace_vue_tool_wiring --no-run`，执行使用 `--test-threads=1`。日志 SHA256 如下。

- `r2-standalone-package-vue-tests-final.log` `073BC27AD8F44E83885E893054920831EF793F032DC3845570C78D7160CBE447`
- `r2-standalone-package-single-file-regression-final.log` `BF24F56027E07EA59A09F3D9700B05BC2342E0FC31528D80F8EF1BB4DA070002`
- `r2-standalone-package-cli.log` `CAA6871C95F4D55DA3C49C7CEE07DC61357C1B22A9FA7AAFAD845AA1B8CAF219`
- `r2-standalone-package-browser.log` `FDA71CD79C6F7C51F05DCA64C0AF71BE6BAD11841E72F5521E32F812C4D64E8E`
- `r2-standalone-package-core-tests.log` `F2146DD49C3026198EBC0FE740742E760E5B98FD3C6F307A5080AF40D8A401E7`
- `r2-standalone-package-original-027.log` `58E6A56BF06A91F590031E3C1707A66E5B1D47E2178C20A819A1A3EABED8ABC3`
- `r2-standalone-package-timer-probe.log` `5DD41675265E3932A29B225E563E129BAA2C4A194454F9E314D9242D9C6C5768`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 30 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 5c19098229dcf3ec5a02bddf3338bfa90acbb388 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 条件计时器 | outcome: partial | AutoLang `9edbbb8c0ec9d25f7844f76a1330b569b45ae6d3` | AutoUI `8c0ee4b4850950fbd0eaa0ce6722df6ab689eea1` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 原 027 的记录列表、视图及处理器，继续接受检文件效果。

AGE 包前门保留 `when` 的完整表达式，核验布尔类型、毫秒周期以及无载荷消息处理器。AV 桌面桥在根页面实例打开后设置到期时间，以当前受检模型求值并经现有消息更新帧。A2Vue 在组件挂载后按实例设置调度，每一拍读取当前模型，卸载后清理计时器并丢弃旧回调。桌面当前支持本计划需要的根组件计时器。嵌套组件计时器在桌面具名拒绝。

| 验证 | 结果 |
|---|---|
| 原源探针 | 保留 027 原样声明 `BootTick (every_ms: 250, when: .booting)` 的分段探针通过，原 027 与 CollectionView 全文和分段探针 **0.163 s**。独立模型探针保留原行号，在 027 第 152 行定位 `collection_columns` 的首个记录字面量。分段验证还确认原 027 的 view、on 和 CollectionView 的 computed、view、on 有独立阻塞。严格生产 CLI 处理原 027 仍以 `source.syntax` 拒绝，返回 1，**3.371 s**。 |
| AV 桌面宿主 | 原计时器拼写的夹具用已准备的 UI980 provider 打开两个实例。核到期前空闲、各派发一次、只重启目标实例、关闭后空闲及重开恢复初值。周期为零、消息未声明和条件非布尔均拒绝。定向执行 **1.939 s**，完整 `plan114_widget_packages` 11 例 **2.158 s**。通过实际桌面桥的周期入口验证，物理 Iced 窗口留整体走查。 |
| A2Vue 生产生成与 Chrome | 同计时器夹具以 `auto build --render vue --gen-only` 生成，**2.303 s**，Vite 编译 **6.169 s**。两个 Chrome 页面各自派发一次，条件翻为假后不再派发，单页重启只改变该页。直接卸载 Vue App 后活跃计时器归零，手动调用旧回调无效果，重开从初值运行。执行及清理 **5.962 s**。 |
| 相邻回归 | AutoLang `plan981` 与 `plan1072_component_style` 277 例 **1.171 s**，离线锁定 debug 准备 **15.095 s**。实测重冻结 `age-package` 身份为 `acd5ec72137e2338f104a04663f857deaa8dd4244427f900291f629c89dfd3de`，邻接 profile 逐项匹配。AutoUI `plan124_routes`、`plan980_package_sfc` 和 `plan124_ace_vue_routes` 分别 4、11、2 例，**1.753、0.452、1.910 s**。最终锁定 debug 准备 **17.257 s**，首次准备发现新增测试源码位置和借用寿命问题，调整测试后复验通过。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。AutoUI 最终准备命令为 `cargo test --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --test plan124_routes --test plan980_package_sfc -p auto-ui-tools --features ace-vue --test plan124_ace_vue_routes --no-run`，执行使用 `--test-threads=1`。日志 SHA256 如下。

- `r2-timer-core-tests.log` `AB4D3B55D1C787C8D9257F7DAB09532FC8837D8CFC033421D92E2E931398CAFE`
- `r2-timer-027-frontdoor.log` `9546CBC81A6D02B40D0C6AA9B03AB8D3A9ED3EB843CEE76B5DBBAF34E1B717DD`
- `r2-timer-original-model.log` `ECC0BCBF7B11C968E3E8CEB851F2E6F71B77661E7E92507F2B9E5F6C519BAE6B`
- `r2-boot-timer-native-focus.log` `92A4BF3DA56D718DA51F009077A299A6844BB21B6E57BB0B192411D13B54F8C1`
- `r2-boot-timer-widget-package-tests.log` `ED0BB12E08FCE0E214FFB025F1BD56EBDA3AAAEF12CC6A1BD8B259EF8F67D514`
- `r2-boot-timer-cli.log` `B0FB1487271AF8F068D92B63088BE119BE5EBE71CAEDFCAED282A35611B494F8`
- `r2-boot-timer-browser.log` `5A4CA7AB5AE074465BEF500B835F11C0DDC139202300087855F6F631733BBBC3`
- `r2-boot-timer-regression.log` `10C07F59594BB3944233B1444362E33005E46E128F1F77E28FB2E60046EEB4EC`
- `r2-boot-timer-original-cli.log` `30FC743AE83C2A57F49DF30DDA5F9E19355DEA710CA71B12308C77D952DA44FC`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 31 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 8c0ee4b4850950fbd0eaa0ce6722df6ab689eea1 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 受检记录列表与完整视图投影 | outcome: partial | AutoLang `939f0dcfdaa466a91cd7c8e7aaa4890d131fb49d` | AutoUI `03a741bf6153f79254b8857c44752defe714fdc7` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 原处理器控制流、类型化效果与 CollectionView 的两端接线。

对原 027 的历史、面包屑、快捷目录、盘符、列和文件行补显式列表类型。列与行的原始列表字面量逐字节一致。面包屑内部字段从 AGE 已有的 `sep` 视图词改为 `separator`，同步更新原 helper 与两处读取。包闭包纳入页面按目录名或声明包名显式导入的普通模块。AV 一次降低同一受检包的定义，继续逐字段通过原有执行与验证链读取标量。

| 验证 | 结果 |
|---|---|
| 原 027 源 | 原页完整模型、消息、设计、计时器和整个 view 一起通过受检投影，**0.114 s**。使用原消息声明逐段检查，7 个直接视图片段均通过，46 个处理器片段中 23 个仍在语法前门拒绝，含 `BootTick`、`NavTo` 及操作处理器，探针合计 **1.043 s**。全页严格生成仍返回 1，**2.012 s**。当前 `app.at` SHA256 `3864DC97DEFAF4C33D9FB81B4358F8C01008EEFE00A81B409DF4EFDB2C1989E4`。 |
| AV 桌面宿主 | 夹具取原 117 行 model、两处生产记录定义及原计时器形态，通过实际包入口启动。两个实例都读出原四列标签、一个初始行及空历史。清空一个实例只改变该实例，关闭重开恢复两份初值。两次旧执行外部耗时 **10.442、10.176 s**，超过新增用例时限。保留同一程序、输入及断言，共用降低结果后最终 **8.638 s**，程序内 **8.456 s**。最终独立探针链接准备 **7.855 s**。 |
| A2Vue 与 Chrome | 同模型夹具实际 CLI 初次生成 **3.686 s**，Vite 准备 **1.312 s**。Chrome 验证原列标签顺序、单页清空及重开恢复，含清理 **1.304 s**。最终 CLI **3.603 s**，SFC 与构建浏览器产品用的源逐字节一致。 |
| 原 027 现有 Vue/VM 兼容 | 原项目生成成功，首次冷生成 **11.514 s**。真实浏览器从原地址栏进入隔离根下的 `inside114`，见真实 `nested114.txt`、路径按钮和 `›`。复核根外拒绝、两个文档独立会话和关闭后旧地址 404。现有启动链准备 **7.001 s**，Chrome 回放 **3.909 s**。这些结果使用现有 Vue/VM 轨。 |
| 定向回归 | AutoLang `plan981_age_package_av` 15 例通过，**0.617 s**，锁定 debug 准备 **53.984 s**。AutoUI `plan114_widget_packages`、`plan124_routes`、`plan980_package_sfc`、`plan124_ace_vue_routes` 分别 11、4、11、2 例，**2.198、0.993、0.184、1.258 s**，锁定 debug 准备 **83.547 s**。扩展的包闭包用例单跑 **0.275 s**，同一目录的组件标签依赖、显式类型导入和无关模块隔离同时通过。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。AutoUI 使用 `cargo test --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --test plan124_routes --test plan980_package_sfc -p auto-ui-tools --features ace-vue --test plan124_ace_vue_routes --no-run` 准备。AutoLang 用 `cargo test --locked --offline -j 2 -p ace-av --test conformance --no-run` 准备。输出中所列测试二进制均以 `--test-threads=1` 执行。日志 SHA256 如下。

- `r2-file-model-literal-identity.log` `AB3064380B6C255060ECEF8274C78CF92F17928893C09376E80C9251850A9BFF`
- `r2-file-model-original-full-view.log` `26A84411E579ADC13704DA92CB960F12A58307CE2F1A36E05928CD146AF6569A`
- `r2-file-model-section-probe.log` `16881D135ADEEA5AEF9388A999F66DC8860E6B09DFF7AB3DB50F438BE772BCEC`
- `r2-file-model-native.log` `1C1F5A23FD937EE35EC413D06B820C8542AA1D213517B7194AB02407553D9DB0`，单独复跑 `r2-file-model-native-isolated.log` `7E3B500A5FAB623FC7747C5EE97498B76D413C0C1C0278B3026A71202E8A0B90`
- `r2-file-model-native-final.log` `34E5CE585DB393446807823C3AF1E781CA8C44CB16B255B9E7664F075F1D6EA7`
- `r2-file-model-cli-verified.log` `63035AECA0A588B2B3136B45A8BE624389A3CE2E4DD3C1B8EE61E2936B20BE64`
- `r2-file-model-browser.log` `D53249417985B26AE6B4A80FE0406C6AB1F4A2A344BBAEFDAF59AB19B476E9BB`
- `r2-file-model-original-breadcrumb-browser.log` `64CD4B2761180E3A9036DED0DD6D61836746E50EA9D909E093F38C5C350FB1F7`
- `r2-file-model-original-strict-final.log` `793B927976661E896CE33023212468A3B9A291A66D56A6D12F87530869F84408`
- `r2-file-model-av-tests.log` `735AD74C394818602818D9C06694EC7EB8C444CA98D8291DDDF70098DAD65CF8`
- `r2-file-model-regressions-final.log` `33E1086A7EABF3A6FC65BB8EFE2877AC42DAEECB32F57D38EC3BCF6673F48BEB`
- `r2-file-model-direct-import-focus.log` `850439092B1AEC9B2CCE0DB428C2F793D0745393A675ABADF0BFA4159CD5D81D`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 32 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 03a741bf6153f79254b8857c44752defe714fdc7 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检处理器循环和原 027 选中 | outcome: partial | AutoLang `bb0aaf2d0fde92cad64a3bd378c959bcdda9c14a` | AutoUI `84e1dcfb1a7d859cec9311c9b43a8b76f2cf3a99` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 分支内的类型化消息派发、文件效果与原 CollectionView 接线。

受检 Widget 包处理器保留作者的形参类型，与消息载荷逐位核对。`var`、局部赋值和 `while` 进入同一份受检程序，局部变量在嵌套分支按词法作用域解析，点前缀写入仍指向模型。A2Vue 的 AGE 包接入已实现的受检记录复制。处理器把记录字段的文本保存到模型时，模型持有独立副本。

| 验证 | 结果 |
|---|---|
| 原 027 处理器投影 | 仍使用相同的 46 个原处理器片段，语法拒绝从 23 个降到 20 个，新增通过 `SelectItem`、`CollectionSelection`、`CollectionAction` 的投影。整组探针 **1.412 s**。`GoBack` 的原分支调用 `.NavTo(prev_path)` 仍被语法拒绝，去掉这行的诊断对照可投影，二者同跑 **0.134 s**。`CollectionSelection` 和 `CollectionAction` 的内层消息派发尚待验证。原页仍未严格生成，实际 CLI 返回 1，**2.278 s**。 |
| 原 `SelectItem` 执行 | 使用原 14 行处理器、原相关模型字段和 `FileManagerRow` 定义，夹具提供一条中文文件记录。AV 桌面宿主派发点击，校对完整选中文本与路径、清空及第二实例隔离，最终 **0.431 s**。独立二进制链接准备 **7.858 s**。A2Vue 最终生产 CLI **1.550 s**，Vite 准备 **1.226 s**，Chrome 用同一处理器验证选中、清空和页面隔离，含清理 **1.563 s**。 |
| 循环和反例 | 扫描受检记录列表，在另一分支遮蔽局部，核总数 5、分支值 101、末尾文本 `d` 和第二实例的分支隔离。错误形参注解与重赋值给 `let` 分别具名拒绝。AutoUI 定向用例 **0.386 s**。A2Vue CLI **1.549 s**，Vite 准备 **1.258 s**，Chrome 含清理 **1.563 s**。 |
| 失败和修复 | 原选择处理器首次在 A2X AGE 准入被 `RecordClone` 拒绝。接入已有复制发射后，Chrome 点击复现 `double free on Str`。临时对照只给赋入模型的两个读取加文本复制后通过，最终由受检前门发射同一所有权语义，并重新生成、执行与验证。首次 AutoUI 准备遇 Windows rustc 导入解析内部错误，按相同目标重跑后通过。 |
| 定向回归 | AutoLang `plan981`、`plan980_on_if`、`plan980_lambda_if`、`plan1072_component_style` 共 286 例，**1.396 s**。`ts_ui_fragment` 5 例，**1.064 s**。最终锁定 debug 准备 **6.728 s**。实测重冻结 `age-package` 文法身份 `4eea971007834fd9551eb1ff639d7058835c836b90200175600bf019f5d01468`，相邻冻结面保持。AutoUI `plan114_widget_packages`、`plan124_routes`、`plan980_package_sfc`、`plan124_ace_vue_routes` 分别 12、4、11、2 例，**2.609、1.055、0.161、1.246 s**，最终锁定 debug 准备 **92.679 s**。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。AutoUI 使用上一轮的锁定 debug 准备命令。AutoLang 使用 `cargo test --locked --offline -j 2 -p ace-core --test conformance -p ace-a2x-ts --test ts_ui_fragment --no-run`。测试二进制均以 `--test-threads=1` 执行。日志 SHA256 如下。

- `r2-handler-control-original-sections.log` `5D2A1766172BCFDCDC0C44D24E055D7B27D01E49E131F1B5BF0B4AA4EEB1B209`
- `r2-handler-control-goback-diagnostic.log` `4D154C38700D5771C35CEACD9BA19F6B374035AC2DB5574B46894D2C4CF0D9B5`
- `r2-handler-select-cli.log` `822D7F53A29DB747C2DBADAE8552DB21CCA14CCFF146D4B58A6C246CC58FF3D1`
- `r2-handler-select-browser-diagnostic.log` `FC5049440CA61F27B4CA62B90ECD475074EC53F8662B2D90F5E806909704BD6D`
- `r2-handler-select-cli-owned.log` `F5E40391D971A3A28A417BE97425C3827585837F90842DF93CA227D1B885E83A`
- `r2-handler-select-native-owned.log` `0800C7D9839CB6F87140B3D9F3E5DBB6F9F94D38B9EA692E83A304557E70431B`
- `r2-handler-select-browser-owned.log` `FF340A385ADEA434551086C24E80DF02F6D5FF1798A8E1B9DCAE144DB1BAD1C1`
- `r2-handler-loop-browser-owned.log` `71386BC4748A8486DA9E7F9B77C396CA0AA77D282433AF933A6B26468BE01C4A`
- `r2-handler-owned-native-focus.log` `7C8275BC9FFAFBFFFD93A707F2D4B9288A29F43D06D479A9EFA70FBB6D455813`
- `r2-handler-owned-core-tests.log` `AF0FB761C3C0277F9A88A010ED88526CF6BD9DD2852D1E0CA621D2E8AB726E35`
- `r2-handler-owned-ts-tests.log` `33FA5658DDF8033DF80B0044F0ED6F63DE4535FB39FB74C8D1269F6E1EE59DEC`
- `r2-handler-owned-ui-tests.log` `F82DFF67343D78037A4F948FEF03406E8081925829E8B2F7AB25F10C48581B56`
- `r2-handler-owned-original-strict.log` `D1CFD251BD346E59750370D5E5762DFB6C442B6B1171DFE5014C13F82522F81D`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 33 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 84e1dcfb1a7d859cec9311c9b43a8b76f2cf3a99 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检同实例消息调用与原 027 历史分支 | outcome: partial | AutoLang `63384924891df483bc6081007ceaaae6354637d8` | AutoUI `0d79ec0efdbee1973933becf1a42a1a7ca32e11a` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 原 `NavTo` 的类型化文件请求、模型更新和 CollectionView 接线。

AGE 包处理器支持在本实例的分支同步调用其他受检消息。参数按声明类型和源码顺序求值一次，调用者与被调方各有局部作用域，模型更新按调用顺序发生。历史列表索引的文本局部和文本载荷持有独立副本。编译具名拒绝未知消息、参数个数或类型不符、递归及当前需要宿主效果的被调处理器，并限制展开规模。AV 与 A2Vue 复用同一受检处理器降低。

| 验证 | 结果 |
|---|---|
| 原 027 分段 | 同一组 46 个处理器中，语法拒绝从 20 降至 13。`GoBack`、`GoForward`、`GoUp`、`AddrGo`、`CtxOpen`、`CtxRename` 和 `CtxDelete` 进入受检消息解析，单段隔离探针具名报告其未随段定义的被调方。`BootTick`、正式 `NavTo` 和文件操作体仍待处理。原 `app.at` 无改动，SHA256 `3864DC97DEFAF4C33D9FB81B4358F8C01008EEFE00A81B409DF4EFDB2C1989E4`。原页严格 CLI 仍返回 1，**2.419 s**。 |
| 原历史分支执行 | 夹具直接取原 20 行 `GoBack`、`GoForward`，使用纯状态 `NavTo` 和双参数 `Publish`。AV 实际桌面宿主和 Chrome 均验证回退后路径 `docs`、索引 0、越界回退无更新、前进恢复 `docs/计划`、索引 1，以及第二实例隔离。AV 单跑 **0.799 s**，生产 CLI **1.704 s**，Vite 准备 **5.288 s**，Chrome 含清理 **2.744 s**。 |
| 定向回归 | AutoLang `plan981`、`plan980_on_if`、`plan980_lambda_if`、`plan1072_component_style` 共 286 例，程序内 **0.99 s**。实测冻结 `age-package` 文法身份 `e0ae12ad145592d52173c44306ef80bba75ac175925ffb137d34c580b6448fb9`，相邻冻结面保持。AutoUI 四个既有目标分别 13、4、11、2 例，**2.983、0.965、0.158、1.300 s**。初次 AutoUI 链接准备遇到 Windows rustc 堆损坏异常，重跑通过，最终锁定 debug 准备约 **135 s**。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。AutoUI 复用上一轮的锁定 debug 准备命令，AutoLang 以 `cargo test --locked --offline -j 2 -p ace-core --test conformance --no-run` 准备，测试二进制均以 `--test-threads=1` 执行。日志 SHA256 如下。

- `r2-handler-send-probe-final.log` `EA7DAFE25C1961790605B8EDABB43D49ADF81555FE0D1F923D37F257788544EE`
- `r2-handler-send-original-sections-final.log` `C89AE9554F70A45152FBB23763EA201F11B4488277EEF4E67159398BD47B9051`
- `r2-handler-send-native-focus.log` `AF7A2524A102147A92A528C9634C169A63B1F13DD53D5979BF1BF9E0CBFEE645`
- `r2-handler-send-cli.log` `C4274D7DE29BE4A0F3CE95B5742C64627C0EF1733304025015377B4A4E138A8E`
- `r2-handler-send-vite.log` `714D22088491A54CF2F58A9E445F0B5B64861CE79677E7B4B8418595051A4928`
- `r2-handler-send-browser.log` `82CB98D3E06A3B2727C0896DA7DBEB39ADD50BD56BA6102A81E04D8282135E0E`
- `r2-handler-send-core-regression.log` `0818C8D28D25CC63A1D5C281407D09A57FA18617B8544339EDC3F6E2E80534A2`
- `r2-handler-send-ui-regression.log` `4F7BA9509350CE1B0B18C7946164647AC7F658B75DD57C41B250F97EE6945808`
- `r2-handler-send-original-strict.log` `234C665F872D50D963B3363E423E566F01B481AE6B40BA0F172321B7B660C46F`
- `r2-handler-send-ui-build-final.log` `196243E9D13C632ECBC439112E970BE4DFBEE5FFD2F9EF16C877994E86F7B5C7`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 34 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 0d79ec0efdbee1973933becf1a42a1a7ca32e11a | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检目录查询结果进入 Widget 模型 | outcome: partial | AutoLang `4582c9e7d7c92294517aca8741c85e54017f65dc` | AutoUI `0d79ec0efdbee1973933becf1a42a1a7ca32e11a` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 在原 `NavTo` 的事件提交处接入受检文件请求与双端会话，结果更新原模型。

AGE 包的处理器可接收同模块声明的具体记录。目录结果记录内含路径、成功标志和条目列表，列表类型由既有集合验证器检查。真实宿主候选结果走同一条类型验证。嵌套消息交接接受新构造的记录或 `.move` 转交的局部记录。JS 和 AV 实际执行的局部所有权一致。普通 DOM 事件保留既有标量载荷。输入语法身份不变。

| 验证 | 结果 |
|---|---|
| 真实目录结果探针 | 使用生产 `FileSession` 读取材料根内含中文路径的实际目录，独立核对磁盘元数据，再把同一目录结果交给真实受检 AV 执行。跨根请求被拒，撤权后查询被拒，重新授权后恢复，关闭后查询被拒。合法结果更新路径和首条名称，失败结果保留先前状态，第二实例保持初始值。运行 **0.115 s**，独立探针链接准备 **7.136 s**。探针使用目录级结果夹具和可放入 `int` 的元数据。原窗口的请求分发、全范围元数据、撤权后迟到提交、写操作、CollectionView 与资源回收仍待双端产品接入。 |
| AV 定向用例 | 真 AV 执行同一个 `DirListing` 记录和嵌套 `List<DirEntry>`，复核成功和拒绝结果、两实例隔离、错误载荷与借用记录转交的诊断。单跑 **0.104 s**。首次尝试暴露了列表记录验证域选择与 AV 参数所有权标记丢失，修复后通过。 |
| A2Vue 与 Chrome | 使用同一受检目录结果处理器。夹具把条目先绑定到显式类型的局部变量，供项目入口的既有验证与受检发射使用。首次 Chrome 点击由隐式记录别名复现 `double free on Str`。最终新建输出目录，实际 `auto build --render vue --gen-only` 冷生成 **0.939 s**，Vite **1.061 s**，Chrome 含关闭 **1.027 s**，确认成功、失败后的状态及第二页面隔离。浏览器由夹具构造回复。`App.vue` SHA256 `32EC2DAC12D6074995B013A25077997849A6A4DD2CFC308BB4AEDB1535F0F6FA`。 |
| 原 027 与定向回归 | 原页严格 CLI 仍在 `source.syntax` 返回 1，**2.401 s**。AutoLang `plan981` 等受影响 Core/AV 分组分别 321、16 例通过，**1.106、0.352 s**，最终锁定 debug 准备 **4.67 s**。AutoUI `plan114_widget_packages`、`plan124_routes`、`plan980_package_sfc`、`plan124_ace_vue_routes` 分别 13、4、11、2 例通过，**3.094、0.949、0.244、1.578 s**，最终锁定 debug 准备约 **75 s**。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。AutoUI 使用上一轮的锁定 debug 准备命令，AutoLang 使用 `cargo test --locked --offline -j 2 -p ace-core --test conformance -p ace-av --test conformance --no-run`，测试二进制均以 `--test-threads=1` 执行。日志 SHA256 如下。

- `r2-dir-payload-native-first.log` `0434DAC613A485462A93C083F7827CA6B1A8E7A6CA4B81E7B551CB0AFCC35BFB`
- `r2-dir-payload-native-third.log` `0182E55FD96A27A5293EE2A905D496C67D80F3CAB549A32209C860EB1910DAED`
- `r2-dir-payload-browser-first.log` `ACE00F983C83B4854741501975A770F5471C00BA4AF1D1EBD6520E59E16ED5EE`
- `r2-dir-payload-native-verified.log` `89054C472899AE1826BE8BAF23E18F7443D21E96C79D8350BD954D9F17D4CC55`
- `r2-dir-payload-av-verified.log` `B2E1FF36D286250E7E96FE939B7EB83FB950CC9297BADEE3CF5705A44E31409F`
- `r2-dir-payload-core-verified.log` `C36ADE6FACEA4A51995C388BD23F1866E61ED541E921C733EA3520DDC51359DE`
- `r2-dir-payload-real.log` `4A052F4EED4505A24ECF1AA3D04D66410A8264371425B7EBD0AD3A531C22D925`
- `r2-dir-payload-real-build-final-2.log` `0562B21857465F0F4BC22E674C503A679A656D6AB0547791957F489FCD3A3B80`
- `r2-dir-payload-cli-cold.log` `B1E8BABAC607714778E6449181F09ABB72E37D216206C4E4BF057E4DC1D3F3D0`
- `r2-dir-payload-vite-cold.log` `9A5DBC78F77226B8316B43631D0E44667998A2A352FA413FE4FFCAEE5DD56789`
- `r2-dir-payload-browser-cold.log` `252ECDD0EC254107A4FCD49BDD44CA100F51B8D126E4CD6B584177BC54E22D5F`
- `r2-dir-payload-ui-verified.log` `EB078D5ED8B8615AFEE0B1A4F0F03B0B6E2FDAC3CEA79460D44AE47AAA8923EA`
- `r2-dir-payload-original-strict.log` `5A89B17EF95507469771AD2E2355426D9CD30649E6B2E49CC1BA35EEB9F53D4F`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 35 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 4582c9e7d7c92294517aca8741c85e54017f65dc | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 AV 宿主在提交处执行受检目录请求 | outcome: partial | AutoLang `d3a12b13e05039a49054d8964933c4ee0c5e9214` | AutoUI `e4b43e85a2c150788ac062df31f5ab2893784f8c` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将此请求与结果合同接到 A2Vue 原后端和 027 原 `NavTo`，继续原操作与 CollectionView。

受检 Widget 用 `bind { "file:list" -> .DirectoryListed }` 选择目录读取，模型提供请求路径和待处理标志。加载时校验请求字段及成功和失败的 `DirListing` 回复。AV 只在挂载或事件提交处理仍待执行的请求，通过该实例的 `FileSession` 校验授权并访问文件系统。回复以受检记录进入历史，清除待处理标志后发布整帧。文件大小和时间使用 `u64`、`i64`，语法身份保持。桌面窗口启动参数 `--file-root` 显式授予本次挂载的实例，关闭后释放各实例授权。

| 验证 | 结果 |
|---|---|
| 受检形状与反例 | 直接探针先定位 `u64` 被解析成未知名义类型。64 位字段投影接通后，`u64::MAX` 与 `i64::MIN` 的回复候选通过检查。宿主首次定向运行发现有文件绑定且没有计时器或子组件的入口缺少受检视图，修复后同一组件直接进入受检入口。错误的 `size int` 在挂载前被拒。 |
| AV 桌面宿主 | `desktop_file_binding_commits_one_scoped_read_and_replays_its_reply` 单跑 **1.458 s**。真实临时目录、两实例、未授权及越界、撤权再授、显式刷新、关闭重开、旧窗点击丢弃。磁盘文件替换后普通事件仍保留上次读取结果，显式刷新才看到新文件。 |
| Iced 实窗 | 从同一绑定夹具生成两份材料源，真实窗口以计时器触发目录请求。授权窗显示 `窗口.txt`，6 帧，**2.705 s**。越界窗不显示根外文件，10 帧，**2.586 s**。两次均关闭会话，各丢弃一个旧事件。锁定 debug 窗口准备 **13.51 s**。 |
| 原 027 与定向回归 | 原 027 严格 CLI 仍在 `source.syntax` 返回 1，**2.469 s**。原页面与 A2Vue 还需消费本轮目录合同。AutoLang Core、AV 分别 321、16 例，**1.131、0.424 s**，锁定准备 **17.37 s**。AutoUI 四个既有目标分别 14、4、11、2 例，**4.228、1.058、0.163、1.256 s**，锁定准备约 **86 s**。原写权限、迟到异步回复、CollectionView 和全页双端验收待后续。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。测试准备沿用上一轮命令。窗口以 `cargo build --locked --offline -j 2 -p auto-ui-runtime --features ace-bridge --example ace_node_desktop_window -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 准备。两次窗口均传入材料根下许可目录，脚本为 `96:close,99:late`。日志及窗口证据 SHA256 如下。

- `r2-av-file-effect-arity.log` `05CCEE10EC81C66973A44C53966CDD6A1C9B0FF443243C140C9A5B9BB0985EC1`
- `r2-av-file-effect-arity-verified.log` `787A25BC5969F90411B81258E4C1E0D986C0EBFBAFBD6636C57ED19039F0E2FC`
- `r2-av-file-effect-focus.log` `EF47F4AE36E1DFF936421C7B5032B54247CBAF822725D83113AE96F98D72A555`
- `r2-av-file-effect-focus-verified.log` `2390BFC0DC110FCE63D549559F6AEFA2500CE543B1AF30ABC4A3B8A8C1F91977`
- `r2-av-file-effect-av-verified.log` `EA170C1D781010521532E4F7942853AC1F3DD4490B5DEC775DAF861B06CB3497`
- `r2-av-file-effect-core-verified.log` `ECD4D3DEA2E398C7091C5B159E59EE07D4521FFA988B5CC0BC3E980C9EDEBF3F`
- `r2-av-file-effect-ui-verified.log` `2816C8DA595E6D873884AC38AA16FFB3E62BAC82695790AC17AD2A1E89B30622`
- `r2-av-file-effect-window-app.log` `F9F5B9DF2C4F53654A631493E1CFCDC49A1428F4FEA2BF28DE3FDDF76343F828`
- `r2-av-file-effect-window-outside.log` `C4E85792B08439A3F9C2A7E25189E735965B19EF785E8118118A171EC7F8CE4D`
- `r2-av-file-effect-original-strict.log` `E1403B257B5154618FEC3199E70841FB2082BA428B00B446E42F4EBB6E12F19B`
- `r2-av-file-effect-window/evidence-app.json` `0F9043A1F3B6F4C5B45CA33B09A0E7135A7AFDC21C6D54B68002075694A0D768`
- `r2-av-file-effect-window/evidence-outside.json` `C6ABA1CD5FD7E593525420221AD96058D014F3D003389760EA5D3365C1B51ED1`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 36 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: e4b43e85a2c150788ac062df31f5ab2893784f8c | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 A2Vue 目录查询使用原文件后端 | outcome: partial | AutoLang `6a241b45bef0392fe7868864f1140c9cb7a646b2` | AutoUI `e9a25846fe7f407da2bb36f1dbe0849757d232f3` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原 027 的启动和 NavTo 接入已验证的双端目录合同，继续原列表与文件操作。

A2Vue 生成入口校验与 AV 相同的 `file:list` 回复合同。生成的受检视图在请求后保留先前已提交视图，确认当前请求和视图帧仍有效才提交结果。新导航与卸载取消待处理请求，页面关闭撤销浏览器文件会话。客户端经会话限定的 `/api/fs/list` 访问原文件后端，失败返回进入既有处理器，文件大小和时间精确经过 JSON 和受检集合种子。生成骨架在冷生成和缓存命中时都维护所需客户端及 TypeScript 配置。

| 验证 | 结果 |
|---|---|
| 原后端与 Chrome | 实际 `auto run --render vue --server vm` 使用检查夹具前端与原 027 后端字节相同的 `src/back/api.at`。核对授权根内两个实际中文文件、越界拒绝后保持视图、双页面隔离。实际后端的旧响应在新导航后和卸载后分别交给客户端，未覆盖视图。关闭旧页后原会话返回 404，新页仍可读。最终整例含 Chrome 关闭 **3.182 s**，启动准备 **7.060 s**。首个成功回放 **8.463 s**，后两次受后台标签调度影响分别 **34.732 s** 和 **97.374 s**。改为前台驱动并确认正文实际被消费后，保留了全部断言。后端源码 SHA256 `322E6F6473AE2C5448E5E0AA919FD4DD484772705B15592A654FD31352A6D257`。 |
| 构建与缓存恢复 | 实际 `auto build --render vue --gen-only` 冷生成退出 0，**2.692 s**。故意更改 `allowJs` 并移走文件客户端，缓存命中再次生成，**0.921 s**，恢复两者及原客户端哈希。首次 `vue-tsc` 在受检 JS 组件报 TS7016，修正骨架与缓存恢复后完整 `pnpm run build` 退出 0，**3.839 s**。 |
| 64 位与定向回归 | `ace_a2x_ts` 的 `seed::` 10 例通过，**0.159 s**。Node 对实际 TypeScript 客户端验证 `u64::MAX`、`i64::MIN`、溢出和拒绝结果，**0.168 s**。AutoUI 五组 `plan114_widget_packages`、`plan114_file_session`、`plan124_routes`、`plan980_package_sfc`、`plan124_ace_vue_routes` 分别 14、2、4、11、2 例通过，**4.746、3.698、1.094、0.252、1.338 s**。定向 AV 与 SFC 形状校验 **1.743 s**。锁定 debug 最终依赖准备 **29.62 s**。 |
| 原 027 | 原前端严格 CLI 仍在 `source.syntax` 返回 1，**2.485 s**。本轮使用受检目录夹具验证原后端，原页面输入、写入、CollectionView 和两条 gallery 路由继续由 T-02 覆盖。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。主要证据 SHA256 如下。

- `r2-web-file-effect-seed-tests.log` `910ad70d23b7283c2e89004c5f3e72cbcccf55da29d4c4b57125d8d33968ee94`
- `r2-web-file-effect-width.log` `4c8bfb9db941c7bab5fe18204337004c200357f43c1e5f4fdf8cf2d30079e011`
- `r2-web-file-effect-focus-verified.log` `00eb0747bc70e118a54e5f1fa0ae19319b892a01f890018066607cd10db5c2f5`
- `r2-web-file-effect-ui-tests-final.log` `662d4490e444f4c266069ceb34efef65a7fb1138ca735686744e3fab0c4dad6c`
- `r2-web-file-effect-vite.log` `9824b2617d2e9b818a654e7d638919ff98da3674ff3673a1040e7006ec1ff6fd`
- `r2-web-file-effect-cli-cold-final.log` `8b109bafc41821248133470ee4741da15364df63086c211979e18614178a287a`
- `r2-web-file-effect-cli-cache-final.log` `ff0d55768dc75518c14583607bf8c9f21f8d5206e10f2371d4e6039c9366ca4a`
- `r2-web-file-effect-vite-final.log` `412b7bb8fe6b22aa35dd2811b9f2fd9390b1a6d5fd059763e52cc8ad7b5ee488`
- `r2-web-file-effect-browser-lifecycle.log` `a2dee942ed3c274d6306bca959637413a432171ae2197c45a68fd5a00e65161f`
- `r2-web-file-effect-original-strict.log` `859759442a748c95cd7514cc8db98d2918faf56da6bb84c1b0188f89c02dd1b3`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 37 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: e9a25846fe7f407da2bb36f1dbe0849757d232f3 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的目录历史提交及双端授权起点 | outcome: partial | AutoLang `0d749ceb001b2f2b0fda0ca3d00e45009cfecfba` | AutoUI `65673e6e81192e190b9b4ed7729c4204b5702561` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原 BootTick 和 Navigate 迁到受检文件请求，把目录回复接原行物化和 CollectionView，继续受检文件操作。

原 027 的 `GoBack`、`GoForward` 将候选索引与路径交给 `Navigate`。两条既有数据入口均在目录可读后通过同一处理器更新历史和导航按钮。跳转到新路径时复制保留前缀并替换前进分支。原 Vue/VM 页按导航序号忽略过期回复。受检 AV 与 A2Vue 的空路径目录请求使用当前会话授权根。AV 在一次授权锁内读取根目录。A2Vue 通过原后端同一文档会话的 `/api/fs/home` 和 `/api/fs/list` 取得根及条目。

| 验证 | 结果 |
|---|---|
| 原 027 兼容链 | 用工作树字节一致的原 `pac.at`、`app.at`、辅助模块和 `api.at` 运行 `auto run --render vue --server vm`。真实路径经地址栏进入，移走后退目标，收到 `ok:false` 后保留当前文件和导航方向。恢复目录后重试、前进及历史分支替换通过。将先请求的 Alpha 回复保留到 Beta 导航完成后再实际交付，视图仍显示 Beta。最终 Chrome 含关闭 **5.335 s**，启动准备 **13.587 s**。同一用例并行准备阶段曾通过但耗时 **11.879 s**，随后隔离复跑。中间迭代发现兼容生成器同名参数遮蔽和仅换导入模块时的旧生成缓存，调整参数并在新材料目录复验。原 `app.at` SHA256 `74668605645F5F0AF1F65784A37197D3ACA8FAC5783D5567F2EC1106145A605E`。原模型模块 SHA256 `E5527AD08F22C526B640007C30BE0E4C822E24B93E366148CEC732AA7093BE74`。 |
| AV 与受检生成回归 | `original_history_handlers_call_typed_widget_messages_in_order` 使用原提交、后退、前进处理器及原模型 helper。目录接点用状态夹具控制成功或失败。拒绝、恢复、分支替换、重放与两实例隔离通过，**6.806 s**。初次发现列表文本所有权拒绝及 MIR 转换类型不符，最终显式得到独立拥有的字符串。十次事件的桌面探针首次在末次前进超过冻结的 16 MiB 验证额度，合并重复提交分支后同序列通过，**6.388 s**。`desktop_file_binding_commits_one_scoped_read_and_replays_its_reply` 核实际授权根的正反响应、越界、撤销及关闭后恢复，**2.202 s**。现有 `plan114_widget_packages` 14 例通过，整组 **10.714 s**。 |
| A2Vue 授权根 | 受检测试前端以启动计时器请求授权根，并运行与原 027 字节一致的后端。两页各独立发出一次根查询和目录读取。Chrome 核对真实目录、越界、两种迟到回复、关闭后原地址 404 与另一页仍可读，含清理 **1.423 s**，启动准备 **12.298 s**。`pnpm run build` 执行 `vue-tsc && vite build` 退出 0，**3.112 s**。 |
| 原 027 严格入口 | `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍在 `source.syntax` 返回 1，**2.059 s**。原启动、读盘处理器及行物化的严格迁移待实施。兼容启动及受检夹具的行为证据见上。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。测试准备使用 `cargo test --locked --offline -j 1 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages -p auto-ui-tools --features ace-vue --bin auto-ui-tool --no-run`，最终 **38.755 s**。CLI 用 `cargo build --locked --offline -j 1 -p auto-ui-tools --features ace-vue --bin auto-ui-tool`，关闭占用旧二进制的预览后准备 **34.223 s**。主要证据 SHA256 如下。

- `r2-original-navigation-browser-shared.log` `B911E741C127E5CD4E3252E147E5EB869667E186F8BA7F29803C59E3CAD48A8A`
- `r2-original-navigation-browser-replay.log` `F1985D1AC71CD37216EBC6B34CD4A844F07A43B812AC419EB9C349D91C1FDFBE`
- `r2-original-navigation-strict-shared.log` `B2B8BB2B5B87A87456C25B73BA6B4BDE1FDD745CD95B6F26EFF6DD5940ACBB8A`
- `r2-original-history-desktop-observation.log` `5A27CD73E59287AABECD5C472072A9F8E1DDD9AAEDE81B291A39DD801B659EB1`
- `r2-original-history-desktop-shared.log` `DBB88A4AF3D91137F267D71205E1D565B8B927F03756E02B0206FFFA54A48932`
- `r2-file-bootstrap-original_history_handlers_call_typed_widget_messages_in_order-shared.log` `E0AA7A7BB9BD55AD21C92561646D750E540C60EE830EE3AC0A72ECA3DB120309`
- `r2-file-bootstrap-desktop_file_binding_commits_one_scoped_read_and_replays_its_reply-shared.log` `26A3BE9256B78070601C7D3BB0D7445B564D7D4721620B5771909CB35BC7132C`
- `r2-file-bootstrap-widget-package-regression-shared.log` `C557DA5D51025B51512FFEE56B6233A05DD8233A951B810990E3E2B667D972E6`
- `r2-file-bootstrap-browser-observation.log` `4F8F75A17312BE569E30DABD7A92702CE0EF4BCEA725E195136F5467519280E2`
- `r2-file-bootstrap-vite.log` `71C1FE933A0AABA1F110CDBF44FDCF43C45482CAE982DDDE3A2E852493A9D4E8`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 38 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 65673e6e81192e190b9b4ed7729c4204b5702561 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 复用启动目录结果并共享受检路径提交 | outcome: partial | AutoLang `0d749ceb001b2f2b0fda0ca3d00e45009cfecfba` | AutoUI `de92c2440811f6d356c9e6581d4e2b60ffaff6fd` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原 027 的条目筛选、排序、64 位元数据格式化和行发布改为受检代码，完成原启动和导航的宿主绑定及文件操作。

原 027 的 Vue 启动用同一次授权目录结果填充快捷访问与列表。初始化结果仅在导航序号仍有效时提交。正常导航复用该列表结果处理器。两条现有读盘路径在成功后使用原应用的同一处理器提交地址、选中状态、面包屑和历史。路径与面包屑函数可进入受检 AGE。前端目录回复声明保留 `u64` 大小与 `i64` 修改时间。上一级在浏览根停止，面包屑展开和地址编辑使用各自的点击区域。

| 验证 | 结果 |
|---|---|
| 原 027 兼容链 | 在新材料目录复制实际 `pac.at`、`app.at`、后端及预期依赖，`auto run --render vue --server vm` 启动准备 **17.983 s**。Chrome **5.217 s**。首屏只发出一次授权目录读取，确认真实文件行、折叠展开与重新折叠、上一级到根及根处再次点击无读盘。移走历史目标验证拒绝保持旧列表和历史，恢复后重试通过。先发的 Alpha 回包在 Beta 后实际交付，Beta 视图保持。中间回放发现旧生成器局部参数遮蔽、显式转换内部的 helper 引用未入闭包，以及扩展按钮点击被编辑区域接收。修正后从新材料目录复验。 |
| 原路径提交的受检 AV | `file_manager_path_is_published_only_after_an_authorized_read` 直接使用原目录回复声明、路径/历史提交和展开处理器，宿主从真实隔离目录读取，渲染折叠的中文路径。越界后检查路径、历史与标签，最后展开全链，**4.956 s**。初次夹具跨模块声明消息记录被拒，改用实际原 App 的同模块契约。随后 AV 对分隔符局部的视图转换给出 MIR 诊断，显式获取视图再复制，路径探针 **0.279 s**。锁定 debug 依赖准备 **29.339 s**，`plan114_widget_packages` 15 例通过，整组 **17.185 s**。 |
| A2Vue 受检路径 | 同一原路径处理器及后端经实际 `auto run --render vue --server vm`，启动准备 **17.836 s**，Chrome **1.199 s**。实际根回复包含已知文件，越界 `ok:false` 后面包屑保持，展开显示原折叠中段。实际 `pnpm run build` 的 `vue-tsc && vite build` 退出 0，**3.719 s**。这里使用受检目录绑定夹具接原路径处理器。 |
| 原 027 严格入口 | `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍在 `source.syntax` 返回 1，**2.695 s**。原启动和导航的受检宿主端、条目物化、CollectionView 和写操作继续在 T-02 实施。原 `app.at` SHA256 `2F1EE7DB338D4C35F673A78F92181CF951CA29E382209E51DF92C9FC18D07A53`，路径模块 SHA256 `9E720A50AEA037E1FD15FAD0CF389A8C678C4F488E542808182C9496B2F8DC48`。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。本轮 Rust 测试准备命令沿用上一轮的 `cargo test --locked --offline -j 1 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages -p auto-ui-tools --features ace-vue --bin auto-ui-tool --no-run`。主要证据 SHA256 如下。

- `r2-file-paths-browser-replay.log` `C2C3B601D940942E3234FD21402DD02BCFCD6FFF6C4144D7CB43DC02389C1C36`
- `r2-file-paths-native-replay.log` `2C51B964DDE478AC9003F9106E70F5916F9538A5A503E5BD69A95F40CBA4BCAD`
- `r2-file-paths-av-probe-fixed.log` `40926802468EED8B19D266DB27807D37CD8A86B89C621BF0956F18F23086093B`
- `r2-file-paths-native-group.log` `ADDD71CF91C2557B66471996E9F49EE84D3C7186A0AE8A6795D20AB7D3B46802`
- `r2-file-paths-checked-browser-replay.log` `C34C9FF5FFAF119DFFE26A472C311BBF98162E8A44F01A03354FC16E128437B7`
- `r2-file-paths-vite.log` `7E5B939CFFEB658DED8A29064446C15E03265C5B21F77C5F1A458905A5C8EC43`
- `r2-file-paths-original-strict-fresh.log` `AEAAF04926DC0D7612B0457215EED5ABEA46436C4FE3643044986DC74E794C62`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 39 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: de92c2440811f6d356c9e6581d4e2b60ffaff6fd | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 文件行与目录总量使用受检 64 位格式化 | outcome: partial | AutoLang `a5480ba17e063e955df96f7d3e05777d692cf19e` | AutoUI `6289763fd9f40e502e5678e2e508252c7b7284df` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 迁移条目筛选、排序和受检行发布，贯通原启动、导航、CollectionView 与写操作。

原 027 的文件行持有 `u64` 大小。文件大小、目录总量和 UTC 修改时间由同一个源码模块格式化。目录总量分别累积完整 GiB 和余数字节，可格式化超过 `u64::MAX` 的总字节数，目录条目不计入合计。修改时间以 `i64` 运算，既有不可用时间显示为 `--`。AGE 包给原生 64 位字段、函数签名、局部注解和显式整数转换使用同一校验。整数 `.to(int)` 后缀只加到 AGE 包面。该面 grammar id 更新为 `cb1aac97638c569cacab4df363e7ddd737373213b6d0c53b6773f6dfeffa7dba`。

| 验证 | 结果 |
|---|---|
| 原页面兼容链 | 真实 NTFS 稀疏文件由系统报告 **2,684,354,560 bytes**，真实修改时间 **2208988800**。原 `auto run --render vue --server vm` 页面同时显示行及目录总计 **2.5 GB** 和 UTC **2040-01-01 00:00**。验证单次启动读取、面包屑、授权根上行、拒绝保持和迟到回复。最终工具的启动准备 **7.877 s**，Chrome **4.959 s**。首轮曾因旧兼容生成器把除法输出为浮点且未收集经模块级函数间接引用的总量 helper 而失败。显式给商值定整数类型，并把总量入口与其依赖归入同一模块，随后用真实文件回验。 |
| 受检 AV 与 A2Vue | 原元数据模块与原文件行声明供受检测试 Widget 调用。输入是可控边界值，独立预期为 **17179869183.9 GB**、两个最大文件的合计 **34359738367.9 GB**、**2040-01-01 00:00** 与 **--**，包含目录排除。AV 定向用例 **0.510 s**，独立路径探针 **0.189 s**。实际工具启动生成的 A2Vue，Chrome **1.580 s**，启动准备 **25.350 s**，`vue-tsc && vite build` **2.875 s**。 |
| 定向回归 | AutoLang `plan981_age_package_av` 16 例 **0.639 s**，`plan981_age_package` 13 例 **0.480 s**，相邻文法身份 `plan981_grammar_scope` 4 例 **0.439 s**，`numeric_source` 5 例 **0.298 s**。新增转换正例和混合宽度反例单跑 **0.050 s**。准备采用 `cargo test --locked --offline -j 1 -p ace-core --test conformance -p ace-av --test conformance --no-run`，**46.371 s**。AutoUI `plan114_widget_packages` 16 例 **19.198 s**，锁定 debug 准备 **126.416 s**。CLI debug 构建 **79.553 s**。 |
| 原 027 严格入口 | 实际最终 CLI 严格生成仍在 `source.syntax` 返回 1，**2.605 s**。本轮实盘接入属于现有 Vue 兼容链。原整页的条目处理和宿主接线继续实施。App SHA256 `EF40B980934D92393B94B01919ADCC66ABBA16D3725B117D0AC58C7FFA06F5E9`，元数据模块 SHA256 `DDAE998CD0E7BD6DB2CCE27B3B82CE519FA9F4968EAA6C95FF3210C98DCC8895`。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`。证据 SHA256 如下。

- `r2-file-metadata-original-browser.log` `01A62679117D8D4B5F1EC390CD4F8FE95E49834964799D588324D9E4C3B157B3`
- `r2-file-metadata-original-browser-final.log` `D9BBA1DC5287934C5CD688B866CCAB6FA653D54D8AE3F792E94B1810CA3E0721`
- `r2-file-metadata-av-entry.log` `A710C961C9080C16380EAD30CBD8916DE85A3B7F5A8513516CD1BB45D14926DA`
- `r2-file-metadata-ui-focus.log` `0BAB96F122E3014A4937A71D5D71FBAFF5FAE37A51C5A005BAA962FD0FA4CF1D`
- `r2-file-metadata-ui-regression.log` `B9EBEA074434D03F6DCAB88297AA57802F2D44A3BAE36005AA27570CB274830A`
- `r2-file-metadata-checked-browser.log` `BC80F8C80E95EE75C64E33FA56604C04AC395AA259AEAB40637DCE66CA3BE316`
- `r2-file-metadata-vite.log` `E36C0E4E59AC5EB4E822E4FD99F995C10E48EC7E6094F7C2A5718B3246FB6478`
- `r2-file-metadata-original-strict.log` `AD66BCD3DCE2AAFF7CEF6CD505682AC3689A93473B35D6822B934071999CE91D`
- `r2-file-metadata-lang-p114_age_package_checks_directory_metadata_integer_conversions-wide.log` `3BA9B5DAA797F12F3AAE0A7128ADE0C24C69F016F71BB6EFD6BE9998D180E4F4`
- `r2-file-metadata-lang-plan981_age_package-wide.log` `61AD167D0676D1C1F705377AA6C96AE23B376112A928C937005D62A987847A81`
- `r2-file-metadata-lang-plan981_age_package_av-wide.log` `81B6BD610E16A88A925128AA10B0411C67FE06A8D27EF15CDCE1FD4A4203CBF8`
- `r2-file-metadata-lang-plan981_grammar_scope-wide.log` `6D1A52D3F629555966F44E5E83C313727E34A7F61E0B036E22C7D6DE148752F1`
- `r2-file-metadata-lang-numeric-source.log` `191EE31768CCC85E6B136CA1E1D7492A08AC6E44559D0CA792CB4C8EF9BF4EE9`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 40 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 6289763fd9f40e502e5678e2e508252c7b7284df | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 共用受检文件筛选、稳定排序和路径选中 | outcome: partial | AutoLang `1ff26df98bb74dead931bff7cb044a4961356d38` | AutoUI `404c35885e6136d0fcdbb1bb790e6333f86180b9` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 核定受检 500 行执行预算，接通原目录回复行生成、启动、导航、CollectionView 和写操作。

原 027 两条读盘分支共用点前缀和子串筛选、计数、四列稳定归并排序及行重编号。目录占优，同键按输入顺序。文本按 Unicode 码点顺序比较，统一原生与浏览器在补充平面名称上的顺序。大小在受检链按 `u64` 比较。发布时用文件路径重新定位选中行。AutoLang 的 `codepoint_at` 在受检前门复用字符索引及码点转换，并由旧 VM 支持。兼容 Vue 发射 `codePointAt`，比较循环在合法文件名上与原生保持次序。

| 验证 | 结果 |
|---|---|
| 原 027 兼容入口 | 实际 `auto run --render vue --server vm` 从新材料目录启动，准备 **17.798 s**，Chrome **5.191 s**。验真实目录优先、补充平面和 BMP 文件名顺序、2.5 GB 稀疏文件的大小双向排序、排序后保留同一文件、中文搜索和隐藏项开关。中间复验发现兼容生成器把同名 helper 参数当成模型 ref，改参数名后以新生成目录通过。原 `app.at` SHA256 `72E0C0AD137909059EA25B01C0963562F71184D1EB4CF326458C2C8B69ED35F9`，行模块 SHA256 `EBA6D75BF24D072CD47A1B1B5E862B8163DE3A9D80950FAB438B46D9F65CDB5A`。 |
| 受检 AV 与 A2Vue | `file_manager_rows_preserve_order_and_selection_identity` 调用原行模块和原 `.SelectItem`，核精确区分 `2^63` 与 `2^63−1`、同键稳定、目录优先、排序后实际路径和显示 ID。单例 **0.538 s**。独立 AV 探针 **0.173 s**。同一受检页面的实际 A2Vue 在 Chrome **1.974 s** 通过，启动准备 **12.569 s**，`vue-tsc && vite build` 退出 0，**2.924 s**。 |
| 受检 500 行压力 | 500 条已构造记录直接显示首末路径通过，**0.137 s**。同输入用默认 AV `ExecLimits` 按名称 **0.402 s**、按大小 **0.346 s**，均具名返回 `InstrSteps` 并报告 100 万步超限。尝试以共享前缀二分替代逐码点循环仍超限。完整受检页依然需验证 500 行排序与发布。 |
| 定向回归 | AutoLang `plan981_age_package_av` 16 例、`plan981_age_package` 13 例、`numeric_source` 5 例通过，分别 **0.293 s**、**0.129 s**、**0.058 s**。`plan114_widget_packages --test-threads=1` 17 例，**15.660 s**。并行试跑在既有桌面测试的本机回调处越界退出，串行重跑通过。 |
| 原 027 严格入口 | `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍于 `source.syntax` 拒绝原 `widget`，退出 1，**2.242 s**。当前原页实盘沿兼容 Vue。受检 AV 和 A2Vue 素材直接调用同一原帮助模块及选中处理器。 |

测试和工具使用 `--locked --offline -j1` 调试构建。ACE conformance 准备 **39.684 s**，AutoUI 定向测试准备 **3m18s**，实际 CLI 工具准备 **147.536 s**。材料根 `D:/autostack/plan-artifacts/plan-114/`。主要证据 SHA256 如下。

- `r2-file-rows-original-final-browser.log` `207D6F9893223801AD795F0A5677E9B7D41D5DD7B4D1B7C46805C8521A949C53`
- `r2-file-rows-original-strict.log` `B84A2B4938BBCB100932ED7F68CEFDEDE33B88F713753B7678B1081CA9C4A9CF`
- `r2-file-rows-av-entry-final.log` `F5A1F55AE6AA7DF267602798B51623C9E2402B314C7B5D6F2517C60805E2D141`
- `r2-file-rows-checked-browser.log` `88D2BA057F8CEE3777C4AAABD80C3BCD26D5FFA5A705EE78ABF00593A9379B7B`
- `r2-file-rows-vite.log` `92AEF0957EFD15F216A6953884962638F98175E9AC70EB85E23417D8DEBBE34D`
- `r2-file-rows-ui-focus.log` `E719F10F2C9880AA40C9DBB799238FFB4D5F3C3D57014917F8BBA0A6CD9F9A33`
- `r2-file-rows-ui-regression-serial.log` `12FB162D3C6E9EC59639E7E8DA5D1F46B38505CF32193193EF8AA5163A1C3B8A`
- `r2-file-rows-ui-regression.log` `E93C6C6657AA967532AC23237978657F43841DC08AE6ACF9F973EE6431B4180F`
- `r2-file-rows-av-500.log`、`r2-file-rows-av-500-size.log` `5616F30825B5FA2F62BAAF12213EBBE64BB3AA383850A935529A651FFA690762`
- `r2-file-rows-av-500-input.log` `E298F8D1A32D5F06CFFF9A799D41434F92BC6B92368CC417C3EC80CC64258844`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 41 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 404c35885e6136d0fcdbb1bb790e6333f86180b9 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 目录回复行构造与 500 行桌面帧预算 | outcome: partial | AutoLang `463895bf0a047efc2d11a1884ee79889b070e4a1` | AutoUI `a466eedbc950c45e1664305ce9142d3a9911f3c5` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原目录回复的循环移到受检函数，贯通原启动和导航，并完成 CollectionView 和写操作。复验长文件名及多次目录事件的 AV 预算。

原 Vue 目录回包分支通过 `listed_file_row` 构造实际显示记录。该函数使用同一套 64 位大小、UTC 修改时间和文件类型规则。归并在排序函数内直接读记录字段，消除每次比较传整张表时的 AV 递归计费。字符串比较用最长公共前缀和下一码点。此次复验覆盖名称和大小排序、目录优先、同键输入顺序及路径选中。受检桌面包将视图、标量和计时器条件的每次求值统一到同一帧预算，600 万步、4 秒。通用 AV 观察入口按其默认额度执行。

| 验证 | 结果 |
|---|---|
| 500 行受检桌面帧 | `full_file_manager_listing_publishes_view_and_scalar_state` 使用原行构造、排序和日期格式化，在实际 `PackageStateEngine` 挂载和派发后读回排序首末路径及 2040 时间，终态单例 **2.961 s**。旧排序对同一 500 行原始记录的名称和大小排序均触及 600 万步。优化后仅排序的大小用例在默认 100 万步通过，**0.256 s**。行构造加排序的名称与大小在 600 万步分别 **0.617 s**、**0.456 s**。 |
| AV 剩余边界 | 500 行、每个文件名以 128 个相同汉字开头的合成输入，行构造加名称排序在 600 万步具名拒绝，**0.944 s**。独立探针用 1000 万步通过，**1.109 s**。此证据不覆盖原严格页多次事件和长名称的完整帧。 |
| 原 027 实盘 | `auto run --render vue --server vm` 从字节一致的原项目启动，准备 **37.955 s**，Chrome **7.718 s**。验证真实 2.5 GB 文件、2040 时间、`.É` 小写标签、目录优先、中文搜索、隐藏项、大小双向排序和选中路径。原 `app.at` SHA256 `0C89F9AB9B9BA2EC8CF39EFEC2DC5189F5F39FF5014DCD97C88E97F74F23D857`，行模块 SHA256 `988837ABE17E5BC85F9666F80DBBF07321516BB62E67175550C5596A6BB1AA1D`。 |
| 受检 A2Vue | 同一原行构造及排序模块处理 500 行夹具，启动准备 **27.196 s**，实际 Chrome 显示正确首末路径及 UTC 时间，**6.973 s**。`vue-tsc && vite build` 退出 0，**8.089 s**。 |
| 定向回归 | AutoLang `plan981_age_package_av` 16 例通过，**0.539 s**，锁定调试准备 **53.22 s**。AutoUI `plan114_widget_packages --test-threads=1` 18 例通过，**29.822 s**，终态准备 **1m16s**。新用例初版缺少受检视图触发条件，随后在旧 2 秒墙钟时于标量字段或计时器读取限时退出。带实际有条件计时器且统一帧预算后，单跑和串行组均通过。 |
| 原严格入口 | `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍于 `source.syntax` 退出 1，**2.893 s**。实际 AGE CST 的独立原源码片段通过头、导入、model、computed、design 与 view。对 50 个原处理器逐个投影，14 个片段被拒。`CommitListedDirectory` 的列表追加在 handler 语句位仍拒；`ToggleHidden` 的存储调用实参使用 `if` 表达式，改为语句式局部赋值的隔离对照通过。全原页的受检宿主和原操作仍待接入。 |

测试及工具准备使用 `--locked --offline -j1` 调试构建。材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-file-reply-original-browser.log` `1D7B8B4A159F05A693AA08FCE39D5BD969D5DF08EEC97BE0B9017C7A18B0F258`
- `r2-file-reply-original-strict.log` `AEA7DF68B6E697DDE7B611934BB3D1A580F5682C894302EDB74FA86C828782C8`
- `r2-file-reply-checked-browser.log` `E9051E72F714C05415C8F76C6A780BF3E0C750CF5CD54836731810E9B33692BA`
- `r2-file-reply-vite.log` `9029FFA716A67271CAA18E0922B4472520B2DBF88DCB4A59A7746E9D677E6941`
- `r2-file-reply-desktop-500-bounded.log` `1A3450CF5D8AE0FD5EEDEE11CEACFE4173522A22745F1190DE303473D7C9BA5E`
- `r2-file-reply-ui-regression-bounded.log` `88501B5A5F9C80E44BFE1BACC9ED5EC26746833E19B9BF1C558C65B166FCA21B`
- `r2-file-rows-500-size-current-1000000.log` `483C95787E2B056F277A00F71429BFA6E02133E9FAB87FFD4A9749A6D959DE23`
- `r2-file-reply-500-long-budget-6000000.log` `751F6DB03CE6B85724E3DBF06AB354B2DD6901F57D699BC36DF5847F5C7D38E8`
- `r2-file-reply-original-syntax-slices.log` `9A8C6D1A661C38658E55D452D66C705BAF45457B1E8A7C6CFBE812E2FB183767`
- `r2-file-reply-original-handler-syntax.log` `BBCD1A185470E9FF02A3FD3890C1A3B8D030B14728F42E38D1C45FDF95F6A3B6`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 42 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: a466eedbc950c45e1664305ce9142d3a9911f3c5 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 授权目录回复到行发布的受检链 | outcome: partial | AutoLang `3748de9a97a770039620eb551b3c4645ca713140` | AutoUI `b176e051ac2e75b8d30eca4ab1305f861ff4558f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将原 BootTick 与 Navigate 接入受检 file:list，再贯通原 CollectionView、写操作、多次导航和长名称压力。

授权文件会话随原始目录元数据返回 Unicode 小写扩展名。声明 `extension str` 的受检目录回复在桌面和 A2Vue 带该字段，既有五字段回复也通过回归。原 `CommitListedDirectory` 调用同模块的类型化行准备函数，从真回包完成筛选、500 行截断、排序及行构造，统一提交计数、大小、路径和路径选择状态。普通受检函数返回的同名拥有记录可作为新临时值直接入表。已绑定的局部记录仍由 `.move` 规范移交。旧 Vue 生成器沿本文件函数收集显式导入的调用依赖。受检 AV 标量读取复用视图和计时器已有的可达入口裁剪，并继续对每个执行入口进行验证。

| 验证 | 结果 |
|---|---|
| 受检 AV 真授权回包 | `file_manager_rows_and_path_are_published_only_after_an_authorized_read` 使用原目录回复、纯行准备、路径历史、发布和选中处理器。真实隔离根含 `资料.É` 及 `.profile`。实际 `NodeDesktopBridge` 显示 `é 文件`、`1 个项目`、`5 B` 及原折叠路径。越界回复保持路径、历史和行。展开后显示原中段。可达入口裁剪前隔离单例 **33.690 s**，裁剪后最终 **9.843 s**。入口裁剪前的语义回放通过，但超过单例时限。 |
| 原 027 兼容 Vue | 用实际最终编译器启动字节一致的原项目和后端，准备 **22.060 s**。Chrome 验真实 Unicode 扩展名、2.5 GB 稀疏文件、2040 时间、目录优先、大小双向排序、保留选中路径、中文搜索和隐藏项，**5.143 s**。中间的生成产物暴露了本文件函数间接调用时漏发 `listed_file_row`，改依赖收集后在新材料目录回验。原 `app.at` SHA256 `1FD38ECE36CA16705510D8FAAF3FF12798F71545063EA6FE24383E834D6222D2`，`api.at` SHA256 `BFF9C1ED0843A0D610CA2B0AF4DF53A42A7ECF4784F52E7BBB44CD9584897707`。 |
| 受检 A2Vue 真授权回包 | 受检夹具直接用原回复接点及后端，实际 Chrome 从授权根读取并渲染原行。捕获真实越界回复及页面消费其正文，核对拒绝前后呈现文本一致，**1.191 s**。最终热启动 **4.381 s**，首次启动 **13.155 s**。`vue-tsc && vite build` **4.720 s**。初版浏览器探针把合并的文本节点当成独立元素，改为验证可见文本和真实拒绝消费后通过。 |
| 源检查与回归 | 原目录回复和隐显切换两段的 AGE 语法独立通过。50 段独立语法探针由 14 个拒绝降至 12 个，**0.180 s**。原函数与处理器闭包通过受检前门 **0.279 s**；隐式转移已有局部在 **0.201 s** 返回 `source.move-required`。Core `plan981_age_package`、`collections_rejects` 各 13 例，**0.165 s**、**0.087 s**；AV 同职责组 16 例 **0.268 s**；文件会话 2 例 **2.956 s**；`plan114_widget_packages` 18 例串行 **24.719 s**。 |
| 原 027 严格入口 | 实际 `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍于 `source.syntax` 退出 1，**2.551 s**。原 VM 与 Vue 混合的 BootTick/Navigate 请求与文件动作、CollectionView 回调仍待接入原严格入口。 |

工具、测试准备使用 `--locked --offline -j1`，调试档。AutoUI 完整开发工具准备 **68.372 s**，消费测试初编 **215.230 s**，AV 裁剪后最终 AutoUI 准备 **1m20s**。Core/AV 首次准备 **34.488 s**，最终 AV 准备 **30.20 s**。材料根 `D:/autostack/plan-artifacts/plan-114/`。主要证据 SHA256 如下。

- `r2-file-materialize-original-browser.log` `331389B6E6B191FA11065F1509990513A941BD0B3CCF240E66B4D118AA51516B`
- `r2-file-materialize-checked-authorized.log` `E58CA2BCE24B0F15E6C36E22BC67B4755079CE1312F9BF0F3EFAD4FD7DDECECD`
- `r2-file-materialize-original-strict.log` `D72E5DC9C9C5CC5B294CF2BF01C60D2C9B5D5FB77C985993056150A503A05AA4`
- `r2-file-materialize-vite.log` `BEED3E66332D452F26E7DF351B012A73E7F8690242802EBCDBB9D5E217EDA26C`
- `r2-file-materialize-desktop-isolated.log` `F0F074E7DC53EF29DB76EC6C920C139D94845D2ED1FC88C855B82D6CEB8B2B92`
- `r2-file-materialize-desktop-pruned.log` `383A6E39D5E0E77DB0848038F2E50040F5B8A51A9931FE175D7669A3FA77F4AB`
- `r2-file-materialize-ui-regression.log` `D94C42C021033FA4000FE1850C8BB7D17C7C9EBF1F6FF62281749B0D707654DB`
- `r2-file-materialize-session.log` `D88B5F0DEEA339D07189FDCBC93FABE00C0CD1709A8C9F33D2D39A9CCFCAD19F`
- `r2-file-materialize-handlers.log` `42BED77A83CD731AF0863FAC60D08DF3B79804D46F4794844EACC0DEACB12613`
- `r2-file-materialize-implicit-transfer.log` `B30463814F65D9DB472654FFE3B20511C98FD0DD6379812642995C808F4631AF`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 43 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: b176e051ac2e75b8d30eca4ab1305f861ff4558f | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 启动和导航共用受检目录请求 | outcome: partial | AutoLang `ed01c30a884a487aec268566489319b0841e85db` | AutoUI `5b0932bc96446abc442dc96ebd1a384d93179aa6` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 接完原整页的 BootTick/Navigate 宿主入口和 Init，再迁移 CollectionView 回调、实际写操作及长名称压力。

原 027 声明 `file:list` 绑定。`AdvanceBoot` 管理启动计时并提交授权根请求。`RequestDirectory` 记录目标、候选历史索引和请求版本。`DirectoryListed` 用成功回包更新路径、行、历史与选中，首个根回包还生成主目录、真实存在的快捷访问和授权盘位。旧 Vue 的启动与导航调用同一套请求和回包处理器，启动中的合法新导航可接管目录视图。桌面 AV 在受检事件中先读取请求标志和路径，授权读取完成后渲染并提交包含回包的帧。

| 验证 | 结果 |
|---|---|
| 原 027 真盘 Vue | `auto run --render vue --server vm` 使用原项目字节。启动只发一次目录请求并显示主目录、Documents 快捷项、2.5 GB 稀疏文件与 2040 时间。前进、后退、拒绝保持、恢复、分支替换、迟到导航及面包屑回放 **5.891 s**，冷准备 **26.483 s**。独立回放延迟首个根目录回包，先跳到已授权子目录再交付根正文，子目录保持，**5.497 s**，热准备 **13.085 s**。中间旧生成器用例发现 `reply` 是保留字，改局部名后以新材料路径回验。原 `app.at` SHA256 `ACEE5CA00EFDA6A3D3A30B0ACB79CC3C9E8096F9CCB5B687C1258ACAD0A792BC`。 |
| 受检 A2Vue 真盘 | 受检夹具使用原 `AdvanceBoot`、请求、回包、行发布及路径历史处理器，与原后端配合。实际定时器在最后一个计时节拍读取授权根。Chrome 核单次 home/list、无额外 drives 请求、真实快捷项和盘位。捕获越界拒绝及真实迟到 Alpha 正文，页面保留 Beta 且历史只提交一次。回放 **2.214 s**，准备 **14.987 s**，`vue-tsc && vite build` **4.013 s**。 |
| 受检 AV 与回归 | `file_manager_rows_and_path_are_published_only_after_an_authorized_read` 从原启动阈值触发真实授权根，检查原行、快捷项、盘位及拒绝后的路径与历史。优化前单例核心 **12.790 s**，优化后首次观测外层 **11.687 s**，隔离复验含调用 **7.727 s**。`plan114_widget_packages --test-threads=1` **18/18，26.837 s**，AutoLang `plan981_age_package_av` **16/16，0.354 s**。 |
| 源前门与整页 | 52 个独立处理器片段中 12 个语法拒绝，**0.130 s**。原 `AdvanceBoot`、`RequestDirectory` 和 `DirectoryListed` 通过独立语法且与真实行和历史一起进入受检闭包，**0.334 s**。原整页 `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍返回 `source.syntax`，退出 1，**2.487 s**。`BootTick` 与 `Navigate` 的旧宿主调用和剩余操作仍需迁移到严格入口。 |

测试准备使用 `--locked --offline -j1` 调试档。最终 AutoUI 准备 **2m18s**，最终 ACE AV 准备 **24.43 s**。材料根 `D:/autostack/plan-artifacts/plan-114/`。主要证据 SHA256 如下。

- `r2-file-request-original-browser.log` `7676FCA282CACC0297B3E3C61A8AD717E050D7361347E8124DA897FE61605221`
- `r2-file-request-startup-race.log` `A808D88FD0362F5BA4E0EF116DC5655F6ADEE5DDE010353B8F4B37404B17EA1F`
- `r2-file-request-checked-browser.log` `3CEB5780233A172CE2CC744BD93F0CBF4F9C20FEE9B63E9F4DC49476EEC8D9CE`
- `r2-file-request-vite.log` `4EAAFCC0193731769063AE4719335F36ECCBFEFA6D0908AACFAACB260A1DD663`
- `r2-file-request-desktop-final.log` `ED42173791C2B122A7AFFBCF9857F8FDA88631D6449B2F4D0F39BA573D0695A0`
- `r2-file-request-desktop-optimized.log` `308F7F1CE5067592C88158CD5DE10E3F84B8873895EA4DA484321F06DFAA8BE8`
- `r2-file-request-ui-regression.log` `0B0E869B05C826F67F8ACEB0C521C871E082901BEEA1A804055E0D664FFA13A3`
- `r2-file-request-handlers.log` `A8A6975FCAC878DC846AFD22DCDA0A615B48BB3264C0C08DCE02C13CD38D7E46`
- `r2-file-request-original-strict.log` `BAA943090B807FCF0B5B21F29C05DF39D494032650A3E3D33AD5AA98FDF01481`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 44 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 5b0932bc96446abc442dc96ebd1a384d93179aa6 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的兼容 Vue 目录读取交由宿主绑定 | outcome: partial | AutoLang `ed01c30a884a487aec268566489319b0841e85db` | AutoUI `e1630c757efedccec04943497f8d9c18959a9637` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 迁移原页面 Init、原生 BootTick 和 Navigate，再迁移 CollectionView 回调和授权写操作。

原 027 的 Vue 启动和导航由 `AdvanceBoot` 与 `NavigateInSession` 提交 `file:list` 请求。兼容 Vue 生成器监听请求版本，调用原后端 `fs_home/fs_list`，经 `file:root` 通知页面授权根，再经 `DirectoryListed` 提交当前回包。无根时沿用静态演示。新请求、取消或页面卸载会使旧回包失效。拒绝反馈现在显示在状态栏，成功视图与历史保持到下次合法提交。

| 验证 | 结果 |
|---|---|
| 原 027 真盘 Vue | `auto run --render vue --server vm` 启动目录只读取一次。Chrome 回放 2.5 GB 和 2040 时间、面包屑、前进后退、拒绝后重试、替换历史分支和迟到回包，**8.123 s**。独立交付迟到的根目录回包，新打开的子目录保持，**4.396 s**。授权启动准备 **41.917 s**。 |
| 原 027 无授权根 | 启动环境省略 `AUTO_FILE_MANAGER_ROOT`。Chrome 确认始终只查询一次 home、无目录列表请求、显示 6 行演示及文件大小，**5.124 s**，启动准备 **12.152 s**。显式空路径仍由启动器拒绝。 |
| 受检 A2Vue 真盘 | 夹具调用原 `AdvanceBoot`、`NavigateInSession`、`DirectoryListed` 及行发布和历史处理器。原后端授权根、快捷目录、盘位、越界拒绝和迟到 Alpha 回包通过，Beta 与已提交历史保持。Chrome **4.088 s**，冷准备 **58.903 s**。`pnpm run build` 首次 **11.996 s**，隔离复验 **5.914 s**。 |
| 受检 AV | `file_manager_rows_and_path_are_published_only_after_an_authorized_read` 调用原 `NavigateInSession`，核真实授权根及拒绝时的反馈、旧路径和历史。单例 **9.42 s**，`plan114_widget_packages --test-threads=1` **18/18，31.69 s**。AutoUI 测试准备 **2m16s**。 |
| 原 027 严格入口 | `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍需迁移原生直调、`Init`、CollectionView 和写操作。当前在 `source.syntax` 退出 1，**4.452 s**。原 `app.at` SHA256 `769AB87E77329987AD1DA5D31821AEF9CBA9D6B18FF9C8A6D0376EE781560B67`。 |

主要证据保存在 `D:/autostack/plan-artifacts/plan-114/`，SHA256 如下。

- `r2-legacy-file-binding-original-browser.log` `C7986D1FB83F3D9D2B2AC58F3172E56DDCECFFE3F3C62B4D3CCB111D9ED7F573`
- `r2-legacy-file-binding-startup-race.log` `716B5418561AA2C4186EA3D52A7DF2B78EBFDCC4D733D90B042A3BD43366A3DF`
- `r2-legacy-file-binding-original-offline.log` `50881D222FD284962027A357577554CA0D62BFC692E658BBEA9093DD2A4B5467`
- `r2-legacy-file-binding-checked-browser.log` `15603E5900543EFA7D0E71E714FF5B02CB9482FD85863985BF78F32285260745`
- `r2-legacy-file-binding-checked-vite-isolated.log` `8D60AA579EB7E105E8199103E7420C214AFF72D253BF04397F320A0F76BBBE8D`
- `r2-legacy-file-binding-runtime-focused.log` `98E64687B5D25BE87536FD7A2DA853FFE84992114F735C7320D981DE98E0DA07`
- `r2-legacy-file-binding-runtime-group.log` `3BB0FCEAB6FB45DB35AD506B66B0CA11A1F8CC8A93D8082FED5AED791428F82A`
- `r2-legacy-file-binding-original-strict.log` `DF67107C4729A8AC33E34CEB37FB4880511E66D9FB14B97B8001175DA69C0126`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 45 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: e1630c757efedccec04943497f8d9c18959a9637 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的兼容 Vue 写操作接入授权会话 | outcome: partial | AutoLang `44632c73992f05ed324d5f4a3be77df140453d3e` | AutoUI `f89425278972b178d8f682654a80c9bd391cf98c` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 将 `file:mutate` 投递接到 AV 和 A2Vue 受检宿主，继续原整页启动与 CollectionView 迁移。

原 `src/back/api.at` 声明 `POST /api/fs/mutate` 并委托 VM 的 `fs.session_mutate`。VM 沿当前文件会话执行操作，沿用已持有的目录句柄以及逐操作、逐路径检查。Vue 启动使用宿主设置的 `AUTO_FILE_MANAGER_ROOT`，默认是只读。宿主可用 `AUTO_FILE_MANAGER_WRITES=create,copy,rename,delete` 逐项授权，浏览器文档的连接继承启动权限并独立撤销。原页新建、改名、粘贴、删除提交 `file:mutate`，兼容 Vue watcher 调用生成的 API 并以布尔回包通知 `MutationApplied`。一次只提交一个写请求，成功按导航版本刷新原目录，失败保留目录视图和选中。Vue 启动在前端增量编译前刷新 API 清单，一次运行即可识别新增 API。

| 验证 | 结果 |
|---|---|
| 原 027 Chrome 真盘 | `auto run --render vue --server vm` 使用原 `app.at` 与 `api.at`。保留旧清单的输入首次暴露 `Unknown API function 'fs_mutate'`。调整流程后以同一旧清单逆向复验，在一次启动中写出本轮 `App.vue`。授权准备 **14.428 s**。新建文件与目录、重命名、删除文件及空目录首次 **10.281 s**，同流程单独复验 **6.995 s**。跨目录复制与剪切移动核文件内容、源保留或移除、剪切状态清除，**8.093 s**。延迟交付已真实写盘的回包期间切换目录，较新视图保持，**5.755 s**。 |
| 原 027 默认只读 | 省略 `AUTO_FILE_MANAGER_WRITES`，原文件选中后尝试新建。后端返回 `false`，状态栏显示失败，列表、选中与磁盘保持，**4.029 s**。启动准备 **11.033 s**。 |
| 真实后端和权限 | `browser_file_actions_follow_the_current_root_and_operation_grant --exact --test-threads=1` 核默认只读、仅新建权限、跨站拒绝、复制和改名双侧越界拒绝、真实写入及关页撤权。用例 **1.28 s**，外层 **1.627 s**。同 `plan114_file_session` 三例 **3/3，4.58 s**。`cargo test --locked --offline -j1 ... --no-run` 准备 **1m59s**。启动器首轮编译 **1m58s**，最终 `cargo build --locked --offline -j1 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` **1m00s**。 |
| 受检前门和整页 | AGE 用原 `child_path`、`RequestMutation`、`MutationApplied`、`RequestDirectory` 的闭包通过语法和类型检查，**0.227 s**。原整页 `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 仍以 `source.syntax: expected fn, found widget` 退出 1，**2.282 s**。AV 和 A2Vue 写宿主投递尚未贯通。原 `app.at` SHA256 `E8CB6FAC6F8848A9D47EA80AC06E994728A66F8F38B170E19435E85E2F7FA46E`，`api.at` SHA256 `C5CC3241D5EBBDE138A0040245D4CA72E70A1FB83391B192EFE3D56082B19611`。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-browser-write-original-authorized-launch.log` `69AE600EE08A88FC4B7B5EE68135041DEED08CAAE4DCF0D4769086812BF12C3D`
- `r2-browser-write-original-create_rename-final.log` `5AA5272F2E4E73822F3C1526080695718BD2FB96C0C2DD4739FD86B2F395F922`
- `r2-browser-write-original-copy_cut.log` `16DCD350555D6323A0D5CE358EC1D7FD12BFAE71E5037E4CCDACE47A2EB2D004`
- `r2-browser-write-original-read_only.log` `28761EB96EF5136C935C2B96845DB6E9C35BC3051DFE0C5AABB3C6A6B6D086F2`
- `r2-browser-write-original-late_reply-final.log` `0B78A5C26C99F7EA90A8BBED0A3835569A85BD2ECD7C4602ADDBB362D2248B2F`
- `r2-browser-write-runtime-focused.log` `5857A91551DFB5F9A9DEB3A808EF3900044E37B7848C618559FB46844B82E626`
- `r2-browser-write-runtime-group.log` `1542887C2FF7229E09C853F84FB0848371F1716A09BC44605CA46BB1D396F4D1`
- `r2-browser-write-checked-source-final.log` `0E876D5F3B8F4CD10DC4B2040F0A84B6E02AD1039E68AAF75546009661B12D9F`
- `r2-browser-write-original-strict.log` `14405CA55389602D5ADB1D9D05769DCE9667FCA0273D46A1F3D643851B0DA622`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 46 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: f89425278972b178d8f682654a80c9bd391cf98c | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 AV 和 A2Vue 受检宿主执行 `file:mutate` | outcome: partial | AutoLang `e788f65509e34d14ba73553353e5d4cdff5d7643` | AutoUI `faf29e101023d7939ebae3ecc15d41d6fd9a281b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 定位并修复原 027 整页严格入口的首个真实语法拒绝，继续原生初始化、导航和 CollectionView，接回已验证的写宿主。

受检宿主校验 `file:mutate` 的布尔回包与模型字段。AV 先验证能提交成功及失败两种回包，再用当前实例的授权会话执行一次。场景重放使用已提交回包。即使后续目录刷新的渲染失败，下一次派发仍保留该回包。窗口工具可通过 `--file-root` 与 `--file-writes` 显式配置授权。A2Vue 经页面连接向原后端发送一个 POST，导航请求在途时先提交导航，再将写回包交给当前实例。页面卸载时取消等待，提交过的写请求不会因刷新或重绘重发。

| 验证 | 结果 |
|---|---|
| AV 定向用例 | `checked_file_mutation_commits_only_a_live_authorized_event --exact --test-threads=1` 核默认只读拒绝、显式授权后删除、同路径重建后的事件重放、关闭后授权拒绝及回包不清请求时在写前拒绝。**0.77 s**，外层 **0.935 s**。`plan114_widget_packages --test-threads=1` **19/19，19.22 s**，含原授权目录回归。`plan114_file_session` **3/3，5.93 s**。 |
| Iced 实窗 | 原生窗口工具运行受检的定时器写入夹具。两个已提交事件显示写状态成功且重绘计数为 1，磁盘存在空文件，窗口实际绘制 **11 帧**，**3.960 s**。先前的 `--script click` 探针绘出 45 帧，旧按钮注入未进入受检事件队列。本轮实窗写入证据使用受检定时器入口。 |
| 受检 A2Vue 真盘 | 夹具使用原 `RequestMutation`、`MutationApplied`、`child_path`、原导航与发布处理器，以及原 `api.at`。Chrome 核写后刷新且只发一次 POST，**1.823 s**。双向延迟真实回包，在写回包先交付时保持较新导航，重绘前后共一个 POST，**2.294 s**。默认只读拒绝且不刷新列表或写盘，**3.379 s**。服务准备授权 **38.556 s**、只读 **12.148 s**。`pnpm run build` **13.543 s**、复验 **10.891 s**，直接调用相同 `vue-tsc` 与 `vite build` **7.116 s**。 |
| 构建与原整页 | AutoUI 定向回归准备 **2m45s**，修正测试输入后准备 **44.25 s**。实际工具与窗口 `cargo build --locked --offline -j1` 准备 **47.27 s**。原 027 整页严格生成仍返回 `source.syntax: expected fn, found widget`，退出 1，**2.850 s**。原 `app.at` SHA256 `E8CB6FAC6F8848A9D47EA80AC06E994728A66F8F38B170E19435E85E2F7FA46E`。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-checked-write-av-focused-final.log` `DB65BF08607CBD25A5CDFF7D5F5703479035AAB16E735DFCD9E0B3C819A00325`
- `r2-checked-write-av-group.log` `876093DB0395EC53766F75039A34127E503493843110511DA57B31CE6664E2B9`
- `r2-checked-write-file-session.log` `9567D6AE48CB9BC645DCA362330628223B4797B4F6995D01048A9ADBAA71DE1C`
- `r2-checked-write-window-evidence.json` `B03CF30CC74B899E9761474C12F55D4D1C76BF0F830576039135D8A26CBD014B`
- `r2-checked-write-browser-refresh-final.log` `15BEC1082F018F971B73E51E217F77F6FD57F300432E5E0ADE0E89DD195B8184`
- `r2-checked-write-browser-late-final.log` `4D1EA9AAB41FFB93AEF236FD309395B511006626F8EF4203BD16A697990BACC2`
- `r2-checked-write-browser-denied.log` `09B59AC531C326EEF215BE039FEB770DD516C8B456C9EB694A7CE303E57A2F3C`
- `r2-checked-write-browser-build-direct.log` `D5FDC1334A8D9076C4817754F4F896182D806855D5BEE94469BF3DC2CC5F4B80`
- `r2-checked-write-original-strict.log` `F43F71796715A86A226D7964F9CD4B0668B586285CF6699EFBA5FBA8F1FE7B8D`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 47 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: faf29e101023d7939ebae3ecc15d41d6fd9a281b | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 启动和目录读取迁移至具名类型 | outcome: partial | AutoLang `13e843046d102139bbe16972bd15e89862d88ff7` | AutoUI `97e6c912ef422fb318c8eba4a41e8c0bc26ab634` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 修复原 VM 的重复累加和整数宽度，迁移原 CollectionView 事件及文件操作入口，再重跑原 027 双轨前门。

AutoLang 的 `fs.read_dir_names` 返回含隐藏项的具名字符串列表，沿用旧枚举顺序和出错语义。有请求根目录时沿用原路径授权检查。原 027 的 `.BootTick` 与 `.Navigate` 采用具名的快捷入口、盘符和目录行，读取失败时保留现有路径，已知路径错误由状态栏反馈。复制和剪切的反馈沿用剪贴板状态栏。Vue 生成器现在会追踪记录字面量字段中的辅助函数调用，原页复验覆盖 `sort_files`、`type_label` 和 `format_modified` 的发射。

| 验证 | 结果 |
|---|---|
| 原整页严格前门 | 从精确原源分离的事件探针由 **12 个拒绝降到 8 个**，`.BootTick`、`.Navigate`、`.CtxCopy`、`.CtxCut` 单独通过。剩余 `CollectionMode`、`CollectionSort`、`CollectionItem`、`OpenItem`、`CommitRename`、`CtxPaste`、`CommitNew`、`ExecuteDelete`。具名目录模块单独通过包语法检查 **0.059 s**。原 027 整页严格生成仍退出 1，报 `source.syntax: expected fn, found widget`，**2.508 s**。原 `app.at` SHA256 `36A84375668B69B4C5478E5B5B663453ADAD6118E185DAA110B5302768C28A9C`。 |
| 兼容 Vue 原页与受检组件 | 原页 Chrome 真盘 **5.211 s**，验证稀疏文件 `2.5 GB`、UTC 2040、单次启动读取、父级导航、面包屑、失败后保存视图与历史、重试、前后与迟到回包。Vue 记录字段用例通过 **0.13 s**。受检组件 **19/19，18.96 s**，文件会话 **3/3，3.60 s**。运行工具构建成功，重试 **34.78 s**，最后定向测试准备 **17.14 s**。 |
| 旧 VM 实窗与目录探针 | 原 027 窗口显示主目录、文档和盘符，隐藏项切换后 7→9，子目录及无效路径后保留视图，返回后 9 行及行级元数据仍可读。旧 VM 将 2.5 GB 显示为 `1.9 GB`、2040 显示为 2038，且多文件合计未通过。真盘独立探针中 9 个文件逐行正确显示 `1 B` 至 `9 B`，总计为 `1 B`，期望 `45 B`。直接 `file.size` 和 `file.is_dir` 探针确认其逐项输出正确，剩余问题涉及旧 VM 的宽值与汇总。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-original-handler-localize-pre.log` `95C0D4AC20D9A0C29725BBE2BA2AB410B33A4580D4A848DC453359A0AC31797A`
- `r2-original-handler-localize-native.log` `5B62F35260CE3517B35EFE229F3AEB9B1639D8C9E75AF80531FBD9967F88E484`
- `r2-native-directory-package-final.log` `B3371C05835F062CB0FED1BF11216B83A4DD9697BD632E2F5452EFE1E7732642`
- `r2-native-directory-vue-record-test.log` `2530303408F0053D57B49BED3A478FD770CD38D14E49E14719436EC50F94DD8F`
- `r2-native-directory-original-vue-browser-final.log` `1B144EF4F12821952478053E6CD55078E4DAE9B9B0A6F719208511E473B8F674`
- `r2-native-directory-original-vm-browser.log` `EE5236B7B5A596949143AA18A55CA1745C2BCD8D6BEEB822FB06B363CDD4EE48`
- `r2-native-directory-vm-probe-final.log` `578DEF3F9BEAD54EFDA9C07A5C1B284CB500D664708E981928C37D854F14617F`
- `r2-native-directory-ui-group.log` `FC6D45340D60B6176899610AFC13D310204BCC882F0D93091D5D835AC2D42DBB`
- `r2-native-directory-file-session.log` `24EF7329626E19E970AFF73CD9F56263A3C5C967968CFE4DDDF7E4FE17726FAF`
- `r2-native-directory-original-strict.log` `54703F3EFD06E064915B3D40A787D55AA59B20E89BB9ACDE444FDCC80C467F44`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 48 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 97e6c912ef422fb318c8eba4a41e8c0bc26ab634 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检包读取 CollectionView 的现有 JSON 载荷 | outcome: partial | AutoLang `f60a38e311eef5a932e383cb8c3e855b6358075f` | AutoUI `3a35eefb80a4a570619e31599d5ecf2d23637a25` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 补齐旧 VM 与兼容 Vue 的 JSON 解码接面，将具名字段读取接到原 027 的 CollectionMode、CollectionSort、CollectionItem，并重跑原路径。

受检 AGE 包在实际引用 `std.json` 导入时加载仓库内现有 JSON 标准库，拒绝非冻结的同名 STD 输入。标准库用既有 packages 文法投影，与调用方同图绑定和检查。入口继续使用 AGE 文法身份，包输入身份计入标准库源和其文法。CollectionView 的 JSON 字符串合同仍用于测试，字段覆盖 `instanceToken`、`scopeId`、`baseRevision`、`itemRevision` 和 `id`。原 027 的处理器尚未切换。

| 验证 | 结果 |
|---|---|
| AutoLang 包绑定和身份 | `cargo test --locked --offline -j1 -p ace-core --lib source::packages::binding::tests` **9/9，0.12 s**。验证未引用不加载、引用后受检、伪造 `std::json` 拒绝和 AGE 文法身份。相邻 `plan981_age_package` **13/13，0.14 s**，`plan981_grammar_scope` **4/4，0.37 s**，conformance 准备 **4.10 s**。 |
| AV 与 A2Vue | `checked_collection_events_decode_and_reject_stale_json --exact --test-threads=1` **1/1，5.30 s**。AV 实际派发后，截断 JSON、错目录、旧实例、错版本类型、旧结果版本、旧条目版本保留选中项，合法的 Unicode 转义文件 ID 更新选择。A2Vue 使用同一父处理器，Chrome 从受检子组件触发六种无效和合法请求，**1.322 s**。`pnpm run build` 退出 0，**4.476 s**。运行工具构建 **1m34s**，最终定向测试准备 **27.95 s**。 |
| 回归与未通过入口 | `plan114_widget_packages` **20/20，25.81 s**，`plan114_file_session` **3/3，6.72 s**。将冻结 JSON 源原样交给旧 VM 的真实后端探针，模块编译报 `Undefined variable: char`，返回 503。原 027 `app.at` SHA256 `36A84375668B69B4C5478E5B5B663453ADAD6118E185DAA110B5302768C28A9C`，整页严格生成仍退出 1，报 `source.syntax: expected fn, found widget`，**2.409 s**。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-collection-json-age-binder.log` `DCCB9AE16A1DFFC4799BB45CEACDC5F64C13AEF1F2344C67389CB920C02F7B99`
- `r2-collection-json-age-neighbors.log` `C991E9D1E3C778F7ADAD1959C5419F3A8D674BCA40DACE6B7AB3692EF8404342`
- `r2-collection-json-grammar.log` `1FC82FE21EFA8D7463869F4051D6FD987056470F4A5B2E0A13309237B6164399`
- `r2-collection-json-product-focus.log` `37D0EAEEEFA05367ADA9B2778BE983692A0C94C09162DA4FB7BDFB90771FFFA7`
- `r2-collection-json-product-group.log` `B8C8C2F4952DBD0359EAAA8DE1FAD85956295E57B19E372AE8D7416C14ADF185`
- `r2-collection-json-file-session.log` `65F7D0A90700DF281412D1F9B397C621BE9A70E551A83526B6DF6AFC4C50C274`
- `r2-collection-json-browser-final.log` `DE51E8167CC433321305BB0CA42ECFC85C20FCBFF018B7B7D9E22B2BC4850480`
- `r2-collection-json-browser-build.log` `7E850291BEA52CB946A2A8059A14CB1B73B08D5A6EE42BC04086ED11DB76742E`
- `r2-collection-json-vm-probe-launch.log` `BCAD341FD1C466769E3D2BCE29F4DA683BF055C0B306ADD83CFCE25B6DBEB34D`
- `r2-collection-json-original-strict.log` `62F4F79407B45E097254367ADD74698DE2E8913863DE5AFBB948E40E350081C0`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 49 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3a35eefb80a4a570619e31599d5ecf2d23637a25 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 CollectionView 事件接回类型化 JSON 读取 | outcome: partial | AutoLang `6ae52495731bd7db96bed5d0f9f740f0224b26b8` | AutoUI `5736fcb609386e14216db093cfe80a44fdca068c` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: T-02 优先验证并接入受检 AV 的当前模型转换，消除同一实例的累计重放内存拒绝，再接完原整页五处文件操作。

原 `CollectionMode`、`CollectionSort`、`CollectionItem` 现在从同一 JSON 字符串合同读取明确类型。目录、实例令牌及结果版本共同决定接收，条目事件另核当前条目版本和当前路径对应行。`std.json` 的根对象读取函数在受检包复用仓库 JSON 标准库，在旧 VM 和兼容 Vue 使用各自宿主的 JSON 能力。整数按原始十进制字面量和 i32 范围读取，避免小数、指数和溢出值碰巧匹配版本。重复成员取最后一个值。文件名转义、空字符串及回退值由定向用例覆盖。

| 验证 | 结果 |
|---|---|
| 原 027 严格前门 | 独立语法拒绝从 **8 个降到 5 个**，剩 `OpenItem`、`CommitRename`、`CtxPaste`、`CommitNew`、`ExecuteDelete`。整页严格生成仍退出 1，`source.syntax: expected fn, found widget`，**2.465 s**。原 `app.at` SHA256 `1DF832B10E85A9FA93B92CDB7CF2F316A2552B206E7D610769F607364F41BFE5`。 |
| 原页旧 VM 与兼容 Vue | VM 原 App 真实窗口由 MCP 派发载荷，核转义 Unicode 路径、9 种无效载荷、跨实例拒绝、合法换模式与菜单、排序后旧版本拒绝，**9.459 s**。兼容 Vue Chrome 从实际控件选中、换列表、排序保留选中、开上下文菜单与进入目录，终态并发启动下 **40.794 s**，前一候选 **5.224 s**。同一终态全页导航回归 **5.345 s**，包括稀疏 2.5 GB、2040 时间戳和迟到回包。 |
| 受检 AV 与 A2Vue | 提取原父处理器，保留实际 JSON helper 和真实行身份。AV 定向覆盖十种拒绝后恢复合法选中及各渠道作用，分六个独立挂载执行 **34.21 s**。受检 Chrome 在同一挂载从子组件派发十种被拒载荷，随后合法 Unicode 选择通过，终态并发启动 **40.977 s**，前一候选 **1.569 s**。终态 `vue-tsc && vite build` **3.529 s**。该夹具的文件操作接收方只验证路由。 |
| 累计 AV 重放缺口 | 将十种拒绝连续送入同一挂载时，`PackageStateEngine` 的累计场景在 `selected_id` 状态函数触发 `VerifyMemory`，需约 **17,954,752 B**，超过固定 **16,777,216 B**。保留同实例复现源和日志。现有 `compile_age_package_widget_runtime` 与 AV `execute_program_entry_args` 是下一轮的验证入口，需保证模型、目录和写入回包在实例关闭前保持现有授权及事务语义。当前性能验收仍开放。 |
| 相邻回归 | AutoLang 包绑定 **9/9，0.14 s**，`module_1064` **6/6，0.10 s**。抽取原 Rust shim 函数和测试，用锁定的 serde_json `1.0.154` 离线运行 **1/1**，验证 `-0` 与 i32 边界、指数/小数/溢出拒绝及转义 Unicode。AutoUI `plan114_widget_packages` **20/20，74.34 s**，`plan114_file_session` **3/3，7.67 s**。目标缓存被清理后，最终测试准备 **7m48s**，工具构建 **5m20s**。 |

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-collection-original-handlers.log` `B3E443AD91140D23E755953B4DD0D66CF7B189407928A36FF7554A45EFDC677A`
- `r2-collection-wire-core.log` `0052812F08F3FCF551CAA83B66A95CEB2C1C044BD3AE71D51A0452426E83DBBE`
- `r2-collection-wire-json-neighbors.log` `1E8CE97959980FA3210F2AB84F329736785F78C6CEBDD06785C40A211A20FEA6`
- `r2-collection-wire-shim-unit.log` `5B39139BAABBE88DF2FA421FEA685A1BD4C37D220709832343D808D583A0AA1A`
- `r2-original-collection-native-events.log` `28B88EAF1171070C1869A5A731D09294402B83B7ED9A757698FB6838E21FC18C`
- `r2-original-collection-vue-events.log` `96BF5335E94413053AEC4C6118A882E29C71C48E9B605D18B575D763D640A92F`
- `r2-collection-wire-original-navigation.log` `3C8E9C34AE9E35871F99CAD62A3E8B06DA4ED5C2130A8B64A66D49ABA7F07AD3`
- `r2-collection-wire-checked-vue.log` `2653F83BE74CD0A7809A96BF4DEF0D8ED3D3CD01F063799EC1DAB3C6BB57F4AB`
- `r2-collection-wire-checked-vue-build.log` `99EE72C8CB8F20C16309DEFDECFA64BC7F7033B1B361DF70F17C8B36F20C7177`
- `r2-collection-wire-checked-focus.log` `599B8D2DB5AE22A5D88B6C2853EE8D488FC91999F90FDBD7C273831692EF7AD6`
- `r2-collection-wire-widget-group.log` `65033F00566D2FA9975A8ECCCB09D5919BCCFB0DE79A5E9CCBF09BDCC5FF2C9D`
- `r2-collection-wire-file-session.log` `4ECF9BB0289339DF21842644EF0C7FB3BF02E8FF3DFAE22225F80A02E57982E2`
- `r2-collection-wire-av-cumulative.log` `3D6E73F7BEC73FE22621484B6F31E47C41B9E16F451B041E0B1F54C7ABF523AE`
- `r2-collection-wire-original-strict.log` `B7B60A28A3B2F127740AA2D05C01558DCCCE34C202C90D6375CC8D541B1AC31D`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 50 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 5736fcb609386e14216db093cfe80a44fdca068c | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 单挂载 AV 使用当前模型 | outcome: partial | AutoLang base `6ae52495731bd7db96bed5d0f9f740f0224b26b8` → `24d630e29eeae431c3d1ae3f3fecf6061004a1ba` | AutoUI `5736fcb609386e14216db093cfe80a44fdca068c` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev` | next: work/T-02。

AutoLang 在同一挂载的 AV 会话内保存受检模型记录。源码经既有包前门检查后，初始化、视图、计时条件与模型转换被编译一次，挂载持有独立的模型与包全局变量，派发基于当前模型执行。公开 AV 派发继续检查标量和作用域文本，记录与列表留在经过 MIR 校验和单元链接的模型内。处理器陷阱或拒绝恢复 AV 全局快照，保留闭包环境。

| 验证 | 结果 |
|---|---|
| AutoLang 单元 | `cargo test --locked --offline -j1 -p ace-av --lib` **20/20，0.24 s**，准备 **7.53 s**。新增单项独立运行 **0.08 s**，核两挂载隔离、128 次连续派发、原初始化和计时语义、陷阱与错载荷回滚、Unicode 与列表视图。既有拥有闭包的包全局用例新增失败事务后读取闭包断言。 |
| 相邻包语义 | `cargo test --locked --offline -j1 -p ace-av --test conformance plan981_age_package_av -- --test-threads=1` **16/16，0.30 s**，准备 **7.96 s**。保留目录记录、两实例、树展开与复杂列表、属性和拒绝诊断用例。 |
| 原 027 CollectionView 处理器 | 从原页提取上轮同一组处理器，使用实际 JSON 与行类型模块，单挂载调用新 AV API；准备并打开 **0.298 s**，交替送入十种非法请求共 **100 次**，随后五个合法请求更新布局、排序、选择、菜单和打开目标，最终文本显示 `selected 0`，派发、读状态与最终视图共 **1.435 s**。此前同一夹具通过桌面累计重放在内存预算处拒绝。此回归由独立 AV 探针直接调用，打开目标与菜单以模型字段为判据。 |

下一验证从 AutoUI `PackageStateEngine` 的正常挂载和派发进入，接入当前模型与受检视图，包括子组件属性、带完整 i64/u64 的 `DirListing`、逐挂载授权、先校验成功与失败回包再写盘和已提交写入的后续读取失败分支。原 027 的五处文件操作及整页严格生成仍开放，T-02 不勾选。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-av-live-widget-session.log` `93812EED803747228B091224AC8ADD48C9867C3CF75CA6B237E075B2F6CD835C`
- `r2-av-live-widget-neighbors.log` `AD4D9B9A1894E774112256018CA3D8385C6ECBAE3A9D10050FD9DDEB51D2AAEA`
- `r2-av-live-collection.log` `39FB1FFDC87B8E18DD385171C14E244EB6119D236B56E72338879D63C9F558AC`
- `r2-av-live-collection.rs` `E500524EB80EE052A2DA1BC35A1E91C859FABD5ECF09ACE3CCA2010A0265C583`
- `r2-collection-wire-checked/src/front/app.at` `107B2D5F6BB5B4B66FC1340F2DFA62E38F99834CC0920D5B303A332DE74CB7AF`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 51 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-lang | commit: 24d630e29eeae431c3d1ae3f3fecf6061004a1ba | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 当前模型接入桌面文件会话 | outcome: partial | AutoLang `24d630e29eeae431c3d1ae3f3fecf6061004a1ba` → `01dfdfbf756e088a1ecbb8ef50b498d42e19f988` | AutoUI `5736fcb609386e14216db093cfe80a44fdca068c` → `56924a33c4492b10494e1684f21d7c5f1c45e3aa` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

`PackageStateEngine` 的正常挂载为符合当前接面的受检根模型建立独立 AV 会话。派发一次转换，沿现有视图修订号和子组件事件路由发布当前帧。无值属性和无局部状态的子组件可组合其受检视图。有值属性或持有局部状态的子组件仍使用原场景路径，原整页 CollectionView 的接入仍开放。

目录回包只在逐挂载授权的宿主调用区间内进入专用受检单元。链接检查回包单元只能读取输入单元，输入按其记录、字段名和列表类型校验及计入容量，保留 u64/i64 全宽。正常 UI 事件不能冒用目录与写入的回包消息。写入先在当前挂载分别验证接受和拒绝后的处理器、清除 pending 与渲染结果，再按本挂载的授权访问磁盘，提交对应结果。磁盘写入成功后的目录刷新出错，保留已提交的模型与视图。关窗、重挂与迟到帧继续走原生命周期门。

| 验证 | 结果 |
|---|---|
| AutoLang | `cargo test --locked --offline -j1 -p ace-av --lib` **20/20，0.26 s**，准备 **3.24 s**；现有会话用例加宽类型回包，执行读回 u64 最大与 i64 最小，错类型、容量过小和普通派发伪造拒绝。相邻 `--test conformance plan981_age_package_av -- --test-threads=1` **16/16，0.31 s**，准备 **40.39 s**。 |
| AutoUI 包与路由 | `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --test plan124_routes --no-run` 准备 **2m01s**；用源码校验过的 ui980 provider 直接执行两个二进制，分别 **20/20，13.42 s** 和 **4/4，1.09 s**。 |
| 原 027 CollectionView 父处理器 | 原处理器、JSON 和行模型置于同一桌面挂载，实际 `NodeDesktopBridge` 派发上轮同一组十种拒绝载荷后，合法布局、排序、选择、菜单、打开目标成功，第二挂载保持初始状态，**1.74 s**。打开目标的判据是提取夹具中的状态字段。 |
| 原 027 目录处理器与实际授权 | 原读盘及导航提交处理器的提取夹具走真实目录，启动、Unicode 文件名、扩展名、面包屑、列表和越界拒绝后保留旧视图 **2.40 s**。相邻实际桌面回包用例覆盖访问前拒绝、授权隔离、撤权、点击刷新以及关窗重挂后丢弃旧点击，**0.37 s**。 |
| 实际写入与失败恢复 | 检查伪造写入回包和只读授权拒绝，允许删除后独立验证磁盘，错误回包在写盘前拒绝；另核成功删除后目录回包未清除 pending，已提交成功状态保留，同一路径写入的替换文件不会被后续事件删除。**0.34 s**。上述桌面用例经测试宿主和窗口消息入口执行，实窗与原整页验收仍开放。 |

下轮继续 T-02，原整页五处旧 VM 文件操作处理器、带值属性和状态的 CollectionView 子树、及其严格双端入口仍需贯通。T-02 继续开放，工作树已提交供后续验证。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-desktop-live-av-unit.log` `FE56D648B2E0DD068C47C4A7A8DFCD42C166F83BD8F419B1164BE6342A9ACB5C`
- `r2-desktop-live-av-neighbors.log` `4AC64588562D8E55217E09457431935CD572D57B25C33D29E5194658D21E0FF4`
- `r2-desktop-live-widget-build.log` `209B75E3BE6779CAD40E12DCBC67845FA749BAB13C467FE02401E86153AF216D`
- `r2-desktop-live-plan114_widget_packages.log` `E4F7E7CBA2F1A29954483E03914890E8EE04F8FA5BAA7EB3047A3588CF97944B`
- `r2-desktop-live-plan124_routes.log` `64F16EE1D760F737B34BB49EBD4727A08A4F8C8D8442EFBDF75888BF32FD783E`
- `r2-desktop-live-checked_collection_events_decode_and_reject_stale_json.log` `87ED0D25398819619230B67D2CB782E1AD6C2B164A077FAF49A83A3808EDDE1B`
- `r2-desktop-live-desktop_file_binding_commits_one_scoped_read_and_retains_its_reply.log` `9CCE04A9159554C409B1F41F54CA525F0A7AFD8377BE5B8CC997B575CDBCDBC2`
- `r2-desktop-live-file_manager_rows_and_path_are_published_only_after_an_authorized_read.log` `96166803EE2056224E8E45AC53AEC9610C34A925074BDC7EB65FBCF7B5F1B431`
- `r2-desktop-live-checked_file_mutation_commits_only_a_live_authorized_event.log` `602EA7ED510709C6A6CE72CC7716AFC1D923A4E539E034AE636AF14545DA9C15`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 52 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 56924a33c4492b10494e1684f21d7c5f1c45e3aa | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 文件操作共用受检请求 | outcome: partial | AutoUI `56924a33c4492b10494e1684f21d7c5f1c45e3aa` → `3902aab1467bd5080502f790fface57467fa6927` | AutoLang `01dfdfbf756e088a1ecbb8ef50b498d42e19f988` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

原 `CommitRename`、`CtxPaste`、`CommitNew`、`ExecuteDelete` 汇入既有 `RequestMutation` 和 `MutationApplied`。路径以拥有的字符串生成，清除模态字段前获取独立文本快照，解决 MIR 借用逃逸和写入冲突。原生 VM 的直接文件动作集中在本项目既有兼容范围内的帮助模块，继续以原窗口的确认、剪贴板及目录刷新入口执行。文件动作反馈现在用原页已有状态栏显示。原 `OpenItem` 保留进入目录、原生桌面关联请求与独立 VM 的系统打开分支，浏览器显示无法启动本地文件。系统默认程序启动和 AV 的本地文件打开仍待实证和接线。

| 验证 | 结果 |
|---|---|
| 原页语法与严格入口 | 原 `app.at` 整文件、视图及独立处理器共 **59/59** 语法通过，**1.132 s**。原 `app.at` SHA256 `65E39A194684716BCEC9B19E4775BBAC23D2DB90DAA96675A600FDC1484903A5`，`native_file_actions.at` SHA256 `72B5A846866B424279D4F03D0FC5A9EF52BD9376D43FD6DD9838A1E77C321211`。终态 `AUTO_UI_ACE_VUE=1 AUTO_ACE_CANDIDATE=1 auto build --render vue --gen-only` 退出 1，**2.282 s**，首诊断 `source.age.handler-effect: message SetViewMode requires a host effect`。仅诊断副本移除 10 行旧 `storage.set` 后，严格入口首诊断转到 `collection_view::collection_view` 的语法，**2.902 s**。已验证原仓仍保留设置持久化。新增 VM 兼容帮助模块的 `try/catch` 也需要为受检目标隔离。 |
| 受检 AV 与 A2Vue 定向 | 新用例 `original_file_action_handlers_use_the_scoped_file_session` 提取原四个操作处理器、路径构造、原请求和回包处理器，经 `generate_package_sfc` 生成文件宿主绑定，经真实桌面桥向临时根授予权限。验证只读与越界拒绝，成功新建、改名、复制、删除及随后的实际目录回包，**1.68 s**。采用已准备的 ui980 provider，`cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --no-run` 准备 **32.99 s**，同组 **21/21，15.44 s**。夹具的 VM-only 调用返回专门失败值，以检测误入旧轨。 |
| 原页 VM 实窗 | 最终启动 **4.261 s**，MCP 回放 **13.019 s**。检查目录双击处理器、关联命令的真实文件路径与制表符、同名冲突保持原文件、改名、复制、剪切、新建文件/目录、删除文件、拒绝删除非空目录及删除空目录。该汇总为实窗取证脚本。首次启动曾只到 MCP 监听而无首帧，终止后重试通过。 |
| 原页兼容 Vue 浏览器 | Chrome 经原使用入口及真实 VM 后端。新建、改名及删除 **5.725 s**，复制与剪切 **4.071 s**，推进路径后收到的迟到写回复 **3.954 s**，只读授权拒绝并保持原选中 **3.871 s**。无本地打开的提示另测 **4.045 s**，其处理器随后保持同一源码。冷浏览器一次导航超时，保留日志，同一存活服务的热重试通过。 |
| 原页构建缺口 | 兼容产物的 `pnpm run build` 退出 2，**3.935 s**。原 `native_directory` 的 `path.extension` 未绑定，CollectionView 和 ThumbnailContent 的 Vue 轨常量比较触发三处 TS2367。比对上一轮已生成原页，四处表达式均已存在。严格生成、兼容产物的完整 TS 构建和 AV 整页仍开放。 |

继续 T-02，设置持久化需要受检宿主接面，同时保持原 Vue 和 VM 的恢复值。原 CollectionView 包仍需通过严格语法、类型检查及有状态子组件实例；原生专用辅助函数由对应轨处理。原 027 的受检全窗口、真实本地文件打开和严格 A2Vue 浏览器运行尚未达到 T-03/T-05 条件。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-original-actions-frontdoor.log` `50B3B51AFD471C6DF1BAD67271EC71CF56F99DDBB5A2AD43FB2A64B92A3A27BA`
- `r2-original-actions-original-strict.log` `EC75A8DE7126631D7351E0124D842C1F5779A3D38A4CBCE8BB102AF87C16B5B7`
- `r2-original-actions-strict-no-storage-diagnostic.log` `99B1EB56CD8E1C143BD9E9E0E13F14F1C5AFC678C216DE1775B5703BFBFEB8D1`
- `r2-original-actions-collection-child-diagnostic.log` `F7AE5CBBB0399DB06096852019EB3179845E637CD617CBD18EC758281FC65605`
- `r2-original-actions-av-focus.log` `2333873F905AA1F4C1522E293873AC288C2E7BFAC438FC3FDA8C08DAA910773F`
- `r2-original-actions-widget-group.log` `75BA20F978CB7253121A7B39DD84461F1A091F2ECF060F6944CEE6699AF870E8`
- `r2-original-actions-native-final.log` `F72E07C6226CA67B440552401A6D6D0B98FB7DC128968CB649592B96453FC8FA`
- `r2-original-actions-browser-final-create_rename.log` `B705162E7705B4B0DD985C3DFE2CD158AB3925C870536535F01CD409A03A855A`
- `r2-original-actions-browser-final-copy_cut.log` `1EB3CA9D7C52B25FF7BE41EDE8DD370896CF3BDBAD86B7855D65B153A7C97C0D`
- `r2-original-actions-browser-final-late_reply.log` `3C436D9B79EA359BF544AD4DEF224F9755FF7AA751D504B7729F8096BFFC675B`
- `r2-original-actions-browser-final-readonly.log` `4EEE646DCFEA06604337CC80100ECA6E695ABD3136F3AF559956D1EE1D775794`
- `r2-original-actions-browser-open.log` `BC61FD24DF35A04036CE093CC0D29FBC70C72AE97DEB6CA0A2FD2EAE24D158AD`
- `r2-original-actions-browser-build.log` `D8B0D4ADF6E9F062D197953192FE213893399AA64483C57307A73A6D0B4E5947`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 53 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3902aab1467bd5080502f790fface57467fa6927 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 设置持久化宿主绑定 | outcome: partial | AutoUI `3902aab1467bd5080502f790fface57467fa6927` → `74d95f991b2cbfb5c2b5cb25b1c61ccfdea9ed1a` | AutoLang `01dfdfbf756e088a1ecbb8ef50b498d42e19f988` → `8c92918fa2022d25736cd6a79bd0ab931314d935` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

原 027 在 `bind` 声明四个原有 `fileman.*` 键。VM 桌面和受检 AV 继续使用宿主存储文件，兼容 Vue 和严格 A2Vue 继续使用浏览器 localStorage。原应用的恢复处理器检查已保存的视图模式、排序列、排序方向和隐藏项值。错误值保持初始状态。宿主在初始化后恢复，在设置变更后写回。受检运行时仅从成功发布的状态取值，A2Vue 将持久化文本转换为现有的受检 UTF-8 表示。AutoLang 为带设置绑定的入口生成可执行视图，使只有设置绑定的受检组件也走正常入口。

| 验证 | 结果 |
|---|---|
| 原 027 VM 桌面 | 两次实窗启动分别 **4.327 s**、**4.140 s**。原页先读回预置 `grid,size,desc,true` 并显示隐藏文件，经原处理器修改后写回 `list,name,asc,false`，保留额外键。新进程从同一文件恢复修改后状态并隐藏该文件。两次 MCP 断言分别 **1.396 s**、**0.439 s**。 |
| 原 027 兼容 Vue | Chrome 从原四个键恢复，实点排序列、网格视图和隐藏项，检查写回并刷新，**8.401 s**。默认镜像解析的 Rollup 新版本 tarball 返回 404，生成目录复用已验证的锁定版本并离线安装后启动。 |
| 受检 AV 与严格 A2Vue | AV 用例提取原绑定和四个恢复处理器，验证非法值保留默认、拒绝从事件入口伪造恢复、成功写回、失败转换不落盘和重开还原，组件包组 **22/22，16.52 s**。严格 A2Vue 夹具复用同一原处理器与原键，经正式 CLI 生成，在 Chrome 验证合法与非法旧值、修改和刷新 **1.533 s**，`vue-tsc && vite build` 成功 **7.559 s**。 |
| 原 027 严格入口 | 原 `app.at` SHA256 为 `981EF4D966C1E51B96DC2C380E2083A546BCE563832D406DE51D0B6DE3A2CD75`，VM 和浏览器使用的两份运行副本与工作树一致。严格生成越过原 `SetViewMode` 宿主副作用错误，在原 `collection_view::collection_view` 停于 `source.syntax: expected fn, found widget`，退出 1，**3.719 s**。 |

继续 T-02，先打通原 CollectionView 包的解析和有状态子组件属性与事件，再验证原 027 严格整页。上一轮已记录的兼容产物 TS 构建、本地文件打开及 T-03/T-05 同样继续开放。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-preferences-native-before.log` `F4767322FF4FD1591E5D947D7C569814C6C5657B919E40AFE02362F954E09756`
- `r2-preferences-native-reopened.log` `5E166B4AD873711F423BF2336A437FFE40F42E0684AD2FE9DF086FE5C0DE932E`
- `r2-preferences-browser.log` `0BC5F1058189088FA2F2C63F3E2EB707B00B3D0AB5F4F8D3B4CDEB0F11E1918A`
- `r2-preferences-av-tests.log` `818BB60F446E3D456C0CFF548138AB06BF88E355412193312EC50DE5A9385F3F`
- `r2-preferences-checked-browser.log` `80FA71CFDAD2BE0C6D5E25D7F8CBB2A0E3DD8B8C457FBB976E4451EB22DE97CF`
- `r2-preferences-checked-build.log` `71BC3B6C7B4539083E6437872373F100AA86C96F49421945CCC243E6140B406B`
- `r2-preferences-strict.log` `9751FF10E0A3431898BC19132CBE9578CE4C128D6ACD96139415A96BC8435E1C`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 54 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 74d95f991b2cbfb5c2b5cb25b1c61ccfdea9ed1a | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 集合数据类型化 | outcome: partial | AutoUI `74d95f991b2cbfb5c2b5cb25b1c61ccfdea9ed1a` → `f55f598bc88d39750625567345308b3c04c78c7d` | AutoLang `8c92918fa2022d25736cd6a79bd0ab931314d935` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

原 027 的 `fm_collection_items` 把原文件列表转换为共享的 `List<CollectionItem>`，为子组件传递显式的行、单元格及操作类型。保留输入顺序、真实路径标识、版本、中文文字、类型和缩略图状态、三个元数据单元格及原打开和菜单操作。更名 `fs_util.at` 中与严格语法保留字冲突的形参，调用和值语义沿用。源码和模块闭包验证均绑定同一最终版本，`collection_items.at` SHA256 为 `F309D80329D7623582BA6D41F8027D4AD4E2603271C48E74DCD9982C136C9E65`，`collection_schema.at` SHA256 为 `AC317D21047B2BB3C42CA17DD012053866A150E326DABB3D8024B70653E2F865`。

| 验证 | 结果 |
|---|---|
| CollectionView 原始解析 | 分段探针 **0.198 s**。全部 12 个属性及原导入、消息和模型均通过语法。计算属性中 `config_error` 与 `header` 通过，`rows` 的参数内条件值报错。原视图块、监听块以及含内联条件值、异常块和动态事件体的处理器分段失败。原 027 严格入口仍在 `collection_view::collection_view` 报 `source.syntax: expected fn, found widget`，退出 1，**2.804 s**。 |
| 受检 AV | 原转换函数、行和元数据帮助模块经可执行受检视图读取。用例断言三行的顺序与完整身份、三组嵌套字段及每行两项操作，并排除空路径。构建 **34.72 s**，组件包组 **23/23，18.37 s**。另经持久 AV 实例按同一源码完成 `Load` 及渲染。 |
| 严格 A2Vue 数据夹具 | 正式 CLI 生成，`vue-tsc && vite build` **3.231 s**，Chrome 对生成的生产产物验证原适配器的顺序、版本、缩略图和两项操作，**0.913 s**。夹具视图用于观察原适配器的输出。 |
| 兼容原页 | VM 实际启动 **4.268 s**，MCP 核对窗口真实文件行、Unicode 路径、9 个无效输入、旧实例拒绝、视图与排序及原上下文操作，**6.157 s**。Chrome 实际点击原文件行、列表、排序、右键和目录双击，**4.548 s**。生成目录曾复用旧辅助模块的产物，移出取证副本的 `.auto/ui-cache.json` 后重新生成，使用原依赖锁定版本。 |

继续 T-02。下一切片处理原 CollectionView 及 ThumbnailContent 的条件视图和事件处理语法，并核对键盘焦点与有状态子组件的进行中能力。原 027 严格整页和其后的实机验收继续开放。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-collection-child-parts-baseline.log` `8EADE6AC0F11ADF18415604423A5BE62D5B2CCA7254CC357A0F6C9C4F05501A3`
- `r2-collection-typed-av-group.log` `21D07A44207EE46C9D1AC36D161ADCBEBCAD5960A0F3D880BC479F50A89CBF76`
- `r2-collection-typed-checked-build.log` `A0A6E0DBE5B9FB805D9D2D5004F1E7534297462863375EFE3756C281F2E62053`
- `r2-collection-typed-checked-browser.log` `0E7394B3A3933FBF4C85A8BCF0AB7E8BA86BE281B56226D92D16778F7EB74C61`
- `r2-collection-typed-compat-browser.log` `45AD3549607716F859F71180D286BFB4F1BD9AFF355E3437C148B4C585308DE0`
- `r2-collection-typed-native.log` `50191BC7F4558127C5BFFC4DCEC712E40D41672B9FEFA18332F04B42C5461D99`
- `r2-collection-typed-original-strict.log` `86A94A02EBB6C6B91C0978DAC7B606113F851AAB6E83319E9C3FAF67BC383676`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 55 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: f55f598bc88d39750625567345308b3c04c78c7d | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原集合布局与受检浏览器事件 | outcome: partial | AutoUI `f55f598bc88d39750625567345308b3c04c78c7d` → `36fcfc4145d8764bdc4183d3d83a1342ee90a771` | AutoLang `8c92918fa2022d25736cd6a79bd0ab931314d935` → `1b4dbb442df93cfd08910c738a2406df698c5022` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

原 CollectionView 使用同一源码计算条目类名、当前目录的焦点和网格步长。保留 640、1024 两个网格断点、焦点目标选择及原缩略图图标回退。兼容 Vue 的 Shadcn Button 生成透传原有 DOM ID，焦点恢复据此命中条目。受检 A2Vue 消费当前帧的焦点、失焦、键盘、双击及右键事件，检查原有键和修饰符。AutoLang 输出受检按钮的布尔禁用属性，AV 和 A2Vue 使用相同的禁用值，AV 的帧入口拒绝该按钮提交。

| 验证 | 结果 |
|---|---|
| 源码前门 | CollectionView 的原视图、三项计算属性、`RestoreFocus`、`FocusCurrent` 与 `Move` 分段语法通过，探针 **0.202 s**。整个 ThumbnailContent 语法通过，**0.075 s**。构建后的原 027 严格入口仍在 CollectionView 模块报 `source.syntax: expected fn, found widget`，退出 1，**2.865 s**。分段剩余为 watch 及含 JSON 的五个事件处理器。 |
| 受检 AV | 正式组件包组 **24/24，15.82 s**，新增用例 **0.486 s**。用同一当前帧验证禁用项拒绝和重新启用后提交。 |
| 严格 A2Vue 事件夹具 | 正式 CLI 生成，`vue-tsc && vite build` **3.332 s**。Chrome 对生产产物验证焦点、失焦、键盘修饰符、双击、右键、禁用事件拒绝及网格断点，**1.183 s**。 |
| 原 027 兼容 Vue | Chrome 实测方向键移动保持选中、空格提交选中、切换列表后恢复到同一条目，**6.326 s**。另测排序保持选中、右键菜单及双击进目录，**3.744 s**。移出取证副本的旧生成缓存后，用原锁定依赖复跑。 |
| CollectionView VM 原独立示例 | 既有产品入口执行真实窗口的选中、旧事件拒绝、方向键加空格、关闭与重开，脚本 **8.737 s**，外层计时 **8.889 s**。 |
| 原 027 VM | 首次在 MCP 监听后未出首帧，进程终止后重试启动 **4.495 s**。既有原页回归核对 Unicode 文件行、切换视图、上下文菜单、排序和旧实例拒绝，**10.003 s**。 |

继续 T-02。将原 CollectionView 的监听块及 JSON 事件接入受检包，再贯通原 027 的有状态子组件、媒体与原生焦点和键盘入口。源码 SHA256 为 `collection_view.at` `027C526552457C2C4C7072BFF1BD587E462992168F121DB82EF974BF3B963689`、`collection_layout.at` `02E7C157275CB0CD41B309AFE5E60190B2B772012E6E800806EC8BA0C22601A7`、`thumbnail_content.at` `B1AC9E382F41F364FA8D6BD94C0D525FD1AEA99932B5B7B700919EA13AB2383C`。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-cv-syntax-parts-after.log` `F0B21CCC0D96CDA75955C73FFF5D2DBA2E571E67F761233FAD74EF343833A89D`
- `r2-cv-thumbnail-parse-final.log` `68ACC909E2872DD1CCA635BDFD516B189163AA34658311DB65BBF598AB628564`
- `r2-cv-interactions-av-focus.log` `F8D75507103C30698BFA32E235090D7EF4216BAD8E88B465B8F061499FD82D17`
- `r2-cv-interactions-av-group.log` `2CB7DFE3EEAAED9B3EA3B3113E90C4B7DFA89A6963EB94ED692003BF20CCFEAA`
- `r2-cv-interactions-checked-build.log` `B6FF3D90501D8262C5E5D0E591D41CA837153DC02049FFFDCDF1875242FD3F6D`
- `r2-cv-interactions-checked-browser.log` `020BA2C4015AC95D46EDDD12F48AA423FC9A2A03F6FEC6B3A752DA6D7062FBFA`
- `r2-cv-layout-original-keyboard.log` `C0BA485338FB7257CB3607711B7FE7FB1F4F444C2E0FA04F916387409721CF6F`
- `r2-cv-layout-original-compat-final.log` `643CC98648DE1389204E356D2F836F2AA4A0AEEFD794F8F73D6356A50A3C02C3`
- `r2-cv-layout-native-demo-final.log` `9363E8A4F9E6DEC306693D0ED1FB39B55D5929D3EBF8D6687206ECEADDC4511B`
- `r2-cv-layout-original-vm.log` `FCC3CBD8D48CBF2F8EBF94D96CE0BDE4AF0D1F92E3C54E188109A124D3253339`
- `r2-cv-layout-original-strict.log` `09F8B904CC4CD13FDDAF638963158352B5DAD42EB3B4B9DDB4F770741C976299`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 56 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 36fcfc4145d8764bdc4183d3d83a1342ee90a771 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 CollectionView JSON 事件编码与校验 | outcome: partial | AutoUI `36fcfc4145d8764bdc4183d3d83a1342ee90a771` → `2d0bf08b5330f2c9c9d599b5081d35b15653cceb` | AutoLang `1b4dbb442df93cfd08910c738a2406df698c5022` → `d483fb38f7d4c763ec5d87d34cd57b4be646c4de` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

原 CollectionView 的视图、排序、选中和条目操作使用共享 JSON 编码，保留原事件名、顺序号、实例令牌、目录、结果版本和条目字段。选中事件省略 `actionId`，动作事件携带它。焦点、键盘和鼠标共用条目绑定校验，保留实例、目录、版本、条目内容和禁用项校验。AutoLang 的公共 JSON 字符串编码补齐 C0 控制字符，VM 标准库沿用宿主现有序列化器。兼容 Vue 的控件直接导入对应标准库函数，使组件携带其依赖。

| 验证 | 结果 |
|---|---|
| 原组件源码和严格入口 | `on` 全块与 11 个独立处理器的语法通过，分段运行 **0.177 s**。原 027 正式严格生成仍在 CollectionView 的 `watch` 报语法错误，退出 1，**3.282 s**。仅取证副本移除 watch 后，严格入口推进到 `source.age.handler-message: message viewModeRequested has no handler`，退出 1，**2.878 s**。 |
| 受检 AV 与标准库 | 四种原共享请求经过受检处理器生成，由独立 JSON 解析核对全部字段、控制字符、Unicode、引号和反斜杠，**0.785 s**。组件包组 **25/25，12.18 s**。 |
| 严格 A2Vue 载荷用例 | 正式 CLI 生成，`vue-tsc && vite build` **3.669 s**。Chrome 点击受检按钮，逐项核对生产产物的四种载荷与带引号的 Unicode 路径，**1.590 s**。 |
| 原 027 兼容入口 | Vue 浏览器点击真实行、模式、排序、菜单和目录双击通过，**4.243 s**。VM 按实际窗口节点点击 Unicode 行、网格、列表和排序，验证选择与排序版本，**3.734 s**，该次启动 **4.332 s**。一次 VM 启动未出首帧已保留日志。 |
| 原包与迁移位置 | CollectionView 原独立 VM 示例的实际事件、旧绑定拒绝、键盘和重开回归 **8.225 s**。正式工具测试 **2/2，3.20 s**，含独立组件生成与迁移包的 JSON 函数闭包，单例约 **1.541 s** 和 **1.657 s**。 |

继续 T-02，实现受检子组件向父组件投递声明事件的执行链并接通监听块。完整文件入口、状态化子组件、媒体与原生焦点继续开放。原 `collection_view.at` SHA256 `C6DC9F16A0421A18A63CAD6C2026FF20D0159C8C1C2B2A7EDB73FDE6724C10D4`，`collection_view_util.at` `F9021F464C030F894309F20F448C7DB54AEC10D84EBE6FF4146C20FC9362F2B3`，`collection_request.at` `CAC6663981EC6E3F0D037B37F24E96EB537E2290F5C9A7A56A4DC0CC7F5462F4`。

材料根 `D:/autostack/plan-artifacts/plan-114/`，主要证据 SHA256 如下。

- `r2-cv-json-syntax-parts.log` `21B6ECC4AE0DA07F1DBDA70C09BADA51954C28F13BFF59B184F811D0467648AF`
- `r2-cv-json-av-focus.log` `01B7D3E9238FFF0D33B45EAA0AE04E5A9252030FE0E0DD8E9A85D11543AD5BF8`
- `r2-cv-json-av-group.log` `B3066CD50E2FE34C294F52A85C7D8DCF52919584C943847126F347708DC8B00F`
- `r2-cv-json-relocated.log` `D294573E6DB058EB36D33FE89D15DC94FEBB32219BA79A3F942899A7E9710A73`
- `r2-cv-json-checked-build.log` `56CFE7112725C1B942180D4FC814D888E700A4214B661718A655C1C8F14CC463`
- `r2-cv-json-checked-browser.log` `5533ADE81865F1C4ED50748AF21746AEB02A10367EA0E2024C065546ABF87FAF`
- `r2-cv-json-original-compat.log` `41099355A30DF42338C8ED5A6BA6931A2ACB71D921C7E20B1516152053DB2EF1`
- `r2-cv-json-original-vm.log` `B5CE9267E74A01D6B8220D2FC942D02954ADA26D37F31A2DE0DB7F9A9DF69EF2`
- `r2-cv-json-native-demo.log` `69B86E8B9627CDB36680CBDCD47CC586246BB8E82F460BD8A2407286D9142AEE`
- `r2-cv-json-original-strict.log` `630D541027BF1FE0AC8A9FDD26368472A99F56372CC6BE3D8A8EEB21E9814F0A`
- `r2-cv-json-watch-followon.log` `300F0AAA4D3528074C84AD7F526403DEE42FDB114D30A70290FD708A694EAF10`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 57 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 2d0bf08b5330f2c9c9d599b5081d35b15653cceb | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检有状态子组件向父组件投递事件 | outcome: partial | AutoUI `2d0bf08b5330f2c9c9d599b5081d35b15653cceb` → `54d1c69815e18c62c17e1f406fbd65390f11f951` | AutoLang `d483fb38f7d4c763ec5d87d34cd57b4be646c4de` → `1a7e501641cb68fac99f83b46b7d7fa0b78415f5` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

共享受检编译为调用已声明且无本地处理器的消息生成每次转换独立的有序输出队列，验证参数数量与字符串、i32、布尔类型。A2Vue 按实例的作者绑定投递到父组件，暂存本次涉及的模型，渲染成功后统一发布。子组件实例以语法站点和作者键或站点内位置标识，Vue 以同一标识管理渲染片段。父处理器失败保留已发布帧，迟到目录回包也遵循当前帧身份及作用域检查。受检 AV 装配遇到有状态组件的输出队列时返回 `ace.desktop.widget-outbound`，A2Vue 装配对 `Init` 中需要挂载父组件的事件返回具名拒绝。

- `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 通过。以测试内的作者源建立独立示例，由严格入口 `auto build --render vue --gen-only` 和 `pnpm run build` 产出生产包。Chrome 在 2073 ms 内覆盖三种标量载荷、有序投递、未绑定事件丢弃、同级实例状态隔离、父处理器出错回滚，以及卸载和重新挂载前的旧回调拒绝。
- 目录诊断夹具使用原 027 后端及目录回包合同，增设两个有状态目录按钮。经 `auto run --render vue --server vm` 在 Chrome 中验证启动只读取一次、越权读取保持当前帧、甲目录的回包延迟到乙目录发布之后才返回、历史只提交一次。乙子组件计数为 1，甲子组件计数保持 0，浏览器用时 1722 ms。
- `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages -- --test-threads=1` 通过 26/26，用时 15.70 s。新增定组用例经受检 SFC 检查消息表与错误载荷，经 AV 装配验证具名拒绝。三个工作树的交付边界仍为 T-02。
- 原 027 严格入口失败，`collection_view::collection_view` 报 `source.syntax: expected fn, found widget`。单独的诊断副本只移除 `watch` 后，错误推进到 `source.age.handler-effect` 的 `FocusCurrent`。后续补齐有状态 AV 投递、属性观察及受检宿主焦点接线，保留原源码语义并重跑双端入口。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-cv-outbound-checked-gen.log` `CA6E27A78D8B4F8AA396849B2BD2C3411D553169086D1010127A27B2535C64C7`
- `r2-cv-outbound-checked-vite.log` `1456A5BE9653727EC8DA435B38305E7BA6ED689D393B0325A9382517924D927A`
- `r2-cv-outbound-checked-browser.log` `376466C89BCD9DC22A48CEDFBDFAC3C1E0C78B211666980AC086A313F9526FB8`
- `r2-cv-outbound-file-recheck-browser.log` `81583A00B3BEBBDC2B03D2BAC2F4CA8C461AB925F1876EED05D757202118E431`
- `r2-cv-outbound-widget-group.log` `418127489DF8ABC5F92ACF1930D7418BC0614318D2E47FC2B2B947FD104D747F`
- `r2-cv-outbound-original-strict.log` `49DA6F51A0CB234C13F657E3A8DDEEBAA990A96EF850414C1666FEDB3C518866`
- `r2-cv-outbound-watch-followon.log` `6B1A7406F1A69B4698019B23DFA6C289F1EA0390F3967CACCC5C27FC0F482F09`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 58 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 54d1c69815e18c62c17e1f406fbd65390f11f951 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 AV 有状态子组件事件及类型化属性 | outcome: partial | AutoUI `54d1c69815e18c62c17e1f406fbd65390f11f951` → `33a5f609ebf733100ad702610b7ab5f0695b5ea3` | AutoLang `1a7e501641cb68fac99f83b46b7d7fa0b78415f5` → `c328d8c84d915fe57bcd2c179e6f7af6ea3fd813` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

正式 `NodeDesktopBridge` 从当前帧的组件路径定位受检子实例。AV 用同一快照中的受检单元格保存并恢复每个挂载实例的模型与类型化属性，事件按作者站点的绑定逐条投递给父组件。父处理器或后续帧求值失败时，根模型和子实例一起恢复到上一帧。记录列表属性由受检属性求值器传递，循环作用域内的标量属性由同一视图的受检数据传递。根宿主专用消息仍由绑定端点投递。采用此路径的包仍只调度根计时器，装配对子计时器与需要向父组件投递的子 `Init` 分别返回具名拒绝。

- 使用 `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --no-run` 编译测试二进制，最终增量准备 29.91 s。设置既有的源绑定桌面准备目录后，定向执行 `stateful_child_transitions_compile_checked_outbound_events --exact --nocapture`，通过 1/1，用时 0.43 s。断言涵盖两同型实例隔离、字符串/i32/布尔按序投递、出错回滚、真 Iced 消息入口拒绝旧帧、卸载后重挂载、记录列表实时更新及循环属性投影。
- 同一测试二进制使用 `--test-threads=1` 通过 26/26，执行 13.31 s。受影响的旧 AV 根派发、无状态子回调、文件读取、写入回包和偏好设置一并回验。完整定组并集仍由最终交付门禁负责。
- 同一作者夹具的 A2Vue 生产包经 `auto build --render vue --gen-only`、离线依赖安装与 `pnpm run build` 生成。Vue CLI 使用上一轮已准备的同一生成器，当前 Rust 用例另验证本轮源码的 SFC 生成。Chrome 生产包回放用时 914 ms，覆盖同一记录列表、循环标量属性、两实例及事件顺序和失败恢复。
- 定向测试初次加入跨模块记录时，因夹具漏写公开导出在 0.09 s 内被类型检查拒绝。已修正公开导出。浏览器首次定位使用独立标签查找，诊断确认文本已生成在父渲染片段中，改为断言实际页面文本的数量后通过。
- 原 027 的属性观察和宿主焦点仍待接入。保留其 `watch` 与 `FocusCurrent` 作者源码，沿第 58 轮两条独立诊断检查继续工作，然后从原 027 重跑 AV/Iced 和 A2Vue。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-av-child-typed-final-build.log` `C97E2B742781DF420D2AD0B36FF2F4163BFE696874282630D13E4F0C020F40AA`
- `r2-av-child-typed-focused-final.log` `A3B17E28EE1D9224F8039C890F7282BC27352C8FE4116F098BD99C14297766A7`
- `r2-av-child-typed-group.log` `E4106D4AA608A571AD4DD7EB874C919C4E45CAC91AAC074CA7CE992F011606D0`
- `r2-av-child-typed-gen.log` `CC250710A04BA9FAFF975847B4E4782B8E32146E78DEC6A8D5E447286BFB09DE`
- `r2-av-child-typed-vite.log` `AE6A2757146DF8DF832D4FE2D6AE3AD71B6E0AA54FD219A22178E13810C385FB`
- `r2-av-child-typed-browser-final.log` `1F28F4285E836AF86A3E8DFA1E9ABA7C3DA66AD1E00022A028A771657C572247`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 59 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 33a5f609ebf733100ad702610b7ab5f0695b5ea3 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 双端受检属性观察 | outcome: partial | AutoUI `33a5f609ebf733100ad702610b7ab5f0695b5ea3` → `cb55e97353cedbf748206216ce2eb46ed157697b` | AutoLang `c328d8c84d915fe57bcd2c179e6f7af6ea3fd813` → `1c45828f50814bfa0bdf4313fbf79a2bd727c526` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

AGE 包从原作者语法投影多来源 `watch`，将条件分支和同实例消息调用交给现有处理器校验与模型转换。每帧使用该实例上一次发布的模型字段和属性，与本次准备发布的值比较。列表和具名记录按内容比较。符合条件的观察器按声明序各执行一次。初次挂载只执行标记 `immediate` 的观察器。受检 AV 与 A2Vue 均在帧提交前执行，并将观察结果和模型一起发布或回滚。本轮接入模型字段和具名值属性。宿主焦点调用仍待接线。

- `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --no-run` 最终增量准备 29.28 s，直接使用该二进制和已准备的桌面提供方。新用例 `checked_watches_compare_published_values_per_mount` 通过，执行 0.38 s。覆盖未声明观察源、未声明消息、挂载触发规则、两个来源共变只触发一次、等值记录列表重建、独立模型观察器、原生错误回滚及重挂载。首次测试构建有两处夹具闭包借用错误，补明确的 `&str` 参数后通过。
- 同一二进制用 `--test-threads=1` 通过 27/27，执行 12.92 s。AutoLang `cargo test --manifest-path experimental/ace/Cargo.toml --locked --offline -j2 -p ace-core --test conformance plan981_members_slot -- --test-threads=1` 通过 19/19，准备 15.69 s，执行 0.07 s。
- 工具以 `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 编译，耗时 1m 59s。使用同一测试的作者源，经严格 CLI 生成、离线准备依赖和 Vite 生产包后，Chrome 在 1964 ms 内验证同一观察触发矩阵和回滚。
- 原 027 的严格 A2Vue 入口在 3615 ms 后报 `source.age.handler-effect`。首个拒绝为原 `CollectionView.FocusCurrent` 的宿主效果。原入口复制的 `collection_view.at` 与工作树 SHA-256 均为 `C6DC9F16A0421A18A63CAD6C2026FF20D0159C8C1C2B2A7EDB73FDE6724C10D4`，包括未经改写的五来源 `watch`。下一轮接入宿主焦点及原交互，再核验严格双端入口。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-watch-tool-build.log` `DBD42522D7E83CCCCC3A6EE9EFBABC1FFCE31BCB74E1393A6A00E093FCB33331`
- `r2-watch-checked-browser.log` `AA6CC7C795AA7682C747F52AF7BA2D4E224C041742C6263F7A1C73880588ED5D`
- `r2-watch-native-focused.log` `C5F6C202A708FA165C26D9F19375A1A6C214F91F80DBC8893D5A050E01370029`
- `r2-watch-native-group.log` `6B608D5E6D656F12B6B6A2FAE5AE56BC198617EE9D9C977828A32465F788DE32`
- `r2-watch-source-members.log` `4981CCA284D4D68BFAA9AC6C096A1D5E298FDCCDB61C7540E5E79C1A79E8F3B0`
- `r2-watch-original-strict.log` `B463917B751CDDC9D0298CAAC9BF405AC878355103852142EC0C9E114784FE94`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 60 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: cb55e97353cedbf748206216ce2eb46ed157697b | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 受检焦点请求和 A2Vue 焦点生效 | outcome: partial | AutoUI `cb55e97353cedbf748206216ce2eb46ed157697b` → `59403c2dd0ad889bb49d35d4eed88bfcf576c6f2` | AutoLang `1c45828f50814bfa0bdf4313fbf79a2bd727c526` → `a890b41b178528854255b4a5e40b8b0eec8bd79b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

AGE 包将 `ui.focus` 的单个受检字符串参数降到随转换返回的有序请求列表，同实例消息调用与 `watch` 使用同一投递路径。AV 的窗口桥发布带有来源实例路径的焦点请求，并过滤本帧中已卸载的来源。Iced 控件焦点和键盘事件的实际接线仍需完成。A2Vue 在提交的帧进入 DOM 后，通过该组件实例注册的精确 ID 目标应用焦点，忽略过期帧、已销毁实例和歧义目标。后台目录请求暂存对应的焦点请求，回包通过当前事务校验后再发布。

- 受检夹具经严格 CLI 和 Vite 产生生产包。Chrome 在 1801 ms 内检查了两个实例内相同的、带单引号的 ID、观察器请求、处理器错误零效果、后续帧不重放及来源卸载后零效果。早期使用 Vue ref 的构建在预组装 VNode 挂载时失败。改为生成宿主目标标记并在提交后解析，复测通过。首轮 Rust 工具构建也发现旧平铺 SFC 处理器投影缺少新语句分支，已补具名拒绝。
- 以原 027 后端的隔离目录夹具执行 `auto run --render vue --server vm`。Chrome 在 2557 ms 内验证甲目录回包暂存期间不转移焦点、乙目录回包提交后聚焦乙控件、迟到的甲回包不抢走焦点。目录权限、历史和两个子组件计数也符合原有断言。
- `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan114_widget_packages --no-run` 准备 1m 32s。使用已准备的桌面提供方定向执行 `checked_focus_requests_belong_to_committed_widget_instances`，通过 1/1，执行 0.31 s，验证类型错误、当前帧请求归属、失败回滚、观察器请求及卸载过滤。同一二进制定组通过 28/28，执行 14.34 s。AutoLang `plan981_members_slot` 通过 19/19，准备 9.30 s，执行 0.07 s。
- 原 027 严格 A2Vue 入口已经越过 `FocusCurrent`，4003 ms 后首次拒绝 `collection_view::collection_view_util` 中的 `??`。原 `collection_view.at` 哈希仍与上一轮记录一致。下一轮继续 Iced 控件焦点、键盘接线和集合工具的受检源码，再从原入口回验。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-focus-tool-marker-build.log` `D90AB60CB17FEDFF8F8A8504F362B82A807FE2488BD966275AFBB9681B90742A`
- `r2-focus-checked-browser-final.log` `37845C6BE6AB49F5D902587E01940BDAD7B1F752B4262ECDF6B79969E1D68255`
- `r2-focus-file-browser.log` `BE693D91F61FC33CA8C2B46E87E3DF07C4503B05285BC34613AED50B280F3439`
- `r2-focus-native-focused.log` `7326446E91263463A57D69E9D07BD4FCBD85BC92556808781361FDE1A2EBC55E`
- `r2-focus-native-group.log` `2E2C00C2329378B9F5B16876EE88021F6DFD5F0E8FF167FB0B9507863B7FE9BD`
- `r2-focus-source-members.log` `FDCD1803987627F8F1440DB4F6A2883790F766C650386741C23710B21CA3AFC3`
- `r2-focus-original-final-strict.log` `7BA12E838563EA786E55BCB5A9731BDCA7B6CCEE2FCBFCCA7CBF1D6A0D5AFB42`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 61 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 59403c2dd0ad889bb49d35d4eed88bfcf576c6f2 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 Iced 受检焦点和指针事件 | outcome: partial | AutoUI `59403c2dd0ad889bb49d35d4eed88bfcf576c6f2` → `a304e1ed3c45f522522e2064eb35ef3f84d21984` | AutoLang `a890b41b178528854255b4a5e40b8b0eec8bd79b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- AV 已提交帧把受检 `ui.focus` 解析到当前组件实例的精确 ID。Iced 宿主每帧领取一次焦点任务，按窗口和帧代次校验，在实际控件树中聚焦按钮或带 `tabindex` 的集合容器。方向键进入该控件绑定的受检消息。鼠标区域负责右键与双击，内层按钮捕获单击时，外层仍能识别双击。
- 使用已准备的 AV 桌面提供方和 Iced 无窗口控件树运行定向夹具，验证同名 ID 跨实例隔离、方向键派发、容器聚焦、失败回滚、来源卸载、旧帧淘汰、关闭重开窗口隔离，以及普通点击、右键和嵌套按钮双击。定向 1/1，1.70 s。首轮分组旧夹具按处理器名取第一个动作，误取新接通的菜单动作。改为同时匹配原测试要触发的 `onclick` 后，禁用门仍按原断言工作。复测分组 28/28，14.16 s，构建 16.52 s。
- Iced 的 `onfocus` 与 `onblur` 状态通知仍需接通。原 027 仍需解决 `collection_view::collection_view_util` 的严格源码边界并重跑原入口。原集合源码不变，本轮在受检夹具确认了 Iced 事件，不计为原 027 的交互验收。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-iced-focus-test-build.log` `7D6F9415720695D95E6A522E4A8CF5B242EE56E740FB9696C66A8958366873BF`
- `r2-iced-focus-native-focused.log` `A0D488FC385E2A35C9EEF4E235997C390BE98FF497CB126809D9FA1FF59CDE01`
- `r2-iced-focus-native-group-initial.log` `A3E4CFDC6A1AC6EC662FF0E8E61906B7C21FC062CDE74248F872BAD108271A6D`
- `r2-iced-focus-native-group.log` `1EA683928A3E7945CC9AD0F6B13F4A2932F5CDB33A8F0DAF5E04132E0F012A2B`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 62 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: a304e1ed3c45f522522e2064eb35ef3f84d21984 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 集合条目、列与缩略图请求进入受检源码 | outcome: partial | AutoUI `a304e1ed3c45f522522e2064eb35ef3f84d21984` → `9f3e2a49b5a992061654380f67a39e7b2523db24` | AutoLang `a890b41b178528854255b4a5e40b8b0eec8bd79b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 集合改用 027 已采用的 `CollectionItem` 和 `CollectionColumn`，适配独立示例与 AssetBrowser 的输入构造。条目绑定仍覆盖已发布的实例、scope、结果版本、顺序、条目版本和全字段快照，包括嵌套单元格与动作。公开事件 JSON 的字段和用途沿用原合同。缩略图原生 HTTP 前缀和请求对象结构继续保留。
- 严格集合工具定向用例通过 1/1，0.84 s，运行受检 AV 状态并生成 SFC，验证当前绑定、同版本内容变化、跨实例和跨 scope 拒绝、禁用、重复 ID、默认名称列及缩略图状态。包括上轮 Iced 焦点用例的已准备提供方分组通过 29/29，14.98 s，构建 15.44 s。
- 兼容独立示例真实原生窗口产品门通过，9.539 s。Chrome 实测通过，1517 ms，覆盖选择、排序、列表切换、方向键、移除后的焦点、默认动作、禁用条目和重开。原 AssetPicker 兼容入口生成成功，17303 ms，隔离 Vite 构建成功，2.59 s。
- 从同组当前原 027 源复制 `pac.at` 和 `src`，`app.at` 源及隔离件 SHA-256 均为 `981EF4D966C1E51B96DC2C380E2083A546BCE563832D406DE51D0B6DE3A2CD75`。用 `AUTO_UI_ACE_VUE=1` 经 `auto gen` 重验。严格入口越过集合工具与 `ThumbnailContent`，3208 ms 后首先拒绝 `official::native_file_actions` 的 `try` 语句。独立受检集合闭包探针还暴露了旧 `auto.math` 等宿主导入需要接通，原 027 全量受检运行及 Iced `onfocus`、`onblur` 通知仍待完成。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-cv-typed-test-build.log` `4D64F8B3B9C7D593CE5877D7BD89CC2455E9580E66BC0A455AFB54BF94774551`
- `r2-cv-typed-focused.log` `826E373D46BD957009786D844CEA449F0DE7095FDF70546B81501AB3B52D50AB`
- `r2-cv-typed-group.log` `5D138F54F5532F65364C8156AF79BFF206EA8B442435DBE471B77E1AF5039AD2`
- `r2-cv-typed-compat-demo-native.log` `5127CF682D62032ED0716129F7958B154216426651E6C4AA1BB73E5E2734088E`
- `r2-cv-typed-compat-demo-browser.log` `2012D72745C7449493011F76CC06EAC225896B737B4A6614E68057B1916F2C75`
- `r2-cv-typed-compat-asset.log` `9662015B2C6D0C9E341DB718B40A092C4D24F1B22DC80B3D883AC613AAAED8DE`
- `r2-cv-typed-compat-asset-build.log` `14A4FE0CD0857E6779BB55EEC8746B77F26DAC0BD56213F0F768D31C02AF1004`
- `r2-cv-typed-original-final.log` `104769177935887CB912C4E5BCFDBB119236AF34A24B4268913BD1381D150514`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 63 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 9f3e2a49b5a992061654380f67a39e7b2523db24 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 Iced 焦点变化通知与 027 原入口诊断 | outcome: partial | AutoUI `9f3e2a49b5a992061654380f67a39e7b2523db24` → `ddb0657098c33c4992c52616e731168b89cf2c6a` | AutoLang `a890b41b178528854255b4a5e40b8b0eec8bd79b` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- Iced 控件树在焦点确定后派发 `onfocus` 与 `onblur`，同帧焦点转移先失焦再聚焦。通知保留窗口、组件实例、产生帧和受检参数。前序动作发布新帧时，只有同一控件仍提供完全相同的受检事件才继续派发。控件消失、事件参数变化、迟到或窗口关闭时拒绝，点击入口也拒绝伪装的焦点事件。
- 在已准备 AV 桌面提供方的 Iced 无窗口控件树中验证了同名 ID 转移、重复重绘零重放、参数变化拒绝、程序聚焦和鼠标点击共存、关闭清空待处理通知与重开隔离。定向 1/1，2.44 s。分组 29/29，15.35 s，构建 35.66 s。
- 原 027 的集合状态和空列表文案由已有状态独立计算，原语义保留。源 `app.at` 与隔离件 SHA-256 都为 `4D49AB4B8BBC17111272E2890627CA292D0383064E11094042961440D1A01812`。兼容入口生成通过，12339 ms，Vite 构建 3.35 s。原样严格入口在 4178 ms 后仍首先拒绝 `official::native_file_actions` 的 `try` 语句。
- 隔离诊断件只暂时移除旧 VM 模块的 `try/catch`，首次越过该语法后暴露了集合 `status` 条件属性边界。迁到具名计算后，同样的诊断件进入链接并拒绝未知宿主模块 `auto`。诊断件未作为产品源码运行。后续须处理旧 VM 直接调用与受检文件会话的分流及宿主依赖，原 027 全量实窗和浏览器验收仍待完成。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-iced-focus-notice-build.log` `9E608DFC98133EDC3C4089FFA42B6FA133A4DCD415937FEEB26CEE66242DEBA5`
- `r2-iced-focus-notice-focused.log` `256362E649A67216748C8A26A371043A58BB640FE51841AEB6E4E83FF1044E27`
- `r2-iced-focus-notice-group.log` `54136ED9D5C9EBBFEB40E31819DA97770C44415954E5F64615B18F1FF35996F5`
- `r2-file-mode-original-strict.log` `EB0F9F72B0932C453245532325C31732E127D0B71A61BD5C3661ECABCB55FD40`
- `r2-file-mode-source-probe.log` `A26C1FD5087E8F827B5B0FA3BB4C01F6AD1235C30A8ACB9EFC02E88060CCA1B6`
- `r2-file-mode-source-probe-current.log` `EC59F4AC0D6AC9B071478F228C11AC8E24382FB94FAF23B207BBAA3F45E689D7`
- `r2-file-mode-original-compat.log` `9A514BB99425B800EDE2900F6E3A59D6D8326A374DEDEC44C2369A1DD2EEEC24`
- `r2-file-mode-original-compat-build.log` `8E4A59D302FF1A94727C1EC313FCB71378B9E1917526241A814EC4C01DB3DF71`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 64 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ddb0657098c33c4992c52616e731168b89cf2c6a | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的旧 VM 实现与受检文件会话分流 | outcome: partial | AutoUI `ddb0657098c33c4992c52616e731168b89cf2c6a` → `d9d609425d2423309a09919723f802fc072a7da2` | AutoLang `a890b41b178528854255b4a5e40b8b0eec8bd79b` → `b0ddc187da4ea650eec8f1543ecbd114459bdb91` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 受检 AGE 在组件处理器中识别实际旧轨判断 `Env.track() == "vm"` 与其反向判断。未激活分支保留原源位置但退出代码生成和导入引用集。局部或模块声明遮蔽 `Env` 时保留普通解析语义。AutoUI 使用检查器给出的引用集收集明确的包成员导入。原 027 的旧 VM 环境、目录、文件操作和系统打开都通过上述原语义的轨别判断保护，受检路径使用现有目录与修改会话。
- 定向用例通过 1/1，0.26 s，验证 AV 与 SFC 读取检查后保留分支、有效引用不能藏过源检查、局部遮蔽生效。AutoUI 组件组 30/30，24.83 s，准备 35.23 s。AutoLang `ace-core --test conformance plan981_` 首次发现上轮已交 watch 与 `ui.focus` 改过的 AGE grammar ID 未同步冻结断言。按实际源码重基线后通过 250/250，1.03 s。其余相邻 grammar 冻结断言通过。
- 通过 `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 构建 1m44s。原 `app.at` 与隔离件 SHA-256 都为 `E1D21A3F58F1D177327C5EF0EE7177B3B666901EB8BFD53EEF2BF664CF99C480`。正式 `AUTO_UI_ACE_VUE=1 auto gen` 首次在 3331 ms 后拒绝 `collection_view::collection_view` 的未知 `auto` 模块，经过实源完成了上一轮仅在诊断件中越过的文件模块边界。原兼容 Vue 生成与 Vite 构建通过，后者 5.30 s。
- 旧 VM 的当前实窗尝试和替换回第 62、63、64 轮入口与集合模块的三个对照都启动了 MCP，但未发布首帧。同一工具的第 60 轮完整隔离来源在 4727 ms 内发布了目录。当前尚无旧 VM 原入口复验通过的证据，需继续定位此源差异与首帧故障，并在受检端接通集合实例令牌和其余宿主值。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-file-track-focused.log` `02CA6088289D7F87C28DC451B61EEC00540652E49DE5666D9C04CD16BB360742`
- `r2-file-track-group.log` `E768BAB5D3A6AD2C3808272C6E08DA327A39C138454308F4D7B306D3AD6AF218`
- `r2-file-track-source-group.log` `7C2F4B876F1A004A5965A5C6B8EC94C0ACCEC277751551E986A7666C90CBE14E`
- `r2-file-track-source-group-final.log` `FBB1D0B473FDFFC11270A5973863C900D0FEB5EA15DB083D0FFBB55B7C15FC4B`
- `r2-file-track-tool-build.log` `BBAF82A454C9503767CC0516C2BCAC2272683880ACC7B22ED81F93D60D09CD4C`
- `r2-file-track-original-strict.log` `5FC513E40A9041748B6AEB06032E68C8078F99DDE6355996BAFE42B63766F550`
- `r2-file-track-original-compat-build.log` `A7259B889B7AC9C91ED9808834C2FD9311B9F80C3B4ACB362C6A9BAF27FE2F39`
- `r2-file-track-native-final-vm-launch.log` `63F5E428F383C5E1F3280C7B4A70E157C1EA062EA4579C5985EA2A34F112EDFD`
- `r2-file-track-native-before-vm-launch.log` `A332EE58E4722112BE59B75601E6D84A47A6A858CAB1DF3319040AE60D0B5720`
- `r2-file-track-native-next-control-vm-launch.log` `8663B69CCE200228CEE4F1C34D75DD78FD8A76B075552D6DF6FA28D33E5A2D1B`
- `r2-file-track-native-pre-typed-vm-launch.log` `54B2195A6F1C8787C96A0A8821941DB582126544C7237C22C4942F2976EA7D3F`
- `r2-file-track-native-baseline.log` `E3DA53DEA14ADF855599F30F1F122A9DE2C281229B3F79D830027733A46FCD33`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 65 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: d9d609425d2423309a09919723f802fc072a7da2 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 集合视图的宿主实例上下文 | outcome: partial | AutoUI `d9d609425d2423309a09919723f802fc072a7da2` → `6a68fadee2b0b71eb38f1a51cf38caeefd7caa66` | AutoLang `b0ddc187da4ea650eec8f1543ecbd114459bdb91` → `8ff12730facf74ec4d5b8bd9a0d8da7dcc39dfff` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 集合视图从 `bind "widget:context"` 取得挂载令牌与窗口宽度。检查器要求绑定到只写入两个对应类型模型字段的 `(str, int)` 处理器。AV 与 A2Vue 在挂载时分配令牌，并在初始化和派发事件前注入上下文。两端均限制直接事件向宿主处理器传参。旧 VM 和兼容 Vue 通过原有轨别判断保留实例命名和旧存储处理。原 027 的哨兵行改用等值的显式 `FileManagerRow` 构造。
- AV 与 SFC 的上下文定向测试通过 1/1，完整组件测试通过 31/31，AutoLang 源码一致性测试通过 250/250。用同一份定向组件源从严格 Vue CLI 生成 20 个文件并通过 Vite 构建。Chrome 回放验证 700 与 1150 像素下的事件宽度、并列实例令牌独立、同次挂载令牌稳定、重挂载和重载后换令牌。
- 原 027 的 `AUTO_UI_ACE_VUE=1 auto gen` 越过上一轮集合包未知 `auto` 模块，并经哨兵行显式类型进入源码检查。4440 ms 后退出 1。当前错误包括布尔值传入 `FStr`、缺少 `self` 记录作用域及 `Str.starts_with` 源码能力。原入口的严格 Vue 生成和实窗验收仍待完成。
- 逐目录核对上一轮无首帧诊断件的 `.at` 源码与可启动对照一致。在已有实测目录中以 `auto run --render vm --server vm --merged` 跑当前原入口，3267 ms 内验证首帧、隐藏项目、无效路径保留原视图、后退恢复，并实测创建空文件和拒绝同名。当前和冻结基线启动时总计均为 `1.9 GB`，后退后两者都显示总计 `6 B`，而文件行含 `1.9 GB`。该汇总差异仍未闭合。隔离文件与覆盖源已恢复。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-widget-context-tool-build.log` `3C7EA16CC650DE8D7AC70B0E2103E1D24E021D4738CDE9C42DC5C8C2FFE04BCD`
- `r2-widget-context-test-final-build.log` `8962F446809599B524B17095C83A3C532EE66B3577F3D75D627E083D391B257C`
- `r2-widget-context-focused.log` `558B57D2E55FE47394FACDD9AFBC63CD4AD61DBB4BFDFF85D1CCC11979B5D077`
- `r2-widget-context-group.log` `676FA83E9C84E81F4CAD6853F3856F2291ABA308908511AF89F8A4FE9555D616`
- `r2-widget-context-ace-source.log` `22053D6E7BF74C9C17D430A3BCF01B64F729671F8AF63317A2E9CBD0F3883F3A`
- `r2-widget-context-web-gen.log` `558B235DA8CD507B1987C96F26718E0309D4D90986708348308025CB2B6AB83C`
- `r2-widget-context-web-build.log` `41F76D6B36C95B9428A045F173590E60AE7DEF9D826D273B1B18209D643E58D6`
- `r2-widget-context-web-browser.log` `BF868380C97C240ED4FF5CF94727B7E65E4C9B4F1AACF81732AC9D82779949C3`
- `r2-widget-context-original-strict.log` `D13980D495C1E46D5F302AD51185E8A9C78609E33A9BEF02D7EDC357F2AB3219`
- `r2-widget-context-original-final.log` `BB401F76173343AE4DE08F1B61642F5D26577A58E076A9AADBA7AFF2C5009F86`
- `r2-widget-context-legacy-summary-initial.log` `3369B0C0B59C8202358ACFD8D2117E35290D48F693988796298175609003BBBF`
- `r2-widget-context-legacy-baseline.log` `279323A2927F47E9547165FBB28C7C73D673E5ED219BB5E74D874042163CCB2A`
- `r2-widget-context-legacy.log` `F0B58907826F32F631AF1BF459EFCF431AA9308C47756C6B66518DD1F3644EFD`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 66 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 6a68fadee2b0b71eb38f1a51cf38caeefd7caa66 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 受检属性和无状态组件转发 | outcome: partial | AutoUI `6a68fadee2b0b71eb38f1a51cf38caeefd7caa66` → `b0242e459bd85a203e7838f26b65900bb087863a` | AutoLang `8ff12730facf74ec4d5b8bd9a0d8da7dcc39dfff` → `e0aa235294869f3b4fe2131d4b7d3a62ea41ea67` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- AGE 按布尔值检查侧栏 `active` 与确认框 `open`，沿用已有视图值编码，并接通共享文本库的 `starts_with`。源码诊断保留原模块和行列。集合视图焦点恢复复用键盘处理的配置校验。文件版本和缩略图身份保留原文本拼接顺序，改用受检插值，取出列表行的属性文字时显式复制。
- AV 为无状态父组件的作者属性计算器传入已检查的父输入，修复 `Thumbnail` 向 `ThumbnailContent` 转发时缺参。以同形的无状态包装和有状态子组件扩展原回归。AV 定向 1/1、组件组 31/31，Chrome 从严格 CLI 生成并构建 21 文件后回放通过。布尔属性与 `starts_with` 定向 1/1，覆盖空前缀、Unicode 和较长前缀。源码组 250/250，A2X 具名拒绝 2/2 与片段发射 5/5。
- 原 `app.at` 与隔离副本一致，当前版本的严格 `auto gen` 通过此前的布尔、字段作用域、文本类型及所有权检查，13.259 s 后停在 A2X 发射。具名目标是 `module.global.get:g3_appm3_app_col_tight`，来自原样式常量 `col_tight`。严格浏览器仍未达到原入口的 Vite 与运行验收。
- 从当前原入口运行 `ace_node_desktop_window --source … --file-root …`。默认链接栈在主线程加载阶段溢出。仅在诊断版执行器用 `/STACK:16777216` 复现后，先得到转发属性缺参，再由本轮修复推进到运行视图的 MIR 所有权预算。原根视图估算 `56992320` 字节，超过冻结上限 `16777216`。诊断保留原视图和文件授权范围，最终退出 4，尚无 AV 首帧。分解生成视图或减少 MIR 槽位与分支规模后需重测默认宿主和实窗。
- 旧 VM 原入口当前源码复跑保留返回目录的真实行，包括 `1.9 GB` 行，并完成空文件创建与同名拒绝。聚合仍为第 66 轮同样的 `6 B`，大小汇总尚待核对。隔离源与临时文件均已恢复。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-view-values-tool-final-build.log` `4B2ECD615399861D3EDBA31EF36F73431BA6EA78F44319EF7B0F8D26AC0DCCC2`
- `r2-view-values-ace-source.log` `A40218BDEDB0E158177BC7A33C835AC8AAE7D746DE7650D2361BA47952164371`
- `r2-view-values-a2x.log` `4C6C659D5F68AF54A2B3A87A02D7F795CF2A6BE3F5233746F862920EB76DF583`
- `r2-view-values-focused-first.log` `3D442083D9D5EA68ED73486879D11294489305266B221EE34E654D4BEBF130EE`
- `r2-view-values-final-wrapper-focused.log` `8B0DCE327F902F870E398104FD5334A742515DBD25E19757E8C11491B5C0579F`
- `r2-view-values-final-regression.log` `661F5D869013A0FDA0ACF25464386EDFBCBEE51220F78E9AC96CB8B737BDB2EC`
- `r2-view-values-wrapper-gen.log` `D6256A1C8B035B131C6623386A4C61C7897FB44806785D513137E65DE4BBAA08`
- `r2-view-values-wrapper-build.log` `3539374C3187E92E1BD7188D3F5EE0D95374AAE5D42FDEE2F39A9580CDD2740D`
- `r2-view-values-wrapper-browser.log` `49CDA268141B5BCCC1B82FFE0A8D7188439C9D364A84CFFB455A8AC9D9D748DE`
- `r2-view-values-legacy-without-conversion.log` `E189DC45940AFC8C6C7191820CF0BC82FCB57F20331B21D5A1CE88BFA90DD444`
- `r2-view-values-original-first.log` `A10A6319B22F391F07ABEA73A939B65296B9D76E0F83234702B1D792043585A9`
- `r2-view-values-original-a2x.log` `670B1D5E1BB139B29620D0F796316D858892019E078AF00A078BB76534F76F4A`
- `r2-view-values-original-native.log` `20BF208CC741B285376B906D278EE6227FCD235FAACB422A32C156E875DA2359`
- `r2-view-values-native-props-build.log` `A8DFF5091A97BC49FB2A270D5E014DB69AFF6A324985DCFF53994AE888D7A16C`
- `r2-view-values-original-native-stack.log` `91B71B4F191B65270F23B36BD3B21EEA75F3204F118D4583B3EBCD941F1E333F`
- `r2-view-values-original-native-props.log` `18D98081A662186D1AB6A440E4C94BE3BCC192F4BAA19872DB90F8E3626D9326`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 67 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: b0242e459bd85a203e7838f26b65900bb087863a | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的 A2Vue 生成、目录和新建 | outcome: partial | AutoUI `b0242e459bd85a203e7838f26b65900bb087863a` → `18180309ffd29ef971ac0c1b90f245a0e241ca61` | AutoLang `e0aa235294869f3b4fe2131d4b7d3a62ea41ea67` → `fde922bff1c76c48789722d2cd190df8a5d478b9` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 受检 A2Vue 按绑定器的初始化顺序在 Vue 实例内建立不可变标量和文本常量，原 `col_tight` 通过其受检访问器读取。常量文本在帧级资源清理后仍可读。A2X 将归入拥有位置的记录文本字段和列表文本项物化为独立值。AGE 对已有文本赋给模型字段采用按值复制，解决目录行清理与打开地址栏暴露的两处双重释放。真实 Node 覆盖跨模块常量依赖、Unicode 文本、两次独立装载及字段复制。
- 受检浏览器视图接通原 `input` 文本消息、回车处理、激活中的侧栏项、缩略图元素与确认框。确认框由已挂载的 `dialog` 打开。`loader-circle` 在原项目锁定版本 Lucide 0.312.0 映射到实际导出的 `Loader2`。A2X 三个测试目标通过 11/11，源码组 250/250。确认框挂载调整前的组件组 31/31，最终挂载路径由原入口 Chrome 复验。
- 原 `app.at` 及依赖 `.at` 与工作树相同。隔离冷目录的严格 `AUTO_UI_ACE_VUE=1 auto gen` 生成 27 个文件并退出 0。确认框挂载调整前的冷生成 40872 ms，最终启动重新生成 `App.vue`，该文件 SHA-256 为 `A1DB22C711E0372A7A235A6BD72B49A095EAF3F9E0EB35C39B3C723C7D0E4614`，`pnpm exec vite build` 2.93 s 退出 0。标准 `pnpm run build` 的对照在 `vue-tsc` 退出 2，涉及兼容产物 `CollectionView.vue`、`ThumbnailContent.vue` 以及对 `App.vue` 的模块解析，仍需接通标准构建。
- `auto run --render vue --server vm` 从原入口启动真实文件服务，设定隔离授权目录及 `create` 写权限，26.533 s 就绪。Chrome 验证首帧、真实文件行、搜索隐藏和恢复、拒绝访问授权根外目录且保留列表。通过原新建确认框和回车创建空文件，并以实际磁盘和页面行相互核对。两个标签页取得不同会话，关闭一个后旧会话返回 404，另一个保留目录。回放无页面异常，进程与临时创建文件已清理。
- 027 的其余原交互及文件操作仍待回放。AV 继续从第 67 轮的原入口加载和 MIR 所有权验证预算问题推进。T-02 尚未结项。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-style-constants-a2x-copy.log` `F58D510896638CF85E46712CCBCFD585A41CE83957E977572E313F1B82706EA4`
- `r2-style-constants-copy-source.log` `84240A0E66559F14C239DB47B28CFAAA6A5DC7DC96C260BDF11451683B2C3AE5`
- `r2-style-constants-runtime-copy-group.log` `D3586D4B1579144CBF8713729EC45F09C9C296B2B0F62EEE7F0EAD22CFC0C0C5`
- `r2-style-constants-diagnostics-browser.log` `1A21BB7E60368F566AE6C04287296D9940C961BD8350C9E51D6065D29E2807A5`
- `r2-style-constants-owned-browser.log` `E128BFF89338CA104E92E77FFFD55F8EED1431D71D0FA52EBC797F1F7988E202`
- `r2-style-constants-copy-browser.log` `52D974DAD50F433D7C25035F29222B2FE743C85B86246A806B6074B9DCD9B6E4`
- `r2-style-constants-dialog-build.log` `CB2C8FB11495A3A9B774675E0F91F8E5D4FD45B3975FB112EDE5A301F2D56683`
- `r2-style-constants-dialog-vite.log` `B169673359708EDA173E399C9FECD6176F7D19ABFEDB89688D1FC11B619CD83E`
- `r2-style-constants-dialog-browser.log` `CCED7F0B551762F68579C1BA164779E78B3A0B0A35FFEA3179EFA146B5741210`
- `r2-style-constants-dialog-launch.log` `9DBB76506FE4E694AF3E126C1268664CD5127E57E933723EED3B858B7DB09E97`
- `r2-style-constants-delivery-cold.log` `DD30A5D40B9F042389B3A576BCD94FA675C9E31A8D3BE3AF78791C87A9B8411E`
- `r2-style-constants-delivery-package.log` `162B59C31D2190C4C8DD53C33F1B18431F4EDE63B90B9FEE14AB92298C2A499B`
- `r2-style-constants-delivery-vite.log` `F9DE12127EAFA0CFD11CB91D083046A91F921EE5EFF14426F39D898B00A18575`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 68 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 18180309ffd29ef971ac0c1b90f245a0e241ca61 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 的受检 AV 视图与 Iced 实窗读目录 | outcome: partial | AutoUI `18180309ffd29ef971ac0c1b90f245a0e241ca61` → `15bdb83d85410cca70aa6b04572395d5b6998a2d` | AutoLang `fde922bff1c76c48789722d2cd190df8a5d478b9` → `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- AGE 在不捕获循环局部量的作用域按约 96 个视图单元拆分生成函数。每段按原顺序追加到共享列表，使用原模型值和已检查的组件输入。AV 所有权验证仍沿用原 16 MiB 上限。拆分后原 027 通过先前超预算的根视图，验证器随后定位到具体的借用返回与文本转换。027 和集合视图的短文本帮助函数返回拥有文本，AV 将拥有文本的 `.to(Str)` 降为复制，并保留对文本视图的物化。
- 原生入口在 16 MiB 工作线程编译，挂载与 Iced 保留在调用线程。试验将挂载也移出后，现有窗口宽度回归读到默认的 `1024`，并且原入口仍溢出。已撤回挂载线程调整。最终原生组件组 32/32，源码组 250/250，A2X 三组 11/11，MIR lowering 9/9。实际 AV 快照与 Node 测试核对长片段中的顺序、入参和状态切换。
- 在隔离文件授权范围内，用原 027 `app.at` 和仅为诊断执行器设置的 `/STACK:16777216` 启动 Iced，进程退出 0。证据记录 `App` 的 16 个实际渲染帧与 9 次 `BootTick`，会话首代、无错误。终态结束启动，路径处于授权根，显示 `7 个项目` 和 `2.5 GB`。该次未执行文件写入。原源文件 SHA-256 为 `EE3E6B7B1373919E0CD77CD9BB6013F0286618077C49BCD02BCF6CF9B6A3C50F`。
- 普通 Windows 构建已越过 MIR 错误，随后主线程栈溢出，退出码 `-1073741571`。首次 CDB 观察在约 180 s 应用 CPU 时间后结束，尚未取得异常栈。继续定位普通构建的实际调用栈，补齐原入口的原生交互、关闭和复开回放。原 027 的 A2Vue 标准 `pnpm run build` 类型检查及其余操作仍待验收，T-02 保持开放。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-view-fragments-core.log` `991CCA3EEC255ED51AAE1E16040D8E310A17244705E3E6B55503E5AC69BA126D`
- `r2-view-fragments-a2x.log` `F23D0724CF2D99AE921E66EE67449CC9ED99AFA2552A5A28D74389F95FCEE56F`
- `r2-view-fragments-lowering.log` `F367305B0FA6769A5C0AA2FA44663D2CAC211EF635341F2AE1A1B35D6C4A18C1`
- `r2-view-fragments-original-first.log` `C41DD29936CEC6CB948E73D07B3640617B91512B803A3709D43790416FFC26A9`
- `r2-view-fragments-original-return.log` `10512696D6973DC333808EAF10D59919CA6152E77A553C75034C26D4AA628B01`
- `r2-view-fragments-original-conversion.log` `B935FD996C40BAC4FA1F6CAF74E9FB03207D16B54C7BFF4BD66E030370C114A1`
- `r2-view-fragments-original-stack.json` `64049A7E9812B29629A8C1EFA0F820C3EDBC85AAE5E83B7E7BDEBCF95B3E94A8`
- `r2-view-fragments-original-mount.log` `D56072359C28CA9544ED854BD60816E4E9E0A962E404C78C963964150B11AA46`
- `r2-view-fragments-mount-group.log` `040F08309FC2732D7B794D0A1489865D1AB3C4C0621F579874CECFDE4AA56644`
- `r2-view-fragments-restored-group.log` `06E1C36EDD6340D1442FDB18DBF931AAB1AEF69A4E440B54D49B9B7D953B610C`
- `r2-view-fragments-cdb.log` `D801E0262C4B417165426ACC6E873C1BCEB9E866FA93A2AED081854628763F7B`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 69 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 15bdb83d85410cca70aa6b04572395d5b6998a2d | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 普通 Windows 实窗与会话重开 | outcome: partial | AutoUI `15bdb83d85410cca70aa6b04572395d5b6998a2d` → `72bacb6e07f84d74c287931fa05b8b5fed4f2479` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- CDB 用 `-hd` 关闭调试堆并在 `c00000fd` 首次异常断下。捕获到同一 Iced `IntoIcedElement::into_iced` 分派连续递归，Windows 未优化构建每层预留约 190 KiB，溢出点为下一层列子项克隆。将各视图投影分支放入独立栈帧，渲染与输入仍在窗口所属线程。六层列、滚动和行组合的回归在显式 1 MiB 线程上验证实际 Iced 元素。
- `cargo build --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --example ace_node_desktop_window` 退出 0。PE 头实测主线程栈保留 `1048576` 字节，初始提交 `4096` 字节。以原 027 源和隔离授权目录启动，100 帧后退出 0，无错误，9 次 `BootTick` 结束启动。独立读盘核对 7 个可见项目、`2684354564` 字节，实窗状态显示 `7 个项目` 和 `2.5 GB`。
- 同一执行器回放 `--script '80:close,85:late,90:reopen'`，在观察期限内渲染 206 帧，处理 18 次 `BootTick`，退出 0，无桥错误。会话 1 关闭并拒掉 1 个迟到事件。会话 2 获得新世代，未重新授权时为空路径和 0 项，显示「无法打开授权目录」。此回放覆盖窗口存续期间的会话关闭与重开。实际关窗清理和重授后恢复仍待回放。
- 原生组件组 33/33，文件会话组 3/3。实窗对照 004、005、007、010 通过。006 的源码装载在 `GetStarted` 被 `source.age.component-unsupported` 拒绝，未进入渲染，整组 4/5。原 027 的正式 CLI 当前没有示例执行器的文件授权接线。接下来核对正式入口授权与原生输入、对话框，从实窗操作入口完成其余 V-07 回放。A2Vue 标准构建与其余操作仍待验收，T-02 保持开放。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-native-stack-cdb.log` `AD604F40C21AF99890E3CF68A94EEDA532B100090BA0D7166413E8FC29C246FE`
- `r2-native-stack-projection-build.log` `E5F49103BE6CF2647E3943DBFF3EF7230AA70BCCC84AA07C80FD3204FAEBFFD8`
- `r2-native-stack-projection.json` `C32D46D1377212C663901EB3E462E5A3DAB88BA55B5AEF3D59CE000F02B8709A`
- `r2-native-stack-projection-reopen.json` `8F108848F72E7E775C75679427FF48B0A4B05437A98796ECB790842265BDB9D4`
- `r2-native-stack-verified.log` `2AA51D791470A1928631CA6C68C27F83E1DFDA0EED58D618A04AF9992F8F3AC6`
- `r2-native-stack-projection-group.log` `1727363E9797D14CA193CD9C610E71E1BF2501CD107677D75010CA1028AADE7F`
- `r2-native-stack-control-windows.log` `A05ED2BDA43454274C0C34BB7D3A77FF6DCE957CF9D0C992FB6A7AB52B5A6858`
- `r2-native-stack-control-files.log` `E28B8C365EFC71630E22F46610C98D0090DCC83BE1B44207D1020FE557D9AEFD`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 70 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 72bacb6e07f84d74c287931fa05b8b5fed4f2479 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 正式桌面启动授权与宿主退出 | outcome: partial | AutoUI `72bacb6e07f84d74c287931fa05b8b5fed4f2479` → `6973668279bac670f90911c5f201378b365253d5` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 正式 `AUTO_UI_ENGINE=ace auto run --render vm` 的 File Manager 会话复用 Web 的 `AUTO_FILE_MANAGER_ROOT` 和 `AUTO_FILE_MANAGER_WRITES`。相对授权根以项目目录解析，未设置写权限时按只读。受检 File Manager 的文件请求直接进入原生宿主，托管模式沿既有 `run_ui → vm-session` 子进程分离窗口。工具进程树未启动 Rust 或 VM HTTP 后端，外层 CLI 仍会预先显示 pac.at 的通用 split mode 描述。
- 运行时在安装窗口桥前向每个活动实例发放授权。无效目录返回 `ace.desktop.file-grant` 并释放已打开的会话。窗口事件循环返回后，运行时取下桥并关闭 AV 与文件会话。托管的 `q` 经现有 Input 通道进入 Iced 的退出路径。新增回归验证失败后没有活动会话，并在同一个桥上用有效目录重开两个实例。
- `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 退出 0。通过该工具和真实 `auto.exe` 从原 027 项目闭包启动，源文件与工作树逐字节一致。隔离目录内有 `native-scope114.txt` 与 `Documents`，授权配置为相对路径 `probe114-data` 和 `create`。首次回放确认操作系统窗口 `Auto Ui Runtime - Iced`，宿主输入 `q` 后工具结果 `Completed`，CLI 退出 0。最终构建复跑也取得新窗口句柄，宿主退出和 CLI 退出码相同。无效写权限与不存在的授权目录两条对照均退出 1。原生组件组 33/33，文件会话组 3/3。本计划的探针进程均已结束。
- Windows 锁屏，Computer Use 两次返回 `failed to activate captured window`，截图只显示锁屏。尚待使用标题栏关闭，以及直接查看并操作授权列表。下一步继续接原生输入和确认框，完成实际交互、文件操作与恢复回放。A2Vue 标准构建和其余操作仍待验收，T-02 保持开放。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-native-cli-delivery-build.log` `CFE8AA9D449182E035C190AED7FBEE4C826CB860BEAD0E3BA5D8D74F2979B67B`
- `r2-native-cli-group-final.log` `4081F46DD90C07EF91BC4B2973C7DB56964A763EDBB9FA7BC66985C0FCB3BC98`
- `r2-native-cli-host-quit-launch.log` `149B45B65E519A5CB78516865B8362EE60C40939F793AC7595891376F1744E45`
- `r2-native-cli-window-observed.log` `8B95C6FC767181240A5E203B75AF5F140A84FC07DDF79B4624BF50A6A72B45EB`
- `r2-native-cli-host-quit-final-launch.log` `E96E6E64C5EA4375E0E33CA544E97F4C60ADEC7D3D9631729B653B571B85CAF3`
- `r2-native-cli-host-quit-final-driver.log` `F1D893EE2DB1ABA4603356064D6EB4EA52B8A23A2198348999A3B7FEEFDC82DF`
- `r2-native-cli-window-final.log` `3850E28BD5711346028D03EB689A76AE555BF84A339896BBA89B480A8E72E618`
- `r2-native-cli-invalid-launch.log` `6C8D916458195323D2ACCF8E9498BC1BD9E09A124F45CDCBF346D099584A529A`
- `r2-native-cli-missing-launch.log` `5EE64C1A7BEA097082E1F5EA36CE6718C8533BD846B63C448E8BB3E88CDD9090`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 71 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 6973668279bac670f90911c5f201378b365253d5 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 原生输入和文件确认框 | outcome: partial | AutoUI `6973668279bac670f90911c5f201378b365253d5` → `a65a7a1260cede041794ced7a9517a116cb0dbad` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 受检 `input` 按原值渲染，为 `oninput` 产生自带文字的 Iced 消息，为 `onenter` 产生受检提交事件。宿主检查当前会话、实例、帧版本和载荷种类。同一帧已排队的键入与回车可在更新后按原控件、组件及处理器重新解析，关闭或换掉控件时拒绝遗留事件。
- 受检 `alert-dialog` 复用现有 Iced 模态 Popover，动作与取消使用原生按钮。第一轮整页回放发现外层滚动视口把名称输入框裁成零高度，随后将模态提升到窗口层。关闭时确认框内容和动作从受检帧移除。
- `plan114_widget_packages::original_file_manager_iced_dialog_creates_the_typed_name` 装载原 `027` 的完整入口和依赖，从原 `NewFileOpen` 绑定开启模态，用 Iced 控件回放点击名称框、全选、连续键入 `验` 和 `A`、按回车。AV 先接收两条独立文字更新，再派发 `CommitNew`。隔离授权目录出现实际空文件 `验A`，原有文件内容核对通过。重开模态后点击原取消按钮，排队的编辑被拒；关闭之后到达的旧帧编辑也被另行拒绝。
- 原生组件 34/34，文件会话 3/3，控件状态 4/4。默认 Windows 示例执行器重建后从原入口启动 Iced 并退出 0，记录 100 帧、9 次启动计时事件、0 错误，仍读到授权目录 7 个项目和 2.5 GB。新建与取消在 Iced 控件回放中验收，标题栏、原生地址和搜索、其余文件写操作的实际项目回放及 A2Vue 标准构建仍待完成。T-02 保持开放。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-native-input-focused.log` `F211EE4F841CBC3B0E40FC5ACCB0F8FAFC9E4CF1EBB79B046B85E9FD57BFC0F4`
- `r2-native-input-modal-root.log` `12F00B4A84665954D5F3A9D7E780F743D4A32A55F297DB92DAB54045BB1E1711`
- `r2-native-input-final-group.log` `9847668769F617B08954BE1EE7F451D3EA740B3160683509AFD50FC23256AEA0`
- `r2-native-input-window-build.log` `28F166BAFACBA9C6FCC8B09012E3DB28D001F868B5D3CCF4CB4439C0E7E5A369`
- `r2-native-input-window.json` `C32D46D1377212C663901EB3E462E5A3DAB88BA55B5AEF3D59CE000F02B8709A`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 72 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: a65a7a1260cede041794ced7a9517a116cb0dbad | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 A2Vue 标准构建与浏览器文件操作 | outcome: partial | AutoUI `a65a7a1260cede041794ced7a9517a116cb0dbad` → `178070c6b0e75db625927650864ba313a648d5e6` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 已安装的 Volar `resolveModuleName.js` 在 4 MiB 拒绝解析额外扩展名，原 `App.vue` 超过该上限。生成器把超限的受检 SFC 发布为模板、可类型检查的 Vue setup 和独立的 JavaScript 工厂。每次 setup 建立该实例状态。编译器继续检查 AGE/A2X 源，`vue-tsc` 检查项目 TypeScript 和实际导入的 Vue 组件，Tailwind 扫描生成的工厂脚本。
- 受检包生成路径跳过独立兼容依赖 SFC。冷生成 24 文件，`App.vue` 173 字节，`App.ace.js` 5571380 字节，`App.ace.d.ts` 87 字节。实际 `AUTO_UI_ACE_VUE=1 auto gen` 32561 ms 退出 0。标准 `pnpm run build` 的 `vue-tsc && vite build` 退出 0。恢复旧兼容 SFC 文件仍通过标准构建。临时把旧 `CollectionView.vue` 接入 `App.vue` 的实际导入链，`vue-tsc` 如预期以 TS2305 退出 2。生成器工厂回归 1/1，受检路由与独立项目 2/2，lane 门和拒绝面 1/1。
- 在同一隔离原项目移除生成工厂后，经正式 `auto run --render vue --server vm` 启动，23196 ms 就绪。工厂、类型声明与入口哈希均和冷生成一致。真实 Chrome 验证授权目录首帧、搜索隐藏并恢复、根外导航被拒并保留列表。通过原确认框新建空文件、重命名，经原上下文菜单复制并粘贴到授权 `Documents`，逐字节核对两份文件。通过原删除确认框删除副本，源文件和根外文件均保留。返回授权根后重新显示目录列表。关闭一个标签页后旧租约返回 404，第二个标签页继续读取。回放无页面异常，测试文件和进程均已清理，增量运行后标准构建再次退出 0。
- Windows Iced 的原目录导航、列表视图行为和其余文件操作的控件回放仍待完成。A2Vue 尚需按原矩阵核对排序、视图切换及完整权限负控，并复跑原画廊入口。T-02 保持开放。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-a2vue-type-tool-final-build.log` `6F40BCC5200D8BB82C6E97EFFAB2D621EAA3ACAB0C607B2B1E424CF9DCFB81CC`
- `r2-a2vue-type-delivery-cold.log` `B54891836556B0CC13FB38A800048CEB8640F9FAE8DE3F4FF652AF5C170A5628`
- `r2-a2vue-type-delivery-build.log` `906B7961CE2E53C79F548FAB73A7A1E0E6AD72C7FC035DCDEF3FD8B59E9474AB`
- `r2-a2vue-type-warm-with-orphans.log` `C9A2EA1796CEBBF4ABBA8FD84C07C6C913EDE6EAEAF2F94B622CD2478D79207B`
- `r2-a2vue-type-linked-type-negative.log` `3DC823D5056E44EC278E3302905E429B471555C82F4BF1E54320173DDE691876`
- `r2-a2vue-type-browser-run.log` `07D4B918EA783F7F7E370656E36C76FA30D3E23CC5A40AE806C9CDE716257D3B`
- `r2-a2vue-type-delivery-browser-launch.log` `11563A4F48A627D31D92D7DEE9B2C4E49E8AC5F1086B5C40A652BA04DCC50CB4`
- `r2-a2vue-type-post-run-build.log` `D5A7089736998F16E00BA891D2417E97946093C9578B28D3AB1B4904A7F23BDD`
- `r2-a2vue-type-writer.log` `63344DA0641777B782779CA359382BBB86A810905FEF4C982CFD5FFB317375C3`
- `r2-a2vue-type-routes.log` `3DD838CC49D8B71C27C0CF3DD8AFE466C8C3D07A2D7581C069D7BEBB8AA78E6C`
- `r2-a2vue-type-legacy-wiring.log` `FDB1D53CB8CA1704137234EBA8F9D77E7DB7384EA3E12C80B43820909283415C`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 73 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 178070c6b0e75db625927650864ba313a648d5e6 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 Iced 视口与导航以及 A2Vue 权限和原画廊回放 | outcome: partial | AutoUI `178070c6b0e75db625927650864ba313a648d5e6` → `2f6f6f6462f952fa631a5d5bba276622d0c49aba` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 原 027 成功读取 3 个授权条目后，Iced 仍把 `alpha-large.txt` 布局为 `175 × 0`，无可见区域。原生受检包现在让独占窗口的全高根直接获得有限视口，并复用运行时的行、列、网格和滚动布局。原控件回放核到两条真实文件行可见、搜索过滤与清除、网格不同列、列表大小升降序、跨视图切换保持选中身份。
- 原地址编辑器通过 Iced 输入法和回车派发 `SetAddr`、`AddrGo`。根外和撤权请求均维持原路径、条目与选中；重新授权后进入子目录，前进及后退通过原绑定消息核对两边均显示实际文件，根外文件逐字节保持。`plan114_widget_packages` 35/35、`plan114_file_session` 3/3、`control_surface_states` 4/4；补跑 `plan124_routes` 4/4、`plan980_package_sfc` 11/11。
- 默认 Iced 窗口示例重建后从原 `app.at` 启动，两个进程退出均为 0，9 次启动计时事件、0 错误；授权大文件隔离目录仍显示 7 个项目和 2.5 GB。14 秒窗口采集 55 帧，延长到 42 秒上限达到 100 帧。控件回放和原窗口启动分别记录，Windows 实窗人工输入及自然关闭仍待验收。
- 当前受检 A2Vue 工具经原 027 `auto run --render vue --server vm` 27270 ms 就绪。在 Chrome 中大小升序两个样本的 y 位置为 223、281，降序为 339、281，切换网格与列表保持选中。只读会话通过原菜单和确认框拒绝新建、重命名、复制及删除，显示相应错误；磁盘核对原件字节与目标缺席，页面异常 0。启动器未在退出等待内结束，清理本回放的进程树。浏览器租约正常关闭沿用此前正向回放的 404 对照。
- 原画廊 PAC 与两个原页面的临时副本逐字节核对。`auto build --render vue --gen-only` 冷生成 70 条路由、90 文件，77121 ms；材料路径变化后离线刷新 `file:` 锁入口，再离线冻结安装退出 0，`pnpm run build` 7634 ms 退出 0。复跑用例读取当前 Vue 点击回调数组，Chromium 156 完成两条路由轮换。FileTree 行数 10→614→614→10→614→10，选中、badge 和旧回调隔离均通过。TreeView 展开数 2→12→0→2，叶选中 `bottom` 与重开空选中通过，页面异常 0。
- T-02 保持开放，继续核原 027 Iced 重命名、复制、删除及写权限矩阵，之后核实 Windows 原窗口交互和关闭，再进入独立复审。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-native-navigation-bounds.log` `C568EAAC26807D9DA1A13E08D7760F77997CF5C4517D6512DFF310FD94A64E0F`
- `r2-native-navigation-verified-group.log` `D1DC273A328C7DA4DEC3B64732609A17279E143FC9B678651A90641208193319`
- `r2-native-navigation-routes-sfc.log` `1D9A305A03152A9ABFD4DF351968ED7BFF7D16215A9277D651E8B2671DFFC2C3`
- `r2-native-navigation-window-build.log` `17BAB09E9DEB7124C5C09356301371007FEDA6F9CCDF7102CAA9B5FE9A91E68F`
- `r2-native-navigation-window.json` `9BC327D13BCF14CF9F980A5E69D8F407CE0BE1AB83933AD7FC396078734594B2`
- `r2-native-navigation-window-extended.json` `C32D46D1377212C663901EB3E462E5A3DAB88BA55B5AEF3D59CE000F02B8709A`
- `r2-native-navigation-browser-readonly.log` `B7524B0E25AAFA22C213FD8958CAACD754E1D64B60EBB55A9D9532C8E92EF1E1`
- `r2-native-navigation-readonly-launch.log` `C072497D16EDAC15ED4BBAC51E97B54EC2B3067433555B6F5F02B63F519984B6`
- `r2-native-navigation-gallery-cold.log` `8F832FC8CFEE171631A32E6C5D7856AD82FF5A9C096B7660A2A064B2FEC0E679`
- `r2-native-navigation-gallery-install.log` `16065171B66BC9B1A8DE3ADC12E281E808EE695ABCCC3E6D50F522141B7DC924`
- `r2-native-navigation-gallery-build.log` `891971AC572396FF9BE63CB2B10D8984041E4F59465754B9E9FFA1AA9B9738BE`
- `r2-native-navigation-gallery-browser-verified.log` `03FD55A5C10AFB50B03DD7B178D282624A184D490563B0E659EADC9DF48728C4`
- `r2-native-navigation-gallery-browser.py` `367A153C8FDE1D0E199C90453A892203B4DF64ADF90EE39102C22032F1AFDB8F`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 74 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 2f6f6f6462f952fa631a5d5bba276622d0c49aba | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 原 027 写权限与关闭回放、常驻回归；T-03 实窗等待 | outcome: partial | AutoUI `2f6f6f6462f952fa631a5d5bba276622d0c49aba` → `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 原 027 Iced 完整控件回放通过右键、原菜单、输入及确认按钮，实盘核验读取权限默认拒绝新建、重命名、复制、删除；拒绝维持所核路径、条目、选中和原件。发放三项写权限后通过原交互重命名、粘贴副本、删除副本，源文件和根外文件按字节对证。关闭会话后的旧确认被计为一次丢弃，磁盘未变。
- 原应用完整回放独跑 106.70 秒，保留其完整源与结果作为待实窗验收的对照。常驻回归复用同一受检 Iced 宿主，缩小源闭包，分别核对有限视口中的可见文件行、列表网格切换保选中、同帧连续输入后提交与取消、过期输入拒绝。初版未选到受检视图路径；最终以受检包和停用计时器激活该路径。最终代码仅收敛测试覆盖，生产代码沿用第 74 轮。
- 正式 `auto run --render vue --server vm` 在 24123 ms 就绪。Chrome 通过原地址栏进入子目录，再用原工具栏执行后退、前进、后退。同期重验排序和四项只读拒绝，页面异常 0，测试文件与执行进程已清理。
- 原 027 Iced 再次创建唯一 `Auto Ui Runtime - Iced` 窗口。Computer Use 激活返回 `failed to activate captured window`，随后截图显示 Windows 锁屏，未发指针和键盘输入。隔离实例计时退出 0，707 帧、9 次启动事件、0 错误。T-02 的接口与双轨定向正反验证完成，T-03 实窗交互与自然关闭等待解锁；T-06 已有冷构建及原项目浏览器证据，后续与冻结版本对证。

| 实际范围 | 准备与构建 | 执行与结果 |
|---|---|---|
| 原 027 完整 Iced 控件及文件矩阵 | debug 准备 28.57 秒 | 定向 1/1，106.70 秒；后续版本在组件组 35/35 中复验 |
| `checked_iced_viewport_shows_rows_grid_and_modal` | debug 准备 13.35 秒 | 定向 1/1，4.80 秒，低于 10 秒 |
| 最终 `control_surface_states`、`plan114_file_session`、`plan114_widget_packages` | 复用编译产物，Cargo 准备 0.75 秒 | 分别为 4/4、3/3、34/34，对应 0.23、4.52、20.24 秒 |
| 原 027 Chrome 历史与只读权限回放 | 正式启动到 HTTP 就绪 24.123 秒 | 控件、磁盘断言均通过，脚本整体与清理未分别计时 |

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-native-actions-iced-first.log` `72E82445664349F84681231A4B40507A2C680AF4F21ED79D189B2DC0E25F6FBC`
- `r2-native-actions-final.log` `3F02DF596959FCA6EED59C4C61E5D092E43BA170074DBCCD444CB9574E6ADE9D`
- `r2-native-actions-full-original-cases.rs` `77002326914FC33541935CE74A3D165D2CB4946C4E99152AE83229F802981A53`
- `r2-native-actions-fast-v5.log` `23771A449721BF21B7804CC7A8DA938B8EB6B2CF153AE29E25E9D966E3DE07A0`
- `r2-native-actions-daily.log` `7E128A8A5F03586E8F4EFA282579E03109ED315F6187774CCE01F8A7551FEEEA`
- `r2-native-actions-browser-history.log` `7F917AB7B367FFC6A8E7E819DBD7041A3EFE8AA951565E776CA17AEB78F0EBC9`
- `r2-native-navigation-history-launch.log` `AB23C0DF141E1E535B240B65C07C7EA68C91068AE8A5245FFF039DF620E6955A`
- `r2-native-actions-live-lock.txt` `86EF8250DAD7586641493F828336D81B4F129707C97296A690D62F1D331A7A06`
- `r2-native-actions-live.json` `1DF615E226301BB0B046FBFED245E7E0C500347AC919B6DDDC90EE0BC1A7E560`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 75 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ef532be85b7c48b0c0a7eaded190a8ccd18b6c22 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-06 三个原入口的 A2Vue 验收与来源对账 | outcome: partial | AutoUI `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 原 027 的 PAC、前后端源码与原用到的 CollectionView、Themes 包共 31 个跟踪文件，原画廊 PAC 与 `src` 共 91 个跟踪文件。逐个核对隔离验收输入与当前工作树的 SHA-256，122 个全部一致。AutoDown 与 AutoWeb 分别核实在 §4 冻结提交，画廊实际使用的 AutoDown Engine 为 0.5.0，包装清单与固定源码一致。AutoUI 工具协议为 1，执行工具 `auto-ui-tool.exe` SHA-256 为 `48105C1B19F065C1C04DD58BAF38473A41165B9D59D141826110A8F0A94ABDE5`，编译器 Aura Schema SHA-256 为 `948506F7A2FAE32CBACC0495649F916419CB4DB182F603711CC61062CE49DD25`。两套实际安装均为 Vue 3.5.35、Vite 5.4.21、vue-tsc 2.2.12、TypeScript 5.9.3。第 73 轮后只改了原生投影及其回归，A2Vue 工具和服务端的验收收据沿用第 73–75 轮。
- 原 027 的冷生成、标准构建、可写和只读权限矩阵、排序及视图身份、旧租约在关闭后返回 404，沿用已核对的收据。新增 Chrome 157 回放选择真实文件，再从原地址栏请求根外目录，核对原路径、列表和选中同时保留。随后通过地址栏进入授权子目录并后退，错误清除，根外与原文件字节相同，页面异常 0。正式启动就绪 25.184 秒，浏览器断言 6.528 秒，含进程与隔离目录清理共 40.146 秒。
- 原画廊沿用冷生成的 70 条路由产物和标准构建。Chromium 156 复跑 FileTree 的归档展开、badge、选中、收起和重开，旧回调没有改变新实例。TreeView 单枝从 2 展开为 3，再选中 `main.rs`，展开数仍为 3；同浏览器上下文另一个标签页从 2 和空选择启动。关闭第二个标签页后，首个标签页折叠该枝并维持 `main.rs`。随后复跑全展开、全收起及路由重开，页面异常 0，回放共 2.377 秒。首次追加测试把第二页放进单页上下文，按 Playwright 的错误改用共享上下文后通过。
- T-06 完成，交最终复审核验的入口为原 027、`/filetree` 与 `/treeview`。重新读取 Windows 会话 1 的 `WTSSessionInfoEx`，返回 `flags=0`，仍处于锁屏。T-03 的真实窗口输入和自然关闭等待解锁。T-04、T-05、T-07、T-08 仍待执行。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-round76-a2vue-source-freeze.txt` `13677792ABC4D32DE151313BC2D6B1C69BC4A93B849E98D158C8CEAB548FBD06`
- `r2-round76-desktop-session.txt` `E229F789E4E629210319D99348BC9C8321FAD576E1CDA640E9F4034967E78D77`
- `r2-round76-browser-recovery.py` `9A4952BB0E15B6D8EBC0D40CF45E03739EE28905F4B213558AB094B15EA14B20`
- `r2-round76-browser-recovery.log` `7A2842F38DB3A18665FEEAA994F7389EFF4E894B53A8761E7BC480CF8AFC8A66`
- `r2-round76-browser-launch.log` `824A27249F446725B1F536E9E2596E40EE78974E521180A1A8334CB7E7D0E4CB`
- `r2-round76-gallery-browser-final.py` `B37E17036B1E74695729237CB9962527629E341C10BA072CD4F61E4C8E7BA8DD`
- `r2-round76-gallery-browser-final.log` `9C216B22CF06C48CA866986A0CC33FED6A458430EEEDF4E91945DACA4EFB3F9C`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 76 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ef532be85b7c48b0c0a7eaded190a8ccd18b6c22 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 原 027 Windows 实窗输入及隔离磁盘对证 | outcome: partial | AutoUI `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- Windows 会话 1 初次查询返回 `flags=1`。从原 `examples/ui/027-file-manager/src/front/app.at` 启动单实例 `ace_node_desktop_window.exe`，绑定隔离根并授予 `create,copy,rename`。通过实际 HWND 在 Iced 文件列表选中 `alpha-live.txt`，切换网格和列表，切换大小升序与降序。可见次序变更，状态栏和事件日志保持同一文件身份。
- 在原地址栏用键盘输入隔离根外路径并回车。实窗和 `AddrGo` 日志同时核对授权路径、三条列表及原选中保持，状态栏显示拒绝。下一次键盘输入合法子目录后出现 `inside-live.txt`，拒绝消失。原工具栏的后退和前进依次显示根目录和授权子目录。
- 原新建对话框由真实键盘输入 `验收-r77.txt` 并回车。原菜单重命名为 `renamed-r77.txt`，隔离磁盘验证文件为 0 字节且旧名不存在。通过真实右键菜单复制子目录的 `inside-live.txt`，回到授权根并从原状态栏粘贴，实窗新增第四行。独立读取源和副本的 SHA-256 一致，根外对照及原有根文件字节保持。
- 复制后下一次右键返回 `failed to activate captured window`。21:05:11 和 21:07:09 两次读取 Windows 会话均返回 `flags=0`。原生执行器在采集计时器到期时退出 0，总耗时 312.336 秒，记录 9931 帧、42 条事件及 0 错误。T-03 剩余原 027 实窗删除验收和正常关闭，以及原画廊两条树页的实窗交互与正常关闭。
- 在当前提交复建正式 `auto-ui-tool.exe`。`cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 退出 0，Cargo 构建耗时 52.04 秒。最新启动器 SHA-256 为 `48E9B76E099725EF2842DE48901A8C13C78012C236C1F6F0FAB856AA02B8050C`。T-06 对账用的原启动器快照按原哈希 `48105C1B19F065C1C04DD58BAF38473A41165B9D59D141826110A8F0A94ABDE5` 保存在材料目录。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-round77-desktop/observations.txt` `1077B8DC51028C7F610B32D9B5EBB06236531D6F33747484AA7F5706974A1B4A`
- `r2-round77-desktop/disk-readback.txt` `96AD0B474A1643A0FAE3C64C497C4BA6E3563C471E1F9D60C98C8726A5D06630`
- `r2-round77-desktop/process-exit.txt` `7F574474132300DEBE41A1C5ABDB9B013A2F53A0F97F3DCE5D867DA4B0C05F7B`
- `r2-round77-desktop/timeout-evidence.json` `53BFDB11A7F7BF5230AB86E01D1344D5AF8FE67A87D778E8CF05A232C0D83366`
- `r2-round77-native-cli-build.log` `926A3C75D0A4E5605B55DC7D1F0A4A72E914747FC3AA21E647963F62FEEFD4BE`
- `r2-round77-native-cli-build-identity.txt` `EE403D0E9D49BF151441F0D9B817C6570F62593B8D954B1D8B16AF68EFA1C0F1`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 77 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ef532be85b7c48b0c0a7eaded190a8ccd18b6c22 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 正式原生启动器与宿主退出复验 | outcome: partial | AutoUI `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 用第 77 轮复建并校验 SHA-256 的 `auto-ui-tool.exe`，分别在原 027 和原画廊的隔离副本执行 `AUTO_UI_ENGINE=ace auto run --render vm`。入口和依赖中复制的 `.at` 文件均逐字节核对。027 挂载隔离只读根，日志明确记录 `AV/Iced File Manager with host-scoped file access`。原 027 和原画廊分别于 14.077 秒、2.245 秒出现所启动进程子树的窗口句柄。
- 通过正式 CLI 的标准输入 `q` 结束两个托管会话。两条日志均报 `[tool] run vm -> Completed`，进程退出码均为 0，总耗时分别为 14.991 秒和 2.710 秒，两个窗体进程都已退出。桌面锁定期间只读取进程元数据并使用已经交付的宿主命令。Windows 会话 1 在 21:08:34 与 21:10:32 均为 `flags=0`；标题栏正常关闭、027 实窗删除和画廊两个目标页的实窗交互仍待解锁。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-round78-production-native.py` `FCAE40B5515122DDE12BE93BC9CE28B31C7736F2592FA18A784E8CA7A6A2C127`
- `r2-round78-production-native-driver.log` `1E9DC46A1ABA6B0178A613FBA79106E22B1FF70D5C0BDDC39D2F97DE7F630E2B`
- `r2-round78-production-native/file-manager.log` `3E31E4987780AC6F43F8FB9B6B8B8BBFA6BB7DB519D7AF7EFE24B3D9DBD6AA45`
- `r2-round78-production-native/widgets-gallery.log` `DEC65A929013F95A46235F62E12B7D20D3F3D74D56A47707D4EFBAF33B632100`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 78 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ef532be85b7c48b0c0a7eaded190a8ccd18b6c22 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 Windows 实窗验收阻塞复核 | outcome: blocked | AutoUI `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

21:11:28 直接查询 Windows 会话 1 的 `WTSSessionInfoEx` 得 `flags=0`。第 77 轮再次锁屏后，本轮为连续第三轮确认相同阻塞。三个实现工作树干净，T-06 已验收，第 78 轮正式启动和宿主退出的证据已保留。T-03 余项依赖解锁，后续实窗复审与合并依赖 T-03。恢复动作见 §10 的 B-R2-01。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 79 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ef532be85b7c48b0c0a7eaded190a8ccd18b6c22 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 原画廊两树页实窗修复与 027 只读删除对照 | outcome: partial | AutoUI `ef532be85b7c48b0c0a7eaded190a8ccd18b6c22` → `3f1fab0c00dfc2e6f261a7a209f26095c5efe612` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- Windows 会话解锁后，旧版正式画廊实窗显示导航栏，侧栏与 `outlet` 所在区域未取得可用高度。修复让独占窗口的受检路由外壳获得有限视口，给带样式的 `scroll` 和 `sidebar_content` 复用现有滚动投影，并从外壳主题继承背景和前景。保留原 AutoOS PAC、画廊源和业务数据。原生路由回归在 Iced 控件树核对导航、`outlet` 邻接内容和子页可见按钮，保留原会话重开及迟到事件断言。
- 重新构建正式 `auto-ui-tool.exe` 后，从原画廊 PAC 启动。最终工具 SHA-256 为 `A759B135DCA1AD78471CF4D6156FD8896DBEF5720FD75FEA41870BC74C93F388`。真实指针滚动侧栏并进入 `/filetree`。起始 `docs` 与 `docs/plans` 展开，`archive` 收起，badge 611 与 604 可读。展开、选中 `001-vm-function-integration.md`、收起和路由重开均核对通过。进入 `/treeview` 后，`main.rs` 选中时展开数保持 2，单枝展开后为 3且选中保持，全展开后为 12 并显示 `bottom.txt`，选中回传 `bottom`。收起全部后为 0，路由重开为 2 与空选中。两条目标页在最终深色主题下可读，最后点击标题栏关闭，正式 CLI 退出 0。原画廊首页仍显示既有 `source.syntax` 诊断，目标两页走原侧栏。
- 原 027 经正式 `auto run --render vm` 打开由本轮脚本生成的只读隔离根。选中 `delete-probe-r80.txt`，在实际右键菜单与确认框提交只读拒绝对照。状态栏显示文件操作失败，目录、两条文件和原选择保持。分别读取两文件的 SHA-256，均与原字节一致。原 027 经标题栏关闭，正式 CLI 退出 0。原生路由修复未改变 027 单页分支或 A2Vue 入口。
- 构建 `cargo build --locked --offline -j2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 用时 51.14 秒。最终 `cargo test --locked --offline -j2 -p auto-ui-runtime --features ace-bridge --test plan124_routes --test plan114_widget_packages --test plan114_file_session --test control_surface_states -- --test-threads=1` 准备 78 秒，组结果分别为 4/4（2.45 秒）、34/34（32.71 秒）、3/3（4.24 秒）、4/4（0.14 秒）。定向可见性回归单跑 1/1，用时 1.56 秒。
- 锁屏阻塞已解除。T-03 唯一剩余的是发放删除许可后，在原 027 实窗确认删除指定隔离样本并验证磁盘结果。已发出 Computer Use 所需的界面删除即时确认，等待用户选择。

证据位于 `D:/autostack/plan-artifacts/plan-114`，以下为 SHA-256。

- `r2-round80-actual-observations.txt` `E555BF385F7EA4F4EF2214EB0A1F09A69E58E2E7F675D8758C3ECBD5CCFA74E0`
- `r2-round80-gallery-production.log` `FDF2A2A6E012CCD166A0D4511C3E4ACEC120E04481600A523A8E2436BB6F54DF`
- `r2-round80-gallery-exit.txt` `AC3BBEA1B6B9F175AB0A8F0C9F1E53449B0C0682264B7BD29FBE4997EC022078`
- `r2-round80-delete/production.log` `8246F8873E020F82BF83B9F11C6AC4C2EEE0E4FCB2EAB628154C52E95EF59B6D`
- `r2-round80-delete/exit.txt` `1FD89F7EC87472544C3B78277825EE9C4A91BAEAA8FF0FD569CB009C43B18BD1`
- `r2-round80-routed-viewport-v3.log` `1C5E8A4D23906BA24FB5154004189612091647D5C4EB250D0EC16DF8211BB0E0`
- `r2-round80-native-regression.log` `6A8BF714F9852CDDA5D2E20F3DE3032DFDB30ED5436D9B643439CCD50D2DBDBC`
- `r2-round80-themed-producer.log` `F106DD4C7835A41DEE499C4A246545E8FF128DA1FF5F748BA5BD60227E143E8B`
- `r2-round80-gallery-themed.log` `28CF6B3B99BB2432D5D3097E7F87607B9A52E8F318272928DF9C468D7DE08FB7`
- `r2-round80-gallery-themed-exit.txt` `AA575E6E1B081C3DAAA834673D5FCCDC2BFCAFA73015723EF60A89C25524DF19`

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 80 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 最新原生二进制逐例耗时与隔离数据复核 | outcome: partial | AutoUI `3f1fab0c00dfc2e6f261a7a209f26095c5efe612` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 复用第 80 轮四个原生回归二进制和已准备的 ui980 Provider。先核每个二进制 SHA-256，再按 `--exact` 每次单独启动一个用例，测量含进程启动及退出的实际时间。`control_surface_states` 4/4 最长 0.052 秒，`plan114_file_session` 3/3 最长 2.752 秒，`plan114_widget_packages` 34/34 最长 5.030 秒，`plan124_routes` 4/4 最长 1.680 秒。合计 45/45，单例均低于 10 秒，整组 35.861 秒。
- Windows 会话 1 在 21:42:52 返回 `flags=1`。两份隔离文件 SHA-256 与第 80 轮相同。B-R2-02 仍待用户确认界面删除，本轮未启动删除窗口；实窗授权后的删除结果、独立复审和合并仍待完成。

- `r2-round81-native-case-audit.py` SHA-256 `C4CAA48781A24E1ED5F1A4BAAB20619D90B3D151CB883178C15C8250C0E6E66B`。
- `r2-round81-native-case-audit.log` SHA-256 `0398E7720B0D5828831346E781E018CF173EDE9BC962AAAF0156DE8EBD1DB59B`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 81 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 隔离文件实窗删除确认阻塞复核 | outcome: blocked | AutoUI `3f1fab0c00dfc2e6f261a7a209f26095c5efe612` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

第 80–82 轮连续缺少对 B-R2-02 指定隔离文件的界面永久删除确认。两份隔离文件的 SHA-256 仍与第 80 轮一致，验证窗口均已退出，三个实现工作树干净。T-06 和已通过的 T-03 验收证据保持可复查。T-03 的最后一项及其后的独立复审和交付等待用户确认。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 82 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | next: user

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 授权删除成功及 AV 实窗验收完成 | outcome: pass | AutoUI `3f1fab0c00dfc2e6f261a7a209f26095c5efe612` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282` | branch: `plan-114-dev`。

- 用户即时确认只允许删除测试脚本创建的 `delete-probe-r80.txt`。在已解锁会话用第 80 轮最终工具 SHA-256 `A759B135DCA1AD78471CF4D6156FD8896DBEF5720FD75FEA41870BC74C93F388` 启动原 027 PAC，命令为 `AUTO_UI_ENGINE=ace auto run --render vm`，同一隔离根仅授予 `delete`。
- HWND `84608832` 通过原右键菜单打开确认框并核对文件名，点击确认后状态栏显示操作完成，列表由 2 项变为 1 项、26 B。磁盘读回证实目标已消失，`keep-r80.txt` 内容哈希 `C9128FE81EF70C120F641228B28DF9BD9AC7334343A35FBCD84889934D2B1FD3` 与删除前一致。用标题栏关闭，CLI 输出 `[tool] run vm -> Completed` 且退出 0，连同启动、操作及关闭总耗时 52.042 秒。
- 保留第 77、80、81 轮其余原生实窗、拒绝与控件回归，第 73–76 轮 A2Vue 记录继续适用。此轮只验证原应用，没有新增测试用例。T-03 完成，进入 T-04 的 AV 复审。

- `r2-round83-delete-live.log` SHA-256 `922EE05E47B1252A9DCBAD447DDE0C96DECD487294C58628CE241B1771BA7FD2`。
- `r2-round83-delete-exit.txt` SHA-256 `ABB9FE41A0B66C934D3AD28D5B7BD700719099E6601E9528AF4EE8779DBB5CB5`。
- `r2-round83-delete-disk-readback.txt` SHA-256 `49A114DE2AE9C15193F48A4546DBFEB9C685245828371FF5F69288820E2B16BB`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 83 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | next: review/T-04

2026-11-21 stage: review | review_scope: phase T-04 | reviewer: 实施会话内按冻结源码和原始记录重核，独立上下文复审尚待 | plan_id: PLAN-114 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-114/auto-ui | branch: plan-114-dev | reviewed_commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | base_commit: 9bdbbcf859e5185335014425af5e74f5512e8c29 | dependencies: AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f`, AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- AC-01 的原入口、依赖身份及输入对账沿用第 76、80、83 轮。AC-02 原窗口行为、生命周期和授权删除正负磁盘对照均已验，回归准备的负对照发现 F-114-R84-01。AC-03 的冷生成、标准构建及浏览器记录暂沿用，终审再核。当前 canonical SD-01–03 输入 SHA-256 分别为 `F35774BFD64C13B6B3994014B4C7982372D63F6203ECE49C6DA6146FEF7AF85D`、`98C9A6C3322BB4FE6784ECEAFA30FAE1B603277ECC523445A32C84B59BB916B7`、`8AB7017DAFDFDE55ED0BF121DF3F15244D7F2BA1E5BB76E9603EC12B00D219BF`。
- F-114-R84-01（AC-02、T-02，需修复）。在清除 `ACE_NODE_DESKTOP_PREPARED_RUSTC` 与 `ACE_NODE_DESKTOP_ROOT` 后，直接执行已冻结的 `plan114_widget_packages` 测试二进制，精确运行 `checked_iced_viewport_shows_rows_grid_and_modal`，实际退出 0 并报告 1 例通过、0.00 秒。该用例在准备变量缺失时提前返回，未执行 Iced 断言。同模块有相同的空缺分支。测试纪律要求依赖未准备时返回具名失败。修复后用同一未准备负控及准备好的正向原生组复验。新跑的负控 0.00 秒，不替代第 81 轮的实际 5.030 秒正控。`r2-round83-review-missing-provider.log` SHA-256 `12FACE030DE7139038AB8CFC67607A82D2FEF296285D9C6983B4CA84719BFBD1`。
- 集成预检 `git merge-tree --write-tree --name-only` 使用 AutoUI 主线 `eacf2dc5bc002181e3a8c03448a1e052c904ab32`、AutoLang 主线 `eb04da7b2806ea727ec7d8f09f3b709af476ce0d`、AutoOS 主线 `680a6ae344f89cbf4dcf5efd25a118889fe611c7`。前两仓存在 20、27 个冲突文件，AutoOS 无内容冲突。整合后仍需在同组工作树复验行为；统一组是后续合并门，当前阶段未运行。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 84 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3f1fab0c00dfc2e6f261a7a209f26095c5efe612 | next: work/T-02

2026-11-21 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-02 修复 F-114-R84-01 | outcome: pass | AutoUI `1599a1ead8e90a2987d7aa88958b9ef96fede6f3` | AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- `plan114_widget_packages` 的原生断言统一要求已准备的 ui980 Provider。移除提前返回及静默跳过，保留现有断言。清除准备变量后重复第 84 轮的精确用例，退出 101，明确报 `ace.ui980.provider.not-prepared`。测试二进制离线编译通过。
- 使用同一已准备 Provider 分别运行 45 项原生控件、文件会话及路由测试，全部通过。最长单例 5.156 秒，总计 35.151 秒。变更仅限测试代码，第 73–83 轮的 A2Vue 和实窗证据继续适用。F-114-R84-01 修复，T-02 恢复完成。
- `r2-round85-provider-preflight-build.log` SHA-256 `09EEA9D9370D9387C02CE1F55BE8075CED0690E60638FB8DFCA9E79AF02063FB`。
- `r2-round85-provider-preflight-negative.log` SHA-256 `E38F978A8F5A15FFF86D469A3C7272F1EB1FC646DB2FD0158412DFAC8AA9FD75`。
- `r2-round85-native-case-audit.log` SHA-256 `DFF3A650D017C5E231044F9289CE979B6F2F8C0E353D905DF929963FCAE276F0`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 85 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1599a1ead8e90a2987d7aa88958b9ef96fede6f3 | next: review/T-04

2026-11-21 stage: review | review_scope: phase T-04 | reviewer: 实施会话内修复复核 | plan_id: PLAN-114 | plan_revision: r2 | reviewed_commit: 1599a1ead8e90a2987d7aa88958b9ef96fede6f3 | outcome: blocked | next: B-R2-03、B-R2-04。

- 对同一离线编译二进制清除准备变量后，以 `--exact <用例> --nocapture` 分别重放 `checked_iced_viewport_shows_rows_grid_and_modal`、`checked_file_mutation_commits_only_a_live_authorized_event`、`stateless_widget_forwards_each_site_to_its_authored_channel`。三例均退出 101，报告一例失败及 `ace.ui980.provider.not-prepared`，墙钟用时分别 92、208、171 ms。首例与两项同机制用例覆盖 F-114-R84-01。第 85 轮准备后的 45 项正向结果沿用。此轮未新增用例。AC-01 及 AC-02/03 的产品证据未因测试代码变更失效。AC-04 的独立 AV 阶段复审和终审仍待执行。
- 三个实现检出干净。第 84 轮记录的主线 SHA 未变。重新运行 `git merge-tree --write-tree --name-only <default> HEAD`，AutoUI、AutoLang 分别有 19、28 个冲突路径，AutoOS 无内容冲突。三个默认检出都有其它工作的未提交改动，详见 B-R2-04。AC-05 的整合、合并门、主线验证和交付均未开始。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 86 | from: review | to: review | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1599a1ead8e90a2987d7aa88958b9ef96fede6f3 | next: user

2026-11-22 stage: review | review_scope: phase T-04 | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: 用户授权独立子代理复审并保留其他计划的未提交改动。

- 独立复审使用原同组工作树。预检的默认分支为 AutoUI `eacf2dc5bc002181e3a8c03448a1e052c904ab32`、AutoLang `b260b6e65311e7092c0f4a710df7e1efe317fd64`、AutoOS `680a6ae344f89cbf4dcf5efd25a118889fe611c7`。执行 `git merge-tree --write-tree --name-only --no-messages <default> HEAD`，三仓分别退出 1、1、0。AutoUI 的 19 条冲突路径涉及受检宿主、路由、SFC、原 027 及 CollectionView，AutoLang 的 28 条涉及 AGE 前门、文法、AV、A2TS 及其测试。AutoOS 计划改动的 6 条画廊路径和主线图表改动不相交。
- 主检出未提交路径与本计划分支差量的交集逐仓均为空，未提交改动保持。加载现有 `scripts/test-groups.toml` 核对拟用固定组。除先前的 ACE 包组，`ace-syntax` 覆盖共享文法，`ace-lowering` 覆盖 HIR 适配，`stdlib` 纳入 `file_session`，`vm` 覆盖 VM 调用与算术。固定组仍待审定及实际执行。

2026-11-22 stage: review | review_scope: phase T-04 | reviewer: `/root/t04_review` 独立上下文 | plan_id: PLAN-114 | plan_revision: r2 | outcome: needs_fix | branch: `plan-114-dev` | worktree_path: `D:/autostack/.wt/ui-114` | reviewed_commit: AutoUI `1599a1ead8e90a2987d7aa88958b9ef96fede6f3`, AutoLang `fe7c556a576fb915a69d34b463d72c44b847be7f`, AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- AC-01 通过。31 个 AutoUI、91 个 AutoOS 冻结源文件分别对账当前工作树和正式运行副本，全部匹配。R77、R78、R80、R83 与 R85 原始记录哈希匹配。已有实窗的导航、排序、授权、写盘、树页、关闭证据保持。
- F-114-T04-01（AC-02、T-02/T-03）。原 027 `OpenItem` 在兼容 VM 使用桌面关联或系统默认应用，受检 AV 会显示仅 VM 支持。`checked_project_only_opens_legacy_vm_dependencies_on_the_legacy_track` 精确复跑证实两路实际分流，319 ms。第 1359 行记录的受检 AV 本地文件打开待接线未闭合。补授权根内普通文件打开，使用原实窗打开动作和目标应用观测，增加授权外及关闭后仍无打开的负对照，保留目录行为和 Vue 的提示。
- F-114-T04-02（AC-02、T-02）。从 AutoLang `a3655eb2f` 起新增 30 个 Rust 用例，其中 28 个涉及 AV 和共享域。部分记录只有组耗时，包括 `route_outlet_retains_its_properties` 与 `p114_package_projects_deep_typed_tree_data_with_the_standard_worker_stack`。现有 ACE AV 单元测试二进制早于 `fe7c556a5` 的会话测试变更。需要用明确准备的 debug/test 二进制逐例补墙钟和输入身份。未观察到新用例超时。
- F-114-R84-01 已闭环。复审精确运行 7 次，无构建。缺准备及工具链身份不符的三个负控实际退出 101，用时 83、780、251 ms。准备好的文件会话、Iced 视口与模态、路由生命周期、兼容 VM 分流 4 项通过，用时 470、5455、739、319 ms。R85 的 45 项原生回归沿用，最长 5.156 s。
- SD-01–03 的输入哈希与 R84 一致。`### 规范增量` 后至 `### T-01` 的 UTF-8 LF 摘要 SHA-256 为 `B11B8635552AA119E8BE7EB054E1027E3A5F5C5531D30AB0593C37B83C66946F`，修复后补原生打开边界。待合并组 `ace-source ace-syntax ace-av ace-lowering ace-a2x ace-stdlib ace-abi stdlib vm`，另核 `ace-semantics` 的源码查询消费者。当前复审未执行合并门、AC-03 终审或 AC-05 交付。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 87 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1599a1ead8e90a2987d7aa88958b9ef96fede6f3 | next: work/T-02,T-03

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: F-114-T04-01 接通普通文件打开及定向回验 | outcome: partial | AutoUI `1010a7702f53f8799e50f2248f8fc7ade47159c8` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- 027 保留原打开条目事件和兼容 VM 桌面关联，受检 AV 向当前文件会话提交 `file:open`。宿主按持有的根逐级检查目录和文件身份、拒绝链接与非普通文件，核对宿主回复可发布后，通过 Windows 文件关联打开。A2Vue 接同一作者事件和回复面，直接显示桌面提示。
- AutoLang `cargo test --locked --offline -j 2 -p auto-lang-core --lib --no-run` 准备 47.065 s。已准备二进制 `auto_lang_core-e095d82ffbdb2b50.exe` SHA-256 `B9EBD2C85B13A18D4C5617C533D3A697D7DC06AA8817D38ADE1A221EC024B38A`。新增 `file_open_target_requires_a_live_grant_and_a_regular_entry` 通过，149 ms，用独立回调验证授权根中的 Unicode 普通文件确实送入打开器，无授权、越界、父路径、目录联接、非普通文件、撤权和关闭均未调用打开器。四个相邻用例通过，单例 16–178 ms。
- AutoUI 原生 4 组离线 `--no-run` 准备 85.846 s。新增 `checked_file_open_rejects_forged_replies_and_released_mounts` 精确运行 366 ms，验证关闭后和伪造回复拒绝、找不到文件的错误终态、SFC 的无外部打开结果与错型绑定诊断。正式 `cargo build --locked --offline -j 2 -p auto-ui-tools --features ace-vue --bin auto-ui-tool` 准备 61.097 s。工具 SHA-256 `F5DC7529ED0A24759B43EE51A869EBBB9EA8FF21FC6230987A9FAE2C52F05012`。
- 隔离原 027 源的 `src/front/app.at` SHA-256 `6880398D42D45BC65FE87837DC3F9AF7EED3B379BBCAE4201E1F51DC9BCC7584`。由正式 `auto run --render vm` 创建原生窗口。Windows 锁屏，Computer Use 激活失败且截图确认锁屏，未发送点击。停止本轮专属窗口进程后，CLI 退出 1，墙钟 67.737 s。尚需实窗双击文件、确认默认应用并正常关闭。
- 正式 A2Vue `auto run --render vue --server vm` 冷准备 40.906 s。Chrome 157 双击同时出现桌面提示、保持原行、未发送文件 API 请求、0 页错误且文件字节不变，6.622 s。浏览器关闭后结束本轮专属服务，整命令 56.067 s。复用了旧实窗及非打开项的产品验收证据；本轮未宣称原生实窗打开成功。
- `r2-round88-file-open-ui-test.log` SHA-256 `89E91504FCBDB29292A865CB315E75D47D9B8F917C70BFE58B2667E37EE613F7`。`r2-round88-file-open/{desktop.log,desktop-exit.txt,browser-proof.log}` SHA-256 分别为 `154D48795DFB42B8B1B5AF380E3EF4C2513D1560C438D891ED8E3332B9E346C9`、`52E7448770BBF164701B3FBB45A0E95538A3206ABCEBFE4673C6E30E9740C509`、`7E30AA6881FE5BFD5B6E1699588870AD9EDE637AC31E4C72541748A07D1EB16A`。
- 同一生成目录标准 `pnpm run build` 实际执行 `vue-tsc && vite build`，退出 0，合计 7.507 s。Vite 报单个 chunk 578.90 kB 的大小提示。`r2-round88-file-open/browser-build.log` SHA-256 `77F3452309D0215A1A42AE0FD4FA4EA00E0A8A0ED7DD75AA3E18403AFDA6D0CB`。

F-114-T04-02 的复核输入以 `a3655eb2ff2638cdbecc002b7bfd74122d6ab43a` 为 AutoLang 增量基线。逐个发现已显式离线准备的 debug/test 二进制，再 `--exact --test-threads=1 --nocapture` 跑 31 条新增 Rust 用例和 4 条修改的既有轨用例。35/35 通过。执行日志包含二进制 SHA-256、逐例命令作用域和墙钟，SHA-256 `B598C3431C74B25B893DE69EAD613237226C56E9C4B8927006BF69B77EFBC3D5`。以下最长单例包括其中的用例设置与进程退出，组清理未另计。

| 实际模块 | 通过 | 准备 | 独立发现 | 最长单例 |
|---|---:|---:|---:|---:|
| `auto-lang-core` | 5/5 | 47.065 s | 22 ms | 193 ms |
| `ace-core` | 15/15 | 12.689 s | 84 + 64 ms | 344 ms |
| `ace-av` | 10/10 | 36.356 s | 95 + 16 ms | 207 ms |
| `ace-av-core` | 1/1 | 8.213 s | 117 ms | 10 ms |
| `ace-source-query` | 1/1 | 14.091 s | 77 ms | 59 ms |
| `ace-a2x-ts` | 3/3 | 3.981 s | 76 + 79 ms | 185 ms |

- `route_outlet_retains_its_properties` 为 37 ms，`p114_package_projects_deep_typed_tree_data_with_the_standard_worker_stack` 为 39 ms，`live_widget_sessions_keep_current_models_and_refuse_invalid_transitions` 按本轮 `ace-av` 正式准备二进制跑为 103 ms。
- 同一准备好的 ui980 驱动逐例跑 AutoUI 46/46，覆盖 `control_surface_states`、`plan114_file_session`、`plan114_widget_packages`、`plan124_routes`。整段 38.427 s，发现/清理未另计，最长视口/模态 4.992 s，新文件打开拒绝用例 335 ms，均不超过 10 s。`r2-round88-native-case-audit.log` SHA-256 `C3606CFF8F397BD5F834385749AF8D36D14AC8B91FCE588B2EF53BC049735B3E`。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 88 | from: work | to: review | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1010a7702f53f8799e50f2248f8fc7ade47159c8 | next: review/T-04

2026-11-22 stage: review | review_scope: phase T-04 repair | reviewer: `/root/t04_review` 独立上下文 | plan_id: PLAN-114 | plan_revision: r2 | outcome: blocked | reviewed_commit: AutoUI `1010a7702f53f8799e50f2248f8fc7ade47159c8`, AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2`, AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- F-114-T04-02 已闭环。独立对账 R88 准备记录、已准备二进制哈希、源码差量和逐例运行记录。35 项 AutoLang 与 46 项 AutoUI 原子结果均通过，复审未重跑构建或测试。
- F-114-T04-01 的源码和定向回归复审通过。实际原 027、授权会话与 Windows 文件关联接线对证，越界、联接、撤权、关闭、伪造回复、失败和 A2Vue 界面提示证据齐全，未发现新增产品缺陷。锁屏时未观察原 027 实窗成功打开文件及自然关闭，T-03、T-04 保持开放。待 B-R2-05 解锁后完成最后实窗验证，再独立收口。T-05 及后续合并门尚未执行。
- 三个计划检出干净。三个默认分支仍为第 87 轮预检版本。再次核对尚未提交的其它计划文件，与当前新提交的差量无重叠，均保留。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 89 | from: review | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1010a7702f53f8799e50f2248f8fc7ade47159c8 | next: B-R2-05,work/T-03

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: T-03 原 027 正式实窗打开普通文件 | AutoUI `1010a7702f53f8799e50f2248f8fc7ade47159c8` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- `AUTO_UI_ENGINE=ace auto run --render vm` 从与实现树相同的原 027 副本启动，工具 SHA-256 `F5DC7529ED0A24759B43EE51A869EBBB9EA8FF21FC6230987A9FAE2C52F05012`，作者入口 SHA-256 `6880398D42D45BC65FE87837DC3F9AF7EED3B379BBCAE4201E1F51DC9BCC7584`。宿主授权单一隔离目录，未设置写权限。Iced 窗口 HWND `28246202` 列出唯一 59 B 的 `plan114-doc-open-control.txt`。
- 首次对文件名发送一次双击，本次只观察到选中，未观察到默认应用启动。随后从同一文件行点击原「打开」按钮，Windows 记事本 HWND `45747176` 显示确切文件名和内容 `PLAN-114 original File Manager system association control`，027 状态栏显示已调用系统默认程序打开该文件。仅关闭测试标签页，027 通过标题栏关闭。CLI 报 `[tool] run vm -> Completed`，退出 0，整条命令 103.137 s。读回原文件 59 B，SHA-256 `CEFF3C48935479BFE37EBBDEF91404FC817CF94E19737B50D9357F0638C0A2AE` 与启动前相同。
- `r2-round90-desktop.log` SHA-256 `80B42B107D8E9597EAE2A192FDF10392876A0E1C5C3ACD9923E48F801648F39F`。`r2-round90-desktop-exit.txt` SHA-256 `FC3ED77C54F2343CF897121E13CECBFD7B799A4DB75EB6098BD191EAE84266C8`。交独立复审当前实窗证据及双击观察。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 90 | from: work | to: review | status: executing | outcome: partial | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1010a7702f53f8799e50f2248f8fc7ade47159c8 | next: review/T-04

2026-11-22 stage: review | review_scope: phase T-04 | reviewer: `/root/t04_review` 独立上下文 | plan_id: PLAN-114 | plan_revision: r2 | outcome: needs_fix | reviewed_commit: AutoUI `1010a7702f53f8799e50f2248f8fc7ade47159c8`, AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2`, AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- F-114-T04-01 闭环。独立复审接受 R90 原行内打开、目标记事本、真实内容及 CLI 关闭证据。F-114-T04-02 已闭环。
- F-114-T04-03（AC-02、T-02/T-03）。原 CollectionView 文件行的单击选择和双击默认动作可在同帧入队。宿主按序派发，选择提交下一帧后，已排队的普通动作因帧版本变更被丢弃。已有 Iced 控件用例未按实际顺序派发全部同帧消息。计划同一工具、原项目、权限和文件再开第二次，HWND `14090450`，双击只显示选中，列表未变，无测试文件的记事本窗口。标题栏正常关闭，CLI 退出 0，60.891 s。`r2-round91-desktop.log` SHA-256 `5A7B3BB5A2DFF41286E8C8D17FE1B192FC4244A80E79505340C73C3F74447BB7`，`r2-round91-desktop-exit.txt` `66145FE34EFC9275A0216F56EEFB0A64A75DC27B7264F9721BE9CE6D24A65DDF`。
- 修复按来源键和受检动作重查已入队的双击。正向覆盖同帧选择先提交，负向覆盖条目、动作或所属会话已变化。修复后跑原实窗双击。原 AV 默认应用打开结论继续保留。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 91 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 1010a7702f53f8799e50f2248f8fc7ade47159c8 | next: work/T-02,T-03

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: F-114-T04-03 同帧入队及实窗复测 | outcome: partial | AutoUI `4cc57fdbc8fb506d4ff12e9518fdf8edbe75f4a4` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- 仅已在当前帧获准入队且有作者键的双击复用队列绑定重查。在同一挂载验证相同控件键、消息、参数和实例，条目替换及旧会话仍拒绝。新增 `checked_keyed_double_click_survives_selection_but_rejects_replaced_items` 复现 Iced 同帧先选择、后双击，结果依次派发 Select 及 Open 并保发现约束，替换与重开负控通过。debug/test `--no-run` 准备 106.782 s，精确回放 2.753 s。`r2-round92-batched-doubleclick.log` SHA-256 `A0C95985F55D13417ACEC89F0E951BB448018D1C26DEAD9D8F8296651336CF24`。
- 正式工具重建 72.278 s，SHA-256 `A7CA302FB412A6EA07A5E5AEE4271EB815BB4EDE40F73845923655AFC1CB0220`。原 027 真窗 HWND `17565084` 两次测试双击仍只选中，未出现记事本，标题栏正常关闭、CLI 退出 0，97.922 s。该试验不能确认实际鼠标事件是否被识别成双击。下一步限本树的调试构建跟踪 Iced 指针识别与宿主入队，保留第 90 轮行打开正向证据。`r2-round92-desktop.log` SHA-256 `57F28CD264734B52E57524C52D58383C310C6607B8EBA5B7F611C8D8CDD864A0`，`r2-round92-desktop-exit.txt` `0F398D62AF9B248BDA3DCFE63B2994D1FEBD24950F60A87F1D03B6002893C323`。

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: F-114-T04-03 原生消息计时诊断 | outcome: partial。

- Iced 原窗口两次显式收到同坐标按下，按处理时钟相差约 755、820 ms，均分类为 `Single`，只入队 `onclick`。另一次记录 Windows 消息时间分别为 `285865343`、`285865531` ms，真实消息间隔 188 ms，第二次处理滞后 594 ms，Iced 处理间隔约 784 ms。Windows 双击设置为 500 ms，Iced 使用 300 ms 的处理时刻窗口。没有默认动作进入已修复的队列层。两次诊断均走原 027 正式 CLI，窗口标题栏关闭、退出 0，分别为 102.102、93.484 s。
- `r2-round93-desktop.log` SHA-256 `0218C64FA8365A130BD4FF0EF328F2D88E5366F238EC86571AB99DECC248617B`，`r2-round94-desktop.log` `8EB1F22F67F8A4E3D0C3563E338870704EBE76D8ED2E2CE607745EF1B06C3DE6`。临时诊断未入库。正式修复在受检独立 Iced 窗口采集原生按下时间并使用当前系统间隔，继续由已验证的同帧事件绑定将双击交给相同条目。

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: F-114-T04-03 原 027 Windows 实窗双击正向 | AutoUI `ab19a4644978b1a3d1449b5ef776a4f6f5130b0f` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- Windows 受检独立 Iced 窗口在实际 HWND 的消息回调记录按下原生时标，Iced 根据时标和系统原有双击间隔处理。正式工具 debug 构建 77.110 s，SHA-256 `28D847CBB6ED65A81C3BC45F305506207ABFD3D38D94B8116D57A3B9EDFBD944`。原作者项目隔离只读根及文件沿用 R90。调试跟踪均移出正式源。
- 正式 `AUTO_UI_ENGINE=ace auto run --render vm` 启动 027 HWND `37621162`，在原文件名位置发送一次双击。系统默认 Windows 记事本 HWND `45747176` 显示确切文件名 `plan114-doc-open-control.txt` 和合成文件正文，027 状态栏显示已调用系统默认程序打开该文件。关闭测试标签页及 027 标题栏，CLI 退出 0、报告 Completed，整条命令 66.097 s，工具窗口进程退出。文件 59 B，SHA-256 `CEFF3C48935479BFE37EBBDEF91404FC817CF94E19737B50D9357F0638C0A2AE` 和 R90 一致。
- `r2-round95-desktop.log` SHA-256 `1CC65F645F243E9E41EF5C948C1488078A823A1E73BED0DD1EBD471B0EADDAA1`，`r2-round95-desktop-exit.txt` `872662E5E015C0F6EF1C5D28EF5D3DE22492FE116A2746FD0E786BF6FF2F8045`。最终定向回归准备与独立复审接续。
- R95 同源离线准备 4 个测试二进制，逐例 47/47，整段 39.195 s，最长 5.087 s，双击回归 1.449 s。`r2-round95-native-case-audit.log` SHA-256 `A6D0A0CDECA1E0603A6E82D47940265F4152181CCF17FB8A71DE1F01D65B60F2`。

2026-11-22 stage: review | review_scope: phase T-04 repair | reviewer: `/root/t04_review` 独立上下文 | plan_id: PLAN-114 | plan_revision: r2 | outcome: needs_fix | AutoUI `ab19a4644978b1a3d1449b5ef776a4f6f5130b0f` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- F-114-T04-03 的同帧和原项目无弹窗双击闭环。独立对账冷准备、数字日志、产物哈希和退出 0。F-114-T04-01/02 沿通过。
- F-114-T04-04（AC-02、T-02/T-03）。Windows 原生按下队列由根视图消费，Iced 模态层捕获「取消」时跳过根视图。模态按下未消费会挤占下一次底层文件行按下的时间点，短间隔双击可能漏派发，之后单击可能错误配成双击。源码路径已定位到原 027 的 `alert-dialog` 与 Iced 覆盖层优先事件规则。关闭条件为覆盖层和根按实际事件顺序配对，正向取消后有效双击，负向随后间隔较长的单击分别处理。独立复审未运行新测试。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 96 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: ab19a4644978b1a3d1449b5ef776a4f6f5130b0f | next: work/T-02,T-03

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | checkpoint: F-114-T04-04 补齐模态和基础视图时标对齐 | outcome: pass | AutoUI `86c04c13ec27ccee1362991c2b8ce18a11afc03b` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- 根包装按 Iced 覆盖层的事件顺序消费原生时标。捕获后的按下由覆盖层持有，未捕获时按序交给随后处理的基础视图。补测原模态取消消耗该次时标、188 ms 原生快击合成与 600 ms 慢击分别处理、替换条目和旧会话拒绝。R97 离线准备 87.198 s，4 套 47/47，37.633 s，最长 5.015 s，修复用例 1.918 s，无超限。`r2-round97-native-case-audit.log` SHA-256 `0D3F7CDB8C17C7B3B5828E3F19616B24883526490DE62E569237A41E70A1D958`。R97 正式工具 debug 构建 63.081 s，SHA-256 `84EF15AF0BBA8179900DABA6EE81044C993D83026FE996037AAE24A856110196`。
- R97 实窗取消弹窗后尝试两次双击，后来显示确切记事本标签页，无法区分是哪次输入导致，故补单一输入对照。R98 同一正式工具启动原 027 HWND `37752234`，打开原「新建文件」并点击「取消」。弹窗关闭后 10440 ms 仅双击原文件名一次，未发送其他输入，双击后 8587 ms 观察到默认记事本 HWND `45747176`、确切文件名与正文，027 报系统默认程序打开。
- 关闭且确认仅测试标签消失。两次单击间隔 7272 ms，第二次后 5392 ms 未出现测试记事本标签。027 标题栏关闭、CLI 报 Completed 并退出 0，95.366 s。工具进程退出，隔离目录仍只有 59 B 原文件，SHA-256 `CEFF3C48935479BFE37EBBDEF91404FC817CF94E19737B50D9357F0638C0A2AE`。`r2-round98-desktop.log` SHA-256 `E6D706E062C162DC2800589A10B5C0238CC47C2FB01847E32280C1D345E9DE96`，`r2-round98-desktop-exit.txt` `66614D9B560409C540641B212E0BC87E3AC44D9CB2847657F916FE5AD5252558`。受影响的原生交互验证完成，交 T-04。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 98 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 86c04c13ec27ccee1362991c2b8ce18a11afc03b | next: review/T-04

2026-11-22 stage: review | review_scope: phase T-04 | reviewer: `/root/t04_review` 独立上下文 | plan_id: PLAN-114 | plan_revision: r2 | outcome: pass | AutoUI `86c04c13ec27ccee1362991c2b8ce18a11afc03b` | AutoLang `52dfcc6b21e120dff750aee771faab19e3cf40b2` | AutoOS `1e02ba0bd61ecf6811d428d58289fad84844f282`。

- AC-01、AC-02 及 AV 阶段 AC-04 通过，F-114-T04-01 至 04 与 F-114-R84-01 闭环。独立核对三仓干净、R98 原入口与实窗正反例、CLI 日志与退出文件、正式工具和作者源身份。复用 R97 AutoUI 47/47 与 R88 AutoLang 35/35 的逐例证据，最长 5.015 s、344 ms，未在复审额外构建或运行。AC-03 既有证据供 T-07 终审。
- 允许进入 AV 的 T-05。待合并组 `ace-source ace-syntax ace-av ace-lowering ace-a2x ace-semantics ace-stdlib ace-abi stdlib vm`。落地前按实际合流后的源码与依赖复验，向 W1 交原 027 与两棵原画廊树。最终终审与合并门未执行。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 99 | from: review | to: merge | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 86c04c13ec27ccee1362991c2b8ce18a11afc03b | next: merge/T-05

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 100 | checkpoint: T-05 专属工作树合流 | outcome: in_progress。

- AutoOS 由独立复审提交重放到当前主线 `680a6ae344f89cbf4dcf5efd25a118889fe611c7`，候选 `734ad776e6ef51ace79507f92bd69cd777b325fe`。`git range-diff` 的 5 个提交全部等价，工作树干净。
- AutoUI、AutoLang 的冻结提交保存在各自 `refs/plan-recovery/114/ui-t04`、`refs/plan-recovery/114/lang-t04`。复审源树与各自聚合提交 `6772cf89038a38b7c1ffc8d4442b0eef22c79156`、`7feced185602aa99d8c2ec7287949fe1c5e7fe7b` 经 `git diff --exit-code` 验证完全相同。候选只在计划工作树重放，基线为 AutoUI `eacf2dc5bc002181e3a8c03448a1e052c904ab32` 与 AutoLang `b260b6e65311e7092c0f4a710df7e1efe317fd64`。共享实现的冲突逐项合流，当前仍有 AutoUI 12 路径、AutoLang 17 路径待处理。
- 三仓默认分支尚未移动。除本 Plan 台账外，默认检出的其他未提交改动继续保留。合流回归、W1 交付和 T-07 仍待完成。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 101 | checkpoint: T-05 合流代码复验 | outcome: in_progress。

- AutoUI 与 AutoLang 的 Git 冲突已逐路径处理，候选仍只在原 Plan 专属工作树。合并实现将 checked 文件会话、列表焦点、指针时标和队列复核接在主线的图表指针与布局处理上。027 的兼容 VM/Vue 源分支保留主线 PromptDialog，受检宿主保留 R98 实窗核过的输入弹窗。用编译器选取宿主已知分支，跳过兼容轨声明对受检闭包的导入。
- AutoLang 定向准备成功。恢复主线局部变量的遮蔽顺序后，`plan981_age_package_av` 37/37，0.719 s，`ts_ui_fragment` 5/5，0.609 s，覆盖现有图表、027 typed listing 和树组件。核心组 `plan981_` 251/253，关联合流发现为条件 class 的布尔检查和未知 timer interval 键，两项已处理并重跑。其它冻结语法身份核对通过，AGE 包面新 ID 为 `29b2191fb50f9f9d96b5d7e9014d322e2f89dd44aa4e5d8d7aca854ee76b44c5`。
- AutoUI 工具与宿主库已过首次类型检查，测试夹具已对齐主线字段与 timer tape 行，待定向准备及两个原目标复验。两仓尚未结束重放。三仓默认分支和其他计划的未提交内容继续保留。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 103 | checkpoint: AutoLang 合流候选 | outcome: in_progress。

- AutoLang 重放完成，候选 `a221dde82e2516df0b893d2e3b15385848d19b5e`，树 `554483a6f1049b1e0e5b76a008d20ab7f99167a0`，父提交 `b260b6e65311e7092c0f4a710df7e1efe317fd64`，候选工作树干净。SD-01 已写于候选，等待交付复核。
- 定向离线准备成功。共享核心/语法 `plan981_` 253/253，1.272 s；AV `plan981_age_package_av` 37/37，0.805 s；A2Vue `ts_ui_fragment` 5/5，0.597 s。对应日志 SHA-256 `593fb5b1e50e433d39c1497c5b343b7726523636c4ff437aa739c4bd2a69e87a`、`90bc82b901b906a160a202136d79cad8b26baf827ce52f72d5ab6a153bc788ce`、`87d9d93fe5185a72fc307a1619b311bdee99cbffff27ae7fc024eedd4b88b8c9`。固定组 `--select-only` 成功，准备与运行分开记账。
- 原 027 兼容对话框的嵌套标题表达式与 JSON 字段内联条件改为同值的变量赋值，已用候选 ACE 解析入口实际检查。受检树含 114 个元素和 65 个 handler，包含原受检对话框，兼容 PromptDialog 分支已按宿主选取。此探针覆盖原源解析及分支投影，完整生成与重放待 AutoUI 候选。SD-02、SD-03 待随 AutoUI 合流复核。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 104 | checkpoint: AutoUI 合流候选及定向审计 | outcome: in_progress。

- AutoUI 候选 `83b784410295af63965f090531a8acbffec50e84`，树 `932f69455abdf7a7ebaf6b60a78a3a5299352cab`，父提交 `eacf2dc5bc002181e3a8c03448a1e052c904ab32`，候选工作树干净。SD-02、SD-03 已写于候选。AutoLang 修复提交 `065f6e001062f01002308235af992441082caa25`。
- 合流首轮 UI 75 例，68 通过、7 失败，已核每例命令终态。两项实窗相关模拟用例通过，包括键绑定双击 1.849 s、视口和模态 4.300 s。失败覆盖原生标签与未挂载依赖、局部读的遮蔽、跨行名称谓词、宿主效果诊断、原面积图与柱图的视图读取、以及顶层具名诊断。修复已提交，正在重建后复验。首轮日志 SHA-256 `15c1aa9389f9c4b6f0b4fc3bcf158fd9b8f82e141fd46ced646ee8c41a4aa45e`。
- 固定组准备起于早一版 AutoLang 候选。第 104 轮修复变更了源树，最终门按新版本完整准备确认。三仓现有默认分支尚未移动。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 105 | checkpoint: 受影响消费者回归 | outcome: pass。

- AutoUI `83b784410295af63965f090531a8acbffec50e84` 对接 AutoLang `065f6e001062f01002308235af992441082caa25`。离线测试准备 89 s，原生文件会话、checked widgets、原生路由、包面 SFC、A2Vue 路由和工具接线 6 套逐例 75/75，最长 4.675 s（`plan114_widget_packages::checked_iced_viewport_shows_rows_grid_and_modal`）。`r2-round105-ui-cases.log` SHA-256 `63165ef065076a32ebf05fa0947427c33e55dc8d780ce8e398b68cef8b989c4a`。正式工具 SHA-256 `AA909D84FE5B1B4D77F1E52E014BE25EE3DD2847414CACB86FBCAA508957BBB9`。
- 原 027 来源闭包按工作树复制 38 文件并逐文件对证。App SHA-256 `66ff7437c46bbac0155240ff4d1452414234a6a010ebd93f19224a67cb73895b`。`auto build --render vue --gen-only` 退出 0，43.394 s，生成 22 文件且 App 含 checked package view。离线 pnpm 安装 14.768 s，`vue-tsc && vite build` 退出 0，48.747 s。原画廊生成器返回 70 路由、90 生成文件。
- 正式固定组的首次准备卡在主线共享 nextest 配置的 ACE-only package filter。新增 AutoLang `63c6bec5f7d69f999b785c31f4d30935782ea518`，在普通 workspace 容许空匹配，在 ACE 保留同一目标包与并发组。固定组执行仍待完成。三仓交付路径复核与默认工作树的其它改动交集为空，默认分支尚未移动。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 108 | checkpoint: 原入口 A2Vue 复验及运行门准备 | outcome: in_progress。

- 候选工具正式启动原 027 的 A2Vue 与 VM 后端。28.950 s 就绪。Chrome 157.0.8103.80 从该会话看到原授权文件，双击同一行只显示桌面使用提示。双击前后行可见，其间零文件 API 请求且零页面错误，文件仍 59 B、SHA-256 `CEFF3C48935479BFE37EBBDEF91404FC817CF94E19737B50D9357F0638C0A2AE`。浏览器核对 6.394 s。测试浏览器关闭，自有预览启动进程在退出等待后按其进程树清理。证据 `r2-round106-original-browser.log` SHA-256 `9938112e1fedbc731971dbeff31c82166362fa31c849538b76f3abfcb558b57c`。
- 原画廊 A2Vue 标准构建退出 0，16.692 s。Chrome 复验在实际原 `/filetree` 与 `/treeview` 页面观察 FileTree 行数 10→614→收起 10、badge 与选择，TreeView 展开数 2→12→0→重开 2。画廊主题、侧栏抽屉与返回的控件状态对证，脱离页的旧监听器不改变新挂载。2.573 s，零浏览器错误。探针读取 Vue 实际保存的函数或函数数组并完整复放，期望结果不变。`r2-round108-gallery-browser.log` SHA-256 `6148bb70858b62ac9c2becb4b37c44c86e03a629cbf9ee1bc48edfe19a26fce2`。
- 关闭受检 Vue 开关后，正式生成原 027 兼容 Vue 100 文件，20.650 s、退出 0，App SFC 引用原 PromptDialog。兼容 Vue 标准构建待核。
- 上游共享 nextest 配置的两个 AAC 专属过滤器已保留等价目标范围。普通 workspace 与 ACE workspace 均实际通过 nextest 二进制列表口。最新 AutoLang 候选 `0d58de3a47edee3f0101118061a0bcaefebcb02d`，树 `807fb1a7c90a9a93557257aecbf985928ad7bd88`。完整固定组准备进行中。
- 合流版正式原 027 已启动 AV/Iced 会话，窗口 `75302074`，Windows 实窗捕获回到锁屏画面。待再次解锁后用同一窗口复验双击并正常退出。第 98 轮的冻结版实窗与 T-04 已通过。已请求 T-07 独立子代理授权，等待用户答复。默认分支未动。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 109 | checkpoint: 兼容构建与固定组对照 | outcome: in_progress。

- 原 027 兼容 Vue 在合流输入上生成成功，标准构建退出 2。用同一工具及依赖对 AutoUI 主线 `eacf2dc5bc002181e3a8c03448a1e052c904ab32` 的原 027 重新生成，`vue-tsc && vite build` 通过。差异涉及兼容轨条件、宿主信号和 CollectionView 的类型来源。修复按原声明生成本地 TS 类型，并隔离桌面文件、控件上下文信号和键盘入口，工具重建中。
- AutoLang 固定组 `--prepare-only --debug-tools` 产出 9 个 invocation。首轮 `--run-prepared` 中，`ace-syntax`、`ace-lowering`、`ace-a2x`、`ace-semantics`、`ace-stdlib` 通过。`ace-source` 973 通过、7 失败，`ace-av` 1558 通过、20 失败，`ace-abi` 300 通过、28 失败。`stdlib vm` 的 archive 发现成功，随后输入盘点无法访问已移除的临时测试二进制，现用普通固定组入口复跑。首轮运行期间源码开始修复，整轮最终返回输入漂移，结果仅用于定位。`r2-round108-fixed-run/results.json` SHA-256 `51108A8A76F02ABA15D0FB428CB2DE4752E82B6676E43B3C491744AF79DD22C7`。
- 语言侧已恢复组件小写别名缺包诊断与 `List<str>` 默认值检查，`plan1072_component_style` 30/30，相关既有图表反例通过。组件包解析按实际载入的清单记录解析结果，包括本次没有活动引用的包。其余失败已保存逐例输出，按主线已知债务、已准备工具和候选回归分别复查。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 112 | checkpoint: 固定组归因与原 027 两种 Vue 生成 | outcome: in_progress。

- 当前候选的普通 `cargo t stdlib` 执行 176 项，通过，另有 24 项按既有条件排除。`cargo t stdlib vm` 普通入口走到了只读盘点，实际报 `legacy.17_cffi.001_strlen.strlen is absent`。该 VM 夹具和盘点输入在计划差量中未变。已从主线确切提交导出控制源，独立复跑同一盘点。
- 主线控制 `ace-core` 的 `text_record_source` 21 项中 16 通过、5 同名失败，对应候选 `verify.double-move`。本轮 `ace-av` 的 13 项命中 PLAN-1055/1094 的预存红记录。另 7 项均报缺 Python provider。用本树 `prepare-av-providers` 产出实际闭包，以同一候选归档的原 7 项复验，7/7，组执行 0.366 s。`ace-abi` 28 项均在 `source_foreign` 族，继续核同提交和诊断。
- 重建工具期间，Windows 仍占用首次合流工具路径，旧 027 进程保留。已从实际编译产物独立取用隔离工具核对。首次修复实现了本地 schema 类型投影，并把宿主信号从键盘映射移出。兼容 Vue 暴露剩余模板条件和字符串复制，已按原方法语义更正。受检 Vue 暴露无活动引用的包清单应保留身份、无需交编译器作为空源模块，修复后重跑两种正式生成与构建。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 113 | checkpoint: 候选正式构建复验 | outcome: in_progress。

- AutoLang 候选 `e99e28b0d70811a8998b3e6c2499d3a9fd59e32a`，树 `210b383edc597d4ed878bee5eb00ba31f62f82c0`。解析定向组 30/30，包含保留缺包诊断、空活动引用及缺具名组件三个对照。主线 `b260b6e65311e7092c0f4a710df7e1efe317fd64` 的只读 VM 盘点复现完全相同的 `legacy.17_cffi.001_strlen.strlen is absent`，失败证据 SHA-256 `E271E7830C49A8AAEDD5A926448542F971067FD009140224B640C0C7803990DA`。
- 候选工具 SHA-256 `0CD0B31FB7D8068E1647DDD9AC704E868C05872C124F7C4689FC364809046966`。原 027 的两种冷目录均逐文件核对 46 个作者文件。兼容 Vue 生成 100 文件用时 18.380 s，标准构建 8.595 s、退出 0。A2Vue 生成 22 文件用时 49.237 s，标准构建退出 0。候选正式 A2Vue 与 VM 后端在 Chrome 157.0.8103.80 再次通过原文件打开提示、零文件 API 请求、零页面错误与文件字节一致，真实浏览器验证 8.965 s。兼容 Vue 真浏览器启动后报未定义项的 `path` 求值，继续定位。
- 为释放测试构建输出路径，核 SHA-256 后将旧工具原样移动至本计划材料；原回归窗口仍是 `75302074`，当前工具进程 ID 保持 `28888`、`66792`。Windows 继续显示锁屏，原合流窗口自然退出仍待重复观察。三个默认分支未移，交付差量与其他未提交路径的交集为空。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 116 | checkpoint: 固定组执行与兼容浏览器修复 | outcome: in_progress。

- 固定组准备与执行均绑定 AutoLang `e99e28b0d`，source_state `4d89ee1003ff49de14d22c236294d2ebd99bdc6e61033490fb3b767fc02f6802`。`ace-syntax`、`ace-lowering`、`ace-a2x`、`ace-semantics`、`ace-stdlib` 通过。`ace-source` 975 通过、5 失败，`ace-av` 1565 通过、13 失败。18 项匹配归档具名基线，`text_record_source` 的五项另在确切主线复现。两项图表恢复，七个 Python 用例亦在正式组通过。
- `ace-abi` 初次为 296 通过、32 失败。25 项与 PLAN-1109 的 `D-1109-ABI-C-RUNTIME-PIN` 清单同名。另三项是 `source_resources` 的分配计量失败，确切主线只读控制在跑。对本轮设置的 `ACE_AV_PROVIDER_ROOT` 恢复 ABI 原环境后，同一归档的四个 Python 会话 `os error 5` 串行重跑 4/4，通过，执行 2.625 s。正式组原失败和复跑分别保留。
- AutoUI 上一候选 7 套逐例 79/79，包含原 PromptDialog 的 4 项。兼容 Vue 真实浏览器将启动故障定位到原 `FileManagerRow` 初始化为 `undefined`，目录处理另发现局部字符串 `path.len/slice` 被误转成 VM-only 调用。已按现有记录构造语义发射初始化器，并只在 `path.extension` 的未遮蔽模块调用使用 VM-only 分支。继续复验原 027 的两种 Vue 生成、构建与浏览器。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 117 | checkpoint: 原入口浏览器与基线复验 | outcome: in_progress。

- AutoUI 候选 `f8ffb398c7cf216e3948212100bc5cc86456f8d4`，树 `649f963f16453fe7a26b5dd51d8b61c27c42079d`。工具 SHA-256 `A53636AA3E9CB49C1FBDA1B0A6D31A00056513F2906D4C4B4CD782C0906B79BB`。类型化记录初始化的新增回归 1/1，0.14 s。AutoUI 7 套逐例 79/79，准备 89.157 s，执行 61.758 s，最长 4.812 s，证据 SHA-256 `521DF3555E1FB07C5BF9744AAA799C6734B8C029CE434298A8022CC30BC0705F`。
- 原 027 的两种正式 Vue 冷生成与构建均退出 0，兼容 Vue 构建 6.930 s，A2Vue 构建 50.156 s。A2Vue Chrome 复验同一隔离文件，双击显示桌面使用提示，其间零文件 API 请求、零浏览器错误、文件字节一致，5.050 s。兼容 Vue 已在真实浏览器显示原授权文件及 59 B，大部分回归闭环。针对双击和行内打开仍无反馈，读回实际 Vue 组件状态，确认 row 与 scope 有效、监听器与请求均存在，计算属性函数的格式字符串参数落成 `undefined`，令 `itemRevision` 被原防陈旧判断拒绝。修复了计算属性调用内的格式字符串发射，正在复验。
- AutoOS 原画廊输入 91 个文件逐一验哈希。候选工具生成并标准构建后，Chrome 对原 `/filetree` 与 `/treeview` 复验通过，展开和重开数、旧监听器丢弃、主题及抽屉行为均保持，零错误，2.485 s。日志 SHA-256 `095C51864A7B9E5437F6C32983115B8EED23DF40086762CFC9C6EF5E91A7895B`。
- `ace-abi` 中三项分配计量失败已在确切 AutoLang 主线源及原依赖上实际复现，三个用例及断言值一致，控制日志 SHA-256 `67A32E28DD0930E13098471066959FFF828EB14991A655DC1579E7A92A4F3C5B`。其余 25 个失败 ID 对应 PLAN-1109 已复现的 C 运行时指纹问题，四项 Python 会话恢复原环境后已绿。整组原结果仍保留并标明基线与重放。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 120 | checkpoint: 两种 Vue 正式入口通过 | outcome: pass。

- AutoUI `7f55c6347fd7df0db5edb8a6dbb316bd6ea8c7c5`，树 `028b29d2a0a1e46b705b28ba758f2d200a17974c`，依赖 AutoLang `e99e28b0d70811a8998b3e6c2499d3a9fd59e32a`。计算属性内格式串参数读当前响应值的新用例 1/1。兼容 Vue 经 `file:open` 向原 `FileOpened` 送桌面限定结果。两种正式生成、标准 TS 构建及浏览器均通过。兼容 Vue 真实浏览器 4.169 s，A2Vue 5.009 s。同一隔离文件双击显示桌面提示，行继续可见、两端零文件 API 请求及零页面错误、原文件哈希一致。收据 SHA-256 分别 `E4E9EE5775B410C7E74A7AFDEA53B70844B6EF6CED2CB1E296C50DEB27B8014A`、`D8DFEC4EA6AADAC817705442E9F245EB6838E1E171759AB6F89F9767A918C724`。
- AutoUI 冻结候选 7 套逐例再过 79/79，日志 SHA-256 `39283E3B2756F6A704B18277A0467ED65BDC9817E24C3E4D70EF01AF50498F3E`。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 121 | checkpoint: 对齐已交 PLAN-1095 | outcome: in_progress。

- AutoLang 默认分支由其他交付推进到 `65ce06b0e9071c00c552896481a9941186caac1f`。与本计划共触 7 文件，唯一 Git 内容冲突为 `plan981_grammar_scope.rs` 的同一文法指纹断言。上一验证版本保存在 `refs/plan-recovery/114/lang-before-1095`。计划专属工作树已重放 5 提交，后 4 项补丁等价，首项保留 1095 的普通 `auto` 节点语法并重算 AGE 包面。合流后候选 `b11452e3a959ec84d2ddb210aff250611f990075`，树 `fbb1275e58c875c70b3b816e5483e2f54d5d7d6a`。AGE 包新 ID `04f644c886e303914b5f93b637f7b79f22cee67abd32d4f0a06e91156d5bb1be`，保留 1095 的 AgeH0 与 AgeComponent 指纹。定向验证及原入口重新构建进行中。
- 合流版原窗口 `75302074` 最后观察仍为锁屏，T-04 冻结证据有效。T-07 独立终审授权尚待答复，T-05 与 T-08 仍开放。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 122 | checkpoint: 新基线消费者复验 | outcome: in_progress。

- 重放后的 `plan981_`、图表组件面、PLAN-1095 普通组合及 `ts_ui_fragment` 定向共 377/377，执行 16.514 s。在确切 1095 基线与本计划候选上用同一语法编译器分别构建 28 个 profile。26 个指纹一致。AGE 包差异沿本计划；`aac` 明确包括 `age-package`，随之从 `28808ef6f57fd63c1a972e9ddaad91b72349f8527d1115cd93e04617fabdbecf` 变为 `f51b85dc495ac0c42af295a0e184e902905c7949084eebcea5fbb8e04bfc981f`。已追加实际 AAC 源码前端消费验证。
- 新基线的普通组编译发现 1095 新增的 `nodes` 字段在六个既有测试夹具中缺省。仅在 114 专属工作树补齐无节点的预期和观测。生产代码保持 `b11452e3a959ec84d2ddb210aff250611f990075` 对应内容。当前 AutoLang 候选 `123f999d919f6022ae65c3c25954060d9e1035da`，树 `c2190ad8ee4f8bcddc45561aee58d96140d182d9`，`stdlib vm` 正式入口正在复跑。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 123 | checkpoint: 合流候选定向证据及 AAC 数据差异 | outcome: in_progress。

- AutoUI `7f55c6347fd7df0db5edb8a6dbb316bd6ea8c7c5` 依赖 AutoLang `123f999d919f6022ae65c3c25954060d9e1035da`，七套逐例 79/79。准备 197.286 s，执行 61.932 s。`r2-round123-ui-cases.log` SHA-256 `E52851B5F6FE60B62F3D0B639895471037BA31F777E415201E433061F8668583`。`cargo t stdlib` 执行 176 项通过，7.97 s。`r2-round123-stdlib/results.json` SHA-256 `888903A3DF6D48C030D150F5A7D4E501DC743F7B403DC5EFAD6D1ACA1C340E1D`。
- 对齐 1095 后用生产提交 `b11452e3a959ec84d2ddb210aff250611f990075` 重建原 027 两种 Vue。原作者输入 46 个文件逐一验哈希，正式冷生成和标准构建通过。Chrome 隔离文件双击均显示桌面使用提示、保留行，零文件 API 请求、零页面错误，文件字节保持。兼容 Vue 浏览器 4.646 s，A2Vue 6.239 s。`r2-round121-legacy-browser.log` 与 `r2-round121-ace-browser.log` 摘要为 `55EC07B42D9C98AFB3AC91B2CCCABA311F3255D6B1AEDBA91A5446BD3714F0AF`、`EF22053EDF6A5E30230B7CFA8E2436DF041262E59E57D5F26FD771D391A1B1DA`。
- `cargo t stdlib vm` 普通入口编译与发现完成，旧 VM 只读盘点仍报 `legacy.17_cffi.001_strlen.strlen is absent`，与 R112 对主线准确提交的实测一致。保留 `r2-round122-vm-stdlib/0/inputs/vm-cache-inputs.log` 和初始失败。
- AAC 使用正式 `prepare-native` 生成可执行程序后，`conformance` 37/51，嵌入文法身份与实际编译结果存在差异。仓库 `generate-grammar` 生成的 `r2-round123-syntax-grammar-data.at` 已在计划产物目录核对，SHA-256 `B9767606FEDA0165ECE19EEDE78FF2E9679E894F7D9107C8A27A1BAA14B6829C`。对八张实际内容表逐一比较，`auto`、`age-package`、`aac` 变化，其余五张字节保持。待同候选的八个固定组结束并固定输入摘要后，将生成物作为单一文件提交并重建 AAC 原生驱动复验。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 124 | checkpoint: 固定组完成与重建 AAC | outcome: in_progress。

- R123 同源码准备并执行八个固定组。准备 418.915 s，执行 264.190 s，准备和执行的 `source_state` 均为 `71a4195e4c2da8f64534acccc17060eb1c4cb5f2bb2aa7c0b3f5d03ca147187e`。`ace-syntax` 159/159、`ace-lowering` 9/9、`ace-a2x` 303/303、`ace-semantics` 295/295、`ace-stdlib` 2/2。`ace-source` 975/980、`ace-av` 1570/1583、`ace-abi` 300/328。逐 ID 比较 R113，未新增失败，原先受 provider 环境影响的四项 ABI 通过。保留三个固定组失败，后续依 R110–117 基线控制和独立终审批定。`r2-round123-fixed-run/results.json` SHA-256 `9316B039E8A379DD9B3B24A1DE36F34C6AC9E2B41252532CFDB4D57E02FDF7F1`。
- 此后仅在计划的 AutoLang 专属树对齐默认 `38bbbe7259755612b0fc32dbb1cf1d4fa1820fa4`，默认新增量只涉及已归档 PLAN-1095 的台账。回退引用 `refs/plan-recovery/114/lang-before-aac-refresh` 保存 `123f999d919f6022ae65c3c25954060d9e1035da`。六提交重放经 `git range-diff` 全部补丁等价。候选 `06d49066f697be04abef82114a95fe2f8f2d6db3`、树 `ea3cc15a7bfbe3ddf251764de8f39c9817a5c2b4`，另提交 R123 的仓库生成 AAC 文法资产，文件 SHA-256 与生成物相同。上述运行源码到此候选的变化仅为上游归档台账及该嵌入数据。八个固定组及 AutoUI 原入口未引用该嵌入数据，AAC 消费者按此新提交重建驱动并执行 `cargo t ace-aac-syntax`。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 125 | checkpoint: AAC 文法身份复验 | outcome: in_progress。

- 正式 `prepare-native` 重建 AAC 原生驱动，75.223 s，驱动 SHA-256 `735493D4748408EA0683DF4ABCCCF76FA910597667311ED5650F1B5F173C894F`。其后使用 `cargo t ace-aac-syntax --prepare-only --debug-tools` 与 `--run-prepared` 执行正式固定组。首次执行 50/51，文法身份、八张表解码及其他语料均通过。唯一失败 `aac1046_corpus_text` 的原生进程等候 8.026 s 超时，日志记录 `prepared AAC native run`，未到 CST 比对。首次结果 SHA-256 `6997528B67EB27547D9E177228F7C78433D5C3D198C16CA3C52D9FABD53E0C34`；使用同驱动、同组选项及原有单例预算串行复跑。
- 重新核对三个专属工作树均干净，当前默认分支是各候选祖先，既有其它未提交路径与交付差量交集为空。三树 `wt-guard` 只读扫描无 reparse point。最新一次可视检查，合流版原 027 的同一实窗 `75302074` 仍显示锁屏。保留原进程及隔离目录。R98 的原实窗和 R99 的独立 T-04 通过仍有效。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 126 | checkpoint: 最新合流候选原画廊浏览器 | outcome: pass。

- AutoOS `734ad776e6ef51ace79507f92bd69cd777b325fe`、树 `476f2c114fe8803952268ea98f489c449524af35` 的原 `pac.at`、作者源码 91 个文件复制后、运行后均逐个对证。使用 R121 构建的候选正式工具，SHA-256 `F8C943388D68702D483FCC9DDFE8F0D5FA364C9BDC1F130FF59E71DFACFB6FA0`。原入口 `auto build --render vue --gen-only` 退出 0，116.422 s，70 条路由、90 个文件。`pnpm install --offline` 14.719 s，标准 `pnpm run build` 16.871 s，均退出 0。
- Chrome 的实际原 `/filetree` 与 `/treeview` 通过。FileTree 展开、收起、badge 与选中对证，TreeView 展开数 2→12→0→2。主题、抽屉和旧页面监听器隔离对证，零页面错误。浏览器 3.987 s，`r2-round126-gallery-browser.log` SHA-256 `760692F162A32891025007220B75EDF4D36B6308EA6AE25227403EBA73873904`。

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 127 | checkpoint: AAC 固定组及候选冻结 | outcome: pass。

- R125 的同一驱动与正式归档完整复跑，`NEXTEST_TEST_THREADS=1`，保留原预算和全部用例，51/51。固定组执行 41.338 s，先前超时的 `aac1046_corpus_text` 5.756 s 完成且校验通过。准备及运行源码摘要均为 `20092001dc0fb38edd9c0485b93600ae419d2c319b3b395bdaf48eade160f59c`。`r2-round127-aac-fixed/results.json` SHA-256 `5CFC9797471B0EDD541512684D482DE32601821C929CD7CD1FD31E9A7FCC2BD5`。
- 供终审冻结的三个候选为 AutoUI `7f55c6347fd7df0db5edb8a6dbb316bd6ea8c7c5`、AutoLang `06d49066f697be04abef82114a95fe2f8f2d6db3`、AutoOS `734ad776e6ef51ace79507f92bd69cd777b325fe`。R98–99 保留原 027 成功打开及退出 0 和独立 AV 复审，R123 及 R126 保留兼容 Vue、A2Vue 原 027 和画廊两页面正式浏览器证据。规范候选 SD-01、SD-02、SD-03 的文件 SHA-256 依次为 `01B404B9F25D62C49CD0D7B2B3E484CAD177973F317793FA662659DC5F416F4E`、`D82FD56CA57F6EFDDC67CB53C229D3CD9F34282200E411C3127A3D677355354F`、`3CF0FE27A67057DDE7660027739B1D83786E2D112F7125A670C15DC796566EAA`。
- 尚待合流版 Windows 同一实窗解锁复验、独立 T-07 授权和审查。`ace-source`、`ace-av`、`ace-abi` 的旧失败与 `vm` 盘点在 R124 及 R123 继续明示。三仓默认检出的其它未提交内容保持，T-05 实际交付以及 T-08 合入归档保持开放。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 127 | from: merge | to: review | status: executing | outcome: waiting | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 7f55c6347fd7df0db5edb8a6dbb316bd6ea8c7c5 | next: user

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 128 | checkpoint: 对账保留实窗与终审候选 | outcome: waiting。

- 确认仍为 R106 同一个原 027 HWND `75302074`，CLI 和受检窗口进程仍在，实窗截图仍为 Windows 锁屏。该窗口使用 R105 正式工具 SHA-256 `AA909D84FE5B1B4D77F1E52E014BE25EE3DD2847414CACB86FBCAA508957BBB9`，`r2-round106-desktop-exit.txt` 尚未写入。
- R105 后，候选变更还涉及共享的组件装载、027 路径提示、CollectionView 焦点/行身份及缩略图参数，AutoLang 也接入了 PLAN-1095。R123 已跑当前源码的文件会话和受检 Iced 逐例，实窗运行证据按各自构建归属。待解锁先完成保持中的原 027 验证，再用 R121 工具 `F8C943388D68702D483FCC9DDFE8F0D5FA364C9BDC1F130FF59E71DFACFB6FA0` 与 `r2-round121-original-027/ace` 中逐文件对证的原入口确认当前候选的双击和退出。最新工具所用生产源码与终审候选之间只有归档台账、测试夹具和 AAC 专用生成数据差异。冻结版 R98–99 的 T-04 证据仍有效。

2026-11-22 stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 129 | from: merge | to: review | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 7f55c6347fd7df0db5edb8a6dbb316bd6ea8c7c5 | next: user

- 确认 CLI 会话仍活动，保留实窗 `75302074` 仍显示锁屏，三仓候选与 R127 相同，交付变更与其它未提交路径的交集均为空。恢复所需的外部条件连续三轮未变化。等待 Windows 解锁，并确认是否授权独立子代理执行 T-07。继续保留原实窗、隔离目录及各计划未提交改动。

2026-11-22 stage: work | plan_id: PLAN-114 | plan_revision: r2 | round: 130 | checkpoint: 用户批准沿用原实窗及准备本次合入 | outcome: in_progress。

- 用户明确以「不需要在验证合流版的窗口了。直接完成merge工作吧」取消合流版窗口的追加交互复验，沿用已验原窗口 R80、R83、R98–99，结合合流候选的 R123 受检 Iced 台账。回收核对时，保留的 R106 CLI 已写出 `exit_code=0`，日志显示 `run vm -> Completed`；没有将其当作此次合流版双击的证据。原隔离文件哈希仍一致。
- AutoLang 默认推进至 PLAN-1096 已归档 `f4b3688bce89dd531186abca1280baba3ab61f54`。新增九个路径仅为语料、语料套件和文档，计划文件交集为空；唯一直接依赖变更套件的固定组是 `case-av-language-parity`，R123、R127 的执行集合不使用该套件。专属工作树七个提交重放全部补丁等价，AutoLang 候选为 `0046408c5bc1225e1d6f899dbabba297137e3ab9`，树为 `a8970f3910503700223cd77364b9795da6d0c0bd`。
- 终审的告警检查发现一处本计划新增的未调用私有函数 `text_directive`，没有实际调用或引用。专属 AutoUI 提交 `3dbdc756d5ed4414a36b49336aa2dcf602ffef5c` 只删除该函数六行，补跑 `plan980_package_sfc`。其余产品行为与 R127 冻结版等价。此前已通过的测试与浏览器证据仍按实际版本记录。

2026-11-22 stage: review | plan_id: PLAN-114 | plan_revision: r2 | round: 131 | review_scope: final | reviewer: Codex，同一实施会话，从冻结源码、测试断言及原始工件重建结论 | outcome: pass。

- 终审对象为 `plan-114-dev` 的 AutoUI `3dbdc756d5ed4414a36b49336aa2dcf602ffef5c`（默认基线 `eacf2dc5bc002181e3a8c03448a1e052c904ab32`）、AutoLang `0046408c5bc1225e1d6f899dbabba297137e3ab9`（默认基线 `f4b3688bce89dd531186abca1280baba3ab61f54`）、AutoOS `734ad776e6ef51ace79507f92bd69cd777b325fe`（默认基线 `680a6ae344f89cbf4dcf5efd25a118889fe611c7`）。AutoDown `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5` 与 AutoWeb `26d085f7e54ea20c6997edf2dc23eef0cb57faf6` 为验证的只读固定依赖。三项规范摘要沿 R127，源码差量均按实际 owner 核对。
- AC-01：原 027 的 `pac.at`、应用、后端 `fs_list/fs_home/fs_drives` 和依赖以及原画廊 `/filetree` 与 `/treeview` 的作者页、`TreeNode` 与组件闭包均有输入映射。浏览器重新核查原 027 46 个文件及画廊 91 个文件的来源；两棵树节点与原 badge 保持，画廊首页已有诊断独立记录。
- AC-02：核 `FileSession::with_open_target` 与目录和写操作、`file_open::open_document` 以及 Iced 消费链。R80 实窗的拒绝保持根、选择及文件，R83 授权隔离删除只影响指定文件，R98 原入口既能由默认记事本打开又能标题栏正常退出，慢单击负控通过。合流候选 R123 含伪造回复、失效挂载、授权写入、模态处理与同键和替换条目反例。两原画廊页的展开、选择、返回和关窗沿 R80。按用户决定复用实窗证据，合流版只单列 R106 CLI 已正常结束。
- AC-03：复核原 027 真实 A2Vue 后端授权写入与只读拒绝、根外失败、重试和旧标签页会话 404 的既有运行记录，结合 R123 最新双 Vue 冷生成、标准 TS 构建、桌面使用提示及零文件请求对照。R126 原画廊两页 3.987 s，展开/收起/重开、选中、badge、旧监听器失效均有实际 Chrome 断言。
- AC-04：按 `file:list`、`file:mutate`、`file:open`、受检路由、实例身份与 Windows 计时分别对照 SD-01–03。变更文件无临时调试输出，三仓差量格式检查通过（AutoLang 按既有 CRLF 判定）。移除无引用的 SFC 私有函数后，`cargo test --offline --locked -j 2 -p auto-ui-runtime --features ace-bridge --test plan980_package_sfc -- --test-threads=1` 19/19，准备和运行合计 149.467 s，测试体 0.60 s，日志 SHA-256 `4BA47CFE85D96188AA6FA13CE9D89C79D3A31AC8033E0B5FE1C377EDAE46DF89`。既有告警与链接器消息保留在原日志，不影响此次差量。
- 限定合入检查采用原固定组完整结果。R124 的 3659 项中 3613 通过、46 失败；`ace-source` 五项按原诊断及独立基线复现，`ace-av` 十三项逐 ID 对应已登记失败（含 `D-1088-LAYOUT-01`），`ace-abi` 二十五项对应 `D-1109-ABI-C-RUNTIME-PIN`、三项分配计量在基线复现。旧 `vm` 盘点缺少 `legacy.17_cffi.001_strlen.strlen` 亦在基线复现。原失败和退出结果保留，判定为仅已知基线失败，新增失败为零。普通 `stdlib` 176/176、AAC 51/51、AutoUI 79/79。此轮仅执行有影响的 SFC 验证，未重跑输入不变的固定组。AC-05 交合入、规范及下游收据。

stage: goal | plan_id: PLAN-114 | plan_revision: r2 | round: 131 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-114/auto-ui | commit: 3dbdc756d5ed4414a36b49336aa2dcf602ffef5c | next: merge

2026-11-22 stage: merge | plan_id: PLAN-114 | plan_revision: r2 | round: 132 | checkpoint: prepared, landed, canonical_verified | gate: known-reds-only | outcome: pass。

- `prepared` 采用 R131 当前终审及 R124、R127、R130 的实际回归记录，三树干净、守卫通过、主线祖先可快进。AutoLang、AutoUI、AutoOS 交付的 64、88、6 个路径与其他未提交改动交集均为空。
- `landed` 按依赖顺序执行 `git merge --ff-only plan-114-dev`。AutoLang `master` → `0046408c5bc1225e1d6f899dbabba297137e3ab9`，AutoUI `master` → `3dbdc756d5ed4414a36b49336aa2dcf602ffef5c`，AutoOS `main` → `734ad776e6ef51ace79507f92bd69cd777b325fe`。逐仓核实际 HEAD 等于冻结交付提交，交付路径的工作区与 HEAD 一致。其他未提交改动按索引状态与文件哈希逐路径比对字节保持，AutoUI 16、AutoLang 26、AutoOS 22。
- `canonical_verified` 核三份规范在实际默认检出的 SHA-256 等于 R127。落地主检出与通过浏览器的验证目录逐文件核对，原 027 及三依赖包共 44 个跟踪文件，原画廊 91 个源/入口文件，全部相同。主检出 `cargo tree --locked --offline -p auto-ui-tools --features ace-vue --depth 1` 退出 0，1.321 s，SHA-256 `F657420E06D5541D8C9EAB846B8FB2F1B8090AE0652EEBF414F7995AD27BD2E1`。
- 当次测试使用冻结的 debug 工具。主检出已有的 debug `auto-ui-tool.exe` 日期为 2026-11-19 00:50（北京时间），早于交付。此次交付为源代码合并与归档。批回归核对 `last_covered_plan_id=1109`，当次收据约 21.41 小时，未达编号或 48 小时触发条件。
- `archived` 归档及 U04 导航登记为 AV 与 A2Vue 下游接收点。实际路径验证 11 条链接、3 条导向、8 个唯一任务。归档时 7/8 完成，T-08 尚待入库后的工作树与专属产物清理回读。

## 10. 待澄清事项

- r1 的 B-01、B-02 经用户授权转为本计划的 T-02 实施项。文件侧的实际 host 凭证、权限校验位置和 A2Vue 后端适配由 T-02 的原入口贯通探针确定。并发触及共享文件时先对账交付次序，当前树页前门切片具备独立开工条件。
- B-R2-01（已解除）第 80 轮 Windows 解锁后，原画廊两目标页完成实窗交互，原 027、原画廊均通过标题栏正常关闭。
- B-R2-02（已解除）用户即时确认允许通过 027 界面删除指定隔离测试文件。第 83 轮读回证实目标消失，对照文件的字节保持，实窗正常关闭。
- B-R2-03（已解除）经用户授权的独立子代理在第 99 轮对当前冻结版本完成 T-04。第 87、91、96 轮发现全部闭环。
- B-R2-04（后续交付）各默认检出有其它计划的未提交记录。用户明确保留。第 87 轮源码路径交集为空。后续线性合并前复核实际交付差量和其它改动，合流及复验在计划自己的工作树内进行。
- B-R2-05（已解除）用户解锁桌面。第 90 轮原 027 行内打开经记事本对证，027 正常退出。第 91 轮独立发现 F-114-T04-03，按 §9 修复双击事件派发。

## 11. 下游交接

| 接收方 | 原行 | 交付收据与消费版本 |
|---|---|---|
| [1071 W1 / AV](../../../../auto-lang/docs/plans/archive/1071-autoui-w1-av.md) | U04 · 原 027、原画廊 `/filetree` 与 `/treeview` | R80、R83、R98–99 的原实窗证据，R123 合流定向验证。以 R132 三仓交付版本为消费入口。 |
| [1072 A2Vue](../../../../auto-lang/docs/plans/archive/1072-autoui-vue-web.md) | U04 · 原 027、原画廊 `/filetree` 与 `/treeview` | R123 原 027 与 R126 原画廊正式生成、构建、真实浏览器回放，含原 027 兼容 Vue 对照。以 R132 三仓交付版本为消费入口。 |

106 和 123 后续可按实际引用复用 R132 的 AGE 包绑定、受检组件和树节点事件，保留各自原页的运行验收。
