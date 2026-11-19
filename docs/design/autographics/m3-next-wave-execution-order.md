# AGE下一批：二维组合、国风首切片、正式光影与联合验收

2026-11-11，设计v2。以下为当前规划；历史M3文件名保留，不新增第三阶段。权威总口径见[AGE两阶段路线](age-three-milestones-and-ace-m2.md)。本轮只规划、不实施/构建/先证/素材/原生输入/实施树或发布。

## 已交输入

AGE M1由AG026完成11/11并签三仓v0.6-AGE-M1，DEBT-001/002已闭。UI055交付有限二维研究（无地面27credits，满铺阻挡和35>32为反例）；UI059交付方向光/阴影研究桥bounded-go，正式Light API未交；AG022交付国风提案及r187的621项盘点、完整同语义0。三项研究不重做，不把研究当产品go。

## 当前合同

| 合同 | 当前状态 | 职责与最小交接 |
|---|---|---|
| [UI058](../../plans/058-age-topdown2d-queries-and-sprites.md) | drafting r3，0/10 | 原Player+两障碍+双Target、query/sprite同Tick/原费用，加可见非阻挡地面；T-02中立机制先行，最终作者链等981所需A正式切片 |
| [AG023](../../../../ag-examples/docs/plans/023-age-m3-topdown2d-example.md) | drafting r3，0/10 | 教学小例与源码双端冷包，不扩完整种田；实现等UI058与981所需A正式来源 |
| [AG029](../../../../ag-examples/docs/plans/029-age-farming-first-playable-slice.md) | drafting r1，0/10 | 纯2D国风小院/两田种收/暂停/最小存档/原创许可和最终同源AV/A2R包；存档公共接口归973，不在内容复制引擎 |
| [UI080](../../plans/080-age-node3d-directional-light-and-shadow.md) | drafting r1，0/10 | 正式Node单方向光、硬阴影、六项typed公共能力/资源；复用059，非新研究 |
| [AG030](../../../../ag-examples/docs/plans/030-age-directional-light-content-and-oracles.md) | drafting r1，0/10 | 原创平面/立方体、六项独立Lambert/遮挡/ROI、双端冷包，partial映射不冒621整项 |
| [AG031](../../../../ag-examples/docs/plans/031-age-m2-joint-acceptance-and-signoff.md) | drafting r1，0/10 | E/G2/G3/F+真实OS+增强混合实际运行、独立review与落地签章，不重复各线实现 |

## 执行顺序与并行

未来明确启动后UI058/080 T-01→T-02可各自做中立有界决定；AG023/029/030/031 T-01/02只按合同准备输入/oracle/许可。GPU/窗口单实例，不把服务并行解释为并发抢焦点。981/SOURCE后SESSION与RUST可分别推进，979只证通用前门、980独立交传统UI，均不反等新增强。

UI058/080作者整链只等979与981/SOURCE/SESSION/RUST及所用LANGUAGE A切片正式来源，不等981整包B归档。UI058→AG023→AG029，UI080→AG030；AG029/030正式产品实现须各包前置及M2A-08。AG031接全部required与OS064/984 OS覆盖切片，985/A不等它；AGE不等ACE整个M2标签。AG020仍独立维护，物理根ag-examples不动。

## 范围与证据

国风纯2D首切片不是整款种田游戏，2DHD保升级选项；完整种植/库存/存档和实际包不能由AG023小例抵消。Light正式参数/单灯/2048²硬阴影与六项能力见UI080，不额外承诺PBR/多灯/软阴影/浏览器；AG030只做内容和独立oracle，不复制GPU服务。

原M1分母/锁/原生覆盖边界保持，候选改变按Spec失效规则重验；普通计划只跑受影响命名case，AG031在里程碑窗口统筹必要同版并集。新包/源码/素材均尚未制作，未执行/缺环境不算PASS。临时材料只在plan-artifacts/plan-NNN/<repo>/，本轮文档辅助材料在系统临时目录；每Plan每仓唯一树、禁链接、清理先guard。

## 历史说明

2026-11-05 v1中“055/AG022尚未交付、054/AG021在途、M1未收口、未来正式光影/国风合同待立”是当日快照，现由上述交付/合同替代。原研究/产品细节仍在其归档Plan与canonical，不重开或倒签；当前规划不授予实施许可。
