# M2游戏优先路线：引擎、完整游戏、Gallery三线并行

2026-11-03，设计v4。AG用AutoUI式Node声明、局部model/msg/on、view-2d/view-3d、普通use与具名资源/事件表达图形应用。主线为完整2D闭环→3D公共表现/音画→完整3D游戏，Demo并行；当前目标桌面，M2两游戏/编辑/离线发行/Gallery/最终同一版本原分母不变，ACE/AOT/ECS/CPC/跨端不新增为前置。

## 1. 当前实际状态

| 事项 | 现场证据和边界 |
|---|---|
| 完整2D | [UI032](../../plans/archived/032-autographics-node2d-full-game-composition.md)交付a8c04d22；[AG014](../../../../ag-examples/docs/plans/archived/014-autographics-complete-2d-platformer.md)交付c141df8/独立final/Spec/归档，保其保存关开/SaveAs具名复用边界，不外推新3D豁免 |
| G11/3d | [AG013](../../../../ag-examples/docs/plans/archived/013-autographics-gallery-g11-3d-node-migration.md)交付b48b304；UI030独立复审证据已补录c29e84cf |
| 3D纹理/动画与G10 | [UI034](../../plans/archived/034-autographics-node3d-materials-and-animation.md)交付93eaa907，[AG015](../../../../ag-examples/docs/plans/archived/015-autographics-gallery-g10-node-migration.md)交付80e0b65；实际source/生成物/metadata为**19/22**，余g04两维/g05三维。两份复审收据明确同实现会话复核，不称独立上下文final；相关034持名hold未自行关闭 |
| 3D公共音画/完整G3组合 | [UI035](../../plans/035-autographics-node3d-audio-hud-composition.md)仍在实施，唯一树ui-035/auto-ui、codex/plan-035-dev、观察65f5ce47/主checkout收据e6ca119f；r1任务3/10、T-04实质实施中、元数据仍drafting。T-03先证候选不充最终交付，不接管owner WIP |
| 本轮新合同 | UI039查询、AG016完整G3、AG017 G05/3d，均drafting/r1/0-10，**仅规划** |

权威分母为auto-lang [M2-01..09](../../../../auto-lang/docs/design/autographics/m2-editor-and-public-game-prototypes.md)、[G2-01..09/G3-01..08/原关卡](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)；Gallery19/22不是M2百分比。旧UI011两候选33/32的no_go保历史，不能抹掉也不能外推成永久语言限制。

## 2. 下一批三合同

详见[查询与完整G3设计](node3d-queries-and-complete-game.md)。

| 线/责任仓 | 合同 | 可独立推进 | 完整实施硬门 |
|---|---|---|---|
| 引擎/auto-ui | [UI039](../../plans/039-autographics-node3d-queries-and-mixed-markers.md) | 021/024/030已交付基础上的3D segment/sweep、动态组、原2D标记有限组合；不等035 | T-02全链有界先证/冻结后T-03实现 |
| 完整游戏/ag-examples | [AG016](../../../../ag-examples/docs/plans/016-autographics-complete-3d-platformer.md) | T-01/02原G3八项、固定资源许可、模块/oracle/预算/包准备 | T-03完整接线硬等035实际交付/独立final/canonical/工具及相关034证据/hold核验；不等039/AG017 |
| Demo/ag-examples | [AG017](../../../../ag-examples/docs/plans/017-autographics-gallery-node3d-queries.md) | T-01原G05/3d三变体/九义务/多part GLB/原2d冻结/包准备 | T-02完整新链硬等039实际交付/独立final/canonical/工具；不等035/AG016 |

035继续原owner工作；039可与之独立推进，公用parser/load/translate/assembly短窗单写/核交叠回归。两内容Plan分树固定消费各自实际delivery，不追tip；共享gallery/packager/注册短窗协调，target/材料scope独占。真实GPU/窗口焦点/音频重门机器单实例。准备阶段并行不越依赖门，本轮不自动启动。

## 3. 引擎和内容的边界

034实际作者面是typed textured3d/animated3d、公开resource-meta两段生成、具名clip/pose和原模型角色；以实际canonical `gallery-node3d-textures-and-animation.md`为准，不用原提案名假装路径存在。035继续公共audio3d/hud3d与显式有界组合，原G3八项/actor-free服务均应实际证明；字段名/Idle等不成为Rust玩法约定。

039补原G05/3d真实缺口：group目前静态、parser同Node双view无条件拒绝、Node尚无三维typed query。新显式profile允许有限同根view-3d＋view-2d共model/msg/on共同呈现，子Node单维；T-02冻结机制，旧profile原拒绝/产品保持。查询仍真实Auto执行，driver只准备/复验实际all-mesh几何与结果；不换圆柱/删二维marker/强加character/camera。不是任意生产UI视口或环境变体系统。

原32credits/物理写32/8状态Node/8slot/8pair、120base+8reserved128、deep/contact64/mesh512、raw8身份32MiB单件8MiB/CPU-GPU各64MiB与VM/source/time/声HUD/动画预算均保持。多模块共享世界预算，不按Node增池；候选31/32不代正式G3最高分支，不能停输入/漏事件/移物/静音过门。

## 4. 游戏、Demo及独立发行

examples/保能力Demo，games/platformer-2d持已交付G2，games/platformer-3d由[AG016合同](../../../../ag-examples/docs/plans/016-autographics-complete-3d-platformer.md)/[内容设计](../../../../ag-examples/docs/design/g3-game-contract.md)承接；共用内容仓Plan号，不新建游戏Git仓。各游戏独立源码/关卡/资源manifest/固定引擎锁/公开运行/验收/离线包，不能借其它Demo目录/生产者checkout/cache/网络兜底。

新G3保原三平台/三币/危险/越界/相机/胜败重开和必要clip/jump-land-coin/HUD，真实Auto/Host/GPU/PCM/native/资源owner及新链保存关开/SaveAs/A→B冷再生/坏件修复再Run齐验；UI035夹具或AG014旧保存报告不代正式游戏。AG017保原G05九义务/三变体/全部GLB part/二维标记与021旧2d链完整单包，迁一入口净增0；交付后预计20/22，须实际重算。

## 5. 剩余里程碑和维护

本批只取039/016/017，不预取后续编号。之后剩G04两维教程新源迁移/真实编辑与OS准入，以及M2最终两游戏/Gallery统一公共引擎SDK版本、独立消费/资源离线发行/原M2-01..09逐项里程碑核验。游戏可玩/19-20项迁移不等M2完成，不以Plan数估百分比。

034/015同会话复核与持名问题如实保留；相关独立复审前置由后继T依赖门核，不改档造证据，不自动建agent。每计划每仓一权威树、同树不同上下文final、merge冻结与实际canonical/来源核验；只测触面，全AG里程碑档pending。临时材料仅plan-artifacts/plan-NNN/<repo>/，merge摘要折Plan/全输出对账后删自有scope/构建物，仅具名未解决问题保留。禁止链接/attachments/派生JSON、清树先fresh guard；不覆盖主题等其他路线WIP，不实施本批Plans。
