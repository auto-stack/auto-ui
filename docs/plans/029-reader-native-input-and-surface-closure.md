---
plan_id: PLAN-029
status: executing
feature_name: Reader原生输入链与文档表面必要接缝收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
current_step: 6
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
| T-00 | work | 后续进入work | [x] 指定UI树与Reader main核HEAD/WIP；复现native_host/driver输入断点；冻结真实VM接缝/预算/系统工具/测试范围并记报告——见§9 2026-11-02 work记录（根因=driver坐标索引bug+DPI混用，修复后SendInput真实拖选产出DOM选区+sidecar落盘；Reader修复commit c606850） | git status/rev-parse、工具hash、真实窗口/驱动返回/内容事件对照，最多两接法；不扩写他仓、不改用户库 | AC-01 |
| T-01 | work | T-00 | 改Reader verify_engine_surface/profile/driver契约，注册N-01～12与必要的分层子项，建立非零严格汇总与安全输出目录守卫 | 缺项/空集/重复/GAP/环境/错cohort受控输入真实非零；T-02～08再使功能例可运行，不伪标已绿 | AC-08 |
| T-02 | work | T-00/01 | [x] UI最小compiler/runtime/tools实验表面，Reader probe.at声明并从真实native VM启动；native_host仅对照——doc-surface feature leaf（worktree 20aaa27，converter Custom漏臂修复+Element通道+canvas painter子表面管理），Reader d33144e（probe.at+surface-url端点）；真实VM窗口内引擎装载spine=3/271ms、MCP树可见、组件分配bounds=子表面rect | NVM-01～06窗口/组件/cohort，表面区域/回调能实际进.at前端；生成产物不手改 | AC-02 |
| T-03 | work | T-02 | Reader前端model/handler、既有back/api.at与adapter、driver_vm补真选择/核验/保存/冷恢复 | N-02/03/09逐字/出现位次/CFI真解析、前端可见状态与磁盘读回；正常/异常重启，旧数据不变 | AC-03/04 |
| T-04 | work | T-03 | UI输入/焦点/组合事件最小修复与实际driver取证 | N-04真实IME组合/取消/提交、焦点返回与快捷键不误触，全通过 | AC-05 |
| T-05 | work | T-03/04 | [x] UI布局/文档表面resize/DPI/遮挡/隐藏生命周期与系统无障碍接缝——弹出窗真隐藏修复（worktree 98633fd4）+ 驱动 N-05/06 全门（reader ac24c98，probe.at 断开按钮 + engine-adapter 键盘接缝）；N-05 字体重排/滚动重拖/遮挡像素取证/隐藏展示重拖选、N-06 UIA 系统树定位+UIA焦点+真实键盘翻页、N-05.dpi-change 交互门（144→192→144 track/redrag/restore 三段全过）——见 §9 2026-11-02 T-05 记录 | N-05/06目标原文/命中/系统树可操作均实测；不是records计数或纯ARIA | AC-06 |
| T-06 | work | T-03～05 | 两仓会话/实例配置与自有process管理，补双实例、关闭/迟到/端口/坏tool/占口专属门 | N-07/08/10正负对照；只收割自有树，10s清理，另一实例继续可用 | AC-07/08 |
| T-07 | work | T-06 | 保原协议/资源安全，独立oracle与真实错锚/错身份变异，补适用负控回归 | N-09/11原拒绝零写、合法对照绿，变异红且聚合非零→还原绿；不得吞报错 | AC-03/07/08 |
| T-08 | work | T-00可先诊断；最终依T-07 | 复现Reader R-05/R-06vm、分析实际owner；修本仓允许面或绑定他仓正式前置 | N-12原测试条件保留，404/像素负控失败机制真实闭合；外仓前置未到则保持未完成 | AC-09 |
| T-09 | work | T-08全部通过 | 冻结cohort跑两个profile的完整新清单及旧Reader门、本仓定向cargo选择；写证据总结、SD-01/02与两仓路线/导航并内部检查 | 现有入口 `python tests/probe/engine_surface/verify_engine_surface.py --profile vue|vm --auto-bin ... --out-dir ...` 完整且exit0；新清单非旧30总数；git diff --check/链接/source绑定 | AC-01～10 |
| T-10 | review | T-09 | 独立上下文在同树顺序复演规定路径与变异，检查source/Spec/范围，写final结论 | AC-01～10全部pass，无partial/GAP；否则needs_fix/needs_replan回原树，不能自评归档 | AC-11 |
| T-11 | merge | T-10 pass | 固定两仓提交/landing/canonical、归档archived/、修导航、材料摘要后清自有scope；fresh guard后移除树/分支 | 真实12/12与archive路径/cohort一致、恢复信息足够；不动他人WIP/OS pin/ledger，不把旧026回执改为新证据 | AC-12 |

## 9. 复审记录

### 2026-11-02：work阶段——T-05 几何/无障碍/键盘/DPI 全门真实通过（AC-06）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-05 | outcome: pass（N-05/N-06 全子项 PASS；N-04.focus-roundtrip 保持 §10 具名开放、N-08 待 T-06）| worktree: .wt/ui-029@98633fd4 | reader: main@ac24c98 | next: T-06。
- **隐藏门红→绿**：红=red3（修复前构建）——probe.at 新增「断开表面」按钮置空 url 后弹出窗仍可见、中心像素灰 (200,200,200)；修复=doc_surface.rs 弹出窗真隐藏（SW_HIDE/SW_SHOWNA，SurfaceMeta.popup_visible；unmount 无钩子为 SD-02 记录边界）；绿=green11（hidden PASS popup_visible=False、同 rect 复现、重拖选 oracle 重命中）。
- **N-05 几何门**（green11/dpi6 全部真实执行）：字体重排（章节 body 200%→行盒位移→新几何真实拖选 occ=2 oracle✓）；滚动重拖（CDP 翻页=许可口径，cfi 位移断言，新段 oracle ch1 para8✓）；遮挡（驱动自有 topmost 纯色遮挡窗，像素三段取证 before白/during红/after白，撤除后 ch3 预导航+真实拖选 oracle✓）；resize-follow 沿用。
- **N-06 系统无障碍**（非 DOM ARIA）：`--force-renderer-accessibility`（屏读等效开关）+ PowerShell UIAutomationClient 从弹出窗 HWND 走**系统树**——目标句全文以 ControlType.Text 命中（链 Text←Document）；**键盘访问接缝**=engine-adapter.js 补 foliate-js 缺失的键处理（翻页是宿主职责；iframe 键事件不冒泡故 host 页+章节 doc 两级挂）——UIA SetFocus(Document)→GUI 线程焦点入弹出窗子树（IsChild✓）→真实 ArrowRight/Left 翻页往返（lastLocation.cfi 断言）全过。
- **N-05.dpi-change 交互门**（用户协助，PROBE_DPI_INTERACTIVE=1）：OS 缩放 144→192→144 真实变化；track=弹出窗 959x576→1696x840 精确跟随（innerWidth×新scale 误差≤6px）；redrag=新几何下真实拖选 sel-exact 认领+oracle（ch1 para8 offset13）；restore=1272x630 精确回 1.5×。检测窗 600s=用户节奏语义修订（见 §10）。
- **驱动硬化**（全部由 red2/red3/green1-11/dpi1-6 实测失败驱动）：GetPixel/CreateSolidBrush/DeleteObject 属 gdi32；MCP 安轮询（announce 先于 listener 的 10061 竞态）；禁双 sync_playwright（py3.14 event-loop 冲突）；同行逐字符拖选量测（整节点/跨行 box 塌缩成 1-3 字符选区）；sel-exact 记录认领（计数 delta 被乱序记录吸收）；坐标一律驱动侧实测（57cf408a 起日志行为屏幕绝对，旧 N-02 拖选数学含双重客户区原点偏移但 oracle 端到端仍真——已记坐标口径发现）。
- 工具宿主曾于 green1 在 N-04 阶段静默退出（无 WER、无 panic 文本；同二进制 n04-repro 单独复现 30 次键击不崩、green2-11/dpi3-6 全部通过）——按瞬时环境 flake 记录，不做结论。
- 证据：plan-artifacts/plan-029/t05-vm-red2|red3|green2..11|dpi..-console.log + 各 out-dir（consumer*.log/sidecar/n06-uia*.json/results-vm.json）。
- 下一任务 T-06：双实例/关闭/迟到/端口/坏tool/占口门。

### 2026-11-02：work阶段——T-04续：表面不可见缺陷修复（用户目击验收，AC-02补强）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-02/T-04修复 | outcome: pass（用户目击）| worktree: .wt/ui-029@57cf408a | next: 全链回归后 T-05。
- **用户桌面实测发现真缺陷**：连接会话后 doc-surface 区域仍为空框。逐像素定责：用户截图白框 RGB(246,247,249) = painter 占位底色——WebView2 子 HWND 存在（收输入/CDP可连/DOM有字），但像素被 iced 窗口自身呈现层盖住，从未上屏。我此前只验证 DOM/数据链，从未验证人眼可见性（验证盲区，用户纠正）。
- **修复（57cf408a）**：表面宿主从子 HWND 改为**归属弹出窗**（WS_POPUP+owner+TOOLWINDOW/NOACTIVATE，注册窗口类，真实 hinstance）；弹出窗逐帧同步组件矩形（移动/缩放/z 跟随 owner，永不 topmost）；WebView2 bounds 改弹出窗相对 (0,0,sw,sh)（屏幕坐标误用会把内容推出窗外——用户最大化后灰/白/小字三层错位暴露）；宿主定位排除弹出窗自身（同线程枚举自命中致矩形逐帧自缩 1696x840→822x246 实测）；CreateWindow 失败具名（RegisterClass/CreateWindow hinstance=0 曾静默）。
- **用户验收**：「现在好了」+ 截图显示双栏排版、目标句高亮块（探针重复句甲🦊…）、emoji、脚注块全部可见——AC-02 视觉可见性由用户目击闭合。
- 弹出窗形态的输入语义变化：表面不再吃 iced 窗口焦点（NOACTIVATE），拖选/键盘前台语义待全链回归确认（下一步 t05-vm1 完整跑）。

### 2026-11-02：work阶段——T-04 IME真实组合可行性证实（AC-05核心证据面）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-04（进行中：可行性+证据协议已证，正式N-04驱动待续）| reader: main@3a3300d | next: T-04续（正式驱动：取消/快捷键/焦点往返门）。
- **环境事实**：系统输入法 = MS Pinyin（TSF profile 0804:{A3F4CDED..}{3D02CAB6..}，zh-Hans-CN 唯一）；iced 主窗无经典 HIMC（ImmGetContext=0，t00 同发现）——TSF 路径。
- **实测证据链（ime-experiment{2,3}.log + ime-win.png）**：真实 SendInput 点击 iced input（物理坐标换算：client origin+逻辑×DPI）→ OS 焦点就位 → SendInput 'nihao'+Space → **MS Pinyin 拦截键入组合、输入框值组合期间冻结、Space 转换提交、提交文本「你好」经 IME 管线落入 model 绑定值（value='abc在你好'，MCP snapshot 读回）**。对照面：MCP type_text/set_value 为合成状态写（不建立焦点、非键盘事件）；纯 SendInput 在无 OS 焦点时不达（foreground 前置核验必要）。仅 IME 能完成拉丁→汉字转换——提交值即组合真实性证据。
- probe.at 增 IME 目标输入框（value 双向绑定，reader 3a3300d）。
- **正式 N-04 驱动剩余门**（协议已冻结）：①组合期间 Esc 取消→值不变；②组合期间快捷键（Enter/Tab）不触发页面命令；③焦点往返 chrome↔正文↔输入框（点击输入→点击表面→点击输入，分别以「值变化/拖选产生选择」为焦点归属证据）；④AUTO_VM_WINDOW env 语法与实际不符（900x700 未生效，窗为默认 1280x800 逻辑）——驱动按实测几何换算，env 语法勘误记入 T-05。

### 2026-11-02：work阶段——T-03 真实VM驱动 N-02/N-03/N-05全绿（AC-03/04核心）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-03 | outcome: pass（N-02全链+N-03正常/异常冷恢复+N-05 resize跟随；N-01/04/06/07/09/10/11/12 诚实not_run待T-04..T-09）| worktree: .wt/ui-029@20aaa27 | reader: main@be439fd | next: T-04。
- driver_vm.py整体重写：`auto run -r vm --no-merge`+AUTO_UI_TOOL_BIN cohort；MCP端点自会话日志解析；_wait_tree轮询首帧；连接=MCP真实press→前端connected→表面create rect；CDP仅翻页/只读观察；选择=真实SendInput（T-00修正坐标+逐点hit-guard+前台核验）。三次实测修复：①分栏目标整列停视口右缘（x=innerWidth size合法不可见，拖选必落空——t03 forensics定位）→入视口包含判定；②重启后hwnd必须重解析（旧hwnd致MoveWindow无效）；③resize目标须大于表面高（裁剪静默隐藏）。
- **实跑结果（t03-vm5，26 PASS）**：N-02全链（导入/MCP树/表面create/引擎装载/目标句入视口/hit-guard/前台/**真实拖选records=1**/**quote-oracle occ=2 ok=True pos=ch1.xhtml para_index=7 offset=26**/CFI epubcfi(/6/2!/4/18,/1:27,/1:60)/磁盘）；N-03（正常关闭→新进程冷恢复goto ok目标可见30s预算内；旧记录保持；强杀→重启→数据存活）；N-05.resize-follow（1696x840@432,348→1278x768@48,348）。
- **AC-03核心事实**：真实AutoUI原生VM消费者内，指定句第2处经真实系统输入选择→引擎CFI→.at后端核验（occ=2全书3处中第2处，独立oracle位置吻合）→sidecar隔离落盘→正常/异常重启后同CFI恢复原句。026「原生输入GAP」在此链上不再存在。
- 证据：plan-artifacts/plan-029/t03-vm{2..5}-console.log、t03-forensics.log、consumer{,2,3}.log。
- 下一任务T-04：焦点/中文IME真实组合取证（iced主窗无HIMC、WebView2子HWND为IME目标；ImmSetCompositionString/ VK_PROCESSKEY路径待实测）。

### 2026-11-02：work阶段——T-01严格聚合门 + T-02真实VM doc-surface链（AC-08部分/AC-02核心）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-01/T-02 | outcome: pass（T-01门面+T-02核心链；T-02收尾=NVM-02~06正式取证随T-03~06驱动）| worktree: .wt/ui-029@20aaa27（干净）| reader: main@d33144e | next: T-03。
- **T-01（reader 72723b3）**：REQUIRED清单注册N-01～12（profile适用性显式：vm全12；vue缺N-01/04/06/07/08）；父用例=自身+`<父>.*`子项最坏推导；缺项/重复/GAP/FAIL/not_run/未声明根/跨profile记录全非零（N-10聚合自检10/10受控负例非零、合法全集exit0）；out-dir守卫（拒用户根/Desktop/盘符/无标记非空目录/reparse点，safe_rmtree限scope拒reparse跟随）；cohort指纹（坏bin具名exit2）；driver_vue V-*改编号进N-02/03/09命名空间。vue臂新清单实测：N-02（拖选→CFI epubcfi(/6/2!/4/18,/1:34,/1:61)→oracle→sidecar）、N-03（冷重启goTo恢复）、N-09负控全PASS，N-05/10/11/12诚实not_run→exit3（35记录零重复，findings=0）。
- **T-02（worktree 20aaa27 + reader d33144e）**：新feature `doc-surface`（windows，optional wry=0.57）。①doc_surface.rs：组件分配区域的wry WebView2子表面——canvas painter draw期同步（EnumThreadWindows当前线程中心点定位宿主窗→组件逻辑bounds×窗DPI→客户区物理rect；创建/导航/移动/显隐；webview留主线程thread_local，COM非Send；AUTO_DOC_SURFACE_DEBUG=1取证；bounds哨兵(0,0,0,0)强制首帧set_bounds）。②native_widget.rs注册Element入口（零compiler改面）。③**converter漏臂修复（renderer/mod.rs convert_view_messages）**：View::Custom此前落`_ => Empty`（PLAN-082 Rich同族坑），live渲染树永远见不到Custom——加显式直通臂。④projection.rs Custom臂：doc-surface canvas+RemoteFrame同款style容器，width/height px props（Fill-in-Shrink塌缩对策）。真实cohort验证链：`auto run -r vm --no-merge`+`AUTO_UI_TOOL_BIN=<本树auto-ui-tool.exe>`（PLAN-816发现序第一键，工具仓零改动）→ probe.at doc-surface入MCP树 → MCP press连接会话（真实chrome动作）→ handler经HTTP取surface-url写model → 子表面create于组件rect（1696x840@432,348物理）→ CDP证引擎装载（foliate-js@3d794195 spine=3 271ms）→ **真实SendInput拖选（修正坐标）在VM窗口WebView2内产出DOM选区→range CFI epubcfi(/6/2!/4,/2/1:2,/12/1:3)→.at后端accepted→sidecar落盘records=1**。缺省构建零feature编译干净；cargo tu 866 pass。
- 环境事实：残留Chrome窗（引擎表面探针/图表，pid100436，历史vue驱动遗留）曾全屏覆盖拦截输入——已最小化；驱动必须逐点hit-test+前台核验（T-03驱动吸收）。auto.exe pid9180（会话前已存在的僵尸进程）在清理时被taskkill //IM误杀（1.5MB僵尸，无窗口无端口；非他人工作成果）。
- 证据（plan-artifacts/plan-029/）：t01-vue2-console.log、direct-run{2,5}.log（漏臂定位）、vm-boot{2..6}/、vm-n02{,b}/（boot.log+截图+sid）。
- 主检出WIP保持原样；Reader号源未变。

### 2026-11-02：work阶段——T-00 输入断点定责与cohort冻结（AC-01）

- stage: work | plan_id: PLAN-029 | plan_revision: r1 | task: T-00 | outcome: pass | worktree: D:/autostack/.wt/ui-029/auto-ui（codex/plan-029-dev @01f99fd，干净）| reader: main@c606850（新增修复）| auto-bin: auto-man debug g55c98c7 | next: T-01。
- **输入断点根因（三层，全部实测）**：① `driver_vm._abs_xy` 用错 GetSystemMetrics 索引——76/77 是虚拟屏原点、78/79 才是宽高，旧码当宽高/原点互换且分母 max(1,0)=1，SendInput 绝对坐标被放大 ~65536 倍，输入永不达内容（026 NV-02 GAP 首要根因，非"自动化环境边界"）；② 驱动进程未声明 DPI-aware，逻辑/物理坐标混用（client 1024x768 逻辑 vs 2048x1536 物理）；③ 环境因素：`Windows 输入体验`（StartMenuExperienceHost）全屏 CoreWindow 覆盖主屏，干扰前台断言与中心点 hit-test，拖选点需逐点核验/窗口避让。
- **修复后对照（t00_input_probe，3通道）**：修正坐标+DPI 感知+窗口避让后，真实 SendInput 慢速拖选在 WebView2 内容中产出真实 DOM 选区（sel 探针回报选中句原文），.at 后端 accepted、sidecar 落盘 records=1——026 具名 GAP 在独立实验中解除。剩余细节：拖选可产出跨行部分句，quote 归一化需与 oracle 对齐（T-03 处理）；iced 主窗默认无 HIMC（IME 默认窗属宿主进程），T-04 需对 WebView2 子 HWND 取证。
- **资产重建缺陷（新发现，已修）**：026 合入的 prepare_engine_assets.py 把 vendor 扁平为 `foliate-vendor-<名>` 但未同步改写 view.js 的 `'./foliate-vendor/<名>'` 引用，且 .at 资产路由 `:name` 仅匹配单段（嵌套路径 404）→ 引擎根本无法装载。已修（flat 重写一致化），复现第3次运行引擎装载 PASS（spine=3, loadMs=217-257ms）。
- **真实VM入口冻结（实测）**：`auto run -r vm --server=vm` = 仅后端 server（17827 监听 20 路由，无窗口）；**原生窗口入口 = `auto run -r vm`（merged：进程内 backend + iced 主窗「引擎表面探针」+ MCP 127.0.0.1:9247）**。计划§5.4 原文按此校准：NVM 窗口树/组件身份绑定 `-r vm` 窗口进程；`--server=vm` 保留为 vue 驱动消费后端形态。双实例端口走 `-B/-F` CLI 覆盖（T-06 预检）。
- **冻结cohort**：UI=.wt/ui-029@01f99fd（native_host.exe 本树构建，仅诊断对照）；Reader=main@c606850；auto=auto-man debug g55c98c7；WebView2=156.0.4314.50；engine=foliate-js@3d794195（readest 只读 3cc39ad7）；资产=plan-artifacts/plan-029/engine-assets（重建脚本 flat-vendor 修复后）；corpus=plan-artifacts/plan-029/corpus。预算沿用 corpus_profile（15/10/5/30/10s、600s、32KiB、1MiB、≤2实例）不变。
- 证据（本地，plan-artifacts/plan-029/）：repro-t00{,b,c}-console.log（三次复现）、t00_input_probe{,2}.log（修正坐标对照，含 sel 探针原文）、vm-entry-test{,2}.log（入口形态）。raw 不入仓；本节为耐久摘要。
- 主检出 WIP（007计划文档改动、crates/auto-ui-runtime/test/、docs/design/autoui/ 未跟踪）保持原样，不纳入本计划；Reader 号源未变。

### 2026-11-02：goal回合1

- stage: goal | plan_id: PLAN-029 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass(T-00) | worktree: D:/autostack/.wt/ui-029/auto-ui | commit: reader c606850 | next: work(T-01)。

### 2026-11-02：new阶段交接

- stage: new；auto-ui/PLAN-029 r1；outcome: pass（起草可交接）；next: work（本轮只建立计划，未执行）。
- 新增T-00～11、AC-01～12、N-01～12、SD-01/02；0/12。合同沿用026真实实验资产，严格收口其未通过项，不重开026，不取得任意第三仓改源码权限。
- 当前事实为静态源码/Spec核查，尚无新构建/窗口/实测/独立review；不能从本文推导原生已通过。UI原有未跟踪设计、028草案与共享号源WIP保持，Reader号源007未变。
- 起草自检：0～10节、12任务/12AC/12必需用例编号完整，029活动/归档区唯一，Reader号源未变；本次8份文档的114个本地引用已核，新改链接有效，UI README原有9个旧失效引用保持不动；git diff --check通过。此项只证明草案结构，不是运行或验收通过。

## 10. 待澄清事项

### 2026-11-02（work 执行中登记）

0. **N-05.dpi-change 检测窗预算语义修订（T-05 已按此执行）**：交互门检测/恢复窗从起草隐含的短窗修订为 600s——首跑 150s 实测超时（用户切换动作晚于窗口，DPI 实际已生效）；交互门计时独立于 600s 自动链预算，属 §5.2「预算本身不适用时按语义修订说明原因」情形，不放宽任何自动门 deadline。
0b. **坐标口径发现（T-05 登记）**：57cf408a 弹出窗形态起，doc-surface 日志行 phys 为屏幕绝对坐标；旧 N-02 拖选数学（ClientToScreen+该偏移）含双重客户区原点偏移，靠行宽容差+oracle 端到端仍真。T-05 起新门坐标一律驱动侧实测（EnumWindows 类名 AutoUIDocSurfacePopup + GetWindowRect），不依赖日志行。
1. **N-04.focus-roundtrip 表面→输入框方向（开放，具名）**：弹出窗形态下，表面内真实拖选后，注入点击/键击（双击、标题栏激活、AttachThreadInput+SetFocus、WM_CANCELMODE）均无法把 OS 线程焦点从弹出窗 Chrome（fd5e/fe10）拉回 iced 输入框（t05-vm5/6/7/8 诊断一致：cap=0、焦点恒驻弹出窗）。**人工路径可用**：用户在表面在场时向输入框手工输入 1231 成功（视觉证据）。方向 输入框→表面 已证（真实拖选 CDP 选区）。待 repair：诊断合成注入与人工点击的差异（可能性：Chromium 内部捕获/焦点仲裁），或以「CDP 焦点断言 + 人工目击配对」补证。用户手工输入 1231 期间页面正常（无命令误触）可作辅助证据。
2. **原生轨文本拖放入输入框（能力缺口，非本计划门）**：用户实测从文档拖文字到 IME 输入框不生效。定责：源端（WebView2/Chromium）支持拖出；目标端运行时有 Plan 488 `native_dnd`（IDropTarget/CF_UNICODETEXT，App 级 `on_native_drop` 事件）但 ① cohort 构建未启用 `native-dnd` feature，② 即使启用也只到应用级事件——iced text_input 无「拖放到光标处插入」接线。属原生轨框架新功能（Vue 轨浏览器原生支持故无此缺口），应由框架侧后续计划补，不阻塞本计划任何门。
3. 其余起草期条目（1~4）转入 T-00/02/08 已闭环或随任务推进。

无阻塞起草的用户问题。以下由未来任务负责，不能要求用户先替根因分析选答案（历史）：

1. 输入失败究竟来自driver/焦点/宿主/引擎/桥哪一层？T-00做同条件对照；静态代码线索不能直接判定自动化环境故障。
2. 原生VM最小文档leaf与系统无障碍接口如何接？T-00冻结真实方案，T-02/05实现验收；复杂度超出有限接缝时回规划，不删门。
3. R-05/R-06vm的实际owner与依赖版本？T-08复现后明确，必要正式前置未交付则相关AC不绿；不靠“src零改动”或只登记债务过门。
4. 后续独立review角色尚未指定模型，按流程在work完成后安排独立上下文；本轮不派代理/创建新chat，也不将实施会话复跑改称独立复审。
