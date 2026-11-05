---
plan_id: PLAN-049
status: reviewed
feature_name: AGE基础UI与二维三维Node同应用组合
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-05
plan_revision: 1
current_step: 6
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

### T-02 机制冻结（r1，2026-11-05，c70fd053；详据见§9 T-02 receipt）

1. **语法**：`view-2d/view-3d name { props+elements }` 于基础UI view元素树内（挂载关系）；区域props冻结`width`/`height`（int字面量1..=640，占位rect=子内容逻辑尺寸）＋可选`onclick: .Ev|inst.Ev`（同按钮ABI）。`Node2DMixedDialect`唯一解析入口；node级view+view-2d/view-3d共存守卫与039双出口档在任何dialect下字节不变。
2. **产物**：`autographics.node-ui-mixed/1`（generator `plan049 node-ui-mixed`）；Embed元素在def元素树内（name/dim/width/height/click/scene）；场景元素冻结quad/circle/cylinder，数值LitNum|Field（Field经既有int getter——**世界源与048字节同面，零新增ABI行**）；区域点击并入plan.clicks→同一`_ui_click_`派发器。
3. **Owner/VM**：一个MixedUiApp＝一个Session（`Api::MixedUi2d3d`，main-first同一合同）；app_start每Run一次、app_update仅显式步；UI文字/条件/布局与区域场景绑定同一accept_state遍历（键`{path}#{region}#{elem}#{prop}`）——呈现者零Tick、零VM访问。
4. **Send隔离**：auto-lang-core Session持Rc（线程受限）→运行时路径iced内容不可持Session；`MixedMirror`（Send）仅携带版本化已接受状态副本＋64界出站队列；Session owner在宿主侧，每帧桥接（outbound→handle_string_message→drain→presentation回推）。
5. **组合**：UI＝窗口面view（Iced）；区域占位＝定尺寸容器（真实布局），实际widget bounds（container探测）驱动Flat2d交互挂载（offset=rect.xy、scale=child/rect、clip=rect；变化走`SetMount`）；每区域一Game2d/Scene3d child surface（内容无关quad合成＋scissor）。
6. **输入**：CompositionInput门户优先命中——区域内→child（区域事件入同一64队列），区域外→UI；互斥已证（区域点击不改count；UI点击不动区域）；Tab/Enter环保持048语义。
7. **配额**：048常数全部不动；新增冻结：区域/def≤4、场景元素/区域≤8、区域边≤640、**文字字节缓冲半缓冲拆分（2槽×64B×2）——SDK字面量数组恰256元素静默截断为0（048复审实测＋本轮app_start故障复证），公开len/at ABI不变**。
8. **故障**：Session故障→相关域停止沿048；非法区域（边长越界/未知dim/非root区域）降层与消费双侧具名拒绝。
9. **T-03+具名边界**：非root def区域（实例挂载身份）、区域内If/Group/Instance、坐标负载事件、区域条件隐藏/卸载代次、resize/DPI真实重排、真实native OS通道（本轮自动链posted注入；native手工/授权门归T-05）。

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
| T-01 | work | 用户另行启动；可先只读准备，不等048 | [x] 盘点完成：树`.wt/ui-049/auto-ui`@b3414750分支`codex/plan-049-dev`＋只读sibling auto-lang@30b396d72＋SDK681（`prepare_sdk.py --from-snapshot` PASS 681文件）；verify_plan048 --controls ALL PASS（world 3c85ca6e…）；dialect/pilot/driver/prototype四层端口、owner/预算/旧档/政策全录——证据`plan-artifacts/plan-049/auto-ui/t01-inventory.md`；048互斥守卫parse.rs:444-454为049打开点；driver既有4测试文件Scene3dContent缺hud_overlay红（基线已坏，非049面）；主检出WIP属其它路线未接管 | AC-01/04 | [x] |
| T-02 | work | T-01；048实际交付/独立final/canonical/工具来源 | [x] 完成（commit c70fd053，2026-11-05）：全链机制先证通过并冻结——语法（view内view-2d/view-3d嵌入区域＋Node2DMixedDialect；node级互斥守卫字节不变）、产物（node-ui-mixed/1，Embed元素在def元素树内，场景绑定复用048 int getter ABI，区域点击同`_ui_click_`派发器）、单owner（MixedUiApp=一Session；MixedMirror Send桥承载版本化已接受状态副本——Session持Rc线程受限，不可进组合内容）、组合（区域实际widget bounds驱动Flat2d交互挂载＋SetMount）、输入（CompositionInput门户优先命中，互斥已证）、配额（048常数不动＋区域≤4/def、场景元素≤8、边≤640、半缓冲2槽×64B——SDK字面量数组恰256元素静默截断为0，公开len/at ABI不变）；真实GPU窗口证据probe exit0（按钮×2→count=2＋2D红方块真实位移＋3D圆柱真实平移＋双区域点击回传UI＝1/1＋空闲零Tick）；回归五面全绿（dialect 95+6+4+2、pilot gates 4/4、driver flow 3/3＋storage 3/3、prototype 29+6）。证据`plan-artifacts/plan-049/auto-ui/t02/`。失败replan未触发 | AC-01..06 | [x] |
| T-03 | work | T-02 go | [x] 完成（f3ee443f，2026-11-05）：提交夹具 fixtures/node-mixed/{main,counter}.at（root 双维区域＋嵌套纯UI实例）＋冻结产物 generated-node-mixed/ui-ab-mixed/（node-ui-mixed/1，world sha 82d41ad4…，byte-pin）＋永久门 tests/node_mixed_views_gates.rs 5例（正例降层/区域表/实例链/世界ABI针脚含半缓冲<256断言；byte-identity；048默认dialect载入拒嵌入头；混合dialect下node级双出口/共存守卫且039双view解析档保持开放；7项具名负例）。门证据新增：非root def 区域降层侧补具名拒绝（driver解析侧已有；门捕获pilot曾放行Counter嵌区域）——T-02切片双侧闭合。不放宽旧档已证 | AC-01/02/05 | [x] |
| T-04 | work | T-03 | [x] 完成（56c8f877，2026-11-05；F-1 已闭合 40313ff0）：MixedHost 单一生产链（装配/每帧桥/区域路由/resize 事务重建）——窗口 bin 与离屏测试共用，无第二份装配；离屏 host 测试 5 例（状态＋像素 oracle：2D quad 真实位移新红旧址清、3D 圆柱投影绿；Field 布局增长真实重挂载 oracle——旧顶归 UI/新中心归区域/双派发/count 不动；resize 后路由随真实布局；空闲零回调；显式步单次更新）；真窗探针经 MixedHost passed exit0（posted 通道；Poll 确定性泵送）。门证据驱动修复：Field 序列键冲突（ArtifactProp flatten 同层重复键——048 夹具从未用 Field 布局 prop，spacer 首次暴露；双侧改 `field` 键）＋半缓冲 LitStr 整槽偏移越界（Counter 槽2 标签 128/64；改半内寻址）；产物再生成 byte-pin 随新 | AC-02..04 | [x] |
| T-05 | work | T-04 | [x] 完成（6d57a3de，2026-11-05）：driver flow 门 6 例（node_mixed_views_flow.rs，离屏确定性单实例 GPU）——V-S01/02：双实例同定义独立推进（a+1/b+2 不串扰）＋root 按钮同步推进 count/2D 像素/3D＋动态重挂载＋显式步单次/空闲零 Tick；V-S03：resize 后路由随真实布局（T-04 host 门）＋真窗探针 exit0（279帧，posted 通道；native SendInput 手工门留 review）；生命周期：陈旧代次具名拒绝（stale_rejected）、Session 故障相关域停止＋最后可用呈现保留、队列满 64 具名丢弃计数；预算：区域/边长/dim/非root/schema 消费门具名拒绝；旧档并集：048 ui-ab 产物 driver 侧照常装配/派发＋048 schema 拒嵌入＋混合解析拒 048 schema。门修正：测试入队须走派发器 ABI（直呼 handler 跳过 world 侧 refresh——adapter 合同在测试面同样成立） | AC-02..05 | [x] |
| T-06 | work | T-05 | [x] 完成（089a1798，2026-11-05）：node_mixed_views_storage.rs 3 例（冷A→B 双维同链消费＋identity 交叉核验；坏 world 副本字节翻转具名拒绝＋精确修复下一 Run 恢复；closure/schema/生成器漂移与 048↔mixed schema 互斥具名拒绝）；tools/verify_plan049.py --controls ALL PASS 6 项（再生字节一致/身份链/混合形状 root切片+派发器+半缓冲<256/双图常数含混合新增/SDK681/旧档控制在场）；门捕获 CLI 修复：mixed identity provenance.product 曾嵌 temp --out 全路径（048 用 file_name）→再生 byte-drift，改 048 约定并登记 AG021 consumer 配方；SD-01 候选 docs/specs/auto-graphics/node-mixed-ui-2d-3d.md 已备于工作树（merge 落位） | AC-05/06 | [x] |
| T-07 | review | T-06 | 同树独立final，finding返work后同员受影响复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结触面门与来源核，实际落主 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/当前导航和精确消费者身份，不自动升内容tip | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard清自有树/scope与全部实际输出，摘要/hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: review | review_scope: final（F-1 修复后受影响范围复查→最终） | reviewer_identity/limitations: 同会话实现者复审（限制同前次声明）；复查基于修复 diff 逐行＋同冻结产物探针重跑（reviewer 自跑 exit0） | plan_id: PLAN-049 | plan_revision: r1 | outcome: **pass**（reviewed） | worktree_path: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | reviewed_commit: 40313ff0 | base_commit: b3414750 | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读）；SDK681 verify PASS | spec_inputs: docs/specs/auto-graphics/node-mixed-ui-2d-3d.md（SD-01 候选，逐条与实际行为核对一致；随 merge 落位） | acceptance_results: F-1 闭合一览——修复 diff 仅探针门（5+/2−，bin/mixed_ui_window.rs，产品代码零触碰）：活性判据改确定性闭环（posted=4 且 count=2/reg2=1/reg3=1 全部观测＝四击全被处理；frames 降为诊断）；复查探针重跑同冻结产物 passed exit0（27 帧，节流桌面）；089a1798 上全部不受影响证据保留有效（修复未触任何被测实现文件——flow 6/6、host 5/5、storage 3/3、mixed gates 5/5、node_ui_gates 4/4、node_ui_flow 3/3、node_ui_storage 3/3、dialect 107、verify_plan049 ALL PASS、python 独立字节探针全过，均 reviewer 在 089a1798 自跑）→ AC-01..06 pass（维持）＋AC-07 pass（本审）；工作树干净 | pending_closeout_gates: 统一回归门（owner=merge；并集回归在冻结基线全绿的复跑记录见前次收据） | findings: 无新发现（F-1 已闭合） | evidence: diff 089a1798..40313ff0（1 文件）；t07/reports3/mixed-ui-native.json passed exit0（40313ff0） | next: merge（T-08..T-10）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 7 | from: executing | to: reviewed | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: 40313ff0 | next: merge

2026-11-05 / stage: review | review_scope: final | reviewer_identity/limitations: 同会话实现者复审（无独立分派 final 角色）——按技能要求声明此限制；裁定不基于执行者摘要而是**在冻结 HEAD 089a1798 上全量独立重执行**（下述命令均 reviewer 自跑、全新输出），并新增 reviewer 自有反例探针（python 独立实现复核提交产物字节 vs 冻结合同） | plan_id: PLAN-049 | plan_revision: r1 | outcome: **needs_fix**（F-1，探针基础设施；产品与合同面全绿） | worktree_path: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | reviewed_commit: 089a17984c635464fd5199f705e1d486575423da（工作树干净 0 dirty） | base_commit: b3414750 | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读）；SDK681 verify PASS | spec_inputs: docs/specs/auto-graphics/node-mixed-ui-2d-3d.md（SD-01 候选，工作树；逐条与实际行为核对一致） | acceptance_results: AC-01 pass（verify controls mixed-shape＋gates 5/5＋门证据）；AC-02 pass（flow 6/6：双实例独立 a+1/b+2、区域回传、互斥）；AC-03 pass（host 5/5：动态重挂载 Field 增长 oracle、resize 路由随真实布局）；AC-04 pass（存储 3/3 故障域＋队列 64＋陈旧拒绝；空闲零回调）；AC-05 pass（存储 3/3 身份链/坏件拒绝/修复恢复＋verify 再生字节一致）；AC-06 pass（verify ALL PASS 6 控制＋永久门在场＋SD 候选＋AG021 配方在 identity.json）；AC-07 未决（本审）；AC-08/09 pending（merge） | pending_closeout_gates: 统一回归门（owner=merge；并集回归已每轮执行全绿，冻结基线 089a1798 上 reviewer 复跑全绿） | findings: **F-1（低，探针基础设施，非产品/合同缺陷）**：真窗探针通过门含 `frames > 60` 活性启发——节流桌面下确定性不可满足（HEAD 两次运行 frames=26/23，其余 oracle 全绿：count=2、reg2/reg3=1、idle 零 Tick、posted=4；T-04/T-05 基线曾 274/279 帧通过——环境状态非产品回归）。违反规则：无合同条款要求帧数；该断言为 T-04 探针自加的活性启发，其前提（非节流桌面）在本环境不成立。修复：探针门改确定性闭环判据（posted=4 且全部状态 oracle 观测到即＝四击全被处理；frames 降为诊断记录），不动产品代码。复查范围：探针门一行＋同 HEAD 重跑探针；其余 AC 证据不受影响 | evidence: reviewer 全新执行——verify_plan049 --controls ALL PASS exit0；mixed 三套 driver 门 flow 6/6＋host 5/5＋storage 3/3；pilot mixed gates 5/5＋node_ui_gates 4/4；048 套件 node_ui_flow 3/3＋node_ui_storage 3/3；dialect 95+6+4+2；reviewer 反例探针（python 独立复核提交产物字节）：身份链/区域 root-only/边长预算/派发器绑定/9 个文字缓冲全部 <256 全过；真窗探针两次（frames=26/23，状态 oracle 全绿，门失败留痕 t07/reports{,2}/mixed-ui-native.json） | next: work（F-1 一行探针门修复＋同 HEAD 重跑）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 6 | from: execution_done | to: executing | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: 089a1798 | next: work

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-06） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: 6d57a3de | code_commit: 089a1798 | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读） | task_ids: T-06 | evidence: node_mixed_views_storage.rs 3/3＋verify_plan049 --controls ALL PASS（再生字节一致/身份链/混合形状/双图常数/SDK681/旧档控制）＋SD-01 候选 docs/specs/auto-graphics/node-mixed-ui-2d-3d.md（工作树） | findings: 门捕获 CLI 修复一项——mixed identity provenance.product 曾嵌 temp --out 全路径（048 用 file_name），再生 byte-drift 被再生控制捕获；改 048 约定＋登记 AG021 consumer 配方于 identity.json | blockers: 无 | next_responsible_role: review（T-07 同树独立 final）

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（execution_done；work 阶段交接） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: b3414750 | code_commit: 089a1798 | task_ids: T-01..T-06（全 [x]） | AC: AC-01..06 逐项证据在 §8 各行＋§9 T-02..T-06 receipts（AC-01/02 声明+双向链 AC-03 布局/路由/重挂载 AC-04 owner/预算/故障域 AC-05 身份链/坏件恢复 AC-06 永久门/verify/SD 候选＋AG021 配方） | readiness: 无 open in-contract finding；无未决合同决定； scoped checks 全绿（最新一轮：mixed gates 5/5＋flow 6/6＋storage 3/3＋host 5/5＋node_ui_gates 4/4＋node_ui_flow 3/3＋node_ui_storage 3/3＋surface_contract 29/29＋verify_plan049 ALL PASS＋真窗探针 exit0）；统一回归门显式 pending（owner=review/merge，沿仓库工作流） | worktree 干净（fmt drift 已回退） | blockers: 无 | next_responsible_role: review（T-07 同树独立 final，review_scope: final）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 5 | from: executing | to: execution_done | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: 089a1798 | next: review

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-05） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: 56c8f877 | code_commit: 6d57a3de | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读） | task_ids: T-05 | evidence: tests/node_mixed_views_flow.rs 6/6＋真窗探针 t05/reports/mixed-ui-native.json passed exit0（279帧）＋并集回归 flow 6/6＋host 5/5＋node_ui_flow 3/3＋node_ui_storage 3/3＋mixed gates 5/5＋node_ui_gates 4/4＋surface_contract 29/29 | findings: 门修正一项——测试入队必须走派发器 ABI（直呼 handler 返回 Ok 但跳过 world 侧 refresh，文字缓冲不更新；adapter 'dispatchers only' 合同在测试面同样成立）；夹具增第二 Counter 实例（b: start10/step2，产物再生成 byte-pin 随新） | blockers: 无 | next_responsible_role: work（T-06 冷A→B/坏件恢复/身份/verify_plan049/SD与AG021消费配方）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 4 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: 6d57a3de | next: work

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-04） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: f3ee443f | code_commit: 56c8f877 | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读） | task_ids: T-04 | evidence: node_mixed_host.rs（单一生产链）＋tests/node_mixed_views_host.rs 5/5（离屏确定性）＋真窗探针 t04/reports/mixed-ui-native.json passed exit0（274帧）＋回归 driver flow 3/3＋storage 3/3＋pilot mixed 5/5＋node_ui 4/4 | findings: 两项门/测试证据驱动修复：①Field 序列键冲突（ArtifactProp 自有 name 与 flatten Field.name 同层重复键——048 夹具从未用 Field 布局 prop，spacer 首次暴露；双侧改 `field` 键，产物再生成）；②半缓冲 LitStr 整槽偏移越界（Counter 槽2 标签 128/64；半内寻址）。窗口探针像素 oracle 边界：几何/帧节奏证据归离屏 host 测试（确定性），真窗面为 OS 通道端到端＋状态/互斥/空闲（像素作诊断记录） | blockers: 无 | next_responsible_role: work（T-05 新gates/flow正反例/真实窗口/预算/生命周期和受影响旧档并集）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 3 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: 56c8f877 | next: work

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-03） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: c70fd053 | code_commit: f3ee443f | dependency_revisions: auto-lang @30b396d72（只读）；auto-down @7c4d5cbb（只读） | task_ids: T-03 | evidence: fixtures/node-mixed/＋generated-node-mixed/ui-ab-mixed/（byte-pin）＋tests/node_mixed_views_gates.rs 5/5＋node_ui_gates 4/4 | findings: 一项门证据驱动守卫补面——非root def 区域在pilot降层侧补具名拒绝（T-02切片双侧闭合；driver解析侧原有） | blockers: 无 | next_responsible_role: work（T-04 adapter/布局/viewport/命中焦点/一次更新与共享内容owner公共链）

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 2 | from: executing | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: f3ee443f | next: work

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-02） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: b3414750 | code_commit: c70fd053 | dependency_revisions: auto-lang sibling detached @30b396d72（只读）；auto-down sibling detached @7c4d5cbb（只读；workspace清单解析需同级布局，本轮新增）；SDK681 | task_ids: T-02 | evidence: plan-artifacts/plan-049/auto-ui/t02/（mixed-fixture/main.at 四文字槽夹具；gen-mixed/ ui.json schema mixed/1＋world.at sha16 17f04bdbb986c2ab；reports/mixed-ui-native.json passed=true 全字段：count=2、双区域回传=1/1、2D红方块真实位移229/110/110新址+旧址清退、3D圆柱真实平移中心绿→天空、idle_no_tick=true；probe-final2.log EXIT=0）；回归：dialect 95+6+4（新mixed解析面4例）+2、pilot node_ui_gates 4/4（048冻结负控字节不变）、driver node_ui_flow 3/3＋node_ui_storage 3/3（单实例GPU真实窗口）、prototype surface_contract 29/29＋surface_review_r3 6/6 | findings: 三项合同内精化（证据驱动、有界、非静默弱化）：①文字字节缓冲按半缓冲拆分（2槽×64B=128×2）——SDK字面量数组恰256元素静默截断为0（048复审"256→0"实测＋本轮app_start故障复证），048公开len/at ABI不变、容量4槽保持；②Session持Rc（auto-lang-core线程受限）→组合内容不可持Session，新增MixedMirror Send桥（版本化已接受状态副本＋64界出站队列），Session owner在宿主侧——§5"或用版本化数据和清楚owner隔离"分支成立；③运行时widget_bounds探测补container回调（048窗口侧Operation先例；现有focusable消费不受影响）。T-03+具名边界：非root def区域（实例挂载身份）、区域内If/Group/Instance、坐标负载事件、区域条件隐藏/卸载代次、resize/DPI真实重排、真实native OS通道（本轮自动链为posted注入；native手工/授权门归T-05） | blockers: 无 | next_responsible_role: work（T-03 类型化嵌入AST/降层/模块/资产与挂载产品）

2026-11-05 / stage: work | plan_id: PLAN-049 | plan_revision: r1 | outcome: pass（T-01） | worktree: D:/autostack/.wt/ui-049/auto-ui | branch: codex/plan-049-dev | base_commit: b3414750 | dependency_revisions: auto-lang sibling detached @30b396d72（只读）；SDK681 prepare_sdk --from-snapshot PASS | task_ids: T-01 | evidence: plan-artifacts/plan-049/auto-ui/t01-inventory.md；verify_plan048 --controls ALL PASS | blockers: 无 | next_responsible_role: work（T-02 全链先证）

2026-11-04 / stage: new / PLAN-049:r1 / outcome: pass（合同可审阅，未来T-01可准备），blocked（T-02/后续接线待048真实交付与其来源门）；next: 另行启动后T-01，048交付后T-02。drafting0/10，本轮无树/材料scope/探针/构建/UI/GPU/源码/SDK/canonical变更，不改019归档或043/044。

2026-11-04 / stage: new / planning_validation: pass（仅合同/来源核查）。四份当前合同0..10节、各10T（work6/review1/merge3）/9AC/6V-S/1SD与owner_stage齐、任务全未勾，依赖按仓限定无环（UI048→049→AG021→020），本地号唯一/next UI050、AG022；暂存白名单42个新增/修改本地链接可解析，三仓白名单空白检查通过。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/许可保持；019归档SHA前16=1d8e991170ec3969、canonical=601a457ab52edfec逐字节未变。未构建/执行任何新profile/窗口/GPU或故障探针；新UI/混合应用验证均待work，不能当已实现证据。没有新实施树/应用目录/scope/目录迁移/agent；保留其它路线WIP，未改源码/资产/SDK/消费锁或任何归档合同。

## 10. 待澄清事项

D-01（已解，T-02）：048实际交付（cd1e161a，master）已为T-02只读冻结来源——profile/ABI/容量常数按其canonical事实冻结，未从草案预填；完整双维ABI组合成立（见§9 T-02 findings），无replan。D-02（已解，T-02）：区域最小嵌入=width/height int字面量1..=640、onclick同按钮拼写；世界/VM零新增ABI行（场景绑定复用int getter）；文字半缓冲拆分为新增target支持界（公开ABI不变）。D-03：原生政策——T-02自动链用posted注入（路径证据）；真实native OS通道留T-05具名核（048手工取证先例），缺证不充pass。唯一材料scope`D:/autostack/plan-artifacts/plan-049/auto-ui/`；开工登记owner/用途/状态，外部B/SDK/target/探针/输出均scope，不散落repo/attachments。merge摘要折Plan、全部实际路径对账/fresh guard清树、删自有scope构建物，具名hold方可保留；不动同号别仓材料、不创建任何worktree链接。

2026-11-05 / stage: goal | plan_id: PLAN-049 | plan_revision: r1 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-049/auto-ui | commit: c70fd053 | next: work
