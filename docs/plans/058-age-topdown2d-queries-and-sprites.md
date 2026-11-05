---
plan_id: PLAN-058
status: drafting
feature_name: AGE M3俯视二维控制与查询精灵组合
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [auto-ui/PLAN-055, ag-examples/PLAN-022, ag-examples/PLAN-021]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-graphics/node2d-topdown-composition.md]
touched_goals: [GOAL-008]
affects: [node2d-authoring, query-sprite-composition, autographics-auto-driver]
---

# PLAN-058：俯视二维控制、查询与精灵公共组合

## 0. 变更摘要

承接055真实研究与AG022需求输入，交付有限、显式版本的正常Auto俯视二维组合作者链：无重力四/八向控制、单次sweep停墙、最近交互查询、具名精灵帧与普通use组件局部事件。不是完整种田游戏。本轮只规划，正式实现先核研究结果和M2收口，不预判055会go。见[本批路线](../design/autographics/m3-next-wave-execution-order.md)。

## 1. 目标

普通Node/use/model/msg/on/Life.Tick承载业务；一Player、两静态障碍、两个同定义交互物构成最小完整链。非零输入轴归一，斜走不快于直走；一个Tick最多一次sweep并停于首次接触，不要求自动滑墙。查询/世界位置与当Tick精灵位置一致，两个交互物的状态/事件各自独立；最近目标、同距顺序、超距不触发由公共查询结果＋Auto逻辑表达。

不包含tilemap批绘、任意大图、多角色/动态碰撞、完整背包种植存档、深度排序/2DHD光照、混合UI、寻路/自动滑墙、ACE/AOT/跨端/预算提升。后继完整游戏规则仍等AG022，不将本例改写M2平台游戏或Gallery分母。

## 2. 架构方案

显式新profile（名称/版本T-02冻结）复用020局部组件、021 segment/sweep、022 PNG/帧表和固定SDK681；打通同一状态owner下查询、物件移动与精灵呈现，不把不同旧profile拼贴后假称通用支持。Auto计算方向/规则，Rust仅提供通用服务和受检装配，不写专用种田业务。

拟唯一树`D:/autostack/.wt/ui-058/auto-ui`，分支`codex/plan-058-dev`。同组依赖源按实际交付只读，无链接；共享AST/load/translate/API/host写面先核054与其它owner短窗，报告/target/scope独占。本轮不建树。

## 3. 技术栈

已核pilot `src/{load,translate,lower_components2d_runtime}.rs`，driver `src/authoring/runtime_sprites.rs`和既有查询/精灵服务；现有CLI `node2d-lower --profile queries-2d|sprites-2d|components-2d-runtime --entry … --out …`分别有不同支持域，不能当现成组合入口。保持两个独立Cargo图/固定SDK681，不引入新解释器。

## 4. 需求分析与背景调查

用户2026-11-05授权下一批计划及全部状态/顺序；本轮只设计/计划/导航，不执行。当前auto-ui主观察64aef780（其它UI052已推进，不把其实现归AGE）；055/AG022未交付，054已有唯一树与实现提交1f3be333但未交付，AG021 T-03等待054。没有俯视组合canonical，新知识由SD-01在review/merge沉淀，当前不改已交付Specs。

已读Specs `node2d-queries.md`（SHA前16 0d930490f45d9a12）、`node2d-sprites-and-frame-animation.md`（f997bae66305eef0）、`node2d-full-game-composition.md`（e693338b0a88a7e6）。021明确不含sprite/自动slide，query世界不纳spawned slot；022帧写依有限render槽；032要求一角色/相机及固定state/pose字段，不能换重力/业务字段就称通用俯视。源码lower_components2d_runtime SHA6eed9a6c6ac447ae、translate9350491665faf8a0为静态观察。本Plan不假设这些约束已消除。

## 5. 详细设计

T-01可只读准备支持差距/作者草图/独立oracle；T-02完整先证硬等055实际研究、AG022实际需求/容量结果以及AG021对M2-01..10全部必需门的真实收口结论。AG020更名不作前置；实际更名后只解析同仓新根/历史身份，不另取号。

T-02至多两种机制：优先统一静态场景行查询/位置与sprite表现；若旧slot限制必须分离碰撞代理与外观，必须证明一个逻辑owner、同Tick共同接受、没有重复Tick或错位/隐藏阻挡。冻结普通具名资源/组件事件、输入轴/pressed语义、世界单位/dt、无输入/同时反向输入、归一/起始重叠/最近命中/同距顺序、相机和资源身份。具体API不由本合同凭空命名；先证失败需needs_replan，不把raw API成功充正常Node链go。

最低验收就是1Player＋2障碍＋2同定义交互物，根及局部状态总数≤8、场景≤128行、原每世界callback32credits/物理写32/1000步每slice/原source与资产包络保持。地图仅小范围，不把055三规模都当必须接受；scale以055/AG022实际结果冻结且不得缩掉以上最低对象/行为。完整合法峰值同Tick包含移动、必要query/事件和精灵收口；steps/credits/ms/GPU字节分开计，不以隔帧查目标/停帧写图或复制池省预算。若最小链需提高限额或新的计费策略，T-02列原账/候选账并needs_replan，不越权放宽。

新profile必须版本隔离：旧021/022/032的schema/产物与价格保持，旧M2项目固定锁不自动升级。独立f32轨迹/segment最近命中oracle与GPU位置/帧/像素共同验证；按键四向/斜向/停走/反向同时按、墙/墙角/起始overlap、两目标同距/超距、目标事件只更新自身、改Auto速度/事件后实际变化。暂停/Step/freshRun与旧句柄/坏源/资产恢复下一Run闭环。

T-03..06拟新增pilot `tests/topdown2d_composition_gates.rs`、driver `tests/node_topdown2d_{flow,storage}.rs`与`tools/verify_plan058.py --controls`（新路径，创建后冻结真实命令）；真实窗口输入层和受控fixture层分别记录。T-06给AG023准确交付版本/语法/运行与冷包配方。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/node2d-topdown-composition.md | 有限query/sprite各档→显式俯视组合支持/一次sweep/同Tick表现和原预算边界；旧档保持 | 正常Auto作者与分仓可验证交付 | AC-01..06 |

## 6. 测试设计

以下均为未来work验证合同，本轮未执行。实施者先自检；同一权威树由独立上下文final复审，不能用同实现会话重跑充独立。只跑真实受影响case/必要消费门，GPU/窗口焦点/音频重门按机器单实例；不为merge重复全量AGE，不改根语言时不跑根全仓。新路径/入口由任务建立后才可执行。

| case | AC | owner_stage | 输入/入口与可区分预期 | 覆盖边界 |
|---|---|---|---|---|
| V-S01 | AC-01/02 | work | T-02全链＋未来gates/flow：正常use模块、轴(1,0)/(1,1)同速、零/反向输入、改Auto速度真实变，比较f32轨迹/GPU | 不能由平台重力或raw接口代作者支持 |
| V-S02 | AC-02/03 | work | 一次sweep首次停墙/墙角/起始overlap；scene位置与精灵像素同Tick，最近和同距target顺序独立oracle | 不要求自动slide，不改旧query容差 |
| V-S03 | AC-03/04 | work | 两同定义目标A两次/B一次各状态2/1、超距不触发；合法峰值≤32credits和第33拒绝 | 四/八向与交互不隔帧；各单位独立 |
| V-S04 | AC-04/05 | work | Pause/Step/20轮Run，旧句柄/坏模块/资产拒绝→精确修复→相同公开入口下次真Run | owner/上传/残留对象有收支证据 |
| V-S05 | AC-05/06 | work | verify_plan058/storage冷A→B无producer根/target/网络，改源旧Ready失效，旧021/022/032身份不变 | 不改M2消费者锁 |
| V-S06 | AC-01/06 | work | 055实际结论/AG022需求/M2门→支持与限制/SD/AG023配方逐项对照 | 未证不冒go，缺研究结论不进入正式实现 |

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 | 验证 |
|---|---|---|---|
| AC-01 | work | 研究/需求/M2前置真实闭合，完整先证和版本化支持表可审 | V-S01/06 |
| AC-02 | work | 归一无重力控制/一次sweep停墙/数值与普通Node真执行成立 | V-S01/02 |
| AC-03 | work | query位置/精灵同Tick一致，最近目标和双实例事件隔离正确 | V-S02/03 |
| AC-04 | work | 原预算峰值/owner/Run/超界和错误恢复真实成立 | V-S03/04 |
| AC-05 | work | 冷消费/坏源修复下一Run及旧profile/锁兼容齐 | V-S04/05 |
| AC-06 | work | SD/独立oracle与精确AG023消费入口齐，无完整种田能力虚称 | V-S05/06 |
| AC-07 | review | 同树独立上下文final逐AC/SD/实际来源与代表性反例pass，finding返work同员有界复核 | T-07 |
| AC-08 | merge | 冻结触面门/来源核后实际落主、canonical及同号归档/导航，结论和实际支持域一致 | T-08/09 |
| AC-09 | merge | 全实际输出对账/必要源命令结果摘要折Plan/fresh guard清自有树与scope构建物，具名hold登记 | T-10 |

## 8. 执行步骤

| T | owner_stage | 依赖 | 写面/可验证结果 | AC | 进度 |
|---|---|---|---|---|---|
| T-01 | work | 后续另行启动；T-01不等研究 | 只读差距/源版本/最低样本和oracle准备，核owner/唯一树scope | AC-01/06 | [ ] |
| T-02 | work | T-01；055/AG022实交付及M2必需门收口 | ≤2机制完整最小链先证，冻结profile/端口/预算/数值/共同发布；失败replan | AC-01..04 | [ ] |
| T-03 | work | T-02 go | 显式组合profile/lower/受检服务，正常use/事件和具名sprite引用 | AC-01..03/05 | [ ] |
| T-04 | work | T-03 | 四八向/查询与呈现同步、实例/owner/故障及必要输入门 | AC-02..05 | [ ] |
| T-05 | work | T-04 | gates/flow/预算合法超界与独立数值oracle、旧档触面回归 | AC-02..06 | [ ] |
| T-06 | work | T-05 | 冷A→B/修复下一Run、SD和AG023准确消费版本/配方 | AC-05/06 | [ ] |
| T-07 | review | T-06 | 同树独立final，真实逐AC/SD/源和反例核验；finding同员有界复核 | AC-07 | [ ] |
| T-08 | merge | T-07 pass | 冻结受影响门/准确身份后落主，不扩大无关全套门 | AC-08 | [ ] |
| T-09 | merge | T-08 | canonical/同号归档/导航与真实支持/未证边界，非产品研究不冒功能交付 | AC-08 | [ ] |
| T-10 | merge | T-09 | 全路径对账/摘要折Plan/fresh guard清自有树与scope、hold登记 | AC-09 | [ ] |

## 9. 复审记录

2026-11-05 / stage: new / auto-ui/PLAN-058:r1 / outcome: pass（合同可审阅，dependency-ready任务见T-01）；后续依赖和真实先证门未满足时不得推进。当前drafting0/10；仅规划、没有新树/材料scope/源码/探针/构建/GPU/资产/canonical/SDK变更或agent。具体可执行边界见§5/§8，未把上游草案或WIP当交付。

2026-11-05 / stage: new / handoff: pass（T-01只读准备可进）；blocked（T-02等055/AG022实际交付与M2必需门真实收口）。drafting/r1/0-10，全部任务未勾；本批仅规划，未授权或启动本批实施。

2026-11-05 / stage: new / planning_validation: pass（合同预检，不是独立final/实现认证）。三份新合同各11节/10T（work6/review1/merge3）/9AC/6V-S/1SD，依赖实际解析DAG无环、M3未成为M2前置；主/归档及注册树独占取号后核UI058/059、AG023唯一，next UI060/AG024。三仓当前白名单新增/修改本地链接均可解析（含UI008补录），空白检查通过；其中共享导航包含他路线WIP，仅选择本轮AGE行/章节暂存，不代其提交。状态表逐项34 UI＋23 AG＝57（49交付/8未完），49个所列delivery均为各自主HEAD祖先，UI039旧draft影子不重复计。初次表计数模式只匹配“UI编号]”而漏带中文标题的8行，修正为编号后允许标题后复核通过；不是实现红。固定e02ca9f8的pipeline/lower_components2d_runtime与本轮所记源码内容一致；历史canonical/归档及当前054/AG021合同未由本会话修改。现有`python tools/gallery.py validate --all` exit0，12逻辑/22入口/六旧锁与许可一致。未构建/运行本批新probe/GPU/原生输入/容量/代码，未创建本批树/材料scope/新canonical/资产/SDK或agent；研究未执行不充go，计划未实施不充功能交付。

2026-11-05 / planning_count_correction：末轮逐项归属核验补入UI008（004身份修复，0261ae9为主祖先），正确统计49已交付/8未完；仅修正路线簿记数字，不改任何目标/AC/依赖/预算或执行状态。

## 10. 待澄清事项

D-01：055/AG022结果尚无，T-02前核最低链是否可在原包络成立；no-go/需提预算就replan，不把前置当已完成。D-02：目标输入轴/绑定/姿态、单sweep和sprite碰撞所有权由T-02冻结，禁止默加滑墙承诺。D-03：M2仍有具名必需门未证，不因本Plan草案而忽略。材料唯一`D:/autostack/plan-artifacts/plan-058/auto-ui/`，开工核跨仓同号owner并登记用途/路径/在途，外部B/SDK副本/target/原日志探针全收自有子scope；merge摘要折Plan、全输出对账/guard清树/删除自有scope构建物，仅具名owner/理由/关闭条件hold例外；不穿透链接、不碰外仓同号、不创建第二权威树。
