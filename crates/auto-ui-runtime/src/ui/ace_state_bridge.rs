//! PLAN-980 B-STATE：ACE 状态执行桥。
//!
//! PLAN-1106 T-05 起内部改接 **AV 增量事件会话**
//! （`ace_av::component_event_sessions`）：源在 `new` 时经共享前门与唯一
//! verify **编译一次**；每个挂载实例 = 一个独立会话（独立全局 cell 状态根，
//! 共享已验代码 `Arc`）；每个被接纳事件 = 恰一次目标 handler 真实派发
//! （不再按累计历史重编/重放）；状态读取 = cell 纯观察（零派发）；卸载 =
//! 关闭会话（状态根释放）。公共 API 面（mount/unmount/dispatch/read/
//! spawn_async 与全部枚举）保持 980/981 冻结形状。
//!
//! - 实例生命周期：keyed mount / unmount / 重挂新鲜 / 晚到事件丢弃（世代纪律，
//!   与 `vm_bridge` 的 ChildInstanceRecord 世代语义同构）；
//! - props：mount 时按实例种子初始字段值（`ComponentInitials` 语义 → 会话
//!   seed 覆盖）；
//! - 异步（MIX-03 无 HTTP 版）：ticket 在 ACE scheduler（968 面）真实执行到
//!   静止，完成回写只落在**存活且同世代**的实例上（对齐 CompletionPorts 的
//!   DropLate 语义）。
//!
//! 观察口（PLAN-1106 AC-06 的执行层 oracle）：[`EngineExecutionAudit`] 在
//! 真实调用边界计量——`handler_entries` 汇总会话审计（只在真实执行器派发
//! 成功后增长）、`replay_compiles` 只在负控车道真实调用旧场景 producer 时
//! 增长。`debug_force_replay_lane` 是 **V-06 负控专用**开关：开启后 dispatch
//! 改走旧场景重放道（真实 `observe_component_scenario_states` 调用——保留
//! 为显式离线 oracle 的同一 producer），外层计数/最终数字不变而执行层证据
//! 必然背离（`events_delivered > handler_entries` 且 `replay_compiles` 增长）
//! ——生产路径恒关，不构成在线 fallback。
//!
//! 边界（具名，不冒完整 STATE 面）：多单元源（host/store 内嵌跨引用）等
//! IF-981-MODULE；旧 bytecode 路径全面退役归 PLAN-987。本桥不解释 guest
//! 逻辑——状态/事件求值全部在 ACE 共享 verify + AV 内完成。

use ace_av::age_component::{
    self, ComponentFieldValue, ComponentInitials, ComponentScenario, InitialValue,
    LegacyWidgetAlias, ScenarioOp,
};
use ace_av::component_event_sessions::{ComponentEventSession, ComponentEventSessionProgram};

// 消费者（含集成测试）经本模块取面配置类型，不必直接依赖 ace-av。
pub use ace_av::age_component::{InitialValue as AceInitialValue, LegacyWidgetAlias as AceAlias};

/// UI 侧状态值（与组件面观察值同构）。
///
/// PLAN-981 F32 消费同窗（D-981-F32-CONSUMER-WINDOW）：F32 按**位相等**
/// （与 981 ComponentFieldValue::F32 同一策略：NaN payload 可判别、
/// -0.0 ≠ 0.0），故 PartialEq/Eq 手写而不是 derive（f32 无 Eq）。
#[derive(Clone, Debug)]
pub enum AceStateValue {
    Int(i32),
    Bool(bool),
    Str(String),
    F32(f32),
}

impl PartialEq for AceStateValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::F32(a), Self::F32(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl Eq for AceStateValue {}

impl From<&ComponentFieldValue> for AceStateValue {
    fn from(v: &ComponentFieldValue) -> Self {
        match v {
            ComponentFieldValue::Int(i) => AceStateValue::Int(*i),
            ComponentFieldValue::Bool(b) => AceStateValue::Bool(*b),
            ComponentFieldValue::Str(s) => AceStateValue::Str(s.clone()),
            ComponentFieldValue::F32(f) => AceStateValue::F32(*f),
        }
    }
}

impl AceStateValue {
    /// 桥值 → 会话载荷值（同一值域拷贝；int 域检查在 mount 侧）。
    pub(crate) fn to_field_value(&self) -> ComponentFieldValue {
        match self {
            AceStateValue::Int(i) => ComponentFieldValue::Int(*i),
            AceStateValue::Bool(b) => ComponentFieldValue::Bool(*b),
            AceStateValue::Str(s) => ComponentFieldValue::Str(s.clone()),
            AceStateValue::F32(f) => ComponentFieldValue::F32(*f),
        }
    }
}

/// 丢弃原因（可区分 oracle：晚到 ≠ 未挂 ≠ 世代过期）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropReason {
    UnknownIdentity,
    InstanceReleased,
    StaleGeneration,
    /// PLAN-108：同实例已有 Http 派发worker在途（重入拒绝；完成后再派）。
    HttpBusy,
}

/// 定向事件结果。
#[derive(Debug, PartialEq)]
pub enum DispatchOutcome {
    Delivered(Vec<(String, AceStateValue)>),
    Dropped(DropReason),
    /// PLAN-108（T-03c M2）：Http 效应派发已上 worker 线程在途；完成经
    /// [`AceStateEngine::poll_http_completions`] 回写（世代/存活核对，
    /// 迟到完成丢弃=卸载后迟到结果隔离）。本返回值不含状态。
    InFlight,
}

/// PLAN-108：在途 Http worker 派发。`SyncReceiver` 的 unsafe Sync 与
/// `SyncSession`（176 行）同纪律：一切访问经桥 RwLock 串行化，worker 侧
/// 只持 Sender。
struct SyncReceiver(std::sync::mpsc::Receiver<HttpWorkerResult>);
unsafe impl Sync for SyncReceiver {}

struct HttpPending {
    identity: String,
    generation: u64,
    msg: String,
    rx: SyncReceiver,
}

struct HttpWorkerResult {
    session: SyncSession,
    outcome: Result<Vec<(String, ComponentFieldValue)>, String>,
}

/// PLAN-108：Http 派发完成（桥逐步循环消费回写）。
#[derive(Debug)]
pub struct HttpCompletion {
    pub identity: String,
    pub generation: u64,
    pub msg: String,
    pub result: Result<Vec<(String, AceStateValue)>, String>,
}

/// 卸载结果（返回被释放实例的世代）。
#[derive(Debug)]
pub enum UnmountOutcome {
    Released(u64),
    NotMounted,
}

/// 异步 ticket：spawn 时钉住 (identity, mount generation)。
#[derive(Clone, Debug)]
pub struct AsyncTicket {
    identity: String,
    generation: u64,
    completion_msg: String,
    task_src: String,
}

/// 异步完成结果。
#[derive(Debug, PartialEq)]
pub enum AsyncOutcome {
    Completed(Vec<(String, AceStateValue)>),
    Dropped(DropReason),
}

/// spawn 拒绝（异步必须在实例存活时启动）。
#[derive(Debug)]
pub enum AsyncSpawnError {
    UnknownIdentity,
    InstanceReleased,
}

/// 执行层观察口（真实调用边界计量；V-02/V-06..08 的 oracle）。
///
/// **有效域（复审 F-2 冻结）**：`incremental_invariant_holds` 在**生产道
/// 全程**（含 unmount/关开）可对账——`handler_entries` 为存活会话入口数 +
/// 已卸载会话折入的 `retired_handler_entries` 累计；负控诊断注入道开启
/// 期间的 delivered 计数在旋钮关闭时重定基到真实入口数。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EngineExecutionAudit {
    /// 引擎已交付的事件数（含负控车道）。
    pub events_delivered: u64,
    /// 会话侧真实 handler 入口数（存活会话 + 已折入的卸载会话累计）。
    pub handler_entries: u64,
    /// 已卸载/被替换会话折入的真实入口数（不再出现在存活会话审计里）。
    pub retired_handler_entries: u64,
    /// 会话链接数（每 mount +1；事件/读取/切换不得增长）。
    pub links: u64,
    /// Real init entries on all mounted sessions.
    pub init_runs: u64,
    /// State roots released through the production unmount path.
    pub released_roots: u64,
    /// 程序编译次数（new 时 1；事件不得增长）。
    pub compile_count: u64,
    /// 负控车道对旧场景 producer 的真实调用次数（生产路径恒 0）。
    pub replay_compiles: u64,
    /// V-07 负控车道的真实重编次数（生产路径恒 0）。
    pub rebuild_compiles: u64,
}

impl EngineExecutionAudit {
    /// 增量不变量：每个交付事件恰一次真实 handler 入口。
    pub fn incremental_invariant_holds(&self) -> bool {
        self.events_delivered == self.handler_entries
    }
}

/// 会话的单线程纪律包装（与本文件族 `HeldLibrary` 的 unsafe Send/Sync 先例
/// 同族）：`ProgramSnapshot` 的内部可变性（全局 cell 表/init 位）与 str 值
/// 携带的非原子 arena 引用，只在**宿主静态 `RwLock` 串行化**的访问下触碰
/// （`BRIDGE` / `ASE_SESSIONS_HOST` 的写锁；窗口 tick 单线程）。跨线程从无
/// 并发引用——本包装只把该既有纪律声明给类型系统，不引入新的跨线程使用。
struct SyncSession(ComponentEventSession);
unsafe impl Send for SyncSession {}
unsafe impl Sync for SyncSession {}

struct AceStateSlot {
    identity: String,
    live: bool,
    /// mount 世代（每次 mount 取新值；重挂后旧 ticket 不再匹配）。
    generation: u64,
    /// 本实例的增量会话（独立状态根；live 期间常驻）。
    session: Option<SyncSession>,
    /// 最近一次观察到的状态（字段声明序；读取零派发）。
    states: Vec<(String, AceStateValue)>,
}

/// 单个组件源的状态执行引擎（每 engine = 一个组件定义；双 App = 双 engine））
///
/// PLAN-1106：内部持有**一次编译**的增量事件程序；每挂载实例一个会话。
pub struct AceStateEngine {
    src: String,
    module_path: String,
    alias: LegacyWidgetAlias,
    program: ComponentEventSessionProgram,
    slots: Vec<AceStateSlot>,
    /// V-06 负控：强制在线走旧场景重放道（生产路径恒关）。
    force_replay: bool,
    /// V-06 负控基线：开启时的当前状态（场景 initials）——重放道外层数字
    /// 与增量道保持一致的前提。
    replay_baseline: Vec<(String, Vec<(String, InitialValue)>)>,
    /// V-07 负控：每事件重编重建 + seed 补数道（生产路径恒关）。
    force_rebuild: bool,
    /// V-08 负控：读取/文本刷新意外触发业务 handler 派发（生产路径恒关）。
    force_getter_dispatch: bool,
    /// 负控车道的累计历史（identity → ops；只在 force_replay 开启时累积）。
    replay_history: Vec<(String, Vec<(String, Option<Vec<AceStateValue>>)>)>,
    /// 观察口计数。
    audit: EngineExecutionAudit,
    /// PLAN-108（T-03c M2）：宿主 HTTP provider（UI 桥注入；None=HttpCall
    /// 具名 fail-closed）。`SyncSession` 已 Send/Sync，派发可上 worker。
    http_provider:
        Option<std::sync::Arc<std::sync::Mutex<dyn ace_av::component_event_sessions::HttpProvider>>>,
    /// PLAN-108：本程序含 Http 效应（派发整体路由 worker 线程）。
    program_has_http: bool,
    /// PLAN-108：在途 Http 派发（每 identity 至多一个）。
    http_pending: Vec<HttpPending>,
}

/// 组件面单场景实例上限（256^i 打包编码的可区分域；面级常量）。
const MAX_INSTANCES: usize = 4;

impl AceStateEngine {
    /// 一次真实编译（共享前门 + 唯一 verify + lower/admit/link 前探）校验
    /// 源形状，错误带具名诊断码。
    pub fn new(src: &str, module_path: &str, alias: LegacyWidgetAlias) -> Result<Self, String> {
        let program = ComponentEventSessionProgram::compile(src, module_path, alias)?;
        let compile_count = program.compile_count;
        let program_has_http = program.has_http_effects();
        Ok(Self {
            src: src.to_string(),
            module_path: module_path.to_string(),
            alias,
            program,
            program_has_http,
            http_provider: None,
            http_pending: Vec::new(),
            slots: Vec::new(),
            force_replay: false,
            replay_baseline: Vec::new(),
            force_rebuild: false,
            force_getter_dispatch: false,
            replay_history: Vec::new(),
            audit: EngineExecutionAudit {
                compile_count,
                ..Default::default()
            },
        })
    }

    pub(crate) fn accepts_clock_tick(&self) -> bool {
        self.program.handlers().iter().any(|handler| {
            handler.msg == "Tick"
                && matches!(handler.payload.as_slice(), [spelling] if matches!(spelling.as_str(), "int" | "i32"))
        })
    }

    fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| s.live).count()
    }

    fn slot(&self, identity: &str) -> Option<&AceStateSlot> {
        self.slots.iter().find(|s| s.identity == identity)
    }

    fn slot_mut(&mut self, identity: &str) -> Option<&mut AceStateSlot> {
        self.slots.iter_mut().find(|s| s.identity == identity)
    }

    /// 挂载一个 keyed 实例（props = 初始字段值）。重挂已释放身份 = 全新实例
    /// （新世代、新会话）；已挂载身份重复挂载是契约错误。初值非法时回滚
    /// 挂载（引擎不被坏输入毒化——会话不开启）。
    pub fn mount(
        &mut self,
        identity: &str,
        initials: Vec<(String, InitialValue)>,
    ) -> Result<u64, String> {
        if let Some(slot) = self.slot(identity) {
            if slot.live {
                return Err(format!("instance `{identity}` is already mounted"));
            }
        }
        if self.live_count() >= MAX_INSTANCES {
            return Err(format!(
                "the component face holds at most {MAX_INSTANCES} live instances per engine"
            ));
        }
        // 初值 → 会话种子（int 域/类型由会话先验核对）。
        let mut seeds = Vec::with_capacity(initials.len());
        for (name, v) in &initials {
            let fv = match v {
                InitialValue::Int(i) => {
                    let Ok(small) = i32::try_from(*i) else {
                        return Err(format!(
                            "initial for `{name}` ({i}) is outside the i32 state domain"
                        ));
                    };
                    ComponentFieldValue::Int(small)
                }
                InitialValue::Bool(b) => ComponentFieldValue::Bool(*b),
                InitialValue::Str(s) => ComponentFieldValue::Str(s.clone()),
                InitialValue::Float(f) => ComponentFieldValue::F32(*f),
            };
            seeds.push((name.clone(), fv));
        }
        // 开会话（link + init + 可选 seed）：失败即不挂载（干净回滚）。
        let mut session = self.program.open_session(&seeds)?;
        // PLAN-108：Http provider 随会话（worker 派发时随会话上 worker 线程）。
        if let Some(provider) = self.http_provider.clone() {
            session.set_http_provider(provider);
        }
        let generation = session.generation();
        self.audit.init_runs += session.audit().init_runs;
        let states = session
            .last_committed_states()
            .iter()
            .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
            .collect();
        let session = SyncSession(session);
        match self.slot_mut(identity) {
            Some(slot) => {
                slot.live = true;
                slot.generation = generation;
                slot.session = Some(session);
                slot.states = states;
            }
            None => self.slots.push(AceStateSlot {
                identity: identity.to_string(),
                live: true,
                generation,
                session: Some(session),
                states,
            }),
        }
        self.audit.links += 1;
        // 负控车道历史同步清零（重挂 = 全新实例）。
        if let Some((_, ops)) = self
            .replay_history
            .iter_mut()
            .find(|(id, _)| id == identity)
        {
            ops.clear();
        }
        Ok(generation)
    }

    /// 卸载实例：关闭会话（释放状态根）；晚到事件/旧世代完成不再可见。
    pub fn unmount(&mut self, identity: &str) -> UnmountOutcome {
        let mut retired = 0u64;
        let mut released = false;
        let outcome = match self.slot_mut(identity) {
            Some(slot) if slot.live => {
                if let Some(mut session) = slot.session.take() {
                    retired += session.0.audit().handler_entries;
                    session.0.close();
                    released = session.0.audit().released;
                }
                slot.live = false;
                slot.states.clear();
                UnmountOutcome::Released(slot.generation)
            }
            _ => UnmountOutcome::NotMounted,
        };
        // 卸载把该会话的真实入口数折入累计账（增量不变量在关开全程可对账）。
        self.audit.retired_handler_entries += retired;
        self.audit.released_roots += u64::from(released);
        outcome
    }

    /// 定向事件：一次被接纳事件 = 恰一次目标 handler 真实派发（全部求值在
    /// ACE 内；桥只路由消息）。已释放/未知身份 → 丢弃（不复活、不重定向、
    /// 不诊断）。未声明消息/载荷错在会话面先验具名拒（零效果）。
    pub fn dispatch(&mut self, identity: &str, msg: &str) -> Result<DispatchOutcome, String> {
        self.dispatch_impl(identity, msg.to_string(), None)
    }

    /// PLAN-981 B-W1（IF-981-EVENT-PAYLOAD-B）：带载荷的定向事件——typed 值
    /// 按声明绑定到会话入口实参（唯一 verify 过的签名；错 type/arity 在效果
    /// 前拒）。已释放/未知身份照旧丢弃（世代/所有权纪律不变）。
    pub fn dispatch_with_args(
        &mut self,
        identity: &str,
        msg: &str,
        args: &[AceStateValue],
    ) -> Result<DispatchOutcome, String> {
        self.dispatch_impl(identity, msg.to_string(), Some(args.to_vec()))
    }

    /// PLAN-108（T-03c M2）：注入宿主 HTTP provider（UI 桥开窗时调用）；
    /// 已挂载的存活会话即时补注（worker 派发随会话携带）。
    pub fn set_http_provider(
        &mut self,
        provider: std::sync::Arc<std::sync::Mutex<dyn ace_av::component_event_sessions::HttpProvider>>,
    ) {
        self.http_provider = Some(provider.clone());
        for slot in self.slots.iter_mut() {
            if slot.live {
                if let Some(session) = slot.session.as_mut() {
                    session.0.set_http_provider(provider.clone());
                }
            }
        }
    }

    /// PLAN-108（T-03c M2）：收取在途 Http 派发结果。世代/存活核对后回写
    /// 会话与状态；已释放/重挂实例的结果**丢弃**（worker 会话随之释放——
    /// 卸载后的迟到结果隔离面）。每步可调（try_recv 不阻塞）。
    pub fn poll_http_completions(&mut self) -> Vec<HttpCompletion> {
        let pending = std::mem::take(&mut self.http_pending);
        let mut out = Vec::new();
        let mut still = Vec::new();
        for p in pending {
            match p.rx.0.try_recv() {
                Ok(res) => {
                    let accepted = matches!(
                        self.slot(&p.identity),
                        Some(slot)
                            if slot.live && slot.generation == p.generation && slot.session.is_none()
                    );
                    if accepted {
                        let slot = self.slot_mut(&p.identity).expect("checked above");
                        let states: Vec<(String, AceStateValue)> = match &res.outcome {
                            Ok(states) => states
                                .iter()
                                .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                                .collect(),
                            Err(_) => slot.states.clone(),
                        };
                        if res.outcome.is_ok() {
                            slot.states = states.clone();
                            slot.session = Some(res.session);
                        } else {
                            // 执行失败（含 http 错误）：事务回滚，会话仍存活。
                            slot.session = Some(res.session);
                        }
                        out.push(HttpCompletion {
                            identity: p.identity,
                            generation: p.generation,
                            msg: p.msg,
                            result: res.outcome.map(|_| states),
                        });
                    } else {
                        // 迟到完成：实例已释放或重挂——worker 会话整体丢弃，
                        // 结果不得落账（取消/迟到隔离见证面）。
                        self.audit.events_delivered += 0;
                        out.push(HttpCompletion {
                            identity: p.identity,
                            generation: p.generation,
                            msg: p.msg,
                            result: Err(
                                "dropped: instance released or remounted before http completion (late result isolated)"
                                    .to_string(),
                            ),
                        });
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => still.push(p),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    out.push(HttpCompletion {
                        identity: p.identity,
                        generation: p.generation,
                        msg: p.msg,
                        result: Err("http worker terminated without a result".to_string()),
                    });
                }
            }
        }
        self.http_pending = still;
        out
    }

    fn dispatch_impl(
        &mut self,
        identity: &str,
        msg: String,
        args: Option<Vec<AceStateValue>>,
    ) -> Result<DispatchOutcome, String> {
        {
            let Some(slot) = self.slot(identity) else {
                return Ok(DispatchOutcome::Dropped(DropReason::UnknownIdentity));
            };
            if !slot.live {
                return Ok(DispatchOutcome::Dropped(DropReason::InstanceReleased));
            }
        }
        // V-06 负控车道：强制走旧场景重放（真实 producer 调用；外层结果
        // 形状一致，执行层证据背离由观察口暴露）。
        if self.force_replay {
            return self.replay_dispatch(identity.to_string(), msg, args);
        }
        // V-07 负控车道：每事件重编重建 + seed 补数（数字一致、真实编译/链接
        // 增长——prepare/init 证据必红）。
        if self.force_rebuild {
            return self.rebuild_dispatch(identity, msg, args);
        }
        let payload: Vec<ComponentFieldValue> = match &args {
            None => Vec::new(),
            Some(values) => values.iter().map(|v| v.to_field_value()).collect(),
        };
        // PLAN-108（T-03c M2）：含 Http 效应的程序——派发上 worker 线程
        // （provider 阻塞等待不冻结 UI；完成经 poll_http_completions 回写）。
        // 同实例重入拒绝（HttpBusy）。
        if self.program_has_http {
            if self.http_provider.is_none() {
                // 无 provider：落到同步道，HttpCall 具名 fail-closed（M1）。
            } else if self.http_pending.iter().any(|p| p.identity == identity) {
                return Ok(DispatchOutcome::Dropped(DropReason::HttpBusy));
            } else {
                let slot = self.slot_mut(identity).expect("live checked above");
                let Some(sync) = slot.session.take() else {
                    return Ok(DispatchOutcome::Dropped(DropReason::InstanceReleased));
                };
                let generation = slot.generation;
                let msg_for_worker = msg.clone();
                let payload_for_worker = payload.clone();
                let provider = self.http_provider.clone().expect("checked above");
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    http_dispatch_worker(sync, msg_for_worker, payload_for_worker, provider, tx)
                });
                self.http_pending.push(HttpPending {
                    identity: identity.to_string(),
                    generation,
                    msg,
                    rx: SyncReceiver(rx),
                });
                return Ok(DispatchOutcome::InFlight);
            }
        }
        let result = {
            let slot = self.slot_mut(identity).expect("live checked above");
            let session = slot.session.as_mut().expect("live slot holds a session");
            session.0.dispatch(&msg, &payload).map(|_| {
                session
                    .0
                    .last_committed_states()
                    .iter()
                    .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                    .collect::<Vec<(String, AceStateValue)>>()
            })
        };
        match result {
            Ok(states) => {
                self.audit.events_delivered += 1;
                self.slot_mut(identity).unwrap().states = states.clone();
                Ok(DispatchOutcome::Delivered(states))
            }
            Err(e) => Err(e),
        }
    }

    /// V-06 负控：旧场景重放道（`observe_component_scenario_states` 真实调用
    /// ——与离线 oracle 同一 producer）。保持外层结果形状（Delivered(states)）
    /// 与最终数字一致；观察口计 `replay_compiles`（每 dispatch 一次真实编译）
    /// 且**不**增 handler_entries——增量不变量在此必红。
    fn replay_dispatch(
        &mut self,
        identity: String,
        msg: String,
        args: Option<Vec<AceStateValue>>,
    ) -> Result<DispatchOutcome, String> {
        let live: Vec<String> = self
            .slots
            .iter()
            .filter(|s| s.live)
            .map(|s| s.identity.clone())
            .collect();
        if let Some(entry) = self
            .replay_history
            .iter_mut()
            .find(|(id, _)| *id == identity)
        {
            entry.1.push((msg, args));
        } else {
            self.replay_history
                .push((identity.clone(), vec![(msg, args)]));
        }
        let anchor = ace_core::atom_text::Span {
            start: 0,
            end: self.src.len(),
        };
        let mut ops = Vec::new();
        for (new_idx, id) in live.iter().enumerate() {
            let Some((_, history)) = self.replay_history.iter().find(|(hid, _)| hid == id) else {
                continue;
            };
            for (m, a) in history {
                ops.push(match a {
                    None => ScenarioOp::Send {
                        instance: new_idx,
                        msg: m.clone(),
                    },
                    Some(values) => ScenarioOp::send_args_from_values(
                        new_idx,
                        m.clone(),
                        &values.iter().map(initial_of).collect::<Vec<_>>(),
                        anchor,
                    ),
                });
            }
        }
        let scenario = ComponentScenario {
            instances: live.len(),
            ops,
        };
        let initials = ComponentInitials {
            per_instance: live
                .iter()
                .map(|id| {
                    self.replay_baseline
                        .iter()
                        .find(|(bid, _)| bid == id)
                        .map(|(_, vals)| vals.clone())
                        .unwrap_or_default()
                })
                .collect(),
        };
        // 真实旧 producer 调用（每事件一次整源重编 + 全历史重放）。
        self.audit.replay_compiles += 1;
        let observed = age_component::observe_component_scenario_states(
            &self.src,
            &self.module_path,
            self.alias,
            &scenario,
            &initials,
        )?;
        self.audit.events_delivered += 1;
        for (row_idx, id) in live.iter().enumerate() {
            if let Some(slot) = self.slot_mut(id) {
                slot.states = observed.per_instance[row_idx]
                    .iter()
                    .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                    .collect();
            }
        }
        let states = self
            .slot(&identity)
            .map(|s| s.states.clone())
            .unwrap_or_default();
        Ok(DispatchOutcome::Delivered(states))
    }

    /// V-07 负控：每事件重编重建 + seed 补数道。**真实 producer 调用**
    /// （`ComponentEventSessionProgram::compile` + `open_session`，seed=当前
    /// 状态补数）——最终数字与增量道一致，但每事件一次编译+链接+init：
    /// `rebuild_compiles`/`links` 增长、状态根身份每事件更换，执行层证据
    /// 必红（增量不变量 events == handler_entries 同样破裂——重建道不执行
    /// handler）。
    fn rebuild_dispatch(
        &mut self,
        identity: &str,
        msg: String,
        args: Option<Vec<AceStateValue>>,
    ) -> Result<DispatchOutcome, String> {
        // 先按正常道执行一次拿目标终态（数字正确性由真实语义保证），再
        // 把该终态作为 seed 重建（重编+重链+重 init——真实 producer 调用）。
        let target = {
            let payload: Vec<ComponentFieldValue> = match &args {
                None => Vec::new(),
                Some(values) => values.iter().map(|v| v.to_field_value()).collect(),
            };
            let slot = self.slot_mut(identity).expect("live checked above");
            let session = slot.session.as_mut().expect("live slot holds a session");
            session.0.dispatch(&msg, &payload).map(|_| {
                session
                    .0
                    .last_committed_states()
                    .iter()
                    .map(|(n, v)| (n.clone(), v.clone()))
                    .collect::<Vec<(String, ComponentFieldValue)>>()
            })?
        };
        let program =
            ComponentEventSessionProgram::compile(&self.src, &self.module_path, self.alias)?;
        self.audit.rebuild_compiles += 1;
        let mut rebuilt = program.open_session(&target)?;
        if let Some(provider) = self.http_provider.clone() {
            rebuilt.set_http_provider(provider);
        }
        self.audit.links += 1;
        // 状态根替换（旧会话关闭——重建道的可观察事实就是身份更换）。
        let mut retired = 0u64;
        let delivered_states: Vec<(String, AceStateValue)> = {
            let slot = self.slot_mut(identity).unwrap();
            if let Some(mut old) = slot.session.take() {
                retired += old.0.audit().handler_entries;
                old.0.close();
            }
            slot.session = Some(SyncSession(rebuilt));
            slot.states = target
                .iter()
                .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                .collect();
            slot.states.clone()
        };
        self.audit.events_delivered += 1;
        self.audit.retired_handler_entries += retired;
        Ok(DispatchOutcome::Delivered(delivered_states))
    }

    /// 读单个字段（最近观察值；已释放实例读不到）。
    pub fn read_state(&self, identity: &str, field: &str) -> Option<AceStateValue> {
        let slot = self.slot(identity)?;
        if !slot.live {
            return None;
        }
        slot.states
            .iter()
            .find(|(n, _)| n == field)
            .map(|(_, v)| v.clone())
    }

    /// 读整行状态（字段声明序；已释放实例读不到）。纯观察：零派发。
    pub fn read_all(&self, identity: &str) -> Option<Vec<(String, AceStateValue)>> {
        let slot = self.slot(identity)?;
        if !slot.live {
            return None;
        }
        Some(slot.states.clone())
    }

    /// 文本叶求值（纯读：每叶一次派发、不执行业务 handler；会话文本面）。
    /// states-only 边界内的源（f-string 叶）具名不可用。
    pub fn read_texts(&mut self, identity: &str) -> Result<Vec<String>, String> {
        // V-08 负控：读取意外触发业务 handler（真实派发——静态画面背后的
        // 业务重跑在 handler_entries 上必红）。
        if self.force_getter_dispatch {
            let slot = self
                .slot_mut(identity)
                .filter(|s| s.live)
                .ok_or_else(|| format!("instance `{identity}` is not mounted"))?;
            let session = slot.session.as_mut().expect("live slot holds a session");
            let handlers = session.0.program_handlers_for_debug();
            if let Some(first) = handlers.first() {
                let payload: Vec<ComponentFieldValue> = first
                    .1
                    .iter()
                    .map(|spelling| match spelling.as_str() {
                        "int" | "i32" => ComponentFieldValue::Int(0),
                        "bool" => ComponentFieldValue::Bool(false),
                        "f32" | "float" => ComponentFieldValue::F32(0.0),
                        _ => ComponentFieldValue::Str(String::new()),
                    })
                    .collect();
                session.0.dispatch(&first.0, &payload)?;
                let states: Vec<(String, AceStateValue)> = session
                    .0
                    .last_committed_states()
                    .iter()
                    .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                    .collect();
                slot.states = states;
            }
        }
        let slot = self
            .slot_mut(identity)
            .filter(|s| s.live)
            .ok_or_else(|| format!("instance `{identity}` is not mounted"))?;
        let session = slot.session.as_mut().expect("live slot holds a session");
        session.0.read_texts()
    }

    /// 实例当前世代（存活时）。
    pub fn generation_of(&self, identity: &str) -> Option<u64> {
        self.slot(identity).filter(|s| s.live).map(|s| s.generation)
    }

    /// 执行层观察口（会话审计汇总 + 引擎级计数）。
    pub fn execution_audit(&self) -> EngineExecutionAudit {
        let mut audit = self.audit.clone();
        audit.handler_entries = self.current_real_handler_entries();
        audit.compile_count = self.program.compile_count;
        audit
    }

    /// 当前真实 handler 入口总数：存活会话 + 已折入的卸载/被替换会话累计
    /// （增量不变量在生产道全程——含 unmount/关开——可对账的口径）。
    fn current_real_handler_entries(&self) -> u64 {
        let live: u64 = self
            .slots
            .iter()
            .filter(|s| s.live)
            .map(|s| {
                s.session
                    .as_ref()
                    .map(|x| x.0.audit().handler_entries)
                    .unwrap_or(0)
            })
            .sum();
        live + self.audit.retired_handler_entries
    }

    /// V-06 负控开关：强制在线走旧场景重放道（真实 producer 调用）。
    /// **仅测试负控使用**——生产路径恒关；开启本身不改任何外层语义。
    #[doc(hidden)]
    pub fn debug_force_replay_lane(&mut self, on: bool) {
        self.force_replay = on;
        // 基线 = 开启瞬间的当前状态（作为场景 initials）——负控的外层数字
        // 一致性由此保持；关闭即清。
        self.replay_baseline = if on {
            self.slots
                .iter()
                .filter(|s| s.live)
                .map(|s| {
                    (
                        s.identity.clone(),
                        s.states
                            .iter()
                            .map(|(n, v)| {
                                (
                                    n.clone(),
                                    match v {
                                        AceStateValue::Int(i) => InitialValue::Int(*i as i64),
                                        AceStateValue::Bool(b) => InitialValue::Bool(*b),
                                        AceStateValue::Str(t) => InitialValue::Str(t.clone()),
                                        AceStateValue::F32(f) => InitialValue::Float(*f),
                                    },
                                )
                            })
                            .collect(),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        self.replay_history.clear();
        if !on {
            // 诊断注入车道的事件不计入生产计数：关闭时重定基到**累计**真实
            // 入口数（存活 + 已折入——增量不变量描述生产道全程的账）。
            self.audit.events_delivered = self.current_real_handler_entries();
        }
    }

    /// V-07 负控开关：每事件重编重建 + seed 补数道（真实 producer 调用）。
    #[doc(hidden)]
    pub fn debug_force_rebuild_lane(&mut self, on: bool) {
        self.force_rebuild = on;
        if !on {
            self.audit.events_delivered = self.current_real_handler_entries();
        }
    }

    /// V-08 负控开关：读取/文本刷新意外触发业务 handler 派发。
    #[doc(hidden)]
    pub fn debug_force_getter_dispatch_lane(&mut self, on: bool) {
        self.force_getter_dispatch = on;
    }

    /// 启动异步：记录完成消息与任务源，钉住当前世代（对齐"异步任务返回只写
    /// 仍存活实例"的完成端口语义）。身份必须存活。
    pub fn spawn_async(
        &mut self,
        identity: &str,
        completion_msg: &str,
        task_src: &str,
    ) -> Result<AsyncTicket, AsyncSpawnError> {
        match self.slot(identity) {
            None => Err(AsyncSpawnError::UnknownIdentity),
            Some(slot) if !slot.live => Err(AsyncSpawnError::InstanceReleased),
            Some(slot) => Ok(AsyncTicket {
                identity: identity.to_string(),
                generation: slot.generation,
                completion_msg: completion_msg.to_string(),
                task_src: task_src.to_string(),
            }),
        }
    }

    /// 等待异步完成：任务源先在 ACE scheduler 真实执行到静止（968 面：task/
    /// go/await/sleep、虚拟时钟），随后按世代纪律回写——只有存活且同世代的
    /// 实例收到完成消息。
    pub fn await_async(&mut self, ticket: AsyncTicket) -> Result<AsyncOutcome, String> {
        // 真实异步执行先行（无论回写是否会被丢弃，任务本身必须真实跑过）。
        let task_path = format!("{}.task.at", self.module_path);
        let run = ace_av::scheduler_run::run_task_source(&ticket.task_src, &task_path)?;
        if !run.success {
            return Err(format!(
                "async task did not succeed: {} (stdout: {})",
                run.outcome_debug, run.stdout
            ));
        }
        let dropped = match self.slot(&ticket.identity) {
            None => Some(DropReason::UnknownIdentity),
            Some(slot) if !slot.live => Some(DropReason::InstanceReleased),
            Some(slot) if slot.generation != ticket.generation => Some(DropReason::StaleGeneration),
            _ => None,
        };
        if let Some(reason) = dropped {
            return Ok(AsyncOutcome::Dropped(reason));
        }
        Ok(
            match self.dispatch(&ticket.identity, &ticket.completion_msg)? {
                DispatchOutcome::Delivered(states) => AsyncOutcome::Completed(states),
                DispatchOutcome::Dropped(reason) => AsyncOutcome::Dropped(reason),
                // PLAN-108：http 程序的完成消息落入 worker 车道——ticket 语义
                // 为同步回写，InFlight 视为 busy 丢弃（完成经逐步 poll 落账）。
                DispatchOutcome::InFlight => AsyncOutcome::Dropped(DropReason::HttpBusy),
            },
        )
    }
}

/// 桥值 → 场景初值域（负控车道沿用；同一 int/bool/str 值域的拷贝映射）。
pub(super) fn initial_of(v: &AceStateValue) -> InitialValue {
    match v {
        AceStateValue::Int(i) => InitialValue::Int(*i as i64),
        AceStateValue::Bool(b) => InitialValue::Bool(*b),
        AceStateValue::Str(s) => InitialValue::Str(s.clone()),
        AceStateValue::F32(f) => InitialValue::Float(*f),
    }
}

/// PLAN-108（T-03c M2）：宿主 HTTP provider（reqwest blocking）。**阻塞语义**：
/// 只在 worker 线程上调用（含 Http 效应的程序派发整体路由 worker——
/// `AceStateEngine` 的 http_pending 车道）；UI 线程不等待。
pub struct UiHttpProvider;

impl UiHttpProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ace_av::component_event_sessions::HttpProvider for UiHttpProvider {
    fn execute(&mut self, verb: &str, url: &str, body: Option<&str>) -> Result<String, String> {
        plain_http_request(verb, url, body)
    }
}

/// PLAN-108：极简 HTTP/1.1 客户端（std::net::TcpStream；无代理/TLS——
/// M2 闭合域=组件面探针的明文 http 慢端点；生产广度客户端另行立项）。
/// 阻塞语义：只在 worker 线程调用。
fn plain_http_request(
    verb: &str,
    url: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let method = match verb {
        "auto.http.get.v1" | "auto.http.get-json.v1" => "GET",
        "auto.http.delete.v1" => "DELETE",
        "auto.http.put.v1" | "auto.http.put-json.v1" => "PUT",
        "auto.http.patch.v1" | "auto.http.patch-json.v1" => "PATCH",
        _ => "POST",
    };
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("unsupported url scheme (plain http only): {url}"))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rfind(':') {
        Some(i) => (&authority[..i], authority[i + 1..].parse::<u16>().map_err(|e| format!("bad port: {e}"))?),
        None => (authority, 80u16),
    };
    let body = body.unwrap_or("");
    let content_type = if verb.ends_with("-json.v1") {
        "application/json"
    } else {
        "text/plain; charset=utf-8"
    };
    let mut stream = std::net::TcpStream::connect((host, port))
        .map_err(|e| format!("connect {host}:{port} failed: {e}"))?;
    std::net::TcpStream::set_read_timeout(
        &stream,
        Some(std::time::Duration::from_secs(120)),
    )
    .map_err(|e| format!("set_read_timeout failed: {e}"))?;
    let request = format!(
        "{method} {path} HTTP/1.1
Host: {authority}
Connection: close
Content-Type: {content_type}
Content-Length: {}

{body}",
        body.len()
    );
    use std::io::{Read, Write};
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("request write failed: {e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("response read failed: {e}"))?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let Some(header_end) = text.find("\r\n\r\n") else {
        return Err(format!("malformed response (no header terminator): {text:.120}"));
    };
    let header = &text[..header_end];
    let status: u32 = header
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("malformed status line: {header}"))?;
    let body_out = text[header_end + 4..].to_string();
    if (200..300).contains(&status) {
        Ok(body_out)
    } else {
        Err(format!("HTTP {status}: {body_out:.200}"))
    }
}

/// PLAN-108：Http worker 线程体（独立 fn——2021 disjoint capture 会经
/// tuple-struct 解构按字段捕获，独立 fn 保证 `SyncSession` 整体按值跨线程；
/// `SyncSession` 的 Send/Sync 与 176 行 unsafe 纪律同约：访问经桥 RwLock
/// 串行化）。
#[allow(clippy::type_complexity)]
fn http_dispatch_worker(
    sync: SyncSession,
    msg: String,
    payload: Vec<ComponentFieldValue>,
    provider: std::sync::Arc<
        std::sync::Mutex<dyn ace_av::component_event_sessions::HttpProvider>,
    >,
    tx: std::sync::mpsc::Sender<HttpWorkerResult>,
) {
    let SyncSession(mut session) = sync;
    session.set_http_provider(provider);
    let outcome = session
        .dispatch(&msg, &payload)
        .map(|_| {
            session
                .last_committed_states()
                .iter()
                .map(|(n, v)| (n.clone(), v.clone()))
                .collect::<Vec<(String, ComponentFieldValue)>>()
        });
    let _ = tx.send(HttpWorkerResult {
        session: SyncSession(session),
        outcome,
    });
}
