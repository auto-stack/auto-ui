# 三维组件具名引用收口与G11/3d迁移

> 2026-11-03后续核验：030已交付804d7303、独立上下文final证据已补齐；AG013已交付b48b304，当前迁移17/22。最新UI034/035与AG015见[3D表现/音画/G10设计](node3d-presentation-audio-and-g10-migration.md)及[M2路线](m2-game-first-three-lane-roadmap.md)。下文保首次拆分历史，不作为当前状态。

2026-11-02，设计v1；仅规划、暂不实施。承接[作者目标](declarative-authoring-model.md)、[028/012设计](node3d-instances-triggers-and-gallery-motion.md)、[逐入口迁移路线](gallery-node-authoring-migration.md)。引擎/设计计划归auto-ui，内容计划归ag-examples。

## 当前交付与拆分

auto-ui基线01f99fdc，028实际交付ebac1cd，独立final/冻结/归档/清理齐，canonical node3d-instances-triggers-and-rounds.md SHA256 be4d9c152871c70c19d3524237521f81e3cde0d3da956325e2bf7cf3119e540d。ag-examples基线82b881e3，011实际交付f35236dc、canonical gallery-node3d-basics.md SHA676ceb1a5ed9bb747d69627698b8101319773972a81e3990818135275b37742c，已归档/清理。旧导航/收据中的11/22保历史，当前metadata/src/generated清单实际13/22（原21迁12＋新增HUD1），剩9；在途012不提前加三入口。

AG012由用户告知开发中；本轮初核合同/树尚未更新，随后已核见executing/r1、1/10、唯一ag-012树和owner T-01/固定f3dea0d消费记录；T-02待先证。开工仍核最新owner，不接管或重复建树。UI007主题路线另有在途树/WIP，本文不覆盖其源码、合同或未跟踪材料。

| 合同 | 范围 | 前置与并行 |
|---|---|---|
| [UI030](../../plans/archived/030-autographics-node3d-symbolic-component-bindings.md) | 在028之上补Slot/Template/Pair具名引用、跨Node只读ActorRef3D及局部组件接线；不改运行时/原预算 | 020/027/028均已实际交付，可与固定消费027的AG012并行 |
| [AG013](../../../../ag-examples/docs/plans/archived/013-autographics-gallery-g11-3d-node-migration.md) | G11/3d主alt与专项变体迁新作者源；保原25项/玩法/模板/完整G11双维包，净增0 | 028已交付；完整新组件作者形式硬等030实际交付，不把028有限夹具直接计为内容完成 |

两份均drafting r1、0/10，不创建实施树/探针/产品源码/资产、不构建/跑GPU或OS自动键鼠、不创建agent。AG012/013共享Gallery工具/注册/包按owner短窗串行；UI030与其他引擎路线的AST/load/translate改面开工核实际owner。target/报告/消费者独占，GPU/窗口/音频重型门同机单实例。

## 为什么还有030前置

028已真实跑通对象/触发/新局服务，其object3d是无model/msg/on的纯模板，spawn/bind用编译期解析的字符串名，bind subject只接本定义refs元素，event.pair()返回原数值序号。现g11-main/character.at把六句柄、巡逻、bag/result全放一个Character组件，并以p==0..5分派。它是有效有限交付，但不证明同一个Coin定义可用不同具名槽/模板/pair复用，更不证明跨Node角色引用已经可传入。

因此本轮不重开028，不把纯模板改称带状态组件，也不让内容写死数字pair或另造生成器。030用新显式作者模式补引用与组件接线；013依赖其实际交付。此依赖是落实既定可读性/局部状态目标，不是运行时服务再次重写。028字符串端口/数值ABI与原夹具继续冻结。

## 030支持边界

拟只读名义端口SlotRef3D、TemplateRef3D、PairRef3D、ActorRef3D（准确拼写T-02冻结）由根具名池、导入模板和静态Node实例路径解析；可作为普通组件参数传递，含同定义多实例。对外不暴露整数slot/pair/scene ID。symbol在下降时转原int/常量表，普通事件/状态仍由真实Auto执行；这不是引入完整语言类型系统/CPC或动态反射。

跨组件subject引用只绑定静态同Run三维leaf/角色body，不可引用group、错维/错kind或越作用域元素，不提供另一组件model的任意可写访问。实例引用/代次仍原Run对象句柄，本片不把它假装成完整静态借用系统。参数只读、初始化建立一次，重排声明重算身份映射而不串对象/接收器；动态Node拓扑、跨场景/多挂载仍非目标。

object3d继续纯静态模板，共享不可变mesh/许可；有状态Coin/Hazard/Goal等是普通逻辑Node，通过typed参数持各自实例句柄/model/msg/on，不要求每个逻辑Node另造一片可见几何。根只持关卡计数/结果与明确调度；Player持运动，Coin持一次收集/销毁，Hazard持巡逻，纯Ground/模板不浪费有状态配额。用一个根＋Player＋三同定义Coin＋Hazard＋Goal＋Void的八有状态节点组合先证，不静默提高八组件或八运行slot上限，两者是不同单位。

事件读取/分派的有限策略由作者显式组织。新增具名pair比较/查询不重新读取事件；真正的event.next/pair/phase/source仍各按原API收费，不能因“绑定”免费化。作者可在根Tick显式调用子msg推进Player，再消费上一Tick事件、向Coin/危险/目标投递，最后按原条件提交巡逻位置/检查fresh Restart；不能根Tick总先跑导致移动或重开延一拍，也不能同时Life.Tick和显式msg重复推进一次。

新模式沿Triggers3V1（108/profile19），不新造VM/对象表/触发几何/队列/round系统。保原32credits、move26/每对象调用与事件读取1、相机基读0，8slot/8pair、120base+8reserved=128、512潜在mesh、8有状态Node和VM/CPU/源码/资源额。原G11移动读一、idle至多三、有事件不提交巡逻位置是作者明示策略；不成为编译器全局规则。零额外服务读不等于无限VM成本，完整八组件链必须先实测。

## 013的内容合同

保G11原八行几何/三种Mob缩放.5/.6/.7/固定材质、编辑y=1而Start y=.5、六实例六pair、主speed2/hbase2.4/hdir.05与alt speed2.6/hbase.9/hdir.08（后续折返仍±.05）、三收集/危险/终点/相机越界/冻结/新鲜Enter重开。绑定、状态、事件引用改为030公开符号/typed参数，不手写序号、不在内容仓补引擎。模板与Coin逻辑组件分文件并复用普通use。

原G11/3d25项全部逐名走新源链，另补同定义Coin局部状态/两个父receiver、改名重排仍正确、完整预算与来源反例。旧G11/2d保持固定017的src/generated/八世界变体/锁/收据；完整G11包保两维及全部公开/专项作者入口，A→B真Run、SaveAs关开、仅闭包冷再生、坏件拒绝→精确恢复→同入口再Run。引擎夹具绿不代Gallery接受，也不顺手迁二维到030。

## 后续滚动方向

030/013不包含G05/3d。该旧例实际是三维多部件GLB＋动态parent group＋二维诊断标记；后继三维查询需明确有限2D/3D Node组合、group写和原v3/v4 query/结果寿命，不能换圆柱、删除标记或强加物理角色/相机。其引擎/内容后继等本轮边界稳定后单独立合同，不预取编号。再按实际证据规划G10纹理/动画与G04编辑器教程。

完整M2两游戏/整链预算/发行仍pending，跨端、生产UI/ACE/AOT/ECS/CPC不增加本轮前置。Specs本轮只提SD，review/merge后沉淀；不改旧收据/历史标签或派生JSON。未来材料唯一plan-artifacts/plan-NNN/<repo>/，开工登记owner/用途/在途，merge折摘要后对账全输出并删自有scope/构建物，仅具名未解决问题持有；禁止链接/attachments，清树先fresh guard。
