# PLAN-091 T-01 决策记录

按[执行与视觉指南 §5](../../../../docs/design/autoui/common-components-execution-and-visual-guide.md) 结构记录。机械决策随证据落在本页；语义仍以 Plan §5 r2 表为准。

## 1. baseline

| 项 | 值 |
|---|---|
| 仓库 / 树 | `auto-ui` @ `D:/autostack/.wt/ui-091/auto-ui`（组 `D:/autostack/.wt/ui-091/`） |
| 分支 / base | `codex/ui-091-asset-browser`，base = master `da997de2919a268c9aa02d31377de2558872689c` |
| 依赖兄弟树 | `auto-lang` @ `94ac2d062`、`auto-web` @ `26d085f`、`auto-down` @ `72b0970`（均 master HEAD detached，与主检出同基线；经 `../../../auto-lang|auto-web|auto-down` 相对 path 解析）；`auto-man` @ `4a30483`（组内兄弟树，`crates/auto` 经 `../../../auto-ui` 反向指到本工作树，构建 `auto.exe`） |
| 二进制 | 组内 `auto-man/target/debug/auto.exe`（dev profile，3m36s 冷构建）；worktree `target/debug/auto-ui-tool.exe` |
| 观察源码 | `packages/tree-view|pagination|image-viewer`（085/086/089 交付包）；`docs/specs/ui/{tree-view,pagination,image-viewport,file-dialog}.md`；`docs/design/autoui/{component-library-extension-strategy.md,common-components-execution-and-visual-guide.md}`；`crates/auto-ui-runtime/src/ui/child_emit.rs`、`dynamic.rs`（emit 派发臂 ~3770-3870）、`aura_view_builder/child_widgets.rs`（路由登记 ~355-400） |
| 089 运行期先例 | `docs/reviews/plan089/decisions.md` §7 P1-P11（split 媒体票据、cwd 绝对路径、App 级 timer、`view` 保留字、双 exe 重建、登记竞态、计数纪律）全部沿用为本计划探针/门约定 |
| 新增路径 | `packages/asset-browser/`、`crates/auto-ui-tools/tests/asset_browser_component.rs`、`docs/reviews/plan091/`（本目录）；探针夹具在 `D:/autostack/plan-artifacts/plan-091/probe/`（scratch，不入库） |
| 主检出状态 | `docs/plans/105-*.md`、`docs/plans/README.md` 未提交改动为 PLAN-105 既有文档 WIP（非本计划所有，不触碰）；代码路径零 WIP |

## 2. public interface（对 Plan §5 r2 表逐项核对）

- **事件承载机制**：r2 表所列公开事件以带引号 msg 变体实现——AssetBrowser `"queryRequested"(str)`/`"selectionRequested"(str)`/`"previewRequested"(str)`；AssetPicker 追加 `"confirmRequested"(str)`/`"cancelled"`。消费方 `onqueryRequested: .X($event)` 接法（§4 指南）。载荷为 r2 JSON 形态：`{seq, baseRevision, query:{folderId,text,kind,page,pageSize}}`、`{queryId,id,revision,label}`、`{id,revision,label,kind,src}`、`{seq,session,baseRevision,id,revision,queryId}`。`query`/`kind`/`page`/`text`/`records`/`selection` 等键名经探针实测非保留字，无需引号键（区别于 089 的 `view`）。
- **props**：r2 表逐项落地（records/folders/query/querySeq/resultQueryId/total/selection/revision/loading/errorMessage/viewMode + picker 的 open/title/pending/confirmation）。**一处增补**：AssetBrowser 增 `locked: bool = false`（r2"确认pending时禁用query/selection变更"的可视禁用臂——picker 把 `confirm_lock || pending` 传入，普通按钮走 disabled、容器走条件 style 弱化、状态行给文字）。增补为带默认值的加法项，不改冻结字段；复审定夺是否回写 r2 表。
- **受控投影**：沿 NumericField/089 模式——Tick(interval=80) 按 query 原串快照增量重解析镜像（q_id/q_folder/q_text/q_kind/q_page/q_pageSize），selection/confirmation 同式解析；computed 平铺无互引（record_error/folder_nodes/cards/data_current/result_stale/request_pending/has_filter）。
- **接口形状**：AssetBrowser 单文件单 widget（PLAN-614 纪律）；KindIcon 独立文件；工具函数 `asset_browser_util.at`（`use asset_browser_util: …` 显式导入，漏写静默空——085 同款约束）。卡片点击区用 **button**（029 网格同款 `photo_btn` 扁平样式）：实测 `col (onclick:…)` 在 VM 轨被布局平坦化丢 onclick，`mouse-area` 同样丢失；loop 内 `button (onclick: .CardSelect(c.id))` 双轨可靠。
- **预览归属（r2 细化）**：预览面板（089 ImageViewport 同面板视图）落在 **AssetBrowser 内部**（`pv_*` 局部临时视口 + `previewRequested` 照发）。理由见 §3 P-3：VM 轨子件事件按名直达根绑定，中间层 picker 转发臂不参与派发，picker 无法拦截 preview 打开自身面板。

## 3. mechanism probe（探针实录，2026-11-20）

探针夹具：`plan-artifacts/plan-091/probe/`（pac + front-only app，多包绝对路径 dep，App 级 `timer { Tick (every_ms: 80) }`，MCP 驱动，读回区文本断言）。运行：组内 `auto.exe run -r vm` + `AUTOUI_MCP_PORT`。

### 3.1 已核实链路事实（P-1…P-8，均本计划实证）

| # | 发现 | 证据 | 决议/状态 |
|---|---|---|---|
| P-1 | **三.level 包子件 handler 体内的裸引号 emit 会被静默丢弃**：AssetBrowser.CardSelect 内 `."selectionRequested"(…)` 执行（handler 侧 .dbg 旁证 `idx=0 locked=false recs=25`）但应用永收不到；同 handler 同载荷 try 包裹后 100% 送达 | 探针 out/session-*.log + MCP ActionResult（`handler: …CardSelect` 行存在、`state_changes` 空 vs `__emit_payload` 有值） | **包子件 handler 体内所有引号 emit 一律 try{} 包裹**（try 空catch 无副作用）。Browser 内 DoSearch/SetKind/FoldSelect/PageChanged/CardSelect/CardPreview 已全量套用；Picker（2 层）裸 emit 实测可达，保持原样。归因指向 handler_codegen 对子件 handler 体内回调调用的 strip/重写臂（PLAN-051 STRIPPED 机制族），未改运行时——具名遗留观察，不阻塞本计划 |
| P-2 | **中间层转发臂在 VM 轨不参与派发**：picker 的 OnChildQuery/OnChildSelection/OnChildPreview（绑定在 AssetBrowser 元素上）计数恒 0，而应用 handler 照常收到——子件 emit 按事件名直达**根绑定**（`child_emit` ROUTES 表 + 按名匹配；AUTO_DEBUG_EMIT 登记轨迹：AssetBrowser 路由有登记但派发走根） | 探针仪表化轮（fwd_q/fwd_s/fwd_p 恒 0 + app n_sel/n_query 照增）；dynamic.rs 派发臂源码 | picker 的 OnChild\* 转发保留（vue 轨真实组件 emit 链需要它）；VM 轨语义 = 按名隧道到根，两层都声明同名事件恰好单路送达。**中间层不得依赖拦截子件事件做内部状态**（预览因此移入 browser，§2） |
| P-3 | `col (onclick:…)` / `mouse-area (onclick:…)` 在 VM 轨网格循环内被平坦化，onclick 丢失；`button (onclick:…)` 可靠（MCP press 携带 handler） | 快照：卡片点击区 col/mouse-area 无 onclick 行；button 显示 `onclick: .CardSelect`；press 结果 `handler: …CardSelect` | 卡片点击区 = button（`w-full h-auto p-0 border-0 …` 扁平样式，029 先例）；选择/预览是兄弟命中区（嵌套可点击在 Vue 冒泡双触发，085 同族规避） |
| P-4 | **首开 emit 丢失窗**：同一运行内，picker 首次打开后的前若干次子件 emit 即使 try 包裹仍可能整窗丢弃；关闭重开后同路径稳定可达 | 矩阵 L1 偶发失败轮：重开后 L3 选择/确认全部成功 | 门策略：驱动侧 close+reopen 恢复 + 内容轮询重试（探针 drive.py `press_card_sel(reopen_after=3)`）。具名遗留观察（与 089 P6c 登记竞态同族），T-05 门把重开恢复作为标准手段 |
| P-5 | 多包整链真实挂载（asset-browser+tree-view+pagination+image-viewer 同一 Dialog）在 native VM 成功：TreeView 分类行、PaginationControl 页码、卡片网格、ImageViewport 预览面同屏 | 快照 snapshot-picker.txt（`全部/风景/城市/文档` 行、`第 1 / 5 页`、`Lake.jpg` 卡、预览面按钮） | AC-01 native 侧成立；vue 侧由 tools 门生成 + 产品门实跑 |
| P-6 | 查询/确认/取消/代次闭环真实事件实测通过：queryRequested(seq/baseRevision+完整query) → 应用发布 q2/page=2 → resultQueryId 回灌；确认 `{"session":1|2,"seq":…}` → accepted 后父 open=false；取消一次 cancelled；双按只发一次；provider 故障时确认禁用 | 矩阵 P1/L1/L2/L2b/L3/L3b/L4/L6/L6b pass（probe-results.json） | r2 §5 查询流/确认流合同在 VM 轨成立 |
| P-7 | 慢A快B代次门：text="slow" 延迟 15 拍发布，期间 kind 翻转立即应用 B；A 到期被应用侧代次门丢弃（staleDrop=1），组件结果保持 B | 矩阵 L5 pass（一轮实证）；驱动轮询有抖动时以 provider 读回为准 | r2 "应用通过请求代次阻止乱序响应"成立 |
| P-8 | front 进程 `auto.image` 会话（P1 of 089 同拓扑）为记录提供真实媒体 URI：Init 打开会话 → current_uri `/api/__auto/media/{id}/1` 回填 thumbnailUri，卡片 image 元素走真实授权媒体路径；asset-picker 例改用 **back 进程会话**（`api: rust` 合并模式下票据注册表共享，back api `media_uris` 无状态端点）同出真实 URI | 读回 `lake_uri=/api/__auto/media/…`；示例卡片真图渲染 | 缩略图/预览 src = 既有授权媒体 URI（opaque），组件零字节持有 |
| P-9 | **示例相对包路径在 dev 运行也静默缺席**：asset-picker 例 `../../`、`../../../` 两种相对形态 + image-preview 例既有 `../../../packages/image-viewer` 提交形态，dev `auto run` 下组件全部不挂载（快照无组件节点）；绝对路径挂载成功。029 既有 5-up 相对形态同族 | smoke5-7 对照 + image-preview dev 快照（放大/宽度工具栏缺席） | 提交源沿仓库先例保留相对形态（image-preview/029 同款），**产品门必须原地重写为绝对路径**（089 P6b 规程）；跨例的路径解析基准问题具名为仓库级观察，不在本计划内改运行时 |
| P-10 | a2r 生成后端的自定义模块 lift（`use attachdb`）产物引用 `once_cell` 但 axum 族 manifest 不含该依赖 → back 编译失败（E0433） | smoke2.log error[E0433]；生成物 attachdb.rs:4 `use once_cell::sync::Lazy` | 示例改用 031 同构无状态 back + 应用 Storage 承载附件日志（029 favs 同款应用自有持久化）；生成器缺口具名，不在本计划内修 |

### 3.2 tools 生成门（已落地，永久门 T-05 复用）

`cargo test -p auto-ui-tools --test asset_browser_component` → **2/2 pass（5.61s，dev profile）**：

- `official_asset_browser_compiles_through_both_generators`：AssetBrowser/AssetPicker 经 rust+vue 双生成器（ui_build）编译携带 widget 名。
- `vendored_closure_generates_and_missing_dependency_fails_named`：asset-browser+tree-view+pagination+image-viewer 四包整目录字节一致 vendor 到第二绝对根 → vue 工程生成成功（App.vue 绑 AssetPicker；AssetPicker.vue 带 queryRequested/selectionRequested/confirmRequested/cancelled 面；AssetBrowser.vue 组合 PaginationControl/TreeView/ImageViewport）；删依赖后 strict 生成 = 具名失败（含 "asset"）。

### 3.3 探针矩阵残余（诚实边界）

- 驱动矩阵 `loops` 共 18 腿；多轮累计已通过：P1、L1/L1b、L2/L2b、L3/L3c/L3b、L4/L4b、L5、L6/L6b、L9（分页 page=2）、L10 系（预览面/Escape）与 L12 系（身份失效拒绝/重试）分批通过；**单轮全绿尚未达成**——首轮开窗丢失（P-4）+ 驱动固定时序与异步回灌的竞态是残余根因。处置：门版（T-05 verify_asset_picker.py）以内容轮询 + 重开恢复 + Case 计时纪律重建，不以本探针驱动为准。
- 键盘腿（方向键焦点/Enter 选择/Escape 先预览）在矩阵 L11 分批通过前不定案；`onkeydown.escape` 已挂 browser 根 col（KEscape），与 dialog onclose 的先后实测待 T-05 定案——若 Escape 双达（预览关 + 对话框取消），以记录为准在 T-03 决议（预览期间压制取消或调整绑定面）。

## 4. dependencies

| 依赖 | 交付状态 | 本计划用法 |
|---|---|---|
| PLAN-085 TreeView | 已交付归档（`packages/tree-view`；交付提交见归档收据） | 分类侧栏：`{id,label,children}` 薄适配为全键节点（ab_adapt_folders）；受控 selected=q_folder、expanded=组件局部呈现态 |
| PLAN-086 PaginationControl | 已交付归档（交付 `765251b9`；Spec 已落主） | 结果区分页：page/pageSize/total 受控投影，changed→完整 query 请求；showPageSize=true、hideWhenSingle=false |
| PLAN-089 ImageViewport / ImagePreviewDialog | 已交付归档（pin=`5dbc3661`，strategy CL-09 记录） | 浏览面内预览：`packages/image-viewer` 的 ImageViewport（src=授权媒体 URI）；ImagePreviewDialog 本计划不强制消费（预览为同面板视图，不嵌套第二 Dialog） |
| FileDialog/主题/Dialog 内建 | 已交付 | dialog 组合模式（dlg_open 镜像、cancel_armed 单次武装）沿 084/089 先例；主题走现有 background/card/border/primary 语义 |
| 跨仓变更 | **无** | 运行时/codegen 未改（P-1 归因为遗留观察，不在本计划内修） |

## 5. runners

| 轨道 | 入口 | 状态 |
|---|---|---|
| tools 生成门 | `cargo test -p auto-ui-tools --test asset_browser_component` | 本机 Windows 可用（2/2，5.61s） |
| native 产品门 | `python tests/product/verify_asset_picker.py --auto-bin <组内 auto.exe> --ui-tool-bin <worktree auto-ui-tool.exe> --out-dir <isolated> --backend native`；会话沿 numeric-fields 模式（ready 文本/退出控件按本例适配，不复用 FileDialog Session） | T-05 实现；探针已验证 `auto run -r vm` + MCP 全链 |
| vue 产品门 | dep 物化 + vite + Playwright（089 V-04R 收口轨形态：媒体走 back HTTP） | T-05 实现；本计划缩略图/预览可走 back API 真实 HTTP 图源，规避 089 P10 |
| Linux native | 无本机 runner | 单列 not_run，不并入总绿 |

## 6. next

- 允许进入：T-02 收尾（README 前）+ T-03（树/分页/预览/Dialog 接入已随包实现，逐项过 V 腿）+ T-04（examples/asset-picker + 029 网格/搜索迁移）。
- 前置缺口：无阻塞性缺口。P-1/P-2/P-4 为具名运行时观察，按本页决议绕行，不混入 T-02 自由设计。
- 未决机制：Escape 双达定案（§3.3）；门版驱动在 T-05 重建。


## 9. T-05 运行期发现补充（2026-11-20 续，P-12/P-13）

| # | 发现 | 证据 | 决议/状态 |
|---|---|---|---|
| P-12 | **handler 体内 f-string 参数插值（${param}）使 handler 静默中止**：InvalidateSelected 的 `.status = f"...${nid}"`（nid 为 handler 参数）执行后后续语句全部不执行（无报错） | 钩子按压 handler 命中但 status 无变化；改串拼接后立即生效 | 示例已改串拼接；f-string 参数插值缺口具名（auto-lang owner），本包内未再用该形态 |
| P-13 | **029 LBrace 根因订正（非间歇竞态）**：`use { package: asset_browser ... }` 误置于 `029/src/front/app.at` 顶层而非 `widget App` 体内（对比 picker demo / image-viewer 均在 widget 体内）导致 LBrace 解析失败；另 029 `Init` 未发布初始 `.query` 致 `q_error` 常驻、旧 `app-029*-back.exe` 占端口致 `AUTO_PHOTO_ROOT` 夹具失效 | 移入 `widget App` + `Init` 先发布 `.query` + 清旧 back 进程后，029 迁移 4 腿（库面/查看器/收藏/目录导航）全部稳定通过 | 纯 `.at` 与门夹具修复已落地；取消“间歇竞态”挂起 |
| P-14 | **delivery 普查方法**：MCP ActionResult 的 `delivery: sent=N delivered=M` 是可观测的投递丢失探针（delivered=0 = 状态写不落）；主页面裸按压 2/10、分段隔离/降频/清缓存均无效——丢失窗与组件、对话框、包层数无关 | guardtest/rate*.log 普查 | 门驱动以 delivered 计数 + provider 读回双重校验；运行时修复由 delivery 计数器直接验收 |
| P-15 | **Vue 轨 `computed` f-string 与 `DialogContent` 网格约束**：`computed` 内 `f"共 ${.total} 项"` 在 `vue.rs` 生成 `undefined`；`DialogContent` 默认 `display: grid` 下子节点未设 `min-h-0 overflow-hidden` 导致 25 卡网格将分页条与底栏 `确认`/`取消` 挤出 680px 视口 | `gate-vue-r1/r2/r3` 截图与 DOM 断言对照 | `asset_browser.at` 改用视图层条件插值；`asset_picker.at` 与 `asset_browser.at` 补 `min-h-0 overflow-hidden` 约束后 Vue 轨 5/5 全绿 |

最终门证据（2026-11-20）：
- **tools 生成门**：`cargo test -p auto-ui-tools --test asset_browser_component` → **2/2 pass**（7.83s）。
- **native 产品门**：`gate-native-final/report.json` → **18 pass / 0 fail / 0 not_run**（1075.6s，含 `asset-picker` V-01..V-06 与 `029-photo-gallery` V-04 四腿），SHA256 `62E45AC4F2CE0D8E3CC1591F429A204627B88D972DD9CFF3F0CBFA733B0CD1E8`。
- **vue 产品门**：`gate-vue-final/report.json` → **5 pass / 0 fail / 0 not_run**（66.6s，含 `v01-picker-vue.png` 整面截图），SHA256 `A77847BCCDEE44F700D5AA7F58D92AC5BFFF9E1A09812844C56F91A26A5E4D34`。
- **Linux native**：无本机 Linux runner，单列 `not_run`（不并入总绿）。
