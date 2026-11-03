---
title: AutoGraphics 三维查询与同根双出口诊断标记作者入口
status: candidate
plan: auto-ui/PLAN-039 r1
applies-to: node2d-lower --profile queries-3d（pilot experimental/autographics-node2d-pilot）；消费 experimental/autographics-auto-driver（assembly behavior/3、behavior/4）
assembly: autographics.behavior/3（segment-only 内容派生）/ autographics.behavior/4（含 sweep；内容派生，零新版本、零运行时单元改动）
date: 2026-11-03
---

# queries-3d：三维 segment/sweep、具名叶/组引用与有限同根双出口

## 0. 范围与机制（T-02 冻结）

显式查询 profile `queries-3d`（CLI `node2d-lower --profile queries-3d`）把已交付的
v3/v4 三维查询数值接到普通 Node：`query3d.segment/sweep` typed 面、具名叶/组引用、
以及仅本 profile 放行的同根有限双出口（`view-3d`＋`view-2d` 共用同一 model/msg/on）。
机制（D-01 关闭）：新增组合 dialect `Node2DQueriesDialect`——与默认
`Node2DDialect` 完全同一文法，唯一差异是解析期放行同 node 双 view；所有旧 profile
仍走默认路径，解析期具名拒绝字节不变
（`node '<n>' declares both view-2d and view-3d (single view exit per node)`）。
放行后的有限组合由本档降层准入裁决。未验证 v3/v4 数值语义（零算法改动）：
slab 最近 t、exact tie 低 ordinal、signed-zero、Minkowski 收缩、t 容差
max(1e-6,8EPS)/位置容差 max(1e-3,8EPS×max(1,|e|,|a|)) 全部沿
`api/behavior_segment_v3.at`/`behavior_sweep_v4.at` 冻结实现。

## 1. 声明面（T-02 冻结拼写）

- `query3d.segment(ignore, visible_only, ax, ay, az, bx, by, bz)`（8 参）/
  `query3d.sweep(mover, visible_only, dx, dy, dz)`（5 参）；仅 handler 体内。
- handle 实参 = `refs.<view-3d 叶>` | `refs.<顶层实例>` | `self.row()` |
  字面量 `-1`（仅 segment ignore；sweep mover 必须具名/self）。
  **组与二维标记不进查询 handle**（具名拒绝：
  `'refs.<g>' is a group row; groups are not queryable geometry` /
  `'refs.<m>' is a view-2d marker row; 3D queries address view-3d leaves`）。
- `visible_only` 必须字面量 0/1 或 true/false；typed 结果仅 `let q = <查询调用>`；
  cast getter `.hit()/.node()/.t()/.point(axis)/.normal(axis)`，sweep getter
  `.hit()/.node()/.t()/.origin(axis)/.normal(axis)/.start_overlap()`（axis 0..2）；
  错 kind/未绑定/重绑/`var` 绑定/再赋值全部具名拒绝（沿 021 文本族）。
- 行写/读：三维叶/组走 024 space3 形（`refs.<名>.place/shift/show/world/local/
  visible`；组行 place/shift/show/world/local 合法——动态父组写经组行）；
  二维标记 ref 仅 place/shift/show，且 **z 实参必须字面量 0**
  （`place on a view-2d marker keeps z at the literal 0`）；`self.place/shift`
  三轴形；标记不响应 world/local 读。
- 实例：顶层实例 ref = primary 静态叶（实例须恰一 view-3d 叶，否则具名拒绝）；
  `self.row()` 经每定义 `{prefix}_{elem}_rows=[…]` 行表按 SoA 下标解析；
  组行表 `{prefix}_{group}_rows=[…]` 随档发行。

## 2. 有限双出口（仅本 profile）

仅 root 允许 `view-2d`（子 Node 单维；非 root 携带 view-2d 具名拒绝
`view-2d is only allowed on the queries-3d root node (child nodes belong to
one dimension)`）。view-2d 仅接受 `shape2d`（kind: quad|circle；
kind/position/scale/rgba/visible 白名单；position z 与 scale z 必须字面量
0/1）。标记行 = 无 parent 的顶层 quad/circle 对象（作者 rgba；屏幕坐标），
在 `hierarchy::ordered()` 先序中位于全部三维行之后；**禁止跨维 parent**
（driver 载入层 `graph cross dimension` 兜底）。标记不被删除、不能冒充
生产 UI/HUD：它们只是本档场景文档的 quad/circle 行，经两个呈现出口
（Game2d 面 / Scene3d 面 + 屏幕叠层）按 kind 原生路由（driver host 现有
`kind.is_2d()` 分流，零新呈现管线）。多出口不复制 model 或 Tick；
model/msg/on 根内聚共享；每回调 Tick 恰一次（020 冻结序）。

## 3. 数值、费用、错误与包络（原额零改动）

费用沿 v3/v4 共享 32credits/callback 池：cast 8 / sweep 16 / getter 1 /
AABB attempt 1，先收费后读；segment 链实测 22/tick（本档夹具
8+2+8+4）、sweep 链 22/tick（16+6）。错误 9（几何不可用）/12（非有限）/
13（错 kind/stale token）/14（token 耗尽不回绕）由受信单元承担；
33 计费 sticky fault 10（现有 budget gates）。latest slot：q 即 token，
同一世界共享；第二次查询/失败 attempt/callback 起止失效——本档夹具以
`okint` 标志实证 stale 读等于原 snapshot。8 有状态实例/128 场景行/
VM 时限包络沿 020/024，本档零新增配额。

## 4. 行为 API 内容派生与身份

`behavior.api` 按生成内容派生：任一 handler 携带 `query3d.sweep` →
`autographics.behavior/4`；否则 `autographics.behavior/3`（G05 两出口
遗产一致；v4 单元为 v3 超集）。产品四件：
`scene.agscene`（behavior.source 绑 world sha256+byte_length；assets 清单）+
`world.at` + `identity.json`（generator `autographics-node2d-pilot plan039
queries-3d`）+ `closure/<module>.at`（冷再生唯一输入）+ `assets/<sha>.glb`
字节等值复制。变体 = 声明入口；任何被用源变化使旧身份失效。
A→B 冷再生：仅 closure + GLB 字节即字节等值再生 scene+world
（verify_plan039 实测）。

## 5. 永久夹具与验证入口（全部真实执行）

fixtures/queries3d/{q3-seg,q3-changed,q3-sweep,q3-cross}（原 G05/3d 多部件
GLB 42c9da67… 1528B）。冻结产品 generated-queries3d/。
- 再生/身份/形状/A→B/坏件负控：
  `python tools/verify_plan039.py`（--controls 只跑再生/形状）。
- 降层门（V-S01 拒绝矩阵）：
  `cargo test -p autographics-node2d-pilot --test node3d_queries_gates`。
- flow 门（V-S02..04，独立 f32 oracle + 真实 Host/GPU）：
  `cargo test --manifest-path experimental/autographics-auto-driver/Cargo.toml
  --test node3d_queries_flow -- --test-threads=1`。

## 6. G05 九义务映射（AG017 消费基面；D-03 适配）

原 runner 498ce814 与 OS phase-14 不可用（D-03 继续持有）；各义务经新链以
真实 CPU/GPU/保存证据再生产：①segment 运行绿＝flow real_host segment；
②phase14 等价＝host start→tick→pause→trace→stop 生命周期；③trace≥1＝
逐 tick oracle 断言；④saveas 导出＝storage 保存链（identity/坏件恢复）；
⑤sweep 运行绿＝flow real_host sweep（behavior/4）；⑥sweep 生命周期＝
start→20 hold→stop；⑦changed 一致＝q3-changed 同几何异 t；⑧changed 源哈希
差＝world sha 不同；⑨partial 拒绝＝具名拒绝矩阵 + 33 计费 fault 10。
完整 G05 逐字迁移（含 OS 探针历史）归 AG017；本档交付其作者面与可锁消费
接口：场景文档 G05 形约定（id 1..6 + 组 id、标记行序）以
samples/segment_projects/sight3d 为准，AG017 迁移时按 §1/§2 拼写重写。

## 7. 非目标（不冒称支持）

精确三角/OBB 或无限射线、运动响应/刚体、动态拓扑/资源/scale/yaw/pitch、
任意 UI 内嵌 3D 视口、多屏呈现变体选择、角色/相机组合、旧 profile 双出口
放行（旧 profile 原拒绝字节不变）、G3 游戏/声音 HUD 组合、跨场景多挂载。
本档的有限双出口不解释为所有旧 profile 自动支持混合呈现。
