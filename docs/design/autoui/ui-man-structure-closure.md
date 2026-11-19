# UI/Man结构收口：现有行为、职责与状态所有权

> target-state r1 / 2026-10-30；正式合同为本仓[PLAN-014](../../plans/014-ui-man-structure-closure.md)（drafting，0/11）。
> 用户本轮只授权提前规划边界、顺序和验收；不实施、不建树/仓、不运行构建/测试/GUI，不创建agent或联系其他会话，不修改他方执行合同。
> 现行行为以已交付Spec和固定版本为准，本文不发布新行为。来源是语言仓[Design37](../../../../auto-lang/docs/design/37-v06-modularization-and-repo-boundaries.md)、[873后结构范围](../../../../auto-lang/docs/plans/reports/post-873-structural-work-packages-2026-10-29.md)及当前源码，不另启动新架构或细拆crate。

## 1. 一份完整合同的职责

主改面是auto-ui，Man CLI在auto-man；语言仓维护路线及必要消费/规范指针。使用独立身份`auto-ui/PLAN-014`，不再给同一工作包分配auto-lang编号或Man并行合同。正式交付覆盖四个必做面：

| 面 | 既有生产入口 | 目标结构与不变量 |
|---|---|---|
| M1 CLI | Man `crates/auto/src/main.rs`、已有`cmd_*`及`tool_ui_ops.rs` | 解析/分派/诊断/工具编排各有归属；主入口组合既有模块。Core/full、flags/help、机器输出/退出码、工具发现、cwd/env、watch/取消与owned子进程保持 |
| U1 Vue | UI `crates/auto-ui-compiler/src/ui_gen/vue.rs` | template/script/style、导入与library/store等有主题模块；复用`import_policy`/`shared`。一份生成状态/registry，生成字节/资源和已支持模式保持，生成器复用无前次残留 |
| U2 Aura | UI `crates/auto-ui-runtime/src/ui/aura_view_builder.rs` | 节点族/绑定求值/构建与跟踪分层；同一VmBridge与绑定/槽/身份上下文，惰性分支与顺序、挂载/路由/调试ID、借用和作用域恢复保持 |
| U3 renderer | UI `crates/auto-ui-runtime/src/ui/iced/renderer.rs` | 先冻结消息/状态/订阅/active判据及资源owner，再整理更新、view投影、图标字形、部件与纹理/快照、宿主边界；保GPU资源和窗口/焦点/输入/帧时序 |

默认顺序M1→U1→U2→U3，均在一个Plan内。已有后继模块如在U890基线已完成某职责，逐符号验收并复用，不为“重构”再造重复层。必须证明所有四面职责已闭合；只移动测试、改文件名、套`include!`或新增转发壳不能代替生产职责收口。不规定任意行数上限、净删目标或速度指标。

## 2. 与890、UI-007的边界

| 合同 | 独占负责 | UI-014的接受/回交 |
|---|---|---|
| [auto-lang/890](../../../../auto-lang/docs/plans/890-auto-web-independent-repository-delivery.md) | Web仓身份、HTTP领域契约/运行/生成/政策、VM适配、服务客户端、独立发行与四仓迁移验收、旧Web原件退出 | U890后接受正式SDK/Web/Man/UI版本、端口/兼容/依赖/资源、逐ID测试和剩余限制。只整理CLI/UI的调用装配，不重搬或复制Web绑定/模板/算法，不回退旧auto-api-gen路径 |
| **auto-ui/014** | M1/U1/U2/U3既有实现的职责、状态/资源与测试归属，实际生产调用、兼容和消费闭环 | 向007交稳定入口/后继路径、状态及消息owner、测试ID/fixture定位和精确版本；不承诺新主题API已实现 |
| [auto-ui/007 r3](../../plans/007-autoui-tree-theme-and-examples.md) | 全部框架`examples/ui`归位、新主题树/引用/extend/mode、`.Design`准备生命周期、官方候选主题、Theme Lab、002/005首批主题改写及后继清单 | UI-014不做这些迁移或新语义；仅用现有源作只读回归或具名小型测试fixture。007沿新结构重绑实际符号/命令，不依赖旧行号，不因014交付就判主题支持完成 |

**正式实施顺序：U(auto-lang/890) → UI-014完整实施/独立终审/冻结合入/规范/归档guard → 重核结构出口后按UI-007合同确认及原门推进。** UI-014不等007，不引反向依赖；007已确认的主题/示例方向保留，不借结构工作抢跑其代码。原UI-007的“结构收口出口”在当前路线具体对应U014；UI架构其他候选不因此自动获实施授权。

现在可立约；UI-014所有建树、实施盘点/探针、fixture、构建和测试都等明确实施授权与U890。本轮收尾读回890已executing0/12，合同记录用户确认885收尾并关闭该历史证据启动缺口，原始guard日志未恢复；这不等于U890。UI-014接受其真实完整交付，不重造旧证据或替它实施。U888不作全局前置；真实脚本/注册/SDK/锁按已交付版本与单写者窗口交接。AG/ACE/OS及业务应用只按实际交集协调，不等无关全线，也不接管其实现。

## 3. 结构与行为验证

先冻结函数族→调用入口→单一状态/资源owner→消费者→测试/原子义务的映射。状态对象仍有唯一拥有者；主题模块接显式借用/上下文，不复制全局注册表、session/store、renderer cache或生命周期。既有`pub`/re-export、类型身份、feature/profile/线程栈/协议和支持面不变，不以`pub`私有内部或反向依赖消除模块访问问题。

Vue现有golden默认字节一致，包含已交付裸store方法路由；只允许预先具名的来源指纹/元数据字段因真实源码搬动变化，不能宽泛归一化、改golden或floor掩盖行为漂移。Aura追踪/非追踪、key/route/loop/slot与惰性边界逐条核对。renderer比较消息→状态/任务/订阅→真实窗口输入→输出与资源终态，不能只看静态截图或headless结果。

独立消费保语言/协议/类型单源、UI compiler不反依赖runtime、语言基础不反依赖UI/Man/Web实现、Man管理核心不拉VM/renderer。保U890新Web出口和支持等级，不改变I-PROJECT/I-TOOL/ABC/native ABI、默认feature或SDK公开承诺；若行为修复/协议变更成为必需，给具体输入与owner重规划，不静默并入结构任务。

测试沿867/887的逐ID及聚合子case守恒；重命名有映射，不降分母、弱化断言、复制裁判或以list证明执行。固定原基线合法/错误控制、实际工具进程、外部生成工程、Vue/VM交互、owned退出和恢复后重开均属Plan验收。UI-014只对实际影响面负责；已知红/skip/环境门控/未执行单列，新增回归不能转成历史债。

## 4. 有限退出与后继

M1/U1/U2/U3均有生产职责/调用与状态所有权证据、独立消费/DAG、测试守恒、真实双端触面与GPU/资源证据后，才可转最终review。final pass不等U；最终冻结门、各实际修改仓真实合入、各自Spec/账本、来源读回、归档与逐树guard完成才U014。旧M1/M2交付不重开，历史全应用债、80万、A1默认UI特性调整、更多crate、ACE/AG全线和未来主题全量不作为隐式出口。

work/review只跑定向反例、模块/fixture和必要check；最终review后merge按实际修改仓统一冻结门。完整批次/重型测试单实例且按现行到期规则，未改AG不跑全量AG套件。复用证据要核输入/依赖/配置未变；GPU/原生操作使用独立数据/端口及指定窗口，只清本次owned资源。原始证据仅本地，规范只记摘要/命令/退出码/路径+hash。

本轮仅完成目标设计，全部未来验收未执行。正式任务、AC/V/SD及有界交接入口以UI-014为唯一合同，不用本设计另建竞争清单。

2026-10-30静态预检通过：UI014的11任务/12AC/14V/6SD、完整AC承接/依赖/身份/授权边界、当前触面54个链接、4个稳定来源hash和两仓限定diff/空白/围栏。新结构规范未发布，独立final review与全部实施验证未执行；U890及UI014后续明确实施授权仍pending。
