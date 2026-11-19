# AutoUI RenderQueue完善子路线

2026-11-12，**v13当前**。总调度见[拆分重构路线v56](../../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)，本次[收口状态复核](../../../../auto-lang/docs/plans/reports/split-refactor-closeout-status-audit-2026-11-12.md)核到1006/989均已正式归档；不再等待其实施。989已签有限支持域本路线/联合M1 ready=true，但原恢复合同仍有有限差额待处理。[1015 r1](../../../../auto-lang/docs/plans/archive/1015-split-refactor-closeout-supplement-and-contract-alignment.md)现已archived9/9、正式落地/清理齐；按归档用户移交边界本路线有限M1收口，013 AddTodo按本轮用户裁定列为M2 AutoUI修复项，不阻M1。本轮仅核查/状态同步，不实施。

## 1. 当前已交与支持范围

1003已交C生产默认/full恢复/身份/真实消费与收益，P1与R7关闭；OS068按用户裁定正式交付，不再等二者实施。当前里程碑声明按OS068 §0/§10.11及989 B-01～08精确限定，不沿旧文档宣称所有严格指标均实现。

| 合同 | 状态/正式来源 | 可消费范围 |
|---|---|---|
| [UI061](../../plans/archived/061-rq-fidelity-contract-and-feasibility.md) | archived r3，6/6 | 研究合同/模型/原型，不冒生产完整交付 |
| [UI062](../../plans/archived/062-rq-lossless-rendering-and-resource-parity.md) | archived r5，7/7 | 已交原语/资源/静态帧，具名残差/局部计时边界保留，不重开 |
| [UI063](../../plans/archived/063-rq-vm-rust-runtime-and-input-parity.md) | archived r4，7/7 | 通用运行、输入/IME/服务、事件/deadline/present与公平机制；已正式交付96236725及Man347c29f、canonical与清理；真实最终规模/present仍归OS063 |
| [UI073](../../plans/archived/073-rq-m1-media-fidelity-and-resource-closure.md) | archived r1；头7/8，最终8/8 | 连续媒体/RemoteFrame、精确绘制/SVG、资源重排负控、绘制/资源相关旧红；已交82b61084、canonical/归档/清理；review同实施上下文局限保留，最终规模/产品仍归OS063 |
| [OS063](../../../../auto-os/docs/plans/archive/063-rq-dual-host-assembly-and-parity.md) | archived r4；头3/7与最终收据有差额 | 全产品/两Host真实运行、M-02/06/07与全部P/M最终组合、消费方重建；有限交付6/8嵌入；Todo/Video coverage、测量终值及同实施上下文审查局限具名，M1不得自动缩分母；不转嫁给Linux062 |
| [UI074](../../plans/archived/074-rq-desktop-broker-attach-closure.md) | archived r1，5/5 | OS063 DG-1外部CLI attach修复已交5d8056f3、3/3 Active+compose；输入点击归OS063，os-063组树具名持有到其收尾 |
| [UI075](../../plans/archived/075-rq-broker-attach-budget-tiering.md) | archived r2，5/5 | attach预算旋钮已交，不等Todo/Video coverage已闭；有限边界与review局限纳最终核查 |
| [UI077](../../plans/archived/077-windows-rq-product-coverage-closure.md) | archived r1，7/7 | fa006603有限交付；Todo queue精确list-none覆盖、Video remote，旧queue视频能力仍拒绝；不冒全部业务完成 |
| [语言1003](../../../../auto-lang/docs/plans/archive/1003-generated-debug-policy-c-production.md) | archived r1；头7/10、末merge/清理已交 | lang11e1f1fd5/UI393ea17c，实际C/full/override/context/收据/delivery及净益证据；历史审查独立性限制保留 |
| [OS066](../../../../auto-os/docs/plans/archive/066-windows-rq-final-parity-and-performance.md) | archived r1，9/9；旧AC-02 partial | 仅其原有限输入，现由068正式结果/明示修约接续，不重开 |
| [OS068](../../../../auto-os/docs/plans/archive/068-windows-rq-remaining-fixes-and-full-acceptance.md) | archived r1，13/13，修约后final pass | OS3ea7ead/UI69104765/langf29f6facc，7Demo/解析/chat/N8等修复、8/8及有限任务/性能；支持修约边界见下 |
| [语言1006](../../../../auto-lang/docs/plans/archive/1006-web-generated-member-independent-delivery-and-recovery.md) | archived r1，11/11 | Web成员导出/仓外HTTP与状态/首版A-B恢复已交，P2关闭 |
| [语言989](../../../../auto-lang/docs/plans/archive/989-split-refactor-joint-delivery-and-tag-readiness.md) | archived r8，9/9 | 正式联合验收/Spec/清理已交，签两个ready=true；本轮原恢复合同差额见§3，未创建联合tag |
| [语言1015](../../../../auto-lang/docs/plans/archive/1015-split-refactor-closeout-supplement-and-contract-alignment.md) | archived r1，9/9 | Counter真交互与013勾选/过滤等UI20/20；AddTodo移交owner。RQ服务/数据18/18，窗口面身份复用，未新跑两Host窗口；主线8abc862f4与清理齐 |
| [OS062](../../../../auto-os/docs/plans/062-linux-native-rqhost-admission.md) | 关联Linux发行应用准入 | 非当前Windows M1/989硬前置，不代整条Linux发行版完成 |


## 2. 已批修约不扩大为未测能力

- 显示支持至60Hz，交付证据为ETW DWM呈现节奏；不宣称>60Hz或光学光子终点实测。
- M-04字节级计数未交，按零重绘/双槽机制面接受；P-03按direct与RQ不同呈现层的配对描述接受，不签未存在harness的replay p95旧门。
- M-01按12轮MCP协议面分布/业务响应接受，原生≤100ms与input-to-photon未证；不能将协议112ms改叫物理延迟。
- Video按demo030真实媒体加载/选片状态机/端点机制面修约接收，嵌入MCP/surface和seek原生通道边界保留；N8 video帧是空库待机，不冒真实播放。产品扫描/流服务back合同移交auto-video owner，真实实现/验收/正式交付才关闭该后继，不冒已交。
- 041 VM适用格、接口/保存旁路及故障证据与真实字符/IME证据分层；GPU设备/提交完成代理不冒独立GPU执行时间。未被明确修约的原required若缺证，989仍核适用性/独立断言并定向补证或具名阻断，不借本条自动豁免。

未修约保真/资源/隔离/错误拒绝等合同保持；功能分母按实际适用格与原源码/合同冻结，不以产品总数、首帧或attach代完整任务，更不补填未支持Rust格PASS。未来严格高刷/光子/字节/原生输入或完整Video能力需真实后继证据，不在本轮实施。

## 3. 1015有限交付与后继

F-989-CLOSE-01/02已按1015所记用户裁定的有限完成面关闭，生产/历史归档不重开。Counter A/B/恢复A真0→1→Reset、013已有勾选/过滤/计数；AddTodo静态生成的App.input/Store.input分离仍属AutoUI生产缺口，桥接方向探针不冒正式修复。owner需核源显式绑定与生成语义责任、认领有限修复后重验013 A/B/恢复A添加marker→列表+1→勾选，当前无本会话新建号/实施。

RQ服务/数据/故障恢复/旁观18/18是真实补验；standalone与desktop-RQ窗口面是068/989身份绑定复用，不宣称1015新跑两Host窗口。有限M1收口不等所有功能无遗留；若后续标签包含完整AddTodo需先正式交付并补对应断言。

阶段裁定（2026-11-12）：用户同意不影响M1时放M2修复。该AddTodo缺口按已接受边界不作本路线M1/联合M1标签前置；M1支持限制保留，M2再由AutoUI正式立约/修复/验收，具体编号未分配，不默认交ACE/AGE现有合同。联合1013等当前领域门仍独立有效，本轮不实施。

## 4. M1收口与联合交接

用户已接受013静态生成AddTodo移至M2，并确认进入本路线收尾；[收口记录](../../../../auto-lang/docs/plans/reports/split-refactor-m1-closeout-2026-11-12.md)已整理正式源/范围/实际验收/复用/限制、M2 owner与关闭条件。此为工作区文档收口，不是新增独立终审或已创建标签。

联合v0.6-M1仍需1013等当前ACE/AGE签章、到期批门与全仓可消费清单。1013未正式交付且检测到ACE测试活动，不沿旧暂停记录推断无推进；批门最新1014不含1015，待独占窗口跑，不能并发/代签。实际打标/推送由用户指令启动。

OS062仍属Linux支线，正式v0.6按版本现行ACE/AGE M2、Linux/App及专项门。当前不建新计划/施工、跑窗口/安装/恢复、停止其他任务、改canonical/归档、提交或发布。

<details>
<summary>v8子路线快照（1006/989交付前，已由v9取代）</summary>

# AutoUI RenderQueue完善子路线

2026-11-11，**v8当前**。总调度见[拆分重构路线v51](../../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)，有限联合验收合同为[989 r8](../../../../auto-lang/docs/plans/989-split-refactor-joint-delivery-and-tag-readiness.md)。本轮只立P2计划/恢复合同并同步989和路线，不实施或改canonical。

## 1. 当前已交与支持范围

1003已交C生产默认/full恢复/身份/真实消费与收益，P1与R7关闭；OS068按用户裁定正式交付，不再等二者实施。当前里程碑声明按OS068 §0/§10.11及989 B-01～08精确限定，不沿旧文档宣称所有严格指标均实现。

| 合同 | 状态/正式来源 | 可消费范围 |
|---|---|---|
| [UI061](../../plans/archived/061-rq-fidelity-contract-and-feasibility.md) | archived r3，6/6 | 研究合同/模型/原型，不冒生产完整交付 |
| [UI062](../../plans/archived/062-rq-lossless-rendering-and-resource-parity.md) | archived r5，7/7 | 已交原语/资源/静态帧，具名残差/局部计时边界保留，不重开 |
| [UI063](../../plans/archived/063-rq-vm-rust-runtime-and-input-parity.md) | archived r4，7/7 | 通用运行、输入/IME/服务、事件/deadline/present与公平机制；已正式交付96236725及Man347c29f、canonical与清理；真实最终规模/present仍归OS063 |
| [UI073](../../plans/archived/073-rq-m1-media-fidelity-and-resource-closure.md) | archived r1；头7/8，最终8/8 | 连续媒体/RemoteFrame、精确绘制/SVG、资源重排负控、绘制/资源相关旧红；已交82b61084、canonical/归档/清理；review同实施上下文局限保留，最终规模/产品仍归OS063 |
| [OS063](../../../../auto-os/docs/plans/archive/063-rq-dual-host-assembly-and-parity.md) | archived r4；头3/7与最终收据有差额 | 全产品/两Host真实运行、M-02/06/07与全部P/M最终组合、消费方重建；有限交付6/8嵌入；Todo/Video coverage、测量终值及同实施上下文审查局限具名，M1不得自动缩分母；不转嫁给Linux062 |
| [UI074](../../plans/archived/074-rq-desktop-broker-attach-closure.md) | archived r1，5/5 | OS063 DG-1外部CLI attach修复已交5d8056f3、3/3 Active+compose；输入点击归OS063，os-063组树具名持有到其收尾 |
| [UI075](../../plans/archived/075-rq-broker-attach-budget-tiering.md) | archived r2，5/5 | attach预算旋钮已交，不等Todo/Video coverage已闭；有限边界与review局限纳最终核查 |
| [UI077](../../plans/archived/077-windows-rq-product-coverage-closure.md) | archived r1，7/7 | fa006603有限交付；Todo queue精确list-none覆盖、Video remote，旧queue视频能力仍拒绝；不冒全部业务完成 |
| [语言1003](../../../../auto-lang/docs/plans/archive/1003-generated-debug-policy-c-production.md) | archived r1；头7/10、末merge/清理已交 | lang11e1f1fd5/UI393ea17c，实际C/full/override/context/收据/delivery及净益证据；历史审查独立性限制保留 |
| [OS066](../../../../auto-os/docs/plans/archive/066-windows-rq-final-parity-and-performance.md) | archived r1，9/9；旧AC-02 partial | 仅其原有限输入，现由068正式结果/明示修约接续，不重开 |
| [OS068](../../../../auto-os/docs/plans/archive/068-windows-rq-remaining-fixes-and-full-acceptance.md) | archived r1，13/13，修约后final pass | OS3ea7ead/UI69104765/langf29f6facc，7Demo/解析/chat/N8等修复、8/8及有限任务/性能；支持修约边界见下 |
| [语言989](../../../../auto-lang/docs/plans/989-split-refactor-joint-delivery-and-tag-readiness.md) | drafting r8，0/9 | 两上游正式源已接；C-01～05及候选适用性/包对未验，P2生产未交及C-05实际候选绑定未齐阻T-02，只准备标签 |
| [OS062](../../../../auto-os/docs/plans/062-linux-native-rqhost-admission.md) | 关联Linux发行应用准入 | 非当前Windows M1/989硬前置，不代整条Linux发行版完成 |

## 2. 已批修约不扩大为未测能力

- 显示支持至60Hz，交付证据为ETW DWM呈现节奏；不宣称>60Hz或光学光子终点实测。
- M-04字节级计数未交，按零重绘/双槽机制面接受；P-03按direct与RQ不同呈现层的配对描述接受，不签未存在harness的replay p95旧门。
- M-01按12轮MCP协议面分布/业务响应接受，原生≤100ms与input-to-photon未证；不能将协议112ms改叫物理延迟。
- Video按demo030真实媒体加载/选片状态机/端点机制面修约接收，嵌入MCP/surface和seek原生通道边界保留；N8 video帧是空库待机，不冒真实播放。产品扫描/流服务back合同移交auto-video owner，真实实现/验收/正式交付才关闭该后继，不冒已交。
- 041 VM适用格、接口/保存旁路及故障证据与真实字符/IME证据分层；GPU设备/提交完成代理不冒独立GPU执行时间。未被明确修约的原required若缺证，989仍核适用性/独立断言并定向补证或具名阻断，不借本条自动豁免。

未修约保真/资源/隔离/错误拒绝等合同保持；功能分母按实际适用格与原源码/合同冻结，不以产品总数、首帧或attach代完整任务，更不补填未支持Rust格PASS。未来严格高刷/光子/字节/原生输入或完整Video能力需真实后继证据，不在本轮实施。

## 3. 989准入与渠道责任

989未来授权后T-01可先准备；T-02按E-01～06核生产能力、来源/修约范围、有限包对/恢复定义及证据/红处置。它不要求提前跑完T-03～05，否则形成循环。

C-01/P1已由1003关闭，当前独立UI包/移根反控可按身份复用或定向验证。**C-02/P2仍OPEN**，Web SDK安装/服务生成/HTTP或chat links修复不代生成成员独立导出；领域主责auto-web、中立支撑auto-generation-core，必要Man薄编排，已由[中央1006 r1](../../../../auto-lang/docs/plans/1006-web-generated-member-independent-delivery-and-recovery.md)正式立约（Web主责），但尚未实施/交付。仅有owner或Plan号不放行，不能交989顺手实现，也不默认等整条ACE M2 Web迁移。

C-03已交支持域沿OS068，Video移交只限明确裁定子项；C-04沿各SDK/Man/Web正式结果逐形态核。**C-05四类对象已由[恢复合同r1](../../../../auto-lang/docs/plans/reports/split-refactor-recovery-objects.md)核定**，旧正式源/N选择规则、数据/备份/失败恢复oracle有定义；实际新源/命令/schema待1006交接及989 T-01协调owner绑定；配置full回退、冷启动/Todo冷重开不是整个升级恢复渠道的PASS。定义齐后才冻结候选、T-05实际执行；缺已有生产能力回owner合同。

1006 P2未交/恢复对象实际绑定未齐时T-02保持阻断；两上游归档不自动令本路线ready。只复用充分身份/断言证据，缺证/失效/新耦合定向补，不重跑全部领域矩阵或复活已删scope。

## 4. 里程碑与行动边界

ACE1000、AGE AG026均已归档，正式签章/清理和不同SDK的共同组合由989据真实收据核；本路线与联合v0.6-M1分别判定，渠道硬门未过不自动变绿。正式v0.6还等ACE M2、AGE M2、Linux发行版与App强化，均非当前T-02前置；OS062只交应用准入，旧“未来M4”不能延期现行M2门。

本轮只规划/文档，不启动窗口/构建/设备/安装/清理/打标/其他会话；canonical/归档不改，旧审查限制不倒签独立。后续实施材料一Plan一scope按仓分、同一权威树顺序独立复审，现行guard/持有与delete-at-merge规则不变。


</details>

<details>
<summary>v6调度历史（v7已取代当前状态和修约门）</summary>

# AutoUI RenderQueue完善子路线

2026-11-10，**v6当前**。纳入[拆分重构总路线v49](../../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)，本轮只立正式计划/同步文档，不实施。目标设计：[完整保真r5](renderqueue-complete-fidelity.md)；现状以UI runtime Specs和OS066 canonical有限实证为准，不提前把新计划写成已实现。

## 1. 当前出口与正式合同

Windows RQ-W原目标保持：VM/Rust×direct/独立RQHost/桌面RQHost六格，完整能力、所有真实Demo与manifest active产品、真实输入/IME/业务/资源/媒体/生命周期与direct一致；App无自GPU/隐藏direct回退，Host用iced/wgpu统一replay。全产品功能分母不能被1/4/8性能样本替代。

| 合同 | 当前状态 | 唯一责任与正式门 |
|---|---|---|
| [UI061](../../plans/archived/061-rq-fidelity-contract-and-feasibility.md) | archived r3，6/6 | 研究合同/模型/原型，不冒生产完整交付 |
| [UI062](../../plans/archived/062-rq-lossless-rendering-and-resource-parity.md) | archived r5，7/7 | 已交原语/资源/静态帧，具名残差/局部计时边界保留，不重开 |
| [UI063](../../plans/archived/063-rq-vm-rust-runtime-and-input-parity.md) | archived r4，7/7 | 通用运行、输入/IME/服务、事件/deadline/present与公平机制；已正式交付96236725及Man347c29f、canonical与清理；真实最终规模/present仍归OS063 |
| [UI073](../../plans/archived/073-rq-m1-media-fidelity-and-resource-closure.md) | archived r1；头7/8，最终8/8 | 连续媒体/RemoteFrame、精确绘制/SVG、资源重排负控、绘制/资源相关旧红；已交82b61084、canonical/归档/清理；review同实施上下文局限保留，最终规模/产品仍归OS063 |
| [OS063](../../../../auto-os/docs/plans/archive/063-rq-dual-host-assembly-and-parity.md) | archived r4；头3/7与最终收据有差额 | 全产品/两Host真实运行、M-02/06/07与全部P/M最终组合、消费方重建；有限交付6/8嵌入；Todo/Video coverage、测量终值及同实施上下文审查局限具名，M1不得自动缩分母；不转嫁给Linux062 |
| [UI074](../../plans/archived/074-rq-desktop-broker-attach-closure.md) | archived r1，5/5 | OS063 DG-1外部CLI attach修复已交5d8056f3、3/3 Active+compose；输入点击归OS063，os-063组树具名持有到其收尾 |
| [UI075](../../plans/archived/075-rq-broker-attach-budget-tiering.md) | archived r2，5/5 | attach预算旋钮已交，不等Todo/Video coverage已闭；有限边界与review局限纳最终核查 |
| [UI077](../../plans/archived/077-windows-rq-product-coverage-closure.md) | archived r1，7/7 | fa006603有限交付；Todo queue精确list-none覆盖、Video remote，旧queue视频能力仍拒绝；不冒全部业务完成 |
| [OS066](../../../../auto-os/docs/plans/archive/066-windows-rq-final-parity-and-performance.md) | archived r1，9/9；AC-02 partial | 机制28/28、媒体两Host、局部attach与debug N8=5/8等有限证据；7Demo/041/chat/N8三项/全业务/release/真实物理-GPU-高刷未闭 |
| [语言1001](../../../../auto-lang/docs/plans/archive/1001-incremental-cache-lifecycle-and-generated-debug-policy-validation.md) | archived r1；末收据已交 | 缓存治理+研究，用户采用C但产品默认未推广，旧scope已删 |
| [语言1003](../../../../auto-lang/docs/plans/1003-generated-debug-policy-c-production.md) | drafting r1，0/10 | C生产默认/有效身份/用户覆盖与full-debug回退、四入口/SDK/实际收益/迁移残留；独立，不等RQ全量 |
| [OS068](../../../../auto-os/docs/plans/068-windows-rq-remaining-fixes-and-full-acceptance.md) | drafting r1，0/13 | 066剩余R-01..07有限修复、全Demo/产品六格/业务、release P/M/物理上屏/GPU/高刷原门；准备/修复可先，最终T-06以后等1003正式C；required缺测/partial不能归档冒完成 |
| [语言989](../../../../auto-lang/docs/plans/989-split-refactor-joint-delivery-and-tag-readiness.md) | drafting r6，0/9 | 两包与治理实际目标/必需渠道正式就绪后冻结，复用有效证据只补新耦合/失效；两类M1 ready与三路收据清单，不代生产修复/全领域测试 |
| [OS062](../../../../auto-os/docs/plans/062-linux-native-rqhost-admission.md) | drafting r2，0/13；关联Linux支线 | NixOS+COSMIC两应用准入，不阻Windows M1或989；不代表Linux发行版全路线完成 |

## 2. 当前实施顺序与去重

1003独立；OS068 T-01/02根因与T-03方案/原direct预算/设备资格可先，T-04/05有限修复可并行；T-06最终source/tool/SDK/profile冻结重建及T-07～09全六格/性能必须等1003正式交付。再与渠道结果汇入989 T-02。1003不反等068/989，两个M2不倒灌当前Windows前置。

OS068真实修复限已承诺RQ范围，重大公共架构缺口须正式owner合同/依赖，保持本包required未过，不以“已具名后继”结题。UI068 Reader/OS067 Notes FeaturePack/AGE UI078独立；仅按实际共享接口/源码触面消费正式结果、保持单写，不接管WIP或等待无关整包。真实产品在独立clone开发，OS组合pin只消费已交来源。

## 3. 不可缩减的功能与性能门

连续媒体、文字/mesh/SVG/阴影/布局/资源乱序与输入/IME/焦点/捕获/DPI必须沿原能力ID逐项可证；旧062局部2%/p50门、静态poster、旧attach日志不能自动抵最终完全一致。OS068保原direct独立oracle，候选direct同样验证，防共用错误互证。

P-01异步批量/有界提交，P-02驻留复用/局部更新，P-03真实ready/deadline/ACK与present分离，P-04 session/surface隔离与过载；M-01～08按原样本/预算/统计量/同环境direct和App+Host总成本，1/4/8必须完整adopt和业务/进展，不以5/8或饿死重App签公平。release终值与debug历史分列。

真实present/GPU完成/物理上屏/日志到达时刻分别定义，跨进程时钟与event/frame关联校验；GPU计时/物理终点/原高刷适用面缺设备或样本继续未完成，不能以CPU代理/60Hz自动豁免。预算先冻结后测，污染/失败保留，未知不填0，不能测后改门。

## 4. 交付与未来边界

OS068全部R/AC/required真实通过、独立上下文终审、各仓代码/产品pin/Spec正式落地与清理后交989；1003也须实际生产默认/收益/迁移交付。989不无条件重跑所有领域矩阵，只核复用/失效及新耦合，必需渠道C-01～05另按真实owner闭合，完整backend不替Web独立导出。

本路线ready与联合v0.6-M1分判：联合还需1000的ACE M1与AG026的AGE M1正式收据/兼容跨仓清单。正式v0.6另等ACE M2、AGE M2、Linux发行版全路线及版本验收；OS062不是Windows前置，自己的Wayland compositor仍按其批准平台路线。旧“未来M4”文字不延后现行M2替换门。

材料仅各Plan唯一scope按仓区分，review同树独立上下文顺序，merge折摘要后删除自有材料/树，guard先行，不动用户持有或未知WIP。本轮没有构建/GUI/计量/清理/打标/代理。


</details>

<details>
<summary>v5当前段历史快照（v6已取代其调度/状态）</summary>

# AutoUI RenderQueue完善子路线

2026-11-10，v5；本轮只修订规划/立约，不实施。纳入[总路线](../../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)。目标设计见[完整保真](renderqueue-complete-fidelity.md)，当前行为以[保真合同](../../specs/runtime/rq-fidelity-contract.md)及[062资源/绘制Spec](../../specs/runtime/rq-render-packets-and-resources.md)为准；历史“全部r3草案/没有Spec”不作为调度。

## 1. 当前合同与M1定义

本M1是本拆分重构路线的Windows RQ-W与实际生成治理出口，不是AGE M1/ACE M2/v0.6正式发布。VM/Rust×direct/standalone-RQ/desktop-RQ全公共能力/全部Demo/manifest产品的原目标保持；原生装饰与客户区分别验。App无GPU/自窗/全UI截图回退，iced/wgpu Host负责最终渲染。

| 合同 | 当前状态 | 唯一责任与正式门 |
|---|---|---|
| [UI061](../../plans/archived/061-rq-fidelity-contract-and-feasibility.md) | archived r3，6/6 | 研究合同/模型/原型，不冒生产完整交付 |
| [UI062](../../plans/archived/062-rq-lossless-rendering-and-resource-parity.md) | archived r5，7/7 | 已交原语/资源/静态帧，具名残差/局部计时边界保留，不重开 |
| [UI063](../../plans/archived/063-rq-vm-rust-runtime-and-input-parity.md) | archived r4，7/7 | 通用运行、输入/IME/服务、事件/deadline/present与公平机制；已正式交付96236725及Man347c29f、canonical与清理；真实最终规模/present仍归OS063 |
| [UI073](../../plans/archived/073-rq-m1-media-fidelity-and-resource-closure.md) | archived r1；头7/8，最终8/8 | 连续媒体/RemoteFrame、精确绘制/SVG、资源重排负控、绘制/资源相关旧红；已交82b61084、canonical/归档/清理；review同实施上下文局限保留，最终规模/产品仍归OS063 |
| [OS063](../../../../auto-os/docs/plans/archive/063-rq-dual-host-assembly-and-parity.md) | archived r4；头3/7与最终收据有差额 | 全产品/两Host真实运行、M-02/06/07与全部P/M最终组合、消费方重建；有限交付6/8嵌入；Todo/Video coverage、测量终值及同实施上下文审查局限具名，M1不得自动缩分母；不转嫁给Linux062 |
| [UI074](../../plans/archived/074-rq-desktop-broker-attach-closure.md) | archived r1，5/5 | OS063 DG-1外部CLI attach修复已交5d8056f3、3/3 Active+compose；输入点击归OS063，os-063组树具名持有到其收尾 |
| [UI075](../../plans/archived/075-rq-broker-attach-budget-tiering.md) | archived r2，5/5 | attach预算旋钮已交，不等Todo/Video coverage已闭；有限边界与review局限纳最终核查 |
| [UI077](../../plans/077-windows-rq-product-coverage-closure.md) | drafting r1，0/7 | 先分辨真实remote/legacy选路和coverage拒绝，修Todo/Video实际投影/入口；不白名单绕过，交最小真实两Host/执行形态 |
| [OS066](../../../../auto-os/docs/plans/066-windows-rq-final-parity-and-performance.md) | drafting r1，0/9 | 全Demo（auto-edit）/全产品六格、真实画面任务/P-M/重建闭包最终验收；清单可先，T-02起等077及必要观测/预算决定 |
| [语言989](../../../../auto-lang/docs/plans/989-split-refactor-joint-delivery-and-tag-readiness.md) | drafting r4，0/9 | 最终治理+RQ联合核查，等077/066及治理/渠道真实交付；只准备tag |
| [OS062](../../../../auto-os/docs/plans/062-linux-native-rqhost-admission.md) | drafting r2，0/13；Linux发行支线 | Linux发行版M1→M2应用准入；先另行尝试NixOS+COSMIC环境，WSL可选开发。不是拆分重构Windows M1/989/tag-ready硬前置；只对实际共用改动协调回归 |

主链旧Windows合同已有限交付 → UI077修复 → OS066完整最终验收 → 与治理/渠道结果汇合 → 989总收尾；OS063盘点/部分装配可按上述门提前，无UI063/073↔OS063反向等待。UI073/074/075及OS063已归档，完整M1差额仍待关闭；本轮仅读回/同步状态，不启动或接管其他计划。

## 2. 062遗留如何关闭

| 项目 | 实施/验收归属 | M1规则 |
|---|---|---|
| 四个既有红 | UI063核坐标/输入/调度，UI073核绘制/资源；OS063逐产品判当前影响 | autocenter_translates_hits_with_ops、stage3_multi_app_stress_n3/n5、t3_examples_queue_end_to_end各绑输入/断言/候选/owner；命中必需面必须闭，不以基线红免验 |
| 连续视频/RemoteFrame | UI073全链实现及最小六格，OS063最终产品消费 | 静态poster不抵连续帧；8MiB/s与高码率源矛盾须探针/明确决定，不暗改预算/画质 |
| 默认文字重排/13族非exact/阴影/SVG | UI073逐ID精确收口 | 062的2%/p95只记录不自动成为完整路线门；真正平台非确定AA仅允许原目标内经审查的窄区域容差 |
| 资源乱序注入负控 | UI073可控重排、实际Host consumer、双向引用/释放/旧generation | generation只覆盖上传方向，缺源/引用/释放交错另证；placeholder不算完整帧PASS |
| 真实M-02/06/07与尾延迟 | UI063机制，OS063最终双Host、1/4/8规模 | 真实present/截止/启动/公平按先冻结后测的预算，提交代理不得冒上屏；没有预算/样本不PASS |
| isolated标签/未重建/删原证据 | 各执行Plan核实际负载、重建hash/PID；989终核摘要有效性 | 标签不隔离机器；保失败/污染/完整分布。摘要充分可复用，不以附件已删自动全重跑或凭标题计数PASS |

OS063 T-01逐产品/能力给出原始义务→当前证据→缺口/关闭合同，未知/未执行不缩分母。必要业务或重大编译变更回实际owner有限合同；新Plan号本身不能关闭M1。UI073若需进一步拆分先语义修订挂正式依赖，不自动降低目标。062归档保持终态。

## 3. 平台、版本与工作纪律

Standalone/desktop共用资源/replay，UI063交surface身份、placement/z/clip、坐标逆变换、focus/capture、resize/DPI/readiness/present接口；OS063仅策略装配。UI073变更先正式落地再供OS消费，旧新端/能力拒绝与保旧帧、resource generation/release仍须回归；禁止靠同名类/编译绿判兼容。

Linux普通Host沿OS062真实IPC/映射/claim/IME/Nix，扩大完整六格另约；自己的Wayland compositor仍未来独立里程碑。ACE/AGE由语言978/980/981等现行接管合同负责；ACE M2是v0.6正式门，不能沿旧“未来M4”文字延期，也不把整条ACE设本路线M1前置。若候选先换引擎，按实际来源修订/核证，不继承旧VM PASS。

一仓一权威worktree，同树顺序独立复审；主检出只计划簿记。无junction/symlink；材料仅plan-artifacts/plan-NNN按仓区分，merge折摘要入Plan后删scope，移树前wt-guard clean。不自动开代理/聊天、不运行项目门，不碰未知WIP。所有性能保App+Host总成本、同direct对照、源/配置/输入身份及完整样本，未测不外推。

## 4. 本轮修订边界

合同修订仍为UI063/OS063 r4、989 r3、UI073 r1；2026-11-09仅刷新状态：UI073已交（头7/8、最终8/8），074已交5/5，OS063执行3/7、989仍草案；073审查上下文局限保留。原任务与AC保持、增加承接V，062不重开、canonical不提前改写；本节是当前调度，以下v3只历史。静态结构/链接/依赖检查不冒独立review或运行PASS。


2026-11-10状态/归属同步：OS062修订r2，仅属Linux发行版应用准入，环境尚未尝试。Windows M1不能因OS062环境缺失被整项阻塞，也不能因OS063归档省略其Todo/Video及测量holds；本轮不改已归档合同/canonical。


## 5. v5新增Windows完整收口（2026-11-10，仅规划）

OS063归档不重开。UI077先区分实际remote/legacy支路，保未知能力拒绝；OS066核完整分母/图像结构业务/真实时序与预算，不能以6/8或1秒日志签全过。989 r4 T-02必须等077/066正式结果、实际治理与渠道闭包；仅T-01准备可先。OS062独立Linux发行支线，不在默认Windows M1前置。本轮新合同全部未实施。


</details>

<details>
<summary>v3历史设计与启动记录（不作当前调度）</summary>

# AutoUI RenderQueue完善子路线

2026-11-05，v3；纳入[拆分与重构总路线](../../../../auto-lang/docs/plans/reports/v06-extraction-execution-roadmap.md)。本轮只调研、设计、立正式草案；没有实施授权。主设计：[完整保真](renderqueue-complete-fidelity.md)；平台设计：[AutoOS三类Host](../../../../auto-os/docs/design/04-rqhost-platform-and-compositor.md)。

## 1. 要交付什么

当前阶段：所有现有AutoUI公共能力通过VM和Rust两种执行形式，在direct、独立RQHost、虚拟桌面RQHost六格中保持显示、输入、业务运行与生命周期一致。RQ App不自行栅格化或建GPU/window。Windows为当前实现验证基面，Linux普通宿主复用OS062真实平台准入；自有Wayland compositor和ACE/AGE替换是后续有进入条件的里程碑，不拿远期研究阻塞当前完善。

完整功能由真实组件/样式/资源/运行能力清单定义，不等于多跑几个Demo。独立窗、桌面子窗、平台条件、已有direct缺陷和未执行项目分别记录。旧683/690/693/694是有限交付，不重开，也不能用其历史通过关闭本次全覆盖目标。

## 2. 正式工作包

| 阶段/合同 | 交付结果 | 前置与单写面 |
|---|---|---|
| [UI061 保真合同与可行性](../../plans/061-rq-fidelity-contract-and-feasibility.md) | 冻结全能力分母、同源六格oracle、两种录制候选的最小高风险原语验证、wire/资源/版本决策 | 可先独立研究；只使用正式来源，协议/renderer在途修改先核；研究不是生产全功能 |
| [UI062 无损绘制与资源](../../plans/062-rq-lossless-rendering-and-resource-parity.md) | lossless producer/replay、文字/mesh/SVG/图片/特效/媒体与资源事务；direct旧行为保留，两个Host消费同一核心 | 等UI061可行结论；headless/message/endpoint/broker_surface单写，新增未能容纳的架构需具体修订 |
| [UI063 VM/Rust运行与输入](../../plans/063-rq-vm-rust-runtime-and-input-parity.md) | 公开RQ入口统一remote语义、完整输入/IME/服务/init/异步/退出、两个执行形态无App自渲染 | 只读盘点可先行；生产改动等UI061/062/060正式交付；UI053已交付；Man仅薄接线/运行闭包 |
| [OS063 双Host装配验收](../../../../auto-os/docs/plans/063-rq-dual-host-assembly-and-parity.md) | standalone/desktop真实六格、现有Demo全覆盖和28 active产品逐项清单、产品分批重放/失败归属、资源/生命周期与发行闭包 | 清单可先盘点，冻结等UI061；装配等UI062/063与OS054正式交付；不改原AC/借WIP |

主链：UI061 → UI062 → UI063 → OS063；UI060正式交付另汇入UI063，OS054正式组合另汇入OS063。UI063/OS063可提前只读盘点，生产实施按下表进入门。所有实施均待用户后续启动；四份合同现均drafting r3，范围/依赖与接口交接已复核，不启动实施。UI061若证实候选不可行，交具名needs_replan方案，不把no-go说成RQ完整交付，也不让UI062猜实现。

OS063对28产品的每一项必须有真实能力/入口/来源核对；本来尚无VM或Rust有效产物的产品记具名上游缺口和关闭合同，保持路线未完成，不能静默按“不支持”排除。合同内框架/装配问题由本路线修复，产品业务新增/编译器重大变化另立owner计划，不无限塞入一个包。

## 3. 与现有计划的关系

- UI060在途范围包括索引、输入派发/帧刷新和真实native API；本路线接受其交付，不复制修复，不给它追加所有保真债。UI053已归档6/6，UI 9a94d568 / lang 61207576b正式交付且清理完成，可核固定版本消费；其Vue验收不抵RQ。
- OS054验证八产品既定组合；OS063接正式组合做RQ全面扩面。不得反向要求054先过新全能力门才允许自身有限交付。
- OS055正式主题/协议能力须保；OS061产品主题已正式交付9/9（OS ddfe5a5及最终清理收据），不再列在途。UI056/057不是当前RQ的全局前置；只有实际共用store/context/生命周期接口需要版本协调。
- OS062已有Linux真IPC/共享映射、Counter/Notes Rust independent/rq与Nix安装合同；本路线不重复实现UDS/mmap或重编号。Windows保真修复可供062消费；双方真实共享文件短窗单写，Linux特定门以其合同为准。
- ACE/AGE M2/M3独立继续；当前RQ不等待未来M4，自有compositor与M4不能靠本轮立约宣称已完成。

## 4. 后续里程碑与进入条件

| 里程碑 | 启动条件 | 需要独立合同的交付 |
|---|---|---|
| RQ-W Windows完整保真 | 四份草案逐项授权、正式依赖可用 | 六格全能力/产品义务闭合、无静默近似/降级、独立review/合入/规范/清理 |
| RQ-L Linux普通窗口Host | OS062真实准入+RQ-W可消费核心，明确实际支持系统 | 在Linux补VM/desktop与全能力，而非只承诺两Rust应用；按062结果立扩面Plan |
| RQ-C 自有Linux compositor | 真Linux通道/资源、可靠RQ核心、独立Linux会话与nested可行性、OS路线窗口 | nested先证→DRM/KMS与seat/output→标准Wayland客户端兼容→会话/安装/回退；当前只设计，不新增实现Plan |
| RQ-E ACE/AGE迁移 | ACE可消费Schema/执行接口与替换工作启动、AGE M4正式进入门 | 执行与渲染分轴对照，兼容/资源/输入/回退，逐产品迁移；未来再立合同 |

完整当前RQ与远期Linux compositor是分别验收的出口；两者都保留在战略中，不用未来能力无限拖延当前交付，也不把本轮Windows绿外推到未来。

## 5. 性能路线与交付门（r2新增）

设计单源为[完整设计§7.1～7.3](renderqueue-complete-fidelity.md)。路线目标是异步批量显示列表、持久资源、集中GPU执行；避免同步往返和重复处理，不以“画图元”判定快慢。共享内存不自动等于零拷贝，集中GPU也不保证比direct快；现15ms帧泵只作静态风险，未测不报性能数字，不外推X11速度结论。

| 合同 | 性能职责/退出条件 |
|---|---|
| UI061 r3 | 先冻结M-01～08测量定义/负载/样本，测原direct，再在候选比较前冻结绝对和相对预算；以原型验证P-01～04可行与负控敏感性。研究通过不等生产性能已过 |
| UI062 r3 | P-01批量异步/槽信用及P-02持久资源；P-03有界队列/背压、P-04上传/驻留/CPU批工作隔离；消除重复文字处理和有损重放；真实双进程符合冻结预算 |
| UI063 r3 | P-03 readiness/输入/deadline/present驱动，固定轮询不能唯一驱动；P-04交互/后台公平，服务等待不串行阻塞绘图，输入/业务不丢失；真实端到端分段指标 |
| OS063 r3 | 两Host真实六格和混合多App组合；按App+Host总成本与direct N进程对比，轻App响应和重App进展同时取证；未测规模/平台不外推 |

r2增补的AC-07与V-05～08继续有效，r3另加V-09接口/兼容负控，原任务承接、总步数不变。定量指标包括延迟p50/p95/p99/max与链路分段、帧间隔/截止失约/丢弃和画面年龄、CPU/GPU/空闲唤醒、命令/资源带宽和重复上传、内存/队列峰值、冷热启动/恢复、多App公平及保真/业务守恒。六格功能分母不变，性能场景按静态/编辑滚动/动画resize/资源更新/媒体/轻重混合冻结；拟1/4/8规模须先证明可测，不将不可运行规模抹掉或外推。

原direct与候选direct均保留为对照，固定同执行器/画质/硬件/源版本/资源/输入；测量提交时间不冒物理上屏。样本长度、轮次、污染处理及预算先冻结，结果必须含未测/失败与完整分布；不得挑最快样本或测后放宽阈值。当前不捏造百分比/SLA，未冻结预算不签后继性能PASS。性能负结果可如实报告，但超已冻结预算须修复或具体修订，不能以研究归档冒完整RQ交付。

不另开泛化性能治理路线、不追加UI060/OS054/OS062原验收；共享源码和测量窗口按owner协调。Linux compositor与ACE/AGE未来profile须继承异步/资源/调度/隔离原则并重新测本平台，不能继承Windows数值结论。

## 6. 交接纪律

每计划一个权威实施worktree/仓；主检出只簿记，独立review沿同树、独立上下文。实际文件与依赖版本冻结，未执行不得计PASS。原始材料仅`D:/autostack/plan-artifacts/plan-NNN/<repo>/`；merge折摘要入Plan后删自有scope，guard clean后移除树；无junction/symlink，不清别案产物。无预算/测试时限新承诺；重资源与原生窗口验证按实际资源窗口单实例，其他agent/聊天不自动创建。

## 7. r3范围、依赖与启动裁定（2026-11-05）

“可启动”指技术前置满足后的建议，不构成本轮实施授权。depends_on列正式实施/交付依赖；只读准备允许提前，但不能勾完尚未满足冻结输入的任务。

| 计划 | 现在可先做（后续授权后） | 必须等待 | 可独立关闭的责任 |
|---|---|---|---|
| UI061 | T-01～04有界研究：正式来源、全能力/direct基线、独立录制重放探针、接口和预算 | 无整计划硬前置；UI060等同符号生产改动须先正式交付，研究可保持只读 | 有实证的可行接缝、分层六格oracle与能力/接口/性能合同；不是全产品PASS |
| UI062 | 目前可读设计与盘点，不进入生产改动 | T-01～05等UI061正式交付；实际与UI060/OS062同符号写面交接 | 两执行形态真实producer与两Host replay绘图/资源六格、平台无关帧资源事务；不等UI063运行栈 |
| UI063 | T-01只读公开入口/服务/运行差额盘点 | T-01正式冻结及T-02～05等UI061、UI062、UI060正式交付 | 通用VM/Rust运行/输入/服务/调度及Host adapter，最小真实双Host六格；不等OS063产品全量 |
| OS063 | T-01只读manifest全产品/Demo/来源/缺格清单 | T-01冻结等UI061；T-02～05等UI062、UI063及OS054正式组合 | 两Host真实桌面策略、全部能力/产品组合与Windows运行闭包；所需UI策略接线纳同组合同 |
| OS062（既有合同，不修订） | 按自身T-01/02环境与Linux双进程可行性门推进 | 仅实际公共接口/同符号正式依赖；不等RQ-W整线 | Linux IPC/映射/claim、Wayland输入、Counter/Notes Rust两模式、Nix安装；不抵RQ六格全能力 |

### 7.1 去重与补遗漏

| 交叉面/发现 | 唯一实施责任与消费边界 |
|---|---|
| UI060与UI063重复风险 | UI060关闭索引/shim、MCP→broker、输入dispatch/帧刷新、真实native API和原Notes/Todo门；UI063消费正式结果补完整RQ运行差额。UI060可选HandlerCall不变强制，也不追加新性能门 |
| UI062与UI063潜在循环 | UI062用确定状态fixture驱动真实VM/Rust producer、跨进程传输及两Host GPU replay验绘图六格；UI063另验公开CLI/IME/API/异步六格；OS063验全产品组合。fixture不得手造输出替代producer或丢能力，未闭能力保持失败 |
| UI061过度承接全产品 | 研究交付实际高风险样本、六格驱动/负控和缺口→owner映射，不要求先实现后继功能；不可行方案有界needs_replan，不能用未测冒研究PASS |
| OS063实际代码写面遗漏 | host/desktop是薄壳，UI ui/session.rs与iced/renderer/shell_state.rs仍承载桌面策略。OS063允许同组UI树做策略配置/接线；通用机械接口先由UI063交付，资源replay由UI062交付，不复制或无人认领 |
| UI062与OS062协议/共享映射交叉 | UI062管平台无关帧/资源事务与信用，OS062管Linux OS通道/映射/claim/peer/句柄；ACK/generation是同一合同的不同层，不能造两套协议 |
| UI063与OS062生成器/入口/初始化交叉 | Linux ELF/UDS/IME/Nix归OS062，Windows通用运行差额归UI063；rust_ui、client_entry/rqhost、Man cmd_run/tool发现、Notes通用初始化逐符号单写，先正式落地后消费 |
| 发布闭包与全产品原有缺口 | UI063/Man交付模式/工具/Host/资源发现，OS063验Windows实际闭包；Nix归OS062。产品业务或重大编译能力缺口具名owner/合同，必需格未闭则整体未完，不无限塞进OS063或缩分母 |

### 7.2 必须交付的接口与版本记录

1. UI061 T-03/04：能力ID、wire/version/capability、资源与槽单位、ACK/fence职责、分层oracle/负控、P/M测量入口与冻结预算；给每一后继可独立验的输入/输出。
2. UI062 T-04/05：精确packet/replay、enqueue/ready/cancel、frame/session generation、资源寿命及present反馈；旧新端、缺能力/坏帧/背压兼容证据。
3. UI063 T-01/03/04/05：surface身份、placement/z/clip、坐标逆变换、focus/capture、resize/DPI、可见性、readiness/present及平台服务；两Host最小运行驱动与Man闭包发现入口。
4. OS063 T-01～05：正式产品/工具/Host版本组合、OS与必要UI策略接线、来源/能力全分母及Windows运行闭包；必要UI接线先落地再固定OS组合。

共享修改执行前登记实际symbol/owner、基线commit、协议/capability/profile与后继消费版本。先交付的一方保兼容承诺，后改者对旧/新producer-host配对及相关平台作定向验证。OS062的8MiB控制帧/64MiB段门不被新预算暗改；不兼容版本明确拒绝、不静默降级。无法兼容时暂停该接口、具名修订双方消费安排；不制造“OS062等UI063、UI063又等OS062”的全计划循环。Windowsfixture不能替代真实Linux IPC/IME验收，旧Linux有限通过也不抵新wire/全保真通过。

### 7.3 本轮证据边界

只读核UI060主检出恢复合同、追加T-04/AC-07～09与最新续接：数据链局部已有记录，帧刷新/真实输入与最终验收仍open，不能当已交付。UI053归档6/6及cleanup closeout可核，两个delivery commit在当前主线祖先。OS062仍drafting r1/0-13。其余路线状态未在本轮重新全审，实施启动须重读唯一合同。本轮没有实现/编译/运行/测量，没有联系其他会话或代签独立final review。

静态核对：四Plan合计27任务、每份7 AC/9 V，依赖图无环，RQ及当前路线155条本地引用有效，三仓范围内diff --check通过。整索引预扫另发现非本线OS049旧active链接（实际已archived）；保留他方既有行，范围检查仅核本轮RQ指针，未将该旧链接记为通过。

</details>
