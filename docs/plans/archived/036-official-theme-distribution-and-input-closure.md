---
plan_id: PLAN-036
status: archived
feature_name: 正式主题包发行与生成输入收口
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 10
total_steps: 10
supersedes_spec_components: [docs/specs/ui/official-theme-package.md, docs/specs/ui/theme-app-migration.md]
new_spec_components: [docs/specs/build/official-theme-delivery-and-inputs.md]
touched_goals: []
affects: [official-themes, auto-ui-tools, managed-front-inputs, generated-ui-delivery, man-full-sdk-install]
depends_on: [auto-ui/PLAN-031, auto-ui/PLAN-033, auto-lang/PLAN-906, auto-lang/PLAN-910]
---

# [auto-ui/PLAN-036] 正式主题包发行与生成输入收口

## 0. 变更摘要

承接031/033已经交付的主题包与十个基础示例，补齐**正式包进入Full SDK/安装和受管应用交付渠道、实际生成/运行/复用统一识别主题内容**的有限纵向模块。UI持唯一主合同，Man参与安装器，语言仅在真实必要时扩中立生成输入合同。保持主题单源和910的context/member/锁/lease，不新增registry、通用包管理器或缓存体系。

当前drafting r1、0/10。2026-11-03用户授权核查、立正式计划和刷新路线，**暂不实施**；本轮无worktree、生产改动、构建/测试/GUI、包安装或发布，无其他agent/chat创建或联系。既有WIP原样保留。031归档中的F-05渠道缺口由本包接续；031/033不重开，历史通过不当作本包渠道验收。

## 1. 目标

- G1：Full安装SDK和`auto gen --delivery-root`实际携带可验证的官方包及来源身份；外部消费者不手工拷贝教学目录、不依赖兄弟检出或绝对开发根。Core安装仍无UI依赖/资产。
- G2：Vue生成/运行、VM源码运行、Rust生成/构建/复用在消费正式包时使用同一身份与内容规则；非法包、不可读必需输入和收据写入失败明确失败，不以警告或旧产物充新成功。
- G3：包元数据、文件集合及内容的合法变化驱动失效，即使同版本、同长度、同mtime；未变不无谓改写。生成期间输入变化不能为旧字节签发新收据。
- G4：安装/再发行失败保留旧可用集合；外部fresh→locked→第二根重放、真实运行及失败恢复有可复核证据，向OS053和B2给固定来源/接口/升级交接。

非目标：重做031主题scope/原子选择、表达式design/全部控件、State/Feature/HMR；迁移B2/B3或其他产品；OS通知/持久化/真实系统事件；918收据ID校验/审计规则与重冻；Web D-906-1、Man D-910-2；远端registry/签名/公网发布、所有缓存治理、全仓/全平台全绿。

## 2. 架构方案

UI `packages/themes/` → 现compiler校验器 → UI工具的受管快照/身份描述 → Man Full SDK与UI delivery staging → 外部App依赖解析 → 各真实入口的准入/内容输入 → 实际生成、运行和受管复用。

主题语义与校验归UI；Man只装配经验证的主题资产并绑定安装清单，不复制颜色/另写主题解析器。跨语言边界传中立的输入键/摘要/来源描述，不让语言facade、generation-core反向依赖UI。优先扩UI已有front inputs，不预设必须改generation-core或receipt schema；确需中立扩展时在T-01列字段/兼容/消费者与实际Spec差额后实施。

发布沿现staging→完整校验→受管换装，UI包快照自身也须满足目标旧字节保留/可恢复要求。所有输入在使用前准入并绑定实际消费的同一内容；结束时重核或消费冻结快照，不能事后读取新包给旧产物签名。主题数据receipt与Cargo编译context不同，不能把换肤自动等同所有Rust依赖重编。

## 3. 技术栈

现Rust `auto-ui-compiler::design_tokens::theme_package`、UI tools `theme_package.rs`/`run_vue.rs`/`vue.rs`/`run_vm.rs`/`rust_ui.rs`；Man Python安装事务/清单；语言`FrontReceipt`与既有generated-ui consumer。继续使用原包schema 1、themes 0.2.0为合法基线，不预定无需求升版。

## 4. 需求分析与背景调查

### 4.1 授权、规范和观察来源

使用auto-plan-new及verification-contract。已读UI `docs/specs/README.md`、`ui/official-theme-package.md`、`ui/theme-scope-and-publication.md`、`ui/theme-app-migration.md`、`ui/basic-examples-theme-adoption.md`、`build/ui-compilation-configuration.md`；语言[生成交付](../../../auto-lang/docs/specs/repository-boundaries/generated-ui-distribution.md)、[生成生命周期](../../../auto-lang/docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md)，Man[安装说明](../../../auto-man/docs/installation.md)。UI无根AGENTS.md，沿本仓plans README的主仓/archived/独占取号与跨仓唯一合同规则；执行再读参与仓当时指令。

观察HEAD：UI `fa01bd790a2e3fa4c81821203560019159632c52`；语言 `593c334805909c25518b7443f9a11707712a45da`；Man `47d8f7b34e584dccc90fefe340629bbe318c6405`。这是观察快照而非未来执行冻结，OS053组树WIP不消费。主检出存在他方源码/文档WIP，不提交/还原/清理。

| 代码输入 | 本轮SHA256 |
|---|---|
| UI tools/src/theme_package.rs | 621ebc29d12392daa4a6ddd919fa0cba0891cdf52e51e7d483b0a9555d462a8d |
| UI tools/src/rust_ui.rs | 715c6cd344fb4df9dd047692788934d71dbcfc89f98d4b56ae325a69986a9334 |
| Man scripts/man-install.py | 003e0ee4639543c6314deb4dd247c56f26f0490890ab33ee1e7a786898f1c691 |
| language generation-core/src/generated_build.rs | 1782b9a812866bcae884d6f14e8b1fff2a69fbf6fc2310b55b2534de026fe890 |

### 4.2 已交付与真实剩余

| 输入 | 接受的实际交付 | 本包差额 |
|---|---|---|
| [031](archived/031-theme-scope-publication-and-package-closure.md) archived11/11 | 独立review pass@b6c26186，代码在主线祖先；官方包、校验/快照原语、scope与003/010交付 | F-05渠道未接；VM/Rust auto build未接同一receipt；不重新认领scope/控件 |
| [033](archived/033-basic-examples-theme-migration.md) archived9/9 | d734f1fe已在master祖先；六例双端接入并修投影scope/watch竞态；B1累计10、余24 | 其复审来源限制和7个具名既有红保持；非全局绿，不将新渠道验收借旧外部fixture代替 |
| theme_package.rs | 固定官方身份；发布复制清单文件；按路径+文件字节计算digest；app依赖以行扫描找themes/path | digest未显式纳入完整语义清单；扫描漏声明/别名/格式有潜在静默None风险，须T-01以真实pac解析合同核实，不能将合法声明误当无包 |
| run_vue.rs / vue.rs | run_vue预校验；Vue写.theme-inputs.json | receipt写失败仅println警告；独立生成/更新等所有调用链准入时机须核，不能只修run路径 |
| rust_ui.rs::front_input_hashes | pac、front .at、sidecar内容进入910 inputs | 未显式登记外部主题包闭包；record读失败return跳过。须区分必需主题输入错误与确实未消费，不借“无改App源码”复用旧主题 |
| Man man-install.py::stage_sources及verify | Full combined SDK，已有事务/来源清单；复制blueprints/widgets | 当前未复制UI packages/themes；不能称已有SDK发行接线 |
| UI rust_ui.rs交付路径 | DELIVERY_*、delivery-manifest、verify.py、独占staging/rollback | 新主题资产需以UI owner与实际依赖进入闭包/清单，不能作为未登记附带文件 |

以上是静态观察/风险，不是本轮运行失败。现摘要规则与目标差额须由正控/负控实证；不能只扩大摘要而不让真正复用消费者读取它。

### 4.3 与在途及后继的边界

- **OS053**当前executing，登记3/10；日志已推进calc固定vendor快照，来源仍为UI正式0.2.0。它拥有宿主policy/transport/follow/override/重连/真实OS事件和calc业务/正式pin。本包不要求其停工或改合同，不改calc/OS gitlink；固定合法旧快照继续可用。T-01读回最新交付或合同后给来源/安装路径/digest兼容表，共享UI文件/Man工具/锁短窗单写，不借其WIP。若接口实变才给具名适配交接；不能把新渠道追加入053原必需验收。
- **语言918**drafting0/9，拥有D-910-1执行ID消费者和D-906-3边界/基线。本包不编辑其规则/基线或修execution_ids；实际manifest/source owner变化交918或当时审计owner按真实来源接续。918不是本包整项硬前置；如必须消费的控制入口因旧缺陷无法验证，具名报告并协调独立修复，不绕过准入伪PASS。
- **B2**十二目录为011/012/013/014/015/016/017/021/022/023/028/038，尚未立约。本包交固定包来源、入口支持表、receipt/错误/移根及升级合同；B2自身持主题映射、业务、控件和原运行轨验收。可并行盘点/设计，采用本包新渠道的最终验收等本包正式交付；已交付path模式可独立使用，不把本包设成全部B2准备工作的统一前置。013生成阻点归对应consumer，不在此顺手修全App。
- Web只在现闭包真实需要时作为固定只读producer，不改Web发行算法/217分母；Man只改主题安装/校验触面，不接管CLI派发。更大State/Feature/Reload、B3及其余产品继续独立。

## 5. 详细设计

### 5.1 T-01有界冻结与执行入口

依赖031/033正式交付已满足；**本包实施授权尚未给出**。授权后唯一组拟`D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang,auto-web,auto-down}`，UI分支`codex/plan-036-theme-delivery`；参与写仓各一树/分支，未改仓固定detached来源。先核各仓规范/HEAD/WIP/实际包与所有调用链；不建链接，不借主检出写代码。独立review在同组同树、冻结写入后完成，不复制第二实现树。

T-01冻结：source/Full SDK/delivery实际入口与路径、显式path优先级、无path受管包解析方式、theme身份投影、manifest语义digest/旧receipt兼容、VM与Rust现支持轨、准确验证命令/预算和资源窗。优先用现pac AST/已解析依赖，不另造文本包解析器。明确声明但缺path/错声明/不支持来源须报错，合法未声明保持零UI资产；与themes无关的同名子串不可误命中。

冻结产物折进本Plan或统一材料根，列producer→表示→consumer和真实可执行反控。若现支持轨需要新语言语义/后端才能满足目标，先精确重规划，不静默删轨、改支持承诺或以仅解析成功代替运行。

### 5.1.1 T-01 冻结记录（2026-11-03 执行轮）

**组与基线**：唯一组 `D:/autostack/.wt/ui-036/`；auto-ui `codex/plan-036-theme-delivery`@e6ef5381（master）、auto-man `codex/plan-036-man-delivery`@47d8f7b、auto-lang `codex/plan-036-lang-delivery`@ec3e70414（lang master已推进922文档；`generated_build.rs`哈希1782b9a8…与立约观察一致）；auto-web@3de5524、auto-down@7c4d5cb 为固定detached依赖树（UI workspace `../auto-web` 路径依赖所需，首跑已实证缺树即manifest加载失败；两仓不改写）。材料scope `D:/autostack/plan-artifacts/plan-036/auto-ui/` 已建。实施授权来自 `/goal 完成计划036`（2026-11-03），覆盖work/review/有界修复/merge全链。

**状态校正（T-01读回）**：OS053已archived 10/10（`auto-os/docs/plans/archive/053-*.md`）——calc `vendor/themes` 固定快照 digest `9fa07742dc68aa53…bef838e`，宿主config策略 `theme_family_version`+`theme_family_digest` 双键不符整候选拒绝，env双源 `AUTOOS_THEME_PACKAGE_DIR`→`AUTO_UI_ROOT/packages/themes`，升级交接=「UI发新快照替换+calc重验」。918已archived 9/9，其canonical `delivery-admission-and-boundary-policy.md` 生效：§1.1 receipt新增字段归生产owner仓；§2 新cargo依赖边需具名facility admission（本包主题为数据文件、不改固定依赖集，无新边→无需新rule）；§3 本包落地后modularity audit baseline stale由变更owner重采样（merge交接记录）；§4 明示031渠道独立继续。

**冻结事实（当前树）**：
- 包：`packages/themes` 单源；themes 0.2.0 schema 1；manifest `files[]`=[pac.at, themes.at]（不含README）；families Auto/Solarized（light/dark，default light）；`module`/`default`字段现未被compiler verify消费。
- digest：`theme_package_content_digest` = manifest-listed files sorted `path\0bytes` concat sha256，**不含theme-package.json自身**（能力元数据变化不失效，V-05目标）。**兼容约束**：算法不可变（OS053固定快照+宿主策略绑定现值）；身份扩展用独立additive字段，旧receipt缺字段→不fresh→安全重验（`FrontReceipt::read`已容错：缺inputs→stale）。
- 准入：`verify_app_theme_inputs` 未声明→Ok(None)、声明非法→具名Err（C-11）；`app_theme_dep_path` 为pac.at行扫描（`dep`行含`themes`+括号深度+首个`path:`），近似名/注释/别名误命中风险须T-02以真实用例收紧。
- Vue：准入gate仅`run_vue_flow` L123-137；receipt写点 run L281、scaffold L5270/L5330、regen L5506；**`VueProject::generate()` L5005 fresh路径不写receipt（新发现缺口）**；写失败=警告继续（vue.rs L5376，须改为消费失败）。
- VM：`run_vm.rs` 零theme-package接入；主题经design block decl链（`theme_decl_lite`→env `AUTO_UI_TOOL_VM_THEME_DECL`→compose）。
- Rust：`front_input_hashes` L6362-6390（pac.at+front .at+sidecar；**读失败静默跳过**——必需主题输入须改为具名失败或冻结快照）；`write_front_receipt_with_inputs` 冻结不一致→receipt withheld；无theme闭包登记。
- Delivery：`rust_ui.rs` L3936-5903 staging/swap/manifest `generated-ui-delivery/1`；verify.py只验`assets/`资源；零theme处理。bundle为独立workspace（build.rs读`../../crates`+根lock），主题值生成期烘焙进交付源码→delivery需staging快照+manifest身份记录（additive，UI owner schema）。
- Man：`stage_sources` L248-251 full-only blueprints/widgets复制=扩展点（`UI = GROUP/"auto-ui"` sibling）；`manifest_files` 全树hash扫描+verify diff=验证免费；core forbidden `sdk/crates/auto-ui*` 不与 `sdk/packages/themes` 冲突；事务fail-point+旧安装保全（6/6 tests OK@47d8f7b）。
- lang：`FrontReceipt.inputs` BTreeMap防读容错；delivery经 `ClientBody.GenerateUi.delivery_root` 转发UI `tool_ops`；lang侧无代码改动需求（inputs加键）；`driver.py` 主题腿插V02后（镜像V03结构）。

**支持轨（Windows现支持）**：Vue generate/run/build+receipt；VM源码run（decl链+包验证接入点）；Rust dev build（front receipt）+delivery（`--delivery-root`）。不含Linux/通用包管理/公网registry/公网签名。

**命令冻结**：UI `cargo test -p auto-ui-compiler --test theme_package_verify --test theme_package_manifest`、`cargo test -p auto-ui-tools --test theme_package_input`；Man `python scripts/test_install_transaction.py`、`python scripts/test-man-install-compat.py --source <install> --scratch <new>`、`python scripts/man-install.py install|verify --profile full --install-root <scope>`；delivery `auto gen --delivery-root <scope>`（`AUTO_DELIVERY_SOURCE_ROOT`=lang树；driver.py模式）；`python test/fixtures/modularity/generated-ui-consumer/driver.py [--scratch <dir>]`（scratch必须在worktree外）。Vue E2E `tests/test_001_vue.mjs`/`test_006_vue.mjs`/`test_010_vue.mjs`（node/vite，V-08按需）。

**共享写窗/锁**：本会话唯一写者（ui-036三树）；910 `.generation.lock`+`TargetLease` 语义不触碰；UI快照发布零写失败保护、Man staging marker+进程存活判定、delivery StagingGuard/swap 沿用；主检出零代码写入（他方WIP不碰，落地前须其owner自行解决）。

**预算/资源**：cargo冷构建 compiler+tools 数分钟级；driver.py完整delivery构建较重（T-05执行）；无GPU/端口特殊需求。

### 5.2 来源、发行与安装

- 保`packages/themes`唯一编辑源；受管快照记录owner/package/version/schema、规范化相对文件集合/逐文件hash、族/mode/default等有效元数据，绑定所选UI producer；禁止从同名主检出兜底。
- 提议Full SDK落`sdk/packages/themes/`，delivery落其自身受管包目录（拟`packages/themes/`，T-01按真实解析锚确认）；显式合法App path仍优先。外部App可使用已冻结受管包来源，无需手工copy；生成后的解析相对各消费根，安装到B根仍可用。
- Core不携主题和UI tool、无UI反向依赖。Full安装清单与delivery verifier包含主题资产和来源；遗漏/多件/篡改/错版本/坏能力清单必须具名拒绝。沿现许可/来源记录，不隐含远端发布。
- 替换先验证staging完整候选再提交；坏源、复制失败、验证失败和换装失败保旧可用字节或完成恢复，并给准确残留/恢复记录；不覆盖未知非空目录、不删他方/活动路径。单快照发布原语也不能破坏已有合法目标。

### 5.3 生成输入与复用

- UI维护统一已验证主题输入描述，既能给Vue主题receipt，也能加入Rust `front_input_hashes`/`FrontReceipt.inputs`；必要时generation-core仅承载中立键值，禁止UI类型/实现下沉。
- 有效身份包含包owner/name/version/schema、有效能力元数据、相对文件集合及内容摘要。清单字段顺序/空白等语义等价变化可规范化；能力、文件集合或身份改变必然重验并影响适用性/失效。保兼容旧0.2.0包，合法旧快照继续读；旧不完整receipt首次安全重验/重建，不以缺字段判fresh，不无条件删旧产物。
- Vue generate/run/update、VM源码run、Rust gen/build/run/discovery复用按实际消费链前置校验；源码VM不强造Cargo产物receipt，但须证明其所读包与本次输入一致。receipt写失败为消费失败，不打印警告后成功；未声明官方包的App保持旧行为。
- 同mtime合法内容变化与能力变化触发真实重新生成/消费；篡改失败不复用旧exe冒新结果。冻结输入→生成/构建→发布/receipt须防并发改源，变更检测拒签或使用验证过的冻结快照。失败后合法恢复能再次真实消费。
- 910项目锚、同basename隔离、ctx/flags/target、可写worktree隔离/lease/清理保持；不把所有包摘要硬塞context导致无关第三方依赖重复编译。是否需要改变schema须有旧数据迁移/不误复用控制。

### 5.4 真实消费者与交接出口

使用033已迁**001和006**作为轻量静态与交互消费者：001族/模式实际值；006保GetStarted/选择族与accent原handler及状态。010作原输入/Submit兼容控制，不修改这三个示例业务。外部fixture为新建测试输入，UI持资产主题真值；真实安装工具、生成产物和实际运行共同验收，不能只调用快照helper。

交OS053：包版本/来源与摘要变化的兼容说明、旧vendor快照接受、未来可由安装渠道取得的路径/选择规则与升级拒绝恢复；不宣称053真实系统事件通过。交B2：可用入口/限制、单包声明、生成输入/移根/升级验收模板；不以fixture算24例已迁。交918：实际触及文件/manifest/owner输入SHA及审计影响，保缺陷owner边界。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/ui/official-theme-package.md | helper+未接渠道→实际Full SDK/delivery来源、兼容/拒绝和完整身份 | 关闭F-05有限范围 | AC-01/02/04 |
| SD-02 | add | docs/specs/build/official-theme-delivery-and-inputs.md | 新增各入口准入、内容复用/失败发布与旧receipt迁移支持表 | 统一真实consumer规则 | AC-01..05 |
| SD-03 | modify | 语言docs/specs/repository-boundaries/generated-ui-distribution.md；generated-cargo-build-lifecycle.md仅真实差额 | 主题入交付资产/输入；中立receipt变化如确需才沉淀 | 保910及独立消费 | AC-02..05 |
| SD-04 | modify | Man docs/installation.md；docs/specs/modules/generated-cargo-context.md仅真实差额 | Full携主题、Core零UI、安装来源/升级拒绝读回 | 安装owner唯一 | AC-02/04/05 |
| SD-05 | modify | docs/specs/ui/theme-app-migration.md | B1十例接受，B2/OS/918获得实际渠道与支持表，未迁项仍在 | 不扩历史交付 | AC-06 |

参与仓路径为对应仓相对路径；不提前写canonical，本Plan前置metadata仅本仓目标。没有真实差额的拟Spec行明确撤销并说明，不凭规划编造现状。

## 6. 测试设计

以下全部**未执行**。T-01固化真实CLI支持参数与fixture，T-02..06让新增控制可运行；命令从组树执行、明确输出与target落统一scope，不能用默认临时目录泄漏。实现/内部自查归work；未指定特定agent/model；独立上下文final review归review，同树冻结，review不修代码。以下覆盖限定Windows当前支持的Vue/VM/Rust入口，不外推Linux或通用包管理。

| Case ID | AC / 不变量与输入 | 实际入口/区分性oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01；合法现pac/path及受管SDK来源；无themes、别名/格式、缺/坏声明 | 实际依赖解析与run/gen/build准入；合法按所选来源，未声明零变化，声明非法非None且非0，近似名称不误匹配；T-01冻结支持语法 | work |
| V-02 | AC-02；Full安装/verify及Core反控 | `python scripts/man-install.py install/verify --profile full --install-root <scope>`及core；Full包逐件hash与源匹配，Core无UI/theme；缺件/多件/坏manifest拒绝 | work |
| V-03 | AC-02/05；实际delivery→第二绝对根、剥源覆盖及开发根可见性 | `auto gen --delivery-root <scope>`及随包verify.py，fresh后`cargo build --locked`；从B根真实运行的主题/业务值正确，查无开发root依赖，不能只列文件 | work |
| V-04 | AC-03；同版本/同长度/同mtime改token并更新hash，对照不变 | Vue与Rust受管复用入口、VM run；实际颜色/生成代码与receipt变成新包，不变输入不无谓改写；不是只assert摘要不同 | work |
| V-05 | AC-01/03；仅能力/default/文件集合改变、坏schema/身份/模式 | 语义元数据变化进入身份或拒绝；旧receipt缺主题项不fresh；现合法0.2.0和OS053固定快照保持可消费，语义等价清单可稳定 | work |
| V-06 | AC-03/04；篡改、读失败、receipt目标不可写、使用中换包 | 各入口非0+具名诊断；无新成功receipt/旧exe冒新，生成输入与实际字节一致；恢复合法后真实重跑成功 | work |
| V-07 | AC-04；已有可用安装/快照/delivery，prepare/copy/verify/swap故障 | 现Man事务故障点及UI发布入口，旧文件hash和旧运行结果保留或恢复；未知非空/活动staging不误删，准确残留 | work |
| V-08 | AC-05；001/006真实Vue与VM，Rust产物；010兼容控制 | 族×明暗代表往返，006原handler/输出/选择保持，010三值/Submit不变；读取真实CSS/VM状态及Rust执行产物，记录运行支持边界 | work |
| V-09 | AC-05；安装A→B，旧根不可见、离线锁定包；两App/同basename | receipt包内容身份稳定、项目/member按910各自重定位，target可写隔离；未偷偷复用原target或主检出；只承诺包已备齐离线，不冒Cargo冷依赖全离线 | work |
| V-10 | AC-06；OS053/918当前合同与B2清单、旧来源控制 | 固定版本/路径/摘要/实际触面交接可读；不改053必需门/产品pin，不改918基线，24例仍标未迁；真实交付后按新HEAD回读 | work |
| V-11 | AC-07；全部AC/负控/实际调用/未解差额/SD | 同组冻结提交独立final review，逐项实证，不以作者摘要或测试数代替；未闭合同内差额回work | review |
| V-12 | AC-08/09；冻结职责并集/各仓落地/清理 | 被测来源等于落地来源，canonical/路线/归档可读；guard与全输出对账/材料删除或具名持有 | merge |

现有锚：UI `cargo test -p auto-ui-tools --test theme_package_input`、compiler `theme_package_manifest/theme_package_verify`目标及Vue相关受影响目标；新增端到端控制在T-01选择现测试入口落点。语言`test/fixtures/modularity/generated-ui-consumer/driver.py`只扩本包消费腿，不复制另一生成器；Man `python scripts/test_install_transaction.py`与安装兼容脚本保原正负控。CLI参数先由实际help核，不把未核参数写成已执行。

Rust实际修改按UI原`t`、涉及生成的`tu`单进程及必要`ti`职责；新增定向测试先迭代，final review后merge在冻结源跑必要并集一次，既有同输入有效证据可复用。若语言generation-core真改，跑`cargo t auto-generation-core`；治理脚本真改才补其真实固定组，如`infra-fixed`。Man保受影响安装/工具和CLI消费者门。无全四仓发行整档/ACE/AG全量/语言cargo t all自动前置。首次红、环境重放绿、条件排除和未执行分别记；原分母/oracle不减，列准备/编译/运行/清理时间与真实执行数。

## 7. 验收标准

| ID | 可观察验收及对应验证 | owner_stage |
|---|---|---|
| AC-01 | 全部承诺入口采用一致官方包来源/身份校验；声明非法不能变None或警告成功，旧合法/无包控制保持，V-01/05/06 | work |
| AC-02 | 实际Full SDK与delivery带完整受管主题快照、来源/校验及外部获取；Core仍零UI；无手动拷包/开发根兜底，V-02/03 | work |
| AC-03 | 内容/能力/集合进入实际生成复用身份；同mtime变化真实生效、不变稳定，旧receipt安全迁移，并发改源不误签，V-04..06 | work |
| AC-04 | 发布/升级故障保旧可用集合，必需I/O失败明确失败，恢复可再次真实消费、未知/活动目录不误删，V-06/07 | work |
| AC-05 | 外部fresh/locked/第二根与001/006/010实际消费者覆盖承诺轨，910隔离/lease/锁不退化，V-03/08/09 | work |
| AC-06 | OS053/918/B2的来源、接口、差额、未迁与启动门交接准确；无借WIP/越权改合同/冒全产品通过，V-10 | work |
| AC-07 | 冻结源的独立final review逐AC/遗漏/workaround/健康/SD pass，V-11 | review |
| AC-08 | 冻结必要职责门及实际各仓落地、canonical、路线/归档读回，被测==交付来源，V-12 | merge |
| AC-09 | 输出全路径/进程/注册对账、材料摘要折Plan后删自有scope和树；guard clean，具名持有条件齐，V-12 | merge |

## 8. 执行步骤

| ID | 动作/关闭结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 授权后建唯一组；读031/033/053/918最新源，冻结调用链/支持/解析与摘要兼容、外部命令/资源和共享写窗，有界整链可行性控制；决策写本Plan或统一scope [✅ 已完成 2026-11-03] 冻结记录见§5.1.1：ui-036五树（三写auto-ui@e6ef5381/auto-man@47d8f7b/auto-lang@ec3e70414+两detached web@3de5524/down@7c4d5cb）；053/918读回均archived（053固定快照digest 9fa07742…约束digest算法兼容）；新发现：VueProject::generate() fresh路径不写receipt、publish快照对非空目标静默合并、dep行扫描近似名误命中；基线探测 UI compiler 1+2 passed / tools 8 passed、Man 6 OK 全绿 | 实施授权=`/goal 完成计划036`；AC-01..06 | work |
| T-02 | UI统一已验证输入描述、manifest语义身份与快照安全发布；原合法/错声明/失败保旧反控可执行 [✅ 已完成 2026-11-03] commit d566a783：compiler `theme_package_identity`（语义身份=身份+能力元数据规范化，与冻结内容摘要分离）；tools `ThemePackageInputIdentity`/`verified_theme_input_identity`/`read_theme_inputs_identity`；`write_theme_inputs_receipt`→`Result<Option<_>>`（未声明=None零写入旧行为；声明非法/写失败=具名Err）；vue.rs wrapper与run_vue/scaffold/regen四处调用改错误传播（警告后继续已消除）；dep行扫描恰名收紧；publish拒绝非空目标。tests 12/12（含4新增：identity规范化+能力敏感、dep恰名、非空拒绝、旧receipt无identity不当fresh） | T-01；AC-01/03/04，V-01/05/07 | work |
| T-03 | Man Full SDK主题装配/来源verify与UI delivery闭包接线，Core反控、原事务/外部拒绝和恢复 [✅ 已完成 2026-11-03] UI commit 029362e1：delivery staging官方主题快照至`packages/themes/`（publish原语+快照复验，失败旧交付不动）、manifest新增`theme_package`节（additive，UI owner schema）、verify.py重算逐文件sha256+content digest+身份对账+未登记文件；未声明=无theme节旧行为。Man commit 155191c：full staging `sdk/packages/themes`+python自描述清单核验（身份/逐文件sha256/未登记文件具名拒绝，事务保旧）、root_inputs登记、full必需+core具名拒绝反控、installation.md更新。测试：UI 12+1+2全绿；Man 9/9 OK（6原有事务+3新增主题staging/profile反控）。实际Full安装/delivery端到端V-02/03留T-05并集执行 | T-02；AC-02/04，V-02/03/07 | work |
| T-04 | Vue/VM/Rust各入口准入和真实input/receipt复用接线，必要中立语言扩展；必需I/O/并发换源拒签及旧receipt安全迁移 [✅ 已完成 2026-11-03] commit fe2da50a：全部Vue入口（run_vue_flow/prepare_vue_sources build轨/tool_ops generate_ui vue轨/fresh generate()）共享`admit_generation_theme_inputs`具名准入；VM源码run轨同律准入（不强造产物receipt）；Rust轨`front_input_hashes`→fallible并登记`theme/content`+`theme/identity`统一身份闭包（读/验失败具名Err不签reduced身份；旧receipt缺键→inputs_match不成立→安全重建；冻结漂移→receipt withheld沿用）；generate()与tool_ops自愈路径补写receipt。语言侧零代码改动（T-01冻结成立：inputs加键+ClientBody转发即可）。新增7测（rust_ui主题闭包3态+vue fresh receipt/篡改具名失败/未声明零receipt）全过；lib单线程197 passed/6 failed——6红在base e6ef5381逐一复现同败，归因既有红非本包引入 [↻ R-01 reopened 2026-11-03 final review]：准入声明语法（恰名行扫描）窄于模块系统真实消费语法（权威parser接受引号`dep "themes"`与任意名dep经dep_roots名无关消费）——`dep themesX/\"themes\"`声明非法/恶意包仍被静默消费（exit0+包值CSS、零receipt零准入），违反AC-01；修正方向=按§5.1自约改用已解析dep集（parse_document/dep_roots）做声明判定：任何dep其解析目录含theme-package.json即视为已声明（校验通过或具名Err）；单行`dep themes {…}`同修（R-02）；Vue轨补生成期合法换包冻结-比对（R-03，对齐AC-03并发不误签）[✅ 修复完成 2026-11-03] commit 3ca871bc（7 files +319/−109，含SD-01/02措辞同步）：`parsed_theme_declaration`=权威解析面（`auto_project_model::parse_file` 遍历全部dep，名无关、Bare/Quoted/单行/多行同权；任何dep解析目录含theme-package.json=声明→verify通过即消费/失败具名Err/多声明歧义Err；未声明Ok(None)旧行为）；行扫描`app_theme_dep_path`删除0残留；`VueProject.frozen_theme_identity`准入注入+receipt写前比对（漂移→具名置扣Err）。测试（环境commit耗尽解除后执行）：`cargo test -p auto-ui-tools --test theme_package_input`=13 passed/0 failed exit0（含2新R-01/R-02测+11既有正负控）；vue lib `fresh_generate_rejects_evil_package_via_near_name_dep`/`receipt_withheld_when_package_changes_after_admission_freeze` 各1 passed；compiler目标证据复用（本轮零compiler改动） | T-02；AC-01/03/04，V-01/04..06 | work |
| T-05 | 实际安装工具和外部001/006/010、fresh/locked/移根、正确失效/拒绝恢复、910隔离完整并集 [✅ 已完成 2026-11-03] 材料scope plan-036/{auto-man,auto-ui,auto-lang}/登记。**V-02**：real Full install 7871 files verify exit0（sources含auto-ui@fe2da50a，`sdk/packages/themes/`四件在位）+Core 960 files零`sdk/packages`（UI零资产反控）；OS053探针calc快照与canonical逐位一致digest=9fa07742…。**V-07**：real安装fail-point prepare→exit2、旧安装verify绿、零staging残留。**V-01/06**：installed tool篡改→exit1具名`主题包生成准入失败`、恢复后exit0。**V-08**：installed tool真实gen 001/006/010（receipt digest=9fa07742…；001 Auto/light `--background: 0 0% 100%`+.dark块、006 GetStarted/SetTheme/SetAccent、010三值+Submit在生成物中）；VM会话真实启动（`entering vm interpreter`）+VM篡改vendored副本exit1具名准入失败（会话启动前）；既有red归因见T-04。**V-03**：driver.py V06腿全过（交付生成theme_package节digest重算一致+verify.py绿+B根locked构建160s+篡改具名红）；V01/V02/V03基线腿全绿（143s/148s真实构建+全负控）。**V-04**：rust managed轨道5腿全过（receipt含theme/content=9fa07742…+theme/identity；unchanged 0.93s零重生成；mtime保持合法突变→`Regenerating`+receipt新digest=重算精确一致；篡改exit1具名sha256不符；canonical恢复→再消费PASS；旁证：语义损坏被主题数据解析具名拒绝`未知公共颜色键`）。**V-09**：同basename两App共享workspace隔离（17a76e40/0bfdd2d8不同id8、各自receipt同包身份）。既有红/边界具名记录：①fresh-scaffold index.html缺`__AUTO_TREE_THEMES__` bootstrap（regen/run路径有，既有缺口，owner=vue生成器后继）；②已安装SDK缺`crates/auto-web`→installed-toolchain直接delivery生成producer预检具名红（既有闭包形态，owner=Man 816/832）；③driver V04两腿stale于910 id8命名（期望`dev-run`实际`dev-run-b78955e5`；反证探针dev/交付main.rs diff=0、workspace继承形状成立；owner=lang 792-driver）；④driver V05a包内tool构建红（auto-ui-runtime highlight.rs/autodown_render.rs引用autodown_core vs delivery投影——e6ef5381基线既有，owner=815交付面）；⑤sccache机器事实（RUSTC_WRAPPER致0xc0000409，沿driver收据记录）。scratch在plan-artifacts，主检出零触碰 | T-03/04；AC-01..05，V-01..09 | work |
| T-06 | 健康/遗漏/原红归因、自查全部work AC与SD草稿，按最终053/918状态冻结交接/B2模板和来源 [✅ 已完成 2026-11-03] **AC自查**：AC-01✓（全入口准入+恰名解析+具名拒绝；installed/VM/vendored三轨篡改具名红实证）；AC-02✓（Full 7871件携themes+Core 960件零UI+delivery theme_package节+verify.py；外部fixture只依赖已安装快照）；AC-03✓（同mtime突变重生成+receipt新digest精确+能力变化身份失效单测+旧receipt缺identity不当fresh）；AC-04✓（fail-point保旧+零残留+非空拒绝+恢复再消费+receipt-unwritable具名）；AC-05✓（driver B根locked 160s+三Vue消费者真实gen+VM会话+同basename隔离+910形状反证）；AC-06✓（053 archived digest逐位一致、918 §1.1加法/§2无新边/§3重采样交接、B2模板=新spec §1/§5、未迁24例保持）。**SD草稿落树**：UI commit（SD-01 official-theme-package渠道/身份/全入口修订；SD-02 新建build/official-theme-delivery-and-inputs.md；SD-05 migration交接表）；lang commit 8dfd03f23（SD-03 distribution bundle布局+lifecycle front-inputs主题键；driver V06腿）；SD-04已随T-03落地（installation.md）。**原红归因**（详见T-05行）：6个tools lib红@base逐一复现同败；driver V04×2/V05a既有stale/投影缺口均具名owner。**合并期义务**（不阻塞work完成）：modularity audit baseline按918 §3由变更owner重采样；被测==交付source读回归merge。工作owned任务全绿、无open in-contract finding，execution_done [↻ R-5 reopened 2026-11-03 bounded re-review]：delivery spec L15（§1渠道表App消费行校验列）残留`dep 名恰为 \`themes\``，与同文件§2（声明判定不依赖dep名）及实现矛盾——单行改写为名无关声明规则；R-6（info非门槛）tool_ops tauri arm无准入/无freeze为@base 95fc28c3同形既有观察、AC-01未列tauri入口，记具名非阻塞后继跟进 [✅ R-5已修 2026-11-03] commit 57fb23f3（2 spec files +6/−6）：L15改写名无关声明规则（任何dep解析目录含theme-package.json即声明、按§2校验；`dep themes { path: <受管快照> }`保留为推荐形式）；ui spec §3未声明/§5声明两处恰名速记润色；测试计数锚12→13测（修复后实际）；全文件术语grep收口（UI两SD草稿仅剩废弃说明段，lang SD-03零命中）；R-6保持具名非阻塞既有观察；无open in-contract finding，execution_done | T-05；AC-01..06，V-10；execution_done6/10 | work |
| T-07 | 同组同树、独立上下文final review，finding回work且相应证据失效 [✅ 已完成 2026-11-03] final review#1 needs_fix（R-01 major/R-02/R-03，AC-02..06 pass独立重跑）→修复3ca871bc→bounded re-review#2 needs_fix（R-5 spec残留行；R-6 info具名既有tauri观察）→修复57fb23f3→bounded re-review#3 **pass**（R-01/R-02/R-03端到端独立反例+正控digest逐位一致；R-5闭合grep收口；三树clean冻结@57fb23f3/2bbd9542/8dfd03f23） | T-06；AC-07，V-11 | review |
| T-08 | 冻结受影响门并集、参与仓依赖序合入，跨仓被测/落地source精确读回，不代OS/产品发布 [✅ 已完成 2026-11-03] **冻结门并集@三交付树全绿**：UI compiler theme_package_verify(2)+manifest(1)、tools theme_package_input(13)、lib plan036_fresh_generate_receipt_tests(5)+rust_ui theme_input_entries_follow_identity_and_reject_tamper(1)；Man test_install_transaction.py 9/9；lang driver全union 28 PASS（**V06主题腿4/4全过**：manifest digest 9fa07742…重算逐位一致+verify.py绿+B根locked构建171s+篡改具名红）+3具名既有红（V04×2=910 id8命名stale owner=lang 792-driver；V05a=auto-ui-runtime delivery投影 owner=815交付面——与T-05/T-06归因逐一相同，零新红）。**合入（依赖序 lang→Man→UI，全ff-only零merge commit）**：lang 8dfd03f23→rebase master(2f2114a65f基)→**2f558fc93**（range-diff `=`）；Man 2bbd9542→master==base→**2bbd954**（直ff）；UI 57fb23f3→rebase master(cb356bdb基)→**e06b65b2**（6/6 range-diff `=`，映射 d566a783→92ff8975/029362e1→212f2932/fe2da50a→ff336628/95fc28c3→46bf8631/3ca871bc→7c6f9713/57fb23f3→e06b65b2）。三主检出tip==交付commit实证；主检出他方WIP零触碰零包含。**环境机器事实**：本机RUSTC_WRAPPER=机器级sccache——python子进程spawn的cargo在RUSTC_WRAPPER**缺失**时rustc probe 0xc0000142、**置空串**即正常；merge用plan-artifacts cargo.exe shim（set空串后转发）使driver全腿可跑，零仓库代码改动 | T-07 pass；AC-08，V-12 | merge |
| T-09 | canonical/消费者交接与路线/具名差额实证刷新，归档（清理未完不得冒全交付） [✅ 已完成 2026-11-03] **canonical_verified**：UI master@e06b65b2三canonical与复审树hash逐位一致（official-theme-package=182e7938、build/official-theme-delivery-and-inputs=636ea7aa、theme-app-migration=87c7c3e9，git-tracked）；lang master@2f558fc93 generated-ui-distribution=807a8d28+lifecycle=951e7695；Man master@2bbd954 installation.md=cb37bf01。消费者交接（§5.4/SD-05）：OS053交接=digest算法冻结+新渠道并行（已实证053固定快照逐位可读）；918交接=§1.1加法字段/§2无新边/§3 baseline重采样归变更owner（merge交接义务已记录）；B2模板=新spec §1/§5；24例未迁保持。批量回归due-check：lang `.last-batch-regression.json` last=920@2026-11-03T01:01Z，036不被5整除且receipt<48h→**不due**。Plan归档docs/plans/archived/ | T-08；AC-08，V-12 | merge |
| T-10 | 全输出路径对账、必要摘要折Plan，删除统一scope/自有树产物，各树guard clean再移除及分支/空组；终态archived10/10 | T-09；AC-09，V-12 | merge |

材料唯一`D:/autostack/plan-artifacts/plan-036/auto-ui/`，参与仓按repo分区，创建即登记owner/用途/在途。外部SDK/源码副本/发行/探针/截图/日志/显式target皆在该scope；普通编译只在专属树。merge列全部命令实际输出路径，摘要含命令/退出码/关键值/内容hash折Plan后删整个自有scope及树产物；仅具名未闭持有有owner/理由/关闭条件。未知归属/活动进程先核查，不盲删、不清910旧持有、不占用053材料。无worktree纯文档临时检查用系统临时目录；本轮未建材料scope。

## 9. 复审记录

2026-11-03 / stage:new / auto-ui/PLAN-036 r1 / outcome:blocked（只规划，待明确实施授权） / 已核031/033归档及主线祖先、canonical仍有F-05/VM-Rust receipt边界、实际Man装配和UI front输入/receipt错误路径；授权仅立约/路线刷新。本轮静态预检见下，未实施/跑探针/项目门/安装/建树/发布。next：授权后work/T-01重核053/918来源与写窗，冻结完整入口/兼容与命令，再按T-02..06；不等待OS053或918整项交付。

2026-11-03 / stage: work | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass（T-01） | worktree: D:/autostack/.wt/ui-036/auto-ui | branch: codex/plan-036-theme-delivery | base_commit: e6ef5381 | code_commit: e6ef5381（本轮零代码改动） | dependency_revisions: auto-man@47d8f7b、auto-lang@ec3e70414、auto-web@3de5524(detached)、auto-down@7c4d5cb(detached) | task_ids: T-01 | evidence: 组五树建成+plan-artifacts/plan-036/auto-ui/；§5.1.1冻结（053 archived→digest算法兼容约束、918 archived→§1.1/§2/§3边界适用；新发现generate() fresh无receipt、publish非空静默合并、dep扫描近似名）；基线UI `cargo test -p auto-ui-compiler --test theme_package_verify --test theme_package_manifest`=3 passed、`cargo test -p auto-ui-tools --test theme_package_input`=8 passed、Man `python scripts/test_install_transaction.py`=6 OK，全exit0 | blockers: 无 | next_responsible_role: work（T-02）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: e6ef5381 | next: work（T-02；授权=goal`完成计划036`全链，后续轮次继承不再询问）

2026-11-03 / stage: work | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass（T-02..T-06 全链，execution_done） | worktree: D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang} | branch: codex/plan-036-{theme,man,lang}-delivery | base_commit: UI e6ef5381 / Man 47d8f7b / lang ec3e70414 | code_commit: UI 029362e1→fe2da50a→spec-drafts commit、Man 155191c→lock对账→verifier修正commit、lang 8dfd03f23 | dependency_revisions: auto-web@3de5524(detached)、auto-down@7c4d5cb(detached) | task_ids: T-02,T-03,T-04,T-05,T-06 | evidence: 逐任务行见§8（测试12/12+9/9+1+2+3+1全绿；real Full/Core安装verify；driver V01/V02/V03+V06全过含B根locked 160s；rust managed 5腿；VM正负腿；V-09隔离；OS053探针逐位一致）；SD-01..05草稿落三仓树 | blockers: 无open in-contract finding；5项具名既有红/边界+合并期义务已记录（T-05/T-06行），不阻塞review | next_responsible_role: review（T-07 同组同树冻结 final review）
work-handoff readiness：work-owned AC-01..06 全绿且绑定当前证据（§8 T-05/T-06行）；受影响模块定向测试在最终revision全过；统一回归门保持pending（owner=final review/merge按§6规则）；被测源==工作树源（ui-036组，主检出未触碰）；两连败诊断不触发。

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: UI 95fc28c3 / Man 2bbd9542 / lang 8dfd03f23 | next: work（修复R-01/R-02/R-03：解析器驱动声明判定+Vue轨冻结比对+SD措辞同步+端到端反例测试；修复写回同树后bounded re-review）

2026-11-03 / stage: work | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass（R-01/R-02/R-03 修复实现；测试执行遇环境阻塞，重试中） | worktree: D:/autostack/.wt/ui-036/auto-ui | branch: codex/plan-036-theme-delivery | base_commit: e6ef5381 | code_commit: 未提交（修复在工作树，验证后提交） | task_ids: T-04(修复),T-06(重查) | evidence: **R-01/R-02**：声明判定重写为权威解析面——`parsed_theme_declaration` 用 `auto_project_model::parse_file` 遍历全部 dep（名无关、Bare/Quoted/单行/多行同权）：任何 dep 目标目录含 theme-package.json = 声明（verify 通过→消费；失败→具名Err）；多声明=歧义Err；行扫描 `app_theme_dep_path`/恰名测试删除（0残留引用）。**R-03**：VueProject 增 `frozen_theme_identity` 字段+`set_frozen_theme_identity`；新增 lib `resolve_theme_input_identity`；run/prepare/tool_ops 三轨 gate 采集冻结身份注入；wrapper 写前比对，漂移→具名`主题包在生成期间发生变化…receipt 置扣`。**测试**：`declaration_is_parser_based_across_name_and_line_forms`（近名/引号/单行=声明；未声明=Ok(None)零写入；多声明=歧义Err）+ `evil_package_via_near_name_dep_is_named_rejected`（恶包owner=evil-corp经近名→FieldMismatch具名拒绝+receipt零写入）+ vue级 `fresh_generate_rejects_evil_package_via_near_name_dep` + `receipt_withheld_when_package_changes_after_admission_freeze`（冻结后合法换包→置扣Err）。**SD-01/SD-02** 措辞同步（声明判定=解析dep集；生成期换包置扣）。**编译**：lib+all-targets check 绿（`cargo check -p auto-ui-tools --all-targets -j 2` Finished）；**测试执行阻塞**：本机 commit 耗尽（error 1455 页面文件太小/0xc0000409 alloc fail）——他方会话 33 node 进程持压，含 sccache 损坏期 0xc0000142/0xc0000409 间歇；已用 debug=0/-j1/CARGO_INCREMENTAL=0/env-scrub 仍不足；与代码无关（lib check 绿、失败单元为随机小依赖crate）。精确解除动作=他方负载释放后重跑修复测试组→提交→bounded re-review | blockers: 环境commit耗尽（他方），重试循环已挂（5×7min） | next_responsible_role: work（测试落地后提交并交 bounded re-review）

2026-11-03 / stage: review | review_scope: final | reviewer_identity/limitations: 独立上下文=同会话委派fresh agent（非实现摘要复述，逐AC从代码/工件+自建反例重构判定；未重跑重型端到端安装/交付构建——按Plan收据+代码审计+digest常量独立重算采信；GUI未行使；其"T-05材料scope被prune"陈述经复核不实——v02-full/driver-scratch/fixtures均完好在案） | plan_id: PLAN-036 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang} | branch: codex/plan-036-{theme,man,lang}-delivery | reviewed_commit: UI 95fc28c3 / Man 2bbd9542 / lang 8dfd03f23（三树clean复核） | base_commit: UI e6ef5381 / Man 47d8f7b / lang ec3e70414 | dependency_revisions: auto-web@3de5524、auto-down@7c4d5cb | spec_inputs: SD-01..05草稿（95fc28c3/8dfd03f23内） | acceptance_results: AC-02/03/04/05/06 pass（独立重跑12+2+1+9+4测全绿；自建反例P1身份规范化、P2旧receipt、P3目录非空拒绝、P6具名拒绝全过；digest 9fa07742…独立重算逐位一致）；AC-01 partial fail（R-01端到端实证：`dep \"themes\"`/`dep themesX`→恶意包静默消费exit0+包值CSS+零receipt；单行声明R-02被静默忽略） | pending_closeout_gates: 统一回归归review/merge按§6 | findings: R-01(major,AC-01/T-04/T-02,准入语法<消费语法→按已解析dep集修) R-02(minor,单行声明静默忽略,同修) R-03(minor,Vue轨生成期合法换包缺冻结-比对,补齐) | evidence: review/scratch/反例工件+本记录；SD审计SD-01/02对R-01措辞需随修正同步（SD-03/04/05准确） | next: work（重开T-04/T-06，T-02解析器子项随之修订；修复后受影响范围bounded re-review）

静态预检：0–10章节齐，10T/9AC/12V/5SD单行身份唯一连续，任务覆盖各AC/SD、owner_stage分开；036跨活动/归档唯一、独占取号后.next-id=037。本Plan、设计接续及路线/战略当前段本地Markdown引用存在，无尾空格；纯文档检查不等实现测试/独立final review。路线/战略已刷v22，031/033终态及B1十例/余24、053实施/918草案与036仅规划分列。未修改053/918合同或canonical/旧JSON。

2026-11-03 / stage: work | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass（R-01/R-02/R-03 修复验证+提交，execution_done） | worktree: D:/autostack/.wt/ui-036/auto-ui | branch: codex/plan-036-theme-delivery | base_commit: e6ef5381 | code_commit: 3ca871bc | dependency_revisions: auto-man@2bbd9542、auto-lang@8dfd03f23、auto-web@3de5524(detached)、auto-down@7c4d5cb(detached) | task_ids: T-04(修复),T-06(重查) | evidence: 环境解除后重跑修复测试组（重试正当性=commit耗尽实际缓解：free_virt 0.83GB→48.68GB，node 26→5，他方cargo已退出）：`cargo test -p auto-ui-tools --test theme_package_input`=13 passed/0 failed exit0——R-01覆盖=`declaration_is_parser_based_across_name_and_line_forms`（近名/引号/单行=声明、未声明Ok(None)零写入、多声明歧义Err）+`evil_package_via_near_name_dep_is_named_rejected`（恶包owner=evil-corp经近名→具名拒绝+receipt零写入），R-02=同测试单行形态；11既有正负控（identity规范化/receipt随内容/非空拒绝/旧receipt不当fresh等）同轮全绿=回归控制；R-03覆盖=vue lib `fresh_generate_rejects_evil_package_via_near_name_dep`+`receipt_withheld_when_package_changes_after_admission_freeze`（冻结→换包→置扣具名Err）各1 passed exit0；compiler theme_package_verify/manifest证据复用（本轮零compiler改动）；commit含SD-01/02措辞同步，三树status clean复核（UI@3ca871bc/Man@2bbd9542/lang@8dfd03f23） | blockers: 无 | next_responsible_role: review（bounded re-review @3ca871bc，同组同树冻结）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: 3ca871bc | next: review（bounded re-review：R-01/R-02/R-03修复@3ca871bc+Man@2bbd9542+lang@8dfd03f23，同组同树）

2026-11-03 / stage: review | review_scope: final（bounded re-review of R-01/R-02/R-03） | reviewer_identity/limitations: 独立上下文=同会话委派fresh agent（逐finding自建反例+独立重跑受影响检查；AC-02..06保留前轮证据未重跑；run_ui/build_ui未live驱动——与已验证admit+freeze共享代码路径由B/C套件覆盖，limitation记录） | plan_id: PLAN-036 | plan_revision: r1 | outcome: needs_fix（R-5 minor；代码修复本体通过） | worktree_path: D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang} | branch: codex/plan-036-{theme,man,lang}-delivery | reviewed_commit: UI 3ca871bc / Man 2bbd9542 / lang 8dfd03f23（三树clean复核） | base_commit: UI e6ef5381 | dependency_revisions: auto-web@3de5524、auto-down@7c4d5cb | spec_inputs: SD-01/02@3ca871bc（检查E：除R-5一行外全过） | acceptance_results: 检查A=全receipt写点经admission+freeze（run/build/gen-only/generate_ui/fresh generate；VM准入无receipt by design；undeclared Ok(None)零写入不可绕过）；检查B=`cargo test -p auto-ui-tools --test theme_package_input`13/13 exit0+断言质量审计（具名FieldMismatch owner+零receipt，非仅错误存在；R-02单行形态在覆盖内）；检查C=vue lib plan036_fresh_generate_receipt_tests 5/5（2新+3既有；置扣断言具名`生成期间发生变化`）；检查D=独立端到端反例（review2-scratch恶包owner=evil-corp+quoted近名`dep \"themesX\"`→committed二进制具名拒绝`主题包生成准入失败…owner 不符：期望 auto-ui，实际 evil-corp`+零gen/零receipt/零CSS；正控`dep themes`→GenerateOk 18 files+receipt digest 9fa07742…逐位一致；进程exit 0为协议设计，结果随envelope）；检查F=`app_theme_dep_path` crates/ 0残留 | pending_closeout_gates: 统一回归归review/merge按§6；merge期义务（918 §3重采样、被测==交付读回） | findings: R-5(minor,SD同步,delivery spec L15残留`dep 名恰为 themes`与§2名无关规则及实现矛盾→单行改写；ui spec §3/§5速记措辞可同轮润色) R-6(info,非本复审门槛,既有观察:tool_ops tauri arm无准入/无freeze@base同形,AC-01入口清单未含tauri→后继计划具名跟进或spec明示排除) | evidence: review2-scratch恶包反例+wire探针工件+本记录 | next: work（T-06重开：R-5单行doc修复；R-6记具名非阻塞既有观察；修复后bounded re-review）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 5 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: 3ca871bc | next: work（R-5 delivery spec L15单行doc修复→提交→bounded re-review；repair cycle 2/3；两连败诊断义务=说明为何前轮SD同步漏§1渠道表行+改用全文件陈旧术语grep收口）

2026-11-03 / 两连败诊断checkpoint（review#1 R-01..03 → review#2 R-5，918 §9在案）：剩余不变量=SD草稿全文与实现的声明判定规则一致（无恰名残留）；前轮漏因=R-01修复的SD同步按重写小节范围核对§2、未开§1渠道表的压缩校验列（同规则以`dep 名恰为 \`themes\``措辞存放于表单元格），且代码侧`app_theme_dep_path` 0残留grep造成「措辞已同步」的假信心；受影响面=纯SD-01/02文案，零代码路径；oracle调整=措辞收口oracle从「重写小节阅读」改为全文件术语grep（恰为/恰名/dep 名+恰名速记复阅）跨UI两SD草稿+lang SD-03草稿，已随本轮修复执行（结果见下work receipt）。

2026-11-03 / stage: work | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass（R-5 修复，execution_done） | worktree: D:/autostack/.wt/ui-036/auto-ui | branch: codex/plan-036-theme-delivery | base_commit: e6ef5381 | code_commit: 57fb23f3 | dependency_revisions: auto-man@2bbd9542、auto-lang@8dfd03f23、auto-web@3de5524(detached)、auto-down@7c4d5cb(detached) | task_ids: T-06(重闭) | evidence: delivery spec §1渠道表App消费行校验列改写为名无关声明规则（任何dep解析目录含`theme-package.json`即声明、按§2校验——通过→消费、非法→具名拒绝；推荐形式`dep themes { path: <受管快照> }`保留）；ui spec §3未声明/§5声明两处恰名速记润色为名无关措辞；两SD草稿测试计数锚12→13测（与3ca871bc后实际13测一致）；收口grep=`grep -n "恰为\|恰名\|dep 名"` UI两SD草稿仅剩§2废弃说明段（正确用法）、lang SD-03两文件0命中；commit 57fb23f3后树clean | blockers: 无 | next_responsible_role: review（bounded re-review @57fb23f3，scope=SD文案一致性doc检查）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: 57fb23f3 | next: review（bounded re-review：R-5 doc修复@57fb23f3，同组同树；doc内容检查scope）

2026-11-03 / stage: review | review_scope: final（bounded re-review #3，scope=R-5 doc修复） | reviewer_identity/limitations: 主会话独立复核（doc修复按review skill允许的内容/diff检查执行；代码证据复用re-review #2@3ca871bc独立验证结果——本diff零代码改动，其证据不失效） | plan_id: PLAN-036 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang} | branch: codex/plan-036-{theme,man,lang}-delivery | reviewed_commit: UI 57fb23f3 / Man 2bbd9542 / lang 8dfd03f23（三树clean复核；UI diff 3ca871bc..57fb23f3=仅2 spec files +6/−6） | base_commit: UI e6ef5381 | dependency_revisions: auto-web@3de5524、auto-down@7c4d5cb | spec_inputs: SD-01/02@57fb23f3 | acceptance_results: R-5闭合——delivery spec L15=名无关声明规则（任何dep解析目录含theme-package.json即声明、按§2校验；推荐形式`dep themes { path: <受管快照> }`保留于接口列），与§2及实现（re-review #2端到端已证）一致；ui spec §3未声明/§5声明两处=名无关措辞+同节声明判定段可解析；测试计数锚两文件均=13测（与57fb23f3实际13测一致；描述改「声明判定（解析面、名无关）」准确）；`git grep 恰为|恰名|dep 名`@HEAD=仅§2废弃说明段+L15新规则自述（正确用法）；lang SD-03草稿0命中（前轮已证）；表格/列表结构完整无collateral | pending_closeout_gates: 统一回归=冻结受影响门并集归merge（§6：merge在冻结源跑必要并集一次）；merge期义务（918 §3重采样、被测==交付source读回、输出对账） | findings: 无（R-6保持具名非阻塞既有观察，T-06行在案） | evidence: git diff 3ca871bc..57fb23f3全文+git grep@HEAD+本记录 | next: merge（T-08 冻结受影响门并集合入→T-09 canonical/归档→T-10 清理，终态archived 10/10）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-036/auto-ui | commit: 57fb23f3 | next: merge（final pass@r1+57fb23f3/2bbd9542/8dfd03f23；T-08..T-10）

2026-11-03 / stage: merge | plan_id: PLAN-036:r1 | reviewer/operator: 主会话（auto-plan-merge skill全流程） | outcome: pass（delivered；cleanup见T-10/closeout commit） | checkpoints: prepared=三交付树clean@57fb23f3/2bbd9542/8dfd03f23+canonical delta=已复审SD-01..05；landed=lang 2f558fc93（range-diff `=`）/Man 2bbd954（base==master直ff）/UI e06b65b2（6/6 range-diff `=`），三主检出tip==交付commit、零merge commit、他方WIP零触碰；canonical_verified=UI 182e7938/636ea7aa/87c7c3e9+lang 807a8d28/951e7695+Man cb37bf01（git-tracked、与复审树逐位一致）；archived=docs/plans/archived/036-official-theme-distribution-and-input-closure.md（本commit） | frozen_gates: UI compiler3+tools13+lib6全绿、Man 9/9、lang driver 28 PASS含V06主题腿4/4+3具名既有红（V04×2 owner=lang 792-driver、V05a owner=815交付面，与T-05/T-06归因相同）；批量回归不due（last=920@2026-11-03、036非5倍数） | environment_notes: RUSTCWrapperEmptyMachineFact——python子进程spawn的cargo在RUSTC_WRAPPER缺失时rustc probe 0xc0000142（与本机机器级sccache相关），置空串即正常；merge以plan-artifacts/plan-036/auto-ui/cargo-shim/cargo.exe（置空串转发真cargo）承载driver全腿，零仓库改动 | evidence: driver log plan-artifacts/plan-036/auto-lang/merge-driver-run3.log（merge后随scope删除，本记录为durable摘要）、range-diff输出、canonical hash清单 | landing_checks: auto-ui无共享生产进程/发行二进制消费master（工具二进制按项目生成期构建）——观察项记录：无stale production artifact需重建（与PLAN-092三件套情形不适用） | next: closeout（wt-guard→树/分支删除→plan-artifacts删除→registry刷新→cleaned commit）

stage: goal | plan_id: PLAN-036 | plan_revision: r1 | round: 8 | from: merge | to: done | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-036/{auto-ui,auto-man,auto-lang}（cleanup执行中） | commit: UI e06b65b2 / Man 2bbd954 / lang 2f558fc93 | next: done（closeout后终态archived 10/10）

## 10. 待澄清事项

1. 本轮暂不实施；正式合同不自动接管OS053/918或其他会话。未来授权后等价内部选择不重复询问；若新增语义/外部支持/公开兼容或改owner范围，须提出具体差额。
2. SDK包路径/声明解析和digest兼容的具体字段T-01有界冻结；原0.2.0合法包与旧vendor仍支持，不用升版逃避兼容。缺失证据不能写已测，必需轨不可降格为仅静态解析。
3. 本包外部可搬动是已有源码/安装/应用交付渠道，不含公网registry/全冷依赖离线、Linux新支持或远端发布。真实窗口/端口/GPU预算T-01核，不操作用户系统主题。
4. 918旧治理缺陷、Web源暂存/锁、Man-d派发、UI既有红若实际阻断本包必需门，列复现/owner/精确等待或修订，不能以历史债直接充PASS，也不吞入无限修复。
5. OS053既有固定快照与本包新渠道并行可用；其最终交付后再核升级/来源交接。B2可先盘点，采用新渠道的最终门等U036；后继控件/业务/013生成问题独立立约。
