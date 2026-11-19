---
plan_id: PLAN-090
status: drafting
feature_name: MediaPlayer 与 TransportControls 通用媒体播放
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-14
plan_revision: 2
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/media-player.md]
touched_goals: []
---

# PLAN-090 通用媒体播放与播放控制条

## 0. 变更摘要

从020 Music Player和030 Video Player抽取MediaPlayer / TransportControls正式包。用于附件试听、教学视频、普通音视频预览，复用现有受控video/mpv/浏览器媒体链，不建设新播放器内核。路线CL-17，见[组件库strategy](../design/autoui/component-library-extension-strategy.md)。本轮仅立项，0/8。

## 1. 目标

音频/视频播放暂停、进度定位、音量/静音、倍速、时长/当前位置、加载/结束/错误状态，普通控件和键盘/MCP均可操作。Windows/Linux native VM与Vue同义，至少一份PCM WAV音频和一份已核定编码的WebM视频走真实解码回灌。

非目标：新解码器、媒体库扫描、播放队列业务、剪辑/波形/时间轴、直播/DVR、字幕编辑、DRM、跨进程共享播放器、多窗口画中画和全屏宿主重构。组件不保证所有媒体扩展名都能解码。

## 2. 架构方案

消费方提供已授权URI和播放状态 → 原video受控契约 → native mpv / Vue媒体元素 →真实时间/状态回灌→消费方。TransportControls只发播放/seek/音量等请求，不用timer伪造播放进度；MediaPlayer组合视口与控制条，保留业务状态owner。新增`packages/media-player/`。

控制下行与引擎回灌分开：seek target及其修订不是currentTime，不能每次timeupdate重新seek。默认每实例独立媒体身份；显式共享必须绑定同App/session及引用寿命，不延用示例字面key造成全局串音。

## 3. 技术栈

现有 `.at`、progress/onseek、video、mpv engine/contract/channel、Vue受控媒体、主题/MCP；不新增全局媒体服务。首版数值操作可用标准控件，087不是硬依赖。音频继续复用020的受控video媒体路径，保留非零生命周期挂载需求，不声称原audio标签已具备完整native能力。

## 4. 需求分析与背景调查

用户选择常见且能跨应用复用的一半先建计划；本合同覆盖媒体播放，当前为计划/路线阶段，后续按Plan推进实现。无模型/预算/并行代理指定，不写其他仓或发布应用。HEAD观察 `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读Specs索引、native验证、结构/主题/示例归属。当前无正式MediaPlayer组件Spec；示例SPEC中的旧媒体服务路径不能当作当前owner事实，T-01须核对实际pac依赖与可运行后端。某些文档引用的`crates/auto-media-core/tests/fixtures`在当前UI仓不存在，不将此假路径写为已有fixture。

| 已核路径 | 事实和复用点 |
|---|---|
| `examples/ui/030-video-player/src/front/{controls.at,viewport.at,player_store.at}` | 已有真实回灌/seek比例，controls直接依赖应用store，须改正式受控接口；controls SHA256 `8B944DA9B1609FF30F6963988646197317D305EFB11D31AC840B0D81D7D50B5A` |
| `examples/ui/020-music-player/src/front/{viewport.at,controls.at,player_store.at}` | 音频用video通道，1×1非零挂载避免引擎被裁掉；现key为media，抽包须按实例隔离 |
| `crates/auto-ui-runtime/src/ui/mpv/{contract.rs,engine.rs,channel.rs}` | volume=0…100、时间float秒、rate倍速、seek目标变化才下发及generation已有机制 |
| `crates/auto-ui-runtime/tests/{video_contract,mpv_contract,mpv_channel,mpv_engine}.rs` | 现有实际接线和后端回归目标；环境要求逐项核定，不以模型测试替代解码 |
| schema video/audio与`ui/iced/video_uplink.rs` | audio声明fallback不等于通用音频组件已交；保留原video兼容 |

## 5. 详细设计

- MediaPlayer与TransportControls按r2接口接收受控状态并发普通请求，T-01验证真实媒体链能承接，不让正式包import示例PlayerStore。
- 下行音量0…100、进度/时长秒、倍速与旧contract一致；以seek请求序号变化触发一次定位，而不是每次timeupdate或只看目标数值变化。反复渲染不得钉住播放或重置音量；回灌状态是实际引擎确认，不是请求即成功。
- 首版默认暂停，用户/MCP普通播放动作后开始，Vue沿真实浏览器播放限制；拒绝播放给出可恢复错误，不用全局关闭自动播放策略作通过证据。duration未知/不可seek时禁用定位；seek范围夹取到已知范围，非法数值具名拒绝。
- 提供播放/暂停、静音、音量加减、进度和可编辑定位时间的普通入口；按钮/键盘/拖动共用请求链。倍速候选按现有引擎已支持范围固定并展示，不能假装应用成功。
- 换src/seek/关闭按现有generation拒绝旧帧、旧time/end/error；错误不假装ended并跳到别的媒体。两个默认实例独立；卸载一实例不暂停或释放另一个，显式共享依原owner引用计数。
- 空src、加载、暂停、播放、结束、加载失败、后端不可用分别可观察。组件不扫描磁盘、不持有鉴权凭证、不把URI当shell路径。格式/缺libmpv/音频设备错误有明确能力反馈。
- 020和030都迁正式控制条及媒体包装，删除其控件对store的直接依赖；列表/收藏/目录仍归各应用。原声道/媒体服务不迁仓。
- 自动音频门允许静音或虚拟输出但必须真实解码、时钟/时长和引擎属性可观测；不以timer/mock PASS证明播放，不声称验证物理扬声器。视频用已知帧颜色/位置证明显示随seek变化。

### r2 执行接口与样例

按用户2026-11-14再次审核与样式参考要求细化；先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)中090布局/状态。视频和音频均复用同一受控媒体通道，不增新解码器。

| 接口 | r2合同 |
|---|---|
| MediaPlayer props | `src/resourceId/resourceRevision/label:str`, `revision:int=0`, `mediaKind:str="video"`（audio/video）, `paused:bool=true`, `volume:int=100`, `muted:bool=false`, `rate:float=1`, `seekPosition:float=0`, `seekRevision:int=0`, `sessionKey:str=""`（默认实例独立） |
| TransportControls props | 上述公开控制值加实际`currentTime:float=0`, `duration:float=0`, `seekable:bool=false`, `playbackState:str="idle"`, `disabled:bool=false`；不能把currentTime当seekPosition |
| 请求 | `requested(str)`，如`{"seq":4,"baseRevision":2,"action":"seek","position":10,"resourceId":"clip-a","resourceRevision":"1"}`；action取play/pause/seek/volume/mute/rate，参数按action固定类型 |
| 回灌 | `observed(str)`提供真实资源身份及state/time/duration/volume/muted/rate中已观测字段；不得根据请求提前伪造确认；错误用`error(str)`含code/message和资源身份 |
| 状态/能力 | idle/loading/paused/playing/ended/error/unavailable；duration未知或非正时seek禁用；倍速首版候选0.5/1/1.5/2，后端拒绝必须显示失败 |

Parent在接受seek请求时原子更新seekPosition与seekRevision；**即使position仍是10，只要新用户请求也增seekRevision**。MediaPlayer将每个新seekRevision恰好交给引擎一次；timeupdate和普通重绘不增它。现有`MediaContract`按last_seek_target数值去重，不能直接照搬为新公共请求协议；其position=None会清除last_seek_target，可作为候选适配接缝，但必须证明清除/再次请求确实依序到达引擎。T-01验证最小兼容适配/可选请求token，并保持旧video无token行为不变。渲染epoch也不能当seekRevision，否则每帧都重定位。

资源身份换代失效旧time/end/error/frame；currentTime回灌不会自动推进受控目标revision或重建引擎。play/pause是明确动作，不用toggle请求解释旧状态。volume超0…100、非有限seek/rate及非候选rate给具名invalid_control；有效seek夹取已知duration，未确认状态显示pending而非假paused/playing。

MediaPlayer可保留引擎实际状态的只读投影供内置TransportControls显示，并向外转发observed；这不是另一套请求状态。外置TransportControls由应用将同一真实观察值传入。任何观察值都不能自动重新变成requested，也不能用本地时钟预测成功。

requested参数固定：play/pause无额外值；seek用`position:float`；volume用`volume:int`；mute用`muted:bool`；rate用`rate:float`。observed的实际时间字段统一`currentTime`，能力字段`seekable`，状态字段`state`；总带resourceId/resourceRevision，其他值只在真实观测到时给出。这样020/030适配无需各自猜`value`或`position`的含义。

音频视口保持可驱动真实引擎的非零挂载，不照抄0×0隐藏；默认sessionKey为空时按真实App/组件实例分配，020显式共享只能在同App受控owner里复用，不跨App使用全局"media"。关闭/换源应按owner撤销回调和引用，不能杀共享消费者。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/media-player.md | 示例耦合控件→正式受控播放/定位/单位/能力反馈 | 媒体预览复用 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/media-player.md | 无专门门→引擎generation/实例寿命、MCP及真实音视频双端证据 | 不用假进度冒充播放 | AC-04,AC-05,AC-06 |

拟树`D:/autostack/.wt/ui-090/auto-ui`、分支`codex/ui-090-media-player`，未创建；实现和独立最终review同树顺序进行，本轮不写canonical。

## 6. 测试设计

T-04新`examples/media-preview/`，T-05新`crates/auto-ui-tools/tests/media_player_component.rs`及`tests/product/verify_media_player.py`（统一`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`）。T-01核对实际媒体fixture来源/摘要、编码、backend/libmpv和浏览器，缺素材由T-04在隔离目录产生并记录生成方法，不暗用用户媒体库。均未执行。

| Case ID | AC / 实际入口 | 区分性断言 | work owner / 范围 |
|---|---|---|---|
| V-01 | AC-01,02 正式包普通play/pause/seek/音量/倍速，连续重绘 | 精确请求/真实确认值；播放自然推进，重绘不反复seek；暂停稳定，目标seek到已知视频帧 | T-02/03/05 |
| V-02 | AC-03 PCM WAV+核定WebM、损坏/过期URI、未知duration/缺后端 | 真实解码时长/状态/音频参数，视频帧像素变更；错误可恢复且不假ended；不可seek准确禁用 | T-01/04/05；声明格式 |
| V-03 | AC-04 A切B、seek、旧end/time/frame、双实例及关闭一个 | 旧回灌不串源、不误推进队列；另一实例仍播放；当前MCP正例成功，旧action拒绝 | T-03/05；实际generation和引擎引用 |
| V-04 | AC-05 020/030实际列表选择→普通播放控制→后端/画面 | 媒体身份、音量/进度/画面与选中条目一致；不再依赖全局store控件；无双引擎串音 | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，主题/小窗、移根、退出重开 | 实际控件树/可读状态、引擎退出、owned进程/端口释放、新会话再次真实解码 | T-05/06；缺runner/backend不记pass |

拟命令：`cargo test -p auto-ui-runtime --features mpv-gpu --test video_contract --test mpv_contract --test mpv_channel --test mpv_engine`；`cargo test -p auto-ui-tools --test media_player_component`；`python tests/product/verify_media_player.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue独立跑。Cargo.toml中mpv_engine/contract需要mpv-native，channel需要mpv-gpu，后者包含前者；缺lib时原目标内有自行跳过的路径，不能只看该目标绿。T-01记录实际AUTO_MPV_LIB和库摘要，并由新产品门强制要求真实解码与资源关闭证据。

保留源/工具/库/浏览器/fixture摘要、NVM窗口/操作/读回/失败对照及正常退出重开，Q-01受影响文字项关闭。测试不用物理前台焦点或直接写播放器状态代替普通操作。独立最终review在同树独立上下文，不在本轮创建会话。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/06，work，T-02/05）：播放到20秒→用户定位10秒→继续播放→再次输入10秒，第二次必须真正定位；未有新seekRevision时连续重绘/回灌不重复定位。配真实帧/时钟断言，并按指南§3.2验证paused按钮/文字一致、未知时长、错误及小窗。
- Vue借用旧030 Playwright结构时必须去掉其用户E盘依赖及`--autoplay-policy=no-user-gesture-required`；PCM WAV和WebM使用自有可再分发夹具。保留真实策略，普通点击Play后才能起播；缺后端/编码不被UI模拟时钟掩盖。
- T-01先完成实际受控contract→引擎操作→回灌的单资源小探针，再明确最小token适配和source owner。T-03前不能把“按数值去重”和“按用户请求去重”混成一个策略。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 正式包真实导入、原video契约及移根兼容，无示例store依赖；V-01/05 |
| AC-02 | work | 请求/回灌/seek差量/时间音量倍速语义正确，连续重绘仍自然播放；V-01 |
| AC-03 | work | 已声明音频/视频真实解码与失败/能力反馈区分，不用timer假值；V-02 |
| AC-04 | work | MCP/键盘、generation、双实例/关闭释放及旧动作安全；V-03/05 |
| AC-05 | work | 020和030普通选择/播放业务身份及真实反馈通过，共用正式控件；V-04 |
| AC-06 | work | Win/Linux native+Vue、视觉/移根/退出重开与后端身份证据齐全，受影响Q-01闭环；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立复核AC-01…06/SD和真实解码oracle，无未执行必需门，固定reviewed commit |
| AC-08 | merge | Spec/交付/归档/strategy及实际清理收据准确，不把合入代替运行验收 |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan090/decisions.md`审实际后端/fixture、受控回灌/实例key、音频挂载、runner及Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 小型真音频/视频解码+换源探针，固定格式、事件与owner；假路径/缺后端显式记录 | 01…06 |
| T-02 | work / T-01 | 新`packages/media-player/`公开状态与TransportControls请求，复用旧contract | V-01/02及原video/mpv目标通过，回灌不触发seek循环 | 01…03 |
| T-03 | work / T-02 | MediaPlayer组合、MCP/键盘、generation和实例寿命适配 | V-03实际旧回灌/双实例/关闭对照通过 | 04 |
| T-04 | work / T-03 | 新`examples/media-preview/`、020/030迁正式包，记录合法fixtures | V-04真实媒体/结果读回，无示例store控件耦合 | 05 |
| T-05 | work / T-04 | 新tools/product目标、落实V-01…06和必要原后端回归 | §6命令各轨真解码/负例/正常退出重开，不缺门冒充pass；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 | work / T-05 | README、`docs/reviews/plan090/`证据、最终SD和strategy摘要；README含r2 props/events、可运行消费例和视觉指南 | 版本/媒体/后端指纹及Q-01闭环可核对 | 06 |
| T-07 | review / T-06 | 同树独立review，暂停实现写入 | reviewed commit逐AC/SD verdict，失败回work | 07 |
| T-08 | merge / T-07 pass | Spec、合并/归档/strategy交付、实际资源清理 | 最终文件/版本/owned树核验，不写派生索引 | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-090 r1；outcome: pass（规划可交接）；next: work，T-01。新增T-01…08、AC-01…08、SD-01/02，0/8；拟议代码/测试未执行。

2026-11-14修订交接：stage: new；PLAN-090 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-05/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

## 10. 待澄清事项

r2已固定公开控制/回灌、seek请求序号和普通浏览器策略；T-01验证兼容token适配、真实后端/fixture、key/owner、退出和runner。无阻碍立项的用户决策；若需跨仓媒体服务修改则先建立依赖，不因本仓门难跑删真实解码或Linux/Vue轨。
