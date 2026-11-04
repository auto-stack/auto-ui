---
plan_id: PLAN-039
status: drafting
feature_name: AutoGraphics三维查询与有限二维三维组合作者接线
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-021, auto-ui/PLAN-024, auto-ui/PLAN-030]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node3d-queries-and-mixed-markers.md]
touched_goals: [GOAL-008]
affects: [auto-ui-dialect, autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-039：三维segment/sweep、动态组与二维诊断标记Node接线

## 0. 变更摘要

把已有v3/v4三维查询接到普通Node/use/局部model-msg-on、具名GLB/行/组引用及typed结果；保留G05/3d真实多部件GLB、动态父组和同画面二维标记。不是重新实现空间算法，也不强加character/camera。本计划与UI035公共音画组合独立；内容迁移另见[AG017](../../../ag-examples/docs/plans/017-autographics-gallery-node3d-queries.md)。共同设计见[下一阶段](../design/autographics/node3d-queries-and-complete-game.md)。drafting r1、0/10，暂不实施。

## 1. 目标

1. `query3d`的segment/sweep及结果访问沿原三维v3/v4数值、收费、latest-slot生命周期，真实Auto执行查询与标记业务。
2. 普通use复用有局部model/msg/on的组件；具名引用可操作支持域内叶和group，组不成为可查询几何。两个同定义实例状态/receiver/行引用互不串线。
3. 一个有限查询世界同时提交三维内容与二维诊断层，两个呈现出口共用同一Node状态和事件；二维标记不能被删掉或冒充生产UI/HUD。
4. 原G05/3d九义务、segment/sweep/changed三变体及新反例经完整作者→生成→独立准入→VM→Host/GPU/保存搬迁链成立，交付可冻结的内容消费接口。

非目标：完整G3游戏/声音HUD/角色相机组合、精确三角/OBB或无限射线、运动响应/刚体、动态拓扑/资源/yaw/scale、任意UI内嵌3D视口、多屏呈现变体选择、ACE/AOT/ECS/CPC/跨端。不得把本profile的有限双出口解释为所有旧profile自动支持混合呈现。

## 2. 架构方案

复用021 typed query转换与024/030组件/具名资源/refs机制，新增显式查询profile（工作名`queries-3d`，最终CLI/API身份T-02冻结）。静态all-mesh Descriptor仍由driver真实资源建立，Auto执行v3/v4查询与局部写，Host验证完整返回状态并渲染3D＋2D层。不新增第二几何解码器、第二运行图或Rust命中/玩法逻辑。

拟唯一实施/复审树`D:/autostack/.wt/ui-039/auto-ui`，分支`codex/plan-039-dev`；目前不创建。UI035树与开发tip只读，不作为依赖；共享parser/load/translate/assembly短窗单写，合入前核交叠受影响面。pilot语言依赖与driver固定SDK681依赖图分开。

## 3. 技术栈

已观察路径：`crates/auto-ui-dialect/src/node2d/parse.rs`（当前同Node双view无条件拒绝）；`experimental/autographics-node2d-pilot/src/{load.rs,translate.rs,main.rs,lower_components3d.rs,lower_components3d_runtime.rs}`（group位置/显隐目前只许静态）；021 `src/lower_components.rs`查询验证与`translate.rs`发射；driver `api/behavior_segment_v3.at`、`behavior_sweep_v4.at`与`src/authoring/`空间准入/实际Host。拟新增`lower_queries3d.rs`、`tests/node3d_queries_{gates,flow}.rs`、`experimental/autographics-node2d-pilot/tools/verify_plan039.py`及永久最小夹具；新路径尚未存在，T-03/04建立可运行入口。

## 4. 需求分析与背景调查

用户2026-11-03授权继续下一阶段规划，允许两责任仓合同/设计/路线/导航与auto-lang战略簿记；未授权实施、建树、探针、构建或创建agent。此前“引擎＋完整游戏＋Demo三线”沿用。编号039由本仓独占取号；036..038属于其他路线，不重用。

规划观察基线auto-ui `e6ef5381eefba74bb9d167347762ffecd7ff68d6`，ag-examples `92dba6f7dd764415f87451593a63e75637accbcf`；开工冻结新实际基线，不把未提交035/theming WIP打包。当前Specs：

| 来源 | 内容SHA256（规划读取的盘上字节，开工核canonical换行与交付） |
|---|---|
| `docs/specs/auto-graphics/node2d-queries.md` | `0d930490f45d9a1297443eb867718976a8d940d0812bace278a8701e383cf20b` |
| `docs/specs/auto-graphics/node3d-foundation.md`（024交付64166db） | `9451cf0f3c6a82505caa771b54728f1a6fb06a2ff74343052dd164dd2aee155a` |
| `docs/specs/auto-graphics/node3d-symbolic-component-bindings.md`（030交付804d7303/独立复审补录c29e84cf） | `6ed0bf070de48c3fae4ff8c1687dca9d67c8645010aef5783e49ca14b3352e18` |
| auto-lang `docs/specs/auto-graphics/segment-query-profile.md` | `6337e8508fc2af3a785507b11fc0781971843ea7b2cdca385be22059525b3c64` |
| auto-lang `docs/specs/auto-graphics/swept-aabb-profile.md` | `349df4451fa3957c0d85574f15087ce6ed1b943dda49f69a0cd2e010bb8967e9` |

同时读取auto-lang `spatial-query-profile.md`和`overview.md`，ag-examples `tools/verify_examples.py::verify_g05`、`g05-queries/g05-3d/{scene.agscene,segment.at,sweep.at,variants/segment-changed.at}`。原GLB `42c9da673fc67ccb0c65d4b337608834efd28a499f7e460866dcfc5ff2a0c671`/1528B，3个模型叶/门/两个2D quad/一个3D父组；actor静态scale(.4,.7,.5)、yaw.2/pitch.15且组初位(1,0,0)。原组/叶/几何/有效显隐/row顺序必须保持或逐行明确映射。

## 5. 详细设计

### 5.1 作者及呈现支持域

拟声明面`query3d.segment(ignore,visible_only,ax,ay,az,bx,by,bz)`、`query3d.sweep(mover,visible_only,dx,dy,dz)`，typed cast/sweep局部`let`结果与021相同方法族（axis0..2）；实际拼写/nominal参数形态由T-02全链冻结。引用不得让作者写数字ordinal：复用`refs.<实例>`/`self.row()`；group需明确具名路径和叶/组类型，允许local/world/三轴place/shift/show，不允许group作cast ignore或sweep mover。叶写/父写后的查询观察同callback新world，getter仍观察原snapshot。不得把getter读免费缓存或换成Rust命中。

最小双出口提案：仅该显式查询profile允许同根`view-3d`＋`view-2d`同时呈现，共用model/msg/on；子Node各属一个维度，两个出口不是运行环境择一分支，二维诊断标记使用原屏幕坐标/颜色/显隐，原3D camera/projection不改变。现有AST已有两个字段但parse无条件拒绝；T-02先证把限制移至profile准入或等价有限组合，保所有旧profile原拒绝及冻结产品，不能直接删全局检查就认完成。禁止跨维parent/一个实例重复实例化/双重Life调用；多出口不复制model或Tick。一般UI `view { row { view-2d/view-3d } }`及环境变体仍另题，不在本期扩面。

### 5.2 原数值、错误、收费和资源

原有限f32世界坐标/all-mesh八角Ry*Rx*S AABB；闭[0,1]、最近实际t、exact tie低Run ordinal、signed-zero平行不除、inside/boundary起点t0/n0、外部入射同t轴X/Y/Z择优、t1切触。零长segment为点查；sweep固定朝向Minkowski/世界位移、无初始接触时零位移miss，初始overlap t0/start_overlap1/n0；sweep不隐含移动/clearance。hit/order/normal不靠观察容差放宽；t容差max(1e-6,8EPS)，位置max(1e-3,8EPS*max(1,abs(e),abs(a)))。

3D过滤只取同维合法叶；effective visibility与alpha分开，2D标记不混入3D候选；全部匹配几何在nearest剪枝前检查。不可用几何9、非有限运算12、stale/错kind13、序号耗尽14不降为miss。latest slot在query attempt/回调边界失效，失败attempt同样失效，正i32 token不回绕；callback拒绝是既有profile错误策略，不提升为AG全世界故障原则。

原共享32credits/callback：cast8/sweep16/getter1/AABB1，先收费后读，不能每模块另开池。原segment两次cast+11getter=27；sweep16+7getter=23；按真实生成调用重算（变换绑定写仍原价/原写额度，不把这些空间计费当全部费用）。8有状态实例/128行及原Descriptor/VM/source/time/mesh/资源包络不变。100000steps/120resumes/250ms callback、30s init/2s整tick、8ms pump目标口径分别记录；128宽/深原边界保留。SDK固定7b987ca8…的681文件锁/保留副本处理沿现有工具，不换当前SDK。

### 5.3 先证与交付

T-02最多两机制候选，全链用原多part GLB/动态组/双维标记、三变体与两个同定义query组件跑到真实Host/GPU，独立f32 oracle；选定后冻结CLI、profile/API/准确费用、refs/group域/拒绝矩阵、实际源/锁。任一关键路径失败转needs_replan，不删标记/换圆柱/放宽预算。先证不是通过自制JSON/直接旧脚本注入绕过作者链。

产品沿scene/world/identity/closure/resources，模块传递闭包与原件身份绑定；A→B只用包内闭包冷再生、真Run/编辑保存/SaveAs/关开、坏world/GLB/缺模块拒绝→精确修复→同入口再Run。20轮Stop/newRun与实际资源owner/外部pin释放；不每tick重扫/上传/回读。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node3d-queries-and-mixed-markers.md | 3D Node无查询/动态group/双维标记→显式有限profile，列typed结果、具名组/叶、共model双出口、数值/费用/错误/身份/消费 | G05保持原能力迁作者形式；旧profile不改 | AC-01..06 |

## 6. 测试设计

全部为拟执行，T-03/04使新入口可运行。work跑实际受影响文件/命名case；review同权威树独立上下文复核逐AC和反例，绑定r1+提交+SDK/工具/输入；同实现会话重跑不充独立final。merge冻结后复用未变有效证据并跑必要定向并集；不为merge跑全AG/语言根/无关主题测试。全AG仍里程碑pending。用户未指定reviewer，不创建agent/第二树。

| case | AC | owner_stage | 实际/拟入口及关键可区分oracle | 覆盖界限 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | 新`experimental/autographics-node2d-pilot/tools/verify_plan039.py --controls`＋dialect node2d定向测试/new gates：普通use/局部state/msg/两同定义实例、具名叶/组；非法循环/错refs/组作几何/跨维parent/双重复实例化具名拒绝；旧profile混合拒绝保持 | 新语法诊断与身份，不代真GPU |
| V-S02 | AC-01..03 | work | 新`node3d_queries_flow`：独立f32 slab/sweep与真实Auto逐点hit/node/t/point/origin/normal；原多part所有顶点箱/动态父写/目标0→.5写后重查，旧token位置不变；真实3D几何＋2D marker像素对照/changed reach4→2 | 三维v3/v4有限域，非三角物理 |
| V-S03 | AC-03/04 | work | flow/gates：closed t0/t1、signed-zero/零长/零位移、overlap、tie ordinal/X-Y-Z、hidden/alpha0/跨维过滤、几何9/运算12；错kind/stale13、耗尽14、32通过/33拒绝10且last-good；两组件共享32 | 原数值/slot/费用/128宽深保持，非无界性能承诺 |
| V-S04 | AC-02/05 | work | flow：Start一次/每Step一次Tick/Pause与render-only零Tick、20轮Stop/fresh/旧Run拒绝；移除父更新/遗漏GLB一部件/重复Tick/丢2D层实际变异须使对应oracle红；owner和合法外部pin释放 | OS焦点探针缺证具名pending，不伪通过 |
| V-S05 | AC-05/06 | work | verifier/flow：原G05九义务逐名映射、保存关开/新链SaveAs/变化、A→B冷再生hash等值/实际Run，world/GLB/源/锁一字节坏件各拒绝→修复→再Run；固定SDK两图核 | 内容包完整验收归AG017；本期公共夹具先证 |
| V-S06 | AC-04..06 | work | `cargo test -p auto-ui-dialect --test node2d_pilot`（若parser受改）；`cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node3d_queries_gates`（新target由T-03建立）；driver `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test node3d_queries_flow -- --test-threads=1`（新target由T-04建立）及受影响新gates（GPU单实例`--test-threads=1`）；021/024/030及交叠035命名受影响case/冻结再生控制 | T-02登记真实target和命名并集，不写未存在入口为已执行 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | Node/use/局部状态事件、typed三维查询/具名叶与group真实运行且无假角色相机/Rust业务 | V-S01/02 |
| AC-02 | work | 多part GLB原几何/动态组/两维同画面、模型与事件局部性、多实例隔离与原三变体成立 | V-S01/02/04 |
| AC-03 | work | 原数值/过滤/dirtyworld/snapshot/错误与latest slot完整保持、反例真实区分 | V-S02/03 |
| AC-04 | work | 原共享32/128/8与VM资源域不抬，费用边界和原合法峰值实测，无每模块免费池 | V-S03/06 |
| AC-05 | work | 生命周期/实际owner、公开保存搬迁冷消费/坏件恢复、原G05九义务/旧档冻结通过且剩余gap具名 | V-S04..06 |
| AC-06 | work | SD候选与AG017可锁消费交接含准确API/工具/依赖/身份/原额/受影响证据 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/diff/SD/遗漏延期workaround/告警审查，finding闭环绑定revision与源 | T-07 |
| AC-08 | merge | 冻结门及来源核后落主，SD-01沉淀canonical核字节，归档和实际消费身份齐 | T-08/09 |
| AC-09 | merge | fresh guard清自有树/分支/空组、全输出对账/摘要折Plan/删scope，具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；021/024/030已交付 | 冻结实际源码/Spec/SDK/原G05九义务/owner；唯一树及材料scope登记，UI035不追tip | AC-01/05 | [ ] |
| T-02 | work | T-01 | 最多两机制全链先证GLB＋组＋两维标记/多实例/收费，冻结profile/CLI/引用域/支持拒绝矩阵与真实测试入口 | AC-01..06 | [ ] |
| T-03 | work | T-02 go | dialect/loader及新query3d lowering/具名组/typed结果/限定双出口；旧档拒绝和字节保持 | AC-01..04 | [ ] |
| T-04 | work | T-03 | 公共永久夹具/独立准入/gates/flow/verifier，V-S01..04与三变体/负控实际绿 | AC-01..05 | [ ] |
| T-05 | work | T-04 | 新链保存/关开/SaveAs/A→B/恢复/20轮/owner/完整受影响回归 | AC-04/05 | [ ] |
| T-06 | work | T-05 | SD-01候选/AG017固定消费交接与全部预算/gap/源身份摘要 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同权威树独立final与finding修复后同员复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后实际合master | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/归档/导航实际身份核验，不写派生JSON | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard/全部输出路径对账及scope清理/具名hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-03 / stage: new / plan_id: auto-ui/PLAN-039 / plan_revision: 1 / outcome: pass（可审阅合同，用户启动后T-01可开始；T-03受T-02 go约束） / next: work T-01。0/10。读取现有Specs/实际代码/原场景脚本，未运行新链或声称先证go。仅规划，不建树/材料scope/源码/探针/提案canonical Spec，不构建/GPU/OS自动化/创建agent。

2026-11-03本批规划预检（非实施/独立final）：三合同均0..10节、drafting/r1/0-10、10T/9AC/6V-S/1SD，work6/review1/merge3，10任务均未勾；依赖DAG无环，UI039不等035、AG016完整接线等035/相关来源证据、AG017完整迁移等039，各准备阶段独立。两仓独占取号后活动/archived/archive本地号唯一，UI next040（036..038他路线保留）、AG next018；271个所涉管理文档本地链接可解析（历史attachments不作新证据）。原G05九require/三变体及真实场景脚本已逐项承接；原G3八项/原关卡/声bank span/完整预算保留。按实际metadata的声明变体/generated文件和Node作者源交叉计数19/22，剩G04两维/G05三维；仅通用node_authoring注册表计16，另G11/2d与G12双入口属各自旧新作者接线，不漏计。`python tools/gallery.py validate --all`退出0：12逻辑项/22入口、许可一致。UI035最新主checkout收据e6ca119f/权威树65f5ce47为3任务完成、T-04实质进行中，未接管；034/015同实现会话复核限制如实记录，不冒称独立pass。未创建本批树/源码/素材/探针/提案canonical Spec/材料scope，无构建/GPU/OS自动化/agent。以上只证明规划结构与现有目录一致，未来CLI/先证/oracle均未执行。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | 有限双出口在公共parser与各profile准入边界的实现机制 | work T-02全链先证并冻结；不可将所有profile原拒绝静默取消 |
| D-02 | 具名group/叶寻址、序数与typed snapshot准确公开面 | work T-01/02核原GLB/行映射与候选，T-03之前决策；不要求用户重批同域等价机制 |
| D-03 | 原G05 OS14阶段和旧runner498ce814不可用 | 原AG009具名gap继续持有；新链实际CPU/GPU/保存证据必做，OS准入按现有政策，不拿新证据冒充旧runner历史Run |

开工材料唯一`D:/autostack/plan-artifacts/plan-039/auto-ui/`，登记owner/用途/在途；临时探针/日志/包/外置target均归此scope。merge摘要折本Plan后核全部实际输出、删除自有scope/构建物，具名未解决问题才可持有（owner/理由/关闭条件）。禁止attachments/第二暂存区/链接；删worktree前fresh `bash D:/autostack/wt-guard.sh <路径>`；不动同号他仓scope/其他WIP。
