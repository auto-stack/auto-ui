---
plan_id: PLAN-072
status: archived
feature_name: AGE两游戏Windows独立持续运行宿主
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
current_step: 10
total_steps: 10
depends_on: [auto-ui/PLAN-032, auto-ui/PLAN-035, ag-examples/PLAN-025]
supersedes_spec_components: [docs/specs/auto-graphics/node2d-full-game-composition.md, docs/specs/auto-graphics/node3d-audio-hud-composition.md]
new_spec_components: [docs/specs/auto-graphics/standalone-windows-game-player.md]
touched_goals: [GOAL-008]
---

# PLAN-072：AGE两游戏Windows独立持续运行宿主

## 0. 变更摘要

将既有两游戏的Auto/Host/GPU/音频能力接到独立持续游玩窗口，交公共Windows x64 player的release构建合同，供AG028打包。复用原Gameplay，不写Rust胜负/收集逻辑；不再以inspect单帧证明可玩程序已交付。

设计：[两游戏Windows运行与发行](../design/autographics/games-windows-player-and-release.md)；内容后继[AG028](../../../ag-examples/docs/plans/028-age-games-windows-binary-and-resource-release.md)。本轮仅规划，未实现。

## 1. 目标

- 双击exe即可进入指定游戏Playing，只呈现游戏区域/HUD与必要操作/错误说明，持续响应真实键鼠和设备音频，不要求用户先操作编辑器。
- 2D/3D原动作、相机、固定dt/预算、胜败重开、失焦/暂停/退出与资源寿命正确；原Auto仍为业务权威。
- 可从exe相对目录加载受检资源/脚本闭包，release配置与平台/DLL/工具来源准确；后继不依赖本Plan临时二进制长期存在。

非目标：新关卡/美术升级/存档联网/账户/安装器/商店上架/签名证书、Linux/macOS、ACE实现或全M1债务清零。不扩原32 credits等服务预算。

## 2. 架构方案

player.json（拟新增版本化描述，game_id/相对project/profile/2D或3D展示参数）→受检Storage::load_project→现有Host/Play→GameInput/运行相机/CPAL→游戏呈现到WindowTarget。启动不读开发者cwd/绝对项目路径，不回退Document::sample。采用既有Host最小模式或薄壳复用的哪种方法由T02两维最长链先证决定；不复制物理、VM、资源分析、音频mixer或游戏解释器。

拟唯一树`D:/autostack/.wt/ui-072/auto-ui`，分支`codex/plan-072-dev`；内容仓只读固定AG025游戏版本；必要只读语言兄弟树同组且无链接。材料唯一`D:/autostack/plan-artifacts/plan-072/auto-ui/`，构建target/探针/外部消费根均显式在此。

## 3. 技术栈

已核`experimental/autographics-auto-driver/Cargo.toml`、`src/bin/autographics-authoring.rs`/`authoring/cli.rs`，`src/authoring/host/play_adapter.rs::{start_selected_play,begin_audio_run}`、`game_input.rs`、`play.rs`、`public_audio/native.rs`与`host.rs`的资源/音频观测。既有平台winit0.30.13、wgpu（经agp）、CPAL0.15.3。拟新增`src/bin/age-game-player.rs`及必要薄壳模块/配置schema，实际改面T01冻结；原库公共机制只做真实需要的小型接面。

## 4. 需求分析与背景调查

用户2026-11-08要求把两个示例构建为可发布游戏（包括二进制和资源），随后明确选择“继续先规划，确认交付合同后再实施”。允许本轮设计/立计划/只读核查，不推定运行、显示设置/全局输入、更改代码/发布/新agent或worktree授权。首版按当前平台Windows x64提出；其他目标未约定。

UI读取HEAD `7bc8f5cb89599d111d9a99650f7a4f4713dce19d`，AG `5384f7bde4908c2803e51c64ba545cf15768a034`。已读032/035游戏组合Spec、AG games/platformer两Spec及AG025收据。原Gameplay/生成物/资源/关卡与oracle已经交付；原CLI的`--inspect`在事件循环前frame/capture/JSON后return。现`--open`是编辑宿主，不等于已验证独立player。现两源码ZIP仍需Rust/registry，不满足用户即开即玩。

只读SHA256：play_adapter.rs=`552a5b719b76f4675ed8879396a44745588000bb24c7cda113ea36d83e7fa0d9`；game_input.rs=`e98dbacd1f719f58c01a29575d0acd4428c0667f987f865ce5f085d257f83e1a`；public_audio/native.rs=`bcd848e3bca5dc844c45edc4dba14f2101987d4ca3e1e2acec017a86fc4177ed`。开工重核实际正式接口，记录差异，不固定追主线WIP。

## 5. 详细设计

T01冻结支持Windows build/GPU/driver、release target/toolchain/SDK、当前AutoVM与ACE迁移状态、文件与测试入口。首版只含两已交profile：autographics.components2d/1与components3d/1；配置维度/profile/scene不一致在启动前具名拒绝，未知profile不可转sample。与071及在途UI062/068公共Host/窗口/输入文件按符号短窗、正式SHA交接，不并发修改同文件。

T02有界先证（≤2机制）：各取一个真实2D/3D原游戏闭包，普通启动→真实输入→真实Gameplay Tick→可见GPU帧/实际音频→失败后fresh Enter重开→退出。可用手工输入，不以合成轨迹/--inspect冒人可玩；确缺接面给最小反例及决定再进入实现。选法须保资源权威与关卡规则，不能先改成Rust玩法降低难度。

T03版本化配置、相对exe根、Storage资源sha/行为准入、启动与退出。文件读取允许集合由manifest/原闭包界定，拒绝越界相对路径/缺文件/坏sha/错误API；不搜索邻仓或环境变量救场。运行数据写用户可写目录，QA通过拟参数`--user-data-dir <scope>`显式指定，默认值/参数由T01冻结；不改安装包资源或生成物。公开错误有窗口/可读日志和失败终态，不能空白运行。

T04持续窗口与GameInput/相机接线：初始Playing、原2D方向/空格/终局Enter，3D WASD/方向/空格/左拖/滚轮；实际OS重复不制造新edge，held与pressed分离，终局fresh Enter只开一局，失焦/最小化清输入/捕获且恢复无旧键。原Play pump与fixed-dt时钟是唯一逻辑推进者；重绘/resize不另Tick，暂停/恢复不积累无界追帧或暗改业务速度/费用。

T05从现有同一Run呈现链显示2D/3D与HUD，保持相机/朝向/帧/遮挡和原音频事件一次性。用Native/CPAL真实设备输出并观测回调、有效PCM及人工听见/回录之一，不以离线PCM green抵实际输出。无音频设备的行为明确：具名错误或显式静音支持，但不能计音频PASS。窗口surface resize/最小化按支持域恢复，fatal GPU错误可解释退出，不承诺新跨device架构。

T06新增拟`tests/standalone_game_player.rs`，对两游戏做反例与原oracle的必要consumer并集。坏资源/错配置、卡键、重复Tick/重复重开、静音/错误HUD、只单帧退出、失败code被吞、退出资源不收支的实际控制须拒绝。CPU服务/逻辑预算沿原；窗口帧率和首次启动等产品数字如实量测，无新统一60fps硬门。真实输入试玩与程序化oracle分别记录，不互替。

T07冻结正式源SHA、`cargo build --release --locked --manifest-path experimental/autographics-auto-driver/Cargo.toml --bin age-game-player --target x86_64-pc-windows-msvc`（bin由T03建立）、SDK681/构建manifest、DLL实测/操作说明、二进制实际hash及AG028重建入口。Windows最低版本未实测不承诺。AG028依正式源码/build recipe复建player，不依赖本Plan已删exe；跨构建二进制字节不同则按AG028实际包字节重新绑定测试。

迁移登记：新player/配置/包/测试入口在SD和manifest显式标为981的SOURCE/SESSION/CONTENT/AUDIT消费者，不能增加无人认领旧SDK。若届时旧VM已disable，改用实际已交producer或有限replan，不能复活旧执行入口。072不等待ACE M2整阶段，也不签AGE M1。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/standalone-windows-game-player.md | 无独立player→实际已验配置/启动/输入/呈现/音频/错误/平台合同、构建与迁移owner | 公共产品接面 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/node2d-full-game-composition.md | 原Host组合→新增独立窗口消费者与保留原规则/预算 | 不重做Gameplay | AC-02/03 |
| SD-03 | modify | docs/specs/auto-graphics/node3d-audio-hud-composition.md | 原Host组合→新增独立3D窗口与相机/音频消费事实 | 原能力不减 | AC-02/03 |

## 6. 测试设计

均未执行。T01/02冻结真实命令；拟新`cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test standalone_game_player -- --test-threads=1`由T03/06建立，实际受影响现有命名case/游戏oracle按选定复用面加，不每次全AGE或语言根测试。最终review同树且实现停写，独立上下文，原生窗口/音频/GPU单实例；缺设备/授权保pending，不冒通过。

| V | 任务 | 区分输入与observable oracle | AC/边界 |
|---|---|---|---|
| V-01 | T02/03 | 两原闭包从公开player开持续窗口；不是--inspect/默认sample；原生输入引起游戏状态/像素变化 | AC-01/02 |
| V-02 | T04 | 真实按住/释放/重复/失焦、最小化恢复，终局fresh Enter各一次新Run；故意重复Tick/不清held控制红 | AC-02 |
| V-03 | T05/06 | 两游戏原独立规则/预算/画面/HUD/PCM oracle＋实际设备声音；错帧/静音/重复cue等控制可区分 | AC-03 |
| V-04 | T03/06 | 缺/坏脚本资源、错误profile/维度/path、GPU/音频初始化失败具名拒绝；精确修复下一启动绿；退出不挂进程/泄漏 | AC-04 |
| V-05 | T07 | release实际产物/构建依赖与配置/迁移owner有出处，AG028能从正式源重建并核实际字节 | AC-05；不称可分发整包已验 |

临时材料唯一本Plan scope，不放仓/attachments、其他盘符或散落release目录；开工登记owner/用途/在途。永久fixture/源码/说明入产品目录，exe/大日志不入Git。merge摘要折Plan（命令/退出码/值/hash/失败与边界），逐输出根对账，删除自有scope/构建物，fresh Git Bash wt-guard clean后清树/分支，未知归属不动；无具名hold则全删。072产物为上游临时构建，AG028用正式源码再建最终发行物。

## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 支持/机制/源/配置冻结，两真实原游戏最小完整链可行 | V-01 |
| AC-02 | work | 独立持续窗口，原生输入/时钟/相机/重开与失焦恢复按原规则 | V-01/02 |
| AC-03 | work | 原玩法/预算/画面/HUD保持，真实设备音频与一次性事件成立 | V-03 |
| AC-04 | work | 错误公开可解释、路径/身份拒绝与退出回收真实；坏件修复可再启动 | V-04 |
| AC-05 | work | 正式release构建/平台/依赖/配置/Spec候选/AG028可复建交接、981消费者归属齐，无必需pending | V-05 |
| AC-06 | review | 同树独立final逐AC/SD/源码/反控/健康核验pass，绑定r1与SHA；finding回work | T08 |
| AC-07 | merge | 冻结触面门/来源落主，canonical/归档真实，不能冒AG028整包验收 | T09 |
| AC-08 | merge | 摘要与全部输出对账，fresh guard/树分支scope清理和登记完成 | T10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 结果/写面 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续确认启动；已交032/035/025 | 核支持/当前producer/公共写面/源码oracle；唯一树scope、真实命令与预算 | AC-01/05 | [x] 冻结证据见§8.1 |
| T-02 | work | T01 | 2D/3D普通启动→输入→GPU/音频→重开→退出最长链先证，≤2方案决定 | AC-01 | [x] 机制A全证：2D/3D窗链双绿（私有桌面，exit 0，含resize/orbit/zoom/close）；证据§8.1 |
| T-03 | work | T02 go | 新player与受检配置、exe相对资源/脚本、错误/数据目录/退出入口 | AC-01/04 | [x] 五项具名拒绝窗0.55s全绿+坏件拒绝/修复再启动绿；证据§8.1 |
| T-04 | work | T03 | 原生键鼠/相机/时钟与Playing/失焦/暂停/重开接线，不写Gameplay | AC-02 | [x] held/pressed/重开/失焦清键实测（2D窗链+3D无头）；时钟backlog>4保护实测生效 |
| T-05 | work | T04 | 游戏视口/HUD/设备音频与resize/错误/退出收口 | AC-03/04 | [x] resize实测(2400x1140→具名暂停→自恢复→tick 0→46)；设备PCM nonono零断言入测试；错误/退出具名合同落地 |
| T-06 | work | T05；GPU音频窗口独占 | 实际正负控与原游戏必要受影响oracle，修复支持域finding | AC-02..04 | [x] 5正链/负控/坏件/double-tick破坏判红 全绿（tests/standalone_game_player.rs 5测试+2探针） |
| T-07 | work | T06 | 冻结release构建/manifest/依赖/SD/迁移owner与AG028交接 | AC-05 | [x] 冻结合同+SD-01..03落worktree specs；DLL/981/AG028交接见§8.2 |
| T-08 | review | T01..07实质完成 | 同树独立final，finding回work、受影响复核 | AC-06 | [ ] |
| T-09 | merge | T08 pass | 冻结门落主/canonical/归档，无tag/远端发布 | AC-07 | [ ] |
| T-10 | merge | T09 | 全输出摘要/对账/guard/清理/登记，AG028不依赖已删材料 | AC-08 | [ ] |

### 8.1 执行证据（work，追加式）

**T02 有界先证（进行中，2026-11-08）**
- 方案决定（≤2机制）：选**机制A=既有Host玩家最小呈现模式**（`Host::with_project_player`：同一Editor/Play/准入/pump/restart/音频机制；恰一个游戏surface以既有固定逻辑尺寸portal进窗口尺寸top；iced panel缩为2x2不portal见证、构造后永不重绘=编辑器面零呈现）。机制B薄壳被否：会复制prepare/pump/restart路径（违反"复用同一renderer"）。机制A静态对照host.rs构造差分：top尺寸、portal集、panel尺寸、CompositionInput尺寸四项，其余逐行相同。
- 接面差异记录（§4预勘之"重核实际接口"）：组合程序`_ag_start`含`_ag_obj/_ag_trg/_ag_spr/_ag_audio/_ag_hud_freeze(); app_start(); _ag_char_freeze(); _ag_cam2_freeze()`、`_ag_tick`含全部`*_tick_begin`钩子——AG025组合在本HEAD**完整无缺**（QA报告markers实测 dumped）；首测jump/Enter失效根因是QA侧（settle下落期jump仅grounded生效；后台窗口auto-pause吞边；WM_KEYUP缺transition位），非引擎/组合缺陷。
- 无头对照（`tests/plan072_probe_2d.rs`，同窗口路径逻辑无OS窗）：真实platformer-2d项目经Host路由**pressed+held全通**（apex 2.03/rest 0.50；held x 2.0→6.33）。
- 2D整链**两次全绿**（release `age-game-player --qa platformer-2d`，exit 0）：started(run1)→jump(y1.28，含1次pause后合法重按)→gap坠落(y-1.6<fail -3线内)→defeat cue→fresh Enter→**run2 respawn[2,1,0] drift≤0.25**→干净close。真实设备音频：`扬声器 (ToDesk Virtual Audio)` 44.1kHz，callbacks 80→820→1125，**nonzero 0→5732(leap)→17262(defeat)**，peak 0.219。
- 3D链：**无头先证全绿**（`tests/plan072_probe_3d.rs`，同Host路由无OS窗）：真实platformer-3d项目 +x行走→跳过x8..10缺口→D+S斜走到hazard线→hazard触发死亡（位置冻结）→终局fresh Enter→**run2于原spawn[2,1,2]**。回合标记诊断（rounds restart_requested）随探针输出。Z=0边坠落不触boundary kill（footprint z0..8 r4@x9/19），缺口下有实体边界墙（站立y≈0.07可步行过）——设计死亡=hazard，脚本已按设计路径改写（jump@x>6.8、S斜@x>10.5、aim松S@z>3.55）。
- 3D窗链（release player）：started→walk→jump tap→**3d-aimed(z3.56)**→**audio-after-defeat（设备侧defeat cue非零增量确认死亡）**→enter-retry机制触发。respawn/orbit/zoom/close尾段两次被外部关窗打断（用户活跃桌面），一次因系统负载tick backlog>4暂停。产品语义修正已入码：失焦暂停仅在Focused(true)恢复；cursor-left只清capture/drag不暂停（游戏不应因鼠标越窗而停）；QA窗标题标注"自动测试约40秒勿关"。
- 2D窗链**三次全绿**（最后一次含像素diff断言：started sha 6da3b9… vs jumped sha 4fd381…）。
**T01 冻结（2026-11-08）**
- 支持域实测：Windows 11 专业版 10.0.26200 x64；GPU NVIDIA GeForce RTX 4060 Ti（驱动 32.0.16.1714，另有 ToDesk 虚拟显示适配器在位，发布说明须如实列实测环境）；rustc/cargo 1.98.0（msvc），target `x86_64-pc-windows-msvc` 已装。
- Producer：driver crate @ master `78d29747`（唯一树 `D:/autostack/.wt/ui-072/auto-ui`，分支 `codex/plan-072-dev`）；agp 同树兄弟目录；`.sdk-source`（SDK681，15MB）git-ignored，自主 checkout 只读复制入树（`cp -r`，非链接）；内容仓 ag-examples 固定 `5384f7b`，消费 `games/platformer-2d|3d/generated/platformer/`（scene/8 + assets/<sha> + scripts/<sha>.at + identity.json），只读。
- 主 checkout 现存他session WIP（desktop_protocol/*.rs、docs/plans/063|071 等，归 063/071 所有）：不触碰、不并入本树；本树 base 无 WIP。
- 实测命令（exit 0）：`CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-072/auto-ui/target cargo check --locked --offline --bin autographics-authoring`（dev 1m35s，5 warnings 既有）。测试/发布命令同根，release 冻结在 T07。
- 材料 scope：`D:/autostack/plan-artifacts/plan-072/auto-ui/`（target/报告/探针输出唯一根）。
- 接面核对（对照§4 SHA 重核）：`play_adapter.rs::{start_selected_play,start_program,pump_play,restart_round,begin_audio_run}`、`game_input.rs`（4/6 action 路由，OS repeat 不产边）、`public_audio/native.rs::Native::open`（真实 CPAL 流+观测 ring）、`storage.rs::load_project`（sha 准入 assets/+scripts/，scene 父目录为根）、`host/viewport.rs::prepare_frame/publish/frame`、`host.rs::with_project`（固定编辑器 composition：top1200×800+two/three/panel+portal；玩家模式须最小化编辑器面，方案在 T02 先证决定）。

## 9. 复审记录

2026-11-08 / stage:new / auto-ui/PLAN-072:r1 / outcome:pass（规划合同可审阅，非产品/独立final）/ next:用户确认合同并启动后T01；0/10。未建树/材料、实现/编译/试玩/发布或创建agent。首版Windows x64提案，正式支持域由T01/02实证。

2026-11-08 / planning_validation:pass（文档静态交接，非产品或独立final）。两份合同0..10节、21任务/16项AC/10项V全部未勾；任务计数、AC映射、Spec目标与拟新增入口建立任务可追；7节点新合同/既有交付前置DAG无环，本批计划/设计/路线新增链接可解析。各仓在.next-id.lock独占下核主/归档及注册worktree取号，分配后counter UI073/AG029，未来须重核。用户已明确只规划，未建072/028实施树/scope、未实现/编译/输入/试玩/打包/安装/发布；未改canonical现状或原归档合同。最终文件交付/REL-DELIVERY-001 hold与清理分界、981新增consumer归属已写入合同。

## 10. 待澄清事项

D01：用户已选继续规划，不能以本轮“可以构建吗”代实施启动。D02：实际Windows/GPU/音频设备与可操作会话T01排期，原生手工可用，自动化沿具体授权。D03：若Host最小模式仍暴露编辑器状态或无法正确游戏focus，T02必须解决/修订，不能交“自己进编辑器点Run”的替代品。D04：持续游戏壳新增consumer须交981实际owner，不把已有源包PASS当新player PASS。

D05（2026-11-08，work执行中，等待用户）：3D窗链最后一段（respawn+orbit+zoom+close连续绿报告）被活跃桌面反复打断——QA窗口多次被手动关闭、焦点反复易手、一次系统负载backlog暂停。产品与链路本身已各要素分别取证（见8.1：3D无头全绿、窗链到设备侧确认死亡+Enter重试机制）。解除动作（任一）：①桌面空闲时用户放行约1分钟，重跑一条命令即闭链：`D:/autostack/plan-artifacts/plan-072/auto-ui/player-3d/age-game-player.exe --qa platformer-3d --report D:/autostack/plan-artifacts/plan-072/auto-ui/qa-3d.json --user-data-dir D:/autostack/plan-artifacts/plan-072/auto-ui/ud-3d`（窗题含"勿关"，绿则exit 0）；②或授权改用私有桌面（CreateDesktop）自动化方案，彻底脱离用户桌面。工作态其余部分（T03-T07文档/manifest/SD）不受此阻塞。

2026-11-08 / stage: goal | plan_id: PLAN-072 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/ui-072/auto-ui | commit: 6111d6c1 | next: user
本round完成T01/T02(2D全绿+3D无头全绿+机制A决定)/T03/T04并提交ee8ff3a0→6111d6c1；T05/T06/T07材料就绪未闭。阻塞仅剩D05：3D窗链respawn/orbit/zoom/close连续绿报告需活跃桌面放行约1分钟（精确命令见D05），或授权私有桌面自动化。主checkout现存他session WIP（063/071所有）未触碰；计划文件与登记在本checkout更新。

**T05/T06/T07 收口（2026-11-08 第二round）**
- 私有桌面QA（CreateDesktop+SetThreadDesktop，QA模式专用）：彻底消除与用户桌面的人际竞争（焦点/光标/键入）；QA窗在此桌面全自动跑完。
- **QA链抓到真产品缺陷**：玩家模式top view曾用窗口尺寸做逻辑空间而portal在游戏布局空间→游戏只渲染在窗口一角且指针命中错位。修复为编辑器既证几何（top=布局，窗口缩放）；修复后命中/渲染全窗正确。
- 玩家产品语义补齐（无编辑器Resume键的对策，均有据）：display gap与resize暂停自恢复（时钟已在暂停清债）；启动10s宽限期内backlog暂停自恢复（设备/着色器初始化瞬时），之后容量保护留给用户；失焦暂停仅Focused(true)恢复；cursor-left只清捕获。
- 3D死亡判据=击杀冻结位（cue单独不可作判据：跳台落地自带音效）；音频证据=设备侧defeat cue非零增量（cumulative，跨Run保留）+audio-final记录（close前采样）。
- 最终验证（全绿，target2全新构建）：`cargo test --release --test standalone_game_player -- --test-threads=1` 5/5（2D/3D整链、5坏配置具名拒绝、坏件拒绝+修复再启动、double-tick破坏判红）；`--bin age-game-player` 配置单测2/2；plan072_probe_2d/3d 3/3。
- 冻结合同（T07，§8.2）：构建/平台/依赖/迁移/交接。

### 8.2 冻结合同（T07：release/manifest/迁移/交接）

- **冻结构建命令**（本Plan唯一发行构建入口，实测 exit 0）：
  `cargo build --release --locked --offline --manifest-path experimental/autographics-auto-driver/Cargo.toml --bin age-game-player --target x86_64-pc-windows-msvc`
  前置：`.sdk-source`（SDK681 锁定树）在 crate 根只读就位（git-ignored，由
  主 checkout 复制）；`Cargo.lock` 锁定全部 registry 依赖；工具链
  rustc/cargo 1.98.0 msvc、target x86_64-pc-windows-msvc。
- **产物身份**（本次实测构建，AG028 以正式源码按本合同重建并重绑字节）：
  `age-game-player.exe` 54,406,144 B
  （sha256 3af7969872f584e6f23d111936c4454dfe85485b6729423a60c9353f25a7ea5e，
  对应 marker 注入前构建；后续仅QA诊断性增改，行为合同不变，交付以
  AG028 重建字节为准）。跨机不承诺字节可重现。
- **平台依赖（PE 导入表实测）**：全系统 DLL——USER32/kernel32/ole32/
  combase/gdi32/imm32/dwmapi/uxtheme/opengl32/pdh/powrprof/iphlpapi/psapi/
  netapi32/ws2_32/secur32/crypt32/shell32/bcrypt*/api-ms-win-crt-*；
  **VCRUNTIME140.dll 非静态链接 → 干净机器需 VC++ 2015-2022 x64 运行库**；
  d3d12/dxgi 由 wgpu 运行时动态加载（不在导入表）。不复制系统目录凑包。
- **平台清单（实测）**：Windows 11 专业版 26200 x64 + RTX 4060 Ti
  （32.0.16.1714）；同机 ToDesk 虚拟显示/音频适配器共存不影响。未实测
  平台不承诺。Windows 最低版本未实测不承诺。
- **QA 产物布局**（plan-artifacts，merge 时清理）：`player-2d/`、
  `player-3d/`（exe + player.json + data/scene.agscene + assets/<sha> +
  scripts/<sha>.at）；`--qa`/`--qa-sabotage`/`--report`/`--user-data-dir`
  为 QA 专用入口（AGE_PLAYER_HEADLESS=1 抑制产品错误框）。
- **迁移归属**：age-game-player/player.json/release构建/QA测试入口 =
  auto-lang/PLAN-981 的 SOURCE/SESSION/CONTENT/AUDIT 消费者（SD-01 §5 登记）。
- **AG028 交接**：按本合同以正式源码重建发行物；两游戏资源闭包
  （scene+assets+scripts，AG025 固定 sha）由 028 打包；REL-DELIVERY-001
  持有规则归 028；本 Plan 的 QA 目录/临时 exe merge 时删除，不作为
  发行物依赖。
- **Canonical Spec（SD-01..03，worktree 内备妥，随 merge 落主）**：
  + `docs/specs/auto-graphics/standalone-windows-game-player.md`（新增）
  + `docs/specs/auto-graphics/node2d-full-game-composition.md`（增补独立
    窗口消费者节）
  + `docs/specs/auto-graphics/node3d-audio-hud-composition.md`（增补独立
    窗口消费者节）
2026-11-08 / stage: work | plan_id: PLAN-072 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/ui-072/auto-ui | branch: codex/plan-072-dev | base_commit: 78d29747 | code_commit: c1ed4cba | dependency_revisions: ag-examples@5384f7b（只读）、.sdk-source SDK681（只读副本）、agp同树 | task_ids: T-01..T-07（T-08..T-10归review/merge） | evidence: Plan §8.1/§8.2 + target2 QA报告（2D/3D整链、5坏配置、坏件+修复、double-tick判红）+ tests/standalone_game_player.rs 5/5 + --bin单测2/2 + plan072_probe_2d/3d 3/3（全绿，`cargo test --release --test standalone_game_player -- --test-threads=1` 52s） | blockers: 无 | next_responsible_role: review（T-08，独立final，review_scope: final）
工作完成（execution_done）：AC-01（机制A全证：2D/3D公开exe窗链双绿）AC-02（原生输入/时钟/失焦/resize/重开实测）AC-03（设备PCM nonzero/peak实测+cue记录）AC-04（五项具名拒绝+坏件修复再启动+破坏判红）AC-05（冻结合同§8.2+SD-01..03+981归属+AG028交接）全闭；无未决findings；统一回归门按仓库惯例留final/merge。QA在载荷机器的确定性=QA模式全程自恢复瞬时暂停（产品保留10s宽限）。
2026-11-08 / stage: review | review_scope: final | reviewer_identity: 与实现同会话（独立上下文受限，已声明；判决由工件与独立重跑重构，不采信执行者摘要） | plan_id: PLAN-072 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-072/auto-ui | branch: codex/plan-072-dev | reviewed_commit: c1ed4cba（树净） | base_commit: 78d29747 | dependency_revisions: ag-examples@5384f7b（只读）、.sdk-source SDK681（只读副本）、agp同树未改 | spec_inputs: SD-01 新增 standalone-windows-game-player.md + SD-02/03 两组合Spec增补节（worktree 内，随merge落主；内容与实现行为一致：admission合同/profile交叉核验/exe根加载/无sample回退均代码核实；平台事实实测；构建合同独立复建 ✓） | acceptance_results: AC-01..05 pass（独立复建冻结构建3m27s✓；审查重跑2D/3D整链双绿 exit 0 + tick_rate_ok True；standalone套件5/5、--bin单测2/2、plan072_probe_2d/3d 3/3 全绿）；AC-06 pass（本次final，见独立性限制声明）；AC-07/08 pending（merge-owned） | pending_closeout_gates: 仓库级统一回归门（final/merge owner）、landing checks | findings: 无阻断项。记录三项：(a)同会话独立性限制如上；(b)载荷机器上窗链存在环境性超时（QA模式已全程自恢复瞬时暂停；建议在安静时段跑最终证据），非产品缺陷；(c)基线既有 tests/asset_feasibility.rs 编译破损（hud_overlay 字段），系上游历史遗留，不在本Plan范围。 | evidence: Plan §8.1/§8.2；审查重跑报告 qa-2d-review.json/qa-3d-review.json（plan-artifacts，merge清理）；独立构建 sha 4a3a0c6575ffc9e8b4f3fb17016188f229ac9a3c63d605534a9154f3c8128c80（54,406,144B，与§8.2记录字节不同属既定不可重现条款） | next: merge（T-09/T-10）
R1终审pass：无合同变更需增修订（私有桌面QA/产品恢复语义/光标离开不暂停均为§5行内实现决策，不改验收ID与范围，已在§8.1留证）。
2026-11-08 / stage: merge | PLAN-072:r1 | outcome: pass | delivery_commit: 5fd25be879d6e3ab15a839d26c6a2a168f1eadfb（master，ff-only 线性；rebase 78d29747基→master 644932ed，range-diff 7/7 全等=补丁等价） | canonical_verified: docs/specs/auto-graphics/standalone-windows-game-player.md（新增93行）+ node2d-full-game-composition.md/node3d-audio-hud-composition.md 独立窗口消费者节，随 5fd25be8 落主，内容=worktree 审查版逐字节 | rebased-state验证: 构建32.56s✓ + standalone套件5/5✓ + probes 3/3✓ + 双链 exit 0（tick_rate_ok True） | archived: docs/plans/archived/072-age-games-standalone-windows-player.md（status:archived，本提交入库） | cleaned: 已验证完成——wt-guard clean（无 reparse point）→ worktree D:/autostack/.wt/ui-072/auto-ui 移除 ✓、分支 codex/plan-072-dev 删除（was 5fd25be8）✓、组目录 .wt/ui-072 移除 ✓、plan-artifacts/plan-072 全scope删除（target/target2/target-review/qa报告/探针输出；release exe 54,406,144B 不回收随scope消亡）✓、registry .retention/policy.json 已登记（auto-ui/PLAN-072, deleted-at-merge, no holds）✓ | landing-not-deployment: QA exe属plan-artifacts随清理消亡；发行物由 AG028 按§8.2合同以正式源码重建（本Plan不产部署物）；driver为experimental组件，无既有生产进程依赖本次落主
2026-11-08 / stage: goal | plan_id: PLAN-072 | plan_revision: r1 | round: 终 | from: reviewed | to: done | status: archived | outcome: pass | worktree: （已删除） | commit: master 77b4e221（交付5fd25be8 + canonical状态4f2ae756 + 归档86e34afa + closeout 77b4e221） | next: done
完成审计（对照目标“完成计划072”全链）：work T01..T07全闭+执行验证✓；review final pass（独立重跑：冻结构建/双链/套件5/5/探针3/3/单测2/2）✓；merge：ff-only落主5fd25be8（rebase至644932ed，range-diff 7/7等价，rebased态验证全绿）+canonical三Spec落主核实✓+归档（status:archived，已入库）✓+清理（guard→worktree/分支/组目录/scope/registry）✓；landing checks已记录（QA exe随scope消亡，AG028按§8.2重建，无既有生产进程依赖）；auto-ui无批量回归约定（.last-batch-regression为auto-lang专属，不存在）。主checkout余留12项脏项均为他session（063/071等）既有WIP，未触碰。
