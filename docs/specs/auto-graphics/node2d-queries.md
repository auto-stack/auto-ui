# AutoGraphics Node 二维查询作者入口（auto-ui PLAN-021 SD-01 候选）

Status: candidate（auto-ui PLAN-021 T-03..T-06 实测交付；本稿为
docs/specs/auto-graphics/node2d-queries.md 的 SD-01 候选文本，merge 阶段
按收据落位。只描述实际交付并验证的能力，不声称任何未实现的语言/运行时
特性。）

## 0. 范围

`node2d-lower --profile queries-2d` 在 020 components-2d 组件合同之上提供
**显式选择的 typed 二维查询作者档**：普通 `use` 导入的组件在
model/on handler 内以 `query2d.segment(...)` / `query2d.sweep(...)` 对场景
世界发起 v3 segment 最近命中与 v4 swept-AABB 查询（`behavior.api =
autographics.objects2d/1` 不变；装配/运行时/守护零改动——v3/v4/spatial/
motion 单元本就组装在 objects2d/1 内，entry 每 callback
`_ag_spatial_begin()` + `_ag_cast_token=0`）。五条冻结产物链的执行证据
（只述实测）：**CPU 会话**（受信 VM、独立 f32 slab oracle、精确计费、
token 生命周期负控）覆盖 q-segment/q-changed/q-sweep/q-cross/q-cross-plus；
**离屏 GPU 呈现**覆盖 q-segment/q-changed（marker 位移 + door 显隐 +
changed 像素差异）与 q-sweep（clearance 停墙 + 像素间隔）；20 轮
Stop/fresh 与 Pause/Step/render-only 在 q-segment GPU 宿主上重放。V-06 消费/身份对全部五链成立；共享降层准入门覆盖五链，具名负控在 segment lab 与档边界上复放（五链同一下降/准入路径且均降层绿）。

非范围（不冒称支持）：3D 查询/GLB/triangle 或 OBB 精确碰撞、无限 ray、
新 physics/自动 slide/相对动态目标、角色玩法扩面、精灵/音画、长期 query
快照容器、通用借用检查器、动态拓扑/key/HMR、OS 键盘/焦点探针自动化
（原 G05 14 阶段 OS 证据按 D-03 映射为运行时等价观测，OS 阶段具名
pending）。**旧入口不变**：016 legacy、017 g11-2d、019 scene/character/
camera-2d、020 components-2d（components-2d 对 query2d 构造具名拒绝，
`query2d is only available in the queries-2d profile`）；旧消费者
node_components_flow 9/9、verify_plan020 --controls 18 组 8 产品复跑全绿。

## 1. 声明面（T-03 冻结拼写）

- 查询调用（仅 queries-2d；handler 体内）：`query2d.segment(ignore,
  visible_only, ax, ay, bx, by)`（2D 固定，z=0 隐含）与
  `query2d.sweep(mover, visible_only, dx, dy)`。handle 实参 = `refs.<顶层
  实例名>` | `self.row()` | `-1` 字面量（仅 segment 的 ignore；sweep
  mover 必须具名/self）。visible_only 必须字面量 0/1 或 true/false。
  元数错/形态错在执行前具名拒绝（module/line/symbol）。
- typed 结果：仅 `let q = <查询调用>` 绑定；q 按调用种类 typed 为
  cast/sweep 结果，getter 仅限结果变量方法：cast=`.hit()/.node()/.t()/
  .point(axis)/.normal(axis)`，sweep=`.hit()/.node()/.t()/.origin(axis)/
  .normal(axis)/.start_overlap()`。错 kind（'.origin()' is a sweep result
  getter; 'q' is a cast result）、未绑定、重绑、`var` 绑定、再赋值全部
  具名拒绝。q 即 token（运行时 int，同一世界共享 latest slot；第二次
  query/失败 attempt/callback 起止失效——原语义，运行时错 kind/stale
  具名 13，q-cross-plus 33 计费具名 sticky fault 10）。
- 具名行引用：`refs.<顶层实例名>` 值 = 该实例 primary 静态行的行
  ordinal（int 字面量）；实例须恰一静态 shape；character 实例行可寻址
  但位置只读。`self.row()` 经每定义 `{prefix}_qrow=[…]` 数组按 SoA 下标
  解析（多实例同 def 各得己行，如 querier_qrow=[3,4]）。行 op：
  `place(x,y)`（语句形态，两条 ag_set_local）、`shift(dx,dy)`、
  `show(flag)`、`world(axis)`/`local(axis)`；refs 与 self 两形同权；
  character 行位置写具名拒绝（位置由角色服务拥有；运行时 fault 21/20
  backstop）。
- 输入：`input.space()/input.x()/input.y()` → 原 transport 全局
  ag_input_space()/ag_input_x()/ag_input_y()（`_ag_action(space,x,y,toggle)`
  喂入；space==2 = 2D 游戏面）。不占用 character action 位、不强迫启用
  GameInput。
- 其余组件面（model/msg/on/params/Life/嵌套实例/8 包络/128 行/相机）沿
  020 冻结合同不重述；queries-2d 是 components-2d 的超集。

## 2. 行序与数值（T-02/T-04 冻结）

- 查询几何 = 场景文档行：character 行 id1/ord0 恒首 → 根 solid 行 →
  逐实例静态行（声明先序，同 def 多实例 SoA 下标序）；spawned slot 对象
  不入 spatial 世界。行 >128 具名拒绝（020 规则）。
- 数值沿原 v3/v4 profile：finite f32、闭区间、最近真实 t、精确同 t 按
  Run ordinal、signed-zero 平行不除、inside/boundary 起点 t0/n0、切触
  t1、X/Y/Z 法线择优、sweep Minkowski 收缩（L=tlo−half, H=thi+half，
  逐轴）、起始 overlap→t0/start_overlap1/n0（含 dx=0 与背离位移）、
  miss→O+D、GeometryUnavailable 9 / ArithmeticNonFinite 12 不降为 miss。
  观察容差 t=max(1e-6,8EPS)、position=max(1e-3,8EPS×max(1,|e|,|a|))；
  hit/miss/ordering/normal 不放宽。墙钟实测值记于计划收据（断言界为
  <8000us 宿主帧帽），spec 不固化点值。
- 生成源形态：引用行 ordinal 为字面量/qrow 数组查寻；二元运算符带空格
  （词法连字符标识符）；`*1.0` 语句值约定沿 020；`app_start` = 
  `ok=ok+ag_char_bind(1)*1.0` + `ag_camera_follow_2d(...)*1.0`（零 solid
  合法）。

## 3. 预算与包络（V-04/05 实测）

- 32 credits/世界 callback（共享池）：segment 链 13/tick（cast8+5
  getter）、sweep 链 23/tick（16+6 getter+origin 重读）、cross 两实例
  22/tick（2×(8+3)）；q-cross-plus 3 cast+9 getter=33 → 第 33 计费具名
  sticky fault 10。getter 计费 1 进原 API；不私读 VM 全局、无免费缓存。
- VM 守护原额（120 tick 独立报告）：max_call_steps 3418 < 100000、
  max_call_slices 4 ≤ 120。8ms 口径：segment 链 CPU 会话整 tick 墙钟
  断言 <8000us 宿主帧帽（flow 门逐 run 量得并打印，点值记于计划收据，
  本 spec 不固化）。
- 8 有状态实例/8 运行时槽/原 128 场景行包络沿 020；查询链不新增配额。

## 4. 生命周期（V-04 实测，q-segment GPU 宿主）

- 每 Run 重建：20 轮 Stop/fresh 后 runner 回声明位（6,2）、门重隐、链
  重跑绿（无旧 Run 泄漏）。render-only settle 不推进任何 tick；每个
  Step 恰一中性 tick；暂停期帧零移动。Start/Tick 计数语义沿 020。
- typed token 生命周期：callback 起 `_ag_cast_token=0`（entry 冻结）；
  同 callback 第二次 query 使旧 token 失效，重读具名 13；cast token 上
  sweep getter 具名 13（kind guard）；序号耗尽 14 不回绕（原语义）。

## 5. 诊断合同（V-01）

非法构造在执行前具名拒绝，消息含 `module <m>, line <n>, symbol '<s>'`：
query2d 非 queries-2d 档、未知服务/元数、visible_only 非字面量、未知行
引用（refs.ghost）、sweep mover -1、self.row() 在 root、未知结果变量、
错 kind getter、结果重绑/var 绑定/再赋值、character 行位置写、多静态 shape 实例
（具名行引用要求恰一静态 shape）。不 panic
（负面集 tools/verify_plan021.py 与 node_queries_gates q04 双重复放；
其中再赋值诊断同样携带 module/line/symbol 形态）。

## 6. 消费与身份（V-06）

沿 019/020 source-build 合同：每产品一目录 `scene.agscene`（behavior.
source 绑定 world sha256+byte_length）+ `world.at` + `identity.json`
（generator "autographics-node2d-pilot plan021 queries-2d"、api、逐模块
sha、closure 清单、roles、provenance）+ `closure/<module>.at`。五产品
（q-segment 52a37119…、q-sweep f7dc9f19…、q-cross eda9e765…、q-cross-plus
a9b71b0a…、q-changed dac39f5b…）字节冻结：`tools/verify_plan021.py
--controls`（20 控制组）——身份链逐环、仅凭 closure 副本冷 A→B 字节
等值、changed-variant 身份控制（同几何不同源绑定）、runner 位置变异
敏感性（scene/identity 移动、模块 sha 漂移捕获、无关模块不动）、11 条
负控具名复放 + components 档边界、坏件（一字节损坏 scripts/<sha>.at）
载入即拒（'behavior source closure identity: <scripts/<sha>.at>'）→
精确还原 → 同入口真 Run 再绿（driver
node_queries_flow::v06_damaged_queries_world_copy_rejected_then_recovery_
green 实演）。变体 = 声明入口；任何被用源变化使旧身份失效。

## 7. G05 九类义务映射（V-05；AG009 消费者的前置基面）

原 verify_g05 九类 require 在新链的落点：①/⑤ segment/sweep 成功 =
v02/v03 全链绿 + 独立 oracle 吻合（CPU+GPU）；②/⑥ 14 阶段 = OS 探针
阶段结构不属本链，映射为运行时等价观测（render-only settle、单步
tick、暂停零推进、20 轮 fresh 重建），OS 键盘/焦点/OS 画布还原阶段
具名 pending（OS 自动化暂停，不伪称实跑）；③ 非空 trace = flow 门逐
checkpoint 记录 hit/node/t/point/normal 并对 oracle 核验；④ saveas 导出
= 本 §6 source-build 身份/闭包/重开（坏件拒绝→精确恢复→真 Run）；⑦
changed 合法 = q-changed 全链绿；⑧ changed source 真变 = world sha 不
同 **且** 实际结果/像素不同（t 0.5→1.0 闭区间切触、0.3→0.6，AG009 新增
Q10 一并覆盖）；⑨ partial 拒绝 = sticky 具名负控（fault 10 在第 33 计
费、stale/kind 13、坏闭包载入拒绝）。原额（cast8/sweep16/getter1/32
池）与守护（VM/时限）原值保持。

## 8. provenance

本候选由 auto-ui PLAN-021 T-06 建立：事实行为沿 T-02 冻结合同（Plan §9）
与 T-03..T-06 实测（pilot gates 9+10+9+6+6、node_queries_flow 11/11 单
线程、verify_plan021 --controls 20 组 5 产品、verify_plan020 --controls
18 组 8 产品复跑全绿；五产品 world sha 见 §6）。016/017/019/020 既有
spec 与锁保持原状，不在本文件重述。
