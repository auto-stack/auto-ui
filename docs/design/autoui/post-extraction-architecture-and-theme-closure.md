# 拆分交付后的AutoUI架构与主题收口设计

> 当前接续：target-state r4，2026-11-04。031/033/036已归档；037最终合入/清理收据完成11/11但头部7/11仍需owner簿记对账，canonical B1+B2累计22例；038已归档研究，B3十二仍待迁。新立UI041状态实例、042 Reload研究、045–047三教学迁移批和语言930生成交付/独立源守门，全部仅规划，详[当前接续设计](instance-state-and-reload-stage-one.md)。
>
> 下文§0–10是首次重审/036立约时的源码观察与目标历史，不覆盖当前canonical或各最终收据；旧“渠道未接/计划在途”不作为现状。真实产品仍按OS Design03各独立合同，不以教学结果抵产品。

> 主责：auto-ui。承接语言[Design47](../../../../auto-lang/docs/design/47-autoui-state-design-and-feature-architecture.md)、[Design38](../../../../auto-lang/docs/design/38-autoui-features-and-system-services.md)、[Presentation Host草案](../../../../auto-lang/docs/design/autoui/app-presentation-host.md)和[Editing草案](../../../../auto-lang/docs/design/autoui/editing-feature-syntax.md)。旧RFC是输入，本页为910/007之后的排程与主题收口建议，不把旧语法整体批准为实现。

> 2026-11-03历史立约快照（现状以上述2026-11-04为准）：[调研与批次设计](theme-rollout-stage-two.md)已立UI037（B2十二例，0/11）、UI038（B3支持面核验，0/8）、OS055（真实clock与系统事件/载体验收，0/11）；均drafting r1，仅立项。037/055采用036新渠道的最终门接U036，038研究不冒迁移完成；不接管OS054八产品装配。

## 0. 031/033之后的发行与输入收口设计（2026-11-03）

正式包原语已交付，现`official-theme-package` Spec仍明确Full SDK/source/delivery渠道未接、VM/Rust `auto build`未接同一receipt。实际Man安装只拷blueprints/widgets；UI front inputs只收pac/front/sidecar，不显式收外部主题；Vue receipt错误当前仅警告。这些实际入口差额由[036](../../plans/archived/036-official-theme-distribution-and-input-closure.md)接续，不重开031，不以033双端绿推定新渠道已绿。

目标流为UI唯一主题源→校验/规范化身份→受管快照→Man Full SDK与UI delivery→实际依赖解析/准入→Vue、VM、Rust生成/运行/复用。元数据/文件集合/内容影响身份，冻结实际消费输入防并发误签；无变更保持稳定，坏包/I/O失败不冒成功；安装和再发行沿staging完整校验后换装、失败保旧。语言仅承载必要中立输入，禁止重新拉UI实现；Core安装仍零UI资产。无公网registry或新通用包管理器。

验收以001/006真实外部消费者和010兼容控制，覆盖Full/Core反控、delivery fresh/locked/第二根、同mtime合法改包/篡改、旧receipt迁移、发布失败与恢复，不仅验证helper。精确字段/解析支持/命令由获实施授权后的036 T-01冻结；若现支持轨缺真实能力须重规划，不删验收轨逃避。

OS053继续持宿主/transport/真实OS事件/calc与pin；其现固定0.2.0快照保持合法，036不增其原启动门。918持收据执行ID/依赖审计，036交真实manifest与source影响而不改其基线。B2十二例可先盘点/立约，采用新渠道的最终验收等U036；控件/业务/013阻点由B2实际范围核定，不等OS053整项。B3、其他产品、State/Feature/Reload继续独立。

## 1. 结论与本轮边界

源码分仓、语言/UI/Man结构、独立构建和Cargo有限治理已形成可继续开发的基面。下一阶段不继续搬仓、不重开归档计划，而是：**先把主题从单应用试点收成有明确实例/作用域与发布合同的框架能力，再按真实组件依赖迁示例；AutoOS宿主与产品仓分别接收，不等待所有教学例改完。**

状态实例、Feature、Service、命令作用域和热重载需要继续演进；主题所需的最小App/组件身份先明确，并不要求先重写整个状态系统或造OS通用服务。每项实施仍须独立正式合同，保持较大完整模块粒度。本页不给任何源码修改/发布/子模块更新授权。

## 2. 当前证据与交付接受

### 2.1 可重复读取的快照

| 仓 | 本轮只读HEAD |
|---|---|
| auto-lang | `835816fa3986290826de225a4321acdbb88caf8b` |
| auto-ui | `ab3910deead551d476a292158098ffe87c8a8711` |
| auto-web | `3de5524e28bba6eeb8ff2f7f7666676b58c874fa` |
| auto-man | `47d8f7b34e584dccc90fefe340629bbe318c6405` |
| auto-os | `65fc70de8b505657db660d91024b2cf4425b7759` |

910交付lang f665ce113/UI01f99fd/Web3de5524/Man67add3c和007交付UI7f6e3db1/lang835816fa3/Man47d8f7b均经`git merge-base --is-ancestor`核为当前各仓祖先。文档状态、收据和源码三者分列，本轮未重跑历史门。当前UI有其他会话未跟踪`crates/auto-ui-runtime/test/`及既有设计目录WIP，语言设计索引有WIP，均不消费/提交/清理。

关键源码SHA256（后续开工须重核，不把HEAD代替工作目录内容）：

| 路径（相对D:/autostack） | SHA256 |
|---|---|
| auto-ui/crates/auto-ui-compiler/src/style/theme/mod.rs | a6394ca222429e19d9cd65ceb7f0a1803f0b395885f467b7e8cd019683b32e0f |
| auto-ui/crates/auto-ui-runtime/src/ui/aura_view_builder/mod.rs | d127ce33797bd4ee0c4aa89d3e9387ce9044c41e6389879ad33cfab11bf85615 |
| auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs | 1b3512ad3a21f135041ca425e761414d268e6d8f51a9349f000c1dbaf069696c |
| auto-ui/crates/auto-ui-runtime/src/ui/dynamic.rs | d6c3e7263097edbd68b86b37f4941081faa4494d7b61e1e8e3283a240d69c746 |
| auto-ui/crates/auto-ui-compiler/src/ui_gen/vue/store.rs | d727284344d210311c59f240e1145b570122bd4dca2ce5018720ed94f8312bf6 |
| auto-lang/crates/auto-generation-core/src/generated_build.rs | 1782b9a812866bcae884d6f14e8b1fff2a69fbf6fc2310b55b2534de026fe890 |
| auto-os/.gitmodules | 789e6307f470a07e0da14762277de28ead8b332f0aae3531efb29366db9d9119 |

### 2.2 已交付、未关闭和文档陈旧文字

| 来源 | 接受的实际结果 | 不应扩大解释 |
|---|---|---|
| [910归档](../../../../auto-lang/docs/plans/archive/910-generated-cargo-build-governance.md) archived9/9 | 生成上下文/成员bin身份/内容收据/owned lease/清理准入；UI/Web同义工作集正空间收益及移根控制；canonical[生成生命周期](../../../../auto-lang/docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md)已落地 | 不覆盖Tauri、全机可写target或所有历史缓存。D-910-1 execution_ids、D-910-2 CLI -d派发、Web发行腿既有问题和具名缓存持有仍须按owner跟踪，不默认全量门绿色 |
| [007归档](../../plans/archived/007-autoui-tree-theme-and-examples.md) archived10/10 | 381来源文件归位及读者回接、主题数据/普通use/extend/双端投影、Theme Lab与002/005首批；三个[canonical主题/示例页](../../specs/ui/theme-tree-pilot.md)在主线 | 其余示例未完成新主题；多应用隔离/子树scope/全部控件/宿主同步未交付。收据仍保AC-05实际OS主题事件腿pending，本轮不把受控事件当真实OS事件通过 |
| 906/014/018/901 | 保独立源/单owner、结构职责、编译配置与固定测试选择 | 不恢复语言→UI/Web实现边，不为主题再拍平feature或合并可写worktree target |

007旧文首的execution_done、§10旧待授权/旧906未交付文字是历史未刷新；终态以frontmatter与最终merge收据为准。相应原子义务/限制不会因archived消失。若下一合同核出既有实现违约，按复现结果fix-forward，不逆转归档、不将改进候选偷偷降为已有通过。

本次`git ls-files examples/ui`：语言仓仅定位README（1文件、无应用目录）；UI仓383受管文件、38目录（原37目录＋官方themes）；35编号目录身份保留，两031不合并。此为来源状态，不是运行通过分母。旧[清单](theme-migration-inventory.md)仍写“未搬迁/002未迁”等，是2026-10-29历史快照，不能拿它覆盖007交付或当下一批实时输入。

## 3. 架构重审：已有机制、必要收口与独立能力轴

| 轴 | 当前实物 | 重审结论与实施边界 |
|---|---|---|
| 状态与实例 | runtime `ui/vm_bridge.rs`有独立VM、堆对象/子状态；`new_from_decls`合成根路径，`ensure_child_state`仍把props写到根对象 | 不能称所有状态都是全局；也不能凭存在对象就称嵌入隔离已解决。保旧显式共享，按App/组件/Feature实例明确owner、读写receiver及销毁；主题先借现有稳定组件身份建立自己的scope，不先全面替换VM |
| Vue共享store | compiler `ui_gen/vue/store.rs::generate_store_composable_full`仍生成模块级ref singleton | 模块定义与运行实例分开；旧共享声明保兼容，新Feature组合须显式实例/共享选择。另立状态/Feature包，不把它列成简单教学例换肤的全局门 |
| 主题与设计 | compiler `style/theme/mod.rs`线程局部ACTIVE_THEME/THEME_TREES/APP_DESIGN_ENTRIES/THEME_EPOCH；builder `eval_app_design`取这些槽；Vue写document根CSS vars | thread-local不是App身份。先补App/session主题owner、局部scope和两端投影；compiler轻包不成为runtime状态owner。legacy入口可保适配，但不能成为新多App路径的隐式状态源 |
| 命令/Feature/Host | `action_config.rs`仍有进程静态配置；Design38、Presentation Host和Editing均Draft | 保“领域会话/文档/视图/命令目标分开”，不复制整App充Feature。目标绑定、焦点路由、退出和服务能力交接先形成单独合同；不在本主题批次新增整套语法 |
| Reload | `DynamicComponent::reload`旧字段定义传空、写回错误忽略；`check_file_changed`在imports/registry非空时返回None | 多文件保守跳过仍在；需独立schema/行为版本/effect owner/失败恢复设计。主题数据更新可先支持有界重新发布或明确重启，不声称已完成通用HMR |
| 产物/依赖 | 910的project/member/context/receipt区分，007移根后产生新成员 | 新主题包/资源成为实际生成输入，换根按910重新生成，不能让主题迁移绕过fresh/locked、锁/来源校验或清理保护 |

更大的Feature/Service/应用架构方向继续接受Design47/38；本次只将它们从旧v0.5路径重新定位到真实独立仓。全面状态事务/undo/persistence、通用远端query、服务supervisor、插件/热替换不自动纳入第一个后继实施包。

## 4. 主题剩余差额与新合同优先级

### 4.1 已观察的风险，须先实证而非预填通过

| ID | 现状与证据 | 必需关闭方式 |
|---|---|---|
| G-01 | ACTIVE_THEME等为线程局部共享，主题名注册表同名覆盖；Vue document根变量默认同域 | 同进程两个App/同App两个窗与两个嵌入实例使用相反族/模式/同名不同版本数据，任意渲染顺序均不串；退出一个不影响另一个 |
| G-02 | `set_active`只比较`name()`；同名但token/资源/schema变更未在此处触发替换/epoch | 以来源包+内容revision和作用域身份判断；真实更改同名token使两端更新，未变不无效重建。静态风险尚未运行复现 |
| G-03 | builder `eval_app_design`先`set_dark_mode`再查主题/模式，表达式解析失败走continue | 原子候选校验/发布：未知主题＋合法新模式、合法主题＋缺模式、半个selector求值失败，都保旧完整pair；首次失败不发布半套数据。源码顺序与“事务保旧”有差额风险，先具名复现再修，不能只测主题名单独错误 |
| G-04 | design仅radius/background、仅当前元素；未含子树继承和样式/组件状态全表面 | 支持父→子继承、局部覆盖、兄弟/同型实例隔离和词法token引用；Button/Input/Panel及实际第二批所需状态/焦点/禁用/文字边框等完整闭合，其他控件列后继，不能称全控件已接入 |
| G-05 | Vue保留theme_name/dark_mode魔法状态名；新主题Auto/light与legacy scaffold/dark/indigo默认并存 | 选择来自显式design绑定/宿主策略，旧保留名仅兼容；确认CLI/pac/宿主/独立默认的优先级和合法冲突诊断，不能把所有老App静默切为Auto/light |
| G-06 | mode_available无modes时允许任意模式；extend字符串名与族modes继承有既定试点规则 | 后继明确单模式支持/默认mode/system缺测语义，兼容试点或给显式迁移；不要从bool布尔mode拼出未声明能力，亦不把裸名/新Core语义当顺手修复 |
| G-07 | 官方themes包在examples/ui/themes；现有scope只证本仓三个消费者；纯数据dep仍可能告警 | 需要受管、版本化、可外部获得的官方UI数据包来源；示例与产品共享同一源，避免产品依赖教学目录/全仓绝对path；打包/安装/生成输入来源有据 |
| G-08 | 007实际OS主题事件腿pending，runtime整档及旧golden有具名既有红；013-todo Rust轨仍未闭 | 实际事件与受控事件分列、平台边界明示；未做不PASS。013等阻塞相应迁移时由该面owner精确修复/列独立前置，不把全部债设成全局门 |

G-02/G-03是本次代码级差额候选，不是新跑测试的失败结论。下一合同首先建立合法控制与能区分错误发布的反例；若违反现行已承诺行为，明确归为既有缺陷fix-forward，不拿“试点限制”免除要求。新scope/新表面则明确标新能力。

### 4.2 目标对象与发布语义（概念，非已实现API）

1. **主题目录ThemeCatalog**：UI负责schema/token/recipe定义与不可变已验证主题版本，按来源与revision标识；主题名称只是查找标签。不得以相同名字认定数据相同。
2. **主题选择ThemeSelection**：App session持有family、mode策略（light/dark/system）、resolved_mode和来源策略；进程不持唯一可变选择。一个App多窗默认共享session选择，显式preview可建子scope，两个App天然隔离。
3. **局部设计DesignScope**：组件实例拥有局部差异，父scope引用只读；直接字面值解除该消费字段的token关联，子覆盖不改父/兄弟。scope不拥有业务store/文档，也不复制字体/纹理大对象。
4. **已解析快照ResolvedDesign**：先从原始主题＋模式＋局部差异验证/解析出候选，整pair和revision一次发布；失败保旧，失败诊断带来源/槽路径。epoch/cache键包含scope身份＋revision＋resolved_mode，不能只用名字或全进程epoch。

VM/Iced：builder/evaluator/style adapter显式拿当前session/scope快照，临时借用不改变VmBridge/DesktopSession的资源owner；逐帧调用不写另一个App的全局选择。Vue：根容器提供scope和CSS vars，嵌入子树局部覆盖；document根只作为单App legacy适配，不强迫新多App页面共享变量。两端消费同一解析/错误规则，backend不是第二套主题表。

主题更新先验证不写业务、不发用户事件、不做IO；两个selector同一次模型更新只发布一套合法组合。UI线程/渲染生命周期中安装快照与失效顺序须在正式合同绑定真实调用，不宣称全机所有App跨进程原子换肤。

## 5. UI、主题包、示例、AutoOS各自负责什么

| owner | 持有与交付 | 禁止越界 |
|---|---|---|
| auto-ui compiler/dialect/runtime | 主题数据协议、scope/发布/解析、控件投影和诊断、官方主题包单一源、兼容适配 | compiler不反向拉runtime；不拥有OS偏好持久化/跨进程服务、不修改语言通用Core语义来简化UI写法 |
| auto-lang generation-core / auto-web / auto-man | 沿910保持受管输入/SDK来源/锁/缓存/owned构建，必要主题资源进入实际receipt和delivery allowlist | 不造另一主题解析器或另一全局target，不复制Web/生成算法 |
| auto-ui examples | 应用自己的业务与design声明、公共主题依赖、逐app双端行为验证；themes/stylekit保持各自库用途 | 不各复制全套调色表；不把例子更新当产品仓更新，不重搬原381来源文件 |
| auto-os | 桌面theme family/mode偏好、有效策略、系统事件接入、版本化宿主通知/重连与配置持久化、当前会话/窗口路由 | 不重新定义公共token表；新桌面设置不按同名字段直接覆写业务模型，不把主题服务与通用Feature/Service架构捆绑 |
| 产品独立仓 | 选入公共包版本/声明跟随或覆盖、业务专用视觉域、实际双端/宿主/独立启动与升级回滚 | auto-os不在子模块主检出直接改源码；各仓Plan/worktree/合入后再更新gitlink，不能复制教学业务覆盖产品 |

官方主题包归UI，长期发行位置建议从教学examples提升为正式UI包资产（具体包名/落点按现有Auto package渠道在首包合同冻结，不在本设计杜撰已存在registry）。旧`examples/ui/themes`可在有版本迁移路径时保薄引用/明确兼容，不保两份可编辑数据。必须证明外部source/安装/离线已锁重放；若现有渠道不足，给有限来源能力前置，不能仅靠本机兄弟目录通过。

## 6. AutoOS承接：沿已有链补能力，不另造服务

现有事实：AutoOS `shell/desktop.at`已有set_theme dark|light命令；[桌面规范](../../../../auto-os/docs/specs/shell/showdesk-ux-polish.md)已有bus与config外写两条生效链、全场快照invalidate和dark_mode回写。auto-calc pac明确去掉theme跟随链，auto-weather亦有宿主/CLI旧跟随；不是完全没有主题机制。当前未逐产品运行审计。

建议宿主policy至少区分跟随和显式App覆盖，固定优先级：显式启动/应用选入的受控override → 对跟随App的宿主family/mode选择 → App声明的独立默认 → legacy既有回退。应用是否跟随由声明/宿主协商确定，不能让内部theme_name字段每帧反压宿主或自动持久化。CLI旧theme仅dark/light，family新增选择另给明确迁移设计，不混同同名字段。

通知以会话/应用身份和单调revision为序；App先验包版本/模式支持，再本地原子发布。拒绝更新保旧并回报原因，重连/重启获取当前完整快照而非只等增量事件；存储policy与resolved_mode分开，system重新解析真实OS偏好，不能把上次解析dark永久写成显式选择。不承诺所有App同一瞬间切换；需要每个已接受App最终应用最新revision、乱序/重复不倒退。

接收路径至少覆盖：同进程DesktopSession多App、独立App进程（现有desktop protocol/bus选合适版本化扩展）、Vue独立预览/嵌入、系统事件→桌面config→运行App、退出/重连恢复。只实现一个set_theme按钮或改pac不算跨进程接入。真实系统设置修改应在获准的短窗口验证，不能由本设计会话操作系统。

产品来源以现行`.gitmodules`+gitlink+registry联合冻结，不按示例编号推断业务相同。本轮OS HEAD中示例抽样gitlink：calc e5247c549db375480f30b9eb1e417ad9b15191a3、weather 1626a4a16ccf4de717bc92e27b60a9184b8198f2、explorer 59a99b9f42896728aa32a7ff3e38fa08261c7d18、reader fa6185ebd1e35229ae496ef1e07afa2b157c7d74；这些是OS固定来源，不能拿本机子模块HEAD冒当前发布组合。

## 7. 示例差额、批次与应用独立性

现存35个编号目录中，002/005已首批接入；其余33个编号目录待逐项处置，其中032是验证例，因此为**32个其余应用＋1个编号验证例**，不能将33全称产品，也不能把032从验收义务删除。themes/stylekit/remote-frame-e2e另按包/库/验证夹具处理；新增383文件/38目录不能沿旧381分母凑守恒。

| 后继批次（建议，未取号） | 明确候选目录 | 启动与退出 |
|---|---|---|
| B1 基础控件/表单/静态展示 | 001、003、004、006、007、008、009、010 | 主题基础收口交付后；代表数据绑定、focus/disabled/error/hover及旧默认兼容在真实Vue/VM验证；不牺牲业务结构来配两槽试点 |
| B2 列表/状态/导航业务 | 011、012、013、014、015、016、017、021、022、023、028、038 | 按实际组件/异步/数据依赖分完整业务包；013 Rust生成等所选轨阻点先闭，原业务/请求/导航/计时/状态保持，不能只看静态截图 |
| B3 媒体/编辑/专用视图 | 018、019、020、024、025、026、027、029、030、031-image-viewer、031-paint、041 | 公共壳/控件先迁；图表色、图像/视频/画布/编辑语法等专用视觉值逐项说明映射或合法固定值；不因专用内容排除整个App |
| V 验证输入/共享资产 | 032、remote-frame-e2e、stylekit和themes | 保原oracle/时序/压力/平台；共享库只在真实consumer有需求时扩，不能给fixture强装主题菜单或把库计成已迁App |

此分组覆盖全部32剩余应用和编号验证例，不是四份即刻执行合同。每个批次立约前以实际目录/依赖/支持轨再冻结，允许按完整模块重分包但不丢项。全部应用新主题接入是已确认方向，小样评估不能将其降为可选推广。旧语义类/业务固定色并存须逐字段判断；自动搜索/替换和颜色命中数都不是验收。

AutoOS首个真实消费不必等B2/B3或全主题控件完成：主题基础/版本包和宿主合同交付后，即可选基础产品试点（优先已旧跟随链的calc/weather作候选，实际只选一个功能与轨道可闭合的产品）；其余各产品按真实来源/能力分批。Reader029/AG030等现行独立路线不被本页接管或追加全局门。

## 8. 建议落约顺序与启动门

| 工作包 | 主责/依赖 | 本包必须交付、不能借后继补绿 |
|---|---|---|
| A 主题作用域、发布与正式包收口＋第二批代表验证 | auto-ui主责；已有910/007/014/018实际接受；无需等待整个State/Feature/HMR | G-01..07所需基础：两端App/session隔离、同名revision、pair失败保旧、局部继承、B1所需控件面、明确默认与正式包来源；Theme Lab/002/005兼容加B1至少一例真实验证。G-08实际OS事件如不在A支持面不得写PASS，并给O包义务 |
| B1 其余基础示例完整批次 | auto-ui；A正式交付和实际组件面具备 | 表中B1的逐app接入/双端业务/旧theme兼容；A已选代表可以有来源证据接受，B1不能漏其余例 |
| O 宿主主题策略/通知/持久化与一个真实产品纵向接入 | auto-os持宿主合同，产品仓各实际改动；A正式版本/接口，产品先选入实际支持面 | 宿主↔UI版本绑定、配置/OS事件/乱序重连恢复、同进程及独立进程、单产品业务与独立启动；不等待B1全部/B2/B3，也不以教学例通过替代产品 |
| B2/B3和产品后继 | UI与各产品按依赖分别立约 | 每批完整应用/支持轨实际验收，组件缺口有先行owner；不假定B1包已支持全部媒体编辑面 |
| S 状态实例/Feature/命令目标与Host | UI主责、具体领域产品消费，Design47/38再细化 | 两个同型实例隔离/显式共享、领域会话与UI出口/目标生命周期；不要求所有示例先重写，不借全局单例伪实现 |
| R Reload/schema/effect生命周期 | UI主责；所选实例/资源合同稳定 | 明确保留/迁移/重建/拒绝，局部失败旧版本保用及effect退出；不将普通Vite更新或主题重发当AutoUI多文件HMR完成 |

**工作包A已立为[031](../../plans/archived/031-theme-scope-publication-and-package-closure.md)**，将作用域、原子发布、基础控件/官方包和003/010第二批真实消费者作为一个纵向模块。正式包版本/支持表/兼容与边界按031合同，实际接线/资源预算由获实施授权后的T-01有界冻结；本轮不执行。

O可以现在深化方案，但不在A实现前凭“稍后适配”发布产品；S/R可独立深化而不抢A的共享写面。所有实施遵循各仓正式Plan/worktree/独立review/merge，本页不自动分配号源、执行探针或开启其他agent。共享renderer/Vue/evaluator/source-owner/锁写面短窗单写；测试按受影响固定职责和消费者选，不成为ACE/AG全套或全机回归总前置。

## 9. 下一合同必须携带的可证伪验收

| ID | 输入/真实路径 | 可观察关闭结果 |
|---|---|---|
| D-V01 | 一进程两个App、两窗/两嵌入同型实例；同名不同主题内容，交换渲染顺序 | token/epoch与选择各归owner，关闭一实例不影响余者；Vue两个容器/VM DesktopSession都证实，非只Rust手造树 |
| D-V02 | 同名主题改token/导入包revision、模式往返，未变控制 | 新值真实可见，未变不错误失效，缓存/资源身份一致，不用名字判断内容等价 |
| D-V03 | 新theme+mode同事件、未知theme+合法mode、缺mode、selector求值失败 | 原完整有效pair与像素/状态保留；首次失败准确报错；后续合法更新恢复；两端都测 |
| D-V04 | 父scope/子覆盖/孙继承/兄弟/两实例；引用槽与同值字面量 | 继承链正确、局部不串、字面不跟槽变，布局/交互状态跨换肤保持；声明支持的控件状态逐项可见 |
| D-V05 | legacy CLI/pac/default、显式design绑定与非保留字段名、host跟随/覆盖 | 优先级可读回、旧App外观不悄改、新App不依赖魔法字段；未知模式/包失败有据，真实OS事件另列 |
| D-V06 | 官方包外部fresh/locked/安装/第二绝对根/缺源篡改恢复、主题源变化 | 单源/版本/revision/资源进入实际生成receipt，旧根不可见仍运行；910成员/target规则保持，无复制全调色板或主检出兜底 |
| D-V07 | 宿主偏好变更、App拒绝、乱序/重复通知、断线重启、system真实事件 | 每个接受App最终最新revision，拒绝保旧且宿主可见，配置恢复不把resolved写成policy；业务数据不变 |
| D-V08 | 每个迁移App的真实业务反例/焦点/错误态/输入/导航及主题模式往返 | 原功能/条件/oracle保留；按双方已声明支持轨验收，不以代表例或截图替代全清单；不支持轨明确留差额，不能改支持承诺凑绿 |

这些是未来合同的验证输入，全部未执行；确切新fixture/命令与矩阵由相应Plan绑定当前入口，不新造通用runner。现有`theme_tree_pilot`、runtime及Vue真入口、桌面bus/config、产品业务测试和910独立消费门可复用，不凭其旧绿认证新作用域。实际系统事件、GPU/原生输入窗口和预算开工单独确认，未经允许本页不操作桌面。

## 10. 未决与风险归属

- A立约前必须决：App/session/窗口scope策略与实例目标、pair原子发布、正式包来源/旧引用迁移、单模式语义/legacy默认和B1真实支持表。新的语法/公共ABI若超UI域，给精确语言侧合同，不在主题中顺手改变Core。
- 字体/纹理等大资源由原owner留有界共享，scope仅引用及revision；性能采样在真实交互/渲染链做，有必要再立专项，不为主题加全局P90/三warm或全量AG门。
- 007实际OS事件pending、runtime已有红/快照漂移、013-todo生成失败、910缓存持有与D-910-1/2继续具名；是否阻塞按所选验收链决定，不因归档自动关闭、不全部提升为新全局前置。
- 本轮没有重审到每个产品/平台。AutoOS .gitmodules只是来源证据，Vue/Iced/Web与VM原生支持不能互相推定；Linux未验证不写跨平台PASS。
- 本页是目标设计，不改canonical现状；后续实施独立review后才沉淀新Spec并更新消费者。旧Design47/38/主题约定作为历史输入保留，以本页日期边界区分“已有/候选/待验证”。

本轮静态核查方式：读取canonical/归档收据与上述实际代码；git祖先验证、受管示例清单及关键文件hash；不重新跑910空间实验/007交互。设计结论的来源与局限随页保留，不借旧临时scope（均已清理）作为唯一证据。

静态预检（2026-11-02）：10个主章节、G-01..08/D-V01..08唯一，10个本地Markdown引用存在；B1/B2/B3逐目录对账恰覆盖32个其余应用，002/005接受首批、032单列验证义务，无重复/漏项；设计与索引限定diff/尾空白检查通过。此为文档检查，不是新能力测试或独立实现复审。
