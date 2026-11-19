# 主题推广第二阶段：业务示例、专用表面与第二真实产品

> 2026-11-04当前接续：[实例/Reload/生成守门与B3实施合同](instance-state-and-reload-stage-one.md)已接038正式研究并立045–047，041/042与语言930独立规划。本页以下为2026-11-03首次静态盘点/排程历史，不能覆盖现canonical实测/归档与产品独立状态；旧“尚未立约”不作当前指令。

> 2026-11-03，target-state r1。只读调研与正式立约，未运行应用/测试/构建、修改实现、安装包或系统偏好。031/033与OS053、语言918已交付；036仍草案。本文承接[拆分后架构](post-extraction-architecture-and-theme-closure.md)，不替代canonical或各唯一Plan。

> 双轨补充（2026-11-03）：本文B2/B3均为教学Demo；全部实际产品独立盘点/迁移见[AutoOS Design03](../../../../auto-os/docs/design/03-product-theme-rollout.md)与OS056–061。28实际入口不按Demo目录推导；Demo与产品分别修改/分别验收，公共包/接口共享而业务不保持同步。

## 1. 调研结论与立项

| 主合同 | 结论 | 实施门与出口 |
|---|---|---|
| [UI037](../../plans/archived/037-business-examples-theme-migration.md) | B2十二业务示例作为一个保行为的主题迁移包；不是十二个产品开发包 | 获实施授权可先核基线/以现path迁移；采用036渠道的最终验收等U036；不等OS055 |
| [UI038](../../plans/archived/038-specialized-examples-theme-readiness.md) | B3十二例先做**有界支持面核验**，不立即承诺全量主题迁移 | 授权后用实际入口验证图像/媒体/专用绘制/编辑等代表链，给逐例go/blocked/no-go、owner及后继分包；研究归档不算迁移完成 |
| [OS055](../../../../auto-os/docs/plans/055-clock-theme-and-host-evidence-closure.md) | auto-clock为第二真实产品；同时收053真实系统事件与outproc载体证据差额 | T-01可先核载体可行性，正式包接入/最终门接U036；真实事件前须用户批准具体窗口并恢复原值；不能拿协议helper代真实clock |

不重复已有[OS054八应用装配](../../../../auto-os/docs/plans/054-basic-apps-unified-assembly.md)，它明确排除主题/计算器，拥有八产品pin与桌面业务组合；本批OS只改clock pin/必要宿主主题接缝，其他八pin不动。Weather真实产品有独立交互计划008（execution_done）及VM真实数据/Vue桩后端边界，不宜在主题批次顺手认领数据产品差额。

## 2. 观察来源与限制

UI HEAD `56ed6ac33b364ae4045115561468bc6c4e1c4cea`，语言 `8717bd5d3b56c4423c6a75833511575143ad80f3`，OS `092254ce037ef7c7612cf11fe8b3fe00c57a76c6`。clock clone与OS gitlink同为`551d05e065eb2f449c643b3fa4fd671a9ec7a307`，观察时clone clean；工作时重新冻结，本文不授权切分支/还原WIP/改pin。

已读UI主题/包/B1交付Spec、runtime/host-theme-transport，OS shell/host-theme-policy-and-delivery、053归档/KNOWN-DEBT及054，逐目录pac、front/back声明和已有tests/README。产品clock pac/README/tests与weather pac/README/008用于真实来源比较。旧示例README有“单文件app”等陈旧说明，测试脚本有旧语言路径/feature命令；以现文件/CLI实路径核准，不能原样当可执行支持证明。

以下为**源码盘点**，at数含pac和其他.at，不是测试数或执行通过数。未启动任何GUI，没有判任何原红已关闭。

## 3. B2逐例范围和业务守恒

| 目录 | .at文件数 | 观察到的真实链/风险 | 最小区分性业务验收（保原门） |
|---|---:|---|---|
| 011-calculator | 3 | 表达式/科学/32位程序员，stylekit、fit、复制；教学例不是已升级auto-calc | 2+3=5、括号、HEX进位/原错误态，切主题保表达式/模式；科学函数按本例真实已承诺能力，不借产品新版本宣称支持 |
| 012-clock | 2 | Tick、秒表/计圈、倒计时、世界时钟、闹钟storage | 走表→计圈→换肤→继续；倒计时到零、闹钟重启恢复；计时不因换肤重置/重复订阅 |
| 013-todo | 7 | 多组件+store及Rust API；旧Rust生成阻点须当前重核 | 添加/编辑/完成/筛选/清已完成，换肤不丢列表/编辑输入；后端/支持轨不改成静态数组 |
| 014-weather | 4 | demo城市数据模块、条件/AQI样式；非真实weather API产品 | 城市/24h与5日/设置的原handler值不变，按天气类别固定色与公共表面分别核；不新增实时天气要求 |
| 015-notes | 8 | 多组件/store、Rust API、显式dark、AutoDown本机npm link | 新建/编辑/切笔记/搜索/原保存；换肤不失焦或重建编辑文档；编辑引擎依赖有正式来源，无本机路径假绿 |
| 016-calendar | 4 | 月历/闰年/选中日、store设置、显式light | 跨月/选日/Today、原事件/设置；换肤不改变日期/选中/事件，今天/选中/周末身份不混淆 |
| 017-chat | 9 | 多组件/Rust API/联系人/消息/typing/时钟 | 联系人筛选、发送→实际响应/消息、换肤保会话与未发输入；未给真实账户发消息，隔离测试后端 |
| 021-blog-viewer | 8 | 多页面outlet、分类/详情/写作、Rust API | 分类→详情→clap、Write→创建与返回；主题跨路由保持，不能只截图首页 |
| 022-kanban | 6 | 多页/store/Rust API，教学例已从桌面策展退出 | 创建/移列/删除卡片、原拖拽/按钮义务，换肤不丢卡/列/筛选；不重新注册桌面或升级产品kanban |
| 023-realworld | 13 | 注册/登录/文章/评论/收藏/follow及路由、Rust API | 隔离账户的登录→文章/评论→收藏/返回，换肤保鉴权与编辑草稿；不吞入真实auto-blog产品新功能 |
| 028-launcher | 2 | palette/grid、查询/分类/选择、recent storage；独立mock与桌面真注册表有别 | 查询/键盘选中/开关/模式和recent；独立mock不冒真启动，受影响真实宿主动作另用有限受管fixture核 |
| 038-minesweeper | 3 | 确定性雷盘、计时/难度、数字颜色身份 | 原脚本钉定初级首击(4,4)展开29格、踩雷/旗标/重开及各难度；主题不改雷/数字/已揭示/计时数据 |

B2以同一正式themes包与031/033作用域投影为基础，不统一重写store或增加设置菜单。每例给public role、分类/内容色、accent overlay、合法字面豁免映射；“已使用bg-card”不等新族真正进入页面。两个族×明暗逐例观测，实际不支持轨必须具名定位并作为该例交付缺口处理，不能默删。

## 4. B3逐例能力表：为何先做核验

| 目录 | .at数 | 关键现有输入与不能被主题改写的表面 | 核验分组 |
|---|---:|---|---|
| 018-book-reader | 15 | Rust API、书/章节/进度与tree/list，正文/阅读域不是所有字段都随公共壳变 | 阅读/结构 |
| 019-video-app | 8 | 门户列表/路由/播放详情/like，显式dark/pink；不同于030原生播放器 | 门户与路由 |
| 020-music-player | 10 | 真实media scan/播放队列/时间，pac现E:\Music；不得借空库404宣称播放通过 | 音视频 |
| 024-charts | 8 | 编辑数据/图表类型/geometry、series分类色 | 专用绘制 |
| 025-sys-monitor | 10 | sysinfo、刷新、任务确认；仅可结束本次自有测试子进程 | 系统数据 |
| 026-database | 7 | tree/DataTable/SQL/Schema，当前Init样本需与真实能力区分 | 数据/结构 |
| 027-file-manager | 10 | 原真实目录浏览、排序/网格/隐藏/上下文/持久化、open-with | 文件/结构 |
| 029-photo-gallery | 3 | photo scan/thumb/full真实后端，pac指用户Pictures；本轮禁止扫描该目录 | 图像 |
| 030-video-player | 6 | media scan/viewport/OSD/seek/音量/队列/原mpv轨；viewport黑/透明是媒体域 | 音视频 |
| 031-image-viewer | 7 | opaque media URI、session/revision、原小图/损坏/EXIF夹具及性能门 | 图像 |
| 031-paint | 2 | 16×16像素、palette/undo/redo；文档像素色必须跨主题不变 | 专用绘制 |
| 041-auto-edit | 8 | native code_editor、tab/保存/修改标记/选择/undo，render=vm；不可凭Vue生成成功宣称编辑能力 | 原生编辑 |

研究应实测代表性完整链，而不是12张静态截图。尤其媒体需合法小资源与实际后端，不读真实Music/Video/Pictures；数据库用关闭夹具副本；文件操作仅隔离根；code_editor的VM与Vue支持差异具名判定；031-image-viewer原性能/压力条件保留，本研究不缩样或无故重跑整档。

UI038必须为每例留来源/支持轨/角色映射/原测试命令与未执行条件、所需控件和阻点owner；代表探针至少覆盖阅读/路由、音视频、图像、专用绘制、数据/文件/系统、原生编辑六类。输出按整模块建议后继：媒体门户/播放、图像与文件、阅读/编辑/数据/专用绘制，最终以实测再定，**本轮不给尚不明确的B3实施包预分号**。

## 5. OS第二真实产品与053差额

选择clock：已有独立clone，来源等OS gitlink；无后端/出网/凭据依赖，tests/desktop_mcp.py包含秒表/倒计时/世界时钟/闹钟的真实业务oracle，能暴露主题切换导致Tick订阅或storage重建的问题。它不等于UI012教学例，两个来源分别验。

OS053交付的`theme-policy/1`、帧17/18、零装载adopt、standalone config轮询与applied读回可复用。其G-1真实系统事件仍hold；G-2外部自采纳Hello未达broker、缺真实outproc载体，协议单测不代闭环。因此OS055先从clock实际生成/启动→broker/adopt→Hello→push→主题帧评估→result做有界可行性验证，再补必要**已有路径**接缝。不可用手造client/helper代clock、仅配置轮询代outproc推送。

若真实载体依赖新的宿主协议/平台/编译后端，不在研究前承诺“改一个启动参数即可”，T-01应提出具名前置/修订，不能缩为只测standalone后仍称G-2关闭。G-1需在执行到该腿时获用户授权具体系统设置窗口、备份原值并finally恢复；本轮无窗口权限，也不操作设置。最终计划未闭这些必需面不能完整交付，可保留已有不变证据等待条件。

## 6. 顺序与边界

036包发行/输入收口与037的源码映射、038的调研可在授权后按共享文件短窗协调；037/055采用036渠道的最终消费门等U036。037与055互不等待；038给后续B3分包及新控件前置，不抢reader/AG/OS054业务合同。clock的候选SHA、推送与组合pin由055独立clone→OS干净子模块流程完成，不批量改其他产品。

918已交付的准入/窄依赖规则继续生效；新manifest/source变更按正常owner流程审计，不编辑旧918或借永久豁免。Web发行ID集合/schema差额、Man-d派发、全量回归/历史缓存债保各owner，不并入本批主题。

本次立约只做静态来源检查与文档预检，不把计划状态、文件数、候选选型当能力验收；所有新增探针/运行矩阵均为后续获授权的work任务。
