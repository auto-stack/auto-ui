---
plan_id: PLAN-065
status: archived
feature_name: AGE基础UI与混合组件文字槽生成修复
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 8
total_steps: 8
depends_on: [auto-ui/PLAN-054, ag-examples/PLAN-021]
supersedes_spec_components: [docs/specs/auto-graphics/node-mixed-regions-lifecycle.md, docs/specs/auto-graphics/node-basic-ui.md, docs/specs/auto-graphics/node-mixed-ui-2d-3d.md]
new_spec_components: []
touched_goals: [GOAL-008]
---

# PLAN-065：AGE基础UI与混合组件文字槽生成修复

## 0. 变更摘要

关闭AG021 D-06的少文字槽def生成未声明text_b问题；交付公共生成器修复与准确内容消费配方。内容去补槽由AG025承接。

## 1. 目标

正常root/普通use组件0..4合法文字/按钮槽按现有合同生成并执行，bool/int及更新正确。非目标：扩控件/字符串语言、改SDK、提升4槽/每槽64字节分配及63字节UTF-8内容上限、增加新profile或完整UI融合。

## 2. 架构方案

现有Node作者源→lower_node_ui→world.at/ui.json/identity→固定SDK Session→NodeUi/MixedRegions。只修通用生成与其实际受影响路径，保持既有支持语义；允许修复所必需的生成字节变化，逐profile记录，不能伪称全部旧hash不变。

拟唯一树`D:/autostack/.wt/ui-065/auto-ui`，分支`codex/plan-065-dev`；跨仓源码只读取实际交付，若需产品修复超出本仓则交原owner另约，不直接改其主检出。

## 3. 技术栈

已核`experimental/autographics-node2d-pilot/src/lower_node_ui.rs`文字写入/getter与约2207行hi>0才声明text_b；pilot `tests/node_ui_gates.rs`、`mixed_regions_lifecycle_gates.rs`；driver `tests/node_ui_flow.rs`、`mixed_regions_lifecycle_storage.rs`。

## 4. 需求分析与背景调查

用户2026-11-06明确授权“AGE M1剩余收口工作和ACE迁移整链先证正式拆成计划，明确依赖、验收和执行顺序，先规划，不实施”。本合同只规划；没有执行授权、测试结果或新能力交付认领。主检出观察`auto-ui@02316271da9b9cd90750ca5a38ed91143f6c4be4`；路线输入为auto-ui `docs/design/autographics/age-three-milestones-and-ace-m2.md`磁盘稿，非已提交实现。新版M1沿原M2-01..10/G-M01..09全部义务，历史tag/号/归档原文保留。

已读node-mixed-regions-lifecycle Spec及AG021 canonical。源中多个生成分支引用text_b而声明有hi>0条件，静态路径与D-06一致；尚未本轮复现，不能预判唯一修法。旧SDK256元素截断规避仍靠两半缓冲，不能合并成256数组。


只读输入指纹（SHA256，磁盘实读；工作前仍须核正式来源）：

- `auto-ui/experimental/autographics-node2d-pilot/src/lower_node_ui.rs`：`e5a58018a3d8b7f62ddb539cdbe0bbb7c8865b002541ad83548a6a45a367f48b`。
- `auto-ui/docs/specs/auto-graphics/node-mixed-regions-lifecycle.md`：`1278171cbc3063fabd62441898140d623b073062f71145d73e27a10a7dfdc821`。

2026-11-07规划复核：用户授权复核并直接完善这五份合同，仍先不实施。以下r2替代r1当前合同；§9旧记录保留为历史。状态保持drafting/current_step=0，不把本轮静态检查算作产品验证或独立final。

## 5. 详细设计

T01复现一个真实≤2槽普通组件失败与3槽正控；T02选择声明完整或生成分支剪裁等最小等价修法，保4×64B半缓冲/ABI和无实例def行为。root与两个同定义组件分别测0/1/2/3/4槽，按钮标签计预算；无文字0槽仍须合法应用，不能通过删除控件或注入无业务占位槽换绿。bool高槽、负int/0/多位int、静态中文与文字长度边界沿现行Spec断言。第5槽与UTF-8内容达到64字节（含）按既定阶段具名拒绝；63字节接受，字面量与常量字段均测ASCII/多字节中文，按字节而非字符计数，不能把合法少槽改成拒绝。

命令基线：pilot工作目录`cargo test --test node_ui_gates --test mixed_regions_lifecycle_gates -- --test-threads=1`；driver工作目录`cargo test --test node_ui_flow --test mixed_regions_lifecycle_storage -- --test-threads=1`。T02/03新增命名case后冻结过滤与必要旧消费者并集；SDK准备从固定快照且681 hash核对，输出显式scope。AG025只消费本Plan最终交付身份，不能直接改AG主树。

去重边界：UI048/049/054已交基础UI、混合区域、组件局部状态/生命周期与高槽bool修复；本包只补少槽生成缺陷，既有功能按受影响路径回归。`classify_text_value`两处均以`len() >= TEXT_BYTES`拒绝，不能把64字节内容列合法。T04分别核048/049/054消费者，T05对应三个现行Spec提出增量，不重做组件系统。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/node-mixed-regions-lifecycle.md | 少槽def缺陷→0..4槽组件生成与实例隔离实证 | 组件消费修复 | AC-01..AC-03 |
| SD-02 | modify | docs/specs/auto-graphics/node-basic-ui.md | 既有root文字合同→通用生成器修复与63字节内容边界准确说明 | 共享生成器048消费者 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-graphics/node-mixed-ui-2d-3d.md | 既有root混合合同→受影响再生、身份与兼容实证 | 共享生成器049消费者 | AC-03 |

## 6. 测试设计

以下均为未来work门，本轮未执行。实施者自检后交独立上下文final，原始工件与源/输入/配置绑定；缺必需oracle不能execution_done。每个V由对应任务建立可执行入口，不能用拟建脚本名充已运行。

| V | AC | work任务/入口 | 输入与区分性oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 | T01/02；pilot现有gates新增少槽case | 1/2槽bad生成先真编译红，3槽绿；修复后不补槽即编译执行绿 | 不是仅搜索text_b字符串 |
| V-02 | AC-02 | T03；driver node_ui_flow | root/双同定义组件0..4槽；A两次/B一次2:1、bool切换/文字内容正确 | 槽总数含按钮 |
| V-03 | AC-02 | T03；生成/消费边界 | 4槽合法，第5拒绝；每槽64字节分配，63字节内容合法/64拒绝（字面量与常量字段、多字节UTF-8）；坏world/identity拒绝→修复下一Run绿 | 原错误阶段和SDK边界保留 |
| V-04 | AC-03 | T04/05；mixed storage及冻结再生 | 新生成字节/源锁一致；未受影响旧profile byte保持；应变部分明确登记 | 不全量重跑AG |

实施、构建、测试仅在唯一专用树；主检出仅Plan簿记。独立上下文final在同树顺序进行，review期间停实现写入，finding返work后由同员定向复核；自检不代final。本轮不创建agent或实现树。GPU/音频/焦点/重门机器单实例；原生自动化须沿实际授权，不能以未来work授权推断全局OS自动化暂停已解除，手工配方可用，缺设备/授权保持该门pending。只跑真实触面命名case/必要消费者，不每次merge跑全量AGE；里程碑必要并集单独一次窗口。首红、环境重放绿和未执行分别保留。

材料唯一`D:/autostack/plan-artifacts/plan-065/auto-ui/`，work开工核同号owner、登记用途/路径/在途；所有探针、SDK副本、日志、外部B/安装根、外置target/包/截图均在此scope。永久回归夹具可入产品树，临时探针不可散落仓内。merge把命令/退出码/关键数字/源hash/失败与关闭摘要折入Plan，对账全部实际输出（含外盘）后删除自有scope及树内构建物；仅具名owner+理由+关闭条件hold保留。不得删除整个同号父scope波及其他仓。禁止junction/symlink；清树前fresh `bash D:/autostack/wt-guard.sh <本树>`须clean。未知归属fail-closed；不写attachments/legacy specs.json/派生索引，不收编共享导航他人WIP。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 原D-06真实反例关闭，无占位补槽，合法少槽均可生成执行 | V-01 |
| AC-02 | work | 根/组件/实例隔离、文本与原预算正反边界成立 | V-02/03 |
| AC-03 | work | 兼容/来源/冷再生与SD/消费交接准确 | V-04 |
| AC-04 | review | 同树独立final逐AC/SD/源/实际反控复核pass，遗漏延期workaround与告警审查齐，finding关闭；绑定r2/准确提交 | T-06 |
| AC-05 | merge | 冻结实际触面门/源身份后落主，canonical与同号归档真实落位，正确表达能力与限制 | T-07 |
| AC-06 | merge | 全输出对账、摘要折Plan、fresh guard清自有树/分支/scope并刷新具名hold登记 | T-08 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 另行启动；前置已交付 | 重核Spec/源码/SDK/反例，登记唯一树scope，复现少槽与正控 | AC-01 | [x] 2026-11-07：树`D:/autostack/.wt/ui-065/auto-ui`分支`codex/plan-065-dev`base 97d58cf2；SDK prepare_sdk --from-snapshot verify PASS（681文件/17本地包）；auto-lang sibling@30b396d72只读；scope `plan-artifacts/plan-065/auto-ui/`登记（registry in-flight行）。源hash×2与§4一致（lower_node_ui e5a58018/Spec 1278171c）。反例真编译红：探针降层真实1/2槽组件（root/双实例）→固定SDK具名`Undefined variable: few1_ui_text_b_0`/`few2_ui_text_b_0`；3槽控制绿。pilot静态：1/2槽declares_b=false∧references_b=true，3槽双声明。材料t01/在scope |
| T-02 | work | T01 | 通用lower_node_ui最小修复，新增永久gates并冻结影响表 | AC-01 | [x] 2026-11-07 commit a16912d1：最小等价修法=≤2槽def剪裁buf分派（digits直写缓冲A、mag去buf参、ui_text_at单分支；e合法域恒<槽数≤2），≥3槽生成段逐字不变、0槽/无实例def行为不变。永久gates node_ui_gates+5例（few-slot剪裁形状0..4槽、双家族再生字节同一、0槽应用合法、63B合法/64B具名拒（ASCII+多字节、字面量+常量字段、按字节计数）、组件侧第5槽拒、无实例def具名拒合同面）；夹具fixtures/node-ui-few/{a,b,empty}。影响冻结表：≤2槽def生成字节变化（D-06反例无旧冻结产物）；≥3槽/0槽逐字不变（既有gates+verifier全绿证） |
| T-03 | work | T02 | 真实Session/双实例文字更新及合法/非法预算控制 | AC-02 | [x] 2026-11-07 commit 5a88fa0c：冻结产物ui-few（root2槽+Few0/Few1×2/Few2×2）、ui-few-multi（root4槽+Few3×2/Few4×2含63字节多字节字面量）、ui-few-empty（0槽应用）；sha 24def8af/2a112770/77914acf。driver node_ui_few_slot_flow 4/4（GPU单实例、固定SDK）：真点击2:1隔离c=-2/d=9（剪裁digits负数路径）、d按钮标签槽无恙；tick驱动1槽Few1 a=3/b=103参数隔离；root 2槽/4槽文字getter两形态；bool切换true→false→true；-5/0/12负零多位；Few4 63字节内容逐字节相等；0槽应用最小合法运行；坏world具名拒→精确修复下一Run绿 |
| T-04 | work | T03 | 受影响旧048/049/054消费者兼容、再生/身份与下一Run恢复 | AC-03 | [x] 2026-11-07：受影响并集全绿——pilot node_ui_gates 9+mixed_regions_lifecycle_gates 7（F-R1-01勘误：原记4，实跑7/7）+node_mixed_views_gates 5（含components-2d字节pin无漂移）；driver node_ui_flow 3+node_ui_storage 3+mixed_regions_lifecycle_storage 3+mixed_regions_lifecycle_flow 5；verify_plan049/verify_plan054 --controls ALL PASS（再生字节同一+SDK681）。**预存观察（非本Plan回归）**：verify_plan048再生控制对ui-ab产物报DRIFT——base 97d58cf2与当前生成器输出逐字节同一（sha 526e40e6双证），049半缓冲拆分改布局后未重冻结048 ui-ab产物（049 T-05明记「048 ui-ab产物driver侧照常装配/派发」），该verifier再生控制自049起即在master过期；048 driver消费者全绿不受累。pilot src/lower*.rs fmt drift亦预存（非本Plan文件） |
| T-05 | work | T04 | 冻结提交、验证摘要/SD候选与AG025消费配方；全部产品finding关闭 | AC-01..03 | [x] 2026-11-07 commit cd86a2ef：SD-01/02/03候选落worktree docs/specs/auto-graphics/（node-basic-ui半缓冲+剪裁规则+63/64按字节界；mixed-ui-2d-3d剪裁规则+产物无≤2槽def再生不变；mixed-regions-lifecycle 0..4槽组件全域生成执行+实例隔离实证）。AG025消费配方见§9 work receipt。wt-guard clean；瞬态t065探针已删（副本在scope）。产品finding零未关 |
| T-06 | review | T-01..05实质完成 | 独立final；不修实现，finding交work，同员受影响复核 | AC-04 | [x] 2026-11-07：同树独立子代理final@cd86a2ef pass（详见§9 review receipt；全门禁亲跑，F-R1-01..03均P3且1已闭环/2预存登记/3在案注记） |
| T-07 | merge | T-06 pass | 冻结门/身份落主、canonical/导航/同号归档；不自动tag/远端发布 | AC-05 | [x] 2026-11-07：rebase 964328b4（docs-only，range-diff 3/3全等；映射a16912d1→3eabe7e7/5a88fa0c→bf07a942/cd86a2ef→55c20a86）；rebased tip抽核gate 7+9绿+spec sha同；master --ff-only落主tip=55c20a86c412315d9840aa522647df94fa83044d（无merge commit）；canonical三Spec落位sha 0a3e8616/b1d3989e/5aadd45b |
| T-08 | merge | T-07 | 全输出对账/摘要/guard/清理/登记；实完才勾 | AC-06 | [x] 2026-11-07：fresh wt-guard clean→worktree remove+branch -d（was 55c20a86已landed）+auto-lang sibling移除+空组ui-065删除+worktree list零ui-065；plan-artifacts/plan-065整删（13G含外置target；摘要已折§9）；registry行刷新为deleted-at-merge＋F-R1-02具名hold；归档git实证（archived/065...md status:archived@c6466264，旧路径无残留，8任务行齐） |

## 9. 复审记录

2026-11-06 / stage: new / auto-ui/PLAN-065:r1 / outcome: pass（规划合同可审阅，非独立final）；next: 用户后续启动后按T-01进入，逐任务依赖见§8。drafting 0/8；未建树/scope、未实现/构建/GPU/OS操作/迁移/发布/创建agent，未改既有合同及历史锁。

2026-11-06 / stage: new / planning_validation: pass（仅文档预检）。本批五合同各0..10节、44个任务全部未勾，AC/V/SD/owner_stage与任务对应；依赖文件可解析且新合同DAG无环，本地号唯一、counter UI068/AG026。新增路线/导航链接实核通过，实际源码/命令选项已静态核；研究运行结果均未执行。输入SHA见§4，未修改既有实施合同/current Spec/产品代码/历史锁，未创建树或材料scope。此记录不是独立review或实现PASS。

2026-11-07 / stage: new / auto-ui/PLAN-065:r2 / outcome: pass（规划合同修订检查，非实现或独立final）。修正63字节内容/64字节分配的边界；补048/049两消费者Spec增量；054已交能力只回归，少槽缺陷为唯一新修复。 next: 保持drafting，待用户后续启动按§8进入；所有任务未执行，原任务数不变。本次未改产品/Spec现状、未创建树/运行测试/打tag。

2026-11-07 / planning_validation: pass（纯文档静态检查）。五份r2共44任务、36项AC、24项V，均未勾选；每份0..10节/任务计数/owner_stage及Spec目标核对通过。包含本批及981/959/987前置的58节点可达图（已归档节点作交付边界）无显式循环或缺失引用；本批与路线新增相对链接可解析。AG025修改的age-m1-obligations由硬前置AG024新建，非现存Spec。另人工核审计→最终验收、先证→接口、981→AGE阶段三类隐含等待已解开。源码lower_node_ui/character3d与AG021归档SHA仍与§4相符；本批无实施树。此结果不证明产品行为通过，不免未来独立review。

2026-11-07 / stage: work | plan_id: auto-ui/PLAN-065 | plan_revision: r2 | outcome: pass（execution_done，5/5 work任务） | worktree_path: `D:/autostack/.wt/ui-065/auto-ui` | branch: `codex/plan-065-dev` | base_commit: 97d58cf2 | code_commit: a16912d1(T-02修复+gates)→5a88fa0c(T-03/T-04产物+flow)→cd86a2ef(SD候选docs-only后裔；5a88fa0c代码零变化) | dependency_revisions: auto-lang sibling@30b396d72只读detached；driver `.sdk-source` prepare_sdk --from-snapshot verify PASS（681文件/17本地包） | task_ids: T-01..T-05 | evidence: ①T-01真编译红（固定SDK Session::new）：`Undefined variable: few1_ui_text_b_0`/`few2_ui_text_b_0`，3槽few3控制绿；pilot静态1/2槽declares_b=false∧references_b=true。②T-02 pilot node_ui_gates 9/9＋mixed_regions_lifecycle_gates 4/4（含components-2d字节pin）；命令`cargo test --offline --test node_ui_gates --test mixed_regions_lifecycle_gates -- --test-threads=1`（CARGO_TARGET_DIR外置scope）。③T-03 driver node_ui_few_slot_flow 4/4：`cargo test --offline --test node_ui_few_slot_flow -- --test-threads=1`——真点击c×2/d×1→"-2"/"9"（2:1隔离＋剪裁digits负路径）、d槽1按钮标签"加"无恙、root 2槽/4槽getter、Few1 tick隔离a"3"/b"103"、bool true→false→true、-5/0/12、63字节内容逐字节相等、0槽应用最小合法、坏world具名拒→修复下一Run绿。④T-04并集：pilot 9+4+5、driver 3+3+3+5全绿；verify_plan049/verify_plan054 --controls ALL PASS。⑤影响冻结表：≤2槽def生成字节变化（剪裁分支，历史上无法编译故无旧冻结产物）；≥3槽/0槽/无实例def生成段逐字不变——base 97d58cf2与修复后生成器对ui-ab输出sha同（526e40e6）＋既有verifier再生控制复证。AG025消费配方：只消费本Plan交付链（cd86a2ef树）；产物`generated-node-ui/ui-few{,-multi,-empty}/`（world sha 24def8af/2a112770/77914acf，identity.json+closure齐）；canonical Spec=SD-01..03落位后的docs/specs/auto-graphics/{node-basic-ui,node-mixed-ui-2d-3d,node-mixed-regions-lifecycle}.md（merge后取落主sha）；生成器身份`autographics-node2d-pilot` node-ui profile；AG025不得直改AG主树。预存观察（非本Plan回归，留record）：verify_plan048再生控制自049起在master过期（049拆分未重冻结048 ui-ab产物，049 T-05明记driver照常装配）；pilot lower*.rs fmt drift预存。材料scope `D:/autostack/plan-artifacts/plan-065/auto-ui/`（t01反例、t04 base对照、探针副本、外置target×2） | blockers: 无 | next_responsible_role: review（T-06 同树独立上下文 final，review_scope: final；统一冻结并集由review/merge按工作流持有）

2026-11-07 / stage: goal | plan_id: auto-ui/PLAN-065 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-065/auto-ui | commit: cd86a2ef | next: review

2026-11-07 / stage: review | review_scope: final | reviewer_identity/limitations: fresh独立子代理上下文（同宿主会话家族派生，非人工、非独立机器）；全部门禁亲跑（driver串行单实例GPU --test-threads=1）；native SendInput手工门与全量AG回归不在本审（merge-owned/合同外）；首红以base源码机制复核（未重建pre-fix构建，F-R1-03注） | plan_id: auto-ui/PLAN-065 | plan_revision: r2 | outcome: pass（→reviewed） | worktree_path: `D:/autostack/.wt/ui-065/auto-ui` | branch: `codex/plan-065-dev` | reviewed_commit: cd86a2efcb9b047b0caf3f0e886c32033d80eff2 | base_commit: 97d58cf2 | dependency_revisions: auto-lang sibling@30b396d72只读；.sdk-source verify-only PASS 681文件/17本地包（评审者亲跑） | spec_inputs: SD-01/02/03候选@cd86a2ef（docs/specs/auto-graphics/{node-mixed-regions-lifecycle,node-basic-ui,node-mixed-ui-2d-3d}.md，评审者逐条对行为核） | acceptance_results: AC-01 PASS（5处`_ui_text_b_`发射位逐一核：decl hi>0守卫/write elem≥2蕴含≥3槽/digits!has_hi零b引用+mag去buf参/getter单分支；≥3槽fall-through与base逐token同；夹具真实业务内容无占位注水）；AC-02 PASS（few_slot_flow 4/4断言逐条对读亲跑：2:1真点击c"-2"/d"9"＋按钮标签槽无恙、tick隔离a"3"/b"103"、bool true→false→true、负-5/0/多位12、63字节CJK两次逐字节、0槽应用最小合法、坏world具名拒→修复Run绿；63/64边界gate含常量字段路径与"(64 bytes)"具名、组件侧第5槽"4-element budget"）；AC-03 PASS（verify_plan049/054 ALL PASS亲跑；ui-ab再生sha 526e40e6亲算==receipt；冻结产物字节pin；三AG025产物identity链world_sha256/user_source_sha256核对）；AC-04=本轮 | pending_closeout_gates: merge冻结门/落主/canonical SD落位/归档/清理/输出对账 | findings: F-R1-01(P3,closeout，已闭环)——work receipt与§8原记mixed_regions_lifecycle_gates=4实为7（7/7全过，评审者与main会话各自复跑证实）；§8 T-04行已勘误4→7，§9 work receipt按append-only保留原文，本行记录勘误。F-R1-02(P3,预存观察)——verify_plan048再生控制在master自049起DRIFT（评审者复现：committed 3c85ca6e=048时代1586fa47 vs regen 526e40e6=049半缓冲拆分面）；048 driver消费者3/3绿不受累；留merge收尾登记具名hold/记录，非本Plan回归。F-R1-03(P3,证据卫生注)——t01盘面first-red world为修后再生副本，字面pre-fix产物未保留；红以receipt具名SDK错误串＋base源码机制复核支撑，沿V-01"先真编译红"记录在案可接受 | 另核（A-H定向）：9个world.at全扫描零「引用text_b无声明」；bool高槽AG021反例1的verify_plan054标记与gate在位亲证；无specs.json/ledger/index写、无attachments/复建；三新文件rustfmt clean；pilot lower*.rs 14文件fmt drift预存（非本Plan） | evidence: 评审者自跑命令/数字见其报告（AC映射＋A-H全项）；本会话复核：mixed gates单跑7/7 | next: merge（T-07/T-08）

2026-11-07 / stage: goal | plan_id: auto-ui/PLAN-065 | plan_revision: r2 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-065/auto-ui | commit: cd86a2ef | next: merge

2026-11-07 / stage: merge | plan_id: auto-ui/PLAN-065:r2 | outcome: pass（completion_kind: delivered） | prepared: reviewed基线cd86a2ef（SD-01/02/03候选已在树，docs-only后裔于代码冻结5a88fa0c）；依赖无变化 | wt-guard: clean（落主前+清理前fresh各一次） | rebase: master前进964328b4（plan-068 docs簿记，不在依赖图）；rebase后映射 a16912d1→3eabe7e7、5a88fa0c→bf07a942、cd86a2ef→55c20a86，git range-diff 3/3全等0不等（安全重写证明）；rebased tip抽核 pilot gates 7+9绿＋三Spec sha与评审绑定候选逐字节同 | landed: 主检出 git merge --ff-only codex/plan-065-dev → master tip=55c20a86c412315d9840aa522647df94fa83044d（无merge commit，33文件+2795/−34）；落主烟测=rebased tip同内容验证（ff-only字节同一：pilot node_ui_gates 7/7+mixed_regions_lifecycle_gates 7/7+新gates在9/9并集内、src/specs sha亲算），工作树外置target构建面不落主 | canonical_verified: docs/specs/auto-graphics/node-basic-ui.md sha256 0a3e8616b8d2239bd4fb3c29c62c06951cb4ac276aa5fe78d3d79bf9fac61694、node-mixed-ui-2d-3d.md b1d3989e247f77968226efc254bd5e09fa6b424e69c1fa91e25303bb4fdb25be、node-mixed-regions-lifecycle.md 5aadd45b624b0a2817751cd412390469f8d6608660a9066e409eec8f6babdd37 @55c20a86（评审绑定候选逐字落位；无index/ledger写，沿PLAN-106） | archived: 本文件 git移docs/plans/archived/065-age-ui-text-slot-generation-closure.md status:archived（含合并收据提交） | 命名hold（F-R1-02，登记registry）: 048 ui-ab产物冻结面与verify_plan048再生控制自049半缓冲拆分起过期——committed ui-ab world sha 3c85ca6e（048时代1586fa47）vs 现生成器再生 526e40e6；049 T-05明记048产物driver侧照常装配（flow/storage 3/3绿不受累）；owner=auto-ui node-ui后续plan，关闭条件=若未来再生ui-ab须重冻结产物并同步更新verify_plan048再生控制 | 落地非部署: 本仓为库/研究仓，无驻留生产二进制/前端bundle交付面（017/019/020/021/054同例）；批量回归due=auto-ui无.last-batch-regression.json（程序属auto-lang），不适用 | next: cleaned

2026-11-07 / stage: merge | cleaned: 两树fresh guard clean→git worktree remove（auto-ui自仓注销+auto-lang sibling@30b396d72注销）+branch -d codex/plan-065-dev（was 55c20a86已landed）+空组D:/autostack/.wt/ui-065删除+worktree list零ui-065项；plan-artifacts/plan-065整删（13G：外置target×2/t01反例/t04 base对照/探针副本/review/门暂存，摘要均已折§9，无残留scope）；registry .retention/policy.json auto-ui/PLAN-065行→deleted-at-merge＋F-R1-02具名hold（owner=auto-ui node-ui后续plan，关闭条件=再生ui-ab须重冻结产物并更新verify_plan048）；归档实证：archived/065-age-ui-text-slot-generation-closure.md status:archived@c6466264、旧路径无文件、T-01..T-08行齐、frontmatter 8/8 | 落地非部署: 本仓为库/研究仓无生产二进制/前端bundle面；批量回归不适用（无.last-batch-regression.json） | 主检出遗留非本Plan WIP：docs/plans 060-068/README节/design稿/crates PNG均他会话簿记，未接管（本Plan仅worktree更新README 065行未staged，归sections owner） | next: done

2026-11-07 / stage: goal | plan_id: auto-ui/PLAN-065 | plan_revision: r2 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: 无（ui-065树/分支codex/plan-065-dev/组/plan-artifacts-065已清零，worktree list零ui-065） | commit: c6466264（goal收尾；交付链=55c20a86落主+canonical → c6466264归档+receipts → 本cleaned/goal收尾提交） | next: done / Done五项核对：①交付55c20a86在master祖先链、--ff-only无merge commit（range-diff 3/3全等重写证明）②canonical三Spec sha 0a3e8616/b1d3989e/5aadd45b@55c20a86在案（评审绑定候选逐字落位）③归档文件status:archived已提交（c6466264），registry行deleted-at-merge+F-R1-02 hold④树/分支/组/plan-artifacts-065清零实证（worktree list 0项、branch 0、scope不存在）⑤主检出遗留WIP均他会话簿记，本Plan零代码WIP；landing check=研究仓无生产二进制交付面；批量回归不适用（auto-ui无batch程序）收尾

## 10. 待澄清事项

D01：具体修法由T02等价选择，不改合法输入域。D02：若需改公开ABI/SDK或预算，先needs_replan；本合同不授权该扩围。
