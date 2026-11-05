---
plan_id: PLAN-054
status: drafting
feature_name: AGE混合区域组件与生命周期公共接线收口
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-048, auto-ui/PLAN-049]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-mixed-regions-lifecycle.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver, autographics-prototype]
---

# PLAN-054：混合区域组件、条件挂卸与原生接线

## 0. 变更摘要

为M2/AG021补齐049交付后仍缺的公共混合呈现链：按产物/实例身份装配任意具名区域，普通use拆分/多实例，条件隐藏卸载、陈旧输入与资源寿命、真实布局/DPI/焦点验证。旧048/049 profile字节与支持域保持，新版本显式选择。设计见[最后缺口与下一批](../design/autographics/m2-closure-and-m3-first-wave.md)。本轮仅规划，drafting/r1/0-10。

## 1. 目标

- 区域装配/路由/挂载由声明和实际实例表驱动，改App/Node/区域名或交换2D/3D声明顺序不破行为；去除新公共路径对REGION2/REGION3/固定s2d/s3d样例布局的依赖。
- 普通use组件内的图形出口/区域可实例化，两个同定义实例状态隔离；条件关开/卸载一实例不破其它合法owner，重绘/多出口不追加Tick。
- 真实窗口键鼠、Tab/Enter、焦点/捕获、resize/DPI、坏件修复下一Run及原预算/来源链成立；独立final核新候选和复用048/049必需能力，供AG021固定消费。

非目标：重开或倒签已归档048/049、生产全量UI/主题、float或坐标payload全面扩展、所有场景原语/复杂游戏混合、Schema/ACE接入、M3地图、改AG021原分母或自动更名/发布。

## 2. 架构方案

新增显式混合组件profile/版本化产物（名称/版本T-02冻结）→模块/实例与活区域表→共享Auto状态接受快照→metadata驱动MixedHost/CompositionInput/SurfaceRuntime。一个状态owner可被多个呈现观察，但不同组件实例不共用同名字段；不强迫单文件或新增业务VM来躲预算。

拟唯一树`D:/autostack/.wt/ui-054/auto-ui`，分支`codex/plan-054-dev`；必要auto-lang/auto-down sibling只读固定源，无链接。T-01核其它owner写窗（UI052/053等独立路线不接管）；GPU/原生焦点单实例。本轮不建树/scope。AG021完整接线等本Plan交付，不追开发tip。

## 3. 技术栈

已存在路径：dialect `src/node2d/{ast,parse}.rs`；pilot `src/{load,lower_node_ui,lib,main}.rs`；driver `src/{node_ui,node_mixed,node_mixed_host}.rs`与`bin/{node_ui_window,mixed_ui_window}.rs`；prototype `core/composition`、`host/{composition_input,surface_runtime}.rs`。沿固定SDK681/两个独立Cargo依赖图，不换SDK、不造解释器，Rust仅做通用解析/装配/输入/资源，业务全部由Auto model/on执行。

## 4. 需求分析与背景调查

用户2026-11-05授权评估M2余量并规划下一步；未授权本轮实现。只读基线auto-ui e02ca9f8、AG f44f7e8；048 delivery cd1e161a、049 b7f01267为master祖先且已归档/清理；049同会话final记录不是独立上下文。本Plan不改旧记录，AC-07对新候选及必需复用部分重新独立核验，历史限制保持可追。

已读`docs/specs/README.md`、`auto-graphics/{node-basic-ui,node-mixed-ui-2d-3d,node-authoring-consumption}.md`与AG021/019 canonical。源码事实：`parse_mixed_artifact`拒非root；`enqueue_region_click`按region名找且实例固定0；`MixedHost::absorb/refresh_mounts`依赖固定区域常量/两个surface；049 Spec显式不支持区域条件隐藏/卸载代次。AG021 G-M07/AC-04则仍要求这些行为。此为本轮静态核查，未跑新失败探针。

SHA256前16：049 Spec8967cc6c8cf87f0a，048 Spec7bda6f53faf16cae，node_mixed.rs d1e73a4296da2d4a，node_mixed_host.rs5aaf0c448cd7a9be，lower_node_ui.rs2893bb223b55754c。取号发现counter050落后于archive050/051及注册UI052树的052/053合同，锁下核全部本仓号取054/055，next056；不复用外仓OS同号。

并发现场更新：AG021原owner已在唯一ag-021树完成T-01/02准备、记录executing/r2/2-10，T-03等待054。其§9/§10及材料`t01-inventory.md`/`t02-support-matrix.md`报告049两项实测问题：bool文字槽≥2按绝对槽偏移写半缓冲可能越界；父UI不重绘时子场景未重新采样。报告是他owner证据，本轮未重放确认；T-01核输入/源码，T-02/05须具名重现、修通用接线并验证真实子场景自主Tick。不能仅让UI每帧假变化掩盖更新丢失。材料随其merge可能删除，必要结论以AG021计划§9/§10为持久入口，不依赖长期保留原scope。

## 5. 详细设计

T-01做原合同→实际支持→AG021原义务映射，确认root命名/区域instance/条件/输入/容量具体路径；区分新功能缺口、历史同会话review、原生尚未证、019原OS20/34与批门等不同owner。历史必需门不能因本Plan或019归档被静默免除。

T-02最多两种等价全链机制先证：导入两份同定义图形组件和一个异维组件（实例总量≤8、区域活总量≤4），非默认App/区域名，真实UI按钮改Auto状态→各区域真实GPU变化；卸载其中一个、其余继续、旧事件拒绝、恢复新代次正确。可选“纯view-2d/view-3d组件作为UI嵌入”或等价的组件内具名区域机制，但必须普通use/实例状态内聚，不能展开成特定示例根变量。冻结最小语法/身份、父子props/msg、条件寿命、限额/错误/Run代次、产物/ABI/profile与回退；失败needs_replan，不删AG021义务。

新公共路径不按App/Node/区域字符串分派；身份包含Run/实例路径/区域局部名/代次。kind只决定2D/3D adapter，实际layout rect决定viewport/scissor/逆坐标。区域注册/卸载与事件路由在接受状态/图提交边界一致；隐藏后不保留可命中僵尸portal，释放最后owner的target/content，另一实例保状态。UI控件消费的鼠标/Tab/Enter不泄漏到图形；失焦/卸载取消held/capture；旧Run或旧代次事件拒绝。改名/重排/恢复后的次次读入都必须真实运行。

条件保留同一Node仅撤呈现与条件销毁Node是两种语义，T-02沿现有Node寿命合同明确各自Start/Tick/资源owner，禁止用重绘重启业务掩盖状态。先支持M2所需有限2D shape/3D几何，不借此承诺任意场景/角色模型/区域内所有节点构造。

容量拟保持048/049既有8实例（含根）、32元素/def、4文字槽×64B、队列64、1000 VM steps/回调/累计100k；新增活区域**全应用合计≤4**、每区域8场景元素、边长≤640，防实例化复制配额。旧game profile32credits/物理32/128行独立保持，steps/credits/ms不混算；超界正常拒绝，合法边界必须绿。新字段/身份与v1严格隔离，旧048/049生成产物/schema与原有具名拒绝语义保持；确认的文字越界/采样等缺陷可做必要通用修正，注明兼容面并跑定向回归，不能把已知bug当必须保持的语义。任何修正都注明准确影响，不改旧锁伪造hash。

原生证据必须走新通用Host的真实键鼠/DPI通道；posted/离屏可以保机制证据，不能冒称native SendInput/物理输入已验。沿现OS政策用具名手工操作取证或另获授权后自动化，未拿到则AC未闭。本Plan不授权抢焦点批量自动化。旧编辑OS20/34由AG021原M2矩阵按既有owner与入口核，不用新UI点击绿代替。

拟新增永久门：pilot `tests/mixed_regions_lifecycle_gates.rs`、driver `tests/mixed_regions_lifecycle_{flow,storage}.rs`、`tools/verify_plan054.py --controls`（均T-03..06创建）。完整来源/closure/新旧版本错收/坏件恢复下一Run和原生操作配方必须可独立复验。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-mixed-regions-lifecycle.md | root固定混合切片之外，新增显式组件/实例活区域、挂卸代次/原生输入/身份预算；旧v1保持 | 关闭AG021已存在义务缺口，不倒改049 | AC-01..06 |

## 6. 测试设计

执行者自检后同树独立上下文final，复审需实际角色/版本绑定；同会话重跑仅记自检，不能充AC-07。本轮不创建agent。只跑受影响case/必要职责组，driver `--test-threads=1`、GPU/焦点单实例；正式并集依真实触面，不为merge全跑AG或改原条件。

| case | AC | owner_stage | 输入/入口与可区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | gates/flow正常模块与两个同定义实例；改App/区域名、交换声明顺序后真实点击A×2/B×1各状态2/1与正确2D/3D像素，已去固定名路径；文字槽0..3布尔真/假正常读，UI文本不变而子场景Tick仍更新像素 | 旧固定夹具不能代新改名输入；无Rust业务 |
| V-S02 | AC-02/03 | work | 条件隐藏/销毁/重挂、共享观察、N Tick=N推进、另实例不停、陈旧Run/代次事件丢弃；实际target/portal/owner收支与状态对照 | 模块/视图不复制配额或生命周期 |
| V-S03 | AC-03/04 | work | 新通用Host真窗口键鼠/Tab/Enter/失焦/resize与至少一组DPI对照，区域内外命中和scissor独立oracle；按钮已消费输入不穿透 | 原生/posted/离屏分层，缺必需原生证据不pass |
| V-S04 | AC-04/05 | work | 8实例/4区域等合法边界与+1拒绝；循环/错kind/旧代次/零或非有限尺寸/坏资源，旧应用保可用，精确修复下次Run恢复 | 保两图/旧SDK681与各profile单位 |
| V-S05 | AC-05/06 | work | verify_plan054/storage A→B只闭包冷再生/启动，改模块旧Ready失效，v1/v2错收拒绝，坏manifest修复后实际读Run | 不借生产者根/target/任意tip，不改旧回执 |
| V-S06 | AC-01/06 | work | 旧048/049必需触面独立重核/准确版本对照＋20轮新Run寿命；SD与AG021消费配方，旧review/OS/批门逐项明示 | 不倒签旧独立复审，历史保留不自动豁免当前必需门 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 全链先证/普通模块和metadata公共装配，不依固定样例名，源/接口冻结可审 | V-S01/06 |
| AC-02 | work | 同定义区域实例隔离、事件/绑定双向、一次推进与普通use成立 | V-S01/02 |
| AC-03 | work | 条件挂卸/代次/捕获/焦点和真实layout/DPI正确，必要原生证据齐 | V-S02/03 |
| AC-04 | work | owner/target/队列/各预算与故障恢复有合法/超界实测，无静默抬额 | V-S03/04 |
| AC-05 | work | 冷消费/源身份/坏件恢复下一Run与旧profile兼容、版本隔离成立 | V-S04/05 |
| AC-06 | work | 旧必需触面对照/具名边界、SD候选与AG021精确配方完整，不宣称M2已完成 | V-S05/06 |
| AC-07 | review | 独立上下文在同树冻结候选逐AC/SD复核pass，含048/049复用面及新关键反例 | T-07 |
| AC-08 | merge | 审后冻结门/源核落主，canonical/同号归档/明确实际消费commit齐 | T-08/09 |
| AC-09 | merge | fresh guard/全输出对账/摘要折Plan/清自有树和scope构建物、hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续另行启动；048/049已落主 | 原合同/源码/缺口/owner/旧review和原生政策盘点，核AG021所报bool高槽/独立采样反例，唯一树/scope登记 | AC-01/06 | [ ] |
| T-02 | work | T-01 | 最多两全链机制先证，冻结声明/身份/条件寿命/费用支持表；不成立replan | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 新版本AST/lower/产物/实例活区域与metadata装配，去固定名分派 | AC-01/02/05 | [ ] |
| T-04 | work | T-03 | 条件挂卸/代次/布局输入/原生DPI/last-owner，旧档兼容 | AC-02..05 | [ ] |
| T-05 | work | T-04 | 命名gates/flow负控/20轮寿命/合法超界/必要真输入及bool高槽与UI不重绘时子场景更新，原缺口逐项核 | AC-03..06 | [ ] |
| T-06 | work | T-05 | 冷A→B/坏件下一Run/verify_plan054/SD及独立核验输入与AG021配方 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同树独立上下文final，finding返work后同员有界复核，禁同会话自检冒独立 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结触面并集/身份核后落主，不重复无关全AG | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/归档/当前导航与准确新候选，AG021只消费实交付 | AC-08 | [ ] |
| T-10 | merge | T-09 | guard清自有树/实际全部输出对账、scope清理、摘要/hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: new / PLAN-054:r1 / outcome: pass（合同可审阅，后续启动后T-01可进；T-02真实先证门在前）；next: work T-01，当前仅规划。0/10，未建树/scope/新probe/源码或canonical，未运行新UI/GPU/原生输入，不改048/049归档。AG021只作draft r2依赖衔接，原验收不减。

2026-11-05 / stage: new / planning_validation: pass（合同预检，非实施/独立final）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD、drafting/r1/0-10与未勾任务齐；AG021 r2保同一分母，现场原owner已记录executing2/10及T-03等待054，未回退其进度。依赖DAG无环，M3研究不是M2前置；独占号源后重新扫描主/归档和注册树核新号054/055/AG022唯一，next UI056/AG023。三仓49处新增/修改本地链接可解析，白名单空白检查通过；现有`python tools/gallery.py validate --all` exit0，12逻辑项/22入口/许可保持。048/049归档与canonical、019归档/候选canonical逐字节内容未改（SHA前16分别dfbd9df1087d52a7/b7f5c867ee107c2a、7bda6f53faf16cae/8967cc6c8cf87f0a、1d8e991170ec3969/601a457ab52edfec）。未构建/运行新源码或GPU/原生输入/容量研究、未写新canonical/资产/锁、未创建树/scope/agent/发布或迁目录；ag-021已有树与材料属原owner，保留其WIP，本会话不代其提交执行进度或工作收据。第一次预检因AG021并发进入executing不再符合原drafting断言而停，按新证据更新导航后复核通过，不把这次状态变化归为实现测试红。

## 10. 待澄清事项

D-01：纯图形Node作为嵌入或组件内区域的等价机制/具体拼写与新版本由T-02先证冻结；不成立带反例replan。D-02：当前原生取证按手工/明确授权窗口核，不以posted当native。D-03：048/049独立final欠证保历史，本Plan对实际候选/复用必需面独立重核；旧OS20/34/批门另owner在AG021原矩阵对账，不私批豁免。材料唯一`D:/autostack/plan-artifacts/plan-054/auto-ui/`，登记本仓Plan owner/用途/在途，先核是否被外仓OS054材料占用、不共用不盲删；外部B/SDK/target/探针/日志均scope。merge摘要折Plan/全部输出对账后删自有scope/构建物，具名hold例外；不碰别仓同号，不建worktree链接，清树先fresh wt-guard。
