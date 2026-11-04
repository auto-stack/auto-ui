---
plan_id: PLAN-043
status: archived
feature_name: AutoGraphicsNode三维模型资源对象模板接线
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 10
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
| SD-02 | modify | docs/specs/auto-graphics/node3d-character-and-runtime-camera.md | ag_char3_move滑扫对全部三维行碰撞→跳过不可见文档行（parked模板/隐藏表现行不阻挡行走；已交付内容阻挡行均可见，可观察行为不变，g14轨迹对照实证） | 043 D-02 实证缺口的最小收口（character3d_v1.at visible_only 0→1，受信单元字节随 plan874/879 identity 登记） | AC-03 |

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
| T-01 | work | 用户另行启动；030/034/035实交付 | 核来源/已有Model/模板拒绝/资源许可/owner，唯一树scope登记，AG016只读 | AC-01/05 | [x] |
| T-02 | work | T-01 | 最多A/B全链先证平色/opaque/多实例/actor-free及角色/非solid/预算，冻结CLI装配/支持域/测试 | AC-01..06 | [x] |
| T-03 | work | T-02 go | 最小作者模板/元数据/独立准入适配、新显式入口/永久夹具/gates，旧档冻结 | AC-01..04 | [x] |
| T-04 | work | T-03 | 真Auto/GPU/触发/角色对照/费用资源边界/生命周期/变异负控 | AC-02..05 | [x] |
| T-05 | work | T-04 | 新根SaveAs/A→B/坏件恢复/20轮owner与真实受影响并集 | AC-04/05 | [x] |
| T-06 | work | T-05 | SD候选/准确消费交接/源预算与未闭项，不追加M2前置 | AC-05/06 | [x] |
| T-07 | review | T-06 | 同树独立final及finding修复后同员复核 | AC-07 | [x] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后落master | AC-08 | [x] |
| T-09 | merge | T-08 | canonical/归档/导航与固定交付身份核验 | AC-08 | [x] |
| T-10 | merge | T-09 | fresh guard/全输出对账/删scope构建物/摘要与hold登记 | AC-09 | [x] |

## 9. 复审记录

2026-11-04 / stage: work / plan_id: auto-ui/PLAN-043 / plan_revision: 1 / outcome: pass（T-01 完成，1/10）/ worktree: D:/autostack/.wt/ui-043/auto-ui / branch: codex/plan-043-dev / base_commit: 2a830d6c43210f61d2496b707da293c213febeee / code_commit: 2a830d6c（本轮无代码改动）/ dependency_revisions: 本仓单仓，无跨仓依赖树 / task_ids: T-01 / evidence: 见下 / blockers: 无 / next: work T-02。

T-01 证据（只读核查，全部在 worktree codex/plan-043-dev@2a830d6c 工作树上以行号核实）：

- 已有 Model 运行支持（driver 零改预期成立）：`Dim::ThreeD::supports_kind` 接受 kinds 2..=6 且 !=5（runtime_objects.rs:59-64，Kind::Model=6，document.rs:9-16）；受信单元 spawn 3D 准入 kinds 2..=6 且 !=5（api/runtime_objects_v1.at:94-98）；`TemplateRules::eligible` 仅拒 group(5) 与异维（runtime_objects.rs:134-146）；`instance_object` 已复制 template.resource/sprite（runtime_objects.rs:279-280）；handle 编码/8slot/32写/高水拒绝逻辑与 Plan §5.2/5.3 一致（runtime_objects.rs:16-29,150-254；.at 全文）。既有 Model Host 测试面：driver tests/runtime_objects_host.rs、runtime_objects_api.rs、runtime_objects_samples.rs、textured_model_resources.rs。
- 唯一作者侧拒绝点：pilot events 下降 builtin-only 检查 `matches!(e.kind, "cylinder"|"player"|"mob")`（lower_components3d_events.rs:201-211），且模板行 `TemplateRowIn.resource` 恒为 None（同文件 :248-257）。model3d 叶作者面已存在：`model = resources.<name>` 直连/Model3D 参数（lower_components3d.rs:376-456，kind="model"）；resolve_model_3d 产 sha256: 平色键 / closure: 034 纹理键（lower_components3d.rs:1627-1690）；模板行 resource 通路已就绪（TemplateRowIn.resource:1708；collect_scene_3d_with:1944-1962 原样拷贝）。→ T-03 改动面收敛为 events 下降一处准入 + 资源解析接线，driver/受信单元零改。
- 资源准入/闭包链：load.rs model3d.glb(:287-332)/textured3d.glb(:338-) 具名拒绝域（缺文件/非.glb/非glTF magic）；034 presentation 元数据 closure 键（resolve_model_3d:1638-1656）；035 批销毁=behavior.rs 每 callback whole-batch despawn（G2/G3 注释 :63-72），快照原价。
- 资源许可/夹具（本仓既有，不从 AG016 tip 复制）：fixtures/node3d/g02/assets/two-parts.glb 多 part 平色（licenses/two-parts.LICENSE：PLAN-838 自撰 MIT，红/绿双三角，sha256 42c9da67…，1528B）；fixtures/node3d-presentation/g10-animation/{character.glb,colormap.png} 034 已交付纹理夹具。两同定义 owner/多实例/隐藏 variant 目录范式见 fixtures/node3d/{twin,g03-hidden}。
- AG016 只读：PLAN-016 已归档（docs/plans/archived/016-*.md），.wt 下无 ag-016 树（仅 ag-019、ag-examples/probe），固定 035 交付 5903839e 在历史中，本轮未触碰。
- 主检出 WIP 观察（非本 Plan，已上报不代管）：master 工作树有 docs/plans/{045,046,047} 新草案、039 计划文件删除、docs/design 改动与 `?? crates/auto-ui-runtime/test/`——疑似他会话在途；master-zero-WIP 在 merge 落主前需其 owner 处理，本 Plan 全程在 ui-043 树内写码不受影响。
- scope 登记：D:/autostack/plan-artifacts/plan-043/scope-registry.md（sha256 16932afb4d3cfa48af3d1d97166a9d910ca4d9993c786b14542ff22d293e6874），唯一 scratch 根，in-flight: none。

2026-11-04 / stage: merge | auto-ui/PLAN-043:r1 | archived+cleaned：Plan 移 docs/plans/archived/043-*.md（git mv，980e8063，status archived，链接自父目录可解）；fresh wt-guard 双树 clean（auto-ui + auto-lang 兄弟 clone）→ worktree remove、分支 codex/plan-043-dev 删除（was ba080e27）、组目录 .wt/ui-043 移除（含 auto-lang clone --shared）；plan-artifacts/plan-043 整 scope 删除（证据已折本文 §9）；registry policy.json 刷新（records: scope-deleted-at-merge, delivered, holds: none）；主检出 043 范围净。批量回归：不到期（043%5≠0，auto-lang last-batch-regression 在案 <48h）。completion_kind: delivered。统一回归门（全量 AG 套件）按 2026-10-28 裁定属里程碑档 pending，在此登记；落地≠部署观察见上（backend release binary/web bundle 不消费本实验线面，无需重建）。outcome: pass。

2026-11-04 / stage: merge | auto-ui/PLAN-043:r1 | checkpoints：

- prepared：reviewed baseline pass（round 11，368f4fa1）；diff base 2a830d6c；冻结 delta=SD-01 add + SD-02 modify（候选文本 01a5da27 在树）；源目标 docs/specs/auto-graphics/{node3d-model-object-templates.md（新）, node3d-character-and-runtime-camera.md（改）}。
- landed：主检出 master 在 rebase 期间被并行 044 会话推进（9bf9874a→ef8bc233→3420d0e8→df37fb4a 全 docs 面）；开发分支三次 rebase 其上，range-diff 各 10/10 patch 等价（最终 old 368f4fa1→new ba080e27）；rebase 后基线复验全绿（043 gates 4/4 + 028 gates 7/7 + verify_plan043 controls 6/6 + driver flow 8/8 + verify_plan044 controls 全绿——044 与 043 共享 main.rs/lib.rs/pilot 面）；`git merge --ff-only codex/plan-043-dev` 落主，无 merge commit，主 tip 一度 = 交付 ba080e27（044 最终收据 df37fb4a 随其后，线性历史 intact）。
- canonical_verified：master 上两份规范 sha256 与复审树逐字一致（node3d-model-object-templates.md c25e5136…54 36；node3d-character-and-runtime-camera.md bb288532…826d）；main-checkout 冒烟 043 gates 4/4 绿（ landed tip df37fb4a，代码面与已验基线逐字一致=同基线证据复用）。
- 落地≠部署观察：本交付为 experimental AG 线（pilot/driver 生成器、受信单元一字节、测试/夹具/规范），backend release binary 与 web bundle 不消费该面，无需重建（记录观察项：release binary / web bundle 不受影响，构建日期不变）。
- 批量回归到期核：auto-lang docs/plans/.last-batch-regression.json 在案（042 收据同日核 <48h；043 不整除 5）→ 不到期，不触发。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 368f4fa1 | next: merge（T-08 冻结定向门/来源核落 master；T-09 归档/导航/身份；T-10 fresh guard/对账/清理）

2026-11-04 / stage: review | auto-ui/PLAN-043 r1 | outcome: pass（final）| review_scope: final | reviewer_identity: goal 会话独立复验（同会话 work/review 分轮；探针全部独立重跑、diff 逐面重核，非照作者摘要）| reviewed_commit: 368f4fa1（review 修复后 HEAD；核 commence 872b8266 + 簿记修复 368f4fa1）| base_commit: 2a830d6c | dependency_revisions: 本仓；auto-lang f4062cca；SDK 681 锁未动（diff 无 sdk/frozen 面）| acceptance_results: AC-01..AC-06 pass（映射见 §9 work readiness；复核摘要如下）| pending_closeout_gates: merge 统一回归门（里程碑档 pending 按 2026-10-28 裁定登记）+ T-08..T-10 merge 任务 + 主检出 master-zero-WIP（045/046/047 他会话 WIP 需其 owner 处理，非本 Plan 债务）| evidence: 下述独立探针 | next: merge。

独立复核摘要：
- diff 面（2a830d6c..368f4fa1，52 文件 +2729/-28）：核心逻辑增量=events 下降参数化（154 行）+ main.rs 组合臂（55）+ lib.rs 导出（9）+ resolve_model_3d 可见性（1）+ character3d_v1.at（2 行：0→1 + 头注）+ 两份 identity 簿记（plan874 2 行/plan879 2 键）；其余为测试/夹具/生成物/规范候选，零散落面。viewport.rs 无 diff（T-04 调试探针已净移除，artifact 扫描无 eprintln/dbg/TODO 残留）。
- 独立重跑探针（review 本轮）：pilot gates 4/4 + 028 gates 7/7；verify_plan043.py --controls 6/6 全绿；driver flow 8/8（GPU 单实例）；node3d_runtime_flow 7/7（G07/G08 墙/支撑 oracle——char3 语义改动最大风险面）；character3d_host 9/9；events/basic/bindings/audio-hud flows 与 character3d api/samples、camera_host 于 work 轮已绿，review 抽查一致。
- AC 映射复核：AC-01 gates+夹具+flow identity 断言；AC-02 GPU 双 part/亮金变异 oracle（despawn 后判别式=0）；AC-03 fault 31/33 + g14 三阶段轨迹逐位对照；AC-04 fault 10/37 + V-S04 20 轮暖账平坦 + 512 denominator 两侧；AC-05 vs04/vs05 + 受影响并集回归 + SDK 锁未动；AC-06 SD-01/SD-02 候选文本在树内、消费交接 controls 绿。
- 缺口/延后扫描：无未声明 workaround；全量 AG 套件按裁定属里程碑档（已声明非前置）；512 边界用合成 ModelData 走真实 prepare 路径（守卫为生产码）；runtime_camera_host 曾 1 例瞬红复跑绿（AGENTS 原生窗口环境归因规则）；verify_plan874 的 .gitattributes 漂移为 master 既有现象（本 Plan 未引入）。
- review 发现与处置：F-1（唯一 finding）= plan879-identity.json 的 character3d_v1.at 哈希在工作轮声称已更新实则静默未改（dict 键 sha256_bytes/sha256_normalized 未被当时脚本覆盖）——已在 review 修复（368f4fa1，两键核字节/规范化哈希一致）并复验 diff；其余声明与磁盘一致（含 round-7 收据在修复后成立）。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 90f5de5e | next: review（同树独立上下文 final，review_scope: final）

2026-11-04 / stage: work | plan_id: auto-ui/PLAN-043 | plan_revision: 1 | outcome: pass（execution_done，6/10）| worktree: D:/autostack/.wt/ui-043/auto-ui | branch: codex/plan-043-dev | base_commit: 2a830d6c43210f61d2496b707da293c213febeee | code_commit: 872b8266（T-06 规范提交，work 分支 HEAD）| dependency_revisions: 本仓单仓；.wt/ui-043/auto-lang @ f4062cca（clone --shared）；driver SDK 681 锁 prepare_sdk PASS | task_ids: T-01..T-06 | evidence: §9 各轮 receipts + 下述 readiness 汇总 | blockers: 无 | next_responsible_role: review。

work readiness 汇总（AC-01..AC-06 work 面）：AC-01 普通 use/具名模型/纯模板/局部 owner/拒绝域/无数字 ID——g12/g13/g14 夹具 + gates 4/4 + flow 8/8（V-S01/06）；AC-02 多 part/opaque 外观真实实例驱动、状态事件隔离、无静态补绘——GPU 双 part/亮金像素变异 oracle + despawn 消失 oracle（V-S02）；AC-03 Run/代/触发/显隐销毁正确、非 solid/非角色相机/空间查询——fault 31/33 + g14 三阶段轨迹逐位对照 + SD-02 收口（V-S02/03）；AC-04 credits/写/槽/mesh/资源并集/源 VM 预算边界——fault 10/37、512 denominator 两侧、V-S04 20 轮暖账平坦、union 原额内（V-S03/04）；AC-05 actor-free/有角色（g14 walker）/20 轮 owner/保存搬迁冷消费/坏件恢复/SDK 与旧触面保持——vs05 + 受影响并集回归绿（runtime/events/basic/bindings/audio-hud flows、character3d api/host/samples、camera_host；028 gates 7/7 + verify_plan043 controls 全绿 + SDK 681 PASS）；AC-06 SD-01 add + SD-02 modify 候选文本已备于 worktree（docs/specs/auto-graphics/node3d-model-object-templates.md 新增、node3d-character-and-runtime-camera.md 增量），消费交接=verify_plan043.py --controls + 两测试入口（§6 冻结命令），未闭项=无（D-01..D-03 已闭/登记）；不追加 AG016/AG019 M2 前置（未触碰其代码/锁/生成物）。统一回归门（全量 AG 套件）按 2026-10-28 裁定属里程碑档 pending，非本 work handoff 前置；review/merge 按受影响并集复核。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 3713e80b | next: work（T-06 SD候选/消费交接/源预算/未闭项 + 受影响并集终核）

T-05 证据（5/10）：①SaveAs 搬迁——g12 运行中 stop 后 `saveas` 新根（场景+资产+脚本闭包随存），冷打开新根 → start → 双实例 + 绿像素 oracle 绿（同入口真 Run）。②坏件恢复闭环——新根 GLB 单字节损坏：load 具名拒绝（hash/identity）；修复原字节 → load+run 绿。③g13 闭包 PNG 中段字节损坏：拒绝 → 修复 → 双实例 spawn 绿（纹理闭包面同验）。④A→B 冷消费由验证器 cold regen（生产侧）+ 各 flow 测试冷载模式（消费侧）双覆盖；20 轮 owner 暖账 T-04 已证。flow 8/8。提交 3713e80b。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 7a0d6d93 | next: work（T-05 SaveAs/A→B冷消费/坏件恢复/真实受影响并集）

T-04 完成证据（4/10，AC-02..05 work 面收口）：①V-S04 暖账——20 轮 start/6tick/stop：mesh_count 与 decode_attempts 全程平坦（注册表共享 GLB 身份，暖 decode/upload=0）；资源并集=单件 1528B（raw8 身份/单件 8MiB/总 32MiB/CPU/GPU 64MiB 原额内）；20 轮后文档逐字不变。②512 mesh denominator 边界两侧——合成 63/65-mesh 模板×8 槽 + base：521 在 with_project 首帧 prepare 具名拒绝（"resource limit: potential mesh instances 512"，比 play start 更早的收口点）；505 正常启动；invisible parked 模板行计入 denominator（负控语义在案）。③前片已证：fault 10/37/31/33、g14 角色轨迹对照（visible_only 收口）。flow 全文件 7/7。提交 7a0d6d93。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 6c910e14 | next: work（T-04 续：V-S04 资源并集/20轮owner/暖账 + 512 denominator 负控）

T-04 轨迹对照证据（D-02 关闭，真实缺口实证）：g14-model-actor 夹具/产品（walker 角色 +x 每 tick 0.5 车道 z=-3.0；Crate 模板 parked 在车道 x=4.5 invisible；活实例 3.5 处 tick4 销毁）——variant（模板在路径）vs control（场景字节变异 park -30）首跑 **walker 冻结在 3.0**：定位至 driver `ag_char3_move` 对 `ag_move_and_slide(node, 0=visible_only, …)` 的冻结调用——滑扫对 ALL 3D 文档行（含 invisible parked 模板行）碰撞，模板视觉 AABB x∈[2.3,5.3] 罩住 walker。收口（043 范围内最小且对全部已交付内容可观察行为不变）：character3d_v1.at 滑扫 visible_only 0→1（跳过不可见行；头注记录 PLAN-043 收口）。对照复跑：逐 tick 轨迹逐位相等、走过模板/实例点无阻挡。受影响面回归全绿：runtime_flow 8/8（G07/G08 墙/支撑 oracle）、events_flow 7/7、basic 6/6、bindings 8/8、audio_hud 9/9、character3d api/host/samples 26/26、camera_host 12/12（首轮 1 瞬红复跑绿，环境归因）、043 controls 全绿。簿记：plan874/879 identity 中 character3d_v1.at 哈希外科更新；verify_plan874 在主检出同样因 `.gitattributes`（874 归档后删除）漂移而红——既有现象，非本 Plan 引入。提交 6c910e14。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: c864c134 | next: work（T-04 续：角色轨迹非solid对照 + 资源并集/20轮owner实测）

T-04 首片证据（预算/身份负控）：tests/node3d_model_objects_flow.rs 新增 model_objects_typed_budget_faults_are_named——对 g12 提交产物字节做 typed 变异经 SourceSnapshot 启动：①33rd credit→fault 10（tick 注入 31 个 ag_char3_move：角色 op 计费不占 object 写上限，2 object+31 char3=33>32 共享池）；②33rd lifecycle write→fault 37（Start 2 spawn+31 move=写#33，写上限先于计费——先注入 object.move 32 个会先撞 37，已实证区分）；③slot 8 spawn→fault 31；④despawn 后旧句柄再 move→fault 33（sticky 身份拒绝，等待环走到状态机 despawn 分支）。flow 全文件 4/4。提交 c864c134。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: de7654b6 | next: work（T-04 真Auto/GPU/触发/角色对照/费用资源边界/变异负控）

T-03 证据（3/10）：①平色+opaque 混装修复——model-objects-3d 入口对受检元数据按 kind 过滤（model3d 条目滤出→resolve 回落 sha256: 键；textured/animated 保留→closure 键+动画把守），034 presentation 档"元数据在案必须有 closure"冻结语义不动；g12 重生成于 plan043 身份（document_id/generator 切换，模板行仍 sha256: 键），g13 重生成一致。②gates 4/4（新增 B 面负控：animated3d 声明域拒绝（load 期需真实 GLB 字节）、模板 model/msg/on 在组合入口同样拒绝）。③`tools/verify_plan043.py --controls` 全绿：identity（generator/api/document_id/sha/closure/资源键/资产清单）、冷再生 A→resource-meta→B 双 pass 字节一致（平色也过公开导入面，0-closure 条目回落 sha 键）、突变敏感（pad x 场景行变、world.at 行为源稳定——字节稳定性质实证）、旧档冻结（028 g11-main 从闭包再生成字节一致，events 重构零影响）。④回归：043 gates 4/4 + 028 gates 7/7 + driver flow 3/3（产物身份切换后）。提交 de7654b6。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 388426d8 | next: work（T-03 永久入口/gates/负控硬化）

T-02 收口证据（A/B 全链先证完成，任务勾 2/10）：

- B 候选冻结为显式组合入口 **model-objects-3d**：pilot `lower_project_components3d_model_objects(project, meta)`（events 对象/触发/模板域 + 034 受检元数据；events 实现重构为 lower_events_impl 参数化 presentation/标签，028 出口传 None 字节不变——028 gates 7/7 复证）。animated3d.glb 声明整域具名拒绝（不删动画轨道伪装静态）；模板叶带 meta 时二次拒动画内容。CLI：`node2d-lower --profile model-objects-3d --entry main.at --out DIR`（pass A 产 resources-request.json）→ driver `resource_meta --entry <dir> --request ... --out resources.json` → `--resources resources.json`（pass B 产最终产品）。拒绝域：events 档 textured3d.glb 保持拒绝（gates 负控）；model-objects 档接受 textured/拒 animated；旧档零影响。
- B 全链产品 g13-model-opaque：夹具 coin.glb（Kenney CC0，g10-textures 在库许可）+ Textures/colormap.png；生成物 Coin 模板行 resource=`closure:76657e13…`、文档资产带闭包（PNG sha/尺寸）、assets/ 侧车双文件；document_id plan043-model-objects-3d-main。
- B driver 全链（tests/node3d_model_objects_flow.rs g13 用例）：Storage 准入（闭包 GLB+PNG 字节）→ 亮金像素 oracle（coin colormap 金 r224-255/g192-224/b64-96；pad 橙 g~140 与红 g~77 不在类）→ 双实例 spawn/move/hide/触发/despawn（亮金=0）/gen2 重生再暖。driver 运行时代码零改——instance 适配路径本就支持 closure 模型（探针实证：一次"渲染缺口"疑云最终定位为本测试 g13 生命周期误用 g12 产物之测试缺陷，修正后绿）。
- 测试入口冻结：pilot gates `node3d_model_objects_gates`（3/3）+ driver flow `node3d_model_objects_flow`（3/3，--test-threads=1）；028 events gates 7/7 回归。提交 388426d8。
- T-02 范围说明：有角色业务链（g11 式角色行走驱动 spawn）与非 solid 角色轨迹对照、32/33 credit 写预算边界属 T-04（§8 T-04 列"真Auto/GPU/触发/角色对照/费用资源边界"）；T-02 先证目标（A/B 路径可行性 + 入口冻结）已达成，D-01 关闭（opaque 必须走 closure 组合，实证 driver 旧 GLB 准入拒 images/textures）。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 8b36e0b8 | next: work（T-02 收口：B 候选 closure 组合入口设计冻结 + T-02 任务勾）

T-02 A 候选 driver 全链先证（V-S02/V-S05 首证）：g12-model 生成物（scene.agscene 模板行 kind=model/resource=sha256:42c9…/parked false + assets/<sha>.glb 侧车 + world.at typed 端口）经真实消费链 Storage::load_project（资源字节 sha 准入）→ Host → Triggers3V1 装配 → 真 Auto 生命周期：Start 双实例（同模板 id、gen1、位置 [2,.5,0]/[6,.5,0]）→ move+hide+pad 触发 pair0（事件消费滞后作者一 tick，测试用阶段等待环兼容）→ despawn 后绿像素=0 且红色回落圆柱基线（真销毁/真隐藏变异 oracle：静态圆柱默认红不可改色，绿色作判别式）→ 同槽 gen2 重生资源共享再暖 → Stop/重开新 run 再渲染。负控：剥 resource 键→"model resource reference"；GLB 单字节篡改→"asset hash/length"。测试 tests/node3d_model_objects_flow.rs 2/2（GPU 单实例 --test-threads=1）。SDK 681 锁：prepare_sdk --from-snapshot PASS（verify_source 聚合器内 verify_plan013 历史漂移为在库既有现象，与本 Plan 无关）。回归：043 gates 3/3 + 028 events gates 7/7（fixture 重构后复跑）。提交 8b36e0b8。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: e19bba1f | next: work（T-02 续：driver 全链先证 + B 候选 closure 组合入口冻结）

T-02 中段证据（A 候选先证，pilot 半链）：①B 缺口实证——driver 旧 GLB 准入拒绝 images/textures/animations/skins（assets.rs:513-520），sha256: 平色链物理不可携带 opaque GLB → opaque 必须走 034 closure 组合（D-01 关闭方向）；events 档 textured3d.glb 声明拒绝面保持（gates 负控）。②pilot 接线：lower_components3d_events.rs 模板叶门禁扩 model3d（具名 resources.<name>，resolve_model_3d 平色 sha 键），TemplateRowIn.resource 接通；bindings 档未动。③夹具 g12-model（actor-free 哨兵角色 + Crate 模型模板 + 双槽同模板生命周期梯）+ 新 gates tests/node3d_model_objects_gates.rs 3/3（模板行 kind/resource/parked、typed spawn id 3 双槽、字节稳定、负控：未知资源/solid/参数绑定/model 状态块/textured 档域）。④回归 node3d_events_gates 7/7（028 旧档字节面不变）。提交 e19bba1f；依赖兄弟 .wt/ui-043/auto-lang @ f4062cca（clone --shared，仿 042 组内固定）。

2026-11-04 / stage: goal | plan_id: PLAN-043 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-043/auto-ui | commit: 2a830d6c | next: work（T-02 A/B 全链先证）

2026-11-04 / stage: new / plan_id: auto-ui/PLAN-043 / plan_revision: 1 / outcome: pass（合同可审阅，用户启动后T-01可进，T-03需T-02全链go） / next: work T-01。0/10。已有Model运行支持与Node builtin-only拒绝从Spec/代码核实，opaque模板完整链未先证。只规划，无树/材料scope/源码/资产/probe/canonical Spec/构建/GPU/agent，不接管AG016或扩大M2。

2026-11-04本批规划预检（非实施/独立final）：043/044均0..10节、drafting/r1/0-10、10T/9AC/6V-S/1SD，work6/review1/merge3，十任务未勾；依赖DAG无环，二者功能独立，当前AG016..019没有新增043/044依赖。首次取号核到041/042已属组件store/重载路线后，独占重核取得043/044，未覆盖；后续他路线又分配045..047，本轮不回退号源，不提交其草案，下一号以中央.next-id实际独占核为准。262个所涉本地引用258可解析，余4为存量非AG主题036/037/038导航断链（共享索引包含他会话WIP），未代写；新增/修改AG引用可解析。按metadata/generated/Node源交叉核验当前19/22，余g04两维/g05三维；`python tools/gallery.py validate --all`退出0，12逻辑项22入口/许可一致。Model底层支持、Node模板builtin-only及SDK compile region/runtime无span事实从源码/Spec核，不代表新模板/新映射先证已跑。UI040实交付/独立final与D-02 OS15项pending分列；AG016在途3/10固定035，内容合同/源码/锁/生成物均未改，本轮没有创建实现树/材料scope/probe/新canonical Spec、构建/GPU/OS自动化/agent/发布。文档白名单空白与结构核验通过。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 模型模板/opaque闭包跨034/035完整消费机制 | **已闭**（2026-11-04 T-02/03）：实证 driver 旧 GLB 准入拒 images/textures/animations/skins（assets.rs:513-520），opaque 必须走 034 closure；model-objects-3d 组合入口全链实证（g13 GPU 亮金 oracle），driver 运行时代码零改 |
| D-02 | 静态model3d自动solid与模板非solid的交界 | **已闭**（2026-11-04 T-04）：正负对照实证——ag_char3_move 滑扫 visible_only=0 时隐藏模板行阻挡行走（walker 冻结实证）；最小收口=滑扫跳过不可见行（SD-02，character3d_v1.at 0→1，已交付内容可观察行为不变，g14 轨迹逐位对照）；未改 028/030/034/035 在库产品语义 |
| D-03 | 与044共享发射/元数据和未来内容升级 | T-01 已核 owner 短窗：044 为独立草案（docs/plans/044-*.md），本 Plan 不消费/不阻塞其发射面；AG016 固定 035/AG019 既定门保持，新内容消费另约（T-06 复核不变） |

开工材料唯一`D:/autostack/plan-artifacts/plan-043/auto-ui/`，登记owner/用途/在途；临时probe/包/日志/外部消费者/外置target全归scope。merge摘要折Plan/全输出对账后清scope与构建物，仅具名未解决问题持有owner/理由/关闭条件。禁attachments/第二暂存区/链接；清树先fresh `bash D:/autostack/wt-guard.sh <路径>`，他仓同号与在途材料不动。
