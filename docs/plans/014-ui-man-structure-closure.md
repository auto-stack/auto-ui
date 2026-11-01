---
plan_id: PLAN-014
status: executing
feature_name: UI/Man内部结构收口（保持行为与既有crate粒度）
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-11-01
plan_revision: 2
supersedes_spec_components: [docs/specs/README.md]
new_spec_components: [docs/specs/ui/structure-closure.md, docs/specs/ui/structure-test-map.md]
touched_goals: []
affects: [auto-ui-compiler, auto-ui-runtime, auto-ui-tools, auto-man-cli, repository-boundaries]
current_step: 3
total_steps: 11
---

# [auto-ui/PLAN-014] UI/Man内部结构收口

> r2 / executing / 3/11（2026-11-01）。实施授权=2026-11-01用户goal：先推进T-01/T-02及Man CLI（T-03），再做Vue（T-04）；共享接口改动、renderer/GPU验证和最终合入安排短窗串行；不把020完整交付作为014全局前置。T-01/T-02冻结与T-03实施已完成：Man worktree提交5b310f0（基cbbe657），验证unit 24/24、tcore 24/24、plan816+plan890+cmd_ui 10/10（各1既有环境skip）；发现并修复901遗留consumer漂移（Man死转发feature `with-file-history`，master对lang b1a4871a本不可编译）。
> 唯一主责auto-ui；Man参与CLI结构，语言/Web只接受稳定公开依赖及必要消费/规范指针。890、900、901、018均已有归档交付记录；本计划保留其实际支持边界，不重做迁移、feature拍平或发行闭包补全。
> 设计：[UI/Man结构边界](../design/autoui/ui-man-structure-closure.md)。该设计r1的U890等待/旧feature与证据条款由本计划r2更新，四面目标保持；总路线v20仅历史，不覆盖本次实物核查。

## 0. 变更摘要

按较大完整模块粒度交付M1 Man CLI、U1 Vue生成、U2 Aura构建、U3 renderer的内部职责与状态/资源所有权收口。包括现有入口接线、原件/重复退出、测试随owner整理、独立消费者、真实UI/工具验证、规范与收尾；内部调查/阶段不单独立Plan。

接受890的Web交付、900的owner归位、901的语言测试/SDK发行闭包、018的UI稳定编译与运行选择基线，保持语言/UI/Man职责及公开行为，不重做搬仓。结构改变必须能从生产调用、模块职责、状态/资源owner和测试证据观察；源码搬动/文件变短本身不是完成判据，也不自动代表更快或更省内存。

## 1. 目标

- G1：四个必做面均有具名职责/调用图与单一状态owner，旧入口仅保相应编排/兼容，不复制算法/全局态或造common巨包。
- G2：CLI、Vue生成/资源、Aura绑定/惰性/挂载身份、renderer消息/active/帧/输入/生命周期保持现行合同。
- G3：语言SDK、UI compiler/runtime/tools、Man Core/full、Web的公开入口及依赖闭包保持可独立消费；测试义务与强oracle守恒。
- G4：通过实际工具进程、外部生成工程、现有Vue/VM交互、原生GPU及owned退出/恢复验证，给UI-007可接续的稳定结构与证据。
- G5：独立终审、冻结门、所有实际修改仓合入、canonical Spec/归档guard完整后才U(auto-ui/014)。

非目标：新建/再次迁移UI/Man/Web仓、重搬890 HTTP算法/模板、整棵examples/ui归位、新主题树/extend/mode/`.Design`/Theme Lab、官方主题包、UI-007的002/005主题改写、Feature/Service/HMR架构升级、协议/公共ABI/语义修改、默认feature/A1调整、细拆crate、性能专项、业务应用/OS产品或全部既存债修复。现有主题行为必须保持；新增主题能力留007。

## 2. 架构方案

沿[Design37](../../../auto-lang/docs/design/37-v06-modularization-and-repo-boundaries.md)及[873后范围](../../../auto-lang/docs/plans/reports/post-873-structural-work-packages-2026-10-29.md)，默认内部串行M1→U1→U2→U3。以901/018正式交付后的crate/member、兼容别名、真实可选能力和profile为保持基线，本包不再改变它们；在既有包内按职责提取主题模块，用显式借用/上下文访问唯一状态，保公开路径及类型身份。

| 面 | 已观察入口/数据流 | 结构目标与持有关系 |
|---|---|---|
| M1 | `auto-man/crates/auto/src/main.rs`→已有`cmd_*`/`tool_ui_ops.rs`→runtime/真实工具/Web公开端点 | CLI解析声明、分派、诊断/机器输出、命令编排分清；优先复用现有cmd模块/ToolUiOps，不造第二工具发现或owned进程设施 |
| U1 | `crates/auto-ui-compiler/src/ui_gen/vue.rs`的`VueGenerator`→SFC/script/template/style/library/store→tools生成产物 | 一份生成器state/registry，分主题实现；复用`ui_gen/import_policy.rs`与`shared/`。保旧模式/字节/导入、reset、source fingerprint与缓存失效语义 |
| U2 | `crates/auto-ui-runtime/src/ui/aura_view_builder.rs`的`AuraViewBuilder`/build/build_with_debug→节点/绑定/槽/路由→View | 节点转换、表达式/绑定、跟踪与构建职责分开；VmBridge只借用，Bindings/调试path/槽上下文有唯一owner；保惰性、求值次序、键/路由/挂载身份及guard恢复 |
| U3 | `crates/auto-ui-runtime/src/ui/iced/renderer.rs`的IcedMessage、消息/订阅、dynamic_view*→窗口输入/呈现 | 消息/状态/任务与view投影、图标字形/部件、纹理/快照及宿主装配分开。状态仍归当前session/实例；先冻结owner/消息顺序/active判据与资源生命周期再搬动 |

生产源码主体必须按实际职责形成可解释模块；主入口可以保清晰分派，不能把同一巨正文换成`include!`或仅挪测试声称达标。测试外移保其原子义务。若890/900/901/018正式基线已完成某子职责，T-01/02逐符号证明并复用已有交付，不为本Plan人为再拆；四面均必须验收，不能据此跳过整面。

## 3. 技术栈

既有Rust/Cargo、clap/I-TOOL/I-PROJECT、Vue SFC/TS生成、Aura/View/VM/Iced及现有资源owner；Rust/libtest/nextest和既有测试治理/发行consumer。无新增框架/引擎/通用解析器/缓存系统。计划内可新增必需定向回归、职责规则与小型fixture，不另写通用测试执行器/视觉框架。

## 4. 需求分析与背景调查

### 4.1 本次授权与知识来源

2026-11-01用户要求按900交付及901、018合同核清014剩余范围和交接，暂不实施。本轮只修订原014为r2，保留11任务/12AC/14V/6SD；不代改其他Plan/产品/号源，不建树、不运行测试/GUI或联系其他会话。原2026-10-30纯规划授权和014→015取号为历史，本次不重新取号。主检出其他docs/设计WIP保全，不全量add/reset/stash。

已读本仓docs/specs/README.md、build/{ui-compilation-configuration,ui-test-selection}.md、runtime/{native-vm-app-validation,vue-bare-store-routing}.md，及890/900/901、018归档收据；语言docs/specs/overview.md、auto-ui-{compiler,runtime,tools}/project.md、repository-boundaries/generated-ui-distribution.md为已有owner/消费知识入口，滞后页首不覆盖新交付。按auto-plan-new及verification-contract修订，本次不是实现或独立终审。

### 4.2 已交付结果与剩余职责

| 来源 | 实际交付及限制 | 014接受/剩余差额 |
|---|---|---|
| [890 r7](../../../auto-lang/docs/plans/archive/890-auto-web-independent-repository-delivery.md) | archived12/12，Web公开端口/adapter、四仓合入与清理完成 | 不再等Web搬迁；只整理CLI/UI的调用编排，不回迁HTTP算法/模板/VM adapter |
| [900](../../../auto-lang/docs/plans/archive/900-de-ui-dependency-inversion.md) | archived10/10；UI管线/host安装、LSP/cosmic、测试/example已归UI，语言零auto-ui包边 | 不再次搬仓；owner归位不等于M1/U1/U2/U3内部职责已收口；900-N7 Web轻合同path边独立跟踪 |
| [901 r2](../../../auto-lang/docs/plans/archive/901-test-feature-flattening.md) | 已归档并有最终goal读回pass；落地lang2fcae41ba、UIe1fe4ca、Webec9c4ba；语言七测试feature退出、常驻TestCase及SDK投影/闭包补全 | 不恢复旧test-*，不复制发行/TestCase系统；结构新模块须进入现有source清单/指纹并重验实际消费，不让B补本包破损 |
| [UI018 r1](archived/018-ui-build-configuration-closure.md) | archived8/8，UI实现968ac83、归档6ab7f2d；runtime基线恒编译、ui-iced兼容别名、三个测试feature退出，optional能力与平台条件保留；两篇build Spec已在本仓 | 不再次拍平或改默认依赖；保当前t/tu/ti/tn、静态过滤、HTTP/GPU串行和真输入手工门。新结构使相应证据失效时重验 |
| 018验证限制 | 收据记日常2960/2960、ti28/28、5733文件fresh/locked消费、窗口冷启/关闭/重开及widget管线级真实事件控制；完整NVM-03 MCP输入腿未执行，Linux cosmic原生无环境未执行 | 不包装成全链原生输入或跨平台已验。014 V-09/10/13必需真实输入/GPU/后端/owned关闭重开保持，不借前Plan未执行项skip |
| UI007 r3 | drafting0/10，主题/官方系列/示例归位独立合同 | 014不迁主题/整棵examples；900的28个显式example目标不等于007整棵examples/ui已归位，向007交实际差额 |

### 4.3 实物快照（T-01开工重绑）

本次读取UI master6ab7f2def391eca584ebbbbbf375ef75a7f64ec9；Man cbbe6573e2d40e3237f52718dd02a3cd065d5cbd；语言b1a4871aa6a8d960130fc2e1aed8bf6dc4183c5a；Web ec9c4bac90fed62e6b651df7789153149e8a9473。不是未来冻结版本。

| 面 | 当前入口/状态与SHA256 | 剩余结构判断（只读，未实施） |
|---|---|---|
| M1 CLI | Man main.rs3223行；Cli约144、Commands约441、main约975；已有cmd_service/cmd_ui/cmd_watch；hash de5223e729f530aca5f6aa86bd1c18d4c428a3d681ec1558f1db947b1bc4efce。ToolUiOps446行hash747472e0ed16177da84aa8ed1e7b6702deb3583143da7fea4ad141fc0f679646 | 复用cmd/工具/owned设施，仍核解析声明、分派、诊断、编排归属；不从“尚无模块”的旧假设重拆 |
| U1 Vue | vue.rs34615行；VueGenerator约419，impl约918/20357；hash fe786c94219310355b053355534dd91efa11aba04b1899fb27b374acbfd87172 | 复用import_policy/shared/widget；保单generator state/registry、store_surface/store_import_prefix/reset；剩余template/script/style/library/store职责提取 |
| U2 Aura | aura_view_builder.rs28054行；builder约135、impl约309/24333/25513；hash2fed0ad5702d32123b806f6d75185eabec2e290b89dfbe2a80d8385e1d5917c3 | bridge借用、override_state_obj_id/computed/routes/slot_fills/preview/nav上下文仍需按节点/绑定/构建跟踪整理；不复制状态或改求值/生命周期 |
| U3 renderer | renderer.rs42295行；dynamic_view族约25847起，component update/subscription约31550起；hash4c4543bd48672ba56a28f640628512c2ada46261bb4abfcf76d97e8e49a14212 | 018删cfg并未完成内部职责拆分；消息/active/订阅/view、图标纹理/资源/宿主仍需按调用和唯一owner整理 |

行数含测试/注释，仅定位，不是净删/速度/完成阈值。所有测试数据是前计划归档摘要，本轮未枚举/运行测试、未读取已删除的原始材料，不认证新代码PASS。

## 5. 详细设计

### 5.1 启动门与唯一权威树

**G014 = 用户明确批准014 r2实施 ∧ 接受U890/U900/U901/U018正式交付组合**。本轮已核四项有最终交付记录，不再写“等待901/018实施”；当前仍未授权开工，T-01须对届时实际版本/共享写面重新冻结。建树、实施盘点/探针/fixture、构建测试和GUI均属于work，本次修订不包含这些动作。

X014是具体文件/符号、正式来源和接受差额；归档不等于工具/桌面窗口无人使用。唯一预定组D:/autostack/.wt/ui-014/{auto-ui,auto-man,auto-lang,auto-web}，UI分支codex/plan-014-structure；仅实际需写入/验证的仓各一树，其余固定detached依赖，必要auto-down补同组兄弟。开工重读各仓指令，禁止覆盖现有树或建junction/symlink；实现/内部核查/独立终审同组顺序，Plan只在UI主检出簿记。不授予远端发布、改历史或用户运行进程升级权限。

### 5.2 跨Plan责任与顺序

| 对象 | 014接受/排除 | 先后与交接 |
|---|---|---|
| 890/900迁移 | 接公开Web/SDK端口、UI pipeline/host装配、LSP/cosmic/测试/example归位 | 已交付；只整理四面内部职责，不复制回语言或再次搬迁，不追加所有应用验收 |
| 901（B） | 接常驻TestCase、语言feature退出、HTTP helper和完整SDK发行/投影/锁 | 已交付；结构新模块的清单/指纹接线归014；不恢复旧门或新造另一发行系统 |
| 018（D） | 接runtime稳定基线、ui-iced兼容别名、真实optional能力、测试静态运行选择及其公开限制 | 已交付；不得按r1旧基线复原门/无后端stub。结构触面使旧证据失效时重新验证；不把018有限输入控制冒完整NVM |
| UI007 | 排除新主题树/extend/mode/.Design/官方包/Theme Lab、002/005主题改写与整棵examples迁移 | U014后按007合同重绑；交模块/入口/状态owner/测试资源映射、精确SHA与900后实际示例差额，不反向等待007能力 |
| 测试治理/899/ACE | 接896/898固定组与901/018选择合同；只用稳定实现 | 实际共享文件短窗单写；不借WIP、不加无关全线前置、不复活tm；语言all不含ACE，真共享SDK改动才补具名ACE验证 |
| AG/OS/应用及其他计划 | 仅固定公开依赖和实际受影响consumer | 无交集不等整Plan；有renderer/解析/工具/资源交集核正式diff及窗口，不停别的进程，不发未经授权消息 |

最小X：owner/合同版本→已合入SHA及路径符号→协议/类型/资源/工具身份→014接受差额和保持项→真实消费者/测试ID及失效条件→限制/回退→最终回交。T-01定位来源与入口，T-02冻结具体模块/借用/消息资源设计及每V精确命令后再做T-03..06，不凭大文件行数预设全部函数都要搬。

### 5.3 结构、状态与兼容

T-01/02建立唯一的职责表：每面函数族、调用关系、状态/缓存/资源owner、线程与借用、正常/错误/取消/关闭、原入口与后继模块、消费者/测试。T-02产出有界全路径对照，选择局部机制后才重排；不预造新SDK签名，接受890/900/901/018正式公开接口。

CLI现有help/flags/默认值、机器JSON、stdio/退出码、cwd/env、Core/full能力缺席、真实工具选择/重试/取消/watch/owned reader和子树结束保持；服务调用委托U890，不改`service`/daemon `serve`分工。Vue plain/shadcn/library、import_policy/shared、store方法路由/生成状态/reset及缓存准入保持；生产字节默认不变，源移动导致指纹变化必须如实失效重建，不伪造旧fingerprint。只有预先列明的provenance/绝对路径等非语义元数据允许单字段规范化，原golden不重新生成洗绿。

Aura保tracked/untracked现有差异与合同、Bindings/slot/for/key/route、惰性分支和求值顺序、挂载/初始化/调试身份以及上下文guard恢复。renderer保相同消息序列的状态/任务/订阅/active判据、frame与呈现、焦点/输入、资源owner与最后拥有者退出；已授权external pin有明确寿命，不强制提前清零。不得复制状态到主题helper，不能以改变active/tick/队列/预算或吞错来修结构测试。

只保必要合法pub/re-export及长期owner，不通过扩大SDK私有pub面、恢复旧兼容根或循环依赖解决访问。源码/测试/生成/第三方与移仓/净删分列，无≥2000净删、行数上限、提速/CPU收益门。四面已有合理子模块复用，未收口残项必须具名，不能以“未来007会重写”延期本合同必需项。

### 5.4 测试与真实消费

原ID/聚合子case/断言/oracle/条件/档位建立旧→新映射，零未解释丢失/新增红。测试重命名可有证映射，不能用改floor/golden、仅list、wrapper总数或删测试达成。复用867/887义务、890正式Web消费及896/898固定组、901/018实际ID和运行选择；不复活tm或新造动态目录。

既有真实工具进程/CLI driver、UI package/runtime consumer与生成发行consumer优先复用；必要时新增本仓定向测试族`structure_closure`及小型fixture（新路径由T-02冻结，不将拟建目标声称已运行）。可将当前002/005或已有UI003 native control作固定只读消费者，选择依据覆盖实际改动，不复制整棵examples，不改业务/新主题；所有触及的现有golden和case必须覆盖，代表app不替代完整定向义务。

生成SFC/golden字节、真实Vue工程构建/运行与VM原生交互分开验。renderer要求实机窗口身份/真实输入/状态读回、至少一项GPU呈现/资源路径和owned关闭后重开，headless计数/首帧/截图存在均不足；与Vue共用语义做同源双端回放。遵本仓native-vm-app-validation，`-r vm`选择前端，与Web backend分开；有API的消费同时核真实后端，不用直接API写代替UI事件。

从固定交付依赖及新target验证公开Core/full、UI package/runtime/tool与外部生成工程，必要fresh/locked/A→B重定位及缺工具/错源资源/生成失败的已有保旧/恢复；只重放本期影响的U890发行/服务接线，不重做整个Web迁仓或所有应用验收。禁止旧主检出/WIP/全局旧二进制/同名双source隐式兜底。

### 5.5 暂存、证据与收尾（最新用户政策）

开工创建并登记D:/autostack/plan-artifacts/plan-014/的auto-ui身份、用途和在途状态；同号其他仓材料先核owner再分仓隔离，不覆盖。除计划本身外所有结构图/审计表/探针/日志/截图/逐ID输出、收据/spec-delta、外部消费者/SDK/发行包副本及外置target只放该scope；正常target留唯一worktree。不得使用attachments、.wt/evidence、其他盘符或散落输出；旧测试需显式指定输出，必要最小参数接线列T-02/07，不新造证据系统。

merge将必要命令/退出码/关键数字/节选折入本计划，逐项核对实际命令/日志中的全部输出路径及安装根，核owner/链接/活动引用/唯一依赖后删除整个scope与worktree构建物。只有具名未解决问题可持有（owner/理由/关闭条件/路径+hash），登记.retention/policy.json；未知不盲删。旧最近5个/14天/所有exe保留不适用。canonical只写docs/specs，legacy specs.json只读，分类/关系/历史由应用派生，不要求派生index/cache在线。纯规划不建材料scope。

### 规范增量

本轮不写canonical或legacy ledger；后续只维护canonical来源，不写specs.json。frontmatter只列UI本仓已存在/拟新增相对路径；下表跨仓路径带仓名仅是人工交接，不声称backend支持跨仓SD字段。

| delta_id | add/modify/retire | target | before/after rule | rationale / AC |
|---|---|---|---|---|
| SD-01 | add | 本仓`docs/specs/ui/structure-closure.md` | 分散实现说明→已交付M1/U1/U2/U3职责、状态/资源owner、公开兼容和禁止边 | 有限结构合同，不新增语义；AC-01..07/10 |
| SD-02 | add | 本仓`docs/specs/ui/structure-test-map.md` | 原入口分散→旧新ID/聚合义务映射与定向case/真实消费/限制，固定来源/证据 | AC-08/09/10 |
| SD-03 | modify | 本仓`docs/specs/README.md` | 注册结构/验证规范，现行runtime行为页保持其合同 | AC-10/12 |
| SD-04 | add | Man拟`docs/specs/cli/structure-closure.md` | CLI现有实现→解析/分派/diagnostics/工具与Web调用owner、Core/full/生命周期/回归入口 | AC-02/06/07/09/10 |
| SD-05 | modify | lang `docs/specs/auto-ui-{compiler,runtime,tools}/project.md`、`auto-man/project.md` | 更新实际结构与owner指针，旧行为/功能/版本承诺不变；只回写本次真实差额 | AC-01..07/10 |
| SD-06 | add | lang拟`docs/specs/repository-boundaries/ui-man-structure-closure.md` | 跨仓有限结构出口、来源版本/消费、890接受与007回交指针 | AC-07/09/10/12 |

## 6. 测试设计

以下全为未来验证，未执行。T-02把现有目标/命名case及拟新增fixture变为可复跑精确命令，正常/失败控制和实际oracle一起冻结；不能凭预计路径当已有测试。每行有work owner与AC，最终review另独立重建。

| case_id | AC / work owner | 输入/真实入口 | 可区分错误实现的oracle与边界 |
|---|---|---|---|
| V-01 | AC-01/10 / T-01/02 | 890/900/901/018正式组合、M1/U1/U2/U3函数/状态/资源/消费者表 | 每一族有实际入口→后继模块→唯一owner，必需未知0；已有子模块复用有证，不靠空壳/只挪测试达标 |
| V-02 | AC-02 / T-03 | 现有CLI service/passthrough与plan816_cli_driver/cmd_ui，Core/full、help/JSON、合法与缺参数/坏target/缺工具 | 参数/输出/诊断/exit和源owner符合旧合同；真实工具进程，非UI命令不误起UI，Core UI请求具名失败 |
| V-03 | AC-02/06 / T-03/08 | ToolUiOps/现有owned执行、run/watch/service启动失败、取消/超时/退出 | 只终止owned子树/reader，输出完整或准确错误，旧有效产物与用户文件保持；不以返回/置flag代替真退出 |
| V-04 | AC-03 / T-04 | Vue现有golden/库组件、plain/shadcn/library、导入/slot/store单/多依赖及非法准入 | 完整输出字节/资源/错误阶段对固定基面；已交付010唯一归属/歧义/限定调用保持，坏UI不误出半壳；不批改golden |
| V-05 | AC-03/07 / T-04 | 同一VueGenerator A→B→A与全新A，模块调用/registry/reset/缓存指纹 | 生成/告警/导入无前次残留，合法source变动失效重建；禁止缓存旧PASS或伪造旧fingerprint；不改变其既有支持范围 |
| V-06 | AC-04 / T-05 | build/build_with_debug、条件未选分支、for/key/slot、绑定/route参数与不同实例 | 求值次序/次数、惰性、子节点/事件/挂载/初始化/调试ID及上下文恢复与原合同相同，不能只比较最终树文本 |
| V-07 | AC-05 / T-06 | renderer固定消息序列、focus/input/窗口选择、active与订阅/任务/帧 | 在真正update/subscription/view消费边界观察顺序与状态/任务/呈现，实例不串线；不靠放宽active或追加tick修绿 |
| V-08 | AC-05/06 / T-06 | 图标/字体/纹理/快照或实际改动资源路径，创建/替换/断连/失败/关闭及最后owner释放 | 实际GPU/句柄/订阅资源或可验证已有诊断观察；合法external pin在其最后释放前保留，结束后无本Plan新增孤儿，不仅mock计数 |
| V-09 | AC-04..06/09 / T-08 | 固定legacy小型交互源码，真实Vue与`auto run -r vm`，输入/点击/状态读回/关闭重开 | 控件从活树发现、window/PID归本次，输入值structure14/计数2可见且读回一致，owned树/端口退出再重开；后端选项与前端分清；既有跨端差异具名不新增漂移 |
| V-10 | AC-05/09 / T-06/08 | 原定向视觉/GPU输入场景，首帧+交互后帧/焦点/布局/资源变化 | 状态/属性与屏幕证据交叉，实际呈现内容正确；空树/旧截图/headless/静态首帧不能充当全部通过 |
| V-11 | AC-07 / T-07 | 真实cargo metadata/tree、公开API/feature/Core/full及外部package/runtime/tool consumer | compiler不反依赖runtime，SDK无UI/Man/Web实现反向边，Man管理核心无VM/renderer，类型单source；既有pub/feature/栈策略保持 |
| V-12 | AC-08 / T-07 | 现有ID/子case台账→实际定向执行，重命名映射；删除required/0选择/坏映射控制 | 未解释丢失0、断言/条件不弱化、控制确实触发失败；历史红/skip/NOT_RUN单列，局部结果不认证完整批次 |
| V-13 | AC-07/09 / T-08 | 公开UI/Man/Web consumer新target/fresh/locked/A→B，缺资源/错来源/生成中断，恢复后再运行 | 只用固定新组合，无旧source/WIP二进制兜底；消费值/生成字节正确，保旧hash或预定恢复成功且再次真实可用；支持环境限制沿现行合同 |
| V-14 | AC-10 / T-09 | 四面实现/健康/遗漏与SD、890接受/007回交清单及明确版本 | 必需gap/新增warning/debug/重复算法0；工作AC逐项实证；不将新主题/示例已迁入等未来状态写成现状 |

实施/自查由后续执行会话负责；独立final review用同组冻结树的独立上下文，不修代码，finding回work。merge角色持最终门和合入/沉淀/归档guard；本会话当前只规划，不创建/指定agent或模型，不跨聊天交接。原生双端验证后续使用既有autoui-verifier及`auto-lang/.agents/skills/autoui-verifier/scripts/{test_vm_mcp.py,test_vue_playwright.mjs}`，不重写框架。

work/review只做命名反例/模块/fixture及必要`cargo check -p auto-ui-compiler -p auto-ui-runtime`、Man实际包check，不能因送审或单finding跑裸整档。UI变动tests/Rust真实触面决定门，最终review pass后merge在冻结提交统一UI `cargo t`+生成触面`tu`+工具/接口`ti`、Man `t/tcore/tc/ti`按实际触面及其规约；语言如有Rust实际修改才按其`cargo t <固定组>`受影响完整组及必要集成消费者并集，仅读规范不触发根整档/docs_gen。UI完整tu沿单进程规则，新增env全局态测试不塞该族。`tn`、画廊/AG、批量档仍按既有明确触发/手工/里程碑/单实例规则，不自动扩大为全AG/全应用/机器性能测量。必要原生/GPU验证窗口仅用于本Plan受影响面，不占他方任务或清共享缓存。

原始材料仅在§5.5统一scope，Plan/Spec记命令/退出码/选择与实际ID/关键结果/版本及必要节选；attachments含MD均不入git，merge折叠摘要后删除scope。证据复用须证明源码/依赖/配置/输入未变，新的renderer/Vue结构变动使相应旧证据失效。未有GPU/必需消费环境仍为gap，不能自动skip、降支持或以最后一次总测延期。

## 7. 验收标准

| ID | owner_stage | 完成条件 / 验证 |
|---|---|---|
| AC-01 | work | 890/900/901/018正式组合/真实X与四面完整职责/调用/状态/consumer清单齐；无未知必需项，来源与模块结构可复核；V-01 |
| AC-02 | work | CLI解析/分派/诊断/编排职责实际分离并复用旧cmd，Core/full/参数/输出/失败/owned运行保持；V-02/03 |
| AC-03 | work | Vue按主题职责实拆并保单state/import_policy/shared；所有触面golden/模式/010路由与复用隔离/失效保持；V-04/05 |
| AC-04 | work | Aura节点/绑定/构建/跟踪职责明确，借用/惰性/键/路由/挂载与调试身份不变，真实交互可用；V-06/09 |
| AC-05 | work | renderer消息/状态/订阅/active/view与资源/宿主职责按图接线，现行顺序/焦点/帧和GPU呈现有实证；V-07/08/10 |
| AC-06 | work | 唯一owner与线程/作用域保持，正常/错误/取消/断连/关闭后owned子树/reader/订阅/资源真实结束或准确恢复，合法pin寿命保持；V-03/08/09 |
| AC-07 | work | 公开API/类型/feature/profile/栈/依赖禁止边及独立打包源码闭包保持，无旧fallback/重复状态或循环；V-05/11/13 |
| AC-08 | work | 原子测试义务、断言、oracle、条件守恒，旧新ID映射与实际执行账相合，缺项/0选负控有效，新增回归0；V-12 |
| AC-09 | work | 真实CLI/工具、Vue/VM及GPU定向消费和外部fresh/locked/重定位/失败恢复后再用成立，支持/环境/历史红边界诚实；V-03/09/10/13 |
| AC-10 | work | 健康/遗漏/workaround扫描和SD proposal完整，890/900/901/018出口未回退，UI-007获得精确结构/版本/测试/资源回交，未提前实现或宣称主题/示例搬迁；V-01/14 |
| AC-11 | review | 同权威树独立final review绑定各代码/依赖/输入/SD，工作AC/必需反例全闭；明确merge-owned门pending，不用自查冒称独立终审 |
| AC-12 | merge | 冻结门通过、各实际修改仓真实合入与canonical来源读回（不写legacy ledger、不要求派生index在线）、主合同归档及所有Plan树fresh guard/清理收据完整，最终11/11才U014 |

## 8. 执行步骤

全部未执行，11任务=9 work+1 review+1 merge；稳定ID不随阶段改号。未来首个入口是G014后T-01；交付前置已核，但本次未授权实施探针。T-02冻结真正目标/精确命令后才能把拟建fixture作为执行证据。

| task_id | owner_stage / 依赖 | 文件/产物与关闭条件 | AC / 验证 |
|---|---|---|---|
| [✅ T-01] | work / G014 | 唯一ui-014组核各仓规约/HEAD及890/900/901/018实际交付和当前消费；冻结四面函数/消费者/状态/测试与X014，记录现有已完成职责和残项，禁止扩大到新架构 | AC-01；V-01，实际来源/依赖/窗口读回 |
| [✅ T-02] | work / T-01 | 在同树对M1/U1/U2/U3有界最小全链验证，冻结模块/owner/状态与消息/资源图、测试/ID/支持矩阵及每V精确入口；临时图/映射统一`D:/autostack/plan-artifacts/plan-014/auto-ui/structure-and-oracles.md`（新材料，开工登记），永久规则入源码/Spec。公开语义/资源范围不可保先needs_replan | AC-01；V-01与V-02/04/06/07/08必要机制控制，不提前改架构或虚构API |
| [✅ T-03] | work / T-02 | Man `auto/src/main.rs`及cmd_*/tool_ui_ops按冻结职责整理，复用现有owned/client/Web端点，主入口保组合/兼容，永久CLI反控 | AC-02/06；V-02/03 |
| T-04 | work / T-03 | UI compiler `ui_gen/vue.rs`→按template/script/style/import/library/store等具名主题模块（具体新路径T-02冻结）；移除旧正文/重复算法，保公开入口/单state/reset/010 | AC-03；V-04/05 |
| T-05 | work / T-04 | runtime `ui/aura_view_builder.rs`及必要既有相邻模块按节点/绑定/构建/跟踪职责提取；单VmBridge/Bindings/slot/path与guard，旧实现退出 | AC-04；V-06及定向状态控制 |
| T-06 | work / T-05 | runtime `ui/iced/renderer.rs`按已冻结消息/状态/资源owner整理更新/订阅/view、图标部件、纹理/快照与宿主边界；保实际调用、资源终态/帧/active | AC-05/06；V-07/08/10，真实GPU定向，不以只移动代码关闭 |
| T-07 | work / T-06 | 四面API/依赖/feature/类型和源码发行清单核对，测试随owner迁移/外移并保旧新ID与原子义务；复用现有治理及必要小型结构回归，旧重复逻辑/未接线helper退出 | AC-07/08；V-11/12及对应V-05 |
| T-08 | work / T-07 | 既有CLI/工具/公开package-runtime/生成发行consumer及固定legacy app，从新组合真实Vue/VM/GPU交互与owned退出重开；fresh/locked/A→B和失败恢复再消费 | AC-09；V-03/09/10/13，定位指纹/产物与实际工具来源 |
| T-09 | work / T-08 | 四面/全部work AC与V自检、健康/遗漏/SD-01..06 proposal、X回交和UI007接续矩阵冻结；独立审查入口齐后execution_done9/11 | AC-10；V-14及AC-01..09证据读回；不提前合入/沉淀或跑最终整档 |
| T-10 | review / T-09 | 同组冻结SHA独立final review所有work AC/V/SD与遗漏/延后/workaround；finding回同树work修复再审。pass后reviewed10/11，最终门仍pending | AC-11；auto-plan-review，独立来源重建 |
| T-11 | merge / T-10 | 重新冻结各提交/依赖，按实际修改仓统一最终门、必要固定组合认证；按依赖顺序真实合入，UI主规范/Man CLI规范/语言指针canonical沉淀（不写specs.json、索引由应用派生）；主Plan归`archived/`，逐树fresh guard clean后安全移除，统一scope及全部实际输出路径核对/删除、最终读回与007交接 | AC-12；auto-plan-merge，实际来源/版本/规范/档位/归档guard；不假设跨Git原子合并 |

完成后逐任务记`[✅ T-NN]`及提交/命令/退出码/实际结果/证据路径+hash；表格存在不等完成。work9/11即可转独立review，不把未来合入/清理当工作期必需；archive时如guard仍pending保持10/11，真实清理/读回后才11/11。不移除正式仓/他方树，任何递归清理前核绝对路径及guard，无force/链接穿透。

### 8.1 执行记录（2026-11-01，work轮1：T-01/T-02/T-03）

- **授权落点**：用户goal（2026-11-01）= G014第一合取项；第二合取项（890/900/901/018交付组合）r2已核归档，本轮以祖先关系复核（lang 2fcae41ba∈b1a4871a、UI 968ac83∈master、web ec9c4ba=HEAD、Man cbbe657=900收尾）。020完整交付未被列为前置（用户明示）；ui-020树活动与本组树互不触碰。
- **唯一组树**：`D:/autostack/.wt/ui-014/`：auto-ui@695ce0b（codex/plan-014-structure，基master 695ce0b——开工时master自6ab7f2d前移一docs提交，ff重绑）；auto-man@cbbe657→5b310f0（codex/plan-014-structure）；auto-lang@b1a4871a（detached固定依赖）；auto-web@ec9c4ba（detached固定依赖）；auto-down@7c4d5cb（detached固定依赖，§5.1预告的auto-down兄弟补齐）。无junction/symlink。scratch home `D:/autostack/plan-artifacts/plan-014/` 已建并登记。
- **T-01 关闭**（AC-01/V-01）：四面逐符号盘点（4个只读agent，SHA核对绑定）写入 `plan-014/auto-ui/structure-and-oracles.md`（sha256前16=0ed07e8bef88a01d）：M1 main.rs 3223行零`fn cmd_*`、real_main~2105行、10个既有模块已干净；U1 vue.rs单state 75字段/impl一整块/329内嵌测试；U2 builder借&VmBridge（owner=dynamic.rs）、tracked三元组散参；U3 renderer六段42k行、真owner=session.rs DesktopSession。890/900/901/018接受组合核验如上。
- **T-02 关闭**（AC-01）：同文件冻结M1模块设计（cli_spec/diag/9个cmd_*编排/瘦分派）、U1 vue/目录impl拆分（11文件家族映射）、U2 aura_view_builder/目录+TrackCtx收敛、U3 iced/renderer/目录六模块，及§5测试矩阵与每V精确命令；U1/U2/U3标"T-N开工再细绑"，共享写面短窗串行政策落§7。公开语义/资源范围全部可保（纯代码移动），未触发needs_replan。
- **T-03 关闭**（AC-02/06；V-02/V-03）：Man worktree提交**5b310f0**（基cbbe657，16文件+2691/−2293）：`crates/auto/src/main.rs` 3223→952行（纯分派+既有小臂），新增cli_spec.rs(802)/diag.rs(91)/cmd_{build,run,test,env,trans,req,wizard}.rs；`Commands`巨臂按冻结设计原样收编，Args结构字段简写构造（名字绑定防换序）；help/flag/默认值/机器JSON/stdio/退出码/Run臂exit_audit(main_err_ai_mode)/ui-extra门/Core形状全部保持；未造第二工具发现/owned设施。
- **901 consumer漂移修复**（T-03前置，同等cbbe657类）：master auto-man对lang b1a4871a本不可解析——死转发feature `with-file-history`（901已从auto-lang-core退出）残留于crates/auto与crates/auto-core manifests；零代码引用（grep实证）。删除2行+刷新Cargo.lock/core/Cargo.lock，一并入5b310f0。
- **V-02/V-03 实测**：`cargo check -p auto` exit 0（Full）；`cargo check --manifest-path core/Cargo.toml -p auto --no-default-features --features core` exit 0（Core形状）；unit `nextest -p auto --bin auto` 24/24；tcore 24/24（1既有skip）；`plan816_cli_driver+plan890_web_consumers+cmd_ui` 10/10（1既有skip；环境前置`te-child.exe`（lang）与`auto-ui-tool.exe`（ui组树）已补建）。负控：临时改名`cmd_build::run`→2个编译错→还原→0，分派接线为活。全部既有红/skip未新增。group兄弟构建产物（te-child/auto-ui-tool）留在各自worktree target，不入证据scope。

## 9. 复审记录

### r1历史记录（不作为当前启动指令）

修订前r1文件SHA256：7b825d7d009f0b9eccc5341eeea624435b8389e770ced9181e8231dac5859111。以下为2026-10-30纯规划与等待历史；原r1未实施无完成项。r2的§0–8/10为唯一当前合同，四面原义务和稳定ID保留。

2026-10-30 / stage: new / plan_id: auto-ui/PLAN-014 / plan_revision: 1 / outcome: blocked（仅指work前置U890及后续明确UI014实施授权未满足，合同静态预检pass） / scope: 0–10章节唯一有序、11任务/12AC/14V/6SD唯一连续且引用有效、12AC均有任务承接、任务依赖无环、890/014/007职责和先后无反向等待、54个当前本地链接有效、007及三热点4个源码hash稳定、UI014活动/归档唯一、UI号源015、围栏/空白与两仓限定diff检查通过；语言号源未由本会话写入，其他会话进度/WIP保留 / next: 用户审阅；未来U890完整交付并明确UI014实施后work/T-01同组权威树核X，不能提前建树或做probe。未实施、建仓/树/agent、运行项目测试/测量或GUI、改他方执行合同/进度/产品/canonical/ledger、提交或push；本轮文档校验不是实现验收或独立final review。

### r2当前规划交接（2026-11-01）

stage: new（revision） / plan_id: auto-ui/PLAN-014 / plan_revision: 2 / outcome: pass（合同自查，非实施或独立终审） / 授权：仅核查更新、暂不实施。已核890/900/901和018正式交付、canonical build/运行合同及四面源码/既有模块/状态与Man入口；原901/018等待已解除。11任务/12AC/14V/6SD保持；T01/02/07/08/09/11、AC01/07/10/12、V01/11/14接受新基线，新增§5.5统一暂存与merge即删。全部V仍未执行、无建树/代码/测试/GUI/提交/push。next：用户明确批准014 r2实施后work T-01/T-02，重绑届时正式版本与共享写面。

### work轮1（2026-11-01，T-01/T-02/T-03）

stage: work | plan_id: auto-ui/PLAN-014 | plan_revision: 2 | outcome: pass | worktree: D:/autostack/.wt/ui-014/auto-man（+组树 auto-ui@695ce0b, auto-lang@b1a4871a, auto-web@ec9c4ba, auto-down@7c4d5cb detached） | branch: codex/plan-014-structure | base_commit: cbbe657 | code_commit: 5b310f023a0ca1e60a356469e8d115d10e28f83e | dependency_revisions: lang b1a4871a / web ec9c4ba / down 7c4d5cb / UI组树 695ce0b | task_ids: T-01,T-02,T-03 | evidence: `plan-artifacts/plan-014/auto-ui/structure-and-oracles.md`（sha256前16 0ed07e8bef88a01d）+ §8.1执行记录 + 提交5b310f0（16文件+2691/−2293） | blockers: 无 | next_responsible_role: work（T-04 Vue，开工按§2冻结细绑并重读master）

同轮 goal 收据：stage: goal | plan_id: PLAN-014 | plan_revision: r2 | round: 1 | from: drafting | to: executing | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-014/auto-man | commit: 5b310f0 | next: work

### 901 consumer漂移登记（2026-11-01，已在本Plan内修复）

master auto-man（cbbe657，clean）对 auto-lang b1a4871a 解析失败：`with-file-history = ["auto-lang-core/with-file-history"]` 死转发feature残留（901退出该feature时未清扫Man侧consumer），根workspace与core workspace均不可编译，主检出target/debug/auto.exe为旧产物。本轮在T-03提交5b310f0内删除两行别名并刷新双lock；主检出auto-man本仓文件未触碰（该修复仅入014 worktree，随本Plan合入回master）。

## 10. 待澄清事项

| ID | 当前事项 / owner与关闭门 |
|---|---|
| D-01 | 890/900/901/018已有最终交付记录，T-01核当时实际已合入SHA、端口/工具/投影及接受差额；不重造guard、复活885历史阻塞或把018归档前文旧waiting当现状。**2026-11-01已核**：祖先关系全过（§8.1），接收组合成立；901遗留Man侧死feature漂移已发现并随T-03修复 |
| D-02 | 四面精确函数/状态/资源图、模块名与每V命令T-01/02有界冻结；当前有实质结构残项，合理已有职责逐符号复用，不用行数/只挪测试判完成。**2026-11-01已冻结**：structure-and-oracles.md（M1可实施深度，U1/U2/U3设计层+各T-N开工细绑门） |
| D-03 | 018稳定编译/兼容别名与optional语义为保持基线；如需新语义/公共ABI/feature改变，给具体输入/AC/替代方案再修订，不恢复旧测试门/renderer stub |
| D-04 | 原生/GPU/真实支持矩阵T-02核，T-08完成本包必需oracle；018 widget事件控制不冒完整MCP输入，Linux cosmic未执行仍未执行；不用headless/skip替代必需真输入/后端/GPU/owned关闭重开 |
| D-05 | 899/其他AG/OS/应用及新计划的共享符号/工具/窗口按实际正式diff核，短窗单写；不加无关整线前置、不借WIP/停进程；周期语言回归另按due规则，不变UI014全局重复门 |
| D-06 | U014后向007交新模块/入口/状态owner、资源fixture/测试映射、精确版本及900已迁内容与整棵examples/ui剩余差额；007仍独立修订/授权，014不主题化demo/提前宣称搬迁完成 |

