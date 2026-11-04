# 组件与Store实例所有权（component and store instances）

> PLAN-041 交付合同（SD-01/SD-04）。同型 child 的 model/props/handler
> receiver 按实例归属；store 按 App 实例共享、跨 App 默认隔离；owner 退出
> 只释放自己；迟到回调不重定向。分类：feature ｜ 状态：active ｜ 来源计划：
> docs/plans/041-component-and-store-instance-ownership.md

## 1. 实例身份与状态对象（C-01）

- 实例身份 = `<父实例路径>/<widget名>#<调用位 key | 本帧位置序号>`。调用位
  `key:` prop 存在时身份随 key（keyed 重排保身份）；未 keyed 按本帧位置序
  （旧位置语义，**不承诺任意移动保身份**，§10.2）。
- 每个调用位实例持独立 `GenericInstanceData` 堆对象（`<Child>_State@<token>`）：
  种子集 = resolved props（父传 + 声明默认 + 父同名同步一次性）+ model 默认，
  外加隐藏字段 `__auto_parent`（VmRef→父态对象）与 `__auto_root`（VmRef→App
  根态；root 自引用）。字段集 = model ∪ 声明 props ∪ 种子键 **并集**
  （step-2 定格名单不完整——step-3/4/4b 的追加必须并入，否则 prop 默认字段
  不落实例对象）。
- model 字段仅分配时播种（PLAN-536 T8 纪律延伸：handler 写入跨帧存活）；
  **声明 props 每帧重播种**（值变化才 `bump_path`，PLAN-045 T-02 纪律）。
  相同类型/字段名不是共享理由：plan-320 的同名根态同步通道随合并态退役，
  父→子走 prop 通道、子→父走 emit。
- **msg 型回调 prop**（`on_close: msg`）不进入自有字段集——裸名调用保函数
  符号解析（pre-split 形状）；若被状态改写劫持，链接器解析 `<W>_State.on_close`
  类型化符号 → link 失败（widgets-gallery SettingsPanel 实录）。
- 挂载/卸载 = 帧内登记（`ensure_child_instance` 插入 seen 集）；上一帧在册
  本帧缺席 → 帧末释放（堆 stake 反押、Init 记录清除→重挂载重 Init，os-config
  身份语义按实例成立）。memo 命中跳过子树遍历时以 owner 前缀重放存活标记
  （`memo_hit_replay_instances`，漏标=误释放活动实例）。

## 2. receiver 与派发（C-02）

- 视图读：子视图 builder 的 `override_state_obj_id` 指向实例对象；字段缺失
  回落 root（store/父属字段在根解析，提供链语义）。
- handler 派发：`call_handler_for(widget, event, state_obj_id)` receiver
  参数化；实例限定事件（`DynamicMessage::TypedInst`，实例子树绑定产生）按
  identity 解析实例对象；iced Send 边界经 widget 名内嵌 U+0001 标记过界、
  派发入口剥离（与 `@seq` 装饰同款约定）；MCP 派发面同约定。
- 名字合成派发（`on_with_input_for` / `call_widget_handler`，instance=None）：
  该件名恰有**一个**存活实例 → 路由到它（验收通道真实意图）；0 或 ≥2 个
  存活实例保持 root legacy（歧义不由通道静默裁决）。
- **迟到事件**：实例已释放（identity→None）→ 事件丢弃 + 诊断，不重定向
  同名新实例（§5.2 合同 4）。
- 具名边界：parked 在途重入守卫按 `(widget, handler)` fn 名键——同名 handler
  的两个实例不能并发各持一段（pre-existing PLAN-702 语义，未随本包改键；
  需要并发段时另立调度面合同）。

## 3. App 级共享与隔离（C-03）

- **store（VM 轨）**：STORE_GLOBAL + `__module_init` 每 bridge 一次 = 每 App
  一份；跨 App 天然隔离。子件 handler 体内 `store.Method()`/`.store.X` 与
  父属字段统一经 `__state.__auto_root.<f>` 指向共享态（`router.param` 同跳）；
  emit 账面（`__emit_msg`/`__emit_payload`，三写点：桥函数体、空体引号
  handler 两路）同跳落根——清算侧（on_with_input_for 尾部）读 root。
- 方法归属路由（Plan 370/446）不变；实例不静默分叉 store。
- **store（Vue 轨）**：store composable 呈厂形——state refs/watchers/sibling
  facades/SSE/timer 守卫/accent bootstrap 全部在 `create{Name}Store()` 内
  （每 App 上下文一实例）；`use{Name}Store()` 签名不变，`inject('__autoStoreContext')`
  命中 → `ctx.use(name, factory)`（同 App 单例共享），未命中 → 懒初始化模块
  单例 = **具名 legacy 边界**（无 context 的挂载/纯模块消费；新多 App 路径
  不得依赖它）。`main.ts` 与 gallery `mount()` 工厂 `installAutoStoreContext(app)`
  ——每个 createApp 挂载一个 context；两个同名 App 隔离。
- 显式共享（跨 App 共享 owner）走 context 注入端口（mount 时注入外部
  context）；无隐式跨 App 共享。

## 4. 资源与生命周期（C-04）

- 实例释放 = 反押全部字段 stake + `__auto_parent`/`__auto_root` 引用 stake +
  对象 stake；堆对象由常规 RC reaper 回收（id 单调不复用）。
- 播种/重播种配平沿 materialize 协议：`materialize_init_value` 自带容器
  retain，**不得再 `stake_state_value` 叠加**（双押 = 卸载后泄漏，plan841
  compound 实录 rc=2）。
- Tick/订阅（PLAN-654 InstanceTimeSource）与主题 scope（PLAN-031 ThemeScope）
  分属既有机制，不随实例分裂改键；组合验收引 plan033_b1/theme_scope 系。
- 042 正式出口：实例身份串、实例态对象 id、`__auto_parent`/`__auto_root`
  端口、帧登记/释放语义、`child_instance_identities()`/
  `read_instance_state`/`write_instance_state`/`on_with_input_for_instance(_type)`
  即 reload/schema/effect owner 研究可消费的实例描述面；不预发完整 reload 能力。

## 5. 验收锚点

- 永久 fixture：`crates/auto-ui-runtime/tests/fixtures/plan041_instances/`
  （同型双计数 keyed a/b + 动态 key 交换 + show_b 条件卸载 + 共享 P041Store）。
- Oracle：`crates/auto-ui-runtime/tests/plan041_instance_ownership.rs` —
  t01 控制 / t02 props+model 隔离+store 共享 / t04 keyed 重排身份 / t04 卸载
  释放+存活续作+迟到丢弃+重挂载全新。
- Vue 厂形契约：`test_plan041_store_factory_and_context_facade`（compiler）。
- **覆盖边界（如实）**：Vue 轨验收为生成契约级——composable 厂形/context
  注入/legacy 回落断言 + main.ts/mount 工厂安装字符串；无挂载浏览器运行，
  跨 App 运行时隔离由 per-createApp context 构造保证而非浏览器实证。
  双同名 App 的挂载运行隔离验证归后继（需要 Vue e2e 轨道，plan042 研究或
  gallery e2e 批次可消费）。
- 兼容面：根桥读面（`read_state`/`read_all_state`）仍 root 锚定；plan370/
  442/622/841/905-8/063/058/musk-p053-8 原义务套件全绿（oracle 更新至实例态
  读取通道，义务本体不变）。
