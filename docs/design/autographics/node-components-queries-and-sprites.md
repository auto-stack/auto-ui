# Node 组件局部性与下一轮二维能力

2026-11-01，设计 v1；仅规划。承接[全量迁移路线](gallery-node-authoring-migration.md)和已交付[019规范](../../specs/auto-graphics/node2d-basic-authoring.md)。本文件定义长期接口方向，不是执行证据。

## 两个主题，三份合同

| Plan | 责任 | 实际交付依赖 |
|---|---|---|
| [020 有状态Node组件](../../plans/020-autographics-stateful-node-components.md) | 基础二维入口中子Node的model/msg/on、实例receiver、属性绑定、Life调度和父子事件；角色逻辑可真正写在Player内部 | 019 |
| [021 二维查询](../../plans/021-autographics-node2d-queries.md) | G05/2d所需segment/sweep、具名节点引用和有寿命的typed结果、标记呈现、原数值/费用 | 020 |
| [022 精灵与帧动画](../../plans/022-autographics-node2d-sprites-and-frame-animation.md) | G09 atlas/character所需PNG/帧资源、frame/flip绑定、局部角色/动画状态与同tick呈现 | 020 |

三份均在auto-ui管理，状态以各自合同为准。AG-008已完成并合并8a01a34，归档/清理见[最终收据](../../../../ag-examples/docs/plans/archived/008-autographics-gallery-node2d-first-wave.md)；仍固定消费019，不追加020/021/022前置，也不追其开发tip。2026-11-01已按用户后续授权建立[AG009查询](../../../../ag-examples/docs/plans/009-autographics-gallery-node2d-queries.md)与[AG010精灵双入口](../../../../ag-examples/docs/plans/010-autographics-gallery-node2d-sprites-and-frame-animation.md)，分别硬等021/022实际交付；另[UI023→AG006 r2](node-audio-hud-and-gallery-followups.md)补声音/HUD Node接线，023只依赖020与已交付013，不追加020范围。均为规划，当前迁移仍6/21。

020先实际交付，021/022再以同一稳定组件合同实施。它们没有相互功能依赖，若各自新增lower/service模块且共享接线有owner短窗，可以并行；无法隔离共享文件则顺序落地。不能同树并发写，GPU/窗口/音频重型门仍机器单实例。与UI018构建配置的交集按实际diff核，不自动增设全量完成前置。

## 现场与要纠正的限制

019的基础二维profile只在根Node执行model/on，子组件限参数+纯view且拒绝model/msg/on；017的g11-2d已有部分实例状态/父事件，却是有限专用路径。普通use、AutoUI式局部性已经是作者目标，不能把基础二维的拒绝规则升级为永久语言规则。

020把已有机制收敛为可复用的组件表示/接线，不复制另一套parser/VM，也不按Player/Counter/示例名特判。声明树、render parent、状态owner、模块图独立；单个呈现目标不要求单文件、单模块或一Node一VM。

## 020的最小支持合同

- 固定的有限组件拓扑，允许无呈现逻辑Node、嵌套父子和同定义多实例。每实例独立model/handler接收方/事件端口，代码和不可变资源可共享。首切片≤8个有状态实例（含根，沿016实例验证包络），原无状态展开/128场景节点边界不缩减；不承诺八角色或动态solid。
- model/view-2d/on内聚；`.field`与`.Event`只指本实例，`refs.body`按本Node作用域解析。参数为类型化只读输入，初始化字段只在实例建立时求值，后续参数更新不能重置model。需要对外的actor/事件端口显式公开，不靠全局字段名串线。
- 图形资源/引用/静态绑定准备后，每新Run按声明先序执行一次Life.Start；只调用旧API在Start允许的服务。Tick每世界固定1/60，按先序在调用子Tick前求其输入属性，纯Auto业务消息同步投递到绑定receiver，Tick结束更新允许的呈现绑定。重入/递归不能形成无界宿主队列，受原执行守护约束；T-02冻结可支持面，不假称通用消息总线。
- 角色声明position是新Run初始化位置，运行位置由角色服务拥有，不每Tick用初值覆盖。普通视觉的动态position/visible可由局部model驱动，显隐不是销毁。if/动态Node拓扑/key重建不在此切片新增；017已有条件销毁保持原模式。
- 业务事件使用匿名msg和前导点，携带类型化值；不同父实例的同名子组件必须投递到各自父实例。系统Life.Start与自定义.Start不混；不为每Node重置32配额、VM时限或输入边沿。

最小证明不是再展示两个纯view：一个Player把vy/grounded/输入/走跳规则放在自己的model/on；另有两个同类型状态组件使用相同字段名但不同参数，驱动不同GPU区域，并分别给父实例发送带载荷事件。根同名字段为哨兵；Pause/Step/Resume/Stop/fresh Run和旧Run结果隔离要实测。

## 查询是服务能力，不是第二个物理系统

021只桥接已交付v3 segment与v4 sweep：finite f32、最近命中/精确同t按Run ordinal、闭区间/切线/零长/起始重叠、原误差观察容差、visible_only与同callback已完成显式变换写均保持。二维圆形图元的查询仍是AABB，不偷偷借用G11圆触发语义。

结果为带kind/Run/序号的typed句柄，沿旧的**世界共享latest-result槽**，不是每Node各一槽。第二次query/失败attempt/callback结束使旧token失效，setter本身不改已有快照；需要长期保存的是读出的值，不是假装结果token永久有效。cast8/sweep16/getter1和32池不变，不将每个属性访问变成免费宿主缓存。声明绑定同步与显式变换命令的可见时机须冻结，不重排有副作用的代码。

G05/2d的segment、sweep和reach20→10对照、原九类检查及14阶段/trace/保存重开义务逐项承接；本轮不实现Gallery文件迁移或3D查询。

## 精灵状态属于Node，图像仍按旧资源合同

022声明PNG、具名帧表和sprite视图；帧/翻转由局部Auto状态选择，不能用宿主时间或Rust动画器决定idle/run/jump/fall。Player可以自己持有vy/grounded/facing/phase，并在自己的view/on中绑定表现，无须把整份model交回根Node。

沿sprites2d/1：nearest/clamp/sRGB与linear alpha、像素矩形边界、flip只镜像U、不改变碰撞盒、同tick位置/帧/相机共同发布；不改变旧PNG编码准入和8身份/32MiB raw/CPU-GPU各64MiB/8 sprite绑定上限。每sprite写1，完整G09角色候选31、atlas30，必须实测所有分支而不是当可行性结论。

G09原两张PNG及CC0许可/制作来源固定：atlas96×32（三帧），character160×32（五姿态）。原18项oracle完整保留，包括slot销毁重生恢复默认帧、旧代拒绝、复用截图与错误空中姿态负控。必要的typed渲染实例/frame写端口沿现有8slot/Run/代次服务，不把渲染实例生成冒称通用动态Node子树。

## 交付与兼容

016/017/019原入口与生成字节保持历史兼容；新组件与服务模式显式选择，具体CLI/端口由各T-02有界整链冻结，不把未实现的语法草图写成Spec。各Plan提供受检输入闭包、依赖/工具/实际二进制身份与source-build消费契约；019记录当前只建立generator部分，runner/新资源身份必须实际闭合，不能自填hash或借旧绿报告。

2026-11-01核验AG008已合并并完成归档/清理（ag-examples c9c672c，交付8a01a34）；旧ag-008实施树/分支已由原owner清理，不再作为在途owner引用。连同AG007，迁移进度6/21：g11/2d及W1五入口，剩余15；本次仅同步簿记，不重建或接管其它实施树。所有新能力只跑实际触面及必要反例，OS抢焦点自动化继续暂停；临时材料只在plan-artifacts/plan-NNN/auto-ui，摘要折入Plan，合入清理自有scope。完整M2、ACE/AOT、CPC、3D、多呈现和Life.Frame不在本轮。
