---
plan_id: PLAN-026
status: drafting
feature_name: 阅读引擎嵌入与定位回传双端可行性探针
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 0
total_steps: 10
base_branch: master
base_commit: 98cbcd2e79ebc61b0683886f24f0747b20d72617
depends_on: [auto-reader/READER-006]
supersedes_spec_components: []
new_spec_components: [docs/specs/runtime/document-surface-boundary.md]
touched_goals: []
---

# auto-ui/PLAN-026：阅读引擎嵌入与定位回传探针

## 0. 变更摘要

承接 AutoReader 的 [coverage v3 路线](../../../auto-reader/docs/roadmap-v0.6.md)及已归档 READER-006，验证成熟 EPUB 引擎是否能进入 AutoUI 的 Vue 与原生 VM 表面，并让真实选区/定位经过应用 .at handler、持久化、冷重启恢复同一原文。主导仓为 **auto-ui**，AutoReader 为受管资产与纯 .at 消费验证方，Readest 固定源码仅为研究/引擎来源。

本轮用户明确“建立正式计划、核实已有能力、明确分工与双端验收，暂不实施”。当前仅 new，drafting/r1、0/10；不创建实施 worktree、不安装依赖、不写探针源码、不构建/启动应用、不实测、不改 canonical Spec。下述新文件、命令和预期均为未来任务，不是已完成证据。

## 1. 目标

1. 核实现有 View/事件/选择/资源/宿主路径；避免把已有选择复制、富文本或远程帧重复造为新框架能力。
2. 固定有限的 EPUB 样本与引擎，验证真实宿主内的渲染、选区、输入、定位回传和恢复；输出双端逐项证据及真实失败边界。
3. 明确安全资源边界、会话/文档身份、偏移单位、生命周期和旧数据保护，给 Locator v2 后续设计提供实际契约。
4. 交付 go 或 no_go_current_profile 的有据结论、最小缺口 owner 与下一步；探针完成不等于正式阅读器已支持完整 EPUB。

非目标：移植整个 Readest/React/Tauri、发布通用浏览器组件、完整 EPUB/PDF/漫画/TTS、实现 Locator v2 或批量迁移、替换 Reader reading_page、修复所有历史 VM HTTP/工具问题、系统 URI/OS 装配、云服务、Linux/移动端交付。不得降低“覆盖 Readest”的长期目标来通过本探针。

## 2. 架构方案

### 主导仓与单份合同

| 仓 | 责任与允许的未来改面 | 交付边界 |
|---|---|---|
| auto-ui（主导） | 编译器/控件实验接缝、Vue scaffold、原生 iced 宿主内受控文档表面、事件回传、资源与销毁；复用已有公共机制 | 持唯一 PLAN-026 正文/进度/复审/归档；宿主接入与跨端协议责任不下放给应用 |
| auto-reader（消费） | 新增隔离 tests/probe/engine_surface/ 消费工程、语料与驱动；调用本 app 真实导入/受管资产契约，.at 接收与保存探针定位 | 不另分配 READER-007 复制合同；README/路线保留指针与验收分工；不改业务页、书库/注释当前 schema |
| Readest 检出 | 固定 tag 与 foliate-js fork 的只读来源、许可与依赖身份 | 不在 D:/autostack/readest 开发；实际试验只在自有工作目录副本/受控资产中运行 |
| auto-web / auto-lang / auto-man / auto-down | 只读固定依赖和既有服务/CLI/文本能力 | 不是本计划源码写入仓；若发现其独立缺口，记录最小复现并另立实际 owner 前置，不偷偷扩大本计划 |
| AutoOS | 本次不改 shell/manifest/gitlink | 系统装配不是探针前置；跨 app 注册也不纳入 |

UI 主改面是原生宿主与双端桥，故本仓取号。UI 主检出当前 master；拟唯一实施树为 D:/autostack/.wt/ui-026/auto-ui，分支 codex/plan-026-dev，**尚未创建**。work 时固定实际 base 与来源，在同树顺序执行/内检/独立 review；不使用正在被 PLAN-025 使用的树。跨仓依赖按实际项目解析序与真实 sibling 检出准备，不建 junction/symlink；移除任何树先运行 D:/autostack/wt-guard.sh。

Reader 按用户 app 纪律在 D:/autostack/auto-reader 独立 clone main 开发/顺序复审，一 app 一写入者，不另开共享 Git worktree、不动 OS 子模块；进入 review 暂停两仓实现写入。可执行文件、CLI/tool、两仓 source、固定依赖与引擎形成同一 cohort 清单，不能用陈旧已安装 tool 冒充新 UI 源。

### 有界接入决策

T-00 核实当前能力并比较最多两种宿主接法，冻结一条主探针路径和必要对照。已知 iced 中不存在已核实的通用 WebView 叶，不能直接承诺某个库可用；未来允许在本仓增加最小实验宿主/生成/桥接代码和必要固定依赖，范围只为验证本计划的样本/协议，不能在生成 Vue/Rust 上手改交付。现有原生选择、Rich/文档编辑器及 RemoteFrame 可作为对照/复用单元，但不能替代真实 DOM 引擎回传。

若接入需要独立框架/平台重构，停止该候选并给 no_go_current_profile/最小前置建议，不把“探针”扩成完整浏览器项目。纯环境缺失只能记 environment_blocked，不足以证明技术不可行。

## 3. 技术栈

已观察 AutoUI compiler/dialect/runtime/tools，原生 iced，Vue 生成，DynamicMessage::Typed 与 VmBridge；Reader 为 .at + VM HTTP，既有原生/Playwright 验收。WebView 库、原生接缝 API 和运行时引擎装载方式尚未选定，T-00 产决策后才写实现。

主样本引擎 = Readest v0.12.16 对应 foliate-js fork（3d7941954a98b136316a795e8200d60c9643b141）；必要时上游版本作对照，记录准确 pin 和差异。引擎 JS/HTML adapter 属框架接缝与固定第三方资产，应用业务和消费 handler 保持 .at。不复制 Readest 账号/服务密钥、用户 profile 或现有书库。

本探针追踪矩阵 RDT-C02-001/002（分页/滚动）、C02-009（字号重排）、C02-025（脚注）及 RDT-C03-001/020（真实选区/格式定位）的基础接缝；不宣称实现这些整行的全部选项。具体格式/图像为固定profile验证，不将图片显示算成C02-017全功能查看器，也不将探针持久化算成C03-002完整高亮产品面；后续产品计划再更新矩阵覆盖状态。

## 4. 需求分析与背景调查

### 授权与工作位

2026-11-01 用户认可 v3 顺序，并指定先为本探针建立正式计划、核实已有框架能力、明确主导仓/跨仓分工/双端标准，**暂不实施**。本次允许两仓计划/导航文档；不因合同已经写好自动进入 work。用户未分配模型/代理、时间、费用或 token 预算，不生成虚构授权、不启动子代理。未来实施的代码范围受第 2 节约束，超出主改面要另行处理。

静态核查基线：auto-ui=98cbcd2e79ebc61b0683886f24f0747b20d72617（master），auto-reader=bf5d744086cdfc390b79ae67afca89d44d703457（main），Readest=3cc39ad7a558e6612e1a8135058fbd6f3c90b3e3（tag v0.12.16）。UI 007/025 活动合同与本需求不同，未发现同需求活动 Plan。UI 主检出原有未跟踪 docs/design/autoui/，本次保留；不将其当已交付能力或纳入提交。

### 已有能力核查（new 阶段源码/Spec 观察，非运行结果）

| 能力 | 真实入口/观察 | 能复用什么；尚不能证明什么 |
|---|---|---|
| 原生文本选择/复制 | runtime/src/ui/view.rs View::Text.selectable；ui/iced/selectable_text.rs SelectableText，源码写明选区为 widget 本地状态 | 真实拖选/双击/剪贴板与 Unicode 命中对照已存在；View::Text 没有文档 range/CFI 回调字段，不能把复制成功当选择已进 .at |
| 富文本 | runtime/src/ui/view.rs View::Rich + RichSpanView；AutoDown 原生文档链 | 不能笼统声称框架没有任何富文本；现有段落渲染不等于完整出版物 CSS/DOM/CFI 或 Reader 已接入行内标注 |
| 远程帧 | View::RemoteFrame、ui/iced/remote_frame_registry.rs，Vue ui_gen/vue/shadcn_attrs.rs 的 remote_frame 臂 | iced 消费 DisplayList v2 + pointer/char，Vue 仅透传 div 由宿主接管；不是通用浏览器文档表面，也无已核实选区/CFI/无障碍透传 |
| embed | ui_gen/widget/registry.rs Embed；aura_view_builder/convert_element.rs embed_block | 是块引用面板/降级文本，不是 iframe/WebView；fullstack_embed 测试是组件/数据组合，不能据名字判浏览器已嵌入 |
| JS 与宿主消息示例 | tools/src/vscode.rs VSCode Webview 的 onDidReceiveMessage/vscode.postMessage | VSCode 扩展壳示例，不是原生 VM iced 文档 widget；不能套用为已有原生链路 |
| .at 事件派发 | aura_view_builder/events.rs event_to_message_with → DynamicMessage::Typed；现有 $0 等事件期载荷处理 | 可复用类型/handler 路由，但没有查到专属浏览器文档 selectionchange → .at 的完整生产路径；需要验证字符串/多字节/记录传输 |
| 定位/恢复 | View::ScrollAnchorMark；Reader real-library 与 reading-annotations Specs、annotations.at | 现有章/段/逐字锚、occurrence 与持久化可作对照；不是 CFI，不能由扁平段号伪造转换 |
| 输入、原生证据 | native-vm-app-validation.md；Reader test_native_standalone_ui.py 具名 GAP | 要核验真实窗口与回传；旧 Http.post/多字节与工具按压问题不得直接标已修，也不能靠 HTTP 预写绕过 |
| Web/资源所有权 | docs/specs/modules/web-integration.md | HTTP runtime/服务归 auto-web；UI 可复用，不重写一套后端，也不将服务 bind/编译失败判成 WebView 故障 |

搜索范围包含本仓 crates/ 下 .rs/.toml、root Cargo.toml 与相关 Spec；webview/wry/tao 等未检出可确认的通用 native 文档控件，检出的 VSCode/注释/Win32 PostMessage 不计为该能力。**这是当前静态证据的边界，不是所有依赖永远不支持的断言**；T-00 必须按执行版本复核。

| 固定源 | SHA256 |
|---|---|
| docs/specs/runtime/native-vm-app-validation.md | f6ea4068a011a806df86dd784213cdd81e5bba3aa98bc35927aad0049b31ff0c |
| crates/auto-ui-runtime/src/ui/view.rs | 2e147090bde6fc27a4eaa2c3dd40de1cbbbcb189c616827c94563d4b3092e174 |
| crates/auto-ui-runtime/src/ui/iced/selectable_text.rs | 15202aefcee2caf7017917fea4d6493e484cf0f673f7366b41ce634c49d08b55 |
| crates/auto-ui-runtime/src/ui/iced/remote_frame_registry.rs | 093acb0ca91410bf1cf8b0b2d503ad78c7591feab5a0a682443c92ccc3fa21bd |
| auto-reader/docs/roadmap-v0.6.md | 68a0622c5445d583bb79b225a2927ab3643579ddd3d70dc924552b9f4f970b11 |
| auto-reader/docs/specs/reader/reading-annotations.md | c2ace0e4b9de7486d6bae4921662484e22bbe03cc16abbac5c3d182aefbb766b |

权威源还包括 docs/specs/README.md、runtime/native-vm-app-validation.md、build/ui-test-selection.md、modules/web-integration.md，以及语言仓 docs/specs/auto-ui/project.md 的实现 owner 边界；root README 的旧目录描述不替代这些源。006 原始 plan-artifacts 已删除，研究摘要与归档复审保留；新试验需重新生成语料/原始记录，不能声称重用了已删截图。

## 5. 详细设计

### 5.1 实验输入、协议与所有权

主链：Reader 在隔离 AUTO_READER_DATA 中用真实导入入口建立受管 EPUB/书籍身份 → 框架受控资源装载 → 宿主内固定引擎产生真实 DOM range/CFI → 桥验证会话/文档身份 → .at handler → 独立探针记录持久化 → 新进程同文档恢复/核验原文。Reader 现有 v1 读写保持；**CFI 只写探针自有 sidecar，不改变 annotation schema/import_version，不声称已迁移**。旧 annotation_id 与书库样本作不变性对照。

T-00 在写桥前冻结实验消息：schema_version、session_id、document_id/book_id+source_hash、engine_revision、resource/href、真实定位器/范围、原句/上下文、request/sequence 与结果分类。Web 的 UTF-16 offset、原生字节偏移、.at 字符索引不可直接混用；若保留偏移，必须带单位并经真实转换/原文核验，CFI 由对应引擎生成且能在同版本文档解析。重复句锚不能只靠 quote 或百分比。

文档代码无任意本机文件/命令能力；资源仅可读取本次书的受管内容，禁止路径逃逸、跨书读取、出版物活动脚本/外部自动加载。可信引擎脚本与不可信书内脚本分开。桥只接本次文档表面，不允许其他来源/窗口通过通用 postMessage 伪造回传；双端用同一逻辑协议和分类，底层 transport 可不同。

本探针固定两本自制无 DRM EPUB（每本≤1MiB，含中文/emoji、三处重复句、跨行选择、图片、样式和脚注/跨章）、每次最多两个独立消费实例、单条事件≤32KiB UTF-8。超 profile 的输入明确拒绝且不写；这些是探针 profile，不修改现有 Reader 30MiB 导入等产品支持。T-00 冻结解压/资源/窗口/等待与清理 deadline、诊断及两侧合法/非法对照，在依赖实现前记入合同；不得承诺无限资源。未充分评估的图片/字体等不扩大为任意出版物承诺。

### 5.2 双端表面标准

| 路径 | 有效承载 | 不足以通过的替代物 |
|---|---|---|
| Vue + VM HTTP | 正常生成/装配的 AutoUI 消费页面内文档表面；真实鼠标/键盘操作，事件经 .at，保存与冷恢复 | 单独手写 HTML 演示、修改 gen/ 产物、独立 Web 页面、JS 直接写应用文件 |
| 原生 VM + VM HTTP（split） | 本次进程树内可识别的 iced 窗口与控件，文档表面附着其布局/焦点/遮挡/尺寸/关闭生命周期；真实 .at 消费 handler 与保存 | 外开浏览器/Readest 窗口、仅 remote-frame 截图、只见端口、空树、把 Vue 页截图贴进原生窗口 |

两端需各自覆盖焦点从原生 chrome 到正文/输入框再返回、中文 IME 组合输入不误翻页、窗口 resize/字号/滚动保持内容锚、选区与浮层无遮挡/裁剪、键盘可达和系统无障碍树可发现阅读内容/操作。DOM ARIA 存在不等于系统无障碍接缝通过；若平台工具不可用，记 GAP，不计 go。多进程内嵌子表面可用，但 PID/父窗口/布局关系与清理须证明，不要求单进程伪装。

### 5.3 新增产物（未来路径）

- 本仓 experiments/reader-engine-probe/：有限引擎 adapter、资源 manifest 与必要实验 host 胶水（新路径，T-00 确认是否可复用现有设施；不引入第二套生产 app）。
- Reader tests/probe/engine_surface/：纯 .at 消费工程、固定语料生成器与独立 JSON/文本 oracle（新路径）；真实后端受管资产与 probe sidecar 都在隔离数据目录，不往用户库写。
- Reader tests/probe/verify_engine_surface.py：拟永久入口 `python tests/probe/verify_engine_surface.py --profile vue|vm --auto-bin <固定auto> --out-dir <隔离输出>`，T-01/T-04 建立，不声称现在存在；自管端口/进程/数据，传输、断连、恢复和负控均记录。
- 本仓 docs/research/reader-engine-probe.md（新）：固定 cohort、能力/协议/预算、每例输入/预期/实际/命令/退出码、双端结果、go/no_go 与实际 owner、替代成本、迁移契约建议及局限。临时材料统一 plan-artifacts/plan-026/auto-ui/ 与 plan-026/auto-reader/，按本仓管理规则不新增 attachments。

### 5.4 结论与关闭条件

每例记录 PASS / reproduced_fail / GAP / not_run_dependency，不把 skip 计 PASS。**go** 要两端的全链和全部必需接缝/拒绝/清理门均通过，source+tool+engine 与实际执行一致。**no_go_current_profile** 要有可复现的技术失败边界、同条件合法对照/最小复现、责任 owner 与可审查的最小前置/替代方案；下游未能运行的项目保持未运行，不能标成功。

计划是可行性研究：有证据的 no_go 可以交付研究结论，但不交付产品能力。失败可按依赖停止后续无意义实现，T-04～07 仍需完成其失败映射、用户旧数据保护核对、报告与规范边界，不因 no_go 预勾任务。只有“机器缺依赖/启动环境坏了”而无技术结论时保持 executing/blocked evidence，不能归档成 no_go。聚合驱动功能失败须非零；不能为让测试绿把失败吞掉，研究完成由独立 review 评估证据，而非退出码包装。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/document-surface-boundary.md（新） | 从未冻结通用 DOM/CFI 表面 → 如实记录实验 profile 的实际承载、身份/资源/选区/生命周期边界与可用性结论；no_go 时只记缺口/限制，不承诺接口已可用 | 防止实验演示被解释为正式浏览器/全格式能力 | AC-01～08 |

不改 native-vm-app-validation 的原标准；不修改 Reader 已交付数据 Spec 或旧 ledger/index。上述是提出的 delta，本轮不写 canonical Spec，review 据实际实现/无实现结论冻结精确内容，不以行数配额补规范。

### 下一执行入口

本次仅静态核查与起草。未来获启动指令后，T-00 首先确认两仓 HEAD/WIP、UI 025 的共享改面及依赖身份，在指定唯一树准备能力/契约；T-01 才生成语料与驱动。已闭合基础为 Reader 001～006；新宿主、CFI桥、两端真机与迁移可行性均未证，不能从本文推导成功。

## 6. 测试设计

以下用例为未来 mandatory cases（owner_stage=work，建立入口=T-01/T-04，上述 verify_engine_surface.py）；Vue 与 vm 分别执行同一逻辑集，保存独立结果。HTTP/磁盘只作独立读回 oracle，不预写冒充用户保存。真实系统窗口/事件按 NVM-01～06 证明；无法使用工具时具名记录，不通过 JS 直接创建选区替代真实拖选。

| Case ID | AC/任务 | 输入/实际路径 | 区分错误实现的断言；覆盖边界 |
|---|---|---|---|
| V-01 | AC-02/03；T-02/03 | 受管 A.epub→两端宿主→显示图/CSS/脚注→回到正文 | 真图可解码、目标原文正确且表面确属宿主；不是文本扁平化或外窗。只覆盖固定 profile |
| V-02 | AC-03/04；T-04 | 真实拖选三处重复句的第2处（跨行，含中文/emoji）→handler→sidecar | 原句逐字相同、instance/resource/range 对应第2处，解析 locator 回到同一处；UTF-16/字节/字符不能错位；.at状态和磁盘独立读回 |
| V-03 | AC-04；T-04 | V-02后换字号/窗口宽度/滚动→保存→退出所有本次进程→冷重开 | 重新解析真实 locator 后目标可见、原文与指定出现位置一致；隔壁重复句作为反差，禁止仅比百分比/截图 |
| V-04 | AC-03；T-03/05 | chrome↔正文↔搜索输入框，中文 IME 组词、鼠标选区/键盘导航 | 组合输入不误翻页/触发快捷键，焦点可返回，浮层不被宿主盖住；两端分别记录 |
| V-05 | AC-03；T-05 | resize/遮挡/隐藏与重新展示表面；系统无障碍工具观察/键盘访问 | 内容布局/命中跟随宿主；系统树内容/可操作性可发现，DOM ARIA或空树不能代替该门 |
| V-06 | AC-05；T-05 | 在同一合法session发错book/source_hash/旧sequence/其他表面origin的事件；与合法事件配对 | 错身份/迟到/伪造事件被拒且零落盘；合法事件仍接收；同类型正确载荷只改身份避免靠类型错证明安全 |
| V-07 | AC-05；T-05 | path escape/跨书资源、书内脚本或远程URL、损坏/超profile EPUB与32KiB事件两侧 | 具体拒绝分类/错误可见、不执行脚本/远程读、不写部分定位、不改既有书/注释；合法本地图片/引擎脚本对照仍能用 |
| V-08 | AC-06；T-05 | A/B 两实例不同书与会话；关闭A后再送旧事件，B继续操作；重开A | 无跨实例串文/串锚；关闭断资源/窗口/子进程/监听，新A不接旧事件，B不被收割 |
| V-09 | AC-04/05；T-05 | 真实改变/损坏locator目标，或使原文核验失败；保留合法同书对照 | 明确 failed/needs_relocate 且保留原记录，不静默跳第一处、不删除/假升级；恢复后合法路径仍成功 |
| V-10 | AC-01/06；T-00/05 | 固定CLI/tool与probe；坏tool路径/占用端口/旧target断连 | 具名失败与非零/有限退出，不能误连他人实例；清理仅自有PID/端口/目录，合法启动/重开为正对照 |

构建/回归按真实改面冻结一次所需门：UI 档位来自 .cargo/config.toml，编译器发射涉及 ui_gen 时跑 `cargo tu`（单进程），runtime/dialect 影响选本仓 `cargo t` 与新定向目标；工具进程影响再选 `cargo ti`。具体新测试名由 T-00/T-01 注册并在执行前列出/核非零选择，保留 native/GPU/串行/ignore 条件，不以日常 t 替代真窗口。无语言源修改不跑 auto-lang cargo/docs_gen，不叠加多轮全仓回归。

Reader 原有 `tests/basic/verify_standalone.py --profile vue|vm` 在两仓工具cohort冻结后各跑一次守旧门；旧库/旧注释不变性以隔离样本逐字/ID核对。仅文档起草本轮不运行它们。新增驱动应注入一次真实错误locator或错身份使原判据失败非零，修复后转绿，证明oracle有区分力；结构检查本身不能证明语义。

执行者内部检查不等于最终 review。独立最终复审在相同两仓执行位顺序复演 V-02/03、V-06/08 与所有声称 go 的系统接缝；no_go 则复现首个失败边界与对照，核对未运行项和后续建议。用户未指定模型，不自动创建代理或其他chat。

## 7. 验收标准

| ID | owner_stage | 观察/验证标准 |
|---|---|---|
| AC-01 | work | 固定源码/工具/引擎/协议/profile与隔离配置，能力核对和至多两候选接法有源码与实际启动证据；原始事实与假设分列，T-00决策先于接缝实现 |
| AC-02 | work | 实际尝试两端宿主装载；V-01/V-10 证实表面与实例身份。失败需定位首边界，不用外窗/截图或服务编译失败冒充嵌入结论 |
| AC-03 | work | V-02/04/05 按两端给焦点/IME/真实选区、几何/滚动与系统无障碍结果；仅全部通过才满足该域go，无证项显式GAP |
| AC-04 | work | V-02/03/09贯穿真实事件→.at→sidecar→新进程真实locator恢复，逐字/出现位置正确；旧库ID与文件保护，无伪造CFI或虚构迁移 |
| AC-05 | work | V-06/07/09按协议和资源边界给合法/非法对照，错误拒绝且零写入/越界读取；状态、单位与身份核验由真正消费端执行 |
| AC-06 | work | V-08/10的双实例/迟到/关闭/重开、进程/资源/端口归属与清理有实证；旧Reader双端守旧门结果与cohort真实，历史GAP保持可见 |
| AC-07 | work | docs/research/reader-engine-probe.md具双端逐例PASS/fail/GAP/not_run与证据；go需AC-02～06全门通过，no_go需首失败边界复现、对照和可解释owner/最小前置。环境阻塞不能冒充no_go；失败下游不得计PASS |
| AC-08 | work | 真实结果回写实验边界Spec与Reader设计/路线/导航，给引擎/Locator v2契约及后续工作分工；不声称产品支持或降低目标，不改旧研究/归档合同 |
| AC-09 | review | 独立review绑定Plan r1与实际两仓source/tool/engine cohort，复验规定路径与结论；明确研究交付是否pass与路线go是否成立，两者不能混淆 |
| AC-10 | merge | review pass后两仓精确提交/链接可追溯，合入主干并核canonical边界、归档与自有材料/树清理回执；不将别人的WIP/实例纳入，无OS装配义务 |

AC-02～06 的成功语义是**路线go门**，不是允许跳过的产品验收；若真实技术失败导致 no_go，必须逐项给失败/不可运行因果、完成 AC-07/08 的研究交付后由独立review判研究完成，不能把失败门改写为通过。若没有可证技术结论，work不应进入execution_done。

## 8. 执行步骤

全部未执行。未来执行时按本表推进，进度和判据不得因no_go预先打勾。

| ID | owner_stage | 依赖 | 路径/动作与产物 | 验证/关闭条件 | AC |
|---|---|---|---|---|---|
| T-00 | work | 后续明确启动 | UI指定树+Reader独立clone核身份/WIP/007与025共享窗口；实查已有接口、固定工具/引擎，最多两接法比较，冻结消息/预算/deadline/测试选择于报告与合同 | 两仓git status/rev-parse、CLI/tool hash、源码入口与拟测试非零清单；不覆盖现有未跟踪设计，不建链接；依赖未知项先闭合 | AC-01 |
| T-01 | work | T-00 | 新增两本合成EPUB与独立原文/出现序oracle、跨书/畸形/超额/错锚反例；建立实验driver骨架和数据profile | 语料来源/指纹与预期第2处原句可独立重算；V-01～10输入/入口可复演，不把拟用例标已执行 | AC-01/05 |
| T-02 | work | T-01 | UI experiments/reader-engine-probe与最小必要compiler/tools接缝；Vue装载/资源/事件派发正向试验 | Vue真实表面与引擎身份、V-01/02运行；不改生成物，实际失败有最小对照和责任阶段 | AC-02/03 |
| T-03 | work | T-02接口、T-00原生接法 | UI runtime/iced最小原生宿主实验与事件接缝；焦点/IME/几何/系统无障碍试验 | 原生窗口/父子表面/布局关系与V-01/04/05实际结果；不能拿Vue通过代替原生 | AC-02/03 |
| T-04 | work | T-02/03可消费边界 | Reader tests/probe/engine_surface纯.at消费、独立sidecar与verify_engine_surface.py；闭合选区→保存→重开；或记录阻断此链的已复现边界 | 两profile未来入口均可给实际结果与因果；V-02/03/09逐字/身份/出现序oracle；不写当前数据schema，失败不包装成功 | AC-04 |
| T-05 | work | T-04结果 | 两仓运行V-01～10受影响路径、负控、生命周期与旧Reader门；记录不具备前置的未运行项 | `python tests/probe/verify_engine_surface.py --profile vue|vm ...`应由T-01/04建立；正常结果与负例退出码可区分、原生/资源/旧数据保护实证；本仓冻结cargo选择仅跑一次所需面 | AC-03～06 |
| T-06 | work | T-05 | 完成本仓研究报告，给go/no_go_current_profile、失败owner/最小前置/替代成本与范围；无真实结论则继续执行/登记阻塞 | 对照第5.4/7节闭合证据，所有未运行/账号/系统树缺证可见，不能只交截图；新定位契约不冒充Locator v2落地 | AC-07 |
| T-07 | work | T-06 | 沉淀SD-01实际实验边界、Reader设计/路线与两仓导航；内部核对并交独立review | git diff --check；所有本地链接存在，Spec与实际能力/不可用结论一致；仅本合同范围，0用户数据丢失 | AC-08 |
| T-08 | review | T-07 | 同树独立上下文审两仓cohort、复演第6节；给研究pass/needs_fix等与路线结论 | 绑定提交与依赖，复验真实链路/负控或首失败，AC-01～08逐项，内部绿不替代review | AC-09 |
| T-09 | merge | T-08 pass | 精确交付两仓变更/导航，核canonical边界；主合同归档archived/，材料耐久摘要入合同/报告后清自有scope；guard通过后才remove树 | 真实主干/文档源/恢复信息、无他人WIP/链接/进程损伤，计数据实际闭合；不触OS pin或全局ledger | AC-10 |

## 9. 复审记录

### 2026-11-01：new 阶段交接

- stage: new；auto-ui/PLAN-026 r1；outcome: pass（合同可审查）；next: work，**仅待用户后续明确启动，本轮不实施**。
- 新增 T-00～09、AC-01～10、V-01～10、SD-01；current_step=0/10。已有能力核查为静态源码/规范观察，不是探针PASS/route go或独立review。
- 计划号在本仓主检出独占锁下扫描活动与 archive/archived 分配；Reader仅加指针，不消耗007，不在语言仓/OS仓重复登记正文。
- 本次设计输入为 auto-reader coverage v3 与 READER-006；不修改已归档研究、当前Spec、产品实现或原有UI未跟踪设计。

## 10. 待澄清事项

无阻塞本次正式起草的用户信息，已有“暂不实施”指令是本轮终点。下列由未来T-00及探针负责，不要求用户事先选技术答案：

1. 原生宿主可接入方式、引擎资产装载、桥字段/单位/预算与系统无障碍工具？T-00先冻结profile，T-02/03实证。
2. UI 025/007对共享编译/宿主文件的写入时间窗、工具构建与实际source cohort？T-00按owner协调，一app一写入者，不切走他人工作。
3. 若现有VM传输缺口阻断真实.at消费，记录首边界/owner；只允许UI自身最小接缝修复，其他仓缺口另计划，不以API预写降低门。
4. no_go不自动改变产品目标；后续根据成本讨论宿主前置/替代引擎，再建立Locator v2和完整EPUB实施计划。当前不承诺CFI自动迁移、PDF/TTS随引擎一并获得。
