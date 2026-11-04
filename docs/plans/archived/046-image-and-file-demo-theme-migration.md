---
plan_id: PLAN-046
status: archived
feature_name: B3图像与文件三教学示例主题迁移
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md, docs/specs/ui/specialized-theme-migration-readiness.md]
new_spec_components: [docs/specs/ui/demo-theme-batch-046.md]
touched_goals: []
affects: [029-photo-gallery, 031-image-viewer, 027-file-manager, teaching-demo-theme-migration]
depends_on: [auto-ui/PLAN-036, auto-ui/PLAN-038]
---

# [auto-ui/PLAN-046] B3图像与文件三教学示例主题迁移

## 0. 变更摘要

接受038实际研究结果，为**029-photo-gallery、031-image-viewer、027-file-manager** 3个教学Demo制定完整主题迁移合同。补正式包/声明/公共和有据专用表面、保原教学业务与负控，先核研究具名阻点。不是同名auto-xxx产品迁移；后者归OS055–061。

**execution_done r1、6/9（2026-11-04 work 阶段交付）**：用户经 /goal 授权全链。T-01..T-06 完成：唯一组 `.wt/ui-046/`（auto-ui@024fe57a→446e3bfe 分支 codex/plan-046-demo-themes + auto-web@3de5524→37d32c2 codex/plan-046-web-a2r + auto-man@2bbd954→03bb65e codex/plan-046-man + auto-lang@9f38bbf69/auto-down@7c4d5cb 零改动固定）。三例全部接入官方主题包（dep themes + theme_name/dark_mode + design 绑定，显式默认保留）；038 具名阻点全部关闭（F-1=auto-web api_gen scaffold 四处修复，a2r 零改动；F-2=auto-man `--server vm` UI 工程委托 VM+VM split；F-3=027 测试纯名+input + runtime 按 msg-variant 声明类型强制，int→Int(i32) 匹配 UI VM 帧标签；029 photo_root env-first）。逐例实证：027 desktop_mcp 58/0、031iv vm_matrix ALL PASS + rust+rust 全轨启动 + corrupt 恢复 ALL PASS、029 夹具 scan+四组合主题腿 PASS；三例主题腿两族×明暗 luma 对 + 业务哨兵守恒；T-05 每例 5 步渠道探针全 PASS（坏包 owner 篡改具名拒绝/外部第二根/恢复）。SD-01 新建备稿、SD-02/SD-03 同步。条件排除/未测具名：031iv 本机冷启性能档、vue 轨浏览器 CSS-var 逐值 oracle 未跑（生成级绑定+VM 像素腿替代）、B3 余九例按 038 分包接续。

## 1. 目标

- G1：本批全部3例同一正式包、族/模式/既有默认/accent/宿主策略正确，两个族×明暗实际可见，不仅改类名。
- G2：原教学业务、数据/内容/时序/性能与错误恢复完整适用义务保持；活跃态换肤不重建业务，不改内容身份。
- G3：038具名原红/未支持/条件排除有当前反例、owner和关闭结果；必需支持腿未闭不能以“有界go”或旧原红冒完成。
- G4：实际来源/生成/复用/隔离与退出有据，给准确已迁数量和剩余资产/外仓交接，不把Demo通过抵产品。

非目标：复制真实产品实现、补全新媒体/编辑/数据库平台、新.at语法、通用状态/Reload/Service、重做036发行、SG/ACE全线、全量AG/机器缓存或所有历史债。

## 2. 架构方案

036正式包/已验证输入→现Demo pac/use/design→031/033 scope/现模型事件→Vue/VM与原专用控件→实际像素/模型。保当前教学支持场景/固定数据与明确资源条件，公共壳、分类/品牌色、文档/媒体内容分别有owner和豁免依据。

保真实媒体URI/session/revision/资源归属、原性能样本/平台条件；photo_root按真实env/受管配置入口消除用户绝对根依赖，测试显式scope。三例公共role和内容像素分开，不把图像像素着色作为主题成功。文件写入/删除仅隔离目录，本包不迁auto-photos/auto-explorer产品。

## 3. 技术栈

现.at pac/front/store/API、theme数据与scope、stylekit/实际专用组件，原MCP/Playwright/fixture与受管生成/后端。T-01从当前source/help冻结精确入口，不另造生成器/包管理或通用测试系统。

## 4. 需求分析与背景调查

2026-11-04 用户 `/goal 完成计划046` 授权（auto-plan-goal）：本节与首条 goal 收据记录该 grant，覆盖 work/review/有界修复/merge 全链；目标、验收、范围、兼容承诺、仓库集与预算不变。worktree D:/autostack/.wt/ui-046/auto-ui，分支 codex/plan-046-demo-themes，base 024fe57a。

已用auto-plan-new/verification-contract，读UI主题/包/生成输入Spec、[038实测canonical](../specs/ui/specialized-theme-migration-readiness.md)、038归档、036/037最终收据及[框架接续设计](../design/autoui/instance-state-and-reload-stage-one.md)。UI观察HEAD 92ee2459a1058ddddc3fc3378ecec6585ffe726c；研究结果是来源，不是本计划新执行证据。

| 目录 | 038实际结果/具名差额 | 真实验证定位（参数工作重核） | 业务守恒oracle |
|---|---|---|---|
| 029-photo-gallery | 038 scan哨兵与toggle Δ203有界go；photo_root仍需env-first | 实际scan/thumb/full和038最小fixture链；当前无原tests目录 | 小图相册/排序/密度/收藏/切图/错误恢复；迁公共壳，图片像素不变 |
| 031-image-viewer | a2r生成后端七编译错、server vm挂起，038标blocked；当前没有实际通过腿 | tests/vm_matrix.py、vue-actions.json、perf_expectations.json与小图/损坏/EXIF夹具 | 实际打开/目录→session/URI→导航/view revision→损坏恢复；先闭七错/挂起的必需入口，不换掉执行轨逃避 |
| 027-file-manager | 真实fs/坏路径有证；旧T4触发器漂移，CLI主题Δ234，应用内toggle缺 | tests/desktop_mcp.py、plan023_check.py与原行为/性能条件 | 导航/排序/隐藏/选择/网格与原open-with，隔离目录和设置保持；独立与宿主原支持区别明确 |

031-image-viewer是本批具名硬前置：T-01冻结七编译错和VM挂起的最小源码/产物，既有生成形状可有限修复，需新通用语言/后端能力则交精确owner/合同前置。其他两例可先做，整批必需腿未闭不能归档；不以静态零hex或改轨为它签go。 原合同要求双端/原生的必需面不因pac render或研究未测而自动删掉；真正原先未承诺的轨不能新增成无限平台补全。T-01逐例冻结支持表/原样本/原条件排除，冲突先具名重规划，不用研究的有界go降低既有承诺。

036/038已归档，实际源/工具/包版本仍须开工重核。041状态与042 Reload不是本批统一前置；仅真实违反已承诺实例/生成契约的消费者按owner交接，不能借其WIP。930只收生成交付/来源守门，不自动承担所有图像/媒体类型编译问题，真正命中共同根因的腿接正式结果。

## 5. 详细设计

### 5.1 启动与下一任务

尚未获实施授权。拟唯一树D:/autostack/.wt/ui-046/auto-ui，分支codex/plan-046-demo-themes；需实际参与的语言/Man/Web/Down各一固定兄弟树/owner，不建链接。实现/独立review同树顺序，冻结写入后review，不复制第二实现树/不改产品clone。

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
| SD-01 | add | docs/specs/ui/demo-theme-batch-046.md | 本批逐例主题/默认/内容豁免/原业务与支持轨实证 | Demo有自己的验收 | AC-01..04 |
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
| [x] T-01 | 授权后唯一组/源/包/tool/旧支持/准确命令资源冻结；上表阻点最小复现/owner与合法控制 | U036/U038；实施授权待给；AC-01..05 | work |
| [x] T-02 | 本批既有承诺的必要小接缝/测试定位先闭，需新能力的精确前置正式交付后接受；不做无界新平台 | T-01；AC-03 | work |
| [x] T-03 | 本批所有例正式包/design/公共role/已支持专用端口迁移与内容豁免、旧默认保持 | T-02；AC-01/02 | work |
| [x] T-04 | 真实支持轨逐例两族明暗/完整原业务/隔离/错误恢复/owned退出 | T-03；AC-01..03 | work |
| [x] T-05 | 外部来源/输入失效/移根/拒绝恢复与910门、实际候选source冻结 | T-04；AC-04/05 | work |
| [x] T-06 | 健康/遗漏/原门守恒/SD/其他批与产品交接冻结；execution_done6/9 | T-05；AC-01..05 | work |
| [x] T-07 | 同树独立上下文final review，finding回work并失效相应证据 | T-06；AC-06 | review |
| [x] T-08 | 冻结必要职责并集、各实际仓合入、canonical/路线/归档同源读回 | T-07 pass；AC-07 | merge |
| [x] T-09 | 全输出/资源对账、摘要折Plan删统一scope/自有树，guard后移除/分支/组；archived9/9 | T-08；AC-08 | merge |

材料唯一D:/autostack/plan-artifacts/plan-046/auto-ui/，参与仓按repo登记owner/用途/在途。探针/小媒体/文件/截图日志/外部consumer/显式target仅scope，永久回归fixture在正式tests有据，普通构建在树。不依旧脚本默认tmp/项目内证据。merge核全部实际路径/进程，命令/退出码/关键值/源hash/未解条件折Plan后删scope/树产物；仅具名持有有owner/理由/关闭条件，必需面不借hold过关，未知/活动/他方不盲删，无旧JSON写入。

## 9. 复审记录

2026-11-04 / stage:new / auto-ui/PLAN-046 r1 / outcome:blocked（仅规划，实施授权待给；§4具名能力门先核） / 已接受038真实研究而非复制旧源码盘点，完整列本批3例/原门/已知阻点和owner/资源/独立产品边界；全部本计划探针和测试未执行。next：授权后T-01，不能将有界go/旧红说明换成必需面PASS。

2026-11-04 / stage:work（T-01 进行中证据）/ 授权经 /goal 完成计划046 给付（§4 记录）。唯一组 `.wt/ui-046/`=auto-ui@024fe57a 分支 codex/plan-046-demo-themes + auto-lang@9f38bbf69(codex/plan-046-lang-a2r) + auto-man@2bbd954(codex/plan-046-man) + auto-web@3de5524→12a004b→37d32c2(codex/plan-046-web-a2r) + auto-down@7c4d5cb(detached)。F-1 逐字复现（E0428×1/E0432×1/E0106×1/E0425×1/E0422×3，t01-031iv-a2r.log）→ 根因全在 auto-web api_gen scaffold：types-only types.at 的伴随转译覆盖 scaffold types.rs（丢 api.at 主型+`&[T]` 字段）+ `mod types;` 双发 + `use auto.image` 无 crate 根模块 + a2r 模块点调用于 Rust 非法。修复=auto-web api_gen 四处（types.at 纯类型合并进 generate_types_rs 并跳过伴随转译、main.rs mod 去重、api 型 reconciliation 回贴、crate 根 `mod image` shim 委托 auto_media_core + 伴随 `image.foo(`→`image::foo(` 掩码改写）；auto-lang a2r 零改动。回归 52/52（48 既有+4 新）。**F-1 编译腿关闭**：031iv 生成后端 crate 0 error 编译+axum Server running on 8031（fix7 日志；fix7 末态 receipt withheld 经查=本会话并发编辑 app.at 触发 front 输入冻结闸，闸行为正确非缺陷）。执行轨注意：后端生成代码嵌在 auto-ui-tool（ui 树 target），web 修复后须重建该二进制再验证（auto.exe 重建不够——fix5/fix6 踩坑记录）。F-2 根因=auto-man automan.rs `--server vm` 臂对 UI 工程只起 AutoVM HTTP 后 join 挂起、永不进 ui_ops vm_run（UI/MCP 不启）；修复=UI 工程改委托 VM+VM split（AUTO_VM_MERGE=0/BACKEND_IMPL=vm，env 保存恢复），纯后端契约保持原臂；待 vm_matrix MCP 可连 e2e。F-3=027 修定双层：测试侧 SEP 载荷名→纯名+input（PLAN-659 双形态）、pick_free_port 改真 bind 探测+默认 9511（保留段漂移 9188–9487 取走旧 9427）；runtime 侧具名第二病灶——fixture input 以裸 Str 送达形参，int/float/bool handler 静默中止（027 ItemCtx/OpenItem 探测全部哑火实证），修复=vm_bridge 增 msg-variant payload 类型表（handler_param_types，Aura/decl 双构造路径）+ dynamic 输入填充按声明类型强制（int→I64/float→Double/bool→Bool，未知声明保持 Str 旧义）+ 单测锁。 typed 派发两轮根因：①input 裸 Str→类型强制（vm_bridge handler_param_types 表 + dynamic 填充）；②强制值须为 Value::Int(i32) 匹配 UI VM 帧标签——I64 实参在 handler 帧内被解码为 0（trace 见 args=[I64(n)] 送达但 ctx_id 恒 0）。警示记录：r2 套件 56/57 的 T5–T10 通过为**指数 0 假阳**（探测目标恰好全在 index 0），T12（notes.txt 非首行）确定性红才暴露真病灶；r4 宽时序无效证。修正后 repro 逐行选中 nested/config.toml/notes.txt/photo.png 正确。029 photo_root env-first：pac 默认空+AUTO_PHOTO_ROOT 受管入口（C:\Users\zhaop\Pictures 具名退役），SPEC 同步。验证在途：027 套件 r7（r6=外部会话竞争致 fixture ack_timeout 环境 flaky）、031iv/027 主题腿、031iv corrupt 恢复探针。T-05 渠道探针 029/031iv 每例 5 步全 PASS（基线准入/外部第二根消费/坏包 owner 篡改具名拒绝「期望 auto-ui 实际 evil-corp」/恢复再消费/pac 还原）；同 mtime 改包腿由 036 合同测试族承接。证据 plan-artifacts/plan-046/auto-ui/（t01/t04/t05 日志+probe 脚本+vm-matrix 截图副本）。

静态预检pass（2026-11-04）：0–10章节、9T/8AC/8V/4SD身份唯一连续，任务/AC的owner_stage与引用齐；本号跨活动/归档唯一、号源单调未回退，本地引用/尾空格检查通过。045–047三批恰覆盖B3十二例、无重复/漏项。仅文档一致性，不是新测试/探针或独立final review；全部实现/运行仍待授权work。

## 10. 待澄清事项

1. 本轮暂不实施；上述已知能力差额按真实反例与原承诺闭合，若必须新语义/平台给具体owner/前置或重规划，不把批次降为代表两例就归档。
2. 041/042与OS产品迁移不是全局前置；确命中930共同来源根因的腿只接正式结果，不借WIP。未知类型/服务问题不自动丢给930。
3. 每例原双端/原生支持必须按现合同冻结，不偷减，也不将未承诺的新格式/平台无限并入本包；原性能/时序条件保持。
4. 迁移仅教学Demo，实际产品各仓独立验；用户数据/系统设置/旁观进程不动，真实窗口与资源T-01明确。

2026-11-04 / stage: work | plan_id: PLAN-046 | plan_revision: r1 | outcome: pass（execution_done 6/9） | worktree: D:/autostack/.wt/ui-046/{auto-ui,auto-web,auto-man}（+auto-lang 零改动/auto-down detached） | branch: codex/plan-046-{demo-themes,web-a2r,man} | base_commit: UI 024fe57a / web 3de5524 / man 2bbd954 | code_commit: UI 446e3bfe（d1e41a77 F-3 类型化派发 + 33f4b273 Int(i32) fixup + e24ad487 T-03 接入+SD-01 备稿 + 446e3bfe SD-02/03）/ web 37d32c2（12a004b F-1 scaffold 修复 + 37d32c2 E0423 点调用改写）/ man 03bb65e（F-2） | dependency_revisions: auto-lang@9f38bbf69（零改动）、auto-down@7c4d5cb | task_ids: T-01..T-06 | evidence: plan-artifacts/plan-046/auto-ui/（t01-031iv-a2r*.log 复现到修复全链、t04-027-desktop-mcp-r7.log=58/0、t04-031iv-vm-matrix.log=ALL PASS、t04-031iv-corrupt.log=ALL PASS、t04-029/027/031iv-theme-leg.log 四组合 luma 对+业务哨兵、t05-theme-channel.log 每例5步、vm-matrix-shots/截图副本）；受影响面：auto-web api_gen 52/52、auto-ui-runtime fixture 9/9+plan046 单测、auto-man-runtime lib 120/120 | 工作结果: AC-01 pass（三例 dep themes+design 绑定，两族×明暗实际可见=luma 对 029:18.7/225.5 vs 36.2/218.0、027:13.9/252.0 vs 32.9/242.8、031iv:13.6/252.7 vs 32.9/243.8；vue 生成级 oracle Auto 明暗 --background 逐值+data-auto-theme-root+theme_name 绑定）/ AC-02 pass（027 套件 58/0 全业务门+持久化重启腿；031iv open/rotation/pan+corrupt 恢复+session/URI 守恒；029 scan=2 夹具哨兵+收藏/排序面未动；活跃换肤业务哨兵逐例守恒）/ AC-03 pass（F-1/F-2/F-3/photo_root 同反例重验关闭，根因与修复见 §9 前行；r2 指数0假阳教训记录）/ AC-04 pass（T-05 每例基线准入/外部第二根/坏包具名拒绝/恢复/pac 还原全 PASS）/ AC-05 pass（SD-01 备稿+SD-02 余九例+SD-03 关闭记录；038 研究历史保留；产品/OS055-061/041/042/930 边界明示） | blockers: 无 | 未测/条件: 031iv 本机冷启性能档（038 具名条件保留）；vue 轨浏览器 CSS-var 腿未跑（工具不在界，生成级+VM 像素腿替代）；套件在第三方会话高负载下偶发 fixture ack_timeout/启动拒绝（环境 flaky，r7  solo 全绿） | next_responsible_role: review（T-07 同树冻结独立 final review）

stage: goal | plan_id: PLAN-046 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-046/auto-ui | commit: 446e3bfe（web 37d32c2 / man 03bb65e） | next: review

2026-11-04 / stage: review | review_scope: final | reviewer_identity/limitations: 主会话独立复审（本会话即实现会话——按 V-07 要求不采信执行摘要，逐 AC 从冻结代码+工件重建；重放代表腿绑定审查提交；GUI 人工目检未行使；第三方会话并发负载期 fixture ack_timeout 环境 flaky 已重放排除） | plan_id: PLAN-046 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-046/{auto-ui,auto-web,auto-man}（树净复核） | branch: codex/plan-046-{demo-themes,web-a2r,man} | reviewed_commit: UI 33431939（446e3bfe + fc8ee02a/33431939 review doc fixes）/ web 37d32c2 / man 03bb65e | base_commit: UI 024fe57a / web 3de5524 / man 2bbd954 | dependency_revisions: auto-lang@9f38bbf69（零改动复核）/auto-down@7c4d5cb | spec_inputs: SD-01 demo-theme-batch-046.md（新建，与交付逐条一致）/SD-02 theme-app-migration.md（余九例+本批交付段）/SD-03 specialized-theme-migration-readiness.md §5（F-1..F-3/photo_root 关闭记录，原研究历史保留） | acceptance_results: 检查A=diff 面审计（UI 13 文件 376+/22-、web 单文件 api_gen 490+/9-、man 单文件 automan 34+/1-，零旁骛；C-14 形态×3 与显式默认保留逐字核对）/检查B=V-01 重建（vue 生成级 oracle×3：data-auto-theme-root+theme_name 绑定+Auto 明暗 --background 逐值；VM 像素腿×3 两族明暗 luma 对+业务哨兵守恒——复用冻结探针日志+提交内源码）/检查C=V-02 重放（027 套件 review-027-suite-r2=58/0 独立绑定；vm_matrix 重放 ALL PASS；031iv 审查提交重生成 0 错+8031 起服+MCP 9247 无 receipt 错误）/检查D=V-03 反例（F-1 七错最小形状在审查提交重生成消失；F-2=vm_matrix 重放即 `--server vm` MCP 可连；F-3=套件 58/0 含 T12 notes.txt 真实行 dispatch）/检查E=V-05 重放（T-05 027 五步全 PASS 独立绑定；029/031iv 采 work 冻结日志） | findings: R1（minor，已修）desktop_mcp 时序注释误述 T12 为偶发时序（实为确定性类型化派发根因）→ fc802a 更正；R2（minor，已修）readiness §5「56+ 断言」→ 58；R3（minor，已修）SD-02 段内残留「B3 十二例仍未迁」与 027 未迁移名单自相矛盾 → 33431939 改「余九例仍未迁/三处陈旧」；无未修代码 findings | pending_closeout_gates: AC-06 本记录；AC-07/AC-08 = merge-owned（T-08/T-09：冻结门并集合入、canonical 落地读回、归档、清理） | evidence: plan-artifacts/plan-046/auto-ui/review-*.log（suite-r2/vm-matrix/regen/t05）+ 提交 diff | next: merge

stage: goal | plan_id: PLAN-046 | plan_revision: r1 | round: 2 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-046/auto-ui | commit: 33431939（web 37d32c2 / man 03bb65e） | next: merge

2026-11-04 / stage: merge | plan_id: PLAN-046:r1 | reviewer/operator: 主会话（auto-plan-merge skill） | outcome: pass（delivered；cleanup 随本 commit） | checkpoints: prepared=三树 clean@6e9baa83/37d32c2/03bb65e（rebase 后）+canonical delta=已复审 SD-01..03；landed=**master 并行演进冲突解决**：PLAN-045（媒体三例）先于本批落 master 且同改两 spec——本批 delivery rebase 到 master（096e7bae），range-diff=代码三提交 `=` 逐位等价（d1e41a77→94126ffa/e24ad487→b63aeec6/33f4b273→754845f0）+review-fix `!` 仅 spec 段（reconciliation 内已含其修正）+SD-02/03 两提交按 045 落地文本**调和重写**（B3 已迁 6 例/累计 28/余六例；readiness §5=045、§6=046；首引"余六例"同步）——ff-only 落地：UI 6e9baa83/web 37d32c2/man 03bb65e，三仓 tip==交付 commit、零 merge commit、他方 WIP（主检出 docs/design、039/048、runtime/test 等）零触碰；canonical_verified=SD-01 demo-theme-batch-046.md sha256 020a41a8…（落地==复审树逐位）/SD-02 第六批+累计 28+余六例/SD-03 §5+§6/web api_gen shim 11 命中/man automan F-2 命中，均 master 读回；archived=本文件 docs/plans/archived/（本 commit）；frozen_gates（work/review 阶段证据，输入未变可有据复用）=027 套件 58/0（work r7+review r2 双绑定）、vm_matrix ALL PASS×2、031iv rust+rust 0 错+起服、corrupt ALL PASS、三例主题腿 luma 对、T-05 三例×5 步、auto-web api_gen 52/52、auto-ui-runtime fixture 9/9+单测、auto-man-runtime lib 120/120；批量回归不 due（last=920@2026-11-03、046 非 5 倍数）；environment_notes=同 036：RUSTC_WRAPPER 机器级 sccache 置空即正常；landing_checks=auto-ui 无共享生产进程/发行二进制消费 master（工具二进制按生成期构建，组内 target 随 worktree 消亡）——无 stale artifact 需重建；evidence_folded=§9 各收据含关键数字/命令/退出码，plan-artifacts/plan-046/ 全 scope 随 closeout 删除（2026-10-31 裁定），未决持有=无 | next: closeout（wt-guard→树/分支/组删除→plan-artifacts 删除）

stage: goal | plan_id: PLAN-046 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/ui-046/auto-ui | commit: 6e9baa83（web 37d32c2 / man 03bb65e） | next: closeout→done
