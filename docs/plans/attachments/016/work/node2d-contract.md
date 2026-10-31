# PLAN-016 Node2D 执行合同（T-02 冻结）

status: frozen（T-02 go；原型整链已实测通过）
plan: PLAN-016 r3 · auto-ui · tree `D:/autostack/.wt/ui-016/auto-ui` · branch `codex/plan-016-dev`
work base_commit: 69690dfb8e0b691a9af8d9e1eacc240511d2e83e（master tip，含 PLAN-013 交付 `8b1e799`、归档 `60e6d48`）
SDK pin: `7b987ca8`（`research/baseline.json.txt` sdk.repository_commit；`tools/prepare_sdk.py --from-snapshot` 681 files PASS，`.sdk-source` 本地 prepared，不入 Git）
只读依赖: `D:/autostack/auto-lang` @ `3fa75dd9`（只读引用，无写面）；AGP 同仓 path dep。
原型证据: `D:/autostack/plan-artifacts/auto-ui/plan-016/probe/t02-prototype.log` SHA256 `1e54b98f5cdf9a1dfbd9e172e30b51de4bc39246e329500efcaa20e628e72a19`（`cargo test --locked --offline --test plan016_probe -- --test-threads=1 --nocapture` exit 0，2 passed / 0 failed）。

## 1. 接法比较（T-02 上限两种，实测裁定）

**(a) 生产 UI 模块/编译服务执行有限 Node 实例 —— 结构性排除，不改生产代码。**
证据（代码即设计，非缺陷）：VM merged 模式是"单一统一状态对象"，子件 model 字段按名合并、先到先得去重（`crates/auto-ui-runtime/src/ui/vm_bridge.rs:894-916`，Plan 370 D-GAP-4 注释原文）；`ensure_child_state` 把子 props 写进 ROOT 状态对象（`:1791-1839`）；状态实例唯一 `{Widget}_State`（`:972-975`）；handler 派发按 `handler_<Widget>_<Event>` 无实例 receiver（`:3517-3611`）。两个同定义实例的隔离状态需要生产运行时新增每实例状态机制 = 通用 UI 行为变更，违反本计划"显式 pilot 隔离路径/旧 profile 不改"边界（§5.2、§6），故不实施、不修生产面。

**(b) 共同领域模型 → 局部研究 adapter → 固定 AG 运行服务 —— 选中，原型已实测通过。**
全部复用既有面：dialect 解析器（共享 lexer/parser）、`resolve_use_module`/SDK `resolve_module_path` 模块定位、AG `Triggers2V1` 组合 profile（= triggers+objects+camera2+character2+motion+sweep+segment+spatial+base，`assembly.rs` 组合链）、`SourceSnapshot::admit` 准入、`Session` CPU 分片执行、Host/AGP 真 GPU 离屏。**驱动与本仓生产 crate 零改动**（原型的 driver tests/ 文件为 T-02 探针，T-04 被 `node2d_flow` 取代后删除）。

## 2. 整链分段（file/symbol/owner/写面）

| # | 段 | 输入→输出 | 位置/符号 | owner | 写面 |
|---|---|---|---|---|---|
| 1 | 模块工程 | `fixtures/minimal/{main,resources,types,player,hazard,coin,patrol}.at` | pilot workspace 内 fixture | work | 新增 |
| 2 | 领域解析 | 源码文本→Node2D 研究 AST（node/params/model/msg/view-2d/on、Life.*、匿名 msg、前导点） | `auto-ui-dialect` 新增研究 dialect（不动 `UiDialect` 旧路径；旧 widget 解析字节不变） | work | 新增文件+测试 |
| 3 | 模块绑定 | `use` → SDK `resolve_module_path`/`resolve_use_module` 定位、逐文件解析、定义表 | `auto-ui-compiler` `ui_ast.rs`（只消费），测试 `tests/node2d_pilot.rs` | work | 仅新增测试 |
| 4 | 校验/诊断 | 缺模块/错导出/类型错/坏 actor ref/重复绑定 → 带行列位置拒绝（执行前） | pilot `validate`（消费 2/3 的 AST 与 span） | work | pilot |
| 5 | 实例 lowering | 定义表+实例表 → 单文件 Auto 用户源（SoA 存储/逐实例 handler/事件绑定表/编排 app_start·app_update）+ 场景文档 | pilot `lower`；`SourceSnapshot::from_bytes_for(_, Api::Triggers2V1)` 准入 | work | pilot |
| 6 | 执行 | `Session::new(combined)`、`_ag_reset/_ag_identity/_ag_append_field/_ag_spatial_config/_ag_*_map/_ag_start`、`_ag_tick(t, 1/60)`（`_ag_char_input` 宿主侧每 tick 一次采样） | driver `Session`（未改） | 复用 | 无 |
| 7 | GPU 呈现 | pilot 场景文档工程 → `Host::with_project` → start → `h.frame(None)` 泵 → `rt.probe_pixels`/`capture` PNG | AGP `SurfaceRuntime/OffscreenTarget/capture`（未改）；模式同 `tests/public_hud_consumer.rs` | 复用 | 无 |
| 8 | 观测/验收 | 纯 Auto 零消耗 getter（状态）+ `_ag_status/_ag_error_argument`（fault）+ GPU ROI/像素 + `verify_plan016.py --controls` | pilot tests + tools | work | pilot |

## 3. Lowering 冻结规则（原型实测得出，违反即红）

1. **存储**：每 node 定义一组 SoA 数组（`h_phase=[]` 等），实例索引寻址；定义→单 handler + 实例参数。根 node 用标量。根/实例同名字段不共享（哨兵 oracle）。
2. **事件**：匿名 `msg` → 业务 handler fn；`emit`/前导点绑定 → 生成端绑定表（Auto 数组），投递为真实 Auto 调用，receiver 由表驱动；枚举载荷降为 int 常量（`Key.Amber=0`）；`ActorRef2D` → AG 运行时 handle int。系统事件不进 msg：`Life.Start` → `app_start` 内逐实例 starts=1 + 初始化；`Life.Tick(dt)` → `app_update` 内固定顺序逐实例调用。
3. **tick 顺序**（每 `_ag_tick`，固定）：输入采样（宿主 `_ag_char_input` 一次）→ Player（`ag_action_held`→`ag_char_move` 扫掠）→ Hazard 逐实例 patrol（条件三角波）→（AG end-of-tick 触发 evaluate）→ 下一 tick 事件读取→守卫→`Collect`→条件差量（真 despawn，非 hidden）→ 呈现数据随 admitted block 走。
4. **禁止**：float `%`（准入可过、VM 执行 sticky fault，用三角波条件式）；直接引用/调用保留名（`fields`/`_ag_*`，准入拒绝）；顶层非 const 初始化器；`use` 进入 AG 准入（模块已在 lowering 前解析）。声明先于使用（VM 源序编译）。
5. **场景模板**：模板行必须停靠在玩法区外（原型：(-100,-100)），实例从模板 spawn；角色行在 `ag_char_bind` 前落位（原型 (2,1)），否则 born-overlap（motion status=4/3）永久锁死运动。
6. **预算（实测形状）**：共享 32 credits/tick 池 = `_ag_cast_charge` 系（held 1 + move_and_slide 24 + scan 1）+ objects ops（spawn/move/despawn 各 1）+ trg 读（event_next/pair 各 1）。收集 tick 全分支 **恰 32**：held1+sweep24+scan1+2×hazard move+event_next1+pair1+despawn1=32；第 33 个计费调用 → 命名 sticky fault 10（已测）；objects ops 超 32 → fault 37（objects 单元自带守护，AG 既有测试覆盖）。VM 分片 1000steps/2ms、100000steps/120resumes/250ms、30s init/2s tick/8ms pump 目标照旧（`Session`/`Play` 未改）。
7. **读回**：验收读回必须走**纯 Auto getter**（读用户状态，零计费）。`ag_object_alive` 等计费读取每 tick 限次并在 tick 分支内预算；任何直接会话调用落在 tick 间会叠加上一 tick 余量（原型 get_c_alive 事故=合同依据）。声明返回类型与读取 `ResultKind` 必须一致；VM 层跨 fn 返回可能 float 化（camera 实测 2.0f），累加器一律用 float。
8. **生命周期**：`Play`/Host 的 Start 一次/Pause 无 tick/Step 恰一中性 tick/Resume 不重 Start/Stop 回编辑态、8 槽对象/8 对触发/代次 handle（旧代跨 run fault 33/32）全部复用未改；pilot 20 轮短例只做观测（V-04）。

## 4. 冻结入口与命令

- dialect: `cargo test -p auto-ui-dialect --test node2d_pilot`（研究域解析+诊断，V-01）
- compiler: `cargo test -p auto-ui-compiler --test node2d_pilot`（模块绑定/定位诊断，V-01）
- pilot: `cargo test --manifest-path experimental/autographics-node2d-pilot/Cargo.toml --locked --offline --test node2d_flow -- --test-threads=1`（整链 V-02..05；GPU 用例单实例串行）
- 工具: `python experimental/autographics-node2d-pilot/tools/verify_plan016.py --controls`（源身份/负控；0selected 拒绝）
- 准入面: `Api::Triggers2V1` 单一 profile；pilot 源闭包 ≤256KiB 用户/96KiB 桥/352KiB 合计；资源用场景文档程序化图元，不加新资源格式。
- 消费 A→B（T-06）：A=本树构建；B=独立导出目录+冷 `.sdk-source`（`prepare_sdk.py --from-snapshot`）+独立 target，`--locked --offline` 同输入同结果。

## 5. 已知边界（不冒称支持）

view-3d/portrait/widget 与混合嵌套、Life.Frame、多呈现选择器、PNG/GLB 导入、完整 G11（三 Coin/伤害/终点/Enter 新局）、ECS/一Node一VM、生产 UI 融合——均非本合同交付。设计语法中的 `emit` 关键字与 refs/export 端口在研究入口内按上述第 2 条降实现，不声明为最终语言形态（SD 只写实测能力）。

## 6. 阶段门

- T-02 go：本合同+原型通过（已满足）。T-03 起按 §2 写面实施；改 §3 规则或预算先修订本合同再实施。
- 原型探针文件 `experimental/autographics-auto-driver/tests/plan016_probe.rs`：T-02 原型工件，随 T-04 `node2d_flow` 落地删除（superseded），删除记录写 §9。
