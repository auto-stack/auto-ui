---
plan_id: PLAN-059
status: archived
feature_name: AGE M3三维方向光与阴影作者链可行性
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 9
total_steps: 10
completion_kind: delivered
cleanup_status: pending
depends_on: [auto-ui/PLAN-024, auto-ui/PLAN-034, auto-ui/PLAN-043]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md]
touched_goals: [GOAL-008]
affects: [autographics-research, light-authoring, rendering-capacity]
---

# PLAN-059：三维方向光与阴影作者链有界验证

## 0. 变更摘要

为M3三维增强先证正常Auto声明/局部事件改变方向光与阴影参数，经既有wgpu管线真实上屏的可行性、资源和兼容边界。只研究一盏方向光＋一个投影物＋一个受影面；输出go/bounded-go/no-go与最小公共能力建议，不合入正式灯光API/新渲染器，不认领three.js覆盖项。本轮只规划。见[执行路线](../../design/autographics/m3-next-wave-execution-order.md)。

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

2026-11-05 用户 `/goal 完成计划059` 授权完整 work→独立final→有界修复→merge/归档/清理链；取代本节早先“仅规划”的启动限制，目标/AC/预算不变。

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
| T-01 | work | 后续另行启动；024/034/043已交付 | 固定源/单光样本/数学点与容差/资源边界、唯一树scope | AC-01/03/05 | [x] |
| T-02 | work | T-01 | ≤2完整候选Auto→受检光参数→GPU先证，区分backend与作者gap | AC-01/02 | [x] |
| T-03 | work | T-02有结果，含no-go | 方向/能量/阴影及数学oracle/数值和投影反例，不可达支路明确 | AC-02/03 | [x] |
| T-04 | work | T-03 | 固定规模编译加载/Tick/绘制/owner/上传分项测量，旧包络核查 | AC-03/04 | [x] |
| T-05 | work | T-04 | 20轮寿命/resize和坏源恢复下一Run，旧档身份/受影响消费门 | AC-04/05 | [x] |
| T-06 | work | T-05 | 研究SD/完整来源命令摘要和最小正式能力建议，no-go如实 | AC-06 | [x] |
| T-07 | review | T-06 | 同树独立final，真实逐AC/SD/源和反例核验；finding同员有界复核 | AC-07 | [x] |
| T-08 | merge | T-07 pass | 冻结受影响门/准确身份后落主，不扩大无关全套门 | AC-08 | [x] |
| T-09 | merge | T-08 | canonical/同号归档/导航与真实支持/未证边界，非产品研究不冒功能交付 | AC-08 | [x] |
| T-10 | merge | T-09 | 全路径对账/摘要折Plan/fresh guard清自有树与scope、hold登记 | AC-09 | [x] |

## 9. 复审记录

2026-11-05 / stage: new / auto-ui/PLAN-059:r1 / outcome: pass（合同可审阅，dependency-ready任务见T-01）；后续依赖和真实先证门未满足时不得推进。当前drafting0/10；仅规划、没有新树/材料scope/源码/探针/构建/GPU/资产/canonical/SDK变更或agent。具体可执行边界见§5/§8，未把上游草案或WIP当交付。

2026-11-05 / stage: new / handoff: pass（依赖024/034/043已交付，后续另行启动可从T-01进入有界研究）。drafting/r1/0-10，全部任务未勾；本批仅规划，未授权或启动本批实施。

2026-11-05 / stage: new / planning_validation: pass（合同预检，不是独立final/实现认证）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD，依赖实际解析DAG无环、M3未成为M2前置；主/归档及注册树独占取号后核UI058/059、AG023唯一，next UI060/AG024。三仓当前白名单新增/修改本地链接均可解析（含UI008补录），空白检查通过；其中共享导航包含他路线WIP，仅选择本轮AGE行/章节暂存，不代其提交。状态表逐项34 UI＋23 AG＝57（49交付/8未完），49个所列delivery均为各自主HEAD祖先，UI039旧draft影子不重复计。初次表计数模式只匹配“UI编号]”而漏带中文标题的8行，修正为编号后允许标题后复核通过；不是实现红。固定e02ca9f8的pipeline/lower_components2d_runtime与本轮所记源码内容一致；历史canonical/归档及当前054/AG021合同未由本会话修改。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/六旧锁与许可一致。未构建/运行本批新probe/GPU/原生输入/容量/代码，未创建本批树/材料scope/新canonical/资产/SDK或agent；研究未执行不充go，计划未实施不充功能交付。

2026-11-05 / planning_count_correction：末轮逐项归属核验补入UI008（004身份修复，0261ae9为主祖先），正确统计49已交付/8未完；仅修正路线簿记数字，不改任何目标/AC/依赖/预算或执行状态。

2026-11-05 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-059/auto-ui | branch: codex/plan-059-dev | base: fa6833bfd106035ed4a50210b61560b61b6921cf | fixed_research_source: e02ca9f8452555db2d44e66363ebf5e317b5716f | next: work. 授权见§4。主树现有文档WIP和crates下4张terminal_pixel PNG为他会话生成物，未发现源码WIP；保留不纳入。scratch跨仓核查无059 scope，已登记auto-ui owner。

2026-11-05 / stage: work | plan_id: PLAN-059 | plan_revision: r1 | outcome: pass (no-go research, not product Light support) | worktree_path: D:/autostack/.wt/ui-059/auto-ui | branch: codex/plan-059-dev | base_commit: fa6833bfd106035ed4a50210b61560b61b6921cf | code_commit: 7b99d58273b6041efa531afabc451cdbcc11f55e | dependency_revisions: fixed e02ca9f8452555db2d44e66363ebf5e317b5716f; author-side auto-lang 593c334805909c25518b7443f9a11707712a45da (034历史固定依赖); driver SDK frozen 681/681 matched source-lock files | task_ids: T-01..06 | next_responsible_role: independent final review.

T-01冻结：真实源码pipeline SHA a0a31372d4f7b32e0d3eb8d00b2c62eb3fa284492a392278ed2458fcfe36234a；独立公式颜色sRGB、每通道±3/255；几何四边形receiver z=.3/half=.95、caster z=.7/half=.18，光向(-1,0,1)/sqrt(2)，投影x'=x+z-.4、z'=1-z。128²像素(83,64)≈P(.3047,-.0078,.3)，沿光线到caster z=.7的x≈-.0953在内部；(25,64)射线在外。偏离边缘与直接遮挡区。Lambert单独控制±Z/energy0,.5；缩放(2,1,.5)后的法线由CPU逆对角(.5,0,2)归一化，不调用shader公式作oracle。无资源/SDK或生产源修改。

| AC / case | 实测及边界 |
|---|---|
| AC-01 / V-S01/05 | 固定原件＋681文件逐项哈希通过。候选1静态声明/动态标量进入既有Node通路，在light3d/三叶属性拒绝；候选2既有refs通用服务set_energy拒绝。两种入口均无完整可达链，不以补造API凑go。 |
| AC-02 / V-S01/02 | frozen pilot真实构建/CLI：正常cylinder成功exit0；light3d unknown element exit2；direction/energy/shadow unknown prop exit3；Life.Tick refs.receiver.set_energy unknown service exit3。源码surface_runtime.rs:1152仍调用sample_light，无作者参数消费者。真实Auto→light→GPU不可达，定位为作者/受检描述/适配端口gap。Rust seam GPU成立不冒全链go。 |
| AC-03 / V-S02/03 | cargo test --locked --test plan059 backend_light_shadow_and_lifetime -- --nocapture --test-threads=1 实际1pass。独立CPU公式在energy0/.5与光向反转/缩放、shadow on/off/去pass/移动caster通过。shadow RGB(89,99,108)，lit(135,149,162)，开关/去pass对照可区分。零方向、NaN方向、Inf/NaN能量、退化投影均实跑且backend没有拒绝（像素依次89/89/255/0/149红通道），这是不安全准入gap、非fallback支持，不伪造合法结果。 |
| AC-04 / V-S03/04 | 两mesh上传344B/4次；双实例每帧352B/3write(64 camera+160 light+128 instance)。第三法线mesh为独立单面子case，总516B；重复identity零新增。20轮backend pipeline重建/drop，每轮64→128尺寸＋零尺寸拒绝。GPU physical free bytes未知，未用drop推显存完全释放。shader编译与pipeline创建合计20901µs，device703122µs，mesh389µs；双实例阴影CPU submit225µs+wait总395µs，非GPU时间。CPU Tick/credits/steps因无作者桥不可达，GPU timestamp/分离shader编译/显存驻留不可用。mesh/cache=MeshStore、uniform/shadow/depth=MeshPipeline、color=OffscreenTarget；2048²Depth32Float名义16MiB；实例buffer容量512*64B（非实测驻留）。 |
| AC-05 / V-S04/05 | author_probe.py真实20轮坏源exit2→同路径修复→下次新编译进程exit0；resource_restore_next_load真实坏GLB拒绝→恢复原字节→下一次load成功。已有threed_contract glb_failure_taxonomy_missing_and_corrupt 1pass。20轮只是compiler/backend fixture；作者Light driver Run/Stop/原生输入因缺端口未执行，明确未证，不用fixture充产品Run。旧024/034/043源码、所有锁/SDK保持原字节，交付仅新增研究Spec，不重复旧GPU全套。 |
| AC-06 / V-S06 | SD-01新增研究Spec与本表一致：no-go普通作者全链，backend-go有限微样本；最小后继端口、有限非零方向/有限非负能量/合法投影、GPU前拒绝和last-good、真正Run/Stop owner门明确。无AG022目录覆盖率宣称。 |

无未决合同问题/实现修复finding；不可达支路均是允许no-go研究的实际结论，不作为产品支持验收。review须独立判定是否充分；review/merge T-07..10仍待执行。主树他会话文档/PNG未动。实际所有源码、tar、target、fixture、日志仅在登记scratch auto-ui子scope；无跨仓输出。

重建配方（关闭后从Plan恢复以下两个文件，原始日志会删除）：`git archive e02ca9f8 experimental/autographics-prototype experimental/autographics-node2d-pilot crates/auto-ui-compiler crates/auto-ui-dialect` 解包到新scratch auto-ui；`git -C D:/autostack/auto-lang archive 593c33480`解包到其lang-fixed，pilot/两UI crate清单中的../../../auto-lang/仅路径重定位到lang-fixed绝对正斜杠路径，锁不变。原历史依赖不依赖正在变动的主lang。GPU：在prototype执行上述cargo test --locked --test plan059；作者：cargo build --locked --manifest-path <pilot>/Cargo.toml后python author_probe.py。首次测试前TOML路径曾误写反斜杠（parse error，已修为正斜杠），未改依赖版本；候选初版单行语法有解析歧义，改为逐行后取得具名拒绝。早期阴影实验投影与光向未一致，已改为同一几何关系后重跑取现证据。source-only probe保持生产shader未改。

证据scope与SHA256（本地瞬态，merge删除前摘要折本Plan）：

- `D:/autostack/plan-artifacts/plan-059/auto-ui/author_probe.py` SHA256 `1d5d4cf908c7b882e25b911ab0e3a0a03642dbb6c671905cdd9453b763c8f9de`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/experimental/autographics-prototype/tests/plan059.rs` SHA256 `71e450f7090c063db86857733da651d34f4f241cb2944137fe64ff5e49a6d0e7`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/gpu.log` SHA256 `5d4ec94db1ec880b194ed1c581ec4a63468dbbf10c0bffb47185a14c0774b7c8`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/author-results.json` SHA256 `9c4470c7bb63e41bdfd7091469fc0e23c053b97898def6abfee3d903ec31892c`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/resource.log` SHA256 `d780d0b42d779601bf7267b4e64a627cd8e734a8477f428daa50e62cb438b319`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/resource-restore.log` SHA256 `6e02ebf148b6749ed67fbedd4bd6ac3e5f64449da6782c018c452d63afa5986b`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/author-build.log` SHA256 `5969a8e30e59cebc80cf9acc6743b01fadb11c9cf22189d05b2c2df644feb4ee`

<details><summary>可重建研究GPU探针（不作为生产API）</summary>

```rust
// Research only; fixed e02ca9f8 renderer remains unmodified.
use agp::{assets::{ModelData,MeshData,MaterialDef}, render3d::{LightDef,MeshPipeline,MeshStore,MeshInstance},renderer::{GpuDevice,target::OffscreenTarget,capture::capture_png}};
use glam::{Mat4,Vec3};
use std::time::Instant;
#[test]
fn resource_restore_next_load(){
 let original=std::fs::read("assets/squash_the_creeps/art/player.glb").unwrap();
 let path=std::path::Path::new("../../resource-recovery.glb");
 std::fs::write(path,b"bad glb").unwrap();assert!(agp::assets::load_glb_file(path).is_err());
 std::fs::write(path,&original).unwrap();let restored=agp::assets::load_glb_file(path).unwrap();assert_eq!(restored.meshes.len(),3);
 std::fs::remove_file(path).unwrap();println!("bad-resource -> restored -> next actual load success; NOT driver Run");
}
fn quad(id:&str, half:f32,z:f32)->ModelData {
 ModelData{source_sha256:id.into(),meshes:vec![MeshData{name:"quad".into(),positions:vec![[-half,-half,z],[half,-half,z],[half,half,z],[-half,half,z]],normals:vec![[0.,0.,1.];4],indices:vec![0,1,2,0,2,3],material:MaterialDef{name:"self-authored".into(),base_color:[0.4,0.5,0.6,1.],metallic:0.,roughness:1.},visual:None}]}
}
fn srgb(x:f32)->u8 { let y=if x<=0.0031308 {12.92*x}else{1.055*x.powf(1./2.4)-0.055};(y.clamp(0.,1.)*255.).round() as u8 }
fn expect(p:[u8;4],lit:f32) {for (i,c) in [0.4,0.5,0.6].iter().enumerate(){assert!((p[i] as i32-srgb(c*lit) as i32).abs()<=3,"pixel {p:?} lit={lit}");}}
fn draw(g:&GpuDevice,p:&mut MeshPipeline,s:&MeshStore,ins:&[MeshInstance],l:&LightDef,on:bool,size:u32)->image::RgbaImage {
 let target=OffscreenTarget::new(g,size,size).unwrap();
 let before=p.uploads.snapshot(); let t=Instant::now();
 let stats=p.render(g,target.view(),size,size,[0.,0.,0.,1.],&Mat4::IDENTITY,l,s,ins,on);
 let submit=t.elapsed().as_micros();g.wait_idle();let wait=t.elapsed().as_micros();
 assert_eq!(stats.instances,ins.len() as u32);
 let delta=p.uploads.snapshot().delta_since(&before);assert_eq!(delta.bytes_rewritten,224+64*ins.len() as u64);assert_eq!(delta.buffer_rewrites,3);
 println!("frame size={size} instances={} shadow={on} submit_us={submit} submit_wait_us={wait} bytes={}",ins.len(),delta.bytes_rewritten);
 capture_png(g,&target).unwrap().1
}
#[test]
fn backend_light_shadow_and_lifetime(){
 let t=Instant::now();let g=GpuDevice::new().unwrap();println!("adapter={:?} device_us={}",g.adapter_info(),t.elapsed().as_micros());
 let mut store=MeshStore::default();let recv=quad("receiver",0.95,0.3);let cast=quad("caster",0.18,0.7);
 let t=Instant::now();for m in [&recv,&cast]{store.get_or_upload(&g,m,0).unwrap();}println!("mesh_upload_us={} counters={:?}",t.elapsed().as_micros(),store.uploads.snapshot());
 let uploads=store.uploads.snapshot();
 let t=Instant::now();let mut p=MeshPipeline::new(&g,OffscreenTarget::FORMAT).unwrap();println!("pipeline_create_us={}",t.elapsed().as_micros());
 // Camera projects +z depth; shadow projection uses x+z and reversed z depth.
 // Receiver P=(.3,0,.3) intersects caster z=.7 at x=-.1, inside .18.
 // P=(-.6,0,.3) intersects at x=-1.0, outside. Neither camera ray hits caster.
 let vp=Mat4::from_cols_array(&[1.,0.,0.,0., 0.,1.,0.,0., 1.,0.,-1.,0., -0.4,0.,1.,1.]);
 let mut l=LightDef{dir_to_light:Vec3::new(1.,0.,-1.).normalize(),color:[1.;3],energy:0.5,shadow_vp:vp,shadow_bias:0.0005};
 let r=MeshInstance{mesh:("receiver".into(),"quad".into()),model:Mat4::IDENTITY};
 let c=MeshInstance{mesh:("caster".into(),"quad".into()),model:Mat4::IDENTITY};
 // Front normal +z: Lambert controls use +z; shadow cases use (-1,0,1).
 l.dir_to_light=Vec3::Z;
 let full=draw(&g,&mut p,&store,&[r.clone()],&l,false,128);expect(full.get_pixel(83,64).0,0.75);
 l.energy=0.;let zero=draw(&g,&mut p,&store,&[r.clone()],&l,false,128);expect(zero.get_pixel(83,64).0,0.25);
 l.energy=0.5;l.dir_to_light=-Vec3::Z;let reverse=draw(&g,&mut p,&store,&[r.clone()],&l,false,128);expect(reverse.get_pixel(83,64).0,0.25);
 l.dir_to_light=Vec3::Z;
 l.dir_to_light=Vec3::new(-1.,0.,1.).normalize();let lit=0.25+0.5/(2f32).sqrt();
 let on=draw(&g,&mut p,&store,&[r.clone(),c.clone()],&l,true,128);
 let off=draw(&g,&mut p,&store,&[r.clone(),c.clone()],&l,false,128);
 println!("shadow pixels on={:?}/{:?} off={:?}",on.get_pixel(83,64),on.get_pixel(25,64),off.get_pixel(83,64));
 expect(on.get_pixel(83,64).0,0.25);expect(on.get_pixel(25,64).0,lit);expect(off.get_pixel(83,64).0,lit);
 p.set_shadow_sabotage(true);let sabot=draw(&g,&mut p,&store,&[r.clone(),c.clone()],&l,true,128);expect(sabot.get_pixel(83,64).0,lit);p.set_shadow_sabotage(false);
 let moved=MeshInstance{model:Mat4::from_translation(Vec3::new(0.7,0.,0.)),..c};let im=draw(&g,&mut p,&store,&[r.clone(),moved],&l,true,128);expect(im.get_pixel(83,64).0,lit);
 l.dir_to_light=Vec3::Z;
 // Nonuniform scale: independent inverse-diagonal normal oracle.
 let mut tilted=quad("tilted",0.8,0.4);for n in &mut tilted.meshes[0].normals{*n=[1.,0.,1.];}store.get_or_upload(&g,&tilted,0).unwrap();
 let tilted_i=MeshInstance{mesh:("tilted".into(),"quad".into()),model:Mat4::from_scale(Vec3::new(2.,1.,0.5))};
 let im=draw(&g,&mut p,&store,&[tilted_i],&l,false,128);expect(im.get_pixel(83,64).0,0.25+0.5*2./(4.25f32).sqrt());
 for i in 0..20 {let mut session=MeshPipeline::new(&g,OffscreenTarget::FORMAT).unwrap();assert!(OffscreenTarget::new(&g,0,128).is_err());for size in [64,128]{let im=draw(&g,&mut session,&store,&[r.clone()],&l,false,size);expect(im.get_pixel(size/3,size/2).0,0.75);}drop(session);g.wait_idle();println!("backend_session={i} dropped; driver Run/Stop NOT exercised");}
 // Raw backend invalid domains: no checked LightDef admission exists.
 for (name,dir,energy,shadow_vp) in [("zero_direction",Vec3::ZERO,0.5,vp),("nan_direction",Vec3::new(f32::NAN,0.,1.),0.5,vp),("infinite_energy",Vec3::Z,f32::INFINITY,vp),("nan_energy",Vec3::Z,f32::NAN,vp),("degenerate_projection",Vec3::Z,0.5,Mat4::ZERO)] {
  let bad=LightDef{dir_to_light:dir,energy,shadow_vp,..l};
  let im=draw(&g,&mut p,&store,&[r.clone()],&bad,true,128);
  println!("invalid_domain={name} backend_did_not_reject pixel={:?}; NOT a supported fallback",im.get_pixel(83,64));
 }
 // Repeated identity must reuse mesh buffers; third mesh above is separate.
 let pre=store.uploads.snapshot();store.get_or_upload(&g,&recv,0).unwrap();assert_eq!(store.uploads.snapshot().bytes_rewritten,pre.bytes_rewritten);
 println!("initial_meshes={uploads:?} final_meshes={:?}; depth release inferred from owner drop, physical GPU free bytes unavailable; GPU timestamps unavailable",store.uploads.snapshot());
}

```
</details>

<details><summary>可重建作者候选与恢复探针</summary>

```python
import pathlib,subprocess,time,json,hashlib
r=pathlib.Path(__file__).parent
exe=r/'experimental/autographics-node2d-pilot/target/debug/node2d-lower.exe'
cases={
 'accepted':'node Main { view-3d { cylinder3d receiver { position: (0.0, 0.0, 0.0) } } }',
 'declaration':'node Main { view-3d { light3d sun { direction: (1.0, 1.0, 0.0) energy: 0.5 shadow: true } cylinder3d receiver { position: (0.0,0.0,0.0) } } }',
 'direction':'node Main { view-3d { cylinder3d receiver { position: (0.0,0.0,0.0) direction: (1.0,1.0,0.0) } } }',
 'energy':'node Main { view-3d { cylinder3d receiver { position: (0.0,0.0,0.0) energy: 0.5 } } }',
 'shadow':'node Main { view-3d { cylinder3d receiver { position: (0.0,0.0,0.0) shadow: true } } }',
 'service':'node Main { model { var energy = 0.5 } view-3d { cylinder3d receiver { position: (0.0,0.0,0.0) } } on { Life.Tick -> { refs.receiver.set_energy(.energy) } } }',
 'bad_source':'node Main { view-3d { cylinder3d receiver { position: ( } } }',
}
cases={k:v.replace("{", "{\n").replace("}", "\n}\n").replace(" energy:","\n energy:").replace(" shadow:","\n shadow:") for k,v in cases.items()}
out=[]
for name,src in cases.items():
 d=r/'author-inputs'/name;d.mkdir(parents=True,exist_ok=True);(d/'main.at').write_text(src)
 t=time.perf_counter();p=subprocess.run([str(exe),'--profile','components-3d','--entry',str(d/'main.at'),'--out',str(d/'out')],capture_output=True,text=True)
 out.append(dict(case=name,exit=p.returncode,elapsed_ms=(time.perf_counter()-t)*1000,diagnostic=p.stderr[-1800:],source=src))
 assert (p.returncode==0)==(name=='accepted'),out[-1]
# Recovery here is a fresh compiler process, NOT actual driver Run/Stop.
d=r/'author-inputs'/'recovery';d.mkdir(parents=True,exist_ok=True)
for i in range(20):
 for kind in ['bad_source','accepted']:
  (d/'main.at').write_text(cases[kind]);p=subprocess.run([str(exe),'--profile','components-3d','--entry',str(d/'main.at'),'--out',str(d/'out')],capture_output=True,text=True)
  assert (p.returncode==0)==(kind=='accepted'),(i,p.stderr)
out.append(dict(compiler_recovery_rounds=20,driver_run_stop='unreachable: no light port'))
(r/'author-results.json').write_text(json.dumps(out,indent=2),encoding='utf-8')
print(json.dumps(out,indent=2))

```
</details>

2026-11-05 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-059/auto-ui | commit: 7b99d58273b6041efa531afabc451cdbcc11f55e | next: review


2026-11-05 / stage: review | review_scope: final | reviewer_identity: independent agent context `/root/review059`（与实施上下文分离；未暴露可核验模型/会话ID，不冒称特定型号） | plan_id: PLAN-059 | plan_revision: r1 | outcome: **needs_fix** | worktree_path: D:/autostack/.wt/ui-059/auto-ui | branch: codex/plan-059-dev | reviewed_commit: 7b99d58273b6041efa531afabc451cdbcc11f55e | base_commit: fa6833bfd106035ed4a50210b61560b61b6921cf | next: work，同树补研究候选与寿命/恢复证据后提交，重审受影响AC，不merge。

复审基线：入口与结论前均核 `git worktree list --porcelain` / `git status --short` / `git rev-parse HEAD`；唯一已登记树、分支与交接一致，实施树干净，diff仅新增54行研究Spec，`git diff --check base..HEAD` exit0。共享Plan读取前SHA256 `6253ca8355b5bb55cd5a65e641a42f0ce4ff9e152e354f7ef00017ec3d7eddc8`；本次仅状态/进度/证据修正，不改变r1合同。技能source SHA256 `3d9263331895dd09cff5015783d9fc21b8be8441505d4101d52f8197d6d2a1f7`。dependency_revisions：fixed/author/sdk tar PAX commit均e02ca9f8452555db2d44e66363ebf5e317b5716f，lang tar为593c334805909c25518b7443f9a11707712a45da；SDK source-lock SHA256 `f5d7f1f8e3e2bfedf70775ee4b9741080d712b809640ee334ae075a335a54959`，独立逐文件核681/681、0 mismatch。fixed archive159文件原字节一致；author archive1720文件仅3个Cargo.toml路径重定位不同（其余1717一致），未见生产渲染源修改。pipeline哈希与T-01一致。

spec_inputs（SHA256）：node3d-foundation.md=`9451cf0f3c6a82505caa771b54728f1a6fb06a2ff74343052dd164dd2aee155a`；gallery-node3d-textures-and-animation.md=`93e9ae2a191fed645ffb9dc84a3547370b849a9caa01a1b010bd6f906a151d8d`；node3d-model-object-templates.md=`c25e513605f882cc80aff276870693f2b46dc992fd27e6f72494bf8565e15436`。冻结SD-01为该reviewed_commit中`docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md`的完整新增内容，SHA256 `4faddde84eb75d79e02e8ec074364b9efe426f309534aaaa89933beb632c7d70`，可用`git show 7b99d58273b6041efa531afabc451cdbcc11f55e:docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md`恢复；现有生产端口缺口描述真实，但不足以关闭本Plan的研究adapter合同，F-059-01/02关闭后须重定结论。new/supersedes/touched_goals字段目标路径与GOAL-008一致，没有ledger/index写入。

| AC → task / case | acceptance_results | 复审方法、结果与覆盖边界 |
|---|---|---|
| AC-01 → T-01/02 / V-S01/05 | partial | 来源/SDK/支持域可信；缺完整研究候选，见F-059-01。 |
| AC-02 → T-02/03 / V-S01/02 | partial | light3d/三属性/set_energy诊断确证生产API缺口；尚不能证明合同允许的静态描述＋Auto动态标量研究桥不可达。 |
| AC-03 → T-01/03/04 / V-S02/03 | partial | backend独立GPU重放1pass，CPU Lambert/遮挡、shadow on/off/去pass/移动/缩放与原结果一致；零/NaN/Inf/退化矩阵确实未拒绝、已明确为gap。作者值经受检adapter的合法/拒绝路径仍未证。 |
| AC-04 → T-04/05 / V-S03/04 | partial | mesh344B/4写、双实例352B/3写/owner观察和计时单位正确；GPU时间不可用明示合格；Run/Stop与Tick沿可达候选仍未完成，见F-059-02。 |
| AC-05 → T-01/05 / V-S04/05 | fail | 旧源码/锁未动，直接loader恢复与compiler恢复可信；这些未走候选下一次实际Run的恢复与last-good/owner路径。 |
| AC-06 → T-06 / V-S05/06 | partial | 已证边界写实，无PBR/AG022覆盖率冒称；研究全链no-go的候选覆盖不足，SD/建议需随补证校准。 |
| AC-07 → T-07 | fail | 本次独立final为needs_fix；work及review任务不可标完。 |
| AC-08/09 → T-08..10 | pending merge-owned | 未实际落主/归档/清理，禁止提前完成。 |

独立重放：在scratch prototype执行`cargo test --locked --test plan059 backend_light_shadow_and_lifetime -- --nocapture --test-threads=1`，exit0，1passed/0failed/1filtered；RTX4060Ti/Vulkan，shadow `(89,99,108)`、lit/off `(135,149,162)`；两个mesh344B/4写、第三单独法线子case总516B。复审CPU时间device748181µs/pipeline22076µs/mesh306µs，仅观察，不替换既有性能保证。raw `D:/autostack/plan-artifacts/plan-059/auto-ui/review-r1-gpu.log` SHA256 `a399c78224ba4c3ef996c2991e49d79f739f248fafe18b9d2552b118b57423f3`。原probe/log七份哈希均与work表匹配；未运行无关全仓/旧GPU全套。后续可复用此未改backend证据，只补候选桥与受影响数值/恢复/owner路径。

**F-059-01 / P1 / 原合同研究候选未完成 / AC-01/02/03/06，T-02/03/06，V-S01/02/03/06。** 违反§2和T-02明确列出的“静态描述＋Auto动态标量绑定，或等价受检通用服务”候选；当前`author_probe.py`的两组输入只是尝试不存在的`light3d`/属性与`refs.receiver.set_energy`。真实exit2/3证明生产API不存在，不能证明正常Auto状态经研究描述/adapter进入LightDef的链不可达；Rust测试中光向/能量/开关全由Rust直接赋值，未执行Auto状态，故不能以它关闭候选合同。已证原因是该研究桥根本未实施/执行，并非已证SDK或GPU做不到。独立合法控制在同一冻结pilot中编译如下输入exit0（CLI为`node2d-lower --profile components-3d --entry <scratch>/review-r1-dynamic-scalar/main.at --out <scratch>/review-r1-dynamic-scalar/out`）：

```auto
node Main {
 model {
  var energy = 0.5
  var shadow = 1
  var lx = 1.0
  var lz = 1.0
 }
 view-3d {
  cylinder3d receiver {
   position: (0.0, 0.0, 0.0)
  }
 }
 on {
  Life.Tick -> {
   .energy = .energy + 0.1
   .shadow = 0
   .lx = 1.0
  }
 }
}
```

该输入SHA256 `1463700968f12c17fc44e7461728d18e506489167b7626d0469e520b151e86dc`，生成world SHA256 `d0f5d8157e53006ac2d5407804a4c60a6b1536a415f4717c9efa4311ddb14d6f`；真实输出含`r_energy = (r_energy + 0.1)`、`r_shadow = 0`及`fn get_r_energy() float`/`get_r_shadow() int`/`get_r_lx()`/`get_r_lz()`。这只证已有合法标量出口，未声称该控制已跑VM或GPU；初稿将lx初始化为-1.0时被既有root numeric-literal限制拒绝，改用合法正literal后通过，属探针输入修正、非新增产品finding。源码`lower_components3d.rs`的model getters/app_start/app_update与既有driver Session消费测试提供值得实际尝试的接缝，是否可用仍待执行。关闭要求：在≤2完整研究候选预算内，真实执行普通Auto声明/局部事件，经受检数据映射改变方向/energy/shadow并观察GPU，或以真实接缝失败证明研究链no-go；分别保留backend-go/产品API-gap结论。不得新增正式生产Light API来凑验收，也不得把“生产Light API不存在”换成研究候选已失败。最小复核范围为该候选、三个参数、真实状态变更、有效/无效准入与对应稳定像素，不需旧M2全套。

**F-059-02 / P1 / 必需寿命与恢复oracle未执行 / AC-04/05/06，T-04/05/06，V-S04。** §5/T-05要求20轮Run/Stop、resize、坏源/失效描述拒绝→修复→下一次实际Run恢复；当前source注释/日志都明确“driver Run/Stop NOT exercised”，20轮只重建/丢弃裸MeshPipeline，坏源只启动新compiler进程，坏资源只直调load_glb_file。因此没观测真实候选会话生命周期、状态重置、受检描述/资源恢复的下一次读取；孤立组件成功不满足同一路径闭环。作者链未被尝试（F-059-01），故“无Light端口”不能作为免跑所有真实候选Run/Stop的理由。关闭要求：沿完成或失败的研究候选实际入口验证20轮start/tick/stop/resize、owner释放/允许保留的缓存边界，以及坏源/描述/资源具名拒绝后修复并在下一Run真读取恢复；若只能受控研究runner而无原生输入，按原合同明确该层次，不冒生产Host/原生输入已证。若候选接缝确实不可达，须记录其实际失败与受阻分支，重新按原合同判断可关闭性；不能自动豁免此oracle。最小复核仅候选生命周期/恢复与其owner，不重复无影响的GPU公式或全回归。

pending_closeout_gates：T-08..10仍全部pending；此轮未merge/canonical发布/归档/清理。**T-01证据保留完成，T-02..07因未完子项及依赖证据重开，executing 1/10**；历史work“无未决finding”和T-07先前勾选被本轮结论覆盖，原记录保留。实现写入在本轮暂停；复审未改实现、Spec、锁、SDK、他会话WIP或PNG。next：work在同树补F-059-01/02并提交新baseline，提供closure表后由本独立上下文定向复核；不要求无关全套重跑。

2026-11-05 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/ui-059/auto-ui | commit: 7b99d58273b6041efa531afabc451cdbcc11f55e | next: work. 本轮遵F-059-01/02，仅补同一静态描述/动态标量候选和寿命/恢复，不改合同/生产API/预算；既有backend数学与原件证据复用。

2026-11-05 / stage: work | repair_cycle: 1 | plan_id: PLAN-059 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-059/auto-ui | branch: codex/plan-059-dev | base_commit: fa6833bfd106035ed4a50210b61560b61b6921cf | code_commit: 5ab8844e008d15edf243ac54b83536db4b67a564 | task_ids: T-02..06 | next: 同员独立final定向复核。现结论 **bounded-go（研究静态描述＋Auto动态标量绑定）**，取代首轮不充分的研究no-go；正式产品Light API仍无，不修改r1目标/AC/预算。

| finding → invariant | 修复、永久重建输入/控制、当前结果与剩余边界 |
|---|---|
| F-059-01 → 正常Auto状态真实进入受检GPU接缝 | 实施一个完整候选：bridge-main.at正常Node model lx/lz/energy/shadow/phase，Life.Tick依次改能量0、方向-Z、(-1,0,1)shadow1、shadow0。固定pilot产生原生get_r_*与app_start/app_update；静态JSON绑定指定getters，SDK681真实Session编译并执行main初始化globals、app_start和app_update，受检后生成LightDef/同向投影矩阵写GPU。业务分支全由Node作者源码控制；Rust无业务状态机。仅研究adapter，非新生产API。 |
| F-059-01 → 数值/类型准入与区分oracle | typed getter、finite/非零/长度溢出、energy[0,1]、shadow0/1、投影radius>0/0≤near<far/bias[0,.01]、最终矩阵有限，失败不写GPU。实际Auto getter lx=-1误绑energy被拒；缺getter拒；直接validator零/NaN/Inf/负能量/非法开关/退化投影/有限输入长度溢出拒绝。last-good像素/实际uniform写计数不变。合法控制初帧lit=.75，Tick1=.25，Tick2=.25，Tick3遮挡=.25，Tick4取消阴影=.25+.5/sqrt2；每通道±3独立CPU oracle成立。底层去pass/移动caster/非均匀法线反例沿用未变的首轮及独立GPU复核证据。 |
| F-059-02 → 同一实际研究Run入口/Stop owner/恢复 | Run::start每次真读静态描述、生成源及mesh资源，创建/编译SDK会话、通用main、作者Start、发布GPU。20轮真Run/Tick/resize64→128/零尺寸拒绝/Stop；每轮初态.75和首Tick.25独立重置；Session::owner_probe弱strings/heap在close后失活，pipeline Rc弱owner在drop后失活，warm缓存两mesh仍344B无重传。物理GPU释放字节未测，MeshStore显式保留warm缓存，不把drop计显存释放。 |
| F-059-02 → 坏输入拒绝→精确修复→下一实际Run | 同一Run::start读取路径分别损坏world Auto、descriptor绑定函数、mesh资源half=-1；拒绝candidate且live last-good像素保持；修复原字节，下一Run真实重读/编译/查询/呈现初态.75，三路全部通过。未跑生产Host/原生输入；按原合同为受控研究runner层。 |

验证命令与当前结果：`python <fixed-driver>/tools/prepare_sdk.py --from-snapshot` exit0 / SDK681文件17包逐文件核；固定pilot `node2d-lower --profile components-3d --entry <scope>/bridge-main.at --out <scope>/bridge-product` exit0；在固定driver执行`cargo test --locked --test bridge059 -- --nocapture --test-threads=1` exit0 / 1passed 0failed。最后一次样本真实compile_us=30889、首四Auto_tick_us=62/42/31/27，callback max_steps=40/47/52/52、max_slices1；双mesh frame352B/3write，阴影帧CPU submit220µs/submit+wait328µs。GPU timestamp不可用、shader/pipeline分离计时不可用，不冒GPU ms、旧credits/全局性能或生产UI指标。每轮物理mesh/target/shadow容量沿原pipeline不变。

调试证据：首次直接Session仅app_start导致未初始化global的Float标签返回 mismatch 0000000000000000；按既有driver main-first初始化次序加入通用空main并先执行main，再执行作者app_start，随后真实Auto状态/像素/owner全通过。不是Rust赋业务初值。第二版资源key按bridge-mesh.json原始字节SHA+mesh名，避免资源内容变更复用假identity；未更改production decoder、锁或任何SDK文件。有限方向长度溢出另加拒绝反例，测试在最终源重跑通过。原gen/driver/prototype生产源码保持e02原字节（仅研究test新增；author侧3manifest路径重定位沿用已核版本）。

readiness：AC-01..06 work-owned现完成；F1/F2无剩余in-contract缺口。支持边界/未证项与修复后的SD-01一致；T-07..10待review/merge。复用来源、backend CPU/GPU oracle、旧兼容字节身份，不重复M2/根全仓；最终合入文档门由merge负责，无生产二进制更改。所有输出仍登记scope下，driver archive、SDK副本、driver target和bridge输入/输出/日志已纳入清理账。

- `D:/autostack/plan-artifacts/plan-059/auto-ui/bridge-main.at` SHA256 `49426259156be8f0e9e28b4725ac4eccda76e90c5de39fb5ce566beff41aaabd`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/bridge-desc.json` SHA256 `bc367928a492cf54fbf19767717e03d8d2b112e8eb3584b13bfa4cf2860a03e1`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/bridge-mesh.json` SHA256 `a59bd97aa283779c15544ccf63889d86756a9d8a14c7d372adf7ed6a7ffafbb3`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/bridge-product/world.at` SHA256 `02454802c88acac4ef5a6d5af369c0418963b7066c4c73f38210f59af74172be`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/experimental/autographics-auto-driver/tests/bridge059.rs` SHA256 `db8b596346fade01dd58d24f377616084c6f1f0f9c592adba20a1cfa50080151`
- `D:/autostack/plan-artifacts/plan-059/auto-ui/bridge.log` SHA256 `a0b3059bbb1edfa6788262bdf6dc1d8f43be56d3bc2202fbd911e868443d9e36`

重建补充：按前述e02 archive再导出完整experimental/autographics-auto-driver，并运行prepare_sdk.py --from-snapshot。把下列输入恢复到scope根，固定pilot降层到bridge-product，把Rust文件恢复到固定driver/tests/bridge059.rs，执行上面同一目标；runner工作目录为driver，../..就是scope根。没有复制生产checkout的SDK或二进制。

<details><summary>研究候选重建源 bridge-main.at</summary>

```auto
node Main {
 model {
  var phase = 0
  var lx = 0.0
  var lz = 1.0
  var energy = 0.5
  var shadow = 0
 }
 view-3d {
  cylinder3d receiver {
   position: (0.0, 0.0, 0.0)
  }
 }
 on {
  Life.Tick -> {
   .phase = .phase + 1
   if .phase == 1 {
    .energy = 0.0
   }
   if .phase == 2 {
    .energy = 0.5
    .lz = 0.0 - 1.0
   }
   if .phase == 3 {
    .lx = 0.0 - 1.0
    .lz = 1.0
    .shadow = 1
   }
   if .phase == 4 {
    .shadow = 0
   }
  }
 }
}

```
</details>

<details><summary>研究候选重建源 bridge-desc.json</summary>

```json
{"bindings":["get_r_lx","get_r_lz","get_r_energy","get_r_shadow"],"radius":1.0,"near":0.0,"far":1.0,"bias":0.0005}

```
</details>

<details><summary>研究候选重建源 bridge-mesh.json</summary>

```json
{"receiver_half":0.95,"receiver_z":0.3,"caster_half":0.18,"caster_z":0.7}

```
</details>

<details><summary>研究候选重建源 experimental/autographics-auto-driver/tests/bridge059.rs</summary>

```rust
use agp::{assets::{ModelData,MeshData,MaterialDef},render3d::{LightDef,MeshPipeline,MeshStore,MeshInstance},renderer::{GpuDevice,target::OffscreenTarget,capture::capture_png}};
use autographics_auto_driver::{Session,ResultKind,Scalar};
use glam::{Mat4,Vec3};
use std::{path::Path,rc::Rc,cell::RefCell,time::Instant};
fn quad(id:&str,half:f32,z:f32)->ModelData{ModelData{source_sha256:id.into(),meshes:vec![MeshData{name:"quad".into(),positions:vec![[-half,-half,z],[half,-half,z],[half,half,z],[-half,half,z]],normals:vec![[0.,0.,1.];4],indices:vec![0,1,2,0,2,3],material:MaterialDef{name:"self-authored".into(),base_color:[0.4,0.5,0.6,1.],metallic:0.,roughness:1.},visual:None}]}}
fn expect(p:[u8;4],lit:f32){for(i,c)in[0.4,0.5,0.6].iter().enumerate(){let x=c*lit;let v=if x<=0.0031308{12.92*x}else{1.055*x.powf(1./2.4)-0.055};assert!((p[i]as f32-v*255.).abs()<=3.,"{p:?}, lit={lit}");}}
#[derive(serde::Deserialize,Clone)]
struct Desc{bindings:[String;4],radius:f32,near:f32,far:f32,bias:f32}
#[derive(serde::Deserialize)]
struct Mesh{receiver_half:f32,receiver_z:f32,caster_half:f32,caster_z:f32}
fn checked(d:&Desc,lx:f32,lz:f32,energy:f32,shadow:i32)->Result<(LightDef,bool),String>{
 if ![d.radius,d.near,d.far,d.bias,lx,lz,energy].iter().all(|x|x.is_finite()){return Err("nonfinite descriptor/scalar".into())}
 if d.radius<=0.||d.near<0.||d.far<=d.near||d.bias<0.||d.bias>0.01{return Err("invalid projection/bias".into())}
 if !(lx*lx+lz*lz).is_finite()||lx*lx+lz*lz<1e-8||lz.abs()<0.01||!(0.0..=1.0).contains(&energy)||!(0..=1).contains(&shadow){return Err("invalid direction/energy/shadow".into())}
 let dir=Vec3::new(lx,0.,lz).normalize();let slope=lx/lz;
 // Orthographic rays have x'=x-lx/lz*z; map light-facing depth to [0,1].
 let (az,bz)=if lz>0.{(-1./(d.far-d.near),d.far/(d.far-d.near))}else{(1./(d.far-d.near),-d.near/(d.far-d.near))};
 let vp=Mat4::from_cols_array(&[1./d.radius,0.,0.,0., 0.,1./d.radius,0.,0., -slope/d.radius,0.,az,0., 0.4*slope/d.radius,0.,bz,1.]);
 if !vp.is_finite()||!dir.is_finite(){return Err("projection/direction overflow".into())}
 Ok((LightDef{dir_to_light:dir,color:[1.;3],energy,shadow_vp:vp,shadow_bias:d.bias},shadow==1))
}
struct Run{vm:Session,pipe:Rc<RefCell<MeshPipeline>>,target:OffscreenTarget,desc:Desc,instances:Vec<MeshInstance>,last:Option<(LightDef,bool)>}
impl Run{
 fn start(g:&GpuDevice,s:&mut MeshStore,root:&Path)->Result<Self,String>{
  let d:Desc=serde_json::from_slice(&std::fs::read(root.join("bridge-desc.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  checked(&d,0.,1.,0.5,0)?;
  let mesh_bytes=std::fs::read(root.join("bridge-mesh.json")).map_err(|e|e.to_string())?;let mesh_id=agp::renderer::capture::sha256_hex(&mesh_bytes);let m:Mesh=serde_json::from_slice(&mesh_bytes).map_err(|e|e.to_string())?;
  if ![m.receiver_half,m.receiver_z,m.caster_half,m.caster_z].iter().all(|x|x.is_finite())||m.receiver_half<=0.||m.caster_half<=0.||m.receiver_z<0.||m.caster_z>1.||m.receiver_z>=m.caster_z{return Err("invalid mesh resource".into())}
  let source=std::fs::read_to_string(root.join("bridge-product/world.at")).map_err(|e|e.to_string())?;
  let assembled=format!("{source}\nfn main() {{}}\n");let mut vm=Session::new(&assembled)?;vm.call("main",&[],ResultKind::Unit)?;vm.call("app_start",&[],ResultKind::Unit)?;
  let mut instances=vec![];
  for model in [quad(&format!("{mesh_id}:receiver"),m.receiver_half,m.receiver_z),quad(&format!("{mesh_id}:caster"),m.caster_half,m.caster_z)]{s.get_or_upload(g,&model,0).map_err(|e|e.to_string())?;instances.push(MeshInstance{mesh:(model.source_sha256,"quad".into()),model:Mat4::IDENTITY});}
  let mut r=Self{vm,pipe:Rc::new(RefCell::new(MeshPipeline::new(g,OffscreenTarget::FORMAT).map_err(|e|e.to_string())?)),target:OffscreenTarget::new(g,128,128).map_err(|e|e.to_string())?,desc:d,instances,last:None};
  r.publish(g,s)?;println!("Run compiled_us={} heap={}",r.vm.compile_us,r.vm.heap_objects());Ok(r)
 }
 fn publish(&mut self,g:&GpuDevice,s:&MeshStore)->Result<(),String>{
  let b=&self.desc.bindings;
  let lx=self.vm.float(&b[0],&[])?;let lz=self.vm.float(&b[1],&[])?;let e=self.vm.float(&b[2],&[])?;let sh=self.vm.int(&b[3],&[])?;
  let accepted=checked(&self.desc,lx,lz,e,sh)?;self.last=Some(accepted);self.render(g,s);Ok(())
 }
 fn render(&mut self,g:&GpuDevice,s:&MeshStore){let(l,on)=self.last.as_ref().unwrap();let mut p=self.pipe.borrow_mut();let pre=p.uploads.snapshot();let now=Instant::now();let stats=p.render(g,self.target.view(),self.target.width,self.target.height,[0.,0.,0.,1.],&Mat4::IDENTITY,l,s,&self.instances,*on);let submit=now.elapsed().as_micros();g.wait_idle();let delta=p.uploads.snapshot().delta_since(&pre);assert_eq!(stats.instances,2);assert_eq!(delta.bytes_rewritten,352);println!("present energy={} dir={:?} shadow={on} submit_us={submit} submit_wait_us={} uniform_bytes={}",l.energy,l.dir_to_light,now.elapsed().as_micros(),delta.bytes_rewritten);}
 fn tick(&mut self,g:&GpuDevice,s:&MeshStore)->Result<(),String>{let t=Instant::now();self.vm.call("app_update",&[Scalar::Float(1./60.)],ResultKind::Unit)?;println!("Auto_tick_us={} max_steps={} max_slices={}",t.elapsed().as_micros(),self.vm.max_call_steps,self.vm.max_call_slices);self.publish(g,s)}
 fn resize(&mut self,g:&GpuDevice,s:&MeshStore,size:u32)->Result<(),String>{let next=OffscreenTarget::new(g,size,size).map_err(|e|e.to_string())?;assert_ne!(next.build_seq,self.target.build_seq);self.target=next;self.render(g,s);Ok(())}
 fn pixel(&self,g:&GpuDevice)->[u8;4]{capture_png(g,&self.target).unwrap().1.get_pixel(self.target.width*83/128,self.target.height/2).0}
}
#[test]
fn author_binding_run_lifecycle(){
 let root=Path::new("../..");let g=GpuDevice::new().unwrap();println!("adapter={:?}",g.adapter_info());let mut store=MeshStore::default();
 let mut r=Run::start(&g,&mut store,root).unwrap();expect(r.pixel(&g),0.75);
 r.tick(&g,&store).unwrap();expect(r.pixel(&g),0.25);
 r.tick(&g,&store).unwrap();expect(r.pixel(&g),0.25);
 r.tick(&g,&store).unwrap();expect(r.pixel(&g),0.25);
 r.tick(&g,&store).unwrap();expect(r.pixel(&g),0.25+0.5/(2f32).sqrt());
 // Failed publication cannot touch GPU uniform/target or last-good state.
 let good=r.pixel(&g);let pre=r.pipe.borrow().uploads.snapshot();r.desc.bindings[2]="get_r_lx".into();assert!(r.publish(&g,&store).is_err());assert_eq!(r.pixel(&g),good);r.desc.bindings[2]="missing_getter".into();assert!(r.publish(&g,&store).is_err());assert_eq!(r.pixel(&g),good);assert_eq!(r.pipe.borrow().uploads.snapshot().bytes_rewritten,pre.bytes_rewritten);drop(r);
 let d:Desc=serde_json::from_slice(&std::fs::read(root.join("bridge-desc.json")).unwrap()).unwrap();
 for (x,z,e,sh) in [(0.,0.,0.5,0),(f32::NAN,1.,0.5,0),(f32::MAX,1.,0.5,0),(0.,1.,f32::INFINITY,0),(0.,1.,f32::NAN,0),(0.,1.,-0.1,0),(0.,1.,0.5,2)]{assert!(checked(&d,x,z,e,sh).is_err());}
 let mut invalid=d.clone();invalid.far=invalid.near;assert!(checked(&invalid,0.,1.,0.5,0).is_err());
 let warm=store.uploads.snapshot();
 for i in 0..20{
  let mut r=Run::start(&g,&mut store,root).unwrap();let vm_live=r.vm.owner_probe();let pipe=Rc::downgrade(&r.pipe);assert!(vm_live());expect(r.pixel(&g),0.75);r.tick(&g,&store).unwrap();expect(r.pixel(&g),0.25);
  assert!(r.resize(&g,&store,0).is_err());expect(r.pixel(&g),0.25);r.resize(&g,&store,64).unwrap();expect(r.pixel(&g),0.25);r.resize(&g,&store,128).unwrap();expect(r.pixel(&g),0.25);
  r.vm.close();assert!(!vm_live());assert!(r.vm.closed());drop(r);g.wait_idle();assert!(pipe.upgrade().is_none());assert_eq!(store.uploads.snapshot().bytes_rewritten,warm.bytes_rewritten);println!("research_Run_Stop={i} vm_released pipeline_owner_released warm_mesh_bytes={}",warm.bytes_rewritten);
 }
 // Next Run reads all inputs afresh; rejected candidates never replace live Run.
 for (path,bad) in [("bridge-product/world.at","not valid Auto {"),("bridge-desc.json","{\"bindings\":[\"missing\",\"get_r_lz\",\"get_r_energy\",\"get_r_shadow\"],\"radius\":1,\"near\":0,\"far\":1,\"bias\":0.0005}"),("bridge-mesh.json","{\"receiver_half\":-1,\"receiver_z\":0.3,\"caster_half\":0.18,\"caster_z\":0.7}")]{
  let path=root.join(path);let original=std::fs::read(&path).unwrap();let live=Run::start(&g,&mut store,root).unwrap();let before=live.pixel(&g);std::fs::write(&path,bad).unwrap();let rejected=Run::start(&g,&mut store,root);assert!(rejected.is_err());println!("rejected {}: {}",path.display(),rejected.err().unwrap());assert_eq!(live.pixel(&g),before);std::fs::write(&path,original).unwrap();let restored=Run::start(&g,&mut store,root).unwrap();expect(restored.pixel(&g),0.75);drop(restored);drop(live);println!("next_Run_recovered {}",path.display());
 }
 println!("research binding candidate complete; controlled runner, no production Host/native input/Light API; GPU timestamps unavailable");
}

```
</details>

2026-11-05 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-059/auto-ui | commit: 5ab8844e008d15edf243ac54b83536db4b67a564 | next: review


2026-11-05 / stage: review | review_scope: final（第二轮，repair-specific） | reviewer_identity: 同一独立上下文 `/root/review059`，未参与实施修复；模型/会话ID未暴露 | plan_id: PLAN-059 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/ui-059/auto-ui | branch: codex/plan-059-dev | reviewed_commit: 5ab8844e008d15edf243ac54b83536db4b67a564 | base_commit: fa6833bfd106035ed4a50210b61560b61b6921cf | repair_base: 7b99d58273b6041efa531afabc451cdbcc11f55e | next: merge-owned T-08..10。

基线/知识：入口与结论前均核唯一登记树、branch、HEAD和clean status，修复diff仅研究Spec（23增/9删），全交付diff无生产代码/锁/SDK更改，`git diff --check base..HEAD` exit0；共享Plan本轮读取SHA256 `488fdb71896c166a972b32d72e44d50079b1ab402a6263726bc489a00de37520`。dependency_revisions/spec_inputs/skill版本沿第一轮完整身份复用（相关输入未变）；新增完整driver归档PAX commit=e02ca9f8452555db2d44e66363ebf5e317b5716f，1488原件逐字节0差异，SDK frozen再次逐文件681/681。修复6份源码/输入/原log哈希独立核验均与work表一致；本轮运行后输入/bridge059.rs哈希仍原值。固定pilot把bridge-main.at冷降层至review-r2-regenerate，exit0，world与bridge-product/world.at逐字节一致。

冻结SD-01：`docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md` SHA256 `8a1fef0421ee943cf45c9e23697e588108228cb27cb707abe7fba64b753ee6b0`，完整副本为`git show 5ab8844e008d15edf243ac54b83536db4b67a564:docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md`。研究bounded-go与当前真实链一致；x/z方向、energy[0,1]、shadow0/1、数值/投影准入、受控runner/逻辑owner边界均如实，正式Light API/生产Host/原生输入/物理显存/GPU时间仍不冒称。new_spec_components/supersedes_spec_components/touched_goals核验无变化、无ledger/index写入。

| AC → task / check | acceptance_results | 本轮关闭依据 |
|---|---|---|
| AC-01 → T-01/02 / V-S01/05 | pass | 仅1完整研究候选，未把首轮不存在API负控计成额外完整候选；来源、双图、支持表明确。 |
| AC-02 → T-02/03 / F-059-01 | pass | 实际Node→既有原生getters→SDK Session main/app_start/app_update→checked→LightDef/GPU；Rust无作者业务状态机。独立作者变异控制亦能改变GPU。 |
| AC-03 → T-01/03/04 / V-S02/03 | pass | 沿用未变backend独立Lambert/几何/去pass/移动/非均匀缩放证据；新桥实际方向/能量/开关像素通过，非法值/错误类型在写GPU前拒绝，追加兄弟反例通过。 |
| AC-04 → T-04/05 / V-S03/04 | pass | 实际VM compile/Tick/steps/slices单位分离；20轮candidate Run/resize/Stop及warm344B、帧352B，逻辑owner断开，不冒物理显存。 |
| AC-05 → T-01/05 / F-059-02 | pass | 同一候选入口三路坏输入均拒绝且live last-good不变，精确恢复后下一次Run真重读/编译/查询/GPU通过；旧源和锁未变证据复用。 |
| AC-06 → T-06 / V-S06 | pass | bounded-go只指该微样本研究桥，SD及最小后继合同与真实输入/拒绝/未证边界一致。 |
| AC-07 → T-07 | pass | 同树同员独立final完成；F-059-01/02全部关闭，无新增finding。 |
| AC-08/09 → T-08..10 | pending merge-owned | 落主、canonical/归档导航、材料对账/清理尚未执行。 |

复核scope：F1→作者状态因果链、3参数/typed和数值准入、SD；F2→候选Run/Stop/resize、owner和三路恢复。原backend/projection/法线/旧兼容证据在source/deps/config/input未变下复用，不重跑无关全套。实际在scratch固定driver执行：

- `cargo test --locked --test bridge059 -- --nocapture --test-threads=1` exit0，1passed/0failed；日志实际20个research_Run_Stop、3个next_Run_recovered。每轮初态.75/首Tick.25；前四Tick能量0、方向-Z、shadow-on、shadow-off的像素与CPU期望一致。VM owner close后失活、Rc pipeline弱owner drop后失活，mesh上传总344B、frame352B。
- 独立holdout：复制bridge-main.at至review-r2-holdout/main.at，仅把首Tick`.energy = 0.0`改成`.energy = 1.0`（作者源SHA256 `7ba422fbe8bed836955f0496e5037fe628eecfb2b8809364ac81a56fd3fd49f0`），同一固定pilot降层至其bridge-product，复制原desc/mesh；另建`tests/review_bridge059.rs`用`include!("bridge059.rs")`复用冻结研究桥，测试以下独立控制。执行`cargo test --locked --test review_bridge059 review_author_mutation_and_rejection_siblings -- --nocapture --test-threads=1` exit0，1passed/0failed/1filtered。真实Auto首Tick energy1→GPU lit1.25（原work输入为lit.25），排除了Rust替代作者变化。合法3组`(lx,lz,energy,shadow)=(0,1,0,0)/(0,-1,1,1)/(-1,1,.5,1)`；非法10组：radius0、radius=MIN_POSITIVE*.01、near=-1、far=0、bias=.0101、bias=NaN、(1,0,.5,0)、(Inf,1,.5,0)、(0,1,1.01,0)、(0,1,.5,-1)，全部按预期接受/拒绝。实际shadow getter误绑float `get_r_energy`返回`declared return type mismatch`，pixel、uniform写字节和last.energy1均保持原值。

| finding | closure | 剩余边界 |
|---|---|---|
| F-059-01 | closed，完整静态标量binding候选与Auto源因果变异、数值/类型准入实跑成立 | 正式Light语法/API仍缺；研究桥非稳定生成器协议。 |
| F-059-02 | closed，研究runner20轮寿命/resize与三路下一Run恢复均独立重放 | 非生产Host/原生输入；逻辑owner释放不等物理GPU free bytes。 |

本轮raw证据均在同一登记scope：review-r2-bridge.log SHA256 `fc0b9c14d8dfad7adddfac8f1caf8f5a09d47f772fef581815ac8771a090af85`；review-r2-holdout.log SHA256 `df0749439014db4c7526142458caaaf8926a9e6d588e562a29b6a4b29d82da4f`；追加review test SHA256 `b4d7bacf622fab256a6a646019d7911125d3f1ab349cb10ab8a211308e16c57a`。这些review-r2*材料及tests/review_bridge059.rs/target产物一并由T-10清理，无跨scope输出。独立控制可由上面精确输入变异、合法/非法组和三个不变断言复建；生产实现/原研究输入未由复审修改。

结论状态 **reviewed 7/10**：T-01..07实际关闭（T-07此前work预勾不作为证据，本收据才授予final pass）；pending_closeout_gates为T-08..10的受影响文档内容/链接/格式、来源/HEAD最终核验、实际合并与归档导航和完整scratch/构建清理。没有生产二进制变更，不需要重复全仓/旧GPU统一门。后续任何语义/研究输入或SD改动须重定向复核；本次不merge、不发布canonical、不归档、不清树。

2026-11-05 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-059/auto-ui | commit: 5ab8844e008d15edf243ac54b83536db4b67a564 | next: merge.

2026-11-05 / stage: merge | plan_id: PLAN-059:r1 | outcome: in_progress | prepared/landed/canonical_verified: pass | reviewed_commit: 5ab8844e008d15edf243ac54b83536db4b67a564 | delivery_commit: ab5a2e64b95206be8bd09be28001cfaffb3ba646 | canonical: docs/specs/auto-graphics/node3d-lighting-shadow-feasibility.md | canonical_sha256: 8a1fef0421ee943cf45c9e23697e588108228cb27cb707abe7fba64b753ee6b0. 最终delta=68行研究Spec＋3处059归档导航（054旧行仅换行格式归一化，无语义变化）。review后仅导航文档后继，研究源码/依赖/SD逐字未变；git rebase master reported up-to-date，无hash改写；git merge --ff-only实际从fa6833bf落到ab5a2e64，主tip匹配，Spec哈希与final冻结值一致。文档门diff --check exit0、source/导航059目标核验；不跑无生产变更的全仓/旧GPU门。未写ledger/index。初次bash误用WSL找不到Windows路径，改用C:/Program Files/Git/bin/bash.exe执行同一wt-guard，实际clean（无reparse）；无绕过guard。

archive checkpoint：同号Plan移至docs/plans/archived/059-age-node3d-lighting-and-shadow-feasibility.md，status archived/completion_kind delivered，T08/09完成；相对设计链接已调至新父目录，3处导航指向同号归档。docs commit仍由下一条实际Git确认；T10清理pending，不提前记cleaned。

主checkout既有其他计划文档/计数器/测试PNG仍保留，不纳入059提交。README临时path限定stash→ff→apply，恢复前后逐内容比较，除059新导航行外按原始bytes恢复，临时stash已drop；snapshot在本scope，恢复成功后随scope删除。此处清洁检查限定本计划所有权；全主树因已知并发非源码WIP非空，未宣称全树clean、未擅自提交/删除其他owner改动。

landing-not-deployment：本交付纯研究知识，无production source/lock/SDK变化；backend/daemon/web bundle均无新增消费二进制要求，未重启现有进程；自动应用索引非阻塞。auto-lang周期全仓batch门不属本仓文档研究落地主腿。

2026-11-05 / merge artifact audit: archive docs committed fb116b0a; scope plan-059 only auto-ui; 38238 files / 18926786672 bytes, no links/junctions. Own paths = fixed/author/driver/sdk/lang archives, prototype/pilot/driver sources+targets, frozen+.sdk-source SDK, author lang-fixed/crates, inputs/products/probes/logs, review-r1/r2 controls, restored README snapshot/stash record. Necessary reconstruction sources and evidence folded above; no unresolved holds. Fresh guards and actual removal still pending.

2026-11-06 / stage: merge | plan_id: PLAN-059:r1 | outcome: blocked (cleanup only) | delivery_commit: ab5a2e64b95206be8bd09be28001cfaffb3ba646 | archive_commit: fb116b0a21d090e85e3d9c33ac3f9fd6471d7a57 | canonical_verified: pass | cleaned: partial. Fresh Git-Bash guard clean for worktree and whole scope; master ancestry passed; worktree and branch removed; empty ui-059 group removed. Automatic approval rejected recursive native PowerShell cleanup of the verified own plan-059 scope twice before execution, reporting only “blocked by policy”. No alternative recursive deletion attempted. Last inventory: 38238 files / 18926786672 bytes. Source/evidence reconstruction and review controls are already folded above. T10 remains pending; no goal completion claim.

Named cleanup hold: owner=Codex PLAN-059 closeout / user filesystem operator; scope=D:/autostack/plan-artifacts/plan-059/auto-ui; reason=automatic approval refuses recursive cleanup; closure=remove verified own plan-059 scope, verify absence, update registry, close T10 and goal. No unresolved implementation finding. Existing unrelated main docs/PNG remain preserved.

2026-11-06 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 5 | from: merge | to: user | status: archived | outcome: blocked | worktree: removed | commit: ab5a2e64b95206be8bd09be28001cfaffb3ba646 | next: user. First goal turn with this blocker; host goal remains active until its three-consecutive-turn threshold. Resume cleanup only after external state changes, without repeating implementation or GPU gates.

2026-11-06 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 6 | from: user | to: user | status: archived | outcome: waiting | worktree: removed | commit: ab5a2e64b95206be8bd09be28001cfaffb3ba646 | next: user. 上轮属于实际进展（落主/归档/清树），本轮重新读取§10并核磁盘：plan-059暂存目录仍存在，权威工作树/分支已不存在；未收到清理完成或审批限制变化的答复。同一清理阻碍连续第2个goal turn，未重试已被拒绝的删除、未重复GPU/实现验证。T10仍pending、9/10，goal按host阈值保持active。

2026-11-06 / stage: goal | plan_id: PLAN-059 | plan_revision: r1 | round: 7 | from: user | to: user | status: archived | outcome: blocked | worktree: removed | commit: ab5a2e64b95206be8bd09be28001cfaffb3ba646 | next: user. 前轮分类为no progress（仅重新确认阻碍并记账，无活跃进程可等待）；本轮再次核§10及磁盘，plan-059 scope仍存在，工作树/分支仍已移除，没有外部状态变化或用户清理答复。同一审批清理阻碍连续第3个goal turn，已达host blocked阈值；目标转blocked，停止自动续作。T10/9-of-10与具名hold保留；只待用户清理精确scope或审批条件改变后恢复收口，不重做已通过研究/复审/落地主腿。

## 10. 待澄清事项

C-059-cleanup：请用户手动清理 `D:/autostack/plan-artifacts/plan-059/`（约18.9GB；只有本计划auto-ui子目录），或修复自动审批对此精确自有路径清理的限制，之后继续本goal。不得清理plan-artifacts根或其它编号目录。工具只报告“blocked by policy”，没有更具体理由。研究交付/独立复审/归档均完成；T10待清理核验。

D-01：正常作者端口不存在时的两候选由T-02真链裁决，不能因底层有LightDef假定作者已支持。D-02：数学像素容差/投影域在实际环境T-01冻结；GPU计时不可用就明牌。D-03：本研究不绑定尚无结论的three.js首批覆盖，不提前认领PBR。材料唯一`D:/autostack/plan-artifacts/plan-059/auto-ui/`，先核跨仓owner/登记用途在途，研究源/原始结果/SDK副本/target/外部消费均子scope；merge必要源命令和成败摘要折Plan后核所有输出，fresh guard清唯一树/删自有scope与构建物，具名owner/理由/关闭条件hold例外。禁止链接/触碰别仓同号/另建权威树。
