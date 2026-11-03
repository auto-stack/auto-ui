# AG下一阶段：三维查询、完整G3与G05迁移

> 2026-11-03最新核验：UI035交付5903839e、UI039交付8f577a34；AG016/017仍drafting/r1/0-10无树。下一批UI040/AG018/AG019仅规划，见[编辑与同版发行收口](m2-editor-and-release-convergence.md)。下文为本批立约快照；同会话final限制、原预算与合同保持。

2026-11-03 v1，target-state。用户授权继续规划；三份新合同均drafting/r1、0/10，本轮不实施/建树/探针/复制素材/构建/GPU/OS自动化或创建agent。承接[M2总路线](m2-game-first-three-lane-roadmap.md)，公共作者模型仍为Node/use/model/msg/on、Life.Start/Tick及具名资源/事件，声明组织不强迫单文件运行或树形存储。

## 1. 现场与来源限制

- UI032完整二维组合a8c04d22、AG014完整2D游戏c141df8、AG013 G11/3d b48b304均已交付；无需等待全部Demo迁完才能立完整G3。
- UI034表现资源已交付93eaa907、AG015 G10两入口已交付80e0b65；按实际源码/生成物/metadata当前**19/22**，剩g04/2d、g04/3d、g05/3d。不要沿旧17/22快照判断。
- UI035唯一在途树`D:/autostack/.wt/ui-035/auto-ui`、分支`codex/plan-035-dev`，观察HEAD 65f5ce47（主checkout收据e6ca119f），合同r1任务T-01..03完成、T-04实质实施中、3/10，frontmatter仍drafting。以树/任务和最新收据为证；不改owner合同或拿候选components3d/1、31/32先证当交付。
- 034/015归档复审文本明确“实现同会话复核”，不等于独立上下文final。034 GPU共享store等具名hold按最新实际收据/适用路径核，旧T任务里的剩余措辞也不能独立作关闭/失败结论。不得修改历史SHA/归档造出独立pass。完整G3接线前核035实际独立final及相关034来源证据；缺证保留硬门，独立上下文复核的合法交接与相关hold处置由原owner/后续工作明确。AG015不是G3硬依赖，其无关内容复审不横加前置。

## 2. 本批三个合同与可并行阶段

| 责任线 | 合同 | 可独立准备/实现范围 | 完整接线硬门 |
|---|---|---|---|
| auto-ui引擎 | [UI039](../../plans/archived/039-autographics-node3d-queries-and-mixed-markers.md) | 已交付021/024/030之上，三维segment/sweep typed查询、动态group、有限3D＋2D诊断层；不等035 | T-02最多两机制全链先证，准确profile/refs/收费和旧档边界冻结后T-03实现 |
| ag-examples完整游戏 | [AG016](../../../../ag-examples/docs/plans/016-autographics-complete-3d-platformer.md) | T-01/02原G3八项/资源许可/模块草图/oracle/预算/包准备 | T-03等035实际交付/独立final/Spec/工具及相关034证据核；不等039/AG017 |
| ag-examples Demo | [AG017](../../../../ag-examples/docs/plans/017-autographics-gallery-node3d-queries.md) | T-01原G05三变体/九义务/GLB/2d冻结/包与政策准备 | T-02等039实际交付/独立final/Spec/工具；不等035/AG016 |

035继续由原owner实施，039可独立成为下一引擎能力；两者在parser/load/translate/assembly交叠处短窗单写并核合入影响。AG016游戏与AG017示例互不整项等待，固定各自上游delivery，不追开发tip。各Plan每仓唯一树、独立target/材料scope；共享gallery/packager/锁注册短窗协调，GPU/窗口焦点/音频重型门机器单实例。并行准备不等于全链依赖已满足，当前不自动启动任何计划。

## 3. 三维查询和有限组合的支持域

已读取原`g05-3d/scene.agscene`：3个多part GLB模型叶、3D门、2个2D quad、一个可移动3D父组；GLB42c9da67…/1528B。原脚本显式v3/v4，group/目标写后查询、二次查询覆盖latest slot、旧结果snapshot重读、二维marker展示node/t/point/normal。只有查询调用并不够：024 group尚只许静态，当前dialect parser同Node两个view出口也直接拒绝。UI039负责这些真实缺口，不让内容换圆柱/删标记或造假character/camera来绕过。

提案同根`view-3d`与`view-2d`仅在显式查询profile共同呈现、共一个model/msg/on，子Node各属一维，不能跨维parent/重复实例化/重复Life。它们是两个同时输出的呈现层，不是环境变体择一；2D仍原诊断投影，3D仍原camera/projection。T-02比较最多两种全链机制，公共parser的限制可移至profile准入或采取等价方案；旧profile原拒绝和冻结产物均保留。此片不承诺生产UI行布局里的任意交互3D视口、通用跨Node多出口或Context变体机制。

具名叶/组引用区分可读写行和可查询几何；group仅平移/显隐继承，没有shape。typed query结果按原latest snapshot，不作永久对象ID或免费缓存。数值/算法/错误/收费沿原v3/v4：finite f32/all-mesh AABB、闭区间、最近t/ordinal tie、X/Y/Z入射法线、signed-zero、初始overlap/零位移规则、不可用几何/非有限不能当miss；cast8/sweep16/getter1/AABB1共32，所有模块共池，旧API不变。查询是Auto逻辑，Rust只处理几何元数据/独立准入和渲染。

原G05/3d segment reach4/changed2、sweep speed10/clearance.002与请求无dt乘数保持；三变体/原九类require/所有部件像素和诊断点保真，既有OS阶段限制具名记录，不用runtime部分报告充OS14/14。AG009旧runner498ce814不可用是历史gap，新活3d039及活2d021链的包内真Run不可豁免。

## 4. 完整G3与原预算

正式内容见[AG016游戏设计](../../../../ag-examples/docs/design/g3-game-contract.md)，原G3-01..08/原坐标与输入来自auto-lang [样本矩阵](../../../../auto-lang/docs/design/autographics/m2-game-sample-matrix.md)。根＋Player＋三Coin＋Hazard＋Goal＋Boundary八状态Node；普通use与具名资源事件、局部model/msg/on，不靠业务字段名生成Rust行为。原三平台/三个不同币/危险/越界/缺币不能赢/胜败Enterfresh、相机相对运动与归一、接地单跳、原模型idle/walk/jump/朝向、jump/land/coin一次cue和n/3＋终局HUD同一程序齐验。

沿原32credits/物理写32/8slot/8pair/8状态Node/120base+8reserved128、deep/contact64/mesh512、raw8身份32MiB单件8MiB/CPU-GPU各64MiB、VM/source/time和音频/HUD/动画上限。035候选31/32只是上游先证，游戏仍实测完整最高分支；原地图不可达峰值独立容量夹具列明，不能移币/静音/停移动/拆玩法过预算。固定Kenney版本/原GLB纹理和四clip/原soundbank span/字体许可，不因同仓新游戏而升级AG014/其它Demo。

公开操作、修改Auto真实变化、本游戏新组合保存关开/SaveAs、完整离线A→B冷再生/实际胜败重开/坏件拒绝修复再Run必需。UI035永久夹具不是正式游戏，AG014旧profile无关保存报告也不是新G3保存豁免；实际GPU/PCM/native/HUD/资源owner证据与输入注入、OS操作、物理声学分别报告。

## 5. 后续终点与本轮边界

AG017交付后预计**20/22**（净增0），剩G04两维编辑器教程需另核作者源与真实选取/拖动/保存/重开/OS准入；不本轮取号。AG016交付后两款完整游戏内容才均有交付候选，仍需最终统一公共引擎/SDK版本、外部独立工程编辑发行及M2-01..09逐项里程碑核验；不按19/22或Plan数估M2完成比例。

新Plans带SD候选目标，不改canonical Specs；原引擎/内容/资产保持。每计划同树不同上下文独立final后merge冻结门；不因当前无agent授权省略独立复审。开工后材料仅`D:/autostack/plan-artifacts/plan-NNN/<repo>/`，merge摘要折Plan/所有输出对账后删除自有scope/构建物，具名未解决问题才保留。禁链接/attachments/派生JSON，清树先fresh guard，不接管035/主题等WIP。039/016/017各仓身份明确，取号不回退或借用外仓编号。
