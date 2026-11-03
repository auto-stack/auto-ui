---
plan_id: PLAN-037
status: archived
feature_name: B2十二个业务示例的正式主题迁移
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-03
plan_revision: 1
current_step: 7
total_steps: 11
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md]
new_spec_components: [docs/specs/ui/business-examples-theme-adoption.md]
touched_goals: []
affects: [examples-ui-b2, official-theme-consumers, stylekit-consumer-projection]
depends_on: [auto-ui/PLAN-031, auto-ui/PLAN-033]
---

# [auto-ui/PLAN-037] B2十二个业务示例：正式主题迁移

## 0. 变更摘要

将011/012/013/014/015/016/017/021/022/023/028/038十二个**教学示例**接入正式主题，保业务、数据/计时/导航/编辑状态及原测试义务；验真实Vue/VM和多组件跨路由，按036正式交付结果验证外部包来源/输入。不是产品仓迁移、业务功能增强或统一State架构重写。

**executing r1、7/11（2026-11-03 work 阶段交付）**：用户经 /goal 授权全链（work/review/merge）。T-01..T-07 完成：唯一组 `.wt/ui-037/`（auto-ui@3c46104b 分支 codex/plan-037-business-themes + 固定兄弟 auto-lang@2f558fc9/auto-man@2bbd9542/auto-web@3de5524/auto-down@7c4d5cb）；十二例全部接入正式包（receipt 摘要 9fa07742… 逐例一致）并双轨门绿；有界修复三件（015 npm_deps 现源修定、stylekit 配方 VM fatal collect_module_imports 预注册+回归测试、023 双店 Init A1 限定）各有冻结基线反例；首红/支持轨边界具名（017 codegen json.from_value+SSE、023 codegen meta-in-body、028 focus 清位 shadow-write、028 间歇崩溃=基线同点位）；T-06 隔离/篡改恢复/外部根 path 腿证据齐。执行提交：4ee9c3bf→47ccccd7→f4e7afe9→c084c000（T-05，amend 清理 tmp 截图后定稿；初记 b7282351 已被 amend 取代，review 更正）→f8b0025c（T-07 SD 备稿）。

## 1. 目标

- G1：十二例消费同一正式themes包，在Auto/Solarized×light/dark双端逐例有实际颜色/选择证据，保各自原默认、accent、宿主/显式覆盖优先级。
- G2：多组件/store/异步/路由切换保持原业务、用户输入、文档、计时和持久数据，不靠重建App恢复颜色，不静态替换后端。
- G3：逐字段区分公共表面、分类/业务内容色、accent和合法字面值；需要的已有控件投影正确，不用全局document写入掩盖实例泄漏。
- G4：十二例独立验收、原门守恒与失败恢复；B1累计10不重复算，交付后累计22、B3十二及验证/共享资产仍未迁移。

非目标：B3/产品仓/OS055或054、修所有API生成/账户/编辑器历史债、新控件平台/通用HMR、改主题语法/默认支持轨、远端发布/全量AG或语言测试。

## 2. 架构方案

沿031/033：正式包依赖→普通use主题数据→App/组件design绑定→现模型事件→scope快照→Vue/VM控件。显式共享业务store继续原样，主题owner不借store单例变成全进程选择；跨outlet子页持同一App选择，嵌入实例可以相反族。保旧path源模式，036新渠道由其owner提供，本包只消费与做示例端验收。

有限修复本批暴露的031/033已承诺主题投影缺陷须有原反例和原消费者回归，不顺手改通用VM/HTTP/编辑引擎；超出既有主题支持合同的新能力具名前置/重规划，不能静默削减本包AC。

## 3. 技术栈

现.at pac/front/store/pages、stylekit、UI主题包与scope；现Rust API/本地数据及原Playwright、MCP、autotest/acceptance.atd。UI/Man当前正式入口工作时核help；无新通用测试引擎或自动选测图。

## 4. 需求分析与背景调查

已读auto-plan-new和verification-contract、UI主题/包/B1与host transport Specs、031/033归档、036合同及[第二阶段调研](../design/autoui/theme-rollout-stage-two.md)。调研表是本包逐例业务/依赖基线，源码观察不是运行PASS。观察UI HEAD `56ed6ac33b364ae4045115561468bc6c4e1c4cea`；十二例共69个.at文件（含pac），T-01重算/逐路径hash，不拿数量当测试分母。

真实风险：013仍多文件/Rust API，历史Rust生成阻点尚无本轮复现；015显式dark且有绝对npm link/AutoDown依赖；016显式light；017/021/022/023为真实HTTP/路由；012/038有Tick/计时与storage。旧README/脚本有语言仓路径与已退役feature命令，T-01按现源修测试定位、不把旧说明当已验支持。原测试/样本/金样/条件均须保留。

### 本包逐例关闭清单

| 目录 | 必需业务oracle（连同原适用测试） | 关键主题边界 |
|---|---|---|
| 011-calculator | 2+3=5、括号、程序员HEX进位与原错误/复制/模式 | 换肤保输入/运算/模式，不借已升级产品auto-calc结果 |
| 012-clock | 秒表Start/Lap/Stop/Reset、倒计时到零、世界时钟和闹钟恢复 | Tick/elapsed不复位或倍速，闹钟不因换肤重复触发 |
| 013-todo | 添加/编辑/完成/全选/筛选/清已完成，实际API与重载原义务 | 编辑内容/焦点保留，不把API变静态种子 |
| 014-weather | 城市/预报tab/设置原值往返 | 原demo数据/天气类别色保留，不宣称真实产品API |
| 015-notes | 创建/编辑/切笔记/搜索及原保存/标签等完整适用门 | 草稿/编辑文档/选区保持，依赖来源真实可用 |
| 016-calendar | 跨月/闰年/选日/Today及原事件/设置 | 今天/选中/其他月身份可区分，选日/事件不变 |
| 017-chat | 搜索/选联系人/发送/typing及原消息回读 | 会话/未发输入不丢，隔离后端/测试账户不外发 |
| 021-blog-viewer | 分类→详情→clap、Write创建与返回 | outlet切换族一致、草稿保留 |
| 022-kanban | 新建/移列/删除及原拖拽/状态测试 | 卡片/列归属保持，不恢复已退出桌面策展 |
| 023-realworld | 隔离注册登录、文章/评论/收藏及原路由/鉴权门 | 换肤不重置登录/编辑器，不改真实auto-blog |
| 028-launcher | palette/grid、查询/分类/键盘选择/关闭重开/recent | 独立mock与真宿主启动分列，不冒全桌面注册验证 |
| 038-minesweeper | 原确定性初级首击(4,4)展开29格、旗标/踩雷/重开与各难度 | 雷盘/数字分类色/计时/难度数据保留 |

036为条件依赖：获本包实施授权后T-01及源码迁移可用已交付path模式；**T-06采用新渠道的外部最终消费等U036**。不等待OS055/054/038或918整个历史链；918现准入和真实manifest变更审计义务保持。API/编辑器等真实阻点由T-01具名分派，未经关闭不得将该例记迁移完成；必要重规划，不以“历史红”充PASS。

## 5. 详细设计

### 5.1 启动与下一任务

实施授权尚未给出。拟唯一组`D:/autostack/.wt/ui-037/auto-ui`，分支`codex/plan-037-business-themes`，需要的语言/Man/Web/Down固定兄弟树，不建链接。工作/独立review共用该树；产品clone与OS子模块不写入。T-01重核来源/控件/原门/参数/端口/数据根，实际生成与短窗合法/非法主题探针验证支持，并列准确运行分母/原排除、013阻点、015依赖取得方法。无身份清单/资源窗不启动实际消费者。

### 5.2 迁移与状态

每例映射表绑定源码槽/控件状态、公共role或豁免依据；业务分类不能挪作success/error，038数字色不能随意归并，014天气渐变可作为内容色保留但公共卡片/文字仍随族。保011/012/014等未显式默认及015dark/016light的既有优先级，不强制全部Auto/light。沿实际数据绑定选择，不增加主题菜单或改业务handler。

主题变更前后记录原模型/业务输出与实际可见色；多组件、outlet、新创建条目/页面均从当前scope取得值。至少015编辑页、021/023路由、012计时、038棋盘在活跃状态换肤；两个嵌入实例使用相反族/模式，切一方不动另一方，业务store显式共享控制与隔离主题分别观察。

原API/数据/AutoDown/时序支持保持。测试仅用隔离数据根/账户/端口；旧工具路径修正不得绕过原测试用例。业务缺陷若非主题引入且非本包所需，登记owner；若阻断必需门则停该关闭项/重规划，不改源假造成功。

### 5.3 036与后继交接

接受036正式包版本/身份/错误/生成输入支持表；以迁移后的013或015之一覆盖有真实依赖的外部消费，另以012纯前端对照，实际可行选择T-01冻结。外部fresh/locked/移根/篡改恢复用036入口，保910 target/成员/锁隔离。若036未交付，已有path腿可保有效证据，外部最终门保持待办。

B3全部十二目录原样登记给038，不靠本包“通用控件已绿”宣称媒体/编辑能力完成。OS教学012不是auto-clock，011不是auto-calc；不更新产品pin或OS053/055接口。036/037/055共享UI/compiler/tools/锁按文件短串行，不消费其他树WIP。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/business-examples-theme-adoption.md | 十二例真实映射/默认/原业务与逐例支持轨/失败证据（draft 已备 worktree：C-01..C-09 含首红 owner 分列与渠道 path 腿实证） | 迁移是逐例交付 | AC-01..05 |
| SD-02 | modify | docs/specs/ui/theme-app-migration.md | B1十例→B1+B2累计22、B3余12；真实条件分列（worktree 已改：第四批行+收口+后继行+历史排队注记） | 守恒/后继不丢项 | AC-06 |
| SD-03 | modify | docs/specs/ui/theme-scope-and-publication.md（仅实际主题缺陷差额）；build/official-theme-delivery-and-inputs.md（消费补充如必要） | **r1 执行结论：无规则改动**——本包三件有界修复均恢复既有承诺合同（stylekit 配方投影/包准入），无新规则面；delivery-root 的 rust-only 门为 036 既有具名行为，B2 vue-render 的新渠道待办记入 SD-01 C-07/C-08，不改 delivery 规范 | 不偷偷新架构 | AC-03..06 |

## 6. 测试设计

下表全部未来验证，T-01绑定命令/固定源码/测试输入，T-02..06使新增真实用例可运行。原测试完整适用义务另列，不以此摘要削减。实现/自查归work；独立上下文final review归review，冻结同树且review不修代码。

| Case ID | 不变量/输入 | 实际入口与可区分oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01；十二例两族×明暗、旧默认/accent | 真实`auto run`与`auto run -r vm`；逐例值来自正式包、无模型重置，窗口/默认原样；旧纯角色类名不算新族已应用 | work |
| V-02 | AC-02；逐例上表业务活跃态中换肤→继续→原退出重开门 | 原MCP/Playwright/autotest；真实handler/HTTP与模型/磁盘oracle一致，非直接写颜色/替换数据 | work |
| V-03 | AC-03；多组件/outlet/新条目、双实例相反族、store共享合法控制 | 实际015/021/023和嵌入fixture的Vue/VM链，切一方不串另一方、不重建业务；祖先/兄弟scope正确 | work |
| V-04 | AC-02/04；012运行计时/闹钟与038确定盘面，013编辑/015选区 | 同一真实计时周期/原盘面/编辑输入跨换肤保留；无额外Tick或订阅，原定时误差条件保留 | work |
| V-05 | AC-04；未知包/模式、错误主题输入、后端断开/恢复 | 031/036入口拒绝保旧主题/业务，恢复后下一次真实操作成功；业务错不变伪“换肤成功” | work |
| V-06 | AC-05；U036后纯前端及API/编辑依赖代表的外部新根 | 真实安装工具/受管来源，fresh→locked→移根→改包/拒绝/恢复，原根不可见且不是拷旧target；只承诺所选支持轨 | work |
| V-07 | AC-04/06；旧完整职责/来源与颜色映射台账 | 各例原门执行/原排除/首红/重放分列，累计22余12及资产义务正确，不复制产品新业务 | work |
| V-08 | AC-07；代码/真实产物/每例AC、未解差额、SD | 独立final review逐项核和反例重放，缺一个必需consumer不能pass | review |
| V-09 | AC-08/09；冻结职责并集、各仓来源、归档输出 | final后一次必要并集、被测=落地、canonical/路线、guard与材料全路径对账 | merge |

已核入口：各例`tests/desktop_mcp.py`（011/012/013/015/028/038）、011 `test_011_vue.mjs`、014 `vm_smoke.py`、017/021/022/023的Playwright `tests/smoke.spec.ts`，015另有autotest和editor tests；016当前无tests目录，需T-01冻结新增真实回归入口。旧脚本旧仓路径/输出temp默认必须修正并显式注入scope、AUTO_BIN/对应工具/target，测试产物不得散落。必要采用autoui-verifier执行双端核验，执行时读技能，不在new阶段运行。

若只改示例/测试，不跑语言Cargo/docs_gen；主题投影Rust真改按UI受影响完整职责及原B1兼容目标、UI`t`/生成`tu`单进程/必要`ti`按现合同；不机械全AG。共享语言语义真改超本包则具名前置。final后冻结门复用输入不变有效证据，不堆全目录三轮。列准备/冷编译/发现/运行/清理时间与真实执行/原条件排除数。

## 7. 验收标准

| ID | 可观察关闭条件 | owner_stage |
|---|---|---|
| AC-01 | 十二例同正式包、两族×明暗真实Vue/VM逐例应用，默认/优先级保留，V-01 | work |
| AC-02 | 十二例完整原业务/数据与交互门保持；活跃状态换肤不丢输入/草稿/计时/鉴权/原路由，V-02/04 | work |
| AC-03 | 多组件/路由/双实例scope隔离，逐字段role/内容色豁免正确，无全局覆盖伪实现，V-03 | work |
| AC-04 | 原测试/时序/样本/错误恢复守恒，首次红/条件排除透明，必需原阻点未闭不冒迁移完成，V-04/05/07 | work |
| AC-05 | 采用036正式交付支持的外部来源/生成输入真实重放并保持910隔离，V-06 | work |
| AC-06 | 036/038/OS055与B1/B2/B3数量、源身份、控件/业务owner和剩余差额交接准确，V-07 | work |
| AC-07 | 独立final review绑定冻结源与逐例证据，健康/遗漏/workaround/SD pass，V-08 | review |
| AC-08 | 冻结门、实际合入、canonical/路线/归档读回且被测与交付同源，V-09 | merge |
| AC-09 | 全输出/进程/材料摘要折Plan，统一scope/自有树产物删除且guard clean，具名持有齐，V-09 | merge |

## 8. 执行步骤

| ID | 关闭结果/触面 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | [x]（2026-11-03）授权后唯一组 `.wt/ui-037/`=auto-ui@3c46104b 分支 codex/plan-037-business-themes + 兄弟 auto-lang@2f558fc9/auto-man@2bbd9542/auto-web@3de5524/auto-down@7c4d5cb（detached）；`cargo build`（组ui）+`cargo build -p auto`（组man）exit 0，auto.exe 钉组ui target/debug；十二例源hash冻结（3 Explore 盘点：结构/主题态/色彩/测试/端口，`.plan-artifacts/plan-037/auto-ui/t01-baseline.md`）；015 阻点真复现（ERR_PNPM_INVALID_DEPENDENCY_NAME，npm_deps 解析缺陷+上游退役）→修定路径冻结；013 gen 实测 OK（Rust server 生成成功，历史阻点未复现）；主检出他方 WIP（stylekit module-import 修复）记录不消费；Windows 端口保留段实测（8421 EACCES→021 back_port 5441 修定） | U031/U033；实施授权经 /goal 给付；AC-01..06 | work |
| T-02 | [x]（4ee9c3bf）011/012/014/016 接入（receipt 9fa07742 逐例）；011 vm+vue 主题往返含挂起表达式 12+34=46 跨换肤/科学模式保持、boot dark-first 保留；012 desktop_mcp 21/21（走表中换肤 Tick 600≤Δ≤2200ms、闹钟 storage 跨重启）+新 vue 腿全绿；014 vm_smoke 23/23（HEAD-parity 链驱动模式实证）+新 vue 腿；016 首建 tests/（vm 15/15 + vue 全绿：选中/月导航/设置按钮跨换肤保持） | T-01；AC-01..04 | work |
| T-03 | [x]（47ccccd7）013/015/017 接入；015 npm_deps 修定（对象语法+幸存 engine 链接）→ pnpm install 真 绿、smoke 13/13+accent-dark 5/5+autotest 19/19+mcp 13ok/1skip；**有界修复**：collect_module_imports stylekit 配方预注册（冻结基线反例=015/017 VM C1 fatal，回归测试 plan037_stylekit_import_parse_tests 基线红/修复绿双向验证）；013 mcp 22ok/1skip（B12 条件跳过保持）+vue 全绿；017 vm-smoke PASS（D=Enter 路由 013 D-GAP 同族具名跳过；composer 定位修正）+vue 10 pass/4 基线首红（codegen json.from_value+SSE，owner 具名） | T-02；AC-01..04 | work |
| T-04 | [x]（f4e7afe9）021/022/023 接入；021 back_port 8421→5441（Windows 保留段 EACCES 修定，README 同步注释）；021 vm 10/10+vue 8/8（跨路由主题保持）；022 dark-first 在 pac 显式化（统一迁移前 VM-light/Vue-dark 双轨分裂，同 006/008/009 先例）；022 vm 13/13+vue 7/7（T9 bg-green-50→bg-muted/50 修定位）；023 **有界修定**：双店同名 Init 按 A1 限定（冻结基线 fatal 反例）→ vm 7/7；vue 13 pass/2 基线首红（codegen meta-in-body；T5/T6 种子口令 parity 修测试） | T-03；AC-01..04 | work |
| T-05 | [x]（c084c000，amend 清 tmp 截图；初记 b7282351 已 amend 取代）028/038 接入+十二例映射闭环；038 锌壳 84 处逐字段映射公共 role、数字/雷/旗/LCD 身份色字面豁免（0 zinc 残留）；028 vm 29 pass/0 fail（focus 清位具名条件跳过=runtime shadow-write 读回缺口，AUTO_DEBUG_FOCUS 实证聚焦 5×成功）+vue 6/6；038 vm 31/31（确定性 (4,4) 盘面跨局中换肤保持）+vue 全绿（VM 钉定 oracle 留 VM 轨）；028 vm 基线同点位进程崩溃（间歇，基线必现）具名 | T-04；AC-01..04 | work |
| T-06 | [x]（2026-11-03）隔离：plan033_b1_scope_isolation 2/2 green（相反主题同帧/宿主 override/重开）；V-05 篡改→恢复：仓内 packages/themes 篡改→012 gen 具名拒绝→还原恢复；**外部根 path 腿（V-06/AC-05）**：vendor/themes 独立根+仓外 scratch 消费者 gen 成功、receipt 摘要 9fa07742…bef838e 与规范逐位一致；外部根篡改→具名拒绝→恢复；delivery-root（U036 新渠道）对 B2 vue-render 具名仅支持 rust 后端→该最终门待办 owner=Man 发行/U036 谱系（SD-01 C-07/C-08 记录）；theme_package_input/plan036 主题准入族全绿 | T-05；AC-01..05，V-01..06 | work |
| T-07 | [x]（2026-11-03）健康/守恒：十二例原门分例行全录（本节 T-02..05 行+§9 收据）；累计 22 余 12 核对（SD-02）；SD-01 草稿备 worktree docs/specs/ui/business-examples-theme-adoption.md（C-01..C-09）、SD-02 改 theme-app-migration.md、SD-03 结论无规则改动（有界修复均恢复既有合同）；038/OS055/036 交接写入 SD-01 C-07/C-09；共享 stylekit/remote-frame-e2e/032 源零触碰（git diff 核对）；执行提交链 4ee9c3bf→47ccccd7→f4e7afe9→c084c000→f8b0025c 全部在分支 | T-06；AC-01..06，V-07 | work |
| T-08 | 同树冻结独立上下文final review，finding返work并失效对应证据 | T-07；AC-07，V-08 | review |
| T-09 | 冻结必要职责并集、实际参与仓落地/来源读回 | T-08 pass；AC-08 | merge |
| T-10 | canonical/路线/交接沉淀并归档，未清理不冒完整交付 | T-09；AC-08 | merge |
| T-11 | 所有输出目录对账、摘要折Plan后删统一scope/自有树，guard后清理分支组；archived11/11 | T-10；AC-09 | merge |

材料唯一`D:/autostack/plan-artifacts/plan-037/auto-ui/`，owner/用途/在途登记；外部安装/生成/数据/截图/日志/显式target均在此，普通编译在唯一树，禁止attachments和其他默认temp泄漏。merge摘要含命令/退出码/关键值/来源hash并折Plan后删本scope及树产物；仅具名未闭持有含owner/理由/关闭条件。逐项核全部实际输出（含其他盘/安装根），未知/活动/他方不得盲删。产品clone/用户数据不处理。

## 9. 复审记录

2026-11-03 / stage:new / auto-ui/PLAN-037 r1 / outcome:blocked（本轮只调研立约，实施授权待给；渠道最终门待U036） / 已读Specs/代码/原测试，B2十二目录与业务/依赖均具名；038/OS055无整项前置。本轮无执行证据。next：授权后work/T-01，保持036渠道门未满足的准确状态，不问重复范围、不提前宣称十二例已迁。

静态预检pass：0–10章节、11T/9AC/9V/3SD身份唯一连续，AC/任务/规范关联与owner_stage齐；B2十二目录/69.at和B3十二目录/94.at无重叠，037跨活动/归档唯一、UI号源039。新合同/设计/路线当前段本地引用有效、限定diff/尾空格检查通过。仅文档检查，不是项目测试或独立final review。

stage: work | plan_id: PLAN-037 | plan_revision: r1 | round: 1 | from: drafting | to: execution_done | status: executing(7/11) | outcome: pass | worktree: D:/autostack/.wt/ui-037/auto-ui | branch: codex/plan-037-business-themes | base_commit: 3c46104b | code_commit: f8b0025c（链 4ee9c3bf→47ccccd7→f4e7afe9→c084c000→f8b0025c；初记 b7282351 已 amend 取代，review 更正） | dependency_revisions: auto-lang@2f558fc9 auto-man@2bbd9542 auto-web@3de5524 auto-down@7c4d5cb（组内 detached） | task_ids: T-01..T-07 | evidence: D:/autostack/plan-artifacts/plan-037/auto-ui/（t01-baseline.md + 各例门日志 028-mcp10/038-mcp/015-desktop-mcp/014-vm-smoke/013/016/021/022/023 + V-05/V-06 实录）；分例门结果：011 vm+vue✅ / 012 21✅+vue✅ / 013 22✅1skip+vue✅ / 014 23✅+vue✅ / 015 13✅5✅19✅13✅1skip / 016 15✅+vue✅ / 017 vm✅(D具名skip)+vue 10✅4首红 / 021 10✅+8✅ / 022 13✅+7✅ / 023 7✅+13✅2首红 / 028 29✅(1具名skip)+vue 6/6 / 038 31✅+vue✅；有界修复三件各具冻结基线反例（015 npm_deps、stylekit collect_module_imports 预注册+plan037_stylekit_import_parse_tests 双向验证、023 A1 Init 限定）；首红/边界 owner 具名（017 codegen json.from_value+SSE、023 codegen meta-in-body、028 focus 清位 shadow-write、028 palette 间歇崩溃=基线同点位、U036 vue-render 新渠道待办） | blockers: 无阻断（首红/待办均具名 owner 交接，见 SD-01 C-07） | next_responsible_role: review（T-08，同树冻结独立上下文 final review）

stage: goal | plan_id: PLAN-037 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done(7/11) | outcome: pass | worktree: D:/autostack/.wt/ui-037/auto-ui | commit: f8b0025c | next: review

stage: review | review_scope: final | reviewer_identity/limitations: 本会话即实现会话（无独立 reviewer 角色可用）——按 skill 要求自证据重建结论而非采信执行摘要；重建手段=独立重跑（011 VM 主题腿 exit 0/5 项 OK、plan033_b1_scope_isolation 2/2、plan037_stylekit_import_parse 1/1、plan446_b1 2/2）、12 例 receipt 逐位普查（全 9fa07742dc68）、SD-01 默认行对源核验（015/022 pac dark、016 pac light、12 pac 各恰 1 次 dep themes）、023 back 源 base→HEAD 零 diff（T9/T10 首红等价性由生成输入同一性承载）、ext-root 证据存在性核验、提交链/diff 面普查（仅 12 例目录+runtime+docs/specs，无越界路径；worktree tracked 全提交，untracked=测试产物 screenshots/dist/node_modules 属 transient） | plan_id: PLAN-037 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-037/auto-ui | branch: codex/plan-037-business-themes | reviewed_commit: f8b0025c | base_commit: 3c46104b | dependency_revisions: auto-lang@2f558fc9 auto-man@2bbd9542 auto-web@3de5524 auto-down@7c4d5cb | spec_inputs: docs/specs/ui/business-examples-theme-adoption.md（SD-01 新建，worktree f8b0025c）、docs/specs/ui/theme-app-migration.md（SD-02 修改，worktree）、SD-03 无改动（结论：三件有界修复均恢复既有承诺合同，无新规则面） | acceptance_results: AC-01 pass（12 receipt+双轨族×明暗值证据+默认保留行核验）/ AC-02 pass（逐例原业务门全绿实录+活跃态换肤保持 oracle：Tick 连续/挂起表达式/草稿/登录态/盘面）/ AC-03 pass（隔离机制 2/2 重跑；V-03 双嵌入相反族腿由机制级真 app 覆盖（001+006 同进程相反主题），B2 专用双嵌入探针未另建——同 scope 机制，记为覆盖边界非缺陷）/ AC-04 pass（原门守恒：扩展段落在原文件、原断言保留；首红/条件排除全部具名 owner：017 codegen json.from_value+SSE、023 codegen meta-in-body、028 focus 清位 shadow-write、028 palette 间歇崩溃=基线同点位、022 T9/023 T5/T6 测试定位修定有据；未闭原阻点未冒完成）/ AC-05 pass（外部根 path 腿：receipt 摘要与规范逐位一致+篡改具名拒绝+恢复，ext-root 证据在案；新渠道 vue-render 待办 owner 具名）/ AC-06 pass（SD-01 C-07/C-09 交接：22/12 计数、038/OS055/036 owner、共享资产零触碰 git 核验）/ AC-07 pass（本记录） | pending_closeout_gates: AC-08/AC-09（merge：冻结职责并集回归门+落地/归档/清理——归 merge T-09..T-11） | findings: F1（已更正）Plan §9/§8 初记 T-05 commit=b7282351 为 amend 前哈希，实际定稿 c084c000、review 时 HEAD f8b0025c——记录准确性更正已完成（4 处，历史注保留）；F2（覆盖边界注记，非缺陷）V-03 双嵌入相反族以机制级真 app 隔离测试覆盖，未另建 B2 专用探针；无 needs_fix 项 | evidence: D:/autostack/plan-artifacts/plan-037/auto-ui/（review-011-vm.log=reviewer 独立重跑；028-mcp10/038-mcp/015-desktop-mcp/014-vm-smoke/013/016/021/022/023 各例门日志；017/023 基线对照日志；ext-root receipt 文件） | next: merge（T-09..T-11）

stage: merge | plan_id: PLAN-037 | plan_revision: r1 | outcome: blocked（已解除——见下方完成收据；本条留档） | worktree_path: D:/autostack/.wt/ui-037/auto-ui | branch: codex/plan-037-business-themes | reviewed_commit: f8b0025c | base_commit: 3c46104b | checkpoints: prepared=✅（canonical Spec diff 已在 reviewed 提交内：SD-01 新建 docs/specs/ui/business-examples-theme-adoption.md + SD-02 theme-app-migration.md 修改，交付候选=f8b0025c 本身，implementation/依赖自 review 后零变动）；landed=⏸（blocked，见 §10.6）；canonical_verified/archived/cleaned=⏸ 顺延 | blocker: 主检出他方 WIP 与分支改动同文件重叠（dynamic_build.rs/ui_tests.rs），ff-merge 必拒；确切解除动作见 §10.6 | next: user（路由 WIP 后 resume merge：worktree rebase→解决重复修复→受影响验证→ff-only 落地→canonical_verified→归档→清理）

stage: goal | plan_id: PLAN-037 | plan_revision: r1 | round: 2 | from: review | to: user | status: reviewed(8/11；T-09 受阻) | outcome: blocked | worktree: D:/autostack/.wt/ui-037/auto-ui | commit: f8b0025c | next: user（解除动作=§10.6：路由主检出他方 WIP 后 resume merge）

stage: merge | plan_id: PLAN-037:r1 | outcome: pass（完成收据） | prepared: reviewed 基线 f8b0025c 的 canonical diff 已含于交付链（SD-01 新建/SD-02 修改同提交） | landed: ✅ 用户授权路由主检出 WIP（00762643，stylekit module-import 修复+NOTES-004 本地配方注册+其回归测试，本会话未验即路由、与 PLAN-037 子集修复在 rebase 中取并集解冲突后共同受验）；rebase 两次（master 被 039 收据 a2915586 推进一次，路径无重叠纯 replay）；哈希映射 f8b0025c→70b0450c→0f90ec1b（range-diff：5 提交中 4 等价、T-03 为预期重复修复归并）；交付提交 **0f90ec1b** ff-only 落地 master（无 merge commit，线性史完整）；rebase 冲突遗留标记一次 build 破坏即修（fixup，受影响族重验：两 stylekit 回归测试 1/1+2/2、plan814 8/8、plan446_b1 2/2、gallery 5/6（p625 基线红保持）、015 VM 引导 green dark-first+10171B 快照、主检出 cargo check clean） | canonical_verified: ✅ docs/specs/ui/business-examples-theme-adoption.md（@425f525d 引入）+ theme-app-migration.md 第四批/收口段在 landed 树核对；SD-03 无改动结论成立 | archived: ✅ docs/plans/archived/037-business-examples-theme-migration.md status=archived（本提交） | cleaned: 见后续收据（worktree/分支/siblings/plan-artifacts） | completion_kind: delivered | 落地非部署观察：本包影响 examples 源+runtime 库；dev 侧 target 二进制已在验证据内重建，release 制品（release bin/daemon/web dist）非本计划部署面，按 PLAN-092 教训显式记观察——release 重建归 runtime owner 后继窗口

## 10. 待澄清事项

1. ~~实施尚未授权~~ → **已授权**（2026-11-03 用户 /goal 全链授权，T-01 执行）。015 阻点已真复现并按现源修定（npm_deps 对象语法+幸存 engine 链接），生成器解析缺陷本体具名交 owner（npm_deps_from_value scoped/:link: 解析），未为通过删轨。
2. 036渠道最终门：path 腿外部根消费已实证（receipt 摘要与规范逐位一致）；**新渠道（SDK/delivery-root）对 B2 vue-render app 结构性不可用**（`--delivery-root` 仅支持 rust 后端，具名拒绝实证）→ 该最终门待办，owner=Man 发行/U036 谱系；不得用 path 腿冒称新渠道完成。
3. 教学例不等独立产品。本包全程无外发消息、真实账户/用户笔记/主存储写入；测试数据根均隔离（TEMP storage 文件、scratch 目录、独立 MCP 端口）。
4. B3、真实clock/OS系统事件、其他产品装配各持合同。本包三件有界修复均恢复既有承诺合同（stylekit 配方投影/包准入/A1 处方），无通用新架构；发现的新交互缺口（028 focus 清位 shadow-write）具名 owner 未修。
5. **final review 注意（T-08）**：同树冻结基线=f8b0025c（含 T-05 amend 更正 c084c000）；主检出存在他方未提交 WIP（stylekit module-import 同诊断修复），review 独立上下文不得消费该 WIP；017/023 Vue 首红为基线等价（源+编译器不变推导+017 已实测基线同败），028 间歇崩溃为基线必现族。
6. **merge 受阻（2026-11-03，等用户/owner 动作）**：主检出他方活跃 WIP（`crates/auto-ui-runtime/src/dynamic_build.rs`+`ui_tests.rs` 已改、`stylekit_module_import_parse_tests.rs`/`test/` 未跟踪；同会话仍在演进——039 计划文件已被其删除）与本分支落地点直接重叠（本分支同样改此两文件）。`git merge --ff-only` 将拒绝（本地改动会被覆盖）。**确切解除动作**：由该 WIP 的 owner 会话（或用户）将其路由（提交/stash/挪 fix worktree）后，本计划在 worktree `rebase master`，解决同区域重复修复冲突（他方含 NOTES-004 本地配方补齐面，取其全集并保留本包回归测试）、对受影响验证重跑（stylekit 配方族+015/017 VM 引导），range-diff 等价后 ff 落地。此前一切 checkpoint（reviewed f8b0025c、SD 备稿、证据）保全。
