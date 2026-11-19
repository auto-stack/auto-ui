# 俯视二维组合（topdown2d）与 AGE 图形端口

PLAN-058 的交付知识（SD-01；r8 范围修订后本文描述 **AV 支持域**）。
适用身份：本文件由 auto-ui 分支 `codex/plan-058-dev` @ `eb724dd0` 预备、merge 时落主；
SDK 基座=`.sdk-source` 两层快照（base 681 + overlay 1435，auto-lang `0a167f611`，
含 PLAN-981 G1/G2/宿主动态世代批）。旧 055 研究结论见
`topdown2d-feasibility.md`，其 no-go 结论由本组合的显式新档解除，研究数据不重述。

## 结论及使用边界

**正常 Auto 作者的俯视二维最小组合（1 Player＋2 静态障碍＋2 同定义交互目标＋
1 可见非阻挡地面）在 AV 车道完整可用**：无重力四/八向控制（归一化，斜走不快于
直走）、单次 sweep 停于首次接触、segment 最近交互查询、同 Tick 精灵呈现、
每世界回调 32 credits 共享池。**AC 机器执行格与 A2R 真编格未证**——由
PLAN-103（等 1068/T-04＋981 AC/A2R 域端口＋UI-W2-AC/W3-A2R 波）承接，本文不冒
双端支持；作者面"业务→实例事件派发"（触碰路由）同归 PLAN-103。

| 消费面 | 支持域 | 入口 |
|---|---|---|
| AGE 包前门（AV） | `compile_age_package[_with_scenario]`，入口模块 `pkg::mod`；场景=实例数＋定向 ops（`ForNode`/`Send`/`Tick`） | `ace_core::source::age_package` |
| AGE 执行（AV） | `AgeGfxSession`＋`AgeGfxHost::dispatch_with_ctx`（宿主实现 16 op=13 标量＋3 record；`frame2d.at` 纯值构造不派发） | `ace_av::age_gfx` |
| 离线组合档（无 AGE） | `node2d-lower --profile topdown2d`（profile 30/version 121） | driver CLI |

## 组合档与格式（T-03，旧档零漂移）

- **`autographics.topdown2d/1`**（profile 30/version 121，CLI `topdown2d`）：显式
  俯视 2D 栈=render-only 感知的 segment/sweep/motion＋character/camera2d/
  objects/triggers/sprite＋render-only 行表；首个同 assembly 含 query+sprite 的档。
- **`autographics.scene/9`**＝/6 形状＋可选 `layer`（kebab；仅 2D 叶行；
  缺省=legacy collision）。≤/8 含 `layer`、或旧 profile 载入含 layer 文档→具名拒。
  旧 5..8 文档 encode/hash 逐字节不变（无 layer 键）。
- **render-only 行**（`behavior_renderonly_v1.at`，0 credit）：可见但永不进查询
  候选（候选循环跳过；render-only 作 mover/ignore→具名 fault 60）；行仍持真实
  AABB。`Descriptor::query_candidates()/is_query_candidate()` 为公共候选视图。
- **兼容守恒**：旧 22 profile/10 冻结 unit/价目/号逐字节不变；104 个旧
  generated-* 场景 0 layer；旧 021/022 消费者全绿。守护=`tools/verify_plan058.py
  --controls`（C1..C6）。

## AGE 作者面（981 交付的消费合同）

- **行层事实（G1）**：`shape2d <name> { layer: render-only }`——受检行描述携带
  typed `AgeSceneRow.layer`（cpu 面同步 `Symbol("render-only")`）；同一份描述交
  AV/AC/A2R。移除该属性⇒行重入碰撞候选（055 反例可判别）。
- **Tick 作者面（G2）**：`on Life.Tick` 体支持 hook-local `let`（typed
  `let q Sweep2D = query2d.sweep(...)`，本执行路径求值一次）与 `if/else`；赋值
  目标必须是 model 字段（hook-local 只读、分支内绑定不外泄）。
- **021 token 纪律（作者形约束）**：回调内第二次 `query2d.*` 使此前绑定的名字
  失效（语言走查 `source.age.query-token-stale` 具名拒；token 经字段写传播进
  状态槽——每字段状态函数尾读自身槽，故查询前的字段写须在查询后以达成值重写）。
- **每函数静态准入**：隐藏函数内 gfx 调用点合计 ≤32（超界
  `lower.age-gfx-credits` 具名拒零效果）——计费链不能靠分支复制。
- **端口与价目**：segment 8 / sweep 16 / 其余服务 1 / 端口字段读 0；每世界回调
  （Tick）共享池 32：「下一次扣费后累计超过 32 即具名拒绝、零效果，本 Tick 后续
  收费调用保持拒绝」；`begin_tick()` 重置。record op 一次调用返回完整
  `AGE_PORT_FIELDS` 序（Cast2D 7 字段/Sweep2D 8 字段/Image2D 4 字段）。
- **会话事实与判罚**：`AgeGfxCall { tick, callback, query_epoch, credits_used,
  symbol }` 经 `dispatch_with_ctx` 逐派发到达；callback=世代边界（每 `run_def`
  +1、epoch 槽归零）；`query2d.*` 派发单调推进 epoch（1 基）。**判罚点在宿主
  车道**：stale query token（fault 13 语义）与 stale render handle（fault 33
  语义，旧代写拒）由宿主按事实具名拒（UI058 实现为
  `age-gfx.stale-query-token`/`age-gfx.stale-render-handle`，载荷双面命名
  tick/callback/epoch）。
- **模块级资源 `let`**（`let atlas = image2d.png(...)`）在本 AV 切片**不派发**；
  资产 op 在作者调用处运行；资产路径**工程相对**（asset root=工程根）；
  `render2d.frame` 只记录、不触 fs。

## 行为保证（AV 真跑验收）

- 一次 sweep 停于首次接触：候选按 mover 半宽 Minkowski 收缩，最近 t 胜、同距
  保声明序（严格 `<`）；停位不动（t=0 touch=overlap，`start_overlap=true`）。
- segment 最近目标：裸盒、最近 t、同距小 ordinal；目标状态实例独立
  （`use` 实例行 per-instance cpu 几何＋`ForNode` 场景定向，2:1 状态隔离已证）。
- 同 Tick 呈现：Tick 内 `render2d.spawn/frame` 与查询/移动同回调发布；
  GPU 像素=相机投影（≤1px）＋逐帧位移匹配行差（连续帧单宿主 6 帧实证）。
- 输入：`input.x()/y()` 为轴值（−1..=1），对角 ×0.70710677；真实键事件管线
  （焦点门控/OS 抑制重复→sample→宿主权威）已验；暂停清除输入。
- 生命周期：Run=全新 session（无跨 Run 残留、世代计数归零、确定性轨迹）；旧
  句柄/坏模块/缺资产具名拒→修复后同入口真 Run；冷 A→B 按内容 sha 换源、
  `identity().input_hash` 失配即旧 Ready 不适用。

## AG023 AV 消费配方（r8 域）

1. 源：age-package 单模块（样例 `experimental/autographics-auto-driver/
   fixtures/plan058/topdown-node/main.at`）；场景行声明序=行 ordinal；
   地面行 `layer: render-only`。
2. 编译：`compile_age_package_with_scenario(&inputs, "pkg::mod", Reject,
   &[PACKAGES_CAPABILITY], &ComponentScenario{instances, ops})`——消费者按需
   定向（Tick/ForNode/Send）。
3. 执行：实现 `AgeGfxHost::dispatch_with_ctx`（行表按受检描述 cpu 几何构建、
   `layer`→RenderOnly；资产根=工程根）；`AgeGfxSession::new(provider)`；
   每 Tick `begin_tick()`＋`run_def(&checked, "d_<symbol>")`（状态函数名
   `__age_state[_n<node>]_<inst>_<field>`，按符号表解析、不猜 mangling）。
4. 离线档：`node2d-lower --profile topdown2d --entry … --out …`。
5. 验收入口（永久用例）：`tests/plan058_t04_av_chain`、`plan058_t05_gpu_continuous`
   、`plan058_t05_use_instances`、`plan058_t05_gfx_epochs`、
   `plan058_t05_av_os_input`、`plan058_t05_av_lifecycle`、
   `node_topdown2d_flow`、`tools/verify_plan058.py --controls`。

## 未证边界

AC 机器执行格、A2R 真编格、双后端配方、触碰路由作者面（PLAN-103 承接）；
OS 消息泵→winit→Play 边界（运行时窗口域，098 已验，本组合不重述）；
`render2d.frame` 的资源名仅字面透传（G4 语义已由 981 钉死，资源身份深化归后续）。
