//! PLAN-980 B-STATE：ACE 状态执行桥。
//!
//! 用已交付的组件面（PLAN-979/980：AGE Schema → 场景脱糖 → 唯一共享 verify →
//! AV 真值）执行 widget 组件的 model 状态与事件，替代旧 `handler_codegen`
//! bytecode 合成路径**在本交付面上**的职责：
//!
//! - 实例生命周期：keyed mount / unmount / 重挂新鲜 / 晚到事件丢弃（世代纪律，
//!   与 `vm_bridge` 的 ChildInstanceRecord 世代语义同构）；
//! - props：mount 时按实例种子初始字段值（`ComponentInitials`）；
//! - 异步（MIX-03 无 HTTP 版）：ticket 在 ACE scheduler（968 面）真实执行到
//!   静止，完成回写只落在**存活且同世代**的实例上（对齐 CompletionPorts 的
//!   DropLate 语义）。
//!
//! 边界（具名，不冒完整 STATE 面）：多单元源（host/store 内嵌跨引用）等
//! IF-981-MODULE；旧 bytecode 路径全面退役归 PLAN-987。本桥不解释 guest
//! 逻辑——状态/事件求值全部在 ACE 共享 verify + AV 内完成。

use ace_av::age_component::{
    self, ComponentFieldValue, ComponentInitials, ComponentScenario, InitialValue,
    LegacyWidgetAlias, ScenarioOp,
};

// 消费者（含集成测试）经本模块取面配置类型，不必直接依赖 ace-av。
pub use ace_av::age_component::{InitialValue as AceInitialValue, LegacyWidgetAlias as AceAlias};

/// UI 侧状态值（与组件面观察值同构）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AceStateValue {
    Int(i32),
    Bool(bool),
    Str(String),
}

impl From<&ComponentFieldValue> for AceStateValue {
    fn from(v: &ComponentFieldValue) -> Self {
        match v {
            ComponentFieldValue::Int(i) => AceStateValue::Int(*i),
            ComponentFieldValue::Bool(b) => AceStateValue::Bool(*b),
            ComponentFieldValue::Str(s) => AceStateValue::Str(s.clone()),
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

struct AceStateSlot {
    identity: String,
    live: bool,
    /// mount 世代（每次 mount 取新值；重挂后旧 ticket 不再匹配）。
    generation: u64,
    initials: Vec<(String, InitialValue)>,
    /// 本实例的定向消息历史（按序；场景重放语义）。`None` = unit 消息
    /// （`Send`）；`Some(values)` = 带载荷派发（`SendArgs`，PLAN-981 B-W1）。
    ops: Vec<(String, Option<Vec<AceStateValue>>)>,
    /// 最近一次观察到的状态（字段声明序）。
    states: Vec<(String, AceStateValue)>,
}

/// 单个组件源的状态执行引擎（每 engine = 一个组件定义；双 App = 双 engine）。
pub struct AceStateEngine {
    src: String,
    module_path: String,
    alias: LegacyWidgetAlias,
    slots: Vec<AceStateSlot>,
    next_generation: u64,
}

/// 组件面单场景实例上限（256^i 打包编码的可区分域；面级常量）。
const MAX_INSTANCES: usize = 4;

impl AceStateEngine {
    /// 以一次试验编译（1 实例、零 ops、零初值）校验源形状，错误带具名诊断码。
    pub fn new(src: &str, module_path: &str, alias: LegacyWidgetAlias) -> Result<Self, String> {
        let scenario = ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        };
        // 试验编译走同一 observe 入口（含共享 verify），只丢弃结果。初值按
        // 一一覆盖契约给 1 实例的空覆盖（无种子）。
        let initials = ComponentInitials {
            per_instance: vec![Vec::new()],
        };
        age_component::observe_component_scenario_states(
            src,
            module_path,
            alias,
            &scenario,
            &initials,
        )?;
        Ok(Self {
            src: src.to_string(),
            module_path: module_path.to_string(),
            alias,
            slots: Vec::new(),
            next_generation: 1,
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
    /// （新世代、清零历史）；已挂载身份重复挂载是契约错误。初值非法时回滚
    /// 挂载（引擎不被坏输入毒化）。
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
        let generation = self.next_generation;
        self.next_generation += 1;
        let newly_pushed = self.slot(identity).is_none();
        match self.slot_mut(identity) {
            Some(slot) => {
                slot.live = true;
                slot.generation = generation;
                slot.initials = initials;
                slot.ops.clear();
                slot.states.clear();
            }
            None => self.slots.push(AceStateSlot {
                identity: identity.to_string(),
                live: true,
                generation,
                initials,
                ops: Vec::new(),
                states: Vec::new(),
            }),
        }
        if let Err(e) = self.refresh() {
            // 回滚：新槽移除；重挂槽回到已释放形态（旧世代值不再重要）。
            if newly_pushed {
                self.slots.retain(|s| s.identity != identity);
            } else if let Some(slot) = self.slot_mut(identity) {
                slot.live = false;
                slot.initials.clear();
                slot.ops.clear();
                slot.states.clear();
            }
            return Err(e);
        }
        Ok(generation)
    }

    /// 卸载实例：释放状态与历史；晚到事件/旧世代完成不再可见。
    pub fn unmount(&mut self, identity: &str) -> UnmountOutcome {
        match self.slot_mut(identity) {
            Some(slot) if slot.live => {
                slot.live = false;
                slot.ops.clear();
                slot.states.clear();
                UnmountOutcome::Released(slot.generation)
            }
            _ => UnmountOutcome::NotMounted,
        }
    }

    /// 定向事件：追加到该实例历史并重放（全部求值在 ACE 内；桥只路由消息）。
    /// 已释放/未知身份 → 丢弃（不复活、不重定向、不诊断）。未声明消息在
    /// ACE 面具名拒（`source.age.component-event`），失败回滚已入队消息。
    pub fn dispatch(&mut self, identity: &str, msg: &str) -> Result<DispatchOutcome, String> {
        self.dispatch_impl(identity, msg.to_string(), None)
    }

    /// PLAN-981 B-W1（IF-981-EVENT-PAYLOAD-B）：带载荷的定向事件——typed 值
    /// 抬升进场景（唯一 verify、一次求值、按位绑定）。队列**拥有**载荷值
    /// （`AceStateValue` 拷贝入槽，不把临时 borrow 跨 tick/close 存队列）；
    /// 错 type/arity 在效果前拒并回滚（旧 dispatch 的原子性沿同一路径）；
    /// 已释放/未知身份照旧丢弃（世代/所有权纪律不变）。
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
        self.slot_mut(identity).unwrap().ops.push((msg, args));
        if let Err(e) = self.refresh() {
            self.slot_mut(identity).unwrap().ops.pop();
            return Err(e);
        }
        let states = self.slot(identity).unwrap().states.clone();
        Ok(DispatchOutcome::Delivered(states))
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

    /// 读整行状态（字段声明序；已释放实例读不到）。
    pub fn read_all(&self, identity: &str) -> Option<Vec<(String, AceStateValue)>> {
        let slot = self.slot(identity)?;
        if !slot.live {
            return None;
        }
        Some(slot.states.clone())
    }

    /// 实例当前世代（存活时）。
    pub fn generation_of(&self, identity: &str) -> Option<u64> {
        self.slot(identity).filter(|s| s.live).map(|s| s.generation)
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

    /// 重放当前存活实例集（mount/unmount/dispatch 后调用；全部求值在 ACE 内）。
    fn refresh(&mut self) -> Result<(), String> {
        let live: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.live)
            .map(|(i, _)| i)
            .collect();
        if live.is_empty() {
            return Ok(());
        }
        // 宿主值没有作者 span——anchor 取整源区间（arity/unsupported 诊断本就
        // 指向变体声明；type 诊断至少落在本源内，不用 0 冒充位置）。
        let anchor = ace_core::atom_text::Span {
            start: 0,
            end: self.src.len(),
        };
        let mut ops = Vec::new();
        for (new_idx, &slot_idx) in live.iter().enumerate() {
            for (msg, args) in &self.slots[slot_idx].ops {
                ops.push(match args {
                    // unit 消息保持旧 `Send`（等价空载荷）。
                    None => ScenarioOp::Send {
                        instance: new_idx,
                        msg: msg.clone(),
                    },
                    Some(values) => ScenarioOp::send_args_from_values(
                        new_idx,
                        msg.clone(),
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
                .map(|&i| self.slots[i].initials.clone())
                .collect(),
        };
        let observed = age_component::observe_component_scenario_states(
            &self.src,
            &self.module_path,
            self.alias,
            &scenario,
            &initials,
        )?;
        for (row_idx, &slot_idx) in live.iter().enumerate() {
            self.slots[slot_idx].states = observed.per_instance[row_idx]
                .iter()
                .map(|(n, v)| (n.clone(), AceStateValue::from(v)))
                .collect();
        }
        Ok(())
    }
}

/// 桥值 → 场景初值域（同一 int/bool/str 值域的拷贝映射）。
fn initial_of(v: &AceStateValue) -> InitialValue {
    match v {
        AceStateValue::Int(i) => InitialValue::Int(*i as i64),
        AceStateValue::Bool(b) => InitialValue::Bool(*b),
        AceStateValue::Str(s) => InitialValue::Str(s.clone()),
    }
}
