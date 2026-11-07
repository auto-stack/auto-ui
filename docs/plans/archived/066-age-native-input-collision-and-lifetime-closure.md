---
plan_id: PLAN-066
status: archived
feature_name: AGE原生输入碰撞与运行寿命收口
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 9
total_steps: 9
depends_on: [auto-ui/PLAN-054, auto-ui/PLAN-064]
supersedes_spec_components: [docs/specs/auto-graphics/node-mixed-regions-lifecycle.md, docs/specs/auto-graphics/node3d-character-and-runtime-camera.md]
new_spec_components: []
touched_goals: [GOAL-008]
---

# PLAN-066：AGE原生输入碰撞与运行寿命收口

## 0. 变更摘要

关闭引擎侧原生输入证据差额及064具名运行边界；对同进程GPU退出AV作有界归因并修复支持域内缺陷。AG025负责真实内容同版验收。

## 1. 目标

冻结现有支持域，证明或修复真实键盘/焦点/坐标与Run资源寿命、hidden solid/parked分离。非目标：扩多窗/任意状态迁移/全物理/提高容量、改变A2实现为未证历史方案。

## 2. 架构方案

平台窗口事件→MixedRegionsHost/CompositionInput→实例事件队列→Session→呈现；character3d_v1服务→host文档→渲染。各职责分层验证，不把posted或离屏输入冒OS证据。

拟唯一树`D:/autostack/.wt/ui-066/auto-ui`，分支`codex/plan-066-dev`；跨仓源码只读取实际交付，若需产品修复超出本仓则交原owner另约，不直接改其主检出。

## 3. 技术栈

已核driver `src/bin/mixed_regions_window.rs`、`src/node_mixed_regions_host.rs`、`src/lib.rs::Session`、`api/character3d_v1.at`；tests `mixed_regions_lifecycle_flow.rs`、`node3d_runtime_flow.rs`、`node3d_model_objects_flow.rs`。

## 4. 需求分析与背景调查

用户2026-11-06明确授权“AGE M1剩余收口工作和ACE迁移整链先证正式拆成计划，明确依赖、验收和执行顺序，先规划，不实施”。本合同只规划；没有执行授权、测试结果或新能力交付认领。主检出观察`auto-ui@02316271da9b9cd90750ca5a38ed91143f6c4be4`；路线输入为auto-ui `docs/design/autographics/age-three-milestones-and-ace-m2.md`磁盘稿，非已提交实现。新版M1沿原M2-01..10/G-M01..09全部义务，历史tag/号/归档原文保留。

054 Spec只证明posted键盘和scale=2.0；064残余含父隐藏solid、内部trap恢复、Session跨文档及127/128注册。Spec明确128场景行≠128注册solid：127注册INIT受原步门拒绝。AG021多host AV以每例新进程隔离，不能消除同进程风险。


只读输入指纹（SHA256，磁盘实读；工作前仍须核正式来源）：

- `auto-ui/experimental/autographics-auto-driver/api/character3d_v1.at`：`5e24af1c6422781043625d9412abb15b01124557a2e6f4ff3d28b25e27f389e7`。
- `auto-ui/experimental/autographics-auto-driver/src/node_mixed_regions_host.rs`：`0b4304534d14097cfe3040f7f34b28e041ae17793bf0301ca5f796c3affba0b4`。

2026-11-07规划复核：用户授权复核并直接完善这五份合同，仍先不实施。以下r2替代r1当前合同；§9旧记录保留为历史。状态保持drafting/current_step=0，不把本轮静态检查算作产品验证或独立final。

## 5. 详细设计

T02先核支持/所有权与有限复现配方：真实窗口Tab/Enter与鼠标捕获/移出/blur、两种真实OS scale和resize后坐标；若无第二DPI环境，提供可执行手工门但不计pass。允许修复本仓平台路由/焦点/scale更新，不修内容业务。引擎fixture原生绿不替AG025实际showcase。

碰撞：hidden parent+registered solid child阻挡；不可见parked模板不阻挡；可见非solid沿现行043语义，不能顺手改合同。128稀疏场景原例合法；127/128注册分别观察准入，资源拒绝发生前后不得发布部分新状态，不要求提升注册容量。trap/cancel覆盖临时effective修改的失败出口；若Session故障销毁且下Run新建需实证旧状态不可见，不能只援引下一Tick自愈。跨文档Session复用若不可达以调用图+公开入口反控证明；若可达则清注册或受控拒绝并验证无越界。

去重边界：064已交碰撞集合与可见性分离、G3 11项/g14控制；`node3d_runtime_flow.rs`已有128稀疏行合法与127注册INIT拒绝，`node3d_model_objects_flow.rs::hidden_templates_and_live_instances_are_non_solid_on_the_actor_path`已有parked不阻挡。复用其原oracle，新增只覆盖hidden-parent、失败恢复、跨Session和注册边界的缺口；不得把旧绿项包装成新功能或扩大容量。

GPU边界：一个进程顺序创建/使用/退出8个host与同host20轮Run/Stop均记录CPU/GPU owner、最后引用、错误与高水位；最多两种隔离机制（环境/驱动与代码owner），原失败样本保留。无法区分或真实合法域仍AV则不得execution_done；不能仅改为每例进程隔离或重试绿。修复仅限本域；需底层架构扩围则needs_replan。

本Plan不依赖065整包，可用已有3槽fixture准备；共同`lower_node_ui`/window/host/生成夹具写面逐符号稳定SHA短窗。最终AG025候选必须包含065和本Plan两个交付，不能拿两个不同引擎分别绿拼同版。

未来命令：driver `cargo test --test node3d_runtime_flow --test node3d_model_objects_flow --test mixed_regions_lifecycle_flow -- --test-threads=1`；新命名边界用例由T03/04建立；原生使用交付`mixed_regions_window`与T02手工操作脚本，逐项记录实际入口与OS通道；probe只作对照。

寿命判据由T02按实际所有者冻结：正常运行中允许host/共享资源存活；Stop与最终drop分别核对应层级的引用和注册收支。记录冷启动/预热后CPU与GPU内存、高水位及缓存上限出处，不要求驱动进程内存逐轮严格回到启动值，也不接受只看RSS平稳冒资源已释放。无既有缓存界限时先给有依据的界限与区分泄漏的反控，不任意抬原预算（AC-03/V-04）。

跨仓回交：AG025拥有真实showcase与G04编辑OS20/34取证，066拥有上述MixedRegions/character/Session公共面。编辑器独有选择/拖动/保存缺陷不自动塞入066；由AG025登记最小反例、实际auto-ui文件/owner、关闭oracle及所阻原AC，交现有匹配活动合同，确无匹配才另立有限修复。066未归档且属本合同的缺陷回同树修，已归档不得重开；修复交付后AG025更新同版候选并只重验失效面。066不等AG025验收才交付，避免下游反向硬依赖。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/node-mixed-regions-lifecycle.md | 现有知识/缺口→本合同已验证支持域、来源及未证边界 | 逐AC真实沉淀，不把计划写成支持 | AC-01..AC-04 |
| SD-02 | modify | docs/specs/auto-graphics/node3d-character-and-runtime-camera.md | 现有知识/缺口→本合同已验证支持域、来源及未证边界 | 逐AC真实沉淀，不把计划写成支持 | AC-01..AC-04 |

## 6. 测试设计

以下均为未来work门，本轮未执行。实施者自检后交独立上下文final，原始工件与源/输入/配置绑定；缺必需oracle不能execution_done。每个V由对应任务建立可执行入口，不能用拟建脚本名充已运行。

| V | AC | work任务/入口 | 输入与区分性oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 | T02/03；真实window事件 | Tab选中真实按钮Enter仅一次事件；点击外侧/blur不泄漏；DPI真变后命中/剪裁一致 | posted是负对照，不代OS |
| V-02 | AC-02 | T04；node3d runtime/model flow | 父隐藏solid阻挡、parked不挡、128稀疏合法/超容量拒绝，独立轨迹与未误写断言 | 行数与注册数分开 |
| V-03 | AC-02 | T04；Session/host失效入口 | move失败/cancel后恢复或域销毁；新Run初态，无旧注册写入其他文档 | 注入必须到实际失败出口 |
| V-04 | AC-03 | T05；新同进程host寿命case | 8次host/20轮RunStop无AV，Run级owner归零、host级owner在最后host释放后归零；共享缓存有冻结界限且不随轮次持续累积，坏资源修复下次Run正常 | 重试与换进程不算关闭 |
| V-05 | AC-04 | T06；公开消费/定向旧门 | G3 11项/043 g14原oracle及混合控制保持，修复前反例能区分 | 不扩大全AG |

实施、构建、测试仅在唯一专用树；主检出仅Plan簿记。独立上下文final在同树顺序进行，review期间停实现写入，finding返work后由同员定向复核；自检不代final。本轮不创建agent或实现树。GPU/音频/焦点/重门机器单实例；原生自动化须沿实际授权，不能以未来work授权推断全局OS自动化暂停已解除，手工配方可用，缺设备/授权保持该门pending。只跑真实触面命名case/必要消费者，不每次merge跑全量AGE；里程碑必要并集单独一次窗口。首红、环境重放绿和未执行分别保留。

材料唯一`D:/autostack/plan-artifacts/plan-066/auto-ui/`，work开工核同号owner、登记用途/路径/在途；所有探针、SDK副本、日志、外部B/安装根、外置target/包/截图均在此scope。永久回归夹具可入产品树，临时探针不可散落仓内。merge把命令/退出码/关键数字/源hash/失败与关闭摘要折入Plan，对账全部实际输出（含外盘）后删除自有scope及树内构建物；仅具名owner+理由+关闭条件hold保留。不得删除整个同号父scope波及其他仓。禁止junction/symlink；清树前fresh `bash D:/autostack/wt-guard.sh <本树>`须clean。未知归属fail-closed；不写attachments/legacy specs.json/派生索引，不收编共享导航他人WIP。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 现有原生输入/焦点/DPI义务有实证，公开支持域内缺陷已修 | V-01 |
| AC-02 | work | hidden solid/parked及容量/失败/跨Run注册边界正确且原额度保持 | V-02/03 |
| AC-03 | work | 同进程合法寿命无未处置AV，owner收支/恢复齐 | V-04 |
| AC-04 | work | 定向兼容、明确支持/非范围和准确交付配方齐 | V-05 |
| AC-05 | review | 同树独立final逐AC/SD/源/实际反控复核pass，遗漏延期workaround与告警审查齐，finding关闭；绑定r2/准确提交 | T-07 |
| AC-06 | merge | 冻结实际触面门/源身份后落主，canonical与同号归档真实落位，正确表达能力与限制 | T-08 |
| AC-07 | merge | 全输出对账、摘要折Plan、fresh guard清自有树/分支/scope并刷新具名hold登记 | T-09 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 另行启动 | 重核064/054差额、真实支持与owner，唯一树scope | AC-01..03 | [x] 2026-11-07：唯一树`D:/autostack/.wt/ui-066/auto-ui`分支`codex/plan-066-dev` base `3d64877d`；材料scope`D:/autostack/plan-artifacts/plan-066/auto-ui/`（OWNER.md登记在途）。§4输入指纹实读同值：`character3d_v1.at` 5e24af1c…、`node_mixed_regions_host.rs` 0b430453…。重核064 §10四项残余（父隐藏solid、slide内trap恢复、`_ag_char3_solids`跨Run、127/128边界）与054原生门（posted≠native）为本轮T-03/04/05目标；driver `.sdk-source`=主检出生成副本681文件（`verify_source.py --verify-only`因pre-existing 013源集漂移失败，主检出同红，具名见§9）；主检出未改。 |
| T-02 | work | T01 | 冻结原生操作与DPI/资源边界先证、分层owner释放及缓存界限，最多两机制；必要公共写面短窗 | AC-01..03 | [x] 2026-11-07：冻结两条通道=自动链posted注入（`mixed_regions_window --probe`）＋真实OS手工记录器（`--manual`，auto_pass:false）；未引入第三机制。分层owner：Run级surface target（4 slots/7,584,000B，Stop回基线）与host级store（7 meshes/136,952B、4 surfaces，运行期允许存活、host drop后随host释放）；20轮/8host基线逐个相同=`Host::with_project`固定surface集＋暖上传一次的界限，非RSS推断。DPI先证：本机唯一1920×1280虚拟显示、单一scale→第二scale不可达，手工门pending（D01）。 |
| T-03 | work | T02 | 核验并修复真实输入/捕获/焦点/DPI/resize链，永久反例 | AC-01 | [x] 2026-11-07 commit `2c77aa80`：真实窗层此前未接`Focused(false)`/`CursorLeft`→宿主的失焦/离窗入口，已接（取消在途捕获并清焦点）；键盘焦点改为`Tab`取得、`Enter`仅其后激活、失焦清除（`NodeUiApp::focus_active`），永久回归`flow_window_blur_clears_keyboard_focus_and_outside_clicks_do_not_leak`。`--probe`新增resize后命中（posted物理坐标=逻辑×scale）：count=201→202、p1=24、代次3→7、scale 2.0、exit 0。真native SendInput/焦点捕获/DPI真切换=具名手工门（`--manual`），不计pass。**复核F-066-R2-01修复（`5ee7101c`）**：0×0/亚像素逻辑尺寸由宿主`resize`具名拒绝且不改上一good尺寸/布局，窗口层忽略0面积`Resized`，永久回归`flow_degenerate_resize_is_refused_and_the_layout_recovers`；探针改状态门（p1 epoch未挂载+strip移动后才点、逻辑尺寸应用后才post-resize命中）+自适应Tab作业（真实`Focused(false)`后重取焦点、超时具名失败）；同命令3连跑均exit0且count=202/post_resize_hit/idle_no_tick/epochs 3→7/scale 2.0（`t03-fix-native-r2/run{1,2,3}`）。 |
| T-04 | work | T02；写面短窗 | 碰撞/预算/trap/Session复用边界真实执行并修复支持域缺陷 | AC-02 | [x] 2026-11-07 commit `2c77aa80`：`tests/character3d_api.rs`新增3例（21/21绿）——隐藏父group+注册solid子行仍阻挡（未注册对照放行）；一tick第二次move在slide内触发fault 10后`effective[]`已恢复（不靠tick末derive）；127/128注册solid在INIT被原每回调累计步门拒绝、128容量拒绝被遮蔽、8注册正常。跨文档Session复用=不可达（`Play::new`每Run新建Session，play.rs 663-669；`_ag_reset`仅Run起始一次），旧注册不能写入其他文档；043可见行语义/容量/产物未改。 |
| T-05 | work | T02；GPU独占 | 同进程退出AV归因、寿命收支及坏件恢复；不能只留未知且称已闭 | AC-03 | [x] 2026-11-07 commit `2c77aa80`：新`tests/host_lifetime_flow.rs`（driver `--test-threads=1`）同进程顺序创建/使用/drop 8个Host×20轮Start/Stop＋tail host：每轮Stop后4 slots/7,584,000B回基线，store 7 meshes/136,952B与峰值不增，9个host基线逐个相同，tail仍可Run，进程exit0无AV（两次68.64s/50.20s）。坏件修复再Run复用`node3d_runtime_flow::vs06`（SaveAs/单字节损坏恢复）与`node3d_model_objects_flow::vs05`（坏资源修复）原证据。窗口化宿主退出路径未纳入（离屏Host覆盖device/surface/store）。**复核F-066-R2-04修复（`5ee7101c`）**：9个pre-run基线改为逐一断言相同（`baselines.iter().all(== first)`）、tail host在全部drop后断言同基线；SD改述为“Run不新增/不泄漏target·mesh owner，host level常驻基线，drop后观察面=后续host同基线”。 |
| T-06 | work | T03..05 | 受影响完整命名test target/旧G3与g14控制、SD/残余分类/AG025交付配方 | AC-04 | [x] 2026-11-07：受影响命名并集（首轮59例）全绿；native `--probe` exit0（count=202）；受触6文件`rustfmt --check`过；SD-01/02已按实测改并提交`e95da1db`。G3 11项内容验收=AG025跨仓owner；pilot生成门未跑（pilot `Cargo.lock`需随auto-lang tip更新，`--locked`拒绝，主检出同状；未改生成器/产物，字节未变）。AG025配方：候选必须含065交付`55c20a86`＋本Plan code_commit，禁两引擎分别绿拼同版。**复核F-066-R2-02/03/04（`5ee7101c`）**：口径改六点击+五按键；SD分离`Focused(false)`清焦点与`CursorLeft`仅取消捕获，并新增“pointer leave后Enter仍激活”回归；SD寿命措辞改述为可观察面；union复跑60例全绿（`t06-union-r2.log`，mixed_regions_lifecycle_flow 7/7）。 |
| T-07 | review | T-01..06实质完成 | 独立final；不修实现，finding交work，同员受影响复核 | AC-05 | [x] 2026-11-07：独立final（repair re-review）pass；AC-01..AC-04 通过，F-066-R2-01..04 全闭；真 native 手工门与 G3/pilot/merge 门保持 pending；见§9末条。 |
| T-08 | merge | T-07 pass | 冻结门/身份落主、canonical/导航/同号归档；不自动tag/远端发布 | AC-06 | [x] 2026-11-07：rebase onto master `b95b910d`（range-diff 3/3 `=`：`cf7f3995`=2c77aa80、`de85fd5a`=e95da1db、`5a53b789`=5ee7101c）→ `git merge --ff-only codex/plan-066-dev`，master tip=`5a53b789`（无merge commit）；canonical两Spec在main与worktree实算sha256一致（4753CD65…/6C5B5FD3…，同review收据）；landing smoke on main=character3d_api 21/21、mixed_regions_lifecycle_flow 7/7、native `--probe` exit0 count=202（`landing-native/`）。同号归档本提交（docs/plans/archived/066-…）。未打tag/未push。 |
| T-09 | merge | T-08 | 全输出对账/摘要/guard/清理/登记；实完才勾 | AC-07 | [x] 2026-11-07：`wt-guard.sh`（Git bash）clean → worktree `D:/autostack/.wt/ui-066/auto-ui` 移除（exit0）、branch `codex/plan-066-dev` 删除（was `5a53b789`）、空组 `.wt/ui-066` 移除；`git worktree list`/branch/路径三查无残留。scratch `plan-artifacts/plan-066/`（仅本仓 `auto-ui/`，parent 无别仓）guard clean 后整删=20,138 files / 12,961,678,581 bytes；无具名 hold。registry 记 `auto-ui/PLAN-066` deleted-at-merge。 |

## 9. 复审记录

2026-11-06 / stage: new / auto-ui/PLAN-066:r1 / outcome: pass（规划合同可审阅，非独立final）；next: 用户后续启动后按T-01进入，逐任务依赖见§8。drafting 0/9；未建树/scope、未实现/构建/GPU/OS操作/迁移/发布/创建agent，未改既有合同及历史锁。

2026-11-06 / stage: new / planning_validation: pass（仅文档预检）。本批五合同各0..10节、44个任务全部未勾，AC/V/SD/owner_stage与任务对应；依赖文件可解析且新合同DAG无环，本地号唯一、counter UI068/AG026。新增路线/导航链接实核通过，实际源码/命令选项已静态核；研究运行结果均未执行。输入SHA见§4，未修改既有实施合同/current Spec/产品代码/历史锁，未创建树或材料scope。此记录不是独立review或实现PASS。

2026-11-07 / stage: new / auto-ui/PLAN-066:r2 / outcome: pass（规划合同修订检查，非实现或独立final）。标明064既有128稀疏/127拒绝/parked及G3回归，聚焦真实残余；补分层owner/缓存oracle与编辑缺陷跨仓回交，取消笼统无增长断言。 next: 保持drafting，待用户后续启动按§8进入；所有任务未执行，原任务数不变。本次未改产品/Spec现状、未创建树/运行测试/打tag。

2026-11-07 / planning_validation: pass（纯文档静态检查）。五份r2共44任务、36项AC、24项V，均未勾选；每份0..10节/任务计数/owner_stage及Spec目标核对通过。包含本批及981/959/987前置的58节点可达图（已归档节点作交付边界）无显式循环或缺失引用；本批与路线新增相对链接可解析。AG025修改的age-m1-obligations由硬前置AG024新建，非现存Spec。另人工核审计→最终验收、先证→接口、981→AGE阶段三类隐含等待已解开。源码lower_node_ui/character3d与AG021归档SHA仍与§4相符；本批无实施树。此结果不证明产品行为通过，不免未来独立review。

2026-11-07 / stage: work | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: pass（execution_done，6/9） | worktree_path: `D:/autostack/.wt/ui-066/auto-ui` | branch: `codex/plan-066-dev` | base_commit: `3d64877d` | code_commit: `2c77aa80` | docs_commit: `e95da1db` | dependency_revisions: driver `.sdk-source` = 主检出生成快照副本681文件（`verify_source.py --verify-only` 撞 pre-existing PLAN-013 源集漂移：68 extra files，主检出同样失败，非本Plan引入） | task_ids: T-01..T-06 | acceptance_results:
- AC-01 PASS*（posted真实OS窗口：五点击+五按键、resize后命中count 201→202、p1跨挂卸12→24、代次3→7、scale 2.0、空闲零Tick、exit0；新增真实`Focused(false)`/`CursorLeft`→宿主失焦/离窗入口，键盘焦点改为Tab取得/Enter其后激活/失焦清除，flow回归在案。**真native SendInput/焦点捕获/DPI真切换=具名手工门pending**：`mixed_regions_window --manual`可执行记录器已交（auto_pass:false），本机唯一1920×1280虚拟显示/单一scale，不可自动取证，未宣称已验）。
- AC-02 PASS（`character3d_api` 21/21：hidden-parent registered solid阻挡+未注册对照、失败出口恢复effective、127/128 INIT被原步门拒绝；跨文档Session复用不可达=每Run新Session（play.rs 663-669）+ `_ag_reset`仅起始一次；043可见行/容量/产物未改）。
- AC-03 PASS（`host_lifetime_flow`：同进程8 Host×20轮+tail，Run级target 4/7,584,000B每轮回基线、host级store 7/136,952B峰值不增、9 host基线相同、tail可Run、exit0无AV；坏件修复再Run复用vs06/vs05原证据）。
- AC-04 PASS（59例命名并集全绿：character3d_api 21、node3d_runtime_flow 7、node3d_model_objects_flow 8[g14 hidden-template控制]、mixed_regions_lifecycle_flow 6、storage 3、host_lifetime 1、node_ui_flow 3、node_mixed_views_flow 6、node_ui_few_slot_flow 4；native --probe exit0；受触6文件rustfmt --check过；SD-01/02已改并提交）。
| evidence_scope: 日志/报告 `D:/autostack/plan-artifacts/plan-066/auto-ui/{baseline-3d-flow.log,t04-character3d-api.log,t05-host-lifetime.log,t06-union.log,t03-native-posted.log,t03-native/mixed-regions-native.json,pilot-gates.log}`；外置target同scope（driver target/、pilot-target/） | residuals: ①native手工门pending（上述）；②G3 11项内容验收=AG025 owner（跨仓，不在本链）；③pilot生成门未跑（pilot `Cargo.lock`需随auto-lang tip更新，`--locked`拒绝，主检出同状；未改生成器/冻结产物，字节未变）；④pre-existing reds具名：`verify_plan013.py`源集漂移（68 extra files）与`cargo check --tests`中`authoring_feasibility`缺`hud_overlay`字段，均非本Plan引入、不在受影响面 | pending_closeout_gates: merge冻结门/落主/canonical SD/归档/清理/输出对账 | blockers: none（AC-01 native手工门为合同内具名pending，不阻塞work→review） | next_responsible_role: review（T-07独立final，同树顺序）

stage: goal | plan_id: PLAN-066 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/ui-066/auto-ui` | commit: `2c77aa80` | next: review

2026-11-07 / stage: review | review_scope: final | reviewer_identity/limitations: 独立 final-review 子代理上下文（父会话新上下文启动；同机同 session host；非人工/非独立机器；全部 cargo 串行单实例、driver `--test-threads=1`；native `--probe` 亲跑 3 次=1 绿/2 红；真 native SendInput/焦点捕获/DPI 真切换手工门未跑；未在 main 检出运行、未改实现/测试/Spec；复核时 worktree clean） | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: needs_fix（AC-01 posted native 探针非确定性＋0×0 resize 链；AC-02/03 通过，AC-04 部分） | worktree_path: `D:/autostack/.wt/ui-066/auto-ui` | branch: `codex/plan-066-dev` | reviewed_commit: `e95da1db179cd1d23848bc515b06a7fcee3dd925` | base_commit: `3d64877d3837faff6c1175bdb4fee248c3f02418` | dependency_revisions: driver `.sdk-source` 681 文件实读（未跑 prepare_sdk）；`python tools/verify_plan013.py`=AssertionError: source set drift（pre-existing 013 身份冻结；本 Plan 未改该身份；主检出同红为 §9 work receipt 记录）；外置 `CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-066/auto-ui/target` | spec_inputs: `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md` sha256 `FA8D15ABB2DADEF45D1A8EBA34A65BD9FB63EA5F86399EEBAC88AEE2E359E0A4`; `docs/specs/auto-graphics/node3d-character-and-runtime-camera.md` sha256 `9D235A1A3F2BD7BBA2C10BB48A8694DC7E7A169675BB0064A02BE890EFE788E6`（worktree HEAD 实算） | acceptance_results:
- AC-01 **FAIL**：host 级 `flow_window_blur_clears_keyboard_focus_and_outside_clicks_do_not_leak` 通过（Tab 才取焦点、Enter 前/失焦后 inert、外部点击不派发）；但 AC-01 的 `mixed_regions_window --probe` 在冻结 HEAD 亲跑 3 次仅 1 绿：run3 count=202/post_resize_hit=true/exit0；run1 count=102/post_resize_hit=false/exit1（p1_x=24、epochs 7、无 focus_lost/零 resize，phase6 strip 布局跟随点击未落）；run2 count=101/p1_x=12/epochs_after_remount=[null,2]/idle_no_tick=false/exit1（`focus_lost`＋`resized_event 0`/`resized_applied 0`：0×0 `Resized` 经 `size.width.max(1)/scale` 变 logical 0.5×0.5 且被 host 接受；新 `Focused(false)` 清焦点后探针未确定性重取）。记录的 202 是单次绿，非可重复 oracle；真 native 手工门 pending。
- AC-02 **PASS**：亲跑 `character3d_api` 21/21、`node3d_model_objects_flow` 8/8（含 g14）、`node3d_runtime_flow` 7/7（128 稀疏/vs06）；hidden-parent 注册 solid 子行 x≈2.598 阻挡＋未注册对照 x>5；失败出口 `v_ok/v_fail==0`、`ag_visible==0`、`q1>0,q2==0`（fault 10 后）；127/128/129 被原 budget 门拒绝、8 注册可用；跨文档 Session 不可达=每 Run `Play::new` 新 Session＋Run 起始一次 `reset`；043 可见行/容量未改（`character3d_v1.at` 零 diff）。
- AC-03 **PASS（证据强度注）**：亲跑 `host_lifetime_flow` 1/1（49.00s）：8 host×20 轮＋tail；每轮 Run 级 4 slots/7,584,000B 回基线、host 级 store 7 meshes/136,952B 与 peak 不增；9 host 基线日志逐个 `(4,7584000,7,136952)`；tail 可 Run；exit0。坏件修复复用 vs06/vs05 原证据。注：跨 host 基线只打印未断言；V-04“最后 host 释放后 host 级 owner 归零”未直接观测/断言。
- AC-04 **PARTIAL**：59/59 命名并集亲跑（21+1+6+8+3+7+6+4+3=59）exit0；6 触面文件 `rustfmt --check` exit0；但 AC-01 原生证据非确定、SD-01 `CursorLeft` 清焦点措辞不准、G3 11 项=AG025 跨仓未跑、pilot 生成门因 `--locked` Cargo.lock 未跑；pre-existing `authoring_feasibility` 缺 `hud_overlay` 与 013 漂移非本 Plan 引入。
- AC-05=本 review（needs_fix）；AC-06/07 merge-owned 未领。
| pending_closeout_gates: 真 native SendInput/焦点捕获/DPI 真切换手工门（D01，`--manual` 可执行记录器 auto_pass:false）；G3 11 项内容验收=AG025 跨仓；pilot 生成门（pilot Cargo.lock 需随 auto-lang tip 更新，`--locked` 拒绝；未改生成器/冻结产物）；merge 冻结门/落主/canonical SD/归档/清理/输出对账 | findings: F-066-R2-01 (P1, blocking, 产品/验证) posted `--probe` 非确定性＋0×0 resize 链：`experimental/autographics-auto-driver/src/bin/mixed_regions_window.rs` Resized/Focused/CursorLeft 臂与 phase6/12、`src/node_mixed_regions_host.rs::resize`；run2 证明 0×0 `Resized` 被 max(1)/scale 变 0.5×0.5 并接受，run1 无 focus_lost 的 phase6 strip 点击未落（count 102）；closing oracle=0×0/最小 logical 尺寸处理＋host/window 回归（0×0 后 widget bounds/click 路由不变或恢复）＋探针 focus 重取/零 resize 命名失败＋3 连 `--probe` 全绿（count=202/post_resize_hit=true/idle_no_tick=true）＋若 run1 仍复现则补 host 级 post-hide layout-follow click 以区分产品 hit-test 与探针时序；F-066-R2-02 (P3, receipt accuracy) §9 AC-01 “五点击+五按键”应为“六点击+五按键”（report/Spec 均 6 clicks/5 keys；Spec SD-01 已正确）；F-066-R2-03 (P3, SD-01 precision) `Focused(false)`/`CursorLeft` 句把“清焦点”同时归给两者；代码只有 `window_focus_lost` 清键盘焦点，`CursorLeft` 只 `cancel_all` 取消捕获，回归也只覆盖 window_focus_lost；需拆句且不新增合同；F-066-R2-04 (P3, 证据强度) `host_lifetime_flow` 只 eprintln 9 host 基线、未断言跨 host 相等；V-04“最后 host 释放后 owner 归零”未直接观测；需补断言或收窄 Spec/V-04 措辞 | evidence: ①`git -C D:/autostack/.wt/ui-066/auto-ui status --short` 空；HEAD=`e95da1db`；base=`3d64877d`；branch=`codex/plan-066-dev`；②required union 亲跑=21/21+8/8+6/6+1/1（host_lifetime 49.00s）exit0；remaining 5 targets=3/3+7/7+6/6+4/4+3/3 exit0，总 59/59；③6 文件 rustfmt --check exit0；④native raw：`plan-artifacts/plan-066/auto-ui/t03-review-native{,2,3}/mixed-regions-native.json`（102/false、101/false、202/true）＋`review-final-r2-native{,2,3}.log`；⑤`python tools/verify_plan013.py` exit1 source set drift；⑥`Scene3dContent.hud_overlay`（surface_runtime.rs:99）vs 未改 `tests/authoring_feasibility.rs:68-70` 缺该字段=pre-existing；⑦`review-final-r2-tests*.log` | next: work（同树 `D:/autostack/.wt/ui-066/auto-ui`，branch `codex/plan-066-dev`）按 F-066-R2-01..04 定向修；重开 T-03/T-06，Plan 状态应回 `executing`、current_step 4/9（T-01/02/04/05 完成）；修后同上下文复核 AC-01/AC-04 与 closing oracle，再 final。

2026-11-07 / stage: work（review needs_fix 修复） | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: pass（execution_done，6/9；状态经 executing 3/9 重开后再闭） | worktree_path: `D:/autostack/.wt/ui-066/auto-ui` | branch: `codex/plan-066-dev` | base_commit: `3d64877d` | code_commit: `5ee7101c`（前序 `2c77aa80`） | docs_commit: 本条与SD随merge落主 | task_ids: T-03, T-05, T-06（T-01/T-02/T-04 未变，仍闭） | review_receipt_ref: F-066-R2-01..04 | findings_closed:
- **F-066-R2-01（P1）闭**：①根因1＝最小化 `Resized(0×0)` 经 `max(1)/scale` 变 0.5×0.5 且宿主接受 → `MixedRegionsHost::resize` 对非有限/小于1×1逻辑尺寸具名拒绝且不改状态；窗口层忽略0面积 `Resized`（计数`zero_resizes`）。永久回归 `flow_degenerate_resize_is_refused_and_the_layout_recovers`（0×0、0×560、480×0、0.5×0.5、NaN 全拒；布局/命中保持；真实 resize 后再点击成功）。②根因2＝固定700ms相位使“隐藏后布局跟随点击”与 Enter 竞速 → 探针改状态门（p1 `region_epoch` 变 None 且 strip bounds 变化后才点；`logical()==RESIZED` 后才 post-resize 命中），键盘改自适应 Tab 作业（真实 `Focused(false)` 后重取焦点，12次上限后具名 `failure`）。③同一命令连续3次 `--probe` 均 exit0 且 count=202 / post_resize_hit=true / idle_no_tick=true / epochs 3→7 / scale 2.0 / 6 clicks+5 keys / failure=null（`t03-fix-native-r2/run{1,2,3}`）。未以重试掩盖失败：所有门为状态断言，超时写 failure 并 exit1。
- **F-066-R2-02（P3）闭**：口径统一为六点击+五按键（上一条 work 收据“五点击”为笔误，以本条为准；SD-01 原已正确）。
- **F-066-R2-03（P3）闭**：SD-01/§3 改为 `Focused(false)` 清键盘焦点、`CursorLeft` 只取消在途按下/捕获；flow 新增“pointer leave 后 Enter 仍激活”断言（键盘焦点保留）。
- **F-066-R2-04（P3）闭**：`host_lifetime_flow` 对 8 个 host 的 pre-run 基线逐一断言相同、tail host 在全部 drop 后断言同基线；SD-02 改述为“Run 不新增/不泄漏 target·mesh owner；4 targets/7,584,000B、7 meshes/136,952B 为 host 级常驻；drop 后观察面＝后续 host 同基线”，不再写不可直接观测的“host级owner归零”。
| verification: union 60例全绿（`t06-union-r2.log`：character3d_api 21、node3d_runtime_flow 7、node3d_model_objects_flow 8、mixed_regions_lifecycle_flow 7、storage 3、host_lifetime_flow 1、node_ui_flow 3、node_mixed_views_flow 6、node_ui_few_slot_flow 4；`--test-threads=1`）；`rustfmt --check` 5文件过；native 3/3 exit0（r2 报告在材料scope） | residuals: 同前条（native SendInput/焦点捕获/DPI 真切换=具名手工门pending；G3 11项=AG025；pilot 生成门未跑=lock drift；pre-existing 013 源集漂移与 authoring_feasibility 编译红非本Plan） | pending_closeout_gates: merge 冻结门/落主/canonical SD/归档/清理/输出对账 | blockers: none | next_responsible_role: review（受影响 AC-01/AC-04 定向 re-review，然后完整 final）

stage: goal | plan_id: PLAN-066 | plan_revision: r2 | round: 2 | from: review | to: review | status: execution_done | outcome: pass（review needs_fix → 修复 → re-review） | worktree: `D:/autostack/.wt/ui-066/auto-ui` | commit: `5ee7101c` | next: review
2026-11-07 / stage: review | review_scope: final（repair re-review：F-066-R2-01..04 定向复核＋AC-01/AC-04 重判；AC-02/AC-03 仅复用未被改动面的既有证据） | reviewer_identity/limitations: 独立 final-review 子代理新上下文（同机同 session host；前一次 final 尝试产出探针证据后崩溃，本条为同机重试并自跑全部证据；非人工/非独立机器；全部命令串行单实例、driver `--test-threads=1`；真 native SendInput/焦点捕获/DPI 真切换手工门未跑——本机唯一 1920×1280 虚拟显示、单一 scale；未在主检出跑 cargo、未跑 prepare_sdk、未改实现/测试/Spec；窗口层 0 面积 `Resized` 忽略臂=源码核读＋宿主级回归，未做窗口级自动化，记为证据边界而非 finding） | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: pass（AC-01..AC-04 通过；AC-05=本条；AC-06/07 merge-owned 未领） | worktree_path: `D:/autostack/.wt/ui-066/auto-ui` | branch: `codex/plan-066-dev` | reviewed_commit: `5ee7101cc51c2a7caa3eea47aca2a607da16573b` | base_commit: `3d64877d3837faff6c1175bdb4fee248c3f02418` | dependency_revisions: 无新依赖；`git diff --stat e95da1db..HEAD`=7 文件（SD-01/02 两份 Spec＋driver `src/bin/mixed_regions_window.rs`/`src/node_mixed_regions_host.rs`/`src/node_ui.rs`＋`tests/{mixed_regions_lifecycle_flow,host_lifetime_flow}.rs`），`api/character3d_v1.at`、`tests/character3d_api.rs` 零 diff；`node_ui.rs` 增量仅诊断访问器 `focus_active()`（不改派发）；外置 `CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-066/auto-ui/target` | spec_inputs: `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md` sha256 `4753CD6538C34DD84C4374BC37ED59C3A23F6361B79F7DCF3DE235B071D2183E`; `docs/specs/auto-graphics/node3d-character-and-runtime-camera.md` sha256 `6C5B5FD356DA28077D4E510A5A4D9AE210F2B4A38E467F904903544B93600E7D`（worktree HEAD 实算；条内表述与 1–3 实现一致，真 native 手工门在两者均显式 pending） | acceptance_results:
- AC-01 **PASS**：同命令连续 3 次 `mixed_regions_window --probe` 全部 exit0，report 均 count=202 / post_resize_hit=true / idle_no_tick=true / epochs_before=[3,2]→after=[7,2] / failure=null / posted_clicks=6 / posted_keys=5 / scale 2.0 / logical_sizes [[480,560],[400,420]] / zero_resizes=0（frames 347/355/368）；退化尺寸由宿主具名拒绝且窗口层忽略 0 面积事件，永久回归 `flow_degenerate_resize_is_refused_and_the_layout_recovers` 亲跑绿；真 native SendInput/焦点捕获/DPI 真切换=具名手工门 pending（D01，`--manual` auto_pass:false，本机单显示/单 scale 不可自动取证），未宣称已验。
- AC-02 **PASS（复用未变面）**：`e95da1db..HEAD` 未触 `character3d_v1.at`/`tests/character3d_api.rs`；`node_mixed_regions_host.rs` 增量仅 `resize` 入参校验（不涉碰撞/预算/Session 复用面）；前次 final 亲跑 21/21＋8/8（g14）＋7/7 证据仍成立。
- AC-03 **PASS（前次证据强度注已闭）**：亲跑 `host_lifetime_flow` 1/1（47.03s）exit0；源码现逐 host `assert_eq!(base, first)`、循环后 `baselines.iter().all(|b| *b == baselines[0])`、tail host 在全部 drop 后断言 `tail_ledger == baselines[0]`，`eprintln` 仅补充；SD-02 已把“host 级 owner 归零”改述为可观测面（后续 host 同基线），无超观测断言。
- AC-04 **PASS**：SD-01 拆句为 `Focused(false)` 清键盘焦点／`CursorLeft` 只取消在途按下捕获，并新增“pointer leave 后 Enter 仍激活”断言；口径六点击＋五按键与实测一致；5 个受触 rs 文件 `rustfmt --check` exit0；59 例并集除两处受影响 target 外输入未变（本次亲跑 lifecycle 7/7、host_lifetime 1/1）。G3 11 项=AG025 跨仓、pilot 生成门仍为具名 pending/残余。
- AC-05=本 review（pass）；AC-06/07 merge-owned 未领。

| finding | 期望（closing oracle） | 实测 `5ee7101c` | 结论 |
|---|---|---|---|
| F-066-R2-01（P1） | 0×0/亚像素被拒且不毁状态；探针改状态门＋焦点自适应重取；3 连 `--probe` 全绿 | `node_mixed_regions_host.rs:444-459` 在 `rt.resize_surface` 之前对非有限/<1 逻辑尺寸 `Err`，不写 `ci.set_sizes`/`self.logical`；`mixed_regions_window.rs:595-603` 0 面积 `Resized` 计 `zero_resizes` 后 return；phase5 需 `region_epoch(P1).is_none()`＋strip bounds 变化（:383-416）、phase9 需 `logical()==RESIZED`（:466-487）、kb 作业每帧 Tab 至目标聚焦、>12 次具名 `keyboard focus timeout`（:194-235）；timeout→`failure`→report+exit1（:289-299、559-561）；回归覆盖 0×0/0×560/480×0/0.5×0.5/NaN 全拒、logical/布局/命中保持、真实 500×560 恢复；实测 3/3 绿、count=202 | closed |
| F-066-R2-02（P3） | 收据口径应为六点击＋五按键 | report `posted_clicks:6`/`posted_keys:5`×3；SD-01“真实 OS 窗口 posted 注入六点击＋五按键”；公式与 phase0..12 点击序列一致 | closed |
| F-066-R2-03（P3） | SD 拆句且不新增合同；补 pointer-leave 断言 | SD-01 两处均拆为 `Focused(false)` 清键盘焦点／`CursorLeft` 只取消在途捕获；`CursorLeft` 臂仅 `cancel_all`（:667-675）；回归断言 `pointer leave keeps keyboard focus` 且其后 Enter 激活成功（count 续进） | closed |
| F-066-R2-04（P3） | 跨 host 基线必须断言而非打印；tail 断言；SD 改述 | 逐 host `assert_eq!(base, first)`＋`all(== first)`＋`tail_ledger == baselines[0]`，`baselines.len()==HOSTS`；亲跑 1/1（47.03s）绿；SD-02 改述为可观察面 | closed |

| pending_closeout_gates: 真 native SendInput/焦点捕获/DPI 真切换手工门（D01，`--manual` 可执行记录器 auto_pass:false，本机单 1920×1280 虚拟显示/单 scale 不可自动取证）；G3 11 项内容验收=AG025 跨仓 owner；pilot 生成门（pilot `Cargo.lock` 随 auto-lang tip 更新前 `--locked` 拒绝；未改生成器/冻结产物）；merge 冻结门/落主/canonical SD-01/02/同号归档/清理与全输出对账 | findings: none（F-066-R2-01..04 全部关闭，本轮无新 finding；`verify_plan013.py` 源集漂移与 `authoring_feasibility` 缺 `hud_overlay` 仍为 pre-existing、非本 Plan 引入） | evidence: ①`git -C D:/autostack/.wt/ui-066/auto-ui status --short` 空；HEAD=`5ee7101c`；branch=`codex/plan-066-dev`；base=`3d64877d`；`git diff --name-only e95da1db..HEAD`=上列 7 文件；②probe 自跑 3 连=exit0×3，报告 `plan-artifacts/plan-066/auto-ui/t07b-review-native/run{1,2,3}/mixed-regions-native.json`；③`cargo test --test mixed_regions_lifecycle_flow -- --test-threads=1`=7/7（3.74s）exit0；④`cargo test --test host_lifetime_flow -- --test-threads=1`=1/1（47.03s）exit0；⑤`rustfmt --edition 2021 --check` 5 文件 exit0；⑥源码核读定位见表；⑦SD 实算 sha256 同上；⑧主检出 Plan 为未跟踪文件，`git status --short` 仍为既有行（`?? docs/plans/066-…` 等），未新增/删除任何 repo 文件 | next: merge（T-08）按冻结 `5ee7101c` 落主并沉淀 SD-01/02；上述具名 pending 门须在 merge 收口时显式保留/关闭，不得据本条 pass 视为已绿

2026-11-07 / stage: merge | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: pass（prepared/landed/canonical_verified/archived；cleaned 见后续回执） | delivery_commit: `5a53b7895b899fa39f75e09a586cf0a63bbb8e29` | reviewed_commit: `5ee7101c`（rebase前） | base_commit: `3d64877d` → rebase onto `b95b910d` | range_diff: 3/3 `=`（`cf7f3995`=2c77aa80、`de85fd5a`=e95da1db、`5a53b789`=5ee7101c；patch等价，测试证据对代码面沿用）| canonical_verified: `docs/specs/auto-graphics/node-mixed-regions-lifecycle.md` sha256 `4753CD6538C34DD84C4374BC37ED59C3A23F6361B79F7DCF3DE235B071D2183E`；`node3d-character-and-runtime-camera.md` sha256 `6C5B5FD356DA28077D4E510A5A4D9AE210F2B4A38E467F904903544B93600E7D`（main 与 worktree 实算一致，同review收据）| landed: `git merge --ff-only codex/plan-066-dev`；main tip=`5a53b789` 无merge commit | landing_smoke_on_main: character3d_api 21/21、mixed_regions_lifecycle_flow 7/7、native `--probe` exit0 且 count=202/post_resize_hit/idle_no_tick（`plan-artifacts/plan-066/auto-ui/landing-native/`，本回执折入）| archived: 本文件 `docs/plans/archived/066-…md`（本提交）| observations: ①`docs/plans/README.md`/`.next-id`/design docs 在主检出存在他会话未提交WIP，按不覆盖他人WIP规则未改；066行仍指向本Plan（应用派生状态，路径正确），其状态文本由该文件owner后续统一刷新；②`crates/auto-ui-runtime/test/ui/terminal_pixel/*.png` 主检出未跟踪旧产物（D04）未纳入；③batch regression due check：auto-ui 无 `.last-batch-regression.json` 机制（该门属 auto-lang PLAN-896/898），本仓不适用，记录为不适用而非绿；④landing artifacts：本仓为 Rust库＋experimental driver，无生产 release binary/web bundle 部署面，无 stale artifact。| pending: cleanup（guard→worktree/branch/group→scratch 删除→registry）+T-09 | blockers: none | next: cleanup

2026-11-07 / stage: merge（cleanup） | plan_id: auto-ui/PLAN-066 | plan_revision: r2 | outcome: pass（cleaned，T-09 完，9/9） | delivery_commit: `5a53b789` | guard: `"C:/Program Files/Git/bin/bash.exe" D:/autostack/wt-guard.sh D:/autostack/.wt/ui-066/auto-ui` = clean（exit0；本会话PATH的WSL bash会误报clean，见D05） | worktree_removed: `git worktree remove D:/autostack/.wt/ui-066/auto-ui` exit0，`git branch -d codex/plan-066-dev`（was `5a53b789`），空组 `.wt/ui-066` removed；verify：worktree list 无 ui-066、branch list 空、路径不存在 | scratch_deleted: `D:/autostack/plan-artifacts/plan-066/`（resolved 绝对路径已核；仅 `auto-ui/` 本仓；parent 无别仓）guard clean 后整删 = 20,138 files / 12,961,678,581 bytes；无具名 hold | registry: `.retention/policy.json` 追加 `auto-ui/PLAN-066` deleted-at-merge（delivery `5a53b789`、archive commit/sha256 见 registry 记录） | notes: batch due check 不适用（auto-ui 无 `.last-batch-regression.json` 机制）；landing artifacts 不适用（本仓无生产 release binary/web bundle 部署面）；主检出其余 WIP 为他会话（docs/design、README、.next-id、archived/060、untracked test PNG/062/063/067），未纳入未删除 | blockers: none

## 10. 待澄清事项

D01：真实DPI设备/焦点窗口在T02安排，缺条件保pending。**work后状态（2026-11-07）**：本机唯一1920×1280虚拟显示、单一scale，第二DPI环境不可达；`mixed_regions_window --manual`可执行手工门已交（记录真实键/点击/失焦/离窗/resize/scale，auto_pass:false），真DPI切换保持pending，后续由用户手工取证或AG025同版候选复核。
D02：无法归因GPU AV不能判环境豁免；具名记录最小反例和后继需求。**work后状态**：本机同进程8 Host顺序创建/使用/drop+tail host 20轮×8未见AV（exit0，两次50.20s/68.64s），未按“环境豁免”处理——结论限于离屏Host覆盖的device/surface/store退出路径，窗口化宿主退出路径未纳入，若后续复现需具名最小反例。**review后修复（F-066-R2-01，`5ee7101c`）**：窗口层最小化`Resized(0×0)`曾产生0.5×0.5逻辑表面并接受（叠加固定相位竞速造成探针1绿2红）；现宿主拒绝退化尺寸、窗口忽略0面积事件，探针状态门+自适应键盘，同命令3连跑全绿 exit0。GPU AV面不因此改判。
D03：065共享面按符号交接，不因可准备就同时写。**work后状态**：本轮只改`node_ui`键盘焦点路由（不触文本槽/半缓冲/生成面），`node_ui_few_slot_flow` 4/4复跑绿；065交付`55c20a86`未重开。AG025配方：候选须含065交付＋本Plan code_commit，禁两引擎分别绿拼同版。
D04（work后新增，主检出pre-existing）：`crates/auto-ui-runtime/test/ui/terminal_pixel/*.png`为未跟踪的旧测试产物（4文件≈113KB，非本Plan生成、未ignore），开工即存在；未纳入、未删除。merge清主检出处前须由其owner路由（stash/移交）或具名保留，本Plan不静默收编。
D05（merge清理注意，2026-11-07 work实测）：本会话PATH中的`bash`是WSL（`C:\Windows\system32\bash.exe`），直接`bash D:/autostack/wt-guard.sh`因缺`cygpath`/`cmd`会误报clean；必须用Git bash：`"C:/Program Files/Git/bin/bash.exe" D:/autostack/wt-guard.sh <worktree>`。已用junction反控制验证：有链接时BLOCKED(exit1)、摘除后clean(exit0)；本次worktree `D:/autostack/.wt/ui-066/auto-ui` 在该命令下clean。
