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
- **迟到事件（PLAN-051 修订）**：identity 查找 miss 时，MCP/派发面先按件名做
  存活解析（恰一个存活实例 → 重定向它——视图携带的 `TypedInst.instance`
  是页名而 `child_instances` 键为键控身份，格式脱不再丢合法交互）；
  0 个存活 → 事件丢弃 + 诊断（真迟到）；≥2 个存活 → 丢弃（歧义不静默
  裁决）。进程内 `Component::on` 面保持严格 identity 合同（不重定向
  同名新实例，§5.2 合同 4）。
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
- **Vue 轨资源 owner（PLAN-057 SD-02 增量）**：store 实例 effect（SSE 连接/
  timer/watcher）在 factory 内经 `__autoOnCtxDispose` 登记进所属 App store
  context；context 按 owner App 记账（`own`/`releaseApp`，`app.onUnmounted`
  钩子与显式 unmount 释放去重），**最后 owner 退出即释放全部 effect 并清
  实例 memo**。legacy 无 context 消费保持 page-lifetime（不释放）——具名
  边界不扩权。运行语义细目见 vue-app-store-live-isolation.md §3。
- 042 正式出口：实例身份串、实例态对象 id、`__auto_parent`/`__auto_root`
  端口、帧登记/释放语义、`child_instance_identities()`/
  `read_instance_state`/`write_instance_state`/`on_with_input_for_instance(_type)`
  即 reload/schema/effect owner 研究可消费的实例描述面；不预发完整 reload 能力。

## 5. 验收锚点

编译到 Rust 的路由根（PLAN-060）：页面 Init 经过根的子消息包装臂派发，
因此初始化产生的 store 更新与普通页面事件一样回写对应 canonical store。
根无自身周期定时器时，将当前 outlet 页的单一定时器间隔及包装后的消息
提供给原生 runner；非活动页不被该入口轮询，无匹配路由返回 None。
根自身定时器优先、每组件仅暴露首个定时器的既有边界不变；这不扩展为
父子多定时器调度器，也不改变 VM 的实例时间源。

- 永久 fixture：`crates/auto-ui-runtime/tests/fixtures/plan041_instances/`
  （同型双计数 keyed a/b + 动态 key 交换 + show_b 条件卸载 + 共享 P041Store）。
- Oracle：`crates/auto-ui-runtime/tests/plan041_instance_ownership.rs` —
  t01 控制 / t02 props+model 隔离+store 共享 / t04 keyed 重排身份 / t04 卸载
  释放+存活续作+迟到丢弃+重挂载全新。
- Vue 厂形契约：`test_plan041_store_factory_and_context_facade`（compiler）。
- **覆盖边界（PLAN-057 更新，如实）**：041 交付时 Vue 轨验收为生成契约级，
  无挂载浏览器运行——该缺口已由 PLAN-057 关闭：浏览器活体隔离与资源退出
  为实证合同，见 docs/specs/runtime/vue-app-store-live-isolation.md（e2e
  入口与其 §5）；本文 Vue 轨生成契约级断言继续有效。
- 兼容面：根桥读面（`read_state`/`read_all_state`）仍 root 锚定；plan370/
  442/622/841/905-8/063/058/musk-p053-8 原义务套件全绿（oracle 更新至实例态
  读取通道，义务本体不变）。

## 6. Reload 提交下的实例迁移（PLAN-056 SD-02 增量）

- 候选提交时旧桥全部存活身份按 `read_instance_state_all` 读出、跨桥重物化
  （`rematerialize_from_old`，容器深拷进新 VM 防悬垂）、`import_child_instance`
  预播种进新桥（parent 先指新根，首帧 `ensure_child_instance` 命中既有记录
  时修正 parent ref 链）。
- 迁移实例的 Init demand 预置 Done——keyed 重排/提交**不重放 Init**（每身份
  每代际一次的 536 挂载语义保持）；卸载重挂仍重 Init。
- 已删除 widget 类型的身份 tombstone：迟到事件丢弃 + 诊断，不重定向/复活。
  未再挂载身份经常规帧扫释放 + tombstone。
- 实例 model 字段 typed 迁移（同名不同类型 = 整候选拒绝保旧）；prop/seed
  键由父通道每帧重播按名保值；新 model 字段按新定义默认补位。
- 旧桥 busy（parked/CPU continuation）时提交被拒保旧、下拍重试。

## 7. 异步续体的实例与宿主归属（PLAN-068 SD01）

- HTTP 与 CPU 分帧续体保留派发时的 receiver 堆 id。子件本地 model 仍写
  自己；`router.push/back` 的隐藏宿主槽经 `__auto_root` 写所属 App，根件
  通过自引用使用同一路径；没有按件名复制路由字段的回落。
- 实例卸载在字段引用释放前取消以其原始 receiver 挂起的任务（含等待凭据、
  栈份额与 busy 镜像），移除该 receiver 的排队写事件；相同文本 key 重挂后
  使用新堆 id，旧请求不能导航根或改变新实例。已完成的前缀副作用不回滚。
- 续体成功完成后统一处理路由参数、返回请求和视图 memo 失效；失败后已有
  可见状态也失效重建。状态已写入不等于完成可见投影。
- 永久入口 `tests/plan068_routed_continuations.rs` 使用真实延迟 HTTP、实际 CPU
  让出、路由 child 的实例态和 rendered View 作 oracle；包括双同型实例共享
  Store、跨 App 隔离、卸载/重挂迟到控制。原 `(widget, handler)` 在途重入限制不变。

- Send/MCP 消息先按 U+0001 分离件类型与不透明实例地址，再仅在类型段解释
  `@seq`；实例地址内的 `@`、路由斜杠和 key 数据必须原样保留。显式 instance
  参数优先于 carrier，但 carrier 的类型段仍先剥离，不能把地址拼入 handler 名。
  永久反例使用真实 View 绑定产生的路由 HTTP/含@key消息，以及实际 Iced
  生产转换/系统动作入口；仅手传 instance 参数不能替代该 Send 边界覆盖。

## 8. 输入绑定注册与值写投影（PLAN-069 SD-01）

- `input`/`textarea` 的 `value:` 绑定注册键 = **状态字段路径的投影名**，与视图
  读通道同口径：
  - 直 field（`.q`、`input`）→ 裸字段名；
  - 嵌套 store 路径（`.store.q`、`.StoreName.q`——store 字段合并进根态）→
    展平后的根字段裸名；
  - 非 store 别名的真嵌套对象（`.note.title`）**不展平**（保持不注册）。
  绑定事件（oninput/onchange/onenter）首个命中优先，首匹配语义不变。
- 绑定写回顺序与读通道一致：**实例对象优先，实例无该字段回落根态**。页模型
  字段（实例所有）写实例；展平 store 字段（根态合并）写根。
- 一次性键入 = 整体替换该字段值（类型按字段现值保值），不做追加。
- 永久反例：`crates/auto-ui-runtime/src/ui_tests/plan069_synthetic_input_tests.rs`
  （嵌套 store 绑定注册/写入、直 field 对照、二次键入替换、handler 读回同值）。
