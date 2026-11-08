---
plan_id: PLAN-071
status: archived
feature_name: AGE M1本机剪裁验证与原生取证支撑
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
current_step: 9
total_steps: 9
depends_on: [auto-ui/PLAN-065, auto-ui/PLAN-066, ag-examples/PLAN-025]
supersedes_spec_components: [docs/specs/auto-graphics/node-mixed-regions-lifecycle.md]
new_spec_components: [docs/specs/auto-graphics/m1-native-evidence-support.md]
touched_goals: [GOAL-008]
---

# PLAN-071：AGE M1本机剪裁验证与原生取证支撑

## 0. 变更摘要

承接DEBT-001中可本机独立交付的应用级clip/越界oracle，以及两项债务所需的只读原生事件/状态/项目身份记录支撑。不开双DPI外部门，不关闭DEBT-001/002，不重新实现065/066或编辑器。

## 1. 目标

- 在真实UI+2D+3D应用组合的最终合成画面，证明区域内像素可见、区域外不泄漏，并验证区域外点击不误派发；HUD单锚点scissor不能替代。
- 为AG026/027提供可永久重建的记录接口、版本化报告和手工逐项配方接面；记录实际观察，不自动宣称原生语义通过。
- 本机单档scale=2.0即可完整关闭本Plan；缩放模拟、离屏/posted结果各自分层，不能关闭外部真实DPI门。

非目标：改变玩法/预算/碰撞/物理/文本槽，扩新控件、多窗/平台、ACE迁移、完整游戏演示包装、全量重验M1或修改历史证据。

## 2. 架构方案

现有MixedRegionsHost/authoring Host → 只读边界/状态检查点 → 实际合成capture与OS事件记录 → 可比对报告 → 内容仓AG026/027校验。优先使用现有公开capture与widget_bounds，确无所需可观测面才增小型只读访问器/命令参数；不得复制事件路由/游戏状态或把测试公式当产品状态。

拟唯一树`D:/autostack/.wt/ui-071/auto-ui`，分支`codex/plan-071-dev`。材料唯一`D:/autostack/plan-artifacts/plan-071/auto-ui/`；ag-examples只读，不能在此跨仓修改内容/锁或债务状态。独立本机合同交付不等待AG026/027。

## 3. 技术栈

已核driver `src/node_mixed_regions_host.rs::{widget_bounds,region_surface,frame,resize}`、`src/bin/mixed_regions_window.rs`的`--ui/--world/--manual/--report`，以及`src/bin/autographics-authoring.rs`、`src/bin/authoring/cli.rs`、`src/bin/authoring_probe/viewport2d.rs`/`viewport3d.rs`与`tests/node_editor_projects.rs`原阶段表。只读事件记录接真正winit事件路径；两条宿主分别绑定具体项目，不能用默认sample代教程项目。

## 4. 需求分析与背景调查

用户2026-11-08明确要求把DEBT-001/002正式拆成计划，明确本机可先做、外部条件、验收和顺序，避免重复已完成工作，先规划不实施。此次仅编写合同/导航/债务承接指针；没有实现、构建、GPU/OS输入、修改显示设置、设备采购、远端发布、tag或创建agent/worktree授权。未来启动沿届时明确指令；2026-11-07对AG025的定向人工/自动化授权不自动扩为本后继全局许可。

读取UI HEAD `86c4929f444898b2a534e3654c9a945b248ae6aa`、AG HEAD `5384f7bde4908c2803e51c64ba545cf15768a034`；读UI mixed-regions现行Spec，AG age-m1-acceptance、debts与G04 oracle绑定。AG无docs/specs/overview.md，直接按实际模块Spec读取；两外仓根/相关docs目录未发现额外AGENTS。未来开工仍重核当地约束及当前正式交付。

已交：UI065文字少槽55c20a86；UI066输入/碰撞/Run与host寿命5a53b789；AG025同版候选5a53b789+lang30b396d72+SDK681、完整非原生门与人工事件计数，第二scale/应用clip/逐项语义及OS编辑残余仍OPEN。原录制scope已清理；只复用Git摘要与正式来源，不能要求旧文件长期存在。

输入SHA256：mixed_regions_window.rs=`f16505ea442b27d70b715dac7f3aba71b6c1a7f3abcc64411327e9d54300cdbc`；node_mixed_regions_host.rs=`4dc602e6b14915ebcd2611be3e10f26eeebe32d317ac51bfd344feaa30cd35ed`；authoring/cli.rs=`adc411d55256b2242d339c6bfa2f05a92cf4fc2bec84303941f9049e5ee4c116`；node_editor_projects.rs=`64c3eeb986789a3bd1d0061b85a0030bc2efa5ac26c42b4f2445b7badb2bb689`。这些是读取身份，不是新验证结果。

## 5. 详细设计

T01按真实来源核已有观测面，固定最少样本/像素ROI/边界与事件反例，再决定可等价的最小实现；T02先验证capture能观测真实混合最终合成（含UI/两维区域），不得只截图region纹理证明没有合成泄漏。新测试拟`tests/m1_native_evidence_support.rs`，由T02建立并注册，新增采样/报告文件位置由T01冻结到现有driver模块。

| 规则 | 输入/观察与关闭oracle | owner/覆盖 |
|---|---|---|
| C1 应用级clip | 实际showcase内容副本，保持业务/区域语义；让有高对比可识别几何接近边界，读独立计算的内/外ROI，内部有内容而外部不被该区域污染；禁先裁输入图或用同一clip实现算expected | T02/03；二维、三维各一例；只新增取证fixture，不回写原示例 |
| C2 路由/状态 | 边界内一次点击只推进该receiver；外侧/另一层/控件消费后目标不变，比较真实Auto计数与公开状态；像素与状态均从同帧检查点绑定 | T03；不重做066焦点路由 |
| C3 布局/挂卸 | 单档实际2.0窗口resize、卸载/重挂后重新测bounds，旧ROI/旧挂载代次不得错计；需要模拟scale仅作local control | T03；不是双真实scale |
| C4 只读记录 | OS事件、模式、窗口实例、项目路径/哈希、引擎/工具/内容/检查点、候选ID、当前实际scale/尺寸、phase与结果同一run_id；字段缺失/伪来源不可升级native PASS | T04；mixed与editor两宿主，原生记录不注入全局输入 |
| C5 故障分层 | 单档/仅事件计数/posted/离屏/缺语义观察分别保持pending/not-run；坏报告与精确修复下一次校验可区分 | T04/05；支持AG端严格checker |

T04按现有mixed `--manual`检查表固定N1..N6（TabEnter一次、外侧不变、失焦不泄漏、离窗取消按下、resize后新命中、第二真实scale重复）。支持记录真实ScaleFactorChanged或平台等价事件及检查点实际scale/前后计数与观察人结论；事件数多不等语义正确。编辑侧仅提供原项目/当前状态与原OS阶段记录接面，不重建20/34阶段oracle或自动跑原probe。显式`--report`指向本scope，禁止沿代码默认旧plan054路径落材料。记录逻辑不应更改Tick/派发次数；T05用有/无记录同输入的值/帧对照证明观察不干扰业务。

T06冻结可由AG消费的正式SHA、事件报告schema/字段、样本/命令和适用边界。如果改window/runner/library导致候选变化，明确告知AG026最终候选需按原Spec重验同版受影响/必需门；不得改原M1失效规则以省门。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/node-mixed-regions-lifecycle.md | 既有区域路由/布局→新增已实测合成clip与只读证据接面，真实DPI仍未证 | 应用级观测 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-graphics/m1-native-evidence-support.md | 临时计数/旧scope→永久记录schema、宿主/项目绑定、复现入口与覆盖边界 | AG026/027可独立消费 | AC-03/04 |

## 6. 测试设计

均为未来门，本轮未运行。T01把新增target/case和已有受影响命名case的真实命令冻结；driver内执行`cargo test --test m1_native_evidence_support -- --test-threads=1`（拟新增，T02建立），必要既有`--test mixed_regions_lifecycle_flow`与`--test node_editor_projects`只选择实际受影响case。根crate零改动不跑语言根组，不跑全AGE。

| V | work任务 | 输入与区分性断言 | AC/边界 |
|---|---|---|---|
| V-01 | T02/03 | 真实混合合成2D/3D内外ROI；故意移除/偏移clip的实际测试副本必须红，修复同副本回绿；内部缺画也红 | AC-01；不靠截图肉眼宣称 |
| V-02 | T03 | 内/外/重叠控制点击与resize/挂卸真实状态检查点；故意错误坐标/旧bounds被发现 | AC-02；native标签不由此获取 |
| V-03 | T04 | mixed/editor各合法项目记录有真实状态/来源；错项目/hash/run/mode/旧epoch/未到最终检查点拒绝 | AC-03；只读记录 |
| V-04 | T05 | 观察开关业务值/推进次数相等；false/pass字段不能吞错误或timeout，记录器退出收尾可追 | AC-03/04；保原寿命语义 |
| V-05 | T06 | clean消费配方/字段/输入hash齐，单scale明确未证，外部设备与AG接收任务具体 | AC-04；本Plan不关闭债务 |

### 执行与证据规则

未来实现/构建/验证仅在本Plan唯一专用树，主检出只作Plan簿记。独立review在同树、独立上下文且实现停写后进行；finding回同树work，复核绑定revision/提交，不以自检代final。review/merge未来任务不阻work交审；本合同work-owned必需项未闭则不得execution_done。设备/输入条件不足保留未执行与具名等待，不变NA、不把手工记录器auto_pass=false改成成功。GPU/音频/焦点/性能窗口单实例错峰，按真实触面跑命名target/case/反控；全量AGE只在必要里程碑同版窗口执行，不作每次merge门。首次红、环境原因、重放绿分别记录，零选择/子进程失败/timeout不算PASS。

所有临时探针、源码副本、target、包、外部消费根、截图、录制、原始轨迹均在本Plan scope，开工即登记repo/owner/用途/路径/在途。永久fixture/checker/schema与操作说明入产品相应目录；不把临时材料放项目目录或attachments，不写legacy specs.json/派生索引。merge将命令/退出码/关键值/源与工件hash/覆盖边界/失败及关闭事实折入Plan，再逐命令对账所有输出根（含外盘）并删除本仓自有scope与构建物；仅具名owner+理由+关闭条件hold保留，不能删同号其他仓目录。禁止junction/symlink；清树须以`C:/Program Files/Git/bin/bash.exe D:/autostack/wt-guard.sh <本树>`fresh clean（避开WSL bash误报），随后核树/分支/组/输出均实际清理。未知归属fail-closed；不收编/清理共享主检出他人WIP。


## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 同一真实混合合成2D/3D应用级clip内外独立oracle与故障控制成立 | V-01 |
| AC-02 | work | 区域外不误派发、resize/挂卸后坐标与状态检查点一致 | V-02 |
| AC-03 | work | 双宿主只读证据接口项目/候选/模式/实际状态绑定准确，不干扰业务 | V-03/04 |
| AC-04 | work | 永久入口/支持边界/候选失效影响与AG026/027交接齐；无本机合同内未闭项 | V-05 |
| AC-05 | review | 独立逐AC/SD/实际源及反控复核pass，遗漏/延期/workaround/告警格式核齐，绑定r1与SHA | T07 |
| AC-06 | merge | 冻结触面并集/来源后落主，canonical与同号归档真实一致；不宣称双DPI/M1-ready | T08 |
| AC-07 | merge | 全输出摘要折Plan、fresh guard与自有树/分支/scope清理及retention对账完成 | T09 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 结果/写面 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续启动；065/066/025已交 | 核支持/字段/独立ROI与反例；唯一树scope，决定最小观测机制和真实命令 | AC-01..04 | [x] ✅ 4 pin SHA 全实测相符（host/window/cli/editor_projects）；D01答案＝`composite_portals_of` 把门户+clip合成进UI窗口面target，`rt.capture/probe_pixel(s)(host.ui)` 即真实最终合成（离屏/真窗同面，零新增访问器）；clip机制真相＝v2 host extent==clip结构性成立→scissor单独移除app层不可见（引擎层反控留prototype surface_lifecycle V-08），可检真实故障＝mount漂移（`set_portals…set_portal_mount`生产缝，offset/scale/clip同随错rect）；冻结设计`D:/autostack/plan-artifacts/plan-071/auto-ui/t01-frozen-design.md` |
| T-02 | work | T01 | 构造永久混合clip fixture及新增target；验证最终合成可观测，必要最小公开只读接面 | AC-01 | [x] ✅ fixture=`fixtures/node-mixed-regions/{dashclip,gaugeclip,meterclip}.at`（054 showcase副本+边界几何：strip edge quad x=104..128@120、gauge dot种子x=72+edge、dial近边界圆柱x=3@投影半宽3.31）；产物`ui-m1-clip-forensics`（world `9c18df00…`7056B/ui `9388f6e7…`，float绑定被生成器具名拒→int修正）；pilot consumer臂新增；原3份054产物未动；新target`tests/m1_native_evidence_support.rs` 9例全绿 |
| T-03 | work | T02 | clip/路由/resize/挂卸正负控，本机单档真实窗口与GPU错峰 | AC-01/02 | [x] ✅ 2D/3D合成clip正例（rect内边界几何可见/rect外1.5宽扫描线==干净参照且无边界色）；mount漂移反控必红+同副本精确修复回绿；内部缺画反控必红+真实状态变化回绿；边界路由（内缘仅推进命中者/rect外2px不派发/跨区域隔离/按钮消费不误动）；卸载后旧ROI点击落在真实新占位(p2)、重挂代次单调+旧代次具名拒绝、resize后重测bounds命中；真窗单档`--probe`（showcase fixture）exit 0、count=202、scale_factor 2.0、posted 6点击5键、idle零Tick |
| T-04 | work | T01/02；公共写面单写 | mixed/editor只读原生检查点/报告schema，操作配方接面，显式输出目录 | AC-03 | [x] ✅ `src/m1_evidence.rs` schema `autographics.m1-native-evidence/1`（run_id/项目+候选sha绑定/实际scale尺寸/真实ScaleFactorChanged/逐检查点读数/N1..N6/观察人/tier分层；validator：offscreen/posted/state-record永pending、native-manual缺人结论/观察人/单档具名blocked、auto_pass=true伪记录invalid、failure→fail；lib单测6/6）；mixed `--manual`扩展N1..N6+ScaleFactorChanged+逐事件检查点（legacy字段逐字保持；probe路径不变）；editor `--evidence-record/--evidence-attach`（smoke exit 0：document sha `f80459c7…`、attach按sha挂接、窗scale 2.0 [2400,1600]）；显式输出强制 |
| T-05 | work | T03/04 | 观察不干扰、错误/缺字段拒绝及真实触面定向回归 | AC-03/04 | [x] ✅ 不干扰：记录器开/关同业务序列终态+派发完全一致（永久回归）；伪auto_pass/坏schema/缺绑定/坏副本sha差异全部具名拒绝（测试）；定向回归：flow 7/7、node_editor_projects 7/7、pilot gates 7/7、`verify_plan054.py --controls` ALL PASS、m1 target 9/9、lib 6/6。**named红**：`authoring_probe_exit` asset/hierarchy两例"native fixture pointer occluded by another process"——master基线（stash本Plan全部bin/cli改动后重建）同红，plan-066收据同机已记1绿/2红flaky；非本Plan回归，未计入本Plan触面 |
| T-06 | work | T05 | 冻结SHA/Spec候选/AG026与027消费配方及候选失效清单 | AC-04 | [x] ✅ SD-01 §4增量+SD-02新Spec（worktree候选，merge落canonical）：冻结身份（base 126fe757/lang 30b396d72组内detached树/SDK 681 PASS/window runner d20fc737/authoring runner 1c1578bf/fixture shas）；AG026/027逐命令配方（regen/离屏门/--probe真窗/--manual人工/--evidence-record）；候选失效清单沿age-m1-acceptance §3不改原规则；未证边界具名（第二真实scale/N1..N6人工/20-34阶段→DEBT-001/002） |
| T-07 | review | T01..06实质完成 | 同树独立final，finding回work定向复核 | AC-05 | [x] ✅ 独立final子代理（新上下文，read-only两检出）：baseline完整（HEAD/diff 18文件全在合同内/无legacy ledger写/SDK verify PASS/4 pin SHA自算相符）；AC-01..04全部子代理自跑证据pass（m1 9/9、lib 6/6、flow 7/7、editor_projects 7/7、pilot gates 7/7、verify_plan054 ALL PASS、真窗probe exit 0 count=202 scale2.0、editor smoke、validator独立python重实现8控制一致、漂移反例独立计算余量宽）；named红自复现2/4=环境遮挡（probe自身WindowFromPoint检测器+plan-066同机flaky在案）非回归；F-T07-01..03 minor经91ffa61b修正、F-T07-R2-01经a20fa04d修正，final pass重绑r1@`a20fa04d` |
| T-08 | merge | T07 pass | 冻结门/来源、落主/canonical/归档，不自动tag发布 | AC-06 | [x] ✅ 2026-11-08：master无漂移（base=126fe757=tip）无需rebase；guard clean（Git Bash）；`git merge --ff-only codex/plan-071-dev`→main tip=`a20fa04d`（无merge commit）；canonical实算：`node-mixed-regions-lifecycle.md`=`0fc74890…`（含SD-01 §4；pre-diff `4753cd65…`与review收据一致）、`m1-native-evidence-support.md`=`d0231dbf…`，main与worktree字节一致；landing smoke on main：m1 target 9/9、flow 7/7、真窗probe exit0 count=202 scale=2.0（`landing-native/`）。observations：①verify_plan054 regen面在主检出红＝pre-existing环境条件（auto-lang master `7288db534`给auto-lang-core清单+2依赖行，pilot committed lock按pin `30b396d72`解析，`--locked`拒绝更新→exit101；本Plan lock零diff非回归；worktree面pin sibling下ALL PASS为合同权威run）——主检出pilot lock重解析属后续独立事项；②主检出其余WIP为他会话（docs/design、README、.next-id、archived/060、063/072/073等）未纳入未删除；③batch regression due check不适用（auto-ui无`.last-batch-regression.json`机制）；④landing artifacts不适用（本仓Rust库+experimental driver，无生产release binary/web bundle部署面）。未打tag/未push。 |
| T-09 | merge | T08 | 全输出对账/折摘要/guard/清理/登记 | AC-07 | [x] ✅ 2026-11-08：guard×2 fresh clean（Git Bash，ui-071/auto-ui 与 ui-071/auto-lang 均 exit0 无reparse point）；worktree `D:/autostack/.wt/ui-071/auto-ui` removed（`master..codex/plan-071-dev`=0 确认全落主）、branch `codex/plan-071-dev`（was `a20fa04d`）deleted、auto-lang detached 兄弟树 removed、空组 `.wt/ui-071` removed、worktree list 零 ui-071；scratch `D:/autostack/plan-artifacts/plan-071/` 整删＝12GB（t01-frozen-design/t03/t04/t07-review/landing-native 折入本文件§9收据＋target构建中间物随删）、无具名hold；registry `auto-ui/PLAN-071` → deleted-at-merge（delivery/archive/cleanup 记录）；输出对账：主检出 target（landing smoke 构建）为 gitignored 非Plan scope材料、无其他输出根；主检出剩余 WIP 全为他会话文档。 |

## 9. 复审记录

2026-11-08 / stage:new / auto-ui/PLAN-071:r1 / outcome:pass（规划合同静态交接，非产品或独立final）/ next:待后续启动T01。本机合同无需第二DPI设备；不等026/027外部门。0/9，未建树/scope、未实现/构建/原生操作/创建agent。

2026-11-08 / planning_validation:pass（纯文档，不是产品测试或独立final）。三份合同均0..10节，30任务/22项AC/15项V全部未勾；任务计数、AC到任务、Spec修改目标与拟新增入口建立任务可追；6节点新合同/已交前置DAG无环，依赖及新增本地链接可解析。号源在各仓.next-id.lock独占下核主/归档/注册树分配，UI072/AG028为本批分配后的counter（未来取号重核）。两债务保持OPEN、历史waiver原文未改；未创建071/026/027实施树或材料scope、未编译/跑测试/窗口/OS操作/构建演示，产品源码与归档合同未由本轮修改。

## 10. 待澄清事项

D01：T01核GPU capture与真实最终合成ROI关系；若现接面无法观测，先小型机制探证，不以空ROI或黑图凑绿。D02：现单档远程环境为已知限制，不阻本Plan但不解除AG026外部门。D03：若发现真实渲染/路由缺陷超出小型取证接面，保反例并由owner有限修订；不得在内容仓绕过。D04：两游戏是否易于试玩另见路线演示差额，不属于本债务支撑合同。

2026-11-08 / stage: work | plan_id: auto-ui/PLAN-071 | plan_revision: r1 | outcome: pass（execution_done；work-owned AC-01..04 全闭，AC-05..07 review/merge-owned 未领） | worktree_path: `D:/autostack/.wt/ui-071/auto-ui` | branch: `codex/plan-071-dev` | base_commit: `126fe7572e24bfea40ac09ec6cd7341b587d08e4` | code_commit: `88771287a164c7f0875183c6b5881ddc502ba0a4` | dependency_revisions: auto-lang `30b396d72`（AG025候选同版，组内兄弟树`D:/autostack/.wt/ui-071/auto-lang` detached）；driver `.sdk-source` 681文件（worktree内`prepare_sdk.py --from-snapshot` PASS）；外置`CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-071/auto-ui/target` | task_ids: T-01..T-06 | evidence: 离屏`cargo test -p autographics-auto-driver --test m1_native_evidence_support -- --test-threads=1`=9/9（C1 2D/3D合成clip正例+mount漂移/缺画反控必红回绿；C2边界路由+旧ROI/代次/resize；V-03/04/05证据面）；lib m1_evidence 6/6；flow 7/7、node_editor_projects 7/7、pilot gates 7/7、`verify_plan054.py --controls` ALL PASS；真窗`mixed_regions_window --probe`（showcase fixture，显式report入plan-071 scope）exit 0/count=202/scale 2.0/idle零Tick；editor `--evidence-record` smoke exit 0（document sha绑定+attach sha挂接）；fixture产物 world `9c18df00…`/ui `9388f6e7…`；runners d20fc737(window)/1c1578bf(authoring)；Spec SD-01 §4+SD-02候选已在树 | blockers: 无阻work交审项。named边界：①`authoring_probe_exit` asset/hierarchy两例红＝master基线同红（stash验证）+plan-066已记同机flaky"pointer occluded"，非本Plan回归；②`--manual`人工门运行时报告属AG026人工会话（本交付=编译+schema门+probe路径不干扰，符合C5不改auto_pass）；③第二真实scale外部门未闭（DEBT-001，本合同明确不闭） | next_responsible_role: review（T-07 同树独立final；review_scope: final；实现已停写于`88771287`）

2026-11-08 / stage: goal | plan_id: PLAN-071 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/ui-071/auto-ui` | commit: `88771287a164c7f0875183c6b5881ddc502ba0a4` | next: review

2026-11-08 / stage: review | review_scope: final | reviewer_identity/limitations: 独立 final-review 子代理上下文（父会话新上下文启动，无实现会话记忆；同机同 session host；非人工/非独立硬件；全部 cargo/python 串行单实例、`--test-threads=1`、单机单档 scale=2.0 未动显示；真窗 --probe 与 editor 窗各亲跑一次；对两检出只读、结束后核 clean；子代理 cargo test 重建过共享 target 的 debug 二进制→§3 runner exe sha 为交付时点身份，由失效条款覆盖） | plan_id: auto-ui/PLAN-071 | plan_revision: r1 | outcome: pass（3 minor findings，无 blocker/major） | worktree_path: `D:/autostack/.wt/ui-071/auto-ui` | branch: `codex/plan-071-dev` | reviewed_commit: `88771287a164c7f0875183c6b5881ddc502ba0a4` | base_commit: `126fe7572e24bfea40ac09ec6cd7341b587d08e4` | dependency_revisions: auto-lang `30b396d72`（组内 detached 兄弟树，clean）；SDK `--verify-only` PASS 681 文件 | spec_inputs: `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md`（pre-diff canonical `4753cd65…` 子代理自算相符；worktree 含 SD-01 §4）＋ `docs/specs/auto-graphics/m1-native-evidence-support.md`（SD-02 候选，与代码逐条核过无 over-claim） | acceptance_results: AC-01 pass（9/9 自跑；ROI 独立性/合成面/生产缝反控/2D+3D/fixture 语义副本/原054产物零diff 全部独立核）；AC-02 pass（边界判别/旧ROI落真实新占位/代次具名拒绝/resize 重测命中——歧义实现均会被抓）；AC-03 pass（lib 6/6；mixed diff 12 hunks 全 manual 分支/加法，probe 非manual分支未动；editor smoke 子代理自跑 exit 0，document sha 与 t04 相符）；AC-04 pass（fixture/canonical/runner shas 自算相符；runner exe 现值不同＝子代理重建所致，时点身份）；AC-05 pass（named红子代理自复现 2/4，错误文本来自 probe 自身 `WindowFromPoint`-pid 遮挡检测器 authoring_probe/mod.rs:523，diff 未触该路径，plan-066 归档收据同机 1绿/2红 在案→环境红非回归；flow 7/7、node_editor_projects 7/7、pilot gates 7/7、verify_plan054 ALL PASS；真窗 --probe exit 0 count=202 scale 2.0 报告落 t07-review；validator 以子代理独立 python 重实现对照 rust 8 控制全一致） | pending_closeout_gates: merge-owned 统一门（T-08 冻结门/T-09 对账清理）未领 | findings: F-T07-01（minor,AC-03）legacy `mode` 值改名未在 SD-02 注记；F-T07-02（minor,AC-04）probe 报告无 runner sha 自绑，身份绑定分层需明示；F-T07-03（minor,AC-04）§4 配方隐含 cwd 未写明 | evidence: `D:/autostack/plan-artifacts/plan-071/auto-ui/t07-review/`（verdict.md＋子代理自跑 editor-evidence.json＋reports/mixed-regions-native.json）；独立漂移反例计算：extent×2 下 edge 泄漏带 parent fx 1.733..2.0，扫描线 1.74..1.90 有 9 个采样在带内（余量宽、相位不敏感）→非 thin-margin | next: work（三 minor 修正后 bound re-review）→ merge

2026-11-08 / stage: review（repair re-review ×2） | review_scope: final（同子代理上下文续审；bound re-review） | reviewer_identity/limitations: 同前独立 final-review 子代理（read-only git show/diff/sed；两次各 Item 1..6 逐项核） | plan_id: auto-ui/PLAN-071 | plan_revision: r1 | outcome: pass（final pass 重绑至 r1 @ `a20fa04db2839cf3bbdab5898fcf86f1b2a371ce`） | worktree_path: `D:/autostack/.wt/ui-071/auto-ui` | branch: `codex/plan-071-dev` | reviewed_commit: `a20fa04db2839cf3bbdab5898fcf86f1b2a371ce` | base_commit: `126fe7572e24bfea40ac09ec6cd7341b587d08e4` | dependency_revisions: 同前（auto-lang 30b396d72；SDK 681 PASS） | spec_inputs: SD-02 候选经 91ffa61b（F-T07-01..03）→ a20fa04d（F-T07-R2-01）两轮 doc-only 修正 | acceptance_results: round1 items 1/2/4/6 pass、3 partial、5 fail on --manual clause → F-T07-R2-01 bounded 修正；round2 全项 pass（§2 身份分层逐句对回 mixed_regions_window.rs:300-308 与 authoring/cli.rs:542-545；91ffa61b..a20fa04d 恰 1 hunk；§0/1/3/4/5 字节不变；worktree clean）。reviewer 确认 88771287→91ffa61b→a20fa04d 零源码变更，其在 88771287 的全部亲跑证据继续 binding | pending_closeout_gates: merge-owned T-08/T-09 未领 | findings: F-T07-R2-01 已闭（identity-unbound 限定 --probe；--manual 自证 candidate.runner sha；悬空括号修复） | evidence: 子代理两次 round 报告（verdict 续记于 `D:/autostack/plan-artifacts/plan-071/auto-ui/t07-review/`） | next: merge（T-08/T-09）

2026-11-08 / stage: merge | plan_id: auto-ui/PLAN-071 | plan_revision: r1 | outcome: pass（prepared/landed/canonical_verified/archived；cleaned 见后续回执） | delivery_commit: `a20fa04db2839cf3bbdab5898fcf86f1b2a371ce` | reviewed_commit: `a20fa04d`（=delivery，无rebase：master base=tip=`126fe757`，dev为直接后代，ff-only） | canonical_verified: `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md` sha256 `0FC748907406B3F86851031D664B5531DEA31329ACA353ACE1098F4D18934D9A`（SD-01 §4增量；pre-diff canonical `4753CD65…` 与review/AG候选收据一致）；`docs/specs/auto-graphics/m1-native-evidence-support.md` sha256 `D0231DBF991B02D888FAB1402FD76064C7523AE8B5BE38397ECED65372DF28DE`（SD-02新增）；main与worktree `diff` 字节一致 | landed: `git merge --ff-only codex/plan-071-dev`；main tip=`a20fa04d` 无merge commit | landing_smoke_on_main: `cargo test --test m1_native_evidence_support -- --test-threads=1`=9/9、`--test mixed_regions_lifecycle_flow`=7/7、native `--probe` exit0 count=202 scale=2.0（`plan-artifacts/plan-071/auto-ui/landing-native/`，本回执折入） | archived: `docs/plans/archived/071-age-m1-local-clip-and-native-evidence-support.md`（本提交，untracked plan 经 move+add 落版本控制，PLAN-104 UAT-K13 面） | observations: 同T-08行①..④（regen面主检出环境红具名非回归；他会话WIP未动；batch due不适用；无生产部署面） | pending: cleanup（guard→worktree/branch/lang兄弟树/组目录→scratch整删→registry）+T-09 | blockers: none | next: cleanup

2026-11-08 / stage: merge（cleanup） | plan_id: auto-ui/PLAN-071 | plan_revision: r1 | outcome: pass（cleaned，T-09 完，9/9） | delivery_commit: `a20fa04d`（master tip） | archive_commit: `e7443b07` | guard: `"C:/Program Files/Git/bin/bash.exe" D:/autostack/wt-guard.sh` ×2（auto-ui 树＋auto-lang 兄弟树）= clean exit0（避开WSL bash误报） | worktree_removed: `git worktree remove D:/autostack/.wt/ui-071/auto-ui`（先核 `master..codex/plan-071-dev`=0 全落主）；`git branch -d codex/plan-071-dev`（was `a20fa04d`）；`git -C auto-lang worktree remove D:/autostack/.wt/ui-071/auto-lang`（detached `30b396d72`）；空组 `.wt/ui-071` removed；`git worktree list` 零 ui-071 | scratch_deleted: `D:/autostack/plan-artifacts/plan-071/`（resolved绝对路径；仅本仓scope；parent无别仓同号目录）整删＝12GB（含 CARGO_TARGET_DIR 构建中间物）；证据已折入本文件（§8 任务行证据＋§9 work/review/merge/cleanup 四收据含全部命令/退出码/关键值/SHA：base 126fe757、code 88771287、fix 91ffa61b、final a20fa04d、fixture world 9c18df00/ui 9388f6e7、canonical 0fc74890/d0231dbf、pre-diff 4753cd65）；无具名 unresolved-issue hold | registry: `.retention/policy.json` `auto-ui/PLAN-071` scope+plans 条目 → deleted-at-merge（delivery `a20fa04d`、archive `e7443b07`、cleanup 完整记录） | landing_artifacts: 不适用（本仓 Rust 库＋experimental driver，无生产 release binary/web bundle 部署面）；batch due check: 不适用（auto-ui 无该机制） | 主检出状态: 本Plan相关零残留（plan文件已入 archived/ 并提交）；其余 WIP 为他会话未动 | blockers: none | next: done
