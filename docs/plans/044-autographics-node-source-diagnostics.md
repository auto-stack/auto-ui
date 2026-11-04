---
plan_id: PLAN-044
status: execution_done
feature_name: AutoGraphicsNode多模块源码定位与分层诊断
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 1
current_step: 6
total_steps: 10
depends_on: [auto-ui/PLAN-020, auto-ui/PLAN-030, auto-ui/PLAN-035, auto-ui/PLAN-040]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node-source-diagnostics.md]
touched_goals: [GOAL-008]
affects: [autographics-node2d-pilot, autographics-auto-driver]
---

# PLAN-044：Node多模块源码定位与分层诊断

## 0. 变更摘要

给Node生成链补显式可选源映射/分层诊断：导入/降层显示原模块/Node/事件；SDK有真实span的编译错误先经受检assembly region，再回到Node源。固定SDK运行错误无结构化span，必须只给可证明上下文或generated坐标，不能编造原行。

043模型模板能力独立，不做本Plan前置；不换SDK/改ACE，不要求AG016/017/018/019跟升。普通use/组件局部性保持，不能为定位合成单作者文件。[共同设计](../design/autographics/node-resource-components-and-source-diagnostics.md)。drafting/r1/0-10，仅规划。

## 1. 目标

1. 原模块/Node/handler/helper与生成用户源有可证明映射，定位精度明确exact/range/context/generated/unmapped；同名helper/同定义多实例/跨use不串文件或实例。
2. 前端、SDK compile、runtime/API fault、managed内部错误分层，保原stage/code/Run/来源与8KiB截断；有真实span才给精确作者位置。
3. 映射只解释已有诊断，不加执行trace/服务费/玩法；当前runtime无span保事实，不输出虚构调用栈/源行。
4. 可选debug消费绑定模块闭包/生成器/world/assembly身份与接受源快照；错/旧映射拒用并明确generated fallback，不污染游戏接受态或谎称mapped成功。

非目标：断点/单步调试器/变量检查/动态trace/HMR、跨仓语言AST或SDK/ACE改造、所有运行栈恢复/机器码映射、反向改源、旧内容批量升级或新增M2前置。

## 2. 架构方案

模块/已有AST span与发射origin→结构化generated-user-source映射旁件→loader受检绑定/接受源快照→原SourceSnapshot region校验→SDK offset或现有错误上下文→只在可证域映到作者文件/Node/事件。managed保持受信区域身份，runtime无span则context/generated/unmapped。

拟唯一树`D:/autostack/.wt/ui-044/auto-ui`、分支`codex/plan-044-dev`，当前不创建。同树不同上下文独立final顺序进行；043可独立实现，公共load/emit/main/identity/Storage短窗单写，按合入交叠选受影响验证。两Cargo图/SDK681不合并，target/材料scope独占，GPU重门单实例。

## 3. 技术栈

已核pilot（experimental/autographics-node2d-pilot）`src/load.rs`有module/line与function span；`translate.rs::LowerError`部分仅line/what，lower_components/3d重组model/msg/on成world.at。driver `src/authoring/behavior.rs::SourceSnapshot::{sdk_diagnostic,diagnostic}`已有SDK offset→user/managed region和8KiB限额，runtime明确`SDK supplied no mapped line`；`behavior_project.rs`按实际snapshot建Session。新能力不能声称当前全无诊断，也不能拿SDK没有的数据承诺精确栈。

拟新增pilot `src/source_map.rs`、`tests/node_source_map_gates.rs`、driver `tests/node_source_diagnostics.rs`、pilot `tools/verify_plan044.py`与故障模块夹具。显式CLI工作名`--diagnostics-map`、旁件`source-map.json`均T-02冻结提案，新target由T-03/04建立。现有stmt无可证span则降精度到handler范围/上下文，不在本Plan跨仓改AST/SDK。

## 4. 需求分析与背景调查

用户2026-11-04要求UI后继规划，允许合同/设计/导航及相关路线簿记，未授权实现/构建/运行/agent。观察UI `92ee2459a1058ddddc3fc3378ecec6585ffe726c`；040交付fc82061c且独立final有记录；AG016 executing3/10固定035，不新增其依赖。041/042为其他UI组件store/重载路线，独占重核后本批使用043/044。

授权（2026-11-04，/goal 完成计划044）：用户以 auto-plan-goal 授权 work→review→有界修复→merge 全链；树 `D:/autostack/.wt/ui-044/auto-ui`、分支 `codex/plan-044-dev`（2026-11-04 建于 master `d176f85f`），材料scope `D:/autostack/plan-artifacts/plan-044/auto-ui/`（owner/用途已登记）。不授权变更目标/验收/范围/预算；merge 前必须有绑定当前 revision+commit 的 final pass。主检出有他会话 WIP（043 Plan、045-047草案、docs/design、crates/auto-ui-runtime/test/ 等），非本 Plan 所有，不编辑不并入。

已读UI Specs index、`node3d-symbolic-component-bindings.md`(6ed0bf07…)、`node3d-audio-hud-composition.md`(d7272516…)、`node-editor-projects.md`(1daa7111…)及load/translate/behavior源码。语言仓`docs/specs/auto-graphics/behavior-api.md` SHA256 `02751f0e234633f82bb64d7103ca2656e3b8325182e5ef095f0c0eb8b2011dbf`、`behavior-project.md` SHA `9ed44de2cb0b454cfd440d6a9ff63e2b8470fb1adc5f870e643448bf7f2b02a4`要求region/身份伪造拒绝、真实compile span才映射、runtime无span明确无行。新旁件不得削弱这些边界。

SDK固定7b987ca8…/681内容，pilot当前语言另图。单个生成运行单元不是单作者文件要求；本Plan解决多模块来源解释，不新增“可审计性”产品目标或禁use。

## 5. 详细设计

### 5.1 支持精度与有界先证

T-02最多A/B两机制全链先证：真实作者源→lower→独立loader/assembly→SDK compile error/实际runtime fault→可观察诊断。首批包含020 components-2d、035 components-3d-audio-hud的普通use/局部model/msg/on/helper，040纯声明的lower错误也要定位，但它没有world.at，只覆盖前端原声明诊断、不伪造运行源map或world hash；生成源/assembly绑定规则只适用于真实有world的路径。未知profile显式不启用/无映射，不虚报全支持。

A优先结构化发射片段携带已有AST/function span与Node/handler origin；B仅在span不足时用可证明声明/helper范围并明确精度，不跨仓改AST/SDK。至少须证明跨模块helper真实compile span能回正确原文件/范围，重写handler片段能回正确Node/事件上下文。做不到则needs_replan，不能把所有错误统一贴context当完成。实际支持构造/错误输入/位置精度/CLI/schema/资源上界在T-02冻结，拟fixture不是已执行证据。

### 5.2 映射与接受源所有权

可选debug模式产旁件，默认CLI/旧档及world/API源字节保持；映射段含generated user半开byte区间、原module闭包key/SHA/可证span或声明范围、Node/事件/helper和精度。派发/绑定/循环等合成代码标synthetic/generated，不映到作者不存在的行。一原片段可映多个生成段，每段owner/身份清楚。UTF8 byte区间与展示字符列分开，列沿现有.chars()的Unicode scalar口径（不宣称屏幕字素宽），CRLF/LF/中文/组合字符独立重算；非字符边界offset不得强切。

T-02冻结schema/字段/区间排序不重叠/边界/相对路径规范。绑定实际generator、world hash/长度、各被用module hash/长度和assembly user region。SDK offset先判在user region，才算user offset；managed绝不当用户。无exact span输出range/context/generated；runtime/API fault仅保stage/callback/接受源与可证owner，不从对象ID/最后handler/错误文字猜唯一原行。多实例只在有实际实例身份时标具体实例，否则标同定义/候选上下文。

源内容来自受检closure/接受内存snapshot；Run期间磁盘同名源改变/移除不改错误指向，显示源行和实际执行hash一致。SaveAs/搬迁用相对来源，旧Run不能用新map。拒逃逸/绝对盘符/链接/未知module key，不读任意外部文件。原受信region/namespace/执行身份校验保持，map不是代码准入/权限证明。

### 5.3 坏映射、fallback及预算

缺失/坏hash/错world/module/generator、越界/重叠/旧Run或错误精度标识时拒用映射，明确unmapped/generated fallback；默认程序原准入/原stage/code/last-good不改变。显式“映射验证”工具必须非零失败，不静默写Mapped-ready；world/closure本身坏仍原loader拒绝，旁件不能修复/绕过。禁止默认运行插桩或逐调用trace改变VM步数/服务费。

元数据是实际CPU/包开销，非GPU资产或免费内存。T-02按user256/managed96/combined352KiB、8状态Node/128行及发射段数量推导有界线性存储/加载开销，测最大合法源和重复映射后冻结单位/超限分类；不暗加小语法上限删合法域，必要新硬额度或跨仓实现则needs_replan。无运行插桩时debug有无的world/API字节、VM步数/32credits/时限/帧/音画等价；map IO仅准备/加载，不每Tick读盘。正文仍8KiB截断、UTF8安全，map错误不能覆盖原错误。

### 5.4 保存与固定消费

map及其被用原模块/world身份随可选debug包/SaveAs明确携带；A→B只包内接受闭包仍复现相同位置/精度和原错误；坏map拒用→精确修复→下一次真实诊断恢复映射，不能只当前fallback成功。旧内容/当前AG016..019不升级；043将来发射段通过统一origin接口接入，不按Coin/Player/示例名造表。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node-source-diagnostics.md | 现有lower/SDK region诊断→可选Node原源映射/精度/身份/接受snapshot/坏map fallback及无span边界 | 多模块作者可定位错误，保原SDK/旧档/业务语义 | AC-01..06 |

## 6. 测试设计

T-03/04建立拟入口：`python experimental/autographics-node2d-pilot/tools/verify_plan044.py --controls`；pilot `cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --test node_source_map_gates`；driver `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml --test node_source_diagnostics -- --test-threads=1`。真实改parser/dialect才补对应target；不预设跨仓实现，非默认全AG/根全库，必要GPU正控单实例。

| case | AC | owner_stage | 实际/拟入口与区分oracle | 边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | gates/verifier：跨use同名helper/双实例、model/msg/Life；真实lower错误/SDK compile span指原module/Node/helper，独立重算原行/字符列；误报同一行另一文件的变异红 | exact/range/context按可证精度，不猜行 |
| V-S02 | AC-02/03 | work | driver真实assembly/SDK错误：user偏移/managed/合成派发/无span runtime及sticky API fault分层，后者明确no mapped line，假行负控红 | 不承诺当前SDK无span运行栈/断点 |
| V-S03 | AC-03/04 | work | map错world/module/generator hash、越界/重叠/路径逃逸/旧Run/伪精度实际变异拒用、mapped gate失败、原程序错误/fallback仍可读；坏world照旧拒绝 | map不能绕代码准入或恢复受污染VM |
| V-S04 | AC-01/04/05 | work | UTF8/组合字符/CRLF/半开首尾边界、注释变更旧map失效、新map下次读恢复；接受Run后磁盘源变更/删除仍显示当时hash源 | 不读任意外部文件/错snapshot |
| V-S05 | AC-04/05 | work | 最大源/重复段线性存储实测、8KiB截断；debug有无world/API字节、VM步数/费用/帧/音画等价，无每Tick map IO | 上界T-02冻结，不能声明零代价 |
| V-S06 | AC-05/06 | work | 可选debug包A→B冷再生/SaveAs/接受来源迁移，坏map fallback→修复→下次真实错误恢复；原020/030/035/040受影响冻结/诊断门 | 当前内容锁不跟升，043非前置 |

work产证；review同树独立上下文final，从真实compile/Run复核AC/SD/精度及遗漏延期workaround/告警，绑定r1/提交/工具/原输入，finding回work同员复核。无授权不建agent/第二树，同会话自查不代独立final。merge冻审后定向并集，有据复用未变证据。

## 7. 验收标准

| AC | owner_stage | 关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 普通多模块原源定位正确，至少helper真实compile span与Node事件上下文成立、精度明示 | V-S01/04 |
| AC-02 | work | 前端/SDK user/managed/合成/runtime/API分层，原stage/code/来源及8KiB保持 | V-S01/02 |
| AC-03 | work | 无span不编造行/实例，错map拒用/映射验证失败且原运行错误/generated fallback不变 | V-S02/03 |
| AC-04 | work | map/原源/world/assembly/接受Run身份互绑，UTF8/边界/磁盘变化/旧Run/修复下次读取正确 | V-S03/04/06 |
| AC-05 | work | metadata有界/载入与原源VM/费用帧音画无插桩漂移，冷再生/保存迁移及旧触面保持 | V-S05/06 |
| AC-06 | work | SD候选/准确可选CLI-schema/精度矩阵/无span边界与未来origin接入合同齐 | V-S06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/真实源位置与变异，finding闭环pass | T-07 |
| AC-08 | merge | 冻结门/来源核后落主，canonical/归档/可选消费身份及精度限制齐 | T-08/09 |
| AC-09 | merge | fresh guard清树分支空组、全输出对账/摘要折Plan/删scope/登记hold | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 用户另行启动；020/030/035/040交付 | 核已有AST位置/emit/region/SDK无span事实、profile/命名门/owner，唯一树scope登记 | AC-01/02 | [x] 2026-11-04：已核 load.rs `NodeDecl.span`/fn `Range<usize>` byte区间、`Project.node_modules`、uses闭包解析；translate.rs `LowerError::Unsupported{what,line}` 仅行号（部分路径line:0）无模块/字节span；main.rs 19 profile门（legacy..components-2d-runtime）与 identity.json（generator/api/user_source_sha256/module:sha/closure/world_sha256/provenance）已核；driver behavior.rs regions[transport/api/user/bridge]、`sdk_diagnostic` offset→region→line:col(chars口径)、runtime `diagnostic` 明确"SDK supplied no mapped line"、8KiB `bounded_diagnostic`、`error_offset` span变体枚举、admit用户parse行列映射；behavior_project.rs CodeRef/Pool(8/2MiB)/load_declared hash核验。树/分支/scope已建并登记（§4授权记录）。下一步T-02真实先证。 |
| T-02 | work | T-01 | 最多A/B真实模块全链先证，冻结profile/精度/schema/CLI/线性上界/fallback；不足needs_replan | AC-01..06 | [ ] |
| T-03 | work | T-02 go | 可选origin/旁件/loader接受身份/snapshot分层展示/gates，默认字节保持 | AC-01..04 | [x] 2026-11-04：origin=pooled/helper+Life handler+Member recv 唯一属主段；旁件+CLI 冻结；loader 接受身份接线 `SourceMap::load_for_world`（world+map+closure 三样受检物，缺/坏一律 None 走默认路径）；snapshot 分层展示=sdk_diagnostic_mapped 双锚（span 锚+符号锚）+runtime 无 span/managed 永不映射门；默认字节保持=verify_plan044 控制一（真实进程）+建图纯函数门。pilot gates 12/12、driver 7/7、verify 5 控制全绿。 |
| T-04 | work | T-03 | 真实compile/runtime/managed/无span与身份/UTF8/变异，非伪错误文本日志 | AC-01..05 | [x] 2026-11-04：真实 compile（符号锚 broken.at:8:16）、runtime 无 span/managed 永不映射/generated 永不映射、假行文本负控（错误文本内嵌行号不信任）、符号多出现不猜、UTF8+组合符+CRLF 列/行独立重算、变异拒用全家、原错误文本保持。pilot 13/13、driver 8/8、verify 5 控制、回归全绿。 |
| T-05 | work | T-04 | 资源上界/字节步数等价、保存A→B/坏map下次恢复与受影响旧门 | AC-04/05 | [x] 2026-11-04：上界实测重冻结（max(64KiB,4×world)+4MiB+16384 段，worst ratio 3.42）；8KiB 截断 mapped 路径门；debug 有无 world/API 字节等价（verify 控制一）+ VM 步数等价结构性事实（map 只在诊断回调消费、不入 Session、无每 Tick IO）；A→B 搬迁等价消费 + 坏图拒用→修复→下次诊断恢复映射；受影响旧门全绿（components 6/6、3d-audio-hud 26、3d-basic 8、game 3）。 |
| T-06 | work | T-05 | SD候选/可选消费接口/覆盖精度及限制交接，不加M2前置 | AC-05/06 | [x] 2026-11-04：SD-01 候选 `docs/specs/auto-graphics/node-source-diagnostics.md`（worktree 8dc830d1；范围/精度模型/旁件绑定/双锚消费/分层/SDK681 span 冻结事实/预算等价/入口/§8 origin 接入合同）；M2/ACE 触面零（diff 全量核 0 文件）。 |
| T-07 | review | T-06 | 同树独立final与finding修复后同员复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结定向门/来源核后实际落master | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/归档/导航与可选固定身份核验 | AC-08 | [ ] |
| T-10 | merge | T-09 | fresh guard/全输出对账/删scope构建物/摘要hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-04 / stage: new / plan_id: auto-ui/PLAN-044 / plan_revision: 1 / outcome: pass（合同可审阅，用户启动后T-01可进，T-03需T-02真实先证go） / next: work T-01。0/10。已有SDK compile region/runtime无span事实从源码核，不承诺所有运行错误精确映射；旁件/入口/模块fixture未实现。本轮无树/材料scope/源码/探针/canonical Spec/构建/GPU/agent，不改在途内容/SDK。

2026-11-04本批规划预检（非实施/独立final）：043/044均0..10节、drafting/r1/0-10、10T/9AC/6V-S/1SD，work6/review1/merge3，十任务未勾；依赖DAG无环，二者功能独立，当前AG016..019没有新增043/044依赖。首次取号核到041/042已属组件store/重载路线后，独占重核取得043/044，未覆盖；后续他路线又分配045..047，本轮不回退号源，不提交其草案，下一号以中央.next-id实际独占核为准。262个所涉本地引用258可解析，余4为存量非AG主题036/037/038导航断链（共享索引包含他会话WIP），未代写；新增/修改AG引用可解析。按metadata/generated/Node源交叉核验当前19/22，余g04两维/g05三维；`python tools/gallery.py validate --all`退出0，12逻辑项22入口/许可一致。Model底层支持、Node模板builtin-only及SDK compile region/runtime无span事实从源码/Spec核，不代表新模板/新映射先证已跑。UI040实交付/独立final与D-02 OS15项pending分列；AG016在途3/10固定035，内容合同/源码/锁/生成物均未改，本轮没有创建实现树/材料scope/probe/新canonical Spec、构建/GPU/OS自动化/agent/发布。文档白名单空白与结构核验通过。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: d176f85f (base, 尚无代码提交) | next: work T-02。T-01 完成：AST/span/emit/region/SDK无span事实与 profile/命名门/owner 已从源码核实并登记（§8 T-01 行内证据）；授权记录写入 §4；主检出他会话 WIP（043 Plan、045-047 草案、docs/design、crates/auto-ui-runtime/test/）非本 Plan 所有，仅登记不触碰。T-02 起为真实全链先证（A/B 机制、冻结 schema/CLI/上界）。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 70c9075b | next: work T-02（续）。依赖树 `D:/autostack/.wt/ui-044/auto-lang`（detached @ da884ab2b，只读兄弟检出，pilot `../../../auto-lang` 路径解析）。本轮切片：pilot `src/source_map.rs`（MAP_FORMAT=node-source-map/v1；Precision exact/range/context/generated/unmapped；Segment 半开byte区间+module/node/event/helper+源区间；validate 拒坏hash/逃逸key/越界/重叠乱序/非字符边界/伪精度字段一致；lookup 二分；validate_against_world 载入侧 world/user 字节绑定）+ pooled helper 后置锚定（`// pooled fn {name} from {module}` 唯一锚定，verbatim→exact，行内重命名→range）+ generated 间隙补齐铺瓦。未改默认发射字节。gates `tests/node_source_map_gates.rs` 8/8 绿（`cargo test --test node_source_map_gates`）。T-02 余量：真实 profile 全链（load→lower→map→SDK compile 错 offset→回原模块行）、handler 段（node/event 上下文）、`--diagnostics-map`/`source-map.json` 冻结、driver 侧消费与分层诊断、线性上界实测。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 52fbfe40 | next: work T-02（续）。关键实证三则：（1）发射器重命名语义核定——pooled helper 声明与调用的 `{name}(` 恒被改写为 `{module}_{name}(`（rename_module_calls，prev_ok 防链式），故 pooled 段冻结为 range 精度，列经 `remap_range_offset` 重放同一改名算法双指针差分（只接受 `{module}_` 纯插入）回算，实证 helpers 夹具 helpera/helperb 同名 bump 各自回正确模块+1.0/+2.0 不互串（V-S01 映射侧）。（2）load.rs 既有缺陷修正：AST fn span 为 (offset,len) 被误存 start..len（区间倒序；既有消费方只用名字，零触面），修为 start..start+len（52fbfe40）。（3）AST declaration_start 与 fn 文本起点实证差 1（parser 关键字口径），映射源区间改锚 slice_fn_source 可证切片（与发射器同源逐字节）。gates 8/8 绿 + 受影响回归 node_components_gates 6/6 绿。T-02 余量：SDK compile 真实 span 全链（driver 侧）、handler node/event 段、`--diagnostics-map`/`source-map.json` CLI 冻结、线性上界实测。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: a1abfd8f | next: work T-02（续：handler 段 + 上界实测后关闭）。**重大冻结发现（SDK681 span 语义）**：undefined-symbol 是唯一易达且带 span 的作者可修 compile 错误类；实证其 span = "call 表达式末尾 + 前置 // 注释行数"口径（探针矩阵：无注释恒 +12，每注释行 +1，与 token 位置/嵌套/函数序无关；±1 级），**不可字节回算**（miette 索引 <script>=combined source，但 Pos 来自 parser 内部漂移）。type/arity/field 错在 SDK681 编译期放行（惰性）。设计裁定（保 AC 不缩水）：driver 映射消费走**双锚受检**——①span 锚：offset 入 range/exact 段 + 重放校验 + 消息符号与 world 该处逐字节一致；②符号锚恢复：span 锚失败且消息含符号时，全 range 段内标识符边界搜唯一出现，唯一才可证映射。junk span 永不误映（防 766→helpera 误报实证）。**T-02 全链阳性先证成立**：diag-fault 夹具（broken.at `absent_helper` 未定义）→ pilot `--diagnostics-map`（T-02 冻结工作名，components-2d/queries-2d 路径，默认无旁件字节不变）→ 冻结 generated-components2d/diag-fault/ → driver SourceSnapshot 受检装配 → SDK compile 真实错误 → 双锚映射 **broken.at:8:16 Range helper=speedup**（列经重命名差分精确回算）→ 不误入 helpera/helperb；坏 world hash/旧 world/缺图/坏闭包全部拒用且回退原诊断（4/4 driver gates 绿，pilot gates 8/8 + components 6/6 + behavior_source_identity 1/1 回归绿）。`.sdk-source` 按 ui-043 惯例自主检出字节复制（15MB，gitignored 物化）。余量：handler node/event 上下文段（T-02 第二半）、线性上界实测（T-05）、verify_plan044.py（T-06 入口）。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 0da688fb | next: work T-03。**T-02 关闭（2/10）**：handler context 段落地——发射名 `{snake(node)}_life{event}`/`{snake(node)}_{snake(member)}` 锚定 world 中 handler fn，精度 context（重写体不承诺语句级位置），node/event 身份 + 声明行锚；**源区间纯文本锚定**（node 声明区花括号配平内搜 `Life.{event}`/`.{member}` token）——实证本栈 AST span 起点 +1(fn)/+2(handler) 漂移，凡 AST span 均不可字节信，映射一律文本锚。helpers 夹具 Alpha/Beta 段不互串 gate；driver 冻结旁件上 gamma_lifetick context 起点实证（node=Gamma/event=Life.Tick/锚定行内容含 Life.Tick）。pilot gates 9/9、driver 5/5、components 6/6、behavior_source_identity 1/1 全绿（commit 516b152a 标记 T-02 [x]）。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: a90eebb2 | next: work T-03（续）。T-03 第一切片：①Member/recv 段落地——recv fn 名含子实例绑定名（`meter_add`）非 def 名，后置不可证重建，裁定**唯一属主文本规则**（未覆盖顶层 fn 的 `_{suffix}` 与全工程唯一 Member 事件 snake 匹配且恰好一个 node 属主才归属；多属主/无属主不映；未绑定 handler 无发射 fn 即跳过）。修一扫描器 off-by-one（`start+3`）。②默认字节保持单元门（建图纯函数 world 不变；CLI 对照归 verify_plan044.py）。③driver 分层门（V-S02）：runtime 无 span（AutoError::Msg）映射层不编造行、保持 "SDK supplied no mapped line"；managed 区 span 永不映射（managed 绝不当用户）；dev-dep miette=7.6 与锁定树同版（lock 已含零新下载）。pilot gates 11/11、driver 6/6、回归全绿。T-03 余量：loader 接受身份接线（SaveAs/接受源快照消费面，依赖 behavior_project/Storage 路径）、verify_plan044.py 骨架、UTF8/边界独立重算门（V-S04 映射侧）。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: ec5c8e6e | next: work T-03（续）。①`tools/verify_plan044.py --controls` 落地并全绿（ec5c8e6e）：真实进程默认 CLI 无旁件 + world 冻结 sha、--diagnostics-map 旁件绑定（format/world+user sha/闭包 sha/段铺瓦/精度域）、再生字节一致（identity.json 按 provenance.product 归一——该字段随 out 目录名变化，属事实非漂移）、检入产品现行性、driver gates 调用（5 控制全过）。②V-S04 映射侧门：中文+组合符（e\u{0301}）helper 源的列按 Unicode scalar 独立重算实证，段边界字符边界由整图 validate 保证。pilot gates 12/12 绿。T-03 余量：loader 接受身份接线（SaveAs/接受源快照消费面）、map 错误不覆盖原错误门（部分已在 driver gates）、T-04 真实 compile/runtime 分层扩展。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: ba2134f8 | next: work T-04。**T-03 关闭（3/10）**：loader 接受身份接线 `SourceMap::load_for_world(world_path)` 落地——目录内 world.at + source-map.json + closure/ 三样受检物全绑定才 Some，缺旁件/坏图/坏闭包/旧世界字节一律 None（默认诊断路径不变，不改准入）；driver 门 4 态（好包 Some、缺 None、坏闭包 None、漂移 None）。verify_plan044 全控制复跑绿。driver gates 7/7 + pilot 12/12 + components 6/6 + behavior_source_identity 1/1。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 1eb965c8 | next: work T-05。**T-04 关闭（4/10）**：补三负控/边界门——CRLF helper 源逐字节 remap（发射锚注释恒 \n 约定实证）、错误文本内嵌伪造行号永不信任（映射只认受检 span/符号）、符号多出现不猜（bump 跨 helpera/helperb 双段拒绝，唯一符号对照成立）。T-04 全项覆盖：真实 compile/runtime/managed/无 span 分层、身份绑定、UTF8/组合符/CRLF、变异拒用全家、非伪错误文本。pilot 13/13、driver 8/8、verify 5 控制、components 6/6、behavior_source_identity 1/1 全绿。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 6f3e5922 | next: work T-05（续）。**T-05 实测重冻结（D-02 关闭）**：旁件体积实测 n=32/128/256 helper × pad 6/12/30——map/world ratio 3.42/3.33/1.08/2.22，段数=2n+1，json 增量随 n 稳定（非超线性）。旧 T-02 常数上限 64KiB/4096 段会暗删 256KiB 合法域（worst 映射 ~870KiB），重冻结：driver 载入界 = max(64KiB, 4×world) 比率界 + 4MiB 绝对界 + 16384 段界（6f3e5922 双侧同步）；超限分类=整图拒用→generated fallback，不截断。pilot gates 14/14、driver 8/8、verify 5 控制复跑绿。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 57c7f264 | next: work T-06。**T-05 关闭（5/10）**：①8KiB 截断 mapped 路径门（20KiB 错误消息 → broken.at 前缀保持 + 截断标记 + <9KiB）。②V-S06：坏图拒用→精确修复→下次诊断恢复映射（同包全链路）；A→B 搬迁同字节等价消费（来源无关，相对包结构）。③debug 有无等价：world/API 字节等价=verify 控制一实测；VM 步数/费用/帧等价=结构事实（map 仅在 SDK compile 诊断回调消费，不入 Session 状态、零运行期钩子、无每 Tick IO——sdk_diagnostic_mapped 不改变 VM 路径）；音画等价=无呈现触面改动。④受影响旧门：load.rs 触面 3D 回归（audio-hud 26/26、3d-basic 8/8）+ game 3/3 + components 6/6 全绿。driver gates 10/10、verify 5 控制复跑绿。

2026-11-04 / stage: goal | plan_id: PLAN-044 | plan_revision: 1 | round: 12 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/ui-044/auto-ui | commit: 8dc830d1 | next: review。**work 阶段完成（6/6）**：SD-01 候选 spec 落地（精度模型/旁件绑定/双锚消费/SDK681 span 冻结事实/预算等价/§8 origin 接入合同，worktree 8dc830d1）；M2/ACE 触面零。work 交接收据见 §9 stage: work 行（code_commit 8dc830d1）。下一阶段 review：同树独立上下文 final（T-07），逐 AC/SD/真实源位置与变异复核，finding 闭环后同员复核。

2026-11-04 / stage: work | plan_id: PLAN-044 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/ui-044/auto-ui | branch: codex/plan-044-dev | base_commit: d176f85f | code_commit: 8dc830d1 | dependency_revisions: auto-lang 兄弟检出 detached da884ab2b（只读，.sdk-source 自物化） | task_ids: T-01..T-06 | evidence: pilot gates 14/14（node_source_map_gates）、driver gates 10/10（node_source_diagnostics --test-threads=1）、verify_plan044.py --controls 5/5、受影响回归 components 6/6 + 3d-audio-hud 26/26 + 3d-basic 8/8 + game 3/3 + behavior_source_identity 1/1 | blockers: 无 | next_responsible_role: review（同树独立上下文 final；统一回归/里程碑档归 review/merge 并集）。工作侧 AC 结果：AC-01 定位正确+精度明示（helper 符号锚 broken.at:8:16 全链 + handler context 不互串门）；AC-02 分层保原 stage/8KiB（compile/runtime/managed/generated 门）；AC-03 无 span 不编造/错 map 拒用/fallback 不覆盖原错误（负控全家）；AC-04 互绑/UTF8/CRLF/磁盘变化/旧 Run/修复恢复；AC-05 有界重冻结/等价/A→B/旧门绿；AC-06 SD/CLI/schema/精度矩阵/origin 合同齐。未决：无 in-contract gap；统一回归显式 pending（owner=review/merge）。

## 10. 待澄清事项

| ID | 事项 | owner/关闭动作 |
|---|---|---|
| D-01 | AST位置/emit重组可证精度及准确profile覆盖 | T-02真实错误链先证，不一律伪精确或一律context充目标 |
| D-02 | map线性上界/schema/fallback实际接入 | T-02按原源域冻结；新硬额度或跨仓AST/SDK改造需needs_replan |
| D-03 | runtime/API无span与managed区域 | 原SDK事实保持，仅可证上下文，无唯一位置明确unmapped，不暗加运行插桩 |
| D-05 | SDK681 undefined-symbol span 为 call 末尾+注释行数口径，不可字节回算 | T-02 已裁定双锚受检映射（span 锚+符号锚恢复），junk span 永不误映；其它错误类无 span 走原回退。换 SDK 前该口径须重测（round-4 §9 实证） |
| D-04 | 043公共发射交叠/M2固定消费 | T-01核owner短窗；043非前置，当前内容/AG019候选规则不变 |

开工材料唯一`D:/autostack/plan-artifacts/plan-044/auto-ui/`，登记owner/用途/在途；临时probe/包/原始诊断/外部消费者/外置target全归scope。merge摘要折Plan后全输出对账/删scope构建物，仅具名未解决问题持有owner/理由/关闭条件。禁attachments/第二暂存区/链接，清树先fresh guard，不动其它同号仓scope和在途材料。
