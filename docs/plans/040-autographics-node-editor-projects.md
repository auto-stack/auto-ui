---
plan_id: PLAN-040
status: drafting
feature_name: AutoGraphicsNode混合维度编辑工程与项目化验收
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-019, auto-ui/PLAN-024, auto-ui/PLAN-039]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-editor-projects.md]
touched_goals: [GOAL-008]
affects: [autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-040：Node混合维度编辑工程与项目化验收

## 0. 变更摘要

交付由普通Node/use生成的纯声明、混合二维/三维编辑工程，保G04原全部对象/编辑能力，并使项目自己的场景经历可核验的导航/选择/拖动/取消/Undo/Redo/保存重开/SaveAs/Play还原。优先复用039已能生成behavior:null的双出口路径，不能未先证就重复造混合渲染或VM。

后继[AG018](../../../ag-examples/docs/plans/018-autographics-gallery-node-editor-tutorials.md)负责两教程内容；[AG019](../../../ag-examples/docs/plans/019-autographics-m2-unified-release-validation.md)负责最终同版联验。本计划只公共引擎/最小永久夹具与受检消费交接，不认领教程或M2交付。[共同设计](../design/autographics/m2-editor-and-release-convergence.md)。drafting/r1、0/10，暂不实施。

## 1. 目标

1. 同一纯声明工程能保原2D/3D对象和具名组件/use，view-2d/view-3d可共同出现，behavior:null、无用户VM/假角色/运行相机/空Tick；能力分类仍为编辑工具E。
2. 项目感知的编辑验证针对实际载入项目、实际对象与来源，允许稳定ID/行映射但不硬编码默认样本或重新排序来冒充保真。
3. 拷贝/SaveAs成可编辑副本后，编辑接受态可保存搬迁重开；作者源与生成基线字节不被拖动或保存暗改，再生不静默覆盖用户编辑副本。
4. 原2D20阶段、3D34阶段编辑义务与反例、实际Auto拖动/真实GPU/持久化/取消/资源owner证据分层齐备，消费身份可固定给018/019。

非目标：通用AST反向源码补丁/可视化编辑回写Auto、全局HMR/动态拓扑/ECS、任意生产UI嵌3D/多屏变体、游戏玩法/物理或新运行API、跨端/生产UI融合。039的有限查询档不自动扩为所有Node的混合能力；纯场景可用既有路径就不为名称另建profile。

## 2. 架构方案

普通Node/use纯声明→锁定lower及scene/identity/closure→只读可再生基线→公开Host打开并显式SaveAs/复制为独立编辑工程→原Editor/实际Auto交互计算/Renderer/Storage接受态。原编辑副本再开恢复用户变更；重新生成基线是另一显式动作。运行存储不要求照搬声明树。

拟唯一实施及顺序复审树`D:/autostack/.wt/ui-040/auto-ui`、分支`codex/plan-040-dev`；目前不创建。共享parser/load/CLI/storage/viewport短窗单写；035/039已交付、不得重建已清旧树，不接管038/OS056及其它owner。GPU/窗口/音频重门单实例；OS抢焦点自动化暂停政策保持。

## 3. 技术栈

已核pilot `src/{load,main,lower_components3d,lower_queries3d}.rs`：039 dual dialect只对该profile启用，root shape2d标记附于三维行后；`lower_components3d`的has_behavior仅取stateful/Life，故纯声明有behavior:null路径，不能写成“所有混合场景尚未支持”。待先证的是完整原行/对象兼容、纯场景独立编辑交接和项目感知验收，不是重写渲染。

driver现有`src/authoring/{interaction,storage}.rs`、`host/viewport.rs`与`tools/check_editor_viewport{2d,3d}.py`、相关tests；ag-examples `tools/verify_examples.py::verify_g04`现为2D实际项目probe、3D默认文档probe+项目smoke分列（脚本注释17旧阶段，实际checker34）。拟新增pilot `tests/node_editor_gates.rs`、driver `tests/node_editor_projects.rs`、永久双维最小夹具和`tools/verify_plan040.py`，T-02冻结真实入口；不得将测试专用访问伪装公共用户操作。

## 4. 需求分析与背景调查

用户2026-11-03要求继续提出和规划下一批，允许合同/设计/导航/auto-lang路线簿记，未授权实施/建树/构建/GPU/OS自动化/agent。本地取号040（038主题在途、039已交付，不重用号）。观察auto-ui `f83fd06f1b924fce98bd1f821bee9d413dfa8e7f`，ag-examples `139e41d0da3c02af59bbd1009f14eaffe7134cd4`；AG016/017均drafting/r1/0-10且无实施树，当前迁移19/22。

| 来源 | SHA256 / 交付 |
|---|---|
| `docs/specs/auto-graphics/node3d-queries-and-mixed-markers.md` | `117bb3da3cb2e2a5501ea1bbbb416fa108265b7990ec392bffa4265bb9da0220`；039交付8f577a343f61122bf9783c7fd174b2b22202a390 |
| `docs/specs/auto-graphics/node-authoring-consumption.md` | `f6689bf162eb992830dfa3a43f16168f2d1ca6b7470128ebbb51579df75667e3` |
| `docs/specs/auto-graphics/node3d-foundation.md` | `9451cf0f3c6a82505caa771b54728f1a6fb06a2ff74343052dd164dd2aee155a` |
| auto-lang `docs/specs/auto-graphics/editor-viewports.md` | `d689b70bbf5a81a2d8a230cbd635fc9c03a3c96defe34ec6e46ac3dba8b387a6` |
| auto-lang `docs/specs/auto-graphics/viewport-interaction.md` / `authoring-session.md` | `4e9dc609a80fdbcc45924df70614298ac164d60b51dcbeaf3675b3a5e38c8e8b` / `63e9876ff74174ac485e2f23d14ab87f4665ebf63629015c28287325d45661b6` |

035/039落主与canonical可核；final收据均注明同实现会话复核，不称独立上下文pass。本计划独立final验证自身及实际复用交叠面，必要相关前置缺证具名交原owner，不改历史或自行豁免。039 metadata current_step6而merge收据已delivered，按真实落主/canonical/清理证据观察，不凭计数回退状态。

## 5. 详细设计

### 5.1 支持域与最多两条先证路径

T-01盘点原G04：2d工程含6对象（circle/quad、player/mob、2个2d helper），3d工程含5对象（circle/quad、player/mob、cylinder helper），都本就双维、无group/asset/behavior。必须保所有kind/translation/scale/yaw/pitch/rgba/visible/parent、二维数组绘制序/三维几何与选择语义；原ID可显式双向映射但展示名/作者symbol/运行ordinal不能混用。

T-02最多两候选：A复用039静态dual lowering及024/019叶构造，解决对象/映射和无用户脚本VM公共编辑交接；B仅在A有具名正确性/支持域缺口时增加显式`editor-scene`有限入口或等价公共适配。两者都必须真实生成→公开loader→实际Editor/Auto拖动→GPU→保存/新根重开，不以手制JSON或空VM代替。“无用户VM”指纯声明无需用户游戏脚本会话，不禁止编辑拖动使用既有真实Auto计算，也不擅自改变显式Play的原Builtin路线；两类会话/owner在验证中区分。冻结所选CLI、角色为空/behavior:null、支持对象/同维parent/use域、行/ID映射和诊断；超域具名拒绝，旧profile字节/拒绝/数值不变。若A已完整满足，正式交付可以只有公共消费收口/测试/文档，无人为新语法要求。不得删除原2d辅助对象或换player/mob。

### 5.2 编辑与作者源的所有权

原作者模块/closure/身份与生成基线不可被编辑器动作改写。使用已验证公共Open/SaveAs在用户明确目标创建独立编辑副本（若现有公开流程有缺口，在本Plan修公共存储/导出，不加G04特例）；对副本拖动/Undo/Redo/Save后，新根load_project/inspect和真实Host像素应恢复编辑结果。编辑状态不伪称仍与未修改作者源字节等值；基线再生控制与副本持久化两套身份有明确父源/接受revision，不能伪造Ready回执。用户明确再生基线不会悄悄抹去副本，不承诺反向写Auto。

保存失败（写/rename/闭包prepare/确认冲突）、取消/旧revision/resize/blur/late pointer等沿原Storage与手势合同；last-good/原用户文件保持或准确报告已提交/残留与恢复，不以单个父进程exit0代终态。SaveAs后资源/源闭包可迁移；本G04无外部asset也要对实际有闭包最小合法夹具证明不丢依赖（复用现有GLB fixture，不扩大资源格式）。

### 5.3 项目化编辑验证与原域

将实际目标项目路径/场景身份/对象语义映射传到独立checker，原probe默认文档路线仍作旧正控，不能悄悄替换。2D20与3D34各阶段逐项核，3D原grid遮挡/auto-drag/save-reopen/focus-cancel/play-restore/cross-button-capture等集合与部分执行必失败；Node对应项目本身必须有导航/选择/拖动/保存关开与SaveAs证据，不只默认文档probe绿。2D旧probe本无保存阶段，新增项目持久化用例不能据旧20阶段推定已测。

OS自动化暂停时仅执行不抢焦点的受支持Editor消息/实际Auto/离屏GPU路径，记录其与真实OS输入的证据层次。OS阶段保持pending，手工按原阶段完成且绑定新工程/runner也可作为真实交互取证；不得自动抢焦点、不得把未证标Ready或因无法OS验证删验收。

原viewport800×380/800×400、导航夹界/投影/grid128线与16KiB辅助域、4096quad、snap/3px阈值、128场景节点及原Auto交互/运行/资源预算保持；纯声明不以免费VM名义开新额度。保Play/Stop恢复编辑导航/选择，实际资源当前/history/外部pin owner最后释放和20轮打开/编辑副本/关闭检查，无每次导航重传静态mesh。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-editor-projects.md | 有限Node静态/查询混合路径→正式核定纯声明混合编辑工程/基线与副本身份/项目化编辑证据/支持拒绝域 | G04内容可固定消费，非反向源码同步 | AC-01..06 |

## 6. 测试设计

以下拟命令/新target由T-03/04建立，不是已跑：`python experimental/autographics-node2d-pilot/tools/verify_plan040.py --controls`；pilot `cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node_editor_gates`；driver `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test node_editor_projects -- --test-threads=1`。T-02冻结项目化checker参数；原check_editor_viewport2d/3d只在OS准入允许时运行，不在本规划跑。历史build_runner_proof.py含plan857固定输出，未改为自有scope前禁止直接调用；可复用等价已有来源核验设施，不为此泛化构建平台。

| case | AC | owner_stage | 入口与区分oracle | 证据边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | gates/verify：普通use双维纯声明、两同定义纯视觉组件、behavior:null且无用户VM；原G04全部对象逐值/ID映射/二维等alpha序；非法跨维parent/重复ref/不支持动态输入具名拒绝 | 不要求任意UI嵌套或无限组件域 |
| V-S02 | AC-02/03 | work | node_editor_projects：目标项目identity/实际Auto drag+GPU命中/手柄/网格/遮挡/导航/取消/UndoRedo；替换默认场景或拿错row必须红 | 原20/34阶段逐项，OS与消息注入分列 |
| V-S03 | AC-03/04 | work | 基线hash不变、独立副本拖后保存/关开/SaveAs/新根Run&Stop、实际新位置/像素；坏闭包/旧revision/取消/保存故障→保原或准确残留→修复重开 | 不反向写源，不以identity外观充保存成功 |
| V-S04 | AC-04/05 | work | 20轮新副本/close/真实VM和mesh/history/外部pin owner释放；导航无静态mesh重传，原导航/grid/预算边界及溢出拒绝last-good | 不将合法外部pin当泄漏、不删owner制造零值 |
| V-S05 | AC-05/06 | work | A→B仅closure冷再生/完整编辑工程搬迁，错generator/SDK681/模块/场景坏字节拒绝→精确修复→同入口可用；默认文档与真实项目差异控制 | 旧default probe绿不代真实项目绿 |
| V-S06 | AC-01/05/06 | work | 019/024/039实际受影响冻结产品/命名gates，parser有改则dialect `node2d_pilot`定向；来源/SDK两图/SD与下游018准确消费交接 | AG全套仍里程碑，根生产UI无改不跑全库 |

work产证；review同一权威树不同上下文独立final，逐AC/diff/SD/遗漏延期workaround/告警、绑定r1/提交/工具/输入，finding回work同员复核。未授权本会话创建agent或第二树；同实现会话重跑不充独立final。merge只冻审后定向并集，复用未变有效证据，不日常全AG。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通Node/use的纯声明混合工程无假角色/用户VM，支持/拒绝域与旧档边界清楚 | V-S01/06 |
| AC-02 | work | 原全部G04对象保真、稳定映射、项目自身导航/选择/实际Auto拖动及像素正确 | V-S01/02 |
| AC-03 | work | 原20/34逐阶段证据和部分失败敏感、取消/UndoRedo/保存关开/SaveAs；必需OS项未证保持pending并阻止本AC关闭 | V-S02/03 |
| AC-04 | work | 基线/副本所有权分明、故障last-good/文件保护/恢复、新根正确重开，无反向改源或静默覆盖 | V-S03/04 |
| AC-05 | work | 原预算/生命周期/实际owner与来源SDK681两图/冷A→B/旧档命名受影响门保持 | V-S04..06 |
| AC-06 | work | SD候选、准确CLI/工具/证据范围/所需独立来源与018消费交接完整 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/实际project与反例，finding闭环；未证必需门不伪pass | T-07 |
| AC-08 | merge | 冻结门/来源核后实际落主，canonical字节/归档/准确消费身份和剩余项齐 | T-08/09 |
| AC-09 | merge | fresh guard清树/分支/空组，全部输出对账/摘要折Plan/删自有scope，具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；019/024/039已交付 | 核原场景/对象/九义务/20及34/owner/OS政策和来源；唯一树/scope登记 | AC-01/05 | [ ] |
| T-02 | work | T-01 | 最多A/B完整项目链先证；复用能满足则不加profile，冻结支持域/CLI/行映射/基线副本/实际验证参数 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 最小公共lower/编辑消费收口（仅确有缺口）、永久双维纯场景与gates、旧档边界保持 | AC-01/02/04 | [ ] |
| T-04 | work | T-03 | 项目化编辑checks/实际Auto+GPU/保存/取消/故障负控和原阶段映射 | AC-02..05 | [ ] |
| T-05 | work | T-04 | 新根SaveAs/冷A→B/恢复/20轮owner/受影响旧档冻结门，未证OS项具名 | AC-03..05 | [ ] |
| T-06 | work | T-05 | SD候选/准确固定消费交接/完整证据与待闭项，不以新入口名充能力 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同树独立final与finding返work后受影响复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后落master | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/归档/消费身份/导航核验，不写派生JSON | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard、所有实际输出对账/清理、摘要/具名hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-03 / stage: new / plan_id: auto-ui/PLAN-040 / plan_revision: 1 / outcome: pass（可审阅合同，用户启动后T-01可进；T-03需T-02全链go） / next: work T-01。0/10。现有039 has_behavior:false路径已从代码确认，完整G04纯场景/项目化编辑尚未先证；未重复实现/未运行新门。只规划，无树/源码/素材/探针/canonical Spec/材料scope/构建/GPU/OS自动化/agent。

2026-11-03本批规划预检（非实施/独立final）：UI040/AG018/AG019均0..10节、drafting/r1/0-10、10T/9AC/6V-S/1SD，work6/review1/merge3，十任务未勾；依赖DAG无环，040不等016/017，018完整迁移等040，019候选等040、完整同版消费等016/017/018实际交付及必要独立证据。独占取号后两仓活动/archived/archive本地号唯一，UI next041、AG next020。289个所涉本地引用核验286可解析，余3为存量非AG主题036/037导航断链（含共享索引他会话WIP），不属本批、不代写修复；新增/修改AG引用可解析，历史attachments未作为新证据。原verify_g04九require/2d20与3d34及全部原双维对象逐项承接，明确039既有has_behavior:false静态路径与040尚待全链先证的边界，未把它误判为全无混合能力。按metadata/generated/Node活源实算19/22，余g04两维/g05三维；`python tools/gallery.py validate --all`退出0，12逻辑项/22入口/许可一致。本轮AG016/017合同字节和现有源码/消费锁/生成物保持；未创建新树/材料scope/探针/素材/提案canonical Spec，未构建/GPU/OS自动化/创建agent/发布/打tag。规划文档白名单空白检查通过；以上仅规划结构/现有目录验证，不是新作者编辑、统一候选或M2验收go。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 既有039静态混合路径能否保原全部对象/映射和编辑副本消费 | T-02全链先证后选A/B；不在未证前硬规定新profile或宣称已有完整支持 |
| D-02 | 项目输入/OS20-34阶段准入 | T-01核政策，T-04逐项实际取证；自动化暂停保持，无法取证具名pending不删项 |
| D-03 | 同会话final历史及批量回归D-05 | T-01核相关独立证据与批门owner；本Plan独立review不可省，旧D-05不在此偷偷认已闭 |

材料开工唯一`D:/autostack/plan-artifacts/plan-040/auto-ui/`并登记owner/用途/在途；临时探针/日志/包/外置target/外部消费者归此scope。merge摘要折Plan后全部实际输出对账/删除scope及构建物，具名未解决问题才可持有（owner/理由/关闭条件）。禁止attachments/第二暂存区/链接，清树先fresh `bash D:/autostack/wt-guard.sh <路径>`，不动他仓同号材料和其它WIP。
