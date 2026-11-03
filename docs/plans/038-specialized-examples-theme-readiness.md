---
plan_id: PLAN-038
status: reviewed
feature_name: B3十二个专用示例的主题迁移支持面核验
author: [Codex]
created_at: 2026-11-03
updated_at: 2026-11-04
plan_revision: 1
current_step: 5
total_steps: 8
supersedes_spec_components: [docs/specs/ui/theme-app-migration.md]
new_spec_components: [docs/specs/ui/specialized-theme-migration-readiness.md]
touched_goals: []
affects: [examples-ui-b3-inventory, media-editor-theme-support, theme-successor-contracts]
depends_on: [auto-ui/PLAN-031, auto-ui/PLAN-033]
---

# [auto-ui/PLAN-038] B3十二个专用示例：主题迁移支持面核验

## 0. 变更摘要

本包是**有界研究/验证合同**：核018/019/020/024/025/026/027/029/030/031-image-viewer/031-paint/041十二目录的主题支持与原业务/专用表面，执行六类代表性真实链，产出逐例go/blocked/no-go、阻点owner和可独立实施的后继分包。不是全量B3主题迁移实施，不把研究归档计入已迁数量。

drafting r1、0/8。2026-11-03用户授权调研立项；本轮只静态读取/文档与取号，暂不实施，无GUI/探针/构建/业务代码/系统设置改动，不创建或联系其他agent/chat。

## 1. 目标

- G1：十二例的来源/实际依赖/入口与轨道、原业务/性能/时序/平台义务、颜色角色及专用内容域完整对账。
- G2：用真实App/控件/后端做能区分错误实现的六类代表探针，确认scope/包输入能走到实际表面；失败、未支持和未执行分列，不以静态截图替代。
- G3：据实测确定B3的有限实施批次、必须先交的控件/后端/生成前置与owner；不存在“先全迁后再看能否运行”的无界包。
- G4：研究结论可复现、独立复审与有限规范沉淀，不改变原功能/样本/oracle、不抢产品/Reader/AG路线。

非目标：迁移十二例源码、修媒体引擎/读书编辑器/通用VM或HTTP、增加新平台、全部媒体性能评估、真实用户文件扫描/修改、全部OS产品接入、另造测试平台/registry/缓存。

## 2. 架构方案

以现031/033能力、036合同/实际正式交付作为可用输入，记录“公共壳/控件”“业务分类色”“文档/媒体像素/语法着色”等不同owner。scope与包输入真实经过consumer观察，不把内容不随肤改变误判缺陷，也不因此豁免整个App公共壳。

研究夹具/探针与必要既有测试路径修正可在本计划唯一树内实施；验证发现的产品/框架缺陷只做受控最小反例与owner交接，不为证明go顺手修通所有业务。实际修复另有已获授权合同才可接受其正式交付，不消费WIP。

## 3. 技术栈

现.at样例/组件/store/API、正式主题数据/scope，UI现MCP/Playwright/原driver，媒体/图像fixture、code_editor、DataTable/tree、sysinfo；Python/JS定向验证。语言/Man/Web只在所选真实消费者必要时固定来源，无新测试引擎。

## 4. 需求分析与背景调查

已用auto-plan-new及verification-contract，读主题包/作用域/迁移/B1与host transport Spec，031/033归档、036/037合同、[第二阶段调研](../design/autoui/theme-rollout-stage-two.md)，逐例pac、原测试入口和源码。观察UI HEAD `56ed6ac33b364ae4045115561468bc6c4e1c4cea`；十二例共94个.at（含pac），不是测试数。work重新冻结逐文件hash与工具身份。

| 分组/全量目录 | 已观察的真实能力/风险 | 代表探针的关闭方向 |
|---|---|---|
| 阅读/门户：018-book-reader、019-video-app | 书/章节/进度、门户路由/列表/like；均API与多页面 | 阅读/路由中切主题，状态和导航保留；019不冒030播放引擎 |
| 音视频：020-music-player、030-video-player | 真实media scan、队列/seek/时间/音量，pac硬编码用户盘；原native mpv支持 | 小合法媒体扫描→实际播放/seek→换肤继续，失败/空库分列；不读用户媒体盘 |
| 图像：029-photo-gallery、031-image-viewer | scan/thumb/full；opaque URI/session/revision、EXIF/损坏/性能夹具 | 小图真实取回/切图与错误恢复，theme只动壳、不动像素/session；原性能条件不减 |
| 专用绘制：024-charts、031-paint | chart series身份色与geometry；16×16像素/调色板/undo/redo | 切肤改变壳但绘图数据/文档像素不变；分类色与背景对比单独核 |
| 数据/系统/文件：025-sys-monitor、026-database、027-file-manager | sysinfo/杀进程确认、DataTable/SQL样本、真实目录排序/open-with | 数据刷新/选择/隔离目录/表状态跨换肤；原样本不冒真实SQLite写能力 |
| 原生编辑：041-auto-edit | render=vm、code_editor/tab/选择/undo/文件对话框，Vue能力不可推定 | 真编辑内容/选择/dirty/undo跨主题；Vue所选轨实证go或准确拒绝/阻点 |

每组代表必须T-01按现可执行入口冻结，不只选最简单静态页面；优先030、031-image-viewer、031-paint、027、041，并为018/019选一完整路由链。另一同组App仍须逐例源/依赖/支持与门清单，不以代表绿代它已验。选择理由和未测边界进入结果，缺代表可运行链则记录阻断并补能定位原因的实际最小失败证据。

033累计10、037拟迁12，B3仍12；032/remote-frame/stylekit等资产义务单列。此处PLAN-038与目录038-minesweeper不是同一身份，后者在UI037。

## 5. 详细设计

### 5.1 启动与研究边界

实施尚未授权。拟唯一树`D:/autostack/.wt/ui-038/auto-ui`、分支`codex/plan-038-theme-readiness`，所需固定lang/Man/Web/Down兄弟无链接；工作/独立review同树。T-01冻结包/工具/支持/参数/端口和磁盘/原生窗口/解码资源窗，探针各输入规模按既有最小可区分fixture，未批准无上限压力。

036/037非整项硬前置，现path/031/033足够做多数可行性探针；若结论依赖036新渠道，仅接受其正式交付并标该腿条件，不能使用草案当已实现。研究的完成允许结论blocked/no-go，但必须是有边界的实证/明确缺输入、owner和下一关闭动作，不把未跑写go。实施包接受这些限制，不能缩减现支持承诺。

### 5.2 数据与发布规则

原媒体/图像/用户目录不得直接读取：020/030覆盖到小合法本地测试媒体，029覆盖到scope内图像；换肤前后比对解码/当前帧/URI及业务数据与壳色分别的观察点。数据库为关闭副本、文件操作仅隔离目录，sys-monitor结束进程只能本计划自有受控子进程，旁观进程不动。编辑内容/像素/series颜色作为内容哨兵，主题生效不允许把哨兵改成公共token。

每个探针：源/工具hash→实际调用/支持模式→合法正控→最小负控或失败输入→真实可见结果/模型或数据oracle→退出/重连与本次资源回收。探针不得注入最终CSS替代主题路径，不能用无媒体空白框认证播放；重基准/性能阈值更改不在本包。

### 5.3 研究交付与后继

产出十二行支持矩阵：每例源、轨道/真实输入、控件与主题面、公共/内容/字面颜色、原测试/样本、已有交付/未测、缺陷/新能力、owner/精确关闭条件、建议批次及来源。go仅在选定实证覆盖面内成立，未跑的同组例不能go；可以标“源码可接、运行待验证”。

后继拟按媒体播放/门户、图像文件、阅读编辑/数据绘制等完整职责分包，数量和边界由证据确定；每个阻点给实际owner，不把整个ACE/AG/Reader/OS054设成前置。此Plan只交可直接起草实施合同的分包/AC建议，**不自动取后继号或执行迁移**。036解决包渠道，037解决B2，OS055解决clock/宿主，彼此只接受正式接口和影响清单。

### 5.4 实测支持矩阵与发现（T-03/T-04 交付，2026-11-03，r1）

冻结源：组 ui-038 auto-ui@f83fd06f + lang@2f558fc93/man@2bbd954/web@3de5524/down@7c4d5cb；raw 证据=plan-artifacts/plan-038/auto-ui/（各探针日志与脚本）；工具=组内构建 auto.exe（plan-890-verified-r7-t19-17-g2bbd954）。下表结论仅在实测覆盖面内成立；"无主题面"=该例未声明 theme 状态/design 绑定，壳不随族/模式输入改变（实测 luma 恒定/类指纹全同），非缺陷——后继迁移的第一步即补该声明。

| 例 | 实测 | 主题面实测 | 结论（覆盖边界内） |
|---|---|---|---|
| 018-book-reader | vue 原门 5/10 绿（书架/正文渲染/章节导航 URL/控制台净）；书卡→详情空渲染签名 + 进度回写/添加书红 + ThemeToggle 挂载失效（F-10） | 逃生舱 ThemeToggle 生成未挂载（theme-toggle-btn 不在 DOM）；现输入=生成器默认 html.dark | **有界 go（阅读链半通）**；5 红具名 F-10 owner=018+生成器 vue 轨 |
| 019-video-app | vm-smoke PASS（A 初始面/B 设置面板交互；mock 词负控内建） | --theme seeding Δluma 153 实测；设置页 5 accent if-链=迁移主战场（静态具名） | **go（门户/VM 轨）**；vue e2e 未跑=未测边界 |
| 020-music-player | --back-port 覆盖下真实 scan/stream 2 条目（blake3/rel_dir）；原 test 入口 boot 红 F-5（links 冲突）；ToggleDarkMode 死代码 F-11 | seeding ✓ + fixture 派发 ToggleDarkMode live Δ216 ✓ | **go（音频代表，有界）**；playback blocked（F-7） |
| 024-charts | 原门 12/19：类型切换/series/流式全绿（tick 2.48 在 1.5–3.5 门内）；状态契约 7 腿红=F-9（484 组件化后 axisLabels/donutLegend 不存在；golden 差异同因） | 无声明 state；luma 恒 16.8 → 无主题面 | **有界 go（结构/流式）**；测试修定 owner=024 tests |
| 025-sys-monitor | 原门 7/11（退役"用户"页/旧文案红=F-4；VM-merged 不受 8425 保留段影响）；kill 只验对话框+取消 | seeding Δ236 ✓ | **go（有界）**；性能/全量压力未跑=原条件保留 |
| 026-database | 原门 0/8 全陈旧 F-8（断言串与现 UI 不匹配）；app 本体健康实证（标题/tab/表 UI 在树） | 无声明 state；luma 恒 23.5 → 无主题面 | **go（主题=后继补声明）**；测试修定 owner=026 tests |
| 027-file-manager | 真实 fs 链：导航→夹具目录 2 行✅→坏路径拒绝（行/cwd 守恒）✅；原门 T1-T4 绿后触发器漂移 F-3 | --theme CLI Δ234 ✓；应用内 toggle 不存在（desktop-host SetTheme 回写设计）；toast 独立窗不入树/像素（F-12 观察） | **go（文件代表，有界）** |
| 029-photo-gallery | 无原 tests；AUTO_PHOTO_ROOT 夹具腿：scan=2 哨兵恒守（dark boot/live toggle/light boot 全同） | seeding ✓ + 应用内 icon toggle live Δ203 ✓ | **go（图像代表，有界）**；photo_root 硬编码用户目录具名（后继 env-first） |
| 030-video-player | vm-smoke ALL PASS（诚实降级 + mock 词负控 + state 翻转）；assert-style.sh 静态门 PASS；两轨均无 media API（F-6） | VM 降级面未测主题（数据面缺失） | **有界 go（壳/静态门）；media 数据面与播放 blocked**（F-6/F-7） |
| 031-image-viewer | 原门不可达：a2r 生成后端 7 编译错（F-1）；--server vm 挂起（F-2）；静态=全 token 0 hex | 无实际腿（blocked） | **blocked**；vm_matrix spawn 修定已备（F-1 解除即可跑） |
| 031-paint | 原门 31/31 全绿（画/擦/填充/undo/redo/吸色/存取；P553-D1 未触发） | 无声明 state；chrome 跨 env 全同；文档像素哨兵跨进程+跨 theme 恒守 | **go（专用绘制代表，有界）**；无主题面→后继补声明 |
| 041-auto-edit | 原门 50/50 全绿（含 T9 磁盘回读/T10 热重载/T11 keymap/ActQuit）；端口 bind 修定+等待修定已入树 | 无声明 state → 无主题面；语法着色 owner=runtime code_editor（app 零着色面） | **go（原生编辑代表，有界）** |

**具名发现（owner + 关闭条件；最小反例在 plan-artifacts）：**

| ID | 发现 | owner / 关闭条件 |
|---|---|---|
| F-1 | 031iv a2r 生成后端 7 编译错（E0428 `mod types` 重复/E0432 `crate::image` 未生成/E0425/E0422 `ViewerControl` 未降级/E0106 生命周期）——app 源为声明式 control-plane+能力委托模式，020/025/018 同族编译 ✅ 证明为该例形状触发的生成器缺陷 | auto-lang a2r codegen；关闭=`auto run -r vm`（031iv）编译+启动 |
| F-2 | `--server vm`：后端 HTTP 服务起后 UI/MCP 永不启动（029/031iv 双复现；无 api 例不受影响） | auto-ui runtime server-vm boot；关闭=该模式 MCP 可连 |
| F-3 | 027 套件 fixture 触发用 SEP 载荷事件名（`SetSearch\x1fs\x1f`）→ 现行注册表门（PLAN-659 T-05 namespaced 直查）未命中；真实 UI 腿 T1-T4 全绿 | 027 tests + runtime fixture 契约；关闭=套件按纯名+input 修定全绿 |
| F-4/F-8/F-9 | 025（7/11 退役页/文案）、026（0/8 断言串全失配，app 健康）、024（7/19 状态契约 484 后不存在）三处测试陈旧 | 各例 tests；关闭=按现 UI 修定（不补退役 feature） |
| F-5 | 生成消费者依归属根解析依赖：`<例>/rust-workspace` 形态把 auto-web 解析到主检出 → 与组内 auto-lang 的 auto-web-contract links 冲突；组一致路径=producer `target/generated-consumers`（020 手动 boot 实证） | auto-lang generation-core（resolve_producer_root/select_language_producer）；关闭=rust-workspace 形态亦组内自洽 |
| F-6 | 030 无 back contract → `auto run` 两轨均跳过后端，`/api/media/*` 无服务（vite proxy ECONNREFUSED 实证）；media 臂挂载拓扑（back_proxy）与 `auto run` 不接 | 030 back contract + media serving 拓扑（runtime+CLI）；关闭=scan 自 `auto run` 可达 |
| F-7 | 播放腿：VM 需 `--features mpv`+AUTO_MPV_LIB（本机无）；合法小媒体无在界编码器（无 ffmpeg；用户盘禁读） | 后继包前置（夹具+mpv 环境入界）；本研究不缩样 |
| F-10 | 018 vue 链 5 红：书卡→详情空渲染（body 仅导航签名）+进度回写/添加书+ThemeToggle 组件生成未挂载 | 018 例 + 生成器 vue 轨（组件挂载/路由回渲染）；关闭=018 原门 10/10 |
| F-11 | 020 ToggleDarkMode 死代码：msg+handler 在、无任何视图绑定 | 020（后继接线）；关闭=UI 入口可切 |
| F-12 | VM 独立窗 toast 反馈不入树/像素（027 坏路径拒绝仅 state 守恒可见） | auto-ui runtime（toast surface）；观察项非阻断 |
| ENV | back 端口 8320/8330/8425 落 Windows 保留段 8140–8839（os error 10013；037 的 021→5441 同族）；CLI `--theme/--accent` 覆盖 pac，直设 AUTO_UI_THEME env 会被 CLI 按 pac 重写；直连 8018/8019/8027/8031/4029/4429 等不受影响 | 各例 pac 端口修定或 runner override（探针已用 --back-port 绕行） |

**后继分包建议（§5.3 规则：只交可立约的分包/AC 建议，不取号不实施；每例"无主题面"的第一步=补 theme 状态声明/design 绑定）：**

1. **媒体播放/门户包**（020/030/019）：前置 F-5、F-6、F-7、020 死代码接线与端口、019 accent if-链映射。AC 方向：合法小媒体 scan→队列→实际播放→seek→换肤继续；失败/空库分列；030 media API 拓扑收口。
2. **图像与文件包**（029/031-image-viewer/027）：前置 F-1（唯一硬阻断）；029 photo_root env-first 化；031iv 性能门条件保留（本机冷启未达具名）+ P553-D1 债不扩。AC 方向：scan/打开→切图/导航→错误恢复（corrupt.bin）→换肤像素/URI/session 哨兵；fs 全链原门修定（F-3）。
3. **阅读/编辑/数据/专用绘制包**（018/041/026/025/024/031-paint）：前置 F-10（018 五红）；四处测试定位修定（F-3/F-4/F-8/F-9）；四例主题声明补齐（024/026/031-paint/041）。AC 方向：阅读进度/编辑 dirty/undo/数据刷新排序/绘制文档像素跨肤守恒（031p 像素哨兵与 041 语法着色 owner=runtime 的边界维持）。

共享资产义务不变：stylekit/remote-frame-e2e/032 按来源保持，不按业务 app 强套主题；038-minesweeper 属 UI037 不入本批。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/specialized-theme-migration-readiness.md | 无运行支持表→十二例实际支持/证据限制/前置/后继，不把target写已实现 | 知识沉淀可定位 | AC-01..04 |
| SD-02 | modify | docs/specs/ui/theme-app-migration.md | B3泛批次→按实测的分包/入口条件，仍未迁十二例 | 义务不消失 | AC-04 |
| SD-03 | modify | docs/specs/ui/theme-scope-and-publication.md（仅真实支持边界澄清） | 当前已交付能力与专用表面/内容owner区别，不扩承诺 | 避免误用代表例 | AC-02/03 |

## 6. 测试设计

全部未执行，T-01固化真实命令/代表/范围，T-03令探针可运行。原tests入口有旧路径/feature命令，须按现owner修测试定位/显式输出，不按旧命令补退役feature。适用时work读取autoui-verifier，独立final review冻结同树、不修实现。

| Case ID | 不变量/输入 | 实际入口与区分性oracle/覆盖边界 | owner_stage |
|---|---|---|---|
| V-01 | AC-01；十二目录/94.at观察与依赖/原tests | 逐路径源码/清单/测试实际定位，目录全覆盖不重号，031两个目录分别记录；原README和真实入口冲突明确 | work |
| V-02 | AC-02；六类代表真实App合法输入 | 原auto Vue/VM/原native入口，scope/包→壳色变化且业务哨兵不变；至少一真实完整链/组，支持不足有真实失败定位，不拿截图数代运行 | work |
| V-03 | AC-02/03；改族/模式，双实例/嵌入与非法主题 | 实际投影路径，切一方不串另一方、失败保旧；未支持的专用域准确具名，不能强把媒体像素/文档色映射公共role | work |
| V-04 | AC-02/03；损坏图/空媒体/404、错误SQL/路径/编辑恢复 | 所选原错误入口，失败可见、不当成功；合法恢复可再次消费，冻结原oracle/时序/条件，不用空库替代播放 | work |
| V-05 | AC-03；退出/关闭/超时/资源缺失 | 回收本次owned进程/解码/连接/fixture、原用户根零写；GPU/能力缺失与代码失败分开，性能原门未跑明确pending | work |
| V-06 | AC-04；十二行支持/阻点/建议AC和分包 | 来源与结果逐行对应，go覆盖面不超实证；blocked/no-go有owner+关闭条件，未迁目录数不变，后继不含无限新架构 | work |
| V-07 | AC-05；原始命令/产物/状态与SD | 独立review按来源重建代表证据并抽反控；没有代表链又无有效失败定位不能给研究pass | review |
| V-08 | AC-06/07；冻结必要探针、合入/知识/归档清理 | 被测/交付来源一致，Spec只发布实际结论；材料摘要折Plan、guard后清树/输出 | merge |

已核锚：031-image-viewer `tests/README.md`、`vm_matrix.py`/`vue-actions.json`与性能期望；041 `tests/desktop_mcp.py`（原菜单/快捷键与dialog边界）；031-paint `tests/desktop_mcp.py`；030 `tests/smoke.spec.ts`/`vm-smoke.mjs`；018/019原Playwright；其余原tests按T-01选真实支持面，不生造现有CLI选项。未声明运行的原性能档记录条件与后继门，本研究无性能优化代码不为合入重跑全量压力。

仅研究/fixture无语言生产改动不跑语言Cargo/docs_gen；必要UI工具/renderer probe改动跑受影响完整职责与原负控，UI原`t/tu/ti`按真实触面定，不跑全AG/ACE。报告列发现/准备/构建/运行/清理及真实执行/原排除，候选不支持不是PASS。

## 7. 验收标准

| ID | 可观察关闭条件 | owner_stage |
|---|---|---|
| AC-01 | 十二例来源/依赖/原门/支持轨/角色逐行完整，目录/样本守恒，V-01 | work |
| AC-02 | 六类代表有真实整链正控/负控或可定位的失败证据，不能凭静态或协议helper给go，V-02..04 | work |
| AC-03 | 内容/公共壳/资源owner、旧性能平台和错误恢复边界准确，用户根不被访问/修改，V-03..05 | work |
| AC-04 | 可直接用于后继立约的分包/AC/具名前置与036/037/OS055边界，十二例仍未迁；每个结论不超实际证据，V-06 | work |
| AC-05 | 独立final review绑定冻结源、证据/遗漏/workaround/SD，研究结论可信，V-07 | review |
| AC-06 | 必要冻结重放/真实合入、canonical/路线/归档读回，不把研究当实现交付，V-08 | merge |
| AC-07 | 全输出/资源对账，摘要折Plan删scope/自有树产物、guard clean，具名持有条件齐，V-08 | merge |

## 8. 执行步骤

| ID | 关闭结果/触面 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | [x]（2026-11-03）授权经 /goal 给付；唯一组 `.wt/ui-038/`=auto-ui@f83fd06f 分支 codex/plan-038-theme-readiness + 兄弟 auto-lang@2f558fc93/auto-man@2bbd954/auto-web@3de5524/auto-down@7c4d5cb（detached，与037交付组合一致）；`cargo build`（组ui 4m16s）+`cargo build -p auto`（组man，default=ui-extra+python+autodown）exit 0，auto.exe 钉组ui target/debug（`0.1.0+plan-890-verified-r7-t19-17-g2bbd954`）；十二例/94.at 逐目录聚合hash冻结（t01-baseline.md）；主题输入通道冻结：AUTO_UI_THEME/ACCENT 播种已声明 state（host.rs:352-373）+会话词 set_theme/set_theme_policy/set_dark_mode + Vue theme_name/dark_mode 约定；六类代表=030/031iv/031p/027/041+018/019 路由链，媒体/照片/音乐全 env 覆盖夹具根，025 kill 只对自有子进程验对话框+取消 | U031/U033；授权经 /goal 给付；AC-01..04 | work |
| T-02 | [x]（2026-11-03）3×只读盘点（结构/主题态/色彩/测试/端口）@f83fd06f：十二行对账表落 t02-inventory.md；计数 94.at 守恒（15/8/10/8/10/7/10/3/6/7/2/8）；横切：0/12 带 dep themes（研究前提成立）；轨道 11 vue+041 vm 固定，VM MCP 测试 9/12、vue e2e 4/12；018 独有 vue/ 逃生舱；内容哨兵与字面身份色三类分界成表；原门陈旧三处具名（024 状态契约、025 退役页、031iv ps1 伪造时序）；用户盘根三处全 env 可覆（020 E:\Music、030 E:\Video、029 Pictures） | T-01；AC-01，V-01 | work |
| T-03 | [x]（2026-11-03）六类代表真实链全部执行（§5.4 矩阵）：031p 31/31+像素哨兵跨进程跨theme恒守；041 50/50；027 fs链+坏路径守恒+CLI主题Δ234；029 夹具腿+live toggle Δ203；020 --back-port覆盖真实scan+handler派发live Δ216；025/019 seeding Δ236/Δ153；031p/024/026/041 无主题面实证（luma恒定）；018 vue链5/10（5红具名F-10）；030 vm-smoke ALL PASS+assert-style PASS但media API两轨不可达（F-6）；031iv blocked（F-1/F-2）；无迁移业务源码；探针/夹具全在plan-artifacts，媒体/照片根全env/CLI覆盖不读用户盘；kill未真杀；测试定位修定两件入树（031iv vm_matrix spawn+--server vm、041 pick_free_port bind探测+T9-T11等待60s） | T-02；AC-02/03，V-02..05 | work |
| T-04 | [x]（2026-11-03）§5.4 十二行支持矩阵+11 具名发现（F-1..F-12 含 ENV）+三后继分包建议（媒体播放/门户、图像与文件、阅读编辑数据专用绘制；每分包前置具名不取号）；判定=go 有界九例（含 018 阅读链半通）+030 壳面/静态门有界 go（media 数据面与播放 blocked 具名）+031iv blocked——go 均限实测覆盖面，"无主题面"四例的补声明=后继第一步工作项；十二例仍未迁 | T-03；AC-04，V-06 | work |
| T-05 | [x]（2026-11-03）台账=§5.4+plan-artifacts（t01/t02 冻结、12 探针日志/脚本、截图与 luma 数）；SD-01 草稿落组内 worktree docs/specs/ui/specialized-theme-migration-readiness.md（未发布，随 merge 落地）；SD-02/SD-03 修改点已备（theme-app-migration §3 B3 行按实测改写、theme-scope §2 主题输入语义澄清——merge 阶段随交付应用）；交接冻结：worktree 提交=测试定位修定×2+SD-01 草稿，review 同树冻结 | T-04；AC-01..04 | work |
| T-06 | 同树冻结独立上下文final review，发现返work并失效相应证据 | T-05；AC-05，V-07 | review |
| T-07 | 必要冻结代表重放、真实合入与canonical/路线/归档，准确保研究完成≠迁移完成 | T-06 pass；AC-06，V-08 | merge |
| T-08 | 全输出/进程资源对账、摘要折Plan，删统一scope/树产物，guard后清树/分支组；archived8/8 | T-07；AC-07，V-08 | merge |

材料唯一`D:/autostack/plan-artifacts/plan-038/auto-ui/`，按repo身份/用途/owner登记。所有探针/外部工程/生成/小媒体/数据库/图像/日志截图/target显式该scope；普通构建在唯一树。不得依脚本默认写项目内证据、其他盘或用户临时根。merge将可复核命令/退出码/关键值/源hash/节选折Plan后删本scope与树产物；仅具名未闭持有owner/理由/关闭条件，后继可用最新源重测，不为等待新计划永久存原exe。未知/他方/活动目录不盲删。

## 9. 复审记录

stage: review | review_scope: final | reviewer_identity/limitations: 本会话即实现会话（无独立 reviewer 角色可用）——按 skill 要求自证据重建结论而非采信执行摘要；重建手段=冻结基线复核（worktree ui-038 @fe1c8a6d 树净、diff 恰 5 文件：2 测试修定+3 SD）、代表证据独立重跑（031-paint 原门 31/31 fresh 进程复现、029 探针全绿复现且 luma 28.3/231.2/Δ202.9 与 work 记录逐位一致、F-1 codegen 7 错逐字复现 E0428/E0432/E0106/E0422/E0425）、其余例原始日志对读（041 50/50、025 7/11、024 12/19、026 0/8、018 5/10、030 vm-smoke ALL PASS+assert-style PASS） | plan_id: PLAN-038 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-038/auto-ui | branch: codex/plan-038-theme-readiness | reviewed_commit: fe1c8a6d（fc610dc7 + SD-02/03 spec delta；全 delta 在 reviewed 提交内，037 同型） | base_commit: f83fd06f | dependency_revisions: auto-lang@2f558fc93 auto-man@2bbd954 auto-web@3de5524 auto-down@7c4d5cb | spec_inputs: docs/specs/ui/specialized-theme-migration-readiness.md（SD-01 新建 @fe1c8a6d，与 §5.4 矩阵逐行一致、不把 target 写已实现）、docs/specs/ui/theme-app-migration.md（SD-02 B3 行按实测改写）、docs/specs/ui/theme-scope-and-publication.md（SD-03 VM 轨输入解析边界=实测口径澄清非新承诺） | acceptance_results: AC-01 pass（94.at 守恒+十二行对账冻结 hash）/ AC-02 pass（六类代表真实链正控；负控=027 坏路径守恒、030 死流+空库分列+mock 词、031iv corrupt 具名 blocked、026 错误预设随套件陈旧具名）/ AC-03 pass（内容哨兵全绿跨肤恒守；用户根零访问——020/030/029 全 env/CLI 覆盖；kill 未执行；进程/临时物回收+树净复核）/ AC-04 pass（判定不超证据：9 有界 go+030 壳面 go+031iv blocked；11 具名发现+三分包不取号；B3 仍未迁）/ AC-05 pass（本记录：重建+反控抽验）/ AC-06、AC-07 = merge-owned pending（V-08：canonical 落地读回、全输出对账+归档清理） | pending_closeout_gates: AC-06/AC-07（merge T-07/T-08） | findings: R1（非阻断，路由记录）031-paint 原套件 pick_free_port 同 041 旧缺陷（connect_ex 探测穿 Windows 保留段——本次 review 实测保留段已漂移至 9188–9487 使 9253/9553 族不可 bind；套件 default 参数在 def 时绑定致 monkeypatch 覆盖无效，review 以 9500 重跑取证）——owner=031p tests，后继测试修定波同 041 方案；不阻断本研究判定（work 证据取得时段口可用+review 已复现） | evidence: D:/autostack/plan-artifacts/plan-038/auto-ui/（review_031p_boot.log=9253 保留段 bind FATAL 实证、031p 31/31 review 重跑、029 review 重跑输出、review_031iv_boot.log=F-1 复现、fe1c8a6d diff 面普查仅 5 文件） | next: merge（T-07/T-08）

stage: goal | plan_id: PLAN-038 | plan_revision: r1 | round: 1 | from: drafting | to: execution_done | status: execution_done(5/8) | outcome: pass | worktree: D:/autostack/.wt/ui-038/auto-ui | commit: fc610dc7 | next: review

stage: work | plan_id: PLAN-038 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-038/auto-ui | branch: codex/plan-038-theme-readiness | base_commit: f83fd06f | code_commit: fc610dc7（测试定位修定×2 + SD-01 草稿；树内仅此三文件，探针/产物不入仓） | dependency_revisions: auto-lang@2f558fc93 auto-man@2bbd954 auto-web@3de5524 auto-down@7c4d5cb（组内 detached，与 037 交付组合一致） | task_ids: T-01..T-05 | evidence: D:/autostack/plan-artifacts/plan-038/auto-ui/（t01-baseline/t02-inventory、12 探针脚本与日志：031p 31/31+theme 腿 PASS、041 50/50+theme 腿 PASS、027 探针 PASS、029 探针 PASS、020 探针 PASS、030 vm-smoke ALL PASS+assert-style PASS、019 vm-smoke PASS、025 7/11、024 12/19、026 0/8（陈旧具名）、018 vue 5/10、031iv blocked 日志、025/024/019 双 boot luma 记录） | 工作结果: AC-01 十二行对账（94.at 守恒）✅ / AC-02 六类代表真实链正控+负控（027 坏路径、030 死流/空库与 mock 词、031iv corrupt 静态、026 错误预设陈旧具名）✅ / AC-03 内容哨兵全绿（像素/URI/文件数据/编辑内容跨 theme 恒守；用户根零访问——020/030/029 全 env/CLI 覆盖）✅ / AC-04 十二行判定+11 具名发现+三后继分包（不取号）✅ | blockers: 无阻断（F-1..F-12 均具名 owner+关闭条件，030/031iv 的 blocked 属研究结论非执行阻断） | 未测/未跑边界: 019/030 vue e2e（Playwright 真实链）、031iv vue-actions、030 播放腿、各例性能/压力档（原条件保留）、018 五红根因深挖（owner 具名） | next_responsible_role: review（T-06，同树冻结独立上下文 final review）

2026-11-03 / stage:new / auto-ui/PLAN-038 r1 / outcome:blocked（只调研立项，实施授权待给） / 十二例/六类与现入口已静态核查，B3能力存在实测不确定性，故先有界验证而非预填全量迁移承诺；本文全部探针未执行。next：授权后work/T-01；036/037可并行，不自动等待全路线。后继实施合同在有研究结果后另取号，不以本包归档抵迁移。

静态预检pass：0–10章节、8T/7AC/8V/3SD唯一连续、十二例六类全覆盖，未跑/未迁/条件前置与研究结论区分明确；038跨活动/归档唯一、UI号源039，合同/设计引用和尾空格检查通过。不是已完成探针或迁移证据。

## 10. 待澄清事项

1. 尚未授权研究执行；本轮不打开窗口/扫描目录/动进程。代表能力缺失可形成有效blocked/no-go结论，但必须有来源、失败定位/缺输入、owner和下一动作，不能用“太复杂”结题。
2. 031/033能力是输入，036未来能力只按正式结果接受；旧支持承诺与实际不符须具名，不按本研究降级。
3. 原专用性能档/解码平台/对话框条件不删；无代码性能改动无需为研究跑全部性能档。新增能力/阈值变化另立约。
4. Reader/媒体/数据库真实产品与教学例不同，本包不接管其开发计划；UI037含038-minesweeper，不因本Plan号相同重复迁移。
