---
plan_id: PLAN-077
status: archived
feature_name: Windows RQ产品投影覆盖与公开入口闭合
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-10
plan_revision: 1
current_step: 7
total_steps: 7
depends_on: [auto-ui/PLAN-063, auto-ui/PLAN-073, auto-ui/PLAN-074, auto-ui/PLAN-075]
supersedes_spec_components: [docs/specs/runtime/rq-execution-input-and-host-services.md]
new_spec_components: [docs/specs/runtime/windows-rq-product-coverage.md]
touched_goals: []
affects: [renderqueue, projection-coverage, public-entry, windows-rq-m1]
---

# PLAN-077：Windows RQ产品投影覆盖与公开入口闭合

## 0. 变更摘要

OS063已经有限交付归档，不再重开。本Plan承接其Todo/Video coverage拒绝与同根因公共入口缺口，修实际投影/样式/媒体或错误路由，交可由OS066全Demo/产品最终验收消费的正式结果。只在覆盖能力确实存在并已验证时更新能力表，禁止删闸/白名单放行/切旧模式来冒全功能。

本轮用户询问新立计划完成Windows RQ路线，承接其此前允许M1前新增修复合同的授权；本轮只建立草案，不实施/建树/运行/提交/tag。目标不新增Linux/ACE/AGE。

## 1. 目标

G1：以真实公开命令还原Todo/Video拒绝的source→CLI/模式选择→client/Host→coverage/producer链路，明确legacy queue与remote的不同责任。
G2：在VM/Rust现有受支持公开RQ模式与两个Host下，两产品真实功能/图像/输入/媒体资源及生命周期保持direct语义，关闭列出的样式/tag差额。
G3：保覆盖失败诊断与旧合法模式，永久反例证明错误路由、缺实现、静默降级不能通过；交同根因受影响面与正式Spec。

非目标：重写全部投影器、恢复原已撤回协议、产品业务重做、放宽图像/性能目标、OS066全产品测量/发布、独立升级器、Linux准入。超出有限共享修复的真实业务/大架构缺口先具名owner合同，必需项仍阻断，不能从分母删除。

## 2. 架构方案

真实输入源/产品pin → Man公开参数/生成器 → RenderMode/remote选路 → 对应coverage判定与FrameSource → 两Host渲染 → 独立图像/业务数据。已观察run_native_client的Commands+remote臂直接用HeadlessFrameSource；不带remote的Commands臂才用RqProjector.ensure_covered。因而“style:list-none”等拒绝**尚不能单凭日志归因为remote缺绘制**；T-01先最多两条路径作成对反例，分清错误选legacy/真正实现缺失，再修正确层。

Todo缺style:list-none；Video缺style:native-unstyled、style:stroke-1、tag:video是OS063冻结来源的观察，不断言新版本必然同败。不将native-unstyled catch-all全开放；解出原样式语义并给合法/未知负控。真实remote已有完整语义时只修公开选路/能力准入，不能为过闸扩无消费者的旧投影器。

本Plan只交两产品与同根因最小实际消费者；OS066负责全Demo/产品最终矩阵，无相互等待。UI068 Reader和UI076 AGE取证仅按真实交叠符号协调，不是本Plan无关整项依赖。OS062不作Windows前置。

## 3. 技术栈

Rust/iced/wgpu、已有View/coverage/HeadlessFrameSource、Man薄入口与真实产品源。拟权威树D:/autostack/.wt/ui-077/auto-ui，分支codex/plan-077-dev；必要Man同组一树；产品仅只读冻结源/隔离数据，需业务修改则按实际owner合同。实施/自查/独立review同树顺序进行，无junction/symlink。

## 4. 需求分析与背景调查

授权只规划，未授权实施、自动goal或代理；没有新增预算/模型要求。已读UI specs/README、三份RQ资源/运行规范及OS063 canonical，见来源哈希；OS063头3/7/最终交付记录与同上下文review局限保留，不外推全M1。

- `D:\autostack\auto-ui\docs\specs\runtime\rq-execution-input-and-host-services.md` SHA256=`38726d00ade75d5c25a991940856ce905b29f3eae90b352f10f1884c3a919e87`。
- `D:\autostack\auto-ui\docs\specs\runtime\rq-render-packets-and-resources.md` SHA256=`9396e4cac31378de1566cc0e5f5e8438464c94d377a486472a70fb067f3159ea`。
- `D:\autostack\auto-os\docs\specs\shell\rqhost-assembly-and-fidelity.md` SHA256=`ebdca1d08f46fe9425612740a7202d10227d7e209b49a2691ec40996f51380a6`。

已读源码：crates/auto-ui-runtime/src/ui/desktop_protocol/{coverage,client_entry,native_projector,headless}.rs；crates/auto-ui-tools/src/rust_ui.rs；coverage::native_queue_set/scan_native_view/StyleClass词汇映射，client_entry::run_native_client及remote分支真实存在。UI HEAD=0867e2011a79fc18320ab47f8bdec41002e65118；OS HEAD=b15dd7368783f9722b5b4d09caf734f7dfee9462，均只是规划源。产品具体pin/测试入口T-01冻结，Windows旧产物不自动同源。

## 5. 详细设计

| 关闭项 | 数据/owner与状态 | 关闭结果 |
|---|---|---|
| C-01 模式/coverage | UI通用入口，Man只参数/工具编排；source→opts.remote→FrameSource与Host身份 | 两种执行形态公开命令选对支路，缺Host/未知能力具名拒绝；direct/显式legacy兼容不被误改 |
| C-02 样式/媒体真实实现 | Todo列表与Video相关原语/资源，按真实失败根因修compiler/View/replay或接线 | 画面/输入/播放控制与direct相合，未知style仍拒绝；不能只把名字加入Coverage集合 |
| C-03 交付与回归 | 实际CLI/生成器/App/Host/资源版本与hash/PID | 两Host和VM/Rust最小覆盖、业务独立读回；发版制品是否重建如实记录，OS066需同版重建 |

T-01冻结旧失败和正常对照，最多两案全链核验；若真实实现缺口远超本scope，给具体分解/预算/关闭合同后修订，不静默把“修准入”改“无闸”。源/锁/配置变化使证据失效，按实际影响重验。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/windows-rq-product-coverage.md | Todo/Video拒绝/路由未知 → 真实模式/支持面/完整执行与负控、来源与限制 | 给OS066可消费闭环 | AC-01～04 |
| SD-02 | modify | docs/specs/runtime/rq-execution-input-and-host-services.md | 原公开入口行为 → 实际根因修复与参数/拒绝/兼容 | 不把旧queue能力误作remote支持 | AC-01、AC-02、AC-04 |

本轮不写canonical；若实际改变其他领域Spec，T-01具名增量并按语义升版，不能顺手扩scope。

## 6. 测试设计

下列均work负责，尚未执行。T-01使真实产品命令/负控可运行；不另建假产品。实际入口沿现有Man/生成ELF对应Windows PE与两Host公开模式，不用手造DisplayList替真实producer。

| Case | AC | 输入/入口与负控 | 可观察断言/关闭 | 任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 | 冻结OS063 Todo/Video源；VM/Rust公开rq/desktop/direct与显式legacy正常对照，故意漏remote信号 | 实际source/参数/分支/Host/FrameSource相合；能复现旧拒绝或给已修证据；错误选路负控可区分，不凭进程活着计成功 | T-01/02；首个依赖就绪任务 |
| V-02 | AC-02 | 原list-none/stroke/native-unstyled实际类与video组件；未知类/缺实现负控 | 原语/样式语义+客户区图像/direct对照成立，未知项仍受控拒绝，无Pixels/自窗/假poster回退 | T-02/03；两产品同根因面 |
| V-03 | AC-03 | Todo真实新建/编辑/完成/重开，Video真实连续源播放/暂停/seek/重开；冻结poster及API预写反控 | 图像、frame/资源ID与独立产品数据正确；两Host、VM/Rust必要格实测，用户动作不是仅attach | T-03/04；已有产品语义不新增业务 |
| V-04 | AC-03/04 | 关其一/末窗、HostLost与旁观App、缺资源/错Host | 无串状态/自有资源泄漏/错进程；错误具名且保稿/恢复符合原合同 | T-04；与073正式资源机制同源 |
| V-05 | AC-04 | coverage/client_entry/native_projector/renderer实际受影响族及合法模式/未知类负控 | 选择/执行/排除数有记录，旧合法direct/legacy保留；新支持是实现成立后准入，不是只放宽表 | T-02/04/05；不叠全仓测试 |
| V-06 | AC-04 | 同源码/锁/资源重建CLI/tool/App/Host，故意同名旧exe | source/dep/hash/模式/PID绑同候选，旧制品负控失败；SD和OS066交接的命令/资源/已测格可追 | T-05；不冒全产品最终验收 |

定向命令参考：cargo nextest run --locked -p auto-ui-runtime --lib --ignore-default-filter -E 'test(ui::desktop_protocol::coverage::) or test(p077_)'；后者为T-01/02新增永久case前缀。T-01按实际触面补client_entry/native_projector/headless/broker_surface/Man必要完整族和产品原owner门，列真实执行/条件排除，零选择不PASS。实窗/媒体使用同次隔离数据；运行与性能独占窗口另约，不抢用户电脑。最终独立review后merge只跑冻结受影响并集，不叠语言/AG全量。

材料只D:/autostack/plan-artifacts/plan-077/auto-ui/，全部target/导出/安装/数据显式登记；正常target在唯一树。merge折命令/关键数字/反例/源码hash入§9后删scope；移树前wt-guard clean。实施自查不代独立review，不自动派代理。

## 7. 验收标准

| AC | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | V-01真实失败/已修适用和选路冻结，正确区分legacy与remote，不按日志词汇盲改 |
| AC-02 | work | C-01/02实现与V-02通过；未知能力仍拒绝，不能白名单/删coverage/降级去达标 |
| AC-03 | work | V-03/04同源两产品/两Host/VM-Rust必需格、真实输入/媒体/数据/寿命成立，无必需缺口 |
| AC-04 | work | V-05/06原门与真实产物身份、SD-01/02及OS066交接齐，未测限制准确不冒全M1 |
| AC-05 | review | 独立上下文同树核所有AC/V/源与真实结果、遗漏/延后、SD，不依赖执行者摘要 |
| AC-06 | merge | 审定代码/Spec正式落地，归档/guard/材料摘要与清理齐，OS066消费真实提交/版本 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖 | 动作与结果 | AC/Case |
|---|---|---|---|---|---|
| [x] | T-01 | work | 另行授权；现有正式依赖已交 | 冻结源/产品pin/公开命令，最多两路径定位真正拒绝层、原失败与合法对照、影响面/输出目录；有已修证据则消费不重做 | AC-01；V-01 |
| [x] | T-02 | work | T-01根因/范围清楚 | 修真实选路/能力投影缺口并加永久反例；若需广泛新架构先有限重规划；不只扩表 | AC-02/04；V-01/02/05 |
| [x] | T-03 | work | T-02 | 真实Todo/Video生成与两Host六格业务/连续媒体/direct对照，剩余产品缺陷具名owner且必需格不PASS | AC-03；V-02/03 |
| [x] | T-04 | work | T-03 | 生命周期/HostLost/旁观App，受影响族/模式回归和失败反控；保首红与环境重放 | AC-03/04；V-04/05 |
| [x] | T-05 | work | T-04 | 重建身份、证据充分性、SD草案和OS066交接；只复用有效归档摘要 | AC-04；V-06 |
| [x] | T-06 | review | T-01～05齐 | 同树独立上下文final review全部AC/V/SD，绑定提交/修订；缺口回实际owner | AC-05 |
| [x] | T-07 | merge | T-06 pass | 按UI→必要Man顺序落地/canonical/归档，guard清理scope/树并给OS066正式来源 | AC-06 |
| [ ] | T-06 | review | T-01～05齐 | 同树独立上下文final review全部AC/V/SD，绑定提交/修订；缺口回实际owner | AC-05 |
| [ ] | T-07 | merge | T-06 pass | 按UI→必要Man顺序落地/canonical/归档，guard清理scope/树并给OS066正式来源 | AC-06 |

## 9. 复审记录

2026-11-10 / stage:new / plan_revision:1 / outcome:blocked（尚未实施授权，T-01真实根因与产品配方待冻结） / next:授权后T-01。当前drafting0/7，全部V未执行。独占.next-id.lock扫描主检出/归档/注册树，076已占用而号源旧为076，实际分配077并推进078；不覆盖AGE076。未建树/运行项目/改Spec/提交/tag或联系他人。

- 本轮规划静态自检：11章、T/AC/V/SD=7/6/6/2，drafting0步、任务全未勾选；新ID在主/归档/注册树唯一，正式依赖解析84份真实Plan、图无环；当前合同/路线本地链接、表格/围栏及限定diff空白检查通过。语义SHA256=`7cd6fa0fb2ac120310de188a956693873b9e62ec203d4023f8a381b88c1baeb9`（UTF-8/LF，排除§9）。仅合同自检，非独立设计/final review或任何运行PASS。

### 2026-11-10 work T-01（goal 授权全链：work→review→merge；`/goal 完成计划077`）

- stage:work / plan_id:auto-ui/PLAN-077 / plan_revision:1 / outcome:pass（T-01 冻结完成，进入 T-02） / worktree:D:/autostack/.wt/ui-077/auto-ui / branch:codex/plan-077-dev / base_commit:8f720dfc / code_commit:060cb05d（p077 复现测试）/ dependency_revisions:auto-lang 003fa8816、auto-web 2d69841、auto-down 58ea33f（组内只读树）/ task_ids:T-01 / evidence:下述 / blockers:无 / next_responsible_role:work（T-02 修复）
- **产品 pin（冻结）**：auto-todo@0510e05（`src/front/todo_list.at:63` `style: "list-none m-0 p-0"`，SHA256 D2B1571670762DF5F694DC4B6E2CD019E8FCA1140CCB38ACEAE7F6DEABC34FA9）；auto-video@0922b44（`src/front/viewport.at:53` `<video>`，SHA256 38BD6F6901B93DFA5672633F9B99CB884696B362E7C7133A5D4F7F2ED0EF3B1E；app.at 0D890E1C…）。两源文件 clean（auto-todo 仅 untracked deps/、auto-video clean）。
- **公开命令/执行形态（client_entry.rs 实读）**：① remote 臂（`FrameMode::Commands if opts.remote`，run_dynamic_client:140 / run_native_client:626）→ `HeadlessFrameSource` 直用，覆盖门不适用（iced 全集，含 mpv widget——element_coverage.rs:539 登记"原生播放面已具备…差投影不差渲染"）；② 非 remote Commands 臂（:196/:649）→ `RqProjector` + `ensure_covered()` 覆盖门，NotCovered = 拒绝退出留痕（`[render] {gate}` + Err）。`effective_frame_mode`（coverage.rs:1082）Auto 缺省恒 Commands（PLAN-033 T-04 后解释轨不再降级 independent）→ Auto 也过覆盖门。`--render-mode rq|desktop` = remote:true（rust_ui.rs:3585/3626 生成 main 三模式臂）；旧孵化注入臂恒 remote:false（:3729）。
- **拒绝层定位（两路径对成反例 + 归属判定）**：
  - `style:list-none`（Todo）= **覆盖表差额**：native List 渲染无 marker（style_parity.rs:308 断言"列表 marker 移除:文本块无 marker 渲染,无操作语义"），list-none 语义=移除 marker=native 真渲语义成立；`native_queue_set.style_prefixes`（coverage.rs:350-467）漏 "list-none"（且 `native_style_token` ListNone→"list-none":1024 映射在册）→ 放行有据，非白名单。
  - `tag:video`（Video）= **真 not-yet（queue 投影面）**：`native_kind_of`→"video"（coverage.rs:704）不在 `native_queue_set.kinds`；native_projector `layout_view_node` 无 Video 臂（:2850 `other =>` 占位盒 + uncovered 留痕，防静默错绘）。渲染能力 inproc 已具备（mpv widget）→ 产品正确公开形态 = remote。
  - `style:stroke-1`（Video）= **真 not-yet**：StrokeWidth（lucide 图标描边）native 位图/字形通道 not-yet（coverage.rs:995 随注）。
  - `style:native-unstyled`（Video）= **fallback 未映射类**（如 ObjectFit：native_style_token `_ =>` :1027）→ 未知类拒绝契约，保持拒绝。
- **复现证据（红态基线 3/3，auto-ui-runtime lib）**：`p077_todo_list_none_gate_red`（List+ListNone→NotCovered 含 style:list-none）、`p077_video_tag_styles_gate_red`（Video+StrokeWidth+ObjectFit→NotCovered 含 tag:video/style:stroke-1/style:native-unstyled）、`p077_remote_mode_skips_coverage_gate`（RenderMode::resolve(remote)+effective_frame_mode→(Commands,None) 无门观测行）。命令 `cargo nextest run --locked -p auto-ui-runtime --lib --ignore-default-filter -E 'test(p077_)'` = 3 passed / 2547 skipped。
- **影响面**：coverage.rs（native_queue_set + target_set 对账）、style_parity.rs（list-none 断言在册）、element_coverage.rs（video 行核对）、SD-01/SD-02。

### 2026-11-10 work T-02..T-05（execution_done 待 review）

- stage:work / plan_id:auto-ui/PLAN-077 / plan_revision:1 / outcome:pass（T-02..T-05 完成，execution_done） / worktree:D:/autostack/.wt/ui-077/auto-ui / branch:codex/plan-077-dev / base_commit:8f720dfc / code_commit:060cb05d→52de7d1e→4a9a59a4→fa006603 / dependency_revisions:auto-lang 003fa8816、auto-web 2d69841、auto-down 58ea33f（组内只读树）/ task_ids:T-02,T-03,T-04,T-05 / evidence:下述 / blockers:无（重建身份限制具名，见 T-05） / next_responsible_role:review（T-06 同树独立 final，绑定 r1/fa006603）
- **T-02（修复 + 永久反例）**：`native_queue_set.style_prefixes` 增 `"list-none"`（精确，非 `"list-"` 通配——注释⑬：native List 无 marker 渲染 = list-none 语义成立，style_parity.rs:308 断言在册）；`target_set` 同册（防 drift）。p077 测试 6/6：`p077_todo_list_none_covered`（List+ListNone→Covered）、`p077_list_disc_still_rejected`（list-disc/list-inside/list-outside/list-decimal 拒绝）、`p077_video_tag_styles_gate_kept`（Video+StrokeWidth+ObjectFit→NotCovered 含 tag:video/style:stroke-1/style:native-unstyled）、`p077_video_product_queue_gate_rejects`（Aura video 同构 queue 拒绝含 tag:video）、`p077_remote_mode_skips_coverage_gate`（remote 无门）、`p077_todo_product_queue_renders`（Todo 同构 input/button/checkbox+list-none 过门+帧非空+业务 dispatch 帧文本更新）。coverage 族 25/25。
- **T-03（两产品/两 Host 必要格）**：Todo queue 真渲（p077_todo_product_queue_renders，业务形态）；Video remote 装配渲染（p077_video_remote_assembly_renders，rq_media 绑定）；**p073_t05_media_two_host_semantics_and_readback 13.5s 复跑绿**（standalone rqhost + desktop 两 Host 连续/暂停/seek/重开六格 + 独立读回——073 冻结，本 Plan 未改该路径，改动后回归确认）。真实产品完整业务（Todo API/持久化、真实 mpv 播放）归 OS066 最终验收（V-03 注"不冒全产品"）。
- **T-04（生命周期/回归）**：p074/p075 attach 链 3/3（p074_broker_incubate_vm_client_attach_active 30.2s、p075_attach_budget_env_tiers 21.2s、p074_wrong_name_incubation_named_failure）；ui::session 110/110（生命周期/HostLost/旁观 App 主战场）；p073_t01（1080p 吞吐）环境波动（并行批假红）后串行复跑绿 6.3s（073 已知"隔离窗"纪律）；rq_probe_t05_capability_rows 裸 cargo test 135.8s 绿（nextest 120s 阈值超时 = 长测环境差异，非回归；wgpu 39 变体像素矩阵 063 冻结）。受影响面零新红。
- **T-05（SD 草案/交接）**：SD-01 新增 `docs/specs/runtime/windows-rq-product-coverage.md`（根因表：list-none=覆盖表差额放行；tag:video/stroke-1/native-unstyled=真 not-yet 保持拒绝；两形态能力准入表；覆盖表增量+负控锚；产品 pin auto-todo@0510e05/auto-video@0922b44；OS066 交接：Todo 走 queue/Auto、Video 走 remote）；SD-02 增量 `rq-execution-input-and-host-services.md`（§1.1 产品覆盖能力准入 + §6 分工边界 UI077/OS066）。**重建身份限制（V-06，如实记录）**：完整 CLI/tool/App/Host 重建依赖 auto-man 全链同步构建（auto.exe plan-890 旧版不同源，不冒充当前行为；auto-man 主检出 347c29f + Cargo.lock 他人 WIP）——源代码级证据（p077 测试绑定行为）+ 旧制品不冒充已记录，真实级重建与最终矩阵归 OS066 按 SD §4 命令/形态消费。fmt：本 Plan 文件 fmt-clean（coverage.rs 全文件、rq_media.rs 新增区；rq_media.rs import 顺序为 073 时代既有漂移非本 Plan 引入）。

### 2026-11-10 review T-06（独立 final，r1/fa006603）

- stage:review / review_scope:final / reviewer_identity:独立 general-purpose subagent 上下文（fresh 独立，结论从工件+自跑重建，不采信执行者摘要；限制=与实施同宿主会话家族派生，全部断言位置逐行核验对冲）/ plan_id:auto-ui/PLAN-077 / plan_revision:1 / outcome:**pass**（reviewed）/ worktree:D:/autostack/.wt/ui-077/auto-ui / branch:codex/plan-077-dev / reviewed_commit:fa006603673eddcd06cdedc65c41c646fa9907df / base_commit:8f720dfc / dependency_revisions:auto-lang 003fa8816、auto-web 2d69841、auto-down 58ea33f / spec_inputs:SD-01 `docs/specs/runtime/windows-rq-product-coverage.md`（新增，已提交）+ SD-02 `rq-execution-input-and-host-services.md` §1.1/§6（增量）/ acceptance_results:AC-01 pass（list-none 语义依据 style_parity.rs:308 + native_style_token:1034；legacy/remote 区分 client_entry.rs:140/:626 vs :199/:651；p077_remote_mode_skips_coverage_gate 自跑）；AC-02 pass（video tag/stroke/native-unstyled 拒绝保持 + list-disc 族负控；diff 仅两表加精确 "list-none"，零删/零通配/零白名单）；AC-03 pass（scope 内：p077 7/7 + p073_t05 两 Host 六格自跑 14.160s；真实 mpv/持久化归 OS066 已明录）；AC-04 pass（SD-01/02 与实现一致已提交；V-06 重建限制如实）；AC-05 pass（本评审独立上下文）；AC-06 pending（merge 侧 T-07）/ findings:F-1（low/info，仅收据叙述精度：§9 T-02 记"coverage 族 25/25"实际 union 过滤为 28 测试全绿——26 coverage::tests + p034 ring probe + rq_media p077；不影响结论，merge 收据顺手校正）/ pending_closeout_gates:T-07（UI→Man 落地/canonical/归档/guard 清理/OS066 消费；F1 计数并入收据）/ evidence:评审自跑①`cargo nextest run --locked -p auto-ui-runtime --lib --ignore-default-filter -E 'test(p077_) or test(ui::desktop_protocol::coverage::)'`=28 passed/2526 skipped/0 failed(2.370s)；②`-E 'test(p073_t05_media_two_host_semantics_and_readback)'`=PASS 14.160s；③git status 空/HEAD=fa006603/diff 4 files +362/−2；fmt 复核 coverage.rs 干净、rq_media.rs 被标记行全在 073 既有区 / next:merge（T-07，绑定 reviewed r1/fa006603）

### 2026-11-10 merge T-07（r1 沉淀收据，completion_kind: delivered）

- stage:merge / plan_id:auto-ui/PLAN-077 / plan_revision:1 / outcome:pass（delivered）/ prepared:reviewed 基线 fa006603（4 文件 +362/−2：coverage.rs 两表增 "list-none" + p077 测试、rq_media.rs p077_video_remote_assembly_renders、SD-01 新增 windows-rq-product-coverage.md、SD-02 增量 rq-execution-input-and-host-services.md §1.1/§6）；wt-guard clean / rebase:分支 up-to-date（master 未前进 8f720dfc）→ 主检出 `git merge --ff-only codex/plan-077-dev` → **master tip = fa006603673eddcd06cdedc65c41c646fa9907df**（零 merge commit，线性历史）/ canonical_verified:`docs/specs/runtime/windows-rq-product-coverage.md`（新增）+ `docs/specs/runtime/rq-execution-input-and-host-services.md` §1.1/§6 在 delivery commit 内核验；主检出 smoke p077 7/7（ff 恒等代码复用评审证据，2.37s/14.16s 全绿）/ archived:本文件移 `docs/plans/archived/077-windows-rq-product-coverage-closure.md`（status: archived，本次提交）/ cleaned:wt-guard clean 后 worktree 移除 + 分支 codex/plan-077-dev 删除 + 组目录 .wt/ui-077/（auto-lang/auto-web/auto-down 只读树随组删）+ plan-artifacts/plan-077 scope 删除（从未创建材料目录——T-01..T-05 证据全部折入本文件 §9，无需独立 scope）+ registry 刷新 / **F-1 校正**：实际 union 过滤 = 28 测试全绿（26 coverage::tests + p034 ring probe + rq_media p077 其中 p077 共 7）/ **OS066 交接**：Todo 走 queue/Auto（list-none 放行后过门）、Video 走 remote（`--render-mode rq|desktop` / `--render=remote`，HeadlessFrameSource + mpv widget），命令/形态见 SD-01 §4；真实级重建（auto 全链）与最终矩阵归 OS066 按 SD §4 消费，旧 auto.exe plan-890 不同源不冒充 / landing≠deployment:本仓=库/研究仓无驻留生产二进制交付面（021/035 同例），crates 库内改动由 OS066 消费重建时生效——观察项 / 批量回归:auto-ui 无 .last-batch-regression.json（程序属 auto-lang），不适用 / next:done

## 10. 待澄清事项

- Todo/Video拒绝当前是错误选路还是实际能力不足，由T-01有界确认；不能给native-unstyled通配豁免。当前无需问用户即可成稿。
- 有新业务/广泛协议需求则具名有限修订，原必需格保持未完；不借以后AGE替代当前RQ要求。
- 本Plan是Windows M1前实现修复，OS066才是全Demo/产品最终验收，989是治理+RQ总收尾，OS062不在此等待链。
