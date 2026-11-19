---
plan_id: PLAN-091
status: drafting
feature_name: AssetBrowser 与 AssetPicker 通用资源浏览选择
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-14
plan_revision: 2
current_step: 0
total_steps: 8
supersedes_spec_components: []
new_spec_components: [docs/specs/ui/asset-browser.md]
touched_goals: []
---

# PLAN-091 通用资源浏览与选择

## 0. 变更摘要

提供AssetBrowser / AssetPicker，用于附件、图片素材、封面等已有资源库的浏览/选择。复用029 Gallery、085 TreeView、086 PaginationControl和089图片预览；不是另一个本地FileDialog。路线CL-15，见[组件库strategy](../design/autoui/component-library-extension-strategy.md)。本轮只立项。

## 1. 目标

缩略图/列表、文件夹或分类、搜索、类型过滤、分页、单选、图片预览、加载/空/错误态；嵌入页面或作为modal选取。Windows/Linux native VM和Vue消费相同授权资源数据，MCP可按稳定ID操作。先服务图片/普通附件，其他类型用元数据和类型图标展示。

非目标：原始磁盘枚举、上传/删除/移动文件、资源导入/鉴权服务、模型材质解析、媒体库索引、音视频解码、跨页多选。保留FileDialog当前磁盘路径选择owner。

## 2. 架构方案

应用provider提供授权分类、当前页records/total/selection → AssetBrowser呈现 → 发查询/翻页/选择请求 → 应用完成业务查询和更新props。资源引用为opaque ID/revision/URI，不能从显示名称或URL字符串拼本地路径。AssetPicker组合该浏览器与Dialog，确认表达选择意图，应用最终验证资源当前身份和业务权限后消费。

新增`packages/asset-browser/`。085/086/089各有唯一源，禁止内嵌副本；T-01先核定多包移根消费，T-03及迁移等待三依赖评审交付pin。029写入顺序在089查看区迁移之后，避免两计划同时改Gallery。

## 3. 技术栈

现有`.at`、普通列表/网格、ImageSurface/089、TreeView/085、PaginationControl/086、Dialog/主题/MCP；provider和实际资源存储继续归应用。没有新文件系统host能力，也不通过系统文件弹窗选择资源。

## 4. 需求分析与背景调查

用户要求挑通用的一半先建实现计划，本合同为跨应用资源选择。当前为计划/路线阶段，后续按Plan实施；无模型/预算/代理及跨仓发布指定。观察HEAD `93e6b924749a45c5c6c390725a4bb2d8a10bfc07`。

已读Specs索引、示例归属/主题/结构、native验证及当前FileDialog边界。无AssetBrowser专门Spec，交付时新增。

| 已核路径 | 当前观察 |
|---|---|
| `examples/ui/029-photo-gallery/src/front/app.at` | 已有photos/dirs、搜索/分类/密度、打开图片/失败/空/目录缺失；SHA256 `69C5C86B33CF8BACF9FEF715EFA225F74A7FEEF6D24921575525CDF089D2288E` |
| `examples/ui/030-video-player/src/front/playlist.at`、020 tracklist | 资源列表也是业务消费样本，不将其播放/队列逻辑搬进通用浏览器 |
| `packages/file-dialog`及`docs/specs/ui/file-dialog.md` | 本地真实路径选择已有owner，资源库选择应复用视觉/交互而不复制磁盘服务 |
| PLAN-085/086/089 | 分类树/分页/图片预览的正式依赖，目前未交付，需确切版本后接入 |
| `crates/auto-ui-tools/tests/file_dialog_component.rs` | 多包与移根生成验证的现有参考机制，不能依赖另一个主检出回退 |

## 5. 详细设计

- records至少有`id, revision, label, kind, thumbnailUri, resourceRef`，分类独立稳定ID；src/thumbnail是现有授权媒体URI，非用户磁盘路径。重复ID/非法结构具名错误，不能选中另一条同名记录。
- query包含分类、搜索串、类型、page/pageSize；组件发完整查询请求，应用过滤/排序/计数，组件不再本地过滤一套制造total失配。新查询期间显示loading并失效旧结果action；应用通过请求代次阻止乱序响应。
- 首版单选，应用持selected资源身份/元数据，翻页可保留同查询选中项并明确显示选择摘要；改变分类/搜索/类型由应用清除旧选择。请求失败保留上一有效结果但显式标过期/不可确认，不能伪装空库。
- 选择和预览分开，缩略图缺失显示类型图标；图片用089预览，其他类型首版只元数据，不承诺全格式预览。键盘可移动/选择，Enter确认选择意图，Escape在预览层先关闭预览。
- Picker受控open/selection/pending，确认返回`id+revision`而非裸路径；调用方在真实接受handler重新验证当前资源和权限。一轮确认请求只发一次，在父状态反馈前也须阻止重复点击；错误反馈后允许重新选择/重试。r2固定请求/结果修订协议，T-01验证接线。组件不自动写文档或假称资源已存在于消费应用。
- 旧节点绑定实例、查询代次、资源ID/revision，不按网格位置。换查询、同ID新revision、双实例、关闭/重开时旧action拒绝；当前正例可用。
- 复用Dialog焦点/背景隔离和关闭恢复；空、无匹配、加载失败、缩略图失败分别可观察。大列表通过086分页，不一次渲染全部条目；代表夹具1003条证明尾项可选，不承诺任意网络延迟。
- 029网格/搜索/目录消费正式包，保留其真实后端、收藏、相册和资源URL契约；新通用附件选择示例向应用写目标ID日志/附件引用，不复制或改写源文件。092的资源属性扩展可消费本包，但不作为本计划必须改动。

### r2 执行接口与样例

按用户2026-11-14再次审核与样式参考要求补齐；先读[执行与视觉指南](../design/autoui/common-components-execution-and-visual-guide.md)的091布局与消息约定。

| 接口 | r2合同 |
|---|---|
| AssetBrowser props | `records/folders:List=[]`, `query:str`（JSON，见下）, `querySeq:int=0`, `resultQueryId:str=""`, `total:int=0`, `selection:str=""`（空或单条JSON）, `revision:int=0`, `loading:bool=false`, `errorMessage:str=""`, `viewMode:str="grid"`（grid/list） |
| AssetPicker props | 同上，加`open:bool=false`, `title:str="选择资源"`, `pending:bool=false`, `confirmation:str=""`（JSON反馈）；非磁盘FileDialog |
| 查询/选择事件 | `queryRequested(str)`含seq/baseRevision及完整query；`selectionRequested(str)`含当前queryId、id/revision或明确清空；`previewRequested(str)`带资源身份 |
| 确认/取消 | `confirmRequested(str)`如`{"seq":15,"session":2,"baseRevision":7,"id":"lake","revision":"r3","queryId":"q2"}`；`cancelled`关闭本轮选择意图；组件不再发第二个假“业务accepted” |
| 错误 | `invalid_records/invalid_query/stale_selection/provider_error`；配置/用户操作错误与普通空结果区分 |

record样例：`{"id":"lake","revision":"r3","label":"Lake.jpg","kind":"image","thumbnailUri":"/api/__auto/media/lake/3","resourceRef":"asset:lake"}`；selection为相同身份及label摘要，不把resourceRef当路径。query样例：`{"id":"q2","folderId":"nature","text":"lake","kind":"image","page":1,"pageSize":100}`，page与086一致为1-based。id为调用方分配的当前查询身份，外部查询也必须换新id。

查询流：发queryRequested(seq)立即进入本地等待态，应用接受请求后发布新query/id、`querySeq=seq`及loading=true；完成后发布`resultQueryId=query.id`与records/total、loading=false。只有结果id匹配当前query、最后请求已反馈、无错误时数据可确认。失败通过querySeq和errorMessage反馈，旧响应不得覆盖新查询；跨页但folder/text/kind相同保留selection，其他条件变化清空。搜索输入通过Enter/搜索按钮发请求，类型/分类/分页立即发；无隐藏debounce时序要求。

确认流：每次open从false到true递增session；发确认时同步锁定本地请求，不能等pending prop才锁。应用验证返回`confirmation={"session":2,"seq":15,"status":"rejected","code":"not_found","message":"资源已失效"}`后解锁并显示错误；成功为status=accepted且父应将open=false，本轮确认保持禁用。错session/seq反馈忽略；关闭/取消令旧请求失效，应用在真正写入引用前也核对本轮仍有效。调用方只有这一处接受handler执行业务写入。

同页单选和键盘焦点分开，网格/列表切换不更换资源身份；无src缩略图用类型icon。预览打开089临时视口，不改变selection；如果普通单击选择后需要预览，另用命名预览动作。跨页保留的selection摘要必须由调用方持续提供，不能依当前页下标猜旧资源。

folders沿085的嵌套节点形状`{id,label,children}`提供，最终消费其正式版本的薄适配，不另造磁盘目录模型。确认pending时禁用query/selection变更，仍允许取消/关闭；外部资源身份变化撤销旧确认。AssetPicker已在modal内时优先复用089的ImageViewport作为同Dialog预览视图，不强制嵌套第二个Dialog；独立AssetBrowser页面可组合ImagePreviewDialog。Escape先退出预览再按原选择器取消规则处理。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ui/asset-browser.md | 应用内列表→统一资源身份/查询/选择/预览/确认合同 | 附件素材复用 | AC-01,AC-02,AC-03 |
| SD-02 | add | docs/specs/ui/asset-browser.md | 无正式门→MCP代次、单源依赖、Gallery/附件真实消费与双端门 | 不选错资源/不造第二FS | AC-04,AC-05,AC-06 |

拟树`D:/autostack/.wt/ui-091/auto-ui`、分支`codex/ui-091-asset-browser`，未创建；实现/独立最终review在同树顺序执行，canonical本轮不改。

## 6. 测试设计

T-04新`examples/asset-picker/`及授权资源provider夹具，T-05新`crates/auto-ui-tools/tests/asset_browser_component.rs`、`tests/product/verify_asset_picker.py`，采用统一`--auto-bin/--ui-tool-bin/--out-dir/--backend native|vue`。测试只用隔离合成资源，普通输入/选择驱动查询；provider/API只读回作为oracle。均为拟执行。

| Case ID | AC / 入口与输入 | 区分性断言 | work owner / 范围 |
|---|---|---|---|
| V-01 | AC-01,02 正式包1003条，同名不同ID、分类/搜索/过滤/尾页 | query参数与真实返回ID/total/分页完全对应，最后资源可选；不能只更新页码/本地label | T-02/04/05 |
| V-02 | AC-02,03 空库/无匹配/失败/慢A后B、失效缩略图、跨页单选 | 状态准确；A不能覆盖B；跨页摘要保留、query变化清选择，失败不可误确认；图失败不阻止选合法项 | T-02/03/05 |
| V-03 | AC-03,04 预览/重复确认/取消、资源同ID换revision、权限撤回、双实例/旧action | 一轮确认一次且传精确ID/revision；应用拒绝过期选择、不写附件，错误后可重试；旧action拒绝，新正例可用 | T-03/04/05 |
| V-04 | AC-05 Gallery与附件示例实际操作→资源API/结果引用 | 展示图和选择身份正确，Gallery收藏/目录控制不回退，取消不增加附件/改源文件 | T-04/05 |
| V-05 | AC-04,06 Win/Linux native+Vue，浅深/小窗、依赖移根/缺失对照、退出重开 | 所有必要包固定来源；删依赖具名失败不回捡主检出；modal/当前树/owned清理和新会话通过 | T-05/06；Q-01受影响项闭环 |

拟命令：`cargo test -p auto-ui-tools --test asset_browser_component`；`python tests/product/verify_asset_picker.py --auto-bin <cli> --ui-tool-bin <tool> --out-dir <isolated> --backend native`，Vue另跑。T-01记录Gallery真实后端启动/读回及依赖原门；缺平台/窗口/API结果不得skip后pass。

按NVM记录源/依赖/工具/fixture摘要、窗口、当前节点、真实选择/失败对照、截图及正常退出重开；MCP不直接注入resource结果。最终review由后续独立上下文在同树冻结实现后检查，本轮不创建代理。

### r2 补充验证与启动交接

- **V-06**（AC-02/03/06，work，T-02/05）：连续确认两次但父pending尚未回灌时仍只发一次；拒绝反馈解锁后可重试；取消→重开后旧accepted/rejected反馈不得写附件或解锁新请求。查询q1慢返回不能替换q2；按指南§3.2拍单选边框/选择摘要/空错/pending/窄窗。
- T-01需用最小provider实际演示querySeq/resultQueryId与session/seq反馈闭环，不能只在模型里设置pending=true证明防重。依赖085/086/089未交前的probe标probe_only，不计T-03验收。
- T-04迁Gallery时同时核对total、查询id与selection清空/保留规则；不能让UI过滤一套、API计数另一套。

## 7. 验收标准

| ID | owner_stage | 关闭依据 |
|---|---|---|
| AC-01 | work | 正式包及085/086/089完整依赖真实解析/运行/移根，无副本；V-01/05 |
| AC-02 | work | 查询/total/分页/空错/代次/跨页及query变化选择规则准确；V-01/02 |
| AC-03 | work | 预览与选择分开、确认返回ID/revision并由应用校验，取消/失败不写引用；V-02/03 |
| AC-04 | work | MCP/键盘/modal、双实例/资源revision/旧action安全；V-03/05 |
| AC-05 | work | Gallery及附件示例真实资源身份/引用/原业务控制通过；V-04 |
| AC-06 | work | Win/Linux native+Vue、视觉/依赖闭包/退出重开证据齐全，受影响Q-01闭环；V-05/06及执行视觉指南§3 |
| AC-07 | review | 独立逐项复核AC-01…06/SD、资源权限和跨层oracle，绑定提交，无必需门缺口 |
| AC-08 | merge | 新Spec、合并/归档/strategy最终链接及实际清理收据准确，交付完整依赖pin |

## 8. 执行步骤

| 任务 | owner_stage / 依赖 | 路径与结果 | 验证/预期 | AC |
|---|---|---|---|---|
| T-01 | work / 无 | 新`docs/reviews/plan091/decisions.md`核定record/query/pending、现有后端oracle、依赖移根/runner/Q-01；以§5 r2接口为准，机械决策按公共指南§5记录 | 最小真实多包/provider/旧响应探针，决定可运行事件，不复制FS | 01…06 |
| T-02 | work / T-01 | 新`packages/asset-browser/`公开身份/query/单选派生与错误状态 | V-01/02实际包与provider响应断言通过 | 01…03 |
| T-03 | work / T-02 + 085/086/089评审交付pin | 正式树/分页/图片预览/Dialog接入，pending/代次/MCP/键盘 | V-02/03旧节点/权限/双实例及当前正例通过 | 03,04 |
| T-04 | work / T-03 | 新`examples/asset-picker/`、029网格/搜索迁移；保留089查看区及业务服务 | V-04真实图/ID/引用和原收藏/目录结果读回 | 05 |
| T-05 | work / T-04 | 新tools/product目标并落V-01…06、依赖缺失对照 | §6逐平台/后端门，真数据/失败/退出重开；含r2 V-06和指南§3真实视觉/窄窗 | 01…06 |
| T-06 | work / T-05 | README、`docs/reviews/plan091/`证据、SD及strategy摘要；README含r2 props/events、可运行消费例和视觉指南 | 包闭包/工具/资源指纹、Q-01受影响项可复核 | 06 |
| T-07 | review / T-06 | 同树独立review，冻结实现写入 | reviewed commit逐AC/SD结论；失败回work | 07 |
| T-08 | merge / T-07 pass | Spec、合并/归档/strategy交付、实际清理 | 最终文件/版本/树状态正确，不写派生索引/ledger | 08 |

## 9. 复审记录

2026-11-14：stage: new；PLAN-091 r1；outcome: pass（规划可交接）；next: work，从T-01开始，T-03等待085/086/089。新增T-01…08、AC-01…08、SD-01/02，0/8；拟代码与测试未执行。

2026-11-14修订交接：stage: new；PLAN-091 r2；outcome: pass（合同可进入T-01，非实施后独立复审通过）；next: work。依据[本轮复核记录](../reviews/component-plans-087-093-2026-11-14/README.md)处理F-01/F-06/F-09/F-10，补§5接口/数据/状态、§6 V-06及视觉交接，AC-06同步加强；T-01/02/03/05/06执行仍按原8任务计数，当前0/8。Spec增量沿SD-01/02并纳入r2细节。本轮只读源码并修订计划/设计/图片资产，未运行产品门。

2026-11-20：stage: goal（work 进行中，第1轮长会话）；PLAN-091 r2；from: work | to: work | outcome: needs_fix（T-05 部分达成，未到 execution_done）；worktree: D:/autostack/.wt/ui-091/auto-ui | branch: codex/ui-091-asset-browser | base: da997de2 | code: 14dd89d2 | next: work。
已完成：T-01（decisions.md P-1..P-11 全套机制实录+探针矩阵）；T-02（packages/asset-browser 四文件，tools 门 2/2）；T-03（085/086/089 三包真实接入，双生成器+四包 vendor 闭包门）；T-04（examples/asset-picker + 029 库面迁移，dev 挂载实证）；T-05 部分（tests/product/verify_asset_picker.py native 门 picker 矩阵 12/12 两轮全绿 r44/r47，best-of-N 会话重试）。
关键机制发现（详见 docs/reviews/plan091/decisions.md）：P-1 子件 handler 体内裸引号 emit 静默丢失→try 包裹 remedy（browser/picker/pagination 三包全量）；P-2 子件 emit 按名直达根绑定、中间层转发臂不参与 VM 轨派发→预览归属 browser 内部；P-9 dev 运行相对包路径静默缺席（含一次 chr(1) 控制字节损坏事故与修复）；VM 轨 if 条件里 computed 比较恒假→Tick 守卫 state 镜像（空态修复）；FetchUris 无限轮询拖垮 Tick→封顶+异常隔离。
未达成（next 精确动作）：V-05/V-06 视觉与退出重开腿、V-04 029 迁移腿在完整运行中未达成（V-03 后段会话不稳定，P-4 首开 emit 丢失窗+对话框生命周期竞态，best-of-3 未能覆盖）——下轮：先跑 --runs 3 完整门（picker+visual+029 三段），若 V-03 后段仍崩则把 V-03 段落后半（身份失效/旧 vnode/重开清反馈）拆为独立会话段；vue 轨待跑；测试模块耗时表待 T-05 收尾时落 §8。当前 5/8（T-05 计 part）。

2026-11-20（续）：stage: goal（work 进行中，第2轮）；PLAN-091 r2；from: work | to: work | outcome: needs_fix（T-05 继续）；worktree: D:/autostack/.wt/ui-091/auto-ui | code: 14dd89d2 之后追加门韧性提交 | next: work。
本轮达成：native 门 picker 矩阵稳定全绿（r47/r51/r52 三次 12-14/12-14，best-of-N 会话重试下零 fail）；V-03 后段拆为独立会话段（run_picker_stale）；门韧性（cancel emit 丢失窗重试 close_picker_retry、慢路径武装实测、树行结构化走查、预览按钮误匹配修正、完整性下限）；pagination 包 P-1 remedy（4 层链 PaginationControl→browser→picker→app 打通，尾页/分页腿绿）；空态/查询/类型/分类/尾页/跨页/慢代次/确认/落账/预览腿全绿。
新定位（P-4 深化）：**delivery sent=N delivered=0 现象**——MCP ActionResult 报 handler 命中但 VM 派发队列投递 0，连根 App handler 的状态写都不落（InvalidateSelected 空转实测）；按时间窗双模出现，重试可恢复但窗口可持续数十秒。另：handler 体内 f-string 参数插值（${param}）使 handler 静默中止（改串拼接修复）。
未达成（next）：stale 段（身份失效/旧 vnode/V-06 重开清反馈）、visual 段浅色/窄窗/退出重开、029 迁移腿——全部被 delivery=0 窗口阻断（stale 段钩子按压 sent=1 delivered=0 实录）；vue 轨未跑。运行时投递可靠性修复具名超出本计划范围（运行时仓 owner）。下轮可选：降低会话内按压密度重试完整门；或按 execution_done 部分（picker 矩阵）+ 具名边界（P-4 深化）提交复审裁量。当前 5/8（T-05 part）。

## 10. 待澄清事项

r2已固定provider查询和确认反馈协议；T-01验证真实事件/状态接线、Gallery读回、依赖解析与runner。没有阻碍立项的用户决策。多选、模型/音视频深度预览和导入服务保留后续需求，不通过本计划偷偷扩大范围；越界服务依赖需明确owner。
