---
plan_id: PLAN-029
status: drafting
feature_name: Reader原生输入链与文档表面必要接缝收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
current_step: 0
total_steps: 12
base_branch: master
base_commit: e344d2268f6bf40d1ed4f24ea957ce3fbd88313c
depends_on: [auto-ui/PLAN-026, auto-reader/READER-006]
supersedes_spec_components: [docs/specs/runtime/document-surface-boundary.md]
new_spec_components: [docs/specs/runtime/document-surface-native-input.md]
touched_goals: []
---

# auto-ui/PLAN-029：Reader原生输入链与必要接缝收口

## 0. 变更摘要

承接已归档 [026](archived/026-reader-engine-embedding-and-locator-probe.md) 的未闭合项及 [Reader coverage v4](../../../auto-reader/docs/roadmap-v0.6.md)。026已证Vue指定样本全链30/30、独立iced宿主的WebView2嵌入/引擎/IPC；尚未证明实际AutoUI原生VM消费者的真实选区→.at→保存→冷恢复，也未完成焦点/IME、系统无障碍、双实例和专属失败控制。本计划以**全部必需门真实通过**为完成条件，不继承“带GAP的有条件go即可归档”的退出方式。

主导仓auto-ui，消费仓auto-reader，一份合同；不重开026、不另取Reader号。本次仅new、drafting/r1、0/12；不创建实施树、不构建/启动窗口、不安装依赖、不运行探针或修改源码/现实现Spec。下面的新文件、接口与用例均为未来工作。

## 1. 目标

1. 复现并定位原生输入断点，区分驱动、焦点、宿主、文档引擎、桥和消费handler，不预定“只是自动化环境”。
2. 使真实AutoUI原生VM消费工程在同一运行会话内完成精确选区、.at回传/核验、隔离sidecar持久化、退出重开与原文/出现位次恢复。
3. 收口焦点/中文IME、几何与遮挡、系统无障碍、双实例隔离、迟到拒绝、关闭/端口/进程清理和专属失败控制；保持Vue既有链与旧数据语义。
4. 用受控反例证明验收入口会对失败/GAP/缺少必需用例返回非零；经独立上下文最终复审后，为下一阶段Locator v2/完整EPUB产品化提供可靠前置。

非目标：完整Reader阅读页迁移、全出版物CSS/PDF/漫画/TTS、批量v1→CFI迁移、改业务书库/注释schema、账号同步、系统URI/OS装配、Linux/移动平台交付、修复所有VM字符串/HTTP历史问题。原生输入方案失败可触发needs_replan/明确阻塞，不能删门或降目标宣布本计划完成。

## 2. 架构方案

### 主导仓与跨仓分工

| 仓 | 工作范围 | 责任边界 |
|---|---|---|
| auto-ui（主导） | 复用026接法，按需改compiler/dialect/runtime/tools的最小实验文档表面/事件/宿主接线、输入/几何/生命周期/无障碍；实验Rust仅作诊断对照 | 必须接进真实AutoUI原生VM运行与.at组件，不能只扩展独立native_host.exe；持唯一Plan与验收/复审记录 |
| auto-reader（消费） | tests/probe/engine_surface/**、必要tests/basic/spec/native验收修复；受管资产、.at前端消费者/后端handler、sidecar、独立oracle与两端驱动 | 不接管宿主平台实现，不改src/业务数据格式；负责精确原文/出现位次、存储与守旧门事实；不分配READER-007复制合同 |
| auto-lang/auto-web/auto-man等 | 只读固定依赖、工具源与失败定位 | 不预先授权他仓源码修改。R-05等若必须他仓修复，由实际owner另立最小前置并绑定版本；依赖未交付则本计划受影响AC未通过，不能靠登记债务豁免 |
| Readest | foliate-js fork固定3d7941954a98b136316a795e8200d60c9643b141，只读来源 | 不改D:/autostack/readest，不把研究baseline换成未经核实的main；新胶水和试验副本在自有目录 |
| AutoOS | 无源码/manifest/gitlink改面 | 装配/协议注册另计划，非本阶段验收前置 |

拟唯一UI实施树D:/autostack/.wt/ui-029/auto-ui，分支codex/plan-029-dev，**本轮不创建**；计划簿记在主检出master。实际work基线重核当前HEAD，不切换/复用ui-025/027/028等他人树。依赖解析按env覆盖→真实兄弟检出→主检出，Cargo需兄弟源码时准备真实固定依赖检出，不建junction/symlink，拆树先fresh wt-guard。

Reader按用户app纪律用D:/autostack/auto-reader独立clone main、单写者，不在AutoOS子模块或第二app工作树实现。执行、内检与最终复审顺序使用各自同一树，review期间两仓暂停实现写入。主检出已有UI设计/028规划及共享号源WIP保持，不纳入本计划交付。

### 接入原则

真实原生启动入口为消费工程目录的 `auto run -r vm --server=vm`；Vue对照为 `auto run --server=vm`。两者固定真实auto/auto-ui-tool、源码/锁/引擎与端口cohort；启用本计划实验能力的准确开关/语法由T-00冻结。原生文档表面须处在本次AutoUI组件树及宿主布局/焦点/隐藏/关闭生命周期中，.at前端能观察选区状态，并经既有.at后端校验/保存；后端和UI的真实所属必须可查。

允许在本仓增加最小受控实验leaf/adapter及必要生成/运行接线。T-00比较复用既有设施与最小新增两种方案，先冻结接口、能力门与回归范围，再实现；不得把remote-frame的DisplayList语义偷换成任意DOM，或以probe://特殊字符串绕过公开声明。生成Rust/Vue不手改；业务保持.at。若需要大规模宿主重构，回规划明确依赖，不能将整个浏览器项目塞入本合同。

## 3. 技术栈

已知：AutoUI原生VM/iced、Vue生成、DynamicMessage/VmBridge、026 wry/WebView2实验、Reader .at+VM HTTP、Python/Playwright/CDP/Win32诊断、现有MCP证据面与系统无障碍工具。选区不能用JS Range赋值、API直接保存或外开浏览器替代；CDP可做真实鼠标/键盘通道对照，但不能因此宣称OS焦点/IME已通过。

具体宿主接口、事件载荷类型、系统无障碍取证工具和发布/实验开关由T-00以当前source固定。本计划不在起草阶段选未经核实的新库或承诺所有WebView平台可用。

## 4. 需求分析与背景调查

### 授权与基线

2026-11-02用户要求“继续建立后续计划来完成它”，所指为先前明确的原生输入/必要接缝与验收缺口。本轮建立正式合同并同步导航，不自动进入work。未指定时间、费用、token预算、实现/复审模型或子代理，不虚构这些授权。正常实施时按本合同范围推进；改变验收标准/仓域或长期覆盖目标须另行明确。

静态起草基线：UI e344d2268f6bf40d1ed4f24ea957ce3fbd88313c，Reader b6183e0770d1c2a19ab6d415bcd0f5e06d8062e9。活动007/028与本需求不同；026已归档，不覆盖其历史。号源起草前029，独占扫描活动及archive/archived最大号28后分配；Reader号源007不变。

### 源码/Spec核查事实（尚未运行）

- `experiments/reader-engine-probe/src/bin/native_host.rs`是**独立iced实验程序**，不是AutoUI原生VM组件；用window::run/wry子表面，创建时一次set_bounds，Msg只有WindowOpened。存在IPC/状态回报不证明运行期resize/焦点/IME/实际.at前端handler。
- `driver_vm.py`启动该独立exe，默认路径还指向已清理的.wt/ui-026；使用SendInput/固定比例坐标，未充分核查返回值/Win32绑定、DPI/坐标/真实内容目标，候选驱动问题需T-00复现，不能由静态阅读直接裁定根因。当前resize主要以records>=2判断，不能证明重排后命中同一指定原文。
- `app/src/front/pages/probe.at`目前是静态remote-frame占位，Vue由driver挂载adapter、fetch到.at后端；尚未查到原生VM文档事件→.at前端model→可见状态的闭环。下一步必须把实际路径补齐，不能继续以原生宿主装载代替。
- `verify_engine_surface.py`有PASS/FAIL/GAP/not_run记录，但末尾只检查FAIL/not_run；`CASES`仅注册surface，未独立核验全部必需用例集合。因此GAP或遗漏case有返回0的风险，T-01须建立非零选择/缺项/重复/状态守卫。
- `corpus_profile.py`已有：两本≤1MiB EPUB、事件≤32KiB UTF-8、最多2实例，装载15s/章节10s/回传5s/冷恢复30s/清理10s、driver总预算600s；026子宿主曾用120s另限。T-00须对齐真实入口与旧profile，不能靠悄悄放宽deadline解决失败。
- 已交付实验Spec仍标原生输入、焦点/IME/系统无障碍为未证；`native-vm-app-validation.md`要求真实native frontend、窗口树/输入/存储、负例对照与清理重开。本计划不降低该标准。
- 026报告旧Reader门Vue27/28、VM25/27（R-05直连路由404、R-06vm坏图负控rc=0）；报告归属是待复现线索，src未改不是排除工具/集成回归的充分证据。

| 起草源 | SHA256 |
|---|---|
| UI experiments/reader-engine-probe/src/bin/native_host.rs | 3a6dbb67a85acbf46036bf7148f69d145ac6cae4290669e0eb0811afeb1a84be |
| UI docs/specs/runtime/document-surface-boundary.md | 444a970969667d9207863f946727a893f85b3ee83ec65e031638e91b99da6740 |
| Reader tests/probe/engine_surface/driver_vm.py | 24f842197d0d143eed26d2c70467ad30e013509deba13afaf35df7be3aebe0e3 |
| Reader tests/probe/engine_surface/verify_engine_surface.py | aaa285dadfe1505adc2aa094c8e1d3f8b3ee7253b0e9f8f4027edef3a223ed4f |
| Reader tests/probe/engine_surface/app/src/front/pages/probe.at | 47d29ac59e79df454c9215446b9ba2ec92dad3de4bb5f920ee80b8b9586705c4 |
| Reader docs/roadmap-v0.6.md | 79212e19b6a2b912d872693b23bc8f2a35f3cf4f7a7996d3b9468ad7392f8b4f |

证据入口：[026研究报告](../research/reader-engine-probe.md)、[实验Spec](../specs/runtime/document-surface-boundary.md)、[原生应用验收Spec](../specs/runtime/native-vm-app-validation.md)与Reader现实现Specs。026原始材料已清理；未来按prepare_engine_assets.py/gen_corpus.py重建资产并固定hash，历史截图路径不计新证据。

## 5. 详细设计

### 5.1 先定责，再接入实际消费者

T-00用现有native_host作最小对照：核查ctypes原型/结构大小/调用返回/虚拟屏与DPI坐标、目标窗口/内容rect、焦点链、引擎selection事件。比较系统输入、真实桌面手工/可用UI工具与CDP真实输入，分别记录通道结果；JS只读观察可用，不能直接代选。最多比较两种接线实现并冻结最小方案，不无限试库。

随后打通UI实际运行链：.at声明实验文档表面→compiler/运行树→native宿主内受控文档→真实输入→引擎range/CFI→UI桥校验→.at前端handler/model与可见选择状态→既有.at后端原文校验→probe sidecar。窗口/组件/会话/文档版本身份全链关联；单独native_host继续只作诊断对照，不作为AC-02通过入口。

### 5.2 数据与支持profile保持

沿用固定fork@3d794195、合成alpha/beta、CFI+quote+出现序及32KiB/1MiB/两实例预算。数据只进隔离AUTO_READER_DATA/PROBE_DATA_DIR，保留书籍与annotation_id/v1文件，probe sidecar不升级产品schema。两实例指两份文档消费会话，T-00列出其所有自有服务/子进程/端口，端口必须可配置并在bind前预检，不能只在同一目录串行运行就算双实例。

既有15/10/5/30/10s和每profile600s为继承的验收约束；T-00记录测量起止点，功能负控应在同一deadline内返回。发现预算本身不适用时按语义修订说明原因，不偷偷放宽。每次运行验证out-dir属于本次workspace内新建的自有scope，清理不跟随reparse point，拒绝用户库/他人目录，不沿旧driver的无条件rmtree把任意--out-dir清空。

CFI由真实文档树产生且恢复后解析到同一指定出现位置；不能只检查字符串以epubcfi开头、记录数递增、页面含相同quote或任意位置可见。HTTP/磁盘只读回核对；合法选择产生记录后再验证错session/错doc/hash/schema/stale_seq/错锚/超额拒绝零落盘，合法对照仍成功。文档脚本/远程资源/跨书路径和桥来源仍受026资源不变量约束，不因接新host而放开。

### 5.3 必需接缝与证据

原生表面由组件布局分配区域，不能继续盖满整个客户区遮住AutoUI chrome。焦点在chrome→正文→输入框→正文可往返；IME真实组合事件与最终文本/导航状态可观察。resize/缩放/重排/滚动、遮挡/隐藏/展示后重新命中指定原文与选区，不能只看窗口尺寸。

系统无障碍须在宿主窗口可定位文档内容与操作，读屏/系统UIA或等价工具真实取证；DOM ARIA/截图不够。系统工具不可用记GAP并阻塞对应AC，不能写“超出环境所以通过”。若接缝修复需要新增公共边界，范围限定实验消费profile，其他UI行为不变，受影响原生/生成测试须核回归。

双实例A/B并存不同文档/会话与独立记录；关闭A后迟到事件拒绝，B继续真实选择保存，A新进程重开不接旧事件；资源、webview/窗口、会话订阅、owned子进程和端口在deadline内释放。不能靠driver全局kill/强制删目录补宿主正常关闭的缺陷；异常退出另测，分别报告。

### 5.4 严格验收入口与旧门

扩展现有 `python tests/probe/engine_surface/verify_engine_surface.py --profile vue|vm --auto-bin <固定auto> --out-dir <自有隔离输出>`。T-01将第6节N-01～12注册为机器可核查的必需集（两端适用条件显式），每ID唯一；0选择/缺项/重复/未运行/GAP/失败或环境不可用均不得exit0。exit0须机器清单与实际执行记录全覆盖且全部PASS；不照搬旧30这个总数为新分母。

原生driver的默认exe/tool/source解析不再指向已删ui-026；从明确env/固定cohort清单发现并核验，错误路径具名失败，无静默回落陈旧二进制。MCP原生chrome动作按NVM标准真实消费；文档内真实输入通道与宿主作用分别证明。

Reader旧门入口保持 `python tests/basic/verify_standalone.py --profile vue|vm --auto-bin <固定auto>`。T-08在固定cohort复现R-05/R-06vm并定责，允许在本计划修复Reader测试/oracle及UI自身缺口；必须他仓源码修复时记录复现、owner、正式前置与确切交付版本，影响本计划守旧门前不得记AC-09通过。不能把“不是本计划src改动”当自动豁免，不能删反例/改预期/用HTTP绕开UI让门变绿。

### 5.5 产物与规范增量

现有UI compiler/runtime/tools及Reader tests/probe/engine_surface/**为真实改面候选；精确新增leaf/文件由T-00冻结后记录。新增耐久证据总结 `docs/research/reader-native-surface-closure.md`（未来文件），记录当前cohort、根因/假设、逐例输入/预期/实际/退出码、负控、反例修复与独立review结果；临时材料在plan-artifacts/plan-029/{auto-ui,auto-reader}/，不新增attachments，清理前摘要/必要反例复演信息必须保留。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/runtime/document-surface-boundary.md | 原生输入/IME/无障碍未证 → 仅在实测完成后标记本profile原生全链、隔离与失败控制已证，保留026历史baseline/限制 | 不把新证据回写成026当时全通过 | AC-02～09、AC-10 |
| SD-02 | add | docs/specs/runtime/document-surface-native-input.md | 无原生VM文档输入契约 → 实际实验表面API/事件身份与原文核验、布局/焦点/IME/无障碍、失效/关闭、严格聚合门边界 | 给后续EPUB产品化稳定而有限的前置，不承诺全格式 | AC-01～10 |

当前仅提出delta，不写canonical Spec/旧ledger/index；不改Reader产品schema Spec。没有完整通过证据时保留GAP，不因临时文档补齐就声明实现完成。

### 执行交接入口

当前仅new：已闭合基础是026研究与已有Vue指定样本链，不是原生全门。未来进入work先做T-00，确认双方HEAD/WIP/依赖，再T-01注册驱动，依表推进。未充分定责的原生输入、严格聚合、R-05/R-06vm分别归T-00/01/08；在决策/最小前置明确前不伪造实现结果。

## 6. 测试设计

全部为未来mandatory cases，owner_stage=work；由T-01注册、T-02～08建立真实执行。现有verify_engine_surface.py为统一入口，原生profile必须证明 `auto run -r vm --server=vm` 对应的窗口树/组件与source/tool身份；两端共用数据oracle但各自实际执行。未测/环境GAP不计PASS。

| Case ID | AC / task | 输入与真实路径 | 可观察oracle与边界 |
|---|---|---|---|
| N-01 | AC-01/03；T-00 | 同一宿主/文档比较系统/桌面/CDP真实输入，核驱动返回、焦点与内容命中，合法对照成对 | 列出首失败边界与因果，重复复现；不将JS代选/只有前台成功当根因解决 |
| N-02 | AC-02/03；T-02/03 | 实际AutoUI VM消费页真实拖选重复句第2处，包含中文/emoji与跨行 | .at前端可见状态、后端accepted、CFI解析出的resource/DOM范围、独立原文/出现位次和磁盘记录一致；不是独立Rust窗 |
| N-03 | AC-04；T-03 | 保存后正常关闭全部自有实例，换布局后新进程恢复；再异常退出冷恢复 | 恢复同一出现位置且在pane可见，CFI/quote/identity真实读回；相邻重复句作反差，旧库/v1/annotation_id逐字或ID不变 |
| N-04 | AC-05；T-04 | chrome↔正文↔输入框，真实中文IME组合/提交/取消与快捷键 | 组合期间不翻页/触发命令，最终文字逐字、焦点可返回；不能只用程序填值或CDP普通英文键入证明IME |
| N-05 | AC-06；T-05 | resize/DPI缩放/字号重排/滚动、遮挡/隐藏再显示，然后真实拖选/恢复 | 组件分配bounds、文档/浮层命中与目标原文一致；旧记录数增加或窗口尺寸改变不够 |
| N-06 | AC-06；T-05 | 系统无障碍工具在native所属树定位正文和操作，键盘访问 | 系统树/可操作性及焦点反馈可复查，非只有DOM ARIA；本profile真实内容，不承诺所有读屏平台 |
| N-07 | AC-07；T-06 | A/B两实例不同书/会话/记录，关闭A后发旧事件，新A重开，B继续真实选择 | A旧事件拒绝零写入/B不串文且不被kill，新A不接旧session；仅数据目录不同不能证明此门 |
| N-08 | AC-07；T-06 | 正常close/切文档/取消/崩溃分别走owner释放，旧MCP/IPC目标再调用 | 清理10s内owned进程/端口/表面释放，真实断连有界且与合法同方法对照；不杀他人、不靠全局taskkill补正常关闭 |
| N-09 | AC-03/07；T-07 | 错book/hash/session/schema/stale_seq、超过32KiB、路径逃逸/跨书/活动脚本、错CFI或quote；各配合法输入 | 真实接收消费端拒绝且零落盘/远程执行，合法本地引擎脚本/图片和新事件仍用；CFI真解析非前缀检查 |
| N-10 | AC-08；T-01/06 | 缺case/0选择/重复case/GAP/not_run/ENV、坏tool/占口/旧target/绑定错cohort | 聚合非零并有具体分类，合法完整集合才exit0；占口不杀现有实例，driver解析不回落已删树 |
| N-11 | AC-08；T-07 | 临时改变真实已保存locator或错identity使恢复/核验失败，再还原 | 同一原始oracle实际红→还原绿，聚合非零；不得仅mock失败标志/删除断言，负控不污染用户数据 |
| N-12 | AC-09；T-08/09 | 固定cohort重跑Vue旧链/两端守旧门与R-05/R-06vm原负例 | 所有必需项通过；坏图确实使原像素oracle失败且退出非零，正常图对照通过；路由404按实际owner修复后读到真实契约 |

工具/测试范围：UI遵循.cargo/config.toml与ui-test-selection Spec；compiler生成改面跑cargo tu（单进程），runtime/dialect按固定实际受影响测试集与必要cargo t档，tools影响再选cargo ti。T-00/01先登记实际新测试名/枚举非零与串行/native/GPU条件，T-09执行冻结集，不叠加多轮全仓。没有语言源码改动不在auto-lang跑cargo/docs_gen。旧30/30是参考，不替代新增必需清单；系统输入、原生生命周期与无障碍不能用纯单元测试代替。

实施者完成内部检查后进入execution_done；**最终复审必须独立上下文**，使用相同两仓执行位顺序检验，不由实施会话自称“复跑所以独立”。复审至少重演N-02/03、N-04/06、N-07/10/11，并查N-12真实原始oracle。用户未指定模型，不自动派代理或新chat；后续调度须保证独立review角色，若尚无该角色则review待办不能记通过。

## 7. 验收标准

| ID | owner_stage | 完成标准与验证 |
|---|---|---|
| AC-01 | work | 固定两仓source/CLI/tool/engine/profile与输入根因报告；T-00先冻结接入机制/单位/资源/deadline/实例定义与最小前置，保留合法对照，非预设环境归因 |
| AC-02 | work | 实际AutoUI VM原生组件可启动/装载/参与布局与生命周期，窗口树/组件/进程cohort归属可证；不是native_host.exe/独立浏览器替代 |
| AC-03 | work | N-02/09原生真实指定选区贯穿.at前端状态与后端核验/持久化，CFI解析/原句/重复出现位置正确；错身份/错锚零写、合法对照成功 |
| AC-04 | work | N-03正常与异常退出后新进程恢复同一内容/位置，布局变化仍成立；用户/旧v1与注释ID保护，不改产品schema |
| AC-05 | work | N-04原生焦点/中文IME真实组合、提交、取消、输入期间快捷键与返回正文通过，不能只测注入字符串 |
| AC-06 | work | N-05/06原生几何/遮挡/隐藏重现、系统无障碍内容与操作均有实证；无工具/未证即未通过 |
| AC-07 | work | N-07/08/09双实例/迟到拒绝、关闭断链、资源进程/端口与目录安全、失败隔离全通过，有原始对照与清理deadline |
| AC-08 | work | N-10/11严格机器必需集、GAP/缺项/重复/错cohort非零，真实变异使原oracle红再还原绿；无全局out-dir删除或旧硬编码工具回落 |
| AC-09 | work | N-12 Vue既有链与两端Reader守旧门在交付cohort全通过，R-05/R-06vm复现/owner/修复或正式前置交付绑定可核；记录债务不等于完成 |
| AC-10 | work | SD-01/02与两仓设计/路线/导航准确反映真实实验支持，证据报告逐例闭合、限制不超claim；026历史及当前产品边界不回写成已通过 |
| AC-11 | review | 独立上下文最终review绑定r1与实际source/tool/engine cohort，按第6节复演且AC-01～10均pass；partial/GAP不能给“有条件完成” |
| AC-12 | merge | review pass后固定交付两仓精确commit、主干landing、canonical核验、档案/计数/导航与自有材料/树清理回执真实完成，不吞他人WIP/不升级OS组合pin |

**退出原则**：所有work门完整通过才可execution_done；环境不可用、他仓前置未交付、任意必需GAP或负控无区分力均保留未完成。独立review pass后才reviewed，merge实际闭合后才archived；不以达到某个检查总数代替逐项AC，不用流程12/12冒充功能通过。

## 8. 执行步骤

本表是唯一当前清单，全部未执行。新增接口/测例的建立任务明确如下，不把计划文案当可运行实现。

| ID | owner_stage | 依赖 | 路径/动作与结果 | 验证与关闭条件 | AC |
|---|---|---|---|---|---|
| T-00 | work | 后续进入work | 指定UI树与Reader main核HEAD/WIP；复现native_host/driver输入断点；冻结真实VM接缝/预算/系统工具/测试范围并记报告 | git status/rev-parse、工具hash、真实窗口/驱动返回/内容事件对照，最多两接法；不扩写他仓、不改用户库 | AC-01 |
| T-01 | work | T-00 | 改Reader verify_engine_surface/profile/driver契约，注册N-01～12与必要的分层子项，建立非零严格汇总与安全输出目录守卫 | 缺项/空集/重复/GAP/环境/错cohort受控输入真实非零；T-02～08再使功能例可运行，不伪标已绿 | AC-08 |
| T-02 | work | T-00/01 | UI最小compiler/runtime/tools实验表面，Reader probe.at声明并从真实native VM启动；native_host仅对照 | NVM-01～06窗口/组件/cohort，表面区域/回调能实际进.at前端；生成产物不手改 | AC-02 |
| T-03 | work | T-02 | Reader前端model/handler、既有back/api.at与adapter、driver_vm补真选择/核验/保存/冷恢复 | N-02/03/09逐字/出现位次/CFI真解析、前端可见状态与磁盘读回；正常/异常重启，旧数据不变 | AC-03/04 |
| T-04 | work | T-03 | UI输入/焦点/组合事件最小修复与实际driver取证 | N-04真实IME组合/取消/提交、焦点返回与快捷键不误触，全通过 | AC-05 |
| T-05 | work | T-03/04 | UI布局/文档表面resize/DPI/遮挡/隐藏生命周期与系统无障碍接缝 | N-05/06目标原文/命中/系统树可操作均实测；不是records计数或纯ARIA | AC-06 |
| T-06 | work | T-03～05 | 两仓会话/实例配置与自有process管理，补双实例、关闭/迟到/端口/坏tool/占口专属门 | N-07/08/10正负对照；只收割自有树，10s清理，另一实例继续可用 | AC-07/08 |
| T-07 | work | T-06 | 保原协议/资源安全，独立oracle与真实错锚/错身份变异，补适用负控回归 | N-09/11原拒绝零写、合法对照绿，变异红且聚合非零→还原绿；不得吞报错 | AC-03/07/08 |
| T-08 | work | T-00可先诊断；最终依T-07 | 复现Reader R-05/R-06vm、分析实际owner；修本仓允许面或绑定他仓正式前置 | N-12原测试条件保留，404/像素负控失败机制真实闭合；外仓前置未到则保持未完成 | AC-09 |
| T-09 | work | T-08全部通过 | 冻结cohort跑两个profile的完整新清单及旧Reader门、本仓定向cargo选择；写证据总结、SD-01/02与两仓路线/导航并内部检查 | 现有入口 `python tests/probe/engine_surface/verify_engine_surface.py --profile vue|vm --auto-bin ... --out-dir ...` 完整且exit0；新清单非旧30总数；git diff --check/链接/source绑定 | AC-01～10 |
| T-10 | review | T-09 | 独立上下文在同树顺序复演规定路径与变异，检查source/Spec/范围，写final结论 | AC-01～10全部pass，无partial/GAP；否则needs_fix/needs_replan回原树，不能自评归档 | AC-11 |
| T-11 | merge | T-10 pass | 固定两仓提交/landing/canonical、归档archived/、修导航、材料摘要后清自有scope；fresh guard后移除树/分支 | 真实12/12与archive路径/cohort一致、恢复信息足够；不动他人WIP/OS pin/ledger，不把旧026回执改为新证据 | AC-12 |

## 9. 复审记录

### 2026-11-02：new阶段交接

- stage: new；auto-ui/PLAN-029 r1；outcome: pass（起草可交接）；next: work（本轮只建立计划，未执行）。
- 新增T-00～11、AC-01～12、N-01～12、SD-01/02；0/12。合同沿用026真实实验资产，严格收口其未通过项，不重开026，不取得任意第三仓改源码权限。
- 当前事实为静态源码/Spec核查，尚无新构建/窗口/实测/独立review；不能从本文推导原生已通过。UI原有未跟踪设计、028草案与共享号源WIP保持，Reader号源007未使用。
- 起草自检：0～10节、12任务/12AC/12必需用例编号完整，029活动/归档区唯一，Reader号源未变；本次8份文档的114个本地引用已核，新改链接有效，UI README原有9个旧失效引用保持不动；git diff --check通过。此项只证明草案结构，不是运行或验收通过。

## 10. 待澄清事项

无阻塞起草的用户问题。以下由未来任务负责，不能要求用户先替根因分析选答案：

1. 输入失败究竟来自driver/焦点/宿主/引擎/桥哪一层？T-00做同条件对照；静态代码线索不能直接判定自动化环境故障。
2. 原生VM最小文档leaf与系统无障碍接口如何接？T-00冻结真实方案，T-02/05实现验收；复杂度超出有限接缝时回规划，不删门。
3. R-05/R-06vm的实际owner与依赖版本？T-08复现后明确，必要正式前置未交付则相关AC不绿；不靠“src零改动”或只登记债务过门。
4. 后续独立review角色尚未指定模型，按流程在work完成后安排独立上下文；本轮不派代理/创建新chat，也不将实施会话复跑改称独立复审。
