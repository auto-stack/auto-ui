---
plan_id: PLAN-045
status: archived
feature_name: B3媒体门户与播放器三教学示例主题迁移
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md, docs/specs/ui/specialized-theme-migration-readiness.md]
new_spec_components: [docs/specs/ui/demo-theme-batch-045.md]
touched_goals: []
affects: [019-video-app, 020-music-player, 030-video-player, teaching-demo-theme-migration]
depends_on: [auto-ui/PLAN-036, auto-ui/PLAN-038]
---

# [auto-ui/PLAN-045] B3媒体门户与播放器三教学示例主题迁移

## 0. 变更摘要

接受038实际研究结果，为**019-video-app、020-music-player、030-video-player** 3个教学Demo制定完整主题迁移合同。补正式包/声明/公共和有据专用表面、保原教学业务与负控，先核研究具名阻点。不是同名auto-xxx产品迁移；后者归OS055–061。

drafting r1、0/9。2026-11-04用户授权UI/lang后继规划，本轮仅调研/立约/路线刷新，暂不实施；无树/探针/GUI/代码/产品或OS pin/构建测试/远端动作，未创建或联系其他agent/chat。

## 1. 目标

- G1：本批全部3例同一正式包、族/模式/既有默认/accent/宿主策略正确，两个族×明暗实际可见，不仅改类名。
- G2：原教学业务、数据/内容/时序/性能与错误恢复完整适用义务保持；活跃态换肤不重建业务，不改内容身份。
- G3：038具名原红/未支持/条件排除有当前反例、owner和关闭结果；必需支持腿未闭不能以“有界go”或旧原红冒完成。
- G4：实际来源/生成/复用/隔离与退出有据，给准确已迁数量和剩余资产/外仓交接，不把Demo通过抵产品。

非目标：复制真实产品实现、补全新媒体/编辑/数据库平台、新.at语法、通用状态/Reload/Service、重做036发行、SG/ACE全线、全量AG/机器缓存或所有历史债。

## 2. 架构方案

036正式包/已验证输入→现Demo pac/use/design→031/033 scope/现模型事件→Vue/VM与原专用控件→实际像素/模型。保当前教学支持场景/固定数据与明确资源条件，公共壳、分类/品牌色、文档/媒体内容分别有owner和豁免依据。

沿已存在场景迁公共壳/设置/播控、同一themes包。019门户、020音乐、030原生视频的场景与能力不能互相替代。小媒体在scope，禁止读用户Music/Video；内容像素/封面/viewport不被主题染色。现切换事件只有源码无视图接线时按原承诺修必要连接，不强装新菜单或功能。

## 3. 技术栈

现.at pac/front/store/API、theme数据与scope、stylekit/实际专用组件，原MCP/Playwright/fixture与受管生成/后端。T-01从当前source/help冻结精确入口，不另造生成器/包管理或通用测试系统。

## 4. 需求分析与背景调查

已用auto-plan-new/verification-contract，读UI主题/包/生成输入Spec、[038实测canonical](../specs/ui/specialized-theme-migration-readiness.md)、038归档、036/037最终收据及[框架接续设计](../design/autoui/instance-state-and-reload-stage-one.md)。UI观察HEAD 92ee2459a1058ddddc3fc3378ecec6585ffe726c；研究结果是来源，不是本计划新执行证据。

| 目录 | 038实际结果/具名差额 | 真实验证定位（参数工作重核） | 业务守恒oracle |
|---|---|---|---|
| 019-video-app | 038 VM设置面PASS、主题Δ153，Vue e2e未跑；五accent if链仍是主要映射面 | tests/vm-smoke.mjs、test_019_vm.py、smoke.spec.ts与原acceptance | 分类/搜索→详情/like→返回、设置族/模式/accent，路由与已有门户数据保持；它不是030原生播放器 |
| 020-music-player | scan/stream在back-port覆盖下实证；旧测试links归属冲突、播放仍blocked，ToggleDarkMode无视图绑定 | tests/test_020_vm.py、实际scan/stream与合法小媒体入口 | 原队列/选曲/播放时间/音量、空库/失败诊断；真实播放支持未闭不能以scan或假进度代通过 |
| 030-video-player | 原VM降级门/静态style门通过；两轨media API无back contract、播放blocked | tests/vm-smoke.mjs、smoke.spec.ts、assert-style.sh与原native条件 | viewport/队列/OSD/seek/音量/原快捷键保持，降级与真实播放分列，viewport黑/透明为内容域 |

T-01重放来源links、030真实back contract/路由、mpv与合法fixture支持。依原承诺必需的媒体腿先由实际生成/服务owner闭合，不能用降级页面抵播放；未承诺的新格式/引擎不纳本包。命中930同一来源根因的腿接受其正式修复，其他工作不整体等930。 原合同要求双端/原生的必需面不因pac render或研究未测而自动删掉；真正原先未承诺的轨不能新增成无限平台补全。T-01逐例冻结支持表/原样本/原条件排除，冲突先具名重规划，不用研究的有界go降低既有承诺。

036/038已归档，实际源/工具/包版本仍须开工重核。041状态与042 Reload不是本批统一前置；仅真实违反已承诺实例/生成契约的消费者按owner交接，不能借其WIP。930只收生成交付/来源守门，不自动承担所有图像/媒体类型编译问题，真正命中共同根因的腿接正式结果。

## 5. 详细设计

### 5.1 启动与下一任务

尚未获实施授权。拟唯一树D:/autostack/.wt/ui-045/auto-ui，分支codex/plan-045-demo-themes；需实际参与的语言/Man/Web/Down各一固定兄弟树/owner，不建链接。实现/独立review同树顺序，冻结写入后review，不复制第二实现树/不改产品clone。

T-01读实际源/工具/包/原门/支持、触面WIP和资源窗，先对上表具名阻点做有界真实正负控制，再定最小修复/现主题接线。涉及新语义/公开支持/新后端需精确owner/正式前置，不能无限扩大；同批其他无依赖项可先推进，整批验收必须全部必需面闭。

### 5.2 迁移、实例与资源

逐字段冻结公共role/交互状态、合法分类/内容色、accent overlay和字面豁免；现无主题面例补包/design/已声明选择字段，保持既有默认/旧CLI-pac优先级及原handler，不强加新业务菜单。媒体/文档/图形数据不被重染，已声明可主题化内嵌表面须实际消费，不用空页面/假viewport或只截图外壳。

本批每例真实运行两族×明暗及原声明支持轨，原业务活跃态中换肤→继续操作→原条件下退出/重开；fixture与实际可见值/业务数据双oracle。两实例相反选择、子scope/路由保持、非法包/模式拒绝保旧、合法恢复及owned退出均具名。现消息模型/数据/时间/样本不因主题迁移更换；原测试漂移修定位要给反例并保真正原义务，不能直接删红断言。

资源仅scope内小合法媒体/图像/数据库/文件/编辑数据，禁止真实用户目录或杀旁观进程；原性能/压力/平台条件保留且分别记录，主题无性能改动不机械重跑无关重档。实际必需输入/GPU/原生窗口和预算T-01冻结。

### 5.3 来源与后继

以036已落地渠道验证正确包/身份输入、同mtime合法改包生效/篡改拒绝/恢复、外部fresh/locked/第二根与910 member/context/target/lease；无主检出兜底或旧target oracle。仅所选例的现已承诺路径，不冒全部格式/平台全绿。

本批交付后只为本批目录逐例记主题迁移；B1/B2既有交付及其具名原红保持，B3其他批不重复。032/remote-frame/stylekit等共享/验证资产保原义务。真实产品完全独立演进，OS对应应用仍需其独立合同；UI041/042的更大能力、Tauri R-6、Web/Man其他债不因本包闭合。共享renderer/输入/锁短窗单写，必要真实改面才增参与仓。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/demo-theme-batch-045.md | 本批逐例主题/默认/内容豁免/原业务与支持轨实证 | Demo有自己的验收 | AC-01..04 |
| SD-02 | modify | docs/specs/ui/theme-app-migration.md | 仅按实际全部必需面闭的本批目录更新数量，其他未迁仍在 | 守恒不假全量 | AC-05 |
| SD-03 | modify | docs/specs/ui/specialized-theme-migration-readiness.md | 038具名差额按本次真实修复/前置结果接续，原研究历史保留 | 不把go冒迁移 | AC-03/05 |
| SD-04 | modify | UI现生成/组件/主题Spec；必要语言owner页仅真实差额 | 有限既有承诺修复/实际owner/兼容支持 | 不下沉UI算法 | AC-03/04 |

## 6. 测试设计

全部未来验证；T-01冻结准确命令/支持/输入，T-02..05使新反控可运行。work自查/同树独立final review分开，review不修代码。执行真实双端时先读autoui-verifier。原门/原样本/条件不减，父exit0不代子终态。

| Case ID | 不变量/输入 | 实际入口与区分性oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01；每例两族×明暗/旧默认与accent | 原真实Vue/VM及声明native入口；实际公共值来自同包且状态/主题优先级正确，非source grep/旧Demo截图 | work |
| V-02 | AC-02；上表业务活跃态换肤/继续/恢复 | 原完整适用门与实际handler/后端/文件/内容oracle，不重建/丢数据，原样本/时序/性能条件守恒 | work |
| V-03 | AC-03；038具名首红/阻点、合法对照 | 冻结实际输入/工具/轨逐项复现→修既有面或具名前置→同反例重验；必需未闭不得以旧归因PASS，测试漂移不删义务 | work |
| V-04 | AC-01/02；相反实例/子scope/新路由/内容哨兵 | 真实渲染与业务读取，切一方不串另一方、公共面变内容不乱染，原共享选择保持 | work |
| V-05 | AC-04；U036来源、坏包/缺件/同mtime合法改包、外部第二根与恢复 | 真实产物/输入receipt/颜色对应，坏包拒绝保旧/恢复再消费，910隔离保持，无旧target/主检出补源 | work |
| V-06 | AC-02/04/05；错误/关闭/资源/支持与数量交接 | 只回收本次owned树/资源，原错误可见、合法重开继续；条件排除/未测/产品独立与其他批准确 | work |
| V-07 | AC-06；逐AC/各例真实支持/产物/SD与遗漏 | 同树冻结独立review重建，不拿038研究/机制代表例抵本批全部必需面 | review |
| V-08 | AC-07/08；冻结职责/真实落地/知识/全输出 | 被测==交付source，canonical/路线/归档真实读回，摘要/guard/资源材料清理 | merge |

纯Demo/assets/测试不跑语言Cargo/all/docs_gen；真实UI Rust/生成/renderer改动按完整受影响职责及原消费者，UI t/tu单进程/必要ti按触面。语言共享生成/类型真改由实际owner冻结必要固定组，不自动跑ACE/AG。final后冻结必要并集一次，输入未变证据可有据复用；准备/编译/发现/运行/清理时间、执行/原排除/首红/重放/未测分列。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 本批3例同正式包、两族明暗/原默认/品牌及声明轨真实通过，V-01/04 | work |
| AC-02 | 原教学业务/内容/数据/交互/资源与完整适用义务保持，活跃换肤/错误恢复不退化，V-02/04/06 | work |
| AC-03 | 038具名阻点/原红按冻结反例有据闭合或先行owner前置正式交付；必需未闭不pass、不静默删轨，V-03 | work |
| AC-04 | 外部来源/生成/失效/拒绝恢复及owned生命周期/910规则真实成立，V-05/06 | work |
| AC-05 | 本批与其余Demo/真实产品/041/042/930边界与实际数量/未测交接准确，V-06 | work |
| AC-06 | 同树独立final review逐AC/健康/遗漏/SD绑定冻结源pass，V-07 | review |
| AC-07 | 冻结门/实际参与仓落地/canonical/路线/归档同源读回，V-08 | merge |
| AC-08 | 全输出/owned资源对账，摘要折Plan后删scope/树产物、guard clean，持有具名，V-08 | merge |

## 8. 执行步骤

| ID | 动作/可验收结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | 授权后唯一组/源/包/tool/旧支持/准确命令资源冻结；上表阻点最小复现/owner与合法控制 | U036/U038；实施授权待给；AC-01..05 | work |
| T-02 | 本批既有承诺的必要小接缝/测试定位先闭，需新能力的精确前置正式交付后接受；不做无界新平台 | T-01；AC-03 | work |
| T-03 | 本批所有例正式包/design/公共role/已支持专用端口迁移与内容豁免、旧默认保持 | T-02；AC-01/02 | work |
| T-04 | 真实支持轨逐例两族明暗/完整原业务/隔离/错误恢复/owned退出 | T-03；AC-01..03 | work |
| T-05 | 外部来源/输入失效/移根/拒绝恢复与910门、实际候选source冻结 | T-04；AC-04/05 | work |
| T-06 | 健康/遗漏/原门守恒/SD/其他批与产品交接冻结；execution_done6/9 | T-05；AC-01..05 | work |
| T-07 | 同树独立上下文final review，finding回work并失效相应证据 | T-06；AC-06 | review |
| T-08 | 冻结必要职责并集、各实际仓合入、canonical/路线/归档同源读回 | T-07 pass；AC-07 | merge |
| T-09 | 全输出/资源对账、摘要折Plan删统一scope/自有树，guard后移除/分支/组；archived9/9 | T-08；AC-08 | merge |

材料唯一D:/autostack/plan-artifacts/plan-045/auto-ui/，参与仓按repo登记owner/用途/在途。探针/小媒体/文件/截图日志/外部consumer/显式target仅scope，永久回归fixture在正式tests有据，普通构建在树。不依旧脚本默认tmp/项目内证据。merge核全部实际路径/进程，命令/退出码/关键值/源hash/未解条件折Plan后删scope/树产物；仅具名持有有owner/理由/关闭条件，必需面不借hold过关，未知/活动/他方不盲删，无旧JSON写入。

## 9. 复审记录

2026-11-04 / stage:new / auto-ui/PLAN-045 r1 / outcome:blocked（仅规划，实施授权待给；§4具名能力门先核） / 已接受038真实研究而非复制旧源码盘点，完整列本批3例/原门/已知阻点和owner/资源/独立产品边界；全部本计划探针和测试未执行。next：授权后T-01，不能将有界go/旧红说明换成必需面PASS。

静态预检pass（2026-11-04）：0–10章节、9T/8AC/8V/4SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/探针或独立final review；全部实现/运行仍待授权work。

2026-11-04 / stage: work | plan_id: PLAN-045 | plan_revision: r1 | outcome: pass（T-01 冻结基线） | worktree: D:/autostack/.wt/ui-045/auto-ui | branch: codex/plan-045-demo-themes | base_commit: 9bf9874a | dependency_revisions: auto-man@2bbd954（auto.exe 原配冻结）、auto-ui-tool.exe（11-03 原配冻结） | task_ids: T-01 | evidence: plan-artifacts/plan-045/auto-ui/t01-baseline.md（worktree/分支/工具/命令/媒体根冻结：auto build、vm-smoke、test_019/020_vm.py、assert-style.sh、playwright e2e；E:\Music/E:\Video 实测存在）；基线门全绿——build-019/020/030-baseline exit 0、vmsmoke-019/030-baseline PASS、test019/020-baseline PASS（020 增量）、assertstyle-030-baseline PASS；038 具名阻点逐项复现（020 ToggleDarkMode 无视图绑定实测 None、030 两轨无 back contract、019 vue e2e 未跑、links 归属冲突） | blockers: 无 | next_responsible_role: work（T-02）

stage: goal | plan_id: PLAN-045 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-045/auto-ui | commit: 9bf9874a（基线） | next: work（T-02..T-06；授权=goal`完成计划045`全链，后续轮次继承不再询问）

2026-11-04 / stage: work | plan_id: PLAN-045 | plan_revision: r1 | outcome: pass（T-02..T-05 全链） | worktree: D:/autostack/.wt/ui-045/auto-ui | branch: codex/plan-045-demo-themes | base_commit: 9bf9874a | code_commit: 0c9b1ad2 | dependency_revisions: 冻结工具期（T-05 后 worktree 工具被替换为新工具，见既有红交接） | task_ids: T-02,T-03,T-04,T-05 | evidence: 逐任务行见§8（T-02：4801d414 SwitchTheme msg 修复——子 msg 与回调 prop 折叠同键致 VM 双发翻两次，改名错开、Vue emit 名不变；0c9b1ad2 020 VM 测试按真实 UI 文案修定位——links 归属冲突=旧定位文本与 UI 不符，未删红断言，test020-fixed.log ALL PASS。T-03：6ae24936/969b3ce7/3d9e59f1 三例 dep themes + use themes + theme_name + design 块迁移，019 主题魔法变量自 store 上移根 widget，030 新增最小 back contract /api/player/status；build-019-migrated exit 0 + theme receipt 9fa07742dc68、020/030-migrated exit 0、030-back exit 0。T-04：vmtheme 4 fixtures×3例 ALL PASS、vuetheme-019/vue-020 PASS、e2e-019-final 10/10、e2e-030-back 14/14（含真实播放/seek/4K/错误文案）、play-020-v2 真实播放 ALL PASS、empty-020 空库诚实诊断、isolation-019-020 双实例隔离 ALL PASS、assertstyle-030-migrated/vmsmoke-019/030-migrated PASS。T-05：篡改负控——tampered（原配工具缺件 receipt skipped 不拒绝，证明需 036 完整准入）+ tampered2（新工具篡改 themes.at → exit 1 具名拒绝 sha256 不符）；恢复 hash 329C2D56…==原版；来源冻结/910 隔离证据 plan-artifacts/plan-045/auto-ui/t05-source-freeze-and-isolation.md） | blockers: 无 in-contract finding；1 项具名既有红已交接（见 T-06 行），不阻塞 review | next_responsible_role: work（T-06）

work-handoff readiness：work-owned AC-01..05 全绿且绑定当前证据（§8 T-02..T-05 行）；被测源==工作树源（终态证据 e2e-019-final 21:03/vuetheme-019 21:07 均在最后 019 提交 4801d414 之后）；两连败诊断不触发。

2026-11-04 / stage: work | plan_id: PLAN-045 | plan_revision: r1 | outcome: pass（T-06，execution_done 6/9） | worktree: D:/autostack/.wt/ui-045/auto-ui | branch: codex/plan-045-demo-themes | base_commit: 9bf9874a | code_commit: 1bb48327（SD-01..03 草稿） | dependency_revisions: auto-man@2bbd954 | task_ids: T-06 | evidence: 健康=残留 auto/auto-ui-tool 进程与 3019/3020/3030/8019/8320/8330/9419 端口全清；遗漏=TS1005 既有红具名交接（auto-ui-tools vue.rs generate_main_ts 拼接缺陷，plan-041 store_ctx_import 模板缺换行；主检出未迁移 019 同复现 build-019-master-main→main-second，020/030 新工具 build PASS；owner=后继生成器修复批次，merge 不做重建，045 证据用冻结工具 build + 终态 e2e 不受影响）；原门守恒=基线 vs 迁移后门对照全绿（vmsmoke/assert-style/VM 测试/e2e 均保持或按真实文案修定位，无删红）；SD-01 新建 demo-theme-batch-045.md（逐例接入/默认/豁免/守恒/交接）、SD-02 theme-app-migration.md（第五批 045 + 计数 22→25、B3 余 9）、SD-03 readiness 接续 §5（019/020/030 阻点闭合结果，038 历史保留）；其他批/产品交接=046/047 覆盖 B3 余九例（其静态预检已记 045–047 恰覆盖十二例）、OS055–061 产品谱系独立 | blockers: 无 open in-contract finding | next_responsible_role: review（T-07 同树独立上下文 final review）

stage: goal | plan_id: PLAN-045 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-045/auto-ui | commit: 1bb48327 | next: review（T-07 同树独立 final review，冻结源 1bb48327）

2026-11-04 / stage: review | review_scope: final | reviewer_identity/limitations: 独立上下文=同会话委派 fresh general 子代理（general-1，逐 AC 从源码+证据日志+自建核对独立重建判定；未重跑重型 e2e/VM/播放——按冻结工具证据+时间序采信；未改任何文件） | plan_id: PLAN-045 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-045/auto-ui | branch: codex/plan-045-demo-themes | reviewed_commit: 1bb48327（树 clean 复核，除未跟踪探针 plan045-020-play.cjs=scope 测试工件） | base_commit: 9bf9874a | spec_inputs: SD-01/02/03@1bb48327 | acceptance_results: AC-01..06 全 pass——AC-01（vmtheme 4 fixtures×3 例 + vuetheme/vue-020 + isolation-019-020 ALL PASS；三例 pac.at dep themes + 根 widget use themes/theme_name/design 绑定源码核对）、AC-02（e2e-019-final 10/10 exit 0 晚于最后 019 提交、e2e-030-back 14/14 含真实播放/seek/4K、play-020-v2 真实播放、empty-020 空库诊断；diff 仅删主题冗余分支/魔法变量，业务全保留）、AC-03（ToggleDarkMode 接线、030 back contract /api/player/status、test_020_vm.py 按真实文案修定位未删红断言、019 vue e2e 补跑）、AC-04（tampered2 具名拒绝 sha256、恢复 hash 329C2D56… 三方一致、t05 文档诚实）、AC-05（计数 25/余 9、046/047 恰覆盖 3+3+6=12 无重复、产品独立交接）、AC-06（SD-01 新 + SD-02 计数 + SD-03 §5 与证据一致、038 历史保留）；健康/范围 pass（worktree 仅探针未跟踪、diff 恰 21 文件、主检出 WIP 零重叠、045 端口全空）；收据核对 pass（§9 T-01..T-06 与日志逐项吻合、被测源==交付源） | findings: F-1(minor)：`vmtheme-020.log`（20:10）为修复前 FAIL 诊断日志，ALL PASS 属修复后 `vmtheme-020-fixed.log`（20:30）——收据措辞未区分两日志，merge 折档标注；F-2(minor)：`build-019-tampered2.log` 未内联进程 exit code（仅有 [X9] main Err），「exit 1」依 CLI 失败路径惯例——负控语义（具名拒绝、零产物）证据充分不阻塞；F-3(info)：主检出 HEAD 已推进至 024fe57a（043/044/050 已合并），范围断言改用 worktree diff + 主检出重叠检测；F-4(info)：现存 5 个 auto/auto-ui-tool 进程均属他方（ui-046 worktree 027-file-manager VM、auto-man auto-blog 服务器，22:41/22:43 起于 T-06 收据后），非 045 残留，不触碰；F-5(info)：主检出未跟踪 examples/ui/019-video-app/deps/ 为 045 T-05 主检出复现实验产物，属 scope 清理对象 | evidence: 本记录 + review 输出（逐 AC 表） | next: merge（T-08 冻结源 1bb48327 → rebase master → ff-only 合入 → canonical/归档 → T-09 清理）

stage: goal | plan_id: PLAN-045 | plan_revision: r1 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-045/auto-ui | commit: 1bb48327 | next: merge（final pass@r1+1bb48327；T-08..T-09）

2026-11-04 / stage: merge | plan_id: PLAN-045:r1 | reviewer/operator: 主会话（auto-plan-merge skill全流程） | outcome: pass（delivered；cleanup见T-09/closeout commit） | checkpoints: prepared=复审树clean@1bb48327+canonical delta SD-01..03（1bb48327内）+冻结源19ae8e01（rebase后）；landed=worktree rebase 7提交至master 024fe57a→tip 19ae8e01，`git range-diff 9bf9874a..1bb48327 024fe57a..19ae8e01` 全部`=`（逐补丁等价）；主检出 `git merge --ff-only codex/plan-045-demo-themes` → master tip==交付commit 19ae8e01、零merge commit、他方WIP零触碰（main status他方项原样保留）；canonical_verified=12关键文件（SD-01/02/03+三例pac.at/app.at/settings.at/nav_sidebar.at/test_020_vm.py/back api.at）主检出==复审树逐位一致（sha256全OK），packages/themes/themes.at主检出329C2D56…==原版 | frozen_gates: 本批纯Demo/assets/docs，§6明确不跑语言Cargo/all/docs_gen；终态功能门（vmtheme 4fixtures×3例、e2e 10/10+14/14、真实播放、isolation、assert-style、VM测试）由T-04冻结工具证据+review独立采信；**不重建**——canonical读回=文件hash一致（非重新build），019新工具重建=既有红TS1005（T-06交接owner=后继生成器批次，主检出同复现） | environment_notes: F-1/F-2 minor标注——`vmtheme-020.log`(20:10)为修复前FAIL诊断、ALL PASS属`vmtheme-020-fixed.log`(20:30)；`build-019-tampered2.log`未内联进程exit code，「exit 1」依CLI失败路径惯例（具名拒绝+零产物语义证据充分） | evidence: range-diff输出、canonical hash清单（12文件+themes.at）、本记录 | landing_checks: 无共享生产进程/发行二进制消费master的demo源（工具二进制按项目生成期构建）；main 020/030新工具build已证PASS（worktree newtool日志同款新工具）、019新工具build=既有红（已在案，非本merge引入） | next: closeout（T-09 wt-guard→树/分支删除→plan-artifacts删除→registry刷新→cleaned commit）

## 10. 待澄清事项

1. 本轮暂不实施；上述已知能力差额按真实反例与原承诺闭合，若必须新语义/平台给具体owner/前置或重规划，不把批次降为代表两例就归档。
2. 041/042与OS产品迁移不是全局前置；确命中930共同来源根因的腿只接正式结果，不借WIP。未知类型/服务问题不自动丢给930。
3. 每例原双端/原生支持必须按现合同冻结，不偷减，也不将未承诺的新格式/平台无限并入本包；原性能/时序条件保持。
4. 迁移仅教学Demo，实际产品各仓独立验；用户数据/系统设置/旁观进程不动，真实窗口与资源T-01明确。
