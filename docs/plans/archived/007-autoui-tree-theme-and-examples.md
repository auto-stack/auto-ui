---
plan_id: PLAN-007
status: archived
feature_name: autoui-tree-theme-pilot
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-11-02
plan_revision: 4
current_step: 8
total_steps: 10
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/theme-tree-pilot.md, docs/specs/ui/theme-app-migration.md, docs/specs/ui/examples-ownership.md]
touched_goals: []
affects: [auto-ui-compiler, auto-ui-dialect, auto-ui-runtime, auto-lang-test-consumers, auto-man-example-entry, ui-theme-pilot, examples-ui]
---

# [auto-ui/PLAN-007] 树状主题、官方系列与示例归位/首批迁移

> 当前 execution_done（work 阶段 2026-11-02 完成，r4 全 8 个 work 任务闭合，0/10→8/10）；待独立 final review（T-09）与 merge（T-10）。实施组 `.wt/ui-007/`，证据见 §9.1。014/900/901/018已交付；启动条件以§4.2和§5.4为准，不再等待已归档结构包。
> r3历史：本合同迁归 auto-ui 并使用本仓007；语言仓未执行草案已转交，原号释放，不保留第二份活动合同。本计划增加全部 examples/ui 源码归位、现行消费路径修复，再迁移新位置的002/005；其余 app 的新主题接入按后继批次交付，AutoOS 产品仓后继承接。

## 0. 变更摘要

r4接受014/900/901/018正式交付，更新拆分后的实际模块路径、示例剩余清单及905/906交接。保留10个任务、14个AC、10个V、4个SD及原主题/全量后继目标；旧r3的路径/启动/证据政策与现行冲突时，以本修订为准。014的TrackCtx仅接续候选，905研究缺口不自动并入007。

用户同意先约定、再实现试用，并进一步明确必须从 examples/ui 逐 app 迁移、为全部应用提供未来供 AutoOS 默认使用的官方主题系列。本计划交付全部框架示例从语言仓归位 auto-ui/examples/ui、实际入口/测试回接，再交付框架纵向小样、共享官方候选主题包与两个已有 demo 的主题迁移；后继全量主题接入不是可选推广建议。

## 1. 目标

- G1：通过普通 Auto use 与树状数据组织公共 token、应用扩展、主题族和 light/dark 差异；保留可定位的引用关系。
- G2：同一份 Auto 小样在 Vue 与 VM/Iced 真实入口运行，能换两主题/模式，观察局部继承及两个实例隔离。
- G3：官方主题单源维护，两真实 app 先接入；可用性反馈用于修改约定/分批，不取消已确认的 v0.6 examples 全量接入目标。
- G4：全部 examples/ui 受管资产归 auto-ui/examples/ui，当前语言/UI/Man及其他现行入口完成回接；归位后原位置不留活体副本或链接。
- G5：建立逐目录清单及 AutoOS 来源边界，后继逐 app 改主题，不能只改框架或新建实验窗口即停止；不新建 ui-examples 仓。

本计划范围外、由后继明确交付：除002/005外的其余 examples 逐 app 新主题接入（源码归位仍全量属于本计划）与 AutoOS 独立 app/宿主承接。其他非目标：Feature/文档状态树/HMR 总改造、ACE 主线接入、专用内容主题/Figma/DTCG 工具交换、Jet/Ark/RQHost/Linux 主题新支持。公共界面仍需迁移，不能以存在图表/编辑器为由排除整个 app。

## 2. 架构方案

唯一当前约定是 [试用约定 C-01..15](../design/autoui/theme-conventions-and-pilot.md)。[Button 草案](../design/autoui/button-theme-atom.md)提供语义细节；历史 OOP Button 草案、字符串引用与 source/design-scope 不能作为绕行实现。

输入→真实 Auto 模块解析/导出→UI 域 Theme/TokenTree 图与 extend 合成→widget 主题上下文/模式/局部差异→共同校验→Vue 与 VM/Iced 投影→真实交互/属性证据。共享表示的最终归属由 T-01 核实际依赖，不能让纯语言 SDK 反向依赖 UI，不能复制两套不相容解析器。

token/主题定义只读，局部写入归 widget 实例；相同 token 槽引用随当前上下文解析。theme/mode 在同步事件末验证并事务发布；失败保留旧有效设计。新 source 未能挂载时给错误，不发布半成品。模式从原始公共数据重新选择，不继承上次已解析颜色。

示例归属决定与迁移阶段见[示例搬迁约定](../design/autoui/examples-ownership-and-relocation.md)：当前选 auto-ui/examples/ui，不新建 ui-examples；先做原始来源搬迁/路径中性修复，再做主题语义改写。示例归UI不改变语言测试/ACE原有语料义务，也不迁AutoOS产品源码。

## 3. 技术栈

现有 Rust UI 包组、Auto 树状数据、Vue 生成器、VM/Iced renderer。复用现有 ThemeSpec/TokenName/ThemeDecl 与 style recipe 的兼容事实，新增能力由单源数据表示投影；不是替换现有库的完整重写。

## 4. 需求分析与背景调查

### 4.1 取材与观测基线

- 已读语言docs/specs/overview.md（历史总览有迁移前快照）和UI本仓[Specs索引](../specs/README.md)、[结构合同](../specs/ui/structure-closure.md)、[测试映射](../specs/ui/structure-test-map.md)、[编译配置](../specs/build/ui-compilation-configuration.md)、[测试选择](../specs/build/ui-test-selection.md)，以及语言[发行合同](../../../auto-lang/docs/specs/repository-boundaries/generated-ui-distribution.md)。当前本仓已有canonical索引，不再以“尚无Specs”作执行依据；主题增量仍待007实现后沉淀。
- docs/specs/auto-lang/ui/architecture.md ADR-17 与相关 overview：已有语义 token、运行期主题优先级与双端一致性合同。新 system 与局部 scope 是新增提案，不能倒写成旧事实。
- 原草案读取过 auto-lang@4cb9d275；r3 迁移规划重新观测 auto-lang@635c84c944fa47128d4e42f98cad21cce3aa298b、auto-ui@207e4bff8d20bab28cab88a9d99ff17ebaa48e8a、auto-man@e6cc06fbc496f366edde7929abfd49254ec13647。其他会话仍可能更新；未授权提交其WIP。
- r3 前 auto-ui 定向 status 干净；旧README可能滞后。执行时按启动门重核真实提交、依赖与完整读者清单，不机械固定本次观测提交或消费WIP。
- auto-ui/crates/auto-ui-compiler/src/design_tokens/registry.rs：TokenName 31 键封闭词表与 ThemeSpec CSS/VM 单源；新增 app 命名空间必须先设计兼容映射。
- 同目录 decl.rs：ThemeDecl、ComposedTheme、compose，现有 extends/colors 与 light/dark/auto；recipe.rs/tests/design_tokens_recipe.rs 已有 style 导入和校验，不重复宣称从零建设。
- auto-ui/crates/auto-ui-dialect/src/grammar/widget.rs：widget 语法入口；auto-ui-compiler/src/ui_ast.rs：公开 parse/resolve/lower；auto-ui-runtime/src/ui/vm_bridge.rs、style_iced_adapter.rs、system_theme.rs：运行/样式/OS 模式相关接缝。
- crates/auto-atom/src/parser.rs：静态 Atom 不执行 use/变量/表达式；不能新增字符串解释器假装普通模块支持。
- r2 核 examples/ui 的 005-login 已用旧语义颜色与本地 recipe，002-counter 为普通按钮/布局；stylekit 已存在共享配方；auto-os/apps 部分应用已接旧主题链。按[初步清单](../design/autoui/theme-migration-inventory.md)记录37目录/35编号目录，未做新主题运行验收。

- r3 只读统计381个tracked文件/3,263,776字节、37目录（35编号），非忽略未跟踪文件0；UI28个Rust文件、Man3个Rust文件含旧路径，命中包含注释不等于全部是实际读者。已见语言侧media/回归/UI snapshot/AP-16及UI registry/renderer/media真实锚点；必须先分类冻结，不能机械全仓替换。

### 4.2 授权与启动顺序

r1 时仅有小样授权与版本范围询问；r2 新的直接用户指令明确要求“从 examples/ui/ 里的 demo app 开始，一个个改成新的形式，并给所有的 app 都添加统一的主题”，未来作为 AutoOS 官方默认主题系列。现已授权迁移方向/官方主题目标，本合同纳入首批两 app 并保留全量后继；不据此推导提前插队、修改业务功能或无证据的发布完成。

最新直接授权（2026-11-01）：核查并更新007，接收014/900/901/018交付、核清示例迁移剩余范围和905/906交接及启动条件，暂不实施。此前归属auto-ui/examples/ui、全部受管示例归位、首批002/005和官方主题方向继续有效；本轮不创建或联系其他agent/chat、不建树、不改生产/现行Spec、不运行实验或代改905/906进度。

现有明确顺序见 auto-lang/docs/design/strategy/modularization-and-refactoring-strategy.md §2：884→887→Web 有限迁移→UI/Man 结构收口→重审 AutoUI 新架构。876 等已交付历史不重开。本计划不是前述未编号结构包，不消费在途 WIP、不替代其验收。

实施启动须同时满足：①用户明确授权007 r4实施（本轮仅修订）；②核近期有限出口，014和890/900/901/018正式交付已接受，Cargo治理按905结论确认有限验收且取得实际治理结果，研究归档本身不等于关闭；③按§5.4接收906实际落地的来源/SDK/测试通道与锁组合，或用户明确调整先后且冻结可独立验证的无交集范围；④T-01在授权后重绑真实SHA、资源窗口与测试入口。默认先解决相关交接再实施007；不因905/906头部状态推进自动放行。若用户明确调整近期里程碑/实施顺序，在本计划记录具体决定，不由执行者默默豁免。门前仅可读回/修订合同，不能建树或做实施探针。必需工具/系统事件证据缺失保持pending，不用skip充PASS。

2026-11-02 实施授权记录：用户以 `/goal 完成计划007` 直接授权本合同全链交付（work/review/有界修复/merge，含按§8建实施组）。同日只读核验启动门其余条件已满足：905 archived 7/7、906 archived 8/8、910（905结论的实际Cargo治理实施）archived 9/9 且合入落地 lang `f665ce113` / ui `01f99fd` / web `3de5524` / man `67add3c`；auto-ui主检出正处落地提交01f99fd。④由T-01在授权后的实施树内执行。主检出未跟踪项：`docs/design/autoui/`（本合同引用的主题/搬迁设计契约，属计划材料）、`crates/auto-ui-runtime/test/ui/*.png`（测试截图产物，非代码WIP）；auto-lang主检出现存他人docs/design WIP，不消费不提交。

### 4.3 r4交付接受与只读快照（2026-11-01）

本次主检出：lang `174652b5c3b8b9631ec58e8814b4c4b869b5e462`；UI `0b6e74dfe338ea313c14d1852b9e4776e5e5fcc5`；Man `1eeac78036f9c477aa846e1a1bba7ce5407ca128`；Web `ec9c4bac90fed62e6b651df7789153149e8a9473`。均为观察快照，不钉死未来执行版本，不消费其他会话WIP。007 r3原字节SHA256=`0f2fd29348d6e9f1d8fedac94a7c07a08fc4500474bf2f17c1eae661dc1631f6`。UI Cargo.lock/README/022及历史材料既有WIP保留；未发现本仓根或docs子树AGENTS.md，执行前仍重核实际改动路径指令。

| 已交付合同 | 接受的结果 | 007保持/剩余 |
|---|---|---|
| [900](../../../auto-lang/docs/plans/archive/900-de-ui-dependency-inversion.md) archived10/10；lang4b24a157c/UI4e9acd5/Mancbbe657 | 语言零UI依赖，UI管线/LSP/cosmic和28个Rust example目标归UI | 28项在`crates/auto-ui-runtime/examples`，不是`auto-lang/examples/ui`整棵Auto应用；不重复迁Rust目标/语言基础包，余下Auto示例归位由007负责 |
| [901](../../../auto-lang/docs/plans/archive/901-test-feature-flattening.md) archived9/9；lang2fcae41ba/UIe1fe4ca/Webec9c4ba | 语言七test feature退役、TestCase常驻；SDK发行28源码crate+vendor+patch的闭包修复 | 保固定`cargo t <组>`及原条件；28是901发行快照而非007硬编码永恒分母，906落地后重核来源/清单，不恢复test-* |
| [018](archived/018-ui-build-configuration-closure.md) archived8/8；UI968ac83 | runtime稳定基线无条件；ui-iced兼容别名保留、真实可选能力仍可选；三测试feature退出，静态运行选择 | compiler/dialect保持轻依赖；不重新调整默认feature/profile，不让主题用例绕过HTTP/GPU/原生串行/手工条件 |
| [014](archived/014-ui-man-structure-closure.md) archived11/11；UId66627a/Man1eeac78/langd5cfc79c1 | Vue/Aura/renderer与CLI结构、唯一状态/资源owner、公开路径/测试映射已落地 | 007新增语义作用于新模块，不重拆CLI、不搬Web；真实交互/发行证据只证明旧行为，不能代替新主题验收 |

014后的实际改动锚点：compiler `ui_gen/vue/{mod,style_helpers,tags,project_files,library}.rs`及`design_tokens/{registry,decl,recipe}.rs`；runtime `ui/aura_view_builder/{mod,eval,tracked,props,route_memo}.rs`、`ui/iced/renderer/{mod,style_widgets,projection,media,host,subscriptions,shell_state}.rs`。`DynamicComponent.bridge`仍拥有VmBridge，builder仅借用；DesktopSession拥有窗口/app/host/owned退出，主题不得再造owner或为换肤重建业务实例。Man使用`cli_spec.rs`/`cmd_ui.rs`等既有分派接缝。原候选`ui_gen/vue.rs`、`aura_view_builder.rs`、`renderer.rs`已成为目录，不照旧路径新建平行实现。

014交接的TrackCtx三元组（path/id_map/probe）为7文件29签名的可选小步候选，尚未收敛；T-01核主题是否真正需要它。若不需要，保留现有明确owner/穿线并在T-07回交未做理由；不把它加成007必需重构或声称已完成。

### 4.4 示例实际剩余范围

只读`git ls-files examples/ui`复核：语言仓仍381文件、3,263,776字节、37目录（35编号目录）；非忽略未跟踪文件0；UI同路径受管文件0、非忽略未跟踪文件0。因此阶段A仍是全部37目录的来源归位，没有证据支持“900已搬大部分Auto示例”。此数字不是未来固定分母，启动时逐文件hash/许可和新增差额必须重冻；未统计忽略缓存，不将其搬运或清除。

| 类别 | 现有目录/剩余责任 |
|---|---|
| 首批真实主题消费者 | `002-counter`、`005-login`：先来源原样归位/路径修复，再主题改写、原业务双端验收 |
| 其余33个编号目录 | `001-helloworld`、`003-converter`、`004-profile-card`、`006-hero-section`、`007-stats-board`、`008-pricing-table`、`009-article-feed`、`010-contact-form`、`011-calculator`、`012-clock`、`013-todo`、`014-weather`、`015-notes`、`016-calendar`、`017-chat`、`018-book-reader`、`019-video-app`、`020-music-player`、`021-blog-viewer`、`022-kanban`、`023-realworld`、`024-charts`、`025-sys-monitor`、`026-database`、`027-file-manager`、`028-launcher`、`029-photo-gallery`、`030-video-player`、`031-image-viewer`、`031-paint`、`032-handler-async-probe`、`038-minesweeper`、`041-auto-edit`：源码归位和现行读者回接全属007，后继新主题逐app接入另立批次；032按验证例保持义务 |
| 共享库/验证夹具 | `stylekit`、`remote-frame-e2e`：保来源、相对导入、测试与资源，不按业务app强套主题；两个031不合并 |
| 不在搬迁范围 | 900已迁的Rust examples/cosmic/LSP；`examples/capability-tests`等非UI根；AutoOS产品app；忽略的target/node_modules/本机配置。边界读者必要路径适配仍在本计划 |

已核当前实际读者锚点：compiler `src/app_registry.rs`的examples根定位仍经AUTO_LANG_ROOT/兄弟auto-lang/主检出；renderer `mod.rs`的图标coverage测试仍读`../../../auto-lang/examples/ui`；Man `cmd_autodesk.rs`默认`examples/ui`、`cli_spec.rs`帮助仍描述旧workspace根。它们是T-01分类清单的种子，不代表穷举完毕。现有stylekit导入、041的blueprints和015外部依赖、UI媒体/注册表/快照、语言/AP-16/ACE语料/Man与现行脚本读者须逐项确认。仅历史归档/注释不机械改写，已有tests的数量/断言/条件不能随改根消失。

## 5. 详细设计

### 5.1 小样边界

Theme Lab 新增于 auto-ui/examples/theme-lab/（新路径）；先将 auto-lang/examples/ui 全部受管内容归 auto-ui/examples/ui，再在目标树依次改造002-counter/src/front/app.at与005-login/src/front/app.at及必要pac/依赖声明。两官方候选包为 Auto（默认，适配旧公共源）与 Solarized（风格示意，非完整标准），均有 light/dark。共享库/导出归 auto-ui，实际落点由 T-01 冻结；三个消费者不复制主题表。根 system 策略与 app_defaults 展示扩展。

标准 Button 显示 base/hover/pressed/disabled/focus；Input、Panel 仅支持小样必需的颜色/尺寸/状态表面，不宣称所有原生部件接入。两个 CapsulePanel 实例、其中一个内部 ChildPanel 覆盖构成隔离样本。输入值 draft-42 与计数器 2 是换肤保留 oracle。一次实际 token 更新必须影响仍引用的样式；直接字面 radius 必须不受同槽变值影响。

已有两 demo 的新增设计接入遵循 C-13/14：共享官方包，独立时取官方默认、嵌入时不锁死宿主选择；不各自增加复制的配色表或设置菜单。002 保留增/减/重置，005 保留输入、密码字段与必填验证，换肤后数据与校验行为保持。既有语义 class 可经明确兼容映射使用，但默认控件设计/主题选择必须实际接入，不能仅保留旧字符串而称新形式完成。业务相关固定值不盲改。

拟议 .Design 仅在设计准备阶段运行，初次首帧前、主题/模式/读取的输入变更后从基准重算。只允许局部设计写入，不调 I/O/发事件/写业务状态或 theme/mode 选择器；合法顺序/非法副作用给诊断。不能把 .Init 的首帧后改色作为通过。

### 5.2 T-01 有界接缝核验

执行门满足后，在同一实施树检查 ordinary use 数据导出、TokenTree/Theme 数据类型接入、点引用保留、extend、widget 字段及 .Design 到两消费者的完整通路。建立小规模可运行正反输入及报告，不扩成语言能力盘点。

如 UI 域端口足够，列出实际文件/符号/测试命令并进入 T-02。如必须改变通用 Core 对象/模块语义、公共 ABI，或新写全套解释器，记录精确阻点，修订合同再决定依赖实施；不得缩掉公共语法 AC 或用 demo 手工接线补绿。语法允许的普通对象不能全局改为懒求值。任何尚未存在的测试/样例明确标新。

### 5.3 示例来源迁移

T-01 固定源文件清单/哈希/许可、注册/文档/工具/测试等实际读者与基线功能。T-02 将全部37目录的受管内容映射至本仓 examples/ui，保留源文件原始哈希和移入身份，忽略缓存不盲搬。跨仓历史以来源提交+文件manifest+两仓提交映射保留，不重写历史。T-03 对pac相对依赖、stylekit、blueprints、媒体fixture、测试/registry/Man入口等做路径中性修复；每条非字节等价变更注明原因，阶段A闭合后才在阶段B改主题。

语言旧位置可留README迁移指引，不留运行.at副本、不做junction/symlink或主检出隐藏fallback。旧源不可用条件下验证真实新来源；路径变化不能删掉AP-16/原golden/测试义务。必要的有限语言fixture须固定原内容与来源，不能整棵复制。语言普通构建/测试不得因示例迁移新增对UI兄弟检出的必需依赖：UI领域读者归UI，语言自身语义回归用有来源的有限固化输入，确需跨仓语料的ACE/组合验证显式获得固定输入并保持owner/条件；按906的L1/L2独立边界复核，而不是把AUTO_LANG_ROOT兜底改成必需AUTO_UI_ROOT。新增/变动文件按实际启动快照对账，不能拿本次381当未来不可变分母。

完整当前契约见[搬迁约定](../design/autoui/examples-ownership-and-relocation.md)。若新位置不支持既有入口，修复依赖/定位合同；不以留第二套活体旧源作为兼容。

### 5.4 与905、906交接及实施入口

| 对方/状态快照 | 007接收 | 007回交与边界/关闭方式 |
|---|---|---|
| [905](../../../auto-lang/docs/plans/905-generated-cargo-artifact-governance-research.md)：execution_done5/7，待独立review/merge；不是治理实施完成 | 最终研究来源SHA、有效flags/profile/target和资源账；已观察basename成员键碰撞、mtime判regen、flags分裂及Web逐app target，属于治理输入 | 示例搬迁可能改变project路径/成员身份/相对根；T-01/03向其后继提供旧→新身份、两真实app生成/发行基线，不自创第二Cargo缓存/全局共享workspace或改debug。共享`rust_workspace.rs`/`source_owner.rs`/`rust_ui.rs`/锁修改短窗单写，研究失效轴按新来源重测，007不代认证磁盘收益 |
| [906](../../../auto-lang/docs/plans/906-standalone-language-web-boundary.md)：executing2/8；C1已获批准、尚非正式交付 | 当前C1拟合同包Web→Lang单源迁移、facade/API-gen Web面退出、HTTP e2e改显式独立组合通道、SDK source_owner/投影/锁更新；这是已选方案，结果须以最终交付及canonical验收 | 默认接其最终组合后冻结007 SDK/Web/语言fixture身份；不保留旧Web路径兜底、不恢复facade旧入口、不复制合同或Web实现。007只改示例根及其必要消费者接线，不重做906依赖/公共API迁移。906若变化则重核受影响V-09/10和生成fresh/locked证据，不能照901的28项快照硬修回旧布局 |

905提出的`013-todo` Rust生成失败是具名未闭问题，尚不能由本轮只读核查判根因或归罪014。T-01在实际使用该路径前记录原基线：若既有失败且不妨碍来源守恒，迁移保持同型诊断、具名交owner/后继，不能记运行PASS；若007造成回归或阻塞本计划AC-13/14必须在本计划闭合，确需扩大生产修复先修订合同。生成成员碰撞/内容失效修复原则上属Cargo治理后继，不偷偷并入主题实现；若影响本计划必需oracle，记阻点并按既定交接解决，不删验收绕过。

启动门以§4.2为唯一调度规则。当前下一责任是路线规划/用户确认近期Cargo有限出口与007实施顺序，906按其合同独立交付；本轮不让T-01实施探针提前开跑。门满足并获007实施授权后，work/T-01先冻结当前组合、37目录差额/读者/许可/测试ID、共享写面和资源窗口；T-02/03阶段A、T-04..06阶段B顺序不变。未来若批准无交集部分先行，必须具体写明允许任务、冻结版本、哪类证据需906后重做，不能笼统称“并行即可”。ACE/AG全线、所有历史债务不加入全局前置。

### 5.5 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/theme-tree-pilot.md（本仓新；受限实验合同） | 旧 ThemeDecl 31 键兼容不变；增实验树/模式/extend/引用边界 | 树状数据扩展不等于所有 app 已主题化 | AC-01..05 |
| SD-02 | add | 同上 | widget 局部设计生命周期、继承/隔离与失败发布，明确仅小样支持面 | 不把原型冒充全平台产品合同 | AC-03..07 |
| SD-03 | add | docs/specs/ui/theme-app-migration.md（本仓新） | 官方主题单源、应用默认跟随/独立回退、首批两 demo 迁移与全量后继清单 | 明确用户新增真实接入目标 | AC-08/11/12 |
| SD-04 | add | docs/specs/ui/examples-ownership.md（本仓新）；语言现行UI overview只同步来源指针 | 教学示例归本仓、源映射/依赖定位/测试义务保留，历史来源可回溯 | 框架与示例单源、消除反向资产耦合 | AC-13/14 |

正式全量推广按已确认路线另立批次；merge 只沉淀实际验收的首批与实验边界，不能冒称其余 app/AutoOS 完成，原旧合同不静默退役。

## 6. 测试设计

实施者先确认真实模块/CLI 来源、依赖固定与命令。现有入口为 auto run（Vue）与 auto run -r vm；必须调用 worktree 对应 CLI/工具，不能误用全局旧二进制。

| Case ID | AC | 输入/入口与 oracle | owner_stage / 边界 |
|---|---|---|---|
| V-01 | AC-01/02 | 新 theme_tree_pilot 集成测试：真实 Auto use/数据导出→树合成→两种投影；后层 8dp、未改字段保留、数组替换、两个 modes 均保留；只读基主题值不变 | work；新目标由 T-04 建立，不以 Rust 手工造最终树代替源码路径 |
| V-02 | AC-02/03 | 同一 token 改值后，引用圆角同步改变，组件直接字面值不变；若复制成旧值应红。app namespace 扩展合法，公共 Color 填 Length 报错 | work；真实引用与消费字段 |
| V-03 | AC-04/05 | 同源码两包×两模式×两后端逐项；父/兄弟不串色、子继承/子覆盖；dark→light 往返无遗留 | work；属性/颜色/尺寸与屏幕交叉核，不只截图或数据自洽 |
| V-04 | AC-04/06 | 两个 CapsulePanel 状态独立；换肤后输入 draft-42、计数2、组件实例身份保持；同事件 theme+mode 不显示中间组合 | work；真实 runtime 与 Vue 更新，DOM/实例身份+状态+可见结果 |
| V-05 | AC-05/06 | system 受控事件 light→dark；实际 OS 事件单独烟测；单模式显式 dark 拒绝，system 回默认且 resolved_mode 正确；引用环/未知路径/错型拒绝后仍读到旧有效颜色 | work；首次失败与已有有效主题两状态，诊断须源路径可定位 |
| V-06 | AC-03/07 | 首帧即局部圆角；主题/设置变更后 .Design 重算；业务副作用不允许；legacy 未选入控制样例生成/运行结果保持 | work；不以首帧后截图冒充首帧正确 |
| V-07 | AC-08/12 | 完成后记录包/应用实际源码、重复字段、后端接线、失败诊断与操作步骤；给约定修改、逐 app 剩余项与下一批入口 | work；实际结果，不取消全量目标或预填“好用” |
| V-08 | AC-11 | 002 增2/减1/重置，005 空表提交→必填错误→输入→错误清除；两 app 同包两族两模式双端，换肤保持各自状态、尺寸/文字/焦点反馈可用 | work；两 app 的真实源码到实际画面与操作；不以 Theme Lab 代替 |
| V-09 | AC-13 | 阶段A迁移manifest逐文件对账，全部源目录身份/文件哈希匹配；再核路径中性patch的独立差异单 | work；阶段B前冻结，缓存/未跟踪例外明确，不用改主题后的hash倒推搬迁无损 |
| V-10 | AC-14 | 旧活体源不存在时运行已分类真实读者：UI registry/renderer、stylekit导入、041/015依赖、media fixture、语言/AP-16映射、Man及现行脚本/文档入口；重定位到第二绝对根再读 | work；缺新来源须诊断不能悄悄回退，旧测试断言/分母保持；适用测试选择由T-01冻结，缺项不能靠代表例替代 |

定向命令候选（新目标由 T-04 创建后才可执行）：auto-ui worktree 中 cargo check -p auto-ui-compiler -p auto-ui-runtime；cargo test -p auto-ui-compiler --test theme_tree_pilot。UI 生成改动的定向用例在 T-01 明确，不在 work/review 运行整档 tu/t；沿014测试映射，`ui_gen::vue`生成族使用bare cargo test单进程（`-- --test-threads=1`），不放进nextest并发族。

语言侧仅有Rust/fixture真实触面时，在`scripts/test-groups.toml`核既有`auto-gen-core`、`auto-lang-core`、`parser`、`vm`、`corpus`、`http`等相关完整固定组及必要消费者并集，T-01逐读者列实际命令/测试ID/选择理由；不是默认全部运行。ACE语料发生实改则显式选择对应ACE职责组，不删除AP-16，也不因只读引用而构建ACE worker。UI/Man沿各自owner入口；对V-09/10明确枚举实际选择数/执行数/原条件排除数，零选择不算通过，已有红/平台未执行保持具名。迁移后的语言独立消费及UI外部fresh/locked/第二根读取分别验证，不能仅在五仓齐全组树内自证。

双端交互使用 autoui-verifier 技能与已有 auto-lang/.agents/skills/autoui-verifier/scripts/test_vm_mcp.py、test_vue_playwright.mjs，复用 runner 而非另写通用框架；小样动作/断言可新增。原生输入如占用用户桌面只在有实际授权的验证窗口执行。所有计划材料仅放`D:/autostack/plan-artifacts/plan-007/auto-ui/`（跨仓各以repo身份隔离，创建时登记owner/用途/在途状态）；不写attachments（含md），不在仓内散落截图/日志/副本。正常target在唯一实施树，外置target仅统一scope并登记可删。merge将必要命令/退出码/关键数/源码节选与哈希折入本Plan，再删本计划整个scope和树内产物；仅具名未解决问题持有owner/理由/关闭条件，未知归属不盲删。

实现/自查由实施会话负责；final review 用独立上下文核 AC/真实通路，不创建或联系其他 agent/chat，除非后续明确授权。merge 冻结门 pending 不能写成 work 已通过。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 公共骨架、Feature/app 命名空间及普通 use 通过真实源码入口；不要求改框架即可增加一个已声明 app 命名空间 token | work |
| AC-02 | extend 规则与 V-01 正反结果成立，输入不变、模式分支保留、token 引用在扩展后仍联动 | work |
| AC-03 | 局部 token 与组件直接值的不同作用、同路径后写、模式重建、引用错型/循环诊断成立 | work |
| AC-04 | 同一 Auto 小样两包/两模式在 Vue/VM Iced 可交互，双实例/父子隔离正确；不能以生成通过冒充运行通过 | work |
| AC-05 | system 的受控与实际系统事件证据分别齐全，resolved_mode/不支持模式处理明确；环境缺失保持 pending | work |
| AC-06 | 换肤保持输入/计数/实例身份，合法组合原子发布；非法更新保旧有效子树并能再次合法换肤 | work |
| AC-07 | .Design 首帧/重算/副作用边界有证据；legacy 未选入控制不改变现有外观/主题优先级 | work |
| AC-08 | 试用评估明确源码负担、后端接线与剩余缺口；给约定修改和下一迁移批次，不把已确认全量目标降回可选推广，也不冒称全量完成 | work |
| AC-09 | 独立复审核实际实现、所有 AC 与 proposed Spec delta，无未解释遗漏/绕行；最终 verdict 绑定提交 | review |
| AC-10 | 受影响冻结门通过、实际合入、canonical实验Spec沉淀、归档/guard/材料清理完整；legacy specs.json只读，索引由应用派生；未执行保持 pending | merge |
| AC-11 | 官方候选主题单源可被三个真实消费者导入，002/005 依次完成新设计接入和 V-08 双端；独立默认与宿主主题选择不被应用固定值阻断 | work |
| AC-12 | examples/ui 逐目录状态与后继批次可追踪，包含两个不同031目录及公共库/验证夹具分类；AutoOS 来源独立，不称教学修改已传播 | work |
| AC-13 | 全部受管示例/共享库/夹具按来源身份与阶段A哈希归位auto-ui/examples/ui；旧源不留活体副本或链接，迁移前后独立于主题改写验收 | work |
| AC-14 | 已冻结所有现行消费者使用新来源，stylekit/blueprints/外部资源可定位；旧根不可用、不同绝对根仍有效，语言/ACE/其他测试义务未因搬迁消失 | work |

## 8. 执行步骤

预定唯一实施组 D:/autostack/.wt/ui-007/{auto-ui,auto-lang,auto-man}，各仓一棵树；分支 codex/plan-007-theme-examples。仅实际依赖需要时补auto-down及auto-web同组固定来源sibling（如需源码修改各独立分支、同一repo仅一树）；不创建链接。auto-ui主检出持唯一Plan与当前主题设计；实施与所有构建/测试在组内，语言主检出仅有路线/指引。跨仓改动包括源示例删除/README指向、消费者路径与Man入口；不在主检出顺手搬代码。

| ID | 任务/文件或符号/闭合条件 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 核§4.2/5.4启动门及014/900/901/018交付、905有限治理出口/906正式组合；冻结§4.4实时差额/TrackCtx候选处置、生成身份/配置和共享写面；有界探针核主题use/引用/局部上下文到两端通路；同时冻结examples源文件/依赖/许可、live/test/history读者清单与适用命令。已有ui_ast.rs、grammar/widget.rs、design_tokens/*、app_registry.rs、UI/语言/Man媒体/语料读者；超出UI域先修订不绕行 | 启动门；全部work AC前置 | work |
| T-02 | 按T-01实时差额在实施组将全部auto-lang/examples/ui受管内容归auto-ui/examples/ui，逐文件manifest/哈希核对，保留两031及stylekit/验证夹具；源仓清除活体源仅留README指向，记录非忽略新文件与不搬缓存；不改主题语义 | T-01；AC-13 | work |
| T-03 | 依冻结读者逐项修复新根、stylekit/blueprints/pac相对依赖、UI registry/renderer/夹具、语言回归/AP-16、Man/现行网站脚本入口；路径中性改动单列，旧源不可用与第二绝对根验证V-10，阶段A关闭，生成工程fresh/locked/重定位须接受906来源并保持905治理约定，不改profile凑绿 | T-02；AC-14 | work |
| T-04 | 在UI包组建立共同树/引用/extend/模式表示与Schema校验，新增auto-ui-compiler/tests/theme_tree_pilot.rs；接真实模块与官方数据包，保持registry.rs/decl.rs旧消费者兼容，V-01/02 | T-01/03；AC-01..03 | work |
| T-05 | 接widget选择/.Design/事务与Vue/VM Iced；T-01冻结实际路径，候选ui_gen/vue/{mod,style_helpers,tags}.rs、ui/aura_view_builder/{mod,eval,props}.rs、ui/iced/renderer/{style_widgets,projection}.rs、ui/vm_bridge.rs、style_iced_adapter.rs、system_theme.rs；新增Theme Lab并记录完整V-03..06/实际系统事件 | T-04；AC-03..07 | work |
| T-06 | 在本仓examples/ui依次改002-counter、005-login（app/pac/依赖），共用官方包、宿主继承/独立默认；保持原业务并双端验V-08，其他尚未改主题app保持原合同 | T-05；AC-11 | work |
| T-07 | 按首批评估V-07，修订约定不降AC；更新本仓全目录状态和AutoOS来源，给剩余主题迁移批次入口，回交905后继的生成身份映射/013-todo基线和TrackCtx是否接续的有据处置，不再决定做不做迁移 | T-06；AC-08/12 | work |
| T-08 | 闭合work矩阵与来源证据：阶段A迁移/读者、新主题/官方包/两实际app、legacy控制与错误保旧；准备SD-01..04并核本仓唯一合同/语言指引，无重复活体源 | T-07；AC-01..08/11..14 | work |
| T-09 | 独立final review核真正来源到运行消费、遗漏/绕行/Spec delta，修复回同组唯一树并使相关证据失效；不能用迁移后重新基线掩盖业务变化 | T-08；AC-09 | review |
| T-10 | 按冻结改动跑UI t及必要tu/ti、语言/Man实际触面门；实际Rust测试代码有改不得按“仅示例”漏门，但不因只读跨仓就跑无关全档。各仓合入、canonical Spec沉淀（legacy specs.json只读）、归档，核全部实际输出目录/持有登记并删统一scope，所有树先wt-guard clean后清理 | T-09；AC-10 | merge |


新建/修订设计契约及复审后受影响门按本仓规范执行；全量 AG/ACE/旧语言重型回归不因试用自动触发。若 T-01 核出实改 Core，先重规划触面与语言对应门，不能擅自扩大或漏跑。

## 9. 复审记录

以下r1/r2是迁入前同一未执行草案的修订历史，不是语言仓仍存在的活动Plan身份。当前唯一合同是本仓007 r3。

- 2026-10-29：stage=new，plan_revision=1；仅完成设计/源码读取与草案一致性核对。outcome=blocked（执行合同确认及已定结构收口门待满足；不是实现失败）；next=确认合同、按既有顺序完成前置，再进入 work/T-01。AC-01..10/T-01..07 全部未执行。
- 2026-10-29 r2：stage=new；直接用户范围补充使 AC-08/T-05 修改，新增 AC-11/12、V-08、SD-03、T-06（原 review/merge 顺延为 T-07/08）；total_steps=8，均未执行。保留 r1 为历史，不用其“小样之后再决定推广”缩减当前目标。outcome=blocked 仅指既定结构启动门/具体执行合同未就绪，非迁移需求未授权；next=按既定前置核门后 work/T-01。
- 2026-10-29 r3：stage=new；用户直接授权转交/重新编号/源号释放及必要示例归位。新归属auto-ui/007，源草案r2 SHA-256=`bd9474f7d268b308d8c8c01c786c62af235eadde7479e106d54bbcdb1fce6b82`；当前主题设计一并归本仓，语言只保留指引。新增C-15/SD-04/AC-13/14/V-09/10，任务重排为T-01..10，均未执行；outcome=blocked仅指既定结构收口门，next=前置实际关闭后work/T-01。当前转交/计数器更新属于已授权文档动作，不等待新确认。
- 无 build/test/GUI 证据，无独立 final review，无合入或交付声明。全套新语义尚待验证。

- 2026-11-01 / stage:new / PLAN-007 r4 / outcome:blocked（仅实施启动条件，非规划校验失败） / 接受014/900/901/018已归档交付，交付SHA→当前HEAD祖先关系只读核验通过；静态核0–10章节/10T/14AC/10V/4SD唯一、全部本地Markdown链接有效、无尾空格；只读复核381文件/3,263,776字节/37目录全部仍在语言仓、UI目标受管文件0；更新真实拆分模块、现行读者锚点及905/906状态/交接。10任务/14AC/10V/4SD保留，主要修订T-01/02/03/05/07/10及AC-10的canonical口径，未降低其余AC。未建树/探针/构建/测试/GUI/源代码或Spec修改；未联系其他会话、未提交。next：按§4.2确认近期有限治理实际出口、接收906最终组合与007实施授权，之后work/T-01。结构门已关闭，不再把014待交付作为阻塞；905研究交付与Cargo治理关闭仍分列。

## 9.1 work 阶段执行记录（2026-11-02，r4 全链）

实施组：`D:/autostack/.wt/ui-007/{auto-ui,auto-lang,auto-man,auto-web,auto-down}`（§8 三仓 + 依赖仓按需补树）；分支 `codex/plan-007-theme-examples`；基座 = ui `01f99fd` / lang `5923d4dee` / man `67add3c` / web `3de5524` / down `7c4d5cb`。提交：ui 11 个（`1478193`→`32483835`）、lang 2（`4061af61c`、`125317608`）、man 1（`47d8f7b`）。

**T-01 启动门与冻结（✅）**：§4.2①用户 `/goal 完成计划007` 授权全链（记录于§4.2）；②905 archived 7/7 + 910（实际Cargo治理）archived 9/9 已落地（lang f665ce113/ui 01f99fd/web 3de5524/man 67add3c）；③906 archived 8/8。examples 实时差额复核=冻结值（381文件/3,263,775字节/37目录，ui 侧 0）；SHA-256 逐文件清单冻结于 plan-artifacts（清单哈希 `979b55ad9532f1f7…`）。**有界探针**：Core 对象字面量+点访问可 parse；顶层 `pub let` 不可 parse（Token Let 拒绝）；`extend` 不存在 → **判定：UI 域端口足够**（theme 声明/extend/design 块全部落 UI 域，零 Core 语义改动）。读者分类：auto-ui 36 文件/auto-lang 6/auto-man 3（live/test/comment 三类，报告存 plan-artifacts）。

**T-02 来源搬迁（✅ AC-13）**：381/381 文件字节等价落 `auto-ui/examples/ui`（逐文件 SHA-256 对账 0 失配，盘面 381=清单）；lang 侧 `git rm` + 定位 README（`4061af61c`）。缓存/未跟踪 0 搬运；两 031、stylekit、remote-frame-e2e、032 全保留。

**T-03 读者回接（✅ AC-14/V-09/V-10）**：约 40 文件重定位（`../../../auto-lang/examples/ui` → 本仓根；helper 双根改造 stage3/coverage；lang_repo_root 保持语言仓语义 + 新增 `examples_ui_root`）；041 的 bps 依赖改指语言仓兄弟（`../../../../auto-lang/packages/blueprints`）；auto-media-core 改用仓内 `tests/fixtures/` 固定输入（8 文件 + PROVENANCE，非活体镜像）；a2ts tsc 候选/三个脚本（remote_frame_e2e、style_palette_guard、web_delivery scope）随迁。**V-09**：manifest 381/381 + 路径中性 patch 独立 diff（T-03 提交）。**V-10**：registry 28/28、coverage 21/21、media 36/36、plan370 18/18、a2ts 组 PASS、vue_capabilities 015 测试修复转绿、AUTO_UI_EXAMPLES_ROOT 第二绝对根 PASS、Man 编译 0 错。具名既有红（非本计划引入，主检出+旧语料同红）：`ui_snapshots` insta 漂移 ×3。

**T-04 树状主题表示（✅ AC-01..03/V-01/V-02）**：dialect `pub theme <Name> = <树>`（`UiStmt::ThemeDataDecl`）；compiler `design_tokens/theme_tree.rs`（lower/extend 合成 C-04/schema 校验 C-02..03/mode 重建 C-06/槽引用 C-05/可定位诊断 C-11/双投影 css_var_pairs+resolve_rgb/use+dep 收集器）。单测 5/5 + `theme_tree_pilot.rs` V-01/V-02 全绿（真实 use→合成→两投影→槽引用联动→字面独立→负例定位）。**关键设计记录**：extend base 用字符串名（Core 对 `undefined variable` 硬约束，零 Core 改动）；模式差异按族自足（Solarized light 显式重声明——继承基族 modes 会盖 base patch，主题数据边界实证）。

**T-05 双端接线（✅ AC-03..07/V-03..06）**：dialect widget `design` 块（theme/mode 数据绑定 + `parse_design_block_inner`）；VM 轨 `ActiveSpec::Tree` 槽 + `set_theme_tree`（THEME_EPOCH 失效回路复用）、build_dynamic_component 经真实 use/dep 链注册主题树 + design 条目、builder 每帧 `eval_app_design`（事务：未知主题/未声明模式 → 诊断+保旧）、元素 design 对象 → RoundedT/B(Px)+BackgroundColor 结构类（实例隔离）。Vue 轨：`collect_tree_themes_js` → `window.__AUTO_TREE_THEMES__` → THEME_PALETTES；`theme_name` watch → applyTheme 全量 CSS 变量；design 对象 → `:style` 槽引用 `var(--card)`/`var(--radius-control)`、字面量具体值；纯前端 app 跳过 Rust 后端（`resolve_back_api` 门——旧主检出残留兜底掩盖的既有缺口，示例归仓后全数误败，本计划具名修复）。**V-03/V-04（VM 实机 12/12 + 像素证据）**：Auto light `#ffffff/#f4f4f5`、Solarized light `#fdf6e3/#eee8d5`、dark `#002b36/#073642` 逐值精确；计数 2→4、draft-42→draft-99 跨族跨模式保持；往返无遗留。**V-03/V-04（Vue 实机）**：`--background`/`--card` 计算样式逐值同上；count/draft 跨切换保持。**V-05**：受控 system = `mode: "system"` → `system_prefers_dark()` 探测（eval 路径）；单模式拒绝/未知主题保旧 = mode_available + eval 事务（机制经代码核验 + mode_available 负例单测钉死【T-09 R1 补】；keep-old 交互面由 V-03..06 驱动器证据覆盖）；**实际 OS 事件联动的独立烟测 pending**（需变更用户 OS 主题设置，未获授权；不影响其他 AC）。**V-06**：首帧截图即含主题化面板与槽引用圆角（lab_initial）；.Design 语义由 design 块每帧求值承接（主题/模式变更即重算）；业务副作用零（无事件/IO 写）；legacy 未选入控制 = 003-converter vm dispatch + registry/coverage 全族原值通过。**`.Design` 生命周期钩子形态记录**：C-08 的"首帧前计算局部差异"由 design 块的每帧求值承接（求值先于元素转换=整帧一致），独立 `.Design` 方法名未引入——约定允许试用修正（V-07 反馈项）。

**T-06 首批两 app（✅ AC-11/V-08）**：002-counter/005-login 接入 `dep themes` + design 绑定 + 切换 UI，业务原样。**V-08 VM 实机 11/11**（002 增2/减1/重置+换肤保持；005 空表提交→必填错误→输入→清除→换肤保持→未清错误仍在）；**V-08 Vue 实机全绿**（`#002b36` 换肤像素证据 + 业务状态跨切保持）。

**T-07 评估与交接（✅ AC-08/AC-12）**：`examples/ui/README.md` 注册头（归位/官方包/试点/首批/后继批次口径）；theme-lab README 操作步骤。**V-07 试用评估**（源码负担/接线/缺口/约定反馈）：①应用最小接入 = `dep themes` + `use` + `design` 块 + 保留状态名，无框架改动；②包作者只维护数据（themes.at）；③加 app token 零框架改动（命名空间对象）；④槽引用联动/局部隔离/事务保旧可预测（V-01..06 证据）；⑤后端特有接线仅两处投影（VM 结构类、Vue var() 引用）——既定投影架构成立；⑥诊断可定位（路径入错）。**约定修改反馈**（不降 AC）：extend base 字符串名、`.Design` 钩子由 design 块每帧求值承接、style 字段试点保持类串（槽引用走 design 对象）、纯数据 dep 模块的 vue 预编译扫描告警（非致命，具名边界）。**逐 app 剩余**=其余 33 编号目录（后继批次入口=theme-app-migration.md §3）；AutoOS 承接边界不变。**905 后继交接**：生成身份映射=同相对结构直映射（旧 `<lang>/examples/rust-workspace/<name>-<id8>-{front,back}` → 各 app `rust-workspace/`（auto-ui 树内）；新 id8 按新路径派生，成员名含新 id8——910 身份规则照常适用）；`013-todo` Rust 生成失败维持**具名未闭**（源文件字节等价迁移、身份中性，未运行验证不记 PASS，交 905 后继 owner）。**TrackCtx 处置**：不接续——主题系统零新 owner/穿线需求（经 thread-local 槽 + VmBridge read_state 既有通道），未做的理由与证据如上，7文件29签名候选保持登记给 UI-007 后继候选，不升级为本合同项。

**T-08 闭合（✅）**：SD-01..04 canonical 草案入库（`docs/specs/ui/{theme-tree-pilot,theme-app-migration,examples-ownership}.md` + 索引，`539903ac`）；实施树干净（ui/lang/man 三树 status clean）。定向测试族：compiler lib **1132/1132**、vue 族单进程 **327/327**、theme_tree 5/5、pilot 5/5、theme_lab_build 1/1、registry 28/28、coverage 21/21、media 36/36、plan339 7/7、plan370 18/18、plan492 21/21、plan806 18/18、plan814 8/8、dialect 全绿。**runtime 日常档全量**（nextest，`9a087cd3`+后继修复后）：**2338 run = 2318 passed / 20 failed / 182 skipped**；20 个失败**逐一在未改动 master（01f99fd）对照复红或环境归因**：vue_capabilities ×11 + ui_snapshots ×3 + gallery_vue_golden + schema_drift_fence + native_dock_e2e ×3（真窗，AGENTS 环境归因条）= master 同红既存债（具名，非本计划引入）；plan802 剪贴板往返 = 环境态依赖（master 重放绿）。**收敛修复**（对照揪出、本计划闭合）：plan492 charts-gallery 顶层臂（plan370 保留语言兄弟候选）、plan806 变异体负控（收集器对非主题模块解析错误静默让位生产链）、plan814/plan488 两个 capability fixture 的 stylekit dep 改指 auto-ui 兄弟（auto-lang `72b1b50cb`、`210baf528`）。runtime 档的迁移前后对照：本计划触及面全绿，未触及面的红均为既存。

**证据归档**：`D:/autostack/plan-artifacts/plan-007/{auto-ui,auto-lang,auto-man}/`（manifest、读者分类报告、驱动脚本、screenshots 实机像素证据）+ 本节摘要。遗留：second-root-abs-check 临时副本（已登记，merge 删）。

- 2026-11-02 / stage:work / PLAN-007 r4 / outcome:pass（execution_done）/ worktree:`D:/autostack/.wt/ui-007/{auto-ui,auto-lang,auto-man}` / branch:`codex/plan-007-theme-examples` / base:`01f99fd|5923d4dee|67add3c` / code:ui `9a087cd3`+lang `210baf528`+man `47d8f7b` / task_ids:T-01..T-08 全闭（T-09 review、T-10 merge 待）/ evidence:本节 9.1 + plan-artifacts/plan-007/（V-01..V-10 实机证据、manifest、分类报告、runtime 档全量对照账）/ blockers:实际 OS 事件烟测 pending（需用户授权改 OS 主题设置；受控 system 证据已齐）；runtime 档 20 个失败均 master 同红既存或环境归因（逐一对照，见 9.1 T-08）/ next_responsible_role:final review（T-09，review_scope=final）

- 2026-11-02 / stage:review / review_scope:final / reviewer_identity:独立 general-purpose 子上下文（未参与本计划实现；limitations=与实现会话同宿主，凭据为工件与自跑命令，非角色名）/ plan_id:PLAN-007 / plan_revision:r4 / outcome:pass（reviewed）/ worktree:`.wt/ui-007/{auto-ui,auto-lang,auto-man}` / branch:`codex/plan-007-theme-examples` / reviewed_commit:ui `1739d4a5` + lang `210baf528` + man `47d8f7b` / base_commit:`01f99fd` / `5923d4dee` / `67add3c` / dependency_revisions:web `3de5524`、down `7c4d5cb` / spec_inputs:docs/specs/ui/{theme-tree-pilot,theme-app-migration,examples-ownership}.md（worktree 提交 `539903ac`+R1 修订） / acceptance_results:AC-01..08/11..14 pass（评审员独立核验：全量 manifest 对账 375 同+6 处为后继合法提交、2 新增声明文件；读者 0 活体残留；theme_tree 6/6、pilot 5/5、theme_lab_build 1/1 自跑；交互驱动器进程/截图可信度核验；规范三文逐句对码）；AC-05 实际 OS 事件腿按合同 pending；AC-09=本评审；AC-10 merge 待 / pending_closeout_gates:实际 OS 主题事件烟测（需用户授权，AC-05 边界）；landing checks 归 T-10 / findings:F-R1-01（.am 状态件入库→untrack，b240c56e）、F-R1-02（负例单测补 + §9.1 措辞校正）、F-R1-03（SD-01 逐键回退措辞与实现对齐）、F-R1-04（SD-04 源守恒豁免精确化）——四项全部闭合并经独立复核（theme_tree 6/6 自跑；diff 范围=仅测试+规范+untrack，无生产语义变更）；NOTES:ui_snapshots ×3 于 master 150bab3e 同红复核成立，其余既存红为抽样核验 / evidence:评审报告全文存会话工件；本行即持久摘要 / next:merge（T-10，按 auto-plan-merge）

## 10. 待澄清事项

1. 归属/全量源码归位、首批002/005与官方主题方向均已确认；014/900/901/018已交付。本轮明确暂不实施，007实施授权尚未给出。默认门见§4.2，不再复问已确认方向；若调整近期里程碑或与906先后，需记录具体任务/版本/复验决定。
2. 905尚待研究独立复审/收尾，其有限M1建议及后继实施未自动获批/交付；用户确认的有限出口与真实治理结果须由对应合同证明，不以研究归档、单候选no-go、899/901/018的收益替代。无需等待整个Cargo历史问题全清。
3. 906已选C1且具体兼容/owner变化已获批准，但尚未交付。007在相关阶段接受其正式来源/SDK投影/HTTP测试通道/锁清单，不预写C1已实现；T-01按实际落地差额重冻。
4. T-01有界核验真实use/TokenTree/主题与双端接缝、逐文件来源及全部现行读者；公共Core语义/ABI或支持范围变化先修订，不新写全套解释器。TrackCtx按需要判断的候选不升级为本合同必需项；013-todo已有问题与007回归分列，未验不PASS。
5. 全量后继主题接入及AutoOS真实产品/宿主消费另立合同，007不削减方向也不冒称交付；不新建ui-examples仓。必需系统事件/输入/GPU环境缺失保持pending，资源窗口开工时明确。
