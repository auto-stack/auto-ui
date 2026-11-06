---
plan_id: auto-ui/PLAN-060
status: archived
feature_name: a2r 数组索引 VM 语义与 iced 桌面 outproc 输入/验收投递
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-07
plan_revision: 8
current_step: 5
total_steps: 6
base_branch: master
base_commit: 7ac0455bf4cd743248749e4bc4eb9d0aed657266
original_base_commit: 220115df62040f113bc689f0500af80fc11569a3
depends_on: []
supersedes_spec_components: [docs/specs/runtime/component-and-store-instances.md, docs/specs/build/generated-vite-config.md, docs/specs/build/desktop-api-surfaces.md]
new_spec_components: [docs/specs/runtime/desktop-acceptance-input.md, docs/specs/runtime/native-codegen-semantics.md, docs/specs/runtime/native-command-frame-text.md]
touched_goals: [auto-os/PLAN-054]
affects: [a2r-codegen, ui-iced-runtime, mcp-acceptance, vue-wm-close, auto-os/PLAN-054]
---

# auto-ui/PLAN-060：a2r 数组索引 VM 语义与 iced 桌面 outproc 输入/验收投递

## 0. 变更摘要

auto-os/PLAN-054 的 iced 轨在 052 交付后只剩两个框架级缺陷：

1. **native 轨数组索引不按 VM 宽松语义**：`arr[i]` 直接发射 Rust `arr[i as usize]`，
   越界/负索引直接 panic。calendar.exe 启动首帧以 `mm=0` 调
   `dim[(mm - 1) as usize]` → `0-1` 回绕成 `usize::MAX`，进程 exit 101，窗口
   「不可用」。
2. **iced 桌面无法把输入/handler 投递到 outproc 原生 app**：OS 指针不可达
   （前台锁 + PostMessage 被 iced 忽略），MCP `autoui_desktop key` 路由到
   focused wid 但不产生输入焦点（无指针通道），MCP `handler` 对 outproc app
   命中的是桌面侧 rqhost-client 组件、不达 app 进程。结果 notes/todo 等无法
   完成任何用户任务。

本计划在 auto-ui 侧修复这两处：①Index 发射改为边界安全（VM 语义：越界/负索引
→ 缺省值，不 panic）；②验收通道补指针注入并让 handler/键盘真正投递到 outproc
app。消费方 auto-os/PLAN-054 T-04 在交付后恢复 8 app 真实任务验收。

非目标：不改 app 源码；不实现 router/多窗口新特性；不新增 Vue 桌面宿主功能。既有053来源的兼容实现已由060为054集成并验证，范围见 SD-05/06，不代表053计划已完成；
不扩大 VM 语言语义；不改 auto-lang 编译器。

## 1. 目标

- G1：native（iced）轨 `arr[i]` 对任意运行时 i 不 panic；越界/负索引按 VM 语义
  返回缺省（数值 0/串空/bool false/Value Null/记录 Default），且索引仍
  在界内时行为与现有一致。
- G2：iced 桌面验收通道可对 outproc 原生 app 完成真实 UI 操作：指针点击
  （MCP→broker pointer）、键盘字符、handler 直呼三者至少「指针+键盘」可用，
  handler 直呼若协议可行一并落地。
- G3：既有行为零回归：单路由/多 app 桌面、counter、既有 nextest 面不新增红。

## 2. 架构方案

主责 auto-ui 本仓。实施组目录 `D:/autostack/.wt/ui-060/auto-ui`，分支
`codex/plan-060-dev`（自 master `220115df` 起）。验证消费树为
`D:/autostack/.wt/os-054/auto-os`（只读构建消费，不修改其源码；apps 为
已交付 pin）。

- **索引**：`crates/auto-ui-compiler/src/ui_gen/rust.rs` 的
  `Expr::Index` 发射位（约 15965-15985）与 Value 链位（13703）统一改为
  边界安全访问；按目标型选：
  - 已知 Rust `Vec<T>`（T: Clone + Default）→ 注入 shim
    `__at_idx(&v, i)`（`i<0` → default；`get(i as usize)` → cloned 或 default）；
  - `Vec<serde_json::Value>` → 现成 `a2r_std::json::at_owned/get_owned` 语义
    （已有路径优先，不重复造）；
  - 无法判型时保守发射 shim（T 由类型推断，编译期报错即回退该文件既有形态）。
  shim 在 `wrap_example` 文件级注入一次（与 `__at_*` 家族同址）。
- **输入/验收投递**：`crates/auto-ui-runtime/src/ui/session.rs` 增加
  `DesktopInject::Pointer { phase, x, y, button }`（click/down/up），
  `apply_desktop_injects` 走 `broker_pointer_down/up`（已有生产路径）；
  `mcp_server.rs` 的 `autoui_desktop` 增加 `action:"pointer"`（x/y/button/
  phase=click|down|up）。键盘沿用 `route_live_input`；坐标输入更新hover
  位置以供后续滚轮命中。outproc根消息通过既有`InputMsg::Action`（tag11）
  和Hello `input-action/1`能力协商派发，不新增ControlMsg::HandlerCall。
- 桌面侧 `DesktopInject::Handler` 对 outproc 分支改为「找到 owning client →
  发Action」；不支持/发送失败不执行本地占位组件。inproc保留旧字符串参数，
  并增加经正常事件循环派发的受验证标量args（VM既有i32/f32 ABI）。

r2（同一用户授权）：保持G1..G3、AC-01..06及八app目标，按实际实现校正
最初协议草图，并收拢054消费中发现的同链缺口：生成后端真实链接/类型边界、
路由Init的store回写与活动页单定时器、关闭实例的验收快照所有权、原生文字
自身变换的帧投影。均为既有能力正确传播，不新增router/多定时器调度/语言
语义或任意原生子件RPC。app业务源码修复由独立owner计划承担（如Blog007）。

## 3. 技术栈

Rust（auto-ui-compiler / auto-ui-runtime / auto-ui-tools）、a2r 生成器、
iced 桌面会话/协议、MCP acceptance 工具。测试：`cargo nextest`（auto-ui 既有
固定组）+ 生成物编译 + iced 桌面实机驱动（os-054 候选）。

## 4. 需求分析与背景调查

r2授权沿用2026-11-06用户「想办法解决这些问题，并最终完成计划054」及恢复
Goal的明确指令；仍在auto-ui生成器/运行时/验收工具范围内修复既有链路。
Windows电脑控制被用户Esc停止后不自动激活，实际输入验收通过本产品已有MCP
通道；资源清理按已授权自有构建/测试进程处理。不得自动创建代理或新聊天，
AC-05独立复审要求保留，当前实现上下文检查不能冒充终局独立复审。

**授权（2026-11-05，用户直接指令）**：在 auto-ui 立项并交付上述两处修复，
允许修改 auto-ui 生成器/运行时/验收工具，允许在 os-054 候选树重建并实测；
不含 app 源码修改；Vue 桌面仅收口已集成且实际验证的兼容路径（SD-05/06），不承诺053的其余工作。

现状证据（2026-11-05 实机，auto-os/PLAN-054 T-04）：

| 证据 | 内容 |
|---|---|
| calendar panic | `calendar.exe` exit=101，`index out of bounds: len 12, index 18446744073709551615` @ `auto-calendar-85dd4543/src/main.rs:466`，生成码 `dim[(mm - 1) as usize].clone()`（`__call_DaysInMonth`） |
| 输入不达 | `handler NewNote`（app=015-notes / auto-notes / widget=App 三种形态）仅 `queued`，UI 无变化；`key chars` 路由到 focused wid 但 todo 输入框无字符；OS 点击因前台锁/PostMessage 忽略不生效 |
| 桌面侧 handler 路径 | `shell_state.rs::apply_desktop_injects` 的 Handler 臂对 outproc app 取 `state.apps[app_id].component.bridge_mut().call_handler`——该 component 为桌面侧 rqhost-client，非 app 进程 |
| 运行时骨架 | `session.rs::broker_pointer_down/up` 已向 owning client 发 `InputMsg::PointerPressed/Released`；`broker_char/key_event` 发 `CharTyped/KeyPressed`——指针/键盘协议已存在，缺的是「注入入口」与 app 侧消费确认 |
| 工具链 | 052 交付后 auto-ui master `64aef78` → 现 `220115df`；os-054 组内 ui-desktop 已按 052 版重建，paint 已可启动 |

## 5. 详细设计

### 5.1 索引语义

- shim（文件级一次注入，`#[allow(dead_code)]`）：

```rust
fn __at_idx<T: Clone + Default>(v: &[T], i: i64) -> T {
    if i < 0 { return T::default(); }
    v.get(i as usize).cloned().unwrap_or_default()
}
```

- 发射：`Expr::Index(target, idx)` → `__at_idx(&({target}), ({idx}) as i64)`；
  当 target 为 `Vec<Value>` 时保持既有 `at_owned/get_owned` 路径；当元素不可
  Clone/Default 时保留原索引并在该文件记录 `// PLAN-060: non-clone element`
  （编译门兜底）。
- 索引继续参与 `arr[i].field` 链：shim 返回 owned 元素，字段投影照旧。
- 写位（`arr[i] = v`）已走 `.get_mut()`（3243 行），负索引同样补 `i<0` 短路。

### 5.2 输入/验收投递

- `DesktopInject::Pointer { phase: PointerPhase, x: f32, y: f32, button: MouseButton }`
  - `down` → `broker_pointer_down`；`up` → `broker_pointer_up`；`click` = down+up；
    坐标更新last_cursor；不承诺独立move动词。
- MCP `autoui_desktop action:"pointer"` 参数：`x,y`（窗口逻辑坐标）、
  `button`（left/right）、`phase`（click/down/up）。AUTOUI_ACCEPTANCE门不变。
- outproc：Hello能力门通过后发`InputMsg::Action { wid, name, arg }`，child
  `Component::dispatch_named`进入真实事件循环；零参数/单字符串与root App
  为已实现边界。inproc标量args不扩展这个wire边界。
- 路由页Init经根子消息包装臂回写canonical store。根无自身timer时只转发
  当前页既有单timer（原本地优先/首timer边界不变）。
- 原生帧文字先组合item父变换与Paragraph/Editor自身变换，再发v1坐标或
  v2 Transform；正确滚动/点击通过完整producer→host replay验证。

### 5.3 原生 Http profile 与 JSON 身份字段（r3 有界修订）

OS054 的真实 Chat 反例证明：PLAN052 原生 Http 适配直接把 `/api/register` 交给网络内核，既未解析应用后端地址，也未内嵌后端；不能按其他 `api.*` 应用推断其可用。沿 Chat 既有共享 Rust HTTP profile，服务供给与所有权由 OS054 现有启动接缝及必要 app owner 前置处理，本计划不新增 supervisor 或更换后端。

生成代码的 Http.post/get/get_json 经 native_http 适配接收本 app 生成时选定的后端基址。相对根路径仅附加此地址；绝对 URL 保留自身 origin；不修改宿主全局环境。基址常量与 receipt 的 profile/http-base 输入同源，端口/生成 profile 改变不得复用旧产物。发布布局版本升为 v10；exe 与含 Http 的 shell-library 生成面都提供所需常量。响应体保持 JSON 文本，API HTTP 拒绝不伪作网络断连；transport status0 则进入既有 try/catch 的连接失败路径，诊断不得携带 URL token/请求体。

原生 JSON 属性名启发式不得覆盖已声明 String 状态目标；字符串身份 u-54 保持原值，整数控制只对原始 JSON 读取转换一次，已拼装字符串不重复当 Value。边界是本次状态赋值和既有 Http 命名空间，不承诺新增语言类型/路由器。下一执行入口：compiler Http/ID 两个实际红例→runtime 真实 TCP POST/GET/拒绝控制→tools 生成常量/缓存输入→正式工具/Chat 重建→OS054 实机注册、双客户端和 C 隔离。只完成前段不能关闭 app/OS 验收。

### 5.4 原生子组件回调闭环（r4 有界修订）

实际 Chat 输入框键入后点击发送，数据库消息仍为零（native-chat-callback-red-r152.json SHA079a13ab3f8066c253a68ce09b9d39e01c3a0376688d867c8329a9900cbf1bbb）；`Composer::DoSend` 中 `on_send(.draft)` 被发射为空操作。父包装枚举只转发输入，不等价于回调。AC与任务数量不变，继承054完整交付授权，修复仍归T04/05。

输入链为 .at `msg` prop调用→子组件处理器→当前实例待转发队列→父组件已声明绑定的消息。队列属于组件实例，单次输入处理后按调用顺序取走；必须先回写子store再调用父消息，保留嵌套回调，不因再次渲染或后续输入重放。支持既有普通消息绑定（含零参、字符串、数值、多参），按父消息声明反序列化实际参数；绑定/形状错误应响亮失败，不能静默默认。没有绑定的可选回调不触发其他父消息；不使用全局队列、网络协议或业务专用分支。

实施入口ca754c7e；永久compiler案例覆盖实际解析提取后的事件/prop绑定、不同回调目标、调用顺序与参数保持，执行生成代码验证父态一次性变更及子态清空；实际Chat重建后点击联系人/输入/发送→双端显示/数据库发送者与文本一致。先跑旧码红例，最终生成执行和真实任务都通过才关闭此项。循环子件沿现有临时实例模式，回调也从该实例消费；其私有状态持久化不在本修复新增承诺。

### 5.5 JSON载荷与生成输入完整性（r5有界修订）

Chat原生生成源码仍把失败pending重建中的pp.client_msg_id按字段名转为整数，字符串拼接得到0而非原意图编号；`.messages.push(sr.message)`把对象投影成空字符串。此为r3字段启发式修复的相邻消费者，必须按实际值语境保持动态JSON：字符串串接读取原Value后用既有str垫片，Value集合push保持原JSON形状。仅处理可辨认JSON读（局部/字段/安全索引）；声明String、typed实体和数值算术保持既有类型语义。数组索引仍经__at_idx，不因重建Dot链恢复越界访问。源与fixture在compiler rust.rs，owner T04/05。

原生生成器实际吸收本地src/back/*.at（否则back/*.at），但front_input_hashes未记录它们。沿同一选择序登记该本地目录的.at内容与文件集合；添加/修改/删除均应失效旧receipt，必需后端输入读取失败中止，不能把缺失当已验输入。外部back.project的完整闭包属于既有独立接缝，不在本次扩作通用依赖系统。保留pac/front/sidecar/theme/http-profile输入；发布版本v12。tools永久fixture实际改变后端内容/文件集合，README控制不失效，覆盖src/back优先与back回退。

执行入口42573475（行为bcacacf7）；先执行compiler JSON串接/对象push与tools后端mutation实际红例，再修复/回归相关p060子集、工具重建、Chat003正式重建和真实断连→failed→恢复→点击原失败泡重试，数据库须使用原会话nonce和原序号仅加一行。其他6app冻结构建与父054混合/失败门仍是后续关闭条件；不以Vue重试通过代替native原意图验证。AC/任务总数/认证与协议不变，继承完整目标授权。

### 5.6 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/runtime/native-codegen-semantics.md | 原占位路径不存在；记录数组安全读取及实际native生成边界，合并后端查找委托当前SDK字节偏移规则 | 与VM及已落地956标准库语义对齐，不保留旧字符偏移适配 | AC-01/AC-04 |
| SD-02 | new | docs/specs/runtime/desktop-acceptance-input.md | pointer/keyboard、根组件 Action、旧客户端能力门、关闭重开契约；生成入口主题包装器也必须转发命名消息及其handled结果 | iced 桌面可驱动原生 app，主题包装不得吞输入 | AC-02/AC-03 |
| SD-03 | add | docs/specs/runtime/native-command-frame-text.md | 文字自身/父项变换组合一次，v1/v2滚动几何一致 | 投递后的真实画面正确 | AC-02/AC-04 |
| SD-04 | modify | docs/specs/runtime/component-and-store-instances.md | 编译原生路由Init回写/活动页单timer传播 | 同组件事件与store所有权 | AC-02/AC-04 |
| SD-05 | modify | docs/specs/build/desktop-api-surfaces.md | 新默认基线已有同源API规范及956服务边界；保留全文并补060为054集成验证责任，以及既有标题栏关闭控件不得进入拖动捕获、实际卸载后重开 | 既有关闭/卸载合同实际可达；053来源兼容实现不被冒称独立交付 | AC-04 |
| SD-06 | modify | docs/specs/build/generated-vite-config.md | 桌面依赖并集及分app代理现已在新main，保留规则，仅补准确集成验证来源；非桌面不变 | 与已验证桌面API隔离及生成/run自愈同源，不制造第二套规范 | AC-04 |

（落点文件以仓库现有 Spec 为准，work 时按实际路径修订；若该节不存在则在本
文件对应 Spec 新增小节。）

### 5.6 原生样式滚动容器同义转写（r6有界修订）

OS054真实Chat第六条消息已落库但气泡被裁切，滚轮不改变位置。定位：RustGenerator普通row/col分支只发射View::col().style(overflow-y-auto).build()；VM tags_container::convert_col既有转换为View::Scrollable。r2的原生滚动测试显式手写View::scrollable，未覆盖样式声明→生成器→真实滚动的前段。修复归既有T04/AC02/04原生呈现消费者，不改变任务数/AC/发布范围；之前compiler9/tools2与JSON/callback证据仅在其不变边界保留，完整最终生成/runtime联合门仍待执行。

原生普通row/col生成在最终实际View上调用有界overflow转换；按求值后的Style检测OverflowYAuto/OverflowAuto，动态style同样只求值一次。无overflow声明与overflow-hidden保持原View；显式scroll标签不重复包裹。共享VM已有split_scroll_visual_classes/ensure_full_scroll_dims拆分规则移到view模块供两端复用，视觉留视口/布局留内容，尺寸与滚动轴保持现有Y合同；不新增横向滚动或自动滚到底功能。只处理普通row/col路径，不将未实现的其他标签兼容声称全覆盖。

关闭条件：编译器旧发射反例红→修复后literal/dynamic style与空/嵌套列控制绿；实际HeadlessFrameSource/HostEndpoint窄视口的样式列长内容，滚轮后底部文字/按钮进入视口，v1/v2及点击命中一致；无滚动列/hidden/显式scroll不额外包装；已有VM视觉拆分与根viewport针对性用例通过。随后正式工具和固定pin Chat重建、真实长会话最后气泡完整可读，并保留原六条数据。测试入口仍compiler rust.rs p060与runtime headless/view针对性测试，具体新测试由work建立，当前未运行。私有rqhost退出归父054T05独立生命周期设计，未混入本r6契约。

## 6. 测试设计

- 单测（rust.rs）：`arr[i]` 发射含 `__at_idx`；`i=-1`/越界返回 default；
  `Vec<Value>` 走既有 at_owned；快照型断言与现有 generator 测试同址。
- 单测（session.rs）：`DesktopInject::Pointer` 经 `apply_desktop_injects` 调用
  broker_pointer_*（mock client 记录 InputMsg）；handler outproc 分支发
  协商后的Action，并验证旧客户端不发送、失败不落占位组件。
- 端到端（os-054 候选，只读消费）：
  - calendar.exe 启动不 panic、窗口可用、渲染月视图；
  - iced 桌面 pointer 点击 notes「New note」→ 编辑器出现；chars 输入标题；
    handler NewNote 建条；
  - todo 输入框点击聚焦 + chars + Enter 建任务；
  - 反控：边界索引用例在旧发射下必 panic（保留为回归用例输入）。
- 既有门：`cargo nextest`（auto-ui 固定组，diff 交集面）；生成物编译
  （notes/chat/blog 任一）。

## 7. 验收标准

- AC-01（work）：native 生成码对越界/负索引不 panic，返回缺省；单测覆盖。
- AC-02（work）：iced 桌面 MCP pointer 注入可点击 outproc app 元素并产生
  可见状态变化（notes New note 打开编辑器）。
- AC-03（work）：`handler` 对 outproc app 经协议投递生效（notes NewNote 建条）；
  若协议面被证不可行，降级为「指针+键盘可用」并在 §10 记具名边界。
- AC-04（work）：calendar.exe 在 iced 桌面启动、窗口可用、无 panic；六 app
  既有构建零回归。
- AC-05（review）：独立上下文复核代码与实机证据，绑定 revision。
- AC-06（merge）：归档/清理/落地；os-054 组内工具链 pin 更新为交付提交。

## 8. 执行步骤

- [x] T-00（work）：定位 Index 发射全路径与 rqhost 客户端输入消费点，产出
  `docs/plans/evidence/060/decision.md`（边界索引发射策略 + handler 协议可行性）。
- [x] T-01（work）：Index 安全发射与正式 shim；F060-R6-01修复见round70永久运行控制，原Calendar生成/实机证据保留。
  panic 消失。
- [x] T-02（work）：`DesktopInject::Pointer` + MCP `action:"pointer"`；notes 点击实测。
- [x] T-03（work）：outproc handler 协议投递（或降级边界）+ 键盘聚焦实测。
- [x] T-04（work）：新main/949共同依赖整合验证待闭合；原六 app 回归/桌面实测/证据保留；F060-R6-01/02修复及准确六目标规范清单见round70。既有六app实机成功保留，最终SDK影响核对仍由merge负责。round67 sibling-store 导入修复于 round68 完成正式完整 Vue 消费验证；原生生成/runtime 不变边界复用。
- [x] T-05（review/merge）：独立复审、归档、工具链 pin 落地。

## 9. 复审记录

- stage: new | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: pass（授权已记录，证据来自 auto-os/PLAN-054 T-04 实机） | next: work/T-00

## 10. 待澄清事项

- handler 协议投递若需修改 rq 协议（握手/能力协商），先按 T-00 决策；不改变
  已交付 app 的既有行为，旧客户端忽略未知控制消息。
- 索引 shim 对非 Clone/Default 元素类型保留原发射并在该生成文件注记（编译门兜底）。


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: pass（T-01 完成） | task_ids: T-01 | code_commit: bcefa9b2 | evidence: `__at_idx<T: Clone+Default>(&v,i)` shim 注入 + `Expr::Index` 非 Value 目标改走该 shim（i<0→default，get→cloned/default）；calendar 以 patched 工具重生成（`auto build -r rust` exit 0，生成码含 `__at_idx`），`calendar.exe` 单独启动 exit 前 panic 消失（存活 5s+，仅因无桌面端点退出）；旧生成码 `dim[(mm-1) as usize]` 不再出现。 | next: T-02
- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | outcome: blocked（T-02 部分：投递已达 client，app 无响应） | task_ids: T-02 | code_commit: 4211d1cb（diag） | evidence: `DesktopInject::Pointer{phase,x,y,button}` + MCP `action:"pointer"`（click/down/up）；重建 ui-desktop 后实测：`[plan060] pointer Click (272,49) hit=Some(Wid(1))`、`down delivered=true`、`up delivered=true`——事件已投递给 owning broker client；但 notes app UI 无变化（多组坐标/「Create your first note」均无反应）。app 侧 `headless.rs::on_input` 具备 point_at+input_to_iced_events+surface.update 路径，需 app 侧可观测性（当前 app stderr 未进桌面日志）定位是坐标映射还是事件映射问题。 | blockers: app 侧输入消费需诊断（headless on_input 坐标/事件面）；not 阻塞 index 修复 | next: T-02 续（app 侧日志/回归）

## 9. 复审记录（追加）

- stage: work | plan_id: auto-ui/PLAN-060 | round: 1 | from: new | to: work | status: executing | outcome: pass（T-01）/ partial（T-02） | worktree: D:/autostack/.wt/ui-060/auto-ui | branch: codex/plan-060-dev | commit: 4211d1cb | next: T-02/T-03

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 2 | outcome: pass（T-02 输入投递打通，apps 侧消息已派发） | task_ids: T-02 | commit: 06ea326c + 后续 diag 提交 | evidence:
  - app 侧可观测性（`AUTO_PLAN060_DIAG` 落文件）打通后实测：`PointerPressed{wid:1,x:256,y:33}` 到达 app；网格扫描定位真实控件——`(440,24)`→`[NewNote]`、`(360,24)`→`[ToggleDarkMode]`、`(40,220)/(120,220)`→`[NewNote]`；即 **指针注入 → broker → app 进程 → iced 控件消息** 全链贯通。
  - 坐标口径：canvas = app + window rect（adopt 固定 `(16,16)`，surface `480x320`）；此前 (272,49)→app(256,33) 落在空白区（0 msg）并非通道坏。
  - 残留：`NewNote`/`ToggleDarkMode` 已派发但桌面窗帧无可见变化（截图前后一致）——疑 adopt 模式下输入后的新帧未推回桌面（frame 回传/sink）或 handler 无持久化；下一步定位 adopt 帧回传路径（desktop host on_frame/adopt sink）与 app push_frame。
  - 诊断代码为 env 门控（`plan060_diag`，缺省零开销），T-04 前决定保留或摘除。
 | next: T-02 收尾（帧回传）→ T-03

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 3 | outcome: blocked（第三处根因：native API 全部为 stub） | task_ids: T-02/T-03 | commit: 后续 diag 提交 | evidence:
  - 桌面侧确认收到并合成输入后的新帧：`[plan060] frame compose wid=1 frame_id=2/3`；app 侧 `FRAME rev=1`。输入→帧链路两端都通。
  - 但 NewNote/ToggleDarkMode 无可见状态变化 → 追到生成码：notes `list_notes() -> Vec::new()`、`create_note(...) -> Note::default()`、`update_note_r -> RepoResult::default()` 等**全部是 stub**（todo 同样：`list_todos -> LoadResult::default()`、`create_todo -> MutationResult::default()`）。即 a2r native profile 在未链接 back 时静默发射空实现，app 数据层不存在——这是所有用户任务不可能成功的真正根因（052 的「编译绿」未覆盖）。
  - 影响：054 T-04 的 8 app 真实任务被此缺陷阻断；060 的输入/索引修复本身正确且已实证。
  - 待用户裁定：是否把「native back API 链接（in-process 合并或 HTTP client）」纳入 060 范围（或另开计划）。
 | next: 等用户裁定（范围扩展）

## 11. 范围扩展（2026-11-05 用户裁定：并入本计划）

**新增根因 R3**：a2r native profile 对 `api: "rust"` 的 app 未生成真实数据层——
front 生成码里 `list_notes()->Vec::new()`、`create_note()->Note::default()`、
`update_note_r()->RepoResult::default()`（todo 同型）都是**静默 stub**；而
back 侧 `auto-notes-...-back` 是完整 axum/DB 实现。故 iced 桌面上任何
「新建/保存/读取」类用户任务不可能成功（054 T-04 被此阻断）。

**新增任务**（编号续接，原 T-04/T-05 顺延为 T-05/T-06 执行序）：

- T-04（work）：native API 真实链接。T-00 决策二选一：①in-process 链接
  （back 暴露 lib，front 调 `repository::*`）；②HTTP client（back 以 daemon
  起，front 按 api.at 生成请求，base URL 走 `<APP>_DAEMON` / pac daemon 声明）。
  优先②（与 vue 轨同构、与 013 daemon 纪律一致，且不改 back 结构）；①仅在②
  不可行时启用。生成器落点：`crates/auto-ui-compiler/src/ui_gen/rust.rs`
  （stub/prototype 发射位 ~2027/2149 起）+ `crates/auto-ui-tools/src/rust_ui.rs`
  （app 元数据/daemon 注入）+ 需要时 `auto run` 拉起 back 的 launch 协作。
- T-05（work）：六 app 回归 + os-054 候选桌面实测（notes 建条/按键、todo 建任务、
  calendar 月视图渲染），证据落 `docs/plans/evidence/060/`。
- T-06（review/merge）：独立复审、归档、工具链 pin 落地（auto-os/PLAN-054 T-07 消费）。

**新增验收**：
- AC-07：notes 在 iced 桌面「New note → 输入标题 → 列表出现该条」全链成功。
- AC-08：todo 在 iced 桌面「点击输入框 → 输入 → Enter → 列表出现任务」成功。
- AC-09：API 失败面显式（后端不可达时前端报错可见/状态可诊断），不得再静默 stub。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 4 | task_ids: T-04 | outcome: 定位完成（合并 API 层架构缺口），实现待续 | commit: 06ea326c..(diag) | evidence:
  - `generate_api_client`（rust_ui.rs:1585）在 merged 模式走 `generate_merged_api_client`；`merged_db_impl`（1459）对 `crate::types`/`super::`/`crate::api::` 引用**直接返回 None**（注释明写「015-notes 类结构型 db 不可吸收」），于是 endpoint 落 CRUD 脚手架、非 endpoint 导入（list_notes/create_note/update_note_r/get_note_r/delete_note/toggle_pin…）落 `generate_api_stubs`（空实现）→ 前端数据层不存在。
  - `merged_route_a_impl`（1422）只服务「无 db.at」的 app（companions + api_impl 内嵌）；有 db.at 的 app 无 in-process 真实现路径。
  - 诊断实测：`parse_api_module` 正常（`module=Some(28) endpoints=6`），证明 api.at 可解析，问题在 db 吸收门与导入包装。
  - 待实现（T-04 主体，下轮继续）：①`merged_back_modules`：types.at + db.at + db 引用的 companions（repository/p2probe）一并转译内嵌，类型经 `pub mod types` + `pub use types::*` 供根名使用；②按 db.rs 解析函数签名生成导入包装（i64↔i32、&str→String、结构/Vec 直传），仅覆盖 widget 实际导入名；③去掉重复根类型 struct（避免与 types 模块重名）；④notes 先落地，再 todo/calendar。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 5 | task_ids: T-04 | outcome: pass（数据层打通，UI 帧仍不刷新） | commits: 见下 | evidence:
  - **T-04 数据层落地**：`merged_absorb` 把 types/companions/db 一并转译内嵌（带 source_dir 的 `transpile_back_module_to_rs_with_proofs`，解决 companion `&str` 实参漏 `&` 的 E0308，另加机械 `rewrite_companion_str_args`）；front 根类型（i32）与 `types` 模块（i64）分存，边界用 `__db2ui_*`/`__ui2db_*` 垫片转换；按 db 签名生成委派（i64↔i32、&str/String、&[T]、结构/Vec/Option）。
  - 实测：notes 以新生成码编译通过；iced 桌面点击 `New note` 后 **sqlite `inbox.db` 新增 n-2 行**（python 直查：notes count 2），证明用户任务的数据链已真实工作。
  - **残留**：UI 仍显示 "No notes yet"（窗口不刷新新帧）；已加 `mark_shell_dirty()` 于三处 compose 臂（ComposeFrame/V2/Shared），重建桌面后仍不复现刷新——`take_shell_dirty` 消费点在 `desktop_protocol/host.rs:4670`（另一循环），需把 outproc broker 循环与该脏位连通，或直接在 broker 循环内触发 iced 重绘。
 | next: T-02 收尾（帧刷新接线）→ T-05 回归

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 6 | task_ids: T-02/T-04 | outcome: 定位帧不刷新机制 | evidence:
  - 桌面日志：点击后 `frame compose wid=1 frame_id=2/3`（帧确实到达并被合成），但窗口内容不变。
  - 构造点：`headless.rs::on_input` 先 `surface.update(&events)`（产出消息）→ `rev += 1` → 运行时 `push_frame()`；而消息派发（`component.on(msg)`）若发生在 push_frame 之后，则**帧渲染的是处理前的状态**，且此后再无追帧 → 窗口永远停在旧内容（数据层已变：`inbox.db` 新增行）。
  - 结论：T-02 收尾 = 输入消息派发与产帧顺序修正（先 dispatch 后 push，或 dispatch 后补一帧）；非渲染器/合成路径问题。
 | next: T-02 顺序修正 → 复测 notes/todo

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 7 | task_ids: T-02/T-04 | outcome: pass（notes 用户任务全链打通） | commits: 见下 | evidence:
  - 关键修复：api.at 的 `#[api]` 端点体（`return db.all_notes()` 等）转译为 `api_impl` 模块内嵌（去 `#[api]`/`pub type`、补 `->`、`post_process_db_rs`、剥 `impl ` 前缀、`use crate::types::*`）；端点委派优先 `api_impl::<fn>`、回落 `db::<fn>`，边界类型经 `__db2ui_*`/`__ui2db_*` 垫片。
  - 实测：notes 重新生成编译通过；iced 桌面点击 `New note` 后窗口渲染 **"Untitled / Saved / Pin note / Delete / Just now / Tag / Start writing..."** 编辑器（截图 `tmp/autoui-screenshot-1793889521044.png`），数据落 `inbox.db`（前轮 n-2 行），帧 dump 同步显示同样文本。**AC-07 达成**（notes 建条 + 编辑面出现）。
  - 附带：桌面窗被最小化时 screenshot 报 zero-size；`ShowWindow(SW_RESTORE)` 可恢复（验收脚本需先恢复窗口）。
 | next: T-05 回归（todo/calendar、键入文本、六 app）

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 8 | task_ids: T-05 | outcome: partial（文本输入未落字） | evidence:
  - 指针定焦成功：点击编辑器正文后 `CharTyped` 到达 app 并派发 `EditorPanel(EditBody(""))`（说明 text_input 已聚焦、on_input 在跑），但每个字符的值都是空串 → **字符插入没有生效**（`ime_commit "hello"` 同样 `EditBody("")`）。
  - 帧侧正常：每字符 `FRAME rev` 前进（rev 4→8），说明组件在更新；问题在 iced text_input 的事件形态（`KeyPressed{key:Character, text:Some}` 未被接受插入）或 headless 事件通道细节。
  - 备注：notes 窗口 surface 480x320，窗口底部 y≈336 之外点击不命中（验收坐标需约束在 surface 内）；桌面窗被最小化时 screenshot 报 zero-size，需先 `ShowWindow(SW_RESTORE)`。
 | next: T-05 续（text_input 插入事件形态定位：对照 iced 0.14 text_input on_event 的 KeyPressed/InputMethod 分支）

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 9 | task_ids: T-05 | outcome: 根因定位（输入文本载荷空） | evidence:
  - 生成视图层：`.on_change(EditorPanelMsg::EditBody("".to_string()))`——`View::input/textarea` 的 `on_change(msg: M)` 只接收**固定消息值**，文本不入消息；投影层（`renderer/projection.rs:1352`）在 on_input 里写 `INPUT_TEXT` thread-local（`last_input_text()` 读面），供生成 `on()` 读取。
  - 但 notes 的 `EditorPanelMsg::EditBody(v) => self.store.on(NotesStoreMsg::EditBody(v.clone()))` 用的是载荷 v（恒空串），没有走 `last_input_text()` → 落字恒空。
  - 修正方向：生成器对「绑定输入的 String 载荷变体」在 `on()` 分支注入 `let v = last_input_text();`（或把该变体载荷变量改写为该读面），使 handler 体拿到真实文本；与 PLAN-052/025 既有的 last_input_text 契约一致。
 | next: T-05 生成器修正 + 复测键入

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 10 | task_ids: T-05 | outcome: 注入已落地，读面在 outproc 路径未写 | evidence:
  - 生成器：新增 `input_variants` 预扫（`collect_input_event_handlers`，on() 先于 view() 生成的顺序问题）+ 带 String 载荷且绑定输入的臂注入 `let v = last_input_text();`——notes 生成码已见该行。
  - 实测仍 `EditBody("")`：`last_input_text()` 读 `INPUT_TEXT` thread-local，而该写入点在 **AbstractView 投影**（`renderer/projection.rs:1356`，inproc 轨）；outproc app 的 `HeadlessSurface` 走 `view.into_iced()` 另一转换路径，未写 INPUT_TEXT → 读面恒空。
  - 下一步：在 HeadlessSurface 的 View→iced 转换（input/textarea 的 on_input 接线）同点写 INPUT_TEXT（与 projection 同律），复测键入。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 11 | task_ids: T-05 | outcome: pass（键入文本打通） | evidence:
  - 关键：`input_variants` 预扫 + `last_input_text()` 覆盖落地后，iced 桌面 notes 正文键入 **"hello"** 真实显示（截图 `tmp/autoui-screenshot-1793891155440.png`：Untitled / Unsaved changes / Save / hello 光标），此前 `EditBody("")` 的空载荷问题消失。
  - 数据面同步（inbox.db 累计 n-3 行）。行内保存按钮坐标仍待校准（点击落到标题输入框），但「建条 + 键入」AC-07 全链已成立。
 | next: T-05 余项（todo 输入/calendar、六 app 回归）→ T-06 review

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 12 | task_ids: T-05 | outcome: 分型发现（inproc 窗不走 broker 指针） | evidence:
  - todo 在 iced 桌面以 **inproc VM** 宿主（日志 `promote 013-todo app=7 → wid 1`，无 `launch_app(outproc-native)`、无独立进程）；验收指针 inject 走 broker 臂 → `pointer down delivered=false`（无 broker client），窗口由桌面自身 VM 渲染，点击不达。
  - 分型结论：outproc（notes 等，broker client）用 **pointer inject**；inproc（todo/calendar 等 VM 宿主）应用 **handler inject**（`component.call_handler` 真实消费臂，既有设定即此），或在 shell 侧补 inproc 指针分支。
  - 本计划 T-05 以 outproc pointer + inproc handler 两路验收；inproc 指针分支列为后续可选增强（不影响 054 T-04 任务可完成性）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 13 | task_ids: T-05 | outcome: inproc handler 注入有效；todo 数据写入但 UI 未刷新 | evidence:
  - `handler app=013-todo StoreInput arg=alpha` → 界面输入框显示 "alpha"；`AddTodo` → 日志 `[CALL_HANDLER] widget=App event_name=AddTodo fn_name=handler_App_AddTodo args=[]`（真实消费臂）。
  - 数据面：`todos.db` 出现 `(1,'alpha',0)`——任务落库；但 UI 未刷新（input 未清空、列表未出现），与 store 的 `create_todo(...).ok` 响应判定/刷新路径相关（VM 宿主 + back 契约），非输入通道。
  - 结论：inproc（VM）验收走 handler 注入可行（StoreInput/AddTodo 均执行）；AC-08 的「列表出现」取决于 create_todo 响应契约与列表刷新，列为后续。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 14 | task_ids: T-05 | outcome: back 契约验证通过；VM 端 UI 刷新仍待查 | evidence:
  - 手动起 todo back（`auto-todo-c1a8adb9-back.exe`，监听 **8080**，非 pac back_port 17815——前端默认 8080）：`GET /api/todos` → `{"ok":true,"err_code":"","todos":[…]}`；`POST /api/todos {"text":"gamma"}` → `{"ok":true,…,"todo":{…}}`。
  - handler 驱动 `StoreInput("beta")+AddTodo` 后 `todos.db` 增 `(2,'beta',0)`（数据真实写入），但 UI 输入框仍显示 beta、列表未出现/无失败横幅——需查 VM 侧 `create_todo` 响应→`MutationResult.ok` 映射或 handler 后列表刷新（`load_todos`）链。
  - 结论：todo 数据面已通；VM 端「响应→状态→重渲染」是最后一段，列 T-05 尾项。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 15 | task_ids: T-05 | outcome: 根因（VM async API vs call_handler 注入） | evidence:
  - 日志：`[VM-PARKED] handler_App_Init parked … resumed to completion (1.9s)`——VM 的 HTTP API 是**异步挂起续跑**模型；而注入的 `AddTodo` 只出现 `[CALL_HANDLER]`，**没有 parked/resume** → `create_todo` 在 handler 内同步返回默认值（r.ok=false），HTTP 随后台完成（行仍写入：todos 4 行 alpha/beta/gamma/delta），所以「数据进库但 UI 不清输入/不刷列表」。
  - 结论：inproc VM app 的验收驱动应注入**消息/事件**（走组件正常事件循环，HTTP 会挂起续跑），而不是 `call_handler`（同步直调，绕过挂起）。下一步：`autoui_desktop` 增加 `action:"msg"`（app + msg 名 + 载荷）→ `component.on(msg)` 或等价事件泵；再验 todo 列表刷新。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 16 | task_ids: T-05 | outcome: msg 注入已落地；VM 挂起仍只发生在 boot Init | evidence:
  - 新增 `DesktopInject::Msg` + MCP `action:"msg"`（`component.on(DynamicMessage::Typed{…})`，走组件事件循环）。
  - 实测 todo：`msg StoreInput("epsilon")` + `msg AddTodo` → 输入框显示 epsilon、`todos.db` 增 `(5,'epsilon',0)`；但日志仍只有 `VM-PARKED handler_App_Init parked/resumed (3.3s)`，**AddTodo 不挂起**、UI 不清输入/不刷列表。
  - 结论：VM 的 HTTP 挂起续跑仅在 boot Init 的驱动路径生效；后续消息（on/msg 或 call_handler）走同步执行、HTTP 异步旁路。修复需 VM 宿主把**任意** handler 纳入挂起续跑驱动（auto-lang VM 侧集成），超出本计划 060 范围——列为 054 的具名前置（VM inproc 宿主异步事件循环）。
  - 060 已完成面：索引语义（AC-01）、outproc 指针/键盘（AC-02/07 notes 全链）、native API 层（AC-07 数据链）。VM inproc 任务（AC-08 todo）待该前置。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 17 | outcome: waiting（VM 挂起修复转 auto-lang/PLAN-949） | evidence: 用户裁定继续补 VM 异步事件循环 → auto-lang PLAN-949（`6fae9d3c8`，worktree `.wt/lang-949/auto-lang`）；060 的 T-05 AC-08 待 949 交付后复验，其余已完成面保持。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 18 | outcome: 新缺口（outproc 指针下行） | evidence: 054 date: 2026-11-06 round 101 — 验收指针 canvas=屏幕÷2 标定成功（`hit=Some(Wid(1))`），但 016-calendar（outproc-native）在命中点击后无反应，`[plan060] pointer` 无 delivered/App 反应 ⇒ outproc 协议缺指针下行（现仅键盘/文本面）。下一步：DesktopProtocol 增 Pointer 消息并接线（inproc 已有）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 19 | task_ids: T-07 | outcome: Action 线格式已落，child 派发待接线 | evidence:
  - `message.rs`：新增 `InputMsg::Action { wid, name, arg }`（tag 11，encode+decode 完成，提交见 auto-ui 分支）。
  - 待接线：child 侧 `headless.rs::on_input` 的泛型 `C: Component` 无法构造 `C::Msg`——需在具体 DynamicComponent 的 Surface 实现处派发 `component.on(DynamicMessage::String(name))`；宿主 `session::Msg` 臂对 broker client 转发 Action（当前只对 in-process 组件 on）。
  - 复验目标：calendar `NextMonth` 翻月、`SelectDay` 选日。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 20 | task_ids: T-07 | outcome: Action 下行接通（宿主+child）；旧 app 二进制不兼容导致窗口关闭 | evidence:
  - 实现：`Component::dispatch_named` 默认方法 + `DynamicComponent` 实现（`call_handler_for("App", name, …)`）；`RqProjector::on_input`/`headless::on_input` 消费 `InputMsg::Action`；宿主 `shell_state` 的 `Msg` 臂先对 broker client 发 Action，未命中再走 in-process `on()`。非穷尽 match 已在 editor_frame/input_to_iced_events 补臂。
  - 实测：`msg 016-calendar NextMonth` 后 **calendar 窗口消失**；日志无 `[plan060] action` 记录。根因：`calendar.exe` 是旧构建（`auto-calendar-85dd4543`），其 auto-ui 不含 tag 11 → 解码失败断链 → 宿主回收窗口。
  - 下一步：app 侧用新 auto-ui 重建（协议版本升级）；或在握手加 capability，未声明支持时不下发 Action（向后兼容）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 21 | task_ids: T-07 | outcome: dispatch 入口修正；宿主转发未命中（待日志） | evidence:
  - `DynamicComponent::dispatch_named` 改走 `bridge.call_handler(name, args)`（与 `DynamicMessage::String` 同源 root state 绑定）——此前用 `call_handler_for("App", name, 0, …)` 的 state id 0 是错的。
  - calendar 二进制已用新 auto-ui/auto-lang（组树 ui-060）重建并发布；`msg NextMonth` 后窗口存活（协议兼容），但日志无 `[plan060] action`、月份未变 ⇒ 宿主 `Msg` 臂的 broker 转发未命中（疑似 `wm.wins[].app == app_id` 不成立或 owns_wid 不匹配）。
  - 下一步：在宿主 Msg 臂加诊断（app_id/wid/owns），确认为何未转发。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 22 | task_ids: T-07 | outcome: 宿主发送成功（delivered=true），child 侧未进入 Action 分支 | evidence:
  - 诊断：`[plan060] action NextMonth sent wid=1 delivered=true` —— 宿主已把 `InputMsg::Action` 写入 broker 传输。
  - 但 child 侧无 `[plan060] child action …` 日志（该日志位于 `RqProjector::on_input` 的 Action 分支）⇒ child 的输入泵未把 Action 投到该 Surface，或该 app 的 Surface 不是 RqProjector 路径。
  - 已确认部署的 calendar.exe 含新标记（`child action` 字符串在二进制内、时间 02:00:40）。
  - 下一步：查 child 侧 Input 消息路由（client_entry/shell_client 的 face→surface 分发表），把 Action 归入与 pointer/key 同一投递面。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 23 | task_ids: T-07 | outcome: 诊断面修正——child stderr 未归档到桌面日志，之前"无 child action 日志"不构成未送达证据 | evidence:
  - `endpoint.rs:514`：`(Active, ProtocolMsg::Input(input)) => self.session.on_input(&input)`——Action 与 pointer 同通道同入口，投递路径无疑。
  - client_entry 的 Commands 臂用 `RqProjector` + `client_runtime::run_client_session`；`syslog!` 双写 `eprintln`（syslog.rs:183）——child 诊断只进 stderr。
  - 之前看到的 `[plan060] frame2 …` 来自**桌面进程内的 mini VM**，不是 outproc child；`launch_app(outproc-native)` spawn 的 child stderr 未进入桌面日志 ⇒ "无 child action 日志"不能证明未送达。
  - 下一步：把 outproc-native child 的 stderr 落文件（spawn 侧 pipe→日志文件），再判定 Action 是否被 child 消费；同时用 `SelectDay` 观测高亮变化。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 24 | task_ids: T-07 | outcome: 决定性定位——child 收到 Action，但 a2r 生成的 Rust 组件未实现 dispatch_named | evidence:
  - 部署二进制确认含标记（`child endpoint input`/`child action`，02:11:19 重建）。
  - 运行日志：`[plan060] child endpoint input Discriminant(10)`（= Action 变体）→ 输入**已到达 child 的 HostEndpoint.on_message**，随后 `session.on_input` 分派。
  - 但无 `[plan060] child action` 日志：child 的 Surface 是 **HeadlessFrameSource**（client_entry 的 Commands 臂之外的装配），且 calendar 是 **rust profile**（a2r 生成组件，非 VM DynamicComponent）——`Component::dispatch_named` 走默认实现返回 false。
  - 结论：Action 下行协议链路（宿主→child→Surface）已通；缺的是**生成器侧为 a2r 组件生成 dispatch_named**（消息名 → 生成 Msg 变体）。
  - 下一步：在 `crates/auto-ui-tools/src/rust_ui.rs` 生成 `impl Component { fn dispatch_named(...) }`，把 handler 名映射到生成的消息枚举并派发；再复验 calendar `NextMonth`。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 25 | task_ids: T-07 | outcome: 生成器已加 dispatch_named 发射，但 calendar 生成产物未包含（发射点未命中） | evidence:
  - `auto-ui-compiler/src/ui_gen/rust.rs::generate_component_impl` 新增 emission（无参/单 str 参变体 → `self.on(Msg::Variant)`），提交 `25b7a805`。
  - 重建 `auto-ui-tool`（02:30:06）后 `auto build -r rust`：生成产物 `main.rs` **仍无 dispatch_named**，重链接后二进制亦无该串。
  - 发现旁路问题：`rust-workspace/Cargo.toml` 的 `auto-web` path 指向 `../../../../auto-web`（os-054 兄弟）而非 ui-060 组树——与 auto-lang-core(ui-060) 的 auto-web-contract 冲突（links 重复）；已临时改写该行绕过（生成器每次 build 会重写，需在生成器侧修正路径）。
  - 下一步：确认 calendar 组件走的是哪个 emitter（是否 `generate_component_impl` 路径、message_variants 是否在发射时已填充）；必要时在同一文件内加诊断打印。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 28 | task_ids: T-07 | outcome: calendar 重生成成功且含 dispatch_named；child 的 Surface 仍未命中我们的 Action 分支 | evidence:
  - **关键修复**：`auto build -r rust` 必须统一组树（AUTO_LANG_ROOT/AUTO_UI_ROOT/AUTO_MAN_ROOT 全指 `.wt/os-054/*`），否则 auto-web(兄弟) 与 auto-lang-core(ui-060) 的 auto-web-contract links 双包冲突 → 统一后 `[tool] build rust (Completed)`，生成 `main.rs` 含 `fn dispatch_named`（App: PrevMonth/NextMonth/Today/…；CalendarStore: Init/NextMonth/SelectDay/…）。
  - 但运行期：`[plan060] child endpoint input Discriminant(10)` 后无 `[plan060] child action`/`child(headless) action` 日志 ⇒ child 的 `endpoint.session` 具体类型不是 RqProjector/HeadlessFrameSource 中我们打过日志的路径（候选：`host.rs:549` 的 Surface 或 broker/demo 变体）。
  - 下一步：读 `client_entry`/`host.rs` 里实际构造的 Surface，把 Action 分支加进去（或统一到 `Component::dispatch_named` 的单一出口）。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 29 | task_ids: T-07 | outcome: 强候选根因——像素臂 `PixelsNoopSource::on_input` 是 no-op | evidence:
  - `pixels.rs:83`：`impl FrameSource for PixelsNoopSource { fn on_input(&mut self, _input: &InputMsg) {} }` —— v1.3 independent（像素）臂的 child 表面**显式丢弃所有输入**。
  - 若 calendar 客户端裁决为 Pixels 帧模式，endpoint 的 `session` 即 `PixelsNoopSource`：Action（及 pointer/key）到达 endpoint（日志 Discriminant(10) 已证）后被丢弃，与「无 child action 日志 / 指针无反应」全部吻合。
  - 下一步：确认 calendar 的 `frame_mode`（Hello/activate 裁决）；若为 Pixels，则要么在该臂把输入转发给真实组件（需 PixelsNoopSource 持有组件/投影器），要么把该 app 裁决为 Commands 档。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 30 | task_ids: T-07 | outcome: 验证回路被构建产物/publish 漂移污染（必须先钉产物新鲜度） | evidence:
  - 部署的 `bin/debug/calendar.exe` = 03:09:03；含旧 `child endpoint input` 标记、**不含**最新 `mode={:?} state={:?}`；而 `c2e02416/debug/calendar-85dd4543.exe`（手建 target）也无该标记。
  - ⇒ 本轮及前几轮的「无 child action 日志」不可信：运行的二进制不是最新 auto-ui 编出的。
  - 正确回路：更新组树后必须走 **`auto build -r rust`**（它负责 publish 到 `bin/debug/`），不要手工从 guessed target 目录拷贝；每次验证前先比对二进制标记/时间。
  - 下一步：固定流程（组树→auto build→校验标记→运行），再判定 Action 是否被消费。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 31 | task_ids: T-07 | outcome: native headless Action root cause fixed, regression red→green | evidence:
  - 2026-11-06 用户授权继续解决054全部前置。native remote入口确实创建泛型HeadlessFrameSource；此前Action分支只在VM专用McpHeadlessSource，泛型源的input_to_iced_events(Action)=[]导致提前返回。修复在泛型源先dispatch_named，成功时rev+1；未知消息/缺参数不推进。
  - permanent case `ui::desktop_protocol::headless::tests::p060_native_headless_action_updates_component_and_frame_revision`：真实native Component经FrameSource输入；未知/缺参数对照保持状态与revision，合法SetLabel改变状态并产出包含after的帧。修复前0/1（revision left=0/right=1），修复后 `cargo test -p auto-ui-runtime --lib p060_native_headless_action_updates_component_and_frame_revision` exit=0，1 passed/0 failed/0 ignored，0.12s。日志 D:/autostack/plan-artifacts/plan-054/t04/headless-action-green.log；不把结构测试等同桌面业务通过。
  - 构建漂移旧结论纠正：源码格式串mode={:?}编译为独立字面量，calendar发布物确含` mode=`/` state=`。published SHA=923f77a304ba26a820aa325b2b7398baa01a73bcc43f763f93b4c2cee9e17b9b与generation.json一致；手建target SHA不同属于未publish，且r65运行早于发布物mtime。下一步消费本修复→auto build→核对artifact/receipt/process identity→实机NextMonth与事件任务。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 32 | outcome: follow-up input/lifecycle defects fixed with focused checks | evidence:
  - 054实机已证明calendar NextMonth November→December、新建跨日中文事件落盘、关闭重开可读、真实输入编辑落manifest rev2；产物三方hash与进程路径绑定，原Surface/构建阻塞已解除（详054 round123）。
  - native named文本：生成on(input)会读last_input_text覆盖typed载荷；dispatch_named仅对input_variants先store_input_text(a)，普通String消息不污染输入口。`cargo test -p auto-ui-compiler --lib p060_named_input_dispatch_stages_text_for_input_handler` 1 passed/0 ignored（named-input-codegen.log）。
  - Commands坐标：broker painter在客户区原坐标重放，不按WM外框拉伸。broker_pointer_down/up改用同一映射：普通窗减TITLEBAR_H，壳伪窗不减，标题栏/外沿不向child投递；Commands不再错误按surface/outer比例缩放；Pixels既有分支保持。p060_command_pointer_matches_unscaled_client_canvas验证三种WM尺寸、header拒绝/外点拒绝/伪窗对照。
  - 重开身份：launch_app_outproc的未认领筛选原先把缺失WM窗也视作registry_id=None，可能认领尚未回收的旧client/wid。改为WM窗实际存在且registry_id缺席；永久真子进程测试launch_app_outproc_lands_window增加WM窗移除但client仍驻留→再次launch→新wid/正确registry_id。
  - Ctrl+A：实机编辑时字母VK被vk_to_named过滤；补Character键身份后回归仍红（bxefore≠x），定位iced text_input读内部ModifiersChanged缓存而不只读KeyPressed.modifiers。按压/释放先同步修饰态，CharTyped同步空修饰态，字母/数字键保留身份但不重复发文本。真实HeadlessSurface input回归焦点→Ctrl+A→CharTyped(x) now x，非仅事件结构断言。
  - `cargo test -p auto-ui-runtime --lib p060` 最终3 passed/0 failed/0 ignored（input-followup-tests2.log）；随后 `cargo test -p auto-ui-runtime --lib launch_app_outproc_lands_window -- --test-threads=1` 1 passed/0 ignored/0.20s（reopen-registry-test.log）。MCP schema补齐msg/app任意注册ID及pointer参数，避免工具声明与消费面不符。下一步统一消费+重建tool/desktop/calendar，实机复验这批修复；这些单测不代替054业务验收。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 33 | outcome: generated-code freshness boundary corrected | evidence: front_generator_identity had a documented boundary: unchanged .at inputs plus unchanged tool version/producer key reused old main.rs after emitter changes. Bump FRONT_LAYOUT_VERSION v2→v3 for 060 dispatch_named/input staging emission, using the existing receipt mismatch→code regeneration gate. This avoids deleting app members or editing app source just to force refresh. Verification pending: rebuilt official tool must regenerate unchanged calendar, main.rs must stage FormTitle arg, receipt must record v3 and published artifact hashes must match. 054 T-05 separately retains observed close-resource gap: CloseWindow removes WM/App bookkeeping but does not yet send broker endpoint Close; old child lived until host exit. Do not call this final060/054 pass.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 34 | outcome: official regeneration and live input/reopen/screenshot proof | evidence:
  - 0d444808正式tool重建9.22s exit0；calendar未改源自动重生成，receipt front-layout/v3、main.rs input dispatch staging在场，build Completed/exit0，三方artifact hash0863e1c8dd12ff98b3d3d4d30326231ed7c6e91fd50ed5f5dca997b1b540e531。calendar实机正常坐标编辑，named文本正确保存rev3；重开消息路由到新wid2；真实DeleteEvent→rev4/events[]，再重开仍空且身份hash不变。详054 round125。
  - 新截图反例：共享request可被零尺寸的虚拟AppState先消费，误判宿主未布局。63a1f9c1改用现有实际window::size守卫，host重建exit0。原生child在场4/4连拍成功，最小化负控正确拒绝，恢复成功。照片t04/screenshots/r126-delete-reopen-3.png，hash739ffe76d01e6f1dcd18958ed0c1966a0d54012cc8973f5ff46dca24e10af74a。
  - close资源反例：桌面20868关闭wid1后calendar47508仍存活。正在补既有Close→ExitRequest→Reclaim接线与真子进程退出/旁观者对照，测试待完成。不改app业务源码，不把以上当054整体或060最终pass。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 35 | outcome: close handshake repair verified | evidence:
  - Added broker_close_window(wid), called before DC::CloseWindow removes WM/App records; sends existing HostEndpoint.close only to the owning primary client, leaving reclamation/buffer release to ExitRequest/EOF pump. Real-child regression initially failed (closed child still alive after5s), exposing in-flight FrameAck racing Close.
  - AppEndpoint Closing now tolerates FrameAck/BitmapAck from frames already sent; waits for BufferRelease instead of treating late ACK as WrongState and reconnecting. Existing host_close_then_exit_handshake_then_reclaim now asserts both ACKs retain Closing then normal reclaim. `cargo test -p auto-ui-runtime --lib host_close_then_exit_handshake_then_reclaim` 1 passed; `cargo test -p auto-ui-runtime --lib launch_app_outproc_lands_window -- --test-threads=1` 1 passed/0 ignored/0.28s, includes stale WM reopen, clean closed-child process exit, unrelated first child still alive. Logs close-ack-race-test.log / broker-close-test2.log. Live host/calendar rebuild still pending.
  - Independent054 Charts probe: UI import actualChineseCSV parks/resumes68.9ms but headerRows=[]/keysText=0, not a pass. Direct isolated GET localhost2998/apps/024-charts/api/v1/status returns503 session failed to load(degraded); existing log::error lacks visible desktop subscriber. Add stderr duplicate of existing source-load diagnostic to expose exact cause on next rebuilt host. No app source or data written by this probe; Charts task remains open.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 36 | outcome: live close passed; backend const loading repaired | evidence:
  - f835f1cc消费后host重建exit0/47.11s，calendar正式auto build exit0/Completed/1m01s，target/published/receipt三方hash54c18330d80dceb4c663ff5fef139535271ee55a1e1eb4e430b0d721f01fc6f3。实机desktop13556，calendar45696/wid2：bus close后WaitForExit5s返回true、ExitCode0、DesktopAlive=true，旁观Charts窗口仍可snapshot。证明正常协议关闭，不靠kill/宿主退出回收。
  - 新stderr诊断定位Charts503：`back chain stmt compile failed: Undefined variable: DEC_ERR_SYNTAX`。back_proxy::load_back_session仅收集StoreKind::Var，而语言已支持模块级Const为全局。把Const纳入全局登记与__module_init，保持Let既有局部语义；不修改app业务源。
  - permanent HTTP test p054_back_proxy_initializes_imported_constants_and_preserves_vars：临时多模块服务导入整型37/负小数-2.25/字符串stable三常量，真实GET断言值；var hits跨请求count38→39保持。`cargo test -p auto-ui-runtime --lib p054_back_proxy_initializes_imported_constants_and_preserves_vars -- --test-threads=1` 1 passed/0 ignored/0.06s（t04/back-const-test.log）。下一步消费/重建host，复验Charts真实CSV导入；尚不能据这个小夹具签Charts业务通过。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 37 | outcome: whole-body string binding aligned with standalone HTTP | evidence:
  - After const fix, Charts session starts and receives POST, but import returns400 `missing param payload`; front shows0 rows/cols and remains not accepted. Standalone auto-web::http_server::bind_api_args_by_name documents whole-body single-param tolerance after path/body/query binding; desktop proxy omitted it.
  - Add fallback only for a single declared str parameter with nonempty UTF-8 body, after named path/body/query and before metadata convention. JSON objects, arrays, string shells and raw text preserve exact body bytes; non-string missing parameters stay400. No app source changes or synthetic UI result writes.
  - HTTP regression p054_back_proxy_single_string_body_preserves_payload_and_named_precedence covers CJK/path/decimal object, array, raw+&#% text, JSON-string shell, named body precedence, query precedence and non-string rejection. `cargo test -p auto-ui-runtime --lib p054_back_proxy_single_string_body_preserves_payload_and_named_precedence -- --test-threads=1` exit0/1 passed/0 ignored/0.05s (t04/back-body-test.log). Next official host rebuild + real Charts import replay; still no Charts task pass.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 38 | outcome: Charts real import path verified after adapter repairs | evidence: 789a53fb消费到os-054，host重建exit0/31.49s，desktop26988（t04/desktop-r129.*）。从真实桌面入口launch024-charts，Msg CsvPathChanged(中文目录-图表/月度数据.csv)+DoImport后VM handler真实park/resume；per-app snapshot的datasetLoaded=true，keysText=[月份,销量,利润]，headerRows/recs/rowsText均3条，statusErr=false，statusMsg已导入3行×3列。全文t04/charts-after-body.txt。这是实际用户CSV，不是LoadSample；列映射/小数柱图/保存重开/SVG仍待完成，不把导入成功扩算整项通过。

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 39 | outcome: negotiated input compatibility repaired; delivered053 prerequisite integrated | evidence:
  - 57466e63 closes the named-input legacy disconnect: Hello advertises input-action/1 in its existing extensible capability tail; BrokerClient refuses Action without this exact capability or window ownership. Desktop Msg and root Handler use one broker path; unsupported/failed sends never execute the rqhost placeholder. Root widget App supported; other broker widget targets explicitly unsupported. Existing inproc widget semantics retained. This realizes the optional root handler design with existing InputMsg::Action, not the originally sketched new ControlMsg::HandlerCall. No arbitrary subwidget RPC claimed.
  - Real named-pipe test verifies legacy no-send, different-window no-send, new-peer CJK delivery, and old keyboard traffic still alive. Existing native component Action/frame, Ctrl+A replacement, and coordinate tests pass. Final combined cargo test -p auto-ui-runtime --lib p060_ exit0/5 passed/0 ignored/0.12s after053 integration (t04/action-final-test-r130.log). Separate Hello advertisement1 passed/0 ignored. New canonical SD-02 file documents actual protocol and evidence limits.
  - Dependency audit corrected an assumption: 9a94d568 (delivered UI053) was NOT an ancestor of060/consumer789a53fb; Lang61207576b IS present in949 and consumer. Vue scratch initial build exited0 but registry contained1/8 (Charts), so it is not an eight-app pass. Cherry-picked delivered053 into060 as644d8366; sole back_proxy conflict preserves060 strict UTF8/typed single-str binding. Two existing053 real HTTP regressions pass/0 ignored/0.69s (053-integration-test-r130.log), plus the5 p060 tests above. Consumer detached644d8366, official auto-ui-tool build exit0/1m01s. Current live iced host26988 remains789a53fb; new capability delivery not yet live-validated and native apps must rebuild before new-host acceptance.
  - Windows Sky screenshot displayed lock screen, so stopped native input and requested user unlock. Independent work continues: five native app builds exit0 and receipt/published/Cargo SHA matches; Vue scratch copies1326 source/vendor files hash-equal, physical deps. Per-app Vue official builds underway to materialize each app-owned API client before rerunning the8-app host. All raw evidence under plan-054/t04, no app business source changed. No final review/merge or054 completion claimed.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 40 | outcome: native Hello positional-tail regression repaired; live consumer replay pending | evidence:
  - PLAN054 full Vue eight-app task coverage has progressed on644d8366 (054 round132), but new native Notes launch correctly failed rather than count fallback success. Diagnostic child29096 exited0 with HostLost/revision0; host43612 only reported incubation budget failure. Add bounded attach protocol/transport error diagnostics4bbb59b7 (no message payload dump). Official diagnostic host build exit0/37.46s; host36800 SHA258e4ea5f8a9f6901717ffc1a2c379699f68c0d4f305946de3b61fc49d9989d3, true launch015-notes explicitly yields transport TooShort then child HostLost. Raw logs under plan054/t04/desktop-r138.*.
  - Root cause is Hello optional-tail framing: encoder omitted the surface-count field when surfaces=[] but still emitted nonempty theme_caps. Decoder always consumes the surface tail first; input-action/1 now makes this broken single-surface path unavoidable. Structured endpoint/memory transport tests did not serialize this combination; do not reuse those green tests as wire coverage.
  - New p060_hello_capability_tail_round_trips_without_extra_surfaces exercises actual ProtocolMsg encode/decode across empty/one surface × empty/one/two capabilities (including CJK title). Before fix exit101/1 failed/0 ignored, precise TooShort at surfaces=[]/input-action/1; hello-tail-red-r138.log SHA7823badf6e22cb7d6c4e5f9b6ae38c28c310173760a0ccca34a2af174d667a6e. Fix bc238d28 writes zero preceding surface count whenever a later capability tail exists, preserving old no-tail bytes when both empty. No new wire version/tag or permissive decoder fallback.
  - cargo test -p auto-ui-runtime --lib p060_ -- --test-threads=1 exit0/6 passed/0 failed/0 ignored/0.23s (build55.79s), retains named-pipe capability/ownership, native frame/action, Ctrl+A and coordinate checks; hello-tail-green-r138.log SHA604e7f8e2503718fc0aa5aebe6f450fdf8bea3489dbec3363bfda62f9bca82d9. Consumer UI detachedbc238d28. Official Notes rebuild in flight session77780, notes-hello-fix-r138-build.log; no native attach/business pass yet. Current native host diagnostic env AUTO_OUTPROC_STDERR=1 must be disabled before native Chat credentials flow; Vue secure Rust Chat test did not traverse VM handler diagnostics and no live credentials were emitted to evidence.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 41 | outcome: repaired Hello consumed by real native Notes desktop task | evidence: PLAN054 round133 records official Notes rebuild exit0/Completed (Cargo43.59s), receipt/published/target SHA a10b6a49773b73490db1c65fa16c85ae5e5496c3a1911f213cae5ab4157b9d68; actual desktop launch→adopt OK caps=[input-action/1]→wid2 frames. Root named NewNote, real child pointer+chars input, Chinese title/body+Save→n-6 rev2, close wid2→child49744 gone, launch→wid3/child2460 restores exact title/body, next input+Save→same n-6 rev3. Independent SQLite oracle SHA47ee4581df9ea822f229904b8c0a3c02686c6abf6f40b05a6de575530695f8c3 and screenshot SHA4b9696e0ae52368dcac5a5a66208cf88608875f2cda911ae89e78d1564dc55fb under plan054/t04. Current host4bbb decoder unchanged bybc238; childbc238 fixes emitted wire. Remaining5native official builds sequential session50975, no global app pass or final060 review claimed. Existing Vue evidence unaffected by wire-only fix; final SDK drift audit still required.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 42 | outcome: native mini launch invariant, table loops/value text and clickable dynamic text repaired and consumed | evidence:
  - 5255cbe0 prevents convention-native apps from hatching/promoting interpreted minis, and rechecks stale minis via normal launch decision. Real convention-exe red regression 1failed; final runtime p060_ 7passed/0ignored/.33s, existing hatch_ 2passed/.07s. Native31244/5255cbe0 now shows Todo placeholder until actual native launch. Details, exact red/green hashes and host SHA in PLAN054 round134; no VM-promotion task counted as native.
  - 73758365 flattens direct table-row For cells, uses row preset without cell flex, and displays Value text via __at_str. v3->v4 freshness. Real compiler AST regression red1failed then p060_ 2passed/.05s plus existing row-for1passed/.07s. Official tool/Database v4 build exit0; actual native two-column SQL displays 张三100.5/李四250.75 horizontally without JSON quotes, screenshot SHAd8334e0c6940b515405a332443cb5438897b8474bfbfcb6a0d41b50e38dd2b02. Long root-path soft-wrap duplication remains unlocalized; do not count solved.
  - 5e175f3c preserves text onclick through typed MouseArea and conditional style through text shortcut; layout v5. Correct fixture behavioral red missing hit target, then compiler p060_3passed/0ignored/.03s (red2 SHAd782902ecd0b647bc5b41cc9bcff8c93a1a629519d3ef474a3e70249de1cabaf; finalgreen5 SHAc5bc41784dce341cbc5550818220d93898f6ca286e400fb1fb2378a804ef6f22). Earlier malformed fixture/compiler/intermediate shortcut failures are not green evidence. Official v5 tool exit0/1m21s and native Todo build exit0/Cargo1m30s. Actual text click/edit/Enter, checkbox complete, close child32456 exits normally, native launch27400 restores, uncomplete/next edit persists; independent SQLite oracle SHA01f0d2e8ecedce98e5816855abeefbcc98d322c06805f737c028176f50399328. All raw evidence plan054/t04; source/tool shadow hashes and capture IDs in parent round134. Final060 review/merge remains pending, other native tasks open.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 43 | outcome: headless soft-wrap duplication repaired; payload-free logs; native Blog absorption failure localized | evidence:
  - 31af0ce0 removes unconditional frame text and handler argument payloads (counts retained). Runtime p060_7passed/0ignored/.18s, raw payload-log-runtime-r140.log under plan054/t04. Running host31244 remains5255cbe0, so this is source verification, not yet live Chat privacy acceptance; opt-in hot/emit traces stay off.
  - cde759da fixes shared v1/v2 buffer_runs: cosmic run.text is whole logical line for each soft-wrapped visual run; emit only min glyph.start..max glyph.end UTF8 cluster range. Real narrow cosmic Buffer regression Latin/CJK/newline before fix exit101/1failed (Latin repeated4times), wrapped-text-red-r140.log SHAc93113e834674a9a32701294e2fa0554235e7229529a6ae90057e701c850c4ad. Final runtime p060_8passed/0ignored/.21s, green SHA62f755d4d35ebe32891b7185869ad54f65e14b89cddee9975a30619f961154fe. Live native Database consumption rebuild pending; do not count old v4 capture as wrap fix.
  - 1a64cc52 adds bounded merged-back stage errors. Official tool consumer build exit0/1m13s, immutable shadow auto-ui-tool-backend-diag-r140.exe SHAbcfa583e25e2339f272feea778011875aa896bf3226027b3aa1fcceb61ec4802. True Blog entry023-realworld author-page NewDraft pointer returns create_failed; generated create/open/save return defaults, actual app data empty. Formal tool replay regenerates and reveals merged-back endpoint transpilation failed: undefined variable. db and companion transpilation completed; failure is api-body stage, not backend filesystem/security failure. Parent054 r135 records exact artifact/task evidence and corrected IDs. Investigating stripped API type declarations vs typed struct literals; targeted regression pending, no fix/pass inferred yet.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 44 | outcome: structural endpoint absorption fixed with compatibility controls; real Blog consumption pending | commit: f0c6533e9cff284c1acbf7ddabcdec82d031f014 | evidence:
  - Real Blog formal replay on diagnostic1a64cc52 regenerated frontend, then build exit0/Cargo1m44s even though endpoint transpilation failed/undefined variable and create_draft defaulted. Build success is not business success. Failure occurs after successful db/companion conversion; native Blog child34112 closed normally before rebuilding, no data written.
  - Permanent p060_merged_endpoint_body_keeps_declared_struct_types constructs real API/db files and runs actual parsing/transpilation, with typed struct literal returned in an endpoint branch. Initial test build exposed four preexisting helper-call signatures still at2args after060 production helper became3args; updated all4 plus new fixture to pass canonical type emission. That compile failure is not behavioral red. First fixture also used direct struct return in db and hit a different known parser shape; corrected db fixture to typed local + return and multiline type. Then exact pre-fix endpoint failure red3 exit101/1failed/0ignored/.92s, SHAecdad2e9e71d53c961e37193e232df6d1249c05a3e43d31b562bef92969701de.
  - Import-only candidate did not repair parser knowledge, retained intermediate failures. Final f0c6533e keeps source type declarations for endpoint parsing, then reuses shared promote_api_types_in_companion for emitted endpoint and companion types (no new duplicate struct hierarchy). Removes obsolete strip_api_types helper. front-layout v5->v6 forces real regeneration.
  - Existing4 merged compatibility controls initially2passed/2failed exposed preexisting060 full-absorption precedence drift for scalar-only API contracts. Preserve established db-first/explicit demo fallback by restricting new full structural absorption to modules with declared API types; structural endpoints retain their actual endpoint body (including auth/error branches). Final cargo test -p auto-ui-tools --lib merged_ -- --test-threads=1 exit0/5passed/0ignored/1.08s, merged-final-green-r140.log SHA07ddee60fb4923e93ed479b86882f187c2862f537b96d19d58cf1d637485f028. All assertions retained for db42-vs-api0, demo fallback, route A real/failed cases, and typed body delegation/no default.
  - Consumer detachedf0c6533e; official tool v6 build in flight session60317, tool-backend-v6-r140-build.log. Live Blog author task/rebuild and live wrap consumption remain pending. Host31244 still5255cbe0; do not expose native Chat credentials to its old payload logs. Sky rediscovery returns single Auto-Desktop5573708 but state is wallpaper and activation fails; no reliable OS input claimed, application acceptance channel continues for native broker inputs.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 45 | outcome: index adapter verified; consumption waiting on disk space | commit: eafadfadaec3141e36f8b93773a288e31b985e33 | evidence: Blog v6 true backend now fails only2 E0308 (find-from i64 offset vs i32 SDK), raw log plan054/t04/blog-backend-v6-r140-build.log SHA1ba3a0c42242d0e4641331b7fc8f35595d9c14951f7ee2d0dafb137babe84ee7. Add shared emitted/tested wide character-offset adapter using SDK search at relative0 (no narrowing), code-region rewrite preserves strings/comments, v7 freshness. merged_6passed/0ignored/1.04s (60 SDK controls +3wide cases), green SHA51ce97d8196c0f778f40ec883e54b3f0f0d56f2a70e5b942777c0dff7e105740. Consumer detachedeafadfad, clean source. Tool build7859 terminal101 LNK1318/PDB FILE_SYSTEM; D free0. Own idle incremental18.7GB/reparse0 verified, cleanup rejected by automatic approval review blocked by policy; no deletion executed. User asked to free8GB; see parent054 round137. No toolv7/business pass, no final review/merge. Receipt may remain uncommitted until space returns.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 46 | outcome: v7 native Blog backend consumed; two exact acceptance-input/composition gaps found | baseline: eafadfad | next: scoped repairs
  After user-requested OS compilation cleanup, formal tool v7 low-debug rebuild exit0/4m29s, CLI exit0/297.3s, host exit0/119.6s; host30912/SHA20887eca45969cbd9adce309583500954713087ddf268b5c0c7ae191fa603883 owns MCP9247. Tool immutable SHA78dc7a3ce356eb249a6286835a2e2dda6fae7ac6ae5e83846e5195242507a621. Formal Blog v7 build exit0/Cargo5m07s; receipt/published/target SHA28578e92eb72b826245bb37e38eac4ec7f56a781136e49e30117f725f6c68ef2, source generator v7. Actual native author NewDraft pointer creates work w-1793926791923-1/rev1; real title/body input renders dirty, but after8s revision1/bodyempty persists. Pointer NewDraft again performs genuine save-before-new, preserving original title/body in rev2 and creating another blank draft; not a periodic-autosave pass. Gap oracle native-blog-periodic-save-gap-r142.json SHAb97871875ea67a2746239c8b45841409494e1f6903d7c720d6fbeefe4012a95c; dirty screenshot SHAc749efc83d99c13e94894bc9ddddda277ccb4611b02c597d739037c18183e415.
  Actual path: generated author exposes tick_interval_ms3000/tick_msg AutosaveTick, but generated route-root App exposes neither. Headless poll_tick queries only owning root Component; existing child enum on() already syncs canonical store and UI. Bounded repair: roots without their own periodic timer forward the active outlet child's existing single timer and wrap its typed message through the existing root child-on arm (including single-route Fold). Existing local/root timer precedence and first-timer-per-component boundary remain unchanged; no new routing or language/timer semantic expansion claimed. Representative checks: folded outlet, active multi-route selection/no-match, local-root timer control, actual Blog idle-edit persistence. Scope is missing propagation in native component composition, not adding timer channels.
  Second concrete gap: inproc Paint requires Paint(int), while current acceptance msg supports only arg:string. Live action=msg,args=[17] returned queued but silently dropped args (argc0/VM underflow); canvas stayed256white. Do not call that drawing. Proposed additive acceptance-only scalar args array preserves int/bool/float/string through the existing DynamicMessage::Typed path; legacy arg/string and outproc wire are unchanged, typed arrays must not fall back into broker placeholders. No direct state/DB injection. Validation planned: actual MCP parsing/queue to selected VM component, mixed scalar preservation/rejection, nonselected/broker isolation, real Paint/Charts user-task replay. This is a test-input preservation seam needed by parent054, not a new app feature or non-root native RPC.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 47 | outcome: routed single timer propagation repaired; live consumer pending
  New targeted test first failed to compile because its helper belonged to another private test module (route-timer-red-r142.log); fixed the fixture helper and route syntax, then confirmed actual behavior red: folded root missing timer (route-timer-red-r143.log, exit101/1 failed). Generator now forwards only the active outlet child's existing single timer when the root has none, maps through the root child message, and retains local timer precedence. Layout freshness v8. Targeted regression exit0/1 passed; local timer control1 passed; outlet controls4 passed/0 ignored (logs *r143.log under plan054/t04). Existing pre-change outlet baseline was3 passed/1 failed: an old PLAN039 test still expected rejection of multi-route functionality already implemented by PLAN052. Updated it to verify persistent route selection/message wrapping, not silence failure. Actual native Blog idle persistence is still pending consumption; no final060 or054 pass.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 48 | outcome: native Blog idle save verified; initial route store writeback and typed inproc messages repaired
  Consumer05657fc0 toolv8 SHA55ac462ae164e00b692dca34290b49057994efef431153500ec2fdc48910e321, official Blog build exit0/Cargo30.13s. Actual newhost27596 owns9247; child48944/wid1 NewDraft then real Chinese title/body and idle8sec (no Save/New) persisted w-1793927770702-1/rev3. Oracle native-blog-v8-idle-oracle-r143.json SHA0ba39bf18c4d48fe0bd7cc040e62d340e38740405121e43db1b3988081f2ccbf; screenshot a6c689fab5308a4dd8d9cb7aa0a73fe98f945c6d904f4d57143306739bb4208b. Child48944 then exited after actual close wid1, host27596 remained. Whole Blog task still open: initial list was empty until NewDraft, bottom preview controls not reachable in current viewport.
  Initial-list root cause: __apply_route called child.on(Init) directly, so child store mutations were overwritten by canonical root store on next view/event. Changed route Init to existing root child-message wrapper (normal store sync/writeback and navigation path); v9 freshness. Regression verifies this call path with project lifecycle metadata registered; route-init-green-r144.log1pass and outlet-controls-r144.log4pass/0ignored. Initial test fixture omitted project lifecycle pre-registration; corrected rather than treating that fixture failure as product red. Actual live v9 reopen still pending.
  Added acceptance msg args scalar array for inproc roots, mutually exclusive with legacy arg:string. Validation before queue; event-loop dispatch uses DynamicMessage::Typed. Reject nested/null/out-of-range VM Int, float overflow; existing handler ABI is i32/f32, bool arguments use its existing0/1 storage convention. No unsupported wide-int claim or truncation. Broker-owned arrays reject before wire/placeholder execution. Runtime p060_10pass/0ignored/.15s (scalar-final-r143.log): actual parser+queue→VM mixed values/selected-target and live named-pipe broker no-send/placeholder isolation, plus8existingcontrols. Earlier fixture semicolons, Value string Debug quoting, and bool readback expected variant were corrected against existing VM ABI; no product behavior weakened. Canonical Specs updated with exact supported boundary. Actual host/Paint/Charts consumption remains pending, no final060/054 pass.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 49 | outcome: typed messages consumed by full native Paint task; stale closed snapshot ownership repaired
  Official v9 tool exit0/43.23s SHA38e85efde60169619ab86ab80227e1be9ddbfa95286845edf51f8b3c8ed42fd0; new host exit0/44.66s SHA8f4f33e8aa5276366fed173bfe1d59ac67dbf2aea802f2130d012696de74324f, live43076. Actual Paint(int) now argc1 and changes the pixel (old args silently dropped). Controlled New→17black/18red→Undo/Redo→Save/New/Load→close/reopen→Load→34black→Save/Export completed; exact live9:auto-paint state, independent scopedKV and SVG raster geometry all256pixels match, actual screenshot native-paint-grid-r145.png SHAd51345c87e46ead6d4900a2aef6903e30bcf63d077d710aac233aeba89a3b926. Final oracle native-paint-final-oracle-r145.json SHA9e9f846b2ebdf924befe4b24f8becfdb02e3075b73d9b821a08b894809538977. Paint task pass does not close whole054/060. Intermediate failed probes were stale selector/SVG oracle mistakes, not hidden product failures: SVG intentionally uses white background+3colored rects, independently expanded to256cells. Window was zero-size/minimized then user restored; actual app layout grid brought window visibly into viewport. No Windows auto activation after user Esc.
  New exact finding: SharedState app_surfaces kept closed7:auto-paint and find_app_surface(name) selected it ahead of reopened9. Explicitly excluded those old-selector probes from acceptance. ServiceTick now prunes published surfaces whose AppId is no longer in DesktopSession.apps, after desktop close commands and before fit/screenshot early returns. Keeps live observer surfaces and requires a new publication before reopened name lookup; no synthesis of business state or rendered frames. Regression exercises old close→no-result→new instance publication→correct name result and unaffected observer. p060_11passed/0ignored/.32s, surface-lifecycle-r145.log. Canonical ownership boundary documented; host consumption still pending.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 1 | round: 50 | outcome: surface retirement consumed; full native Charts task; injected click-to-wheel hover repaired
  Consumerdfa61f39 official tool exit0/40.02s immutable tools/auto-ui-tool-v9-surface-r146.exe SHA2023faf889cf33c44c588938bd4cb77f1411e3d4c097240059dcba0c74f85e33; host exit0/30.62s,39560/SHAe96eb4e74c7a7edc90bb05a784db8d6625a9f193be31c6a98fa0072e5abe1344 owns9247. Blog007 clone native5e54d648... under explicit extra ID blog007-author adopted1280x800/capsinput-action; initial existing3draft list is now visible immediately, genuine pointer opens persisted Chinese body: route Init writeback consumed. No preview/whole Blog pass yet.
  Charts actual VM native task (stable ID024-charts), no sample injection: Chinese CSV path import→typed SetIndexCol[0]/ToggleValueCol[1]→three decimal bars→Save/Export→closewid2. Name snapshot unavailable after close (new retirement code); reopen publishes9:auto-charts vs old8, OpenProject restores mapping/embedded rows, NameChanged/DoSave creates next project. With own CSV temporarily renamed, project export succeeds; source restored byte-equal. Independent SVG threebars/zero-baseline/ratios and diskproject values[10.5,23.75,8.125] match actual chart. Final oracle native-charts-final-oracle-r146.json SHAc804d5476a1ac097f077d284508c90f51efa99625ad613f99967cb4e9137f739; screenshots native-charts-mapped-r146.png SHA7e7c18ea2d84e93bfac349747250655ddcad736a3c35657cc16970b269d56ebe and native-charts-reopen-next-r146.png SHA3deb1fa803c4fde6ec90b5d9022d784ad3a1a7b2e56287c51540226c8857f248. Native Blog remained alive through Charts close; not a full054 mixed/failure gate.
  Blog scroll probe exposed exact acceptance-input gap: broker_pointer_down/up carry explicit coordinates but never update WmState.last_cursor, while route_live_input(Wheel) uses that cursor for hit-test. Real named-pipe regression moved prior cursor outside target, injected click, then asserted hover and subsequent Scroll ownership; red actual cursor700,100 vs100,100 (pointer-hover-red-r146.log). Fix records coordinate-bearing pointer position before hit-test; existing OS path idempotent, no new wire/action. Clarified MCP dy description to existing raw iced positive-up/negative-down convention (no sign reversal in hardware path). Final p060_12passed/0ignored/.27s (pointer-hover-green-r146.log), includes actual pointer/scroll pipe and all11prior cases. Host live replay of this last fix still pending. No final060/054 delivery claimed.

- stage: new | plan_id: auto-ui/PLAN-060 | plan_revision: 2 | outcome: pass | next: work/T-04
  Consolidates the actual same-goal repairs and replaces the original unimplemented move/ControlMsg::HandlerCall sketch with delivered click/down/up and negotiated InputMsg::Action. G1..G3, AC-01..06, eight-app scope, independent final review and no automatic new-context constraint retained. Canonical SD paths corrected: original ui/runtime-and-renderer.md does not exist; native-codegen-semantics.md and native-command-frame-text.md now document real supported boundaries, with existing desktop input/component ownership Specs. No new app business feature, router/timer scheduler or wire extension authorized by this revision.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 2 | round: 51 | outcome: live wheel targeting consumed; native text-local transform loss repaired in both frame formats
  Consumer7c986e8a host rebuild from OS cwd exit0/17.24s, new28824/SHA05a521fa83413e03ed952cd6502f6a66068d64812c83150a4229c608ce2d34a9. Initial mistaken build from UI cwd exited101 package-not-found (host-hover-r147-build.log), corrected command host-hover-r147b-build.log; no source regression. Actual pointer+negative raw wheel now reaches Blog root ScrollArea, but text remained at unscrolled coordinates while culling advanced. Whole Blog preview remains unpassed.
  Exact root cause confirmed against installed iced_tiny_skia0.14.1 Layer::draw_paragraph and iced_graphics0.14 Text: Paragraph/Editor owns its transformation, while Item::Live.transformation returns identity. Existing lowering read only the item/group transform. New actual native ScrollPage red: bottom label y836.9997 outside160px viewport despite scroll state at bottom (native-scroll-seam-r147c.log). Earlier r147/r147b logs are test fixture compile corrections, not product behavior red.
  Compose item parent × text-local transform once. v1 translation and v2 per-text Transform envelope both corrected; existing cached-text handling retained. Regression checks native scroll bottom location, v1/v2 equality, host canvas replay with additional parent translation5,7 (detects lost/doubled transforms), and actual bottom-button click. Final p060_13passed/0ignored/.27s in native-scroll-final-r147.log; includes all prior input/capability/closed-surface/typed-args/softwrap controls. Only the changed function/test snippets formatted, unrelated file formatting retained. No change to rotation/shear/raw/mesh support. Live rebuilt Blog child still pending; SDK runtime change is checked by Cargo, no front-layout version bump because emitted source layout unchanged.
  Blog007 Vue official production build also exit0/14.78s; official auto run --server rust -F3412 -B17829 active session45314 now Vite ready, isolated blog007-web-r147 data. Owner Vue live/375px and native scroll/preview/reopen still pending. Native Paint/Charts task evidence retained; parent054 full gates/review/landing not complete.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: 2 | round: 52 | outcome: existing WM client resize protocol connected and regression verified
  Native Blog007 consumed text-transform fix6f00f375; wheel moves text but footer remains below actual WM client. Root cause: Hello1280x800 was used as decorated outer size, broker sent no WM resize, endpoint ignored Frame Resize, Headless sources ignored lifecycle resize. Repair uses existing protocol only: bound initial command outer rectangle to usable area including existing chrome; each active non-shell command client receives primary-surface Resize when actual WM-derived client extent changes, metadata updates only after send. Pixel/shell negotiation retained. Endpoint validates primary ownership/positive finite dimensions and deduplicates; headless/MCP/projector sources invalidate frame, retaining component/cache.
  Genuine pre-fix endpoint/native-scroll red: resizing must invalidate the frame (native-resize-red-r149.log, terminal101), not a test compile failure. Initial green attempt had local TransportError Display compile error, corrected to typed Result without message formatting (r149 green log); no behavior result claimed for it. Final native-resize-final-r149.log p06014passed/0ignored/.30s after formatting and explicit component-state7→resize retains7→visible bottom click8. Existing v1/v2 replay controls retained. New real-pipe broker adoption verifies bounded outer geometry, correct client dimensions, maximize notification, repeated-pump deduplication and pixel control. Native Blog build/actual footer task remains pending, not inferred from unit green.
  Current old host28824 and descendants already absent when normal shutdown command attempted (connection refused); not a successful commanded exit or resource-release pass. No forced Windows activation or foreign process cleanup. Next rebuild frozen candidate/child and actual owner007 task, then parent fixed-pin consumer and remaining gates. Specs native-command-frame-text.md records bounded resize contract and old-client limitation, no tag/capability addition.

- stage: new/work | plan_id: auto-ui/PLAN-060 | plan_revision: r3 | round: 53 | outcome: bounded Http-profile/JSON-state repair verified in scoped checks; app consumer pending | next: formal tool/Chat build and OS054 shared Rust service
  Inherited same OS054 completion authorization, AC01..06/tasks/budgets/repositories unchanged. Parent054r2 withdraws only Chat embedded-backend assumption: native Http.post('/api/register') passed raw relative URL to kernel, no backend/db created, public UI registration rejected. Native Http now consumes generated app base without process-env mutation; absolute URLs preserved; status0 raises static connection failure, API HTTP error body preserved. Baked profile is captured in receipt profile/http-base and layoutv10; normal exe and Http-bearing shell-library generation provide constant. String state target uses raw JSON/VM-compatible string conversion so u-54 is not integer0; integer control consumes raw Value once. Indexed JSON field projection leaves Index to normal emitter, preserving __at_idx instead of raw Vec indexing.
  Actual baseline native-http-codegen-red-r151.log: two behavior assertions fail (raw Http adapter and string id numeric cast). First green attempt exposed a fixture semicolon parser error, corrected to normal model newlines; next assertion exposed existing double numeric projection, fixed to raw JSON conversion. Supplemental p060_native_http_json_string_projection_keeps_safe_array_indexing red in native-http-index-red-r151.log caught the new string path bypassing index protection. One broad patch initially hit adjacent null-coalesce collection and caused compile E0425/E0433; restored that function and scoped the intended helper change, final diff confirms no null-coalesce edit. These are work checks/fixture corrections, not failed final-review cycles.
  Final compiler p060_7passed/0ignored/.04s, native-http-codegen-final-r151b.log (14.13s build). Includes prior text/table/route/input controls plus HTTP/profile, string/int/composed-string and out-of-bounds-index cases. Runtime native-http-runtime-r151.log2passed/0ignored/.01s: real TCP POST UTF8 body and idu-54, absolute GET retains origin, HTTP403 JSON denial preserved; transport failure static diagnostic. Tools native-http-profile-r151.log1passed/0ignored/.04s: generated profile constant equals receipt input, differing ports yield distinct identity. New helper module formatted; no broad/full regression yet. Final six native builds and actual Chat/UI/DB/security task still pending; no task completion inferred from scoped green.
  Existing official CLI auto service --server rust is confirmed server-only (no Vue/Vite/UI), strict generation, no occupant killing, loopback service config/real AUTO_SERVICE_READY and __auto health/shutdown. Existing desktop_backends::DesktopBackends already owns Rust/VM server children for Vue. OS054 bounded probe will reuse these existing service/ownership facilities; no need to assume a new app health endpoint or handwrite a router/secret backend. Production iced provision wiring/last-owner semantics remains OS054 work; no app source/pin changed. No credential acquired in failed Chat attempt; worker exited and temporary private OCR image removed.

- stage: new | plan_id: auto-ui/PLAN-060 | plan_revision: r4 | round: 54 | outcome: pass | next: work/T04/T05
  Actual native Chat composer callback red recorded in §5.4. Revised only codegen forwarding design within original native-input/app-task scope; AC01..04/6 original plus prior added tasks unchanged. Source ca754c7e. Ordinary parent-message references supported; existing single child-type instance model means conflicting bindings and fixed-argument bindings are rejected, not guessed. Constructor callback lifecycle is not newly promised. No new context or agent requested.
- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r4 | round: 55 | outcome: scoped codegen repair ready for actual consumer | next: work/Chat rebuild
  Generated instance queue retains msg-prop calls/typed JSON payloads, parent takes queue after child/store writeback and dispatches its declared message, unbound optional callback drained, invalid binding static compile_error. Compiler p060_8passed/0ignored/.15s (native-callback-codegen-final-r152.log), including actual parsed child→parent zero/string/int/multiarg binding, call order, unbound and fixed-argument rejection controls; prior7 passed unchanged. First fixture attempt had model-semicolon syntax error; next assertion incorrectly omitted existing String clone, corrected fixture/assertion, neither counted as behavioral red. Actual real-input DB-zero counterexample remains the red evidence. Front-layoutv11 invalidates old generated app. Spec updated; generated execution and actual Chat input not yet claimed.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r4 | round: 56 | outcome: actual callback consumer closed; app reopen invariant routed CHAT003 | next: work/T05 + scoped JSON/freshness follow-up
  bcacacf7 formal v11 tool38.09s SHA6f25fbf88c7fa2219b17c998527ef434a4374f9ef9e889a2b5b5b27a8804bf28; candidate Chat37.33s published/target/receiptfd0b88cdf9c672bb9cd820b750b9713a8833126b6a3a3d1d0e09e1cd8687caae. Real sidebar contact callback selects AB; actual composer pointer Send/Enter forwards through parent/store and persists exactly one intended message per actor, visible on the peer; extra empty Send did not replay. C actual login/read request shows no AB messages; C write rejected/no DB change. Parent054 round151 holds exact screen/oracle receipts. Reopen exposed app-local cm-0 identity reuse, not a callback failure; CHAT003 source5b9725f adds independent CSPRNG session prefix and Vue C01/T5/T9/T10 scoped checks pass. Owner native test in progress on bcacacf7, explicit owner preview distinct from fixed OS pin.
  Two additional source-grounded gaps remain before framework freeze: JSON client_msg_id in string concatenation still uses numeric field-name heuristic (failure pending reconstruction can lose original retry key), and front input hashes omit local embedded back.at changes. Neither is counted closed by eight compiler tests; next bounded design/fixture unit must address them. Six native build union, final lifetime/SDK consumer gate and review/landing still pending.

- stage: new | plan_id: auto-ui/PLAN-060 | plan_revision: r5 | round: 57 | outcome: pass | next: work/T04/T05
  Bounded semantic correction in§5.5 from actual generated Chat failure-key/JSON-object projection and omitted local backend inputs. Existing AC/6 task envelope unchanged; producer42573475/behaviorbcac, original authorization retained. Local frontend JSON use contexts and local backend closure only; no new language type system, protocol, supervisor or external dependency resolver.
- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r5 | round: 58 | outcome: scoped implementation/checks passed; actual native retry pending | next: work/toolv12→CHAT003 retry
  Actual pre-fix compiler intent fixture1failed/exit101 (native-json-intent-red-r154.log); tools backend-only mutation1failed/exit101, old/current input maps exactly equal (native-back-input-red-r154.log). raw_json_read preserves known Value bases including typed store Value fields and existing safe indexing; string contexts use __at_str(original Value), Value collection pushes clone original object/array/scalar. Typed/string/numeric controls stay unchanged. Final compiler p0609passed/0ignored/.14s, native-json-intent-final-r154.log; includes actual client_msg_id string context, original object push, numeric state and safe index control plus prior8 regressions.
  front_input_hashes now includes selected src/back else back .at content+file set, file/dir read errors propagate; v12 invalidates old generation. Tools p060_native2passed/0ignored/.62s (native-inputs-gate-final-r154.log) includes actual managed front_needs_regeneration accepted unchanged receipt(false,false)→backend-only same-size edit(false,true), addition/removal, README unaffected, fallback precedence and read-error rejection. This is the real reuse gate, not just map inequality; initial map-only green2/.10s strengthened with the consumer gate and error oracle before final commitment. No runtime HTTP helper change, its earlier realTCP2pass reused by unchanged source. Actual v12 tool/Chat rebuild and native failure→same-key retry still required, no six-app or whole054 pass inferred.

- stage: work | plan_id: PLAN-060 | plan_revision: r5 | round: 59 | outcome: actual native r5 JSON/retry consumer closed | next: work/frozen integration union
  Chat owner source5b9725f, formal native1ae9764a06dedcb5dfe6e11407baf47effb360d2128e0e2e34a643fd11ffeaec uses SDK8774c1c8/toolv12. Real visible draft pretyped while online, own backend stopped and kept absent until native failed bubble observed; no premature recovery counted. After same data backend recovery, actual pointer1140,370 retries same pending key cm-[same nonce]-2-u-4, DB adds exactly1 exact text/sender, preserved baseline/other conversation; final settled normal bubble observed, extra click no duplicate. Oracle D:/autostack/plan-artifacts/plan-003/native-retry-final-oracle-r2.json SHA9a21d18f7a60393b5715c80cb4ad58a3d3cb50027151ca86c2303a3f78053b0b. Initial restored-too-early probe counted ordinary send only, explicitly excluded failure/retry. This runtime closes raw Value key/object semantics plus callback forwarding actual consumer; compiler9/tools2 scoped proofs retained unchanged. App CHAT003 reviewed same-context/frozen gate10+8/archived and pushed5cdca0f; parent OS054 fixed-pin v12 and six-native frozen union/final framework review/landing still pending. No final framework pass or OS independent review claimed.

- stage: work | plan_id: PLAN-060 | plan_revision: r5 | round: 60 | outcome: fixed-pin native consumer and product backend owner verified; new native viewport/lifecycle findings retained | next: scoped diagnosis/owner contract
  Parent OS054643b8c6 finite desktop owner actual official CLI/Rust service path passed: immutable product host f968945759058dc4301ddaa8a9b6921438f9cdde38b4b84e0fa0f7f62127c377, fixed Chat5cdca0f/native58bff417/SDK8774. Real single host14388→CLI34880→backend35912/17823, stable launch/real credential recovery/contact/history/new message6 rows, normal last Chat close retains backend, relaunch/recovery restores, normal desktop/CLI/backend all0 and port absent; independent Blog backend47236 preserved. Parent T03 closed4/10; no fullOS or framework final pass.
  New actual T04 viewport finding: after5 original messages, sixth native message renders final bubble clipped midway and two real wheel injections leave it clipped. DB exact6 correct, client callback/raw JSON/transport task valid, but full history rendering is not declared passed. Screenshot D:/autostack/plan-artifacts/plan-054/t04/screenshots/native-chat-single-owner-send-settled-r158.png SHA451629b53fa55538c5153395a725e2b2da53fb96d682966fc1b4188d1aae096c. Reopened history same bounded viewport. Investigate actual message-list flex/overflow/layout producer and wheel routing before repair; do not use DB-only assertion as full UI success.
  Parent T05 private rqhost red reproduced on multiple normal product exits: host18260/backend31956/CLI16052 all0 yet own private rq15528 live; host22164 all0 yet rq29924 live; oldhost24164 normal0 yet rq38512 live; single final host14388 all0 yet rq44260 live. Actual spawn_desktop_daemon drops std Child immediately, and zero-window rqhost remains resident by legacy-q policy. Need explicit minimal owner contract before code: reclaim only own private daemon on normal desktop return, preserve global/manual-q semantics and existing forced-host-death compatibility. Lifetime extension not silently implemented under a stale r5 review. Diagnostic witnesses retained, not force-killed/counting cleanup pass.
  A parallel two-iced probe collided at fixed autodesk-broker (new child adopted by old host; new showed unavailable), excluded from acceptance and rerun with one iced host. Both normalclosed0; no new multi-iced support promise. Product normal shutdown via actual power request/confirm was also verified; first missing acceptance env properly refused privileged bus, no gate bypass. Parent production oracle SHA0c91c3d52c7a6f3c818ce091a4427e911c88fedf10f1d9efbde95c143c3f5400. Current no native host/client/shared17823; parent memory-only credential workers remain for later frozen gates.

- stage: new | plan_id: PLAN-060 | plan_revision: r6 | round: 61 | outcome: pass | next: work/T04 native overflow lowering
  Current writer0de4007f, consumer8774c1c8; committed actual Chat sixth-bubble red in parent054round156. Code verifies only VM converter creates Scrollable for style overflow, typed ViewBuilder does not, Rust plain col emitter ends build. Existing explicit-scroll headless fixture misses producer gap. Added §5.6 execution contract and scoped controls; inherited original parent054 full-delivery authorization and native semantics goal, AC01..06/six task IDs unchanged. Prior real backend supply/lifetime is parent T03 passed; private daemon lifecycle remains separately scoped parentT05. No implementation/Spec change in this new-stage receipt.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 62 | outcome: styled-overflow invariant implemented and scoped controls passed; native consumer pending | code_commit: 43a191338b0e2d80f9b9f2799494e66d201babd5 | next: work/T04 actual fixed-pin Chat long history
  Producer gap reproduced: permanent compiler parse/extract/generate literal/dynamic/empty-column case old output0 overflow conversions versus expected3, exit101. Runtime old-column-equivalent control uses a temporary identity conversion helper; actual HeadlessFrameSource→HostEndpoint narrowed240x160→wheel fails bottom text in scrolled frame, exit101/.10s. This is a rendering/interaction oracle, not only a generated-string assertion. Baseline data is actual Chat six-row history with clipped final bubble, unchanged.
  Fix normalizes generated ordinary row/col after actual style evaluation via View::with_overflow_scroll. Runtime detects actual OverflowYAuto/OverflowAuto only; normal/hidden layouts and explicit scroll retain structure. Reuses the exact VM split_scroll_visual_classes/ensure_full_scroll_dims bodies moved to view.rs (mechanical body equality verified against previous committed module); visuals stay viewport, content retains layout/children/spacing/padding/onclick. No duplicate explicit scroll wrapper, no new horizontal/auto-bottom contract. Native emitter literal/dynamic/empty paths use same conversion. front-layout/v13 invalidates old output under unchanged source. Spec native-codegen-semantics updated under reviewed future delta.
  Scoped final compiler cargo test --locked -p auto-ui-compiler --lib p060_10pass/0ignored/.16s (SHAf8eab4317f0bd97919f93050b90b342f9822bdbc4eb9b57ee369d954e78cae15). Runtime --lib p060_native_8pass/0ignored/.07s (SHA025de92550b389124db7ad3a71f07ac9b16fde87ac23857a58024614bd6dc130), including real style-derived scroll and explicit-scroll accepted control, v1/v2 parent transform replay, bottom-button click, view structure/paint/handler/no-double-wrap controls, existing HTTP2 and native-input controls. VM existing p748_j1_2pass/1.83s, actual viewport paint and shrink/stretch geometry retained. Initial runtime red command omitted --lib and built unrelated integration executables before its one selected unit failure; no unrelated cases counted as executed. Final runtime command selects library only.
  Tools permanent p060_native_2pass/0ignored/.67s, now mutates actual accepted receipt generator to v12 and requires managed front_needs_regeneration(false,true), restores valid receipt then preserves existing back-content/file-set/read-error controls. First launch inherited CARGO_TARGET_DIR in consumer's other checkout; its fixture correctly rejected another-tree writable target before executing the intended freshness check. No production gate bypass or weakened assertion: same compiled test executable rerun with that unrelated explicit build env absent, selecting own allowed fixture context. Final tools SHA30861b0c9ad2e003f7898e75026d4f78cb244c30e4e5146ce4e4474a3b5880df.
  Unit writer Lang4fdf1686a69bde6bebbcd18ad7261e7b71f187e8 differs from parent consumer Lang23f3a031; both recorded, not claimed one final frozen SDK. Read-only consumerUI detached43a19133, formal immutable tool cargo rustc exit0/37.19s SHA79d8a4e302d66843cfc1a7b787bde0db7d01de7b0e29937fd08228586eda1dfb. Parent fixed Chat pac17823 formal v13 build session95793 running/log chat-fixed-v13-overflow-r159-build.log. Actual complete long-history frame, common frozen SDK/six-native union and independent review/landing remain pending; tasks/counts unchanged.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 63 | outcome: real native scroll moves; WGPU clipped-paint counterexample repaired; final visual pending | code_commit: 72bbcb045e141233db1380978efdd7d03b2a47af | next: work/T04 actual paint consumer
  Formal fixed Chat v13 build37.55s/exit0, published receipt9deef22d320c6d366e2c941b5a4e4d84ade628521e03f772a7c3a8be55fc7e14, app5cdca0f/SDK43a19133. Actual stable launcher/real retained A UI recovery/B contact restores six rows. First async frame still empty, excluded; settled frame shows history and real wheel moves final message fully into viewport (88a41e555c4a8b52b742439a299dbfdfff1ab04c69e8301502728841e8c761ee), but all bubble backgrounds disappear, own dark text unreadable. Not counted full visible-history pass. Old host source643b8c6/SDK8774, source-bound new app, common frozen SDK gate still pending. Scoped host18012 had AUTO_DESKTOP_RQHOST=0 explicitly to avoid adding known leaked idle helpers; not a T05 lifetime pass. Child22864/host18012/backend34988/CLI30108 all normalclose0 after capture, data remains6, no API message prewrite.
  Added actual HeadlessSurface colored-text fixture: both unscrolled/scrolled produce one red background Quad. This rules out lost source paint; first test draft had wrong Fill alias, compile error corrected and not claimed renderer red. Read official installed WGPU geometry Frame::paste: clipped child meshes append before parent's still-unflushed base buffers. Product single-geometry painter therefore paints root base after clipped bubbles. Permanent device-free Recorder uses actual iced_wgpu::geometry::Frame, NOT a mock paint model: actual pre-fix mesh clip order[50,100] versus required[100,50], assertion red/exit101.
  Both production DrawList(v1)/DisplayList(v2) canvas programs now emit ordered clip-run geometries. Root clear first; segment only at Scissor boundaries, carry/balance active transform/clip scopes, retain adjacent primitives in a run (no per-primitive geometry split). Wire schema unchanged. GPU-backend v1/v2 color-order case passes on actual generated meshes. Runtime p060_native_10pass/0ignored/.79s includes prior8, background-source proof and WGPU paint-order proof; added batch/scope case1pass/.00s separately afterward. No repeated full suite required for a test-only addition. Dev-only iced_wgpu dependency permits actual mesh backend without a Device/window; production optional feature graph unchanged. Spec native-command-frame-text adds ordered clipping rule.
  Read-only consumerUI detached72bbcb04, parent Lang23f3a031 unchanged. Formal immutable host build session30956/log host-native-overflow-paint-r159-build.log in progress; app v13 output reused under explicit host-painter-only change boundary (no app-facing output/schema change). Actual full-color long-history frame and final common frozen SDK campaign remain pending. Parent private rqhost lifecycle/native-dock and aggregate gates remain open; no final framework review/landing or parent independent review claimed.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 64 | outcome: actual fixed-pin native long-history scroll and paint repair passed | next: work/T04 final common SDK union
  Parent immutable host cargo rustc exit0/35.17s, UI72bbcb04/root643b8c6, exe SHA1645d52a7ed149bdf64e9f1f36139376907acfe1bc81d1eb4177193850385576. Same exact app5cdca0f/SDK43a19133/v13 native9deef22d320c6d366e2c941b5a4e4d84ade628521e03f772a7c3a8be55fc7e14 retained under host-only painter change boundary. Stable017 launchwid1/child46844, true retained A credential UI login, real B contact/history/wheel. Actual bottom frame932b63ef47285f9dd8ea5de0a3627e240c17e12391af9846bd002e43beab41ac shows all final sixth bubble/text/timestamp readable, own primary and incoming muted backgrounds restored, visible scrollbar. This closes clipping + WGPU masking counterexample together, not merely scrolling movement.
  Real composer Enter sends new128.25 message; independent read-only SQLite exact7 rows/senderu2/cvAB and byte-value equality of all previous6 against production owner oracle. After queued new frame arrives, first scroll state was previous max offset; second wheel on observed new content reaches actual final timestamp. Final full bubble/timestamp image512c940bbcc7c51fc78d136eb34f0e88b8d9500d043f85d57249628633de3ebc inspected. No auto-bottom promise, no credential/API prewrite, all credentials remain private workers. Final native-chat-overflow-final-oracle-r159.json SHAb7831479f2ed16bca7584c19a4f5bb713c19b44a6c6aa1729dee258182160156 binds source/artifact/data/images.
  Current scoped host28532 owns official CLI41336/backend19892/17823; native46844 live for subsequent checks. AUTO_DESKTOP_RQHOST=0 explicitly avoids creating a further known private helper leak while checking only scroll/paint; parent T05 remains open. Final common frozen SDK/6-native union, independent framework review and landing still pending. No full054 or finalUI delivery claimed from this scoped pass.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 65 | outcome: pass (work only, execution_done5/6) | code_commit: 72bbcb045e141233db1380978efdd7d03b2a47af | authoritative_tree: D:/autostack/.wt/ui-060/auto-ui | branch: codex/plan-060-dev | next: independent final review on this same tree
  Reconciled original T00..03 unchecked task rows against their prior detailed §9 implementation/unit/real-input receipts and current source; current_step2 was stale bookkeeping, not two newly invented feature completions. T00 decision and subsequent bounded designs are durable in this Plan (§5/§9), scratch-only raw evidence policy replaces old promised docs/plans/evidence file. No new decision attachment or source/spec index written.
  Final common-consumer union now actual: parent054 official public scripts/desktop.ps1 -Track iced -CandidateDir frozen-sdk-r161 starts root916a142 with UI72bbcb04, Lang23f3a031, Man03bb65ef. Producer CLI e6e8afa15adbaf22c32b9d1e9523d8f6cb6a19a7590ae6f6aa93a498a77ef53a/tool e90da49163891bbc811456bf4edeb3d3b1fd722d883ef53da886446807cbd61c/host46596ee20bfc9205e3a1670318a0f4ee4371410dff1cecdccf318f3fe59f6820 formally built0; all six native apps rebuilt0 and receipt source inputs plus actual target/published/receipt exe hash equal6/6. Artifact evidence D:/autostack/plan-artifacts/plan-054/t04/frozen-native-identity-r161.json SHA522513f040aa5afdeb9dee67dd38cd7ace7ce5c59714610f87de83e29029bd5d. Current authoritative238c6765 versus consumer72bb has zero crates/Cargo diff; writer Lang4fdf versus consumer23f has zero crates/Cargo diff (documentation-only successor). Earlier scoped current-code compiler/runtime/tools regression controls remain valid under this checked source equivalence, not a new frozen language source claim.
  Same host19944 completes all six native real task families with fixed delivered pins: Notes pointer/key Chinese title/body + Save, close0/reopen/next Save; Todo create/check/actual edit+Enter, close0/reopen retained completion/next edit; Calendar cross-day create/edit/delete, close0/reopen stable generation/deletion and next id; Database actual tree/SQL rows/read-only write rejection/switch/close0/reopen next filter and unchanged two SQLite file hashes; Blog actual NewDraft/multiline/Preview/Return/Save, close0/reopen list fields/next multiline explicit Save; Chat actual retained A/B credential UI login, A send256.5/B sees/replies512.25, close0/reopen loginA/history/next1024.125. Independent DB/file oracles and inspected real frames agree; long history final colored bubble/timestamp readable on same final SDK, final image85b19332f6a7a083302d433b9708a00ec4e8b4fcc654a582420c23f1c065b65e. Chat first7 preserved byte-value equal and exact10 final rows. Native Task oracles in parent t04: Notes fc5ec8eee4a36697e1cf0a61720ac97180e43df3d950358c1918f62572f49cc4; Todo825b9230ecd86c3473e82066b18d4cb68ac5c5721144d6d8a733d83db9e6dff8; Calendar cd51ecb7a01f407050564f4b25601734b60d973e60e9c2b15cf63c8f63127354; Database4c3edccc8491260008178e8f90922369215624ca0e52f6c149fee9f3e804fdc9; Blog bc1711d82356281b4982013259f86293f87ad6160171474358759b743743d3a3; Chat b95a61e296d0d62bcd868c6e81863d064d033ec2b3d0850bad90e3eeb1c55f1d. Paint/Charts also complete as iced inproc VM, not counted as native six.
  Native component Action dispatch remains only its declared root messages/scalar args; Notes child EditorPanel messages not declared at root correctly do not act, actual pointer/key child controls used. No arbitrary child RPC promise, no source-template snapshot as native UI result, no bearer/secret API prewrite or published credential frame. Parent normal close of all ten windows gives six native exit0, normal product host/CLI/Rust service0 and private helper controlled cancellation1; private helper lifetime is parent's separate OS owner, not framework compatibility change.
  T04 closed by the common SDK six-native build/task union above. Existing affected permanent red/accepted controls documented in rounds31..64 remain bound to unchanged code; final review and merge's frozen affected-module union gate are still required per stage workflow. AC05 independent final review has not run; no reviewed/pass, archive, upstream master merge or OS default pin landing inferred. Parent054 still executing4/10 with Vue/mixed/failure/aggregate/mutation/review/landing outstanding. Framework implementation writes now frozen for independent context on this authoritative tree; generated consumer app outputs and parent-only work may continue.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 66 | outcome: user-requested compilation cleanup completed; source review tree retained | source_commit: 360ecfb3a740c62b45f0abd0739594c2bb360d24 | next: independent final review
  User asked whether UI060 was complete and could be cleared to recover disk. Explained work execution_done5/6 versus outstanding independent review/merge; preserved authoritative source tree for T05 and cleaned only rebuildable target. Preflight exact repo/branch, target absolute containment under UI060, authentic Cargo CACHEDIR.TAG signature, target reparse0 and active consumers0. Standard cargo clean --target-dir D:/autostack/.wt/ui-060/auto-ui/target exit0: Removed28061 files/38.9GiB. Target absent afterward, source HEAD360ecfb3 and tracked status clean unchanged. D free124105351168 bytes afterward (concurrent other disk changes mean this entire free delta is not attributed solely to cleanup). Raw before/after/log D:/autostack/plan-artifacts/plan-060/compile-clean-{before,after}-r66.json and compile-clean-r66.log. Parent frozen SDK/app copies and prior raw test/visual evidence outside target retained; no worktree removal, archive or final review pass claimed. Later reruns rebuild required binaries normally; cleanup does not waive merge's affected regression gate.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 67 | outcome: Vue sibling import generation repaired; real public consumer pending | next: work/T04
  Parent054 canonical full-registry Vue public entry on UI72bb exposes esbuild Unexpected brace in os-config DesktopCfg/Modules stores. Generator emits static reactive import after createStore function opening. Reopened T04 and status executing4/6; no acceptance or app scope change. Move only reactive import to module declarations, retain sibling reactive instances inside per-App factory. Native generator/runtime source untouched; prior iced common six-consumer evidence remains applicable within this boundary.
  Permanent p060_vue_sibling_imports_remain_at_module_scope exercises zero/one/two siblings through actual store generator, checks every import before factory and every sibling instance after it. Actual old code red1, fixed1pass/0ignored; prior factory/context/watch control1pass/0ignored. cargo test --locked -p auto-ui-compiler --lib scoped filters, logs plan-060/vue-{sibling-red,sibling-final,factory-final}-r67.log. Shared c2e02416 target reused so cleaned UI060 target remains absent. Initial test placement accidentally landed in existing raw Auto fixture and selected0tests; corrected before red replay, zero run not counted. Real canonical Vue generation/load still pending; no execution_done or independent review pass claimed.

- stage: work | plan_id: PLAN-060 | plan_revision: r6 | round: 68 | outcome: pass (execution_done5/6; independent review pending) | code_commit: d2a9ba65668a12a2cb08fc14a12349b4670a1ccf | next: independent final review on same writer
  Read-only consumer detached d2a9ba65; formal immutable CLI build0/37.84s SHA4925596a5cdc5e626b7ac785b9958ee2745106bf32834f18c4376beda9a70bcc and tool0/33.63s SHA8f7ed783b9a889d282d2c3f8cdc54093746589f520adb20348438911aad358ec. Parent frozen-sdk-r164 retains byte-identical accepted iced host; source diff72bb to d2 affects only Vue store import location/test/Plan. Actual public desktop.ps1 -Track vue -CandidateDir frozen-sdk-r164 regenerates whole registry (no eight-app subset), both os-config generated imports now module-level. Canonical Vue npm run build exit0, 4185modules/13.19s, raw plan-054/t04/canonical-vue-production-build-r164.log. No esbuild parse failures. Actual headless browser public Auto Desktop entry loads with pageerror0, Notes/Todo/Calendar/Database/Paint real launcher user tasks and normal window close/reopen/next operations pass with independent disk/localStorage/SQLite and inspected frames. Initial Vite dependency optimization reload interrupted first launch probe; excluded and rerun after settled reload. Both consumer OS/Lang wt-guard clean after generation/npm. No OS activation.
  Current six-native iced tasks from round65 remain unchanged source/output/runtime evidence; scoped Vue regression checks round67 plus full generated Vue build cover new finding, not a repeat of unrelated native/GC suites. Parent still executing4/10 with remaining Vue tasks, mixed/failures/permanent gates and final reviews. T05 independent review/frozen affected module union/archive/landing outstanding; source now frozen again on this writer. User compilation cleanup remains complete: no recreated UI060 target (tests reused explicit shared target).


### 独立最终复审 r6（2026-11-06）

- stage: review | review_scope: final | reviewer_identity: independent child context `/root/review_ui060`（用户明确授权；未参与实现；未声明不可见模型/会话身份） | plan_id: auto-ui/PLAN-060 | plan_revision: 6 | outcome: needs_fix | next: work，同一writer修复以下两项后仅复审受影响不变式；不得merge/push/archive。
- worktree_path: `D:/autostack/.wt/ui-060/auto-ui` | branch: `codex/plan-060-dev` | reviewed_commit: `286e039744a3ebd6a8e726cbe54f062df2c775fe` | base_commit: `220115df62040f113bc689f0500af80fc11569a3` | dependency_revisions: writer sibling auto-lang clean detached `408ca8b5096df96d2a91831ea36ef25336781889`；consumer UI detached `d2a9ba65668a12a2cb08fc14a12349b4670a1ccf`。`git worktree list --porcelain`核实原树；初始status空，复审结束前再次HEAD相同/status空。HEAD相对d2的crates/Cargo差异为零。
- Authority: 主检出`D:/autostack/auto-ui`当前master `7ac0455bf4cd743248749e4bc4eb9d0aed657266`，含未跟踪旧060及其他用户WIP；只读确认后按本在途遗留计划指定writer r6为权威。仓根无AGENTS.md；遵守会话给定仓/伞形规则及auto-plan-review技能。没有修改/同步主检出、949树或实现，没有GUI/CUA/MCP/OS激活。此前5/6是末任务含review/merge，不作为遗漏work功能的理由；本次重开仅因具名finding。

| acceptance_results | task | outcome | 独立方法及边界 |
|---|---|---|---|
| AC-01 | T-01/T-04 | fail | 普通Expr::Index用__at_idx；但NullCoalesce组合路径仍直接Vec索引，F060-R6-01。此前Calendar真实启动成功保留，不外推全路径安全。 |
| AC-02 | T-02/T-04 | pass（既有输入范围） | 阅读MCP→DesktopInject→broker→native组件/新帧路径和命中断言；本轮runtime p060_21/21含真实pipe/viewport/scroll/replay/paint；r161 Notes pointer/key/Save/reopen原始oracle哈希吻合，未重演GUI。 |
| AC-03 | T-03 | pass（根零参/单字符串边界） | 实读Action tag11/Hello input-action/1能力门、owning wid及无本地占位回退；本轮真实pipe测试含旧peer拒绝/错wid拒绝/CJK新peer/旧键盘控制。未承诺任意child RPC或扩wire数值载荷。 |
| AC-04 | T-04 | partial | r167六app正式build exit0与receipt/published/target实际哈希18/18一致；r161 Calendar与六app真实任务oracle仍适用于未变原生生成/runtime边界。索引全不变式及完整Spec交付清单未闭，不能据六个成功app覆盖F01。 |
| AC-05 | T-05 | partial | 本独立上下文完成最终复核并绑定r6；两项finding使最终pass不可发。 |
| AC-06 | T-05 | pending merge-owned | 未执行落地/归档/清理/pin；需修复、复审通过及冻结联合门后交merge。 |
| 历史扩项AC-07/08/09 | T-04 | pass（所列consumer边界） | Notes/Todo真实任务与独立磁盘oracle，native_http真实TCP/HTTP拒绝和transport区分断言；Chat真实注册/双端/隔离/重试/长历史证据保留；不是所有未来app或所有语言组合的保证。 |

**F060-R6-01 / P1 / 原验收缺口，AC-01、T-01/T-04。**

既有规则G1/§5.1要求受支持数组对任意运行时索引不panic，且`arr[i].field`链保持安全。本轮独立选取已有语法的组合：

```auto
widget CoalesceIndex {
    model { var text str = "" }
    msg { Load }
    view { text .text }
    on { .Load -> {
        var rows List = [{ id: "u-54" }]
        .text = rows[999].id ?? "fallback"
    } }
}
```

- 生产入口是`parse_ui`→`extract_widget_from_decl`→`RustGenerator::generate`，不是手写替代算法；scratch harness链接正式consumer的compiler rlib，writer/consumer编译器源码已验证无差异。实际生成如下（完整输出保留于scratch）：

```rust
let mut rows = vec![serde_json::json!({"id": "u-54"})];
self.text = auto_lang_core::a2r_std::json::get_str_or(&((rows)[999].clone()), &["id"], "fallback").to_string();
```

- 期望：安全取得缺省Null，再由既有`??`返回`fallback`；实际：先对长度1的Vec直接取999，再调用get_str_or，无法满足边界安全。此结论由真实生成结果及Rust下标语义建立；未运行故意异常退出程序，也没有声称捕获过运行期panic。
- 已确认原因：`rust.rs::null_coalesce_rust`的`base_is_vec`首索引分支（本冻结版13862行）构造`({recv})[{idx_str}].clone()`，绕过普通`Expr::Index`的__at_idx。不是普通访问shim失败。邻面为局部/声明Vec基座、缺省表达式与字段投影的组合；`.rows`模型字段在试探中生成另一路get_owned(&self)表达式，该分支未做完整编译验证，不单列已证实产品finding。
- 最小关闭oracle：永久实际parse/extract/generate并执行`rows[i].id ?? "fallback"`，覆盖i=0、越界、负i、空列表及普通安全Index控制；保留字段存在/缺席的`??`原语义。若改链遍历，加入嵌套Value数组/模型与store基座的代表控制，确认不会重复求值。修复后只跑受影响compiler/tools生成执行控制与现有安全索引控制；是否重建消费者由生成输出变化决定，不附全仓/GC矩阵。不得削弱G1来消除反例。
- 可复验scratch：`D:/autostack/plan-artifacts/plan-060/coalesce-index-probe.rs` SHA256 `1af59209d77b8c96a94b3196575df48be3027146aab165f38ff4ae7f0753b6c9`；生成输出`coalesce-index-generated.rs` SHA256 `d88b96254392ddc08f88e7e27c2bd49970cf7e6caf8c0971981716ce14a932d5`。用rustc edition2021链接C2 deps中的`libauto_ui_compiler-b4417c10966df5fe.rlib`及`libauto_ui_dialect-908bc1e788acc861.rlib`，`-L dependency=<C2>/debug/deps`及安装的windows_x86_64_msvc-0.52.6/lib。初稿缺trait import、直接rustc缺native库路径及输入model分隔符均修正后才计为生成证据；另一次隔离表达式编译遇serde crate identity冲突，不计产品失败/通过。

**F060-R6-02 / P2 / 知识交付清单缺口，AC-05、T-04/T-05。**

`git diff 220115df HEAD -- docs/specs`包含六个目标，而§5.6 SD-01..04及frontmatter只列四个runtime目标。提交`644d8366`还新增`docs/specs/build/desktop-api-surfaces.md`、修改`docs/specs/build/generated-vite-config.md`：包含per-app API/代理profile/进程所有权、路由挂载、npm并集及解析器迭代边界。这些是实际行为规范，不是缓存/索引；其中来源写PLAN-053，但writer中的053仍drafting、0步、未复审，不能把其标签当已批准/交付凭证。r6非目标仍写不做053，未说明这两个准备落地Spec的审核与归属。

最小关闭：work对照已实现/已授权的054兼容整合，补齐准确add/modify目标、before/after/理由/相关AC及影响头清单；明确053关系与实际验证归属，不替053伪造完成。若只是清单/归属补正而不改语义可保留r6；若需改变承诺或边界须按修订规约处理。review不代写语义、不删除已实现合同以规避。复验仅需六目标差异与所有权/既有行为证据核对；无需因此重跑全局用例。

**spec_inputs / 冻结候选delta，尚未通过整体交付审核：**

| target | reviewed SHA256 |
|---|---|
| docs/specs/runtime/native-codegen-semantics.md | 8f18b07c8f03dad21a3d9781fba0ed989ff26cc9aa1376037bccf1e23a5c5e54 |
| docs/specs/runtime/desktop-acceptance-input.md | df2df27b95062098a136be23d34abcdfed234e2fd810027a2cfa8f079020309d |
| docs/specs/runtime/native-command-frame-text.md | 570deba3bfe4d676d2249d3226c0e584ceecee374c4f39f5fe48655998f926e7 |
| docs/specs/runtime/component-and-store-instances.md | 54da3e882677bc8ef1fb2287d15c0033e927106912f6873d0e51e0b7ad74bb94 |
| docs/specs/build/desktop-api-surfaces.md | a92054257fd52e4da0be9059b59c58ec516b88771917602a2de8e40419313adb |
| docs/specs/build/generated-vite-config.md | c6e7be128ae678fa769b682a2661c177bd23aa16f7b8272080eebb897bdd9e99 |

读取并核对了当前四runtime候选规则：Http基址/JSON身份/动态载荷、实例回调消费与store回写、样式overflow转换、text变换与v1/v2 viewport/裁剪绘制边界均有代码及有区分力断言支持；未发现另一个已证实缺陷，不表示穷举所有类型。SD-01的安全索引承诺受F01影响，整体delta不通过；supersedes/new/touched头清单待F02补全，不写旧JSON ledger或派生索引。

**evidence / 本轮实际结果与复用边界：**

- `cargo test --locked -p auto-ui-compiler --lib p060_` exit0，11 passed/0 ignored/.16s；log `D:/autostack/plan-artifacts/plan-060/independent-compiler-r6.log` SHA256 `1f4893010b9280b28a68f4b718f9c8e596ded893d1369c36e5cf0d1dcd3d5610`。
- `cargo test --locked -p auto-ui-runtime --lib p060_` exit0，21 passed/0 ignored/.32s；log `independent-runtime-r6.log` SHA256 `387bd823d673d83b7067d801e4915d1acd9b2c6a18c6e52c376df55361e114ae`。含真实pipe能力门、WM client resize、native Action/Ctrl+A、实际HeadlessFrameSource→HostEndpoint滚轮与v1/v2回放/底部点击、实际WGPU mesh顺序、样式转换结构控制、HTTP真实TCP与错误控制。
- tools同范围首次3pass/1fail是测试继承跨检出CARGO_TARGET_DIR，被既有缓存根守卫拒绝，未计产品缺陷。原编译测试exe `auto_ui_tools-6649610c6339f3e5.exe p060_`在移除该环境变量后exit0，4 passed/0 ignored/.89s，含backend内容/文件集合/read-error及profile常量绑定。log `independent-tools-clean-env-r6.log` SHA256 `e8b116c7e62c09c646f366fe4f44df6630b5746d5076162785d18e44b834e0a4`。构建共享C2、jobs2/incremental0/devdebug0，事先协调；本轮启动的Cargo任务均已完成，未重建UI060 target。
- 独立重算`plan-054/t06/frozen-native-identity-r167.json`所列六app receipt SHA、published exe SHA、Cargo target exe SHA，18/18与各记录一致。来源提交/输入清单及v13身份与正式日志可核。r161 Notes/Calendar/Chat等oracle的既有SHA重算相符；亲阅`plan-054/t04/screenshots/native-chat-v13-paint-next-settled-r159.png`，最后128.25消息完整有色气泡/时间戳与滚动条可见。图片是原始观察复核，不冒称本轮现场驱动。
- 最新Vue sibling修复只移动module import，factory内实例保持；本轮11compiler控制包含zero/one/two sibling断言。r164完整4185模块Vue正式build及公开入口浏览器pageerror0/真实任务原始记录可复用，未按每次review身份重复现场任务。父054 mixed/failure/rollback完整验收及949新发现不在本回执冒称通过。
- `git diff --check 220115df HEAD` exit1仅报14处新增EOF空行；作为非阻断格式清理建议保留，未归为第三个功能finding。无其他实现写入，只有本Plan复审回执及状态/任务重开。

**pending_closeout_gates（仍属merge，不以其尚未执行作为本次needs_fix原因）：**

修复并重新最终pass后，merge owner须在新冻结版本核对code/依赖/config/输入并完成本仓受影响union：compiler rust生成及Vue store；dialect view/widget解析与嵌套；runtime session/desktop_protocol/headless/endpoint/message、broker canvas/viewport/input/close生命周期、mcp、native_http、back_proxy/dynamic/vm桥；tools rust_ui/merged_string_index、vue/run_vue/desktop_backends/proc_host。复用同源同配置的有效原子结果，仅补因修复或949依赖变化失效的组及必要集成闭合；此处不是新增全仓测试要求，也不是把auto-lang的cargo-t目录套到UI。最终生成物、所建/所发布/所测身份须一致；若SDK生成输出变化，应重生成/构建受影响正式app并复验受影响真实任务。父054自身完整门由054 owner执行。

其后尚需：完成准确六目标Spec审核与落地、revision-bound独立pass、949前置自身修复/复审、工具链交付提交及os054消费pin更新、正常归档与证据摘要折入Plan、按merge规约清理。没有满足这些步骤前，reviewed不等于已交付；本次明确未reviewed。保留不变面证据，T00/T02/T03完成，T01/T04重开，current_step=3/6。


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r6 | round: 70 | outcome: execution_done | findings: F060-R6-01/02 repaired | next: same independent final reviewer
  F060-R6-01: null_coalesce_rust keeps typed local/model/store Vec receivers intact and uses the existing production __at_idx for the first index. The real `.store.rows` Dot chain is classified by its exact store owner, not an arbitrary same-named JSON field. Remaining JSON segments keep Value semantics: owned lookup takes one actual field key (the old slice argument did not match the production helper), and negative JSON literals convert through signed i64 before usize. Every receiver/index appears once. No VM leniency or input protocol changes.
  Added permanent tools test p060_coalesced_indices_execute_safe_fallbacks: real parse/extract/generate; 45 generated expressions covering local/model/store/nested JSON/mixed Vec→field→index, each normal0/999/-1 × present/empty/missing-field, then a side-effectful index evaluated exactly once. It compiles and runs the real generated expressions with the production AT_BUILTIN_METHOD_SHIMS and exact current test-binary core/serde dependencies resolved from Cargo fingerprints; no GUI and no duplicated JSON/shim algorithm. Unsafe direct subscript is rejected before execution. Original red1/1fail; successive tests exposed real store traversal and JSON signature/signed-literal gaps, now fixed. One harness attempt used the unrelated standalone a2r_std module; a later harness lacked the Windows native import library search path; neither was counted as product pass. Final exact-dependency harness + mixed chain: actual1pass/0fail, execution2.376s; production-source edits after that run were indentation only.
  Related compiler controls actual11/11pass, .195s. Initial nextest attempt selected0 under default-filter and exit4, not counted; explicit --ignore-default-filter selected/executed all11 without changing repository filters. Tools5-case suite initially4pass/1fail because inherited foreign CARGO_TARGET_DIR was correctly rejected by the receipt guard; reran the same built test binary with that env removed: actual5pass/0fail/0ignored,2.26s. Final expanded mixed-expression test then1/1pass. Existing21 runtime/protocol/input tests and prior six-native/complete Vue task evidence have unchanged implementations; no repeat solely for this generation repair. Common final SDK/dependency impact and affected merge union remain pending, particularly repaired949 versus SDK167.
  F060-R6-02: header and §5.6 now exactly enumerate all six canonical Spec targets, adding build/desktop-api-surfaces and modifying build/generated-vite-config. Source053 provenance retained but integration/verification responsibility accurately assigned to060 for054; no claim that053 itself is reviewed or complete. Original runtime SD01..04 unchanged in scope; SD01 clarifies this safe coalescing boundary. T01/T04 closed under unchanged r6 semantics, execution5/6; T05 independent pass/merge/cleanup still open. No landing/push/archive/cleanup.

- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r6 | round: 71 | outcome: freshness repair verified; resume same independent review
  Parent final-consumer audit found the changed native coalescing output still used FRONT_LAYOUT_VERSION v13. Reviewer paused at clean a9afdd91 and confirmed the existing freshness contract requires an increment; no final pass issued. Bumped to v14 and changed the permanent receipt control to the actual prior v13. The targeted red run failed the identity-distinction assertion with current=old=v13 (exit101), not counted as a product pass. Final actual five-case tools test binary with foreign CARGO_TARGET_DIR removed: exit0,5pass/0fail/0ignored,2.56s. It accepts an unchanged current receipt, rejects oldv13 as needs-regeneration with identical inputs, and executes all45 generated coalescing expressions plus the once-only index control. No manual consumer-cache deletion is required or used to substitute for this invalidation. Raw logs plan-060/freshness-v13-red-r6.log, freshness-v14-final-r6.log and corresponding build logs. Final code differs from the passing run only in explanatory comments. Common SDK consumers must now emit v14 receipts. T05 final independent review and merge responsibilities remain pending.
  Evidence: D:/autostack/plan-artifacts/plan-060/coalesce-execute-red-r6.log SHA256 78f9796361ec26d959897154863d778b2df35558821ec5b021963260e6108c61.
  Evidence: D:/autostack/plan-artifacts/plan-060/coalesce-compiler-selected-r6.log SHA256 bcdc85e50d2fd9a7d3f1fb5e69fcf48940cf3b041c17424b34c6500fb8917ac5.
  Evidence: D:/autostack/plan-artifacts/plan-060/coalesce-tools-clean-env-r6.log SHA256 de7c21a5145cc6fed9bb6f66a116da2d780ab37ce3b5b3968f3def7fe823174b.
  Evidence: D:/autostack/plan-artifacts/plan-060/coalesce-mixed-final-r6.log SHA256 06aacb56fbaa1402343d1a9977ff8a13efbc649d8888a17bbd09bc43f6fb9b3e.


### 独立最终复审：F01/F02 修复与 v14 产物失效闭合（2026-11-06）

- stage: review | review_scope: final | reviewer_identity/limitations: 原独立子上下文`/root/review_ui060`，同一原writer续审；未参与实现，仅写本Plan；非GUI重演，复用不变面原始证据 | plan_id: auto-ui/PLAN-060 | plan_revision: 6 | outcome: pass（仅绑定下列冻结源码/依赖；不等于已合入或当前master整合已通过） | worktree_path: `D:/autostack/.wt/ui-060/auto-ui` | branch: `codex/plan-060-dev` | reviewed_commit: `f24dc98374daa55a0a5ee855ff5712f228782bda` | base_commit: `220115df62040f113bc689f0500af80fc11569a3` | dependency_revisions: sibling auto-lang clean detached `408ca8b5096df96d2a91831ea36ef25336781889` | next: merge owner核对最新主干/949整合影响，必要受影响复审及下列冻结联合门；不可凭本pass跳过。
- 初始/最终核对`git rev-parse HEAD`、`git status --short`、worktree/branch：指定原writer clean；本次修复起点a9afdd9180d2f524757ddd0f26d8877a8798ba22及最终f24均检查，未创建checkout、改实现或变更依赖。UI060 target仍不存在；复用已协调C2，所有本轮编译/测试均终态。主检出旧060与用户WIP未动。

| finding / acceptance_results | result | 独立关闭依据与保留边界 |
|---|---|---|
| F060-R6-01；AC-01，T-01/T-04 | closed / pass | null_coalesce_rust对typed Vec基座不再剥为JSON；首索引用正式__at_idx，后继Value字段逐键get_owned、JSON索引经signed i64→usize，保持一次求值。实际parse/extract/generate→编译并运行45个local/model/store/nested JSON/mixed链×正常/越界/负索引×存在/空/缺字段控制及side-effect index一次。原例与同不变式邻面闭合，不降G1。 |
| F060-R6-02；AC-05知识审核，T-04/T-05 | closed / pass | frontmatter准确列4个new+2个modify目标，SD01..06与实际六文件差异集合一致；新增两build目标before/after/理由/AC归属完整。053保留实现来源，060承担为054集成及实际验证责任；未称053已完成。没有新语义或改验收，r6保持。 |
| round71 freshness缺口；AC-04/产物身份 | closed / pass（代码与控制） | FRONT_LAYOUT_VERSION现v14；同inputs的正式旧v13 receipt通过生产front_needs_regeneration返回(false,true)，当前receipt返回(false,false)。独立重放receipt mutation用例pass；不能用手删consumer缓存替代。所建/所测v14消费者属于下列强制merge联合门，尚未声称本轮已交付。 |
| AC-02/03，T-02/T-03 | retained pass | 上轮真实协议能力/owning wid/no-placeholder fallback及native输入/viewport/scroll21控制、原始实际Notes任务均保留；本次没有runtime/协议实现变动，不重复。根零参/单字符串及inproc既有标量边界不扩张。 |
| AC-04，T-04及历史AC-07/08/09 | pass于本次work/code acceptance边界；新SDK消费仍pending | compiler相关11控制、tools5控制、生成执行及freshness通过。原r161六native真实任务/完整Vue证据作为此前已完成任务保留，不能冒充f24+新949/最新master的新SDK任务；其必要重新生成/构建/受影响真实任务由合入冻结门明确承接。 |
| AC-05 | pass | 原独立上下文最终源码/断言/原始结果及六Spec审核完成；与初次needs_fix历史同存，不覆盖未来代码。 |
| AC-06 / T-05剩余归档、pin、清理 | pending merge-owned | 只设reviewed/current_step5/6，T-05仍未勾；没有merge/push/archive、消费者更新或删除证据/缓存。 |

**独立实际复验与断言审查。**

1. `cargo test --locked -p auto-ui-tools --lib p060_coalesced_indices_execute_safe_fallbacks`在a9af冻结实现上exit0，1 passed/0 ignored/2.84s，48.62s构建。该test确实提取本轮生成赋值表达式，用生产AT_BUILTIN_METHOD_SHIMS及当前Cargo fingerprint精确对应core/serde库编译执行；没有重写替代算法或将只含字符串断言当运行通过。审读45项expected矩阵、实际rustc失败传播及程序assert传播。f24相对a9af只改布局版本、receipt控制和注释，不改变这些生成表达式/执行逻辑；因此这次执行对f24有效。log `D:/autostack/plan-artifacts/plan-060/independent-coalesce-repair-r6.log` SHA256 `55657dcb6b1733d51bf2b53ba34ecef4d4932d05346f2af49777ca0fdaacd719`。
2. 独立选择不同于work45项的同机制生成控制：`doc.outer.rows[first_index()].inner[second_index()].ok ?? <bool/int/str default>`，实际parse/extract/generate三个右值族；断言两索引各只出现一次、三次逐key get_owned、两次at_owned且字段顺序保留，exit0。此项只称生成断言，不称三族额外运行实证。scratch `independent-coalesce-siblings.rs` SHA256 `4e3c4a7b5aee7ba506fc20d97cbef50df251c09920ee03f02c06c666a8cfea08`；log同名前缀`.log` SHA256 `1b2220ca5e57eb26683e02509298e6ae80b9a7532bbe1de818f2b312be9c081b`。首次直接rustc因选错compiler对应dialect导致crate identity冲突，未执行；按当前compiler fingerprint精确解析dialect458c27896599f648后通过；不将环境冲突记产品红。
3. 当前v14正式已构建test binary仅运行`p060_native_receipt_tracks_embedded_backend_content_and_file_set`（移除外树CARGO_TARGET_DIR），exit0，1 passed/0 ignored/.69s；当前receipt/旧v13同inputs mutation均实际经过生产判断。log `independent-freshness-v14-r6.log` SHA256 `f8700b55d7e32c8b27ed060d09ff8d4e249abe577ee10b78ec84a27e151b815d`。核原始red `freshness-v13-red-r6.log` SHA256 `d315fb10cea3416e565c85181a6aaefe0f2b3c2045f179df0083b81e74daf413`：current=old=v13的identity区别断言确实失败；不是伪造receipt结果。核最终5case日志`freshness-v14-final-r6.log` SHA256 `8871932de60ed3c6e40e9372b3fcc712e6bf98ae6fe0aeb96984066187d9f9ae`，5/5、2.56s实际执行，不把早前0-selection或foreign-target拒绝混入pass。
4. 本次修复生成器相关compiler11/11既有原子结果复用；上轮runtime21/21、tools相邻profile/backend freshness/merged controls及原始SDK/用户任务结果按未变源码/验证输入边界保留。未重跑runtime/GC/全catalog，无GUI/MCP/OS激活。`git diff --check a9afdd91 f24dc983` exit0；原初提交既有EOF空行非阻断建议保留，不扩大修复。

**spec_inputs / 本次最终审核六目标候选快照（当前树准备diff，非现场发布）：**

| docs/specs/... target | reviewed SHA256 |
|---|---|
| docs/specs/runtime/native-codegen-semantics.md | f8422d0ad30a99c6eae430c126d3e5e37bac297fe0e2657e212efa7bb71524b4 |
| docs/specs/runtime/desktop-acceptance-input.md | df2df27b95062098a136be23d34abcdfed234e2fd810027a2cfa8f079020309d |
| docs/specs/runtime/native-command-frame-text.md | 570deba3bfe4d676d2249d3226c0e584ceecee374c4f39f5fe48655998f926e7 |
| docs/specs/runtime/component-and-store-instances.md | 54da3e882677bc8ef1fb2287d15c0033e927106912f6873d0e51e0b7ad74bb94 |
| docs/specs/build/desktop-api-surfaces.md | f9b305343213713a2101b749c1dca77df56f9639714e2ee7b2073df18b837099 |
| docs/specs/build/generated-vite-config.md | dfe637b4eaecfdc9f30ed6d0ea674daa5c44c3bbfaa2d718ec46e64d994acf9f |

SD01新段描述既有受支持local/model/store Vec及嵌套JSON的安全coalescing；不扩大任意Rust类型/完整路由器。SD05/06只校准来源/本次验证责任，无从drafting053推导已完成。其余已审核规则未改。规范中overflow的v13叙述是该先前修复的失效版本；本轮coalescing当前生产身份为v14，最终消费必须v14。touched_goals保留真实auto-os/PLAN-054；没有写legacy ledger或派生索引。

**pending_closeout_gates / 冻结影响并集，必须执行后才能落地：**

- 本pass只绑定f24与writer Lang408。最新UI master `7ac0455bf4cd743248749e4bc4eb9d0aed657266`包含056原子reload/057 Vue挂载store等变化，不能把本pass直接搬到rebase结果。merge owner须核patch等价/冲突、canonical六Spec并发差异及store/生命周期/生成行为影响，交受影响整合复审；057可能改变父r164旧Chat保留身份的观察，最终Vue真实消费需重新核实。本review不变基线、不代做该整合。
- auto-lang/949的新冻结/rebase依赖（父报告f03226e10）尚不属于本次408依赖证据；须消费其有效独立复审与受影响fixed-group联合门，再核UI集成。旧SDK167及其v13 app不是该新组合的通过证据。
- UI冻结受影响union继续保持：compiler rust生成/Vue store；dialect view/widget解析与嵌套；runtime session、desktop_protocol/headless/endpoint/message、broker canvas/viewport/input/close生命周期、mcp、native_http、back_proxy/dynamic/vm桥；tools rust_ui/merged_string_index、vue/run_vue/desktop_backends/proc_host。针对rebase/依赖/输出变化选择实际完整受影响组及必要集成闭合；保留有效同源同配置原子结果，禁止因换owner重复全仓套件，也不能把known/pending算绿。此为精确模块/路径并集，不把auto-lang gate命令泛套UI。
- 正式工具/宿主/应用须绑定最终共同源码/依赖/config/输入；生成器v14使现有v13 receipt自动失效，重生成并构建受影响六native/完整Vue消费面，重新核对receipt/target/published三方身份、Calendar/Notes/Todo/Blog/Chat/Database及057影响的真实任务、必要混合/失败控制。父054自己的完整gate由其owner完成，unit/code检查不能代替实际任务。
- 完成上述整合复审/冻结门后，才由merge处理工具链交付、OS消费pin、准确Spec落地、归档/证据摘要折入及授权清理；本次未做。无剩余已知work-owned finding；本pass未预批后来新增代码，发生语义变化须回work并复审受影响面。


- stage: merge -> work | plan_id: auto-ui/PLAN-060 | plan_revision: r6 | round: 73 | outcome: rebase integration pending | next: affected work/independent integration re-review
  Source-bound independent pass at f24dc983 retained, Plan-only f97c9ec8 saved in refs/plan-recovery/060-reviewed-r6. Normal rebase onto master7ac0455b completed at f04f31bd4af159ad1432a9f2549afaaa007c9366. 130 commits replayed; no automatic landing. Overlaps resolved explicitly: back_proxy whole-body string binding retains reviewed ParamKind and strict UTF8 rejection rather than lossy decoding; Vue routed wrapper keeps main057 owned store context release; existing956 Rust build/publish/cwd Spec retained; session attach diagnostics keep existing055 equivalent diagnostics; Hello optional-tail logic identical, both055/060 tests kept; generated theme_pkg_static and per-app http_base both retained; existing956 backend build test and new060 generated-execution test both retained. No unrelated default feature deletion. Full range-diff retained plan-060/rebase-range-diff-r6.txt; not falsely claimed globally equal because already-landed duplicate behavior and integrations changed context.
  Canonical delta reconciled to new base: desktop-api-surfaces is now modify (already exists on master), generated-vite behavior already exists and only attribution changes. Six targets remain exact, original base recorded separately; no user-facing requirement/AC/task-count change. T04 reopened solely for new-main057 lifecycle/native-theme, updated949 dependencies and final formal v14 consumer integration; status executing4/6. Old task/atomic evidence retained where unchanged, new baseline must pass affected verification and independent integration review before merge.


- stage: work | plan_id: auto-ui/PLAN-060 | plan_revision: r6 | round: 74 | outcome: integration compatibility repaired; actual SDK consumer gate pending
  Updated read-only Lang sibling from408ca8b5 to reviewed/rebasedf03226e10, same dependency as parent consumer. Actual combined targeted work command selected54/executed54:53pass/1fail, no missing cases. New dependency exposed original merged_string_index adapter preserving character offsets while Lang956 canonical contract now uses UTF8 byte offsets; first failure empty needle/start100 got100 versusSDK-1. This was a real compatibility finding, not waived or counted as green. Shared adapter now delegates directly to the current SDK generic TryInto<usize> entry without narrowing, character slicing or copied bounds logic. Preserves wide backend int inputs, negative-to-zero policy, current inside-character forward clamp, past-end-1. Explicit independent expectations cover Chinese byte3/6, emoji byte4, huge i64 and past-end empty needle. Retained code-region rewrite excludes comments/string literals and emits helper once.
  Follow-up actual tools subset6/6pass, execution3.151s/preparation1m29: all5p060 tests plus956 real build/publish/cwd failure control. Includes real45generated coalescing combinations+once and current/oldv13 receipt controls. Original other48 compiler/runtime/protocol/VM-slice/Init checks remain unchanged and passed on the same updated source/dependency/config; their mechanisms do not depend on this tools-only helper. Full54 accounting therefore has explicit repaired/fresh or unchanged successful evidence, not a rerun of unaffected groups. Commands use --target-dir C2 with CARGO_TARGET_DIR absent; no foreign-cache guard waivers.
  Spec SD01 adds currentSDK byte-delegation boundary with956 provenance; no new language semantics, task count or user goal. v14 has not yet been published to a formal app; final v14 SDK will include this fix before its first consumer generation. T04 remains open/current4of6 for formal commonSDK native output identity, necessary real native/Vue consumers (including057 lifetime), independent integration review and eventual merge-owned frozen union. No final combined pass/landing/archive claimed.
  Evidence D:/autostack/plan-artifacts/plan-060/rebase-integration-work-r6.log SHA256 20c6fbce4f143b0b6226c20797fa91cf43f27679a136f2f94b620454cebb54d1.
  Evidence D:/autostack/plan-artifacts/plan-060/rebase-find-from-final-r6.log SHA256 c93992a5123914b1f27900c25dbf703cbb1de407d1b5cf3dbf5872de0392180b.


- stage:work | plan_id:auto-ui/PLAN-060 | revision:r7 | round:75 | outcome: existing Vue close lifecycle repaired, actual new-SDK green pending | code_commit:2f9a8b55364db32ba493a0118ed4568ebd998c2c | next:T04 current actual Vue8/native6+affected review
  Bounded existing-rule repair under original close/reopen AC04/T04, not a new host feature or per-window store promise. During finalSDK190 publicVue tasks, strengthened driver retained actualwindow DOM handle instead of assuming click==close. Header redbutton click did not remove the window. Explicit independent controls on same Paint window: actualpointer click +5seconds leaves oldDOMconnected/windowcountunchanged; visible footerclose then detaches exactly same oldDOM and decrementscount1. D:/autostack/plan-artifacts/plan-054/t04/wm-header-close-red-r190.json SHA8484699c5ecad4b1f25a662641b9e61260a78f84cabbe282db4d25538cd76948; both realframes retained. No product success inferred from old 400ms no-op. Notes/Todo/Calendar currentSDK save/data legs valid, but their attempted header close/reopen excluded until rerun; DB/Paint tasks explicitlyfailed and not completed oracles. Prior final/full r164 close assumptions do not stand for this missing DOM leg; valid footer lifecycle/mixed controls remain within exactoldboundary.
  Root cause in existing WM leaf: titlebar pointerdown always captures pointer; childclose had click.stop only. Capture retargets pointerup/click to titlebar, so close never reaches synchronous store.close (splice/unmount/containerremove). Single source fix in tools/assets/wm/VirtualWindow.vue adds pointerdown.stop on closebutton. No generated file patch, OS input synthesis, timeout waiver or footer-only workaround. Ordinary titlebar drag and eightresizehandles retain capture path, button click still runs existing close/focus handoff.
  SD05 now explicitly includes already-intended clickable close/unmount path; same six Spec targets, originalsix tasks/ACs retained. Revision7 names this discovered existing contract gap; T04 stillopen/current4of6, prior r6 source pass retained only for unchanged surfaces. Native FRONT_LAYOUT_VERSION remainsv14: this changes only the VueWM asset, not nativecomponent emission/protocol/runtime. Source/dependency equivalence and finalactual consumers will be recorded before finalindependentpass; no new all-family/GC tests or unrelated app rewrite. Real headerclose+DOMdetach, footer comparator, normalheaderdrag/resize and Blog fresh routedApp/Init are required in regenerated publicVue run. One-line event repair uses actual original tasks as regression, not an implementation-mirroring unit.
  Scope correction from independent reviewer retained: canonical vue-app-store-live-isolation/component-and-store-instances own contexts per createApp; desktop-api-surfaces explicitly excludes single-view per-window store lifetime. Blog wrapper creates/releases App/context; plain Chat may retain hostcontext B session. Record actualChatclose/recovery rather than invent a new logout-on-close contract. Unchanged057 lower resource assertions may be reused with source identity. FinalSDK190 had independently reviewed Web990/c36 fix and built3bins/all8roots; its actualBlog fullHTTP backend nowcompiled, but after this UIasset fix a new immutable SDK and cleanactualVue tasks are required, not relabelled190success.

- stage:work | plan_id:auto-ui/PLAN-060 | revision:r8 | round:76 | outcome: native carrier named-action integration finding reproduced and bounded repair | next:T04 current SDK native actual channel/tasks + affected review
  FinalSDK194 integration produced a real native named-action counterexample afterrebase055 carrier: Todo StoreInput/AddTodo queued but left frame/input/DB unchanged; pointer/key actualcreate/edit/complete/reopen/nextedit works. GeneratedApp.dispatch_named hascorrectvariant andinputtext staging, but generatedentry wrapsAppinNativeThemeCarrier::with_package; carrierComponentimpl forwards on/view/timers/key/state but omittednewdispatch_named, inheritedfalse. NativeNotesNewNote wasrequestedvia pointerandMsg; given thisconfirmedwrappergap, no nativeMsgdelivery success inferredfromthatstep. Functional pointer/key tasks retainedonlywithinthatboundary. No claim all194 native6tasks complete.
  Commoninvariant nativeinputAction musttraverse allcomponentwrappers to actualApp exactlyonce, preserving optionalstring andhandled/unhandled. RuntimeproductiongenericComponentwrapperaudit findsNativeThemeCarrier as relevantwrapper; otherComponentimpls are DynamicComponent(realimplementation) and testfixtures. Permanent p060_native_theme_carrier_forwards_named_actions invokesactualbare+embeddedcarrier withdirectinnerpositivecontrol, zeroarg,Unicode/emoji string,emptystring,unknown andmissingpayload controls. Pre-fixactualcargo test -p auto-ui-runtime --lib <name> --locked --target-dir producerC2 exits101/0pass1fail atNoArg,false; actualwrapperfailure not copiedhandleralgorithm. Minimaloverride delegatesinner result. Samecurrentworkdependencies runtimecarrier4/4pass0ignored,including3existing055themeapply/reject/preservebusiness controls; no theme/protocol/argumentABI redesign.
  Nativecachedartifactidentity containspackage/front-layout/producerroot, notruntime source; existingv14 appswithunchangedappsource wouldretain brokenwrapper behavior. Advanceformalnativeidentity tov15 so v13/v14receipts bothstale; existingpermanentfreshnesscase now explicitly mutates each priorversion andretainscurrentacceptedcontrol. Appmainwidgetshape/wiretag11 unchanged, butnewruntimeinputsemantics requirefreshnativebinary. CorrespondingSD01currentSpecversion andSD02wrapperroute madeexplicit, same6Spectargets/originalACtasks. Nativeformalrebuildandactualmessage-channelclosure pending; noSourcepass/mergetilldone.
  CurrentwriterdependencyWebhadremained37d whileSDK194actualconsumerc36. VerifiedwriterWebdetachedtrackedclean, safelyalignedexistingdependencytoc36(noextraWT/link), Down7c4/Langf032/UI1dba unchanged; record plan-060/writer-dependency-alignment-r7.json. Allcurrentcarrierred/greenareactualwriterCargo pathdependencies Webc36, notolddefault37d. No competingCargo whiletestsran; rawscope remains plan-060. Othermanifest/profile/core/GUI controls reusedonlywithinunchangedboundaries.
  Parent194currentVue all8actualtasks/trueheaderdetach+reopen+nextop,10mixedheadercloseone/sibling9/dataretention,closeall10+pageerrors[]/browser0; latestpublicfrontgen/assets2f9/1dbasourcebound. Thisnative-onlycarrierrepair doesnotchangeVuegenerator/assets orappfront; exactsource-equivalence willbeprovenfornextSDKinstead ofrelabellingruntimeframes. Native194sixformalbuildsexit0/18published-target-receiptidentities/67inputs verified (identity8890bf78a0973d39979e88a4c8ce1d5810144589310c17bb9a591f29a7a7d21b); NotesandTodo truepointer/keytasks+normalclose0 complete butnotnamedchannelorfinalSDKacceptance. Parentremainingnative4Rust+VM2stillpending. Native194root19420/Chatservice/Cargo/wrappernatural0,privateRQ18172controlled1/reaped; fourNotes/Todofirst/reopenedchildrenallnormal0,manifest2036c398/selectedportsabsent/foreign47236unchanged. NoOSactivation/CUA.
  r8当前最小工作检查已终结：native_theme_carrier::tests4/4exit0（含新增命名转发及原3主题控制），native receipt当前接受+旧v13/v14拒绝/原backend内容和文件集合邻面1/1exit0/.81s，writer实际Webc36。T04仍pending当前SDK live named通道和八native任务，不能用单元结果关闭。本文新增Spec句/缓存标识更新只落实原输入及所建即所测合同。


- stage:work | plan_id:auto-ui/PLAN-060 | revision:r8 | round:77 | outcome:execution_done5/6 | source_commit:9c009f70c5aca4eb0ced80f3b8a0ab2c32432086 | next:independent final code/AC/Spec review, merge-owned bounded union pending
  FrozenSDK195 exact8roots Langf032/UI9c/Webc36/Man2a9/OS39d, CLI a77cb353b5a79d088749e9a29dc382d79a9256ce1331005892b1e39a74f1c01a, tool454bf9020fef0f9bcaf22957ed3df3642a8b96c242af1a076aa30060bcff9a30, hostf9064cfec9ab35d1d19615b406ebd464d81ee7470c57b1c2eba5d9a6aeb2e4a3. All6native v15 formalbuilds0/18artifactbindings/67inputs verified; nativeidentity781ce3fa232a2d873aca7242678ec220a7872970fb0419cfd97819fc118da546. Actual195 named Todo StoreInput renders intendedChinese; AddTodo uniqueDBid1/inputcleared/count1; realedit/complete/close0/reopen/nextedit. Notes namedNewNote plusrealpointer/keySave/close0/reopen/nextSave id9rev3. Calendar namedNextMonth showsDecember, Today/create/edit/save pluspointerEdit/Delete, crossday eventev4deleted and allversionsabsent afterclose0/reopen; next ev5saved/reopened retained, oldev3byteexact. Database actualquery/reject/switch/close0/reopen/filter. Blog fullChinese create/preview/return/Save/close0/reopen/nextSave exacttitle/body/current/revision8. NativeChat realrecoveredA login/send, independentlegitimateB receives/replies, samekeyretryoneDBrow, C read[]/sendnot-a-participant; close0/freshlogin/history/nextsend; prior10exact. Paint256pixel Undo/Redo/Save/close/reopen/nextdraw/SVGexact; Charts CSVChinese3decimals, save/close/reopen/nextsave/sourceofflineexport, actualPNGbarheights+SVG verified. No API/DB prewrites for desktop business tasks.
  Permanent actual-parent gate verify_basic_app_assembly.py --profile iced --run t06/iced-run-r195.json and --profile vue --run t06/vue-run-r195.json each8required8pass0fail/exit0. Actual195Vue8fresh tasks/production4185modules/14.53s/exit0 and10window trueheaderclose lifecycle alreadyrecorded. Native10windowcloseNotes preserves other9sameWMids/allNotesTodoChatrows/CalendarBlogChartsfiles/PaintKV andClock/Mines surfaces; Minesbusinessstateexact, onlyvolatileupdated_ms normalized. Nativeall6finalchildren0; host38420/backendservice32896/Cargo19116+36652/backend9756/wrapper18292 natural0/reaped, privateRQ3288 controlled1; selected9247/17823absent, originalmanifest2036c398/foreign47236unchanged.
  Work-owned AC01..04 complete and T04closed; no open work finding, no scope reduction. Source repair controls carrier4/4 and freshnessv15 current/rejectv13,v14 retained; original48compiler/runtime and6tools evidence reuse only unchanged inputboundaries. ClarifiedTickWrap localdevtools wrapper is outside actualgeneratedbrokerchain; no futurewrapper claim. Earlyframes before asynchronoussettlement excluded (Calendar firstreopen shot showedNotes); correct secondnormalCalendarclose/reopen retainedev5/absenceev4 frame nowbound. FirstCharts import beforeVMready lostpath; afteractualsurface ready actualimport succeeds. Peerfixture initially asserted error instead ofdocumentederr; finalu8/u9complete. No productfailure hidden or evidence quota replaced. Final bounded compiler4/runtime7/tools6union belongsmerge after independentfinalreview; notclaimedpassedhere.
  Current packet hashes: iced cd232e671e0aa7a3c1e0637a5a98e055eb9343345756faf231ca442e1e2cc042; Vue 348aeaa52a3ca74a987f8e96570cb92d72913ad117c230ec5990c6eb013f3bd0; mixed b0679520d485f3d3379f27cd8be11d7b043533e252788e91381152bfe1867746. Raw single parent scope D:/autostack/plan-artifacts/plan-054; all durable commands/counts above.


### r8 正式独立最终复审（2026-11-07；不以early audit代替final）

- stage: review | review_scope: final | reviewer_identity/limitations: 原独立子上下文`/root/review_ui060`持续复审；未实施r7/r8，仅对本Plan有回执写权；本轮只读代码/原始数据/截图，不冒称重新现场驱动 | plan_id: auto-ui/PLAN-060 | plan_revision: 8 | outcome: pass | worktree_path: `D:/autostack/.wt/ui-060/auto-ui` | branch: `codex/plan-060-dev` | reviewed_commit: `314990bd67d1ebd32185a2497ba1edca91a40a19` | source_commit: `9c009f70c5aca4eb0ced80f3b8a0ab2c32432086` | base_commit: `7ac0455bf4cd743248749e4bc4eb9d0aed657266` | findings: none open | next: merge-owned bounded frozen union与最新main等价核对；随后落地/归档，不在review内执行。
- 原writer/branch与worktree list核实；入口及回执前HEAD/status clean，314相对9c仅Plan变动。consumer UI detached9c，implementation/Specs与writer对应source相同。writer实际Lang `f03226e1063e191af61dd119bc066ec60bf848ca`、Web `c36e5d09b15ee9c8dce0d34fb4b6f9f1b37e26e1`；旧Web37d依赖不再被误当当前闭合。最终SDK195共同8根：Down7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c、Langf032、Man2a9b16e50ec60bf8e591fcf5dae14226567ac32c、Muskde92b909171743fc29004eddc617699984cf94f5、OS39d75750478d0cf1a70f8a617f1bc0bb0cd7a4d3、UI9c、Webc36、jade-edit5fb1e9e28e5a7e379317b7dffb400bad6c94606d，按原始SDK root-binding逐项核对。

| acceptance_results / task | outcome | 证据与范围 |
|---|---|---|
| AC-01 / T-01 | pass | 原F060-R6-01已闭，local/model/store/Value/mixed安全coalescing正式生成编译运行45组合+索引一次；相关compiler11及字节find适配现SDK956控制保留。r7 WM和r8 carrier不改索引算法，未重复无关测试。 |
| AC-02 / T-02 | pass | SDK195 Notes真实pointer/key改标题正文并保存、正常close0/reopen/nextSave；原协议/viewport/scroll及paint不变控制保留，实际图/独立DB对应。Vue实际headerclose/drag/resize另按SD05及T04闭合。 |
| AC-03 / T-03 | pass | SDK195已使用真实NativeThemeCarrier包层：Todo named StoreInput产生可见中文、AddTodo唯一id1且清输入，随后pointer/key编辑/完成；Notes named NewNote、Calendar named NextMonth/Today也有实际状态。新增delegate不改变wire能力门/未知false/可选串；原queued无变化的SDK194失败不计通过。 |
| AC-04 / T-04 | pass | 同SDK195六native全部正式v15构建成功/三方身份一致；当前native8、公共Vue8实际任务各有可见图、持久化oracle和关闭重开下一次操作。Calendar初始异步误帧排除，采用第二次正常close/reopen ev5保留图；Blog完整title/body图和版本文件一致。Paint/Charts为iced inproc VM，不计入六Rust native数量。 |
| AC-05 / T-05 review部分 | pass | 本独立最终代码/契约/原始结果/Spec审核绑定r8与314/9c；r6 source-bound pass、r7/r8早期audit只作各自历史，不自动跨版本。 |
| AC-06 / T-05 merge部分 | pending | 归档/工具链pin/清理/最终bounded union由merge完成；reviewed5/6、末checkbox仍未勾，不声称已落地。 |
| 历史AC-07/08/09 | pass | Notes/Todo真实业务闭环、HTTP/typedJSON后端与错误区分控制、Chat合法双端/隔离/同意图重试保留；当前实际消费不以旧194/旧app pin替代。 |

**关闭规则与源码边界。**

- r7：WM close button `pointerdown.stop`阻止titlebar捕获，既有click.close继续同步splice/unmount。原实际red保持：同Paint oldDOM点击5秒仍连接，而footer comparator立即detach。SDK195重新正式生成/生产build4185 modules/14.53s、8任务均检查实际old ElementHandle脱离，再重开下一次操作；mixed10关Notes后9个旧siblings DOM与数据保留，close-all10均脱离/pageerrors[]。普通drag/resize实际路径保留。没有生成目录手改、footer-only workaround或以400ms静默等待假装关闭。
- r8：`NativeThemeCarrier<C>::dispatch_named`直接一次委托inner，保留bool与Option<&str>；bare/package、零参、Unicode、空字符串、unknown/缺串false及调用序列永久测试已核。独立early审计实际重放当前carrier4/4、0ignored/.07s，包含原三theme/state控制，源自9c未改，故此final可复用；work red0/1→green4/4及receipt1/1原始日志同存。v15实际生产freshness对当前receipt接受、同inputs的v13/v14逐个拒绝，替换后恢复当前；非删除缓存规避。全六SDK195 native均v15。没有增加tag/ABI或任意child RPC。
- 生产包装器审计：当前generated broker入口client_entry→HeadlessFrameSource/RqProjector直接持NativeThemeCarrier；两FrameSource直接调用组件dispatch_named。TickWrap仅本地iced run_app/devtools分支，不在此broker链；DevToolsWrapper不是该Component包层、FaceProjector为ShellSurface。没有把此审计外推为未来所有wrapper已通过。
- 057范围保持：每createApp/context及最后owner退出，不承诺plain虚拟窗口关闭即注销。Blog routed wrapper确实创建/卸载App/context，fresh Init与下一次操作有实际证明；plain Chat可能保留DesktopVueApp context的B会话，按实际关闭/重开/合法恢复记录，不增“每窗强制fresh login”义务。r8相对r7 Vue assets/generator/store/run_vue字节不变；本轮仍有新的SDK195实际Vue8证据，不靠该等价性把194业务冒称195。

**正式构建与原始证据独立复核。**

- SDK195三个exe SHA实际重算3/3：CLI `a77cb353b5a79d088749e9a29dc382d79a9256ce1331005892b1e39a74f1c01a`；tool `454bf9020fef0f9bcaf22957ed3df3642a8b96c242af1a076aa30060bcff9a30`；host `f9064cfec9ab35d1d19615b406ebd464d81ee7470c57b1c2eba5d9a6aeb2e4a3`。`plan-054/t07/frozen-native-identity-r195.json` SHA `781ce3fa232a2d873aca7242678ec220a7872970fb0419cfd97819fc118da546`：六receipt/published/Cargo target18/18、artifact6/6、67实际源码input哈希独立重算全部吻合，所有generator=v15。Notesbb064ca/ Todoe915894/Calendar7dab8f2/Chat5cdca0f/Blog33df0fd/Databasedee81e0准确pin；对应真实app owner修复未转为UI业务特判。
- `plan-054/t06/iced-run-r195.json` SHA `cd232e671e0aa7a3c1e0637a5a98e055eb9343345756faf231ca442e1e2cc042`；Vue packet最终SHA `9c91aadc95570bc78e24251bfa44939b99b7c12fed25cd096de55360eb09db1d`。两packet的iced40个/Vue38个path/hash引用独立重算，无丢失/错配。Vue原SHA348aeaa52a3ca74a987f8e96570cb92d72913ad117c230ec5990c6eb013f3bd0仅`recorded_run`误留194文字；review指出后由artifact owner更正为实际195新run，cases/SDK/data/frames未改，validator重新8/8 exit0（vue-gate-metadata-r195.log），不重演也不伪称194续跑。
- 亲自只读SQLite/file验证：双轨Notes title/body/revision精确（native id9/rev3、Vue id1/rev3）；Todo id1精确中文文本/done1；Blog两轨meta/title/body/current与当前revision版本文件完全一致（native8、Vue4）；Calendar generation/rev符合、删除id全部版本文件及manifest条目缺席、next事件标题/保留正确；Database双fixture查询精确/禁止写入表不存在/文件哈希吻合；Charts两轨rows_frag/列映射一致、SVG可解析；Paint256像素与SVG、nativeKV保存像素完全一致。不是仅相信completed计数。
- Chat独立DB按目标conversation核对native3条、Vue5条逐字段精确；native全库共14条且14个intent key唯一，并非packet声明全库仅3条。native先前10条和首次peer的另一合法会话与最终u2/u8会话有清晰scope；初peer脚本误用error而非err的失败未冒称产品红/通过。原始peer oracle含同key重试仅一行、C read[]/send not-a-participant；秘密只在RAM worker，不读取或输出凭据。实际桌面发送未API/DB预写。
- 亲阅关键PNG而非只验SHA：native Todo下一次编辑/完成、Calendar二次reopen next-retained、Blog fulltitle/body、Chat重开后nextsend与peer回复、Charts三根实际高度；Vue Blog rev4、Chat五消息可见，均与对应数据相合。截图边界：Vue Blog next-save位于页面下部，完整正文由DOM精确值与独立版本文件补证；不冒称该图显示所有字段。原生Calendar控件局部拥挤为现存呈现边界，本案所验月视图/事件状态/交互oracle成立，未追加像素布局承诺。
- 两永久父gate `verify_basic_app_assembly.py --profile iced --run t06/iced-run-r195.json`及`--profile vue --run t06/vue-run-r195.json`原始输出均required8/passed8/failed0/exit0。它们是父054验收证据，本Plan只消费相关框架闭合，不代发054最终pass。原生混合关闭记录SHA `b0679520d485f3d3379f27cd8be11d7b043533e252788e91381152bfe1867746`；Vue mixed `230792ef4259bb1b7a706916eee059eb0503780f5fe30c5263a4a008228c3ab2`、close-all `6a64a3db3d220a943969cf9c0a7a432d3a1002c21a1e0b5bf24780b7fcf98440`。读取normal-stop：六native children正常0；根host/backend/service/Cargo/wrapper正常0/reaped，privateRQ明确controlled1/reaped；manifest恢复、selected listeners absent、foreign47236未处理。未把控制取消当自然0。

**spec_inputs / 六目标完整候选delta通过，review不发布canonical：**

| docs/specs/... target | reviewed SHA256 |
|---|---|
| build/desktop-api-surfaces.md | 29eb4ef325ca1321031cb0273fe16865fe16c349813f76eddc692b7cc111e1d1 |
| build/generated-vite-config.md | dfe637b4eaecfdc9f30ed6d0ea674daa5c44c3bbfaa2d718ec46e64d994acf9f |
| runtime/component-and-store-instances.md | 8961ae2f95604cfd7cc9937ee9a027f8e932db921515636e104c18fea823d3d9 |
| runtime/desktop-acceptance-input.md | 782db4447cb581f5a1e3261532d19b179a4eac86f8e8b39cb65fc891da0184a4 |
| runtime/native-codegen-semantics.md | 9811a2c1f3cddde4eb9a860a5f55ce5a8046751402eeb4484ca4de9095335a38 |
| runtime/native-command-frame-text.md | 8a0d879fc95cc8094ef4292422fa7523f9274cf423a62f6d23f88e34209f7117 |

完整`git diff 7ac0455b HEAD -- docs/specs` SHA256 `fd05eb2cc1089660d454a5356976f5ca56e66bfb2aa3f74884cd0f538b5e6d2b`。六目标与SD01..06/frontmatter一致：3新增、3修改；desktop-api已在新基线，修改归属/正常close合同而非重复添加，956服务构建/发布/cwd附节全文保留；057 owner与plain边界不削弱；SD01当前v15、956字节find委托、SD02carrier传播、SD03帧变换/有序裁剪、SD04canonical store/活动页单timer均与代码和可辨断言一致。没有用旧JSON账本/手工索引替代规范。

**pending_closeout_gates / merge责任，尚未声称执行或通过：**

1. 冻结完整影响范围仍为compiler rust/Vue store，dialect view/widget，runtime session/protocol/headless/viewport/input/close/mcp/native_http/backproxy/dynamic/vm桥，tools rust_ui/merged_index/vue/run_vue/desktop_backends/proc_host。既有48compiler/runtime原子结果、修复后的tools6、r8carrier4与freshness1按其不变源码/依赖/输入边界保留；r7仅WM资产无镜像unit，已由真实red/green业务闭合。最终缺项仅既已审定的小union，root在本pass后执行，不叠完整workspace/catalog：
   - **compiler4**：`ui_gen::rust::tests::named_style_and_typed_payload_rust_codegen`；`ui_gen::vue::tests::test_plan041_store_factory_and_context_facade`；`test_plan051_store_struct_default_kept`；`test_plan051_store_non_struct_defaults_unchanged`。
   - **runtime7**：`ui::desktop_protocol::native_theme_carrier::tests::`完整4项；`endpoint::tests::host_close_then_exit_handshake_then_reclaim`；`host::tests::reclaim_removes_win_app_surface_and_notifies`；`ui_tests::back_proxy_tests::p054_back_proxy_initializes_imported_constants_and_preserves_vars`。
   - **tools6**：`vue::plan053_probes::{desktop_window_root_clamps_page_not_nested_canvas,desktop_clients_with_same_store_keep_their_owners}`；`desktop_backends::tests::`完整3项；`rust_ui::plan036_theme_input_tests::theme_input_entries_follow_identity_and_reject_tamper`。
   用原writer实际Webc36/Langf032、`cargo nextest run --locked --target-dir <C2> -p <exact-package> --lib --ignore-default-filter -E <上述精确filter>`，CARGO_TARGET_DIR不继承；按实际枚举结果核对4/7/6，0选择/ignored不得称pass。055 Hello双控制与956 build/publish/cwd已在有效原子证据；057 archived浏览器lower资源oracle及未改生成/owner实现可复用，未重建归档057 scratch或启动其ignored SSE/browser。若新结果失败，返回具体受影响work/复审，不把pending归绿。
2. 当前main已至c5c6fe05e11d1266e6673a2778761035c1d0430d。本轮额外只读净差异7ac→c5：35文件，集中065/068计划、三个auto-graphics规范、experimental/autographics-node2d-pilot lower_node_ui/fixtures/generated-world/tests；生产`crates/`、Cargo.toml/lock、packages/scripts/WMassets/本Plan六Spec无交叠。未触060生成/输入/主题载体或SDK195实际业务路径。正常rebase若patch/source/config/input等价并无新冲突，则保留本次SDK195实际任务证据，不因commit换名重复任务；merge owner仍须对真实rebase结果做range-diff/六Spec核对，发现意外交叠再受影响复审。本pass不自动批准尚未生成的新commit。
3. 冻结union通过及合入等价核对后，merge才完成准确canonical落地、工具链/消费者pin、正式归档、证据摘要折入与授权清理。父054剩余步骤/最终复审属于其owner；本review没有merge/push/archive/更新消费者/动进程或foreign47236。T05 merge部分仍pending；本Plan状态reviewed5/6，不冒称目标整体已交付。

### merge 收口（2026-11-07，merge owner=auto-os/PLAN-054 授权链）

- stage: merge | plan_id: auto-ui/PLAN-060 | plan_revision: r8 | outcome: pass | checkpoints: prepared/landed/canonical_verified/archived/cleaned | delivery_commit: 见下
  - **merge-owned bounded union 全绿**（r8 复审 pending_closeout_gates 第1项，writer 实际 Webc36/Langf032，C2 目标 `D:/autostack/.wt/os-054/auto-lang/target/generated-consumers/c2e02416`，CARGO_BUILD_JOBS=2/INCREMENTAL=0/DEBUG=0/SCCACHE_DISABLE=1）：
    - compiler4：`final-compiler-r8.log` 4 run/4 passed/1157 skipped（named_style_and_typed_payload_rust_codegen、test_plan041_store_factory_and_context_facade、test_plan051_store_struct_default_kept、test_plan051_store_non_struct_defaults_unchanged）。
    - runtime7：`final-runtime-r8.log` 7 run/7 passed/2436 skipped（native_theme_carrier 全4 + host_close_then_exit_handshake_then_reclaim + reclaim_removes_win_app_surface_and_notifies + p054_back_proxy_initializes_imported_constants_and_preserves_vars）。
    - tools6：`final-tools-r8.log` 5 run/5 passed（plan053_probes 2 + desktop_backends 非 ignore 2 + theme_input 1）+ `final-tools-owned-backend-r8b.log` desktop_rust_backend_roundtrip_and_cleanup **1 run/1 passed/188.492s**（`--run-ignored all`，overlay profile r8b slow-timeout 300s×3；首轮 `final-tools-owned-backend-r8.log` 120s 超时为共享 C2 冷编译时长所致的诚实红，非产品失败，保留原日志）。desktop_backends::tests:: 完整 3 项全过。
  - **rebase 等价**：`final-rebase-range-diff-r8.log` 135 补丁全 `=`（旧 a1db3d79/2f9a8b55/9c009f70/314990bd 线 → 新 a4ea5a75/a568ed78/d1bcb2a7/25283216/cbbe4e69 线）；r8 复审第2项最新 main c5c6fe05e 净差异无生产 `crates/` 交叠。
  - **landed**：auto-ui master `--ff-only` 至交付提交（commit hash 见主检出 git log，基 c5c6fe05e）。
  - **archived/cleaned**：Plan 移入 docs/plans/archived/；wt-guard clean 后移除 .wt/ui-060/auto-ui 工作树与 codex/plan-060-dev 分支；plan-artifacts/plan-060 折叠后全删（上述日志/命令已折叠入本节），registry 刷新。
  - 工具链 pin：SDK195 三件套 CLI a77cb353…/tool 454bf902…/host f9064cfe… 即本交付源码（9c009f70/cbbe4e69 线）的正式构建；父 054 T-07/T-09 按此收口。
