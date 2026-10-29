# PLAN-001 runtime-objects 冻结合同（T-02 产出）

计划：`auto-ui/PLAN-001`（r3）。本文档冻结有限运行对象的公开合同：profile、身份编码、
公开函数、准入块、计价、错误码、支持包络与整路径预算。冻结基线：worktree
`D:/autostack/.wt/ui-001/auto-ui`，分支 `codex/plan-001-dev`，base `a9e8d70`（=规划 pin
`4adf2d0` + 两笔 docs-only 提交，代码恒等）。

## 1. Profile 与装配

| 候选 | 版本串 | API number | wire profile | 组合 |
|---|---|---|---|---|
| objects2d | `autographics.objects2d/1` | 105 | 16 | motion(v5)+character2d+camera2d+objects |
| objects3d | `autographics.objects3d/1` | 106 | 17 | motion(v5)+character3d+camera3d+objects |

- 单一可信单元 `api/runtime_objects_v1.at`，两维共用；装配期把 `var _ag_obj_dim=0`
  字面量替换为 2/3（require 标记保证恰好一次），运行期无任何改维入口。
- 组合 profile 必含角色与相机（`_ag_char*_freeze`/`_ag_cam*_freeze` 在 app_start 后，
  `_ag_obj_freeze()` 在 app_start **前**——表必须先于首次 spawn 存在）。
- 每回调入口链：`_ag_char*_tick_begin(); _ag_obj_tick_begin(); app_update(dt)`——
  后者只重置每回调生命周期写计数器。
- entry 改写、命名空间化、compact 规则与 875 完全一致；单元内禁止字符串字面量
  （segment_compact 既有守卫）与无空格 `-`（`a-b` 会词法成 GenName，实测确认）。

## 2. 身份（整数句柄）

- `handle = run_id*524288 + slot*65536 + generation`；slot∈[0,8)，generation∈[1,65535]。
- 全程整数运算；句柄解码 `rel=handle-base; rel<0 → 外来/旧 Run（33）`。
- 耗尽均具名拒绝、不回绕：generation>65535 → 36；run_id>4095（句柄域顶）→ 36。
- 每槽 generation 单调递增（despawn 不清零水位）；同代内 template 冻结。
- 禁止浮点截断/低位别名：spawn/move 的 slot/template 是 int 参、坐标是 float 参，
  VM 类型系统隔离（host Session 调用上限 4 参不影响用户侧 5 参 spawn）。

## 3. 公开函数（全部先查 sticky error）

| 函数 | 返回 | 语义 |
|---|---|---|
| `ag_object_spawn(slot int, template int, x f, y f, z f) int` | 句柄/0 | 空槽+同维非组叶+受支持 kind+有限坐标（2D z=0）→ 建表项，gen=水位+1 |
| `ag_object_despawn(token int) int` | 1/0 | 撤销身份；后续读写/二次销毁均 33 |
| `ag_object_move(token int, x f, y f, z f) int` | 1/0 | 只改位置 |
| `ag_object_visible(token int, flag int) int` | 1/0 | 只改显隐（不销毁、不撤身份） |
| `ag_object_alive(token int) int` | 0/1 | 唯一合法 absence 检查：旧代/旧 Run/死槽返 0，不设 sticky |
| `ag_object_x/y/z(token int) f` | 值 | 读位置 |
| `_ag_obj_map(stable, node)` / `_ag_obj_freeze()` / `_ag_obj_tick_begin()` | 内部 | host 装配注入，非公开 |

## 4. 错误码（新段 31–38，sticky first error，整 callback Fault）

31 InvalidSlot · 32 SlotOccupied（arg=活代） · 33 ObjectNotAlive（死/旧代/旧 Run 句柄）
· 34 InvalidTemplate（未知/组/错维/不支持 kind） · 35 ObjectDomain（非有限/越界/2D z≠0/坏 flag）
· 36 身份耗尽（generation 或 run 句柄域） · 37 每回调生命周期写超 32 · 38 未冻结/配置错。
`0.0/0.0`、`1.0/0.0` 在 VM 算术边界 DivisionByZero fail-closed（先于任何 spawn）。

## 5. 计价与写帽（整路径 probe 冻结）

- 每次公开对象调用（含失败调用）= **1 credit**，共享既有 `_ag_cast_charge` 32/callback
  预算；对象读也计费（与 spatial 一致，主机直调同样扣）。
- 生命周期写（spawn/despawn/move/visible）每 callback ≤ **32 条**；写帽检查**先于**
  扣费（第 33 条写=37，而非预算 10）。存活回调的实测上限形状：角色移动
  （`ag_char*_move` ≈26 credits）+ 3 销毁 + 3 重生 ≈ 32 credits 恰好达标——这是
  002"三物品同时收集"的预留操作组合（V-02 probe 证据，见 §8）。

## 6. 帧块与准入（Play::admit）

- 布局：`[4 head][22×N nodes][camera block][objects block]`；objects 块 =
  `4 头（status,count,run,tick）+ 8×6（gen,tmpl_runidx,x,y,z,vis）` = 52，定长。
- 每行校验：free 行全零（防混叠）；live 行 gen∈[1,65535]、template 落在受检 base
  顺序且同维非组叶、坐标有限且 |v|≤100000、2D z=0、vis∈{0,1}。
- 转移校验（对上一 admitted 帧）：gen 严格不减；同代只许位置/显隐变化；
  重生（经 absent）必须 gen> 该槽历史高水位（host 侧 [i32;8] 跟踪）。
  status≠1/run/tick 不符/计数不符 → 具名拒绝，候选整帧丢弃（last-good 保持）。

## 7. Host 侧包络与渲染

- 构造门（`Host::start_program`）：`base 3D mesh 数 + 8×最劣合格模板 mesh 数 ≤ 512`，
  具名 `objects mesh envelope`；`Play::new_user_spatial`：`base nodes + 8 ≤ 128`，
  具名 `objects slot capacity`（121 节点即拒，120 可起）。
- prepare（每帧复检）：潜在 mesh = base 3D + **每个活槽**的完整模板 mesh 数
  （隐藏/未激活不折减）；>512 具名拒绝，无半帧。
- 渲染：非持久适配 `runtime_objects::instance_object(template, slot)`——id=0（不可能
  与任何文档对象/选择混淆）、模板 kind/scale/yaw/pitch/材质/resource、实例位置/显隐；
  2D 走活动相机/视图投影，3D 走 `store.get_or_upload`（隐藏模板首生上传一次，此后共享）
  + MeshInstance。不写 Document/scene4/dirty/Undo。
- owner：实例表随 Frame/Play 生命周期；Stop/Fault 释放 VM weak owner（20 轮 churn
  证据 §8）；mesh 按 (sha,name) 共享，不随出生数增长。

## 8. 整路径 probe 证据（冻结依据，本地工件）

- 会话级 probe（真实装配 VM）：`cargo test --locked --test runtime_objects_api --
  --test-threads=1` → **9 passed 0 failed**（≈20s）。覆盖：身份/装配冻结数、8 槽全
  生命周期、隐藏读活、同槽重生 gen2、占位 32、双销 33、旧代 33、跨 Run 33、
  65535 代耗尽 36 不回绕、32 写帽 37、33 credits 10、可见性语义、命名反例表
  （31/34/35 各 arg、VM 除零 fail-closed）、三物品收集组合（两维）。
- 主机级 probe：`cargo test --locked --test runtime_objects_host -- --test-threads=1`
  → **6 passed 0 failed**（≈12s）。覆盖：实例 2D 绘制随帧同事务移动、隐藏不绘制、
  3D 全 mesh 渲染+位移同帧、多 mesh GLB 模板（two-parts，2 primitive 全渲染）、
  失败 last-good、120 节点包络边界起/121 拒、20 轮 churn owner 关闭+上传平坦。
- 本地原始输出：`D:/autostack/plan-artifacts/auto-ui/plan-001/work/`（摘要记录，
  命令/exit/数字以本节为准）。

## 9. 支持集与非目标（继承计划 §5.2，逐字冻结）

每 Run 单维；8 预留槽；模板=本 Run 接受的同维非组叶（2D: circle/quad；3D:
cylinder/player/mob/model）；资源/kind/scale/yaw/pitch/材质冻结自模板；spawn 只给
根级世界 xyz。实例非 solid、不可作角色/相机 target、不进旧 ordinal 查询。触发事件/
新局/Gallery = 002；不改旧 profile 装配/输出（回归证据：lib 27 通过含 assembly
9 API 全边界变异，camera/character/authoring_play/output_admission 选受影响套件）。
