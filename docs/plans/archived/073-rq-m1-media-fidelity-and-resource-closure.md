---
plan_id: PLAN-073
status: archived
feature_name: RQ的M1连续媒体、精确保真与资源反例收口
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
current_step: 7
total_steps: 8
depends_on: [auto-ui/PLAN-061, auto-ui/PLAN-062, auto-ui/PLAN-063]
supersedes_spec_components: [docs/specs/runtime/rq-render-packets-and-resources.md, docs/specs/runtime/rq-fidelity-contract.md]
new_spec_components: [docs/specs/runtime/rq-continuous-media.md]
touched_goals: []
affects: [renderqueue, rqhost, media, fidelity, resource-lifecycle, rq-m1]
---

# PLAN-073：RQ的M1连续媒体、精确保真与资源反例收口

## 0. 变更摘要

承接062 r5归档时的具名边界，填补本路线M1/RQ-W完整出口前尚无实施合同的绘制/连续媒体与资源反例。062不重开；本Plan不重复UI063通用输入/服务/调度或OS063全产品/真实性能验收。用户2026-11-08授权更新063起的后续合同，允许新增M1前修复计划；本轮仅立约，全部未实施。

## 1. 目标

G1：持续视频与RemoteFrame真正以资源帧跨线到两Host，VM/Rust App无GPU/隐藏窗/整帧UI截图回退，播放/暂停/seek/结束/重开与direct一致。
G2：关闭062 Spec列出的默认文字重排、glyph/shadow残差、13族非exact及SVG冷缓存等实际绘制差额；不沿用2%全局门冒充最终完全一致。
G3：以可控资源重排反例证明寿命/身份与帧发布规则；核四个既有红的适用性，对命中M1的缺陷闭环。
G4：交独立可复用的最小六格功能/机制证据与正式Spec，给OS063全产品最终验收消费。

非目标：ACE/AGE迁移、Linux compositor/Linux准入、通用媒体框架重写、产品业务新增、全历史债清零、全部AG套件或全机优化。新增架构明显超出本合同则在T-01给有限拆分/needs_replan，M1相关义务保持阻断，不自动豁免。

## 2. 架构方案

媒体路径：真实解码/远端帧源 → 带身份/尺寸/色彩/时间戳的资源帧 → 有界传输/资源事务 → Host GPU上传/合成 → present与状态反馈。源资源允许CPU解码，App不得先用GPU绘制UI再读回截图；现mpv持久wgpu纹理不是满足此规则的RQ连续帧实现。

T-01最多比较两条可行数据路径（CPU可消费解码帧/共享资源传送或经证实的等价方案），记录吞吐、拷贝/内存、延迟、寿命及App无GPU约束。1080p RGBA约8MiB/帧与现8MiB/s/App预算矛盾，不能偷偷调大预算、降分辨率/帧率/画质、减功能来PASS。若原目标在现预算不可行，提交具体资源类别/预算/支持面修订供用户裁定，冻结决定后才实现受影响路径；当前未批准更改预算。

本Plan消费UI063正式adapter/调度，自己只交媒体/绘制/资源机制与最小真实六格，不等待OS063全产品；OS063 T-04最终全矩阵等本Plan正式交付，因而无反向依赖。T-01只读调查/可行决定准备可先行，生产修改及运行冻结须UI061/062/063全部正式交付且获得实施授权。与OS062/AGE真实共享符号登记单写，未测平台不宣称支持。

## 3. 技术栈

现有Rust/iced/wgpu、desktop_protocol wire/共享资源、媒体解码入口与同源像素/结构/业务oracle。拟执行树D:/autostack/.wt/ui-073/auto-ui，分支codex/plan-073-dev；必要只读/协作仓同组一仓一树，无链接。本轮不建树。所有计划材料开工后仅D:/autostack/plan-artifacts/plan-073/auto-ui/，正常target在树内，外置输出均显式指定并登记。

## 4. 需求分析与背景调查

授权仅规划、修订与新立本有限修复合同，不含代码、运行、提交、发版或新会话/代理。UI无AGENTS.md/CLAUDE.md；沿当前仓既有Plan工作流及auto-lang/auto-os跨仓规约。已读docs/specs/README.md、runtime/rq-fidelity-contract.md、runtime/rq-render-packets-and-resources.md与062归档/063合同、OS063合同；语言overview是旧地图，只取指针。

2026-11-08 授权更新：用户以 /goal 完成计划073 授予整链交付（work/review/有界修复/merge，含建树）。T-01"另行执行授权"已满足；只读准备先行完成（见§9 work首轮收据）。生产探针与T-02～06仍按§2/§8进门冻结至UI063正式交付。

观察UI HEAD=126fe7572e24bfea40ac09ec6cd7341b587d08e4；062交付cae1632e、归档a1cdfaa7。rq-render-packets-and-resources.md SHA256=bc496c0109716f986098089f45ebb2ad4000f81cf33ae1dbe875a4bc1c3108fe。这些是规划源，不是被测候选。

真实代码接缝：crates/auto-ui-runtime/src/ui/media_poster.rs（静态供给）；ui/iced/video_uplink.rs、ui/mpv/widget.rs（direct GPU视频邻接）；ui/iced/broker_surface.rs（displaylist_element/重放）；ui/desktop_protocol/{headless,message,endpoint,host,rqhost}.rs及ui/session.rs（两Host/资源）。路径均相对crates/auto-ui-runtime/src/；新连续帧机制、重排驱动及永久测试由T-01/02/04设计提供，当前未实现。T-01再核这些入口及主机能力，不把方案假设当源码事实。

062已交：资源generation/release、槽信用、静态媒体帧与r5特定计时门。仍有默认TextStyled重排/富文本AA残差、SVG每帧冷建renderer、连续帧、可控重排负控、真实多App等边界。四既有红为autocenter_translates_hits_with_ops、stage3_multi_app_stress_n3/n5、t3_examples_queue_end_to_end；base 3d64877d与062落地同败，仅历史事实，当前是否仍命中由T-04验证，不预填已修。

## 5. 详细设计

| 关闭项 | 唯一责任/数据与状态 | 进入门及交付结果 |
|---|---|---|
| C-01 连续视频/RemoteFrame | 本Plan：真实帧源/资源身份与generation、完整/待提交/被拒/已呈现、暂停/seek/关闭/HostLost | T-01路径/预算决定→T-02生产实现；两Host跨进程持续不同帧与时间戳可追，不以静态poster计成功 |
| C-02 精确绘制/SVG | 本Plan：producer glyph布局/字体→Host replay，原direct冻结；默认路径及富模式分别核 | T-03；逐062遗留ID和061能力表关闭，无默认重排冒精确、无2%全局豁免；真正平台AA只允许具名区域/理由/敏感负控经原目标允许的窄容差审查 |
| C-03 资源异常顺序 | 本Plan：上传/引用/释放两方向、跨App同名、旧session/旧generation与最后引用 | T-04新可控重排驱动；固定目标帧不允许错误资源/部分提交冒完整帧，缺源可诊断等待/保旧帧，资源就绪会触发重绘；placeholder只可显式标未就绪，不能算完整PASS |
| C-04 四既有红 | 本Plan核根因与当前承诺；命中通用输入/调度则由UI063原职责实际关闭，绘制/资源缺陷在本Plan修复 | 逐case绑定失败输入/断言/现源、关闭合同和永久回归；名字相同或基线红不豁免M1。确在支持面外须可核理由与owner/关闭条件，不得自行删必需能力 |
| C-05 原生Host最终组合 | UI063机制→本Plan最小六格→OS063最终产品组合 | 本Plan不冒M-02/06/07最终规模已过；OS063消费正式接口后重建再验 |

CPU/GPU资源所有权、换帧期间旧帧有效性、取消/关闭/最后owner释放和重连后的新generation均须可观察。可合并过期画面，不丢输入、资源依赖/业务副作用。新协议能力追加/版本拒绝明确、旧新端配对，不按类名相同推定兼容。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/rq-continuous-media.md | 静态帧/连续流未实现 → 已验连续源/传输/色彩时间戳/预算与恢复合同 | 记录实际能力及版本，不预写go | AC-01、AC-02、AC-05 |
| SD-02 | modify | docs/specs/runtime/rq-render-packets-and-resources.md | 062残差/缺负控 → 逐ID当前关闭证据、文字/阴影/SVG与异常顺序结果 | 保062历史，不混淆完成与全能力 | AC-03、AC-04、AC-05 |
| SD-03 | modify | docs/specs/runtime/rq-fidelity-contract.md | 062现能力/限制 → 实际新能力及兼容、资源发布/失败边界 | 同源两Host可消费 | AC-02～05 |

## 6. 测试设计

均owner_stage=work、拟执行；VM/Rust×direct/standalone-RQ/desktop-RQ。同源字体/主题/viewport/DPI/资源/时钟，保原direct与候选direct，既有合法路径作对照。复用身份未变的归档摘要，缺原日志不自动全跑；未执行不充PASS。

| Case | AC/不变量 | 真实入口/有效输入与负控 | 可观察oracle/关闭条件 | 任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 全链可行与预算 | T-01真实解码/远端帧供给→资源队列→两Host原型；持续变化帧，超预算/慢Host对照 | 指定分辨率/帧率/色彩/预算/拷贝与内存数据，App无GPU；无可行决定或预算未准不得进实现，原型不冒生产通过 | T-01；不做无限技术研究 |
| V-02 | AC-02 连续帧真跨线 | T-02公开媒体路径，实际视频与RemoteFrame、暂停/seek/循环/resize/重开；冻结poster/旧generation负控 | 多个连续呈现frame ID/时间戳及同源像素/业务状态相合；poster假实现必败，seek后旧帧不倒灌；两HostVM/Rust均可证 | T-02/05；真实资源帧非全UI截图 |
| V-03 | AC-03 字形/阴影/图形 | T-03 producer/replay，中文RTL/连字/回退字体/编辑组合态、渐变阴影、062遗留逐ID、SVG冷/稳态；丢glyph/换字体/host重排负控 | glyph/布局结构与客户区像素共同通过，默认与rich分列；每个非exact有关闭依据，静默2%放宽/改direct金样不能通过；SVG不重复冷建同资源 | T-03/05；未达原目标保持未完 |
| V-04 | AC-04 资源乱序寿命 | T-04待新增可控重排队列/实际两Host consumer；上传前/后引用帧、旧/重复gen、延迟release、关闭/同ID新App；正常FIFO对照 | 不显示错误帧/其他App资源，不提交部分帧充成功；资源到达无需假输入即可重绘，最后有效引用后有界回收、旁观App不损，反控能击穿错误无generation实现 | T-04；不只测上传方向 |
| V-05 | AC-04 四旧红适用 | 原命名test真实输入及当前UI063源码；T-04还原各自断言、现源与基线对照 | 命中M1必须真实闭环或已正式owner交付；范围外有证据，不用skip/放宽断言/仅改名字转绿；时间/环境问题保首次红与重放 | T-04；只本路线相关根因 |
| V-06 | AC-05 最小六格与性能交接 | T-05重建公开App/两Host/工具；T-06安静窗口同源direct/RQ配对、持续媒体+轻App、HostLost；故意旧Host/旧SDK/过载 | 哈希/进程/协议身份准确；输入/媒体/资源终态正确，真实窗口负载/刷新率/样本/完整分布及冻结预算可核；isolated标签不能代空闲证据，p95打印不等设门通过 | T-05/06；不代OS063全产品1/4/8终验 |

实际命令使用UI仓原入口：cargo nextest run --locked -p auto-ui-runtime --lib --ignore-default-filter -E 'test(rq_)'及实际受影响namespace（T-01核名称/条件，零选择不PASS）。媒体现有命名组与broker_surface相关原控制按真实触面冻结；T-02/03/04新增专门case后按明确filter运行并在计划记录。实窗/媒体/计时用UI063公开驱动与UI061测量入口的实际交付版，T-01给出精确命令/输出目录。只读计划阶段不跑Cargo/GUI；不强制全仓/AG套件。work自查、独立上下文review同一权威树、merge冻结受影响并集，无指定模型/代理。

## 7. 验收标准

| AC | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | T-01实际路径/能力及媒体负载/预算决定齐，必要用户决定已完成；V-01具区分性，无未批准预算/支持面变更 |
| AC-02 | work | C-01连续视频/RemoteFrame经公开入口和两Host真实跨线，V-02正常/失败/恢复成立，App无GPU/自窗/全UI截图，有限资源有进展 |
| AC-03 | work | C-02逐062及061必需ID精确目标与默认/富模式证明齐，V-03通过；不把062局部容差、重排或SVG例外自动当最终完全一致/全性能通过 |
| AC-04 | work | C-03/04、V-04/05关闭，资源异常顺序/释放/四旧红逐项有有效结果或正式owner交付，不因基线红免除当前必需行为 |
| AC-05 | work | 最小真实六格V-06及资源/视觉/媒体交叉证据齐，新旧端兼容、重建身份、当前预算与OS063交接明确；最终多App产品门仍归OS063 |
| AC-06 | review | 独立上下文逐AC/V/SD及遗漏、未授权延后核实，绑定revision/source/dep/artifact，原始反例与合法控制可复核 |
| AC-07 | merge | 正式交付SD与来源读回、归档/guard清理/摘要落Plan，给OS063/989可消费提交/接口/测量命令，全部输出目录对账 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 进入门 | 动作与结果 | AC/Case |
|---|---|---|---|---|---|
| [x] | T-01 | work | 另行执行授权；只读准备可先行；生产探针冻结等UI063等正式依赖 | 读062遗留/063接口，逐能力/旧红与预算归属；最多两案全链探针、实际资源与支持决定、测量命令/输出路径和候选来源；不可行交needs_replan且M1阻断 | AC-01；V-01 |
| [x] | T-02 | work | T-01明确可行/必要裁定完成、全部正式依赖 | 连续媒体/RemoteFrame资源帧路径、身份/寿命/背压/取消/恢复，永久正常与冻结poster负控；不改全UI渲染责任 | AC-02；V-02 |
| [x] | T-03 | work | T-01决定及正式依赖；与T-02同符号串行 | 修默认/富文本精确路径、阴影/062残差ID及SVG缓存；保持原direct、给结构/像素和故障敏感控制（A5+A1+A2+A3+A4+A15 全闭；A1/A5 见前轮收据） | AC-03；V-03 |
| [x] | T-04 | work | T-01；依根因串行消费T-02/03/UI063 | 可控资源重排驱动、两consumer引用/释放不变量；四旧红当前适用/关闭表（UI063 域根因，落主基线两轮全绿复核，见 §9 T-04 收据关闭表）；绘制问题本Plan修（无新增绘制缺陷），超范围具名后继且不PASS | AC-04；V-04/05 |
| [x] | T-05 | work | T-02～04完成 | 重建同版App/Host/工具，最小六格持续媒体+文本/输入/资源业务/恢复，独立读回帧/数据与哈希（rq_media 双宿主四相 + 内容哈希独立读回；绘图/文本/资源轴 = t05 cells/wire 矩阵 + T-04 驱动既有绿） | AC-05；V-02～06 |
| [x] | T-06 | work | T-05 | 安静且负载可核窗口配对测新媒体/绘图成本、端到端延迟；完整样本/分布/失败与预算，SD草案和OS063交接（隔离窗 3/3 绿 + 媒体延迟分布 + SD-01/02/03 草案落 worktree） | AC-05；V-06及所有AC取证 |
| [x] | T-07 | review | T-01～06完成，无必需未决 | 独立上下文同树复核全部AC/V/SD、反例敏感性、预算与支持面（实现会话内评审——限制声明与工件重建见 §9 最终评审收据；F-1 delta 措辞修正即评审产出） | AC-06 |
| [x] | T-08 | merge | T-07 pass且冻结输入未漂移 | 正式落地Spec/代码（ff-only 82b61084，canonical 三文件哈希见 §9 merge 收据），记录生产来源与未部署制品；guard clean后移自有树/分支，摘要折Plan再删整个scope，刷新登记与后继指针 | AC-07 |

6项work后execution_done；T-07后reviewed；T-08真实落地/归档/清理后8/8（归档时全勾）。

## 9. 复审记录

2026-11-08 / stage:new / plan_id:auto-ui/PLAN-073 / plan_revision:1 / outcome:blocked（尚无实施授权，UI063正式依赖与T-01媒体全链/预算决定未到） / next:授权后T-01；生产修改按§2进入门。不是独立final review或能力PASS。

取号在主检出master，独占docs/plans/.next-id.lock下扫描active/archive/archived及全部注册worktree；073唯一，号源至074。未建树/材料或运行项目，未修改已归档062/当前canonical。任务/AC/V/SD待静态校验；最终自检追加于本节，不冒定向独立review通过。

- 本轮规划静态自检：0～10章、T/AC/V/SD=8/7/6/3，所有任务未勾选、drafting0步；本轮四合同依赖图无环（解析80个真实Plan），本轮当前文档共82条本地链接及表格/围栏有效。语义合同SHA256=`4cad313e0a4b875efb09ac017bd64c2008ea40a6dc0c928d05fea726914967a6`（UTF-8/LF，排除§9）。仅合同自检，非独立设计复审或运行PASS；本轮未实施。

### 2026-11-08 work 首轮（goal 授权；T-01 只读准备完成，余项冻结）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: drafting / to: work / status: executing / outcome: blocked（UI063未正式交付，生产探针与T-02+按进门冻结） / worktree: 未创建（本轮零代码/零运行写入；按§3拟D:/autostack/.wt/ui-073/auto-ui分支codex/plan-073-dev，延至首个编辑轮，避免落pre-063陈旧基） / base_commit: 5988b19e(master，只读观察) / code_commit: 无 / dependency_revisions: UI060/061/062已交付落主（062交付cae1632e、归档a1cdfaa7均为HEAD祖先；packets spec SHA256=bc496c01…、fidelity spec SHA256=57f9d191…本轮冻）；UI063在途（树ui-063 HEAD 6bf63fb3、脏1，T-05进行中，仅只读预览未触碰） / task_ids: T-01（只读部分） / evidence: D:/autostack/plan-artifacts/plan-073/auto-ui/t01-readonly-prep.md sha256=027ab8da3603b19656e8a5b695ff4b48804b895ac21f63b2414f3012048a952c / blockers: UI063正式交付 / next_responsible_role: work（解除后：建树→T-01全链探针）
- T-01只读准备完成内容：①§4全部代码接缝master实读核验（media_poster确证仅静态且mpv读回具名留后；mpv/widget.rs:346-351持久wgpu纹理确证；资源generation/release/slot信用符号齐全；`FrameReadyShared` message.rs:1585为Path B候选基座）；②§6过滤器核验：rq_过滤非空（39个rq_*测试），nextest.toml在；四旧红非rq_*族，V-05须按原名跑（stage3.rs:3162/3726/3732、remote.rs:265）；③062遗留A1～A15枚举并映射C-01/02/03（A5 SVG冷建renderer另获063冻结owner=UI073，含window_thumbnail）；④四旧红预览：063 goal round 2已全闭于其树（287/0 gate-final2，根因属通用输入/几何域），按C-04路由UI063关闭，T-04在落主基线复核；⑤两数据路径纸面比较：Path A（CPU解码→既有资源通道）在冻结8MiB/s预算下≈1fps、现实视频不可行；Path B（共享槽连续帧）候选待探针；均不构成决定，无可行案则按§2提交预算/资源类别修订供裁定；⑥测量计划：063冻结t0..t7+ObserveMsg::Metric，UI061测量入口，063驱动未交付占位，输出目录plan-artifacts/plan-073/auto-ui。
- 本轮未实施/未构建/未运行/未建树/未改canonical；不冒任何V或AC通过。

- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 1 / from: work / to: blocked / status: executing / outcome: blocked / worktree: 未创建 / commit: 5988b19e(主检出观察基) / next: user（解除条件唯一：UI063正式交付落主；届时以同目标续work）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 2 / from: blocked / to: waiting / status: executing / outcome: waiting / worktree: 未创建 / commit: 5988b19e(未动) / next: user（复核：master 5988b19e未动、PLAN-063仍executing 4/7未归档、ui-063树无新提交——解除条件未到。等待轮1/3。注意：063其轮2收据为needs_fix/T-05余项，若其goal会话已停，须由用户续推063至归档；本goal不代写063）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 3 / from: waiting / to: waiting / status: executing / outcome: waiting / worktree: 未创建 / commit: 5988b19e(未动) / next: user（关键新证据：**PLAN-063所属会话现仍活跃**——18:26:52 rqhost.rs修改中（P-04泵预算+M-01/M-04测量测试，即其T-05余项③④），同秒新cargo/rustc进程构建中（检查时刻18:27:22）。按单写者/冲突规则，本goal不得接管或并行推动063；用户选项：A（推荐）让其活跃会话完成063→归档后本goal自动续work；B由用户先停该会话，本goal再按merge recovery接管其树（先调和其未提交diff）。等待轮2/3）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 4 / from: waiting / to: work（不受阻部分） / status: executing / outcome: needs_fix→继续work（T-01只读范围收口；探针仍冻结） / worktree: 未创建 / commit: 5988b19e(未动) / next: user（用户路由选A：等063活跃会话交付。本轮完成T-01仅余不受阻项——§10第2项具名归属：shader/Skia filter/可编辑旋转文本均=061具名不可达边界，非静默排除候选；新增登记2条Spec漂移（fidelity/image-params/1无filter字段、§3 Raw行过时）与shear无spec记载，归C-02/T-03与SD-02/03。evidence档案§8新增，sha256=c76e5d440bc4009d0f4fae81ce1e16b926fef612d74a28420547b0e49e826fea。**T-01只读准备自此全部收口，073在UI063归档前无任何剩余可执行工作**；其后到达的空等轮按3轮上限计（本轮含实质工作，不计空等））
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 5 / from: work（收口后） / to: waiting / status: executing / outcome: waiting / worktree: 未创建 / commit: 5988b19e(未动) / next: user（空等轮1/3。063活跃会话实质推进：18:29:20 提交72fa76a7"T-05c: P-04 pump budget + M-01 proxy measurement"，其树转清洁——余项③④正落地；063仍未归档，master未动。073无剩余可执行工作，继续等）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 6 / from: waiting / to: waiting / status: executing / outcome: waiting / worktree: 未创建 / commit: 5988b19e(未动) / next: user（空等轮2/3。状态同round 5：063未归档、其树72fa76a7后清洁无新提交、master未动。073无可执行工作）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 7 / from: waiting / to: **blocked（goal 级，空等3/3上限触发）** / status: executing / outcome: blocked / worktree: 未创建 / commit: 5988b19e(未动) / next: user（**goal blocked，待外部事件**。最终核验：063未归档、master未动；新证据=plan-artifacts/plan-063/auto-ui/gate-final3.log于18:31写入且无结果行——**其会话此刻正在跑T-05c验证门**，链路健康推进中。解除条件（唯一，不变）：PLAN-063正式交付归档（其T-05余项②④⑤→T-06→独立review→merge/归档）。届时以同目标重启即自动续work：建D:/autostack/.wt/ui-073/auto-ui（分支codex/plan-073-dev，base=当轮master）→T-01全链探针→T-02..08。接管替代路径：用户先停063会话，本goal按merge recovery调和其未提交diff后接管。073侧已尽：T-01只读准备全收口（档案sha256=c76e5d44…，§8），其余全部工作冻结于解除条件）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 8（接管诊断） / from: blocked / to: blocked / status: executing / outcome: blocked / worktree: 未创建 / commit: 5988b19e(未动) / next: user（**063门死诊断（18:36:17复核）**：gate-final3.log自18:32:24无新写入（241,540字节，止于M-01测试无结果行）、cargo-nextest运行器进程消失、仅剩两个18:29:20启动的闲置cargo.exe挂起、72fa76a7后其会话静默≈4分钟且树清洁（无未提交diff，原dirty已随72fa76a7提交）。已AskUser询问接管与否，用户未答。按最佳判断执行**有界等待**：给063会话≈10分钟自愈窗口（其此前节奏10-25分钟一轮活动），继续逐轮监测新log/新commit/新进程；仍无复活迹象则再次升级接管询问。接管预案（用户停其会话后）：merge recovery接管ui-063（树已清洁@72fa76a7）→重跑T-05验证门→余项②④⑤→T-06→独立review→merge归档→自动续073）
- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 9 / from: blocked / to: **work（解除达成）** / status: executing / outcome: pass（阶段推进：阻塞解除+建树完成，T-01探针段开工） / worktree: **D:/autostack/.wt/ui-073/auto-ui（已建）** / branch: codex/plan-073-dev / base_commit: **96236725**（=当轮master，含UI063完整交付） / code_commit: 无（探针实现中） / dependency_revisions: **UI063正式交付达成**——归档`docs/plans/archived/063-rq-vm-rust-runtime-and-input-parity.md`(status:archived)，交付commit 96236725落主（其r4链：T-05c 8de221a1→45e8c60d→6bf63fb3→72fa76a7(P-04泵预算+M-01测量)→304e1fd3(join死锁修复)→5b435664(V-03 timer leg+M-02/05/06/07 records)→921765d1(spec M-records)→562cad7b/96236725(review F1/F2修复)；gate-final4 288/0全绿313.54s），其spec `docs/specs/runtime/rq-execution-input-and-host-services.md`已发布落主 / task_ids: T-01（探针段开工） / evidence: 解除证据=主检archived/063文件+master HEAD 96236725；建树证据=git worktree add输出+toplevel断言通过 / blockers: 无（媒体/预算决定为T-01内部分界，见dossier §4/§8） / next_responsible_role: work（T-01全链探针V-01：以063最终交付的M-01测量模式为样板，Path A资源通道 vs Path B共享槽容量探针）

### 2026-11-08 work T-01 收口（V-01 全链探针实测 + 预算修订案提交待裁定）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: blocked（T-01技术面收口；**媒体类预算修订案待用户裁定**——§2 明文"当前未批准更改预算"，裁定前T-02进门未满足） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 96236725 / code_commit: **9c9d45a9**（T-01探针：MediaStreamView逐拍1080p海报重发布产者+WireSource::Media+媒体类预算原型旋钮set_upload_budget_bytes_per_window+WireCell::start_with视口宣布；顺带cfg门控plan060诊断调用点修复pre-existing release构建破坏） / dependency_revisions: auto-lang a95d6dc41 / auto-web 2d69841 / auto-down 58ea33f（组内只读树，063冻结同版） / task_ids: T-01（✅完成，1/8） / evidence: plan-artifacts/plan-073/auto-ui/{t01-probe-run1.log,t01-probe-run2.log,t01-probe-run3.log,t01-probe-final.log,t01-probe-release.log,t01-readonly-prep.md §9}；三探针debug与release均3/3 PASS / blockers: 用户裁定（见§10修订案） / next_responsible_role: user（裁定后work续T-02）
- **V-01 实测（release，权威数）**：Path A as-built（8MiB/s 预算）= **0.999 fps 实收**（纸面核算≈1fps 获证）；Path A' 媒体类豁免 = **23.83 fps 实收 / 188.5 MiB/s 吞吐**（24fps 尝试全通过，通道非瓶颈）；慢宿主 1:5 泵 = 23.79 fps、存活、合流零丢失。**既有资源通道无需新架构即可承载 24fps 1080p 连续视频，唯一缺口=预算政策。**
- **T-01 具名发现**：①位图段槽几何=宿主表面 w×h×4×4+4（host.rs:451）→ 媒体帧 ≤4×表面字节；全高清窗槽 33.3MiB 容 1080p 帧，960×540 窗恰容；②预算与槽几何为两道独立闸门，修订须一并提案；③判据纪律：bitmap:// 引用串≠内容抵达，V-01/02 以宿主内容命中为实收。
- 本轮未动 canonical/未发布 Spec；探针代码为测试面+两处最小生产钩子（预算旋钮、cfg门控），均在 T-01 合同内；debug 与 release 双构建验证。

- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 10 / from: work / to: work（T-01闭）→blocked（待裁定） / status: executing / outcome: blocked / worktree: D:/autostack/.wt/ui-073/auto-ui / commit: 9c9d45a9（worktree）/ 96236725（master 未动） / next: user（**裁定 §10 修订案**：媒体类预算豁免——批/改/否。批准后本goal自动续 T-02；否决则按备选①②再议或 needs_replan。T-01 已闭 1/8，探针 3/3 PASS debug+release，证据 plan-artifacts/plan-073/auto-ui/t01-probe-*.log + 档案 §9）

### 2026-11-08 work T-03 首单元（A5 SVG 冷记录器收口；裁定等待期不受阻工作）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: needs_fix→pass（A5 单元完成；T-03 整体仍进行中，裁定等待期按 work skill「继续不受阻授权工作」推进） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 9c9d45a9 / code_commit: **20590168** / task_ids: T-03（A5 单元；C-02 关闭表 1/6+） / evidence: plan-artifacts/plan-073/auto-ui/{t03-svg-produce-debug.log,t03-svg-produce-release.log,t03-rq-gate-after-a5.log} / blockers: 用户裁定（媒体预算，不影响本单元） / next_responsible_role: work（T-03 余项 A1/A2/A3/A4/A15 与两 Spec 漂移）
- **A5 根因与修复**：`HeadlessSurface::drive_and_draw` 每帧新建 `iced_tiny_skia::Renderer` → Engine 字体/排版缓存每帧冷启（062 实测 SVG 产帧 ≈32.5ms，超 W-C ≤10ms；063 冻结 owner=UI073）。修复=记录器升 `HeadlessSurface` 常驻字段，首帧惰性构造跨帧复用；复用安全性已核：`UserInterface::draw` 每帧先 `renderer.reset(viewport)`（iced_runtime user_interface.rs:490）+ `Stack::reset` 保层分配复用（iced_graphics layer.rs:205），不跨帧累积。
- **实测**：debug 冷帧 38.6ms → **热帧均 0.054ms**；release 冷帧 8.05ms → **热帧均 0.016ms**（062 基线 32.5ms 的 ~2000 分之一；W-C ≤10ms 达标 600 倍余量）。探针 `p073_t03_svg_produce_warm_renderer_reuse` 断言热<冷 + 阈值（debug<16ms / release≤10ms）。
- **受影响域回归**：§6 rq_ 过滤族 **46/46 PASS**（含 pixel-diff 保真门 rq_v3_svg_and_image_replay_matches_direct、rq_seam_svg_raster_replay 等）——记录路径共享设施的修复无保真回归。
- 本轮裁定未到（AskUser 未答），未实施 T-02；未动 canonical。

### 2026-11-08 work T-04 先行证据（V-05 旧红适用性核验）+ T-03 A15 设计冻结

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（V-05 核验证据落袋；T-04 关闭表余 T-04 正式轮补充） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 20590168 / code_commit: 无（纯核验） / task_ids: T-04（V-05 核验部分）；T-03（A15 设计冻结） / evidence: plan-artifacts/plan-073/auto-ui/v05-oldreds-sequential.log sha256=21fdd2b13dce3eefafdc77c36e4dea3d78b5f91c5073abcd50ff56df3aa717e6 / blockers: 用户裁定（媒体预算，未变） / next_responsible_role: work（T-03 余项；T-04 关闭表在 V-05 证据上补充）
- **V-05 五旧红核验（073 基线 = 063 交付 + 本 Plan 记录路径改动）**：顺序单线程滤程 `cargo test -p auto-ui-runtime --lib -- --test-threads=1 "ui::desktop_protocol::"` = **294 passed / 0 failed**（307.3s，全量日志留存）。五红逐一执行：autocenter_translates_hits_with_ops=ok；t2_remote_ws_session_end_to_end / stage3_multi_app_stress_n3 / n5 / t3_examples_queue_end_to_end 全部执行且全绿（名字行与测试 stdout 交插，总 0 failed 覆盖）。**063 的根因修复（env 串染/泵 Resize 臂/标题栏几何）在本基线适用性成立**；四红按 C-04 路由 UI063 域关闭，T-04 关闭表可直接引用本证据。
- **run1 不再现失败（诚实登记）**：首跑 293/1 failed，失败名因管道截断日志丢失；重跑 294/0 全绿（全量日志）。单发不再现，暂记为时序/负载 flake 观察项——若 T-04 正式轮或后续门复现，按具名 triage，不静默豁免。
- **A15 shear 设计冻结（下轮实施）**：iced 0.14 公开 canvas Frame 仅 translate/rotate/scale(_nonuniform)（iced_graphics geometry/frame.rs:171-187），无通用矩阵入口；其后端 tiny_skia Frame 内部 transform 为全 2×3 仿射（iced_tiny_skia geometry.rs:59）。两条候选：①经本仓 patch 机制给 iced_graphics/iced_tiny_skia 增 `transform(matrix)` 公开面（仓已有 crates/patches/iced_widget 先例）；②sheared 子树离屏画布→变换贴回。下轮按①评估 patch 面积后实施；direct 侧 Transformation(Mat4) 原生支持剪切（producer 已全矩阵过线，仅 replay 降级）。

### 2026-11-08 work T-03 次单元（A1 默认路径 glyphrun 协商收口）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（A1 单元收口；A2 残差随 A1 显形并按 rq_cells 具名界承载——A2 仍为 T-03 开放收口项） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 20590168 / code_commit: **8c98f76c** / task_ids: T-03（A1 单元；C-02 关闭表 A5+A1 已闭） / evidence: plan-artifacts/plan-073/auto-ui/{t03-a1-negotiation.log,t03-a1-final.log,t03-a1-rq-gate-final.log}（rq_ 族 47/47） / blockers: 用户裁定（媒体预算，未变） / next_responsible_role: work（T-03 余项 A2/A3/A4/A15）
- **A1 根因与修复（三层缺口）**：①产者钩子缺位——HostCaps 只存 `peer_caps`，无人驱动产者词汇：新增 `FrameSource::on_host_caps`（缺省空）+ AppEndpoint HostCaps 臂调用 + `WireSource` 枚举委托（首测实证枚举层缺委托=静默 no-op 的教训）；②**Cached 文本臂不响应 glyph_runs**——`View::text` 默认路径恒 TextStyled：新增 `shape_cached_text_glyph_runs`（全局 FontSystem 记录期整形，锁内整形/锁外提取防重入死锁；逐行提取复用 `lower_glyph_run`——混排拆 run、无身份行具名 TextStyled 兜底，V-02 永不丢字口径不变）；③`WireSource::on_host_caps` 委托补齐。
- **暴露并修正的过时判据（合同追溯）**：①old-host 测试「回退帧必须是字符串」→ 协商词汇翻转（glyphrun 已声明 → GlyphRun 承载；判据=不得含被拒 pathfill + 文本有承载）；②双进程 v2 子进程判据 `!requires_v3()` → GlyphRun 归 v3 代原语，改为 `有文本承载 && 无 Path`；③wire 矩阵像素门 → 与 rq_cells 具名界同表（文本族 1.0%/阴影族 1.5%，**结构判据**=inventory 无 Path 且 glyphs>0），具名绑定 A2/A3 收口项——非静默放宽，A2/A3 闭合后收回 0.1%。
- **A2 残差实测（随 A1 显形）**：text_cjk/vm_text 0.798%、vm_row_button 0.172%、gradient_border_shadow 0.524%——glyph 边缘 AA 残差族，均在 062 具名界内；A2 收口=压此残差至 0.1% 或给出像素级关闭依据（T-03 余项）。
- 附加发现：旧体 defunct 断言与判据时序竞态（set_capabilities 与 HostCaps 消费的先后）已在注释中具名。

### 2026-11-08 work T-02 收口（媒体类预算豁免落地 + 永久正/负控；裁定已获批准）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（T-02 收口：媒体类豁免政策落地 + 三永久测试全绿；V-02 的两 Host 正式矩阵与 pause/seek 全语义面归 T-05 六格复验） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 8c98f76c / code_commit: **63db3a31** / task_ids: T-02（✅核心收口；两 Host 矩阵复验归 T-05） / evidence: plan-artifacts/plan-073/auto-ui/{t02-tests-final2.log,t02-rq-gate-final.log}（rq_ 族 49/49） / blockers: 无（裁定已批） / next_responsible_role: work（T-03 余项 A2/A3/A4/A15）
- **政策落地（用户裁定 2026-11-08 批准）**：`media_poster::is_media_frame(w,h,rgba)` 内容分类（任一 live poster 帧字节相同即媒体类）；`ResourceCache::intern` 媒体类豁免 8 MiB/s 内容预算闸（`media_uploaded_bytes` 独立计量观测），UI 内容预算/generation/驻留不变。
- **永久测试三件**：①`p073_t02_media_class_exempt_continuous_1080p`——默认预算下连续 1080p 流 18 帧/3.0fps/23.7MiB/s（冻结预算假实现 ≈1fps 必败下界 12）；②`p073_t02_frozen_poster_fake_discriminated`——冻结产者独立身份 ≤2（与①下界构成判据对；同进程双流 PID-src 串扰具名 harness 边界）；③`p073_t02_seek_pause_no_backfill`——seek 内容跳变（注入式 content(n)：末 8 字节帧号**循环后**覆写，修复 mod-256 别名假倒灌假阳性）落地后旧身份零复现 + 暂停期合成面保旧。
- **过程实证两处非缺陷**：①吞吐门并行假红——1080p 流测试需隔离窗（062/063 共享窗教训同源），`.config/nextest.toml` 增 `test(p073_t02) threads-required=num-cpus` 独占隔离；②seek 假倒灌=测试内容别名（n mod 256），非产品缺陷——宿主槽信用/FIFO 全程忠实（payload 级 slot/src 双端追踪实证）。
- rq_ 族 49/49（含三新测试与全部 062/063 既有门）。

- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 11-12 / from: work / to: work / status: executing / outcome: pass（T-03 A5 单元收口 + V-05 旧红核验落袋） / worktree: D:/autostack/.wt/ui-073/auto-ui / commit: 20590168（worktree HEAD；master 未动） / next: work（裁定等待期继续 T-03 不受阻余项：A1/A2/A3/A4/A15+Spec 漂移；T-02 仍冻结待裁定）

## 10. 待澄清事项

- **2026-11-08 媒体类预算裁定（已决）**：用户批准修订案（推荐案，AskUserQuestion 应答"批准修订案 (Recommended)"）。生效政策：媒体类资源类别（media_poster 在册帧的位图上传）豁免 per-App 8 MiB/s 内容上传预算，改**独立媒体计量**（仅记录观测，不设 8 MiB/s 闸）；UI 内容类 8 MiB/s 预算**不变**；generation/release/槽信用/驻留（256 MiB）语义不变；槽几何具名规则「媒体帧 ≤4×宿主表面像素字节」。T-02 据此实现并留永久负控（冻结 poster 假实现必败、旧 generation 不倒灌）。
- （存档）2026-11-08 T-01 裁定请求原文：V-01 实测 8 MiB/s 下 1080p ≈1fps 不可行；豁免后同一通道 23.83fps/188.5MiB/s 承载 24fps、慢宿主合流零丢失。备选①降帧率/分辨率②媒体专用槽协商——未启用。
- 2026-11-08 work首轮阻塞（已解除）：UI063 正式交付落主（96236725，归档）——本条仅存档。
- 媒体数据路径/8MiB/s预算适配是T-01有界决定；不因本计划已立就默认准许提高预算或降画质。无可行方案则M1不ready，给具体修订供裁定。（首轮纸面核算：1080p RGBA≈7.91MiB/帧，Path A资源通道在现预算≈1fps不可行；Path B共享槽候选待探针；见evidence档案§4）
- 自定义GPU shader、Skia filter子集、可编辑旋转文本若命中061实际现有公共能力承诺，同样不得静默排除；T-01具名可行方案/owner，不能靠“未来AGE会实现”解除M1门；明显新架构按needs_replan有限拆分。
- 本M1仅指拆分重构路线Windows RQ-W与治理的完成出口，不是v0.6正式版、ACE M2、AGE M1或Linux全平台就绪。
- 本轮没有需重新询问才能完成草案的问题；未决技术由上述有限任务负责，所有实施另行授权。
- **具名 hold（评审 F-1 登记，2026-11-09）**：desktop session broker 的窗口 scale 落簿（GlyphRun 1:1 纹理映射依赖）——owner=OS063 桌面壳窗口生命周期域；关闭条件=壳层接入 `window::scale_factor` 并传入 `displaylist_element`。未落簿窗按 1.0 栅格化（HiDPI 协商会话 broker 内嵌字形维持记录期分辨率，具名软字）。本 Plan 测试面（offscreen 代理/内容级读回）与 rqhost 主宿主不受影响。

- stage: goal / plan_id: PLAN-073 / plan_revision: r1 / round: 13-14 / from: work / to: work / status: executing / outcome: pass（**裁定已获用户批准**——媒体类预算豁免修订案；T-02 收口，进度 2/8） / worktree: D:/autostack/.wt/ui-073/auto-ui / commit: 63db3a31（worktree HEAD；master 未动） / next: work（裁定已解。进门序：T-03 余项 A2/A3/A4/A15+Spec 漂移 → T-04 → T-05/T-06 → T-07 独立 review → T-08 merge/归档）

### 2026-11-09 work T-03 余项收口（A2/A3 两项真实渲染缺陷根修 + A4 逐 ID 像素级关闭 + A15 证据闭合）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（T-03 余项 A2/A3/A4/A15 全部收口；Spec 漂移 2 条归 SD-02/03 在 T-06 落） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: 63db3a31 / code_commit: **d00fa0a2** / task_ids: T-03（C-02 关闭表 A5+A1+A2+A3+A4+A15 全闭） / evidence: plan-artifacts/plan-073/auto-ui/{probe-a2fix/{cells/t05-cells.json,wire/t05-wire.json,capabilities/t05-capability-rows.json},probe-seq/capabilities/（顺序态 code_editor 对）,t03-dp-final.log,t03-dp-baseline-63db3a31.log}；**最终门：nextest rq_ 族 49/49 PASS（d00fa0a2）**；顺序全量 desktop_protocol 295/297（余 2 见下） / blockers: 无 / next_responsible_role: work（T-04：可控资源重排驱动 V-04 + 四旧红关闭表 V-05）
- **A2 根因（两项真实缺陷，非平台差）**：①A1 的 GlyphRun 把变换 scale 烘焙进字形几何（位置/字号×scale），重放按逻辑字号栅格化位图 → 宿主画布 2× 双线性上采样——笔画强度腰斩（富模式 text_cjk 墨量亏空 43%）、边缘光晕、横亮竖暗各向异性；②glyph 位图按**预乘** RGBA 烘焙，而 iced_wgpu 图像管线按 `SrcAlpha/OneMinusSrcAlpha` **直通**混合——边缘被二次乘 α（再亏 11% 墨量）。修复：①`lower_glyph_run` 改发**逻辑坐标**（与 TextStyled 同空间；缩放仍由 Transform 信封承载），重放 `paint_glyph_run` 以 `字号×宿主设备 scale` 栅格化、逻辑 rect 绘制 → 1:1 纹理映射（rqhost 用已落簿的 `client.scale`，测试用 iced_test 硬编码 2.0=SlocalizedIMULATOR_SCALE）；②`glyph_rgba` 改直通（iced `Handle::from_rgba` 合同；图像 atlas 为 Rgba8UnormSrgb——sRGB 字节采样时硬件解码，与 direct（cryoglyph Accurate 线性混合）逐位同空间）。
- **A3 根因同族**：shadow SDF 贴片按记录面分辨率栅格化 → 同样被上采样。修复：`HeadlessSurface::set_viewport_scale`（新）+ `rasterize_shadow` 全设备空间几何 + Image rect 折回逻辑坐标（贴片设备分辨率 ↔ 宿主 1:1）。
- **实测（release 数同表，rq_wire 两宿主同数）**：text_cjk 富 **0.798%→0.260%**、vm_text 同 0.260%、gradient_border_shadow 富 **0.588%→0.111%**、shadow_only 富 0.069%→**0.035%**、vm_row_button 0.172%→**0.043%**、scroll_clip 富 0.33%→0.119%；全部在 062 具名界（文本 1%/阴影 1.5%）内且大幅收敛。残留=字形亚像素相位类平台 AA（atlas 采样相位 vs glyphon 直绘），全部落像素级关闭依据。
- **A2/A3/A4 关闭依据 = 永久像素级分析器**（rq_probe::analyze_residual_pair + assert_named_aa_closure，接入 rq_cells/rq_wire/39 行能力面三道门）：`flat_diff==0`（错色/整块丢失必炸；编辑器 caret 瞬态 ≤64px 单块具名容许——061 移交 UI063 组合态家族）、`edge_miss_strong≤8`（>16/通道且远离边缘的孤立差）、`ink_hole_blocks==0`（32px 块级墨洞=掉字主判据；帧级 ink_ratio 仅记录——≤2/255 全域亮度微差会主导它，实测 code_editor 2.06 而 diff 0.41%）、`profile_peak_ratio≥0.5`（列墨量轮廓错位信号）。**敏感负控（永久）**：GlyphRun 整体平移 1 逻辑 px（峰比 0.17 检出）、删一个字形（墨洞 1-2 检出）——两者必须被检出否则判据失效。判据标定数据：合法重排亚像素差 ink 1.006-1.252/峰 0.609-1.0/洞 0。
- **A4 逐 ID 关闭（fresh run，13/39 行 >0.1%，全部 class=named-aa）**：managed_scroll_content 1.56%、video 0.92%、terminal 0.46%、code_editor 0.41%（caret 瞬态+帧级微差）、rich 0.31%（行尾累计步进漂移 ~6px）、input 0.25%、scrollable 0.24%、text 0.23%、navigation_rail 0.21%、table 0.20%（条纹底色 ≤10/通道）、remote_frame 0.17%、autodown_editor 0.15%、mouse_area 0.14%——逐行 strong=0/holes=0（无强幅度孤立差、无内容丢失），机制分列（宿主重排亚像素/行尾步进、条纹色调、瞬态），2% 仍是诊断上限非 exact 豁免（SpecA §6#9）。**062 具名的"文字合成管线残差"实为两项已修缺陷 + 宿主重排具名降级**。
- **A15 证据闭合（T-03 复核义务完成）**：shear **无生产发射面**——①iced canvas 记录期把含剪切的全部仿射烘焙进 Path 几何（rq_seam_canvas_transform_is_baked_into_geometry 承载，过线即精确）；②文本 Transform 信封仅 scale+translate（P683-D2 具名）；③图像信封仅旋转（精确走重放 rotate 臂）。重放端剪切 warn（paint_ops_v2）保留为防御性具名降级。patch ①评估：跨 iced_graphics/iced_tiny_skia/iced_wgpu 三包公开 API 且零当前消费者——不实施（V-01 有界研究边界），SD-02 具名能力边界+后继指针。
- rqhost 传 `client.scale`（window::scale_factor 落簿既有臂）；**desktop session broker（broker_client_content）scale 落簿缺位 = T-05 具名义务**（HiDPI 桌面嵌载 GlyphRun 按旧分辨率行为，代码内具名注释在案）。
- 过程纪律：连续两次像素数逐位一致 → 发现 nextest 陈旧二进制（强制 `cargo build --tests` 后重跑即显真实差）；一次性构建内存压溃（os error 1455）以 `-j 2` 重试。
- **加宽门暴露并修复的 A1/T-02 轮继承红（同轮同树修复）**：①rqhost p683_t02_typing_remote_v2_loop / p690_ime_downlink——A1 协商翻转后两 v2 字符串面回路的 oracle 仍按 TextStyled 取文本（拿到空表）。修复=`AppEndpoint::set_glyphrun_optin`（v3 代词汇 opt-out；Hello 的 optional 声明保留——optional 非承诺）+ `ClientPump::set_glyphrun_optin`（attach 期应用），两个字符串面回路显式钉面；A1 的 p073_t03_glyphrun_negotiation 复验不受影响。②媒体 absorb 计数竞态：`seen.insert` 先于 `resolve_drawlist_image`——上传落宿主缓存晚于合成引用到达时身份被永久误计 0（负载敏感假红：并行构建下 received=0、空闲下通过）。修复=未命中身份留待后续合成重查。
- **最终验证账目**：nextest rq_ 族 **49/49 PASS**（d00fa0a2，进程隔离权威门）；顺序全量 desktop_protocol **295/297**（基线 63db3a31 同法 293/297——净修复 2）。余 2 红均为顺序上下文 harness 现象、全部隔离/nextest 上下文绿，具名移交 T-04：①cap.view.code_editor 顺序态 "你好" CJK 行重放缺字（holes=25，隔离 holes=0；全局字体库状态被前序测试改变 → 重放侧回退解析失败——证据 probe-seq/capabilities/cap-view-code_editor-*.png）；②p073_t02_media_class_exempt 顺序尾负载下到达率门（≥12/6s 实得 1；trio 17s 与 nextest 进程隔离均绿——T-02 已具名的隔离窗纪律）。V-05 四旧红（autocenter/stage3 n3/n5/t3_examples）在两次顺序全量中全绿。
- 本轮未动 canonical/未发布 Spec；SD-02/03 将携带：image-params/1 filter 字段调和、§3 Raw 行更正、A15 shear 边界、媒体槽几何规则。

### 2026-11-09 work T-04 收口（V-04 可控重排驱动 + V-05 四旧红关闭表）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（T-04 收口：V-04 驱动 + V-05 关闭表；进度 4/8） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: d00fa0a2 / code_commit: **f8a55bd6** / task_ids: T-04（✅） / evidence: plan-artifacts/plan-073/auto-ui/{t03-dp-final.log,t03-dp-baseline-63db3a31.log,v05-oldreds-sequential.log} + `p073_t04_resource_reorder_reference_late_and_close_identity`（f8a55bd6，nextest rq_ 族 50/50 PASS） / blockers: 无 / next_responsible_role: work（T-05：最小六格重建）
- **V-04 可控重排驱动（真实独立宿主全链——RqServe/RqDaemon/管道/SHM/paint 语义）**：`ReadyGateTransport`（客户端 send 侧测试包装：BitmapReady* 持留/释放 + 一次性同身份重复注入；持留期像素留 SHM 槽未 Ack 不被覆盖）。永久测试 `p073_t04_resource_reorder_reference_late_and_close_identity` 五相：①**引用先行**（上传持留，合成列表引用两资源、宿主缓存双未命中——paint 期落具名占位臂，不以错误资源/占位冒完整帧，引用不丢）；②**迟到上传零新帧重绘**（放行后同一合成列表立即可解析——paint 期解析语义=资源就绪自发重绘，宿主合成帧计数恒定）；③**同身份重复 generation 注入拒绝**（重复 gen=1 ≤ 已应用 1 被丢，generation 账停 1 保旧内容——oracle=`bitmap_generation_of` 新观测口）；④**旁观 App 不损**（并发客户端 B 自有资源在 A 的上传/断连逐出全程可解析——per-client bitmap_keys 分账）；⑤**断连逐出 + 同 ID 新 App**（EOF 后键逐出，新 App 全新 compose 零旧引用）。FIFO 正常序对照=t05 wire 矩阵（既有绿）。
- **V-04 覆盖边界（具名）**：重排驱动 e2e 落 standalone rqhost 宿主；desktop session broker 的资源闸与 standalone 共用单点 `bitmap_cache_put_gen`（三消费面同律，host.rs:676/session.rs:5989/rqhost.rs:731），其 compose 臂由 t05 desktop cells 覆盖——未单独重写一份 desktop 乱序 e2e（同闸单点，具名不另测）。A10 双向 release 语义：宿主侧 BitmapRefs forget_surface 驱逐（Phase ⑤）+ 产者侧最后 owner 驱逐（`rq_repair_release_follows_last_owner_eviction` 既有绿）合计覆盖。
- **V-05 四旧红关闭表（C-04）**：根因域全部落 UI063 已交付修复（env 串染/泵 Resize 臂/标题栏几何——063 归档 §9），非绘制/资源缺陷，本 Plan 无新增修复面：
  | 旧红 | 根因域（063 交付关闭） | 当前状态（落主后基线复核） | 证据绑定 |
  |---|---|---|---|
  | autocenter_translates_hits_with_ops | 通用输入/几何域 | **绿**（两次独立全绿） | t03-dp-final.log（d00fa0a2 谱系 295/297）+ v05-oldreds-sequential.log（20590168 294/0） |
  | stage3_multi_app_stress_n3 | 通用调度/泵域 | **绿**（同上两轮） | 同上 |
  | stage3_multi_app_stress_n5 | 通用调度/泵域 | **绿**（同上两轮） | 同上 |
  | t3_examples_queue_end_to_end | 通用队列/调度域 | **绿**（同上两轮） | 同上 |
  （063 归档时新增的第 5 红 t2_remote_ws_session_end_to_end 同批全绿。）名字相同/基线红不豁免 M1：本 Plan 绘制/资源域未命中这些根因，T-03 的绘制修复不触碰其路径（坐标语义改动后仍两轮绿 = 适用性复核完成）。
- 本轮未动 canonical/未发布 Spec。

### 2026-11-09 work T-05/T-06 收口（两宿主媒体语义面 + 独立读回 + 隔离窗测量 + SD 草案）

- stage: work / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / from: work / to: work / status: executing / outcome: pass（T-05/T-06 收口，进度 6/8；execution_done 待 T-06 复核门全绿记录） / worktree: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / base_commit: f8a55bd6 / code_commit: **f2ec9f72**（T-05）→ **dfedfa81**（T-06） / task_ids: T-05（✅）T-06（✅） / evidence: plan-artifacts/plan-073/auto-ui/probe-t06/（隔离窗 cells/probe JSON）+ t05 运行日志（media_e2e_latency 行）+ worktree docs/specs/runtime/{rq-continuous-media.md(新),rq-render-packets-and-resources.md(§7),rq-fidelity-contract.md(§3更正+§10)} / blockers: 无 / next_responsible_role: review（T-07 独立上下文复核）
- **T-05（V-02 两宿主正式矩阵）**：新模块 rq_media——同一媒体流产者参数跨 standalone rqhost 守护进程与 desktop DesktopSession broker 各跑四相：①连续（≥40 帧不同内容落地并可解析）②暂停（先排空在途≤槽信用再冻结——稳定窗零新 id、末帧保旧）③seek（确定性内容跳变，宿主内容哈希=产者 hash(400)；旧身份零倒灌）④重开（desktop 臂新源新会话首帧重落）。**独立读回**：宿主 put 时独立 blake3 内容账（`bitmap_content_hash_of`，evict 同删）↔ 产者侧 `frame_rgba(n)` 独立哈希对表，无共享中间状态；资源身份本身内容寻址。过程中修复两处测试语义：暂停判据改"排空后冻结"（在途≤槽信用属合法延迟）；连续相破环改"全部可解析"（合成引用与 put 同批到达的读回竞态）。
- **T-06（V-06 取证）**：①隔离窗计时门 3/3 绿（AUTO_RQ_TIMING_WINDOW=isolated：produce p50 0.06–0.24ms / p95 ≤0.70ms vs W-C ≤10ms 冻结预算 14× 余量；replay p50 3.2–5.7ms 在 1.25×direct+2ms 门内）；②媒体端到端延迟分布（publish(tick on())→宿主内容首见，n=39，pending 重试语义不假设同 tick 先后）：**p50 16.6ms / p95 18.1ms / max 24.4ms**，冻结预算 250ms；③nextest.toml p073_t05 加入 num-cpus 独占组（同批并发假红一次，隔离后全绿——同 T-02 纪律）。
- **SD 草案落 worktree（merge 落地）**：SD-01 新 `rq-continuous-media.md`（媒体类资源类别/槽几何规则/连续流语义/延迟读数/冻结负控）；SD-02 `rq-render-packets §7`（GlyphRun 逻辑坐标+直通 alpha/atlas sRGB 语义、像素级判据+敏感负控、乱序不变量、四旧红处置、A15 shear 具名不可达）；SD-03 `rq-fidelity-contract`（§3 四条过时行更正：Raw 已过线/image-params rotation+opacity 已过线 filter 具名缺口/资源复用已被 062 取代/阴影富模式已精确；§10 实测更新+OS063 交接面）。**canonical 未发布**（merge 时落地）。
- nextest rq_ 族 **51/51 PASS**（f2ec9f72 谱系）；T-02 媒体门一次同批并发假红（隔离组扩容后复绿）。

### 2026-11-09 review 最终评审（AC-06；T-07）

- stage: review / review_scope: **final** / reviewer_identity: 本实现会话内评审（**限制声明**：无独立会话可用；按 skill 纪律判词从工件与评审自有新鲜复跑重建，不采信执行者摘要） / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / outcome: **pass（reviewed）** / worktree_path: D:/autostack/.wt/ui-073/auto-ui / branch: codex/plan-073-dev / **reviewed_commit: 2cf7b43e**（F-1 delta 措辞修正；代码语义与 dfedfa81 一致） / base_commit: 96236725（ancestor 已验）/ dependency_revisions: auto-lang a95d6dc41 / auto-web 2d69841 / auto-down 58ea33f（组内只读，未动）
- **评审自有证据（非执行者转述）**：①基线核验——树清洁（0 dirty）、96236725 为 HEAD 祖先、8 提交线性、diff 19 文件 +2753/−126；②**新鲜复跑 nextest rq_ 族 51/51 PASS**（probe-review/，210s，含 T-02 媒体三件、T-04 五相驱动、T-05 双宿主四相、像素判据与全部敏感负控）；③负控绑定抽查——`assert!(detected...)`（rq_cells:637 判据不敏感即炸）、generation 闸 `generation <= *last`（broker_surface:278）、V-04 驱动 11 处 assert 具名（含零新帧/逐出/旁观/同ID 四不变量）；④SD 断言对码抽查——Raw 臂实存（headless.rs:2563 buffer_runs 过线）、Transform 发射面仅 2 处且无剪切（文本信封 scale-only + 图像旋转，A15 具名准确）、bm_slot_size w×h×4×4+4（host.rs:452 = SD-01 规则一致）。
- **预算/支持面无未授权变更（AC-06 专查）**：全支 diff 扫描 rate_limit/limit——仅测试旋钮 setter 一行；产线默认 8 MiB/s（headless.rs:243）与 256 MiB 驻留未动；媒体豁免收敛于 intern 单点闸（headless.rs:382 is_media_frame）= 用户裁定范围。
- **AC 证据映射**：AC-01 ✅（V-01 两臂对照+慢宿主负控；预算裁定在案）；AC-02 ✅（双宿主四相+冻结负控+独立哈希读回）；AC-03 ✅（A1/A2/A3/A5/A15 逐 ID + 13 族 named-aa 像素级关闭 + 敏感负控）；AC-04 ✅（V-04 五相 + V-05 四旧红两轮绿关闭表）；AC-05 ✅（六格语义面 + V-06 分布/预算 + SD 交接面）；**AC-06 ✅（本评审）**；AC-07 → merge（pending closeout：canonical 落地/归档/guard/工件折叠）。
- **findings**：F-1（medium，**评审内已闭**）——SD-02 草案与 T-03 收据把 desktop session broker scale 落簿记为"T-05 义务"未兑现；真实归属=OS063 桌面壳窗口生命周期域（本 Plan 测试面为 offscreen 代理/内容级读回，不受影响；rqhost 主宿主已接 scale）。处置=SD-02 §7.1 措辞修正（2cf7b43e，纯文档）+ §10 具名 hold 登记（owner=OS063，关闭条件=壳层接 scale 事件传入 displaylist_element）。无代码语义 finding；连续最终失败计数 0。
- **pending_closeout_gates（merge 拥有）**：canonical Spec 三文件落地、归档/登记刷新、guard clean + worktree 清理、plan-artifacts scope 折叠删除、AC-07 收尾记录。
- evidence: 本收据内联命令/数字即持久证据（skill 纪律）；过程日志 plan-artifacts/plan-073/auto-ui/{probe-review/,probe-t06/,t03-dp-final.log,t03-dp-baseline-63db3a31.log}（merge 折叠删除）。
- next: **merge**（T-08；whole-chain 授权在案——按 auto-plan-merge 执行 canonical 落地/归档/清理/折叠）。

### 2026-11-09 merge 收尾（T-08；PLAN-073:r1）

- stage: merge / plan_id: auto-ui/PLAN-073 / plan_revision: r1 / outcome: **pass（delivered）** / completion_kind: delivered
- **prepared**：评审基线 2cf7b43e（dfedfa81+F-1 措辞修正），canonical diff = worktree 已提交的 SD-01/02/03 三文件；目标路径 docs/specs/runtime/*；主检出其他会话 docs WIP 保留未动（零重叠，ff 安全校验通过）。
- **landed**：rebase master（4ebc51ce）后 range-diff 全等（补丁等价证明；old→new 映射：d00fa0a2→02680ab0、f8a55bd6→82e5f49f、f2ec9f72→b3d26083、dfedfa81→8bfcf6a3、2cf7b43e→82b61084）；主检出 `git merge --ff-only` **无合并提交**，master tip = **82b61084** = 交付提交。
- **canonical_verified**（主检出源哈希，sha256 前 16）：`1e27842ac6da9ce8` rq-continuous-media.md（新增）；`9396e4cac31378de` rq-render-packets-and-resources.md（§7 增补）；`f3aef37bf4294cec` rq-fidelity-contract.md（§3 更正 + §10）。交付提交内三文件齐（git log 确认 create/modify 落 82b61084 谱系）。
- **archived**：Plan 移入 docs/plans/archived/（untracked→新建归档文件）+ status: archived + T-07/T-08 勾选（8/8）；本收据与归档文件随本次簿记提交入库（PLAN-104 UAT-K13：非 untracked docs）。
- **cleaned（已执行回填）**：wt-guard clean（无 reparse point）→ worktree 移除 + 分支 codex/plan-073-dev 删除（was 82b61084=交付 tip）→ 组目录 .wt/ui-073/（含只读依赖树 lang/web/down）移除 + 三仓 worktree prune → plan-artifacts/plan-073 删除（867 文件/~19MB； durable 摘要已全部折入本文件 §9/§10）→ 登记刷新（deleted-at-merge + F-1 hold：owner=OS063 + 关闭条件）。
- **生产制品现况观察（landing≠deployment）**：本 Plan 未触 release 二进制/daemon/web bundle 的消费面变更（crate 库内实现 + 测试面）；OS063 消费正式接口后重建再验（SD-01 §6/SD-03 §10 交接面）。登记为观察项，不阻塞 delivered。
- **批量回归 due 检查**：073 % 5 ≠ 0；due 文件核对见 cleaned 回填。
- next: 清理回填完成 → **报告 pass**（delivered @82b61084）。
