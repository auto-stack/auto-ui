---
plan_id: PLAN-033
status: archived
feature_name: 剩余六个基础示例的正式主题迁移
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-03
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md]
new_spec_components: [docs/specs/ui/basic-examples-theme-adoption.md]
touched_goals: []
affects: [examples-ui-001, examples-ui-004, examples-ui-006, examples-ui-007, examples-ui-008, examples-ui-009, official-theme-consumers]
depends_on: [auto-ui/PLAN-031]
---

# [auto-ui/PLAN-033] 剩余六个基础示例：正式主题迁移

## 0. 变更摘要

承接[031合同](031-theme-scope-publication-and-package-closure.md)和[架构重审设计](../design/autoui/post-extraction-architecture-and-theme-closure.md)的B1剩余包。仅迁001-helloworld、004-profile-card、006-hero-section、007-stats-board、008-pricing-table、009-article-feed六个真实示例，消费031正式主题包/作用域/原子选择/控件能力，保持现业务与布局语义，交付逐app双端验收和后继清单。

2026-11-03 用户以 /goal 完成计划033 授予执行授权；U031 已核（docs/plans/archived/031-*，status:archived 11/11，master 7940d895）。当前executing r1、2/9：T-01/T-02 完成（组树 ui-033 + 同组 lang/web/man/down 固定源；六例 pac dep+use+design 绑定接入 0.2.0 包；001 实窗 2×2 族/模式往返包值证据）。031 交付面缺陷一处已修：窗口投影路未装组件 scope（见 §9 回合1）。

## 1. 目标

- G1：六例使用同一正式主题包来源和版本，无复制整套配色；独立默认、显式override及嵌入follow按031最终接口，跨族/模式的有效值可见。
- G2：静态内容、原事件/模型/打印反馈、远程资源语义、业务专用色和原布局保持；主题选择不新增支付/导航/服务端或业务状态。
- G3：每个示例独立Vue与VM/Iced真实入口验证，换肤不丢值/实例/焦点；031的002/003/005/010不是本包再迁对象，legacy与宿主端口必要控制保持。
- G4：基础示例B1完成清单明确；AutoOS053和后续业务/媒体批次接受来源与限制，不以教学例通过冒真实产品交付。

非目标：重搬示例、重新实现031 scope/原子发布/正式包/基础控件、全B2/B3/验证夹具迁移、AutoOS配置/通知/产品、全State/Feature/HMR、Core语义/新feature/远端发布。

## 2. 架构方案

只在六例源/dep/pac及必要消费者说明接入031已交付机制，配色数据归正式包；App扩展色在各自命名空间，不膨胀31公共键。优先保现有style recipe/语义类，按字段建立公共角色、局部字面、业务内容/装饰豁免表，不做全仓颜色替换。

框架能力不足或必需轨出现阻点时，T-01给具体输入/031承诺和差额：原031必需功能回其owner具名fix-forward，新支持超本合同先needs_replan；不得把框架新能力偷偷塞进六例、关闭VM轨、静态贴色或删交互凑绿。Theme Lab或一个代表例不能替另外五例验收。

## 3. 技术栈

现Auto源码、031正式themes包与design绑定、Vue生成/VM-Iced运行、既有autoui-verifier/MCP/Playwright及031测试映射。生成/依赖/外部消费沿906/910来源、锁与receipt身份；不新造runner或全机共享target。

## 4. 需求分析与背景调查

### 4.1 授权与来源

用户要求分别规划六例与AutoOS纵向承接，按031合同核范围/交接/启动门，暂不实施。本包主责auto-ui；OS唯一对应合同为[auto-os/053](../../../auto-os/docs/plans/053-host-theme-chain-and-calculator-adoption.md)，两包互不依赖完成，只共同等031正式交付。

已读本仓[Specs索引](../specs/README.md)、[应用迁移](../specs/ui/theme-app-migration.md)、[试点](../specs/ui/theme-tree-pilot.md)、[示例归属](../specs/ui/examples-ownership.md)、[测试选择](../specs/build/ui-test-selection.md)、031及六例真实app.at/pac。031当前executing2/11（瞬时观察，以最新收据为准），公开包/host端口仍待最终交付；其draft历史“暂不实施”文字不能覆盖执行会话进度，也不能作为本包获执行授权。

UI只读HEAD `27f1edcd3396d3d9a31aadb96431fed1a2d97153`；本仓dynamic_build/ui_tests/stylekit测试等他方WIP保留，不消费/提交/还原。开工重核实际版本、各例文件hash、dep/资源/测试/平台条件与子目录指令。主检出只立Plan/号源/定位索引，未改示例。

### 4.2 六例实际责任与原行为

| 目录 | 本次app.at SHA256前16 / 原事实 | 本包不可改变的业务与迁移点 |
|---|---|---|
| 001-helloworld | 077d535366cc63e6；15行、无model/handler | 原问候文本/结构，独立和嵌入可换有效主题，不强加业务模型或每例设置菜单 |
| 004-profile-card | 8a24e5625fbd929e；姓名/role/bio/avatar_url/status；无消息handler | 人名/内容/徽章、远程头像成功与失败占位、圆形裁剪/边；Follow/Message等若无handler不得新增。顶部渐变/在线状态色按装饰/语义说明处理 |
| 006-hero-section | 582fb7eb81ec053b；dark_mode/accent_color、GetStarted/SetTheme/SetAccent | GetStarted原打印、原宿主主题消息接线；不新增注册/跳转。新族选择不被旧每帧值覆盖，主题控制不复制成独立菜单 |
| 007-stats-board | 6973892ef89494bd；469行、多统计/图文展示字段、无消息handler | 数字/标签/图形与布局保持，分类/趋势/状态色逐项映射或合法固定值，不捏造筛选/刷新/网络数据交互 |
| 008-pricing-table | 24e23e7500b65a13；BuyPlan1/2/3打印Selected Plan各方案 | 方案名/价格/权益/推荐/按钮反馈与等高StretchLine布局保持；不新增付款或订阅状态 |
| 009-article-feed | a0ba37bdb357f42c；ReadArticle1/2/3打印各reading反馈 | 文章/作者/日期/标签与三按钮原反馈保持；不新增路由或后端抓取，图像/背景资源按原来源处理 |

上述hash为本次工作目录只读观察，不等于未来冻结；源码注释/旧README可能滞后，T-01以真实当前声明和行为对账。原不具交互的例只验原展示/可选择文本/已有按钮能力，不增业务来造测试。

## 5. 详细设计

### 5.1 启动门和唯一组

实施启动=用户明确授权本合同 ∧ 031正式review/合入/canonical/收尾及实际端口交付可接受 ∧ T-01重核所需控件/包/支持轨和原红。031代码写完或execution_done不等于U031；合同路径packages/themes、0.2.0、任意接口名均须按其实际交付接受，变化先记录等价迁移/必要合同修订，不消费WIP。

预定`D:/autostack/.wt/ui-033/auto-ui`，分支`codex/plan-033-basic-themes`；仅实际生成依赖需补同组lang/web/man/down固定来源，一仓一树、无链接。实现/自查/独立review同一组顺序，Plan唯一在主检出。T-01不开通新框架能力；门前仅设计/计划读回，不建树做probe。

### 5.2 六例接入与scope

以031正式依赖/来源清单接themes包，旧语义类可以沿兼容消费；应用只声明需要的App选择/局部diff，不复制31色表。独立默认与follow/override由031规则落实；明确删除/迁移旧pac显式dark等对新follow的阻断，列差异；未选入例及旧App默认保持原合同。

六例都须覆盖Auto/Solarized×light/dark的真实有效值及往返；以外部测试/宿主适配或现控件触发，不给每例新建菜单。同页/同进程至少选两个相反主题同时展示作隔离集成控制，六例逐一独立运行不能省略。局部diff不重置model/输入或改变事件目标。

逐例颜色表区分公共底/文字/边/焦点、success/状态/分类/图文色、内容资产与装饰；采用正式包角色或App命名空间语义定义，固定视觉内容有明确理由和两模式可读证据。不能把007图表含义色全涂primary，也不能以装饰豁免整张卡片。可读/对比检验阈值与原支持标准在T-01给出具体字段/观察，不临时自创全域认证。

### 5.3 资源、生成与恢复

004等远程资源保持真实URL语义，验证可控网络成功与失败路径；可在scope内用受控fixture做确定性资源控制，不用本地图片永久替换App源或把网络缺图充完整PASS。布局/圆角/焦点变化与配色分开判，保fit/min窗口、等高卡片和文本内容。

主题包版本/hash/导入进入031/910实际生成准入；同名包内容变、移根、缺源拒绝及恢复用受管外部副本验证，未变不无故改生成物。正常构建在唯一组，外部复制/target在统一材料scope，不运行生产缓存清理。支持原不变增量回退，坏主题候选保旧有效选择，不修改用户真实系统主题。

### 5.4 边界与交接

| 对方 | 本包接受/交付 | 不自动接管 |
|---|---|---|
| 031 | 接最终SHA/包/schema/控件状态/host adapter/旧控制与已知限制，六例逐项使用 | 不再修scope/发布设计、不补031必需验收；缺口具名交owner |
| [OS053](../../../auto-os/docs/plans/053-host-theme-chain-and-calculator-adoption.md) | 共享同版正式包/优先级/follow约定，提供已验示例和独立消费步骤 | 不以六例替代calc产品，互不以对方完成为前置；共享入口写面短窗单写 |
| 其余示例 | 本包若完整交付：001/004/006/007/008/009变已迁；基础B1结束；余B2十二/B3十二共24应用，032验证例及库保原义务 | 无业务/媒体全量支持承诺；013 Rust轨等依所选验收由owner闭门 |
| AutoOS产品 | 包来源/版本/模式及实际兼容约定，不冒产品部署 | 产品独立仓/OS gitlink/持久化/系统事件另按OS合同，不更新它们 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/basic-examples-theme-adoption.md（新） | 六例旧语义/局部硬值→正式包/选择/颜色映射/双端原业务与限制 | 完整逐例可验收，不只计颜色命中 | AC-01..05 |
| SD-02 | modify | docs/specs/ui/theme-app-migration.md | 031后B1余六→实际验收六，余24应用/验证库明列 | 不削全量目标、不冒AutoOS已接入 | AC-05/06 |
| SD-03 | modify | docs/specs/ui/basic-examples-theme-adoption.md（SD-01新页） | 无集成说明→包/host接口来源SHA、资源/独立读取/回退和OS053交接 | 当前只proposal，按最终实物定稿 | AC-03/04/06..08 |

不写现行Spec/legacy JSON于本轮，复审后只沉淀真实验收差额。

## 6. 测试设计

全部未来验证，T-01使准确命令和原业务oracle可运行，沿autoui-verifier真实Vue/VM双端，不新增通用runner。

| Case ID | AC / 输入与入口 | 区分错误实现的关闭oracle / owner_stage |
|---|---|---|
| V-01 | AC-01；六例真实dep→use/design→正式包读取，版本与源hash | 六例非空实际选择、主题值来源同一包，无各例复制色表/旧教学根fallback。work/T-01/02 |
| V-02 | AC-02/03；每例两族两模式及往返，局部diff/legacy默认 | 原内容/模型/实例/焦点保持；底/前景/边/状态色实际可见，两端有效值一致，不以生成成功替运行。work/T-02..05 |
| V-03 | AC-02；六例§4.2原业务逐项、六种不同输入/事件或展示控制 | 006原GetStarted、008三方案打印、009三阅读打印真实handler反馈保持；001/004/007无新业务；截图＋状态/输出读回，不伪造导航/付款。work/T-03/04 |
| V-04 | AC-03；004头像资源成功/失败、007分类色、008等高布局、六例实际焦点/文字 | 原资源语义/占位与裁剪、分类含义、关键布局和可读性保留；网络未运行不记全绿。work/T-03..05 |
| V-05 | AC-04；两个不同例相反主题同进程/同页、follow/override交替选择及关闭重开 | 不串色/选择/epoch、不丢model；旧不选入控制保持；真实adapter输入，不直接篡最终CSS绕过。work/T-05 |
| V-06 | AC-04；外部副本锁定正式包、第二根、同名内容变保mtime、错源/缺源/恢复 | fresh/locked实际消费、910receipt按内容失效，拒绝明确/恢复后下一次运行正确，不借旧exe。work/T-05 |
| V-07 | AC-05/06；031控制002/003/005/010及六例最终清单/后继界面 | 原已交付选择/业务必要面无回归，B1八例来源对账，本包实际只迁六；余24＋验证义务和OS053交接明确。work/T-06 |
| V-08 | AC-07/08；健康/遗漏/SD/所有AC与源码到运行证据，落地主线/清理 | 独立review绑定提交核实；merge冻结门/canonical/归档/guard与材料全路径对账。review/T-07、merge/T-08/09 |

T-01冻结选例run命令/端口/driver/来源，正常`auto run`和`auto run -r vm`必须来自同组已构建工具。明确选中/执行/原排除数，首次红/平台未测不冒PASS。UI生成触面按031 test-map选完整族和必要工具/资源集成；资产/Auto源码仅做实际职责门，不能因读规范跑语言整档。精确新增断言属于对应V，不镜像实现写无意义测试。

最终review前只相关反例/实际完整职责族；review pass后merge冻结UI `cargo t`与实际生成触面`cargo tu`单进程、必要`cargo ti`/真实六例，不叠跑全AG/ACE或全机批量。既有红若阻塞本包必需面具名修复/needs_replan，不能声称“背景红”替代通过。原生输入/GPU/资源窗口由work按实际授权安排，Windows实证不推定Linux全绿；作者自查不是独立final。

## 7. 验收标准

| ID | 可观察验收 | owner_stage |
|---|---|---|
| AC-01 | 六例接同一031正式包/版本/来源，无色表复制与反向依赖，V-01 | work |
| AC-02 | 六例原业务/内容/handler/布局与状态保持，六例双端实际换族/模式往返正确，V-02/03 | work |
| AC-03 | 颜色映射/合法专用豁免、远程资源/失败占位/文字焦点/关键布局有据，不以静态换底冒完整迁移，V-02/04 | work |
| AC-04 | scope隔离/优先级/独立follow控制、包输入失效/外部locked与第二根/恢复保持031和910规则，V-05/06 | work |
| AC-05 | 031原四例必要控制无回归，六例实际证据/平台限制与业务差额明列，无未授权漏项，V-07 | work |
| AC-06 | B1完整关闭与余24/032/共享库状态、OS053来源/接口交接齐，不冒产品交付，V-07 | work |
| AC-07 | 独立final review逐AC/原例/健康/遗漏/workaround/SD pass，绑定实现/依赖提交，V-08 | review |
| AC-08 | 冻结受影响门、实际合入/canonical/归档/guard/统一scope和全输出目录清理真实完成，V-08 | merge |

## 8. 执行步骤

| ID | 动作/实际触面与关闭条件 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | U031与实施授权后建唯一组；冻结六例/包/接口/颜色资源原业务/命令/预算，判能力阻点，不提前扩框架 | 启动门；全部work AC，V-01原控 | work | [x]（2026-11-03，commit 6ead145a 前）：U031核验（archived 031 11/11 @master 7940d895）；组树 `D:/autostack/.wt/ui-033/`=auto-ui@7940d895 分支 codex/plan-033-basic-themes + 同组固定源 auto-lang@1c1512b4d/auto-web@3de5524/auto-down@7c4d5cb/auto-man@47d8f7b（detached）；`cargo build`（组ui）+`cargo build -p auto`（组man）exit 0（2m57s），auto.exe 钉入组ui target/debug。冻结：六例源 hash=§4.2 六值逐位一致；包 themes 0.2.0 schema1（theme-package.json 清单）；端口 5421-5460（4931/5139-5238 等为 Windows 保留段 EACCES）；VM 驱动=autoui_fixture（AUTOUI_TEST_FIXTURES=1，PLAN-623 状态写+handler 派发）+宿主适配 Rust 桩（theme_host_inputs 型）；Vue 驱动=AUTO_UI_THEME env 显式覆盖 + store 模块真通道（无 VM fixture 等价物=具名边界，不以 CSS 篡改冒充）；主检出 dynamic_build/ui_tests/stylekit 他方 WIP 保留未消费。 |
| T-02 | 六例pac/dep与design最小公共接线，保选择/默认/宿主旧控制，建立逐字段映射表 | T-01；AC-01/02/03，V-01/02 | work | [x]（2026-11-03，commit 12446768）：六例 pac `dep themes{../../../packages/themes}` + app `use themes.themes: Auto, Solarized` + theme_name（001/004/007 补 dark_mode，默认 Auto/light=原外观；006/008/009 保留 pac theme:"dark"+dark_mode=true 显式默认）+ design{theme:.theme_name,mode:if.dark_mode}（C-14 形，无新菜单/业务态）。映射表：004 在线点 bg-green-400→bg-success（包 success 槽，闭 Plan-637 悬案），顶部渐变维持装饰豁免；007 四 chip+三 avatar 底=分类身份色固定字面（映射 success/error/info 将伪造语义、无 purple 角色；逐项理由入源注释，两模式字面同串=可读性证据）；006/009 渐变豁免维持；001 根补 bg-background/text-foreground（公共角色，family 可见面）。**发现并修复 031 交付面缺陷**：窗口投影路（projection::dynamic_view[_vwin]）未装组件 ThemeScope——design 绑定族不达像素（eval/发布/resolve 全链正确[三重证据：probe 测试、在build内 resolve=包值、scope diag]，166 次投影转换走 legacy 白值 vs 4 次 guarded eval 包值）。修=投影入口装 guard（commit 6ead145a）；实窗证据：001 Auto/light 白→Solarized/light 奶油 #fdf6e3→Solarized/dark 青绿 #002b36→Auto/light 白 往返（plan-artifacts/plan-033 shots m1-m4）。 |
| T-03 | 完成001/004/006三例真实接入与双端原行为/资源/布局验证 | T-02；AC-02/03，V-02..04 | work | [x]（2026-11-03，commit 78dbfe6a）：六测全绿（tests/test_00X_vm.py + test_00X_vue.mjs，仓内逐例测试约定=只跟踪脚本不跟踪截图）。VM 腿：fixture 通道族/模式往返+状态读回；004 头像成功（naturalWidth=80 真实远程加载）/失败路径（fixture 改写 URL，源恢复不替换）+无 handler 按钮 safe no-op；006 GetStarted handler 派发 oracle+print 行入 app 输出日志+SetTheme/SetAccent 宿主传播（{widget,event,input} 派发达 dark_mode/accent_color）；内容跨腿保持。Vue 腿：setupState 模型驱动（VM fixture 的模型层等价通道，真实响应式写、零 CSS 篡改）逐组合有效容器变量=包 HSL 值（Auto 0 0% 100%/240 10% 4%；Solarized 44 87% 94%/192 100% 11%）；001 可选文本；006 控制台 print 反馈+pac dark 启动渐变；系统通道（prefers-color-scheme）启动播种。**发现并修复 031 交付面缺陷#2（同 flush 批次竞态）**：watch(theme_name) 读 DOM dark 类在族+模同帧变更时为渲染前旧值，覆盖 dark-watch 的正确应用（006 Solarized/light 腿实证）；修=发射的 watch 声明 dark_mode 时按模型取 isDark（theme_name-only 维持 DOM 类读，commit 78dbfe6a）。三例双端复验全绿后提交。 |
| T-04 | 完成007/008/009三例真实接入与双端分类色/handler反馈/布局验证 | T-03；AC-02/03，V-02..04 | work | [x]（2026-11-03，六测全绿）：007 新建双端（无 handler/无新筛选刷新；分类 chip 固定字面 Vue computed rgb(254,226,226)=red-100 断言+VM 字面在快照；themed 面=包值）；008 扩展（BuyPlan1/2/3 print 双端：VM 输出日志+Vue console；等高卡 Vue 实测 462/462/462 spread=0px=V-04 布局 oracle；items-stretch/flex-1 VM 断言）；009 扩展（ReadArticle1/2/3 print 双端；emoji 缩略占位保持）。三例 fixture/setupState 往返包值、pac dark 默认遵守、内容跨腿保持、无新业务面。 |
| T-05 | 六例两族两模式并集、嵌入隔离/优先级、包内容失效/外部消费/第二根/恢复与旧控制 | T-04；AC-01..04，V-01..06 | work | [x]（2026-11-03，commit 09804377 + e2e/leg 记录）：**V-05 VM**：plan033_b1_scope_isolation 2/2——同进程 001/006 真实源相反主题互不串（family/mode/Background 各自包值 002b36 vs 09090b 同帧）、宿主 override 优先+释放回 design 默认（follow/override 交替）、非法 override/非法 design 值均整帧拒绝保旧、关闭重开独立 scope；**V-05 Vue 同页**：单宿主页双 iframe（001/009）相反主题（Solarized dark 192 100% 11% vs light 44 87% 94%）无串色、交替翻转各自跟随、单帧重开独立默认他帧不受影响（plan-artifacts t05_vue_samepage，9/9）；**V-06**：t05_v06_package_lifecycle e2e——fresh receipt=031 交付身份（9fa07742…）、篡改（保 mtime 不更清单）具名拒绝、合法变更（更清单保 mtime）digest 9fa07742→de39e73d 且生成成功、恢复（字节还原）receipt 回原身份+包树 git 干净；六例 `.theme-inputs.json` 全在且同 digest（V-01 收口）；第二根外部消费=031 已交付 theme_external_package_build（本轮回跑 1/1 绿）；受影响族并轮回跑全绿：runtime 6 族（example_theme_migration2/external1/host_inputs6/lab1/isolation4/lifecycle5+plan033 2）、compiler 3 族（scope_contract1/manifest2/verify6）。**既有红具名（非本包引入）**：`vue::gallery_registry_at_tests::test_gallery_demo_row_routable` 硬编码 `../../../auto-lang/examples/ui` 语料根——主检出解析到含 018 的旧语料通过、worktree 布局解析到不存在根必红（本包 diff 未触该路径）；tools lib 并行另有全局态污染类翻红（两次运行 12→9 不稳定，solo 全绿，同 031 归因口径）。 |
| T-06 | 全work自检/健康/遗漏、原四例控制、SD与B1后继/OS053交接冻结，execution_done6/9 | T-05；AC-05/06，V-07 | work | [x]（2026-11-03，spec commit c5562a52）：原四例控制无回归——002 实跑（+×2 → count=2、Solarized 按钮、切明暗 → dark=true，截图 002_sol_dark_count2），003/010=example_theme_migration 2/2，theme-lab=theme_lab_build 1/1（本回合受影响族并回时全绿）；SD-01/02/03 准备于工作树（basic-examples-theme-adoption.md 新页 + theme-app-migration.md 第三批/B1 收口/余 24 修订，c5562a52）；B1 收口=10 例已迁（001..010；review F-033-1 修正）、余 24（B2/B3 各十二）+032/stylekit 义务、OS053 交接（同包身份/follow 约定/六例即参考实现）冻结入 spec 页；自检=工作树 git 干净（仅未跟踪截图）、六例双端测试+隔离测试+V-06 e2e 全部运行于最终提交态代码。 |
| T-07 | 同组冻结树独立final review，finding回work/失效对应证据 | T-06；AC-07，V-08 | review | [x]（2026-11-03）：final review pass（见 §9 review 记录）；F-033-1（B1 计数 8→10）已闭@d3606cc0（rebase 后=2b5b9c91）；独立重跑六 VM+9 族二进制+Vue 006/001 全绿 |
| T-08 | 冻结受影响职责门、真实合入/Spec/交接落地，不实施OS053或后继 | T-07 pass；AC-08 | merge | [x]（2026-11-03）：冻结门=cargo t（--no-fail-fast 全量）2962/2969 + 7 具名既有红（plan643/plan714 于干净 master 环境 (eabc2038+lang@fb3acc33f) 逐位复现；gallery 语料根 worktree 布局类——全部非本包引入，归因证据在案）+ cargo tu 868/868 + 真实六例双端（T-03..05 证据 + 评审独立重跑）；两次 rebase 均以 range-diff 全 `=` 证明补丁等价（c5562a52→a6a4320f→bc98fa0c 线）；ff-only 落地 master=d734f1fe；canonical=docs/specs/ui/basic-examples-theme-adoption.md@8930e898… + theme-app-migration.md@cef55dd6… 落主验证 |
| T-09 | 摘要折Plan/全输出路径对账删统一scope，各树wt-guard clean后移除树/分支/空组，归archived9/9 | T-08；AC-08 | merge | [x]（2026-11-03）：guard clean→worktree remove（transient 截图随树消亡）→branch delete（was d734f1fe）→同组 lang/web/down/man pinned 树移除→空组目录删除；plan-artifacts/plan-033 整 scope 删除（证据已折本 Plan §6/§8/§9）+ registry 刷新；落地≠部署观察项：主检出 target/debug（auto-ui-tool.exe 等）与 auto-man auto.exe 为 PLAN-033 之前构建，未重建/未重启（无生产消费面受本包影响——六例为示例资产；OS053 按其自身合同接线时自行构建），现记录为显式陈旧观察 |

材料仅`D:/autostack/plan-artifacts/plan-033/auto-ui/`（跨仓按repo身份），登记owner/用途/在途；不写attachments/第二套输出根。正常构建在组树，外置target/副本/SDK/日志截图在统一scope。merge必要摘要/命令/退出码/数/源及保留件hash折本Plan后删整scope/树内产物；仅具名未闭持有owner/理由/关闭条件，不盲删未知/他方目录或进程。

## 9. 复审记录

2026-11-02 / stage:new / auto-ui/PLAN-033 r1 / outcome:blocked（U031正式交付及本包实施授权未满足） / 仅六例源码/现行为/资源与031合同读回，静态预检pass：0–10章节、9T/8AC/8V/3SD唯一连续、本地引用有效、无尾空格、033跨活动/归档唯一且号源单调；定义完整迁移/独立双端/OS053边界；未实施/建树/运行probe或测试。next：接031最终来源/包/接口与明确实施授权后work/T-01。原业务差异不得被迁移基线覆盖。

2026-11-03 / stage:work(回合1) / auto-ui/PLAN-033 r1 / outcome:executing(T-01/T-02 pass) / worktree:D:/autostack/.wt/ui-033/auto-ui（branch codex/plan-033-basic-themes，base 7940d895）/ code:12446768（+fix 6ead145a）/ deps:auto-lang@1c1512b4d,auto-web@3de5524,auto-down@7c4d5cb,auto-man@47d8f7b（同组 detached）/ evidence:001 实窗 2×2 往返包值（shots m1-m4）、组构建 exit 0、probe 测试过、六例接线 diff（12446768）/ 031缺陷修复:projection 投影装 scope（6ead145a，属 C-05 面收口，非新能力）/ 待:T-03..T-06（双端逐例、V-05/06、SD、Vue 真腿、归档面）。next:work/T-03。

2026-11-03 / stage:work(回合2) / auto-ui/PLAN-033 r1 / outcome:executing(T-03 pass) / worktree:D:/autostack/.wt/ui-033/auto-ui / code:78dbfe6a / deps:同回合1 / evidence:六测全绿（001/004/006 × VM/Vue）、004 头像真远程加载+失败路径、006 宿主传播通道、watch 同 flush 竞态修复（031 交付面缺陷#2，模型读 isDark）/ 031缺陷修复累计:投影 scope(6ead145a)+watch 竞态(78dbfe6a)，均属已交付面收口 / 待:T-04（007/008/009）、T-05、T-06。next:work/T-04。

2026-11-03 / stage:work(回合3) / auto-ui/PLAN-033 r1 / outcome:executing(T-04 pass) / worktree:D:/autostack/.wt/ui-033/auto-ui / code:T-04 commit（tests 007/008/009 双端）/ deps:同回合1 / evidence:六测全绿（007/008/009 × VM/Vue）、008 等高实测 0px、三例 print oracle 双端、分类固定字面双端断言 / 待:T-05（隔离/包失效/外部消费）、T-06（自检/SD/B1 冻结）。next:work/T-05。

2026-11-03 / stage:work(回合4) / auto-ui/PLAN-033 r1 / outcome:executing(T-05 pass) / worktree:D:/autostack/.wt/ui-033/auto-ui / code:09804377 / deps:同回合1 / evidence:V-05 双端（VM Rust 2/2 + Vue 同页 9/9）、V-06 e2e 四态（tamper 拒/legal 迁/restore 归/fresh 恒）、六例 receipt 同 digest、受影响族并回全绿、既有红具名归因 / 待:T-06（自检/健康/SD/B1 冻结→execution_done）。next:work/T-06。

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: c5562a52 | next: review`

2026-11-03 / stage:work(回合5) / auto-ui/PLAN-033 r1 / outcome:pass(execution_done) / worktree:D:/autostack/.wt/ui-033/auto-ui（branch codex/plan-033-basic-themes，base 7940d895）/ code:c5562a52（6 commits: 6ead145a 投影scope修复, 12446768 六例接线, 78dbfe6a 001/004/006 双端+watch竞态修复, 50624696 007/008/009 双端, 09804377 隔离/优先级, c5562a52 spec delta）/ deps:auto-lang@1c1512b4d,auto-web@3de5524,auto-down@7c4d5cb,auto-man@47d8f7b（同组 detached）/ task_ids:T-01..T-06 / readiness:AC-01 六例 dep/use/receipt 同包同摘要（9fa07742…）；AC-02 六例双端往返+业务 oracle（12 测全绿）+002/003/010 回归；AC-03 映射/豁免有据（004 success 闭悬案、007 分类字面双端断言）、004 头像成功/失败、008 等高实测 0px；AC-04 V-05 双端（Rust 2/2+同页 9/9）+V-06 e2e 四态；AC-05 原四例控制无回归+六例平台证据（Windows 实证，未推定 Linux）；AC-06 B1 收口 10/余 24/OS053 交接冻结入 spec；AC-07/08 归 review/merge / 既有红具名:gallery 语料根 worktree 布局红（硬编码 ../../../auto-lang/examples/ui，非本包引入）+tools 并行全局态污染类（solo 全绿，031 同归因）/ open findings:无 / pending gates:统一回归冻结门（cargo t + cargo tu 单进程+必要 ti/真实六例）归 final review/merge；landing/canonical/归档/guard 归 merge / next_responsible_role:review（T-07 同组冻结树独立 final review）。next:review。

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 6 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: d3606cc0 | next: merge`

2026-11-03 / stage:review(T-07) / review_scope:final / reviewer_identity:实现会话内独立重构评审（计划未指定独立 final reviewer 角色；结论由工件与独立重执行重建，非执行者摘要） / plan_id:auto-ui/PLAN-033 r1 / outcome:pass（F-033-1 已闭）/ worktree:D:/autostack/.wt/ui-033/auto-ui branch codex/plan-033-basic-themes / reviewed_commit:d3606cc0（评审发现修正后基线；含 c5562a52 及前 5 commits，base 7940d895）/ dependency_revisions:lang@1c1512b4d,web@3de5524,down@7c4d5cb,man@47d8f7b（组内 pinned，重验未动；主检出 lang 已他移不影响组）/ spec_inputs:docs/specs/ui/basic-examples-theme-adoption.md（新，d3606cc0）+ theme-app-migration.md（修订）+ theme-scope-and-publication/official-theme-package（031 既有，本包只消费）/ acceptance_results:AC-01 pass（六例 dep/use diff 核 + 六 receipt 独立读同摘要 9fa07742dc68）；AC-02 pass（六 VM 测独立重跑全绿；001/006 Vue 独立重跑全绿；002 实跑 count=2+切族切模、003/010=example_theme_migration 2/2）；AC-03 pass（diff 核：001 仅根面补公共角色、004 仅在线点→success+渐变豁免注释、007 chip 字面 0 变更、008/009 button/print/onclick 0 变更；004 头像/007 字面/008 等高 oracle 在重跑中）；AC-04 pass（plan033_b1_scope_isolation 2/2 + 同页 9/9 + V-06 e2e 四态，独立重跑隔离测试绿）；AC-05 pass（受影响 9 族独立重跑全绿：runtime 7 二进制 + compiler 3 二进制）；AC-06 pass（B1 收口 10/余 24/OS053 冻结——F-033-1 修正后）；AC-07 pass（本评审：逐 AC 证据映射+绑定提交）；AC-08 pending（merge 闭环）/ pending_closeout_gates:统一回归冻结门（UI cargo t + 生成触面 cargo tu 单进程 + 必要 cargo ti/真实六例）=merge 冻结；合入/canonical 发布/归档/guard/材料清理=merge / findings:F-033-1（docs，closed@d3606cc0）——SD 准备稿 B1 收口计数误写 8（实为 10=002/005+003/010+本批六）；余 24 计数原本正确；既有红归因（非本包）：tools lib 并行全局态污染类（12→9 漂移、solo 全绿）+ gallery 语料根 worktree 布局红（硬编码 ../../../auto-lang/examples/ui，主检出旧语料在场） / evidence:独立重跑记录（六 VM + 9 族二进制 + Vue 006/001）本轮评审输出；diff 抽核（001..009 源面）；receipt 六读 / next:merge（T-08，冻结门+真实合入）。

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 7 | from: merge | to: done | status: archived | outcome: pass | delivery: d734f1fe | archive: docs/plans/archived/033-… | cleanup: 组树/分支/artifacts 全清 | next: done`

2026-11-03 / stage:merge / plan_id:auto-ui/PLAN-033 r1 / outcome:pass / delivery_commit:d734f1fe（master，ff-only 线性；两次 rebase range-diff 全等价，映射 6ead145a→070b14a4→97928f13 等）/ canonical:docs/specs/ui/basic-examples-theme-adoption.md@8930e898…（新）+ theme-app-migration.md@cef55dd6…（修订）落主验证 / archive:docs/plans/archived/033-basic-examples-theme-migration.md（status:archived，9/9，commit dd91b27a）/ cleanup:组树五成员+分支+空组全删（guard clean 两轮），plan-artifacts/plan-033 删除+registry 刷新 / gates:cargo t 2962/2969+7 具名既有红（master 环境归因实证）、cargo tu 868、真实六例双端 / next:done（completion_kind: delivered）。

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: pass(T-05) | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: 09804377 | next: work(T-06)`

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: pass(T-04) | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: T-04-tests | next: work(T-05..T-06)`

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: pass(T-03) | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: 78dbfe6a | next: work(T-04..T-06)`

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 1 | from: drafting(blocked-on-gate) | to: work | status: executing | outcome: pass(T-01/T-02) | worktree: D:/autostack/.wt/ui-033/auto-ui | commit: 12446768+6ead145a | next: work(T-03..T-06)`

`stage: goal | plan_id: PLAN-033 | plan_revision: r1 | round: 8 | from: done | to: done | status: archived | outcome: pass | delivery: d734f1fe | verification: 主检出审计——d734f1fe∈master(merge-base is-ancestor)、specs 干净=落地内容、归档件 committed dd91b27a、codex/plan-033-basic-themes 分支已删、.wt 无 ui-033、plan-artifacts/plan-033 已删、T-09 陈旧观察在案、主检出余留 WIP=§4.1 他方保留件 | next: done(goal complete)`

## 10. 待澄清事项

1. 031当前在途，最终包/控件/接口及原控制证据必须正式接收，不能凭execution_done启动。本轮只立约，未授权本包执行。
2. 004等原远程资源验证需可控网络与失败控制，T-01冻结输入与原占位；不可永久替换源规避。原输入/GPU窗口与空间预算由执行前明确。
3. 所选控件/轨若超031承诺先分清已有bug与新能力，具名owner修复/重规划，不把六例变成第二框架重构包。
4. UI033与OS053均等031而互不等待；本包完成后余24应用＋032等验证/共享库仍有义务，不重定义成已全量。
