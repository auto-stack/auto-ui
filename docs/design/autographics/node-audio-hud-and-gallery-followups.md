# Node声音/HUD能力与下一轮Gallery迁移

2026-11-01，设计v1；下文保留首次拆分时的历史审计/合同输入，当前状态见本次补记。承接[作者目标](declarative-authoring-model.md)、[组件/查询/精灵](node-components-queries-and-sprites.md)与[全量迁移路线](gallery-node-authoring-migration.md)。不修改当前引擎/示例/SDK或运行验证，不接管020/UI014在途树；本文件是设计依据，不是实施/独立复审证据。

## 本次补记：已交付能力与新规划

截至024/025规划核验，020组件16ddfa0、021查询1bec52f、023有限game32d7a81均已实际交付；022按owner记录复审/合入，AG006固定023与AG009固定021在途，实际进度看各owner记录。当前仍6/21。023真实支持固定game2d收集例，不等于全部通用Node音画接口；[新阶段设计](node3d-foundation-and-generic-presentation.md)明确[024 3D基础](../../plans/024-autographics-node3d-foundation.md)与[025通用声音/HUD](../../plans/025-autographics-node-presentation-services.md)各drafting0/10，保原预算和验收，不改AG006依赖/接管其实施。

## 首次规划时的现场与能力审计（历史快照）

只读基线：auto-ui695ce0b、ag-examples d88243c；UI012音频78632af、013 HUD8b1e799、019基础Node c93adb7已实际交付；AG008五入口8a01a34已交付并清理。当前Gallery仍12逻辑项/21入口，6/21已迁、15待迁，规划不计完成。020已有独立实施树和T-01记录，实际交付尚未发生。

| 所需能力 | 已有事实 | 本轮责任 |
|---|---|---|
| 普通use、局部model/msg/on、同定义实例/receiver | 017有限实例已验证，019子组件限纯view；020补共用状态组件 | 固定020实际交付，不接管或追加其实施范围 |
| G05二维segment/sweep、typed结果、spatial input/显式变换 | 原v3/v4底层已交付；021为Node接线草案 | AG009固定消费021迁G05/2d，保九类旧义务 |
| G09 PNG/帧/flip与typed渲染实例 | 004底层已交付；022为Node资源/服务草案 | AG010固定消费022迁atlas/character，保18旧oracle |
| bank/cue/ordinal/输入快照/一次声音 | 012底层audio2d已交付；Node load只有Quad/Circle/Color，CLI没有声音模式 | UI023补typed资源/服务，真实Auto决定声音 |
| 字体/四槽/ASCII表达式/完整HUD状态与一次呈现 | 013底层hud2d/scene8已交付；Node AST/translate/CLI没有接线 | UI023补局部文字/槽绑定与一次Auto收口，AG006 r2消费 |
| 收集/销毁/父事件/Restart与HUD最坏组合 | 017有限触发与020组件不等于audio/hud profile已经可组合 | UI023用原API证明必要typed触发/快照/原额整链，不由Gallery补引擎 |

审计源：pilot src/{load,translate,main,lower_g11}.rs、dialect node2d AST；driver api/public_{audio,audio2d,hud,hud2d}_v1.at和authoring/public_{audio,hud}/。CLI实际只legacy/g11-2d/scene-2d/character-2d/camera-2d，load::ResourceKind仅Quad/Circle/Color，字符串和声音/HUD资源/服务未降层。底层可运行不等于Node作者入口已支持。

## 分仓合同与依赖

| 仓/计划 | 范围 | 实施硬前置 | 首次规划时状态 |
|---|---|---|---|
| [UI023](../../plans/archived/023-autographics-node2d-audio-and-hud.md) | 二维Node bank/cue/font/HUD槽、必要ASCII、快照/typed触发与一次呈现收口；复用012/013 | 020实际交付、013已交付 | drafting r1，0/10 |
| [AG006 r2](../../../../ag-examples/docs/plans/006-autographics-gallery-event-hud.md) | 原G12b三收集/胜败/音效/HUD，新作者形式；原audio同项迁移/保八旧oracle/完整双入口包 | UI023实际交付、AG005/008已交付 | drafting r2，0/10 |
| [AG009](../../../../ag-examples/docs/plans/009-autographics-gallery-node2d-queries.md) | G05/2d segment/sweep/changed、九旧义务/14阶段与完整混合维度包 | UI021实际交付、AG008已交付 | drafting r1，0/10 |
| [AG010](../../../../ag-examples/docs/plans/010-autographics-gallery-node2d-sprites-and-frame-animation.md) | G09 atlas/character与main/alt、原图/局部状态/18旧oracle/完整包 | UI022实际交付、AG008已交付 | drafting r1，0/10 |

020交付后，021/022/023没有相互功能依赖；各自独立模块可并行，shared AST/load/translate/CLI接线必须owner短窗顺序进行，不能同树并发写。023当前只程序化图元＋无sprite的呈现(kind=-1)，G12原audio也无sprite/PNG，因此不强制等待022；将来需要sprite呈现写入另按真实022合同衔接，不能复制图像/帧实现。

内容分别等实际上游delivery后执行，不追开发tip；三内容计划无相互功能前置，registry/gallery.py/package和共享收据按owner短窗落地。真实GPU/音频/窗口重型门机器单实例，target/报告/consumer不共用，OS键鼠/焦点自动化继续暂停。AG008保持固定019和已归档事实，不追加新前置。

## 作者模型与保真要求

Player自己的model/on持输入/速度/grounded，Coin自己的状态控制一次收集/销毁，父节点处理关卡计数/结果，HUD节点持本地呈现；普通use共享资源/helper/组件。业务事件使用匿名msg与前导点，Life.Start/Life.Tick沿020合同。不得退回根全量model复制、手写app回调/整数ID业务、Rust计分/帧选择或根据截图补逻辑。

音频/HUD原上限沿012/013：同世界精确int ordinal严格递增、≤4声音请求/8voice，四槽完整状态/每槽32 ASCII、固定Lilita One及OFL/RFN，原bank/三cue来源/量化不变。多个Node请求经有界生成Auto在回调末一次收口，不每Node各bundle或独立发号；不引入无限host队列、免费getter或新全局游戏事务要求。声音接受后设备Fault不承诺撤回PCM。

旧G12a源的32注释漏列event_pair，而原边界oracle用独立grounded循环；它不证明新业务最坏路径。UI023/AG006必须先实测包括触发分派/所有getter/绑定/字符串/收口的完整Tick，原32池、CPU/VM/资源包络不改，不成立回规划。原图声阈值与硬件证据不可被banner/自报success或manager存在替代，native须本服务实际非零PCM。

AG006原AC-01..08/T-01..10保留，r1历史在Git d88243c；r2增加普通Node和旧audio迁移义务，原玩法/四槽/资源/预算/主alt/离线/坏件恢复不降。旧audio与G09文件/锁/回执保历史，新活源src与generated分离；新source-build只适用于明确新模式，不改旧exact-binary锁容纳任意exe。内容仓不写引擎，所有新接口和测试均尚未实现。

## 迁移计数与知识归属

AG009迁移现有一个入口，AG010迁移现有两个，均净增0。AG006迁移现有g12/audio并新增g12/hud一个；HUD原本不在21分母，不把新增混称旧入口迁移。每次实际合入再按现场更新原21项迁移账和当前库存；目前仍6/21，不能记作这些计划已完成。全部新计划均有单独SD，本轮不直接修改canonical Specs；新引擎Spec归auto-ui、新内容消费Spec归ag-examples，旧auto-lang知识只读引用，派生JSON不写。

独占取号分别锁本仓.next-id并复扫活动/archived/archive后创建，不预领其它波次；本次仅在UI新增024/025，当前号源026，外仓号源独立重核且本轮未取Gallery号。每Plan每仓唯一权威树、同树不同上下文终审；无新agent/chat。临时材料唯一plan-artifacts/plan-NNN/<repo>/，合同/证据摘要折Plan，merge对账全部路径后删自有scope/构建物，仅具名未解决问题可登记持有；禁止attachments含md和跨仓同号覆盖。
