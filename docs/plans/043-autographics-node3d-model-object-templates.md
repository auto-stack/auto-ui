---
plan_id: PLAN-043
status: drafting
feature_name: AutoGraphicsNode三维模型资源对象模板接线
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-030, auto-ui/PLAN-034, auto-ui/PLAN-035]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-model-object-templates.md]
touched_goals: [GOAL-008]
affects: [autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-043：Node3D模型资源对象模板与实例生命周期接线

## 0. 变更摘要

把已有运行时Model对象能力接到Node object3d模板：具名静态平色GLB/opaque纹理GLB成为真正spawn/move/visible/despawn的对象外观，多实例共享资源但身份/触发独立。底层已经支持Model实例；本期优先补作者/准入接线，不重造对象系统。引擎和永久最小夹具归本仓，内容后续显式消费另立合同。

AG016继续固定035，本Plan不改其实现/原关卡/预算/验收，不追加为AG016或AG019当前M2的硬前置。[共同设计](../design/autographics/node-resource-components-and-source-diagnostics.md)。drafting/r1/0-10，仅规划，不实施。

## 1. 目标

1. 普通use中具名Model3D/静态Textured3D资源可声明纯object3d模板；局部owner Node持自己的句柄/事件，沿SlotRef3D/TemplateRef3D/PairRef3D接线，作者不写数字资源/row/slot ID。
2. 完整多part/opaque纹理由真正运行实例出生/运动/显隐/销毁驱动；两个同定义owner状态/receiver隔离，不能另挂常驻静态GLB补绘。
3. 模板和活实例保持非solid、非character/camera目标、不进原空间查询世界；触发使用原显式shape/radius/身份，不从视觉AABB推导玩法。
4. 原Run/slot/generation、32共享credits/物理写32、8slot/8pair及资源/mesh/VM包络、owner释放和完整闭包离线消费保持。

非目标：运行实例动画pose/骨骼、动态mesh/material/TRS/纹理替换、动态Node树/跨场景、完整资源参数多态、新物理/触发算法/音频HUD、游戏迁移、ACE/AOT/ECS/CPC/跨端。模板首版仍一个静态模型叶、无参数/model/msg/on；业务属于独立有状态owner。

## 2. 架构方案

Node/use具名资源/纯模板→034公开resource-meta准入/closure→显式新作者入口（工作名model-objects-3d，T-02冻结）→既有模板base row/运行对象协议→driver TemplateRules/instance_object/Renderer→Run-owned实例/资源。复用030名义引用和035有界组合；现有协议足够则零改，不为新入口必造新运行API。

拟唯一树`D:/autostack/.wt/ui-043/auto-ui`，分支`codex/plan-043-dev`，本轮不创建。同树不同上下文独立final；044非功能前置，可独立推进，load/translate/main/identity/Storage按owner短窗。AG016树与固定035版本只读，pilot语言图与driver SDK681图分开，target/材料scope独占。

## 3. 技术栈

已核pilot `src/lower_components3d_events.rs`（下均相对experimental/autographics-node2d-pilot）：只许cylinder/player/mob模板，TemplateRowIn.resource目前None；`lower_components3d.rs`、`lower_node3d_presentation.rs`及034公开资源请求/元数据链。driver `experimental/autographics-auto-driver/src/authoring/runtime_objects.rs::Dim::supports_kind`已接受Model，`instance_object`复制resource/sprite，已有multi-mesh denominator及Model Host测试；`api/runtime_objects_v1.at`和035批销毁机制可复用。

拟新增pilot `tests/node3d_model_objects_gates.rs`、driver `tests/node3d_model_objects_flow.rs`、最小actor-free/有角色/多实例夹具及pilot `tools/verify_plan043.py`，T-03/04建立可运行入口。opaque closure模板能否直接复用必须T-02实证，不凭Model枚举存在认完成。

## 4. 需求分析与背景调查

用户2026-11-04授权UI后继规划，允许合同/设计/导航及相关路线簿记，未授权实施/建树/探针/复制资产/构建/GPU/agent。观察UI `92ee2459a1058ddddc3fc3378ecec6585ffe726c`；040交付fc82061c，独立final/归档清理有记录，OS层15项仍D-02 pending。AG016 executing/r1/3-10、唯一树ag-016/ag-examples、固定035 delivery5903839e；T-03记录模板builtin限制导致币GLB视觉另接有状态Coin。本Plan是后继能力改进，不强迫游戏升级；该适配是否满足原半径/几何/玩法参数与AC仍由其后续验收及独立复审核，T-03 go不替代内容终验。

| 权威来源 | SHA256（规划盘上字节） |
|---|---|
| auto-lang docs/specs/auto-graphics/runtime-objects.md | `81cd9204b7ac7bc7556b11ff72bc5e54199675b62154a6636bda2f510a4b52f0` |
| auto-lang docs/specs/auto-graphics/textured-model-resources.md | `72ac476e104e4eb410ca747680999814e5418d857b18d602bb8e2783be4e9d39` |
| docs/specs/auto-graphics/node3d-symbolic-component-bindings.md | `6ed0bf070de48c3fae4ff8c1687dca9d67c8645010aef5783e49ca14b3352e18` |
| docs/specs/auto-graphics/gallery-node3d-textures-and-animation.md | `93e9ae2a191fed645ffb9dc84a3547370b849a9caa01a1b010bd6f906a151d8d` |
| docs/specs/auto-graphics/node3d-audio-hud-composition.md | `d7272516a359f56eba009463668b3bd10aba593f83024b098db48a029f62d209` |

034/035旧同会话final限制保实际收据；本Plan新独立final不可省，不改历史。SDK固定7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/681内容锁，历史对象不可达用受检保留副本，不换tip。取号时041/042已被组件store/重载准备路线占用；本Plan独占取得043，不动其草案。

## 5. 详细设计

### 5.1 支持域与最多两候选先证

仅平色静态GLB和opaque静态纹理GLB，完整root/PNG原字节/相对URI/closure key与公开独立准入不改。模型内部全部mesh/primitive保留，一个模板叶不是一个mesh；不删动画轨道将character伪装静态，不自行解码GLB。资源静态具名引用可经普通use，模板原无参数/model/msg/on域保持，两个owner用原名义引用持各自slot/handle/事件，资源共享不共享状态。

T-02最多A/B两全链候选：A复用runtime_objects+034闭包，仅补作者模板/具名resource与公共准备；B仅在A有具体准入/渲染缺口时作有限适配/新显式组合身份，旧档字节/费用不变。两路径都须作者→resource-meta→lower→独立loader/TemplateRules→实际Auto spawn/trigger/despawn→GPU/保存搬迁，actor-free与有角色都成立。冻结准确CLI/API/装配、支持拒绝域、资源费用、测试入口后才依赖实现；不足needs_replan，不能只删builtin检查过关。

### 5.2 身份、表现与碰撞边界

原int handle=run*524288+slot*65536+generation；slot0..7、generation1..65535、旧Run/代拒绝与耗尽不回绕。template index绑定本Run accepted base order和真实resource closure。两个实例各自位置/visible/触发/despawn；隐藏仍活，销毁回收slot/触发身份，重用必须新代；拒绝候选保last-good，不污染generation高水。

模板parking/展示沿原规则；模板与活实例均不得成为静态solid/character/camera绑定或混入原空间查询世界。特别核035静态model3d碰撞收集：模板不能因kind=model误入solid。控制将隐藏模板故意放角色路径，出生前/有活实例/销毁后角色轨迹都与无模板对照同；不把模板停远处掩盖错误。收集与接触仍由既有显式trigger shape/radius/对象身份驱动，视觉模型不自动决定碰撞。

不得用额外常驻GLB代运行对象；GPU要看到真实多部件/纹理/深度/阴影和单实例销毁后消失、另一实例不变。初期不授slot实例pose/动画，不以静态模板id冒充活实例pose目标；图片/材质格式沿原域。

### 5.3 原额、owner和独立消费

spawn/despawn/move/visible等各1credit，035 batch/快照原价，所有模块共享32/callback；物理写32先于收费、第33写37/第33计费10。原8状态Node/8slot/8pair、120base+8reserved128；base meshes+8×最坏合法模板mesh数≤512，hidden也计；worst按实际TemplateRules的全部eligible叶统计，不能只数Node显式模板或本Tick将spawn者，每帧live denominator独立复验。资源并集包含模板/current/history/pending/外部pin：raw8身份/32MiB/单件8MiB、CPU/GPU各64MiB及原源VM/time域不抬。

registry模型/opaque闭包真实共享，20轮出生/销毁/重用/Stop/newRun暖decode-upload0，Weak owner证明最后释放；Editor或外部pin合法持有不误判泄漏，最后pin释放后CPU/GPU回收。SDK两图/真实工具/所有被用模板模块/GLB/PNG/元数据互绑；A→B仅闭包冷再生、SaveAs/搬迁/load/真Run；缺图/坏GLB/错闭包或模板身份拒绝→精确恢复→同入口绿。原件/许可T-01核，不从AG016开发tip复制新资产。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-model-object-templates.md | Node模板builtin-only→显式入口支持具名平色/opaque静态模型、实例生命周期/非solid/owner/资源域 | 补作者接线，复用已有Model运行能力，旧档保持 | AC-01..06 |

## 6. 测试设计

T-03/04建立拟入口：`python experimental/autographics-node2d-pilot/tools/verify_plan043.py --controls`；pilot `cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_model_objects_gates`；driver `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test node3d_model_objects_flow -- --test-threads=1`。按真实触面补runtime_objects_host/原030/034/035命名case与冻结控制，非日常全AG/语言根；GPU/音频/窗口门单实例，OS自动化暂停保持。

| case | AC | owner_stage | 实际/拟入口与区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | gates/verifier：跨use具名模型模板、两同定义owner隔离；错资源kind/未知模板/动态模板/动画/错Run或slot具名拒绝 | 纯模板非动态Node树/动画实例 |
| V-S02 | AC-02/03 | work | flow真Auto/GPU：两个多part/opaque实例运动/隐藏/单个despawn/重用及另一实例不变；漏part/静态替身/假销毁实际变异红 | 非只数命令/JSON |
| V-S03 | AC-03/04 | work | 模板放运动路径的非solid轨迹对照；显式trigger一次/旧代拒绝；原32/33与写32/33、512边界/hidden denominator负控 | 不改原物理/事件算法 |
| V-S04 | AC-04/05 | work | raw/decoded/GPU并集/槽pair状态/源VM实测，20轮warm0/真实owner与外部pin最后释放、拒绝不污染高水 | 共享不当零内存 |
| V-S05 | AC-05/06 | work | actor-free/有角色两链，A→B/SaveAs/坏GLB/PNG/元数据闭包拒绝→修复→真Run；SDK681/两图/真实工具核 | 不读AG016 tip或改其锁 |
| V-S06 | AC-01/05/06 | work | 030/034/035受影响冻结/命名消费者及必要Runtime Model门，SD/准确消费域核 | 旧档字节/费用不跟升，未来内容另约 |

work产证；review同权威树不同上下文独立final，逐AC/diff/SD/遗漏延期workaround/告警和真实owner，绑定r1/代码/工具/输入，finding回work同员复核。未授权本会话建agent/第二树，同会话自查不代独立final。merge冻审后定向并集/来源核，复用未变有效证据。

## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通use/具名模型/纯模板/局部owner可用，拒绝域明确、作者无数字ID | V-S01/06 |
| AC-02 | work | 完整多part/opaque外观由真实活实例驱动，状态事件隔离、无静态补绘 | V-S01/02 |
| AC-03 | work | Run/代/触发/显隐销毁正确，模板/实例非solid且非角色相机/空间查询对象 | V-S02/03 |
| AC-04 | work | 原credits/写/槽/mesh/资源并集/源VM预算和边界实测，高水/last-good保持 | V-S03/04 |
| AC-05 | work | actor-free/有角色/20轮owner/保存搬迁冷消费/坏件恢复/SDK与旧触面保持 | V-S04..06 |
| AC-06 | work | SD候选/准确新作者身份CLI/API/资源域和固定消费交接齐，不改在途内容 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/实际资源反例，finding闭环pass | T-07 |
| AC-08 | merge | 冻结门/来源核后实际落主，canonical/来源/归档/消费身份齐 | T-08/09 |
| AC-09 | merge | fresh guard清树分支空组、全输出对账/摘要折Plan/删scope/具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；030/034/035实交付 | 核来源/已有Model/模板拒绝/资源许可/owner，唯一树scope登记，AG016只读 | AC-01/05 | [ ] |
| T-02 | work | T-01 | 最多A/B全链先证平色/opaque/多实例/actor-free及角色/非solid/预算，冻结CLI装配/支持域/测试 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 最小作者模板/元数据/独立准入适配、新显式入口/永久夹具/gates，旧档冻结 | AC-01..04 | [ ] |
| T-04 | work | T-03 | 真Auto/GPU/触发/角色对照/费用资源边界/生命周期/变异负控 | AC-02..05 | [ ] |
| T-05 | work | T-04 | 新根SaveAs/A→B/坏件恢复/20轮owner与真实受影响并集 | AC-04/05 | [ ] |
| T-06 | work | T-05 | SD候选/准确消费交接/源预算与未闭项，不追加M2前置 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同树独立final及finding修复后同员复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后落master | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/归档/导航与固定交付身份核验 | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard/全输出对账/删scope构建物/摘要与hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-04 / stage: new / plan_id: auto-ui/PLAN-043 / plan_revision: 1 / outcome: pass（合同可审阅，用户启动后T-01可进，T-03需T-02全链go） / next: work T-01。0/10。已有Model运行支持与Node builtin-only拒绝从Spec/代码核实，opaque模板完整链未先证。只规划，无树/材料scope/源码/资产/probe/canonical Spec/构建/GPU/agent，不接管AG016或扩大M2。

2026-11-04本批规划预检（非实施/独立final）：043/044均0..10节、drafting/r1/0-10、10T/9AC/6V-S/1SD，work6/review1/merge3，十任务未勾；依赖DAG无环，二者功能独立，当前AG016..019没有新增043/044依赖。首次取号核到041/042已属组件store/重载路线后，独占重核取得043/044，未覆盖；后续他路线又分配045..047，本轮不回退号源，不提交其草案，下一号以中央.next-id实际独占核为准。262个所涉本地引用258可解析，余4为存量非AG主题036/037/038导航断链（共享索引包含他会话WIP），未代写；新增/修改AG引用可解析。按metadata/generated/Node源交叉核验当前19/22，余g04两维/g05三维；`python tools/gallery.py validate --all`退出0，12逻辑项22入口/许可一致。Model底层支持、Node模板builtin-only及SDK compile region/runtime无span事实从源码/Spec核，不代表新模板/新映射先证已跑。UI040实交付/独立final与D-02 OS15项pending分列；AG016在途3/10固定035，内容合同/源码/锁/生成物均未改，本轮没有创建实现树/材料scope/probe/新canonical Spec、构建/GPU/OS自动化/agent/发布。文档白名单空白与结构核验通过。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 模型模板/opaque闭包跨034/035完整消费机制 | T-02全链先证后冻结，不足needs_replan，不把删拒绝充完成 |
| D-02 | 静态model3d自动solid与模板非solid的交界 | T-02/04路径正负对照，必要新收口仅改新入口，不改原profile |
| D-03 | 与044共享发射/元数据和未来内容升级 | T-01核owner短窗；AG016固定035/AG019既定门保持，新内容消费另约 |

开工材料唯一`D:/autostack/plan-artifacts/plan-043/auto-ui/`，登记owner/用途/在途；临时probe/包/日志/外部消费者/外置target全归scope。merge摘要折Plan/全输出对账后清scope与构建物，仅具名未解决问题持有owner/理由/关闭条件。禁attachments/第二暂存区/链接；清树先fresh `bash D:/autostack/wt-guard.sh <路径>`，他仓同号与在途材料不动。
