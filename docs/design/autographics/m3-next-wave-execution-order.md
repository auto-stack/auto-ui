# AGE下一批：俯视二维组合、三维光照验证与内容消费

2026-11-05，设计v1；用户授权规划下一批并列全路线状态/顺序。本轮只规划，不接管054/AG021实现。当前逐项状态见[全部合同与执行顺序](plan-status-and-execution-order-2026-11-05.md)。

## 本批合同

| 合同 | 可交付范围 | 前置与先行边界 |
|---|---|---|
| [UI058俯视二维组合](../../plans/058-age-topdown2d-queries-and-sprites.md) | 正常Node/use下无重力四/八向、一sweep停墙、最近交互、精灵帧同Tick，原32credits包络 | T-01仅准备可先行；T-02等055/AG022实际结论与AG021对M2必需门真实收口；先证失败或需提预算就replan |
| [UI059方向光/阴影作者链研究](../../plans/059-age-node3d-lighting-and-shadow-feasibility.md) | 1方向光/1投影物/1受影面，Auto→GPU、独立Lambert/遮挡oracle、容量与go/no-go | 可独立研究，不等054/055/AG022/AG021；GPU/焦点错峰；研究不冒正式Light API或three.js已覆盖 |
| [AG023俯视组合小示例](../../../../ag-examples/docs/plans/023-age-m3-topdown2d-example.md) | 一个Player/两障碍/两个同定义Target，正常模块/具名帧、公开运行/改源/冷包 | T-01/02准备可先行；T-03正式接线等058实际交付/独立final/来源、AG022实交付及M2必需门齐；不追tip |

055和AG022尚未交付，本批不编造研究结果或先定完整国风游戏规模。058只承诺有限最小组合；没有自动滑墙、多角色、tilemap批绘或背包存档。059复用已有LightDef/MeshPipeline底层，源已有固定sample_light、ambient=0.25，不等于Auto作者已有Light。三维正式能力/Gallery映射在研究与AG022目录证据后再立合同；当前不领取未来编号。

AG023拟新增examples/m3/topdown-2d独立M3内容身份，不改M2的G01..G12/22入口47变体与两游戏、catalog/锁或验收分母。它是教学组合小例，不是完整种田游戏。AG022需求继续整理国风切片/素材许可，2DHD画面和完整玩法在后继合同冻结。

## 顺序与并行

M2主链：054实际交付/独立final/准确canonical → AG021 T-03..10及M2-01..10所有必需门 → M2结论；020在真实收口后的独立维护窗口更名，既不增加功能也不作为058/059前置。

准备/研究：055与AG022 T-01..05、059可并行；AG022 T-06需055实际结论。058 T-01和AG023 T-01/02可只做输入/源码意图/验收准备，不接线未交付API。正式M3二维链：055→AG022实际输入 ＋ M2真实收口 → 058先证/交付 → AG023。研究no-go不会自动允许改变预算/目标，先形成具名replan。

共享AST/load/translate/资源API/Host写面先核owner短窗；研究固定已交付源，target/报告/scope按仓计划独占；GPU/原生焦点/音频重门机器单实例，不把“可并行计划”解释为并发抢窗口。每Plan每仓一权威树、独立上下文final与merge冻结门，禁链接、清理先guard，临时材料只在plan-artifacts/plan-NNN/<repo>/，merge摘要折Plan后清自有scope。

## 当前现场与身份

054树已有1f3be333实现提交，主合同头部仍drafting/旧current_step，与T-01/02（或后续）执行记录有不同步；按真实实现中报告，不代owner修改头部。AG021原owner已记录executing/r2/2-10等待054。055/AG022仍草案且无实现树，不能用此前路线文案推断交付。UI056/057取号期间已由其它AutoUI路线占用（多文件reload/schema、Vue Store隔离），本批独占核主/归档/注册树后取058/059，counter060；AG023/counter024。外仓同号不是本仓ID，也不能共用材料owner。

M3不倒灌M2；M4正式AutoUI/ACE融合仍等Schema成熟且旧编译器替换实际启动。根目录仍ag-examples，020真实更名后保同仓历史号/交付身份更新当前路径，本批不迁仓/更名/创建agent/实施。
