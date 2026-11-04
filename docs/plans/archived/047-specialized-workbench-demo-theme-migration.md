---
plan_id: PLAN-047
status: archived
feature_name: B3阅读编辑数据与绘制六教学示例主题迁移
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 6
total_steps: 9
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md, docs/specs/ui/specialized-theme-migration-readiness.md]
new_spec_components: [docs/specs/ui/demo-theme-batch-047.md]
touched_goals: []
affects: [018-book-reader, 041-auto-edit, 026-database, 025-sys-monitor, 024-charts, 031-paint, teaching-demo-theme-migration]
depends_on: [auto-ui/PLAN-036, auto-ui/PLAN-038]
---

# [auto-ui/PLAN-047] B3阅读编辑数据与绘制六教学示例主题迁移

## 0. 变更摘要

接受038实际研究结果，为**018-book-reader、041-auto-edit、026-database、025-sys-monitor、024-charts、031-paint** 6个教学Demo制定完整主题迁移合同。补正式包/声明/公共和有据专用表面、保原教学业务与负控，先核研究具名阻点。不是同名auto-xxx产品迁移；后者归OS055–061。

drafting r1、0/9。2026-11-04用户授权UI/lang后继规划，本轮仅调研/立约/路线刷新，暂不实施；无树/探针/GUI/代码/产品或OS pin/构建测试/远端动作，未创建或联系其他agent/chat。

## 1. 目标

- G1：本批全部6例同一正式包、族/模式/既有默认/accent/宿主策略正确，两个族×明暗实际可见，不仅改类名。
- G2：原教学业务、数据/内容/时序/性能与错误恢复完整适用义务保持；活跃态换肤不重建业务，不改内容身份。
- G3：038具名原红/未支持/条件排除有当前反例、owner和关闭结果；必需支持腿未闭不能以“有界go”或旧原红冒完成。
- G4：实际来源/生成/复用/隔离与退出有据，给准确已迁数量和剩余资产/外仓交接，不把Demo通过抵产品。

非目标：复制真实产品实现、补全新媒体/编辑/数据库平台、新.at语法、通用状态/Reload/Service、重做036发行、SG/ACE全线、全量AG/机器缓存或所有历史债。

## 2. 架构方案

036正式包/已验证输入→现Demo pac/use/design→031/033 scope/现模型事件→Vue/VM与原专用控件→实际像素/模型。保当前教学支持场景/固定数据与明确资源条件，公共壳、分类/品牌色、文档/媒体内容分别有owner和豁免依据。

六例按公共壳/控件、文档/语法/表数据/series/画布内容三层映射；无主题面例补正式包/design声明，已有编辑/绘制引擎走现可用主题端口。没有可用端口先给具名缺口，不能只改外壳就宣称内嵌面全主题化。不迁真实Reader/Database/Charts/Paint等产品，不复制它们新业务/数据。

## 3. 技术栈

现.at pac/front/store/API、theme数据与scope、stylekit/实际专用组件，原MCP/Playwright/fixture与受管生成/后端。T-01从当前source/help冻结精确入口，不另造生成器/包管理或通用测试系统。

## 4. 需求分析与背景调查

已用auto-plan-new/verification-contract，读UI主题/包/生成输入Spec、[038实测canonical](../specs/ui/specialized-theme-migration-readiness.md)、038归档、036/037最终收据及[框架接续设计](../design/autoui/instance-state-and-reload-stage-one.md)。UI观察HEAD 92ee2459a1058ddddc3fc3378ecec6585ffe726c；研究结果是来源，不是本计划新执行证据。

| 目录 | 038实际结果/具名差额 | 真实验证定位（参数工作重核） | 业务守恒oracle |
|---|---|---|---|
| 018-book-reader | 038原门5/10，详情空渲染/进度回写/添加书/ThemeToggle挂载五红 | tests/smoke.spec.ts、acceptance.atd及原阅读输入 | 书卡→详情/章节/进度/添加书的原支持须先实证；不替换成正式Reader007引擎 |
| 041-auto-edit | 038 VM真实50/50，当前无theme状态声明，语法色归runtime code_editor | tests/desktop_mcp.py与原菜单/快捷键/文件条件 | 编辑/选择/undo/dirty/tab及真实支持的磁盘回读/退出，主题不重建文档或改语法内容 |
| 026-database | 旧断言0/8但app本体健康有证，当前无主题面 | 现tests/desktop_mcp.py及既有表/SQL输入 | 树/表分页排序/查询/错误保留；旧样本不冒产品真实SQLite能力，测试仅副本/隔离输入 |
| 025-sys-monitor | 038原门7/11，退役页/文案漂移；kill只验确认/取消 | tests/desktop_mcp.py及原sysinfo/确认条件 | 真实刷新/选中/图表不重置；危险动作仅owned测试子进程或原取消路径，旁观进程不动 |
| 024-charts | 038原门12/19，七腿对应旧状态契约，当前无主题面 | 原tests和图表数据/geometry fixtures | 数据编辑/图表类型/series分类与坐标结果保持，主题只改公共面和有据绘制role |
| 031-paint | 038实际31/31、文档像素哨兵稳定，当前无主题面 | tests/desktop_mcp.py | 16×16画/擦/填充/palette/undo/redo/存取，换肤后文档像素/历史逐值不变 |

T-01先冻结018五红中真实挂载/路由回显根因与四处旧测试定位漂移；修既有承诺的小接缝并保原oracle，真生成/运行缺口交owner，不按旧错误断言或简单删case结题。041-auto-edit原render=vm，精确接收其已承诺支持轨，不把未实现Vue编辑能力静默写PASS或擅自改支持承诺。 原合同要求双端/原生的必需面不因pac render或研究未测而自动删掉；真正原先未承诺的轨不能新增成无限平台补全。T-01逐例冻结支持表/原样本/原条件排除，冲突先具名重规划，不用研究的有界go降低既有承诺。

036/038已归档，实际源/工具/包版本仍须开工重核。041状态与042 Reload不是本批统一前置；仅真实违反已承诺实例/生成契约的消费者按owner交接，不能借其WIP。930只收生成交付/来源守门，不自动承担所有图像/媒体类型编译问题，真正命中共同根因的腿接正式结果。

## 5. 详细设计

### 5.1 启动与下一任务

尚未获实施授权。拟唯一树D:/autostack/.wt/ui-047/auto-ui，分支codex/plan-047-demo-themes；需实际参与的语言/Man/Web/Down各一固定兄弟树/owner，不建链接。实现/独立review同树顺序，冻结写入后review，不复制第二实现树/不改产品clone。

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
| SD-01 | add | docs/specs/ui/demo-theme-batch-047.md | 本批逐例主题/默认/内容豁免/原业务与支持轨实证 | Demo有自己的验收 | AC-01..04 |
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
| AC-01 | 本批6例同正式包、两族明暗/原默认/品牌及声明轨真实通过，V-01/04 | work |
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
| T-01 | [x]（2026-11-04）授权后唯一组 `.wt/ui-047/`=auto-ui@024fe57a 分支 codex/plan-047-demo-themes + 只读兄弟 auto-lang@9f38bbf69/auto-web@3de5524/auto-down@7c4d5cb/auto-man@2bbd9542（detached）；`cargo build -p auto-ui-tools --bin auto-ui-tool`（组ui）+`cargo build -p auto`（组man，CARGO_TARGET_DIR=组ui target）exit 0，auto.exe/auto-ui-tool.exe 钉组ui target/debug；六例测试入口冻结（018 playwright :3018 + acceptance.atd；041/024/025/026/031p desktop_mcp VM）；018 基线真复现 5/10=038 记录吻合（024 7腿/025 7/11/026 0/8 静态定位+首跑核对）；[✅ 已完成] | U036/U038；实施授权经 /goal 给付；AC-01..05 | work |
| T-02 | [x]（2026-11-04）018 原门五红根修四层生成器缝（main.ts import 拼接 74d581d9 引入；json.from_value 恒等；store facade await；as int Math.trunc——回归锚单测各一）+ demo 侧漂移修定（pct 取整/Chapters 标题）→ 018 门 9/10；compiler lib 1141→1143 全绿，tools lib 13 红均 base 同族 stash 对拍零新增；[✅ 已完成] commit a3dc9ec5 | T-01；AC-03 | work |
| T-03 | [x]（2026-11-04）六例全部接入：pac dep themes + 根 widget theme_name/dark_mode + design 绑定（025/031p dark-first 保留；041/024/026 Auto/light；018 dark-first 保留迁移前有效默认、逃生舱退役、侧栏官方切换承接 README 承诺位）；024+025 desktop_mcp 漂移修定、026 0/8→23 修定、024 vue 首建入口 test_024_vue.mjs、025 smoke 修定、041 T12 腿；[✅ 已完成] commits 5af97628(018) dd502025(024+vue 注入双缝) 5e98b078(031) c6d9d1dc(025) 19644f93(026) 604ae87f(041) | T-02；AC-01/02 | work |
| T-04 | [x]（2026-11-04）逐例两族×明暗真实通过：018 vue html.dark 官方翻转 10/10×2；024 VM 23/23 + vue CSS-var 2×2 12/12（Solarized 44 87% 94%/192 100% 11% 上身）；031p 34/34（px 像素哨兵跨肤逐值恒守）；025 VM 20/20（fixture 2×2+业务守恒+Auto/dark 默认）；026 23/23（2×2+守恒）；041 56/56（T12 2×2+tab 恒守）；scope 隔离 theme_scope_contract 6/6；非法主题状态写不崩渲染 VM 实证；[✅ 已完成] | T-03；AC-01..03 | work |
| T-05 | [x]（2026-11-04）U036 面以工具已承诺测试面为界：theme_package_input 13/13（官方包准入/身份/receipt content-not-mtime/篡改拒绝/近似名拒绝/snapshot 发布拒绝）；gen receipt 实录 9fa07742dc68；非法包/模式拒绝=pac/生成层 PLAN-036 R-03 准入（admission 测试面覆盖）；外部 delivery-root/SDK 最终消费=Man 发行/U036 谱系不变（B2 交接）；[✅ 已完成] | T-04；AC-04/05 | work |
| T-06 | [x]（2026-11-04）SD-01..04 备稿（worktree 内，merge 落地）；遗漏/边界冻结：025 vue 轨后端 a2r codegen E0432 base 同败存量缺陷具名 owner 交接（a2r 谱系，本包 §10 不承诺）；生成器五缝回归锚在 compiler/tools lib；execution_done 6/9；[✅ 已完成] commit 162f3ca9 | T-05；AC-01..05 | work |
| T-07 | 同树独立上下文final review，finding回work并失效相应证据 | T-06；AC-06 | review |
| T-08 | 冻结必要职责并集、各实际仓合入、canonical/路线/归档同源读回 | T-07 pass；AC-07 | merge |
| T-09 | 全输出/资源对账、摘要折Plan删统一scope/自有树，guard后移除/分支/组；archived9/9 | T-08；AC-08 | merge |

材料唯一D:/autostack/plan-artifacts/plan-047/auto-ui/，参与仓按repo登记owner/用途/在途。探针/小媒体/文件/截图日志/外部consumer/显式target仅scope，永久回归fixture在正式tests有据，普通构建在树。不依旧脚本默认tmp/项目内证据。merge核全部实际路径/进程，命令/退出码/关键值/源hash/未解条件折Plan后删scope/树产物；仅具名持有有owner/理由/关闭条件，必需面不借hold过关，未知/活动/他方不盲删，无旧JSON写入。

## 9. 复审记录

2026-11-04 / stage:new / auto-ui/PLAN-047 r1 / outcome:blocked（仅规划，实施授权待给；§4具名能力门先核） / 已接受038真实研究而非复制旧源码盘点，完整列本批6例/原门/已知阻点和owner/资源/独立产品边界；全部本计划探针和测试未执行。next：授权后T-01，不能将有界go/旧红说明换成必需面PASS。

静态预检pass（2026-11-04）：0–10章节、9T/8AC/8V/4SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/测试或独立final review；全部实现/运行仍待授权work。

2026-11-04 / stage: goal | plan_id: PLAN-047 | plan_revision: r1 | round: 1 | from: new(blocked) | to: work | status: drafting | outcome: pass | worktree: D:/autostack/.wt/ui-047/auto-ui | commit: 024fe57a(基)→a3dc9ec5..162f3ca9(8 commits) | next: work。/goal 完成计划047 给付实施授权（work/review/有界修复/merge 全链）；只读兄弟树 auto-lang@9f38bbf69/auto-web@3de5524/auto-down@7c4d5cb/auto-man@2bbd9542 建立并记revision；主检出WIP核查=docs/PNG证据级他方在途（045/046会话），未消费未触碰。

2026-11-05 / stage: review | review_scope: final | reviewer_identity: 同会话上下文（与实现同 session——按技能条款声明局限，判定由冻结基线上的独立重放/复跑与制品重建，不采信执行摘要）| plan_id: PLAN-047 | plan_revision: r1 | outcome: pass（reviewed）| worktree: D:/autostack/.wt/ui-047/auto-ui | branch: codex/plan-047-demo-themes | reviewed_commit: 21906e0b（rebase 后链 6dc25877..21906e0b，base=master@7e517e0c）| base_commit: 7e517e0c（原 work 基 024fe57a 已被 045/046/051/051-followup 等 42 commit 前进——rebase 同步 + 冲突解决记录于本行后附注）| dependency_revisions: auto-web@37d32c2（plan-046 F-1 api_gen 修复入点）/auto-lang@43ff927f3/auto-man@03bb65e/auto-down@7c4d5cb（只读 detached，refresh 实录）| spec_inputs: SD-01 demo-theme-batch-047.md（新）+ SD-02 theme-app-migration.md + SD-03 specialized-theme-migration-readiness.md + SD-04 theme-scope-and-publication.md（修改，冲突解决后与 045/046 已落地行相容，B3 收口 12/12 表述）| acceptance_results: AC-01 pass（六例同包两族明暗：018 vue 10/10×2 html.dark 官方翻转、024 VM 23/23+vue 12/12 CSS-var 逐值、031p 34/34、025 VM 20/20、026 23/23、041 56/56——rebase 后全数重跑实录）；AC-02 pass（业务守恒：px 像素哨兵/tab/排序/分页/SQL/流式/真实刷新跨肤恒守，scope 隔离 6/6，非法主题写不崩渲染实证）；AC-03 pass（038 F-10 五红闭=原门 10/10、F-9 七腿修定+vue 数据腿、F-8 0/8→23/23、F-4 修定+重启腿；025 vue 后端 a2r E0432 重验=新基线同败、与 046 F-1 修复（031iv shape）不同实例，owner=auto-web api_gen scaffold 谱系具名交接）；AC-04 pass（theme_package_input 13/13：准入/身份/receipt content-not-mtime/篡改拒绝/近似名拒绝/发布拒绝；gen receipt 9fa07742dc68 实录；外部 delivery/SDK 最终消费=Man 发行/U036 谱系不变）；AC-05 pass（边界：B3 12/12 收口表述与 045/046 相容、34 示例累计、025 a2r 交接、041 语法着色 owner 不变）；AC-06 pass（本行）| pending_closeout_gates: merge 冻结职责并集=compiler lib 1145 + tools lib（环境族 8-13 红已具名，与 master 基线同族）+ theme_scope_contract 6/6 + theme_package_input 13/13 + 六例门并集（本轮 fresh 全绿实录即冻结基线证据）；025 smoke.spec.ts 执行待 a2r owner 交付（具名 hold 不阻本包 VM 原门验收）| findings: R-0（rebase 冲突）vue.rs main.ts 注释双计划署名合并（PLAN-051 T-02① 与 PLAN-047 T-02 同一缺陷独立发现，代码同形，注释合并保留双引用）；R-1 SD 文本 stale（045/046 在途表述）——已在 rebase 中修正为收口表述（amend 入 21906e0b）；R-2 025 E0432 新基线复验同败=非本批回归，具名 hold 维持 | evidence: compiler lib 1145/0；theme_scope_contract 6/6；theme_package_input 13/13；tools lib 205/8（族=base 同族）；六例门 fresh 全绿（018 10/10、024 23+12、031 34/34、025 20/20、026 23/23、041 56/56，AUTOUI_TEST_FIXTURES=1）| next: merge

**rebase 附注（2026-11-05）**：master 024fe57a→7e517e0c 42 commits（045/046 交付、051 交付+followup、044 autographics 收敛）。冲突两处：vue.rs main.ts 前导
（PLAN-051 独立修复同一缺陷，注释双署名合并）；theme-app-migration.md 批次表（045/046 第五/六批行 vs 本批第五批稿——重写为第七批 + B3 收口 12/12/34 累计）。SD-01 标题改「批次047（B3 分包③=第七批）」、readiness 头注补 §7 指引。依赖 pin refresh：auto-web 3de5524→37d32c2（046 F-1 修复）、auto-lang 9f38bbf69→43ff927f3、auto-man 2bbd9542→03bb65e、auto-down 不变。

2026-11-05 / stage: merge | plan_id: PLAN-047:r1 | outcome: pass（delivered）| delivery_commit: 21906e0b（ff-only landing @master，无 merge commit）| rebase_mapping: a3dc9ec5→6dc25877 / 5af97628→7d2decf2 / dd502025→529b0503 / 5e98b078→a78a1011 / c6d9d1dc→a7b52bb7 / 19644f93→c3d34b7e / 604ae87f→de61201c / 162f3ca9→21906e0b；range-diff：#2..#7 全等 `=`，#1/#8 `!` 为已审差异（#1=PLAN-051 双署名注释合并，代码同形；#8=SD 文本与 045/046 已落地行对账收口）——冻结验证并集在 21906e0b 上 fresh 全绿（compiler lib 1145/0、theme_scope_contract 6/6、theme_package_input 13/13、tools lib 205/8 环境族、六例门 018=10/10×2 / 024=23+12 / 031=34/34 / 025=20/20 / 026=23/23 / 041=56/56）| canonical_spec_paths: docs/specs/ui/demo-theme-batch-047.md(1207fe78feed) + theme-app-migration.md(0dd98ad8a305) + specialized-theme-migration-readiness.md(a67d477427bd) + theme-scope-and-publication.md(fa8829b04c11)——landed 后 main vs worktree hash-object 逐一相同（canonical_verified）| archive: docs/plans/archived/047-specialized-workbench-demo-theme-migration.md（本文件，status: archived，随本提交入库——UAT-K13 tracked）| cleanup: wt-guard clean（pnpm junction 类 gen/node_modules 工件删除后）；worktree+branch 移除与 plan-artifacts 删除见后记 | landing-not-deployment 观察：本仓无生产部署面（教学示例 gen 产物按需再生；main 检出无 target/debug/auto.exe 消费者——`which auto`=auto-man 主检出构建，本次未触 auto-man 源（仅只读兄弟树），无需重建）；named holds：025 vue 轨后端 a2r codegen E0432（owner=auto-web api_gen scaffold 谱系，closure=该 shape 修复落地后跑 examples/ui/025-sys-monitor/tests/smoke.spec.ts 全绿）+ 他方 WIP 观察：main 检出 `M crates/auto-ui-runtime/src/ui/dynamic.rs` 为 plan-051 followup 会话在途 WIP，本包未消费未触碰 | batch_regression_due: auto-ui 仓 docs/plans/.last-batch-regression.json 不存在（如适用归 auto-plan-regress 批次门，非本包阻塞）

2026-11-04 / stage: work | plan_id: PLAN-047 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-047/auto-ui | branch: codex/plan-047-demo-themes | base_commit: 024fe57a | code_commit: 162f3ca9 | dependency_revisions: auto-lang@9f38bbf69/auto-web@3de5524/auto-down@7c4d5cb/auto-man@2bbd9542 (detached 只读) | task_ids: T-01..T-06 | evidence: 018 playwright 10/10×2（BOOK_URL=:3018，vite+back 直连）；024 desktop_mcp 23/23 + test_024_vue.mjs 12/12；031 desktop_mcp 34/34（px 哨兵跨肤恒守）；025 desktop_mcp 20/20（含 T6 重启腿）；026 desktop_mcp 23/23；041 desktop_mcp 56/56（T12 腿 + T9/T10/T11 独立进程组原样）；compiler lib 1143 全绿（含回归锚单测5枚）+ theme_scope_contract 6/6 + theme_package_input 13/13；tools lib 13 红均 base 同族 stash 对拍零新增 | blockers: 无阻断；具名交接=025 vue 轨后端 a2r codegen E0432（base stash 同败，owner=a2r 谱系）+ 生成器五缝回归锚已固化（main.ts 分行/from_value 恒等/store facade await/as int 截断/dep 根感知注入，单测各一）| next_responsible_role: review（T-07 同树独立 final review）

## 10. 待澄清事项

1. （已答）实施授权经 /goal 给付，T-01..T-06 已交付；能力差额按真实反例闭（见 §9 work 收据）。025 vue 轨后端 a2r codegen E0432 为存量缺陷（base 同败），owner=a2r 谱系具名交接，不借 WIP、不冒关闭；其 smoke.spec.ts 修定已随仓待该轨可用后执行。
2. 041/042与OS产品迁移不是全局前置；确命中930共同来源根因的腿只接正式结果，不借WIP。未知类型/服务问题不自动丢给930。
3. 每例原双端/原生支持必须按现合同冻结，不偷减，也不将未承诺的新格式/平台无限并入本包；原性能/时序条件保持。
4. 迁移仅教学Demo，实际产品各仓独立验；用户数据/系统设置/旁观进程不动，真实窗口与资源T-01明确。
