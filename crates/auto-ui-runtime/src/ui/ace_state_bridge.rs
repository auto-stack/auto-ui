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
use ace_av::component_event_sessions::{
    ComponentEventSession, ComponentEventSessionProgram,
};

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
    fn to_field_value(&self) -> ComponentFieldValue {
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
}

/// 定向事件结果。
#[derive(Debug, PartialEq)]
pub enum DispatchOutcome {
    Delivered(Vec<(String, AceStateValue)>),
    Dropped(DropReason),
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EngineExecutionAudit {
    /// 引擎已交付的事件数（含负控车道）。
    pub events_delivered: u64,
    /// 会话侧真实 handler 入口数（只在执行器派发成功后增长）。
    pub handler_entries: u64,
    /// 会话链接数（每 mount +1；事件/读取/切换不得增长）。
    pub links: u64,
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
    initials: Vec<(String, InitialValue)>,
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
}

/// 组件面单场景实例上限（256^i 打包编码的可区分域；面级常量）。
const MAX_INSTANCES: usize = 4;

impl AceStateEngine {
    /// 一次真实编译（共享前门 + 唯一 verify + lower/admit/link 前探）校验
    /// 源形状，错误带具名诊断码。
    pub fn new(src: &str, module_path: &str, alias: LegacyWidgetAlias) -> Result<Self, String> {
        let program =
            ComponentEventSessionProgram::compile(src, module_path, alias)?;
        let compile_count = program.compile_count;
        Ok(Self {
            src: src.to_string(),
            module_path: module_path.to_string(),
            alias,
            program,
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
        let generation = session.generation();
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
                slot.initials = initials;
                slot.session = Some(session);
                slot.states = states;
            }
            None => self.slots.push(AceStateSlot {
                identity: identity.to_string(),
                live: true,
                generation,
                initials,
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
        match self.slot_mut(identity) {
            Some(slot) if slot.live => {
                if let Some(mut session) = slot.session.take() {
                    session.0.close();
                }
                slot.live = false;
                slot.initials.clear();
                slot.states.clear();
                UnmountOutcome::Released(slot.generation)
            }
            _ => UnmountOutcome::NotMounted,
        }
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
        let result = {
            let slot = self.slot_mut(identity).expect("live checked above");
            let session = slot.session.as_mut().expect("live slot holds a session");
            session
                .0
                .dispatch(&msg, &payload)
                .map(|_| {
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
            let Some((_, history)) = self
                .replay_history
                .iter()
                .find(|(hid, _)| hid == id)
            else {
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
            session
                .0
                .dispatch(&msg, &payload)
                .map(|_| {
                    session
                        .0
                        .last_committed_states()
                        .iter()
                        .map(|(n, v)| (n.clone(), v.clone()))
                        .collect::<Vec<(String, ComponentFieldValue)>>()
                })?
        };
        let program = ComponentEventSessionProgram::compile(
            &self.src,
            &self.module_path,
            self.alias,
        )?;
        self.audit.rebuild_compiles += 1;
        let rebuilt = program.open_session(&target)?;
        self.audit.links += 1;
        // 状态根替换（旧会话关闭——重建道的可观察事实就是身份更换）。
        let delivered_states: Vec<(String, AceStateValue)> = {
            let slot = self.slot_mut(identity).unwrap();
            if let Some(mut old) = slot.session.take() {
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
        audit.handler_entries = self
            .slots
            .iter()
            .filter(|s| s.live)
            .map(|s| s.session.as_ref().map(|x| x.0.audit().handler_entries).unwrap_or(0))
            .sum();
        audit.compile_count = self.program.compile_count;
        audit
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
            // 诊断注入车道的事件不计入生产计数：关闭时重定基到真实入口数
            //（增量不变量描述的是**生产道**的账）。
            let real: u64 = self
                .slots
                .iter()
                .filter(|s| s.live)
                .map(|s| s.session.as_ref().map(|x| x.0.audit().handler_entries).unwrap_or(0))
                .sum();
            self.audit.events_delivered = real;
        }
    }

    /// V-07 负控开关：每事件重编重建 + seed 补数道（真实 producer 调用）。
    #[doc(hidden)]
    pub fn debug_force_rebuild_lane(&mut self, on: bool) {
        self.force_rebuild = on;
        if !on {
            let real: u64 = self
                .slots
                .iter()
                .filter(|s| s.live)
                .map(|s| s.session.as_ref().map(|x| x.0.audit().handler_entries).unwrap_or(0))
                .sum();
            self.audit.events_delivered = real;
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
            },
        )
    }
}

/// 桥值 → 场景初值域（负控车道沿用；同一 int/bool/str 值域的拷贝映射）。
fn initial_of(v: &AceStateValue) -> InitialValue {
    match v {
        AceStateValue::Int(i) => InitialValue::Int(*i as i64),
        AceStateValue::Bool(b) => InitialValue::Bool(*b),
        AceStateValue::Str(s) => InitialValue::Str(s.clone()),
        AceStateValue::F32(f) => InitialValue::Float(*f),
    }
}
