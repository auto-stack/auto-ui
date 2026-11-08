# AGE M1 只读原生取证支撑（auto-ui PLAN-071 SD-02 候选）

Status: candidate（PLAN-071 T-04..T-06 实测交付；本档只描述已交付并验证的
记录接口、schema 与消费边界。手工逐项语义与第二真实 DPI scale **仍未
证**——记录接口使它们可被永久重建，不自动宣称通过；消费方为
ag-examples AG026/DEBT-001 与 AG027/DEBT-002。）

## 0. 范围与判据

一个版本化报告 schema（`autographics.m1-native-evidence/1`）＋两个宿主的
只读记录接面（mixed `--manual` 检查表扩展、editor `--evidence-record`），
使 AGE M1 的原生手工门可被**永久重建**：每次记录绑定真实项目身份（路径＋
sha256）、候选身份、窗口实例（实际 scale/尺寸/真实 ScaleFactorChanged）、
逐检查点实际读数与 N1..N6 人工逐项观察。记录逻辑只读宿主公开面
（texts/callbacks/scene_state/epochs/widget_bounds），不改 Tick/派发/路由
（不干扰性有永久回归）。

分层规则（validator 强制，`m1_evidence::validate`）：

- `offscreen` / `posted-window` / `state-record` 是**控制/绑定层**，永不
  native-pass（具名 blocked）；
- `native-manual` 记录在任一 N1..N6 结论缺失、观察人缺失、项目/候选身份
  缺失、或实际 scale 档 <2 时保持 `pending`（逐条具名 blocked 原因）；
- `auto_pass` 恒为 false；报告出现 true 视为**伪记录**具名 invalid；
  `failure` 字段非空 → `fail`（不是 pending，不吞错误/timeout）；
- 事件计数多少不等于语义正确——结论字段（人工逐项）才是判据；
- 单一 scale 档（本机 2.0）永不闭第二真实档；模拟 scale/posted/离屏通道
  不可升级。

## 1. 检查表与字段（N1..N6，冻结）

| 项 | 内容（沿 PLAN-066 T-02 手工检查表具名化） |
|---|---|
| N1 | Tab + Enter activate the next button exactly once |
| N2 | a click outside every control changes nothing |
| N3 | Alt-Tab away then back: the stale focused button does not activate on Enter |
| N4 | mouse leave cancels the press (no region click) |
| N5 | resize; the next click lands on the widget's new position |
| N6 | repeat on a second DPI scale if the machine has one |

每项携带观察值（计数/尺寸/文本）；**结论**只由观察人写入
（`conclude_checklist` / 手工填报告 `checklist_items.N*.conclusion`）；
观察人字段（name/notes）同面。`ScaleFactorChanged` 记录真实 winit 事件
（前后实际读数）并扩展 `tier.scales_observed`；resize/键鼠/失焦/离窗事件
各产生一个检查点（实际 scale/尺寸/count 文本/callbacks/active regions/
epochs）。

## 2. 记录接口（两个宿主）

- **mixed**：`mixed_regions_window --manual --ui <ui.json> --world
  <world.at> --report <显式目录>`（关窗写 `manual-native-gate.json`；legacy
  字段 title/mode_note/observed/checklist/note 与 auto_pass=false 逐字保
  持，新增 schema/run_id/project/candidate/window/checkpoints/os_events/
  checklist_items/observer/tier/status 面）。`--probe` 代码路径不变
  （记录器只在 manual 分支活动）；`--report` 必须显式指向本 scope，
  禁止沿代码默认旧 plan054 路径落材料。
- **editor**：`autographics-authoring --open <project/scene.agscene>
  --evidence-record <显式 out.json> [--evidence-attach <原 OS 阶段报告>]`
  ——绑定实际项目/当前状态（open path、document sha、history/dirty/
  selection、captures、实际窗口 scale/px、runner sha），attach 以内容身份
  （sha256＋解析摘要）挂接原阶段记录；**不重建 20/34 阶段 oracle、不自动
  跑原 probe**。无 host（项目未开）写具名 fail 记录。
- 共享模块：driver `src/m1_evidence.rs`（builder＋严格 validator＋
  `validate_file` 消费入口；单元测试 6 例＋集成面在
  `tests/m1_native_evidence_support.rs` 3 例）。

## 3. 冻结身份（2026-11-08 交付）

| 项 | 值 |
|---|---|
| base / 分支 | auto-ui `126fe757`（master）/ `codex/plan-071-dev` |
| 依赖 | auto-lang `30b396d72`（AG025 候选同版，组内兄弟树 detached）；SDK 快照 681 文件（`prepare_sdk.py --from-snapshot`，PASS） |
| fixture world.at | `9c18df0014dbf45da5affe4f9e97a3882f8c871b7d29f9f5fa6c346ecd1ce7a8`（7056 B） |
| fixture ui.json | `9388f6e7da2373cd9742eee9a94ed7f452451880d4d5ba7049b2a9156b052cac` |
| window runner | `mixed_regions_window` d20fc737…（本 Plan 构建面；AG 候选 93f82722 为历史身份） |
| authoring runner | `1c1578bf…`（同上；历史候选 71a95437） |
| 054 canonical | `node-mixed-regions-lifecycle.md` 原文 `4753cd65…`（本 Plan SD-01 增量见 §4） |

**候选失效影响（沿 age-m1-acceptance §3 规则，不改原失效规则）**：
window/authoring runner、driver 库（node_mixed_regions_host/m1_evidence）、
pilot 生成器（consumer 臂新增）任一变化 → 本档冻结的 runner sha 与
`tests/m1_native_evidence_support` 结果失效，AG026 最终候选须按原 Spec
在同版受影响门重验；原 M1 失效规则不放宽，第二真实 scale 外部门不因本
档关闭。

## 4. AG026/027 消费配方（交付时命令，全部显式输出）

- 再生成 fixture（字节恒等由 pilot gates 钉）：
  `cargo run -p autographics-node2d-pilot --bin node2d-lower -- --profile
  node-ui-mixed-regions --entry fixtures/node-mixed-regions/dashclip.at
  --out generated-node-mixed-regions/ui-m1-clip-forensics`
- 应用级 clip/路由/挂卸门（离屏，本档 oracle）：
  `cargo test -p autographics-auto-driver --test m1_native_evidence_support
  -- --test-threads=1`
- 真窗单档 posted 回归（原 showcase fixture，exit 0 oracle）：
  `mixed_regions_window --probe --ui
  generated-node-mixed-regions/ui-dash-regions/ui.json --world …/world.at
  --report <显式目录>`
- 手工门（AG026 N1..N6 逐项＋双档，人工执行；结论/观察人由人填）：
  `mixed_regions_window --manual --ui
  generated-node-mixed-regions/ui-m1-clip-forensics/ui.json --world
  …/world.at --report <显式目录>`，随后 `m1_evidence::validate_file` 判
  pending/native-pass。
- 编辑器绑定记录（AG027 原阶段记录接面）：
  `autographics-authoring --open <project/scene.agscene>
  --evidence-record <显式 out.json> --evidence-attach <原报告>`。

## 5. 当前未证边界（不因本档关闭）

- 第二真实 DPI scale、真 native SendInput/焦点捕获、N1..N6 逐项人工语义
  ——DEBT-001（AG026 外部门）；本机远程虚拟显示单档 2.0。
- 真实 OS 的 20/34 编辑输入阶段——DEBT-002（AG027）；本档只提供记录接面
  与绑定，不代执行。
- 手工记录器 `--manual` 的运行时报告由人工会话产生；本交付含编译＋schema
  门＋`--probe` 路径不干扰证据，**不含**一份已填观察人的人工报告。
