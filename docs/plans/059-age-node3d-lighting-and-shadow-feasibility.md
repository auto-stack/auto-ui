---
plan_id: PLAN-059
status: drafting
feature_name: AGE M3三维方向光与阴影作者链可行性
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-024, auto-ui/PLAN-034, auto-ui/PLAN-043]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md]
touched_goals: [GOAL-008]
affects: [autographics-research, light-authoring, rendering-capacity]
---

# PLAN-059：三维方向光与阴影作者链有界验证

## 0. 变更摘要

为M3三维增强先证正常Auto声明/局部事件改变方向光与阴影参数，经既有wgpu管线真实上屏的可行性、资源和兼容边界。只研究一盏方向光＋一个投影物＋一个受影面；输出go/bounded-go/no-go与最小公共能力建议，不合入正式灯光API/新渲染器，不认领three.js覆盖项。本轮只规划。见[执行路线](../design/autographics/m3-next-wave-execution-order.md)。

## 1. 目标

区分底层已能画阴影、可由Auto真实执行驱动、正常Node声明作者链三种能力；至少方向/能量/阴影开关从作者源进入真实GPU，并有独立Lambert/投影几何oracle。测明资源owner、更新/上传/深度图和数值边界，给AG022覆盖表真实研究输入，不凭截图声称标准材质/PBR等支持。

不包含点/聚光、多灯、PBR/透明材质、全three.js、生产UI/混合区域、完整2DHD、浏览器/WebXR、ACE/AOT、预算提升或修改原M2交付。研究结论不新增M2前置。

## 2. 架构方案

固定已交付`e02ca9f8`源码及其SDK681依赖图，在独立研究树串起作者参数→实际Auto状态→研究性受检描述/adapter→LightDef/MeshPipeline/真实GPU。最多两个完整候选：静态描述＋Auto动态标量绑定，或等价的受检通用服务；缺接口明确记gap，不给不存在的Light节点作已实现宣称。Rust只做通用渲染接缝，不代作者业务。

拟唯一树`D:/autostack/.wt/ui-059/auto-ui`，分支`codex/plan-059-dev`；与054/055无功能依赖，可先研究，GPU/焦点错峰。临时研究源码/补丁在计划scope；必要知识沉淀，正式能力另立约，不把probe作为生产API合入。AG022可独立继续目录盘点，不等待本Plan完成，也不倒加依赖。

## 3. 技术栈

已核prototype `src/render3d/pipeline.rs::{LightDef,sample_light,MeshPipeline::render_with}`，`host/surface_runtime.rs`已有固定sample_light调用；pilot `src/translate.rs`、既有components-3d/034资源链，driver公共resource-meta。复用wgpu深度pass和现有静态平色/opaque资源，不新下载大GLB/素材，不换SDK。

## 4. 需求分析与背景调查

用户2026-11-05授权下一批规划及状态/顺序，本轮不执行。已读`node3d-foundation.md`（9451cf0f3c6a8250）、`gallery-node3d-textures-and-animation.md`、`node3d-model-object-templates.md`及M3路线。UI024/034/043已交付不等普通作者Light已支持。观察主64aef780；研究输入固定e02ca9f8，T-01核引用源完整身份。

pipeline SHA前16 a0a31372d4f7b32e；LightDef已有dir/color/energy/shadow_vp/bias，sample_light固定方向/范围；WGSL用ambient＋max(dot(n,l),0)×energy×shadow，现render_with把ambient固定0.25。底层能渲染不等该结构已成为Auto声明/业务可改端口；AG022尚未交付固定目录差距，不声称本微样本对应某个上游ID已覆盖。

## 5. 详细设计

T-01冻结固定源、现有光/相机/网格/资源所有权、shader原语义与基线；样本仅1方向光/1投影物/1受影面，采用自制小几何或已有明确许可资源。T-02最多两个完整链候选先证：作者修改方向/能量/阴影开关→实际Auto求值/状态→受检数据→GPU变化。若只能Rust改LightDef，记backend已成立、作者桥缺失，不记全链go。T-03/04在可达链上作方向正负/能量0与正值/阴影开关、投影物移动及非均匀缩放法线、零方向/NaN/Inf/非法投影范围/坏源负控。无法运行的支路定位原因，不伪造shader结果。

独立CPU数学oracle选远离三角边界/阴影边缘的稳定像素验证明暗与光色；明暗期望由独立法线/Lambert计算，考虑目标格式/gamma/采样误差后T-01冻结明确容差；不能用整图hash不同作为唯一阴影正确证据。阴影检查已知几何的光线遮挡区域正反点，toggle关闭与错误方向/去投影pass的代表性控制须有可区分结果。原阴影bias/frustum与正反面语义保留，原性能数字不冒本次测量。

T-04分别计shader/编译/加载/CPU Tick/提交/稳态绘制、uniform重写、深度图/mesh/cache实际owner/创建和释放，未知GPU计时如实标不可用；原game32credits/steps/ms/CPU-GPU字节不混算，不静默增灯数量/资源额。只有上述固定微样本，不做全目录压测或M2重复全套。T-05做20轮Run/Stop、resize/失效描述拒绝→修复→下次实际Run恢复，未跑原生输入只能记受控fixture层。

T-06输出支持/缺口/风险、明确go/bounded-go/no-go、可重建源码命令/实际数字和正式作者Light能力的最小合同建议。AG022已交付时可增量对照其固定目录，但不重写全目录/扩大其当前验收；M3正式产品能力实施仍核M2真实收口。本研究本身可先行，独立于AG020更名和AutoUI/ACE融合。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md | 固定样例光照已有→普通Auto方向光/阴影链可行性与实际边界；研究不冒正式API | 正常Auto作者与分仓可验证交付 | AC-01..06 |

## 6. 测试设计

以下均为未来work验证合同，本轮未执行。实施者先自检；同一权威树由独立上下文final复审，不能用同实现会话重跑充独立。只跑真实受影响case/必要消费门，GPU/窗口焦点/音频重门按机器单实例；不为merge重复全量AGE，不改根语言时不跑根全仓。新路径/入口由任务建立后才可执行。

| case | AC | owner_stage | 输入/入口与可区分预期 | 覆盖边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | 固定源与≤2候选全链；改Auto方向/energy/shadow，真实GPU受控点变化；仅Rust变光则判作者gap | 不新增产品Light API支持宣称 |
| V-S02 | AC-02/03 | work | 独立CPU Lambert/几何遮挡oracle，光向反转/能量零与正、shadow on/off、去pass控制在预选稳定点可区分 | 明确颜色空间/容差，非仅整图hash |
| V-S03 | AC-03/04 | work | 零方向/非有限/坏投影/非均匀缩放，验证受检拒绝或约定fallback与正确法线；不允许未初始化矩阵 | 候选数值域T-01/02先冻结 |
| V-S04 | AC-04/05 | work | 20轮Run/Stop/resize、depth/mesh/target owner和uniform字节实测；坏源→修复→实际重读恢复 | 没有GPU计时不伪填GPU ms |
| V-S05 | AC-01/05 | work | 固定SDK681/图锁/源身份与旧034/043触面；probe与生产源清楚区分 | 不改旧锁/历史回执或大资产 |
| V-S06 | AC-06 | work | 研究SD/重建配方/最小后继建议逐项对应真实结果和失败；与AG022目录对照只作可选输入 | 未取目录不声称覆盖率，no-go也可合格研究 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 固定源/支持表与≤2完整候选有真实结果，研究/产品状态明确 | V-S01/05 |
| AC-02 | work | Auto→方向/能量/阴影→GPU链可证明或缺口有真实定位 | V-S01/02 |
| AC-03 | work | 数学/几何独立oracle与非法数值反例正确，不以整图hash代正确性 | V-S02/03 |
| AC-04 | work | 固定规模资源/上传/Run及各单位测量真实，未证项明确 | V-S03/04 |
| AC-05 | work | 坏源/描述/资源拒绝和修复下一Run、旧源兼容可复核 | V-S04/05 |
| AC-06 | work | go/bounded-go/no-go和SD/正式能力建议与实际证据一致 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/实际来源与代表性反例pass，finding返work同员有界复核 | T-07 |
| AC-08 | merge | 冻结触面门/来源核后实际落主、canonical及同号归档/导航，结论和实际支持域一致 | T-08/09 |
| AC-09 | merge | 全实际输出对账/必要源命令结果摘要折Plan/fresh guard清自有树与scope构建物，具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续另行启动；024/034/043已交付 | 固定源/单光样本/数学点与容差/资源边界、唯一树scope | AC-01/03/05 | [ ] |
| T-02 | work | T-01 | ≤2完整候选Auto→受检光参数→GPU先证，区分backend与作者gap | AC-01/02 | [ ] |
| T-03 | work | T-02有结果，含no-go | 方向/能量/阴影及数学oracle/数值和投影反例，不可达支路明确 | AC-02/03 | [ ] |
| T-04 | work | T-03 | 固定规模编译加载/Tick/绘制/owner/上传分项测量，旧包络核查 | AC-03/04 | [ ] |
| T-05 | work | T-04 | 20轮寿命/resize和坏源恢复下一Run，旧档身份/受影响消费门 | AC-04/05 | [ ] |
| T-06 | work | T-05 | 研究SD/完整来源命令摘要和最小正式能力建议，no-go如实 | AC-06 | [ ] |
| T-07 | review | T-06 | 同树独立final，真实逐AC/SD/源和反例核验；finding同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结受影响门/准确身份后落主，不扩大无关全套门 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/导航与真实支持/未证边界，非产品研究不冒功能交付 | AC-08 | [ ] |
| T-10 | merge | T-09 | 全路径对账/摘要折Plan/fresh guard清自有树与scope、hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: new / auto-ui/PLAN-059:r1 / outcome: pass（合同可审阅，dependency-ready任务见T-01）；后续依赖和真实先证门未满足时不得推进。当前drafting0/10；仅规划、没有新树/材料scope/源码/探针/构建/GPU/资产/canonical/SDK变更或agent。具体可执行边界见§5/§8，未把上游草案或WIP当交付。

2026-11-05 / stage: new / handoff: pass（依赖024/034/043已交付，后续另行启动可从T-01进入有界研究）。drafting/r1/0-10，全部任务未勾；本批仅规划，未授权或启动本批实施。

2026-11-05 / stage: new / planning_validation: pass（合同预检，不是独立final/实现认证）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD，依赖实际解析DAG无环、M3未成为M2前置；主/归档及注册树独占取号后核UI058/059、AG023唯一，next UI060/AG024。三仓当前白名单新增/修改本地链接104处均可解析，空白检查通过；其中共享导航包含他路线WIP，仅选择本轮AGE行/章节暂存，不代其提交。状态表逐项33 UI＋23 AG＝56（48交付/8未完），48个所列delivery均为各自主HEAD祖先，UI039旧draft影子不重复计。初次表计数模式只匹配“UI编号]”而漏带中文标题的8行，修正为编号后允许标题后复核通过；不是实现红。固定e02ca9f8的pipeline/lower_components2d_runtime与本轮所记源码内容一致；历史canonical/归档及当前054/AG021合同未由本会话修改。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/六旧锁与许可一致。未构建/运行本批新probe/GPU/原生输入/容量/代码，未创建本批树/材料scope/新canonical/资产/SDK或agent；研究未执行不充go，计划未实施不充功能交付。

## 10. 待澄清事项

D-01：正常作者端口不存在时的两候选由T-02真链裁决，不能因底层有LightDef假定作者已支持。D-02：数学像素容差/投影域在实际环境T-01冻结；GPU计时不可用就明牌。D-03：本研究不绑定尚无结论的three.js首批覆盖，不提前认领PBR。材料唯一`D:/autostack/plan-artifacts/plan-059/auto-ui/`，先核跨仓owner/登记用途在途，研究源/原始结果/SDK副本/target/外部消费均子scope；merge必要源命令和成败摘要折Plan后核所有输出，fresh guard清唯一树/删自有scope与构建物，具名owner/理由/关闭条件hold例外。禁止链接/触碰别仓同号/另建权威树。
