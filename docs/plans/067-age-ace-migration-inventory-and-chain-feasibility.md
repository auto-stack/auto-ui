---
plan_id: PLAN-067
status: execution_done
feature_name: AGE迁移ACE全消费者清单与最小整链先证
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 6
total_steps: 9
depends_on: []
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/ace-migration-feasibility.md]
touched_goals: [GOAL-008]
---

# PLAN-067：AGE迁移ACE全消费者清单与最小整链先证

## 0. 变更摘要

独立研究合同：完整追踪AGE旧引擎依赖，先证正常作者源→ACE→图形宿主→真实GPU→改源/错误恢复；输出go/bounded-go/no-go与可执行后继接口合同。不交付全量迁移，不切默认或disable。

## 1. 目标

形成准确迁移分母/所有权与最低可行链；真Auto业务、普通use/局部事件，不手写HIR或Rust状态替代。不认领ACE M1/M2签章、AGE M1-ready、完整Schema/UI融合或全Gallery已迁。

## 2. 架构方案

auto-ui主责研究，auto-lang/ACE与ag-examples只读；探针/适配器仅统一scope。复用已交统一源码前门/语义/AV/host机制，优先Schema投影；至多两种机制候选。无正式领域接口时给精确no-go，不在本Plan修改语言核心或偷偷使用legacy Parser。

拟唯一树`D:/autostack/.wt/ui-067/auto-ui`，分支`codex/plan-067-dev`；跨仓源码只读取实际交付，若需产品修复超出本仓则交原owner另约，不直接改其主检出。

## 3. 技术栈

旧链锚点driver `src/lib.rs::Session`/Cargo/.sdk-source，pilot `src/load.rs`/`lower_node_ui.rs`与Cargo，内容`tools/gallery_node_authoring.py`/锁及showcase包。ACE只读锚点`experimental/ace/crates/core/src/source/cst/`、`source-query/src/modules.rs`、`av-core/src/host.rs`、`av/src/resident.rs`（旧Spec中的probe路径已迁移，不能照旧路径执行）；是否可消费以T01实际交付确认。

## 4. 需求分析与背景调查

用户2026-11-06明确授权“AGE M1剩余收口工作和ACE迁移整链先证正式拆成计划，明确依赖、验收和执行顺序，先规划，不实施”。本合同只规划；没有执行授权、测试结果或新能力交付认领。主检出观察`auto-ui@02316271da9b9cd90750ca5a38ed91143f6c4be4`；路线输入为auto-ui `docs/design/autographics/age-three-milestones-and-ace-m2.md`磁盘稿，非已提交实现。新版M1沿原M2-01..10/G-M01..09全部义务，历史tag/号/归档原文保留。

已读语言source-frontdoor/finite-source-modules、shared-interface-and-scalar-bridge、resident-online-reload Specs与REP合同。旧Session真实依赖AutoVM/Parser。937公开lane无f32；AGE位置f32且bool/数组有旧边界。不能把f64桥强制cast当语义保真。ACE模块/Schema支持与普通源profile分别核；新941/938/939/944等状态开工重核，WIP不作依赖。

当前有限模块Spec来自PLAN-950，`ac.source.modules.scalar/v1`仅i32/bool顶层自由函数；无公共类型/资源、包目录发现/循环模块，模块与有限模板组合具名拒绝。因此“已有use模块”不等于Node/model/msg/on/float图形链已可用。T01优先核这些具体差额，T03必须用正常作者源实际触发准入/拒绝；不能把950的标量模块通过当本合同整链go。在线执行实路径现为`experimental/ace/crates/av/src/resident.rs`，历史Spec提到的probe路径只作来源。


只读输入指纹（SHA256，磁盘实读；工作前仍须核正式来源）：

- `auto-lang/docs/specs/auto-syntax/finite-source-modules.md`：`e0870a71b96fb6f375a59bdc986aa097f7b3415e3318cdb531d7b2a1909cd617`。
- `auto-lang/docs/specs/auto-abi/shared-interface-and-scalar-bridge.md`：`c15d5ee6566bc8e0c7af7571bcd90af4c8243b3b6a6cf4620a5821bf19b04618`。
- `auto-ui/experimental/autographics-auto-driver/src/lib.rs`：`7926267bd2ec52e44c73c81da3632b87bb0bcb0d59d34b9ae95b5a4247441375`。

2026-11-07规划复核：用户授权复核并直接完善这五份合同，仍先不实施。以下r2替代r1当前合同；§9旧记录保留为历史。状态保持drafting/current_step=0，不把本轮静态检查算作产品验证或独立final。

## 5. 详细设计

T01逐行冻结入口→作者producer→编译/执行→host→内容→包/安装/日常测试图，至少behavior/queries/motion/Node2D/3D/audio/HUD/editor/mixed全部真实profile及旧GUI/CLI入口；不能只看AG021或因外部exe把内容仓排除。输入模块/negative dependency/SDK来源/feature/旧binary/恢复/回调/权限/owner均有分类与后继。

最低整链样本固定为普通use root＋两个同定义Counter组件，各自model/msg/on，Life.Start/Tick；root拥有UI动作并驱动一个2D和一个3D区域，组件局部事件2:1隔离，N步恰N次推进。保正常作者层，不把完整showcase所有功能当本研究分母；本最小混合链不可缩成标量控制台。独立CPU预期状态/空间变换与GPU ROI比较，变更一个组件源常数后结果变化。资源采用现有合法过程几何/字形，不加载第三方素材。T02固定源和独立oracle后，缺语法/语义/host支持须具名no-go，不删除use/组件/图形出口凑绿。

候选A：Schema/统一前门直接领域投影→受检ACE语义→AV→受检host；候选B仅在A缺正式领域投影时评估复用中立领域描述/受检adapter，业务仍走统一源前门，不调用旧Parser/AST emitter、不正则改写源码、不另造解释器。B未满足作者语义与源码span则no-go；本研究不批准过渡方案成为产品架构。

冻结bool/int/f32/文本/数组、句柄代次/错误/cancel/释放与回调签名。无f32公共lane时先核可复用typed host面，或显式f64边界适配的舍入/NaN/Inf/-0与空间判据；不把私有ABI别名公开foreign。缺口交语言owner，937/938仅标量不代表939资源/回调已具备。只等实际被读接口交付，不依赖完整ACE M2、AGE M1或所有标准库计划。

T04观测稳定宿主业务源编辑cargo/rustc次数、未改函数P/IO、同PID更新/旧调用链与坏候选保持last-good；T05观测冷恢复/取消/Stop/20轮新Run/坏资源修复/owner收支。研究未实现某完整REP门必须逐项pending，不以研究总pass冒迁移签章。32credits/1000steps与ACE fuel分账；不临时抬额。

结论规则：go只给固定样本和已实测接口；bounded-go必须列支持/排除及产品差额；no-go可作为研究交付，但须正常源真实失败入口/阶段/诊断、至少一项相邻合法ACE正控和无法推进的具体符号/类型/接口/owner/关闭oracle。若在首个前门已阻断，后段明确not-run，不能编造完整GPU失败或要求无限扩语言；两候选裁决之后停止替代机制探索。未知/仅文档推测不是可交研究结论。

未来在scope建立`probe/run.py --controls --out <scope>`（新，由T02/03建立），固定依赖源码/构建图和完整命令；验证bad binding/source/epoch/cancel和精确修复。禁止隐藏legacy fallback，构建依赖与实际dispatch双证。所有显式ACE构建在本Plan依赖树/独占target，语言主检出不测试；无语言代码改动不跑根全量。

### 与既有ACE合同的唯一职责交接

2026-11-07核得以下均为语言仓现存drafting合同，存在不等于接口已交付；本Plan不等它们整包完成才研究，也不创建平行生产迁移计划。

| 交接对象 | 本Plan提供 | 正式责任与关闭方 |
|---|---|---|
| auto-lang/959 | AGE真实入口/旧SDK间接消费者与证据引用 | 全产品唯一分母；UI067保AGE细化图并引用959行ID，不维护第二套总表；959未冻结时用具名临时映射，T06交接不硬等959 |
| auto-lang/979、967/968/969/970（按实际接口） | 正常源拒绝、span/数值/host/调度/恢复差额与最小反例 | 语言仓通用Schema、受检语义、执行宿主与session接面；不让UI探针补核心实现 |
| auto-lang/980、981 | 已证机制、失败支路、作者/Session全图与原内容oracle | 980现有AutoUI迁移；981统筹AGE生产接入，auto-ui拥有领域语法/生成器/driver/renderer；UI067可复用证据不替代其T01生产适用性验证 |
| auto-lang/981、985/986/987 | 两游戏/Gallery/showcase包、冻结SDK与测试链清单 | ag-examples内容/包/消费验证归981跨仓实施批次；985联合认证，986安装发行，987旧引擎退役签章；语言计划不能代写内容业务 |

T06逐差额记录接收Plan/任务/文件/owner/关闭oracle；T01若发现既有相同输入与配置的整链证据则复用，不再搭另一套探针；新结果只覆盖缺口。未知接口允许在固定已交基线上触发真实拒绝后给no-go，不能因“须接口已交付”无限等待；有接口的候选不可只凭另一候选失败判no-go。后段not-run有不可达因果与接收owner，缺证不能伪称研究已完。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/ace-migration-feasibility.md | 现有知识/缺口→本合同已验证支持域、来源及未证边界 | 逐AC真实沉淀，不把计划写成支持 | AC-01..AC-05 |

## 6. 测试设计

以下均为未来work门，本轮未执行。实施者自检后交独立上下文final，原始工件与源/输入/配置绑定；go/bounded-go的已支持支路缺必需oracle不能execution_done；no-go支路按§5真实拒绝/合法正控/后继关闭oracle验收研究结论，未可达V-03/V-04保持not-run，不充运行PASS或免除未来迁移义务。每个V由对应任务建立可执行入口，不能用拟建脚本名充已运行。

| V | AC | work任务/入口 | 输入与区分性oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 | T01；只读图与清单 | 每类真实入口/旧副本/包/测试有producer/consumer/owner，遗漏known profile控制使盘点失败 | 不能从grep0推disable |
| V-02 | AC-02 | T02/03；未来scope probe | 正常多模块/组件/三呈现样本经ACE执行，Auto变源改结果，CPU状态与GPU位置像素一致 | 或按明确no-go规则记录真实阻断，不删最低样本 |
| V-03 | AC-03 | T03；绑定数值控制 | 错bool/签名/旧句柄/非有限值拒绝；合法f32边界按独立舍入oracle；失败不发布 | 不可达按真实前门阻断＋相邻合法正控＋后继oracle记录no-go/not-run，不写运行pass |
| V-04 | AC-04 | T04/05；probe运行/进程与生产点 | Rust构建0实际进程观测；更新PID/调用链与取消恢复/owner分账；强制重启/旧callback控制可区分 | 未通路径not-run，REP不由微样本签章 |
| V-05 | AC-05 | T06；研究报告/SD | 两机制真实裁决、缺口后继和REP映射，无未归属unknown；准备/构建/运行/清理分列 | 研究no-go可交，非迁移完成 |

实施、构建、测试仅在唯一专用树；主检出仅Plan簿记。独立上下文final在同树顺序进行，review期间停实现写入，finding返work后由同员定向复核；自检不代final。本轮不创建agent或实现树。GPU/音频/焦点/重门机器单实例；原生自动化须沿实际授权，不能以未来work授权推断全局OS自动化暂停已解除，手工配方可用，缺设备/授权保持该门pending。只跑真实触面命名case/必要消费者，不每次merge跑全量AGE；里程碑必要并集单独一次窗口。首红、环境重放绿和未执行分别保留。

材料唯一`D:/autostack/plan-artifacts/plan-067/auto-ui/`，work开工核同号owner、登记用途/路径/在途；所有探针、SDK副本、日志、外部B/安装根、外置target/包/截图均在此scope。永久回归夹具可入产品树，临时探针不可散落仓内。merge把命令/退出码/关键数字/源hash/失败与关闭摘要折入Plan，对账全部实际输出（含外盘）后删除自有scope及树内构建物；仅具名owner+理由+关闭条件hold保留。不得删除整个同号父scope波及其他仓。禁止junction/symlink；清树前fresh `bash D:/autostack/wt-guard.sh <本树>`须clean。未知归属fail-closed；不写attachments/legacy specs.json/派生索引，不收编共享导航他人WIP。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 迁移分母覆盖全部真实旧消费者/构建/安装/测试，版本/owner可追 | V-01 |
| AC-02 | work | 固定正常作者最低整链有真实结果或可复现no-go及正控，未删分母 | V-02 |
| AC-03 | work | 数值/资源/host可达支路有区分性证据；不可达支路有真实上游拒绝/合法正控、逐类型缺口及后继oracle，不伪称已运行 | V-03 |
| AC-04 | work | 开发/恢复/寿命观测与未执行项如实分层，无微样本冒REP完成 | V-04 |
| AC-05 | work | 有界研究结论/SD/后继接口与owner/关闭oracle可用于正式立约 | V-05 |
| AC-06 | review | 同树独立final逐AC/SD/源/实际反控复核pass，遗漏延期workaround与告警审查齐，finding关闭；绑定r2/准确提交 | T-07 |
| AC-07 | merge | 冻结实际触面门/源身份后落主，canonical与同号归档真实落位，正确表达能力与限制 | T-08 |
| AC-08 | merge | 全输出对账、摘要折Plan、fresh guard清自有树/分支/scope并刷新具名hold登记 | T-09 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 另行启动；独立于M1收口 | 冻结ACE实际交付/旧引擎全图/跨仓owner与材料，重核922 F11/C08/UK06 | AC-01 | [x] 2026-11-07（R2 已修）冻结清单 `plan-artifacts/plan-067/auto-ui/t01-inventory.md`（14,305B / sha256 `0ced1c65…698d1415`）；整链7段+profile矩阵+ACE 13 profile 与 **4 个公开能力常量**（3 标量入口实测 + `ac.source.modules.packages/v1` 未触达）+922 F-11/C-08/UK-06重核 |
| T-02 | work | T01 | 固定最低样本/独立oracle/类型和资源规则、最多两机制与可执行probe入口 | AC-02/03 | [x] 2026-11-07 冻结见 `plan-artifacts/plan-067/auto-ui/t02-design.md`（6,464B / sha256 `ff62617223…fb8089a`）；样本 main.at/counter.at 被交付前门 `load_project_mixed`+`lower_project_node_mixed_regions` 接受，产出 `world_sha256=1599d225…4218b9`（15,689B）；oracle 常量与生成源逐项相同、改源 b.step 2→3 产品变 `[1,3]`；`probe/run.py` 三阶段 pass，ACE 阶段 not-run（T-03） |
| T-03 | work | T02；固定当前正式交付基线（go支路另需对应接口实交付） | 执行最小整链或真实阻断对照，确定go/bounded-go/no-go，缺接口不跨仓补实现 | AC-02/03 | [x] 2026-11-07（R3 已修）**no-go（作者profile整链）**：ACE `436ac91f` 前门实测 main.at `source.unsupported`@`use`486..489、counter.at `source.syntax` "expected `fn`, found `node`"319..323、legacy world.at `source.unsupported`@`var`228..231；相邻正控标量模块图 accepted、unknown/duplicate/private/cycle 四负控具名拒绝；语义/AV/host/GPU 后段 not-run。B 候选因需自解析/复制语义核心亦 no-go，停止替代机制。探针源 `probe/ace_frontdoor/src/main.rs` 10,159B `7f63ba0b…89faf0e`、`probe/run.py` 12,190B `bcc887e8…83cea000`；详见 t03-result.md（8,155B / sha256 `3eb86f4b…bdcae98`） |
| T-04 | work | T03；可达支路 | 源编辑/增量/同PID观察与legacy反控；阻断支路明确not-run | AC-04 | [x] 2026-11-07 **not-run（不可达）**：T-03 作者源在 ACE 首个 Text 门被拒，无 ACE 目标/会话可编辑或同 PID 观测；legacy 侧仅有 T-02 产品级改源证据（`counter_step [1,2]→[1,3]`），不构成运行时增量证据；REP 不由研究签章 |
| T-05 | work | T03；可达支路 | 恢复/取消/20轮Run/资源收支及坏件修复；未知不能冒pass | AC-04 | [x] 2026-11-07 **not-run（不可达）**：无 ACE 运行会话，冷恢复/取消/Stop/20 轮/Run 代次与资源收支均无可观测对象；不伪造微样本 PASS，REP-07 等仍归后续 ACE 宿主合同 |
| T-06 | work | T04/05分类齐 | 冻结SD/完整缺口与既有959/979–981/985–987的逐项接收映射、实际owner和关闭oracle；覆盖项不另立重复合同，报告与真实工件一致 | AC-05 | [x] 2026-11-07（R1/R2 已修）SD-01 候选落工作树 `docs/specs/auto-graphics/ace-migration-feasibility.md`（15,647B / sha256 `340571dd…12919670`，commit `bb1d52d0`）；G1–G9 逐项映射 981 B-SOURCE/B-SESSION/B-CONTENT、980 B-DIALECT/B-NATIVE、979 T-02/T-03/T-08、961、939 T-03..T-08、967/968、985–987；REP 按权威 M2 合同全称与 owner 分账（REP-01 原源与全消费者=959+每能力包；REP-06=967+980；REP-08=986；REP-10=987；REP-11=985+各consumer），全 pending；能力常量更正为 4 个；UK-06 具名移交、无未归属 unknown；附录A与样本逐字相等（check exit 0）；未新立合同 |
| T-07 | review | T-01..06实质完成 | 独立final；不修实现，finding交work，同员受影响复核 | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结门/身份落主、canonical/导航/同号归档；不自动tag/远端发布 | AC-07 | [ ] |
| T-09 | merge | T-08 | 全输出对账/摘要/guard/清理/登记；实完才勾 | AC-08 | [ ] |

## 9. 复审记录

2026-11-06 / stage: new / auto-ui/PLAN-067:r1 / outcome: pass（规划合同可审阅，非独立final）；next: 用户后续启动后按T-01进入，逐任务依赖见§8。drafting 0/9；未建树/scope、未实现/构建/GPU/OS操作/迁移/发布/创建agent，未改既有合同及历史锁。

2026-11-06 / stage: new / planning_validation: pass（仅文档预检）。本批五合同各0..10节、44个任务全部未勾，AC/V/SD/owner_stage与任务对应；依赖文件可解析且新合同DAG无环，本地号唯一、counter UI068/AG026。新增路线/导航链接实核通过，实际源码/命令选项已静态核；研究运行结果均未执行。输入SHA见§4，未修改既有实施合同/current Spec/产品代码/历史锁，未创建树或材料scope。此记录不是独立review或实现PASS。

2026-11-07 / stage: new / auto-ui/PLAN-067:r2 / outcome: pass（规划合同修订检查，非实现或独立final）。消除T03接口先交付与no-go出口冲突；不可达数值/寿命支路如实not-run并有拒绝正控；新增959/979–981/985–987等现有合同接收映射，避免重复B/C/D立项。 next: 保持drafting，待用户后续启动按§8进入；所有任务未执行，原任务数不变。本次未改产品/Spec现状、未创建树/运行测试/打tag。

2026-11-07 / planning_validation: pass（纯文档静态检查）。五份r2共44任务、36项AC、24项V，均未勾选；每份0..10节/任务计数/owner_stage及Spec目标核对通过。包含本批及981/959/987前置的58节点可达图（已归档节点作交付边界）无显式循环或缺失引用；本批与路线新增相对链接可解析。AG025修改的age-m1-obligations由硬前置AG024新建，非现存Spec。另人工核审计→最终验收、先证→接口、981→AGE阶段三类隐含等待已解开。源码lower_node_ui/character3d与AG021归档SHA仍与§4相符；本批无实施树。此结果不证明产品行为通过，不免未来独立review。

- stage: work | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: pass（T-01 完成；T-02..T-06 未开始，work 阶段继续） | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | base_commit: 4bb54e36 | code_commit: 无（本单元只读盘点，树未改） | dependency_revisions: auto-lang 436ac91f（experimental/ace 0 修改）/ ag-examples 5384f7b / engine-lock sdk_pin 7b987ca8（source-lock 另钉 932ad487，UK-06 未闭） | task_ids: T-01 | evidence: plan-artifacts/plan-067/auto-ui/t01-inventory.md（14,100B / sha256 f1feb80a01…0f8cdc4a）；§4三条输入指纹重核全等；.sdk-source==sdk/frozen 681/681 全等；ACE 13 profile+3 capability、有限模块能力仅 i32/bool 顶层自由函数；auto-ui 直接 ACE 依赖 0（仅经旧 auto-lang-core 链入数据契约 ace-semantics-contract） | blockers: 无用户待答 | next_responsible_role: work（T-02 固定最低样本+独立 oracle+probe 入口）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass（T-01 完成，work 继续） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: 4bb54e36 | next: work T-02。

- stage: work | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: pass（T-02 完成；T-03..T-06 未开始，work 阶段继续） | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | base_commit: 4bb54e36 | code_commit: 无（产品树未改；探针/样本/构建均在 scratch `plan-artifacts/plan-067/auto-ui/`） | dependency_revisions: auto-lang 436ac91f（新增只读兄弟树 `D:/autostack/.wt/ui-067/auto-lang`，detached、clean，供前门 path 依赖解析）/ ag-examples 5384f7b / engine-lock sdk_pin 7b987ca8（UK-06 两钉不一致未闭） | task_ids: T-02 | evidence: t02-design.md（sha256 ff62617223…）+ `probe/run.py --out out/probe` 结果 partial：oracle/legacy_frontdoor/legacy_source_change 三阶段 pass、ace_frontdoor not-run；交付前门接受样本并产出 world_sha256 1599d225…4218b9（15,689B；schema autographics.node-ui-mixed-regions/2）；11 项结构断言全真；改源 b.step 2→3 后生成 `counter_step=[1,3]` | blockers: 无用户待答 | next_responsible_role: work（T-03 ACE 前门实测与 go/bounded-go/no-go）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-02 完成） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: 4bb54e36 | next: work T-03。

- stage: work | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: pass（T-03 完成，裁决 no-go；T-04/T-05 将按不可达记 not-run，T-06 待做） | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | base_commit: 4bb54e36 | code_commit: 无（产品树未改；探针在 scratch） | dependency_revisions: auto-lang 436ac91f（只读兄弟树，ACE 子树 clean）/ ag-examples 5384f7b | task_ids: T-03 | evidence: `python probe/run.py --out out/probe --controls` exit 0、report `pass`；t03-result.md（sha256 `7163a1e4…22bc0af4`）；前门实测：main.at `source.unsupported`@`use`(486..489)、counter.at `source.syntax` expected `fn` found `node`(319..323)、生成的 world.at `source.unsupported`@`var`(228..231)；正控 std.answer+entry accepted（units≥2/items≥1）；负控 module-missing / module-duplicate-import / module-private / module-cycle 全具名拒绝；观察：`answer(1)` 绑定期接受（类型核验属下游）；后段语义/AV/host/GPU/编辑/恢复 not-run | blockers: 无用户待答 | next_responsible_role: work（T-04/T-05 not-run 记录 + T-06 缺口映射与 SD-01）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-03 裁决 no-go） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: 4bb54e36 | next: work T-04/T-05 not-run + T-06。

- stage: work | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: pass（work 阶段完成：T-01/T-02/T-03/T-06 完成，T-04/T-05 因 T-03 首门阻断记 not-run） | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | base_commit: 4bb54e36 | code_commit: d24c4635（SD-01 候选；工作树 clean） | dependency_revisions: auto-lang 436ac91f（只读兄弟树，ACE 子树 clean）/ ag-examples 5384f7b | task_ids: T-04, T-05, T-06 | AC 结果: AC-01=T-01 分母图（7段/profile矩阵/959 F-11·K-02 引用）；AC-02=冻结样本经 legacy 前门接受+ACE 真实 no-go+相邻正控；AC-03=可达支路 legacy 产品/oracle 常量逐项一致（`world_sha256 1599d225…`），不可达支路具名拒绝（`source.unsupported`@`use`486..489、`source.syntax` expected `fn` found `node`319..323、`@var`228..231）+逐类型缺口 G1–G9+后继 oracle；AC-04=T-04/T-05 not-run 如实分层、REP-01/06/08/10/11 全 pending、不以微样本签章；AC-05=SD-01 `docs/specs/auto-graphics/ace-migration-feasibility.md`（14,922B / sha256 `f77b2708…bbe60f5c`，commit d24c4635）+ G1–G9→981 B-SOURCE/B-SESSION/B-CONTENT、980 B-DIALECT/B-NATIVE、979 T-02/T-03/T-08、961、939 T-03..T-08、967/968、985–987 的 owner/关闭 oracle，未新立合同，UK-06 具名移交、无未归属 unknown | evidence: t06-report.md（4,987B / sha256 `0927b9bb…2e2524`；阶段分列/一致性核对）；check_spec_appendix.py exit 0（附录A与样本逐字相等）；`python probe/run.py --out out/probe --controls` exit 0 report pass（前轮在案） | 遗留: 无 in-contract finding；统一回归/冻结门归 review/merge | blockers: 无用户待答 | next_responsible_role: review（T-07，独立 final）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 4 | from: work | to: review | status: execution_done | outcome: pass（work 阶段完成，T-04/T-05 not-run 已具名） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: b9233c84 | next: review T-07。

- stage: review | review_scope: final | reviewer: 独立 subagent 上下文（043c5659-0e2d-49b7-a9a3-65456484222d；与实现会话无共享上下文） | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: needs_fix | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | reviewed_commit: d24c4635 | base_commit: 4bb54e36 | dependency_revisions: auto-lang 436ac91f（ACE 子树 clean）/ ag-examples 5384f7b | spec_inputs: `docs/specs/auto-graphics/ace-migration-feasibility.md` sha256 f77b2708…bbe60f5c；sample main.at 7a14519f… / counter.at 8734bef7… | acceptance_results: AC-01 pass、AC-02 pass、AC-03 pass、AC-04 pass、AC-05 **partial**、AC-06 satisfied（本审查）；独立复跑 `probe/run.py --controls` exit 0 report pass，三输入双入口拒绝（`source.unsupported`@`use`486..489、`source.syntax` expected `fn` found `node`319..323、`source.unsupported`@`var`228..231），正控 accepted（units=2/items=2），四负控具名拒绝，附录逐字相等，ACE 树 436ac91f clean，主检出 PLAN-062 WIP 未触 | findings: **F-067-R1**(major, AC-04/05, T-06) SD §5.1 REP-01 标签/owner 与权威 M2 合同不符——REP-01 应为「原源与全消费者」、owner=959+每能力包（`ace-replacement-first-m2-contract-2026-11-02.md:62`、`ace-m2-delivery-roadmap.md:71`），并须对齐 REP-08 全入口实际切换/REP-10 旧实现彻底disable/REP-11 产品及测试迁移收据；**F-067-R2**(minor, AC-05, T-01) 漏第四能力 `ac.source.modules.packages/v1`（`packages.rs:11`、`semantics-contract/src/assembly.rs:11`、`docs/specs/ace-migration/m2-modules.md:11`）；**F-067-R3**(minor, AC-02/03, T-03) 探针源哈希未落——`probe/ace_frontdoor/src/main.rs` 10,159B 7f63ba0b…89faf0e、`probe/run.py` 12,190B bcc887e8…83cea000 | pending_closeout_gates: AC-07/AC-08 属 merge；统一回归/冻结门归 merge | evidence: 复核报告 `plan-artifacts/plan-067/auto-ui/review/review-report.md`（18,563B / sha256 e9bc2929…88c4d9）+ `out/review/report.json` | next: work（修复 R1–R3，同员受影响复核）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 5 | from: review | to: work | status: executing | outcome: needs_fix（AC-05 partial；R1 major、R2/R3 minor） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: d24c4635 | next: work 修复 F-067-R1/R2/R3 → 同员受影响复核。

- stage: work | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | outcome: pass（复核 F-067-R1/R2/R3 修复；work 回到 execution_done） | worktree: D:/autostack/.wt/ui-067/auto-ui | branch: codex/plan-067-dev | base_commit: 4bb54e36 | code_commit: bb1d52d0（工作树 clean） | dependency_revisions: auto-lang 436ac91f（ACE clean）/ ag-examples 5384f7b | task_ids: T-01, T-03, T-06（repair） | 修复: R1 SD §5.1 REP 表改为权威全称/owner（REP-01 原源与全消费者=959+每能力包、REP-06 现有宿主完整性=967+980、REP-08 全入口实际切换=986、REP-10 旧实现彻底disable=987、REP-11 产品及测试迁移收据=985+各consumer），t06-report 同步；R2 §2.1 补第 4 能力 `ac.source.modules.packages/v1`（`source/packages.rs:11`）并注明未触达，t01-inventory 与 T-01 行更正；R3 探针源哈希落 t03-result §1 与 T-03 行（main.rs 10,159B `7f63ba0b…`、run.py 12,190B `bcc887e8…`、Cargo.toml 419B `a1eeac3f…`） | evidence: SD-01 `docs/specs/auto-graphics/ace-migration-feasibility.md` 15,647B / sha256 `340571dd…12919670`（commit bb1d52d0）；附录A与样本逐字相等（check_spec_appendix.py exit 0）；t01-inventory 14,305B `0ced1c65…`、t03-result 8,155B `3eb86f4b…`、t06-report 5,136B `6e083a64…` | findings: F-067-R1/R2/R3 已关；无新 finding | blockers: 无用户待答 | next_responsible_role: review（同员受影响复核 AC-05 与证据一致性）

- stage: goal | plan_id: auto-ui/PLAN-067 | plan_revision: r2 | round: 6 | from: work | to: review | status: execution_done | outcome: pass（R1–R3 修复完成） | worktree: D:/autostack/.wt/ui-067/auto-ui | commit: bb1d52d0 | next: review 同员受影响复核。

## 10. 待澄清事项

D01：Schema领域投影/host支持是否已可消费由T01核真实交付；不预判go。（2026-11-07 T-03 实测：当前**不可消费**——正常作者源在 ACE 首个 Text 门被具体拒绝，见§9 r3 与 `t03-result.md`；结论 no-go，不影响 D02/D03 口径。）D02：研究no-go不是AGE M1阻塞，也不解除ACE M2最终迁移义务。D03：B/C/D优先交既有959/967–970/979–981/985–987合同，见§5；确无owner覆盖的差额才修订或另立有限合同，不重复取号/施工。
