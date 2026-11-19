---
plan_id: PLAN-099
status: drafting
feature_name: AGE M2图块地图与分块图层AV扩展
author: [Codex]
created_at: 2026-11-17
updated_at: 2026-11-17
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-058, auto-lang/PLAN-985, auto-lang/PLAN-981]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/tilemap-chunks-and-layers.md]
touched_goals: [GOAL-008]
---

# PLAN-099：AGE M2图块地图与分块图层AV扩展

## 0. 变更摘要

2026-11-17首版执行合同，drafting/0完成。仅规划；没有创建实现树/代码/测试/素材/构建/窗口、运行网络模型、派agent、合入或签章。首个64×64格、2层、16×16格分块地图，镜头跨块可见集/遮挡序正确；图块不是逐格model实例，移动/query仍沿058，同Tick提交视觉/碰撞投影。

## 1. 目标

首个64×64格、2层、16×16格分块地图，镜头跨块可见集/遮挡序正确；图块不是逐格model实例，移动/query仍沿058，同Tick提交视觉/碰撞投影。

非目标：无限地图、联网世界、完整地图编辑器、刚体物理/导航网格；不改变058/旧M1的128行/8状态/32credits，也不把这些数当新地图永久语言上限。

## 2. 架构方案

普通Auto Node源/资源→共同Schema/CST原span→共享类型/唯一verify→实际AV→同一Rust/FFI中立宿主→真实GPU/产品结果。Forge的编辑/Agent仅是该链的消费者，不复制语言或引擎。已有手工原型/旧SDK只作按身份限定的参考，不冒新链。

## 3. 技术栈

复用现ACE/AutoUI/Iced/wgpu与owner锁文件；MeshStore/SpriteStore/资源及既有driver，不指定未经核的最新版本。不新增平行renderer/parser/测试runner；实际依赖版本由T-01基于正式源码/SDK身份冻结。所有拟新模块/目标由work任务建立，本次均未创建。

## 4. 需求分析与背景调查

授权：用户本轮要求统一AGE M2三线strategy/设计、细化引擎首批后扩展、新版AutoForge架构和首批执行计划；复用研究/计划、不重复立项，**仅规划、不启动实施**。本包只承接下列现有owner未覆盖的增量；旧归档计划与M1签章不重开，现有在途进度/暂停不由本合同恢复。

取材基线：auto-lang `c2c77d80e355622acab97791726218a647c4746f`；auto-ui `2f42d6613c6b1d0f24f7bd5679979ae42601d0aa`；age-examples `ea0fc594b2c85620c75b570cda370c31e18b10f0`；auto-forge `e4feb73da57025584763cd9ed36c4caeab98d88c`。取材只证明文件/接口当前存在，不是已跑新能力。已读topdown2d-feasibility、node-editor-projects与node3d-directional-light-and-shadow现规范；本包新增支持不是已交声明。

复用/去重：UI055/AG022已交容量与255/256/257计数证据、UI058的移动/query/sprite同Tick端口；UI022既有图集/动画；AG029两田玩法不重写。

实际来源路径：`experimental/autographics-prototype/src/render2d/mod.rs`；`experimental/autographics-prototype/src/renderer/sprite.rs`；`experimental/autographics-auto-driver/src/age_session.rs`。本包不信任旧snapshot自报PASS；T-01核source/hash/Schema/SDK/平台与原case，实际漂移后只失效相关证据。

## 5. 详细设计

拟新增`TileMapAsset/ChunkKey/TileLayer/MapBudget`（扩展现sprite/asset owner，不造场景树）：格坐标为有符号世界格，16×16分块，camera→可见块→SpriteStore instanced quads；非阻挡地面与碰撞位独立。空间块可见集按(cx,cy)去重计4，ChunkKey仍包含layer；首包map=64×64×2，合法cell索引0..255，最多4个resident空间块（每块两层，即最多8个layer-chunk）/2048可见quad、8张≤256² RGBA8图集、16MiB本包GPU owned buffer预算；地图元数据≤4MiB。超域具名拒/保旧，不扩大旧profile。T-02最低负载不成立则needs_replan，不能删层/隔帧更新/偷用逐格VM。
`ChunkKey=(map_sha,layer,cx,cy,generation)`；异步候选块全准入后按世界Tick发布可见集，迟到旧代回调零上传。query坐标由相同map transform派生，不能从纹理像素猜碰撞。相机x跨15→16时可见集变化、非阻挡底层不挡人物，上层按显式layer/order排；同层稳定次序固定(row,column,id)。255/256/257 cell数组完整长度由ACE表示读回，不回旧SDK截断编码。
拟议作者面（不是现有API）：`use resources: tiles`；`node MapDemo { model { var camera_x int = 0 } view-2d { tilemap ground { asset: tiles, layer: 0, camera_x: .camera_x } } }`。实际分行/分隔符沿共同AGE Schema，T-01在原字段表冻结；`resources.at`的`map2d.load(...) -> TileMapAsset`签名注册由981 LANGUAGE/SESSION现owner完成，UI只供typed Rust服务自动binding/shim，不扩旧parser。

### 阶段、最小前置与交付责任

新产品实现入口=M2A-08正式收据＋本包实际最小981/980 AV typed端口/Schema/资源支持；不等整个981或980 UI-W1全包。中立源码/研究复用不授权现在运行。T-01把所需Float/tuple/opaque资源/输入/场景字段逐项交给981现LANGUAGE/SESSION任务补接口，不让981等本包最终产品完成。旧parser/VM、手写binding、Rust代业务不得冒ACE作者支持。
本Plan交**中立公共实现＋AV作者产品切片**，本包完整AV验收通过后即可独立review/落主/归档清树；不每次开发构建A2R。其同源AC证据由1068/T-04＋981 FUSION/LANGUAGE的所用AC域格在UI-W2-AC后接；A2R由981 RUST/FUSION＋980 B-RUST在UI-W3-A2R后接，生成物真实rustc，缺口仍回实际能力owner。AG031扩展行最终收三端，不能把本包归档冒三端已交。UI-W1/2/3既有全UI分母**不反等新增扩展包**，否则形成循环。
所有新API/文件/命令为拟定，T-02/03使其存在后才运行。shared Rust渲染算法是engine实现，作者玩法/状态仍普通Auto，不能在host藏业务。保原M1 profile/Schema/hash/预算，新增支持用版本化扩展cap与独立metadata。

frontmatter为最终owner并集，不等整包依赖全部完成才做局部任务；**gate只认已审正式phase来源，不认WIP/分支reviewed**。最小缺口逐字段/操作交既有owner，必须保原合法输入/独立oracle，不许只为绕依赖做窄假样本。新Plan不反向成为985/A或UI-W1/2/3基础收据前置。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-MAP-01 | add（拟议） | `docs/specs/auto-graphics/tilemap-chunks-and-layers.md`（新目标，当前无此能力Spec） | 交付时只沉淀本包实证支持域/接口/错误/资源/阶段与未交项，旧规范/源码锁保持 | 本包增量不冒全引擎/三端/旧产品 | AC-01..06 |
| SD-MAP-02 | modify（交接，原owner后续沉淀） | `age-examples/docs/specs/auto-graphics/age-m2-acceptance.md`（AG031既定目标；本包只交来源，不跨仓写未来Spec） | 来源/原例/实际consumer/oracle/后端/平台逐项，未做端NOT_RUN/owner可追 | 防止AV阶段冒三端或产品总完成 | AC-03/06/07 |

## 6. 测试设计

以下均为拟议，T-02/03建立实际入口、永久source/expected及负控后才执行。本包新增target `experimental/autographics-prototype/tests/plan099_map.rs`（新路径），拟命令`cargo test --manifest-path experimental/autographics-prototype/Cargo.toml --test plan099_map -- --test-threads=1`。另需driver作者链命名测试（同号测试文件，由T-04在原driver tests建立），GPU/窗口单实例，只跑新增/受影响完整职责文件与必要消费者；零根语言改动不跑语言全仓。共享ACE变化明确选`cargo t ace-source ace-syntax ace-av`及真实触面组，不把正式ace-perf加每次收尾，不反复跑全AG套件。原Windows/Linux/ignored/环境条件保留，未运行不充PASS。

| Case / work任务 | 输入/状态/真实入口 | 独立oracle与拒绝/恢复边界 |
|---|---|---|
| V-MAP-CPU / T-03..T-06 | 格(-1,0)/(15,0)/(16,0)与独立floor-div/可见块集合；长度255/256/257，读回首尾值各不同 | 无截断/错负坐标，数据与画面同Tick |
| V-MAP-DRAW / T-03..T-06 | 两层重叠色块、相机跨块、图集第0/255格、不可见块移出 | 独立ROI/次序、可见集、上传计数；不画错误格或全图常驻 |
| V-MAP-QUERY / T-03..T-06 | 沿058一次移动/最近目标/两实例2:1，在格墙与非阻挡地面 | 独立轨迹/命中/精灵位置一致，原058业务/预算不重做 |
| V-MAP-FAIL / T-03..T-06 | 坏atlas索引/截短chunk/第五resident空间块/超字节/旧generation→修复 | 具名拒且旧接受集不变，迟到零效果，合法峰值恢复 |
| V-MAP-LIFE / T-03..T-06 | 一窗20轮Run/Stop/resize/空视口、chunk unload与最后pin | owner/上传/资源数有收支；不能只history.len=0 |
| V-MAP-SOURCE / T-03..T-06 | 同源Node AV真业务；变相机/图层数据、删实际map绑定反控 | CPU/ROI随源变，删除映射须红；交AC/A2R所需精确source/ABI账 |

### 旧成果到新Case的具体映射

| 原已存在路径 / 命名case | 本包消费/新增oracle |
|---|---|
| `experimental/autographics-prototype/tests/sprite_resources.rs::nearest_clamp_flip_alpha_and_warm_reuse_draw_real_pixels`、`frame_uv_and_capacity_are_bounded` | V-MAP-DRAW/FAIL复用纹理/UV/容量的既有判据，新增chunk/layer/camera真ROI与255/256/257数据长度，不改原expected |
| 同文件`identity_device_and_missing_texture_rules_reject_before_drawing` | V-MAP-FAIL/LIFE增加map generation与可见集事务，仍保device/缺纹理准入 |
| UI058 `V-S01..08`、AG023 `V-S01..07`（待实施，非已交测试） | 只消费其正式AV对应轨迹/query/sprite同Tick来源；V-MAP-QUERY验证地图投影，不重复实现移动/玩法 |

原用例映射：不是复制旧expected造新oracle。首包用源路径+hash+原Case→新Case→受检producer→真实consumer→独立期望登记；原控件/游戏/资源/错误/生命周期与条件不删。实现者自检、同树停写后独立复审；本轮未指定模型/独立agent，不自动创建会话。普通新例以确定性小输入区分错误实现；资源/大规模或真实模型需要的门另具名计量，不偷偷缩样本/超时当成功。工具/模型/平台缺失必须NOT_RUN/BLOCKED，最终required没跑不能交付。

## 7. 验收标准

| AC / owner_stage | 关闭条件 | 证据/任务 |
|---|---|---|
| AC-01 / work | 实际普通源/当前范围/最小前置/原用例与各owner冻结，跨阶段不循环等待 | T-01/02，原源/Schema/SDK/接口与映射 |
| AC-02 / work | §5真实支持/typed语义/合法输入完成，未支持不静默丢失/回退 | T-03/04，V全部合法正例 |
| AC-03 / work | §6区分性错误/资源/取消/关闭/恢复与同源真实consumer通过；peer equality不代独立oracle | T-04..06，V全部反控/恢复 |
| AC-04 / work | 原规范/旧场景/业务/权限/预算不被破坏，未来端/平台有准确未交账 | T-05/06，旧受影响正反例及新合法邻例 |
| AC-05 / work | 冻结并集/零新增未处置warning/格式/无debug残留，命令/执行数/排除/计时/source身份真实 | 最后work任务；无全仓盲测 |
| AC-06 / review | 逐AC/Case/原义务与Spec delta独立重建；遗漏/延期/绕行和来源错配显式扫描，真实修复回work | 独立review任务、冻结SHA |
| AC-07 / merge | 审定来源正式合入/Spec只落已交范围/计划归档或正确phase fold，最终交付者可消费 | merge任务，不把实现正确性留merge |
| AC-08 / merge | 自有输出逐路径对账、证据摘要折Plan、材料与树guard后清理，未知scope不盲删 | 最后merge收尾，实际清理后才勾 |

## 8. 执行步骤

| T / owner_stage | 前置 | 动作/交付 | AC | 状态 |
|---|---|---|---|---|
| T-01 / work | 明确后续实施指令；只读可准备 | 核正式M2A-08/最小AV Schema/host与本包上游、代码/原case/支持域/资源版本；缺字段具名回981，登记唯一树/材料。 | AC-01 | [ ] |
| T-02 / work | T-01实际接口齐 | 建立真实最长链最低负载/合法与非法近邻、独立CPU oracle及新测试目标；所定样本不可行先给反证needs_replan，不重做已交可行性研究。 | AC-01/02 | [ ] |
| T-03 / work | T-02 go/已冻结机制 | 在§4原模块实现§5公共增量、参数/格式/预算/owner准入；建永久fixtures，保持旧profile。 | AC-02/04 | [ ] |
| T-04 / work | T-03及981所用AV端口正式齐 | 接普通Node Schema/自动binding/AV真实作者链及driver同号测试；创建运行入口/负控，真实改变源应改变结果，不写Rust业务。 | AC-02/03 | [ ] |
| T-05 / work | T-03/04 | 运行本包V全部合法/错误/资源/恢复/兼容与真实GPU/作者consumer，准确记录命令/数值/源/工具/排除/用时。 | AC-03/04 | [ ] |
| T-06 / work | T-05 | 冻结cap版本/源资源/ABI/原例映射/独立oracle、AV正式交接与AC/A2R待交字段给981/1068/AG031，不为每端另立Plan。 | AC-01/03/04 | [ ] |
| T-07 / work | T-03..06完成 | 核本次全部受影响文件/必要consumer的冻结并集、health/格式/警告与SD候选；缺required不execution_done。 | AC-05 | [ ] |
| T-08 / review | work全齐/停写冻结SHA | 同一唯一树独立重建全部AC/Case/资源/旧例/Spec/后延；needs_fix回work，phase总绿不替本包。 | AC-06 | [ ] |
| T-09 / merge | T-08 pass | 核正式来源/冻结门、沉淀已交AV与中立Spec、合入/归档，交AV能力包；AC/A2R仍具名待交，不签总三端。 | AC-07 | [ ] |
| T-10 / merge | T-09 | 全输出/缓存/测试目录对账，摘要折Plan，清自有scope/树/分支（fresh wt-guard clean），更新登记；不删其他repo同号scope。 | AC-08 | [ ] |

唯一实施树拟`D:/autostack/.wt/ui-099/auto-ui`，分支`codex/plan-ui-099-dev`，未创建。跨仓同组兄弟/正式SDK按owner现规约解析，禁止junction/symlink；材料唯一自有scope拟`D:/autostack/plan-artifacts/plan-099/auto-ui/`，不混同其他仓同号scope，merge只清本owner目录。当前纯文档材料在系统Temp，不虚构在途实施登记。

## 9. 复审记录

规划预检：编号/目录/元数据/0–10节/AC与Case映射/旧成果具体复用/Spec owner/阶段依赖和新增本地链接已静态核对；8新Plan共74任务，阶段DAG无环。四仓`git diff --check`文档检查通过；没有运行拟议测试命令，不能用本条替代未来独立产品复审。

2026-11-17 stage:new / PLAN-099 r1 / outcome:pass（可审阅的规划合同，不是实现/独立产品review PASS） / 当前0/10、drafting。source路径/原owner/最小接口/数据与类型/区分case/AC/任务/SD和阶段边界已列；所有拟新命令/fixture/API/模型验证未执行。next:未来明确实施指令且所用正式输入齐时按T-01执行；本轮不自动work、不派agent、不建树或目标。

## 10. 待澄清事项

不再把本包增量统称“待后继立项”；责任已由本合同明确。T-01只核实际源漂移/具体适配/版本与等价实现，T-02所需机制不能支撑最低合法样本时给真实反证needs_replan，不缩源/oracle或偷偷回旧引擎。共享语言/ABI公共支持改变须沿原owner设计/版本交接。规划不是实施授权；缺正式接口/工具/平台/真实模型时具名阻塞，不要求本轮用户重复确认已定产品方向。
