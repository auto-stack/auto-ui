---
plan_id: PLAN-005
status: executing
feature_name: AutoGraphics有限纹理模型与UV资源闭包
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 2
current_step: 1
total_steps: 10
depends_on: [auto-ui/PLAN-004]
supersedes_spec_components:
  - docs/specs/auto-graphics/scene-assets.md
new_spec_components:
  - docs/specs/auto-graphics/textured-model-resources.md
touched_goals: [GOAL-008]
---

# PLAN-005：有限纹理模型与UV资源闭包

## 0. 变更摘要

消费004实际交付的图片/资源准入与存储，支持有限GLB的baseColorTexture、TEXCOORD_0及KHR_texture_transform，闭合3D纹理渲染/编辑保存/离线资源关系。仅引擎与夹具；G10a内容后继由ag-examples独立计划承接。

## 1. 目标

选定Kenney固定来源的platform/coin/flag及受检小型纹理模型通过公开导入、编辑、Play/相机、保存重开与仓外消费。纹理颜色/UV真实可见，GPU深度/阴影接缝保持；任何依赖缺失/损坏/不支持材质必须具名拒绝，不能退回平色声称成功。为006保留原始节点/primitive关联，不以不可逆烘焙把后继节点动画堵死。

非目标：动画播放、完整character动作支持、蒙皮/morph、PBR全材质/IBL/法线贴图/压缩纹理、材质编辑器、网络URL、Gallery目录/ZIP、正式游戏、新平台/生产UI融合。含动画的原始character.glb在本计划仍不宣称支持，不能删动画后当作006已交付。

## 2. 架构方案

受控本地GLB根及PNG相对依赖→先验证完整闭包/预算→不可变root+edge+image身份→保留UV/材质/节点索引→same-device纹理与mesh cache→统一prepare/publish。场景保存不保留用户机器的原导入目录；重开按项目声明的SHA关系消费已保存字节。根GLB原始SHA保持，闭包身份另含依赖关系，不能只以根SHA缓存可变外图。

唯一实现/顺序复审树拟`D:/autostack/.wt/ui-005/auto-ui`，分支`codex/plan-005-dev`；等004**实际合入及交付**后从核实master创建。共享assets/document/storage/AGP改面串行，不与004各写一套资源实现。主检出仅簿记；依赖/target/报告独立、禁止链接、清理先guard。

## 3. 技术栈

既有driver `authoring/{assets,document,storage,preview,session}.rs`、`host/viewport.rs`、AGP `assets/glb.rs`、`render3d/pipeline.rs`与004交付的PNG存储/缓存。AGP MeshData当前仅position/normal/index/material平色，旧loader把node变换烘焙并拒纹理；不能直接让gltf::import_slice自由读取外部文件。

拟新增`experimental/autographics-auto-driver/tests/textured_model_resources.rs`、`experimental/autographics-prototype/tests/textured_models.rs`、driver `tools/check_textured_models.py`与`verify_plan005.py`；为T-03..06待建入口，非已有功能。WGSL/UV属性与材质绑定采用现有wgpu27路径。

## 4. 需求分析与背景调查

初始r1仅规划；2026-10-29用户在本会话授权完成UI-005全链，继承work/review/有界修复/merge/知识沉淀与清理授权。004实际交付e0fce138及身份修复0261ae9均已落地，当前实施base为2b88bd69；唯一树D:/autostack/.wt/ui-005/auto-ui，分支codex/plan-005-dev。用户另明确同意将有限材质域扩展至opaque双面，保留固定源原件，加入背面光照/阴影验证；其余范围/预算不变。原规划素材/知识指纹见[基线](attachments/004/planning/baseline.md)；总设计见[表现与资源](../design/autographics/m2-presentation-and-resources.md)。独立review在实现冻结后同树顺序执行，不并行写实现。

现有权威：[静态GLB规则](../../../auto-lang/docs/specs/auto-graphics/scene-assets.md)、[M2素材研究](../../../auto-lang/docs/specs/auto-graphics/m2-game-sample-research.md)、[游戏资源分母](../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。原始静态profile拒images/textures/URI/animation。研究记录四GLB引用Textures/colormap.png并声明KHR_texture_transform，角色有static/idle/walk/jump且无skin；这些是历史研究结论，本轮未重新读取原件。旧文档所引858原始JSON在当前master及保留autographics-dev路径未找到；同号语言master的858附件是另一条ACE路线，禁止拿它替代。T-01须恢复固定来源字节并重建逐文件身份，不凭历史总字节数通过素材准入。

## 5. 详细设计

### 5.1 前置与有限支持

T-01核004实际delivery/review/冻结门/Spec合同/两锁、SDK闭包与新增PNG/存储真实签名，冻结本计划输入；004只work/review完成或分支存在不够。T-02先写`attachments/005/work/textured-model-contract.md`，用固定源platform/coin/flag+色图走完整导入→保存→重开→真实GPU路径，冻结支持表/预算/缓存身份和后继动画节点关联。失败回new，不降为平色替代。

新版本明确支持glTF2 GLB单内嵌geometry buffer、静态三角形、原有POSITION/NORMAL和受检TEXCOORD_0、opaque单面或双面、baseColorFactor×baseColorTexture。双面材质按front_facing校正背面法线并在颜色与阴影pass一致关闭剔除；旧平色profile仍保持单面限制。图像仅004已交付PNG支持域；可以来自有界GLB bufferView或受控相对PNG依赖，禁止data/network/绝对URI、query/fragment/百分号变体绕根、路径穿越/链接/reparse，不能将glTF URI直接传给通用加载器做IO。所有声明资源含未引用项均先检。只接受KHR_texture_transform作用于本期UV0的有限offset/rotation/scale与合法texCoord；其余required扩展/多UV/复杂材质具名unsupported，不能静默丢弃。重复JSON键、非有限UV/变换、越界accessor/错误image spans需invalid；超额独立resource_limit。

支持glTF nearest/linear、clamp/repeat/mirrored-repeat的有界采样组合；mipmap minFilter按无mipmap下等价基础采样明确归一并记录，实际需要多级才能等价的输入拒绝，不能宣称完整glTF采样器。颜色纹理按sRGB解码/线性乘色，原alpha必须opaque，原有透明sprite规则不推广到3D半透明。材质/UV正确性用非均匀色块/旋转偏移可判别夹具，而非单色colormap截图。

### 5.2 闭包/身份/兼容

原始GLB rootSHA/长度保留；新closure标识由root身份、有序且唯一的URI或bufferView边、实际图片身份和采样/材质声明派生。相同根字节搭不同外图不会共享错误纹理；不同模型可共享同图，任一闭包快照不可变。同一root若有两个合法依赖环境，可由不同closure ID并存；不改写原GLB URI或伪造原始SHA。导入时有界读取依赖，之后SaveAs用已接受闭包，原导入文件变更不污染快照。

规范项目路径为SHA blob加显式映射，不能依赖Textures目录在生产者路径仍存在。项目锁/无覆盖/entry最后提交/读回确认/残留报告/修复再打开延续004；依赖表本身含entry schema哈希或等价不可变绑定，不能只验证blob独立存在却丢绑定。老scene/平色GLB与004图片表示照常读取，旧schema不误认新closure。

### 5.3 预算/呈现/寿命

旧运行与源码包络同时保持：32credits/callback、1000steps/slice、100000steps/callback、120resumes/250ms、30s初始化/2s tick/8ms目标、用户源256KiB、受管源96KiB/组合352KiB。本计划不因纹理解码在初始化/prepare路径而豁免既有时限或后台无界重试。

004既有上限不再扩大：资源原始身份合计<=8/32MiB（每个root GLB与独立PNG分别计，内嵌PNG其raw已包含root不能重复计raw但解码/GPU照计）；单raw<=8MiB、GLB JSON<=1MiB、nodes<=128、expanded primitives<=16、vertices/primitive<=65536、vertices/asset<=262144、indices<=786432、potential mesh instances<=512。CPU retained总额<=64MiB/GPU总额<=64MiB必须新增UV、材质表、节点元数据、像素、纹理对齐/缓冲；预测在分配前，实际分配核对。隐藏资源、history/pending/current去重union仍付费，不能挪到未计账cache。

纹理/mesh在同device代次共享，发布前先验证closure、建候选资源；OOM/坏依赖/上传失败保持last-good整帧。warm相机/运动不重解码或重传静态mesh/texture；Stop保留合法Editor owner，最终关闭/外部pin释放后实际weak owner失效。阴影pass使用相同几何，UV不改变碰撞AABB。当前scope不转变自动物理或动画规则。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（auto-lang知识仓） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/textured-model-resources.md | 仅平色→有限色图/UV/采样/扩展、完整依赖身份 | M2选中3D素材 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/scene-assets.md | 新版本闭包/格式支持与旧静态profile分列，共享原额/事务 | 不静默放宽旧格式 | AC-01/02/04 |

## 6. 测试设计

T-03..06建立新入口：`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test textured_model_resources -- --test-threads=1`；AGP同式`--test textured_models`；`python experimental/autographics-auto-driver/tools/check_textured_models.py --controls`；`python experimental/autographics-auto-driver/tools/verify_plan005.py --controls`。本轮未运行；work冻结脚本真实参数，0selected/旧runner/部分证据拒绝。只测本计划受影响面，旧profiles/纯色按可复现约1/10受影响代表集；新增AC直接正反例不能抽掉。OS抢焦点自动化暂停，无窗口GPU单实例；全AG里程碑pending，根crate零改不跑根t/tf/tv/tu/docs_gen。

| V/补充case | AC/owner_stage/启用任务 | 输入→实际路径 | 区分性oracle/关闭条件 | 边界 |
|---|---|---|---|---|
| V-01/S-01 | AC-01 work T-02/03 | 固定原件platform/coin/flag与PNG、同根不同外图、嵌入图；坏URI/扩展/类型/跨度/尺寸bomb | 可接受原件实际准入；非法在IO/decoder/分配前阻断；不同closure内容不串用 | 不允许联网加载，不把动画character当静态正控 |
| V-02/S-02 | AC-02 work T-03 | import→save A→move B→open→render；缺图/错边/SHA/alias/entry中断/恢复 | B无原路径仍真运行；错误具名且不假saved；修复后同入口重新读回成功 | 本地盘，无恶意并发路径替换承诺 |
| V-03/S-03 | AC-03 work T-04 | 非对称UV色图、offset/rotation/scale、采样边界、depth遮挡/光照、双面正背面与阴影 | GPU ROI颜色/翻转/边界与独立CPU采样期望容差匹配；去掉UV/变换负控必红；背面可见且法线/阴影与双面规则一致 | 仅支持表材质，不测完整PBR |
| V-04/S-04 | AC-04 work T-05 | GPU prepare故障、20轮Play/Stop/关闭、两个模型共享图/外部pin | 整帧last-good、warm零静态资源重传；实际弱引用最后drop失效，CPU/GPU账不漏UV/纹理 | 不以引用数归零假定驱动缓存释放 |
| V-05/S-05 | AC-05 work T-06 | fresh SDK consumer A→B、旧静态GLB/004sprite定向、identity controls | 离线编译/实际运行/保存重开；完整current源码/SDK/两锁/runner绑定，源变异敏感 | 不等于G10a单项产品ZIP或最终游戏 |

实现者/模型未指定；同一唯一树顺序独立上下文review全部AC/V/S/SD、warnings/fmt/延期与静默材质替代；review pass显式merge门pending，不能仅复用004绿报告。失败改变代码后复审受影响面，再在新冻结提交跑定向门。

## 7. 验收标准

| ID | owner_stage | 结果与证据 |
|---|---|---|
| AC-01 | work | 固定素材身份重建；有限格式/URI/扩展/预算先检后解码，V-01，原额不抬 |
| AC-02 | work | 完整闭包绑定/版本化scene/编辑与离线搬迁保存、缺损拒绝及恢复，V-02 |
| AC-03 | work | 真3D纹理×材质/UV变换/采样/深度可见、像素敏感，V-03，不回退平色 |
| AC-04 | work | 候选事务/资源总额/owner/共享/warm行为，V-04 |
| AC-05 | work | fresh消费与旧004/静态兼容、当前identity及公开交付说明，V-05；为006保留节点关联 |
| AC-06 | review | 同树独立审查全部AC/V/SD与遗漏，无未处置finding，冻结门pending |
| AC-07 | merge | 冻结提交定向门绿后合auto-ui/master，AG全量里程碑pending |
| AC-08 | merge | SD/ledger与索引、本仓Plan归档/交付SHA、guard clean清树和恢复收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 操作/产物 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 004实际交付、用户启动 | 核004/SDK/两锁/owner；恢复固定素材逐文件清单/许可/来源，建唯一ui-005树 | AC-01/05 | [x] base 2b88bd69、004与修复均已合入；SDK681冷准备通过；固定提交原件/许可/逐文件SHA恢复，见attachments/005/work/fixed-source.md |
| T-02 | work | T-01 | textured-model-contract：支持表/身份/预算/节点关联、原件全链可行性，失败回new | AC-01/03/05 | [ ] |
| T-03 | work | T-02通过 | 无任意IO的GLB/PNG闭包准入、持久化/编辑绑定、V-01/02 | AC-01/02 | [ ] |
| T-04 | work | T-03 | UV/材质/纹理shader与cache接线、V-03深度和变异 | AC-03 | [ ] |
| T-05 | work | T-04 | 候选故障/预算/20轮/owner/共享/静态暖上传V-04 | AC-04 | [ ] |
| T-06 | work | T-05 | fresh consumer/兼容/identity controls、fmt/warnings/提交、SD与006交付说明 | AC-01..05 | [ ] |
| T-07 | review | T-06 | 同树独立review并绑定实际revision；merge门pending | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结提交定向门/有效复用，通过才合master | AC-07 | [ ] |
| T-09 | merge | T-08 | Specs/ledger/索引与本地归档、delivery SHA/闭包/节点表示接口 | AC-08 | [ ] |
| T-10 | merge | T-09 | wt-guard clean后清唯一树/分支/空组，保必要恢复收据 | AC-08 | [ ] |

## 9. 复审记录

2026-10-29 bounded new r2：固定提交3fa8a04b1c01ab23db43123d4ce814a34c3fc7f0原件platform/coin/flag均声明doubleSided:true，与r1单面域矛盾；用户明确选择“扩展支持双面材质（推荐，保留原件）”。仅扩大本期opaque材质面的支持并新增V-03背面光照/阴影oracle；保留所有AC、预算、原件身份、旧profile与006边界。全链授权继续有效，next: work T-01/T-02。

2026-10-29 执行授权与入口核对：用户在本会话明确授权“完成计划UI-005”，覆盖work→独立review→有界修复→merge/Spec/ledger/归档/guard清理；003最终批回归按用户要求留给BLOG-004会话，本计划不接管。原new收据的004前置现已解除：交付e0fce138及身份修复0261ae9均为当前master 2b88bd69祖先，原路径verify_source.py exit 0（1431 hashes、SDK681、479 packages、29 path edges）。主检出仅有其它会话docs WIP，零生产代码WIP；保留原样。唯一实施树已创建：D:/autostack/.wt/ui-005/auto-ui，codex/plan-005-dev，base=2b88bd69b2a48172bf761615cc4e78a1df563b62；根/分支断言通过，无AGENTS.md。进入T-01恢复固定源与T-02可行性合同。

2026-10-29 new r1：`stage: new | plan_id: auto-ui/PLAN-005 | plan_revision: 1 | outcome: blocked | next: 等004实际delivery后work T-01/T-02`。前置阻塞是执行顺序，不影响本合同可评审；0/10，未建树/实现/跑测试。源858附件缺失显式认领于T-01，不能以同号ACE材料替代。

2026-10-29 合同预检：三份Plan的0..10节、各10任务/8AC/5V、work/review/merge职责、依赖与SD引用检查通过；本批49个局部链接可解析，新增入口均标待建。此为文档预检，非实现测试或独立final review。

2026-10-29 goal round 1：`stage: goal | plan_id: PLAN-005 | plan_revision: r2 | round: 1 | from: work/new | to: work | status: executing | outcome: pass (T-01 only; work incomplete) | worktree: D:/autostack/.wt/ui-005/auto-ui | commit: 2b88bd69 (implementation base), 2c5e4cb (authorized contract/source receipt) | evidence: fixed-source.md、fixed-source-manifest.json SHA 74a88dbc…、prepare_sdk --from-snapshot exit0 | next: work T-02`。T-01已完成1/10；r2支持域及T-02待实现执行合同已记录于attachments/005/work/textured-model-contract.md；未改产品代码，未执行GPU/功能门，不声称work完成。独立review/merge/归档/清理均待后续。

## 10. 待澄清事项

004实际交付与固定来源恢复已由T-01核实，不再阻塞；原件双面材质支持已获用户授权并纳入r2。当前无待用户决策；T-02继续验证原件全链与预算，不把草拟合同当通过。超出支持域/已有预算不得静默接受。Gallery内容/完整character动画由后继独立Plan负责，未在此承诺已可用。
