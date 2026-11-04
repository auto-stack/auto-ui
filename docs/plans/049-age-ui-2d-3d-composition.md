---
plan_id: PLAN-049
status: drafting
feature_name: AGE基础UI与二维三维Node同应用组合
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-048, auto-ui/PLAN-039, auto-ui/PLAN-040]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-mixed-ui-2d-3d.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver, autographics-prototype]
---

# PLAN-049：基础UI布局中的2D／3D嵌入、状态与输入组合

## 0. 变更摘要

在048实际交付的基础UI Node链上，允许普通`view`布局容纳`view-2d`和`view-3d`嵌入区域，同一应用/窗口真实显示UI、二维与三维内容，状态/事件、布局/裁剪、输入/生命周期一致。纯2D/3D Node仍直接用其出口。补[M2-10设计](../design/autographics/m2-mixed-ui-2d-3d-composition.md)的引擎部分；AG021交综合示例与验收。drafting/r1/0-10，暂不实施。

## 1. 目标

1. root UI view内row/column可同时嵌入独立Node/module的2D和3D呈现，不将其解析为互斥变体，不复制model/Tick。
2. 真实UI按钮消息进入Auto model/on，经普通组件事件/绑定影响2D位置/3D变换；图形局部输入/逻辑也能反馈UI，不在Rust写业务。
3. 同设备合成、嵌入布局/viewport/scissor/命中/焦点/捕获/resize正确；多实例/挂载/Stop/新Run状态和资源owner可核。
4. 公开混合入口、固定SDK/来源/离线闭包/旧档兼容，供AG021固定消费。

非目标：完整3D游戏再实现、生产所有UI/主题/复杂文本编辑、任意UI贴到3D网格/曲面与三维深度正确UI、多窗口/跨端、Life.Frame/环境变体选择、ACE/AOT/ECS/HMR、043模型模板/044映射、旧Gallery自动改锁或远端发布。048新增基础UI词法/控件业务链不重复实现。

## 2. 架构方案

048普通UI树中增加类型化嵌入节点→显式应用/内容owner与呈现挂载描述→AGP CompositionGraph/SurfaceRuntime/CompositionInput；2D/3D adapter复用现有真实渲染与space/query等最小必要公共服务，声明树与运行图分离。共享状态owner一次Start/Tick，多观察出口不另立业务状态；运行可以跨一个或多个VM，但有唯一可查的状态所有权/事件顺序与配额映射，不能因VM/模块/出口翻倍更新和预算。

拟唯一实施树`D:/autostack/.wt/ui-049/auto-ui`、`codex/plan-049-dev`；同组语言依赖只读固定源，禁链接。本轮不建。T-01可先盘点，T-02全链及后续接线硬等048实际交付/独立final/canonical/真实工具来源，不读其WIP或复制未交付tip；不强依043/044，写面共享按owner短窗。

## 3. 技术栈

实际生产/消费路径：dialect `src/node2d/{ast,parse}.rs`、pilot `src/{load,lib,main,translate}.rs`与已有lower；driver `src/authoring/`和`src/lib.rs::Session`；prototype `src/{core/composition,renderer/composition,host/{surface_runtime,iced_surface,composition_input}}`。新增显式混合profile/产品（拟`mixed-ui-2d-3d`，T-02冻结），不把039的queries-3d有限marker档扩大或偷换身份。

## 4. 需求分析与背景调查

2026-11-04用户授权新语法UI/2D/3D同应用目标、只核查/设计/分仓规划、不实施且不打断019。019已delivery3daf2eb并归档清理，保持其原候选fc82061c和历史Spec字节。048先交基础UI，049补其未有的混合组合机制。

基线auto-ui2a830d6c；NodeDecl目前无普通UI view，默认拒绝同Node双维，只有039特定root shape双出口；`node3d-queries-and-mixed-markers.md`明确marker不是UI。`surface-composition.md`已约定单设备/渲染依赖/输入坐标/owner；core graph max_depth默认8，宿主IcedSurface真实控件已存在而应用逻辑仍Rust。Specs另读`node-authoring-consumption.md`、`node3d-foundation.md`、`node3d-queries-and-mixed-markers.md`、`node3d-audio-hud-composition.md`；048是未交付上游，不把本轮草案当能力。

源SHA256前16：node2d/ast e9bc5b47983674ef、parse 7d348e1dbb1b54c4、pilot/lib12edf42a8fed9828、load ba29abf6e510f471、driver/lib4db69f9748a97c88、surface_runtime7144cb08de491c82；固定旧SDK7b987ca8…/681。缺口不是新renderer算法，而是新作者树到多surface/真实Auto状态/输入的桥；未经完整先证不能称改动很小。

## 5. 详细设计

T-02最多两种等价公共接线机制，必须使用048真实交付：root UI含两嵌入区域；一个真实按钮→Auto递增/颜色等作者状态→2D真实移动与3D真实变换＋UI文字，2D/3D区域输入各能回传UI；多视图不重复Tick。先证两维服务装配/符号是否可共用已冻结Session，或用版本化数据和清楚owner隔离，不能将pilot tip和driver固定SDK同图、强制单VM、编新解释器或Rust业务。失败/预算不闭合则needs_replan，不用纯截图/假读回绕过。

首期明确支持平面UI容器里同时嵌入两维/三维；可用纯图元/静态真实三维网格与公共transform，不强带两款完整游戏或混合物理父子。UI layout节点与2D/3D场景父子不同域，禁止跨维场景parent，嵌入是挂载关系；纯2D/3D Node不需加view外壳。嵌入名字/组件类型由symbol解析，不让作者手写surface/对象整数ID。沿既有props/msg协作，不按字段同名串共享。

048基础UI下新增嵌入AST/验证与布局桥，属性拼写优先现有约定，由T-02冻结。每个区域的实际logical rect/physical viewport、DPI、裁剪和坐标逆变换一致；支持窗口resize/row-column重排与合法隐藏/卸载。窗口零尺寸按现有WindowTarget暂停/恢复；图拒绝非法零尺寸surface，不能向已拒图提交输入。每Surface内容至多一次渲染/帧，同状态owner一次Tick/逻辑步；同一内容多挂载保状态、卸载一挂载不销毁仍活owner，条件卸载最后owner才释放。多个观察出口不能导致重启Life.Start。

输入沿CompositionInput命中/坐标变换/焦点/捕获；UI消费点击和Tab/Enter不得漏到图形；图形键盘只给获焦区域，边界和DPI变换取真实layout，不猜屏幕常量；失焦/卸载取消held/capture，旧Run消息拒绝。事件按冻结顺序派发给目标Node实例，UI与图形接受的状态revision可核，不展示跨owner混合半成品；发布相关状态域的选择明确，不强设全世界事务。

原32credits/回调、8有状态Node、128场景行/物理32/资源/VM上界不抬，模块/视图/VM不能多开同owner池；T-02冻结owner→世界→VM→预算/事件桥表、新布局目标/队列内存支持边界。GPU单设备/target owner沿现有合成，稳态不以GPU→CPU逐帧回读拼UI；测试捕获单独记账。循环图/错资源/非法viewport/陈旧句柄/失效模块受控拒绝并保原可用应用；修复后下一实际Run闭环。错误域、GPU失效、根VM失效与局部业务错误分别记录。

新产品含源闭包/各嵌入内容、资产/精确角色和挂载映射、schema/profile/API/SDK/build source身份。新消费者只有显式选profile可达，旧039/040与所有原profile冻结协议/行为/拒绝继续保持。候选供AG021选择实际049交付，不预填未来SHA。

拟新增永久入口：pilot `tests/node_mixed_views_gates.rs`、driver `tests/{node_mixed_views_flow,node_mixed_views_storage}.rs`、pilot `tools/verify_plan049.py --controls`，T-03..06创建。既有prototype `tests/surface_contract.rs`与`surface_review_r{2,3,4}.rs`只选真实受影响case和原负控；改共享UI AST时补受影响生产解析/生成职责组，不为merge跑全AG。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-mixed-ui-2d-3d.md | 独立图形/有限marker→新UI布局中类型化2D/3D嵌入、状态/输入/资源/消费链；旧档保持 | M2-10引擎组合 | AC-01..06 |

## 6. 测试设计

实施者自检后同树独立上下文final；new不创建agent或另一review树。以下新入口均待T-03..06使可运行。GPU/窗口/焦点重门单实例，driver命名case `--test-threads=1`；原OS政策保持，手工取证或另有明确自动化授权，离屏/注入不能冒充原生输入。无必需真实证据则AC未闭，不删项。

| case | AC | owner_stage | 输入/入口与实际可区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | node_mixed_views_flow同窗口三类真实内容；UI按一次→Auto状态+1、2D x+已知增量/3D变换+已知增量/文字一致，改Auto增量对照真变；分别图形输入回写UI | 必须真实GPU/Iced/Auto，不是Rust按名字/计数截图 |
| V-S02 | AC-02/03 | work | 同定义两实例与共享内容双挂载；N个Tick只有N次业务推进，重layout/渲染不追加Tick，卸一挂载另一保状态，最后owner释放 | 无强制单VM，凭owner计数/行为核 |
| V-S03 | AC-03/04 | work | 原生鼠标/Tab/Enter、两区域边界、resize/DPI/裁剪、焦点/失焦/捕获/卸载；UI已消费输入不移动场景，焦点区域动作互斥，实际区域内外像素独立oracle | 原生和离屏分层，不凭固定坐标猜命中 |
| V-S04 | AC-04/05 | work | 新gates合法导入/错view类型/跨维parent/循环图/非有限和零尺寸/陈旧Run/资源坏件；错误定位/保旧应用/精确修复下一Run恢复；32边界正例与33拒绝等受影响预算 | 不沿错误图运行，不静默全局多池 |
| V-S05 | AC-05/06 | work | verify_plan049 --controls＋storage A→B闭包冷再生/实际Run；无旧根/其它项目，篡改一模块/图映射/资产拒绝→还原下次启动，源码修改失效旧Ready | 源/SDK681/工具全链身份，不自填hash |
| V-S06 | AC-01/04/06 | work | 20轮Run/Stop/挂卸/resize目标owner与upload/readback实测；旧039/040/048及受影响原档正反对照；SD和公共CLI/包配方齐 | 原旧oracle/预算不减，不以RSS等价GPU资源 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | UI内嵌双维及纯双维声明语义/公共profile有全链先证和真实消费 | V-S01/06 |
| AC-02 | work | Auto状态/事件UI↔双场景真实生效，多模块/实例隔离且不重复推进 | V-S01/02 |
| AC-03 | work | resize/DPI/布局/焦点/捕获/区域路由/卸载代次正确，必要原生证据齐 | V-S02/03 |
| AC-04 | work | 单设备/owner/资源/队列/预算和故障域可核，边界拒绝不破旧应用 | V-S03/04/06 |
| AC-05 | work | 源/SDK/协议身份和冷消费/修复下次Run闭环，旧档无隐式升级/回归 | V-S04/05 |
| AC-06 | work | 永久正反例/原预算与源风险覆盖，SD候选/固定消费配方可供AG021 | V-S05/06 |
| AC-07 | review | 同树独立final逐AC/SD/实际链pass，修复同员复核 | T-07 |
| AC-08 | merge | 冻结门/source核后落主、canonical与同号归档，准确交付来源 | T-08/09 |
| AC-09 | merge | fresh guard/全部输出对账/清自有树和scope构建物，hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；可先只读准备，不等048 | 现有合成/输入/图形端口/owner/预算/旧档/政策盘点，唯一树/scope | AC-01/04 | [ ] |
| T-02 | work | T-01；048实际交付/独立final/canonical/工具来源 | 最多两完整UI→Auto→双维＋反馈机制先证，冻结syntax/owner-world-VM/配额/故障/产物身份；失败replan | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 类型化嵌入AST/降层/模块/资产与挂载产品，不放宽旧档 | AC-01/02/05 | [ ] |
| T-04 | work | T-03 | adapter/布局/viewport/命中焦点/一次更新与共享内容owner公共链 | AC-02..04 | [ ] |
| T-05 | work | T-04 | 新gates/flow正反例/真实窗口/预算/生命周期和受影响旧档并集 | AC-02..05 | [ ] |
| T-06 | work | T-05 | 冷A→B/坏件恢复/身份/verify_plan049/SD与AG021消费配方 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同树独立final，finding返work后同员受影响复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结触面门与来源核，实际落主 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/当前导航和精确消费者身份，不自动升内容tip | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清自有树/scope与全部实际输出，摘要/hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-04 / stage: new / PLAN-049:r1 / outcome: pass（合同可审阅，未来T-01可准备），blocked（T-02/后续接线待048真实交付与其来源门）；next: 另行启动后T-01，048交付后T-02。drafting0/10，本轮无树/材料scope/探针/构建/UI/GPU/源码/SDK/canonical变更，不改019归档或043/044。

2026-11-04 / stage: new / planning_validation: pass（仅合同/来源核查）。四份当前合同0..10节、各10T（work6/review1/merge3）/9AC/6V-S/1SD与owner_stage齐、任务全未勾，依赖按仓限定无环（UI048→049→AG021→020），本地号唯一/next UI050、AG022；暂存白名单42个新增/修改本地链接可解析，三仓白名单空白检查通过。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/许可保持；019归档SHA前16=1d8e991170ec3969、canonical=601a457ab52edfec逐字节未变。未构建/执行任何新profile/窗口/GPU或故障探针；新UI/混合应用验证均待work，不能当已实现证据。没有新实施树/应用目录/scope/目录迁移/agent；保留其它路线WIP，未改源码/资产/SDK/消费锁或任何归档合同。

## 10. 待澄清事项

D-01：048准确profile/ABI尚未交付，T-02只读实际版本冻结，不从草案预填；完整双维ABI组合不成立则replan。D-02：UI最小嵌入尺寸/事件属性、世界/VM配额与新增target/队列支持由T-02先证，不能改变已确认语义/旧预算。D-03：原生政策与设备条件在T-01/05具名核，缺证不充pass。唯一材料scope`D:/autostack/plan-artifacts/plan-049/auto-ui/`；开工登记owner/用途/状态，外部B/SDK/target/探针/输出均scope，不散落repo/attachments。merge摘要折Plan、全部实际路径对账/fresh guard清树、删自有scope构建物，具名hold方可保留；不动同号别仓材料、不创建任何worktree链接。
