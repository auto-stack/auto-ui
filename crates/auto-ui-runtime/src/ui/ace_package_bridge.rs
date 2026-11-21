//! PLAN-980 ④-A 首片 #2b：**AGE 包闭包的桌面桥引擎**。
//!
//! 组件面引擎（`AceStateEngine`）只吃单源；本引擎把"编译单元集合 + 入口"作为
//! 输入，dispatch 时按场景（实例数 + 定向 op）重编译，再用
//! `ace_av::age_package::observe_age_package_layout_inputs` 取 AV 真值——与 ④-A 首片 #2a 的
//! AV 观察同一条 `lower → MIR verify → ABC → AV` 链，不引入第二解释器。
//!
//! 首版边界（具名）：只观察 `int/i32` model 字段（`observe_age_package` 的
//! 状态面即该域）；包面场景**无 initials 面**，非空 props 种子具名拒；
//! Payload events are directed to the entry component and checked by the package front door.
use ace_av::age_component::{ComponentScenario, InitialValue, LegacyWidgetAlias, ScenarioOp};
use ace_av::age_package::{ExecLimits, WidgetViewItem};
use ace_core::atom_text::Span;
use ace_core::source::age_component::ComponentNode;
use ace_core::source::age_package::{compile_age_package_with_scenario, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

use crate::ui::ace_state_bridge::{initial_of, AceStateValue, DispatchOutcome};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};

mod frame;
pub use frame::{PackageAction, PackageFrame};

fn view_limits() -> ExecLimits {
    // A full checked view can materialize every FileTree row, its label and
    // payloads in one invocation. Keep that operation bounded too.
    ExecLimits { max_steps: 6_000_000, ..Default::default() }
}

/// 一个实例槽（keyed identity + 该实例的定向 op 历史）。
struct Slot {
    identity: String,
    live: bool,
    generation: u64,
    ops: Vec<ScenarioOp>,
    frame: Option<Arc<PackageFrame>>,
    initialized: BTreeSet<usize>,
    timers: BTreeMap<frame::PackageTimer, Instant>,
}

#[derive(Default)]
pub(super) struct PackageSnapshot {
    pub states: Vec<(String, AceStateValue)>,
    pub texts: Vec<String>,
}

/// AGE 包闭包的状态执行引擎（每 engine = 一个包工程；实例 = keyed 槽）。
pub struct PackageStateEngine {
    units: Vec<PackageModuleInput>,
    entry: String,
    entry_node: usize,
    event_anchor: Span,
    alias: LegacyWidgetAlias,
    message_spans: BTreeMap<String, Span>,
    slots: Vec<Slot>,
    next_generation: u64,
    checked_views: bool,
    startup_handlers: BTreeSet<usize>,
}

impl PackageStateEngine {
    /// 以一次试验编译（1 实例、零 op）校验工程形状；错误带具名诊断码。
    pub fn new(
        units: Vec<PackageModuleInput>,
        entry: &str,
        alias: LegacyWidgetAlias,
    ) -> Result<Self, String> {
        Self::new_with_root(units, entry, alias).map(|(engine, _)| engine)
    }

    pub(super) fn new_with_root(
        units: Vec<PackageModuleInput>,
        entry: &str,
        alias: LegacyWidgetAlias,
    ) -> Result<(Self, ComponentNode), String> {
        let compiled = compile_package(&units, entry, alias, 1, Vec::new())?;
        let checked_views = compiled.first_widget_ref().is_some();
        let (entry_node, component) = compiled
            .description
            .components
            .iter()
            .enumerate()
            .find(|(_, component)| component.module == entry)
            .ok_or_else(|| format!("entry module `{entry}` declares no component"))?;
        let startup_handlers = if checked_views {
            Self::initial_widget_handlers(&compiled, entry_node)?
        } else {
            BTreeSet::new()
        };
        let node = compiled.entry_node().ok_or_else(|| format!("entry module `{entry}` declares no component"))?;
        Ok((Self {
            units,
            entry: entry.to_string(),
            entry_node,
            event_anchor: component.head_span,
            alias,
            message_spans: component.messages.iter()
                .map(|message| (message.name.clone(), message.span)).collect(),
            slots: Vec::new(),
            next_generation: 1,
            checked_views,
            startup_handlers,
        }, node.clone()))
    }

    pub(crate) fn has_checked_view(&self) -> bool {
        self.checked_views
    }

    /// 试验/重放共用：按场景编译一次包闭包。
    fn compile(
        &self,
        instances: usize,
        ops: Vec<ScenarioOp>,
    ) -> Result<ace_core::source::age_package::AgePackageCompilation, String> {
        compile_package(&self.units, &self.entry, self.alias, instances, ops)
    }

    fn slot(&self, identity: &str) -> Option<usize> {
        self.slots.iter().position(|s| s.identity == identity)
    }

    /// 挂载 keyed 实例。包面场景**没有** props/initials 面：非空种子具名拒
    /// （不静默丢种子、也不假装已承接）。
    pub fn mount(
        &mut self,
        identity: &str,
        initials: Vec<(String, InitialValue)>,
    ) -> Result<u64, String> {
        if !initials.is_empty() {
            return Err(
                "the AGE package scenario has no props/initials surface yet; seed the model \
                 in source (package initials are a named pending surface)"
                    .to_string(),
            );
        }
        if let Some(i) = self.slot(identity) {
            if self.slots[i].live {
                return Err(format!("instance `{identity}` is already mounted"));
            }
        }
        let generation = self.next_generation;
        let mut slot = Slot {
            identity: identity.to_string(), live: true, generation,
            ops: Vec::new(), frame: None, initialized: BTreeSet::new(),
            timers: BTreeMap::new(),
        };
        if self.checked_views {
            slot.initialized = self.startup_handlers.clone();
            slot.ops.extend(self.startup_handlers.iter().copied().map(Self::init_op));
            self.refresh_frame(&mut slot)?;
        }
        self.next_generation += 1;
        if let Some(i) = self.slot(identity) {
            self.slots[i] = slot;
        } else {
            self.slots.push(slot);
        }
        Ok(generation)
    }

    pub(crate) fn unmount(&mut self, identity: &str) {
        if let Some(i) = self.slot(identity) {
            let slot = &mut self.slots[i];
            slot.live = false;
            slot.frame = None;
            slot.ops.clear();
            slot.initialized.clear();
            slot.timers.clear();
        }
    }

    pub fn package_frame(&self, identity: &str) -> Option<Arc<PackageFrame>> {
        self.slots.get(self.slot(identity)?)?.frame.clone()
    }

    fn initial_widget_handlers(compiled: &AgePackageCompilation, entry: usize) -> Result<BTreeSet<usize>, String> {
        if !compiled.description.components.iter().any(|node| node.handler_msgs.iter().any(|msg| msg == "Init")) {
            return Ok(BTreeSet::new());
        }
        let root = &compiled.description.components[entry];
        let syntax = compiled.bound.units.get(&root.module)
            .and_then(|unit| unit.age_components.iter().find(|(node, _, _)| node.name.name == root.name))
            .map(|(node, _, _)| &node.view);
        let unconditional = syntax.is_some_and(|view| {
            use ace_core::source::age_component::{view_contains_cond, view_contains_for};
            view.named_trees.is_empty()
                && view_contains_cond(&view.tree).is_none()
                && view_contains_for(&view.tree).is_none()
        });
        let mut visible = BTreeSet::from([entry]);
        if unconditional {
            let sites = compiled.widget_refs.get(&(root.module.clone(), root.name.clone()));
            visible.extend(sites.into_iter().flatten().filter_map(|site| {
                compiled.description.components.iter().position(|node| node.module == site.module && node.name == site.node)
            }));
        } else {
            // The checked entry decides which children mounted in a conditional
            // or repeated view. Inactive branches do not run Init.
            let items = ace_av::age_package::evaluate_package_entry_view(compiled, 0, &view_limits())?;
            let mut pending: Vec<_> = items.iter().collect();
            while let Some(item) = pending.pop() {
                let WidgetViewItem::Element(element) = item else { continue };
                if let Some(site) = frame::child_site(compiled, entry, element) {
                    if let Some(index) = compiled.description.components.iter()
                        .position(|node| node.module == site.module && node.name == site.node)
                    {
                        visible.insert(index);
                    }
                } else {
                    pending.extend(&element.children);
                }
            }
        }
        visible.retain(|&index| compiled.description.components[index].handler_msgs.iter().any(|msg| msg == "Init"));
        Ok(visible)
    }

    fn init_op(node: usize) -> ScenarioOp {
        ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::Send { instance: 0, msg: "Init".into() }),
        }
    }

    fn init_visible(compiled: &AgePackageCompilation, visible: BTreeSet<usize>, slot: &mut Slot) -> bool {
        let before = slot.ops.len();
        for index in visible {
            if compiled.description.components[index].handler_msgs.iter().any(|msg| msg == "Init")
                && slot.initialized.insert(index)
            {
                slot.ops.push(Self::init_op(index));
            }
        }
        before != slot.ops.len()
    }

    fn refresh_frame(&self, slot: &mut Slot) -> Result<(), String> {
        let revision = slot.frame.as_ref().map_or(1, |frame| frame.revision + 1);
        loop {
            let compiled = self.compile(1, slot.ops.clone())?;
            let mut frame = frame::render(&compiled, self.entry_node, revision)?;
            if !Self::init_visible(&compiled, frame.mounted.clone(), slot) {
                // Only the published frame needs a state snapshot. A frame
                // that discovers a new child will be rebuilt after Init.
                frame.states = ace_av::age_package::observe_package_scalar_states(
                    &compiled, self.entry_node, 0,
                )?.into_iter()
                    .map(|(name, value)| (name, AceStateValue::from(&value)))
                    .collect();
                let now = Instant::now();
                slot.timers.retain(|timer, _| frame.timers.contains(timer));
                for timer in &frame.timers {
                    slot.timers.entry(timer.clone()).or_insert_with(|| {
                        now + Duration::from_millis(u64::from(timer.every_ms))
                    });
                }
                slot.frame = Some(Arc::new(frame));
                return Ok(());
            }
        }
    }

    /// 逐次重放：所有存活槽的 op 按槽序扁平化后编译场景，返回该实例的状态行
    /// （标量字段声明序，与 AV 的实际状态值一致）。
    pub fn read_all(&self, identity: &str) -> Option<Vec<(String, AceStateValue)>> {
        let inst = self.slot(identity)?;
        if !self.slots[inst].live {
            return None;
        }
        if self.checked_views {
            return self.slots[inst].frame.as_ref().map(|frame| frame.states.clone());
        }
        self.read_snapshot(identity).map(|snapshot| snapshot.states)
    }

    /// 该实例按声明顺序排列的文本叶值，包含未显示分支的槽位。
    pub fn texts(&self, identity: &str) -> Option<Vec<String>> {
        self.read_snapshot(identity).map(|snapshot| snapshot.texts)
    }

    pub(super) fn read_snapshot(&self, identity: &str) -> Option<PackageSnapshot> {
        let inst = self.slot(identity)?;
        if !self.slots[inst].live {
            return None;
        }
        let ops = self.flatten_ops();
        let compiled = self.compile(self.slots.len().max(1), ops).ok()?;
        self.snapshot_for(&compiled, inst).ok()
    }

    /// 定向 unit 消息：追加 op → 重编译 → 观察（失败即具名拒，槽不被毒化：
    /// op 先入临时表，编译成功才提交）。
    pub fn dispatch(&mut self, identity: &str, msg: &str) -> Result<DispatchOutcome, String> {
        self.dispatch_event(identity, msg, None)
    }

    pub fn dispatch_with_args(
        &mut self,
        identity: &str,
        msg: &str,
        args: &[AceStateValue],
    ) -> Result<DispatchOutcome, String> {
        self.dispatch_event(identity, msg, Some(args))
    }

    pub(crate) fn dispatch_frame(
        &mut self,
        identity: &str,
        revision: u64,
        target: usize,
        coordinates: Option<(f32, f32)>,
    ) -> Result<(String, DispatchOutcome), String> {
        let frame = self.package_frame(identity).ok_or("checked view is not mounted")?;
        if revision != frame.revision {
            return Err("ace.desktop.widget-event-stale: view revision has changed".into());
        }
        let action = frame.actions.get(target).ok_or("unknown checked view event target")?;
        let motion = action.source.key.eq_ignore_ascii_case("onmousemove");
        let args: Vec<_> = match coordinates {
            Some((x, y)) if motion && x.is_finite() && y.is_finite() =>
                vec![AceStateValue::F32(x), AceStateValue::F32(y)],
            None if !motion => action.source.args.iter().map(AceStateValue::from).collect(),
            _ => return Err("ace.desktop.pointer: invalid host event coordinates".into()),
        };
        let index = self.slot(identity).expect("frame has a mounted slot");
        let outcome = self.dispatch_checked(index, action.node, &action.message, &args, action.anchor)?;
        Ok((action.message.clone(), outcome))
    }

    pub(crate) fn dispatch_due_timer(
        &mut self,
        identities: &[String],
        now: Instant,
    ) -> Option<Result<(usize, String, DispatchOutcome), String>> {
        let (instance, index, timer) = identities.iter().enumerate().find_map(|(instance, identity)| {
            let index = self.slot(identity)?;
            let slot = &mut self.slots[index];
            if !slot.live {
                return None;
            }
            let (timer, deadline) = slot.timers.iter_mut()
                .filter(|(_, due)| **due <= now)
                .min_by_key(|(_, due)| **due)?;
            *deadline = now + Duration::from_millis(u64::from(timer.every_ms));
            Some((instance, index, timer.clone()))
        })?;
        let anchor = Span { start: timer.anchor, end: timer.anchor };
        Some(self.dispatch_checked(index, timer.node, &timer.message, &[], anchor)
            .map(|outcome| (instance, timer.message, outcome)))
    }

    fn dispatch_checked(&mut self, index: usize, node: usize, message: &str, args: &[AceStateValue], anchor: Span)
        -> Result<DispatchOutcome, String>
    {
        let slot = &self.slots[index];
        let mut trial = Slot {
            identity: slot.identity.clone(), live: slot.live, generation: slot.generation,
            ops: slot.ops.clone(), frame: slot.frame.clone(), initialized: slot.initialized.clone(),
            timers: slot.timers.clone(),
        };
        let values: Vec<_> = args.iter().map(initial_of).collect();
        trial.ops.push(ScenarioOp::ForNode {
            node,
            op: Box::new(ScenarioOp::send_args_from_values(
                0, message, &values, Span { start: anchor.start, end: anchor.start },
            )),
        });
        self.refresh_frame(&mut trial)?;
        let states = trial.frame.as_ref().expect("successful render produced a frame").states.clone();
        self.slots[index] = trial;
        Ok(DispatchOutcome::Delivered(states))
    }

    fn dispatch_event(
        &mut self,
        identity: &str,
        msg: &str,
        args: Option<&[AceStateValue]>,
    ) -> Result<DispatchOutcome, String> {
        if !self.checked_views {
            return self.dispatch_snapshot(identity, msg, args)
                .map(|snapshot| DispatchOutcome::Delivered(snapshot.states));
        }
        let inst = self
            .slot(identity)
            .ok_or_else(|| format!("instance `{identity}` is not mounted"))?;
        if !self.slots[inst].live {
            return Err(format!("instance `{identity}` is released"));
        }
        self.dispatch_checked(inst, self.entry_node, msg, args.unwrap_or_default(), self.event_anchor)
    }

    pub(super) fn dispatch_snapshot(
        &mut self,
        identity: &str,
        msg: &str,
        args: Option<&[AceStateValue]>,
    ) -> Result<PackageSnapshot, String> {
        let inst = self.slot(identity).filter(|&inst| self.slots[inst].live)
            .ok_or_else(|| format!("instance `{identity}` is not mounted"))?;
        if args.into_iter().flatten()
            .any(|value| matches!(value, AceStateValue::F32(value) if !value.is_finite()))
        {
            return Err(format!("source.age.component-event: message `{msg}` carries a non-finite float"));
        }
        let op = match args {
            Some(values) => ScenarioOp::send_args_from_values(
                inst,
                msg,
                &values.iter().map(initial_of).collect::<Vec<_>>(),
                self.message_spans.get(msg).copied().unwrap_or(self.event_anchor),
            ),
            None => ScenarioOp::Send {
                instance: inst,
                msg: msg.to_string(),
            },
        };
        let directed = ScenarioOp::ForNode {
            node: self.entry_node,
            op: Box::new(op),
        };
        let mut ops = self.flatten_ops();
        ops.push(directed.clone());
        let compiled = self.compile(self.slots.len().max(1), ops)?;
        let snapshot = self.snapshot_for(&compiled, inst)?;
        // 只有在整条观察链成功后才把 op 提交进槽。
        self.slots[inst].ops.push(directed);
        Ok(snapshot)
    }

    fn flatten_ops(&self) -> Vec<ScenarioOp> {
        self.slots
            .iter()
            .filter(|s| s.live)
            .flat_map(|s| s.ops.iter().cloned())
            .collect()
    }

    fn snapshot_for(
        &self,
        compiled: &ace_core::source::age_package::AgePackageCompilation,
        inst: usize,
    ) -> Result<PackageSnapshot, String> {
        let mut observed = ace_av::age_package::observe_age_package_layout_inputs(
            compiled, self.slots.len().max(1),
        )?;
        let component = observed.components.get_mut(self.entry_node)
            .ok_or_else(|| "package observation produced no entry component".to_string())?;
        let row = component.instance_scalar_states.get_mut(inst)
            .ok_or_else(|| "package observation produced no state row".to_string())?;
        let states = std::mem::take(row).into_iter()
            .map(|(name, value)| (name, AceStateValue::from(&value))).collect();
        let texts = component.instance_texts.get_mut(inst).map(std::mem::take)
            .ok_or_else(|| "package observation produced no text row".to_string())?;
        Ok(PackageSnapshot { states, texts })
    }
}

fn compile_package(
    units: &[PackageModuleInput],
    entry: &str,
    alias: LegacyWidgetAlias,
    instances: usize,
    ops: Vec<ScenarioOp>,
) -> Result<AgePackageCompilation, String> {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario { instances, ops };
    compile_age_package_with_scenario(units, entry, alias, &caps, &scenario).map_err(|diagnostics| {
        diagnostics
            .first()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .unwrap_or_else(|| "package scene refused".to_string())
    })
}
